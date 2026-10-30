//! Runs the server binary to check its startup errors: a bad configuration and an address in use.

use std::net::TcpListener;
use std::process::{Command, Output, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);

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

fn run_with_site_addr(site_addr: &str) -> Result<Output, String> {
    run(Command::new(env!("CARGO_BIN_EXE_crabbage"))
        .envs(VALID_CONFIG)
        .env("LEPTOS_OUTPUT_NAME", "crabbage")
        .env("LEPTOS_SITE_ADDR", site_addr))
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
    let output = run_with_site_addr("not-an-address").expect("binary runs and exits");
    assert!(!output.status.success());
    assert_ne!(output.stderr.len(), 0);
}

#[test]
fn site_addr_in_use_is_a_bind_error() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("port is free");
    let address = listener.local_addr().expect("address is known");
    let output = run_with_site_addr(&address.to_string()).expect("binary runs and exits");
    drop(listener);
    assert!(!output.status.success());
    assert_ne!(output.stderr.len(), 0);
}
