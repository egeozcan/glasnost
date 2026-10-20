//! Tests of how the live game draws projectiles in flight (S-0306 claims 28
//! to 32) and the target line (S-0301 claim 24), on invented art and the
//! simulation's synthetic combat ruleset.

use glasnost_convert::art::{ProjectileColour, ProjectileLook, TypeArt};
use glasnost_convert::kinds::Kind;
use glasnost_render::compose::{DepthTest, Paint, Profile};
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::projection::{height_lift, project_lifted};
use glasnost_render::scene::{Group, Item, ItemDepth, Scene};
use glasnost_sim::world::{HouseId, ObjectId};
use glasnost_sim::{rules::Kind as SimKind, testkit};

use super::build::{
    Assets, Inputs, Options, ProjectileDraw, ProjectilePalette, TARGET_LINE_INDEX, build_parts,
    dotted_line,
};
use super::live::{Entrant, Live};
use super::tests::{RULES, context_with, shp};

/// The rules of the test context with a vehicle type `CART` (the
/// simulation's cart) whose weapon names the projectile `Shell` (S-0004:
/// projectiles are registered by reference).
fn shell_rules() -> String {
    format!(
        "{RULES}[VehicleTypes]\n2=CART\n[CART]\nPrimary=Gun\n[Gun]\nProjectile=Shell\nWarhead=HE\n[HE]\nVerses=1\n"
    )
}

/// The test context with art `look` for the projectile `Shell`: the sprite
/// `shellx.shp` of `frames` frames (canvas 10 × 10, frame i a 2 × 2 block
/// of value i + 1 at (4, 4)), or the synthetic voxel model `shellx.vxl` for
/// a voxel look; and the invented overlay palette as the general palette.
fn shell_context(
    look: ProjectileLook,
    frames: usize,
) -> (
    glasnost_convert::Import,
    glasnost_convert::theater::Catalogue,
    Assets,
    usize,
) {
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
    let (mut import, catalogue, mut assets) = context_with(&shell_rules());
    let idx = import.outcome.registries[&Kind::Projectiles]
        .iter()
        .position(|t| t.name.eq_ignore_ascii_case(b"Shell"))
        .expect("Shell registered");
    let mut art = TypeArt {
        image: b"SHELLX".to_vec(),
        projectile: Some(look.clone()),
        ..Default::default()
    };
    if look.voxel {
        art.voxels.insert("body".into(), b"shellx.vxl".to_vec());
        assets.insert(
            b"shellx.vxl",
            VxlWriter {
                model: &synthetic_model(),
            }
            .write(),
        );
    } else {
        art.sprite = Some(b"shellx.shp".to_vec());
        assets.insert(b"shellx.shp", shp((10, 10), frames, (2, 2), (4, 4)));
    }
    import.arts.insert((Kind::Projectiles, idx), art);
    // The simulation's cart is drawn with the test vehicle's sprite.
    let cart = import.outcome.registries[&Kind::Vehicles]
        .iter()
        .position(|t| t.name == b"CART")
        .expect("CART registered");
    import.arts.insert(
        (Kind::Vehicles, cart),
        super::tests::art_of("CARX", "carx.shp"),
    );
    let pal: Vec<u8> = (0..768u32).map(|i| (i * 7 % 64) as u8).collect();
    assets.insert(super::combat::OVERLAY_PALETTE, pal);
    (import, catalogue, assets, idx)
}

/// Draws `list` with the painter of a fresh scene of the context.
fn draw(
    import: &glasnost_convert::Import,
    catalogue: &glasnost_convert::theater::Catalogue,
    assets: &Assets,
    list: &[ProjectileDraw],
) -> (Scene, Vec<Vec<Item>>) {
    let map = glasnost_sim::testkit::empty_map(24);
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let inputs = Inputs {
        map: &map,
        import,
        catalogue,
        palettes: &palettes,
        assets,
        options: &options,
        rules: None,
        cells: None,
    };
    let (built, mut painter) = build_parts(&inputs, true);
    let mut scene = built.scene;
    let (items, _) = painter.projectiles(&inputs, &mut scene, list);
    (scene, items)
}

fn sprite_look() -> ProjectileLook {
    ProjectileLook {
        rotates: true,
        shadow: true,
        ..ProjectileLook::default()
    }
}

fn at(
    idx: usize,
    frame: i32,
    palette: ProjectilePalette,
    shadow: Option<(i32, i32)>,
) -> ProjectileDraw {
    ProjectileDraw {
        idx,
        pos: (2000, 2200, 300),
        facings: (8, 8),
        frame,
        palette,
        shadow,
    }
}

/// spec: S-0306 claims 28, 31 and 32, S-0102 claims 10, 12, 13 and 19 (a sprite body centred on the draw point, a flat reader with bias −30 − L(z), in its palette at brightness 1000; its shadow first, the same frame centred L(η) rows lower, a flat reader with bias −10 − L of the ground under it, drawn as a darkening)
#[test]
fn sprite_body_and_shadow() {
    let (import, catalogue, assets, idx) = shell_context(sprite_look(), 32);
    let d = at(idx, 5, ProjectilePalette::General, Some((180, 120)));
    let (scene, items) = draw(&import, &catalogue, &assets, std::slice::from_ref(&d));
    let items = &items[0];
    assert_eq!(items.len(), 2, "{items:?}");
    let (px, py) = project_lifted(d.pos.0, d.pos.1, d.pos.2);
    let (s, b) = (&items[0], &items[1]);
    // The body: canvas 10 × 10 centred, frame rectangle at (4, 4).
    assert_eq!((b.x, b.y), (px - 5 + 4, py - 5 + 4));
    assert_eq!(
        scene.images[b.image as usize].pixels()[0],
        6,
        "image frame 5"
    );
    assert_eq!(b.test, DepthTest::Reader);
    assert_eq!(
        b.depth,
        ItemDepth::Profile {
            profile: Profile::Flat,
            bias: -30 - height_lift(300),
            companion: None,
        }
    );
    let Paint::Lit(row) = b.paint else {
        panic!("a lit body: {:?}", b.paint)
    };
    assert_eq!(row.rows, 53, "the general palette's 53-row table");
    assert_eq!(b.group, Group::Effects);
    // The shadow: the same frame, L(η) rows below, a darkening.
    assert_eq!(s.image, b.image);
    assert_eq!((s.x, s.y), (b.x, b.y + height_lift(180)));
    assert_eq!(s.paint, Paint::Shadow);
    assert_eq!(s.test, DepthTest::Reader);
    assert_eq!(
        s.depth,
        ItemDepth::Profile {
            profile: Profile::Flat,
            bias: -10 - height_lift(120),
            companion: None,
        }
    );
    // Without a shadow: the body alone.
    let (_, items) = draw(
        &import,
        &catalogue,
        &assets,
        &[at(idx, 5, ProjectilePalette::General, None)],
    );
    assert_eq!(items[0].len(), 1);
}

/// spec: S-0306 claim 28 (nothing at all is drawn for a frame number at or above the frame count, an invisible projectile, or one without a sprite)
#[test]
fn nothing_drawn_without_a_frame_or_art() {
    let (import, catalogue, assets, idx) = shell_context(sprite_look(), 8);
    let (_, items) = draw(
        &import,
        &catalogue,
        &assets,
        &[
            at(idx, 7, ProjectilePalette::General, Some((100, 0))),
            at(idx, 8, ProjectilePalette::General, Some((100, 0))),
            at(idx, -1, ProjectilePalette::General, None),
        ],
    );
    assert_eq!(items[0].len(), 2);
    assert!(items[1].is_empty() && items[2].is_empty());
    let inviso = ProjectileLook {
        inviso: true,
        ..sprite_look()
    };
    let (import, catalogue, assets, idx) = shell_context(inviso, 8);
    let (_, items) = draw(
        &import,
        &catalogue,
        &assets,
        &[at(idx, 0, ProjectilePalette::General, None)],
    );
    assert!(items[0].is_empty());
    let (mut import, catalogue, assets, idx) = shell_context(sprite_look(), 8);
    import
        .arts
        .get_mut(&(Kind::Projectiles, idx))
        .unwrap()
        .sprite = None;
    let (_, items) = draw(
        &import,
        &catalogue,
        &assets,
        &[at(idx, 0, ProjectilePalette::General, None)],
    );
    assert!(items[0].is_empty());
}

/// spec: S-0306 claim 31 (the anim palette, a house's 53-shade palette or the general palette, always at brightness 1000)
#[test]
fn sprite_palettes() {
    let (import, catalogue, assets, idx) = shell_context(sprite_look(), 32);
    let (_, items) = draw(
        &import,
        &catalogue,
        &assets,
        &[
            at(idx, 0, ProjectilePalette::Anim, None),
            at(idx, 0, ProjectilePalette::House(b"Neutral".to_vec()), None),
            at(idx, 0, ProjectilePalette::General, None),
        ],
    );
    let rows: Vec<_> = items
        .iter()
        .map(|i| match i[0].paint {
            Paint::Lit(r) => r,
            p => panic!("{p:?}"),
        })
        .collect();
    assert!(
        rows[0].base != rows[1].base
            && rows[1].base != rows[2].base
            && rows[0].base != rows[2].base
    );
    // All three at brightness 1000.
    assert!(rows.iter().all(|r| r.rho == rows[0].rho));
}

/// spec: S-0306 claim 30, S-0102 claims 10 and 13 (a voxel body: one image around the draw point, no shadow; non-flat neither tests nor writes depth, flat reads with bias −65 and no −L(z); in the colour definition's palette)
#[test]
fn voxel_body() {
    let look = ProjectileLook {
        voxel: true,
        shadow: true,
        colour: ProjectileColour::Shades(1),
        ..ProjectileLook::default()
    };
    let (import, catalogue, assets, idx) = shell_context(look.clone(), 0);
    let mut d = at(idx, 0, ProjectilePalette::General, Some((100, 0)));
    let (_, items) = draw(&import, &catalogue, &assets, std::slice::from_ref(&d));
    assert_eq!(items[0].len(), 1, "no shadow for a voxel projectile");
    let body = items[0][0];
    assert_eq!(body.test, DepthTest::Ignore);
    let Paint::Lit(row) = body.paint else {
        panic!("{:?}", body.paint)
    };
    assert_eq!(row.rows, 53);
    // Another heading: another image.
    d.facings = (20, 4);
    let (_, turned) = draw(
        &import,
        &catalogue,
        &assets,
        &[d.clone(), at(idx, 0, ProjectilePalette::General, None)],
    );
    assert_eq!(turned[0].len(), 1);
    let flat = ProjectileLook {
        flat: true,
        colour: ProjectileColour::Single(0),
        ..look
    };
    let (import, catalogue, assets, idx) = shell_context(flat, 0);
    let (_, items) = draw(
        &import,
        &catalogue,
        &assets,
        &[at(idx, 0, ProjectilePalette::General, None)],
    );
    let body = items[0][0];
    assert_eq!(body.test, DepthTest::Reader);
    assert_eq!(
        body.depth,
        ItemDepth::Profile {
            profile: Profile::Flat,
            bias: -65,
            companion: None,
        }
    );
    let Paint::Lit(row) = body.paint else {
        panic!("{:?}", body.paint)
    };
    assert_eq!(row.rows, 1, "a single-shade palette");
    // Without voxel art: nothing.
    let (mut import, catalogue, assets, idx) = shell_context(
        ProjectileLook {
            voxel: true,
            ..ProjectileLook::default()
        },
        0,
    );
    import
        .arts
        .get_mut(&(Kind::Projectiles, idx))
        .unwrap()
        .voxels
        .clear();
    let (_, items) = draw(
        &import,
        &catalogue,
        &assets,
        &[at(idx, 0, ProjectilePalette::General, None)],
    );
    assert!(items[0].is_empty());
}

/// spec: S-0301 claim 24 g (one pixel per step along the longer screen axis, every second one drawn, the phase swapping the drawn and undrawn ones)
#[test]
fn dotted_lines() {
    let even = dotted_line((0, 0), (6, 2), false);
    assert_eq!(even, [(0, 0), (2, 1), (4, 1), (6, 2)]);
    let odd = dotted_line((0, 0), (6, 2), true);
    assert_eq!(odd, [(1, 0), (3, 1), (5, 2)]);
    assert_eq!(dotted_line((3, 3), (3, -3), false).len(), 4);
    assert_eq!(dotted_line((3, 3), (3, 3), false), [(3, 3)]);
    assert!(dotted_line((3, 3), (3, 3), true).is_empty());
}

/// A live game of the synthetic combat ruleset (the cart's type with the
/// target-laser flag when `laser`) on an empty map of 24 with the Shell
/// art `look`: a cart of house 0 and an unarmed walker of house 1 three cells
/// apart.
fn shell_live(
    look: ProjectileLook,
    laser: bool,
) -> (
    Live,
    Scene,
    glasnost_map::Map,
    glasnost_convert::Import,
    glasnost_convert::theater::Catalogue,
    Assets,
    ObjectId,
) {
    let (import, catalogue, assets, _) = shell_context(look, 32);
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
    let mut r = testkit::combat_ruleset();
    r.sim.vehicles.get_mut("CART").unwrap().target_laser = Some(laser);
    r.sim.infantry.get_mut("WALKER").unwrap().primary = None;
    let mut world = testkit::game_with(&r, 24, 3);
    let cart = testkit::spawn(
        &mut world,
        SimKind::Vehicle,
        "CART",
        0,
        testkit::mid(24, -1, 0),
        16384,
    );
    testkit::spawn(
        &mut world,
        SimKind::Infantry,
        "WALKER",
        1,
        testkit::mid(24, 2, 0),
        49152,
    );
    let rules = world.rules_arc();
    let live = Live::new(
        &inputs,
        &mut built,
        painter,
        (world, rules, testkit::setup(3), Vec::new()),
    );
    (live, built.scene, map, import, catalogue, assets, cart)
}

/// spec: S-0306 claims 28 and 29, S-0102 claims 3 and 7 (a projectile in flight is an entrant of the air layer drawn with its art at its draw point, the rotating sprite's frame (28 − ψ) mod 32; the debug view's stand-in marker only for one that drew nothing)
#[test]
fn live_projectiles_are_drawn_in_the_air_layer() {
    let (mut live, mut scene, map, import, catalogue, assets, _) = shell_live(sprite_look(), false);
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
    let mut seen = 0;
    for _ in 0..400 {
        live.step();
        live.redraw(&inputs, &mut scene);
        let air = live.layer_list(super::slots::Layer::Air);
        for p in live.world.projectiles.values() {
            let rules = live.rules.clone();
            if rules
                .combat
                .projectile_of(p.weapon)
                .is_some_and(|k| k.inviso)
            {
                continue;
            }
            assert!(air.contains(&Entrant::Projectile(p.id)), "in the air layer");
            let (psi, _) = glasnost_sim::combat::projectile::drawn_facings(&live.world, p.vel);
            let frame = (28 - i32::from(psi)).rem_euclid(32);
            let (px, py) = project_lifted(p.pos.x, p.pos.y, p.pos.z);
            let body = scene.items.iter().find(|i| {
                i.group == Group::Effects
                    && (i.x, i.y) == (px - 1, py - 1)
                    && matches!(i.paint, Paint::Lit(_))
                    && scene.images[i.image as usize].width() == 2
            });
            let body = body.expect("the body at its draw point");
            assert_eq!(
                i32::from(scene.images[body.image as usize].pixels()[0]) - 1,
                frame,
                "the rotating frame"
            );
            // No 7 × 7 stand-in marker for a projectile with a body.
            assert!(!scene.items.iter().any(|i| {
                (i.x + 3, i.y + 3) == (px, py) && scene.images[i.image as usize].width() == 7
            }));
            seen += 1;
        }
        if seen > 10 {
            break;
        }
    }
    assert!(seen > 10, "shells were drawn in flight");
}

/// The target line's pixels in `scene`: 1 × 1 images of palette index 8
/// painted from a row with no depth test.
fn line_pixels(scene: &Scene) -> Vec<(i32, i32)> {
    scene
        .items
        .iter()
        .filter(|i| {
            let img = &scene.images[i.image as usize];
            img.width() == 1
                && img.height() == 1
                && img.pixels()[0] == TARGET_LINE_INDEX
                && matches!(i.paint, Paint::Row(_))
                && i.test == DepthTest::Ignore
        })
        .map(|i| (i.x, i.y))
        .collect()
}

/// spec: S-0301 claim 24 b to h (the line is drawn in every drawn frame with 0 ≤ t − t₀ < 15 after each starting shot while the vehicle has a target, from its turret pivot to the aim point of its target, 3 × 3 squares at both ends; only for the owning local player; a type without the flag shows none)
#[test]
fn target_line_follows_its_timer() {
    for (laser, viewer) in [
        (true, None),
        (true, Some(HouseId(0))),
        (true, Some(HouseId(1))),
        (false, None),
    ] {
        let (mut live, mut scene, map, import, catalogue, assets, cart) =
            shell_live(sprite_look(), laser);
        live.set_viewer(viewer);
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
        let owner = laser && viewer.is_none_or(|v| v == HouseId(0));
        let mut t0: Option<i32> = None;
        let (mut shown, mut hidden, mut targetless) = (0, 0, 0);
        for _ in 0..600 {
            let shots = live.counts.shots;
            live.step();
            // The unarmed walker never fires: every shot is the cart's.
            let t = (live.world.tick - 1).max(0);
            if live.counts.shots > shots {
                t0 = Some(t);
            }
            live.redraw(&inputs, &mut scene);
            let px = line_pixels(&scene);
            let Some(o) = live.world.obj(cart) else {
                break;
            };
            let due = owner && t0.is_some_and(|s| t - s < 15) && o.target.is_some();
            assert_eq!(!px.is_empty(), due, "tick {t}, shot at {t0:?}");
            if due {
                let target = o.target.unwrap();
                let a = glasnost_sim::combat::position::turret_pivot(&live.world, cart);
                let b = glasnost_sim::combat::projectile::led_aim_point(&live.world, cart, target);
                for p in [project_lifted(a.x, a.y, a.z), project_lifted(b.x, b.y, b.z)] {
                    for dy in 0..3 {
                        for dx in 0..3 {
                            assert!(
                                px.contains(&(p.0 - dx, p.1 - dy)),
                                "the 3 × 3 square at {p:?}"
                            );
                        }
                    }
                }
                shown += 1;
            } else if owner && t0.is_some_and(|s| t - s < 15) {
                targetless += 1;
            } else {
                hidden += 1;
            }
        }
        if owner {
            assert!(
                shown > 0 && hidden > 0,
                "shown after shots and gone between them: {shown} {hidden}"
            );
            let _ = targetless;
        } else {
            assert_eq!(shown, 0);
        }
    }
}
