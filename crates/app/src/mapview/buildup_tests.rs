//! #423: a building in the *construction* state draws its build-up sprite,
//! not its finished body (S-0108 claims 1 to 5, 7 and 9; S-0406 claims 14,
//! 15 and 17, F6 and F7). A building placed in play is followed from its
//! placement to its handover, and then through its sale. Invented types and
//! numbers, no game data.

use std::sync::Arc;

use glasnost_convert::kinds::Kind;
use glasnost_render::compose::Paint;
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::scene::{Group, Scene};
use glasnost_rules::sim::BuildingType;
use glasnost_sim::econ::placement::begin_construction;
use glasnost_sim::econ::sell::{Stage, buildup_range};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Kind as SimKind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::HouseId;

use super::build::{Inputs, Options, build_parts};
use super::live::{Live, sim_code};
use super::tests::{ART, RULES, context_full, shp};

const SIZE: i32 = 24;
const ALPHA: HouseId = HouseId(0);

/// The body sprite's frames are 4 × 4 (tests.rs); the build-up sprite's
/// are 5 × 5, so their items can be told apart.
const BODY: u32 = 4;
/// See [`BODY`].
const BUILDUP: u32 = 5;
/// The build-up sprite's image-frame count *N_b*: 4 build-up frames and
/// their 4 shadows (S-0108 claim 5: *n* = ⌊*N_b* / 2⌋ = 4).
const BUILDUP_FRAMES: usize = 8;
/// The gate-stage count *G* of the gate (S-0108 claim 4: *n* = *G* + 1 = 4).
const GATE_STAGES: i32 = 3;

/// The type under test.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Hut {
    /// Its art names a build-up sprite.
    sprite: bool,
    /// It is a gate.
    gate: bool,
}

/// What a drawn frame shows of the building.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Look {
    /// Its lit body items: (image width, pixel value = image frame + 1).
    body: Vec<(u32, u8)>,
    /// Its shadow items, the same way.
    shadow: Vec<(u32, u8)>,
    /// Its bib items.
    bibs: usize,
}

/// One drawn frame.
#[derive(Clone, Debug)]
struct Seen {
    /// The tick the frame shows (the last one the game ran).
    tick: i32,
    /// The building's construction has not completed.
    building_up: bool,
    /// The tick its build-down began, while it has one.
    down: Option<i32>,
    /// The building is still in play.
    present: bool,
    look: Look,
}

/// What `scene` shows of object code `code`.
fn look(scene: &Scene, code: u32) -> Look {
    let mut l = Look::default();
    for i in scene.items.iter().filter(|i| i.object == Some(code)) {
        let image = &scene.images[i.image as usize];
        let px = (image.width(), image.pixels()[0]);
        match (i.group, i.paint) {
            (Group::Bibs, _) => l.bibs += 1,
            (_, Paint::Shadow) => l.shadow.push(px),
            (Group::Buildings, Paint::Lit(_)) => l.body.push(px),
            _ => {}
        }
    }
    l
}

/// The building's range (*n*, *d_b*) and placement tick *T*, and every
/// drawn frame from its placement in play to its handover and, with
/// `sell`, through its sale to its removal.
fn trace(hut: Hut, sell: bool) -> ((i32, i32), i32, Vec<Seen>) {
    let rules_text = if hut.gate {
        format!("{RULES}[HUTX]\nGate=yes\n")
    } else {
        RULES.to_string()
    };
    let art_text = if hut.gate {
        ART.replacen(
            "[HUTX]\n",
            &format!("[HUTX]\nGateStages={GATE_STAGES}\n"),
            1,
        )
    } else {
        ART.to_string()
    };
    let (mut import, catalogue, mut assets) = context_full(&rules_text, &art_text);
    if hut.sprite {
        import
            .arts
            .get_mut(&(Kind::Buildings, 0))
            .expect("HUTX's art")
            .building_sprites
            .insert("buildup", b"hutxmk.shp".to_vec());
        // Canvas 40 × 30 as the body's; frame i has pixel value i + 1.
        assets.insert(
            b"hutxmk.shp",
            shp((40, 30), BUILDUP_FRAMES, (BUILDUP, BUILDUP), (6, 8)),
        );
    }
    let map = testkit::empty_map(SIZE);
    let mut r = testkit::combat_ruleset();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("HUTX".into());
    let mut t = BuildingType {
        strength: Some(400),
        foundation: Some([2, 2]),
        ..Default::default()
    };
    if hut.sprite {
        // spec: S-0108 Import mapping (Buildup: the type has a build-up sprite of N_b frames)
        t.econ.buildup = Some(true);
        t.econ.buildup_frames = Some(BUILDUP_FRAMES as i32);
    }
    if hut.gate {
        t.gate = Some(true);
        t.econ.gate_stages = Some(GATE_STAGES);
    }
    r.sim.buildings.insert("HUTX".into(), t);
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let setup = testkit::setup(7);
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
    // spec: S-0406 claim 17 (a building placed in play enters under construction)
    begin_construction(&mut world, b, false);
    let placed = world.tick;
    let o = world.obj(b).expect("the building");
    let range = buildup_range(&world, o.type_index);
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
    let mut seen = Vec::new();
    let mut sold = false;
    let mut after = 0;
    for _ in 0..600 {
        live.step();
        live.redraw(&inputs, &mut scene);
        let o = live.world.obj(b);
        let s = Seen {
            // The drawing shows the last tick the game ran.
            tick: (live.world.tick - 1).max(0),
            building_up: o.is_some_and(|o| o.econ.construction.building_up),
            down: o
                .and_then(|o| o.econ.sale.as_ref())
                .and_then(|s| match s.stage {
                    Stage::BuildDown(since) => Some(since),
                    _ => None,
                }),
            present: o.is_some(),
            look: look(&scene, sim_code(b)),
        };
        let done = !s.building_up;
        seen.push(s);
        if o.is_none() {
            break;
        }
        if done {
            after += 1;
            if !sell && after > 3 {
                break;
            }
            if sell && !sold && after > 3 {
                assert!(
                    glasnost_sim::orders::issue(&mut live.world, ALPHA, Order::Sell { actor: b }),
                    "the sell order is accepted"
                );
                sold = true;
            }
        }
    }
    ((range.n, range.delay), placed, seen)
}

/// The build-up stage *k* a placed building shows at tick `t`, placed at
/// tick `placed` with range (`n`, `d`) of at least 2 stages and a positive
/// delay; none from its handover on (S-0406 claim 17 and F7: the first run
/// from *T* to *R* = *T* + (*n* − 1)*d*, the restart at *C* = *R* + 1, the
/// handover at *H* = *C* + (*n* − 1)*d*).
fn placed_stage(t: i32, placed: i32, (n, d): (i32, i32)) -> Option<i32> {
    let r = placed + (n - 1) * d;
    let c = r + 1;
    let h = c + (n - 1) * d;
    if t <= r {
        Some((t - placed) / d)
    } else if t < h {
        Some((t - c) / d)
    } else {
        None
    }
}

/// The look of build-up frame `f` of the build-up sprite: its body frame
/// and its shadow, frame `f` + ⌊N_b / 2⌋ (S-0108 claim 9), and no bib
/// (claim 7).
fn buildup_look(f: i32) -> Look {
    let half = (BUILDUP_FRAMES / 2) as i32;
    Look {
        body: vec![(BUILDUP, (f + 1) as u8)],
        shadow: vec![(BUILDUP, (f + half + 1) as u8)],
        bibs: 0,
    }
}

/// #423. spec: S-0108 claims 1, 2, 5, 7 and 9 (in the construction state the build-up sprite is
/// drawn at frame k, its shadow from the same sprite, no bib) and S-0406 claim 17, F7
/// (`quirks.placed_buildup_twice`: the build-up runs from the placement tick T to R = T + (n − 1)d,
/// shows stage 0 again at C = R + 1 and hands over at H = T + 2(n − 1)d + 1, where the body sprite
/// takes over with its bib)
#[test]
fn a_placed_building_shows_its_build_up_twice_then_its_body() {
    let hut = Hut {
        sprite: true,
        gate: false,
    };
    let (range, placed, seen) = trace(hut, false);
    let (n, d) = range;
    assert_eq!(n, 4, "n = ⌊N_b / 2⌋");
    assert!(d >= 2, "a delay of several ticks a stage: {d}");
    let handover = placed + 2 * (n - 1) * d + 1;
    assert!(
        seen.iter().any(|s| s.tick >= handover),
        "followed to the handover at {handover}"
    );
    for s in &seen {
        match placed_stage(s.tick, placed, range) {
            Some(k) => {
                assert!(s.building_up, "still building at tick {}", s.tick);
                assert_eq!(
                    s.look,
                    buildup_look(k),
                    "tick {}: build-up stage {k}",
                    s.tick
                );
            }
            None => {
                assert!(!s.building_up, "handed over by tick {}", s.tick);
                assert_eq!(
                    s.look.body.iter().map(|b| b.0).collect::<Vec<_>>(),
                    vec![BODY],
                    "tick {}: the body sprite after the handover",
                    s.tick
                );
                assert_eq!(s.look.bibs, 1, "tick {}: the bib is back", s.tick);
            }
        }
    }
}

/// #423. spec: S-0108 claims 2, 5, 7 and 9 (a sold building's build-down shows frame
/// a + c − k − 1 = n − 1 − k of the build-up sprite from the first tick of its build-down, its
/// shadow from the same sprite and no bib; the body before it) and claim 10's timing (the
/// build-down from its first tick)
#[test]
fn a_sold_building_plays_its_build_up_backwards() {
    let hut = Hut {
        sprite: true,
        gate: false,
    };
    let ((n, d), _, seen) = trace(hut, true);
    let down: Vec<&Seen> = seen.iter().filter(|s| s.down.is_some()).collect();
    assert!(
        down.len() > 2,
        "a build-down of several ticks: {}",
        down.len()
    );
    assert!(
        seen.last().is_some_and(|s| !s.present),
        "the sale ends with the building's removal"
    );
    for s in &down {
        let since = s.down.unwrap();
        let k = (s.tick - since) / d;
        assert_eq!(
            s.look,
            buildup_look(n - 1 - k),
            "tick {}: build-down stage {k} from {since}",
            s.tick
        );
    }
    let before = seen
        .iter()
        .filter(|s| !s.building_up && s.down.is_none() && s.present);
    for s in before {
        assert_eq!(
            s.look.body.iter().map(|b| b.0).collect::<Vec<_>>(),
            vec![BODY],
            "tick {}: the body before the build-down",
            s.tick
        );
    }
}

/// #423. spec: S-0108 claim 4 (a gate's build-up runs backwards, f = a + c − k − 1 with
/// n = G + 1) and claim 5 (the gate range), S-0406 claim 17 (the same two runs)
#[test]
fn a_placed_gate_builds_up_backwards() {
    let hut = Hut {
        sprite: true,
        gate: true,
    };
    let (range, placed, seen) = trace(hut, false);
    let (n, _) = range;
    assert_eq!(n, GATE_STAGES + 1, "n = G + 1");
    let mut building = 0;
    for s in &seen {
        if let Some(k) = placed_stage(s.tick, placed, range) {
            building += 1;
            assert_eq!(
                s.look,
                buildup_look(n - 1 - k),
                "tick {}: gate build-up stage {k}",
                s.tick
            );
        }
    }
    assert!(building > 2, "built up over {building} drawn frames");
}

/// #423. spec: S-0108 claim 1 (a building in the construction state whose type has no build-up
/// sprite draws nothing of its body: no body, bib or shadow) and S-0406 claim 17 (its zero-delay
/// range hands over at T + 3)
#[test]
fn a_type_without_a_build_up_sprite_draws_no_body_while_building() {
    let hut = Hut {
        sprite: false,
        gate: false,
    };
    let (_, _, seen) = trace(hut, false);
    let building: Vec<&Seen> = seen.iter().filter(|s| s.building_up).collect();
    assert!(!building.is_empty(), "drawn while building");
    for s in &building {
        assert_eq!(
            s.look,
            Look::default(),
            "tick {}: nothing of its body",
            s.tick
        );
    }
    let last = seen.last().expect("drawn frames");
    assert!(!last.building_up, "handed over");
    assert_eq!(last.look.body.len(), 1, "the body after the handover");
}
