//! S-0309 claim 16 and F8 for a killing hit (#173): the anger a hit gives
//! is the share of its strength the victim lost, whatever the kill
//! handling leaves of the victim, and crippling a cyborg (S-0302 claim 15)
//! never lowers it. Synthetic data, every name and number invented.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, UnitType, WarheadType};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::combat::data::WarheadId;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn vehicle(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.vehicles["BUGGY"].clone();
    t.strength = Some(1000);
    t.primary = None;
    t.combat.armor = None;
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.strength = Some(1000);
    t.primary = None;
    t.crawls = Some(true);
    for k in ["stand", "walk", "go_prone", "prone", "crawl"] {
        t.sequences.insert(
            k.into(),
            glasnost_rules::sim::SequenceEntry {
                start: 0,
                count: 4,
                multiplier: 0,
                direction: None,
            },
        );
    }
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    vehicle(&mut r, "BOX", |t| t.cost = Some(500));
    infantry(&mut r, "MAN", |t| t.cost = Some(500));
    infantry(&mut r, "BORG", |t| {
        t.cost = Some(500);
        t.cyborg = Some(true);
    });
    for (name, kind) in [("PLAIN", 0), ("DIE1", 1)] {
        r.sim.warheads.insert(
            name.into(),
            WarheadType {
                verses: Some(vec![Decimal("1".into()); 11]),
                inf_death: Some(kind),
                ..Default::default()
            },
        );
    }
    r
}

fn wh(w: &World, name: &str) -> WarheadId {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == name)
        .unwrap() as u32
}

fn by(a: ObjectId, amount: i32, h: WarheadId) -> Hit {
    Hit {
        amount,
        warhead: Some(h),
        distance: 0,
        attacker: Some(a),
        house: None,
        ignore_defences: false,
        no_escape: false,
    }
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, 0), 0)
}

fn anger(w: &World) -> i32 {
    w.houses[usize::from(BETA)].combat.anger[usize::from(ALPHA)]
}

/// Kills a fresh object of `ty` at 500 of 1000 health with one hit and
/// returns (result, still in the world, anger gained by the victim's owner).
fn kill(kind: Kind, ty: &str, warhead: &str) -> (HitResult, bool, i32) {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let h = wh(&w, warhead);
    let a = put(&mut w, Kind::Vehicle, "BOX", ALPHA, -8);
    let t = put(&mut w, kind, ty, BETA, 0);
    w.obj_mut(t).unwrap().health = 500;
    let before = anger(&w);
    let res = damage::apply(&mut w, t, by(a, 2000, h));
    (res, w.obj(t).is_some(), anger(&w) - before)
}

// spec: S-0309 claim 16 and F8 (Δ = conv((d / S) × C): 500 / 1000 × 500 = 250)
#[test]
fn a_nonlethal_hit_gives_anger() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let h = wh(&w, "PLAIN");
    let a = put(&mut w, Kind::Vehicle, "BOX", ALPHA, -8);
    let t = put(&mut w, Kind::Vehicle, "BOX", BETA, 0);
    damage::apply(&mut w, t, by(a, 500, h));
    assert_eq!(anger(&w), 250);
}

// spec: S-0309 claim 16 and F8 (d is the damage dealt, with no exception for a killing hit: the hit takes 500 of 1000 health, 250, whether the victim leaves the world at its kill or stays for a death sequence) and S-0302 F4 (m′ = min(m, H))
#[test]
fn a_killing_hit_gives_the_same_anger_whatever_stays_in_play() {
    let veh = kill(Kind::Vehicle, "BOX", "PLAIN");
    let inf_seq = kill(Kind::Infantry, "MAN", "DIE1");
    let inf_gone = kill(Kind::Infantry, "MAN", "PLAIN");
    assert_eq!(
        (veh.2, inf_seq.2, inf_gone.2),
        (250, 250, 250),
        "vehicle {veh:?}, infantry with a death sequence {inf_seq:?}, infantry removed at once {inf_gone:?}"
    );
}

// spec: S-0302 claims 11 and 15 (the hit takes the cyborg's health, then it is crippled at ⌊S / 4⌋ = 250) and S-0309 claim 16 and F8 (d is the damage dealt, 125 here: conv(125 / 1000 × 500) = conv(62.5) = 62, every step exact; negative only for a healing hit)
#[test]
fn crippling_a_cyborg_gives_the_anger_of_the_health_taken() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let h = wh(&w, "PLAIN");
    let a = put(&mut w, Kind::Vehicle, "BOX", ALPHA, -8);
    let t = put(&mut w, Kind::Infantry, "BORG", BETA, 0);
    w.obj_mut(t).unwrap().health = 125;
    w.houses[usize::from(BETA)].combat.anger[usize::from(ALPHA)] = 100;
    let res = damage::apply(&mut w, t, by(a, 2000, h));
    assert_eq!(res, HitResult::NowRed);
    assert_eq!(w.obj(t).unwrap().health, 250);
    assert_eq!(anger(&w), 162, "100 before, 62 from the hit");
}
