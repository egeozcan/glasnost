//! Direct links between the clients of a game (M9 N7), on fake links
//! ([`crate::game::direct::fake`]) beside a real relay in the test
//! process: the links carry the frames, hostile bytes on them change
//! nothing, a pair that cannot link and a client switched to the relay
//! only play on the relay, and a link that dies mid-game falls back to the
//! relay and is used again once it comes back; every game agrees tick for
//! tick. Our own netcode; no spec applies.

use std::collections::{BTreeMap, VecDeque};
use std::time::Instant;

use super::*;
use crate::game::client::PairReport;
use crate::game::direct::Timeouts;
use crate::game::direct::fake::FakeNet;
use crate::game::link::{Link, LinkState, native};

/// The relay path's one-way lag in these tests: a relay on loopback
/// answers at once, so without it the relay's copy of a frame would
/// always come first.
const RELAY_LAG: Duration = Duration::from_millis(40);

/// The native link to the relay, [`RELAY_LAG`] late each way.
struct Lagged {
    inner: Box<dyn Link>,
    out: VecDeque<(Instant, Vec<u8>)>,
    inq: VecDeque<(Instant, Vec<u8>)>,
}

impl Lagged {
    fn pump(&mut self) {
        let now = Instant::now();
        while self.out.front().is_some_and(|(t, _)| *t <= now) {
            let (_, b) = self.out.pop_front().unwrap();
            self.inner.send(b);
        }
        while let Some(b) = self.inner.recv() {
            self.inq.push_back((now + RELAY_LAG, b));
        }
    }
}

impl Link for Lagged {
    fn send(&mut self, bytes: Vec<u8>) {
        self.out.push_back((Instant::now() + RELAY_LAG, bytes));
        self.pump();
    }
    fn recv(&mut self) -> Option<Vec<u8>> {
        self.pump();
        if self.inq.front().is_some_and(|(t, _)| *t <= Instant::now()) {
            return self.inq.pop_front().map(|(_, b)| b);
        }
        None
    }
    fn state(&self) -> LinkState {
        self.inner.state()
    }
    fn close(&mut self) {
        self.inner.close();
    }
}

fn lagged(url: &str) -> Box<dyn Link> {
    Box::new(Lagged {
        inner: native::connect(url),
        out: VecDeque::new(),
        inq: VecDeque::new(),
    })
}

/// Short timeouts, so that a test sees attempts fail and come back.
fn quick() -> Timeouts {
    Timeouts {
        connect_ms: 1_500,
        stale_ms: 500,
        ping_ms: 100,
        retry_ms: 700,
        retry_max_ms: 2_000,
    }
}

/// The clients meet (the first hosts; one row per client, then a
/// computer) and start their game.
fn meet(relay: &Relay, clock: &Clock, cs: &mut [&mut Client]) {
    let n = cs.len();
    press_menu(cs[0], MenuHit::Multiplayer);
    press_net(cs[0], NetHit::Host);
    pump(clock, &mut [&mut *cs[0]], "the room", |c| {
        c[0].room_code().is_some()
    });
    let code = cs[0].room_code().unwrap().to_owned();
    for row in 1..=n {
        let want = if row < n {
            SlotKind::Human
        } else {
            SlotKind::Ai
        };
        for _ in 0..4 {
            if cs[0].room().unwrap().setup.slots[row].kind == want {
                break;
            }
            press_room(cs[0], SetupHit::Kind(row, -1));
            let at = cs[0].room().unwrap().setup.slots[row].kind;
            pump(clock, &mut [&mut *cs[0]], "the row's change", |c| {
                c[0].room().unwrap().setup.slots[row].kind != at
            });
        }
        assert_eq!(cs[0].room().unwrap().setup.slots[row].kind, want);
    }
    let _ = relay;
    // One join at a time, so the clients' rows are their order here.
    for i in 1..n {
        press_menu(cs[i], MenuHit::Multiplayer);
        cs[i].netform.code = code.clone();
        press_net(cs[i], NetHit::Join);
        pump(clock, cs, "the join", |c| {
            c[..=i]
                .iter()
                .all(|c| c.room().is_some_and(|v| v.members.len() == i + 1))
        });
    }
    // Each person picks a fixed colour rising with their row, so that the
    // persons' slots are their rows. Set from the last person's row down,
    // so that no step of the cycle from "random" passes a colour another
    // row holds.
    // spec: S-0009 claim 19 (persons by increasing colour slot)
    for row in (0..n).rev() {
        while cs[row].room().unwrap().setup.slots[row].colour != Some(row as u8) {
            let at = cs[row].room().unwrap().setup.slots[row].colour;
            press_room(cs[row], SetupHit::Colour(row, 1));
            pump(clock, cs, "the colour's change", |c| {
                c.iter()
                    .all(|c| c.room().unwrap().setup.slots[row].colour != at)
            });
        }
    }
    for c in cs.iter_mut().skip(1) {
        press_room(c, SetupHit::Start_);
    }
    pump(clock, cs, "ready", |c| {
        c[0].room()
            .unwrap()
            .members
            .iter()
            .all(|m| m.ready || m.row == 0)
    });
    press_room(cs[0], SetupHit::Start_);
    pump(clock, cs, "the start", |c| {
        c.iter()
            .all(|c| c.lockstep().is_some_and(|ls| ls.phase() == Phase::Running))
    });
}

/// A client on the fake network, with short timeouts.
fn on(relay: &Relay, net: &FakeNet, name: &str) -> Client {
    let mut c = client(relay, name);
    c.connect = Some(lagged);
    c.direct = Some(net.opener());
    c.direct_timeouts = quick();
    c
}

fn slot(c: &Client) -> u8 {
    c.lockstep().unwrap().me()
}

/// `c`'s pair with `other`.
fn pair(c: &Client, other: &Client) -> PairReport {
    let s = slot(other);
    c.direct_stats()
        .into_iter()
        .find(|p| p.slot == s)
        .expect("the pair")
}

/// Every two games agree on every tick both saw.
fn agree(hashes: &[BTreeMap<i32, String>], at_least: usize) {
    for a in 0..hashes.len() {
        for b in a + 1..hashes.len() {
            let mut both = 0;
            for (t, h) in &hashes[a] {
                if let Some(theirs) = hashes[b].get(t) {
                    assert_eq!(h, theirs, "tick {t}");
                    both += 1;
                }
            }
            assert!(both >= at_least, "{both} ticks seen by both");
        }
    }
}

/// Two persons on a direct link: their frames come first the short way,
/// the relay still carries every one, and their games agree. Bytes on the
/// link that are not the peer's frames are refused, and frames of the
/// peer that do not read are dropped, without a panic or a desync.
#[test]
fn direct_links_carry_the_frames_and_hostile_bytes_on_them_change_nothing() {
    let (_rt, relay) = relay(51);
    let clock = Clock(Instant::now());
    let net = FakeNet::new(20);
    let mut a = on(&relay, &net, "Ann");
    let mut b = on(&relay, &net, "Bea");
    meet(&relay, &clock, &mut [&mut a, &mut b]);
    let mut hashes = vec![BTreeMap::new(); 2];
    let mut rng = [11u64, 12];
    play_to(&clock, &mut [&mut a, &mut b], &mut hashes, &mut rng, 120);
    let (sa, sb) = (slot(&a), slot(&b));
    for (x, y) in [(&a, &b), (&b, &a)] {
        let p = pair(x, y);
        assert!(p.enabled && p.open, "{p:?}");
        assert!(
            p.stats.connect_ms.is_some() && p.stats.rtt_ms.is_some(),
            "{p:?}"
        );
        assert_eq!(p.stats.fallbacks, 0, "{p:?}");
    }
    // Hostile bytes on a's link from b: noise, a ping of another
    // version, b's header with another slot's frames, the relay's sender
    // byte, and b's frames header over a body that does not read.
    let v = glasnost_net::wire::WIRE_VERSION;
    for bytes in [
        vec![0xff; 40],
        b"GP\x09\x00\x01\x02\x03\x04".to_vec(),
        vec![b'G', b'N', v, sa, 1, 0, 0],
        vec![b'G', b'N', v, glasnost_net::wire::RELAY, 1, 0, 0],
        vec![b'G', b'N', v, sb, 1, 0xff, 0xff, 0xff, 0xff, 0xff],
        Vec::new(),
    ] {
        assert!(net.inject(sa, sb, bytes), "a's link with b");
    }
    play_to(&clock, &mut [&mut a, &mut b], &mut hashes, &mut rng, 300);
    agree(&hashes, 250);
    let p = pair(&a, &b);
    assert!(p.stats.refused >= 4, "{p:?}");
    for (x, y) in [(&a, &b), (&b, &a)] {
        let p = pair(x, y);
        let paths = p.paths.expect("paths");
        assert!(paths.direct_first > 50, "{paths:?}");
        assert_eq!(paths.conflicts, 0, "{paths:?}");
        assert!(x.net_desync().is_none());
        assert!(
            paths.relay_top > 250,
            "the relay carried them too: {paths:?}"
        );
    }
    relay.stop();
}

/// Three persons; one pair can never link: those two play on the relay
/// while the other two pairs link. A client switched to the relay only
/// (the settings' `direct=0`) answers no offer: its pairs stay on the
/// relay. Every game agrees, and nobody does anything.
#[test]
fn a_pair_that_cannot_link_and_a_switched_client_play_on_the_relay() {
    let (_rt, relay) = relay(52);
    let clock = Clock(Instant::now());
    let net = FakeNet::new(20);
    let mut a = on(&relay, &net, "Ann");
    let mut b = on(&relay, &net, "Bea");
    let mut c = on(&relay, &net, "Cy");
    // Rows 0 and 2 never link (rows are slots).
    net.block(0, 2);
    meet(&relay, &clock, &mut [&mut a, &mut b, &mut c]);
    assert_eq!((slot(&a), slot(&b), slot(&c)), (0, 1, 2));
    let mut hashes = vec![BTreeMap::new(); 3];
    let mut rng = [21u64, 22, 23];
    play_to(
        &clock,
        &mut [&mut a, &mut b, &mut c],
        &mut hashes,
        &mut rng,
        250,
    );
    agree(&hashes, 200);
    for (x, y) in [(&a, &b), (&b, &a), (&b, &c), (&c, &b)] {
        let p = pair(x, y);
        assert!(p.open, "{p:?}");
        assert!(p.paths.unwrap().direct_first > 0, "{p:?}");
    }
    for (x, y) in [(&a, &c), (&c, &a)] {
        let p = pair(x, y);
        assert!(!p.open, "{p:?}");
        assert_eq!(p.stats.opened, 0, "{p:?}");
        assert_eq!(p.paths.unwrap().direct_first, 0, "{p:?}");
    }
    // The offering end tried, gave up and tried again.
    let p = pair(&a, &c);
    assert!(p.stats.attempts >= 2 && p.stats.failed >= 1, "{p:?}");
    relay.stop();

    // The relay-only switch. The relay-only end declines the offer (M9
    // N8c), so the offering end makes one attempt in all, not one per
    // back-off: 300 ticks are several connect timeouts and retries.
    let (_rt2, relay) = super::relay(53);
    let mut a = on(&relay, &net, "Ann");
    let mut b = on(&relay, &net, "Bea");
    b.settings.direct = false;
    meet(&relay, &clock, &mut [&mut a, &mut b]);
    let mut hashes = vec![BTreeMap::new(); 2];
    play_to(&clock, &mut [&mut a, &mut b], &mut hashes, &mut rng, 300);
    agree(&hashes, 250);
    let p = pair(&b, &a);
    assert!(!p.enabled && !p.open && p.stats.attempts == 0, "{p:?}");
    let p = pair(&a, &b);
    assert!(p.enabled && !p.open, "{p:?}");
    assert_eq!((p.stats.attempts, p.stats.failed), (1, 1), "{p:?}");
    for (x, y) in [(&a, &b), (&b, &a)] {
        assert_eq!(pair(x, y).paths.unwrap().direct_first, 0);
    }
    relay.stop();
}

/// A direct link that goes silent mid-game: both ends give it up, the
/// relay carries the frames, the game never waits for the link, and once
/// the route is back the lower slot links again and the link is used.
#[test]
fn a_direct_link_that_dies_mid_game_falls_back_and_is_used_again() {
    let (_rt, relay) = relay(54);
    let clock = Clock(Instant::now());
    let net = FakeNet::new(20);
    let mut a = on(&relay, &net, "Ann");
    let mut b = on(&relay, &net, "Bea");
    meet(&relay, &clock, &mut [&mut a, &mut b]);
    let mut hashes = vec![BTreeMap::new(); 2];
    let mut rng = [31u64, 32];
    play_to(&clock, &mut [&mut a, &mut b], &mut hashes, &mut rng, 100);
    assert!(pair(&a, &b).open && pair(&b, &a).open);
    let before = pair(&a, &b).paths.unwrap();

    net.darken(0, 1);
    let mut until = 100;
    while pair(&a, &b).stats.fallbacks == 0 || pair(&b, &a).stats.fallbacks == 0 {
        until += 10;
        play_to(&clock, &mut [&mut a, &mut b], &mut hashes, &mut rng, until);
        assert!(until < 600, "no fallback: {:?}", pair(&a, &b));
    }
    let dark = pair(&a, &b).paths.unwrap();
    play_to(
        &clock,
        &mut [&mut a, &mut b],
        &mut hashes,
        &mut rng,
        until + 80,
    );
    let relayed = pair(&a, &b).paths.unwrap();
    assert!(
        relayed.relay_first > dark.relay_first + 40,
        "the relay carried the frames: {relayed:?}"
    );

    net.light(0, 1);
    until += 80;
    while !(pair(&a, &b).open && pair(&b, &a).open) {
        until += 10;
        play_to(&clock, &mut [&mut a, &mut b], &mut hashes, &mut rng, until);
        assert!(until < 1_000, "no new link: {:?}", pair(&a, &b));
    }
    let back = pair(&a, &b).paths.unwrap();
    play_to(
        &clock,
        &mut [&mut a, &mut b],
        &mut hashes,
        &mut rng,
        until + 100,
    );
    agree(&hashes, until as usize);
    let p = pair(&a, &b);
    assert!(p.stats.opened >= 2, "{p:?}");
    let after = p.paths.unwrap();
    assert!(
        after.direct_first > back.direct_first + 30,
        "the new link carries frames: {before:?} {after:?}"
    );
    relay.stop();
}
