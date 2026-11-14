//! Claim tests for S-0402 claim 11's docking, payment and unloading
//! animations (S-0402 Terms; S-0108 claims 12 and 20) and the harvester's
//! wait for the unloading animation (#430), with r84's order within the
//! payment and empty-load rows (Q-3062, Q-3063), on synthetic data:
//! invented animation names, frame counts, types and loads, no game data.

mod harvest_common;

use std::collections::BTreeMap;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::AnimationType;
use glasnost_sim::depot::DepotSlot;
use glasnost_sim::depot_anim;
use glasnost_sim::econ::harvest::{self, DockState, HarvestState};
use glasnost_sim::rules::Behaviour;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};
use harvest_common::*;

fn anim(frames: i32, loop_count: Option<i32>) -> AnimationType {
    AnimationType {
        frame_count: Some(frames),
        middle_frame: Some(frames / 2),
        loop_count,
        ..Default::default()
    }
}

fn pair(n: &str, d: &str) -> [String; 2] {
    [n.to_owned(), d.to_owned()]
}

/// The harvest ruleset plus animation types DOCK, DOCKL (a long docking
/// animation), PAY, PAYE (endless), UNLD and their damaged forms DOCKD,
/// PAYD, UNLDD; REFN's art gives docking DOCK/DOCKD, payment PAY/PAYD and
/// unloading UNLD/UNLDD; DOCKB is REFN without the refinery flag (a
/// dock-unload building that is not a refinery-flag building).
fn rules() -> Ruleset {
    let mut r = ruleset();
    for n in [
        "DOCK", "DOCKL", "DOCKD", "PAY", "PAYE", "PAYD", "UNLD", "UNLDD",
    ] {
        r.registries
            .entry("animations".into())
            .or_default()
            .push(n.into());
    }
    let s = &mut r.sim;
    for (n, f, l) in [
        ("DOCK", 10, None),
        ("DOCKL", 120, None),
        ("DOCKD", 10, None),
        ("PAY", 6, None),
        // spec: S-0109 Import mapping (LoopCount −1: endless)
        ("PAYE", 6, Some(-1)),
        ("PAYD", 6, None),
        ("UNLD", 20, None),
        ("UNLDD", 20, None),
    ] {
        s.animations.insert(n.into(), anim(f, l));
    }
    let names = BTreeMap::from([
        ("PreProductionAnim".to_owned(), pair("DOCK", "DOCKD")),
        ("SpecialAnim".to_owned(), pair("PAY", "PAYD")),
        ("ProductionAnim".to_owned(), pair("UNLD", "UNLDD")),
    ]);
    let refn = s.buildings.get_mut("REFN").unwrap();
    refn.econ.slot_names = Some(names);
    let mut dockb = refn.clone();
    dockb.econ.refinery = None;
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("DOCKB".into());
    r.sim.buildings.insert("DOCKB".into(), dockb);
    r
}

/// Changes the slot names of `building` in `r`.
fn names(r: &mut Ruleset, building: &str, f: impl FnOnce(&mut BTreeMap<String, [String; 2]>)) {
    let b = r.sim.buildings.get_mut(building).unwrap();
    f(b.econ.slot_names.as_mut().unwrap());
}

/// A harvester docked at a building of type `building` (contact, slot
/// held) with `load`, facing east, in unload; the building's top-left
/// cell is (−6, −1) and the harvester stands on its docking cell (−3, 0),
/// with the building in its west neighbour cell.
fn docked(w: &mut World, building: &str, load: Vec<i32>) -> (ObjectId, ObjectId) {
    let refn = place_building(w, building, 0, at(-6, -1));
    let h = place_vehicle(w, "HARV", 0, at(-3, 0), 16384);
    let o = w.obj_mut(h).unwrap();
    o.contact = Some(refn);
    o.econ.harvest = Some(HarvestState {
        load,
        ..Default::default()
    });
    w.obj_mut(refn).unwrap().econ.dock = Some(DockState { holders: vec![h] });
    let t = w.tick;
    w.obj_mut(h).unwrap().beh.force(Behaviour::Unload, t, true);
    (refn, h)
}

/// The names of the building's running modelled slot animations.
fn playing(w: &World, b: ObjectId) -> Vec<String> {
    depot_anim::draws(w, b)
        .into_iter()
        .map(|d| d.name)
        .collect()
}

fn plays(w: &World, b: ObjectId, name: &str) -> bool {
    playing(w, b).iter().any(|n| n == name)
}

/// Runs tick by tick until `f` holds, returning the tick after the step in
/// which it first held.
fn until(w: &mut World, max: i32, mut f: impl FnMut(&World) -> bool) -> Option<i32> {
    for _ in 0..max {
        testkit::run(w, 1);
        if f(w) {
            return Some(w.tick);
        }
    }
    None
}

/// The ticks of one unloading of `load` at a `building`: the change to
/// *finishing* (f), the end of the unloading animation (e, `None` when it
/// never played) and the contact break.
struct Run {
    f: i32,
    e: Option<i32>,
    broken: i32,
}

fn unload(r: &Ruleset, building: &str, load: Vec<i32>) -> Run {
    let mut w = game(r);
    let (b, h) = docked(&mut w, building, load);
    let f = until(&mut w, 400, |w| harvest::state_of(w, h).finishing).expect("finishing");
    let mut e = None;
    let mut broken = None;
    let mut seen = plays(&w, b, "UNLD");
    for _ in 0..400 {
        testkit::run(&mut w, 1);
        let now = plays(&w, b, "UNLD");
        if seen && !now && e.is_none() {
            e = Some(w.tick);
        }
        seen |= now;
        if w.obj(h).unwrap().contact.is_none() {
            broken = Some(w.tick);
            break;
        }
    }
    Run {
        f,
        e,
        broken: broken.expect("contact broken"),
    }
}

// spec: S-0402 Terms and claim 11 (the start row: the building in the neighbouring cell plays its docking animation; a payment row: its payment animation; the empty-load row: the payment animation stopped, the unloading animation of a refinery-flag building played)
#[test]
fn claim_11_docking_payment_and_unloading_animations() {
    let r = rules();
    let mut w = game(&r);
    let (b, h) = docked(&mut w, "REFN", vec![2, 0]);
    assert!(playing(&w, b).is_empty(), "nothing plays before unloading");
    until(&mut w, 100, |w| harvest::state_of(w, h).unloading).expect("unloading");
    assert!(
        plays(&w, b, "DOCK"),
        "the docking animation: {:?}",
        playing(&w, b)
    );
    let c0 = w.houses[0].credits;
    until(&mut w, 100, |w| w.houses[0].credits != c0).expect("the payment");
    assert!(
        plays(&w, b, "PAY"),
        "the payment animation: {:?}",
        playing(&w, b)
    );
    assert!(!plays(&w, b, "UNLD"));
    until(&mut w, 100, |w| harvest::state_of(w, h).finishing).expect("finishing");
    assert!(
        plays(&w, b, "UNLD"),
        "the unloading animation: {:?}",
        playing(&w, b)
    );
}

// spec: S-0402 claim 11 (the empty-load row stops the payment animation if playing; a payment while it plays does not start it again)
#[test]
fn claim_11_payment_animation_kept_then_stopped() {
    let mut r = rules();
    names(&mut r, "REFN", |n| {
        n.insert("SpecialAnim".into(), pair("PAYE", "PAYE"));
    });
    let mut w = game(&r);
    let (b, h) = docked(&mut w, "REFN", vec![2, 3]);
    let c0 = w.houses[0].credits;
    until(&mut w, 100, |w| w.houses[0].credits != c0).expect("first payment");
    let first = w
        .obj(b)
        .unwrap()
        .depot
        .slots
        .get(&DepotSlot::Special1)
        .copied();
    assert!(first.is_some(), "the payment animation plays");
    let c1 = w.houses[0].credits;
    until(&mut w, 100, |w| w.houses[0].credits != c1).expect("second payment");
    assert_eq!(
        w.obj(b)
            .unwrap()
            .depot
            .slots
            .get(&DepotSlot::Special1)
            .copied(),
        first,
        "already playing: not started again"
    );
    until(&mut w, 100, |w| harvest::state_of(w, h).finishing).expect("finishing");
    assert!(!plays(&w, b, "PAYE"), "stopped at the empty-load row");
}

// spec: S-0402 claim 11 (finishing waits while the refinery-flag building still plays its unloading animation) and scenario H1 variant e (i) (contact is broken in tick e or e + 1, e the end of the unloading animation, and not before it)
#[test]
fn claim_11_harvester_waits_for_the_unloading_animation() {
    let r = rules();
    let run = unload(&r, "REFN", vec![2, 0]);
    let e = run.e.expect("the unloading animation ends");
    assert!(e > run.f + 1, "the animation lasts: f {} e {e}", run.f);
    assert!(
        run.broken == e || run.broken == e + 1,
        "broken at {} with e {e}",
        run.broken
    );
}

// spec: S-0402 scenario H1 variant e (ii) (a longer docking animation leaves the break at the same tick relative to f: the docking and payment animations never delay the harvester)
#[test]
fn claim_11_docking_animation_does_not_delay() {
    let short = unload(&rules(), "REFN", vec![2, 0]);
    let mut r = rules();
    names(&mut r, "REFN", |n| {
        n.insert("PreProductionAnim".into(), pair("DOCKL", "DOCKL"));
    });
    let long = unload(&r, "REFN", vec![2, 0]);
    assert_eq!(long.broken - long.f, short.broken - short.f);
}

// spec: S-0402 claim 11 and scenario H1 variant e (iii) (no unloading animation: contact broken in tick f + 1)
#[test]
fn claim_11_no_unloading_animation_no_wait() {
    let mut r = rules();
    names(&mut r, "REFN", |n| {
        n.remove("ProductionAnim");
    });
    let run = unload(&r, "REFN", vec![2, 0]);
    assert_eq!(run.e, None);
    assert_eq!(run.broken, run.f + 1);
}

// spec: S-0402 Terms and scenario H1 variant e (a dock-unload building without the refinery flag still plays its docking and payment animations, and supplies no unloading animation or wait)
#[test]
fn claim_11_non_refinery_dock_building_plays_no_unloading_animation() {
    let r = rules();
    let mut w = game(&r);
    let (b, h) = docked(&mut w, "DOCKB", vec![2, 0]);
    until(&mut w, 100, |w| harvest::state_of(w, h).unloading).expect("unloading");
    assert!(plays(&w, b, "DOCK"), "{:?}", playing(&w, b));
    let c0 = w.houses[0].credits;
    until(&mut w, 100, |w| w.houses[0].credits != c0).expect("the payment");
    assert!(plays(&w, b, "PAY"), "{:?}", playing(&w, b));
    let run = unload(&r, "DOCKB", vec![2, 0]);
    assert_eq!(run.e, None, "no unloading animation");
    assert_eq!(run.broken, run.f + 1, "no wait");
}

/// `rules()` with a refinery smoke type on REFN (S-0616 claim 1): the
/// smoke system PLUME (smoke kind, PUFF, a long emission interval) at one
/// set smoke point, with a long refinery-smoke time.
fn smoky_rules() -> Ruleset {
    use glasnost_rules::sim::{ParticleSystemType, ParticleType};
    let mut r = rules();
    r.registries
        .entry("particles".into())
        .or_default()
        .push("PUFF".into());
    r.sim.particles.insert(
        "PUFF".into(),
        ParticleType {
            behaves_like: Some("smoke".into()),
            max_ec: Some(1000),
            ..Default::default()
        },
    );
    r.registries
        .entry("particle_systems".into())
        .or_default()
        .push("PLUME".into());
    r.sim.particle_systems.insert(
        "PLUME".into(),
        ParticleSystemType {
            behaves_like: Some("smoke".into()),
            holds_what: Some("PUFF".into()),
            spawn_frames: Some(1000),
            spawn_cutoff: dec("100000.0"),
            ..Default::default()
        },
    );
    let refn = r.sim.buildings.get_mut("REFN").unwrap();
    refn.refinery_smoke_particle_system = Some("PLUME".into());
    refn.refinery_smoke_offsets = Some(vec![[40, 0, 0]]);
    r.sim.general.refinery_smoke_frames = Some(5000);
    r
}

/// The animation in building `b`'s `slot`, if one runs.
fn slot(w: &World, b: ObjectId, s: DepotSlot) -> Option<ObjectId> {
    w.obj(b).unwrap().depot.slots.get(&s).copied()
}

// spec: S-0402 claim 11 (r84, Refinery smoke: the smoke event comes first in its row, before the row's animations start; Order within the two rows: the smoke event's particle systems enter play, then the payment animation) and scenario H28 part a
#[test]
fn claim_11_r84_smoke_before_payment_animation() {
    let r = smoky_rules();
    let mut w = game(&r);
    let (b, _h) = docked(&mut w, "REFN", vec![2, 0]);
    let c0 = w.houses[0].credits;
    until(&mut w, 100, |w| w.houses[0].credits != c0).expect("the payment");
    let pay = slot(&w, b, DepotSlot::Special1).expect("the payment animation plays");
    let smoke: Vec<ObjectId> = w
        .gas
        .systems
        .iter()
        .filter(|(_, s)| s.owner_object == Some(b))
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(smoke.len(), 1, "one smoke system at the payment");
    assert!(
        smoke[0] < pay,
        "the smoke system {:?} enters play before the payment animation {pay:?}",
        smoke[0]
    );
}

/// The scenario-stream values taken from docking at REFN with an empty
/// load until the tick in which the harvester becomes *finishing*, with
/// PAY's random rate `rate`, and the world at the end of that tick.
fn empty_docking(rate: Option<[i32; 2]>) -> (u64, World, ObjectId) {
    let mut r = rules();
    r.sim.animations.get_mut("PAY").unwrap().random_rate = rate;
    let mut w = game(&r);
    let (b, h) = docked(&mut w, "REFN", vec![0, 0]);
    let t0 = w.rng.taken();
    until(&mut w, 200, |w| harvest::state_of(w, h).finishing).expect("finishing");
    (w.rng.taken() - t0, w, b)
}

// spec: S-0402 claim 11 (r84, Order within the two rows: in the empty-load row the payment animation is started unless playing and always stopped last, so one that was not playing enters play with its creation draws and leaves play in the same processing; a docking with an empty load plays it this way) and scenario H28 part b; S-0109 claim 3 (one ranged draw when d₁′ ≠ d₂′)
#[test]
fn claim_11_r84_empty_load_row_starts_and_stops_the_payment_animation() {
    let (plain, _, _) = empty_docking(None);
    let (ranged, w, b) = empty_docking(Some([450, 225]));
    assert!(
        ranged > plain,
        "the payment animation's creation draw is taken: {ranged} vs {plain}"
    );
    assert_eq!(
        slot(&w, b, DepotSlot::Special1),
        None,
        "the payment animation has left play"
    );
    assert!(plays(&w, b, "UNLD"), "{:?}", playing(&w, b));
}

// spec: S-0402 Terms (r84: the art read fills an empty damaged name with the normal name, S-0108 Import mapping, so a damaged building whose art gives only the normal name plays the normal animation, for each of the three alike) and scenario H28 part d
#[test]
fn claim_11_r84_damaged_with_only_normal_names_plays_normal() {
    let mut r = rules();
    names(&mut r, "REFN", |n| {
        n.insert("PreProductionAnim".into(), pair("DOCK", "DOCK"));
        n.insert("SpecialAnim".into(), pair("PAY", "PAY"));
        n.insert("ProductionAnim".into(), pair("UNLD", "UNLD"));
    });
    let mut w = game(&r);
    let (b, h) = docked(&mut w, "REFN", vec![2, 0]);
    // spec: S-0003 claim C9 (the yellow threshold: the testkit's 0.5)
    let full = w.obj(b).unwrap().health;
    w.obj_mut(b).unwrap().health = full / 2;
    until(&mut w, 100, |w| harvest::state_of(w, h).unloading).expect("unloading");
    assert!(plays(&w, b, "DOCK"), "{:?}", playing(&w, b));
    let c0 = w.houses[0].credits;
    until(&mut w, 100, |w| w.houses[0].credits != c0).expect("the payment");
    assert!(plays(&w, b, "PAY"), "{:?}", playing(&w, b));
    until(&mut w, 100, |w| harvest::state_of(w, h).finishing).expect("finishing");
    assert!(plays(&w, b, "UNLD"), "{:?}", playing(&w, b));
    let f = w.tick;
    let mut broken = None;
    for _ in 0..200 {
        testkit::run(&mut w, 1);
        if w.obj(h).unwrap().contact.is_none() {
            broken = Some(w.tick);
            break;
        }
    }
    assert!(
        broken.expect("contact broken") > f + 1,
        "the harvester waits for the normal unloading animation"
    );
}

// spec: S-0402 Terms (r84: a start plays nothing when the art gives only the damaged name and the building is not damaged) and scenario H28 part d with H1 variant e (iii) (no animation; the harvester changes from finishing 1 tick later)
#[test]
fn claim_11_r84_only_damaged_names_at_full_health_play_nothing() {
    let mut r = rules();
    names(&mut r, "REFN", |n| {
        n.insert("PreProductionAnim".into(), pair("", "DOCKD"));
        n.insert("SpecialAnim".into(), pair("", "PAYD"));
        n.insert("ProductionAnim".into(), pair("", "UNLDD"));
    });
    let mut w = game(&r);
    let (b, h) = docked(&mut w, "REFN", vec![2, 0]);
    until(&mut w, 100, |w| harvest::state_of(w, h).unloading).expect("unloading");
    let c0 = w.houses[0].credits;
    until(&mut w, 100, |w| w.houses[0].credits != c0).expect("the payment");
    assert!(playing(&w, b).is_empty(), "{:?}", playing(&w, b));
    let run = unload(&r, "REFN", vec![2, 0]);
    assert_eq!(run.e, None);
    assert_eq!(run.broken, run.f + 1);
}

// spec: S-0402 Terms (each is played in its damaged form when the building's health ratio is at or below the yellow threshold) and S-0108 claim 12
#[test]
fn claim_11_damaged_forms() {
    let r = rules();
    let mut w = game(&r);
    let (b, h) = docked(&mut w, "REFN", vec![2, 0]);
    // spec: S-0003 claim C9 (the yellow threshold: the testkit's 0.5)
    let full = w.obj(b).unwrap().health;
    w.obj_mut(b).unwrap().health = full / 2;
    until(&mut w, 100, |w| harvest::state_of(w, h).unloading).expect("unloading");
    assert!(plays(&w, b, "DOCKD"), "{:?}", playing(&w, b));
    let c0 = w.houses[0].credits;
    until(&mut w, 100, |w| w.houses[0].credits != c0).expect("the payment");
    assert!(plays(&w, b, "PAYD"), "{:?}", playing(&w, b));
    until(&mut w, 100, |w| harvest::state_of(w, h).finishing).expect("finishing");
    assert!(plays(&w, b, "UNLDD"), "{:?}", playing(&w, b));
    assert!(
        !playing(&w, b)
            .iter()
            .any(|n| n == "DOCK" || n == "PAY" || n == "UNLD")
    );
}
