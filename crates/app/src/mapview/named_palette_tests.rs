//! Tests of the named palettes (S-0103 claims 16 and 17), on invented
//! data: the test palette name, its file's colours and every number here
//! are made up.

use glasnost_convert::kinds::Kind;
use glasnost_convert::map::{DecodeContext, decode};
use glasnost_convert::theater::Catalogue;
use glasnost_fmt::image::Palette;
use glasnost_fmt::pal::Palette6;
use glasnost_ini::{Document, Quirks};
use glasnost_render::compose::{Paint, Row565, Shade, row565};
use glasnost_render::light::shade_table;
use glasnost_render::palette::{
    PaletteQuirks, PaletteState, TINT_EXEMPT, house_palette, named_palette_file,
};
use glasnost_render::scene::{Item, Scene};
use glasnost_sim::world::ObjectId;
use glasnost_vfs::theater::Theater;

use super::build::{
    Assets, Attitude, InfantryFrame, Inputs, Mobile, Options, build, build_parts, owner_definition,
};
use super::live::sim_code;
use super::tests::{ART, RULES, art_of, context_full, scene_map_text, shp};

/// The invented art `Palette` value.
const NAME: &[u8] = b"test";

/// An invented named palette file, unlike the all-black unit palette of
/// the tests.
fn named_file() -> Palette6 {
    let mut colors = [[0u8; 3]; 256];
    for (i, c) in colors.iter_mut().enumerate() {
        *c = [(i % 64) as u8, (63 - i % 64) as u8, ((i / 4) % 64) as u8];
    }
    Palette6 { colors }
}

/// The test art with `extra` added after it (a repeated section adds its
/// keys to the earlier one).
fn art_with(extra: &str) -> String {
    format!("{ART}{extra}")
}

/// What a test draws in: the art and rules additions, whether CARX is
/// drawn as voxels, and the theater the named file is stored for (`None`:
/// no file).
struct Setup<'a> {
    art: &'a str,
    rules: &'a str,
    voxel: bool,
    file_for: Option<Theater>,
}

impl Default for Setup<'_> {
    fn default() -> Self {
        Self {
            art: "",
            rules: "",
            voxel: false,
            file_for: Some(Theater::Temperate),
        }
    }
}

/// The test context of `s` and the map of `map_text`.
fn context_of(
    s: &Setup,
    map_text: &[u8],
) -> (
    glasnost_convert::Import,
    Catalogue,
    Assets,
    glasnost_map::Map,
) {
    use glasnost_fmt::vxl::{VxlWriter, synthetic_model};
    let (mut import, catalogue, mut assets) =
        context_full(&format!("{RULES}{}", s.rules), &art_with(s.art));
    if s.voxel {
        let art = import.arts.get_mut(&(Kind::Vehicles, 0)).unwrap();
        art.voxels.insert("body".into(), b"car.vxl".to_vec());
        assets.insert(
            b"car.vxl",
            VxlWriter {
                model: &synthetic_model(),
            }
            .write(),
        );
    }
    let map = decode(
        &Document::parse(map_text, Quirks::default(), &mut Vec::new()),
        &DecodeContext {
            catalogue: Some(&catalogue),
            ..Default::default()
        },
    )
    .map;
    if let Some(t) = s.file_for {
        assets.insert(&named_palette_file(NAME, t).unwrap(), named_file().write());
    }
    (import, catalogue, assets, map)
}

/// The painter inputs of `s` on the test map, drawn by `f`.
fn with_inputs<T>(
    s: &Setup,
    f: impl FnOnce(&Inputs, &mut super::build::Painter, &mut Scene) -> T,
) -> T {
    let (import, catalogue, assets, map) = context_of(s, &scene_map_text());
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
    let mut scene = built.scene;
    f(&inputs, &mut painter, &mut scene)
}

/// A neutral mobile of `kind` at cell (2, 3).
fn mobile(kind: Kind) -> Mobile {
    Mobile {
        kind,
        idx: 0,
        owner: b"Neutral".to_vec(),
        pos: (2 * 256 + 128, 3 * 256 + 128, 0),
        body: 8192,
        turret: 8192,
        elevation: 0,
        anim: 0,
        infantry: (kind == Kind::Infantry).then_some(InfantryFrame {
            start: 3,
            count: 1,
            multiplier: 2,
            stage: 0,
        }),
        object: sim_code(ObjectId(9)),
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

/// The shades of the lit (non-shadow) items.
fn lit_shades(items: &[Item]) -> Vec<Shade> {
    items
        .iter()
        .filter_map(|i| match i.paint {
            Paint::Lit(s) => Some(s),
            _ => None,
        })
        .collect()
}

/// The rows shade `s` reads.
fn table(scene: &Scene, s: Shade) -> Vec<Row565> {
    scene.rows[s.base as usize..][..usize::from(s.rows)].to_vec()
}

/// The 53-shade table of colour definition `hsv` built from `source`
/// with tint `tint` (‰), tint-exempt at 240 to 254 (S-0103 claims 6, 7
/// and 17).
fn expect(source: &Palette, hsv: [u8; 3], tint: [i32; 3]) -> Vec<Row565> {
    shade_table(&house_palette(source, hsv), tint, 53, Some(TINT_EXEMPT))
        .iter()
        .map(row565)
        .collect()
}

/// The named palette of the test file, and the neutral owner's colour
/// definition.
fn named_and_hsv(inputs: &Inputs) -> (Palette, [u8; 3]) {
    (
        inputs.palettes.load_named(&named_file()),
        owner_definition(inputs.import, inputs.options, b"Neutral").0,
    )
}

/// The table of the first lit shade `painter` draws `m` with.
fn mobile_table(
    inputs: &Inputs,
    painter: &mut super::build::Painter,
    scene: &mut Scene,
    m: &Mobile,
) -> Vec<Row565> {
    let (e, _) = painter.mobiles(inputs, scene, std::slice::from_ref(m));
    let s = lit_shades(&e[0].items)[0];
    table(scene, s)
}

/// spec: S-0103 claim 16 (an infantry type with a named palette in the current theater is drawn
/// with the 53-shade named palette of its owner's colour definition) and claim 17 (built as that
/// definition's house-colour palette with the named file, loaded with claim 1's ×4, in place of
/// the unit palette; white tint before any effect)
#[test]
fn claim_16_an_infantry_type_takes_its_named_palette() {
    let s = Setup {
        art: "[MANX]\nPalette=test\n",
        ..Default::default()
    };
    with_inputs(&s, |inputs, painter, scene| {
        let (named, hsv) = named_and_hsv(inputs);
        let t = mobile_table(inputs, painter, scene, &mobile(Kind::Infantry));
        assert_eq!(t, expect(&named, hsv, [1000; 3]));
        assert_ne!(t, expect(&inputs.palettes.unit, hsv, [1000; 3]));
    });
}

/// spec: S-0103 claim 16 (a vehicle drawn as a sprite without a turret takes its named palette;
/// one drawn as a voxel model or as a sprite with a turret keeps its owner's ordinary 53-shade
/// house-colour palette)
#[test]
fn claim_16_only_turretless_sprite_vehicles_take_it() {
    let art = "[CARX]\nPalette=test\n";
    let cases = [
        (false, "", true),
        (true, "", false),
        (false, "[CARX]\nTurret=yes\n", false),
    ];
    for (voxel, rules, named_expected) in cases {
        let s = Setup {
            art,
            rules,
            voxel,
            ..Default::default()
        };
        with_inputs(&s, |inputs, painter, scene| {
            let (named, hsv) = named_and_hsv(inputs);
            let source = if named_expected {
                &named
            } else {
                &inputs.palettes.unit
            };
            let t = mobile_table(inputs, painter, scene, &mobile(Kind::Vehicles));
            assert_eq!(
                t,
                expect(source, hsv, [1000; 3]),
                "voxel {voxel}, rules {rules:?}"
            );
        });
    }
}

/// spec: S-0103 claim 16 (a missing file for the current theater: drawn exactly as if the art
/// named no palette; another theater's file does not substitute) and Import mapping (an empty
/// value names no palette)
#[test]
fn claim_16_a_missing_file_draws_as_without_one() {
    for (art, file_for) in [
        ("[MANX]\nPalette=test\n", None),
        ("[MANX]\nPalette=test\n", Some(Theater::Snow)),
        ("[MANX]\nPalette=\n", Some(Theater::Temperate)),
    ] {
        let s = Setup {
            art,
            file_for,
            ..Default::default()
        };
        with_inputs(&s, |inputs, painter, scene| {
            let (_, hsv) = named_and_hsv(inputs);
            let t = mobile_table(inputs, painter, scene, &mobile(Kind::Infantry));
            assert_eq!(t, expect(&inputs.palettes.unit, hsv, [1000; 3]), "{art:?}");
        });
    }
}

/// spec: S-0103 claim 17 (in our engine every lighting re-evaluation and every trigger-action
/// tint change retints every named palette with the tint of the 53-shade house-colour palettes;
/// "normal" takes it back); S-0104 claims 3 and 5; Formulas, "Map value storage" (‰ =
/// hundredths × 10)
#[test]
fn claim_17_an_effect_tint_retints_the_named_palettes() {
    // An invented effect tint, hundredths.
    const TINT: [i32; 3] = [120, 80, 150];
    let s = Setup {
        art: "[MANX]\nPalette=test\n",
        ..Default::default()
    };
    with_inputs(&s, |inputs, painter, scene| {
        let (named, hsv) = named_and_hsv(inputs);
        let man = mobile(Kind::Infantry);
        let (e, _) = painter.mobiles(inputs, scene, std::slice::from_ref(&man));
        let mut kept = e[0].items.clone();
        painter.set_effect_tint(Some(TINT));
        let tinted = expect(&named, hsv, TINT.map(|t| t * 10));
        assert_eq!(mobile_table(inputs, painter, scene, &man), tinted, "drawn");
        // A drawing kept from frame to frame is shaded again with the same palette.
        assert!(painter.retint_houses(inputs, scene, &mut kept));
        assert_eq!(table(scene, lit_shades(&kept)[0]), tinted, "kept");
        painter.set_effect_tint(None);
        assert_eq!(
            mobile_table(inputs, painter, scene, &man),
            expect(&named, hsv, [1000; 3]),
            "normal"
        );
    });
}

/// The map text of one neutral HUTX on (6, 10).
const HUT_MAP: &[u8] = b"[Basic]\nNewINIFormat=4\n[Map]\nSize=0,0,10,10\nLocalSize=0,0,10,10\n\
    [Structures]\n0=Neutral,HUTX,256,6,10,0\n";

/// The static scene of [`HUT_MAP`] in setup `s`, with the slot animation
/// SLOTA as the hut's idle animation, its named palette and the neutral
/// colour definition.
fn hut_scene(s: &Setup) -> (super::build::Built, Palette, [u8; 3], PaletteState) {
    let (mut import, catalogue, mut assets, map) = context_of(s, HUT_MAP);
    import
        .arts
        .insert((Kind::Animations, 0), art_of("SLOTA", "slota.shp"));
    assets.insert(b"slota.shp", shp((20, 20), 6, (2, 2), (9, 9)));
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let built = build(&map, &import, &catalogue, &palettes, &assets, &options);
    let named = palettes.load_named(&named_file());
    let hsv = owner_definition(&import, &options, b"Neutral").0;
    (built, named, hsv, palettes)
}

/// The rules and art of a hut whose idle slot animation is SLOTA.
const SLOT_RULES: &str = "[Animations]\n0=SLOTA\n";

/// spec: S-0103 claim 16 (a building's body and sprite parts take its named palette, and so do
/// its own animations drawn with it, claim 8 row 3); S-0108 claim 11 (the set-up starts the idle
/// slot)
#[test]
fn claim_16_a_building_and_its_own_animations_take_it() {
    let s = Setup {
        art: "[HUTX]\nPalette=test\nIdleAnim=SLOTA\n[SLOTA]\nLayer=ground\n",
        rules: SLOT_RULES,
        ..Default::default()
    };
    let (b, named, hsv, _) = hut_scene(&s);
    let want = expect(&named, hsv, [1000; 3]);
    let shades = lit_shades(&b.scene.items);
    assert!(shades.len() >= 2, "a body and a slot animation");
    for s in shades {
        assert_eq!(table(&b.scene, s), want);
    }
}

/// spec: S-0103 claim 16 (a terrain-palette building keeps the tile palette for its body and
/// sprite parts, claim 8; its own animations drawn with it use the named palette)
#[test]
fn claim_16_a_terrain_palette_building_names_only_its_animations() {
    let s = Setup {
        art: "[HUTX]\nPalette=test\nTerrainPalette=yes\nIdleAnim=SLOTA\n[SLOTA]\nLayer=ground\n",
        rules: SLOT_RULES,
        ..Default::default()
    };
    let (b, named, hsv, palettes) = hut_scene(&s);
    let want = expect(&named, hsv, [1000; 3]);
    // The body and its sprite parts take tile tables; only the slot animation can take the
    // named one.
    let (named_items, others): (Vec<Shade>, Vec<Shade>) = lit_shades(&b.scene.items)
        .into_iter()
        .partition(|&s| table(&b.scene, s) == want);
    assert!(!named_items.is_empty(), "its own animation");
    let house = expect(&palettes.unit, hsv, [1000; 3]);
    assert!(
        !others.is_empty() && others.iter().all(|&s| table(&b.scene, s) != house),
        "the body and sprite parts keep the tile palette"
    );
}
