//! Tests of the live map scene: pacing (S-0201), the ground layer (S-0102
//! claim 6), the dynamic pass (S-0101, S-0102 claim 5, S-0107) and the flow
//! of selection and orders through the input rules (S-0213, S-0214), on
//! invented data and the simulation's synthetic scenario.

use glasnost_convert::kinds::Kind;
use glasnost_convert::map::{DecodeContext, decode};
use glasnost_ini::{Document, Quirks};
use glasnost_render::compose::Paint;
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::projection::project_lifted;
use glasnost_render::scene::Scene;
use glasnost_sim::targeting::{ClickOn, KeyCommand, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, Target};
use glasnost_sim::{save, testkit};

use super::build::{InfantryFrame, Inputs, Mobile, Options, build_parts, infantry_frame};
use super::live::{Entrant, GroundLayer, Live, Pacer, period_ms, sim_code};
use super::tests::{context, scene_map_text};

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

/// spec: S-0102 claim 6 (enter before the first strictly greater key; equal
/// keys keep entry order; on an unsorted list a newcomer can land early)
#[test]
fn ground_layer_entry() {
    let mut g = GroundLayer::default();
    let o = |n| Entrant::Object(ObjectId(n));
    g.enter(o(1), 5);
    g.enter(o(2), 10);
    g.enter(o(3), 3);
    // 5, 10 … 3 goes before 5.
    g.enter(o(4), 5);
    assert_eq!(g.order().collect::<Vec<_>>(), [o(3), o(1), o(4), o(2)]);
    // The claim's example: keys 5, 10, 3 unsorted, a newcomer with key 4 goes first.
    let mut h = GroundLayer::default();
    h.enter(o(1), 5);
    h.enter(o(2), 10);
    h.list_for_test_push(o(3), 3);
    h.enter(o(4), 4);
    assert_eq!(h.order().next(), Some(o(4)));
}

/// spec: S-0102 claim 6 (one pass of adjacent swaps per tick: a grown key
/// can travel far back, a shrunk key moves at most one place forward)
#[test]
fn ground_layer_pass() {
    let o = |n| Entrant::Object(ObjectId(n));
    let mut g = GroundLayer::default();
    for (n, k) in [(1, 10), (2, 20), (3, 30), (4, 40)] {
        g.enter(o(n), k);
    }
    // Object 1 moves far back in one pass.
    g.set_key(o(1), 45);
    g.pass();
    assert_eq!(g.order().collect::<Vec<_>>(), [o(2), o(3), o(4), o(1)]);
    // Object 1 drops to the front: one place per pass.
    g.set_key(o(1), 0);
    g.pass();
    assert_eq!(g.order().collect::<Vec<_>>(), [o(2), o(3), o(1), o(4)]);
    g.pass();
    assert_eq!(g.order().collect::<Vec<_>>(), [o(2), o(1), o(3), o(4)]);
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
/// position), S-0102 claim 5 (sort key X + Y), S-0107 claim 2 (frame)
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
        infantry: Some(InfantryFrame {
            start: 3,
            count: 1,
            multiplier: 2,
            stage: 0,
        }),
        object,
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
