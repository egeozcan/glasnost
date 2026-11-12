//! A building of the map that changes in play (the lead's note 11 of M8
//! step A4, round 2: a captured tech building kept its old colours and
//! showed no animation). The map's buildings are drawn in the cell pass
//! built when the game starts; one that no longer looks as drawn there is
//! drawn live. A building that needs an engineer and is not manned holds
//! its slots (S-0108 claim 26) until its owner changes. Invented types and
//! numbers, no game data.

use std::collections::BTreeSet;
use std::sync::Arc;

use glasnost_convert::kinds::Kind;
use glasnost_map::objects::{EntryFields, Placed, Structure};
use glasnost_render::compose::Paint;
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::scene::{Item, Scene};
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, UnitType};
use glasnost_sim::econ::sell::Stage;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Kind as SimKind, SimRules};
use glasnost_sim::targeting::{ClickOn, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::{GameSetup, HouseId, HouseRole, HouseSetup, ObjectId};

use super::build::{Assets, Inputs, Options, build_parts};
use super::live::{Live, sim_code};
use super::quirks::ViewQuirks;
use super::tests::{ART, RULES, art_of, context_full, shp};

const SIZE: i32 = 24;
const ALPHA: HouseId = HouseId(0);
const BETA: HouseId = HouseId(1);
const NEUTRAL: HouseId = HouseId(2);

/// The building's image frames are 4 × 4 (tests.rs), the slot
/// animation's 3 × 3 and the infantry's 2 × 2, so their items can be told
/// apart.
const BODY_FRAME: u32 = 4;
/// See [`BODY_FRAME`].
const SLOT_FRAME: u32 = 3;

/// HUTX's art keys that start the looping slot animation SLOTL in its
/// *active-1* slot (S-0108 claim 11), counting *freeze-unpowered* (the
/// default, claim 13).
const SLOT: &str = "ActiveAnim=SLOTL\n";
/// The same with *freeze-unpowered* off and *light-unpowered* on (S-0108
/// Import mapping, claim 13).
const LIGHT_SLOT: &str = "ActiveAnim=SLOTL\nActiveAnimPowered=no\nActiveAnimPoweredLight=yes\n";

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
    import_with(if slot { SLOT } else { "" })
}

/// The import with `keys` in HUTX's art section.
fn import_with(
    keys: &str,
) -> (
    glasnost_convert::Import,
    glasnost_convert::theater::Catalogue,
    Assets,
) {
    let rules = format!("{RULES}[Animations]\n0=SLOTL\n");
    let hut = format!("[HUTX]\n{keys}");
    let art = format!(
        "{}[SLOTL]\nLayer=ground\nLoopCount=-1\n",
        ART.replacen("[HUTX]\n", &hut, 1)
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
    /// The same, with the engineer standing before the building in the
    /// update order, as an engineer the map places does (S-0108 claim 26).
    CaptureByEarlierEngineer,
    /// It is damaged to 1 health point.
    Damage,
    /// It loses 1 health point: no damage frame (S-0108 F3).
    Chip,
    /// It is protected (S-0102 claim 34): drawn live for its look.
    Protect,
    /// Its charge flash runs (S-0519 claim 25): drawn live for its look.
    Flash,
    /// It is switched off (S-0108 claim 28 disables it), then an engineer
    /// of Alpha's captures it.
    SwitchOffThenCapture,
    /// An engineer of Alpha's captures it, it is switched off, then an
    /// engineer of Beta's captures it.
    CaptureSwitchOffThenRecapture,
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
    /// The building's body pixels before (in the cell pass or live) and
    /// after (live) the act.
    body: (Vec<u8>, Vec<u8>),
    /// The building's slot-animation pixels before the act, in the cell
    /// pass or live.
    still: Vec<u8>,
    /// For [`Act::SwitchOffThenCapture`] and
    /// [`Act::CaptureSwitchOffThenRecapture`], the slot-animation pixels
    /// redraw by redraw over six ticks while it is switched off.
    held: Vec<Vec<u8>>,
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
    scenario_with(
        if slot { SLOT } else { "" },
        tech,
        act,
        ViewQuirks::default(),
    )
}

/// An engineer of Alpha's, ordered by a click on the building, captures
/// it (S-0509 claims 1, 7 and 8).
fn capture(live: &mut Live, engineer: ObjectId, b: ObjectId) {
    capture_for(live, ALPHA, engineer, b);
}

/// An engineer of `house`'s, ordered by a click on the building, captures
/// it (S-0509 claims 1, 7 and 8).
fn capture_for(live: &mut Live, house: HouseId, engineer: ObjectId, b: ObjectId) {
    let r = live.click(
        house,
        &mut vec![engineer],
        ClickOn::Object(b),
        Modifiers::default(),
    );
    assert_eq!(r.refused, 0, "the capture order is accepted");
    let mut captured = false;
    for _ in 0..800 {
        live.step();
        if live.world.obj(b).is_some_and(|o| o.owner == Some(house)) {
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

/// [`scenario`] with `keys` in HUTX's art section and the drawing's quirk
/// switches `quirks`.
fn scenario_with(keys: &str, tech: bool, act: Act, quirks: ViewQuirks) -> Seen {
    let (import, catalogue, assets) = import_with(keys);
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
    live.set_view_quirks(quirks);
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
    let either = |scene: &Scene, size| {
        [pixels(scene, code, size), pixels(scene, sim_code(b), size)].concat()
    };
    let body_before = either(&scene, BODY_FRAME);
    let still = either(&scene, SLOT_FRAME);
    let mut held = Vec::new();
    match act {
        Act::Capture => capture(&mut live, engineer, b),
        Act::CaptureByEarlierEngineer => {
            live.world.update.order.retain(|&x| x != engineer);
            live.world.update.order.insert(0, engineer);
            capture(&mut live, engineer, b);
        }
        Act::SwitchOffThenCapture => {
            // Two more ticks, so that the slot is held at a frame other than its first and its last.
            live.step();
            live.step();
            live.world.obj_mut(b).unwrap().econ.building.switched_off = true;
            for _ in 0..6 {
                live.step();
                live.redraw(&inputs, &mut scene);
                held.push(pixels(&scene, sim_code(b), SLOT_FRAME));
            }
            capture(&mut live, engineer, b);
            assert!(
                live.world.obj(b).unwrap().econ.building.switched_off,
                "the switch is kept at the ownership change (S-0405 claim 10)"
            );
        }
        Act::CaptureSwitchOffThenRecapture => {
            capture(&mut live, engineer, b);
            // A few ticks, so that the slot is held at a frame other than its first and its last.
            live.step();
            live.step();
            live.world.obj_mut(b).unwrap().econ.building.switched_off = true;
            for _ in 0..6 {
                live.step();
                live.redraw(&inputs, &mut scene);
                held.push(pixels(&scene, sim_code(b), SLOT_FRAME));
            }
            let other = testkit::spawn(
                &mut live.world,
                SimKind::Infantry,
                "MANX",
                1,
                testkit::mid(SIZE, 6, 0),
                0,
            );
            capture_for(&mut live, BETA, other, b);
            assert!(
                live.world.obj(b).unwrap().econ.building.switched_off,
                "the switch is kept at the ownership change (S-0405 claim 10)"
            );
        }
        Act::Damage => {
            live.world.obj_mut(b).unwrap().health = 1;
        }
        Act::Chip => {
            live.world.obj_mut(b).unwrap().health -= 1;
        }
        Act::Protect => {
            use glasnost_sim::superweapons::{ProtectKind, Protection, Tint};
            let tick = live.world.tick;
            live.world.obj_mut(b).unwrap().sw.protection = Some(Protection {
                start: tick - 1,
                duration: 1000,
                kind: ProtectKind::Curtain,
                tint: Tint::Rise(tick - 1),
            });
        }
        Act::Flash => {
            glasnost_sim::mark::start_flash(&mut live.world, b, 1000);
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
        still,
        held,
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

/// spec: S-0108 claim 26 (Q-2186: a held slot first advances in the tick of the capture when the
/// engineer stands before the building's slots in the update order, as one the map places does,
/// and in the next tick when it stands after them, as one trained during the game or a starting
/// one does): the capture by the earlier engineer runs one tick ahead of the other
#[test]
fn a_capture_by_an_engineer_before_the_slots_releases_them_one_tick_earlier() {
    let late = scenario(true, true, Act::Capture);
    let early = scenario(true, true, Act::CaptureByEarlierEngineer);
    assert_ne!(early.slots, late.slots, "the two timelines differ");
    // The capture by the earlier engineer is one tick ahead: its timeline, from the second
    // redraw on, is the other's shifted by one.
    assert_eq!(
        early.slots[..5],
        late.slots[1..],
        "{:?} {:?}",
        early.slots,
        late.slots
    );
}

/// spec: S-0108 claim 11 (a building placed during the set-up starts its idle and active slots) and
/// S-0109 claims 1 and 5 (a running slot animation shows its current image frame, which advances);
/// S-0103 claim 8 (the new owner's palette); S-0108 claim 26 and S-0515 claim 2 (a map building
/// that needs an engineer is not manned until its first ownership change and is disabled from its
/// placement, so its freeze-unpowered slot holds the image frame it was created with; the
/// ownership change re-enables it and the slot resumes): a captured map tech building's slot
/// animation plays, drawn once. Before the capture it is drawn live, its slot at its first frame
#[test]
fn a_captured_map_building_animates() {
    let s = scenario(true, true, Act::Capture);
    assert!(s.before.0 == 0 && s.before.1 > 0, "{:?}", s.before);
    assert_eq!(s.still, [1], "held at the frame it was created with");
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

/// What [`map_building_drawn_live`] changes on the map's building.
#[derive(Clone, Copy, Debug)]
enum Look {
    /// Its turret turns (S-0108 claim 2 draws the turret at its facing).
    Turn,
    /// Its animation-frame counter moves on (S-0105 claim 16, S-0609 claim 14).
    Spin,
    /// It starts unloading (S-0108 claims 22 and 23).
    Unload,
}

/// Whether the map's building (a tech HUTX, so it holds still in the
/// cell pass) is drawn live after `look` changes on it, and whether it was
/// in the cell pass before.
fn map_building_drawn_live(look: Look) -> ((usize, usize), (usize, usize)) {
    let (import, catalogue, assets) = import_with("");
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
    let rules = rules(true);
    let setup = setup();
    let world = glasnost_sim::load::new_game(rules.clone(), &map, testkit::catalogue(), &setup)
        .expect("the synthetic game loads");
    let b = world
        .objects
        .values()
        .find(|o| o.kind == SimKind::Building && o.pos.cell() == at)
        .map(|o| o.id)
        .expect("the map's building");
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
    let mut live = Live::new(&inputs, &mut built, painter, (world, rules.clone(), setup, Vec::new()));
    let mut scene = built.scene;
    let code = *live
        .building_ids
        .iter()
        .find(|(_, id)| **id == b)
        .expect("the building is the map's")
        .0;
    let counts = |scene: &Scene| (items_of(scene, code).len(), items_of(scene, sim_code(b)).len());
    for _ in 0..4 {
        live.step();
    }
    live.redraw(&inputs, &mut scene);
    let before = counts(&scene);
    let tick = live.world.tick;
    let o = live.world.obj_mut(b).unwrap();
    match look {
        Look::Turn => o.turret = glasnost_sim::turning::Turning::at(o.turret.value(tick) ^ 0x4000),
        Look::Spin => o.combat.anim_frame += 1,
        Look::Unload => o.beh.current = Some(glasnost_sim::rules::Behaviour::Unload),
    }
    // The state as a tick left it, drawn without another tick.
    live.redraw(&inputs, &mut scene);
    (before, counts(&scene))
}

/// spec: S-0108 claim 2 (a building's turret is drawn at its facing), S-0105 claim 16 and S-0609
/// claim 14 (its animation-frame counter picks its frames), S-0108 claims 22 and 23 (the unloading
/// images): a map building held in the cell pass that turns, spins or unloads is drawn live (#97)
#[test]
fn a_map_building_that_turns_spins_or_unloads_is_drawn_live() {
    for look in [Look::Turn, Look::Spin, Look::Unload] {
        let (before, after) = map_building_drawn_live(look);
        assert!(before.0 > 0 && before.1 == 0, "{look:?}: {before:?}");
        assert!(after.0 == 0 && after.1 > 0, "{look:?}: {after:?}");
    }
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

/// spec: S-0103 claim 10 (the neutral house shows the light-grey definition) and claim 8 (a
/// building is drawn with its owner's palette); S-0108 claim 2 (its damage changes its body
/// frame); S-0515 claim 2 (it is manned only from its first ownership change); S-0108 claims 12,
/// 13 and 26 (the damage swap restarts its slots, and a start while not manned freezes a slot only
/// when the type needs power, so the slots of a type that does not need power play from then on
/// although the building is still disabled): the lead's note 18 of M8 step A4, round 4. A neutral
/// tech building that is damaged but never captured animates, in the neutral house's colour, not a
/// player's
#[test]
fn a_damaged_unmanned_tech_building_animates_in_the_neutral_colour() {
    let s = scenario(true, true, Act::Damage);
    assert!(s.before.0 == 0 && s.before.1 > 0, "{:?}", s.before);
    assert_eq!(s.still, [1], "held at the frame it was created with");
    assert!(s.after.0 == 0 && s.after.1 > 0, "{:?}", s.after);
    assert!(
        slot_frames(&s).len() > 1,
        "the slot animation plays: {:?}",
        s.slots
    );
    // A neutral building that needs no engineer is drawn live from the start: the neutral row.
    let neutral = scenario(true, false, Act::Nothing).rows;
    assert_eq!(neutral.len(), 1, "{neutral:?}");
    assert_eq!(s.rows, neutral, "the neutral house's palette");
    assert_ne!(s.rows, s.reference, "not a player house's palette");
}

/// spec: S-0108 claim 2 and F3 (a building above the damaged ratio shows its normal body frame);
/// S-0515 claim 2 (unmanned until its first ownership change); S-0108 claims 12 and 26 (no damage
/// swap, so no start: its freeze-unpowered slot keeps holding the image frame it was created
/// with): the lead's note 18 of M8 step A4, reopened. A neutral tech building that loses a single
/// point of health looks undamaged and its slot stands still
#[test]
fn a_chipped_unmanned_tech_building_looks_undamaged_and_holds_its_slot() {
    let s = scenario(true, true, Act::Chip);
    assert!(s.before.0 == 0 && s.before.1 > 0, "{:?}", s.before);
    assert!(s.after.0 == 0 && s.after.1 > 0, "{:?}", s.after);
    assert_eq!(s.body.0, s.body.1, "the normal body frame");
    assert_eq!(s.still, [1], "held at the frame it was created with");
    assert_eq!(
        slot_frames(&s),
        BTreeSet::from([1]),
        "the slot stands still: {:?}",
        s.slots
    );
}

/// spec: S-0102 claim 34 (a protected building's body shows the protection appearance) and
/// S-0519 claim 25 (the charge flash repaints the body); S-0108 claims 11 and 26 and S-0515 claim 2
/// (a map building that needs an engineer and is not manned holds its freeze-unpowered slots at the
/// image frame they were created with, whatever its look; the guard order starts no slot): the
/// lead's note 18 of M8 step A4, reopened
#[test]
fn an_unmanned_tech_building_holds_its_slots_whatever_its_look() {
    for act in [Act::Protect, Act::Flash, Act::Nothing] {
        let s = scenario(true, true, act);
        assert!(s.before.0 == 0 && s.before.1 > 0, "{:?}", s.before);
        assert!(s.after.0 == 0 && s.after.1 > 0, "{:?}", s.after);
        assert_eq!(s.still, [1], "held at the frame it was created with");
        assert_eq!(
            slot_frames(&s),
            BTreeSet::from([1]),
            "the slot stands at the frame it was created with: {:?}",
            s.slots
        );
    }
    // A building that needs no engineer, protected, still animates (S-0108 claim 11).
    let s = scenario(true, false, Act::Protect);
    assert!(
        slot_frames(&s).len() > 1,
        "the slot animation plays: {:?}",
        s.slots
    );
}

/// spec: S-0108 claims 13 and 26 (a slot whose art turns freeze-unpowered off keeps playing while
/// the building that needs an engineer is not manned; only each slot's counting flag decides) and
/// claim 27 (it has never been powered, so no power-loss reaction stops its light-unpowered slot)
#[test]
fn a_light_unpowered_slot_of_an_unmanned_tech_building_plays() {
    let s = scenario_with(LIGHT_SLOT, true, Act::Nothing, ViewQuirks::default());
    assert!(s.before.0 == 0 && s.before.1 > 0, "{:?}", s.before);
    assert!(
        slot_frames(&s).len() > 1,
        "the slot animation plays: {:?}",
        s.slots
    );
}

/// The frame a slot held over [`Seen::held`], after its first redraw.
fn held_frame(s: &Seen) -> u8 {
    let held: BTreeSet<u8> = s.held[1..].iter().flatten().copied().collect();
    assert_eq!(held.len(), 1, "held while switched off: {:?}", s.held);
    let frame = *held.first().unwrap();
    assert_ne!(
        frame, 1,
        "held at a frame other than its first: {:?}",
        s.held
    );
    frame
}

/// spec: S-0108 claim 28 (switching off disables the building: its freeze-unpowered slot holds its
/// image frame) and claim 26 (at every ownership change of a building whose type needs an engineer
/// it is re-enabled and every frozen freeze-unpowered slot resumes from the image frame it held,
/// not from the frame it was created with, advancing at its first eligible update, S-0109 claim 23,
/// whether or not the building is in operation after the change, here still switched off:
/// `quirks.enable_unfreezes_unpowered`); S-0108 Edge cases (that switch off: the slot stays frozen,
/// as the building is not in operation after the change). S-0108 B22 part (k)
#[test]
fn a_capture_resumes_a_held_slot_from_its_held_frame() {
    for unfreezes in [true, false] {
        let quirks = ViewQuirks {
            enable_unfreezes_unpowered: unfreezes,
            ..ViewQuirks::default()
        };
        let s = scenario_with(SLOT, true, Act::CaptureSwitchOffThenRecapture, quirks);
        let frame = held_frame(&s);
        if unfreezes {
            // Resumed: the first redraw after the change shows the frame after the held one (not
            // the first frame, as a start again would).
            assert_eq!(s.slots[0], [frame + 1], "{:?}", s.slots);
            assert!(slot_frames(&s).len() > 1, "it plays: {:?}", s.slots);
        } else {
            assert_eq!(
                slot_frames(&s),
                BTreeSet::from([frame]),
                "still held: {:?}",
                s.slots
            );
        }
    }
}

/// spec: S-0108 claim 13 (an ownership change of a building whose type does not need an engineer is
/// neither a disabling nor a re-enabling) and claim 28 (an ownership change while switched off
/// leaves the building switched off, S-0405 claim 10, and for such a type the held slots stay
/// held), in both positions of `quirks.enable_unfreezes_unpowered`
#[test]
fn a_capture_of_a_switched_off_building_that_needs_no_engineer_keeps_its_slot_held() {
    for unfreezes in [true, false] {
        let quirks = ViewQuirks {
            enable_unfreezes_unpowered: unfreezes,
            ..ViewQuirks::default()
        };
        let s = scenario_with(SLOT, false, Act::SwitchOffThenCapture, quirks);
        let frame = held_frame(&s);
        assert_eq!(
            slot_frames(&s),
            BTreeSet::from([frame]),
            "still held after the capture: {:?}",
            s.slots
        );
    }
}

/// A neutral HUTX placed by the map with its switched-on field 0 (S-0009
/// claim 40), with a power drain when `drain` (else none), and an engineer
/// of Alpha's. Over the first ticks, the capture, and Alpha's switch-on
/// order: the slot-animation pixels redraw by redraw, and whether the
/// building is switched off at each.
struct PlacedOff {
    /// Switched off after the set-up.
    off_at_start: bool,
    /// Redraw by redraw from the first ticks to the capture.
    before: Vec<Vec<u8>>,
    /// Redraw by redraw over six ticks after the capture.
    captured: Vec<Vec<u8>>,
    /// Switched off after the capture.
    off_after_capture: bool,
    /// The redraw after the tick of the switch-on order.
    order_tick: Vec<u8>,
    /// Redraw by redraw over six ticks after that.
    after: Vec<Vec<u8>>,
}

/// See [`PlacedOff`].
fn placed_off(drain: bool) -> PlacedOff {
    let (import, catalogue, assets) = import(true);
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
    map.objects.structure_fields.push(EntryFields {
        // spec: S-0009 claim 40 (the map's switched-on field, here 0)
        switched_on: Some(false),
        ..EntryFields::default()
    });
    let mut r = ruleset(false);
    if drain {
        // A drain of 10 (an invented test value): S-0405 claim 18 lets the map switch it off.
        r.sim.buildings.get_mut("HUTX").unwrap().econ.power = Some(-10);
    }
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let setup = setup();
    let mut world = glasnost_sim::load::new_game(rules.clone(), &map, testkit::catalogue(), &setup)
        .expect("the synthetic game loads");
    let b = world
        .objects
        .values()
        .find(|o| o.kind == SimKind::Building && o.pos.cell() == at)
        .map(|o| o.id)
        .expect("the map's building");
    let engineer = testkit::spawn(
        &mut world,
        SimKind::Infantry,
        "MANX",
        0,
        testkit::mid(SIZE, -4, 0),
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
    let off = |live: &Live| live.world.obj(b).unwrap().econ.building.switched_off;
    let off_at_start = off(&live);
    let mut before = Vec::new();
    for _ in 0..6 {
        live.step();
        live.redraw(&inputs, &mut scene);
        before.push(pixels(&scene, sim_code(b), SLOT_FRAME));
    }
    capture(&mut live, engineer, b);
    let mut captured = Vec::new();
    for _ in 0..6 {
        live.step();
        live.redraw(&inputs, &mut scene);
        captured.push(pixels(&scene, sim_code(b), SLOT_FRAME));
    }
    let off_after_capture = off(&live);
    let report = live.issue_orders(ALPHA, vec![Order::PowerOn { actor: b }]);
    assert_eq!(report.refused, 0, "the power-on order is accepted");
    live.step();
    live.redraw(&inputs, &mut scene);
    let order_tick = pixels(&scene, sim_code(b), SLOT_FRAME);
    assert!(!off(&live), "switched on");
    let mut after = Vec::new();
    for _ in 0..6 {
        live.step();
        live.redraw(&inputs, &mut scene);
        after.push(pixels(&scene, sim_code(b), SLOT_FRAME));
    }
    PlacedOff {
        off_at_start,
        before,
        captured,
        off_after_capture,
        order_tick,
        after,
    }
}

/// The distinct pixels of one slot image per redraw.
fn frames_of(redraws: &[Vec<u8>]) -> BTreeSet<u8> {
    redraws
        .iter()
        .map(|px| {
            assert_eq!(px.len(), 1, "one slot image per redraw: {px:?}");
            px[0]
        })
        .collect()
}

/// spec: S-0108 claim 28 (a building the map places switched off, S-0405 claim 18 allowing it for a
/// type with a base drain above 0: disabled at the end of its placement, so its freeze-unpowered
/// slot shows the image frame it was created with and does not advance, although the type does not
/// need power; an ownership change of a type that does not need an engineer leaves it switched off
/// and is no re-enabling, so the slot stays held; the switch-on re-enables it at once and the slot
/// resumes from the frame it held, advancing at its first eligible update, in the tick after the
/// power order and not later, S-0109 claim 23, S-0203 claim 7): S-0108 B22 part (l), test type D
#[test]
fn a_map_building_placed_switched_off_holds_its_slot_until_switched_on() {
    let s = placed_off(true);
    assert!(s.off_at_start, "switched off by the map");
    assert_eq!(frames_of(&s.before), BTreeSet::from([1]), "{:?}", s.before);
    assert!(s.off_after_capture, "still switched off after the capture");
    assert_eq!(
        frames_of(&s.captured),
        BTreeSet::from([1]),
        "held through the ownership change: {:?}",
        s.captured
    );
    assert_eq!(s.order_tick, [1], "still held in the order's tick");
    assert_eq!(
        s.after[0],
        [2],
        "the frame after the held one in the next tick, no restart: {:?}",
        s.after
    );
    assert!(frames_of(&s.after).len() > 1, "it plays: {:?}", s.after);
}

/// spec: S-0108 claim 28 (a type with neither a base drain above 0 nor the needs-power flag is not
/// switched off by the map, S-0405 claim 18; unless claim 26 disables it, its slots play from its
/// placement): S-0108 B22 part (l), test type Z
#[test]
fn a_map_building_that_draws_no_power_is_not_switched_off_by_the_map() {
    let s = placed_off(false);
    assert!(!s.off_at_start, "the map's field 0 does not switch it off");
    assert!(frames_of(&s.before).len() > 1, "it plays: {:?}", s.before);
    assert!(frames_of(&s.captured).len() > 1, "{:?}", s.captured);
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

/// spec: S-0108 claim 11 (a building placed in play, like a replacement that trigger action code
/// 130 restores, enters with *construction* queued, and its grand opening is at the completion of
/// that construction, not at its placement) and S-0406 claim 17 (at entry no behaviour is current
/// and *construction* is queued; a rule that asks for its behaviour then sees the queued one)
#[test]
fn a_building_placed_in_play_opens_at_its_completion() {
    use glasnost_sim::econ::placement::{begin_construction, seen_behaviour};
    use glasnost_sim::rules::Behaviour;
    let (import, catalogue, assets) = import(true);
    let map = testkit::empty_map(SIZE);
    let mut r = ruleset(false);
    {
        let hut = r.sim.buildings.get_mut("HUTX").unwrap();
        hut.econ.buildup = Some(true);
        // spec: S-0108 claim 5 (a build-up of N_b frames has n = ⌊N_b / 2⌋ stages) and S-0406 claim 15 (a range of two stages or more hands over at its last stage)
        hut.econ.buildup_frames = Some(8);
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
    begin_construction(&mut world, b, false);
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
    let mut scene = built.scene;
    live.redraw(&inputs, &mut scene);
    assert!(
        live.slot_names(b).is_empty(),
        "no slot at its entry, while construction is only queued"
    );
    let mut building = 0;
    let mut opened = None;
    for t in 0..3000 {
        live.step();
        live.redraw(&inputs, &mut scene);
        let o = live.world.obj(b).expect("the building stays");
        if seen_behaviour(o) == Some(Behaviour::Construction) {
            building += 1;
            assert!(
                live.slot_names(b).is_empty(),
                "no slot before its completion (tick {t})"
            );
        } else if !live.slot_names(b).is_empty() {
            opened = Some(t);
            break;
        }
    }
    assert!(building > 1, "it was built up over {building} ticks");
    assert!(
        opened.is_some(),
        "its slots start at its completion (built up over {building} ticks)"
    );
}

/// #87. spec: S-0606 claims 6 and 12 (the walls' event starts the special slots) and S-0108
/// claim 11 (a completed building's slots run)
///
/// A walls event of a tick in which the building's type has not been drawn yet (a network game
/// that ticks without drawing, or a type's first tick) is kept until the type's slot definitions
/// are known, not dropped.
#[test]
fn a_walls_event_before_the_first_drawing_is_kept() {
    use glasnost_sim::combat::effects::Effect;
    let (import, catalogue, assets) = import_with("ActiveAnim=SLOTL\nSpecialAnim=SLOTL\n");
    let map = testkit::empty_map(SIZE);
    let rules = rules(false);
    let setup = setup();
    let world = glasnost_sim::load::new_game(rules.clone(), &map, testkit::catalogue(), &setup)
        .expect("the synthetic game loads");
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
    let mut scene = built.scene;
    // A building of a type not drawn yet enters in play; ticks without
    // drawing follow, and in the last one the walls' event starts
    // special-1.
    let b = testkit::spawn(
        &mut live.world,
        SimKind::Building,
        "HUTX",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    live.step();
    live.step();
    live.world.effects.push(Effect::SpecialAnims {
        building: b,
        slots: [1, 0],
        start: true,
        damaged: false,
    });
    live.after_tick();
    // Drawing resumes.
    live.redraw(&inputs, &mut scene);
    live.step();
    live.redraw(&inputs, &mut scene);
    let names = live.slot_names(b);
    assert!(
        names.iter().any(|n| n.starts_with("Special1=")),
        "special-1 runs: {names:?}"
    );
    // A second redraw and tick do not start it again.
    live.step();
    live.redraw(&inputs, &mut scene);
    assert_eq!(
        live.slot_names(b)
            .iter()
            .filter(|n| n.starts_with("Special1="))
            .count(),
        1
    );
}

/// The superweapon slots of a HUTX that has the superweapon CURTX
/// (recharge 0.1 minutes, 90 ticks, S-0512): its art starts SLOTL in
/// *super-1* and in *super-3*; with `reads` the switch
/// `quirks.ungranted_super_reads_clock` on; with `map_placed` the map places
/// it for the neutral house at the set-up, otherwise it is placed for Alpha
/// in play before the first tick. The
/// names of its superweapon slots at its grand opening, and whether its
/// owner's record was granted at the end of the set-up and at the opening.
fn super_slots_at_opening(reads: bool, map_placed: bool) -> (Vec<String>, bool, bool) {
    use glasnost_rules::sim::Decimal;
    use glasnost_rules::sim_econ::SuperweaponType;
    let (import, catalogue, assets) = import_with("SuperAnim=SLOTL\nSuperAnimThree=SLOTL\n");
    let mut map = testkit::empty_map(SIZE);
    let at = testkit::mid(SIZE, 0, 0);
    if map_placed {
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
    }
    let mut r = ruleset(false);
    r.sim.superweapons.insert(
        "CURTX".into(),
        SuperweaponType {
            kind: Some("IronCurtain".into()),
            action: Some("IronCurtain".into()),
            recharge_time: Some(Decimal("0.1".into())),
            ..Default::default()
        },
    );
    r.registries
        .entry("superweapons".into())
        .or_default()
        .push("CURTX".into());
    r.sim.buildings.get_mut("HUTX").unwrap().econ.super_weapon = Some("CURTX".into());
    r.globals.quirks.ungranted_super_reads_clock = reads;
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let setup = setup();
    let mut world = glasnost_sim::load::new_game(rules.clone(), &map, testkit::catalogue(), &setup)
        .expect("the synthetic game loads");
    let b = if map_placed {
        world
            .objects
            .values()
            .find(|o| o.kind == SimKind::Building && o.pos.cell() == at)
            .map(|o| o.id)
            .expect("the map's building")
    } else {
        testkit::spawn(&mut world, SimKind::Building, "HUTX", 0, at, 0)
    };
    let owner = world.obj(b).unwrap().owner.expect("an owner");
    assert_eq!(owner, if map_placed { NEUTRAL } else { ALPHA });
    let set_up = glasnost_sim::superweapons::record(&world, owner, 0).is_some_and(|r| r.granted);
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
    let mut scene = built.scene;
    live.redraw(&inputs, &mut scene);
    for _ in 0..3 {
        live.step();
        live.redraw(&inputs, &mut scene);
        let names: Vec<String> = live
            .slot_names(b)
            .into_iter()
            .filter(|n| n.starts_with("Super"))
            .collect();
        if !names.is_empty() {
            let granted = glasnost_sim::superweapons::record(&live.world, owner, 0)
                .is_some_and(|r| r.granted);
            return (names, set_up, granted);
        }
    }
    panic!("no superweapon slot opened");
}

/// spec: S-0108 claim 11 and Edge cases (`quirks.ungranted_super_reads_clock`, default on: the
/// slot reads the record's clock whether or not it is granted, and a record never granted and
/// never reset reads 0, so a set-up opening, which meets no record a building granted, starts the
/// charged *super-3*; off: a record that is not granted counts as not charged, so *super-1*. A
/// building placed in play opens after its house's next update has granted the record, with its
/// clock started at the grant: *super-1* in both positions)
#[test]
fn an_ungranted_superweapon_record_is_read_by_its_clock() {
    for (reads, slot) in [(true, "Super3="), (false, "Super1=")] {
        let (names, set_up, _) = super_slots_at_opening(reads, true);
        assert!(!set_up, "the set-up grants no record");
        assert_eq!(names.len(), 1, "one superweapon slot: {names:?}");
        assert!(
            names[0].starts_with(slot),
            "set-up, switch {reads}: {names:?}"
        );
        let (names, _, granted) = super_slots_at_opening(reads, false);
        assert!(granted, "granted at the house's update before the opening");
        assert_eq!(names.len(), 1, "one superweapon slot: {names:?}");
        assert!(
            names[0].starts_with("Super1="),
            "in play, switch {reads}: {names:?}"
        );
    }
}
