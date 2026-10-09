//! Loading a map for viewing: decode it (S-0009), import the rules with the
//! map as the map layer (S-0006), read the theater's palettes (S-0103), its
//! tile catalogue (Q-0009) and every file the scene needs, then build the
//! scene.

use glasnost_convert::map::{DecodeContext, Decoded, decode, to_vfs_theater};
use glasnost_convert::theater::{Catalogue, data_file_names};
use glasnost_convert::{Import, ImportOptions, inputs::InputFile};
use glasnost_fmt::pal::Palette6;
use glasnost_ini::Document;
use glasnost_map::Map;
use glasnost_render::palette::{ANIM_PALETTE_FILE, PaletteQuirks, PaletteState};
use glasnost_vfs::Vfs;
use glasnost_vfs::theater::Theater;

use super::build::{self, Assets, Built, Options};

/// A loaded map.
pub struct Loaded {
    /// The decoded map.
    pub map: Map,
    /// Importer warnings about the map file.
    pub map_warnings: Vec<String>,
    /// The rules import with the map layer.
    pub import: Import,
    /// The theater's tile catalogue.
    pub catalogue: Catalogue,
    /// The palettes in force.
    pub palettes: PaletteState,
    /// Files read for the scene.
    pub assets: Assets,
    /// The scene.
    pub built: Built,
    /// Minimap (width, height, RGBA).
    pub minimap: (u32, u32, Vec<u8>),
    /// Problems that kept parts of the map from being drawn.
    pub problems: Vec<String>,
}

async fn read_named(vfs: &Vfs, name: &str) -> Option<Vec<u8>> {
    let r = vfs.resolve(name)?;
    vfs.read_resolved(&r).await.ok()
}

/// Loads `bytes` (a map file found at `origin`) from the install mounted in
/// `vfs`, whose search order is built.
pub async fn load(vfs: &mut Vfs, bytes: Vec<u8>, origin: String, options: &Options) -> Loaded {
    let mut problems = Vec::new();
    let doc = Document::parse(&bytes, Default::default(), &mut Vec::new());
    let first = decode(&doc, &DecodeContext::default());
    let theater: Theater = to_vfs_theater(first.map.theater);
    let import = glasnost_convert::import(
        vfs,
        &ImportOptions {
            theater: Some(theater),
            map: Some(InputFile {
                name: "map".into(),
                origin,
                bytes: bytes.clone(),
            }),
            ..Default::default()
        },
    )
    .await;
    problems.extend(import.errors.iter().cloned());
    // provisional (Q-0009): the theater data file
    let mut catalogue = Catalogue::default();
    let mut found = false;
    for name in data_file_names(theater) {
        if let Some(b) = read_named(vfs, &name).await {
            let tdoc = Document::parse(&b, Default::default(), &mut Vec::new());
            catalogue = Catalogue::read(&tdoc, theater);
            found = true;
            break;
        }
    }
    if !found {
        problems.push("the theater's data file was not found (Q-0009): no terrain is drawn".into());
    }
    let Decoded { map, warnings } = decode(
        &doc,
        &DecodeContext {
            catalogue: Some(&catalogue),
            ..Default::default()
        },
    );
    let map_warnings = warnings
        .iter()
        .map(|w| match w.line {
            Some(l) => format!("line {l}: {}", w.text),
            None => w.text.clone(),
        })
        .collect();
    let pal = |b: Option<Vec<u8>>| b.and_then(|b| Palette6::parse(&b).ok());
    // spec: S-0103 State and ordering (program start: the snow unit palette and the anim palette)
    let mut palettes = PaletteState::at_program_start(
        pal(read_named(vfs, &Theater::Snow.unit_palette()).await).as_ref(),
        pal(read_named(vfs, ANIM_PALETTE_FILE).await).as_ref(),
        PaletteQuirks::default(),
    );
    let mut files = std::collections::BTreeMap::new();
    for name in [
        theater.unit_palette(),
        theater.tile_palette(),
        theater.theater_palette(),
    ] {
        files.insert(name.clone(), pal(read_named(vfs, &name).await));
    }
    // spec: S-0103 claim 3 (a game in a new theater loads its palettes)
    palettes.start_game(theater, |n| files.get(n).cloned().flatten());
    let mut assets = Assets::default();
    for name in build::wanted_files(&map, &import, &catalogue) {
        let Ok(text) = std::str::from_utf8(&name) else {
            continue;
        };
        if let Some(b) = read_named(vfs, text).await {
            assets.insert(&name, b);
        }
    }
    let built = build::build(&map, &import, &catalogue, &palettes, &assets, options);
    let minimap = build::minimap(&map, &catalogue, &assets);
    Loaded {
        map,
        map_warnings,
        import,
        catalogue,
        palettes,
        assets,
        built,
        minimap,
        problems,
    }
}
