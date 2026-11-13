//! A live building's slot animations among the ground layer's mobiles
//! (#429, #421): a building slot animation is an animation object of its
//! own, in its type's layer, ordered by X + Y of its position plus its
//! slot's y-sort (S-0102 claims 5, 6 and 17). As a depth reader it is
//! painted over by every reader drawn after it (claim 10), so a vehicle
//! behind the animation has to be drawn before it. Invented types and
//! numbers, no game data.

use std::sync::Arc;

use glasnost_convert::kinds::Kind;
use glasnost_map::CellXY;
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::scene::Scene;
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::rules::{Kind as SimKind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::ObjectId;

use super::build::{Inputs, Options, build_parts};
use super::live::{Live, sim_code};
use super::tests::{ART, RULES, art_of, context_full, shp};

const SIZE: i32 = 24;
/// The slot animation's image frames are 3 × 3 (the building's 4 × 4, the
/// vehicle's 2 × 2), so its items can be told apart.
const SLOT_FRAME: u32 = 3;

/// The import: HUTX's art with `keys`, the looping ground-layer animation
/// SLOTL of 4 image frames.
fn import(
    keys: &str,
) -> (
    glasnost_convert::Import,
    glasnost_convert::theater::Catalogue,
    super::build::Assets,
) {
    let rules = format!("{RULES}[Animations]\n0=SLOTL\n");
    let art = format!(
        "{}[SLOTL]\nLayer=ground\nLoopCount=-1\n",
        ART.replacen("[HUTX]\n", &format!("[HUTX]\n{keys}"), 1)
    );
    let mut ctx = context_full(&rules, &art);
    ctx.0
        .arts
        .insert((Kind::Animations, 0), art_of("SLOTL", "slotl.shp"));
    ctx.2.insert(
        b"slotl.shp",
        shp((20, 20), 8, (SLOT_FRAME, SLOT_FRAME), (8, 8)),
    );
    ctx
}

/// The simulation's rules: the synthetic combat ruleset with HUTX, a
/// 2 × 2 building, and CARX, a vehicle.
fn rules() -> Arc<SimRules> {
    let mut r = testkit::combat_ruleset();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("HUTX".into());
    r.registries
        .get_mut("vehicles")
        .unwrap()
        .push("CARX".into());
    r.sim.buildings.insert(
        "HUTX".into(),
        BuildingType {
            strength: Some(400),
            foundation: Some([2, 2]),
            ..Default::default()
        },
    );
    r.sim.vehicles.insert(
        "CARX".into(),
        UnitType {
            strength: Some(200),
            speed: Some(6),
            rot: Some(6),
            locomotion_class: Some(LocomotionClass::Wheel),
            locomotor: Some(Locomotor::Drive),
            ..Default::default()
        },
    );
    Arc::new(SimRules::new(&r, "synthetic-test"))
}

/// Where in the drawing order the items of a scene stand.
#[derive(Debug)]
struct Order {
    /// The indices of the building's slot-animation items.
    slot: Vec<usize>,
    /// The indices of each vehicle's items, in the order given.
    cars: Vec<Vec<usize>>,
}

/// Alpha's own HUTX (art keys `keys`) at the middle cell C, placed in
/// play, and Alpha's vehicles at the cells C + `cars`. With `walls`, the
/// walls' event starts its *special-1* slot (S-0606 claims 6 and 7). The
/// drawing order after a few ticks.
fn order(keys: &str, walls: bool, cars: &[(i32, i32)]) -> Order {
    use glasnost_sim::combat::effects::Effect;
    let (import, catalogue, assets) = import(keys);
    let map = testkit::empty_map(SIZE);
    let rules = rules();
    let setup = testkit::setup(7);
    let mut world = glasnost_sim::load::new_game(rules.clone(), &map, testkit::catalogue(), &setup)
        .expect("the synthetic game loads");
    let c = testkit::mid(SIZE, 0, 0);
    let b = testkit::spawn(&mut world, SimKind::Building, "HUTX", 0, c, 0);
    let ids: Vec<ObjectId> = cars
        .iter()
        .map(|&(dx, dy)| {
            testkit::spawn(
                &mut world,
                SimKind::Vehicle,
                "CARX",
                0,
                CellXY::new(c.x + dx, c.y + dy),
                0,
            )
        })
        .collect();
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let cells = world.cells.grid.clone();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: Some(&rules),
        cells: Some(&cells),
    };
    let (mut built, painter) = build_parts(&inputs, true);
    let mut live = Live::new(
        &inputs,
        &mut built,
        painter,
        (world, rules.clone(), setup, Vec::new()),
    );
    let mut scene: Scene = built.scene;
    live.redraw(&inputs, &mut scene);
    live.step();
    if walls {
        live.world.effects.push(Effect::SpecialAnims {
            building: b,
            slots: [1, 0],
            start: true,
            damaged: false,
        });
        live.after_tick();
    }
    for _ in 0..3 {
        live.step();
    }
    live.redraw(&inputs, &mut scene);
    let at = |code: u32, slot: bool| -> Vec<usize> {
        scene
            .items
            .iter()
            .enumerate()
            .filter(|(_, i)| i.object == Some(code))
            .filter(|(_, i)| !slot || scene.images[i.image as usize].width() == SLOT_FRAME)
            .map(|(n, _)| n)
            .collect()
    };
    Order {
        slot: at(sim_code(b), true),
        cars: ids.iter().map(|&id| at(sim_code(id), false)).collect(),
    }
}

/// #429. spec: S-0102 claims 5, 6 and 17 (a ground-layer slot animation is ordered among the
/// layer's mobiles by X + Y of its position plus its slot's y-sort) and claim 10 (readers never
/// write depth: a vehicle drawn after the animation paints over it)
///
/// HUTX's *active-1* animation stands at the building's anchor with a y-sort of 300, so its key
/// is 256 · (C_x + C_y) + 300. A vehicle at the centre of C + (−1, −1) has the key
/// 256 · (C_x + C_y) − 256: it is behind the animation and drawn before it. A vehicle at the
/// centre of C + (2, 2) has the key 256 · (C_x + C_y) + 1280: in front, drawn after it.
#[test]
fn a_vehicle_behind_a_live_buildings_slot_animation_is_drawn_before_it() {
    let o = order(
        "ActiveAnim=SLOTL\nActiveAnimYSort=300\n",
        false,
        &[(-1, -1), (2, 2)],
    );
    assert!(!o.slot.is_empty(), "the slot animation is drawn: {o:?}");
    assert!(
        o.cars.iter().all(|c| !c.is_empty()),
        "both vehicles are drawn: {o:?}"
    );
    let first_slot = o.slot[0];
    let last_slot = *o.slot.last().unwrap();
    assert!(
        o.cars[0].iter().all(|&i| i < first_slot),
        "the vehicle behind is drawn before the animation: {o:?}"
    );
    assert!(
        o.cars[1].iter().all(|&i| i > last_slot),
        "the vehicle in front is drawn after it: {o:?}"
    );
}

/// #421. spec: S-0606 Terms (a bunkered vehicle stays on the map, on a cell of the bunker's
/// foundation, drawn) and claims 6 and 7 (the walls-up effects start *special-1*), S-0102
/// claims 5, 6, 10 and 17
///
/// The walls are HUTX's *special-1* animation with a y-sort of 1000: key
/// 256 · (C_x + C_y) + 1000. A vehicle at the centre of the foundation cell C + (1, 1) has the
/// key 256 · (C_x + C_y) + 768: the walls are in front of it and drawn after it.
#[test]
fn a_vehicle_on_a_live_buildings_foundation_is_drawn_before_its_front_walls() {
    let o = order(
        "SpecialAnim=SLOTL\nSpecialAnimYSort=1000\n",
        true,
        &[(1, 1)],
    );
    assert!(!o.slot.is_empty(), "the walls are drawn: {o:?}");
    assert!(!o.cars[0].is_empty(), "the vehicle is drawn: {o:?}");
    let first_slot = o.slot[0];
    assert!(
        o.cars[0].iter().all(|&i| i < first_slot),
        "the vehicle is drawn before the walls in front of it: {o:?}"
    );
}
