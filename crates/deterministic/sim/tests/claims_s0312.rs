//! Claim tests for S-0312 (death and destruction) and S-0107 claim 17
//! (infantry deaths) on synthetic data: invented types, weapons and
//! animation names (none of them a registered animation type, so creating
//! one takes no draw of its own). Kills are delivered directly through
//! damage resolution, and the draws of a death are checked by replaying
//! the spec's draw list on a copy of the random stream.

use glasnost_num::F32;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, CombatFields, Decimal, LocomotionClass, Locomotor, SequenceEntry, UnitType,
    VoxelAnimationType, WarheadType, WeaponType,
};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::combat::death;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::infantry::Seq;
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, World};

const SIZE: i32 = 24;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn ones() -> Option<Vec<Decimal>> {
    Some((0..11).map(|_| Decimal("1".into())).collect())
}

fn names(v: &[&str]) -> Option<Vec<String>> {
    Some(v.iter().map(|s| s.to_string()).collect())
}

fn vehicle(strength: i32) -> UnitType {
    UnitType {
        strength: Some(strength),
        speed: Some(5),
        rot: Some(5),
        cost: Some(600),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    }
}

fn seq(count: i32) -> SequenceEntry {
    SequenceEntry {
        start: 0,
        count,
        multiplier: 0,
        direction: None,
    }
}

fn soldier(strength: i32) -> UnitType {
    let mut t = UnitType {
        strength: Some(strength),
        speed: Some(4),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        ..Default::default()
    };
    for k in [
        "stand", "walk", "go_prone", "prone", "crawl", "get_up", "die_1", "die_2",
    ] {
        t.sequences.insert(k.into(), seq(4));
    }
    t
}

/// The combat ruleset with the types of these tests: unarmed vehicles
/// VICTIM (strength 200) and NEIGH (strength 100), infantry TECH
/// (strength 60) and SOLDIER, building HUT (3 × 3), aircraft PLANE,
/// weapons Boom (500, warhead BLAST of spread 2) and warheads W1, W2, W4,
/// W9 of those infantry-death kinds and DK (delay kill).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    for n in ["VICTIM", "NEIGH"] {
        push(&mut r, "vehicles", n);
    }
    for n in ["TECH", "SOLDIER"] {
        push(&mut r, "infantry", n);
    }
    for n in ["HUT", "PAD", "FENCE"] {
        push(&mut r, "buildings", n);
    }
    push(&mut r, "aircraft", "PLANE");
    let s = &mut r.sim;
    s.vehicles.insert("VICTIM".into(), vehicle(200));
    s.vehicles.insert("NEIGH".into(), vehicle(100));
    s.infantry.insert("TECH".into(), soldier(60));
    s.infantry.insert("SOLDIER".into(), soldier(100));
    s.aircraft.insert(
        "PLANE".into(),
        UnitType {
            strength: Some(150),
            ..Default::default()
        },
    );
    let building = |w: i32, h: i32| BuildingType {
        strength: Some(500),
        foundation: Some([w, h]),
        ..Default::default()
    };
    s.buildings.insert("HUT".into(), building(3, 3));
    s.buildings.insert(
        "PAD".into(),
        BuildingType {
            helipad: Some(true),
            ..building(2, 2)
        },
    );
    s.buildings.insert(
        "FENCE".into(),
        BuildingType {
            laser_fence: Some(true),
            ..building(1, 1)
        },
    );
    s.weapons.insert(
        "Boom".into(),
        WeaponType {
            damage: Some(500),
            rof: Some(30),
            range: dec("2"),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("BLAST".into()),
            ..Default::default()
        },
    );
    s.weapons.insert(
        "Dud".into(),
        WeaponType {
            damage: Some(20),
            rof: Some(30),
            range: dec("2"),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            suicide: Some(true),
            ..Default::default()
        },
    );
    s.warheads.insert(
        "BLAST".into(),
        WarheadType {
            cell_spread: dec("2"),
            percent_at_max: dec("1"),
            verses: ones(),
            ..Default::default()
        },
    );
    for (n, k) in [("W1", 1), ("W2", 2), ("W4", 4), ("W9", 9)] {
        s.warheads.insert(
            n.into(),
            WarheadType {
                verses: ones(),
                inf_death: Some(k),
                ..Default::default()
            },
        );
    }
    s.warheads.insert(
        "DK".into(),
        WarheadType {
            verses: ones(),
            cell_spread: dec("2"),
            causes_delay_kill: Some(true),
            delay_kill_frames: Some(5),
            delay_kill_at_max: dec("7"),
            ..Default::default()
        },
    );
    r
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 11)
}

fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

fn warhead(r: &Ruleset, name: &str) -> u32 {
    r.sim.warheads.keys().position(|k| k == name).unwrap() as u32
}

/// A lethal hit that ignores defences, with no attacker.
fn lethal() -> Hit {
    Hit {
        amount: 100_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    }
}

fn kill(w: &mut World, id: ObjectId) -> HitResult {
    damage::apply(w, id, lethal())
}

/// A hit of `amount` with warhead `h` that does not ignore defences.
fn hit_with(h: u32, amount: i32) -> Hit {
    Hit {
        amount,
        warhead: Some(h),
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    }
}

fn anims(w: &World) -> Vec<(String, Pos)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, pos, .. } => Some((name.clone(), *pos)),
            _ => None,
        })
        .collect()
}

fn detonations(w: &World) -> Vec<i32> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Detonation { damage, .. } => Some(*damage),
            _ => None,
        })
        .collect()
}

fn half(w: &World) -> bool {
    w.rules().quirks.ranged_draw_half_span
}

fn set_exp(w: &mut World, id: ObjectId, bits: u32) {
    w.obj_mut(id).unwrap().combat.experience = F32(bits);
}

const ELITE: u32 = 0x4000_0000; // 2.0
const VETERAN: u32 = 0x3F80_0000; // 1.0

// ---------------------------------------------------------------- claim 1

// spec: S-0312 claim 1 (death effects come at once inside damage resolution: an exploding vehicle's death weapon kills a neighbour within the same hit; scenario D1)
#[test]
fn claim_1_death_weapon_kills_within_the_same_hit() {
    let mut r = rules();
    let v = r.sim.vehicles.get_mut("VICTIM").unwrap();
    v.combat.explodes = Some(true);
    v.combat.death_weapon = Some("Boom".into());
    let mut w = game(&r);
    let a = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let b = spawn(&mut w, Kind::Vehicle, "NEIGH", 1, 1, 0);
    let t = w.tick;
    assert_eq!(kill(&mut w, a), HitResult::NowDead);
    assert_eq!(w.tick, t);
    assert!(w.obj(a).is_none());
    assert!(w.obj(b).is_none());
}

// ---------------------------------------------------------------- claim 2

fn cyborg_rules(strength: i32) -> Ruleset {
    let mut r = rules();
    let mut c = soldier(strength);
    c.cyborg = Some(true);
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("BORG".into());
    r.sim.infantry.insert("BORG".into(), c);
    r.sim.general.infantry_explode = Some("BORGX".into());
    r
}

// spec: S-0312 claim 2 (a cyborg that is not prone is crippled: the infantry-explosion animation, health max(1, trunc(H × 0.25)), made to go prone; scenario D2)
#[test]
fn claim_2_cyborg_crippled() {
    let r = cyborg_rules(200);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Infantry, "BORG", 0, 0, 0);
    w.effects.clear();
    let res = damage::apply(&mut w, id, hit_with(warhead(&r, "W1"), 1000));
    assert_eq!(res, HitResult::NowRed);
    let o = w.obj(id).unwrap();
    assert_eq!(o.health, 50);
    assert!(!o.combat.killed);
    let st = o.infantry.as_ref().unwrap();
    assert!(st.prone);
    assert_eq!(st.seq, Some(Seq::GoProne));
    assert_eq!(anims(&w)[0].0, "BORGX");
    // The second lethal hit kills the prone cyborg, with no death sequence (S-0107 claim 17 row 2).
    let res = damage::apply(&mut w, id, hit_with(warhead(&r, "W1"), 1000));
    assert_eq!(res, HitResult::NowDead);
    assert!(w.obj(id).is_none());
}

// spec: S-0312 claim 2 (D2 variant b: strength 2 leaves 1 health)
#[test]
fn claim_2_cyborg_crippled_minimum_one() {
    let r = cyborg_rules(2);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Infantry, "BORG", 0, 0, 0);
    damage::apply(&mut w, id, hit_with(warhead(&r, "W1"), 1000));
    assert_eq!(w.obj(id).unwrap().health, 1);
}

// spec: S-0312 claim 2 (a prone cyborg, or one hit with damage that ignores defences, dies normally; D2 variant c)
#[test]
fn claim_2_prone_or_ignoring_defences_kills() {
    let r = cyborg_rules(200);
    let mut w = game(&r);
    let a = spawn(&mut w, Kind::Infantry, "BORG", 0, 0, 0);
    w.obj_mut(a).unwrap().infantry.as_mut().unwrap().prone = true;
    assert_eq!(
        damage::apply(&mut w, a, hit_with(warhead(&r, "W1"), 1000)),
        HitResult::NowDead
    );
    let b = spawn(&mut w, Kind::Infantry, "BORG", 0, 2, 0);
    assert_eq!(kill(&mut w, b), HitResult::NowDead);
    assert!(w.obj(b).is_none_or(|o| o.combat.killed));
}

// ---------------------------------------------------------------- claim 3, F6

fn delay_rules() -> Ruleset {
    let mut r = rules();
    r.sim
        .buildings
        .get_mut("HUT")
        .unwrap()
        .eligible_for_delay_kill = Some(true);
    r
}

fn delay_hit(r: &Ruleset, distance: i32) -> Hit {
    Hit {
        distance,
        ..hit_with(warhead(r, "DK"), 10_000)
    }
}

// spec: S-0312 claim 3 and F6 (a delay-kill warhead keeps the building at 1 health and times its destruction T ticks later: F = 5, A = 7, spread 2 gives 5 at distance 0 and trunc(5 + 30 × 256 / 512) = 20 at 256; scenario D3)
#[test]
fn claim_3_delayed_building_death() {
    let r = delay_rules();
    for (d, t) in [(0, 5), (256, 20)] {
        let mut w = game(&r);
        let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
        let now = w.tick;
        assert_eq!(
            damage::apply(&mut w, id, delay_hit(&r, d)),
            HitResult::PostMortem
        );
        let o = w.obj(id).unwrap();
        assert_eq!(o.health, 1);
        assert_eq!(o.combat.delay_kill_until, Some(now + t));
    }
}

// spec: S-0312 claim 3 (D3 variant b: a running delay with at most T ticks left is kept; a shorter new delay replaces it)
#[test]
fn claim_3_shorter_delay_wins() {
    let r = delay_rules();
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
    let t0 = w.tick;
    damage::apply(&mut w, id, delay_hit(&r, 256));
    assert_eq!(w.obj(id).unwrap().combat.delay_kill_until, Some(t0 + 20));
    testkit::run(&mut w, 3);
    let t1 = w.tick;
    damage::apply(&mut w, id, delay_hit(&r, 0));
    assert_eq!(w.obj(id).unwrap().combat.delay_kill_until, Some(t1 + 5));
    // The reverse order keeps the first delay of 5.
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
    let t0 = w.tick;
    damage::apply(&mut w, id, delay_hit(&r, 0));
    testkit::run(&mut w, 1);
    damage::apply(&mut w, id, delay_hit(&r, 256));
    assert_eq!(w.obj(id).unwrap().combat.delay_kill_until, Some(t0 + 5));
}

// spec: S-0312 claim 3 (a building of a type that is not eligible, or killed by another warhead, dies at once)
#[test]
fn claim_3_other_deaths_are_immediate() {
    let r = rules();
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
    assert_eq!(
        damage::apply(&mut w, id, delay_hit(&r, 0)),
        HitResult::NowDead
    );
    assert!(w.obj(id).is_none());
    let r = delay_rules();
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
    assert_eq!(kill(&mut w, id), HitResult::NowDead);
    assert!(w.obj(id).is_none());
}

// spec: S-0312 claim 3 (a building in a delayed-death countdown is destroyed at once by a hit of another warhead, with its death effects: S-0302 claim 16)
#[test]
fn claim_3_other_warhead_ends_the_countdown() {
    let r = delay_rules();
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
    damage::apply(&mut w, id, delay_hit(&r, 0));
    assert!(w.obj(id).is_some());
    let res = damage::apply(&mut w, id, hit_with(warhead(&r, "W1"), 10));
    assert_eq!(res, HitResult::NowDead);
    assert!(w.obj(id).is_none());
}

/// Kills, score and countdown end after two delayed-kill hits by house 1's
/// NEIGH on house 0's HUT, at distances 256 and then 0.
fn credited_again(on: bool) -> (i32, i32, Option<i32>, i32) {
    let mut r = delay_rules();
    r.globals.quirks.delay_kill_credited_again = on;
    r.sim.buildings.get_mut("HUT").unwrap().cost = Some(300);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
    let a = spawn(&mut w, Kind::Vehicle, "NEIGH", 1, 6, 0);
    let hit = |d| Hit {
        attacker: Some(a),
        ..delay_hit(&r, d)
    };
    damage::apply(&mut w, id, hit(256));
    testkit::run(&mut w, 1);
    let now = w.tick;
    damage::apply(&mut w, id, hit(0));
    let h = &w.houses[1].combat;
    (
        h.kills,
        h.score,
        w.obj(id).unwrap().combat.delay_kill_until,
        now,
    )
}

// spec: S-0312 claim 3 and S-0302 claim 16 (quirks.delay_kill_credited_again on: a further delayed-kill hit credits the destruction again; off: no further credit; in both settings the countdown is shortened)
#[test]
fn quirk_delay_kill_credited_again_both_settings() {
    let (kills, score, until, now) = credited_again(true);
    assert_eq!((kills, score), (2, 600));
    assert_eq!(until, Some(now + 5));
    let (kills, score, until, now) = credited_again(false);
    assert_eq!((kills, score), (1, 300));
    assert_eq!(until, Some(now + 5));
}

// spec: S-0312 Edge cases (quirks.delay_kill_whole_cell_spread on: a spread below one cell gives T = 0; off: R of S-0302 F3 gives a countdown rising from F)
#[test]
fn quirk_delay_kill_whole_cell_spread_both_settings() {
    for (on, expected) in [(true, 0), (false, 5 + 30 * 64 / 128)] {
        let mut r = delay_rules();
        r.globals.quirks.delay_kill_whole_cell_spread = on;
        r.sim.warheads.get_mut("DK").unwrap().cell_spread = dec("0.5");
        let mut w = game(&r);
        let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
        let now = w.tick;
        damage::apply(&mut w, id, delay_hit(&r, 64));
        assert_eq!(
            w.obj(id).unwrap().combat.delay_kill_until,
            Some(now + expected)
        );
    }
}

// ---------------------------------------------------------------- claim 4, F1

// spec: S-0312 claim 4 and F1 (N = R(Y, X − 1); with debris animations, N animations each of entry R(0, c − 1), at the position raised by 20 leptons)
#[test]
fn f1_debris_animations() {
    let mut r = rules();
    let c = &mut r.sim.vehicles.get_mut("VICTIM").unwrap().combat;
    c.min_debris = Some(1);
    c.max_debris = Some(5);
    c.debris_anims = names(&["DA", "DB", "DC"]);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let pos = w.obj(id).unwrap().pos;
    w.effects.clear();
    let h = half(&w);
    let mut rr: Rng = w.rng.clone();
    let n = rr.ranged(1, 4, h);
    let expect: Vec<(String, Pos)> = (0..n)
        .map(|_| {
            let k = rr.ranged(0, 2, h) as usize;
            (
                ["DA", "DB", "DC"][k].to_string(),
                Pos::new(pos.x, pos.y, pos.z + 20),
            )
        })
        .collect();
    kill(&mut w, id);
    assert_eq!(anims(&w), expect);
    assert_eq!(w.rng, rr);
}

// spec: S-0312 F1 (voxel debris types with maxima are dealt as S-0303 F6: one raw value per turn, |raw| mod (u + 1), capped at the pieces left; at the object's position) and claim 4 with S-0317 claims 1 and 3 (each a piece of S-0317 with the object's house, made with its creation draws as it is dealt)
#[test]
fn f1_voxel_debris_dealt() {
    let mut r = rules();
    let c = &mut r.sim.vehicles.get_mut("VICTIM").unwrap().combat;
    c.min_debris = Some(3);
    c.max_debris = Some(7);
    c.debris_types = names(&["VX1", "VX2"]);
    c.debris_maximums = Some(vec![2, 1]);
    for n in ["VX1", "VX2"] {
        r.registries
            .entry("voxel_animations".into())
            .or_default()
            .push(n.into());
        r.sim
            .voxel_animations
            .insert(n.into(), VoxelAnimationType::default());
    }
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let pos = w.obj(id).unwrap().pos;
    let owner = w.obj(id).unwrap().owner;
    w.effects.clear();
    let h = half(&w);
    let mut rr = w.rng.clone();
    let mut left = rr.ranged(3, 6, h);
    let maxima = [2, 1];
    let mut expect = Vec::new();
    let mut j = 0;
    while left > 0 {
        let i = j % 2;
        let raw = rr.raw() as i32;
        let g = (raw.wrapping_abs() % (maxima[i] + 1)).min(left);
        for _ in 0..g {
            expect.push((["VX1", "VX2"][i].to_string(), pos));
            // The piece's creation draws (S-0317 claim 3 and F5): four raw
            // values, then three ranged draws.
            for _ in 0..4 {
                rr.raw();
            }
            for _ in 0..3 {
                rr.ranged(-65535, 65535, h); // spec: S-0317 F5 and Constants (R(−65535, 65535))
            }
        }
        left -= g;
        j += 1;
    }
    kill(&mut w, id);
    let types = &w.rules().pieces.types;
    let got: Vec<(String, Pos)> = w
        .pieces
        .values()
        .map(|p| (types[p.type_index as usize].name.clone(), p.pos))
        .collect();
    assert_eq!(got, expect);
    assert!(w.pieces.values().all(|p| p.house == owner));
    assert!(anims(&w).is_empty(), "no debris animation");
    assert_eq!(w.rng, rr);
}

// spec: S-0312 F1 (no debris types or animations: N animations from the game-wide metallic debris list, at the raised point)
#[test]
fn f1_metallic_debris() {
    let mut r = rules();
    let c = &mut r.sim.vehicles.get_mut("VICTIM").unwrap().combat;
    c.min_debris = Some(2);
    c.max_debris = Some(3);
    r.sim.general.metallic_debris = names(&["M1", "M2"]);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let pos = w.obj(id).unwrap().pos;
    w.effects.clear();
    let h = half(&w);
    let mut rr = w.rng.clone();
    let expect: Vec<(String, Pos)> = (0..2)
        .map(|_| {
            (
                ["M1", "M2"][rr.ranged(0, 1, h) as usize].to_string(),
                Pos::new(pos.x, pos.y, pos.z + 20),
            )
        })
        .collect();
    kill(&mut w, id);
    assert_eq!(anims(&w), expect);
    assert_eq!(w.rng, rr);
}

// spec: S-0312 Edge cases (departure: an empty metallic debris list takes the draws R(0, −1) and creates no debris)
#[test]
fn edge_empty_metallic_list_takes_draws() {
    let mut r = rules();
    let c = &mut r.sim.vehicles.get_mut("VICTIM").unwrap().combat;
    c.min_debris = Some(2);
    c.max_debris = Some(3);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    w.effects.clear();
    let h = half(&w);
    let mut rr = w.rng.clone();
    rr.ranged(0, -1, h);
    rr.ranged(0, -1, h);
    kill(&mut w, id);
    assert!(anims(&w).is_empty());
    assert_eq!(w.rng, rr);
}

// spec: S-0312 Edge cases (departure: debris that can never be placed, every maximum 0, stops after one full pass that places nothing)
#[test]
fn edge_debris_never_placed_stops_after_one_pass() {
    let mut r = rules();
    let c = &mut r.sim.vehicles.get_mut("VICTIM").unwrap().combat;
    c.min_debris = Some(3);
    c.max_debris = Some(4);
    c.debris_types = names(&["VX1", "VX2"]);
    c.debris_maximums = Some(vec![0]);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    w.effects.clear();
    let before = w.rng.taken();
    kill(&mut w, id);
    assert!(anims(&w).is_empty());
    assert_eq!(w.rng.taken() - before, 2);
}

// spec: S-0312 claim 4 (an object whose type has no positive maximum debris count throws none and takes no draw)
#[test]
fn claim_4_no_debris_no_draw() {
    let r = rules();
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let before = w.rng.clone();
    kill(&mut w, id);
    assert_eq!(w.rng, before);
}

// ---------------------------------------------------------------- claim 6, F3

/// The death-weapon damage of a killed VICTIM after `edit`.
fn death_detonations(edit: impl FnOnce(&mut Ruleset), exp: u32) -> Vec<i32> {
    let mut r = rules();
    edit(&mut r);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    set_exp(&mut w, id, exp);
    w.effects.clear();
    kill(&mut w, id);
    detonations(&w)
}

// spec: S-0312 claim 6 and F3 (the type's own death weapon: D = trunc(RZ₅₃(w × φ)))
#[test]
fn f3_own_death_weapon() {
    let d = death_detonations(
        |r| {
            let c = &mut r.sim.vehicles.get_mut("VICTIM").unwrap().combat;
            c.explodes = Some(true);
            c.death_weapon = Some("Cannon".into());
            c.death_weapon_damage_modifier = dec("0.5");
        },
        0,
    );
    assert_eq!(d, vec![27]);
}

// spec: S-0312 claim 6 and F3 (no own death weapon: the current primary weapon, with the factor φ applied too)
#[test]
fn f3_primary_weapon_with_factor() {
    let d = death_detonations(
        |r| {
            let v = r.sim.vehicles.get_mut("VICTIM").unwrap();
            v.primary = Some("Gun".into());
            v.combat.explodes = Some(true);
            v.combat.death_weapon_damage_modifier = dec("0.5");
        },
        0,
    );
    assert_eq!(d, vec![15]);
}

// spec: S-0312 claim 6 (the primary weapon is the elite weapon of the first slot when elite)
#[test]
fn claim_6_elite_primary_as_death_weapon() {
    let edit = |r: &mut Ruleset| {
        let v = r.sim.vehicles.get_mut("VICTIM").unwrap();
        v.primary = Some("Gun".into());
        v.elite_primary = Some("Cannon".into());
        v.combat.explodes = Some(true);
    };
    assert_eq!(death_detonations(edit, 0), vec![30]);
    assert_eq!(death_detonations(edit, ELITE), vec![55]);
}

// spec: S-0312 claim 6 and F3 (no own or primary weapon: the game-wide death weapon with D = trunc(RZ₅₃(H × 0.5)))
#[test]
fn f3_game_wide_death_weapon() {
    let d = death_detonations(
        |r| {
            r.sim.vehicles.get_mut("VICTIM").unwrap().combat.explodes = Some(true);
            r.sim.general.death_weapon = Some("Rifle".into());
        },
        0,
    );
    assert_eq!(d, vec![100]);
}

// spec: S-0312 claim 6 (an object that is not exploding fires no death weapon at death)
#[test]
fn claim_6_not_exploding_fires_nothing() {
    let d = death_detonations(
        |r| {
            let v = r.sim.vehicles.get_mut("VICTIM").unwrap();
            v.primary = Some("Gun".into());
            v.combat.death_weapon = Some("Cannon".into());
            r.sim.general.death_weapon = Some("Rifle".into());
        },
        0,
    );
    assert!(d.is_empty());
}

// spec: S-0312 Terms (exploding object: the explodes flag, the explodes ability of its rank (S-0311 claim 3), or a current weapon with the suicide flag)
#[test]
fn terms_exploding_object() {
    let ability = |r: &mut Ruleset| {
        let v = r.sim.vehicles.get_mut("VICTIM").unwrap();
        v.elite_abilities = vec!["EXPLODES".into()];
        v.combat.death_weapon = Some("Cannon".into());
    };
    assert!(death_detonations(ability, VETERAN).is_empty());
    assert_eq!(death_detonations(ability, ELITE), vec![55]);
    let suicide = |r: &mut Ruleset| {
        r.sim.vehicles.get_mut("VICTIM").unwrap().primary = Some("Dud".into());
    };
    assert_eq!(death_detonations(suicide, 0), vec![20]);
}

// spec: S-0312 claim 6 (the dying object is the detonation's source and is not hit by it) and claim 1 (it is credited to nobody: no attacker)
#[test]
fn claim_6_dying_object_not_hit_and_neighbour_damaged() {
    let mut r = rules();
    let v = r.sim.vehicles.get_mut("VICTIM").unwrap();
    v.combat.explodes = Some(true);
    v.combat.death_weapon = Some("Boom".into());
    v.combat.death_weapon_damage_modifier = dec("0.1");
    r.sim.vehicles.get_mut("NEIGH").unwrap().strength = Some(1000);
    let mut w = game(&r);
    let a = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let b = spawn(&mut w, Kind::Vehicle, "NEIGH", 1, 1, 0);
    w.effects.clear();
    kill(&mut w, a);
    assert_eq!(detonations(&w), vec![50]);
    let hb = w.obj(b).unwrap().health;
    assert!((950..1000).contains(&hb));
}

/// Anims of a VICTIM's death whose death weapon's warhead throws metallic
/// debris, with `quirks.death_weapon_unplaced_projectile` set to `on`.
fn death_weapon_debris(on: bool) -> (Pos, Vec<(String, Pos)>) {
    let mut r = rules();
    r.globals.quirks.death_weapon_unplaced_projectile = on;
    let v = r.sim.vehicles.get_mut("VICTIM").unwrap();
    v.combat.explodes = Some(true);
    v.combat.death_weapon = Some("Boom".into());
    let b = r.sim.warheads.get_mut("BLAST").unwrap();
    b.min_debris = Some(1);
    b.max_debris = Some(2);
    r.sim.general.metallic_debris = names(&["WD"]);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let pos = w.obj(id).unwrap().pos;
    w.effects.clear();
    kill(&mut w, id);
    (pos, anims(&w).into_iter().filter(|a| a.0 == "WD").collect())
}

// spec: S-0312 Edge cases (quirks.death_weapon_unplaced_projectile on: the warhead debris of S-0303 claim 22 uses the unplaced projectile's default position; off: the dying object's position)
#[test]
fn quirk_death_weapon_unplaced_projectile_both_settings() {
    let (_, on) = death_weapon_debris(true);
    assert_eq!(on, vec![("WD".to_string(), Pos::new(0, 0, 20))]);
    let (pos, off) = death_weapon_debris(false);
    assert_eq!(
        off,
        vec![("WD".to_string(), Pos::new(pos.x, pos.y, pos.z + 20))]
    );
}

// ---------------------------------------------------------------- claims 9, 15

fn listed_victim(r: &mut Ruleset) {
    let c = &mut r.sim.vehicles.get_mut("VICTIM").unwrap().combat;
    c.explosion = names(&["EX1", "EX2", "EX3"]);
    c.destroy_anim = names(&["DS1", "DS2"]);
}

// spec: S-0312 claim 9 (one list pick from the explosion list, then one from the destruction-animation list, at the vehicle's position) and claim 15 (it then leaves play)
#[test]
fn claim_9_explosion_and_destruction_picks() {
    let mut r = rules();
    listed_victim(&mut r);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let pos = w.obj(id).unwrap().pos;
    w.effects.clear();
    let mut rr = w.rng.clone();
    let e = rr.list_pick(3) as usize;
    let d = rr.list_pick(2) as usize;
    kill(&mut w, id);
    assert_eq!(
        anims(&w),
        vec![
            (["EX1", "EX2", "EX3"][e].to_string(), pos),
            (["DS1", "DS2"][d].to_string(), pos)
        ]
    );
    assert_eq!(w.rng, rr);
    assert!(w.obj(id).is_none());
}

// spec: S-0312 claim 9 (an exploding vehicle with unlimited ammunition, or ammunition left, shows the last explosion entry; the list pick's draw is still taken)
#[test]
fn claim_9_exploding_vehicle_shows_last_entry() {
    for (ammo, left, last) in [
        (None, None, true),
        (Some(3), Some(1), true),
        (Some(3), Some(0), false),
    ] {
        let mut r = rules();
        listed_victim(&mut r);
        let v = r.sim.vehicles.get_mut("VICTIM").unwrap();
        v.combat.explodes = Some(true);
        v.combat.ammo = ammo;
        let mut w = game(&r);
        let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
        if let Some(a) = left {
            w.obj_mut(id).unwrap().combat.ammo = a;
        }
        w.effects.clear();
        let mut rr = w.rng.clone();
        let e = rr.list_pick(3) as usize;
        let _ = rr.list_pick(2);
        kill(&mut w, id);
        let shown = &anims(&w)[0].0;
        if last {
            assert_eq!(shown, "EX3");
        } else {
            assert_eq!(shown, ["EX1", "EX2", "EX3"][e]);
        }
        assert_eq!(w.rng, rr);
    }
}

// spec: S-0312 claim 9 (each pick only when its list is not empty)
#[test]
fn claim_9_empty_lists_take_no_draw() {
    let mut r = rules();
    r.sim
        .vehicles
        .get_mut("VICTIM")
        .unwrap()
        .combat
        .destroy_anim = names(&["DS1", "DS2"]);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let mut rr = w.rng.clone();
    let d = rr.list_pick(2) as usize;
    w.effects.clear();
    kill(&mut w, id);
    assert_eq!(anims(&w)[0].0, ["DS1", "DS2"][d]);
    assert_eq!(w.rng, rr);
}

// ---------------------------------------------------------------- claim 13, claim 25, F4

fn crew_rules() -> Ruleset {
    let mut r = rules();
    r.sim.vehicles.get_mut("VICTIM").unwrap().crewed = Some(true);
    r.sim.general.crew_escape = dec("1");
    r.sim.general.technician = Some("TECH".into());
    r
}

fn crews(w: &World) -> Vec<ObjectId> {
    let ti = w.rules().index_of(Kind::Infantry, "TECH").unwrap();
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Infantry && o.type_index == ti)
        .map(|o| o.id)
        .collect()
}

// spec: S-0312 claim 13 and F4 (a crewed vehicle with no passenger capacity: a chance test with one unit-interval value, the crew type with no draw for a house without a side (the technician), the creation draw, the position draw of a centre class (revision 51), then health R(5, trunc(S / 2)); owned by the vehicle's house, facing map north, on guard)
#[test]
fn claim_13_crew_escape_draws() {
    let r = crew_rules();
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let mode = w.rules().mode;
    let h = half(&w);
    let mut rr = w.rng.clone();
    let _chance = rr.unit(mode);
    let _creation = rr.raw();
    let _position = rr.ranged(0, 3, h);
    let health = rr.ranged(5, 30, h);
    let before = w.rng.taken();
    kill(&mut w, id);
    let c = crews(&w);
    assert_eq!(c.len(), 1);
    let o = w.obj(c[0]).unwrap();
    assert_eq!(o.health, health);
    assert_eq!(o.owner, Some(glasnost_sim::world::HouseId(0)));
    assert_eq!(o.body.value(w.tick), 0);
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert!(w.rng.taken() >= rr.taken());
    assert!(rr.taken() > before);
}

// spec: S-0312 claim 13 (the chance test fails with a crew escape chance of 0: no crew, only the unit-interval value is taken)
#[test]
fn claim_13_chance_fails() {
    let mut r = crew_rules();
    r.sim.general.crew_escape = dec("0");
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let mut rr = w.rng.clone();
    rr.unit(w.rules().mode);
    kill(&mut w, id);
    assert!(crews(&w).is_empty());
    assert_eq!(w.rng, rr);
}

// spec: S-0312 claim 13 (no crew and no draw when the killing damage carries the no-escape marker, the type is not crewed, or it has a passenger capacity)
#[test]
fn claim_13_no_crew_cases() {
    for case in 0..3 {
        let mut r = crew_rules();
        let v = r.sim.vehicles.get_mut("VICTIM").unwrap();
        match case {
            1 => v.crewed = Some(false),
            2 => v.passengers = Some(2),
            _ => {}
        }
        let mut w = game(&r);
        let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
        let before = w.rng.clone();
        let hit = Hit {
            no_escape: case == 0,
            ..lethal()
        };
        damage::apply(&mut w, id, hit);
        assert!(crews(&w).is_empty(), "case {case}");
        assert_eq!(w.rng, before, "case {case}");
    }
}

/// Kills VICTIM `id` and returns its crews and the stream as it stands.
fn crew_of(w: &mut World, id: ObjectId) -> (Vec<ObjectId>, Rng) {
    kill(w, id);
    (crews(w), w.rng.clone())
}

/// A world of `r` whose stream has been moved on by `k` raw values, so
/// that runs of different `k` draw different values.
fn game_at(r: &Ruleset, k: usize) -> World {
    let mut w = game(r);
    for _ in 0..k {
        w.rng.raw();
    }
    w
}

/// The health a VICTIM's technician crew gets from the stream as it stands:
/// the chance value, the creation draw, the position draw when `position`,
/// then R(5, 30).
fn crew_health(w: &World, position: bool) -> i32 {
    let h = half(w);
    let mut rr = w.rng.clone();
    rr.unit(w.rules().mode);
    rr.raw();
    if position {
        rr.ranged(0, 3, h);
    }
    rr.ranged(5, 30, h)
}

/// Kills VICTIM `id` of a world whose stream stands as in `w0` and checks
/// that its one crew took the position draw exactly when `position`; over
/// the runs, `told` counts those whose two predictions differ. Returns the
/// crew.
fn crew_drawn(w: &mut World, id: ObjectId, position: bool, told: &mut usize) -> ObjectId {
    let want = crew_health(w, position);
    if want != crew_health(w, !position) {
        *told += 1;
    }
    let (crew, _) = crew_of(w, id);
    assert_eq!(crew.len(), 1);
    assert_eq!(
        w.obj(crew[0]).unwrap().health,
        want,
        "position draw {position}"
    );
    crew[0]
}

// spec: S-0312 claim 13 (revision 51: the crew is put down at the vehicle's exact position after the vehicle has given up its place in the occupancy record, so the vehicle never refuses its own crew; at the surface height inside the usable area S-0208 claim 3's table gives the standing position, with R(0, 3) for a centre class before the health draw; a side class takes its own position with no draw)
#[test]
fn claim_13_crew_standing_position() {
    let r = crew_rules();
    let mut told = [0, 0];
    for k in 0..12 {
        // The vehicle at its cell centre: the centre class, one ranged draw
        // over 0 to 3 picks the row of S-0208's table O.
        let mut w = game_at(&r, k);
        let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
        let c = w.obj(id).unwrap().pos.cell();
        let crew = crew_drawn(&mut w, id, true, &mut told[0]);
        assert_eq!(w.obj(crew).unwrap().pos.cell(), c, "the vehicle's cell");
        // The vehicle in the right part of its cell: the right class, its
        // own position, no position draw.
        let mut w = game_at(&r, k);
        let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
        {
            let o = w.obj_mut(id).unwrap();
            o.pos.x += 70;
            o.pos.y -= 70;
        }
        let c = w.obj(id).unwrap().pos.cell();
        let crew = crew_drawn(&mut w, id, false, &mut told[1]);
        let cp = w.obj(crew).unwrap().pos;
        let (u, v) = glasnost_sim::walk::standing_point(glasnost_sim::cells::position::RIGHT);
        assert_eq!(
            (cp.cell(), cp.x.rem_euclid(256), cp.y.rem_euclid(256)),
            (c, u, v)
        );
    }
    assert!(
        told.iter().all(|&n| n > 0),
        "runs that tell the draw apart: {told:?}"
    );
}

// spec: S-0312 claim 13 (revision 51: a crew for which the table's first row refuses the cell, another vehicle in it, gets no position: its chance and creation draws are taken, no position draw and no health draw, and it never enters play)
#[test]
fn claim_13_crew_with_no_position() {
    let r = crew_rules();
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let c = w.obj(id).unwrap().pos.cell();
    // Another vehicle's vehicle-present claim on the cell.
    w.cells
        .claim_vehicle(c, glasnost_sim::world::Layer::Ground, true);
    let mut rr = w.rng.clone();
    rr.unit(w.rules().mode);
    rr.raw();
    let (crew, after) = crew_of(&mut w, id);
    assert!(crew.is_empty());
    assert_eq!(after.taken(), rr.taken(), "chance and creation only");
}

// spec: S-0312 claim 13 (revision 51: one entry test, S-0209's ordinary return placement for an infantry mover; only clear accepts the crew, so a cell whose entry outcome is not clear refuses it after the table chose a position: chance, creation and position draws, no health draw); claim 18 (a survivor makes no such test)
#[test]
fn claim_13_crew_refused_by_its_entry_test() {
    let r = crew_rules();
    let mut w = game(&r);
    // A building of another house on the vehicle's cell: the table ignores
    // the building-present flag, the entry test does not give clear.
    let hut = spawn(&mut w, Kind::Building, "HUT", 1, -1, -1);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let c = w.obj(id).unwrap().pos.cell();
    assert!(
        w.cells
            .list(c, glasnost_sim::world::Layer::Ground)
            .contains(&hut)
    );
    let h = half(&w);
    let mut rr = w.rng.clone();
    rr.unit(w.rules().mode);
    rr.raw();
    rr.ranged(0, 3, h);
    let (crew, after) = crew_of(&mut w, id);
    assert!(crew.is_empty(), "refused by the entry test");
    assert_eq!(after.taken(), rr.taken(), "no health draw");
}

/// Puts a TECH of `house` on standing position `p` of cell `c` (its mark
/// set by the placement, S-0209 claim 1); `moving` gives it a walking
/// target, so that it has movement in progress (S-0209 Terms).
fn stand(w: &mut World, house: u8, c: glasnost_map::CellXY, p: u8, moving: bool) -> ObjectId {
    let ti = w.rules().index_of(Kind::Infantry, "TECH").unwrap();
    let id = glasnost_sim::objects::create(
        w,
        Kind::Infantry,
        ti,
        Some(glasnost_sim::world::HouseId(house)),
    );
    let (u, v) = glasnost_sim::walk::standing_point(p);
    let (x, y) = (256 * c.x + u, 256 * c.y + v);
    let z = w.surface_at(x, y);
    glasnost_sim::objects::place(w, id, Pos::new(x, y, z), 0);
    if moving {
        let o = w.obj_mut(id).unwrap();
        let at = o.pos;
        o.walk.get_or_insert_with(Default::default).target = Some(at);
    }
    id
}

// spec: S-0312 claim 13 (the crew's one entry test accepts only clear: every other outcome refuses it at once, moving blocker included, with no waiting) and S-0209 claim 4 (rows WR4 and WR5: the three side positions taken by allied infantry, not all three still, gives moving blocker; all three still gives stationary friendly) (M8 seam sweep 5a)
#[test]
fn claim_13_crew_refused_by_a_moving_blocker() {
    use glasnost_sim::cells::position;
    use glasnost_sim::occupancy::{self, Outcome};
    let r = crew_rules();
    // `moving`: whether one of the three allies in the side positions is moving.
    let run = |moving: Option<bool>| {
        let mut w = game(&r);
        let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
        let c = w.obj(id).unwrap().pos.cell();
        if let Some(m) = moving {
            for (i, &p) in position::SIDES.iter().enumerate() {
                stand(&mut w, 0, c, p, m && i == 0);
            }
        }
        // The entry outcome a TECH of the vehicle's house meets in the cell
        // with the vehicle's own claims set aside, as the crew does.
        let probe = spawn(&mut w, Kind::Infantry, "TECH", 0, 5, 5);
        // The vehicle stands at rest at its cell's centre: its one claim is
        // its own cell's, and it is set aside with its listing.
        let ground = glasnost_sim::world::Layer::Ground;
        let listed = w.cells.list(c, ground).iter().position(|&x| x == id);
        w.cells.claim_vehicle(c, ground, false);
        w.cells.leave(c, ground, id);
        let level = w.cells.cell(c).map_or(0, |x| i32::from(x.level));
        let out = occupancy::infantry_outcome(&w, probe, c, level);
        w.cells.restore(c, ground, id, listed.unwrap());
        w.cells.claim_vehicle(c, ground, true);
        glasnost_sim::objects::remove(&mut w, probe);
        let before = crews(&w);
        let (after, _) = crew_of(&mut w, id);
        let new: Vec<_> = after.into_iter().filter(|x| !before.contains(x)).collect();
        (out, new.len())
    };
    assert_eq!(run(Some(true)), (Outcome::MovingBlocker, 0), "refused");
    assert_eq!(
        run(Some(false)),
        (Outcome::StationaryFriendly, 0),
        "control: three still allies refuse too"
    );
    assert_eq!(run(None), (Outcome::Clear, 1), "control: an empty cell takes the crew");
}

/// A game of `r` on the testkit's empty map with its local area cut by 2
/// cells on each side, so that some legal cells lie outside the usable
/// area, its stream moved on by `k` raw values.
fn cut_game_at(r: &Ruleset, k: usize) -> World {
    let mut m = testkit::empty_map(SIZE);
    let fp = glasnost_map::Footprint::new(SIZE, SIZE, [2, 2, SIZE - 4, SIZE - 4]);
    let mut cells = glasnost_map::CellGrid::new(fp);
    let all: Vec<glasnost_map::CellXY> = cells.iter().map(|(c, _)| c).collect();
    for c in all {
        if let (Some(dst), Some(src)) = (cells.get_mut(c), m.cells.get(c)) {
            *dst = *src;
        }
    }
    m.footprint = fp;
    m.cells = cells;
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(r, "synthetic-test"));
    let mut w = glasnost_sim::load::new_game(rules, &m, testkit::catalogue(), &testkit::setup(11))
        .expect("synthetic test game loads");
    for _ in 0..k {
        w.rng.raw();
    }
    w
}

// spec: S-0312 claim 13 (revision 51: in a cell outside the usable area the choice ignores the cell's occupants and takes no draw: the crew stands at its class's own position)
#[test]
fn claim_13_crew_outside_the_usable_area() {
    let r = crew_rules();
    let mut told = 0;
    for k in 0..12 {
        let mut w = cut_game_at(&r, k);
        let c = w
            .cells
            .grid
            .iter()
            .map(|(c, _)| c)
            .find(|&c| !w.cells.usable(c))
            .expect("a cell outside the usable area");
        let id = testkit::spawn(&mut w, Kind::Vehicle, "VICTIM", 0, c, 0);
        let crew = crew_drawn(&mut w, id, false, &mut told);
        let cp = w.obj(crew).unwrap().pos;
        let (u, v) = glasnost_sim::walk::standing_point(glasnost_sim::cells::position::CENTRE);
        assert_eq!(
            (cp.cell(), cp.x.rem_euclid(256), cp.y.rem_euclid(256)),
            (c, u, v)
        );
    }
    assert!(told > 0);
}

// spec: S-0312 claim 13 (revision 51: when the height of the vehicle's position differs from the terrain surface there, no position is chosen and no draw taken: the crew is put at the vehicle's exact position)
#[test]
fn claim_13_crew_above_the_surface() {
    let r = crew_rules();
    let mut told = 0;
    for k in 0..12 {
        let mut w = game_at(&r, k);
        let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
        w.obj_mut(id).unwrap().pos.z += 40;
        let p = w.obj(id).unwrap().pos;
        let crew = crew_drawn(&mut w, id, false, &mut told);
        let cp = w.obj(crew).unwrap().pos;
        assert_eq!((cp.x, cp.y), (p.x, p.y), "the vehicle's exact position");
    }
    assert!(told > 0);
}

// spec: S-0312 claim 18 and Import mapping (revision 51: the crewed flag absent means no for every kind of type: a vehicle type without it leaves no crew)
#[test]
fn claim_13_crewed_absent_on_a_vehicle_is_no() {
    let mut r = crew_rules();
    r.sim.vehicles.get_mut("VICTIM").unwrap().crewed = None;
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let ti = w.rules().index_of(Kind::Vehicle, "VICTIM").unwrap();
    assert!(!w.rules().vehicles[ti as usize].vehicle.crewed);
    let before = w.rng.clone();
    let (crew, after) = crew_of(&mut w, id);
    assert!(crew.is_empty());
    assert_eq!(after, before, "no chance test");
}

/// The crew's behaviour after the death of a VICTIM of house 0, with house
/// 0 human-controlled or not and of the given role.
fn crew_behaviour(human: bool, role: glasnost_sim::world::HouseRole) -> Option<Behaviour> {
    let r = crew_rules();
    let mut setup = testkit::setup(11);
    setup.houses[0].human = human;
    setup.houses[0].role = role;
    let mut w = glasnost_sim::load::new_game(
        std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "s0312")),
        &testkit::empty_map(SIZE),
        testkit::catalogue(),
        &setup,
    )
    .unwrap();
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    kill(&mut w, id);
    let c = crews(&w);
    assert_eq!(c.len(), 1);
    w.obj(c[0]).unwrap().beh.current
}

// spec: S-0312 claim 25 (orders after a death depend on the owner: a player house's crew, human or AI, is put on guard; the neutral or special house's crew on hunt)
// spec: S-0312 claim 13 (the crew of an owner that is not human-controlled is put on hunt)
#[test]
fn claim_25_crew_orders_by_owner() {
    use glasnost_sim::world::HouseRole;
    assert_eq!(
        crew_behaviour(true, HouseRole::Player),
        Some(Behaviour::Guard)
    );
    assert_eq!(
        crew_behaviour(false, HouseRole::Neutral),
        Some(Behaviour::Hunt)
    );
    assert_eq!(
        crew_behaviour(false, HouseRole::Special),
        Some(Behaviour::Hunt)
    );
}

// spec: S-0312 Import mapping (survivor divisors absent from every layer: 200, 100 and 300, engine constants); F5 (q by side)
#[test]
fn f5_survivor_divisor_defaults() {
    let r = crew_rules();
    let rules = glasnost_sim::rules::SimRules::new(&r, "s0312");
    assert_eq!(rules.combat.general.survivor_divisor, [200, 100, 300]);
    let mut r = crew_rules();
    r.sim.general.soviet_survivor_divisor = Some(40);
    let rules = glasnost_sim::rules::SimRules::new(&r, "s0312");
    assert_eq!(rules.combat.general.survivor_divisor, [200, 40, 300]);
}

// spec: S-0312 State and ordering (a vehicle's death effects in order: debris (F1), explosion pick, destruction pick, then the crew's unit-interval value, creation draw and health)
#[test]
fn ordering_vehicle_draws() {
    let mut r = crew_rules();
    listed_victim(&mut r);
    let c = &mut r.sim.vehicles.get_mut("VICTIM").unwrap().combat;
    c.min_debris = Some(1);
    c.max_debris = Some(4);
    c.debris_anims = names(&["DA", "DB"]);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 0);
    let mode = w.rules().mode;
    let h = half(&w);
    let mut rr = w.rng.clone();
    let n = rr.ranged(1, 3, h);
    let mut expect: Vec<String> = (0..n)
        .map(|_| ["DA", "DB"][rr.ranged(0, 1, h) as usize].to_string())
        .collect();
    expect.push(["EX1", "EX2", "EX3"][rr.list_pick(3) as usize].to_string());
    expect.push(["DS1", "DS2"][rr.list_pick(2) as usize].to_string());
    rr.unit(mode);
    rr.raw();
    let health = rr.ranged(5, 30, h);
    w.effects.clear();
    kill(&mut w, id);
    let got: Vec<String> = anims(&w).into_iter().map(|a| a.0).collect();
    assert_eq!(got, expect);
    assert_eq!(w.obj(crews(&w)[0]).unwrap().health, health);
}

// ---------------------------------------------------------------- claim 16

// spec: S-0312 claim 16 (a killed aircraft on the ground shows one entry of its explosion list, one list pick, and leaves play at once)
#[test]
fn claim_16_landed_aircraft() {
    let mut r = rules();
    r.sim.aircraft.get_mut("PLANE").unwrap().combat.explosion = names(&["AX1", "AX2"]);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Aircraft, "PLANE", 0, 0, 0);
    let pos = w.obj(id).unwrap().pos;
    w.effects.clear();
    let mut rr = w.rng.clone();
    let k = rr.list_pick(2) as usize;
    kill(&mut w, id);
    assert_eq!(anims(&w), vec![(["AX1", "AX2"][k].to_string(), pos)]);
    assert_eq!(w.rng, rr);
    assert!(w.obj(id).is_none());
}

// ---------------------------------------------------------------- claim 17

/// A building of type `ty` at the map's middle with two crewed contacts:
/// one inside its foundation (181 leptons from a 2 × 2 reference point)
/// and one two cells east of its top-left cell (405 leptons).
fn contacts(ty: &str) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut r = crew_rules();
    r.sim.buildings.get_mut("HUT").unwrap().foundation = Some([2, 2]);
    let mut w = game(&r);
    let b = spawn(&mut w, Kind::Building, ty, 0, 0, 0);
    let ti = w.rules().index_of(Kind::Vehicle, "VICTIM").unwrap();
    let put = |w: &mut World, dx: i32| {
        let id = glasnost_sim::objects::create(
            w,
            Kind::Vehicle,
            ti,
            Some(glasnost_sim::world::HouseId(1)),
        );
        let p = Pos::centre(testkit::mid(SIZE, dx, if dx == 1 { 1 } else { 0 }), 0);
        glasnost_sim::objects::place(w, id, p, 0);
        w.obj_mut(id).unwrap().contact = Some(b);
        id
    };
    let near = put(&mut w, 1);
    let far = put(&mut w, 2);
    (w, b, near, far)
}

// spec: S-0312 claim 17 (a contact closer than 256 leptons to the building's reference point is destroyed by ten times its strength with the demolition warhead, ignoring defences, no-escape, no attacker; any other contact is only released)
#[test]
fn claim_17_contacts() {
    let (mut w, b, near, far) = contacts("HUT");
    let kills_before: i32 = w.houses.iter().map(|h| h.combat.kills).sum();
    kill(&mut w, b);
    assert!(w.obj(near).is_none());
    assert!(crews(&w).is_empty(), "no-escape: no crew");
    let kills_after: i32 = w.houses.iter().map(|h| h.combat.kills).sum();
    assert_eq!(kills_after, kills_before, "credited to no one");
    let f = w.obj(far).unwrap();
    assert_eq!(f.contact, None);
    assert_eq!(f.health, 200);
}

// spec: S-0312 claim 17 (a helipad destroys its contacts whatever the distance)
#[test]
fn claim_17_helipad_contacts() {
    let (mut w, b, near, far) = contacts("PAD");
    kill(&mut w, b);
    assert!(w.obj(near).is_none());
    assert!(w.obj(far).is_none());
}

// ---------------------------------------------------------------- claims 19, 21

fn building_rules(fw: i32, fh: i32) -> Ruleset {
    let mut r = rules();
    let b = r.sim.buildings.get_mut("HUT").unwrap();
    b.foundation = Some([fw, fh]);
    b.combat = CombatFields {
        explosion: names(&["BX1", "BX2"]),
        destroy_anim: names(&["BD1", "BD2", "BD3"]),
        ..Default::default()
    };
    r
}

/// Replays claim 19's draws (a), (b) and (e) for a HUT of `fw` × `fh`
/// with `discarded` the smudge switch; returns the expected anims.
fn replay_building(w: &World, rr: &mut Rng, id: ObjectId, discarded: bool) -> Vec<(String, Pos)> {
    let o = w.obj(id).unwrap();
    let (fw, fh) = o.foundation;
    let h = half(w);
    if fw >= 2 && fh >= 2 {
        if discarded {
            if fw > 2 {
                rr.ranged(0, fw - 2, h);
            }
            if fh > 2 {
                rr.ranged(0, fh - 2, h);
            }
        }
        rr.percent();
    }
    let mut out = Vec::new();
    let tl = o.pos.cell();
    for dy in 0..fh {
        for dx in 0..fw {
            let c = glasnost_map::CellXY::new(tl.x + dx, tl.y + dy);
            let raw = rr.raw();
            let p = death::nearby_point(w, Pos::centre(c, o.pos.z), 64, raw, false);
            rr.ranged(0, 3, h);
            let k = rr.list_pick(2) as usize;
            out.push((["BX1", "BX2"][k].to_string(), p));
        }
    }
    out.push((
        ["BD1", "BD2", "BD3"][rr.list_pick(3) as usize].to_string(),
        o.pos,
    ));
    out
}

// spec: S-0312 claim 19 (a) and Edge cases (quirks.building_smudge_discarded_draws on: discarded draws R(0, W − 2) and R(0, H − 2) for sides above 2, then the percent draw; (b) per foundation cell in S-0302's order: nearby point raw value, delay R(0, 3), list pick, the animation at N(cell centre, 64, b); (e) one destruction list pick) and claim 21 (it then leaves play)
#[test]
fn claim_19_building_draws() {
    for (fw, fh) in [(3, 3), (3, 2), (2, 2), (1, 1)] {
        let r = building_rules(fw, fh);
        let mut w = game(&r);
        let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
        let mut rr = w.rng.clone();
        let expect = replay_building(&w, &mut rr, id, true);
        w.effects.clear();
        kill(&mut w, id);
        assert_eq!(anims(&w), expect, "{fw} × {fh}");
        assert_eq!(w.rng, rr, "{fw} × {fh}");
        assert!(w.obj(id).is_none());
    }
}

// spec: S-0312 Edge cases (quirks.building_smudge_discarded_draws off: the discarded draws of claim 19 (a) are not taken)
#[test]
fn quirk_building_smudge_discarded_draws_off() {
    let mut r = building_rules(3, 3);
    r.globals.quirks.building_smudge_discarded_draws = false;
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
    let mut rr = w.rng.clone();
    let expect = replay_building(&w, &mut rr, id, false);
    w.effects.clear();
    kill(&mut w, id);
    assert_eq!(anims(&w), expect);
    assert_eq!(w.rng, rr);
}

// spec: S-0312 claim 26 and Edge cases (quirks.smudge_footprints_beyond_map_edge, default on; smudge types are not modelled, so no type fits under either setting: n = 0, no smudge draw, and the building's draws are claim 19's alone)
#[test]
fn quirk_smudge_footprints_beyond_map_edge_both_settings() {
    assert!(
        glasnost_rules::quirks::Quirks::default().smudge_footprints_beyond_map_edge,
        "the default is the original"
    );
    for on in [true, false] {
        let mut r = building_rules(3, 3);
        r.globals.quirks.smudge_footprints_beyond_map_edge = on;
        let mut w = game(&r);
        let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
        let mut rr = w.rng.clone();
        let expect = replay_building(&w, &mut rr, id, true);
        w.effects.clear();
        kill(&mut w, id);
        assert_eq!(anims(&w), expect, "switch {on}");
        assert_eq!(w.rng, rr, "switch {on}");
    }
}

// spec: S-0312 claim 26 (zero-coordinate exception), Edge cases (quirks.destruction_smudge_zero_anchor)
// Smudge types are not modelled, so no candidate fits anywhere: both
// settings take the same draws (no smudge-choice draw).
#[test]
fn quirk_destruction_smudge_zero_anchor_both_settings() {
    assert!(
        glasnost_rules::quirks::Quirks::default().destruction_smudge_zero_anchor,
        "the default is the original"
    );
    for on in [true, false] {
        let mut r = building_rules(3, 3);
        r.globals.quirks.destruction_smudge_zero_anchor = on;
        let mut w = game(&r);
        let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
        let mut rr = w.rng.clone();
        let expect = replay_building(&w, &mut rr, id, true);
        w.effects.clear();
        kill(&mut w, id);
        assert_eq!(anims(&w), expect, "switch {on}");
        assert_eq!(w.rng, rr, "switch {on}");
    }
}

// ---------------------------------------------------------------- F2

// spec: S-0312 F2 (b = 0 points to map north, 64 east, 128 south, 192 west, r leptons away; z unchanged)
#[test]
fn f2_nearby_point_directions() {
    let r = rules();
    let w = game(&r);
    let q = Pos::centre(testkit::mid(SIZE, 0, 0), 37);
    let r0 = 128;
    for (b, dx, dy) in [(0u32, 0, -r0), (64, r0, 0), (128, 0, r0), (192, -r0, 0)] {
        let p = death::nearby_point(&w, q, r0, b + 1024, false);
        assert!((p.x - (q.x + dx)).abs() <= 1, "b = {b}: {p:?}");
        assert!((p.y - (q.y + dy)).abs() <= 1, "b = {b}: {p:?}");
        assert_eq!(p.z, 37);
    }
    // Every direction lies r leptons away, to the truncation.
    for b in 0..256u32 {
        let p = death::nearby_point(&w, q, r0, b, false);
        let (dx, dy) = (i64::from(p.x - q.x), i64::from(p.y - q.y));
        let d2 = dx * dx + dy * dy;
        assert!((126 * 126..=130 * 130).contains(&d2), "b = {b}");
    }
}

// spec: S-0312 F2 (snapping gives the centre of the result's cell; a cell coordinate outside 0 to 511 gives Q itself)
#[test]
fn f2_nearby_point_snap_and_limits() {
    let r = rules();
    let w = game(&r);
    let q = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let p = death::nearby_point(&w, q, 256, 64, true);
    assert_eq!(p, Pos::centre(p.cell(), 0));
    assert_eq!(
        p.cell(),
        glasnost_map::CellXY::new(q.cell().x + 1, q.cell().y)
    );
    // S-0204 claim 2 truncates toward zero, so a cell coordinate of −1 needs a lepton coordinate of −256 or less.
    let edge = Pos::new(100, 100, 5);
    assert_eq!(death::nearby_point(&w, edge, 512, 192, false), edge);
    assert_eq!(death::nearby_point(&w, edge, 512, 0, true), edge);
    let inside = death::nearby_point(&w, edge, 200, 192, false);
    assert!(inside.x < 0 && inside.x > -256 && inside != edge);
}

// ---------------------------------------------------------------- S-0107 claim 17

fn soldier_death(
    edit: impl FnOnce(&mut Ruleset),
    wh: &str,
    prep: impl FnOnce(&mut World, ObjectId),
) -> (World, ObjectId) {
    let mut r = rules();
    edit(&mut r);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Infantry, "SOLDIER", 0, 0, 0);
    prep(&mut w, id);
    let h = warhead(&r, wh);
    damage::apply(&mut w, id, hit_with(h, 1000));
    (w, id)
}

fn seq_of(w: &World, id: ObjectId) -> Option<Seq> {
    w.obj(id)
        .and_then(|o| o.infantry.as_ref())
        .and_then(|s| s.seq)
}

// spec: S-0107 claim 17 (rows 5 to 8: death kind w = 1 gives die-1, w = 2 die-2, any other w no sequence and removal at once); S-0611 claims 6 and 7 (w = 9 on open ground converts: the victim leaves play at once; a refused conversion's die-2 is claims_s0611's)
#[test]
fn s0107_claim_17_death_kinds() {
    for (wh, expect) in [
        ("W1", Some(Seq::Die1)),
        ("W2", Some(Seq::Die2)),
        ("W9", None),
        ("W4", None),
    ] {
        let (w, id) = soldier_death(|_| {}, wh, |_, _| {});
        match expect {
            Some(q) => {
                assert_eq!(seq_of(&w, id), Some(q), "{wh}");
                assert!(w.obj(id).unwrap().combat.killed);
            }
            None => assert!(w.obj(id).is_none(), "{wh}"),
        }
    }
}

// spec: S-0107 claim 17 (row 4: a type marked not human shows die-1 whatever the death kind)
#[test]
fn s0107_claim_17_not_human() {
    for wh in ["W1", "W2", "W4"] {
        let (w, id) = soldier_death(
            |r| r.sim.infantry.get_mut("SOLDIER").unwrap().not_human = Some(true),
            wh,
            |_, _| {},
        );
        assert_eq!(seq_of(&w, id), Some(Seq::Die1), "{wh}");
    }
}

// spec: S-0107 claim 17 (row 2: a prone cyborg or a jumpjet type shows no sequence and is removed at once)
#[test]
fn s0107_claim_17_jumpjet_removed() {
    let (w, id) = soldier_death(
        |r| r.sim.infantry.get_mut("SOLDIER").unwrap().jumpjet = Some(true),
        "W1",
        |_, _| {},
    );
    assert!(w.obj(id).is_none());
}

// spec: S-0107 claim 17 (the death kind is 5 for a laser-fence killer: row 8, removed at once, even with a warhead of kind 1)
#[test]
fn s0107_claim_17_laser_fence_killer() {
    let r = rules();
    let mut w = game(&r);
    let fence = spawn(&mut w, Kind::Building, "FENCE", 1, 4, 0);
    let id = spawn(&mut w, Kind::Infantry, "SOLDIER", 0, 0, 0);
    let hit = Hit {
        attacker: Some(fence),
        ..hit_with(warhead(&r, "W1"), 1000)
    };
    damage::apply(&mut w, id, hit);
    assert!(w.obj(id).is_none());
}

// spec: S-0107 claim 17 (the death kind is 3 while the current sequence is parachute: row 8, removed at once)
#[test]
fn s0107_claim_17_parachute_kind_3() {
    let (w, id) = soldier_death(
        |r| {
            r.sim
                .infantry
                .get_mut("SOLDIER")
                .unwrap()
                .sequences
                .insert("parachute".into(), seq(4));
        },
        "W1",
        |w, id| {
            w.obj_mut(id).unwrap().infantry.as_mut().unwrap().seq = Some(Seq::Parachute);
        },
    );
    assert!(w.obj(id).is_none());
}

// spec: S-0107 claim 17 (cyborgs and damage that ignores defences: removed at once on the death request whether or not it succeeded)
#[test]
fn s0107_claim_17_cyborg_ignoring_defences() {
    let r = cyborg_rules(100);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Infantry, "BORG", 0, 0, 0);
    let hit = Hit {
        ignore_defences: true,
        ..hit_with(warhead(&r, "W1"), 1000)
    };
    damage::apply(&mut w, id, hit);
    assert!(w.obj(id).is_none());
}

// spec: S-0107 claim 17 (when die-1 completes the unit is removed on the same tick: it stays in play while the sequence plays)
#[test]
fn s0107_claim_17_removed_when_sequence_completes() {
    let (mut w, id) = soldier_death(|_| {}, "W1", |_, _| {});
    testkit::run(&mut w, 1);
    assert!(w.obj(id).is_some());
    let mut ticks = 0;
    while w.obj(id).is_some() && ticks < 500 {
        let s = w.obj(id).unwrap().infantry.clone().unwrap();
        assert_eq!(s.seq, Some(Seq::Die1));
        testkit::run(&mut w, 1);
        ticks += 1;
    }
    assert!(w.obj(id).is_none());
}

// spec: S-0107 claim 17 and Edge cases (quirks.refused_death_keeps_unit on: a death request refused because the entry has C = 0 leaves the unit in play with its current sequence; off: it is removed on the tick of its death)
#[test]
fn quirk_refused_death_keeps_unit_both_settings() {
    for on in [true, false] {
        let (w, id) = soldier_death(
            |r| {
                r.globals.quirks.refused_death_keeps_unit = on;
                r.sim
                    .infantry
                    .get_mut("SOLDIER")
                    .unwrap()
                    .sequences
                    .insert("die_1".into(), seq(0));
            },
            "W1",
            |w, id| {
                w.obj_mut(id).unwrap().infantry.as_mut().unwrap().seq = Some(Seq::Stand);
            },
        );
        if on {
            assert_eq!(seq_of(&w, id), Some(Seq::Stand));
            assert!(w.obj(id).unwrap().combat.killed);
        } else {
            assert!(w.obj(id).is_none());
        }
    }
}

// spec: S-0312 claim 8 (a killed infantry unit takes the shared effects, debris here, and then S-0107 claim 17's death sequence)
#[test]
fn claim_8_infantry_debris_then_sequence() {
    let mut r = rules();
    let c = &mut r.sim.infantry.get_mut("SOLDIER").unwrap().combat;
    c.min_debris = Some(2);
    c.max_debris = Some(3);
    c.debris_anims = names(&["GIB"]);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Infantry, "SOLDIER", 0, 0, 0);
    w.effects.clear();
    damage::apply(&mut w, id, hit_with(warhead(&r, "W1"), 1000));
    assert_eq!(anims(&w).len(), 2);
    assert_eq!(seq_of(&w, id), Some(Seq::Die1));
}

// ---------------------------------------------------------------- claim 21 (revision 4)

/// HUT with the explodes flag as given, destroyed by a lethal hit; returns
/// the world and the building.
fn awaiting(explodes: bool, second_round: bool) -> (World, ObjectId) {
    let mut r = rules();
    r.globals.quirks.building_removal_second_round = second_round;
    r.sim.buildings.get_mut("HUT").unwrap().combat.explodes = Some(explodes);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
    assert_eq!(kill(&mut w, id), HitResult::NowDead);
    (w, id)
}

// spec: S-0312 claim 21 (a building whose type has the explodes flag awaits removal after its death effects: in play with 0 health, still in its cells, and cannot be hit; in its own next turn it is removed)
#[test]
fn claim_21_exploding_building_awaits_removal() {
    let (mut w, id) = awaiting(true, true);
    let o = w.obj(id).expect("still in play");
    assert_eq!(o.health, 0);
    assert!(o.combat.killed);
    let cell = o.pos.cell();
    assert!(
        w.cells
            .occ(cell, glasnost_sim::world::Layer::Ground)
            .is_some_and(|c| c.buildings > 0)
    );
    assert_eq!(kill(&mut w, id), HitResult::Unaffected);
    glasnost_sim::tick::step(&mut w);
    assert!(w.obj(id).is_none());
    assert!(
        w.cells
            .occ(cell, glasnost_sim::world::Layer::Ground)
            .is_some_and(|c| c.buildings == 0)
    );
}

// spec: S-0214 claim 16 (a building awaiting removal can still be selected by a click: zero health is not a selection exclusion; a type whose selection flag is off is not selectable)
#[test]
fn awaiting_removal_can_be_selected() {
    use glasnost_sim::targeting::{self, ClickOn, Modifiers};
    use glasnost_sim::world::HouseId;
    let (w, id) = awaiting(true, true);
    assert_eq!(w.obj(id).unwrap().health, 0);
    assert!(targeting::selectable(&w, id));
    let r = targeting::click(
        &w,
        HouseId(0),
        &[],
        ClickOn::Object(id),
        Modifiers::default(),
    );
    assert_eq!(r.selection, Some(vec![id]));
    // The type's selection flag off: not selectable, with or without health.
    let mut r = rules();
    let hut = r.sim.buildings.get_mut("HUT").unwrap();
    hut.combat.explodes = Some(true);
    hut.selectable = Some(false);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
    assert!(!targeting::selectable(&w, id));
    assert_eq!(kill(&mut w, id), HitResult::NowDead);
    assert!(!targeting::selectable(&w, id));
}

// spec: S-0312 claim 21 (only the type's explodes flag counts: a building without it, not being sold, leaves play at once)
#[test]
fn claim_21_other_buildings_leave_at_once() {
    let (w, id) = awaiting(false, true);
    assert!(w.obj(id).is_none());
}

// spec: S-0312 claim 21 (a building whose behaviour is the sale's at its destruction awaits removal too)
#[test]
fn claim_21_selling_building_awaits_removal() {
    let r = rules();
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
    w.obj_mut(id).unwrap().beh.current = Some(Behaviour::Selling);
    assert_eq!(kill(&mut w, id), HitResult::NowDead);
    assert_eq!(w.obj(id).map(|o| o.health), Some(0));
}

/// The random stream and the building after the removal turn.
fn removal_round(second_round: bool) -> (Rng, bool) {
    let (mut w, id) = awaiting(true, second_round);
    glasnost_sim::tick::step(&mut w);
    (w.rng.clone(), w.obj(id).is_some())
}

// spec: S-0312 claim 21 and Edge cases (quirks.building_removal_second_round on: the removal takes a second round of claims 18 and 19 (g), with C afresh by F5; C = 0 in our engine until sides exist, so the round takes no draw and the building leaves play in that turn)
#[test]
fn quirk_building_removal_second_round_on() {
    let (rng, present) = removal_round(true);
    assert!(!present);
    assert_eq!(rng, removal_round(false).0);
}

// spec: S-0312 Edge cases (quirks.building_removal_second_round off: no second round; it leaves its cells and leaves play in its removal turn)
#[test]
fn quirk_building_removal_second_round_off() {
    let (_, present) = removal_round(false);
    assert!(!present);
}

// spec: S-0312 claim 3 and S-0302 claim 16 (the expiry hit destroys the building in its turn with every death effect; an exploding building then awaits removal until its own next turn)
#[test]
fn claim_3_expiry_then_awaiting_removal() {
    let mut r = delay_rules();
    r.sim.buildings.get_mut("HUT").unwrap().combat.explodes = Some(true);
    r.sim.general.c4_warhead = Some("W1".into());
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
    let t = w.tick;
    damage::apply(&mut w, id, delay_hit(&r, 0));
    testkit::run(&mut w, 6);
    // Expired in the turn of tick t + 5: 0 health, still in play.
    assert_eq!(w.tick, t + 6);
    assert_eq!(w.obj(id).map(|o| o.health), Some(0));
    glasnost_sim::tick::step(&mut w);
    assert!(w.obj(id).is_none());
}

// ---------------------------------------------------------------- claim 27 (revision 4)

// spec: S-0312 claim 27 (absent from every layer: crew escape chance 0.5, sinking weight 3.0, survivor divisors 200, 100 and 300; a given value replaces them)
#[test]
fn claim_27_defaults_of_the_crew_survivor_and_sinking_values() {
    let mut r = testkit::combat_ruleset();
    let g = &mut r.sim.general;
    g.crew_escape = None;
    g.ship_sinking_weight = None;
    g.allied_survivor_divisor = None;
    g.soviet_survivor_divisor = None;
    g.third_survivor_divisor = None;
    let w = testkit::game_with(&r, 16, 1);
    let mode = w.rules().mode;
    let gw = &w.rules().combat.general;
    assert!(
        gw.crew_escape
            .eq_num(glasnost_sim::rules::nearest_double("0.5"))
    );
    assert!(
        gw.sinking_weight
            .eq_num(glasnost_num::F64::from_i64(3, mode))
    );
    assert_eq!(gw.survivor_divisor, [200, 100, 300]);
    r.sim.general.crew_escape = dec("0.25");
    let w = testkit::game_with(&r, 16, 1);
    assert!(
        w.rules()
            .combat
            .general
            .crew_escape
            .eq_num(glasnost_sim::rules::nearest_double("0.25"))
    );
}

// spec: S-0312 claim 21 (revision 5: a building destroyed while being sold is not processed at 0 health, so its sale does not advance or pay its refund; it is removed in its own next turn like an exploding building)
#[test]
fn claim_21_destroyed_sale_stops() {
    let r = rules();
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Building, "HUT", 0, 0, 0);
    w.obj_mut(id).unwrap().beh.current = Some(Behaviour::Selling);
    assert_eq!(kill(&mut w, id), HitResult::NowDead);
    let credits = w.houses[0].credits;
    glasnost_sim::tick::step(&mut w);
    assert!(w.obj(id).is_none(), "removed in its own next turn");
    assert_eq!(w.houses[0].credits, credits, "no sale refund");
}

// spec: S-0312 claim 16 (an empty explosion list: no aircraft-specific animation and no draw, no fallback to the destruction list; the same on the ground)
#[test]
fn claim_16_empty_explosion_list() {
    let mut r = rules();
    r.sim.aircraft.get_mut("PLANE").unwrap().combat.destroy_anim = names(&["DX1", "DX2"]);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Aircraft, "PLANE", 0, 0, 0);
    w.effects.clear();
    let before = w.rng.clone();
    kill(&mut w, id);
    assert!(anims(&w).is_empty(), "{:?}", anims(&w));
    assert_eq!(w.rng, before, "no list-pick draw");
    assert!(w.obj(id).is_none());
}

// ---------------------------------------------------------------- claim 29

mod destruction_triggers {
    use super::*;
    use glasnost_map::triggers::{Action, Event, TagDef, TriggerDef, Triggers};
    use std::sync::Arc;

    /// One tag `TAG` (repeat type `repeat`) over one trigger whose only event
    /// is destroyed-by-anything (48) with the given actions.
    fn triggers(repeat: i32, actions: Vec<Action>) -> Triggers {
        Triggers {
            triggers: vec![TriggerDef {
                events: vec![Event {
                    code: 48, // spec: S-0312 Import mapping (event code 48, destroyed by anything)
                    kind: 0,
                    value: 0,
                    type_name: None,
                }],
                actions,
                ..TriggerDef::created(b"T".to_vec())
            }],
            tags: vec![TagDef {
                id: b"TAG".to_vec(),
                repeat,
                name: Vec::new(),
                first: Some(0),
                chain: vec![0],
            }],
            ..Default::default()
        }
    }

    /// A sound-at-a-waypoint action (S-0011 claims 32 and 33), whose effect
    /// shows when the action has run.
    fn sound() -> Action {
        Action {
            code: 99,
            kind: 7,
            name: b"Bell".to_vec(),
            number: -1,
            waypoint: 0,
            ..Default::default()
        }
    }

    fn game_with_triggers(r: &Ruleset, t: Triggers) -> World {
        let mut m = testkit::empty_map(SIZE);
        m.triggers = t;
        glasnost_sim::load::new_game(
            Arc::new(glasnost_sim::rules::SimRules::new(r, "synthetic-test")),
            &m,
            testkit::catalogue(),
            &testkit::setup(11),
        )
        .expect("synthetic game loads")
    }

    fn tagged(w: &mut World, kind: Kind, name: &str, dx: i32) -> ObjectId {
        let id = spawn(w, kind, name, 0, dx, 0);
        glasnost_sim::triggers::object_tag(w, id, Some(b"TAG"));
        id
    }

    /// A lethal hit from a known attacker of house 1.
    fn by(w: &mut World) -> Hit {
        let a = spawn(w, Kind::Vehicle, "NEIGH", 1, 6, 6);
        Hit {
            attacker: Some(a),
            house: w.obj(a).unwrap().owner,
            ..lethal()
        }
    }

    // spec: S-0312 claim 29 (the destruction raises happen in the killing hit after health reaches 0 and before the death debris: the fired action's effect precedes the first debris animation)
    #[test]
    fn claim_29_trigger_action_before_debris() {
        let mut r = rules();
        let c = &mut r.sim.vehicles.get_mut("VICTIM").unwrap().combat;
        c.min_debris = Some(2);
        c.max_debris = Some(3);
        c.debris_anims = names(&["DA"]);
        let mut w = game_with_triggers(&r, triggers(0, vec![sound()]));
        let id = tagged(&mut w, Kind::Vehicle, "VICTIM", 0);
        w.effects.clear();
        kill(&mut w, id);
        let bell = w
            .effects
            .iter()
            .position(|e| matches!(e, Effect::Trigger { code: 99, .. }))
            .expect("the action ran");
        let debris = w
            .effects
            .iter()
            .position(|e| matches!(e, Effect::Anim { name, .. } if name == "DA"))
            .expect("debris");
        assert!(bell < debris, "trigger {bell}, debris {debris}");
        assert!(w.obj(id).is_none());
    }

    /// The firings of a repeating destroyed-by-anything tag at one death.
    fn firings(r: &Ruleset, kind: Kind, name: &str, attacker: bool) -> u64 {
        let mut w = game_with_triggers(r, triggers(2, vec![]));
        let id = tagged(&mut w, kind, name, 0);
        let hit = if attacker { by(&mut w) } else { lethal() };
        let f0 = w.triggers.fired;
        damage::apply(&mut w, id, hit);
        w.triggers.fired - f0
    }

    // spec: S-0312 claim 29 and scenario D20 (with the persistent latch, a vehicle killed by a known attacker gives seven firings and four without, against two for infantry, aircraft and buildings)
    #[test]
    fn claim_29_vehicle_extra_group() {
        let r = rules();
        assert_eq!(firings(&r, Kind::Vehicle, "VICTIM", true), 7);
        assert_eq!(firings(&r, Kind::Vehicle, "VICTIM", false), 4);
        for (kind, name) in [
            (Kind::Infantry, "TECH"),
            (Kind::Aircraft, "PLANE"),
            (Kind::Building, "HUT"),
        ] {
            assert_eq!(firings(&r, kind, name, true), 2, "{name}");
            assert_eq!(firings(&r, kind, name, false), 2, "{name}");
        }
    }

    // spec: S-0312 Edge cases (quirks.vehicle_destruction_extra_raises off: only the common group, two firings for a vehicle)
    #[test]
    fn quirk_vehicle_destruction_extra_raises_off() {
        let mut r = rules();
        r.globals.quirks.vehicle_destruction_extra_raises = false;
        assert_eq!(firings(&r, Kind::Vehicle, "VICTIM", true), 2);
        assert_eq!(firings(&r, Kind::Vehicle, "VICTIM", false), 2);
    }

    // spec: S-0312 claim 29 (a supplied attacking house without an attacker: only the common group)
    #[test]
    fn claim_29_supplied_house_gives_only_the_common_group() {
        let r = rules();
        let mut w = game_with_triggers(&r, triggers(2, vec![]));
        let id = tagged(&mut w, Kind::Vehicle, "VICTIM", 0);
        let f0 = w.triggers.fired;
        damage::apply(
            &mut w,
            id,
            Hit {
                house: Some(glasnost_sim::world::HouseId(1)),
                ..lethal()
            },
        );
        assert_eq!(w.triggers.fired - f0, 2);
    }

    // spec: S-0312 claim 29 (deferred deaths: a delayed-kill building raises on its initial lethal hit, before its health is restored to 1; a cyborg's crippling makes no raise)
    #[test]
    fn claim_29_deferred_deaths() {
        let r = delay_rules();
        let mut w = game_with_triggers(&r, triggers(0, vec![sound()]));
        let id = tagged(&mut w, Kind::Building, "HUT", 0);
        w.effects.clear();
        assert_eq!(
            damage::apply(&mut w, id, delay_hit(&r, 0)),
            HitResult::PostMortem
        );
        assert_eq!(w.obj(id).unwrap().health, 1);
        assert!(
            w.effects
                .iter()
                .any(|e| matches!(e, Effect::Trigger { code: 99, .. }))
        );
        let r = cyborg_rules(200);
        let mut w = game_with_triggers(&r, triggers(2, vec![]));
        let id = tagged(&mut w, Kind::Infantry, "BORG", 0);
        let f0 = w.triggers.fired;
        assert_eq!(
            damage::apply(&mut w, id, hit_with(warhead(&r, "W1"), 1000)),
            HitResult::NowRed
        );
        assert_eq!(w.triggers.fired, f0, "the crippling fires nothing");
        damage::apply(&mut w, id, hit_with(warhead(&r, "W1"), 1000));
        assert_eq!(
            w.triggers.fired - f0,
            2,
            "the later lethal hit fires normally"
        );
    }
}

// spec: S-0312 claim 2 and scenario D2 (strength 203: the first hit leaves 50 health, not 51, the quarter truncated toward zero)
#[test]
fn claim_2_cyborg_crippled_truncates() {
    let r = cyborg_rules(203);
    let mut w = game(&r);
    let id = spawn(&mut w, Kind::Infantry, "BORG", 0, 0, 0);
    damage::apply(&mut w, id, hit_with(warhead(&r, "W1"), 1000));
    assert_eq!(w.obj(id).unwrap().health, 50);
}
