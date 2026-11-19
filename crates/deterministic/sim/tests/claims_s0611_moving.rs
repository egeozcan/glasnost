//! S-0611 claim 15 (Q-3121, #291): the point, marks and frees of
//! infantry-making animations that move (bouncing or attached to an owner)
//! or are removed before their end, under both settings of
//! `quirks.infantry_animation_stale_marks`. Synthetic rules only, on the
//! `sw_common` superweapon ruleset: infantry GROWN (the made type), the
//! animation-to-infantry list `[GROWN]`, a neutral civilian-side house Civ
//! (house 2) and the animation types
//! - LONG: twenty frames at one update a frame, make-infantry index 0;
//! - FLY: a falling type (bouncer flag), two hundred frames at one update a
//!   frame, make-infantry index 0: it lands before its last pass ends;
//! - HOP: a falling type, four frames at one update a frame, make-infantry
//!   index 0: its last pass ends in flight.
//!
//! Every name and number is made up.

mod sw_common;

use std::collections::BTreeSet;

use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::AnimationType;
use glasnost_sim::anims::{self, Spawn};
use glasnost_sim::bounce::Flight;
use glasnost_sim::cells::{Marker, position};
use glasnost_sim::combat::effects;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, World};
use glasnost_sim::{save, testkit};
use sw_common::{BETA, at};

const CIV: HouseId = HouseId(2);

fn reg(r: &mut Ruleset, registry: &str, name: &str) {
    let l = r.registries.entry(registry.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn add(r: &mut Ruleset, name: &str, t: AnimationType) {
    r.sim.animations.insert(name.into(), t);
    reg(r, "animations", name);
}

fn rules(stale: bool) -> Ruleset {
    use glasnost_rules::sim::Country;
    let mut r = sw_common::ruleset();
    r.sim.infantry.get_mut("WALKER").unwrap().primary = None;
    let grown = r.sim.infantry["WALKER"].clone();
    r.sim.infantry.insert("GROWN".into(), grown);
    reg(&mut r, "infantry", "GROWN");
    let frames = |n: i32| AnimationType {
        make_infantry: Some(0),
        frame_count: Some(n),
        rate: Some(900),
        ..Default::default()
    };
    add(&mut r, "LONG", frames(20));
    add(
        &mut r,
        "FLY",
        AnimationType {
            bouncer: Some(true),
            ..frames(200)
        },
    );
    add(
        &mut r,
        "HOP",
        AnimationType {
            bouncer: Some(true),
            ..frames(4)
        },
    );
    r.sim.general.superweapons.anim_to_infantry = Some(vec!["GROWN".into()]);
    r.sim.countries.insert(
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
    reg(&mut r, "countries", "Civland");
    r.globals.quirks.infantry_animation_stale_marks = stale;
    r
}

/// A game on [`rules`] with the neutral civilian-side house Civ (house 2).
fn game(stale: bool) -> World {
    let r = rules(stale);
    let mut s = testkit::setup(1);
    s.houses.push(glasnost_sim::world::HouseSetup {
        name: "Civ".into(),
        country: "Civland".into(),
        human: false,
        start: None,
        role: glasnost_sim::world::HouseRole::Neutral,
        ai: false,
    });
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-test"));
    let mut w = glasnost_sim::load::new_game(
        rules,
        &testkit::empty_map(sw_common::SIZE),
        testkit::catalogue(),
        &s,
    )
    .expect("loads");
    testkit::run(&mut w, 4);
    w
}

/// The right-hand standing position of cell `c`, `dz` above the terrain.
fn side(w: &World, c: CellXY, dz: i32) -> Pos {
    let (u, v) = glasnost_sim::walk::standing_point(position::RIGHT);
    let (x, y) = (256 * c.x + u, 256 * c.y + v);
    Pos::new(x, y, w.surface_at(x, y) + dz)
}

fn grown(w: &World) -> Vec<ObjectId> {
    let ti = w.rules().index_of(Kind::Infantry, "GROWN").unwrap();
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Infantry && o.type_index == ti && o.on_map)
        .map(|o| o.id)
        .collect()
}

/// Every standing position (cell, record, position) animation `a` holds.
fn marks_of(w: &World, a: ObjectId) -> BTreeSet<(i32, i32, bool, usize)> {
    let mut out = BTreeSet::new();
    for dy in -9..=9 {
        for dx in -9..=9 {
            let c = at(dx, dy);
            for (deck, l) in [(false, Layer::Ground), (true, Layer::Deck)] {
                let Some(o) = w.cells.occ(c, l) else { continue };
                for (p, m) in o.marks.iter().enumerate() {
                    if matches!(m, Some(m) if matches!(m.by, Marker::Anim(x, _) if x == a)) {
                        out.insert((c.x, c.y, deck, p));
                    }
                }
            }
        }
    }
    out
}

fn ground_mark(w: &World, c: CellXY, p: u8) -> Option<Marker> {
    w.cells
        .occ(c, Layer::Ground)
        .and_then(|o| o.marks[usize::from(p)])
        .map(|m| m.by)
}

fn playing(w: &World, a: ObjectId) -> bool {
    w.anims.anims.contains_key(&a)
}

/// A bouncing animation `name` at `from` with the flight velocity `v`
/// (leptons per update), set after its creation (S-0319 claim 4).
fn fly(w: &mut World, name: &str, from: Pos, v: [i64; 3]) -> ObjectId {
    let id = anims::spawn(w, name, Some(from), Spawn::default())
        .id
        .expect("made");
    let m = w.rules().mode;
    let a = w.anims.anims.get_mut(&id).expect("in play");
    a.flight = Some(Flight {
        p: [from.x, from.y, from.z].map(|c| F64::from_i64(i64::from(c), m).to_f32(m)),
        v: v.map(|c| F64::from_i64(c, m).to_f32(m)),
    });
    a.pos = Some(from);
    id
}

/// Runs a bouncing animation `a` to its end, returning every standing
/// position it held at a tick's end and whether one it held was ever freed
/// while it played.
fn fly_out(w: &mut World, a: ObjectId) -> (BTreeSet<(i32, i32, bool, usize)>, bool) {
    let mut seen = BTreeSet::new();
    let mut freed = false;
    for _ in 0..300 {
        sw_common::step(w);
        let now = marks_of(w, a);
        if playing(w, a) && !seen.is_subset(&now) {
            freed = true;
        }
        seen.extend(now);
        if !playing(w, a) {
            return (seen, freed);
        }
    }
    panic!("the animation does not end");
}

// spec: S-0611 claim 15 (a bouncing animation's point is its position in play: each update marks the standing position of its point; with quirks.infantry_animation_stale_marks on no earlier mark is freed when the point moves on, so each standing position it marked stays taken) and Landing (a landing tries no creation and makes no infantry; with the switch on no position is freed) (#291)
#[test]
fn claim_15_bouncing_marks_stay_and_a_landing_makes_none() {
    let mut w = game(true);
    let from = side(&w, at(-2, 0), 300);
    let a = fly(&mut w, "FLY", from, [40, 0, 0]);
    let (seen, freed) = fly_out(&mut w, a);
    let cells: BTreeSet<(i32, i32)> = seen.iter().map(|&(x, y, _, _)| (x, y)).collect();
    assert!(
        cells.len() >= 2,
        "the flight marks standing positions of two cells or more: {seen:?}"
    );
    assert!(!freed, "no mark is freed while it flies");
    assert_eq!(
        marks_of(&w, a),
        seen,
        "every mark stays taken after the landing"
    );
    assert!(grown(&w).is_empty(), "a landing makes no infantry");
}

// spec: S-0611 claim 15 (switch off: an update that marks another standing position first frees the earlier one; a removal other than a successful end attempt, the landing included, frees the last mark) and Landing (no infantry) (#291)
#[test]
fn claim_15_bouncing_marks_freed_with_the_switch_off() {
    let mut w = game(false);
    let from = side(&w, at(-2, 0), 300);
    let a = fly(&mut w, "FLY", from, [40, 0, 0]);
    let mut seen = BTreeSet::new();
    for _ in 0..300 {
        sw_common::step(&mut w);
        let now = marks_of(&w, a);
        assert!(now.len() <= 1, "one mark at a time: {now:?}");
        seen.extend(now);
        if !playing(&w, a) {
            break;
        }
    }
    assert!(!playing(&w, a));
    assert!(seen.len() >= 2, "it marked as it moved: {seen:?}");
    assert!(
        marks_of(&w, a).is_empty(),
        "the landing frees its last mark"
    );
    assert!(grown(&w).is_empty(), "a landing makes no infantry");
}

// spec: S-0611 claim 15 (a last pass that ends in flight: the end attempt at the point after that update's motion step; with the switch on it frees the standing position of that point, so the positions marked before stay taken; the unit is made by claims 9 and 10 at that point) (#291)
#[test]
fn claim_15_end_attempt_in_flight() {
    let mut w = game(true);
    let from = side(&w, at(-2, 0), 300);
    let a = fly(&mut w, "HOP", from, [200, 0, 0]);
    let (seen, _) = fly_out(&mut w, a);
    assert!(seen.len() >= 2, "it marked as it flew: {seen:?}");
    let made = grown(&w);
    assert_eq!(made.len(), 1, "the end attempt in flight makes the unit");
    assert_eq!(w.obj(made[0]).unwrap().owner, Some(CIV));
    assert!(
        !marks_of(&w, a).is_empty(),
        "the marks made in flight stay taken"
    );
}

// spec: S-0611 claim 15 (switch off: the end attempt frees the standing position the animation marked last, in its record, and still places at its point; no mark is left) (#291)
#[test]
fn claim_15_end_attempt_in_flight_switch_off() {
    let mut w = game(false);
    let from = side(&w, at(-2, 0), 300);
    let a = fly(&mut w, "HOP", from, [200, 0, 0]);
    let (seen, _) = fly_out(&mut w, a);
    assert!(!seen.is_empty());
    assert_eq!(grown(&w).len(), 1);
    assert!(marks_of(&w, a).is_empty(), "every mark freed");
}

/// LONG attached to a vehicle at the middle cell, two cells east of it.
fn attached(w: &mut World) -> (ObjectId, ObjectId) {
    let car = sw_common::spawn(w, "CART", BETA, 0, 0);
    let p = w.obj(car).unwrap().pos;
    let d = Pos::new(512, 0, 0);
    let s = Spawn {
        owner: Some((car, d)),
        ..Default::default()
    };
    let a = anims::spawn(w, "LONG", Some(Pos::new(p.x + d.x, p.y, p.z)), s)
        .id
        .expect("made");
    (car, a)
}

/// Moves `id` to the centre of cell `c` at terrain height without taking
/// it off the map, as a teleporting jump does: going off the map would end
/// the attachment (S-0109 r45 claim 22).
fn move_to(w: &mut World, id: ObjectId, c: CellXY) {
    let old = w.obj(id).unwrap().pos.cell();
    w.cells.leave_both(old, id);
    let p = Pos::centre(c, 0);
    let z = w.surface_at(p.x, p.y);
    w.obj_mut(id).unwrap().pos = Pos::new(p.x, p.y, z);
    w.cells.enter(c, Layer::Ground, id, false);
}

fn cells_of(w: &World, a: ObjectId) -> BTreeSet<(i32, i32)> {
    marks_of(w, a).iter().map(|&(x, y, _, _)| (x, y)).collect()
}

// spec: S-0611 claim 15 (an attached animation that is not bouncing: in our engine its point is the owner's position plus its displacement; an update that marks another standing position first frees the earlier one, whatever the switch; its end attempt places at that point) (#291)
#[test]
fn claim_15_attached_follows_its_owner() {
    for stale in [true, false] {
        let mut w = game(stale);
        let (car, a) = attached(&mut w);
        sw_common::step(&mut w);
        let first = at(2, 0);
        assert_eq!(
            cells_of(&w, a),
            BTreeSet::from([(first.x, first.y)]),
            "marked at the owner's position plus the displacement"
        );
        move_to(&mut w, car, at(0, 3));
        sw_common::step(&mut w);
        let next = at(2, 3);
        assert_eq!(
            cells_of(&w, a),
            BTreeSet::from([(next.x, next.y)]),
            "the earlier mark freed when the point moves (stale {stale})"
        );
        for _ in 0..60 {
            if !playing(&w, a) {
                break;
            }
            sw_common::step(&mut w);
        }
        let made = grown(&w);
        assert_eq!(made.len(), 1, "the end attempt makes the unit");
        assert_eq!(w.obj(made[0]).unwrap().pos.cell(), next);
        assert!(marks_of(&w, a).is_empty());
    }
}

// spec: S-0611 claim 15 (the owner's removal: no infantry; with the switch on nothing is freed, with it off the last mark is freed) and S-0109 claim 22 (an owner removed: deleted at its next update) (#291)
#[test]
fn claim_15_owner_removed() {
    for stale in [true, false] {
        let mut w = game(stale);
        let (car, a) = attached(&mut w);
        sw_common::run(&mut w, 2);
        let held = marks_of(&w, a);
        assert_eq!(held.len(), 1);
        sw_common::remove(&mut w, car);
        sw_common::step(&mut w);
        assert!(!playing(&w, a), "deleted at its next update");
        assert!(grown(&w).is_empty(), "no infantry");
        if stale {
            assert_eq!(marks_of(&w, a), held, "the mark stays taken");
        } else {
            assert!(marks_of(&w, a).is_empty(), "the last mark freed");
        }
    }
}

// spec: S-0611 claim 15 (a removal by another rule, here the end of loading's or a drain link's removal: no infantry; with the switch on nothing is freed, with it off the last mark is freed) (#291)
#[test]
fn claim_15_removed_by_another_rule() {
    for stale in [true, false] {
        let mut w = game(stale);
        let c = at(0, 0);
        let p = side(&w, c, 0);
        let a = anims::spawn(&mut w, "LONG", Some(p), Spawn::default())
            .id
            .expect("made");
        sw_common::run(&mut w, 2);
        assert!(matches!(
            ground_mark(&w, c, position::RIGHT),
            Some(Marker::Anim(x, _)) if x == a
        ));
        anims::remove(&mut w, a);
        sw_common::run(&mut w, 2);
        assert!(grown(&w).is_empty(), "no infantry");
        let kept = matches!(
            ground_mark(&w, c, position::RIGHT),
            Some(Marker::Anim(x, _)) if x == a
        );
        assert_eq!(kept, stale, "the mark stays taken only with the switch on");
    }
}

/// Plays an infantry-making animation 500 leptons above the terrain of a
/// cell that no high bridge marks, with a vehicle in the cell so that every
/// end attempt is refused (claims 10 and 11); returns whether its ground
/// mark was ever free at a tick's end once made.
fn high_refused(stale: bool, maker: bool) -> bool {
    let mut w = game(stale);
    let c = at(0, 0);
    sw_common::spawn(&mut w, "CART", BETA, 0, 0);
    let p = side(&w, c, 500);
    if maker {
        effects::anim(&mut w, "LONG", p);
    } else {
        anims::spawn(&mut w, "LONG", Some(p), Spawn::default());
    }
    let mut marked = false;
    let mut free = false;
    for _ in 0..60 {
        sw_common::step(&mut w);
        let held = matches!(ground_mark(&w, c, position::RIGHT), Some(Marker::Anim(..)));
        if held {
            marked = true;
        } else if marked {
            free = true;
        }
    }
    assert!(marked, "the animation marks its ground standing position");
    assert!(grown(&w).is_empty(), "every attempt is refused");
    free
}

// spec: S-0611 claim 15 (switch on: at a point at least 416 leptons above the terrain surface in a cell that no high bridge marks, the mark is made in the ground record but the end attempt's free in the deck record, so the ground mark stays; switch off: the free is made in the record the mark was made in) and claim 11 (a refused attempt retries) (#291)
#[test]
fn claim_15_high_free_misses_the_ground_mark() {
    for maker in [false, true] {
        assert!(
            !high_refused(true, maker),
            "switch on: the ground mark is never freed (maker {maker})"
        );
        assert!(
            high_refused(false, maker),
            "switch off: the refused attempt frees the ground mark (maker {maker})"
        );
    }
}

// spec: S-0611 claim 15 (a bouncing infantry-making animation survives a save and load mid-flight: the same marks, the same state hash) (#291)
#[test]
fn claim_15_save_load_mid_flight() {
    let mut w = game(true);
    let from = side(&w, at(-2, 0), 300);
    let a = fly(&mut w, "FLY", from, [40, 0, 0]);
    sw_common::run(&mut w, 3);
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    assert_eq!(l.anims, w.anims);
    let (seen_w, _) = fly_out(&mut w, a);
    let (seen_l, _) = fly_out(&mut l, a);
    assert_eq!(seen_w, seen_l);
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
}
