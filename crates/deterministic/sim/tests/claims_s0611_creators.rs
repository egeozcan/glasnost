//! S-0611 claims 8 to 11 for infantry-making animations that other creators
//! make (#291): a projectile trailer, a plain ordinary animation and a
//! reverse request. The rules apply to every infantry-making animation,
//! whoever created it (claim 9; S-0109 claim 16). Synthetic rules only, on
//! the `sw_common` superweapon ruleset: infantry GROWN (the made type),
//! animation MUTA (make-infantry index 0, 4 image frames), the
//! animation-to-infantry list `[GROWN]` and a neutral civilian-side house
//! Civ (house 2). Every name and number is made up.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::AnimationType;
use glasnost_sim::anims::{self, Spawn};
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

fn rules() -> Ruleset {
    use glasnost_rules::sim::Country;
    let mut r = sw_common::ruleset();
    r.sim.infantry.get_mut("WALKER").unwrap().primary = None;
    let grown = r.sim.infantry["WALKER"].clone();
    r.sim.infantry.insert("GROWN".into(), grown);
    reg(&mut r, "infantry", "GROWN");
    r.sim.animations.insert(
        "MUTA".into(),
        AnimationType {
            make_infantry: Some(0),
            frame_count: Some(4),
            ..Default::default()
        },
    );
    reg(&mut r, "animations", "MUTA");
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
    r
}

/// A game on [`rules`] with the neutral civilian-side house Civ (house 2).
fn game() -> World {
    let r = rules();
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

/// The right-hand standing position of cell `c`, at terrain height.
fn side(w: &World, c: CellXY) -> Pos {
    let (u, v) = glasnost_sim::walk::standing_point(position::RIGHT);
    let (x, y) = (256 * c.x + u, 256 * c.y + v);
    Pos::new(x, y, w.surface_at(x, y))
}

fn grown(w: &World) -> Vec<ObjectId> {
    let ti = w.rules().index_of(Kind::Infantry, "GROWN").unwrap();
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Infantry && o.type_index == ti && o.on_map)
        .map(|o| o.id)
        .collect()
}

fn mark(w: &World, c: CellXY, p: u8) -> Option<Marker> {
    w.cells
        .occ(c, Layer::Ground)
        .and_then(|o| o.marks[usize::from(p)])
        .map(|m| m.by)
}

/// The only ordinary animation in play.
fn only_anim(w: &World) -> ObjectId {
    let ids: Vec<ObjectId> = w.anims.anims.keys().copied().collect();
    assert_eq!(ids.len(), 1, "one ordinary animation: {ids:?}");
    ids[0]
}

/// Runs up to `n` ticks; returns the made units and whether the animation
/// `a` is still in play.
fn run_out(w: &mut World, a: ObjectId, n: i32) -> (Vec<ObjectId>, bool) {
    for _ in 0..n {
        sw_common::step(w);
    }
    (grown(w), w.anims.anims.contains_key(&a))
}

// Control: the warhead, death and launch path already makes the unit.
// spec: S-0611 claim 9 (an ownerless animation changes owner to the civilian house and the unit goes to it)
#[test]
fn claim_9_control_effects_anim_makes_its_unit() {
    let mut w = game();
    let p = side(&w, at(0, 0));
    effects::anim(&mut w, "MUTA", p);
    testkit::run(&mut w, 40);
    let made = grown(&w);
    assert_eq!(made.len(), 1);
    assert_eq!(w.obj(made[0]).unwrap().owner, Some(CIV));
}

// spec: S-0611 claim 9 (the rules apply to every infantry-making animation) and S-0109 claims 16 and 29 (a projectile trailer: an initial loop delay of 1 update, then it plays as any animation) (#291)
#[test]
fn claim_9_a_trailer_makes_its_unit() {
    let mut w = game();
    let c = at(0, 0);
    let p = side(&w, c);
    effects::trailer(&mut w, "MUTA", p);
    let a = only_anim(&w);
    let (made, playing) = run_out(&mut w, a, 40);
    assert_eq!(made.len(), 1, "a trailer of an infantry-making type makes its unit");
    assert!(!playing, "then it is removed");
    let o = w.obj(made[0]).unwrap();
    assert_eq!(o.owner, Some(CIV));
    assert_eq!(o.pos.cell(), c);
}

// spec: S-0611 claim 9 and S-0109 claim 16 (an animation made by anims::spawn of the same type makes the unit) (#291)
#[test]
fn claim_9_a_plain_ordinary_animation_makes_its_unit() {
    let mut w = game();
    let p = side(&w, at(0, 0));
    anims::spawn(&mut w, "MUTA", Some(p), Spawn::default());
    let a = only_anim(&w);
    let (made, playing) = run_out(&mut w, a, 40);
    assert_eq!(made.len(), 1);
    assert!(!playing);
}

// spec: S-0611 claim 9 (every infantry-making animation, a reverse request not excepted) (#291)
#[test]
fn claim_9_a_reverse_request_makes_its_unit() {
    let mut w = game();
    let p = side(&w, at(0, 0));
    effects::anim_reversible(&mut w, "MUTA", p, true);
    testkit::run(&mut w, 40);
    assert_eq!(grown(&w).len(), 1);
}

// spec: S-0611 claim 8 (while it plays, in each of its updates its standing position is marked taken; at its end the position is shown free again, and the made unit then marks its own) (#291)
#[test]
fn claim_8_a_trailer_marks_its_standing_position() {
    let mut w = game();
    let c = at(0, 0);
    let p = side(&w, c);
    effects::trailer(&mut w, "MUTA", p);
    let a = only_anim(&w);
    let mut marked = false;
    for _ in 0..40 {
        sw_common::step(&mut w);
        if !w.anims.anims.contains_key(&a) {
            break;
        }
        if matches!(mark(&w, c, position::RIGHT), Some(Marker::Anim(x, _)) if x == a) {
            marked = true;
        }
    }
    assert!(marked, "the animation marks its position while it plays");
    let made = grown(&w);
    assert_eq!(made.len(), 1);
    assert!(matches!(
        mark(&w, c, position::RIGHT),
        Some(Marker::Infantry(x, _)) if x == made[0]
    ));
}

// spec: S-0611 claim 11 (a refused placement: no unit, the animation stays and tries again; once the cell is clear the unit appears) (#291)
#[test]
fn claim_11_a_refused_trailer_stays_and_retries() {
    let mut w = game();
    let c = at(0, 0);
    let p = side(&w, c);
    effects::trailer(&mut w, "MUTA", p);
    let a = only_anim(&w);
    let car = sw_common::spawn(&mut w, "CART", BETA, 0, 0);
    let (made, playing) = run_out(&mut w, a, 30);
    assert!(made.is_empty(), "a vehicle in the cell refuses the placement");
    assert!(playing, "the animation stays");
    sw_common::remove(&mut w, car);
    let (made, playing) = run_out(&mut w, a, 30);
    assert_eq!(made.len(), 1);
    assert!(!playing);
}

// spec: S-0611 claims 8 to 11 (an ordinary infantry-making animation survives a save and load mid-play: the same unit at the same tick, the same state hash) (#291)
#[test]
fn save_load_mid_trailer() {
    let mut w = game();
    let p = side(&w, at(0, 0));
    effects::trailer(&mut w, "MUTA", p);
    sw_common::step(&mut w);
    sw_common::step(&mut w);
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    testkit::run(&mut w, 30);
    testkit::run(&mut l, 30);
    assert_eq!(grown(&w), grown(&l));
    assert_eq!(grown(&w).len(), 1);
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
}
