# GROBID service

The CLI needs a running GROBID instance. It prefers one that is already up and
only starts its own Docker container as a fallback.

## Health check

`GET {grobid_url}/api/isalive` is polled before and during startup. The base
URL is `http://localhost:{GROBID_PORT}` (default port `8070`).

## Auto-start

When `isalive` fails and `docker` is on `PATH`, the CLI runs:

```bash
docker rm -f "$GROBID_CONTAINER"          # best-effort cleanup of stale container
docker run -d --rm --init \
  --ulimit core=0 \
  -e JAVA_TOOL_OPTIONS=-Xms1g -Xmx4g \
  -p "${GROBID_PORT}:8070" \
  --name "$GROBID_CONTAINER" \
  "$GROBID_IMAGE"
```

Notes:

- `--rm` + `--init` keep the container ephemeral and signal-safe.
- `JAVA_TOOL_OPTIONS` gives the JVM 1–4 GB heap; `ulimit core=0` disables core
  dumps. These mirror `docker-compose.yaml`.
- The container is removed when it stops; `docker rm -f` before `run` clears
  any leftover container of the same name.
- The CLI polls `isalive` once per second for up to **120 seconds** and reports
  "GROBID ready after Ns" on success.

If Docker is not installed, the CLI logs a warning and re-checks `isalive` —
useful when GROBID runs on another host or was started manually. If the CLI is
installed but the **daemon is unreachable**, `docker info` is probed with a
15-second timeout and the run exits `1` immediately rather than blocking on
`docker run`; all Docker invocations carry timeouts and the health check uses
a 5-second request timeout, so a wedged daemon cannot hang the CLI.

## Conversion endpoint

Each PDF is sent as `multipart/form-data` to:

```text
POST {grobid_url}/api/processFulltextDocument
```

Form fields:

| Field               | Value                          |
| ------------------- | ------------------------------ |
| `input`             | the PDF file                   |
| `consolidateHeader` | `1` (`GROBID_CONSOLIDATE_HEADER`)|

A `200` response returns the TEI XML body, which is written to
`tei/<pdf-name>.tei.xml`. Non-200 statuses are logged with the elapsed time
and counted as per-file failures.

## docker-compose.yaml

The repo-root compose file runs the same service for manual use:

```bash
docker compose up --build   # grobid on :8070
docker compose down -v      # stop and remove volumes
```

It is optional — the CLI auto-starts its own container when needed. Compose is
useful when you want a persistent GROBID with named volumes or a fixed port.

## Troubleshooting

| Symptom                              | Likely cause / fix                                   |
| ------------------------------------ | ---------------------------------------------------- |
| `GROBID is not available`            | Docker missing and nothing on `:8070`; start GROBID manually |
| `GROBID did not become ready in time`| First run pulls the image; re-run once pulled, or raise the wait by patching `start_grobid` |
| Port already in use                  | Set `GROBID_PORT` or stop the conflicting service    |
| Slow first conversion                | CRF models load on first request; subsequent calls are faster |
