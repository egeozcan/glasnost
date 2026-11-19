//! Claim tests of S-0502 r34 and S-0310 r13 (Q-3257, #445): a balloon-hover
//! vehicle over the object it attacks. S-0502 claim 16's point of an object
//! destination, claim 10's *Balloon-hover vehicle over its target*, scenario
//! J34's parts and engine tests, S-0310 claims 7 and 9 and the switch
//! `quirks.balloon_hover_mobile_fire_by_destination`, and the setup of
//! Q-3257's *Possible observation* (S-0307 RG13 with a 2 × 2 building and a
//! vertical weapon of reach 0.5 cell). Synthetic rules on the jumpjet
//! fixtures: every name and number is invented; no game data.

mod jump_common;

use std::sync::Arc;

use glasnost_num::geom::{Facing, delta, direction};
use glasnost_rules::sim::{BuildingType, ProjectileType, WeaponType};
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};
use jump_common::*;

/// Map east, the direction between two points with no horizontal
/// difference.
// spec: S-0204 F3 and S-0502 claim 10 (both differences 0: 16383 facing units with quirks.table_direction on)
const EAST: Facing = 16383;

/// The aiming tolerance of a projectile that does not home.
// spec: S-0310 claim 2 (τ = 2048)
const TAU: i32 = 2048;

/// J34's bound on the approach heading's distance from map east.
// spec: S-0502 J34 (b) (void unless the approach heading is more than 4096 facing units from 16383)
const APPROACH_OFF: i32 = 4096;

/// Ticks recorded after the first tick of the hover window.
// spec: S-0502 J34 (a) (record until 600 ticks after the first tick of the hover window)
const AFTER_WINDOW: i32 = 600;

/// The longest run before a hover window, so that a part with none ends.
const LONGEST: i32 = 2000; // nocite: a synthetic bound of the test

/// The weapon's rate-of-fire delay.
const ROF: i32 = 30; // nocite: synthetic test data

/// The variant of a part: the weapon's reach in cells, its fire-while-moving
/// flag, the type's mobile-fire flag and the two switches.
#[derive(Clone, Copy, Debug)]
struct Setup {
    range: &'static str,
    fire_while_moving: bool,
    mobile_fire: bool,
    mobile_switch: bool,
    hover_switch: bool,
}

/// J34's variant 1, with the weapon's range of 5 cells.
const V1: Setup = Setup {
    range: "5",
    fire_while_moving: false,
    mobile_fire: true,
    mobile_switch: true,
    hover_switch: true,
}; // nocite: synthetic test data (S-0502 J34 variant 1)

/// The jumpjet fixtures plus BOMBER, a balloon-hover jumpjet vehicle with no
/// turret, a body turn rate of 4 and the weapon BOMB (the vertical
/// projectile FALL, not homing, no minimum range, neither omni-fire nor, in
/// variants 1 and 3, fire-while-moving), and BIGBOX, a 2 × 2 building with no
/// weapon and none of the pad, repair-depot and vehicle-bunker flags, too
/// strong for any part to destroy.
fn world(s: Setup) -> World {
    let mut r = ruleset();
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("BOMBER".into());
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("BIGBOX".into());
    let sim = &mut r.sim;
    sim.projectiles.insert(
        "FALL".into(),
        ProjectileType {
            vertical: Some(true),
            ..Default::default()
        },
    );
    sim.weapons.insert(
        "BOMB".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(ROF),
            range: Some(d(s.range)),
            speed: Some(100),
            projectile: Some("FALL".into()),
            warhead: Some("SA".into()),
            fire_while_moving: Some(s.fire_while_moving),
            ..Default::default()
        },
    );
    let mut b = sim.vehicles["BALLOON"].clone();
    b.primary = Some("BOMB".into());
    b.rot = Some(4);
    b.turret = Some(false);
    b.mobile_fire = Some(s.mobile_fire);
    sim.vehicles.insert("BOMBER".into(), b);
    let mut big = BuildingType {
        strength: Some(100_000),
        foundation: Some([2, 2]),
        ..Default::default()
    };
    big.combat.armor = None;
    sim.buildings.insert("BIGBOX".into(), big);
    let mut w = testkit::game_with(&r, SIZE, 7);
    let mut rules = (*w.rules_arc()).clone();
    rules.quirks.balloon_hover_mobile_fire_by_destination = s.mobile_switch;
    rules.quirks.hovering_jumpjet_body_on_heading = s.hover_switch;
    w.rules = Some(Arc::new(rules));
    testkit::run(&mut w, 4);
    w
}

/// One recorded tick of the vehicle (J34's recorder).
#[derive(Clone, Copy, Debug)]
struct Row {
    tick: i32,
    phase: Phase,
    pos: Pos,
    heading: Facing,
    body: Facing,
    dest: Option<Target>,
    target: Option<Target>,
    fired: bool,
}

/// A recorded part.
struct Part {
    w: World,
    t: ObjectId,
    /// The target's position at the attack order.
    target_at: Pos,
    rows: Vec<Row>,
}

impl Part {
    /// The first row after the attack order in which it is not hovering.
    fn departure(&self) -> usize {
        self.rows
            .iter()
            .position(|r| r.phase != Phase::Hovering)
            .unwrap_or_else(|| panic!("void: no departure\n{}", self.trace()))
    }

    /// The rows of the hover window: the first unbroken run after the
    /// departure in which it hovers with its target.
    fn window(&self) -> std::ops::Range<usize> {
        let d = self.departure();
        let start = (d..self.rows.len())
            .find(|&i| in_window(&self.rows[i]))
            .unwrap_or_else(|| panic!("void: no hover window\n{}", self.trace()));
        let end = (start..self.rows.len())
            .find(|&i| !in_window(&self.rows[i]))
            .unwrap_or(self.rows.len());
        start..end
    }

    fn shots_before_window(&self) -> usize {
        self.rows[..self.window().start]
            .iter()
            .filter(|r| r.fired)
            .count()
    }

    fn shots_in_window(&self) -> Vec<i32> {
        self.rows[self.window()]
            .iter()
            .filter(|r| r.fired)
            .map(|r| r.tick)
            .collect()
    }

    /// The approach heading: the heading in the last tick before the window.
    fn approach(&self) -> Facing {
        self.rows[self.window().start - 1].heading
    }

    /// The kept facing: the heading in the last tick of the part.
    fn kept(&self) -> Facing {
        self.rows.last().unwrap().heading
    }

    fn trace(&self) -> String {
        self.rows
            .iter()
            .map(|r| format!("{r:?}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn in_window(r: &Row) -> bool {
    r.phase == Phase::Hovering && r.target.is_some()
}

fn row(w: &World, u: ObjectId, fired: bool) -> Row {
    let o = w.obj(u).unwrap();
    Row {
        tick: w.tick,
        phase: phase(w, u),
        pos: o.pos,
        heading: o.jump.as_ref().unwrap().heading.value(w.tick),
        body: o.body.value(w.tick),
        dest: o.dest,
        target: o.target,
        fired,
    }
}

/// A part: the target (BIGBOX, or the weaponless vehicle BUGGY standing
/// with no order) at the map's middle, owned by Beta, and BOMBER hovering
/// with no target over the centre of the cell `from` cells away, ordered to
/// attack it; recorded until 600 ticks after the first tick of the hover
/// window.
fn part(s: Setup, vehicle: bool, from: (i32, i32)) -> Part {
    let mut w = world(s);
    let t = if vehicle {
        put(&mut w, "BUGGY", Kind::Vehicle, 1, 0, 0)
    } else {
        put(&mut w, "BIGBOX", Kind::Building, 1, 0, 0)
    };
    let u = put(&mut w, "BOMBER", Kind::Vehicle, 0, from.0, from.1);
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    step(&mut w);
    // spec: S-0502 J34 (a) (void unless it hovers over that cell's centre with no target when the attack order is given)
    let c = Pos::centre(at(from.0, from.1), 0);
    let o = w.obj(u).unwrap();
    assert_eq!(phase(&w, u), Phase::Hovering, "set-up: hovering");
    assert_eq!((o.pos.x, o.pos.y), (c.x, c.y), "set-up: over the centre");
    assert!(o.target.is_none(), "set-up: no target");
    let target_at = pos(&w, t);
    orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: u,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(t)),
            dest: None,
        },
    );
    let mut rows = Vec::new();
    let mut last = w.obj(u).unwrap().last_fired;
    let mut start: Option<i32> = None;
    let mut departed = false;
    for _ in 0..(LONGEST + AFTER_WINDOW) {
        step(&mut w);
        let now = w.obj(u).unwrap().last_fired;
        let r = row(&w, u, now != last);
        last = now;
        departed |= r.phase != Phase::Hovering;
        if departed && start.is_none() && in_window(&r) {
            start = Some(r.tick);
        }
        rows.push(r);
        match start {
            Some(s0) if w.tick >= s0 + AFTER_WINDOW => break,
            None if rows.len() as i32 >= LONGEST => break,
            _ => {}
        }
    }
    Part {
        w,
        t,
        target_at,
        rows,
    }
}

/// The expectations of J34 (a) for the hover position, the destination,
/// the body facing and the movement heading in the hover window.
fn hover_expectations(p: &Part, point: (i32, i32)) {
    let win = p.window();
    for r in &p.rows[win.clone()] {
        // spec: S-0502 claim 16 (r34: the movement point is the target's point at the assignment, kept exactly by a balloon-hover vehicle) and J34 (a) (in every tick of the hover window its horizontal position is exactly that point)
        assert_eq!(
            (r.pos.x, r.pos.y),
            point,
            "hover position at tick {}\n{}",
            r.tick,
            p.trace()
        );
        // spec: S-0307 claim 18 (the target is its destination, and it keeps it)
        assert_eq!(r.dest, Some(Target::Object(p.t)), "destination");
    }
    let mut settled = false;
    for i in (win.start + 1)..win.end {
        let (a, b) = (p.rows[i - 1], p.rows[i]);
        // spec: S-0502 claim 10 (from the second tick of the hover window on, its body facing equals its movement heading) and J34 (a)
        assert_eq!(b.body, b.heading, "body on the heading at tick {}", b.tick);
        // spec: S-0502 claims 10 and 13 (over the point nothing turns the heading again: once it has stayed the same from one tick to the next, it does not change)
        if settled {
            assert_eq!(b.heading, a.heading, "heading turned at tick {}", b.tick);
        }
        settled |= b.heading == a.heading;
    }
}

/// The building's reference point: its position plus (128, 128) leptons.
fn reference(p: &Part) -> (i32, i32) {
    let r = p.w.reference_point(p.t);
    // spec: S-0204 claim 4 and S-0502 J34 (the building's reference point is its position plus (128, 128) leptons, its position the centre of its top-left cell)
    assert_eq!(
        (r.x, r.y),
        (p.target_at.x + 128, p.target_at.y + 128),
        "set-up: the 2 × 2 building's reference point"
    );
    (r.x, r.y)
}

fn off_east(f: Facing) -> i32 {
    delta(f, EAST).abs()
}

// spec: S-0502 claims 10 and 16 (r34) and J34 (a): variant 1 from the west hovers over the building's reference point, holds its fire on the way in (S-0310 claim 7: a balloon-hover vehicle gets moving while it is moving now) and fires from the hover, its kept facing within the tolerance of map east
#[test]
fn j34_a_from_the_west_it_fires_from_over_the_reference_point() {
    let p = part(V1, false, (-10, 0));
    let point = reference(&p);
    assert!(off_east(p.kept()) <= TAU, "void: kept facing {}", p.kept());
    hover_expectations(&p, point);
    assert_eq!(p.shots_before_window(), 0, "no shot on the way in");
    let shots = p.shots_in_window();
    // spec: S-0502 J34 (a) (in the hover window it fires at the weapon's rate-of-fire delay)
    assert!(
        shots.len() as i32 >= AFTER_WINDOW / ROF / 2,
        "shots {shots:?}\n{}",
        p.trace()
    );
}

// spec: S-0502 claims 10 and 16 (r34) and J34 (b) to (d): variant 1 from the east, north and south hovers over the same point with its body on the heading and never fires: the target straight below lies map east, outside the tolerance of the kept facing
#[test]
fn j34_b_to_d_from_the_other_sides_it_hovers_there_and_holds_its_fire() {
    for from in [(11, 0), (0, -10), (0, 11)] {
        let p = part(V1, false, from);
        let point = reference(&p);
        assert!(
            off_east(p.approach()) > APPROACH_OFF && off_east(p.kept()) > TAU,
            "void from {from:?}: approach {} kept {}",
            p.approach(),
            p.kept()
        );
        hover_expectations(&p, point);
        assert!(
            !p.rows.iter().any(|r| r.fired),
            "no shot from {from:?}\n{}",
            p.trace()
        );
    }
}

// spec: S-0502 J34 (e) and S-0310 claim 7 (variant 2, the fire-while-moving flag: on the way in its heading, and with it its body, turns towards the target, and it fires while the building is in range; in the hover window it does not fire)
#[test]
fn j34_e_with_fire_while_moving_it_fires_on_the_way_in_only() {
    let v2 = Setup {
        fire_while_moving: true,
        ..V1
    };
    let p = part(v2, false, (11, 0));
    let point = reference(&p);
    assert!(
        off_east(p.approach()) > APPROACH_OFF && off_east(p.kept()) > TAU,
        "void: approach {} kept {}",
        p.approach(),
        p.kept()
    );
    hover_expectations(&p, point);
    let win = p.window();
    for r in p.rows[..win.start].iter().filter(|r| r.fired) {
        let d = direction(
            i64::from(point.0 - r.pos.x),
            i64::from(point.1 - r.pos.y),
            true,
        );
        // spec: S-0310 claims 1 and 2 (a shot only while its body lies within τ of the direction of the target's point)
        assert!(delta(r.body, d).abs() <= TAU, "shot at tick {}", r.tick);
    }
    assert!(
        p.shots_before_window() >= 1,
        "it fires on the way in\n{}",
        p.trace()
    );
    assert!(p.shots_in_window().is_empty(), "no shot from the hover");
}

// spec: S-0310 claim 7 (r13: a balloon-hover type without the mobile-fire flag gets moving while its target is its destination, also while it hovers) and Edge cases (quirks.balloon_hover_mobile_fire_by_destination on); S-0502 J34 (f): variant 3 hovers over the point of (a) and never fires
#[test]
fn j34_f_without_mobile_fire_it_never_fires() {
    let v3 = Setup {
        mobile_fire: false,
        ..V1
    };
    let p = part(v3, false, (-10, 0));
    let point = reference(&p);
    hover_expectations(&p, point);
    assert!(
        !p.rows.iter().any(|r| r.fired),
        "no shot in the run\n{}",
        p.trace()
    );
}

// spec: S-0310 Edge cases (quirks.balloon_hover_mobile_fire_by_destination off: for a balloon-hover type the first condition reads its locomotor moving it, for a jumpjet moving now, so it gets no moving from it while it hovers) and S-0502 J34 engine tests ((f) must fire from the hover as (a) does)
#[test]
fn switch_off_j34_f_fires_from_the_hover_as_a_does() {
    let v3 = Setup {
        mobile_fire: false,
        mobile_switch: false,
        ..V1
    };
    let p = part(v3, false, (-10, 0));
    let point = reference(&p);
    hover_expectations(&p, point);
    assert_eq!(p.shots_before_window(), 0, "no shot on the way in");
    assert!(
        !p.shots_in_window().is_empty(),
        "it fires from the hover\n{}",
        p.trace()
    );
}

// spec: S-0310 Edge cases (every vehicle other than a balloon-hover type is unchanged by quirks.balloon_hover_mobile_fire_by_destination); S-0310 claim 7 (a type with the mobile-fire flag gets no moving from the first condition)
#[test]
fn the_mobile_fire_switch_leaves_a_type_with_the_flag_alone() {
    let trace = |on: bool| {
        let s = Setup {
            mobile_switch: on,
            ..V1
        };
        let p = part(s, false, (-10, 0));
        p.rows
            .iter()
            .map(|r| (r.pos, r.heading, r.body, r.fired))
            .collect::<Vec<_>>()
    };
    assert_eq!(trace(true), trace(false));
}

// spec: S-0502 claim 16 (r34: a vehicle with no next movement point gives its position) and J34 (g): from the east, over a weaponless vehicle standing with no order, it hovers exactly over that vehicle's position with its body on the heading and never fires
#[test]
fn j34_g_over_a_vehicle_it_hovers_at_its_position() {
    let p = part(V1, true, (11, 0));
    assert!(
        off_east(p.approach()) > APPROACH_OFF && off_east(p.kept()) > TAU,
        "void: approach {} kept {}",
        p.approach(),
        p.kept()
    );
    assert_eq!(pos(&p.w, p.t), p.target_at, "void: the vehicle moved");
    hover_expectations(&p, (p.target_at.x, p.target_at.y));
    assert!(!p.rows.iter().any(|r| r.fired), "no shot\n{}", p.trace());
}

// spec: S-0502 Edge cases (quirks.hovering_jumpjet_body_on_heading off) and S-0310 claim 9 (r13: a hovering jumpjet vehicle counts neither as moved by its locomotor nor as having a destination, so it takes the row with no destination) and S-0502 J34 engine tests ((b) must turn its body to 16383 in the hover window and then fire)
#[test]
fn switch_off_j34_b_turns_its_body_to_map_east_and_fires() {
    let s = Setup {
        hover_switch: false,
        ..V1
    };
    let p = part(s, false, (11, 0));
    let point = reference(&p);
    let win = p.window();
    for r in &p.rows[win.clone()] {
        assert_eq!((r.pos.x, r.pos.y), point, "hover position");
    }
    let turned = p.rows[win.clone()]
        .iter()
        .position(|r| r.body == EAST)
        .unwrap_or_else(|| panic!("the body turns to map east\n{}", p.trace()));
    let fired = p.rows[win.clone()]
        .iter()
        .position(|r| r.fired)
        .unwrap_or_else(|| panic!("it fires from the hover\n{}", p.trace()));
    assert!(fired >= turned, "it fires once the body has turned");
    assert_eq!(p.shots_before_window(), 0, "no shot on the way in");
    assert!(
        off_east(p.kept()) > TAU,
        "the movement heading is not turned to map east"
    );
}

/// Q-3257's *Possible observation*: the weapon reaches half a cell.
const HALF: Setup = Setup { range: "0.5", ..V1 }; // nocite: synthetic test data (Q-3257, a vertical weapon of reach 0.5 cell)

// spec: S-0502 claims 10 and 16 (r34) (Q-3257's observation, S-0307 RG13 with a 2 × 2 building and a vertical weapon of reach 0.5 cell: from the west it hovers over the reference point and fires; from the east it hovers there and does not)
#[test]
fn q3257_half_a_cell_from_the_west_fires_and_from_the_east_does_not() {
    let west = part(HALF, false, (-10, 0));
    let point = reference(&west);
    hover_expectations(&west, point);
    assert!(
        !west.shots_in_window().is_empty(),
        "from the west it fires\n{}",
        west.trace()
    );
    let east = part(HALF, false, (11, 0));
    hover_expectations(&east, reference(&east));
    assert!(
        !east.rows.iter().any(|r| r.fired),
        "from the east it does not\n{}",
        east.trace()
    );
}

// spec: S-0310 claim 7 (r13) and Edge cases (quirks.balloon_hover_mobile_fire_by_destination, both positions) (Q-3257's observation with the mobile-fire flag off: on, it never fires; off, it fires from the hover as with the flag)
#[test]
fn q3257_half_a_cell_without_mobile_fire_under_both_settings() {
    for on in [true, false] {
        let s = Setup {
            mobile_fire: false,
            mobile_switch: on,
            ..HALF
        };
        let p = part(s, false, (-10, 0));
        hover_expectations(&p, reference(&p));
        let fired = !p.shots_in_window().is_empty();
        assert_eq!(fired, !on, "switch {on}\n{}", p.trace());
    }
}

// spec: S-0502 claim 16 (r34: the point is the object's point at the moment of the assignment, and it does not follow the object afterwards)
#[test]
fn the_point_does_not_follow_the_object() {
    let mut w = world(V1);
    let t = put(&mut w, "BUGGY", Kind::Vehicle, 1, 0, 0);
    let u = put(&mut w, "BOMBER", Kind::Vehicle, 0, -6, 0);
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    let was = pos(&w, t);
    glasnost_sim::econ::assign_destination(&mut w, u, Some(Target::Object(t)));
    let point = |w: &World| w.obj(u).unwrap().jump.as_ref().unwrap().dest.unwrap();
    let p0 = point(&w);
    assert_eq!((p0.x, p0.y), (was.x, was.y), "the vehicle's position");
    w.obj_mut(t).unwrap().pos.x += 512;
    for _ in 0..10 {
        step(&mut w);
        assert_eq!(point(&w), p0, "the movement point stays at tick {}", w.tick);
    }
}

// spec: S-0204 F3 and S-0502 claim 10 (r34: the direction between two points with no horizontal difference is map east: 16383 with quirks.table_direction on, 16384 with it off)
#[test]
fn the_direction_straight_down_is_map_east() {
    assert_eq!(direction(0, 0, true), EAST);
    assert_eq!(direction(0, 0, false), 16384);
}
