//! Disguise (S-0516 claims 30 to 36, 39): taking, keeping and losing a
//! disguise, and the game-state test *appears disguised to H*.

use serde::{Deserialize, Serialize};

use super::{centre_cell, detected};
use crate::combat::data::WeaponId;
use crate::combat::veterancy::type_combat;
use crate::rules::Kind;
use crate::world::{HouseId, Layer, ObjectId, Target, World};

/// A disguise type (S-0516 Terms): an infantry, vehicle, overlay or
/// terrain-object type, by registration index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DisguiseAs {
    /// An infantry type.
    Infantry(u32),
    /// A vehicle type.
    Vehicle(u32),
    /// An overlay type.
    Overlay(u8),
    /// A terrain-object type.
    Terrain(u32),
}

/// A unit's disguise (S-0516 claim 30): the disguised flag is the
/// `Option` holding it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Disguise {
    /// The disguise type; `None` for a side default that names no type
    /// (S-0516 Edge cases).
    pub what: Option<DisguiseAs>,
    /// The disguise house, possibly none.
    pub house: Option<HouseId>,
    /// The tick the disguise was taken (F6).
    pub since: i32,
}

/// Whether the object's type can disguise and disguises permanently.
fn kinds(w: &World, id: ObjectId) -> (bool, bool, bool) {
    let Some(o) = w.obj(id) else {
        return (false, false, false);
    };
    let tc = type_combat(w, id);
    let can = tc.is_some_and(|t| t.can_disguise);
    let still = tc.is_some_and(|t| t.disguise_when_still);
    // spec: S-0516 Import mapping (PermaDisguise absent: no)
    let perma = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.raw.perma_disguise.unwrap_or(false));
    (can, perma, still)
}

/// The default disguise of a house's side (S-0516 claim 32): the first
/// side the allied default, the second the soviet default, any other side
/// the third default.
pub fn side_default(w: &World, h: Option<HouseId>) -> Option<u32> {
    let r = w.rules();
    let side = h
        .and_then(|h| w.houses.get(usize::from(h.0)))
        .and_then(|house| {
            r.econ
                .tech
                .country_side
                .get(house.country as usize)
                .copied()
                .flatten()
        });
    // spec: S-0516 claim 32 (first side: allied default; second: soviet default; any other: third default)
    let i = match side {
        Some(0) => 0,
        Some(1) => 1,
        _ => 2,
    };
    r.cloak.side_disguise[i]
}

/// The permanent disguise of a unit (S-0516 claim 32): its side default,
/// with its owner as the disguise house.
fn default_disguise(w: &World, id: ObjectId) -> Option<Disguise> {
    let o = w.obj(id)?;
    Some(Disguise {
        what: side_default(w, o.owner).map(DisguiseAs::Infantry),
        house: o.owner,
        since: w.tick,
    })
}

/// Creation (S-0516 claim 32): a permanent disguiser is created disguised.
pub fn created(w: &mut World, id: ObjectId) {
    let (can, perma, _) = kinds(w, id);
    // spec: S-0516 claim 32 (a type that can disguise and disguises permanently)
    if !(can && perma) {
        return;
    }
    let d = default_disguise(w, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.disguise = d;
    }
}

/// Clears a disguise (S-0516 claim 34): a permanently disguising infantry
/// unit returns to its default, any other unit becomes undisguised. The
/// call of S-0517 claim 6 (undeploy) and of a disguise house leaving the
/// game (S-0414).
pub fn clear(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.cloak.disguise.is_none() {
        return;
    }
    let (_, perma, _) = kinds(w, id);
    // spec: S-0516 claim 32 (cleared permanent infantry returns to the default; vehicles become undisguised)
    let next = if o.kind == Kind::Infantry && perma {
        default_disguise(w, id)
    } else {
        None
    };
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.disguise = next;
    }
}

/// A house left the game (S-0516 claim 34 (e)): every disguise naming it
/// is cleared. Seam: the call S-0414's house removal makes.
pub fn house_left(w: &mut World, h: HouseId) {
    let ids: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.cloak.disguise.is_some_and(|d| d.house == Some(h)))
        .map(|o| o.id)
        .collect();
    for x in ids {
        // spec: S-0516 claim 34 (e) (the house its disguise names leaves the game)
        clear(w, x);
    }
}

/// The disguise reaction to a hit (S-0302 claim 12, S-0516 claim 34 (a)):
/// a type that can disguise but not permanently loses its disguise and
/// gets a 2 *m*′-tick lockout after *unchanged*, *now yellow* or *now
/// red*.
pub fn hit_taken(w: &mut World, id: ObjectId, result: crate::combat::damage::HitResult, lost: i32) {
    use crate::combat::damage::HitResult as R;
    if !matches!(result, R::Unchanged | R::NowYellow | R::NowRed) {
        return;
    }
    let (can, perma, _) = kinds(w, id);
    if !can || perma {
        return;
    }
    let tick = w.tick;
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.disguise = None;
        // spec: S-0302 claim 12 and Constants (the disguise-disruption countdown: 2 ticks per health point lost)
        ob.cloak.lockout_until = tick.saturating_add(lost.saturating_mul(2));
    }
}

/// Disguise when still (S-0516 claim 33), in a vehicle's turn.
pub fn turn(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Vehicle {
        return;
    }
    let (can, perma, still) = kinds(w, id);
    // spec: S-0516 claim 33 (vehicles whose type can disguise but does not disguise permanently; never infantry)
    if !can || perma {
        return;
    }
    let tick = w.tick;
    let owner = o.owner;
    let layer = o.layer;
    let c = o.pos.cell();
    let moving = crate::occupancy::in_progress(w, id);
    let mut applied = false;
    if moving {
        // spec: S-0516 claim 33 (a) (a vehicle whose locomotor is moving loses any disguise)
        if let Some(ob) = w.obj_mut(id) {
            ob.cloak.disguise = None;
        }
        applied = true;
    }
    // spec: S-0516 claim 33 (b) and quirks.mirage_adjacent_check_cadence (ticks not a multiple of 8; off: multiples of 8)
    let due = if w.rules().quirks.mirage_adjacent_check_cadence {
        tick.rem_euclid(8) != 0
    } else {
        tick.rem_euclid(8) == 0
    };
    if due {
        // spec: S-0516 claim 33 (b) (on the bridge layer if it stands on a bridge)
        let l = if layer == Layer::Deck {
            Layer::Deck
        } else {
            Layer::Ground
        };
        let blink = w.rules().cloak.blink;
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
            let n = glasnost_map::CellXY::new(c.x + dx, c.y + dy);
            // spec: S-0516 claim 33 (b) (only the first infantry unit the cell lists)
            let first = w
                .cells
                .list(n, l)
                .iter()
                .copied()
                .find(|&x| w.obj(x).is_some_and(|ob| ob.kind == Kind::Infantry));
            let Some(x) = first else { continue };
            let xo = w.obj(x).and_then(|ob| ob.owner);
            // spec: S-0516 claim 33 (b) (not allied with the vehicle's owner, neutral counting as not allied: the disguise is lost, the lockout becomes the infantry-blink time)
            if !w.allied(owner, xo) {
                if let Some(ob) = w.obj_mut(id) {
                    ob.cloak.disguise = None;
                    ob.cloak.lockout_until = tick.saturating_add(blink);
                }
                applied = true;
                break;
            }
        }
    }
    if applied || !still {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    // spec: S-0516 claim 33 (c) (not disguised, still, in contact with nothing, the lockout run out)
    if o.cloak.disguise.is_some() || o.contact.is_some() || tick < o.cloak.lockout_until {
        return;
    }
    let list = w.rules().cloak.mirage.clone();
    let n = list.len() as i32;
    if n == 0 {
        // spec: S-0516 Edge cases (an empty default-disguise list: no disguise is taken, no draw)
        return;
    }
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0516 claim 33 (c) (one ranged draw R(0, n − 1); the entry, with house none)
    let k = w.rng.ranged(0, n - 1, half);
    let what = list.get(k as usize).copied().map(DisguiseAs::Terrain);
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.disguise = Some(Disguise {
            what,
            house: None,
            since: tick,
        });
    }
}

/// A vehicle whose type disguises when still fired and created a
/// projectile (S-0516 claim 39): its fake blink is the weapon's fake-blink
/// time.
pub fn fired(w: &mut World, firer: ObjectId, weapon: WeaponId) {
    let Some(o) = w.obj(firer) else { return };
    let (_, _, still) = kinds(w, firer);
    // spec: S-0516 claim 39 (a vehicle whose type disguises when still; infantry never)
    if o.kind != Kind::Vehicle || !still {
        return;
    }
    let t = w.rules().combat.weapon(weapon).disguise_fake_blink;
    let tick = w.tick;
    if let Some(ob) = w.obj_mut(firer) {
        ob.cloak.blink_until = tick.saturating_add(t);
    }
}

/// Whether a unit's fake blink runs (S-0516 claim 39).
pub fn blinking(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| w.tick < o.cloak.blink_until)
}

/// A disguise-flag warhead's projectile detonates on its target (S-0516
/// claim 31): the firer copies the target as its kind allows.
pub fn disguise_hit(w: &mut World, firer: Option<ObjectId>, target: Option<Target>) {
    let Some(f) = firer else { return };
    let Some(fo) = w.obj(f) else { return };
    let fkind = fo.kind;
    let tick = w.tick;
    let r = w.rules();
    let copy = match (fkind, target) {
        (Kind::Infantry, Some(Target::Object(t))) => {
            let Some(to) = w.obj(t) else { return };
            // spec: S-0516 claim 31 (an infantry firer copies only infantry: the target's disguise, else its type and owner)
            if to.kind != Kind::Infantry {
                return;
            }
            match to.cloak.disguise {
                Some(d) => Some((d.what, d.house)),
                None => Some((Some(DisguiseAs::Infantry(to.type_index)), to.owner)),
            }
        }
        (Kind::Vehicle, Some(Target::Object(t))) => {
            let Some(to) = w.obj(t) else { return };
            match (to.kind, to.cloak.disguise) {
                // spec: S-0516 claim 31 (a vehicle firer: a disguised vehicle's disguise)
                (Kind::Vehicle, Some(d)) => Some((d.what, d.house)),
                // spec: S-0516 claim 31 (a vehicle firer: a terrain object, with house none)
                (Kind::Terrain, _) => Some((Some(DisguiseAs::Terrain(to.type_index)), None)),
                _ => None,
            }
        }
        (Kind::Vehicle, Some(Target::Cell(c))) => {
            // spec: S-0516 claim 31 (a vehicle firer: a rock overlay, with house none)
            w.cells
                .cell(c)
                .map(|cl| cl.overlay)
                .filter(|&ov| r.overlays.get(usize::from(ov)).is_some_and(|o| o.rock))
                .map(|ov| (Some(DisguiseAs::Overlay(ov)), None))
        }
        _ => None,
    };
    let Some((what, house)) = copy else { return };
    if let Some(ob) = w.obj_mut(f) {
        ob.cloak.disguise = Some(Disguise {
            what,
            house,
            since: tick,
        });
    }
}

/// Whether the object's type disguises permanently (S-0516 claim 32).
pub fn perma(w: &World, id: ObjectId) -> bool {
    kinds(w, id).1
}

/// Whether a unit appears disguised to house `h` (S-0516 claim 35).
pub fn appears_disguised(w: &World, id: ObjectId, h: Option<HouseId>) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(d) = o.cloak.disguise else {
        return false;
    };
    // spec: S-0516 claim 35 (a disguised building or aircraft appears disguised to every house); Q-0728 (interim: applied unchanged)
    if matches!(o.kind, Kind::Building | Kind::Aircraft) {
        return true;
    }
    // spec: S-0516 claim 35 (owner not allying H; no detector coverage of H on its cell; disguise house H, none or one H allies)
    !w.allied(o.owner, h)
        && !detected(w, h, centre_cell(w, id))
        && (d.house.is_none() || d.house == h || w.allied(h, d.house))
}

/// Seam: S-0309 claim 25 (g), the neutral and special houses' draw that
/// may see through a blinking disguise. Not built: such a searcher never
/// sees through, and no draw is taken.
pub fn neutral_sees_through(w: &World, searcher: ObjectId, target: ObjectId) -> bool {
    let _ = (w, searcher, target);
    false
}
