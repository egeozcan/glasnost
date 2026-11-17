//! S-0607 revision 52 (Q-3182, Q-3183, #285). Going into a vehicle bunker is
//! not leaving play: a lifter that grips keeps its grip, a dropper keeps its
//! place, a seizing shot in flight is not voided and a magnetic wave is left
//! to its other ending conditions. The riders of an open-topped transport
//! that itself goes inside do not leave play either: they take turns, lose
//! their targets, keep their grips and stay the droppers of the units they
//! dropped. Synthetic data only: every name and number is invented.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::combat::fire;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::bunker::{self, Stage};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{lift, objects, transport};
use lift_common::*;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The lift ruleset with:
/// - PIT: a 2 × 2 vehicle bunker with no impassable rows;
/// - OPEN: an open-topped vehicle transport with a short-range weapon, so
///   that it is bunker-eligible (S-0606 claim 1); BIG: a closed vehicle
///   transport that OPEN fits in;
/// - MAGTROOP: an infantry type with the magnetic beam (a walking lifter);
/// - SLOW: a victim whose jumpjet speed is so low that it does not reach its
///   lifter while a test runs (claim 8's arrival ends a grip);
/// - FLOATY: a victim that climbs fast to a great height and flies slowly,
///   so that a fall lasts long;
/// - SHIELD: an immune, bunker-eligible vehicle (its size is not under the
///   beam's damage, claim 3), a wave target that is never seized;
/// - SLOWLIFT: LIFTER with SlowBeam, the magnetic beam with a slow
///   projectile, whose shot is still in flight when the bunker's last stage
///   ends.
fn rules_world() -> World {
    world(|r| {
        let mut pit = BuildingType {
            strength: Some(1000),
            foundation: Some([2, 2]),
            bunker: Some(true),
            number_impassable_rows: Some(0),
            ..Default::default()
        };
        pit.econ.buildup = Some(true);
        pit.econ.buildup_frames = Some(4);
        register(r, "buildings", "PIT");
        r.sim.buildings.insert("PIT".into(), pit);
        vehicle(r, "OPEN", |t| {
            t.strength = Some(1000);
            t.passengers = Some(4);
            t.size_limit = Some(d("2"));
            t.open_topped = Some(true);
            t.primary = Some("Doom".into());
        });
        vehicle(r, "BIG", |t| {
            t.strength = Some(1000);
            t.passengers = Some(4);
            t.size_limit = Some(d("3"));
        });
        infantry(r, "MAGTROOP", |t| t.primary = Some("MagBeam".into()));
        vehicle(r, "SLOW", |t| t.jumpjet_speed = Some(2));
        vehicle(r, "FLOATY", |t| {
            t.jumpjet_speed = Some(2);
            t.jumpjet_height = Some(6000);
            t.jumpjet_climb = Some(d("50"));
        });
        vehicle(r, "SHIELD", |t| {
            t.size = Some(d("10"));
            t.primary = Some("Doom".into());
        });
        let mut slow = r.sim.weapons["MagBeam"].clone();
        // The testkit's Bullet is invisible and lands at once; its Rocket
        // homes and flies at the weapon's speed.
        slow.projectile = Some("Rocket".into());
        slow.speed = Some(5);
        r.sim.weapons.insert("SlowBeam".into(), slow);
        vehicle(r, "SLOWLIFT", |t| {
            t.strength = Some(1000);
            t.primary = Some("SlowBeam".into());
        });
    })
}

/// A bunker of `house` with its top-left cell at the middle cell, settled
/// for a few ticks, past the order block at the game's start (S-0213 claim
/// 1).
fn pit(w: &mut World, house: u8) -> ObjectId {
    let b = spawn(w, Kind::Building, "PIT", house, at(0, 0));
    run(w, 10);
    assert!(
        !glasnost_sim::orders::blocked(w),
        "set-up: orders are taken"
    );
    b
}

/// The enter order on bunker `b` (S-0214 table M: no target, the bunker as
/// destination), applied in the next tick.
fn enter(w: &mut World, u: ObjectId, b: ObjectId) {
    let house = w.obj(u).unwrap().owner.unwrap();
    assert!(orders::issue(
        w,
        house,
        Order::Behaviour {
            actor: u,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        },
    ));
    run(w, 1);
}

fn bunkered(w: &World, u: ObjectId, b: ObjectId) -> bool {
    bunker::bunkered_in(w, u) == Some(b)
}

fn stage(w: &World, b: ObjectId) -> Stage {
    w.obj(b).unwrap().structure.bunker_stage
}

/// Gives `firer` `target` as its target without changing its behaviour and
/// fires its beam at it; the shot is in flight.
fn fire_at(w: &mut World, firer: ObjectId, target: ObjectId) {
    w.obj_mut(firer).unwrap().target = Some(Target::Object(target));
    assert!(fire::shot(w, firer, 0, Target::Object(target)));
    assert!(!w.projectiles.is_empty(), "the shot flies");
}

/// Fires as [`fire_at`] and runs until the shot has detonated.
fn grab(w: &mut World, firer: ObjectId, target: ObjectId) {
    fire_at(w, firer, target);
    assert!(
        run_until(w, 60, |w| w.projectiles.is_empty()),
        "the beam did not detonate"
    );
}

fn wave_live(w: &World, wave: ObjectId) -> bool {
    w.waves.get(&wave).is_some_and(|x| !x.ended)
}

// ---------------------------------------------------------------------------
// Q-3182: a vehicle bunker.

// spec: S-0607 claim 12 (revision 52, Lifters and vehicle bunkers: a lifter that grips while it goes into a bunker, its grip having begun after it took the bunker as its destination, keeps its grip at the entry and while it is bunkered; the entry does not change its target)
#[test]
fn a_lifter_that_grips_while_it_goes_into_a_bunker_keeps_its_grip() {
    let mut w = rules_world();
    let b = pit(&mut w, 0);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(4, 1));
    let v = spawn(&mut w, Kind::Vehicle, "SLOW", 1, at(4, 9));
    enter(&mut w, l, b);
    assert_eq!(
        w.obj(l).unwrap().dest,
        Some(Target::Object(b)),
        "the bunker is its destination"
    );
    grab(&mut w, l, v);
    assert!(
        lift::grips(&w, l, v),
        "the grip begins after the bunker became its destination"
    );
    for _ in 0..1500 {
        if bunkered(&w, l, b) {
            break;
        }
        run(&mut w, 1);
        assert!(
            lift::grips(&w, l, v),
            "the grip lasts on the way in, tick {} (stage {:?})",
            w.tick,
            stage(&w, b)
        );
    }
    assert!(bunkered(&w, l, b), "the lifter is bunkered");
    for _ in 0..30 {
        run(&mut w, 1);
        assert!(
            lift::grips(&w, l, v),
            "the grip lasts while it is bunkered, tick {}",
            w.tick
        );
    }
    assert_eq!(
        w.obj(l).unwrap().target,
        Some(Target::Object(v)),
        "the entry does not change its target"
    );
    assert!(
        w.obj(l).unwrap().on_map,
        "a bunkered vehicle stays on the map"
    );
}

// spec: S-0607 claim 14 (revision 52, Bunkers and riders: going into a vehicle bunker is not leaving play for the dropper rule; a dropper that goes in keeps its place as dropper) and claim 12 (the enter order's destination, accepted while it grips, ends the grip by the second row)
#[test]
fn a_dropper_that_goes_into_a_bunker_stays_the_dropper() {
    let mut w = rules_world();
    let b = pit(&mut w, 0);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(2, 0));
    let v = spawn(&mut w, Kind::Vehicle, "FLOATY", 1, at(2, 8));
    beam(&mut w, l, v);
    assert!(lift::grips(&w, l, v));
    run(&mut w, 300);
    assert!(lift::grips(&w, l, v), "set-up: still gripped");
    let high = height(&w, v);
    assert!(high > 300, "the victim climbs: height {high}");
    enter(&mut w, l, b);
    // The order is applied after the turn pass (S-0203 claim 7); the lifter
    // accepts the destination at its next turn (`lift::lifter_destination`).
    assert!(
        run_until(&mut w, 3, |w| !lift::grips(w, l, v)),
        "accepting the bunker as destination ends the grip"
    );
    assert!(lift::dropped(&w, v), "dropped, being high");
    assert_eq!(w.obj(v).unwrap().lift.dropper, Some(l));
    for _ in 0..1500 {
        if bunkered(&w, l, b) {
            break;
        }
        run(&mut w, 1);
    }
    assert!(bunkered(&w, l, b), "the dropper is bunkered");
    assert!(
        lift::dropped(&w, v),
        "the dropped unit is still falling when its dropper is bunkered (dropped from {high}, now {}, tick {})",
        height(&w, v),
        w.tick
    );
    let o = w.obj(v).unwrap();
    assert_eq!(o.lift.dropper, Some(l), "the dropper keeps its place");
    assert_eq!(o.lift.dropper_house, Some(HouseId(0)));
}

// spec: S-0607 claim 2 (revision 52: a firer that goes into a vehicle bunker while its seizing projectile flies has not left play; the hit is tested as any other)
#[test]
fn a_seizing_shot_in_flight_as_its_firer_is_bunkered_still_seizes() {
    let mut w = rules_world();
    let b = pit(&mut w, 0);
    let l = spawn(&mut w, Kind::Vehicle, "SLOWLIFT", 0, at(4, 1));
    let v = spawn(&mut w, Kind::Vehicle, "SLOW", 1, at(4, 10));
    enter(&mut w, l, b);
    assert!(
        run_until(&mut w, 1500, |w| stage(w, b) == Stage::RaisingWalls),
        "the entry reaches its last stage before the link"
    );
    fire_at(&mut w, l, v);
    let fired = w.tick;
    let mut landed = None;
    while !bunkered(&w, l, b) && w.tick < fired + 30 {
        run(&mut w, 1);
        if landed.is_none() && w.projectiles.is_empty() {
            landed = Some((w.tick, lift::seized(&w, v), stage(&w, b)));
        }
    }
    assert!(bunkered(&w, l, b), "bunkered while the shot flies");
    assert!(
        landed.is_none(),
        "the shot is still in flight at the link, tick {} (fired {fired}; gone at, seized, stage: {landed:?})",
        w.tick
    );
    assert!(
        run_until(&mut w, 1000, |w| w.projectiles.is_empty()),
        "the beam detonates"
    );
    assert!(lift::seized(&w, v), "the hit is not voided");
    assert_eq!(lift::lifter_of(&w, v), Some(l));
}

// spec: S-0607 claim 21 (revision 52, Vehicle bunkers and riders: a target that goes into a vehicle bunker leaves the wave to the other ending conditions)
#[test]
fn a_wave_lasts_when_its_target_goes_into_a_bunker() {
    let mut w = rules_world();
    let b = pit(&mut w, 1);
    let t = spawn(&mut w, Kind::Vehicle, "SHIELD", 1, at(4, 1));
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(4, 9));
    attack(&mut w, l, t);
    assert!(fire::shot(&mut w, l, 0, Target::Object(t)));
    let wave = lift::wave_of(&w, l).expect("the shot makes a wave");
    enter(&mut w, t, b);
    attack(&mut w, l, t);
    for _ in 0..1500 {
        run(&mut w, 1);
        attack(&mut w, l, t);
        assert!(
            wave_live(&w, wave),
            "the wave lasts on the target's way in, tick {} (bunkered: {})",
            w.tick,
            bunkered(&w, t, b)
        );
        if bunkered(&w, t, b) {
            break;
        }
    }
    assert!(bunkered(&w, t, b), "the target is bunkered");
    for _ in 0..10 {
        run(&mut w, 1);
        attack(&mut w, l, t);
        assert!(
            wave_live(&w, wave),
            "the wave lasts while its target is bunkered, tick {}",
            w.tick
        );
    }
}

// spec: S-0607 claim 21 (revision 52, Vehicle bunkers and riders: a wave a bunkered firer creates after it takes a target again is left by the entry to the other ending conditions)
#[test]
fn a_bunkered_firers_wave_lasts() {
    let mut w = rules_world();
    let b = pit(&mut w, 0);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(4, 1));
    enter(&mut w, l, b);
    assert!(
        run_until(&mut w, 1500, |w| bunkered(w, l, b)),
        "the lifter is bunkered"
    );
    let t = spawn(&mut w, Kind::Vehicle, "SHIELD", 1, at(4, 9));
    attack(&mut w, l, t);
    assert!(fire::shot(&mut w, l, 0, Target::Object(t)));
    let wave = lift::wave_of(&w, l).expect("the bunkered firer's shot makes a wave");
    assert!(wave_live(&w, wave), "not ended at its creation update");
    for _ in 0..10 {
        run(&mut w, 1);
        attack(&mut w, l, t);
        assert!(
            wave_live(&w, wave),
            "the wave lasts while its firer is bunkered, tick {}",
            w.tick
        );
    }
}

// spec: S-0607 claims 12 and 21 (revision 52: being let out of a bunker is not entering play again; the vehicle keeps its place in the update order, S-0203 claim 3)
#[test]
fn a_vehicle_let_out_of_its_bunker_keeps_its_place_in_the_update_order() {
    let mut w = rules_world();
    let b = pit(&mut w, 0);
    let l = spawn(&mut w, Kind::Vehicle, "LIFTER", 0, at(4, 1));
    enter(&mut w, l, b);
    assert!(
        run_until(&mut w, 1500, |w| bunkered(w, l, b)),
        "the lifter is bunkered"
    );
    let place = w.update.order.iter().position(|&x| x == l);
    assert!(place.is_some(), "a bunkered vehicle is in the update order");
    // spec: S-0606 claim 13 (a release when the bunker is destroyed, sold or erased)
    bunker::let_out(&mut w, b);
    assert!(!bunkered(&w, l, b), "let out");
    assert_eq!(
        w.update.order.iter().position(|&x| x == l),
        place,
        "the same place in the update order"
    );
}

// spec: S-0607 claims 12 and 21 (revision 52: an open-topped transport that goes into a bunker does not leave play or the map, so its riders keep their targets, their grips and their waves)
#[test]
fn an_open_topped_transport_in_a_bunker_keeps_its_riders_targets_grips_and_waves() {
    let mut w = rules_world();
    let b = pit(&mut w, 0);
    let t = spawn(&mut w, Kind::Vehicle, "OPEN", 0, at(4, 1));
    let m = spawn(&mut w, Kind::Infantry, "MAGTROOP", 0, at(4, 2));
    let v = spawn(&mut w, Kind::Vehicle, "SLOW", 1, at(4, 9));
    transport::board(&mut w, m, t);
    beam(&mut w, m, v);
    assert!(lift::grips(&w, m, v));
    let wave = lift::wave_of(&w, m).expect("the rider's shot makes a wave");
    enter(&mut w, t, b);
    attack(&mut w, m, v);
    for _ in 0..1500 {
        if bunkered(&w, t, b) {
            break;
        }
        run(&mut w, 1);
        attack(&mut w, m, v);
        assert!(lift::grips(&w, m, v), "the grip lasts, tick {}", w.tick);
    }
    assert!(bunkered(&w, t, b), "the transport is bunkered");
    for _ in 0..10 {
        run(&mut w, 1);
        assert_eq!(
            w.obj(m).unwrap().target,
            Some(Target::Object(v)),
            "the rider keeps its target"
        );
        assert!(lift::grips(&w, m, v), "the grip lasts, tick {}", w.tick);
        assert!(wave_live(&w, wave), "the wave lasts, tick {}", w.tick);
    }
}

// ---------------------------------------------------------------------------
// Q-3183: the riders of an open-topped transport that goes inside.

/// A walking lifter of house 0 aboard OPEN, gripping SLOW of house 1.
fn riding_grip() -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = rules_world();
    let t = spawn(&mut w, Kind::Vehicle, "OPEN", 0, at(0, 0));
    let m = spawn(&mut w, Kind::Infantry, "MAGTROOP", 0, at(0, 1));
    let v = spawn(&mut w, Kind::Vehicle, "SLOW", 1, at(6, 0));
    transport::board(&mut w, m, t);
    beam(&mut w, m, v);
    assert!(lift::grips(&w, m, v));
    (w, t, m, v)
}

/// A closed transport BIG of house 0 beside the middle cell.
fn big(w: &mut World) -> ObjectId {
    spawn(w, Kind::Vehicle, "BIG", 0, at(-2, 0))
}

// spec: S-0607 claim 12 (revision 52, A riding lifter whose transport goes inside: the riders do not leave play; the loss of the target, S-0507 claim 15, is a change of target, not an event of the table, so the grip lasts and the gripped unit keeps flying to the movement point fixed at the seize)
#[test]
fn a_riding_lifters_grip_lasts_when_its_transport_boards_a_closed_transport() {
    let (mut w, t, m, v) = riding_grip();
    let k = big(&mut w);
    transport::board(&mut w, t, k);
    assert_eq!(transport::riding(&w, t), Some(k), "the transport is inside");
    assert_eq!(w.obj(m).unwrap().target, None, "the rider loses its target");
    let start = w.obj(v).unwrap().pos;
    for _ in 0..20 {
        run(&mut w, 1);
        assert!(lift::grips(&w, m, v), "the grip lasts, tick {}", w.tick);
        assert_eq!(lift::lifter_of(&w, v), Some(m));
    }
    assert_ne!(
        w.obj(v).unwrap().pos,
        start,
        "the gripped unit keeps flying"
    );
}

// spec: S-0607 claim 12 (revision 52: when an open-topped transport goes inside a transport or a building, its riders stay in the update order and take turns while it is inside; S-0203 claim 3 and S-0507 claim 14, Changes for other specs)
#[test]
fn the_riders_of_a_transport_that_went_inside_take_turns() {
    let (mut w, t, m, _) = riding_grip();
    let k = big(&mut w);
    transport::board(&mut w, t, k);
    assert!(w.update.contains(m), "the rider stays in the update order");
    for _ in 0..5 {
        run(&mut w, 1);
        assert_eq!(
            w.obj(m).unwrap().last_turn,
            w.tick - 1,
            "the rider took its turn in the tick just run"
        );
    }
}

// spec: S-0607 claim 21 (revision 52, Vehicle bunkers and riders: the going inside of an open-topped transport that the firer rides is not leaving play for the wave; the firer loses its target then, S-0507 claim 15, so the wave ends at its next update by the target condition)
#[test]
fn a_riding_firers_wave_ends_by_the_target_condition_when_its_transport_goes_inside() {
    let (mut w, t, m, _) = riding_grip();
    let wave = lift::wave_of(&w, m).expect("the rider's shot makes a wave");
    let k = big(&mut w);
    transport::board(&mut w, t, k);
    assert!(
        wave_live(&w, wave),
        "not ended by the boarding itself: the riders do not leave play"
    );
    run(&mut w, 1);
    assert!(
        !wave_live(&w, wave),
        "ended at its next update: the firer has no target"
    );
}

// spec: S-0607 claim 14 (revision 52, Bunkers and riders: the going inside of an open-topped transport that the dropper rides is not leaving play for the dropper rule; the dropper keeps its place)
#[test]
fn a_riding_dropper_stays_the_dropper_when_its_transport_goes_inside() {
    let (mut w, t, m, v) = riding_grip();
    assert!(
        run_until(&mut w, 400, |w| height(w, v) > 300),
        "the victim rises"
    );
    // spec: S-0607 claim 13 (a grip that ends at height 1 lepton or more drops the unit, the lifter its dropper)
    lift::end_grip(&mut w, m, false);
    assert!(lift::dropped(&w, v));
    assert_eq!(w.obj(v).unwrap().lift.dropper, Some(m));
    let k = big(&mut w);
    transport::board(&mut w, t, k);
    let o = w.obj(v).unwrap();
    assert_eq!(o.lift.dropper, Some(m), "the rider stays the dropper");
    assert_eq!(o.lift.dropper_house, Some(HouseId(0)));
}

// spec: S-0607 claim 2 (revision 52: a firer that rides an open-topped transport that goes inside a transport while its seizing projectile flies has not left play; the hit is tested as any other)
#[test]
fn a_riders_seizing_shot_in_flight_as_its_transport_goes_inside_still_seizes() {
    let mut w = rules_world();
    let t = spawn(&mut w, Kind::Vehicle, "OPEN", 0, at(0, 0));
    let m = spawn(&mut w, Kind::Infantry, "MAGTROOP", 0, at(0, 1));
    let v = spawn(&mut w, Kind::Vehicle, "SLOW", 1, at(6, 0));
    let k = big(&mut w);
    transport::board(&mut w, m, t);
    fire_at(&mut w, m, v);
    transport::board(&mut w, t, k);
    assert!(
        run_until(&mut w, 60, |w| w.projectiles.is_empty()),
        "the beam detonates"
    );
    assert!(lift::seized(&w, v), "the hit is not voided");
    assert_eq!(lift::lifter_of(&w, v), Some(m));
}

// spec: S-0607 claim 12 (revision 52, A riding lifter whose transport goes inside: going inside a building as well as a transport; the riders' grips last)
#[test]
fn a_riding_lifters_grip_lasts_when_its_transport_goes_inside_a_building() {
    let (mut w, t, m, v) = riding_grip();
    objects::take_inside(&mut w, t);
    assert!(!w.obj(t).unwrap().on_map, "the transport is inside");
    assert_eq!(w.obj(m).unwrap().target, None, "the rider loses its target");
    for _ in 0..20 {
        run(&mut w, 1);
        assert!(lift::grips(&w, m, v), "the grip lasts, tick {}", w.tick);
    }
}
