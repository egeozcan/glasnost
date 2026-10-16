//! Shared fixtures of the M6c structures claim tests (S-0508, S-0509,
//! S-0510): a synthetic ruleset with garrisonable buildings, occupier,
//! assaulter, engineer and spy infantry, and a game of houses Alpha (0),
//! Beta (1) and a neutral civilian-side house Civ (2). Every name and number
//! is invented; no game data.
#![allow(dead_code)]

use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_map::triggers::{Action, Event, Owner, TagDef, TriggerDef, Triggers};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    ArmourClass, BuildingType, Country, Decimal, LocomotionClass, Locomotor, UnitType, WeaponType,
};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, HouseRole, HouseSetup, ObjectId, Target, World};

pub const SIZE: i32 = 24;
pub const A: HouseId = HouseId(0);
pub const B: HouseId = HouseId(1);
pub const CIV: HouseId = HouseId(2);

pub fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

pub fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn building(w: i32, h: i32) -> BuildingType {
    BuildingType {
        strength: Some(400),
        foundation: Some([w, h]),
        ..Default::default()
    }
}

fn hut(w: i32, h: i32, capacity: i32, tech: Option<i32>) -> BuildingType {
    let mut b = building(w, h);
    b.can_be_occupied = Some(true);
    b.can_occupy_fire = Some(true);
    b.max_number_occupants = Some(capacity);
    b.tech_level = tech;
    b.combat.armor = Some(ArmourClass::Wood);
    // Not trainable: the experience of its kills goes to its occupant (S-0311 claim 8).
    b.combat.trainable = Some(false);
    // A build-up sprite, so that the building can be sold (S-0408 claim 4).
    b.econ.buildup = Some(true);
    b.econ.buildup_frames = Some(4);
    b
}

fn walker(strength: i32) -> UnitType {
    UnitType {
        strength: Some(strength),
        speed: Some(6),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        crushable: Some(true),
        ..Default::default()
    }
}

/// The ruleset of the garrison tests.
///
/// Buildings: HUT (2 × 2, civilian tech level −1, capacity 3), HUT1 (1 × 1,
/// −1, capacity 3), HUT3 (3 × 2, −1, capacity 3), MIL (2 × 2, tech level 5,
/// capacity 3), NEG (1 × 1, −1, capacity −1), ZERO (1 × 1, −1, capacity 0),
/// PLAIN (not garrisonable). Infantry: GARRI (occupier, occupying weapon
/// OccGun), ELITE (occupier with an elite occupying weapon), BARE (occupier
/// without an occupying weapon, primary Rifle), ASSAULT (assaulter), BOTH
/// (both marks) and the testkit's WALKER. Country Civland is of the
/// civilian side and passive.
pub fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries.insert(
        "buildings".into(),
        [
            "HUT", "HUT1", "HUT3", "MIL", "NEG", "ZERO", "PLAIN", "TOWER",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
    );
    for n in ["GARRI", "ELITE", "BARE", "ASSAULT", "BOTH"] {
        r.registries
            .get_mut("infantry")
            .unwrap()
            .push(n.to_string());
    }
    r.registries
        .get_mut("countries")
        .unwrap()
        .push("Civland".to_string());
    let s = &mut r.sim;
    s.weapons.insert(
        "OccGun".into(),
        WeaponType {
            damage: Some(20),
            rof: Some(10),
            range: Some(d("3")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "EliteOccGun".into(),
        WeaponType {
            damage: Some(35),
            rof: Some(10),
            range: Some(d("3")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Beam".into(),
        WeaponType {
            damage: Some(8),
            rof: Some(10),
            range: Some(d("3")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            disk_laser: Some(true),
            ..Default::default()
        },
    );
    s.buildings.insert("HUT".into(), hut(2, 2, 3, Some(-1)));
    s.buildings.insert("HUT1".into(), hut(1, 1, 3, Some(-1)));
    s.buildings.insert("HUT3".into(), hut(3, 2, 3, Some(-1)));
    s.buildings.insert("MIL".into(), hut(2, 2, 3, Some(5)));
    s.buildings.insert("NEG".into(), hut(1, 1, -1, Some(-1)));
    s.buildings.insert("ZERO".into(), hut(1, 1, 0, Some(-1)));
    let mut plain = building(1, 1);
    plain.tech_level = Some(-1);
    s.buildings.insert("PLAIN".into(), plain);
    let occupier = |occ: Option<&str>, elite: Option<&str>| {
        let mut u = walker(100);
        u.occupier = Some(true);
        u.primary = Some("Rifle".into());
        u.occupy_weapon = occ.map(Into::into);
        u.elite_occupy_weapon = elite.map(Into::into);
        u.combat.trainable = Some(true);
        u
    };
    s.infantry
        .insert("GARRI".into(), occupier(Some("OccGun"), None));
    s.infantry.insert(
        "ELITE".into(),
        occupier(Some("OccGun"), Some("EliteOccGun")),
    );
    s.infantry.insert("BARE".into(), occupier(None, None));
    let mut assault = walker(100);
    assault.assaulter = Some(true);
    assault.primary = Some("Rifle".into());
    s.infantry.insert("ASSAULT".into(), assault);
    let mut both = walker(100);
    both.assaulter = Some(true);
    both.occupier = Some(true);
    both.occupy_weapon = Some("OccGun".into());
    s.infantry.insert("BOTH".into(), both);
    if let Some(u) = s.infantry.get_mut("WALKER") {
        u.combat.trainable = Some(true);
    }
    s.countries.insert(
        "Civland".into(),
        Country {
            multiplay_passive: Some(true),
            econ: glasnost_rules::sim_econ::CountryEcon {
                side: Some("Civilian".into()),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    s.general.condition_red = Some(d("0.5"));
    s.general.occupy_rof_multiplier = Some(d("1"));
    s.general.occupy_damage_multiplier = Some(d("1"));
    s.general.occupy_weapon_range = Some(2);
    r
}

/// Triggers: the testkit's entry triggers (ENT counts entry raises into
/// variable 0, DIE counts destroyed raises into variable 1).
pub fn triggers() -> Triggers {
    testkit::entry_triggers()
}

/// A game on `r` with the neutral civilian-side house Civ (house 2), the
/// houses Alpha and Beta not allied, on an empty `SIZE` map, with `t` as
/// the map's trigger data, run until orders are allowed.
pub fn game_on(r: &Ruleset, t: Triggers) -> World {
    let mut map = testkit::empty_map(SIZE);
    map.triggers = t;
    let mut s = testkit::setup(7);
    s.houses.push(HouseSetup {
        name: "Civ".into(),
        country: "Civland".into(),
        human: false,
        start: None,
        role: HouseRole::Neutral,
        ai: false,
    });
    let rules = Arc::new(SimRules::new(r, "synthetic-test"));
    let mut w = glasnost_sim::load::new_game(rules, &map, testkit::catalogue(), &s)
        .expect("synthetic test game loads");
    testkit::run(&mut w, 4);
    w
}

pub fn game() -> World {
    game_on(&ruleset(), Triggers::default())
}

pub fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

/// Sends infantry `unit` to capture (garrison, enter or infiltrate)
/// building `b`: the order of S-0214 table M (a behaviour order, *capture*
/// with the building as destination).
pub fn send(w: &mut World, unit: ObjectId, b: ObjectId) {
    let house = w.obj(unit).unwrap().owner.unwrap();
    orders::issue(
        w,
        house,
        Order::Behaviour {
            actor: unit,
            behaviour: Behaviour::Capture,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        },
    );
}

/// Runs until `f` holds, at most `max` ticks; the tick it first held in.
pub fn until(w: &mut World, max: i32, mut f: impl FnMut(&World) -> bool) -> Option<i32> {
    for _ in 0..max {
        testkit::run(w, 1);
        if f(w) {
            return Some(w.tick);
        }
    }
    None
}

pub fn on_map(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.on_map)
}

pub fn owner(w: &World, id: ObjectId) -> Option<HouseId> {
    w.obj(id).and_then(|o| o.owner)
}

pub fn occ(w: &World, b: ObjectId) -> Vec<ObjectId> {
    glasnost_sim::structures::garrison::occupants(w, b)
}

/// Tags `id` with the trigger tag named `name`.
pub fn tag(w: &mut World, id: ObjectId, name: &str) {
    glasnost_sim::triggers::object_tag(w, id, Some(name.as_bytes()));
}

/// A trigger definition with one event and one action.
pub fn trigger(id: &str, event: Event, actions: Vec<Action>) -> TriggerDef {
    TriggerDef {
        owner: Owner::None,
        events: vec![event],
        actions,
        ..TriggerDef::created(id.as_bytes().to_vec())
    }
}

pub fn tag_def(i: u32, t: &TriggerDef, repeat: i32) -> TagDef {
    TagDef {
        id: t.id.clone(),
        repeat,
        name: t.id.clone(),
        first: Some(i),
        chain: vec![i],
    }
}
