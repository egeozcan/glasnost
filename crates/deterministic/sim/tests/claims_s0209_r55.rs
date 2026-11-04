//! Claim tests of S-0209 revision 55 (M8 step A3 intake 88, Q-2396): an
//! armed walker meets an enemy building, a closed enemy gate or an enemy
//! vehicle that its weapon cannot fire at as *enemy obstacle* (claim 4,
//! rows WP13, WP24 and WP28), takes it by an override to *attack* with no
//! destination (claim 9), loses it at S-0309 claim 29's check and rests;
//! the switch `quirks.walker_attacks_unhittable_obstacle` (Edge cases); and
//! S-0215 claim 23's explosive-charges row that the charges case reads.
//! Synthetic data only: every type, weapon, warhead and number is invented.
//! The shapes follow scenario C15 parts e1 to e3 and e5 to e7. The ledger
//! is `docs/m8-claims/intake88.md`.

use std::sync::Arc;

use glasnost_map::{CellXY, Map};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{ArmourClass, Decimal, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::damage::Hit;
use glasnost_sim::occupancy::{self, Outcome};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{AttackMoveMemory, HouseId, Layer, ObjectId, Target, World};

const SIZE: i32 = 40;
const ALPHA: u8 = 0;
const BETA: u8 = 1;
/// The check period of S-0309 claim 29, written out for the expected ticks.
const PERIOD: i32 = 16;
/// The level of the impassable cells: no walker climbs four levels.
const CLIFF: i8 = 4;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

/// Eleven versus values, all `v`.
fn all(v: &str) -> Option<Vec<Decimal>> {
    Some(vec![dec(v); 11])
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn infantry(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut UnitType)) {
    let mut t = r.sim.infantry["WALKER"].clone();
    t.strength = Some(1000);
    t.primary = None;
    t.secondary = None;
    f(&mut t);
    register(r, "infantry", name);
    r.sim.infantry.insert(name.into(), t);
}

fn building(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut glasnost_rules::sim::BuildingType)) {
    let mut b = r.sim.buildings["TOWER"].clone();
    b.primary = None;
    b.secondary = None;
    b.turret = None;
    f(&mut b);
    register(r, "buildings", name);
    r.sim.buildings.insert(name.into(), b);
}

/// A direct weapon of range 3 cells with warhead `wh` and damage `damage`.
fn weapon(r: &mut Ruleset, name: &str, wh: &str, damage: i32) {
    r.sim.weapons.insert(
        name.into(),
        WeaponType {
            damage: Some(damage),
            rof: Some(30),
            range: Some(dec("3")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some(wh.into()),
            ..Default::default()
        },
    );
}

/// The rules of this file:
/// - warhead `NIL` (0 against every armour) with weapon `Nil` (damage 20),
///   warhead `FULL` (1 against every armour) with weapon `Full`, and weapon
///   `Blank` (warhead `FULL`, damage 0);
/// - infantry `W` (primary `Nil`: armed, its firing check answers *illegal*
///   against every object), `H` (primary `Full`), `C` (`W` with explosive
///   charges), `Z` (primary `Blank`: a primary weapon but not armed) and
///   `S` (only a secondary `Full`: armed with no primary weapon);
/// - buildings `B` (concrete, unarmed), `G` (a gate), `NOC` (`B` that does
///   not accept charges) and `GHOST` (`B` invisible in play); the vehicle
///   `BUGGY` of the testkit is unarmed.
fn ruleset(quirk: bool) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.globals.quirks.walker_attacks_unhittable_obstacle = quirk;
    for (n, v) in [("NIL", "0"), ("FULL", "1")] {
        r.sim.warheads.insert(
            n.into(),
            WarheadType {
                verses: all(v),
                ..Default::default()
            },
        );
    }
    weapon(&mut r, "Nil", "NIL", 20);
    weapon(&mut r, "Full", "FULL", 20);
    weapon(&mut r, "Blank", "FULL", 0);
    infantry(&mut r, "W", |t| t.primary = Some("Nil".into()));
    infantry(&mut r, "H", |t| t.primary = Some("Full".into()));
    infantry(&mut r, "C", |t| {
        t.primary = Some("Nil".into());
        t.c4 = Some(true);
    });
    infantry(&mut r, "Z", |t| t.primary = Some("Blank".into()));
    infantry(&mut r, "S", |t| t.secondary = Some("Full".into()));
    let concrete = |b: &mut glasnost_rules::sim::BuildingType| {
        b.strength = Some(300);
        b.combat.armor = Some(ArmourClass::Concrete);
    };
    building(&mut r, "B", concrete);
    building(&mut r, "G", |b| {
        concrete(b);
        b.gate = Some(true);
    });
    building(&mut r, "NOC", |b| {
        concrete(b);
        b.can_c4 = Some(false);
    });
    building(&mut r, "GHOST", |b| {
        concrete(b);
        b.invisible_in_game = Some(true);
    });
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// The cell in the gap, *X*.
fn x() -> CellXY {
    at(0, 0)
}

/// The cell in front of *X* on the walker's side.
fn front() -> CellXY {
    at(-1, 0)
}

/// The destination beyond the gap.
fn beyond() -> CellXY {
    at(4, 0)
}

/// The shape of the map: a box of impassable cells round x from −8 to 8
/// and y from −3 to `bottom`, cut in two by a wall at x = 0 whose only
/// gaps are *X* and, with `far`, the cell (0, `bottom`).
struct Shape {
    bottom: i32,
    far: bool,
}

/// C15's shape for parts e1, e2 and e5 to e7: the wall closes the box, *X*
/// is the only way across.
const CLOSED: Shape = Shape {
    bottom: 3,
    far: false,
};

fn map(shape: &Shape) -> Map {
    let mut m = testkit::empty_map(SIZE);
    let mut cliff = |c: CellXY| {
        let cell = m.cells.get_mut(c).expect("cell on the map");
        cell.level = CLIFF;
    };
    for dx in -9..=9 {
        cliff(at(dx, -4));
        cliff(at(dx, shape.bottom + 1));
    }
    for dy in -4..=shape.bottom + 1 {
        cliff(at(-9, dy));
        cliff(at(9, dy));
    }
    for dy in -3..=shape.bottom {
        if dy == 0 || (shape.far && dy == shape.bottom) {
            continue;
        }
        cliff(at(0, dy));
    }
    m
}

/// A game of `ruleset(quirk)` on `shape`, the orders open.
fn game(quirk: bool, shape: &Shape) -> World {
    let rules = Arc::new(SimRules::new(&ruleset(quirk), "synthetic-test"));
    let mut w =
        glasnost_sim::load::new_game(rules, &map(shape), testkit::catalogue(), &testkit::setup(1))
            .expect("synthetic test game loads");
    while w.tick < 4 {
        step(&mut w);
    }
    w
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, c: CellXY) -> ObjectId {
    testkit::spawn(w, kind, name, house, c, 0)
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn order(w: &mut World, id: ObjectId, dest: CellXY, attack_move: bool) {
    let o = Order::Behaviour {
        actor: id,
        behaviour: Behaviour::Move,
        attack_move,
        target: None,
        dest: Some(Target::Cell(dest)),
    };
    assert!(orders::issue(w, HouseId(ALPHA), o), "order refused");
}

/// The walker's entry outcome for cell `c`.
fn inf(w: &World, id: ObjectId, c: CellXY) -> Outcome {
    occupancy::infantry_outcome(w, id, c, occupancy::reference_level(w, id))
}

fn target(w: &World, id: ObjectId) -> Option<Target> {
    w.obj(id).and_then(|o| o.target)
}

/// Runs until `id` holds a target, at most `max` ticks; returns the tick
/// whose run gave it (the tick number less one when it shows).
fn run_to_take(w: &mut World, id: ObjectId, max: i32) -> Option<i32> {
    for _ in 0..max {
        step(w);
        if target(w, id).is_some() {
            return Some(w.tick - 1);
        }
    }
    None
}

/// What the run of C15's procedure records over `ticks` ticks after the
/// order (the take included): the ticks of each take and each loss, the
/// cells the walker stood in, and the tick of its last shot.
#[derive(Debug, Default)]
struct Record {
    takes: Vec<i32>,
    losses: Vec<i32>,
    entered_x: bool,
    rests: Vec<i32>,
    /// Each change of cell: the tick and the cell's offset from *X*.
    trail: Vec<(i32, i32, i32)>,
}

fn record(w: &mut World, id: ObjectId, ticks: i32) -> Record {
    let mut r = Record::default();
    let mut held = target(w, id).is_some();
    let mut resting = false;
    for _ in 0..ticks {
        step(w);
        let o = w.obj(id).expect("walker alive");
        let k = w.tick - 1;
        let c = o.pos.cell();
        // Listed in X's cell: a diagonal step past X's corner can put its
        // position's cell in X for a tick without entering it.
        if w.cells.list(x(), Layer::Ground).contains(&id) {
            r.entered_x = true;
        }
        let off = (k, c.x - x().x, c.y - x().y);
        if r.trail.last().is_none_or(|l| (l.1, l.2) != (off.1, off.2)) {
            r.trail.push(off);
        }
        match (held, o.target.is_some()) {
            (false, true) => r.takes.push(k),
            (true, false) => r.losses.push(k),
            _ => {}
        }
        held = o.target.is_some();
        let rest = matches!(o.beh.current, Some(Behaviour::Guard | Behaviour::AreaGuard))
            && o.attack_move.is_none()
            && o.dest.is_none();
        if rest && !resting {
            r.rests.push(k);
        }
        resting = rest;
    }
    r
}

/// The walker `W` at (−4, 0) with its order to `beyond()`, and the
/// occupant made by `occupant` in *X*; returns the world, the walker and
/// the occupant.
fn c15(
    quirk: bool,
    shape: &Shape,
    walker: &str,
    attack_move: bool,
    occupant: impl FnOnce(&mut World) -> ObjectId,
) -> (World, ObjectId, ObjectId) {
    let mut w = game(quirk, shape);
    let occ = occupant(&mut w);
    let id = put(&mut w, Kind::Infantry, walker, ALPHA, at(-4, 0));
    order(&mut w, id, beyond(), attack_move);
    (w, id, occ)
}

fn b_in_x(w: &mut World) -> ObjectId {
    put(w, Kind::Building, "B", BETA, x())
}

// ---------------------------------------------------------------------------
// Claim 4: rows WP13, WP24 and WP28 ask no firing check.
// ---------------------------------------------------------------------------

/// The walker's outcome for an occupant of each of the three rows, with
/// the switch in position `quirk`: a building, a closed gate and a
/// vehicle, each of house Beta, next to a walker of type `walker`.
fn three_rows(quirk: bool, walker: &str) -> [Outcome; 3] {
    let mut w = game(quirk, &CLOSED);
    let id = put(&mut w, Kind::Infantry, walker, ALPHA, at(-4, 0));
    put(&mut w, Kind::Building, "B", BETA, at(-5, 1));
    put(&mut w, Kind::Building, "G", BETA, at(-4, 1));
    put(&mut w, Kind::Vehicle, "BUGGY", BETA, at(-3, 1));
    step(&mut w);
    [at(-5, 1), at(-4, 1), at(-3, 1)].map(|c| inf(&w, id, c))
}

// spec: S-0209 claim 4 (Armed walkers and the firing check, Q-2396: rows WP24, WP13 and WP28 give enemy obstacle to an armed walker whose firing check answers illegal against the occupant) and Edge cases (quirks.walker_attacks_unhittable_obstacle on, the default)
#[test]
fn the_three_rows_give_enemy_obstacle_to_a_walker_that_cannot_fire_at_the_occupant() {
    assert!(glasnost_rules::quirks::Quirks::default().walker_attacks_unhittable_obstacle);
    assert_eq!(three_rows(true, "W"), [Outcome::EnemyObstacle; 3]);
}

// spec: S-0209 Edge cases (quirks.walker_attacks_unhittable_obstacle off: each of rows WP24, WP13 and WP28 gives blocked when the firing check without the range test, for the weapon the walker would use, answers illegal, as for an unarmed walker under row WP22)
#[test]
fn switch_off_the_three_rows_give_blocked_for_an_occupant_the_walker_cannot_fire_at() {
    assert_eq!(three_rows(false, "W"), [Outcome::Blocked; 3]);
}

// spec: S-0209 Edge cases (quirks.walker_attacks_unhittable_obstacle: only an illegal answer changes the rows; an occupant the walker can fire at gives enemy obstacle in both positions)
#[test]
fn an_occupant_the_walker_can_fire_at_gives_enemy_obstacle_in_both_positions() {
    for quirk in [true, false] {
        assert_eq!(
            three_rows(quirk, "H"),
            [Outcome::EnemyObstacle; 3],
            "switch {quirk}"
        );
    }
}

// spec: S-0209 claim 4 (r55: row WP13 tests the walker's primary weapon, WP24 and WP28 *armed* in Terms; a walker with a primary weapon of no damage is not armed, one with only a secondary weapon is armed with no primary)
#[test]
fn row_wp13_reads_the_primary_weapon_and_rows_wp24_and_wp28_read_armed() {
    // A primary weapon of no damage: a primary weapon, not armed (rows WP13 and WP22).
    assert_eq!(
        three_rows(true, "Z"),
        [Outcome::Blocked, Outcome::EnemyObstacle, Outcome::Blocked]
    );
    // Only a secondary weapon: armed, no primary weapon (rows WP24, WP14 and WP28).
    assert_eq!(
        three_rows(true, "S"),
        [
            Outcome::EnemyObstacle,
            Outcome::Blocked,
            Outcome::EnemyObstacle
        ]
    );
}

// ---------------------------------------------------------------------------
// Claim 9 and scenario C15: the take, the drop and the rest.
// ---------------------------------------------------------------------------

/// Checks the take of claim 9 at the tick it shows: the walker stands in
/// front of *X* and holds `occ` in *attack* with no destination, its *move*
/// suspended and the order's destination put aside.
fn assert_take(w: &World, id: ObjectId, occ: ObjectId) {
    let o = w.obj(id).unwrap();
    assert_eq!(o.pos.cell(), front(), "it stops in front of X");
    assert_eq!(o.target, Some(Target::Object(occ)));
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
    assert_eq!(o.dest, None, "the override gives no destination");
    assert!(o.path.steps.is_empty(), "it discards its path");
    assert_eq!(o.beh.suspended, Some(Behaviour::Move));
    assert_eq!(
        o.beh.aside,
        Some((None, Some(Target::Cell(beyond())))),
        "the order's destination is put aside"
    );
    assert!(!o.combat.opportunity, "the take is not a search");
}

/// C15 part e1 with occupant `occ` (also e6 and e7): the take, no shot, the
/// loss at the first multiple of 16 after it, the rest, and no take again.
fn e1(occupant: impl FnOnce(&mut World) -> ObjectId) {
    let (mut w, id, occ) = c15(true, &CLOSED, "W", false, occupant);
    let fired = w.obj(id).unwrap().last_fired;
    let t = run_to_take(&mut w, id, 300).expect("it takes the occupant");
    assert_take(&w, id, occ);
    let r = record(&mut w, id, 600);
    assert_eq!(r.losses.len(), 1, "{r:?}");
    let loss = r.losses[0];
    assert_eq!(loss.rem_euclid(PERIOD), 0, "lost at a check tick, {r:?}");
    assert!(
        loss > t && loss - t <= PERIOD,
        "the first check tick after the take {t}"
    );
    assert!(r.takes.is_empty(), "it takes no target again: {r:?}");
    assert!(!r.entered_x, "it never enters X");
    let o = w.obj(id).unwrap();
    assert_eq!(o.last_fired, fired, "it fires no shot");
    assert_eq!(o.pos.cell(), front(), "it rests in front of X");
    assert!(
        matches!(o.beh.current, Some(Behaviour::Guard | Behaviour::AreaGuard)),
        "it fell back to rest: {:?}",
        o.beh.current
    );
    // spec: S-0209 claim 9 (its suspended behaviour stays suspended, its destination still put aside)
    assert_eq!(o.beh.suspended, Some(Behaviour::Move));
    assert_eq!(o.dest, None);
}

// spec: S-0209 claim 9 (The take is an override; After the take, without attack-move memory: the drop of S-0309 claim 29 at a check tick, the fall-back to rest in front of the obstacle, no take again) and scenario C15 part e1
#[test]
fn e1_a_walker_takes_a_building_it_cannot_fire_at_loses_it_and_rests() {
    e1(b_in_x);
}

// spec: S-0209 claim 9 and scenario C15 part e6 (a closed enemy gate, row WP13)
#[test]
fn e6_the_same_rest_in_front_of_a_closed_enemy_gate() {
    e1(|w| {
        let g = put(w, Kind::Building, "G", BETA, x());
        assert!(
            !glasnost_sim::process::gate_open(w, g),
            "the gate is closed"
        );
        g
    });
}

// spec: S-0209 claim 9 and scenario C15 part e7 (an idle unarmed enemy vehicle, row WP28)
#[test]
fn e7_the_same_rest_in_front_of_an_enemy_vehicle() {
    e1(|w| put(w, Kind::Vehicle, "BUGGY", BETA, x()));
}

// spec: S-0209 claim 9 (With attack-move memory: each loss at a check tick; after each loss the walker either takes the occupant again without moving on past X or rests, and once it rests it stays at rest and takes no target again; the memory is removed at the first fall-back that finds it not engaged) and scenario C15 part e2
#[test]
fn e2_an_attack_moving_walker_takes_and_loses_the_building_until_it_rests() {
    let (mut w, id, b) = c15(true, &CLOSED, "W", true, b_in_x);
    run_to_take(&mut w, id, 300).expect("it takes B");
    assert_take(&w, id, b);
    assert_eq!(
        w.obj(id).unwrap().attack_move,
        Some(AttackMoveMemory::Dest(Target::Cell(beyond())))
    );
    let r = record(&mut w, id, 3000);
    assert!(!r.entered_x, "it never moves on past X: {r:?}");
    for l in &r.losses {
        assert_eq!(
            l.rem_euclid(PERIOD),
            0,
            "each loss at a multiple of 16: {r:?}"
        );
    }
    assert!(!r.rests.is_empty(), "the cycle ends at a rest: {r:?}");
    let rest = r.rests[0];
    assert_eq!(r.rests.len(), 1, "once it rests it stays at rest: {r:?}");
    assert!(
        r.takes.iter().all(|&k| k < rest),
        "no take after the rest: {r:?}"
    );
    assert_eq!(
        r.takes.len() + 1,
        r.losses.len(),
        "each take is lost: {r:?}"
    );
    let o = w.obj(id).unwrap();
    assert_eq!(o.attack_move, None, "the memory is removed");
    assert_eq!(o.pos.cell(), front());
}

// spec: S-0209 claim 9 (With attack-move memory: the take neither sets nor ends the engaged mark; a walker engaged at the fall-back resumes its attack move, which ends the mark, searches a path to the remembered destination again, meets the same cell and takes the occupant again; at the first fall-back that finds it not engaged its memory is removed and it rests)
#[test]
fn e2_an_engaged_walker_resumes_its_attack_move_and_takes_the_building_again() {
    let (mut w, id, b) = c15(true, &CLOSED, "W", true, b_in_x);
    run_to_take(&mut w, id, 300).expect("it takes B");
    assert_take(&w, id, b);
    // As if engaged already when it took B (our harness sets the mark;
    // C15 leaves this case `unmeasured`).
    w.obj_mut(id).unwrap().combat.engaged = true;
    let r = record(&mut w, id, 1200);
    assert!(!r.entered_x, "{r:?}");
    assert!(!r.losses.is_empty(), "{r:?}");
    let first = r.losses[0];
    assert_eq!(first.rem_euclid(PERIOD), 0, "{r:?}");
    assert!(
        r.takes.first().is_some_and(|&k| k > first),
        "it takes B again after the first loss: {r:?}"
    );
    assert!(
        r.trail.iter().all(|&(_, dx, _)| dx < 0),
        "never past X: {r:?}"
    );
    for l in &r.losses {
        assert_eq!(l.rem_euclid(PERIOD), 0, "each loss at a check tick: {r:?}");
    }
    let o = w.obj(id).unwrap();
    assert!(!o.combat.engaged, "the resumption ended the mark");
}

// spec: S-0209 claim 9 (the take is an override: the destruction of the target reverts it, S-0215 claim 10, so the walker is in move with its order's destination again and leaves the cell in front of B) and scenario C15 part e5
#[test]
fn e5_the_destruction_of_the_building_reverts_the_override() {
    let (mut w, id, b) = c15(true, &CLOSED, "W", false, b_in_x);
    let t = run_to_take(&mut w, id, 300).expect("it takes B");
    assert_take(&w, id, b);
    // Right after the tick of the take (`t`), so before the next check
    // tick, a killing hit destroys B: our harness runs its death path
    // directly, as `claims_s0306` does, in place of C15's attacker K.
    assert_eq!(w.tick, t + 1);
    w.obj_mut(b).unwrap().health = 0;
    glasnost_sim::combat::death::kill(
        &mut w,
        b,
        &Hit {
            amount: 10_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: Some(HouseId(ALPHA)),
            ignore_defences: false,
            no_escape: false,
        },
    );
    let o = w.obj(id).unwrap();
    assert_eq!(o.target, None, "B is dropped at the killing hit");
    assert_eq!(o.beh.current, Some(Behaviour::Move), "the revert");
    assert_eq!(o.dest, Some(Target::Cell(beyond())));
    let left = (0..300).find(|_| {
        step(&mut w);
        w.obj(id).unwrap().pos.cell() != front()
    });
    assert!(left.is_some(), "it leaves the cell in front of B");
}

// spec: S-0209 claim 4 (the path search gives the step into X the enemy obstacle cost of S-0212 claim 8, so a path runs through it when every detour costs more) and scenario C15 part e3 (a short way round: the walker goes round, in both positions of the switch; ours is 2 straight steps longer)
#[test]
fn e3_a_short_detour_is_taken_in_both_positions() {
    // The second gap is the cell next to X: the way round is 2 straight
    // steps longer. A gap farther off is not used: our walker starts inside
    // the usable area, where the corridor of S-0212 claim 14 can keep the
    // search to X's side of the wall, and C15 part e3 removes that by
    // starting the walker outside it (a seam of this test, ledger).
    let shape = Shape {
        bottom: 1,
        far: true,
    };
    for quirk in [true, false] {
        let (mut w, id, _) = c15(quirk, &shape, "W", false, b_in_x);
        let r = record(&mut w, id, 600);
        assert!(r.takes.is_empty(), "switch {quirk}: {r:?}");
        // Its path goes round through the second gap: it stands in that
        // cell on the way. (Its diagonal step into that gap crosses X's
        // corner between infantry positions, so its position is in X's cell
        // for some ticks: walking geometry of S-0208, not a step into X,
        // which would have given the take; a seam noted in the ledger.)
        assert!(
            r.trail.iter().any(|&(_, dx, dy)| (dx, dy) == (0, 1)),
            "switch {quirk}: {r:?}"
        );
        assert_eq!(w.obj(id).unwrap().pos.cell(), beyond(), "switch {quirk}");
    }
}

// spec: S-0209 Edge cases (switch on: a way round that costs more than the enemy obstacle step leaves the path through X and the take; switch off: the path search does not consider that step, S-0212 claim 8, and the walker routes round the occupant)
#[test]
fn a_long_detour_is_taken_only_with_the_switch_off() {
    let shape = Shape {
        bottom: 16,
        far: true,
    };
    let (mut w, id, b) = c15(true, &shape, "W", false, b_in_x);
    run_to_take(&mut w, id, 300).expect("switch on: it takes B");
    assert_take(&w, id, b);
    let (mut w, id, _) = c15(false, &shape, "W", false, b_in_x);
    let r = record(&mut w, id, 1500);
    assert!(r.takes.is_empty(), "switch off: no take: {r:?}");
    assert!(!r.entered_x);
    assert_eq!(w.obj(id).unwrap().pos.cell(), beyond(), "it went round");
}

// spec: S-0209 Edge cases (switch off: with no way round, the walker never takes the occupant this way; the path search does not consider the step into X)
#[test]
fn switch_off_a_walker_with_no_way_round_never_takes_the_occupant() {
    for occupant in [
        b_in_x as fn(&mut World) -> ObjectId,
        |w| put(w, Kind::Building, "G", BETA, x()),
        |w| put(w, Kind::Vehicle, "BUGGY", BETA, x()),
    ] {
        for attack_move in [false, true] {
            let (mut w, id, _) = c15(false, &CLOSED, "W", attack_move, occupant);
            let r = record(&mut w, id, 600);
            assert!(r.takes.is_empty(), "{r:?}");
            assert!(!r.entered_x);
        }
    }
}

// spec: S-0209 claim 9 (an occupant the walker can fire at: the same take, and it fires; the switch changes nothing here)
#[test]
fn a_walker_that_can_fire_at_the_building_takes_it_and_fires_in_both_positions() {
    for quirk in [true, false] {
        let (mut w, id, b) = c15(quirk, &CLOSED, "H", false, b_in_x);
        run_to_take(&mut w, id, 300).expect("it takes B");
        assert_take(&w, id, b);
        let fired = (0..200).any(|_| {
            step(&mut w);
            w.obj(id).unwrap().last_fired > 0
        });
        assert!(fired, "switch {quirk}: it fires at B");
    }
}

// ---------------------------------------------------------------------------
// The charges case and S-0215 claim 23's first infantry row.
// ---------------------------------------------------------------------------

/// Gives `id` target `x` in *attack* with its processing due now.
fn attack(w: &mut World, id: ObjectId, x: ObjectId) {
    let t = w.tick;
    let o = w.obj_mut(id).unwrap();
    o.beh.force(Behaviour::Attack, t, false);
    o.target = Some(Target::Object(x));
}

// spec: S-0215 claim 23 (Infantry, first row: explosive charges, its target a building that accepts charges and is not invisible: the target becomes its destination and sabotage is queued, none of the general rules)
#[test]
fn claim_23_a_charges_walker_in_attack_on_a_building_queues_sabotage() {
    let mut w = game(true, &CLOSED);
    let id = put(&mut w, Kind::Infantry, "C", ALPHA, at(-4, 0));
    let b = put(&mut w, Kind::Building, "B", BETA, at(-4, 2));
    attack(&mut w, id, b);
    step(&mut w);
    let o = w.obj(id).unwrap();
    assert_eq!(o.dest, Some(Target::Object(b)));
    assert!(
        o.beh.queued == Some(Behaviour::Sabotage) || o.beh.current == Some(Behaviour::Sabotage),
        "sabotage queued: {:?}",
        o.beh
    );
    assert_eq!(o.target, Some(Target::Object(b)));
}

// spec: S-0215 claim 23 (the first infantry row needs explosive charges and a building that accepts charges and is not invisible; otherwise the general rules apply)
#[test]
fn claim_23_the_charges_row_needs_charges_and_a_building_that_accepts_them() {
    for (walker, ty) in [("W", "B"), ("C", "NOC"), ("C", "GHOST")] {
        let mut w = game(true, &CLOSED);
        let id = put(&mut w, Kind::Infantry, walker, ALPHA, at(-4, 0));
        let b = put(&mut w, Kind::Building, ty, BETA, at(-4, 2));
        attack(&mut w, id, b);
        step(&mut w);
        let o = w.obj(id).unwrap();
        assert_ne!(o.beh.queued, Some(Behaviour::Sabotage), "{walker} on {ty}");
        assert_ne!(o.beh.current, Some(Behaviour::Sabotage), "{walker} on {ty}");
        assert_ne!(o.dest, Some(Target::Object(b)), "{walker} on {ty}");
    }
}

// spec: S-0209 claim 9 (Charges case: a walker with explosive charges that takes a building that accepts charges goes to sabotage when that behaviour is current in its turn of the next check tick, and keeps the building as its target; otherwise the check drops it as for any other walker)
#[test]
fn charges_case_the_take_leads_to_sabotage_or_to_the_drop() {
    let (mut w, id, b) = c15(true, &CLOSED, "C", false, b_in_x);
    let t = run_to_take(&mut w, id, 300).expect("it takes B");
    assert_take(&w, id, b);
    let check = t + PERIOD - t.rem_euclid(PERIOD);
    while w.tick <= check {
        step(&mut w);
    }
    let o = w.obj(id).unwrap();
    if o.beh.current == Some(Behaviour::Sabotage) {
        // spec: S-0309 claim 29 (no check while sabotage is current)
        assert_eq!(o.target, Some(Target::Object(b)), "sabotage keeps B");
        assert_eq!(o.dest, Some(Target::Object(b)));
    } else {
        assert_eq!(o.target, None, "the check drops B as for any walker");
    }
}

// spec: S-0209 Edge cases (switch off: rows WP24 gives blocked for a walker with charges too, which never reaches sabotage this way)
#[test]
fn switch_off_a_charges_walker_never_takes_the_building() {
    let (mut w, id, _) = c15(false, &CLOSED, "C", false, b_in_x);
    for _ in 0..600 {
        step(&mut w);
        let o = w.obj(id).unwrap();
        assert_eq!(o.target, None);
        assert_ne!(o.beh.seen(), Some(Behaviour::Sabotage));
    }
}
