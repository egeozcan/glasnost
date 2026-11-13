//! What the client reads from a mounted install before the menus: the
//! base rules (for the setup screen's values and the sound names), the
//! multiplayer maps with their start positions, the sound and speech
//! databases, and the sound bank.

use std::sync::Arc;

use glasnost_audio::bank::{self, SoundBank};
use glasnost_audio::music_files::InstallMusic;
use glasnost_audio::names::SoundNames;
use glasnost_convert::find::Listing;
use glasnost_convert::map::PreviewSource;
use glasnost_ini::Document;
use glasnost_map::Theater;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::rules::SimRules;
use glasnost_ui::draw::Rgba;
use glasnost_ui::mappick::{MapFacts, Marks};
use glasnost_ui::setup::{MapEntry, SetupRules};
use glasnost_vfs::Vfs;

use super::strings::StringTable;

/// The install's data the menus need.
pub struct Install {
    /// The setup screen's values and ranges.
    pub setup: SetupRules,
    /// The multiplayer maps offered, with their entries.
    pub maps: Vec<(Listing, MapEntry)>,
    /// Each offered map's content hash (`session::content_hash`), in the
    /// order of `maps`: a network room names its map by it.
    pub map_hashes: Vec<String>,
    /// What the map picker knows of each offered map, in the order of
    /// `maps` (empty: nothing known).
    pub map_info: Vec<MapInfo>,
    /// The base rules' hash (`sim_rules_hash`), which a network room's
    /// members must share.
    pub rules_hash: String,
    /// The sound names of the base rules.
    pub names: SoundNames,
    /// The sound database (S-0005 Import mapping), read by S-0701 claims 1
    /// to 3 when a game starts, under that game's quirk switches.
    pub sounds: Option<Document>,
    /// The speech database (S-0005 Import mapping, S-0701 claim 26).
    pub speech: Option<Document>,
    /// The samples found.
    pub bank: SoundBank,
    /// The music (S-0703): the music database, the shuffle and repeat
    /// settings and the track files found, or none when the music database
    /// does not resolve.
    pub music: Option<InstallMusic>,
    /// The cursor palette as the session keeps it (S-0111 claim 19a),
    /// read here, before any game, when the switch keeps it.
    pub cursor_palette: super::session::CursorPaletteKeep,
    /// The string table (S-0015), read once when the program starts and
    /// handed to each game.
    pub strings: Option<StringTable>,
    /// What was found, for the menu's status line.
    pub summary: String,
}

/// The countries the setup screen offers (S-0412 claim 15), in
/// registration order: registration index 0 … 9 with the multiplayer
/// listing flag on. The passive flag, the side and the neutral and special
/// houses' countries are not tested. The resolved display-name condition is
/// not applied: country display names are not read yet (see
/// docs/development.md).
pub fn offered_countries(r: &Ruleset, rules: &SimRules) -> Vec<String> {
    rules
        .country_names
        .iter()
        // spec: S-0412 claim 15 (only registration indices 0 … 9 are offered)
        .take(10)
        .filter(|n| {
            // spec: S-0412 claim 15 and Import mapping (the multiplayer listing flag; absent: off)
            r.sim
                .countries
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(n))
                .is_some_and(|(_, c)| c.econ.multiplay.unwrap_or(false))
        })
        .cloned()
        .collect()
}

/// The swatch of each lobby colour slot 0 … 7 (S-0103 claims 11 and 12):
/// the slot's fixed RGB after the 16-bit step, the same for the open list
/// and the selected choice, and independent of the colour definitions.
pub fn slot_swatches() -> Vec<Rgba> {
    (0..8u8)
        .filter_map(glasnost_render::palette::lobby_swatch_565)
        .map(|p| {
            let (r, g, b) = glasnost_render::compose::expand565(p);
            [r, g, b, 255]
        })
        .collect()
}

/// The swatch of each lobby colour slot 0 … 7 when the person asks for
/// colour-blind house colours (M8 step C3): our colour definition
/// ([`crate::mapview::build::COLOUR_BLIND_SLOTS`]) at full saturation and
/// value as plain RGB, so the setup shows the colours the game will.
pub fn colour_blind_swatches() -> Vec<Rgba> {
    crate::mapview::build::COLOUR_BLIND_SLOTS
        .iter()
        .map(|&[h, s, v]| {
            let c = glasnost_render::palette::hsv_to_rgb(h, s, v);
            [c.r, c.g, c.b, 255]
        })
        .collect()
}

/// The setup screen's values from the base rules (S-0412 claim 1 and
/// Import mapping).
pub fn setup_rules(r: &Ruleset, sim: &SimRules) -> SetupRules {
    let g = &r.sim.general.economy;
    SetupRules {
        // spec: S-0412 Import mapping (Money absent: 10000)
        money: g.money.unwrap_or(10000),
        money_range: (
            // spec: S-0412 Import mapping (MinMoney absent: 2500)
            g.min_money.unwrap_or(2500),
            // spec: S-0412 Import mapping (MaxMoney absent: 10000)
            g.max_money.unwrap_or(10000),
            // spec: S-0412 Import mapping (MoneyIncrement absent: 100)
            g.money_increment.unwrap_or(100).max(1),
        ),
        // spec: S-0412 Import mapping (UnitCount absent: 10)
        units: g.unit_count.unwrap_or(10),
        units_range: (
            // spec: S-0412 Import mapping (MinUnitCount absent: 1)
            g.min_unit_count.unwrap_or(1),
            // spec: S-0412 Import mapping (MaxUnitCount absent: 20)
            g.max_unit_count.unwrap_or(20),
        ),
        // spec: S-0412 claim 1 (the bases setting's default: the base rules' Bases, or yes)
        bases: g.bases.unwrap_or(true),
        flags: [
            // spec: S-0412 Import mapping (ShortGame absent: no)
            g.short_game.unwrap_or(false),
            // spec: S-0412 Import mapping (SuperWeaponsAllowed absent: yes)
            g.super_weapons_allowed.unwrap_or(true),
            // spec: S-0412 Import mapping (Crates absent: yes)
            g.crates.unwrap_or(true),
            // spec: S-0412 Import mapping (BuildOffAlly absent: yes)
            g.build_off_ally.unwrap_or(true),
            // spec: S-0412 Import mapping (MCVRedeploys absent: yes)
            g.mcv_redeploys.unwrap_or(true),
        ],
        // spec: S-0201 claim 7 (the lobby's game-speed index from the rules)
        game_speed: sim.general.lobby_game_speed,
        countries: offered_countries(r, sim),
        colours: slot_swatches(),
    }
}

/// The lobby's player limit of a map file: its start waypoints among 0 …
/// 7 (S-0009 claim 23).
pub fn start_count(bytes: &[u8]) -> u32 {
    start_count_of(&Document::parse(bytes, Default::default(), &mut Vec::new()))
}

/// What the map picker knows of an offered map besides its entry (design:
/// map picker (lead's request 2026-11-13)): read once when the program
/// starts, with the preview kept packed until it is shown.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapInfo {
    /// The size, theater and whether a preview is embedded.
    pub facts: MapFacts,
    /// The embedded preview, still packed (S-0009 claim 24, S-0012).
    pub preview: Option<PreviewSource>,
    /// The preview's start markers (S-0009 claim 25), when they are drawn
    /// from the header block (quirk `preview_markers_from_header`).
    pub marks: Option<Marks>,
}

/// The theater's name as the map picker shows it.
// design: map picker (lead's request 2026-11-13)
pub fn theater_label(t: Theater) -> &'static str {
    match t {
        Theater::Temperate => "Temperate",
        Theater::Snow => "Snow",
        Theater::Urban => "Urban",
        Theater::Desert => "Desert",
        Theater::NewUrban => "New urban",
        Theater::Lunar => "Lunar",
    }
}

/// The map picker's facts of a parsed map file.
pub fn map_info_of(doc: &Document) -> MapInfo {
    let quirks = glasnost_convert::map::MapQuirks::default();
    let f = glasnost_convert::map::list_facts(doc, quirks);
    // spec: S-0009 claim 25 (the numbered markers come from the header block, quirks.preview_markers_from_header; with the switch off, where they go is not specified, so none are drawn: Q-3064)
    let marks = quirks.preview_markers_from_header.then(|| Marks {
        count: f.header.count,
        origin: f.header.origin,
        extent: f.header.extent,
        // spec: S-0009 Import mapping ([Header] WaypointN: pairs 1 … 8 default to (0,0); an unreadable component keeps the value in force, 0)
        pairs: f
            .header
            .pairs
            .iter()
            .take(8)
            .map(|p| {
                let [x, y] = p.unwrap_or([None, None]);
                (x.unwrap_or(0), y.unwrap_or(0))
            })
            .collect(),
    });
    MapInfo {
        facts: MapFacts {
            size: f.size,
            theater: theater_label(f.theater).to_owned(),
            preview: f.preview.is_some(),
        },
        preview: f.preview,
        marks,
    }
}

/// [`start_count`] of a parsed map file.
pub fn start_count_of(doc: &Document) -> u32 {
    let zero_counts = glasnost_convert::map::MapQuirks::default().lobby_counts_zero_waypoint;
    (0..8u8)
        .filter(|k| {
            let key = k.to_string();
            doc.value(b"Waypoints", key.as_bytes()).is_some_and(|v| {
                let v = v.trim_ascii();
                // spec: S-0009 claim 23 (an entry counts when present with a value other than −1; quirks.lobby_counts_zero_waypoint)
                v != b"-1" && (zero_counts || v != b"0")
            })
        })
        .count() as u32
}

/// A database file read through the game's search order and parsed.
async fn database(vfs: &Vfs, name: &str) -> Option<Document> {
    let f = glasnost_convert::inputs::read(vfs, name).await?;
    Some(Document::parse(
        &f.bytes,
        Default::default(),
        &mut Vec::new(),
    ))
}

/// Reads the install's menus data: the base rules resolved, the
/// multiplayer maps with at least two start positions, and the samples.
pub async fn prepare(vfs: &mut Vfs) -> Result<Install, String> {
    // spec: S-0015 claims 1 and 2, S-0005 claim 16 (the string table `ra2md.csf` is read once, when the program starts; one that cannot be resolved, or whose header fails claim 2's checks, stops the program from starting)
    let mut table = super::strings::read(vfs)
        .await
        .map_err(|e| format!("the program cannot start: {e}"))?;
    // spec: S-0015 claim 9 (the importer's warnings of a table read as far as its whole entries)
    for w in std::mem::take(&mut table.warnings) {
        log::warn!("{w}");
    }
    let import = glasnost_convert::import(vfs, &glasnost_convert::ImportOptions::default()).await;
    let resolved = glasnost_rules::resolve_ruleset(
        Some((import.tree.clone(), import.provenance.clone())),
        &[],
    )
    .map_err(|e| format!("the rules did not resolve: {e}"))?;
    let sim = Arc::new(SimRules::new(&resolved.ruleset, &resolved.sim_rules_hash));
    let setup = setup_rules(&resolved.ruleset, &sim);
    if setup.countries.len() < 2 {
        return Err("the rules offer fewer than two countries".into());
    }
    let mut maps = Vec::new();
    let mut map_hashes = Vec::new();
    let mut map_info = Vec::new();
    for l in glasnost_convert::find::find_maps(vfs).await {
        if !l.multiplayer {
            continue;
        }
        let Ok(bytes) = vfs.read(&l.file).await else {
            continue;
        };
        let doc = Document::parse(&bytes, Default::default(), &mut Vec::new());
        let starts = start_count_of(&doc);
        if starts >= 2 {
            let label = l.name.clone();
            maps.push((l, MapEntry { label, starts }));
            map_hashes.push(super::session::content_hash(&bytes));
            map_info.push(map_info_of(&doc));
        }
    }
    if maps.is_empty() {
        return Err("no multiplayer map with two start positions was found".into());
    }
    let names = SoundNames::from_ruleset(&resolved.ruleset);
    // spec: S-0005 Import mapping (`soundmd.ini`, the sound database; `evamd.ini`, the speech database)
    let sounds = database(vfs, "soundmd.ini").await;
    let speech = database(vfs, "evamd.ini").await;
    let (bank, report) = bank::discover(vfs).await;
    // spec: S-0703 claim 8 (the track files are found once, when the program starts, through the mounted archives and loose files)
    let music = InstallMusic::read(vfs).await;
    if let Some(m) = music.as_ref() {
        // spec: S-0703 Edge cases (outside a game the side restrictions are checked against the sides of the rules data)
        let sides: Vec<String> = resolved
            .ruleset
            .sides
            .iter()
            .map(|s| s.name.clone())
            .collect();
        for w in m.db.side_warnings(&sides) {
            log::warn!("{w}");
        }
        // spec: S-0703 claim 8a (our engine: the importer warning for a found sound file that does not parse, naming the entry)
        for w in &m.warnings {
            log::warn!("{w}");
        }
    }
    // spec: S-0111 claim 19a (with quirks.art_cache_never_invalidated on, the cursor palette is read once when the program starts, before any game)
    let cursor_palette = super::session::CursorPaletteKeep::at_program_start(vfs).await;
    let summary = format!(
        "{} maps, {} countries, {} samples, {} music tracks",
        maps.len(),
        setup.countries.len(),
        report.samples,
        music.as_ref().map_or(0, InstallMusic::present_count)
    );
    Ok(Install {
        setup,
        maps,
        map_hashes,
        map_info,
        rules_hash: resolved.sim_rules_hash.clone(),
        names,
        sounds,
        speech,
        bank,
        music,
        cursor_palette,
        strings: Some(table),
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0009 claim 23 (start waypoints among 0 … 7; −1 does not count)
    #[test]
    fn start_positions_are_counted_from_the_waypoints() {
        let text = b"[Waypoints]\n0=1000\n1=2000\n2=-1\n5=3000\n8=4000\n";
        assert_eq!(start_count(text), 3);
        assert_eq!(start_count(b"[Basic]\nName=x\n"), 0);
        let doc = Document::parse(text, Default::default(), &mut Vec::new());
        assert_eq!(start_count_of(&doc), 3);
    }

    fn parse(text: &str) -> Document {
        Document::parse(text.as_bytes(), Default::default(), &mut Vec::new())
    }

    /// The picker's facts of an invented map with a preview and a header
    /// block (S-0009 claims 24, 25 and 30).
    #[test]
    fn map_info_reads_size_theater_preview_and_markers() {
        let text = glasnost_convert::map::synth::MapText {
            ini: "[Map]\nSize=0,0,70,40\nTheater=SNOW\n[Preview]\nSize=0,0,2,1\n\
                  [Header]\nNumberStartingPoints=2\nStartX=1\nStartY=2\nWidth=30\nHeight=20\n\
                  Waypoint1=5,6\nWaypoint2=x,9\nWaypoint3=7,7\n"
                .into(),
            preview: Some(vec![1, 2, 3, 4, 5, 6]),
            ..Default::default()
        }
        .text();
        let info = map_info_of(&parse(&text));
        assert_eq!(
            info.facts,
            MapFacts {
                size: (70, 40),
                theater: "Snow".into(),
                preview: true,
            }
        );
        let p = info.preview.as_ref().expect("a preview").decode();
        assert_eq!((p.width, p.height, p.rgb), (2, 1, vec![1, 2, 3, 4, 5, 6]));
        // spec: S-0009 claim 25 and Import mapping (pairs 1 … N_h; a pair that does not read keeps the value in force, (0,0))
        assert_eq!(
            info.marks,
            Some(Marks {
                count: 2,
                origin: (1, 2),
                extent: (30, 20),
                pairs: vec![(5, 6), (0, 0)],
            })
        );
    }

    /// A map without a preview block, or with an invalid preview size, has
    /// no preview; without a header block its markers read as −1 (none drawn,
    /// S-0009 claim 25); an absent theater is temperate.
    #[test]
    fn map_info_without_a_preview_or_header() {
        let info = map_info_of(&parse("[Map]\nSize=0,0,50,60\n[Preview]\nSize=0,0,4,4\n"));
        assert!(!info.facts.preview && info.preview.is_none());
        assert_eq!(info.facts.theater, "Temperate");
        let m = info.marks.expect("the header is read");
        assert_eq!((m.count, m.extent), (-1, (-1, -1)));
        assert!(
            glasnost_ui::mappick::marker_corners(&m, glasnost_ui::geom::Rect::new(0, 0, 9, 9))
                .is_empty()
        );
        let text = glasnost_convert::map::synth::MapText {
            ini: "[Preview]\nSize=0,0,0,4\n".into(),
            preview: Some(vec![9; 12]),
            ..Default::default()
        }
        .text();
        assert!(!map_info_of(&parse(&text)).facts.preview);
    }

    /// spec: S-0103 claims 11 and 12 (the setup swatches are the fixed slot colours after the 16-bit step: flat, eight slots, independent of the colour definitions) (Q-0280)
    #[test]
    fn lobby_swatches_are_fixed_colours() {
        let s = slot_swatches();
        assert_eq!(s.len(), 8);
        let (r, g, b) = glasnost_render::compose::expand565(glasnost_render::palette::hicolor_565(
            glasnost_render::palette::LOBBY_SWATCHES[1],
        ));
        assert_eq!(s[1], [r, g, b, 255]);
        assert_eq!(s[1][0], 255);
        // No colour definition is consulted: the swatches take no rules input.
        assert_ne!(s[0], s[1]);
        assert_eq!(glasnost_render::palette::lobby_swatch_565(8), None);
    }

    /// Our colour-blind swatches (M8 step C3) are eight distinct opaque colours, none of them
    /// the fixed slot swatch of the same slot.
    #[test]
    fn colour_blind_swatches_differ_from_the_slot_swatches() {
        let ours = colour_blind_swatches();
        let fixed = slot_swatches();
        assert_eq!(ours.len(), 8);
        for (i, c) in ours.iter().enumerate() {
            assert_eq!(c[3], 255);
            assert_ne!(Some(c), fixed.get(i));
            assert!(!ours[i + 1..].contains(c));
        }
    }

    // spec: S-0412 claim 15 (registration indices 0 … 9 with the listing flag on, in registration order; passive countries are not excluded)
    #[test]
    fn claim_15_offered_countries() {
        let mut r = glasnost_ai::fixture::ruleset();
        let names: Vec<String> = (0..12).map(|i| format!("Land{i}")).collect();
        r.registries.insert("countries".into(), names.clone());
        for (i, n) in names.iter().enumerate() {
            let mut c = glasnost_rules::sim::Country::default();
            c.econ.multiplay = Some(i != 2);
            c.multiplay_passive = Some(i == 1);
            r.sim.countries.insert(n.clone(), c);
        }
        let sim = SimRules::new(&r, "synthetic-countries");
        let got = offered_countries(&r, &sim);
        let want: Vec<String> = [0, 1, 3, 4, 5, 6, 7, 8, 9]
            .iter()
            .map(|i| format!("Land{i}"))
            .collect();
        assert_eq!(got, want);
    }

    #[test]
    fn content_hash_names_maps_by_content() {
        let a = super::super::session::content_hash(b"abc");
        assert_eq!(a, super::super::session::content_hash(b"abc"));
        assert_ne!(a, super::super::session::content_hash(b"abd"));
        assert!(a.starts_with("map-"));
    }
}
