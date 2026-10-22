//! The native client's direct link to one other peer (M9 N7): a WebRTC
//! data channel through `matchbox_socket` and webrtc-rs, signalled through
//! the relay's WebSocket.
//!
//! One [`Pair`] is one attempt at a link between two peers of a game. The
//! client (`glasnost-app`, `game::direct`) carries its offer, answer and ICE
//! candidates ([`Signal`]) through the relay as lobby messages and moves
//! packets; this crate never opens a connection to a signalling server of
//! its own: matchbox's signaller is replaced by two in-process queues, so
//! its WebSocket client is compiled but never used.
//!
//! The channel is unordered and unreliable (negotiated id 0, no
//! retransmits): a lost packet is not sent again, because every frame
//! also goes through the relay and the first copy wins
//! (`glasnost_net::session`, N7).
//!
//! ICE: only the servers of [`Ice`], none by default (host candidates: a
//! link forms on a LAN or loopback, or not at all). matchbox's own default
//! (two public STUN servers) is never used, so nothing here contacts a
//! server the person did not name. webrtc-rs answers multicast DNS queries
//! for `.local` candidates on the local network only.
//!
//! Each attempt runs matchbox's message loop on its own thread under
//! `catch_unwind`: a peer's hostile SDP that makes the loop panic ends that
//! attempt only. The webrtc-rs side runs on the tokio runtime that
//! matchbox's `async-compat` starts once per process. Our own netcode; no
//! spec applies.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

use futures::StreamExt;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use futures::channel::oneshot;
use futures::future::{Either, select};
use glasnost_net::lobby::Signal;
use matchbox_socket::async_trait::async_trait;
use matchbox_socket::{
    ChannelConfig, PeerEvent, PeerId, PeerRequest, PeerSignal, PeerState, RtcIceServerConfig,
    SignalingError, Signaller, SignallerBuilder, WebRtcSocket,
};

/// The largest packet a link sends or takes: a lockstep frames message is
/// far smaller; anything larger is dropped.
// nocite: our own netcode (16 KiB, the size every WebRTC stack passes unfragmented)
pub const MAX_PACKET: usize = 16 * 1024;

/// The ICE servers of a link: STUN or TURN URLs (`stun:host:port`,
/// `turn:host:port`), with the TURN server's user and password. None by
/// default.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ice {
    /// The servers' URLs.
    pub urls: Vec<String>,
    /// The TURN user, if any.
    pub username: Option<String>,
    /// The TURN password, if any.
    pub credential: Option<String>,
}

/// Where an attempt is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// The offer, the answer and ICE are under way.
    Connecting,
    /// The data channel is open.
    Open,
    /// The attempt ended, and why.
    Closed(String),
}

/// This end's id in matchbox's terms; the ids never leave the process.
const ME: u128 = 1; // nocite: our own netcode (any two distinct ids)
/// The other end's id.
const PEER: u128 = 2; // nocite: our own netcode

/// The signaller matchbox's message loop talks to: what the peer sent
/// (through the relay) comes in, what is for the peer goes out.
struct Queues {
    from_peer: UnboundedReceiver<PeerEvent>,
    to_peer: UnboundedSender<PeerRequest>,
}

#[async_trait]
impl Signaller for Queues {
    async fn send(&mut self, request: PeerRequest) -> Result<(), SignalingError> {
        // A request after the pair is gone has nobody to go to.
        let _ = self.to_peer.unbounded_send(request);
        Ok(())
    }

    async fn next_message(&mut self) -> Result<PeerEvent, SignalingError> {
        self.from_peer
            .next()
            .await
            .ok_or(SignalingError::StreamExhausted)
    }
}

/// Hands matchbox the one signaller of an attempt.
struct Once(Mutex<Option<Queues>>);

impl std::fmt::Debug for Once {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Once")
    }
}

#[async_trait]
impl SignallerBuilder for Once {
    async fn new_signaller(
        &self,
        _attempts: Option<u16>,
        _url: String,
    ) -> Result<Box<dyn Signaller>, SignalingError> {
        let q = self.0.lock().ok().and_then(|mut g| g.take());
        match q {
            Some(q) => Ok(Box::new(q)),
            None => Err(SignalingError::StreamExhausted),
        }
    }
}

/// One attempt at a direct link to one peer.
pub struct Pair {
    socket: Option<WebRtcSocket>,
    to_loop: UnboundedSender<PeerEvent>,
    from_loop: UnboundedReceiver<PeerRequest>,
    stop: Option<oneshot::Sender<()>>,
    /// Why the message loop ended, once it has.
    ended: Arc<Mutex<Option<String>>>,
    state: State,
    inbox: VecDeque<Vec<u8>>,
}

impl Pair {
    /// Starts an attempt: the offering end (the lower slot) makes the offer
    /// at once, the other waits for it.
    pub fn open(ice: &Ice, offerer: bool) -> Self {
        let (to_loop, from_peer) = unbounded();
        let (to_peer, from_loop) = unbounded();
        let me = PeerId(uuid::Uuid::from_u128(ME));
        let _ = to_loop.unbounded_send(PeerEvent::IdAssigned(me));
        if offerer {
            let _ = to_loop.unbounded_send(PeerEvent::NewPeer(peer()));
        }
        let queues = Queues { from_peer, to_peer };
        let (socket, run) = WebRtcSocket::builder("glasnost:")
            .signaller_builder(Arc::new(Once(Mutex::new(Some(queues)))))
            .ice_server(RtcIceServerConfig {
                urls: ice.urls.clone(),
                username: ice.username.clone(),
                credential: ice.credential.clone(),
            })
            .reconnect_attempts(None)
            .signaling_keep_alive_interval(None)
            .add_channel(ChannelConfig::unreliable())
            .build();
        let ended = Arc::new(Mutex::new(None));
        let (stop, stopped) = oneshot::channel::<()>();
        let shared = Arc::clone(&ended);
        let spawned = std::thread::Builder::new()
            .name("glasnost-rtc".into())
            .spawn(move || {
                let r = catch_unwind(AssertUnwindSafe(move || {
                    futures::executor::block_on(select(run, stopped))
                }));
                let why = match r {
                    Ok(Either::Left((Ok(()), _))) => "the link's loop ended".to_owned(),
                    Ok(Either::Left((Err(e), _))) => format!("the link failed: {e}"),
                    Ok(Either::Right(_)) => "closed".to_owned(),
                    Err(_) => "the link's loop failed on what the peer sent".to_owned(),
                };
                if let Ok(mut g) = shared.lock() {
                    *g = Some(why);
                }
            });
        let state = match spawned {
            Ok(_) => State::Connecting,
            Err(e) => State::Closed(format!("no thread for the link: {e}")),
        };
        Self {
            socket: Some(socket),
            to_loop,
            from_loop,
            stop: Some(stop),
            ended,
            state,
            inbox: VecDeque::new(),
        }
    }

    /// Takes a signal the peer sent through the relay.
    pub fn signal(&mut self, s: Signal) {
        let data = match s {
            Signal::Offer(sdp) => PeerSignal::Offer(sdp),
            Signal::Answer(sdp) => PeerSignal::Answer(sdp),
            Signal::Candidate(c) => PeerSignal::IceCandidate(c),
            // The pair takes a decline itself (app::game::direct); a link
            // never sees one.
            Signal::Decline => return,
        };
        let _ = self.to_loop.unbounded_send(PeerEvent::Signal {
            sender: peer(),
            data,
        });
    }

    /// The next signal for the peer, to send through the relay.
    pub fn take_signal(&mut self) -> Option<Signal> {
        while let Ok(r) = self.from_loop.try_recv() {
            if let PeerRequest::Signal { data, .. } = r {
                return Some(match data {
                    PeerSignal::Offer(sdp) => Signal::Offer(sdp),
                    PeerSignal::Answer(sdp) => Signal::Answer(sdp),
                    PeerSignal::IceCandidate(c) => Signal::Candidate(c),
                });
            }
        }
        None
    }

    /// Looks at the loop and the channel: the state changes, and what
    /// arrived is kept for [`Pair::recv`].
    pub fn poll(&mut self) {
        if matches!(self.state, State::Closed(_)) {
            return;
        }
        let ended = self.ended.lock().ok().and_then(|g| g.clone());
        if let Some(why) = ended {
            return self.end(why);
        }
        let Some(socket) = self.socket.as_mut() else {
            return self.end("closed".into());
        };
        match socket.try_update_peers() {
            Ok(changes) => {
                for (_, s) in changes {
                    self.state = match s {
                        PeerState::Connected => State::Open,
                        PeerState::Disconnected => State::Closed("the peer's link closed".into()),
                    };
                }
            }
            Err(_) => return self.end("the link's loop ended".into()),
        }
        if let Ok(ch) = socket.get_channel_mut(0) {
            for (_, p) in ch.receive() {
                if p.len() <= MAX_PACKET {
                    self.inbox.push_back(p.into_vec());
                }
            }
        }
        if let State::Closed(why) = self.state.clone() {
            self.end(why);
        }
    }

    /// Where the attempt is (as of the last [`Pair::poll`]).
    pub fn state(&self) -> State {
        self.state.clone()
    }

    /// Sends one packet, if the channel is open; it may be lost.
    pub fn send(&mut self, bytes: Vec<u8>) {
        if self.state != State::Open || bytes.len() > MAX_PACKET {
            return;
        }
        if let Some(ch) = self.socket.as_mut().and_then(|s| s.get_channel_mut(0).ok())
            && ch.try_send(bytes.into_boxed_slice(), peer()).is_err()
        {
            self.end("the link's loop ended".into());
        }
    }

    /// The next packet that arrived, oldest first.
    pub fn recv(&mut self) -> Option<Vec<u8>> {
        self.inbox.pop_front()
    }

    /// Ends the attempt: the loop stops and the connection goes.
    pub fn close(&mut self) {
        self.end("closed".into());
    }

    fn end(&mut self, why: String) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        self.to_loop.close_channel();
        self.socket = None;
        if !matches!(self.state, State::Closed(_)) {
            self.state = State::Closed(why);
        }
    }
}

impl Drop for Pair {
    fn drop(&mut self) {
        self.close();
    }
}

fn peer() -> PeerId {
    PeerId(uuid::Uuid::from_u128(PEER))
}
