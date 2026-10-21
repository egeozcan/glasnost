//! A WebSocket client of the relay for the loopback tests, and the room
//! messages they share.

#![allow(dead_code)]

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use glasnost_net::lobby::{self, FromRelay, MapChoice, Offered, ToRelay};
use glasnost_relay::Entropy;
use glasnost_ui::setup::{Setup, Slot, SlotKind};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::{Bytes, Message};

/// How long a test waits for a message before it fails.
pub const WAIT: Duration = Duration::from_secs(10);

/// A seeded stand-in for the operating system's randomness.
pub struct Fake(pub u64);

impl Entropy for Fake {
    fn fill(&mut self, buf: &mut [u8]) {
        for b in buf {
            // xorshift64
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            *b = (self.0 >> 24) as u8;
        }
    }
}

/// What reached a client.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum Incoming {
    /// A lobby message.
    Lobby(FromRelay),
    /// A game message's bytes.
    Game(Vec<u8>),
    /// The connection closed, with the close frame's reason if there was
    /// one.
    Closed(Option<String>),
}

/// A connected client: what it sends goes out in order, what arrives
/// waits in a queue.
pub struct Client {
    tx: mpsc::UnboundedSender<Message>,
    rx: mpsc::UnboundedReceiver<Incoming>,
    tasks: [JoinHandle<()>; 2],
    paused: Arc<AtomicBool>,
}

impl Client {
    /// Connects to the relay at `addr`.
    pub async fn connect(addr: SocketAddr) -> Client {
        let (ws, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/"))
            .await
            .expect("the relay accepts");
        let (mut sink, mut source) = ws.split();
        let (tx, mut out) = mpsc::unbounded_channel::<Message>();
        let (inc, rx) = mpsc::unbounded_channel();
        let paused = Arc::new(AtomicBool::new(false));
        let stop = paused.clone();
        let writer = tokio::spawn(async move {
            while let Some(m) = out.recv().await {
                if sink.send(m).await.is_err() {
                    return;
                }
            }
        });
        let reader = tokio::spawn(async move {
            loop {
                if stop.load(Ordering::SeqCst) {
                    // Reads nothing more, so answers no keepalive.
                    std::future::pending::<()>().await;
                }
                let Some(m) = source.next().await else { break };
                let item = match m {
                    Ok(Message::Binary(b)) if lobby::wire::is_lobby(&b) => {
                        Incoming::Lobby(lobby::wire::decode(&b).expect("the relay's lobby message"))
                    }
                    Ok(Message::Binary(b)) => Incoming::Game(b.to_vec()),
                    Ok(Message::Close(f)) => {
                        let _ = inc.send(Incoming::Closed(f.map(|f| f.reason.to_string())));
                        return;
                    }
                    Ok(_) => continue,
                    Err(_) => break,
                };
                if inc.send(item).is_err() {
                    return;
                }
            }
            let _ = inc.send(Incoming::Closed(None));
        });
        Client {
            tx,
            rx,
            tasks: [writer, reader],
            paused,
        }
    }

    /// Sends one binary message.
    pub fn send(&self, bytes: Vec<u8>) {
        let _ = self.tx.send(Message::Binary(Bytes::from(bytes)));
    }

    /// Sends a lobby message.
    pub fn lobby(&self, m: &ToRelay) {
        self.send(lobby::wire::encode(m));
    }

    /// Sends a text message (not ours).
    pub fn text(&self, s: &str) {
        let _ = self.tx.send(Message::text(s));
    }

    /// The next thing that arrives; fails the test after [`WAIT`].
    pub async fn next(&mut self) -> Incoming {
        tokio::time::timeout(WAIT, self.rx.recv())
            .await
            .expect("a message in time")
            .unwrap_or(Incoming::Closed(None))
    }

    /// The next lobby message, skipping game messages.
    pub async fn next_lobby(&mut self) -> FromRelay {
        loop {
            match self.next().await {
                Incoming::Lobby(m) => return m,
                Incoming::Game(_) => continue,
                Incoming::Closed(r) => panic!("closed while waiting: {r:?}"),
            }
        }
    }

    /// What has arrived, without waiting.
    pub fn try_next(&mut self) -> Option<Incoming> {
        self.rx.try_recv().ok()
    }

    /// Waits for the close and returns its reason, after the relay's
    /// `Closing` message with the same reason.
    pub async fn closed(&mut self) -> String {
        let mut closing = None;
        loop {
            match self.next().await {
                Incoming::Lobby(FromRelay::Closing(r)) => closing = Some(r),
                Incoming::Closed(r) => {
                    let r = r.expect("a close reason");
                    assert_eq!(
                        closing.as_deref(),
                        Some(r.as_str()),
                        "Closing, then the close"
                    );
                    return r;
                }
                _ => {}
            }
        }
    }

    /// Stops reading (after the message being read, if any): the client
    /// keeps its socket open but answers no keepalive.
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    /// Cuts the connection at once, as a crashed client would.
    pub fn kill(self) {
        for t in &self.tasks {
            t.abort();
        }
    }
}

/// A setup with these rows and the rest closed.
pub fn setup(rows: &[SlotKind]) -> Setup {
    let mut slots = [Slot::closed(); 8];
    for (s, &k) in slots.iter_mut().zip(rows) {
        s.kind = k;
    }
    Setup {
        map: 0,
        slots,
        credits: 2000,
        units: 3,
        flags: [false, true, true, true, true],
        speed: 2,
    }
}

/// The map every test room uses.
pub fn map_choice(hash: &str, starts: u32) -> MapChoice {
    MapChoice {
        hash: hash.into(),
        label: "synthetic".into(),
        starts,
    }
}

/// A room of these rows on map `hash`.
pub fn create(rows: &[SlotKind], hash: &str, starts: u32) -> ToRelay {
    ToRelay::Create {
        name: "Host".into(),
        build: "build-1".into(),
        rules_hash: "rules-1".into(),
        offered: Offered {
            countries: 2,
            credits: (1000, 5000),
            units: (1, 9),
        },
        map: map_choice(hash, starts),
        setup: setup(rows),
    }
}

/// A join of room `code` with this build and map.
pub fn join(code: &str, name: &str, build: &str, map_hash: &str) -> ToRelay {
    ToRelay::Join {
        code: code.into(),
        name: name.into(),
        build: build.into(),
        rules_hash: "rules-1".into(),
        map_hash: map_hash.into(),
    }
}

/// Ready for map `hash`.
pub fn ready(hash: &str) -> ToRelay {
    ToRelay::Ready {
        ready: true,
        map_hash: hash.into(),
    }
}
