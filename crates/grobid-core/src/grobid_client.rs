//! GROBID client: Docker management and PDF → TEI conversion.

use std::path::Path;
use std::time::{Duration, Instant};

use crate::config;

/// How long to wait for a single Docker CLI invocation before giving up.
const DOCKER_CMD_TIMEOUT: Duration = Duration::from_secs(30);

/// How long to wait for `docker info` when probing the daemon.
const DOCKER_INFO_TIMEOUT: Duration = Duration::from_secs(15);

/// Check if the GROBID server is alive.
pub async fn is_grobid_alive() -> bool {
    let url = config::health_url();
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    match client.get(&url).send().await {
        Ok(resp) => resp.status().as_u16() == 200,
        Err(_) => false,
    }
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

/// Check whether the Docker daemon is reachable (not just the CLI installed).
async fn docker_daemon_ready() -> bool {
    docker_status(&["info".to_string()])
        .await
        .is_some_and(|s| s.success())
}

/// Start the GROBID Docker container if not already running.
pub async fn start_grobid() -> bool {
    if is_grobid_alive().await {
        tracing::info!("GROBID already running at {}", config::grobid_url());
        return true;
    }

    if which("docker").is_none() {
        tracing::warn!(
            "Docker not found. Start GROBID manually at {}",
            config::grobid_url()
        );
        return is_grobid_alive().await;
    }

    // The CLI being on PATH is not enough — bail out fast if the daemon is
    // down or wedged instead of blocking on `docker run`.
    if !docker_daemon_ready().await {
        tracing::error!(
            "Docker daemon is not reachable. Start Docker or run GROBID manually at {}",
            config::grobid_url()
        );
        return is_grobid_alive().await;
    }

    let container = config::grobid_container();
    let image = config::grobid_image();
    let port = config::grobid_port();

    // Remove any existing container.
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

    match docker_status(&run_args).await {
        Some(status) if status.success() => {}
        Some(status) => {
            tracing::error!(
                "Failed to start GROBID container (exit {:?})",
                status.code()
            );
            return false;
        }
        None => {
            tracing::error!("Timed out starting GROBID container {}", container);
            return false;
        }
    }

    tracing::info!("Starting GROBID container {}...", container);
    for i in 0..120 {
        if is_grobid_alive().await {
            tracing::info!("GROBID ready after {}s", i + 1);
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    tracing::error!("GROBID did not become ready in time");
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

    let part = reqwest::multipart::Part::bytes(pdf_bytes)
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

    let resp = client.post(&url).multipart(form).send().await;
    let elapsed = elapsed(started);

    match resp {
        Ok(resp) => {
            let status = resp.status().as_u16();
            if status != 200 {
                return (status, None, elapsed);
            }
            let tei = resp.text().await.unwrap_or_default();
            if tei.is_empty() {
                return (status, None, elapsed);
            }
            let out_path = tei_path_fn(pdf_path);
            if let Err(e) = std::fs::write(&out_path, tei.as_bytes()) {
                tracing::error!("Failed to write TEI {}: {e}", out_path.display());
                return (status, None, elapsed);
            }
            (status, Some(out_path), elapsed)
        }
        Err(e) => {
            tracing::error!("GROBID request failed: {e}");
            (0, None, elapsed)
        }
    }
}

fn elapsed(start: Instant) -> f64 {
    start.elapsed().as_secs_f64()
}
