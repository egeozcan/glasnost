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
/// Paralysis (S-0214) is a seam; being erased is S-0514's.
fn disabled(w: &World, id: ObjectId) -> bool {
    // spec: S-0516 Terms (EMP-disabled, paralysed, warping: being erased or a chrono warp)
    crate::econ::power::under_pulse(w, id) || crate::econ::power::being_erased(w, id)
}

/// Seized (S-0607 claims 5, 12 and 20).
fn seized(w: &World, id: ObjectId) -> bool {
    crate::lift::seized(w, id)
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
    // spec: S-0615 claim 14 (an object that begins to cloak is lost to its disc lasers)
    crate::disc::object_lost(w, id);
    sound(w, id);
    // spec: S-0506 claim 11 (r37: the start of cloaking loses the object as a controller target or pending target, whatever the spawner's house and whatever sensors cover it: no exception, unlike claim 14's drop below)
    crate::spawn::target_lost(w, id);
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
        // spec: S-0516 claim 18 (a cloaked building falls from level 15; an explicit cause on a cloaking building changes it to uncloaking at its current fade level, no jump to 15)
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
/// claim 15, S-0516 claim 2 (d)). Called by `crates` when a cloaking
/// crate is picked up.
pub fn grant_own_flag(w: &mut World, id: ObjectId) {
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.own = true;
    }
}

/// Every object that has `id` as its target keeps it only if it has the
/// same owner or its house senses the target (S-0516 claim 14, S-0215
/// claim 10: an overridden attacker reverts first, then drops it if it is
/// still its target); each one that drops it takes S-0309 claim 28's timer
/// reset first.
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
        // spec: S-0309 claim 28 (each cloak drop resets the holder's targeting timer as in claim 27; one kept under an exception takes no draw)
        crate::combat::held::timer_draw(w, x);
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
            // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
            crate::emitters::retarget_let_go(w, x);
            crate::combat::held::note_revert(w, x);
        }
        let Some(ob) = w.obj_mut(x) else { continue };
        if ob.target == Some(Target::Object(id)) {
            crate::spawn::clear_own_target(ob);
        }
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        // spec: S-0107 r46 claim 11, Changes of target (the cloak drop's loss of the target, or the earlier target put back when the override ends, S-0215 claim 10, ends an infantry holder's attack state and makes its request at once)
        crate::infantry::watch_test(w, x);
        if let Some(before) = restored {
            // spec: S-0107 claim 15 (a destination brought back is assigned: the destination-driven get-up)
            crate::infantry::destination_assigned(w, x, before);
        }
    }
}

/// The candidate distance of S-0516 claim 13 for a position (`x`, `y`) in
/// leptons: ⌊r(u² + w²)⌋ with u = x mod 256 and w = y mod 256, measured
/// from the top corner of the cell holding the position. Unlike S-0207
/// claim 15, `quirks.drive_nearest_from_top_corner` does not apply.
pub fn sensor_candidate_distance(w: &World, x: i32, y: i32) -> u64 {
    let r = w.rules();
    // spec: S-0516 claim 13 (u = x mod 256, w = y mod 256, residues from 0 to 255; height ignored)
    let (u, v) = (i64::from(x.rem_euclid(256)), i64::from(y.rem_euclid(256)));
    // spec: S-0516 claim 13 (⌊r(u² + w²)⌋ with S-0204 claim 5's table root, subject to quirks.table_root_distance)
    glasnost_num::geom::root_floor((u * u + v * v) as u64, r.quirks.table_root_distance, r.mode)
}

/// The one candidate of a cell layer for the sensor tests of S-0516 claims
/// 13 and 18: among infantry, vehicles, aircraft and buildings, the least
/// [`sensor_candidate_distance`] of each one's own position, ties to the
/// first in the cell-layer order (S-0209 claim 13).
pub fn nearest_top_corner(w: &World, c: CellXY, layer: Layer) -> Option<ObjectId> {
    let mut best: Option<(u64, ObjectId)> = None;
    for &x in w.cells.list(c, layer) {
        let Some(o) = w.obj(x) else { continue };
        // spec: S-0516 claim 13 (candidates are infantry, vehicles, aircraft and buildings)
        if !matches!(
            o.kind,
            Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
        ) {
            continue;
        }
        // spec: S-0516 claim 13 (a building uses its position within the cell containing that position)
        let d = sensor_candidate_distance(w, o.pos.x, o.pos.y);
        // spec: S-0516 claim 13 (the least integer distance; ties to the first in the cell-layer order)
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

/// The object's ordinary cloak update (S-0516 claim 42, S-0203): cell
/// entry, a generator's ring and a building's coverage, and the state
/// machine. A vehicle, infantry unit or aircraft takes it after its due
/// behaviour processing and before its ordinary locomotion and separate
/// firing (`tick::object_turn`); a building after its behaviour-fired
/// shots (`econ::events::turn`).
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
}

/// The cloak rules late in the object's own turn, after its movement
/// (`econ::events::turn`): an aircraft's hidden target (claim 25) and
/// disguise when still (claim 33).
pub fn late_turn(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if !o.on_map || o.kind == Kind::Terrain {
        return;
    }
    aircraft_target(w, id);
    super::disguise::turn(w, id);
}

/// The cell-arrival update of a vehicle, infantry unit or aircraft (S-0516
/// Terms, claims 13 and 22): a mobile sensor's disc moves to its current
/// cell, even the cell it was already centred on, then a fully cloaked
/// mover takes the adjacent sensor test. Its sources, by claim 22's table:
/// the driving and ship locomotors on reaching a track target
/// (`drive::spend`), the walking one at its arrival on a walking target
/// (`walk::tick`), the hovering one at each non-refusing heading-point
/// choice (`hover`), the teleporting one in the jump tick
/// (`teleport::jump`), a superweapon warp member at its arrival-effects
/// update (`superweapons::warp`), and a completed fall landing (`fall`).
/// Ordinary flight and rocket flight never supply one. A jumpjet's
/// touchdown supplies one (S-0502 claim 15, `jump::fly`); take-off,
/// in-flight cell crossings, cruising arrival and hovering supply none. An
/// underground mover's emergence, mechanical walking and drop-pod descent
/// are seams (those locomotors are not built). Entering play alone, unloading included, is not one: the disc is
/// added by [`entered_map`] with no adjacency test.
pub fn cell_movement_completed(w: &mut World, id: ObjectId) {
    // spec: S-0405 claim 13 (revision 20: a powered-unit vehicle is disabled when it completes its arrival in a cell, which covers driving, landing after a fall and arriving by a chrono jump; placement and creation are not arrivals; revision 47, Q-1710: for a hovering vehicle the arrivals are the cell-arrival updates of its non-refusing heading-point choices, read on the cell it stands in)
    crate::econ::power::arrival(w, id);
    let Some(o) = w.obj(id) else { return };
    if !o.on_map || !o.kind.is_mobile() {
        return;
    }
    let c = centre_cell(w, id);
    // spec: S-0516 claim 22 (at a cell-arrival update: remove around the last recorded coverage cell, add around the object's cell and record it, even when it equals the old cell)
    super::sensor::mobile_moved(w, id);
    // spec: S-0516 claims 13 and 22 (the same event supplies the adjacency check of a fully cloaked mover, sensor or not)
    adjacent_reveal(w, id, c);
}

/// An aircraft's position crossed into another cell in ordinary flight
/// (`air`): with `quirks.flying_sensor_disc_stationary` nothing happens;
/// off, its sensor disc moves to the new cell, with no adjacency test.
pub fn flight_cell_crossed(w: &mut World, id: ObjectId) {
    if w.rules().quirks.flying_sensor_disc_stationary {
        // spec: S-0516 claim 22 and Edge cases (quirks.flying_sensor_disc_stationary: ordinary flying movement never relocates a sensor disc, even on landing)
        return;
    }
    // spec: S-0516 Edge cases (flying_sensor_disc_stationary off: a horizontal cell-boundary crossing relocates the disc to the new current cell; coverage only, no claim 13 adjacency uncloak)
    super::sensor::mobile_moved(w, id);
}

/// An object entered cell `c` (S-0516 claims 6, 13 and 22).
fn entered_cell(w: &mut World, id: ObjectId, c: CellXY) {
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.cell = Some(c);
    }
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
            let mut full = false;
            // spec: S-0516 claim 18 (a cloaking building rises one level per tick; level 15 is fully cloaked)
            if since >= 1
                && let Some(ob) = w.obj_mut(id)
            {
                ob.cloak.stage += 1;
                ob.cloak.stepped = tick;
                // spec: S-0516 claim 18 (fade level 15)
                if ob.cloak.stage >= 15 {
                    ob.cloak.stage = 15; // spec: S-0516 claim 18 (fade level 15)
                    ob.cloak.state = CloakState::Cloaked;
                    full = true;
                }
            }
            if full {
                // spec: S-0516 claim 18 and S-0616 claim 1 (at level 15 the natural smoke system is removed at once with its particles)
                crate::emitters::natural_smoke_cloaked(w, id);
            }
        }
        CloakState::Cloaking => {
            // spec: S-0516 claims 7 and 43, quirks.cloak_zero_speed_stall (one step every S ticks; a speed of 0 cloaks one stage per tick; a negative speed's countdown is always expired: one step per update)
            let per = speed(w, id).max(1);
            // spec: S-0516 claim 42 (each ordinary cloak update advances at most one stage; the step, when due, precedes the abort and completion tests)
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
                    // spec: S-0516 claim 42 (no abort draw on the step reaching the fully cloaked threshold)
                    return;
                }
            }
            let k = w.obj(id).map_or(0, |o| o.cloak.stage);
            let x = fade_fraction(k, n);
            // spec: S-0516 claims 8 and 42 (the resulting stage in the 50 % band 64 ≤ x < 128 at health at or below red: one draw R(0, 99), below 10 aborts, silently; also on an update with no step)
            if (64..128).contains(&x) && !above_red(w, id) {
                let half = w.rules().quirks.ranged_draw_half_span;
                let r = w.rng.ranged(0, 99, half);
                if r < 10 {
                    uncloak(w, id, false);
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
                if s == 0 && w.rules().quirks.cloak_zero_speed_stall && o.cloak.stage > 0 {
                    // spec: S-0516 Edge cases (quirks.cloak_zero_speed_stall: a speed of 0 never advances uncloaking from a positive stage; at stage 0, N = 1, it completes at the next ordinary update, claim 43)
                    return;
                }
                // spec: S-0516 claim 43 (a negative speed's countdown is always expired: one stage per ordinary update, not a stall)
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
                } else {
                    // spec: S-0516 claim 18 and S-0616 claim 1 (at level 0 a new natural smoke system, only with a displacement other than (0, 0, 0))
                    crate::emitters::natural_smoke(w, id, true);
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
    if to.cloak.state == CloakState::Cloaked
        && !w.allied(owner, to.owner)
        && !senses(w, owner, t)
        && let Some(ob) = w.obj_mut(id)
    {
        crate::spawn::clear_own_target(ob);
        ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
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

/// A crusher actually crushed an object (S-0516 claim 11 (c), S-0210 claim
/// 4). Overlays, walls, empty passage and a thief's take-over do not call
/// this.
pub fn crushed(w: &mut World, crusher: ObjectId) {
    // spec: S-0516 claim 11 (c) (a vehicle actually crushes at least one object)
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
    // spec: S-0516 claim 16 (only a vehicle with its own cloak flag placed outside the usable area is forced fully cloaked, the usable-area result tested afresh at each placement; ordinary placement and transport exits keep the state and stage)
    let forced = o.kind == Kind::Vehicle && o.cloak.own && !w.cells.usable(c);
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.cell = Some(c);
        if forced {
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
    // spec: S-0516 claim 20 (removal, sale, destruction: an immediate collapse)
    super::field::vanish(w, id);
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
    // spec: S-0516 claim 20 (capture: an immediate collapse with the restart test, then growth for the new owner)
    super::field::captured(w, id);
    // spec: S-0516 claim 34 (a change of owner keeps the disguise)
}
