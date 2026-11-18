//! A tree beside the west corner of a large building the game placed
//! (#437): both are depth writers, the tree in the cell pass and the
//! building's body with its companion depth image, so each pixel goes to
//! the nearer of the two (S-0102 claims 10 and 16) whatever their order,
//! the earlier one on a tie. These tests replay the pixels where the two
//! overlap and check each writer's depth against F2 and F5 by hand, the
//! depth stored before each and which one shows, on the CPU reference and
//! on the GPU path. Invented types, images and depth image, no game data:
//! the companion here gives every column of the body the upright depth of
//! a wall standing on the foundation's front edge.

use std::sync::Arc;

use glasnost_convert::kinds::Kind;
use glasnost_fmt::image::IndexedImage;
use glasnost_fmt::shp::{ShpWriter, WriteFrame};
use glasnost_map::CellXY;
use glasnost_map::objects::TerrainObject;
use glasnost_render::compose::{
    ComposeQuirks, DEPTH_REFERENCE, DepthTest, Frame, Profile, quantise_depth,
};
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::projection::project_lifted;
use glasnost_render::scene::{Filter, Group, Item, ItemDepth, Scene, View};
use glasnost_rules::sim::BuildingType;
use glasnost_sim::rules::{Kind as SimKind, SimRules};
use glasnost_sim::testkit;

use super::build::{COMPANION_FILE, Inputs, Options, build_parts};
use super::live::{Live, sim_code};
use super::tests::{ART, RULES, context_full};

const SIZE: i32 = 24;
/// The foundation, 4 × 4 cells (under the companion's width limit of 8).
const FOOT: i32 = 4;
/// The body image frame: 240 × 185 pixels from 120 left of and 60 above
/// the draw point (the foundation's top corner), so it spans the whole
/// foundation diamond (its west corner 120 left, its south corner 120
/// below) and its last row is 124 below the draw point.
const BODY: (u32, u32) = (240, 185);
const BODY_AT: (i32, i32) = (-120, -60);
/// The tree's image frame: 70 × 60 pixels, its last row 4 below its draw
/// point (the centre of its cell), 35 to either side of it.
const TREE: (u32, u32) = (70, 60);
const TREE_AT: (i32, i32) = (-35, -55);

/// A shape file of canvas `canvas` whose image frames are `frames`: a
/// rectangle size, its place on the canvas and one value for every pixel.
pub(super) fn shape(canvas: (u16, u16), frames: &[((u32, u32), (u16, u16), u8)]) -> Vec<u8> {
    ShpWriter {
        width: canvas.0,
        height: canvas.1,
        frames: frames
            .iter()
            .map(|&((w, h), (x, y), v)| WriteFrame {
                x,
                y,
                image: IndexedImage::new(w, h, vec![v; (w * h) as usize]).unwrap(),
                compression: 1,
            })
            .collect(),
    }
    .write()
}

/// The row of the foundation's front (south-west and south-east) edge
/// below column `u` of the body, both counted from the draw point.
fn front_edge(u: i32) -> i32 {
    FOOT * 30 - u.abs() / 2
}

/// The invented companion adjustment *a* at body pixel (`u`, `v`) (counted
/// from the draw point): with the body's anchor *d*(*r*₁) − 2 (F2, F5),
/// pixel depth *d*(*r*₁) − 2 − *a* is the depth an upright wall standing on
/// the front edge *g* below the pixel would have, *d*(*g*) − 2 −
/// ⌊(*g* − *v*)/3⌋; under the edge it is the ground's, *d*(*v*) − 2.
fn adjustment(u: i32, v: i32) -> i32 {
    let last = BODY_AT.1 + BODY.1 as i32 - 1;
    let g = front_edge(u);
    if v <= g {
        (g - last) + (g - v) / 3
    } else {
        v - last
    }
}

/// The shared companion file: one image frame whose stored rectangle
/// starts at the canvas corner, placed by F5 for a 4 × 4 live body with no
/// art displacement, D = (198, 446 − 15 · 6), and covering the whole body.
fn companion() -> Vec<u8> {
    let d = (198, 446 - 15 * (2 * FOOT - 2));
    let (w, h) = (
        d.0 + BODY_AT.0 + BODY.0 as i32,
        d.1 + BODY_AT.1 + BODY.1 as i32,
    );
    let mut bytes = Vec::with_capacity((w * h) as usize);
    for cv in 0..h {
        for cu in 0..w {
            let (u, v) = (cu - d.0, cv - d.1);
            let on_body = (BODY_AT.0..BODY_AT.0 + BODY.0 as i32).contains(&u)
                && (BODY_AT.1..BODY_AT.1 + BODY.1 as i32).contains(&v);
            let a = if on_body { adjustment(u, v) } else { 0 };
            assert!((-64..=127).contains(&a), "a = {a} has a byte of its own");
            // spec: S-0102 F5 (a = ((b − 65 + 128) mod 256) − 128, so b = a + 65 for a ≠ −65)
            bytes.push((a + 65).rem_euclid(256) as u8);
        }
    }
    ShpWriter {
        width: w as u16,
        height: h as u16,
        frames: vec![WriteFrame {
            x: 0,
            y: 0,
            image: IndexedImage::new(w as u32, h as u32, bytes).unwrap(),
            compression: 1,
        }],
    }
    .write()
}

/// The simulation's rules: the synthetic combat ruleset with HUTX, a
/// 4 × 4 building.
fn rules() -> Arc<SimRules> {
    let mut r = testkit::combat_ruleset();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("HUTX".into());
    r.sim.buildings.insert(
        "HUTX".into(),
        BuildingType {
            strength: Some(400),
            foundation: Some([FOOT, FOOT]),
            ..Default::default()
        },
    );
    Arc::new(SimRules::new(&r, "synthetic-test"))
}

/// A drawn scene of one case: Alpha's HUTX placed by the game with its
/// top-left foundation cell at C, and the map's TREEX on C + `tree`.
struct Case {
    scene: Scene,
    /// The tree's body item and the building's body item, by index.
    tree: usize,
    body: usize,
    /// The building's draw point and the tree's draw point.
    anchor: (i32, i32),
    tree_point: (i32, i32),
}

fn case(tree: (i32, i32)) -> Case {
    let art = ART.replacen("Foundation=2x2", "Foundation=4x4", 1);
    let (mut import, catalogue, mut assets) = context_full(RULES, &art);
    import
        .arts
        .get_mut(&(Kind::Buildings, 0))
        .unwrap()
        .building_sprites
        .remove("bib");
    // Two body frames (whole and damaged), two empty shadow frames.
    let body_at = ((150 + BODY_AT.0) as u16, (150 + BODY_AT.1) as u16);
    assets.insert(
        b"hutx.shp",
        shape(
            (300, 300),
            &[
                (BODY, body_at, 5),
                (BODY, body_at, 5),
                ((1, 1), (0, 0), 0),
                ((1, 1), (0, 0), 0),
            ],
        ),
    );
    let tree_at = ((40 + TREE_AT.0) as u16, (60 + TREE_AT.1) as u16);
    assets.insert(
        b"treex.shp",
        shape(
            (80, 120),
            &[
                (TREE, tree_at, 9),
                (TREE, tree_at, 9),
                ((1, 1), (0, 0), 0),
                ((1, 1), (0, 0), 0),
            ],
        ),
    );
    assets.insert(COMPANION_FILE, companion());
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
    let b = testkit::spawn(&mut world, SimKind::Building, "HUTX", 0, c, 0);
    let tree_cell = CellXY::new(c.x + tree.0, c.y + tree.1);
    let mut map = testkit::empty_map(SIZE);
    map.objects.terrain.push(TerrainObject {
        cell: tree_cell,
        type_name: b"TREEX".to_vec(),
    });
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
    for _ in 0..3 {
        live.step();
    }
    live.redraw(&inputs, &mut scene);
    let one = |what: &str, f: &dyn Fn(&Item) -> bool| {
        let found: Vec<usize> = (0..scene.items.len())
            .filter(|&i| f(&scene.items[i]))
            .collect();
        assert_eq!(found.len(), 1, "one {what}: {found:?}");
        found[0]
    };
    let tree_idx = one("tree body", &|i| i.group == Group::TerrainObjects);
    let body_idx = one("building body", &|i| {
        i.group == Group::Buildings && i.object == Some(sim_code(b))
    });
    let centre = glasnost_render::projection::cell_centre(tree_cell.x, tree_cell.y);
    Case {
        scene,
        tree: tree_idx,
        body: body_idx,
        anchor: project_lifted(256 * c.x, 256 * c.y, 0),
        tree_point: project_lifted(centre.0, centre.1, 0),
    }
}

/// What happened at one pixel where the tree and the body overlap.
#[derive(Debug, PartialEq, Eq)]
struct Pixel {
    at: (i32, i32),
    /// The depth stored before the tree, the tree's pixel depth, the depth
    /// stored before the body and the body's pixel depth.
    before_tree: u16,
    tree: u16,
    before_body: u16,
    body: u16,
    /// Whether the body's pixel was drawn over the tree's.
    body_shows: bool,
}

/// The frame of the items `items` of `scene` over the rectangle `rect`
/// (world pixels: left, top, width, height).
fn compose(scene: &Scene, items: Vec<Item>, rect: (i32, i32, u32, u32)) -> Frame {
    let mut s = scene.clone();
    s.items = items;
    s.spots.clear();
    let view = View {
        origin: (rect.0, rect.1),
        size: (rect.2, rect.3),
        reference: DEPTH_REFERENCE,
    };
    s.compose(&view, &Filter::default(), &ComposeQuirks::default())
}

/// The rectangle (left, top, width, height) of an item.
fn rect(scene: &Scene, item: &Item) -> (i32, i32, i32, i32) {
    let img = &scene.images[item.image as usize];
    (item.x, item.y, img.width() as i32, img.height() as i32)
}

/// The overlap of the two items' rectangles, as a view rectangle.
fn overlap(c: &Case) -> (i32, i32, u32, u32) {
    let a = rect(&c.scene, &c.scene.items[c.tree]);
    let b = rect(&c.scene, &c.scene.items[c.body]);
    let l = a.0.max(b.0);
    let t = a.1.max(b.1);
    let r = (a.0 + a.2).min(b.0 + b.2);
    let bt = (a.1 + a.3).min(b.1 + b.3);
    assert!(
        l < r && t < bt,
        "the tree and the body overlap: {a:?} {b:?}"
    );
    (l, t, (r - l) as u32, (bt - t) as u32)
}

/// Every overlap pixel of the case, replayed on the CPU reference.
fn trace(c: &Case) -> Vec<Pixel> {
    let o = overlap(c);
    let items = &c.scene.items;
    let before_tree = compose(&c.scene, items[..c.tree].to_vec(), o);
    let tree = compose(&c.scene, vec![items[c.tree].clone()], o);
    let before_body = compose(&c.scene, items[..c.body].to_vec(), o);
    let body = compose(&c.scene, vec![items[c.body].clone()], o);
    let after_body = compose(&c.scene, items[..=c.body].to_vec(), o);
    let mut out = Vec::new();
    for y in 0..o.3 {
        for x in 0..o.2 {
            let i = (y * o.2 + x) as usize;
            out.push(Pixel {
                at: (o.0 + x as i32, o.1 + y as i32),
                before_tree: before_tree.depth[i],
                tree: tree.depth[i],
                before_body: before_body.depth[i],
                body: body.depth[i],
                body_shows: after_body.color[i] == body.color[i]
                    && after_body.depth[i] == body.depth[i],
            });
        }
    }
    out
}

/// The tree's pixel depth at world row `r` by hand (S-0102 F1, F2, claim
/// 13: upright, −L(0) − 12, rounded at its last row).
fn tree_depth(c: &Case, r: i32) -> i32 {
    let last = c.tree_point.1 + TREE_AT.1 + TREE.1 as i32 - 1;
    quantise_depth(DEPTH_REFERENCE - last - 12) - (last - r) / 3
}

/// The body's pixel depth at world pixel (`x`, `r`) by hand (S-0102 F2,
/// F5, claim 13: the unrounded upright anchor d(r₁) − L(0) − 2 minus the
/// companion's adjustment there).
fn body_depth(c: &Case, x: i32, r: i32) -> i32 {
    let last = c.anchor.1 + BODY_AT.1 + BODY.1 as i32 - 1;
    DEPTH_REFERENCE - last - 2 - adjustment(x - c.anchor.0, r - c.anchor.1)
}

/// Checks the case's two items and every overlap pixel against the hand
/// values; returns the pixels.
fn checked(c: &Case) -> Vec<Pixel> {
    let tree = &c.scene.items[c.tree];
    let body = &c.scene.items[c.body];
    assert!(
        c.tree < c.body,
        "the tree is drawn first (claim 16 ranks 4 and 5)"
    );
    assert!(
        c.tree < c.scene.cell_pass_len,
        "the tree is in the cell pass"
    );
    assert_eq!(
        tree.test,
        DepthTest::Writer,
        "the tree writes depth (claim 10)"
    );
    assert!(matches!(
        tree.depth,
        ItemDepth::Profile {
            profile: Profile::Upright,
            bias: -12,
            companion: None
        }
    ));
    assert_eq!(
        (tree.x, tree.y),
        (c.tree_point.0 + TREE_AT.0, c.tree_point.1 + TREE_AT.1)
    );
    assert_eq!(
        body.test,
        DepthTest::Writer,
        "the body writes depth (claim 10)"
    );
    assert!(
        matches!(
            body.depth,
            ItemDepth::Profile {
                profile: Profile::Upright,
                bias: -2,
                companion: Some(_)
            }
        ),
        "the body: upright, −2, with its companion (claims 13 and 31): {:?}",
        body.depth
    );
    assert_eq!(
        (body.x, body.y),
        (c.anchor.0 + BODY_AT.0, c.anchor.1 + BODY_AT.1)
    );
    let pixels = trace(c);
    for p in &pixels {
        let (x, r) = p.at;
        assert_eq!(i32::from(p.tree), tree_depth(c, r), "{p:?}");
        assert_eq!(i32::from(p.body), body_depth(c, x, r), "{p:?}");
        // spec: S-0102 claim 10, F2 (a writer is drawn iff its depth is strictly less than the
        // stored depth, and stores its depth there)
        let after_tree = if p.tree_drawn() {
            p.tree
        } else {
            p.before_tree
        };
        assert!(
            p.before_body <= after_tree,
            "the tree's depth is kept: {p:?}"
        );
        assert_eq!(p.body_shows, p.body < p.before_body, "{p:?}");
    }
    pixels
}

impl Pixel {
    /// Whether the tree's pixel was drawn (over the ground: the invented
    /// tile's depth data hides the tree's last rows).
    fn tree_drawn(&self) -> bool {
        self.tree < self.before_tree
    }
}

/// The overlap pixels where the tree was drawn, and those of them where the
/// body was drawn over it.
fn contested(pixels: &[Pixel]) -> (usize, Vec<&Pixel>) {
    let drawn: Vec<&Pixel> = pixels.iter().filter(|p| p.tree_drawn()).collect();
    let n = drawn.len();
    (n, drawn.into_iter().filter(|p| p.body_shows).collect())
}

/// #437. spec: S-0102 claims 10 and 16 (two cell-pass writers: the nearer pixel shows), claim 23
/// (the tree at its cell's centre), claims 13 and 31, F2 and F5 (the tree upright with −12, the
/// body's unrounded anchor with −2 minus its companion's values)
///
/// The tree stands on the cell north-west of the foundation's west corner cell, behind the
/// foundation (its x + y one less): wherever the two overlap the body is nearer and covers it.
#[test]
fn a_tree_behind_a_buildings_west_corner_is_hidden_by_its_body() {
    let c = case((-1, FOOT - 1));
    let pixels = checked(&c);
    let (drawn, covered) = contested(&pixels);
    assert!(drawn > 100, "the tree is drawn at {drawn} overlap pixels");
    let shows: Vec<&Pixel> = pixels
        .iter()
        .filter(|p| p.tree_drawn() && !p.body_shows)
        .collect();
    assert_eq!(covered.len(), drawn, "the tree shows at {shows:?}");
}

/// #437. spec: S-0102 claims 10, 13, 16, 23 and 31, F2 and F5 (as above)
///
/// The tree stands on the cell west of the foundation's west corner cell, beside it (the same
/// x + y): its base is lower on screen than the body's front edge there and its bias 10 nearer,
/// so it covers the body's west corner wherever they overlap. The same on the cell south-west of
/// the corner cell, in front of the foundation.
#[test]
fn a_tree_beside_or_before_a_buildings_west_corner_covers_its_body() {
    for tree in [(-1, FOOT), (0, FOOT)] {
        let c = case(tree);
        let pixels = checked(&c);
        let (drawn, covered) = contested(&pixels);
        assert!(
            drawn > 100,
            "{tree:?}: the tree is drawn at {drawn} overlap pixels"
        );
        assert!(
            covered.is_empty(),
            "{tree:?}: the body covers the tree at {covered:?}"
        );
    }
}

/// #437: the GPU map path draws the overlap as the CPU reference does, for the three cases. Needs
/// a GPU adapter (software is fine); skipped without one unless `GLASNOST_REQUIRE_GPU=1`.
#[test]
fn the_gpu_draws_a_tree_at_a_buildings_west_corner_as_the_cpu_does() {
    use glasnost_render::gpu;
    use glasnost_render::map_gpu::{COLOR_FORMAT, MapRenderer, targets};
    use glasnost_render::offscreen::read_texture;
    let required = std::env::var("GLASNOST_REQUIRE_GPU").is_ok_and(|v| v == "1");
    let (device, queue) = match gpu::shared_device() {
        Ok(g) => (g.device, g.queue),
        Err(gpu::SharedGpuError::NoAdapter(e)) if !required => {
            eprintln!("skipping: {e} (set GLASNOST_REQUIRE_GPU=1 to make this fatal)");
            return;
        }
        Err(e) => panic!("a GPU device: {e}"),
    };
    for tree in [(-1, FOOT - 1), (-1, FOOT), (0, FOOT)] {
        let c = case(tree);
        let o = overlap(&c);
        let view = View {
            origin: (o.0, o.1),
            size: (o.2, o.3),
            reference: DEPTH_REFERENCE,
        };
        let q = ComposeQuirks::default();
        let cpu = c.scene.compose(&view, &Filter::default(), &q).to_rgba();
        let mut r = MapRenderer::new(&device, &queue, &c.scene, COLOR_FORMAT).expect("uploads");
        let (color, depth) = targets(&device, view.size, COLOR_FORMAT);
        let mut encoder = device.create_command_encoder(&Default::default());
        r.render(
            &device,
            &queue,
            &mut encoder,
            &color,
            &depth.create_view(&Default::default()),
            &c.scene,
            &view,
            &Filter::default(),
            &q,
        );
        queue.submit([encoder.finish()]);
        let gpu = read_texture(&device, &queue, &color, view.size.0, view.size.1);
        assert!(
            cpu == gpu,
            "{tree:?}: the GPU and the CPU differ over {o:?}"
        );
    }
}

/// #437: the trace itself, printed (`--nocapture`): for each case the tree's cell against the
/// foundation's top-left cell, the draw order and, for the overlap's pixels in the tree's middle
/// column, the depth stored before each writer, each writer's depth and which one shows.
#[test]
#[ignore = "a trace for the notes, not a check"]
fn trace_a_tree_at_a_buildings_west_corner() {
    for (name, tree) in [
        ("behind", (-1, FOOT - 1)),
        ("beside", (-1, FOOT)),
        ("before", (0, FOOT)),
    ] {
        let c = case(tree);
        let pixels = checked(&c);
        let o = overlap(&c);
        let (drawn, covered) = contested(&pixels);
        println!(
            "{name}: tree on C + {tree:?}, foundation {FOOT} x {FOOT} from C; tree item {} \
             (cell pass, {} items), body item {}; overlap {o:?}, {} pixels, the tree drawn at \
             {drawn}, the body over it at {}",
            c.tree,
            c.scene.cell_pass_len,
            c.body,
            pixels.len(),
            covered.len()
        );
        let column = o.0 + o.2 as i32 / 2;
        for p in pixels.iter().filter(|p| p.at.0 == column).step_by(6) {
            println!("  {p:?}");
        }
    }
}
