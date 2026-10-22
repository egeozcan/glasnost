//! The soak's wasm client (M9 N6): the game client
//! (`glasnost_app::game::client::Client`) compiled to
//! `wasm32-unknown-unknown`, reaching the relay through the browser's
//! WebSocket link (`glasnost_platform_web::socket`), and played by our AI
//! as a person (`glasnost_app::game::bot`), with the same options as
//! `glasnost-headless net-client`. It has no page: `node/net-client.mjs`
//! runs it under node (whose WebSocket is the browser's API), bound with
//! `wasm-bindgen --target nodejs`. The install's files are read through
//! JS (`globalThis.glasnostReadAt`), a desync report is stored through JS
//! (`globalThis.glasnostSaveReport`), and the clock comes from JS.
//!
//! What this exercises is the wasm32 build of the client: its lockstep,
//! simulation, AI, desync evidence and rejoin, and the web link. It is not
//! a browser: no page, no drawing, no audio, no folder picker. Our own
//! harness; no spec applies.
//!
//! On native targets this crate is empty.

#[cfg(target_arch = "wasm32")]
mod node {
    use std::future::Future;
    use std::io;
    use std::rc::Rc;
    use std::task::{Context, Poll, Waker};

    use glasnost_app::game::bot::{self, Driver, Options, synthetic};
    use glasnost_app::game::client::{Client, run_job};
    use glasnost_vfs::source::Unopened;
    use glasnost_vfs::{ByteSource, MountOptions, PickedFile, ReadFuture};
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        /// Reads `len` bytes at `offset` of the install's file `path`.
        #[wasm_bindgen(js_namespace = globalThis, js_name = glasnostReadAt, catch)]
        fn read_at(path: &str, offset: u64, len: u32) -> Result<Vec<u8>, JsValue>;

        /// Stores a desync report; returns where it went, or "" when it
        /// could not be stored.
        #[wasm_bindgen(js_namespace = globalThis, js_name = glasnostSaveReport)]
        fn save_report(name: &str, bytes: &[u8]) -> String;
    }

    /// An install file read through JS.
    struct JsFile {
        path: String,
        len: u64,
    }

    impl ByteSource for JsFile {
        fn len(&self) -> u64 {
            self.len
        }

        fn read_at(&self, offset: u64, len: usize) -> ReadFuture<'_> {
            let r = if offset.checked_add(len as u64).is_none_or(|e| e > self.len) {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "read past the end of the source",
                ))
            } else {
                read_at(&self.path, offset, len as u32)
                    .map_err(|e| io::Error::other(format!("{e:?}")))
                    .and_then(|b| {
                        if b.len() == len {
                            Ok(b)
                        } else {
                            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "a short read"))
                        }
                    })
            };
            Box::pin(std::future::ready(r))
        }
    }

    fn save(name: &str, bytes: &[u8]) -> Result<String, String> {
        let place = save_report(name, bytes);
        if place.is_empty() {
            Err("the report could not be stored".into())
        } else {
            Ok(place)
        }
    }

    /// Runs a future whose reads are all ready (the install is read
    /// synchronously through JS), without an event loop.
    fn block_on<F: Future>(f: F) -> F::Output {
        let mut f = std::pin::pin!(f);
        let mut cx = Context::from_waker(Waker::noop());
        // nocite: our own harness (every read is ready: a pending poll is a bug)
        for _ in 0..1_000_000 {
            if let Poll::Ready(v) = f.as_mut().poll(&mut cx) {
                return v;
            }
        }
        panic!("a job waited on something other than a read");
    }

    fn run_jobs(c: &mut Client, synthetic_install: bool) {
        if synthetic_install {
            synthetic::run_jobs(c);
            return;
        }
        while let Some(job) = c.job.take() {
            let done = block_on(run_job(job));
            c.done(done);
        }
    }

    /// A net-client: a client played by our AI.
    #[wasm_bindgen]
    pub struct NetClient {
        c: Client,
        d: Driver,
        opts: Options,
    }

    #[wasm_bindgen]
    impl NetClient {
        /// A client of the options `args` (those of `glasnost-headless
        /// net-client`). With `--game-dir`, `names` and `lens` list the
        /// install folder's files and their sizes; their archives are read
        /// through `globalThis.glasnostReadAt(name, offset, len)`.
        #[wasm_bindgen(constructor)]
        pub fn new(
            args: Vec<String>,
            names: Vec<String>,
            lens: Vec<u64>,
        ) -> Result<NetClient, JsError> {
            console_error_panic_hook::set_once();
            let _ = console_log::init_with_level(log::Level::Info);
            let opts = bot::parse(&args, None).map_err(|e| JsError::new(&e))?;
            let mut c = if opts.game_dir.is_some() {
                let ext = MountOptions::default().archive_extensions;
                let files = names
                    .into_iter()
                    .zip(lens)
                    .map(|(path, len)| {
                        let x = path
                            .rsplit_once('.')
                            .map(|(_, x)| x.to_ascii_lowercase())
                            .unwrap_or_default();
                        let source: Rc<dyn ByteSource> = if ext.contains(&x) {
                            Rc::new(JsFile {
                                path: path.clone(),
                                len,
                            })
                        } else {
                            Rc::new(Unopened(len))
                        };
                        PickedFile { path, source }
                    })
                    .collect();
                let mut c = Client::new();
                c.resize((1280, 800)); // nocite: our own harness (the native net-client's size)
                c.mount(files);
                run_jobs(&mut c, false);
                if c.install.is_none() {
                    return Err(JsError::new(&format!(
                        "the install does not read: {}",
                        c.status
                    )));
                }
                c
            } else {
                let mut c = synthetic::client();
                c.resize((1280, 800)); // nocite: our own harness (the native net-client's size)
                c
            };
            c.connect = Some(glasnost_platform_web::socket::connect);
            c.save_report = Some(save);
            let d = Driver::new(opts.bot.clone());
            Ok(NetClient { c, d, opts })
        }

        /// One frame at `now_ms` (the caller's clock): the client's update,
        /// its jobs and the bot's step; returns the status lines to print.
        pub fn step(&mut self, now_ms: f64) -> Vec<String> {
            let now = now_ms as i64;
            self.c.update(now);
            run_jobs(&mut self.c, self.opts.game_dir.is_none());
            self.d.step(&mut self.c, now)
        }

        /// The client's update alone (after the bot finished: its peers
        /// still need its frames for their last ticks).
        pub fn linger(&mut self, now_ms: f64) {
            self.c.update(now_ms as i64);
        }

        /// Whether the bot finished.
        pub fn done(&self) -> bool {
            self.d.bot.done()
        }

        /// Whether the client still has a link.
        pub fn networked(&self) -> bool {
            self.c.networked()
        }

        /// Whether the bot reached its game.
        pub fn in_game(&self) -> bool {
            self.d.in_game()
        }

        /// The room code once the room is set up.
        pub fn code(&self) -> Option<String> {
            let s = &self.d.bot.stats;
            s.code.clone().filter(|_| s.configured)
        }

        /// What the client says of its link.
        pub fn status(&self) -> String {
            self.c.net_status().0.to_string()
        }

        /// The checkpoint lines not yet taken.
        pub fn hash_lines(&mut self) -> String {
            self.d.hash_lines()
        }

        /// The report text.
        pub fn report(&self) -> String {
            self.d.report(self.opts.seed)
        }

        /// Why the bot gave up, if it did.
        pub fn failure(&self) -> Option<String> {
            self.d.bot.failure().map(str::to_string)
        }

        /// The final line: the ticks and the desync, if any.
        pub fn summary_line(&self) -> String {
            format!(
                "done ticks {} desync {:?}",
                self.d.bot.stats.ticks, self.d.bot.stats.desync
            )
        }
    }
}
