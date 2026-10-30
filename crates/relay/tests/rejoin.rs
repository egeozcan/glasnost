//! Reconnects through the relay's core (M9 N5), with no sockets: persons
//! (and a computer) meet in a room of the hub, start, and each runs its own
//! lockstep session and its own `glasnost_ai::game::Game` on the synthetic
//! skirmish (invented rules and map, `glasnost_ai::fixture`), on a virtual
//! clock with a fixed latency from the hub to each client. One peer is cut
//! off mid-game; a new connection brings its slot's ticket back, is sent
//! the game by a live peer, catches up and is readmitted, and from then on
//! every peer's state hash after every tick equals the others' and a
//! single-process run of the same orders. Past the grace period the
//! dropped house surrenders on every peer on the same tick; forged relay
//! messages, markers and tickets are refused. The rejoin protocol and the
//! grace period are our own netcode; the surrender is S-0414 claim 11's
//! order.
//!
//! Some games also give pairs of peers a direct link (N7): a queue beside
//! the hub that carries each peer's frames messages to the other a few
//! milliseconds after it sends them, while its messages to the hub may lag.
//! The peers then run frames the hub has not forwarded yet; a drop of such
//! a peer goes through the hub's round of `Closing` and `Held`, and every
//! peer still agrees with every other.

mod common;

use std::collections::{BTreeMap, VecDeque};

use common::skirmish::{build, game, map8, scripted};
use common::{Fake, Incoming, create, join, ready};
use glasnost_ai::game::Game;
use glasnost_map::Map;
use glasnost_net::lobby::{self, Begin, FromRelay, Part, Refusal, Signal, ToRelay};
use glasnost_net::rejoin::{self, Assembler, Transfer};
use glasnost_net::simnet::SplitMix;
use glasnost_net::wire::{self, Body, Envelope, Frame, Frames, Hash};
use glasnost_net::{Config, Event, Field, Phase, Session, Simulation, StartInfo, Tick, Via};
use glasnost_relay::{ConnId, Hub, Limits, Out};
use glasnost_sim::orders::Order;
use glasnost_sim::world::HouseId;
use glasnost_ui::setup::SlotKind;

/// The tick period of the test games, in milliseconds.
const TICK_MS: u32 = 20;

/// Milliseconds from the hub to a client.
const LATENCY: u64 = 12;

/// Milliseconds of the virtual clock per round of the loop.
const STEP: u64 = 2;

/// Ticks a catching-up peer runs at most per round.
const CATCH_UP_BURST: usize = 64;

/// Snapshot parts a serving peer sends per round.
const PARTS_PER_ROUND: usize = 2;

/// Milliseconds a direct link takes (N7).
const DIRECT_MS: u64 = 3;

/// The clock of the first peers' processes when the game begins: a
/// rejoining peer is a new process whose clock starts at 0, far below the
/// stamps its slot's old process sent.
const FIRST_CLOCK: u64 = 1_000_000;

/// The hub and what it sent each connection, delivered `LATENCY` later,
/// and the direct links' messages in flight, by receiving slot.
struct Net {
    hub: Hub,
    t: u64,
    inbox: BTreeMap<ConnId, VecDeque<(u64, Incoming)>>,
    wires: BTreeMap<u8, VecDeque<(u64, u8, Vec<u8>)>>,
}

impl Net {
    fn new(limits: Limits) -> Self {
        Self {
            hub: Hub::new(limits, Box::new(Fake(0x5eed_1234))),
            t: 0,
            inbox: BTreeMap::new(),
            wires: BTreeMap::new(),
        }
    }

    /// A frames message from slot `from` on its direct link to `to`.
    fn wire(&mut self, from: u8, to: u8, bytes: Vec<u8>) {
        let at = self.t + DIRECT_MS;
        self.wires
            .entry(to)
            .or_default()
            .push_back((at, from, bytes));
    }

    /// The next direct message for slot `to` that has arrived.
    fn next_wire(&mut self, to: u8) -> Option<(u8, Vec<u8>)> {
        let q = self.wires.get_mut(&to)?;
        if q.front().is_some_and(|(at, _, _)| *at <= self.t) {
            q.pop_front().map(|(_, f, b)| (f, b))
        } else {
            None
        }
    }

    fn connect(&mut self) -> ConnId {
        let c = self.hub.connect(self.t).expect("room for a connection");
        self.inbox.insert(c, VecDeque::new());
        c
    }

    fn send(&mut self, conn: ConnId, bytes: &[u8]) {
        self.hub.receive(self.t, conn, bytes);
        self.route();
    }

    fn lobby(&mut self, conn: ConnId, m: &ToRelay) {
        self.send(conn, &lobby::wire::encode(m));
    }

    /// The connection is cut, as by a crash.
    fn kill(&mut self, conn: ConnId) {
        self.inbox.remove(&conn);
        self.hub.closed(self.t, conn);
        self.route();
    }

    fn route(&mut self) {
        let at = self.t + LATENCY;
        for o in self.hub.take_out() {
            let (to, item) = match o {
                Out::Send { to, bytes } if lobby::wire::is_lobby(&bytes) => (
                    to,
                    Incoming::Lobby(lobby::wire::decode(&bytes).expect("the hub's lobby message")),
                ),
                Out::Send { to, bytes } => (to, Incoming::Game(bytes)),
                Out::Keepalive { to } => {
                    self.hub.heard(self.t, to);
                    continue;
                }
                Out::Close { to, reason } => (to, Incoming::Closed(Some(reason))),
            };
            if let Some(q) = self.inbox.get_mut(&to) {
                q.push_back((at, item));
            }
        }
    }

    fn next(&mut self, conn: ConnId) -> Option<Incoming> {
        let q = self.inbox.get_mut(&conn)?;
        if q.front().is_some_and(|(at, _)| *at <= self.t) {
            q.pop_front().map(|(_, m)| m)
        } else {
            None
        }
    }

    /// Everything delivered to `conn` by now.
    fn drain(&mut self, conn: ConnId) -> Vec<Incoming> {
        std::iter::from_fn(|| self.next(conn)).collect()
    }

    fn advance(&mut self) {
        self.t += STEP;
        if self.t.is_multiple_of(100) {
            self.hub.tick(self.t);
            self.route();
        }
    }
}

/// One person's client: its session (none while a rejoin waits for the
/// game), its game, and what it saw.
struct Peer {
    slot: u8,
    begin: Begin,
    info: StartInfo,
    conn: Option<ConnId>,
    session: Option<Session>,
    game: Game,
    house: HouseId,
    rng: SplitMix,
    next_due: u64,
    input_for: Option<u32>,
    hashes: BTreeMap<u32, Hash>,
    ticks: BTreeMap<u32, Tick>,
    events: Vec<Event>,
    /// Lockstep messages that came before the game did.
    early: Vec<Vec<u8>>,
    asm: Assembler,
    /// Snapshot parts still to send, encoded.
    parts: VecDeque<Vec<u8>>,
    caught_up_sent: bool,
    lobby: Vec<FromRelay>,
    closed: Option<String>,
    /// Bytes of game this peer was sent, and when it had them all.
    transfer: Option<(usize, u64)>,
    /// Bytes of game this peer sent.
    served: Vec<usize>,
    /// The slots it has a direct link to (N7).
    direct: Vec<u8>,
    /// It told the hub of its direct links.
    signalled: bool,
    /// How long its lockstep messages take to the hub, and those on the
    /// way.
    lag: u64,
    outq: VecDeque<(u64, Vec<u8>)>,
    /// Its process's clock at the virtual time 0 (a rejoining peer's starts
    /// at 0 when it is made).
    clock0: i64,
}

impl Peer {
    fn new(begin: Begin, map: &Map, conn: ConnId, rejoining: bool, now: u64) -> Self {
        let (game, info, _) = build(&begin, map);
        let house = HouseId(info.slots[usize::from(begin.you)].house);
        let clock0 = if rejoining {
            -(now as i64)
        } else {
            FIRST_CLOCK as i64
        };
        let session = (!rejoining).then(|| {
            Session::new(
                Config::new(TICK_MS),
                info.clone(),
                begin.you,
                now.saturating_add_signed(clock0),
            )
            .expect("a person's slot")
        });
        Self {
            slot: begin.you,
            begin,
            info,
            conn: Some(conn),
            session,
            game,
            house,
            rng: SplitMix::new(1000 + now),
            next_due: 0,
            input_for: None,
            hashes: BTreeMap::new(),
            ticks: BTreeMap::new(),
            events: Vec::new(),
            early: Vec::new(),
            asm: Assembler::default(),
            parts: VecDeque::new(),
            caught_up_sent: false,
            lobby: Vec::new(),
            closed: None,
            transfer: None,
            served: Vec::new(),
            direct: Vec::new(),
            signalled: false,
            lag: 0,
            outq: VecDeque::new(),
            clock0,
        }
    }

    /// Its process's clock at virtual time `t`.
    fn clock(&self, t: u64) -> u64 {
        t.saturating_add_signed(self.clock0)
    }

    fn ran(&self) -> u32 {
        self.session.as_ref().map_or(0, Session::ran)
    }

    fn live(&self) -> bool {
        self.conn.is_some()
    }

    /// Readmitted after a rejoin, or never away.
    fn playing(&self) -> bool {
        self.session.as_ref().is_some_and(|s| !s.catching_up())
    }

    fn record(&mut self, done: Tick) {
        self.hashes
            .insert(done.tick, Simulation::state_hash(&self.game));
        self.ticks.insert(done.tick, done);
    }

    /// One round: take what arrived, send snapshot parts, run what is due,
    /// send what the session has.
    fn run(&mut self, net: &mut Net) {
        let Some(conn) = self.conn else {
            return;
        };
        let t = net.t;
        let now = self.clock(t);
        while let Some((from, b)) = net.next_wire(self.slot) {
            // As a client does: a link is used while the session says so.
            if let Some(s) = self.session.as_mut().filter(|s| s.direct_ok(from)) {
                s.receive_via(now, &b, Via::Direct(from));
            }
        }
        while let Some(m) = net.next(conn) {
            match m {
                Incoming::Game(b) => match self.session.as_mut() {
                    Some(s) => s.receive(now, &b),
                    None => self.early.push(b),
                },
                Incoming::Lobby(FromRelay::Snapshot(part)) => self.take_part(&part, t),
                Incoming::Lobby(m) => self.lobby.push(m),
                Incoming::Closed(r) => {
                    self.closed = r;
                    self.conn = None;
                    return;
                }
            }
        }
        for _ in 0..PARTS_PER_ROUND {
            if let Some(b) = self.parts.pop_front() {
                net.send(conn, &b);
            }
        }
        let Some(s) = self.session.as_mut() else {
            return;
        };
        if s.catching_up() {
            // Unpaced, then once blocked on frames: caught up.
            let mut blocked = false;
            for _ in 0..CATCH_UP_BURST {
                match s.step(now, &mut self.game) {
                    Some(done) => {
                        self.hashes
                            .insert(done.tick, Simulation::state_hash(&self.game));
                        self.ticks.insert(done.tick, done);
                    }
                    None => {
                        blocked = true;
                        break;
                    }
                }
            }
            if blocked && !self.caught_up_sent {
                self.caught_up_sent = true;
                net.lobby(conn, &ToRelay::CaughtUp);
            }
        } else if s.phase() == Phase::Running && t >= self.next_due {
            let tick = s.ran();
            if self.input_for != Some(tick) {
                self.input_for = Some(tick);
                for o in scripted(&self.game, self.house, tick, &mut self.rng) {
                    s.submit(o);
                }
            }
            if let Some(done) = s.step(now, &mut self.game) {
                self.next_due = t + u64::from(TICK_MS);
                self.record(done);
            }
        }
        let s = self.session.as_mut().expect("a session");
        if !self.signalled
            && s.phase() == Phase::Running
            && !s.catching_up()
            && !self.direct.is_empty()
        {
            // The signals that would set the links up; the hub remembers
            // the pairs.
            self.signalled = true;
            for &to in &self.direct {
                net.lobby(
                    conn,
                    &ToRelay::Signal {
                        to,
                        attempt: 1,
                        signal: Signal::Offer("v=0\r\n".into()),
                    },
                );
            }
        }
        for m in s.poll(now) {
            if wire::is_frames_from(&m, self.slot) {
                for &to in self.direct.iter().filter(|&&d| s.direct_ok(d)) {
                    net.wire(self.slot, to, m.clone());
                }
            }
            if self.lag > 0 {
                self.outq.push_back((t + self.lag, m));
            } else {
                net.send(conn, &m);
            }
        }
        while self.outq.front().is_some_and(|(at, _)| *at <= t) {
            let (_, m) = self.outq.pop_front().expect("a message");
            net.send(conn, &m);
        }
        for e in s.take_events() {
            if let Event::Serve { .. } = e {
                let bytes = Transfer::of(s, &self.game)
                    .expect("a running session")
                    .to_bytes();
                self.served.push(bytes.len());
                let ps = rejoin::parts(&bytes);
                let n = ps.len() as u32;
                for (i, b) in ps.into_iter().enumerate() {
                    self.parts
                        .push_back(lobby::wire::encode(&ToRelay::Snapshot(Part {
                            part: i as u32,
                            parts: n,
                            bytes: b,
                        })));
                }
            }
            self.events.push(e);
        }
    }

    fn take_part(&mut self, part: &Part, t: u64) {
        let Some(bytes) = self
            .asm
            .add(part.part, part.parts, &part.bytes)
            .expect("the parts in order")
        else {
            return;
        };
        let tr = Transfer::from_bytes(&bytes).expect("a transfer");
        self.game = tr
            .load(&self.game)
            .expect("the game loads to the sent hash");
        let now = self.clock(t);
        let mut s = Session::rejoin(
            Config::new(TICK_MS),
            self.info.clone(),
            self.slot,
            &tr.catchup,
            now,
        )
        .expect("a catch-up of our slot");
        for b in self.early.drain(..) {
            s.receive(now, &b);
        }
        self.session = Some(s);
        self.transfer = Some((bytes.len(), t));
    }
}

/// The persons of `rows` (the host first) meet in a room and begin: each
/// connection's `Begin`.
fn meet(net: &mut Net, rows: &[SlotKind]) -> (String, Vec<(ConnId, Begin)>) {
    let host = net.connect();
    net.lobby(host, &create(rows, "synthetic-8", 8));
    let code = net.hub.codes()[0].clone();
    let persons = rows.iter().filter(|k| **k == SlotKind::Human).count();
    let mut conns = vec![host];
    for i in 1..persons {
        let c = net.connect();
        net.lobby(c, &join(&code, &format!("P{i}"), "build-1", "synthetic-8"));
        conns.push(c);
    }
    for &c in &conns[1..] {
        net.lobby(c, &ready("synthetic-8"));
    }
    net.lobby(host, &ToRelay::Start);
    net.t += LATENCY;
    let mut out = Vec::new();
    for c in conns {
        let begin = net
            .drain(c)
            .into_iter()
            .find_map(|m| match m {
                Incoming::Lobby(FromRelay::Begin(b)) => Some(b),
                _ => None,
            })
            .expect("a begin");
        out.push((c, begin));
    }
    (code, out)
}

/// A game with one peer cut off.
struct Plan {
    rows: Vec<SlotKind>,
    limits: Limits,
    /// The slot cut off.
    victim: u8,
    /// Its ticks run when it is cut off.
    kill_at: u32,
    /// Milliseconds after the cut that its slot rejoins, if it does.
    back_after: Option<u64>,
    /// The ticks every peer still in the game runs.
    ticks: u32,
    /// The first rejoining peer is cut off as soon as it says it caught
    /// up; the slot rejoins again `back_after` later.
    cut_back: bool,
    /// Pairs of slots with a direct link (N7).
    direct: Vec<(u8, u8)>,
    /// Slots whose lockstep messages take this long to the hub.
    lag: Vec<(u8, u64)>,
    /// A slot cut off for good once it has run this many ticks, before
    /// the victim is.
    gone: Option<(u8, u32)>,
    /// A slot cut off for good once the rejoined peer plays and has run
    /// this many ticks.
    leaves: Option<(u8, u32)>,
    /// The rejoined peer signals the victim's direct links again (N7).
    relink: bool,
}

struct Played {
    net: Net,
    code: String,
    map: Map,
    /// The first peers, by slot.
    peers: Vec<Peer>,
    /// The rejoined peer.
    back: Option<Peer>,
    /// A rejoining peer cut off in turn.
    lost: Option<Peer>,
    /// When the victim was cut off.
    cut_at: u64,
    /// The victim's frames the hub had forwarded then, and the most ticks
    /// any other peer had run.
    at_cut: (u32, u32),
}

impl Played {
    fn survivors(&self, victim: u8) -> Vec<&Peer> {
        self.peers.iter().filter(|p| p.slot != victim).collect()
    }
}

fn play(plan: &Plan) -> Played {
    let mut net = Net::new(plan.limits);
    let (code, met) = meet(&mut net, &plan.rows);
    let map = map8();
    let now = net.t;
    let mut peers: Vec<Peer> = met
        .into_iter()
        .map(|(c, b)| Peer::new(b, &map, c, false, now))
        .collect();
    peers.sort_by_key(|p| p.slot);
    assert!(
        peers
            .iter()
            .enumerate()
            .all(|(i, p)| usize::from(p.slot) == i)
    );
    for &(a, b) in &plan.direct {
        peers[usize::from(a)].direct.push(b);
        peers[usize::from(b)].direct.push(a);
    }
    for &(s, lag) in &plan.lag {
        peers[usize::from(s)].lag = lag;
    }
    let mut at_cut = (0, 0);
    let v = usize::from(plan.victim);
    let mut back: Option<Peer> = None;
    let mut lost: Option<Peer> = None;
    let mut cut_at = None;
    let mut last_cut = None;
    let mut gone = false;
    let mut left = false;
    // Virtual milliseconds: the ticks at their period, three times over.
    let deadline = 3 * u64::from(plan.ticks) * u64::from(TICK_MS) + 10_000;
    loop {
        if net.t >= deadline {
            let state: Vec<String> = peers
                .iter()
                .chain(back.as_ref())
                .map(|p| {
                    let s = p.session.as_ref();
                    format!(
                        "slot {} live {} ran {} catching {:?} waiting {:?} lives {:?} parts {} closed {:?} lobby {:?}",
                        p.slot,
                        p.live(),
                        p.ran(),
                        s.map(Session::catching_up),
                        s.map(Session::waiting_for),
                        s.map(|s| (0..4).map(|i| s.life(i)).collect::<Vec<_>>()),
                        p.parts.len(),
                        p.closed,
                        p.lobby,
                    )
                })
                .collect();
            panic!(
                "the game did not finish in time:\n{}\n{:?}",
                state.join("\n"),
                net.hub.stats()
            );
        }
        for p in &mut peers {
            p.run(&mut net);
        }
        if let Some(b) = back.as_mut() {
            b.run(&mut net);
        }
        if let Some((g, at)) = plan.gone
            && !gone
            && peers[usize::from(g)].ran() >= at
        {
            let c = peers[usize::from(g)].conn.take().expect("a connection");
            net.kill(c);
            gone = true;
        }
        if let Some((l, at)) = plan.leaves
            && !left
            && back.as_ref().is_some_and(|b| b.playing() && b.ran() >= at)
        {
            let c = peers[usize::from(l)].conn.take().expect("a connection");
            net.kill(c);
            left = true;
        }
        if cut_at.is_none() && (plan.gone.is_none() || gone) && peers[v].ran() >= plan.kill_at {
            at_cut = (
                net.hub.kept(&code, plan.victim).expect("a game"),
                peers
                    .iter()
                    .filter(|p| p.slot != plan.victim)
                    .map(Peer::ran)
                    .max()
                    .unwrap_or(0),
            );
            let c = peers[v].conn.take().expect("the victim's connection");
            net.kill(c);
            cut_at = Some(net.t);
            last_cut = cut_at;
        }
        if plan.cut_back
            && lost.is_none()
            && let Some(mut b) = back.take_if(|b| b.caught_up_sent)
        {
            net.kill(b.conn.take().expect("the rejoining connection"));
            lost = Some(b);
            last_cut = Some(net.t);
        }
        if let (Some(at), Some(after), None) = (last_cut, plan.back_after, back.as_ref())
            && net.t >= at + after
        {
            let conn = net.connect();
            let b = peers[v].begin.clone();
            net.lobby(
                conn,
                &ToRelay::Rejoin {
                    code: code.clone(),
                    slot: plan.victim,
                    ticket: b.ticket,
                    build: b.build.clone(),
                    rules_hash: b.rules_hash.clone(),
                },
            );
            let mut b = Peer::new(b, &map, conn, true, net.t);
            if plan.relink {
                b.direct = peers[v].direct.clone();
            }
            back = Some(b);
        }
        let others_done = peers
            .iter()
            .filter(|p| p.live())
            .all(|p| p.ran() >= plan.ticks);
        let back_done = match (&back, plan.back_after) {
            (_, None) => true,
            (None, Some(_)) => false,
            (Some(b), _) => {
                !b.live()
                    || b.lobby.iter().any(|m| matches!(m, FromRelay::Refused(_)))
                    || (b.playing() && b.ran() >= plan.ticks)
            }
        };
        if cut_at.is_some()
            && others_done
            && back_done
            && (!plan.cut_back || lost.is_some())
            && (plan.leaves.is_none() || left)
        {
            break;
        }
        net.advance();
    }
    Played {
        net,
        code,
        map,
        peers,
        back,
        lost,
        cut_at: cut_at.expect("cut"),
        at_cut,
    }
}

/// No peer saw a desync, a refusal or a bad message.
fn clean(p: &Peer) {
    for e in &p.events {
        assert!(
            !matches!(
                e,
                Event::Desync { .. } | Event::Refused { .. } | Event::BadMessage { .. }
            ),
            "slot {}: {e:?}",
            p.slot
        );
    }
}

/// `a` and `b` ran the same orders to the same hashes on every tick both
/// ran; how many that is.
fn agree(a: &Peer, b: &Peer) -> usize {
    let mut n = 0;
    for (t, h) in &a.hashes {
        if let Some(theirs) = b.hashes.get(t) {
            assert_eq!(h, theirs, "slots {} and {}: tick {t}", a.slot, b.slot);
            assert_eq!(a.ticks[t], b.ticks[t], "the orders of tick {t}");
            n += 1;
        }
    }
    n
}

/// A single-process run of `p`'s ticks gives its hashes.
fn replays(p: &Peer, map: &Map) {
    let (_, _, gs) = build(&p.begin, map);
    let mut one = game(map, &gs);
    for (t, tick) in &p.ticks {
        for a in &tick.orders {
            one.issue(a.house, a.order.clone());
        }
        Simulation::step(&mut one);
        assert_eq!(Simulation::state_hash(&one), p.hashes[t], "tick {t}");
    }
}

/// The ticks whose orders hold `slot`'s surrender.
fn surrenders(p: &Peer, slot: u8) -> Vec<u32> {
    p.ticks
        .values()
        .filter(|t| {
            t.orders
                .iter()
                .any(|a| a.slot == slot && a.order == Order::Surrender)
        })
        .map(|t| t.tick)
        .collect()
}

fn kept(p: &Peer, slot: u8) -> Vec<u32> {
    p.events
        .iter()
        .filter_map(|e| match e {
            Event::Dropped { slot: s, kept } if *s == slot => Some(*kept),
            _ => None,
        })
        .collect()
}

fn resumed(p: &Peer, slot: u8) -> Vec<u32> {
    p.events
        .iter()
        .filter_map(|e| match e {
            Event::Resumed { slot: s, from } if *s == slot => Some(*from),
            _ => None,
        })
        .collect()
}

/// A drop and a rejoin: the rejoined peer and every other run the same
/// ticks to the same hashes, the slot is readmitted on the same frame
/// everywhere, and nobody surrenders.
fn drop_and_rejoin(plan: &Plan) -> Played {
    let played = play(plan);
    let v = plan.victim;
    let back = played.back.as_ref().expect("a rejoin");
    let rejoins = 1 + u32::from(plan.cut_back);
    assert!(back.live(), "the rejoined peer stays: {:?}", back.closed);
    assert!(back.playing(), "readmitted");
    assert_eq!(
        back.lobby
            .iter()
            .filter_map(|m| match m {
                FromRelay::Rejoined(b) => Some(b),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [&played.peers[usize::from(v)].begin],
        "the rejoin's start values are the slot's own"
    );
    let survivors = played.survivors(v);
    for p in survivors.iter().copied().chain([back]) {
        clean(p);
        assert!(surrenders(p, v).is_empty(), "slot {}: no surrender", p.slot);
        assert_eq!(p.session.as_ref().and_then(|s| s.surrender_at(v)), None);
    }
    for p in &survivors {
        replays(p, &played.map);
    }
    // The rejoined process measures its round trips again, though its clock
    // starts far below its slot's old stamps, which the peers echoed while
    // the slot was away.
    let s = back.session.as_ref().expect("a session");
    for p in survivors.iter().filter(|p| p.info.is_human(p.slot)) {
        assert!(s.rtt(p.slot).is_some(), "a round trip to slot {}", p.slot);
    }
    // Each drop at the same frame, each readmission from the same frame,
    // everywhere; a rejoining peer cut off after it asked to be readmitted
    // drops again at its readmission's first frame.
    let k = kept(survivors[0], v);
    assert_eq!(k.len() as u32, rejoins, "{k:?}");
    let from = resumed(survivors[0], v);
    assert_eq!(from.len() as u32, rejoins, "{from:?}");
    for p in &survivors {
        assert_eq!(kept(p, v), k);
        assert_eq!(resumed(p, v), from);
    }
    let last = from.len() - 1;
    assert_eq!(
        resumed(back, v),
        from[last..],
        "the rejoined peer readmits itself"
    );
    assert!(from[0] >= k[0]);
    if plan.cut_back {
        assert_eq!(k[1], from[0], "dropped again where it would have come back");
        let lost = played.lost.as_ref().expect("the first rejoining peer");
        assert!(lost.transfer.is_some() && lost.caught_up_sent);
        for p in &survivors {
            agree(p, lost);
        }
    }
    let (k, from) = (&k[last..], &from[last..]);
    let donors: Vec<u8> = survivors
        .iter()
        .filter(|p| !p.served.is_empty())
        .map(|p| p.slot)
        .collect();
    assert_eq!(donors.len(), 1, "one peer serves the rejoins");
    // The rejoined peer runs from where it was sent the game, the same as
    // everyone; its orders count again from the readmission on.
    let first = *back.ticks.keys().next().expect("ticks after the rejoin");
    assert!(first >= k[0], "caught up from after the drop");
    for p in &survivors {
        let n = agree(p, back);
        assert!(n as u32 >= plan.ticks - first - 1, "slot {}: {n}", p.slot);
        let theirs: Vec<u32> = p
            .ticks
            .values()
            .filter(|t| t.orders.iter().any(|a| a.slot == v))
            .map(|t| t.tick)
            .collect();
        assert!(
            theirs.iter().all(|&t| t < k[0] || t >= from[0]),
            "the slot's orders only while it was in: {theirs:?}"
        );
        assert!(
            theirs.iter().any(|&t| t >= from[0]),
            "its orders after the readmission count"
        );
    }
    for (i, a) in survivors.iter().enumerate() {
        for b in &survivors[i + 1..] {
            agree(a, b);
        }
    }
    let stats = played.net.hub.stats();
    assert_eq!((stats.rejoins, stats.readmitted), (u64::from(rejoins), 1));
    assert_eq!(stats.forged_rejoins, 0);
    assert_eq!(played.net.hub.rejoining(&played.code), None);
    let (bytes, loaded) = back.transfer.expect("the game came");
    let sent: usize = survivors.iter().flat_map(|p| p.served.iter()).sum();
    assert_eq!(stats.snapshot_bytes, sent as u64);
    eprintln!(
        "rejoin of slot {v}: {bytes} bytes of game, loaded {} ms after the cut, caught up from tick {first}, readmitted from frame {}",
        loaded - played.cut_at,
        from[0]
    );
    played
}

fn limits() -> Limits {
    Limits::default()
}

#[test]
fn two_persons_drop_and_rejoin_with_identical_hashes() {
    drop_and_rejoin(&Plan {
        rows: vec![SlotKind::Human, SlotKind::Human],
        limits: limits(),
        victim: 1,
        kill_at: 90,
        back_after: Some(1_500),
        ticks: 400,
        cut_back: false,
        direct: vec![],
        lag: vec![],
        gone: None,
        leaves: None,
        relink: false,
    });
}

/// A member whose peer went before the game ran (while its map loaded,
/// say) comes in late by the same rejoin: its frames count from the
/// readmission on, and it plays on with the same hashes.
#[test]
fn a_member_gone_before_the_game_ran_joins_late() {
    let played = drop_and_rejoin(&Plan {
        rows: vec![SlotKind::Human, SlotKind::Human, SlotKind::Ai],
        limits: limits(),
        victim: 1,
        kill_at: 0,
        back_after: Some(1_500),
        ticks: 400,
        cut_back: false,
        direct: vec![],
        lag: vec![],
        gone: None,
        leaves: None,
        relink: false,
    });
    let host = &played.peers[0];
    assert_eq!(kept(host, 1), [0], "none of its frames counted");
}

#[test]
fn three_persons_and_a_computer_drop_and_rejoin_with_identical_hashes() {
    // The victim is slot 0, so the serving peer is slot 1.
    let played = drop_and_rejoin(&Plan {
        rows: vec![
            SlotKind::Human,
            SlotKind::Human,
            SlotKind::Human,
            SlotKind::Ai,
        ],
        limits: limits(),
        victim: 0,
        kill_at: 120,
        back_after: Some(2_000),
        ticks: 450,
        cut_back: false,
        direct: vec![],
        lag: vec![],
        gone: None,
        leaves: None,
        relink: false,
    });
    assert!(!played.peers[1].served.is_empty(), "slot 1 serves");
    assert!(
        played.peers[0].begin.slots.len() == 4,
        "the computer's slot is in the game"
    );
}

/// The grace period of these tests: 6.6 s at the room's speed (the
/// control at 2, index 4, 66 ms a tick in a network game, S-0201 claim 17
/// and F5) is 100 ticks.
fn short_grace() -> Limits {
    Limits {
        grace_ms: 6_600,
        ..Limits::default()
    }
}

#[test]
fn past_the_grace_period_the_house_surrenders_on_every_peer_on_one_tick() {
    let victim = 2;
    let mut played = play(&Plan {
        rows: vec![SlotKind::Human, SlotKind::Human, SlotKind::Human],
        limits: short_grace(),
        victim,
        kill_at: 60,
        back_after: None,
        ticks: 260,
        cut_back: false,
        direct: vec![],
        lag: vec![],
        gone: None,
        leaves: None,
        relink: false,
    });
    let survivors = played.survivors(victim);
    assert_eq!(survivors[0].begin.params.grace_ticks, 100);
    let k = kept(survivors[0], victim);
    assert_eq!(k.len(), 1);
    let x = k[0] + 100;
    for p in &survivors {
        clean(p);
        assert_eq!(kept(p, victim), k);
        assert_eq!(surrenders(p, victim), [x], "slot {}", p.slot);
        let applied = p.ticks[&x]
            .orders
            .iter()
            .find(|a| a.slot == victim)
            .expect("the surrender");
        assert!(applied.taken, "the simulation takes the surrender");
        assert!(p.events.contains(&Event::Surrendered {
            slot: victim,
            tick: x
        }));
        assert_eq!(
            p.session.as_ref().and_then(|s| s.surrender_at(victim)),
            None
        );
        replays(p, &played.map);
    }
    let n = agree(survivors[0], survivors[1]);
    assert_eq!(n, 260);
    // A rejoin after that is refused.
    let net = &mut played.net;
    let conn = net.connect();
    let b = played.peers[usize::from(victim)].begin.clone();
    net.lobby(
        conn,
        &ToRelay::Rejoin {
            code: played.code.clone(),
            slot: victim,
            ticket: b.ticket,
            build: b.build.clone(),
            rules_hash: b.rules_hash.clone(),
        },
    );
    net.t += LATENCY;
    assert!(
        net.drain(conn)
            .contains(&Incoming::Lobby(FromRelay::Refused(Refusal::Surrendered))),
        "too late"
    );
}

#[test]
fn a_rejoin_inside_the_grace_period_cancels_the_surrender() {
    let played = drop_and_rejoin(&Plan {
        rows: vec![SlotKind::Human, SlotKind::Human, SlotKind::Ai],
        limits: short_grace(),
        victim: 1,
        kill_at: 60,
        back_after: Some(1_200),
        ticks: 400,
        cut_back: false,
        direct: vec![],
        lag: vec![],
        gone: None,
        leaves: None,
        relink: false,
    });
    // The game ran well past the tick the house would have surrendered on.
    let p = &played.peers[0];
    let k = kept(p, 1)[0];
    assert!(p.ran() > k + 100 + 100);
}

#[test]
fn a_rejoining_peer_cut_off_before_its_readmission_can_rejoin_again() {
    drop_and_rejoin(&Plan {
        rows: vec![
            SlotKind::Human,
            SlotKind::Human,
            SlotKind::Human,
            SlotKind::Ai,
        ],
        limits: limits(),
        victim: 2,
        kill_at: 80,
        back_after: Some(1_000),
        ticks: 450,
        cut_back: true,
        direct: vec![],
        lag: vec![],
        gone: None,
        leaves: None,
        relink: false,
    });
}

#[test]
fn a_rejoin_after_the_grace_period_is_too_late() {
    // Back about 140 ticks after the cut, the house surrendered 100 ticks
    // after it: the rejoin is refused, or ended once the relay hears.
    let victim = 1;
    let played = play(&Plan {
        rows: vec![SlotKind::Human, SlotKind::Human, SlotKind::Ai],
        limits: short_grace(),
        victim,
        kill_at: 60,
        back_after: Some(2_800),
        ticks: 300,
        cut_back: false,
        direct: vec![],
        lag: vec![],
        gone: None,
        leaves: None,
        relink: false,
    });
    let p = &played.peers[0];
    clean(p);
    let x = kept(p, victim)[0] + 100;
    assert_eq!(surrenders(p, victim), [x]);
    assert!(resumed(p, victim).is_empty());
    let back = played.back.as_ref().expect("a rejoin");
    let refused = back
        .lobby
        .contains(&FromRelay::Refused(Refusal::Surrendered));
    let ended = back.closed.as_deref()
        == Some("too late: the grace period ran out and your house surrendered");
    assert!(refused || ended, "{:?} {:?}", back.lobby, back.closed);
    assert!(!back.playing());
    assert_eq!(played.net.hub.rejoining(&played.code), None);
    assert_eq!(played.net.hub.stats().readmitted, 0);
}

/// A game of `persons` persons and a computer, begun, with nothing run.
fn begun(persons: usize) -> (Net, String, Vec<(ConnId, Begin)>) {
    let mut net = Net::new(Limits::default());
    let mut rows = vec![SlotKind::Human; persons];
    rows.push(SlotKind::Ai);
    let (code, mut met) = meet(&mut net, &rows);
    met.sort_by_key(|(_, b)| b.you);
    (net, code, met)
}

fn rejoin_msg(code: &str, slot: u8, ticket: [u8; 16], build: &str) -> ToRelay {
    ToRelay::Rejoin {
        code: code.into(),
        slot,
        ticket,
        build: build.into(),
        rules_hash: "rules-1".into(),
    }
}

fn answers(net: &mut Net, conn: ConnId) -> Vec<Incoming> {
    net.t += LATENCY;
    net.drain(conn)
}

#[test]
fn a_rejoin_into_someone_elses_slot_is_refused() {
    let (mut net, code, met) = begun(3);
    let tickets: Vec<[u8; 16]> = met.iter().map(|(_, b)| b.ticket).collect();
    assert!(
        tickets.iter().all(|t| *t != [0; 16]),
        "every person has a ticket"
    );
    assert!(
        tickets[0] != tickets[1] && tickets[1] != tickets[2],
        "of its own"
    );
    net.kill(met[2].0);
    let refused = |m: &[Incoming]| {
        m.iter()
            .find_map(|i| match i {
                Incoming::Lobby(FromRelay::Refused(r)) => Some(*r),
                _ => None,
            })
            .expect("a refusal")
    };
    // Slot 2 is away: slot 1's ticket, a made-up one, a computer's slot
    // and a closed one do not open it.
    for (slot, ticket) in [
        (2u8, tickets[1]),
        (2, [9; 16]),
        (2, [0; 16]),
        (3, [0; 16]),
        (7, [0; 16]),
        (200, tickets[2]),
    ] {
        let c = net.connect();
        net.lobby(c, &rejoin_msg(&code, slot, ticket, "build-1"));
        assert_eq!(
            refused(&answers(&mut net, c)),
            Refusal::Ticket,
            "slot {slot}"
        );
    }
    // Nor does slot 2's own ticket for slot 1, which is live.
    let c = net.connect();
    net.lobby(c, &rejoin_msg(&code, 1, tickets[2], "build-1"));
    assert_eq!(refused(&answers(&mut net, c)), Refusal::Ticket);
    // The right ticket of another build is refused on the build.
    let c = net.connect();
    net.lobby(c, &rejoin_msg(&code, 2, tickets[2], "build-2"));
    assert_eq!(
        refused(&answers(&mut net, c)),
        Refusal::Mismatch(Field::Build)
    );
    // A wrong room.
    let c = net.connect();
    net.lobby(c, &rejoin_msg("ZZZZZZ", 2, tickets[2], "build-1"));
    assert_eq!(refused(&answers(&mut net, c)), Refusal::NoRoom);
    assert_eq!(net.hub.rejoining(&code), None, "no rejoin began");
    assert_eq!(net.hub.stats().rejoins, 0);
    // The slot's own ticket is taken: the lowest live slot serves it.
    let c = net.connect();
    net.lobby(c, &rejoin_msg(&code, 2, tickets[2], "build-1"));
    let got = answers(&mut net, c);
    let Some(Incoming::Lobby(FromRelay::Rejoined(b))) = got.first() else {
        panic!("rejoined: {got:?}");
    };
    assert_eq!(b, &met[2].1);
    assert_eq!(net.hub.rejoining(&code), Some(2));
    let serve = net.drain(met[0].0);
    assert!(serve.iter().any(|m| matches!(
        m,
        Incoming::Game(bytes) if wire::decode(bytes).is_ok_and(|e| e.from == wire::RELAY
            && e.body == Body::Serve { slot: 2, on: true })
    )));
    // A second rejoin while that one runs waits its turn.
    net.kill(met[1].0);
    let d = net.connect();
    net.lobby(d, &rejoin_msg(&code, 1, tickets[1], "build-1"));
    assert_eq!(refused(&answers(&mut net, d)), Refusal::Rejoining);
}

fn envelope(from: u8, body: Body) -> Vec<u8> {
    wire::encode(&Envelope { from, body })
}

fn marker_frames(from: u8, first: u32, slot: u8) -> Vec<u8> {
    envelope(
        from,
        Body::Frames(Frames {
            first,
            frames: vec![Frame {
                resume: Some(slot),
                ..Frame::default()
            }],
            ..Frames::default()
        }),
    )
}

/// The lockstep messages that reached `conn` by now.
fn games(net: &mut Net, conn: ConnId) -> Vec<Envelope> {
    answers(net, conn)
        .into_iter()
        .filter_map(|m| match m {
            Incoming::Game(b) => Some(wire::decode(&b).expect("a lockstep message")),
            _ => None,
        })
        .collect()
}

#[test]
fn forged_relay_messages_and_readmission_markers_are_refused() {
    let (mut net, code, met) = begun(3);
    let (a, b, c) = (met[0].0, met[1].0, met[2].0);
    net.kill(c);
    games(&mut net, a);
    games(&mut net, b);
    // A person cannot ask for a drop, a serve or a readmission: none is
    // passed on.
    for body in [
        Body::Dropped { slot: 1, kept: 0 },
        Body::Serve { slot: 2, on: true },
        Body::Resume { slot: 2 },
    ] {
        net.send(a, &envelope(0, body));
    }
    assert!(games(&mut net, b).is_empty());
    let s = net.hub.stats();
    assert_eq!((s.forged_drops, s.forged_rejoins), (1, 2));
    // Nor a marker nobody asked for: its sender is closed.
    net.send(b, &marker_frames(1, 0, 2));
    let got = games(&mut net, a);
    assert!(
        got.iter().all(|e| e.from == wire::RELAY),
        "not passed on: {got:?}"
    );
    assert!(
        got.iter()
            .any(|e| e.body == Body::Dropped { slot: 1, kept: 0 }),
        "{got:?}"
    );
    assert_eq!(closed(&mut net, b).as_deref(), Some(UNASKED));
    // A rejoin of slot 2, served by slot 0 (the one live): the rejoining
    // connection cannot pose as another slot or send the relay's messages,
    // and a marker from the serving peer before the rejoining peer says it
    // caught up is forged too.
    let r = net.connect();
    net.lobby(r, &rejoin_msg(&code, 2, met[2].1.ticket, "build-1"));
    assert_eq!(net.hub.rejoining(&code), Some(2));
    let _ = answers(&mut net, r);
    let _ = games(&mut net, a);
    let s0 = net.hub.stats();
    net.send(r, &envelope(0, Body::Frames(Frames::default())));
    net.send(r, &envelope(2, Body::Resume { slot: 2 }));
    assert!(games(&mut net, a).is_empty());
    let s1 = net.hub.stats();
    assert_eq!(s1.forged_senders, s0.forged_senders + 1);
    assert_eq!(s1.forged_rejoins, s0.forged_rejoins + 1);
    net.send(a, &marker_frames(0, 0, 2));
    assert_eq!(closed(&mut net, a).as_deref(), Some(UNASKED));
}

/// The hub's reason for closing a sender of a marker it did not ask for.
const UNASKED: &str = "a readmission marker nobody asked for";

/// The reason `conn` was closed with, by now.
fn closed(net: &mut Net, conn: ConnId) -> Option<String> {
    answers(net, conn).into_iter().find_map(|m| match m {
        Incoming::Closed(r) => r,
        _ => None,
    })
}

#[test]
fn only_the_asked_peer_readmits_and_only_after_the_catch_up() {
    let (mut net, code, met) = begun(3);
    let (a, b, c) = (met[0].0, met[1].0, met[2].0);
    // Every slot's frames 0 to 9 are forwarded; then slot 2 goes.
    for (conn, slot) in [(a, 0u8), (b, 1), (c, 2)] {
        net.send(
            conn,
            &envelope(
                slot,
                Body::Frames(Frames {
                    first: 0,
                    frames: vec![Frame::default(); 10],
                    ..Frames::default()
                }),
            ),
        );
    }
    net.kill(c);
    assert_eq!(net.hub.kept(&code, 2), Some(10));
    let r = net.connect();
    net.lobby(r, &rejoin_msg(&code, 2, met[2].1.ticket, "build-1"));
    assert_eq!(net.hub.rejoining(&code), Some(2));
    games(&mut net, a);
    games(&mut net, b);
    let _ = answers(&mut net, r);
    // Slot 1 was not asked: its marker closes it, and it drops.
    net.send(b, &marker_frames(1, 10, 2));
    let drops: Vec<Body> = games(&mut net, a).into_iter().map(|e| e.body).collect();
    assert!(
        drops.contains(&Body::Dropped { slot: 1, kept: 10 }),
        "{drops:?}"
    );
    // The rejoining peer says it caught up: slot 0 is asked.
    net.lobby(r, &ToRelay::CaughtUp);
    let asked: Vec<Body> = games(&mut net, a).into_iter().map(|e| e.body).collect();
    assert_eq!(asked, [Body::Resume { slot: 2 }]);
    // Its marker in frame 10 is passed on, to the rejoining peer too; the
    // slot is live again from frame 10 + max_delay + 1.
    net.send(a, &marker_frames(0, 10, 2));
    let to_r = games(&mut net, r);
    assert!(
        to_r.iter()
            .any(|e| e.from == 0 && matches!(&e.body, Body::Frames(f) if f.first == 10))
    );
    let gap = u32::from(met[0].1.params.max_delay) + 1;
    assert_eq!(net.hub.kept(&code, 2), Some(10 + gap));
    assert_eq!(net.hub.rejoining(&code), None);
    assert_eq!(net.hub.stats().readmitted, 1);
    // The same marker again, resent, is fine; a second one is not.
    net.send(a, &marker_frames(0, 10, 2));
    assert!(
        net.drain(a)
            .iter()
            .all(|m| !matches!(m, Incoming::Closed(_)))
    );
    net.send(a, &marker_frames(0, 11, 2));
    assert_eq!(closed(&mut net, a).as_deref(), Some(UNASKED));
    // The readmitted connection now plays slot 2: its frames from there on
    // are counted.
    net.send(
        r,
        &envelope(
            2,
            Body::Frames(Frames {
                first: 10 + gap,
                frames: vec![Frame::default(); 3],
                ..Frames::default()
            }),
        ),
    );
    assert_eq!(net.hub.kept(&code, 2), Some(10 + gap + 3));
}

/// Whether `p` saw the hub ask for `slot`'s frames (N7).
fn asked(p: &Peer, slot: u8) -> bool {
    p.events.contains(&Event::Closing { slot })
}

/// No peer saw two copies of a frame differ (N7).
fn one_story(p: &Peer) {
    assert!(
        !p.events
            .iter()
            .any(|e| matches!(e, Event::Conflicting { .. })),
        "slot {}",
        p.slot
    );
}

#[test]
fn a_drop_with_direct_links_counts_every_frame_a_peer_ran() {
    // Every pair of the three persons has a direct link, and the victim's
    // messages take 200 ms to the hub: when it is cut off the others have
    // run frames of it the hub never forwarded. The hub asks them first,
    // and every peer drops it at the same frame, past those.
    let victim = 2;
    let played = play(&Plan {
        rows: vec![
            SlotKind::Human,
            SlotKind::Human,
            SlotKind::Human,
            SlotKind::Ai,
        ],
        limits: limits(),
        victim,
        kill_at: 120,
        back_after: None,
        ticks: 400,
        cut_back: false,
        direct: vec![(0, 1), (0, 2), (1, 2)],
        lag: vec![(victim, 200)],
        gone: None,
        leaves: None,
        relink: false,
    });
    let (forwarded, ran) = played.at_cut;
    assert!(
        ran > forwarded,
        "the others ran its direct frames: ran {ran}, forwarded {forwarded}"
    );
    let survivors = played.survivors(victim);
    let k = kept(survivors[0], victim);
    assert_eq!(k.len(), 1);
    assert!(k[0] >= ran, "kept {} counts every frame run ({ran})", k[0]);
    for p in &survivors {
        clean(p);
        one_story(p);
        assert!(asked(p, victim), "slot {}", p.slot);
        assert_eq!(kept(p, victim), k);
        replays(p, &played.map);
        let paths = p
            .session
            .as_ref()
            .and_then(|s| s.paths(victim))
            .expect("paths");
        assert!(paths.direct_first > 0, "slot {}: {paths:?}", p.slot);
    }
    assert_eq!(agree(survivors[0], survivors[1]), 400);
    let stats = played.net.hub.stats();
    assert_eq!(stats.rounds, 1);
    assert!(stats.held_parts >= 2, "{stats:?}");
    assert_eq!((stats.held_refused, stats.unanswered), (0, 0), "{stats:?}");
    assert_eq!(played.net.hub.kept(&played.code, victim), Some(k[0]));
    // The two survivors' link stays: frames of each came on it.
    let paths = survivors[0]
        .session
        .as_ref()
        .and_then(|s| s.paths(1))
        .expect("paths");
    assert!(paths.direct_first > 0 && paths.relay_top > 0, "{paths:?}");
}

#[test]
fn a_drop_and_rejoin_over_a_direct_link_keeps_identical_hashes() {
    let played = drop_and_rejoin(&Plan {
        rows: vec![SlotKind::Human, SlotKind::Human],
        limits: limits(),
        victim: 1,
        kill_at: 90,
        back_after: Some(1_500),
        ticks: 400,
        cut_back: false,
        direct: vec![(0, 1)],
        lag: vec![(1, 200)],
        gone: None,
        leaves: None,
        relink: false,
    });
    let (forwarded, ran) = played.at_cut;
    assert!(ran > forwarded, "ran {ran}, forwarded {forwarded}");
    assert!(asked(&played.peers[0], 1));
    one_story(&played.peers[0]);
    assert_eq!(played.net.hub.stats().rounds, 1);
}

#[test]
fn a_rejoin_while_two_others_play_on_a_direct_link() {
    // Slots 1 and 2 have a direct link and slot 2's messages lag; slot 0
    // drops and rejoins, served by slot 1, whose frames of slot 2 are
    // ahead of the hub's. The game plays on well past a lead after the
    // readmission, with the link in use and nothing held back.
    let played = drop_and_rejoin(&Plan {
        rows: vec![
            SlotKind::Human,
            SlotKind::Human,
            SlotKind::Human,
            SlotKind::Ai,
        ],
        limits: limits(),
        victim: 0,
        kill_at: 100,
        back_after: Some(1_500),
        ticks: 600,
        cut_back: false,
        direct: vec![(1, 2)],
        lag: vec![(2, 150)],
        gone: None,
        leaves: None,
        relink: false,
    });
    let stats = played.net.hub.stats();
    assert_eq!(stats.rounds, 0, "slot 0 had no direct link: {stats:?}");
    for p in played.survivors(0) {
        one_story(p);
    }
    let p1 = &played.peers[1];
    let paths = p1.session.as_ref().and_then(|s| s.paths(2)).expect("paths");
    assert!(paths.direct_first > 400, "{paths:?}");
    let back = played.back.as_ref().expect("the rejoined peer");
    one_story(back);
    let theirs = back
        .session
        .as_ref()
        .and_then(|s| s.paths(2))
        .expect("paths");
    assert_eq!(theirs.gated, 0, "{theirs:?}");
}

/// A game with a lone stretch (the M9 exit run's second game): the victim
/// drops and rejoins while the slot `plan.leaves` names plays alone, the
/// rejoined peer plays on, and then that slot leaves for good. Every peer
/// still in the game takes its drop at the same frame, with no bad
/// message, and its house surrenders on the same tick everywhere, the
/// grace period after that frame. The peers left, the rejoined one first.
fn after_a_lone_stretch(plan: &Plan) -> Played {
    let played = play(plan);
    let (l, _) = plan.leaves.expect("a slot that leaves");
    let v = plan.victim;
    let back = played.back.as_ref().expect("a rejoin");
    assert!(back.live() && back.playing(), "{:?}", back.closed);
    let left: Vec<&Peer> = [back]
        .into_iter()
        .chain(played.peers.iter().filter(|p| p.live()))
        .collect();
    for p in left.iter().copied().chain(played.peers.iter()) {
        clean(p);
    }
    let grace = back.begin.params.grace_ticks;
    assert_eq!(grace, 100);
    // The rejoin came after a stretch in which slot `l` played alone.
    let from = resumed(back, v);
    assert_eq!(from.len(), 1);
    assert!(from[0] > kept(&played.peers[usize::from(l)], v)[0] + 50);
    let k = kept(back, l);
    assert_eq!(k.len(), 1, "one drop of slot {l}: {k:?}");
    assert!(
        k[0] > from[0],
        "dropped after the readmission: {k:?} {from:?}"
    );
    assert_eq!(played.net.hub.kept(&played.code, l), Some(k[0]));
    let x = k[0] + grace;
    for p in &left {
        assert_eq!(kept(p, l), k, "slot {}", p.slot);
        assert_eq!(surrenders(p, l), [x], "slot {}", p.slot);
        let applied = p.ticks[&x]
            .orders
            .iter()
            .find(|a| a.slot == l)
            .expect("the surrender");
        assert!(applied.taken, "the simulation takes the surrender");
        assert!(p.events.contains(&Event::Surrendered { slot: l, tick: x }));
    }
    for p in played.survivors(v) {
        replays(p, &played.map);
        agree(p, back);
    }
    played
}

#[test]
fn a_peer_that_played_alone_leaves_and_its_house_surrenders_everywhere() {
    // Slot 1 is cut off at tick 60 and back 1.5 s later, about 75 ticks
    // that slot 0 plays alone; then slot 0 leaves.
    after_a_lone_stretch(&Plan {
        rows: vec![SlotKind::Human, SlotKind::Human],
        limits: short_grace(),
        victim: 1,
        kill_at: 60,
        back_after: Some(1_500),
        ticks: 450,
        cut_back: false,
        direct: vec![],
        lag: vec![],
        gone: None,
        leaves: Some((0, 250)),
        relink: false,
    });
}

#[test]
fn a_peer_left_alone_in_a_game_of_three_leaves_and_its_house_surrenders() {
    // Slot 2 leaves for good at tick 30 (its house surrenders); slot 1 is
    // cut off at tick 160 and back 1.5 s later, while slot 0 plays alone;
    // then slot 0 leaves.
    let played = after_a_lone_stretch(&Plan {
        rows: vec![SlotKind::Human, SlotKind::Human, SlotKind::Human],
        limits: short_grace(),
        victim: 1,
        kill_at: 160,
        back_after: Some(1_500),
        ticks: 550,
        cut_back: false,
        direct: vec![],
        lag: vec![],
        gone: Some((2, 30)),
        leaves: Some((0, 330)),
        relink: false,
    });
    let host = &played.peers[0];
    let k2 = kept(host, 2);
    assert_eq!(k2.len(), 1);
    assert_eq!(surrenders(host, 2), [k2[0] + 100]);
}

#[test]
fn a_peer_that_played_alone_and_relinked_leaves_through_a_drop_round() {
    // As the first, with a direct link between the two that the rejoined
    // peer sets up again, and slot 0's messages lagging: slot 0's drop
    // goes through the hub's round (N7), which starts from the frames the
    // hub forwarded of it.
    let played = after_a_lone_stretch(&Plan {
        rows: vec![SlotKind::Human, SlotKind::Human],
        limits: short_grace(),
        victim: 1,
        kill_at: 60,
        back_after: Some(1_500),
        ticks: 450,
        cut_back: false,
        direct: vec![(0, 1)],
        lag: vec![(0, 200)],
        gone: None,
        leaves: Some((0, 250)),
        relink: true,
    });
    let back = played.back.as_ref().expect("the rejoined peer");
    assert!(asked(back, 0), "the hub asked the rejoined peer");
    one_story(back);
    let stats = played.net.hub.stats();
    assert_eq!(stats.rounds, 2, "{stats:?}");
    assert_eq!((stats.held_refused, stats.unanswered), (0, 0), "{stats:?}");
}
