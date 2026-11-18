//! The object under a still pointer over a flying, firing jumpjet infantry
//! unit above a row of trees (#435): which drawn item gives the pick in each
//! drawn frame, and why it changes. Invented types, images and map, no game
//! data.

use std::collections::BTreeSet;
use std::sync::Arc;

use glasnost_map::CellXY;
use glasnost_map::objects::TerrainObject;
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE, Paint};
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::scene::{Filter, Group, Scene, View};
use glasnost_rules::sim::{Decimal, LocomotionClass, Locomotor, MovementZone, UnitType};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind as SimKind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Target};

use super::build::{Inputs, Options, build_parts};
use super::live::{Live, sim_code};
use super::tests::{ART, RULES, context_full};
use super::tree_depth_tests::shape;

const SIZE: i32 = 24;

/// The simulation's rules: the synthetic combat ruleset with MANX, a
/// jumpjet infantry type (cruise height 500) armed with the ruleset's gun,
/// strong enough to outlive the fight.
fn rules() -> Arc<SimRules> {
    let mut r = testkit::combat_ruleset();
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("MANX".into());
    r.sim.infantry.insert(
        "MANX".into(),
        UnitType {
            strength: Some(10_000),
            speed: Some(4),
            locomotion_class: Some(LocomotionClass::Foot),
            movement_zone: Some(MovementZone::Fly),
            locomotor: Some(Locomotor::Jumpjet),
            jumpjet: Some(true),
            jumpjet_speed: Some(10),
            jumpjet_climb: Some(Decimal("5.0".into())),
            jumpjet_height: Some(500),
            primary: Some("Gun".into()),
            ..Default::default()
        },
    );
    Arc::new(SimRules::new(&r, "synthetic-test"))
}

/// A drawn item's image, place and size.
type Body = (u32, (i32, i32), (u32, u32));

/// What the pick at the still pointer found in one drawn frame.
#[derive(Debug)]
struct Seen {
    tick: i32,
    /// The unit's height and its body item (image, place, size).
    z: i32,
    body: Option<Body>,
    /// The pick: the unit, a tree, nothing (the ground), or another code.
    pick: &'static str,
    /// The items over the pointer, in draw order: group and object code.
    over: Vec<(Group, Option<u32>)>,
}

/// One run: Alpha's MANX on cell C attacks Beta's CART on C + (10, 0) over
/// a row of trees on C to C + (9, 0), each `tree` (width, height) from its
/// image's place in a 80 × 160 canvas. The unit's sprite has 40 frames of
/// 12 × 20, each of one value, so its body is opaque wherever it lies. Once
/// the unit has reached 400 and holds still (it hovers and fires), the
/// pointer is held on the middle of its body's upper part; from then on each
/// drawn frame's pick at the pointer is recorded. Also gives the unit's
/// (tick, height) changes and the target's health at the end.
fn run(tree: ((u32, u32), (u16, u16))) -> (Vec<Seen>, Vec<(i32, i32)>, Option<i32>) {
    let (import, catalogue, mut assets) = context_full(RULES, ART);
    assets.insert(
        b"manx.shp",
        super::tests::shp((40, 40), 40, (12, 20), (14, 10)),
    );
    let (size, at) = tree;
    assets.insert(
        b"treex.shp",
        shape(
            (80, 160),
            &[
                (size, at, 9),
                (size, at, 9),
                ((1, 1), (0, 0), 0),
                ((1, 1), (0, 0), 0),
            ],
        ),
    );
    let rules = rules();
    let setup = testkit::setup(7);
    let mut world = glasnost_sim::load::new_game(
        rules.clone(),
        &testkit::empty_map(SIZE),
        testkit::catalogue(),
        &setup,
    )
    .expect("the synthetic game loads");
    let c = testkit::mid(SIZE, 0, 0);
    let jj = testkit::spawn(&mut world, SimKind::Infantry, "MANX", 0, c, 0);
    let cart = testkit::spawn(
        &mut world,
        SimKind::Vehicle,
        "CART",
        1,
        CellXY::new(c.x + 10, c.y),
        0,
    );
    let mut map = testkit::empty_map(SIZE);
    for k in 0..10 {
        map.objects.terrain.push(TerrainObject {
            cell: CellXY::new(c.x + k, c.y),
            type_name: b"TREEX".to_vec(),
        });
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
    let (mut built, painter) = build_parts(&inputs, true);
    let mut live = Live::new(
        &inputs,
        &mut built,
        painter,
        (world, rules.clone(), setup, Vec::new()),
    );
    let mut scene: Scene = built.scene;
    live.redraw(&inputs, &mut scene);
    let trees: BTreeSet<u32> = scene
        .items
        .iter()
        .filter(|i| i.group == Group::TerrainObjects)
        .filter_map(|i| i.object)
        .collect();
    assert_eq!(trees.len(), 10, "the trees are drawn");
    let attack = Order::Behaviour {
        actor: jj,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(cart)),
        dest: None,
    };
    let mut ordered = false;
    let mut point: Option<(i32, i32)> = None;
    let mut seen: Vec<Seen> = Vec::new();
    let mut heights: Vec<(i32, i32)> = Vec::new();
    let mut last_pos = None;
    for _ in 0..600 {
        if !ordered {
            ordered = live.issue_orders(HouseId(0), vec![attack.clone()]).issued > 0;
        }
        live.step();
        live.redraw(&inputs, &mut scene);
        let o = live.world.obj(jj).expect("the unit outlives the fight");
        let z = o.pos.z;
        let still = last_pos == Some(o.pos);
        last_pos = Some(o.pos);
        if heights.last().is_none_or(|&(_, h)| h != z) {
            heights.push((live.world.tick, z));
        }
        let body = scene
            .items
            .iter()
            .find(|i| {
                i.object == Some(sim_code(jj))
                    && i.group != Group::Status
                    && i.paint != Paint::Shadow
            })
            .map(|i| {
                let img = &scene.images[i.image as usize];
                (i.image, (i.x, i.y), (img.width(), img.height()))
            });
        if point.is_none()
            && z >= 400
            && still
            && let Some((_, (x, y), (w, h))) = body
        {
            point = Some((x + w as i32 / 2, y + h as i32 / 4));
        }
        let Some((px, py)) = point else { continue };
        let view = View {
            origin: (px - 64, py - 64),
            size: (128, 128),
            reference: DEPTH_REFERENCE,
        };
        let code = scene.object_at(px, py, &view, &Filter::default(), &ComposeQuirks::default());
        let pick = match code {
            Some(k) if k == sim_code(jj) => "unit",
            Some(k) if trees.contains(&k) => "tree",
            Some(_) => "other",
            None => "ground",
        };
        let over = scene
            .items_at(px, py, view.origin.1)
            .into_iter()
            .map(|i| (scene.items[i].group, scene.items[i].object))
            .collect();
        seen.push(Seen {
            tick: live.world.tick,
            z,
            body,
            pick,
            over,
        });
    }
    assert!(ordered, "the attack is issued");
    (seen, heights, live.world.obj(cart).map(|o| o.health))
}

/// The short trees: 50 × 40, their image from (15, 45) in the canvas, under
/// the flying unit's body.
const SHORT: ((u32, u32), (u16, u16)) = ((50, 40), (15, 45));
/// The tall trees: 70 × 150 from (5, 5), reaching up over its body.
const TALL: ((u32, u32), (u16, u16)) = ((70, 150), (5, 5));

/// #435: while a jumpjet unit hovers and fires under a still pointer, its
/// shots' effects pass over the pointer, and the pick stays the unit in
/// every drawn frame: an effect has no object and does not hide the object
/// under it, and the pick is the replayed drawing of the frame's own scene
/// (S-0102 claims 9 and 10; design: render (#68)). With a body opaque
/// throughout, nothing but the unit's own pixels could change the pick.
#[test]
fn the_pick_over_a_hovering_firing_jumpjet_stays_the_unit() {
    let (seen, _, health) = run(SHORT);
    assert!(
        seen.len() >= 100,
        "the unit hovers under the pointer: {} frames",
        seen.len()
    );
    assert!(
        seen.iter()
            .any(|s| s.over.iter().any(|(g, _)| *g == Group::Effects)),
        "an effect passes over the pointer"
    );
    // The synthetic CART's strength is 300.
    assert!(
        health.is_some_and(|h| h < 300),
        "the unit's shots hit: {health:?}"
    );
    let other: Vec<_> = seen.iter().filter(|s| s.pick != "unit").collect();
    assert!(other.is_empty(), "the pick leaves the unit: {other:?}");
}

/// #435: the trace, printed (`--nocapture`), for the short trees and the
/// tall ones.
#[test]
#[ignore = "a trace for the notes, not a check"]
fn trace_the_pick_over_a_flying_firing_jumpjet() {
    for (name, tree) in [("short", SHORT), ("tall", TALL)] {
        let (seen, heights, health) = run(tree);
        println!(
            "{name} trees: {} frames; heights (tick, z): {heights:?}",
            seen.len()
        );
        let mut last: Option<&Seen> = None;
        for s in &seen {
            let changed = last.is_none_or(|l| {
                l.pick != s.pick || l.body != s.body || l.z != s.z || l.over != s.over
            });
            if changed {
                println!(
                    "  tick {}: z {}, body {:?}, pick {}, over {:?}",
                    s.tick, s.z, s.body, s.pick, s.over
                );
            }
            last = Some(s);
        }
        let flips = seen.windows(2).filter(|w| w[0].pick != w[1].pick).count();
        let count = |what: &str| seen.iter().filter(|s| s.pick == what).count();
        let effects = seen
            .iter()
            .filter(|s| s.over.iter().any(|(g, _)| *g == Group::Effects))
            .count();
        println!(
            "{name} trees: pick changes {flips}; unit {}, tree {}, ground {}, other {}; frames \
             with an effect over the pointer: {effects}; the target's health at the end: {health:?}",
            count("unit"),
            count("tree"),
            count("ground"),
            count("other"),
        );
    }
}
