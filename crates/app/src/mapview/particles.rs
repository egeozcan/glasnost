//! How the game client draws a particle (S-0613, S-0616): its type's
//! import entry, its image frame and its translucency code. The rest of
//! the drawing (sprite, palette, placement, layer) is the interim rule of
//! Q-1520 (docs/development.md, *Visual review 2026-10-19*).

use std::collections::BTreeMap;

use glasnost_convert::Import;
use glasnost_convert::kinds::Kind;
use glasnost_convert::layers::What;
use glasnost_ini::value;
use glasnost_render::light::DetailLevel;
use glasnost_sim::gas::Behaviour;

/// A particle type of the import, for drawing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticleEntry {
    /// Its import registration index (its art).
    pub idx: usize,
    /// The start-frame field.
    pub start_frame: i32,
}

/// The import entry of every particle type of the simulation's rules, by
/// the simulation's type index: `names` the simulation's type names.
/// `None` for a name the import does not list.
pub fn entries(import: &Import, names: &[Vec<u8>]) -> Vec<Option<ParticleEntry>> {
    let mut by_name: BTreeMap<Vec<u8>, ParticleEntry> = BTreeMap::new();
    if let Some(reg) = import.outcome.registries.get(&Kind::Particles) {
        for (idx, t) in reg.iter().enumerate() {
            // spec: S-0613 Import mapping (StartFrame: an integer, absent 0), S-0003 claim 3 (an unreadable integer keeps the value in force)
            let mut start_frame = None;
            if let Some(k) = t.keys.get(&b"StartFrame"[..]) {
                for e in &k.history {
                    match &e.what {
                        What::Set(v) => start_frame = value::integer(v).or(start_frame),
                        What::Reset => start_frame = None,
                        _ => {}
                    }
                }
            }
            // spec: S-0003 claim 14 (names match with ASCII case folding; the first registered wins)
            by_name
                .entry(t.name.to_ascii_uppercase())
                .or_insert(ParticleEntry {
                    idx,
                    start_frame: start_frame.unwrap_or(0),
                });
        }
    }
    names
        .iter()
        .map(|n| by_name.get(&n.to_ascii_uppercase()).cloned())
        .collect()
}

/// The drawn image frame of a particle at stage `stage` whose type has
/// behaviour kind `kind` and start-frame field `start_frame`.
pub fn frame(kind: Option<Behaviour>, stage: i8, start_frame: i32) -> i32 {
    if kind == Some(Behaviour::Gas) {
        // spec: S-0613 claim 15 and Terms (a gas particle, by its type's kind: the stage itself, the start frame not added)
        i32::from(stage)
    } else {
        // spec: Q-1520 interim rule (the other kinds: the stage plus the start frame)
        i32::from(stage).saturating_add(start_frame)
    }
}

/// The translucency code (0 opaque, 1, 2 and 3 for 25, 50 and 75 %) of a
/// particle whose type's stored translucency is `t`, at detail `detail`.
/// The stage fields and a smoke successor's raised value are not drawn
/// (Q-1520 interim rule).
pub fn translucency(t: i8, detail: DetailLevel) -> u8 {
    // spec: S-0613 claim 16 (opaque below the highest detail setting)
    if detail != DetailLevel::High {
        return 0;
    }
    // spec: S-0613 claim 16 (25 and 50 used as they are, a stored signed 75 … 127 drawn as 75, any other value opaque)
    match t {
        25 => 1,
        50 => 2,
        75..=127 => 3,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0613 claim 15 (a gas particle's frame is its stage, the
    /// start frame not added); Q-1520 interim rule (stage plus start frame
    /// for the other kinds)
    #[test]
    fn frames() {
        assert_eq!(frame(Some(Behaviour::Gas), 3, 5), 3);
        assert_eq!(frame(Some(Behaviour::Smoke), 3, 5), 8);
        assert_eq!(frame(Some(Behaviour::Fire), 0, 2), 2);
        assert_eq!(frame(Some(Behaviour::Spark), 1, 0), 1);
        assert_eq!(frame(Some(Behaviour::Railgun), 2, 1), 3);
        assert_eq!(frame(None, 2, 1), 3);
        assert_eq!(frame(Some(Behaviour::Gas), -1, 4), -1);
    }

    /// spec: S-0613 claim 16 and scenario G11 (at the highest detail 25 and
    /// 50 as they are, 75 … 127 as 75, others opaque; imported 127, 128,
    /// 255 and 281 stored as signed bytes give 75 %, opaque, opaque and
    /// 25 %; opaque at the other detail settings)
    #[test]
    fn translucency_codes() {
        let high = DetailLevel::High;
        for (t, code) in [
            (0, 0),
            (25, 1),
            (50, 2),
            (75, 3),
            (80, 3),
            (127, 3),
            (30, 0),
        ] {
            assert_eq!(translucency(t, high), code, "translucency {t}");
        }
        // spec: S-0613 claim 16 (reduced modulo 256 at creation, signed)
        for (imported, code) in [(127, 3), (128, 0), (255, 0), (281, 1)] {
            assert_eq!(
                translucency(imported as i8, high),
                code,
                "imported {imported}"
            );
        }
        for t in [25, 50, 75, 100] {
            assert_eq!(translucency(t, DetailLevel::Medium), 0);
            assert_eq!(translucency(t, DetailLevel::Low), 0);
        }
    }
}
