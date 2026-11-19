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

/// The paint of a drawing as it shows: its kind and the palette rows it
/// takes, by their contents.
fn paint_shown(scene: &Scene, paint: &Paint) -> String {
    let table = |s: &glasnost_render::compose::Shade| {
        format!(
            "{:?} rho {} mask {}",
            &scene.rows[s.base as usize..][..usize::from(s.rows)],
            s.rho,
            s.mask
        )
    };
    match paint {
        Paint::Row(r) => format!("row {:?}", scene.rows[*r]),
        Paint::Diamond(r) => format!("diamond {:?}", scene.rows[*r]),
        Paint::Lit(s) => format!("lit {}", table(s)),
        Paint::LitDiamond(s) => format!("lit diamond {}", table(s)),
        Paint::Translucent(s, t) => format!("translucent {t:?} {}", table(s)),
        Paint::Shadow => "shadow".into(),
        Paint::Pixel(c) => format!("pixel {c:?}"),
    }
}

/// Every drawing of `scene` in order as it shows: its place, depth test,
/// depth (its depth images by their contents), paint, group, flags,
/// object and stack, with its image's size and pixels. Two scenes compare
/// equal by this whatever image and row numbers they hold.
fn scene_shown(scene: &Scene) -> Vec<String> {
    use glasnost_render::scene::ItemDepth;
    scene
        .items
        .iter()
        .map(|i| {
            let image = &scene.images[i.image as usize];
            let depth = match i.depth {
                ItemDepth::Tile {
                    depth,
                    reference_row,
                } => format!(
                    "tile {:?} {reference_row}",
                    depth.map(|d| scene.images[d as usize].pixels().to_vec())
                ),
                ItemDepth::Profile {
                    profile,
                    bias,
                    companion,
                } => format!(
                    "profile {profile:?} {bias} {:?}",
                    companion.map(|(k, x, y)| (format!("{:?}", scene.signed[k as usize]), x, y))
                ),
            };
            format!(
                "{:?} {:?} {depth} {} {:?} {} {:?} {:?} {:?} {:?}",
                (i.x, i.y),
                i.test,
                paint_shown(scene, &i.paint),
                i.group,
                i.provisional,
                i.object,
                i.stack,
                (image.width(), image.height()),
                image.pixels(),
            )
        })
        .collect()
}

/// The scene after a catch-up of [`CATCH_UP`] ticks of a game with a map
/// building held in the cell pass, a wall of the map and a gun firing a
/// bolt, and the drawing's cosmetic stream after it. Each tick's world is
/// changed after the tick as `act` says; with `put_off` the frames of
/// every tick but the last are put off ([`Live::put_off_frame`]), without
/// it every tick is redrawn.
fn catch_up_scene(put_off: bool) -> (Scene, glasnost_sim::rng::Rng) {
    use glasnost_sim::combat::effects::Effect;
    use glasnost_sim::world::Pos;
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
    // A wall of the map's (FENCEX, the import's overlay 2, with data value 1), drawn in the cell pass.
    let wall = testkit::mid(SIZE, -5, 4);
    let fresh = testkit::mid(SIZE, -3, 5);
    {
        let cell = map.cells.get_mut(wall).unwrap();
        cell.overlay = 2;
        cell.overlay_data = 1;
    }
    let rules = rules(true);
    let setup = setup();
    let mut world = glasnost_sim::load::new_game(rules.clone(), &map, testkit::catalogue(), &setup)
        .expect("the synthetic game loads");
    let b = world
        .objects
        .values()
        .find(|o| o.kind == SimKind::Building && o.pos.cell() == at)
        .map(|o| o.id)
        .expect("the map's building");
    let firer = testkit::spawn(
        &mut world,
        SimKind::Infantry,
        "MANX",
        0,
        testkit::mid(SIZE, -4, 0),
        0,
    );
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
    live.set_bolt_colours(super::bolts::Colours {
        side: [10, 10, 10],
        alt_side: [5, 5, 5],
        main: [15, 15, 15],
    });
    let mut scene = built.scene;
    for _ in 0..4 {
        live.step();
    }
    live.redraw(&inputs, &mut scene);
    for k in 0..CATCH_UP {
        glasnost_sim::tick::step(&mut live.world);
        let w = &mut live.world;
        match k {
            0 => {
                // The map's building leaves its start pose (#97) and a new wall is built.
                w.obj_mut(b).unwrap().combat.anim_frame += 1;
                let cell = w.cells.grid.get_mut(fresh).unwrap();
                cell.overlay = 2;
                cell.overlay_data = 1;
            }
            1 => {
                // The building is back at its start pose; the map's wall changes.
                w.obj_mut(b).unwrap().combat.anim_frame -= 1;
                w.cells.grid.get_mut(wall).unwrap().overlay_data = 0;
            }
            2 => {
                // The map's wall is as it started; a long bolt is fired.
                w.cells.grid.get_mut(wall).unwrap().overlay_data = 1;
                w.effects.push(Effect::Bolt {
                    firer,
                    weapon: 0,
                    slot: 0,
                    burst: 0,
                    from: Pos {
                        x: 8 * 256,
                        y: 8 * 256,
                        z: 0,
                    },
                    to: Pos {
                        x: 13 * 256,
                        y: 11 * 256,
                        z: 0,
                    },
                    building: None,
                    vehicle_on_map: false,
                });
            }
            // An invented change of the current ambient, hundredths: the cells are lit again.
            3 => w.lighting.current += 37,
            _ => {}
        }
        live.after_tick();
        if put_off && k + 1 < CATCH_UP {
            live.put_off_frame(&inputs, &mut scene);
        } else {
            live.redraw(&inputs, &mut scene);
        }
    }
    (scene, live.cosmetic().clone())
}

/// The ticks of [`catch_up_scene`].
const CATCH_UP: usize = 6;

/// design: render (#138): a timing probe of a rejoining peer's catch-up
/// update (10 ticks, then the update's frame) on a synthetic game of 40
/// buildings and 300 infantry of HUTX and MANX, the drawing's work only
/// (after the tick, the redraws or the frames put off, and the frame's
/// redraw), each frame redrawn against the frames put off; with the
/// frames put off that fell back on a redraw. Run it in release with
/// `--ignored --nocapture`; it prints mean milliseconds an update.
#[test]
#[ignore = "a timing probe (#138)"]
fn catch_up_probe() {
    const SIDE: i32 = 64;
    const UPDATES: u32 = 20;
    const TICKS: u32 = 10;
    let (import, catalogue, assets) = import_with("");
    let map = testkit::empty_map(SIDE);
    let rules = rules(false);
    let setup = setup();
    let mut world = glasnost_sim::load::new_game(rules.clone(), &map, testkit::catalogue(), &setup)
        .expect("the synthetic game loads");
    for k in 0..40 {
        let at = testkit::mid(SIDE, k % 8 * 6 - 24, k / 8 * 6 - 15);
        testkit::spawn(&mut world, SimKind::Building, "HUTX", k as u8 % 2, at, 0);
    }
    for k in 0..300 {
        let at = testkit::mid(SIDE, k % 20 * 2 - 20, k / 20 * 2 - 14);
        testkit::spawn(&mut world, SimKind::Infantry, "MANX", k as u8 % 2, at, 0);
    }
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
    let run = |put_off: bool| {
        let (mut built, painter) = build_parts(&inputs, true);
        let mut live = Live::new(
            &inputs,
            &mut built,
            painter,
            (world.clone(), rules.clone(), setup.clone(), Vec::new()),
        );
        let mut scene = built.scene;
        live.redraw(&inputs, &mut scene);
        let mut drawing = std::time::Duration::ZERO;
        let mut fell_back = 0;
        for _ in 0..UPDATES {
            for _ in 0..TICKS {
                glasnost_sim::tick::step(&mut live.world);
                let t = std::time::Instant::now();
                live.after_tick();
                if put_off {
                    live.put_off_frame(&inputs, &mut scene);
                    fell_back += u32::from(!live.owes_frame());
                } else {
                    live.redraw(&inputs, &mut scene);
                }
                drawing += t.elapsed();
            }
            let t = std::time::Instant::now();
            if live.owes_frame() {
                live.redraw(&inputs, &mut scene);
            }
            drawing += t.elapsed();
        }
        (
            drawing.as_secs_f64() * 1000.0 / f64::from(UPDATES),
            fell_back,
            scene.items.len(),
        )
    };
    let (every, _, items) = run(false);
    let (put_off, fell_back, put_off_items) = run(true);
    assert_eq!(items, put_off_items);
    println!(
        "catch-up probe: {items} items; an update of {TICKS} ticks, drawing: every frame redrawn {every:.3} ms, frames put off {put_off:.3} ms ({fell_back} of {} put off fell back on a redraw)",
        UPDATES * TICKS
    );
}

/// design: render (#138) (the frames of ticks no frame sees are put off to the next redraw, which
/// draws the scene a redraw after every tick leaves); spec: S-0201 claim 6 (one drawn frame per
/// tick), S-0316 claims 5 and 9 (a bolt's draws are taken in every drawn frame, from the cosmetic
/// stream), S-0108 claim 2 and #97 (a map building that left its start pose is drawn live from
/// then on), S-0112 claims 1 to 9 (a wall changed in play is drawn from the game's cells from then
/// on), S-0104 claims 4 and 5 (a lighting change lights the cells again): a catch-up with its
/// frames put off shows what a catch-up redrawn after every tick shows, item by item, spot by spot
/// and pixel by pixel, and leaves the cosmetic stream where it leaves it.
#[test]
fn a_catch_up_with_its_frames_put_off_draws_what_every_frame_draws() {
    use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE};
    use glasnost_render::scene::{Filter, View};
    let (every, every_rng) = catch_up_scene(false);
    let (put_off, put_off_rng) = catch_up_scene(true);
    assert!(
        every.items.len() > every.cell_pass_len,
        "something is drawn live"
    );
    assert!(
        every.spots.iter().any(|g| g.at == usize::MAX),
        "the bolt is drawn"
    );
    assert_eq!(put_off.cell_pass_len, every.cell_pass_len, "the cell pass");
    assert_eq!(scene_shown(&put_off), scene_shown(&every), "the drawings");
    assert_eq!(put_off.spots, every.spots, "the spots");
    assert_eq!(put_off_rng, every_rng, "the cosmetic stream");
    // The whole scene composed on the CPU.
    let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for i in &every.items {
        let image = &every.images[i.image as usize];
        x0 = x0.min(i.x);
        y0 = y0.min(i.y);
        x1 = x1.max(i.x + image.width() as i32);
        y1 = y1.max(i.y + image.height() as i32);
    }
    let view = View {
        origin: (x0, y0),
        size: ((x1 - x0) as u32, (y1 - y0) as u32),
        reference: DEPTH_REFERENCE,
    };
    let compose = |s: &Scene| {
        s.compose(&view, &Filter::default(), &ComposeQuirks::default())
            .to_rgba()
    };
    assert!(compose(&put_off) == compose(&every), "the pixels");
}

/// spec: S-0104 claim 3 (under an effect, the house-colour tables are rebuilt with the chosen
/// set's tint; at "normal" each table goes back to its own tint, white here) and claim 5: a map
/// building held in the cell pass takes the effect's tint too (#75)
#[test]
fn a_cell_pass_building_takes_the_effect_tint() {
    use glasnost_render::compose::{Shade, row565};
    use glasnost_render::light::shade_table;
    use glasnost_render::palette::{TINT_EXEMPT, house_palette};
    use glasnost_sim::lighting::Choice;
    // An invented effect tint, hundredths.
    const TINT: [i32; 3] = [120, 80, 150];
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
    let hsv = super::build::owner_definition(&import, &options, b"Neutral").0;
    let pal = house_palette(&palettes.unit, hsv);
    let expect = |tint: [i32; 3]| {
        shade_table(&pal, tint, 53, Some(TINT_EXEMPT))
            .iter()
            .map(row565)
            .collect::<Vec<_>>()
    };
    // The house tables the building's lit items show.
    let tables = |scene: &Scene| {
        let lit: Vec<Shade> = items_of(scene, code)
            .iter()
            .filter_map(|i| match i.paint {
                Paint::Lit(s) | Paint::LitDiamond(s) => Some(s),
                _ => None,
            })
            .collect();
        assert!(!lit.is_empty(), "the building is drawn in the cell pass");
        lit.iter()
            .map(|s| scene.rows[s.base as usize..][..usize::from(s.rows)].to_vec())
            .collect::<Vec<_>>()
    };
    live.step();
    live.redraw(&inputs, &mut scene);
    assert!(
        tables(&scene).iter().all(|t| *t == expect([1000; 3])),
        "white light"
    );
    live.world.lighting.map.ion.tint = TINT;
    live.world.lighting.choice = Choice::Ion;
    live.redraw(&inputs, &mut scene);
    assert!(
        tables(&scene)
            .iter()
            .all(|t| *t == expect(TINT.map(|t| t * 10))),
        "the effect's tint"
    );
    assert_eq!(
        items_of(&scene, sim_code(b)).len(),
        0,
        "still in the cell pass"
    );
    live.world.lighting.choice = Choice::Normal;
    live.redraw(&inputs, &mut scene);
    assert!(
        tables(&scene).iter().all(|t| *t == expect([1000; 3])),
        "its own tint again"
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

/// design: #442 (the trace under the pointer, our own diagnostics): a
/// building's slot animation is listed with its building (id and type),
/// its slot, its type, its loop count and the tick in which the drawing
/// started it, and the building is among the objects in the pointer's cell.
#[test]
fn the_trace_names_a_buildings_slot_animation() {
    let (import, catalogue, assets) = import(true);
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
    let statics = built.objects;
    let cell = testkit::mid(SIZE, 0, 0);
    let b = testkit::spawn(&mut live.world, SimKind::Building, "HUTX", 0, cell, 0);
    // nocite: our own diagnostics (a test length)
    for _ in 0..4 {
        live.step();
        live.redraw(&inputs, &mut scene);
    }
    let names = live.slot_names(b);
    assert!(
        names.iter().any(|n| n.starts_with("Active1=SLOTL")),
        "the slot animation runs: {names:?}"
    );
    let pos = live.world.obj(b).expect("the building").pos;
    let text = live.trace_report(
        &scene,
        &statics,
        super::live::TraceAt {
            screen: (0, 0),
            world: glasnost_render::projection::project_lifted(pos.x, pos.y, pos.z),
            top: 0,
            cell,
        },
    );
    let all = text.join("\n");
    let line = text
        .iter()
        .find(|l| l.contains("slot Active1:"))
        .unwrap_or_else(|| panic!("the slot animation's line: {all}"));
    assert!(
        line.contains(&format!("building object #{} Building HUTX", b.0)),
        "{line}"
    );
    assert!(line.contains("slot Active1: SLOTL;"), "{line}");
    assert!(line.contains("loop count -1;"), "{line}");
    assert!(line.contains("created in tick "), "{line}");
    let objects = text
        .iter()
        .position(|l| l.starts_with("objects in the pointer's cell"))
        .expect("the objects' section");
    assert!(
        text[objects + 1..]
            .iter()
            .any(|l| l.contains(&format!("object #{} Building HUTX", b.0))),
        "{all}"
    );
}

/// One tick of the payment-animation run: the tick, whether the house was
/// paid in it, whether the harvester is *finishing*, the simulation's slot
/// animations on the refinery (slot and frame) and the drawing's (import
/// index and frame).
type PayRow = (i32, bool, bool, Vec<(String, i32)>, Vec<(usize, i32)>);

/// spec: S-0402 r86 claim 11 (*What a docking shows*: for a refinery whose
/// art names only the payment animation, the start row plays nothing, the
/// payment animation enters play at the first payment row and each start
/// is drawn from its start until it ends on its own or the empty-load row
/// removes it, whichever comes first) and scenario H29 part a (a load of
/// one tiberium), with S-0108 claim 20 (the drawing shows the simulation's
/// slot) — #444: the drawing's slot draws hold the payment animation in
/// every tick from the payment on while the simulation plays it, and
/// nothing on the refinery before. Invented types: HUTX as a
/// refinery-flag dock-unload building whose art names only PAYX in its
/// payment slot, HARVX a harvester of one bale of ORE.
#[test]
fn a_refinery_draws_its_payment_animation_from_the_payment_on() {
    use glasnost_rules::sim::AnimationType;
    use glasnost_rules::sim_econ::TiberiumType;
    use glasnost_sim::econ::harvest::{self, DockState, HarvestState};
    use glasnost_sim::rules::Behaviour;
    let rules_text = format!("{RULES}[Animations]\n0=PAYX\n");
    let hut = "[HUTX]\nSpecialAnim=PAYX\n";
    let art = format!("{}[PAYX]\n", ART.replacen("[HUTX]\n", hut, 1));
    let (mut import, catalogue, mut assets) = context_full(&rules_text, &art);
    import
        .arts
        .insert((Kind::Animations, 0), art_of("PAYX", "payx.shp"));
    // Canvas 20 × 20, 8 image frames (4 images, 4 shadows).
    assets.insert(
        b"payx.shp",
        shp((20, 20), 8, (SLOT_FRAME, SLOT_FRAME), (8, 8)),
    );
    let mut r = ruleset(false);
    r.registries
        .entry("animations".into())
        .or_default()
        .push("PAYX".into());
    r.sim.animations.insert(
        "PAYX".into(),
        AnimationType {
            frame_count: Some(8),
            middle_frame: Some(4),
            ..Default::default()
        },
    );
    r.registries
        .entry("tiberiums".into())
        .or_default()
        .push("ORE".into());
    r.sim.tiberiums.insert(
        "ORE".into(),
        TiberiumType {
            value: Some(25), // invented
            ..Default::default()
        },
    );
    let refinery = r.sim.buildings.get_mut("HUTX").unwrap();
    refinery.econ.refinery = Some(true);
    refinery.econ.dock_unload = Some(true);
    // spec: S-0402 Import mapping (the payment animation from the art's payment key, its normal name only)
    refinery.econ.slot_names = Some(std::collections::BTreeMap::from([(
        "SpecialAnim".to_owned(),
        ["PAYX".to_owned(), String::new()],
    )]));
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("HARVX".into());
    let mut harv = UnitType {
        strength: Some(400),
        speed: Some(7),
        rot: Some(6),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        harvester: Some(true),
        storage: Some(5),
        ..Default::default()
    };
    harv.econ.dock = Some(vec!["HUTX".into()]);
    r.sim.vehicles.insert("HARVX".into(), harv);
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let map = testkit::empty_map(SIZE);
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
    // The harvester docked as the simulation's claim tests dock one: on the
    // cell east of the refinery's foundation, in contact, holding the dock
    // slot, in unload.
    let top_left = testkit::mid(SIZE, -2, -1);
    let b = testkit::spawn(&mut live.world, SimKind::Building, "HUTX", 0, top_left, 0);
    let h = testkit::spawn(
        &mut live.world,
        SimKind::Vehicle,
        "HARVX",
        0,
        testkit::mid(SIZE, 0, -1),
        16384,
    );
    let o = live.world.obj_mut(h).unwrap();
    o.contact = Some(b);
    o.econ.harvest = Some(HarvestState {
        load: vec![1],
        ..Default::default()
    });
    live.world.obj_mut(b).unwrap().econ.dock = Some(DockState { holders: vec![h] });
    let t = live.world.tick;
    live.world
        .obj_mut(h)
        .unwrap()
        .beh
        .force(Behaviour::Unload, t, true);
    let type_index = live.world.obj(b).unwrap().type_index;
    let mut rows: Vec<PayRow> = Vec::new();
    // nocite: a test length (past the payment, the empty-load row and the contact's end)
    for _ in 0..600 {
        let credits = live.world.houses[0].credits;
        live.step();
        live.redraw(&inputs, &mut scene);
        let idx = live
            .import_index(SimKind::Building, type_index)
            .expect("the refinery is drawn");
        rows.push((
            live.world.tick,
            live.world.houses[0].credits != credits,
            harvest::state_of(&live.world, h).finishing,
            glasnost_sim::depot_anim::draws(&live.world, b)
                .into_iter()
                .map(|d| (format!("{:?}={}", d.slot, d.name), d.frame))
                .collect(),
            live.slot_draws(b, idx)
                .into_iter()
                .map(|l| (l.idx, l.frame))
                .collect(),
        ));
        if live.world.obj(h).is_some_and(|o| o.contact.is_none()) {
            break;
        }
    }
    let paid = rows
        .iter()
        .position(|r| r.1)
        .unwrap_or_else(|| panic!("a payment: {rows:#?}"));
    assert!(
        rows[..paid]
            .iter()
            .all(|r| r.3.is_empty() && r.4.is_empty()),
        "nothing plays on the refinery before the payment: {rows:#?}"
    );
    assert_eq!(
        rows[paid].3.first().map(|(n, _)| n.as_str()),
        Some("Special1=PAYX"),
        "the payment animation enters play with the payment: {rows:#?}"
    );
    let fin = rows
        .iter()
        .position(|r| r.2)
        .unwrap_or_else(|| panic!("finishing: {rows:#?}"));
    let ended = rows[paid..]
        .iter()
        .position(|r| r.3.is_empty())
        .map(|i| paid + i)
        .unwrap_or_else(|| panic!("the payment animation ends: {rows:#?}"));
    assert!(ended <= fin, "gone by the empty-load row: {rows:#?}");
    for r in &rows[paid..ended] {
        let frames: Vec<(usize, i32)> = r.3.iter().map(|(_, f)| (0, *f)).collect();
        assert_eq!(
            r.4, frames,
            "tick {}: the drawing shows the payment animation as the simulation plays it: {rows:#?}",
            r.0
        );
    }
    assert!(
        rows[ended..].iter().all(|r| r.4.is_empty()),
        "nothing on the refinery once it has gone: {rows:#?}"
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

/// The follow-up type's image frames are 5 × 5, so its items can be told
/// from [`SLOT_FRAME`]'s.
const FOLLOW_UP_FRAME: u32 = 5;

/// spec: S-0109 claim 15 (at its end, a slot animation whose type names a follow-up type continues as
/// the follow-up type) and S-0108 claim 11 (a building placed during the set-up starts its active
/// slot); design: render (#136): the drawing takes the follow-up type's images from the switch on
#[test]
fn a_slot_animation_is_drawn_as_its_follow_up_type_after_the_switch() {
    let rules_text = format!("{RULES}[Animations]\n0=SLOTL\n1=SLOTN\n");
    let hut = format!("[HUTX]\n{SLOT}");
    let art = format!(
        "{}[SLOTL]\nLayer=ground\nLoopCount=1\nNext=SLOTN\n[SLOTN]\nLayer=ground\nLoopCount=-1\n",
        ART.replacen("[HUTX]\n", &hut, 1)
    );
    let (mut import, catalogue, mut assets) = context_full(&rules_text, &art);
    import
        .arts
        .insert((Kind::Animations, 0), art_of("SLOTL", "slotl.shp"));
    import
        .arts
        .insert((Kind::Animations, 1), art_of("SLOTN", "slotn.shp"));
    assets.insert(
        b"slotl.shp",
        shp((20, 20), 8, (SLOT_FRAME, SLOT_FRAME), (8, 8)),
    );
    assets.insert(
        b"slotn.shp",
        shp((20, 20), 8, (FOLLOW_UP_FRAME, FOLLOW_UP_FRAME), (8, 8)),
    );
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
    let rules = rules(false);
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
    let mut live = Live::new(
        &inputs,
        &mut built,
        painter,
        (world, rules.clone(), setup, Vec::new()),
    );
    let mut scene = built.scene;
    // The slot images' widths, redraw by redraw.
    let mut widths = Vec::new();
    for _ in 0..40 {
        live.step();
        live.redraw(&inputs, &mut scene);
        let w: BTreeSet<u32> = items_of(&scene, sim_code(b))
            .iter()
            .filter(|i| matches!(i.paint, Paint::Lit(_)))
            .map(|i| scene.images[i.image as usize].width())
            .filter(|&w| w != BODY_FRAME)
            .collect();
        widths.push(w);
    }
    let first = widths
        .iter()
        .position(|w| w.contains(&FOLLOW_UP_FRAME))
        .unwrap_or_else(|| panic!("the follow-up type is drawn: {widths:?}"));
    assert!(
        widths[..first]
            .iter()
            .all(|w| *w == BTreeSet::from([SLOT_FRAME])),
        "SLOTL until the switch: {widths:?}"
    );
    assert!(
        widths[first..]
            .iter()
            .all(|w| *w == BTreeSet::from([FOLLOW_UP_FRAME])),
        "SLOTN from the switch on: {widths:?}"
    );
}
