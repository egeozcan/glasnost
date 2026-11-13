//! Our engine's side of S-0313 scenario P10 (Q-1600): where a turreted
//! voxel vehicle's shot animation is drawn against the vehicle's unshifted
//! draw point (S-0313 claims 11, 11a, 11d and 11e), on invented art and the
//! simulation's synthetic combat ruleset (no stock values).
//!
//! The fire points come from the simulation (`combat::position::fire_point`,
//! S-0313 claim 1); the drawing is the painter's. Every expected position is
//! exact, as P10's engine run asks: a difference of one pixel falsifies it.

use glasnost_convert::art::TypeArt;
use glasnost_convert::kinds::Kind;
use glasnost_fmt::bytes::F32Bits;
use glasnost_fmt::vxl::{Vxl, VxlWriter, synthetic_model};
use glasnost_render::compose::Paint;
use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::projection::{height_lift, project_lifted};
use glasnost_render::scene::Scene;
use glasnost_render::voxel::{self, VoxelQuirks};
use glasnost_rules::sim::{Decimal, WeaponType};
use glasnost_sim::combat::position::fire_point;
use glasnost_sim::turning::Turning;
use glasnost_sim::world::{ObjectId, Pos, World};
use glasnost_sim::{rules::Kind as SimKind, testkit};

use super::build::{AnimDraw, Assets, Attitude, Inputs, Mobile, Options, build_parts};
use super::live::sim_code;
use super::tests::{ART, RULES, art_of, context_full, shp};

/// The invented turret displacement δ, leptons.
const DELTA: i32 = 37;
/// The invented forward and lateral muzzle parts, leptons.
const FORWARD: i32 = 93;
const LATERAL: i32 = 21;
/// The test animation's invented vertical draw shift v, pixels.
const V: i32 = -7;
/// The test animation's canvas, odd in both directions; its one image
/// frame is the centre pixel.
const CANVAS: (u16, u16) = (21, 15);
/// One height level, leptons (S-0101 Constants).
const LEVEL: i32 = 104;
/// The vehicle's position: chosen so that its projection has fractional
/// parts (S-0101 claim 2 truncates).
const AT: (i32, i32) = (10 * 256 + 77, 11 * 256 + 141);

type Context = (
    glasnost_convert::Import,
    glasnost_convert::theater::Catalogue,
    Assets,
    usize,
    usize,
);

/// One shot: (turret step, burst counter), the marker's centre counted
/// from the unshifted draw point, the fire point, the vehicle's position.
type Shot = ((u16, i32), (i32, i32), Pos, Pos);

/// The test context: the simulation's cart `CART` with the turret flag,
/// voxel body and turret (and a separate barrel model when `barrel`), the
/// art turret displacement `art_delta`, and the test animation `MARKX`
/// (canvas [`CANVAS`], vertical draw shift [`V`]). Holds the import indices
/// of the cart and the animation last.
fn context(barrel: bool, art_delta: i32) -> Context {
    // The animation list and a reference to its entry, as the animation tests' context has.
    let rules = format!(
        "{RULES}[VehicleTypes]\n2=CART\n[CART]\nTurret=yes\n[Animations]\n0=MARKX\n\
         [General]\nMoveFlash=MARKX\n"
    );
    let art = format!("{ART}[CART]\nTurretOffset={art_delta}\n[MARKX]\nYDrawOffset={V}\n");
    let (mut import, catalogue, mut assets) = context_full(&rules, &art);
    let index = |import: &glasnost_convert::Import, kind: Kind, name: &[u8]| {
        import.outcome.registries[&kind]
            .iter()
            .position(|t| t.name.eq_ignore_ascii_case(name))
            .expect("registered")
    };
    let cart = index(&import, Kind::Vehicles, b"CART");
    let marker = index(&import, Kind::Animations, b"MARKX");
    let mut art = TypeArt {
        image: b"CART".to_vec(),
        ..Default::default()
    };
    for (part, file) in parts(barrel) {
        art.voxels.insert(part.into(), file.as_bytes().to_vec());
        assets.insert(
            file.as_bytes(),
            VxlWriter {
                model: &model(part),
            }
            .write(),
        );
    }
    import.arts.insert((Kind::Vehicles, cart), art);
    import
        .arts
        .insert((Kind::Animations, marker), art_of("MARKX", "markx.shp"));
    let (cx, cy) = CANVAS;
    assets.insert(b"markx.shp", shp(CANVAS, 1, (1, 1), (cx / 2, cy / 2)));
    (import, catalogue, assets, cart, marker)
}

/// The voxel parts and their files.
fn parts(barrel: bool) -> Vec<(&'static str, &'static str)> {
    let mut v = vec![("body", "cartb.vxl"), ("turret", "cartt.vxl")];
    if barrel {
        v.push(("barrel", "cartg.vxl"));
    }
    v
}

/// An invented model per part, of distinct widths so the parts' images differ.
fn model(part: &str) -> Vxl {
    let mut m = synthetic_model();
    let width = match part {
        "body" => 30.0,
        "turret" => 12.0,
        _ => 20.0,
    };
    m.limbs[0].max[0] = F32Bits::new(width);
    m
}

/// A world of the combat ruleset whose cart has the turret displacement
/// [`DELTA`], a weapon and the muzzle position (F, L, `h`), standing at
/// [`AT`] at world height `z`.
fn world(h: i32, z: i32) -> (World, ObjectId) {
    world_flh([FORWARD, LATERAL, h], z)
}

/// [`world`] with the muzzle position `flh`.
fn world_flh(flh: [i32; 3], z: i32) -> (World, ObjectId) {
    let mut r = testkit::combat_ruleset();
    r.sim.weapons.insert(
        "ZAP".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(30),
            range: Some(Decimal("10".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.primary = Some("ZAP".into());
    cart.combat.primary_fire_flh = Some(flh);
    cart.combat.turret_offset = Some(DELTA);
    let mut w = testkit::game_with(&r, 24, 7);
    let a = testkit::spawn(
        &mut w,
        SimKind::Vehicle,
        "CART",
        0,
        testkit::mid(24, 0, 0),
        0,
    );
    w.obj_mut(a).unwrap().pos = Pos::new(AT.0, AT.1, z);
    (w, a)
}

/// The cart of `w` as the painter draws it.
fn mobile(w: &World, a: ObjectId, cart: usize) -> Mobile {
    let o = w.obj(a).unwrap();
    Mobile {
        kind: Kind::Vehicles,
        idx: cart,
        owner: b"Neutral".to_vec(),
        pos: (o.pos.x, o.pos.y, o.pos.z),
        body: o.body.value(w.tick),
        turret: o.turret.value(w.tick),
        elevation: 0,
        anim: 0,
        infantry: None,
        object: sim_code(a),
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
        deck: false,
    }
}

/// One drawn shot: the marker's centre pixel and the lit vehicle items'
/// rectangles (x, y, width, height), sorted.
struct Drawn {
    marker: (i32, i32),
    parts: Vec<(i32, i32, u32, u32)>,
}

/// Draws the cart of `w` and a shot animation at `fire`, with the typed
/// rules of `w` given to the painter when `with_rules`.
fn draw(ctx: &Context, w: &World, a: ObjectId, fire: Pos, with_rules: bool) -> Drawn {
    let (import, catalogue, assets, cart, marker) = ctx;
    let map = testkit::empty_map(24);
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let rules = w.rules_arc();
    let inputs = Inputs {
        map: &map,
        import,
        catalogue,
        palettes: &palettes,
        assets,
        options: &options,
        rules: with_rules.then_some(&*rules),
        cells: None,
    };
    let (built, mut painter) = build_parts(&inputs, true);
    let mut scene: Scene = built.scene;
    let (entries, _) = painter.mobiles(&inputs, &mut scene, &[mobile(w, a, *cart)]);
    let mut parts: Vec<_> = entries[0]
        .items
        .iter()
        .filter(|i| matches!(i.paint, Paint::Lit(_)))
        .map(|i| {
            let img = &scene.images[i.image as usize];
            (i.x, i.y, img.width(), img.height())
        })
        .collect();
    parts.sort();
    let anim = AnimDraw {
        idx: *marker,
        frame: 0,
        stage: 0,
        end: 0,
        pos: (fire.x, fire.y, fire.z),
        carrier: None,
        named: None,
        z_adjust: None,
    };
    let (drawn, _) = painter.anims(&inputs, &mut scene, &[anim]);
    let (_, _, items) = drawn[0].clone().expect("the marker is drawn");
    let image = items
        .iter()
        .find(|i| !matches!(i.paint, Paint::Shadow))
        .expect("the marker's image");
    // The image frame is the canvas's centre pixel (S-0102 claim 19), so its
    // top-left pixel is the marker's centre.
    Drawn {
        marker: (image.x, image.y),
        parts,
    }
}

/// The rectangles of the cart's parts drawn independently around the
/// unshifted draw point `point`, with turret displacement `delta`.
fn expected_parts(
    barrel: bool,
    body: u16,
    turret: u16,
    delta: i32,
    point: (i32, i32),
) -> Vec<(i32, i32, u32, u32)> {
    let q = VoxelQuirks::default();
    let pose = voxel::vehicle_parts(body, turret, delta, 0, &q);
    let mut v: Vec<_> = parts(barrel)
        .into_iter()
        .filter_map(|(part, _)| {
            let draw = match part {
                "body" => pose.body,
                "turret" => pose.turret,
                _ => pose.barrel,
            };
            let img = voxel::draw_part(&model(part), None, 0, &draw, None, false, &q)?;
            (img.width > 0 && img.height > 0).then(|| {
                (
                    point.0 + img.offset[0] as i32,
                    point.1 + img.offset[1] as i32,
                    img.width,
                    img.height,
                )
            })
        })
        .collect();
    v.sort();
    v
}

fn set_pose(w: &mut World, a: ObjectId, body: u16, turret: u16) {
    let o = w.obj_mut(a).unwrap();
    o.body = Turning::at(body);
    o.turret = Turning::at(turret);
}

fn diff(p: (i32, i32), q: (i32, i32)) -> (i32, i32) {
    (p.0 - q.0, p.1 - q.1)
}

/// The predicted marker centre counted from the unshifted draw point
/// (S-0313 claim 11a): the whole-pixel projection of the fire point's world
/// position minus that of the vehicle's position, each with S-0101 claim 3's
/// lift of its own height, then moved down by v.
fn predicted(fire: Pos, at: Pos) -> (i32, i32) {
    let d = diff(
        project_lifted(fire.x, fire.y, fire.z),
        project_lifted(at.x, at.y, at.z),
    );
    (d.0, d.1 + V)
}

/// The shots at level `level` and muzzle height `h`, for body facing
/// `body`, every turret step and both burst counters. Checks every marker
/// against [`predicted`] and the body, turret and barrel placement.
fn run(level: i32, h: i32, body: u16, barrel: bool) -> Vec<Shot> {
    let ctx = context(barrel, DELTA);
    let (mut w, a) = world(h, level * LEVEL);
    let mut out = Vec::new();
    for step in 0..32u16 {
        let turret = step * 2048;
        set_pose(&mut w, a, body, turret);
        let at = w.obj(a).unwrap().pos;
        let point = project_lifted(at.x, at.y, at.z);
        for k in [0, 1] {
            let fire = fire_point(&w, a, 0, k);
            let drawn = draw(&ctx, &w, a, fire, true);
            let from_point = diff(drawn.marker, point);
            assert_eq!(
                from_point,
                predicted(fire, at),
                "level {level}, H {h}, body {body}, step {step}, k {k}: the marker"
            );
            // spec: S-0313 claim 11 (our driving locomotor adds no draw shift: body and turret
            // are drawn from the unshifted draw point, the turret with F2's δ and facings)
            assert_eq!(
                drawn.parts,
                expected_parts(barrel, body, turret, DELTA, point),
                "level {level}, H {h}, body {body}, step {step}, k {k}: the parts"
            );
            out.push(((step, k), from_point, fire, at));
        }
    }
    out
}

/// spec: S-0313 claims 11 and 11a, S-0109 claim 26, S-0102 claim 19 (P10's engine run: at every
/// turret step and both burst counters, at levels 0 and 6, the marker's centre counted from the
/// vehicle's unshifted draw point is the whole-pixel projection of the fire point minus that of
/// the vehicle's position, moved down by v, exactly; the body and turret are drawn from the
/// unshifted draw point; the residual does not change with the turret's direction)
#[test]
fn p10_marker_at_the_projected_fire_point() {
    let mut differs = 0;
    for level in [0, 6] {
        for h in [0, 40, 105] {
            for (_, offset, fire, at) in run(level, h, 0, false) {
                // Not the projection of the displacement alone: where the coordinates have
                // fractional parts the two can differ by a pixel (claim 11a).
                let alone = project_lifted(fire.x - at.x, fire.y - at.y, 0);
                let lift = height_lift(fire.z) - height_lift(at.z);
                if (alone.0, alone.1 - lift + V) != offset {
                    differs += 1;
                }
            }
        }
    }
    assert!(
        differs > 0,
        "the test position separates the two projections at some turret step"
    );
}

/// spec: S-0313 claim 11 (P10's H40 variant: at every body facing too the marker is where claim
/// 11a puts it, exactly, so its residual does not change with the body facing; the turret is
/// drawn with F2's δ and the 32-step turret and body facings)
#[test]
fn p10_every_body_facing() {
    for level in [0, 6] {
        for body in (0..32u16).map(|b| b * 2048) {
            run(level, 40, body, false);
        }
    }
}

/// spec: S-0313 claims 11a and 11e, S-0101 claim 3 and F2 (P10's moves between muzzle heights at
/// the same direction: straight up by our lift's difference, 6, 15 and 9 pixels at both levels;
/// the H0 marker is the same at levels 0 and 6)
#[test]
fn p10_moves_between_muzzle_heights() {
    // Our lift: L(40) = 6, L(105) = 15, L(624) = 90, L(664) = 96, L(729) = 105 (S-0313 claim 11e).
    for (z, want) in [(40, 6), (105, 15), (624, 90), (664, 96), (729, 105)] {
        assert_eq!(height_lift(z), want, "L({z})");
    }
    for level in [0, 6] {
        let runs: Vec<_> = [0, 40, 105]
            .into_iter()
            .map(|h| run(level, h, 0, false))
            .collect();
        for ((s0, s40), s105) in runs[0].iter().zip(&runs[1]).zip(&runs[2]) {
            assert_eq!((s0.0, s40.0), (s105.0, s105.0), "the same step and counter");
            let [h0, h40, h105] = [s0.1, s40.1, s105.1];
            assert_eq!(diff(h40, h0), (0, -6), "level {level}: H0 to H40");
            assert_eq!(diff(h105, h0), (0, -15), "level {level}: H0 to H105");
            assert_eq!(diff(h105, h40), (0, -9), "level {level}: H40 to H105");
        }
    }
    // The H0 residual difference between levels 6 and 0 is 0: counted from the unshifted draw
    // point, the H0 marker is the same at both levels.
    let offsets = |level| -> Vec<_> {
        run(level, 0, 0, false)
            .into_iter()
            .map(|(key, offset, _, _)| (key, offset))
            .collect()
    };
    assert_eq!(offsets(0), offsets(6));
}

/// spec: S-0313 claims 11 and 11a (P10's muzzle variant: changing the forward and lateral parts
/// moves the marker by the change in the fire point's projection and leaves the turret's pixels
/// unchanged), claim 2 (the odd counter's lateral part is mirrored)
#[test]
fn p10_muzzle_variant_and_mirroring() {
    let ctx = context(false, DELTA);
    let shot = |flh: [i32; 3], k: i32| {
        let (mut w, a) = world_flh(flh, 0);
        set_pose(&mut w, a, 0, 5 * 2048);
        let fire = fire_point(&w, a, 0, k);
        (fire, draw(&ctx, &w, a, fire, true))
    };
    let (f0, d0) = shot([FORWARD, LATERAL, 40], 0);
    let (f1, d1) = shot([FORWARD, LATERAL, 40], 1);
    assert_ne!(f0, f1, "the odd counter is mirrored");
    assert_eq!(
        d0.parts, d1.parts,
        "the turret does not move with the counter"
    );
    // The muzzle variant: other forward and lateral parts, the same height.
    let (g0, e0) = shot([FORWARD + 61, -LATERAL - 30, 40], 0);
    assert_ne!(g0, f0);
    assert_eq!(e0.parts, d0.parts, "the turret's pixels are unchanged");
    assert_eq!(
        diff(e0.marker, d0.marker),
        diff(
            project_lifted(g0.x, g0.y, g0.z),
            project_lifted(f0.x, f0.y, f0.z)
        )
    );
}

/// spec: S-0313 claim 11d (our engine: a separate barrel model changes neither the fire point nor
/// the shot animation's drawn position, at any turret step or counter)
#[test]
fn p10_barrel_model_moves_nothing() {
    for level in [0, 6] {
        let with = run(level, 40, 0, true);
        let without = run(level, 40, 0, false);
        for (b, n) in with.iter().zip(&without) {
            assert_eq!((b.0, b.1, b.2), (n.0, n.1, n.2), "level {level}");
        }
    }
}

/// spec: S-0313 claim 11 (the turret is drawn with the same displacement δ that F2 uses for the
/// fire point: the simulation's imported value when the typed rules name the type), S-0105
/// Import mapping (the art read otherwise)
#[test]
fn turret_drawn_with_the_fire_points_displacement() {
    // The art read gives another value than the simulation holds.
    let ctx = context(false, 5);
    let (mut w, a) = world(40, 0);
    set_pose(&mut w, a, 0, 3 * 2048);
    let at = w.obj(a).unwrap().pos;
    let point = project_lifted(at.x, at.y, at.z);
    let fire = fire_point(&w, a, 0, 0);
    let with = draw(&ctx, &w, a, fire, true);
    assert_eq!(
        with.parts,
        expected_parts(false, 0, 3 * 2048, DELTA, point),
        "the simulation's δ"
    );
    let without = draw(&ctx, &w, a, fire, false);
    assert_eq!(
        without.parts,
        expected_parts(false, 0, 3 * 2048, 5, point),
        "no typed rules: the art read"
    );
    assert_ne!(with.parts, without.parts);
}

/// spec: S-0313 claim 11 (the turret is drawn with the same 32-step turret and body facings that
/// F2 uses for the fire point: S-0105's 32-step facing equals S-0204 F4's with n = 32 for every
/// raw facing)
#[test]
fn turret_drawn_with_the_fire_points_facings() {
    for f in 0..=u16::MAX {
        assert_eq!(
            u32::from(voxel::facing32(f)),
            glasnost_num::geom::n_step(f, 32),
            "raw facing {f}"
        );
    }
}
