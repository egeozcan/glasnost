//! The control rings' art (S-0601 claim 18; S-0109 *Changes for other
//! specs*, the S-0601 claim 18 entry): the rings are the rules' ring
//! animation types, played as animations attached to their objects.
//! Invented ring types, vehicles and building, no game data. The tests set
//! the control records by hand and move the game on by its tick alone,
//! calling the drawing's tick step (`Live::after_tick`): no simulation step
//! runs, so the records stay as set.

use std::sync::Arc;

use glasnost_convert::kinds::Kind;
use glasnost_map::{CellGrid, Map};
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE, DepthTest, Frame, Paint, Profile};
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::projection::{height_lift, project_lifted};
use glasnost_render::scene::{Filter, Group, Item, ItemDepth, Scene, View};
use glasnost_rules::sim::{AnimationType, BuildingType, UnitType};
use glasnost_sim::mind::Victim;
use glasnost_sim::rules::{Kind as SimKind, SimRules};
use glasnost_sim::save::state_hash;
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos};

use super::build::{Assets, Inputs, Options, build_parts};
use super::live::{Live, sim_code};
use super::tests::{ART, RULES, art_of, context_full, shp};
use super::tree_depth_tests::shape;

const SIZE: i32 = 24;
/// The ring type RINGX's sprite: a 20 × 20 canvas, four image frames of
/// 6 × 4 at (7, 8).
const RING_CANVAS: (u16, u16) = (20, 20);
const RING_FRAME: (u32, u32) = (6, 4);
const RING_AT: (u16, u16) = (7, 8);
/// The permanent ring type PERMX's sprite: a 30 × 16 canvas, three image
/// frames of 10 × 2 at (10, 7).
const PERMA_CANVAS: (u16, u16) = (30, 16);
const PERMA_FRAME: (u32, u32) = (10, 2);
const PERMA_AT: (u16, u16) = (10, 7);
/// The ring height of vehicle type RV2 (RV1 gives none).
const RV2_HEIGHT: i32 = 300;

/// Which ring types the rules name.
#[derive(Clone, Copy)]
struct Names {
    ring: Option<&'static str>,
    perma: Option<&'static str>,
}

const BOTH: Names = Names {
    ring: Some("RINGX"),
    perma: Some("PERMX"),
};

/// The simulation's rules: the synthetic combat ruleset with the ring
/// types `names`, vehicle types RV1 (no ring height), RV2 (its own) and RV3
/// (a ring height of 0), and HUTX, a 2 × 2 building. RINGX has a random rate whose delays run from 2
/// to 4, so its creation takes a draw (S-0109 claim 3).
fn rules(names: Names) -> Arc<SimRules> {
    let mut r = testkit::combat_ruleset();
    r.registries
        .insert("animations".into(), vec!["RINGX".into(), "PERMX".into()]);
    r.sim.animations.insert(
        "RINGX".into(),
        AnimationType {
            random_rate: Some([450, 225]),
            ..Default::default()
        },
    );
    r.sim.general.controlled_animation_type = names.ring.map(Into::into);
    r.sim.general.perma_controlled_animation_type = names.perma.map(Into::into);
    for (name, height) in [("RV1", None), ("RV2", Some(RV2_HEIGHT)), ("RV3", Some(0))] {
        r.registries.get_mut("vehicles").unwrap().push(name.into());
        let mut u = UnitType {
            strength: Some(300),
            speed: Some(4),
            ..Default::default()
        };
        u.combat.mind_control_ring_offset = height;
        r.sim.vehicles.insert(name.into(), u);
    }
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("HUTX".into());
    let mut hut = BuildingType {
        strength: Some(400),
        foundation: Some([2, 2]),
        ..Default::default()
    };
    // spec: S-0601 r32 claim 18 (the ring-height field of the building's type takes no part) and
    // Import mapping (MindControlRingOffset: not read for a building)
    hut.combat.mind_control_ring_offset = Some(HUT_RING_FIELD);
    r.sim.buildings.insert("HUTX".into(), hut);
    Arc::new(SimRules::new(&r, "synthetic-test"))
}

/// The ring-height field of building type HUTX, which its ring does not
/// read (S-0601 r32 claim 18).
const HUT_RING_FIELD: i32 = 500;
/// The ring type RINGX's art fields by default: endless (S-0109 claim 7).
const ENDLESS: &str = "LoopCount=-1\n";

/// What the drawing reads.
struct Data {
    import: glasnost_convert::Import,
    catalogue: glasnost_convert::theater::Catalogue,
    assets: Assets,
    map: Map,
    palettes: PaletteState,
    options: Options,
    cells: CellGrid,
    rules: Arc<SimRules>,
}

impl Data {
    fn inputs(&self) -> Inputs<'_> {
        Inputs {
            map: &self.map,
            import: &self.import,
            catalogue: &self.catalogue,
            palettes: &self.palettes,
            assets: &self.assets,
            options: &self.options,
            rules: Some(&self.rules),
            cells: Some(&self.cells),
        }
    }
}

/// A live game with the ring types' art, its drawing and its objects.
struct Case {
    data: Data,
    live: Live,
    scene: Scene,
    /// Vehicles of RV1 and RV2 and the building, all of house 0.
    rv1: ObjectId,
    rv2: ObjectId,
    hut: ObjectId,
}

/// The import with both ring types and their art; HUTX's body an image of
/// 120 × 120 from 60 left of and 60 above its draw point, so that it covers
/// the whole foundation and the point above its centre. `hut` is added to
/// HUTX's art section and `ring` is RINGX's.
fn import(
    hut: &str,
    ring: &str,
) -> (
    glasnost_convert::Import,
    glasnost_convert::theater::Catalogue,
    Assets,
) {
    let rules = format!("{RULES}[Animations]\n0=RINGX\n1=PERMX\n");
    let art = ART.replacen("[HUTX]\n", &format!("[HUTX]\n{hut}"), 1);
    let art = format!("{art}[RINGX]\n{ring}[PERMX]\n");
    let (mut import, catalogue, mut assets) = context_full(&rules, &art);
    import
        .arts
        .insert((Kind::Animations, 0), art_of("RINGX", "ringx.shp"));
    import
        .arts
        .insert((Kind::Animations, 1), art_of("PERMX", "permx.shp"));
    assets.insert(b"ringx.shp", shp(RING_CANVAS, 4, RING_FRAME, RING_AT));
    assets.insert(b"permx.shp", shp(PERMA_CANVAS, 3, PERMA_FRAME, PERMA_AT));
    let body = ((120, 120), (150 - 60, 150 - 60), 5);
    assets.insert(
        b"hutx.shp",
        shape(
            (300, 300),
            &[body, body, ((1, 1), (0, 0), 0), ((1, 1), (0, 0), 0)],
        ),
    );
    (import, catalogue, assets)
}

fn case(names: Names) -> Case {
    case_with(names, "", ENDLESS)
}

/// The case with `hut` added to HUTX's art section and `ring` as RINGX's.
fn case_with(names: Names, hut: &str, ring: &str) -> Case {
    let (import, catalogue, assets) = import(hut, ring);
    let rules = rules(names);
    let setup = testkit::setup(7);
    let mut world = glasnost_sim::load::new_game(
        rules.clone(),
        &testkit::empty_map(SIZE),
        testkit::catalogue(),
        &setup,
    )
    .expect("the synthetic game loads");
    let rv1 = testkit::spawn(
        &mut world,
        SimKind::Vehicle,
        "RV1",
        0,
        testkit::mid(SIZE, -4, 2),
        0,
    );
    let rv2 = testkit::spawn(
        &mut world,
        SimKind::Vehicle,
        "RV2",
        0,
        testkit::mid(SIZE, 4, 2),
        0,
    );
    let hut = testkit::spawn(
        &mut world,
        SimKind::Building,
        "HUTX",
        0,
        testkit::mid(SIZE, 0, -4),
        0,
    );
    let cells = world.cells.grid.clone();
    let data = Data {
        import,
        catalogue,
        assets,
        map: testkit::empty_map(SIZE),
        palettes: PaletteState::at_program_start(None, None, PaletteQuirks::default()),
        options: Options::default(),
        cells,
        rules: rules.clone(),
    };
    let inputs = data.inputs();
    let (mut built, painter) = build_parts(&inputs, true);
    let mut live = Live::new(
        &inputs,
        &mut built,
        painter,
        (world, rules.clone(), setup, Vec::new()),
    );
    live.set_debug_markers(false);
    let mut scene = built.scene;
    live.redraw(&inputs, &mut scene);
    Case {
        data,
        live,
        scene,
        rv1,
        rv2,
        hut,
    }
}

/// Which ring type an item is of, by its image's size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Of {
    Ring,
    Perma,
}

/// A ring item drawn: its type, its place and its image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Drawn {
    of: Of,
    at: (i32, i32),
    image: u32,
}

impl Case {
    /// One tick of the game as the drawing sees it: the tick number moves
    /// on, then the drawing's tick step and a redraw. Checks that neither
    /// changes the game's state.
    fn tick(&mut self) {
        self.live.world.tick += 1;
        let before = state_hash(&self.live.world);
        self.live.after_tick();
        let inputs = self.data.inputs();
        self.live.redraw(&inputs, &mut self.scene);
        assert_eq!(
            state_hash(&self.live.world),
            before,
            "the rings' drawing changes no game state"
        );
    }

    fn ticks(&mut self, n: usize) {
        for _ in 0..n {
            self.tick();
        }
    }

    fn control(&mut self, id: ObjectId) {
        self.live.world.mind.victims.insert(
            id,
            Victim {
                controller: self.hut,
                original_owner: None,
                line_until: 0,
            },
        );
    }

    fn make_permanent(&mut self, id: ObjectId) {
        self.live.world.mind.permanent.insert(id);
    }

    /// The ring items of the scene, in drawing order.
    fn rings(&self) -> Vec<Drawn> {
        self.scene
            .items
            .iter()
            .filter(|i| i.group == Group::Effects)
            .filter_map(|i| {
                let img = &self.scene.images[i.image as usize];
                let size = (img.width(), img.height());
                let of = if size == RING_FRAME {
                    Of::Ring
                } else if size == PERMA_FRAME {
                    Of::Perma
                } else {
                    return None;
                };
                Some(Drawn {
                    of,
                    at: (i.x, i.y),
                    image: i.image,
                })
            })
            .collect()
    }

    /// The one ring item drawn.
    fn ring(&self) -> Drawn {
        let r = self.rings();
        assert_eq!(r.len(), 1, "one ring drawn: {r:?}");
        r[0]
    }

    fn pos(&self, id: ObjectId) -> Pos {
        self.live.world.obj(id).unwrap().pos
    }
}

/// Where a ring of type `of` at `at` draws its image frames (S-0109 claim
/// 26, S-0102 F6: the position's screen pixel with its height, less half
/// the canvas, plus the frame's place on it).
fn placed(of: Of, at: Pos) -> (i32, i32) {
    let (canvas, frame) = match of {
        Of::Ring => (RING_CANVAS, RING_AT),
        Of::Perma => (PERMA_CANVAS, PERMA_AT),
    };
    let p = project_lifted(at.x, at.y, at.z);
    (
        p.0 - i32::from(canvas.0) / 2 + i32::from(frame.0),
        p.1 - i32::from(canvas.1) / 2 + i32::from(frame.1),
    )
}

fn raised(p: Pos, h: i32) -> Pos {
    Pos::new(p.x, p.y, p.z + h)
}

/// spec: S-0601 claim 18 (a controlled object carries the rules' controlled-animation type,
/// raised above a mobile object by its type's ring height in leptons) and Constants (the default
/// ring height, 140 leptons, for a type without the key); S-0109 claims 22 and 26
#[test]
fn claim_18_a_controlled_vehicle_shows_the_ring_type_raised_by_its_ring_height() {
    let mut c = case(BOTH);
    assert!(c.rings().is_empty(), "no ring before any control");
    c.control(c.rv1);
    c.control(c.rv2);
    c.tick();
    let mut want = vec![
        // spec: S-0601 Constants (default ring height 140 leptons)
        (Of::Ring, placed(Of::Ring, raised(c.pos(c.rv1), 140))),
        (Of::Ring, placed(Of::Ring, raised(c.pos(c.rv2), RV2_HEIGHT))),
    ];
    let mut got: Vec<(Of, (i32, i32))> = c.rings().iter().map(|d| (d.of, d.at)).collect();
    want.sort_by_key(|w| w.1);
    got.sort_by_key(|g| g.1);
    assert_eq!(got, want);
    assert_eq!(c.live.counts.anims, 2, "one animation per ring");
}

/// spec: S-0601 claim 18 (a permanently controlled object carries the permanent-controlled-animation
/// type, also when it is controlled as well) and design: render (#122) (the rings come from the
/// control records alone: none for a record whose object is gone or is off the map)
#[test]
fn claim_18_a_permanently_controlled_object_shows_the_permanent_type() {
    let mut c = case(BOTH);
    c.make_permanent(c.rv1);
    c.control(c.rv2);
    c.make_permanent(c.rv2);
    c.control(ObjectId(999_999));
    let away = testkit::spawn(
        &mut c.live.world,
        SimKind::Vehicle,
        "RV1",
        0,
        testkit::mid(SIZE, 0, 6),
        0,
    );
    c.control(away);
    c.live.world.obj_mut(away).unwrap().on_map = false;
    c.tick();
    let mut got: Vec<(Of, (i32, i32))> = c.rings().iter().map(|d| (d.of, d.at)).collect();
    got.sort_by_key(|g| g.1);
    let mut want = vec![
        (Of::Perma, placed(Of::Perma, raised(c.pos(c.rv1), 140))),
        (
            Of::Perma,
            placed(Of::Perma, raised(c.pos(c.rv2), RV2_HEIGHT)),
        ),
    ];
    want.sort_by_key(|w| w.1);
    assert_eq!(got, want);
    assert_eq!(c.live.counts.anims, 2);
}

/// spec: S-0601 claim 18 (attached to the object and following it) and S-0109 claim 22 (its
/// position is the owner's plus the displacement it had when attached)
#[test]
fn claim_18_the_ring_follows_its_object() {
    let mut c = case(BOTH);
    c.control(c.rv1);
    c.tick();
    let first = c.ring();
    let p = c.pos(c.rv1);
    let moved = Pos::new(p.x + 300, p.y - 100, p.z + 40);
    c.live.world.obj_mut(c.rv1).unwrap().pos = moved;
    c.tick();
    let now = c.ring();
    assert_eq!(now.at, placed(Of::Ring, raised(moved, 140)));
    assert_ne!(now.at, first.at);
    assert_eq!(c.live.counts.anims, 1, "the same animation moved");
}

/// spec: S-0601 claim 18 (the ring is removed at release, claim 9; when the object leaves the game
/// the ring goes with it) and S-0109 *Changes for other specs* (the S-0601 claim 18 entry: it ends
/// at each event of claim 22's *End with the owner* list: destroyed, gone off the map, removed)
#[test]
fn claim_18_the_ring_goes_at_release_and_with_its_object() {
    let mut c = case(BOTH);
    c.control(c.rv1);
    c.control(c.rv2);
    c.tick();
    assert_eq!(c.rings().len(), 2);
    c.live.world.mind.victims.remove(&c.rv1);
    c.tick();
    assert_eq!(
        c.ring().at,
        placed(Of::Ring, raised(c.pos(c.rv2), RV2_HEIGHT))
    );
    c.live.world.obj_mut(c.rv2).unwrap().combat.killed = true;
    c.tick();
    assert!(c.rings().is_empty(), "destroyed: no ring");
    c.live.world.obj_mut(c.rv2).unwrap().combat.killed = false;
    c.tick();
    assert_eq!(c.rings().len(), 1, "a ring again while it is in play");
    c.live.world.objects.remove(&c.rv2);
    c.tick();
    assert!(c.rings().is_empty(), "removed from the game: no ring");
    c.ticks(3);
    assert!(c.rings().is_empty());
}

/// spec: S-0109 *Changes for other specs* (the S-0601 claim 18 entry: our engine shows the ring
/// again whenever the controlled object is on the map, not destroyed and neither cloaking nor
/// cloaked; the ring shown again is a new animation, created as at the start of the control, its
/// creation and playback draws from the unsynchronised stream, changing no game state) and S-0109
/// claims 1, 3 and 5 (a new playback starts at its first image frame, with claim 3's draw)
#[test]
fn claim_18_each_return_of_the_ring_starts_a_new_playback() {
    use glasnost_sim::cloak::CloakState;
    let mut c = case(BOTH);
    c.control(c.rv1);
    let start = c.live.cosmetic().clone();
    c.tick();
    let first = c.ring().image;
    assert_ne!(
        c.live.cosmetic(),
        &start,
        "the ring's creation takes its random-rate draw from the cosmetic stream"
    );
    c.ticks(6);
    assert_ne!(c.ring().image, first, "the ring has advanced");
    type Change = fn(&mut glasnost_sim::world::Object, bool);
    let cloak: Change = |o, hide| {
        o.cloak.state = if hide {
            CloakState::Cloaking
        } else {
            CloakState::Uncloaked
        }
    };
    let cloaked: Change = |o, hide| {
        o.cloak.state = if hide {
            CloakState::Cloaked
        } else {
            CloakState::Uncloaking
        }
    };
    let off_map: Change = |o, hide| o.on_map = !hide;
    for (what, change) in [
        ("cloaking", cloak),
        ("cloaked", cloaked),
        ("off the map", off_map),
    ] {
        change(c.live.world.obj_mut(c.rv1).unwrap(), true);
        c.tick();
        assert!(c.rings().is_empty(), "{what}: no ring");
        let before = c.live.cosmetic().clone();
        change(c.live.world.obj_mut(c.rv1).unwrap(), false);
        c.tick();
        assert_eq!(c.ring().image, first, "{what}, back: a new playback");
        assert_ne!(
            c.live.cosmetic(),
            &before,
            "{what}, back: its draws come from the cosmetic stream"
        );
        c.ticks(6);
        assert_ne!(c.ring().image, first, "{what}, back: it plays");
    }
    assert_eq!(c.live.counts.anims, 4, "one animation for each showing");
}

/// spec: S-0109 *Changes for other specs* (the S-0601 claim 18 entry: the ring is drawn, advances
/// and ends as S-0109 gives for the ring's type) and S-0109 claims 7 and 16 (a type with one pass
/// ends after it): an ended ring does not start again while its object stays controlled
#[test]
fn claim_18_a_ring_whose_type_ends_stays_gone_while_the_control_lasts() {
    let mut c = case(BOTH);
    c.make_permanent(c.rv1);
    c.tick();
    assert_eq!(c.ring().of, Of::Perma);
    c.ticks(6);
    assert!(c.rings().is_empty(), "the one pass has ended");
    c.ticks(10);
    assert!(c.rings().is_empty(), "and it does not start again");
    assert_eq!(c.live.counts.anims, 1);
}

/// spec: S-0601 claim 18 (the ring is the rules' type): with no type named nothing is drawn, and a
/// type the art does not list is an animation without a sprite, which draws nothing (S-0109 Edge
/// cases)
#[test]
fn claim_18_no_ring_is_drawn_without_a_type_or_its_art() {
    let mut c = case(Names {
        ring: None,
        perma: Some("GHOSTX"),
    });
    c.control(c.rv1);
    c.make_permanent(c.rv2);
    c.ticks(2);
    assert!(c.rings().is_empty());
    assert_eq!(c.live.counts.anims_drawn, 0);
    assert_eq!(c.live.counts.anims, 1, "only the permanent ring is played");
    assert_eq!(c.live.counts.anims_without_sprite, 1, "with no sprite");
}

/// The building case: HUTX controlled, its body and bib painted with one colour and its ring with
/// another; the ring's item index and the view rectangle of its image.
struct Front {
    c: Case,
    ring: usize,
    rect: (i32, i32, u32, u32),
}

fn front() -> Front {
    front_with("", ENDLESS)
}

/// The building case with `hut` added to HUTX's art section and `ring` as
/// RINGX's.
fn front_with(hut: &str, ring: &str) -> Front {
    let mut c = case_with(BOTH, hut, ring);
    c.control(c.hut);
    c.tick();
    let ring = c.ring();
    let items: Vec<usize> = (0..c.scene.items.len())
        .filter(|&i| {
            let it = &c.scene.items[i];
            it.group == Group::Effects && it.image == ring.image && (it.x, it.y) == ring.at
        })
        .collect();
    assert_eq!(items.len(), 1);
    let at = glasnost_sim::mind::ring(&c.live.world, c.hut).unwrap().at;
    let p = c.pos(c.hut);
    // spec: S-0601 r32 claim 18 (the building's centre point, S-0204 claim 4: for a 2 × 2
    // foundation its position plus (128, 128); the type's ring-height field takes no part). The
    // simulation gives it at the building's own height: the raise by 104 leptons per art height
    // level is a seam in `glasnost_sim::mind::ring`.
    assert_eq!(at, Pos::new(p.x + 128, p.y + 128, p.z));
    assert_eq!(
        ring.at,
        placed(Of::Ring, at),
        "the ring is drawn at the simulation's point for the building"
    );
    let white = c.scene.add_row([0xFFFF; 256]);
    let red = c.scene.add_row([0xF800; 256]);
    let code = sim_code(c.hut);
    for it in &mut c.scene.items {
        if it.object == Some(code) {
            it.paint = Paint::Row(white);
        }
    }
    c.scene.items[items[0]].paint = Paint::Row(red);
    let rect = (ring.at.0, ring.at.1, RING_FRAME.0, RING_FRAME.1);
    Front {
        c,
        ring: items[0],
        rect,
    }
}

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

/// The ring's pixels in `f`, against the ring drawn alone: how many there
/// are and how many of them `f` shows in the ring's colour.
fn shown(fr: &Front, f: &Frame) -> (usize, usize) {
    let s = &fr.c.scene;
    let alone = compose(s, vec![s.items[fr.ring]], fr.rect);
    let empty = compose(s, Vec::new(), fr.rect);
    let ring: Vec<usize> = (0..alone.color.len())
        .filter(|&i| alone.color[i] != empty.color[i])
        .collect();
    let showing = ring
        .iter()
        .filter(|&&i| f.color[i] == alone.color[i])
        .count();
    (ring.len(), showing)
}

/// The depth a ring item of an ordinary animation row has at `at` with the
/// depth adjustment `adjust` and no vertical draw shift (S-0102 claim 13).
fn ring_depth(at: Pos, adjust: i32) -> ItemDepth {
    ItemDepth::Profile {
        profile: Profile::Upright,
        bias: adjust - height_lift(at.z) - 2,
        companion: None,
    }
}

/// spec: S-0601 r32 claim 18 (a building's ring is an ordinary attached animation in the ground
/// layer and a depth reader, S-0102 claim 10; the takeover replaces its animation depth adjustment
/// by −1024, whatever the ring type's own is: its bias is the ring type's vertical draw shift −
/// 1024 − L(z) − 2) and Constants (building ring depth adjustment −1024); S-0102 claim 13 (the
/// animation's depth adjustment unless its creator replaced it): a mobile object's ring keeps its
/// type's own
#[test]
fn claim_18_a_buildings_ring_takes_the_depth_adjustment_minus_1024() {
    let mut c = case_with(BOTH, "", "LoopCount=-1\nZAdjust=2000\n");
    c.control(c.hut);
    c.control(c.rv1);
    c.tick();
    let hut_at = glasnost_sim::mind::ring(&c.live.world, c.hut).unwrap().at;
    let rv1_at = raised(c.pos(c.rv1), 140);
    let item = |at: Pos| {
        let p = placed(Of::Ring, at);
        let r = c
            .rings()
            .into_iter()
            .find(|d| d.at == p)
            .expect("the ring is drawn");
        c.scene
            .items
            .iter()
            .find(|i| i.group == Group::Effects && i.image == r.image && (i.x, i.y) == p)
            .cloned()
            .unwrap()
    };
    let hut = item(hut_at);
    assert_eq!(
        hut.test,
        DepthTest::Reader,
        "the building's ring reads depth"
    );
    // spec: S-0601 r32 Constants (building ring depth adjustment −1024 depth units)
    assert_eq!(hut.depth, ring_depth(hut_at, -1024));
    let rv1 = item(rv1_at);
    assert_eq!(rv1.test, DepthTest::Reader);
    assert_eq!(
        rv1.depth,
        ring_depth(rv1_at, 2000),
        "a vehicle's ring keeps its type's own depth adjustment"
    );
}

/// spec: S-0601 r32 claim 18 (in front of the building: its body, a depth writer, hides a ring
/// pixel only where its stored depth is not greater than the ring pixel's, S-0102 F2; with the
/// building's own depth adjustment at 0 its body less than 700 rows tall hides none of the ring;
/// the ring is drawn after the body) and S-0102 claims 10 and 13
#[test]
fn claim_18_a_buildings_ring_is_drawn_in_front_of_it() {
    let fr = front();
    let s = &fr.c.scene;
    let body = s
        .items
        .iter()
        .position(|i| i.object == Some(sim_code(fr.c.hut)) && i.group == Group::Buildings)
        .expect("the building's body is drawn");
    assert!(body < fr.ring, "the ring is drawn after the body");
    assert_eq!(s.items[fr.ring].test, DepthTest::Reader);
    let (pixels, showing) = shown(&fr, &compose(s, s.items.clone(), fr.rect));
    assert!(pixels > 0, "the ring has pixels");
    assert_eq!(showing, pixels, "every ring pixel shows over the body");
    // With the ring type's own depth adjustment (0) in place of −1024, the body's depth would
    // hide some of it there (S-0102 claim 10).
    let at = glasnost_sim::mind::ring(&fr.c.live.world, fr.c.hut)
        .unwrap()
        .at;
    let mut items = s.items.clone();
    items[fr.ring].depth = ring_depth(at, 0);
    let (_, own) = shown(&fr, &compose(s, items, fr.rect));
    assert!(
        own < pixels,
        "with its type's adjustment the body hides some of it"
    );
}

/// spec: S-0601 r32 claim 18 (the ring's depth against the body's under S-0102 F2: the ring shows
/// at a pixel where the building's normal depth adjustment n puts the body farther, and the body
/// where n puts it nearer; the ring type's own depth adjustment changes nothing) and S-0601
/// scenario MC13 part c (c1: n = −524 and n = −1524, the ring type's own adjustment at 0 and at
/// 2000); S-0102 claim 13 (the body's bias −L(z) − 2 + the normal depth adjustment)
#[test]
fn claim_18_the_buildings_depth_adjustment_decides_against_its_ring() {
    for ring in [ENDLESS, "LoopCount=-1\nZAdjust=2000\n"] {
        let far = front_with("NormalZAdjust=-524\n", ring);
        let s = &far.c.scene;
        let (pixels, showing) = shown(&far, &compose(s, s.items.clone(), far.rect));
        assert!(pixels > 0);
        assert_eq!(showing, pixels, "n = −524 ({ring:?}): the ring shows");
        let near = front_with("NormalZAdjust=-1524\n", ring);
        let s = &near.c.scene;
        let (pixels, showing) = shown(&near, &compose(s, s.items.clone(), near.rect));
        assert!(pixels > 0);
        assert_eq!(showing, 0, "n = −1524 ({ring:?}): the body hides the ring");
    }
}

/// spec: S-0601 r32 claim 18 (a building's ring is a depth reader in the ground layer ordered by X +
/// Y of the centre point plus the ring type's sort adjustment, S-0102 claim 5: where another depth
/// reader overlaps it, the later one in the ground layer's order shows, S-0102 claim 6; a reader
/// with a larger key, such as a unit in front of the building, is drawn after it) and S-0102 claim
/// 10 (readers never store depth)
#[test]
fn claim_18_a_buildings_ring_orders_with_the_ground_layers_readers_by_its_key() {
    let mut c = case(BOTH);
    c.control(c.hut);
    let at = glasnost_sim::mind::ring(&c.live.world, c.hut).unwrap().at;
    // Vehicles with a ring height of 0 at the ring's point moved by d leptons on both map axes:
    // their rings overlap the building's on screen, with keys 2d from its key.
    let mut ringed = |d: i32, cell: i32| {
        let id = testkit::spawn(
            &mut c.live.world,
            SimKind::Vehicle,
            "RV3",
            0,
            testkit::mid(SIZE, cell, 6),
            0,
        );
        c.live.world.obj_mut(id).unwrap().pos = Pos::new(at.x + d, at.y + d, at.z);
        id
    };
    let behind = ringed(-32, -2);
    let ahead = ringed(32, 2);
    c.control(behind);
    c.control(ahead);
    c.tick();
    let index = |p: Pos| {
        let p = placed(Of::Ring, p);
        c.scene
            .items
            .iter()
            .position(|i| i.group == Group::Effects && (i.x, i.y) == p)
            .expect("the ring is drawn")
    };
    let (b, h, a) = (index(c.pos(behind)), index(at), index(c.pos(ahead)));
    assert!(
        b < h,
        "a reader with a smaller key is drawn before the building's ring"
    );
    assert!(
        h < a,
        "a reader with a larger key is drawn after the building's ring"
    );
    for i in [b, h, a] {
        assert_eq!(c.scene.items[i].test, DepthTest::Reader);
    }
}

/// spec: S-0601 r32 claim 18 (the takeover creates the controlled ring with a repeat factor of 1,
/// so a ring type whose loop count is −1 plays until the ring ends) and S-0109 claim 7 (with a
/// factor other than 1 it would stop after at most 254 passes) and S-0601 scenario MC13 part a
/// (the ring still shows after more than 254 passes)
#[test]
fn claim_18_an_endless_ring_type_plays_on_past_254_passes() {
    let mut c = case(BOTH);
    c.control(c.hut);
    c.tick();
    let first = c.ring();
    // RINGX's four stages last at most 4 ticks each, so 255 passes take at most 4080 ticks.
    for _ in 0..4200 {
        c.live.world.tick += 1;
        c.live.after_tick();
    }
    let inputs = c.data.inputs();
    c.live.redraw(&inputs, &mut c.scene);
    assert_eq!(c.ring().at, first.at, "the ring still shows");
    assert_eq!(c.live.counts.anims, 1, "the same ring");
}

/// spec: S-0601 r32 claim 18 (a ring shown again is a display-only animation: its end makes no
/// follow-up switch; it is drawn, advances and ends as S-0109 gives for the ring's type otherwise)
/// and S-0109 claim 15 (the ring of a control's start is an ordinary animation: its last pass ends
/// with the follow-up switch). Seam: the ring of a control's start is the original's own
/// animation, which the drawing plays in the simulation's place (S-0109 claim 28)
#[test]
fn claim_18_a_ring_shown_again_makes_no_follow_up_switch() {
    let mut c = case_with(BOTH, "", "LoopCount=1\nNext=PERMX\n");
    c.control(c.rv1);
    c.tick();
    assert_eq!(c.ring().of, Of::Ring);
    let mut switched = false;
    for _ in 0..40 {
        c.tick();
        switched |= c.rings().iter().any(|d| d.of == Of::Perma);
    }
    assert!(switched, "the first ring switches to its follow-up type");
    assert!(c.rings().is_empty(), "and the follow-up type ends");
    c.live.world.obj_mut(c.rv1).unwrap().on_map = false;
    c.tick();
    c.live.world.obj_mut(c.rv1).unwrap().on_map = true;
    c.tick();
    assert_eq!(c.ring().of, Of::Ring, "the ring is shown again");
    for _ in 0..40 {
        c.tick();
        assert!(
            c.rings().iter().all(|d| d.of == Of::Ring),
            "a ring shown again does not switch to its follow-up type"
        );
    }
    assert!(c.rings().is_empty(), "it ends after its pass");
}

/// spec: S-0601 r32 claim 18; S-0102 claims 10 and 13: the GPU map path draws a building's ring,
/// a depth reader with the replaced depth adjustment, as the CPU reference does.
/// Needs a GPU adapter (software is fine); skipped without one unless `GLASNOST_REQUIRE_GPU=1`.
#[test]
fn the_gpu_draws_a_buildings_ring_as_the_cpu_does() {
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
    let fr = front();
    let s = &fr.c.scene;
    let view = View {
        origin: (fr.rect.0, fr.rect.1),
        size: (fr.rect.2, fr.rect.3),
        reference: DEPTH_REFERENCE,
    };
    let q = ComposeQuirks::default();
    let cpu = s.compose(&view, &Filter::default(), &q).to_rgba();
    let mut r = MapRenderer::new(&device, &queue, s, COLOR_FORMAT).expect("uploads");
    let (color, depth) = targets(&device, view.size, COLOR_FORMAT);
    let mut encoder = device.create_command_encoder(&Default::default());
    r.render(
        &device,
        &queue,
        &mut encoder,
        &color,
        &depth.create_view(&Default::default()),
        s,
        &view,
        &Filter::default(),
        &q,
    );
    queue.submit([encoder.finish()]);
    let gpu = read_texture(&device, &queue, &color, view.size.0, view.size.1);
    assert!(cpu == gpu, "the GPU and the CPU differ over {:?}", fr.rect);
}
