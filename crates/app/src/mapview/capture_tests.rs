//! A building of the map that changes in play (the lead's note 11 of M8
//! step A4, round 2: a captured tech building kept its old colours and
//! showed no animation). The map's buildings are drawn in the cell pass
//! built when the game starts; one that no longer looks as drawn there is
//! drawn live. Invented types and numbers, no game data.

use std::collections::BTreeSet;
use std::sync::Arc;

use glasnost_convert::kinds::Kind;
use glasnost_map::objects::{Placed, Structure};
use glasnost_render::compose::Paint;
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::scene::{Item, Scene};
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::econ::sell::Stage;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Kind as SimKind, SimRules};
use glasnost_sim::targeting::{ClickOn, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::{GameSetup, HouseId, HouseRole, HouseSetup};

use super::build::{Assets, Inputs, Options, build_parts};
use super::live::{Live, sim_code};
use super::tests::{ART, RULES, art_of, context_full, shp};

const SIZE: i32 = 24;
const ALPHA: HouseId = HouseId(0);
const NEUTRAL: HouseId = HouseId(2);

/// The building's image frames are 4 × 4 (tests.rs), the slot
/// animation's 3 × 3 and the infantry's 2 × 2, so their items can be told
/// apart.
const BODY_FRAME: u32 = 4;
/// See [`BODY_FRAME`].
const SLOT_FRAME: u32 = 3;

/// The import. With `slot`, HUTX's art starts the looping slot animation
/// SLOTL in its *active-1* slot (S-0108 claim 11); SLOTL has 4 image
/// frames and a stage delay of 1 tick (S-0109 Import mapping, `Rate`
/// absent).
fn import(
    slot: bool,
) -> (
    glasnost_convert::Import,
    glasnost_convert::theater::Catalogue,
    Assets,
) {
    let rules = format!("{RULES}[Animations]\n0=SLOTL\n");
    let hut = if slot {
        "[HUTX]\nActiveAnim=SLOTL\n"
    } else {
        "[HUTX]\n"
    };
    let art = format!(
        "{}[SLOTL]\nLayer=ground\nLoopCount=-1\n",
        ART.replacen("[HUTX]\n", hut, 1)
    );
    let mut ctx = context_full(&rules, &art);
    ctx.0
        .arts
        .insert((Kind::Animations, 0), art_of("SLOTL", "slotl.shp"));
    // Canvas 20 × 20, 8 image frames (4 images, 4 shadows); frame i has pixel value i + 1.
    ctx.2.insert(
        b"slotl.shp",
        shp((20, 20), 8, (SLOT_FRAME, SLOT_FRAME), (8, 8)),
    );
    ctx
}

/// The simulation's rules: the synthetic combat ruleset with HUTX, a
/// capturable 2 × 2 building, with `tech` one that needs an engineer (a
/// tech building, S-0515 Terms), and MANX, an engineer.
fn rules(tech: bool) -> Arc<SimRules> {
    Arc::new(SimRules::new(&ruleset(tech), "synthetic-test"))
}

/// The ruleset of [`rules`].
fn ruleset(tech: bool) -> glasnost_rules::schema::Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("HUTX".into());
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("MANX".into());
    r.sim.buildings.insert(
        "HUTX".into(),
        BuildingType {
            strength: Some(400),
            foundation: Some([2, 2]),
            capturable: Some(true),
            needs_engineer: Some(tech),
            ..Default::default()
        },
    );
    r.sim.infantry.insert(
        "MANX".into(),
        UnitType {
            strength: Some(100),
            speed: Some(6),
            locomotion_class: Some(LocomotionClass::Foot),
            locomotor: Some(Locomotor::Walk),
            engineer: Some(true),
            ..Default::default()
        },
    );
    r
}

/// The setup: houses Alpha (0) and Beta (1) and the neutral house (2).
fn setup() -> GameSetup {
    let mut s = testkit::setup(7);
    s.houses.push(HouseSetup {
        name: "Neutral".into(),
        country: "Alphaland".into(),
        human: false,
        start: None,
        role: HouseRole::Neutral,
        ai: false,
    });
    s
}

/// The items of object code `code` in `scene`.
fn items_of(scene: &Scene, code: u32) -> Vec<Item> {
    scene
        .items
        .iter()
        .filter(|i| i.object == Some(code))
        .copied()
        .collect()
}

/// The lit items of `code` whose image is `size` pixels wide.
fn lit(scene: &Scene, code: u32, size: u32) -> Vec<Item> {
    items_of(scene, code)
        .into_iter()
        .filter(|i| scene.images[i.image as usize].width() == size)
        .filter(|i| matches!(i.paint, Paint::Lit(_)))
        .collect()
}

/// The base shade rows of the lit body items of `code`.
fn body_rows(scene: &Scene, code: u32) -> BTreeSet<u32> {
    lit(scene, code, BODY_FRAME)
        .iter()
        .filter_map(|i| match i.paint {
            Paint::Lit(s) => Some(s.base),
            _ => None,
        })
        .collect()
}

/// The pixel values (image frame + 1) of the lit items of `code` whose
/// image is `size` pixels wide.
fn pixels(scene: &Scene, code: u32, size: u32) -> Vec<u8> {
    lit(scene, code, size)
        .iter()
        .map(|i| scene.images[i.image as usize].pixels()[0])
        .collect()
}

/// What happens to the map's building in play.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Act {
    /// An engineer of Alpha's captures it, ordered by a click on it.
    Capture,
    /// It is damaged to 1 health point.
    Damage,
    /// Nothing.
    Nothing,
}

/// What the drawing showed.
struct Seen {
    /// The building's items in the cell pass (its static code) and drawn
    /// live (its simulation code) after the first ticks, before the act.
    before: (usize, usize),
    /// The same after the act.
    after: (usize, usize),
    /// The building's body pixels before (static) and after (live) the act.
    body: (Vec<u8>, Vec<u8>),
    /// The building's body rows after the act.
    rows: BTreeSet<u32>,
    /// The body rows of a building of Alpha's own (drawn live).
    reference: BTreeSet<u32>,
    /// The building's slot-animation pixels, redraw by redraw over six
    /// ticks after the act.
    slots: Vec<Vec<u8>>,
}

/// A neutral HUTX placed by the map (with the slot animation when `slot`,
/// needing an engineer when `tech`) and an engineer of Alpha's; `act`
/// happens to the building.
fn scenario(slot: bool, tech: bool, act: Act) -> Seen {
    let (import, catalogue, assets) = import(slot);
    let mut map = testkit::empty_map(SIZE);
    let at = testkit::mid(SIZE, 2, 0);
    map.objects.structures.push(Structure {
        placed: Placed {
            key: b"0".to_vec(),
            owner: b"Neutral".to_vec(),
            type_name: b"HUTX".to_vec(),
            health: 256,
            cell: at,
            facing: 0,
            raw: Vec::new(),
        },
    });
    let rules = rules(tech);
    let setup = setup();
    let mut world = glasnost_sim::load::new_game(rules.clone(), &map, testkit::catalogue(), &setup)
        .expect("the synthetic game loads");
    let b = world
        .objects
        .values()
        .find(|o| o.kind == SimKind::Building && o.pos.cell() == at)
        .map(|o| o.id)
        .expect("the map's building");
    assert_eq!(world.obj(b).unwrap().owner, Some(NEUTRAL));
    let engineer = testkit::spawn(
        &mut world,
        SimKind::Infantry,
        "MANX",
        0,
        testkit::mid(SIZE, -4, 0),
        0,
    );
    // A building of Alpha's own, drawn in Alpha's colour: the reference row.
    let own = testkit::spawn(
        &mut world,
        SimKind::Building,
        "HUTX",
        0,
        testkit::mid(SIZE, -4, 6),
        0,
    );
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options {
        slots: vec![(b"Alpha".to_vec(), 1), (b"Beta".to_vec(), 2)],
        ..Options::default()
    };
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
    let mut scene = built.scene;
    let code = *live
        .building_ids
        .iter()
        .find(|(_, id)| **id == b)
        .expect("the building is the map's")
        .0;
    let counts = |scene: &Scene| {
        (
            items_of(scene, code).len(),
            items_of(scene, sim_code(b)).len(),
        )
    };
    for _ in 0..4 {
        live.step();
    }
    live.redraw(&inputs, &mut scene);
    let before = counts(&scene);
    let body_before = pixels(&scene, code, BODY_FRAME);
    match act {
        Act::Capture => {
            let r = live.click(
                ALPHA,
                &mut vec![engineer],
                ClickOn::Object(b),
                Modifiers::default(),
            );
            assert_eq!(r.refused, 0, "the capture order is accepted");
            let mut captured = false;
            for _ in 0..800 {
                live.step();
                if live.world.obj(b).is_some_and(|o| o.owner == Some(ALPHA)) {
                    captured = true;
                    break;
                }
            }
            assert!(captured, "the engineer captures the building");
            assert!(
                live.world.obj(engineer).is_none(),
                "the engineer is used up"
            );
        }
        Act::Damage => {
            live.world.obj_mut(b).unwrap().health = 1;
        }
        Act::Nothing => {}
    }
    live.step();
    live.redraw(&inputs, &mut scene);
    let after = counts(&scene);
    let body_after = pixels(&scene, sim_code(b), BODY_FRAME);
    let rows = body_rows(&scene, sim_code(b));
    let reference = body_rows(&scene, sim_code(own));
    let mut slots = Vec::new();
    for _ in 0..6 {
        slots.push(pixels(&scene, sim_code(b), SLOT_FRAME));
        live.step();
        live.redraw(&inputs, &mut scene);
    }
    Seen {
        before,
        after,
        body: (body_before, body_after),
        rows,
        reference,
        slots,
    }
}

/// spec: S-0509 claims 1, 7 and 8 and S-0214 table M (an engineer ordered to capture by a click on
/// the building walks to it and captures it: an ownership change to the engineer's house);
/// S-0103 claim 8 (a building is drawn with its owner's house-colour palette: after the change,
/// the capturing house's)
#[test]
fn a_captured_map_building_takes_its_new_owners_colour() {
    let s = scenario(false, true, Act::Capture);
    // Before the capture the map's building is drawn in the cell pass only.
    assert!(s.before.0 > 0 && s.before.1 == 0, "{:?}", s.before);
    // After it, it is drawn live only: no copy is left in the cell pass.
    assert!(s.after.0 == 0 && s.after.1 > 0, "{:?}", s.after);
    assert_eq!(s.reference.len(), 1, "{:?}", s.reference);
    assert_eq!(s.rows, s.reference, "the capturing house's palette");
}

/// The slot pixels seen over the redraws, one slot image each time.
fn slot_frames(s: &Seen) -> BTreeSet<u8> {
    s.slots
        .iter()
        .map(|px| {
            assert_eq!(px.len(), 1, "one slot image per redraw: {px:?}");
            px[0]
        })
        .collect()
}

/// spec: S-0108 claim 11 (a building placed during the set-up starts its idle and active slots) and
/// S-0109 claims 1 and 5 (a running slot animation shows its current image frame, which advances);
/// S-0103 claim 8 (the new owner's palette); S-0405 claims 8 and 9 and S-0515 claim 2 (a building
/// that needs an engineer is not in operation until its first ownership change): a captured map
/// tech building's slot animation plays, drawn once. Before the capture it keeps its cell-pass
/// drawing (Q-1772 asks whether its slots run then); S-0108 claim 13 (power) is not modelled
#[test]
fn a_captured_map_building_animates() {
    let s = scenario(true, true, Act::Capture);
    // Unmanned, the tech building stays in the cell pass (Q-1772 open).
    assert!(s.before.0 > 0 && s.before.1 == 0, "{:?}", s.before);
    assert!(s.after.0 == 0 && s.after.1 > 0, "{:?}", s.after);
    assert_eq!(s.rows, s.reference, "the capturing house's palette");
    assert!(
        slot_frames(&s).len() > 1,
        "the slot animation plays: {:?}",
        s.slots
    );
}

/// spec: S-0108 claim 11 (a building placed during the set-up starts its idle and active slots) and
/// S-0109 claims 1 and 5 (its slot animation advances): a map building that needs no engineer
/// animates from the start, drawn once
#[test]
fn a_map_building_animates_from_the_start() {
    let s = scenario(true, false, Act::Nothing);
    assert!(s.before.0 == 0 && s.before.1 > 0, "{:?}", s.before);
    assert!(s.after.0 == 0 && s.after.1 > 0, "{:?}", s.after);
    assert!(
        slot_frames(&s).len() > 1,
        "the slot animation plays: {:?}",
        s.slots
    );
}

/// spec: S-0108 claim 2 and F3 (a damaged building's body frame): a map building damaged in play
/// shows its damaged frame
#[test]
fn a_damaged_map_building_shows_its_damage() {
    let s = scenario(false, true, Act::Damage);
    assert!(s.before.0 > 0 && s.before.1 == 0, "{:?}", s.before);
    assert!(s.after.0 == 0 && s.after.1 > 0, "{:?}", s.after);
    assert!(!s.body.0.is_empty() && !s.body.1.is_empty(), "{:?}", s.body);
    assert_ne!(s.body.0, s.body.1, "the damaged frame");
}

/// Alpha's own HUTX with the slot animation, sellable (its type has a
/// build-up sprite, S-0408 claim 4), sold after the first ticks: for each
/// tick from the order on, the sale's stage and the building's
/// slot-animation pixels as redrawn after the tick.
fn sale_slots(packs: bool) -> Vec<(Option<Stage>, Vec<u8>)> {
    let (import, catalogue, assets) = import(true);
    let map = testkit::empty_map(SIZE);
    let mut r = ruleset(false);
    r.sim.buildings.get_mut("HUTX").unwrap().econ.buildup = Some(true);
    if packs {
        // A type that undeploys into a vehicle packs up on a sell order (S-0213 claim 12).
        r.sim.buildings.get_mut("HUTX").unwrap().undeploys_into = Some("CART".into());
    }
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let setup = setup();
    let mut world = glasnost_sim::load::new_game(rules.clone(), &map, testkit::catalogue(), &setup)
        .expect("the synthetic game loads");
    let b = testkit::spawn(
        &mut world,
        SimKind::Building,
        "HUTX",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options {
        slots: vec![(b"Alpha".to_vec(), 1), (b"Beta".to_vec(), 2)],
        ..Options::default()
    };
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
    let mut scene = built.scene;
    for _ in 0..4 {
        live.step();
    }
    live.redraw(&inputs, &mut scene);
    assert_eq!(
        pixels(&scene, sim_code(b), SLOT_FRAME).len(),
        1,
        "the slot animation is drawn before the sale"
    );
    assert!(
        glasnost_sim::orders::issue(&mut live.world, ALPHA, Order::Sell { actor: b }),
        "the sell order is accepted"
    );
    let mut out = Vec::new();
    for _ in 0..120 {
        live.step();
        live.redraw(&inputs, &mut scene);
        let stage = live
            .world
            .obj(b)
            .and_then(|o| o.econ.sale.as_ref())
            .map(|s| s.stage);
        out.push((stage, pixels(&scene, sim_code(b), SLOT_FRAME)));
        if live.world.obj(b).is_none() {
            break;
        }
    }
    out
}

/// spec: S-0405 claim 9 and Changes for other specs (revision 36, S-0108 claim 10 and scenario
/// B14: a sold building's slot animations stay drawn until the first tick of its build-down and
/// are hidden from then on) and S-0408 claim 6 (the build-down starts at a later processing than
/// the sale's first)
#[test]
fn a_sold_buildings_slots_stay_until_its_build_down() {
    let seen = sale_slots(false);
    let begun = seen
        .iter()
        .position(|(s, _)| *s == Some(Stage::Begun))
        .expect("the sale's first processing");
    let down = seen
        .iter()
        .position(|(s, _)| matches!(s, Some(Stage::BuildDown(_))))
        .expect("the build-down starts");
    assert!(begun < down, "{seen:?}");
    for (s, px) in &seen[..down] {
        assert_eq!(px.len(), 1, "drawn before the build-down ({s:?}): {px:?}");
    }
    for (s, px) in &seen[down..] {
        assert!(
            px.is_empty(),
            "hidden from the build-down's first tick ({s:?}): {px:?}"
        );
    }
}

/// spec: S-0405 claim 9 and S-0415 claim 9 (W15 part k: a pack-up goes the same stage path as a sale, so a packing building's slot animations stay drawn until the first tick of its build-down and are hidden from then on)
#[test]
fn a_packing_buildings_slots_stay_until_its_build_down() {
    let seen = sale_slots(true);
    let begun = seen
        .iter()
        .position(|(s, _)| *s == Some(Stage::Begun))
        .expect("the pack-up's first processing");
    let down = seen
        .iter()
        .position(|(s, _)| matches!(s, Some(Stage::BuildDown(_))))
        .expect("the build-down starts");
    assert!(begun < down, "{seen:?}");
    for (s, px) in &seen[..down] {
        assert_eq!(px.len(), 1, "drawn before the build-down ({s:?}): {px:?}");
    }
    for (s, px) in &seen[down..] {
        assert!(
            px.is_empty(),
            "hidden from the build-down's first tick ({s:?}): {px:?}"
        );
    }
}
