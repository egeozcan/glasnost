//! Claim tests for S-0501 r134 (Q-2646 to Q-2650, M8 intake 104): claim 7's
//! landing check at the update's landing step, its free cell, the touchdown
//! in the refusing update and the demolition hit where no cell is found;
//! every touchdown setting both speed fractions to 0; claim 6's aircraft on
//! the ground given no destination; claim 47's listing, object-present flag
//! and tracking of an aircraft that slides after an untested touchdown
//! (`quirks.aircraft_corner_sector_untracked`); claim 24's entry restarted
//! in the approaching stage; claim 26's release before the skip. Synthetic
//! data through the real tick path; every type, weapon, map and number is
//! invented. Ledger: `docs/m8-claims/intake104.md`.

mod common;
mod econ_common;

use econ_common::{SIZE, building, factory, game, issue, ready};
use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, Locomotor, UnitType, WarheadType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::combat::flying;
use glasnost_sim::econ::release::Exit;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

/// AC31 part h (1)'s displacement (test value): from the 2 × 2 foundation's
/// centre to the centre of the cell two cells east of its east column, in
/// its south row, at height 0.
const OFF_PAD: [i32; 3] = [640, 128, 0];

/// The ruleset: the pad APAD (an aircraft factory, helipad and reload pad,
/// one docking place at `off`, a 2 × 2 foundation); JET (armed, landable,
/// not pad-bound, 2 rounds, docking at APAD); BOUND (as JET, pad-bound);
/// TANK, a ground vehicle; DEMO, the demolition warhead, whose verses are
/// all `verses`. `untested` and `drop` set claim 7's switches.
fn rules(untested: bool, drop: bool, verses: &str, off: [i32; 3]) -> Ruleset {
    let mut r = econ_common::ruleset();
    let c = testkit::combat_ruleset();
    r.sim.weapons.extend(c.sim.weapons.clone());
    r.sim.projectiles.extend(c.sim.projectiles.clone());
    r.sim.warheads.extend(c.sim.warheads.clone());
    push(&mut r, "warheads", "DEMO");
    r.sim.warheads.insert(
        "DEMO".into(),
        WarheadType {
            verses: Some(vec![Decimal(verses.into()); 11]),
            ..Default::default()
        },
    );
    r.sim.general.c4_warhead = Some("DEMO".into());
    push(&mut r, "buildings", "APAD");
    let mut pad = factory("aircraft");
    pad.helipad = Some(true);
    pad.econ.unit_reload = Some(true);
    pad.econ.number_of_docks = Some(1);
    pad.econ.docking_offsets = Some(vec![off]);
    r.sim.buildings.insert("APAD".into(), pad);
    r.sim.weapons.insert(
        "Agm".into(),
        WeaponType {
            damage: Some(40),
            rof: Some(30),
            range: econ_common::dec("6"),
            speed: Some(60),
            projectile: Some("Rocket".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    let jet = || {
        let mut u = UnitType {
            strength: Some(200),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(Locomotor::Fly),
            tech_level: Some(1),
            owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
            cost: Some(1000),
            primary: Some("Agm".into()),
            ..Default::default()
        };
        u.combat.ammo = Some(2);
        u.econ.landable = Some(true);
        u.econ.dock = Some(vec!["APAD".into()]);
        u
    };
    push(&mut r, "aircraft", "JET");
    r.sim.aircraft.insert("JET".into(), jet());
    let mut bound = jet();
    bound.econ.airport_bound = Some(true);
    push(&mut r, "aircraft", "BOUND");
    r.sim.aircraft.insert("BOUND".into(), bound);
    r.globals.quirks.aircraft_descent_touchdown_untested = untested;
    r.globals.quirks.aircraft_refused_kept_drop = drop;
    r
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn st(w: &World, id: ObjectId) -> air::AirState {
    w.obj(id).unwrap().air.clone().unwrap_or_default()
}

fn describe(w: &World, id: ObjectId) -> String {
    let o = w.obj(id).unwrap();
    format!(
        "t={} pos={:?} h={} hp={} beh={:?}/{:?} last={:?} dest={:?} contact={:?} tracked={} air={:?}",
        w.tick,
        o.pos,
        air::height(w, id),
        o.health,
        o.beh.current,
        o.beh.queued,
        o.beh.last,
        o.dest,
        o.contact,
        flying::tracked(w, id),
        o.air
    )
}

fn foundation(w: &World, k: ObjectId) -> Vec<CellXY> {
    let b = w.obj(k).unwrap();
    glasnost_sim::econ::placement::foundation_cells(w, b.type_index, b.pos.cell())
}

fn cruise(w: &World, a: ObjectId) -> i32 {
    air::cruise_height(w, a)
}

fn zero(f: F64) -> bool {
    f.eq_num(F64::ZERO)
}

fn one(w: &World) -> F64 {
    F64::from_i64(1, w.rules().mode)
}

/// Whether cell `c` is cell `d` or one of its eight neighbours.
fn near(c: CellXY, d: CellXY) -> bool {
    (c.x - d.x).abs() <= 1 && (c.y - d.y).abs() <= 1
}

/// Steps until the next processing of `a`; returns its tick and delay.
fn next_processing(w: &mut World, a: ObjectId) -> (i32, i32) {
    let before = w.obj(a).unwrap().beh.last;
    for _ in 0..50 {
        step(w);
        let last = w.obj(a).unwrap().beh.last;
        if last != before {
            return last.unwrap();
        }
    }
    panic!("no processing: {}", describe(w, a));
}

/// A `name` aircraft of house 0 in contact with the pad, landing by hand
/// toward it from `dz` leptons above the centre of cell *X* (`dx`, `dy`
/// cells from the pad's first foundation cell), in *sleep*. The pad's
/// docking position lies two cells east of the foundation, so its cell
/// holds no building: claim 7's check refuses it to a pad-bound aircraft.
/// `tank`: a TANK stands in *X*. Returns (world, pad, aircraft, *X*).
fn landing_by_hand(
    r: &Ruleset,
    name: &str,
    dx: i32,
    dy: i32,
    dz: i32,
    tank: bool,
) -> (World, ObjectId, ObjectId, CellXY) {
    let mut w = game(r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = testkit::spawn(&mut w, Kind::Aircraft, name, 0, testkit::mid(SIZE, 8, 0), 0);
    step(&mut w);
    let f0 = foundation(&w, k)[0];
    let x = CellXY::new(f0.x + dx, f0.y + dy);
    if tank {
        testkit::spawn(&mut w, Kind::Vehicle, "TANK", 0, x, 0);
    }
    // Set-up: lifted off the ground first, so that its listing and flag go.
    w.obj_mut(a).unwrap().pos.z += 200;
    air::sync_listing(&mut w, a);
    air::clear_mark(&mut w, a);
    let centre = Pos::centre(x, 0);
    let z = w.surface_at(centre.x, centre.y) + dz;
    let t = w.tick;
    {
        let o = w.obj_mut(a).unwrap();
        o.pos = Pos::new(centre.x, centre.y, z);
        o.beh.force(Behaviour::Sleep, t, true);
        o.dest = Some(Target::Object(k));
        o.contact = Some(k);
        o.econ.exit = Some(Exit {
            factory: k,
            tethered: false,
            place: 0,
        });
        let s = o.air.as_mut().unwrap();
        s.ground = false;
        s.landing = true;
        s.taking_off = false;
        s.held = 0;
        s.flying_to = Some(Target::Object(k));
        s.own_point = false;
    }
    air::sync_listing(&mut w, a);
    (w, k, a, x)
}

// ---------------------------------------------------------------------------
// Claim 7: the landing check's refusal (Q-2646)

// spec: S-0501 claim 7 (r134, Q-2646: when a cell is found and the refusing update leaves the aircraft at or below the landing's held height, that update's touchdown still follows the check; it makes no cell test with quirks.aircraft_descent_touchdown_untested on or off, and its forget test is made against the new flight point; one it does not make forget is on the ground with its destination and that point, and climbs from its next update with no slide first, since its take-off gave it the held height of ordinary flight; every touchdown sets both speed fractions to 0) and claim 47 (b) (that touchdown ends its tracking and returns its listing, which its next update's climb ends) and Edge cases (quirks.aircraft_descent_touchdown_untested and quirks.aircraft_refused_kept_drop, each in both positions: neither tests that touchdown nor gives it a slide)
#[test]
fn claim_7_the_touchdown_in_a_refusing_update_is_untested_and_climbs_with_no_slide() {
    for untested in [true, false] {
        for drop in [true, false] {
            let r = rules(untested, drop, "1", OFF_PAD);
            // Over a cell a TANK holds, which F7 would refuse at a tested
            // touchdown, three cells west of the foundation.
            let (mut w, k, a, x) = landing_by_hand(&r, "BOUND", -3, 0, 1, true);
            step(&mut w);
            let o = w.obj(a).unwrap();
            let s = st(&w, a);
            let tag = format!("untested {untested}, drop {drop}: {}", describe(&w, a));
            assert_eq!(o.pos.cell(), x, "{tag}");
            assert_eq!(air::height(&w, a), 0, "no rise of 10 leptons: {tag}");
            assert!(
                !s.landing && !s.taking_off && !s.ground && !s.sliding,
                "on the ground by its height, not at rest, no slide: {tag}"
            );
            assert_eq!(s.held, cruise(&w, a), "the take-off's held height: {tag}");
            assert!(zero(s.cur) && zero(s.tgt), "both fractions 0: {tag}");
            assert_eq!(
                o.dest,
                Some(Target::Object(k)),
                "the destination kept: {tag}"
            );
            let Some(Target::Cell(c)) = s.flying_to else {
                panic!("a free cell as its flight point: {tag}");
            };
            assert!(c != x && !foundation(&w, k).contains(&c), "{tag}");
            assert!(
                !flying::tracked(&w, a),
                "the touchdown ends tracking: {tag}"
            );
            assert!(w.cells.list(x, Layer::Ground).contains(&a), "listed: {tag}");
            assert_eq!(s.mark, Some(x), "its flag: {tag}");
            step(&mut w);
            let s = st(&w, a);
            assert!(
                air::height(&w, a) > 0,
                "it climbs at once: {}",
                describe(&w, a)
            );
            assert!(!s.sliding && s.mark.is_none(), "{}", describe(&w, a));
            assert!(!w.cells.list(x, Layer::Ground).contains(&a));
        }
    }
}

// spec: S-0501 claim 7 (r134, Q-2646: the free cell is found by S-0212 claim 25's nearby search around the cell the aircraft is over, so over a free cell it is that cell; the touchdown in the refusing update is then in its flight point's cell, so its forget test forgets the destination and the flight point, and it rests there) and Edge cases (quirks.aircraft_descent_touchdown_untested in both positions)
#[test]
fn claim_7_a_refusing_update_over_a_free_cell_rests_there() {
    for untested in [true, false] {
        let r = rules(untested, true, "1", OFF_PAD);
        let (mut w, _, a, x) = landing_by_hand(&r, "BOUND", -3, 0, 1, false);
        step(&mut w);
        let o = w.obj(a).unwrap();
        let s = st(&w, a);
        let tag = format!("untested {untested}: {}", describe(&w, a));
        assert_eq!(o.pos.cell(), x, "{tag}");
        assert!(s.ground && !s.landing && !s.taking_off, "at rest: {tag}");
        assert_eq!(o.dest, None, "forgotten: {tag}");
        assert_eq!(s.flying_to, None, "{tag}");
        assert_eq!(s.mark, Some(x), "{tag}");
        assert!(zero(s.cur) && zero(s.tgt), "{tag}");
        assert!(!flying::tracked(&w, a), "{tag}");
    }
}

// spec: S-0501 claim 7 (r134, Q-2646: a refusal above the landing's held height ends the take-off in that update at its take-off step, the climb coming from the next update; the aircraft keeps its destination and flies to the free cell's centre, its flight point, in ordinary flight) and claim 47 (a) (the refusal's take-off starts its tracking when it is not tracked already)
#[test]
fn claim_7_a_refusal_in_flight_flies_on_to_the_free_cell() {
    let r = rules(true, true, "1", OFF_PAD);
    let (mut w, k, a, _) = landing_by_hand(&r, "BOUND", 0, 0, 200, false);
    let h = air::height(&w, a);
    step(&mut w);
    let s = st(&w, a);
    assert!(
        !s.landing && !s.taking_off && s.held == cruise(&w, a),
        "{}",
        describe(&w, a)
    );
    let h1 = air::height(&w, a);
    assert!(h1 < h, "the update's descent came before the check");
    assert!(flying::tracked(&w, a), "{}", describe(&w, a));
    assert_eq!(w.obj(a).unwrap().dest, Some(Target::Object(k)));
    let Some(Target::Cell(c)) = s.flying_to else {
        panic!("{}", describe(&w, a));
    };
    assert!(!foundation(&w, k).contains(&c), "never a building's cell");
    step(&mut w);
    assert!(air::height(&w, a) > h1, "it climbs from the next update");
    assert_eq!(st(&w, a).flying_to, Some(Target::Cell(c)), "the point kept");
}

// spec: S-0501 claim 7 (r134, Q-2646: when no free cell is found, the aircraft takes a hit of its current health with the demolition warhead, ignoring its defences, with no attacker and no house, S-0302 claim 1 deciding its outcome; no touchdown follows in that update; r146, Q-2676: its flight point becomes the map's origin point, which a destroyed aircraft holds too, its flight unfinished) and Import mapping (C4Warhead); S-0302 F3 case (i) (a hit that ignores defences: d₀, the warhead's verses not applied, so the hit of its current health destroys it whatever the verses, unless a stage 2 row of S-0302 claim 8 stops it, which these rules give no row for; the survivor is `claims_s0501_r146`'s)
#[test]
fn claim_7_with_no_free_cell_it_takes_the_demolition_hit() {
    for verses in ["1", "0.5"] {
        let r = rules(true, true, verses, OFF_PAD);
        let (mut w, _, a, _) = landing_by_hand(&r, "BOUND", 0, 0, 200, false);
        // Set-up: every cell's ground record holds an object, so the
        // nearby search finds no free cell.
        let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
        for c in cells {
            if let Some(rec) = w.cells.occ_mut(c, Layer::Ground) {
                rec.others = rec.others.saturating_add(1);
            }
        }
        step(&mut w);
        let tag = format!("verses {verses}: {}", describe(&w, a));
        let o = w.obj(a).unwrap();
        let s = st(&w, a);
        assert!(!s.landing && !s.ground, "no touchdown: {tag}");
        assert_eq!(
            s.flying_to, None,
            "no flight point but the origin point: {tag}"
        );
        assert!(s.origin, "the origin point: {tag}");
        assert_eq!(o.health, 0, "{tag}");
        assert!(glasnost_sim::combat::damage::dead(o), "{tag}");
    }
}

// ---------------------------------------------------------------------------
// The untested touchdown that kept its flight point (Q-2647, Q-2648)

/// A JET of house 0 flying east at full speed in flight toward a cell 12
/// cells east, a pad APAD of house 0 two cells west and eight south of the
/// middle. A stop order, then, at the landing's start, the pad given by
/// hand as destination and flight point, in *sleep*, with no contact: the
/// pad's cell holds a building that is not its contact, so the touchdown
/// outside that cell forgets nothing (claim 7). Runs to the end of the
/// touchdown's update. Returns (world, pad, JET, the cell it touched down
/// in).
fn kept_touchdown(r: &Ruleset) -> (World, ObjectId, ObjectId, CellXY) {
    let mut w = game(r);
    let k = building(&mut w, "APAD", 0, -2, 8);
    ready(&mut w);
    let a = testkit::spawn(
        &mut w,
        Kind::Aircraft,
        "JET",
        0,
        testkit::mid(SIZE, -8, 0),
        0,
    );
    step(&mut w);
    let c = w.obj(a).unwrap().pos.cell();
    assert!(issue(
        &mut w,
        0,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(CellXY::new(c.x + 12, c.y))),
        }
    ));
    let mut cruising = false;
    for _ in 0..600 {
        step(&mut w);
        let s = st(&w, a);
        let h = air::height(&w, a);
        if h > 0 && !s.taking_off && h == s.held && s.cur.eq_num(one(&w)) {
            cruising = true;
            break;
        }
    }
    assert!(cruising, "{}", describe(&w, a));
    assert!(issue(&mut w, 0, Order::Stop { actor: a }));
    let mut landing = false;
    for _ in 0..800 {
        step(&mut w);
        if st(&w, a).landing {
            landing = true;
            break;
        }
    }
    assert!(landing, "{}", describe(&w, a));
    let t = w.tick;
    {
        let o = w.obj_mut(a).unwrap();
        o.beh.force(Behaviour::Sleep, t, true);
        o.dest = Some(Target::Object(k));
        let s = o.air.as_mut().unwrap();
        s.flying_to = Some(Target::Object(k));
        s.own_point = false;
    }
    for _ in 0..800 {
        let was = st(&w, a);
        step(&mut w);
        if was.landing && !st(&w, a).landing && air::height(&w, a) <= 0 {
            let s = st(&w, a);
            assert!(!s.ground, "kept: {}", describe(&w, a));
            assert_eq!(w.obj(a).unwrap().dest, Some(Target::Object(k)));
            let x = w.obj(a).unwrap().pos.cell();
            return (w, k, a, x);
        }
    }
    panic!("no touchdown: {}", describe(&w, a));
}

// spec: S-0501 claim 7 (r134: every touchdown, tested or not, sets the current and target speed fractions to 0; the untested one that kept the flight point too) and claim 47 (b) (r134, Q-2647: its touchdown ends its tracking and returns its listing, setting its cell's object-present flag; while it slides at height 0 each movement update takes it out of the cell layer of the cell it was in, clearing that cell's flag, and lists it in that of the cell it ends the update in, at the front, setting that cell's flag; the listing ends in the first update whose climb leaves its height positive) and claim 47 (c) (r134: the touchdown's fraction of 0 means no sector update in its next update)
#[test]
fn claim_47_a_sliding_aircraft_is_listed_and_flagged_until_it_climbs() {
    let r = rules(true, true, "1", [0, 0, 0]);
    let (mut w, _, a, x) = kept_touchdown(&r);
    let s = st(&w, a);
    assert!(s.sliding, "{}", describe(&w, a));
    assert!(zero(s.cur) && zero(s.tgt), "{}", describe(&w, a));
    assert_eq!(s.mark, Some(x));
    assert!(w.cells.list(x, Layer::Ground).contains(&a));
    assert!(!flying::tracked(&w, a));
    let mut updates = 0;
    let mut was_cell = x;
    loop {
        step(&mut w);
        updates += 1;
        assert!(updates < 300, "it never climbs: {}", describe(&w, a));
        let c = w.obj(a).unwrap().pos.cell();
        let s = st(&w, a);
        if air::height(&w, a) > 0 {
            assert_eq!(
                s.mark,
                None,
                "the climb clears its flag: {}",
                describe(&w, a)
            );
            assert!(!w.cells.list(c, Layer::Ground).contains(&a), "unlisted");
            assert!(!w.cells.list(was_cell, Layer::Ground).contains(&a));
            break;
        }
        assert_eq!(
            s.mark,
            Some(c),
            "its flag in the cell it ends in: {}",
            describe(&w, a)
        );
        assert_eq!(
            w.cells.list(c, Layer::Ground).first(),
            Some(&a),
            "listed at the front: {}",
            describe(&w, a)
        );
        if c != was_cell {
            assert!(!w.cells.list(was_cell, Layer::Ground).contains(&a));
            assert_eq!(
                w.cells.occ(was_cell, Layer::Ground).unwrap().others,
                0,
                "the old cell's flag cleared"
            );
        }
        was_cell = c;
        if updates == 1 {
            assert!(!flying::tracked(&w, a), "no sector update at fraction 0");
        }
    }
}

// spec: S-0501 claim 47 (c) (r134, Q-2647: an aircraft that an untested touchdown left holding its flight point is tracked again by (c)'s rule for an untracked aircraft, on the ground as in flight, at its first movement update with a positive carried fraction; with its cell in sector (0, 0) of S-0303 F7 it joins no list) and Edge cases (quirks.aircraft_corner_sector_untracked on; off: it joins sector (0, 0)'s list like any other)
#[test]
fn claim_47_the_corner_switch_applies_to_a_sliding_aircraft() {
    for on in [true, false] {
        let mut r = rules(true, true, "1", [0, 0, 0]);
        r.globals.quirks.aircraft_corner_sector_untracked = on;
        // Set-up: no cell of the 30 × 30 map lies in sector (0, 0), so the
        // slide is set by hand on a map of 10 × 10 cells, B = 19, sector
        // size 0, every cell in sector (0, 0) (S-0303 F7 and Edge cases): a
        // JET at height 0 in the middle, untracked and listed, sliding with
        // a flight point of its own three cells east and a positive carried
        // fraction, as an untested touchdown that kept its flight point
        // leaves it (claim 7, r132 and r134).
        let mut w = testkit::game_with(&r, 10, 7);
        let c = testkit::mid(10, 0, 0);
        assert_eq!(flying::sector(&w, c), (0, 0));
        let a = testkit::spawn(&mut w, Kind::Aircraft, "JET", 0, c, 0);
        let p = Pos::centre(c, 0);
        let z = w.surface_at(p.x, p.y);
        let half = w.rules().air.half;
        {
            let o = w.obj_mut(a).unwrap();
            o.pos = Pos::new(p.x, p.y, z);
            let s = o.air.get_or_insert_with(Default::default);
            s.landing = false;
            s.taking_off = false;
            s.ground = false;
            s.held = 0;
            s.sliding = true;
            s.flying_to = Some(Target::Cell(CellXY::new(c.x + 3, c.y)));
            s.own_point = true;
            s.unfinished = true;
            s.cur = half;
            s.tgt = half;
        }
        flying::unregister(&mut w, a);
        air::sync_listing(&mut w, a);
        assert!(!flying::tracked(&w, a));
        step(&mut w);
        assert_eq!(air::height(&w, a), 0, "{}", describe(&w, a));
        assert_eq!(
            flying::tracked(&w, a),
            !on,
            "switch {on}: {}",
            describe(&w, a)
        );
        // Away from the corner the same aircraft is tracked again at once.
        if on {
            let (mut v, _, b, _) = kept_touchdown(&r);
            let half = v.rules().air.half;
            v.obj_mut(b).unwrap().air.as_mut().unwrap().cur = half;
            let c = v.obj(b).unwrap().pos.cell();
            assert_ne!(flying::sector(&v, c), (0, 0));
            step(&mut v);
            assert!(flying::tracked(&v, b), "{}", describe(&v, b));
        }
    }
}

// spec: S-0501 claim 6 (r134, Q-2647: an aircraft on the ground by its height that is moving, as one sliding on after an untested touchdown that kept its flight point, given no destination gets the new flight point like one in flight; being on the ground and not landing it takes off toward that point, claim 8, which starts its tracking, claim 47 (a), and lands there, so its slide does not end where it was given none)
#[test]
fn claim_6_a_sliding_aircraft_given_no_destination_takes_off() {
    let r = rules(true, true, "1", [0, 0, 0]);
    let (mut w, _, a, _) = kept_touchdown(&r);
    step(&mut w);
    assert!(
        st(&w, a).sliding && air::height(&w, a) == 0,
        "{}",
        describe(&w, a)
    );
    glasnost_sim::econ::assign_destination(&mut w, a, None);
    let s = st(&w, a);
    assert_eq!(w.obj(a).unwrap().dest, None);
    let Some(Target::Cell(c)) = s.flying_to else {
        panic!("a new flight point: {}", describe(&w, a));
    };
    assert!(s.own_point && s.unfinished, "{}", describe(&w, a));
    assert!(
        s.taking_off && !s.sliding && !s.ground && s.held == cruise(&w, a),
        "a take-off: {}",
        describe(&w, a)
    );
    assert!(flying::tracked(&w, a), "its take-off tracks it");
    let mut started = None;
    for _ in 0..800 {
        let was = st(&w, a);
        step(&mut w);
        let s = st(&w, a);
        if !was.landing && s.landing && started.is_none() {
            started = Some(w.obj(a).unwrap().pos.cell());
        }
        if s.ground {
            break;
        }
    }
    let at = started.expect("a landing");
    assert!(near(at, c), "it lands there: {at:?}, {c:?}");
    let s = st(&w, a);
    assert!(s.ground, "{}", describe(&w, a));
    assert_eq!(s.flying_to, None);
}

// ---------------------------------------------------------------------------
// Claim 24: the entry restarted in the approaching stage (Q-2650); claim 26
// (Q-2649)

// spec: S-0501 claim 24 (r134, Q-2650: a queued behaviour is replaced by enter and lost, and entry starts again within that processing, in the approaching stage as in the docking stage: it makes entry's beginning, with its 1-tick delay, puts the aircraft in the approaching stage and leaves it ready to change, and no rule of the stage it was in follows in that processing)
#[test]
fn claim_24_an_entry_restarted_in_the_approaching_stage_makes_its_beginning_at_once() {
    let r = rules(true, true, "1", OFF_PAD);
    let mut w = game(&r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = testkit::spawn(
        &mut w,
        Kind::Aircraft,
        "JET",
        0,
        testkit::mid(SIZE, 8, 0),
        0,
    );
    step(&mut w);
    w.obj_mut(a).unwrap().combat.ammo = 0;
    glasnost_sim::process::fall_back(&mut w, a, false);
    assert_eq!(w.obj(a).unwrap().beh.current, Some(Behaviour::Enter));
    next_processing(&mut w, a);
    next_processing(&mut w, a);
    assert_eq!(w.obj(a).unwrap().contact, Some(k), "{}", describe(&w, a));
    assert!(!st(&w, a).enter_docking, "approaching: {}", describe(&w, a));
    // Set-up: in flight with its destination gone, so that the approaching
    // stage asks the pad again, and a behaviour queued while it is not
    // ready to change.
    let start0 = w.obj(a).unwrap().beh.start_tick;
    {
        let o = w.obj_mut(a).unwrap();
        o.dest = None;
        o.beh.queued = Some(Behaviour::Guard);
        o.air.as_mut().unwrap().mark_cleared = true;
    }
    let (t0, d) = next_processing(&mut w, a);
    assert_eq!(d, 1, "entry's beginning: 1 tick: {}", describe(&w, a));
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Enter), "{}", describe(&w, a));
    assert_ne!(o.beh.start_tick, start0, "entry started again");
    assert_eq!(o.beh.queued, None, "the queued behaviour replaced");
    assert_eq!(o.dest, Some(Target::Object(k)), "the pad given back");
    let s = st(&w, a);
    assert!(!s.enter_docking, "in the approaching stage");
    assert_eq!(s.enter_of, Some(o.beh.start_tick), "the beginning made");
    assert!(!s.mark_cleared, "left ready to change");
    let (t1, d1) = next_processing(&mut w, a);
    assert_eq!(t1, t0 + 1, "its next processing 1 tick later");
    assert_eq!(d1, 3, "the approaching stage's processing in contact");
}

// spec: S-0501 claim 26 (r134, Q-2649: in each service step an aircraft ready for take-off, no target and full ammunition, with full health, is released before the skip of one still flying in or moving: so one that slides after its report, not settled, is released first; the release is the fall-back to rest with guard forced; one not ready is skipped)
#[test]
fn claim_26_the_release_comes_before_the_skip() {
    for full in [true, false] {
        let r = rules(true, true, "1", [0, 0, 0]);
        let (mut w, k, a, _) = kept_touchdown(&r);
        // Set-up: in contact with the pad while it slides, in guard started
        // earlier, the pad's service step due now.
        let t = w.tick;
        {
            let o = w.obj_mut(a).unwrap();
            o.contact = Some(k);
            o.econ.exit = Some(Exit {
                factory: k,
                tethered: false,
                place: 0,
            });
            o.beh.force(Behaviour::Guard, t - 5, true);
            if !full {
                o.combat.ammo = 0;
            }
        }
        w.obj_mut(k).unwrap().air.get_or_insert_default().service = Some(t);
        assert!(st(&w, a).sliding);
        air::pad_turn(&mut w, k);
        let o = w.obj(a).unwrap();
        assert_eq!(o.beh.current, Some(Behaviour::Guard), "{}", describe(&w, a));
        assert_eq!(o.beh.queued, None, "no sleep queued: {}", describe(&w, a));
        if full {
            assert_eq!(
                o.beh.start_tick,
                t,
                "released: guard forced: {}",
                describe(&w, a)
            );
        } else {
            assert_eq!(o.beh.start_tick, t - 5, "skipped: {}", describe(&w, a));
            assert_eq!(o.dest, Some(Target::Object(k)));
        }
    }
}
