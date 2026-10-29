//! Claim tests of the rocking reports of S-0303 claim 20 and S-0614 claims
//! 9 and 10 on synthetic data (invented types, every number our own). The
//! simulation keeps no rocking state: it reports what rocks, takes no draw
//! for it, and the drawing keeps the angles.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, WarheadType};
use glasnost_sim::combat::area::{self, Detonation, DetonationResult};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::Kind;
use glasnost_sim::superweapons::{ProtectKind, protect};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, World};

const SIZE: i32 = 30;

fn ruleset(spread: &str) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.warheads.insert(
        "ROCK".into(),
        WarheadType {
            cell_spread: Some(Decimal(spread.into())),
            verses: Some(vec![Decimal("0".into()); 11]),
            rocker: Some(true),
            ..Default::default()
        },
    );
    r
}

fn world(spread: &str) -> World {
    testkit::game_with(&ruleset(spread), SIZE, 3)
}

fn put(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Vehicle, "CART", 1, testkit::mid(SIZE, dx, dy), 0)
}

fn warhead(w: &World, name: &str) -> Option<u32> {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == name)
        .map(|i| i as u32)
}

fn boom(w: &mut World, damage: i32, source: Option<ObjectId>, name: &str) -> DetonationResult {
    let point = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let warhead = warhead(w, name);
    w.effects.clear();
    area::detonate(
        w,
        Detonation {
            point,
            damage,
            warhead,
            source,
            house: None,
            resources: false,
        },
    )
}

fn rocks(w: &World) -> Vec<(i32, Vec<(ObjectId, Pos)>)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Rock { damage, pushed } => Some((*damage, pushed.clone())),
            _ => None,
        })
        .collect()
}

// spec: S-0303 claim 20 (rocking when min(D × 0.01, 4.0) exceeds 0.3, exactly when D ≥ 31; the warhead's rocking flag) and Constants
#[test]
fn claim_20_rocks_from_31_damage_with_the_flag() {
    let mut w = world("1");
    let cart = put(&mut w, 1, 0);
    boom(&mut w, 30, None, "ROCK");
    assert!(
        rocks(&w).is_empty(),
        "D = 30: strength 0.3 does not exceed 0.3"
    );
    boom(&mut w, 31, None, "ROCK");
    let r = rocks(&w);
    assert_eq!(r.len(), 1, "D = 31");
    assert_eq!(r[0].0, 31);
    assert_eq!(r[0].1.iter().map(|p| p.0).collect::<Vec<_>>(), [cart]);
    boom(&mut w, 500, None, "HE");
    assert!(rocks(&w).is_empty(), "a warhead without the rocking flag");
}

// spec: S-0303 claim 20 and Constants (the 7 × 7 cells around the centre cell: three cells each way, the fourth not)
#[test]
fn claim_20_reaches_the_seven_by_seven_cells() {
    let mut w = world("1");
    let near = [put(&mut w, 3, 0), put(&mut w, -3, 3), put(&mut w, 0, -3)];
    let far = [put(&mut w, 4, 0), put(&mut w, 0, -4), put(&mut w, 4, 4)];
    boom(&mut w, 100, None, "ROCK");
    let pushed: Vec<ObjectId> = rocks(&w)[0].1.iter().map(|p| p.0).collect();
    for id in near {
        assert!(pushed.contains(&id), "{id:?} within three cells");
    }
    for id in far {
        assert!(!pushed.contains(&id), "{id:?} four cells out");
    }
}

// spec: S-0303 claim 20 and claim 4 (rocking reaches the objects in the same cell layer: a detonation on a high bridge's deck, more than 208 above the terrain, rocks the deck's objects and not those on the ground below, and a detonation on the ground the reverse)
#[test]
fn claim_20_rocks_the_detonations_cell_layer() {
    use glasnost_sim::world::Layer;
    let mut w = world("1");
    let c = testkit::mid(SIZE, 0, 0);
    w.cells.mark_high_bridge(c, 237);
    assert!(w.high_bridge(c));
    let below = put(&mut w, 1, 0);
    let above = put(&mut w, 1, 1);
    let p = w.obj(above).unwrap().pos;
    w.cells.leave(p.cell(), Layer::Ground, above);
    w.obj_mut(above).unwrap().layer = Layer::Deck;
    glasnost_sim::objects::place(&mut w, above, Pos::new(p.x, p.y, p.z + 416), 0);
    assert!(w.cells.list(p.cell(), Layer::Deck).contains(&above));
    let (x, y) = (c.x * 256 + 128, c.y * 256 + 128);
    let ground = w.surface_at(x, y);
    // On the deck.
    w.effects.clear();
    let rock = warhead(&w, "ROCK");
    area::detonate(
        &mut w,
        Detonation {
            point: Pos::new(x, y, ground + 416),
            damage: 100,
            warhead: rock,
            source: None,
            house: None,
            resources: false,
        },
    );
    let pushed: Vec<ObjectId> = rocks(&w)[0].1.iter().map(|p| p.0).collect();
    assert_eq!(pushed, [above], "the deck's object only");
    // On the ground (at the surface: not more than 208 above it).
    boom(&mut w, 100, None, "ROCK");
    let pushed: Vec<ObjectId> = rocks(&w)[0].1.iter().map(|p| p.0).collect();
    assert_eq!(pushed, [below], "the ground's object only");
}

// spec: S-0303 claim 20 (the centre cell's objects away from the source object when there is one, the others away from P); design: Q-2081 question 5 (without a source, away from P)
#[test]
fn claim_20_pushes_the_centre_from_the_source_and_the_rest_from_p() {
    let mut w = world("1");
    let source = put(&mut w, -2, 0);
    let middle = put(&mut w, 0, 0);
    let side = put(&mut w, 2, 0);
    let p = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let from = w.obj(source).unwrap().pos;
    boom(&mut w, 100, Some(source), "ROCK");
    let pushed = rocks(&w)[0].1.clone();
    let of = |id| pushed.iter().find(|x| x.0 == id).map(|x| x.1);
    assert_eq!(of(middle), Some(from), "centre cell: from the source");
    assert_eq!(of(side), Some(p), "another cell: from P");
    assert_eq!(of(source), Some(p), "the source itself is pushed from P");
    boom(&mut w, 100, None, "ROCK");
    let pushed = rocks(&w)[0].1.clone();
    assert_eq!(
        pushed.iter().find(|x| x.0 == middle).map(|x| x.1),
        Some(p),
        "no source"
    );
}

// spec: S-0303 claim 20 (the other cells only when S > 0)
#[test]
fn claim_20_without_spread_rocks_only_the_centre_cell() {
    let mut w = world("0");
    let middle = put(&mut w, 0, 0);
    let _side = put(&mut w, 1, 0);
    boom(&mut w, 100, None, "ROCK");
    let pushed: Vec<ObjectId> = rocks(&w)[0].1.iter().map(|p| p.0).collect();
    assert_eq!(pushed, [middle]);
}

// spec: S-0303 claim 15 and State and ordering (a nullified detonation has no rocking)
#[test]
fn claim_20_a_nullified_detonation_does_not_rock() {
    let mut r = ruleset("0.25");
    r.sim.warheads.get_mut("ROCK").unwrap().verses = Some(vec![Decimal("1".into()); 11]);
    let mut w = testkit::game_with(&r, SIZE, 3);
    let cart = put(&mut w, 0, 0);
    protect::start(&mut w, cart, ProtectKind::Curtain, 100);
    let res = boom(&mut w, 100, None, "ROCK");
    assert_eq!(res, DetonationResult::Nullified);
    assert!(rocks(&w).is_empty());
}

// spec: S-0303 claim 20 (rocking takes no draw and is believed not to reach game state)
#[test]
fn claim_20_rocking_takes_no_draw_and_changes_no_state() {
    let run = |rocker: bool| {
        let mut r = ruleset("1");
        r.sim.warheads.get_mut("ROCK").unwrap().rocker = Some(rocker);
        let mut w = testkit::game_with(&r, SIZE, 3);
        put(&mut w, 0, 0);
        put(&mut w, 2, 1);
        boom(&mut w, 400, None, "ROCK");
        let rocked = !rocks(&w).is_empty();
        (rocked, w.rng.clone(), glasnost_sim::save::state_hash(&w))
    };
    let (with, rng_with, hash_with) = run(true);
    let (without, rng_without, hash_without) = run(false);
    assert!(with && !without);
    assert_eq!(rng_with, rng_without, "no draw");
    assert_eq!(hash_with, hash_without, "no game state");
}
