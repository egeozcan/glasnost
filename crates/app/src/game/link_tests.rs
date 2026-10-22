//! The native link over TLS (M9 N8b): `wss://` to a local TLS WebSocket
//! echo that uses a self-signed certificate made for these tests.

use std::net::TcpListener;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

use super::{Link, LinkState, native};

// A synthetic certificate for 127.0.0.1 and localhost, valid for a hundred
// years, with its key. It signs nothing real and is trusted only by these
// tests (made with `openssl req -x509`, EC P-256).
const CERT: &str = "-----BEGIN CERTIFICATE-----
MIIBmDCCAT6gAwIBAgIUJ6B46nWoGHrm1oTkxrsl9FWlDKEwCgYIKoZIzj0EAwIw
FDESMBAGA1UEAwwJbG9jYWxob3N0MCAXDTI2MTAyMjEyMzkxNloYDzIxMjYwOTI4
MTIzOTE2WjAUMRIwEAYDVQQDDAlsb2NhbGhvc3QwWTATBgcqhkjOPQIBBggqhkjO
PQMBBwNCAARJTDXB7n3rtu3kojNUlIlsguA+iJTCiqVnFwm4FwElVVpI79Gza9Gd
rruueU8sjdvaXIOcQ+gFhawzSHSvBIAko2wwajAdBgNVHQ4EFgQU+649eOdgX6wM
3yu0fHQtJWAwFW4wHwYDVR0jBBgwFoAU+649eOdgX6wM3yu0fHQtJWAwFW4wGgYD
VR0RBBMwEYIJbG9jYWxob3N0hwR/AAABMAwGA1UdEwEB/wQCMAAwCgYIKoZIzj0E
AwIDSAAwRQIhAP0KxnyepIHUVqB5yAEt0G5oMACVsaRuhZrHsJcmU/l1AiBHUii4
MwJQP8+bWUcEuWjCTfZwA24AwNeEntqLH8Wi/w==
-----END CERTIFICATE-----
";
const KEY: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgmz1Dt2gZziL8Y/Lr
7fAKwan6ffR3uCKlYgo9YOOvqhehRANCAARJTDXB7n3rtu3kojNUlIlsguA+iJTC
iqVnFwm4FwElVVpI79Gza9GdrruueU8sjdvaXIOcQ+gFhawzSHSvBIAk
-----END PRIVATE KEY-----
";

/// A TLS WebSocket server on a free port that echoes one binary message,
/// then closes. Returns its port and thread.
fn echo_server() -> (u16, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let port = listener.local_addr().expect("an address").port();
    let cert = CertificateDer::from_pem_slice(CERT.as_bytes()).expect("a certificate");
    let key = PrivateKeyDer::from_pem_slice(KEY.as_bytes()).expect("a key");
    let config = Arc::new(
        rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .expect("a server configuration"),
    );
    let thread = std::thread::spawn(move || {
        let (tcp, _) = listener.accept().expect("a client");
        let tls = rustls::StreamOwned::new(
            rustls::ServerConnection::new(config).expect("a connection"),
            tcp,
        );
        // A client that does not trust the certificate fails the handshake.
        let Ok(mut ws) = tungstenite::accept(tls) else {
            return;
        };
        if let Ok(msg) = ws.read() {
            let _ = ws.send(msg);
        }
        let _ = ws.close(None);
        let _ = ws.flush();
    });
    (port, thread)
}

fn wait_for(link: &mut Box<dyn Link>) -> Result<Vec<u8>, LinkState> {
    let end = Instant::now() + Duration::from_secs(10);
    while Instant::now() < end {
        if let Some(m) = link.recv() {
            return Ok(m);
        }
        if let LinkState::Closed(why) = link.state() {
            return Err(LinkState::Closed(why));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Err(link.state())
}

#[test]
fn a_wss_link_to_a_server_it_trusts_carries_messages_both_ways() {
    let (port, server) = echo_server();
    let mut roots = rustls::RootCertStore::empty();
    roots
        .add(CertificateDer::from_pem_slice(CERT.as_bytes()).expect("a certificate"))
        .expect("a trusted certificate");
    let config = Arc::new(
        rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    );
    let mut link = native::connect_with(&format!("wss://localhost:{port}/"), Some(config));
    link.send(b"GL over tls".to_vec());
    assert_eq!(wait_for(&mut link), Ok(b"GL over tls".to_vec()));
    link.close();
    server.join().expect("the server ends");
}

#[test]
fn a_wss_link_refuses_a_certificate_the_system_does_not_trust() {
    let (port, server) = echo_server();
    // The ordinary link checks against the system's roots, which do not
    // hold the test certificate.
    let mut link = native::connect(&format!("wss://localhost:{port}/"));
    link.send(b"never arrives".to_vec());
    match wait_for(&mut link) {
        Err(LinkState::Closed(why)) => {
            assert!(why.contains("cannot reach the relay"), "{why}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    server.join().expect("the server ends");
}
