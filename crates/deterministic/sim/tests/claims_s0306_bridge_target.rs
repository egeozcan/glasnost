//! S-0306 Terms, *points of a cell target* (revision 89; render batch R9's
//! seam, sim batch S13): a cell target that is a high-bridge cell has its
//! target point at the deck height, 416 leptons above its terrain; its
//! reference point stays at the terrain height. Invented rules (a straight
//! projectile, gravity 10) on a flat synthetic map.

use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{ProjectileType, WarheadType, WeaponType};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::projectile::{launch, led_aim_point, turn};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

const N: i32 = 40;

/// The combat ruleset with gravity 10 and a weapon `Shot` (straight
/// projectile `Ball`, warhead `Dud`) on the cart.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.general.gravity = Some(10);
    r.sim.weapons.insert(
        "Shot".into(),
        WeaponType {
            damage: Some(50),
            rof: Some(100),
            range: Some(glasnost_rules::sim::Decimal("10".into())),
            speed: Some(50),
            projectile: Some("Ball".into()),
            warhead: Some("Dud".into()),
            ..Default::default()
        },
    );
    r.sim
        .projectiles
        .insert("Ball".into(), ProjectileType::default());
    r.sim.warheads.insert("Dud".into(), WarheadType::default());
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("Shot".into());
    r
}

fn cell_of(dx: i32, dy: i32) -> CellXY {
    testkit::mid(N, dx, dy)
}

/// A world with the cart of house 0 at the middle cell; cell `c` raised to
/// `level`, and a high-bridge cell when `deck`.
fn world(c: CellXY, level: i8, deck: bool) -> (World, ObjectId) {
    let mut w = testkit::game_with(&rules(), N, 7);
    let a = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, cell_of(0, 0), 0);
    w.cells.grid.get_mut(c).unwrap().level = level;
    if deck {
        let i = w.cells.idx(c).unwrap();
        w.cells.deck[i] = true;
    }
    (w, a)
}

/// The launch velocity and the stored target point K of a shot at cell
/// `c`, which is also the firer's target.
fn shot_at(c: CellXY, level: i8, deck: bool) -> ([F64; 3], Pos) {
    let (mut w, a) = world(c, level, deck);
    w.obj_mut(a).unwrap().target = Some(Target::Cell(c));
    let p = w.obj(a).unwrap().pos;
    let wp = w.rules().combat.weapon_index["Shot"];
    let id = launch(&mut w, a, wp, Target::Cell(c), p, 50).expect("a launch");
    let x = w.projectiles.get(&id).unwrap();
    (x.vel, x.stored)
}

// spec: S-0306 Terms (points of a cell target: a high-bridge cell's target point is its centre raised by the deck height of 416, whatever layer the firer stands on) with claim 1 (the shot speed towards the shot target's target point), claim 3 (a cell target's aim point is its target point) and claim 9 (K is the cell's target point)
#[test]
fn a_high_bridge_cell_is_aimed_at_its_deck_height() {
    let c = cell_of(5, 2);
    let (v, k) = shot_at(c, 0, true);
    // The control: a cell four levels up (4 × 104 = 416 leptons) with no
    // bridge, whose reference and target points are both at that height.
    let (cv, ck) = shot_at(c, 4, false);
    assert_eq!(ck, Pos::centre(c, 416), "the control's point");
    assert_eq!(k, ck, "K at the deck height");
    assert_eq!(v, cv, "the same launch velocity");
}

// spec: S-0306 Terms (points of a cell target: the target point of a high-bridge cell for the aim point of claim 3)
#[test]
fn the_aim_point_of_a_high_bridge_cell_is_at_its_deck_height() {
    let c = cell_of(5, 2);
    let (w, a) = world(c, 0, true);
    let terrain = Pos::centre(c, 0);
    assert_eq!(
        led_aim_point(&w, a, Target::Cell(c)),
        Pos::new(terrain.x, terrain.y, w.surface_at(terrain.x, terrain.y) + 416)
    );
}

// spec: S-0306 Terms (points of a cell target: the first row of claim 20 (c) moves the point to the cell's target point) with claim 20 (b) (the reached row snaps it to the reference point first); scenario P17 run c (an area-fire shot at a high-bridge cell ends at the deck height)
#[test]
fn a_shot_reaching_a_high_bridge_cell_detonates_at_its_deck_height() {
    let c = cell_of(5, 2);
    let (mut w, a) = world(c, 0, true);
    let p = w.obj(a).unwrap().pos;
    let wp = w.rules().combat.weapon_index["Shot"];
    let id = launch(&mut w, a, wp, Target::Cell(c), p, 50).expect("a launch");
    // Low over the cell's centre: the reached row (h < 208), no deck crossing.
    let at = Pos::centre(c, 100);
    {
        let x = w.projectiles.get_mut(&id).unwrap();
        x.pos = at;
        x.vel = [F64::ZERO; 3];
        x.prev_cell = at.cell();
        x.omega = 1_000_000;
    }
    w.effects.clear();
    turn(&mut w, id);
    assert!(!w.projectiles.contains_key(&id), "it detonated");
    let det = w.effects.iter().find_map(|e| match e {
        Effect::Detonation { pos, .. } => Some(*pos),
        _ => None,
    });
    assert_eq!(det, Some(Pos::centre(c, 416)), "at the deck height");
}
