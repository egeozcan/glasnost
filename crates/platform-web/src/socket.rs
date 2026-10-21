//! The browser's link to a relay (M9 N3): a `web-sys` WebSocket with binary
//! messages as array buffers. Its callbacks run on the page's event loop
//! and only queue; the client takes what arrived once per frame. Messages
//! sent while the socket opens wait for it. Our own netcode.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use glasnost_app::game::link::{Link, LinkState};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{BinaryType, CloseEvent, Event, MessageEvent, WebSocket};

/// The largest message the client accepts (the relay's own limit).
// nocite: our own netcode (the relay's 64 KiB limit)
const MAX_MESSAGE: u32 = 64 * 1024;

/// The WebSocket close code of a normal closure.
// nocite: the WebSocket protocol (RFC 6455, 7.4.1)
const NORMAL_CLOSURE: u16 = 1000;

struct Shared {
    inbox: VecDeque<Vec<u8>>,
    state: LinkState,
}

/// A WebSocket to the relay with its callbacks.
pub struct WebLink {
    ws: Option<WebSocket>,
    shared: Rc<RefCell<Shared>>,
    waiting: Vec<Vec<u8>>,
    _on_open: Option<Closure<dyn FnMut(Event)>>,
    _on_message: Option<Closure<dyn FnMut(MessageEvent)>>,
    _on_close: Option<Closure<dyn FnMut(CloseEvent)>>,
    _on_error: Option<Closure<dyn FnMut(Event)>>,
}

/// Opens a link to `url` (`ws://host:port/` or `wss://…`).
pub fn connect(url: &str) -> Box<dyn Link> {
    let shared = Rc::new(RefCell::new(Shared {
        inbox: VecDeque::new(),
        state: LinkState::Connecting,
    }));
    let mut link = WebLink {
        ws: None,
        shared: Rc::clone(&shared),
        waiting: Vec::new(),
        _on_open: None,
        _on_message: None,
        _on_close: None,
        _on_error: None,
    };
    let ws = match WebSocket::new(url) {
        Ok(ws) => ws,
        Err(e) => {
            shared.borrow_mut().state =
                LinkState::Closed(format!("cannot open a WebSocket to {url}: {e:?}"));
            return Box::new(link);
        }
    };
    ws.set_binary_type(BinaryType::Arraybuffer);
    let s = Rc::clone(&shared);
    let on_open = Closure::<dyn FnMut(Event)>::new(move |_| {
        let mut s = s.borrow_mut();
        if s.state == LinkState::Connecting {
            s.state = LinkState::Open;
        }
    });
    let s = Rc::clone(&shared);
    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        // Text messages are not ours.
        if let Ok(buf) = e.data().dyn_into::<js_sys::ArrayBuffer>()
            && buf.byte_length() <= MAX_MESSAGE
        {
            s.borrow_mut()
                .inbox
                .push_back(js_sys::Uint8Array::new(&buf).to_vec());
        }
    });
    let s = Rc::clone(&shared);
    let address = url.to_owned();
    let on_close = Closure::<dyn FnMut(CloseEvent)>::new(move |e: CloseEvent| {
        let mut s = s.borrow_mut();
        if matches!(s.state, LinkState::Closed(_)) {
            return;
        }
        let reason = e.reason();
        s.state = LinkState::Closed(if !reason.is_empty() {
            reason
        } else if s.state == LinkState::Connecting {
            format!("cannot reach the relay at {address}")
        } else {
            format!("the connection closed (code {})", e.code())
        });
    });
    let s = Rc::clone(&shared);
    let address = url.to_owned();
    let on_error = Closure::<dyn FnMut(Event)>::new(move |_| {
        let mut s = s.borrow_mut();
        if s.state == LinkState::Connecting {
            s.state = LinkState::Closed(format!("cannot reach the relay at {address}"));
        }
    });
    ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    link.ws = Some(ws);
    link._on_open = Some(on_open);
    link._on_message = Some(on_message);
    link._on_close = Some(on_close);
    link._on_error = Some(on_error);
    Box::new(link)
}

impl WebLink {
    /// Sends what waited for the socket to open.
    fn flush(&mut self) {
        let Some(ws) = self.ws.as_ref() else {
            return;
        };
        if ws.ready_state() != WebSocket::OPEN {
            return;
        }
        for b in self.waiting.drain(..) {
            if ws.send_with_u8_array(&b).is_err() {
                break;
            }
        }
    }
}

impl Link for WebLink {
    fn send(&mut self, bytes: Vec<u8>) {
        match self.ws.as_ref().map(WebSocket::ready_state) {
            Some(WebSocket::CONNECTING) => self.waiting.push(bytes),
            Some(WebSocket::OPEN) => {
                self.flush();
                if let Some(ws) = self.ws.as_ref() {
                    let _ = ws.send_with_u8_array(&bytes);
                }
            }
            _ => {}
        }
    }

    fn recv(&mut self) -> Option<Vec<u8>> {
        self.flush();
        self.shared.borrow_mut().inbox.pop_front()
    }

    fn state(&self) -> LinkState {
        self.shared.borrow().state.clone()
    }

    fn close(&mut self) {
        if let Some(ws) = self.ws.as_ref() {
            let _ = ws.close_with_code_and_reason(NORMAL_CLOSURE, "left");
        }
        let mut s = self.shared.borrow_mut();
        if !matches!(s.state, LinkState::Closed(_)) {
            s.state = LinkState::Closed("closed".into());
        }
    }
}

impl Drop for WebLink {
    fn drop(&mut self) {
        if let Some(ws) = self.ws.take() {
            ws.set_onopen(None);
            ws.set_onmessage(None);
            ws.set_onclose(None);
            ws.set_onerror(None);
            let _ = ws.close_with_code_and_reason(NORMAL_CLOSURE, "left");
        }
    }
}
