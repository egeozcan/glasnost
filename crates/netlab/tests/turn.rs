//! The lab's STUN and TURN server on loopback (M9 N8c): a probe of two of
//! its ports sees one outside address (no NAT in between), and a TURN
//! client allocates a relay, sends a datagram through it to a peer and
//! gets the peer's answer, all of it counted. Our own harness.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use glasnost_netlab::{Options, parse, probe, start};
use turn::client::{Client, ClientConfig};
use util::Conn;

fn options() -> Options {
    let args: Vec<String> = "--listen 127.0.0.1:0 --listen 127.0.0.1:0 --relay-ip 127.0.0.1 --ports 51300-51340 --user lab=word --stats 0"
        .split_whitespace()
        .map(str::to_string)
        .collect();
    parse(&args).expect("the options")
}

#[test]
fn a_probe_without_a_nat_sees_one_address_and_a_turn_relay_carries_both_ways() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime");
    let server = rt.block_on(start(&options())).expect("the server starts");
    let addrs = server.addrs.clone();
    // The probe, from a blocking socket on another thread's time.
    let p = probe(
        "127.0.0.1:0".parse().unwrap(),
        &addrs,
        Duration::from_secs(2),
    )
    .expect("both ports answer");
    assert!(p.independent(), "{}", p.line());
    assert_eq!(
        p.mapped[0], p.local,
        "no NAT: the outside address is the socket's"
    );
    assert_eq!(server.counts.binding.load(Ordering::Relaxed), 2);

    rt.block_on(async {
        let conn = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let client = Client::new(ClientConfig {
            stun_serv_addr: addrs[0].to_string(),
            turn_serv_addr: addrs[0].to_string(),
            username: "lab".into(),
            password: "word".into(),
            realm: glasnost_netlab::REALM.into(),
            software: String::new(),
            rto_in_ms: 0,
            conn: Arc::new(conn),
            vnet: None,
        })
        .await
        .expect("a client");
        client.listen().await.expect("listens");
        let relay = client.allocate().await.expect("an allocation");
        let relayed = relay.local_addr().expect("the relayed address");
        assert!((51300..=51340).contains(&relayed.port()), "{relayed}");
        assert_eq!(server.allocations().await, 1);

        let peer = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let peer_addr = peer.local_addr().unwrap();
        let mut buf = [0u8; 64];
        // The first send makes the permission; resend until it passes.
        let mut from = None;
        for _ in 0..20 {
            relay.send_to(b"through", peer_addr).await.expect("sends");
            if let Ok(Ok((n, f))) =
                tokio::time::timeout(Duration::from_millis(200), peer.recv_from(&mut buf)).await
            {
                assert_eq!(&buf[..n], b"through");
                from = Some(f);
                break;
            }
        }
        let from = from.expect("the peer got the datagram");
        assert_eq!(from, relayed, "it came from the relayed address");
        peer.send_to(b"back", from).await.unwrap();
        let (n, f) = tokio::time::timeout(Duration::from_secs(2), relay.recv_from(&mut buf))
            .await
            .expect("an answer in time")
            .expect("an answer");
        assert_eq!((&buf[..n], f), (&b"back"[..], peer_addr));
        let c = &server.counts;
        assert!(c.allocate.load(Ordering::Relaxed) >= 1);
        assert!(c.relayed.load(Ordering::Relaxed) >= 1, "{}", c.line(1));
        relay.close().await.unwrap();
        client.close().await.unwrap();
    });
    rt.block_on(server.close());
}
