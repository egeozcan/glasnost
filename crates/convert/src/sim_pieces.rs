//! Voxel-animation types: the debris pieces of S-0317 (Import mapping),
//! and the importer warnings its departures name.

use glasnost_num::{F64, Rounding};

use super::combat::{bool_of, int_of, key, last_set};
use super::*;

/// The keys of a voxel-animation type that change play (S-0317 Import
/// mapping): our field name, the INI key and its form. The two angular
/// velocities follow their own layering (S-0006 claims 3 and 5) and are
/// read by [`Builder::voxel_animation`]; `StopSound` has no effect (claim
/// 14) and is not kept.
// spec: S-0317 Import mapping (the voxel-animation type section's keys)
const VOXEL_ANIMATION_FIELDS: &[(&str, &str, Form)] = &[
    ("duration", "Duration", Int),
    ("is_meteor", "IsMeteor", Bool),
    ("max_xy_vel", "MaxXYVel", Dec),
    ("min_z_vel", "MinZVel", Dec),
    ("max_z_vel", "MaxZVel", Dec),
    ("elasticity", "Elasticity", Dec),
    ("damage", "Damage", Int),
    ("damage_radius", "DamageRadius", Int),
    ("warhead", "Warhead", Ref(Kind::Warheads)),
    ("bounce_anim", "BounceAnim", Ref(Kind::Animations)),
    ("expire_anim", "ExpireAnim", Ref(Kind::Animations)),
    ("trailer_anim", "TrailerAnim", Ref(Kind::Animations)),
    ("spawns", "Spawns", Ref(Kind::VoxelAnimations)),
    ("spawn_count", "SpawnCount", Int),
    ("is_tiberium", "IsTiberium", Bool),
    (
        "attached_system",
        "AttachedSystem",
        Ref(Kind::ParticleSystems),
    ),
    ("start_sound", "StartSound", Text),
];

/// The two angular velocities (S-0317 Import mapping, S-0006 claims 3
/// and 5): our field name and the INI key.
const ANGULAR: [(&str, &str); 2] = [
    ("min_angular_velocity", "MinAngularVelocity"),
    ("max_angular_velocity", "MaxAngularVelocity"),
];

/// Every voxel-animation key this module converts, for `coverage.csv`.
pub fn piece_keys() -> Vec<String> {
    let mut out: Vec<String> = VOXEL_ANIMATION_FIELDS
        .iter()
        .map(|f| f.1.to_owned())
        .collect();
    out.extend(ANGULAR.iter().map(|a| a.1.to_owned()));
    // spec: S-0317 claim 14 and Import mapping (StopSound is read and has no effect)
    out.push("StopSound".to_owned());
    out
}

/// The degrees-to-radians constant (S-0317 Constants: 3.1415 / 180).
pub(crate) const DEGREES_TO_RADIANS: &str = "0.01745277777777778"; // spec: S-0317 Constants (degrees to radians 0.01745277777777778)

/// An angular velocity as the rules layers leave it, in degrees per tick
/// (S-0006 claims 3 and 5): a section that omits the key resets it to −1;
/// a value of 0 keeps the held value; `None` when no layer gave a value or
/// a reset, so the engine default before any reading stays (0 for the
/// minimum, 10 for the maximum).
fn angular(state: Option<&KeyState>) -> Option<String> {
    let mut held = None;
    for e in state.into_iter().flat_map(|s| &s.history) {
        match &e.what {
            What::Set(v) => {
                if let Some(d) = decimal_text(v) {
                    held = Some(d);
                }
            }
            // spec: S-0006 claim 5 and Constants (a voxel animation's angular velocities reset to −1 when a section omits them)
            What::Reset => held = Some("-1".to_owned()),
            _ => {}
        }
    }
    held
}

/// A decimal key's value, parsed as S-0003 reads it, `None` when no layer
/// gave one.
fn decimal_value(t: &TypeRec, k: &str, mode: Rounding) -> Option<F64> {
    let text = fold(key(t, k), None, |v, _| decimal_text(v))?;
    glasnost_num::decimal::parse(&text, glasnost_num::decimal::Field::Decimal, mode).ok()
}

/// The conversion of S-0205 claim 7 is in range for a span value: its
/// truncation fits a signed 32-bit integer.
fn in_range(x: F64) -> bool {
    x.trunc_i128()
        .is_some_and(|t| (i128::from(i32::MIN)..=i128::from(i32::MAX)).contains(&t))
}

impl Builder<'_> {
    /// A voxel-animation type's record (S-0317 Import mapping).
    pub(super) fn voxel_animation(&mut self, rec: &mut Rec, t: &TypeRec) {
        self.fields(rec, t, Kind::VoxelAnimations, VOXEL_ANIMATION_FIELDS);
        for (field, k) in ANGULAR {
            if let Some(d) = angular(key(t, k)) {
                rec.put(field, jsonc::string(d));
            }
        }
        self.piece_type_checks(t);
    }

    /// The importer warnings of one piece type (S-0317 claims 12 and 16,
    /// Terms).
    fn piece_type_checks(&mut self, t: &TypeRec) {
        if t.reads.is_empty() {
            return;
        }
        // The spans as F1, F2 and F5 form them, under the default rounding (claim 16: the intervals hold up to that rounding).
        let m = Rounding::TowardZero;
        let name = show(&t.name);
        let num = |s: &str| {
            glasnost_num::decimal::parse(s, glasnost_num::decimal::Field::Decimal, m)
                .unwrap_or_default()
        };
        // spec: S-0317 Import mapping (MaxXYVel 15.0, MinZVel 3.5, MaxZVel 5.0 when absent)
        let big_m = decimal_value(t, "MaxXYVel", m).unwrap_or_else(|| num("15.0"));
        let zmin = decimal_value(t, "MinZVel", m).unwrap_or_else(|| num("3.5"));
        let zmax = decimal_value(t, "MaxZVel", m).unwrap_or_else(|| num("5.0"));
        let to_rad = num(DEGREES_TO_RADIANS);
        let deg = |k: &str, default: &str| {
            let d = angular(key(t, k)).unwrap_or_else(|| default.to_owned());
            glasnost_num::decimal::parse(&d, glasnost_num::decimal::Field::Decimal, m)
                .unwrap_or_default()
                .mul(to_rad, m)
        };
        // spec: S-0317 Import mapping (the engine defaults before any reading: 0 and 10 degrees)
        let amin = deg("MinAngularVelocity", "0");
        let amax = deg("MaxAngularVelocity", "10");
        let one = F64::from_i64(1, m);
        let meteor = bool_of(key(t, "IsMeteor")).unwrap_or(false);
        // spec: S-0317 F1 (S_xy = trunc(2M), S_z = trunc(Z_max − Z_min + 1)) and F5 (S_a = trunc(A_max − A_min + 1))
        let sxy = big_m.mul(F64::from_i64(2, m), m);
        let sz = zmax.sub(zmin, m).add(one, m);
        let sa = amax.sub(amin, m).add(one, m);
        let mut spans = vec![("horizontal", "MaxXYVel", sxy)];
        if !meteor {
            // spec: S-0317 claim 16 (a falling piece's vertical span is not used by a meteor piece)
            spans.push(("vertical", "MaxZVel", sz));
        }
        spans.push(("spin", "MaxAngularVelocity", sa));
        for (what, k, s) in spans {
            if !in_range(s) {
                // spec: S-0317 Terms (the importer warns for a type whose fields make a conversion of this spec out of range, S-0205)
                self.warn_at(key(t, k), format!(
                    "voxel animation {name}: its {what} span is outside the 32-bit integer range; the conversion follows S-0205 claim 7 (S-0317 Terms)"
                ));
            } else if s.to_i32_common() == 0 {
                // spec: S-0317 claim 16 (a span of 0 is a departure: the remainder equals the dividend, and the importer warns)
                self.warn_at(key(t, k), format!(
                    "voxel animation {name}: its {what} span is 0, which stops the original with a division by zero; our engine takes the remainder as the dividend (S-0317 claim 16, a departure)"
                ));
            }
        }
        let count = key(t, "SpawnCount");
        if int_of(count).unwrap_or(0) == i32::MAX {
            // spec: S-0317 claim 12 and S-0202 claim 5 (a spawn count of 2147483647: each draw has the span 2147483647, and the importer warns)
            self.warn_at(count, format!(
                "voxel animation {name}: SpawnCount 2147483647 gives each spawn draw the span of S-0202 claim 5 (S-0317 claim 12)"
            ));
        }
        if bool_of(key(t, "IsTiberium")).unwrap_or(false) {
            self.piece_resource_check(t);
        }
    }

    /// Claim 12's departure: the overlay type at position 127 is not a
    /// resource overlay, or the list is shorter.
    fn piece_resource_check(&mut self, t: &TypeRec) {
        let outcome = self.i.outcome;
        let overlays = outcome.registries.get(&Kind::Overlays);
        // spec: S-0317 claim 12 (the tiberium of the overlay type at position 127, counting from 0)
        let at = overlays.and_then(|o| o.get(127));
        let resource = at.is_some_and(|o| bool_of(key(o, "Tiberium")).unwrap_or(false));
        if !resource {
            let (layer, line) = last_set(key(t, "IsTiberium"));
            // spec: S-0317 claim 12 (an overlay type at position 127 that is not a resource overlay, or a list of fewer than 128: a departure; no new resource cell, no draws, and the importer warns)
            self.warn(layer, line, format!(
                "voxel animation {}: IsTiberium needs a resource overlay at position 127 of the overlay list; {}: the original's result is undefined, our engine adds no resource (S-0317 claim 12, a departure)",
                show(&t.name),
                if at.is_none() { "the list has fewer than 128 entries" } else { "that overlay type is not a resource overlay" },
            ));
        }
    }
}
