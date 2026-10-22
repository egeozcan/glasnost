//! The soak (M9 N6): a local 2v2 of four `net-client` processes and the
//! relay on loopback, each client behind its own bad network
//! ([`crate::netem`]), played by our AI to a winner or to a game-time
//! limit, with links cut and one client process killed and started again
//! mid-game; then every client's checkpoint hashes are compared.
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
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Program::Native(_) => "native",
            Program::Wasm { .. } => "wasm",
        }
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
    /// The relay's address; it is started by the caller.
    pub relay: SocketAddr,
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
    /// The transport.
    pub transport: Transport,
}

/// One client's run.
#[derive(Clone, Debug, Default)]
pub struct ClientRun {
    /// `native` or `wasm`.
    pub program: &'static str,
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
        let _ = writeln!(
            s,
            "soak: {} persons, {} install, limit {} min / {} ticks, map index {}, transport {}",
            PERSONS,
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
            }
        );
        for (i, n) in soak.netem.iter().enumerate() {
            let _ = writeln!(s, "row {i} network: {}", n.describe());
        }
        for t in &self.trouble {
            let _ = writeln!(s, "trouble: {t}");
        }
        let _ = writeln!(s, "wall_ms {}", self.wall_ms);
        let _ = writeln!(
            s,
            "checkpoints common {} last common tick {} mismatches {}",
            self.common,
            self.last_common,
            self.mismatches.len()
        );
        for k in ["winners", "over", "ticks", "game_ms"] {
            let v: Vec<&str> = self
                .clients
                .iter()
                .map(|c| c.report.get(k).map_or("-", String::as_str))
                .collect();
            let _ = writeln!(s, "{k} by row: {}", v.join(" "));
        }
        if let Some(first) = self.mismatches.first() {
            let _ = writeln!(s, "first mismatch tick {first}");
        }
        // The agreed delay: the lowest and highest it was, and the tick
        // share at each value (from the delay changes `TICK:DELAY`).
        for (i, c) in self.clients.iter().enumerate() {
            if let Some(d) = c.report.get("delay")
                && let Some(t) = c.report.get("ticks").and_then(|t| t.parse::<u32>().ok())
            {
                let _ = writeln!(s, "row {i} delay: {}", delay_line(d, t));
            }
        }
        for (i, c) in self.clients.iter().enumerate() {
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
        for f in &self.failures {
            let _ = writeln!(s, "failure: {f}");
        }
        let _ = writeln!(s, "result {}", if self.passed() { "pass" } else { "FAIL" });
        s
    }
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

/// Runs the soak.
pub fn run(soak: &Soak) -> Result<SoakRun, String> {
    std::fs::create_dir_all(&soak.out).map_err(|e| format!("cannot make the output: {e}"))?;
    let dirs: Vec<PathBuf> = (0..PERSONS)
        .map(|i| soak.out.join(format!("row{i}")))
        .collect();
    for d in &dirs {
        if d.exists() {
            std::fs::remove_dir_all(d).map_err(|e| format!("cannot clear {}: {e}", d.display()))?;
        }
        std::fs::create_dir_all(d).map_err(|e| format!("cannot make {}: {e}", d.display()))?;
    }
    let proxies: Vec<Proxy> = soak
        .netem
        .iter()
        .map(|n| Proxy::start(soak.relay, *n))
        .collect::<Result<_, _>>()
        .map_err(|e| format!("a proxy does not start: {e}"))?;
    let (tx, rx) = mpsc::channel();
    let mut seen = Vec::new();
    let mut run = SoakRun::default();
    let mut procs: Vec<Running> = Vec::new();
    // The host, then the guests one at a time.
    let p = spawn(soak, 0, &proxies[0].url(), None, false, &dirs[0], 0, &tx)?;
    procs.push(Running {
        proc: p,
        starts: 1,
        killed: Vec::new(),
    });
    let room = wait_line(&rx, 0, "room ", Duration::from_secs(ROOM_S), &mut seen)?;
    let code = room
        .split_whitespace()
        .nth(1)
        .ok_or("the host said no code")?
        .to_string();
    for row in 1..PERSONS {
        let p = spawn(
            soak,
            row,
            &proxies[row].url(),
            Some(&code),
            false,
            &dirs[row],
            0,
            &tx,
        )?;
        procs.push(Running {
            proc: p,
            starts: 1,
            killed: Vec::new(),
        });
        wait_line(&rx, row, "room ", Duration::from_secs(ROOM_S), &mut seen)?;
    }
    for row in 0..PERSONS {
        wait_line(&rx, row, "game ", Duration::from_secs(ROOM_S), &mut seen)?;
    }
    let start = Instant::now();
    let mut trouble = soak.trouble.clone();
    trouble.sort_by_key(|t| (t.0, t.1));
    let mut next = 0usize;
    let mut alive = [true; PERSONS];
    loop {
        // Trouble due.
        while next < trouble.len() && start.elapsed().as_secs() >= trouble[next].0 {
            let (at, row, what) = trouble[next];
            next += 1;
            if !alive[row] {
                continue;
            }
            match what {
                Trouble::Cut => proxies[row].cut(),
                Trouble::Kill => {
                    let r = &mut procs[row];
                    let _ = r.proc.0.kill();
                    r.killed.push(r.proc.0.wait().ok().and_then(|s| s.code()));
                    r.proc = spawn(
                        soak,
                        row,
                        &proxies[row].url(),
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
        for (row, r) in procs.iter_mut().enumerate() {
            if alive[row]
                && let Ok(Some(_)) = r.proc.0.try_wait()
            {
                alive[row] = false;
            }
        }
        if alive.iter().all(|a| !a) {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    run.wall_ms = start.elapsed().as_millis();
    // The numbers and the hashes.
    for (row, r) in procs.iter_mut().enumerate() {
        let mut c = ClientRun {
            program: soak.programs[row].label(),
            ..ClientRun::default()
        };
        c.exits = r.killed.clone();
        c.exits.push(None);
        for s in 0..r.starts {
            let rep = read_report(&dirs[row].join(format!("report-{s}.txt")));
            if !rep.is_empty() {
                c.report = rep;
            }
        }
        if let Ok(Some(st)) = r.proc.0.try_wait() {
            *c.exits.last_mut().expect("an exit") = st.code();
        }
        let text = std::fs::read_to_string(dirs[row].join("hashes.txt")).unwrap_or_default();
        for l in text.lines() {
            let Some((t, h)) = l.split_once(' ') else {
                continue;
            };
            let Ok(t) = t.parse::<u32>() else { continue };
            if let Some(old) = c.hashes.insert(t, h.to_string())
                && old != h
            {
                c.self_mismatch += 1;
            }
        }
        c.bytes = proxies[row].bytes();
        let k = &proxies[row].counts;
        use std::sync::atomic::Ordering::Relaxed;
        c.proxy = (
            k.connections.load(Relaxed),
            k.stalls.load(Relaxed),
            k.lost.load(Relaxed),
            k.longest_stall_ms.load(Relaxed),
        );
        if c.exits.last().copied().flatten() != Some(0) {
            run.failures
                .push(format!("row {row} exited with {:?}", c.exits.last()));
        }
        if c.report.get("desync").is_some_and(|d| d != "-") {
            run.failures.push(format!(
                "row {row} noticed a desync at tick {}",
                c.report["desync"]
            ));
        }
        if c.self_mismatch > 0 {
            run.failures
                .push(format!("row {row} ran a tick twice with different hashes"));
        }
        let num = |k: &str| c.report.get(k).and_then(|v| v.parse::<i64>().ok());
        let over = c.report.get("over").is_some_and(|w| w != "-");
        let limit = (soak.minutes > 0 && num("game_ms") >= Some(soak.minutes * 60_000))
            || (soak.ticks > 0 && num("ticks") >= Some(i64::from(soak.ticks)));
        if !over && !limit {
            run.failures.push(format!(
                "row {row} saw neither the end of the game nor the limit"
            ));
        }
        run.clients.push(c);
    }
    let mut all: BTreeMap<u32, Vec<&String>> = BTreeMap::new();
    for c in &run.clients {
        for (t, h) in &c.hashes {
            all.entry(*t).or_default().push(h);
        }
    }
    for (t, hs) in &all {
        if hs.iter().any(|h| *h != hs[0]) {
            run.mismatches.push(*t);
        }
        if hs.len() == PERSONS {
            run.common += 1;
            run.last_common = *t;
        }
    }
    drop(rx);
    Ok(run)
}

/// The usage line of `glasnost-headless soak`.
pub const USAGE: &str = "soak --headless BIN [--out DIR] [--minutes M | --ticks N] [--map M] [--seed S] \
[--transport relay|rtc] [--clean] [--latency MS] [--jitter MS] [--loss PERMILLE] [--no-stalls] [--client-arg ARG ...] [--wasm-row R --node NODE --script FILE] [--no-trouble | --trouble S:ROW:cut|kill,...] [--game-dir DIR | --synthetic]";

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

/// Reads the options after `soak`; the relay's address is filled in by the
/// caller once it runs.
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
    let mut wasm_row = None;
    let mut node = PathBuf::from("node");
    let mut script = None;
    let mut trouble = true;
    let mut custom = None;
    let mut transport = Transport::Relay;
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
            "--wasm-row" => wasm_row = Some(num(it.next(), "--wasm-row")? as usize),
            "--node" => node = it.next().map(PathBuf::from).ok_or("--node needs a path")?,
            "--script" => script = it.next().map(PathBuf::from),
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
    let wasm_row = wasm_row.map(|r| r.min(PERSONS - 1));
    if let Some(r) = wasm_row {
        let script = script.ok_or("--wasm-row needs --script")?;
        programs[r] = Program::Wasm {
            node,
            script: std::path::absolute(&script).map_err(|e| e.to_string())?,
        };
    }
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
    // The wasm client keeps no settings file: it is cut, not killed.
    if let Some(r) = wasm_row {
        for x in &mut t {
            if x.1 == r {
                x.2 = Trouble::Cut;
            }
        }
    }
    Ok(Soak {
        out: std::path::absolute(&out).map_err(|e| e.to_string())?,
        relay: "127.0.0.1:0".parse().expect("an address"),
        programs,
        game_dir,
        minutes,
        ticks,
        map,
        netem,
        trouble: t,
        seed,
        client_args,
        transport,
    })
}
