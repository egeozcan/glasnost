//! The browser's direct links (M9 N7): one `RTCPeerConnection` per attempt
//! with one data channel, negotiated (id 0 on both ends, so neither end
//! waits for the other to announce it), unordered and never retransmitted:
//! the relay carries every frame anyway, so a lost packet costs nothing
//! and a late one is worth nothing.
//!
//! - ICE servers: only the ones the client names (none by default: host
//!   candidates only, so nothing outside the two peers and the relay is
//!   contacted). An empty URL list is left out entirely, because browsers
//!   reject an `iceServers` entry without URLs.
//! - Signals: the offer and the answer as SDP text, candidates as the
//!   JSON of `RTCIceCandidate.toJSON()` (the same fields the native
//!   `glasnost-rtc` writes, so a browser and a native client can link).
//!   Candidates either way wait until the remote description is set.
//! - A browser without `RTCPeerConnection` (or one that refuses to make
//!   one) closes the attempt at once; the pair stays on the relay.
//! - Callbacks only queue; the client takes what arrived once per frame.
//!   Dropping the link closes the connection and unhooks the callbacks.
//!
//! Our own netcode; no spec applies.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::rc::Rc;

use glasnost_app::game::direct::{PairInfo, PairLink, PairState};
use glasnost_net::lobby::Signal;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
    Event, MessageEvent, RtcConfiguration, RtcDataChannel, RtcDataChannelInit, RtcDataChannelState,
    RtcDataChannelType, RtcIceCandidateInit, RtcPeerConnection, RtcPeerConnectionIceEvent,
    RtcPeerConnectionState, RtcSdpType, RtcSessionDescriptionInit,
};

/// The largest packet a link takes in; a frame message is far smaller.
// nocite: our own netcode (glasnost-rtc's own limit)
const MAX_PACKET: u32 = 16 * 1024;

/// Packets that may wait between two frames of the client; more are
/// dropped (a peer flooding its link).
// nocite: our own netcode
const MAX_INBOX: usize = 1024;

/// How much may wait in the channel's send buffer; past it a packet is
/// dropped rather than queued (it would only come late).
// nocite: our own netcode
const MAX_BUFFERED: u32 = 256 * 1024;

/// What the callbacks and the promises share with the link.
struct Shared {
    state: PairState,
    inbox: VecDeque<Vec<u8>>,
    /// Signals for the peer.
    out: VecDeque<Signal>,
    /// Whether the remote description is set (the peer's candidates may
    /// be added).
    remote: bool,
    /// Whether this end's candidates may go: after the offer was answered,
    /// or after this end's answer.
    sending: bool,
    /// The peer's candidates, waiting for the remote description.
    early: Vec<String>,
    /// This end's candidates, waiting for [`Shared::sending`].
    local: Vec<String>,
}

impl Shared {
    fn fail(&mut self, why: String) {
        if !matches!(self.state, PairState::Closed(_)) {
            self.state = PairState::Closed(why);
        }
    }

    fn closed(&self) -> bool {
        matches!(self.state, PairState::Closed(_))
    }
}

/// One attempt at a direct link in the browser.
pub struct WebPair {
    pc: Option<RtcPeerConnection>,
    dc: Option<RtcDataChannel>,
    offerer: bool,
    shared: Rc<RefCell<Shared>>,
    _on_candidate: Option<Closure<dyn FnMut(RtcPeerConnectionIceEvent)>>,
    _on_state: Option<Closure<dyn FnMut(Event)>>,
    _on_open: Option<Closure<dyn FnMut(Event)>>,
    _on_message: Option<Closure<dyn FnMut(MessageEvent)>>,
    _on_close: Option<Closure<dyn FnMut(Event)>>,
}

/// Whether this browser (or worker, or node) has `RTCPeerConnection`.
pub fn available() -> bool {
    js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("RTCPeerConnection"))
        .is_ok_and(|v| v.is_function())
}

/// The configuration: the named ICE servers, else none at all.
fn configuration(info: &PairInfo) -> RtcConfiguration {
    let config = RtcConfiguration::new();
    let servers = js_sys::Array::new();
    if !info.ice.urls.is_empty() {
        let server = js_sys::Object::new();
        let urls: js_sys::Array = info.ice.urls.iter().map(|u| JsValue::from_str(u)).collect();
        let _ = js_sys::Reflect::set(&server, &"urls".into(), &urls);
        if let Some(u) = info.ice.username.as_deref() {
            let _ = js_sys::Reflect::set(&server, &"username".into(), &u.into());
        }
        if let Some(c) = info.ice.credential.as_deref() {
            let _ = js_sys::Reflect::set(&server, &"credential".into(), &c.into());
        }
        servers.push(&server);
    }
    config.set_ice_servers(&servers);
    config
}

/// Opens an attempt ([`glasnost_app::game::direct::OpenPair`]).
pub fn open(info: &PairInfo) -> Box<dyn PairLink> {
    let shared = Rc::new(RefCell::new(Shared {
        state: PairState::Connecting,
        inbox: VecDeque::new(),
        out: VecDeque::new(),
        remote: false,
        sending: false,
        early: Vec::new(),
        local: Vec::new(),
    }));
    let mut link = WebPair {
        pc: None,
        dc: None,
        offerer: info.offerer,
        shared: Rc::clone(&shared),
        _on_candidate: None,
        _on_state: None,
        _on_open: None,
        _on_message: None,
        _on_close: None,
    };
    if !available() {
        shared
            .borrow_mut()
            .fail("this browser has no WebRTC (RTCPeerConnection)".into());
        return Box::new(link);
    }
    let pc = match RtcPeerConnection::new_with_configuration(&configuration(info)) {
        Ok(pc) => pc,
        Err(e) => {
            shared
                .borrow_mut()
                .fail(format!("the browser made no RTCPeerConnection: {e:?}"));
            return Box::new(link);
        }
    };
    let init = RtcDataChannelInit::new();
    init.set_negotiated(true);
    init.set_id(0); // nocite: our own netcode (the one negotiated channel)
    init.set_ordered(false);
    init.set_max_retransmits(0); // nocite: our own netcode (never resent: the relay has it)
    let dc = pc.create_data_channel_with_data_channel_dict("glasnost", &init);
    dc.set_binary_type(RtcDataChannelType::Arraybuffer);

    let s = Rc::clone(&shared);
    let on_candidate = Closure::<dyn FnMut(RtcPeerConnectionIceEvent)>::new(
        move |e: RtcPeerConnectionIceEvent| {
            // No candidate: gathering ended.
            let Some(c) = e.candidate() else {
                return;
            };
            if c.candidate().is_empty() {
                return;
            }
            let Some(json) = js_sys::JSON::stringify(&c.to_json())
                .ok()
                .and_then(|j| j.as_string())
            else {
                return;
            };
            let mut s = s.borrow_mut();
            if s.sending {
                s.out.push_back(Signal::Candidate(json));
            } else if s.local.len() < MAX_INBOX {
                s.local.push(json);
            }
        },
    );
    let s = Rc::clone(&shared);
    let pc2 = pc.clone();
    let on_state = Closure::<dyn FnMut(Event)>::new(move |_| {
        // "disconnected" may come back; the client's own silence timer
        // decides. "failed" and "closed" do not.
        match pc2.connection_state() {
            RtcPeerConnectionState::Failed => s.borrow_mut().fail("ICE failed".into()),
            RtcPeerConnectionState::Closed => s.borrow_mut().fail("the connection closed".into()),
            _ => {}
        }
    });
    let s = Rc::clone(&shared);
    let on_open = Closure::<dyn FnMut(Event)>::new(move |_| {
        let mut s = s.borrow_mut();
        // A channel that opens after the attempt ended stays ended.
        if s.state == PairState::Connecting {
            s.state = PairState::Open;
        }
    });
    let s = Rc::clone(&shared);
    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        // Text messages are not ours.
        if let Ok(buf) = e.data().dyn_into::<js_sys::ArrayBuffer>()
            && buf.byte_length() <= MAX_PACKET
        {
            let mut s = s.borrow_mut();
            if s.inbox.len() < MAX_INBOX {
                s.inbox.push_back(js_sys::Uint8Array::new(&buf).to_vec());
            }
        }
    });
    let s = Rc::clone(&shared);
    let on_close = Closure::<dyn FnMut(Event)>::new(move |_| {
        s.borrow_mut().fail("the channel closed".into());
    });
    pc.set_onicecandidate(Some(on_candidate.as_ref().unchecked_ref()));
    pc.set_onconnectionstatechange(Some(on_state.as_ref().unchecked_ref()));
    dc.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    dc.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    dc.set_onclose(Some(on_close.as_ref().unchecked_ref()));

    if info.offerer {
        let pc2 = pc.clone();
        let s = Rc::clone(&shared);
        run(&shared, "the offer", async move {
            let offer = JsFuture::from(pc2.create_offer()).await?;
            let offer: RtcSessionDescriptionInit = offer.unchecked_into();
            JsFuture::from(pc2.set_local_description(&offer)).await?;
            let sdp = pc2.local_description().map(|d| d.sdp()).unwrap_or_default();
            let mut s = s.borrow_mut();
            if !s.closed() {
                s.out.push_back(Signal::Offer(sdp));
            }
            Ok(())
        });
    }

    link.pc = Some(pc);
    link.dc = Some(dc);
    link._on_candidate = Some(on_candidate);
    link._on_state = Some(on_state);
    link._on_open = Some(on_open);
    link._on_message = Some(on_message);
    link._on_close = Some(on_close);
    Box::new(link)
}

/// Runs a promise chain; its failure ends the attempt.
fn run(
    shared: &Rc<RefCell<Shared>>,
    what: &'static str,
    fut: impl Future<Output = Result<(), JsValue>> + 'static,
) {
    let s = Rc::clone(shared);
    spawn_local(async move {
        if let Err(e) = fut.await {
            s.borrow_mut().fail(format!("{what} failed: {e:?}"));
        }
    });
}

/// The remote description is set: the peer's waiting candidates go in.
fn remote_set(pc: &RtcPeerConnection, s: &Rc<RefCell<Shared>>) {
    let early = {
        let mut s = s.borrow_mut();
        s.remote = true;
        std::mem::take(&mut s.early)
    };
    for c in early {
        add_candidate(pc, &c);
    }
}

/// This end's candidates may go: the waiting ones go now.
fn start_sending(s: &mut Shared) {
    s.sending = true;
    let local = std::mem::take(&mut s.local);
    s.out.extend(local.into_iter().map(Signal::Candidate));
}

/// Adds a peer's candidate; one that does not read is ignored (the
/// attempt may still open on the others).
fn add_candidate(pc: &RtcPeerConnection, json: &str) {
    let Ok(v) = js_sys::JSON::parse(json) else {
        return;
    };
    if !v.is_object() {
        return;
    }
    let init: RtcIceCandidateInit = v.unchecked_into();
    let p = pc.add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(&init));
    spawn_local(async move {
        let _ = JsFuture::from(p).await;
    });
}

impl PairLink for WebPair {
    fn poll(&mut self, _now_ms: u64) {}

    fn state(&self) -> PairState {
        self.shared.borrow().state.clone()
    }

    fn send(&mut self, bytes: Vec<u8>) {
        if self.shared.borrow().state != PairState::Open {
            return;
        }
        if let Some(dc) = self.dc.as_ref()
            && dc.ready_state() == RtcDataChannelState::Open
            && dc.buffered_amount() <= MAX_BUFFERED
        {
            let _ = dc.send_with_u8_array(&bytes);
        }
    }

    fn recv(&mut self) -> Option<Vec<u8>> {
        self.shared.borrow_mut().inbox.pop_front()
    }

    fn signal(&mut self, sig: Signal) {
        let Some(pc) = self.pc.clone() else {
            return;
        };
        if self.shared.borrow().closed() {
            return;
        }
        match sig {
            Signal::Offer(sdp) => {
                // Only the answering end takes an offer, and only one.
                if self.offerer || self.shared.borrow().remote {
                    return;
                }
                let s = Rc::clone(&self.shared);
                run(&self.shared, "the answer", async move {
                    let offer = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
                    offer.set_sdp(&sdp);
                    JsFuture::from(pc.set_remote_description(&offer)).await?;
                    remote_set(&pc, &s);
                    let answer = JsFuture::from(pc.create_answer()).await?;
                    let answer: RtcSessionDescriptionInit = answer.unchecked_into();
                    JsFuture::from(pc.set_local_description(&answer)).await?;
                    let sdp = pc.local_description().map(|d| d.sdp()).unwrap_or_default();
                    let mut s = s.borrow_mut();
                    if !s.closed() {
                        // The answer goes before this end's candidates.
                        s.out.push_back(Signal::Answer(sdp));
                        start_sending(&mut s);
                    }
                    Ok(())
                });
            }
            Signal::Answer(sdp) => {
                if !self.offerer || self.shared.borrow().remote {
                    return;
                }
                let s = Rc::clone(&self.shared);
                run(&self.shared, "the peer's answer", async move {
                    let answer = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
                    answer.set_sdp(&sdp);
                    JsFuture::from(pc.set_remote_description(&answer)).await?;
                    remote_set(&pc, &s);
                    start_sending(&mut s.borrow_mut());
                    Ok(())
                });
            }
            Signal::Candidate(json) => {
                let mut s = self.shared.borrow_mut();
                if s.remote {
                    drop(s);
                    add_candidate(&pc, &json);
                } else if s.early.len() < MAX_INBOX {
                    s.early.push(json);
                }
            }
        }
    }

    fn take_signal(&mut self) -> Option<Signal> {
        self.shared.borrow_mut().out.pop_front()
    }

    fn close(&mut self) {
        self.shared.borrow_mut().fail("closed".into());
        if let Some(dc) = self.dc.as_ref() {
            dc.close();
        }
        if let Some(pc) = self.pc.as_ref() {
            pc.close();
        }
    }
}

impl Drop for WebPair {
    fn drop(&mut self) {
        if let Some(dc) = self.dc.take() {
            dc.set_onopen(None);
            dc.set_onmessage(None);
            dc.set_onclose(None);
            dc.close();
        }
        if let Some(pc) = self.pc.take() {
            pc.set_onicecandidate(None);
            pc.set_onconnectionstatechange(None);
            pc.close();
        }
    }
}

/// The browser lane's check of this module (`web-tests/rtc.mjs`, on the
/// page `?mode=probe`): two attempts in one page, signalled to each other
/// directly, must open and carry a packet each way; then an attempt fed
/// an offer that does not read must end, not panic. The answer is a line
/// of `key=value` words (`open=MS`, `echo=yes`, `hostile=closed`).
#[cfg(feature = "start")]
#[wasm_bindgen]
pub async fn rtc_probe(timeout_ms: u32) -> String {
    use glasnost_app::game::direct::Ice;
    let info = |me, peer, offerer| PairInfo {
        me,
        peer,
        offerer,
        attempt: 1,
        ice: Ice::default(),
    };
    if !available() {
        return "unavailable".into();
    }
    let mut a = open(&info(0, 1, true));
    let mut b = open(&info(1, 0, false));
    let start = js_sys::Date::now();
    let mut words = Vec::new();
    let mut sent = false;
    let (mut at_b, mut at_a) = (false, false);
    loop {
        sleep(5).await;
        while let Some(s) = a.take_signal() {
            b.signal(s);
        }
        while let Some(s) = b.take_signal() {
            a.signal(s);
        }
        if let (PairState::Closed(why), _) | (_, PairState::Closed(why)) = (a.state(), b.state()) {
            words.push(format!("closed={why:?}"));
            break;
        }
        if a.state() == PairState::Open && b.state() == PairState::Open {
            if !sent {
                words.push(format!("open={}", (js_sys::Date::now() - start) as u64));
                a.send(b"ab".to_vec());
                b.send(b"ba".to_vec());
                sent = true;
            }
            at_b |= b.recv().is_some_and(|p| p == b"ab");
            at_a |= a.recv().is_some_and(|p| p == b"ba");
            if at_a && at_b {
                words.push("echo=yes".into());
                break;
            }
        }
        if js_sys::Date::now() - start > f64::from(timeout_ms) {
            words.push(format!("timeout a={:?} b={:?}", a.state(), b.state()));
            break;
        }
    }
    drop((a, b));
    // A hostile offer: the answering end's description fails, and only
    // that attempt ends.
    let mut c = open(&info(1, 0, false));
    c.signal(Signal::Offer("v=0\r\no=hostile\r\nnot an sdp".into()));
    let start = js_sys::Date::now();
    while c.state() == PairState::Connecting && js_sys::Date::now() - start < 3_000.0 {
        sleep(10).await;
    }
    words.push(match c.state() {
        PairState::Closed(_) => "hostile=closed".into(),
        s => format!("hostile={s:?}"),
    });
    words.join(" ")
}

/// Waits `ms` on the page's timer.
#[cfg(feature = "start")]
async fn sleep(ms: i32) {
    let p = js_sys::Promise::new(&mut |resolve, _| {
        if let Some(w) = web_sys::window() {
            let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        }
    });
    let _ = JsFuture::from(p).await;
}
