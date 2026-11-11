//! Tests of a sinking vehicle's lean in the view (S-0504 claims 15 and
//! 16): what the view reads from the game state, and the images a leaning
//! body, turret and shadow get. Every type name and number is made up.

use glasnost_convert::art::TypeArt;
use glasnost_convert::kinds::Kind;
use glasnost_convert::map::{DecodeContext, decode};
use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
use glasnost_ini::{Document, Quirks};
use glasnost_render::compose::Paint;
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::scene::Scene;
use glasnost_rules::sim::Locomotor;
use glasnost_sim::rules::Kind as SimKind;
use glasnost_sim::testkit;
use glasnost_sim::world::ObjectId;

use super::build::{Attitude, Inputs, Mobile, Options, build_parts};
use super::live::{attitude, sim_code};
use super::tests::{RULES, context_with, scene_map_text};

/// A world with one vehicle of the synthetic type, whose type locomotor is
/// `loco`, marked sinking.
fn sinking_world(loco: Locomotor) -> (glasnost_sim::world::World, ObjectId) {
    let mut r = testkit::combat_ruleset();
    r.sim.vehicles.get_mut("CART").unwrap().locomotor = Some(loco);
    let mut w = testkit::game_with(&r, 24, 3);
    let cart = testkit::spawn(
        &mut w,
        SimKind::Vehicle,
        "CART",
        0,
        testkit::mid(24, 0, 0),
        16384,
    );
    (w, cart)
}

// spec: S-0504 claim 15 and F4 (the pitch the simulation steps at each descent; the view holds no state of its own), claim 16 (the lean drawn for the driving, ship, hovering and jumpjet locomotors), Terms (sinking)
#[test]
fn the_view_reads_the_pitch_the_simulation_steps() {
    let (mut w, cart) = sinking_world(Locomotor::Ship);
    let read = |w: &glasnost_sim::world::World| attitude(w, w.obj(cart).unwrap());
    assert_eq!(read(&w), Attitude::None, "not sinking: heading only");
    glasnost_sim::naval::start_sinking(&mut w, cart);
    assert_eq!(
        read(&w),
        Attitude::Sinking {
            pitch: 0,
            roll: 0,
            drawn: true
        },
        "level before the first descent, and marked sinking"
    );
    for k in 1..=3 {
        assert!(!glasnost_sim::naval::sinking_turn(&mut w, cart));
        let p = glasnost_sim::naval::pitch(&w, cart);
        assert_ne!(p.0, 0, "descent {k} leans the vessel");
        assert_eq!(
            read(&w),
            Attitude::Sinking {
                pitch: p.0,
                roll: 0,
                drawn: true
            },
            "descent {k}"
        );
    }
    // Reading changed nothing in the game.
    let before = glasnost_sim::naval::pitch(&w, cart);
    let _ = read(&w);
    assert_eq!(glasnost_sim::naval::pitch(&w, cart), before);
    assert!(glasnost_sim::naval::sinking(&w, cart));
}

// spec: S-0504 claim 16 (only the driving, ship, hovering and jumpjet locomotors draw the lean; S-0106 claims 5 and 9); S-0207 claim 14 (the type's locomotor, teleporting when absent)
#[test]
fn only_the_rocking_locomotors_draw_the_lean() {
    for (loco, drawn) in [
        (Locomotor::Drive, true),
        (Locomotor::Ship, true),
        (Locomotor::Hover, true),
        (Locomotor::Jumpjet, true),
        (Locomotor::Mech, false),
        (Locomotor::Walk, false),
        (Locomotor::Teleport, false),
        (Locomotor::Tunnel, false),
    ] {
        let (mut w, cart) = sinking_world(loco);
        glasnost_sim::naval::start_sinking(&mut w, cart);
        assert!(!glasnost_sim::naval::sinking_turn(&mut w, cart));
        let Attitude::Sinking {
            drawn: d, pitch, ..
        } = attitude(&w, w.obj(cart).unwrap())
        else {
            panic!("{loco:?}: sinking")
        };
        assert_eq!(d, drawn, "{loco:?}");
        assert_ne!(pitch, 0, "{loco:?}: the pitch still steps (claim 16)");
    }
}

/// The images (as pixels) of the body, turret and shadow a vehicle type
/// with a turret draws for each of `attitudes` in turn, all in one painter,
/// with how many shadow items each drawing has.
pub(super) fn images(
    attitudes: &[Attitude],
) -> Vec<(Vec<glasnost_fmt::image::IndexedImage>, usize)> {
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
            walk: Default::default(),
            deck: false,
        };
        let (entries, _) = painter.mobiles(&inputs, &mut scene, &[m]);
        let items = &entries[0].items;
        let shadows = items
            .iter()
            .filter(|i| matches!(i.paint, Paint::Shadow))
            .count();
        let mut parts: Vec<_> = items
            .iter()
            .filter(|i| matches!(i.paint, Paint::Lit(_) | Paint::Shadow))
            .map(|i| scene.images[i.image as usize].clone())
            .collect();
        parts.sort_by_key(|i| (i.width(), i.height()));
        out.push((parts, shadows));
    }
    out
}

/// A sinking attitude with pitch `p` (radians) and a level roll.
fn sinking(p: f32, drawn: bool) -> Attitude {
    Attitude::Sinking {
        pitch: p.to_bits(),
        roll: 0,
        drawn,
    }
}

// spec: S-0504 claim 16 (no shadow from the first drawn frame after sinking begins; the body, turret and barrel lean on F5's D, S-0105 claim 8); S-0105 claim 26 and S-0106 claim 13 (a rocking body, |p| or |r| at least 0.005, is drawn fresh, a settled one from the cache); S-0504 F4 (the full lean, 0.785398185253143310546875)
#[test]
fn a_leaning_vessel_has_images_of_its_own_and_no_shadow() {
    let mid = sinking(0.4, true);
    // The simulation's full lean, 79 steps from level.
    let lean = (0..79).fold(glasnost_num::F32::ZERO, |p, _| {
        glasnost_sim::naval::pitch_step(
            p,
            glasnost_sim::naval::Lean::BowDown,
            glasnost_num::Rounding::TowardZero,
        )
    });
    assert_eq!(lean.0, 0x3F49_0FDB, "L in single precision");
    let full = sinking(f32::from_bits(lean.0), true);
    // Level first, then leaning, then level again, in one painter.
    let a = images(&[
        Attitude::None,
        mid,
        Attitude::None,
        sinking(0.0, true),
        full,
        sinking(0.004, true),
        sinking(-0.4, true),
    ]);
    let (level, level_shadows) = &a[0];
    assert_eq!(level.len(), 3, "a body, a turret and a shadow");
    assert_eq!(*level_shadows, 1);
    assert_eq!(
        a[2], a[0],
        "a level vessel after a leaning one is unchanged"
    );
    let settled = &a[3].0;
    assert_eq!(settled.len(), 2, "a body and a turret");
    assert!(
        settled.iter().all(|i| level.contains(i)),
        "level at the start of the descent: the settled body and turret"
    );
    for k in [1, 3, 4, 5, 6] {
        assert_eq!(a[k].1, 0, "drawing {k}: no shadow while sinking");
        assert_eq!(a[k].0.len(), 2, "drawing {k}: a body and a turret");
    }
    assert_eq!(
        a[5].0, *settled,
        "below the rocking threshold: settled images"
    );
    for k in [1, 4, 6] {
        for (i, img) in a[k].0.iter().enumerate() {
            assert!(!level.contains(img), "drawing {k}, part {i}: leaning");
        }
    }
    assert_ne!(a[1].0, a[4].0, "mid-descent and full leans differ");
    // (The made-up hull is a box, the same fore and aft, so bow down and
    // stern down are not told apart here; the render crate's F5 test
    // checks which end goes down.)
    // Leaning first, in a fresh painter: a leaning image depends on its
    // angles alone, and the level images that follow are unchanged.
    let b = images(&[mid, Attitude::None]);
    assert_eq!(b[0], a[1], "a leaning image depends on its angles alone");
    assert_eq!(
        b[1], a[0],
        "a level vessel after a leaning one is unchanged"
    );
}

// spec: S-0504 claim 16 (a body drawn by another locomotor does not show the lean, and still draws no shadow while sinking)
#[test]
fn a_body_that_draws_no_rocking_is_drawn_settled_without_a_shadow() {
    let a = images(&[Attitude::None, sinking(0.6, false), sinking(0.0, true)]);
    assert_eq!(a[1].1, 0, "no shadow while sinking");
    assert_eq!(
        a[1].0, a[2].0,
        "drawn settled, as at the start of the descent"
    );
}
