//! S-0312 revision 88 claim 34 (Q-3158, #383): a passenger that a killing
//! hit reaches inside its transport. Synthetic rules: the test kit's combat
//! rules plus
//! - animations ROW0, BODYA and BOOM (10 frames, no creation draws), the
//!   general corpse list BODYA;
//! - warheads DK0 and DK1 (infantry-death kinds 0 and 1, versus 1);
//! - DYER (WALKER with a die-1 sequence of 2 frames), ANIMED (WALKER with its
//!   own death animation list ROW0);
//! - VAN (an unarmed BUGGY with 2 seats), OPEN (VAN, open-topped) and TANKY
//!   (CART with the explosion list BOOM).
//!
//! Every name and number is invented; no game data.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, Decimal, SequenceEntry, WarheadType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::infantry::Seq;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, Pos, World};
use glasnost_sim::{testkit, tick, transport};

const SIZE: i32 = 24;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    for n in ["ROW0", "BODYA", "BOOM"] {
        register(&mut r, "animations", n);
        r.sim.animations.insert(
            n.into(),
            AnimationType {
                frame_count: Some(10),
                ..Default::default()
            },
        );
    }
    r.sim.general.dead_bodies = Some(vec!["BODYA".into()]);
    for k in [0, 1] {
        let n = format!("DK{k}");
        register(&mut r, "warheads", &n);
        r.sim.warheads.insert(
            n,
            WarheadType {
                verses: Some(vec![Decimal("1".into()); 11]),
                inf_death: Some(k),
                ..Default::default()
            },
        );
    }
    let mut dyer = r.sim.infantry["WALKER"].clone();
    dyer.sequences.insert(
        "die_1".into(),
        SequenceEntry {
            start: 0,
            count: 2,
            multiplier: 0,
            direction: None,
        },
    );
    register(&mut r, "infantry", "DYER");
    r.sim.infantry.insert("DYER".into(), dyer);
    let mut animed = r.sim.infantry["WALKER"].clone();
    animed.death_anims = Some(vec!["ROW0".into()]);
    register(&mut r, "infantry", "ANIMED");
    r.sim.infantry.insert("ANIMED".into(), animed);
    let mut van = r.sim.vehicles["BUGGY"].clone();
    van.primary = None;
    van.passengers = Some(2);
    van.size_limit = Some(Decimal("2".into()));
    let mut open = van.clone();
    open.open_topped = Some(true);
    register(&mut r, "vehicles", "VAN");
    r.sim.vehicles.insert("VAN".into(), van);
    register(&mut r, "vehicles", "OPEN");
    r.sim.vehicles.insert("OPEN".into(), open);
    let mut tanky = r.sim.vehicles["CART"].clone();
    tanky.combat.explosion = Some(vec!["BOOM".into()]);
    register(&mut r, "vehicles", "TANKY");
    r.sim.vehicles.insert("TANKY".into(), tanky);
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// A game on `r` with `passenger` of `kind` at (3, 0) boarded into
/// `carrier` at (0, 0), which then moves to (−5, 2), its passengers kept
/// with it as S-0507 claim 15 has it for an open-topped one. Returns the
/// world, the passenger, the carrier and the passenger's position when it
/// went inside.
fn inside(r: &Ruleset, kind: Kind, passenger: &str, carrier: &str) -> (World, ObjectId, ObjectId, Pos) {
    let mut w = testkit::game_with(r, SIZE, 7);
    testkit::run(&mut w, 2);
    let p = testkit::spawn(&mut w, kind, passenger, 0, at(3, 0), 0);
    let t = testkit::spawn(&mut w, Kind::Vehicle, carrier, 0, at(0, 0), 0);
    testkit::run(&mut w, 2);
    let went_in = w.obj(p).unwrap().pos;
    transport::board(&mut w, p, t);
    assert_eq!(transport::riding(&w, p), Some(t), "set-up: boarded");
    w.obj_mut(t).unwrap().pos = Pos::centre(at(-5, 2), 0);
    transport::sync(&mut w, t);
    w.effects.clear();
    (w, p, t, went_in)
}

fn warhead(w: &World, name: &str) -> u32 {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == name)
        .unwrap() as u32
}

fn kill(w: &mut World, id: ObjectId, wh: &str) {
    let h = warhead(w, wh);
    damage::apply(
        w,
        id,
        Hit {
            amount: 10_000,
            warhead: Some(h),
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
}

/// The animations the effects show, with their positions.
fn anims(w: &World) -> Vec<(String, Pos)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, pos, .. } => Some((name.clone(), *pos)),
            _ => None,
        })
        .collect()
}

fn killed_effects(w: &World, id: ObjectId) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Killed { id: k, .. } if *k == id))
        .count()
}

// spec: S-0312 claim 34 (b) (inside a transport that is not open-topped, the death effects come at the position the passenger had when it went inside: such a transport does not move its passengers with itself) and (d) (the animations of S-0107 claim 17's rows that show no sequence still appear; removed at once at the end of its death effects)
#[test]
fn closed_transport_the_death_animation_comes_where_the_passenger_went_in() {
    let (mut w, p, t, went_in) = inside(&rules(), Kind::Infantry, "ANIMED", "VAN");
    kill(&mut w, p, "DK1");
    assert_eq!(anims(&w), [("ROW0".to_string(), went_in)]);
    assert!(w.obj(p).is_none(), "removed at once");
    assert_eq!(transport::count(&w, t), 0, "part (a): the seat is free");
}

// spec: S-0312 claim 34 (b) (inside an open-topped transport, the death effects come at the transport's own position at the hit, S-0507 claim 15)
#[test]
fn open_topped_transport_the_death_animation_comes_at_the_transport() {
    let (mut w, p, t, _) = inside(&rules(), Kind::Infantry, "ANIMED", "OPEN");
    let here = w.obj(t).unwrap().pos;
    kill(&mut w, p, "DK1");
    assert_eq!(anims(&w), [("ROW0".to_string(), here)]);
    assert!(w.obj(p).is_none(), "a row with no sequence: removed at once");
    assert_eq!(transport::count(&w, t), 0);
}

// spec: S-0312 claim 34 (f) and Edge cases (quirks.closed_passenger_death_effects off: a passenger killed inside a transport that is not open-topped shows no death effects and takes no draws, and is removed at once; the switch does not reach an open-topped transport's passenger)
#[test]
fn the_switch_off_closed_transport_passengers_show_nothing() {
    let mut r = rules();
    r.globals.quirks.closed_passenger_death_effects = false;
    let (mut w, p, t, _) = inside(&r, Kind::Infantry, "ANIMED", "VAN");
    let before = w.rng.clone();
    kill(&mut w, p, "DK1");
    assert!(anims(&w).is_empty(), "no death effects");
    assert_eq!(w.rng, before, "no draws");
    assert!(w.obj(p).is_none());
    assert_eq!(transport::count(&w, t), 0);
    let (mut w, p, t, _) = inside(&r, Kind::Infantry, "ANIMED", "OPEN");
    let here = w.obj(t).unwrap().pos;
    kill(&mut w, p, "DK1");
    assert_eq!(anims(&w), [("ROW0".to_string(), here)], "open-topped: unchanged");
}

// spec: S-0312 claim 34 (a) (the passenger leaves the list at the hit and the open-topped transport no longer moves it) and (c) (S-0107 claim 17's death sequence plays in its own turns, off the map; meanwhile it takes no damage; at its completion the corpse comes at part b's position and the unit is removed)
#[test]
fn open_topped_transport_infantry_plays_its_death_sequence_off_the_map() {
    let (mut w, p, t, _) = inside(&rules(), Kind::Infantry, "DYER", "OPEN");
    let here = w.obj(t).unwrap().pos;
    kill(&mut w, p, "DK1");
    let o = w.obj(p).expect("it stays in play while its sequence plays");
    assert_eq!(o.infantry.as_ref().unwrap().seq, Some(Seq::Die1));
    assert!(!o.on_map, "off the map");
    assert_eq!(transport::riding(&w, p), None, "off the list");
    assert_eq!(transport::count(&w, t), 0, "the seat is free at once");
    // The transport moves on; the dying unit does not move with it.
    w.obj_mut(t).unwrap().pos = Pos::centre(at(5, 5), 0);
    transport::sync(&mut w, t);
    assert_eq!(w.obj(p).unwrap().pos, here);
    // A second hit does nothing.
    kill(&mut w, p, "DK1");
    assert_eq!(killed_effects(&w, p), 1, "one death");
    w.effects.clear();
    for _ in 0..200 {
        tick::step(&mut w);
        if w.obj(p).is_none() {
            break;
        }
    }
    assert!(w.obj(p).is_none(), "removed at the sequence's completion");
    assert_eq!(anims(&w), [("BODYA".to_string(), here)], "the corpse at part b's position");
}

// spec: S-0312 claim 34 (d) (infantry inside a closed transport: a death sequence is replaced by the removal at once, with no corpse and no corpse-pick draw)
#[test]
fn closed_transport_infantry_takes_no_sequence_and_no_corpse() {
    let (mut w, p, t, _) = inside(&rules(), Kind::Infantry, "DYER", "VAN");
    let before = w.rng.clone();
    kill(&mut w, p, "DK1");
    assert!(w.obj(p).is_none(), "removed at once");
    assert!(anims(&w).is_empty(), "no corpse");
    assert_eq!(w.rng, before, "no corpse-pick draw");
    assert_eq!(transport::count(&w, t), 0);
}

// spec: S-0312 claim 34 (e) (a vehicle passenger shows no explosion or destruction animation, leaves no crate and no wreck, and is removed at once)
#[test]
fn a_vehicle_passenger_is_removed_with_no_explosion() {
    let (mut w, p, t, _) = inside(&rules(), Kind::Vehicle, "TANKY", "VAN");
    kill(&mut w, p, "DK0");
    assert!(w.obj(p).is_none(), "removed at once");
    assert!(anims(&w).is_empty(), "no animation of claim 9");
    assert_eq!(transport::count(&w, t), 0);
}

// spec: S-0312 claim 34 (a) (when an action its destruction raises fire destroys the transport first, claims 5, 12 and 16 and S-0507 claim 16 pass over the passenger whose own killing hit is in progress: it leaves the list at once and its hit goes on)
#[test]
fn a_transport_dying_during_its_passengers_hit_passes_over_it() {
    let (mut w, p, t, _) = inside(&rules(), Kind::Infantry, "DYER", "OPEN");
    // The passenger's hit is in progress: health 0 and killed, still listed.
    let o = w.obj_mut(p).unwrap();
    o.health = 0;
    o.combat.killed = true;
    kill(&mut w, t, "DK0");
    assert!(w.obj(t).is_none(), "set-up: the transport is gone");
    assert!(w.obj(p).is_some(), "passed over: not removed");
    assert!(!w.obj(p).unwrap().on_map, "passed over: not let out");
    assert_eq!(transport::riding(&w, p), None, "off the list");
}
