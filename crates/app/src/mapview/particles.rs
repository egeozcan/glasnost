//! How the game client draws a particle (S-0613, S-0616 claims 23 to 30):
//! its type's import entry, whether it is drawn, its image frame (with
//! F15's direction group for fire) and its translucency code. The
//! placement, depth and pixel drawing are in `build::Painter::particles`.

use std::collections::BTreeMap;

use glasnost_convert::Import;
use glasnost_convert::kinds::Kind;
use glasnost_num::Rounding;
use glasnost_render::light::DetailLevel;
use glasnost_sim::gas::Behaviour;
use glasnost_sim::world::Pos;

/// A particle type of the import, for drawing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleEntry {
    /// Its import registration index (its art).
    pub idx: usize,
}

/// The import entry of every particle type of the simulation's rules, by
/// the simulation's type index: `names` the simulation's type names.
/// `None` for a name the import does not list.
pub fn entries(import: &Import, names: &[Vec<u8>]) -> Vec<Option<ParticleEntry>> {
    let mut by_name: BTreeMap<Vec<u8>, ParticleEntry> = BTreeMap::new();
    if let Some(reg) = import.outcome.registries.get(&Kind::Particles) {
        for (idx, t) in reg.iter().enumerate() {
            // spec: S-0003 claim 14 (names match with ASCII case folding; the first registered wins)
            by_name
                .entry(t.name.to_ascii_uppercase())
                .or_insert(ParticleEntry { idx });
        }
    }
    names
        .iter()
        .map(|n| by_name.get(&n.to_ascii_uppercase()).cloned())
        .collect()
}

/// Whether a particle's kind is a pixel kind (spark or railgun), drawn as
/// one pixel of its colour rather than with its sprite (S-0616 claims 23
/// and 28).
pub fn is_pixel(kind: Option<Behaviour>) -> bool {
    // spec: S-0616 claim 23 (pixel kinds are spark and railgun; sprite kinds gas, smoke, fire and no kind)
    matches!(kind, Some(Behaviour::Spark | Behaviour::Railgun))
}

/// Whether a particle of kind `kind` is drawn at detail `detail` (S-0616
/// claim 23): the lowest setting hides the smoke and spark kinds. S-0613
/// claim 16's performance cutoff is not modelled (it needs a drawn-frame
/// rate) and fog is not reachable in skirmish.
pub fn shown(kind: Option<Behaviour>, detail: DetailLevel) -> bool {
    // spec: S-0616 claim 23 (at the lowest detail smoke and spark particles are not drawn; gas, fire, railgun and no kind still are)
    !(detail == DetailLevel::Low && matches!(kind, Some(Behaviour::Smoke | Behaviour::Spark)))
}

/// The direction group *g* of F15 of a fire particle whose creation
/// position and target point project to the screen pixels `from` and `to`
/// (height lift included).
pub fn fire_group(from: (i32, i32), to: (i32, i32), m: Rounding) -> i32 {
    // spec: S-0616 F15 (Δx = x₂ − x₁, Δy = y₁ − y₂, σ = Δy / Δx against τ = sinₜ(π/8) / cosₜ(π/8) in floating point)
    // F15's ratio, ratio τ and bands are F6's with q = σ, and each group
    // joins F6's two opposite directions δ and δ + 4, so g = δ mod 4.
    let p = Pos::new(from.0, from.1, 0);
    let q = Pos::new(to.0, to.1, 0);
    // spec: S-0616 F15 (opposite screen directions share one group; a zero vector is group 0)
    (glasnost_sim::particles::step_direction(p, q, m) % 4) as i32
}

/// The drawn image frame of a sprite-kind particle at stage `stage`
/// (S-0616 claim 25): kind `kind`, end-stage field `end_stage` and, for
/// fire, direction group `group` (F15).
pub fn frame(kind: Option<Behaviour>, stage: i8, end_stage: i8, group: i32) -> i32 {
    match kind {
        // spec: S-0616 claim 25 (gas and smoke: the stage itself; S-0613 claim 15 for gas)
        Some(Behaviour::Gas | Behaviour::Smoke) => i32::from(stage),
        // spec: S-0616 claim 25 (fire: s + g × E, both signed)
        Some(Behaviour::Fire) => i32::from(stage) + group * i32::from(end_stage),
        // spec: S-0616 claim 25 (no kind: frame 0, always; the start-frame field is never used)
        _ => 0,
    }
}

/// The translucency code (0 opaque, 1, 2 and 3 for 25, 50 and 75 %) of a
/// sprite-kind particle whose own translucency value is `t`, at detail
/// `detail` (S-0616 claim 26, by S-0613 claim 16's rule).
pub fn translucency(t: u8, detail: DetailLevel) -> u8 {
    // spec: S-0616 claim 26 and S-0613 claim 16 (opaque below the highest detail setting)
    if detail != DetailLevel::High {
        return 0;
    }
    // spec: S-0616 claim 26 and S-0613 claim 16 (exactly 25 or 50 as they are, a signed reading 75 … 127 drawn as 75, any other value opaque)
    match t as i8 {
        25 => 1,
        50 => 2,
        75..=127 => 3,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const M: Rounding = Rounding::Nearest;

    /// spec: S-0616 claim 25 (gas and smoke: the stage; no kind: 0; fire:
    /// s + g × E, signed; the start-frame field unused; a fire particle held
    /// at its end stage shows (g + 1) × E)
    #[test]
    fn claim25_frames() {
        assert_eq!(frame(Some(Behaviour::Gas), 3, 9, 2), 3);
        assert_eq!(frame(Some(Behaviour::Smoke), 3, 9, 2), 3);
        assert_eq!(frame(None, 5, 9, 2), 0);
        assert_eq!(frame(Some(Behaviour::Fire), 2, 6, 0), 2);
        assert_eq!(frame(Some(Behaviour::Fire), 2, 6, 3), 20);
        assert_eq!(frame(Some(Behaviour::Fire), 6, 6, 1), 12, "(g + 1) × E");
        assert_eq!(frame(Some(Behaviour::Fire), 1, -4, 2), -7, "signed");
        assert_eq!(frame(Some(Behaviour::Gas), -1, 0, 0), -1);
    }

    /// spec: S-0616 F15 (Δy positive up the screen; up or down and the zero
    /// vector 0, up-right or down-left 1, left or right 2, down-right or
    /// up-left 3; opposite directions share a group)
    #[test]
    fn f15_direction_groups() {
        let o = (100, 100);
        for (to, g) in [
            ((100, 100), 0),
            ((100, 40), 0),
            ((100, 160), 0),
            ((103, 40), 0),
            ((140, 60), 1),
            ((60, 140), 1),
            ((160, 103), 2),
            ((40, 97), 2),
            ((140, 140), 3),
            ((60, 60), 3),
        ] {
            assert_eq!(fire_group(o, to, M), g, "to {to:?}");
        }
        // σ = τ belongs to group 1, just under it to group 2: slope 41/100
        // is under τ ≈ 0.41421, 42/100 over it.
        assert_eq!(fire_group((0, 0), (100, -41), M), 2);
        assert_eq!(fire_group((0, 0), (100, -42), M), 1);
    }

    /// spec: S-0616 claim 23 (the lowest detail hides smoke and spark only)
    #[test]
    fn claim23_detail() {
        use Behaviour::*;
        for k in [Some(Gas), Some(Fire), Some(Railgun), None] {
            assert!(shown(k, DetailLevel::Low), "{k:?}");
        }
        for k in [Some(Smoke), Some(Spark)] {
            assert!(!shown(k, DetailLevel::Low), "{k:?}");
            assert!(shown(k, DetailLevel::Medium), "{k:?}");
            assert!(shown(k, DetailLevel::High), "{k:?}");
        }
        assert!(is_pixel(Some(Spark)) && is_pixel(Some(Railgun)));
        assert!(!is_pixel(Some(Gas)) && !is_pixel(Some(Smoke)) && !is_pixel(None));
    }

    /// spec: S-0616 claim 26 and S-0613 claim 16 (at the highest detail 25
    /// and 50 as they are, a signed 75 … 127 as 75, others opaque: 30,
    /// 128 and up; a successor chain 0, 25, 50, 75, 100, 125, 150 draws
    /// opaque, 25, 50, 75, 75, 75, opaque; opaque at the other settings)
    #[test]
    fn claim26_translucency_codes() {
        let high = DetailLevel::High;
        for (t, code) in [
            (0, 0),
            (25, 1),
            (50, 2),
            (30, 0),
            (127, 3),
            (128, 0),
            (255, 0),
        ] {
            assert_eq!(translucency(t, high), code, "translucency {t}");
        }
        let chain: Vec<u8> = [0, 25, 50, 75, 100, 125, 150]
            .iter()
            .map(|&t| translucency(t, high))
            .collect();
        assert_eq!(chain, [0, 1, 2, 3, 3, 3, 0]);
        for t in [25, 50, 75, 100] {
            assert_eq!(translucency(t, DetailLevel::Medium), 0);
            assert_eq!(translucency(t, DetailLevel::Low), 0);
        }
    }
}
