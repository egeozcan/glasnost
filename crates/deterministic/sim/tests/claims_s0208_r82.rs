//! Claim tests of S-0208 r82 claims 19 and 27 (Q-3317, Q-3318): the point a
//! building gives a teleporting unit as its destination, played as scenario
//! W22 on an empty synthetic map with invented types. Every name and number
//! here is made up; no game data. Houses Alpha (0) and Beta (1) of the
//! testkit's sandbox game.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, UnitType};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::bunker;
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use glasnost_sim::{save, superweapons, testkit, tick};

const SIZE: i32 = 30;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn building(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut BuildingType)) {
    let mut t = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        ..Default::default()
    };
    // A build-up sprite, as the recycling buildings of `claims_s0603.rs`.
    t.econ.buildup = Some(true);
    t.econ.buildup_frames = Some(4);
    f(&mut t);
    register(r, "buildings", name);
    r.sim.buildings.insert(name.into(), t);
}

fn unit(r: &mut Ruleset, reg: &str, from: &str, name: &str, f: impl FnOnce(&mut UnitType)) {
    let base = if reg == "infantry" {
        &r.sim.infantry
    } else {
        &r.sim.vehicles
    };
    let mut t = base[from].clone();
    t.locomotor = Some(glasnost_rules::sim::Locomotor::Teleport);
    f(&mut t);
    register(r, reg, name);
    if reg == "infantry" {
        r.sim.infantry.insert(name.into(), t);
    } else {
        r.sim.vehicles.insert(name.into(), t);
    }
}

/// The six foundations of W22 part a.
const FOUNDATIONS: [(i32, i32); 6] = [(1, 1), (2, 2), (3, 2), (3, 3), (4, 4), (5, 3)];

fn grinder_name(w: i32, h: i32) -> String {
    format!("G{w}{h}")
}

/// The combat ruleset with:
/// - G11, G22, G32, G33, G44, G53: recycling buildings of those
///   foundations, with none of the helipad, unit-repair, unit-reload and
///   vehicle-bunker flags (W22 part a);
/// - RLD: G33 with the unit-reload flag and one docking place 256 leptons
///   east of the centre; RLDF: the same with the place 1024 east (part f);
/// - PIT: a 2 × 2 vehicle bunker with no impassable rows (part c);
/// - CAPB: a capturable 3 × 2 building, not recycling (part e);
/// - WARPER: an unarmed self-teleporting infantry type; ENGT: an engineer
///   with the teleporting locomotor; TELV: a BUGGY with the teleporting
///   locomotor; TELT: a turreted, armed CRUSHER (bunker-eligible, S-0606
///   claim 1) with the teleporting locomotor;
/// - the warp animation WARPX (S-0207 claim 20 e).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    for (fw, fh) in FOUNDATIONS {
        building(&mut r, &grinder_name(fw, fh), |t| {
            t.foundation = Some([fw, fh]);
            t.grinding = Some(true);
        });
    }
    for (name, dx) in [("RLD", 256), ("RLDF", 1024)] {
        building(&mut r, name, |t| {
            t.foundation = Some([3, 3]);
            t.grinding = Some(true);
            t.econ.unit_reload = Some(true);
            t.econ.number_of_docks = Some(1);
            t.econ.docking_offsets = Some(vec![[dx, 0, 0]]);
        });
    }
    building(&mut r, "PIT", |t| {
        t.bunker = Some(true);
        t.number_impassable_rows = Some(0);
    });
    building(&mut r, "CAPB", |t| {
        t.foundation = Some([3, 2]);
        t.capturable = Some(true);
    });
    unit(&mut r, "infantry", "WALKER", "WARPER", |t| {
        t.primary = None;
        t.teleporter = Some(true);
    });
    unit(&mut r, "infantry", "WALKER", "ENGT", |t| {
        t.primary = None;
        t.engineer = Some(true);
    });
    unit(&mut r, "vehicles", "BUGGY", "TELV", |t| t.primary = None);
    unit(&mut r, "vehicles", "CRUSHER", "TELT", |t| t.crusher = None);
    r.sim.general.warp_out = Some("WARPX".into());
    r
}

fn game() -> World {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    testkit::run(&mut w, 2);
    w
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

/// Building `name` of `house` with its position cell at (0, 0), settled.
fn place_b(w: &mut World, name: &str, house: u8) -> ObjectId {
    let b = put(w, Kind::Building, name, house, 0, 0);
    testkit::run(w, 3);
    b
}

/// The four starting cells of W22, each six cells from a W × H foundation
/// at (0, 0) on one side: west, east, north, south.
fn sides(fw: i32, fh: i32) -> [(i32, i32); 4] {
    [
        (-6, fh / 2),
        (fw - 1 + 6, fh / 2),
        (fw / 2, -6),
        (fw / 2, fh - 1 + 6),
    ]
}

/// The cell claim 27 gives for a building that gives its centre: the
/// foundation cell (⌊W / 2⌋, ⌊H / 2⌋) cells on from the position cell.
fn expected(fw: i32, fh: i32) -> CellXY {
    at(fw / 2, fh / 2)
}

fn on_foundation(c: CellXY, fw: i32, fh: i32) -> bool {
    let o = at(0, 0);
    (o.x..o.x + fw).contains(&c.x) && (o.y..o.y + fh).contains(&c.y)
}

fn ring(c: CellXY, e: CellXY) -> i32 {
    (c.x - e.x).abs().max((c.y - e.y).abs())
}

/// The first ring around `e` that has a cell outside the foundation.
fn first_outside_ring(e: CellXY, fw: i32, fh: i32) -> i32 {
    (1i32..)
        .find(|&k| {
            (-k..=k).any(|dx| {
                (-k..=k).any(|dy| {
                    (dx.abs().max(dy.abs()) == k)
                        && !on_foundation(CellXY::new(e.x + dx, e.y + dy), fw, fh)
                })
            })
        })
        .unwrap()
}

fn order(w: &mut World, u: ObjectId, behaviour: Behaviour, b: ObjectId) {
    let house = w.obj(u).unwrap().owner.unwrap();
    orders::issue(
        w,
        house,
        Order::Behaviour {
            actor: u,
            behaviour,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(b)),
        },
    );
}

/// Gives `u` the order and steps until its jump: the landing point is the
/// position of the last warp animation of the jump tick (W22 Observable:
/// the second one plays at the landing point, also for a unit consumed in
/// that tick).
fn jump(w: &mut World, u: ObjectId, behaviour: Behaviour, b: ObjectId) -> Pos {
    order(w, u, behaviour, b);
    for _ in 0..60 {
        w.effects.clear();
        tick::step(w);
        let warps: Vec<Pos> = w
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::Anim { name, pos, .. } if name == "WARPX" => Some(*pos),
                _ => None,
            })
            .collect();
        if let Some(&p) = warps.last() {
            assert_eq!(warps.len(), 2, "both warp animations in the jump tick");
            return p;
        }
    }
    panic!("no jump");
}

fn within(p: Pos) -> (i32, i32) {
    (p.x.rem_euclid(256), p.y.rem_euclid(256))
}

/// Steps until `u` no longer phases in (S-0207 claim 20 f).
fn until_phased_in(w: &mut World, u: ObjectId) {
    for _ in 0..300 {
        if !glasnost_sim::teleport::phasing_in(w, u) {
            return;
        }
        tick::step(w);
    }
    panic!("still phasing in");
}

fn round_trip(w: &World) {
    let bytes = save::save(w);
    let l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(w));
}

// spec: S-0208 r82 claim 27 and claim 19 (Q-3317, scenario W22 part a): a recycling building without a docking flag gives a teleporting infantry unit its centre; P's cell is the foundation cell (⌊W / 2⌋, ⌊H / 2⌋) cells on from the position cell, the larger-coordinate side of an even side, from every side; P's class is the centre, so the occupant-ignoring choice of *eaten* lands at that cell's centre position
#[test]
fn w22_a_an_infantry_unit_lands_in_the_cell_of_the_centre() {
    for (fw, fh) in FOUNDATIONS {
        for (dx, dy) in sides(fw, fh) {
            let what = format!("{fw}x{fh} from ({dx}, {dy})");
            let mut w = game();
            let b = place_b(&mut w, &grinder_name(fw, fh), 0);
            let u = put(&mut w, Kind::Infantry, "WARPER", 0, dx, dy);
            testkit::run(&mut w, 2);
            let l = jump(&mut w, u, Behaviour::Eaten, b);
            assert_eq!(l.cell(), expected(fw, fh), "{what}: the landing cell");
            assert_eq!(within(l), (128, 128), "{what}: the cell's centre");
        }
    }
}

// spec: S-0208 r82 claim 27 (Q-3317, scenario W22 part b): a teleporting vehicle's destination cell of S-0207 claim 20 (b) is the cell of the building's centre; its first recycle order lands it beside, in the first ring around that cell with a cell outside the foundation (the nearby-cell search is centred on it, S-0603 claim 16 e), and its second lands it at that cell's centre
#[test]
fn w22_b_a_vehicle_searches_round_and_lands_in_the_cell_of_the_centre() {
    for (fw, fh) in FOUNDATIONS {
        for (dx, dy) in sides(fw, fh) {
            let what = format!("{fw}x{fh} from ({dx}, {dy})");
            let mut w = game();
            let b = place_b(&mut w, &grinder_name(fw, fh), 0);
            let v = put(&mut w, Kind::Vehicle, "TELV", 0, dx, dy);
            testkit::run(&mut w, 2);
            let e = expected(fw, fh);
            let first = jump(&mut w, v, Behaviour::Eaten, b).cell();
            assert!(
                !on_foundation(first, fw, fh),
                "{what}: beside, at {first:?}"
            );
            assert_eq!(
                ring(first, e),
                first_outside_ring(e, fw, fh),
                "{what}: the first ring round {e:?} with an outside cell, at {first:?}"
            );
            until_phased_in(&mut w, v);
            let l = jump(&mut w, v, Behaviour::Eaten, b);
            assert_eq!(l.cell(), e, "{what}: the second order's landing cell");
            assert_eq!(within(l), (128, 128), "{what}: the cell's centre");
        }
    }
}

// spec: S-0208 r82 claim 27 (Q-3317, scenario W22 part c): a vehicle bunker gives a bunker-eligible teleporting vehicle S-0606 claim 4's quadrant point, taken from the vehicle's position when the point is fixed; it lands at the centre of the foundation cell that holds it
#[test]
fn w22_c_a_vehicle_bunker_lands_the_vehicle_in_the_quadrant_cell() {
    let mut cells = Vec::new();
    for (dx, dy) in sides(2, 2) {
        let what = format!("from ({dx}, {dy})");
        let mut w = game();
        let b = place_b(&mut w, "PIT", 0);
        let v = put(&mut w, Kind::Vehicle, "TELT", 0, dx, dy);
        testkit::run(&mut w, 2);
        assert!(bunker::eligible(&w, v), "{what}: bunker-eligible");
        let q = bunker::docking_point(&w, b, v).cell();
        assert!(on_foundation(q, 2, 2), "{what}: {q:?}");
        let l = jump(&mut w, v, Behaviour::Enter, b);
        assert_eq!(l.cell(), q, "{what}: the quadrant point's cell");
        assert_eq!(within(l), (128, 128), "{what}: the cell's centre");
        cells.push(q);
        if dx == -6 {
            round_trip(&w);
        }
    }
    cells.sort_by_key(|c| (c.x, c.y));
    cells.dedup();
    assert!(cells.len() > 1, "the sides give different cells: {cells:?}");
}

// spec: S-0208 r82 claim 19 (Q-3317, scenario W22 part d): an invulnerable recycling building refuses the entry (S-0209 row WP3), and the object destination's nearby-cell search is centred on the cell of the building's centre, so the unit lands in the first ring round that cell with a cell outside the foundation
#[test]
fn w22_d_the_search_round_an_invulnerable_building_is_centred_on_its_centre() {
    for (dx, dy) in sides(2, 2) {
        let what = format!("from ({dx}, {dy})");
        let mut w = game();
        let b = place_b(&mut w, "G22", 0);
        let u = put(&mut w, Kind::Infantry, "WARPER", 0, dx, dy);
        testkit::run(&mut w, 2);
        superweapons::protect::start(&mut w, b, superweapons::ProtectKind::Curtain, 2000);
        let e = expected(2, 2);
        let c = jump(&mut w, u, Behaviour::Eaten, b).cell();
        assert!(!on_foundation(c, 2, 2), "{what}: beside, at {c:?}");
        assert_eq!(ring(c, e), first_outside_ring(e, 2, 2), "{what}: at {c:?}");
    }
}

// spec: S-0208 r82 claims 19 and 27 (Q-3317, scenario W22 part e, the engineer): an enemy building an engineer captures gives it its centre; capture ignores occupants and row WP3 gives clear for the mission target, so the engineer lands at the centre position of the cell of the centre
#[test]
fn w22_e_an_engineer_lands_in_the_cell_of_the_centre() {
    for (dx, dy) in sides(3, 2) {
        let what = format!("from ({dx}, {dy})");
        let mut w = game();
        let b = place_b(&mut w, "CAPB", 1);
        let u = put(&mut w, Kind::Infantry, "ENGT", 0, dx, dy);
        testkit::run(&mut w, 2);
        let l = jump(&mut w, u, Behaviour::Capture, b);
        assert_eq!(l.cell(), expected(3, 2), "{what}: the landing cell");
        assert_eq!(within(l), (128, 128), "{what}: the centre position");
    }
}

// ---------------------------------------------------------------------------
// Claim 19 (Q-3318): the occupant-ignoring behaviours.

/// Gives infantry unit `u` destination `x` with `beh` current and fixes its
/// landing point at once (S-0208 claim 19). Returns the landing point and
/// whether the choice took exactly one ranged draw over 0 to 3 (true), took
/// none (false); any other count fails. With one draw, also returns the
/// side position table O's centre row gives for it.
fn landing_with(w: &mut World, u: ObjectId, beh: Behaviour, x: ObjectId) -> (Pos, Option<u8>) {
    use glasnost_sim::cells::position;
    let t = w.tick;
    let o = w.obj_mut(u).unwrap();
    o.beh.force(beh, t, true);
    o.dest = Some(Target::Object(x));
    let before = w.rng.clone();
    let mut probe = before.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let k = probe.ranged(0, 3, half);
    glasnost_sim::teleport::take_destination(w, u);
    let l = w
        .obj(u)
        .and_then(|o| o.teleport.as_ref())
        .and_then(|s| s.landing)
        .expect("a landing point");
    if w.rng == before {
        return (l, None);
    }
    assert_eq!(w.rng, probe, "one ranged draw");
    // spec: S-0208 table O (the centre row: R0 and R1 right first, R2 left, R3 bottom)
    let first = match k {
        0 | 1 => position::RIGHT,
        2 => position::LEFT,
        _ => position::BOTTOM,
    };
    (l, Some(first))
}

fn centre_position() -> (i32, i32) {
    glasnost_sim::walk::standing_point(glasnost_sim::cells::position::CENTRE)
}

/// A world with an infantry unit of type `name` at (0, 6) and, at (0, 0),
/// building `b` of Beta, or an Alpha BUGGY when `b` is None.
fn scene(name: &str, b: Option<&str>) -> (World, ObjectId, ObjectId) {
    let mut w = game();
    let x = match b {
        Some(b) => place_b(&mut w, b, 1),
        None => put(&mut w, Kind::Vehicle, "BUGGY", 0, 0, 0),
    };
    let u = put(&mut w, Kind::Infantry, name, 0, 0, 6);
    testkit::run(&mut w, 2);
    (w, u, x)
}

// spec: S-0208 r82 claim 19 (Q-3318, scenario W22 part e, the saboteur): sabotage is not among the occupant-ignoring behaviours, so a saboteur landing on the building it is to sabotage respects occupants; S-0209 row WP3 clears the cell of claim 27's point for its mission target, and claim 3's centre row gives a side position of that cell with its draw
#[test]
fn claim_19_a_saboteur_respects_occupants() {
    let (mut w, u, b) = scene("WARPER", Some("CAPB"));
    let (l, row) = landing_with(&mut w, u, Behaviour::Sabotage, b);
    assert_eq!(l.cell(), expected(3, 2), "the cell of the centre");
    let first = row.expect("claim 3's draw");
    assert_eq!(
        within(l),
        glasnost_sim::walk::standing_point(first),
        "the centre row's first side position"
    );
}

// spec: S-0208 r82 claim 19 (Q-3318: sabotage respects occupants also with a vehicle in C as destination; the vehicle-present cell refuses the position choice, claim 3's first row, and the nearby-cell search's second choice takes the centre row's draw)
#[test]
fn claim_19_a_saboteur_respects_a_vehicles_cell() {
    let (mut w, u, v) = scene("WARPER", None);
    let (l, row) = landing_with(&mut w, u, Behaviour::Sabotage, v);
    assert_ne!(l.cell(), at(0, 0), "refused in the vehicle's cell");
    assert!(
        ring(l.cell(), at(0, 0)) == 1,
        "a neighbour, at {:?}",
        l.cell()
    );
    assert!(row.is_some(), "the second choice draws");
}

// spec: S-0208 r82 claim 19 (Q-3318: patrol is among the occupant-ignoring behaviours, with the same destination test, the building occupying C; an engineer type's patrol has its destination as mission target, S-0208 claim 26, so row WP3 gives clear; claim 4's choice: the centre position, no draw)
#[test]
fn claim_19_patrol_ignores_occupants_for_a_building() {
    let (mut w, u, b) = scene("ENGT", Some("CAPB"));
    let (l, row) = landing_with(&mut w, u, Behaviour::Patrol, b);
    assert_eq!(l.cell(), expected(3, 2));
    assert_eq!(row, None, "no draw");
    assert_eq!(within(l), centre_position());
}

// spec: S-0208 r82 claim 19 (Q-3318: patrol with a vehicle in C as destination ignores occupants; for an engineer type row WP3 clears the vehicle's cell, and it lands at the centre position there without a draw)
#[test]
fn claim_19_patrol_ignores_occupants_for_a_vehicle() {
    let (mut w, u, v) = scene("ENGT", None);
    let (l, row) = landing_with(&mut w, u, Behaviour::Patrol, v);
    assert_eq!(l.cell(), at(0, 0));
    assert_eq!(row, None, "no draw");
    assert_eq!(within(l), centre_position());
}

// spec: S-0208 r82 claim 19 (Q-3318: the destination test is the same for every type; a patrolling unit without the engineer flag has no mission target, S-0208 claim 26, so the vehicle's cell refuses the entry, and the nearby-cell search's second choice keeps the occupant-ignoring condition: the centre position of a neighbour, no draw)
#[test]
fn claim_19_patrol_ignores_occupants_in_the_second_choice() {
    let (mut w, u, v) = scene("WARPER", None);
    let (l, row) = landing_with(&mut w, u, Behaviour::Patrol, v);
    assert_eq!(
        ring(l.cell(), at(0, 0)),
        1,
        "a neighbour, at {:?}",
        l.cell()
    );
    assert_eq!(row, None, "no draw");
    assert_eq!(within(l), centre_position());
}

// spec: S-0208 r82 claim 27 (Q-3317, scenario W22 part f): a building with the unit-reload flag and none of the helipad, unit-repair and vehicle-bunker flags gives its centre, not its docking place: an infantry unit and a vehicle's second order land in the centre's cell, not the docking place's
#[test]
fn w22_f_the_unit_reload_flag_alone_gives_the_centre() {
    let dock = at(2, 1);
    for (dx, dy) in sides(3, 3) {
        let what = format!("from ({dx}, {dy})");
        let mut w = game();
        let b = place_b(&mut w, "RLD", 0);
        let u = put(&mut w, Kind::Infantry, "WARPER", 0, dx, dy);
        testkit::run(&mut w, 2);
        let l = jump(&mut w, u, Behaviour::Eaten, b);
        assert_eq!(l.cell(), expected(3, 3), "{what}: infantry");
        assert_ne!(l.cell(), dock);

        let mut w = game();
        let b = place_b(&mut w, "RLD", 0);
        let v = put(&mut w, Kind::Vehicle, "TELV", 0, dx, dy);
        testkit::run(&mut w, 2);
        jump(&mut w, v, Behaviour::Eaten, b);
        until_phased_in(&mut w, v);
        let l = jump(&mut w, v, Behaviour::Eaten, b);
        assert_eq!(
            l.cell(),
            expected(3, 3),
            "{what}: the vehicle's second order"
        );
    }
}

// spec: S-0208 r82 claim 19 (Q-3317, scenario W22 part f, the far-place run): the nearby-cell search for a building with the unit-reload flag alone is centred on its centre's cell, not on its docking place four cells east, so the refused unit lands in the first ring round the centre's cell with a cell outside the foundation, not in the docking place's cell or next to it
#[test]
fn w22_f_far_place_the_search_is_centred_on_the_centre() {
    let dock = at(1 + 4, 1);
    for (dx, dy) in sides(3, 3) {
        let what = format!("from ({dx}, {dy})");
        let mut w = game();
        let b = place_b(&mut w, "RLDF", 0);
        let u = put(&mut w, Kind::Infantry, "WARPER", 0, dx, dy);
        testkit::run(&mut w, 2);
        superweapons::protect::start(&mut w, b, superweapons::ProtectKind::Curtain, 2000);
        let e = expected(3, 3);
        let c = jump(&mut w, u, Behaviour::Eaten, b).cell();
        assert!(!on_foundation(c, 3, 3), "{what}: beside, at {c:?}");
        assert_eq!(ring(c, e), first_outside_ring(e, 3, 3), "{what}: at {c:?}");
        assert!(ring(c, dock) > 1, "{what}: not at the docking place, {c:?}");
    }
}
