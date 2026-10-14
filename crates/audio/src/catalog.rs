//! The sound catalog: what a sound name (from the rules data) plays.
//!
//! The meaning of the player's sound and speech databases is not specified
//! yet (Q-0260, Q-0261; planned S-0701). Until it is, the catalog runs on an
//! interim rule: a sound name plays the single sample of the same name, at
//! full gain, with the default priority and no per-sound limit. Entries can
//! be defined explicitly ([`SoundCatalog::define`]); that is the seam a sound
//! spec will fill from the databases.

use std::collections::BTreeMap;

use crate::mixer::UNITY;

/// Priority of a sound with no priority of its own (our design, Q-0260).
pub const DEFAULT_PRIORITY: i32 = 0;

/// What a sound plays.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoundDef {
    /// Sample names it may play. With several, the first is played (no
    /// variant choice is specified yet, Q-0260, so no draw is taken).
    pub samples: Vec<String>,
    /// Gain, [`UNITY`] = full.
    pub gain: u32,
    /// Priority for voice stealing.
    pub priority: i32,
    /// At most this many copies at once; `None`: no limit.
    pub limit: Option<u16>,
}

impl SoundDef {
    /// The interim definition of a name (Q-0260): its own sample, full
    /// gain, default priority, no limit.
    pub fn same_name(name: &str) -> Self {
        Self {
            samples: vec![name.to_owned()],
            gain: UNITY,
            priority: DEFAULT_PRIORITY,
            limit: None,
        }
    }
}

/// Sound names and what they play.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SoundCatalog {
    defs: BTreeMap<String, SoundDef>,
    same_name: bool,
}

impl SoundCatalog {
    /// The interim catalog (Q-0260): every name not defined explicitly
    /// plays the sample of the same name.
    pub fn interim() -> Self {
        Self {
            defs: BTreeMap::new(),
            same_name: true,
        }
    }

    /// A catalog that knows only explicitly defined names.
    pub fn explicit() -> Self {
        Self::default()
    }

    /// Defines a name (ASCII case folded).
    pub fn define(&mut self, name: &str, def: SoundDef) {
        self.defs.insert(name.to_ascii_uppercase(), def);
    }

    /// What a name plays, or `None` for a name the catalog does not know.
    pub fn resolve(&self, name: &str) -> Option<SoundDef> {
        if name.is_empty() {
            return None;
        }
        match self.defs.get(&name.to_ascii_uppercase()) {
            Some(d) => Some(d.clone()),
            None if self.same_name => Some(SoundDef::same_name(name)),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interim_names_play_their_own_sample() {
        let c = SoundCatalog::interim();
        assert_eq!(c.resolve("Zing").unwrap().samples, ["Zing"]);
        assert_eq!(c.resolve("Zing").unwrap().gain, UNITY);
        assert!(c.resolve("").is_none());
        assert!(SoundCatalog::explicit().resolve("Zing").is_none());
    }

    #[test]
    fn explicit_definitions_win() {
        let mut c = SoundCatalog::interim();
        c.define(
            "zing",
            SoundDef {
                samples: vec!["a".into(), "b".into()],
                gain: UNITY / 4,
                priority: 3,
                limit: Some(1),
            },
        );
        let d = c.resolve("ZING").unwrap();
        assert_eq!((d.samples.len(), d.priority, d.limit), (2, 3, Some(1)));
    }
}
