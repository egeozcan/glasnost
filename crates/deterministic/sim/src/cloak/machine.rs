//! The cloak state machine (S-0516 claims 1 to 18, 25, 41) and the event
//! hooks that keep the coverage in step with objects entering, leaving and
//! changing owner.

use glasnost_map::CellXY;
use glasnost_num::F64;

use super::{CloakState, centre_cell, cloaked_stage, fade_fraction, in_own_field, senses};
use crate::combat::check::Answer;
use crate::combat::data::Ability;
use crate::combat::effects::{Effect, Sound};
use crate::combat::veterancy::{has_ability, type_combat};
use crate::rules::Kind;
use crate::world::{HouseId, Layer, ObjectId, Target, World};

/// A presentation event of the cloak machine (S-0516 claim 26): the app
/// prunes its selection on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloakEvent {
    /// The object started cloaking: it leaves every non-owner's selection.
    Started,
    /// A fully cloaked object was re-evaluated (claim 6): it leaves the
    /// selection of a non-owner that does not sense it.
    Reevaluated,
}

/// The number of stages *N* (S-0516 claim 1).
fn stages(w: &World) -> i32 {
    w.rules().cloak.stages
}

/// *S*, the type's cloaking speed (S-0516 claim 1).
fn speed(w: &World, id: ObjectId) -> i32 {
    // spec: S-0516 Import mapping (CloakingSpeed absent: 7)
    type_combat(w, id).map_or(7, |t| t.cloaking_speed)
}

/// The own cloak flag (S-0516 claim 2 (a) and (d)).
fn own(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.cloak.own)
}

/// Whether the object can cloak by itself (S-0516 Terms, claim 3): its own
/// flag, not barred by cloak-stop while its locomotor moves.
fn self_cloak(w: &World, id: ObjectId) -> bool {
    if !own(w, id) {
        return false;
    }
    let stop = type_combat(w, id).is_some_and(|t| t.cloak_stop);
    // spec: S-0516 claim 3 (cloak-stop: the own flag lets it start only while its locomotor is not moving)
    !(stop && crate::occupancy::in_progress(w, id))
}

/// The disable tests (S-0516 Terms): EMP-disabled, paralysed or warping.
/// Paralysis (S-0214) and the chrono warp (S-0514) are seams.
fn disabled(w: &World, id: ObjectId) -> bool {
    // spec: S-0516 Terms (EMP-disabled, paralysed, warping: being erased or a chrono warp)
    crate::econ::power::under_pulse(w, id) || crate::econ::power::being_erased(w, id)
}

/// Seized (S-0607 claims 5, 12 and 20). Seam: the seizing specs are not
/// built.
fn seized(w: &World, id: ObjectId) -> bool {
    let _ = (w, id);
    false
}

/// A cloaking veterancy ability (S-0516 claim 2 (b), S-0311 claim 3).
fn ability(w: &World, id: ObjectId) -> bool {
    has_ability(w, id, Ability::Cloak)
}

/// Whether the object may cloak (S-0516 claim 2).
pub fn may_cloak(w: &World, id: ObjectId) -> bool {
    // spec: S-0516 claim 2 (own flag, veterancy ability, own cloak field; a crate sets the own flag)
    own(w, id) || ability(w, id) || in_own_field(w, id)
}

/// Whether the object is ready to cloak (S-0516 claim 4, and claim 18's
/// sensor ring for a building).
pub fn ready(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if !may_cloak(w, id) {
        return false;
    }
    let tick = w.tick;
    let building = o.kind == Kind::Building;
    // spec: S-0516 claim 4 (fully cloaked; rearm countdown; a target in range; a non-building at a stage other than 0; the cloak delay; seized; height ≥ 1 lepton)
    if o.cloak.state == CloakState::Cloaked
        || tick < o.combat.rearm_until
        || (!building && o.cloak.stage != 0)
        || tick < o.cloak.delay_until
        || seized(w, id)
        || w.height_above_surface(id) >= 1
    {
        return false;
    }
    if let Some(t) = o.target {
        let weapon = crate::combat::weapon::current_weapon(w, id);
        if weapon.is_some() && crate::combat::range::in_range(w, id, weapon, Some(t), None) {
            return false;
        }
    }
    // spec: S-0516 claim 18 (a building is not ready while a foreign sensor unit stands in its foundation or ring)
    !(building && sensor_ring(w, id))
}

/// F4: whether the health ratio *h* / *h*ₘₐₓ is strictly above the red
/// ratio (double precision).
fn above_red(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let s = crate::combat::damage::strength(w, id);
    if s <= 0 {
        return false;
    }
    let mode = w.rules().mode;
    // spec: S-0516 F4 (h / hₘₐₓ in double precision, compared with ρ; strict for the start)
    let h = F64::from_i64(i64::from(o.health), mode).div(F64::from_i64(i64::from(s), mode), mode);
    h.gt(w.rules().general.condition_red)
}

/// Whether the object is in contact with a war-factory building (S-0516
/// claim 5 (b)).
fn war_factory_contact(w: &World, id: ObjectId) -> bool {
    let Some(c) = w.obj(id).and_then(|o| o.contact) else {
        return false;
    };
    w.obj(c).is_some_and(|b| {
        b.kind == Kind::Building
            && w.rules()
                .buildings
                .get(b.type_index as usize)
                .is_some_and(|t| t.raw.weapons_factory.unwrap_or(false))
    })
}

/// Pushes the cloak sound (S-0516 claim 15).
fn sound(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let pos = o.pos;
    // spec: S-0516 claim 15 (the game-wide cloak sound at the object)
    w.effects.push(Effect::Sound {
        sound: Sound::Cloak,
        of: None,
        house: None,
        pos: Some(pos),
    });
}

/// Starts cloaking (S-0516 claims 5, 6, 14, 15 and 26).
fn start(w: &mut World, id: ObjectId) {
    let tick = w.tick;
    let Some(ob) = w.obj_mut(id) else { return };
    let building = ob.kind == Kind::Building;
    ob.cloak.state = CloakState::Cloaking;
    if !building {
        // spec: S-0516 claim 7 (cloaking starts at stage 0)
        ob.cloak.stage = 0;
    }
    ob.cloak.stepped = tick;
    sound(w, id);
    // spec: S-0516 claim 14 (the start of cloaking: attackers drop the target)
    drop_attackers(w, id);
    // spec: S-0516 claim 26 (an object that starts cloaking leaves non-owners' selections)
    w.effects.push(Effect::Cloak {
        id,
        event: CloakEvent::Started,
    });
}

/// Starts uncloaking (S-0516 claims 7, 11, 15, 18): a no-op for an object
/// that is uncloaked or already uncloaking. `with_sound`: the cause plays
/// the cloak sound (claim 15; the abort of claim 8 does not). Returns
/// whether an uncloak started.
///
/// This is the call of every uncloak cause, the seams included: C4 planting
/// (S-0519), EMP and other special-weapon effects (S-0518, S-0525,
/// S-06xx).
pub fn uncloak(w: &mut World, id: ObjectId, with_sound: bool) -> bool {
    let n = w.obj(id).map(|_| stages(w)).unwrap_or(0);
    let full = w.rules().quirks.uncloak_from_cloaking_full_duration;
    let tick = w.tick;
    let Some(ob) = w.obj_mut(id) else {
        return false;
    };
    let from = ob.cloak.state;
    if matches!(from, CloakState::Uncloaked | CloakState::Uncloaking) {
        // spec: S-0516 claims 10 and 15 (an uncloaking object completes its uncloak; a repeated cause plays no sound)
        return false;
    }
    if ob.kind == Kind::Building {
        // spec: S-0516 claim 18 (a cloaked building falls from level 15); Q-0726 (interim: a cloaking building hit reverses at its current level)
        if from == CloakState::Cloaked {
            ob.cloak.stage = 15; // spec: S-0516 claim 18 (fade levels 0 to 15)
        }
    } else if from == CloakState::Cloaked || full {
        // spec: S-0516 claim 7 and quirks.uncloak_from_cloaking_full_duration (uncloaking always starts at N − 1)
        ob.cloak.stage = (n - 1).max(0);
    }
    // quirks.uncloak_from_cloaking_full_duration off: a cloaking object keeps its current stage.
    ob.cloak.state = CloakState::Uncloaking;
    ob.cloak.stepped = tick;
    if with_sound {
        sound(w, id);
    }
    true
}

/// Gives an object its own cloak flag: the cloaking crate's effect (S-0520
/// claim 15, S-0516 claim 2 (d)). Seam: crates are not built; this is the
/// call they make.
pub fn grant_own_flag(w: &mut World, id: ObjectId) {
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.own = true;
    }
}

/// Every object that has `id` as its target keeps it only if it has the
/// same owner or its house senses the target (S-0516 claim 14, S-0215
/// claim 10: an overridden attacker reverts first, then drops it if it is
/// still its target).
fn drop_attackers(w: &mut World, id: ObjectId) {
    let Some(owner) = w.obj(id).map(|o| o.owner) else {
        return;
    };
    let ids: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.target == Some(Target::Object(id)))
        .map(|o| o.id)
        .collect();
    for x in ids {
        let xo = w.obj(x).map(|o| o.owner).unwrap_or(None);
        // spec: S-0516 claim 14 (kept: the same owner, or the attacker's house senses the cloaking object's cell)
        if xo == owner || senses(w, xo, id) {
            continue;
        }
        let Some(ob) = w.obj_mut(x) else { continue };
        let mut restored = None;
        if ob.beh.suspended.is_some()
            && let Some((t, d)) = ob.beh.revert()
        {
            crate::spawn::set_own_target(ob, t);
            if ob.kind.is_mobile() {
                restored = Some(ob.dest);
                ob.dest = d;
            }
        }
        if ob.target == Some(Target::Object(id)) {
            crate::spawn::clear_own_target(ob);
        }
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        if let Some(before) = restored {
            // spec: S-0107 claim 15 (a destination brought back is assigned: the destination-driven get-up)
            crate::infantry::destination_assigned(w, x, before);
        }
    }
}

/// The object of a cell layer nearest the cell's top corner (S-0516 claims
/// 13 and 18). Q-0722 (interim: the smallest squared distance from the
/// object's position to (256 x, 256 y), the earlier-listed object on a
/// tie).
pub fn nearest_top_corner(w: &World, c: CellXY, layer: Layer) -> Option<ObjectId> {
    // spec: S-0009 Terms (256 leptons per cell: the top corner of cell (x, y) is (256 x, 256 y))
    let (cx, cy) = (i64::from(c.x) * 256, i64::from(c.y) * 256);
    let mut best: Option<(i64, ObjectId)> = None;
    for &x in w.cells.list(c, layer) {
        let Some(o) = w.obj(x) else { continue };
        let (dx, dy) = (i64::from(o.pos.x) - cx, i64::from(o.pos.y) - cy);
        let d = dx * dx + dy * dy;
        if best.is_none_or(|(b, _)| d < b) {
            best = Some((d, x));
        }
    }
    best.map(|(_, x)| x)
}

/// Whether a unit counts as a sensor by its type flag or, when `by_ability`,
/// its sensors veterancy ability (S-0516 claims 13 and 18).
fn is_sensor(w: &World, id: ObjectId, by_ability: bool) -> bool {
    type_combat(w, id).is_some_and(|t| t.sensors)
        || (by_ability && has_ability(w, id, Ability::Sensors))
}

/// The building sensor ring (S-0516 claim 18): a foreign sensor unit in the
/// foundation or the 1-cell ring around it.
fn sensor_ring(w: &World, id: ObjectId) -> bool {
    let Some(b) = w.obj(id) else { return false };
    let owner = b.owner;
    let tl = b.pos.cell();
    let (fw, fh) = b.foundation;
    let owner_only = w.rules().quirks.building_sensor_ring_owner_only;
    // spec: S-0516 claim 18 (the foundation and the 1-cell ring around it)
    for dy in -1..=fh {
        for dx in -1..=fw {
            let c = CellXY::new(tl.x + dx, tl.y + dy);
            let mut cand = nearest_top_corner(w, c, Layer::Ground);
            if cand == Some(id) {
                // spec: S-0516 claim 18 (the building itself as candidate: the first infantry occupant instead)
                cand = w
                    .cells
                    .list(c, Layer::Ground)
                    .iter()
                    .copied()
                    .find(|&x| w.obj(x).is_some_and(|o| o.kind == Kind::Infantry));
            }
            let Some(x) = cand else { continue };
            let Some(xo) = w.obj(x) else { continue };
            // spec: S-0516 claim 18 (a unit: a vehicle, infantry unit or aircraft)
            if !xo.kind.is_mobile() {
                continue;
            }
            let hit = if owner_only {
                // spec: S-0516 claim 18 and quirks.building_sensor_ring_owner_only (the type flag, an owner other than the building's)
                is_sensor(w, x, false) && xo.owner != owner
            } else {
                // spec: S-0516 Edge cases (building_sensor_ring_owner_only off: flag or ability, not allied)
                is_sensor(w, x, true) && !w.allied(owner, xo.owner)
            };
            if hit {
                return true;
            }
        }
    }
    false
}

/// The adjacent-sensor reveal of a fully cloaked mobile object entering
/// cell `c` (S-0516 claim 13).
fn adjacent_reveal(w: &mut World, id: ObjectId, c: CellXY) {
    let Some(o) = w.obj(id) else { return };
    if !o.kind.is_mobile() || o.cloak.state != CloakState::Cloaked {
        return;
    }
    let owner = o.owner;
    // spec: S-0516 claim 13 (the 8 neighbouring cells that pass the usable-area test)
    for (dx, dy) in [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ] {
        let n = CellXY::new(c.x + dx, c.y + dy);
        if !w.cells.usable(n) {
            continue;
        }
        // spec: S-0516 claim 13 (one candidate per cell: the ground-layer object nearest the top corner)
        let Some(x) = nearest_top_corner(w, n, Layer::Ground) else {
            continue;
        };
        let xo = w.obj(x).and_then(|o| o.owner);
        // spec: S-0516 claim 13 (not allied with the mover's owner, with the sensors flag or a sensors ability)
        if !w.allied(xo, owner) && is_sensor(w, x, true) {
            uncloak(w, id, true);
            return;
        }
    }
}

/// Re-evaluates an object (S-0516 claim 6): an uncloaked object (or an
/// uncloaking building, claim 18) inside its owner's field that is ready
/// starts cloaking; a fully cloaked one is deselected by non-owners that do
/// not sense it (claim 26).
pub fn reevaluate(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if !o.on_map || !(o.kind.is_mobile() || o.kind == Kind::Building) {
        return;
    }
    match o.cloak.state {
        CloakState::Uncloaked => {
            // spec: S-0516 claim 6 (in the owner's field and ready: starts at once, no health test, no draw, no disable gate, no war-factory test)
            if in_own_field(w, id) && ready(w, id) {
                start(w, id);
            }
        }
        CloakState::Uncloaking if o.kind == Kind::Building => {
            // spec: S-0516 claim 18 (an uncloaking building re-evaluated inside its own field and ready reverses, keeping its fade level)
            if in_own_field(w, id) && ready(w, id) {
                start(w, id);
            }
        }
        CloakState::Cloaked => {
            // spec: S-0516 claims 6 and 26 (re-evaluation deselects a fully cloaked object)
            w.effects.push(Effect::Cloak {
                id,
                event: CloakEvent::Reevaluated,
            });
        }
        _ => {}
    }
}

/// Re-evaluates every object of a cell (both layers).
pub fn reevaluate_cell(w: &mut World, c: CellXY, mobile_ground_only: bool) {
    let mut ids: Vec<ObjectId> = w.cells.list(c, Layer::Ground).to_vec();
    if !mobile_ground_only {
        ids.extend_from_slice(w.cells.list(c, Layer::Deck));
    }
    for x in ids {
        let mobile = w.obj(x).is_some_and(|o| o.kind.is_mobile());
        if mobile_ground_only && !mobile {
            continue;
        }
        reevaluate(w, x);
    }
}

/// The object's cloak turn (S-0516), in its own turn after its movement
/// (S-0203): cell entry, a generator's ring and a building's coverage, the
/// state machine, an aircraft's hidden target (claim 25) and disguise when
/// still (claim 33).
pub fn turn(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if !o.on_map || o.kind == Kind::Terrain {
        return;
    }
    let c = centre_cell(w, id);
    if o.cloak.cell != Some(c) {
        entered_cell(w, id, c);
    }
    if w.obj(id).is_some_and(|o| o.kind == Kind::Building) {
        super::field::turn(w, id);
        super::sensor::building_turn(w, id);
    }
    step(w, id);
    aircraft_target(w, id);
    super::disguise::turn(w, id);
}

/// An object entered cell `c` (S-0516 claims 6, 13 and 22).
fn entered_cell(w: &mut World, id: ObjectId, c: CellXY) {
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.cell = Some(c);
    }
    // spec: S-0516 claim 22 (a mobile sensor that changes cell moves its disc); Q-0721 (interim: at its turn)
    super::sensor::mobile_moved(w, id);
    // spec: S-0516 claim 13 (a fully cloaked mover entering a cell)
    adjacent_reveal(w, id, c);
    // spec: S-0516 claim 6 (re-evaluated when it enters a cell)
    reevaluate(w, id);
}

/// The state machine step of the object's turn (S-0516 claims 5 to 9, 18).
fn step(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let building = o.kind == Kind::Building;
    let tick = w.tick;
    let since = tick - o.cloak.stepped;
    let n = stages(w);
    match o.cloak.state {
        CloakState::Uncloaked => try_start(w, id),
        CloakState::Cloaking if building => {
            // spec: S-0516 claim 18 (a cloaking building rises one level per tick; level 15 is fully cloaked)
            if since >= 1
                && let Some(ob) = w.obj_mut(id)
            {
                ob.cloak.stage += 1;
                ob.cloak.stepped = tick;
                if ob.cloak.stage >= 15 {
                    ob.cloak.stage = 15; // spec: S-0516 claim 18 (fade level 15)
                    ob.cloak.state = CloakState::Cloaked;
                }
            }
        }
        CloakState::Cloaking => {
            let k = o.cloak.stage;
            let x = fade_fraction(k, n);
            // spec: S-0516 claim 8 (the 50 % band 64 ≤ x < 128 at health at or below red: one draw R(0, 99), below 10 aborts, silently)
            if (64..128).contains(&x) && !above_red(w, id) {
                let half = w.rules().quirks.ranged_draw_half_span;
                let r = w.rng.ranged(0, 99, half);
                if r < 10 {
                    uncloak(w, id, false);
                    return;
                }
            }
            // spec: S-0516 claim 7 and quirks.cloak_zero_speed_stall (one step every S ticks; S ≤ 0 cloaks one stage per tick; Q-0725 interim for S < 0)
            let per = speed(w, id).max(1);
            if since >= per {
                let mut done = false;
                if let Some(ob) = w.obj_mut(id) {
                    ob.cloak.stage += 1;
                    ob.cloak.stepped = tick;
                    // spec: S-0516 claim 7 (fully cloaked when the stage first reaches c = ⌈N / 2⌉; the stage returns to 0)
                    if ob.cloak.stage >= cloaked_stage(n) {
                        ob.cloak.state = CloakState::Cloaked;
                        ob.cloak.stage = 0;
                        done = true;
                    }
                }
                if done {
                    // spec: S-0516 claim 14 (full cloak of a vehicle, infantry unit or aircraft: attackers drop the target again)
                    drop_attackers(w, id);
                }
            }
        }
        CloakState::Cloaked => {
            // spec: S-0516 claim 9 (kept by (a) the own flag and no disable, (b) an ability, (c) the owner's field)
            let keep = (own(w, id) && !disabled(w, id)) || ability(w, id) || in_own_field(w, id);
            // spec: S-0516 claim 18 (a cloaked building uncloaks while a foreign sensor unit stands in its ring)
            let ring = building && sensor_ring(w, id);
            if !keep || ring {
                uncloak(w, id, true);
            }
        }
        CloakState::Uncloaking => {
            let per = if building {
                1 // spec: S-0516 claim 18 (an uncloaking building falls one level per tick)
            } else {
                let s = speed(w, id);
                if s <= 0 && w.rules().quirks.cloak_zero_speed_stall {
                    // spec: S-0516 Edge cases (quirks.cloak_zero_speed_stall: a speed of 0 never advances uncloaking; Q-0725 interim for S < 0)
                    return;
                }
                s.max(1)
            };
            if since < per && o.cloak.stage > 0 {
                return;
            }
            let delay = w.rules().cloak.delay;
            let Some(ob) = w.obj_mut(id) else { return };
            if ob.cloak.stage > 0 && since >= per {
                ob.cloak.stage -= 1;
                ob.cloak.stepped = tick;
            }
            // spec: S-0516 claim 7 (the uncloak completes when the stage reaches 0; the stage never goes below 0)
            if ob.cloak.stage <= 0 {
                ob.cloak.stage = 0;
                ob.cloak.state = CloakState::Uncloaked;
                if !building {
                    // spec: S-0516 claim 7 (the cloak delay D starts when the uncloak completes); claim 18 (not for a building)
                    ob.cloak.delay_until = tick.saturating_add(delay);
                }
            }
        }
    }
}

/// Claims 5 and 6 in the object's own turn.
fn try_start(w: &mut World, id: ObjectId) {
    if !may_cloak(w, id) {
        return;
    }
    // spec: S-0516 claim 6 (each tick in the owner's field: ready starts at once)
    if in_own_field(w, id) {
        if ready(w, id) {
            start(w, id);
        }
        return;
    }
    // spec: S-0516 claim 5 (a) (self-cloaking and not disabled, or a cloaking ability)
    let a = (self_cloak(w, id) && !disabled(w, id)) || ability(w, id);
    // spec: S-0516 claim 5 (b) and (c) (no war-factory contact; ready)
    if !a || war_factory_contact(w, id) || !ready(w, id) {
        return;
    }
    let building = w.obj(id).is_some_and(|o| o.kind == Kind::Building);
    // spec: S-0516 claim 5 (d) (above red starts at once; otherwise one R(0, 99) below 4); claim 18 (no health test for a building)
    if !building && !above_red(w, id) {
        let half = w.rules().quirks.ranged_draw_half_span;
        if w.rng.ranged(0, 99, half) >= 4 {
            return;
        }
    }
    start(w, id);
}

/// An aircraft drops a target that is fully cloaked, not allied with it and
/// not sensed by its house (S-0516 claim 25).
fn aircraft_target(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Aircraft {
        return;
    }
    let Some(Target::Object(t)) = o.target else {
        return;
    };
    let owner = o.owner;
    let Some(to) = w.obj(t) else { return };
    // spec: S-0516 claim 25 (fully cloaked, not allied with the aircraft, not sensed by its house)
    if to.cloak.state == CloakState::Cloaked && !w.allied(owner, to.owner) && !senses(w, owner, t) {
        if let Some(ob) = w.obj_mut(id) {
            crate::spawn::clear_own_target(ob);
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        }
    }
}

/// The firing reply (S-0516 claim 11 (a)): a firer that tries to fire and
/// gets *cloaked* starts uncloaking; a vehicle only when its target is in
/// range of the weapon.
pub fn firing_answer(w: &mut World, id: ObjectId, slot: usize, target: Target, a: Answer) {
    if a != Answer::Cloaked {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    if o.kind == Kind::Vehicle {
        let weapon = crate::combat::weapon::slot_weapon(w, id, slot);
        // spec: S-0516 claim 11 (a) (vehicles within range of their target)
        if !crate::combat::range::in_range(w, id, weapon, Some(target), None) {
            return;
        }
    }
    // spec: S-0516 claim 11 (a) (firing with a weapon that gets the answer cloaked)
    uncloak(w, id, true);
}

/// Whether the firing check answers *cloaked* (S-0301 claim 12): a weapon
/// that needs the firer uncloaked, the firer in any state but uncloaked;
/// an aircraft only while fully cloaked.
pub fn needs_uncloak(w: &World, firer: ObjectId, decloak_to_fire: bool) -> bool {
    let Some(o) = w.obj(firer) else { return false };
    if !decloak_to_fire {
        return false;
    }
    // spec: S-0301 claim 12 (cloaking, cloaked or uncloaking; an aircraft only while fully cloaked)
    if o.kind == Kind::Aircraft {
        o.cloak.state == CloakState::Cloaked
    } else {
        o.cloak.state != CloakState::Uncloaked
    }
}

/// The hit reaction (S-0516 claim 12): a surviving hit that reaches the
/// damage-state result uncloaks; `lost` is the health it took.
pub fn hit_taken(w: &mut World, id: ObjectId, result: crate::combat::damage::HitResult, lost: i32) {
    use crate::combat::damage::HitResult as R;
    let Some(o) = w.obj(id) else { return };
    if o.health <= 0 {
        return;
    }
    if !matches!(
        result,
        R::Unaffected | R::Unchanged | R::NowYellow | R::NowRed
    ) {
        return;
    }
    // spec: S-0516 Edge cases (quirks.heal_reveals_cloaked off: only hits that lower health uncloak)
    if !w.rules().quirks.heal_reveals_cloaked && lost <= 0 {
        return;
    }
    // spec: S-0516 claim 12 (unaffected, unchanged, now yellow or now red on a surviving cloaked or cloaking object)
    uncloak(w, id, true);
}

/// Every cloaked object in the ground layer of a blocking cell, of any
/// house, starts uncloaking (S-0516 claim 11 (e), S-0209 claim 9).
pub fn blocked_reveal(w: &mut World, c: CellXY) {
    let ids: Vec<ObjectId> = w.cells.list(c, Layer::Ground).to_vec();
    for x in ids {
        // spec: S-0516 claim 11 (e) (every cloaked object of the ground layer, of any house)
        if w.obj(x)
            .is_some_and(|o| o.cloak.state == CloakState::Cloaked)
        {
            uncloak(w, x, true);
        }
    }
}

/// A crusher crushed something (S-0516 claim 11 (c)). Q-0723 (interim:
/// every crush).
pub fn crushed(w: &mut World, crusher: ObjectId) {
    // spec: S-0516 claim 11 (c) (a vehicle crushes something)
    if w.obj(crusher).is_some_and(|o| o.kind == Kind::Vehicle) {
        uncloak(w, crusher, true);
    }
}

/// Creation (S-0516 claims 2 (a) and 32): a vehicle or infantry unit takes
/// its type's cloak flag; a permanent disguiser is disguised.
pub fn created(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let ground_only = w.rules().quirks.cloak_flag_ground_mobile_only;
    let flag = type_combat(w, id).is_some_and(|t| t.cloakable);
    // spec: S-0516 claim 2 (a) and quirks.cloak_flag_ground_mobile_only (vehicles and infantry take the type's flag; off: every kind)
    let takes = matches!(o.kind, Kind::Vehicle | Kind::Infantry)
        || (!ground_only && matches!(o.kind, Kind::Aircraft | Kind::Building));
    if takes
        && flag
        && let Some(ob) = w.obj_mut(id)
    {
        ob.cloak.own = true;
    }
    super::disguise::created(w, id);
}

/// The object entered play (S-0516 claims 16, 22, 23, 40).
pub fn entered_map(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind == Kind::Terrain {
        return;
    }
    let c = centre_cell(w, id);
    let vehicle_own = o.kind == Kind::Vehicle && o.cloak.own;
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.cell = Some(c);
        if vehicle_own {
            // spec: S-0516 claim 16 (a vehicle with its own cloak flag enters play fully cloaked); Q-0727 (interim: also after unloading)
            ob.cloak.state = CloakState::Cloaked;
            ob.cloak.stage = 0;
        }
    }
    // spec: S-0516 claim 22 (a mobile sensor's disc is added when it enters play)
    super::sensor::mobile_entered(w, id);
    super::sensor::building_placed(w, id);
    super::field::placed(w, id);
}

/// The object is about to leave play (S-0516 claims 20, 22, 23, 40).
pub fn leaving_map(w: &mut World, id: ObjectId) {
    // spec: S-0516 claim 22 (the disc is removed around its current cell)
    super::sensor::mobile_left(w, id);
    super::sensor::building_removed(w, id);
    // spec: S-0516 claim 20 (removal: the field vanishes at once)
    super::field::vanish(w, id, true);
}

/// The object changed owner from `old` (S-0516 claims 20, 22, 23, 34).
pub fn owner_changed(w: &mut World, id: ObjectId, old: Option<HouseId>) {
    let Some(o) = w.obj(id) else { return };
    if old == o.owner {
        return;
    }
    // spec: S-0516 claim 22 (the disc moves from the old owner to the new one)
    super::sensor::mobile_owner_changed(w, id);
    super::sensor::building_owner_changed(w, id);
    // spec: S-0516 claim 20 (capture: the field vanishes at once and grows again for the new owner)
    super::field::captured(w, id);
    // spec: S-0516 claim 34 (a change of owner keeps the disguise)
}
