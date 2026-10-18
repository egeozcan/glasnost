//! Entry outcomes (S-0209 claims 3 to 5): what a moving unit gets for a
//! neighbouring cell.
//!
//! Object states that later milestones own read as their default here:
//! nothing is being erased, invulnerable, a parasite or a slave. The cloak
//! state is S-0516's ([`crate::cloak::fully_cloaked`]).
//! Whether a gate is open is S-0215 claim 35 ([`crate::process::gate_open`]).
//! A wall's owner is S-0210 claim 14.

use glasnost_map::{CellXY, NO_OVERLAY};
use glasnost_rules::sim::MovementZone;

use crate::cells::position;
use crate::passability;
use crate::rules::{Behaviour, Kind};
use crate::world::{Layer, ObjectId, Target, World};

/// The eight entry outcomes in order of increasing severity (S-0209 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Outcome {
    /// 0: the unit may enter.
    Clear,
    /// 1: a cloaked enemy stands there.
    CloakedBlocker,
    /// 2: friendly units on the move, or the vehicle-present flag.
    MovingBlocker,
    /// 3: an allied closed gate.
    ClosedFriendlyGate,
    /// 4: an allied wall that could be destroyed.
    FriendlyObstacle,
    /// 5: an enemy object the unit could destroy.
    EnemyObstacle,
    /// 6: friendly units standing still.
    StationaryFriendly,
    /// 7: the unit may not enter.
    Blocked,
}

/// The cell layer that counts for an asked cell (S-0209 Terms, *cell
/// layer*): (layer whose objects are listed, layer whose record is read).
pub fn layers_for(w: &World, c: CellXY, reference_level: i32) -> (Layer, Layer) {
    let Some(cell) = w.cells.cell(c) else {
        return (Layer::Ground, Layer::Ground);
    };
    let deck = w.cells.idx(c).is_some_and(|i| w.cells.deck[i]);
    if !deck {
        return (Layer::Ground, Layer::Ground);
    }
    let lv = i32::from(cell.level);
    // spec: S-0209 Terms (deck objects when the reference differs by more than 1; deck record when exactly 4 above)
    let objects = if (reference_level - lv).abs() > 1 {
        Layer::Deck
    } else {
        Layer::Ground
    };
    // spec: S-0209 Terms (the deck record when the reference is exactly 4 above)
    let record = if reference_level - lv == 4 {
        Layer::Deck
    } else {
        Layer::Ground
    };
    (objects, record)
}

/// A unit's reference height level: its cell's level, plus 4 on a deck
/// (S-0209 Terms).
pub fn reference_level(w: &World, id: ObjectId) -> i32 {
    let Some(o) = w.obj(id) else { return 0 };
    let lv = w.cells.cell(o.pos.cell()).map_or(0, |c| i32::from(c.level));
    // spec: S-0209 Terms (plus 4 while on a bridge deck)
    lv + if o.layer == Layer::Deck { 4 } else { 0 }
}

/// Whether an object has movement in progress (S-0107 Terms) or a
/// destination.
pub fn moving_or_dest(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .is_some_and(|o| o.dest.is_some() || in_progress(w, id))
}

/// Whether a unit has movement in progress (S-0107 Terms).
pub fn in_progress(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.walk.as_ref().is_some_and(|s| s.target.is_some())
            || o.drive
                .as_ref()
                .is_some_and(|d| d.track.is_some() || d.turning_in_place)
            // spec: S-0503 Terms (a hovering vehicle with a heading point is moving)
            || o.hover.as_ref().is_some_and(|h| h.point.is_some())
            // spec: S-0502 Terms and claim 20 (a jumpjet's locomotor is moving it in every jumpjet state except grounded and hovering)
            || crate::jump::moving_now(w, o.id)
            || !o.path.steps.is_empty()
    })
}

/// Whether a unit counts as moving in the sense of S-0212 claim 27: it has a
/// destination, is turning, or its locomotor reports it moving.
pub fn counts_as_moving(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if o.kind == Kind::Building {
        // spec: S-0212 claim 27 (a building never counts as moving)
        return false;
    }
    o.dest.is_some() || o.body.turning(w.tick) || in_progress(w, id)
}

/// The mean damage armed test of a walker (S-0209 Terms, *armed*).
fn walker_armed(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(u) = w.rules().unit(o.kind, o.type_index) else {
        return false;
    };
    let weapons: Vec<&String> = [&u.primary, &u.secondary].into_iter().flatten().collect();
    if weapons.is_empty() {
        return false;
    }
    let sum: i64 = weapons
        .iter()
        .map(|n| {
            let wt = w.rules().weapons.get(*n);
            i64::from(wt.and_then(|x| x.damage).unwrap_or(0))
                + i64::from(wt.and_then(|x| x.ambient_damage).unwrap_or(0))
        })
        .sum();
    // spec: S-0209 Terms (mean damage = trunc(Σ (d + a) / n); armed when positive)
    sum / weapons.len() as i64 > 0
}

/// Whether a unit has a primary weapon (S-0209 Terms).
fn has_primary(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    match o.kind {
        Kind::Building => w.rules().buildings[o.type_index as usize]
            .raw
            .primary
            .is_some(),
        _ => w
            .rules()
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.primary.is_some()),
    }
}

fn primary_warhead_flag(w: &World, id: ObjectId, wall: bool) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(p) = w
        .rules()
        .unit(o.kind, o.type_index)
        .and_then(|u| u.primary.clone())
    else {
        return false;
    };
    let wh = w.rules().weapons.get(&p).and_then(|x| x.warhead.clone());
    wh.and_then(|h| w.rules().warheads.get(&h).cloned())
        .is_some_and(|h| {
            if wall {
                h.wall.unwrap_or(false)
            } else {
                h.wood.unwrap_or(false)
            }
        })
}

fn primary_hits_ground(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(p) = w
        .rules()
        .unit(o.kind, o.type_index)
        .and_then(|u| u.primary.clone())
    else {
        return false;
    };
    let pj = w.rules().weapons.get(&p).and_then(|x| x.projectile.clone());
    // spec: S-0214 Import mapping (projectile AG absent: yes)
    pj.and_then(|n| w.rules().projectiles.get(&n).cloned())
        .is_none_or(|x| x.ag.unwrap_or(true))
}

/// Whether `mover` is a crusher (S-0210 claim 1).
pub fn is_crusher(w: &World, mover: ObjectId) -> bool {
    let Some(o) = w.obj(mover) else { return false };
    let Some(u) = w.rules().unit(o.kind, o.type_index) else {
        return false;
    };
    // spec: S-0210 claim 1 (the crusher flag or the crusher veterancy ability; with quirks.omnicrusher_needs_crusher off the omni flag too)
    u.crusher
        || crate::combat::veterancy::has_ability(w, mover, crate::combat::data::Ability::Crusher)
        || (!w.rules().quirks.omnicrusher_needs_crusher && u.omni_crusher)
}

/// Whether `obj` can be crushed by `crusher` (S-0210 claim 2).
pub fn crushable_by(w: &World, crusher: ObjectId, obj: ObjectId) -> bool {
    let (Some(c), Some(o)) = (w.obj(crusher), w.obj(obj)) else {
        return false;
    };
    if o.kind == Kind::Terrain {
        return false;
    }
    // spec: S-0210 claim 2 (not allied, from the crusher's side)
    if w.allied(c.owner, o.owner) {
        return false;
    }
    // spec: S-0210 claim 2 (not invulnerable; S-0513 claim 27)
    if crate::superweapons::protected(w, obj) {
        return false;
    }
    let cu = w.rules().unit(c.kind, c.type_index);
    let omni = cu.is_some_and(|u| u.omni_crusher);
    let (crushable, resistant, deployed_uncrushable) = match o.kind {
        Kind::Building => (
            w.rules().buildings[o.type_index as usize]
                .raw
                .crushable
                .unwrap_or(false),
            false,
            false,
        ),
        _ => {
            let u = w.rules().unit(o.kind, o.type_index);
            let deployed = o
                .infantry
                .as_ref()
                .and_then(|s| s.seq)
                .is_some_and(|q| q == crate::infantry::Seq::Deployed);
            (
                u.is_some_and(|u| u.crushable),
                u.is_some_and(|u| u.omni_crush_resistant),
                deployed && u.is_some_and(|u| !u.deployed_crushable),
            )
        }
    };
    // spec: S-0210 claim 2 (omni-crusher on a non-building that is not resistant, or the crushable flag)
    (omni && o.kind != Kind::Building && !resistant) || (crushable && !deployed_uncrushable)
}

/// A unit's mission target (S-0209 Terms).
pub fn mission_target(w: &World, mover: ObjectId) -> Option<Target> {
    let o = w.obj(mover)?;
    // spec: S-0209 Terms (mission target: the destination or target while in a behaviour with one)
    match o.beh.seen() {
        Some(Behaviour::Enter | Behaviour::Capture | Behaviour::Sabotage | Behaviour::Eaten) => {
            o.dest.or(o.target)
        }
        _ => None,
    }
}

/// Whether the wall in `c` belongs to a house allied with the mover's
/// house (the mover's view). A wall without an owner is allied with no
/// house (S-0209 Terms; S-0210 claim 14).
fn wall_allied(w: &World, mover: ObjectId, c: CellXY) -> bool {
    let owner = w.cells.idx(c).and_then(|i| w.cells.wall_owner[i]);
    owner.is_some() && w.obj(mover).is_some_and(|m| w.allied(m.owner, owner))
}

/// The entry outcome of an infantry mover for cell `c` (S-0209 claim 4).
pub fn infantry_outcome(w: &World, mover: ObjectId, c: CellXY, reference: i32) -> Outcome {
    let Some(m) = w.obj(mover) else {
        return Outcome::Blocked;
    };
    let r = w.rules();
    let (obj_layer, rec_layer) = layers_for(w, c, reference);
    let Some(cell) = w.cells.cell(c) else {
        return Outcome::Blocked;
    };
    let armed = walker_armed(w, mover);
    let mission = mission_target(w, mover);
    // 4c: occupants, listing order; overrides.
    let mut values: Vec<(bool, Option<Outcome>)> = Vec::new();
    // 4b: overlays (index 0, ahead of every occupant).
    let mut overlay_value = None;
    let not_human = m.owner.is_some_and(|h| !w.houses[usize::from(h.0)].human);
    if cell.overlay != NO_OVERLAY
        && let Some(o) = r.overlays.get(usize::from(cell.overlay))
        && o.crate_
        && not_human
    {
        // spec: S-0209 claim 4 (row WO1: a crate blocks a walker whose owner is not human-controlled; in our engine the neutral and special houses' walkers, an AI house counting as human-controlled)
        overlay_value = Some(Outcome::Blocked);
    } else if cell.overlay != NO_OVERLAY
        && let Some(o) = r.overlays.get(usize::from(cell.overlay))
        && o.wall
    {
        // spec: S-0209 claim 4 (rows WO2 to WO4: walls by primary weapon and owner)
        overlay_value = Some(
            if !(has_primary(w, mover) && primary_warhead_flag(w, mover, true)) {
                Outcome::Blocked
            } else if wall_allied(w, mover, c) {
                // spec: S-0209 claim 4 (row WO3: a wall owned by an allied house)
                Outcome::FriendlyObstacle
            } else {
                // spec: S-0209 claim 4 (row WO4: a wall of another house, or without an owner)
                Outcome::EnemyObstacle
            },
        );
    }
    let mut override_value: Option<(usize, Outcome)> = None;
    let master = crate::econ::slaves::master_of(w, mover);
    let mut excused = false;
    for &oid in w.cells.list(c, obj_layer) {
        if oid == mover {
            continue;
        }
        let Some(o) = w.obj(oid) else { continue };
        let allied = w.allied(m.owner, o.owner);
        let in_cell_target = mission == Some(Target::Object(oid))
            || mission == Some(Target::Cell(c)) && o.pos.cell() == c;
        let wp2 = master == Some(oid) && crate::econ::slaves::unloading_cell(w, oid) == Some(c);
        let v: (bool, Option<Outcome>) = if wp2 {
            // spec: S-0209 claim 4 (row WP2: the walker's master at its unloading cell gives none, and the walker is excused from the vehicle-present flag)
            excused = true;
            (false, None)
        } else if in_cell_target && !crate::econ::power::being_erased(w, oid) {
            if crate::superweapons::protected(w, oid) {
                // spec: S-0209 claim 4 (row WP3: blocked when that occupant is invulnerable)
                (true, Some(Outcome::Blocked))
            } else {
                // spec: S-0209 claim 4 (row WP3: the mission target overrides: clear, unless that occupant is being erased)
                (true, Some(Outcome::Clear))
            }
        } else if o.kind == Kind::Building {
            let b = &r.buildings[o.type_index as usize];
            if b.invisible {
                // spec: S-0209 claim 4 (row WP7: a building invisible in play gives none)
                (false, None)
            } else if b.gate && crate::process::gate_open(w, oid) {
                // spec: S-0209 claim 4 (row WP11: an open gate gives none; open or closed by S-0215 claim 35)
                (false, None)
            } else if b.gate && w.allied(o.owner, m.owner) {
                // spec: S-0209 claim 4 (row WP12: a closed gate allied with the walker, the gate owner's view)
                (false, Some(Outcome::ClosedFriendlyGate))
            } else if b.gate {
                // spec: S-0209 claim 4 (rows WP13 and WP14: any other closed gate: enemy obstacle with a primary weapon, else blocked)
                (
                    false,
                    Some(if armed {
                        Outcome::EnemyObstacle
                    } else {
                        Outcome::Blocked
                    }),
                )
            } else if b.raw.firestorm_wall.unwrap_or(false) {
                // spec: S-0209 claim 4 (row WP10: firestorm off gives none)
                (false, None)
            } else if allied {
                // spec: S-0209 claim 4 (row WP19: allied building blocked)
                (false, Some(Outcome::Blocked))
            } else if crate::cloak::fully_cloaked(w, oid) {
                // spec: S-0209 claim 4 (row WP21: cloaked enemy object, armed walker or not; the cloak state alone, Terms *cloaked*)
                (false, Some(Outcome::CloakedBlocker))
            } else if !armed {
                // spec: S-0209 claim 4 (row WP22: enemy object, unarmed walker)
                (false, Some(Outcome::Blocked))
            } else if b.raw.bridge_repair_hut.unwrap_or(false) {
                // spec: S-0209 claim 4 (row WP23: bridge repair hut blocked)
                (false, Some(Outcome::Blocked))
            } else {
                // spec: S-0209 claim 4 (row WP24: other enemy building, armed)
                (false, Some(Outcome::EnemyObstacle))
            }
        } else if o.kind == Kind::Terrain {
            // spec: S-0209 claim 4 (row WP27: a terrain object gives none; row WP22 excludes terrain objects)
            (false, None)
        } else if allied {
            match o.kind {
                Kind::Vehicle => {
                    let guarding = m.beh.seen() == Some(Behaviour::AreaGuard)
                        && m.guard_spot == Some(Target::Object(oid));
                    if guarding {
                        // spec: S-0209 claim 4 (row WP5: a vehicle the walker guards)
                        (false, Some(Outcome::Blocked))
                    } else if !moving_or_dest(w, oid) {
                        // spec: S-0209 claim 4 (row WP16: allied vehicle at rest)
                        (false, Some(Outcome::StationaryFriendly))
                    } else if crate::drive::between_steps_or_chain(w, oid) {
                        // spec: S-0209 claim 4 (row WP17: moving, between steps or at a chain point)
                        (false, Some(Outcome::MovingBlocker))
                    } else {
                        // spec: S-0209 claim 4 (row WP18: in the middle of a step)
                        (false, None)
                    }
                }
                // spec: S-0209 claim 4 (row WP19: allied aircraft blocked; WP20: allied infantry none)
                Kind::Aircraft => (false, Some(Outcome::Blocked)),
                _ => (false, None),
            }
        } else if crate::cloak::fully_cloaked(w, oid) {
            // spec: S-0209 claim 4 (row WP21: cloaked enemy object, armed walker or not; the cloak state alone, Terms *cloaked*)
            (false, Some(Outcome::CloakedBlocker))
        } else if !armed {
            // spec: S-0209 claim 4 (row WP22)
            (false, Some(Outcome::Blocked))
        } else if o.kind == Kind::Infantry && crate::cloak::appears_disguised(w, oid, m.owner) {
            // spec: S-0209 claim 4 (row WP25: enemy infantry that appears friendly to the walker's house through a disguise, armed walker) and S-0516 claim 35; a disguise with no disguise house is included)
            (false, Some(Outcome::StationaryFriendly))
        } else if o.kind == Kind::Infantry {
            // spec: S-0209 claim 4 (row WP26: other enemy infantry, armed: none)
            (false, None)
        } else {
            // spec: S-0209 claim 4 (row WP28: other enemy objects, armed)
            (false, Some(Outcome::EnemyObstacle))
        };
        if v.0 && override_value.is_none() {
            override_value = Some((values.len(), v.1.unwrap_or(Outcome::Clear)));
        }
        values.push(v);
    }
    // spec: S-0209 claim 4 (4a: the zero walker speed check applies outside the listed cases, also when the mission-target override WP3 decides)
    if let Some(u) = w.rules().unit(m.kind, m.type_index)
        && let Some(class) = u.class
        && !passability::enterable_by_class(r, w, c, Some(class), u.zone, false)
        && obj_layer == Layer::Ground
    {
        return Outcome::Blocked;
    }
    if let Some((k, value)) = override_value {
        // spec: S-0209 claim 4 (an override: blocked when an earlier value is blocked, else the override's value)
        let earlier_blocked = overlay_value == Some(Outcome::Blocked)
            || values[..k]
                .iter()
                .any(|v| !v.0 && v.1 == Some(Outcome::Blocked));
        // spec: S-0209 Edge cases (`quirks.occupancy_override_hides_later_blockers` off: any blocked occupant decides)
        let later_blocked = !r.quirks.occupancy_override_hides_later_blockers
            && values[k + 1..]
                .iter()
                .any(|v| !v.0 && v.1 == Some(Outcome::Blocked));
        return if earlier_blocked || later_blocked {
            Outcome::Blocked
        } else {
            value
        };
    }
    let cval = values
        .iter()
        .filter(|v| !v.0)
        .filter_map(|v| v.1)
        .chain(overlay_value)
        .max()
        .unwrap_or(Outcome::Clear);
    let Some(rec) = w.cells.occ(c, rec_layer) else {
        return Outcome::Blocked;
    };
    // 4d: the record table.
    if rec.vehicle_present() && !excused && cval == Outcome::Clear {
        // spec: S-0209 claim 4 (row WR1)
        return Outcome::MovingBlocker;
    }
    if let Some(h) = rec.house {
        let allied = w.allied(m.owner, Some(h));
        if !allied {
            // spec: S-0209 claim 4 (rows WR2 and WR3)
            return if armed {
                cval.max(Outcome::EnemyObstacle)
            } else {
                Outcome::Blocked
            };
        }
        if rec.full() && matches!(cval, Outcome::Clear | Outcome::CloakedBlocker) {
            let still = w
                .cells
                .list(c, obj_layer)
                .iter()
                .filter(|&&id| {
                    w.obj(id).is_some_and(|o| {
                        o.kind == Kind::Infantry
                            && w.allied(m.owner, o.owner)
                            && !in_progress(w, id)
                    })
                })
                .count();
            // spec: S-0209 claim 4 (rows WR4 and WR5: exactly three still allies, quirks.stationary_friendly_exactly_three)
            let stationary = if r.quirks.stationary_friendly_exactly_three {
                still == 3
            } else {
                still >= 3
            };
            return if stationary {
                Outcome::StationaryFriendly
            } else {
                Outcome::MovingBlocker
            };
        }
    } else if rec.full() && cval == Outcome::Clear {
        // spec: S-0209 claim 4 (row WR6)
        return Outcome::Blocked;
    }
    // spec: S-0209 claim 4 (row WR7)
    cval
}

/// The entry outcome of a ground-vehicle mover for cell `c` (S-0209 claim
/// 5).
pub fn vehicle_outcome(w: &World, mover: ObjectId, c: CellXY, reference: i32) -> Outcome {
    let Some(m) = w.obj(mover) else {
        return Outcome::Blocked;
    };
    let r = w.rules();
    let Some(u) = r.unit(m.kind, m.type_index) else {
        return Outcome::Blocked;
    };
    let (obj_layer, rec_layer) = layers_for(w, c, reference);
    let Some(cell) = w.cells.cell(c) else {
        return Outcome::Blocked;
    };
    let crusher = is_crusher(w, mover);
    let armed = u.primary.is_some() || u.train;
    let crushes_wall =
        |crushable: bool| (crushable && crusher) || u.zone == MovementZone::CrusherAll;
    let mut overlay_value = None;
    if cell.overlay != NO_OVERLAY
        && let Some(o) = r.overlays.get(usize::from(cell.overlay))
        && o.wall
    {
        let allied_wall = wall_allied(w, mover, c);
        overlay_value = if crushes_wall(o.crushable) {
            if allied_wall {
                // spec: S-0209 claim 5 (row VO2: a wall it crushes, allied owner: friendly obstacle)
                Some(Outcome::FriendlyObstacle)
            } else {
                // spec: S-0209 claim 5 (row VO3: a wall it crushes, other owner or none: none)
                None
            }
        } else if u.primary.is_none() {
            // spec: S-0209 claim 5 (row VO4)
            Some(Outcome::Blocked)
        } else if primary_warhead_flag(w, mover, true)
            || (o.wood && primary_warhead_flag(w, mover, false))
        {
            // spec: S-0209 claim 5 (row VO5: friendly obstacle for an allied owner, enemy obstacle for another)
            Some(if allied_wall {
                Outcome::FriendlyObstacle
            } else {
                Outcome::EnemyObstacle
            })
        } else {
            // spec: S-0209 claim 5 (row VO6)
            Some(Outcome::Blocked)
        };
    }
    let mut values: Vec<Option<Outcome>> = Vec::new();
    let mut crushable_occupant = false;
    let mut self_listed = false;
    let mut override_value = None;
    for &oid in w.cells.list(c, obj_layer) {
        let Some(o) = w.obj(oid) else { continue };
        if oid == mover {
            // spec: S-0209 claim 5 (row VP1: itself; excused from the vehicle-present flag)
            self_listed = true;
            continue;
        }
        let allied = w.allied(m.owner, o.owner);
        let v: Option<Outcome>;
        let mut over = false;
        if m.beh.seen() == Some(Behaviour::Enter)
            && m.dest == Some(Target::Object(oid))
            && o.kind == Kind::Vehicle
        {
            // spec: S-0209 claim 5 (row VP17: the vehicle it enters)
            over = true;
            v = Some(Outcome::Clear);
        } else if o.kind == Kind::Building
            && r.buildings[o.type_index as usize].absorb.vehicle
            && m.beh.seen() == Some(Behaviour::Enter)
            && m.dest == Some(Target::Object(oid))
            && !crate::econ::power::being_erased(w, oid)
        {
            // spec: S-0209 claim 5 (row VP5: a building that absorbs units, the vehicle's behaviour enter and its destination that building, unless being erased: overrides, clear)
            over = true;
            v = Some(Outcome::Clear);
        } else if o.kind == Kind::Building
            && r.buildings[o.type_index as usize]
                .raw
                .grinding
                .unwrap_or(false)
            && m.beh.seen() == Some(Behaviour::Eaten)
            && m.dest == Some(Target::Object(oid))
            && !crate::econ::power::being_erased(w, oid)
        {
            // spec: S-0209 claim 5 (row VP6: a building that grinds units, the vehicle's behaviour eaten and its destination that building, unless being erased: overrides, clear) and S-0603 claim 4
            over = true;
            v = Some(Outcome::Clear);
        } else if o.kind == Kind::Building
            && r.buildings[o.type_index as usize]
                .raw
                .grinding
                .unwrap_or(false)
            && w.cells.list(m.pos.cell(), Layer::Ground).contains(&oid)
        {
            // spec: S-0209 claim 5 (row VP7: a building that grinds units and is also listed in the cell the vehicle stands in: overrides, clear) and S-0603 claim 4
            over = true;
            v = Some(Outcome::Clear);
        } else if o.kind == Kind::Building {
            let b = &r.buildings[o.type_index as usize];
            if m.contact == Some(oid) && !blocking_cell(w, oid, c) {
                // spec: S-0209 claim 5 (row VP3: a building the vehicle is in contact with, on a cell that is not one of its blocking cells: none)
                v = None;
            } else if b.gate && crate::process::gate_open(w, oid) {
                // spec: S-0209 claim 5 (row VP8: an open gate gives none; open or closed by S-0215 claim 35)
                v = None;
            } else if b.gate && w.allied(o.owner, m.owner) {
                // spec: S-0209 claim 5 (row VP9: a closed gate allied with the vehicle, the gate owner's view)
                v = Some(Outcome::ClosedFriendlyGate);
            } else if b.gate {
                // spec: S-0209 claim 5 (rows VP10 and VP11: any other closed gate: enemy obstacle with a primary weapon, else blocked)
                v = Some(if armed {
                    Outcome::EnemyObstacle
                } else {
                    Outcome::Blocked
                });
            } else if b.invisible || b.raw.firestorm_wall.unwrap_or(false) {
                // spec: S-0209 claim 5 (rows VP13, VP15: invisible, firestorm off: none)
                v = None;
            } else if (b.raw.unit_repair.unwrap_or(false) || b.raw.bunker.unwrap_or(false))
                && !blocking_cell(w, oid, c)
            {
                // spec: S-0209 claim 5 (row VP12: repair or bunker building off its blocking cells)
                v = None;
            } else if b.raw.bib.unwrap_or(false) && c.x - o.pos.cell().x == o.foundation.0 - 1 {
                // spec: S-0209 claim 5 (row VP16: the last column along x of a building with a bib)
                v = None;
            } else if allied {
                // spec: S-0209 claim 5 (row VP19)
                v = Some(Outcome::Blocked);
            } else if crate::cloak::fully_cloaked(w, oid) {
                // spec: S-0209 claim 5 (row VP24: cloaked enemy object; the cloak state alone, Terms *cloaked*)
                v = Some(Outcome::CloakedBlocker);
            } else if crusher && crushable_by(w, mover, oid) {
                crushable_occupant = true;
                v = None;
            } else if !armed {
                // spec: S-0209 claim 5 (row VP26)
                v = Some(Outcome::Blocked);
            } else if b.raw.bridge_repair_hut.unwrap_or(false) {
                // spec: S-0209 claim 5 (row VP27)
                v = Some(Outcome::Blocked);
            } else {
                // spec: S-0209 claim 5 (row VP28)
                v = Some(Outcome::EnemyObstacle);
            }
        } else if o.kind == Kind::Terrain {
            // spec: S-0209 claim 5 (rows VP26 and VP29: unarmed blocked; the weapon against wood decides)
            v = Some(if !armed {
                Outcome::Blocked
            } else if primary_warhead_flag(w, mover, false) {
                Outcome::EnemyObstacle
            } else {
                Outcome::Blocked
            });
        } else if m.beh.seen() == Some(Behaviour::AreaGuard)
            && m.guard_spot == Some(Target::Object(oid))
        {
            // spec: S-0209 claim 5 (row VP18)
            v = Some(Outcome::Blocked);
        } else if allied {
            if !counts_as_moving(w, oid) {
                // spec: S-0209 claim 5 (row VP20)
                v = Some(Outcome::StationaryFriendly);
            } else if close_case(w, mover, oid) {
                // spec: S-0209 claim 5 (row VP21: the close case of S-0212 claim 27)
                v = Some(Outcome::Blocked);
            } else if (o.kind != Kind::Infantry && crate::drive::between_steps_or_chain(w, oid))
                || crate::drive::at_chain_point(w, oid)
            {
                // spec: S-0209 claim 5 (row VP22) and S-0212 claim 27 (a moving allied vehicle between steps, or any unit at a chain point: moving blocker)
                v = Some(Outcome::MovingBlocker);
            } else {
                // spec: S-0209 claim 5 (row VP23) and S-0212 claim 27 (any other moving allied unit, moving infantry among them, gives no outcome of its own)
                v = None;
            }
        } else if crate::cloak::fully_cloaked(w, oid) {
            // spec: S-0209 claim 5 (row VP24: cloaked enemy object; the cloak state alone, Terms *cloaked*)
            v = Some(Outcome::CloakedBlocker);
        } else if crusher && crushable_by(w, mover, oid) {
            // spec: S-0209 claim 5 (row VP25: a crushable occupant)
            crushable_occupant = true;
            v = None;
        } else if !armed {
            // spec: S-0209 claim 5 (row VP26)
            v = Some(Outcome::Blocked);
        } else {
            // spec: S-0209 claim 5 (row VP30)
            v = Some(Outcome::EnemyObstacle);
        }
        if over && override_value.is_none() {
            override_value = Some((values.len(), v.unwrap_or(Outcome::Clear)));
            values.push(None);
            continue;
        }
        values.push(v);
    }
    if let Some((k, value)) = override_value {
        // spec: S-0209 claim 5 (the override rule as in claim 4)
        let earlier_blocked = overlay_value == Some(Outcome::Blocked)
            || values[..k].contains(&Some(Outcome::Blocked));
        // spec: S-0209 Edge cases (`quirks.occupancy_override_hides_later_blockers` off)
        let later_blocked = !r.quirks.occupancy_override_hides_later_blockers
            && values[k + 1..].contains(&Some(Outcome::Blocked));
        return if earlier_blocked || later_blocked {
            Outcome::Blocked
        } else {
            value
        };
    }
    let cval = values
        .iter()
        .flatten()
        .copied()
        .chain(overlay_value)
        .max()
        .unwrap_or(Outcome::Clear);
    // spec: S-0209 claim 5 (5a: the zero speed check applies outside the listed cases)
    if obj_layer == Layer::Ground
        && !passability::enterable_by_class(r, w, c, u.class, u.zone, false)
    {
        return Outcome::Blocked;
    }
    if !passability::restricted_allows(w, c, u.restricted_to, obj_layer == Layer::Deck) {
        return Outcome::Blocked;
    }
    let Some(rec) = w.cells.occ(c, rec_layer) else {
        return Outcome::Blocked;
    };
    let flag = rec.vehicle_present() && !self_listed;
    // 5e: the record table.
    if cval != Outcome::Clear {
        // spec: S-0209 claim 5 (row VR1)
        return cval;
    }
    if crushable_occupant && flag {
        // spec: S-0209 claim 5 (row VR2: the first vehicle of the ground layer decides, quirks.crusher_entry_ground_layer)
        let layer = if r.quirks.crusher_entry_ground_layer {
            Layer::Ground
        } else {
            obj_layer
        };
        let first = w
            .cells
            .list(c, layer)
            .iter()
            .copied()
            .find(|&id| w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle));
        return match first {
            Some(v) if crushable_by(w, mover, v) => Outcome::Clear,
            _ => Outcome::MovingBlocker,
        };
    }
    if crushable_occupant {
        // spec: S-0209 claim 5 (row VR3)
        return Outcome::Clear;
    }
    let any_taken = rec.marks.iter().any(Option::is_some);
    if !any_taken && !flag {
        // spec: S-0209 claim 5 (row VR4)
        return Outcome::Clear;
    }
    if flag {
        // spec: S-0209 claim 5 (row VR5)
        return Outcome::MovingBlocker;
    }
    if rec.house.is_some_and(|h| w.allied(m.owner, Some(h))) {
        // spec: S-0209 claim 5 (row VR6)
        return Outcome::MovingBlocker;
    }
    if crusher {
        // spec: S-0209 claim 5 (row VR7)
        return Outcome::Clear;
    }
    if u.train {
        // spec: S-0209 claim 5 (row VR8)
        return Outcome::EnemyObstacle;
    }
    if u.primary.is_none() {
        // spec: S-0209 claim 5 (row VR9)
        return Outcome::Blocked;
    }
    // spec: S-0209 claim 5 (row VR10)
    if primary_hits_ground(w, mover) {
        Outcome::EnemyObstacle
    } else {
        Outcome::Blocked
    }
}

/// Whether `c` is one of the building's blocking cells (S-0209 Terms).
pub fn blocking_cell(w: &World, b: ObjectId, c: CellXY) -> bool {
    let Some(o) = w.obj(b) else { return false };
    let rows = w.rules().buildings[o.type_index as usize]
        .raw
        .number_impassable_rows;
    let tl = o.pos.cell();
    // spec: S-0209 Terms and S-0606 claim 7 (a bunker type in which a unit sits: every foundation cell blocks)
    if crate::structures::bunker::is_bunker(w, b) && crate::structures::bunker::occupied(w, b) {
        return true;
    }
    match rows {
        // spec: S-0209 Terms (no impassable-rows value: every foundation cell blocks)
        None => true,
        // spec: S-0209 Terms (cells fewer than k cells along x from the top-left cell)
        Some(k) => c.x - tl.x < k,
    }
}

/// The close case of S-0212 claim 27.
pub fn close_case(w: &World, mover: ObjectId, occupant: ObjectId) -> bool {
    let (Some(m), Some(o)) = (w.obj(mover), w.obj(occupant)) else {
        return false;
    };
    let t = w.tick;
    let q = w.rules().quirks;
    let fm = m.body.value(t);
    let fo = o.body.value(t);
    // spec: S-0212 claim 27 (the occupant's facing turned by 32767, or 32768 with quirks.opposite_facing_reversal off)
    let turn: u16 = if q.opposite_facing_reversal {
        32767
    } else {
        32768
    };
    let opposite = glasnost_num::geom::eight_direction(fo.wrapping_add(turn))
        == glasnost_num::geom::eight_direction(fm);
    let dx = i64::from(o.pos.x - m.pos.x);
    let dy = i64::from(o.pos.y - m.pos.y);
    let dz = i64::from(o.pos.z - m.pos.z);
    // spec: S-0212 claim 27 (spatial distance of F6c at most 511 leptons)
    let d = glasnost_num::geom::root_floor(
        (dx * dx + dy * dy + dz * dz) as u64,
        q.table_root_distance,
        w.rules().mode,
    );
    let ahead = glasnost_num::geom::eight_direction(glasnost_num::geom::direction(
        dx,
        dy,
        q.table_direction,
    )) == glasnost_num::geom::eight_direction(fm);
    opposite && d <= 511 && ahead // spec: S-0212 claim 27 (at most 511 leptons)
}

/// The free side positions of a cell layer's record, by S-0208's table O
/// row for a class (S-0208 claim 3).
pub fn first_free(w: &World, c: CellXY, l: Layer, row: &[u8]) -> Option<u8> {
    let rec = w.cells.occ(c, l)?;
    row.iter().copied().find(|&p| !rec.taken(p))
}

/// Whether a walker gets no standing position in a cell because of the
/// vehicle-present or object-present flag (S-0209 claim 3; S-0208 claim 3
/// first row).
pub fn no_position_by_flags(w: &World, c: CellXY, walker_layer: Layer) -> bool {
    let rec = w.cells.occ(c, walker_layer);
    let ground = w.cells.occ(c, Layer::Ground);
    // spec: S-0209 claim 3 (the object-present flag keeps walkers out unless an open gate stands in the cell)
    let open_gate = || {
        w.cells
            .list(c, Layer::Ground)
            .iter()
            .any(|&g| crate::process::gate_open(w, g))
    };
    rec.is_some_and(|r| r.vehicle_present())
        || (ground.is_some_and(|g| g.others > 0) && !open_gate())
}

/// The side positions (used by tests).
pub const SIDES: [u8; 3] = position::SIDES; // spec: S-0208 table P (the three side positions)
