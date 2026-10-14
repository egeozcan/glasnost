//! Art lookup (S-0007): from a type's rules and art settings to the names
//! of the art files the game loads for one theater.
//!
//! Every type read of a game (S-0007 Terms) is replayed in order: the image
//! name and its arctic suffix, the art section's flags, the sprite or voxel
//! files, the sidebar icons. The theater refresh then rebuilds the
//! theater-dependent art of overlays, terrain objects, smudges, buildings
//! and animations. Names resolve through [`ArtFiles`], which answers as the
//! game's search order does (S-0005).

use std::collections::BTreeMap;

use glasnost_ini::value::{self, boolean, text};
use glasnost_ini::{Document, Section};
use glasnost_vfs::theater::Theater;

use crate::kinds::Kind;
use crate::layers::{Outcome, TypeRec, What};
use crate::names::show;

/// Longest image name (S-0007 Constants).
pub(crate) const IMAGE_MAX: usize = 24; // spec: S-0007 Constants (image name length)
/// Longest building art-section image key and secondary sprite name.
const BUILDING_NAME_MAX: usize = 63; // spec: S-0007 Constants (building names)
/// Longest build-up sprite name.
const BUILDUP_MAX: usize = 15; // spec: S-0007 Constants (build-up sprite name length)
/// Longest icon and translucent-sprite name.
const ICON_MAX: usize = 24; // spec: S-0007 Constants (icon name lengths)
/// Largest multiple-turret count the original supports (S-0007 claim 17).
const TURRET_COUNT_MAX: i32 = 18; // spec: S-0007 claim 17
/// Placeholder main icon (S-0007 claim 22).
const PLACEHOLDER_ICON: &[u8] = b"XXICON.SHP"; // spec: S-0007 Constants (placeholder icon)
/// Position from which a "TUR" in a building's turret-animation name
/// counts (1-based, S-0007 claim 20).
const LATE_TUR_START: usize = 5; // spec: S-0007 Constants (late "TUR" start)

/// Building secondary sprites (S-0007 claim 11, Import mapping), with our
/// role names.
// spec: S-0007 Import mapping (building secondary sprite keys)
const SECONDARY: [(&str, &str); 8] = [
    ("deploying", "DeployingAnim"),
    ("roof_deploying", "RoofDeployingAnim"),
    ("door", "DoorAnim"),
    ("under_door", "UnderDoorAnim"),
    ("under_roof_door", "UnderRoofDoorAnim"),
    ("rubble", "Rubble"),
    ("special_overlay", "SpecialZOverlay"),
    ("bib", "BibShape"),
];

/// Answers file questions as the game's search order does.
pub trait ArtFiles {
    /// Whether `name` resolves (S-0005), matched without regard to case.
    fn exists(&self, name: &[u8]) -> bool;
    /// Whether a resolved model or motion file is valid.
    fn valid(&self, name: &[u8]) -> bool;
}

/// Quirk switches of S-0007. Each defaults to the original's behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtQuirks {
    /// `quirks.art_generic_fallback_any_name` (claim 8).
    pub art_generic_fallback_any_name: bool,
    /// `quirks.arctic_suffix_on_flag_turning_on` (claim 10).
    pub arctic_suffix_on_flag_turning_on: bool,
    /// `quirks.building_substitution_ignores_flag` (claim 11).
    pub building_substitution_ignores_flag: bool,
    /// `quirks.theater_art_uses_type_name` (claims 13 and 14).
    pub theater_art_uses_type_name: bool,
    /// `quirks.theater_refresh_building_no_fallback` (claim 12).
    pub theater_refresh_building_no_fallback: bool,
    /// `quirks.apc_water_body` (claim 16).
    pub apc_water_body: bool,
    /// `quirks.voxel_part_failure_discards_all` (claim 18).
    pub voxel_part_failure_discards_all: bool,
    /// `quirks.building_turret_without_tur_in_barrel_role` (claim 20).
    pub building_turret_without_tur_in_barrel_role: bool,
    /// `quirks.building_tur_search_skips_four` (claim 20).
    pub building_tur_search_skips_four: bool,
    /// `quirks.loading_archive_sprites_vanish` (S-0005 claim 13).
    pub loading_archive_sprites_vanish: bool,
    /// `quirks.sequence_value_length_limit` (S-0107 claim 22): the
    /// `Sequence=` value and each kind value are read as at most 31
    /// characters. Off: read whole.
    pub sequence_value_length_limit: bool,
    /// `quirks.case_sensitive_direction` (S-0107 claim 22): a declared
    /// direction is recognised only in exact upper case. Off: any case.
    pub case_sensitive_direction: bool,
}

impl Default for ArtQuirks {
    fn default() -> Self {
        // spec: S-0007 Edge cases, S-0005 Edge cases (each defaults to the original)
        Self {
            art_generic_fallback_any_name: true,
            arctic_suffix_on_flag_turning_on: true,
            building_substitution_ignores_flag: true,
            theater_art_uses_type_name: true,
            theater_refresh_building_no_fallback: true,
            apc_water_body: true,
            voxel_part_failure_discards_all: true,
            building_turret_without_tur_in_barrel_role: true,
            building_tur_search_skips_four: true,
            loading_archive_sprites_vanish: true,
            // spec: S-0107 claim 22 (both switches default to the original)
            sequence_value_length_limit: true,
            case_sensitive_direction: true,
        }
    }
}

impl ArtQuirks {
    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); 12] {
        [
            (
                "art_generic_fallback_any_name",
                self.art_generic_fallback_any_name,
            ),
            (
                "arctic_suffix_on_flag_turning_on",
                self.arctic_suffix_on_flag_turning_on,
            ),
            (
                "building_substitution_ignores_flag",
                self.building_substitution_ignores_flag,
            ),
            (
                "theater_art_uses_type_name",
                self.theater_art_uses_type_name,
            ),
            (
                "theater_refresh_building_no_fallback",
                self.theater_refresh_building_no_fallback,
            ),
            ("apc_water_body", self.apc_water_body),
            (
                "voxel_part_failure_discards_all",
                self.voxel_part_failure_discards_all,
            ),
            (
                "building_turret_without_tur_in_barrel_role",
                self.building_turret_without_tur_in_barrel_role,
            ),
            (
                "building_tur_search_skips_four",
                self.building_tur_search_skips_four,
            ),
            (
                "loading_archive_sprites_vanish",
                self.loading_archive_sprites_vanish,
            ),
            (
                "sequence_value_length_limit",
                self.sequence_value_length_limit,
            ),
            ("case_sensitive_direction", self.case_sensitive_direction),
        ]
    }
}

/// A type's art for one theater.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeArt {
    /// Image name after every read (claims 1, 9 and 10).
    pub image: Vec<u8>,
    /// Flags in force: `theater`, `new_theater`, `voxel`, `arctic`.
    pub flags: Vec<&'static str>,
    /// Sprite file seen in play.
    pub sprite: Option<Vec<u8>>,
    /// Voxel model files by role (each with its motion file).
    pub voxels: BTreeMap<String, Vec<u8>>,
    /// Other building sprites by role.
    pub building_sprites: BTreeMap<&'static str, Vec<u8>>,
    /// Main icon.
    pub icon: Option<Vec<u8>>,
    /// Alternate icon.
    pub alt_icon: Option<Vec<u8>>,
    /// Translucent sprite.
    pub translucent: Option<Vec<u8>>,
    /// The image-frame count *N_b* of the build-up sprite, read from the
    /// file while the loading archives are mounted (S-0108 claim 5); none
    /// when there is no build-up sprite or the file does not parse.
    pub buildup_frames: Option<i32>,
    /// Notes on open or unusual cases.
    pub notes: Vec<String>,
}

fn with_ext(name: &[u8], ext: &str) -> Vec<u8> {
    let mut f = name.to_vec();
    f.push(b'.');
    f.extend_from_slice(ext.as_bytes());
    f
}

/// Letter substitution (S-0007 claim 7, Formulas `subst`).
pub fn subst(f: &[u8], letter: u8) -> Vec<u8> {
    let mut out = f.to_vec();
    // spec: S-0007 claim 7 (first character G, N, C or Y; second A or T; any case)
    if out.len() >= 2
        && matches!(out[0].to_ascii_uppercase(), b'G' | b'N' | b'C' | b'Y')
        && matches!(out[1].to_ascii_uppercase(), b'A' | b'T')
    {
        out[1] = letter;
    }
    out
}

/// The generic fallback name (S-0007 claim 8, Formulas `generic`).
pub fn generic(f: &[u8]) -> Vec<u8> {
    let mut out = f.to_vec();
    if out.len() >= 2 {
        // spec: S-0007 claim 8 (the second character becomes G)
        out[1] = b'G';
    }
    out
}

struct Ctx<'a> {
    art: &'a Document,
    files: &'a dyn ArtFiles,
    theater: Theater,
    q: ArtQuirks,
    warnings: Vec<String>,
}

impl Ctx<'_> {
    fn ext(&self) -> String {
        self.theater.extension().to_ascii_lowercase()
    }

    /// `found` of S-0007 Formulas, with the generic fallback when allowed.
    /// `substituted` says whether letter substitution changed the name
    /// (the fixed behaviour of claim 8 retries only such names).
    fn found(&self, f: &[u8], fallback: bool, substituted: bool) -> Option<Vec<u8>> {
        if self.files.exists(f) {
            return Some(f.to_vec());
        }
        // spec: S-0007 claim 8 (retry once with G as the second character)
        if fallback && (self.q.art_generic_fallback_any_name || substituted) {
            let g = generic(f);
            if self.files.exists(&g) {
                return Some(g);
            }
        }
        None
    }

    fn exists_then(&self, f: Vec<u8>) -> Option<Vec<u8>> {
        self.files.exists(&f).then_some(f)
    }

    fn subst_found(&self, f: &[u8], apply: bool) -> Option<Vec<u8>> {
        let s = if apply {
            subst(f, self.theater.letter())
        } else {
            f.to_vec()
        };
        let changed = s != f;
        self.found(&s, true, changed)
    }
}

fn section_bool(s: Option<&Section>, key: &str) -> Option<bool> {
    s.and_then(|s| s.value(key.as_bytes())).and_then(boolean)
}

fn section_text(s: Option<&Section>, key: &str, limit: usize) -> Option<Vec<u8>> {
    s.and_then(|s| s.value(key.as_bytes()))
        .map(|v| text(v, limit).to_vec())
}

/// The stored value a type's rules section gave `key` in layer `layer`.
pub(crate) fn given_at<'a>(t: &'a TypeRec, key: &str, layer: usize) -> Option<&'a [u8]> {
    t.keys
        .get(key.as_bytes())?
        .history
        .iter()
        .rev()
        .find_map(|e| match &e.what {
            What::Set(v) if e.layer == layer => Some(v.as_slice()),
            _ => None,
        })
}

/// A boolean key's value in force after layer `layer` (S-0003 claim 2: an
/// unrecognised value keeps the value in force).
pub(crate) fn bool_as_of(t: &TypeRec, key: &str, layer: usize, default: bool) -> bool {
    let Some(k) = t.keys.get(key.as_bytes()) else {
        return default;
    };
    k.history
        .iter()
        .filter(|e| e.layer <= layer)
        .fold(default, |v, e| match &e.what {
            What::Set(s) => boolean(s).unwrap_or(v),
            _ => v,
        })
}

/// An integer key's value in force after layer `layer` (S-0003 claim 3).
pub(crate) fn int_as_of(t: &TypeRec, key: &str, layer: usize, default: i32) -> i32 {
    let Some(k) = t.keys.get(key.as_bytes()) else {
        return default;
    };
    k.history
        .iter()
        .filter(|e| e.layer <= layer)
        .fold(default, |v, e| match &e.what {
            What::Set(s) => value::integer(s).unwrap_or(v),
            _ => v,
        })
}

/// A text key's value in force after layer `layer`.
pub(crate) fn text_as_of(t: &TypeRec, key: &str, layer: usize, limit: usize) -> Option<Vec<u8>> {
    let k = t.keys.get(key.as_bytes())?;
    k.history
        .iter()
        .filter(|e| e.layer <= layer)
        .fold(None, |v, e| match &e.what {
            What::Set(s) => Some(text(s, limit).to_vec()),
            _ => v,
        })
}

/// Kinds whose art this module looks up.
pub const KINDS: [Kind; 8] = [
    Kind::Infantry,
    Kind::Vehicles,
    Kind::Aircraft,
    Kind::Buildings,
    Kind::Animations,
    Kind::Overlays,
    Kind::TerrainObjects,
    Kind::Smudges,
];

/// Looks up the art of every type of the kinds in [`KINDS`]. Returns the
/// art by (kind, registration index) and importer warnings.
pub fn lookup(
    outcome: &Outcome,
    art: &Document,
    theater: Theater,
    files: &dyn ArtFiles,
    q: &ArtQuirks,
) -> (BTreeMap<(Kind, usize), TypeArt>, Vec<String>) {
    let mut ctx = Ctx {
        art,
        files,
        theater,
        q: *q,
        warnings: Vec::new(),
    };
    let mut out = BTreeMap::new();
    for kind in KINDS {
        for (i, t) in outcome.registries[&kind].iter().enumerate() {
            let a = if kind == Kind::Animations {
                animation(&mut ctx, t)
            } else {
                object(&mut ctx, kind, t)
            };
            out.insert((kind, i), a);
        }
    }
    (out, ctx.warnings)
}

/// The art section's flags, kept across reads (claim 2).
#[derive(Clone, Copy, Debug, Default)]
struct Flags {
    theater: bool,
    new_theater: bool,
    voxel: bool,
}

/// A building's, infantry's, vehicle's or aircraft's image name walked
/// read by read (claims 1, 2, 4, 9 and 10): the image key replaces the
/// name, the flags come from the art section of the name before this
/// read's suffix, and on snow an A is appended where the arctic flag turns
/// on, never with the voxel flag. The art lookup and the importer's
/// art-file references (S-0004 claim 14, claim 33) share it, so both
/// select the same section at each read.
pub(crate) struct ImageWalk {
    /// The image name in force after the last step.
    pub(crate) image: Vec<u8>,
    flags: Flags,
    arctic_prev: bool,
    mobile: bool,
    snow: bool,
    on_turning_on: bool,
}

impl ImageWalk {
    /// The walk before a type's first read: the image name is the type
    /// name (claim 1).
    pub(crate) fn new(t: &TypeRec, kind: Kind, snow: bool, q: &ArtQuirks) -> Self {
        ImageWalk {
            image: t.name.clone(),
            flags: Flags::default(),
            arctic_prev: false,
            mobile: matches!(kind, Kind::Infantry | Kind::Vehicles | Kind::Aircraft),
            snow,
            on_turning_on: q.arctic_suffix_on_flag_turning_on,
        }
    }

    /// The type read of layer `r`.
    pub(crate) fn step(&mut self, art: &Document, t: &TypeRec, r: usize) {
        // spec: S-0007 claim 1 (the image key replaces the image name; a missing key keeps it)
        if let Some(v) = given_at(t, "Image", r) {
            self.image = text(v, IMAGE_MAX).to_vec();
        }
        // spec: S-0007 claims 2 and 9 (flags from the art section of the name before this read's suffix)
        let pre = art.section(&self.image);
        let f = &mut self.flags;
        f.theater = section_bool(pre, "Theater").unwrap_or(f.theater);
        f.new_theater = section_bool(pre, "NewTheater").unwrap_or(f.new_theater);
        if self.mobile {
            // spec: S-0007 claim 4 (the voxel flag, for infantry, vehicles and aircraft)
            f.voxel = section_bool(pre, "Voxel").unwrap_or(f.voxel);
        }
        let arctic = bool_as_of(t, "AlternateArcticArt", r, false);
        // spec: S-0007 claims 9 and 10 (the A is appended where the flag turns on, on snow, never with the voxel flag)
        if self.on_turning_on && self.snow && arctic && !self.arctic_prev && !f.voxel {
            let mut s = self.image.clone();
            s.push(b'A');
            self.image = text(&s, IMAGE_MAX).to_vec();
        }
        self.arctic_prev = arctic;
    }
}

fn object(ctx: &mut Ctx, kind: Kind, t: &TypeRec) -> TypeArt {
    let mut a = TypeArt {
        image: t.name.clone(),
        ..TypeArt::default()
    };
    if t.reads.is_empty() {
        a.notes
            .push("no type read: no art is built (S-0007 Terms, type read)".into());
        return a;
    }
    let mobile = matches!(kind, Kind::Infantry | Kind::Vehicles | Kind::Aircraft);
    let buildable = mobile || kind == Kind::Buildings;
    let snow = ctx.theater == Theater::Snow;
    let ext = ctx.ext();
    let mut walk = ImageWalk::new(t, kind, snow, &ctx.q);
    let mut flags = Flags::default();
    let mut buildup: Option<Vec<u8>> = None;
    let mut icon_set = false;
    let last = *t.reads.last().unwrap_or(&0);
    for &r in &t.reads {
        // spec: S-0007 claims 1, 2, 4, 9 and 10 (the image name and flags of this read)
        walk.step(ctx.art, t, r);
        a.image.clone_from(&walk.image);
        flags = walk.flags;
        let sec = ctx.art.section(&a.image);
        let image = a.image.clone();
        match kind {
            Kind::Infantry | Kind::Vehicles | Kind::Aircraft | Kind::Overlays => {
                a.sprite = if flags.voxel {
                    None
                } else if flags.theater {
                    // spec: S-0007 claim 6 (theater extension, no letter substitution)
                    ctx.found(&with_ext(&image, &ext), true, false)
                } else {
                    // spec: S-0007 claims 5 and 7 (letter substitution with the new-theater flag)
                    ctx.subst_found(&with_ext(&image, "shp"), flags.new_theater)
                };
            }
            Kind::TerrainObjects => {
                // spec: S-0007 claim 13 (no fallback, no substitution)
                a.sprite = if flags.theater {
                    None
                } else {
                    ctx.exists_then(with_ext(&image, "shp"))
                };
            }
            Kind::Smudges => {
                // spec: S-0007 claim 13
                let e = if flags.theater { ext.as_str() } else { "shp" };
                a.sprite = ctx.exists_then(with_ext(&image, e));
            }
            Kind::Buildings => {
                let subst_on = ctx.q.building_substitution_ignores_flag || flags.new_theater;
                // spec: S-0007 claim 11 (the art section's own image key, afresh at every read)
                let j = section_text(sec, "Image", BUILDING_NAME_MAX)
                    .filter(|j| !j.is_empty())
                    .unwrap_or_else(|| image.clone());
                let e = if flags.theater { ext.as_str() } else { "shp" };
                a.sprite = ctx.subst_found(&with_ext(&j, e), subst_on);
                // spec: S-0007 claim 11 (the build-up name, cut to 15; unset or empty keeps the earlier one)
                if let Some(b) = section_text(sec, "Buildup", BUILDUP_MAX).filter(|b| !b.is_empty())
                {
                    buildup = Some(b);
                }
                match &buildup {
                    Some(b) => {
                        if let Some(f) = ctx.subst_found(&with_ext(b, "shp"), subst_on) {
                            a.building_sprites.insert("buildup", f);
                        } else {
                            a.building_sprites.remove("buildup");
                        }
                    }
                    None => {
                        a.building_sprites.remove("buildup");
                    }
                }
                for (role, key) in SECONDARY {
                    // spec: S-0007 claim 11 (a later read whose art section does not set the name, or sets it empty, keeps the sprite)
                    let Some(n) =
                        section_text(sec, key, BUILDING_NAME_MAX).filter(|n| !n.is_empty())
                    else {
                        continue;
                    };
                    // spec: S-0007 claim 11 (set: the file found replaces the sprite; none found leaves none)
                    match ctx.subst_found(&with_ext(&n, "shp"), subst_on) {
                        Some(f) => {
                            a.building_sprites.insert(role, f);
                        }
                        None => {
                            a.building_sprites.remove(role);
                        }
                    }
                }
            }
            _ => {}
        }
        if buildable {
            // spec: S-0007 claim 22 (the main icon, set at the first read only)
            if !icon_set {
                icon_set = true;
                let c = section_text(sec, "Cameo", ICON_MAX).unwrap_or_default();
                let f = with_ext(&c, "shp");
                a.icon = Some(if !c.is_empty() && ctx.files.exists(&f) {
                    f
                } else {
                    PLACEHOLDER_ICON.to_vec()
                });
            }
            // spec: S-0007 claim 25 (the alternate icon, while none is loaded)
            if a.alt_icon.is_none()
                && let Some(c) = section_text(sec, "AltCameo", ICON_MAX).filter(|c| !c.is_empty())
            {
                a.alt_icon = ctx.exists_then(with_ext(&c, "shp"));
            }
        }
    }
    // spec: S-0007 Edge cases (switch off: the final image gets exactly one A on snow)
    if !ctx.q.arctic_suffix_on_flag_turning_on
        && snow
        && bool_as_of(t, "AlternateArcticArt", last, false)
        && !flags.voxel
    {
        let mut s = a.image.clone();
        s.push(b'A');
        a.image = text(&s, IMAGE_MAX).to_vec();
        a.notes.push(
            "arctic suffix applied once at the end (quirks.arctic_suffix_on_flag_turning_on off)"
                .into(),
        );
    }
    // spec: S-0007 claim 23 (the translucent sprite: no theater handling, no fallback)
    if let Some(n) = text_as_of(t, "AlphaImage", last, ICON_MAX).filter(|n| !n.is_empty()) {
        a.translucent = ctx.exists_then(with_ext(&n, "shp"));
    }
    // Theater refresh (Table 2).
    match kind {
        Kind::Overlays => {
            // spec: S-0007 claim 15 (the refresh gives the same names as the type read)
            if flags.theater {
                a.sprite = ctx.found(&with_ext(&a.image, &ext), true, false);
            } else if flags.new_theater {
                a.sprite = ctx.subst_found(&with_ext(&a.image, "shp"), true);
            }
        }
        Kind::TerrainObjects | Kind::Smudges if flags.theater => {
            // spec: S-0007 claim 13 (the refresh loads the type name with the theater extension)
            let base = if ctx.q.theater_art_uses_type_name {
                t.name.clone()
            } else {
                a.image.clone()
            };
            a.sprite = ctx.exists_then(with_ext(&base, &ext));
        }
        // spec: S-0007 claim 12 (image name and build-up name with the extension; no fallback)
        Kind::Buildings if flags.theater && ctx.q.theater_refresh_building_no_fallback => {
            a.sprite = ctx.exists_then(with_ext(&a.image, &ext));
            match buildup
                .as_ref()
                .and_then(|b| ctx.exists_then(with_ext(b, &ext)))
            {
                Some(f) => {
                    a.building_sprites.insert("buildup", f);
                }
                None => {
                    a.building_sprites.remove("buildup");
                }
            }
        }
        _ => {}
    }
    if mobile && flags.voxel {
        voxel_parts(ctx, kind, t, &mut a, last);
    } else if kind == Kind::Vehicles && a.sprite.is_some() && bool_as_of(t, "Turret", last, false) {
        // spec: S-0007 claim 19 (a sprite body with a voxel turret: each part on its own)
        for (role, suffix) in [("turret", "TUR"), ("barrel", "BARL")] {
            let mut base = a.image.clone();
            base.extend_from_slice(suffix.as_bytes());
            if voxel_ok(ctx, &base) == Some(true) {
                a.voxels.insert(role.into(), with_ext(&base, "vxl"));
            }
        }
    }
    if kind == Kind::Buildings {
        building_voxels(ctx, t, &mut a, last);
    }
    for (name, on) in [
        ("theater", flags.theater),
        ("new_theater", flags.new_theater),
        ("voxel", flags.voxel),
        ("arctic", bool_as_of(t, "AlternateArcticArt", last, false)),
    ] {
        if on {
            a.flags.push(name);
        }
    }
    a
}

/// `Some(true)` when the model `base.vxl` and its motion file `base.hva`
/// both resolve and are valid, `Some(false)` when the model resolves but
/// either is missing or invalid, `None` when the model does not resolve.
fn voxel_ok(ctx: &Ctx, base: &[u8]) -> Option<bool> {
    let model = with_ext(base, "vxl");
    if !ctx.files.exists(&model) {
        return None;
    }
    let motion = with_ext(base, "hva");
    // spec: S-0007 claim 18 (a found model loads its motion file; both must be valid)
    Some(ctx.files.valid(&model) && ctx.files.exists(&motion) && ctx.files.valid(&motion))
}

fn voxel_parts(ctx: &mut Ctx, kind: Kind, t: &TypeRec, a: &mut TypeArt, last: usize) {
    let image = a.image.clone();
    let part = |suffix: &str| {
        let mut b = image.clone();
        b.extend_from_slice(suffix.as_bytes());
        b
    };
    let mut discard = false;
    let mut parts: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut take = |role: String, base: Vec<u8>, required: bool, ctx: &Ctx, discard: &mut bool| {
        match voxel_ok(ctx, &base) {
            Some(true) => {
                parts.insert(role, with_ext(&base, "vxl"));
            }
            Some(false) => {
                // spec: S-0007 claim 18 (an invalid part or motion file discards the voxel art)
                if ctx.q.voxel_part_failure_discards_all || required {
                    *discard = true;
                }
            }
            None if required => *discard = true,
            None => {}
        }
    };
    // spec: S-0007 claims 16 and 18 (the body is required)
    take("body".into(), image.clone(), true, ctx, &mut discard);
    let vehicle = kind == Kind::Vehicles;
    let turret = bool_as_of(t, "Turret", last, false);
    let gattling = bool_as_of(t, "IsGattling", last, false);
    let mut n = int_as_of(t, "TurretCount", last, 0);
    if n > TURRET_COUNT_MAX {
        // spec: S-0007 claim 17 (our engine clamps the count to 18 and reports it)
        ctx.warnings.push(format!(
            "{}: turret count {n} above {TURRET_COUNT_MAX} is treated as {TURRET_COUNT_MAX} (S-0007 claim 17)",
            show(&t.name)
        ));
        n = TURRET_COUNT_MAX;
    }
    let numbered = vehicle && turret && n >= 1 && !gattling;
    if numbered {
        // spec: S-0007 claim 16 (turret k and barrel k; a missing numbered turret discards the art)
        for k in 0..n {
            let suffix = if k == 0 { String::new() } else { k.to_string() };
            let tur = part(&format!("TUR{suffix}"));
            let barl = part(&format!("BARL{suffix}"));
            let role_t = format!("turret{k}");
            match voxel_ok(ctx, &tur) {
                None if ctx.q.voxel_part_failure_discards_all => discard = true,
                _ => take(role_t, tur, false, ctx, &mut discard),
            }
            take(format!("barrel{k}"), barl, false, ctx, &mut discard);
        }
    } else if !vehicle || turret {
        // spec: S-0007 claim 16 (single turret and barrel)
        take("turret".into(), part("TUR"), false, ctx, &mut discard);
        take("barrel".into(), part("BARL"), false, ctx, &mut discard);
    }
    if vehicle && !turret {
        // spec: S-0007 claim 16 (alternate bodies: "WO" with the no-spawn-alternate flag, "W" for the type APC)
        if bool_as_of(t, "NoSpawnAlt", last, false) {
            take(
                "alternate_body".into(),
                part("WO"),
                false,
                ctx,
                &mut discard,
            );
        } else if !ctx.q.apc_water_body || t.name == b"APC" {
            take("alternate_body".into(), part("W"), false, ctx, &mut discard);
        }
    }
    if discard {
        a.notes.push(
            "voxel art discarded: a required or invalid part (S-0007 claim 18); nothing is drawn"
                .into(),
        );
        a.voxels.clear();
    } else {
        a.voxels = parts;
    }
}

/// Position of the first late "TUR" in `t` (S-0007 claim 20).
fn late_tur(t: &[u8], skip_four: bool) -> Option<usize> {
    let from = if skip_four { LATE_TUR_START - 1 } else { 0 };
    (from..t.len().saturating_sub(2)).find(|&i| t[i..i + 3].eq_ignore_ascii_case(b"TUR"))
}

fn building_voxels(ctx: &Ctx, t: &TypeRec, a: &mut TypeArt, last: usize) {
    let turret_voxel = bool_as_of(t, "TurretAnimIsVoxel", last, false);
    let barrel_voxel = bool_as_of(t, "BarrelAnimIsVoxel", last, false);
    if !turret_voxel && !barrel_voxel {
        return;
    }
    let anim = text_as_of(t, "TurretAnim", last, value::COMMON_TEXT_LIMIT).unwrap_or_default();
    // spec: S-0007 claim 20 (the table of turret and barrel models)
    let (turret, barrel) = match late_tur(&anim, ctx.q.building_tur_search_skips_four) {
        Some(p) => {
            let mut b = anim[..p].to_vec();
            b.extend_from_slice(b"BARL");
            (Some(anim.clone()), Some(b))
        }
        None if barrel_voxel => (
            None,
            text_as_of(t, "VoxelBarrelFile", last, value::COMMON_TEXT_LIMIT),
        ),
        None if ctx.q.building_turret_without_tur_in_barrel_role => (None, Some(anim.clone())),
        None => (Some(anim.clone()), None),
    };
    for (role, base) in [("turret", turret), ("barrel", barrel)] {
        if let Some(base) = base.filter(|b| !b.is_empty())
            && voxel_ok(ctx, &base) == Some(true)
        {
            a.voxels.insert(role.into(), with_ext(&base, "vxl"));
        }
    }
}

fn animation(ctx: &mut Ctx, t: &TypeRec) -> TypeArt {
    let mut a = TypeArt {
        image: t.name.clone(),
        ..TypeArt::default()
    };
    if t.reads.is_empty() {
        a.notes
            .push("no type read: no art is built (S-0007 Terms, type read)".into());
        return a;
    }
    let ext = ctx.ext();
    let snow = ctx.theater == Theater::Snow;
    let own = ctx.art.section(&t.name);
    // spec: S-0007 claim 9 (an animation's arctic flag comes from its own art section)
    let arctic = section_bool(own, "AlternateArcticArt").unwrap_or(false);
    let mut flags = Flags::default();
    let mut holds = false;
    let mut arctic_prev = false;
    for _ in &t.reads {
        // spec: S-0007 claim 1 (an animation's image key comes from its own art section)
        if let Some(i) = section_text(own, "Image", IMAGE_MAX) {
            a.image = i;
        }
        let sec = ctx.art.section(&a.image);
        // spec: S-0007 claim 2 (the flag table for animations)
        for (flag, key) in [
            (&mut flags.theater, "Theater"),
            (&mut flags.new_theater, "NewTheater"),
        ] {
            *flag = if holds {
                section_bool(sec, key).unwrap_or(*flag)
            } else {
                section_bool(own, key)
                    .or_else(|| section_bool(sec, key))
                    .unwrap_or(*flag)
            };
        }
        // spec: S-0007 claims 9 and 10 (the A only at reads after which the theater flag is set)
        if flags.theater {
            if ctx.q.arctic_suffix_on_flag_turning_on && snow && arctic && !arctic_prev {
                let mut s = a.image.clone();
                s.push(b'A');
                a.image = text(&s, IMAGE_MAX).to_vec();
            }
            arctic_prev = arctic;
        }
        // spec: S-0007 claim 14 (theater flag: image.E; else image.shp without substitution)
        a.sprite = if flags.theater {
            ctx.found(&with_ext(&a.image, &ext), true, false)
        } else {
            ctx.found(&with_ext(&a.image, "shp"), true, false)
        };
        holds = a.sprite.is_some();
    }
    // spec: S-0007 claim 14 (the theater refresh)
    if flags.theater {
        let base = if ctx.q.theater_art_uses_type_name {
            t.name.clone()
        } else {
            a.image.clone()
        };
        a.sprite = ctx
            .exists_then(with_ext(&base, &ext))
            .or_else(|| ctx.exists_then(with_ext(&base, "shp")));
    } else if flags.new_theater {
        a.sprite = ctx.subst_found(&with_ext(&a.image, "shp"), true);
    }
    // spec: S-0007 claim 23 (an animation's translucent sprite comes from its own art section)
    if let Some(n) = section_text(own, "AlphaImage", ICON_MAX).filter(|n| !n.is_empty()) {
        a.translucent = ctx.exists_then(with_ext(&n, "shp"));
    }
    for (name, on) in [
        ("theater", flags.theater),
        ("new_theater", flags.new_theater),
        ("arctic", arctic),
    ] {
        if on {
            a.flags.push(name);
        }
    }
    a
}

/// Notes the sprites that are not drawn in play: a deferred sprite (S-0005
/// claim 22) whose name resolves only while the game loads (claim 13,
/// `quirks.loading_archive_sprites_vanish`). `in_play` answers with the
/// loading archives released.
pub fn note_vanishing_sprites(
    arts: &mut BTreeMap<(Kind, usize), TypeArt>,
    in_play: &dyn Fn(&[u8]) -> bool,
    q: &ArtQuirks,
) {
    if !q.loading_archive_sprites_vanish {
        return;
    }
    for ((kind, _), a) in arts.iter_mut() {
        let Some(s) = &a.sprite else {
            continue;
        };
        // spec: S-0005 claim 22 (overlays, animations and names with ".shp" are deferred sprites)
        let deferred = matches!(kind, Kind::Overlays | Kind::Animations)
            || s.windows(4).any(|w| w.eq_ignore_ascii_case(b".shp"));
        if deferred && !in_play(s) {
            let note = format!(
                "{} resolves only while the game loads: not drawn in play (S-0005 claim 13)",
                show(s)
            );
            a.notes.push(note);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use glasnost_ini::Quirks;

    use super::*;
    use crate::layers::{self, GameKind, Layer, LayerQuirks, Role};

    struct Files(BTreeSet<Vec<u8>>);

    impl ArtFiles for Files {
        fn exists(&self, name: &[u8]) -> bool {
            self.0.contains(&name.to_ascii_uppercase())
        }
        fn valid(&self, name: &[u8]) -> bool {
            !name.to_ascii_uppercase().starts_with(b"BAD")
        }
    }

    fn files(names: &[&str]) -> Files {
        Files(
            names
                .iter()
                .map(|n| n.to_ascii_uppercase().into_bytes())
                .collect(),
        )
    }

    fn run(
        rules: &[(&str, Role)],
        art: &str,
        theater: Theater,
        f: &Files,
    ) -> (Outcome, BTreeMap<(Kind, usize), TypeArt>) {
        let doc = |t: &str| {
            glasnost_ini::Document::parse(t.as_bytes(), Quirks::default(), &mut Vec::new())
        };
        let layers_: Vec<Layer> = rules
            .iter()
            .map(|(t, role)| Layer {
                role: *role,
                file: "x".into(),
                doc: doc(t),
            })
            .collect();
        let art_doc = doc(art);
        let o = layers::apply(
            &layers_,
            &art_doc,
            "art",
            GameKind::Skirmish,
            &LayerQuirks::default(),
        );
        let (a, _) = lookup(&o, &art_doc, theater, f, &ArtQuirks::default());
        (o, a)
    }

    fn get<'a>(
        o: &Outcome,
        a: &'a BTreeMap<(Kind, usize), TypeArt>,
        kind: Kind,
        name: &str,
    ) -> &'a TypeArt {
        let i = o.registries[&kind]
            .iter()
            .position(|t| t.name == name.as_bytes())
            .unwrap();
        &a[&(kind, i)]
    }

    fn s(v: &Option<Vec<u8>>) -> Option<String> {
        v.as_ref().map(|v| String::from_utf8_lossy(v).into_owned())
    }

    /// spec: S-0007 claims 5 to 8 (scenarios A1 to A3)
    #[test]
    fn sprite_names_substitution_and_fallback() {
        let rules = "[InfantryTypes]\n1=GTank\n2=Ctx\n3=Plain\n4=Q\n[GTank]\nX=1\n[Ctx]\nX=1\n[Plain]\nX=1\n[Q]\nX=1\n";
        let art = "[GTank]\nNewTheater=yes\n[Ctx]\nNewTheater=yes\n[Plain]\nTheater=yes\n";
        let f = files(&["GDank.shp", "CGx.shp", "Plain.tem", "QGshp"]);
        let (o, a) = run(&[(rules, Role::Base)], art, Theater::Desert, &f);
        assert_eq!(
            s(&get(&o, &a, Kind::Infantry, "GTank").sprite).as_deref(),
            Some("GDank.shp")
        );
        // Substituted name missing: the generic fallback replaces the second character.
        assert_eq!(
            s(&get(&o, &a, Kind::Infantry, "Ctx").sprite).as_deref(),
            Some("CGx.shp")
        );
        let (o, a) = run(&[(rules, Role::Base)], art, Theater::Temperate, &f);
        assert_eq!(
            s(&get(&o, &a, Kind::Infantry, "Plain").sprite).as_deref(),
            Some("Plain.tem")
        );
        assert_eq!(
            s(&get(&o, &a, Kind::Infantry, "Q").sprite).as_deref(),
            Some("QGshp")
        );
    }

    /// spec: S-0007 claims 9 and 10 (scenarios A6, A7)
    #[test]
    fn arctic_suffix_rules() {
        let base = "[VehicleTypes]\n1=Sled\n[Sled]\nAlternateArcticArt=yes\n";
        let f = files(&["SledA.shp", "Sled.shp"]);
        let (o, a) = run(&[(base, Role::Base)], "", Theater::Snow, &f);
        assert_eq!(get(&o, &a, Kind::Vehicles, "Sled").image, b"SledA");
        // A later read with an image key loses the A while the flag stays set.
        let map = "[Sled]\nImage=Sled\n";
        let (o, a) = run(
            &[(base, Role::Base), (map, Role::Map)],
            "",
            Theater::Snow,
            &f,
        );
        assert_eq!(get(&o, &a, Kind::Vehicles, "Sled").image, b"Sled");
        // Set, cleared, set again: two As.
        let mode = "[Sled]\nAlternateArcticArt=no\n";
        let map = "[Sled]\nAlternateArcticArt=yes\n";
        let (o, a) = run(
            &[(base, Role::Base), (mode, Role::Mode), (map, Role::Map)],
            "",
            Theater::Snow,
            &f,
        );
        assert_eq!(get(&o, &a, Kind::Vehicles, "Sled").image, b"SledAA");
        let (o, a) = run(&[(base, Role::Base)], "", Theater::Temperate, &f);
        assert_eq!(get(&o, &a, Kind::Vehicles, "Sled").image, b"Sled");
    }

    /// spec: S-0007 claims 11 and 12 (buildings; scenarios A1, A2, A12)
    #[test]
    fn building_sprites() {
        let rules = "[BuildingTypes]\n1=GAWIDG\n2=NAGIZMO\n[GAWIDG]\nX=1\n[NAGIZMO]\nX=1\n";
        let art = "[GAWIDG]\nBuildup=GAWIDGMK\nBibShape=GAWIDGBB\n[NAGIZMO]\nTheater=yes\nBuildup=NAGIZMK\n";
        let f = files(&[
            "GDWIDG.shp",
            "GDWIDGMK.shp",
            "GDWIDGBB.shp",
            "NAGIZMO.des",
            "NAGIZMK.des",
            "NDGIZMO.des",
        ]);
        let (o, a) = run(&[(rules, Role::Base)], art, Theater::Desert, &f);
        let p = get(&o, &a, Kind::Buildings, "GAWIDG");
        assert_eq!(s(&p.sprite).as_deref(), Some("GDWIDG.shp"));
        assert_eq!(
            p.building_sprites
                .get("buildup")
                .map(|v| String::from_utf8_lossy(v).into_owned())
                .as_deref(),
            Some("GDWIDGMK.shp")
        );
        assert!(p.building_sprites.contains_key("bib"));
        // Theater flag: the refresh loads image.E and buildup.E, without substitution.
        let n = get(&o, &a, Kind::Buildings, "NAGIZMO");
        assert_eq!(s(&n.sprite).as_deref(), Some("NAGIZMO.des"));
        assert!(n.icon.as_deref() == Some(PLACEHOLDER_ICON));
    }

    /// spec: S-0007 claim 11 (the eight other building sprites at a later
    /// type read; scenario A19)
    #[test]
    fn building_secondary_sprites_at_a_later_read() {
        let base = "[BuildingTypes]\n1=ZW\n[ZW]\nX=1\n";
        let map = "[ZW]\nImage=ZB\n";
        let f = files(&["ZW.shp", "ZB.shp", "ZWBIB.shp", "ZBBIB.shp", "ZWRUB.shp"]);
        let bib = |art: &str| {
            let (o, a) = run(
                &[(base, Role::Base), (map, Role::Map)],
                art,
                Theater::Temperate,
                &f,
            );
            let p = get(&o, &a, Kind::Buildings, "ZW");
            assert_eq!(s(&p.sprite).as_deref(), Some("ZB.shp"));
            (
                p.building_sprites
                    .get("bib")
                    .map(|v| String::from_utf8_lossy(v).into_owned()),
                p.building_sprites
                    .get("rubble")
                    .map(|v| String::from_utf8_lossy(v).into_owned()),
                p.notes.clone(),
            )
        };
        let first = "[ZW]\nBibShape=ZWBIB\nRubble=ZWRUB\n";
        // Not set at the later read: the earlier read's sprite stays.
        let (b, r, notes) = bib(&format!("{first}[ZB]\nX=1\n"));
        assert_eq!(b.as_deref(), Some("ZWBIB.shp"));
        assert_eq!(r.as_deref(), Some("ZWRUB.shp"));
        assert!(notes.is_empty(), "{notes:?}");
        // Set empty: as absent.
        let (b, _, _) = bib(&format!("{first}[ZB]\nBibShape=\nX=1\n"));
        assert_eq!(b.as_deref(), Some("ZWBIB.shp"));
        // Set to a name that resolves: replaced.
        let (b, r, _) = bib(&format!("{first}[ZB]\nBibShape=ZBBIB\n"));
        assert_eq!(b.as_deref(), Some("ZBBIB.shp"));
        assert_eq!(r.as_deref(), Some("ZWRUB.shp"));
        // Set to a name that neither resolves nor falls back: none.
        let (b, _, _) = bib(&format!("{first}[ZB]\nBibShape=ZNOPE\n"));
        assert_eq!(b, None);
        // No earlier read gave one and the later read does not set it: none.
        let (b, _, _) = bib("[ZB]\nX=1\n");
        assert_eq!(b, None);
    }

    /// spec: S-0007 claim 13 (terrain objects and smudges use the type name at the refresh)
    #[test]
    fn terrain_and_smudges() {
        let rules = "[TerrainTypes]\n1=Tree\n[Tree]\nImage=Bush\n[SmudgeTypes]\n1=Crater\n[Crater]\nImage=Hole\n";
        let art = "[Bush]\nTheater=yes\n[Hole]\nTheater=yes\n";
        let f = files(&["Tree.urb", "Hole.urb"]);
        let (o, a) = run(&[(rules, Role::Base)], art, Theater::Urban, &f);
        assert_eq!(
            s(&get(&o, &a, Kind::TerrainObjects, "Tree").sprite).as_deref(),
            Some("Tree.urb")
        );
        // The type read found Hole.urb, but the refresh needs Crater.urb, which is missing.
        assert_eq!(get(&o, &a, Kind::Smudges, "Crater").sprite, None);
    }

    /// spec: S-0007 claims 2 and 14 (animations: flags, refresh by type name)
    #[test]
    fn animations() {
        let rules = "[Animations]\n1=Fire\n2=Lamp\n";
        let art = "[Fire]\nTheater=yes\n[Lamp]\nNewTheater=yes\n";
        let f = files(&["Fire.tem", "Fire.shp", "LTmp.shp", "Lamp.shp"]);
        let (o, a) = run(
            &[(rules, Role::Base), ("", Role::Mode), ("", Role::Map)],
            art,
            Theater::Temperate,
            &f,
        );
        assert_eq!(
            s(&get(&o, &a, Kind::Animations, "Fire").sprite).as_deref(),
            Some("Fire.tem")
        );
        // New-theater: the refresh substitutes, but only names starting with G, N, C or Y change.
        assert_eq!(
            s(&get(&o, &a, Kind::Animations, "Lamp").sprite).as_deref(),
            Some("Lamp.shp")
        );
    }

    /// spec: S-0007 claims 16 to 18 (voxel parts)
    #[test]
    fn voxel_parts() {
        let rules = "[VehicleTypes]\n1=Tank\n2=Multi\n3=Broken\n[Tank]\nTurret=yes\n[Multi]\nTurret=yes\nTurretCount=2\n[Broken]\nX=1\n";
        let art = "[Tank]\nVoxel=yes\n[Multi]\nVoxel=yes\n[Broken]\nVoxel=yes\nImage=Ignored\n";
        let f = files(&[
            "Tank.vxl",
            "Tank.hva",
            "TankTUR.vxl",
            "TankTUR.hva",
            "Multi.vxl",
            "Multi.hva",
            "MultiTUR.vxl",
            "MultiTUR.hva",
            "Broken.vxl",
        ]);
        let (o, a) = run(&[(rules, Role::Base)], art, Theater::Temperate, &f);
        let tank = get(&o, &a, Kind::Vehicles, "Tank");
        assert_eq!(tank.voxels.keys().collect::<Vec<_>>(), ["body", "turret"]);
        // Turret 1 of 2 is missing: the voxel art is discarded.
        assert!(get(&o, &a, Kind::Vehicles, "Multi").voxels.is_empty());
        // A model without its motion file discards the art.
        assert!(get(&o, &a, Kind::Vehicles, "Broken").voxels.is_empty());
    }

    /// spec: S-0007 claim 20 (building voxel turrets)
    #[test]
    fn building_voxel_turrets() {
        assert_eq!(late_tur(b"GTFOOBARTUR", true), Some(8));
        assert_eq!(late_tur(b"TURX", true), None);
        assert_eq!(late_tur(b"ATUR", true), None);
        assert_eq!(late_tur(b"ATUR", false), Some(1));
        let rules = "[BuildingTypes]\n1=Gun\n2=Pillbox\n[Gun]\nTurretAnimIsVoxel=yes\nTurretAnim=GUNHEADTUR\n\
                     [Pillbox]\nTurretAnimIsVoxel=yes\nTurretAnim=PILLGUN\n";
        let f = files(&[
            "GUNHEADTUR.vxl",
            "GUNHEADTUR.hva",
            "GUNHEADBARL.vxl",
            "GUNHEADBARL.hva",
            "PILLGUN.vxl",
            "PILLGUN.hva",
        ]);
        let (o, a) = run(&[(rules, Role::Base)], "", Theater::Temperate, &f);
        assert_eq!(get(&o, &a, Kind::Buildings, "Gun").voxels.len(), 2);
        let p = get(&o, &a, Kind::Buildings, "Pillbox");
        assert_eq!(p.voxels.keys().collect::<Vec<_>>(), ["barrel"]);
    }

    /// spec: S-0007 claims 22 and 25 (icons)
    #[test]
    fn icons() {
        let base = "[InfantryTypes]\n1=Guy\n[Guy]\nX=1\n";
        let map = "[Guy]\nImage=Other\n";
        let art = "[Guy]\nCameo=GUYICON\n[Other]\nCameo=OTHERICON\nAltCameo=ALTICON\n";
        let f = files(&["GUYICON.shp", "OTHERICON.shp", "ALTICON.shp"]);
        let (o, a) = run(
            &[(base, Role::Base), (map, Role::Map)],
            art,
            Theater::Temperate,
            &f,
        );
        let g = get(&o, &a, Kind::Infantry, "Guy");
        // The main icon is frozen at the first read; the alternate icon arrives later.
        assert_eq!(s(&g.icon).as_deref(), Some("GUYICON.shp"));
        assert_eq!(s(&g.alt_icon).as_deref(), Some("ALTICON.shp"));
    }
}
