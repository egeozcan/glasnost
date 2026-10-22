//! The soak (M9 N6): a local 2v2 of four `net-client` processes and the
//! relay on loopback, each client behind its own bad network
//! ([`crate::netem`]), played by our AI to a winner or to a game-time
//! limit, with links cut and one client process killed and started again
//! mid-game; then every client's checkpoint hashes are compared.
//!
//! A split game (M9 N8c): `--relay-url` plays on a relay the soak did not
//! start (on another machine, or in the network lab's "internet"), and
//! `--rows` runs only some of the four rows here; the others run in another
//! soak elsewhere, with the room code passed between the two (`--code`,
//! `--code-in`, `--code-out`). [`compare`] then puts the halves' hash logs
//! together and applies the same pass rule to all four rows. The room seats
//! each joiner in the next free row, so the parts must join in row order:
//! `--code-out` is written only once this soak's rows are seated, and with
//! more than two parts each one waits for the previous one's file.
//!
//! The host makes the room (four persons, rows 0 to 3); the others join one
//! at a time, so that the rows follow the order of the joins. Rows 0 and 2
//! are team 1, rows 1 and 3 team 2. One client can be the wasm build
//! (`glasnost-headless-web` under node), started with the same options.
//! Every client runs in its own folder under the output folder, where its
//! log, hash log, report, settings and any desync report go. Counts and
//! timings only. Our own harness; no spec applies.
//!
//! The transport (M9 N7): `--transport relay` (the default, N6's soak)
//! keeps every client on the relay (`--relay-only`); `--transport rtc`
//! lets the native clients make direct links to each other on loopback,
//! each with the same bad network on its direct path as on its relay path
//! (`--direct-shape`, its own draws), so that a pair's direct path and its
//! relay path cross the same two bad links. The wasm client under node has
//! no RTCPeerConnection: its pairs stay on the relay.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::netem::{Netem, Proxy};

/// The persons of the soak.
pub const PERSONS: usize = 4;

/// The team of each row: rows 0 and 2 against rows 1 and 3.
pub const TEAMS: [u8; PERSONS] = [1, 2, 1, 2];

/// How long the relay may take to build and start, in seconds.
// nocite: our own harness
const RELAY_START_S: u64 = 900;

/// How long a client may take to reach its room, in seconds (a mount of
/// the install, or a wasm start, comes first).
// nocite: our own harness
const ROOM_S: u64 = 600;

/// How a client process is started.
#[derive(Clone, Debug)]
pub enum Program {
    /// `glasnost-headless net-client` at this path.
    Native(PathBuf),
    /// The wasm client: `node SCRIPT` with the built package beside it.
    Wasm {
        /// The node binary.
        node: PathBuf,
        /// The driver script.
        script: PathBuf,
    },
    /// The wasm client in a browser page (M9 N8c): `node SCRIPT --browser
    /// NAME`, a Playwright driver (`web-tests/net-client.mjs`).
    Browser {
        /// The node binary.
        node: PathBuf,
        /// The driver script.
        script: PathBuf,
        /// `chromium` or `firefox`.
        browser: String,
    },
}

impl Program {
    fn command(&self) -> Command {
        match self {
            Program::Native(bin) => {
                let mut c = Command::new(bin);
                c.arg("net-client");
                c
            }
            Program::Wasm { node, script } => {
                let mut c = Command::new(node);
                c.arg(script);
                c
            }
            Program::Browser {
                node,
                script,
                browser,
            } => {
                let mut c = Command::new(node);
                c.arg(script).args(["--browser", browser]);
                c
            }
        }
    }

    /// `native`, `wasm` or the browser's name.
    pub fn label(&self) -> String {
        match self {
            Program::Native(_) => "native".into(),
            Program::Wasm { .. } => "wasm".into(),
            Program::Browser { browser, .. } => browser.clone(),
        }
    }

    /// Whether the client keeps a settings file, so that a killed process
    /// can be started again and rejoin (the wasm clients keep none).
    fn restarts(&self) -> bool {
        matches!(self, Program::Native(_))
    }
}

/// How the clients reach each other (M9 N7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transport {
    /// Everything through the relay.
    Relay,
    /// Direct links beside the relay.
    Rtc,
}

/// What happens to a client mid-game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trouble {
    /// Its link is cut (the proxy closes it); it reconnects.
    Cut,
    /// Its process is killed and started again on its settings file; the
    /// new process rejoins.
    Kill,
}

/// The soak's settings.
#[derive(Clone, Debug)]
pub struct Soak {
    /// The output folder.
    pub out: PathBuf,
    /// The relay's address; it is started by the caller (or it is
    /// [`Soak::relay_url`]'s, for the proxies).
    pub relay: SocketAddr,
    /// A relay the soak does not start (M9 N8c), as the clients name it.
    /// The clients reach it through their proxies when [`Soak::proxy`] is
    /// on (then it must be `ws://IP:PORT/`), and straight otherwise.
    pub relay_url: Option<String>,
    /// Whether the clients go through their bad-network proxies (always,
    /// unless `--no-proxy` with `--relay-url`). Without them a link cannot
    /// be cut.
    pub proxy: bool,
    /// The rows this soak runs (all four by default; a split game runs the
    /// others elsewhere).
    pub rows: Vec<usize>,
    /// The room code, when row 0 (the host) runs elsewhere.
    pub code: Option<String>,
    /// A file to wait for the room code in, when row 0 runs elsewhere.
    pub code_in: Option<PathBuf>,
    /// A file to write the room code to once this soak's rows are seated,
    /// for the next soak of a split game.
    pub code_out: Option<PathBuf>,
    /// The program of each row's client.
    pub programs: [Program; PERSONS],
    /// The install, or none for the synthetic one.
    pub game_dir: Option<PathBuf>,
    /// The game-time limit, in minutes (0: the tick limit or a winner).
    pub minutes: i64,
    /// The tick limit (0: the game-time limit or a winner).
    pub ticks: u32,
    /// The map, by its place in the install's list.
    pub map: usize,
    /// The bad network of each row's client.
    pub netem: [Netem; PERSONS],
    /// The trouble: (wall seconds after the game started, row, what).
    pub trouble: Vec<(u64, usize, Trouble)>,
    /// The seed of the clients' jitter.
    pub seed: u64,
    /// More options for every client (`--client-arg`).
    pub client_args: Vec<String>,
    /// More options for one row's client (`--row-arg R:ARG`).
    pub row_args: [Vec<String>; PERSONS],
    /// The transport.
    pub transport: Transport,
}

/// One client's run.
#[derive(Clone, Debug, Default)]
pub struct ClientRun {
    /// Its row.
    pub row: usize,
    /// `native`, `wasm` or a browser's name.
    pub program: String,
    /// The exit codes of its processes.
    pub exits: Vec<Option<i32>>,
    /// Its report's `key value` lines.
    pub report: BTreeMap<String, String>,
    /// Its checkpoints: tick to hash text.
    pub hashes: BTreeMap<u32, String>,
    /// Checkpoints it logged twice with different hashes.
    pub self_mismatch: u32,
    /// Bytes up and down through its proxy.
    pub bytes: (u64, u64),
    /// Connections through its proxy, stalls and lost chunks.
    pub proxy: (u64, u64, u64, u64),
}

/// The soak's result.
#[derive(Clone, Debug, Default)]
pub struct SoakRun {
    /// Each row's client.
    pub clients: Vec<ClientRun>,
    /// Wall time from the game's start to the end, in milliseconds.
    pub wall_ms: u128,
    /// Ticks every client logged a checkpoint for.
    pub common: usize,
    /// Ticks two clients disagree on.
    pub mismatches: Vec<u32>,
    /// The highest tick every client logged.
    pub last_common: u32,
    /// What went wrong, if anything.
    pub failures: Vec<String>,
    /// The trouble made, as `row what at second` lines.
    pub trouble: Vec<String>,
}

impl SoakRun {
    /// Whether the soak passed: every client finished, the game reached its
    /// end or the limit on every client, none desynced and every
    /// checkpoint agrees.
    pub fn passed(&self) -> bool {
        self.failures.is_empty() && self.mismatches.is_empty() && self.common > 0
    }

    /// The report text (counts and timings only).
    pub fn text(&self, soak: &Soak) -> String {
        let mut s = String::new();
        let rows: Vec<String> = soak.rows.iter().map(usize::to_string).collect();
        let _ = writeln!(
            s,
            "soak: {} persons, rows {} here, {} install, limit {} min / {} ticks, map index {}, transport {}, relay {}",
            PERSONS,
            rows.join(","),
            if soak.game_dir.is_some() {
                "real"
            } else {
                "synthetic"
            },
            soak.minutes,
            soak.ticks,
            soak.map,
            match soak.transport {
                Transport::Relay => "relay",
                Transport::Rtc => "rtc",
            },
            if soak.relay_url.is_some() {
                "external"
            } else {
                "local"
            }
        );
        if soak.proxy {
            for &i in &soak.rows {
                let _ = writeln!(s, "row {i} network: {}", soak.netem[i].describe());
            }
        } else {
            let _ = writeln!(s, "network: no proxy (the real path to the relay)");
        }
        for t in &self.trouble {
            let _ = writeln!(s, "trouble: {t}");
        }
        let _ = writeln!(s, "wall_ms {}", self.wall_ms);
        s.push_str(&clients_text(
            &self.clients,
            self.common,
            self.last_common,
            &self.mismatches,
        ));
        for f in &self.failures {
            let _ = writeln!(s, "failure: {f}");
        }
        let _ = writeln!(s, "result {}", if self.passed() { "pass" } else { "FAIL" });
        s
    }
}

/// The clients' part of a report: the checkpoints, the game's end, each
/// row's delay, proxy counts, report and pairs.
fn clients_text(clients: &[ClientRun], common: usize, last: u32, mismatches: &[u32]) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "checkpoints common {common} last common tick {last} mismatches {}",
        mismatches.len()
    );
    for k in ["winners", "over", "ticks", "game_ms"] {
        let v: Vec<&str> = clients
            .iter()
            .map(|c| c.report.get(k).map_or("-", String::as_str))
            .collect();
        let _ = writeln!(s, "{k} by row: {}", v.join(" "));
    }
    if let Some(first) = mismatches.first() {
        let _ = writeln!(s, "first mismatch tick {first}");
    }
    // The agreed delay: the lowest and highest it was, and the tick share
    // at each value (from the delay changes `TICK:DELAY`).
    for c in clients {
        if let Some(d) = c.report.get("delay")
            && let Some(t) = c.report.get("ticks").and_then(|t| t.parse::<u32>().ok())
        {
            let _ = writeln!(s, "row {} delay: {}", c.row, delay_line(d, t));
        }
    }
    for c in clients {
        let i = c.row;
        let _ = writeln!(
            s,
            "row {i} ({}): exits {:?}, up {} B, down {} B, connections {}, stalls {}, lost chunks {}, longest stall {} ms, self-mismatch {}",
            c.program,
            c.exits,
            c.bytes.0,
            c.bytes.1,
            c.proxy.0,
            c.proxy.1,
            c.proxy.2,
            c.proxy.3,
            c.self_mismatch
        );
        let line: Vec<String> = c
            .report
            .iter()
            .filter(|(k, _)| !k.starts_with("pair") && *k != "delay")
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        let _ = writeln!(s, "row {i} report: {}", line.join(" "));
        for (k, v) in c.report.iter().filter(|(k, _)| k.starts_with("pair")) {
            let _ = writeln!(s, "row {i} {k}: {v}");
        }
    }
    s
}

/// Whether the clients agree: the ticks every one of them logged, the last
/// of those, and the ticks two of them logged with different hashes.
pub fn agreement(clients: &[ClientRun]) -> (usize, u32, Vec<u32>) {
    let mut all: BTreeMap<u32, Vec<&String>> = BTreeMap::new();
    for c in clients {
        for (t, h) in &c.hashes {
            all.entry(*t).or_default().push(h);
        }
    }
    let (mut common, mut last, mut mismatches) = (0, 0, Vec::new());
    for (t, hs) in &all {
        if hs.iter().any(|h| *h != hs[0]) {
            mismatches.push(*t);
        }
        if hs.len() == clients.len() {
            common += 1;
            last = *t;
        }
    }
    (common, last, mismatches)
}

/// A row's hash log (`TICK HASH` lines): the checkpoints, and how many it
/// logged twice with different hashes.
fn read_hashes(p: &Path) -> (BTreeMap<u32, String>, u32) {
    let text = std::fs::read_to_string(p).unwrap_or_default();
    let mut hashes = BTreeMap::new();
    let mut twice = 0;
    for l in text.lines() {
        let Some((t, h)) = l.split_once(' ') else {
            continue;
        };
        let Ok(t) = t.parse::<u32>() else { continue };
        if let Some(old) = hashes.insert(t, h.to_string())
            && old != h
        {
            twice += 1;
        }
    }
    (hashes, twice)
}

/// The last report a row's processes wrote (`report-K.txt`, the highest K
/// that holds one).
fn last_report(dir: &Path) -> BTreeMap<String, String> {
    let mut k: Vec<(u32, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let n = name
                .strip_prefix("report-")?
                .strip_suffix(".txt")?
                .parse()
                .ok()?;
            Some((n, e.path()))
        })
        .collect();
    k.sort();
    k.iter()
        .rev()
        .map(|(_, p)| read_report(p))
        .find(|r| !r.is_empty())
        .unwrap_or_default()
}

/// The failures a client's own records show: a desync noticed, a tick run
/// twice with different hashes, neither the game's end nor the limit.
fn record_failures(c: &ClientRun, minutes: i64, ticks: u32, out: &mut Vec<String>) {
    let row = c.row;
    if c.report.get("desync").is_some_and(|d| d != "-") {
        out.push(format!(
            "row {row} noticed a desync at tick {}",
            c.report["desync"]
        ));
    }
    if c.self_mismatch > 0 {
        out.push(format!("row {row} ran a tick twice with different hashes"));
    }
    let num = |k: &str| c.report.get(k).and_then(|v| v.parse::<i64>().ok());
    let over = c.report.get("over").is_some_and(|w| w != "-");
    let limit = (minutes > 0 && num("game_ms") >= Some(minutes * 60_000))
        || (ticks > 0 && num("ticks") >= Some(i64::from(ticks)));
    if !over && !limit {
        out.push(format!(
            "row {row} saw neither the end of the game nor the limit"
        ));
    }
}

/// The halves of a split game put together (M9 N8c): every row's hash log
/// and last report from the soak folders given, compared as one game.
#[derive(Clone, Debug, Default)]
pub struct Compare {
    /// Each row found, in row order.
    pub clients: Vec<ClientRun>,
    /// Ticks every row logged.
    pub common: usize,
    /// The last of those.
    pub last_common: u32,
    /// Ticks two rows disagree on.
    pub mismatches: Vec<u32>,
    /// Each folder's own result line (`result pass` or `result FAIL`).
    pub halves: Vec<(PathBuf, String)>,
    /// What went wrong.
    pub failures: Vec<String>,
}

impl Compare {
    /// Whether the whole game passed: all four rows found, each half
    /// passed, none desynced and every checkpoint agrees.
    pub fn passed(&self) -> bool {
        self.failures.is_empty() && self.mismatches.is_empty() && self.common > 0
    }

    /// The report text (counts and timings only).
    pub fn text(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "compare: {} rows", self.clients.len());
        for (d, r) in &self.halves {
            let name = d.file_name().map_or_else(
                || d.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            );
            let _ = writeln!(s, "half {name}: {r}");
        }
        s.push_str(&clients_text(
            &self.clients,
            self.common,
            self.last_common,
            &self.mismatches,
        ));
        for f in &self.failures {
            let _ = writeln!(s, "failure: {f}");
        }
        let _ = writeln!(s, "result {}", if self.passed() { "pass" } else { "FAIL" });
        s
    }
}

/// Puts the halves of a split game together: the soak folders `dirs`
/// (each with its `rowN/` folders and its `soak.txt`), with the game-time
/// and tick limits the halves played to.
pub fn compare(dirs: &[PathBuf], minutes: i64, ticks: u32) -> Compare {
    let mut out = Compare::default();
    let mut found: [Option<usize>; PERSONS] = [None; PERSONS];
    for (k, d) in dirs.iter().enumerate() {
        let result = std::fs::read_to_string(d.join("soak.txt"))
            .unwrap_or_default()
            .lines()
            .rev()
            .find(|l| l.starts_with("result "))
            .unwrap_or("result missing")
            .to_string();
        if result != "result pass" {
            out.failures.push(format!("{}: {result}", d.display()));
        }
        out.halves.push((d.clone(), result));
        for (row, seen) in found.iter_mut().enumerate() {
            let rd = d.join(format!("row{row}"));
            if !rd.join("hashes.txt").exists() {
                continue;
            }
            if let Some(other) = *seen {
                out.failures.push(format!(
                    "row {row} is in {} and {}",
                    dirs[other].display(),
                    d.display()
                ));
                continue;
            }
            *seen = Some(k);
            let (hashes, self_mismatch) = read_hashes(&rd.join("hashes.txt"));
            let c = ClientRun {
                row,
                program: "-".into(),
                report: last_report(&rd),
                hashes,
                self_mismatch,
                ..ClientRun::default()
            };
            record_failures(&c, minutes, ticks, &mut out.failures);
            out.clients.push(c);
        }
    }
    for (row, seen) in found.iter().enumerate() {
        if seen.is_none() {
            out.failures.push(format!("row {row} is in no folder"));
        }
    }
    out.clients.sort_by_key(|c| c.row);
    (out.common, out.last_common, out.mismatches) = agreement(&out.clients);
    out
}

/// A delay summary from a report's `TICK:DELAY,...` changes and its last
/// tick: `lo-hi`, then each value's share of the ticks in percent.
pub fn delay_line(changes: &str, ticks: u32) -> String {
    let steps: Vec<(u32, u32)> = changes
        .split(',')
        .filter_map(|p| p.split_once(':'))
        .filter_map(|(t, d)| Some((t.parse().ok()?, d.parse().ok()?)))
        .collect();
    if steps.is_empty() {
        return "-".into();
    }
    let mut share: BTreeMap<u32, u64> = BTreeMap::new();
    for (i, &(t, d)) in steps.iter().enumerate() {
        let end = steps.get(i + 1).map_or(ticks.max(t), |n| n.0);
        *share.entry(d).or_default() += u64::from(end.saturating_sub(t));
    }
    let total: u64 = share.values().sum::<u64>().max(1);
    let lo = share.keys().next().copied().unwrap_or(0);
    let hi = share.keys().next_back().copied().unwrap_or(0);
    let parts: Vec<String> = share
        .iter()
        .filter(|(_, n)| **n > 0)
        .map(|(d, n)| format!("{d}:{}%", n * 100 / total)) // nocite: our own harness (percent)
        .collect();
    format!("{lo}-{hi} ({})", parts.join(" "))
}

/// A child process, killed when dropped.
struct Proc(Child);

impl Drop for Proc {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// The relay binary, started with `cargo run --release -p glasnost-relay`
/// from the workspace at `root`, killed when dropped.
pub struct RelayProcess {
    _proc: Proc,
    /// Its address.
    pub addr: SocketAddr,
}

impl RelayProcess {
    /// Starts the relay on a free loopback port, logging to `log`, and
    /// waits until it accepts connections.
    pub fn start(root: &Path, log: &Path) -> Result<Self, String> {
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .and_then(|l| l.local_addr())
            .map_err(|e| format!("no free port: {e}"))?
            .port();
        let addr: SocketAddr = format!("127.0.0.1:{port}").parse().expect("an address");
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let out = std::fs::File::create(log).map_err(|e| format!("cannot log the relay: {e}"))?;
        let err = out.try_clone().map_err(|e| e.to_string())?;
        let child = Command::new(cargo)
            .args(["run", "--release", "-q", "-p", "glasnost-relay", "--"])
            .args(["--listen", &addr.to_string()])
            .current_dir(root)
            .stdin(Stdio::null())
            .stdout(out)
            .stderr(err)
            .spawn()
            .map_err(|e| format!("cargo does not run the relay: {e}"))?;
        let mut p = Proc(child);
        let deadline = Instant::now() + Duration::from_secs(RELAY_START_S);
        while TcpStream::connect(addr).is_err() {
            if let Ok(Some(st)) = p.0.try_wait() {
                return Err(format!("the relay exited: {st}"));
            }
            if Instant::now() > deadline {
                return Err("the relay did not start".into());
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Ok(Self { _proc: p, addr })
    }
}

struct Running {
    proc: Proc,
    starts: u32,
    /// The exit codes of its killed processes.
    killed: Vec<Option<i32>>,
}

/// Starts row `row`'s client process, its lines sent to `tx` and its log.
#[allow(clippy::too_many_arguments)]
fn spawn(
    soak: &Soak,
    row: usize,
    url: &str,
    code: Option<&str>,
    resume: bool,
    dir: &Path,
    starts: u32,
    tx: &mpsc::Sender<(usize, String)>,
) -> Result<Proc, String> {
    let mut cmd = soak.programs[row].command();
    cmd.args(["--relay", url]);
    cmd.args(["--name", &format!("Bot{row}")]);
    match code {
        None => {
            cmd.args(["--host", &PERSONS.to_string()]);
            cmd.args(["--map", &soak.map.to_string()]);
            cmd.args(["--code-file", "code.txt"]);
        }
        Some(c) => {
            cmd.args(["--join", c]);
        }
    }
    cmd.args(["--slot", &row.to_string()]);
    cmd.args(["--team", &TEAMS[row].to_string()]);
    cmd.args(["--seed", &(soak.seed + row as u64 * 101).to_string()]);
    cmd.args(["--hash-log", "hashes.txt"]);
    cmd.args(["--report", &format!("report-{starts}.txt")]);
    cmd.args(["--settings", "settings.txt"]);
    if soak.minutes > 0 {
        cmd.args(["--minutes", &soak.minutes.to_string()]);
    }
    if soak.ticks > 0 {
        cmd.args(["--ticks", &soak.ticks.to_string()]);
    }
    match soak.transport {
        Transport::Relay => {
            cmd.arg("--relay-only");
        }
        Transport::Rtc => {
            let n = &soak.netem[row];
            cmd.args([
                "--direct-shape",
                &format!(
                    "{},{},{},{},{},{},{}",
                    n.latency_ms,
                    n.jitter_ms,
                    n.loss_permille,
                    n.stall_every_ms.0,
                    n.stall_every_ms.1,
                    n.stall_ms.0,
                    n.stall_ms.1
                ),
            ]);
        }
    }
    cmd.args(&soak.client_args);
    cmd.args(&soak.row_args[row]);
    if resume {
        cmd.arg("--resume");
    }
    match &soak.game_dir {
        Some(d) => {
            cmd.arg("--game-dir").arg(d);
        }
        None => {
            cmd.arg("--synthetic");
        }
    }
    let log = std::fs::File::create(dir.join(format!("log-{starts}.txt")))
        .map_err(|e| format!("cannot log row {row}: {e}"))?;
    let mut child = cmd
        .current_dir(dir)
        .env_remove(crate::local_lane::GAME_DIR_VAR)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(log)
        .spawn()
        .map_err(|e| format!("row {row} does not start: {e}"))?;
    let out = child.stdout.take().expect("a piped stdout");
    let tx = tx.clone();
    let lines = dir.join(format!("out-{starts}.txt"));
    std::thread::spawn(move || {
        let mut f = std::fs::File::create(lines).ok();
        for line in BufReader::new(out).lines().map_while(Result::ok) {
            if let Some(f) = f.as_mut() {
                use std::io::Write as _;
                let _ = writeln!(f, "{line}");
            }
            if tx.send((row, line)).is_err() {
                break;
            }
        }
        let _ = tx.send((row, CLOSED.into()));
    });
    Ok(Proc(child))
}

/// The line the soak passes on when a client's output closed (it exited).
const CLOSED: &str = "(output closed)";

/// Waits for a line of `row` that starts with `prefix`; returns it.
fn wait_line(
    rx: &mpsc::Receiver<(usize, String)>,
    row: usize,
    prefix: &str,
    within: Duration,
    seen: &mut Vec<(usize, String)>,
) -> Result<String, String> {
    let deadline = Instant::now() + within;
    loop {
        if let Some(i) = seen
            .iter()
            .position(|(r, l)| *r == row && l.starts_with(prefix))
        {
            return Ok(seen.remove(i).1);
        }
        if seen.iter().any(|(r, l)| *r == row && l == CLOSED) {
            return Err(format!(
                "row {row} exited before it said `{prefix}` (see its log)"
            ));
        }
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(format!("row {row} did not say `{prefix}`"));
        }
        match rx.recv_timeout(left) {
            Ok(l) => seen.push(l),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err("the clients' output closed".into());
            }
        }
    }
}

/// Reads a `key value` report.
fn read_report(p: &Path) -> BTreeMap<String, String> {
    std::fs::read_to_string(p)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.split_once(' '))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// Waits for a room code in `p` (a file another soak writes), up to
/// `within`.
fn wait_code(p: &Path, within: Duration) -> Result<String, String> {
    let deadline = Instant::now() + within;
    loop {
        if let Ok(t) = std::fs::read_to_string(p) {
            let t = t.trim();
            if !t.is_empty() {
                return Ok(t.to_string());
            }
        }
        if Instant::now() > deadline {
            return Err(format!("no room code in {} in time", p.display()));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Runs the soak.
pub fn run(soak: &Soak) -> Result<SoakRun, String> {
    std::fs::create_dir_all(&soak.out).map_err(|e| format!("cannot make the output: {e}"))?;
    let dirs: Vec<PathBuf> = (0..PERSONS)
        .map(|i| soak.out.join(format!("row{i}")))
        .collect();
    for &row in &soak.rows {
        let d = &dirs[row];
        if d.exists() {
            std::fs::remove_dir_all(d).map_err(|e| format!("cannot clear {}: {e}", d.display()))?;
        }
        std::fs::create_dir_all(d).map_err(|e| format!("cannot make {}: {e}", d.display()))?;
    }
    let proxies: Vec<Option<Proxy>> = (0..PERSONS)
        .map(|row| {
            if soak.proxy && soak.rows.contains(&row) {
                Proxy::start(soak.relay, soak.netem[row]).map(Some)
            } else {
                Ok(None)
            }
        })
        .collect::<Result<_, _>>()
        .map_err(|e| format!("a proxy does not start: {e}"))?;
    let urls: Vec<String> = proxies
        .iter()
        .map(|p| match (p, &soak.relay_url) {
            (Some(p), _) => p.url(),
            (None, Some(u)) => u.clone(),
            (None, None) => format!("ws://{}/", soak.relay),
        })
        .collect();
    let (tx, rx) = mpsc::channel();
    let mut seen = Vec::new();
    let mut run = SoakRun::default();
    let mut procs: BTreeMap<usize, Running> = BTreeMap::new();
    // The host (or the code from the soak that hosts), then the guests one
    // at a time: the rows follow the order of the joins.
    let mut code = soak.code.clone();
    for &row in &soak.rows {
        if row == 0 {
            let p = spawn(soak, 0, &urls[0], None, false, &dirs[0], 0, &tx)?;
            procs.insert(
                0,
                Running {
                    proc: p,
                    starts: 1,
                    killed: Vec::new(),
                },
            );
            let room = wait_line(&rx, 0, "room ", Duration::from_secs(ROOM_S), &mut seen)?;
            code = Some(
                room.split_whitespace()
                    .nth(1)
                    .ok_or("the host said no code")?
                    .to_string(),
            );
            continue;
        }
        if code.is_none() {
            let p = soak
                .code_in
                .as_ref()
                .ok_or("no room code: row 0 is not here, and neither --code nor --code-in")?;
            code = Some(wait_code(p, Duration::from_secs(ROOM_S))?);
        }
        let p = spawn(
            soak,
            row,
            &urls[row],
            code.as_deref(),
            false,
            &dirs[row],
            0,
            &tx,
        )?;
        procs.insert(
            row,
            Running {
                proc: p,
                starts: 1,
                killed: Vec::new(),
            },
        );
        wait_line(&rx, row, "room ", Duration::from_secs(ROOM_S), &mut seen)?;
    }
    let code = code.ok_or("no room code")?;
    if let Some(p) = &soak.code_out {
        std::fs::write(p, &code).map_err(|e| format!("cannot write the code: {e}"))?;
    }
    for &row in &soak.rows {
        wait_line(&rx, row, "game ", Duration::from_secs(ROOM_S), &mut seen)?;
    }
    let start = Instant::now();
    let mut trouble = soak.trouble.clone();
    trouble.sort_by_key(|t| (t.0, t.1));
    let mut next = 0usize;
    let mut alive: BTreeMap<usize, bool> = soak.rows.iter().map(|&r| (r, true)).collect();
    loop {
        // Trouble due.
        while next < trouble.len() && start.elapsed().as_secs() >= trouble[next].0 {
            let (at, row, what) = trouble[next];
            next += 1;
            if alive.get(&row) != Some(&true) {
                continue;
            }
            match what {
                Trouble::Cut => {
                    if let Some(p) = &proxies[row] {
                        p.cut();
                    }
                }
                Trouble::Kill => {
                    let r = procs.get_mut(&row).expect("a row of this soak");
                    let _ = r.proc.0.kill();
                    r.killed.push(r.proc.0.wait().ok().and_then(|s| s.code()));
                    r.proc = spawn(
                        soak,
                        row,
                        &urls[row],
                        Some(&code),
                        true,
                        &dirs[row],
                        r.starts,
                        &tx,
                    )?;
                    r.starts += 1;
                }
            }
            run.trouble.push(format!("row {row} {what:?} at {at} s"));
        }
        // Lines and exits.
        while let Ok(l) = rx.try_recv() {
            if l.1.starts_with("lost") || l.1.starts_with("readmitted") || l.1.starts_with("done") {
                println!("row {}: {}", l.0, l.1);
            }
            seen.push(l);
        }
        for (row, r) in procs.iter_mut() {
            if alive[row]
                && let Ok(Some(_)) = r.proc.0.try_wait()
            {
                alive.insert(*row, false);
            }
        }
        if alive.values().all(|a| !a) {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    run.wall_ms = start.elapsed().as_millis();
    // The numbers and the hashes.
    for (&row, r) in procs.iter_mut() {
        let mut c = ClientRun {
            row,
            program: soak.programs[row].label(),
            ..ClientRun::default()
        };
        c.exits = r.killed.clone();
        c.exits.push(None);
        c.report = last_report(&dirs[row]);
        if let Ok(Some(st)) = r.proc.0.try_wait() {
            *c.exits.last_mut().expect("an exit") = st.code();
        }
        (c.hashes, c.self_mismatch) = read_hashes(&dirs[row].join("hashes.txt"));
        if let Some(p) = &proxies[row] {
            c.bytes = p.bytes();
            let k = &p.counts;
            use std::sync::atomic::Ordering::Relaxed;
            c.proxy = (
                k.connections.load(Relaxed),
                k.stalls.load(Relaxed),
                k.lost.load(Relaxed),
                k.longest_stall_ms.load(Relaxed),
            );
        }
        if c.exits.last().copied().flatten() != Some(0) {
            run.failures
                .push(format!("row {row} exited with {:?}", c.exits.last()));
        }
        record_failures(&c, soak.minutes, soak.ticks, &mut run.failures);
        run.clients.push(c);
    }
    (run.common, run.last_common, run.mismatches) = agreement(&run.clients);
    drop(rx);
    Ok(run)
}

/// The usage line of `glasnost-headless soak`.
pub const USAGE: &str = "soak --headless BIN [--out DIR] [--minutes M | --ticks N] [--map M] [--seed S] \
[--transport relay|rtc] [--clean] [--latency MS] [--jitter MS] [--loss PERMILLE] [--no-stalls] [--client-arg ARG ...] [--row-arg ROW:ARG ...] \
[--wasm-row R ... --node NODE --script FILE] [--browser-row R:chromium|firefox ... --browser-script FILE] \
[--relay-url URL [--no-proxy]] [--rows R,R,...] [--code CODE | --code-in FILE] [--code-out FILE] \
[--no-trouble | --trouble S:ROW:cut|kill,...] [--game-dir DIR | --synthetic]\n\
  soak compare [--minutes M | --ticks N] DIR DIR ...";

/// The default trouble of a soak of about `secs` wall seconds: a link cut
/// at a sixth of the way for row 1, at two sixths for row 3, row 2's
/// process killed and started again at half way, and the host's link cut at
/// four sixths. Each comes long after the last was over (a rejoin takes
/// seconds), and well inside the grace period.
pub fn default_trouble(secs: u64) -> Vec<(u64, usize, Trouble)> {
    let s = secs / 6; // nocite: our own harness
    vec![
        (s, 1, Trouble::Cut),
        (2 * s, 3, Trouble::Cut),
        (3 * s, 2, Trouble::Kill),
        (4 * s, 0, Trouble::Cut),
    ]
}

/// Reads a `ROW:VALUE` option's value.
fn row_value<'a>(v: Option<&'a String>, what: &str) -> Result<(usize, &'a str), String> {
    let (r, x) = v
        .and_then(|v| v.split_once(':'))
        .ok_or_else(|| format!("{what} needs ROW:VALUE"))?;
    let r: usize = r
        .parse()
        .ok()
        .filter(|r| *r < PERSONS)
        .ok_or_else(|| format!("{what}: no row {r}"))?;
    Ok((r, x))
}

/// The socket address of a `ws://IP:PORT/` relay URL.
fn ws_addr(url: &str) -> Option<SocketAddr> {
    url.strip_prefix("ws://")?
        .trim_end_matches('/')
        .parse()
        .ok()
}

/// Reads the options after `soak`; the relay's address is filled in by the
/// caller once it runs (unless `--relay-url` names one).
pub fn parse(args: &[String]) -> Result<Soak, String> {
    let mut headless = None;
    let mut out = PathBuf::from("local-lane/m9/n6");
    let mut minutes = 0i64;
    let mut ticks = 0u32;
    let mut map = 0usize;
    let mut seed = 1u64;
    let mut clean = false;
    let (mut latency, mut jitter, mut loss, mut stalls) = (None, None, None, true);
    let mut client_args = Vec::new();
    let mut row_args: [Vec<String>; PERSONS] = Default::default();
    let mut wasm_rows = Vec::new();
    let mut browser_rows = Vec::new();
    let mut node = PathBuf::from("node");
    let mut script = None;
    let mut browser_script = None;
    let mut trouble = true;
    let mut custom = None;
    let mut transport = Transport::Relay;
    let mut relay_url = None;
    let mut proxy = true;
    let mut rows = None;
    let mut code = None;
    let mut code_in = None;
    let mut code_out = None;
    let mut game_dir = std::env::var_os(crate::local_lane::GAME_DIR_VAR).map(PathBuf::from);
    let mut it = args.iter();
    let num = |v: Option<&String>, what: &str| -> Result<u64, String> {
        v.and_then(|s| s.parse().ok())
            .ok_or_else(|| format!("{what} needs a number"))
    };
    while let Some(a) = it.next() {
        match a.as_str() {
            "--headless" => headless = it.next().map(PathBuf::from),
            "--out" => out = it.next().map(PathBuf::from).ok_or("--out needs a folder")?,
            "--minutes" => minutes = num(it.next(), "--minutes")? as i64,
            "--ticks" => ticks = num(it.next(), "--ticks")? as u32,
            "--map" => map = num(it.next(), "--map")? as usize,
            "--seed" => seed = num(it.next(), "--seed")?,
            "--clean" => clean = true,
            "--transport" => {
                transport = match it.next().map(String::as_str) {
                    Some("relay") => Transport::Relay,
                    Some("rtc") => Transport::Rtc,
                    _ => return Err("--transport needs relay or rtc".into()),
                }
            }
            "--latency" => latency = Some(num(it.next(), "--latency")?),
            "--jitter" => jitter = Some(num(it.next(), "--jitter")?),
            "--loss" => loss = Some(num(it.next(), "--loss")? as u32),
            "--no-stalls" => stalls = false,
            "--client-arg" => {
                client_args.push(it.next().cloned().ok_or("--client-arg needs a value")?)
            }
            "--row-arg" => {
                let (r, v) = row_value(it.next(), "--row-arg")?;
                row_args[r].push(v.to_string());
            }
            "--wasm-row" => wasm_rows.push(num(it.next(), "--wasm-row")? as usize),
            "--browser-row" => {
                let (r, b) = row_value(it.next(), "--browser-row")?;
                if !matches!(b, "chromium" | "firefox" | "webkit") {
                    return Err(format!("--browser-row: no browser {b}"));
                }
                browser_rows.push((r, b.to_string()));
            }
            "--node" => node = it.next().map(PathBuf::from).ok_or("--node needs a path")?,
            "--script" => script = it.next().map(PathBuf::from),
            "--browser-script" => browser_script = it.next().map(PathBuf::from),
            "--relay-url" => {
                relay_url = Some(it.next().cloned().ok_or("--relay-url needs a URL")?);
            }
            "--no-proxy" => proxy = false,
            "--rows" => {
                let v = it.next().ok_or("--rows needs ROW,ROW,...")?;
                let mut r: Vec<usize> = v
                    .split(',')
                    .map(|p| p.trim().parse::<usize>().ok().filter(|r| *r < PERSONS))
                    .collect::<Option<_>>()
                    .ok_or_else(|| format!("--rows: bad rows {v}"))?;
                r.sort_unstable();
                r.dedup();
                rows = Some(r);
            }
            "--code" => code = Some(it.next().cloned().ok_or("--code needs a code")?),
            "--code-in" => code_in = it.next().map(PathBuf::from),
            "--code-out" => code_out = it.next().map(PathBuf::from),
            "--no-trouble" => trouble = false,
            "--trouble" => {
                let spec = it
                    .next()
                    .ok_or("--trouble needs SECONDS:ROW:cut|kill,...")?;
                let mut t = Vec::new();
                for part in spec.split(',').filter(|p| !p.is_empty()) {
                    let f: Vec<&str> = part.split(':').collect();
                    let (Some(at), Some(row), Some(what)) = (
                        f.first().and_then(|v| v.parse().ok()),
                        f.get(1).and_then(|v| v.parse::<usize>().ok()),
                        f.get(2),
                    ) else {
                        return Err(format!("bad trouble {part}"));
                    };
                    let what = match *what {
                        "cut" => Trouble::Cut,
                        "kill" => Trouble::Kill,
                        _ => return Err(format!("bad trouble {part}")),
                    };
                    t.push((at, row.min(PERSONS - 1), what));
                }
                custom = Some(t);
            }
            "--game-dir" => game_dir = it.next().map(PathBuf::from),
            "--synthetic" => game_dir = None,
            other => return Err(format!("unknown option {other}")),
        }
    }
    let headless = headless.ok_or("--headless is needed")?;
    let native = Program::Native(std::path::absolute(&headless).map_err(|e| e.to_string())?);
    let mut programs = [native.clone(), native.clone(), native.clone(), native];
    for &r in &wasm_rows {
        let script = script.as_ref().ok_or("--wasm-row needs --script")?;
        programs[r.min(PERSONS - 1)] = Program::Wasm {
            node: node.clone(),
            script: std::path::absolute(script).map_err(|e| e.to_string())?,
        };
    }
    for (r, b) in &browser_rows {
        let script = browser_script
            .as_ref()
            .ok_or("--browser-row needs --browser-script")?;
        programs[*r] = Program::Browser {
            node: node.clone(),
            script: std::path::absolute(script).map_err(|e| e.to_string())?,
            browser: b.clone(),
        };
    }
    let rows = rows.unwrap_or_else(|| (0..PERSONS).collect());
    if rows.is_empty() {
        return Err("--rows names no row".into());
    }
    if !rows.contains(&0) && code.is_none() && code_in.is_none() {
        return Err("without row 0 (the host) the soak needs --code or --code-in".into());
    }
    if rows.contains(&0) && (code.is_some() || code_in.is_some()) {
        return Err("row 0 hosts the room: --code and --code-in are for the other rows".into());
    }
    if relay_url.is_none() {
        proxy = true;
    }
    let relay = match (&relay_url, proxy) {
        (Some(u), true) => ws_addr(u).ok_or_else(|| {
            format!("--relay-url {u}: the proxies need ws://IP:PORT/ (or --no-proxy)")
        })?,
        _ => "127.0.0.1:0".parse().expect("an address"),
    };
    let netem = std::array::from_fn(|i| {
        let s = seed.wrapping_mul(31).wrapping_add(i as u64);
        let mut n = if clean {
            Netem::clean(s)
        } else {
            Netem::bad(s)
        };
        n.latency_ms = latency.unwrap_or(n.latency_ms);
        n.jitter_ms = jitter.unwrap_or(n.jitter_ms);
        n.loss_permille = loss.unwrap_or(n.loss_permille);
        if !stalls {
            n.stall_every_ms = (0, 0);
            n.stall_ms = (0, 0);
        }
        n
    });
    let secs = if minutes > 0 {
        minutes as u64 * 60
    } else {
        // nocite: our own harness (the fastest paced speed's 16 ms ticks)
        u64::from(ticks) * 16 / 1000
    };
    let mut t = match (trouble, custom) {
        (false, _) => Vec::new(),
        (true, Some(t)) => t,
        (true, None) => default_trouble(secs),
    };
    // Only this soak's rows; a client that keeps no settings file (the
    // wasm clients) is cut, not killed.
    t.retain(|x| rows.contains(&x.1));
    for x in &mut t {
        if !programs[x.1].restarts() {
            x.2 = Trouble::Cut;
        }
        if x.2 == Trouble::Cut && !proxy {
            return Err(format!(
                "row {} cannot be cut without a proxy (--no-proxy)",
                x.1
            ));
        }
    }
    Ok(Soak {
        out: std::path::absolute(&out).map_err(|e| e.to_string())?,
        relay,
        relay_url,
        proxy,
        rows,
        code,
        code_in,
        code_out,
        programs,
        game_dir,
        minutes,
        ticks,
        map,
        netem,
        trouble: t,
        seed,
        client_args,
        row_args,
        transport,
    })
}

/// Reads the options after `soak compare`: the limits, then the folders.
pub fn parse_compare(args: &[String]) -> Result<(Vec<PathBuf>, i64, u32), String> {
    let (mut minutes, mut ticks, mut dirs) = (0i64, 0u32, Vec::new());
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--minutes" => {
                minutes = it
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or("--minutes needs a number")?
            }
            "--ticks" => {
                ticks = it
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or("--ticks needs a number")?
            }
            d if d.starts_with("--") => return Err(format!("unknown option {d}")),
            d => dirs.push(PathBuf::from(d)),
        }
    }
    if dirs.is_empty() {
        return Err("compare needs the soak folders".into());
    }
    Ok((dirs, minutes, ticks))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(str::to_string).collect()
    }

    #[test]
    fn a_split_half_parses() {
        let s = parse(&args(
            "--headless bin --synthetic --relay-url ws://10.0.0.1:7400/ --rows 3,2 --code-in c.txt \
             --row-arg 2:--relay-only --row-arg 3:--ice --row-arg 3:stun:10.0.0.1:3478 --trouble 5:2:kill,9:1:cut",
        ))
        .expect("parses");
        assert_eq!(s.rows, vec![2, 3]);
        assert_eq!(s.relay, "10.0.0.1:7400".parse::<SocketAddr>().unwrap());
        assert!(s.proxy);
        assert_eq!(s.code_in, Some(PathBuf::from("c.txt")));
        assert_eq!(s.row_args[2], vec!["--relay-only".to_string()]);
        assert_eq!(s.row_args[3].len(), 2);
        // Row 1 is not here: its trouble is the other half's.
        assert_eq!(s.trouble, vec![(5, 2, Trouble::Kill)]);
    }

    #[test]
    fn split_options_are_checked() {
        let bad = [
            "--headless b --rows 1,2",
            "--headless b --rows 0,1 --code ABC",
            "--headless b --relay-url wss://relay.lab/ --rows 0",
            "--headless b --relay-url ws://1.2.3.4:5/ --no-proxy --trouble 3:0:cut",
            "--headless b --rows 0,9",
            "--headless b --row-arg 7:--x",
            "--headless b --browser-row 1:opera --browser-script s.mjs",
            "--headless b --browser-row 1:firefox",
        ];
        for b in bad {
            assert!(parse(&args(b)).is_err(), "{b}");
        }
        // Straight to an external wss:// relay, no proxy; a kill is fine.
        let s = parse(&args(
            "--headless b --relay-url wss://relay.lab/ --no-proxy --rows 0,1 --code-out c --trouble 3:1:kill",
        ))
        .expect("parses");
        assert!(!s.proxy);
        assert_eq!(s.relay_url.as_deref(), Some("wss://relay.lab/"));
        assert_eq!(s.trouble, vec![(3, 1, Trouble::Kill)]);
    }

    #[test]
    fn browser_rows_are_cut_not_killed() {
        let s = parse(&args(
            "--headless b --browser-row 1:chromium --browser-row 3:firefox --browser-script s.mjs --trouble 4:1:kill,5:2:kill",
        ))
        .expect("parses");
        assert_eq!(s.programs[1].label(), "chromium");
        assert_eq!(s.programs[3].label(), "firefox");
        assert_eq!(s.programs[0].label(), "native");
        assert_eq!(s.trouble, vec![(4, 1, Trouble::Cut), (5, 2, Trouble::Kill)]);
    }

    #[test]
    fn halves_compare_as_one_game() {
        let root = std::env::temp_dir().join(format!("glasnost-compare-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let half = |name: &str, rows: &[usize], result: &str, hash_of_60: &str| {
            let d = root.join(name);
            for &r in rows {
                let rd = d.join(format!("row{r}"));
                std::fs::create_dir_all(&rd).unwrap();
                std::fs::write(
                    rd.join("hashes.txt"),
                    format!("30 aa\n60 {hash_of_60}\n90 cc\n"),
                )
                .unwrap();
                std::fs::write(rd.join("report-0.txt"), "ticks 90\ndesync -\nover -\n").unwrap();
                std::fs::write(rd.join("report-1.txt"), "ticks 120\ndesync -\nover -\n").unwrap();
            }
            std::fs::write(d.join("soak.txt"), format!("soak\nresult {result}\n")).unwrap();
            d
        };
        let a = half("a", &[0, 1], "pass", "bb");
        let b = half("b", &[2, 3], "pass", "bb");
        let c = compare(&[a.clone(), b.clone()], 0, 100);
        assert!(c.passed(), "{}", c.text());
        assert_eq!((c.common, c.last_common), (3, 90));
        assert_eq!(c.clients.len(), 4);
        // The later process's report counts.
        assert_eq!(c.clients[0].report["ticks"], "120");
        // One half's tick 60 differs; another lacks a row.
        let d = half("d", &[2, 3], "pass", "XX");
        let c = compare(&[a.clone(), d], 0, 100);
        assert_eq!(c.mismatches, vec![60]);
        assert!(!c.passed());
        let c = compare(std::slice::from_ref(&a), 0, 100);
        assert!(
            c.failures
                .iter()
                .any(|f| f.contains("row 2 is in no folder"))
        );
        let e = half("e", &[2, 3], "FAIL", "bb");
        assert!(!compare(&[a, e], 0, 100).passed());
        let _ = std::fs::remove_dir_all(&root);
    }
}
