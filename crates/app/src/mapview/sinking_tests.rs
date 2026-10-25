//! Tests of the sinking vessel's list, our interim of Q-1871, in the view:
//! what the view reads from the game state, and the images a listed body,
//! turret and shadow get. Every type name and number is made up.

use glasnost_convert::art::TypeArt;
use glasnost_convert::kinds::Kind;
use glasnost_convert::map::{DecodeContext, decode};
use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
use glasnost_ini::{Document, Quirks};
use glasnost_render::compose::Paint;
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::scene::Scene;
use glasnost_sim::rules::Kind as SimKind;
use glasnost_sim::testkit;
use glasnost_sim::world::ObjectId;

use super::build::{Attitude, Inputs, Mobile, Options, build_parts};
use super::live::{attitude, sim_code};
use super::sinking::{STEPS, list};
use super::tests::{RULES, context_with, scene_map_text};

// design: Q-1871 interim (the view reads the sinking flag, S-0504 Terms, and the height of S-0504 claim 4; no state of its own)
#[test]
fn the_view_lists_a_vehicle_marked_sinking_by_its_depth() {
    let mut w = testkit::game_with(&testkit::combat_ruleset(), 24, 3);
    let cart = testkit::spawn(
        &mut w,
        SimKind::Vehicle,
        "CART",
        0,
        testkit::mid(24, 0, 0),
        16384,
    );
    let read = |w: &glasnost_sim::world::World| attitude(w, w.obj(cart).unwrap());
    assert_eq!(read(&w), Attitude::None, "not sinking: heading only");
    glasnost_sim::naval::start_sinking(&mut w, cart);
    let surface = {
        let o = w.obj(cart).unwrap();
        w.surface_at(o.pos.x, o.pos.y)
    };
    assert_eq!(read(&w), Attitude::Sinking(list(0, cart.0)));
    let Attitude::Sinking(l) = read(&w) else {
        unreachable!()
    };
    assert!(l.level(), "level at the start of the descent");
    for (depth, step) in [(60, 3), (120, 6), (240, STEPS), (395, STEPS)] {
        w.obj_mut(cart).unwrap().pos.z = surface - depth;
        let Attitude::Sinking(l) = read(&w) else {
            panic!("sinking at depth {depth}")
        };
        assert_eq!(l.step, step, "depth {depth}");
        assert_eq!(l, list(-depth, cart.0));
    }
    // The view changed nothing in the game.
    assert!(glasnost_sim::naval::sinking(&w, cart));
}

/// The images (as pixels) of the body, turret and shadow a vehicle type
/// with a turret draws for each of `attitudes` in turn, all in one painter.
fn images(attitudes: &[Attitude]) -> Vec<Vec<glasnost_fmt::image::IndexedImage>> {
    let rules = format!("{RULES}[CARX]\nTurret=yes\n");
    let (mut import, catalogue, mut assets) = context_with(&rules);
    import.arts.insert(
        (Kind::Vehicles, 0),
        TypeArt {
            voxels: [
                ("body".to_string(), b"hull.vxl".to_vec()),
                ("turret".to_string(), b"hull.vxl".to_vec()),
            ]
            .into(),
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
    let mut out = Vec::new();
    for &attitude in attitudes {
        let m = Mobile {
            kind: Kind::Vehicles,
            idx: 0,
            owner: b"Neutral".to_vec(),
            pos: (3 * 256 + 128, 3 * 256 + 128, 0),
            body: 8192,
            turret: 24576,
            elevation: 16384,
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
        };
        let (entries, _) = painter.mobiles(&inputs, &mut scene, &[m]);
        let mut parts: Vec<_> = entries[0]
            .items
            .iter()
            .filter(|i| matches!(i.paint, Paint::Lit(_) | Paint::Shadow))
            .map(|i| scene.images[i.image as usize].clone())
            .collect();
        parts.sort_by_key(|i| (i.width(), i.height()));
        out.push(parts);
    }
    out
}

// design: Q-1871 interim (a listed body, turret and shadow are drawn from the listed pose and kept apart from the level images: neither is reused for the other)
#[test]
fn a_listed_vessel_has_images_of_its_own() {
    let start = Attitude::Sinking(list(0, 5));
    let mid = Attitude::Sinking(list(-120, 5));
    let full = Attitude::Sinking(list(-400, 5));
    // Level first, then listed, then level again, in one painter.
    let a = images(&[Attitude::None, mid, Attitude::None, start, full]);
    let level = &a[0];
    assert_eq!(level.len(), 3, "a body, a turret and a shadow");
    assert_eq!(
        a[2], *level,
        "a level vessel after a listed one is unchanged"
    );
    assert_eq!(a[3], *level, "at the start of the descent it is level");
    assert_eq!(a[1].len(), 3);
    for (k, listed) in [(1, &a[1]), (4, &a[4])] {
        for (i, img) in listed.iter().enumerate() {
            assert!(!level.contains(img), "drawing {k}, part {i}: listed");
        }
    }
    assert_ne!(a[1], a[4], "mid-descent and full lists differ");
    // Listed first, in a fresh painter: the level images that follow are
    // the same as when nothing was listed before them.
    let b = images(&[mid, Attitude::None]);
    assert_eq!(b[0], a[1], "a listed image depends on the list alone");
    assert_eq!(
        b[1], *level,
        "a level vessel after a listed one is unchanged"
    );
}
