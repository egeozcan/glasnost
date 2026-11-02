//! Claim tests of S-0522 (parachutes and drops), claims 4 to 15 and the
//! drop part of claim 16, on synthetic rules (`tests/sw_common`, extended
//! here). The launch side (claims 1 to 3 and 16) is tested in
//! `claims_s0513_planes.rs`. Every name and number is invented.
//!
//! Additions to the superweapon ruleset: the parachute animation CHUTE
//! (a random rate pair whose delays differ, so its creation takes one
//! ranged draw), the
//! parachute sound `Whoosh`, a parachute maximum fall rate of −4, the
//! infantry type STUCK (float locomotion class, with a float multiplier
//! of 0 on clear and road land, so it can enter no cell of the map), sequence entries for WALKER (stand, walk, parachute) and
//! the armed aircraft GUNSHIP (primary weapon Gun).

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    AnimationType, Decimal, LandType, LocomotionClass, Locomotor, SequenceEntry, UnitType,
};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::infantry::Seq;
use glasnost_sim::rules::Behaviour;
use glasnost_sim::superweapons::{SwEvent, planes};
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};
use glasnost_sim::{air, paradrop, save, transport, walk};
use sw_common::*;

/// The parachute maximum fall rate of these rules.
const CHUTE_FLOOR: i32 = -4;

fn reg(r: &mut Ruleset, registry: &str, name: &str) {
    r.registries
        .entry(registry.into())
        .or_default()
        .push(name.into());
}

/// The superweapon ruleset with this file's additions.
fn rules() -> Ruleset {
    let mut r = ruleset();
    // The paradrop plane is armed, as the aircraft weapon rows of S-0522's
    // Import mapping and S-0501 claim 20 take it to be: a weaponless one
    // answers *impossible* at its target cell (S-0301 claim 9) and drops it
    // at its first check tick (S-0309 claim 29 r94; the test
    // `a_weaponless_paradrop_plane_drops_its_cell_and_retreats` below).
    r.sim.aircraft.get_mut("PDPLANE").unwrap().primary = Some("Gun".into());
    r.sim.animations.insert(
        "CHUTE".into(),
        AnimationType {
            random_rate: Some([450, 225]),
            // spec: S-0109 claim 7 (loop count −1: r₀ = 255, endless), so the parachute is still present on landing
            loop_count: Some(-1),
            ..Default::default()
        },
    );
    reg(&mut r, "animations", "CHUTE");
    {
        let g = &mut r.sim.general.superweapons;
        g.parachute = Some("CHUTE".into());
        g.chute_sound = Some("Whoosh".into());
    }
    r.sim.general.parachute_max_fall_rate = Some(CHUTE_FLOOR);
    r.sim.infantry.insert(
        "STUCK".into(),
        UnitType {
            strength: Some(100),
            speed: Some(4),
            locomotion_class: Some(LocomotionClass::Float),
            locomotor: Some(Locomotor::Walk),
            ..Default::default()
        },
    );
    reg(&mut r, "infantry", "STUCK");
    for l in [LandType::Clear, LandType::Road] {
        r.sim.land_types.get_mut(&l).unwrap().float = Some(Decimal("0".into()));
    }
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    for k in ["stand", "walk", "parachute"] {
        walker.sequences.insert(
            k.into(),
            SequenceEntry {
                start: 0,
                count: 4,
                multiplier: 0,
                direction: None,
            },
        );
    }
    r.sim.aircraft.insert(
        "GUNSHIP".into(),
        UnitType {
            strength: Some(100),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(Locomotor::Fly),
            primary: Some("Gun".into()),
            passengers: Some(3),
            ..Default::default()
        },
    );
    reg(&mut r, "aircraft", "GUNSHIP");
    r
}

/// These rules with the second side group's payload replaced by `n`
/// units of `name`.
fn rules_with(name: &str, n: i32) -> Ruleset {
    let mut r = rules();
    let g = &mut r.sim.general.superweapons;
    g.sov_para_drop_inf = Some(vec![name.into()]);
    g.sov_para_drop_num = Some(vec![n]);
    r
}

/// The single plane of a paradrop by ALPHA (second side group) at the
/// middle cell.
fn launch(w: &mut World) -> ObjectId {
    planes::paradrop(w, ALPHA, at(0, 0), false)[0]
}

fn seq(w: &World, id: ObjectId) -> Option<Seq> {
    w.obj(id)?.infantry.as_ref()?.seq
}

fn falling(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.fall.falling)
}

fn passes(w: &World, plane: ObjectId) -> i32 {
    w.obj(plane).unwrap().air.as_ref().unwrap().passes
}

/// Puts `plane` at (`x`, `y`) at its present height, facing `f`, with
/// ammunition `ammo`, for a drop attempt made outside a tick.
fn hold(w: &mut World, plane: ObjectId, x: i32, y: i32, f: u16, ammo: i32) {
    let t = w.tick;
    let o = w.obj_mut(plane).unwrap();
    o.pos = Pos::new(x, y, o.pos.z);
    o.body.set(f, t, false);
    o.combat.ammo = ammo;
}

/// The west edge point of cell `c` at its middle row, so that the even
/// side's drop point (claim 9, F4, facing 0) is `c`'s centre.
fn west_of(c: CellXY) -> (i32, i32) {
    (256 * c.x, 256 * c.y + 128)
}

fn chute_events(w: &World) -> Vec<ObjectId> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            glasnost_sim::combat::effects::Effect::Superweapon {
                event: SwEvent::Parachute { trooper, .. },
                ..
            } => Some(*trooper),
            _ => None,
        })
        .collect()
}

fn surface(w: &World, id: ObjectId) -> i32 {
    let p = w.obj(id).unwrap().pos;
    w.surface_at(p.x, p.y)
}

// --- Claims 4 and 5 -----------------------------------------------------------

#[test]
fn claims4_5_planes_arrive_from_and_retreat_to_the_house_edge() {
    // spec: S-0522 claims 4 and 5 (the arrival cell lies on the house's arrival edge, outside the usable area; retreat flies to a cell on the edge chosen then, S-0501 claim 34). The edge geometry itself is tested in claims_s0501.rs.
    let mut w = game(&rules());
    let p = launch(&mut w);
    assert!(!w.cells.usable(w.obj(p).unwrap().pos.cell()));
    until(&mut w, 3000, |w| transport::count(w, p) == 0);
    until(&mut w, 100, |w| {
        let o = w.obj(p).unwrap();
        o.beh.current == Some(Behaviour::Retreat) && o.dest.is_some()
    });
    let Some(Target::Cell(c)) = w.obj(p).unwrap().dest else {
        panic!("a cell destination")
    };
    assert!(!w.cells.usable(c), "an edge cell");
}

// --- Claim 6 -----------------------------------------------------------------

#[test]
fn claim6_approach_sets_destination_and_queues_overfly_inside_radius() {
    // spec: S-0522 claim 6 (no destination: the target becomes the destination; within the radius: overfly next, pass counter − 1; 3-tick processing delay)
    let mut w = game(&rules());
    let p = launch(&mut w);
    assert_eq!(passes(&w, p), 5, "Constants: the pass counter starts at 5");
    until(&mut w, 10, |w| w.obj(p).unwrap().dest.is_some());
    let o = w.obj(p).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(at(0, 0))));
    assert_eq!(o.beh.last.map(|l| l.1), Some(3));
    until(&mut w, 2000, |w| {
        w.obj(p).unwrap().beh.queued == Some(Behaviour::ParadropOverfly)
            || w.obj(p).unwrap().beh.current == Some(Behaviour::ParadropOverfly)
    });
    assert_eq!(passes(&w, p), 4);
    let o = w.obj(p).unwrap();
    let d =
        (o.pos.x - (256 * at(0, 0).x + 128)).pow(2) + (o.pos.y - (256 * at(0, 0).y + 128)).pow(2);
    assert!(
        d <= 1024 * 1024 + 2 * 1024,
        "within the default radius of 1024"
    );
}

#[test]
fn claim6_no_target_loses_destination_and_retreats() {
    // spec: S-0522 claim 6 (with no target it loses its destination and its next behaviour is retreat)
    let mut w = game(&rules());
    let p = launch(&mut w);
    until(&mut w, 10, |w| w.obj(p).unwrap().dest.is_some());
    w.obj_mut(p).unwrap().target = None;
    // The processing itself, so that S-0501 claim 10's exit (no target) does not remove the plane first.
    assert_eq!(paradrop::approach(&mut w, p), 3);
    let o = w.obj(p).unwrap();
    assert_eq!(o.dest, None);
    assert_eq!(o.beh.queued, Some(Behaviour::Retreat));
}

// spec: S-0309 claim 29 r94 (every aircraft makes the check in its own behaviours, those in which no weapon is aimed at the held target included; illegal or impossible drops it at a check tick, with no draw) and S-0301 claim 9 (no weapon in the slot: impossible)
// spec: S-0522 claim 6 (with no target the plane loses its destination and its next behaviour is retreat)
#[test]
fn a_weaponless_paradrop_plane_drops_its_cell_and_retreats() {
    let mut r = rules();
    r.sim.aircraft.get_mut("PDPLANE").unwrap().primary = None;
    let mut w = game(&r);
    let p = launch(&mut w);
    let launched = w.tick;
    let after = until(&mut w, 40, |w| w.obj(p).is_none_or(|o| o.target.is_none()));
    let dropped = after - 1;
    assert_eq!(dropped % 16, 0, "dropped in a check tick");
    assert!(dropped - launched < 16, "at the first check tick");
    assert_eq!(transport::count(&w, p), 3, "nobody dropped");
    until(&mut w, 20, |w| {
        w.obj(p).is_none_or(|o| {
            o.beh.current == Some(Behaviour::Retreat) || o.beh.queued == Some(Behaviour::Retreat)
        })
    });
}

// --- Claim 7 -----------------------------------------------------------------

#[test]
fn claim7_overfly_holds_the_run_lock_and_drops_every_five_ticks() {
    // spec: S-0522 claim 7 (inside the radius: run lock held, one attempt per 5-tick processing, at most one paratrooper per attempt); F1
    let mut w = game(&rules());
    let p = launch(&mut w);
    let t1 = until(&mut w, 2000, |w| transport::count(w, p) < 3);
    let o = w.obj(p).unwrap();
    assert_eq!(transport::count(&w, p), 2, "one paratrooper per attempt");
    assert_eq!(o.beh.current, Some(Behaviour::ParadropOverfly));
    assert!(o.air.as_ref().unwrap().run_lock);
    assert_eq!(o.beh.last.map(|l| l.1), Some(5));
    let t2 = until(&mut w, 2000, |w| transport::count(w, p) < 2);
    assert_eq!((t2 - t1) % 5, 0, "attempts are 5 ticks apart");
}

#[test]
fn claim7_no_passenger_left_retreats_without_target() {
    // spec: S-0522 claim 7 (no passenger left: run lock off, target and destination lost, next behaviour retreat)
    let mut w = game(&rules());
    let p = launch(&mut w);
    until(&mut w, 3000, |w| transport::count(w, p) == 0);
    until(&mut w, 20, |w| {
        let o = w.obj(p).unwrap();
        o.beh.current == Some(Behaviour::Retreat) || o.beh.queued == Some(Behaviour::Retreat)
    });
    let o = w.obj(p).unwrap();
    assert_eq!(o.target, None);
    assert!(!o.air.as_ref().unwrap().run_lock);
}

// --- Claim 8 -----------------------------------------------------------------

#[test]
fn claim8_five_failed_passes_then_silent_exit_with_the_paratroopers() {
    // spec: S-0522 claim 8 (five passes without a success, then retreat; paratroopers still on board leave play without a death or loss count)
    // spec: S-0501 claim 10 (in retreat it is removed once outside the usable area) and claim 4 (one that may leave the map now, without the fly-by flag, is shifted back from the map's edge): on a map whose edge cells are usable it would stay on the map, so the map keeps its edge band outside the usable area (S-0522 claim 4)
    let mut w = banded_game(&rules_with("STUCK", 2));
    let p = launch(&mut w);
    let ps = transport::passengers(&w, p);
    let losses = glasnost_sim::econ::stats::of(&w, ALPHA).losses;
    let mut entries = 0;
    let mut was = w.obj(p).unwrap().beh.current;
    until(&mut w, 20_000, |w| {
        let now = w.obj(p).and_then(|o| o.beh.current);
        if now == Some(Behaviour::ParadropOverfly) && was != now {
            entries += 1;
        }
        was = now;
        now == Some(Behaviour::Retreat)
    });
    assert_eq!(entries, 5, "five passes");
    assert_eq!(passes(&w, p), 0);
    assert_eq!(
        transport::count(&w, p),
        2,
        "it retreats with its paratroopers"
    );
    assert_eq!(w.obj(p).unwrap().target, None);
    until(&mut w, 20_000, |w| w.obj(p).is_none());
    for x in ps {
        assert!(
            w.obj(x).is_none(),
            "the paratroopers left play with the plane"
        );
    }
    assert_eq!(glasnost_sim::econ::stats::of(&w, ALPHA).losses, losses);
}

// --- Claims 9 and 16 ---------------------------------------------------------

#[test]
fn claim9_successful_drop_on_the_even_side() {
    // spec: S-0522 claim 9 and F4 (ammunition − 1; even: f + 16383, facing 0 drops to the east; at the standing position, plane height, facing 32768, falling, parachute sequence, sound at the plane)
    let mut w = game(&rules());
    let p = launch(&mut w);
    let c = at(1, 1);
    let (x, y) = west_of(c);
    hold(&mut w, p, x, y, 0, 3);
    let first = transport::passengers(&w, p)[0];
    let z = w.obj(p).unwrap().pos.z;
    w.effects.clear();
    assert!(paradrop::attempt(&mut w, p));
    assert_eq!(w.obj(p).unwrap().combat.ammo, 2);
    assert_eq!(transport::count(&w, p), 2);
    let o = w.obj(first).unwrap();
    assert!(o.on_map);
    assert_eq!(o.pos.cell(), c);
    assert_eq!(o.pos.z, z, "at the plane's height");
    assert_eq!(o.body.value(w.tick), 32768);
    assert_eq!(o.layer, Layer::Ground);
    assert!(o.fall.falling && o.fall.chute);
    assert_eq!(seq(&w, first), Some(Seq::Parachute));
    let sound: Vec<_> = w
        .effects
        .iter()
        .filter_map(|e| match e {
            glasnost_sim::combat::effects::Effect::Superweapon {
                event: SwEvent::Chute,
                pos,
                ..
            } => *pos,
            _ => None,
        })
        .collect();
    assert_eq!(
        sound,
        vec![Pos::new(x, y, z)],
        "the sound plays at the plane"
    );
    assert_eq!(chute_events(&w), vec![first]);
}

#[test]
fn claim9_odd_side_is_the_planes_left() {
    // spec: S-0522 F4 (odd new ammunition: f − 16383; a plane facing north drops to the west)
    let mut w = game(&rules());
    let p = launch(&mut w);
    let c = at(1, 1);
    hold(&mut w, p, 256 * c.x + 256, 256 * c.y + 128, 0, 4);
    let first = transport::passengers(&w, p)[0];
    assert!(paradrop::attempt(&mut w, p));
    assert_eq!(w.obj(p).unwrap().combat.ammo, 3);
    assert_eq!(w.obj(first).unwrap().pos.cell(), c);
    // The drop point is F4's step (S-0501 F3 with step 128).
    let (dx, _) = walk::step_point(
        256 * c.x + 256,
        256 * c.y + 128,
        0u16.wrapping_sub(16383),
        128,
        true,
        w.rules().mode,
    );
    assert_eq!(dx, 256 * c.x + 128);
}

#[test]
fn claim9_entry_refusal_keeps_passenger_and_ammunition_and_takes_no_draw() {
    // spec: S-0522 claim 9 (Q-0750: an entry outcome that is not clear, a class restriction included, suppresses the standing-position selection: the paratrooper stays first, the ammunition goes back up, no draw is taken)
    let mut w = game(&rules_with("STUCK", 2));
    let p = launch(&mut w);
    let (x, y) = west_of(at(1, 1));
    hold(&mut w, p, x, y, 0, 3);
    let before = transport::passengers(&w, p);
    let expected = w.rng.clone();
    assert!(!paradrop::attempt(&mut w, p));
    assert_eq!(transport::passengers(&w, p), before);
    assert_eq!(w.obj(p).unwrap().combat.ammo, 3);
    assert_eq!(w.rng, expected, "no standing-position draw");
}

#[test]
fn claim9_refusals_after_a_clear_entry_take_the_standing_position_draw() {
    // spec: S-0522 claims 9 and 16 (Q-0750: once entry is clear the standing position is selected, so a later refusal, here a high-bridge cell without the deck-entry mark, has taken the centre-class row's R(0, 3))
    let mut w = game(&rules());
    let p = launch(&mut w);
    let c = at(1, 1);
    bridge(&mut w, c, false);
    let (x, y) = west_of(c);
    hold(&mut w, p, x, y, 0, 3);
    let before = transport::passengers(&w, p);
    let mut expected = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    expected.ranged(0, 3, half);
    assert!(!paradrop::attempt(&mut w, p));
    assert_eq!(transport::passengers(&w, p), before);
    assert_eq!(w.obj(p).unwrap().combat.ammo, 3);
    assert_eq!(w.rng, expected);
}

#[test]
fn claim9_the_deck_mark_follows_the_actual_height() {
    // spec: S-0522 claim 9 (Q-0750: at or above the terrain surface plus 416 leptons on a high-bridge cell the paratrooper marks the deck, below that the ground; exact integer comparisons)
    for (dz, layer) in [(0, Layer::Deck), (-1, Layer::Ground)] {
        let mut w = game(&rules());
        let p = launch(&mut w);
        let c = at(1, 1);
        bridge(&mut w, c, true);
        let (x, y) = west_of(c);
        hold(&mut w, p, x, y, 0, 3);
        let surface = w.surface_at(256 * c.x + 128, 256 * c.y + 128);
        w.obj_mut(p).unwrap().pos.z = surface + 416 + dz;
        let first = transport::passengers(&w, p)[0];
        assert!(paradrop::attempt(&mut w, p));
        assert_eq!(w.obj(first).unwrap().layer, layer, "dz {dz}");
    }
}

#[test]
fn claim16_drop_draws_standing_position_then_parachute() {
    // spec: S-0522 claim 16 (a successful centre-class drop: R(0, 3), then the parachute animation's creation draws, claim 13)
    let mut w = game(&rules());
    let p = launch(&mut w);
    let (x, y) = west_of(at(1, 1));
    hold(&mut w, p, x, y, 0, 3);
    let half = w.rules().quirks.ranged_draw_half_span;
    let (d1, d2) = w.rules().animation("CHUTE").unwrap().random_delay;
    assert_ne!(d1, d2);
    let mut expected = w.rng.clone();
    expected.ranged(0, 3, half);
    expected.ranged(d1, d2, half);
    assert!(paradrop::attempt(&mut w, p));
    assert_eq!(w.rng, expected);
}

// --- Claim 10 ----------------------------------------------------------------

#[test]
fn claim10_a_falling_paratrooper_refuses_a_second_drop_into_its_cell() {
    // spec: S-0522 claim 10 (a falling paratrooper occupies its landing layer already, so another drop there is refused)
    let mut w = game(&rules());
    let p = launch(&mut w);
    let (x, y) = west_of(at(1, 1));
    hold(&mut w, p, x, y, 0, 3);
    assert!(paradrop::attempt(&mut w, p));
    hold(&mut w, p, x, y, 0, 3);
    let first = transport::passengers(&w, p)[0];
    assert!(!paradrop::attempt(&mut w, p));
    assert_eq!(transport::passengers(&w, p)[0], first);
}

#[test]
fn claim10_cells_the_class_cannot_enter_refuse_every_drop() {
    // spec: S-0522 claim 10 (over cells its class cannot enter, nothing is dropped; the plane keeps its paratroopers)
    let mut w = game(&rules_with("STUCK", 1));
    let p = launch(&mut w);
    let x = transport::passengers(&w, p)[0];
    until(&mut w, 2000, |w| {
        w.obj(p).unwrap().beh.current == Some(Behaviour::ParadropOverfly)
    });
    for _ in 0..40 {
        step(&mut w);
        assert!(!w.obj(x).unwrap().on_map, "nothing is dropped");
    }
    assert_eq!(transport::count(&w, p), 1);
}

fn bridge(w: &mut World, c: CellXY, entry: bool) {
    let i = w.cells.idx(c).expect("cell");
    w.cells.deck[i] = true;
    w.cells.deck_entry[i] = entry;
    w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::Y;
}

#[test]
fn claim10_over_a_bridge_the_paratrooper_lands_on_the_deck() {
    // spec: S-0522 claims 9, 10 and 12 (a bridge cell with the deck-entry mark: on the deck; it lands on the deck, 416 above the terrain)
    let mut w = game(&rules());
    let p = launch(&mut w);
    let c = at(1, 1);
    bridge(&mut w, c, true);
    let (x, y) = west_of(c);
    hold(&mut w, p, x, y, 0, 3);
    let first = transport::passengers(&w, p)[0];
    assert!(paradrop::attempt(&mut w, p));
    assert_eq!(w.obj(first).unwrap().layer, Layer::Deck);
    until(&mut w, 2000, |w| !falling(w, first));
    assert_eq!(w.obj(first).unwrap().pos.z, surface(&w, first) + 416);
}

#[test]
fn claim9_bridge_without_deck_entry_refuses() {
    // spec: S-0522 claim 9 (a bridge cell must have the deck-entry mark)
    let mut w = game(&rules());
    let p = launch(&mut w);
    let c = at(1, 1);
    bridge(&mut w, c, false);
    let (x, y) = west_of(c);
    hold(&mut w, p, x, y, 0, 3);
    assert!(!paradrop::attempt(&mut w, p));
}

// --- Claim 11 ----------------------------------------------------------------

#[test]
fn claim11_a_carrying_aircrafts_shot_is_a_drop() {
    // spec: S-0522 claim 11 and S-0501 claim 20 (a shot of an aircraft with passengers is a drop attempt with no projectile; success clears the rearm wait and resets the pass counter)
    let mut w = game(&rules());
    let g = spawn(&mut w, "GUNSHIP", ALPHA, -4, -4);
    let inf = spawn(&mut w, "WALKER", ALPHA, 4, 4);
    transport::board(&mut w, inf, g);
    assert_eq!(transport::count(&w, g), 1);
    w.obj_mut(g)
        .unwrap()
        .air
        .get_or_insert_with(Default::default)
        .passes = 2;
    let c = at(1, 1);
    let (x, y) = west_of(c);
    hold(&mut w, g, x, y, 0, 3);
    let objects = w.objects.len();
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        g,
        0,
        Target::Cell(at(3, 3))
    ));
    assert_eq!(transport::count(&w, g), 0);
    assert!(w.obj(inf).unwrap().on_map);
    assert_eq!(w.obj(inf).unwrap().pos.cell(), c);
    let o = w.obj(g).unwrap();
    assert_eq!(o.combat.rearm_until, w.tick, "the rearm wait is cleared");
    assert_eq!(o.air.as_ref().unwrap().passes, 5);
    assert_eq!(w.objects.len(), objects, "no projectile");
}

/// A gunship holding at the west edge of cell (1, 1) with ammunition 3 and
/// one passenger of type `passenger`.
fn carrier(r: &Ruleset, passenger: &str) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let g = spawn(&mut w, "GUNSHIP", ALPHA, -4, -4);
    let inf = spawn(&mut w, passenger, ALPHA, 4, 4);
    transport::board(&mut w, inf, g);
    let (x, y) = west_of(at(1, 1));
    hold(&mut w, g, x, y, 0, 3);
    (w, g, inf)
}

#[test]
fn claim11_a_shot_drop_replaces_the_ordinary_shot_effects() {
    // spec: S-0522 claim 11 (Q-0751): no ordinary delay and no delay draw, no burst-counter advance, no spray draw, no projectile and no last-fired mark; a successful drop expires the rearm countdown now and resets the passes, its draws are the attempt's alone
    let r = rules();
    let (mut a, g, _) = carrier(&r, "WALKER");
    let (mut b, gb, _) = carrier(&r, "WALKER");
    // The twin makes the drop attempt directly: the shot adds nothing to its draws.
    assert!(paradrop::attempt(&mut b, gb));
    let before = a.obj(g).unwrap().clone();
    let projectiles = a.projectiles.len();
    assert!(glasnost_sim::combat::fire::shot(
        &mut a,
        g,
        0,
        Target::Cell(at(3, 3))
    ));
    assert_eq!(a.rng, b.rng, "only the attempt's own draws");
    let o = a.obj(g).unwrap();
    assert_eq!(o.combat.burst, before.combat.burst);
    assert_eq!(o.last_fired, before.last_fired);
    assert_eq!(o.combat.rearm_delay, before.combat.rearm_delay);
    assert_eq!(o.combat.rearm_until, a.tick, "expired now");
    assert_eq!(a.projectiles.len(), projectiles);
}

#[test]
fn claim11_a_refused_shot_drop_changes_nothing_but_still_counts_as_a_shot() {
    // spec: S-0522 claim 11 (Q-0751): a refused drop leaves the rearm countdown, the burst counter, the passes and the draws unchanged, and is not a void shot (the firing pass still makes a round pending, S-0501 claim 15, so the shot reports true)
    let r = rules();
    let (mut w, g, inf) = carrier(&r, "STUCK");
    {
        let t = w.tick;
        let o = w.obj_mut(g).unwrap();
        o.combat.rearm_until = t + 9;
        o.air.get_or_insert_with(Default::default).passes = 2;
    }
    let before = w.obj(g).unwrap().clone();
    let rng = w.rng.clone();
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        g,
        0,
        Target::Cell(at(3, 3))
    ));
    assert_eq!(w.rng, rng);
    let o = w.obj(g).unwrap();
    assert_eq!(o.combat.rearm_until, before.combat.rearm_until);
    assert_eq!(o.combat.burst, before.combat.burst);
    assert_eq!(o.combat.ammo, before.combat.ammo);
    assert_eq!(o.air.as_ref().unwrap().passes, 2);
    assert_eq!(transport::passengers(&w, g), vec![inf]);
}

/// The S-0301 claim tests of the drop attempt (revision 17), on a carrier whose weapon has a
/// burst of 3.
fn burst_carrier() -> (World, ObjectId, ObjectId) {
    let mut r = rules();
    let gun = r.sim.weapons.get_mut("Gun").unwrap();
    gun.burst = Some(3);
    // A projectile that always has a launch solution, so that the ordinary shot is not void.
    gun.projectile = Some("Bullet".into());
    carrier(&r, "WALKER")
}

#[test]
fn s0301_claims_1_4_18_a_drop_attempt_takes_no_ordinary_delay_burst_step_or_draw() {
    // spec: S-0301 claim 1 (a drop attempt leaves the countdown unchanged on refusal and expires it immediately on success; no ordinary shot delay), claim 4 (an attempt made while a first passenger exists never advances the burst counter or chooses an ordinary delay, draws included; once no first passenger remains, further attempts of the same burst are not cargo drops) and claim 18 (no ordinary-shot draws)
    let (mut w, g, inf) = burst_carrier();
    {
        let t = w.tick;
        let o = w.obj_mut(g).unwrap();
        o.combat.rearm_until = t + 9;
        o.combat.rearm_delay = 9;
    }
    let (mut twin, tg, _) = burst_carrier();
    assert!(paradrop::attempt(&mut twin, tg));
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        g,
        0,
        Target::Cell(at(3, 3))
    ));
    let o = w.obj(g).unwrap();
    assert!(w.obj(inf).unwrap().on_map, "released");
    assert_eq!(
        o.combat.rearm_until, w.tick,
        "success expires the countdown"
    );
    assert_eq!(o.combat.burst, 0, "the burst counter did not advance");
    assert_eq!(w.rng, twin.rng, "only the attempt's own draws");
    // The exception ends with the cargo: the next attempt of the same burst is an ordinary shot.
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        g,
        0,
        Target::Cell(at(2, 1))
    ));
    let o = w.obj(g).unwrap();
    assert_eq!(o.combat.burst, 1, "an ordinary shot advances the counter");
    let wait = o.combat.rearm_until - w.tick;
    assert!((3..=5).contains(&wait), "the in-burst delay, got {wait}");
}

#[test]
fn s0301_claims_10_15_a_drop_spends_one_round_at_once_and_is_not_a_void_shot() {
    // spec: S-0301 claim 10 (a successful drop spends one round immediately, a refused one changes nothing), claim 15 (a drop is not a void shot although it launches no projectile; its rearm countdown expires, a refusal retains it; neither produces ordinary-shot effects: no weapon report, no muzzle animation)
    let (mut w, g, _) = burst_carrier();
    let projectiles = w.projectiles.len();
    w.effects.clear();
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        g,
        0,
        Target::Cell(at(3, 3))
    ));
    assert_eq!(w.obj(g).unwrap().combat.ammo, 2, "one round at once");
    assert_eq!(w.projectiles.len(), projectiles);
    assert!(
        w.effects.iter().all(|e| !matches!(
            e,
            glasnost_sim::combat::effects::Effect::Shot { .. }
                | glasnost_sim::combat::effects::Effect::Anim { .. }
                | glasnost_sim::combat::effects::Effect::ShotAnim { .. }
        )),
        // spec: S-0501 claim 20 (revision 19: no shot animation, S-0301 claims 20 to 23)
        "no report, no muzzle animation and no shot animation"
    );
    // A refused drop: the round count and the countdown stay.
    let r = rules();
    let (mut w, g, inf) = carrier(&r, "STUCK");
    let t = w.tick;
    w.obj_mut(g).unwrap().combat.rearm_until = t + 9;
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        g,
        0,
        Target::Cell(at(3, 3))
    ));
    assert_eq!(w.obj(g).unwrap().combat.ammo, 3);
    assert_eq!(w.obj(g).unwrap().combat.rearm_until, t + 9);
    assert_eq!(transport::passengers(&w, g), vec![inf]);
}

// --- Claims 12 and 13 --------------------------------------------------------

/// The per-update height changes of a paratrooper dropped from a held
/// plane, until it lands.
fn descent(r: &Ruleset) -> (World, ObjectId, Vec<i32>) {
    let mut w = game(r);
    let p = launch(&mut w);
    let (x, y) = west_of(at(1, 1));
    hold(&mut w, p, x, y, 0, 3);
    let first = transport::passengers(&w, p)[0];
    assert!(paradrop::attempt(&mut w, p));
    let mut z = w.obj(first).unwrap().pos.z;
    let mut steps = Vec::new();
    while falling(&w, first) {
        step(&mut w);
        let nz = w.obj(first).unwrap().pos.z;
        steps.push(nz - z);
        z = nz;
        assert!(steps.len() < 5000);
    }
    (w, first, steps)
}

/// The total sensor-disc count of house `h` (the discs of its sensor units).
fn disc_total(w: &World, h: u8) -> i32 {
    w.cloak
        .sensors
        .iter()
        .filter(|((hh, _), _)| *hh == h)
        .map(|(_, n)| *n)
        .sum()
}

#[test]
fn claim9_12_release_adds_the_sensor_disc_at_the_release_cell_and_landing_keeps_it() {
    // spec: S-0522 claim 9 (revision 5: a release adds any mobile sensor coverage at once at the release cell, with no adjacency reveal; descent does not relocate it) and claim 12 (landing supplies S-0516 claim 22's cell-arrival update, also in the same cell, before the parachute ends)
    let mut r = rules();
    {
        let w = r.sim.infantry.get_mut("WALKER").unwrap();
        w.combat.sensors = Some(true);
        w.combat.sensors_sight = Some(2);
    }
    let mut w = game(&r);
    let p = launch(&mut w);
    let (x, y) = west_of(at(1, 1));
    // One round: this is the only drop, so no second trooper adds a disc.
    hold(&mut w, p, x, y, 0, 1);
    let first = transport::passengers(&w, p)[0];
    assert_eq!(disc_total(&w, ALPHA.0), 0, "a passenger covers nothing");
    assert!(paradrop::attempt(&mut w, p));
    let release = w.obj(first).unwrap().pos.cell();
    let disc = |w: &World| w.obj(first).unwrap().cloak.disc;
    assert_eq!(
        disc(&w),
        Some((ALPHA, release)),
        "the disc exists in the release tick, at the release cell"
    );
    assert!(w.cloak.sensors.contains_key(&(ALPHA.0, release)));
    // Descent moves nothing; the landing keeps the disc at the same cell.
    let mut ticks = 0;
    while falling(&w, first) {
        step(&mut w);
        ticks += 1;
        assert!(ticks < 5000);
        assert_eq!(disc(&w), Some((ALPHA, release)), "not moved in the air");
    }
    assert_eq!(w.obj(first).unwrap().pos.cell(), release);
    assert_eq!(disc(&w), Some((ALPHA, release)));
    assert!(w.cloak.sensors.contains_key(&(ALPHA.0, release)));
}

#[test]
fn claim12_parachute_recurrence_and_landing() {
    // spec: S-0522 claim 12 and F5 (S-0511 F3 with the parachute recurrence v′ = max(v − 1, floor) while the parachute is attached; landing below 1 lepton: height 0, no damage, stand)
    let (mut w, id, steps) = descent(&rules());
    assert!(steps.len() > 6, "a plane at cruise height");
    assert_eq!(&steps[..5], &[0, -1, -2, -3, -4]);
    let last = steps.len() - 1;
    // A 0 is an update the object did not get (S-0203 claim 2's removal skip, as the leaving plane is taken out of the update order).
    assert!(steps[4..last].iter().all(|&d| d == CHUTE_FLOOR || d == 0));
    let o = w.obj(id).unwrap();
    assert_eq!(o.pos.z, surface(&w, id));
    assert!(!o.fall.chute);
    assert_eq!(o.health, 100, "no fall damage");
    step(&mut w);
    assert_eq!(
        seq(&w, id),
        Some(Seq::Stand),
        "S-0107: asks for stand on landing"
    );
}

#[test]
fn claim12_falling_paratrooper_turns_down_other_sequences() {
    // spec: S-0107 claim 6 row 2 (while the sequence is parachute and the unit falls, a request is turned down); S-0522 claim 12
    let mut w = game(&rules());
    let p = launch(&mut w);
    let (x, y) = west_of(at(1, 1));
    hold(&mut w, p, x, y, 0, 3);
    let first = transport::passengers(&w, p)[0];
    assert!(paradrop::attempt(&mut w, p));
    assert!(!glasnost_sim::infantry::request(
        &mut w,
        first,
        Seq::Walk,
        true
    ));
    step(&mut w);
    assert_eq!(seq(&w, first), Some(Seq::Parachute));
}

#[test]
fn claim13_no_parachute_type_uses_the_no_parachute_recurrence() {
    // spec: S-0522 claim 13 (Q-0752: with no parachute type nothing stays attached, so after the release tick the no-parachute recurrence applies)
    let mut r = rules();
    r.sim.general.superweapons.parachute = None;
    let (w, id, steps) = descent(&r);
    assert!(
        steps.iter().any(|&d| d < CHUTE_FLOOR),
        "faster than the parachute floor"
    );
    assert!(chute_events(&w).is_empty());
    assert!(!w.obj(id).unwrap().fall.chute);
}

#[test]
fn claim13_absent_type_uses_the_parachute_recurrence_once_in_the_release_tick() {
    // spec: S-0522 claim 13 (Q-0752: an absent parachute type does not refuse the release and takes no animation draws; with quirks.paratrooper_any_animation_slow_fall on the release-tick update uses the parachute recurrence once and the no-parachute one from the next tick; off, the no-parachute one from the first update; a parachute floor of 0 tells them apart)
    let mut r = rules();
    r.sim.general.superweapons.parachute = None;
    r.sim.general.parachute_max_fall_rate = Some(0);
    r.sim.general.no_parachute_max_fall_rate = Some(-50);
    let (w, id, steps) = descent(&r);
    assert_eq!(&steps[..4], &[0, 0, -1, -2], "{:?}", &steps[..6]);
    assert!(!w.obj(id).unwrap().fall.chute && !w.obj(id).unwrap().fall.chute_once);
    assert!(chute_events(&w).is_empty(), "no attached animation");
    r.globals.quirks.paratrooper_any_animation_slow_fall = false;
    let (_, _, off) = descent(&r);
    assert_eq!(&off[..4], &[0, -1, -2, -3]);
    // No animation-creation draw: only the standing-position draw.
    let mut w = game(&r);
    let p = launch(&mut w);
    let (x, y) = west_of(at(1, 1));
    hold(&mut w, p, x, y, 0, 3);
    let mut expected = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    expected.ranged(0, 3, half);
    assert!(paradrop::attempt(&mut w, p));
    assert_eq!(w.rng, expected);
}

#[test]
fn claim13_an_unlisted_name_is_a_type_not_an_absent_one() {
    // spec: S-0522 claim 13 (Q-0752: a non-reserved unlisted name creates an animation type; release attaches it normally, it has no fields so takes no draw; the absent-type transient does not apply)
    let mut r = rules();
    r.sim.general.superweapons.parachute = Some("NEWCHUTE".into());
    let (w, id, steps) = descent(&r);
    assert_eq!(&steps[..5], &[0, -1, -2, -3, -4]);
    assert!(chute_events(&w).contains(&id));
}

#[test]
fn quirk_any_animation_slow_fall_both_settings_agree() {
    // spec: S-0522 Edge cases (quirks.paratrooper_any_animation_slow_fall: for a held parachute type the parachute is the only attachment, so both settings give the same fall)
    let mut r = rules();
    let (_, _, on) = descent(&r);
    r.globals.quirks.paratrooper_any_animation_slow_fall = false;
    let (_, _, off) = descent(&r);
    assert_eq!(on, off);
}

#[test]
fn claim13_parachute_attached_until_landing() {
    // spec: S-0522 claim 13 (the parachute follows the unit down; an endless one still present on landing gets remaining passes 0, so its current pass is its last, S-0109 claim 16)
    let mut w = game(&rules());
    let p = launch(&mut w);
    let (x, y) = west_of(at(1, 1));
    hold(&mut w, p, x, y, 0, 3);
    let first = transport::passengers(&w, p)[0];
    assert!(paradrop::attempt(&mut w, p));
    let chute = glasnost_sim::anims::parachute_of(&w, first).expect("attached");
    for _ in 0..5 {
        step(&mut w);
        assert!(w.obj(first).unwrap().fall.chute);
        let u = w.obj(first).unwrap().pos;
        let shown = glasnost_sim::anims::shown(&w, chute).and_then(|s| s.2);
        assert_eq!(shown, Some(u), "at the paratrooper's position");
    }
    until(&mut w, 2000, |w| !falling(w, first));
    assert!(!w.obj(first).unwrap().fall.chute);
    assert!(
        glasnost_sim::anims::shown(&w, chute).is_some(),
        "the last pass still plays"
    );
    until(&mut w, 2000, |w| {
        glasnost_sim::anims::shown(w, chute).is_none()
    });
}

#[test]
fn claim13_a_short_parachute_ends_before_landing() {
    // spec: S-0522 claim 13 (the loop count decides the passes, so a short finite parachute ends before landing) and claim 12 (then the no-parachute recurrence applies)
    let mut r = rules();
    let a = r.sim.animations.get_mut("CHUTE").unwrap();
    a.loop_count = Some(1);
    a.frame_count = Some(4);
    let mut w = game(&r);
    let p = launch(&mut w);
    let (x, y) = west_of(at(1, 1));
    hold(&mut w, p, x, y, 0, 3);
    let first = transport::passengers(&w, p)[0];
    assert!(paradrop::attempt(&mut w, p));
    until(&mut w, 200, |w| !w.obj(first).unwrap().fall.chute);
    assert!(falling(&w, first), "the parachute ended in the air");
    assert!(glasnost_sim::anims::parachute_of(&w, first).is_none());
    let before = w.obj(first).unwrap().fall.rate;
    step(&mut w);
    assert!(
        w.obj(first).unwrap().fall.rate < before.min(CHUTE_FLOOR),
        "past the parachute floor: the no-parachute recurrence"
    );
}

// --- Claim 17 ----------------------------------------------------------------

#[test]
fn claim17_released_infantry_rests_with_guard_queued_and_a_move_click_while_falling_is_kept_for_landing()
 {
    // spec: S-0522 claim 17 (Q-0753: at release the ordinary placement fall-back to rest leaves the current behaviour none with guard or area guard queued, no target and no destination; r12, Q-2000: falling does not limit the player's orders, so a move click during the fall gives a move order at once, which sets the destination; the unit takes no step while falling and carries the move out after landing) and S-0214 Terms (r62: falling is part of neither movable nor able to fire); scenario P10 (c)
    let mut w = game(&rules());
    let p = launch(&mut w);
    let (x, y) = west_of(at(1, 1));
    hold(&mut w, p, x, y, 0, 3);
    let first = transport::passengers(&w, p)[0];
    assert!(paradrop::attempt(&mut w, p));
    let o = w.obj(first).unwrap();
    assert_eq!(o.beh.current, None);
    assert!(
        matches!(o.beh.queued, Some(Behaviour::Guard | Behaviour::AreaGuard)),
        "{:?}",
        o.beh.queued
    );
    assert!(o.target.is_none() && o.dest.is_none());
    assert!(
        glasnost_sim::targeting::movable(&w, ALPHA, first),
        "falling: still movable"
    );
    let goal = at(4, 1);
    let click = glasnost_sim::targeting::click(
        &w,
        ALPHA,
        &[first],
        glasnost_sim::targeting::ClickOn::Cell(goal),
        glasnost_sim::targeting::Modifiers::default(),
    );
    assert_eq!(click.orders.len(), 1, "a move order at once");
    issue_all(&mut w, click.orders);
    step(&mut w);
    assert!(falling(&w, first), "still in the air");
    assert_eq!(w.obj(first).unwrap().dest, Some(Target::Cell(goal)));
    let (fx, fy) = (w.obj(first).unwrap().pos.x, w.obj(first).unwrap().pos.y);
    while falling(&w, first) {
        let o = w.obj(first).unwrap();
        assert_eq!((o.pos.x, o.pos.y), (fx, fy), "no step while falling");
        assert_eq!(o.dest, Some(Target::Cell(goal)), "the move is kept");
        step(&mut w);
    }
    until(&mut w, 2000, |w| w.obj(first).unwrap().pos.cell() == goal);
}

#[test]
fn s0215_claims_4_11_a_fresh_paratrooper_with_no_current_behaviour_waits_450_ticks() {
    // spec: S-0215 claim 4 (a due processing with the current behaviour none does nothing and chooses exactly 450 ticks (no delay draw can be part of it), changing neither slot) and claim 11 (B44: placement queued rest, falling prevents its start; landing issues no order; the queued rest starts when ordinary readiness permits, without waiting for the 450-tick delay)
    let mut w = game(&rules());
    let p = launch(&mut w);
    let (x, y) = west_of(at(1, 1));
    hold(&mut w, p, x, y, 0, 3);
    let first = transport::passengers(&w, p)[0];
    assert!(paradrop::attempt(&mut w, p));
    let queued = w.obj(first).unwrap().beh.queued;
    assert!(matches!(
        queued,
        Some(Behaviour::Guard | Behaviour::AreaGuard)
    ));
    assert!(falling(&w, first));
    // A due processing now (nothing has run for this unit yet).
    glasnost_sim::process::turn(&mut w, first);
    let at = w.tick;
    let o = w.obj(first).unwrap();
    assert_eq!(o.beh.current, None, "still none");
    assert_eq!(o.beh.queued, queued, "the queued rest is untouched");
    assert_eq!(o.beh.last, Some((at, 450)), "exactly 450 ticks");
    // (A delay of exactly 450 leaves no room for the standard delay's 0 to 2 draw.)
    assert!(o.target.is_none() && o.dest.is_none());
    // While it falls the rest does not start; after landing it starts well before 450 ticks.
    while falling(&w, first) {
        assert_eq!(
            w.obj(first).unwrap().beh.current,
            None,
            "not started in the air"
        );
        step(&mut w);
    }
    until(&mut w, 200, |w| w.obj(first).unwrap().beh.current.is_some());
    assert!(
        matches!(
            w.obj(first).unwrap().beh.current,
            Some(Behaviour::Guard | Behaviour::AreaGuard)
        ),
        "the queued rest started"
    );
    assert!(
        w.tick < at + 450,
        "started before the 450-tick delay was over"
    );
}

// --- Claim 14 ----------------------------------------------------------------

#[test]
fn claim14_killed_in_descent_is_removed_and_frees_the_cell() {
    // spec: S-0522 claim 14 and S-0107 claim 17 (death kind 3 while parachuting: removed at once; the cell can take a later drop)
    let mut w = game(&rules());
    let p = launch(&mut w);
    let (x, y) = west_of(at(1, 1));
    hold(&mut w, p, x, y, 0, 3);
    let first = transport::passengers(&w, p)[0];
    assert!(paradrop::attempt(&mut w, p));
    step(&mut w);
    kill(&mut w, first);
    assert!(w.obj(first).is_none(), "removed at once");
    hold(&mut w, p, x, y, 0, 3);
    assert!(paradrop::attempt(&mut w, p), "the freed cell takes a drop");
}

// --- Claim 15 ----------------------------------------------------------------

#[test]
fn claim15_plane_shot_down_kills_its_passengers_not_the_dropped_ones() {
    // spec: S-0522 claim 15 (a crashing plane's passengers die, credited to the attacker; those already dropped are unaffected)
    let mut w = game(&rules());
    let p = launch(&mut w);
    let (x, y) = west_of(at(1, 1));
    hold(&mut w, p, x, y, 0, 3);
    let dropped = transport::passengers(&w, p)[0];
    assert!(paradrop::attempt(&mut w, p));
    let aboard = transport::passengers(&w, p);
    assert_eq!(aboard.len(), 2);
    let shooter = spawn(&mut w, "WALKER", BETA, -4, -4);
    let kills = glasnost_sim::econ::stats::of(&w, BETA).kills;
    damage::apply(
        &mut w,
        p,
        Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: Some(shooter),
            house: Some(BETA),
            ignore_defences: true,
            no_escape: false,
        },
    );
    for x in aboard {
        assert!(
            w.obj(x).is_none_or(damage::dead),
            "a passenger dies with the crashing plane"
        );
    }
    assert!(
        glasnost_sim::econ::stats::of(&w, BETA).kills >= kills + 2,
        "the passengers' deaths are credited to the attacker"
    );
    let o = w.obj(dropped).unwrap();
    assert!(!damage::dead(o));
    assert!(o.on_map);
}

// --- Claims 4 and 5 (through the overfly) and the save round trip -----------

#[test]
fn paratroopers_land_and_stand_for_their_house() {
    // spec: S-0522 claims 6 to 9 and 12 (a full paradrop: every paratrooper lands, stands and stays its house's)
    let mut w = game(&rules());
    let p = launch(&mut w);
    let ps = transport::passengers(&w, p);
    let full: Vec<i32> = ps.iter().map(|&x| w.obj(x).unwrap().health).collect();
    until(&mut w, 3000, |w| transport::count(w, p) == 0);
    until(&mut w, 3000, |w| ps.iter().all(|&x| !falling(w, x)));
    step(&mut w);
    for (x, h) in ps.into_iter().zip(full) {
        let o = w.obj(x).unwrap();
        // spec: S-0522 claim 12 and Edge cases (no paradrop fall damage)
        assert_eq!(o.health, h);
        assert_eq!(o.owner, Some(ALPHA));
        assert!(o.on_map);
        assert_eq!(o.pos.z, surface(&w, x));
        assert_eq!(seq(&w, x), Some(Seq::Stand));
    }
    assert!(air::is_aircraft(&w, p) || w.obj(p).is_none());
}

#[test]
fn save_load_mid_descent_round_trips() {
    // spec: S-0522 claims 7, 12 and 13 (the pass counter, the fall and the parachute attachment are state that survives a save and load)
    let mut w = game(&rules());
    let p = launch(&mut w);
    let ps = transport::passengers(&w, p);
    until(&mut w, 3000, |w| ps.iter().any(|&x| falling(w, x)));
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    for _ in 0..200 {
        step(&mut w);
        step(&mut l);
    }
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
}

// --- Claim 17 with the deploy key (M8 A4 round 5, note 32) -------------------

/// These rules with the second side group's payload one DROPDEP: WALKER
/// made an in-place deployer with no undeploy delay (deploy, deployed and
/// undeploy sequences of its own).
fn deployer_rules() -> Ruleset {
    let mut r = rules_with("DROPDEP", 1);
    let mut inf = r.sim.infantry["WALKER"].clone();
    inf.deployer = Some(true);
    inf.undeploy_delay = Some(-1);
    for (k, n) in [("deploy", 4), ("deployed", 1), ("undeploy", 6)] {
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
    r.sim.infantry.insert("DROPDEP".into(), inf);
    reg(&mut r, "infantry", "DROPDEP");
    r
}

fn deploy_press(w: &World, id: ObjectId) -> Vec<glasnost_sim::orders::Order> {
    glasnost_sim::targeting::key_command(
        w,
        ALPHA,
        &[id],
        glasnost_sim::targeting::KeyCommand::Deploy,
    )
}

/// Drops the first paratrooper of the deployer rules and returns it,
/// falling.
fn dropped_deployer(w: &mut World) -> ObjectId {
    let p = launch(w);
    let (x, y) = west_of(at(1, 1));
    hold(w, p, x, y, 0, 3);
    let first = transport::passengers(w, p)[0];
    assert!(paradrop::attempt(w, p));
    assert!(falling(w, first));
    first
}

fn deployed(w: &World, id: ObjectId) -> bool {
    matches!(
        seq(w, id),
        Some(Seq::Deploy | Seq::Deployed | Seq::DeployedIdle | Seq::DeployedFire)
    )
}

fn issue_all(w: &mut World, orders: Vec<glasnost_sim::orders::Order>) {
    for o in orders {
        assert!(glasnost_sim::orders::issue(w, ALPHA, o));
    }
}

#[test]
fn claim17_a_deploy_press_while_falling_deploys_after_landing() {
    // spec: S-0522 claim 17 (r12, Q-2000: a deploy press during the fall is taken as an order at once; its behaviour becomes the queued behaviour in place of the queued rest; the unit cannot start it while falling; after landing an in-place deployer deploys) with S-0213 table T2's deploy rule and claim 10 (unload); scenario P10 (a)
    let mut w = game(&deployer_rules());
    let first = dropped_deployer(&mut w);
    let orders = deploy_press(&w, first);
    assert_eq!(
        orders,
        vec![glasnost_sim::orders::Order::Deploy { actor: first }],
        "an order at once"
    );
    issue_all(&mut w, orders);
    step(&mut w);
    assert!(falling(&w, first));
    assert_eq!(
        w.obj(first).unwrap().beh.queued,
        Some(Behaviour::Unload),
        "queued the tick after the press"
    );
    while falling(&w, first) {
        assert!(!deployed(&w, first), "no deploy in the air");
        step(&mut w);
    }
    until(&mut w, 200, |w| deployed(w, first));
}

#[test]
fn claim17_a_deploy_press_after_landing_deploys() {
    // spec: S-0522 claim 17 (landing issues no order: with no press nothing follows by itself) and scenario P10 (b): the same press after landing gives the ordinary deploy
    let mut w = game(&deployer_rules());
    let first = dropped_deployer(&mut w);
    while falling(&w, first) {
        step(&mut w);
    }
    run(&mut w, 60);
    assert!(!deployed(&w, first), "{:?}", seq(&w, first));
    let orders = deploy_press(&w, first);
    assert_eq!(
        orders,
        vec![glasnost_sim::orders::Order::Deploy { actor: first }]
    );
    issue_all(&mut w, orders);
    until(&mut w, 200, |w| deployed(w, first));
}

#[test]
fn claim17_stop_or_a_second_press_during_the_fall_keeps_the_deploy() {
    // spec: S-0522 claim 17 (a stop order during the fall clears destination and target but keeps the ordered behaviour, S-0213 claim 8, so the deploy still happens after landing; a second press gives the same order again and cancels nothing); scenario P10 (f)
    for second in [false, true] {
        let mut w = game(&deployer_rules());
        let first = dropped_deployer(&mut w);
        let orders = deploy_press(&w, first);
        issue_all(&mut w, orders);
        step(&mut w);
        assert!(falling(&w, first));
        let o = if second {
            deploy_press(&w, first)
        } else {
            vec![glasnost_sim::orders::Order::Stop { actor: first }]
        };
        issue_all(&mut w, o);
        step(&mut w);
        assert!(falling(&w, first));
        assert_eq!(w.obj(first).unwrap().beh.queued, Some(Behaviour::Unload));
        until(&mut w, 2000, |w| !falling(w, first));
        until(&mut w, 200, |w| deployed(w, first));
    }
}

#[test]
fn claim17_a_mixed_selection_deploys_the_landed_at_once_and_the_falling_after_landing() {
    // spec: S-0522 claim 17 (a falling deployer counts as not deployed for the mixed-selection exception; an ordered landed one deploys at its next start, an ordered falling one after landing); scenario P10 (e)
    let mut w = game(&deployer_rules());
    let first = dropped_deployer(&mut w);
    let landed = spawn(&mut w, "DROPDEP", ALPHA, -3, -3);
    run(&mut w, 1);
    assert!(falling(&w, first));
    let orders = glasnost_sim::targeting::key_command(
        &w,
        ALPHA,
        &[landed, first],
        glasnost_sim::targeting::KeyCommand::Deploy,
    );
    assert_eq!(
        orders,
        vec![
            glasnost_sim::orders::Order::Deploy { actor: landed },
            glasnost_sim::orders::Order::Deploy { actor: first },
        ]
    );
    issue_all(&mut w, orders);
    until(&mut w, 200, |w| deployed(w, landed));
    assert!(
        falling(&w, first),
        "the landed one deploys while the other still falls"
    );
    assert!(!deployed(&w, first));
    until(&mut w, 2000, |w| !falling(w, first));
    until(&mut w, 200, |w| deployed(w, first));
}

/// Drops the first paratrooper of the plain rules over cell (1, 1) and
/// places an enemy STUCK (a target that cannot move) at (2, 1), within
/// weapon range of the landing cell. Returns both, the paratrooper
/// falling.
fn dropped_with_enemy(w: &mut World) -> (ObjectId, ObjectId) {
    let p = launch(w);
    let (x, y) = west_of(at(1, 1));
    hold(w, p, x, y, 0, 3);
    let first = transport::passengers(w, p)[0];
    assert!(paradrop::attempt(w, p));
    let enemy = spawn(w, "STUCK", BETA, 2, 1);
    assert!(falling(w, first));
    (first, enemy)
}

#[test]
fn claim17_an_ordinary_click_on_an_enemy_while_falling_gives_a_move_order() {
    // spec: S-0522 claim 17 (r14, Q-2140: the attack action follows the firing rules, which rule every target illegal for a falling unit, S-0301 claim 9 rank 4; without force-fire the cursor over an enemy offers no attack but the fallback move of S-0214 row I28, no-move; the click gives the move order of table M toward the target at once, not an attack order; no shot before landing; the move is carried out after landing) and S-0214 claim 6 (e) (r64: the falling refusal counts against part (e)); scenario P10 (d), first run, and S-0214 T4 part D
    let mut w = game(&rules());
    let (first, enemy) = dropped_with_enemy(&mut w);
    let full = w.obj(enemy).unwrap().health;
    let on = glasnost_sim::targeting::ClickOn::Object(enemy);
    let plain = glasnost_sim::targeting::Modifiers::default();
    assert_eq!(
        glasnost_sim::targeting::cursor(&w, ALPHA, &[first], on, plain),
        glasnost_sim::targeting::Action::NoMove,
        "no attack cursor during the fall"
    );
    let click = glasnost_sim::targeting::click(&w, ALPHA, &[first], on, plain);
    assert!(
        matches!(
            click.orders.as_slice(),
            [glasnost_sim::orders::Order::Behaviour {
                behaviour: Behaviour::Move,
                target: None,
                dest: Some(_),
                ..
            }]
        ),
        "a move order at once, not an attack order: {:?}",
        click.orders
    );
    issue_all(&mut w, click.orders);
    step(&mut w);
    assert!(falling(&w, first));
    let o = w.obj(first).unwrap();
    assert!(o.target.is_none(), "no target recorded");
    assert!(o.dest.is_some(), "a destination by the enemy");
    assert_eq!(o.beh.queued, Some(Behaviour::Move));
    while falling(&w, first) {
        assert_eq!(w.obj(enemy).unwrap().health, full, "no shot in the air");
        step(&mut w);
    }
    // The control: the same click after landing offers attack.
    assert_eq!(
        glasnost_sim::targeting::cursor(&w, ALPHA, &[first], on, plain),
        glasnost_sim::targeting::Action::Attack,
        "after landing the click offers attack"
    );
}

#[test]
fn claim17_a_force_fire_click_while_falling_attacks_after_landing() {
    // spec: S-0522 claim 17 (r14: with force-fire held claim 6 (e) does not apply, so an attack click during the fall is taken as an attack order at once and its target recorded; no shot before landing, S-0203 claim 4 and S-0511 claim 10; the attack after landing) and S-0214 claim 6 (e) (force-fire passes part (e)); scenario P10 (d), second run
    let mut w = game(&rules());
    let (first, enemy) = dropped_with_enemy(&mut w);
    let full = w.obj(enemy).unwrap().health;
    let on = glasnost_sim::targeting::ClickOn::Object(enemy);
    let force = glasnost_sim::targeting::Modifiers {
        force_fire: true,
        ..Default::default()
    };
    assert_eq!(
        glasnost_sim::targeting::cursor(&w, ALPHA, &[first], on, force),
        glasnost_sim::targeting::Action::Attack
    );
    let click = glasnost_sim::targeting::click(&w, ALPHA, &[first], on, force);
    assert!(
        matches!(
            click.orders.as_slice(),
            [glasnost_sim::orders::Order::Behaviour {
                behaviour: Behaviour::Attack,
                ..
            }]
        ),
        "an attack order at once: {:?}",
        click.orders
    );
    issue_all(&mut w, click.orders);
    step(&mut w);
    assert!(falling(&w, first));
    assert_eq!(w.obj(first).unwrap().target, Some(Target::Object(enemy)));
    while falling(&w, first) {
        assert_eq!(w.obj(enemy).unwrap().health, full, "no shot in the air");
        step(&mut w);
    }
    until(&mut w, 600, |w| {
        w.obj(enemy).is_none_or(|o| o.health < full)
    });
}
