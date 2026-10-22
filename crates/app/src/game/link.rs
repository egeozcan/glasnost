//! The client's connection to the relay: whole binary WebSocket messages,
//! in order, both ways. The lobby's messages (`glasnost_net::lobby::wire`,
//! magic `GL`) and the lockstep's (`glasnost_net::wire`, magic `GN`) share
//! one link; the client tells them apart by the magic.
//!
//! A link never blocks the caller: [`Link::send`] queues, [`Link::recv`]
//! takes what has arrived. Natively ([`native`]) a tungstenite socket runs
//! on its own thread; in the browser `glasnost-platform-web` gives the
//! client a `web-sys` WebSocket through a [`Connect`] function. Neither
//! reaches the simulation: the client moves bytes between a link and its
//! `glasnost_net::Session`, which has no I/O of its own. Our own netcode.

/// Where a link is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkState {
    /// Opening (the TCP connection and the WebSocket handshake).
    Connecting,
    /// Messages flow.
    Open,
    /// Closed, and why (the relay's close reason, or the error).
    Closed(String),
}

/// A message connection to the relay.
pub trait Link {
    /// Queues one message. Messages queued while connecting leave once the
    /// link is open; on a closed link they are lost.
    fn send(&mut self, bytes: Vec<u8>);
    /// The next message that arrived, oldest first.
    fn recv(&mut self) -> Option<Vec<u8>>;
    /// Where the link is.
    fn state(&self) -> LinkState;
    /// Closes the link (the relay then sees the connection go).
    fn close(&mut self);
}

/// Opens a link to a relay address such as `ws://127.0.0.1:8787/`.
pub type Connect = fn(&str) -> Box<dyn Link>;

/// The relay address the multiplayer screen starts with.
// nocite: our own netcode (the relay binary's default listening address)
pub const DEFAULT_RELAY: &str = "ws://127.0.0.1:8787/";

/// The platform's own way to open a link, where it has one built in: the
/// native socket thread. The web shell passes its own.
pub fn platform_connect() -> Option<Connect> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Some(native::connect)
    }
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
}

/// A link over a tungstenite socket on its own thread (native only).
#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use std::io::ErrorKind;
    use std::net::TcpStream;
    use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use tungstenite::client::IntoClientRequest;
    use tungstenite::protocol::CloseFrame;
    use tungstenite::protocol::frame::coding::CloseCode;
    use tungstenite::stream::MaybeTlsStream;
    use tungstenite::{Connector, Error, Message, WebSocket};

    use super::{Link, LinkState};

    /// How long the socket thread waits for a message before it looks at
    /// what the client queued.
    // nocite: our own netcode (short against the 16 ms frame, long enough not to spin)
    const POLL: Duration = Duration::from_millis(2);

    /// The largest message the client accepts (the relay's own limit).
    // nocite: our own netcode (the relay's 64 KiB limit)
    const MAX_MESSAGE: usize = 64 * 1024;

    enum Command {
        Send(Vec<u8>),
        Close,
    }

    /// The client's end of a socket thread.
    pub struct NativeLink {
        to: Sender<Command>,
        from: Receiver<Vec<u8>>,
        state: Arc<Mutex<LinkState>>,
    }

    /// Opens a link to `url` (`ws://host:port/` or `wss://host:port/`) on a
    /// new thread. `wss://` checks the relay's certificate against the
    /// system's trusted roots (M9 N8b).
    pub fn connect(url: &str) -> Box<dyn Link> {
        connect_with(url, None)
    }

    /// [`connect`] with a TLS configuration of its own, for `wss://` to a
    /// relay the system does not trust (tests).
    pub fn connect_with(url: &str, tls: Option<Arc<rustls::ClientConfig>>) -> Box<dyn Link> {
        let (to, commands) = mpsc::channel();
        let (deliver, from) = mpsc::channel();
        let state = Arc::new(Mutex::new(LinkState::Connecting));
        let shared = Arc::clone(&state);
        let url = url.to_owned();
        let spawned = std::thread::Builder::new()
            .name("glasnost-link".into())
            .spawn(move || {
                let end = run(&url, tls, &commands, &deliver, &shared);
                set(&shared, LinkState::Closed(end));
            });
        if let Err(e) = spawned {
            set(
                &state,
                LinkState::Closed(format!("no thread for the link: {e}")),
            );
        }
        Box::new(NativeLink { to, from, state })
    }

    fn set(state: &Mutex<LinkState>, s: LinkState) {
        if let Ok(mut g) = state.lock() {
            *g = s;
        }
    }

    /// The socket thread: connects, then moves messages until either end
    /// closes. Returns why it ended.
    fn run(
        url: &str,
        tls: Option<Arc<rustls::ClientConfig>>,
        commands: &Receiver<Command>,
        deliver: &Sender<Vec<u8>>,
        state: &Mutex<LinkState>,
    ) -> String {
        let mut socket = match open(url, tls) {
            Ok(s) => s,
            Err(e) => return format!("cannot reach the relay at {url}: {e}"),
        };
        // The poll timeout goes on the TCP socket under the TLS, too.
        let tcp = match socket.get_mut() {
            MaybeTlsStream::Plain(s) => Some(s),
            MaybeTlsStream::Rustls(s) => Some(&mut s.sock),
            _ => None,
        };
        if let Some(s) = tcp
            && let Err(e) = TcpStream::set_read_timeout(s, Some(POLL))
        {
            return format!("the socket refused a timeout: {e}");
        }
        set(state, LinkState::Open);
        loop {
            loop {
                match commands.try_recv() {
                    Ok(Command::Send(bytes)) => {
                        if let Err(e) = socket.send(Message::Binary(bytes.into())) {
                            return ended(&e);
                        }
                    }
                    Ok(Command::Close) | Err(TryRecvError::Disconnected) => {
                        return close(&mut socket);
                    }
                    Err(TryRecvError::Empty) => break,
                }
            }
            match socket.read() {
                Ok(Message::Binary(b)) if b.len() <= MAX_MESSAGE => {
                    if deliver.send(b.to_vec()).is_err() {
                        return close(&mut socket);
                    }
                }
                Ok(Message::Close(frame)) => {
                    let _ = socket.flush();
                    return frame.map_or_else(
                        || "the relay closed the connection".to_owned(),
                        |f| f.reason.to_string(),
                    );
                }
                // Pings are answered by tungstenite on the next flush; text
                // and oversized messages are not ours.
                Ok(_) => {
                    let _ = socket.flush();
                }
                Err(Error::Io(e))
                    if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) =>
                {
                    if let Err(e) = socket.flush() {
                        return ended(&e);
                    }
                }
                Err(e) => return ended(&e),
            }
        }
    }

    /// Connects and does the WebSocket handshake (over TLS for `wss://`).
    fn open(
        url: &str,
        tls: Option<Arc<rustls::ClientConfig>>,
    ) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, Error> {
        let Some(config) = tls else {
            return tungstenite::connect(url).map(|(s, _)| s);
        };
        let request = url.into_client_request()?;
        let host = request
            .uri()
            .host()
            .ok_or(Error::Url(tungstenite::error::UrlError::NoHostName))?;
        // nocite: the standard ports of ws and wss
        let port =
            request
                .uri()
                .port_u16()
                .unwrap_or(if request.uri().scheme_str() == Some("wss") {
                    443
                } else {
                    80
                });
        let stream = TcpStream::connect((host, port)).map_err(Error::Io)?;
        let _ = stream.set_nodelay(true);
        tungstenite::client_tls_with_config(request, stream, None, Some(Connector::Rustls(config)))
            .map(|(s, _)| s)
            .map_err(|e| match e {
                tungstenite::HandshakeError::Failure(e) => e,
                tungstenite::HandshakeError::Interrupted(_) => {
                    Error::Io(ErrorKind::Interrupted.into())
                }
            })
    }

    fn ended(e: &Error) -> String {
        match e {
            Error::ConnectionClosed | Error::AlreadyClosed => "the connection closed".into(),
            e => format!("the connection failed: {e}"),
        }
    }

    fn close(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> String {
        let _ = socket.close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: "left".into(),
        }));
        // Let the close frame leave and the relay's answer come, briefly.
        for _ in 0..50 {
            match socket.read() {
                Err(Error::Io(e))
                    if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
                Err(_) => break,
                Ok(_) => {}
            }
        }
        "closed".into()
    }

    impl Link for NativeLink {
        fn send(&mut self, bytes: Vec<u8>) {
            let _ = self.to.send(Command::Send(bytes));
        }

        fn recv(&mut self) -> Option<Vec<u8>> {
            self.from.try_recv().ok()
        }

        fn state(&self) -> LinkState {
            self.state
                .lock()
                .map_or(LinkState::Closed("the link failed".into()), |s| s.clone())
        }

        fn close(&mut self) {
            let _ = self.to.send(Command::Close);
        }
    }

    impl Drop for NativeLink {
        fn drop(&mut self) {
            let _ = self.to.send(Command::Close);
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "link_tests.rs"]
mod tests;
