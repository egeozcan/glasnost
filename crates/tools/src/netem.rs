//! A bad network between a client and the relay (M9 N6, the soak): a TCP
//! proxy on loopback that delays what passes through it, seeded.
//!
//! A WebSocket runs over TCP, which never loses or reorders data: a lost
//! packet is sent again, and everything behind it waits. So the proxy
//! models loss as late delivery, never as missing bytes:
//!
//! - **Latency with jitter**: every chunk read is delivered
//!   `latency + jitter` later, the jitter drawn in `0..=jitter_ms`; never
//!   before the chunk ahead of it (the order is kept: head-of-line
//!   blocking, as on TCP).
//! - **Loss**: a chunk is "lost" with a chance of `loss_permille` in a
//!   thousand and delivered `retransmit_ms` later still (TCP's resend), and
//!   everything behind it waits.
//! - **Stalls**: now and then, seeded, a direction delivers nothing for a
//!   while (a route that hangs), then all that waited at once.
//! - **Cuts**: [`Proxy::cut`] closes every connection through the proxy,
//!   as a dropped link does; the client then reconnects through it.
//!
//! The proxy keeps reading while it holds data back, so a stall never
//! pushes back on the sender (the relay's queue limits see the delay only
//! as late answers). It counts the bytes each way. Our own harness; no
//! spec applies.

use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// The one-way latency of the default bad network, in milliseconds.
// nocite: our own harness (a long continental route)
pub const LATENCY_MS: u64 = 40;
/// The jitter of the default bad network, in milliseconds.
// nocite: our own harness
pub const JITTER_MS: u64 = 30;
/// The chance in a thousand that a chunk is lost and sent again.
// nocite: our own harness (2 %, a poor wireless link)
pub const LOSS_PERMILLE: u32 = 20;
/// How much later a lost chunk arrives, in milliseconds (a TCP resend).
// nocite: our own harness (TCP's minimum retransmission timeout, RFC 6298 2.4, is 1 s; Linux uses 200 ms)
pub const RETRANSMIT_MS: u64 = 200;
/// The shortest and longest wait between two stalls of one direction, in
/// milliseconds.
// nocite: our own harness
pub const STALL_EVERY_MS: (u64, u64) = (45_000, 150_000);
/// The shortest and longest stall, in milliseconds: well under the
/// session's 5 s silence notice and the relay's 15 s silence limit.
// nocite: our own harness
pub const STALL_MS: (u64, u64) = (500, 2_500);

/// The size of one read.
// nocite: our own harness
const CHUNK: usize = 16 * 1024;

/// What the proxy does to the traffic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Netem {
    /// One-way latency, in milliseconds.
    pub latency_ms: u64,
    /// Jitter, in milliseconds (drawn in `0..=jitter_ms`).
    pub jitter_ms: u64,
    /// The chance in a thousand that a chunk is lost and sent again.
    pub loss_permille: u32,
    /// How much later a lost chunk arrives, in milliseconds.
    pub retransmit_ms: u64,
    /// The shortest and longest wait between two stalls; `(0, 0)`: none.
    pub stall_every_ms: (u64, u64),
    /// The shortest and longest stall, in milliseconds.
    pub stall_ms: (u64, u64),
    /// The seed of the proxy's random draws.
    pub seed: u64,
}

impl Netem {
    /// The default bad network ([`LATENCY_MS`] and the constants after it).
    pub fn bad(seed: u64) -> Self {
        Self {
            latency_ms: LATENCY_MS,
            jitter_ms: JITTER_MS,
            loss_permille: LOSS_PERMILLE,
            retransmit_ms: RETRANSMIT_MS,
            stall_every_ms: STALL_EVERY_MS,
            stall_ms: STALL_MS,
            seed,
        }
    }

    /// A perfect network (the proxy only passes the bytes on).
    pub fn clean(seed: u64) -> Self {
        Self {
            latency_ms: 0,
            jitter_ms: 0,
            loss_permille: 0,
            retransmit_ms: 0,
            stall_every_ms: (0, 0),
            stall_ms: (0, 0),
            seed,
        }
    }

    /// The settings as one line of text.
    pub fn describe(&self) -> String {
        format!(
            "latency {} ms, jitter 0..={} ms, loss {}/1000 (+{} ms), stalls of {}..{} ms every {}..{} ms",
            self.latency_ms,
            self.jitter_ms,
            self.loss_permille,
            self.retransmit_ms,
            self.stall_ms.0,
            self.stall_ms.1,
            self.stall_every_ms.0,
            self.stall_every_ms.1
        )
    }
}

/// SplitMix64: the proxy's seeded draws.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // nocite: SplitMix64's published constants (Steele, Lea, Flood 2014)
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A draw in `lo..=hi`.
    fn range(&mut self, (lo, hi): (u64, u64)) -> u64 {
        if hi <= lo {
            return lo;
        }
        lo + self.next() % (hi - lo + 1)
    }
}

/// The proxy's counts.
#[derive(Default, Debug)]
pub struct Counts {
    /// Bytes from the client to the relay.
    pub up: AtomicU64,
    /// Bytes from the relay to the client.
    pub down: AtomicU64,
    /// Connections accepted.
    pub connections: AtomicU64,
    /// Chunks "lost" (delivered a resend later).
    pub lost: AtomicU64,
    /// Stalls, both ways.
    pub stalls: AtomicU64,
    /// The longest stall, in milliseconds.
    pub longest_stall_ms: AtomicU64,
    /// Cuts.
    pub cuts: AtomicU64,
}

/// A running proxy: it serves until dropped.
pub struct Proxy {
    /// The address clients connect to.
    pub addr: SocketAddr,
    /// Its counts.
    pub counts: Arc<Counts>,
    live: Arc<Mutex<Vec<TcpStream>>>,
    stop: Arc<AtomicBool>,
}

impl Proxy {
    /// A proxy on a free loopback port in front of `upstream`.
    pub fn start(upstream: SocketAddr, netem: Netem) -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        listener.set_nonblocking(true)?;
        let counts = Arc::new(Counts::default());
        let live = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (c, l, s) = (counts.clone(), live.clone(), stop.clone());
        thread::Builder::new()
            .name(format!("netem {addr}"))
            .spawn(move || accept(listener, upstream, netem, c, l, s))?;
        Ok(Self {
            addr,
            counts,
            live,
            stop,
        })
    }

    /// The WebSocket address of the relay through the proxy.
    pub fn url(&self) -> String {
        format!("ws://{}/", self.addr)
    }

    /// Closes every connection through the proxy, as a dropped link does.
    pub fn cut(&self) {
        let mut live = self.live.lock().expect("the proxy's list");
        for s in live.drain(..) {
            let _ = s.shutdown(Shutdown::Both);
        }
        self.counts.cuts.fetch_add(1, Ordering::Relaxed);
    }

    /// Bytes up and down.
    pub fn bytes(&self) -> (u64, u64) {
        (
            self.counts.up.load(Ordering::Relaxed),
            self.counts.down.load(Ordering::Relaxed),
        )
    }
}

impl Drop for Proxy {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.cut();
    }
}

fn accept(
    listener: TcpListener,
    upstream: SocketAddr,
    netem: Netem,
    counts: Arc<Counts>,
    live: Arc<Mutex<Vec<TcpStream>>>,
    stop: Arc<AtomicBool>,
) {
    let mut n = 0u64;
    while !stop.load(Ordering::Relaxed) {
        let client = match listener.accept() {
            Ok((s, _)) => s,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
                continue;
            }
            Err(_) => continue,
        };
        let _ = client.set_nonblocking(false);
        let Ok(server) = TcpStream::connect(upstream) else {
            let _ = client.shutdown(Shutdown::Both);
            continue;
        };
        let _ = client.set_nodelay(true);
        let _ = server.set_nodelay(true);
        n += 1;
        counts.connections.fetch_add(1, Ordering::Relaxed);
        {
            let mut l = live.lock().expect("the proxy's list");
            for s in [&client, &server] {
                if let Ok(s) = s.try_clone() {
                    l.push(s);
                }
            }
        }
        let seed = netem.seed ^ n.wrapping_mul(0x2545_f491_4f6c_dd1d);
        for (from, to, up) in [
            (client.try_clone(), server.try_clone(), true),
            (server.try_clone(), client.try_clone(), false),
        ] {
            let (Ok(from), Ok(to)) = (from, to) else {
                continue;
            };
            let dir_seed = if up { seed } else { !seed };
            pipe(from, to, netem, dir_seed, up, counts.clone());
        }
    }
}

/// One direction: a reader that stamps every chunk with its delivery time,
/// and a writer that delivers it then.
fn pipe(mut from: TcpStream, mut to: TcpStream, netem: Netem, seed: u64, up: bool, c: Arc<Counts>) {
    let (tx, rx) = mpsc::channel::<(Instant, Vec<u8>)>();
    let counts = c.clone();
    let _ = thread::Builder::new()
        .name("netem read".into())
        .spawn(move || {
            let mut rng = Rng(seed);
            let mut last = Instant::now();
            let mut buf = vec![0u8; CHUNK];
            loop {
                let n = match from.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => n,
                };
                let mut wait = netem.latency_ms + rng.range((0, netem.jitter_ms));
                if netem.loss_permille > 0 && rng.next() % 1000 < u64::from(netem.loss_permille) {
                    wait += netem.retransmit_ms;
                    counts.lost.fetch_add(1, Ordering::Relaxed);
                }
                let at = (Instant::now() + Duration::from_millis(wait)).max(last);
                last = at;
                if tx.send((at, buf[..n].to_vec())).is_err() {
                    break;
                }
            }
            let _ = from.shutdown(Shutdown::Read);
        });
    let _ = thread::Builder::new()
        .name("netem write".into())
        .spawn(move || {
            let mut rng = Rng(seed.rotate_left(17));
            let stalls = netem.stall_every_ms.1 > 0;
            let mut next_stall =
                Instant::now() + Duration::from_millis(rng.range(netem.stall_every_ms));
            let mut held: VecDeque<(Instant, Vec<u8>)> = VecDeque::new();
            let mut open = true;
            while open || !held.is_empty() {
                // Take what was read (without waiting when something is due).
                let timeout = held
                    .front()
                    .map(|(at, _)| at.saturating_duration_since(Instant::now()))
                    .unwrap_or(Duration::from_millis(50));
                match rx.recv_timeout(timeout) {
                    Ok(m) => held.push_back(m),
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => open = false,
                }
                while let Ok(m) = rx.try_recv() {
                    held.push_back(m);
                }
                let now = Instant::now();
                if stalls && now >= next_stall {
                    // The route hangs: nothing is delivered for a while, but
                    // reading goes on.
                    let len = rng.range(netem.stall_ms);
                    let until = now + Duration::from_millis(len);
                    while Instant::now() < until {
                        match rx.recv_timeout(until.saturating_duration_since(Instant::now())) {
                            Ok(m) => held.push_back(m),
                            Err(mpsc::RecvTimeoutError::Timeout) => {}
                            Err(mpsc::RecvTimeoutError::Disconnected) => {
                                open = false;
                                break;
                            }
                        }
                    }
                    c.stalls.fetch_add(1, Ordering::Relaxed);
                    c.longest_stall_ms.fetch_max(len, Ordering::Relaxed);
                    next_stall =
                        Instant::now() + Duration::from_millis(rng.range(netem.stall_every_ms));
                }
                let now = Instant::now();
                while held.front().is_some_and(|(at, _)| *at <= now) {
                    let (_, bytes) = held.pop_front().expect("a chunk");
                    // Counted before the write, so that whoever has read
                    // the bytes sees them counted.
                    let n = bytes.len() as u64;
                    if up {
                        c.up.fetch_add(n, Ordering::Relaxed);
                    } else {
                        c.down.fetch_add(n, Ordering::Relaxed);
                    }
                    if to.write_all(&bytes).is_err() {
                        let _ = to.shutdown(Shutdown::Both);
                        return;
                    }
                }
            }
            let _ = to.shutdown(Shutdown::Write);
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bytes pass through in order, late by at least the latency.
    #[test]
    fn bytes_pass_in_order_and_late() {
        let echo = TcpListener::bind("127.0.0.1:0").unwrap();
        let up = echo.local_addr().unwrap();
        thread::spawn(move || {
            let (mut s, _) = echo.accept().unwrap();
            let mut buf = [0u8; 1024];
            loop {
                match s.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => s.write_all(&buf[..n]).unwrap(),
                }
            }
        });
        let mut n = Netem::clean(7);
        n.latency_ms = 30;
        n.jitter_ms = 20;
        n.loss_permille = 500;
        n.retransmit_ms = 10;
        let p = Proxy::start(up, n).unwrap();
        let mut s = TcpStream::connect(p.addr).unwrap();
        let t = Instant::now();
        let sent: Vec<u8> = (0..200u8).collect();
        for b in sent.chunks(10) {
            s.write_all(b).unwrap();
            thread::sleep(Duration::from_millis(1));
        }
        let mut got = vec![0u8; sent.len()];
        s.read_exact(&mut got).unwrap();
        assert_eq!(got, sent);
        // There and back: twice the latency at least.
        assert!(t.elapsed() >= Duration::from_millis(60));
        assert_eq!(p.bytes(), (200, 200));
        assert!(p.counts.lost.load(Ordering::Relaxed) > 0);
    }

    /// A cut closes the connection; a new one passes again.
    #[test]
    fn a_cut_closes_and_a_new_connection_passes() {
        let echo = TcpListener::bind("127.0.0.1:0").unwrap();
        let up = echo.local_addr().unwrap();
        thread::spawn(move || {
            for s in echo.incoming() {
                let mut s = s.unwrap();
                thread::spawn(move || {
                    let mut buf = [0u8; 64];
                    while let Ok(n) = s.read(&mut buf) {
                        if n == 0 || s.write_all(&buf[..n]).is_err() {
                            break;
                        }
                    }
                });
            }
        });
        let p = Proxy::start(up, Netem::clean(1)).unwrap();
        let mut a = TcpStream::connect(p.addr).unwrap();
        a.write_all(b"hi").unwrap();
        let mut b2 = [0u8; 2];
        a.read_exact(&mut b2).unwrap();
        thread::sleep(Duration::from_millis(20));
        p.cut();
        let mut rest = Vec::new();
        assert_eq!(a.read_to_end(&mut rest).map(|_| rest.len()).unwrap_or(0), 0);
        let mut b = TcpStream::connect(p.addr).unwrap();
        b.write_all(b"yo").unwrap();
        b.read_exact(&mut b2).unwrap();
        assert_eq!(&b2, b"yo");
    }
}
