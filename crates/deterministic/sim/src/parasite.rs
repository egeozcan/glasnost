//! Parasites (S-0614): latching weapons, the terror drone's and the squid's attachment to a host, release and exit.
//!
//! Every object that takes part keeps its links in its lazily created
//! [`ParasiteState`]: a parasite records its carrier projectile, the
//! object it fired at, its host and its suppression timer; a mobile unit
//! records the parasite attached to it, its hold timer and its latch
//! block. A game without parasites never creates the state.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::F64;
use glasnost_num::geom::{Facing, eight_direction, n_step};
use glasnost_rules::sim::LocomotionClass;
use serde::{Deserialize, Serialize};

use crate::combat::damage::Hit;
use crate::combat::data::WeaponId;
use crate::combat::veterancy::{self, Rank};
use crate::rules::Kind;
use crate::world::{Layer, Object, ObjectId, Pos, Target, World};

/// Latch block length, ticks.
const LATCH_BLOCK: i32 = 20; // spec: S-0614 Constants (latch block length 20 ticks)
/// Height limit for a latching shot, leptons.
const HEIGHT_LIMIT: i32 = 208; // spec: S-0614 Constants (2 levels, 208 leptons)
/// Suppression set by curtain protection, unloading or consumption, ticks.
const EXIT_SUPPRESSION: i32 = 50; // spec: S-0614 claim 12 and Constants (50 ticks, set by curtain protection, unloading or consumption)
const WARP_SUPPRESSION: i32 = 500; // spec: S-0513 Constants (warp parasite suppression 500 ticks, claim 34)
/// The grab phase ends, and the first push starts, this many ticks after its first tick.
const GRAB_END: i32 = 29; // spec: S-0614 F3 (grab of 10 frames × 3 ticks, its first frame counted in its first tick: the first push starts at g + 29, so that the first strike is at g + 69)
/// A push lasts this many ticks; the strike follows in the next tick.
const PUSH: i32 = 40; // spec: S-0614 F3 and Constants (push of 10 frames × 4 ticks; strike period 40)
/// Splash effects per strike.
const SPLASHES: i32 = 3; // spec: S-0614 Constants (3 splash effects per strike)
/// Wakes per push start.
const WAKES: i32 = 3; // spec: S-0614 claim 10 (three wakes at every push start)
/// One quarter turn less one unit, 16-bit facing units.
const QUARTER: u16 = 16383; // spec: S-0614 F4 (q = 16383)
/// The hold after coming out of a living host, in rates of fire.
const EXIT_HOLD_RATES: i32 = 3; // spec: S-0614 claim 13 and F1 (held for 3 × R ticks)
/// The number of tiles of the water set.
const WATER_TILES: i32 = 14; // spec: S-0614 claim 2 (the 14 tiles starting at the theater's water-set role)

/// An object's parasite links (S-0614 Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParasiteState {
    // --- As a parasite ---
    /// The host it is attached to (claim 3).
    pub host: Option<ObjectId>,
    /// The projectile it travels in (claim 1).
    pub carrier: Option<ObjectId>,
    /// The object its latching shot was fired at; `None` for a cell (claim 1).
    pub latch_target: Option<ObjectId>,
    /// The cell it last stood in before firing (claim 4).
    pub fired_cell: Option<CellXY>,
    /// Its cell layer there.
    pub fired_layer: Layer,
    /// Target and destination kept because it had attack-move memory when
    /// it fired (claims 4, 13 and 15: not cleared with that memory).
    pub kept: Option<(Option<Target>, Option<Target>)>,
    /// Whether its host has taken its first turn since the attachment
    /// (claim 3: the first delivery; F3: the grab's first tick).
    pub started: bool,
    /// The tick of its next delivery (a drone) or next grappling event.
    pub next: i32,
    /// A grappling parasite still in its grab phase (table G).
    pub grabbing: bool,
    /// Suppressed while the tick is below this (S-0315 claim 6).
    pub suppressed_until: Option<i32>,
    /// A grappling parasite's grab animation in play, from its creation in
    /// the grab's first turn until its removal or end (claim 25).
    #[serde(default)]
    pub grab_anim: Option<ObjectId>,
    /// The tick the grappling cycle's current phase started (table G).
    #[serde(default)]
    pub phase_start: i32,
    /// The current push's way draw (claim 10); `None` in the grab.
    #[serde(default)]
    pub push_way: Option<i32>,
    // --- As a mobile unit ---
    /// The parasite attached to it (claim 2: at most one).
    pub attached: Option<ObjectId>,
    /// Held while the tick is below this (Terms, *hold*).
    pub hold_until: Option<i32>,
    /// No latching shot may be fired at it while the tick is below this (claim 5).
    pub latch_block_until: Option<i32>,
}

fn state(w: &World, id: ObjectId) -> Option<&ParasiteState> {
    w.obj(id).and_then(|o| o.parasite.as_ref())
}

fn state_mut(w: &mut World, id: ObjectId) -> Option<&mut ParasiteState> {
    w.obj_mut(id)
        .map(|o| o.parasite.get_or_insert_with(Default::default))
}

/// Whether a mobile unit is held (Terms, *hold*; claim 8).
pub fn held(w: &World, id: ObjectId) -> bool {
    let t = w.tick;
    state(w, id)
        .and_then(|s| s.hold_until)
        .is_some_and(|u| t < u)
}

/// The parasite attached to a host.
pub fn parasite_of(w: &World, host: ObjectId) -> Option<ObjectId> {
    state(w, host).and_then(|s| s.attached)
}

/// The host a parasite is attached to.
pub fn host_of(w: &World, parasite: ObjectId) -> Option<ObjectId> {
    state(w, parasite).and_then(|s| s.host)
}

/// Whether a parasite is suppressed now (S-0315 claim 6).
pub fn suppressed(w: &World, parasite: ObjectId) -> bool {
    let t = w.tick;
    state(w, parasite)
        .and_then(|s| s.suppressed_until)
        .is_some_and(|u| t < u)
}

/// Whether a weapon is a latching weapon (Terms): its warhead has the
/// parasite flag and the weapon the launch-inside flag.
pub fn latching(w: &World, weapon: WeaponId) -> bool {
    let r = w.rules();
    r.combat.weapon(weapon).limbo_launch && r.combat.warhead_of(weapon).is_some_and(|h| h.parasite)
}

fn type_naval(w: &World, id: ObjectId) -> bool {
    veterancy::type_combat(w, id).is_some_and(|tc| tc.naval)
}

/// A naval parasite (S-0315 Terms): a parasite of a naval type.
pub fn naval(w: &World, id: ObjectId) -> bool {
    type_naval(w, id)
}

/// A grappling parasite (Terms): its type is both naval and organic.
pub fn grappling(w: &World, id: ObjectId) -> bool {
    veterancy::type_combat(w, id).is_some_and(|tc| tc.naval && tc.organic)
}

/// Whether a cell's tile is one of the theater's water tiles (claim 2;
/// the test of S-0306's level projectile).
pub fn water_tile(w: &World, c: CellXY) -> bool {
    let water = w.catalogue.as_deref().map_or(i32::MIN, |c| c.roles.water);
    let tile = w.cells.cell(c).map_or(-1, |c| c.tile);
    water >= 0 && (water..water + WATER_TILES).contains(&tile)
}

/// Whether `target` is latchable for `parasite` (claim 2).
pub fn latchable(w: &World, parasite: ObjectId, target: ObjectId) -> bool {
    let Some(t) = w.obj(target) else {
        return false;
    };
    // spec: S-0614 claim 2 (a mobile unit on the map, not destroyed, health different from 0)
    if !t.kind.is_mobile() || !t.on_map || t.health == 0 || crate::combat::damage::dead(t) {
        return false;
    }
    // spec: S-0614 claim 2 (no parasite attached: a host carries at most one)
    if parasite_of(w, target).is_some() {
        return false;
    }
    // spec: S-0614 claim 2 (the can-carry-a-parasite flag)
    if !w
        .rules()
        .unit(t.kind, t.type_index)
        .is_some_and(|u| u.vehicle.parasiteable)
    {
        return false;
    }
    // spec: S-0614 claim 2 and S-0606 claim 9 (it is not bunkered in a tank bunker)
    if crate::structures::bunker::bunkered(w, target) {
        return false;
    }
    // spec: S-0614 claim 2 (a naval parasite: the target's cell is a water tile)
    if naval(w, parasite) && !water_tile(w, t.pos.cell()) {
        return false;
    }
    true
}

/// The parasite firing tests of claim 5 for a latching weapon fired by
/// `firer` at the mobile unit `target`: whether the shot is illegal.
pub fn shot_illegal(w: &World, firer: ObjectId, target: ObjectId) -> bool {
    let (Some(f), Some(t)) = (w.obj(firer), w.obj(target)) else {
        return true;
    };
    let now = w.tick;
    // spec: S-0614 claim 5 (not latchable; before the latch block; a height difference above 2 levels either way)
    !latchable(w, firer, target)
        || state(w, target)
            .and_then(|s| s.latch_block_until)
            .is_some_and(|u| now < u)
        || (i64::from(f.pos.z) - i64::from(t.pos.z)).abs() > i64::from(HEIGHT_LIMIT)
}

/// Whether a held firer's organic type makes its shots illegal (claim 8).
pub fn held_organic(w: &World, firer: ObjectId) -> bool {
    w.obj(firer).is_some_and(|o| o.kind.is_mobile())
        && held(w, firer)
        && veterancy::type_combat(w, firer).is_some_and(|tc| tc.organic)
}

/// Whether a held mobile firer's spawner weapon is refused (claim 8,
/// S-0506 claim 10: *impossible*).
pub fn held_spawner(w: &World, firer: ObjectId, weapon: WeaponId) -> bool {
    w.rules().combat.weapon(weapon).spawner
        && w.obj(firer).is_some_and(|o| o.kind.is_mobile())
        && held(w, firer)
}

/// A latching weapon has fired (claim 1): the firer leaves the map and
/// travels in projectile `carrier`; a mobile target's latch block is set.
pub fn fired(w: &mut World, firer: ObjectId, carrier: ObjectId, target: Target) {
    let now = w.tick;
    let Some(o) = w.obj(firer).cloned() else {
        return;
    };
    let target_obj = match target {
        Target::Object(x) => Some(x),
        Target::Cell(_) => None,
    };
    if let Some(x) = target_obj
        && w.obj(x).is_some_and(|t| t.kind.is_mobile())
        && let Some(s) = state_mut(w, x)
    {
        // spec: S-0614 claim 1 (a mobile target's latch block: the current tick plus 20)
        s.latch_block_until = Some(now.wrapping_add(LATCH_BLOCK));
    }
    if let Some(s) = state_mut(w, firer) {
        s.carrier = Some(carrier);
        s.latch_target = target_obj;
        // spec: S-0614 claim 4 (the cell it last stood in before firing)
        s.fired_cell = Some(o.pos.cell());
        s.fired_layer = o.layer;
        s.kept = o.attack_move.is_some().then_some((o.target, o.dest));
    }
    // spec: S-0614 claim 1 (it leaves the map at once and travels inside the projectile)
    crate::objects::take_inside(w, firer);
}

/// A projectile ends (claim 1): `detonated` when it detonated; the
/// parasite inside it, if any, latches on or fails to latch. A projectile
/// removed without a detonation strands it (claim 21).
pub fn carrier_ended(w: &mut World, carrier: ObjectId, weapon: WeaponId, detonated: bool) {
    if !w
        .rules()
        .combat
        .warhead_of(weapon)
        .is_some_and(|h| h.parasite)
    {
        return;
    }
    let Some(p) = w
        .objects
        .values()
        .find(|o| {
            o.parasite
                .as_ref()
                .is_some_and(|s| s.carrier == Some(carrier))
        })
        .map(|o| o.id)
    else {
        return;
    };
    let target = state_mut(w, p).and_then(|s| {
        s.carrier = None;
        s.latch_target.take()
    });
    if !detonated && w.rules().quirks.parasite_carrier_loss_strands_unit {
        // spec: S-0614 claim 21 (removed without detonating: the parasite stays off the map, unattached and unable to act; no latch, return, death or recovery)
        return;
    }
    let special =
        crate::combat::projectile::special_of(w, weapon, target.map(crate::world::Target::Object));
    // spec: S-0614 claim 24 (answers Q-1032: takeover, bomb planting or electric charging selected under S-0514 claim 20 at a detonation leaves the carried parasite off the map, unattached and unable to act, whatever the target's latchability and whether or not the higher effect succeeds; no return, recovery timer, death, experience or replacement); quirks.parasite_suppressed_latch_strands_unit off: the surviving parasite attempts claim 4's return, and never attaches
    if detonated
        && matches!(
            special,
            Some(
                crate::combat::projectile::Special::Takeover
                    | crate::combat::projectile::Special::Bomb
                    | crate::combat::projectile::Special::Charge
            )
        )
    {
        if w.rules().quirks.parasite_suppressed_latch_strands_unit {
            return;
        }
        fail_latch(w, p);
        return;
    }
    // spec: S-0514 claim 20 and S-0614 claims 1 and 3 (attachment is the fourth row: attached only when it is the selected special hit)
    let selected = special == Some(crate::combat::projectile::Special::Parasite);
    match target {
        // spec: S-0614 claim 3 (the projectile's target object is latchable: it becomes the host; no distance is tested)
        Some(t) if detonated && selected && latchable(w, p, t) => attach(w, p, t),
        // spec: S-0614 claim 4 (not latchable, or a cell: the parasite fails to latch); claim 21 (quirks.parasite_carrier_loss_strands_unit off: removal without detonation attempts the same return)
        _ => fail_latch(w, p),
    }
}

fn attach(w: &mut World, p: ObjectId, host: ObjectId) {
    if let Some(s) = state_mut(w, p) {
        s.host = Some(host);
        // spec: S-0614 claim 3 (the first delivery at the host's next turn)
        s.started = false;
        s.grabbing = false;
    }
    if let Some(s) = state_mut(w, host) {
        s.attached = Some(p);
    }
    let pos = w.obj(host).map(|o| o.pos);
    if let (Some(pos), Some(o)) = (pos, w.obj_mut(p)) {
        // spec: S-0614 claim 3 (it travels with the host)
        o.pos = pos;
        // spec: S-0614 claim 3 (it keeps no target while attached)
        crate::spawn::clear_own_target(o);
        o.dest = None;
    }
}

/// Restores the target and destination kept by attack-move memory (claims
/// 4, 13, 15: cleared unless it has that memory).
fn restore_kept(w: &mut World, p: ObjectId) {
    let kept = state_mut(w, p).and_then(|s| s.kept.take());
    if let Some(o) = w.obj_mut(p) {
        if o.attack_move.is_some()
            && let Some((t, d)) = kept
        {
            crate::spawn::set_own_target(o, t);
            o.dest = d;
        } else {
            crate::spawn::clear_own_target(o);
            o.dest = None;
        }
    }
    // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
    crate::emitters::retarget_let_go(w, p);
}

/// Silent removal (Terms): removed from the game at once, with no death
/// effects, wreck, survivor, experience or sound.
fn silent_remove(w: &mut World, p: ObjectId) {
    if let Some(s) = state_mut(w, p) {
        s.host = None;
    }
    crate::objects::remove(w, p);
}

fn fail_latch(w: &mut World, p: ObjectId) {
    let Some((cell, layer)) = state(w, p).and_then(|s| s.fired_cell.map(|c| (c, s.fired_layer)))
    else {
        silent_remove(w, p);
        return;
    };
    // spec: S-0614 claim 4 (the cell it last stood in, facing map north, under claim 20's placement rules; removed silently when it cannot be placed)
    if !place_ordinary(w, p, cell, layer, 0) {
        silent_remove(w, p);
        return;
    }
    // spec: S-0614 claim 4 (target and destination cleared unless attack-move memory; not held)
    restore_kept(w, p);
    if let Some(s) = state_mut(w, p) {
        s.hold_until = None;
    }
}

fn cell_z(w: &World, c: CellXY, layer: Layer) -> Pos {
    let p = Pos::centre(c, 0);
    let deck = if layer == Layer::Deck {
        crate::teleport::DECK_HEIGHT
    } else {
        0
    };
    Pos::new(p.x, p.y, w.surface_at(p.x, p.y).wrapping_add(deck))
}

/// Whether a cell holds an infantry unit or a vehicle of any house, other
/// than `except`.
fn has_unit(w: &World, c: CellXY, layer: Layer, except: Option<ObjectId>) -> bool {
    w.cells.list(c, layer).iter().any(|&x| {
        Some(x) != except
            && w.obj(x)
                .is_some_and(|o| matches!(o.kind, Kind::Infantry | Kind::Vehicle))
    })
}

fn has_building(w: &World, c: CellXY) -> bool {
    w.cells
        .occ(c, Layer::Ground)
        .is_some_and(|o| o.buildings > 0)
        || w.cells
            .list(c, Layer::Ground)
            .iter()
            .any(|&x| w.obj(x).is_some_and(|o| o.kind == Kind::Building))
}

fn class_zone(
    w: &World,
    id: ObjectId,
) -> Option<(Option<LocomotionClass>, glasnost_rules::sim::MovementZone)> {
    let o = w.obj(id)?;
    let u = w.rules().unit(o.kind, o.type_index)?;
    Some((u.class, u.zone))
}

/// The clear test of claim 13: enterable by class for the parasite's own
/// class and zone, a bridge cell judged on its deck, and no infantry and
/// no vehicle at all.
fn clear_for(w: &World, p: ObjectId, c: CellXY, layer: Layer) -> bool {
    let Some((class, zone)) = class_zone(w, p) else {
        return false;
    };
    // spec: S-0614 claim 13 (the clear test: enterable by class, a bridge cell judged on its deck, no infantry and no vehicle of any house)
    crate::passability::enterable_by_class(w.rules(), w, c, class, zone, layer == Layer::Deck)
        && !has_unit(w, c, layer, None)
}

/// The ordinary placement of a returning parasite (claim 20): one attempt
/// in exactly cell `c`. An infantry parasite requested at terrain height
/// inside the usable area first makes S-0208 claim 3's occupant-aware
/// centre-class position choice on the ground record (its draw kept even
/// when entry then refuses); then the entry outcome of S-0209 claims 4 and
/// 5, asked with the ground record and ground occupants, must be *clear*.
/// A vehicle uses the cell centre.
fn place_ordinary(w: &mut World, p: ObjectId, c: CellXY, layer: Layer, facing: Facing) -> bool {
    let Some(o) = w.obj(p) else { return false };
    let kind = o.kind;
    let ground_level = w.cells.cell(c).map_or(0, |x| i32::from(x.level));
    let base = cell_z(w, c, layer);
    let pos = if kind == Kind::Infantry && layer == Layer::Ground && w.cells.usable(c) {
        // spec: S-0614 claim 20 (infantry: the occupant-aware position choice of S-0208 claim 3, centre class, ground record, one ranged choice over 0 … 3 unless the flags refuse; before entry acceptance)
        let Some(k) = crate::walk::choose_position(
            w,
            c,
            Layer::Ground,
            crate::cells::position::CENTRE,
            false,
        ) else {
            return false;
        };
        let (u, v) = crate::walk::standing_point(k);
        Pos::new(256 * c.x + u, 256 * c.y + v, base.z) // spec: S-0204 claim 2 (a position within the cell)
    } else {
        // spec: S-0614 claim 20 (a vehicle uses the requested centre)
        base
    };
    // spec: S-0614 claim 20 (the entry outcome must be clear: S-0209 claims 4 and 5 with S-0211 claim 10, no approach direction or source elevation, ground occupants and ground record)
    let outcome = if kind == Kind::Infantry {
        crate::occupancy::infantry_outcome(w, p, c, ground_level)
    } else {
        crate::occupancy::vehicle_outcome(w, p, c, ground_level)
    };
    if outcome != crate::occupancy::Outcome::Clear {
        return false;
    }
    put(w, p, pos, layer, facing);
    true
}

/// Puts the parasite on the map at `pos` in `layer`.
fn put(w: &mut World, p: ObjectId, pos: Pos, layer: Layer, facing: Facing) {
    if let Some(o) = w.obj_mut(p) {
        o.layer = layer;
    }
    crate::objects::place(w, p, pos, facing);
}

/// The host-cell test of claim 13 on cell `c` for host `host` and
/// parasite `p`.
fn host_cell_ok(w: &World, host: ObjectId, p: ObjectId, c: CellXY) -> bool {
    // spec: S-0614 claim 22 (in the air: an aircraft by S-0501 claim 18's in-the-air answer; any other host by S-0307's airborne test, on the map and at least 208 leptons above its surface, a deck counted)
    let aircraft = w.obj(host).is_some_and(|o| o.kind == Kind::Aircraft);
    let in_air = if aircraft {
        crate::combat::weapon::in_air(w, host)
    } else {
        crate::combat::weapon::airborne_by_height(w, host)
    };
    if in_air {
        return false;
    }
    // spec: S-0614 claim 13 (no building stands in the cell)
    if has_building(w, c) {
        return false;
    }
    if !naval(w, p) {
        let land_type = w.cells.cell(c).map_or(land::CLEAR, |x| x.land);
        // spec: S-0614 claim 13 (not naval: a bridge, or a land type other than water, rock or beach)
        if !w.cells.is_bridge(c) && matches!(land_type, land::WATER | land::ROCK | land::BEACH) {
            return false;
        }
        // spec: S-0614 claim 13 (a vehicle parasite: no terrain object in the cell; a bridge bypasses only the land-type restriction)
        if w.obj(p).is_some_and(|o| o.kind == Kind::Vehicle)
            && w.cells
                .list(c, Layer::Ground)
                .iter()
                .any(|&x| w.obj(x).is_some_and(|o| o.kind == Kind::Terrain))
        {
            return false;
        }
    }
    true
}

/// The exit direction of F4 for a host facing `f`.
pub fn exit_direction(f: Facing) -> Facing {
    // spec: S-0614 F4 (f + q when o(f) is 0, 1 or 2, otherwise f − q, wrapped)
    if eight_direction(f) <= 2 {
        f.wrapping_add(QUARTER)
    } else {
        f.wrapping_sub(QUARTER)
    }
}

/// A 16-bit facing converted to 8-bit facing units by S-0204's rounding,
/// as a 16-bit facing.
fn rounded(f: Facing) -> Facing {
    // spec: S-0614 F4 (converted to 8-bit facing units by S-0204's rounding, F4 with n = 256)
    (n_step(f, 256) * 256) as Facing
}

fn neighbour(c: CellXY, dir: u8) -> CellXY {
    let (dx, dy) = crate::regions::STEPS[usize::from(dir)];
    CellXY::new(c.x + dx, c.y + dy)
}

/// How a parasite leaves a living host (claim 12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leave {
    /// A sonic hit, a teleport jump and the other exits: removed silently
    /// when not naval and suppressed, else claim 13.
    Exit,
    /// A healing hit whose warhead is not sonic, or a repair step: a
    /// parasite that is not naval is removed silently.
    Healing,
    /// The grappling parasite's off-water exit and its cull: claim 13
    /// without the hold afterwards.
    NoHold,
}

fn unlink(w: &mut World, p: ObjectId) -> Option<ObjectId> {
    let host = state(w, p).and_then(|s| s.host)?;
    // spec: S-0614 claim 25 (the grab animation is removed when the parasite leaves the host by any way of claims 11 to 13, when the host is lost and when the parasite is removed; before the host leaves play)
    remove_grab_anim(w, p);
    if let Some(s) = state_mut(w, p) {
        s.host = None;
    }
    if let Some(s) = state_mut(w, host) {
        s.attached = None;
        // spec: S-0614 claim 7 (when the parasite leaves or is lost, the host's hold ends at once)
        s.hold_until = None;
    }
    Some(host)
}

/// A parasite leaves its living host (claims 12 and 13).
pub fn leave_living(w: &mut World, p: ObjectId, how: Leave) {
    let Some(host) = state(w, p).and_then(|s| s.host) else {
        return;
    };
    let Some(h) = w.obj(host).cloned() else {
        return;
    };
    unlink(w, p);
    let naval = naval(w, p);
    // spec: S-0614 claim 12 and S-0315 claim 7 (healing or repair removes a parasite that is not naval; otherwise one that is not naval and suppressed)
    if !naval && (how == Leave::Healing || suppressed(w, p)) {
        silent_remove(w, p);
        return;
    }
    let q = w.rules().quirks;
    let hc = h.pos.cell();
    let f = h.body.value(w.tick);
    let exit = exit_direction(f);
    // spec: S-0614 claim 13 (the candidate: the host's own cell, or for a naval parasite the neighbour in the 8-direction of the exit direction)
    let candidate = if naval {
        neighbour(hc, eight_direction(exit))
    } else {
        hc
    };
    let layer_of = |w: &World, c: CellXY| {
        if h.layer == Layer::Deck && w.cells.is_bridge(c) {
            Layer::Deck
        } else {
            Layer::Ground
        }
    };
    let cl = layer_of(w, candidate);
    let chosen = if clear_for(w, p, candidate, cl) {
        Some(candidate)
    } else {
        // spec: S-0614 claim 13 and S-0212 table T5 (*Parasite exit*: centred on the host's cell, the host's class with winged as track and the host's zone; quirks.parasite_exit_search_host_mobility)
        let mob = if q.parasite_exit_search_host_mobility {
            class_zone(w, host)
        } else {
            class_zone(w, p)
        };
        mob.and_then(|(class, zone)| {
            crate::regions::ensure(w);
            let deck = h.layer == Layer::Deck;
            let settings = crate::path::NearbySettings {
                class: class.map(|k| match k {
                    LocomotionClass::Winged => LocomotionClass::Track,
                    other => other,
                }),
                zone,
                reach: Some(crate::regions::reach_read(w, hc, zone, deck)),
                deck,
                height: false,
                reference: None,
                no_bridges: false,
                no_overlay: false,
            };
            crate::path::nearby(w, hc, crate::path::NearbyUse::Settings(settings))
        })
    };
    let Some(c) = chosen else {
        // spec: S-0614 claim 13 (no cell: removed silently)
        silent_remove(w, p);
        return;
    };
    // spec: S-0614 claim 13 (the host-cell test on the host's cell; quirks.parasite_exit_tests_host_cell off: on the chosen cell)
    let tested = if q.parasite_exit_tests_host_cell {
        hc
    } else {
        c
    };
    if !host_cell_ok(w, host, p, tested) {
        silent_remove(w, p);
        return;
    }
    // spec: S-0614 claim 13 (the requested point is the centre of the chosen cell, facing the exit direction, under claim 20's placement rules)
    if !place_ordinary(w, p, c, layer_of(w, c), rounded(exit)) {
        silent_remove(w, p);
        return;
    }
    restore_kept(w, p);
    let now = w.tick;
    let r = primary_rof(w, p);
    if let Some(s) = state_mut(w, p) {
        // spec: S-0614 claim 13 and F1 (held for 3 × R ticks, wrapped); claims 10 and 11 (not held after leaving off water or a cull)
        s.hold_until =
            (how != Leave::NoHold).then(|| now.wrapping_add(EXIT_HOLD_RATES.wrapping_mul(r)));
    }
    // Claim 19's re-selection is presentation: selection is not game state.
}

/// The host leaves the map or is removed (claim 15), called before it
/// does; also unlinks an attached parasite that is itself leaving.
pub fn leaving(w: &mut World, id: ObjectId) {
    let Some(s) = state(w, id).cloned() else {
        return;
    };
    if s.host.is_some() {
        unlink(w, id);
    }
    if let Some(p) = s.attached {
        host_lost(w, id, p);
    }
}

fn host_lost(w: &mut World, host: ObjectId, p: ObjectId) {
    let Some(h) = w.obj(host).cloned() else {
        return;
    };
    unlink(w, p);
    let q = w.rules().quirks;
    let naval = naval(w, p);
    // spec: S-0614 claim 15 (suppressed: removed silently, naval too with quirks.parasite_host_loss_suppresses_naval; temporary invulnerability is a seam)
    if suppressed(w, p) && (q.parasite_host_loss_suppresses_naval || !naval) {
        silent_remove(w, p);
        return;
    }
    let hc = h.pos.cell();
    let f = h.body.value(w.tick);
    let infantry = w.obj(p).is_some_and(|o| o.kind == Kind::Infantry);
    let at = if host_cell_ok(w, host, p, hc) {
        // spec: S-0614 claim 15 (the host's position: a vehicle at the centre of its cell, at the deck's height on a bridge; an infantry parasite at the host's position class, S-0208 claim 2, with no regard to occupants and no draw)
        let class = if infantry {
            let (u, v) = (h.pos.x.rem_euclid(256), h.pos.y.rem_euclid(256)); // spec: S-0204 claim 2 (the point within its cell)
            crate::walk::position_class(u, v, q.top_position_counts_as_centre)
        } else {
            crate::cells::position::CENTRE
        };
        Some((hc, h.layer, class))
    } else {
        // spec: S-0614 claim 15 (the neighbour in the host's own 8-direction)
        let n = neighbour(hc, eight_direction(f));
        let land_type = w.cells.cell(n).map(|x| x.land);
        // spec: S-0614 claim 15 (quirks.parasite_host_loss_bridge_test: the host's cell's bridge; off: the neighbour's)
        let bridge = if q.parasite_host_loss_bridge_test {
            w.cells.is_bridge(hc)
        } else {
            w.cells.is_bridge(n)
        };
        match land_type {
            None => None,
            Some(l) if matches!(l, land::WATER | land::ROCK | land::BEACH) && !naval && !bridge => {
                None
            }
            Some(_) => {
                // spec: S-0614 claim 15 (the fallback's occupant-aware centre-class position choice of S-0208 claim 3 on the neighbour's ground record, even for a vehicle; one ranged choice unless the flags refuse)
                let k = crate::walk::choose_position(
                    w,
                    n,
                    Layer::Ground,
                    crate::cells::position::CENTRE,
                    false,
                );
                match k {
                    Some(k) => Some((n, surface_layer(w, &h, n, k, infantry), k_of(k, infantry))),
                    // spec: S-0614 claim 23 (failed choice, a flag refusal or no free side position: an infantry parasite is removed silently)
                    None if infantry => None,
                    // spec: S-0614 claim 23 (quirks.parasite_host_loss_failed_choice_origin: a vehicle survives at world position (128, 128, 0), unattached, unmarked on a bridge; off: removed silently)
                    None if q.parasite_host_loss_failed_choice_origin => {
                        put(w, p, Pos::new(128, 128, 0), Layer::Ground, f);
                        released(w, p);
                        return;
                    }
                    None => None,
                }
            }
        }
    };
    let Some((c, layer, k)) = at else {
        // spec: S-0614 claim 15 (no position: removed silently)
        silent_remove(w, p);
        return;
    };
    // spec: S-0614 claim 15 (final placement skips claim 20's entry check; it faces the host's facing; not held)
    let base = cell_z(w, c, layer);
    let (u, v) = crate::walk::standing_point(k);
    let (x, y) = (256 * c.x + u, 256 * c.y + v); // spec: S-0204 claim 2 (a position within the cell)
    let z = if layer == Layer::Deck {
        // spec: S-0614 claim 23 (on the deck: the point's terrain surface plus 416 leptons)
        w.surface_at(x, y)
            .wrapping_add(crate::teleport::DECK_HEIGHT)
    } else {
        base.z
    };
    put(w, p, Pos::new(x, y, z), layer, f);
    released(w, p);
}

/// A parasite released by host loss: its kept orders back and no hold
/// (claim 15).
fn released(w: &mut World, p: ObjectId) {
    restore_kept(w, p);
    if let Some(s) = state_mut(w, p) {
        s.hold_until = None;
    }
}

/// The standing position a fallback choice `k` gives: an infantry
/// parasite's side position, a vehicle's cell centre (claim 15).
fn k_of(k: u8, infantry: bool) -> u8 {
    if infantry {
        k
    } else {
        // spec: S-0614 claim 15 (a vehicle uses the resulting point's cell centre when the choice succeeds)
        crate::cells::position::CENTRE
    }
}

/// The layer a parasite released at fallback point `k` of cell `n` stands
/// on (claim 23): the deck exactly when `n` is a high-bridge cell and
/// either the host stood on a high bridge or the point's terrain surface
/// is below the host cell's centre terrain height.
fn surface_layer(w: &World, h: &Object, n: CellXY, k: u8, infantry: bool) -> Layer {
    // spec: S-0614 claim 23 (after vehicle centring; exact integer heights)
    let (u, v) = crate::walk::standing_point(k_of(k, infantry));
    let z = w.surface_at(256 * n.x + u, 256 * n.y + v); // spec: S-0204 claim 2 (a position within the cell)
    let hc = Pos::centre(h.pos.cell(), 0);
    let host_z = w.surface_at(hc.x, hc.y);
    let host_on_bridge = h.layer == Layer::Deck;
    // spec: S-0614 claim 23 (deck when B and either the host stood on a high bridge or z < h)
    if w.cells.is_bridge(n) && (host_on_bridge || z < host_z) {
        Layer::Deck
    } else {
        Layer::Ground
    }
}

fn primary(w: &World, p: ObjectId) -> Option<WeaponId> {
    // spec: S-0614 claims 6 and 10 (the primary weapon, the elite slot when elite, S-0311 claim 6)
    crate::combat::weapon::slot_weapon(w, p, 0)
}

fn primary_rof(w: &World, p: ObjectId) -> i32 {
    primary(w, p).map_or(0, |wp| w.rules().combat.weapon(wp).rof)
}

/// Restarts a host's hold timer at the warhead's paralysis length.
fn renew_hold(w: &mut World, host: ObjectId, weapon: Option<WeaponId>) {
    let now = w.tick;
    let p = weapon
        .and_then(|wp| w.rules().combat.warhead_of(wp))
        .map_or(0, |h| h.paralyzes);
    if let Some(s) = state_mut(w, host) {
        // spec: S-0614 claim 7 and F1 (held during t ≤ tick < t + P)
        s.hold_until = Some(now.wrapping_add(p));
    }
}

/// The attached parasite acts late in its host's turn (claim 18): a
/// drone's deliveries (claim 6) or a grappling parasite's cycle (claim
/// 10), after the host's self-healing, behaviour processing and movement,
/// before its separate firing update.
pub fn host_turn(w: &mut World, host: ObjectId) {
    let Some(p) = parasite_of(w, host) else {
        return;
    };
    if w.obj(p).is_none() {
        if let Some(s) = state_mut(w, host) {
            s.attached = None;
        }
        return;
    }
    let pos = w.obj(host).map(|o| o.pos);
    if let (Some(pos), Some(o)) = (pos, w.obj_mut(p)) {
        o.pos = pos;
    }
    if grappling(w, p) {
        grapple_turn(w, host, p);
    } else {
        drone_turn(w, host, p);
    }
}

fn drone_turn(w: &mut World, host: ObjectId, p: ObjectId) {
    let now = w.tick;
    let Some(s) = state_mut(w, p) else { return };
    if !s.started {
        // spec: S-0614 claim 3 (the first delivery t₀ at the host's next turn)
        s.started = true;
        s.next = now;
    }
    if now < s.next {
        return;
    }
    let weapon = primary(w, p);
    // spec: S-0614 claim 6 (each delivery restarts the host's hold timer at P first)
    renew_hold(w, host, weapon);
    deliver(w, host, p, weapon);
    if host_of(w, p) == Some(host) {
        let r = weapon.map_or(0, |wp| w.rules().combat.weapon(wp).rof);
        if let Some(s) = state_mut(w, p) {
            // spec: S-0614 claim 6 and F1 (then every max(1, R) ticks)
            s.next = now.wrapping_add(r.max(1));
        }
    }
}

fn deliver(w: &mut World, host: ObjectId, p: ObjectId, weapon: Option<WeaponId>) {
    let Some(wp) = weapon else { return };
    let Some(h) = w.obj(host).cloned() else {
        return;
    };
    let rules = w.rules_arc();
    let wr = rules.combat.weapon(wp).clone();
    let half = rules.quirks.ranged_draw_half_span;
    if h.kind == Kind::Infantry {
        // spec: S-0614 claim 6 (an infantry host: a hit of its current health, ignoring defences, no-escape, the parasite as attacker, no house; no rocking choice)
        crate::combat::damage::apply(
            w,
            host,
            Hit {
                amount: h.health,
                warhead: wr.warhead,
                distance: 0,
                attacker: Some(p),
                house: None,
                ignore_defences: true,
                no_escape: true,
            },
        );
        return;
    }
    // spec: S-0614 claim 9 and S-0616 claim 1 (a system of the game-wide spark type at the host's position; creating it takes no draw)
    crate::particles::spark_system(w, h.pos);
    // spec: S-0614 claim 9 (the animation indexed directly by the host's 8-direction, none past the list's end or for an empty list)
    let dir = eight_direction(h.body.value(w.tick));
    if let Some(name) = wr.anim.get(usize::from(dir)) {
        crate::combat::effects::anim(w, name, h.pos);
    }
    // spec: S-0614 claim 9 (one draw R(0, 1) for the rocking side, after the effects and before the hit)
    let side = w.rng.ranged(0, 1, half);
    // spec: S-0614 claim 9 and F2 (the rocking is presentation: the drawing rocks the host to the side the draw chose)
    w.effects
        .push(crate::combat::effects::Effect::DroneRock { host, side });
    // spec: S-0614 claim 6 (a hit of the weapon's damage at distance 0, not ignoring defences, no-escape, the parasite as attacker, no house)
    crate::combat::damage::apply(
        w,
        host,
        Hit {
            amount: wr.damage,
            warhead: wr.warhead,
            distance: 0,
            attacker: Some(p),
            house: None,
            ignore_defences: false,
            no_escape: true,
        },
    );
}

fn grapple_turn(w: &mut World, host: ObjectId, p: ObjectId) {
    let now = w.tick;
    let weapon = primary(w, p);
    // spec: S-0614 claim 10 (first the hold restarts at P, then the host's cell is checked)
    renew_hold(w, host, weapon);
    let hc = w.obj(host).map(|o| o.pos.cell());
    if hc.is_some_and(|c| !water_tile(w, c)) {
        // spec: S-0614 claim 10 (not a water tile: it leaves at once and is not held afterwards)
        leave_living(w, p, Leave::NoHold);
        return;
    }
    let Some(s) = state_mut(w, p) else { return };
    if !s.started {
        // spec: S-0614 F3 (g: the host's first turn after latching, the grab's first tick)
        s.started = true;
        s.grabbing = true;
        s.next = now.wrapping_add(GRAB_END);
        s.phase_start = now;
        s.push_way = None;
        // spec: S-0614 claim 25 (created once per hold, in the grab's first turn, after the hold renewal and the water check, before the phase-frame step)
        create_grab_anim(w, host, p);
    }
    // spec: S-0614 claim 25 (in each turn of the hold that reaches the step: reattached when found detached, the parasite's palette and the host's cell given)
    tend_grab_anim(w, host, p);
    // spec: S-0614 claim 25 and table G (the phase-frame step: a step of j to 1 … 9 sets the grab animation's stage)
    grab_stage(w, host, p);
    let Some(s) = state_mut(w, p) else { return };
    if now < s.next {
        return;
    }
    if s.grabbing {
        s.grabbing = false;
        // spec: S-0614 table G (the grab ends: a push starts in the same tick)
        let way = push_start(w, host);
        if let Some(s) = state_mut(w, p) {
            s.next = now.wrapping_add(PUSH);
            s.phase_start = now;
            s.push_way = Some(way);
        }
        return;
    }
    strike(w, host, p, weapon);
}

/// The fixed grab animation type's name (claim 25, Import mapping).
const GRAB_ANIM: &str = "SQDG"; // spec: S-0614 Import mapping (the grab animation: animation type `SQDG`, an engine constant, found without regard to letter case)
/// The grab animation's stage delay at each setting, in ticks.
const GRAB_STAGE_DELAY: i32 = 128; // spec: S-0614 claim 25 (the stage clock restarted with a stage delay of 128 ticks, at every game speed)
/// Ticks per phase frame in the grab.
const GRAB_FRAME: i32 = 3; // spec: S-0614 table G and F3 (phase frames advance one per 3 ticks in the grab)
/// Ticks per phase frame in a push.
const PUSH_FRAME: i32 = 4; // spec: S-0614 table G and F3 (one per 4 ticks in a push)
/// The last phase frame whose step sets the stage.
const LAST_SET_FRAME: i32 = 9; // spec: S-0614 claim 25 (a step of j to 1 … 9 sets k; a step to 10 sets nothing)
/// The stage offset *b* in a push with σ = −1 and σ = +1.
const PUSH_BASE_DOWN: i32 = 80; // spec: S-0614 claim 25 (b = 80 in a push with σ = −1)
const PUSH_BASE_UP: i32 = 160; // spec: S-0614 claim 25 (b = 160 in a push with σ = +1)
/// Stages per 8-direction step.
const DIRECTION_STAGES: i32 = 10; // spec: S-0614 claim 25 (k = b + 10 × d + j)

/// The grab animation's creation (claim 25): at the host's position,
/// height included, with no loop delay, a repeat factor of 1 and no
/// reverse request, attached to the host with no displacement. Nothing
/// when no animation type has the fixed name.
fn create_grab_anim(w: &mut World, host: ObjectId, p: ObjectId) {
    let Some(at) = w.obj(host).map(|o| o.pos) else {
        return;
    };
    let spawn = crate::anims::Spawn {
        // spec: S-0614 claim 25 (no initial loop delay, a repeat factor of 1, no reverse request)
        creation: crate::anim_clock::Creation {
            loop_delay: 0,
            repeat: 1,
            reverse: false,
        },
        // spec: S-0614 claim 25 and S-0109 claim 22 (attached to the host with no displacement)
        owner: Some((host, Pos::new(0, 0, 0))),
        role: crate::anims::Role::Grab {
            parasite: p,
            house: None,
            cell: None,
        },
    };
    // spec: S-0614 claim 25 (the creation draws of S-0109 claims 3 and 13, from the scenario stream, in turn g; an absent type: none created and no draw)
    let made = crate::combat::effects::anim_spawn(w, GRAB_ANIM, at, spawn);
    if let Some(s) = state_mut(w, p) {
        s.grab_anim = made;
    }
}

/// The parasite tends its grab animation in a turn of the hold that
/// reaches the attached-parasite step (claim 25): attached again when
/// found detached, given the parasite's house for its palette and the
/// host's cell for its brightness. Nothing once it has ended.
fn tend_grab_anim(w: &mut World, host: ObjectId, p: ObjectId) {
    let Some(id) = state(w, p).and_then(|s| s.grab_anim) else {
        return;
    };
    let house = w.obj(p).and_then(|o| o.owner);
    let cell = w.obj(host).filter(|o| o.on_map).map(|o| o.pos.cell());
    let Some(a) = w.anims.anims.get_mut(&id) else {
        // spec: S-0614 claim 25 (when the animation has ended, nothing recreates it in that hold)
        if let Some(s) = state_mut(w, p) {
            s.grab_anim = None;
        }
        return;
    };
    if a.owner.is_none() {
        // spec: S-0614 claim 25 (attached again, with no displacement, in a later turn that finds it detached)
        a.owner = Some((host, Pos::new(0, 0, 0)));
    }
    // spec: S-0614 claim 25 (the house-colour palette of the parasite's house; the unscaled ground level of the host's cell, when it is in a cell: the view reads both)
    a.role = crate::anims::Role::Grab {
        parasite: p,
        house,
        cell: cell.or(match a.role {
            crate::anims::Role::Grab { cell, .. } => cell,
            _ => None,
        }),
    };
}

/// The phase-frame step of the grappling cycle (table G) and the grab
/// animation's stage setting (claim 25): in the turn at which phase frame
/// *j* steps to a value from 1 to 9, *k* = *b* + 10 × *d* + *j*.
fn grab_stage(w: &mut World, host: ObjectId, p: ObjectId) {
    let now = w.tick;
    let Some(s) = state(w, p) else { return };
    let Some(id) = s.grab_anim else { return };
    // spec: S-0614 table G and F3 (the phase's first frame counted in the tick it starts: frame j at its turn n = offset + 1 is ⌊n / 3⌋ in the grab, ⌊n / 4⌋ in a push)
    let (period, base) = match s.push_way {
        None => (GRAB_FRAME, 0), // spec: S-0614 claim 25 (b = 0 in the grab)
        // spec: S-0614 claim 25 and S-0113 claim 14 (σ = +1 for a way draw of 0, −1 for 1)
        Some(0) => (PUSH_FRAME, PUSH_BASE_UP),
        Some(_) => (PUSH_FRAME, PUSH_BASE_DOWN),
    };
    let turn = now.wrapping_sub(s.phase_start).wrapping_add(1);
    // spec: S-0614 claim 25 (set only at a step of j: the grab's turns 3, 6, …, 27 and a push's turns 4, 8, …, 36)
    if turn <= 0 || turn % period != 0 {
        return;
    }
    let j = turn / period;
    if !(1..=LAST_SET_FRAME).contains(&j) {
        return;
    }
    let Some(f) = w.obj(host).map(|o| o.body.value(now)) else {
        return;
    };
    // spec: S-0614 claim 25 (d: the 8-direction of the host's current body facing at that step, S-0204 F4)
    let d = i32::from(eight_direction(f));
    if let Some(a) = w.anims.anims.get_mut(&id) {
        // spec: S-0614 claim 25 (k = b + 10 × d + j, the clock restarted with a 128-tick stage delay; no draw)
        a.player
            .set_stage(base + DIRECTION_STAGES * d + j, now, GRAB_STAGE_DELAY);
    }
}

/// Removes a parasite's grab animation now (claim 25).
fn remove_grab_anim(w: &mut World, p: ObjectId) {
    let Some(id) = state_mut(w, p).and_then(|s| s.grab_anim.take()) else {
        return;
    };
    // spec: S-0614 claim 25 (removed at once: no longer drawn from that turn on, it leaves play in that turn, S-0203 claim 10)
    crate::anims::remove(w, id);
}

/// A push starts (claim 10): R(0, 1) for its way, then three wakes, each
/// after R(−180, 180) and R(−64, 64) for its displacement. Returns the
/// way draw.
fn push_start(w: &mut World, host: ObjectId) -> i32 {
    let rules = w.rules_arc();
    let half = rules.quirks.ranged_draw_half_span;
    let pos = w.obj(host).map_or(Pos::default(), |o| o.pos);
    // spec: S-0614 claim 10 (one draw R(0, 1): which way the push rocks)
    let way = w.rng.ranged(0, 1, half);
    // spec: S-0614 claim 10 (the rocking of the host during the push is presentation: the drawing rocks it the way the draw chose)
    w.effects
        .push(crate::combat::effects::Effect::GrapplePush { host, way });
    for _ in 0..WAKES {
        // spec: S-0614 claim 10 (each wake preceded by R(−180, 180) and then R(−64, 64) leptons)
        let dx = w.rng.ranged(-180, 180, half);
        let dy = w.rng.ranged(-64, 64, half); // spec: S-0614 claim 10 (R(−64, 64))
        if let Some(wake) = rules.combat.general.wake.clone() {
            crate::combat::effects::anim(w, &wake, Pos::new(pos.x + dx, pos.y + dy, pos.z));
        }
    }
    way
}

fn strike(w: &mut World, host: ObjectId, p: ObjectId, weapon: Option<WeaponId>) {
    let now = w.tick;
    let rules = w.rules_arc();
    let half = rules.quirks.ranged_draw_half_span;
    let Some(h) = w.obj(host).cloned() else {
        return;
    };
    let list = rules.combat.general.splash_list.clone();
    let n = list.len() as i32;
    for _ in 0..SPLASHES {
        // spec: S-0614 claim 10 (each splash: R(0, 1), then R(0, n − 1), before the cull test)
        let _ = w.rng.ranged(0, 1, half);
        let k = w.rng.ranged(0, n - 1, half);
        if let Some(name) = usize::try_from(k).ok().and_then(|k| list.get(k)) {
            crate::combat::effects::anim(w, name, h.pos);
        }
    }
    let wr = weapon.map(|wp| rules.combat.weapon(wp).clone());
    let culling = weapon
        .and_then(|wp| rules.combat.warhead_of(wp))
        .is_some_and(|x| x.culling);
    if culling && cull_due(w, host, p) {
        cull(w, host, p);
        return;
    }
    if let Some(wr) = wr {
        // spec: S-0614 claim 10 (a hit of the weapon's damage at distance 0, not ignoring defences, no-escape, the parasite as attacker, no house)
        crate::combat::damage::apply(
            w,
            host,
            Hit {
                amount: wr.damage,
                warhead: wr.warhead,
                distance: 0,
                attacker: Some(p),
                house: None,
                ignore_defences: false,
                no_escape: true,
            },
        );
        if w.obj(host).is_none_or(|o| o.health <= 0) {
            // spec: S-0614 claim 25 (removed when the strike's own hit destroys the host, in that strike's turn)
            remove_grab_anim(w, p);
        }
    }
    // spec: S-0614 claim 10 and table G (when the host still has this parasite: the next push starts in the strike's tick)
    if host_of(w, p) == Some(host) && w.obj(host).is_some() {
        let way = push_start(w, host);
        if let Some(s) = state_mut(w, p) {
            s.next = now.wrapping_add(PUSH);
            s.phase_start = now;
            s.push_way = Some(way);
        }
    }
}

/// The cull test of F6.
fn cull_due(w: &World, host: ObjectId, p: ObjectId) -> bool {
    let Some(h) = w.obj(host) else { return false };
    let r = w.rules();
    let mode = r.mode;
    let s = crate::combat::damage::strength(w, host);
    // spec: S-0614 F6 (h, the health ratio in binary64, S-0111)
    let ratio =
        F64::from_i64(i64::from(h.health), mode).div(F64::from_i64(i64::from(s), mode), mode);
    let red = r.general.condition_red;
    let yellow = r.general.condition_yellow;
    let elite = veterancy::rank(w, p) == Rank::Elite;
    // spec: S-0614 F6 (h ≤ red and health > 0, or red < h ≤ yellow and the parasite elite)
    (ratio.le(red) && h.health > 0) || (red.lt(ratio) && ratio.le(yellow) && elite)
}

fn cull(w: &mut World, host: ObjectId, p: ObjectId) {
    let trainable = veterancy::type_combat(w, p).is_some_and(|tc| tc.trainable);
    if w.rules().quirks.cull_experience_path {
        if trainable {
            // spec: S-0614 F5 and claim 16 (e′ = S-0311 F3 with V = A_h, the host's actual cost, and A_r = A_p, the parasite's for its own house; no rank multiplier, alliance or unscored test)
            let ah = veterancy::actual_cost(w, host);
            let ap = veterancy::actual_cost(w, p);
            veterancy::gain(w, p, ah, ap);
        }
    } else {
        // spec: S-0614 Edge cases (quirks.cull_experience_path off: a destruction credited to the parasite under S-0311 claims 7 to 10)
        let house = w.obj(p).and_then(|o| o.owner);
        veterancy::destruction_credited(w, host, Some(p), house);
    }
    // spec: S-0614 claim 11 (the parasite leaves, not held afterwards, before the host starts to sink)
    leave_living(w, p, Leave::NoHold);
    // spec: S-0614 claim 11 and S-0312 claim 10 (the host starts to sink without a hit, whatever its weight)
    crate::naval::start_sinking(w, host);
}

/// A hit on a host (S-0315 claims 6 and 7; S-0614 claim 12), before the
/// hit changes the host's health. The caller leaves out the hits of
/// S-0315 claim 6's vehicle-factory exception.
pub fn host_hit(w: &mut World, host: ObjectId, hit: &Hit) {
    let Some(p) = parasite_of(w, host) else {
        return;
    };
    let rules = w.rules_arc();
    let sonic = hit.warhead.is_some_and(|h| rules.combat.warhead(h).sonic);
    if sonic {
        // spec: S-0315 claim 7 and S-0614 claim 12 (a sonic hit, whatever its amount: the parasite leaves)
        leave_living(w, p, Leave::Exit);
        return;
    }
    if hit.amount < 0 {
        // spec: S-0315 claim 7 and S-0614 claim 12 (a healing hit whose warhead is not sonic)
        leave_living(w, p, Leave::Healing);
        return;
    }
    let t = w
        .obj(p)
        .and_then(|o| rules.unit(o.kind, o.type_index))
        .map_or(0, |u| u.vehicle.suppression_threshold);
    let d = hit.amount;
    // spec: S-0315 claim 6 (d ≥ 0, d > T, the attacker not the parasite itself)
    if d > t && hit.attacker != Some(p) {
        // spec: S-0315 Formulas (t = W(2d − T); t ≤ 0: not suppressed)
        let time = (2 * i64::from(d) - i64::from(t)) as i32;
        let now = w.tick;
        if let Some(s) = state_mut(w, p) {
            s.suppressed_until = (time > 0).then(|| now.wrapping_add(time));
        }
    }
}

/// A repair step on a host goes ahead (S-0314 claim 3; S-0614 claim 12).
pub fn repair_step(w: &mut World, host: ObjectId) {
    if let Some(p) = parasite_of(w, host) {
        // spec: S-0614 claim 12 (a repair step: as a healing hit whose warhead is not sonic)
        leave_living(w, p, Leave::Healing);
    }
}

/// The host makes its own jump under the teleporting locomotor (S-0207
/// claim 20 e; S-0614 claim 12): the parasite exits before the host leaves
/// its old position. Not the hook for a teleporting superweapon's
/// departure: whether that releases a drone parasite is unresolved
/// (S-0614 claim 12 and Open questions).
pub fn host_jumps(w: &mut World, host: ObjectId) {
    if let Some(p) = parasite_of(w, host) {
        // spec: S-0614 claim 12 (the teleport jump: it exits before the host leaves its old position)
        leave_living(w, p, Leave::Exit);
    }
}

/// The host, a transport, lets a passenger out (claim 12), called by the
/// transports topic (S-0507 claims 10 and 13).
pub fn host_unloading(w: &mut World, host: ObjectId) {
    suppress_and_exit(w, host);
}

/// The host reaches a consuming building while being eaten (claim 12): a
/// hook for that building topic, before the host's removal.
pub fn host_consumed(w: &mut World, host: ObjectId) {
    suppress_and_exit(w, host);
}

fn suppress_and_exit(w: &mut World, host: ObjectId) {
    let Some(p) = parasite_of(w, host) else {
        return;
    };
    let now = w.tick;
    if let Some(s) = state_mut(w, p) {
        // spec: S-0614 claim 12 (50 ticks of suppression just before the exit)
        s.suppressed_until = Some(now.wrapping_add(EXIT_SUPPRESSION));
    }
    leave_living(w, p, Leave::Exit);
}

/// A teleporting superweapon catches the host (claim 12), at launch: a
/// naval parasite exits at the host's old position; a drone parasite is
/// not released at launch. Its fate at the departure is unresolved
/// (S-0614 claim 12 and Open questions), so no departure hook exists. A
/// hook for the superweapon topic, which our engine does not model yet.
pub fn superweapon_teleport(w: &mut World, host: ObjectId) {
    if let Some(p) = parasite_of(w, host)
        && naval(w, p)
    {
        let now = w.tick;
        if let Some(s) = state_mut(w, p) {
            // spec: S-0513 claim 34 and Constants (a naval parasite is released with 500 ticks of suppression before any other effect of the warp)
            s.suppressed_until = Some(now.wrapping_add(WARP_SUPPRESSION));
        }
        // spec: S-0614 claim 12 (a naval parasite exits at launch, before the teleport)
        leave_living(w, p, Leave::Exit);
    }
}

/// The host starts curtain protection (S-0513 claim 25; S-0614 claim 12):
/// 50 ticks of suppression are set just before the release, so a drone
/// parasite is removed silently and a naval parasite exits by claim 13;
/// the host's hold ends. A hook for the superweapon topic, which our
/// engine does not model yet.
pub fn host_curtain_started(w: &mut World, host: ObjectId) {
    // spec: S-0614 claim 12 (curtain protection: suppression set just before release)
    suppress_and_exit(w, host);
}

/// An effect replaces the host's locomotor (claim 12): only a naval
/// parasite exits. A hook for the seizing warhead (S-0607 claims 1 and 5).
pub fn locomotor_replaced(w: &mut World, host: ObjectId) {
    if let Some(p) = parasite_of(w, host)
        && naval(w, p)
    {
        // spec: S-0614 claim 12 (a replaced locomotor: a naval parasite exits, a drone stays)
        leave_living(w, p, Leave::Exit);
    }
}

/// Whether `t` is a mobile unit with a parasite on it (S-0214 claim 6).
pub fn carries_parasite(w: &World, t: ObjectId) -> bool {
    w.obj(t).is_some_and(|o| o.kind.is_mobile()) && parasite_of(w, t).is_some()
}
