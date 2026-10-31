//! Claim tests for S-0506 revision 19, claim 27 (close targets and the
//! launcher's own blast, Q-2159) and its scenario R23, on synthetic data:
//! invented types, weapons, warheads and rocket parameter values, none of
//! them stock values.
//!
//! Claim 27 adds no rule of its own: it states what other claims give
//! together (the minimum range of S-0307 claim 9 in the firing check and in
//! the controller's adoption test, S-0506 claims 5, 6 and 11; the allies row
//! of S-0302 claim 8 read from the missile's owner; S-0210's crush rules).
//! These tests drive the engine's own paths and check that it holds.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LocomotionClass, Locomotor, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::check::{Answer, check};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::range;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::spawn::{self, ControllerState, SlotState};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{crush, occupancy, psychic, testkit};

const SIZE: i32 = 40;
const ALPHA: HouseId = HouseId(0);

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn launcher(weapon: &str) -> UnitType {
    let mut t = UnitType {
        strength: Some(400),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        primary: Some(weapon.into()),
        missile_spawn: Some(true),
        can_approach_target: Some(true),
        ..Default::default()
    };
    t.combat.spawns = Some("RKA".into());
    t.combat.spawns_number = Some(1);
    t.combat.spawn_regen_rate = Some(50);
    t.combat.spawn_reload_rate = Some(0);
    t.combat.primary_fire_flh = Some([60, 0, 80]);
    t
}

/// The combat ruleset with R23's test values:
/// - RKA, the set-A rocket type (missile-spawn flag);
/// - LNM: a driving launcher of one RKA slot whose spawner weapon LaunchM
///   has range 12 cells and minimum range 4 cells, with the can-approach
///   property and no deploy-to-fire setting; LNMC: LNM made crushable;
///   LNN: a crushable launcher whose spawner weapon LaunchN has no minimum
///   range;
/// - GUNM: LNM's twin with an ordinary weapon GunM of the same range and
///   minimum range (no spawned type);
/// - VICTIM (strength 5000), RUNNER (fast, strength 5000), MASH (an unarmed
///   crusher) and the crushable infantry GRUNT;
/// - STRUCT, a high-health 1 × 1 building;
/// - the rocket warhead RKW, whose reach (12 cells) covers the spawner and
///   its neighbours from every detonation point in range, with full damage
///   at every distance; `allies` sets its affects-allies field.
fn rules_with(allies: Option<bool>) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    register(&mut r, "aircraft", "RKA");
    for n in ["LNM", "LNMC", "LNN", "GUNM", "VICTIM", "RUNNER", "MASH"] {
        register(&mut r, "vehicles", n);
    }
    register(&mut r, "infantry", "GRUNT");
    register(&mut r, "buildings", "STRUCT");
    let s = &mut r.sim;
    s.aircraft.insert(
        "RKA".into(),
        UnitType {
            strength: Some(40),
            speed: Some(50),
            rot: Some(20),
            missile_spawn: Some(true),
            spawned: Some(true),
            locomotor: Some(Locomotor::Rocket),
            ..Default::default()
        },
    );
    s.vehicles.insert("LNM".into(), launcher("LaunchM"));
    let mut lnmc = launcher("LaunchM");
    lnmc.crushable = Some(true);
    s.vehicles.insert("LNMC".into(), lnmc);
    let mut lnn = launcher("LaunchN");
    lnn.crushable = Some(true);
    s.vehicles.insert("LNN".into(), lnn);
    let mut gunm = launcher("GunM");
    gunm.missile_spawn = None;
    gunm.combat.spawns = None;
    gunm.combat.spawns_number = None;
    s.vehicles.insert("GUNM".into(), gunm);
    let plain = |speed: i32| UnitType {
        strength: Some(5000),
        speed: Some(speed),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    };
    s.vehicles.insert("VICTIM".into(), plain(4));
    let mut runner = plain(100);
    runner.rot = Some(127);
    s.vehicles.insert("RUNNER".into(), runner);
    let mut mash = plain(6);
    mash.crusher = Some(true);
    s.vehicles.insert("MASH".into(), mash);
    let mut grunt = s.infantry["WALKER"].clone();
    grunt.crushable = Some(true);
    grunt.primary = None;
    s.infantry.insert("GRUNT".into(), grunt);
    s.buildings.insert(
        "STRUCT".into(),
        BuildingType {
            strength: Some(100_000),
            foundation: Some([1, 1]),
            ..Default::default()
        },
    );
    let spawner = |min: Option<&str>| WeaponType {
        damage: Some(1),
        rof: Some(20),
        range: dec("12"),
        minimum_range: min.and_then(dec),
        projectile: Some("Bullet".into()),
        warhead: Some("HE".into()),
        spawner: Some(true),
        ..Default::default()
    };
    s.weapons.insert("LaunchM".into(), spawner(Some("4")));
    s.weapons.insert("LaunchN".into(), spawner(None));
    s.weapons.insert(
        "GunM".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(20),
            range: dec("12"),
            minimum_range: dec("4"),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    s.warheads.insert(
        "RKW".into(),
        WarheadType {
            cell_spread: dec("12"),
            percent_at_max: dec("1"),
            verses: Some((0..11).map(|_| Decimal("1".into())).collect()),
            affects_allies: allies,
            ..Default::default()
        },
    );
    let g = &mut s.general;
    g.v3_rocket_type = Some("RKA".into());
    g.v3_rocket_pause_frames = Some(4);
    g.v3_rocket_tilt_frames = Some(10);
    g.v3_rocket_altitude = Some(600);
    g.v3_rocket_pitch_final = dec("0.6");
    g.v3_rocket_acceleration = dec("3");
    g.v3_rocket_damage = Some(100);
    g.v3_rocket_elite_damage = Some(100);
    g.v3_warhead = Some("RKW".into());
    g.v3_elite_warhead = Some("RKW".into());
    r
}

fn rules() -> Ruleset {
    rules_with(None)
}

fn game(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, SIZE, 7);
    testkit::run(&mut w, 2);
    w
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

fn open(w: &mut World) {
    w.tick = w.tick.max(w.orders_blocked_through + 1);
}

/// A behaviour order for *attack* with `target` (S-0213 claim 5): a force
/// fire by the owner.
fn attack(w: &mut World, actor: ObjectId, target: ObjectId) {
    open(w);
    let house = w.obj(actor).unwrap().owner.unwrap();
    assert!(orders::issue(
        w,
        house,
        Order::Behaviour {
            actor,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(target)),
            dest: None,
        },
    ));
}

fn controller(w: &World, id: ObjectId) -> spawn::SpawnController {
    w.obj(id).unwrap().spawn.clone().unwrap()
}

fn missile_of(w: &World, id: ObjectId) -> ObjectId {
    controller(w, id).slots[0].object.unwrap()
}

fn launched(w: &World, missile: ObjectId) -> bool {
    w.obj(missile).is_none_or(|o| o.on_map)
}

fn weapon(w: &World, id: ObjectId) -> Option<glasnost_sim::combat::data::WeaponId> {
    glasnost_sim::combat::weapon::would_use(w, id, None).1
}

/// Whether `id` could attack `t` from where it stands by S-0307's range
/// test, the minimum range included.
fn in_range(w: &World, id: ObjectId, t: ObjectId) -> bool {
    range::in_range(w, id, weapon(w, id), Some(Target::Object(t)), None)
}

/// What R23 part a records of one run: per tick, whether the spawner is in
/// range of its target (minimum range included), and whether a spawner shot
/// (a controller target or pending target), a launch, the target and the
/// destination are present.
struct Record {
    ticks: Vec<(bool, bool, bool, bool, bool)>,
}

fn record(w: &mut World, id: ObjectId, t: ObjectId, n: i32) -> Record {
    let missile = w.obj(id).unwrap().spawn.as_ref().map(|_| missile_of(w, id));
    let mut ticks = Vec::new();
    for _ in 0..n {
        let before = in_range(w, id, t);
        testkit::run(w, 1);
        let after = in_range(w, id, t);
        let shot = w.obj(id).unwrap().spawn.as_ref().is_some_and(|c| {
            c.pending.is_some() || c.target.is_some() || c.state != ControllerState::Idle
        });
        let launch = missile.is_some_and(|m| launched(w, m));
        let o = w.obj(id).unwrap();
        ticks.push((
            before || after,
            shot,
            launch,
            o.target == Some(Target::Object(t)),
            o.dest.is_some(),
        ));
        if launch {
            break;
        }
    }
    Record { ticks }
}

/// R23 part a's scene: the actor of type `ty` and a high-health structure
/// of its own house 2 cells away (inside the 4-cell minimum range), with an
/// attack order; `berserk` makes the actor berserk first and gives it the
/// structure as its target in *hunt* (part c).
fn part_a(ty: &str, berserk: bool) -> (World, ObjectId, ObjectId) {
    let r = rules();
    let mut w = game(&r);
    let a = put(&mut w, Kind::Vehicle, ty, 0, 0, 0);
    let s = put(&mut w, Kind::Building, "STRUCT", 0, 2, 0);
    assert!(!in_range(&w, a, s), "inside the minimum range");
    if berserk {
        // spec: S-0610 claim 4 (the berserk state: hunt queued)
        assert!(psychic::enter(&mut w, a, 10_000));
        let o = w.obj_mut(a).unwrap();
        o.beh.current = Some(Behaviour::Hunt);
        o.beh.queued = None;
        o.target = Some(Target::Object(s));
    } else {
        attack(&mut w, a, s);
    }
    (w, a, s)
}

// spec: S-0506 claim 27 (no shot inside the minimum range: the firing check's range test includes the spawner weapon's minimum range, S-0307 claim 9) and scenario R23 part a
#[test]
fn claim_27_the_firing_check_refuses_inside_the_minimum_range() {
    let (mut w, a, s) = part_a("LNM", false);
    w.obj_mut(a).unwrap().target = Some(Target::Object(s));
    // spec: S-0307 claim 9 (minimum range: out of range)
    assert_eq!(
        check(&w, a, Some(Target::Object(s)), 0, true),
        Answer::OutOfRange
    );
    // An unlimited-range spawner weapon passes every target (S-0307 claim 2): claim 27 is for a weapon that is not of unlimited range.
    let mut r = rules();
    r.sim.weapons.get_mut("LaunchM").unwrap().range = dec("-2");
    let mut w = game(&r);
    let a = put(&mut w, Kind::Vehicle, "LNM", 0, 0, 0);
    let s = put(&mut w, Kind::Building, "STRUCT", 0, 2, 0);
    assert!(in_range(&w, a, s), "unlimited range: in range");
}

/// Part a or c's run: no shot and no launch while the spawner is inside the
/// minimum range; it keeps the target and gets a destination, as its twin
/// with an ordinary weapon of the same ranges does; every shot comes in
/// range.
fn close_target_run(berserk: bool) {
    let (mut w, a, s) = part_a("LNM", berserk);
    let (mut tw, ta, ts) = part_a("GUNM", berserk);
    let rec = record(&mut w, a, s, 400);
    let twin = record(&mut tw, ta, ts, 400);
    // spec: S-0506 claim 27 (no fire event and no launch while the distance is below the minimum range)
    for (i, &(ranged, shot, launch, _, _)) in rec.ticks.iter().enumerate() {
        if shot || launch {
            assert!(
                ranged,
                "tick {i}: a shot or launch inside the minimum range"
            );
        }
    }
    // spec: S-0506 claim 27 and S-0215 claim 37 parts d and f (the spawner keeps the target and gets a destination, as for any other target that is not in range)
    let first = rec.ticks.iter().position(|t| t.4).expect("a destination");
    let twin_first = twin.ticks.iter().position(|t| t.4).expect("the twin's");
    assert_eq!(first, twin_first, "the same approach step as its twin");
    assert!(
        rec.ticks[..=first].iter().all(|t| t.3),
        "it keeps the target"
    );
    assert!(!rec.ticks[first].0, "the destination comes while inside");
    // It gets out to the minimum range and launches.
    assert!(
        rec.ticks.last().unwrap().2,
        "it launches once it is out of the minimum range"
    );
    if berserk {
        assert!(psychic::berserk(&w, a), "still berserk");
    }
}

// spec: S-0506 claim 27 (the refusal inside the minimum range and the approach step's outcome: the spawner keeps the target in attack and gets a destination by S-0215 claim 37 f) and scenario R23 part a
#[test]
fn r23_a_no_launch_inside_the_minimum_range_and_the_approach_step() {
    close_target_run(false);
}

// spec: S-0506 claim 27 (being berserk changes neither test: a berserk spawner in hunt, whose type may approach, keeps the target, gets a destination and does not fire inside the minimum range; S-0610 claim 6's berserk-friendly refusal is the only one it adds) and scenario R23 part c
#[test]
fn r23_c_a_berserk_spawner_does_the_same_in_hunt() {
    close_target_run(true);
}

// spec: S-0506 claims 5, 11 and 27 (an idle controller whose adopted target is inside the minimum range at the adopting pass resets instead of launching) and scenario R23 part b
#[test]
fn r23_b_reset_at_adoption_inside_the_minimum_range() {
    for (dx, inside) in [(2, true), (6, false)] {
        let r = rules();
        let mut w = game(&r);
        let a = put(&mut w, Kind::Vehicle, "LNM", 0, 0, 0);
        let v = put(&mut w, Kind::Vehicle, "VICTIM", 0, dx, 0);
        assert_eq!(in_range(&w, a, v), !inside);
        let missile = missile_of(&w, a);
        // A shot hands the target over (claim 3); the next pass adopts it.
        spawn::shot(&mut w, a, Target::Object(v));
        let pass = controller(&w, a).next_pass;
        while w.tick <= pass {
            testkit::run(&mut w, 1);
        }
        let c = controller(&w, a);
        if inside {
            // spec: S-0506 claim 11 (the reset: target and pending cleared, idle)
            assert_eq!(c.state, ControllerState::Idle);
            assert_eq!(c.target, None);
            assert_eq!(c.pending, None);
        } else {
            assert_eq!(c.state, ControllerState::Launching);
            assert_eq!(c.target, Some(Target::Object(v)));
        }
        testkit::run(&mut w, 30);
        // spec: S-0506 claim 27 (no launch in a counted run)
        assert_eq!(launched(&w, missile), !inside, "dx {dx}");
    }
}

// spec: S-0506 claims 5, 6 and 27 (a launching controller does not repeat the range test: a target that comes inside the minimum range after adoption still gets the launch at the next pass, while the spawner's locomotor reports that it is not moving) and scenario R23 part d
#[test]
fn r23_d_a_target_that_comes_close_after_adoption_is_launched_at() {
    let mut counted = 0;
    for lead in 0..6 {
        let r = rules();
        let mut w = game(&r);
        let a = put(&mut w, Kind::Vehicle, "LNM", 0, 0, 0);
        // Just outside the minimum range (4 cells between the centres).
        let v = put(&mut w, Kind::Vehicle, "RUNNER", 0, 4, 0);
        assert!(in_range(&w, a, v));
        let missile = missile_of(&w, a);
        spawn::shot(&mut w, a, Target::Object(v));
        let adopt = controller(&w, a).next_pass;
        while w.tick <= adopt {
            testkit::run(&mut w, 1);
        }
        let c = controller(&w, a);
        assert_eq!(c.state, ControllerState::Launching);
        assert!(in_range(&w, a, v), "at least the minimum range at adoption");
        let next = c.next_pass;
        // The order tick varies; the target drives to a free cell next to the spawner.
        let wait = lead.min(next - w.tick - 1).max(0);
        testkit::run(&mut w, wait);
        open(&mut w);
        assert!(orders::issue(
            &mut w,
            ALPHA,
            Order::Behaviour {
                actor: v,
                behaviour: Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(Target::Cell(testkit::mid(SIZE, 1, 0))),
            },
        ));
        let mut inside_at_next = false;
        let mut moving_at_next = true;
        while w.tick <= next {
            if w.tick == next {
                inside_at_next = !in_range(&w, a, v);
                moving_at_next = occupancy::in_progress(&w, a);
            }
            testkit::run(&mut w, 1);
        }
        if !inside_at_next || moving_at_next {
            continue;
        }
        counted += 1;
        let c = controller(&w, a);
        // spec: S-0506 claim 27 (a launch at that next pass, and no reset)
        assert!(
            launched(&w, missile),
            "lead {lead}: launched at the next pass"
        );
        // spec: S-0506 claims 7 and 8 (the missile is sent at the controller target; with one slot the same pass releases the salvo and the controller cools down, where claim 11's reset would have left it idle with the slot empty)
        assert_eq!(
            w.obj(missile).unwrap().dest,
            Some(Target::Object(v)),
            "lead {lead}: sent at the close target"
        );
        assert_eq!(
            c.state,
            ControllerState::CoolingDown,
            "lead {lead}: no reset"
        );
        assert_eq!(c.slots[0].state, SlotState::OnRail);
    }
    // Every order tick of this scene is a counted run.
    assert_eq!(counted, 6, "counted runs");
}

/// R23 part e: a target of another house (allied with no other house)
/// outside the minimum range; the spawner's housemate stands in the next
/// cell. Returns the health losses of the spawner, the housemate and the
/// target after the first detonation, and the attackers its hits named.
fn own_blast(allies: bool) -> (i32, i32, i32, Vec<Option<ObjectId>>, ObjectId) {
    let r = rules_with(Some(allies));
    let mut w = game(&r);
    let a = put(&mut w, Kind::Vehicle, "LNM", 0, 0, 0);
    let mate = put(&mut w, Kind::Vehicle, "VICTIM", 0, 0, 1);
    let t = put(&mut w, Kind::Building, "STRUCT", 1, 6, 0);
    assert!(in_range(&w, a, t));
    let missile = missile_of(&w, a);
    let hp = |w: &World, id: ObjectId| w.obj(id).map_or(0, |o| o.health);
    let (a0, m0, t0) = (hp(&w, a), hp(&w, mate), hp(&w, t));
    attack(&mut w, a, t);
    let mut attackers = Vec::new();
    let mut gone = false;
    for _ in 0..600 {
        testkit::run(&mut w, 1);
        for e in std::mem::take(&mut w.effects) {
            if let Effect::Hit { attacker, .. } = e {
                attackers.push(attacker);
            }
        }
        if w.obj(missile).is_none() {
            gone = true;
            break;
        }
    }
    assert!(gone, "the rocket detonates");
    (
        a0 - hp(&w, a),
        m0 - hp(&w, mate),
        t0 - hp(&w, t),
        attackers,
        missile,
    )
}

// spec: S-0506 claim 27 (the launcher in its own blast: the missile is the hit's attacker, S-0302 claim 1; area damage leaves out only that source object, S-0303 claim 9; the allies row reads the missile's owner, the spawner's house, claim 9) and scenario R23 part e
// spec: S-0302 claim 8 (stage 2 allies row: a warhead that does not affect allies stops the hit on the spawner and its housemates)
#[test]
fn r23_e_the_launchers_own_blast_with_both_allies_settings() {
    for allies in [true, false] {
        let (spawner, mate, target, attackers, missile) = own_blast(allies);
        assert!(target > 0, "allies {allies}: the target loses health");
        // spec: S-0506 claim 27 (each hit of the rocket names the missile as attacker)
        assert!(!attackers.is_empty());
        assert!(
            attackers.iter().all(|&x| x == Some(missile)),
            "{attackers:?}"
        );
        if allies {
            assert!(spawner > 0, "the spawner takes the hit");
            assert!(mate > 0, "the housemate takes the hit");
        } else {
            assert_eq!(spawner, 0, "the allies row stops the hit on the spawner");
            assert_eq!(mate, 0, "and on its housemate");
        }
    }
}

// spec: S-0506 claim 27 (crushing a berserk object: being berserk does not change S-0210 claim 2's test; a crusher reaching the centre of the cell crushes the berserk object as its twin) and scenario R23 parts f and g
// spec: S-0210 claims 2 and 3 (crushable, not allied; every crushable object of the cell layer less than 128 leptons from the crusher)
#[test]
fn r23_f_g_a_berserk_object_is_crushed_as_its_twin() {
    for (kind, ty) in [(Kind::Infantry, "GRUNT"), (Kind::Vehicle, "LNMC")] {
        for berserk in [false, true] {
            let r = rules();
            let mut w = game(&r);
            let m = put(&mut w, Kind::Vehicle, "MASH", 0, 0, 0);
            let x = put(&mut w, kind, ty, 1, 0, 0);
            if berserk {
                assert!(psychic::enter(&mut w, x, 10_000));
            }
            assert!(occupancy::crushable_by(&w, m, x), "{ty} berserk {berserk}");
            crush::at_cell_centre(&mut w, m);
            assert!(
                w.obj(x).is_none_or(|o| o.health <= 0 || !o.on_map),
                "{ty} berserk {berserk}: crushed"
            );
        }
    }
}

// spec: S-0506 claim 27 (S-0210 claim 11's variants set the crusher's destination to the object itself, never the cell under a spawner; a berserk target is treated the same) and scenario R23 part g
#[test]
fn r23_g_the_crushing_approach_names_the_berserk_spawner_not_its_cell() {
    for berserk in [false, true] {
        let r = rules();
        let mut w = game(&r);
        // A test-only computer-controlled player house keeps the original's attack-target variant (S-0210 claim 11).
        w.houses[0].human = false;
        let m = put(&mut w, Kind::Vehicle, "MASH", 0, 0, 0);
        let x = put(&mut w, Kind::Vehicle, "LNMC", 1, 1, 0);
        if berserk {
            assert!(psychic::enter(&mut w, x, 10_000));
        }
        let o = w.obj_mut(m).unwrap();
        o.target = Some(Target::Object(x));
        o.dest = None;
        glasnost_sim::approach::step(&mut w, m);
        assert_eq!(
            w.obj(m).unwrap().dest,
            Some(Target::Object(x)),
            "berserk {berserk}"
        );
    }
}

// spec: S-0506 claim 27 (the crush response and a spawner's rockets: every rocket hit names the missile as attacker, so the response never names the spawner) and scenario R23 part h
// spec: S-0210 claim 11 and S-0215 claim 39 (the response names the hit's attacker object only)
#[test]
fn r23_h_the_crush_response_never_names_the_spawner() {
    let r = rules_with(Some(false));
    let mut w = game(&r);
    let a = put(&mut w, Kind::Vehicle, "LNN", 0, 0, 0);
    // The crusher's house is computer-controlled (a test-only player house), which S-0210 claim 11's table allows the response.
    w.houses[1].human = false;
    w.houses[1].skill = i32::MAX;
    let m = put(&mut w, Kind::Vehicle, "MASH", 1, 1, 0);
    let h0 = w.obj(m).unwrap().health;
    attack(&mut w, a, m);
    let mut hits = Vec::new();
    for _ in 0..400 {
        testkit::run(&mut w, 1);
        for e in std::mem::take(&mut w.effects) {
            if let Effect::Hit { id, attacker, .. } = e
                && id == m
            {
                hits.push(attacker);
            }
        }
        if let Some(o) = w.obj(m) {
            assert_ne!(o.dest, Some(Target::Object(a)), "never the spawner");
        }
    }
    assert!(!hits.is_empty(), "the rockets hit the crusher");
    assert!(w.obj(m).is_none_or(|o| o.health < h0));
    for x in &hits {
        let x = x.expect("an attacker");
        assert_ne!(x, a, "never the spawner");
        // The attacker is a rocket missile of the spawner's house.
        assert!(w.obj(x).is_none_or(|o| o.kind == Kind::Aircraft));
    }
}
