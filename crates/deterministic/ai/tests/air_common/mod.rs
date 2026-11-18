//! Scenes for the own AI under air attack (#434, #443): the synthetic M7b
//! skirmish, where Alpha's DISC is a balloon-hover jumpjet vehicle with a
//! gun, and Beta (the AI house under test) has GUNR, a vehicle whose gun
//! cannot fire at the air, and FLAKT, the same vehicle with an anti-air
//! gun (test data of our own, as in `tests/antiair.rs`). The disc is put in
//! *sleep* and hovers where it is put, so it fires only the shots a test
//! gives it; Beta's brain thinks on every tick, and its orders are issued
//! as the game issues them.

#![allow(dead_code)]

use std::sync::Arc;

use glasnost_ai::brain::Brain;
use glasnost_ai::fixture;
use glasnost_ai::fixture_m7b as kit;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_rules::sim::{ProjectileType, WeaponType};
use glasnost_sim::combat::{fire, range, weapon};
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

/// The AI house under test.
pub const BETA: HouseId = HouseId(1);

/// The disc's hover height in leptons: at or above the airborne height of
/// S-0307 Terms (208).
pub const HOVER: i32 = 500; // nocite: a synthetic hover height

/// The M7b ruleset with FLAKT: Beta's vehicle with an anti-air gun FLAKW
/// (its projectile FLAKP hits the air and the ground).
pub fn ruleset() -> glasnost_rules::schema::Ruleset {
    let mut r = kit::ruleset();
    r.sim.projectiles.insert(
        "FLAKP".into(),
        ProjectileType {
            aa: Some(true),
            ag: Some(true),
            ..Default::default()
        },
    );
    let gun = r.sim.weapons["Gun"].clone();
    r.sim.weapons.insert(
        "FLAKW".into(),
        WeaponType {
            projectile: Some("FLAKP".into()),
            ..gun
        },
    );
    let mut flak = r.sim.vehicles["GUNR"].clone();
    flak.primary = Some("FLAKW".into());
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("FLAKT".into());
    r.sim.vehicles.insert("FLAKT".into(), flak);
    r
}

/// [`ruleset`]'s rules.
pub fn rules() -> Arc<SimRules> {
    Arc::new(SimRules::new(&ruleset(), kit::MAP_NAME))
}

/// The skirmish's world at Beta's first think once orders are taken.
pub fn world() -> World {
    world_with(rules())
}

/// [`world`] with the given rules.
pub fn world_with(rules: Arc<SimRules>) -> World {
    let mut g = Game::new(
        rules,
        &kit::map(),
        kit::MAP_NAME,
        fixture::catalogue(),
        kit::setup(5),
    )
    .expect("the M7b skirmish loads");
    while g.world.tick <= g.world.orders_blocked_through
        || !glasnost_ai::scripted::thinks_at(BETA, g.world.tick)
    {
        g.step();
    }
    g.world.clone()
}

/// The cell of house `h`'s first object on the map.
pub fn first_cell(w: &World, h: HouseId) -> CellXY {
    w.objects
        .values()
        .find(|o| o.owner == Some(h) && o.on_map)
        .map(|o| o.pos.cell())
        .unwrap()
}

/// Beta's home, as the brain finds it: its first object's cell.
pub fn home(w: &World) -> CellXY {
    first_cell(w, BETA)
}

/// Spawns a `name` of house `house` at (`dx`, `dy`) from Beta's home.
pub fn spawn(w: &mut World, name: &str, kind: Kind, house: u8, dx: i32, dy: i32) -> ObjectId {
    let h = home(w);
    let c = CellXY::new(h.x + dx, h.y + dy);
    assert!(w.cells.usable(c), "{name} is put on a usable cell, {c:?}");
    testkit::spawn(w, kind, name, house, c, 0)
}

/// Spawns Alpha's disc at (`dx`, `dy`) from Beta's home, in *sleep* and
/// hovering at [`HOVER`] (S-0502), and shows its area to Beta (a reveal, as
/// a map trigger gives), so that Beta sees it.
pub fn disc(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    let d = spawn(w, "DISC", Kind::Vehicle, 0, dx, dy);
    let (t, stale) = (w.tick, w.rules().quirks.stale_suspended_revert);
    w.obj_mut(d).unwrap().beh.force(Behaviour::Sleep, t, stale);
    jump::start_in_flight(w, d, Phase::Hovering, HOVER);
    let p = w.obj(d).unwrap().pos;
    glasnost_sim::econ::shroud::area_reveal(w, BETA, Pos::centre(p.cell(), 0), 10);
    testkit::run(w, 2);
    assert!(weapon::airborne(w, d), "the disc is in the air");
    d
}

/// One shot of the disc's gun at `x`, as a person sees it: the disc holds
/// `x` as its target and fires (S-0206 claim 9: the tick of the last shot).
pub fn shoot(w: &mut World, d: ObjectId, x: ObjectId) {
    w.obj_mut(d).unwrap().target = Some(Target::Object(x));
    assert!(fire::shot(w, d, 0, Target::Object(x)), "the disc fires");
}

/// Whether `x` is within the disc's gun's range (S-0307).
pub fn in_reach(w: &World, d: ObjectId, x: ObjectId) -> bool {
    let t = Some(Target::Object(x));
    let (_, wp) = weapon::would_use(w, d, t);
    wp.is_some() && range::in_range(w, d, wp, t, None)
}

/// Beta's orders to `id` in `out`.
pub fn orders_to(out: &[Order], id: ObjectId) -> Vec<Order> {
    out.iter()
        .filter(|o| o.actor() == Some(id))
        .cloned()
        .collect()
}

/// The destination cell of a move order.
pub fn move_cell(o: &Order) -> Option<CellXY> {
    match o {
        Order::Behaviour {
            behaviour: Behaviour::Move,
            dest: Some(Target::Cell(c)),
            ..
        } => Some(*c),
        _ => None,
    }
}

/// The squared distance between two cells.
pub fn dist2(a: CellXY, b: CellXY) -> i64 {
    let (dx, dy) = (i64::from(a.x - b.x), i64::from(a.y - b.y));
    dx * dx + dy * dy
}

/// What Beta's brain gave `x` over a run: per think, the tick and its
/// orders to `x`.
pub type Log = Vec<(i32, Vec<Order>)>;

/// Runs `ticks` ticks with Beta's brain thinking on every tick and its
/// orders issued; `each` runs before every tick (the disc's shots). Returns
/// Beta's orders to `x` at each of its thinks.
pub fn run(
    w: &mut World,
    br: &mut Brain,
    x: ObjectId,
    ticks: i32,
    mut each: impl FnMut(&mut World),
) -> Log {
    let mut log = Vec::new();
    for _ in 0..ticks {
        each(w);
        let thinks = glasnost_ai::scripted::thinks_at(BETA, w.tick);
        let out = br.think(w);
        if thinks {
            log.push((w.tick, orders_to(&out, x)));
        }
        for o in out {
            orders::issue(w, BETA, o);
        }
        testkit::run(w, 1);
    }
    log
}

/// Beta's brain for `w`.
pub fn brain(w: &World) -> Brain {
    Brain::new(w, &kit::map(), BETA)
}
