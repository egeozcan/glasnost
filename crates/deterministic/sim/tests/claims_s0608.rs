//! Claim tests of S-0608 (drain) on an empty synthetic map with invented
//! types: every name and number here is made up; no game data. Houses Alpha
//! (0) and Beta (1) of the testkit's sandbox game, not allied. The ledger is
//! `docs/m7-claims/S-0608.md`.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    AnimationType, BuildingType, Decimal, LocomotionClass, Locomotor, MovementZone, ProjectileType,
    UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::{fire, weapon};
use glasnost_sim::drain::{self, DrainEvent};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use glasnost_sim::{approach, econ, mind, objects, save, testkit, tick};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;
/// The invented payout period *P* and amount *A*.
const P: i32 = 10;
const A: i32 = 25;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn building(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut BuildingType)) {
    let mut t = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        ..Default::default()
    };
    // A build-up sprite, so that the building can be sold (S-0408 claim 4).
    t.econ.buildup = Some(true);
    t.econ.buildup_frames = Some(4);
    f(&mut t);
    register(r, "buildings", name);
    r.sim.buildings.insert(name.into(), t);
}

fn vehicle(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.vehicles["CRUSHER"].clone();
    t.crusher = None;
    t.strength = Some(300);
    f(&mut t);
    register(r, "vehicles", name);
    r.sim.vehicles.insert(name.into(), t);
}

/// A balloon-hovering jumpjet vehicle (S-0502, S-0307 claim 18).
fn hover_disc() -> UnitType {
    UnitType {
        strength: Some(300),
        speed: Some(4),
        locomotion_class: Some(LocomotionClass::Hover),
        movement_zone: Some(MovementZone::Fly),
        locomotor: Some(Locomotor::Jumpjet),
        jumpjet: Some(true),
        sight: Some(3),
        jumpjet_speed: Some(14),
        jumpjet_climb: Some(dec("5.0")),
        jumpjet_crash: Some(dec("6.0")),
        jumpjet_height: Some(500),
        jumpjet_accel: Some(dec("2.0")),
        jumpjet_wobbles: Some(dec("0.15")),
        jumpjet_deviation: Some(40),
        jumpjet_turn_rate: Some(4),
        balloon_hover: Some(true),
        mobile_fire: Some(true),
        primary: Some("Gun".into()),
        secondary: Some("SUCKER".into()),
        ..Default::default()
    }
}

/// The combat ruleset with:
/// - the vertical projectile DROP and the draining weapon SUCKER (range
///   1.5, fires in any direction);
/// - DISC: a ground vehicle, primary Gun, secondary SUCKER; HDISC: the
///   balloon-hovering jumpjet of the same weapons; LEECH: an infantry unit
///   whose primary is SUCKER; DCART: a drainable vehicle;
/// - buildings (2 × 2 unless said, build-up sprite): PLANT (output 100,
///   drainable), REF (3 × 2, resource destination, drainable), BOTH
///   (output 50, resource destination, drainable), SHACK (resource
///   destination, not drainable), PSI (a controller of capacity 3,
///   drainable), CASHREF (resource destination, drainable, a periodic
///   income of 100 every 10 ticks), and the testkit's TOWER made drainable;
/// - SPAWNY and BOOMER: DISCs whose secondary is SUCKER made a spawner
///   weapon (SUCKSPAWN) or a suicide weapon (SUCKBOOM); VAN: a transport
///   with one place;
/// - the payout period 10, the amount 25 and the drain animation SUCK
///   (8 frames).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.projectiles.insert(
        "DROP".into(),
        ProjectileType {
            vertical: Some(true),
            inviso: Some(true),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "SUCKER".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(50),
            range: Some(dec("1.5")),
            speed: Some(100),
            projectile: Some("DROP".into()),
            warhead: Some("SA".into()),
            drain_weapon: Some(true),
            // It fires in any direction (S-0310 claim 2): a drainer over its target has no direction to face.
            omni_fire: Some(true),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            verses: Some(vec![dec("1"); 11]),
            mind_control: Some(true),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Ray3".into(),
        WeaponType {
            damage: Some(3),
            rof: Some(30),
            range: Some(dec("10")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MIND".into()),
            ..Default::default()
        },
    );
    vehicle(&mut r, "DISC", |t| {
        t.primary = Some("Gun".into());
        t.secondary = Some("SUCKER".into());
    });
    register(&mut r, "vehicles", "HDISC");
    r.sim.vehicles.insert("HDISC".into(), hover_disc());
    vehicle(&mut r, "DCART", |t| {
        t.primary = None;
        t.combat.drainable = Some(true);
    });
    vehicle(&mut r, "BOX", |t| t.primary = None);
    let mut leech = r.sim.infantry["WALKER"].clone();
    leech.primary = Some("SUCKER".into());
    register(&mut r, "infantry", "LEECH");
    r.sim.infantry.insert("LEECH".into(), leech);
    building(&mut r, "PLANT", |t| {
        t.econ.power = Some(100);
        t.combat.drainable = Some(true);
    });
    building(&mut r, "REF", |t| {
        t.foundation = Some([3, 2]);
        t.econ.resource_destination = Some(true);
        t.combat.drainable = Some(true);
    });
    building(&mut r, "BOTH", |t| {
        t.econ.power = Some(50);
        t.econ.resource_destination = Some(true);
        t.combat.drainable = Some(true);
    });
    building(&mut r, "SHACK", |t| {
        t.econ.resource_destination = Some(true);
    });
    building(&mut r, "PSI", |t| {
        t.primary = Some("Ray3".into());
        t.combat.immune_to_psionics = Some(false);
        t.combat.drainable = Some(true);
    });
    r.sim.buildings.get_mut("TOWER").unwrap().combat.drainable = Some(true);
    register(&mut r, "animations", "SUCK");
    r.sim.animations.insert(
        "SUCK".into(),
        AnimationType {
            frame_count: Some(8),
            random_rate: Some([450, 225]),
            ..Default::default()
        },
    );
    // Claims 3 and 16: a draining weapon that is also a spawner weapon, and
    // one that is also a suicide weapon, on DISC-like vehicles.
    let mut spawny = r.sim.weapons["SUCKER"].clone();
    spawny.spawner = Some(true);
    r.sim.weapons.insert("SUCKSPAWN".into(), spawny);
    let mut boom = r.sim.weapons["SUCKER"].clone();
    boom.suicide = Some(true);
    r.sim.weapons.insert("SUCKBOOM".into(), boom);
    vehicle(&mut r, "SPAWNY", |t| {
        t.primary = Some("Gun".into());
        t.secondary = Some("SUCKSPAWN".into());
    });
    vehicle(&mut r, "BOOMER", |t| {
        t.primary = Some("Gun".into());
        t.secondary = Some("SUCKBOOM".into());
    });
    // Claim 17: a transport with one place.
    vehicle(&mut r, "VAN", |t| {
        t.primary = None;
        t.passengers = Some(1);
        t.size_limit = Some(dec("2"));
    });
    // Claim 15: a drained resource destination with a periodic income of 100.
    building(&mut r, "CASHREF", |t| {
        t.econ.resource_destination = Some(true);
        t.combat.drainable = Some(true);
        t.produce_cash_amount = Some(100);
        t.produce_cash_delay = Some(P);
    });
    let g = &mut r.sim.general;
    g.drain_money_frame_delay = Some(P);
    g.drain_money_amount = Some(A);
    g.drain_animation_type = Some("SUCK".into());
    r
}

fn game_of(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 7)
}

fn game() -> World {
    game_of(&rules())
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

fn obj(x: ObjectId) -> Option<Target> {
    Some(Target::Object(x))
}

/// Moves `id` to the centre of `cell` at height `z` above the ground.
/// A move on the map, not a storage: the links are kept across the
/// helper's take-out and put-back, which stand for the drainer's walk
/// (S-0608 claim 17 ends them only at temporary off-map storage).
fn shift(w: &mut World, id: ObjectId, cell: CellXY, z: i32) {
    let (links, effects) = (w.drain.clone(), w.effects.len());
    objects::take_inside(w, id);
    w.drain = links;
    w.effects.truncate(effects);
    let p = Pos::centre(cell, 0);
    let ground = w.surface_at(p.x, p.y);
    objects::place(w, id, Pos::new(p.x, p.y, ground + z), 0);
}

/// A drain attempt: `d` aims at `t` and makes a shot with its slot `slot`.
fn attempt_slot(w: &mut World, d: ObjectId, slot: usize, t: Target) -> bool {
    w.obj_mut(d).unwrap().target = Some(t);
    fire::shot(w, d, slot, t)
}

/// A drain attempt with the DISC's draining secondary.
fn attempt(w: &mut World, d: ObjectId, b: ObjectId) -> bool {
    attempt_slot(w, d, 1, Target::Object(b))
}

fn events(w: &World) -> Vec<DrainEvent> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Drain { event, .. } => Some(event.clone()),
            _ => None,
        })
        .collect()
}

fn credits(w: &World, h: u8) -> i32 {
    w.houses[usize::from(h)].credits
}

fn set_credits(w: &mut World, h: u8, c: i32) {
    w.houses[usize::from(h)].credits = c;
}

fn round_trip(w: &World) {
    let bytes = save::save(w);
    let l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(w));
    assert_eq!(l.drain, w.drain);
}

fn lethal(w: &World, id: ObjectId) -> Hit {
    Hit {
        amount: w.obj(id).unwrap().health * 10,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    }
}

/// Orders `d` to attack `t` (after the order block of the game's start)
/// and runs the tick in which the order applies.
fn attack(w: &mut World, d: ObjectId, t: ObjectId) {
    while orders::blocked(w) {
        tick::step(w);
    }
    let house = w.obj(d).unwrap().owner.unwrap();
    assert!(orders::issue(
        w,
        house,
        Order::Behaviour {
            actor: d,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: obj(t),
            dest: None,
        },
    ));
    // The order applies after the turn pass (S-0203 claim 7).
    tick::step(w);
}

/// Steps until `done` holds, at most `max` ticks; whether it held.
fn until(w: &mut World, max: i32, done: impl Fn(&World) -> bool) -> bool {
    for _ in 0..max {
        if done(w) {
            return true;
        }
        tick::step(w);
    }
    done(w)
}

/// A DISC of Alpha linked to a `name` building of Beta at (0, 0).
fn linked(name: &str) -> (World, ObjectId, ObjectId) {
    linked_in(game(), name)
}

fn linked_in(mut w: World, name: &str) -> (World, ObjectId, ObjectId) {
    let b = put(&mut w, Kind::Building, name, BETA, 0, 0);
    let d = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 0, 0);
    attempt(&mut w, d, b);
    assert_eq!(drain::partner(&w, d), Some(b), "linked");
    w.effects.clear();
    (w, d, b)
}

// ---------------------------------------------------------------------------
// Claims 1 to 6: the attempt and the link.

// spec: S-0608 claim 1 (a drain attempt replaces the shot: no projectile, no shot damage, no ordinary-shot draws; no rearm countdown, no burst advance, no ammunition; the target is cleared against a drainable object and the destination is not touched)
#[test]
fn claim_1_a_drain_attempt_replaces_the_shot() {
    // No drain animation, so that forming a link takes no draws either.
    let mut r = rules();
    r.sim.general.drain_animation_type = None;
    let mut w = game_of(&r);
    let b = put(&mut w, Kind::Building, "PLANT", BETA, 0, 0);
    let d = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 0, 0);
    w.obj_mut(d).unwrap().dest = Some(Target::Cell(at(5, 5)));
    let before = w.obj(d).unwrap().combat.clone();
    let rng = w.rng.clone();
    let health = w.obj(b).unwrap().health;
    assert!(!attempt(&mut w, d, b), "not counted as a shot made");
    let o = w.obj(d).unwrap();
    assert_eq!(o.target, None, "cleared");
    assert_eq!(
        o.dest,
        Some(Target::Cell(at(5, 5))),
        "the destination is kept"
    );
    assert_eq!(o.combat.rearm_until, before.rearm_until, "no countdown");
    assert_eq!(o.combat.burst, 0);
    assert_eq!(o.combat.ammo, before.ammo, "no ammunition");
    assert!(w.projectiles.is_empty(), "no projectile");
    assert_eq!(w.obj(b).unwrap().health, health, "no damage");
    assert_eq!(w.rng, rng, "no draws");
    assert!(drain::being_drained(&w, b));
    // A second attempt is limited only by the firing check, not by the rate of fire.
    let b2 = put(&mut w, Kind::Building, "PLANT", BETA, 4, 4);
    assert_ne!(
        check::check(&w, d, obj(b2), 1, false),
        Answer::Rearming,
        "no countdown runs"
    );
}

// spec: S-0608 claim 1 (against a cell, or an object whose type is not drainable, nothing happens and the firer keeps its target; against a drainable object the target is cleared whether or not a link forms, whatever the fire-once flag says)
#[test]
fn claim_1_cells_and_undrainable_objects_keep_the_target() {
    let mut w = game();
    let d = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 0, 0);
    let cell = Target::Cell(at(0, 0));
    assert!(!attempt_slot(&mut w, d, 1, cell));
    assert_eq!(w.obj(d).unwrap().target, Some(cell));
    let shack = put(&mut w, Kind::Building, "SHACK", BETA, 0, 0);
    assert!(!attempt(&mut w, d, shack));
    assert_eq!(w.obj(d).unwrap().target, obj(shack));
    assert!(w.drain.links.is_empty());
    // Fire-once off or on: the target goes against a drainable object, linked or not.
    for once in [false, true] {
        let mut r = rules();
        r.sim.weapons.get_mut("SUCKER").unwrap().fire_once = Some(once);
        let mut w = game_of(&r);
        let far = put(&mut w, Kind::Building, "PLANT", BETA, 4, 0);
        let d = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 0, 0);
        attempt(&mut w, d, far);
        assert_eq!(w.obj(d).unwrap().target, None, "fire once {once}");
        assert!(!drain::draining(&w, d));
    }
}

// spec: S-0608 claim 2 (a link forms only if the cell containing the drainer's position, height aside, is any cell of the target building's foundation; from a neighbouring cell the target is lost without a link; a drainable mobile object never gets a link)
#[test]
fn claim_2_only_over_the_foundation() {
    // REF is 3 × 2: every one of its six cells will do.
    for dx in 0..3 {
        for dy in 0..2 {
            let mut w = game();
            let b = put(&mut w, Kind::Building, "REF", BETA, 0, 0);
            let d = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 9, 9);
            shift(&mut w, d, at(dx, dy), 600);
            attempt(&mut w, d, b);
            assert_eq!(drain::drainer_of(&w, b), Some(d), "cell ({dx}, {dy})");
        }
    }
    for (dx, dy) in [(3, 0), (0, 2), (-1, 0), (0, -1)] {
        let mut w = game();
        let b = put(&mut w, Kind::Building, "REF", BETA, 0, 0);
        let d = put(&mut w, Kind::Vehicle, "DISC", ALPHA, dx, dy);
        attempt(&mut w, d, b);
        assert!(!drain::being_drained(&w, b), "cell ({dx}, {dy})");
        assert_eq!(w.obj(d).unwrap().target, None, "the target is lost");
    }
    // A drainable vehicle in the drainer's own cell: the check allows it, no link.
    let mut w = game();
    let v = put(&mut w, Kind::Vehicle, "DCART", BETA, 0, 0);
    let d = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 0, 0);
    assert_ne!(check::check(&w, d, obj(v), 1, false), Answer::Illegal);
    attempt(&mut w, d, v);
    assert!(w.drain.links.is_empty());
    assert_eq!(w.obj(d).unwrap().target, None);
}

// spec: S-0608 claim 3 (illegal at rank 4: (a) the building the firer drains, whatever the weapon; (b) a draining weapon against an object already being drained; (c) a draining weapon against a type that is not drainable; b and c not against a cell; no alliance test)
#[test]
fn claim_3_firing_check_refusals() {
    let (mut w, d, b) = linked("PLANT");
    // (a) whatever the weapon.
    assert_eq!(check::check(&w, d, obj(b), 0, false), Answer::Illegal);
    assert_eq!(check::check(&w, d, obj(b), 1, false), Answer::Illegal);
    // (b) another drainer against the held building, with the draining weapon only.
    let d2 = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 1, 1);
    assert_eq!(check::check(&w, d2, obj(b), 1, false), Answer::Illegal);
    assert_ne!(check::check(&w, d2, obj(b), 0, false), Answer::Illegal);
    // The weapon choice does not test the drained state: it picks the draining weapon.
    assert_eq!(weapon::choose_slot(&w, d2, obj(b)), 1);
    // (c) a type that is not drainable, with the draining weapon only.
    let shack = put(&mut w, Kind::Building, "SHACK", BETA, 4, 0);
    assert_eq!(check::check(&w, d2, obj(shack), 1, false), Answer::Illegal);
    assert_ne!(check::check(&w, d2, obj(shack), 0, false), Answer::Illegal);
    // Not against a cell.
    assert_ne!(
        check::check(&w, d2, Some(Target::Cell(at(6, 6))), 1, false),
        Answer::Illegal
    );
    // No alliance or ownership test: an own drainable building is not refused here.
    let own = put(&mut w, Kind::Building, "PLANT", ALPHA, -4, -4);
    assert_ne!(check::check(&w, d2, obj(own), 1, false), Answer::Illegal);
    // The weapon choice is the only alliance test: it does not pick the draining weapon against it.
    assert_eq!(weapon::choose_slot(&w, d2, obj(own)), 0);
}

// spec: S-0608 claim 4 and S-0308 table P row 8 (a draining drainer uses its other weapon against every target, so an attack order on another building does not move the link)
#[test]
fn claim_4_one_to_one() {
    let (mut w, d, b) = linked("PLANT");
    let b2 = put(&mut w, Kind::Building, "PLANT", BETA, 4, 0);
    assert_eq!(weapon::choose_slot(&w, d, obj(b2)), 0, "the other weapon");
    attack(&mut w, d, b2);
    testkit::run(&mut w, 10);
    assert_eq!(drain::partner(&w, d), Some(b));
    assert!(!drain::being_drained(&w, b2));
    // Once the link ends the row picks the draining weapon again (claim 14).
    drain::end(&mut w, b);
    assert_eq!(weapon::choose_slot(&w, d, obj(b2)), 1);
}

// spec: S-0608 Edge cases and claim 4 (quirks.drain_relink_strands_previous: on, a second link strands the first building, still drained and paying, out of the alliance test and the position check; off, forming the second link first ends the first)
#[test]
fn quirk_relink_strands_previous_both_positions() {
    for strands in [true, false] {
        let mut r = rules();
        r.globals.quirks.drain_relink_strands_previous = strands;
        let (mut w, d, first) = linked_in(game_of(&r), "REF");
        let second = put(&mut w, Kind::Building, "PLANT", BETA, 6, 0);
        shift(&mut w, d, at(6, 0), 0);
        // The weapon choice never picks it for a draining firer; the attempt is made directly (data that picks it, S-0308 row 16).
        attempt(&mut w, d, second);
        assert_eq!(drain::partner(&w, d), Some(second), "strands {strands}");
        assert_eq!(
            drain::being_drained(&w, first),
            strands,
            "strands {strands}"
        );
        let ended = events(&w).contains(&DrainEvent::Ended);
        assert_eq!(ended, !strands);
        if strands {
            // spec: S-0608 claim 3 (Q-1441: part a refuses only the current building; the stranded earlier one is refused to the draining weapon by part b, and the other weapon is not refused)
            assert_ne!(check::check(&w, d, obj(first), 0, false), Answer::Illegal);
            assert_eq!(check::check(&w, d, obj(first), 1, false), Answer::Illegal);
            assert_eq!(check::check(&w, d, obj(second), 0, false), Answer::Illegal);
            // The stranded first building keeps paying the drainer's house.
            set_credits(&mut w, BETA, 1000);
            let start = credits(&w, ALPHA);
            testkit::run(&mut w, 2 * P);
            assert_eq!(credits(&w, ALPHA) - start, 2 * A);
            // Neither the alliance test nor the position check looks at it.
            assert!(drain::being_drained(&w, first));
        }
        round_trip(&w);
    }
}

// spec: S-0608 claim 5 (at the attempt that forms a link: the building's house is due to work out its power, a controller building releases all its victims, the drain animation starts on the drainer with its creation draw; with no drain animation nothing is shown; a failed attempt releases nothing)
#[test]
fn claim_5_when_the_link_forms() {
    let mut w = game();
    let psi = put(&mut w, Kind::Building, "PSI", BETA, 0, 0);
    let victim = put(&mut w, Kind::Vehicle, "BOX", ALPHA, 6, 6);
    assert!(mind::take(&mut w, psi, victim));
    let d = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 3, 0);
    // From a neighbouring cell: no link, no release.
    attempt(&mut w, d, psi);
    assert_eq!(mind::controlled_by(&w, victim), Some(psi));
    shift(&mut w, d, at(1, 1), 0);
    w.houses[usize::from(BETA)].econ.power.due = false;
    w.effects.clear();
    attempt(&mut w, d, psi);
    assert!(drain::being_drained(&w, psi));
    assert!(w.houses[usize::from(BETA)].econ.power.due, "due");
    assert_eq!(mind::controlled_by(&w, victim), None, "released");
    assert_eq!(w.obj(victim).unwrap().owner, Some(HouseId(ALPHA)));
    let formed: Vec<_> = events(&w)
        .into_iter()
        .filter(|e| matches!(e, DrainEvent::Formed { .. }))
        .collect();
    assert_eq!(formed.len(), 1);
    let DrainEvent::Formed { anim, delay } = &formed[0] else {
        unreachable!()
    };
    assert_eq!(anim.as_deref(), Some("SUCK"));
    assert!(delay.is_some(), "its creation draw");
    // No drain animation in the data: nothing is shown.
    let mut r = rules();
    r.sim.general.drain_animation_type = None;
    let (w, _, _) = {
        let mut w = game_of(&r);
        let b = put(&mut w, Kind::Building, "PLANT", BETA, 0, 0);
        let d = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 0, 0);
        attempt(&mut w, d, b);
        (w, d, b)
    };
    assert!(events(&w).contains(&DrainEvent::Formed {
        anim: None,
        delay: None
    }));
}

// spec: S-0608 claims 2 and 6, S-0215 claim 37 (h) and S-0405 claim 5 (end to end: a balloon-hovering drainer ordered to attack a building takes it as its destination and flies over it; with a range of 0.1 cells it forms a link over the foundation, its target is then cleared, it stays over the building and the house's output is 0 at its next power update; with a range that reaches from the neighbouring cell it fires there and loses its target without draining, keeping its destination)
#[test]
fn claim_6_a_hover_disc_drains_end_to_end() {
    for (range, links) in [("0.1", true), ("1.5", false)] {
        let mut r = rules();
        r.sim.weapons.get_mut("SUCKER").unwrap().range = Some(dec(range));
        let mut w = game_of(&r);
        let b = put(&mut w, Kind::Building, "PLANT", BETA, 0, 0);
        let d = put(&mut w, Kind::Vehicle, "HDISC", ALPHA, -6, 0);
        testkit::run(&mut w, 2);
        assert_eq!(w.houses[usize::from(BETA)].econ.power.output, 100);
        attack(&mut w, d, b);
        assert!(
            until(&mut w, 1500, |w| w.obj(d).unwrap().target.is_none()),
            "range {range}: an attempt"
        );
        assert_eq!(drain::draining(&w, d), links, "range {range}");
        if !links {
            assert_eq!(w.obj(d).unwrap().dest, obj(b), "the destination is kept");
            assert!(!foundation(&w, b).contains(&w.obj(d).unwrap().pos.cell()));
            continue;
        }
        assert!(foundation(&w, b).contains(&w.obj(d).unwrap().pos.cell()));
        testkit::run(&mut w, 100);
        assert_eq!(drain::partner(&w, d), Some(b), "it stays over the building");
        assert_eq!(w.houses[usize::from(BETA)].econ.power.output, 0, "blackout");
        round_trip(&w);
    }
}

fn foundation(w: &World, b: ObjectId) -> Vec<CellXY> {
    let o = w.obj(b).unwrap();
    econ::placement::foundation_cells(w, o.type_index, o.pos.cell())
}

// spec: S-0608 claim 6 and S-0215 claim 37 (d) (a draining drainer takes no ordinary approach step: an out-of-range target of an object in guard is dropped)
#[test]
fn claim_6_no_approach_while_draining() {
    for draining in [false, true] {
        let (mut w, d, b) = linked("PLANT");
        if !draining {
            drain::end(&mut w, b);
        }
        let far = put(&mut w, Kind::Vehicle, "BOX", BETA, 10, 0);
        w.obj_mut(d).unwrap().target = obj(far);
        approach::step(&mut w, d);
        assert_eq!(
            w.obj(d).unwrap().target.is_none(),
            draining,
            "draining {draining}"
        );
    }
}

// ---------------------------------------------------------------------------
// Claims 7 to 10: payouts.

/// The payouts of the next `n` ticks, with their tick numbers.
fn payouts(w: &mut World, n: i32) -> Vec<(i32, i32, HouseId, HouseId)> {
    let mut out = Vec::new();
    for _ in 0..n {
        w.effects.clear();
        let t = w.tick;
        tick::step(w);
        for e in events(w) {
            if let DrainEvent::Paid {
                amount,
                payer,
                recipient,
            } = e
            {
                out.push((t, amount, payer, recipient));
            }
        }
    }
    out
}

// spec: S-0608 claim 7 (a drained resource-destination building pays t = min(A, C) from its house to the drainer's in its own turn of every tick that is a multiple of |P|; a type without the flag pays nothing even when drainable; power and payouts are independent)
#[test]
fn claim_7_who_pays_when_how_much() {
    let (mut w, _, _) = linked("REF");
    set_credits(&mut w, BETA, 1000);
    set_credits(&mut w, ALPHA, 0);
    let p = payouts(&mut w, 3 * P);
    assert_eq!(p.len(), 3);
    for (t, amount, payer, recipient) in &p {
        assert_eq!(t.rem_euclid(P), 0, "tick {t}");
        assert_eq!(
            (*amount, *payer, *recipient),
            (A, HouseId(BETA), HouseId(ALPHA))
        );
    }
    assert_eq!(credits(&w, BETA), 1000 - 3 * A);
    assert_eq!(credits(&w, ALPHA), 3 * A);
    // Drainable without the flag: no payout.
    let (mut w, _, _) = linked("PLANT");
    set_credits(&mut w, BETA, 1000);
    assert!(payouts(&mut w, 3 * P).is_empty());
    assert_eq!(credits(&w, BETA), 1000);
    // Output and the flag: both the blackout and the payouts.
    let (mut w, _, _) = linked("BOTH");
    set_credits(&mut w, BETA, 1000);
    assert_eq!(payouts(&mut w, 2 * P).len(), 2);
    assert_eq!(w.houses[usize::from(BETA)].econ.power.output, 0);
}

// spec: S-0608 claim 8 and F1 (the payout ticks are fixed by the tick number alone, not by the link: the first payout of a link comes 0 to |P| ticks after it forms)
#[test]
fn claim_8_cadence_follows_the_clock() {
    for offset in 0..P {
        let mut w = game();
        testkit::run(&mut w, offset + 1);
        let (mut w, _, _) = linked_in(w, "REF");
        set_credits(&mut w, BETA, 1000);
        let formed = w.tick;
        let p = payouts(&mut w, P + 1);
        let first = p.first().expect("a payout").0;
        assert_eq!(first.rem_euclid(P), 0);
        assert!((0..=P).contains(&(first - formed)), "offset {offset}");
    }
}

// spec: S-0608 claim 9 and F2 (available credits below A and above 0: the payout takes all of them; 0: it moves nothing and the link continues; none is refused, postponed or made up)
#[test]
fn claim_9_insufficient_funds() {
    let (mut w, d, b) = linked("REF");
    set_credits(&mut w, BETA, 7);
    let p = payouts(&mut w, P);
    assert_eq!(p.iter().map(|x| x.1).collect::<Vec<_>>(), vec![7]);
    assert_eq!(credits(&w, BETA), 0);
    let p = payouts(&mut w, P);
    assert_eq!(p.iter().map(|x| x.1).collect::<Vec<_>>(), vec![0]);
    assert_eq!(drain::partner(&w, d), Some(b), "the link continues");
    // Money arriving later is not used to make up the missed amount.
    set_credits(&mut w, BETA, 1000);
    let p = payouts(&mut w, P);
    assert_eq!(p.iter().map(|x| x.1).collect::<Vec<_>>(), vec![A]);
}

// spec: S-0608 claim 10 (a drained building being erased makes no payout and keeps its link; being switched off and damage do not stop payouts)
#[test]
fn claim_10_erasure_skips_the_payout() {
    let (mut w, d, b) = linked("REF");
    let pay_tick = (w.tick / P + 1) * P;
    while w.tick < pay_tick {
        tick::step(&mut w);
    }
    set_credits(&mut w, BETA, 1000);
    w.temporal.locks.insert(
        b,
        glasnost_sim::temporal::Lock {
            leader: d,
            joiners: Vec::new(),
            amount: 0,
            was_off: false,
            suspended: false,
        },
    );
    drain::building_turn(&mut w, b);
    assert_eq!(credits(&w, BETA), 1000, "no payout while being erased");
    assert_eq!(drain::partner(&w, d), Some(b), "the link stays");
    w.temporal.locks.clear();
    // Switched off and damaged: still paying.
    {
        let o = w.obj_mut(b).unwrap();
        o.econ.building.switched_off = true;
        o.health = 100;
    }
    drain::building_turn(&mut w, b);
    assert_eq!(credits(&w, BETA), 1000 - A);
}

// ---------------------------------------------------------------------------
// Claims 11 to 14: the end of the link.

// spec: S-0608 claim 11 (in every drainer turn the link ends if the building's house allies the drainer's house, the same house included, read in that direction only; an owner change to a house that does not ally the other party leaves the link, and the new house pays)
#[test]
fn claim_11_alliance_ends_the_link() {
    // One way, the drainer's house allying the building's: the link stays.
    let (mut w, d, b) = linked("REF");
    w.houses[usize::from(ALPHA)].allies.push(HouseId(BETA));
    w.houses[usize::from(ALPHA)].allies.sort();
    testkit::run(&mut w, 3);
    assert_eq!(drain::partner(&w, d), Some(b));
    // The building's house allying the drainer's: it ends at the drainer's next turn.
    w.houses[usize::from(BETA)].allies.push(HouseId(ALPHA));
    w.houses[usize::from(BETA)].allies.sort();
    w.effects.clear();
    tick::step(&mut w);
    assert!(!drain::draining(&w, d));
    assert!(events(&w).contains(&DrainEvent::Ended));
    // Captured by the drainer's own house.
    let (mut w, d, b) = linked("REF");
    w.obj_mut(b).unwrap().owner = Some(HouseId(ALPHA));
    tick::step(&mut w);
    assert!(!drain::draining(&w, d), "the same house");
    // An owner change to a house that allies neither: the link stays, and the new house pays.
    let (mut w, d, b) = linked_in(testkit::game_with_non_players(&rules(), SIZE, 7), "REF");
    let third = HouseId(2);
    w.obj_mut(b).unwrap().owner = Some(third);
    set_credits(&mut w, 2, 1000);
    let p = payouts(&mut w, P);
    assert_eq!(p[0].2, third);
    assert_eq!(drain::partner(&w, d), Some(b));
}

// spec: S-0608 claim 12 and F3 (a vehicle drainer checks in its turn of every tick with n mod 16 = 2: off the building the link ends there, not before; moving within the foundation keeps it; the payouts and blackout go on meanwhile)
#[test]
fn claim_12_position_check() {
    let (mut w, d, b) = linked("REF");
    // Within the foundation: kept through several checks.
    shift(&mut w, d, at(2, 1), 0);
    testkit::run(&mut w, 40);
    assert_eq!(drain::partner(&w, d), Some(b));
    // Off the building: kept until the next tick with n mod 16 = 2.
    shift(&mut w, d, at(6, 6), 0);
    let mut ended_at = None;
    for _ in 0..20 {
        let t = w.tick;
        tick::step(&mut w);
        if !drain::draining(&w, d) {
            ended_at = Some(t);
            break;
        }
        assert!(drain::being_drained(&w, b));
    }
    let t = ended_at.expect("ended");
    assert_eq!(t.rem_euclid(drain::CHECK_PERIOD), drain::CHECK_PHASE);
    assert_eq!((drain::CHECK_PERIOD, drain::CHECK_PHASE), (16, 2));
    // Over another building: the building that covers the cell is not the one it drains.
    let (mut w, d, _) = linked("REF");
    put(&mut w, Kind::Building, "PLANT", BETA, 8, 0);
    shift(&mut w, d, at(8, 0), 0);
    testkit::run(&mut w, 17);
    assert!(!drain::draining(&w, d));
}

// spec: S-0608 claim 12 and Edge cases (quirks.drain_position_check_vehicles_only: on, a drainer that is not a vehicle never checks and keeps its link after it moves away; off, every drainer checks)
#[test]
fn quirk_position_check_vehicles_only_both_positions() {
    for vehicles_only in [true, false] {
        let mut r = rules();
        r.globals.quirks.drain_position_check_vehicles_only = vehicles_only;
        let mut w = game_of(&r);
        let b = put(&mut w, Kind::Building, "PLANT", BETA, 0, 0);
        let l = put(&mut w, Kind::Infantry, "LEECH", ALPHA, 0, 0);
        attempt_slot(&mut w, l, 0, Target::Object(b));
        assert_eq!(drain::partner(&w, l), Some(b));
        shift(&mut w, l, at(6, 6), 0);
        // Hold it there.
        let t = w.tick;
        w.obj_mut(l).unwrap().beh.force(Behaviour::Sleep, t, true);
        testkit::run(&mut w, 40);
        assert_eq!(
            drain::draining(&w, l),
            vehicles_only,
            "vehicles only {vehicles_only}"
        );
    }
}

// spec: S-0608 claim 13 (a killing hit on either party ends the link at once, before the death effects; removal in any other way ends it too)
#[test]
fn claim_13_destruction_and_removal() {
    let (mut w, d, b) = linked("REF");
    let h = lethal(&w, d);
    damage::apply(&mut w, d, h);
    assert!(!drain::being_drained(&w, b), "the drainer's killing hit");
    let (mut w, d, b) = linked("REF");
    let h = lethal(&w, b);
    damage::apply(&mut w, b, h);
    assert!(!drain::draining(&w, d), "the building's killing hit");
    let (mut w, d, b) = linked("REF");
    objects::remove(&mut w, b);
    assert!(!drain::draining(&w, d), "the building removed");
    let (mut w, d, b) = linked("REF");
    objects::remove(&mut w, d);
    assert!(!drain::being_drained(&w, b), "the drainer removed");
    assert!(w.drain.links.is_empty() && w.drain.partner.is_empty());
}

// spec: S-0608 claim 13 and Edge cases (a stranded first link does not end at its drainer's killing hit, only at its removal; the stranded building's killing hit or removal also ends the drainer's newer link)
#[test]
fn claim_13_the_stranded_first_link() {
    let stranded = || {
        let (mut w, d, first) = linked("REF");
        let second = put(&mut w, Kind::Building, "PLANT", BETA, 6, 0);
        shift(&mut w, d, at(6, 0), 0);
        attempt(&mut w, d, second);
        assert!(drain::being_drained(&w, first) && drain::being_drained(&w, second));
        (w, d, first, second)
    };
    // The drainer's killing hit: only the newer link ends at once.
    let (mut w, d, first, second) = stranded();
    let h = lethal(&w, d);
    damage::apply(&mut w, d, h);
    assert!(!drain::being_drained(&w, second));
    if w.obj(d).is_some() {
        assert!(drain::being_drained(&w, first), "until removal");
        objects::remove(&mut w, d);
    }
    assert!(!drain::being_drained(&w, first), "removal ends it");
    // The stranded building's killing hit tears down the newer link too.
    let (mut w, d, first, second) = stranded();
    let h = lethal(&w, first);
    damage::apply(&mut w, first, h);
    assert!(!drain::being_drained(&w, second));
    assert!(!drain::draining(&w, d));
    // Its removal likewise.
    let (mut w, d, first, second) = stranded();
    objects::remove(&mut w, first);
    assert!(!drain::being_drained(&w, second));
    assert!(!drain::draining(&w, d));
}

// spec: S-0608 claim 14 (ending restores: not being drained, the house due and its power back at the next update; firing answer and sale back to their ordinary rules; the drainer no longer draining; credits stay paid and the target is not given back)
#[test]
fn claim_14_what_ending_restores() {
    let (mut w, d, b) = linked("BOTH");
    let tower = put(&mut w, Kind::Building, "TOWER", BETA, 8, 8);
    let d2 = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 8, 8);
    attempt(&mut w, d2, tower);
    set_credits(&mut w, BETA, 1000);
    testkit::run(&mut w, P + 2);
    // S-0405 claim 5 and S-0405 T1, S-0408 claim 3: blackout, refused firing, no sale.
    assert_eq!(w.houses[usize::from(BETA)].econ.power.output, 0);
    assert_eq!(econ::power::firing_answer(&w, tower), Some(Answer::Illegal));
    assert!(!econ::sell::sellable(&w, b));
    let paid = credits(&w, ALPHA);
    assert!(paid > 0);
    w.houses[usize::from(BETA)].econ.power.due = false;
    drain::end(&mut w, b);
    assert!(w.houses[usize::from(BETA)].econ.power.due, "due");
    assert!(!drain::draining(&w, d));
    assert_eq!(w.obj(d).unwrap().target, None, "no target given back");
    // Another drained building (the tower has no output) does not hold the blackout.
    drain::end(&mut w, tower);
    testkit::run(&mut w, 2);
    assert_eq!(w.houses[usize::from(BETA)].econ.power.output, 50);
    assert_ne!(econ::power::firing_answer(&w, tower), Some(Answer::Illegal));
    assert!(econ::sell::sellable(&w, b));
    assert_eq!(credits(&w, ALPHA), paid, "credits stay paid");
    assert_eq!(weapon::choose_slot(&w, d, obj(b)), 1, "row 8 again");
    assert_ne!(check::check(&w, d, obj(b), 0, false), Answer::Illegal);
}

// spec: S-0405 claim 5 and S-0608 claim 14 (another drained building with output keeps the house blacked out after one link ends)
#[test]
fn another_drained_building_keeps_the_blackout() {
    let (mut w, _, b) = linked("PLANT");
    let b2 = put(&mut w, Kind::Building, "PLANT", BETA, 6, 0);
    let d2 = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 6, 0);
    attempt(&mut w, d2, b2);
    testkit::run(&mut w, 2);
    drain::end(&mut w, b);
    testkit::run(&mut w, 2);
    assert_eq!(w.houses[usize::from(BETA)].econ.power.output, 0);
    drain::end(&mut w, b2);
    testkit::run(&mut w, 2);
    assert_eq!(w.houses[usize::from(BETA)].econ.power.output, 200);
}

// ---------------------------------------------------------------------------
// Formulas, constants, import mapping, edge cases, state.

// spec: S-0608 F1 (n mod |P| = 0, exact; tick 0 pays; a negative period as its absolute value, the minimum signed integer included; P = 0 never pays, the proposed departure)
#[test]
fn f1_payout_ticks() {
    assert!(drain::payout_tick(0, 10));
    assert!(drain::payout_tick(30, 10));
    assert!(!drain::payout_tick(31, 10));
    assert!(drain::payout_tick(30, -10));
    assert!(!drain::payout_tick(5, -10));
    assert!(drain::payout_tick(0, i32::MIN));
    assert!(!drain::payout_tick(i32::MAX, i32::MIN));
    assert!(!drain::payout_tick(0, 0));
    assert!(!drain::payout_tick(100, 0));
}

// spec: S-0608 F2 and Edge cases (t = min(A, C); negative amounts or credits reverse the flow; signed 32-bit wrapping, no zero clamp on the recipient)
#[test]
fn f2_payout_amount_and_negative_data() {
    assert_eq!(drain::payout_amount(25, 1000), 25);
    assert_eq!(drain::payout_amount(25, 7), 7);
    assert_eq!(drain::payout_amount(25, 0), 0);
    assert_eq!(drain::payout_amount(-25, 1000), -25);
    assert_eq!(drain::payout_amount(-25, -40), -40);
    assert_eq!(drain::payout_amount(25, -40), -40);
    // In play: A < 0, the drained house gains and the drainer's house loses, below 0 too.
    let mut r = rules();
    r.sim.general.drain_money_amount = Some(-A);
    let (mut w, _, _) = linked_in(game_of(&r), "REF");
    set_credits(&mut w, BETA, 100);
    set_credits(&mut w, ALPHA, 0);
    payouts(&mut w, P);
    assert_eq!((credits(&w, BETA), credits(&w, ALPHA)), (100 + A, -A));
    // Wrapping at the limits.
    let (mut w, _, b) = linked("REF");
    let pay_tick = (w.tick / P + 1) * P;
    while w.tick < pay_tick {
        tick::step(&mut w);
    }
    set_credits(&mut w, BETA, 1000);
    set_credits(&mut w, ALPHA, i32::MAX);
    drain::building_turn(&mut w, b);
    assert_eq!(credits(&w, ALPHA), i32::MAX.wrapping_add(A));
}

// spec: S-0608 Edge cases and F1 (a negative payout period gives the same payout ticks as its absolute value)
#[test]
fn a_negative_period_pays_as_its_absolute_value() {
    let mut r = rules();
    r.sim.general.drain_money_frame_delay = Some(-P);
    let (mut w, _, _) = linked_in(game_of(&r), "REF");
    set_credits(&mut w, BETA, 1000);
    let p = payouts(&mut w, 3 * P);
    assert_eq!(p.len(), 3);
    assert!(p.iter().all(|x| x.0.rem_euclid(P) == 0));
}

// spec: S-0608 Import mapping and Constants (absent: period 100 ticks, amount 0 credits, no animation; DrainWeapon, Drainable and ResourceDestination absent: no)
#[test]
fn import_defaults() {
    let w = testkit::game_with(&testkit::combat_ruleset(), SIZE, 7);
    let g = &w.rules().combat.general.drain;
    assert_eq!((g.period, g.amount, g.anim.clone()), (100, 0, None));
    let w = game();
    let g = &w.rules().combat.general.drain;
    assert_eq!(
        (g.period, g.amount, g.anim.as_deref()),
        (P, A, Some("SUCK"))
    );
    // Absent flags: no draining weapon, not drainable, not a resource destination.
    let mut w = game();
    let tower = put(&mut w, Kind::Building, "TOWER", BETA, 0, 0);
    let shack = put(&mut w, Kind::Building, "SHACK", BETA, 4, 0);
    let plant = put(&mut w, Kind::Building, "PLANT", BETA, 8, 0);
    let rules = testkit::combat_ruleset();
    assert_eq!(rules.sim.buildings["TOWER"].combat.drainable, None);
    assert!(drain::drainable(&w, tower), "set here");
    assert!(!drain::drainable(&w, shack));
    assert!(drain::resource_destination(&w, shack));
    assert!(!drain::resource_destination(&w, plant));
    assert_eq!(rules.sim.weapons["Gun"].drain_weapon, None);
    let c = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, -4, 0);
    assert_eq!(weapon::choose_slot(&w, c, obj(plant)), 0);
}

// spec: S-0608 Edge cases (drain attempts against cells do nothing and keep the target, so the harmless attempt repeats at every chance to fire)
#[test]
fn attempts_at_a_cell_repeat() {
    let mut w = game();
    let d = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 0, 0);
    let cell = Target::Cell(at(0, 0));
    for _ in 0..3 {
        assert!(!attempt_slot(&mut w, d, 1, cell));
        assert_eq!(w.obj(d).unwrap().target, Some(cell));
    }
    assert!(w.drain.links.is_empty());
}

// spec: S-0608 State and ordering (the link state is saved and hashed: a round trip keeps it, the stranded first link included)
#[test]
fn save_round_trip_keeps_links() {
    let (mut w, _, _) = linked("REF");
    testkit::run(&mut w, 5);
    round_trip(&w);
    let h = save::state_hash(&w);
    w.drain.links.clear();
    assert_ne!(save::state_hash(&w), h, "the links are hashed");
}

// ---------------------------------------------------------------------------
// Revision 5: claim 3's precedence and claims 15 to 17.

/// Makes building `b` hidden from every house without a sensor: a cloaked
/// building at fade level 11 (S-0516 Terms and claim 18).
fn hide(w: &mut World, b: ObjectId) {
    let o = w.obj_mut(b).unwrap();
    o.cloak.state = glasnost_sim::cloak::CloakState::Cloaked;
    o.cloak.stage = 11;
}

// spec: S-0608 claim 3 (Precedence relevant to drain: the own-current-building refusal precedes the hidden-target impossible and the absence-of-weapon impossible, and the materialising firer's rearming precedes it; the draining-weapon refusals follow the hidden-target answer and precede the empty-spawner rearming)
#[test]
fn claim_3_precedence() {
    let (mut w, d, b) = linked("PLANT");
    let other = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 1, 1);
    // An empty slot: illegal at its own building, impossible elsewhere.
    assert_eq!(check::check(&w, d, obj(b), 2, false), Answer::Illegal);
    assert_eq!(
        check::check(&w, other, obj(b), 2, false),
        Answer::Impossible
    );
    // A hidden building: its own drainer still gets illegal; another
    // drainer's draining weapon gets the hidden target's impossible first.
    hide(&mut w, b);
    assert!(glasnost_sim::cloak::hidden_from(
        &w,
        b,
        Some(HouseId(ALPHA))
    ));
    assert_eq!(check::check(&w, d, obj(b), 0, false), Answer::Illegal);
    assert_eq!(check::check(&w, d, obj(b), 1, false), Answer::Illegal);
    assert_eq!(
        check::check(&w, other, obj(b), 1, false),
        Answer::Impossible
    );
    // A materialising firer at its own building: rearming.
    let t = w.tick;
    w.obj_mut(d).unwrap().teleport = Some(glasnost_sim::teleport::TeleportState {
        phasing: Some((t, 50)),
        ..Default::default()
    });
    assert_eq!(check::check(&w, d, obj(b), 0, false), Answer::Rearming);
    // A draining spawner weapon with no live spawn: refused as draining
    // (part c, a type that is not drainable) before the empty-spawner rearming.
    let s = put(&mut w, Kind::Vehicle, "SPAWNY", ALPHA, 6, 6);
    let shack = put(&mut w, Kind::Building, "SHACK", BETA, 8, 8);
    assert_eq!(check::check(&w, s, obj(shack), 1, false), Answer::Illegal);
    let free = put(&mut w, Kind::Building, "PLANT", BETA, -6, 6);
    assert_eq!(
        check::check(&w, s, obj(free), 1, false),
        Answer::Rearming,
        "the empty spawner's own answer when no drain refusal applies"
    );
}

// spec: S-0608 claim 15 (Q-1440: the erasure exit skips the alliance test, so an erased drainer keeps its link through an alliance change; its position check still runs before that exit)
#[test]
fn claim_15_an_erased_drainer() {
    let lock = |w: &mut World, d: ObjectId, leader: ObjectId| {
        w.temporal.locks.insert(
            d,
            glasnost_sim::temporal::Lock {
                leader,
                joiners: Vec::new(),
                // The remaining amount (S-0514 F3): far from completion.
                amount: 1_000_000,
                was_off: false,
                suspended: false,
            },
        );
    };
    // The alliance test: skipped while erased, run otherwise.
    for erased in [true, false] {
        let (mut w, d, b) = linked("REF");
        let leader = put(&mut w, Kind::Vehicle, "BOX", BETA, 8, 8);
        while w.tick.rem_euclid(drain::CHECK_PERIOD) == drain::CHECK_PHASE {
            tick::step(&mut w);
        }
        if erased {
            lock(&mut w, d, leader);
        }
        w.houses[usize::from(BETA)].allies.push(HouseId(ALPHA));
        w.houses[usize::from(BETA)].allies.sort();
        tick::step(&mut w);
        assert_eq!(glasnost_sim::temporal::erased(&w, d), erased);
        assert_eq!(
            drain::partner(&w, d),
            erased.then_some(b),
            "erased {erased}"
        );
    }
    // The position check: run though erased.
    let (mut w, d, _) = linked("REF");
    let leader = put(&mut w, Kind::Vehicle, "BOX", BETA, 8, 8);
    shift(&mut w, d, at(6, 6), 0);
    while w.tick.rem_euclid(drain::CHECK_PERIOD) != drain::CHECK_PHASE {
        tick::step(&mut w);
    }
    lock(&mut w, d, leader);
    tick::step(&mut w);
    assert!(!drain::draining(&w, d), "ended by the position check");
}

// spec: S-0608 claim 15 (Q-1440: a building's payout follows its due periodic income in the same turn, so income of that turn can be taken by it) and S-0515 claim 5
#[test]
fn claim_15_the_payout_takes_same_turn_income() {
    let (mut w, _, b) = linked("CASHREF");
    let pay = (w.tick / P + 2) * P;
    // The income cycle due in the payout tick (S-0515 F1).
    let short = i32::from(w.rules().quirks.cash_cycle_one_short);
    w.obj_mut(b).unwrap().econ.building.income_restart = Some(pay - P + short);
    while w.tick < pay {
        tick::step(&mut w);
    }
    set_credits(&mut w, BETA, 0);
    set_credits(&mut w, ALPHA, 0);
    let p = payouts(&mut w, 1);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].1, A, "the income of that turn is taken");
    assert_eq!(credits(&w, BETA), 100 - A);
    assert_eq!(credits(&w, ALPHA), A);
}

// spec: S-0608 claim 15 (Q-1440: house power recalculation follows the turn pass, so a link ended by a position check is already gone for that tick's power update of the drained building's owner) and S-0405 claim 7
#[test]
fn claim_15_power_sees_the_position_check() {
    let (mut w, d, _) = linked("BOTH");
    testkit::run(&mut w, 2);
    assert_eq!(w.houses[usize::from(BETA)].econ.power.output, 0);
    shift(&mut w, d, at(6, 6), 0);
    while w.tick.rem_euclid(drain::CHECK_PERIOD) != drain::CHECK_PHASE {
        tick::step(&mut w);
        assert!(drain::draining(&w, d));
    }
    tick::step(&mut w);
    assert!(!drain::draining(&w, d));
    assert_eq!(w.houses[usize::from(BETA)].econ.power.output, 50);
}

// spec: S-0608 claim 16 (Q-1442: the attempt records no new most recent shot, makes no firing reveal to the target's house and does not uncloak the firer, whether or not a link forms) and S-0409 Import mapping (RevealOnFire absent: yes, so an ordinary shot would reveal)
#[test]
fn claim_16_no_shot_record_reveal_or_uncloak() {
    for over in [true, false] {
        let mut w = game();
        let b = put(&mut w, Kind::Building, "REF", BETA, 0, 0);
        let d = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 0, 0);
        if !over {
            shift(&mut w, d, at(-2, 0), 0);
        }
        testkit::run(&mut w, 2);
        w.obj_mut(d).unwrap().cloak.state = glasnost_sim::cloak::CloakState::Cloaked;
        let (fired, shroud) = (w.obj(d).unwrap().last_fired, w.econ.shroud.clone());
        attempt(&mut w, d, b);
        assert_eq!(drain::partner(&w, d), over.then_some(b), "over {over}");
        let o = w.obj(d).unwrap();
        assert_eq!(o.last_fired, fired, "no shot recorded (over {over})");
        assert_eq!(
            o.cloak.state,
            glasnost_sim::cloak::CloakState::Cloaked,
            "still cloaked (over {over})"
        );
        assert!(w.econ.shroud == shroud, "no reveal (over {over})");
    }
}

// spec: S-0608 claim 16 (Q-1442: a suicide weapon that is also draining makes the suicide hit on its firer instead of the attempt: no link, no drain target clearing)
#[test]
fn claim_16_suicide_replaces_the_attempt() {
    let mut w = game();
    let b = put(&mut w, Kind::Building, "REF", BETA, 0, 0);
    let d = put(&mut w, Kind::Vehicle, "BOOMER", ALPHA, 0, 0);
    attempt(&mut w, d, b);
    assert!(!drain::being_drained(&w, b));
    assert!(
        w.obj(d).is_none_or(glasnost_sim::combat::damage::dead),
        "the suicide hit"
    );
}

/// A LEECH of Alpha draining a PLANT of Beta at (0, 0), then moved to
/// (6, 6) with an Alpha VAN there (an infantry drainer keeps its link off
/// the building, S-0608 claim 12's switch, on by default).
fn leech_by_van(strands: bool) -> (World, ObjectId, ObjectId, ObjectId, Option<ObjectId>) {
    let mut r = rules();
    r.globals.quirks.drain_relink_strands_previous = true;
    let mut w = game_of(&r);
    let b = put(&mut w, Kind::Building, "PLANT", BETA, 0, 0);
    let l = put(&mut w, Kind::Infantry, "LEECH", ALPHA, 0, 0);
    attempt_slot(&mut w, l, 0, Target::Object(b));
    let first = if strands {
        let b2 = put(&mut w, Kind::Building, "PLANT", BETA, 4, 0);
        shift(&mut w, l, at(4, 0), 0);
        attempt_slot(&mut w, l, 0, Target::Object(b2));
        assert!(drain::being_drained(&w, b) && drain::being_drained(&w, b2));
        Some(b)
    } else {
        None
    };
    let b = drain::partner(&w, l).expect("linked");
    let van = put(&mut w, Kind::Vehicle, "VAN", ALPHA, 6, 6);
    shift(&mut w, l, at(6, 6), 0);
    w.obj_mut(l).unwrap().dest = Some(Target::Object(van));
    (w, l, b, van, first)
}

// spec: S-0608 claim 17 (Q-1443: boarding a transport ends the drainer's links at once, a stranded earlier link included, with claim 14's effects; returning does not restore them) and S-0507 claim 6
#[test]
fn claim_17_boarding_ends_the_links() {
    for strands in [false, true] {
        let (mut w, l, b, van, first) = leech_by_van(strands);
        w.effects.clear();
        assert!(glasnost_sim::transport::arrival(&mut w, l), "boards");
        assert!(!drain::draining(&w, l), "strands {strands}");
        assert!(!drain::being_drained(&w, b));
        if let Some(f) = first {
            assert!(!drain::being_drained(&w, f), "the stranded link too");
        }
        assert!(w.drain.links.is_empty() && w.drain.partner.is_empty());
        assert!(events(&w).contains(&DrainEvent::Ended));
        // Out again: no link comes back.
        w.obj_mut(van)
            .unwrap()
            .transport
            .passengers
            .retain(|&x| x != l);
        w.obj_mut(l).unwrap().transport.riding = None;
        let p = Pos::centre(at(6, 6), 0);
        let ground = w.surface_at(p.x, p.y);
        objects::place(&mut w, l, Pos::new(p.x, p.y, ground), 0);
        testkit::run(&mut w, 3);
        assert!(!drain::draining(&w, l));
        assert!(!drain::being_drained(&w, b));
        round_trip(&w);
    }
}

// spec: S-0608 claim 17 (Q-1443: a failed boarding is not storage: the link stays) and S-0507 claim 6 (a full transport turns it away)
#[test]
fn claim_17_failed_boarding_keeps_the_link() {
    let (mut w, l, b, van, _) = leech_by_van(false);
    let rider = put(&mut w, Kind::Infantry, "WALKER", ALPHA, 6, 6);
    glasnost_sim::transport::board(&mut w, rider, van);
    assert!(!glasnost_sim::transport::arrival(&mut w, l), "turned away");
    assert_eq!(drain::partner(&w, l), Some(b));
}

// ---------------------------------------------------------------------------
// Claim 19: protection does not stop a drain (Q-1507).

fn protect_for(w: &mut World, id: ObjectId, kind: glasnost_sim::superweapons::ProtectKind, d: i32) {
    glasnost_sim::superweapons::protect::start(w, id, kind, d);
    assert!(glasnost_sim::superweapons::protected(w, id));
}

// spec: S-0608 claim 19 (forming: an attempt against a building under either protection kind forms the link as against an unprotected one; the firing check gives a player house no extra answer; payouts follow claim 7) and S-0513 claim 27 (revision 19: protection does not refuse a draining weapon's attempt)
#[test]
fn claim_19_a_protected_building_is_drained() {
    use glasnost_sim::superweapons::ProtectKind;
    for kind in [ProtectKind::Curtain, ProtectKind::ForceShield] {
        let mut w = game();
        let b = put(&mut w, Kind::Building, "BOTH", BETA, 0, 0);
        let d = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 0, 0);
        protect_for(&mut w, b, kind, 1000);
        assert_ne!(check::check(&w, d, obj(b), 1, false), Answer::Illegal);
        attempt(&mut w, d, b);
        assert_eq!(drain::partner(&w, d), Some(b), "{kind:?}: the link forms");
        set_credits(&mut w, BETA, 1000);
        set_credits(&mut w, ALPHA, 0);
        assert_eq!(payouts(&mut w, 2 * P).len(), 2, "{kind:?}: due payouts");
        assert_eq!(credits(&w, ALPHA), 2 * A);
        assert_eq!(
            w.houses[usize::from(BETA)].econ.power.output,
            0,
            "{kind:?}: the drain blackout"
        );
        assert!(glasnost_sim::superweapons::protected(&w, b));
        round_trip(&w);
    }
}

// spec: S-0608 claim 19 (protection starting on the drained building or on the drainer while linked, and its end, change nothing: the link and the payouts go on until claims 11 to 13 or 17 end it)
#[test]
fn claim_19_protection_starting_and_ending_changes_nothing() {
    use glasnost_sim::superweapons::ProtectKind;
    for kind in [ProtectKind::Curtain, ProtectKind::ForceShield] {
        for on_drainer in [false, true] {
            let (mut w, d, b) = linked("REF");
            set_credits(&mut w, BETA, 1000);
            set_credits(&mut w, ALPHA, 0);
            protect_for(&mut w, if on_drainer { d } else { b }, kind, 3 * P);
            let p = payouts(&mut w, 2 * P);
            assert_eq!(p.len(), 2, "{kind:?} {on_drainer}: payouts while protected");
            assert_eq!(drain::partner(&w, d), Some(b));
            round_trip(&w);
            let done = until(&mut w, 10 * P, |w| {
                !glasnost_sim::superweapons::protected(w, d)
                    && !glasnost_sim::superweapons::protected(w, b)
            });
            assert!(done, "the protection ends");
            assert_eq!(
                drain::partner(&w, d),
                Some(b),
                "{kind:?} {on_drainer}: no detach at the end"
            );
            assert_eq!(payouts(&mut w, 2 * P).len(), 2, "payouts go on");
        }
    }
}

// spec: S-0608 claim 19 (who may fire: a firer of a non-player house gets S-0513 claim 28's illegal answer against a protected target with its draining weapon, and makes no attempt; a draining weapon given a parasite warhead is refused as any weapon by S-0513 claim 27)
#[test]
fn claim_19_non_player_houses_and_parasite_drainers_are_refused() {
    use glasnost_sim::superweapons::ProtectKind;
    let mut r = rules();
    let mut sap = r.sim.warheads["SA"].clone();
    sap.parasite = Some(true);
    r.sim.warheads.insert("SAP".into(), sap);
    let mut sucker = r.sim.weapons["SUCKER"].clone();
    sucker.warhead = Some("SAP".into());
    r.sim.weapons.insert("SUCKPAR".into(), sucker);
    vehicle(&mut r, "PDISC", |t| {
        t.primary = Some("Gun".into());
        t.secondary = Some("SUCKPAR".into());
    });
    let mut w = testkit::game_with_non_players(&r, SIZE, 7);
    while orders::blocked(&w) {
        tick::step(&mut w);
    }
    let b = put(&mut w, Kind::Building, "PLANT", BETA, 0, 0);
    let mine = put(&mut w, Kind::Vehicle, "DISC", ALPHA, 0, 0);
    // House 2 is the neutral house (testkit::game_with_non_players).
    let neutral = put(&mut w, Kind::Vehicle, "DISC", 2, 0, 0);
    let parasite = put(&mut w, Kind::Vehicle, "PDISC", ALPHA, 0, 0);
    assert_ne!(
        check::check(&w, neutral, obj(b), 1, false),
        Answer::Illegal,
        "unprotected: no refusal"
    );
    assert_ne!(
        check::check(&w, parasite, obj(b), 1, false),
        Answer::Illegal,
        "unprotected: no refusal"
    );
    protect_for(&mut w, b, ProtectKind::Curtain, 1000);
    assert_ne!(check::check(&w, mine, obj(b), 1, false), Answer::Illegal);
    assert_eq!(check::check(&w, neutral, obj(b), 1, false), Answer::Illegal);
    assert_eq!(
        check::check(&w, parasite, obj(b), 1, false),
        Answer::Illegal
    );
}

// spec: S-0109 State and ordering (an animation that leaves play in its own detonation: nothing more happens to it in that update, no middle event, pass end, follow-up switch or end; our engine's rule) and S-0608 claims 13 and 14 (the drain animation's own damage kills the drainer: the link ends and the drain animation is removed in its own turn)
#[test]
fn the_drain_animation_leaves_play_in_its_own_detonation() {
    let run = |next: Option<&str>| {
        let mut r = rules();
        register(&mut r, "warheads", "KILL");
        r.sim.warheads.insert(
            "KILL".into(),
            WarheadType {
                verses: Some(vec![dec("1"); 11]),
                ..Default::default()
            },
        );
        r.sim.general.flame_damage2 = Some("KILL".into());
        let suck = r.sim.animations.get_mut("SUCK").unwrap();
        suck.damage = Some(dec("5000"));
        suck.end = Some(1);
        suck.next = next.map(Into::into);
        register(&mut r, "animations", "AFTER");
        r.sim.animations.insert(
            "AFTER".into(),
            AnimationType {
                frame_count: Some(8),
                random_rate: Some([450, 150]),
                ..Default::default()
            },
        );
        let (mut w, d, b) = linked_in(game_of(&r), "REF");
        let ids = glasnost_sim::anims::with_role(&w, glasnost_sim::anims::Role::Drain(d));
        assert_eq!(ids.len(), 1, "the drain animation");
        let id = ids[0];
        assert!(
            until(&mut w, 40, |w| !w.anims.anims.contains_key(&id)),
            "the drain animation is removed"
        );
        assert!(!drain::draining(&w, d) && !drain::being_drained(&w, b));
        let t = w.tick;
        (w, t)
    };
    let (w, t) = run(Some("AFTER"));
    let (reference, tr) = run(None);
    assert_eq!(t, tr, "removed in the same tick");
    assert_eq!(
        w.rng, reference.rng,
        "no follow-up switch and its delay draw"
    );
}
