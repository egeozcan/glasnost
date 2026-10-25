//! Everything a game on one map needs from the install: the rules import
//! with the map as its map layer (S-0006), the resolved typed ruleset, the
//! theater's tile catalogue (S-0010 claims 1 to 5) and the decoded map
//! (S-0009). Shared by the map viewer and the headless runner.

use glasnost_ini::Document;
use glasnost_map::Map;
use glasnost_map::terrain::TileCatalogue;
use glasnost_rules::Resolved;
use glasnost_rules::layers::Layers;
use glasnost_vfs::Vfs;
use glasnost_vfs::theater::Theater;

use crate::inputs::InputFile;
use crate::map::{DecodeContext, Decoded, decode, to_vfs_theater};
use crate::theater::{Catalogue, data_file_names};
use crate::{Import, ImportOptions};

/// A map's game inputs.
pub struct GameInputs {
    /// The decoded map.
    pub map: Map,
    /// Importer warnings about the map file.
    pub map_warnings: Vec<String>,
    /// The rules import with the map layer.
    pub import: Import,
    /// The resolved ruleset, or why it could not be resolved.
    pub resolved: Result<Resolved, String>,
    /// The theater's catalogue (tilesets, roles, tile files).
    pub catalogue: Catalogue,
    /// The simulation's tile catalogue (terrain and slope codes), when the
    /// theater data file was found.
    pub tiles: Option<TileCatalogue>,
    /// Problems: import errors, a missing theater data file, catalogue
    /// warnings.
    pub problems: Vec<String>,
}

async fn read_named(vfs: &Vfs, name: &str) -> Option<Vec<u8>> {
    let r = vfs.resolve(name)?;
    vfs.read_resolved(&r).await.ok()
}

/// Loads the game inputs of `bytes` (a map file found at `origin`) from the
/// install mounted in `vfs`, whose search order is built, with our own patch
/// layers `layers` after the converted tree ([`Layers::REFERENCE`]: none,
/// the reference ruleset; M8 step C7).
pub async fn load(vfs: &mut Vfs, bytes: Vec<u8>, origin: String, layers: Layers) -> GameInputs {
    let mut problems = Vec::new();
    let doc = Document::parse(&bytes, Default::default(), &mut Vec::new());
    let first = decode(&doc, &DecodeContext::default());
    let theater: Theater = to_vfs_theater(first.map.theater);
    let import = crate::import(
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
    let resolved = glasnost_rules::resolve_ruleset(
        Some((import.tree.clone(), import.provenance.clone())),
        &layers.sources(),
    )
    .map_err(|e| e.to_string());
    let mut catalogue = Catalogue::default();
    let mut tiles = None;
    // spec: S-0010 claim 1 (the theater data file, by table T)
    for name in data_file_names(theater) {
        if let Some(b) = read_named(vfs, &name).await {
            let tdoc = Document::parse(&b, Default::default(), &mut Vec::new());
            catalogue = Catalogue::read(&tdoc, theater);
            problems.extend(catalogue.warnings.iter().cloned());
            let (t, warnings) = catalogue.tile_catalogue_from(vfs).await;
            problems.extend(warnings);
            tiles = Some(t);
            break;
        }
    }
    if tiles.is_none() {
        problems.push("the theater's data file was not found: no terrain".into());
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
    GameInputs {
        map,
        map_warnings,
        import,
        resolved,
        catalogue,
        tiles,
        problems,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic map with no install behind it: the converted tree is
    /// empty, so the rules are our schema's defaults.
    const MAP: &[u8] = b"[Map]\nTheater=TEMPERATE\nSize=0,0,4,4\nLocalSize=0,0,4,4\n";

    /// The loader resolves the rules with the layers asked for (M8 step
    /// C7): none gives the defaults; the play-fixes layer turns its
    /// switches off and changes the rules hash.
    #[test]
    fn the_rules_take_the_layers_asked_for() {
        let load_with = |layers| {
            pollster::block_on(load(
                &mut Vfs::default(),
                MAP.to_vec(),
                "synthetic".into(),
                layers,
            ))
            .resolved
            .expect("the rules resolve")
        };
        let plain = load_with(Layers::REFERENCE);
        let fixed = load_with(Layers::PLAY_FIXES);
        assert_eq!(plain.ruleset.globals.quirks, Default::default());
        assert_eq!(
            fixed.ruleset.globals.quirks,
            Layers::PLAY_FIXES.apply_quirks(Default::default()).unwrap()
        );
        assert_ne!(plain.sim_rules_hash, fixed.sim_rules_hash);
        assert_eq!(
            plain.sim_rules_hash,
            load_with(Layers::REFERENCE).sim_rules_hash
        );
    }
}
