//! Runs the server binary to check its startup errors (a bad configuration, an address in use) and
//! its behaviour while the identity provider is unreachable.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

const VALID_CONFIG: [(&str, &str); 4] = [
    ("CRABBAGE_PUBLIC_URL", "https://recipes.example.com"),
    ("OIDC_ISSUER_URL", "https://id.example.com"),
    ("OIDC_CLIENT_ID", "crabbage"),
    ("OIDC_CLIENT_SECRET", "secret"),
];

fn run(command: &mut Command) -> Result<Output, String> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    while child
        .try_wait()
        .map_err(|error| error.to_string())?
        .is_none()
    {
        if Instant::now() >= deadline {
            child.kill().map_err(|error| error.to_string())?;
            return Err(format!("binary did not exit within {STARTUP_TIMEOUT:?}"));
        }
        sleep(Duration::from_millis(50));
    }
    child.wait_with_output().map_err(|error| error.to_string())
}

/// A running server that is killed when dropped, so a failing test leaves no process behind.
struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        drop(self.0.kill());
        drop(self.0.wait());
    }
}

#[expect(
    clippy::expect_used,
    reason = "test helpers stop the test on setup failure"
)]
fn free_loopback_address() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("port is free");
    listener.local_addr().expect("address is known").to_string()
}

/// The status code of a `GET` to `path` on `address`, or why the request failed.
fn status_of(address: &str, path: &str) -> Result<u16, String> {
    let mut stream = TcpStream::connect(address).map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(REQUEST_TIMEOUT))
        .map_err(|error| error.to_string())?;
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )
    .map_err(|error| error.to_string())?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| error.to_string())?;
    response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| format!("no status line in {response:?}"))
}

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("crabbage-binary-{}-{name}", std::process::id()));
        drop(std::fs::remove_dir_all(&path));
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        drop(std::fs::remove_dir_all(&self.0));
    }
}

fn server_command(data_dir: &TempDir, site_addr: &str, issuer_url: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_crabbage"));
    command
        .envs(VALID_CONFIG)
        .env("OIDC_ISSUER_URL", issuer_url)
        .env("CRABBAGE_DATA_DIR", &data_dir.0)
        .env("LEPTOS_OUTPUT_NAME", "crabbage")
        .env("LEPTOS_SITE_ADDR", site_addr);
    command
}

fn run_with(name: &str, site_addr: &str, issuer_url: &str) -> Result<Output, String> {
    let data_dir = TempDir::new(name);
    run(&mut server_command(&data_dir, site_addr, issuer_url))
}

fn run_with_site_addr(name: &str, site_addr: &str) -> Result<Output, String> {
    run_with(name, site_addr, "http://127.0.0.1:1")
}

#[test]
fn missing_config_names_the_missing_vars() {
    let output = run(Command::new(env!("CARGO_BIN_EXE_crabbage")).env_clear())
        .expect("binary runs and exits");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("OIDC_CLIENT_ID"), "{stderr}");
}

#[test]
fn invalid_site_addr_is_a_configuration_error() {
    let output =
        run_with_site_addr("invalid-addr", "not-an-address").expect("binary runs and exits");
    assert!(!output.status.success());
    assert_ne!(output.stderr.len(), 0);
}

#[test]
fn site_addr_in_use_is_a_bind_error() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("port is free");
    let address = listener.local_addr().expect("address is known");
    let output =
        run_with_site_addr("addr-in-use", &address.to_string()).expect("binary runs and exits");
    drop(listener);
    assert!(!output.status.success());
    assert_ne!(output.stderr.len(), 0);
}

#[test]
fn unreachable_issuer_serves_health_and_fails_sign_in_with_503() {
    let site_addr = free_loopback_address();
    let issuer = format!("http://{}", free_loopback_address());
    let data_dir = TempDir::new("unreachable-issuer");
    let mut command = server_command(&data_dir, &site_addr, &issuer);
    let server = Server(
        command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("binary starts"),
    );
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    while status_of(&site_addr, "/healthz") != Ok(200) {
        assert!(Instant::now() < deadline, "/healthz is not served in time");
        sleep(Duration::from_millis(50));
    }
    assert_eq!(status_of(&site_addr, "/auth/login"), Ok(503));
    drop(server);
}
