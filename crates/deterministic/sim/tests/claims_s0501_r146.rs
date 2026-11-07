//! Claim tests for S-0501 r146 (Q-2676, Q-2677, M8 intake 108): claim 7's
//! survivor of the hit a refused landing check gives where no free cell is
//! found, which holds the map's origin point as flight point, in each of
//! claim 8's height bands, its kept destination, the order that ends it, the
//! pad that gives a destination back, and its flight off the legal map
//! (`quirks.aircraft_edge_hold`); claim 33's *move* at the movement update,
//! on the ground included; claim 24's slide after an entry report (AC31 part
//! h (6), `quirks.aircraft_grounded_enter_stall`) and the release in the
//! report's tick. Synthetic data through the real tick path; every type,
//! weapon, map and number is invented. Ledger: `docs/m8-claims/intake108.md`.

mod common;
mod econ_common;

use econ_common::{SIZE, building, factory, game, issue, ready};
use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, Locomotor, UnitType, WarheadType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::econ::release::Exit;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};
use glasnost_sim::{save, testkit};

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
/// BACK (as BOUND, with the fly-back flag); TANK, a ground vehicle; DEMO,
/// the demolition warhead. `immune`: DEMO is a poison warhead and every
/// aircraft type is immune to poison, so S-0302 claim 8's stage 2 row stops
/// the hit, which ignores defences (S-0302 claim 9): it survives.
fn rules(immune: bool, off: [i32; 3]) -> Ruleset {
    let mut r = econ_common::ruleset();
    let c = testkit::combat_ruleset();
    r.sim.weapons.extend(c.sim.weapons.clone());
    r.sim.projectiles.extend(c.sim.projectiles.clone());
    r.sim.warheads.extend(c.sim.warheads.clone());
    push(&mut r, "warheads", "DEMO");
    r.sim.warheads.insert(
        "DEMO".into(),
        WarheadType {
            verses: Some(vec![Decimal("1".into()); 11]),
            poison: Some(immune),
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
        u.combat.immune_to_poison = Some(immune);
        u.econ.landable = Some(true);
        u.econ.dock = Some(vec!["APAD".into()]);
        u
    };
    push(&mut r, "aircraft", "JET");
    r.sim.aircraft.insert("JET".into(), jet());
    let mut bound = jet();
    bound.econ.airport_bound = Some(true);
    push(&mut r, "aircraft", "BOUND");
    r.sim.aircraft.insert("BOUND".into(), bound.clone());
    bound.econ.fly_back = Some(true);
    push(&mut r, "aircraft", "BACK");
    r.sim.aircraft.insert("BACK".into(), bound);
    r.globals.quirks.aircraft_descent_touchdown_untested = true;
    r.globals.quirks.aircraft_refused_kept_drop = true;
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
        "t={} pos={:?} h={} hp={} ammo={} beh={:?}/{:?} last={:?} dest={:?} contact={:?} air={:?}",
        w.tick,
        o.pos,
        air::height(w, id),
        o.health,
        o.combat.ammo,
        o.beh.current,
        o.beh.queued,
        o.beh.last,
        o.dest,
        o.contact,
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

fn service(w: &World, k: ObjectId) -> Option<i32> {
    w.obj(k).unwrap().air.as_ref().and_then(|s| s.service)
}

/// The planar distance of `p` from the map's origin point (test measure:
/// the larger coordinate magnitude, enough to see it fall).
fn from_origin(p: Pos) -> i32 {
    p.x.abs().max(p.y.abs())
}

/// Claim 8's height band at height `h` for cruise height `c`: 2 above the
/// upper threshold c − ⌊c / 3⌋, 1 above ⌊c / 2⌋ up to it, 0 lower.
fn band(h: i32, c: i32) -> u8 {
    if h > c - c / 3 {
        2
    } else if h > c / 2 {
        1
    } else {
        0
    }
}

/// A `name` aircraft of house 0 in contact with the pad, landing by hand
/// toward it from `dz` leptons above the centre of the pad's first
/// foundation cell, with `beh` current, its current fraction one half and
/// its target fraction 0, as its landing left them. The pad's docking
/// position lies two cells east of the foundation, so its cell holds no
/// building: claim 7's check refuses it to a pad-bound aircraft. Every
/// cell's ground record holds an object, so the nearby search finds no free
/// cell and the aircraft takes claim 7's hit. Returns (world, pad,
/// aircraft).
fn refused(r: &Ruleset, name: &str, dz: i32, beh: Behaviour) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = testkit::spawn(&mut w, Kind::Aircraft, name, 0, testkit::mid(SIZE, 8, 0), 0);
    step(&mut w);
    let x = foundation(&w, k)[0];
    // Set-up: lifted off the ground first, so that its listing and flag go.
    w.obj_mut(a).unwrap().pos.z += 200;
    air::sync_listing(&mut w, a);
    air::clear_mark(&mut w, a);
    let centre = Pos::centre(x, 0);
    let z = w.surface_at(centre.x, centre.y) + dz;
    let t = w.tick;
    let half = w.rules().air.half;
    {
        let o = w.obj_mut(a).unwrap();
        o.pos = Pos::new(centre.x, centre.y, z);
        o.beh.force(beh, t, true);
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
        s.cur = half;
        s.tgt = F64::ZERO;
    }
    air::sync_listing(&mut w, a);
    let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
    for c in cells {
        if let Some(rec) = w.cells.occ_mut(c, Layer::Ground) {
            rec.others = rec.others.saturating_add(1);
        }
    }
    (w, k, a)
}

/// [`refused`] run through the update of the hit; asserts the survivor's
/// state at its end and returns it with its band there.
fn survivor(r: &Ruleset, name: &str, dz: i32, beh: Behaviour) -> (World, ObjectId, ObjectId, u8) {
    let (mut w, k, a) = refused(r, name, dz, beh);
    step(&mut w);
    let o = w.obj(a).unwrap();
    let s = st(&w, a);
    let tag = format!("dz {dz}: {}", describe(&w, a));
    assert_eq!(o.health, 200, "the stage 2 row stopped the hit: {tag}");
    assert!(
        !s.landing && !s.taking_off && !s.ground,
        "no touchdown: {tag}"
    );
    assert!(s.origin && s.flying_to.is_none(), "the origin point: {tag}");
    assert!(s.unfinished, "its flight stays unfinished: {tag}");
    assert_eq!(o.dest, Some(Target::Object(k)), "kept: {tag}");
    let b = band(air::height(&w, a), cruise(&w, a));
    (w, k, a, b)
}

/// The heights above the pad's cell, one per band of claim 8, at which a
/// BOUND's refused landing leaves it in that band at the hit.
fn band_heights(r: &Ruleset) -> [i32; 3] {
    let (w, _, a) = refused(r, "BOUND", 0, Behaviour::Sleep);
    let c = cruise(&w, a);
    let mut found = [None; 3];
    let mut dz = 0;
    while dz <= 2 * c && found.iter().any(Option::is_none) {
        let (mut w, _, a) = refused(r, "BOUND", dz, Behaviour::Sleep);
        step(&mut w);
        let b = band(air::height(&w, a), c) as usize;
        found[b].get_or_insert(dz);
        dz += 5;
    }
    found.map(|f| f.expect("every band reached"))
}

// ---------------------------------------------------------------------------
// Claim 7: the survivor of the hit (Q-2676)

// spec: S-0501 claim 7 (r146, Q-2676: a survivor of the hit holds the map's origin point as flight point and its flight stays unfinished; the hit update's take-off step makes claim 8's requests toward that point by band: above the upper threshold the aiming request, in the middle band the body toward the origin point and full speed, lower none; its movement updates treat that point as no flight point for claim 5's calculation and for guidance, while claim 4's forward step, the climb to its held height, the step of the current fraction and claim 6's exact-cell trigger go on; no movement update hands its destination to its flight again, so a kept pad does not take it back; it stops in the air where its slowing ends, or after the middle band's requests flies on along that line at full speed; no landing, no check and no hit again) and claim 8 (the thresholds H − ⌊H / 3⌋ and ⌊H / 2⌋); S-0302 claim 8 (stage 2's poison row) and claim 9 (the rows that can stop a hit that ignores defences)
#[test]
fn claim_7_a_survivor_holds_the_origin_point_in_each_band() {
    let r = rules(true, OFF_PAD);
    let heights = band_heights(&r);
    for (want, dz) in heights.into_iter().enumerate() {
        let (mut w, k, a, b) = survivor(&r, "BOUND", dz, Behaviour::Sleep);
        assert_eq!(b as usize, want);
        let s = st(&w, a);
        let tag = format!("band {b}: {}", describe(&w, a));
        if b == 1 {
            assert!(s.tgt.eq_num(one(&w)), "full speed requested: {tag}");
        } else {
            assert!(zero(s.tgt), "the landing's target fraction kept: {tag}");
        }
        let start = w.obj(a).unwrap().pos;
        let mut last = start;
        let mut still = 0;
        for i in 0..1500 {
            step(&mut w);
            let o = w.obj(a).expect("never removed");
            let s = st(&w, a);
            let tag = format!("band {b}, update {i}: {}", describe(&w, a));
            assert!(!s.landing && !s.ground, "no landing again: {tag}");
            assert!(s.origin && s.flying_to.is_none(), "{tag}");
            assert_eq!(o.dest, Some(Target::Object(k)), "kept: {tag}");
            assert_eq!(o.health, 200, "no hit again: {tag}");
            assert!(air::height(&w, a) <= s.held.max(air::height(&w, a)));
            if b == 1 {
                assert!(s.tgt.eq_num(one(&w)), "the fraction stays 1: {tag}");
                if i < 100 {
                    assert!(
                        from_origin(o.pos) <= from_origin(last),
                        "toward the origin point: {tag}"
                    );
                }
            } else {
                assert!(zero(s.tgt), "{tag}");
                if (o.pos.x, o.pos.y) == (last.x, last.y) {
                    still += 1;
                } else {
                    assert_eq!(still, 0, "once stopped it stays: {tag}");
                }
            }
            last = o.pos;
        }
        let s = st(&w, a);
        assert_eq!(air::height(&w, a), cruise(&w, a), "at its held height");
        assert_eq!(s.held, cruise(&w, a), "the take-off's cruise height");
        if b == 1 {
            assert!(s.cur.eq_num(one(&w)), "{}", describe(&w, a));
            assert!(from_origin(last) != from_origin(start));
        } else {
            assert!(zero(s.cur), "it has come to a stop: {}", describe(&w, a));
            assert!(still > 1000, "{}", describe(&w, a));
            let d = (last.x - start.x).abs().max((last.y - start.y).abs());
            assert!(d < 256, "it stopped near the hit: {}", describe(&w, a));
        }
    }
}

// spec: S-0501 claim 7 (r146, Q-2676: the hit update's take-off step makes claim 8's requests by band: above the upper threshold the aiming request, lower no aiming request) and claim 8 (above H − ⌊H / 3⌋ the aiming facing is requested to match the current body facing, with no body request) (seam sweep 6: intake 108's row)
#[test]
fn seams6_a_survivor_above_the_upper_threshold_makes_the_aiming_request() {
    use glasnost_sim::turning::Turning;
    let r = rules(true, OFF_PAD);
    let heights = band_heights(&r);
    for (want, dz) in heights.into_iter().enumerate() {
        let (mut w, _, a) = refused(&r, "BOUND", dz, Behaviour::Sleep);
        let t = w.tick;
        let body = w.obj(a).unwrap().body.value(t);
        let away = body.wrapping_add(16384);
        let o = w.obj_mut(a).unwrap();
        o.body = Turning::at(body);
        o.turret = Turning::at(away);
        step(&mut w);
        let o = w.obj(a).unwrap();
        let s = st(&w, a);
        assert!(s.origin, "the hit's survivor: {}", describe(&w, a));
        let b = band(air::height(&w, a), cruise(&w, a));
        assert_eq!(b as usize, want);
        if b == 2 {
            assert_eq!(
                o.turret.target, o.body.target,
                "the aiming facing is asked to match the body"
            );
            assert_eq!(o.body.target, body, "no body request");
        } else {
            assert_eq!(o.turret.target, away, "no aiming request in band {b}");
        }
    }
}

// spec: S-0501 claim 7 (r146, Q-2676: the origin point lasts until the survivor is given a destination by an order, which is handed to its flight as any destination is, claim 8) and claim 6 (r146: it is still moving, so a stop order, being given no destination, gives it claim 6's new flight point: the centre of the cell claim 40's selection gives, its own point)
#[test]
fn claim_7_an_order_ends_the_origin_point() {
    let r = rules(true, OFF_PAD);
    let heights = band_heights(&r);
    let (mut w, _, a, _) = survivor(&r, "BOUND", heights[2], Behaviour::Sleep);
    let c = w.obj(a).unwrap().pos.cell();
    let to = CellXY::new(c.x + 6, c.y);
    assert!(issue(
        &mut w,
        0,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(to)),
        }
    ));
    // The order is applied in the next tick; the movement update after it
    // hands the destination over.
    let mut handed = None;
    for i in 0..3 {
        step(&mut w);
        if !st(&w, a).origin {
            handed = Some(i);
            break;
        }
    }
    assert!(handed.is_some(), "{}", describe(&w, a));
    let s = st(&w, a);
    assert_eq!(w.obj(a).unwrap().dest, Some(Target::Cell(to)));
    assert_eq!(s.flying_to, Some(Target::Cell(to)), "{}", describe(&w, a));
    let (mut w, _, a, _) = survivor(&r, "BOUND", heights[2], Behaviour::Sleep);
    assert!(issue(&mut w, 0, Order::Stop { actor: a }));
    step(&mut w);
    let s = st(&w, a);
    assert_eq!(w.obj(a).unwrap().dest, None);
    assert!(!s.origin, "{}", describe(&w, a));
    let Some(Target::Cell(_)) = s.flying_to else {
        panic!("claim 6's new flight point: {}", describe(&w, a));
    };
    assert!(s.own_point && s.unfinished, "{}", describe(&w, a));
}

// spec: S-0501 claim 7 (r146, Q-2676: a survivor with no destination in flight that holds a contact gets its pad back as destination when the pad finds it settled, claim 24's rule for an aircraft in flight with no destination, so it flies back, and a new landing there can be refused and hit again in the same way) and claim 24 (that rule)
#[test]
fn claim_7_a_survivor_with_no_destination_gets_its_pad_back() {
    let r = rules(true, OFF_PAD);
    let heights = band_heights(&r);
    let (mut w, k, a, _) = survivor(&r, "BOUND", heights[2], Behaviour::Sleep);
    // Set-up: its destination gone, as an order that clears it would leave
    // it once claim 6's new point is reached; here the clear is by hand, and
    // the movement update meets it as being given no destination.
    w.obj_mut(a).unwrap().dest = None;
    step(&mut w);
    let s = st(&w, a);
    assert!(!s.origin && s.own_point, "{}", describe(&w, a));
    // A survivor that holds no destination (set by hand: the origin point
    // with no destination and nothing kept).
    let (mut w, k2, a2, _) = survivor(&r, "BOUND", heights[2], Behaviour::Sleep);
    w.obj_mut(a2).unwrap().dest = None;
    w.obj_mut(a2).unwrap().air.as_mut().unwrap().ignored = None;
    step(&mut w);
    let s = st(&w, a2);
    assert!(s.origin, "{}", describe(&w, a2));
    assert_eq!(w.obj(a2).unwrap().dest, None);
    for _ in 0..20 {
        step(&mut w);
        let s = st(&w, a2);
        assert!(s.origin && !s.landing, "{}", describe(&w, a2));
    }
    let t = w.tick;
    w.obj_mut(k2).unwrap().air.get_or_insert_default().service = Some(t);
    air::pad_turn(&mut w, k2);
    assert_eq!(
        w.obj(a2).unwrap().dest,
        Some(Target::Object(k2)),
        "the pad given back: {}",
        describe(&w, a2)
    );
    step(&mut w);
    let s = st(&w, a2);
    assert!(!s.origin, "handed to its flight: {}", describe(&w, a2));
    assert_eq!(s.flying_to, Some(Target::Object(k2)));
    // It flies back toward the pad's docking position: a new landing there
    // can be refused (claim 6's pad-bound rule refuses each landing start
    // off the foundation here, so no second check comes in this set-up).
    let dock = glasnost_sim::econ::release::pad_position(&w, k2, a2).0;
    let d0 = from_origin(Pos::new(
        w.obj(a2).unwrap().pos.x - dock.x,
        w.obj(a2).unwrap().pos.y - dock.y,
        0,
    ));
    let mut closest = d0;
    for _ in 0..1500 {
        step(&mut w);
        let p = w.obj(a2).unwrap().pos;
        closest = closest.min(from_origin(Pos::new(p.x - dock.x, p.y - dock.y, 0)));
        assert!(!st(&w, a2).origin, "{}", describe(&w, a2));
    }
    assert!(
        closest < d0 && closest <= 512,
        "flies back: {d0} {closest}: {}",
        describe(&w, a2)
    );
    let _ = (k, a);
}

// spec: S-0501 claim 24 (r146, Q-2676: a pad-bound survivor in enter keeps the pad as destination and its contact; its processings stay those of the approaching stage in contact, which find it neither landing nor on the ground, and its request to the pad finds it not settled, since its locomotor does not report stopped and its destination is the pad: it stays in enter and is never serviced, whether stopped in the air or flying on) and claim 40 (its locomotor does not report stopped)
#[test]
fn claim_24_a_pad_bound_survivor_in_enter_is_never_serviced() {
    let r = rules(true, OFF_PAD);
    let heights = band_heights(&r);
    for b in [2, 1] {
        let (mut w, k, a, got) = survivor(&r, "BOUND", heights[b], Behaviour::Enter);
        assert_eq!(got as usize, b);
        w.obj_mut(a).unwrap().combat.ammo = 0;
        for _ in 0..1200 {
            step(&mut w);
            let o = w.obj(a).expect("never removed");
            let tag = format!("band {b}: {}", describe(&w, a));
            assert_eq!(o.beh.current, Some(Behaviour::Enter), "{tag}");
            assert_eq!(o.contact, Some(k), "{tag}");
            assert_eq!(o.dest, Some(Target::Object(k)), "{tag}");
            assert_eq!(o.combat.ammo, 0, "never serviced: {tag}");
            assert!(st(&w, a).origin && !st(&w, a).landing, "{tag}");
            assert!(!st(&w, a).enter_docking, "approaching: {tag}");
        }
    }
}

// spec: S-0501 claim 7 (r146, Q-2676: where the survivor's flight leaves the legal map, S-0009 claim 1, claims 4 and 10 act on it as on any aircraft's, with quirks.aircraft_edge_hold as for any aircraft; one that may not leave the map now, as a pad-bound aircraft that is not visiting, is moved on outside and never removed by claim 10; outside the legal map our engine keeps acting every rule the claim gives the survivor inside it, so each later movement update moves it on along that line by claim 4's step, and it adds no rule there) and Open question 28 (a design choice of scope) and Edge cases (quirks.aircraft_edge_hold in both positions: on, a visiting survivor with the fly-back flag is pushed back at the edge; off, it is removed on leaving the legal map); claim 10; the state survives a save and load
#[test]
fn claim_7_a_survivor_flies_on_off_the_legal_map() {
    for hold in [true, false] {
        let mut r = rules(true, OFF_PAD);
        r.globals.quirks.aircraft_edge_hold = hold;
        let heights = band_heights(&r);
        // Pad-bound, not visiting: it may not leave now.
        let (mut w, _, a, b) = survivor(&r, "BOUND", heights[1], Behaviour::Sleep);
        assert_eq!(b, 1);
        let mut outside = 0;
        let mut was = w.obj(a).unwrap().pos;
        for i in 0..6000 {
            step(&mut w);
            let o = w
                .obj(a)
                .unwrap_or_else(|| panic!("hold {hold}: removed at {i}"));
            let p = o.pos;
            if w.cells.cell(p.cell()).is_none() {
                outside += 1;
                assert_ne!((p.x, p.y), (was.x, was.y), "moved on: {}", describe(&w, a));
            }
            was = p;
        }
        assert!(
            outside > 100,
            "hold {hold}: off the legal map: {}",
            describe(&w, a)
        );
        let s = st(&w, a);
        assert!(s.origin && s.tgt.eq_num(one(&w)), "{}", describe(&w, a));
        let bytes = save::save(&w);
        let mut l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
        assert_eq!(save::state_hash(&l), save::state_hash(&w));
        for _ in 0..50 {
            step(&mut w);
            step(&mut l);
        }
        assert_eq!(save::state_hash(&l), save::state_hash(&w));
        // Visiting, with the fly-back flag: it may leave now (claim 10), so
        // claim 4's push-back holds it at the edge with the switch on, and
        // claim 10 removes it on leaving the legal map with it off.
        let (mut w, _, a, b) = survivor(&r, "BACK", heights[1], Behaviour::Sleep);
        assert_eq!(b, 1);
        {
            let s = w.obj_mut(a).unwrap().air.as_mut().unwrap();
            s.visiting = true;
            s.entered = true;
        }
        let mut removed = None;
        for i in 0..6000 {
            step(&mut w);
            let Some(o) = w.obj(a) else {
                removed = Some(i);
                break;
            };
            assert!(
                w.cells.cell(o.pos.cell()).is_some(),
                "hold {hold}: never off the legal map while held: {}",
                describe(&w, a)
            );
        }
        assert_eq!(removed.is_some(), !hold, "hold {hold}: {removed:?}");
    }
}

// spec: S-0501 claim 7 (r134: when the hit destroys it, it is dead; r146: the origin point is the survivor's only) and S-0302 claim 8 (the stage 2 row holds only for a poison warhead against a type immune to poison)
#[test]
fn claim_7_without_the_stage_2_row_the_hit_destroys_it() {
    let r = rules(false, OFF_PAD);
    let (mut w, _, a) = refused(&r, "BOUND", 200, Behaviour::Sleep);
    step(&mut w);
    let o = w.obj(a).unwrap();
    assert!(glasnost_sim::combat::damage::dead(o), "{}", describe(&w, a));
}

// ---------------------------------------------------------------------------
// Claim 33: *guard*'s *move* at the movement update (Q-2677)

/// A `name` aircraft of house 0 at rest on the ground in the middle, with
/// `beh` current, a destination six cells east and nothing queued. Returns
/// (world, aircraft, destination).
fn grounded(r: &Ruleset, name: &str, beh: Behaviour) -> (World, ObjectId, CellXY) {
    let mut w = game(r);
    ready(&mut w);
    let a = testkit::spawn(&mut w, Kind::Aircraft, name, 0, testkit::mid(SIZE, 0, 0), 0);
    step(&mut w);
    assert!(st(&w, a).ground, "{}", describe(&w, a));
    let c = w.obj(a).unwrap().pos.cell();
    let to = CellXY::new(c.x + 6, c.y);
    let t = w.tick;
    {
        let o = w.obj_mut(a).unwrap();
        o.beh.force(beh, t, true);
        o.beh.queued = None;
        o.dest = Some(Target::Cell(to));
    }
    (w, a, to)
}

// spec: S-0501 claim 33 (r146, Q-2677: a living aircraft that is not pad-bound and not crashing, in guard with a destination and nothing queued at a movement update, gets move queued by that update, at any height, on the ground included; move then becomes current at claim 31's third point, by the end of that tick) and claim 29 (r146: guard's processing at any height gives no such move itself)
#[test]
fn claim_33_guard_with_a_destination_gets_move_on_the_ground() {
    let r = rules(false, OFF_PAD);
    let (mut w, a, to) = grounded(&r, "JET", Behaviour::Guard);
    step(&mut w);
    let o = w.obj(a).unwrap();
    assert!(
        o.beh.current == Some(Behaviour::Move) || o.beh.queued == Some(Behaviour::Move),
        "{}",
        describe(&w, a)
    );
    for _ in 0..3 {
        step(&mut w);
    }
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Move), "{}", describe(&w, a));
    assert_eq!(o.dest, Some(Target::Cell(to)));
    // Pad-bound: no such move; nor in another behaviour; nor without a
    // destination.
    for (name, beh, dest) in [
        ("BOUND", Behaviour::Guard, true),
        ("JET", Behaviour::Sleep, true),
        ("JET", Behaviour::Guard, false),
    ] {
        let (mut w, a, _) = grounded(&r, name, beh);
        if !dest {
            w.obj_mut(a).unwrap().dest = None;
        }
        step(&mut w);
        let o = w.obj(a).unwrap();
        assert!(
            o.beh.current != Some(Behaviour::Move) && o.beh.queued != Some(Behaviour::Move),
            "{name} {beh:?} dest {dest}: {}",
            describe(&w, a)
        );
    }
}

// spec: S-0501 claim 33 (r146, Q-2677: in flight too, the movement update queues move for guard with a destination and nothing queued; not for a crashing aircraft, claim 9) and claim 9 (a living crash switches its locomotor off)
#[test]
fn claim_33_in_flight_and_not_when_crashing() {
    let r = rules(false, OFF_PAD);
    for crash in [false, true] {
        let (mut w, a, _) = grounded(&r, "JET", Behaviour::Guard);
        // Set-up: in flight 40 leptons above its cruise height, where a
        // processing of guard with full ammunition does nothing (claim 29:
        // the cruise-height outcomes come at exactly that height).
        let c = cruise(&w, a);
        {
            let o = w.obj_mut(a).unwrap();
            o.pos.z += c + 40;
            let s = o.air.as_mut().unwrap();
            s.ground = false;
            s.held = c;
        }
        air::sync_listing(&mut w, a);
        air::clear_mark(&mut w, a);
        if crash {
            assert!(air::start_crash(&mut w, a, true));
        }
        step(&mut w);
        let o = w.obj(a).unwrap();
        let moved = o.beh.current == Some(Behaviour::Move) || o.beh.queued == Some(Behaviour::Move);
        assert_eq!(moved, !crash, "crash {crash}: {}", describe(&w, a));
    }
}

// ---------------------------------------------------------------------------
// Claim 24: the slide after the entry report (Q-2677, AC31 part h (6))

/// AC31 part h (1)'s pad and a JET in contact with it, in *enter*,
/// approaching with its ammunition spent, put `dx`, `dy` cells from the
/// middle. Returns (world, pad, JET).
fn approaching(r: &Ruleset, dx: i32, dy: i32) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = testkit::spawn(
        &mut w,
        Kind::Aircraft,
        "JET",
        0,
        testkit::mid(SIZE, dx, dy),
        0,
    );
    step(&mut w);
    w.obj_mut(a).unwrap().combat.ammo = 0;
    glasnost_sim::process::fall_back(&mut w, a, false);
    assert_eq!(w.obj(a).unwrap().beh.current, Some(Behaviour::Enter));
    for _ in 0..12 {
        step(&mut w);
        if w.obj(a).unwrap().contact == Some(k) {
            return (w, k, a);
        }
    }
    panic!("no contact: {}", describe(&w, a));
}

/// One tick of the record of AC31 part h (6).
#[derive(Clone, Debug)]
struct Row {
    tick: i32,
    cur: Option<Behaviour>,
    queued: Option<Behaviour>,
    dest: Option<Target>,
    contact: Option<ObjectId>,
    cell: CellXY,
    h: i32,
    landing: bool,
    taking_off: bool,
    ammo: i32,
    servicing: bool,
}

fn row(w: &World, k: ObjectId, a: ObjectId) -> Row {
    let o = w.obj(a).unwrap();
    let s = st(w, a);
    Row {
        tick: w.tick - 1,
        cur: o.beh.current,
        queued: o.beh.queued,
        dest: o.dest,
        contact: o.contact,
        cell: o.pos.cell(),
        h: air::height(w, a),
        landing: s.landing,
        taking_off: s.taking_off,
        ammo: o.combat.ammo,
        servicing: service(w, k).is_some(),
    }
}

/// How a run whose landing is moved `ox`, `oy` leptons at its start goes:
/// steps to its touchdown *u*; returns the record from *u* to 1200 ticks
/// after it, with the docking position's cell, and whether that touchdown
/// is in a cell next to the docking position's (part h (6) keeps only those
/// runs; the record of any other run is *u* alone).
fn h6_run(r: &Ruleset, ox: i32, oy: i32) -> (Vec<Row>, CellXY, bool) {
    let (mut w, k, a) = approaching(r, 8, 0);
    let dc = glasnost_sim::econ::release::pad_position(&w, k, a).0.cell();
    let mut was = st(&w, a);
    let mut u = false;
    let mut shifted = false;
    for _ in 0..2000 {
        step(&mut w);
        let s = st(&w, a);
        if s.landing && !shifted {
            // Set-up: at the end of the tick its landing starts, moved
            // `ox`, `oy` leptons, so that it touches down in a cell next to
            // the docking position's (in our engine every approach of this
            // pad lands in that cell itself).
            shifted = true;
            let o = w.obj_mut(a).unwrap();
            o.pos.x += ox;
            o.pos.y += oy;
        }
        if was.landing && !s.landing && air::height(&w, a) <= 0 {
            u = true;
            break;
        }
        was = s;
    }
    assert!(u, "a touchdown: {}", describe(&w, a));
    let c = w.obj(a).unwrap().pos.cell();
    let mut rec = vec![row(&w, k, a)];
    if c == dc || !near(c, dc) {
        return (rec, dc, false);
    }
    for _ in 0..1200 {
        step(&mut w);
        rec.push(row(&w, k, a));
    }
    (rec, dc, true)
}

/// The runs of part h (6) from a spread of starts, each with the docking
/// position's cell: those whose touchdown is in a cell next to it. Also a
/// summary of every run: its touchdown cell and the docking position's.
fn h6_runs(r: &Ruleset) -> (Vec<(Vec<Row>, CellXY)>, String) {
    let mut runs = Vec::new();
    let mut all = String::new();
    for (ox, oy) in [(256, 0), (256, -256), (256, 256), (0, -256), (0, 256)] {
        let (rec, dc, kept) = h6_run(r, ox, oy);
        all += &format!("({ox},{oy}): u {:?} dc {dc:?}; ", rec[0].cell);
        if kept {
            runs.push((rec, dc));
        }
    }
    (runs, all)
}

// spec: S-0501 claim 24 (r146, Q-2677: an aircraft that is not pad-bound and slides after its entry report gets guard at the movement update's first point and claim 33's move in the same update, current at the third point, so guard is never processed while it slides; move's first processing keeps the pad, claim 40, and takes it off; it lands again at the docking position and rests in enter, the stay of quirks.aircraft_grounded_enter_stall, never serviced; with that switch off it reports again and is serviced) and claim 33 (r146) and Edge cases (quirks.aircraft_grounded_enter_stall in both positions); AC31 part h (6)
#[test]
fn ac31h6_after_a_touchdown_beside_the_docking_position_it_takes_off_and_lands_again() {
    for stall in [true, false] {
        let mut r = rules(false, OFF_PAD);
        r.globals.quirks.aircraft_grounded_enter_stall = stall;
        let (runs, all) = h6_runs(&r);
        let mut kept = 0;
        let mut void = 0;
        for (rec, dc) in runs {
            let k_contact = rec[0].contact;
            // r: the entry report, the first tick at whose end enter is no
            // longer current.
            let Some(ri) = rec.iter().position(|x| x.cur != Some(Behaviour::Enter)) else {
                panic!("stall {stall}: no report: {:?}", rec.last());
            };
            // Void: no destination at the end of a tick from u to r.
            if rec[..=ri].iter().any(|x| x.dest.is_none()) {
                void += 1;
                continue;
            }
            kept += 1;
            let rr = &rec[ri];
            let tag = format!("stall {stall}: r {rr:?}");
            assert_eq!(rr.cur, Some(Behaviour::Move), "{tag}");
            assert_eq!(rr.queued, None, "{tag}");
            assert!(rr.dest.is_some(), "{tag}");
            assert!(rr.h == 0 && !rr.taking_off, "{tag}");
            let r1 = &rec[ri + 1];
            assert!(r1.taking_off || r1.h > 0, "taken off at r + 1: {r1:?}");
            let mi = (ri + 1..rec.len())
                .find(|&i| rec[i].dest.is_none() && rec[i].h == 0 && !rec[i].landing)
                .unwrap_or_else(|| panic!("stall {stall}: no m: {:?}", rec.last()));
            let m = &rec[mi];
            assert!(m.tick > rr.tick, "{m:?}");
            assert!(
                rec[ri..=mi].iter().all(|x| x.cur == Some(Behaviour::Move)),
                "move current from r to m: {tag}"
            );
            assert!(
                rec[ri..mi].iter().any(|x| x.landing),
                "landing again: {tag}"
            );
            assert_eq!(m.cell, dc, "at the docking position: {m:?}");
            let ei = (mi + 1..rec.len())
                .find(|&i| rec[i].cur != m.cur)
                .unwrap_or_else(|| panic!("no e: {m:?}"));
            assert_eq!(rec[ei].cur, Some(Behaviour::Enter), "{:?}", rec[ei]);
            // With the switch off its second report starts a servicing,
            // which has it queue sleep (claim 26): no sleep before e only.
            let end = if stall { rec.len() } else { ei };
            assert!(
                rec[ri + 1..end]
                    .iter()
                    .all(|x| x.cur != Some(Behaviour::Sleep)),
                "never sleep: {tag}"
            );
            if stall {
                for x in &rec[ei..] {
                    assert_eq!(x.cur, Some(Behaviour::Enter), "the stay: {x:?}");
                    assert_eq!(x.contact, k_contact, "{x:?}");
                }
                for x in &rec[mi..] {
                    assert_eq!(x.ammo, 0, "never serviced: {x:?}");
                    assert!(!x.servicing, "{x:?}");
                }
            } else {
                assert!(
                    rec[ei..].iter().any(|x| x.ammo > 0),
                    "reported again and serviced: {:?}",
                    rec.last()
                );
            }
        }
        assert!(kept > 0, "stall {stall}: no run kept ({void} void): {all}");
    }
}

// spec: S-0501 claim 24 (r146, Q-2677: one ready for take-off at full health that a service step releases in the report's tick falls back to rest with guard forced, which empties its queue, S-0215 claim 9, so at the end of that tick it has guard current and nothing queued, not move; that fall-back is T2's row on the ground and forgets the pad as destination; as it still holds its flight point it is moving, so claim 6's landing where it is gives it a new point of its own and it takes off toward it; claim 33 then queues no move while it has no destination) and claim 26 (the release before the skip)
#[test]
fn claim_24_a_release_in_the_report_tick_leaves_guard() {
    let r = rules(false, [0, 0, 0]);
    let mut w = game(&r);
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
    // Set-up: sliding on the ground with the pad as destination and flight
    // point, in contact, guard forced at its report and claim 33's move
    // queued by the report tick's movement update; the pad's service step
    // due now.
    let t = w.tick;
    {
        let o = w.obj_mut(a).unwrap();
        o.contact = Some(k);
        o.econ.exit = Some(Exit {
            factory: k,
            tethered: false,
            place: 0,
        });
        o.beh.force(Behaviour::Guard, t, true);
        o.beh.queued = Some(Behaviour::Move);
        o.dest = Some(Target::Object(k));
        let s = o.air.as_mut().unwrap();
        s.ground = false;
        s.sliding = true;
        s.flying_to = Some(Target::Object(k));
        s.own_point = false;
        s.unfinished = true;
        s.held = 0;
    }
    air::sync_listing(&mut w, a);
    w.obj_mut(k).unwrap().air.get_or_insert_default().service = Some(t);
    air::pad_turn(&mut w, k);
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard), "{}", describe(&w, a));
    assert_eq!(o.beh.queued, None, "the queue emptied: {}", describe(&w, a));
    assert_eq!(o.dest, None, "the pad forgotten: {}", describe(&w, a));
    // The clear is met at its next movement update: claim 6's new point
    // of its own, and a take-off toward it.
    step(&mut w);
    let s = st(&w, a);
    assert!(
        s.own_point && matches!(s.flying_to, Some(Target::Cell(_))),
        "{}",
        describe(&w, a)
    );
    assert!(
        !s.sliding && (s.taking_off || air::height(&w, a) > 0),
        "{}",
        describe(&w, a)
    );
    for _ in 0..5 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        assert!(
            o.beh.current != Some(Behaviour::Move) && o.beh.queued != Some(Behaviour::Move),
            "no move: {}",
            describe(&w, a)
        );
    }
}
