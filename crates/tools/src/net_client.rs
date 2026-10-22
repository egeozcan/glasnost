//! `glasnost-headless net-client` (M9 N6): a windowless game client on
//! the native link that takes a row in a room as a person and plays it
//! with our AI ([`glasnost_app::game::bot`]): the same screens, session and
//! order path a person's client uses, with its desync evidence and reports
//! (N4) and its rejoin after a drop (N5).
//!
//! It hosts a room (`--host N`: N persons) or joins one (`--join CODE`),
//! plays until the game ends or the tick or game-time limit, and prints
//! status lines on standard output (`room CODE row R`, `game slot S house
//! H`, `lost`, `readmitted`, `done`). It writes every checkpoint hash it ran
//! to the hash log as it goes, its numbers (counts and timings only) to the
//! report, and a desync report under `desync-reports/` of the working
//! directory. On the install of `--game-dir` (or `GLASNOST_GAME_DIR`), or on
//! the synthetic install. The options and the loop around the bot are the
//! wasm client's too (`glasnost_app::game::bot::{parse, Driver}`). Our own
//! harness; no spec applies.

use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

use glasnost_app::game::bot::{self, BotStats, Driver, Options, synthetic};
use glasnost_app::game::client::{Client, run_job};
use glasnost_vfs::source::{FileSource, Unopened};
use glasnost_vfs::{ByteSource, MountOptions, PickedFile};

pub use glasnost_app::game::bot::USAGE;

/// How long the client may take to get from the start to its game, in
/// seconds.
// nocite: our own harness
pub const LOBBY_TIMEOUT_S: u64 = 900;

/// How long the client goes on running its game after the bot finished,
/// so that its peers have its frames for their last ticks, in milliseconds.
// nocite: our own harness
pub const LINGER_MS: u64 = 3_000;

/// The window size of the windowless client (laying out its targets).
// nocite: our own harness
pub const SIZE: (i32, i32) = (1280, 800);

/// Reads the options after `net-client` (the install folder defaults to
/// `GLASNOST_GAME_DIR`).
pub fn parse(args: &[String]) -> Result<Options, String> {
    let dir = std::env::var(crate::local_lane::GAME_DIR_VAR).ok();
    bot::parse(args, dir)
}

/// The install's files as the client's mount takes them: archives opened,
/// other files by their size only.
pub fn files(dir: &Path) -> Result<Vec<PickedFile>, String> {
    let ext = MountOptions::default().archive_extensions;
    let mut out = Vec::new();
    let rd = std::fs::read_dir(dir).map_err(|e| format!("cannot read the install folder: {e}"))?;
    for e in rd.flatten() {
        let Ok(meta) = e.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let name = e.file_name().to_string_lossy().into_owned();
        let x = name
            .rsplit_once('.')
            .map(|(_, x)| x.to_ascii_lowercase())
            .unwrap_or_default();
        let source: Rc<dyn ByteSource> = if ext.contains(&x) {
            match FileSource::open(&e.path()) {
                Ok(f) => Rc::new(f),
                Err(_) => continue,
            }
        } else {
            Rc::new(Unopened(meta.len()))
        };
        out.push(PickedFile { path: name, source });
    }
    Ok(out)
}

/// A client with the install of `dir` mounted, or the synthetic one.
pub fn client(dir: Option<&Path>) -> Result<Client, String> {
    let Some(dir) = dir else {
        let mut c = synthetic::client();
        c.resize(SIZE);
        return Ok(c);
    };
    let mut c = Client::new();
    c.resize(SIZE);
    c.mount(files(dir)?);
    run_jobs(&mut c, false);
    if c.install.is_none() {
        return Err(format!("the install does not read: {}", c.status));
    }
    Ok(c)
}

/// Runs the jobs the client handed out: on the install (a mount, a map
/// load) or the synthetic start.
pub fn run_jobs(c: &mut Client, synthetic_install: bool) {
    if synthetic_install {
        synthetic::run_jobs(c);
        return;
    }
    while let Some(job) = c.job.take() {
        let done = pollster::block_on(run_job(job));
        c.done(done);
    }
}

/// The client's log on standard error, with the milliseconds since the
/// start: `GLASNOST_LOG=debug` (or `warn`, `error`, `off`) sets the level,
/// `info` by default.
struct Stderr(Instant);

impl log::Log for Stderr {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::max_level()
    }

    fn log(&self, r: &log::Record) {
        if self.enabled(r.metadata()) {
            eprintln!(
                "{:>9} {:<5} {}",
                self.0.elapsed().as_millis(),
                r.level(),
                r.args()
            );
        }
    }

    fn flush(&self) {}
}

/// Starts the log of [`Stderr`].
pub fn init_log() {
    let level = match std::env::var("GLASNOST_LOG").as_deref() {
        Ok("debug") => log::LevelFilter::Debug,
        Ok("trace") => log::LevelFilter::Trace,
        Ok("warn") => log::LevelFilter::Warn,
        Ok("error") => log::LevelFilter::Error,
        Ok("off") => log::LevelFilter::Off,
        _ => log::LevelFilter::Info,
    };
    let logger: &'static Stderr = Box::leak(Box::new(Stderr(Instant::now())));
    if log::set_logger(logger).is_ok() {
        log::set_max_level(level);
    }
}

/// Plays as the options say; returns the bot's numbers, or why it gave up.
pub fn run(opts: &Options) -> Result<BotStats, String> {
    let dir = opts.game_dir.as_deref().map(Path::new);
    let t = Instant::now();
    let mut c = client(dir)?;
    println!("mounted {} ms", t.elapsed().as_millis());
    if let Some(p) = &opts.settings {
        c.attach_store(Box::new(glasnost_app::settings::FileStore(p.into())));
    }
    let mut log = match &opts.hash_log {
        Some(p) => Some(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(p)
                .map_err(|e| format!("cannot open the hash log: {e}"))?,
        ),
        None => None,
    };
    let mut d = Driver::new(opts.bot.clone());
    let clock = Instant::now();
    let now = || clock.elapsed().as_millis() as i64;
    let lobby_deadline = Instant::now() + Duration::from_secs(LOBBY_TIMEOUT_S);
    let mut wrote_code = false;
    while !d.bot.done() {
        let ms = now();
        c.update(ms);
        run_jobs(&mut c, dir.is_none());
        for line in d.step(&mut c, ms) {
            println!("{line}");
        }
        if !wrote_code
            && d.bot.stats.configured
            && let (Some(p), Some(code)) = (&opts.code_file, &d.bot.stats.code)
        {
            wrote_code = true;
            std::fs::write(p, code).map_err(|e| format!("cannot write the code: {e}"))?;
        }
        if !d.in_game() && Instant::now() > lobby_deadline {
            return Err(format!(
                "no game after {LOBBY_TIMEOUT_S} s: {}",
                c.net_status().0
            ));
        }
        let lines = d.hash_lines();
        if let Some(f) = log.as_mut()
            && !lines.is_empty()
        {
            let _ = f.write_all(lines.as_bytes());
            let _ = f.flush();
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let lines = d.hash_lines();
    if let Some(f) = log.as_mut() {
        let _ = f.write_all(lines.as_bytes());
        let _ = f.flush();
    }
    // Run on a moment, so that the peers have this client's frames for
    // their last ticks.
    let linger = Instant::now();
    while linger.elapsed() < Duration::from_millis(LINGER_MS) && c.networked() {
        c.update(now());
        std::thread::sleep(Duration::from_millis(2));
    }
    if let Some(p) = &opts.report {
        std::fs::write(p, d.report(opts.seed))
            .map_err(|e| format!("cannot write the report: {e}"))?;
    }
    match d.bot.failure() {
        Some(why) => Err(why.to_string()),
        None => {
            println!(
                "done ticks {} desync {:?}",
                d.bot.stats.ticks, d.bot.stats.desync
            );
            Ok(d.bot.stats)
        }
    }
}
