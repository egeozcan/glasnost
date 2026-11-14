//! Bridges (S-0511) on synthetic maps: destroy attempts, sections, piers,
//! notices, span checks and band raises, blow-ups and falls, low strips,
//! hut demolition, engineer repair and the repair cursor.
//!
//! The theater is invented: a bridge role of 16 tiles 4 × 4 at tile 5 and a
//! wooden-bridge role at tile 21, with the positions start ramps 1 and 2
//! (along x), 4 and 5 (along y), far ramps 3 (x) and 6 (y), and the pier
//! bases 7 (x) and 12 (y). Overlays 0 to 238 are registered so that the
//! piece-role binding's indices exist; the low pieces are road and the
//! destroyed piece is water, which no ground unit may enter.

use std::sync::Arc;

use glasnost_map::terrain::{BridgePositions, Roles, TileCatalogue, TileCell, TileInfo};
use glasnost_map::{CellXY, Map, NO_OVERLAY};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LandRow, LandType, Locomotor, OverlayType, UnitType, WarheadType,
};
use glasnost_sim::bridges::blow::Ctx;
use glasnost_sim::bridges::pieces::{self, Family, Line, Role};
use glasnost_sim::bridges::{self, high, hut};
use glasnost_sim::combat::area::{self, Detonation};
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::rules::{Behaviour, Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};

const SIZE: i32 = 50;
/// First tile of the bridge role.
const B0: i32 = 5;
/// First tile of the wooden-bridge role.
const W0: i32 = 21;
/// Positions *q* of the invented theater.
const START_X: i32 = 1;
const FAR_X: i32 = 3;
const START_Y: i32 = 4;
const FAR_Y: i32 = 6;
const M1: i32 = 7;
const M2: i32 = 12;

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn tile(f: Family, q: i32) -> i32 {
    match f {
        Family::Bridge => B0 + q - 1,
        Family::Wooden => W0 + q - 1,
    }
}

fn catalogue() -> Arc<TileCatalogue> {
    let mut c = (*testkit::catalogue()).clone();
    let big = TileInfo {
        width: 4,
        height: 4,
        has_image: true,
        cells: vec![
            Some(TileCell {
                terrain_code: 0,
                slope: 0,
            });
            16
        ],
        variants: 0,
        anim: None,
        no_burrowing: false,
    };
    // A ramp tile whose cells have slope shape 5 (claim 19's ineligible slope).
    let steep = TileInfo {
        width: 1,
        height: 1,
        has_image: true,
        cells: vec![Some(TileCell {
            terrain_code: 0,
            slope: 5,
        })],
        variants: 0,
        anim: None,
        no_burrowing: false,
    };
    assert_eq!(c.tiles.len() as i32, B0);
    for _ in 0..32 {
        c.tiles.push(big.clone());
    }
    c.tiles.push(steep);
    c.allow_tiberium = vec![true; c.tiles.len()];
    c.roles = Roles {
        bridge: B0,
        wooden_bridge: W0,
        ..Roles::default()
    };
    c.bridge_positions = BridgePositions {
        top_left: [START_X, 2],
        top_right: [START_Y, 5],
        bottom_right: [FAR_X, -1],
        bottom_left: [FAR_Y, -1],
        middle: [M1, M2],
    };
    Arc::new(c)
}

/// The steep 1 × 1 tile.
const STEEP: i32 = W0 + 16;

fn ruleset() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let names: Vec<String> = (0..=238).map(|i| format!("OV{i}")).collect();
    for (i, n) in names.iter().enumerate() {
        let mut o = OverlayType {
            has_art: Some(true),
            ..Default::default()
        };
        if let Some(l) = pieces::low_of(i as u8) {
            o.land = Some(if l.role == Role::X {
                LandType::Water
            } else {
                LandType::Road
            });
            o.no_use_tile_land_type = Some(true);
        }
        r.sim.overlays.insert(n.clone(), o);
    }
    r.registries.insert("overlays".into(), names);
    let zero = Some(dec("0"));
    r.sim.land_types.insert(
        LandType::Water,
        LandRow {
            foot: zero.clone(),
            track: zero.clone(),
            wheel: zero.clone(),
            hover: zero,
            ..Default::default()
        },
    );
    let verses = Some(vec![dec("1"); 11]);
    r.sim.warheads.insert(
        "DEMO".into(),
        WarheadType {
            verses: verses.clone(),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "BLAST".into(),
        WarheadType {
            verses,
            wall: Some(true),
            ..Default::default()
        },
    );
    r.sim.general.c4_warhead = Some("DEMO".into());
    // A bridge strength of 1: every chance draw R(1, 1) = 1 is below any damage above 1.
    r.sim.general.bridge_strength = Some(1);
    let reg = |r: &mut Ruleset, k: &str, n: &str| {
        let l = r.registries.entry(k.into()).or_default();
        if !l.iter().any(|x| x == n) {
            l.push(n.into());
        }
    };
    let mut eng = r.sim.infantry["WALKER"].clone();
    eng.engineer = Some(true);
    eng.primary = None;
    r.sim.infantry.insert("ENGI".into(), eng);
    reg(&mut r, "infantry", "ENGI");
    let jet = UnitType {
        strength: Some(100),
        speed: Some(5),
        locomotion_class: Some(glasnost_rules::sim::LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Jumpjet),
        ..Default::default()
    };
    r.sim.vehicles.insert("JET".into(), jet);
    reg(&mut r, "vehicles", "JET");
    r.sim.buildings.insert(
        "HUT".into(),
        BuildingType {
            strength: Some(500),
            foundation: Some([1, 1]),
            bridge_repair_hut: Some(true),
            ..Default::default()
        },
    );
    reg(&mut r, "buildings", "HUT");
    r
}

fn game(r: &Ruleset, map: &Map) -> World {
    let rules = Arc::new(SimRules::new(r, "synthetic-bridges"));
    glasnost_sim::load::new_game(rules, map, catalogue(), &testkit::setup(5))
        .expect("synthetic bridge map loads")
}

fn put_tile(m: &mut Map, c: CellXY, t: i32, sub: u8, level: i8) {
    let cell = m.cells.get_mut(c).expect("cell on the map");
    cell.tile = t;
    cell.sub_tile = sub;
    cell.level = level;
}

fn put_section(m: &mut Map, c: CellXY, f: Family, line: Line) {
    let cell = m.cells.get_mut(c).expect("cell on the map");
    cell.overlay = pieces::high_overlay(f, line);
    cell.overlay_data = pieces::data_of(line, pieces::State::Intact);
}

/// A high bridge of `f` along x on row 0: a start ramp at x = `x0` (sub-tile
/// 8, with a sub-tile 7 cell before it for S-0010's span), sections on
/// x0 … x1 − 1 (pier reference cells excepted) and a far ramp at `x1`
/// (sub-tile 4). Each x in `piers` holds a pier whose reference cell (sub-
/// tile 4) is at x, with its column at sub-tiles 0 and 8 and its span start
/// at x + 1 (sub-tile 5).
fn high_x(m: &mut Map, f: Family, x0: i32, x1: i32, piers: &[i32]) {
    put_tile(m, at(x0 - 1, 0), tile(f, START_X), 7, 0);
    put_tile(m, at(x0, 0), tile(f, START_X), 8, 0);
    put_tile(m, at(x1, 0), tile(f, FAR_X), 4, 0);
    for &p in piers {
        let t = tile(f, M1);
        put_tile(m, at(p, -1), t, 0, 4);
        put_tile(m, at(p, 0), t, 4, 4);
        put_tile(m, at(p, 1), t, 8, 4);
        put_tile(m, at(p + 1, 0), t, 5, 4);
    }
    for x in x0..x1 {
        if !piers.contains(&x) {
            put_section(m, at(x, 0), f, Line::X);
        }
    }
}

/// A low bridge of `f` along x: strips centred on row 0 at x0 … x1, each
/// holding `role(x)`.
fn low_x(m: &mut Map, f: Family, x0: i32, x1: i32, role: impl Fn(i32) -> Role) {
    for x in x0..=x1 {
        let o = pieces::low_overlay(f, Line::X, role(x));
        for (k, dy) in [-1, 0, 1].into_iter().enumerate() {
            let cell = m.cells.get_mut(at(x, dy)).expect("cell on the map");
            cell.overlay = o;
            cell.overlay_data = k as u8;
        }
    }
}

fn data(w: &World, c: CellXY) -> Option<u8> {
    let cell = w.cells.cell(c)?;
    (cell.overlay != NO_OVERLAY).then_some(cell.overlay_data)
}

fn role(w: &World, c: CellXY) -> Option<Role> {
    pieces::low_at(w, c).map(|l| l.role)
}

fn pier_k(w: &World, c: CellXY) -> Option<i32> {
    match pieces::piece_at(w, c) {
        Some((_, pieces::Piece::Pier(_, k))) => Some(k),
        _ => None,
    }
}

fn alive(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.health > 0)
}

fn spawn(w: &mut World, kind: Kind, name: &str, c: CellXY) -> ObjectId {
    testkit::spawn(w, kind, name, 0, c, 0)
}

/// Moves a vehicle from the ground of its cell onto the deck there, with
/// its occupancy claim.
fn onto_deck(w: &mut World, id: ObjectId) {
    let c = w.obj(id).unwrap().pos.cell();
    w.cells.leave(c, Layer::Ground, id);
    w.cells.claim_vehicle(c, Layer::Ground, false);
    w.cells.enter(c, Layer::Deck, id, false);
    w.cells.claim_vehicle(c, Layer::Deck, true);
    let o = w.obj_mut(id).unwrap();
    o.layer = Layer::Deck;
    o.pos.z += 416;
    // The driving claim moves with the flag, to the deck record (S-0209 claim 1).
    if let Some(d) = o.drive.as_mut() {
        d.claims = vec![(c, Layer::Deck)];
    }
}

/// A repeating tag over one trigger whose only event is the bridge-
/// destroyed code (31), with no action, carried by `cells`.
fn tagged(m: &mut Map, cells: &[CellXY]) {
    use glasnost_map::triggers::{Event, TagDef, TriggerDef, Triggers};
    let trig = TriggerDef {
        events: vec![Event {
            code: 31, // spec: S-0511 claim 8 (S-0011 claim 20's bridge-destroyed event)
            kind: 0,
            value: 0,
            type_name: None,
        }],
        actions: vec![],
        ..TriggerDef::created(b"T".to_vec())
    };
    m.triggers = Triggers {
        triggers: vec![trig],
        tags: vec![TagDef {
            id: b"TAG".to_vec(),
            repeat: 2,
            name: Vec::new(),
            first: Some(0),
            chain: vec![0],
        }],
        cell_tags: cells
            .iter()
            .map(|c| ((c.x as i16, c.y as i16), 0))
            .collect(),
        ..Default::default()
    };
}

fn base() -> Map {
    testkit::empty_map(SIZE)
}

fn wh(w: &World, name: &str) -> u32 {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == name)
        .unwrap() as u32
}

fn blast(w: &mut World, c: CellXY, z: i32, warhead: &str) {
    let h = wh(w, warhead);
    let p = Pos::centre(c, z);
    area::detonate(
        w,
        Detonation {
            point: p,
            damage: 100,
            warhead: Some(h),
            source: None,
            house: None,
            resources: false,
        },
    );
}

// ---------------------------------------------------------------------
// Claims 1 and 2: where attempts come from and what they act on.

// spec: S-0511 claim 1 (area damage with the bridge-destruction setting on and a wall warhead makes attempts on the centre cell; the setting gates area damage only; the height window applies to a high-bridge cell)
#[test]
fn claim_1_area_damage_is_gated() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 6, &[]);
    let r = ruleset();
    let c = at(3, 0);
    // The setting off: no change.
    let mut w = game(&r, &m);
    w.econ.settings.bridge_destruction = Some(false);
    blast(&mut w, c, 416, "BLAST");
    assert_eq!(data(&w, c), Some(0));
    // A warhead that does not damage walls: no change.
    let mut w = game(&r, &m);
    w.econ.settings.bridge_destruction = Some(true);
    blast(&mut w, c, 416, "SA");
    assert_eq!(data(&w, c), Some(0));
    // Below the height window: no change.
    blast(&mut w, c, 0, "BLAST");
    assert_eq!(data(&w, c), Some(0));
    // In the window: one attempt, damaged.
    blast(&mut w, c, 416, "BLAST");
    assert_eq!(data(&w, c), Some(6));
    // The trigger seam's direct attempts ignore the setting: up to 4, stopping at the first success.
    let mut w = game(&r, &m);
    w.econ.settings.bridge_destruction = Some(false);
    assert!(bridges::cell_attempts(&mut w, c));
    assert_eq!(data(&w, c), None, "lost after its second attempt");
    assert_eq!(
        data(&w, at(2, 0)),
        Some(8),
        "the start-side neighbour is open on its far side"
    );
}

// spec: S-0511 claim 1 (the ion-cannon warhead bypasses the chance draws: up to four attempts on a high test, one on a low test)
#[test]
fn claim_1_ion_cannon_attempts() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 6, &[]);
    low_x(&mut m, Family::Wooden, 0, 4, |_| Role::I(0));
    // Move the low bridge away from the high one.
    let mut m2 = base();
    high_x(&mut m2, Family::Bridge, 0, 6, &[]);
    for x in 0..=4 {
        for dy in [-1, 0, 1] {
            let src = *m.cells.get(at(x, dy)).unwrap();
            let cell = m2.cells.get_mut(at(x, dy + 6)).unwrap();
            cell.overlay = src.overlay;
            cell.overlay_data = src.overlay_data;
        }
    }
    let mut r = ruleset();
    r.sim.general.ion_cannon_warhead = Some("BLAST".into());
    let mut w = game(&r, &m2);
    w.econ.settings.bridge_destruction = Some(true);
    blast(&mut w, at(3, 0), 416, "BLAST");
    assert_eq!(
        data(&w, at(3, 0)),
        None,
        "the high section is lost in up to four attempts"
    );
    blast(&mut w, at(2, 6), 0, "BLAST");
    assert_eq!(
        role(&w, at(2, 6)),
        Some(Role::D(0)),
        "a low test makes one attempt"
    );
}

// spec: S-0511 claims 1 and 22 (a successful attempt from area damage makes every object whose attack target is the centre cell itself drop it; a neighbouring cell's target stays)
#[test]
fn claim_1_success_drops_cell_targets() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 6, &[]);
    let mut w = game(&ruleset(), &m);
    w.econ.settings.bridge_destruction = Some(true);
    let c = at(3, 0);
    let v = spawn(&mut w, Kind::Vehicle, "CART", at(3, 8));
    let u = spawn(&mut w, Kind::Vehicle, "CART", at(4, 8));
    w.obj_mut(v).unwrap().target = Some(Target::Cell(c));
    w.obj_mut(u).unwrap().target = Some(Target::Cell(at(4, 0)));
    blast(&mut w, c, 416, "BLAST");
    assert_eq!(
        w.obj(v).unwrap().target,
        Some(Target::Cell(c)),
        "a failed attempt keeps it"
    );
    blast(&mut w, c, 416, "BLAST");
    assert_eq!(data(&w, c), None);
    assert_eq!(w.obj(v).unwrap().target, None);
    assert_eq!(w.obj(u).unwrap().target, Some(Target::Cell(at(4, 0))));
}

// spec: S-0511 claim 22 (a suspended behaviour is reverted first, restoring target and destination; a destination naming the centre cell stays; a projectile aimed at the cell loses it, one aimed at a neighbouring cell keeps it; no new behaviour)
#[test]
fn claim_22_revert_and_projectile_target_loss() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 6, &[]);
    let mut w = game(&ruleset(), &m);
    w.econ.settings.bridge_destruction = Some(true);
    let c = at(3, 0);
    let v = spawn(&mut w, Kind::Vehicle, "CART", at(3, 8));
    let u = spawn(&mut w, Kind::Vehicle, "CART", at(4, 8));
    let mover = spawn(&mut w, Kind::Vehicle, "CART", at(5, 8));
    // An override in effect on v: guard put aside with u as target and a destination.
    {
        let o = w.obj_mut(v).unwrap();
        o.target = Some(Target::Cell(c));
        o.beh.current = Some(Behaviour::Attack);
        o.beh.suspended = Some(Behaviour::Guard);
        o.beh.aside = Some((Some(Target::Object(u)), Some(Target::Cell(at(1, 9)))));
    }
    w.obj_mut(mover).unwrap().dest = Some(Target::Cell(c));
    let current = w.obj(mover).unwrap().beh.current;
    let wp = w
        .rules()
        .combat
        .weapons
        .iter()
        .position(|x| x.projectile.is_some())
        .unwrap() as u32;
    let from = w.obj(u).unwrap().pos;
    let hit = glasnost_sim::combat::projectile::launch(&mut w, u, wp, Target::Cell(c), from, 10)
        .expect("a projectile at the centre cell");
    let miss =
        glasnost_sim::combat::projectile::launch(&mut w, u, wp, Target::Cell(at(4, 0)), from, 10)
            .expect("a projectile at the neighbouring cell");
    blast(&mut w, c, 416, "BLAST");
    blast(&mut w, c, 416, "BLAST");
    assert_eq!(data(&w, c), None, "the attempt succeeded");
    let o = w.obj(v).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard), "reverted");
    assert_eq!(o.beh.suspended, None);
    assert_eq!(o.target, Some(Target::Object(u)), "restored target");
    assert_eq!(o.dest, Some(Target::Cell(at(1, 9))), "restored destination");
    let o = w.obj(mover).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(c)), "a destination is not erased");
    assert_eq!(o.beh.current, current, "no new behaviour");
    assert_eq!(w.projectiles[&hit].target, None);
    assert_eq!(w.projectiles[&miss].target, Some(Target::Cell(at(4, 0))));
}

// spec: S-0511 claim 2 (a high-bridge cell: the section whose overlay marks it; a pier at a sub-tile without a deck; other pier cells and other cells: nothing, the attempt fails)
#[test]
fn claim_2_what_an_attempt_acts_on() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[4]);
    let mut w = game(&ruleset(), &m);
    // A marked cell one step north of the section cell acts on that section.
    assert!(w.cells.is_bridge(at(2, -1)));
    assert!(!bridges::attempt(&mut w, at(2, -1)));
    assert_eq!(data(&w, at(2, 0)), Some(6));
    // The pier's reference cell (sub-tile 4, even) is attempted.
    assert!(!bridges::attempt(&mut w, at(4, 0)));
    assert_eq!(pier_k(&w, at(4, 0)), Some(3));
    // A plain cell: nothing.
    assert!(!bridges::attempt(&mut w, at(0, 9)));
}

// spec: S-0511 claim 2 (the responsible section: no search for another overlay when it is missing; a non-collapsed pier tile of a family at the attempted cell supplies the family over a replaced responsible overlay, and claim 3 acts on the responsible cell's retained state; a state outside the section-state table gives no change and is not a no-deck pier attempt)
#[test]
fn claim_2_responsible_section() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut w = game(&ruleset(), &m);
    let (r, c) = (at(2, 0), at(2, -1));
    assert_eq!(w.cells.responsible(c), Some(r));
    // The responsible overlay removed: the deck stays and nothing else is searched.
    let before: Vec<Option<u8>> = (0..8).map(|x| data(&w, at(x, 0))).collect();
    w.cells.grid.get_mut(r).unwrap().overlay = NO_OVERLAY;
    assert!(w.cells.is_bridge(c));
    assert!(!bridges::attempt(&mut w, c));
    for x in (0..8).filter(|&x| x != 2) {
        assert_eq!(data(&w, at(x, 0)), before[x as usize], "x = {x}");
    }
    // A replacement overlay that is neither a low piece nor a section, and
    // a pier tile of the bridge family at the attempted cell.
    let other = 1;
    assert!(pieces::low_of(other).is_none() && pieces::high_of(other).is_none());
    w.cells.grid.get_mut(c).unwrap().tile = tile(Family::Bridge, M1);
    for (v, lost) in [(99, false), (6, true)] {
        let cell = w.cells.grid.get_mut(r).unwrap();
        cell.overlay = other;
        cell.overlay_data = v;
        assert_eq!(bridges::attempt(&mut w, c), lost, "retained state {v}");
        let cell = w.cells.cell(r).unwrap();
        if lost {
            assert_eq!(
                cell.overlay, NO_OVERLAY,
                "the replacement overlay is removed"
            );
            assert!(!w.cells.is_bridge(c));
        } else {
            assert_eq!((cell.overlay, cell.overlay_data), (other, v), "no change");
            assert_eq!(pier_k(&w, c), Some(0), "not a pier attempt");
        }
    }
}

// ---------------------------------------------------------------------
// Claims 3 to 5: sections, notices, losses.

// spec: S-0511 claim 3 (intact or chipped → damaged with damage notices both ways, fails; damaged → lost with loss notices far then start, succeeds; open on one side → lost with a notice to the other side only)
#[test]
fn claim_3_direct_attempts_on_a_section() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut w = game(&ruleset(), &m);
    assert!(!bridges::attempt(&mut w, at(3, 0)));
    assert_eq!(data(&w, at(3, 0)), Some(6), "damaged");
    assert_eq!(data(&w, at(2, 0)), Some(5), "chipped on its far side");
    assert_eq!(data(&w, at(4, 0)), Some(4), "chipped on its start side");
    assert!(bridges::attempt(&mut w, at(3, 0)));
    assert_eq!(data(&w, at(3, 0)), None, "lost");
    assert_eq!(data(&w, at(2, 0)), Some(8), "open on its far side");
    assert_eq!(data(&w, at(4, 0)), Some(7), "open on its start side");
    // An open section needs one attempt; its notice goes to the other side only.
    assert!(bridges::attempt(&mut w, at(4, 0)));
    assert_eq!(data(&w, at(4, 0)), None);
    assert_eq!(data(&w, at(5, 0)), Some(7));
}

// spec: S-0511 claim 4 (damage notice on a section chipped on the other side: damaged; a loss notice to a section open on the other side loses it at once and passes on; a ramp takes its alternate look)
#[test]
fn claim_4_notices() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut w = game(&ruleset(), &m);
    // x = 3 chipped on its far side by x = 4's damage, then on its start side by x = 2's: damaged.
    bridges::attempt(&mut w, at(4, 0));
    assert_eq!(data(&w, at(3, 0)), Some(5));
    bridges::attempt(&mut w, at(2, 0));
    assert_eq!(
        data(&w, at(3, 0)),
        Some(6),
        "chipped on the other side → damaged"
    );
    // Lose x = 2: x = 3 becomes open on its start side.
    bridges::attempt(&mut w, at(2, 0));
    assert_eq!(data(&w, at(3, 0)), Some(7));
    // Lose x = 4 (damaged): its start-side notice reaches x = 3, open on the other side: lost at once.
    bridges::attempt(&mut w, at(4, 0));
    assert_eq!(data(&w, at(3, 0)), None, "lost at once");
    // A ramp receiving a notice takes the alternate look.
    assert!(w.cells.alternate.is_empty());
    bridges::attempts(&mut w, at(7, 0), 2);
    assert!(w.cells.alternate.contains(&at(8, 0)));
}

// spec: S-0511 claim 5 (a lost section's overlay is removed; its four cells lose the high-bridge marking and gain the bridge-body mark; blow-ups in the order section cell, one and two steps along the marking direction, one step against it, after the notices' losses)
#[test]
fn claim_5_losing_a_section() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut w = game(&ruleset(), &m);
    let c = at(3, 0);
    let cells = [c, at(3, -1), at(3, -2), at(3, 1)];
    assert!(cells.iter().all(|&x| w.cells.is_bridge(x)));
    bridges::attempt(&mut w, c);
    let s = pieces::section_in(&w, c).unwrap();
    let mut ctx = Ctx::default();
    assert!(high::attempt_section(&mut w, s, &mut ctx));
    assert_eq!(ctx.blown, cells.to_vec());
    for x in cells {
        assert!(!w.cells.is_bridge(x));
        assert!(w.cells.body(x));
    }
    // A chain loss: its notices' losses blow up before the attempted section.
    bridges::attempt(&mut w, at(5, 0));
    bridges::attempt(&mut w, at(5, 0));
    // x = 4 is now open on both sides and was lost by the notice.
    assert_eq!(data(&w, at(4, 0)), None);
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut w = game(&ruleset(), &m);
    bridges::attempts(&mut w, at(3, 0), 2);
    bridges::attempt(&mut w, at(5, 0));
    let s = pieces::section_in(&w, at(5, 0)).unwrap();
    let mut ctx = Ctx::default();
    high::attempt_section(&mut w, s, &mut ctx);
    assert_eq!(ctx.blown[0], at(4, 0), "the notice's loss blows up first");
    assert_eq!(ctx.blown[4], at(5, 0));
}

// ---------------------------------------------------------------------
// Claim 6: piers.

// spec: S-0511 claim 6 (direct attempt on states 0 to 2: state 3 with damage notices both ways; on state 3: collapse to state 4, every cell of the tile at the reference level − 4, column blow-ups in decreasing-cross, reference, increasing-cross order before the loss notices)
#[test]
fn claim_6_direct_pier_attempts() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 9, &[4]);
    let mut w = game(&ruleset(), &m);
    let r = at(4, 0);
    assert!(!bridges::attempt(&mut w, r));
    assert_eq!(pier_k(&w, r), Some(3));
    for c in [at(4, -1), at(4, 1), at(5, 0)] {
        assert_eq!(pier_k(&w, c), Some(3), "every cell of the tile");
    }
    assert_eq!(
        data(&w, at(3, 0)),
        Some(5),
        "damage notice to the start side"
    );
    assert_eq!(
        data(&w, at(5, 0)),
        Some(4),
        "damage notice to the section of the pier's span start"
    );
    let mut ctx = Ctx::default();
    assert!(high::attempt_pier(
        &mut w,
        r,
        Family::Bridge,
        Line::X,
        3,
        &mut ctx
    ));
    assert_eq!(pier_k(&w, r), Some(4));
    assert_eq!(&ctx.blown[..3], &[at(4, -1), r, at(4, 1)]);
    for c in [at(4, -1), r, at(4, 1), at(5, 0)] {
        assert_eq!(w.cells.cell(c).unwrap().level, 0, "4 − 4");
    }
    assert_eq!(
        data(&w, at(3, 0)),
        Some(8),
        "loss notice: open on its far side"
    );
}

// spec: S-0511 Terms (Pier: a state replacement changes the eight-neighbour-connected component of cells with exactly the reference cell's pre-change tile number; a different tile laid over part of the pier, a disconnected copy and a different-state tile stay; a diagonal same-tile copy changes with it) and claim 6 (a collapse sets every changed cell to the reference level − 4)
#[test]
fn claim_6_pier_tile_component() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 9, &[4]);
    let t = tile(Family::Bridge, M1);
    put_tile(&mut m, at(4, 1), 0, 0, 4); // laid over part of the pier
    put_tile(&mut m, at(6, 1), t, 0, 6); // a diagonal-only copy, touching (5, 0)
    put_tile(&mut m, at(3, -2), t + 1, 0, 4); // a different state, touching (4, -1)
    put_tile(&mut m, at(4, -4), t, 0, 4); // the same tile, not connected
    let mut w = game(&ruleset(), &m);
    let r = at(4, 0);
    let tile_at = |w: &World, c: CellXY| w.cells.cell(c).unwrap().tile;
    assert!(!bridges::attempt(&mut w, r));
    for c in [at(4, -1), r, at(5, 0), at(6, 1)] {
        assert_eq!(tile_at(&w, c), t + 3, "{c:?} in the component");
    }
    assert_eq!(tile_at(&w, at(4, 1)), 0, "the overlaid cell stays");
    assert_eq!(
        tile_at(&w, at(3, -2)),
        t + 1,
        "a different state does not join"
    );
    assert_eq!(tile_at(&w, at(4, -4)), t, "a disconnected copy stays");
    assert!(bridges::attempt(&mut w, r));
    for c in [at(4, -1), r, at(5, 0), at(6, 1)] {
        assert_eq!(tile_at(&w, c), t + 4);
        assert_eq!(
            w.cells.cell(c).unwrap().level,
            0,
            "{c:?}: the reference level 4 − 4"
        );
    }
    assert_eq!(
        w.cells.cell(at(4, 1)).unwrap().level,
        4,
        "not a changed cell"
    );
    assert_eq!(w.cells.cell(at(3, -2)).unwrap().level, 4);
}

// spec: S-0511 Terms (Pier: a position in both non-collapsed pier ranges is read along x, x states 0 to 3 before y states 0 to 3; an x match only at collapsed state 4 does not override a non-collapsed y match; repair's start-piece selection reads the axis-specific sub-tile)
#[test]
fn terms_overlapping_pier_ranges() {
    let mut m = base();
    put_tile(&mut m, at(0, 0), tile(Family::Bridge, M1 + 2), 7, 0);
    let mut w = game(&ruleset(), &m);
    let mut c = (*catalogue()).clone();
    // An invented theater whose y pier range starts two positions after the x one.
    c.bridge_positions.middle = [M1, M1 + 2];
    w.catalogue = Some(Arc::new(c));
    use pieces::Piece::Pier;
    assert_eq!(pieces::piece_of_q(&w, M1 + 2), Some(Pier(Line::X, 2)));
    assert_eq!(pieces::piece_of_q(&w, M1 + 3), Some(Pier(Line::X, 3)));
    assert_eq!(
        pieces::piece_of_q(&w, M1 + 4),
        Some(Pier(Line::Y, 2)),
        "x-collapsed, y-live"
    );
    assert_eq!(pieces::piece_of_q(&w, M1 + 6), Some(Pier(Line::Y, 4)));
    // Read along y, the cell at sub-tile 7 is a y pier's start piece.
    assert_eq!(
        pieces::pier_along(&w, at(0, 0), Line::Y),
        Some((Family::Bridge, 0))
    );
    assert!(high::is_start_piece(&w, at(0, 0), Line::Y));
    assert!(
        !high::is_start_piece(&w, at(0, 0), Line::X),
        "sub-tile 5 starts along x"
    );
}

// spec: S-0511 claim 6 (notice table: loss notice on the start side of a pier in state 0 → 3; a pier chipped on the arriving side ignores it with quirks.pier_loss_notice_skips_chipped, becomes 3 without)
#[test]
fn claim_6_pier_notice_table() {
    for skips in [true, false] {
        let mut m = base();
        high_x(&mut m, Family::Bridge, 0, 9, &[4]);
        let mut r = ruleset();
        r.globals.quirks.pier_loss_notice_skips_chipped = skips;
        let mut w = game(&r, &m);
        let p = at(4, 0);
        // A damage notice from the start side: state 1.
        bridges::attempt(&mut w, at(3, 0));
        assert_eq!(pier_k(&w, p), Some(1));
        // A loss notice from the start side.
        bridges::attempt(&mut w, at(3, 0));
        assert_eq!(pier_k(&w, p), Some(if skips { 1 } else { 3 }));
    }
    // State 3 collapses on a loss notice; the collapse's tile and height change follow.
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 9, &[4]);
    let mut w = game(&ruleset(), &m);
    bridges::attempt(&mut w, at(4, 0));
    assert_eq!(pier_k(&w, at(4, 0)), Some(3));
    bridges::attempts(&mut w, at(3, 0), 2);
    assert_eq!(pier_k(&w, at(4, 0)), Some(4));
    assert_eq!(w.cells.cell(at(4, 0)).unwrap().level, 0);
}

// ---------------------------------------------------------------------
// Claims 7 and 8: span checks and the bridge-destroyed raises.

// spec: S-0511 claim 7 (runs of section cells touching neither the span's first nor its last cell are lost; runs touching either stay)
#[test]
fn claim_7_isolated_runs_fall() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 10, &[]);
    let mut w = game(&ruleset(), &m);
    bridges::attempts(&mut w, at(3, 0), 2);
    for x in [0, 1, 2, 4, 5, 6, 7, 8, 9] {
        assert!(data(&w, at(x, 0)).is_some(), "x = {x} stays");
    }
    bridges::attempts(&mut w, at(7, 0), 2);
    for x in [4, 5, 6] {
        assert_eq!(data(&w, at(x, 0)), None, "x = {x} isolated");
    }
    for x in [0, 1, 2, 8, 9] {
        assert!(data(&w, at(x, 0)).is_some(), "x = {x} touches an end");
    }
}

// spec: S-0511 claim 7 (quirks.bridge_span_cap: with no far piece within 29 cells the check does nothing; off: the far piece is found at any distance)
#[test]
fn claim_7_span_cap() {
    for cap in [true, false] {
        let mut m = base();
        high_x(&mut m, Family::Bridge, -16, 15, &[]);
        let mut r = ruleset();
        r.globals.quirks.bridge_span_cap = cap;
        let mut w = game(&r, &m);
        bridges::attempts(&mut w, at(-10, 0), 2);
        bridges::attempts(&mut w, at(-6, 0), 2);
        let isolated = data(&w, at(-8, 0)).is_none();
        assert_eq!(isolated, !cap, "cap {cap}");
    }
}

// spec: S-0511 claim 7 (finding the span: from the named loss cell, the first marked neighbour's section, then the nearest start piece toward the start side up to the legal-map boundary, passing over non-bridge cells)
#[test]
fn claim_7_start_search_passes_over_a_gap() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 10, &[]);
    for x in [1, 2] {
        m.cells.get_mut(at(x, 0)).unwrap().overlay = NO_OVERLAY;
    }
    let mut w = game(&ruleset(), &m);
    // Neither a deck nor a bridge-body cell: a non-bridge gap.
    w.cells.clear_body(&[at(1, 0), at(2, 0)]);
    assert!(!w.cells.is_bridge(at(1, 0)) && !w.cells.body(at(1, 0)));
    bridges::attempts(&mut w, at(6, 0), 2);
    for x in [3, 4, 5] {
        assert_eq!(
            data(&w, at(x, 0)),
            None,
            "x = {x}: isolated from the start ramp"
        );
    }
    for x in [0, 7, 8, 9] {
        assert!(data(&w, at(x, 0)).is_some(), "x = {x} touches an end");
    }
}

// spec: S-0511 claims 6 and 7 and Edge cases (a direct pier collapse searches from its start-side and then its far-side named cell on the post-collapse tiles; a collapsed pier never qualifies as a start piece and, with quirks.collapsed_pier_span_search, is passed over at the far end too, so both checks select the same outer-ramp-to-outer-ramp extent and raise its band n + 2 times for n isolated losses in the first; off: each former span is checked on its own)
#[test]
fn claim_7_pier_collapse_checks_the_combined_extent() {
    for search in [true, false] {
        for (tag, name) in [(at(2, 0), "start side"), (at(8, 0), "far side")] {
            let mut m = base();
            high_x(&mut m, Family::Bridge, 0, 12, &[5]);
            tagged(&mut m, &[tag]);
            let mut r = ruleset();
            r.globals.quirks.collapsed_pier_span_search = search;
            let mut w = game(&r, &m);
            // A missing section in each former span.
            bridges::attempts(&mut w, at(3, 0), 2);
            bridges::attempts(&mut w, at(9, 0), 2);
            assert_eq!(data(&w, at(3, 0)), None);
            assert_eq!(data(&w, at(9, 0)), None);
            let f0 = fired(&w);
            assert!(!bridges::attempt(&mut w, at(5, 0)));
            assert!(bridges::attempt(&mut w, at(5, 0)));
            assert_eq!(pier_k(&w, at(5, 0)), Some(4));
            let kept = |x: i32| data(&w, at(x, 0)).is_some();
            // The outer runs touch the extent's first or last cell and stay.
            for x in [0, 1, 2, 10, 11] {
                assert!(kept(x), "{search} {name}: cell {x} stays");
            }
            if search {
                // The far side's run 6..=8 lies inside the combined extent:
                // three isolated losses, 3 + 1 raises, and one more from the
                // repeated check.
                for x in 6..=8 {
                    assert!(!kept(x), "{name}: cell {x} lost");
                }
                assert_eq!(fired(&w) - f0, 5, "{name}: n + 2 band raises");
            } else {
                for x in 6..=8 {
                    assert!(kept(x), "{name}: cell {x} stays");
                }
                assert_eq!(fired(&w) - f0, 1, "{name}: one band pass over its span");
            }
        }
    }
}

/// Firings of the bridge-destroyed tag.
fn fired(w: &World) -> u64 {
    w.triggers.fired
}

// spec: S-0511 claim 8 (one raise per destroyed low strip over the run's band: each band cell, one across toward increasing y, one and two toward decreasing y; the last band cell is never processed with quirks.bridge_event_band_short; one tag over several cells is raised once per cell)
#[test]
fn claim_8_low_band_raises() {
    for short in [true, false] {
        let mut m = base();
        low_x(&mut m, Family::Bridge, 0, 4, |_| Role::I(0));
        // Cells of the band: x = 1 at +1 and −2 across; the last band cell x = 4.
        tagged(&mut m, &[at(1, 1), at(1, -2), at(4, 0)]);
        let mut r = ruleset();
        r.globals.quirks.bridge_event_band_short = short;
        let mut w = game(&r, &m);
        let f0 = fired(&w);
        assert!(!bridges::attempt(&mut w, at(2, 0)));
        assert_eq!(fired(&w), f0, "damage raises nothing");
        assert!(bridges::attempt(&mut w, at(2, 0)));
        assert_eq!(fired(&w) - f0, if short { 2 } else { 3 }, "short {short}");
    }
}

// spec: S-0511 claim 7 (a span with a missing section has n + 1 band raises for n isolated losses; one when n = 0)
#[test]
fn claim_8_high_band_passes() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 10, &[]);
    tagged(&mut m, &[at(2, 0)]);
    let mut w = game(&ruleset(), &m);
    let f0 = fired(&w);
    bridges::attempts(&mut w, at(3, 0), 2);
    assert_eq!(fired(&w) - f0, 1, "n = 0: one pass");
    let f1 = fired(&w);
    bridges::attempts(&mut w, at(7, 0), 2);
    assert_eq!(fired(&w) - f1, 4, "three isolated losses: four passes");
}

// ---------------------------------------------------------------------
// Claims 9 to 11: blow-ups, falling, heading hits.

// spec: S-0511 claim 9 (ground-layer objects die by the demolition hit; deck objects drop to the ground layer and fall with the bomb mark; explosion draws only with a non-empty bridge-explosion list)
#[test]
fn claim_9_blow_up() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut w = game(&ruleset(), &m);
    let under = spawn(&mut w, Kind::Vehicle, "CART", at(3, 1));
    let top = spawn(&mut w, Kind::Vehicle, "CART", at(3, 0));
    onto_deck(&mut w, top);
    bridges::attempt(&mut w, at(3, 0));
    let s0 = w.rng.clone();
    let mut ctx = Ctx::default();
    glasnost_sim::bridges::blow::blow_up(&mut w, at(5, 9), 0, &mut ctx);
    assert_eq!(
        w.rng.raw(),
        s0.clone().raw(),
        "an empty explosion list takes no draw"
    );
    bridges::attempt(&mut w, at(3, 0));
    assert!(!alive(&w, under), "the unit under the bridge dies");
    let t = w.obj(top).unwrap();
    assert_eq!(t.layer, Layer::Ground);
    assert!(t.fall.falling && t.fall.bomb_mark);
    assert!(w.cells.list(at(3, 0), Layer::Ground).contains(&top));
    // A non-empty list: explosions appear.
    let mut r = ruleset();
    r.sim.general.bridge_explosions = Some(vec!["BOOM".into()]);
    let mut w = game(&r, &m);
    w.effects.clear();
    bridges::attempts(&mut w, at(3, 0), 2);
    let booms = w
        .effects
        .iter()
        .filter(|e| matches!(e, Effect::Anim { name, .. } if name == "BOOM"))
        .count();
    assert!(booms > 0);
}

// spec: S-0511 claim 9 (the drop keeps target, destination, queued destinations, path and movement state; a stationary unit's deck mark or present flag is cleared and its ground mark made at once, so its former deck record is empty before claim 11 and no heading hit follows)
#[test]
fn claim_9_drop_keeps_orders_and_moves_marks() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut w = game(&ruleset(), &m);
    let c = at(3, 0);
    let top = spawn(&mut w, Kind::Vehicle, "CART", c);
    onto_deck(&mut w, top);
    let walker = spawn(&mut w, Kind::Infantry, "WALKER", c);
    {
        let h = w.obj(walker).unwrap().owner.unwrap();
        let ground = *w.cells.occ(c, Layer::Ground).unwrap();
        let p = (0..5).find(|&p| ground.taken(p)).unwrap();
        w.cells
            .clear_position(c, Layer::Ground, p, glasnost_sim::objects::occ_quirks(&w));
        w.cells.leave(c, Layer::Ground, walker);
        w.cells.enter(c, Layer::Deck, walker, false);
        w.cells.mark_infantry(c, Layer::Deck, p, walker, h);
        let o = w.obj_mut(walker).unwrap();
        o.layer = Layer::Deck;
        o.pos.z += 416;
    }
    // A unit heading to c at deck height, which would die if c's deck
    // record were still marked.
    let deck_z = w.surface_at(256 * c.x + 128, 256 * c.y + 128) + 416;
    let comer = spawn(&mut w, Kind::Infantry, "WALKER", at(3, 2));
    w.obj_mut(comer).unwrap().walk.as_mut().unwrap().target = Some(centre(c, deck_z));
    let far = at(6, 4);
    {
        let o = w.obj_mut(top).unwrap();
        o.dest = Some(Target::Cell(far));
        o.target = Some(Target::Cell(at(1, 4)));
        o.queued_dests.push(Target::Cell(at(7, 4)));
    }
    bridges::attempts(&mut w, c, 2);
    let o = w.obj(top).unwrap();
    assert!(o.fall.falling && o.layer == Layer::Ground);
    assert_eq!(o.dest, Some(Target::Cell(far)), "the destination is kept");
    assert_eq!(o.target, Some(Target::Cell(at(1, 4))), "the target is kept");
    assert_eq!(o.queued_dests, vec![Target::Cell(at(7, 4))]);
    let deck = w.cells.occ(c, Layer::Deck).unwrap();
    assert!(deck.empty(), "the former deck record is empty");
    let ground = w.cells.occ(c, Layer::Ground).unwrap();
    assert!(ground.vehicle_present(), "the ground flag is made at once");
    assert!(
        ground.marks.iter().flatten().any(|m| matches!(
            m.by,
            glasnost_sim::cells::Marker::Infantry(who, _) if who == walker
        )),
        "the walker's ground mark is made at once"
    );
    assert!(alive(&w, comer), "no heading hit: the deck record is empty");
}

// spec: S-0511 claim 10 (a dropped object falls 416 leptons and lands on its 30th update; with the bomb mark it dies on landing)
#[test]
fn claim_10_falling_and_landing() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut w = game(&ruleset(), &m);
    let top = spawn(&mut w, Kind::Vehicle, "CART", at(3, 0));
    onto_deck(&mut w, top);
    bridges::attempts(&mut w, at(3, 0), 2);
    assert!(w.obj(top).unwrap().fall.falling);
    let mut landed = None;
    for k in 1..=40 {
        glasnost_sim::tick::step(&mut w);
        if !w.obj(top).is_some_and(|o| o.fall.falling && o.health > 0) {
            landed = Some(k);
            break;
        }
    }
    assert_eq!(landed, Some(30));
    assert!(!alive(&w, top));
}

// spec: S-0511 claim 10 (landing does not spend or clear the bomb mark: a grounded survivor holding it takes no hit on later updates, and a later fall that adds no new mark still ends with the landing hit; an unmarked object's fall does not)
#[test]
fn claim_10_retained_bomb_mark() {
    let m = base();
    let mut w = game(&ruleset(), &m);
    let marked = spawn(&mut w, Kind::Vehicle, "CART", at(2, 4));
    let plain = spawn(&mut w, Kind::Vehicle, "CART", at(6, 4));
    // A survivor of an earlier marked landing.
    w.obj_mut(marked).unwrap().fall.bomb_mark = true;
    let h = w.obj(marked).unwrap().health;
    for _ in 0..10 {
        glasnost_sim::tick::step(&mut w);
    }
    assert_eq!(w.obj(marked).unwrap().health, h, "no hit while grounded");
    assert!(w.obj(marked).unwrap().fall.bomb_mark);
    // A later fall without a new mark.
    for id in [marked, plain] {
        w.obj_mut(id).unwrap().pos.z += 200;
        glasnost_sim::fall::start(&mut w, id, false);
    }
    for _ in 0..40 {
        glasnost_sim::tick::step(&mut w);
    }
    assert!(
        !alive(&w, marked),
        "the retained mark gives the landing hit"
    );
    assert!(alive(&w, plain));
    assert!(!w.obj(plain).unwrap().fall.falling);
}

// spec: S-0511 claim 10 and S-0301 claim 9 rank 4 (the fall step precedes behaviour processing, locomotion and the firing check; a falling firer is illegal; a still-falling unit neither advances nor fires)
#[test]
fn claim_10_falling_unit_neither_moves_nor_fires() {
    use glasnost_sim::combat::check::{Answer, check};
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut w = game(&ruleset(), &m);
    let top = spawn(&mut w, Kind::Vehicle, "CRUSHER", at(3, 0));
    onto_deck(&mut w, top);
    let foe = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, at(3, 3), 0);
    w.obj_mut(top).unwrap().target = Some(Target::Object(foe));
    bridges::attempts(&mut w, at(3, 0), 2);
    assert_eq!(
        check(&w, top, Some(Target::Object(foe)), 0, true),
        Answer::Illegal
    );
    let x0 = (w.obj(top).unwrap().pos.x, w.obj(top).unwrap().pos.y);
    let health = w.obj(foe).unwrap().health;
    for _ in 0..10 {
        glasnost_sim::tick::step(&mut w);
        let o = w.obj(top).unwrap();
        assert!(o.fall.falling);
        assert_eq!((o.pos.x, o.pos.y), x0, "no horizontal advance");
    }
    assert_eq!(w.obj(foe).unwrap().health, health, "no shot while falling");
}

/// The centre point of `c` at height `z`.
fn centre(c: CellXY, z: i32) -> Pos {
    Pos::new(256 * c.x + 128, 256 * c.y + 128, z)
}

// spec: S-0511 claim 11 (rev 7, B6 negative-kind variants: drop-pod, fly, jumpjet, rocket, teleport and tunnel locomotion never count as heading to the cell, whatever their destination or track; an ordinary drive unit in the same block does; mech is an open question, not asserted)
#[test]
fn claim_11_other_locomotion_kinds_never_head_to_the_cell() {
    use glasnost_rules::sim::Locomotor;
    let c = at(3, 0);
    for (loco, expect) in [
        (Locomotor::Drive, true),
        (Locomotor::DropPod, false),
        (Locomotor::Fly, false),
        (Locomotor::Jumpjet, false),
        (Locomotor::Rocket, false),
        (Locomotor::Teleport, false),
        (Locomotor::Tunnel, false),
    ] {
        let mut r = ruleset();
        r.sim.vehicles.get_mut("CART").unwrap().locomotor = Some(loco);
        let mut m = base();
        high_x(&mut m, Family::Bridge, 0, 8, &[]);
        let mut w = game(&r, &m);
        let z = w.surface_at(256 * c.x + 128, 256 * c.y + 128) + 416;
        let id = spawn(&mut w, Kind::Vehicle, "CART", at(2, 2));
        let o = w.obj_mut(id).unwrap();
        o.dest = Some(Target::Cell(c));
        if let Some(d) = o.drive.as_mut() {
            d.track = Some(glasnost_sim::drive::Track {
                d: 2,
                e: None,
                target: (256 * c.x + 128, 256 * c.y + 128),
                target_cell: c,
                height: z,
                next: 0,
                last: (256 * c.x - 128, 256 * c.y + 128),
                cells: vec![c],
                crush: false,
                exit: false,
                bunker: None,
            });
        }
        assert_eq!(
            glasnost_sim::bridges::blow::heads_to(&w, id, c, z),
            expect,
            "{loco:?}"
        );
    }
}

// spec: S-0511 claim 11 (after the blow-ups, a blown-up cell whose deck record is not empty hits every unit in the other 24 cells of its block whose locomotion heads to P, the cell's centre at floor + 416: walk by its heading point's cell and stored height within 104 leptons; drive by its current track's end cell with its stored track-start height (S-0207 claim 24), 0 for a factory exit track, or its own current height; a destination without a heading point does not count)
#[test]
fn claim_11_units_heading_onto_a_lost_deck() {
    for occupied in [true, false] {
        let mut m = base();
        high_x(&mut m, Family::Bridge, 0, 8, &[]);
        let mut w = game(&ruleset(), &m);
        let c = at(3, 0);
        let deck_z = w.surface_at(256 * c.x + 128, 256 * c.y + 128) + 416;
        // A walker whose heading point is P, and one whose heading point is
        // c at its floor height.
        let high = spawn(&mut w, Kind::Infantry, "WALKER", at(3, 2));
        w.obj_mut(high).unwrap().walk.as_mut().unwrap().target = Some(centre(c, deck_z - 50));
        let low = spawn(&mut w, Kind::Infantry, "WALKER", at(4, 2));
        w.obj_mut(low).unwrap().walk.as_mut().unwrap().target = Some(centre(c, deck_z - 416));
        // A vehicle whose destination is c, with no track.
        let car = spawn(&mut w, Kind::Vehicle, "CART", at(2, 2));
        w.obj_mut(car).unwrap().dest = Some(Target::Cell(c));
        // A deck vehicle on a track that ends in c; its destination is elsewhere.
        let tank = spawn(&mut w, Kind::Vehicle, "CART", at(2, 0));
        onto_deck(&mut w, tank);
        let track_to_c = |height: i32| glasnost_sim::drive::Track {
            d: 2,
            e: None,
            target: (256 * c.x + 128, 256 * c.y + 128),
            target_cell: c,
            height,
            next: 0,
            last: (256 * c.x - 128, 256 * c.y + 128),
            cells: vec![c],
            crush: false,
            exit: false,
            bunker: None,
        };
        {
            let o = w.obj_mut(tank).unwrap();
            o.dest = Some(Target::Cell(at(6, 0)));
            let z = o.pos.z;
            o.drive.as_mut().unwrap().track = Some(track_to_c(z));
        }
        // A ground vehicle under the deck whose track began at floor height
        // heading to c: neither test holds.
        let under = spawn(&mut w, Kind::Vehicle, "CART", at(2, 0));
        {
            let o = w.obj_mut(under).unwrap();
            let z = o.pos.z;
            o.drive.as_mut().unwrap().track = Some(track_to_c(z));
        }
        // A descending vehicle: its stored track-start height is within 104
        // of P, its current height more than 104 below (S-0207 claim 24).
        let down = spawn(&mut w, Kind::Vehicle, "CART", at(4, 2));
        {
            let o = w.obj_mut(down).unwrap();
            assert!(deck_z - o.pos.z > 104);
            o.drive.as_mut().unwrap().track = Some(track_to_c(deck_z - 50));
        }
        // Factory exit tracks to c store heading height 0 (S-0207 claim 24):
        // one begun on the deck still meets the current-height track test;
        // one begun at deck height that has since descended meets neither.
        // (`claim_5_exit_points_store_height_zero` in claims_s0417.rs
        // tests that the exit instruction stores 0.)
        let exit_track = glasnost_sim::drive::Track {
            exit: true,
            bunker: None,
            ..track_to_c(0)
        };
        let exit_up = spawn(&mut w, Kind::Vehicle, "CART", at(1, 0));
        onto_deck(&mut w, exit_up);
        w.obj_mut(exit_up).unwrap().drive.as_mut().unwrap().track = Some(exit_track.clone());
        let exit_down = spawn(&mut w, Kind::Vehicle, "CART", at(1, 1));
        assert!(deck_z - w.obj(exit_down).unwrap().pos.z > 104);
        w.obj_mut(exit_down).unwrap().drive.as_mut().unwrap().track = Some(exit_track);
        if occupied {
            // The high walker's reservation is in c's deck record.
            let h = w.obj(high).unwrap().owner.unwrap();
            w.cells.mark_infantry(c, Layer::Deck, 0, high, h);
        }
        bridges::attempts(&mut w, c, 2);
        assert_eq!(
            alive(&w, exit_up),
            !occupied,
            "an exit track meets the current-height test"
        );
        assert!(alive(&w, exit_down), "an exit track's stored height is 0");
        assert_eq!(alive(&w, high), !occupied, "occupied {occupied}");
        assert_eq!(alive(&w, tank), !occupied, "occupied {occupied}");
        assert!(alive(&w, low), "a heading point at floor height");
        assert!(alive(&w, under), "a floor-start track under the deck");
        assert_eq!(
            alive(&w, down),
            !occupied,
            "the stored heading height qualifies"
        );
        assert!(alive(&w, car), "a destination is not a heading point");
    }
}

// spec: S-0511 claim 12 (a successful attempt names the lost section's cell: the nearby link stops being enabled; connectivity is updated)
#[test]
fn claim_12_connectivity() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut w = game(&ruleset(), &m);
    assert_eq!(w.cells.links.len(), 1);
    assert!(w.cells.links[0].enabled);
    bridges::attempt(&mut w, at(3, 0));
    assert!(w.cells.links[0].enabled, "a failed attempt names no cell");
    w.regions.dirty = false;
    bridges::attempt(&mut w, at(3, 0));
    assert!(!w.cells.links[0].enabled);
    assert!(w.regions.dirty);
}

// ---------------------------------------------------------------------
// Claims 13 to 15: low strips.

// spec: S-0511 claim 13 (an attempt at any cross-cell affects the strip's centre; intact → D₀, fails; damaged → destroyed, succeeds; a whole end piece → its damaged position)
#[test]
fn claim_13_low_strip_attempts() {
    let mut m = base();
    low_x(&mut m, Family::Bridge, 0, 4, |x| match x {
        0 => Role::S,
        4 => Role::E,
        _ => Role::I(0),
    });
    let mut w = game(&ruleset(), &m);
    assert!(!bridges::attempt(&mut w, at(2, -1)));
    for dy in [-1, 0, 1] {
        assert_eq!(role(&w, at(2, dy)), Some(Role::D(0)));
    }
    assert!(bridges::attempt(&mut w, at(2, 1)));
    for dy in [-1, 0, 1] {
        assert_eq!(role(&w, at(2, dy)), Some(Role::X));
    }
    assert!(!bridges::attempt(&mut w, at(4, 0)));
    assert_eq!(role(&w, at(4, 0)), Some(Role::E2));
    assert!(
        !bridges::attempt(&mut w, at(4, 0)),
        "a damaged end piece: no change"
    );
    assert_eq!(role(&w, at(4, 0)), Some(Role::E2));
    assert!(!bridges::attempt(&mut w, at(0, 0)));
    assert_eq!(role(&w, at(0, 0)), Some(Role::S2));
}

// spec: S-0511 claim 14 (neighbour re-evaluation: s = a + b; along x, a from increasing x and b from decreasing x; the new piece by s)
#[test]
fn claim_14_neighbour_re_evaluation() {
    let mut m = base();
    low_x(&mut m, Family::Bridge, 0, 4, |_| Role::I(0));
    let mut w = game(&ruleset(), &m);
    bridges::attempt(&mut w, at(2, 0));
    assert_eq!(
        role(&w, at(1, 0)),
        Some(Role::I(5)),
        "s = 1 (D₀ on the increasing side)"
    );
    assert_eq!(
        role(&w, at(3, 0)),
        Some(Role::I(4)),
        "s = 4 (D₀ on the decreasing side)"
    );
    bridges::attempt(&mut w, at(2, 0));
    assert_eq!(role(&w, at(1, 0)), Some(Role::D(2)), "s = 2 (X)");
    assert_eq!(role(&w, at(3, 0)), Some(Role::D(1)), "s = 8 (X)");
    assert_eq!(role(&w, at(0, 0)), Some(Role::I(0)), "no further spread");
}

// spec: S-0511 Edge cases (quirks.low_strip_regress off: a damaged strip never moves to the intact class; on: it may)
#[test]
fn claim_14_low_strip_regress() {
    for regress in [true, false] {
        let mut m = base();
        // x = 2 damaged D₁, its increasing neighbour x = 3 I₄ gives a = 1, nothing on the decreasing side: s = 1 → I₅.
        low_x(&mut m, Family::Bridge, 1, 4, |x| match x {
            2 => Role::D(1),
            3 => Role::I(0),
            _ => Role::I(0),
        });
        let mut r = ruleset();
        r.globals.quirks.low_strip_regress = regress;
        let mut w = game(&r, &m);
        // Damage x = 3 to D₀: x = 2 is re-evaluated with a = 1 (D₀), b = 0 (I₀ at x = 1): s = 1, I₅.
        bridges::attempt(&mut w, at(3, 0));
        let want = if regress { Role::I(5) } else { Role::D(1) };
        assert_eq!(role(&w, at(2, 0)), Some(want), "regress {regress}");
    }
}

// spec: S-0511 Edge cases (quirks.low_bridge_wrong_axis_recheck: a strip of the other axis next to a changed strip is re-evaluated with on, left alone with off)
#[test]
fn claim_14_wrong_axis_recheck() {
    for recheck in [true, false] {
        let mut m = base();
        low_x(&mut m, Family::Bridge, 0, 2, |_| Role::I(0));
        // A piece along y in the cell one step east of the x strip at x = 2.
        let cell = m.cells.get_mut(at(3, 0)).unwrap();
        cell.overlay = pieces::low_overlay(Family::Bridge, Line::Y, Role::I(0));
        cell.overlay_data = 1;
        let mut r = ruleset();
        r.globals.quirks.low_bridge_wrong_axis_recheck = recheck;
        let mut w = game(&r, &m);
        bridges::attempt(&mut w, at(2, 0));
        let changed = role(&w, at(3, 0)) != Some(Role::I(0));
        assert_eq!(changed, recheck, "recheck {recheck}");
    }
}

// spec: S-0511 claim 15 (kill check: a ground unit that could not enter the changed cell dies; a jumpjet type dies when the cell became destroyed; damage alone harms no ground unit that may enter)
#[test]
fn claim_15_kill_check() {
    let mut m = base();
    low_x(&mut m, Family::Bridge, 0, 4, |_| Role::I(0));
    let mut w = game(&ruleset(), &m);
    let car = spawn(&mut w, Kind::Vehicle, "CART", at(2, 1));
    let jet = spawn(&mut w, Kind::Vehicle, "JET", at(2, -1));
    bridges::attempt(&mut w, at(2, 0));
    assert!(
        alive(&w, car) && alive(&w, jet),
        "damaged road is enterable"
    );
    bridges::attempt(&mut w, at(2, 0));
    assert!(!alive(&w, car), "destroyed: the car cannot enter water");
    assert!(!alive(&w, jet), "a jumpjet type dies in a destroyed cell");
    // A strip's state changes take no draw and create no animation.
    let mut w = game(&ruleset(), &m);
    let s0 = w.rng.clone();
    w.effects.clear();
    bridges::attempts(&mut w, at(2, 0), 2);
    assert_eq!(w.rng.raw(), s0.clone().raw());
    assert!(!w.effects.iter().any(|e| matches!(e, Effect::Anim { .. })));
}

// ---------------------------------------------------------------------
// Claims 16 to 21: the hut.

fn hut(w: &mut World, c: CellXY) -> ObjectId {
    spawn(w, Kind::Building, "HUT", c)
}

// spec: S-0511 claim 16 (a high bridge: up to three attempts at the start piece and three at the cell before the far endpoint; the isolated runs follow, so the whole span falls)
#[test]
fn claim_16_hut_demolishes_a_high_bridge() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut w = game(&ruleset(), &m);
    let h = hut(&mut w, at(3, 3));
    hut::demolish(&mut w, h);
    for x in 0..8 {
        assert_eq!(data(&w, at(x, 0)), None, "x = {x}");
    }
    assert!(!w.cells.links[0].enabled);
}

// spec: S-0511 claim 16 (a low bridge: up to four strips from the centre displaced by −trunc((w − e) / 2), toward increasing coordinate when e ≥ w, each with three low-form animations and up to three attempts)
#[test]
fn claim_16_hut_demolishes_a_low_bridge() {
    let mut m = base();
    low_x(&mut m, Family::Bridge, 0, 9, |_| Role::I(0));
    let mut r = ruleset();
    r.sim.general.bridge_explosions = Some(vec!["BOOM".into()]);
    let mut w = game(&r, &m);
    // The hut's block reaches x = 0 … 2 at most; the first piece is x = 0 (w = 1, e = 10): first strip x = 0 + 4 = 4.
    let h = hut(&mut w, at(2, 3));
    w.effects.clear();
    hut::demolish(&mut w, h);
    let destroyed: Vec<i32> = (0..=9)
        .filter(|&x| role(&w, at(x, 0)) == Some(Role::X))
        .collect();
    assert_eq!(destroyed, vec![4, 5, 6, 7]);
    let booms = w
        .effects
        .iter()
        .filter(|e| matches!(e, Effect::Anim { name, .. } if name == "BOOM"))
        .count();
    assert_eq!(booms, 12, "three low-form animations per strip");
}

// spec: S-0511 claim 16 (quirks.bridge_family_by_block: a wooden piece in the block chooses the wooden family; off: the family of the bridge the search finds)
#[test]
fn claim_16_family_by_block() {
    for by_block in [true, false] {
        let mut m = base();
        high_x(&mut m, Family::Bridge, 0, 8, &[]);
        // A wooden-role tile in the hut's block.
        put_tile(&mut m, at(4, 4), tile(Family::Wooden, 16), 0, 0);
        let mut r = ruleset();
        r.globals.quirks.bridge_family_by_block = by_block;
        let mut w = game(&r, &m);
        let h = hut(&mut w, at(3, 3));
        hut::demolish(&mut w, h);
        assert_eq!(
            data(&w, at(4, 0)).is_none(),
            !by_block,
            "by block {by_block}: the bridge-role sections fall only without the block family"
        );
    }
}

fn engineer(w: &mut World, c: CellXY) -> ObjectId {
    spawn(w, Kind::Infantry, "ENGI", c)
}

// spec: S-0511 claim 17 (the engineer's entry: the hut's entered-by raise, the bridge-repaired announcement and the repair sound, the repair, and the engineer leaves play; quirks.repair_consumes_engineer_always)
#[test]
fn claim_17_engineer_enters_the_hut() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut r = ruleset();
    r.sim.general.repair_bridge_sound = Some("FIXED".into());
    let mut w = game(&r, &m);
    let h = hut(&mut w, at(3, 3));
    let e = engineer(&mut w, at(3, 2));
    bridges::attempts(&mut w, at(3, 0), 2);
    assert_eq!(data(&w, at(3, 0)), None);
    w.effects.clear();
    assert!(hut::engineer_entered_hut(&mut w, e, h));
    assert!(w.obj(e).is_none(), "the engineer leaves play");
    assert_eq!(data(&w, at(3, 0)), Some(0), "repaired");
    let sounds: Vec<Sound> = w
        .effects
        .iter()
        .filter_map(|x| match x {
            Effect::Sound { sound, .. } => Some(*sound),
            _ => None,
        })
        .collect();
    assert_eq!(sounds, vec![Sound::BridgeRepaired, Sound::RepairBridge]);
    // An intact bridge: used up with the quirk on, refused with it off.
    for always in [true, false] {
        let mut r = ruleset();
        r.globals.quirks.repair_consumes_engineer_always = always;
        let mut w = game(&r, &m);
        let h = hut(&mut w, at(3, 3));
        let e = engineer(&mut w, at(3, 2));
        assert_eq!(hut::engineer_entered_hut(&mut w, e, h), always);
        assert_eq!(w.obj(e).is_none(), always);
    }
}

// spec: S-0511 claim 18 (the start piece: the nearest start ramp or pier on the start-side ray; a pier is restored with the bridge two steps toward its start side; quirks.repair_walk_unbounded)
#[test]
fn claim_18_which_bridge_is_repaired() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 10, &[4]);
    let mut w = game(&ruleset(), &m);
    let h = hut(&mut w, at(7, 3));
    let e = engineer(&mut w, at(7, 2));
    // Collapse the pier and lose sections on both sides of it.
    bridges::attempts(&mut w, at(4, 0), 2);
    bridges::attempts(&mut w, at(2, 0), 2);
    bridges::attempts(&mut w, at(7, 0), 2);
    assert_eq!(pier_k(&w, at(4, 0)), Some(4));
    assert!(hut::engineer_entered_hut(&mut w, e, h));
    assert_eq!(pier_k(&w, at(4, 0)), Some(0), "the pier is restored");
    for x in [0, 1, 2, 3, 5, 6, 7, 8, 9] {
        assert_eq!(data(&w, at(x, 0)), Some(0), "x = {x} has an intact section");
    }
    assert!(w.cells.links.iter().all(|l| l.enabled));
}

// spec: S-0511 Edge cases (quirks.repair_walk_unbounded off: the walk toward the start side stops at the first cell that is neither a high-bridge cell nor bridge-body marked)
#[test]
fn claim_18_repair_walk_bound() {
    for unbounded in [true, false] {
        let mut m = base();
        high_x(&mut m, Family::Bridge, 0, 8, &[]);
        let mut r = ruleset();
        r.globals.quirks.repair_walk_unbounded = unbounded;
        let mut w = game(&r, &m);
        bridges::attempts(&mut w, at(3, 0), 2);
        // Clear the deck marks and the body of x = 1 … 2, as a gap on the start side.
        for x in [1, 2] {
            let i = w.cells.idx(at(x, 0)).unwrap();
            w.cells.deck[i] = false;
            w.cells.bridge_body[i] = false;
        }
        hut::repair(&mut w, at(5, 0));
        assert_eq!(
            data(&w, at(3, 0)).is_some(),
            unbounded,
            "unbounded {unbounded}"
        );
    }
}

// spec: S-0511 claim 19 (sections of the family with data 0 along x on every eligible cell, marking four cells and clearing the body mark; no section on a cell of slope above 4; a collapsed pier's column rises by 4 with quirks.pier_repair_partial_height, the whole tile goes back to its levels without)
#[test]
fn claim_19_high_bridge_repair() {
    for partial in [true, false] {
        let mut m = base();
        high_x(&mut m, Family::Bridge, 0, 10, &[4]);
        // A cell of the far span whose tile has slope shape 5 (it keeps its section until lost).
        put_tile(&mut m, at(8, 0), STEEP, 0, 0);
        // A pier cell off the column (sub-tile 1, row 0) at its own level.
        put_tile(&mut m, at(5, -1), tile(Family::Bridge, M1), 1, 6);
        let mut r = ruleset();
        r.globals.quirks.pier_repair_partial_height = partial;
        let mut w = game(&r, &m);
        bridges::attempts(&mut w, at(4, 0), 2);
        bridges::attempts(&mut w, at(7, 0), 2);
        assert!(
            bridges::attempt(&mut w, at(8, 0)),
            "open on its start side: one attempt"
        );
        assert_eq!(w.cells.cell(at(5, -1)).unwrap().level, 0);
        hut::repair(&mut w, at(7, 0));
        for x in [5, 6, 7, 9] {
            assert_eq!(data(&w, at(x, 0)), Some(0), "x = {x}");
            assert!(w.cells.is_bridge(at(x, -1)));
            assert!(!w.cells.body(at(x, 0)));
        }
        assert_eq!(data(&w, at(8, 0)), None, "slope 5: no section");
        assert_eq!(pier_k(&w, at(4, 0)), Some(0));
        assert_eq!(w.cells.cell(at(4, 0)).unwrap().level, 4);
        assert_eq!(
            w.cells.cell(at(5, -1)).unwrap().level,
            if partial { 0 } else { 6 },
            "partial {partial}"
        );
    }
}

// spec: S-0511 claim 18 (r13, Q-3046: Repair extent: a selected start-piece pier is restored only if it is collapsed and otherwise keeps its state; either way the bridge from two steps toward its start side and its own far-side span are repaired) and claim 19 (Connectivity: only a restored pier names a cell)
#[test]
fn claim_18_r13_a_damaged_start_piece_pier_keeps_its_state() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 10, &[4]);
    // No start ramp: the walk from two steps toward the start side selects
    // nothing, so only the pier's own far-side span is repaired.
    put_tile(&mut m, at(-1, 0), 0, 0, 0);
    put_tile(&mut m, at(0, 0), 0, 0, 0);
    let mut w = game(&ruleset(), &m);
    // State 3; its damage notice chips the far span's first section.
    assert!(!bridges::attempt(&mut w, at(4, 0)));
    assert_eq!(pier_k(&w, at(4, 0)), Some(3));
    assert_ne!(data(&w, at(5, 0)), Some(0));
    hut::repair(&mut w, at(7, 0));
    for c in [at(4, -1), at(4, 0), at(4, 1), at(5, 0)] {
        assert_eq!(pier_k(&w, c), Some(3), "{c:?}: not restored");
        assert_eq!(w.cells.cell(c).unwrap().level, 4, "{c:?}: not raised");
    }
    for x in [5, 6, 7, 8, 9] {
        assert_eq!(data(&w, at(x, 0)), Some(0), "x = {x}: the far span");
    }
}

// spec: S-0511 claim 20 (the whole run: damaged, destroyed, I₄ and I₅ become Iᵣ with r from the map-generator stream, F4; a damaged end piece becomes whole; the scenario stream takes no draw)
#[test]
fn claim_20_low_bridge_repair() {
    let mut m = base();
    low_x(&mut m, Family::Bridge, 0, 6, |x| match x {
        0 => Role::S2,
        1 => Role::I(4),
        2 => Role::D(1),
        3 => Role::X,
        4 => Role::I(2),
        5 => Role::I(5),
        _ => Role::E,
    });
    let mut w = game(&ruleset(), &m);
    let s0 = w.rng.clone();
    let mut g = w.map_rng.clone().unwrap();
    hut::repair(&mut w, at(1, 2));
    assert_eq!(w.rng.raw(), s0.clone().raw(), "no scenario-stream draw");
    let mut want = Vec::new();
    for _ in 0..4 {
        // F4 with the invented stream: the accepted value of each draw.
        let r = loop {
            let v = i128::from(g.raw()) * 4 / i128::from(u32::MAX);
            if v <= 3 {
                break v as u8;
            }
        };
        want.push(r);
    }
    let got: Vec<Option<Role>> = (0..=6).map(|x| role(&w, at(x, 0))).collect();
    assert_eq!(
        got,
        vec![
            Some(Role::S),
            Some(Role::I(want[0])),
            Some(Role::I(want[1])),
            Some(Role::I(want[2])),
            Some(Role::I(2)),
            Some(Role::I(want[3])),
            Some(Role::E),
        ]
    );
    for dy in [-1, 1] {
        assert_eq!(role(&w, at(3, dy)), role(&w, at(3, 0)), "the whole strip");
    }
}

// spec: S-0511 claim 21 (a low candidate: its own row's contiguous run must hold a destroyed piece; damage alone does not offer repair)
#[test]
fn claim_21_low_candidate() {
    let mut m = base();
    low_x(&mut m, Family::Bridge, 0, 4, |_| Role::I(0));
    let mut w = game(&ruleset(), &m);
    let h = hut(&mut w, at(2, 3));
    assert!(!hut::repair_offered(&w, h));
    bridges::attempt(&mut w, at(2, 0));
    assert!(!hut::repair_offered(&w, h), "damage alone");
    bridges::attempt(&mut w, at(2, 0));
    assert!(hut::repair_offered(&w, h), "destroyed");
}

/// The cursor fixture: a high bridge along x from 0 to 8 (its link runs
/// from (−1, 0) to (8, 0)) and a lone start-ramp tile (position 0 within the
/// role) at sub-tile 15 in (1, 1). That tile's origin is (−2, −2), so its
/// probe (1, 2) from the origin is (−1, 0), the link's first end. With
/// `chain`, a low span along y from (8, −5) ends at (8, 0) as well.
fn cursor_map(chain: bool) -> Map {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    put_tile(&mut m, at(1, 1), tile(Family::Bridge, START_X), 15, 0);
    if chain {
        put_tile(&mut m, at(8, -5), tile(Family::Bridge, START_Y), 7, 0);
    }
    m
}

// spec: S-0511 claim 21 (the last candidate of the hut's 5 × 5 block; the probe table from the placed tile's origin, S-0307 claim 23; the initial traversal cell is the probe; an enabled link ending there is crossed and the next probe is one cell past its other end; no link near a probe or a disabled link decides)
#[test]
fn claim_21_high_link_traversal() {
    // The block of (1, 3) holds one candidate, (1, 1).
    let mut w = game(&ruleset(), &cursor_map(false));
    let h = hut(&mut w, at(1, 3));
    assert!(
        !hut::repair_offered(&w, h),
        "crossing the intact link to (8, 0): no link near (9, 0)"
    );
    bridges::attempts(&mut w, at(3, 0), 2);
    assert!(!w.cells.links[0].enabled);
    assert!(
        hut::repair_offered(&w, h),
        "the disabled link offers repair"
    );

    // The low span (8, −5)–(8, 0) is disabled; it is not near the first
    // probe, but it is the only link near (9, 0).
    let mut w = game(&ruleset(), &cursor_map(true));
    let ls = &w.cells.links;
    assert_eq!(ls.len(), 2);
    assert!(ls.iter().any(|l| l.first == at(8, -5) && !l.enabled));
    assert!(ls.iter().any(|l| l.first == at(-1, 0) && l.enabled));
    let h = hut(&mut w, at(1, 3));
    assert!(
        hut::repair_offered(&w, h),
        "the intact link is crossed, then the disabled one offers repair"
    );
}

// spec: S-0511 claim 21 (the last qualifying cell in increasing y, then x; a tile at positions 6 to 15 has no probe and gives no repair cursor even when an earlier candidate would)
#[test]
fn claim_21_last_candidate_decides() {
    let mut m = cursor_map(false);
    // A pier tile (position 6) after (1, 1) in the block of (1, 3).
    put_tile(&mut m, at(2, 4), tile(Family::Bridge, M1), 0, 0);
    let mut w = game(&ruleset(), &m);
    let h = hut(&mut w, at(1, 3));
    bridges::attempts(&mut w, at(3, 0), 2);
    assert!(!w.cells.links[0].enabled);
    assert!(!hut::repair_offered(&w, h));
}

// spec: S-0511 claim 21 and Edge cases (quirks.bridge_cursor_unrelated_link: on, an enabled first link with neither end at the traversal cell offers repair; off, such links are skipped and no endpoint link means no repair cursor)
#[test]
fn claim_21_unrelated_link_quirk() {
    // The block of (0, 2) ends on the start ramp's sub-tile 8 cell (0, 0):
    // origin (0, −2), probe (1, 0), which is not an end of the intact link.
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut w = game(&ruleset(), &m);
    let h = hut(&mut w, at(0, 2));
    assert!(w.cells.links[0].enabled);
    assert!(hut::repair_offered(&w, h), "quirk on: the intact bridge");
    let mut r = ruleset();
    r.globals.quirks.bridge_cursor_unrelated_link = false;
    let mut w = game(&r, &m);
    let h = hut(&mut w, at(0, 2));
    assert!(!hut::repair_offered(&w, h), "quirk off");
}

// spec: S-0511 State and ordering (bridge state: overlays, alternate looks, pier levels, falls and the map-generator stream survive a save and load with the same state hash)
#[test]
fn bridge_state_save_round_trip() {
    use glasnost_sim::save;
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 10, &[4]);
    let mut w = game(&ruleset(), &m);
    let top = spawn(&mut w, Kind::Vehicle, "CART", at(7, 0));
    onto_deck(&mut w, top);
    bridges::attempts(&mut w, at(4, 0), 2);
    bridges::attempts(&mut w, at(7, 0), 2);
    bridges::attempts(&mut w, at(9, 0), 2);
    glasnost_sim::tick::step(&mut w);
    assert!(w.obj(top).unwrap().fall.falling);
    assert!(!w.cells.alternate.is_empty() && !w.cells.pier_levels.is_empty());
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    for _ in 0..40 {
        glasnost_sim::tick::step(&mut w);
        glasnost_sim::tick::step(&mut l);
    }
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
}

// ---------------------------------------------------------------------
// Sim issues S6 (#420): reservations dropped from a lost deck (#165).

fn walker_mark(w: &World, c: CellXY, l: Layer, id: ObjectId) -> bool {
    w.cells
        .occ(c, l)
        .unwrap()
        .marks
        .iter()
        .flatten()
        .any(|m| matches!(m.by, glasnost_sim::cells::Marker::Infantry(who, _) if who == id))
}

// spec: S-0511 claim 9 (movement reservations are cleared at the locomotor's heading point, at that point's stored height: a moving walker's deck reservation in a lost section's cell)
#[test]
fn claim_9_moving_walker_drop_clears_its_deck_reservation() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut w = game(&ruleset(), &m);
    let c = at(3, 0);
    let deck_z = w.surface_at(256 * c.x + 128, 256 * c.y + 128) + 416;
    let walker = spawn(&mut w, Kind::Infantry, "WALKER", c);
    {
        let h = w.obj(walker).unwrap().owner.unwrap();
        let ground = *w.cells.occ(c, Layer::Ground).unwrap();
        let p = (0..5).find(|&p| ground.taken(p)).unwrap();
        w.cells
            .clear_position(c, Layer::Ground, p, glasnost_sim::objects::occ_quirks(&w));
        w.cells.leave(c, Layer::Ground, walker);
        w.cells.enter(c, Layer::Deck, walker, false);
        w.cells.mark_infantry(c, Layer::Deck, p, walker, h);
        let o = w.obj_mut(walker).unwrap();
        o.layer = Layer::Deck;
        o.pos.z += 416;
        o.walk.as_mut().unwrap().target = Some(centre(c, deck_z));
    }
    assert!(bridges::attempts(&mut w, c, 2));
    let o = w.obj(walker).unwrap();
    assert!(o.fall.falling && o.layer == Layer::Ground);
    assert!(
        !walker_mark(&w, c, Layer::Deck, walker),
        "the deck reservation is cleared at the drop"
    );
    for _ in 0..60 {
        glasnost_sim::tick::step(&mut w);
    }
    assert!(!alive(&w, walker));
    hut::repair(&mut w, c);
    assert!(w.cells.is_bridge(c), "the deck is rebuilt");
    assert!(
        !walker_mark(&w, c, Layer::Deck, walker),
        "no mark of a dead unit after the repair"
    );
}

// spec: S-0511 claim 9 and S-0207 claim 24 (a drive vehicle's heading-point reservation is cleared in the record of the track's stored height, also when the heading cell has just lost its deck)
#[test]
fn claim_9_moving_vehicle_drop_clears_its_deck_reservation() {
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 8, &[]);
    let mut w = game(&ruleset(), &m);
    let c = at(3, 0);
    let deck_z = w.surface_at(256 * c.x + 128, 256 * c.y + 128) + 416;
    let cart = spawn(&mut w, Kind::Vehicle, "CART", c);
    onto_deck(&mut w, cart);
    {
        let o = w.obj_mut(cart).unwrap();
        let last = (o.pos.x, o.pos.y);
        let d = o.drive.as_mut().unwrap();
        d.claims = vec![(c, Layer::Deck)];
        d.between_steps = false;
        d.track = Some(glasnost_sim::drive::Track {
            d: 2,
            e: None,
            target: (256 * c.x + 128, 256 * c.y + 128),
            target_cell: c,
            height: deck_z,
            next: 0,
            last,
            cells: vec![c],
            crush: false,
            exit: false,
            bunker: None,
        });
    }
    assert_eq!(w.cells.occ(c, Layer::Deck).unwrap().vehicles, 1);
    assert_eq!(w.cells.occ(c, Layer::Ground).unwrap().vehicles, 0);
    assert!(bridges::attempts(&mut w, c, 2));
    let o = w.obj(cart).unwrap();
    assert!(o.fall.falling && o.layer == Layer::Ground);
    assert_eq!(
        w.cells.occ(c, Layer::Deck).unwrap().vehicles,
        0,
        "the deck flag is cleared at the drop"
    );
    assert_eq!(
        w.cells.occ(c, Layer::Ground).unwrap().vehicles,
        0,
        "the ground record is not touched"
    );
    for _ in 0..60 {
        glasnost_sim::tick::step(&mut w);
    }
    assert!(!alive(&w, cart));
    hut::repair(&mut w, c);
    assert!(w.cells.is_bridge(c));
    assert_eq!(
        w.cells.occ(c, Layer::Deck).unwrap().vehicles,
        0,
        "no flag of a dead vehicle after the repair"
    );
}

// ---------------------------------------------------------------------
// Sim issues S6 (#420): a pier whose reference cell does not hold it
// (#166), connectivity after a pier collapse with no link nearby (#167).

fn count_tile(w: &World, t: i32) -> usize {
    w.cells.grid.iter().filter(|(_, x)| x.tile == t).count()
}

fn tile_at(w: &World, c: CellXY) -> i32 {
    w.cells.cell(c).unwrap().tile
}

// spec: S-0511 Terms (the reference cell is the pier's cell at sub-tile 4; a different tile laid over part of a pier is not changed) and claim 6 (r13, Q-3046: A reference cell that does not hold the pier: a reference position painted over, the attempt changes nothing, fails and sends no notice); scenario B17 a, our engine's expected result
#[test]
fn claim_6_a_painted_over_reference_cell_floods_nothing() {
    let mut m = base();
    let t = tile(Family::Bridge, M1);
    put_tile(&mut m, at(4, -1), t, 0, 4);
    put_tile(&mut m, at(4, 1), t, 8, 4);
    put_tile(&mut m, at(5, 0), t, 5, 4);
    // at(4, 0), the reference position, keeps ground tile 0.
    let mut w = game(&ruleset(), &m);
    let ground = count_tile(&w, 0);
    assert!(!bridges::attempt(&mut w, at(4, -1)));
    assert_eq!(count_tile(&w, 0), ground, "no ground cell rewritten");
    assert_eq!(tile_at(&w, at(-20, 15)), 0, "ground far from the pier");
    assert_eq!(
        pier_k(&w, at(4, -1)),
        Some(0),
        "the hit cell keeps its state"
    );
}

// spec: S-0511 claim 6 (a collapse lowers the pier's own cells; r13, Q-3046: A reference cell that does not hold the pier, in state 3 too: no collapse and no success); scenario B17 a3, our engine's expected result
#[test]
fn claim_6_a_painted_over_reference_cell_in_state_3_collapses_nothing() {
    let mut m = base();
    let t = tile(Family::Bridge, M1);
    put_tile(&mut m, at(4, -1), t + 3, 0, 4);
    put_tile(&mut m, at(4, 0), 0, 0, 4);
    let mut w = game(&ruleset(), &m);
    let far = at(-20, 15);
    w.cells.grid.get_mut(far).unwrap().level = 9;
    assert!(
        !bridges::attempt(&mut w, at(4, -1)),
        "no pier at the reference cell: no success"
    );
    assert_eq!(
        w.cells.cell(far).unwrap().level,
        9,
        "far ground not lowered"
    );
    assert_eq!(count_tile(&w, t + 4), 0, "no collapsed pier tile written");
    assert_eq!(pier_k(&w, at(4, -1)), Some(3));
}

// spec: S-0511 claim 6 (a collapsed pier, state 4, is not attempted; the collapse lowers it once; r13, Q-3046: a cut-off part in another state than the reference cell does nothing, fails and warns); scenario B17 b, our engine's expected result
#[test]
fn claim_6_a_cut_off_part_does_not_restore_or_collapse_again() {
    let mut m = base();
    let t = tile(Family::Bridge, M1);
    put_tile(&mut m, at(4, -1), t, 0, 4);
    put_tile(&mut m, at(4, 0), t, 4, 4);
    put_tile(&mut m, at(5, 0), t, 5, 4);
    // at(4, 1) (sub-tile 8) is painted over with ground; at(4, 2), the
    // tile's sub-tile 12, is cut off from the reference cell's component.
    put_tile(&mut m, at(4, 2), t, 12, 4);
    let mut w = game(&ruleset(), &m);
    let r = at(4, 0);
    assert!(!bridges::attempt(&mut w, r));
    assert!(bridges::attempt(&mut w, r));
    assert_eq!(pier_k(&w, r), Some(4));
    assert_eq!(w.cells.cell(r).unwrap().level, 0);
    assert_eq!(pier_k(&w, at(4, 2)), Some(0), "the cut-off cell stays");
    assert!(!bridges::attempt(&mut w, at(4, 2)));
    assert_eq!(pier_k(&w, r), Some(4), "the collapsed pier stays collapsed");
    assert!(!bridges::attempt(&mut w, r), "no second collapse");
    assert_eq!(w.cells.cell(r).unwrap().level, 0, "lowered once");
}

// spec: S-0511 claim 6 (r13, Q-3046: A received notice uses no reference cell: it reads the state of the cell it arrives at and replaces that cell's component, so a cut-off part changes only through notices that arrive at its own cells) and Terms (the affected tile cells of a replacement by a notice: the component around the cell the notice arrives at)
#[test]
fn claim_6_r13_a_notice_at_a_cut_off_part_changes_that_part() {
    use glasnost_sim::bridges::high::{Notice, Side};
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 9, &[4]);
    let t = tile(Family::Bridge, M1);
    // at(4, 1) painted over with ground; at(4, 2), sub-tile 12, cut off.
    put_tile(&mut m, at(4, 1), 0, 0, 4);
    put_tile(&mut m, at(4, 2), t, 12, 6);
    let mut w = game(&ruleset(), &m);
    let mut ctx = Ctx::default();
    high::notice(&mut w, at(4, 2), Notice::Damage, Side::Start, &mut ctx);
    assert_eq!(pier_k(&w, at(4, 2)), Some(1), "the arriving cell's part");
    for c in [at(4, -1), at(4, 0), at(5, 0)] {
        assert_eq!(pier_k(&w, c), Some(0), "{c:?}: the rest of the pier");
    }
}

// spec: S-0511 claim 6 (r13, Q-3046: a collapse from a notice takes its blow-up cells and its height from a centre cell, the arriving cell when its sub-tile is even along x and otherwise the cell one step toward the start side; the three blow-ups are the centre and its two neighbours across, centre first along x; every cell of the replaced component takes the centre's level minus 4; the onward notice goes from the arriving cell)
#[test]
fn claim_6_r13_a_notice_collapse_centres_on_the_arriving_cell() {
    use glasnost_sim::bridges::high::{Notice, Side};
    let mut m = base();
    high_x(&mut m, Family::Bridge, 0, 9, &[4]);
    let t3 = tile(Family::Bridge, M1) + 3;
    put_tile(&mut m, at(4, -1), t3, 0, 4);
    put_tile(&mut m, at(4, 0), t3, 4, 4);
    put_tile(&mut m, at(4, 1), t3, 8, 6);
    put_tile(&mut m, at(5, 0), t3, 5, 4);
    // A pier cell at an odd sub-tile, with no deck, below the span start.
    put_tile(&mut m, at(5, 1), t3, 9, 4);
    let mut w = game(&ruleset(), &m);
    let start = data(&w, at(5, 0));
    let mut ctx = Ctx::default();
    high::notice(&mut w, at(5, 1), Notice::Loss, Side::Start, &mut ctx);
    assert_eq!(pier_k(&w, at(5, 1)), Some(4));
    // The centre is at(4, 1), one step toward the start side.
    assert_eq!(&ctx.blown[..3], &[at(4, 1), at(4, 0), at(4, 2)]);
    for c in [at(4, -1), at(4, 0), at(4, 1), at(5, 0), at(5, 1)] {
        assert_eq!(pier_k(&w, c), Some(4), "{c:?}");
        assert_eq!(w.cells.cell(c).unwrap().level, 2, "{c:?}: 6 − 4");
    }
    assert_eq!(
        data(&w, at(5, 0)),
        start,
        "the onward notice goes from at(5, 1), not from the reference cell"
    );
}

// spec: S-0511 claim 12, S-0212 claims 3 and 5 (a pier collapse with no bridge link within 3 cells: no link changes, but connectivity is brought up to date, so the running game's regions are those a save and load rebuild)
#[test]
fn claim_12_pier_collapse_with_no_link_nearby_updates_the_regions() {
    use glasnost_sim::{regions, save};
    let mut m = base();
    // A plateau at level 4 around the pier.
    for dx in 1..=8 {
        for dy in -3..=3 {
            put_tile(&mut m, at(dx, dy), 0, 0, 4);
        }
    }
    // A pier along x with no ramp or other pier in its row: on no span.
    let t = tile(Family::Bridge, M1);
    put_tile(&mut m, at(4, -1), t, 0, 4);
    put_tile(&mut m, at(4, 0), t, 4, 4);
    put_tile(&mut m, at(4, 1), t, 8, 4);
    put_tile(&mut m, at(5, 0), t, 5, 4);
    let mut w = game(&ruleset(), &m);
    assert!(w.cells.links.is_empty(), "the lone pier is on no span");
    let (r, n) = (at(4, 0), at(3, 0));
    let idx = |w: &World, c: CellXY| w.cells.idx(c).unwrap();
    regions::ensure(&mut w);
    assert_eq!(w.regions.region[idx(&w, r)], w.regions.region[idx(&w, n)]);
    assert!(!bridges::attempt(&mut w, r), "state 0 to state 3");
    assert!(bridges::attempt(&mut w, r), "state 3 collapses");
    assert_eq!(w.cells.cell(r).unwrap().level, 0);
    assert_eq!(w.cells.cell(n).unwrap().level, 4);
    assert!(w.regions.dirty, "connectivity is marked out of date");
    regions::ensure(&mut w);
    let bytes = save::save(&w);
    let l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_ne!(
        w.regions.region[idx(&w, r)],
        w.regions.region[idx(&w, n)],
        "the lowered pier is no longer joined to the plateau"
    );
    assert_eq!(w.regions.region, l.regions.region);
}

// spec: S-0202 claim 13 (the map-generator stream starts an ordinary-map game in its seed-0 state, whatever the game seed)
#[test]
fn map_generator_stream_starts_at_seed_zero() {
    let rules = Arc::new(SimRules::new(&ruleset(), "synthetic-bridges"));
    for seed in [5, 77] {
        let w = glasnost_sim::load::new_game(
            rules.clone(),
            &base(),
            catalogue(),
            &testkit::setup(seed),
        )
        .expect("synthetic map loads");
        let mut got = w.map_rng.clone().unwrap();
        let mut want = glasnost_sim::rng::Rng::new(0);
        assert_eq!(got.raw(), want.raw(), "seed {seed}");
    }
}
