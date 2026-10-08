//! Lookup precedence: which archive wins when several hold the same name.
//!
//! **Provisional.** Which copy of a file the game uses when several archives
//! (or loose files) hold it is game behaviour and needs a spec; it is asked
//! in spec request Q-0002. Until that is answered the default,
//! [`PlaceholderPrecedence`], is an arbitrary, deterministic order of our
//! own, and the importer must not rely on it for rules data. Callers that
//! need a particular order plug in their own [`Precedence`] (for example
//! [`NamedPrecedence`] from a user setting).

use crate::ArchiveId;

/// What a [`Precedence`] knows about each mounted archive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveSummary {
    /// The archive.
    pub id: ArchiveId,
    /// Its own name (a top-level file name, a known name, or the hex ID).
    pub name: String,
    /// The archive that contains it, if nested.
    pub parent: Option<ArchiveId>,
    /// Nesting depth (top-level archives are 0).
    pub depth: u32,
}

/// Orders archives for lookup.
pub trait Precedence {
    /// Returns every archive in `archives` once, highest precedence first.
    /// `archives` is in mount order.
    fn order(&self, archives: &[ArchiveSummary]) -> Vec<ArchiveId>;
}

/// Placeholder order pending Q-0002: mount order, i.e. top-level archives
/// sorted by case-insensitive file name, each followed depth-first by the
/// archives nested in it. The first archive holding a name wins.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlaceholderPrecedence;

impl Precedence for PlaceholderPrecedence {
    fn order(&self, archives: &[ArchiveSummary]) -> Vec<ArchiveId> {
        archives.iter().map(|a| a.id).collect()
    }
}

/// A user-chosen order: archives whose name matches an entry of `names`
/// (case-insensitive), in that order, ahead of all others; nested archives
/// follow their parent's position. Unlisted archives keep the placeholder
/// order after the listed ones.
#[derive(Clone, Debug, Default)]
pub struct NamedPrecedence {
    /// Archive names, highest precedence first.
    pub names: Vec<String>,
}

impl Precedence for NamedPrecedence {
    fn order(&self, archives: &[ArchiveSummary]) -> Vec<ArchiveId> {
        let rank_of = |a: &ArchiveSummary| {
            self.names
                .iter()
                .position(|n| n.eq_ignore_ascii_case(&a.name))
        };
        // Each archive inherits the rank of its nearest listed ancestor.
        let mut ranks: Vec<Option<usize>> = Vec::with_capacity(archives.len());
        for a in archives {
            let inherited = a
                .parent
                .and_then(|p| archives.iter().position(|x| x.id == p))
                .and_then(|i| ranks.get(i).copied().flatten());
            ranks.push(rank_of(a).or(inherited));
        }
        let mut order: Vec<usize> = (0..archives.len()).collect();
        // Stable: ties keep mount order.
        order.sort_by_key(|&i| ranks[i].unwrap_or(usize::MAX));
        order.into_iter().map(|i| archives[i].id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(id: u32, name: &str, parent: Option<u32>) -> ArchiveSummary {
        ArchiveSummary {
            id: ArchiveId(id),
            name: name.into(),
            parent: parent.map(ArchiveId),
            depth: u32::from(parent.is_some()),
        }
    }

    #[test]
    fn named_order_lifts_listed_archives_and_their_children() {
        let archives = [
            summary(0, "a.mix", None),
            summary(1, "inner.mix", Some(0)),
            summary(2, "b.mix", None),
            summary(3, "c.mix", None),
        ];
        let order = NamedPrecedence {
            names: vec!["C.MIX".into(), "a.mix".into()],
        }
        .order(&archives);
        assert_eq!(
            order,
            [ArchiveId(3), ArchiveId(0), ArchiveId(1), ArchiveId(2)]
        );
        assert_eq!(
            PlaceholderPrecedence.order(&archives),
            [ArchiveId(0), ArchiveId(1), ArchiveId(2), ArchiveId(3)]
        );
    }
}
