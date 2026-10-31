//! The `--healthcheck` command: asks the running server for `/healthz`, for the image's health check.

use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(3);
const MAX_RESPONSE: u64 = 4096;

/// Checks the local server's health endpoint using `LEPTOS_SITE_ADDR` or port 3000.
#[must_use]
pub fn healthcheck() -> bool {
    let address = match std::env::var("LEPTOS_SITE_ADDR") {
        Ok(value) => match value.parse::<SocketAddr>() {
            Ok(address) => address,
            Err(_) => return false,
        },
        Err(std::env::VarError::NotPresent) => SocketAddr::from((Ipv4Addr::LOCALHOST, 3000)),
        Err(std::env::VarError::NotUnicode(_)) => return false,
    };
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), address.port());
    check(address, TIMEOUT).unwrap_or(false)
}

fn check(address: SocketAddr, timeout: Duration) -> std::io::Result<bool> {
    let mut stream = TcpStream::connect_timeout(&address, timeout)?;
    stream.set_write_timeout(Some(timeout))?;
    stream.set_read_timeout(Some(timeout))?;
    stream.write_all(b"GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;
    let mut response = Vec::new();
    match stream.take(MAX_RESPONSE).read_to_end(&mut response) {
        Ok(_) => {}
        Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
            return Ok(false);
        }
        Err(error) => return Err(error),
    }
    let response = String::from_utf8_lossy(&response);
    Ok(response
        .split_once("\r\n\r\n")
        .is_some_and(|(head, body)| head.starts_with("HTTP/1.1 200 ") && body == "ok"))
}

#[cfg(test)]
mod tests {
    use super::check;
    use std::io::{Read, Write};
    use std::net::{Ipv4Addr, SocketAddr, TcpListener};
    use std::thread;
    use std::time::Duration;

    fn listener() -> (TcpListener, SocketAddr) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        (listener, address)
    }

    fn serve(response: &'static [u8]) -> SocketAddr {
        let (listener, address) = listener();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 256];
            let _read_result = stream.read(&mut request);
            stream.write_all(response).unwrap();
        });
        address
    }

    #[test]
    fn accepts_ok_body() {
        let address = serve(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok");
        assert!(check(address, Duration::from_secs(1)).unwrap());
    }

    #[test]
    fn rejects_wrong_body() {
        let address = serve(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nnope");
        assert!(!check(address, Duration::from_secs(1)).unwrap());
    }

    #[test]
    fn rejects_error_status() {
        let address = serve(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 2\r\n\r\nok");
        assert!(!check(address, Duration::from_secs(1)).unwrap());
    }

    #[test]
    fn rejects_connection_refused() {
        let (listener, address) = listener();
        drop(listener);
        assert!(check(address, Duration::from_millis(100)).is_err());
    }

    #[test]
    fn rejects_timeout() {
        let (listener, address) = listener();
        thread::spawn(move || {
            let _accepted = listener.accept().unwrap();
            thread::sleep(Duration::from_millis(200));
        });
        assert!(!check(address, Duration::from_millis(50)).unwrap());
    }
}
