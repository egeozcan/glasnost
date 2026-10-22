//! Two real direct links in one process (M9 N7): the signals are carried
//! by hand, as the relay would. They open UDP sockets on the machine's
//! host addresses, so they are opt-in (`--ignored`); the client's tests
//! use fake links. Our own netcode; no spec applies.

#![cfg(not(target_arch = "wasm32"))]

use std::time::{Duration, Instant};

use glasnost_net::lobby::Signal;
use glasnost_rtc::{Ice, MAX_PACKET, Pair, State};

/// How long a link may take to open on one machine.
// nocite: our own test (the client's own connect timeout)
const OPEN_S: u64 = 10;

/// Carries every signal of one end to the other; returns how many.
fn carry(from: &mut Pair, to: &mut Pair) -> usize {
    let mut n = 0;
    while let Some(s) = from.take_signal() {
        assert!(s.plausible(), "{s:?}");
        to.signal(s);
        n += 1;
    }
    n
}

fn open_both() -> (Pair, Pair, Duration) {
    let ice = Ice::default();
    let mut a = Pair::open(&ice, true);
    let mut b = Pair::open(&ice, false);
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(OPEN_S) {
        carry(&mut a, &mut b);
        carry(&mut b, &mut a);
        a.poll();
        b.poll();
        if a.state() == State::Open && b.state() == State::Open {
            return (a, b, t.elapsed());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("no link: {:?} {:?}", a.state(), b.state());
}

#[test]
#[ignore = "opens real UDP sockets"]
fn two_ends_open_a_link_and_pass_packets() {
    let (mut a, mut b, took) = open_both();
    eprintln!("open after {} ms", took.as_millis());
    let t = Instant::now();
    let mut got = Vec::new();
    while got.len() < 20 && t.elapsed() < Duration::from_secs(5) {
        a.send(vec![7; 100]);
        // Too large: never sent.
        a.send(vec![0; MAX_PACKET + 1]);
        a.poll();
        b.poll();
        while let Some(p) = b.recv() {
            got.push(p);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(got.len() >= 20, "{}", got.len());
    assert!(got.iter().all(|p| p == &vec![7; 100]));
    // One end closes: the other sees it in time, or the client's own
    // ping does.
    a.close();
    assert!(matches!(a.state(), State::Closed(_)));
}

#[test]
#[ignore = "opens real UDP sockets"]
fn hostile_signals_end_only_that_attempt() {
    let ice = Ice::default();
    let mut b = Pair::open(&ice, false);
    // An offer that is not SDP, then candidates that are not candidates.
    b.signal(Signal::Offer("v=0\r\nnot an offer".into()));
    b.signal(Signal::Candidate("{\"candidate\":\"garbage\"}".into()));
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(OPEN_S) && b.state() == State::Connecting {
        b.poll();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(matches!(b.state(), State::Closed(_)), "{:?}", b.state());
    // A new attempt in the same process still works.
    let (_a, _b, _) = open_both();
}
