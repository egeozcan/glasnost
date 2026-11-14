//! Which of the rarer back-references exist anywhere, noted once inside a
//! removal (#402). Our engine's bookkeeping, not game behaviour: it only
//! lets a walk of the object table that would find nothing be skipped.
//!
//! The first walk of every removal, the drop of the object as a held
//! target ([`crate::combat::held::holders_drop`]), also notes whether any
//! object at all is an aircraft or holds an aircraft's state, a spawn
//! controller, a planted bomb, a charger list or a dropper. A later walk of
//! the same removal that looks only at such objects is skipped when the
//! census says none exists. Such a reference is made only at a few places:
//! an object's creation, a spawn controller's, a bomb's planting, a
//! charger's registration, a drop, and the put-back of a removed object for
//! a read. Each of them forgets the census, so the walks after it run as
//! before. The census lives only inside one removal (or inside one drop of
//! a held target at a killing hit or a departure from the map): it is
//! empty between actions, and neither saved nor hashed.

use crate::rules::Kind;
use crate::world::{Object, World};

/// Whether any object holds a reference of each kind (see the module).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Census {
    aircraft: bool,
    spawners: bool,
    bombs: bool,
    chargers: bool,
    dropped: bool,
}

/// The kinds of reference the census notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Held {
    /// An aircraft, or any object with an aircraft's state (a destination
    /// that is an object, a recorded heading pad).
    Aircraft,
    /// A spawn controller (its clear request, pending and controller
    /// targets).
    Spawner,
    /// A planted bomb (its planter).
    Bomb,
    /// A non-empty charger list.
    Chargers,
    /// A dropper.
    Dropped,
}

/// Whether object `o` holds a reference of kind `h`.
fn holds(o: &Object, h: Held) -> bool {
    match h {
        Held::Aircraft => o.kind == Kind::Aircraft || o.air.is_some(),
        Held::Spawner => o.spawn.is_some(),
        Held::Bomb => o.combat.bomb.is_some(),
        Held::Chargers => !o.combat.chargers.is_empty(),
        Held::Dropped => o.lift.dropper.is_some(),
    }
}

impl Census {
    /// Notes object `o`.
    pub(crate) fn add(&mut self, o: &Object) {
        self.aircraft |= holds(o, Held::Aircraft);
        self.spawners |= holds(o, Held::Spawner);
        self.bombs |= holds(o, Held::Bomb);
        self.chargers |= holds(o, Held::Chargers);
        self.dropped |= holds(o, Held::Dropped);
    }

    fn has(self, h: Held) -> bool {
        match h {
            Held::Aircraft => self.aircraft,
            Held::Spawner => self.spawners,
            Held::Bomb => self.bombs,
            Held::Chargers => self.chargers,
            Held::Dropped => self.dropped,
        }
    }
}

/// Forgets the census: a reference of one of its kinds may have been made,
/// or the action it was taken for has ended.
pub(crate) fn forget(w: &mut World) {
    w.census = None;
}

/// Whether the census of the action under way says that no object holds a
/// reference of kind `h`, so that a walk looking only at such objects
/// would find nothing. Without a census: false.
pub(crate) fn none(w: &World, h: Held) -> bool {
    let none = w.census.is_some_and(|c| !c.has(h));
    debug_assert!(
        !none || !w.objects.values().any(|o| holds(o, h)),
        "the census says no object holds {h:?}, but one does"
    );
    none
}
