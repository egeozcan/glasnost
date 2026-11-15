//! S-0615 claim 14: a disc laser is lost by events after the shot (issues
//! 313 and 314). Synthetic data only.

mod common;

use common::{Scene, cell, code_tile};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    Decimal, LandRow, LandType, LocomotionClass, Locomotor, MovementZone, UnitType, WarheadType,
    WeaponType,
};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::combat::effects::{DiscEvent, Effect};
use glasnost_sim::econ::stats;
use glasnost_sim::naval;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{testkit, tick, transport};

const WATER_CODE: u8 = 9;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn land_row(ground: &str, float: &str) -> LandRow {
    LandRow {
        foot: dec(ground),
        track: dec(ground),
        wheel: dec(ground),
        hover: dec("1"),
        float: dec(float),
        amphibious: dec("1"),
        float_beach: dec(float),
        buildable: None,
    }
}

fn ship(weight: &str) -> UnitType {
    let mut t = UnitType {
        strength: Some(300),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Float),
        locomotor: Some(Locomotor::Ship),
        movement_zone: Some(MovementZone::Water),
        weight: dec(weight),
        ..Default::default()
    };
    t.combat.naval = Some(true);
    t
}

fn naval_rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    r.sim.land_types.insert(LandType::Clear, land_row("1", "0"));
    r.sim.land_types.insert(LandType::Road, land_row("1", "0"));
    r.sim.land_types.insert(LandType::Water, land_row("0", "1"));
    r.sim.general.ship_sinking_weight = dec("3");
    push(&mut r, "warheads", "ZAP");
    r.sim.warheads.insert(
        "ZAP".into(),
        WarheadType {
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "DISC".into(),
        WeaponType {
            damage: Some(40),
            rof: Some(100),
            range: Some(Decimal("6".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("ZAP".into()),
            disk_laser: Some(true),
            ..Default::default()
        },
    );
    let saucer = UnitType {
        primary: Some("DISC".into()),
        ..r.sim.vehicles["CART"].clone()
    };
    for (n, u) in [
        ("SAUCER", saucer),
        ("HEAVY", ship("4")),
        ("LIGHT", ship("2")),
    ] {
        r.sim.vehicles.insert(n.into(), u);
        push(&mut r, "vehicles", n);
    }
    r
}

/// A flat map with water at dx <= 1 and land at dx >= 2 around the middle.
fn naval_world() -> World {
    let mut s = Scene::new();
    s.ruleset = naval_rules();
    for dx in -6..=1 {
        for dy in -4..=4 {
            s.set_cell(cell(dx, dy), code_tile(WATER_CODE), 0);
        }
    }
    s.build()
}

fn lethal() -> Hit {
    Hit {
        amount: 100_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: true,
    }
}

/// Fires one disc laser from a saucer on the shore at a ship of type `ty`,
/// kills the ship 4 ticks later with an unrelated hit, and runs on. Returns
/// the disc events after the kill, whether the ship was sinking after the
/// kill, and the firer's house's kill count at the end.
fn sink_run(ty: &str) -> (Vec<DiscEvent>, bool, i32) {
    let mut w = naval_world();
    let f = testkit::spawn(&mut w, Kind::Vehicle, "SAUCER", 0, cell(3, 0), 0);
    let t = testkit::spawn(&mut w, Kind::Vehicle, ty, 1, cell(0, 0), 0);
    while w.tick < 4 {
        tick::step(&mut w);
    }
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        f,
        0,
        Target::Object(t)
    ));
    assert_eq!(w.discs.len(), 1);
    for _ in 0..4 {
        tick::step(&mut w);
    }
    assert_eq!(w.discs.len(), 1, "the disc laser is still in play");
    // The killing hit: it brings the ship's health to 0 (S-0615 claim 14).
    assert_eq!(damage::apply(&mut w, t, lethal()), HitResult::NowDead);
    let sinking = naval::sinking(&w, t);
    w.effects.clear();
    let mut events = Vec::new();
    for _ in 0..16 {
        tick::step(&mut w);
        for e in &w.effects {
            if let Effect::Disc { event, .. } = e {
                events.push(event.clone());
            }
        }
        w.effects.clear();
    }
    (events, sinking, stats::of(&w, HouseId(0)).kills)
}

// spec: S-0615 claim 14 (Cancelled by loss: the hit that brings the target's health to 0 loses it at once, even when it then stays in play: a sinking object; When the loss applies: permanent, no later re-test)
#[test]
fn issue_313_a_sinking_target_cancels_the_disc_laser() {
    // Control: a ship too light to sink leaves play at its killing hit.
    let (events, sinking, kills) = sink_run("LIGHT");
    assert!(!sinking);
    assert_eq!(events, [DiscEvent::Cancelled], "control");
    assert_eq!(kills, 0, "control");
    // A ship that sinks stays in play at 1 health point.
    let (events, sinking, kills) = sink_run("HEAVY");
    assert!(sinking, "the heavy ship sinks");
    assert_eq!(
        (events, kills),
        (vec![DiscEvent::Cancelled], 0),
        "the disc laser is cancelled at the killing hit: no further step, no damage, no kill"
    );
}

const SIZE: i32 = 24;

fn open_rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("OPEN".into());
    r.sim.vehicles.insert(
        "OPEN".into(),
        UnitType {
            strength: Some(400),
            speed: Some(6),
            rot: Some(8),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            passengers: Some(4),
            size_limit: Some(Decimal("2".into())),
            open_topped: Some(true),
            ..Default::default()
        },
    );
    for w in r.sim.weapons.values_mut() {
        w.disk_laser = Some(true);
    }
    r
}

fn at(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

/// Counts the disc events of firer `p` over `n` ticks: (shots, hits,
/// cancelled).
fn count(w: &mut World, p: ObjectId, n: i32) -> (usize, usize, usize) {
    let (mut shots, mut hits, mut cancelled) = (0, 0, 0);
    for _ in 0..n {
        w.effects.clear();
        tick::step(w);
        for e in &w.effects {
            if let Effect::Disc { firer, event, .. } = e
                && *firer == p
            {
                match event {
                    DiscEvent::Shot => shots += 1,
                    DiscEvent::Hit { .. } => hits += 1,
                    DiscEvent::Cancelled => cancelled += 1,
                    DiscEvent::ChargeUp { .. } => {}
                }
            }
        }
    }
    (shots, hits, cancelled)
}

// spec: S-0615 claims 12 and 14 (a loss is an event after the shot, there is no later re-test; the open-topped factor applies to a disc laser's damage)
#[test]
fn issue_314_an_open_topped_passengers_disc_laser_lands() {
    let r = open_rules();
    // Control: the same infantry unit on foot, at the transport's cell.
    let mut w = testkit::game_with(&r, SIZE, 7);
    let p = at(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    let e = at(&mut w, Kind::Vehicle, "BUGGY", 1, 3, 0);
    let full = w.obj(e).unwrap().health;
    let (shots, hits, _) = count(&mut w, p, 120);
    assert!(shots > 0 && hits > 0, "on foot the disc laser lands");
    assert!(w.obj(e).is_none_or(|o| o.health < full));
    // The same unit riding in an open-topped transport.
    let mut w = testkit::game_with(&r, SIZE, 7);
    let t = at(&mut w, Kind::Vehicle, "OPEN", 0, 0, 0);
    let p = at(&mut w, Kind::Infantry, "WALKER", 0, 0, 3);
    let _e = at(&mut w, Kind::Vehicle, "BUGGY", 1, 3, 0);
    transport::board(&mut w, p, t);
    assert_eq!(transport::riding(&w, p), Some(t));
    assert!(!w.obj(p).unwrap().on_map);
    let (shots, hits, cancelled) = count(&mut w, p, 120);
    assert_eq!(transport::riding(&w, p), Some(t), "still riding");
    assert!(shots > 0, "the passenger fires");
    assert!(
        hits > 0,
        "the passenger's disc laser lands; got shots {shots}, hits {hits}, cancelled {cancelled}"
    );
}

// spec: S-0615 claim 14 (it enters any transport, an open-topped one too: lost at that moment, permanently)
#[test]
fn issue_314_boarding_after_the_shot_still_cancels() {
    let r = open_rules();
    let mut w = testkit::game_with(&r, SIZE, 7);
    let t = at(&mut w, Kind::Vehicle, "OPEN", 0, 0, 1);
    let p = at(&mut w, Kind::Infantry, "WALKER", 0, 0, 0);
    let e = at(&mut w, Kind::Vehicle, "BUGGY", 1, 3, 0);
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        p,
        0,
        Target::Object(e)
    ));
    assert_eq!(w.discs.len(), 1);
    tick::step(&mut w);
    transport::board(&mut w, p, t);
    w.effects.clear();
    tick::step(&mut w);
    let ev: Vec<&DiscEvent> = w
        .effects
        .iter()
        .filter_map(|x| match x {
            Effect::Disc { firer, event, .. } if *firer == p => Some(event),
            _ => None,
        })
        .collect();
    assert!(ev.contains(&&DiscEvent::Cancelled), "{ev:?}");
    assert!(w.discs.iter().all(|d| d.firer != p || d.updates == 0));
}
