//! The relay's tokio shell: a WebSocket listener that feeds the [`Hub`].
//!
//! One task owns the hub and takes events from every connection, in the
//! order they come, plus a clock tick. Each connection has a reader, which
//! hands the hub every message, and a writer, which drains a bounded queue
//! of what the hub sends it. A connection whose queue is full (a client
//! that does not read) is closed as too slow, so one client can never hold
//! up the others.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Semaphore, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::{Bytes, Error as WsError, Message};

use crate::hub::{ConnId, Entropy, Hub, Limits, Out};

/// The operating system's randomness.
pub struct OsEntropy;

impl Entropy for OsEntropy {
    fn fill(&mut self, buf: &mut [u8]) {
        // The relay cannot make room codes nobody can guess without it.
        getrandom::fill(buf).expect("the operating system's randomness");
    }
}

/// How long a client may take over the WebSocket handshake.
// nocite: our own netcode
const HANDSHAKE: Duration = Duration::from_secs(10);

/// How often the hub's clock ticks.
// nocite: our own netcode
const TICK: Duration = Duration::from_millis(50);

/// Messages queued for one connection before it is closed as too slow.
// nocite: our own netcode
const QUEUE: usize = 256;

/// Events queued for the hub before the readers wait.
// nocite: our own netcode
const EVENTS: usize = 4096;

/// Handshakes and connections at once above the connection limit, so that
/// the hub, not the listener, refuses them with a reason.
// nocite: our own netcode
const SPARE: usize = 64;

/// How long a connection being closed is given to answer the close (or,
/// when it sent too large a message, to receive it).
// nocite: our own netcode
const LINGER: Duration = Duration::from_secs(1);

/// How long one message may take to leave before the connection is cut.
// nocite: our own netcode
const SEND: Duration = Duration::from_secs(5);

enum Outgoing {
    Binary(Vec<u8>),
    Ping,
    Close(String),
}

enum Event {
    Open {
        tx: mpsc::Sender<Outgoing>,
        reply: oneshot::Sender<Option<ConnId>>,
    },
    Message(ConnId, Vec<u8>),
    Heard(ConnId),
    TooLarge(ConnId),
    Closed(ConnId),
}

/// A running relay.
pub struct Relay {
    /// The address it listens on.
    pub addr: SocketAddr,
    task: JoinHandle<()>,
}

impl Relay {
    /// Waits until the relay stops (it runs until the task is aborted).
    pub async fn wait(self) {
        let _ = self.task.await;
    }

    /// Stops the relay: no new connection is accepted; the open ones close
    /// as their tasks notice.
    pub fn stop(&self) {
        self.task.abort();
    }
}

/// Binds `addr` and serves the relay on the current tokio runtime with
/// host randomness from the operating system.
pub async fn bind(addr: &str, limits: Limits) -> std::io::Result<Relay> {
    let mut probe = [0u8; 1];
    getrandom::fill(&mut probe).map_err(|e| std::io::Error::other(e.to_string()))?;
    bind_with(addr, limits, Box::new(OsEntropy)).await
}

/// Binds `addr` and serves the relay with this randomness (tests use a
/// seeded one).
pub async fn bind_with(
    addr: &str,
    limits: Limits,
    entropy: Box<dyn Entropy>,
) -> std::io::Result<Relay> {
    let listener = TcpListener::bind(addr).await?;
    let addr = listener.local_addr()?;
    let task = tokio::spawn(serve(listener, limits, entropy));
    Ok(Relay { addr, task })
}

/// Serves the relay on `listener` until the task is dropped.
pub async fn serve(listener: TcpListener, limits: Limits, entropy: Box<dyn Entropy>) {
    let (events, rx) = mpsc::channel(EVENTS);
    let hub = tokio::spawn(run_hub(Hub::new(limits, entropy), rx));
    let permits = Arc::new(Semaphore::new(limits.max_connections + SPARE));
    let config = WebSocketConfig::default()
        .max_message_size(Some(limits.max_message))
        .max_frame_size(Some(limits.max_message));
    // The hub stops with the listener.
    let _guard = AbortOnDrop(hub);
    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(_) => {
                // Out of file descriptors, say: wait instead of spinning.
                tokio::time::sleep(TICK).await;
                continue;
            }
        };
        let Ok(permit) = permits.clone().try_acquire_owned() else {
            drop(stream);
            continue;
        };
        let events = events.clone();
        tokio::spawn(async move {
            connection(stream, config, events).await;
            drop(permit);
        });
    }
}

struct AbortOnDrop(JoinHandle<()>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn connection(stream: TcpStream, config: WebSocketConfig, events: mpsc::Sender<Event>) {
    let _ = stream.set_nodelay(true);
    let accept = tokio_tungstenite::accept_async_with_config(stream, Some(config));
    let Ok(Ok(ws)) = tokio::time::timeout(HANDSHAKE, accept).await else {
        return;
    };
    let (mut sink, mut source) = ws.split();
    let (tx, mut rx) = mpsc::channel(QUEUE);
    let (reply, id) = oneshot::channel();
    if events.send(Event::Open { tx, reply }).await.is_err() {
        return;
    }
    let Ok(Some(id)) = id.await else {
        let full = sink.send(close_message("the relay is full".to_owned()));
        let _ = tokio::time::timeout(SEND, full).await;
        return;
    };
    let writer = async {
        while let Some(out) = rx.recv().await {
            let msg = match out {
                Outgoing::Binary(b) => Message::Binary(Bytes::from(b)),
                Outgoing::Ping => Message::Ping(Bytes::new()),
                Outgoing::Close(reason) => {
                    // The client answers the close (the reader then ends)
                    // or is cut off after a while.
                    let _ = tokio::time::timeout(SEND, sink.send(close_message(reason))).await;
                    tokio::time::sleep(LINGER).await;
                    return;
                }
            };
            // A client that does not read would hold the writer here, and
            // the connection's task, for good.
            if !matches!(tokio::time::timeout(SEND, sink.send(msg)).await, Ok(Ok(()))) {
                return;
            }
        }
    };
    let reader = async {
        while let Some(msg) = source.next().await {
            let event = match msg {
                Ok(Message::Binary(b)) => Event::Message(id, b.to_vec()),
                // Not ours: the hub closes the connection with the reason.
                Ok(Message::Text(t)) => Event::Message(id, t.as_bytes().to_vec()),
                Ok(Message::Ping(_) | Message::Pong(_)) => Event::Heard(id),
                Ok(Message::Frame(_)) => continue,
                Ok(Message::Close(_)) => return,
                Err(WsError::Capacity(_)) => {
                    let _ = events.send(Event::TooLarge(id)).await;
                    tokio::time::sleep(LINGER).await;
                    return;
                }
                Err(_) => return,
            };
            if events.send(event).await.is_err() {
                return;
            }
        }
    };
    tokio::select! {
        () = writer => {}
        () = reader => {}
    }
    let _ = events.send(Event::Closed(id)).await;
}

fn close_message(reason: String) -> Message {
    Message::Close(Some(CloseFrame {
        code: CloseCode::Policy,
        reason: reason.into(),
    }))
}

async fn run_hub(mut hub: Hub, mut events: mpsc::Receiver<Event>) {
    let start = Instant::now();
    let now = || u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let mut conns: std::collections::BTreeMap<ConnId, mpsc::Sender<Outgoing>> =
        std::collections::BTreeMap::new();
    let mut tick = tokio::time::interval(TICK);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            ev = events.recv() => {
                let Some(ev) = ev else { return };
                let t = now();
                match ev {
                    Event::Open { tx, reply } => {
                        let id = hub.connect(t);
                        if let Some(id) = id {
                            conns.insert(id, tx);
                        }
                        let _ = reply.send(id);
                    }
                    Event::Message(id, bytes) => hub.receive(t, id, &bytes),
                    Event::Heard(id) => hub.heard(t, id),
                    Event::TooLarge(id) => hub.too_large(t, id),
                    Event::Closed(id) => {
                        conns.remove(&id);
                        hub.closed(t, id);
                    }
                }
            }
            _ = tick.tick() => hub.tick(now()),
        }
        dispatch(&mut hub, &mut conns, now());
    }
}

/// Carries out what the hub asked for. A connection whose queue is full is
/// closed as too slow, which may ask for more.
fn dispatch(
    hub: &mut Hub,
    conns: &mut std::collections::BTreeMap<ConnId, mpsc::Sender<Outgoing>>,
    now: u64,
) {
    loop {
        let outs = hub.take_out();
        if outs.is_empty() {
            return;
        }
        for out in outs {
            match out {
                Out::Send { to, bytes } => {
                    let Some(tx) = conns.get(&to) else { continue };
                    if tx.try_send(Outgoing::Binary(bytes)).is_err() {
                        conns.remove(&to);
                        hub.kick(now, to, "too slow");
                    }
                }
                Out::Keepalive { to } => {
                    let Some(tx) = conns.get(&to) else { continue };
                    if tx.try_send(Outgoing::Ping).is_err() {
                        conns.remove(&to);
                        hub.kick(now, to, "too slow");
                    }
                }
                Out::Close { to, reason } => {
                    // A full queue drops the sender instead: the writer
                    // then ends and the socket closes without the reason.
                    if let Some(tx) = conns.remove(&to) {
                        let _ = tx.try_send(Outgoing::Close(reason));
                    }
                }
            }
        }
    }
}
