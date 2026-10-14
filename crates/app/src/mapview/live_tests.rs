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
