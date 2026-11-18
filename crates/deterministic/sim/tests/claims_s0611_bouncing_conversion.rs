//! S-0611 claims 8, 9 and 15 for a bouncing conversion animation (#291,
//! S20's rest): the rules of claims 9 to 11 apply to every infantry-making
//! animation, the conversion animation included, and a bouncing one's point
//! is its position in play (claim 15). Synthetic rules on the superweapon
//! ruleset of `sw_common`, with houses Alpha (0) and Beta (1):
//! - infantry GROWN (the made type) and JJC (crashable, jumpjet movement
//!   without the jumpjet type flag, as `claims_s0611`'s);
//! - the warhead Mutate (infantry-death kind 9);
//! - the conversion animation types MUTF (bouncer flag, a thousand frames
//!   at one update a frame, make-infantry index 0: it lands long before its
//!   last pass ends) and MUTH (bouncer flag, four frames: its last pass ends
//!   in flight).
//!
//! Every name and number is made up.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, Decimal, Locomotor, WarheadType};
use glasnost_sim::cells::{Marker, position};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{Layer, ObjectId, Pos, World};
use glasnost_sim::{jump, objects};
use sw_common::{ALPHA, BETA, at};

/// MUTF's frame count: far more updates than its flight lasts.
const LONG: i32 = 1000;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn reg(r: &mut Ruleset, registry: &str, name: &str) {
    let l = r.registries.entry(registry.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The ruleset with conversion animation `conversion`.
fn rules(conversion: &str) -> Ruleset {
    let mut r = sw_common::ruleset();
    r.sim.infantry.get_mut("WALKER").unwrap().primary = None;
    r.sim.general.max_damage = Some(1_000_000);
    r.sim.warheads.insert(
        "Mutate".into(),
        WarheadType {
            inf_death: Some(9),
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    reg(&mut r, "warheads", "Mutate");
    let walker = r.sim.infantry["WALKER"].clone();
    r.sim.infantry.insert("GROWN".into(), walker.clone());
    reg(&mut r, "infantry", "GROWN");
    let mut jjc = walker;
    jjc.locomotor = Some(Locomotor::Jumpjet);
    jjc.jumpjet = Some(false);
    jjc.jumpjet_speed = Some(14);
    jjc.jumpjet_climb = Some(d("5.0"));
    jjc.jumpjet_crash = Some(d("6.0"));
    jjc.jumpjet_height = Some(500);
    jjc.jumpjet_accel = Some(d("2.0"));
    jjc.crashable = Some(true);
    r.sim.infantry.insert("JJC".into(), jjc);
    reg(&mut r, "infantry", "JJC");
    let bouncing = |n: i32| AnimationType {
        make_infantry: Some(0),
        frame_count: Some(n),
        rate: Some(900),
        bouncer: Some(true),
        ..Default::default()
    };
    for (n, frames) in [("MUTF", LONG), ("MUTH", 4)] {
        r.sim.animations.insert(n.into(), bouncing(frames));
        reg(&mut r, "animations", n);
    }
    let g = &mut r.sim.general.superweapons;
    g.mutate_warhead = Some("Mutate".into());
    g.infantry_mutate = Some(conversion.into());
    g.anim_to_infantry = Some(vec!["GROWN".into()]);
    r
}

/// Infantry `name` of Beta at the right-hand standing position of `c`,
/// `dz` above the terrain.
fn put(w: &mut World, name: &str, c: CellXY, dz: i32) -> ObjectId {
    let ti = w.rules().index_of(Kind::Infantry, name).unwrap();
    let id = objects::create(w, Kind::Infantry, ti, Some(BETA));
    let (u, v) = glasnost_sim::walk::standing_point(position::RIGHT);
    let (x, y) = (256 * c.x + u, 256 * c.y + v);
    let z = w.surface_at(x, y) + dz;
    objects::place(w, id, Pos::new(x, y, z), 0);
    id
}

/// A mutating hit of `amount` from Alpha, no attacker object.
fn mutate(w: &mut World, id: ObjectId, amount: i32, ignore_defences: bool) {
    let h = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "Mutate")
        .unwrap() as u32;
    damage::apply(
        w,
        id,
        Hit {
            amount,
            warhead: Some(h),
            distance: 0,
            attacker: None,
            house: Some(ALPHA),
            ignore_defences,
            no_escape: false,
        },
    );
}

/// The ordinary animations of type `name` in play that hold an
/// infantry-making state.
fn making(w: &World, name: &str) -> Vec<ObjectId> {
    let i = w.rules().anim_index(name.as_bytes()).unwrap();
    w.anims
        .anims
        .iter()
        .filter(|(_, a)| a.anim == Some(i) && a.making.is_some())
        .map(|(&id, _)| id)
        .collect()
}

fn grown(w: &World) -> Vec<ObjectId> {
    let ti = w.rules().index_of(Kind::Infantry, "GROWN").unwrap();
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Infantry && o.type_index == ti && o.on_map)
        .map(|o| o.id)
        .collect()
}

/// The animation `a` holds standing position `p` of cell `c`'s ground
/// record.
fn marked_by(w: &World, c: CellXY, p: u8, a: ObjectId) -> bool {
    w.cells
        .occ(c, Layer::Ground)
        .and_then(|o| o.marks[usize::from(p)])
        .is_some_and(|m| matches!(m.by, Marker::Anim(x, _) if x == a))
}

// spec: S-0611 claim 8 (a conversion animation at the victim's position, owned by the attacking house when there is no attacker object), claim 9 (claims 9 to 11 apply to every infantry-making animation, the conversion animation included) and claim 15 (a bouncing animation's point is its position in play; Landing: it leaves play in its landing turn, tries no creation and makes no infantry) (#291)
#[test]
fn issue_291_a_bouncing_conversion_animation_flies_and_a_landing_makes_none() {
    let mut w = sw_common::game(&rules("MUTF"));
    let v = put(&mut w, "WALKER", at(0, 0), 0);
    mutate(&mut w, v, 100_000, true);
    assert!(w.obj(v).is_none(), "the converted victim leaves play");
    let made = making(&w, "MUTF");
    assert_eq!(made.len(), 1, "one conversion animation, an ordinary one");
    let a = made[0];
    let anim = &w.anims.anims[&a];
    assert!(anim.flight.is_some(), "it flies");
    assert_eq!(
        anim.making.unwrap().owner,
        Some(ALPHA),
        "claim 8's owner, the attacking house"
    );
    assert!(w.mutation.makers.is_empty(), "no fixed infantry maker");
    let t0 = w.tick;
    while w.anims.anims.contains_key(&a) && w.tick < t0 + LONG + 50 {
        sw_common::step(&mut w);
        assert!(w.mutation.makers.is_empty());
    }
    assert!(!w.anims.anims.contains_key(&a), "it left play");
    assert!(
        w.tick - t0 < LONG,
        "it landed before its last pass ended: {} updates",
        w.tick - t0
    );
    assert!(grown(&w).is_empty(), "a landing makes no infantry");
}

// spec: S-0611 claim 7 (Q-1461: a victim that starts a crash keeps the conversion animation), claim 8 (it marks its standing position immediately at conversion; a victim retained by a crash does not clear it in the death tick) and claim 15 (a last pass that ends in flight: the end attempt at its point after that update's motion step; the unit is made by claims 9 and 10 for claim 8's owner) (#291)
#[test]
fn issue_291_a_crashed_victims_bouncing_conversion_animation_marks_at_once_and_makes_its_unit() {
    let mut w = sw_common::game(&rules("MUTH"));
    let c = at(0, 0);
    let v = put(&mut w, "JJC", c, 2000);
    // claims_s0611's MU13 fixture: 1 health left, so that the airborne mutating hit kills.
    w.obj_mut(v).unwrap().health = 1;
    mutate(&mut w, v, 100, false);
    assert!(jump::crashing(&w, v), "it crashes");
    let made = making(&w, "MUTH");
    assert_eq!(made.len(), 1, "one conversion animation, an ordinary one");
    let a = made[0];
    assert!(w.anims.anims[&a].flight.is_some(), "it flies");
    assert!(
        marked_by(&w, c, position::RIGHT, a),
        "its standing position is marked in the death tick"
    );
    let mut steps = 0;
    while w.anims.anims.contains_key(&a) && steps < 50 {
        sw_common::step(&mut w);
        steps += 1;
    }
    assert!(!w.anims.anims.contains_key(&a), "it ended");
    let units = grown(&w);
    assert_eq!(units.len(), 1, "its end attempt in flight makes the unit");
    let u = w.obj(units[0]).unwrap();
    assert_eq!(u.owner, Some(ALPHA), "for claim 8's owner");
}
