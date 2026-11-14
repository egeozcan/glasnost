//! The own AI and the genetic mutator (S-0611; PLAN §4.7).
//!
//! A launch turns the infantry it kills into infantry of the launching
//! house (S-0611 claims 3, 4, 7 to 9: the conversion animation's owner is
//! the launching house), and it kills any infantry in its reach, the
//! house's own and its allies' included (claim 4: every listed infantry of
//! any house). So the house loses each own infantry unit in the reach but
//! gets a made unit for it, its allies lose theirs, and each enemy unit
//! becomes a made unit of the house. A person fires a ready mutator where
//! the enemy gain clearly outweighs the own and allied loss, even with some
//! of its own infantry inside, and at the best such target it sees.
//!
//! - The building that gives the mutator is built by the brain's
//!   superweapon step, like every other kind it fires
//!   ([`crate::brain::Brain`], `build_superweapon`).
//! - [`mutate_cell`] picks the target cell: among the seen enemy infantry
//!   cells, the one where gain less loss is largest, with the gain at least
//!   [`OUTWEIGH`] times the loss. Gain: each seen enemy infantry unit in
//!   the reach, its value and the made unit's. Loss: each own infantry
//!   unit, its value less the made unit's; each allied one, its value; in
//!   explosion mode also what the blast is expected to destroy of the
//!   house's and its allies' other objects (S-0611 claim 3: 10 000 hit
//!   points). The reach is what a person reads off the cursor and the map:
//!   in cell mode the effect block, each cell in its swept layer (S-0513 F1
//!   and claim 4); in explosion mode the blast's radius around the cell
//!   centre, in the one layer it hits (S-0303 claims 3 and 4: infantry on a
//!   bridge deck is out of reach of a blast beside or beneath it).
//!   Until M7b AI round 3 any own infantry in the reach vetoed the launch.
//! - Weapons whose warhead has the mutating death kind need no rule of
//!   their own: the brain sends them like any weapon.
//!
//! It reads only what a person in the house's seat sees and draws no random
//! value. The numbers are tuning of our own opponent, not game behaviour.

use glasnost_map::CellXY;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{Layer, Object, World};

use crate::scripted::Script;

/// The group size the lane reports the mutator's aim against (the aim
/// itself goes by value).
pub const MIN_GROUP: usize = 3; // nocite: tuning of our own opponent

/// How many times the loss the gain must be for a launch.
pub const OUTWEIGH: i64 = 2; // nocite: tuning of our own opponent (a clear gain, as a person judges it)

/// The blast amount of an explosion-mode launch.
const BLAST: i32 = 10_000; // spec: S-0611 claim 3 (one area hit of 10 000 hit points)

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

/// The value of the unit a conversion makes for the house: the made type
/// of the conversion animation (S-0611 claims 8 and 9, F4), 0 when it
/// makes none.
fn made_value(w: &World) -> i64 {
    let r = w.rules();
    let m = &r.superweapons.mutation;
    let i = m
        .anim
        .and_then(|a| r.animations.get(a as usize))
        .map_or(-1, |a| a.make_infantry);
    // spec: S-0611 F4 (the made type is entry i when 0 ≤ i < n; otherwise nothing is created)
    usize::try_from(i)
        .ok()
        .and_then(|i| m.made.get(i).copied())
        .flatten()
        .map_or(0, |t| {
            i64::from(
                glasnost_sim::econ::production::cost_field(
                    w,
                    glasnost_sim::econ::TypeRef {
                        kind: Kind::Infantry,
                        index: t,
                    },
                )
                .max(1),
            )
        })
}

/// The best target cell for the house of `script` and what it is worth:
/// `(cell, gain, loss, enemy infantry reached)`, among the cells where the
/// gain is at least [`OUTWEIGH`] times the loss.
fn best(script: &Script, w: &World) -> Option<(CellXY, i64, i64, usize)> {
    let house = script.house;
    let me = &w.houses[usize::from(house.0)];
    let m = &w.rules().superweapons.mutation;
    let explosion = m.explosion;
    let made = made_value(w);
    let enemies: Vec<&Object> = script
        .seen_enemies(w)
        .into_iter()
        .filter_map(|(id, _)| w.obj(id))
        .filter(|o| o.kind == Kind::Infantry)
        .collect();
    let own = |o: &&Object| o.owner == Some(house);
    let friends: Vec<&Object> = w
        .objects
        .values()
        .filter(|o| o.on_map && o.health > 0 && o.kind == Kind::Infantry)
        .filter(|o| o.owner.is_some_and(|h| h == house || me.allied_with(h)))
        .collect();
    let mut owners = vec![house];
    owners.extend(me.allies.iter().copied().filter(|&h| h != house));
    let mut cands: Vec<CellXY> = enemies.iter().map(|o| o.pos.cell()).collect();
    cands.sort_by_key(|c| (c.y, c.x));
    cands.dedup();
    cands
        .into_iter()
        .filter_map(|c| {
            let reach = Reach::of(w, c);
            let hit: Vec<&&Object> = enemies.iter().filter(|e| reach.holds(e)).collect();
            if hit.is_empty() {
                return None;
            }
            let gain: i64 = hit
                .iter()
                .map(|o| crate::dominator::value(w, o) + made)
                .sum();
            let mut loss: i64 = friends
                .iter()
                .filter(|f| reach.holds(f))
                .map(|f| {
                    let v = crate::dominator::value(w, f);
                    if own(f) { (v - made).max(0) } else { v }
                })
                .sum();
            if explosion {
                // spec: S-0611 claim 3 (one area hit with the explosion warhead; its reach, falloff and versus are S-0303's and S-0302's) and F1 (centred on the mutation point, lifted by the bridge lift on a high-bridge cell)
                // spec: S-0303 claim 4 (a point on a high-bridge cell more than half a deck above the terrain hits the deck layer only)
                // design: own AI (#34): the estimate reads the blast's own point and layer, not the dominator's ground point
                let blast = crate::dominator::Blast {
                    at: glasnost_sim::superweapons::cell_point(w, c, true),
                    layer: if w.high_bridge(c) {
                        glasnost_sim::world::Layer::Deck
                    } else {
                        glasnost_sim::world::Layer::Ground
                    },
                };
                loss += crate::dominator::blast_loss(
                    w,
                    &owners,
                    blast,
                    BLAST,
                    m.explosion_warhead,
                    false,
                );
            }
            (gain >= OUTWEIGH * loss).then_some((c, gain, loss, hit.len()))
        })
        .max_by_key(|&(c, gain, loss, _)| (gain - loss, std::cmp::Reverse((c.y, c.x))))
}

/// The genetic mutator's target cell for the house of `script`, if a
/// target worth it is seen.
pub(crate) fn mutate_cell(script: &Script, w: &World) -> Option<CellXY> {
    best(script, w).map(|(c, ..)| c)
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

/// A lane diagnostic of the mutator's aim for `house`: the seen enemy
/// infantry, the most of them one launch reaches with no regard to the
/// house's own, and the best target's gain, loss and enemy infantry
/// reached, if one is worth a launch.
pub fn probe(
    w: &World,
    house: glasnost_sim::world::HouseId,
) -> (usize, usize, Option<(i64, i64, usize)>) {
    let script = Script::new(house, crate::brain::default_plan());
    let enemies: Vec<&Object> = script
        .seen_enemies(w)
        .into_iter()
        .filter_map(|(id, _)| w.obj(id))
        .filter(|o| o.kind == Kind::Infantry)
        .collect();
    let mut any = 0;
    for e in &enemies {
        let reach = Reach::of(w, e.pos.cell());
        any = any.max(enemies.iter().filter(|x| reach.holds(x)).count());
    }
    (
        enemies.len(),
        any,
        best(&script, w).map(|(_, g, l, n)| (g, l, n)),
    )
}
