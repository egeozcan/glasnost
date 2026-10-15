//! The nuclear strike (S-0513 claims 7 to 13; S-0524 claim 3).
//!
//! A silo launch records the target and, on the silo's next update, fires
//! the upward missile ([`silo_turn`]). When the upward missile bursts with
//! a missile-maker warhead, `combat::projectile` hands it to [`burst`],
//! which creates the falling missile. A one-time record creates the
//! falling missile at once ([`one_time`]). The nuclear detonation itself
//! is S-0306 claim 25's.
//!
//! Seam: radiation fields (S-0525) are not built; [`radiation_seam`] is
//! called with the hand-off of claim 12.

use glasnost_map::CellXY;
use glasnost_num::F64;

use super::{SwEvent, SwKind};
use crate::combat::data::WeaponId;
use crate::combat::state::Projectile;
use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, ObjectId, Pos, Target, World};

/// The upward missile's top speed (Constants).
// spec: S-0513 Constants (upward missile top speed 255 leptons per tick)
pub const UP_TOP_SPEED: i32 = 255;
/// The upward missile's start speed (Constants).
// spec: S-0513 Constants (upward missile start speed 10 leptons per tick)
pub const UP_START_SPEED: i64 = 10;
/// The silo-launched falling missile's start speed (Constants).
// spec: S-0513 Constants (falling missile start speed, silo: 1 lepton per tick)
pub const FALL_START_SPEED: i64 = 1;
/// The one-time missile's start height above the surface (Constants, F2).
// spec: S-0513 Constants and F2 (one-time missile start height 20 000 leptons above the surface)
pub const ONE_TIME_HEIGHT: i32 = 20_000;
/// The one-time missile's start speed (Constants, F2).
// spec: S-0513 Constants and F2 (one-time missile start speed 100 leptons per tick, straight down)
pub const ONE_TIME_SPEED: i64 = 100;
/// The waypoint strike's stored speed (S-0524 claim 3).
// spec: S-0524 claim 3 (the engine constant 50 leptons per tick, not the weapon's speed)
pub const WAYPOINT_SPEED: i32 = 50;

/// Whether a building is selling (its behaviour as seen).
fn behaviour_seen_selling(o: &crate::world::Object) -> bool {
    o.beh.seen() == Some(Behaviour::Selling)
}

/// The silo types a launch of type `i` considers, in type-list order
/// (claim 7).
fn silo_types(w: &World, i: u32) -> Vec<u32> {
    let r = w.rules();
    let sw = &r.superweapons;
    // spec: S-0513 claim 7 (a silo type naming this superweapon type in its own first or second field, in type-list order)
    let all = (0..sw.silo.len())
        .filter(|&b| sw.silo[b] && sw.building_sw[b].contains(&Some(i)))
        .map(|b| b as u32);
    if r.quirks.nuke_silo_first_type_only {
        // spec: S-0513 claim 7 (quirks.nuke_silo_first_type_only: the first such type is the only one considered, owned or not)
        all.take(1).collect()
    } else {
        // spec: S-0513 claim 7 (quirk off: every such silo type, in type-list order)
        all.collect()
    }
}

/// The silo a launch of type `i` uses (claim 7): of the house's on-map
/// buildings of the considered type, the one it has owned longest.
pub fn silo_choice(w: &World, h: HouseId, i: u32) -> Option<ObjectId> {
    let acquired = &w.houses.get(usize::from(h.0))?.econ.acquired;
    for t in silo_types(w, i) {
        // spec: S-0513 claim 7 (the one the house has owned longest: the house's acquisition order, S-0404 claim 2; distance, health, power and selling play no part)
        let found = acquired.iter().copied().find(|&id| {
            w.obj(id).is_some_and(|o| {
                o.kind == Kind::Building && o.type_index == t && o.on_map && o.owner == Some(h)
            })
        });
        if found.is_some() {
            return found;
        }
    }
    None
}

/// Whether a silo accepts the launch behaviour (claim 7): a selling silo
/// refuses it, and so does one whose accepted launch has not left yet
/// (Q-0735 interim for the busy silo).
pub fn accepts(w: &World, s: ObjectId) -> bool {
    let Some(o) = w.obj(s) else { return false };
    // spec: S-0513 claim 7 (a selling silo refuses the launch behaviour)
    if behaviour_seen_selling(o) {
        return false;
    }
    // spec: S-0513 claim 7 (one already in its launch sequence with nothing queued need not start a second; Q-0735 interim: busy until its upward missile leaves)
    o.sw.silo_launch.is_none()
}

/// A projectile of `weapon` placed now (its first turn is in this tick,
/// S-0306 claim 8).
#[allow(clippy::too_many_arguments)]
fn missile(
    w: &mut World,
    weapon: WeaponId,
    owner: Option<ObjectId>,
    house: Option<HouseId>,
    pos: Pos,
    vel: [F64; 3], // nocite: three components x, y, z
    target: CellXY,
    top_speed: i32,
) -> ObjectId {
    let rules = w.rules_arc();
    let wr = rules.combat.weapon(weapon);
    let arm = wr
        .projectile
        .map_or(0, |p| rules.combat.projectiles[p as usize].arm);
    let id = w.new_id();
    let tc = Pos::centre(target, 0);
    let stored = Pos::new(tc.x, tc.y, w.surface_at(tc.x, tc.y));
    let pr = Projectile {
        id,
        weapon,
        attacker: owner,
        house,
        firer: owner,
        firer_house: house,
        pos,
        vel,
        target: Some(Target::Cell(target)),
        stored,
        placed_at: pos,
        arm,
        placed: w.tick,
        damage: wr.damage,
        omega: crate::combat::projectile::d3(w, pos, stored),
        top_speed,
        turns: 0,
        unlocked: 0,
        locked: false,
        closing: F64::ZERO,
        prev_cell: pos.cell(),
        fire_point: pos,
        nuke_wait: None,
        vehicle_scalable: false,
    };
    w.projectiles.insert(id, pr);
    // spec: S-0306 claim 8 (the projectile enters play; its first turn is in the same tick)
    w.update.enter(id);
    id
}

/// The warning marker at the target cell's centre (claim 11). Its
/// visibility and its end with its missile are presentation; the sim only
/// creates it (S-0202 claim 10's creation draws).
fn marker(w: &mut World, cell: CellXY) {
    if w.rules().animation(super::WARNING_MARKER).is_none() {
        // spec: S-0513 Edge cases (a missing marker animation type: no animation)
        return;
    }
    let p = super::cell_point(w, cell, false);
    // spec: S-0513 claim 11 (the warning marker at the target cell's centre)
    crate::combat::effects::anim(w, super::WARNING_MARKER, p);
}

/// The launch announcement and the siren at the target (claims 5 and 8).
fn sounds(w: &mut World, h: Option<HouseId>, i: Option<u32>, cell: CellXY) {
    let p = super::cell_point(w, cell, false);
    // spec: S-0513 claim 5 (nuclear strike: the launch announcement for every player and the siren at the target point)
    super::event(w, SwEvent::Launch, h, i, None);
    super::event(w, SwEvent::Siren, h, i, Some(p));
}

/// A one-time strike (claim 10): the falling missile at once, no owner.
pub fn one_time(w: &mut World, h: Option<HouseId>, i: u32, cell: CellXY) {
    sounds(w, h, Some(i), cell);
    let Some(payload) = w.rules().superweapons.payload else {
        // spec: S-0513 Edge cases (a missing payload weapon: no projectile; the importer warns)
        return;
    };
    let mut p = super::cell_point(w, cell, true);
    // spec: S-0513 F2 (z₀ = h(c) + b + 20 000 leptons)
    p.z += ONE_TIME_HEIGHT;
    let mode = w.rules().mode;
    let speed = w.rules().combat.weapon(payload).speed;
    // spec: S-0513 claim 10 and F2 (straight down at 100 leptons per tick; no owner, the payload weapon's damage, warhead and speed)
    let vel = [F64::ZERO, F64::ZERO, F64::from_i64(-ONE_TIME_SPEED, mode)];
    missile(w, payload, None, None, p, vel, cell, speed);
    // spec: S-0513 claim 10 (the warning marker is attached to the falling missile)
    marker(w, cell);
}

/// A silo launch (claims 7 and 8). `silo` is the chosen silo, if any.
pub fn silo_launch(w: &mut World, h: HouseId, i: u32, cell: CellXY, silo: Option<ObjectId>) {
    let Some(s) = silo else {
        // spec: S-0513 claim 7 (with no matching building, neither missile nor launch sounds)
        return;
    };
    // spec: S-0513 claims 7 and 8 (the target is recorded for the house, a later launch replacing it, and the sounds play, whether or not the silo accepts)
    super::house_mut(w, h).nuke_target = Some(cell);
    sounds(w, Some(h), Some(i), cell);
    if !accepts(w, s) {
        // spec: S-0513 claim 7 (a selling or busy silo: no additional missile)
        return;
    }
    if let Some(o) = w.obj_mut(s) {
        // spec: S-0513 claim 8 (on the silo's next update the upward missile leaves)
        o.sw.silo_launch = Some((Some(cell), i));
    }
}

/// The silo's next update after accepting a launch (claims 8 and 13).
pub fn silo_turn(w: &mut World, s: ObjectId) {
    let Some(o) = w.obj(s) else { return };
    let Some((recorded, i)) = o.sw.silo_launch else {
        return;
    };
    let owner = o.owner;
    let selling = behaviour_seen_selling(o);
    if let Some(ob) = w.obj_mut(s) {
        ob.sw.silo_launch = None;
    }
    if selling || !w.obj(s).is_some_and(|o| o.on_map && o.health > 0) {
        // spec: S-0513 claim 13 (sold or destroyed before its next update: no missile leaves)
        return;
    }
    let rules = w.rules_arc();
    let target = if rules.quirks.silo_target_of_current_owner {
        // spec: S-0513 claim 13 (quirks.silo_target_of_current_owner: the target last recorded for its current house, or none)
        owner.and_then(|h| {
            w.sw.houses
                .get(usize::from(h.0))
                .and_then(|x| x.nuke_target)
        })
    } else {
        // spec: S-0513 claim 13 (quirk off: the target recorded when it accepted)
        recorded
    };
    let typ = if rules.quirks.superweapon_kind_as_list_position {
        // spec: S-0513 Edge cases (quirks.superweapon_kind_as_list_position: the type at the list position equal to the nuclear-strike kind number)
        SwKind::Missile.number()
    } else {
        // spec: S-0513 Edge cases (quirk off: the fired type)
        i
    };
    let Some(weapon) = rules
        .superweapons
        .types
        .get(typ as usize)
        .and_then(|t| t.weapon)
    else {
        // spec: S-0513 Edge cases (a missing silo weapon: no projectile; the importer warns)
        w.note("S-0513 Edge cases: the silo's superweapon type has no weapon; no missile");
        return;
    };
    let Some(target) = target else {
        // spec: S-0513 claim 13 (no valid target: our engine launches nothing; the original's launch at no target is unsafe)
        w.note(
            "S-0513 claim 13: a captured silo whose house never launched has no target; no missile",
        );
        return;
    };
    // spec: S-0513 claim 8 (from the silo's first firing position, S-0313)
    let from = crate::combat::position::fire_point(w, s, 0, 0);
    let mode = rules.mode;
    // spec: S-0513 claim 8 (straight up at 10 leptons per tick, top speed 255, owned by the silo, the weapon's damage and warhead)
    let vel = [F64::ZERO, F64::ZERO, F64::from_i64(UP_START_SPEED, mode)];
    missile(w, weapon, Some(s), owner, from, vel, target, UP_TOP_SPEED);
    if let Some(a) = rules.superweapons.nuke_take_off.clone() {
        // spec: S-0513 claim 8 (the silo's take-off animation at the firing position)
        crate::combat::effects::anim(w, &a, from);
    }
    // spec: S-0513 claim 11 (in a silo launch the marker is attached to the upward missile, from the silo's first update)
    marker(w, target);
}

/// The upward missile's burst with a missile-maker warhead (claims 8 and
/// 9): no damage; the falling missile is created over the target. Called
/// by `combat::projectile` in place of the explosion.
pub fn burst(w: &mut World, up: &Projectile) {
    let rules = w.rules_arc();
    let Some(payload) = rules.superweapons.payload else {
        // spec: S-0513 Edge cases (a missing payload weapon: no projectile)
        return;
    };
    let Some(Target::Cell(cell)) = up.target else {
        return;
    };
    let alt = rules.combat.weapon(up.weapon).projectile.map_or(0, |p| {
        rules.combat.projectiles[p as usize].detonation_altitude
    });
    let mut p = super::cell_point(w, cell, false);
    // spec: S-0513 F8 (z₀ = h(c) + A_up, the upward projectile type's detonation altitude, no bridge lift)
    p.z += alt;
    // spec: S-0513 claim 9 (owned by the upward missile's owner: the silo, or no object if it left play; its current house is credited, claim 13)
    let owner = up.attacker.filter(|&s| w.obj(s).is_some());
    let house = owner.and_then(|s| w.obj(s)).and_then(|o| o.owner);
    let mode = rules.mode;
    let speed = rules.combat.weapon(payload).speed;
    // spec: S-0513 claim 9 (straight down at 1 lepton per tick, the payload weapon's damage, warhead and speed)
    let vel = [F64::ZERO, F64::ZERO, F64::from_i64(-FALL_START_SPEED, mode)];
    missile(w, payload, owner, house, p, vel, cell, speed);
}

/// The nuclear strike at a waypoint (S-0524 claim 3): no firer and no
/// owning house, speed 50, the table-trigonometry downward vector.
pub fn waypoint(w: &mut World, cell: CellXY) {
    let Some(payload) = w.rules().superweapons.payload else {
        // spec: S-0524 claim 3 (no weapon of the fixed name: no effect; the importer warns)
        return;
    };
    let mut p = super::cell_point(w, cell, true);
    // spec: S-0524 claim 3 and Formulas (start = surface-centre point + bridge lift + (0, 0, 20 000))
    p.z += ONE_TIME_HEIGHT;
    let mode = w.rules().mode;
    // spec: S-0524 Formulas (p = 1.5707483884323126 radians, table sine and cosine of S-0105 F1)
    let angle = crate::rules::nearest_double("1.5707483884323126");
    let s = glasnost_num::trig::sin_t(angle, mode);
    let c = glasnost_num::trig::cos_t(angle, mode);
    let m = F64::from_i64(-100, mode); // spec: S-0524 Formulas (the factor −100)
    // spec: S-0524 Formulas (Vx = RZ(RZ(C × C) × −100), Vy = RZ(RZ(S × C) × −100), Vz = RZ(S × −100))
    let vel = [
        c.mul(c, mode).mul(m, mode),
        s.mul(c, mode).mul(m, mode),
        s.mul(m, mode),
    ];
    missile(w, payload, None, None, p, vel, cell, WAYPOINT_SPEED);
}

/// Seam (S-0525 radiation fields, not built): a detonation of a weapon with
/// a positive radiation level leaves radiation at the detonation cell
/// with that level and the warhead's cell spread truncated to whole cells
/// (claim 12).
pub fn radiation_seam(w: &mut World, weapon: WeaponId, cell: CellXY) {
    let level = w.rules().combat.weapon(weapon).rad_level;
    if level <= 0 {
        // spec: S-0525 Import mapping (0 or less leaves no radiation)
        return;
    }
    let _ = cell;
    w.note("S-0513 claim 12: radiation fields (S-0525) are not built; the radiation hand-off does nothing");
}
