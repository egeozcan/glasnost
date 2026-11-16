//! S-0312 revision 88 claim 34 with S-0107 claim 17 row 7 and S-0611
//! claims 6 to 8 (S24's seam of #383): an infantry passenger killed inside
//! its transport by a hit of the mutating death kind. Synthetic rules: the
//! test kit's combat rules plus
//! - warhead MUT (infantry-death kind 9, versus 1);
//! - animation MUTA (make-infantry index 0, 4 frames), named by the general
//!   conversion key, with the made-type list GROWN (a WALKER);
//! - JJC (a WALKER with jumpjet movement and the crash flag);
//! - VAN (an unarmed BUGGY with 2 seats) and OPEN (VAN, open-topped).
//!
//! Every name and number is invented; no game data.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, Decimal, LandRow, LandType, Locomotor, WarheadType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::mutation::MakerEvent;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, World};
use glasnost_sim::{objects, testkit, transport};

const SIZE: i32 = 24;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    register(&mut r, "warheads", "MUT");
    r.sim.warheads.insert(
        "MUT".into(),
        WarheadType {
            verses: Some(vec![d("1"); 11]),
            inf_death: Some(9),
            ..Default::default()
        },
    );
    register(&mut r, "animations", "MUTA");
    r.sim.animations.insert(
        "MUTA".into(),
        AnimationType {
            make_infantry: Some(0),
            frame_count: Some(4),
            ..Default::default()
        },
    );
    let grown = r.sim.infantry["WALKER"].clone();
    register(&mut r, "infantry", "GROWN");
    r.sim.infantry.insert("GROWN".into(), grown);
    let mut jjc = r.sim.infantry["WALKER"].clone();
    jjc.locomotor = Some(Locomotor::Jumpjet);
    jjc.jumpjet = Some(false);
    jjc.jumpjet_speed = Some(14);
    jjc.jumpjet_climb = Some(d("5.0"));
    jjc.jumpjet_crash = Some(d("6.0"));
    jjc.jumpjet_height = Some(500);
    jjc.jumpjet_accel = Some(d("2.0"));
    jjc.crashable = Some(true);
    register(&mut r, "infantry", "JJC");
    r.sim.infantry.insert("JJC".into(), jjc);
    let g = &mut r.sim.general.superweapons;
    g.infantry_mutate = Some("MUTA".into());
    g.anim_to_infantry = Some(vec!["GROWN".into()]);
    let mut van = r.sim.vehicles["BUGGY"].clone();
    van.primary = None;
    van.passengers = Some(2);
    van.size_limit = Some(d("2"));
    let mut open = van.clone();
    open.open_topped = Some(true);
    register(&mut r, "vehicles", "VAN");
    r.sim.vehicles.insert("VAN".into(), van);
    register(&mut r, "vehicles", "OPEN");
    r.sim.vehicles.insert("OPEN".into(), open);
    r
}

/// [`rules`] with a zero foot multiplier on clear land, so that S-0611
/// claim 6's first row refuses every conversion there.
fn refusing_rules() -> Ruleset {
    let mut r = rules();
    r.sim.land_types.insert(
        LandType::Clear,
        LandRow {
            foot: Some(d("0")),
            ..Default::default()
        },
    );
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// A game on `r` with infantry `passenger` of house 0 at (3, 0), raised by
/// `raise` leptons, boarded into `carrier` at (0, 0), which then moves to
/// (−5, 2). Returns the world, the passenger, the carrier and the
/// passenger's position when it went inside.
fn inside(r: &Ruleset, passenger: &str, carrier: &str, raise: i32) -> (World, ObjectId, ObjectId, Pos) {
    let mut w = testkit::game_with(r, SIZE, 7);
    testkit::run(&mut w, 2);
    let p = testkit::spawn(&mut w, Kind::Infantry, passenger, 0, at(3, 0), 0);
    let t = testkit::spawn(&mut w, Kind::Vehicle, carrier, 0, at(0, 0), 0);
    testkit::run(&mut w, 2);
    w.obj_mut(p).unwrap().pos.z += raise;
    let went_in = w.obj(p).unwrap().pos;
    transport::board(&mut w, p, t);
    assert_eq!(transport::riding(&w, p), Some(t), "set-up: boarded");
    w.obj_mut(t).unwrap().pos = Pos::centre(at(-5, 2), 0);
    transport::sync(&mut w, t);
    w.effects.clear();
    (w, p, t, went_in)
}

/// A lethal hit of the mutating death kind on `id`, of house 1.
fn mutate(w: &mut World, id: ObjectId) {
    let h = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "MUT")
        .unwrap() as u32;
    damage::apply(
        w,
        id,
        Hit {
            amount: 10_000,
            warhead: Some(h),
            distance: 0,
            attacker: None,
            house: Some(HouseId(1)),
            ignore_defences: true,
            no_escape: false,
        },
    );
}

/// The conversion animations the effects show as made, with their
/// positions.
fn made(w: &World) -> Vec<Pos> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Maker {
                event: MakerEvent::Made,
                pos,
                ..
            } => Some(*pos),
            _ => None,
        })
        .collect()
}

// spec: S-0312 claim 34 (b) and (d) (S-0107 claim 17's choice applies at the position the passenger had when it went inside; row 7's conversion removes the unit at once with no sequence and no corpse, S-0611 claim 7, so part d replaces none of it) and S-0611 claim 8 (one conversion animation at the victim's position)
#[test]
fn closed_transport_the_conversion_animation_comes_where_the_passenger_went_in() {
    let (mut w, p, t, went_in) = inside(&rules(), "WALKER", "VAN", 0);
    mutate(&mut w, p);
    assert_eq!(made(&w), [went_in]);
    assert!(w.obj(p).is_none(), "removed at once");
    assert_eq!(transport::count(&w, t), 0, "part (a): the seat is free");
}

// spec: S-0312 claim 34 (b) and (c) (inside an open-topped transport, S-0107 claim 17's choice applies as on the map, at the transport's own position at the hit) and S-0611 claim 8
#[test]
fn open_topped_transport_the_conversion_animation_comes_at_the_transport() {
    let (mut w, p, t, _) = inside(&rules(), "WALKER", "OPEN", 0);
    let here = w.obj(t).unwrap().pos;
    mutate(&mut w, p);
    assert_eq!(made(&w), [here]);
    assert!(w.obj(p).is_none(), "removed at once");
    assert_eq!(transport::count(&w, t), 0);
}

// spec: S-0312 claim 34 (d) (inside a closed transport a crash start of S-0107 claim 17's choice gives way: removed at once at the end of its death effects) with S-0611 claim 7 (the crash exception changes only the victim's removal: the conversion animation still starts)
#[test]
fn closed_transport_a_crashable_converted_passenger_starts_no_crash() {
    let (mut w, p, _, went_in) = inside(&rules(), "JJC", "VAN", 300);
    assert!(glasnost_sim::jump::is_jumpjet(&w, p), "set-up: jumpjet");
    mutate(&mut w, p);
    assert!(w.obj(p).is_none(), "removed at once, no crash");
    assert_eq!(made(&w), [went_in], "the conversion animation starts");
}

// spec: S-0312 claim 34 (b) and (d) (the passenger is off the map and holds no standing position) with S-0611 claim 7 (quirks.refused_mutation_frees_position: the refused victim's own standing position is shown free, and its removal frees it again): a refused passenger frees no position another unit holds where it went inside
#[test]
fn a_refused_conversion_inside_frees_no_position_it_does_not_hold() {
    let r = refusing_rules();
    assert!(r.globals.quirks.refused_mutation_frees_position);
    let (mut w, p, _, went_in) = inside(&r, "WALKER", "VAN", 0);
    let c = went_in.cell();
    // Another walker of house 0 stands where the passenger stood.
    let ti = w.rules().index_of(Kind::Infantry, "WALKER").unwrap();
    let other = objects::create(&mut w, Kind::Infantry, ti, Some(HouseId(0)));
    objects::place(&mut w, other, went_in, 0);
    let held = |w: &World| {
        w.cells
            .occ(c, Layer::Ground)
            .map(|o| o.marks.iter().flatten().count())
            .unwrap_or(0)
    };
    let before = held(&w);
    assert!(before > 0, "set-up: the other walker holds a position");
    mutate(&mut w, p);
    assert!(made(&w).is_empty(), "set-up: the conversion is refused");
    assert!(w.obj(p).is_none(), "part (d): removed at once");
    assert_eq!(held(&w), before, "the other walker's position stays taken");
    assert!(
        !w.mutation.refused.contains_key(&p),
        "no freed position is recorded for the removed passenger"
    );
}
