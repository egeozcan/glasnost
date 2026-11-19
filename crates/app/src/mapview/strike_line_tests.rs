//! #448: the air strike's strike line on the client (S-0519 claim 27, F8)
//! and the marked building's laser-target tint (claim 26). A designator
//! with a strike whose strike target is a building, its designating weapon
//! with the laser flag off and on. Invented types and numbers, no game
//! data.

use std::sync::Arc;

use glasnost_render::compose::{ComposeQuirks, Frame};
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::projection::project_lifted;
use glasnost_render::scene::{Filter, Group, Scene, View};
use glasnost_render::spots::{pack, widen};
use glasnost_rules::sim::BuildingType;
use glasnost_sim::rules::{Kind as SimKind, SimRules};
use glasnost_sim::strike::Strike;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos};

use super::beams::segment_pixels;
use super::build::{Inputs, Options, build_parts};
use super::live::Live;
use super::tests::{ART, RULES, context_full};

const SIZE: i32 = 24;

/// One game under test.
#[derive(Clone, Copy)]
struct Case {
    /// The designating weapon has the laser flag.
    laser: bool,
    /// The designator's cell, from the map's middle.
    designator: (i32, i32),
    /// The building's top-left cell, from the map's middle.
    building: (i32, i32),
    /// `quirks.airstrike_line_marker_over_shroud`.
    over_shroud: bool,
}

/// One drawn frame of a case.
struct Drawn {
    /// The tick the frame shows.
    tick: i32,
    /// The designator's fire point for the line's start.
    from: Pos,
    /// The strike target's reference point.
    target: Pos,
    /// The view the frame is composed for.
    view: View,
    /// The frame with nothing drawn and with the effects group alone, at
    /// light-alpha 127.
    plain: (Frame, Frame),
    /// The same at light-alpha 0.
    dark: (Frame, Frame),
    /// Whether the building is drawn marked.
    marked: bool,
}

/// The view holding both ends of the line with a margin.
fn view_of(a: (i32, i32), b: (i32, i32)) -> View {
    let (x0, y0) = (a.0.min(b.0) - 16, a.1.min(b.1) - 16);
    let (x1, y1) = (a.0.max(b.0) + 16, a.1.max(b.1) + 16);
    View {
        origin: (x0, y0),
        size: ((x1 - x0) as u32, (y1 - y0) as u32),
        reference: glasnost_render::compose::DEPTH_REFERENCE,
    }
}

/// Plays `case` for `frames` drawn frames: a designator of house 0 whose
/// strike holds a building of house 0 as its strike target from the start.
fn run(case: Case, frames: usize) -> Vec<Drawn> {
    run_with(case, frames, |_| {})
}

/// [`run`] with `prep` called on the live state before each tick.
fn run_with(case: Case, frames: usize, mut prep: impl FnMut(&mut Live)) -> Vec<Drawn> {
    let (import, catalogue, assets) = context_full(RULES, ART);
    let map = testkit::empty_map(SIZE);
    let mut r = testkit::combat_ruleset();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("HUTX".into());
    r.sim.buildings.insert(
        "HUTX".into(),
        BuildingType {
            strength: Some(400),
            foundation: Some([2, 2]),
            ..Default::default()
        },
    );
    // spec: S-0519 claim 27 (whether the designating weapon has the laser flag plays no part)
    r.sim.weapons.get_mut("Gun").unwrap().is_laser = Some(case.laser);
    r.globals.quirks.airstrike_line_marker_over_shroud = case.over_shroud;
    let rules = Arc::new(SimRules::new(&r, "synthetic-test"));
    let setup = testkit::setup(7);
    let mut world = glasnost_sim::load::new_game(rules.clone(), &map, testkit::catalogue(), &setup)
        .expect("the synthetic game loads");
    let d = testkit::spawn(
        &mut world,
        SimKind::Vehicle,
        "CART",
        0,
        testkit::mid(SIZE, case.designator.0, case.designator.1),
        0,
    );
    let b = testkit::spawn(
        &mut world,
        SimKind::Building,
        "HUTX",
        0,
        testkit::mid(SIZE, case.building.0, case.building.1),
        0,
    );
    // A strike under way at the building, as a start leaves it (S-0519 claim 17): its strike
    // target the building, watching the designator's own target so that no cancel follows
    // (claim 19). The designator has no target of its own and does not fire.
    let o = world.obj(d).expect("the designator");
    let (watched, watched_epoch) = (o.target, o.combat.target_epoch);
    world.strike.strikes.insert(
        d,
        Strike {
            target: Some(b),
            watched,
            watched_epoch,
            ..Default::default()
        },
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
    let mut scene = built.scene;
    let mut out = Vec::new();
    for _ in 0..frames {
        prep(&mut live);
        live.step();
        live.redraw(&inputs, &mut scene);
        out.push(drawn(&live, &scene, d, b));
    }
    out
}

/// What the frame `live` just drew shows.
fn drawn(live: &Live, scene: &Scene, d: ObjectId, b: ObjectId) -> Drawn {
    let w = &live.world;
    let burst = w.obj(d).expect("the designator").combat.burst;
    let from = glasnost_sim::combat::position::fire_point(w, d, 0, burst);
    let target = w.reference_point(b);
    let tick = (w.tick - 1).max(0);
    let view = view_of(
        project_lifted(from.x, from.y, from.z),
        project_lifted(target.x, target.y, target.z),
    );
    let n = (view.size.0 * view.size.1) as usize;
    let quirks = ComposeQuirks::default();
    let nothing = Filter {
        groups: Vec::new(),
        provisional: true,
    };
    let effects = Filter {
        groups: vec![Group::Effects],
        provisional: true,
    };
    let compose = |f: &Filter, a: u8| scene.compose_lit(&view, f, &quirks, Some(vec![a; n]));
    let cell = w.obj(b).expect("the building").pos.cell();
    let marked = live
        .protected_cells()
        .get(&cell)
        .is_some_and(|c| c.look.marked());
    Drawn {
        tick,
        from,
        target,
        view,
        plain: (compose(&nothing, 127), compose(&effects, 127)),
        dark: (compose(&nothing, 0), compose(&effects, 0)),
        marked,
    }
}

/// F8's end point *E* for reference point `r` at tick `n`: t = ⌊n / 4⌋ in
/// our engine, binary64 sine and cosine, truncated toward zero.
fn end_point(r: Pos, n: i32) -> Pos {
    // spec: S-0519 F8 (E = (trunc(X + 15 cos(0.37 t)), Y, trunc(Z + 15 sin t)); our engine t = ⌊n / 4⌋)
    let t = f64::from(n.div_euclid(4));
    Pos::new(
        (f64::from(r.x) + 15.0 * (0.37 * t).cos()).trunc() as i32,
        r.y,
        (f64::from(r.z) + 15.0 * t.sin()).trunc() as i32,
    )
}

/// F8's colour *C* for draw `r`.
fn colour(r: i32) -> [u32; 3] {
    // spec: S-0519 F8 (C = (r, 0, 0) when r ≤ 255, else (255, trunc(110k/14), trunc(30k/14)), k = r − 256)
    if r <= 255 {
        [r as u32, 0, 0]
    } else {
        let k = (r - 256) as u32;
        [255, 110 * k / 14, 30 * k / 14]
    }
}

/// The line's pixels in drawing order with their intensities α, for a line
/// from screen point `s0` to `s4` (F8: 4 parts, each a centre line and two
/// side lines). design: render (#448) (each part's centre line then its side
/// lines; a part after the first leaves out its first pixel, S-0514's
/// interim nearest-integer line between the points).
fn line_pixels(s0: (i32, i32), s4: (i32, i32)) -> Vec<((i32, i32), u32)> {
    let (dx, dy) = (s4.0 - s0.0, s4.1 - s0.1);
    // spec: S-0519 F8 (sₖ = s₀ + (trunc(kΔx/4), trunc(kΔy/4)))
    let s: Vec<(i32, i32)> = (0..5)
        .map(|k| (s0.0 + k * dx / 4, s0.1 + k * dy / 4))
        .collect();
    // spec: S-0519 F8 (side lines by (0, ±1) when |Δx| > |Δy|, else by (±1, 0))
    let side = if dx.abs() > dy.abs() {
        [(0, -1), (0, 1)]
    } else {
        [(-1, 0), (1, 0)]
    };
    // spec: S-0519 F8 (centre-line α 255, 192, 128, 64; side-line α 100, 75, 50, 25)
    let centre = [255, 192, 128, 64];
    let sides = [100, 75, 50, 25];
    let mut out = Vec::new();
    for k in 1..=4 {
        let px: Vec<(i32, i32)> = segment_pixels(s[k - 1], s[k])
            .into_iter()
            .skip(usize::from(k > 1))
            .collect();
        for (a, (ox, oy)) in [
            (centre[k - 1], (0, 0)),
            (sides[k - 1], side[0]),
            (sides[k - 1], side[1]),
        ] {
            out.extend(px.iter().map(|&(x, y)| ((x + ox, y + oy), a)));
        }
    }
    out
}

/// The end marker's pixels at screen point `s4` (F8).
fn marker_pixels(s4: (i32, i32)) -> Vec<(i32, i32)> {
    let (x, y) = s4;
    let mut out = Vec::new();
    // spec: S-0519 F8 (rows y and y + 1 from column x − 2 to x + 1; columns x − 1 and x from row y − 1 to y + 2)
    for row in [y, y + 1] {
        out.extend((x - 2..=x + 1).map(|c| (c, row)));
    }
    for col in [x - 1, x] {
        out.extend((y - 1..=y + 2).map(|r| (col, r)));
    }
    out
}

/// `bg` with the strike line of draw `r` from screen point `s0` to `s4`
/// drawn over it by F8, every pixel passing the depth test (nothing drawn
/// stores depth).
fn expected(bg: &Frame, view: &View, s0: (i32, i32), s4: (i32, i32), r: i32) -> Vec<u16> {
    let mut px = bg.color.clone();
    let w = view.size.0 as i32;
    let at = |(x, y): (i32, i32)| {
        let (vx, vy) = (x - view.origin.0, y - view.origin.1);
        (vx >= 0 && vy >= 0 && vx < w && vy < view.size.1 as i32).then(|| (vy * w + vx) as usize)
    };
    if s4.1 < s0.1 {
        for p in marker_pixels(s4) {
            if let Some(i) = at(p) {
                // spec: S-0519 F8 (the end marker in (255, 0, 0), opaque)
                px[i] = pack([255, 0, 0]);
            }
        }
    }
    let c = colour(r);
    for (p, a) in line_pixels(s0, s4) {
        let Some(i) = at(p) else {
            continue;
        };
        let v = u32::from(bg.alpha[i]);
        if v == 0 {
            // spec: S-0519 F8 (pixels with v = 0 are left as they are)
            continue;
        }
        let d = widen(px[i]);
        // spec: S-0519 F8 (p = ⌊(⌊d(256 − α)/256⌋ + ⌊cα/256⌋) × v/128⌋) and claim 27 (capped at 255)
        let p: [u32; 3] =
            std::array::from_fn(|k| ((d[k] * (256 - a) / 256 + c[k] * a / 256) * v / 128).min(255));
        px[i] = pack(p);
    }
    px
}

/// The draw *r* in 190 … 270 whose strike line the frame shows, if one.
fn shown_draw(f: &Drawn) -> Option<i32> {
    let end = end_point(f.target, f.tick);
    let s0 = project_lifted(f.from.x, f.from.y, f.from.z);
    let s4 = project_lifted(end.x, end.y, end.z);
    let (bg, fx) = &f.plain;
    // spec: S-0519 F8 (one inclusive ranged draw r = R(190, 270))
    (190..=270).find(|&r| expected(bg, &f.view, s0, s4, r) == fx.color)
}

/// spec: S-0519 claim 27 and F8 (a designator whose strike target is a building has a strike line
/// in every drawn frame, whatever the laser flag: from the primary fire point to the jittered end
/// point, 4 parts of a centre line and two side lines one pixel away above and below a wide line,
/// left and right of a tall one, blended by F8 at the light-alpha value; one colour draw per line
/// in each drawn frame)
#[test]
fn a_strike_at_a_building_draws_its_strike_line_whatever_the_laser_flag() {
    for laser in [false, true] {
        // A tall line and a wide one, each with its end lower on screen than its start.
        for (designator, building) in [((-4, -4), (3, 3)), ((-6, 2), (2, -5))] {
            let case = Case {
                laser,
                designator,
                building,
                over_shroud: true,
            };
            let frames = run(case, 12);
            let mut draws = std::collections::BTreeSet::new();
            for f in &frames {
                let (bg, fx) = &f.plain;
                assert_ne!(
                    bg.color, fx.color,
                    "laser {laser}, {designator:?}: tick {}: a strike line is drawn",
                    f.tick
                );
                let r = shown_draw(f);
                assert!(
                    r.is_some(),
                    "laser {laser}, {designator:?}: tick {}: the frame shows F8's line for one draw",
                    f.tick
                );
                draws.extend(r);
            }
            assert!(
                draws.len() > 1,
                "laser {laser}: a new colour draw in each drawn frame: {draws:?}"
            );
        }
    }
}

/// spec: S-0519 claim 27 and F8 (with the end higher on screen than the start, the end marker in
/// opaque (255, 0, 0) with no light-alpha shading: shown even where the light-alpha value is 0,
/// where the line itself leaves every pixel as it is; quirks.airstrike_line_marker_over_shroud
/// off: not drawn where the light-alpha value is 0)
#[test]
fn the_end_marker_shows_over_full_shroud_unless_switched_off() {
    for over_shroud in [true, false] {
        let case = Case {
            laser: false,
            designator: (3, 3),
            building: (-4, -4),
            over_shroud,
        };
        for f in &run(case, 8) {
            let end = end_point(f.target, f.tick);
            let s0 = project_lifted(f.from.x, f.from.y, f.from.z);
            let s4 = project_lifted(end.x, end.y, end.z);
            assert!(s4.1 < s0.1, "the end lies above the start on screen");
            let (bg, fx) = &f.dark;
            let w = f.view.size.0 as i32;
            let mut want = bg.color.clone();
            if over_shroud {
                for (x, y) in marker_pixels(s4) {
                    want[((y - f.view.origin.1) * w + x - f.view.origin.0) as usize] =
                        pack([255, 0, 0]);
                }
            }
            assert_eq!(
                fx.color, want,
                "over_shroud {over_shroud}: tick {}: at light-alpha 0 the marker alone",
                f.tick
            );
            // At light-alpha 127 the marker is drawn first and the line over it.
            assert!(
                shown_draw(f).is_some(),
                "over_shroud {over_shroud}: tick {}: the marker, then F8's line",
                f.tick
            );
        }
    }
}

/// spec: S-0519 claim 26 (a building is drawn marked while it is a strike's strike target, from
/// the strike alone: the designating weapon's laser flag plays no part)
#[test]
fn the_strike_target_is_drawn_marked_whatever_the_laser_flag() {
    for laser in [false, true] {
        let case = Case {
            laser,
            designator: (-4, -4),
            building: (3, 3),
            over_shroud: true,
        };
        for f in &run(case, 3) {
            assert!(f.marked, "laser {laser}: tick {}: marked", f.tick);
        }
    }
}
