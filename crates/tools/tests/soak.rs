//! The soak (M9 N6). The short test runs in CI: a relay in the test
//! process, four windowless clients on the synthetic install (invented
//! rules and map), each behind its own bad network (`netem`), played by
//! our AI as persons in a 2v2 for a few hundred ticks; one client's link is
//! cut mid-game and it rejoins. Every checkpoint hash of every client must
//! agree and none may notice a desync. It runs twice: on the relay alone,
//! and with direct links (M9 N7) on an in-process fake network with the
//! same bad network on the direct path, where most frames must come the
//! direct way first.
//!
//! The long soak (`#[ignore]`, local lane) is `glasnost_tools::soak`: four
//! `glasnost-headless net-client` processes and the relay binary, on the
//! install of `GLASNOST_GAME_DIR` (or the synthetic one), for
//! `GLASNOST_SOAK_MINUTES` minutes of game time (default 30) or to a
//! winner, with links cut and a process killed and started again; one
//! client is the wasm build under node when `GLASNOST_SOAK_WASM` names its
//! driver script. It writes to the git-ignored `local-lane/m9/n6/`
//! (`GLASNOST_LANE_OUT` elsewhere), over `GLASNOST_SOAK_TRANSPORT`
//! (`relay`, the default, or `rtc`: real direct links on loopback).
//! Run with `cargo test --release -p glasnost-tools --test soak --
//! --ignored --nocapture`. Our own harness; no spec applies.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use glasnost_app::game::bot::{Bot, BotConfig, Role, synthetic};
use glasnost_app::game::direct::Shape;
use glasnost_app::game::direct::fake::FakeNet;
use glasnost_relay::{Entropy, Limits, Relay};
use glasnost_tools::netem::{Netem, Proxy};
use glasnost_tools::soak;

/// The ticks the short soak plays.
const TICKS: u32 = 450;

/// The tick at which row 1's link is cut.
const CUT_AT: u32 = 150;

struct Fake(u64);

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

fn relay(seed: u64) -> (tokio::runtime::Runtime, Relay) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime");
    let r = rt
        .block_on(glasnost_relay::bind_with(
            "127.0.0.1:0",
            Limits::default(),
            Box::new(Fake(seed)),
        ))
        .expect("the relay binds");
    (rt, r)
}

/// A bad network with stalls often enough for a short test.
fn bad(seed: u64) -> Netem {
    let mut n = Netem::bad(seed);
    n.stall_every_ms = (1_500, 4_000);
    n.stall_ms = (200, 700);
    n
}

/// The same bad network on the direct path.
fn shape(n: &Netem) -> Shape {
    Shape {
        latency_ms: n.latency_ms,
        jitter_ms: n.jitter_ms,
        loss_permille: n.loss_permille,
        stall_every_ms: n.stall_every_ms,
        stall_ms: n.stall_ms,
        seed: n.seed ^ 0xD1EC,
    }
}

/// Four AI persons in a 2v2 behind bad networks play the synthetic game for
/// a few hundred ticks; row 1's link is cut and it rejoins. Every client's
/// checkpoint hashes agree, none notices a desync, and the numbers are kept.
#[test]
fn four_ai_persons_play_a_2v2_over_bad_links_and_one_rejoins() {
    two_v_two(66, None);
}

/// The same with direct links between the four, on a fake network behind
/// the same bad networks: the game agrees, row 1's rejoin links again, and
/// frames come the direct way first.
#[test]
fn four_ai_persons_play_a_2v2_over_direct_links_beside_the_relay() {
    let net = FakeNet::new(30);
    let bots = two_v_two(67, Some(&net));
    for b in &bots {
        assert_eq!(b.stats.pairs.len(), soak::PERSONS - 1, "{}", b.cfg.name);
        let first: u64 = b
            .stats
            .pairs
            .iter()
            .map(|p| p.paths.unwrap_or_default().direct_first)
            .sum();
        assert!(
            first > 100,
            "{}: {first} frames came direct first",
            b.cfg.name
        );
        for p in &b.stats.pairs {
            assert!(p.stats.opened >= 1, "{}: {p:?}", b.cfg.name);
            assert_eq!(p.paths.unwrap_or_default().conflicts, 0);
        }
    }
}

fn two_v_two(seed: u64, direct: Option<&FakeNet>) -> Vec<Bot> {
    let (_rt, relay) = relay(seed);
    let proxies: Vec<Proxy> = (0..soak::PERSONS)
        .map(|i| Proxy::start(relay.addr, bad(40 + i as u64)).expect("a proxy"))
        .collect();
    let clock = Instant::now();
    let now = || clock.elapsed().as_millis() as i64;
    let mut clients = Vec::new();
    let mut bots = Vec::new();
    for (row, p) in proxies.iter().enumerate() {
        let role = if row == 0 {
            Role::Host {
                persons: soak::PERSONS,
                map: 0,
            }
        } else {
            Role::Join {
                code: String::new(),
            }
        };
        let mut cfg = BotConfig::new(&format!("Bot{row}"), &p.url(), role);
        cfg.team = Some(soak::TEAMS[row]);
        cfg.row = Some(row);
        cfg.max_ticks = TICKS;
        cfg.reconnect_after_ms = 500;
        let mut c = synthetic::client();
        c.direct = direct.map(FakeNet::opener);
        if direct.is_some() {
            c.direct_shape = Some(shape(&bad(40 + row as u64)));
        }
        clients.push(c);
        bots.push(Bot::new(cfg));
    }
    // The host first, then the guests one at a time (rows follow joins).
    let mut joined = 1usize;
    let mut cut = false;
    let deadline = Instant::now() + Duration::from_secs(180);
    loop {
        let ms = now();
        for i in 0..joined {
            clients[i].update(ms);
            synthetic::run_jobs(&mut clients[i]);
            bots[i].step(&mut clients[i], ms);
        }
        if joined < soak::PERSONS
            && bots[joined - 1].stats.row == Some(joined - 1)
            && bots[joined - 1].stats.configured
            && let Some(code) = bots[0].stats.code.clone()
        {
            bots[joined].cfg.role = Role::Join { code };
            joined += 1;
        }
        if !cut && bots[1].stats.ticks >= CUT_AT {
            cut = true;
            proxies[1].cut();
        }
        for (b, c) in bots.iter_mut().zip(&clients) {
            assert!(b.failure().is_none(), "{}: {:?}", b.cfg.name, b.failure());
            let pairs = c.direct_stats();
            if !pairs.is_empty() {
                b.stats.pairs = pairs;
                b.stats.relay_rtt = c.relay_rtt();
            }
        }
        if bots.iter().all(Bot::done) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "timed out: {:?}",
            bots.iter()
                .map(|b| (b.phase.clone(), b.stats.ticks))
                .collect::<Vec<_>>()
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    // Every checkpoint agrees across the four (a rejoined client logs the
    // ticks it ran again; they agree too).
    let mut all: BTreeMap<u32, Vec<_>> = BTreeMap::new();
    for b in &bots {
        for (t, h) in &b.stats.checkpoints {
            all.entry(*t).or_default().push(*h);
        }
    }
    let common = all.values().filter(|v| v.len() >= soak::PERSONS).count();
    assert!(common >= 10, "only {common} common checkpoints");
    for (t, hs) in &all {
        assert!(
            hs.iter().all(|h| *h == hs[0]),
            "the hashes of tick {t} differ"
        );
    }
    for b in &bots {
        let s = &b.stats;
        println!("{}:\n{}", b.cfg.name, s.summary());
        assert_eq!(s.desync, None, "{} noticed a desync", b.cfg.name);
        assert!(s.ticks >= TICKS, "{} ran {} ticks", b.cfg.name, s.ticks);
        assert_eq!(s.period_ms, 16);
        // Two teams of two: one ally each.
        assert_eq!(s.allies, 1, "{}'s allies", b.cfg.name);
        assert!(s.evidence_max > 0);
        assert!(!s.delay.is_empty());
    }
    let r = &bots[1].stats;
    assert_eq!(r.lost, 1, "row 1 lost its game once");
    assert_eq!(r.rejoined_ms.len(), 1, "row 1 was readmitted");
    assert!(r.received_game_bytes > 0, "row 1 was sent the game");
    // Our AI gave orders, and the game took them (from at least one
    // client's replay; the rejoined one's replay starts at its snapshot).
    assert!(bots.iter().map(|b| b.stats.orders).sum::<u64>() > 0);
    for (i, p) in proxies.iter().enumerate() {
        let (up, down) = p.bytes();
        println!("row {i}: up {up} B, down {down} B");
        assert!(up > 0 && down > 0);
    }
    assert!(
        proxies[1]
            .counts
            .connections
            .load(std::sync::atomic::Ordering::Relaxed)
            >= 2
    );
    bots
}

/// The long soak (local lane): see the module comment.
#[test]
#[ignore = "local lane: the 30-minute 2v2 soak"]
fn long_soak_2v2() {
    let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let out = std::env::var("GLASNOST_LANE_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| root.join("local-lane/m9/n6"));
    std::fs::create_dir_all(&out).expect("the output folder");
    let minutes: String = std::env::var("GLASNOST_SOAK_MINUTES").unwrap_or_else(|_| "30".into());
    let mut args: Vec<String> = vec![
        "--headless".into(),
        env!("CARGO_BIN_EXE_glasnost-headless").into(),
        "--out".into(),
        out.display().to_string(),
        "--minutes".into(),
        minutes,
    ];
    if let Ok(t) = std::env::var("GLASNOST_SOAK_TRANSPORT") {
        args.extend(["--transport".into(), t]);
    }
    if let Ok(script) = std::env::var("GLASNOST_SOAK_WASM") {
        // Relative to the repository root (a test runs in its crate's folder).
        let script = root.join(script).display().to_string();
        args.extend(["--wasm-row".into(), "3".into(), "--script".into(), script]);
    }
    let mut s = soak::parse(&args).expect("the soak's options");
    let relay = soak::RelayProcess::start(root, &out.join("relay-log.txt")).expect("the relay");
    s.relay = relay.addr;
    let run = soak::run(&s).expect("the soak runs");
    let text = run.text(&s);
    print!("{text}");
    std::fs::write(out.join("soak.txt"), &text).expect("the soak's report");
    assert!(run.passed(), "the soak failed");
}

/// A split game (M9 N8c): two soaks of two `net-client` processes each,
/// on a relay neither started (here, the test's), the second joining with
/// the room code the first wrote once its rows were seated; row 2's link is
/// cut and it rejoins. Each half passes on its own, and the halves put
/// together (`soak::compare`) agree on every checkpoint of all four rows.
#[test]
fn a_game_split_in_two_halves_agrees() {
    let (_rt, relay) = relay(68);
    let root = std::env::temp_dir().join(format!("glasnost-split-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a folder");
    let code = root.join("code.txt");
    let url = format!("ws://{}/", relay.addr);
    let half = |name: &str, rows: &str, more: &[String]| {
        let mut args: Vec<String> = [
            "--headless",
            env!("CARGO_BIN_EXE_glasnost-headless"),
            "--synthetic",
            "--clean",
            "--ticks",
            "600",
            "--relay-url",
            &url,
            "--rows",
            rows,
            "--out",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        args.push(root.join(name).display().to_string());
        args.extend(more.iter().cloned());
        soak::parse(&args).expect("a half's options")
    };
    let a = half(
        "a",
        "0,1",
        &[
            "--no-trouble".into(),
            "--code-out".into(),
            code.display().to_string(),
        ],
    );
    let b = half(
        "b",
        "2,3",
        &[
            "--trouble".into(),
            "4:2:cut".into(),
            "--code-in".into(),
            code.display().to_string(),
        ],
    );
    let play = |s: soak::Soak| {
        std::thread::spawn(move || {
            let run = soak::run(&s).expect("a half runs");
            let text = run.text(&s);
            std::fs::write(s.out.join("soak.txt"), &text).expect("its report");
            (run.passed(), text)
        })
    };
    let (ta, tb) = (play(a), play(b));
    let (pa, text_a) = ta.join().expect("half a");
    let (pb, text_b) = tb.join().expect("half b");
    println!("{text_a}\n{text_b}");
    assert!(pa && pb, "a half failed");
    let c = soak::compare(&[root.join("a"), root.join("b")], 0, 600);
    println!("{}", c.text());
    assert!(c.passed(), "the halves disagree");
    assert_eq!(c.clients.len(), soak::PERSONS);
    assert!(c.common >= 10, "only {} common checkpoints", c.common);
    assert!(text_b.contains("trouble: row 2 Cut"));
    assert_eq!(
        c.clients[2].report.get("lost").map(String::as_str),
        Some("1")
    );
    let _ = std::fs::remove_dir_all(&root);
}
