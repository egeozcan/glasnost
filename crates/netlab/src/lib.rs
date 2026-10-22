//! The network lab's servers and probe (M9 N8c), test only.
//!
//! - [`serve`]: a STUN and TURN server on webrtc-rs's `turn` crate (which
//!   answers STUN binding requests too). It listens on one or more UDP
//!   addresses, relays from a port range on one IP, takes long-term
//!   logins (`--user NAME=PASS`) and prints a counts line every few
//!   seconds: the packets it took by kind (binding, allocate and other
//!   requests, and the client's relayed data, channel data and send
//!   indications) and the allocations alive. Those counts are what lets
//!   the lab say a pair went "through TURN".
//! - [`probe`]: asks two STUN addresses for this socket's outside address
//!   from one UDP socket, and says whether the NAT in between mapped both
//!   to the same address and port (endpoint-independent mapping) or not
//!   (address- or port-dependent: a symmetric NAT).
//!
//! It is a lab tool, never part of the game: no client crate, neither the
//! lockstep crate nor the direct links' crate, and not the relay may
//! depend on it (`cargo xtask deps`). Our own harness; no spec applies.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use stun::agent::TransactionId;
use stun::message::{BINDING_REQUEST, Getter, Message};
use stun::xoraddr::XorMappedAddress;
use turn::auth::{AuthHandler, generate_auth_key};
use turn::relay::relay_range::RelayAddressGeneratorRanges;
use turn::server::Server;
use turn::server::config::{ConnConfig, ServerConfig};
use util::Conn;
use util::vnet::net::Net;

/// The default realm of the lab's logins.
// nocite: our own harness
pub const REALM: &str = "glasnost-lab";

/// The default relay port range (inclusive).
// nocite: our own harness (a small range, so that a firewall opens few ports)
pub const PORTS: (u16, u16) = (49160, 49200);

/// The default time between two counts lines, in seconds.
// nocite: our own harness
pub const STATS_S: u64 = 10;

/// The server's settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    /// The UDP addresses it listens on.
    pub listen: Vec<SocketAddr>,
    /// The IP its relays are bound to and that it gives clients as their
    /// relayed address.
    pub relay_ip: IpAddr,
    /// The address the relays bind to (default: `relay_ip`); behind a
    /// cloud NAT the bound and the given address differ.
    pub relay_bind: Option<IpAddr>,
    /// The relay port range, inclusive.
    pub ports: (u16, u16),
    /// The realm.
    pub realm: String,
    /// The logins, `NAME` and `PASSWORD`.
    pub users: Vec<(String, String)>,
    /// The seconds between two counts lines (0: none).
    pub stats_s: u64,
}

/// The usage of `glasnost-turn`.
pub const USAGE: &str = "glasnost-turn serve --listen IP:PORT [--listen IP:PORT ...] --relay-ip IP [--relay-bind IP] \
[--ports MIN-MAX] [--realm R] [--user NAME=PASS ...] [--stats SECONDS]\n\
  glasnost-turn probe --server IP:PORT --server IP:PORT [--bind IP:PORT] [--timeout MS]";

/// Reads the options after `serve`.
pub fn parse(args: &[String]) -> Result<Options, String> {
    let mut listen = Vec::new();
    let mut relay_ip = None;
    let mut relay_bind = None;
    let mut ports = PORTS;
    let mut realm = REALM.to_string();
    let mut users = Vec::new();
    let mut stats_s = STATS_S;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = |what: &str| it.next().cloned().ok_or(format!("{what} needs a value"));
        match a.as_str() {
            "--listen" => listen.push(
                val("--listen")?
                    .parse()
                    .map_err(|e| format!("--listen: {e}"))?,
            ),
            "--relay-ip" => {
                relay_ip = Some(
                    val("--relay-ip")?
                        .parse()
                        .map_err(|e| format!("--relay-ip: {e}"))?,
                )
            }
            "--relay-bind" => {
                relay_bind = Some(
                    val("--relay-bind")?
                        .parse()
                        .map_err(|e| format!("--relay-bind: {e}"))?,
                )
            }
            "--ports" => {
                let v = val("--ports")?;
                let (a, b) = v.split_once('-').ok_or("--ports needs MIN-MAX")?;
                let (a, b): (u16, u16) = (
                    a.parse().map_err(|_| "--ports needs MIN-MAX")?,
                    b.parse().map_err(|_| "--ports needs MIN-MAX")?,
                );
                if a == 0 || b < a {
                    return Err(format!("--ports {v}: an empty range"));
                }
                ports = (a, b);
            }
            "--realm" => realm = val("--realm")?,
            "--user" => {
                let v = val("--user")?;
                let (n, p) = v.split_once('=').ok_or("--user needs NAME=PASS")?;
                if n.is_empty() {
                    return Err("--user needs a name".into());
                }
                users.push((n.to_string(), p.to_string()));
            }
            "--stats" => {
                stats_s = val("--stats")?
                    .parse()
                    .map_err(|_| "--stats needs seconds")?
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    if listen.is_empty() {
        return Err("--listen is needed".into());
    }
    Ok(Options {
        listen,
        relay_ip: relay_ip.ok_or("--relay-ip is needed")?,
        relay_bind,
        ports,
        realm,
        users,
        stats_s,
    })
}

/// What the server took, by kind (atomic: the listeners count from their
/// own tasks).
#[derive(Debug, Default)]
pub struct Counts {
    /// STUN binding requests (a client asking its outside address).
    pub binding: AtomicU64,
    /// TURN allocate requests (with or without a login).
    pub allocate: AtomicU64,
    /// Other STUN or TURN requests (refresh, permissions, channel binds).
    pub other: AtomicU64,
    /// Packets of a client's data to relay (channel data and send
    /// indications), and their bytes.
    pub relayed: AtomicU64,
    /// Their bytes.
    pub relayed_bytes: AtomicU64,
    /// Packets sent back to clients, and their bytes.
    pub sent: AtomicU64,
    /// Their bytes.
    pub sent_bytes: AtomicU64,
}

impl Counts {
    /// Counts one packet a client sent the server.
    pub fn take(&self, b: &[u8]) {
        let kind = match b.first() {
            // Channel data: a channel number in 0x4000 to 0x7FFF.
            Some(&f) if (0x40..0x80).contains(&f) => Kind::Data,
            Some(_) if b.len() >= 20 => match u16::from_be_bytes([b[0], b[1]]) {
                0x0001 => Kind::Binding,
                0x0003 => Kind::Allocate,
                // A send indication.
                0x0016 => Kind::Data,
                _ => Kind::Other,
            },
            _ => Kind::Other,
        };
        match kind {
            Kind::Binding => self.binding.fetch_add(1, Ordering::Relaxed),
            Kind::Allocate => self.allocate.fetch_add(1, Ordering::Relaxed),
            Kind::Other => self.other.fetch_add(1, Ordering::Relaxed),
            Kind::Data => {
                self.relayed_bytes
                    .fetch_add(b.len() as u64, Ordering::Relaxed);
                self.relayed.fetch_add(1, Ordering::Relaxed)
            }
        };
    }

    /// The counts line, with the allocations alive.
    pub fn line(&self, allocations: usize) -> String {
        let g = |a: &AtomicU64| a.load(Ordering::Relaxed);
        format!(
            "stats allocations={allocations} binding={} allocate={} other={} relayed={}/{}B sent={}/{}B",
            g(&self.binding),
            g(&self.allocate),
            g(&self.other),
            g(&self.relayed),
            g(&self.relayed_bytes),
            g(&self.sent),
            g(&self.sent_bytes)
        )
    }
}

enum Kind {
    Binding,
    Allocate,
    Other,
    Data,
}

/// A listening socket that counts what passes.
struct Counting {
    sock: tokio::net::UdpSocket,
    counts: Arc<Counts>,
}

#[async_trait]
impl Conn for Counting {
    async fn connect(&self, addr: SocketAddr) -> util::Result<()> {
        Ok(self.sock.connect(addr).await?)
    }
    async fn recv(&self, buf: &mut [u8]) -> util::Result<usize> {
        let n = self.sock.recv(buf).await?;
        self.counts.take(&buf[..n]);
        Ok(n)
    }
    async fn recv_from(&self, buf: &mut [u8]) -> util::Result<(usize, SocketAddr)> {
        let (n, from) = self.sock.recv_from(buf).await?;
        self.counts.take(&buf[..n]);
        Ok((n, from))
    }
    async fn send(&self, buf: &[u8]) -> util::Result<usize> {
        let n = self.sock.send(buf).await?;
        self.counts.sent.fetch_add(1, Ordering::Relaxed);
        self.counts
            .sent_bytes
            .fetch_add(n as u64, Ordering::Relaxed);
        Ok(n)
    }
    async fn send_to(&self, buf: &[u8], target: SocketAddr) -> util::Result<usize> {
        let n = self.sock.send_to(buf, target).await?;
        self.counts.sent.fetch_add(1, Ordering::Relaxed);
        self.counts
            .sent_bytes
            .fetch_add(n as u64, Ordering::Relaxed);
        Ok(n)
    }
    fn local_addr(&self) -> util::Result<SocketAddr> {
        Ok(self.sock.local_addr()?)
    }
    fn remote_addr(&self) -> Option<SocketAddr> {
        None
    }
    async fn close(&self) -> util::Result<()> {
        Ok(())
    }
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }
}

/// The logins.
struct Logins(HashMap<String, Vec<u8>>);

impl AuthHandler for Logins {
    fn auth_handle(
        &self,
        username: &str,
        _realm: &str,
        _src: SocketAddr,
    ) -> Result<Vec<u8>, turn::Error> {
        self.0
            .get(username)
            .cloned()
            .ok_or(turn::Error::ErrNoSuchUser)
    }
}

/// A running server.
pub struct Running {
    server: Server,
    /// Its counts.
    pub counts: Arc<Counts>,
    /// The addresses it listens on (the real ports, for `:0`).
    pub addrs: Vec<SocketAddr>,
}

impl Running {
    /// The allocations alive.
    pub async fn allocations(&self) -> usize {
        self.server
            .get_allocations_info(None)
            .await
            .map_or(0, |m| m.len())
    }

    /// Stops the server.
    pub async fn close(self) {
        let _ = self.server.close().await;
    }
}

/// Starts the server (in the caller's tokio runtime).
pub async fn start(o: &Options) -> Result<Running, String> {
    let counts = Arc::new(Counts::default());
    let mut conn_configs = Vec::new();
    let mut addrs = Vec::new();
    for a in &o.listen {
        let sock = tokio::net::UdpSocket::bind(a)
            .await
            .map_err(|e| format!("cannot listen on udp {a}: {e}"))?;
        addrs.push(sock.local_addr().map_err(|e| e.to_string())?);
        conn_configs.push(ConnConfig {
            conn: Arc::new(Counting {
                sock,
                counts: counts.clone(),
            }),
            relay_addr_generator: Box::new(RelayAddressGeneratorRanges {
                relay_address: o.relay_ip,
                min_port: o.ports.0,
                max_port: o.ports.1,
                max_retries: 0,
                address: o.relay_bind.unwrap_or(o.relay_ip).to_string(),
                net: Arc::new(Net::new(None)),
            }),
        });
    }
    let logins = o
        .users
        .iter()
        .map(|(n, p)| (n.clone(), generate_auth_key(n, &o.realm, p)))
        .collect();
    let server = Server::new(ServerConfig {
        conn_configs,
        realm: o.realm.clone(),
        auth_handler: Arc::new(Logins(logins)),
        channel_bind_timeout: Duration::from_secs(0),
        alloc_close_notify: None,
    })
    .await
    .map_err(|e| format!("the server does not start: {e}"))?;
    Ok(Running {
        server,
        counts,
        addrs,
    })
}

/// Runs `glasnost-turn serve` until Ctrl-C or SIGTERM, printing the counts
/// line every [`Options::stats_s`] seconds and once at the end.
pub fn serve(o: &Options) -> Result<(), String> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    rt.block_on(async {
        let r = start(o).await?;
        let addrs: Vec<String> = r.addrs.iter().map(SocketAddr::to_string).collect();
        println!(
            "glasnost-turn listening on udp {}; relays on {} ports {}-{}; {} login(s)",
            addrs.join(","),
            o.relay_ip,
            o.ports.0,
            o.ports.1,
            o.users.len()
        );
        let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .map_err(|e| e.to_string())?;
        let every = Duration::from_secs(o.stats_s.max(1));
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => break,
                _ = term.recv() => break,
                _ = tokio::time::sleep(every) => {
                    if o.stats_s > 0 {
                        println!("{}", r.counts.line(r.allocations().await));
                    }
                }
            }
        }
        println!("{}", r.counts.line(r.allocations().await));
        r.close().await;
        Ok(())
    })
}

/// What a probe found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probe {
    /// The local address of the probe's socket.
    pub local: SocketAddr,
    /// The outside address each server saw, in the order asked.
    pub mapped: Vec<SocketAddr>,
}

impl Probe {
    /// Whether every server saw the same outside address and port.
    pub fn independent(&self) -> bool {
        self.mapped.windows(2).all(|w| w[0] == w[1])
    }

    /// The probe's line.
    pub fn line(&self) -> String {
        let m: Vec<String> = self.mapped.iter().map(SocketAddr::to_string).collect();
        format!(
            "probe local {} mapped {} mapping {}",
            self.local,
            m.join(" "),
            if self.independent() {
                "endpoint-independent"
            } else {
                "dependent"
            }
        )
    }
}

/// Asks each server in `servers`, from one socket bound to `bind`, for
/// this socket's outside address (a STUN binding request, resent up to
/// three times).
pub fn probe(bind: SocketAddr, servers: &[SocketAddr], timeout: Duration) -> Result<Probe, String> {
    let sock = UdpSocket::bind(bind).map_err(|e| format!("cannot bind {bind}: {e}"))?;
    sock.set_read_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    let local = sock.local_addr().map_err(|e| e.to_string())?;
    let mut mapped = Vec::new();
    for s in servers {
        let mut req = Message::new();
        req.build(&[Box::new(TransactionId::new()), Box::new(BINDING_REQUEST)])
            .map_err(|e| e.to_string())?;
        let mut got = None;
        // nocite: our own harness (a lost datagram is resent)
        for _ in 0..3 {
            sock.send_to(&req.raw, s)
                .map_err(|e| format!("cannot send to {s}: {e}"))?;
            let mut buf = [0u8; 1500];
            let Ok((n, from)) = sock.recv_from(&mut buf) else {
                continue;
            };
            if from != *s {
                continue;
            }
            let mut m = Message::new();
            m.raw = buf[..n].to_vec();
            if m.decode().is_err() || m.transaction_id != req.transaction_id {
                continue;
            }
            let mut x = XorMappedAddress::default();
            if x.get_from(&m).is_ok() {
                got = Some(SocketAddr::new(x.ip, x.port));
                break;
            }
        }
        mapped.push(got.ok_or_else(|| format!("no answer from {s}"))?);
    }
    Ok(Probe { local, mapped })
}

/// Reads the options after `probe`.
pub fn parse_probe(args: &[String]) -> Result<(SocketAddr, Vec<SocketAddr>, Duration), String> {
    let mut bind: SocketAddr = "0.0.0.0:0".parse().expect("an address");
    let mut servers = Vec::new();
    let mut timeout = Duration::from_millis(1_000); // nocite: our own harness
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let v = it.next().ok_or(format!("{a} needs a value"))?;
        match a.as_str() {
            "--bind" => bind = v.parse().map_err(|e| format!("--bind: {e}"))?,
            "--server" => servers.push(v.parse().map_err(|e| format!("--server: {e}"))?),
            "--timeout" => {
                timeout = Duration::from_millis(v.parse().map_err(|_| "--timeout needs ms")?)
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    if servers.is_empty() {
        return Err("--server is needed".into());
    }
    Ok((bind, servers, timeout))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(str::to_string).collect()
    }

    #[test]
    fn options_parse() {
        let o = parse(&args(
            "--listen 10.0.0.1:3478 --listen 10.0.0.1:3479 --relay-ip 10.0.0.1 --ports 50000-50010 --user a=b=c --stats 0",
        ))
        .expect("parses");
        assert_eq!(o.listen.len(), 2);
        assert_eq!(o.ports, (50000, 50010));
        assert_eq!(o.users, vec![("a".to_string(), "b=c".to_string())]);
        assert_eq!(o.realm, REALM);
        for bad in [
            "--relay-ip 10.0.0.1",
            "--listen 10.0.0.1:3478",
            "--listen x --relay-ip 10.0.0.1",
            "--listen 10.0.0.1:1 --relay-ip 10.0.0.1 --ports 9-3",
            "--listen 10.0.0.1:1 --relay-ip 10.0.0.1 --user nopass",
            "--listen 10.0.0.1:1 --relay-ip 10.0.0.1 --what",
        ] {
            assert!(parse(&args(bad)).is_err(), "{bad}");
        }
    }

    #[test]
    fn packets_are_counted_by_kind() {
        let c = Counts::default();
        let mut stun = vec![0u8; 20];
        stun[1] = 1;
        c.take(&stun);
        stun[1] = 3;
        c.take(&stun);
        stun[1] = 0x16;
        c.take(&stun);
        c.take(&[0x40, 0, 0, 4, 1, 2, 3, 4]);
        c.take(&[1]);
        assert_eq!(
            c.line(2),
            "stats allocations=2 binding=1 allocate=1 other=1 relayed=2/28B sent=0/0B"
        );
    }
}
