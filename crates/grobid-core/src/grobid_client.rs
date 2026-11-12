//! GROBID client: Docker management and PDF → TEI conversion.

use std::path::Path;
use std::time::{Duration, Instant};

use crate::config;

/// How long to wait for a single Docker CLI invocation before giving up.
const DOCKER_CMD_TIMEOUT: Duration = Duration::from_secs(30);

/// How long to wait for `docker info` when probing the daemon.
const DOCKER_INFO_TIMEOUT: Duration = Duration::from_secs(15);

/// Short-timeout client for health probes; safe to reuse across poll loops.
fn health_probe_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

async fn alive(client: &reqwest::Client) -> bool {
    match client.get(config::health_url()).send().await {
        Ok(resp) => resp.status().as_u16() == 200,
        Err(_) => false,
    }
}

/// Check if the GROBID server is alive.
pub async fn is_grobid_alive() -> bool {
    alive(&health_probe_client()).await
}

/// Run a Docker CLI command with a timeout; returns the exit status on success.
async fn docker_status(args: &[String]) -> Option<std::process::ExitStatus> {
    let timeout = if args.first().map(String::as_str) == Some("info") {
        DOCKER_INFO_TIMEOUT
    } else {
        DOCKER_CMD_TIMEOUT
    };
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    match tokio::time::timeout(timeout, cmd.status()).await {
        Ok(Ok(status)) => Some(status),
        _ => None,
    }
}

/// Run a Docker CLI command capturing output; returns trimmed stdout on success
/// or the stderr/exit info as an error string for diagnostics.
async fn docker_capture(args: &[String]) -> Result<String, String> {
    let timeout = if args.first().map(String::as_str) == Some("info") {
        DOCKER_INFO_TIMEOUT
    } else {
        DOCKER_CMD_TIMEOUT
    };
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let output = match tokio::time::timeout(timeout, cmd.output()).await {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => return Err(format!("failed to spawn docker: {e}")),
        Err(_) => return Err("docker command timed out".to_string()),
    };
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(if stderr.is_empty() {
        format!("docker exited with {:?}", output.status.code())
    } else {
        stderr
    })
}

/// Check whether the Docker daemon is reachable (not just the CLI installed).
async fn docker_daemon_ready() -> bool {
    docker_status(&["info".to_string()])
        .await
        .is_some_and(|s| s.success())
}

/// Start the GROBID Docker container if not already running.
pub async fn start_grobid() -> bool {
    let probe = health_probe_client();
    if alive(&probe).await {
        tracing::info!("GROBID already running at {}", config::grobid_url());
        return true;
    }

    // A custom GROBID_URL implies a remote/external instance — never try to
    // manage a local process for it.
    if std::env::var("GROBID_URL").is_ok() {
        tracing::warn!(
            "GROBID_URL is set but {} is not reachable",
            config::grobid_url()
        );
        return false;
    }

    // Docker-free mode: a locally installed GROBID JVM service
    // (GROBID_LOCAL_BIN or the default ~/.local/share/grobid.rs path).
    if let Some(bin) = config::grobid_local_bin() {
        return start_local_service(&bin, &probe).await;
    }

    if which("docker").is_none() {
        tracing::warn!(
            "Docker not found. Start GROBID manually at {}",
            config::grobid_url()
        );
        return alive(&probe).await;
    }

    // The CLI being on PATH is not enough — bail out fast if the daemon is
    // down or wedged instead of blocking on `docker run`.
    if !docker_daemon_ready().await {
        tracing::error!(
            "Docker daemon is not reachable. Start Docker or run GROBID manually at {}",
            config::grobid_url()
        );
        return alive(&probe).await;
    }

    let container = config::grobid_container();
    let image = config::grobid_image();
    let port = config::grobid_port();

    // A container that is already running may simply still be booting —
    // killing it would restart the whole JVM warmup. Wait for it first.
    let inspect_args = vec![
        "inspect".to_string(),
        "-f".to_string(),
        "{{.State.Running}}".to_string(),
        container.clone(),
    ];
    if let Ok(state) = docker_capture(&inspect_args).await {
        if state == "true" {
            tracing::info!(
                "Container {} is running; waiting for GROBID to come up...",
                container
            );
            for _ in 0..90 {
                if alive(&probe).await {
                    return true;
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            tracing::warn!(
                "Container {} did not become ready; recreating it",
                container
            );
        }
    }

    // Remove any existing (stopped or wedged) container.
    let _ = docker_status(&["rm".into(), "-f".into(), container.clone()]).await;

    let run_args = vec![
        "run".to_string(),
        "-d".to_string(),
        "--rm".to_string(),
        "--init".to_string(),
        "--ulimit".to_string(),
        "core=0".to_string(),
        "-e".to_string(),
        "JAVA_TOOL_OPTIONS=-Xms1g -Xmx4g".to_string(),
        "-p".to_string(),
        format!("{}:8070", port),
        "--name".to_string(),
        container.clone(),
        image,
    ];

    if let Err(e) = docker_capture(&run_args).await {
        tracing::error!("Failed to start GROBID container {}: {e}", container);
        return false;
    }

    tracing::info!("Starting GROBID container {}...", container);
    for i in 0..120 {
        if alive(&probe).await {
            tracing::info!("GROBID ready after {}s", i + 1);
            return true;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    tracing::error!("GROBID did not become ready in time");
    false
}

/// Spawn the standalone GROBID JVM service (`grobid-service/bin/grobid-service`)
/// as a detached child process and wait for the health endpoint. The service
/// keeps running after this CLI exits, mirroring the Docker container's
/// lifecycle; stdout/stderr go to `grobid-service.log` next to the install.
async fn start_local_service(bin: &Path, probe: &reqwest::Client) -> bool {
    // Note: if an instance is already booting, a duplicate spawn simply fails
    // to bind the port and exits — wait_alive() still succeeds once the
    // original comes up.
    // Service log lives next to the grobid-service dir; fresh file per spawn.
    let log_path = bin
        .parent()
        .and_then(|p| p.parent())
        .map(|dir| dir.join("grobid-service.log"));
    if let Some(path) = &log_path {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::remove_file(path);
    }
    let open_log = || -> Option<std::process::Stdio> {
        let path = log_path.as_ref()?;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .ok()
            .map(std::process::Stdio::from)
    };
    let stdout = open_log();
    let stderr = open_log();

    let mut cmd = tokio::process::Command::new(bin);
    cmd.stdin(std::process::Stdio::null())
        .stdout(stdout.unwrap_or_else(|| std::process::Stdio::null()))
        .stderr(stderr.unwrap_or_else(|| std::process::Stdio::null()))
        // Detach: do NOT kill_on_drop — the service must outlive this CLI.
        .kill_on_drop(false);

    match cmd.spawn() {
        Ok(_) => {
            tracing::info!("Starting GROBID local service {}...", bin.display());
            wait_alive(probe, 180).await
        }
        Err(e) => {
            tracing::error!("Failed to spawn GROBID local service {}: {e}", bin.display());
            false
        }
    }
}

/// Poll the health endpoint for up to `secs` seconds.
async fn wait_alive(probe: &reqwest::Client, secs: u64) -> bool {
    for i in 0..secs {
        if alive(probe).await {
            tracing::info!("GROBID ready after {}s", i + 1);
            return true;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    tracing::error!("GROBID did not become ready in {secs}s");
    false
}

fn which(cmd: &str) -> Option<std::path::PathBuf> {
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(cmd);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Convert a PDF to TEI XML using the GROBID service.
pub async fn pdf_to_tei(
    client: &reqwest::Client,
    pdf_path: &Path,
    tei_path_fn: &dyn Fn(&Path) -> std::path::PathBuf,
) -> (u16, Option<std::path::PathBuf>, f64) {
    let started = Instant::now();
    let url = format!("{}/api/processFulltextDocument", config::grobid_url());

    let pdf_bytes = match std::fs::read(pdf_path) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("Failed to read PDF {}: {e}", pdf_path.display());
            return (0, None, elapsed(started));
        }
    };

    // One retry for transient failures (connection resets, GROBID 5xx);
    // 4xx responses are deterministic and not retried.
    for attempt in 0..2u32 {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        let part = reqwest::multipart::Part::bytes(pdf_bytes.clone())
            .file_name(
                pdf_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            )
            .mime_str("application/pdf")
            .unwrap_or_else(|_| reqwest::multipart::Part::text(""));

        let form = reqwest::multipart::Form::new()
            .text(
                "consolidateHeader",
                config::GROBID_CONSOLIDATE_HEADER.to_string(),
            )
            .part("input", part);

        let resp = match client.post(&url).multipart(form).send().await {
            Ok(resp) => resp,
            Err(e) => {
                tracing::error!("GROBID request failed (attempt {}): {e}", attempt + 1);
                if attempt == 1 {
                    return (0, None, elapsed(started));
                }
                continue;
            }
        };

        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        if status == 200 && !body.is_empty() {
            let out_path = tei_path_fn(pdf_path);
            if let Err(e) = std::fs::write(&out_path, body.as_bytes()) {
                tracing::error!("Failed to write TEI {}: {e}", out_path.display());
                return (status, None, elapsed(started));
            }
            return (status, Some(out_path), elapsed(started));
        }

        let snippet: String = body.chars().take(300).collect();
        tracing::error!(
            "GROBID returned {status} (attempt {}): {}",
            attempt + 1,
            if snippet.is_empty() {
                "<empty response>".to_string()
            } else {
                snippet
            }
        );
        // Retry server errors and empty responses; 4xx is deterministic.
        if (status < 500 && !body.is_empty()) || attempt == 1 {
            return (status, None, elapsed(started));
        }
    }
    (0, None, elapsed(started))
}

fn elapsed(start: Instant) -> f64 {
    start.elapsed().as_secs_f64()
}
