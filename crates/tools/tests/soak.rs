//! The soak (M9 N6). The short test runs in CI: a relay in the test
//! process, four windowless clients on the synthetic install (invented
//! rules and map), each behind its own bad network (`netem`), played by
//! our AI as persons in a 2v2 for a few hundred ticks; one client's link is
//! cut mid-game and it rejoins. Every checkpoint hash of every client must
//! agree and none may notice a desync.
//!
//! The long soak (`#[ignore]`, local lane) is `glasnost_tools::soak`: four
//! `glasnost-headless net-client` processes and the relay binary, on the
//! install of `GLASNOST_GAME_DIR` (or the synthetic one), for
//! `GLASNOST_SOAK_MINUTES` minutes of game time (default 30) or to a
//! winner, with links cut and a process killed and started again; one
//! client is the wasm build under node when `GLASNOST_SOAK_WASM` names its
//! driver script. It writes to the git-ignored `local-lane/m9/n6/`.
//! Run with `cargo test --release -p glasnost-tools --test soak --
//! --ignored --nocapture`. Our own harness; no spec applies.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use glasnost_app::game::bot::{Bot, BotConfig, Role, synthetic};
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

/// Four AI persons in a 2v2 behind bad networks play the synthetic game for
/// a few hundred ticks; row 1's link is cut and it rejoins. Every client's
/// checkpoint hashes agree, none notices a desync, and the numbers are kept.
#[test]
fn four_ai_persons_play_a_2v2_over_bad_links_and_one_rejoins() {
    let (_rt, relay) = relay(66);
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
        clients.push(synthetic::client());
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
        for b in &bots {
            assert!(b.failure().is_none(), "{}: {:?}", b.cfg.name, b.failure());
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
