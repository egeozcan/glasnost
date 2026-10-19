//! The own AI and the genetic mutator (S-0611; PLAN §4.7).
//!
//! A launch turns the enemy infantry it kills into infantry of the
//! launching house (S-0611 claims 3, 4, 7 and 9), and it kills any infantry
//! in its reach, the house's own and its allies' included (claim 4: every
//! listed infantry of any house). So a person fires it at a dense group of
//! enemy infantry that it sees, with none of its own or its allies'
//! infantry inside the reach, and holds it while no group is worth it.
//!
//! - The building that gives the mutator is built by the brain's
//!   superweapon step, like every other kind it fires
//!   ([`crate::brain::Brain`], `build_superweapon`).
//! - [`mutate_cell`] picks the target cell: the seen enemy infantry cell
//!   whose reach holds the most seen enemy infantry, at least [`MIN_GROUP`],
//!   and no infantry of the house or its allies (in explosion mode, no
//!   object of theirs at all, since the blast hurts every object). The reach
//!   is what a person reads off the cursor and the map: in cell mode the
//!   effect block, each cell in its swept layer (S-0513 F1 and claim 4); in
//!   explosion mode the blast's radius around the cell centre, in the one
//!   layer it hits (S-0303 claims 3 and 4: infantry on a bridge deck is out
//!   of reach of a blast beside or beneath it).
//! - Weapons whose warhead has the mutating death kind need no rule of
//!   their own: the brain sends them like any weapon.
//!
//! It reads only what a person in the house's seat sees and draws no random
//! value. The numbers are tuning of our own opponent, not game behaviour.

use glasnost_map::CellXY;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{Layer, Object, World};

use crate::scripted::Script;

/// The fewest seen enemy infantry in the reach for which the mutator is
/// fired.
pub const MIN_GROUP: usize = 3; // nocite: tuning of our own opponent

/// What a launch at `c` reaches, as a person reads it off the aiming
/// cursor and the map.
enum Reach {
    /// Cell mode: the effect block, each cell in its swept layer (S-0611
    /// claim 4, S-0513 F1 and claim 4).
    Block(Vec<(CellXY, Vec<Layer>)>),
    /// Explosion mode: the objects of one cell layer within the blast's
    /// radius of the cell centre (S-0611 claim 3, S-0303 claims 3 and 4).
    Blast {
        x: i64,
        y: i64,
        r: i64,
        layer: Layer,
    },
}

impl Reach {
    fn of(w: &World, c: CellXY) -> Reach {
        let m = &w.rules().superweapons.mutation;
        if !m.explosion {
            return Reach::Block(
                glasnost_sim::superweapons::block(w, c)
                    .into_iter()
                    .map(|(_, x)| (x, glasnost_sim::superweapons::swept_layers(w, x)))
                    .collect(),
            );
        }
        let r = m
            .explosion_warhead
            .map_or(0, |h| glasnost_sim::combat::area::radii(w, h).0);
        // spec: S-0611 F1 (the mutation point: the cell centre, lifted onto the deck on a high-bridge cell)
        // spec: S-0303 claim 4 (a detonation on a bridge deck hits only deck objects, else only ground layers)
        let layer = if w.high_bridge(c) {
            Layer::Deck
        } else {
            Layer::Ground
        };
        Reach::Blast {
            x: i64::from(c.x) * 256 + 128, // spec: S-0611 F1
            y: i64::from(c.y) * 256 + 128, // spec: S-0611 F1
            r: i64::from(r),
            layer,
        }
    }

    fn holds(&self, o: &Object) -> bool {
        match self {
            Reach::Block(cells) => {
                let c = o.pos.cell();
                cells
                    .iter()
                    .any(|(x, layers)| *x == c && layers.contains(&o.layer))
            }
            Reach::Blast { x, y, r, layer } => {
                let (dx, dy) = (i64::from(o.pos.x) - x, i64::from(o.pos.y) - y);
                o.layer == *layer && dx * dx + dy * dy <= r * r
            }
        }
    }
}

/// The genetic mutator's target cell for the house of `script`, if a group
/// worth it is seen.
pub(crate) fn mutate_cell(script: &Script, w: &World) -> Option<CellXY> {
    let house = script.house;
    let me = &w.houses[usize::from(house.0)];
    let explosion = w.rules().superweapons.mutation.explosion;
    let enemies: Vec<&Object> = script
        .seen_enemies(w)
        .into_iter()
        .filter_map(|(id, _)| w.obj(id))
        .filter(|o| o.kind == Kind::Infantry)
        .collect();
    let friends: Vec<&Object> = w
        .objects
        .values()
        .filter(|o| o.on_map && o.health > 0)
        .filter(|o| {
            o.owner
                .is_some_and(|h| h == house || me.allies.contains(&h))
        })
        .filter(|o| explosion || o.kind == Kind::Infantry)
        .collect();
    let mut cands: Vec<CellXY> = enemies.iter().map(|o| o.pos.cell()).collect();
    cands.sort_by_key(|c| (c.y, c.x));
    cands.dedup();
    cands
        .into_iter()
        .filter_map(|c| {
            let reach = Reach::of(w, c);
            if friends.iter().any(|f| reach.holds(f)) {
                return None;
            }
            let n = enemies.iter().filter(|e| reach.holds(e)).count();
            (n >= MIN_GROUP).then_some((n, c))
        })
        .max_by_key(|&(n, c)| (n, std::cmp::Reverse((c.y, c.x))))
        .map(|(_, c)| c)
}

/// The objects in play that a launch at `c` reaches, as a person reads the
/// reach off the cursor: infantry in cell mode, every object of the blast's
/// layer in explosion mode.
pub(crate) fn reached(w: &World, c: CellXY) -> Vec<glasnost_sim::world::ObjectId> {
    let explosion = w.rules().superweapons.mutation.explosion;
    let reach = Reach::of(w, c);
    w.objects
        .values()
        .filter(|o| o.on_map && o.health > 0)
        .filter(|o| explosion || o.kind == Kind::Infantry)
        .filter(|o| reach.holds(o))
        .map(|o| o.id)
        .collect()
}
