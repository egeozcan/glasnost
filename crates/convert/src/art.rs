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
    /// bytes of text. Off: read whole.
    pub sequence_value_length_limit: bool,
    /// `quirks.case_sensitive_direction` (S-0107 claim 22): a declared
    /// direction is recognised only in exact upper case. Off: any case.
    pub case_sensitive_direction: bool,
    /// `quirks.projectile_image_afresh` (S-0007 claim 26): a projectile's
    /// image name is its type read's image key or empty. Off: a later type
    /// read without an image key keeps the earlier read's image name.
    pub projectile_image_afresh: bool,
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
            // spec: S-0007 claim 26, Edge cases (quirks.projectile_image_afresh: the original's)
            projectile_image_afresh: true,
        }
    }
}

impl ArtQuirks {
    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); 13] {
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
            ("projectile_image_afresh", self.projectile_image_afresh),
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
    /// An animation's sprite of its last sprite-found moment: a type read
    /// that found a sprite, or the new-theater refresh when it found one
    /// (S-0109 claim 25); none when no moment found one.
    pub middle_sprite: Option<Vec<u8>>,
    /// The image-frame count of [`TypeArt::middle_sprite`], read from the
    /// file while the loading archives are mounted (S-0109 claim 25); none
    /// when there is no such sprite or the file does not parse.
    pub middle_sprite_frames: Option<i32>,
    /// The image-frame count of [`TypeArt::sprite`], the sprite seen in play
    /// (S-0513 claim 46: the frame count of the animation's current image);
    /// none when there is no sprite or the file does not parse.
    pub sprite_frames: Option<i32>,
    /// The canvas height of [`TypeArt::sprite`]'s header (S-0513 F4).
    pub sprite_height: Option<i32>,
    /// The section count of a voxel animation's own model (S-0317 claim
    /// 23), read from the file while the loading archives are mounted; none
    /// when there is no model or the file does not parse.
    pub voxel_sections: Option<usize>,
    /// A projectile type's drawing fields (S-0306 claims 28 to 32); none
    /// for the other kinds.
    pub projectile: Option<ProjectileLook>,
    /// Notes on open or unusual cases.
    pub notes: Vec<String>,
}

/// The palette a voxel projectile's colour definition selects (S-0306
/// claim 30, Import mapping `Color`), by position in the import's colour
/// definitions (S-0103 claim 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectileColour {
    /// The single-shade palette of that definition: before any type read
    /// has selected one, the first definition's.
    Single(usize),
    /// The 53-shade palette of that definition.
    Shades(usize),
}

impl Default for ProjectileColour {
    fn default() -> Self {
        // spec: S-0306 claim 30 (before any read selects one: the first definition's single-shade palette)
        ProjectileColour::Single(0)
    }
}

/// A projectile type's drawing fields after its type reads (S-0306 Terms
/// *drawing fields*, Import mapping): presentation only.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectileLook {
    /// The art section's rotating flag (claim 29).
    pub rotates: bool,
    /// The low animation field *a*_lo, 0 to 255 (claim 29, F15).
    pub anim_low: u8,
    /// The high animation field *a*_hi, 0 to 255 (claim 29, F15).
    pub anim_high: u8,
    /// The animation rate *R*ₐ, 0 to 255 (claim 29, F15).
    pub anim_rate: u8,
    /// The art section's anim-palette flag (claim 31).
    pub anim_palette: bool,
    /// The art section's flat flag: the surface layer (S-0102 claim 3).
    pub flat: bool,
    /// The voxel flag: drawn as a voxel model (claims 28 and 30).
    pub voxel: bool,
    /// The rules invisible flag: never drawn (claim 28).
    pub inviso: bool,
    /// The rules shadow flag (claim 32).
    pub shadow: bool,
    /// The rules firer's-palette flag (claim 31).
    pub firers_palette: bool,
    /// The palette of a voxel body (claim 30).
    pub colour: ProjectileColour,
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
pub const KINDS: [Kind; 9] = [
    Kind::Infantry,
    Kind::Vehicles,
    Kind::Aircraft,
    Kind::Buildings,
    Kind::Animations,
    Kind::Overlays,
    Kind::TerrainObjects,
    Kind::Smudges,
    Kind::Particles,
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
    // spec: S-0007 claims 26 and 27 (projectiles follow their own image rule, not claim 1)
    for (i, t) in outcome
        .registries
        .get(&Kind::Projectiles)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let a = projectile(&mut ctx, t, &outcome.colours);
        out.insert((Kind::Projectiles, i), a);
    }
    // spec: S-0007 claim 30 (a voxel animation's model)
    for (i, t) in outcome
        .registries
        .get(&Kind::VoxelAnimations)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let a = voxel_animation(&mut ctx, t);
        out.insert((Kind::VoxelAnimations, i), a);
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
    /// The kind reads the voxel flag: infantry, vehicles, aircraft,
    /// particles and voxel animations.
    reads_voxel: bool,
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
            // spec: S-0007 claims 4, 28 and 30 (the kinds whose art section's voxel flag counts)
            reads_voxel: matches!(
                kind,
                Kind::Infantry
                    | Kind::Vehicles
                    | Kind::Aircraft
                    | Kind::Particles
                    | Kind::VoxelAnimations
            ),
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
        if self.reads_voxel {
            // spec: S-0007 claims 4, 28 and 30 (the voxel flag, for infantry, vehicles, aircraft, particles and voxel animations)
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
            // spec: S-0007 claim 28 and Table 2 (a particle's sprite as the infantry sprite; none with the voxel flag)
            Kind::Infantry | Kind::Vehicles | Kind::Aircraft | Kind::Overlays | Kind::Particles => {
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

/// A held art field of a projectile's drawing (S-0306 Import mapping): the
/// selected section's value replaces it, a missing or empty one keeps it.
fn held<T>(sec: Option<&Section>, key: &str, read: impl Fn(&[u8]) -> Option<T>, v: &mut T) {
    if let Some(x) = sec
        .and_then(|s| s.value(key.as_bytes()))
        .filter(|x| !x.is_empty())
        .and_then(read)
    {
        *v = x;
    }
}

/// A projectile type's art and drawing fields (S-0007 claims 26 and 27,
/// S-0306 Import mapping).
fn projectile(ctx: &mut Ctx, t: &TypeRec, colours: &[crate::layers::ColourDef]) -> TypeArt {
    let mut a = TypeArt::default();
    let mut look = ProjectileLook {
        // spec: S-0306 Import mapping (rules Shadow: absent yes)
        shadow: true,
        ..ProjectileLook::default()
    };
    if t.reads.is_empty() {
        a.notes
            .push("no type read: no art is built (S-0007 Terms, type read)".into());
        a.projectile = Some(look);
        return a;
    }
    let snow = ctx.theater == Theater::Snow;
    let ext = ctx.ext();
    let mut flags = Flags::default();
    let mut arctic_prev = false;
    let mut arctic_now = false;
    let mut given_before = false;
    // The colour definition in force and whether its 53-shade palette is selected.
    let (mut def, mut shades) = (0usize, false);
    for &r in &t.reads {
        let key = given_at(t, "Image", r);
        if key.is_none() && given_before {
            // spec: S-0007 claim 26 (the importer warns when a later section drops the image key an earlier layer gave)
            ctx.warnings.push(format!(
                "projectile {}: a later type read has no image key while an earlier one gave it; {} (S-0007 claim 26)",
                show(&t.name),
                if ctx.q.projectile_image_afresh {
                    "the projectile has no art from that read on"
                } else {
                    "the earlier image name is kept (quirks.projectile_image_afresh off)"
                }
            ));
        }
        given_before |= key.is_some_and(|k| !text(k, IMAGE_MAX).is_empty());
        a.image = match key {
            // spec: S-0007 claim 26 (the read's image key, cut to 24 bytes)
            Some(k) => text(k, IMAGE_MAX).to_vec(),
            // spec: S-0007 claim 26 (no image key: empty, nothing carried over; quirk off: kept)
            None if ctx.q.projectile_image_afresh => Vec::new(),
            None => a.image.clone(),
        };
        // spec: S-0306 Import mapping (Color: missing or empty re-selects the 53-shade palette of
        // the definition in force; a known name, compared without regard to case, selects that
        // definition; an unknown name keeps the palette in force)
        match given_at(t, "Color", r).map(|v| text(v, value::COMMON_TEXT_LIMIT)) {
            None | Some([]) => shades = true,
            Some(n) => {
                if let Some(i) = colours.iter().position(|c| c.name.eq_ignore_ascii_case(n)) {
                    (def, shades) = (i, true);
                }
            }
        }
        let arctic = bool_as_of(t, "AlternateArcticArt", r, false);
        // spec: S-0007 claims 10 and 27 (the A of a voxel model where the flag turns on, until the next read)
        arctic_now = arctic && !arctic_prev;
        arctic_prev = arctic;
        if a.image.is_empty() {
            // spec: S-0306 Import mapping (nothing is read when the current rules image is empty)
            continue;
        }
        // spec: S-0007 claims 2, 9 and 27 (the flags of the section named by I, before any suffix)
        let sec = ctx.art.section(&a.image);
        flags.theater = section_bool(sec, "Theater").unwrap_or(flags.theater);
        flags.new_theater = section_bool(sec, "NewTheater").unwrap_or(flags.new_theater);
        flags.voxel = section_bool(sec, "Voxel").unwrap_or(flags.voxel);
        // spec: S-0306 Import mapping (Rotates, AnimPalette: missing or empty keeps the value in force)
        held(sec, "Rotates", boolean, &mut look.rotates);
        held(sec, "AnimPalette", boolean, &mut look.anim_palette);
        // spec: S-0102 Import mapping (art, projectile image, Flat)
        held(sec, "Flat", boolean, &mut look.flat);
        for (k, v) in [
            ("AnimLow", &mut look.anim_low),
            ("AnimHigh", &mut look.anim_high),
            ("AnimRate", &mut look.anim_rate),
        ] {
            let mut n: Option<i32> = None;
            held(sec, k, |x| value::integer(x).map(Some), &mut n);
            if let Some(n) = n {
                if !(0..=255).contains(&n) {
                    // spec: S-0306 Edge cases (fields outside 0 to 255: kept modulo 256, the importer warns)
                    ctx.warnings.push(format!(
                        "projectile image {} {k} {n} is outside 0 to 255: it acts as {} (S-0306 Edge cases)",
                        show(&a.image),
                        n.rem_euclid(256)
                    ));
                }
                // spec: S-0306 Import mapping (kept as its value modulo 256)
                *v = n.rem_euclid(256) as u8;
            }
        }
    }
    let last = *t.reads.last().unwrap_or(&0);
    look.voxel = flags.voxel;
    look.colour = if shades {
        ProjectileColour::Shades(def)
    } else {
        ProjectileColour::Single(def)
    };
    // spec: S-0306 Import mapping (rules Inviso, Shadow default yes, FirersPalette)
    look.inviso = bool_as_of(t, "Inviso", last, false);
    look.shadow = bool_as_of(t, "Shadow", last, true);
    look.firers_palette = bool_as_of(t, "FirersPalette", last, false);
    // spec: S-0007 claim 23 (the translucent sprite: no theater handling, no fallback)
    if let Some(n) = text_as_of(t, "AlphaImage", last, ICON_MAX).filter(|n| !n.is_empty()) {
        a.translucent = ctx.exists_then(with_ext(&n, "shp"));
    }
    if look.inviso {
        // spec: S-0007 claim 27 (an invisible projectile is never drawn: its art need not be looked up)
        a.notes
            .push("invisible projectile: no art looked up (S-0007 claim 27)".into());
    } else if a.image.is_empty() {
        // spec: S-0007 claim 26 (an empty image name: no sprite and no model; the empty-base lookups are suppressed)
        a.notes
            .push("empty image name: no body art (S-0007 claim 26)".into());
    } else if flags.voxel {
        let plain = a.image.clone();
        let suffix = if ctx.q.arctic_suffix_on_flag_turning_on {
            // spec: S-0007 claim 27 (on snow the model is named I ‖ A at a read where claim 10 appends one)
            snow && arctic_now
        } else {
            // spec: S-0007 Edge cases (switch off: the final name gets exactly one A on snow)
            snow && arctic_prev
        };
        if suffix {
            let mut s = plain.clone();
            s.push(b'A');
            a.image = s;
        }
        // spec: S-0007 claims 16, 18 and 27 (the parts of a type that is not a vehicle; claim 18 discards)
        voxel_parts(ctx, Kind::Projectiles, t, &mut a, last);
        a.image = plain;
    } else {
        // spec: S-0007 claim 27 (theater extension; else letter substitution with the new-theater
        // flag; else I.shp; each with the generic fallback once; never an arctic A)
        a.sprite = if flags.theater {
            ctx.found(&with_ext(&a.image, &ext), true, false)
        } else {
            ctx.subst_found(&with_ext(&a.image, "shp"), flags.new_theater)
        };
    }
    for (name, on) in [
        ("theater", flags.theater),
        ("new_theater", flags.new_theater),
        ("voxel", flags.voxel),
        ("arctic", arctic_prev),
    ] {
        if on {
            a.flags.push(name);
        }
    }
    a.projectile = Some(look);
    a
}

/// The share flags of a voxel animation (S-0007 claim 30), in their
/// priority order.
// spec: S-0007 Import mapping (ShareBodyData, ShareTurretData, ShareBarrelData)
const SHARE_FLAGS: [&str; 3] = ["ShareBodyData", "ShareTurretData", "ShareBarrelData"];

/// A voxel animation's model (S-0007 claim 30): its image name walked as
/// claim 1 says, with the arctic suffix only when its art section does not
/// set the voxel flag, and its own model I ‖ .vxl as role `body`, which
/// loads no motion file. A type read that sets a share flag shows a part
/// of its share source instead: that is not built here (a seam), and such
/// a type has no model.
fn voxel_animation(ctx: &mut Ctx, t: &TypeRec) -> TypeArt {
    let mut a = TypeArt {
        image: t.name.clone(),
        ..TypeArt::default()
    };
    if t.reads.is_empty() {
        a.notes
            .push("no type read: no art is built (S-0007 Terms, type read)".into());
        return a;
    }
    let snow = ctx.theater == Theater::Snow;
    let mut walk = ImageWalk::new(t, Kind::VoxelAnimations, snow, &ctx.q);
    for &r in &t.reads {
        // spec: S-0007 claims 1, 9, 10 and 30 (the image name of each read; the A only without the voxel flag)
        walk.step(ctx.art, t, r);
    }
    a.image.clone_from(&walk.image);
    let last = *t.reads.last().unwrap_or(&0);
    // spec: S-0007 claim 30 (the share flags are read afresh at every type read: the last read decides)
    let shared = SHARE_FLAGS
        .iter()
        .any(|k| given_at(t, k, last).and_then(boolean).unwrap_or(false));
    if shared {
        // Seam: the shared part of S-0007 claim 30 (a part of the share source with its motion file) is not built.
        a.notes.push(
            "shares a part of another type (S-0007 claim 30): not built here, nothing is drawn"
                .into(),
        );
        ctx.warnings.push(format!(
            "voxel animation {}: its section sets a share flag; the shared part is not built yet, so it draws nothing (S-0007 claim 30)",
            show(&t.name)
        ));
        return a;
    }
    // spec: S-0007 claim 30 (the own model I ‖ .vxl, no motion file, no other parts, no theater handling, no fallback)
    let model = with_ext(&a.image, "vxl");
    if ctx.files.exists(&model) && ctx.files.valid(&model) {
        a.voxels.insert("body".into(), model);
    } else {
        // spec: S-0007 claim 30, Departures (an unresolved or invalid model: our engine draws nothing and the importer warns)
        a.notes.push(
            "the model does not resolve or is invalid: nothing is drawn (S-0007 claim 30)".into(),
        );
        ctx.warnings.push(format!(
            "voxel animation {}: the model {} does not resolve or is invalid; it draws nothing (S-0007 claim 30)",
            show(&t.name),
            show(&model)
        ));
    }
    if walk.flags.voxel {
        a.flags.push("voxel");
    }
    a
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
        if holds {
            // spec: S-0109 claim 25 (a type read that finds a sprite is a sprite-found moment)
            a.middle_sprite.clone_from(&a.sprite);
        }
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
        if a.sprite.is_some() {
            // spec: S-0109 claim 25 (the new-theater refresh that finds a sprite is a sprite-found
            // moment; the theater flag's refresh is not)
            a.middle_sprite.clone_from(&a.sprite);
        }
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
    /// spec: S-0007 claim 30 (a voxel animation's own model I ‖ .vxl with no
    /// motion file; the arctic suffix only without the voxel flag; an
    /// unresolved model draws nothing; a share flag is our seam)
    #[test]
    fn voxel_animation_models() {
        let rules = "[VoxelAnims]\n1=Chunk\n2=Cold\n3=Iced\n4=Gone\n5=Borrow\n\
                     [Chunk]\nImage=Bit\n[Cold]\nAlternateArcticArt=yes\n\
                     [Iced]\nAlternateArcticArt=yes\n[Gone]\nX=1\n[Borrow]\nShareBodyData=yes\n";
        let art = "[Iced]\nVoxel=yes\n";
        let f = files(&["Bit.vxl", "ColdA.vxl", "Cold.vxl", "Iced.vxl", "Borrow.vxl"]);
        let (o, a) = run(&[(rules, Role::Base)], art, Theater::Snow, &f);
        let body = |n: &str| {
            s(&get(&o, &a, Kind::VoxelAnimations, n)
                .voxels
                .get("body")
                .cloned())
        };
        // The image key names the model; no motion file is needed.
        assert_eq!(body("Chunk").as_deref(), Some("Bit.vxl"));
        // On snow, the A without the voxel flag; never with it.
        assert_eq!(body("Cold").as_deref(), Some("ColdA.vxl"));
        assert_eq!(body("Iced").as_deref(), Some("Iced.vxl"));
        // No model file: no model.
        assert_eq!(body("Gone"), None);
        // A share flag: no own model (the shared part is a seam).
        assert_eq!(body("Borrow"), None);
    }

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
        // spec: S-0007 claims 1 and 10 (only a non-empty later image key replaces; an empty one keeps the suffixed name)
        let map = "[Sled]\nImage=\n";
        let (o, a) = run(
            &[(base, Role::Base), (map, Role::Map)],
            "",
            Theater::Snow,
            &f,
        );
        assert_eq!(get(&o, &a, Kind::Vehicles, "Sled").image, b"SledA");
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

    /// spec: S-0108 Import mapping (`DoorAnim`, `DeployingAnim`, `RoofDeployingAnim`,
    /// `UnderDoorAnim`, `UnderRoofDoorAnim`: the door and unloading sprites, absent none);
    /// S-0007 claim 11 (each name ‖ .shp)
    #[test]
    fn building_unloading_and_door_sprites() {
        let rules = "[BuildingTypes]\n1=ZF\n[ZF]\nX=1\n";
        let art = "[ZF]\nDeployingAnim=ZFDEP\nRoofDeployingAnim=ZFROOF\nUnderDoorAnim=ZFUD\n\
                   UnderRoofDoorAnim=ZFURD\nDoorAnim=ZFDOOR\n";
        let f = files(&[
            "ZF.shp",
            "ZFDEP.shp",
            "ZFROOF.shp",
            "ZFUD.shp",
            "ZFURD.shp",
            "ZFDOOR.shp",
        ]);
        let (o, a) = run(&[(rules, Role::Base)], art, Theater::Temperate, &f);
        let p = get(&o, &a, Kind::Buildings, "ZF");
        let role = |r: &str| {
            p.building_sprites
                .get(r)
                .map(|v| String::from_utf8_lossy(v).into_owned())
        };
        assert_eq!(role("deploying").as_deref(), Some("ZFDEP.shp"));
        assert_eq!(role("roof_deploying").as_deref(), Some("ZFROOF.shp"));
        assert_eq!(role("under_door").as_deref(), Some("ZFUD.shp"));
        assert_eq!(role("under_roof_door").as_deref(), Some("ZFURD.shp"));
        assert_eq!(role("door").as_deref(), Some("ZFDOOR.shp"));
        // Absent keys: no sprite.
        let (o, a) = run(
            &[(rules, Role::Base)],
            "[ZF]\nX=1\n",
            Theater::Temperate,
            &f,
        );
        let p = get(&o, &a, Kind::Buildings, "ZF");
        assert!(p.building_sprites.is_empty(), "{:?}", p.building_sprites);
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

    /// spec: S-0007 claim 28 (a particle's image name from its rules
    /// section, its sprite as the infantry sprite: theater extension, letter
    /// substitution with the new-theater flag, generic fallback, the arctic
    /// suffix; no sprite with the voxel flag)
    #[test]
    fn particles() {
        let rules = "[Particles]\n1=Puff\n2=Gtx\n3=Lamp\n4=Vox\n5=Cold\n6=Bare\n\
                     [Puff]\nImage=Cloud\n[Gtx]\nX=1\n[Lamp]\nX=1\n[Vox]\nX=1\n\
                     [Cold]\nAlternateArcticArt=yes\n[Bare]\nX=1\n";
        let art = "[Gtx]\nNewTheater=yes\n[Lamp]\nTheater=yes\n[Vox]\nVoxel=yes\n";
        let f = files(&[
            "Cloud.shp",
            "GDx.shp",
            "Lamp.sno",
            "Vox.shp",
            "ColdA.shp",
            "BGre.shp",
        ]);
        let (o, a) = run(&[(rules, Role::Base)], art, Theater::Desert, &f);
        let sprite = |o: &Outcome, a: &BTreeMap<(Kind, usize), TypeArt>, n: &str| {
            s(&get(o, a, Kind::Particles, n).sprite)
        };
        // The rules image key names the sprite.
        assert_eq!(sprite(&o, &a, "Puff").as_deref(), Some("Cloud.shp"));
        assert_eq!(get(&o, &a, Kind::Particles, "Puff").image, b"Cloud");
        // New-theater flag: letter substitution.
        assert_eq!(sprite(&o, &a, "Gtx").as_deref(), Some("GDx.shp"));
        // Voxel flag: no sprite, even though the file exists.
        assert_eq!(sprite(&o, &a, "Vox"), None);
        // Generic fallback: Bare.shp is missing, the second character becomes G.
        assert_eq!(sprite(&o, &a, "Bare").as_deref(), Some("BGre.shp"));
        let (o, a) = run(&[(rules, Role::Base)], art, Theater::Snow, &f);
        // Theater flag: the theater extension.
        assert_eq!(sprite(&o, &a, "Lamp").as_deref(), Some("Lamp.sno"));
        // Arctic suffix on snow.
        assert_eq!(sprite(&o, &a, "Cold").as_deref(), Some("ColdA.shp"));
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

    fn run_q(
        rules: &[(&str, Role)],
        art: &str,
        theater: Theater,
        f: &Files,
        q: &ArtQuirks,
    ) -> (Outcome, BTreeMap<(Kind, usize), TypeArt>, Vec<String>) {
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
        let (a, w) = lookup(&o, &art_doc, theater, f, q);
        (o, a, w)
    }

    /// A vehicle whose weapon names projectile `p` (S-0004 claim 6: projectiles by reference).
    const SHOOTER: &str = "[VehicleTypes]\n1=Tank\n[Tank]\nPrimary=Gun\n[Gun]\nProjectile=Shell\nWarhead=Hit\n[Hit]\nVerses=1\n";

    /// spec: S-0007 claims 26 and 27 (scenario A20 a, b: the projectile image afresh, empty without a key, no default of the type name; the sprite from I with the generic fallback)
    #[test]
    fn projectile_image_is_taken_afresh() {
        let base = format!("{SHOOTER}[Shell]\nImage=Ball\n");
        let art = "[Shell]\nRotates=yes\n[Ball]\nRotates=yes\n[Other]\nX=1\n";
        let f = files(&["Ball.shp", "Shell.shp", "OGher.shp"]);
        let (o, a, w) = run_q(
            &[(&base, Role::Base)],
            art,
            Theater::Temperate,
            &f,
            &ArtQuirks::default(),
        );
        let shell = get(&o, &a, Kind::Projectiles, "Shell");
        assert_eq!(s(&shell.sprite).as_deref(), Some("Ball.shp"));
        assert!(shell.projectile.as_ref().unwrap().rotates);
        assert!(w.is_empty());
        // A map override section without an image key: no art for the rest of the game, warned.
        let map = "[Shell]\nArm=1\n";
        let (o, a, w) = run_q(
            &[(&base, Role::Base), (map, Role::Map)],
            art,
            Theater::Temperate,
            &f,
            &ArtQuirks::default(),
        );
        let shell = get(&o, &a, Kind::Projectiles, "Shell");
        assert!(shell.image.is_empty());
        assert_eq!(shell.sprite, None);
        assert!(w.iter().any(|m| m.contains("S-0007 claim 26")), "{w:?}");
        // quirks.projectile_image_afresh off: the earlier image name is kept, still warned.
        let off = ArtQuirks {
            projectile_image_afresh: false,
            ..ArtQuirks::default()
        };
        let (o, a, w) = run_q(
            &[(&base, Role::Base), (map, Role::Map)],
            art,
            Theater::Temperate,
            &f,
            &off,
        );
        let shell = get(&o, &a, Kind::Projectiles, "Shell");
        assert_eq!(shell.image, b"Ball");
        assert_eq!(s(&shell.sprite).as_deref(), Some("Ball.shp"));
        assert!(
            w.iter()
                .any(|m| m.contains("quirks.projectile_image_afresh off"))
        );
        // No image key at all: no default of the type name, even though Shell.shp exists.
        let bare = format!("{SHOOTER}[Shell]\nArm=1\n");
        let (o, a, _) = run_q(&[(&bare, Role::Base)], art, Theater::Temperate, &f, &off);
        assert_eq!(get(&o, &a, Kind::Projectiles, "Shell").sprite, None);
        // The generic fallback replaces the second character once.
        let other = format!("{SHOOTER}[Shell]\nImage=Other\n");
        let (o, a, _) = run_q(&[(&other, Role::Base)], art, Theater::Temperate, &f, &off);
        assert_eq!(
            s(&get(&o, &a, Kind::Projectiles, "Shell").sprite).as_deref(),
            Some("OGher.shp")
        );
    }

    /// spec: S-0007 claim 27 (scenario A20 c to f: a voxel projectile's parts, its discard, the arctic model only at the turning-on read, an invisible one not looked up; a sprite never takes the A)
    #[test]
    fn projectile_voxel_and_arctic_art() {
        let base = format!("{SHOOTER}[Shell]\nImage=Rock\nAlternateArcticArt=yes\n");
        let art = "[Rock]\nVoxel=yes\n[Ball]\nX=1\n";
        let f = files(&[
            "Rock.vxl",
            "Rock.hva",
            "RockA.vxl",
            "RockA.hva",
            "RockTUR.vxl",
            "RockTUR.hva",
            "Ball.shp",
            "BallA.shp",
        ]);
        let q = ArtQuirks::default();
        let (o, a, _) = run_q(&[(&base, Role::Base)], art, Theater::Snow, &f, &q);
        let shell = get(&o, &a, Kind::Projectiles, "Shell");
        // The first read turns the flag on: the arctic model, its parts under I ‖ A.
        assert_eq!(
            s(&shell.voxels.get("body").cloned()).as_deref(),
            Some("RockA.vxl")
        );
        assert!(!shell.voxels.contains_key("turret"));
        // Elsewhere the model is I, and a turret part of a non-vehicle is looked for.
        let (o, a, _) = run_q(&[(&base, Role::Base)], art, Theater::Temperate, &f, &q);
        let warm = get(&o, &a, Kind::Projectiles, "Shell");
        assert_eq!(
            s(&warm.voxels.get("body").cloned()).as_deref(),
            Some("Rock.vxl")
        );
        assert!(warm.voxels.contains_key("turret"));
        assert!(shell.projectile.as_ref().unwrap().voxel);
        assert_eq!(shell.sprite, None);
        // A later read with the flag still set names the model by I again.
        let map = "[Shell]\nImage=Rock\n";
        let (o, a, _) = run_q(
            &[(&base, Role::Base), (map, Role::Map)],
            art,
            Theater::Snow,
            &f,
            &q,
        );
        let shell = get(&o, &a, Kind::Projectiles, "Shell");
        assert_eq!(
            s(&shell.voxels.get("body").cloned()).as_deref(),
            Some("Rock.vxl")
        );
        // An invalid motion file discards the voxel art.
        let broken = files(&["Rock.vxl"]);
        let (o, a, _) = run_q(&[(&base, Role::Base)], art, Theater::Temperate, &broken, &q);
        assert!(get(&o, &a, Kind::Projectiles, "Shell").voxels.is_empty());
        // A sprite projectile on snow: never I ‖ A.
        let sprite = format!("{SHOOTER}[Shell]\nImage=Ball\nAlternateArcticArt=yes\n");
        let (o, a, _) = run_q(&[(&sprite, Role::Base)], art, Theater::Snow, &f, &q);
        assert_eq!(
            s(&get(&o, &a, Kind::Projectiles, "Shell").sprite).as_deref(),
            Some("Ball.shp")
        );
        // An invisible projectile: no art looked up.
        let inviso = format!("{SHOOTER}[Shell]\nImage=Ball\nInviso=yes\n");
        let (o, a, _) = run_q(&[(&inviso, Role::Base)], art, Theater::Temperate, &f, &q);
        let shell = get(&o, &a, Kind::Projectiles, "Shell");
        assert_eq!(shell.sprite, None);
        assert!(shell.projectile.as_ref().unwrap().inviso);
    }

    /// spec: S-0306 Import mapping (the drawing fields: held across reads, skipped while the image is empty, animation fields modulo 256 with a warning; Shadow default yes; FirersPalette default no)
    #[test]
    fn projectile_drawing_fields() {
        let base = format!("{SHOOTER}[Shell]\nImage=Ball\n");
        let art = "[Ball]\nRotates=yes\nAnimLow=2\nAnimHigh=261\nAnimRate=-1\nAnimPalette=yes\nFlat=yes\n[Odd]\nAnimLow=\nRotates=no\n";
        let f = files(&["Ball.shp", "Odd.shp"]);
        let q = ArtQuirks::default();
        let (o, a, w) = run_q(&[(&base, Role::Base)], art, Theater::Temperate, &f, &q);
        let look = get(&o, &a, Kind::Projectiles, "Shell")
            .projectile
            .clone()
            .unwrap();
        assert!(look.rotates && look.anim_palette && look.flat && look.shadow);
        assert!(!look.firers_palette && !look.voxel && !look.inviso);
        assert_eq!((look.anim_low, look.anim_high, look.anim_rate), (2, 5, 255));
        assert_eq!(
            w.iter().filter(|m| m.contains("outside 0 to 255")).count(),
            2
        );
        // A later read naming another section: its given keys replace, an empty value keeps.
        let map = "[Shell]\nImage=Odd\nShadow=no\nFirersPalette=yes\n";
        let (o, a, _) = run_q(
            &[(&base, Role::Base), (map, Role::Map)],
            art,
            Theater::Temperate,
            &f,
            &q,
        );
        let look = get(&o, &a, Kind::Projectiles, "Shell")
            .projectile
            .clone()
            .unwrap();
        assert!(!look.rotates);
        assert_eq!(look.anim_low, 2);
        assert!(!look.shadow && look.firers_palette);
        // A read with an empty image reads nothing: the fields keep their values.
        let empty = "[Shell]\nArm=1\n";
        let (o, a, _) = run_q(
            &[(&base, Role::Base), (empty, Role::Map)],
            art,
            Theater::Temperate,
            &f,
            &q,
        );
        let look = get(&o, &a, Kind::Projectiles, "Shell")
            .projectile
            .clone()
            .unwrap();
        assert!(look.rotates);
        assert_eq!(look.anim_high, 5);
    }

    /// spec: S-0306 claim 30 and Import mapping (Color: a known name in any case selects its 53-shade palette; missing re-selects the definition in force; an unknown name keeps the palette in force, at first the first definition's single-shade palette)
    #[test]
    fn projectile_colour_walk() {
        let colours = "[Colors]\nRuby=0,200,200\nJade=90,200,200\n";
        let with = |c: &str| format!("{colours}{SHOOTER}[Shell]\nImage=Rock\n{c}");
        let colour = |rules: &[(&str, Role)]| {
            let (o, a, _) = run_q(
                rules,
                "",
                Theater::Temperate,
                &files(&[]),
                &ArtQuirks::default(),
            );
            get(&o, &a, Kind::Projectiles, "Shell")
                .projectile
                .clone()
                .unwrap()
                .colour
        };
        assert_eq!(
            colour(&[(&with(""), Role::Base)]),
            ProjectileColour::Shades(0)
        );
        assert_eq!(
            colour(&[(&with("Color=jade\n"), Role::Base)]),
            ProjectileColour::Shades(1)
        );
        assert_eq!(
            colour(&[(&with("Color=Nope\n"), Role::Base)]),
            ProjectileColour::Single(0)
        );
        // Jade, then an unknown name keeps it; then a read without the key re-selects Jade's 53 shades.
        let map = "[Shell]\nColor=Nope\n";
        assert_eq!(
            colour(&[(&with("Color=Jade\n"), Role::Base), (map, Role::Map)]),
            ProjectileColour::Shades(1)
        );
        let after_unknown = "[Shell]\nArm=2\n";
        assert_eq!(
            colour(&[
                (&with("Color=Nope\n"), Role::Base),
                (after_unknown, Role::Map)
            ]),
            ProjectileColour::Shades(0)
        );
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
