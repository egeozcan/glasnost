//! An airburst weapon whose airburst projectile leads back to the same
//! weapon cannot keep the turn pass from ending (#203). Synthetic data
//! only.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ProjectileType, WarheadType, WeaponType};
use glasnost_sim::combat::projectile::{launch, turn};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{Pos, Target, World};

const N: i32 = 40;

fn weapon(projectile: &str) -> WeaponType {
    WeaponType {
        damage: Some(1),
        rof: Some(100),
        range: Some(Decimal("10".into())),
        speed: Some(50),
        projectile: Some(projectile.into()),
        warhead: Some("Dud".into()),
        ..Default::default()
    }
}

fn airburst(weapon: &str) -> ProjectileType {
    ProjectileType {
        airburst: Some(true),
        airburst_weapon: Some(weapon.into()),
        ..Default::default()
    }
}

/// Weapons by name and projectile, and airburst projectiles by name and
/// airburst weapon; the projectile Plain is not airburst.
fn rules(weapons: &[(&str, &str)], projectiles: &[(&str, &str)]) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.general.gravity = Some(10);
    for &(n, p) in weapons {
        r.sim.weapons.insert(n.into(), weapon(p));
    }
    for &(n, a) in projectiles {
        r.sim.projectiles.insert(n.into(), airburst(a));
    }
    r.sim
        .projectiles
        .insert("Plain".into(), ProjectileType::default());
    r.sim.warheads.insert("Dud".into(), WarheadType::default());
    r
}

/// A shot of weapon `first` 100 leptons over its target cell, so that the
/// reached row detonates it in its next turn.
fn world(r: &Ruleset, first: &str) -> World {
    let mut w = testkit::game_with(r, N, 7);
    let a = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, testkit::mid(N, 0, 0), 0);
    let wp = w.rules().combat.weapon_index[first];
    let cell = testkit::mid(N, 5, 0);
    let from = w.obj(a).unwrap().pos;
    let from = Pos::new(from.x, from.y, from.z + 100);
    let id = launch(&mut w, a, wp, Target::Cell(cell), from, 1).expect("a launch");
    let over = Pos::centre(cell, 0);
    let z = w.surface_at(over.x, over.y) + 100;
    let p = w.projectiles.get_mut(&id).unwrap();
    p.pos = Pos::new(over.x, over.y, z);
    p.prev_cell = cell;
    w
}

/// The turn pass of the projectiles by hand, stopped after 20 000 turns.
/// Returns whether it ended and the objects made in it.
fn pass(w: &mut World) -> (bool, u32) {
    let first = w.next_id;
    w.update.begin_pass();
    let mut turns = 0;
    let mut ended = false;
    while turns < 20_000 {
        let Some(x) = w.update.next_turn() else {
            ended = true;
            break;
        };
        if w.projectiles.contains_key(&x) {
            turn(w, x);
        }
        turns += 1;
    }
    w.update.end_pass();
    (ended, w.next_id - first)
}

// spec: S-0306 claim 21 (an airburst projectile detonates once and launches nine projectiles of its airburst weapon): a chain without a loop is unchanged
#[test]
fn an_airburst_chain_without_a_loop_ends() {
    let r = rules(&[("Shot", "Ball"), ("Leaf", "Plain")], &[("Ball", "Leaf")]);
    let mut w = world(&r, "Shot");
    let (ended, made) = pass(&mut w);
    assert!(ended);
    assert!(made >= 9, "nine children: {made}");
}

// nocite: our own input validation (#203): a projectile launched through an airburst link that closes a loop of weapons launches no children through such a link, so one shot makes nine children at most
#[test]
fn an_airburst_weapon_that_names_itself_ends_its_pass() {
    let r = rules(&[("Shot", "Ball")], &[("Ball", "Shot")]);
    let mut w = world(&r, "Shot");
    let (ended, made) = pass(&mut w);
    assert!(
        ended,
        "the pass did not end: {} projectiles",
        w.projectiles.len()
    );
    assert!(made <= 9 + 9, "{made} objects made from one shot");
}

// nocite: our own input validation (#203): the same for a loop of two weapons
#[test]
fn an_airburst_loop_of_two_weapons_ends_its_pass() {
    let r = rules(
        &[("Shot", "Ball"), ("Echo", "Bell")],
        &[("Ball", "Echo"), ("Bell", "Shot")],
    );
    let mut w = world(&r, "Shot");
    let (ended, made) = pass(&mut w);
    assert!(
        ended,
        "the pass did not end: {} projectiles",
        w.projectiles.len()
    );
    assert!(made <= 9 + 9, "{made} objects made from one shot");
}
