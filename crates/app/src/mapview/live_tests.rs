//! Tests of the live map scene: pacing (S-0201), the ground layer (S-0102
//! claim 6), the dynamic pass (S-0101, S-0102 claim 5, S-0107) and the flow
//! of selection and orders through the input rules (S-0213, S-0214), on
//! invented data and the simulation's synthetic scenario.

use glasnost_convert::kinds::Kind;
use glasnost_convert::map::{DecodeContext, decode};
use glasnost_ini::{Document, Quirks};
use glasnost_render::compose::{DepthTest, Paint, Profile};
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::projection::project_lifted;
use glasnost_render::scene::{ItemDepth, Scene};
use glasnost_sim::targeting::{ClickOn, KeyCommand, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, Target};
use glasnost_sim::{save, testkit};

use super::build::{
    Attitude, InfantryFrame, Inputs, LiveBuilding, Mobile, Options, ShadowAnchor, build_parts,
    infantry_frame,
};
use super::live::{Entrant, GroundLayer, Live, Pacer, period_ms, sim_code};
use super::tests::{RULES, context, context_with, scene_map_text, shp};
use super::unload::{DoorView, Unloading};

/// spec: S-0201 claims 2, 4 and 5 (16 × s ms at s = 1 … 6, 8 ms at index 0)
#[test]
fn paced_periods() {
    assert_eq!(period_ms(0), 8);
    assert_eq!(period_ms(1), 16);
    assert_eq!(period_ms(4), 64);
    assert_eq!(period_ms(6), 96);
}

/// spec: S-0201 claims 2 and 3 (one tick per paced period, measured from
/// each cycle's own start; a late cycle is not made up)
#[test]
fn pacing_has_no_catch_up() {
    let mut p = Pacer::default();
    let speed = 2; // 32 ms
    assert!(p.due(1000, speed), "the first frame runs a tick");
    assert!(!p.due(1016, speed));
    assert_eq!(p.remaining(1016, speed), 16);
    assert!(p.due(1032, speed));
    // A stall of 300 ms gives one tick, not ten.
    let ticks = [1332, 1340, 1348, 1356]
        .iter()
        .filter(|&&t| p.due(t, speed))
        .count();
    assert_eq!(ticks, 1);
    // The next cycle counts from the late start: due again 32 ms later.
    assert!(!p.due(1363, speed));
    assert!(p.due(1364, speed));
    // At most one tick per call, however long the call came late.
    let mut q = Pacer::default();
    let n = (0..10).filter(|i| q.due(i * 1000, 0)).count();
    assert_eq!(n, 10);
}

/// spec: S-0102 claim 6 (enter before the first strictly greater key, so a
/// newcomer follows equal keys)
#[test]
fn ground_layer_entry() {
    let mut g = GroundLayer::default();
    let o = |n| Entrant::Object(ObjectId(n));
    g.enter(o(1), 5);
    g.enter(o(2), 10);
    g.enter(o(3), 3);
    g.enter(o(4), 5);
    assert_eq!(g.order().collect::<Vec<_>>(), [o(3), o(1), o(4), o(2)]);
}

/// spec: S-0102 claim 6 (fully ordered in the first frame showing the new
/// keys: a mover crossing several keys at once lands on the right side of
/// all of them, in either direction; a departure, D3)
#[test]
fn ground_layer_mover_crosses_several_keys_at_once() {
    let o = |n| Entrant::Object(ObjectId(n));
    let mut g = GroundLayer::default();
    for (n, k) in [(1, 10), (2, 20), (3, 30), (4, 40)] {
        g.enter(o(n), k);
    }
    // Object 1 moves past three keys in one tick.
    g.update(&[(o(1), 45), (o(2), 20), (o(3), 30), (o(4), 40)]);
    assert_eq!(g.order().collect::<Vec<_>>(), [o(2), o(3), o(4), o(1)]);
    // And back to the front in one tick, not one place per tick.
    g.update(&[(o(1), 0), (o(2), 20), (o(3), 30), (o(4), 40)]);
    assert_eq!(g.order().collect::<Vec<_>>(), [o(1), o(2), o(3), o(4)]);
}

/// spec: S-0102 claim 6 (equal-key objects retain their previous relative
/// order)
#[test]
fn ground_layer_equal_keys_keep_their_order() {
    let o = |n| Entrant::Object(ObjectId(n));
    let mut g = GroundLayer::default();
    for (n, k) in [(1, 10), (2, 20), (3, 30)] {
        g.enter(o(n), k);
    }
    // 3 moves to key 10: it was after 1, so it stays after 1.
    g.update(&[(o(1), 10), (o(2), 20), (o(3), 10)]);
    assert_eq!(g.order().collect::<Vec<_>>(), [o(1), o(3), o(2)]);
    // 1 moves to key 20: the order 3, 1, 2 keeps 1 before 2 (1 was first).
    g.update(&[(o(1), 20), (o(2), 20), (o(3), 10)]);
    assert_eq!(g.order().collect::<Vec<_>>(), [o(3), o(1), o(2)]);
    // Unchanged keys: unchanged order.
    g.update(&[(o(1), 20), (o(2), 20), (o(3), 10)]);
    assert_eq!(g.order().collect::<Vec<_>>(), [o(3), o(1), o(2)]);
}

/// spec: S-0102 claim 6 (a newcomer follows existing equal-key objects,
/// including one that moved to that key in the same frame; leavers drop out)
#[test]
fn ground_layer_newcomer_follows_equal_keys() {
    let o = |n| Entrant::Object(ObjectId(n));
    let (a, b, c, n) = (o(1), o(2), o(3), o(4));
    let mut g = GroundLayer::default();
    g.enter(a, 5);
    g.enter(b, 10);
    g.enter(c, 20);
    // C moves to key 5 and N enters with key 5 in the same frame. The
    // newcomer is listed first in `members` but still follows A and C.
    g.update(&[(n, 5), (a, 5), (b, 10), (c, 5)]);
    assert_eq!(g.order().collect::<Vec<_>>(), [a, c, n, b]);
    // B leaves.
    g.update(&[(a, 5), (c, 5), (n, 5)]);
    assert_eq!(g.order().collect::<Vec<_>>(), [a, c, n]);
    assert!(!g.contains(b));
}

/// spec: S-0107 F1, F2 (frame from stage, count and facing group)
#[test]
fn infantry_frames() {
    let f = InfantryFrame {
        start: 10,
        count: 6,
        multiplier: 6,
        stage: 8,
    };
    // Map north (facing 0) is group 7; stage 8 mod 6 = 2.
    assert_eq!(infantry_frame(&f, 0), 10 + 2 + 6 * 7);
    // Map east (16384) is group 5.
    assert_eq!(infantry_frame(&f, 16384), 10 + 2 + 6 * 5);
    // M ≤ 0: no facing groups; C = 0 counts as 1.
    let g = InfantryFrame {
        start: 4,
        count: 0,
        multiplier: 0,
        stage: 9,
    };
    assert_eq!(infantry_frame(&g, 30000), 4);
}

/// spec: S-0101 F1 and F2 (the drawn point is the projection of the
/// position), S-0102 claim 5 (sort key X + Y), S-0107 claim 2 (frame) and claim 19
/// (no body-sprite shadow above the surface)
#[test]
fn dynamic_pass_places_mobiles_at_their_projected_point() {
    let (import, catalogue, assets) = context();
    let doc = Document::parse(&scene_map_text(), Quirks::default(), &mut Vec::new());
    let map = decode(
        &doc,
        &DecodeContext {
            catalogue: Some(&catalogue),
            ..Default::default()
        },
    )
    .map;
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (built, mut painter) = build_parts(&inputs, true);
    // The live build leaves the map's infantry to the simulation.
    assert!(built.scene.items.iter().all(|i| {
        i.object
            .is_none_or(|o| built.objects[o as usize].kind != "infantry")
    }));
    assert_eq!(built.scene.items.len(), built.scene.cell_pass_len);
    let mut scene: Scene = built.scene;
    let man = |pos, object| Mobile {
        kind: Kind::Infantry,
        idx: 0,
        owner: b"Neutral".to_vec(),
        pos,
        body: 16384,
        turret: 16384,
        elevation: 0,
        anim: 0,
        infantry: Some(InfantryFrame {
            start: 3,
            count: 1,
            multiplier: 2,
            stage: 0,
        }),
        object,
        harvesting: None,
        shadow: None,
        alternative: false,
        attitude: Attitude::None,
        disabled: false,
        above: 0,
        protection: None,
    };
    let pos = (2 * 256 + 40, 3 * 256 + 200, 104);
    let (entries, _) = painter.mobiles(&inputs, &mut scene, &[man(pos, sim_code(ObjectId(7)))]);
    assert_eq!(entries.len(), 1);
    let e = &entries[0];
    assert_eq!(e.key, i64::from(pos.0 + pos.1));
    let body = e.items.iter().find(|i| i.paint != Paint::Shadow).unwrap();
    let (px, py) = project_lifted(pos.0, pos.1, pos.2);
    // Canvas 20 × 20 centred on the point (S-0102 claim 19), frame rectangle at (9, 9).
    assert_eq!((body.x, body.y), (px - 10 + 9, py - 10 + 9));
    assert_eq!(body.object, Some(sim_code(ObjectId(7))));
    // Frame 3 + 2 × group 5 = 13 (pixel value frame + 1).
    assert_eq!(scene.images[body.image as usize].pixels()[0], 14);
    // Drawing the same frame elsewhere reuses its image.
    let images = scene.images.len();
    let moved = (pos.0 + 300, pos.1 - 100, 0);
    let (entries, _) = painter.mobiles(&inputs, &mut scene, &[man(moved, 1)]);
    let body = entries[0]
        .items
        .iter()
        .find(|i| i.paint != Paint::Shadow)
        .unwrap();
    let (px, py) = project_lifted(moved.0, moved.1, moved.2);
    assert_eq!((body.x, body.y), (px - 1, py - 1));
    assert_eq!(scene.images.len(), images);
    // spec: S-0107 claim 19 (height above the surface 0: the body-sprite shadow; above 0, as a
    // dropping paratrooper: none, the generic shadow left undrawn until Q-1525 names its sprite)
    let mut shadows = |above| {
        let m = Mobile {
            above,
            ..man(pos, 1)
        };
        let (entries, _) = painter.mobiles(&inputs, &mut scene, &[m]);
        entries[0]
            .items
            .iter()
            .filter(|i| i.paint == Paint::Shadow)
            .count()
    };
    assert_eq!(shadows(0), 1);
    assert_eq!(shadows(104), 0);
    assert_eq!(shadows(-20), 0);
}

/// spec: S-0402 claim 5a and F8 (the fixed harvesting sprite: its F8 image
/// frame centred on the projected anchor, composited before (under) the
/// vehicle's body and shadow, with the anim palette and no house-colour remap at
/// the vehicle's brightness, flat at U − 2 without writing depth (S-0102 claim 13); a
/// missing file draws nothing)
#[test]
fn harvesting_sprite_is_drawn_under_the_vehicle_body() {
    use super::harvesting::{Harvesting, SPRITE_FILE};
    let (import, catalogue, mut assets) = context();
    let doc = Document::parse(&scene_map_text(), Quirks::default(), &mut Vec::new());
    let map = decode(
        &doc,
        &DecodeContext {
            catalogue: Some(&catalogue),
            ..Default::default()
        },
    )
    .map;
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let pos = (2 * 256 + 128, 3 * 256 + 128, 0);
    let anchor = (pos.0 + 21, pos.1 - 22, pos.2); // any anchor: the painter centres on the one it is given
    let car = |harvesting| Mobile {
        kind: Kind::Vehicles,
        idx: 0,
        owner: b"Neutral".to_vec(),
        pos,
        body: 8192,
        turret: 8192,
        elevation: 0,
        anim: 0,
        infantry: None,
        object: sim_code(ObjectId(9)),
        harvesting,
        shadow: None,
        alternative: false,
        attitude: Attitude::None,
        disabled: false,
        above: 0,
        protection: None,
    };
    let h = Some(Harvesting { frame: 97, anchor });
    // Without the file: no image, no failure.
    {
        let inputs = Inputs {
            map: &map,
            import: &import,
            catalogue: &catalogue,
            palettes: &palettes,
            assets: &assets,
            options: &options,
            rules: None,
            cells: None,
        };
        let (built, mut painter) = build_parts(&inputs, true);
        let mut scene = built.scene;
        let (plain, _) = painter.mobiles(&inputs, &mut scene, &[car(None)]);
        let (missing, _) = painter.mobiles(&inputs, &mut scene, &[car(h)]);
        assert_eq!(missing[0].items.len(), plain[0].items.len());
    }
    // A synthetic harvesting sprite of 120 image frames (8 groups of 15),
    // canvas 30 × 20, frame i of pixel value i + 1.
    assets.insert(SPRITE_FILE, shp((30, 20), 120, (2, 2), (14, 9)));
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (built, mut painter) = build_parts(&inputs, true);
    let mut scene = built.scene;
    let (plain, _) = painter.mobiles(&inputs, &mut scene, &[car(None)]);
    let (entries, _) = painter.mobiles(&inputs, &mut scene, &[car(h)]);
    let items = &entries[0].items;
    assert_eq!(items.len(), plain[0].items.len() + 1);
    // First: the harvesting image, before every item of the body.
    let harv = &items[0];
    assert_eq!(&items[1..], &plain[0].items[..]);
    assert_eq!(harv.object, Some(sim_code(ObjectId(9))));
    assert_eq!(scene.images[harv.image as usize].pixels()[0], 98);
    // Centred on the projected anchor (S-0101 claim 1; S-0102 F6).
    let (px, py) = project_lifted(anchor.0, anchor.1, anchor.2);
    assert_eq!((harv.x, harv.y), (px - 15 + 14, py - 10 + 9));
    assert_eq!(harv.test, DepthTest::Reader);
    // The anim palette's table, not the owner's house table.
    let lit = |i: &glasnost_render::scene::Item| match i.paint {
        Paint::Lit(s) => s,
        p => panic!("not lit: {p:?}"),
    };
    let body = items[1..]
        .iter()
        .find(|i| i.paint != Paint::Shadow)
        .unwrap();
    assert_ne!(lit(harv).base, lit(body).base);
    // spec: S-0402 claim 5a (the vehicle's brightness, S-0104 claim 13: the same as its body's)
    assert_eq!(lit(harv).rho, lit(body).rho);
    // spec: S-0102 claim 13 (flat, no companion, a reader that writes no depth; no longer provisional)
    assert!(matches!(
        harv.depth,
        ItemDepth::Profile {
            profile: Profile::Flat,
            companion: None,
            ..
        }
    ));
    assert!(!harv.provisional);
}

/// A live game on the simulation's synthetic scenario (its map has no art,
/// so nothing is drawn; the plumbing is what is tested).
fn synthetic_live() -> (Live, Scene) {
    let (import, catalogue, assets) = context();
    let map = testkit::map();
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (mut built, painter) = build_parts(&inputs, true);
    let world = testkit::new_game(5);
    let live = Live::new(
        &inputs,
        &mut built,
        painter,
        (world, testkit::rules(), testkit::setup(5), Vec::new()),
    );
    (live, built.scene)
}

/// spec: S-0214 claim 25 (mobile candidates take places in their current
/// drawing order, not object order; static drawings between them consume
/// no place and do not reverse their priority)
#[test]
fn mobile_candidates_follow_the_drawing_order() {
    let mut g = GroundLayer::default();
    g.enter(Entrant::Object(ObjectId(3)), 10);
    g.enter(Entrant::Object(ObjectId(1)), 20);
    g.enter(Entrant::Static(0), 15);
    g.enter(Entrant::Object(ObjectId(2)), 5);
    assert_eq!(
        g.objects().collect::<Vec<_>>(),
        [ObjectId(2), ObjectId(3), ObjectId(1)]
    );
}

/// spec: S-0102 claims 1, 4 and 7 and S-0214 claim 25 (the drawing order,
/// and so the order of the mobile candidates, runs layer by layer: an
/// object put into the air layer leaves the ground layer and is drawn after
/// it; with quirks.layer_change_on_events_only off it is re-evaluated every
/// tick, S-0102 Edge cases)
#[test]
fn mobiles_are_drawn_layer_by_layer() {
    use super::slots::Layer;
    let (mut live, _) = synthetic_live();
    let a = own_mobile(&live, 0);
    let b = own_mobile(&live, 1);
    assert_eq!(live.layer_of(a), Some(Layer::Ground));
    let before = live.mobile_order();
    // Under the default switch a rocket state appearing is no moment.
    live.world.obj_mut(a).unwrap().rocket = Some(Default::default());
    live.after_tick();
    assert_eq!(live.layer_of(a), Some(Layer::Ground));
    assert_eq!(live.mobile_order(), before);
    // Evaluated every tick: a goes to the air layer, then b after it.
    live.set_view_quirks(super::quirks::ViewQuirks {
        layer_change_on_events_only: false,
        ..Default::default()
    });
    live.after_tick();
    live.world.obj_mut(b).unwrap().rocket = Some(Default::default());
    live.after_tick();
    assert_eq!(
        live.layer_list(Layer::Air),
        [Entrant::Object(a), Entrant::Object(b)]
    );
    assert!(live.ground().objects().all(|id| id != a && id != b));
    let order = live.mobile_order();
    assert_eq!(order.len(), before.len());
    assert_eq!(order[order.len() - 2..], [a, b]);
}

fn own_mobile(live: &Live, h: u8) -> ObjectId {
    testkit::house_objects(&live.world, h)
        .into_iter()
        .find(|&id| live.world.obj(id).is_some_and(|o| o.kind.is_mobile()))
        .expect("the scenario gives each house units")
}

/// spec: S-0214 claim 12 (a click selects), S-0213 claim 1 and S-0201
/// claim 14 (orders refused during ticks 0 to 3), S-0213 table T2 (stop)
#[test]
fn selection_and_orders_flow_through_the_input_rules() {
    let (mut live, _) = synthetic_live();
    let house = HouseId(0);
    let unit = own_mobile(&live, 0);
    // The ground layer holds every mobile object.
    let mobiles = live
        .world
        .objects
        .values()
        .filter(|o| o.kind.is_mobile())
        .count();
    assert_eq!(
        live.ground()
            .order()
            .filter(|e| matches!(e, Entrant::Object(_)))
            .count(),
        mobiles
    );
    let mut selection = Vec::new();
    let r = live.click(
        house,
        &mut selection,
        ClickOn::Object(unit),
        Modifiers::default(),
    );
    assert_eq!(selection, [unit]);
    assert!(r.selection_changed);
    assert_eq!(r.issued, 0);
    let start = live.world.obj(unit).unwrap().pos.cell();
    let to = glasnost_map::CellXY::new(start.x + 3, start.y);
    // Tick 0: the order block refuses it.
    let r = live.click(
        house,
        &mut selection,
        ClickOn::Cell(to),
        Modifiers::default(),
    );
    assert_eq!((r.issued, r.refused), (0, 1));
    while live.world.tick < 4 {
        live.step();
    }
    let r = live.click(
        house,
        &mut selection,
        ClickOn::Cell(to),
        Modifiers::default(),
    );
    assert_eq!((r.issued, r.refused), (1, 0));
    live.step();
    assert_eq!(live.world.obj(unit).unwrap().dest, Some(Target::Cell(to)));
    let r = live.key(house, &mut selection, KeyCommand::Stop);
    assert_eq!(r.issued, 1);
    // Another house's unit is not ordered by this house's selection.
    let other = own_mobile(&live, 1);
    let mut foreign = vec![other];
    let r = live.key(house, &mut foreign, KeyCommand::Stop);
    assert_eq!(r.issued, 0);
}

/// Drawing reads the game and never changes it (CONTRIBUTING.md: rendering
/// only reads simulation state); spec: S-0203 (the state hash).
#[test]
fn drawing_leaves_the_game_alone() {
    let (mut live, mut scene) = synthetic_live();
    let (import, catalogue, assets) = context();
    let map = testkit::map();
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let mut reference = testkit::new_game(5);
    for _ in 0..12 {
        live.step();
        live.redraw(&inputs, &mut scene);
        glasnost_sim::tick::step(&mut reference);
    }
    assert_eq!(save::state_hash(&live.world), save::state_hash(&reference));
}

/// spec: S-0112 claims 1 and 6 (an overlay is drawn from its cell's overlay and data value: a
/// wall built, crushed or destroyed in play shows its new state), S-0210 claim 8, S-0406 claim 10
#[test]
fn overlays_changed_in_play_are_redrawn() {
    let (mut live, mut scene) = synthetic_live();
    let (import, catalogue, assets) = context();
    let map = testkit::map();
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    live.redraw(&inputs, &mut scene);
    let before = scene.items.len();
    let c = live
        .world
        .cells
        .grid
        .iter()
        .find(|(_, x)| x.overlay == glasnost_map::NO_OVERLAY)
        .map(|(c, _)| c)
        .expect("an empty cell");
    // FENCEX (the import's overlay 2, upright) with data value 1: frame 1 (pixel 2).
    {
        let cell = live.world.cells.grid.get_mut(c).unwrap();
        cell.overlay = 2;
        cell.overlay_data = 1;
    }
    live.redraw(&inputs, &mut scene);
    assert!(scene.items.len() > before);
    let (qx, qy) = project_lifted(256 * c.x + 128, 256 * c.y + 128, 0);
    let fence: Vec<_> = scene
        .items
        .iter()
        .filter(|i| i.object.is_none() && scene.images[i.image as usize].pixels()[0] == 2)
        .filter(|i| (i.x - qx).abs() < 16 && (i.y - qy).abs() < 32)
        .collect();
    assert!(!fence.is_empty(), "the new wall is drawn at its cell");
    // Removed again: the picture is as it was.
    {
        let cell = live.world.cells.grid.get_mut(c).unwrap();
        cell.overlay = glasnost_map::NO_OVERLAY;
        cell.overlay_data = 0;
    }
    live.redraw(&inputs, &mut scene);
    assert_eq!(scene.items.len(), before);
}

// ---------------------------------------------------------------------------
// Combat in the live view (synthetic combat ruleset, invented overlay art)
// ---------------------------------------------------------------------------

/// The test context's assets plus invented overlay sprites and palette:
/// a pip sprite of 20 frames on a canvas of 8 × 6 (frame i a 2 × 2 block of
/// value i + 1 at (3, 2)), a bracket sprite of 2 frames on 36 × 10, and an
/// overlay palette.
fn overlay_assets() -> super::build::Assets {
    let (_, _, mut assets) = context();
    assets.insert(
        super::combat::PIP_SPRITE,
        super::tests::shp((8, 6), 20, (2, 2), (3, 2)),
    );
    assets.insert(
        super::combat::BRACKET_SPRITE,
        super::tests::shp((36, 10), 2, (34, 4), (1, 3)),
    );
    let pal: Vec<u8> = (0..768u32).map(|i| (i * 7 % 64) as u8).collect();
    assets.insert(super::combat::OVERLAY_PALETTE, pal);
    assets
}

/// A live game of the synthetic combat ruleset on an empty map of 24, with
/// a gun cart of house 0 and a rifle walker of house 1 `gap` cells apart,
/// both in guard. Returns the game, its scene, the map, the import and
/// catalogue, and the two objects.
#[allow(clippy::type_complexity)]
fn combat_live(
    gap: i32,
) -> (
    Live,
    Scene,
    glasnost_map::Map,
    glasnost_convert::Import,
    glasnost_convert::theater::Catalogue,
    ObjectId,
    ObjectId,
) {
    use glasnost_sim::rules::Kind as SimKind;
    let (import, catalogue, _) = context();
    let assets = overlay_assets();
    let map = testkit::empty_map(24);
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (mut built, painter) = build_parts(&inputs, true);
    let mut world = testkit::game_with(&testkit::combat_ruleset(), 24, 3);
    let cart = testkit::spawn(
        &mut world,
        SimKind::Vehicle,
        "CART",
        0,
        testkit::mid(24, -gap / 2, 0),
        16384,
    );
    let walker = testkit::spawn(
        &mut world,
        SimKind::Infantry,
        "WALKER",
        1,
        testkit::mid(24, gap - gap / 2, 0),
        49152,
    );
    let rules = world.rules_arc();
    let live = Live::new(
        &inputs,
        &mut built,
        painter,
        (world, rules, testkit::setup(3), Vec::new()),
    );
    (live, built.scene, map, import, catalogue, cart, walker)
}

/// spec: S-0309 claim 1 (units in guard find targets in range by
/// themselves), S-0101 F1 and F2 (a projectile is drawn at its projected
/// point), S-0111 claims 3, 4, 5 and 6 (the health bars of a selected and
/// a hovered unit: bracket, filled segments, colour frames, places), S-0312
/// claim 15 (a destroyed unit leaves the picture). The projectile, shot and
/// detonation markers are our own provisional placeholders; invisible
/// projectiles are not drawn.
#[test]
fn combat_draws_projectiles_bursts_and_health_bars() {
    use glasnost_render::scene::Group;

    use super::combat::{INFANTRY_SEGMENTS, Ratios, UNIT_SEGMENTS, filled_segments};

    let (mut live, mut scene, map, import, catalogue, cart, walker) = combat_live(3);
    let assets = overlay_assets();
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let mut seen_projectile = false;
    let mut seen_burst = false;
    let mut hurt = false;
    for _ in 0..400 {
        live.step();
        live.redraw(&inputs, &mut scene);
        let effects: Vec<_> = scene
            .items
            .iter()
            .filter(|i| i.group == Group::Effects)
            .collect();
        assert!(effects.iter().all(|i| i.provisional));
        // Every visible projectile has a 7 × 7 marker centred on its point;
        // invisible ones (the rifle's) have none.
        let rules = live.rules.clone();
        for p in live.world.projectiles.values() {
            let inviso = rules
                .combat
                .projectile_of(p.weapon)
                .is_some_and(|k| k.inviso);
            let (px, py) = project_lifted(p.pos.x, p.pos.y, p.pos.z);
            let found = effects.iter().any(|i| {
                (i.x + 3, i.y + 3) == (px, py) && scene.images[i.image as usize].width() == 7
            });
            assert_eq!(found, !inviso, "projectile marker iff visible");
            seen_projectile |= !inviso;
        }
        seen_burst |= effects
            .iter()
            .any(|i| scene.images[i.image as usize].width() > 9);
        let health = |l: &Live, id| l.world.obj(id).map_or(0, |o| o.health);
        if !hurt
            && health(&live, cart) < 300
            && health(&live, walker) < 100
            && health(&live, walker) > 0
        {
            hurt = true;
            // The cart selected, the walker under the pointer.
            assert!(live.set_marks(&[cart], Some(walker)));
            assert!(!live.set_marks(&[cart], Some(walker)));
            live.redraw_status(&inputs, &mut scene);
            let r = &live.rules;
            let ratios = Ratios {
                yellow: r.general.condition_yellow,
                red: r.general.condition_red,
                mode: r.mode,
            };
            let status = |id| -> Vec<_> {
                scene
                    .items
                    .iter()
                    .filter(|i| i.group == Group::Status && i.object == Some(sim_code(id)))
                    .copied()
                    .collect()
            };
            let anchor = |id: ObjectId| {
                let p = live.world.obj(id).unwrap().pos;
                project_lifted(p.x, p.y, p.z)
            };
            // Cart: bracket frame 0 at (1, −26), then f segments from (−15, −25).
            let c = status(cart);
            let f = filled_segments(health(&live, cart), 300, UNIT_SEGMENTS, ratios.mode);
            assert_eq!(c.len() as i32, 1 + f);
            let (ax, ay) = anchor(cart);
            // Bracket canvas 36 × 10, frame at (1, 3).
            assert_eq!((c[0].x, c[0].y), (ax + 1 - 18 + 1, ay - 26 - 5 + 3));
            assert_eq!(scene.images[c[0].image as usize].pixels()[0], 1);
            // Pip canvas 8 × 6, frame at (3, 2): segment i at (−15 + 2i, −25).
            for (i, s) in c[1..].iter().enumerate() {
                assert_eq!(
                    (s.x, s.y),
                    (ax - 15 + 2 * i as i32 - 4 + 3, ay - 25 - 3 + 2)
                );
                let frame = scene.images[s.image as usize].pixels()[0] - 1;
                assert!((16..=18).contains(&frame));
            }
            // Walker, hovered: no bracket, f of 8 segments from (−5, −24).
            let w = status(walker);
            let f = filled_segments(health(&live, walker), 100, INFANTRY_SEGMENTS, ratios.mode);
            assert_eq!(w.len() as i32, f);
            let (ax, ay) = anchor(walker);
            assert_eq!((w[0].x, w[0].y), (ax - 5 - 4 + 3, ay - 24 - 3 + 2));
        }
        if live.world.obj(cart).is_none() || live.world.obj(walker).is_none() {
            break;
        }
    }
    assert!(seen_projectile, "a shell was drawn in flight");
    assert!(seen_burst, "a detonation burst was drawn");
    assert!(hurt, "both took damage while alive");
    assert!(live.counts.shots > 0 && live.counts.detonations > 0);
    // Run on until one of them is destroyed: it leaves the status extras.
    for _ in 0..3000 {
        if live.world.obj(cart).is_none() || live.world.obj(walker).is_none() {
            break;
        }
        live.step();
    }
    live.redraw(&inputs, &mut scene);
    let dead = if live.world.obj(cart).is_none() {
        cart
    } else {
        walker
    };
    assert!(live.world.obj(dead).is_none(), "one of them is destroyed");
    assert!(live.counts.kills >= 1);
    assert!(
        scene.items.iter().all(|i| i.object != Some(sim_code(dead))),
        "a destroyed unit is not drawn"
    );
    assert!(live.world.effects.is_empty(), "the view drains the events");
}

/// spec: S-0214 claim 12 and S-0213 table T2 (a right click on an enemy with
/// a selection gives an attack order through the input rules), S-0201
/// claim 14 (not during ticks 0 to 3)
#[test]
fn right_click_on_an_enemy_orders_an_attack() {
    let (mut live, _, _, _, _, cart, walker) = combat_live(14);
    while live.world.tick < 4 {
        live.step();
    }
    let house = HouseId(0);
    let mut selection = Vec::new();
    live.click(
        house,
        &mut selection,
        ClickOn::Object(cart),
        Modifiers::default(),
    );
    assert_eq!(selection, [cart]);
    let r = live.click(
        house,
        &mut selection,
        ClickOn::Object(walker),
        Modifiers::default(),
    );
    assert_eq!((r.issued, r.refused), (1, 0));
    live.step();
    assert_eq!(
        live.world.obj(cart).unwrap().target,
        Some(Target::Object(walker))
    );
}

/// The test context's building HUTX given the unloading sprites of S-0108
/// claims 22 and 23 named in `roles` (roles as the importer names them),
/// each placed at its own frame x so the drawn sprite can be told apart;
/// the door sprite has 8 image frames, the under-door sprites 2.
fn unloading_context(
    roles: &[&'static str],
) -> (
    glasnost_convert::Import,
    glasnost_convert::theater::Catalogue,
    super::build::Assets,
) {
    let (mut import, catalogue, mut assets) = context();
    let hut = import.arts.get_mut(&(Kind::Buildings, 0)).unwrap();
    for &(role, file, frames, x) in &[
        ("deploying", "hutxdep.shp", 4, 7),
        ("roof_deploying", "hutxroof.shp", 4, 8),
        ("under_door", "hutxud.shp", 2, 5),
        ("under_roof_door", "hutxurd.shp", 2, 4),
        ("door", "hutxdoor.shp", 8, 3),
    ] {
        if roles.contains(&role) {
            hut.building_sprites.insert(role, file.as_bytes().to_vec());
            assets.insert(file.as_bytes(), shp((40, 30), frames, (4, 4), (x, 8)));
        }
    }
    (import, catalogue, assets)
}

/// Each item the live painter draws for HUTX at cell (2, 2), height 0,
/// with `unloading`: (frame x on its canvas, depth test, profile, bias,
/// first pixel).
fn unloading_items(
    roles: &[&'static str],
    unloading: Option<Unloading>,
) -> Vec<(i32, DepthTest, Profile, i32, u8)> {
    unloading_items_with(roles, unloading, None, (256, 256))
}

/// As [`unloading_items`], with HUTX's image renamed and art sections
/// `art` added (`Some((image, art))`), and with `health`.
fn unloading_items_with(
    roles: &[&'static str],
    unloading: Option<Unloading>,
    art: Option<(&str, &str)>,
    health: (i32, i32),
) -> Vec<(i32, DepthTest, Profile, i32, u8)> {
    let (mut import, catalogue, assets) = unloading_context(roles);
    if let Some((image, text)) = art {
        import.arts.get_mut(&(Kind::Buildings, 0)).unwrap().image = image.as_bytes().to_vec();
        let f = import.art_file.as_mut().expect("an art file");
        f.bytes.extend_from_slice(text.as_bytes());
    }
    live_building_items((import, catalogue, assets), unloading, None, health)
}

/// Each item the live painter draws for HUTX at cell (2, 2), height 0, in
/// the context `ctx`, with `unloading`, gate door `gate_door` and `health`.
fn live_building_items(
    ctx: (
        glasnost_convert::Import,
        glasnost_convert::theater::Catalogue,
        super::build::Assets,
    ),
    unloading: Option<Unloading>,
    gate_door: Option<DoorView>,
    health: (i32, i32),
) -> Vec<(i32, DepthTest, Profile, i32, u8)> {
    let (import, catalogue, assets) = ctx;
    let doc = Document::parse(&scene_map_text(), Quirks::default(), &mut Vec::new());
    let map = decode(
        &doc,
        &DecodeContext {
            catalogue: Some(&catalogue),
            ..Default::default()
        },
    )
    .map;
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (built, mut painter) = build_parts(&inputs, true);
    let mut scene: Scene = built.scene;
    let cell = glasnost_map::CellXY::new(2, 2);
    let (drawn, _) = painter.buildings(
        &inputs,
        &mut scene,
        &[LiveBuilding {
            idx: 0,
            cell,
            owner: b"Neutral".to_vec(),
            health,
            facing: 0,
            anim: 0,
            occupied: false,
            slots: Vec::new(),
            object: sim_code(ObjectId(5)),
            unloading,
            gate_door,
            protection: None,
        }],
    );
    // spec: S-0108 claim 8 (centred on the foundation's top corner; canvas 40 wide)
    let (ax, _) = project_lifted(256 * cell.x, 256 * cell.y, 0);
    drawn[0]
        .iter()
        .map(|i| {
            let (profile, bias) = match i.depth {
                ItemDepth::Profile { profile, bias, .. } => (profile, bias),
                ItemDepth::Tile { .. } => panic!("not a profile item"),
            };
            (
                i.x - (ax - 20),
                i.test,
                profile,
                bias,
                scene.images[i.image as usize].pixels()[0],
            )
        })
        .collect()
}

const UNLOAD_ROLES: [&str; 5] = [
    "deploying",
    "roof_deploying",
    "under_door",
    "under_roof_door",
    "door",
];

fn unloading_with(roof: bool, door: DoorView) -> Option<Unloading> {
    Some(Unloading { roof, door })
}

/// spec: S-0108 claims 22 and 23 (outside unloading the body sprite and no unloading, under-door
/// or door image), S-0102 claim 13 (body −L(z) − 2 + the normal depth adjustment)
#[test]
fn no_unloading_images_outside_unloading() {
    let items = unloading_items(&UNLOAD_ROLES, None);
    // Body (frame x 6), shadow and bib only.
    assert_eq!(items.len(), 3, "{items:?}");
    assert_eq!(items[0], (6, DepthTest::Writer, Profile::Upright, -2, 2));
    assert!(items.iter().all(|i| i.0 == 6));
}

/// spec: S-0108 claim 22 (an unmarked unit: the deploying sprite replaces the body, frame f with
/// the claim 3 clamp, and its shadow; the under-door sprite, frame 0 undamaged), claim 23 (the door
/// image with D = 0: frame 0, at the claim 8 anchor), S-0102 claim 13 (deploying −L(z) − 22,
/// upright, writes; under-door −L(z) − 2, flat, writes; door −L(z) − 7, flat, no depth write),
/// claim 16 (body, shadow, bib, under-door)
#[test]
fn deploying_under_door_and_door_while_unloading() {
    let items = unloading_items(&UNLOAD_ROLES, unloading_with(false, DoorView::Closed));
    assert_eq!(items.len(), 5, "{items:?}");
    // Deploying sprite (frame x 7), idle frame 1 (pixel 2); its shadow 1 + 2 = 3 (pixel 4).
    assert_eq!(items[0], (7, DepthTest::Writer, Profile::Upright, -22, 2));
    assert_eq!(items[1], (7, DepthTest::Writer, Profile::Flat, -4, 4));
    // The bib keeps its own sprite.
    assert_eq!(items[2], (6, DepthTest::Writer, Profile::Flat, -3, 2));
    // Under-door sprite (x 5), frame 0.
    assert_eq!(items[3], (5, DepthTest::Writer, Profile::Flat, -2, 1));
    // Door sprite (x 3) in the object pass, frame 0.
    assert_eq!(items[4], (3, DepthTest::Reader, Profile::Flat, -7, 1));
}

/// spec: S-0108 claim 22 (a jumpjet or balloon-hover unit: the roof-deploying and under-roof-door
/// sprites), claim 23 (the door image whatever the unit), S-0102 claim 13 (roof-deploying
/// −L(z) − 42)
#[test]
fn roof_deploying_while_unloading_a_marked_unit() {
    let items = unloading_items(&UNLOAD_ROLES, unloading_with(true, DoorView::Open));
    assert_eq!(items.len(), 5, "{items:?}");
    assert_eq!(items[0], (8, DepthTest::Writer, Profile::Upright, -42, 2));
    assert_eq!(items[3], (4, DepthTest::Writer, Profile::Flat, -2, 1));
    assert_eq!(items[4], (3, DepthTest::Reader, Profile::Flat, -7, 1));
}

/// spec: S-0108 claim 22 and Edge cases (quirks.roof_unload_no_fallback, on by default: a marked
/// unit from a building without roof sprites keeps the body sprite with its normal depth adjustment
/// and draws no under-door image, though the ordinary ones exist), claim 23 (the door image stays)
#[test]
fn roof_unload_without_roof_sprites_has_no_fallback() {
    let roles = ["deploying", "under_door", "door"];
    let items = unloading_items(&roles, unloading_with(true, DoorView::Closed));
    assert_eq!(items.len(), 4, "{items:?}");
    assert_eq!(items[0], (6, DepthTest::Writer, Profile::Upright, -2, 2));
    assert_eq!(items[3], (3, DepthTest::Reader, Profile::Flat, -7, 1));
}

/// spec: S-0108 claim 23 (the door image in every door state of the unload, closed included;
/// absent without a door sprite), F8 and claim 24 (no `DoorStages`: D = 0, not inferred from the sprite:
/// frame 0 in every state)
#[test]
fn door_image_in_every_door_state() {
    for door in [
        DoorView::Closed,
        DoorView::Opening { m: 3, r: 1 },
        DoorView::Open,
        DoorView::Closing { m: 3, r: 2 },
    ] {
        let items = unloading_items(&UNLOAD_ROLES, unloading_with(false, door));
        let doors: Vec<_> = items.iter().filter(|i| i.0 == 3).collect();
        assert_eq!(
            doors,
            [&(3, DepthTest::Reader, Profile::Flat, -7, 1)],
            "{door:?}"
        );
    }
    let roles = ["deploying", "under_door"];
    let items = unloading_items(&roles, unloading_with(false, DoorView::Open));
    assert!(items.iter().all(|i| i.0 != 3));
}

/// spec: S-0108 claim 24 (`DoorStages` is read from the art section named by the image name, not
/// by the type name, in S-0003's partial integer form), claim 23 and F8 (open: D − 1)
#[test]
fn door_inputs_come_from_the_image_section() {
    let art = (
        "HUTXIMG",
        "\n[HUTXIMG]\nFoundation=2x2\nAnimIdle=1,1,0\nDoorStages=4abc\nDamagedDoor=Yes please\n\
         [HUTX]\nDoorStages=2\nDamagedDoor=no\n",
    );
    let open = unloading_with(false, DoorView::Open);
    let items = unloading_items_with(&UNLOAD_ROLES, open, Some(art), (256, 256));
    let doors: Vec<_> = items.iter().filter(|i| i.0 == 3).map(|i| i.4).collect();
    // The first pixel is the frame + 1: frame 4 − 1 = 3 (the type section's 2 would give 1).
    assert_eq!(doors, [4]);
}

/// spec: S-0108 claim 4 (a gate opening or closing draws frame p, damaged p + G + 1, from the
/// body sprite with no clamp, and no bib), claim 2 (a gate at rest: frame 0 and the bib), Import
/// mapping (GateStages absent: 9); S-0215 claim 35 d1 (p = trunc(q × G); at rest open while open is
/// current, G − 1; Q-0685 answered)
#[test]
fn moving_gate_draws_its_door_frame() {
    let ctx = || {
        let (import, catalogue, mut assets) = context_with(&format!("{RULES}[HUTX]\nGate=yes\n"));
        // 40 frames: 20 body, 20 shadow; the first pixel is the frame + 1.
        assets.insert(b"hutx.shp", shp((40, 30), 40, (4, 4), (6, 8)));
        (import, catalogue, assets)
    };
    let full = (256, 256);
    // At rest: frame 0 (pixel 1), shadow 20 (pixel 21), bib frame 0.
    let rest = live_building_items(ctx(), None, Some(DoorView::Closed), full);
    assert_eq!(rest.len(), 3, "{rest:?}");
    assert_eq!(rest[0].4, 1);
    assert_eq!(rest[1].4, 21);
    // Opening halfway through 90 ticks: p = trunc(4.5) = 4 (pixel 5), shadow 24, no bib.
    let half = Some(DoorView::Opening { m: 90, r: 45 });
    let moving = live_building_items(ctx(), None, half, full);
    assert_eq!(moving.len(), 2, "{moving:?}");
    assert_eq!(moving[0].4, 5);
    assert_eq!(moving[1].4, 25);
    // (The damaged frames, p + G + 1, are checked in unload::tests::moving_gate_frames: this
    // context has no simulation rules, so no building here is damaged.)
    // Closing at its start: G, held at G − 1 = 8 (pixel 9).
    let closing = Some(DoorView::Closing { m: 90, r: 90 });
    assert_eq!(live_building_items(ctx(), None, closing, full)[0].4, 9);
    // At rest open while open is current: G − 1 = 8 (pixel 9), no bib.
    let open = live_building_items(ctx(), None, Some(DoorView::Open), full);
    assert_eq!(open.len(), 2, "{open:?}");
    assert_eq!(open[0].4, 9);
}

/// spec: S-0417 Terms (unloading: the building's current behaviour is unload, a queued one is not),
/// claim 2 (the unit in contact is the one released); S-0108 claim 22, Import mapping (`JumpJet` of
/// the unloaded unit's type selects the roof variants); F8 (M and r from the door's movement)
#[test]
fn live_unloading_state_of_a_building() {
    use glasnost_sim::behaviour::Door;
    use glasnost_sim::rules::Behaviour;
    let (mut live, _) = synthetic_live();
    let k = own_mobile(&live, 0);
    let u = own_mobile(&live, 1);
    let tick = live.world.tick;
    {
        let o = live.world.obj_mut(k).unwrap();
        o.beh.current = None;
        o.beh.queued = Some(Behaviour::Unload);
        o.beh.gate.door = Door::Opening {
            since: tick - 2,
            ticks: 5,
        };
    }
    live.world.obj_mut(u).unwrap().contact = Some(k);
    let o = live.world.obj(k).unwrap().clone();
    // Queued only: not unloading.
    assert_eq!(live.unloading_of(&o), None);
    live.world.obj_mut(k).unwrap().beh.current = Some(Behaviour::Unload);
    let o = live.world.obj(k).unwrap().clone();
    // The picture shows the last tick that ran (S-0201 claim 6).
    let shown = (tick - 1).max(0);
    let r = (5 - (shown - (tick - 2))).clamp(0, 5);
    assert_eq!(
        live.unloading_of(&o),
        unloading_with(false, DoorView::Opening { m: 5, r })
    );
    // The unit in contact is marked jumpjet: the roof variants.
    let (kind, t) = {
        let uo = live.world.obj(u).unwrap();
        (uo.kind, uo.type_index as usize)
    };
    let rules = std::sync::Arc::make_mut(&mut live.rules);
    match kind {
        glasnost_sim::rules::Kind::Infantry => rules.infantry[t].jumpjet = true,
        _ => rules.vehicles[t].jumpjet = true,
    }
    assert!(live.unloading_of(&o).is_some_and(|x| x.roof));
}

/// The test context with BOOMX, an animation type of three frames at rate
/// 900 (stage delay 1), which is also the move-feedback animation.
fn anim_context() -> (
    glasnost_convert::Import,
    glasnost_convert::theater::Catalogue,
    super::build::Assets,
) {
    let rules = format!("{RULES}[Animations]\n0=BOOMX\n[General]\nMoveFlash=BOOMX\n");
    let art = format!("{}[BOOMX]\nRate=900\n", super::tests::ART);
    let mut ctx = super::tests::context_full(&rules, &art);
    ctx.0.arts.insert(
        (Kind::Animations, 0),
        super::tests::art_of("BOOMX", "boomx.shp"),
    );
    // Three frames (no shadow flag: every frame is a stage).
    ctx.2.insert(b"boomx.shp", shp((10, 10), 3, (2, 2), (4, 4)));
    ctx
}

/// The effects in the scene that are (`true`) or are not (`false`) our
/// provisional markers.
fn effect_items(scene: &Scene, provisional: bool) -> usize {
    use glasnost_render::scene::Group;
    scene
        .items
        .iter()
        .filter(|i| i.group == Group::Effects && i.provisional == provisional)
        .count()
}

/// spec: S-0109 claims 1, 2, 5, 6 and 9 (an animation event creates a
/// playing animation of its type that advances one frame per stage delay
/// from its first update and is removed at the end of its pass); S-0102
/// claim 3 (unattached, in its type's layer). The game client draws no
/// marker for it, nor for a type without a sprite.
#[test]
fn animation_events_play_their_art_and_end() {
    use glasnost_sim::combat::effects::Effect;
    use glasnost_sim::world::Pos;
    let (import, catalogue, assets) = anim_context();
    let map = testkit::map();
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (mut built, painter) = build_parts(&inputs, true);
    let world = testkit::new_game(5);
    let mut live = Live::new(
        &inputs,
        &mut built,
        painter,
        (world, testkit::rules(), testkit::setup(5), Vec::new()),
    );
    let mut scene = built.scene;
    live.set_debug_markers(false);
    live.step();
    let pos = Pos {
        x: 5 * 256 + 128,
        y: 5 * 256 + 128,
        z: 0,
    };
    live.world.effects.push(Effect::Anim {
        name: "boomx".into(),
        pos,
        delay: None,
        reverse: false,
    });
    live.world.effects.push(Effect::Anim {
        name: "NOSUCHANIM".into(),
        pos,
        delay: None,
        reverse: false,
    });
    live.after_tick();
    assert_eq!(live.counts.anims_with_art, 1);
    assert_eq!(live.counts.anims_without_sprite, 1);
    live.redraw(&inputs, &mut scene);
    assert_eq!(live.counts.anims_drawn, 1);
    assert!(
        effect_items(&scene, false) > 0,
        "the animation's art is drawn"
    );
    assert_eq!(
        effect_items(&scene, true),
        0,
        "no marker in the game client"
    );
    // Frames 0, 1 and 2 at one tick each, then the end of the pass.
    let mut shown = 1;
    for _ in 0..6 {
        live.step();
        live.redraw(&inputs, &mut scene);
        if live.counts.anims_drawn == 1 {
            shown += 1;
        }
    }
    assert_eq!(shown, 3, "one tick per frame, removed at its end");
    assert_eq!(effect_items(&scene, false), 0);

    // spec: S-0214 claim 30 (the move marker plays the move-feedback animation)
    live.move_marker(pos);
    assert_eq!(live.counts.anims_with_art, 2);
    live.redraw(&inputs, &mut scene);
    assert_eq!(live.counts.anims_drawn, 1);
    assert_eq!(effect_items(&scene, true), 0);
}

/// spec: S-0513 claims 33 and 34 (the source marker is shown to the
/// launching player only, a new pick replaces it, and the launch ends it)
#[test]
fn source_marker_shows_for_the_launching_player_until_the_launch() {
    use glasnost_sim::combat::effects::Effect;
    use glasnost_sim::superweapons::SwEvent;
    use glasnost_sim::world::{HouseId, Pos};
    let (import, catalogue, assets) = anim_context();
    let map = testkit::map();
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (mut built, painter) = build_parts(&inputs, true);
    let mut rules = (*testkit::rules()).clone();
    rules.superweapons.chrono_placement = Some("boomx".into());
    let mut live = Live::new(
        &inputs,
        &mut built,
        painter,
        (
            testkit::new_game(5),
            std::sync::Arc::new(rules),
            testkit::setup(5),
            Vec::new(),
        ),
    );
    let mut scene = built.scene;
    live.set_debug_markers(false);
    live.set_viewer(Some(HouseId(0)));
    live.step();
    let pos = Pos {
        x: 5 * 256 + 128,
        y: 5 * 256 + 128,
        z: 0,
    };
    let marker = |event, house| Effect::Superweapon {
        event,
        house: Some(HouseId(house)),
        sw: None,
        pos: Some(pos),
    };
    let base = live.counts.anims;
    live.world.effects.push(marker(SwEvent::SourceMarker, 1));
    live.after_tick();
    assert_eq!(
        live.counts.anims, base,
        "another house's pick shows nothing"
    );
    live.world.effects.push(marker(SwEvent::SourceMarker, 0));
    live.after_tick();
    assert_eq!(live.counts.anims, base + 1);
    live.redraw(&inputs, &mut scene);
    assert_eq!(live.counts.anims_drawn, 1);
    live.world.effects.push(marker(SwEvent::SourceMarker, 0));
    live.after_tick();
    assert_eq!(live.counts.anims, base + 2);
    live.redraw(&inputs, &mut scene);
    assert_eq!(live.counts.anims_drawn, 1, "a new pick replaces the marker");
    live.world.effects.push(marker(SwEvent::MarkerEnd, 0));
    live.after_tick();
    live.redraw(&inputs, &mut scene);
    assert_eq!(live.counts.anims_drawn, 0, "the launch ends it");
}

/// spec: S-0522 claim 13 and S-0109 claim 22 (a paratrooper's parachute is
/// attached to it, follows it in the ground layer, and disappears with it)
#[test]
fn parachute_follows_its_paratrooper() {
    use glasnost_sim::combat::effects::Effect;
    use glasnost_sim::superweapons::SwEvent;
    use glasnost_sim::world::{HouseId, Pos};
    let (import, catalogue, assets) = anim_context();
    let map = testkit::map();
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (mut built, painter) = build_parts(&inputs, true);
    let mut rules = (*testkit::rules()).clone();
    rules.superweapons.parachute = Some("boomx".into());
    let mut live = Live::new(
        &inputs,
        &mut built,
        painter,
        (
            testkit::new_game(5),
            std::sync::Arc::new(rules),
            testkit::setup(5),
            Vec::new(),
        ),
    );
    let mut scene = built.scene;
    live.set_debug_markers(false);
    live.step();
    let cell = glasnost_map::CellXY::new(5, 5);
    let trooper = glasnost_sim::testkit::spawn(
        &mut live.world,
        glasnost_sim::rules::Kind::Infantry,
        "WALKER",
        0,
        cell,
        0,
    );
    let start = Pos {
        x: 5 * 256 + 128,
        y: 5 * 256 + 128,
        z: 600,
    };
    {
        let o = live.world.obj_mut(trooper).unwrap();
        o.pos = start;
        o.fall.falling = true;
        o.fall.chute = true;
    }
    live.world.effects.push(Effect::Superweapon {
        event: SwEvent::Parachute {
            trooper,
            delay: None,
        },
        house: Some(HouseId(0)),
        sw: None,
        pos: Some(start),
    });
    live.after_tick();
    assert_eq!(live.attached(), vec![(trooper, start)]);
    // spec: S-0522 claim 18 (Q-0754: the parachute carries its carrier's house name, for the house-colour unit palette, and the cell light sampled at creation)
    let house = live.world.houses[0].name.as_bytes().to_vec();
    assert_eq!(live.carriers(), vec![(trooper, house)]);
    live.redraw(&inputs, &mut scene);
    assert_eq!(live.counts.anims_drawn, 1, "the parachute is drawn");
    let lower = Pos { z: 590, ..start };
    live.world.obj_mut(trooper).unwrap().pos = lower;
    live.after_tick();
    assert_eq!(
        live.attached(),
        vec![(trooper, lower)],
        "it follows the unit"
    );
    glasnost_sim::objects::remove(&mut live.world, trooper);
    live.after_tick();
    assert!(live.attached().is_empty(), "it disappears with its owner");
}

/// spec: S-0301 claim 22 and S-0109 claim 22 (a shot animation of a firer that is not a building
/// is attached at its displacement from the firer, in the ground layer, follows it and disappears
/// with it; a building's is not attached, keeps its type's layer and carries F8's depth adjustment)
#[test]
fn shot_animation_follows_its_firer() {
    use super::slots::Layer;
    use glasnost_sim::combat::effects::Effect;
    use glasnost_sim::world::Pos;
    let (import, catalogue, assets) = anim_context();
    let map = testkit::map();
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (mut built, painter) = build_parts(&inputs, true);
    let mut live = Live::new(
        &inputs,
        &mut built,
        painter,
        (
            testkit::new_game(5),
            testkit::rules(),
            testkit::setup(5),
            Vec::new(),
        ),
    );
    let mut scene = built.scene;
    live.set_debug_markers(false);
    live.step();
    let firer = glasnost_sim::testkit::spawn(
        &mut live.world,
        glasnost_sim::rules::Kind::Infantry,
        "WALKER",
        0,
        glasnost_map::CellXY::new(5, 5),
        0,
    );
    let at = live.world.obj(firer).unwrap().pos;
    let d = Pos::new(40, -30, 20);
    let launch = Pos::new(at.x + d.x, at.y + d.y, at.z + d.z);
    live.world.effects.push(Effect::ShotAnim {
        name: "boomx".into(),
        pos: launch,
        delay: None,
        attached: Some((firer, d)),
        z_adjust: None,
    });
    // A building's: not attached, with its depth adjustment.
    let fixed = Pos::new(at.x + 512, at.y, 0);
    live.world.effects.push(Effect::ShotAnim {
        name: "boomx".into(),
        pos: fixed,
        delay: None,
        attached: None,
        z_adjust: Some(-37),
    });
    // The same type made by an ordinary animation event, for its type's layer.
    live.world.effects.push(Effect::Anim {
        name: "boomx".into(),
        pos: fixed,
        delay: None,
        reverse: false,
    });
    live.after_tick();
    let shots = live.shot_anims();
    assert_eq!(shots.len(), 2);
    assert_eq!(shots[0], (Some(firer), launch, Layer::Ground, None));
    let type_layer = *live.anim_layers().last().expect("the ordinary animation");
    assert_eq!(shots[1], (None, fixed, type_layer, Some(-37)));
    live.redraw(&inputs, &mut scene);
    assert_eq!(live.counts.anims_drawn, 3, "drawn with their art");
    assert_eq!(effect_items(&scene, true), 0, "no placeholder");
    // The firer moves: the animation keeps its displacement.
    let moved = Pos::new(at.x + 100, at.y + 60, at.z);
    live.world.obj_mut(firer).unwrap().pos = moved;
    live.after_tick();
    let shots = live.shot_anims();
    assert_eq!(
        shots[0].1,
        Pos::new(moved.x + d.x, moved.y + d.y, moved.z + d.z)
    );
    assert_eq!(shots[1].1, fixed, "a building's stays where it was made");
    // The firer is removed: its animation goes with it.
    glasnost_sim::objects::remove(&mut live.world, firer);
    live.after_tick();
    let shots = live.shot_anims();
    assert_eq!(shots.len(), 1);
    assert_eq!(shots[0].0, None);
}

/// S-0105 claim 40 (Q-0799): a flying aircraft's body is drawn at its lifted point and its shadow at the
/// terrain surface under it, 60 pixels higher over a qualifying high bridge; on the ground both share a point.
#[test]
fn flying_aircraft_shadow_lies_on_the_ground() {
    use glasnost_convert::art::TypeArt;
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
    let (mut import, catalogue, mut assets) =
        context_with(&format!("{RULES}[AircraftTypes]\n1=JETX\n"));
    import.arts.insert(
        (Kind::Aircraft, 0),
        TypeArt {
            voxels: [("body".to_string(), b"jetx.vxl".to_vec())].into(),
            ..Default::default()
        },
    );
    assets.insert(
        b"jetx.vxl",
        VxlWriter {
            model: &synthetic_model(),
        }
        .write(),
    );
    let doc = Document::parse(&scene_map_text(), Quirks::default(), &mut Vec::new());
    let map = decode(
        &doc,
        &DecodeContext {
            catalogue: Some(&catalogue),
            ..Default::default()
        },
    )
    .map;
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (built, mut painter) = build_parts(&inputs, true);
    let mut scene: Scene = built.scene;
    let jet = |z| Mobile {
        kind: Kind::Aircraft,
        idx: 0,
        owner: b"Neutral".to_vec(),
        pos: (3 * 256 + 128, 3 * 256 + 128, z),
        body: 16384,
        turret: 16384,
        elevation: 0,
        anim: 0,
        infantry: None,
        object: sim_code(ObjectId(5)),
        harvesting: None,
        shadow: None,
        alternative: false,
        attitude: Attitude::None,
        disabled: false,
        above: 0,
        protection: None,
    };
    let mut offset = |z| {
        let (entries, _) = painter.mobiles(&inputs, &mut scene, &[jet(z)]);
        let items = &entries[0].items;
        let shadow = items.iter().find(|i| i.paint == Paint::Shadow);
        let body = items.iter().find(|i| i.paint != Paint::Shadow);
        if let (Some(s), Some(b)) = (shadow, body) {
            // spec: S-0102 claim 18 (the shadow is a depth reader with the body's own bias and the upright profile, drawn in the object's own drawing before its body, so an aircraft's body is never darkened by its own shadow)
            assert_eq!(s.test, DepthTest::Reader);
            assert_eq!(s.depth, b.depth, "the body's bias and profile");
            let at = |p: Paint| items.iter().position(|i| i.paint == p);
            assert!(at(Paint::Shadow) < items.iter().position(|i| i.paint != Paint::Shadow));
        }
        (shadow.map(|s| s.y), body.map(|b| b.y))
    };
    let (shadow, body) = offset(0);
    assert!(shadow.is_some() && body.is_some(), "both images drawn");
    // On the ground the two share a point, so the shadow's top is no lower than the body's
    // top plus the images' own size difference (checked at height 0 as the reference).
    let reference = shadow.unwrap() - body.unwrap();
    let (shadow, body) = offset(1500);
    // In the air the body is lifted and the shadow is not: the gap grows by the lift.
    assert_eq!(
        shadow.unwrap() - body.unwrap(),
        reference + glasnost_render::projection::height_lift(1500)
    );
    // spec: S-0105 claim 40 (a qualifying high bridge raises the anchor by exactly 60 pixels; the
    // terrain height, not the flight height, places it)
    let mut shadow_y = |anchor| {
        let m = Mobile {
            shadow: Some(anchor),
            alternative: false,
            attitude: Attitude::None,
            disabled: false,
            above: 0,
            protection: None,
            ..jet(1500)
        };
        let (entries, _) = painter.mobiles(&inputs, &mut scene, &[m]);
        entries[0]
            .items
            .iter()
            .find(|i| i.paint == Paint::Shadow)
            .map(|s| s.y)
            .unwrap()
    };
    let flat = shadow_y(ShadowAnchor {
        ground: 0,
        deck: false,
    });
    assert_eq!(
        shadow_y(ShadowAnchor {
            ground: 0,
            deck: true
        }),
        flat - 60
    );
    assert_eq!(
        shadow_y(ShadowAnchor {
            ground: 208,
            deck: false
        }),
        flat - glasnost_render::projection::height_lift(208)
    );
}

/// S-0105 claim 40: the bridge qualification of the aircraft shadow anchor (a high-bridge cell and
/// its neighbour towards y - 1 for an axis along cell x; a >= 416 unmarked, a >= 0 marked), and the
/// terrain anchor otherwise, whatever the flight height. The mark is S-0501 claim 43's aircraft
/// deck mark (`AirState::deck`), not the render layer or the occupant listing (Q-0841 answered).
#[test]
fn aircraft_shadow_anchor_bridge_qualification() {
    use glasnost_sim::air::AirState;
    use glasnost_sim::cells::Axis;
    use glasnost_sim::world::Layer;
    let mut w = testkit::new_game(5);
    let c = glasnost_map::CellXY::new(testkit::SIZE, testkit::SIZE);
    let n = glasnost_map::CellXY::new(c.x, c.y - 1);
    let i = w.cells.idx(c).unwrap();
    let j = w.cells.idx(n).unwrap();
    w.cells.deck[i] = true;
    w.cells.bridge_axis[i] = Axis::X;
    let id = testkit::spawn(&mut w, glasnost_sim::rules::Kind::Vehicle, "CART", 0, c, 0);
    let ground = w.surface_at(256 * c.x + 128, 256 * c.y + 128);
    let anchor = |w: &mut glasnost_sim::world::World, z: i32, marked: bool| {
        let o = w.obj_mut(id).unwrap();
        o.pos.z = ground + z;
        o.air = Some(AirState {
            deck: marked,
            ..Default::default()
        });
        let o = w.obj(id).unwrap().clone();
        super::live::shadow_anchor(w, &o)
    };
    // The neighbour is no high-bridge cell: terrain, at any height.
    assert!(!anchor(&mut w, 1500, false).deck);
    w.cells.deck[j] = true;
    assert!(
        !anchor(&mut w, 415, false).deck,
        "a = 415 is below the deck"
    );
    assert!(anchor(&mut w, 416, false).deck, "a = 416 qualifies");
    assert_eq!(anchor(&mut w, 416, false).ground, ground);
    // Marked on the deck, b = 416: a = z - 416 >= 0.
    assert!(!anchor(&mut w, 415, true).deck);
    assert!(anchor(&mut w, 416, true).deck);
    // The render layer and the listing are no substitute for the mark (S-0501 claim 43).
    {
        let o = w.obj_mut(id).unwrap();
        o.pos.z = ground + 415;
        o.air = Some(AirState::default());
        o.layer = Layer::Deck;
    }
    let o = w.obj(id).unwrap().clone();
    assert!(
        !super::live::shadow_anchor(&w, &o).deck,
        "a deck layer without the mark does not count"
    );
    // A destroyed deck (no mark) cannot qualify.
    w.cells.deck[i] = false;
    assert!(!anchor(&mut w, 1500, false).deck);
}

/// spec: S-0105 claim 12 (a vehicle type with the alternative-body option draws its alternative
/// body, not its main body, while no spawned aircraft is docked; a type without the option never
/// does, and a type without an alternative body file draws the main body; the shadow stays the
/// main body's, claim 29)
#[test]
fn alternative_body_replaces_the_main_body_without_a_docked_spawn() {
    use glasnost_fmt::bytes::F32Bits;
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
    let draw = |option: bool, alt_file: bool, alternative: bool| {
        let rules = if option {
            format!("{RULES}[CARX]\nNoSpawnAlt=yes\n")
        } else {
            RULES.to_string()
        };
        let (mut import, catalogue, mut assets) = context_with(&rules);
        let art = import.arts.get_mut(&(Kind::Vehicles, 0)).unwrap();
        art.voxels.insert("body".into(), b"main.vxl".to_vec());
        let mut main = synthetic_model();
        main.limbs[0].max[0] = F32Bits::new(30.0);
        assets.insert(b"main.vxl", VxlWriter { model: &main }.write());
        if alt_file {
            art.voxels
                .insert("alternate_body".into(), b"alt.vxl".to_vec());
            assets.insert(
                b"alt.vxl",
                VxlWriter {
                    model: &synthetic_model(),
                }
                .write(),
            );
        }
        let doc = Document::parse(&scene_map_text(), Quirks::default(), &mut Vec::new());
        let map = decode(
            &doc,
            &DecodeContext {
                catalogue: Some(&catalogue),
                ..Default::default()
            },
        )
        .map;
        let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
        let options = Options::default();
        let inputs = Inputs {
            map: &map,
            import: &import,
            catalogue: &catalogue,
            palettes: &palettes,
            assets: &assets,
            options: &options,
            rules: None,
            cells: None,
        };
        let (built, mut painter) = build_parts(&inputs, true);
        let mut scene = built.scene;
        let car = Mobile {
            kind: Kind::Vehicles,
            idx: 0,
            owner: b"Neutral".to_vec(),
            pos: (2 * 256 + 128, 3 * 256 + 128, 0),
            body: 8192,
            turret: 8192,
            elevation: 0,
            anim: 0,
            infantry: None,
            object: sim_code(ObjectId(9)),
            harvesting: None,
            shadow: None,
            alternative,
            attitude: Attitude::None,
            disabled: false,
            above: 0,
            protection: None,
        };
        let (entries, _) = painter.mobiles(&inputs, &mut scene, &[car]);
        let size = |shadow: bool| {
            let i = entries[0]
                .items
                .iter()
                .find(|i| (i.paint == Paint::Shadow) == shadow)
                .expect("an item");
            let img = &scene.images[i.image as usize];
            (img.width(), img.height())
        };
        (size(false), size(true))
    };
    let (main_body, main_shadow) = draw(true, true, false);
    let (alt_body, alt_shadow) = draw(true, true, true);
    assert_ne!(main_body, alt_body, "the alternative body is another model");
    assert_eq!(main_shadow, alt_shadow, "the shadow is the main body's");
    // No option: the main body whatever the docked state.
    assert_eq!(draw(false, true, true).0, main_body);
    // The option without an alternative body file: the main body.
    assert_eq!(draw(true, false, true).0, main_body);
}

/// S-0105 claims 41 and 42 (Q-0950): a rocket's body is drawn with its current nose pitch, an image is
/// reused only inside its pitch class and heading, a flying-locomotor aircraft's body and shadow take its
/// aiming facing and its pitch and bank follow height, speed and an unfinished turn. The shadow is never
/// pitched or banked.
#[test]
fn aircraft_attitude_is_drawn() {
    use glasnost_convert::art::TypeArt;
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
    let rules = format!(
        "{RULES}[AircraftTypes]\n1=JETX\n[JETX]\nPitchAngle=40\nRollAngle=-1\nPitchSpeed=0.5\n"
    );
    let (mut import, catalogue, mut assets) = context_with(&rules);
    import.arts.insert(
        (Kind::Aircraft, 0),
        TypeArt {
            voxels: [("body".to_string(), b"jetx.vxl".to_vec())].into(),
            ..Default::default()
        },
    );
    assets.insert(
        b"jetx.vxl",
        VxlWriter {
            model: &synthetic_model(),
        }
        .write(),
    );
    let doc = Document::parse(&scene_map_text(), Quirks::default(), &mut Vec::new());
    let map = decode(
        &doc,
        &DecodeContext {
            catalogue: Some(&catalogue),
            ..Default::default()
        },
    )
    .map;
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (built, mut painter) = build_parts(&inputs, true);
    let mut scene: Scene = built.scene;
    let craft = |body, turret, attitude| Mobile {
        kind: Kind::Aircraft,
        idx: 0,
        owner: b"Neutral".to_vec(),
        pos: (3 * 256 + 128, 3 * 256 + 128, 1500),
        body,
        turret,
        elevation: 0,
        anim: 0,
        infantry: None,
        object: sim_code(ObjectId(5)),
        harvesting: None,
        shadow: None,
        alternative: false,
        attitude,
        disabled: false,
        above: 0,
        protection: None,
    };
    // The body and shadow images drawn: their scene image numbers.
    let mut images = |m: Mobile| {
        let (entries, _) = painter.mobiles(&inputs, &mut scene, &[m]);
        let items = &entries[0].items;
        let id = |shadow: bool| {
            items
                .iter()
                .find(|i| (i.paint == Paint::Shadow) == shadow)
                .map(|i| i.image)
        };
        (id(false), id(true))
    };
    let rocket = |pitch: f32| Attitude::Rocket {
        pitch: pitch.to_bits(),
        initial: 0.25f32.to_bits(),
        final_: 0.75f32.to_bits(),
    };
    // Claim 41: a pitched body differs from the level one, whose image is the heading alone.
    let (level, level_shadow) = images(craft(16384, 16384, rocket(0.0)));
    let (plain, plain_shadow) = images(craft(16384, 16384, Attitude::None));
    assert!(level.is_some() && level_shadow.is_some());
    assert_eq!(level_shadow, plain_shadow);
    let _ = plain;
    let (up, up_shadow) = images(craft(16384, 16384, rocket(0.7)));
    assert_ne!(level, up, "the nose pitch is drawn");
    assert_eq!(level_shadow, up_shadow, "the shadow is not pitched");
    // Reuse: the same pitch class and heading reuse the image; another pitch does not; the
    // 32-step heading bucket decides the heading.
    assert_eq!(up, images(craft(16384, 16384, rocket(0.7))).0);
    assert_eq!(up, images(craft(16384 + 300, 16384, rocket(0.7))).0);
    assert_ne!(up, images(craft(16384, 16384, rocket(0.9))).0);
    assert_ne!(
        up,
        images(craft(20480, 16384, rocket(0.7))).0,
        "another heading"
    );
    // Claim 42: heading is the aiming facing (turret), not the body facing.
    let flying = |height, speed: f64, turn| Attitude::Flying {
        height,
        speed: speed.to_bits(),
        turn,
        crashing: false,
    };
    let (base, base_shadow) = images(craft(16384, 20480, flying(0, 1.0, 0)));
    assert_eq!(
        base,
        images(craft(24576, 20480, flying(0, 1.0, 0))).0,
        "the body facing plays no part"
    );
    assert_ne!(
        base,
        images(craft(16384, 24576, flying(0, 1.0, 0))).0,
        "the aiming facing is the heading"
    );
    assert_eq!(
        base_shadow,
        images(craft(24576, 20480, flying(0, 1.0, 0))).1
    );
    // Pitch needs strictly positive height and a speed fraction above PitchSpeed (0.5 here).
    let pitched = images(craft(16384, 20480, flying(10, 0.75, 0))).0;
    assert_ne!(pitched, base, "pitched in the air at speed");
    assert_eq!(
        images(craft(16384, 20480, flying(0, 0.75, 0))).0,
        base,
        "on the ground"
    );
    assert_eq!(
        images(craft(16384, 20480, flying(10, 0.5, 0))).0,
        base,
        "at the threshold"
    );
    // Bank: a turn banks (RollAngle -1 keeps the default of 30 degrees), the sign by direction,
    // and it also needs the speed condition.
    let cw = images(craft(16384, 20480, flying(10, 0.75, 1))).0;
    let ccw = images(craft(16384, 20480, flying(10, 0.75, -1))).0;
    assert_ne!(cw, pitched);
    assert_ne!(cw, ccw);
    assert_eq!(images(craft(16384, 20480, flying(10, 0.4, 1))).0, base);
    // The shadow stays the heading's whatever the tilt.
    assert_eq!(
        images(craft(16384, 20480, flying(10, 0.75, 1))).1,
        base_shadow
    );
    // A crashing body is drawn level.
    let crash = Attitude::Flying {
        height: 10,
        speed: 0.75f64.to_bits(),
        turn: 1,
        crashing: true,
    };
    assert_eq!(images(craft(16384, 20480, crash)).0, base);
}

/// spec: S-0104 claim 13 (Deactivated voxel vehicles: a voxel vehicle disabled under S-0405
/// claims 13 and 20 is drawn at trunc(b / 2) ‰, before claim 15's shade-row mapping)
#[test]
fn a_disabled_voxel_vehicle_is_drawn_at_half_brightness() {
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
    let draw = |disabled: bool| {
        let (mut import, catalogue, mut assets) = context_with(RULES);
        let art = import.arts.get_mut(&(Kind::Vehicles, 0)).unwrap();
        art.voxels.insert("body".into(), b"main.vxl".to_vec());
        assets.insert(
            b"main.vxl",
            VxlWriter {
                model: &synthetic_model(),
            }
            .write(),
        );
        let doc = Document::parse(&scene_map_text(), Quirks::default(), &mut Vec::new());
        let map = decode(
            &doc,
            &DecodeContext {
                catalogue: Some(&catalogue),
                ..Default::default()
            },
        )
        .map;
        let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
        let options = Options::default();
        let inputs = Inputs {
            map: &map,
            import: &import,
            catalogue: &catalogue,
            palettes: &palettes,
            assets: &assets,
            options: &options,
            rules: None,
            cells: None,
        };
        let (built, mut painter) = build_parts(&inputs, true);
        let mut scene = built.scene;
        let car = Mobile {
            kind: Kind::Vehicles,
            idx: 0,
            owner: b"Neutral".to_vec(),
            pos: (2 * 256 + 128, 3 * 256 + 128, 0),
            body: 8192,
            turret: 8192,
            elevation: 0,
            anim: 0,
            infantry: None,
            object: sim_code(ObjectId(9)),
            harvesting: None,
            shadow: None,
            alternative: false,
            attitude: Attitude::None,
            disabled,
            above: 0,
            protection: None,
        };
        let (entries, _) = painter.mobiles(&inputs, &mut scene, &[car]);
        entries[0]
            .items
            .iter()
            .find_map(|i| match i.paint {
                Paint::Lit(s) => Some(i32::from(s.rho)),
                _ => None,
            })
            .expect("a lit body item")
    };
    let (on, off) = (draw(false), draw(true));
    let rho = glasnost_render::light::brightness_rho;
    let candidates: Vec<i32> = (0..4000).filter(|&b| rho(b) == on).collect();
    assert!(!candidates.is_empty());
    assert!(
        candidates.iter().all(|&b| rho(b / 2) == off),
        "the disabled body takes ρ of trunc(b / 2)"
    );
    assert!(off < on);
}

// ---------------------------------------------------------------------------
// Beams and the game client's placeholders (visual review 2026-10-19)
// ---------------------------------------------------------------------------

/// The synthetic live game of [`synthetic_live`] in the game client's mode
/// (no debug markers), with the inputs' parts.
#[allow(clippy::type_complexity)]
fn client_live() -> (
    Live,
    Scene,
    glasnost_map::Map,
    glasnost_convert::Import,
    glasnost_convert::theater::Catalogue,
    super::build::Assets,
) {
    let (import, catalogue, assets) = context();
    let map = testkit::map();
    let (mut live, scene) = synthetic_live();
    live.set_debug_markers(false);
    (live, scene, map, import, catalogue, assets)
}

/// The beam items of a scene: effects painted with a plain row whose image
/// holds palette index 1.
fn beam_items(scene: &Scene) -> Vec<glasnost_render::scene::Item> {
    use glasnost_render::scene::Group;
    scene
        .items
        .iter()
        .filter(|i| i.group == Group::Effects && matches!(i.paint, Paint::Row(_)))
        .filter(|i| scene.images[i.image as usize].pixels().contains(&1))
        .copied()
        .collect()
}

/// spec: S-0518 claim 6 (a support beam lasts L_s ticks and is 3 pixels thick, in the house's laser
/// colour, Q-1521 interim: the house display RGB of S-0103 claim 15), S-0102 claim 2 (beam effects
/// after all layers)
#[test]
fn support_beam_is_a_three_pixel_line_for_its_life() {
    use glasnost_sim::combat::effects::Effect;
    use glasnost_sim::world::Pos;
    let (mut live, mut scene, map, import, catalogue, assets) = client_live();
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    live.step();
    // From (4, 6) to (6, 4) cells: the same screen row, 2 × 2 × 30 = 120 pixels apart (S-0101 F1).
    let from = Pos {
        x: 4 * 256,
        y: 6 * 256,
        z: 0,
    };
    let to = Pos {
        x: 6 * 256,
        y: 4 * 256,
        z: 0,
    };
    live.world.effects.push(Effect::SupportBeam {
        from,
        to,
        duration: 3,
        house: Some(HouseId(0)),
    });
    live.after_tick();
    live.redraw(&inputs, &mut scene);
    let beams = beam_items(&scene);
    assert_eq!(beams.len(), 1);
    let b = beams[0];
    let img = &scene.images[b.image as usize];
    let (w, h) = (img.width() as usize, img.height() as usize);
    // Every column inside the line holds exactly 3 pixels of the beam.
    let mid = w / 2;
    let column = (0..h).filter(|&y| img.pixels()[y * w + mid] == 1).count();
    assert_eq!(column, 3);
    // In the house display RGB, after every other drawing.
    let name = live.world.houses[0].name.clone();
    let rgb = super::build::owner_radar_colour(&import, &options, name.as_bytes(), [8, 8, 8]);
    let Paint::Row(r) = b.paint else {
        unreachable!()
    };
    assert_eq!(
        scene.rows[r][1],
        glasnost_render::compose::pack565(glasnost_fmt::image::Rgb8::new(rgb[0], rgb[1], rgb[2]))
    );
    assert_eq!(b.test, DepthTest::Ignore);
    // Shown for 3 ticks, then gone.
    for _ in 0..2 {
        live.step();
        live.redraw(&inputs, &mut scene);
        assert_eq!(beam_items(&scene).len(), 1);
    }
    live.step();
    live.redraw(&inputs, &mut scene);
    assert!(beam_items(&scene).is_empty());
}

/// spec: S-0615 claims 15 and 16 (the disc laser's beams: inner and outer colours; the final beam
/// fades from full intensity to none, an arc beam to half; Q-1521 interim: linear darkening)
#[test]
fn disc_laser_beam_fades_in_its_colours() {
    use super::combat::{Beam, Fade, faded};
    use glasnost_sim::combat::effects::Effect;
    use glasnost_sim::world::Pos;
    let (mut live, mut scene, map, import, catalogue, assets) = client_live();
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    live.step();
    live.world.effects.push(Effect::Beam {
        from: Pos {
            x: 4 * 256,
            y: 4 * 256,
            z: 0,
        },
        to: Pos {
            x: 7 * 256,
            y: 5 * 256,
            z: 0,
        },
        duration: 4,
        fade_out: true,
        colours: [[200, 100, 40], [100, 50, 20], [0, 0, 0]],
        house: None,
    });
    live.after_tick();
    live.redraw(&inputs, &mut scene);
    let row = |scene: &Scene| {
        let b = beam_items(scene)[0];
        let Paint::Row(r) = b.paint else {
            unreachable!()
        };
        (scene.rows[r][1], scene.rows[r][2], b)
    };
    let pack = |c: [u8; 3]| {
        glasnost_render::compose::pack565(glasnost_fmt::image::Rgb8::new(c[0], c[1], c[2]))
    };
    let (inner, outer, b) = row(&scene);
    assert_eq!((inner, outer), (pack([200, 100, 40]), pack([100, 50, 20])));
    // The outer colour is the edge around the inner line.
    assert!(scene.images[b.image as usize].pixels().contains(&2));
    // Two ticks into four: half intensity on the way to none.
    live.step();
    live.step();
    live.redraw(&inputs, &mut scene);
    let (inner, _, _) = row(&scene);
    assert_eq!(inner, pack([100, 50, 20]));
    // The intensity rule itself: half at the end of an arc beam, none at the end of the final beam.
    let beam = |fade| Beam {
        a: (0, 0),
        b: (10, 0),
        born: 0,
        life: 8,
        width: 1,
        inner: [200; 3],
        outer: None,
        fade,
    };
    assert_eq!(beam(Fade::ToHalf).intensity(8), 128);
    assert_eq!(beam(Fade::ToNone).intensity(8), 0);
    assert_eq!(beam(Fade::None).intensity(8), 256);
    assert_eq!(faded([200, 100, 40], 128), [100, 50, 20]);
}

/// S-0301 claim 23 (no default or placeholder picture at the muzzle), S-0303 claim 21 (a detonation shows its
/// explosion animation only), Q-1524 (no bomb, charge or strike-mark drawing): the game client draws
/// none of our own placeholders for shots and detonations; the viewer's debug layer does.
#[test]
fn game_client_draws_no_shot_or_burst_placeholders() {
    use glasnost_render::scene::Group;
    let (mut live, mut scene, map, import, catalogue, cart, walker) = combat_live(3);
    let assets = overlay_assets();
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    live.set_debug_markers(false);
    for _ in 0..400 {
        live.step();
        live.redraw(&inputs, &mut scene);
        // Only projectile markers (7 × 7) remain among the placeholders.
        assert!(
            scene
                .items
                .iter()
                .filter(|i| i.group == Group::Effects && i.provisional)
                .all(|i| scene.images[i.image as usize].width() == 7),
            "no muzzle or burst marker in the game client"
        );
        if live.world.obj(cart).is_none() || live.world.obj(walker).is_none() {
            break;
        }
    }
    assert!(live.counts.shots > 0 && live.counts.detonations > 0);
}

/// spec: S-0103 claim 8 row 3 and "Brightness of row 3" (a unit's deploy or undeploy animation is
/// drawn with its object: the owner's house-colour palette at brightness 1000); row 5 (any other
/// animation at the same place stays unowned)
#[test]
fn deploying_presentation_takes_the_owners_palette() {
    use glasnost_sim::combat::effects::Effect;
    let (mut live, _, _, _, _, cart, _) = combat_live(3);
    let o = live.world.obj(cart).unwrap();
    let (pos, ty) = (o.pos, o.type_index as usize);
    let owner = live.world.houses[usize::from(o.owner.unwrap().0)]
        .name
        .clone();
    let mut rules = (*live.rules).clone();
    rules.vehicles[ty].deploying_anim = Some("DEPLOYX".into());
    live.rules = std::sync::Arc::new(rules);
    for name in ["deployx", "OTHERX"] {
        live.world.effects.push(Effect::Anim {
            name: name.into(),
            pos,
            delay: None,
            reverse: false,
        });
    }
    live.after_tick();
    assert_eq!(
        live.anim_colours(),
        [Some((owner.into_bytes(), 1000)), None],
        "the deploy animation in its owner's colours at 1000, the other one unowned"
    );
}

/// spec: S-0102 claim 3 and S-0616 claim 30 (particles in the air layer:
/// after the ground layer's animation, before the top layer's, in the order
/// they were made); S-0616 claim 23 (a sprite kind with no sprite draws
/// nothing, a marker in the debug-markers view only; the lowest detail
/// hides smoke and spark), claim 24 (anim palette at 1000), claim 25 (gas
/// and smoke: the stage, the start frame unused; fire: s + g × E), claim 26
/// (the particle's own translucency value), claim 27 (upright reader, bias
/// −15 − L(z)), claim 28 (a spark is one opaque pixel of its colour, a flat
/// reader with bias −50 − L(z))
#[test]
fn particles_draw_their_sprites_in_the_air_layer() {
    use glasnost_num::F32;
    use glasnost_render::compose::Translucency;
    use glasnost_render::scene::Group;
    use glasnost_rules::sim::{ParticleSystemType, ParticleType};
    use glasnost_sim::combat::effects::Effect;
    use glasnost_sim::gas::{Particle, ParticleSystem};
    use glasnost_sim::world::Pos;
    let rules = format!(
        "{RULES}[Animations]\n0=GROUNDX\n1=TOPX\n\
         [Particles]\n0=PUFF\n1=SOOT\n2=BARE\n3=GLINT\n4=FLAME\n[PUFF]\nX=1\n[SOOT]\nStartFrame=2\n[BARE]\nX=1\n\
         [GLINT]\nX=1\n[FLAME]\nX=1\n"
    );
    let art = format!(
        "{}[GROUNDX]\nLayer=ground\nRate=900\n[TOPX]\nLayer=top\nRate=900\n",
        super::tests::ART
    );
    let (mut import, catalogue, mut assets) = super::tests::context_full(&rules, &art);
    let index = |import: &glasnost_convert::Import, kind: Kind, name: &str| {
        import.outcome.registries[&kind]
            .iter()
            .position(|t| t.name == name.as_bytes())
            .unwrap()
    };
    for (kind, name, file) in [
        (Kind::Animations, "GROUNDX", "groundx.shp"),
        (Kind::Animations, "TOPX", "topx.shp"),
        (Kind::Particles, "PUFF", "puff.shp"),
        (Kind::Particles, "SOOT", "soot.shp"),
        (Kind::Particles, "GLINT", "glint.shp"),
        (Kind::Particles, "FLAME", "flame.shp"),
    ] {
        let i = index(&import, kind, name);
        import
            .arts
            .insert((kind, i), super::tests::art_of(name, file));
    }
    // Frame sizes tell the drawings apart: 7, 9, 3, 5, 13 and 11 pixels
    // wide (a spark is 1); a frame's pixels hold its number plus 1.
    assets.insert(b"groundx.shp", shp((20, 20), 3, (7, 7), (2, 2)));
    assets.insert(b"topx.shp", shp((20, 20), 3, (9, 9), (2, 2)));
    assets.insert(b"puff.shp", shp((20, 20), 4, (3, 3), (2, 2)));
    assets.insert(b"soot.shp", shp((20, 20), 6, (5, 5), (2, 2)));
    assets.insert(b"glint.shp", shp((20, 20), 4, (13, 13), (2, 2)));
    assets.insert(b"flame.shp", shp((20, 20), 9, (11, 11), (2, 2)));
    let map = testkit::map();
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let low = Options {
        detail: glasnost_render::light::DetailLevel::Low,
        ..Options::default()
    };
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let mut r = testkit::ruleset();
    let kind = |k: &str, translucency: i32| ParticleType {
        behaves_like: Some(k.into()),
        translucency: Some(translucency),
        ..Default::default()
    };
    for (n, p) in [
        ("PUFF", kind("gas", 50)),
        ("SOOT", kind("smoke", 0)),
        ("BARE", kind("gas", 0)),
        (
            "GLINT",
            ParticleType {
                color_list: Some(vec![[1, 2, 3], [4, 5, 6]]),
                color_count: Some(2),
                ..kind("spark", 50)
            },
        ),
        (
            "FLAME",
            ParticleType {
                end_state_ai: Some(2),
                ..kind("fire", 0)
            },
        ),
    ] {
        r.registries
            .entry("particles".into())
            .or_default()
            .push(n.into());
        r.sim.particles.insert(n.into(), p);
    }
    r.registries
        .entry("particle_systems".into())
        .or_default()
        .push("CLOUD".into());
    r.sim.particle_systems.insert(
        "CLOUD".into(),
        ParticleSystemType {
            holds_what: Some("PUFF".into()),
            ..Default::default()
        },
    );
    let sim_rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic"));
    let world = glasnost_sim::load::new_game(
        sim_rules.clone(),
        &map,
        testkit::catalogue(),
        &testkit::setup(5),
    )
    .unwrap();
    let (mut built, painter) = build_parts(&inputs, true);
    let mut live = Live::new(
        &inputs,
        &mut built,
        painter,
        (world, sim_rules.clone(), testkit::setup(5), Vec::new()),
    );
    let mut scene = built.scene;
    live.set_debug_markers(false);
    live.step();
    let pos = Pos {
        x: 5 * 256 + 128,
        y: 5 * 256 + 128,
        z: 0,
    };
    for name in ["GROUNDX", "TOPX"] {
        live.world.effects.push(Effect::Anim {
            name: name.into(),
            pos,
            delay: None,
            reverse: false,
        });
    }
    live.after_tick();
    let particle = |type_index: u32, stage: i8, id: u32, translucency: u8| Particle {
        id: ObjectId(id),
        type_index,
        pos,
        vel: [0; 3],
        defl: [F32(0); 2],
        life: 100,
        countdown: 0,
        stage,
        dead: false,
        speed: F32(0),
        dir: [F32(0); 3],
        fvel: [F32(0); 3],
        step: [0; 3],
        advance: 0,
        translucency,
        colour: glasnost_sim::gas::ColourState {
            own: [200, 100, 50],
            ..Default::default()
        },
        origin: pos,
        aim: pos,
    };
    let cloud = sim_rules.gas.system("CLOUD").unwrap();
    let mut system = ParticleSystem::new(
        cloud,
        &sim_rules.gas.systems[cloud as usize],
        pos,
        None,
        sim_rules.mode,
    );
    // The flame flies along +x: down-right on the screen, F15 group 3.
    let flame = Particle {
        aim: Pos {
            x: pos.x + 1024,
            ..pos
        },
        ..particle(4, 1, 9005, 0)
    };
    system.particles = vec![
        particle(0, 1, 9001, 50),
        particle(1, 1, 9002, 0),
        particle(2, 0, 9003, 0),
        particle(3, 1, 9004, 50),
        flame,
    ];
    live.world.gas.systems.insert(ObjectId(9000), system);
    live.redraw(&inputs, &mut scene);
    assert_eq!(live.counts.particles_drawn, 4);
    let effects = |scene: &Scene| -> Vec<(
        usize,
        u32,
        u8,
        Paint,
        glasnost_render::scene::ItemDepth,
        glasnost_render::compose::DepthTest,
    )> {
        scene
            .items
            .iter()
            .enumerate()
            .filter(|(_, i)| i.group == Group::Effects && !i.provisional)
            .map(|(n, i)| {
                let img = &scene.images[i.image as usize];
                (n, img.width(), img.pixels()[0], i.paint, i.depth, i.test)
            })
            .collect()
    };
    let drawn = effects(&scene);
    let at = |w: u32| drawn.iter().find(|d| d.1 == w).cloned().unwrap();
    let (ground, top, puff, soot) = (at(7), at(9), at(3), at(5));
    let (spark, fire) = (at(1), at(11));
    assert!(ground.0 < puff.0 && puff.0 < soot.0 && soot.0 < spark.0);
    assert!(spark.0 < fire.0 && fire.0 < top.0);
    assert_eq!(
        drawn.len(),
        6,
        "the particle without a sprite draws nothing; the spark never uses its sprite"
    );
    // The gas and smoke particles at stage 1 show frame 1 (no start frame);
    // the flame frame 1 + 3 × 2.
    assert_eq!((puff.2, soot.2, fire.2), (2, 2, 8));
    assert!(
        matches!(puff.3, Paint::Translucent(s, Translucency::Half) if s.rho == glasnost_render::light::brightness_rho(1000) as u16)
    );
    assert!(matches!(soot.3, Paint::Lit(_)));
    use glasnost_render::compose::{DepthTest, Profile};
    use glasnost_render::scene::ItemDepth;
    assert_eq!(
        (puff.4, puff.5),
        (
            ItemDepth::Profile {
                profile: Profile::Upright,
                bias: -15,
                companion: None
            },
            DepthTest::Reader
        )
    );
    // The spark: one opaque pixel of its colour, with its translucency of 50 unused.
    let rgb = glasnost_fmt::image::Rgb8 {
        r: 200,
        g: 100,
        b: 50,
    };
    assert_eq!(spark.3, Paint::Pixel(rgb));
    assert_eq!(
        (spark.4, spark.5),
        (
            ItemDepth::Profile {
                profile: Profile::Flat,
                bias: -50,
                companion: None
            },
            DepthTest::Reader
        )
    );
    // At the lowest detail the smoke and the spark are not drawn.
    let low_inputs = Inputs {
        options: &low,
        ..inputs
    };
    live.redraw(&low_inputs, &mut scene);
    let widths: Vec<u32> = effects(&scene).iter().map(|d| d.1).collect();
    assert_eq!(widths.len(), 4, "{widths:?}");
    assert!(!widths.contains(&5) && !widths.contains(&1), "{widths:?}");
    live.redraw(&inputs, &mut scene);
    assert_eq!(
        effect_items(&scene, true),
        0,
        "no marker in the game client"
    );
    live.set_debug_markers(true);
    live.redraw(&inputs, &mut scene);
    assert_eq!(effect_items(&scene, true), 1, "the debug view marks it");
}

/// The rules of the protection tests: the curtain selector names table
/// position 1 and the force-shield selector position 2 (invented triples).
fn protection_rules() -> String {
    format!(
        "{RULES}[AudioVisual]\nIronCurtainColor=1\nForceShieldColor=2\n[ColorAdd]\nA=0,0,0\nB=7,19,3\nC=1,0,1\n"
    )
}

/// The painter inputs of a protection test, drawn by `f`.
fn with_protection_inputs<T>(
    voxel: bool,
    f: impl FnOnce(&Inputs, &mut super::build::Painter, &mut Scene) -> T,
) -> T {
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
    let (mut import, catalogue, mut assets) = context_with(&protection_rules());
    if voxel {
        let art = import.arts.get_mut(&(Kind::Vehicles, 0)).unwrap();
        art.voxels.insert("body".into(), b"car.vxl".to_vec());
        assets.insert(
            b"car.vxl",
            VxlWriter {
                model: &synthetic_model(),
            }
            .write(),
        );
    }
    let doc = Document::parse(&scene_map_text(), Quirks::default(), &mut Vec::new());
    let map = decode(
        &doc,
        &DecodeContext {
            catalogue: Some(&catalogue),
            ..Default::default()
        },
    )
    .map;
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (built, mut painter) = build_parts(&inputs, true);
    let mut scene = built.scene;
    f(&inputs, &mut painter, &mut scene)
}

/// The shades of the lit (non-shadow) items.
fn lit_shades(items: &[glasnost_render::scene::Item]) -> Vec<glasnost_render::compose::Shade> {
    items
        .iter()
        .filter_map(|i| match i.paint {
            Paint::Lit(s) => Some(s),
            _ => None,
        })
        .collect()
}

/// spec: S-0102 claim 34 and F9 (a curtain-protected vehicle drawn as voxels takes the curtain
/// selector's mask on its body; drawn as a sprite it takes none); S-0513 claim 29 and F9 (both take
/// the envelope's brightness before the shade-row selection: factor 512 doubles it)
#[test]
fn protection_curtain_vehicle_rows() {
    use super::protection::Look;
    use glasnost_render::compose::{PixelLayout, protection_mask};
    use glasnost_sim::superweapons::ProtectKind;
    let curtain = protection_mask([7, 19, 3], PixelLayout::Rgb565);
    for voxel in [true, false] {
        let (plain, shown) = with_protection_inputs(voxel, |inputs, painter, scene| {
            let car = |protection| Mobile {
                kind: Kind::Vehicles,
                idx: 0,
                owner: b"Neutral".to_vec(),
                pos: (2 * 256 + 128, 3 * 256 + 128, 0),
                body: 8192,
                turret: 8192,
                elevation: 0,
                anim: 0,
                infantry: None,
                object: sim_code(ObjectId(9)),
                harvesting: None,
                shadow: None,
                alternative: false,
                attitude: Attitude::None,
                disabled: false,
                above: 0,
                protection,
            };
            let look = Look {
                kind: ProtectKind::Curtain,
                factor: Some(512),
                shrouded: false,
            };
            let (plain, _) = painter.mobiles(inputs, scene, &[car(None)]);
            let (shown, _) = painter.mobiles(inputs, scene, &[car(Some(look))]);
            (lit_shades(&plain[0].items), lit_shades(&shown[0].items))
        });
        assert!(
            !plain.is_empty() && plain.len() == shown.len(),
            "voxel {voxel}"
        );
        for (p, s) in plain.iter().zip(&shown) {
            assert_eq!(p.mask, 0);
            assert_eq!(s.mask, if voxel { curtain } else { 0 }, "voxel {voxel}");
            assert!(s.rho > p.rho, "brighter: {} > {}", s.rho, p.rho);
        }
    }
}

/// spec: S-0102 claim 34 (a force-shielded building body drawn as a sprite takes the force-shield
/// selector's mask; a curtain-protected one takes none; both take the envelope's brightness, S-0513
/// F9; its shadow keeps its ordinary darkening)
#[test]
fn protection_building_body_rows() {
    use super::protection::Look;
    use glasnost_render::compose::{PixelLayout, protection_mask};
    use glasnost_sim::superweapons::ProtectKind;
    let shield = protection_mask([1, 0, 1], PixelLayout::Rgb565);
    let draw = |protection| {
        with_protection_inputs(false, |inputs, painter, scene| {
            let (drawn, _) = painter.buildings(
                inputs,
                scene,
                &[LiveBuilding {
                    idx: 0,
                    cell: glasnost_map::CellXY::new(2, 2),
                    owner: b"Neutral".to_vec(),
                    health: (1, 1),
                    facing: 0,
                    anim: 0,
                    occupied: false,
                    object: sim_code(ObjectId(5)),
                    unloading: None,
                    gate_door: None,
                    protection,
                    slots: Vec::new(),
                }],
            );
            let body = drawn[0]
                .iter()
                .find(|i| {
                    i.test == DepthTest::Writer
                        && matches!(
                            i.depth,
                            ItemDepth::Profile {
                                profile: Profile::Upright,
                                ..
                            }
                        )
                })
                .map(|i| i.paint);
            let shadows = drawn[0].iter().filter(|i| i.paint == Paint::Shadow).count();
            (body, shadows)
        })
    };
    let body = |look| match draw(look).0 {
        Some(Paint::Lit(s)) => s,
        p => panic!("no lit body: {p:?}"),
    };
    let look = |kind| {
        Some(Look {
            kind,
            factor: Some(512),
            shrouded: false,
        })
    };
    let plain = body(None);
    let shielded = body(look(ProtectKind::ForceShield));
    let curtained = body(look(ProtectKind::Curtain));
    assert_eq!(plain.mask, 0);
    assert_eq!(shielded.mask, shield);
    assert_eq!(curtained.mask, 0);
    assert!(shielded.rho > plain.rho && curtained.rho == shielded.rho);
    assert_eq!(
        draw(look(ProtectKind::ForceShield)).1,
        draw(None).1,
        "shadow unchanged"
    );
}

/// A look of `kind` at phase factor 512 (S-0513 claim 29: the bright hold).
fn bright(kind: glasnost_sim::superweapons::ProtectKind) -> super::protection::Look {
    super::protection::Look {
        kind,
        factor: Some(512),
        shrouded: false,
    }
}

/// The items the live painter draws for `building` (HUTX) in the
/// protection test context, with the unloading sprites `roles`, the
/// animation type SLOTA (art section `slot_art`), the rules `rules_extra`
/// added and the building look of each cell `cells`.
fn protected_hut_items(
    rules_extra: &str,
    roles: &[&'static str],
    slot_art: &str,
    building: LiveBuilding,
    cells: std::collections::BTreeMap<glasnost_map::CellXY, super::protection::Look>,
) -> Vec<glasnost_render::scene::Item> {
    use super::tests::{ART, art_of, context_full};
    let rules = format!("{}[Animations]\n0=SLOTA\n{rules_extra}", protection_rules());
    let art = format!("{ART}[SLOTA]\n{slot_art}");
    let (mut import, catalogue, mut assets) = context_full(&rules, &art);
    let hut = import.arts.get_mut(&(Kind::Buildings, 0)).unwrap();
    for &(role, file) in &[
        ("deploying", "hutxdep.shp"),
        ("roof_deploying", "hutxroof.shp"),
        ("under_door", "hutxud.shp"),
        ("under_roof_door", "hutxurd.shp"),
        ("door", "hutxdoor.shp"),
    ] {
        if roles.contains(&role) {
            hut.building_sprites.insert(role, file.as_bytes().to_vec());
            assets.insert(file.as_bytes(), shp((40, 30), 4, (4, 4), (6, 8)));
        }
    }
    if rules_extra.contains("Gate=yes") {
        assets.insert(b"hutx.shp", shp((40, 30), 40, (4, 4), (6, 8)));
    }
    import
        .arts
        .insert((Kind::Animations, 0), art_of("SLOTA", "slota.shp"));
    assets.insert(b"slota.shp", shp((20, 20), 4, (2, 2), (9, 9)));
    let doc = Document::parse(&scene_map_text(), Quirks::default(), &mut Vec::new());
    let map = decode(
        &doc,
        &DecodeContext {
            catalogue: Some(&catalogue),
            ..Default::default()
        },
    )
    .map;
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (built, mut painter) = build_parts(&inputs, true);
    let mut scene = built.scene;
    painter.set_protected_cells(cells);
    let (drawn, _) = painter.buildings(&inputs, &mut scene, &[building]);
    drawn.into_iter().next().unwrap_or_default()
}

/// HUTX at cell (2, 2) with `protection`, unloading `unloading`.
fn hut(protection: Option<super::protection::Look>, unloading: Option<Unloading>) -> LiveBuilding {
    LiveBuilding {
        idx: 0,
        cell: glasnost_map::CellXY::new(2, 2),
        owner: b"Neutral".to_vec(),
        health: (1, 1),
        facing: 0,
        anim: 0,
        occupied: false,
        slots: Vec::new(),
        object: sim_code(ObjectId(5)),
        unloading,
        gate_door: None,
        protection,
    }
}

/// The (profile, bias, shade) of each lit item.
fn lit_parts(
    items: &[glasnost_render::scene::Item],
) -> Vec<(Profile, i32, glasnost_render::compose::Shade)> {
    items
        .iter()
        .filter_map(|i| match (i.paint, i.depth) {
            (Paint::Lit(s), ItemDepth::Profile { profile, bias, .. }) => Some((profile, bias, s)),
            _ => None,
        })
        .collect()
}

/// HUTX's foundation cells (2 × 2 from cell (2, 2)), each under `look`.
fn hut_cells(
    look: Option<super::protection::Look>,
) -> std::collections::BTreeMap<glasnost_map::CellXY, super::protection::Look> {
    look.map(|l| {
        [(2, 2), (3, 2), (2, 3), (3, 3)]
            .map(|(x, y)| (glasnost_map::CellXY::new(x, y), l))
            .into_iter()
            .collect()
    })
    .unwrap_or_default()
}

/// A running SLOTA slot animation displaced `y` pixels down.
fn slot_a(y: i32) -> super::build::LiveSlot {
    super::build::LiveSlot {
        def: super::slots::SlotDef {
            normal: b"SLOTA".to_vec(),
            y,
            ..Default::default()
        },
        idx: 0,
        frame: 0,
        stage: 0,
        end: 0,
    }
}

/// spec: S-0513 claim 29b (the bib, the under-door and under-roof-door images, the deploying and
/// roof-deploying images drawn in place of the body and the door image while unloading take the
/// envelope and the force-shield mask under the force shield, none under the curtain: as the
/// body; the door's mask is the drawing correction), S-0102 claim 34 (the same rows), S-0108
/// claims 7, 22 and 23 (which images are drawn), S-0102 claim 13 (their biases)
#[test]
fn protection_claim29b_accessory_rows() {
    use glasnost_render::compose::{PixelLayout, protection_mask};
    use glasnost_sim::superweapons::ProtectKind;
    let shield = protection_mask([1, 0, 1], PixelLayout::Rgb565);
    for roof in [false, true] {
        let unloading = unloading_with(roof, DoorView::Open);
        let parts = |look| {
            lit_parts(&protected_hut_items(
                "",
                &UNLOAD_ROLES,
                "",
                hut(look, unloading),
                Default::default(),
            ))
        };
        let plain = parts(None);
        // Replacement body (−22 or −42, upright), bib (−3), under-door (−2, flat), door (−7).
        let body_bias = if roof { -42 } else { -22 };
        let biases: Vec<(Profile, i32)> = plain.iter().map(|p| (p.0, p.1)).collect();
        assert_eq!(
            biases,
            [
                (Profile::Upright, body_bias),
                (Profile::Flat, -3),
                (Profile::Flat, -2),
                (Profile::Flat, -7),
            ],
            "roof {roof}"
        );
        assert!(plain.iter().all(|p| p.2.mask == 0));
        for (kind, mask) in [
            (ProtectKind::ForceShield, shield),
            (ProtectKind::Curtain, 0),
        ] {
            let shown = parts(Some(bright(kind)));
            assert_eq!(shown.len(), plain.len());
            for (s, p) in shown.iter().zip(&plain) {
                assert_eq!(
                    s.2, shown[0].2,
                    "every part as the body: {kind:?} roof {roof}"
                );
                assert_eq!(s.2.mask, mask, "{kind:?} bias {}", s.1);
                assert!(s.2.rho > p.2.rho, "the envelope: {kind:?} bias {}", s.1);
            }
        }
    }
}

/// spec: S-0513 claim 29b (a special gate image drawn in place of the body: the body's envelope
/// and mask), S-0108 claim 4 (a moving gate draws its frame from the body sprite and no bib; at
/// rest the body and the bib)
#[test]
fn protection_claim29b_gate_image() {
    use glasnost_render::compose::{PixelLayout, protection_mask};
    use glasnost_sim::superweapons::ProtectKind;
    let shield = protection_mask([1, 0, 1], PixelLayout::Rgb565);
    for (door, count) in [
        (DoorView::Opening { m: 90, r: 45 }, 1),
        (DoorView::Closed, 2),
    ] {
        let mut b = hut(Some(bright(ProtectKind::ForceShield)), None);
        b.gate_door = Some(door);
        let shown = lit_parts(&protected_hut_items(
            "[HUTX]\nGate=yes\n",
            &[],
            "",
            b,
            Default::default(),
        ));
        assert_eq!(shown.len(), count, "{door:?}");
        assert!(shown.iter().all(|p| p.2.mask == shield), "{door:?}");
    }
}

/// spec: S-0513 claim 29b (an object whose centre cell is shrouded for the local player keeps the
/// envelope on every part still drawn and takes no mask), S-0102 claim 34
#[test]
fn protection_claim29b_shrouded_centre_cell() {
    use glasnost_sim::superweapons::ProtectKind;
    let unloading = unloading_with(false, DoorView::Open);
    let parts = |look| {
        lit_parts(&protected_hut_items(
            "",
            &UNLOAD_ROLES,
            "",
            hut(look, unloading),
            Default::default(),
        ))
    };
    let open = parts(Some(bright(ProtectKind::ForceShield)));
    let hidden = parts(Some(bright(ProtectKind::ForceShield).shrouded(true)));
    assert_eq!(open.len(), hidden.len());
    for (o, h) in open.iter().zip(&hidden) {
        assert_ne!(o.2.mask, 0);
        assert_eq!(h.2.mask, 0, "no mask under shroud, bias {}", h.1);
        assert_eq!(h.2.rho, o.2.rho, "the envelope stays, bias {}", h.1);
    }
    // A curtain-protected voxel vehicle likewise (S-0102 claim 34's curtain row).
    let m = super::protection::Masks {
        curtain: 0x1234,
        shield: 0x0841,
    };
    let look = Some(bright(ProtectKind::Curtain));
    assert_eq!(m.mobile(look, true, true), 0x1234);
    assert_eq!(m.mobile(look.map(|l| l.shrouded(true)), true, true), 0);
}

/// spec: S-0513 claim 29b (a centre cell: for a unit the cell that holds its position; shrouded
/// for the local player only), S-0409 Terms (explored)
#[test]
fn protection_centre_cell_follows_the_local_players_shroud() {
    use glasnost_sim::superweapons::{ProtectKind, Protection, Tint};
    let (mut live, _) = synthetic_live();
    let unit = own_mobile(&live, 0);
    let tick = live.world.tick;
    live.world.obj_mut(unit).unwrap().sw.protection = Some(Protection {
        start: tick - 1,
        duration: 1000,
        kind: ProtectKind::Curtain,
        tint: Tint::Rise(tick - 1),
    });
    let (cell, pos) = {
        let o = live.world.obj(unit).unwrap();
        (o.pos.cell(), o.pos)
    };
    let shrouded_for = |live: &mut Live, viewer: HouseId| {
        live.set_viewer(Some(viewer));
        live.protection_look_of(unit).expect("protected").shrouded
    };
    // Nothing is explored yet: shrouded for either house.
    assert!(!glasnost_sim::econ::shroud::explored(
        &live.world,
        HouseId(0),
        cell
    ));
    assert!(shrouded_for(&mut live, HouseId(0)));
    // House 1 explores the unit's cell: no longer shrouded for it, still for house 0.
    glasnost_sim::econ::shroud::area_reveal(&mut live.world, HouseId(1), pos, 2);
    assert!(glasnost_sim::econ::shroud::explored(
        &live.world,
        HouseId(1),
        cell
    ));
    assert!(!shrouded_for(&mut live, HouseId(1)));
    assert!(shrouded_for(&mut live, HouseId(0)));
    live.set_viewer(None);
    assert!(
        !live.protection_look_of(unit).unwrap().shrouded,
        "no local player"
    );
}

/// spec: S-0513 claim 29b (a translucent or rippling drawing of a protected object takes the
/// envelope on its source pixels before the F8 blend and no mask; opaque again, its mask again;
/// fully suppressed: nothing drawn), S-0516 claim 28 (25 %, 50 %, rippling drawn 50 %, hidden),
/// S-0102 F8
#[test]
fn protection_claim29b_translucent_drawings() {
    use super::live::cloak_items;
    use glasnost_render::compose::{Shade, Translucency};
    use glasnost_render::scene::{Group, Item};
    use glasnost_sim::cloak::view::Look;
    let shade = Shade::new(0, 4, 1400).with_mask(0x0841);
    let item = Item {
        image: 0,
        x: 0,
        y: 0,
        test: DepthTest::Writer,
        depth: ItemDepth::Profile {
            profile: Profile::Upright,
            bias: 0,
            companion: None,
        },
        paint: Paint::Lit(shade),
        group: Group::Buildings,
        provisional: false,
        object: None,
        stack: None,
    };
    for (look, t) in [
        (Look::Translucent25, Translucency::Quarter),
        (Look::Translucent50, Translucency::Half),
        (Look::Rippling, Translucency::Half),
    ] {
        let mut items = vec![item];
        cloak_items(look, false, &mut items);
        assert_eq!(
            items[0].paint,
            Paint::Translucent(shade.with_mask(0), t),
            "{look:?}: the envelope's ρ kept, no mask"
        );
    }
    let mut items = vec![item];
    cloak_items(Look::Normal, true, &mut items);
    assert_eq!(items[0].paint, Paint::Lit(shade), "opaque: the mask again");
    let mut items = vec![item];
    cloak_items(Look::Hidden, true, &mut items);
    assert!(items.is_empty(), "nothing is drawn");
}

/// spec: S-0102 claim 34 (a building slot animation drawn with its building's palette and stored
/// brightness takes the protected building's brightness; the animation rule: the building
/// occupying the animation's cell supplies the force-shield mask while force-shielded, curtain
/// protection none; the normal-light setting still forces 1000 ‰), S-0513 F9
#[test]
fn protection_slot_animation_rows() {
    use glasnost_render::compose::{PixelLayout, protection_mask};
    use glasnost_sim::superweapons::ProtectKind;
    let shield = protection_mask([1, 0, 1], PixelLayout::Rgb565);
    // The slot animation's shade: its item comes after the body's and the bib's.
    let anim = |look: Option<super::protection::Look>, art: &str, y: i32| {
        let mut b = hut(look, None);
        b.slots = vec![slot_a(y)];
        let items = protected_hut_items("", &[], art, b, hut_cells(look));
        lit_parts(&items)
            .last()
            .copied()
            .expect("the slot animation is drawn")
    };
    let plain = anim(None, "", 0);
    let shielded = anim(Some(bright(ProtectKind::ForceShield)), "", 0);
    let curtained = anim(Some(bright(ProtectKind::Curtain)), "", 0);
    assert_eq!(plain.2.mask, 0);
    assert_eq!(
        shielded.2.mask, shield,
        "on its force-shielded building's cell"
    );
    assert_eq!(curtained.2.mask, 0, "curtain: no mask");
    assert!(shielded.2.rho > plain.2.rho && curtained.2.rho == shielded.2.rho);
    // Displaced far off the foundation: no building occupies its cell, so no mask; the
    // brightness is still its own building's.
    let away = anim(Some(bright(ProtectKind::ForceShield)), "", 600);
    assert_eq!(away.2.mask, 0);
    assert_eq!(away.2.rho, shielded.2.rho);
    // The normal-light setting forces 1000 ‰: no envelope, the mask still by the cell rule.
    let normal = anim(
        Some(bright(ProtectKind::ForceShield)),
        "UseNormalLight=yes\n",
        0,
    );
    let normal_plain = anim(None, "UseNormalLight=yes\n", 0);
    assert_eq!(normal.2.rho, normal_plain.2.rho);
    assert_eq!(normal.2.mask, shield);
    // A shrouded occupying building supplies no mask (Q-1529 interim).
    let shrouded = anim(Some(bright(ProtectKind::ForceShield).shrouded(true)), "", 0);
    assert_eq!(shrouded.2.mask, 0);
}

/// S-0513 Open questions (a slot animation whose own art draws it translucent is open): our
/// interim draws it with the envelope and no mask; spec: S-0109 claim 20 (Translucency=50: code
/// 2), S-0102 F8
#[test]
fn protection_translucent_slot_animation_interim() {
    use glasnost_render::compose::Translucency;
    use glasnost_sim::superweapons::ProtectKind;
    let look = Some(bright(ProtectKind::ForceShield));
    let mut b = hut(look, None);
    b.slots = vec![slot_a(0)];
    let items = protected_hut_items("", &[], "Translucency=50\n", b, hut_cells(look));
    let blended: Vec<_> = items
        .iter()
        .filter_map(|i| match i.paint {
            Paint::Translucent(s, t) => Some((s, t)),
            _ => None,
        })
        .collect();
    assert_eq!(blended.len(), 1);
    assert_eq!(blended[0].1, Translucency::Half);
    assert_eq!(
        blended[0].0.mask, 0,
        "no mask on a translucent slot animation"
    );
    let body = lit_parts(&items)[0].2;
    assert_eq!(
        blended[0].0.rho, body.rho,
        "the protected building's brightness"
    );
}
