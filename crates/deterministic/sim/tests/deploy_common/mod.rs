//! Shared fixtures of the S-0517 claim tests: the jumpjet ruleset plus
//! vehicles that deploy in place, a deploy-fire vehicle, deploy-to-fire
//! vehicles, a deployed infantry type and invented deploying presentations.
//! Every name and number is invented; no game data.
#![allow(dead_code)]

#[path = "../jump_common/mod.rs"]
mod jump_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    AnimationType, LandRow, LandType, LocomotionClass, Locomotor, SequenceEntry, UnitType,
};
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

pub use jump_common::{at, d, go, height, phase, put, step};

pub const SIZE: i32 = jump_common::SIZE;
pub const A: HouseId = HouseId(0);
pub const B: HouseId = HouseId(1);

fn tank() -> UnitType {
    UnitType {
        strength: Some(300),
        speed: Some(6),
        rot: Some(8),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        is_simple_deployer: Some(true),
        primary: Some("Cannon".into()),
        ..Default::default()
    }
}

fn anim(rate: i32, end: i32) -> AnimationType {
    AnimationType {
        rate: Some(rate),
        end: Some(end),
        start: Some(0),
        ..Default::default()
    }
}

/// The ruleset: TANK (in place, no presentation), ATANK (in place,
/// presentation DEPA with d = 2 and E = 5, deployed image IMGT), ZTANK
/// (presentation DEPZ: rate 0, E = 5), ETANK (presentation DEP1: E = 1),
/// GAS (deploy-fire without the in-place flag, weapon Gas in slot 0 and Gun
/// in slot 1), D2F (deploy-to-fire only), D2FIP (deploy-to-fire and in
/// place, presentation DEPA), DLANDA (a landing-first jumpjet, presentation
/// DEPA), DLAND0 (a landing-first jumpjet, no presentation), IMGT (a plain
/// vehicle that is only an image), DEPINF (deployed infantry that fires).
pub fn ruleset() -> Ruleset {
    let mut r = jump_common::ruleset();
    r.registries.get_mut("vehicles").unwrap().extend(
        [
            "TANK", "ATANK", "ZTANK", "ETANK", "GAS", "D2F", "D2FIP", "DLANDA", "DLAND0", "IMGT",
        ]
        .map(String::from),
    );
    r.registries
        .get_mut("infantry")
        .unwrap()
        .extend(["DEPINF", "DEPJET"].map(String::from));
    r.registries.insert(
        "animations".into(),
        ["DEPA", "DEPZ", "DEP1"].map(String::from).to_vec(),
    );
    let s = &mut r.sim;
    s.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            ..Default::default()
        },
    );
    s.animations.insert("DEPA".into(), anim(450, 5));
    s.animations.insert("DEPZ".into(), anim(0, 5));
    s.animations.insert("DEP1".into(), anim(450, 1));
    s.vehicles.insert("TANK".into(), tank());
    let mut a = tank();
    a.deploying_anim = Some("DEPA".into());
    a.unloading_class = Some("IMGT".into());
    s.vehicles.insert("ATANK".into(), a);
    let mut z = tank();
    z.deploying_anim = Some("DEPZ".into());
    s.vehicles.insert("ZTANK".into(), z);
    let mut e = tank();
    e.deploying_anim = Some("DEP1".into());
    s.vehicles.insert("ETANK".into(), e);
    let mut g = tank();
    g.is_simple_deployer = None;
    g.deploy_fire = Some(true);
    g.primary = Some("Gas".into());
    g.secondary = Some("Cannon".into());
    g.deploy_fire_weapon = Some(1);
    s.vehicles.insert("GAS".into(), g);
    let mut f = tank();
    f.is_simple_deployer = None;
    f.deploy_to_fire = Some(true);
    s.vehicles.insert("D2F".into(), f);
    let mut fi = tank();
    fi.deploy_to_fire = Some(true);
    fi.deploying_anim = Some("DEPA".into());
    s.vehicles.insert("D2FIP".into(), fi);
    let mut dl = s.vehicles["DEPL"].clone();
    dl.deploying_anim = Some("DEPA".into());
    s.vehicles.insert("DLANDA".into(), dl.clone());
    dl.deploying_anim = None;
    s.vehicles.insert("DLAND0".into(), dl);
    let mut img = tank();
    img.is_simple_deployer = None;
    img.primary = None;
    s.vehicles.insert("IMGT".into(), img);
    s.weapons.insert(
        "Gas".into(),
        glasnost_rules::sim::WeaponType {
            damage: Some(8),
            rof: Some(30),
            range: Some(d("4")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            omni_fire: Some(true),
            ..Default::default()
        },
    );
    // Deployed infantry that fires: deploy sequences and the deploy-fire flags.
    let mut inf = s.infantry["WALKER"].clone();
    inf.deployer = Some(true);
    inf.deploy_fire = Some(true);
    inf.undeploy_delay = Some(-1);
    inf.primary = Some("Rifle".into());
    inf.secondary = Some("Rifle".into());
    for (k, n) in [
        ("deploy", 4),
        ("deployed", 1),
        ("undeploy", 6),
        ("ready", 1),
        ("stand", 1),
        ("deployed_fire", 2),
    ] {
        inf.sequences.insert(
            k.into(),
            SequenceEntry {
                start: 0,
                count: n,
                multiplier: 0,
                direction: None,
            },
        );
    }
    s.infantry.insert("DEPINF".into(), inf.clone());
    let mut jet = s.infantry["JJ"].clone();
    jet.deployer = Some(true);
    jet.deploy_fire = Some(true);
    jet.undeploy_delay = Some(-1);
    jet.sequences = inf.sequences.clone();
    s.infantry.insert("DEPJET".into(), jet);
    r
}

pub fn game() -> World {
    let mut w = testkit::game_with(&ruleset(), SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

pub fn deploy(w: &mut World, id: ObjectId) {
    let house = w.obj(id).unwrap().owner.unwrap();
    orders::issue(w, house, Order::Deploy { actor: id });
}

pub fn behaviour(w: &World, id: ObjectId) -> (Option<Behaviour>, Option<Behaviour>) {
    let o = w.obj(id).unwrap();
    (o.beh.current, o.beh.queued)
}

/// The processing ticks of `id` (the tick a new delay was chosen) and the
/// delays, over `n` ticks of play.
pub fn processings(w: &mut World, id: ObjectId, n: i32) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    let mut prev = None;
    for _ in 0..n {
        step(w);
        let Some(o) = w.obj(id) else { break };
        if o.beh.last != prev {
            if let Some(l) = o.beh.last {
                out.push(l);
            }
            prev = o.beh.last;
        }
    }
    out
}

pub fn sounds(w: &World) -> Vec<Sound> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Sound { sound, .. } => Some(*sound),
            _ => None,
        })
        .collect()
}

pub fn anims(w: &World) -> Vec<(String, bool)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, reverse, .. } => Some((name.clone(), *reverse)),
            _ => None,
        })
        .collect()
}

pub fn kind_of(k: Kind) -> Kind {
    k
}
