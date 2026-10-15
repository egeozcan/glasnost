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
    r.sim.animations.insert(
        "CHUTE".into(),
        AnimationType {
            random_rate: Some([450, 225]),
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
    let mut w = game(&rules_with("STUCK", 2));
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
    // spec: S-0522 claim 13 (the parachute follows the unit down; still present on landing it ends, S-0109 claim 28 seam: no playback clock)
    let mut w = game(&rules());
    let p = launch(&mut w);
    let (x, y) = west_of(at(1, 1));
    hold(&mut w, p, x, y, 0, 3);
    let first = transport::passengers(&w, p)[0];
    assert!(paradrop::attempt(&mut w, p));
    for _ in 0..5 {
        step(&mut w);
        assert!(w.obj(first).unwrap().fall.chute);
    }
    until(&mut w, 2000, |w| !falling(w, first));
    assert!(!w.obj(first).unwrap().fall.chute);
}

// --- Claim 17 ----------------------------------------------------------------

#[test]
fn claim17_released_infantry_rests_with_guard_queued_and_takes_no_move_while_falling() {
    // spec: S-0522 claim 17 (Q-0753): at release the ordinary placement fall-back to rest leaves the current behaviour none with guard or area guard queued, no target and no destination; a falling unit is not movable, so a move click is refused; landing issues, replaces and clears no order, target or destination
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
        !glasnost_sim::targeting::movable(&w, ALPHA, first),
        "falling: not movable"
    );
    until(&mut w, 2000, |w| !falling(w, first));
    let o = w.obj(first).unwrap();
    assert!(o.dest.is_none(), "no move emerges after landing");
    assert!(
        !matches!(o.beh.current, Some(Behaviour::Move | Behaviour::QueuedMove)),
        "{:?}",
        o.beh.current
    );
    assert!(glasnost_sim::targeting::movable(&w, ALPHA, first));
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
