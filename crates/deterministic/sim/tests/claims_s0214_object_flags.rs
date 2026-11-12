//! S-0214 Import mapping (#208): the object-type flags `Immune` and
//! `LegalTarget` are read for every kind of target, a building and a
//! terrain object included. Synthetic data: the testkit's combat ruleset
//! (CART fires `Gun`, BUGGY is unarmed, TOWER is a plain building) and an
//! invented terrain type TREE.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::TerrainObjectType;
use glasnost_sim::objects;
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};

const SIZE: i32 = 32;
const ALPHA: HouseId = HouseId(0);

fn over(w: &World, actor: ObjectId, t: ObjectId, mods: Modifiers) -> Action {
    targeting::action(w, ALPHA, actor, ClickOn::Object(t), mods)
}

fn plain() -> Modifiers {
    Modifiers::default()
}

fn forced() -> Modifiers {
    Modifiers {
        force_fire: true,
        ..Default::default()
    }
}

/// The combat ruleset with a terrain type TREE.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .entry("terrain_objects".into())
        .or_default()
        .push("TREE".into());
    r.sim.terrain_objects.insert(
        "TREE".into(),
        TerrainObjectType {
            strength: Some(200),
            ..Default::default()
        },
    );
    r
}

/// An own CART, an enemy BUGGY, an enemy TOWER and a TREE.
fn world(r: &Ruleset) -> (World, [ObjectId; 4]) {
    let mut w = testkit::game_with(r, SIZE, 7);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, 0, 0), 0);
    let ev = spawn(&mut w, Kind::Vehicle, "BUGGY", 1, mid(SIZE, 3, 0), 0);
    let eb = spawn(&mut w, Kind::Building, "TOWER", 1, mid(SIZE, 0, 4), 0);
    let ty = w.rules().index_of(Kind::Terrain, "TREE").unwrap();
    let tree = objects::create(&mut w, Kind::Terrain, ty, None);
    objects::place(&mut w, tree, Pos::centre(mid(SIZE, -3, 0), 0), 0);
    (w, [cart, ev, eb, tree])
}

// spec: S-0214 table V row V6 and Import mapping (Object type Immune: immune target; absent no)
#[test]
fn row_v6_reads_immune_for_every_kind() {
    let (w, [cart, ev, eb, tree]) = world(&rules());
    assert_eq!(over(&w, cart, ev, plain()), Action::Attack, "baseline vehicle");
    assert_eq!(over(&w, cart, eb, plain()), Action::Attack, "baseline building");
    assert_eq!(over(&w, cart, tree, forced()), Action::Attack, "baseline tree");

    let mut r = rules();
    r.sim.vehicles.get_mut("BUGGY").unwrap().immune = Some(true);
    r.sim.buildings.get_mut("TOWER").unwrap().immune = Some(true);
    r.sim.terrain_objects.get_mut("TREE").unwrap().immune = Some(true);
    let (w, [cart, ev, eb, tree]) = world(&r);
    assert_eq!(over(&w, cart, ev, plain()), Action::NoMove, "immune vehicle");
    assert_eq!(over(&w, cart, eb, plain()), Action::NoMove, "immune building");
    assert_eq!(over(&w, cart, tree, forced()), Action::NoMove, "immune tree");
}

// spec: S-0214 claim 6 (a) and Import mapping (Object type LegalTarget: legal target; absent yes)
#[test]
fn claim_6a_reads_legal_target_for_a_building() {
    let mut r = rules();
    r.sim.vehicles.get_mut("BUGGY").unwrap().legal_target = Some(false);
    r.sim.buildings.get_mut("TOWER").unwrap().legal_target = Some(false);
    let (w, [cart, ev, eb, _]) = world(&r);
    assert_ne!(over(&w, cart, ev, plain()), Action::Attack, "vehicle");
    assert_ne!(over(&w, cart, eb, plain()), Action::Attack, "building");
    // Force-fire passes part (a) whatever the flag says.
    assert_eq!(over(&w, cart, eb, forced()), Action::Attack, "forced");
}
