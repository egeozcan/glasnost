//! S-0214 table GC, *attackable cell* (#212): a wall cell of a house not
//! allied with the actor, whose wall the warhead can damage, is attackable
//! without force-fire. Synthetic data: the testkit's combat ruleset (CART
//! fires `Gun`, whose warhead HE has the wall flag) and an invented wall
//! overlay RAMPART.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::OverlayType;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 32;
const ALPHA: HouseId = HouseId(0);

fn rules(f: impl FnOnce(&mut OverlayType)) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .entry("overlays".into())
        .or_default()
        .push("RAMPART".into());
    let mut o = OverlayType {
        has_art: Some(true),
        wall: Some(true),
        strength: Some(100),
        damage_levels: Some(3),
        armor: Some("steel".into()),
        ..Default::default()
    };
    f(&mut o);
    r.sim.overlays.insert("RAMPART".into(), o);
    r
}

/// An own CART and a RAMPART wall cell three cells away owned by `owner`.
fn world(r: &Ruleset, owner: Option<HouseId>) -> (World, ObjectId, CellXY) {
    let mut w = testkit::game_with(r, SIZE, 7);
    let ram = w
        .rules()
        .overlays
        .iter()
        .position(|o| o.name == "RAMPART")
        .unwrap() as u8;
    let tank = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, 0, 0), 0);
    let c = mid(SIZE, 3, 0);
    {
        let cell = w.cells.grid.get_mut(c).unwrap();
        cell.overlay = ram;
        cell.overlay_data = 0;
    }
    let i = w.cells.idx(c).unwrap();
    w.cells.wall_owner[i] = owner;
    (w, tank, c)
}

fn plain() -> Modifiers {
    Modifiers::default()
}

// spec: S-0214 table GC row GC4 and *Attackable cell* (ii) (the cell belongs to a house not allied with the actor and its overlay is a wall the warhead can damage: the wall flag); scenario T10 (line GC4: attack and an attack order on the cell over an enemy wall)
#[test]
fn an_enemy_wall_is_attackable_without_force_fire() {
    let r = rules(|_| {});
    let (w, tank, c) = world(&r, Some(HouseId(1)));
    let wp = glasnost_sim::combat::weapon::current_weapon(&w, tank).expect("armed");
    assert!(w.rules().combat.warhead_of(wp).expect("warhead").wall);
    assert_eq!(
        targeting::action(&w, ALPHA, tank, ClickOn::Cell(c), plain()),
        Action::Attack
    );
    let res = targeting::click(&w, ALPHA, &[tank], ClickOn::Cell(c), plain());
    assert_eq!(
        res.orders,
        vec![Order::Behaviour {
            actor: tank,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Cell(c)),
            dest: None,
        }]
    );
}

// spec: S-0214 *Attackable cell* (ii) (a wall of the actor's own house, or of no house, is not attackable); scenario T10 (line GC4: not attack over a wall of its own house)
#[test]
fn an_own_or_unowned_wall_is_not_attackable() {
    let r = rules(|_| {});
    for owner in [Some(ALPHA), None] {
        let (w, tank, c) = world(&r, owner);
        let a = targeting::action(&w, ALPHA, tank, ClickOn::Cell(c), plain());
        assert_ne!(a, Action::Attack, "owner {owner:?}");
    }
}
