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
use super::live::{Entrant, GroundLayer, Live, Pacer, period_ms, sim_code, wave_slots};
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
        generic_shadow: None,
        berserk: None,
        turret_index: None,
        walk: Default::default(),
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
    // dropping paratrooper: none from the body sprite; the generic shadow comes only with a
    // shadow height, none here)
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
        generic_shadow: None,
        berserk: None,
        turret_index: None,
        walk: Default::default(),
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

/// spec: S-0102 claims 3, 4 and 6 and scenario D49 (c), our engine's test
/// of claim 6's order, not a measurement of the original (Q-2346 answered):
/// no step of a superweapon warp re-files its member, so J1, created before
/// G4 and G5 and with one sort key with both, keeps its place before both
/// at every step of the warp, its time off the map included, through its
/// arrival hold and 300 ticks on; both positions of
/// quirks.layer_change_on_events_only. The warp is the simulation's own
/// (S-0513 claims 34, 36 and 39: a vehicle lands at its landing cell's
/// centre).
#[test]
fn d49c_a_warp_member_keeps_its_place_before_equal_key_objects() {
    use std::collections::BTreeSet;

    use glasnost_map::CellXY;
    use glasnost_sim::rules::Kind as SimKind;
    use glasnost_sim::superweapons::{self, warp, warp::Stage};
    use glasnost_sim::world::Layer as CellLayer;

    use super::live::mobile_key;

    for events_only in [true, false] {
        let (mut live, _) = synthetic_live();
        live.set_view_quirks(super::quirks::ViewQuirks {
            layer_change_on_events_only: events_only,
            ..Default::default()
        });
        // The source block's centre and the destination block's centre on
        // one screen row (equal x + y), 6 cells apart along it, away from the
        // scenario's units.
        let (source, dest) = (CellXY::new(46, 54), CellXY::new(52, 48));
        let along = |c: CellXY, k: i32| CellXY::new(c.x + k, c.y - k);
        let mut cells: Vec<CellXY> = [source, dest]
            .into_iter()
            .flat_map(|c| {
                superweapons::block(&live.world, c)
                    .into_iter()
                    .map(|(_, b)| b)
            })
            .collect();
        cells.extend([along(source, 2), along(dest, 2)]);
        assert_eq!(cells.len(), 20, "every cell is on the map");
        for &c in &cells {
            assert!(
                live.world.cells.list(c, CellLayer::Ground).is_empty(),
                "{c:?} is clear"
            );
        }
        // J1 on an edge cell of the source block, G4 the next cell outside
        // it along the row, G5 the next cell outside the destination block
        // along the row from J1's landing cell. J1 is created first.
        let j1 = testkit::spawn(
            &mut live.world,
            SimKind::Vehicle,
            "CART",
            1,
            along(source, 1),
            0,
        );
        let g4 = testkit::spawn(
            &mut live.world,
            SimKind::Vehicle,
            "CART",
            1,
            along(source, 2),
            0,
        );
        let g5 = testkit::spawn(
            &mut live.world,
            SimKind::Vehicle,
            "CART",
            1,
            along(dest, 2),
            0,
        );
        live.after_tick();
        let key = |live: &Live, id: ObjectId| mobile_key(live.world.obj(id).unwrap());
        assert_eq!(key(&live, j1), key(&live, g4));
        assert_eq!(key(&live, j1), key(&live, g5));
        let place = |live: &Live, id: ObjectId| live.ground().objects().position(|x| x == id);
        let check = |live: &Live, what: &str| {
            let (j, a, b) = (place(live, j1), place(live, g4), place(live, g5));
            assert!(j.is_some(), "J1 stays in the ground list ({what})");
            assert!(
                j < a && j < b,
                "J1 before G4 and G5 ({what}, events only {events_only})"
            );
        };
        check(&live, "before the launch");
        warp::launch(&mut live.world, HouseId(1), source, dest);
        let mut seen = BTreeSet::new();
        let mut away_listed = false;
        let mut over = None;
        for t in 0..3000 {
            live.step();
            let o = live.world.obj(j1).expect("J1 lives");
            let stage = o.sw.warp.as_ref().map(|m| m.stage);
            seen.insert(match stage {
                Some(Stage::Pending) => "pending",
                Some(Stage::Leaving(_) | Stage::Expired) => "leaving",
                Some(Stage::Blocked) => "blocked",
                Some(Stage::Away) => "away",
                Some(Stage::Placed) => "placed",
                Some(Stage::Hold(_)) => "hold",
                None => "over",
            });
            away_listed |= !o.on_map && place(&live, j1).is_some();
            check(&live, &format!("tick {t}, {stage:?}"));
            if stage.is_none() {
                let end = *over.get_or_insert(t);
                if t >= end + 300 {
                    break;
                }
            }
        }
        assert!(over.is_some(), "the hold ended");
        for s in ["leaving", "away", "placed", "hold", "over"] {
            assert!(seen.contains(s), "the warp reached {s}: {seen:?}");
        }
        assert!(away_listed, "kept in the ground list while off the map");
        assert_eq!(live.world.obj(j1).unwrap().pos.cell(), along(dest, 1));
        assert_eq!(
            key(&live, j1),
            key(&live, g5),
            "one key at the landing point"
        );
        assert_eq!(live.layer_of(j1), Some(super::slots::Layer::Ground));
    }
}

/// spec: S-0102 claim 4 (the simulation's moments reach the drawing: a
/// re-file reported in the tick's effects moves the object into its
/// evaluated layer under the default switch)
#[test]
fn layer_moments_from_the_simulation_are_applied() {
    use super::slots::Layer;
    use glasnost_sim::combat::effects::Effect;
    let (mut live, _) = synthetic_live();
    let a = own_mobile(&live, 0);
    live.world.obj_mut(a).unwrap().rocket = Some(Default::default());
    live.after_tick();
    assert_eq!(live.layer_of(a), Some(Layer::Ground), "no moment yet");
    live.world.effects.push(Effect::LayerMoment {
        object: a,
        check: None,
    });
    live.after_tick();
    assert_eq!(live.layer_of(a), Some(Layer::Air));
    assert_eq!(live.layer_list(Layer::Air), [Entrant::Object(a)]);
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

/// The view counts a vehicle's walk counter from the game's ticks: it
/// grows in each tick the vehicle is moving now by its locomotor (walk rate
/// absent: 1, so every tick number is a multiple) and holds while it is not
/// (idle rate absent: 0); the walking row follows movement in progress. A
/// vehicle off the map keeps its value.
///
/// spec: S-0102 claim 22a (the one walk counter; moving now; the hold after stopping; No advance off the map) and
/// Import mapping (WalkRate absent 1, IdleRate absent 0)
#[test]
fn a_vehicles_walk_counter_runs_while_it_moves() {
    let (mut live, _) = synthetic_live();
    let house = HouseId(0);
    let car = testkit::house_objects(&live.world, 0)
        .into_iter()
        .find(|&id| {
            live.world
                .obj(id)
                .is_some_and(|o| o.kind == glasnost_sim::rules::Kind::Vehicle)
        })
        .expect("the scenario gives the house a vehicle");
    while live.world.tick < 4 {
        live.step();
    }
    assert_eq!(live.walk(car).counter, 0);
    let mut selection = vec![car];
    let start = live.world.obj(car).unwrap().pos.cell();
    let to = glasnost_map::CellXY::new(start.x + 4, start.y);
    let r = live.click(
        house,
        &mut selection,
        ClickOn::Cell(to),
        Modifiers::default(),
    );
    assert_eq!(r.issued, 1);
    let mut moving_ticks = 0;
    let mut seen_in_progress = false;
    for _ in 0..200 {
        live.step();
        if glasnost_sim::occupancy::locomotor_moving_now(&live.world, car) {
            moving_ticks += 1;
        }
        seen_in_progress |= live.walk(car).in_progress;
        assert_eq!(live.walk(car).counter, moving_ticks);
    }
    assert!(moving_ticks > 0 && seen_in_progress, "the vehicle moved");
    assert!(
        !live.walk(car).in_progress,
        "the vehicle arrived and stands"
    );
    // Off the map (inside a transport): the counter keeps its value.
    live.world.obj_mut(car).unwrap().on_map = false;
    assert!(super::walk::held(&live.world, car));
    live.world.obj_mut(car).unwrap().on_map = true;
    assert!(!super::walk::held(&live.world, car));
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
/// a pip sprite of 21 frames on a canvas of 8 × 6 (frame i a 2 × 2 block of
/// value i + 1 at (3, 2)), a second pip sprite of 21 frames on the same
/// canvas (frame i a 4 × 1 strip of value i + 1 at (2, 2)), a bracket
/// sprite of 2 frames on 36 × 10, and an overlay palette.
fn overlay_assets() -> super::build::Assets {
    let (_, _, mut assets) = context();
    assets.insert(
        super::combat::PIP_SPRITE,
        super::tests::shp((8, 6), 21, (2, 2), (3, 2)),
    );
    assets.insert(
        super::combat::PIP2_SPRITE,
        super::tests::shp((8, 6), 21, (4, 1), (2, 2)),
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
    combat_live_with(gap, overlay_assets())
}

/// [`combat_live`] built with the assets `assets`.
#[allow(clippy::type_complexity)]
fn combat_live_with(
    gap: i32,
    assets: super::build::Assets,
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
            adjust: Default::default(),
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
        z_adjust: None,
        id: None,
    });
    live.world.effects.push(Effect::Anim {
        name: "NOSUCHANIM".into(),
        pos,
        delay: None,
        reverse: false,
        z_adjust: None,
        id: None,
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
            anim: None,
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
        id: None,
    });
    // A building's: not attached, with its depth adjustment.
    let fixed = Pos::new(at.x + 512, at.y, 0);
    live.world.effects.push(Effect::ShotAnim {
        name: "boomx".into(),
        pos: fixed,
        delay: None,
        attached: None,
        z_adjust: Some(-37),
        id: None,
    });
    // The same type made by an ordinary animation event, for its type's layer.
    live.world.effects.push(Effect::Anim {
        name: "boomx".into(),
        pos: fixed,
        delay: None,
        reverse: false,
        z_adjust: None,
        id: None,
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
    // spec: S-0506 claim 19 and S-0102 claim 13 (a rocket's detonation animation carries its depth adjustment, -15, in place of its type's)
    live.world.effects.push(Effect::Anim {
        name: "boomx".into(),
        pos: fixed,
        delay: None,
        reverse: false,
        z_adjust: Some(-15),
        id: None,
    });
    live.after_tick();
    let shots = live.shot_anims();
    assert_eq!(shots.len(), 2);
    assert_eq!(shots[1], (None, fixed, type_layer, Some(-15)));
}

/// spec: S-0105 Import mapping (`ConsideredAircraft` absent: yes for an aircraft type, no for
/// every other type) and claim 31 (the half-size shadow of an aircraft-like type): an aircraft
/// type that leaves the key out draws the shadow it draws with the key on, and a vehicle type
/// the one it draws with the key off.
#[test]
fn considered_aircraft_defaults_by_kind() {
    use glasnost_convert::art::TypeArt;
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
    // The shadow image's size of a type of `kind` whose section adds `extra`.
    let shadow = |kind: Kind, extra: &str| {
        let rules = match kind {
            Kind::Aircraft => format!("{RULES}[AircraftTypes]\n1=JETX\n[JETX]\n{extra}"),
            _ => format!("{RULES}[CARX]\n{extra}"),
        };
        let (mut import, catalogue, mut assets) = context_with(&rules);
        import.arts.insert(
            (kind, 0),
            TypeArt {
                voxels: [("body".to_string(), b"hull.vxl".to_vec())].into(),
                ..Default::default()
            },
        );
        assets.insert(
            b"hull.vxl",
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
        let m = Mobile {
            kind,
            idx: 0,
            owner: b"Neutral".to_vec(),
            pos: (3 * 256 + 128, 3 * 256 + 128, 0),
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
            generic_shadow: None,
            berserk: None,
            turret_index: None,
            walk: Default::default(),
        };
        let (entries, _) = painter.mobiles(&inputs, &mut scene, &[m]);
        let s = entries[0]
            .items
            .iter()
            .find(|i| i.paint == Paint::Shadow)
            .expect("a shadow");
        let img = &scene.images[s.image as usize];
        (img.width(), img.height())
    };
    let half = shadow(Kind::Aircraft, "ConsideredAircraft=yes\n");
    let full = shadow(Kind::Aircraft, "ConsideredAircraft=no\n");
    assert_ne!(half, full, "the key changes the shadow");
    assert_eq!(
        shadow(Kind::Aircraft, ""),
        half,
        "absent: yes for an aircraft type"
    );
    assert_eq!(
        shadow(Kind::Vehicles, ""),
        shadow(Kind::Vehicles, "ConsideredAircraft=no\n")
    );
    assert_ne!(
        shadow(Kind::Vehicles, ""),
        shadow(Kind::Vehicles, "ConsideredAircraft=yes\n"),
        "absent: no for a vehicle type"
    );
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
        generic_shadow: None,
        berserk: None,
        turret_index: None,
        walk: Default::default(),
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
            generic_shadow: None,
            berserk: None,
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
            generic_shadow: None,
            berserk: None,
            turret_index: None,
            walk: Default::default(),
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

/// spec: S-0105 claim 12 (a vehicle of a multi-turret type draws the turret and barrel models of
/// its selected turret index instead of the ordinary ones; with the index none or outside
/// 0 … N − 1 neither a turret nor a barrel; a missing barrel k alone leaves turret k drawn; a type
/// with rotating-barrel weapon stages draws its ordinary turret whatever its count), S-0007 claim
/// 16 (the numbered parts), claim 26 (the index is a turret key component) and claim 16 (a
/// numbered turret is drawn in frame f_b, never from the animation-frame counter, and its key
/// records that frame). The index is supplied by the test; claim 43's rules are
/// tested in `turret_index_tests`.
#[test]
fn multi_turret_draws_the_models_of_the_selected_index() {
    use glasnost_fmt::bytes::F32Bits;
    use glasnost_fmt::hva::synthetic_animation;
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
    // The lit item sizes (body first) and the cached turret keys (index, frame) of one vehicle
    // whose animation-frame counter is `anim`; every turret model has a 4-frame HVA.
    let draw = |extra: &str, index: Option<i32>, anim: u32| {
        let rules = format!("{RULES}[CARX]\nTurret=yes\n{extra}");
        let (mut import, catalogue, mut assets) = context_with(&rules);
        let art = import.arts.get_mut(&(Kind::Vehicles, 0)).unwrap();
        for (part, file, width) in [
            ("body", "body.vxl", 30.0),
            ("turret", "tur.vxl", 12.0),
            ("turret0", "tur0.vxl", 18.0),
            ("turret1", "tur1.vxl", 24.0),
        ] {
            art.voxels.insert(part.into(), file.as_bytes().to_vec());
            let mut model = synthetic_model();
            model.limbs[0].max[0] = F32Bits::new(width);
            assets.insert(file.as_bytes(), VxlWriter { model: &model }.write());
            if part != "body" {
                let hva = file.replace(".vxl", ".hva");
                assets.insert(hva.as_bytes(), synthetic_animation().write());
            }
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
            anim,
            infantry: None,
            object: sim_code(ObjectId(9)),
            harvesting: None,
            shadow: None,
            alternative: false,
            attitude: Attitude::None,
            disabled: false,
            above: 0,
            protection: None,
            generic_shadow: None,
            berserk: None,
            turret_index: index,
            walk: Default::default(),
        };
        let (entries, _) = painter.mobiles(&inputs, &mut scene, &[car]);
        let sizes: Vec<(u32, u32)> = entries[0]
            .items
            .iter()
            .filter(|i| i.paint != Paint::Shadow)
            .map(|i| {
                let img = &scene.images[i.image as usize];
                (img.width(), img.height())
            })
            .collect();
        // The turret keys cached, by the index and frame components (no displacement).
        let keys: Vec<(Option<u8>, u32)> = [None, Some(0), Some(1), Some(5)]
            .into_iter()
            .flat_map(|i| [(i, 0), (i, 3)])
            .filter(|&(i, frame)| {
                let key = glasnost_render::voxel::vehicle_turret_key(
                    glasnost_render::voxel::RAMP_FLAT,
                    8192,
                    8192,
                    0,
                    i,
                    false,
                    frame,
                );
                painter.voxel_cached(
                    Kind::Vehicles,
                    0,
                    &glasnost_render::voxel::ImageKey::Turret(key),
                )
            })
            .collect();
        (sizes, keys)
    };
    // A single-turret type: the ordinary turret, no index in the key.
    let (plain, plain_keys) = draw("", None, 0);
    assert_eq!(plain.len(), 2, "body and turret");
    assert_eq!(plain_keys, [(None, 0)]);
    // An index given to a single-turret type changes nothing.
    assert_eq!(draw("", Some(1), 0), (plain.clone(), plain_keys.clone()));
    // spec: S-0105 claim 16 (body frame 0: the ordinary turret's frame is u mod F_tur)
    assert_eq!(draw("", None, 3).1, [(None, 3)]);
    // A multi-turret type with the index none: no turret at all.
    let multi = "TurretCount=2\n";
    let (none, none_keys) = draw(multi, None, 0);
    assert_eq!(none, plain[..1], "the body alone");
    assert!(none_keys.is_empty());
    // Index 0 and index 1: the numbered models, the index in the key; turret k drawn without
    // barrel k (no numbered barrel is supplied).
    let (zero, zero_keys) = draw(multi, Some(0), 0);
    let (one, one_keys) = draw(multi, Some(1), 0);
    assert_eq!(zero.len(), 2);
    assert_eq!(one.len(), 2);
    assert_ne!(zero[1], plain[1], "turret 0 is not the ordinary turret");
    assert_ne!(zero[1], one[1], "turret 1 is another model");
    assert_eq!(zero_keys, [(Some(0), 0)]);
    assert_eq!(one_keys, [(Some(1), 0)]);
    // spec: S-0105 claim 16 (a numbered turret: frame f_b = 0 whatever the animation-frame counter)
    assert_eq!(draw(multi, Some(1), 3), (one.clone(), one_keys));
    // spec: S-0105 claim 12 (an index outside 0 … N − 1 draws neither part: N, 17, below −1)
    for k in [2, 17, -1, -5] {
        let (out, out_keys) = draw(multi, Some(k), 0);
        assert_eq!(out, plain[..1], "index {k}: the body alone");
        assert!(out_keys.is_empty(), "index {k}");
    }
    // Rotating-barrel weapon stages: the ordinary turret whatever the count and index.
    let gattling = "TurretCount=2\nIsGattling=yes\n";
    assert_eq!(draw(gattling, Some(1), 0), (plain, plain_keys));
}

/// S-0105 claim 12, *Sprite body* (S-0007 claim 19; Q-1885 answered by r39): a sprite-body
/// vehicle with the turret flag is drawn as one composed image (*One image*). The body frame comes
/// first, then the single voxel barrel and the sprite turret frame at 4-step turret facings 0 and
/// 3, the turret frame and the barrel at 1 and 2, later opaque pixels winning. It never draws the
/// voxel turret part nor a numbered model, at any selected turret index (claim 43, *Any body*). The
/// composed image takes the voxel-vehicle row *U*, upright, a reader (*Depth*), two units nearer
/// than the sprite row *U* − 2 of a vehicle without the flag, and its rectangle is exactly the
/// union of its parts'. The barrel is drawn in HVA frame 0 whatever the animation counter (*Frame*)
/// and casts no shadow (*No barrel shadow*): the one shadow is the sprite shadow frame, *U* − 2,
/// flat (S-0102 claim 22). Without a barrel part the body and turret frames are still composed;
/// without the flag no voxel part is drawn.
#[test]
fn a_sprite_body_is_drawn_as_one_composed_image() {
    use glasnost_fmt::bytes::F32Bits;
    use glasnost_fmt::hva::synthetic_animation;
    use glasnost_fmt::image::IndexedImage;
    use glasnost_fmt::shp::{ShpWriter, WriteFrame};
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
    use glasnost_render::scene::Item;
    use glasnost_render::voxel;

    use super::build::compose_parts;
    // A test sprite on a 40 × 40 canvas: 16 body frames of 20 × 12 at (10, 14), 32 turret frames
    // of 6 × 6 at (17, 16), inside the body, and 48 shadow frames of 12 × 4.
    const BODY: u8 = 50;
    const TURRET: u8 = 60;
    let frame = |(x, y, w, h, v): (u16, u16, u32, u32, u8)| WriteFrame {
        x,
        y,
        image: IndexedImage::new(w, h, vec![v; (w * h) as usize]).unwrap(),
        compression: 1,
    };
    let sprite = ShpWriter {
        width: 40,
        height: 40,
        frames: (0..96)
            .map(|i| match i {
                0..16 => frame((10, 14, 20, 12, BODY)),
                16..48 => frame((17, 16, 6, 6, TURRET)),
                _ => frame((14, 24, 12, 4, 70)),
            })
            .collect(),
    }
    .write();
    struct Drawn {
        lit: Vec<Item>,
        shadows: Vec<Item>,
        scene: Scene,
        parts: Option<Vec<(u32, [i32; 2])>>,
        cached: [bool; 2],
    }
    let point = project_lifted(2 * 256 + 128, 3 * 256 + 128, 0);
    let draw_as = |extra: &str,
                   parts: &[(&str, f32)],
                   turret: u16,
                   index: Option<i32>,
                   anim: u32,
                   attitude: Attitude| {
        let rules = format!("{RULES}[CARX]\n{extra}");
        let (mut import, catalogue, mut assets) = context_with(&rules);
        assets.insert(b"carx.shp", sprite.clone());
        // A lighting table, so that the barrel's voxels have colours (S-0105 claim 38).
        assets.insert(
            super::build::VPL_FILE,
            glasnost_fmt::vpl::synthetic_table(32).write(),
        );
        let art = import.arts.get_mut(&(Kind::Vehicles, 0)).unwrap();
        assert!(!art.voxels.contains_key("body"), "a sprite body");
        for &(part, width) in parts {
            let file = format!("{part}.vxl");
            art.voxels.insert(part.into(), file.as_bytes().to_vec());
            let mut model = synthetic_model();
            model.limbs[0].max[0] = F32Bits::new(width);
            assets.insert(file.as_bytes(), VxlWriter { model: &model }.write());
            // Four clearly different HVA frames (the frame run of MS-33 part k).
            let hva = file.replace(".vxl", ".hva");
            assets.insert(hva.as_bytes(), synthetic_animation().write());
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
            body: 0,
            turret,
            elevation: voxel::ELEVATION_LEVEL,
            anim,
            infantry: None,
            object: sim_code(ObjectId(9)),
            harvesting: None,
            shadow: None,
            alternative: false,
            attitude,
            disabled: false,
            above: 0,
            protection: None,
            generic_shadow: None,
            berserk: None,
            turret_index: index,
            walk: Default::default(),
        };
        let (entries, _) = painter.mobiles(&inputs, &mut scene, &[car]);
        let (shadows, lit): (Vec<Item>, Vec<Item>) = entries[0]
            .items
            .iter()
            .partition(|i| i.paint == Paint::Shadow);
        let parts = lit.first().and_then(|i| painter.composed_parts(i.image));
        // The barrel's image key (claim 26) in HVA frames 0 and 3; a single-turret type has no
        // index in its key.
        let barrel_key = |f| {
            let index = index.filter(|_| extra.contains("TurretCount"));
            let key = voxel::vehicle_turret_key(
                voxel::RAMP_FLAT,
                0,
                turret,
                0,
                index.and_then(|k| u8::try_from(k).ok()),
                false,
                f,
            );
            painter.voxel_cached(
                Kind::Vehicles,
                0,
                &voxel::barrel_key(key, voxel::ELEVATION_LEVEL),
            )
        };
        let cached = [barrel_key(0), barrel_key(3)];
        Drawn {
            lit,
            shadows,
            scene,
            parts,
            cached,
        }
    };
    let draw = |extra: &str, parts: &[(&str, f32)], turret: u16, index: Option<i32>, anim: u32| {
        draw_as(extra, parts, turret, index, anim, Attitude::None)
    };
    let size = |d: &Drawn, id: u32| {
        let i = &d.scene.images[id as usize];
        (i.width(), i.height())
    };
    // Each part's opaque pixels as (world x, world y, index).
    let opaque = |d: &Drawn, (id, [x, y]): (u32, [i32; 2])| {
        let i = &d.scene.images[id as usize];
        let mut v = Vec::new();
        for iy in 0..i.height() {
            for ix in 0..i.width() {
                let p = i.get(ix, iy).unwrap();
                if p != 0 {
                    v.push((point.0 + x + ix as i32, point.1 + y + iy as i32, p));
                }
            }
        }
        v
    };
    let shown = |d: &Drawn, (x, y): (i32, i32)| {
        let it = d.lit[0];
        let i = &d.scene.images[it.image as usize];
        i.get((x - it.x) as u32, (y - it.y) as u32).unwrap()
    };
    // A sprite vehicle without the turret flag: its body frame alone, S-0102 claim 13's U − 2.
    let plain = draw("", &[("barrel", 20.0)], 0, None, 0);
    assert_eq!(plain.lit.len(), 1, "the body alone");
    assert_eq!(plain.parts, None, "not composed");
    let ItemDepth::Profile {
        bias: sprite_row, ..
    } = plain.lit[0].depth
    else {
        panic!("a profile depth");
    };
    let turret = "Turret=yes\n";
    let all = [("turret", 12.0), ("barrel", 20.0), ("barrel1", 30.0)];
    let mut barrel_pixels = None;
    for (facing, barrel_last) in [(0, false), (16384, true), (32768, true), (49152, false)] {
        let d = draw(turret, &all, facing, None, 0);
        assert_eq!(d.lit.len(), 1, "facing {facing}: one composed image");
        let item = d.lit[0];
        // spec: S-0105 claim 12 (Depth: U, upright, a reader; the body's colour row)
        assert_eq!(
            item.depth,
            ItemDepth::Profile {
                profile: Profile::Upright,
                bias: sprite_row + 2,
                companion: None,
            },
            "facing {facing}: the voxel-vehicle row U"
        );
        assert_eq!(item.test, DepthTest::Reader);
        assert_eq!(item.paint, plain.lit[0].paint);
        let parts = d.parts.clone().expect("a composed image");
        assert_eq!(
            parts.len(),
            3,
            "facing {facing}: body, barrel and turret frame"
        );
        // spec: S-0105 claim 12 (One image: the body first; the barrel then the turret at 0 and 3,
        // the turret then the barrel at 1 and 2)
        let (barrel, top) = if barrel_last {
            (parts[2], parts[1])
        } else {
            (parts[1], parts[2])
        };
        assert_eq!(size(&d, parts[0].0), (20, 12), "the body frame first");
        assert_eq!(size(&d, top.0), (6, 6), "the sprite turret frame");
        assert_ne!(size(&d, barrel.0), (6, 6), "the voxel barrel");
        // The composed image is the parts painted in order, in exactly their union rectangle.
        let composed = compose_parts(&d.scene, &parts).unwrap();
        assert_eq!(
            d.scene.images[item.image as usize].pixels(),
            &composed.pixels[..]
        );
        let rects: Vec<_> = parts
            .iter()
            .map(|&(id, [x, y])| {
                let (w, h) = size(&d, id);
                (x, y, x + w as i32, y + h as i32)
            })
            .collect();
        let l = rects.iter().map(|r| r.0).min().unwrap();
        let t = rects.iter().map(|r| r.1).min().unwrap();
        let r = rects.iter().map(|r| r.2).max().unwrap();
        let b = rects.iter().map(|r| r.3).max().unwrap();
        assert_eq!((item.x, item.y), (point.0 + l, point.1 + t));
        assert_eq!(size(&d, item.image), ((r - l) as u32, (b - t) as u32));
        // Later opaque pixels win, so each part covers the body where it has pixels, and the
        // later of barrel and turret frame wins between them.
        let (under, over) = if barrel_last {
            (top, barrel)
        } else {
            (barrel, top)
        };
        let over_px = opaque(&d, over);
        assert!(!over_px.is_empty());
        for &(x, y, p) in &over_px {
            assert_eq!(shown(&d, (x, y)), p, "facing {facing}: the later part wins");
        }
        let under_px = opaque(&d, under);
        let covered: std::collections::BTreeSet<_> =
            over_px.iter().map(|&(x, y, _)| (x, y)).collect();
        for &(x, y, p) in under_px.iter().filter(|q| !covered.contains(&(q.0, q.1))) {
            assert_eq!(shown(&d, (x, y)), p, "facing {facing}: over the body");
        }
        assert!(
            opaque(&d, barrel)
                .iter()
                .any(|&(x, y, _)| opaque(&d, parts[0]).iter().any(|q| (q.0, q.1) == (x, y))),
            "facing {facing}: the test barrel overlaps the body"
        );
        // spec: S-0105 claim 12 (No barrel shadow: the sprite shadow frame alone, U − 2, flat)
        assert_eq!(d.shadows.len(), 1);
        assert_eq!(
            size(&d, d.shadows[0].image),
            (12, 4),
            "the sprite shadow frame"
        );
        assert_eq!(
            d.shadows[0].depth,
            ItemDepth::Profile {
                profile: Profile::Flat,
                bias: sprite_row,
                companion: None,
            }
        );
        if facing == 0 {
            barrel_pixels = Some(d.scene.images[barrel.0 as usize].clone());
        }
    }
    let barrel_pixels = barrel_pixels.unwrap();
    // spec: S-0105 claim 12 (Frame: HVA frame 0 whatever the animation-frame counter): with the
    // counter at 3 the barrel is the same image, kept under its frame-0 key.
    let late = draw(turret, &all, 0, None, 3);
    let parts = late.parts.clone().unwrap();
    assert_eq!(late.scene.images[parts[1].0 as usize], barrel_pixels);
    assert_eq!(late.cached, [true, false], "the frame-0 key only");
    // The single barrel at every index of a multi-turret type, none included, never barrel k.
    let multi = "Turret=yes\nTurretCount=2\n";
    for index in [None, Some(0), Some(1), Some(5)] {
        let d = draw(multi, &all, 0, index, 0);
        let parts = d.parts.clone().unwrap();
        assert_eq!(parts.len(), 3, "index {index:?}");
        assert_eq!(
            d.scene.images[parts[1].0 as usize], barrel_pixels,
            "index {index:?}: the single barrel"
        );
        assert!(d.cached[0], "index {index:?}: keyed by its index");
    }
    // Without a barrel part the voxel turret part is not drawn either, and the body and turret
    // frames are still composed into one image with the row U.
    let bare = draw(turret, &[("turret", 12.0)], 0, None, 0);
    assert_eq!(bare.lit.len(), 1);
    assert_eq!(bare.lit[0].depth, late.lit[0].depth);
    let parts = bare.parts.clone().unwrap();
    assert_eq!(parts.len(), 2, "body and sprite turret");
    assert_eq!(size(&bare, parts[0].0), (20, 12));
    assert_eq!(size(&bare, parts[1].0), (6, 6));
    // spec: S-0105 claim 12 (Depth: while the parts end above the body frame's last row, the
    // composed rectangle's last row is the body frame's)
    assert_eq!(
        bare.lit[0].y + size(&bare, bare.lit[0].image).1 as i32,
        plain.lit[0].y + 12
    );
    for &(x, y, p) in &opaque(&bare, parts[1]) {
        assert_eq!(shown(&bare, (x, y)), p, "the turret frame over the body");
    }
    // spec: S-0504 claims 15 and 16 (the lean and the missing shadow of a sinking vehicle are
    // drawn only for a vehicle drawn as a voxel model): a sinking sprite body, even one marked
    // with a pitch and a rocking locomotor, draws the same composed image, barrel included, at
    // the same place and row, and keeps its sprite shadow.
    let level = draw(turret, &all, 16384, None, 0);
    let sinking = draw_as(
        turret,
        &all,
        16384,
        None,
        0,
        Attitude::Sinking {
            pitch: 0.5f32.to_bits(),
            roll: 0,
            drawn: true,
        },
    );
    assert_eq!(sinking.lit.len(), 1, "one composed image while sinking");
    let (a, b) = (level.lit[0], sinking.lit[0]);
    assert_eq!((b.x, b.y, b.depth), (a.x, a.y, a.depth));
    assert_eq!(
        sinking.scene.images[b.image as usize].pixels(),
        level.scene.images[a.image as usize].pixels(),
        "no lean on a sprite body or its barrel"
    );
    assert_eq!(
        sinking.shadows.len(),
        1,
        "a sinking sprite body keeps its shadow"
    );
}

/// S-0105 claim 12 (*One image* and *Depth*): the composed image is the smallest rectangle holding
/// every part's rectangle, exactly (no row or column past the lowest or rightmost part, the
/// original's one-row extension fixed without a switch), so its last row, the anchor row of
/// S-0102 F2, is the body's while the other parts end above it and the lowest part's otherwise;
/// transparent pixels paint nothing and a later opaque pixel wins; the parts are painted body
/// first, then barrel and turret frame at 4-step turret facings 0 and 3, turret frame and barrel at
/// 1 and 2.
#[test]
fn composed_parts_fill_their_exact_union_in_order() {
    use glasnost_fmt::image::IndexedImage;

    use super::build::{compose_parts, sprite_turret_parts};
    let mut scene = Scene::default();
    // A 4 × 3 body of 1s, a 2 × 2 turret of 2s with a transparent corner, a 3 × 1 barrel of 3s.
    let body = scene.add_image(IndexedImage::new(4, 3, vec![1; 12]).unwrap());
    let turret = scene.add_image(IndexedImage::new(2, 2, vec![2, 2, 2, 0]).unwrap());
    let barrel = scene.add_image(IndexedImage::new(3, 1, vec![3; 3]).unwrap());
    let rows = |img: &glasnost_render::voxel::PartImage| {
        img.pixels
            .chunks(img.width as usize)
            .map(<[u8]>::to_vec)
            .collect::<Vec<_>>()
    };
    // Both parts end above the body's last row: the body's rectangle, its last row the anchor.
    let parts = [(body, [-2, -3]), (barrel, [-1, -2]), (turret, [0, -3])];
    let img = compose_parts(&scene, &parts).unwrap();
    assert_eq!((img.width, img.height, img.offset), (4, 3, [-2, -3]));
    assert_eq!(
        rows(&img),
        [vec![1, 1, 2, 2], vec![1, 3, 2, 3], vec![1, 1, 1, 1]],
        "the turret over the barrel, its transparent corner showing the barrel"
    );
    // The barrel reaching one row lower: exactly one more row, no further one.
    let parts = [(body, [-2, -3]), (turret, [0, -3]), (barrel, [-1, 0])];
    let img = compose_parts(&scene, &parts).unwrap();
    assert_eq!((img.width, img.height, img.offset), (4, 4, [-2, -3]));
    assert_eq!(
        rows(&img),
        [
            vec![1, 1, 2, 2],
            vec![1, 1, 2, 1],
            vec![1, 1, 1, 1],
            vec![0, 3, 3, 3]
        ]
    );
    let parts = [(body, [-2, -3]), (barrel, [1, -1])];
    let img = compose_parts(&scene, &parts).unwrap();
    assert_eq!(
        (img.width, img.height, img.offset),
        (6, 3, [-2, -3]),
        "two columns further right"
    );
    assert_eq!(rows(&img)[2], [1, 1, 1, 3, 3, 3]);
    assert_eq!(rows(&img)[0], [1, 1, 1, 1, 0, 0]);
    assert!(compose_parts(&scene, &[]).is_none());
    // The painting order by the 4-step turret facing (raw facings: 0 north, 16384 east, …).
    let (b, r, t) = (
        Some((body, [0, 0])),
        Some((barrel, [0, 0])),
        Some((turret, [0, 0])),
    );
    for (facing, barrel_last) in [(0, false), (16384, true), (32768, true), (49152, false)] {
        let order = sprite_turret_parts(b, r, t, facing);
        let want = if barrel_last {
            [body, turret, barrel]
        } else {
            [body, barrel, turret]
        };
        assert_eq!(
            order.iter().map(|p| p.0).collect::<Vec<_>>(),
            want,
            "{facing}"
        );
    }
    assert_eq!(
        sprite_turret_parts(b, None, t, 0).len(),
        2,
        "no barrel part"
    );
    assert_eq!(
        sprite_turret_parts(b, r, None, 0).len(),
        2,
        "no turret frame"
    );
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
        generic_shadow: None,
        berserk: None,
        turret_index: None,
        walk: Default::default(),
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
        tumble: (0, 0),
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
    // spec: S-0113 claims 18 and 20 (a crashing aircraft at strictly positive signed height is
    // drawn with Rz(θ_flight) · Rx(φ) · Ry(ψ′); ψ′ takes α under the speed test; the bank never
    // enters; at zero or negative height the heading alone; the shadow the heading alone)
    let crash = |height: i32, speed: f64, turn: i32, pitch: f32, roll: f32| Attitude::Flying {
        height,
        speed: speed.to_bits(),
        turn,
        crashing: true,
        tumble: (pitch.to_bits(), roll.to_bits()),
    };
    let (tumbled, tumbled_shadow) = images(craft(16384, 20480, crash(10, 0.75, 0, 0.3, 1.2)));
    assert_ne!(tumbled, base, "a tumbling body is drawn rolled");
    assert_ne!(
        tumbled,
        images(craft(16384, 20480, crash(10, 0.75, 0, 0.3, 1.5))).0
    );
    assert_eq!(
        tumbled_shadow, base_shadow,
        "the shadow keeps the heading alone"
    );
    assert_eq!(
        tumbled,
        images(craft(24576, 20480, crash(10, 0.75, 0, 0.3, 1.2))).0,
        "the heading is the aiming facing, not the body facing"
    );
    // The bank never enters: a turn either way draws the same body.
    assert_eq!(
        tumbled,
        images(craft(16384, 20480, crash(10, 0.75, 1, 0.3, 1.2))).0
    );
    assert_eq!(
        tumbled,
        images(craft(16384, 20480, crash(10, 0.75, -1, 0.3, 1.2))).0
    );
    // α only on the speed test (PitchSpeed 0.5 here): the same tumble below it is another body.
    let slow = images(craft(16384, 20480, crash(10, 0.4, 0, 0.3, 1.2))).0;
    assert_ne!(
        tumbled, slow,
        "the drawing pitch enters above the threshold"
    );
    assert_eq!(
        slow,
        images(craft(16384, 20480, crash(10, 0.5, 1, 0.3, 1.2))).0,
        "at the threshold, no α"
    );
    // Tiny angles are drawn by D_crash too, not as a settled level body: the α shows on them.
    let tiny_slow = images(craft(16384, 20480, crash(10, 0.4, 0, 0.001, 0.004))).0;
    let tiny_fast = images(craft(16384, 20480, crash(10, 0.75, 0, 0.001, 0.004))).0;
    assert_ne!(tiny_slow, tiny_fast);
    // At zero or negative height, the heading alone, the level flying body's image.
    assert_eq!(
        images(craft(16384, 20480, crash(0, 0.75, 1, 0.3, 1.2))).0,
        base
    );
    assert_eq!(
        images(craft(16384, 20480, crash(-5, 0.75, 0, 0.3, 1.2))).0,
        base
    );
    assert_eq!(
        images(craft(16384, 20480, crash(0, 0.75, 1, 0.3, 1.2))).1,
        base_shadow
    );
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
            generic_shadow: None,
            berserk: None,
            turret_index: None,
            walk: Default::default(),
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

/// The spot groups drawn after every item (beam effects, S-0102 claim 2).
fn beam_groups(scene: &Scene) -> Vec<&glasnost_render::spots::SpotGroup> {
    scene.spots.iter().filter(|g| g.at == usize::MAX).collect()
}

/// spec: S-0518 claims 17, 18 and 22 (a support beam: inner L of the supporter's house, house-colour
/// drawing, W = 3; coarse while the activity test fails), claim 19 (drawn for D ticks), S-0102 claim 2
/// (beam effects after all layers and status extras)
#[test]
fn support_beam_draws_in_the_house_laser_colour_for_its_life() {
    use glasnost_render::spots::{SpotKind, pack};
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
    // From (4, 6) to (6, 4) cells: the same screen row, 120 pixels apart (S-0101 F1).
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
    let groups = beam_groups(&scene);
    assert_eq!(groups.len(), 1);
    // The activity count is still below the floor: coarse mode, opaque lines; the centre (drawn
    // last) is L moved toward white at I = 1.
    let name = live.world.houses[0].name.clone();
    let rgb = super::build::owner_radar_colour(&import, &options, name.as_bytes(), [8, 8, 8]);
    let l = super::beams::laser_colour(rgb);
    let centre = l.map(|c| u32::from(c) + (255 - u32::from(c)) * 255 / 256);
    let last = groups[0].spots.last().expect("drawn");
    assert_eq!(
        last.kind,
        SpotKind::Opaque {
            colour: pack(centre)
        }
    );
    assert!(
        groups[0]
            .spots
            .iter()
            .all(|s| matches!(s.kind, SpotKind::Opaque { .. }))
    );
    // Shown for 3 ticks, then gone.
    for _ in 0..2 {
        live.step();
        live.redraw(&inputs, &mut scene);
        assert_eq!(beam_groups(&scene).len(), 1);
    }
    live.step();
    live.redraw(&inputs, &mut scene);
    assert!(beam_groups(&scene).is_empty());
}

/// spec: S-0615 claims 15 and 16 with S-0518 claims 18, 19 and 21 (a disc laser's beam: ordinary
/// drawing in its colours, additive at detail 2 with outer lines, fading toward none over D ticks)
#[test]
fn disc_laser_beam_fades_in_its_colours() {
    use glasnost_render::spots::SpotKind;
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
    let kinds = |scene: &Scene| {
        let g = beam_groups(scene)[0];
        (g.spots[0].kind, g.spots.last().expect("drawn").kind)
    };
    // The outer lines first, the centre last, each at I = 1.
    assert_eq!(
        kinds(&scene),
        (
            SpotKind::Additive { rgb: [100, 50, 20] },
            SpotKind::Additive {
                rgb: [200, 100, 40]
            }
        )
    );
    // P = 2 of D = 4: I = 1/2 on the way to none.
    live.step();
    live.step();
    live.redraw(&inputs, &mut scene);
    assert_eq!(kinds(&scene).1, SpotKind::Additive { rgb: [100, 50, 20] });
}

/// spec: S-0607 claim 26 (the magnetic wave is drawn in the air layer, before the effects after the
/// layers), claim 25 (it is drawn until removed and then vanishes at once)
#[test]
fn magnetic_wave_is_drawn_in_the_air_layer_until_removed() {
    use glasnost_render::spots::SpotKind;
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
    let firer = *live.world.objects.keys().next().expect("an object");
    let id = ObjectId(1 << 20);
    // An ended wave: its ends stay those it was made with.
    live.world.waves.insert(
        id,
        glasnost_sim::lift::Wave {
            firer,
            target: Target::Cell(glasnost_map::CellXY::new(5, 5)),
            a: glasnost_num::soft::F64::ZERO,
            b: glasnost_num::soft::F64::ZERO,
            ended: true,
        },
    );
    live.world.effects.push(Effect::Wave {
        id,
        from: Pos {
            x: 4 * 256,
            y: 4 * 256,
            z: 0,
        },
        to: Pos {
            x: 7 * 256,
            y: 4 * 256,
            z: 0,
        },
    });
    live.after_tick();
    live.redraw(&inputs, &mut scene);
    let waves: Vec<_> = scene
        .spots
        .iter()
        .filter(|g| {
            g.spots
                .iter()
                .any(|s| matches!(s.kind, SpotKind::Wave { .. }))
        })
        .collect();
    assert_eq!(waves.len(), 1);
    assert!(waves[0].at >= scene.cell_pass_len && waves[0].at <= scene.items.len());
    live.world.waves.remove(&id);
    live.step();
    live.redraw(&inputs, &mut scene);
    assert!(scene.spots.is_empty());
}

/// spec: S-0514 claim 27 with S-0109 Terms (an erased body's code 2 or 3 combines with the cloak's:
/// the larger, except that 1 and 2 give 3)
#[test]
fn erasure_translucency_combines_with_the_cloak() {
    use super::live::combine_codes;
    use glasnost_render::compose::Translucency::{Half, Quarter, ThreeQuarter};
    assert_eq!(combine_codes(None, Some(Half)), Some(Half));
    assert_eq!(combine_codes(Some(Quarter), Some(Half)), Some(ThreeQuarter));
    assert_eq!(combine_codes(Some(Half), Some(Half)), Some(Half));
    assert_eq!(
        combine_codes(Some(Quarter), Some(ThreeQuarter)),
        Some(ThreeQuarter)
    );
    assert_eq!(combine_codes(None, None), None);
}

/// spec: S-0513 claim 40 and S-0514 claim 27 (an object phasing in by a warp, from departure until
/// the arrival hold ends, is drawn with code 2, as is the leaving stage of a body being erased;
/// a pending warp is not)
#[test]
fn warp_phasing_in_is_drawn_half_translucent() {
    use glasnost_render::compose::Translucency;
    use glasnost_sim::superweapons::warp::{Member, Stage};
    let (mut live, _scene, _map, _import, _catalogue, cart, _walker) = combat_live(3);
    let pos = live.world.obj(cart).unwrap().pos;
    let set = |live: &mut Live, stage: Option<Stage>| {
        live.world.obj_mut(cart).unwrap().sw.warp = stage.map(|stage| Member {
            stage,
            house: HouseId(0),
            landing: pos,
            redirected: false,
            fell_back: false,
            refused: false,
            entry_ok: true,
        });
    };
    assert_eq!(live.erased_code(cart), None);
    for stage in [
        Stage::Leaving(0),
        Stage::Expired,
        Stage::Blocked,
        Stage::Away,
        Stage::Placed,
        Stage::Hold(0),
    ] {
        set(&mut live, Some(stage));
        assert_eq!(
            live.erased_code(cart),
            Some(Translucency::Half),
            "{stage:?}"
        );
    }
    set(&mut live, Some(Stage::Pending));
    assert_eq!(live.erased_code(cart), None, "pending");
    set(&mut live, None);
    assert_eq!(live.erased_code(cart), None);
}

/// S-0301 claim 23 (no default or placeholder picture at the muzzle), S-0303 claim 21 (a detonation shows its
/// explosion animation only): the game client draws
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
            z_adjust: None,
            id: None,
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
            z_adjust: None,
            id: None,
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
        "{RULES}[AudioVisual]\nIronCurtainColor=1\nForceShieldColor=2\n[ColorAdd]\nA=0,0,0\nB=7,19,3\nC=1,0,1\nD=0,8,0\n"
    )
}

/// The painter inputs of a protection test, drawn by `f`.
fn with_protection_inputs<T>(
    voxel: bool,
    f: impl FnOnce(&Inputs, &mut super::build::Painter, &mut Scene) -> T,
) -> T {
    with_rules_inputs(voxel, &protection_rules(), f)
}

/// The painter inputs of the rules `rules` (CARX drawn as voxels when
/// `voxel`), drawn by `f`.
fn with_rules_inputs<T>(
    voxel: bool,
    rules: &str,
    f: impl FnOnce(&Inputs, &mut super::build::Painter, &mut Scene) -> T,
) -> T {
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
    let (mut import, catalogue, mut assets) = context_with(rules);
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
                generic_shadow: None,
                berserk: None,
                turret_index: None,
                walk: Default::default(),
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
                    adjust: Default::default(),
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
    cells: std::collections::BTreeMap<glasnost_map::CellXY, super::protection::SlotCell>,
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
        adjust: Default::default(),
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
) -> std::collections::BTreeMap<glasnost_map::CellXY, super::protection::SlotCell> {
    look.map(|l| {
        hut_cells_with(super::protection::BuildingLook {
            protection: Some(l),
            ..Default::default()
        })
    })
    .unwrap_or_default()
}

/// HUTX's four foundation cells, each with the building look `look` and
/// not shrouded.
fn hut_cells_with(
    look: super::protection::BuildingLook,
) -> std::collections::BTreeMap<glasnost_map::CellXY, super::protection::SlotCell> {
    hut_cells_shrouded(look, &[])
}

/// HUTX's four foundation cells, each with the building look `look`, the
/// cells `shrouded` shrouded for the local player.
fn hut_cells_shrouded(
    look: super::protection::BuildingLook,
    shrouded: &[(i32, i32)],
) -> std::collections::BTreeMap<glasnost_map::CellXY, super::protection::SlotCell> {
    [(2, 2), (3, 2), (2, 3), (3, 3)]
        .map(|(x, y)| {
            (
                glasnost_map::CellXY::new(x, y),
                super::protection::SlotCell {
                    look,
                    shrouded: shrouded.contains(&(x, y)),
                },
            )
        })
        .into_iter()
        .collect()
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
        ..Default::default()
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
        cloak_items(look, false, None, &mut items);
        assert_eq!(
            items[0].paint,
            Paint::Translucent(shade.with_mask(0), t),
            "{look:?}: the envelope's ρ kept, no mask"
        );
    }
    let mut items = vec![item];
    cloak_items(Look::Normal, true, None, &mut items);
    assert_eq!(items[0].paint, Paint::Lit(shade), "opaque: the mask again");
    let mut items = vec![item];
    cloak_items(Look::Hidden, true, None, &mut items);
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
    // An occupying building whose centre cell is shrouded still supplies its mask while the
    // animation's own cell is not shrouded (S-0513 claim 29b: the centre cell plays no part).
    let centre_shrouded = anim(Some(bright(ProtectKind::ForceShield).shrouded(true)), "", 0);
    assert_eq!(centre_shrouded.2.mask, shield);
}

/// spec: S-0513 claim 29b (slot animations, Q-1529; SW10 parts i1, i2, i3 and i5 in our engine:
/// the animation tests the cell holding its own position; that cell not shrouded, S-0102 claim
/// 34's masks of the occupying building, whatever the state of the building's centre cell; that
/// cell shrouded, no colour mask at all, neither the force-shield nor the laser-target mask; the
/// brightness follows S-0102 claim 34 and S-0519 claim 26 unchanged), S-0519 claim 26
#[test]
fn protection_claim29b_slot_animation_own_cell() {
    use glasnost_render::compose::{PixelLayout, protection_mask};
    use glasnost_sim::mark::Adjust;
    use glasnost_sim::superweapons::ProtectKind;
    let shield = protection_mask([1, 0, 1], PixelLayout::Rgb565);
    let laser = protection_mask(MARK_TRIPLE, PixelLayout::Rgb565);
    assert!(shield & laser == 0 && shield != 0 && laser != 0);
    // HUTX from cell (2, 2): its centre cell is (3, 3) (S-0104 claim 13), its slot animation
    // with no displacement lies in cell E = (2, 2), the anchor cell.
    let draw = |protection: Option<super::protection::Look>, adjust, shrouded: &[(i32, i32)]| {
        let mut b = hut(protection, None);
        b.adjust = adjust;
        b.slots = vec![slot_a(0)];
        let look = super::protection::BuildingLook { protection, adjust };
        let parts = lit_parts(&protected_hut_items(
            MARK_SELECTORS,
            &[],
            "",
            b,
            hut_cells_shrouded(look, shrouded),
        ));
        (parts[0].2, parts.last().expect("the slot animation").2)
    };
    let none = Adjust::default();
    let mark = Adjust {
        mark: Some(Some(512)),
        ..Adjust::default()
    };
    let visible = Some(bright(ProtectKind::ForceShield));
    let centre_hidden = visible.map(|l| l.shrouded(true));
    // i1: E visible, the centre cell shrouded: the animation shows the force-shield colour, the
    // body none.
    let (body, slot) = draw(centre_hidden, none, &[(3, 3)]);
    assert_eq!((body.mask, slot.mask), (0, shield), "i1");
    // i2: E shrouded, the centre cell visible: the body shows the colour, the animation only the
    // protected brightness.
    let (body_i2, slot_i2) = draw(visible, none, &[(2, 2)]);
    assert_eq!((body_i2.mask, slot_i2.mask), (shield, 0), "i2");
    let (_, slot_open) = draw(visible, none, &[]);
    assert_eq!(slot_open.mask, shield);
    assert_eq!(slot_i2.rho, slot_open.rho, "i2: the brightness unchanged");
    // i3: the colour follows E only, not another foundation corner.
    let (_, slot) = draw(centre_hidden, none, &[(3, 2), (3, 3)]);
    assert_eq!(slot.mask, shield, "i3: a corner shrouded, E visible");
    let (_, slot) = draw(centre_hidden, none, &[(2, 2), (3, 3)]);
    assert_eq!(slot.mask, 0, "i3: E shrouded, the corner visible");
    // i5: also marked: the union while E is visible, neither half while E is shrouded.
    let (_, slot) = draw(centre_hidden, mark, &[(3, 3)]);
    assert_eq!(slot.mask, shield | laser, "i5, i1 placement");
    let (_, slot_marked_hidden) = draw(visible, mark, &[(2, 2)]);
    assert_eq!(slot_marked_hidden.mask, 0, "i5, i2 placement");
    let (_, slot_marked_open) = draw(visible, mark, &[]);
    assert_eq!(slot_marked_hidden.rho, slot_marked_open.rho);
    // Only marked: the marking colour while E is visible, none while it is shrouded (S-0519
    // claim 26); the brightness unchanged.
    let (_, open) = draw(None, mark, &[]);
    let (_, hidden) = draw(None, mark, &[(2, 2)]);
    assert_eq!((open.mask, hidden.mask), (laser, 0));
    assert_eq!(open.rho, hidden.rho);
}

/// spec: S-0513 claim 29b (slot animations: the cell holding the animation's own position is
/// shrouded when S-0110 claim 8's point test at its centre, at its ground height, finds it so;
/// on raised ground that test reads a displaced cell, SW10 part i4; only the local player's
/// shroud counts), S-0110 claim 8, S-0519 claim 26 (Houses: shroud per viewer)
#[test]
fn protection_slot_cell_shroud_is_the_point_test() {
    use super::live::slot_cell_shrouded;
    use glasnost_sim::econ::shroud;
    let (mut live, _) = synthetic_live();
    let (a, b) = (HouseId(0), HouseId(1));
    // A cell of the plateau one level up (testkit::map: cell(0, −6)): u = 1, so the test reads
    // the cell itself and its map north-west neighbour.
    let up = glasnost_sim::testkit::cell(0, -6);
    let nw = glasnost_map::CellXY::new(up.x - 1, up.y - 1);
    assert_eq!(live.world.cells.cell(up).unwrap().level, 1);
    assert!(slot_cell_shrouded(&live.world, a, up), "nothing explored");
    // Explore the north-west neighbour but not the cell itself: the raised cell is not
    // shrouded, although its own explored state is not set (the displaced reading).
    let idx = |c: glasnost_map::CellXY| live.world.cells.idx(c).map(|i| i as u32);
    let (i_nw, i_up) = (idx(nw), idx(up));
    let reveal = (-4..=0)
        .flat_map(|dx| (-4..=0).map(move |dy| (dx, dy)))
        .flat_map(|(dx, dy)| (1..=3).map(move |r| (dx, dy, r)))
        .find_map(|(dx, dy, r)| {
            let c = glasnost_map::CellXY::new(nw.x + dx, nw.y + dy);
            let centre = glasnost_sim::world::Pos::centre(c, 0);
            let z = live.world.surface_at(centre.x, centre.y);
            let p = glasnost_sim::world::Pos::new(centre.x, centre.y, z);
            let cells = shroud::sight_cells(&live.world, p, r);
            (i_nw.is_some_and(|i| cells.contains(&i)) && !i_up.is_some_and(|i| cells.contains(&i)))
                .then_some((p, r))
        })
        .expect("a reveal that covers the north-west neighbour only");
    shroud::area_reveal(&mut live.world, a, reveal.0, reveal.1);
    assert!(shroud::explored(&live.world, a, nw) && !shroud::explored(&live.world, a, up));
    assert!(
        !slot_cell_shrouded(&live.world, a, up),
        "displaced cell explored"
    );
    // A flat cell at level 0 reads its own explored state.
    let flat = glasnost_sim::testkit::cell(-8, 6);
    assert_eq!(live.world.cells.cell(flat).unwrap().level, 0);
    assert!(slot_cell_shrouded(&live.world, b, flat));
    shroud::area_reveal(
        &mut live.world,
        b,
        glasnost_sim::world::Pos::centre(flat, 0),
        1,
    );
    assert!(shroud::explored(&live.world, b, flat));
    assert!(!slot_cell_shrouded(&live.world, b, flat), "explored for b");
    assert!(
        slot_cell_shrouded(&live.world, a, flat),
        "still shrouded for a"
    );
}

/// spec: S-0513 claim 29b and SW10 part i4 (on ground raised four levels, u = 4, the slot
/// animation's shroud test for cell (x, y) reads the explored state of cell (x − 2, y − 2), not
/// the cell's own), S-0110 claim 8
#[test]
fn protection_slot_cell_shroud_on_ground_raised_four_levels() {
    use super::live::slot_cell_shrouded;
    use glasnost_sim::econ::shroud;
    let mut map = testkit::empty_map(24);
    let all: Vec<glasnost_map::CellXY> = map.cells.iter().map(|(c, _)| c).collect();
    for c in all {
        if let Some(cell) = map.cells.get_mut(c) {
            cell.level = 4;
        }
    }
    let mut w = glasnost_sim::load::new_game(
        testkit::rules(),
        &map,
        testkit::catalogue(),
        &testkit::setup(3),
    )
    .expect("the raised map loads");
    let viewer = HouseId(1);
    let e = testkit::mid(24, 0, 0);
    let displaced = glasnost_map::CellXY::new(e.x - 2, e.y - 2);
    let centre = glasnost_sim::world::Pos::centre(e, 0);
    assert_eq!(w.surface_at(centre.x, centre.y), 4 * 104);
    // A reveal over the cell itself but not its displaced cell, then one over the displaced cell.
    let idx = |w: &glasnost_sim::world::World, c| w.cells.idx(c).map(|i| i as u32);
    let find = |w: &glasnost_sim::world::World, yes, no| {
        let (iy, ino) = (idx(w, yes), idx(w, no));
        (-3..=3)
            .flat_map(|dx| (-3..=3).map(move |dy| (dx, dy)))
            .flat_map(|(dx, dy)| (1..=2).map(move |r| (dx, dy, r)))
            .find_map(|(dx, dy, r)| {
                let c = glasnost_map::CellXY::new(yes.x + dx, yes.y + dy);
                let p = glasnost_sim::world::Pos::centre(c, 0);
                let p = glasnost_sim::world::Pos::new(p.x, p.y, w.surface_at(p.x, p.y));
                let cells = shroud::sight_cells(w, p, r);
                (iy.is_some_and(|i| cells.contains(&i)) && !ino.is_some_and(|i| cells.contains(&i)))
                    .then_some((p, r))
            })
            .expect("a reveal that covers one cell and not the other")
    };
    let own = find(&w, e, displaced);
    shroud::area_reveal(&mut w, viewer, own.0, own.1);
    assert!(shroud::explored(&w, viewer, e) && !shroud::explored(&w, viewer, displaced));
    assert!(
        slot_cell_shrouded(&w, viewer, e),
        "its own state explored, the displaced cell not: shrouded"
    );
    let other = find(&w, displaced, e);
    let mut w2 = glasnost_sim::load::new_game(
        testkit::rules(),
        &map,
        testkit::catalogue(),
        &testkit::setup(3),
    )
    .expect("the raised map loads");
    shroud::area_reveal(&mut w2, viewer, other.0, other.1);
    assert!(!shroud::explored(&w2, viewer, e) && shroud::explored(&w2, viewer, displaced));
    assert!(
        !slot_cell_shrouded(&w2, viewer, e),
        "the displaced cell explored, its own state not: not shrouded"
    );
}

/// spec: S-0513 claim 29b and SW10 part i6 (revision 34, Q-1740 answered: on a sloped cell the
/// slot animation's shroud test is made at the terrain's surface height at the centre, S-0106
/// claims 2 and 3, not the flat height of the cell's level; a valley or steep slope at level 0
/// has its centre a whole level up, so u = 1 and the test reads *E* and its north-west
/// neighbour *N*; the control, an edge slope with its centre half a level up, has u = 0 and reads
/// *E* alone), S-0110 claim 8
#[test]
fn protection_slot_cell_shroud_on_a_sloped_cell_uses_the_surface_height() {
    use super::live::slot_cell_shrouded;
    use glasnost_sim::econ::shroud;
    let e = testkit::mid(24, 0, 0);
    let n = glasnost_map::CellXY::new(e.x - 1, e.y - 1);
    let viewer = HouseId(1);
    // (slope shape, centre height, shrouded with N explored and E not)
    for (shape, centre_z, shrouded) in [(9u8, 104, false), (13, 104, false), (1, 52, true)] {
        let map = testkit::empty_map(24);
        let mut w = glasnost_sim::load::new_game(
            testkit::rules(),
            &map,
            testkit::catalogue(),
            &testkit::setup(3),
        )
        .expect("the map loads");
        // The slope shape set by hand (the synthetic catalogue has no ramp tiles).
        w.cells.grid.get_mut(e).expect("cell E").slope = shape;
        let centre = glasnost_sim::world::Pos::centre(e, 0);
        assert_eq!(w.cells.cell(e).unwrap().level, 0, "level 0");
        assert_eq!(w.surface_at(centre.x, centre.y), centre_z, "shape {shape}");
        // A reveal of N but not E.
        let idx = |w: &glasnost_sim::world::World, c| w.cells.idx(c).map(|i| i as u32);
        let (i_n, i_e) = (idx(&w, n), idx(&w, e));
        let reveal = (-4..=0)
            .flat_map(|dx| (-4..=0).map(move |dy| (dx, dy)))
            .flat_map(|(dx, dy)| (1..=2).map(move |r| (dx, dy, r)))
            .find_map(|(dx, dy, r)| {
                let c = glasnost_map::CellXY::new(n.x + dx, n.y + dy);
                let p = glasnost_sim::world::Pos::centre(c, 0);
                let p = glasnost_sim::world::Pos::new(p.x, p.y, w.surface_at(p.x, p.y));
                let cells = shroud::sight_cells(&w, p, r);
                (i_n.is_some_and(|i| cells.contains(&i))
                    && !i_e.is_some_and(|i| cells.contains(&i)))
                .then_some((p, r))
            })
            .expect("a reveal that covers N and not E");
        shroud::area_reveal(&mut w, viewer, reveal.0, reveal.1);
        assert!(shroud::explored(&w, viewer, n) && !shroud::explored(&w, viewer, e));
        assert_eq!(
            slot_cell_shrouded(&w, viewer, e),
            shrouded,
            "shape {shape}: the surface height's u decides which cells are read"
        );
    }
}

/// spec: S-0513 claim 29b (slot animations: each cell of a force-shielded building carries its own
/// shroud answer for the local player, so an animation over a visible edge cell *E* takes the
/// mask while the centre cell is shrouded, SW10 part i1; the local player is not the owner;
/// without a local player nothing counts as shrouded), S-0406 claim 2 (the foundation cells)
#[test]
fn protection_slot_cells_carry_their_own_shroud() {
    use glasnost_sim::econ::shroud;
    use glasnost_sim::rules::Kind as SimKind;
    use glasnost_sim::superweapons::{ProtectKind, Protection, Tint};
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
    let mut ruleset = testkit::combat_ruleset();
    ruleset.sim.buildings.get_mut("TOWER").unwrap().foundation = Some([3, 3]);
    let mut world = testkit::game_with(&ruleset, 24, 3);
    let anchor = testkit::mid(24, 0, 0);
    let tower = testkit::spawn(&mut world, SimKind::Building, "TOWER", 0, anchor, 0);
    let tick = world.tick;
    world.obj_mut(tower).unwrap().sw.protection = Some(Protection {
        start: tick - 1,
        duration: 1000,
        kind: ProtectKind::ForceShield,
        tint: Tint::Rise(tick - 1),
    });
    let rules = world.rules_arc();
    let mut live = Live::new(
        &inputs,
        &mut built,
        painter,
        (world, rules, testkit::setup(3), Vec::new()),
    );
    let at = |dx, dy| glasnost_map::CellXY::new(anchor.x + dx, anchor.y + dy);
    // E: the middle cell of the edge away from the anchor; P: the centre cell.
    let (e, centre) = (at(2, 1), at(1, 1));
    let viewer = HouseId(1);
    let idx = |live: &Live, c| live.world.cells.idx(c).map(|i| i as u32);
    let (i_e, i_p) = (idx(&live, e), idx(&live, centre));
    let reveal = (3..=6)
        .flat_map(|dx| (-2..=4).map(move |dy| (dx, dy)))
        .flat_map(|(dx, dy)| (1..=3).map(move |r| (dx, dy, r)))
        .find_map(|(dx, dy, r)| {
            let p = glasnost_sim::world::Pos::centre(at(dx, dy), 0);
            let cells = shroud::sight_cells(&live.world, p, r);
            (i_e.is_some_and(|i| cells.contains(&i)) && !i_p.is_some_and(|i| cells.contains(&i)))
                .then_some((p, r))
        })
        .expect("a reveal that covers E but not the centre cell");
    shroud::area_reveal(&mut live.world, viewer, reveal.0, reveal.1);
    assert!(shroud::explored(&live.world, viewer, e));
    assert!(!shroud::explored(&live.world, viewer, centre));
    live.set_viewer(Some(viewer));
    let cells = live.protected_cells();
    assert_eq!(cells.len(), 9, "the 3 × 3 foundation");
    let (ce, cp) = (cells[&e], cells[&centre]);
    assert!(!ce.shrouded && cp.shrouded);
    assert!(
        ce.look.protection.is_some_and(|l| l.shrouded),
        "the body's centre-cell answer: shrouded"
    );
    let m = super::protection::Masks {
        shield: 0x0841,
        ..Default::default()
    };
    assert_eq!(m.slot_animation(Some(ce)), 0x0841, "i1: E visible");
    assert_eq!(m.slot_animation(Some(cp)), 0, "over the shrouded centre");
    assert_eq!(m.building_look(ce.look), 0, "the body: no mask");
    live.set_viewer(None);
    assert!(live.protected_cells().values().all(|c| !c.shrouded));
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

/// spec: S-0607 claim 26 (a wave is drawn after the air layer's animations made before it, even when a particle precedes such an animation in the list)
#[test]
fn magnetic_wave_follows_the_older_animations_of_the_air_list() {
    let p = Entrant::Particle(ObjectId(40));
    let m = Entrant::Object(ObjectId(41));
    // Animations 3 and 4 were made before the wave (its next serial was 5), 6 after it.
    let list = [
        p,
        Entrant::Anim(3),
        m,
        p,
        Entrant::Anim(4),
        Entrant::Anim(6),
        p,
    ];
    // After animation 4 (index 4), before the newer animation 6 (index 5).
    assert_eq!(wave_slots(&list, [5].into_iter()), vec![5]);
    // Without a newer animation: before the first particle past the older ones.
    let list = [p, Entrant::Anim(3), m, p];
    assert_eq!(wave_slots(&list, [5].into_iter()), vec![3]);
    // Nothing after the older animations: at the end.
    let list = [Entrant::Anim(3), m];
    assert_eq!(wave_slots(&list, [5].into_iter()), vec![2]);
    // No older animation: before the first particle or newer animation.
    let list = [m, Entrant::Anim(7), p];
    assert_eq!(wave_slots(&list, [5, 8].into_iter()), vec![1, 2]);
}

// ---------------------------------------------------------------------------
// Sprites and effects of intake 14B: the charge flash and the strike mark on
// buildings (S-0519 claims 25 and 26, F7), the bomb icon (claim 24), the
// berserk mask (S-0610 claim 7) and the generic infantry shadow (S-0107
// claim 19)
// ---------------------------------------------------------------------------

/// The audio-visual selectors of a marked-building test: the curtain and
/// force-shield selectors of [`protection_rules`] restated (a second
/// section of the same name in the file takes the place of the first) and
/// the laser-target selector 3, whose triple shares no bit with the
/// force-shield triple, so their union differs from each mask alone (as
/// S-0513 SW10 part i5 asks).
const MARK_SELECTORS: &str =
    "[AudioVisual]\nIronCurtainColor=1\nForceShieldColor=2\nLaserTargetColor=3\n";

/// The laser-target triple [`MARK_SELECTORS`] selects.
const MARK_TRIPLE: [u8; 3] = [0, 8, 0];

/// HUTX's body (first lit part) and its SLOTA slot animation (last lit
/// part) under `protection` and `adjust`, with [`MARK_SELECTORS`] and the
/// slot animation's art `art`.
fn marked_hut(
    protection: Option<super::protection::Look>,
    adjust: glasnost_sim::mark::Adjust,
    art: &str,
) -> (
    glasnost_render::compose::Shade,
    glasnost_render::compose::Shade,
) {
    let mut b = hut(protection, None);
    b.adjust = adjust;
    b.slots = vec![slot_a(0)];
    let look = super::protection::BuildingLook { protection, adjust };
    let cells = if look.active() {
        hut_cells_with(look)
    } else {
        Default::default()
    };
    let parts = lit_parts(&protected_hut_items(MARK_SELECTORS, &[], art, b, cells));
    (parts[0].2, parts.last().expect("the slot animation").2)
}

/// spec: S-0519 claim 26 (a marked building's body sprite and the opaque building animations at
/// its cells take the laser-target selector's mask, united with the force-shield mask; curtain
/// protection adds none; the mask never pulses), F7 (marked in a phase with a factor:
/// min(2000, ⌊m × b₂ / 256⌋); without a factor: b₂), S-0102 claim 34 animation rule (the normal
/// light override still forces the animation's brightness)
#[test]
fn mark_claim26_body_and_slot_rows() {
    use glasnost_render::compose::{PixelLayout, protection_mask};
    use glasnost_sim::mark::Adjust;
    use glasnost_sim::superweapons::ProtectKind;
    let laser = protection_mask(MARK_TRIPLE, PixelLayout::Rgb565);
    let shield = protection_mask([1, 0, 1], PixelLayout::Rgb565);
    assert_eq!(shield & laser, 0, "the union differs from each mask alone");
    let marked = |m: Option<i32>| Adjust {
        mark: Some(m),
        ..Default::default()
    };
    let (plain_body, plain_slot) = marked_hut(None, Adjust::default(), "");
    assert_eq!((plain_body.mask, plain_slot.mask), (0, 0));
    // Marked at factor 512: twice the brightness, the laser-target mask on body and animation.
    let (body, slot) = marked_hut(None, marked(Some(512)), "");
    assert_eq!((body.mask, slot.mask), (laser, laser));
    assert!(body.rho > plain_body.rho && slot.rho > plain_slot.rho);
    // Marked in a phase without a factor: the brightness unchanged, the mask the same.
    let (body, slot) = marked_hut(None, marked(None), "");
    assert_eq!((body.mask, slot.mask), (laser, laser));
    assert_eq!((body.rho, slot.rho), (plain_body.rho, plain_slot.rho));
    // Force-shielded too: the union of both masks; curtain-protected: the laser mask alone.
    let (body, slot) = marked_hut(
        Some(bright(ProtectKind::ForceShield)),
        marked(Some(256)),
        "",
    );
    assert_eq!((body.mask, slot.mask), (laser | shield, laser | shield));
    let (body, _) = marked_hut(Some(bright(ProtectKind::Curtain)), marked(Some(256)), "");
    assert_eq!(body.mask, laser);
    // The normal-light override still wins for the animation; the mask stays.
    let (_, normal_plain) = marked_hut(None, Adjust::default(), "UseNormalLight=yes\n");
    let (_, normal) = marked_hut(None, marked(Some(512)), "UseNormalLight=yes\n");
    assert_eq!(normal.rho, normal_plain.rho);
    assert_eq!(normal.mask, laser);
}

/// spec: S-0519 claim 25 and F6 (the flash is on when c mod 4 is 2 or 3: the body and the
/// animations with stored brightness are brightened; no colour change), F7 (unmarked and
/// unprotected: b₁)
#[test]
fn flash_claim25_body_and_slot_rows() {
    use glasnost_sim::mark::Adjust;
    let flash = |c| Adjust {
        flash: c,
        ..Default::default()
    };
    let (plain_body, plain_slot) = marked_hut(None, Adjust::default(), "");
    for c in [2, 3, 6, 7] {
        let (body, slot) = marked_hut(None, flash(c), "");
        assert_eq!((body.mask, slot.mask), (0, 0), "no colour change, c {c}");
        assert!(
            body.rho > plain_body.rho && slot.rho > plain_slot.rho,
            "on, c {c}"
        );
    }
    for c in [1, 4, 5] {
        let (body, slot) = marked_hut(None, flash(c), "");
        assert_eq!(
            (body.rho, slot.rho),
            (plain_body.rho, plain_slot.rho),
            "off, c {c}"
        );
    }
}

/// spec: S-0519 F7 (b₁ = F6's flashed value, b₂ = P(b₁), then the mark factor capped at 2000 ‰;
/// a negative b₂ rounds toward minus infinity), S-0513 F9 (P), S-0519 claim 26 (a building that
/// is also force-shielded takes the union of both masks; curtain adds none), S-0610 claim 7 (the
/// berserk mask: voxel vehicle and infantry, none for a sprite vehicle or an aircraft, none for
/// infantry whose cell centre is shrouded)
#[test]
fn building_look_and_mask_tables() {
    use super::protection::{BuildingLook, Look, Masks, MobileKind};
    use glasnost_sim::mark::Adjust;
    use glasnost_sim::superweapons::ProtectKind;
    let look = |flash, protection: Option<Look>, mark| BuildingLook {
        protection,
        adjust: Adjust {
            flash,
            protection: None,
            mark,
        },
    };
    let shielded = Some(bright(ProtectKind::ForceShield));
    assert_eq!(look(0, None, None).brightness(800), 800);
    assert_eq!(look(2, None, None).brightness(800), 1300, "flash on: +500");
    assert_eq!(
        look(2, None, None).brightness(1600),
        1100,
        "above 1500: −500"
    );
    assert_eq!(look(1, None, None).brightness(800), 800, "flash off");
    // Protection at factor 512 doubles the flashed value; the mark then scales it, capped.
    assert_eq!(look(2, shielded, None).brightness(400), 1800);
    assert_eq!(
        look(2, shielded, None).brightness(800),
        2000,
        "S-0513 F9's cap"
    );
    assert_eq!(look(0, shielded, Some(Some(256))).brightness(700), 1400);
    assert_eq!(
        look(0, None, Some(Some(512))).brightness(1500),
        2000,
        "capped"
    );
    assert_eq!(
        look(0, None, Some(Some(384))).brightness(-10),
        -15,
        "toward minus infinity"
    );
    assert_eq!(look(0, None, Some(None)).brightness(700), 700);
    assert!(look(1, None, None).active() && !look(0, None, None).active());
    let m = Masks {
        curtain: 0x0001,
        shield: 0x0840,
        laser: 0x1002,
        berserk: 0x2004,
    };
    let curtained = Some(bright(ProtectKind::Curtain));
    assert_eq!(m.building_look(look(0, None, Some(None))), 0x1002);
    assert_eq!(m.building_look(look(0, shielded, Some(None))), 0x1842);
    assert_eq!(m.building_look(look(0, curtained, Some(None))), 0x1002);
    assert_eq!(
        m.building_look(look(3, None, None)),
        0,
        "the flash adds no mask"
    );
    assert_eq!(m.berserk_of(true, MobileKind::VoxelVehicle, false), 0x2004);
    assert_eq!(m.berserk_of(true, MobileKind::SpriteVehicle, false), 0);
    assert_eq!(m.berserk_of(true, MobileKind::Aircraft, false), 0);
    assert_eq!(m.berserk_of(true, MobileKind::Infantry, false), 0x2004);
    assert_eq!(
        m.berserk_of(true, MobileKind::Infantry, true),
        0,
        "cell centre shrouded"
    );
    assert_eq!(m.berserk_of(false, MobileKind::VoxelVehicle, false), 0);
}

/// The rules of a berserk test: berserk selector 1 and the curtain selector 2.
fn berserk_rules() -> String {
    format!(
        "{RULES}[AudioVisual]\nBerserkColor=1\nIronCurtainColor=2\n[ColorAdd]\nA=0,0,0\nB=7,19,3\nC=1,0,1\n"
    )
}

/// A synthetic mobile of `kind` at cell (2, 3).
fn plain_mobile(kind: Kind) -> Mobile {
    Mobile {
        kind,
        idx: 0,
        owner: b"Neutral".to_vec(),
        pos: (2 * 256 + 128, 3 * 256 + 128, 0),
        body: 8192,
        turret: 8192,
        elevation: 0,
        anim: 0,
        infantry: (kind == Kind::Infantry).then_some(InfantryFrame {
            start: 3,
            count: 1,
            multiplier: 2,
            stage: 0,
        }),
        object: sim_code(ObjectId(9)),
        harvesting: None,
        shadow: None,
        alternative: false,
        attitude: Attitude::None,
        disabled: false,
        above: 0,
        protection: None,
        generic_shadow: None,
        berserk: None,
        turret_index: None,
        walk: Default::default(),
    }
}

/// spec: S-0610 claim 7 (the berserk selector's triple becomes the mask by S-0102 F9: a voxel
/// vehicle's body and voxel parts take it, united with the curtain mask when both apply; a
/// sprite vehicle takes none; an infantry sprite body takes it unless the centre of its cell is
/// shrouded for the viewer; the mask changes no brightness; shadows keep their darkening)
#[test]
fn berserk_claim7_rows() {
    use glasnost_render::compose::{PixelLayout, protection_mask};
    use glasnost_sim::superweapons::ProtectKind;
    let berserk = protection_mask([7, 19, 3], PixelLayout::Rgb565);
    let curtain = protection_mask([1, 0, 1], PixelLayout::Rgb565);
    for voxel in [true, false] {
        let (plain, mad, both) =
            with_rules_inputs(voxel, &berserk_rules(), |inputs, painter, scene| {
                let car = |berserk, protection| Mobile {
                    berserk,
                    protection,
                    ..plain_mobile(Kind::Vehicles)
                };
                let curtained = Some(super::protection::Look {
                    kind: ProtectKind::Curtain,
                    factor: None,
                    shrouded: false,
                });
                let (plain, _) = painter.mobiles(inputs, scene, &[car(None, None)]);
                let (mad, _) = painter.mobiles(inputs, scene, &[car(Some(false), None)]);
                let (both, _) = painter.mobiles(inputs, scene, &[car(Some(false), curtained)]);
                (
                    lit_shades(&plain[0].items),
                    lit_shades(&mad[0].items),
                    lit_shades(&both[0].items),
                )
            });
        assert!(
            !plain.is_empty() && plain.len() == mad.len(),
            "voxel {voxel}"
        );
        for ((p, m), b) in plain.iter().zip(&mad).zip(&both) {
            assert_eq!(p.mask, 0);
            assert_eq!(m.mask, if voxel { berserk } else { 0 }, "voxel {voxel}");
            assert_eq!(
                b.mask,
                if voxel { berserk | curtain } else { 0 },
                "voxel {voxel}"
            );
            assert_eq!(m.rho, p.rho, "no brightness change");
        }
    }
    let (plain, mad, hidden) =
        with_rules_inputs(false, &berserk_rules(), |inputs, painter, scene| {
            let man = |berserk| Mobile {
                berserk,
                ..plain_mobile(Kind::Infantry)
            };
            let shades = |painter: &mut super::build::Painter, scene: &mut Scene, b| {
                let (e, _) = painter.mobiles(inputs, scene, &[man(b)]);
                let shadow_paint = e[0]
                    .items
                    .iter()
                    .filter(|i| i.paint == Paint::Shadow)
                    .count();
                (lit_shades(&e[0].items), shadow_paint)
            };
            (
                shades(painter, scene, None),
                shades(painter, scene, Some(false)),
                shades(painter, scene, Some(true)),
            )
        });
    assert_eq!(plain.0.len(), 1);
    assert_eq!(plain.0[0].mask, 0);
    assert_eq!(mad.0[0].mask, berserk);
    assert_eq!(mad.0[0].rho, plain.0[0].rho);
    assert_eq!(
        hidden.0[0].mask, 0,
        "the cell centre shrouded for the viewer"
    );
    assert_eq!(mad.1, plain.1, "shadows unchanged");
}

/// The items `painter` draws for MANX at height 520 with shadow height
/// `hs`, the generic shadow sprite `file` among the assets.
fn generic_shadow_items(
    hs: Option<i32>,
    file: Option<Vec<u8>>,
) -> (Vec<glasnost_render::scene::Item>, (i32, i32)) {
    let (import, catalogue, mut assets) = context();
    if let Some(f) = file {
        assets.insert(super::build::GENERIC_SHADOW_FILE, f);
    }
    let map = decode(
        &Document::parse(&scene_map_text(), Quirks::default(), &mut Vec::new()),
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
    let mut m = plain_mobile(Kind::Infantry);
    m.pos.2 = 520;
    m.above = 520;
    m.generic_shadow = hs;
    let (e, _) = painter.mobiles(&inputs, &mut scene, &[m.clone()]);
    (
        e[0].items.clone(),
        project_lifted(m.pos.0, m.pos.1, m.pos.2),
    )
}

/// spec: S-0107 claim 19 (c, d) (the generic shadow: image frame 1 of the fixed generic
/// infantry-shadow sprite, a darkening, centred L(hₛ) rows below the draw point, drawn right
/// before the body, its size fixed; a file that cannot be found or holds fewer than two image
/// frames draws nothing, and nothing else changes), S-0102 claim 13 (−5 − L(z − hₛ), flat, no
/// depth write) and claim 20
#[test]
fn generic_infantry_shadow_claim19() {
    use glasnost_render::projection::height_lift;
    let file = super::tests::shp((16, 8), 3, (4, 2), (6, 3));
    let (items, (px, py)) = generic_shadow_items(Some(104), Some(file.clone()));
    assert_eq!(
        items.len(),
        2,
        "the generic shadow and the body, no body-sprite shadow"
    );
    let (shadow, body) = (items[0], items[1]);
    assert_eq!(shadow.paint, Paint::Shadow);
    assert_ne!(body.paint, Paint::Shadow);
    // Canvas 16 × 8 centred L(104) = 15 rows below the draw point, frame at (6, 3).
    assert_eq!(
        (shadow.x, shadow.y),
        (px - 8 + 6, py + height_lift(104) - 4 + 3)
    );
    assert_eq!(
        shadow.depth,
        ItemDepth::Profile {
            profile: Profile::Flat,
            bias: -5 - height_lift(520 - 104),
            companion: None,
        }
    );
    assert_eq!(shadow.test, DepthTest::Reader);
    // No shadow height: no generic shadow; a short or missing file: none, the body unchanged.
    let (none, _) = generic_shadow_items(None, Some(file));
    assert_eq!(none.len(), 1);
    let short = super::tests::shp((16, 8), 1, (4, 2), (6, 3));
    let (short, _) = generic_shadow_items(Some(104), Some(short));
    assert_eq!(short.len(), 1);
    let (missing, _) = generic_shadow_items(Some(104), None);
    assert_eq!(missing, none);
}

/// spec: S-0107 claim 19 (b, c) (hₛ = h − 416 over a high-bridge cell when h ≥ 416 and the
/// neighbour one step across the bridge towards lower coordinates is a high-bridge cell too, else
/// h; the generic shadow exactly when hₛ > 0; nothing at h = 0 or h < 0)
#[test]
fn generic_shadow_height_claim19b() {
    use glasnost_sim::cells::Axis;
    use glasnost_sim::rules::Kind as SimKind;
    let (mut live, _) = synthetic_live();
    let man = live
        .world
        .objects
        .values()
        .find(|o| o.kind == SimKind::Infantry && o.on_map)
        .map(|o| o.id)
        .expect("the scenario has infantry");
    let c = live.world.obj(man).unwrap().pos.cell();
    let surface = {
        let p = live.world.obj(man).unwrap().pos;
        live.world.surface_at(p.x, p.y)
    };
    let set_h = |live: &mut Live, h| live.world.obj_mut(man).unwrap().pos.z = surface + h;
    let hs = |live: &Live| super::live::generic_shadow(&live.world, live.world.obj(man).unwrap());
    set_h(&mut live, 0);
    assert_eq!(hs(&live), None, "h = 0: the own shadow instead");
    set_h(&mut live, -20);
    assert_eq!(hs(&live), None, "h < 0: no shadow");
    set_h(&mut live, 300);
    assert_eq!(hs(&live), Some(300), "ordinary terrain: hₛ = h");
    // The unit's cell a high-bridge cell along x, its neighbour (x, y − 1) without a deck.
    let mark = |live: &mut Live, cell: glasnost_map::CellXY, axis| {
        let i = live.world.cells.idx(cell).unwrap();
        live.world.cells.deck[i] = true;
        live.world.cells.bridge_axis[i] = axis;
    };
    mark(&mut live, c, Axis::X);
    set_h(&mut live, 500);
    assert_eq!(
        hs(&live),
        Some(500),
        "the edge row: the shadow falls under the bridge"
    );
    mark(&mut live, glasnost_map::CellXY::new(c.x, c.y - 1), Axis::X);
    assert_eq!(hs(&live), Some(84), "over the deck: h − 416");
    set_h(&mut live, 416);
    assert_eq!(hs(&live), None, "hₛ = 0: none");
    set_h(&mut live, 300);
    assert_eq!(hs(&live), Some(300), "below deck height: h");
    // Along y the neighbour is (x − 1, y), which has no deck.
    let i = live.world.cells.idx(c).unwrap();
    live.world.cells.bridge_axis[i] = Axis::Y;
    set_h(&mut live, 500);
    assert_eq!(hs(&live), Some(500));
}

/// spec: S-0519 F3 (frame = min(11, 2⌊(n − t) / q⌋ + b), q = ⌊D / 6⌋ with 0 taken as 1, b = 1
/// when (n mod 2F) ≥ F, and b = 0 when F = 0)
#[test]
fn bomb_icon_frame_f3() {
    use super::combat::bomb_frame;
    // D = 450: q = 75; F = 5.
    assert_eq!(bomb_frame(100, 100, 450, 5), 0);
    assert_eq!(bomb_frame(105, 100, 450, 5), 1);
    assert_eq!(bomb_frame(172, 100, 450, 5), 0);
    assert_eq!(bomb_frame(182, 100, 450, 5), 2);
    assert_eq!(bomb_frame(186, 100, 450, 5), 3);
    assert_eq!(bomb_frame(10_000, 100, 450, 5), 11);
    // D below 6: q = 1; F = 0: b = 0.
    assert_eq!(bomb_frame(103, 100, 5, 0), 6);
    assert_eq!(bomb_frame(115, 100, 5, 0), 11);
}

/// The overlay assets plus the bomb sprite (12 frames on 10 × 10, frame i
/// a 2 × 2 block of value i + 1 at (4, 4)) and a cursor palette.
fn bomb_assets() -> super::build::Assets {
    let mut assets = overlay_assets();
    assets.insert(
        super::combat::BOMB_SPRITE,
        super::tests::shp((10, 10), 12, (2, 2), (4, 4)),
    );
    let pal: Vec<u8> = (0..768u32).map(|i| (i * 3 % 64) as u8).collect();
    assets.insert(super::combat::CURSOR_PALETTE, pal);
    assets
}

/// What [`bomb_run`] gives.
type BombRun = (
    Vec<glasnost_render::scene::Item>,
    Vec<u8>,
    usize,
    (i32, i32),
    std::collections::BTreeSet<String>,
);

/// A bomb on the combat scene's cart, drawn with `assets` in a fresh game
/// (the drawing caches what it loaded): the cart's status items in
/// painting order, the first pixel of each, the icon's F3 frame, the
/// cart's projected position and the notes.
fn bomb_run(assets: &super::build::Assets) -> BombRun {
    use glasnost_render::scene::Group;
    let (mut live, mut scene, map, import, catalogue, cart, _) =
        combat_live_with(3, assets.clone());
    let tick = live.world.tick;
    live.world.obj_mut(cart).unwrap().combat.bomb = Some(glasnost_sim::bomb::Bomb {
        planter: None,
        house: Some(HouseId(0)),
        planted: (tick - 1).max(0),
    });
    // spec: S-0519 claim 10 (the planter's house sees the bomb)
    live.set_viewer(Some(HouseId(0)));
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets,
        options: &options,
        rules: None,
        cells: None,
    };
    live.set_marks(&[cart], None);
    live.redraw_status(&inputs, &mut scene);
    let status: Vec<_> = scene
        .items
        .iter()
        .filter(|i| i.group == Group::Status && i.object == Some(sim_code(cart)))
        .copied()
        .collect();
    let g = &live.rules.combat.general.ivan;
    let shown = (tick - 1).max(0);
    let frame = super::combat::bomb_frame(shown, shown, g.delay, g.flicker);
    let p = live.world.obj(cart).unwrap().pos;
    let pixels: Vec<u8> = status
        .iter()
        .map(|i| scene.images[i.image as usize].pixels()[0])
        .collect();
    (
        status,
        pixels,
        frame,
        project_lifted(p.x, p.y, p.z),
        live.notes.clone(),
    )
}

/// spec: S-0519 claim 24 (the bomb icon: the F3 image frame of the bomb sprite through the cursor
/// palette, neutral and without remapping; centred on a unit's projected position, height
/// included; the first of the carrier's status extras, with no depth test), S-0519 Import mapping
/// (a missing image: no icon is drawn, and a warning)
#[test]
fn bomb_icon_claim24() {
    let (status, pixels, frame, (ax, ay), notes) = bomb_run(&bomb_assets());
    let icon = status[0];
    // Frame f is a block of value f + 1.
    assert_eq!(pixels[0], frame as u8 + 1);
    assert_eq!((icon.x, icon.y), (ax - 5 + 4, ay - 5 + 4));
    assert_eq!(icon.test, DepthTest::Ignore);
    assert!(status.len() > 1, "the bar and bracket follow the icon");
    match (icon.paint, status[1].paint) {
        // spec: S-0111 claim 2 (the bomb icon and the bracket are both shaded, each through its own palette's table)
        (Paint::Lit(a), Paint::Lit(b)) => {
            assert_ne!(a.base, b.base, "the cursor palette, not the overlay one")
        }
        other => panic!("unexpected paints {other:?}"),
    }
    assert!(notes.is_disjoint(&std::collections::BTreeSet::from([
        super::live::BOMB_ICON_MISSING.to_owned(),
        super::live::CURSOR_PALETTE_MISSING.to_owned()
    ])));
    // Without the bomb sprite: no icon, and the importer's warning (at load, whether or not a bomb
    // shows).
    let mut no_sprite = bomb_assets();
    no_sprite.set(super::combat::BOMB_SPRITE, None);
    let (none, _, _, _, notes) = bomb_run(&no_sprite);
    assert_eq!(none.len(), status.len() - 1);
    assert!(notes.contains(super::live::BOMB_ICON_MISSING));
    assert!(!notes.contains(super::live::CURSOR_PALETTE_MISSING));
}

/// spec: S-0111 claim 19a and Edge cases (missing cursor palette: our engine draws no bomb icon,
/// and the importer warns once, naming the file), S-0111 Import mapping (the bomb sprite's
/// palette is the cursor palette MOUSEPAL.PAL)
#[test]
fn cursor_palette_missing_claim19a() {
    assert_eq!(super::combat::CURSOR_PALETTE, b"MOUSEPAL.PAL");
    assert!(
        super::live::CURSOR_PALETTE_MISSING.contains("MOUSEPAL.PAL"),
        "the warning names the file"
    );
    let (with, ..) = bomb_run(&bomb_assets());
    let mut assets = bomb_assets();
    assets.set(super::combat::CURSOR_PALETTE, None);
    let (status, _, _, _, notes) = bomb_run(&assets);
    // The bar and the bracket only: no icon, and one warning, not the sprite's.
    assert_eq!(status.len(), with.len() - 1);
    assert!(notes.contains(super::live::CURSOR_PALETTE_MISSING));
    assert!(!notes.contains(super::live::BOMB_ICON_MISSING));
    // Given at load, whether or not a bomb shows; the game client's kept copy withdraws it.
    let (mut live, ..) = combat_live_with(3, assets.clone());
    assert!(live.notes.contains(super::live::CURSOR_PALETTE_MISSING));
    live.note_cursor_palette(&bomb_assets());
    assert!(!live.notes.contains(super::live::CURSOR_PALETTE_MISSING));
    live.note_cursor_palette(&assets);
    assert!(live.notes.contains(super::live::CURSOR_PALETTE_MISSING));
}

/// spec: S-0108 Import mapping (ChargedAnimTime: a building rules key, a decimal held in single
/// precision; absent: none here, the default 999 turning claim 18 off) and F6 (widened exactly to
/// binary64)
#[test]
fn charged_anim_time_is_read_from_the_building_section() {
    use glasnost_num::{F64, Rounding};
    let m = Rounding::TowardZero;
    let (import, ..) = super::tests::context_with(
        "[BuildingTypes]\n0=DOME\n1=SHED\n[DOME]\nChargedAnimTime=0.1\n[SHED]\nStrength=5\n",
    );
    let tc = super::live::charged_anim_time(&import, "DOME", m).expect("set");
    // 0.1 in single precision is a little above 0.1 in binary64.
    let tenth = F64::from_i64(1, m).div(F64::from_i64(10, m), m);
    assert!(tenth.lt(tc));
    assert_eq!(
        tc,
        glasnost_num::decimal::parse("0.1", glasnost_num::decimal::Field::Single, m).unwrap()
    );
    assert_eq!(super::live::charged_anim_time(&import, "SHED", m), None);
    assert_eq!(super::live::charged_anim_time(&import, "NONE", m), None);
}

/// spec: S-0111 F4 (p = max(2, ⌊N(14, s) / 4⌋): 28, 14, 9, 7, 5, 4 and 4 for s = 0 … 6; image
/// frame ⌊6 (t mod p) / (p − 1)⌋; with quirks.speed_normalised_delays off, p = 3 at every speed)
#[test]
fn wrench_frame_f4() {
    use super::combat::wrench_frame;
    for (s, p) in [(0u8, 28), (1, 14), (2, 9), (3, 7), (4, 5), (5, 4), (6, 4)] {
        // The period: the frame at t = p is the frame at t = 0, and the last tick of a period
        // shows frame 6.
        assert_eq!(wrench_frame(p, s, true), 0, "speed {s}");
        assert_eq!(wrench_frame(p - 1, s, true), 6, "speed {s}");
        assert_eq!(wrench_frame(0, s, true), 0, "speed {s}");
    }
    // s = 3, p = 7: every frame 0 … 6 in turn.
    let frames: Vec<usize> = (0..7).map(|t| wrench_frame(t, 3, true)).collect();
    assert_eq!(frames, vec![0, 1, 2, 3, 4, 5, 6]);
    // s = 0, p = 28: ⌊6·5 / 27⌋ = 1.
    assert_eq!(wrench_frame(5, 0, true), 1);
    // Off: p = 3, frames 0, 3, 6 at every speed.
    for s in 0..=6u8 {
        let frames: Vec<usize> = (0..6).map(|t| wrench_frame(t, s, false)).collect();
        assert_eq!(frames, vec![0, 3, 6, 0, 3, 6], "speed {s}");
    }
}

/// The overlay assets plus the wrench sprite (7 frames on 10 × 10, frame i a 2 × 2 block of value
/// i + 1 at (4, 4)) and a cursor palette.
fn wrench_assets() -> super::build::Assets {
    let mut assets = overlay_assets();
    assets.insert(
        super::combat::WRENCH_SPRITE,
        super::tests::shp((10, 10), 7, (2, 2), (4, 4)),
    );
    let pal: Vec<u8> = (0..768u32).map(|i| (i * 3 % 64) as u8).collect();
    assets.insert(super::combat::CURSOR_PALETTE, pal);
    assets
}

/// What [`wrench_run`] gives: the building's status items, the first pixel of each, the F4 frame,
/// the building's projected status anchor and the notes.
type WrenchRun = (
    Vec<glasnost_render::scene::Item>,
    Vec<u8>,
    usize,
    (i32, i32),
    std::collections::BTreeSet<String>,
);

/// A tower of house 0 on the combat scene, its repair `repairing`, drawn with `assets` for viewer
/// `viewer` (`None`: no local player); `explore` reveals the tower's cell for the viewer first.
fn wrench_run(
    assets: &super::build::Assets,
    repairing: bool,
    viewer: Option<HouseId>,
    explore: bool,
) -> WrenchRun {
    use glasnost_render::scene::Group;
    use glasnost_sim::rules::Kind as SimKind;
    let (mut live, mut scene, map, import, catalogue, ..) = combat_live_with(3, assets.clone());
    let tower = testkit::spawn(
        &mut live.world,
        SimKind::Building,
        "TOWER",
        0,
        testkit::mid(24, 4, 4),
        0,
    );
    live.world.obj_mut(tower).unwrap().econ.building.repairing = repairing;
    if let (Some(v), true) = (viewer, explore) {
        let pos = live.world.obj(tower).unwrap().pos;
        glasnost_sim::econ::shroud::area_reveal(&mut live.world, v, pos, 3);
    }
    live.set_viewer(viewer);
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets,
        options: &options,
        rules: None,
        cells: None,
    };
    live.set_marks(&[], None);
    live.redraw_status(&inputs, &mut scene);
    let status: Vec<_> = scene
        .items
        .iter()
        .filter(|i| i.group == Group::Status && i.object == Some(sim_code(tower)))
        .copied()
        .collect();
    let shown = (live.world.tick - 1).max(0);
    let frame = super::combat::wrench_frame(
        shown,
        live.world.game_speed,
        live.rules.quirks.speed_normalised_delays,
    );
    let p = live.world.reference_point(tower);
    let pixels: Vec<u8> = status
        .iter()
        .map(|i| scene.images[i.image as usize].pixels()[0])
        .collect();
    (
        status,
        pixels,
        frame,
        project_lifted(p.x, p.y, p.z),
        live.notes.clone(),
    )
}

/// spec: S-0111 claim 19 (a building under repair shows the wrench sprite at its F4 image frame,
/// unless its cell, the top-left cell of its foundation, is under shroud by S-0110 claim 8's point
/// test; drawn through the cursor palette, claim 19a), claim 1 and Terms (a centred image at the
/// status anchor), S-0111 Import mapping (a missing sprite: no wrench is drawn, and a warning)
#[test]
fn repair_wrench_claim19() {
    // Under repair, no local player: the wrench, centred at the status anchor.
    let (status, pixels, frame, (ax, ay), notes) = wrench_run(&wrench_assets(), true, None, false);
    assert_eq!(
        status.len(),
        1,
        "the wrench alone: not selected, no insignia"
    );
    let w = status[0];
    // Frame f is a block of value f + 1.
    assert_eq!(pixels[0], frame as u8 + 1);
    assert_eq!((w.x, w.y), (ax - 5 + 4, ay - 5 + 4));
    assert_eq!(w.test, DepthTest::Ignore);
    assert!(matches!(w.paint, Paint::Lit(_)), "shaded: S-0111 claim 2");
    assert!(!notes.contains(super::live::WRENCH_MISSING));
    // Not under repair: nothing.
    let (none, ..) = wrench_run(&wrench_assets(), false, None, false);
    assert!(none.is_empty());
    // A local player that has not explored the cell: shrouded, no wrench; explored: the wrench.
    let (hidden, ..) = wrench_run(&wrench_assets(), true, Some(HouseId(1)), false);
    assert!(hidden.is_empty(), "shrouded for the local player");
    let (seen, ..) = wrench_run(&wrench_assets(), true, Some(HouseId(1)), true);
    assert_eq!(seen.len(), 1, "explored by the local player");
    // Without the sprite: no wrench, and the importer's warning (at load, repairing or not).
    let mut no_sprite = wrench_assets();
    no_sprite.set(super::combat::WRENCH_SPRITE, None);
    let (missing, _, _, _, notes) = wrench_run(&no_sprite, true, None, false);
    assert!(missing.is_empty());
    assert!(notes.contains(super::live::WRENCH_MISSING));
    assert!(!notes.contains(super::live::CURSOR_PALETTE_MISSING));
    let (_, _, _, _, notes) = wrench_run(&no_sprite, false, None, false);
    assert!(notes.contains(super::live::WRENCH_MISSING));
    // Without the cursor palette: no wrench, and the palette's warning.
    let mut no_pal = wrench_assets();
    no_pal.set(super::combat::CURSOR_PALETTE, None);
    let (missing, _, _, _, notes) = wrench_run(&no_pal, true, None, false);
    assert!(missing.is_empty());
    assert!(notes.contains(super::live::CURSOR_PALETTE_MISSING));
}

/// spec: S-0111 claim 19a, S-0103 claim 1 and Edge cases (the cursor palette's components become
/// (4·c) mod 256, so 64 to 79 wrap to 0 to 60; with quirks.palette_component_wrap off, a component
/// above 63 becomes 252: OV14 palette part e's engine fixture)
#[test]
fn cursor_palette_component_wrap() {
    use super::combat::cursor_palette;
    // Entry n has all three components n mod 80.
    let file: Vec<u8> = (0..256u32).flat_map(|n| [(n % 80) as u8; 3]).collect();
    let on = cursor_palette(&file, &PaletteQuirks::default()).unwrap();
    let off = cursor_palette(
        &file,
        &PaletteQuirks {
            palette_component_wrap: false,
            ..PaletteQuirks::default()
        },
    )
    .unwrap();
    for n in 0..256usize {
        let c = (n % 80) as u8;
        let (a, b) = (on.colors[n], off.colors[n]);
        assert_eq!([a.r, a.g, a.b], [c.wrapping_mul(4); 3], "entry {n}");
        let cut = if c > 63 { 252 } else { 4 * c };
        assert_eq!([b.r, b.g, b.b], [cut; 3], "entry {n}");
    }
    // 64 wraps to 0 and 79 to 60; 63 gives 252 in both settings.
    assert_eq!((on.colors[64].r, off.colors[64].r), (0, 252));
    assert_eq!((on.colors[79].r, off.colors[79].r), (60, 252));
    assert_eq!((on.colors[63].r, off.colors[63].r), (252, 252));
    assert!(cursor_palette(b"short", &PaletteQuirks::default()).is_none());
}

/// The game font of the control-group number tests: `combat::digit_font`
/// with bitmap height 7 and line pitch 10, so a number's image spans its
/// box from 3 pixels left of the placement point and its glyph's top row
/// at the point.
fn number_font() -> Option<super::combat::GroupFont> {
    super::combat::GroupFont::new(&super::combat::digit_font(7, 10))
}

/// The control-group number items of object `id` in `scene` (the images
/// holding the box's black fill, index 2), with their placement point (the
/// image's left edge plus 3) and the colours their row gives the house
/// colour (index 1) and the fill.
fn group_numbers(scene: &Scene, id: ObjectId) -> Vec<((i32, i32), u16, u16)> {
    use glasnost_render::scene::Group;
    scene
        .items
        .iter()
        .filter(|i| i.group == Group::Status && i.object == Some(sim_code(id)))
        .filter(|i| {
            let img = &scene.images[i.image as usize];
            img.height() == 8 && img.pixels().contains(&super::combat::GROUP_FILL)
        })
        .map(|i| {
            let row = match i.paint {
                Paint::Row(r) => &scene.rows[r],
                _ => panic!("the group number is drawn through a row"),
            };
            ((i.x + 3, i.y), row[1], row[2])
        })
        .collect()
}

/// spec: S-0111 claim 17 and scenario OV12 (the group digit at (−14, −26)
/// from a vehicle's anchor and (−14, −29) from an infantry unit's, in the
/// owner's house colour), claims 3 and 9 and State and ordering (drawn with
/// the pip scale: only with the health bar, only for an owner the viewer
/// may inspect, after the object's other status extras); the glyph's top
/// row and left column at the point, the box 3 pixels to its left, in the
/// house display RGB of S-0103 claim 15 with a black fill (OV12 a and b).
#[test]
fn the_group_number_follows_the_pip_scale() {
    use glasnost_render::compose::pack565;
    use glasnost_render::scene::Group;
    let assets = overlay_assets();
    let (mut live, mut scene, map, import, catalogue, cart, walker) =
        combat_live_with(3, assets.clone());
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
    let red = [200, 40, 40];
    let blue = [40, 40, 200];
    let px = |c: [u8; 3]| {
        pack565(glasnost_fmt::image::Rgb8 {
            r: c[0],
            g: c[1],
            b: c[2],
        })
    };
    live.set_house_colours(vec![red, blue]);
    live.set_group_font(number_font());
    live.set_viewer(Some(HouseId(0)));
    // spec: S-0111 claim 17 (keyboard groups 5 and 10: index 5 shows 5, index 0 shows 0)
    let digits: std::collections::BTreeMap<_, _> = [
        (cart, super::combat::group_digit(5)),
        (walker, super::combat::group_digit(0)),
    ]
    .into();
    assert!(live.set_group_digits(digits.clone()));
    assert!(!live.set_group_digits(digits));
    let anchor = |l: &Live, id: ObjectId| {
        let p = l.world.reference_point(id);
        project_lifted(p.x, p.y, p.z)
    };
    // Neither selected nor hovered: no pip scale, so no number.
    live.set_marks(&[], None);
    live.redraw_status(&inputs, &mut scene);
    assert!(group_numbers(&scene, cart).is_empty());
    // The cart selected: its number at (−14, −26), in its owner's colour, last of its extras.
    live.set_marks(&[cart], None);
    live.redraw_status(&inputs, &mut scene);
    let (ax, ay) = anchor(&live, cart);
    assert_eq!(
        group_numbers(&scene, cart),
        vec![((ax - 14, ay - 26), px(red), 0)]
    );
    let last = scene
        .items
        .iter()
        .rfind(|i| i.group == Group::Status && i.object == Some(sim_code(cart)))
        .unwrap();
    assert_eq!(
        (last.x, last.y),
        (ax - 14 - 3, ay - 26),
        "it covers the bar"
    );
    // The enemy walker hovered: a health bar but no pip scale (claim 9), so no number.
    live.set_marks(&[cart], Some(walker));
    live.redraw_status(&inputs, &mut scene);
    assert!(group_numbers(&scene, walker).is_empty());
    // Seen as its owner: the infantry place (−14, −29), in its owner's colour.
    live.set_viewer(Some(HouseId(1)));
    live.redraw_status(&inputs, &mut scene);
    let (wx, wy) = anchor(&live, walker);
    assert_eq!(
        group_numbers(&scene, walker),
        vec![((wx - 14, wy - 29), px(blue), 0)]
    );
    assert!(
        group_numbers(&scene, cart).is_empty(),
        "the cart is now the enemy's"
    );
}

/// spec: S-0111 claims 10, 11 and 17, Edge cases
/// (`quirks.building_resource_pips_hidden`; claim 9's draw-for-all mark):
/// a building's number at (−4, −36) from its pip-scale origin P(−α, β, 0);
/// a building of the resource pip-scale kind shows none with the switch on.
#[test]
fn a_building_group_number_and_the_resource_switch() {
    use glasnost_sim::rules::Kind as SimKind;
    let assets = overlay_assets();
    let rules = format!(
        "{}[TOWER]\nPipScale=Tiberium\nPipsDrawForAll=yes\n",
        RULES.replace(
            "[BuildingTypes]\n1=HUTX\n",
            "[BuildingTypes]\n1=HUTX\n2=TOWER\n"
        )
    );
    assert!(rules.contains("2=TOWER"));
    let (import, catalogue, _) = context_with(&rules);
    assert!(
        import.outcome.registries[&Kind::Buildings]
            .iter()
            .any(|e| e.keys.contains_key(b"PipScale".as_slice())),
        "the import holds the tower's pip-scale kind"
    );
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
    let own = testkit::spawn(
        &mut world,
        SimKind::Building,
        "TOWER",
        0,
        testkit::mid(24, -3, 0),
        0,
    );
    let theirs = testkit::spawn(
        &mut world,
        SimKind::Building,
        "TOWER",
        1,
        testkit::mid(24, 3, 0),
        0,
    );
    let rules = world.rules_arc();
    let mut live = Live::new(
        &inputs,
        &mut built,
        painter,
        (world, rules, testkit::setup(3), Vec::new()),
    );
    let mut scene = built.scene;
    live.set_house_colours(vec![[200, 40, 40], [40, 40, 200]]);
    live.set_group_font(number_font());
    live.set_viewer(Some(HouseId(0)));
    live.set_group_digits([(own, 1), (theirs, 5)].into());
    live.set_marks(&[own], Some(theirs));
    // Switch on (the original): a resource-kind building shows no number.
    live.redraw_status(&inputs, &mut scene);
    assert!(group_numbers(&scene, own).is_empty());
    assert!(group_numbers(&scene, theirs).is_empty());
    // Switch off: the number at o + (−4, −36); the enemy's through its draw-for-all mark.
    live.set_view_quirks(super::quirks::ViewQuirks {
        building_resource_pips_hidden: false,
        ..Default::default()
    });
    live.redraw_status(&inputs, &mut scene);
    for id in [own, theirs] {
        let (w, d) = live.world.obj(id).unwrap().foundation;
        let p = live.world.reference_point(id);
        let (ax, ay) = project_lifted(p.x, p.y, p.z);
        // spec: S-0111 claim 10 (whole-cell foundations: o = (−15(W + D), trunc(15(D − W) / 2)))
        let o = (-15 * (w + d), 15 * (d - w) / 2);
        let at = group_numbers(&scene, id);
        assert_eq!(at.len(), 1, "one number for {id:?}");
        assert_eq!(at[0].0, (ax + o.0 - 4, ay + o.1 - 36));
    }
}

/// Two grouped carts of house 0 one cell apart along the map's *y* axis,
/// the first created at the larger sort key, in a live game of the
/// synthetic combat ruleset. Returns the game, scene, map, import,
/// catalogue and the two carts (lower key first).
#[allow(clippy::type_complexity)]
fn two_carts(
    assets: &super::build::Assets,
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
    let map = testkit::empty_map(24);
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (mut built, painter) = build_parts(&inputs, true);
    let mut world = testkit::game_with(&testkit::combat_ruleset(), 24, 3);
    let later = testkit::spawn(
        &mut world,
        SimKind::Vehicle,
        "CART",
        0,
        testkit::mid(24, 0, 1),
        0,
    );
    let earlier = testkit::spawn(
        &mut world,
        SimKind::Vehicle,
        "CART",
        0,
        testkit::mid(24, 0, 0),
        0,
    );
    assert!(
        later < earlier,
        "object ID order is the reverse of the key order"
    );
    let rules = world.rules_arc();
    let live = Live::new(
        &inputs,
        &mut built,
        painter,
        (world, rules, testkit::setup(3), Vec::new()),
    );
    (live, built.scene, map, import, catalogue, earlier, later)
}

/// spec: S-0111 claim 17, State and ordering and scenario OV12 e (each object's status extras,
/// its number last, are complete before the next object's begin, in the object order of S-0102
/// claims 1 and 6: the ground layer by increasing sort key, not object ID order, so a later
/// object's bar covers an earlier object's number); Import mapping (without a usable game font,
/// S-0014 claim 6, no number and no box)
#[test]
fn group_numbers_follow_the_object_order() {
    use glasnost_render::scene::Group;
    let assets = overlay_assets();
    let (mut live, mut scene, map, import, catalogue, earlier, later) = two_carts(&assets);
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
    live.set_house_colours(vec![[200, 40, 40], [40, 40, 200]]);
    live.set_group_font(number_font());
    live.set_viewer(Some(HouseId(0)));
    live.set_group_digits([(earlier, 1), (later, 1)].into());
    live.set_marks(&[later, earlier], None);
    live.redraw(&inputs, &mut scene);
    let owner =
        |i: &glasnost_render::scene::Item| (i.group == Group::Status).then_some(i.object).flatten();
    let codes: Vec<u32> = scene.items.iter().filter_map(owner).collect();
    let (e, l) = (sim_code(earlier), sim_code(later));
    let last_of_earlier = codes.iter().rposition(|&c| c == e).unwrap();
    let first_of_later = codes.iter().position(|&c| c == l).unwrap();
    assert!(
        last_of_earlier < first_of_later,
        "the earlier object's extras are complete before the later one's begin"
    );
    // The earlier cart's number is the last of its own extras.
    let earlier_items: Vec<_> = scene
        .items
        .iter()
        .filter(|i| i.group == Group::Status && i.object == Some(e))
        .collect();
    let last = earlier_items.last().unwrap();
    assert!(
        scene.images[last.image as usize]
            .pixels()
            .contains(&super::combat::GROUP_FILL)
    );
    assert_eq!(group_numbers(&scene, earlier).len(), 1);
    assert_eq!(group_numbers(&scene, later).len(), 1);
    // Without a font: neither digit nor box.
    live.set_group_font(None);
    live.redraw_status(&inputs, &mut scene);
    assert!(group_numbers(&scene, earlier).is_empty());
    assert!(
        scene
            .items
            .iter()
            .filter(|i| i.group == Group::Status && i.object == Some(e))
            .all(|i| !scene.images[i.image as usize]
                .pixels()
                .contains(&super::combat::GROUP_FILL)),
        "no box"
    );
}

/// The status items of `id` drawn from the pip sprites, in draw order:
/// for each its pip centre (the pip sprite's test frames sit at (3, 2) and
/// the second's at (2, 2) on an 8 × 6 canvas), whether it is from the
/// second pip sprite, its image frame and whether it is a shadow.
fn pip_scale_items(scene: &Scene, id: ObjectId) -> Vec<((i32, i32), bool, usize, bool)> {
    use glasnost_render::scene::Group;
    scene
        .items
        .iter()
        .filter(|i| i.group == Group::Status && i.object == Some(sim_code(id)))
        .filter_map(|i| {
            let img = &scene.images[i.image as usize];
            let frame = usize::from(img.pixels()[0]).saturating_sub(1);
            let shadow = i.paint == Paint::Shadow;
            match (img.width(), img.height()) {
                (2, 2) => Some(((i.x + 1, i.y + 1), false, frame, shadow)),
                (4, 1) => Some(((i.x + 2, i.y + 1), true, frame, shadow)),
                _ => None,
            }
        })
        .collect()
}

/// A live game of the synthetic combat ruleset with a cart of house 0 able
/// to carry 3 passengers (pip-scale kind passengers in the import) and a
/// walker of house 0 with 4 rounds of ammunition (pip-scale kind ammo,
/// passenger pip colour yellow). Returns the live game, the scene, the map,
/// import and catalogue, and the two units.
#[allow(clippy::type_complexity)]
fn pip_scale_live(
    assets: &super::build::Assets,
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
    let rules = format!(
        "{}[CART]\nPipScale=Passengers\n[WALKER]\nPipScale=ammo\nPip=Yellow\n",
        RULES
            .replace(
                "[InfantryTypes]\n1=MANX\n",
                "[InfantryTypes]\n1=MANX\n2=WALKER\n"
            )
            .replace(
                "[VehicleTypes]\n1=CARX\n",
                "[VehicleTypes]\n1=CARX\n2=CART\n"
            )
    );
    assert!(rules.contains("2=WALKER") && rules.contains("2=CART"));
    let (import, catalogue, _) = context_with(&rules);
    let map = testkit::empty_map(24);
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (mut built, painter) = build_parts(&inputs, true);
    let mut ruleset = testkit::combat_ruleset();
    let cart_type = ruleset.sim.vehicles.get_mut("CART").unwrap();
    cart_type.passengers = Some(3);
    let walker_type = ruleset.sim.infantry.get_mut("WALKER").unwrap();
    walker_type.combat.ammo = Some(4);
    let mut world = testkit::game_with(&ruleset, 24, 3);
    let cart = testkit::spawn(
        &mut world,
        SimKind::Vehicle,
        "CART",
        0,
        testkit::mid(24, -3, 0),
        16384,
    );
    let walker = testkit::spawn(
        &mut world,
        SimKind::Infantry,
        "WALKER",
        0,
        testkit::mid(24, 3, 0),
        16384,
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

/// spec: S-0111 claims 10, 11 and 12 (a transport's passenger row from the
/// second pip sprite at o + (−5 + 4j, 0), the infantry passenger's pip
/// colour, the empty pips 0), claim 15 (ammunition pips 3 pixels higher,
/// image frame 13 while j < a), claim 16 (the unit-repair marker at
/// (33, −32) from o, from the pip sprite; a shadow while damaged in the
/// first 6 ticks of the period), State and ordering (the pip scale after
/// the health bar)
#[test]
fn pip_rows_and_the_self_heal_marker() {
    let assets = overlay_assets();
    let (mut live, mut scene, map, import, catalogue, cart, walker) = pip_scale_live(&assets);
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
    let anchor = |l: &Live, id: ObjectId| {
        let p = l.world.reference_point(id);
        project_lifted(p.x, p.y, p.z)
    };
    live.set_viewer(Some(HouseId(0)));
    live.world.obj_mut(cart).unwrap().transport.passengers = vec![walker];
    live.world.obj_mut(walker).unwrap().combat.ammo = 2;
    live.set_marks(&[cart, walker], None);
    live.redraw_status(&inputs, &mut scene);
    let (ax, ay) = anchor(&live, cart);
    // o = (−10, 10): pip j at (−15 + 4j, 10); the walker's yellow pip (2) first.
    assert_eq!(
        pip_scale_items(&scene, cart)
            .into_iter()
            .filter(|p| p.1)
            .collect::<Vec<_>>(),
        vec![
            ((ax - 15, ay + 10), true, 2, false),
            ((ax - 11, ay + 10), true, 0, false),
            ((ax - 7, ay + 10), true, 0, false),
        ]
    );
    // The infantry row at o + (6 + 4j, 0), the ammunition 3 pixels higher.
    let (bx, by) = anchor(&live, walker);
    assert_eq!(
        pip_scale_items(&scene, walker)
            .into_iter()
            .filter(|p| p.1)
            .collect::<Vec<_>>(),
        vec![
            ((bx - 4, by + 7), true, 13, false),
            ((bx, by + 7), true, 13, false),
        ]
    );
    // No self-heal ability yet: no marker. The owner gains unit repair.
    assert!(!pip_scale_items(&scene, cart).iter().any(|p| p.2 == 20));
    live.world.houses[0].combat.self_heal[1] = 1;
    live.redraw_status(&inputs, &mut scene);
    let items = pip_scale_items(&scene, cart);
    let marker = *items.last().unwrap();
    assert_eq!(marker, ((ax + 23, ay - 22), false, 20, false));
    // Drawn after the health bar's segments.
    let first_segment = items.iter().position(|p| p.2 == 16).unwrap();
    let first_row = items.iter().position(|p| p.1).unwrap();
    assert!(first_segment < first_row);
    // The walker (infantry) has no infantry healing: no marker.
    assert!(
        !pip_scale_items(&scene, walker)
            .iter()
            .any(|p| p.2 == 13 && !p.1)
    );
    // Damaged at a tick whose remainder by 1000 is at most 5: a shadow.
    let strength = live.world.obj(cart).unwrap().health;
    live.world.obj_mut(cart).unwrap().health = strength - 1;
    live.redraw_status(&inputs, &mut scene);
    let marker = *pip_scale_items(&scene, cart).last().unwrap();
    assert_eq!(marker, ((ax + 23, ay - 22), false, 20, true));
    // An enemy's pip scale is not shown: no rows, no marker.
    live.set_viewer(Some(HouseId(1)));
    live.redraw_status(&inputs, &mut scene);
    assert!(
        !pip_scale_items(&scene, cart)
            .iter()
            .any(|p| p.1 || p.2 == 20)
    );
}

/// spec: S-0111 Import mapping (a missing pip sprite: the rows drawn from it
/// are left out, the other sprite's items stay, and a note is given)
#[test]
fn a_missing_pip_sprite_leaves_its_rows_out() {
    for missing in [super::combat::PIP2_SPRITE, super::combat::PIP_SPRITE] {
        let mut assets = overlay_assets();
        assets.set(missing, None);
        let (mut live, mut scene, map, import, catalogue, cart, walker) = pip_scale_live(&assets);
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
        live.set_viewer(Some(HouseId(0)));
        live.world.obj_mut(cart).unwrap().transport.passengers = vec![walker];
        live.world.obj_mut(walker).unwrap().combat.ammo = 2;
        live.world.houses[0].combat.self_heal[1] = 1;
        live.set_marks(&[cart, walker], None);
        live.redraw_status(&inputs, &mut scene);
        let items: Vec<_> = [cart, walker]
            .into_iter()
            .flat_map(|id| pip_scale_items(&scene, id))
            .collect();
        if missing == super::combat::PIP2_SPRITE {
            assert!(live.notes.contains(super::live::PIPS2_MISSING));
            assert!(!live.notes.contains(super::live::PIPS_MISSING));
            assert!(!items.iter().any(|p| p.1), "no PIPS2.SHP rows");
            assert!(items.iter().any(|p| p.2 == 20), "the marker stays");
        } else {
            assert!(live.notes.contains(super::live::PIPS_MISSING));
            assert!(!live.notes.contains(super::live::PIPS2_MISSING));
            assert!(items.iter().all(|p| p.1), "nothing from PIPS.SHP");
            assert!(items.iter().any(|p| p.1), "the PIPS2.SHP rows stay");
        }
    }
}

/// spec: S-0111 claims 10 and 11 (a building's occupant row from the pip
/// sprite at o + (6 + 4j, −1 + 2j): the occupant's occupant pip image
/// frame, 6 for an empty slot; ShowOccupantPips off hides the row)
#[test]
fn a_building_occupant_row() {
    use glasnost_sim::rules::Kind as SimKind;
    let assets = overlay_assets();
    for show in [true, false] {
        let rules = format!(
            "{}[WALKER]\nOccupyPip=PersonRed\n[TOWER]\nShowOccupantPips={}\n",
            RULES
                .replace(
                    "[InfantryTypes]\n1=MANX\n",
                    "[InfantryTypes]\n1=MANX\n2=WALKER\n"
                )
                .replace(
                    "[BuildingTypes]\n1=HUTX\n",
                    "[BuildingTypes]\n1=HUTX\n2=TOWER\n"
                ),
            if show { "yes" } else { "no" }
        );
        let (import, catalogue, _) = context_with(&rules);
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
        let mut ruleset = testkit::combat_ruleset();
        ruleset
            .sim
            .buildings
            .get_mut("TOWER")
            .unwrap()
            .max_number_occupants = Some(2);
        let mut world = testkit::game_with(&ruleset, 24, 3);
        let tower = testkit::spawn(
            &mut world,
            SimKind::Building,
            "TOWER",
            0,
            testkit::mid(24, -3, 0),
            0,
        );
        let walker = testkit::spawn(
            &mut world,
            SimKind::Infantry,
            "WALKER",
            0,
            testkit::mid(24, 3, 0),
            0,
        );
        world.obj_mut(tower).unwrap().structure.occupants = vec![walker];
        let rules = world.rules_arc();
        let mut live = Live::new(
            &inputs,
            &mut built,
            painter,
            (world, rules, testkit::setup(3), Vec::new()),
        );
        let mut scene = built.scene;
        live.set_viewer(Some(HouseId(0)));
        live.set_marks(&[tower], None);
        live.redraw_status(&inputs, &mut scene);
        let p = live.world.reference_point(tower);
        let (ax, ay) = project_lifted(p.x, p.y, p.z);
        let (w, d) = live.world.obj(tower).unwrap().foundation;
        // spec: S-0111 claim 10 (whole-cell foundations: o = (−15(W + D), trunc(15(D − W) / 2)))
        let o = (ax - 15 * (w + d), ay + 15 * (d - w) / 2);
        let row: Vec<_> = pip_scale_items(&scene, tower)
            .into_iter()
            .filter(|p| p.2 == 10 || p.2 == 6)
            .collect();
        if show {
            assert_eq!(
                row,
                vec![
                    ((o.0 + 6, o.1 - 1), false, 10, false),
                    ((o.0 + 10, o.1 + 1), false, 6, false),
                ]
            );
        } else {
            assert!(row.is_empty());
        }
    }
}

/// Draws one scene item's first pixel into a 1 x 1 frame at light-alpha `alpha` and returns the
/// 5-6-5 colour that results.
fn shroud_pixel(scene: &Scene, item: &glasnost_render::scene::Item, alpha: u8) -> u16 {
    use glasnost_render::compose::{ComposeQuirks, DepthSource, Drawing, Frame};
    let ItemDepth::Profile {
        profile,
        bias,
        companion: _,
    } = item.depth
    else {
        panic!("a flat profile");
    };
    let img = &scene.images[item.image as usize];
    let first = img.pixels().iter().copied().find(|&p| p != 0).unwrap();
    let one = glasnost_fmt::image::IndexedImage::new(1, 1, vec![first]).unwrap();
    let mut f = Frame::new(1, 1, 0, 1000);
    f.set_light_alpha(vec![alpha]);
    f.draw(
        &Drawing {
            image: &one,
            x: 0,
            y: 0,
            test: item.test,
            depth: DepthSource::Profile {
                profile,
                bias,
                companion: None,
            },
            paint: item.paint,
        },
        &scene.rows,
        &ComposeQuirks::default(),
    );
    f.color[0]
}

/// spec: S-0111 claim 1 and scenario OV1's shape (status extras are drawn while the status anchor
/// lies within the battlefield view widened by 360 pixels left and right and 180 above and below,
/// equality included, and vanish one pixel beyond; at each of the four edges; no view known: all)
#[test]
fn claim_1_culling_margins() {
    use glasnost_render::scene::Group;
    let assets = overlay_assets();
    let (mut live, mut scene, map, import, catalogue, cart, _walker) =
        combat_live_with(3, assets.clone());
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
    let p = live.world.reference_point(cart);
    let (ax, ay) = project_lifted(p.x, p.y, p.z);
    let (vw, vh) = (200u32, 100u32);
    live.set_marks(&[cart], None);
    let count = |live: &mut Live, scene: &mut Scene, origin: (i32, i32)| {
        live.battlefield = Some((origin, (vw, vh)));
        live.redraw_status(&inputs, scene);
        scene
            .items
            .iter()
            .filter(|i| i.group == Group::Status && i.object == Some(sim_code(cart)))
            .count()
    };
    live.battlefield = None;
    live.redraw_status(&inputs, &mut scene);
    let all = scene
        .items
        .iter()
        .filter(|i| i.group == Group::Status && i.object == Some(sim_code(cart)))
        .count();
    assert!(all > 1, "the bracket and the segments");
    // Left: the anchor 360 left of the view (origin 360 to its right), then 361.
    assert_eq!(count(&mut live, &mut scene, (ax + 360, ay)), all);
    assert_eq!(count(&mut live, &mut scene, (ax + 361, ay)), 0);
    // Right: the view's right edge is at origin + 200; the anchor 360 beyond it.
    assert_eq!(count(&mut live, &mut scene, (ax - 200 - 360, ay)), all);
    assert_eq!(count(&mut live, &mut scene, (ax - 200 - 361, ay)), 0);
    // Above and below: 180.
    assert_eq!(count(&mut live, &mut scene, (ax, ay + 180)), all);
    assert_eq!(count(&mut live, &mut scene, (ax, ay + 181)), 0);
    assert_eq!(count(&mut live, &mut scene, (ax, ay - 100 - 180)), all);
    assert_eq!(count(&mut live, &mut scene, (ax, ay - 100 - 181)), 0);
    // The cull is on the anchor alone: the view origin moving back draws them again.
    assert_eq!(count(&mut live, &mut scene, (ax, ay)), all);
}

/// spec: S-0111 claim 2 and scenario OV2's shape (the bracket image and the insignia take the
/// light-alpha shade at neutral brightness, so a shroud piece darkens them; health segments are
/// not shaded and keep their palette colour under it)
#[test]
fn claim_2_shading_of_the_bracket_and_the_insignia() {
    use glasnost_render::scene::Group;
    let assets = overlay_assets();
    let (mut live, mut scene, map, import, catalogue, cart, _walker) =
        combat_live_with(3, assets.clone());
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
    let two = glasnost_num::F64::from_i64(2, glasnost_num::Rounding::TowardZero)
        .to_f32(glasnost_num::Rounding::TowardZero);
    live.world.obj_mut(cart).unwrap().combat.experience = two;
    live.set_marks(&[cart], None);
    live.redraw_status(&inputs, &mut scene);
    let items: Vec<_> = scene
        .items
        .iter()
        .filter(|i| i.group == Group::Status && i.object == Some(sim_code(cart)))
        .copied()
        .collect();
    // The elite insignia is frame 15 (pixel 16), the bracket frame 0 (pixel 1).
    let find = |v: u8| {
        items
            .iter()
            .find(|i| scene.images[i.image as usize].pixels()[0] == v)
            .copied()
            .unwrap()
    };
    let (insignia, bracket) = (find(16), find(1));
    let segment = items
        .iter()
        .find(|i| matches!(i.paint, Paint::Row(_)))
        .copied()
        .unwrap();
    for shaded in [insignia, bracket] {
        let Paint::Lit(s) = shaded.paint else {
            panic!("shaded by the light-alpha values");
        };
        // Neutral brightness: rho for 1000 per mille, whatever the map's lighting.
        assert_eq!(s.rho, glasnost_render::light::brightness_rho(1000) as u16);
        let lit = shroud_pixel(&scene, &shaded, 127);
        let dark = shroud_pixel(&scene, &shaded, 0);
        assert_ne!(lit, 0, "visible outside the shroud");
        assert_eq!(dark, 0, "fully shrouded: black");
    }
    let open = shroud_pixel(&scene, &segment, 127);
    let covered = shroud_pixel(&scene, &segment, 0);
    assert_ne!(open, 0);
    assert_eq!(
        open, covered,
        "segments keep their palette colour under shroud"
    );
}

/// spec: S-0111 claim 8 (a selected building gets corner brackets instead of a bracket image: thin
/// lines of overlay-palette index 15, the box's corners, a line along each edge for a quarter of
/// its length rounded toward zero in leptons, projected as displacements)
#[test]
fn claim_8_corner_brackets() {
    use super::combat::{corner_image, corner_segments, displacement};
    // A 2 x 3 foundation, box 208 leptons high: alpha = 256, beta = 384; quarters 128, 192, 52.
    let segs = corner_segments(2, 3, 208);
    // The top face's left corner (-256, 384, 208) runs along x toward +x ...
    let from = displacement(-256, 384, 208);
    assert!(segs.contains(&(from, displacement(-256 + 128, 384, 208))));
    // ... along y toward -y, and down the vertical edge by 208 / 4 = 52 leptons.
    assert!(segs.contains(&(from, displacement(-256, 384 - 192, 208))));
    assert!(segs.contains(&(from, displacement(-256, 384, 208 - 52))));
    // A base corner's vertical line runs up.
    let base_front = displacement(256, 384, 0);
    assert!(segs.contains(&(base_front, displacement(256, 384, 52))));
    // The quarter rounds toward zero: 105 / 4 = 26.
    let odd = corner_segments(1, 1, 105);
    let top = displacement(-128, -128, 105);
    assert!(odd.contains(&(top, displacement(-128, -128, 105 - 26))));
    // The image holds only the requested index, and its pixels include the segments' ends.
    let (img, (ox, oy)) = corner_image(2, 3, 208, 15).unwrap();
    assert!(img.pixels().iter().all(|&p| p == 0 || p == 15));
    for &(a, b) in &segs {
        for (x, y) in [a, b] {
            assert_eq!(img.get((x - ox) as u32, (y - oy) as u32), Some(15));
        }
    }
}

/// spec: S-0111 claims 8 and 2 in the live view: a selected tower shows its corner brackets
/// (index 15, shaded) and no bracket image; a hovered one shows none; units keep their bracket image
#[test]
fn claim_8_a_selected_building_in_the_live_view() {
    use glasnost_render::scene::Group;
    use glasnost_sim::rules::Kind as SimKind;
    let assets = overlay_assets();
    let (mut live, mut scene, map, import, catalogue, cart, _walker) =
        combat_live_with(3, assets.clone());
    let tower = testkit::spawn(
        &mut live.world,
        SimKind::Building,
        "TOWER",
        0,
        testkit::mid(24, 4, 4),
        0,
    );
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
    assert!(live.bracket_drawn(&assets, tower));
    assert!(live.bracket_drawn(&assets, cart));
    let mut bare = assets.clone();
    bare.set(super::combat::OVERLAY_PALETTE, None);
    assert!(!live.bracket_drawn(&bare, tower));
    let mine = |scene: &Scene| -> Vec<_> {
        scene
            .items
            .iter()
            .filter(|i| i.group == Group::Status && i.object == Some(sim_code(tower)))
            .copied()
            .collect()
    };
    live.set_marks(&[], Some(tower));
    live.redraw_status(&inputs, &mut scene);
    assert!(
        mine(&scene)
            .iter()
            .all(|i| !scene.images[i.image as usize].pixels().contains(&15)),
        "hovered: no corner brackets"
    );
    live.set_marks(&[tower], None);
    live.redraw_status(&inputs, &mut scene);
    let corners: Vec<_> = mine(&scene)
        .into_iter()
        .filter(|i| scene.images[i.image as usize].pixels().contains(&15))
        .collect();
    assert_eq!(corners.len(), 1);
    assert!(matches!(corners[0].paint, Paint::Lit(_)), "shaded");
    assert_ne!(shroud_pixel(&scene, &corners[0], 127), 0);
    assert_eq!(shroud_pixel(&scene, &corners[0], 0), 0, "shroud darkens it");
    // The corners lie around the status anchor: the image spans it.
    let p = live.world.reference_point(tower);
    let (ax, _) = project_lifted(p.x, p.y, p.z);
    let w = scene.images[corners[0].image as usize].width() as i32;
    assert!(corners[0].x < ax && ax < corners[0].x + w);
    // The unit keeps its bracket image (frame 34 wide).
    live.set_marks(&[cart], None);
    live.redraw_status(&inputs, &mut scene);
    assert!(
        scene
            .items
            .iter()
            .any(|i| i.object == Some(sim_code(cart))
                && scene.images[i.image as usize].width() == 34)
    );
}
