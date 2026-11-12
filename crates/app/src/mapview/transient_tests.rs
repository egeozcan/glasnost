//! Tests of the transient voxel images of the live view (design: render
//! (#104), our own: S-0105 claim 26 draws these poses fresh and requires no
//! storage lifetime): a rocking, sinking or tumbling body and a rocket's
//! fresh pitch draw images that the painter releases once no redraw has
//! used them for a while, so the images the scene holds level off, and a
//! pose drawn again after its images went draws the same pictures. Every
//! type name and number is made up.

use glasnost_convert::art::TypeArt;
use glasnost_convert::kinds::Kind;
use glasnost_convert::map::{DecodeContext, decode};
use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
use glasnost_ini::{Document, Quirks};
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::scene::Scene;
use glasnost_sim::world::ObjectId;

use super::build::{Attitude, Inputs, Mobile, Options, Painter, build_parts};
use super::live::sim_code;
use super::tests::{RULES, context_with, scene_map_text};

/// Runs `f` with the inputs, painter and scene of a map with one vehicle
/// type with a turret, drawn from a synthetic voxel model.
fn with_painter(f: impl FnOnce(&Inputs, &mut Painter, &mut Scene)) {
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
    f(&inputs, &mut painter, &mut scene);
}

/// The vehicle of object `n` in `attitude`.
fn vehicle(n: u32, attitude: Attitude) -> Mobile {
    Mobile {
        kind: Kind::Vehicles,
        idx: 0,
        owner: b"Neutral".to_vec(),
        pos: (3 * 256 + 128, 3 * 256 + 128, 0),
        body: 8192,
        turret: 24576,
        elevation: 16384,
        anim: 0,
        infantry: None,
        object: sim_code(ObjectId(5 + n)),
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
    }
}

/// Pose `k` of a sequence of rocking, sinking, rocket and tumbling poses,
/// none of which repeats for the first few hundred `k`.
fn pose(k: u32) -> Attitude {
    let small = |n: u32| 0.02 + 0.005 * (n % 100) as f32;
    let other = |n: u32| 0.005 * (n / 100) as f32;
    match k % 4 {
        0 => Attitude::Rocking {
            pitch: small(k / 4).to_bits(),
            roll: (-other(k / 4)).to_bits(),
            drawn: true,
        },
        1 => Attitude::Sinking {
            pitch: (-small(k / 4)).to_bits(),
            roll: other(k / 4).to_bits(),
            drawn: true,
        },
        2 => Attitude::Rocket {
            pitch: (0.1 + 0.003 * (k / 4) as f32).to_bits(),
            initial: 0.5f32.to_bits(),
            final_: 0.25f32.to_bits(),
        },
        _ => Attitude::Flying {
            height: 500,
            speed: 0.5f64.to_bits(),
            turn: 0,
            crashing: true,
            tumble: (
                (0.05 * (k / 4) as f32).to_bits(),
                (0.07 * (k / 4) as f32).to_bits(),
            ),
        },
    }
}

/// What one drawn frame shows: each item's image (size and pixels), place
/// and paint.
type Shown = Vec<((u32, u32), Vec<u8>, i32, i32, String)>;

/// Draws two vehicles in poses `poses(frame)` for `frames` redraws, ending
/// each redraw when `evict`: what each frame shows and the images the scene
/// holds after it.
fn run(frames: u32, evict: bool, poses: impl Fn(u32) -> [Attitude; 2]) -> (Vec<Shown>, Vec<usize>) {
    let mut shown = Vec::new();
    let mut held = Vec::new();
    with_painter(|inputs, painter, scene| {
        for frame in 0..frames {
            let [a, b] = poses(frame);
            let (entries, _) = painter.mobiles(inputs, scene, &[vehicle(0, a), vehicle(1, b)]);
            if evict {
                painter.end_redraw(scene);
            }
            let items: Shown = entries
                .iter()
                .flat_map(|e| &e.items)
                .map(|i| {
                    let img = &scene.images[i.image as usize];
                    (
                        (img.width(), img.height()),
                        img.pixels().to_vec(),
                        i.x,
                        i.y,
                        format!("{:?}", i.paint),
                    )
                })
                .collect();
            // design: render (#104): no item drawn in a redraw names an image released at its end
            assert!(
                items.iter().all(|(size, ..)| size.0 > 0 && size.1 > 0),
                "frame {frame}: an item names a released image"
            );
            shown.push(items);
            held.push(scene.held().0);
        }
    });
    (shown, held)
}

/// design: render (#104): poses that never repeat (rocking, sinking,
/// rocket pitches, crash tumbles) do not make the scene's images grow
/// without end.
#[test]
fn transient_images_level_off() {
    let (_, held) = run(400, true, |f| [pose(2 * f), pose(2 * f + 1)]);
    let (early, late) = (held[100], held[399]);
    assert!(
        late <= early,
        "the scene held {early} images after 100 redraws and {late} after 400"
    );
}

/// design: render (#104): a pose drawn again after its images were released
/// draws the same pictures as a painter that kept them.
#[test]
fn released_poses_draw_the_same_pictures() {
    let poses = |f: u32| {
        // Every pose comes back after 24 redraws, long after a release.
        let k = f % 24;
        [pose(2 * k), pose(2 * k + 1)]
    };
    let (evicting, held) = run(72, true, poses);
    let (keeping, _) = run(72, false, poses);
    assert_eq!(evicting.len(), keeping.len());
    for (frame, (a, b)) in evicting.iter().zip(&keeping).enumerate() {
        assert_eq!(a, b, "frame {frame}");
    }
    assert!(held[71] <= held[23], "{held:?}");
}
