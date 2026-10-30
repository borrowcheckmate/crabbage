//! The container smoke test: starts the image hardened, waits for `/healthz` and removes it.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::sleep;
use std::time::{Duration, Instant};

use super::step::Step;

pub(super) const IMAGE: &str = "crabbage:local";
const CONTAINER: &str = "crabbage-smoke";
const CONTAINER_PORT: u16 = 3000;
const READY_TIMEOUT: Duration = Duration::from_secs(30);
const RETRY_INTERVAL: Duration = Duration::from_millis(500);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// The `docker run` step: a read-only, capability-free container with no privilege escalation,
/// a process and memory limit, a writable tmpfs data directory and the port published on
/// `host_port` of the loopback interface.
pub(super) fn start_step(host_port: u16) -> Step {
    let publish = format!("127.0.0.1:{host_port}:{CONTAINER_PORT}");
    Step::new(
        "docker",
        &[
            "run",
            "--detach",
            "--name",
            CONTAINER,
            "--read-only",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            "--pids-limit",
            "64",
            "--memory",
            "256m",
            "--tmpfs",
            "/data:uid=65532,gid=65532",
            "--env",
            "CRABBAGE_PUBLIC_URL=http://localhost:3000",
            "--env",
            "OIDC_ISSUER_URL=http://127.0.0.1:9",
            "--env",
            "OIDC_CLIENT_ID=smoke",
            "--env",
            "OIDC_CLIENT_SECRET=smoke",
            "--publish",
            &publish,
            IMAGE,
        ],
    )
}

/// The `docker rm -f` step that always runs after the container was started.
pub(super) fn remove_step() -> Step {
    Step::new("docker", &["rm", "--force", CONTAINER])
}

/// A loopback port that was free when asked.
pub(super) fn free_port() -> Result<u16, String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
    listener
        .local_addr()
        .map(|address| address.port())
        .map_err(|error| error.to_string())
}

/// Polls `/healthz` on `host_port` until it answers 200, for up to 30 seconds.
pub(super) fn wait_for_healthz(host_port: u16) -> Result<(), String> {
    let deadline = Instant::now() + READY_TIMEOUT;
    let last = loop {
        let status = healthz_status(host_port);
        if status == Ok(200) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            break status;
        }
        sleep(RETRY_INTERVAL);
    };
    Err(format!(
        "/healthz did not return 200 within {READY_TIMEOUT:?}, last result: {last:?}"
    ))
}

fn healthz_status(host_port: u16) -> Result<u16, String> {
    let address = format!("127.0.0.1:{host_port}");
    let mut stream = TcpStream::connect(&address).map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(REQUEST_TIMEOUT))
        .map_err(|error| error.to_string())?;
    write!(
        stream,
        "GET /healthz HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )
    .map_err(|error| error.to_string())?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| error.to_string())?;
    parse_status(&response).ok_or_else(|| format!("no status line in {response:?}"))
}

/// The status code on the first line of an HTTP/1.x response.
pub(super) fn parse_status(response: &str) -> Option<u16> {
    let mut parts = response.lines().next()?.split_whitespace();
    if !parts.next()?.starts_with("HTTP/1.") {
        return None;
    }
    parts.next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::{parse_status, start_step};

    #[test]
    fn container_runs_hardened_and_publishes_on_loopback() {
        let line = start_step(8080).to_string();
        for flag in [
            "--read-only",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            "--pids-limit 64",
            "--memory 256m",
            "--tmpfs /data:uid=65532,gid=65532",
            "--publish 127.0.0.1:8080:3000",
        ] {
            assert!(line.contains(flag), "{flag} missing from {line}");
        }
    }

    #[test]
    fn status_line_gives_the_code() {
        assert_eq!(parse_status("HTTP/1.1 200 OK\r\n\r\nok"), Some(200));
        assert_eq!(
            parse_status("HTTP/1.0 503 Service Unavailable\r\n"),
            Some(503)
        );
        assert_eq!(parse_status("HTTP/1.1 abc"), None);
        assert_eq!(parse_status("SSH-2.0-openssh"), None);
        assert_eq!(parse_status(""), None);
    }
}
