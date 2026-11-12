//! Weapon choice (S-0308): the weapon in each slot of an object, the
//! weapon it would use against a target (table P, table N), and the mean
//! weapon damage.

use glasnost_map::terrain::land;

use crate::combat::data::{SLOTS, WeaponId};
use crate::combat::veterancy::{self, Rank};
use crate::rules::{Behaviour, Kind};
use crate::world::{Layer, ObjectId, Target, World};

/// The in-the-air answer of S-0501 (claim 18, Terms) and S-0502 claim 30:
/// an aircraft or a jumpjet unit in flight. An object of the set-A or set-B
/// rocket type answers by its flight phase (S-0506 claim 22); for every
/// other object the answer is [`aircraft_in_air`]. An object that is
/// neither, a falling or raised unit among them, answers false here; S-0308's
/// *airborne* is [`airborne`].
pub fn in_air(w: &World, id: ObjectId) -> bool {
    if let Some(o) = w.obj(id)
        && o.kind == Kind::Aircraft
    {
        let m = &w.rules().missiles;
        let ab = [crate::spawn::RocketSetId::A, crate::spawn::RocketSetId::B]
            .iter()
            .any(|&s| m.set(s).type_index == Some(o.type_index));
        if ab {
            // spec: S-0506 claim 22 (the set-A or set-B type: false before flight and in pause, tilt and raise; true in ascent, cruise and descent; independent of height, speed, health and rail membership)
            use crate::rocket::Phase;
            return o
                .rocket
                .as_ref()
                .and_then(|r| r.phase)
                .is_some_and(|p| matches!(p, Phase::Ascent | Phase::Cruise | Phase::Descent));
        }
        if crate::spawn::rocket_type(w, id).is_some() {
            // spec: S-0506 claim 22 (a distinct set-C type: the ordinary aircraft test, on the map and at least 208 leptons above the terrain, S-0307 claim 26)
            return airborne_by_height(w, id);
        }
    }
    aircraft_in_air(w, id)
}

/// The in-the-air answer of an object that is not of a rocket type: for a
/// flying aircraft S-0307's *airborne* (S-0501 Terms); for a unit moving
/// with the jumpjet locomotor, seized units included, on the map with a
/// height above the surface of at least 208 leptons (S-0502 claim 30).
/// Every other object answers false here, whatever its height: this is
/// not S-0308's *airborne* ([`airborne`]).
pub fn aircraft_in_air(w: &World, id: ObjectId) -> bool {
    // spec: S-0501 Terms (an aircraft's in-the-air answer is S-0307's airborne)
    if crate::air::is_aircraft(w, id) {
        return airborne_by_height(w, id);
    }
    // spec: S-0502 claim 30 (revision 18, Q-1501 answered: a unit moving with the jumpjet locomotor, whatever its own kind and seized units included, is in the air exactly when it is on the map and r ≥ 208 leptons, r above the deck only with the on-deck state; not by its flight state)
    crate::jump::is_jumpjet(w, id) && airborne_by_height(w, id)
}

/// Whether an object is airborne in the sense of S-0308 (Terms,
/// *Airborne*; S-0303 claim 10 and S-0309 use the same test): an aircraft
/// by S-0501 claim 18's in-the-air answer ([`in_air`]), any other object
/// by S-0307's height test ([`airborne_by_height`]), so a falling or raised
/// unit high enough is airborne (#178).
pub fn airborne(w: &World, id: ObjectId) -> bool {
    // spec: S-0308 Terms (Airborne: for aircraft, S-0501 claim 18's in-the-air answer; for other objects, S-0307 Terms airborne, which S-0502 claim 30 confirms for jumpjet units)
    if w.obj(id).is_some_and(|o| o.kind == Kind::Aircraft) {
        in_air(w, id)
    } else {
        airborne_by_height(w, id)
    }
}

/// Whether an object is airborne in the sense of S-0307 (Terms): on the
/// map, with a height above the surface of at least 208 leptons.
pub fn airborne_by_height(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    if !o.on_map {
        return false;
    }
    // spec: S-0307 Terms (airborne: height above the surface at least 208 leptons)
    o.pos.z - w.surface_for(id) >= 208
}

/// Whether an object is a target object of table P: an infantry unit,
/// vehicle, aircraft or building.
pub fn is_target_object(w: &World, t: Target) -> bool {
    match t {
        Target::Object(x) => w.obj(x).is_some_and(|o| o.kind != Kind::Terrain),
        Target::Cell(_) => false,
    }
}

/// The weapon in slot `i` of an object (S-0308 Terms, claim 2): the elite
/// slot's weapon for an elite object when that slot holds one, else the
/// ordinary slot's. A building takes it first from its installed upgrades
/// (claim 12); occupants (claim 13) are not modelled (garrisons are a seam).
pub fn slot_weapon(w: &World, id: ObjectId, i: usize) -> Option<WeaponId> {
    let tc = veterancy::type_combat(w, id)?;
    if i >= SLOTS {
        return None;
    }
    if let Some(u) = upgrade_slot_weapon(w, id, i) {
        return Some(u);
    }
    // spec: S-0308 claim 13 (a garrisoned building whose occupants fire: the weapon of every slot a rule asks about comes from its current occupant, unless an upgrade supplies one)
    if crate::structures::garrison::fires(w, id) {
        return occupant_weapon(w, id);
    }
    // spec: S-0308 claim 2 (elite: the elite slot when it holds a weapon, else the slot; per slot)
    if veterancy::rank(w, id) == Rank::Elite
        && let Some(e) = tc.elite_slots[i]
    {
        return Some(e);
    }
    tc.slots[i]
}

/// The weapon of a garrisoned building's current occupant (S-0308 claim 13):
/// an elite occupant's elite occupying weapon, else (with
/// `quirks.elite_occupant_skips_occupy_weapon`) its own slot 0, else the
/// occupying weapon; any other occupant's occupying weapon, else the weapon
/// in its own slot 0. `None` when no occupant exists.
pub fn occupant_weapon(w: &World, building: ObjectId) -> Option<WeaponId> {
    let occ = crate::structures::garrison::current_occupant(w, building)?;
    let tc = veterancy::type_combat(w, occ)?;
    let elite = veterancy::rank(w, occ) == Rank::Elite;
    if elite {
        // spec: S-0308 claim 13 (an elite occupant: its type's elite occupying weapon if the type has one)
        if let Some(e) = tc.elite_occupy_weapon {
            return Some(e);
        }
        // spec: S-0308 claim 13 and Edge cases (quirks.elite_occupant_skips_occupy_weapon: otherwise the weapon in the occupant's own slot 0, claim 2, even when the type has an occupying weapon; off: the occupying weapon)
        if !w.rules().quirks.elite_occupant_skips_occupy_weapon
            && let Some(o) = tc.occupy_weapon
        {
            return Some(o);
        }
        return slot_weapon(w, occ, 0);
    }
    // spec: S-0308 claim 13 (any other occupant: its type's occupying weapon if the type has one, otherwise the weapon in its own slot 0)
    tc.occupy_weapon.or_else(|| slot_weapon(w, occ, 0))
}

/// A building's weapon-providing upgrade types in provider order (S-0308
/// claim 12 with S-0406 claim 12): positions 1 through the used upgrade count
/// *c* (at most 3), in increasing position number, skipping empty positions.
/// Positions above *c* do not take part even when still occupied, and a count
/// of 0 supplies no provider. Installation history decides nothing: the
/// positions do.
pub fn installed_upgrades(w: &World, id: ObjectId) -> impl Iterator<Item = u32> + '_ {
    let (ups, count) = w
        .obj(id)
        .filter(|o| o.kind == Kind::Building)
        .map(|o| (o.econ.building.upgrades, o.econ.building.upgrade_count))
        .unwrap_or_default();
    // spec: S-0406 claim 12 (weapon provider table: positions 1 through c in increasing order, c from 1 to 3; count 0: no upgrade participates; count above 3: positions 1 to 3 only, the bad-data departure)
    let n = usize::try_from(count.clamp(0, 3)).unwrap_or(0);
    ups.into_iter().take(n).flatten()
}

/// The upgrade type whose weapons gate a building's attacks (S-0215 claim 32
/// with S-0406 claim 12): the type recorded in position 1, only while the
/// used upgrade count is nonzero. An empty position 1 bypasses the gate and
/// never substitutes a higher position.
pub fn gate_upgrade(w: &World, id: ObjectId) -> Option<u32> {
    let b = w
        .obj(id)
        .filter(|o| o.kind == Kind::Building)
        .map(|o| &o.econ.building)?;
    // spec: S-0406 claim 12 (attack gate: applies only when the count is nonzero and position 1 is occupied; it tests that position's type only)
    if b.upgrade_count == 0 {
        return None;
    }
    b.upgrades[0]
}

/// The ordinary slot-`i` weapon of a building type, if any.
pub fn building_type_slot(w: &World, ty: u32, i: usize) -> Option<WeaponId> {
    w.rules()
        .buildings
        .get(ty as usize)
        .and_then(|b| b.combat.slots.get(i).copied().flatten())
}

/// The slot-`i` weapon a building's installed upgrades supply (S-0308
/// claim 12), if any.
fn upgrade_slot_weapon(w: &World, id: ObjectId, i: usize) -> Option<WeaponId> {
    // spec: S-0308 claim 12 (the first installed upgrade whose type has a weapon in slot i; always its ordinary slot, never its elite slot)
    installed_upgrades(w, id).find_map(|u| building_type_slot(w, u, i))
}

/// Whether an object's type is a switching type (S-0308 Terms): a
/// multi-weapon type that is not a gattling type.
pub fn switching(w: &World, id: ObjectId) -> bool {
    veterancy::type_combat(w, id).is_some_and(|tc| tc.multi_weapon && !tc.gattling)
}

/// The weapon number, always one of 0 … 17 (S-0308 claim 5).
pub fn weapon_number(w: &World, id: ObjectId) -> usize {
    let n = w.obj(id).map_or(0, |o| o.combat.weapon_number);
    // spec: S-0308 claim 5 (a value outside 0 … 17 sets it to 0)
    if (0..SLOTS as i32).contains(&n) {
        n as usize
    } else {
        0
    }
}

/// Sets the weapon number (S-0308 claim 5): a value outside 0 … 17 sets 0,
/// and a charge-turret type ignores the request.
pub fn set_weapon_number(w: &mut World, id: ObjectId, n: i32) {
    if veterancy::type_combat(w, id).is_some_and(|tc| tc.charge_turret) {
        // spec: S-0308 claim 5 (a charge-turret type ignores every request to change it)
        return;
    }
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0308 claim 5 (outside 0 … 17: 0)
        o.combat.weapon_number = if (0..SLOTS as i32).contains(&n) { n } else { 0 };
    }
}

/// The current weapon (glossary): the weapon in the slot of the weapon
/// number for a multi-weapon type, else slot 0.
pub fn current_weapon(w: &World, id: ObjectId) -> Option<WeaponId> {
    let multi = veterancy::type_combat(w, id).is_some_and(|tc| tc.multi_weapon);
    if multi {
        slot_weapon(w, id, weapon_number(w, id))
    } else {
        slot_weapon(w, id, 0)
    }
}

/// The versus value of a weapon against a target object (S-0304 Terms):
/// its warhead's value for the object's armour class; `None` when the
/// weapon has no warhead or the target has no armour class (a cell).
pub fn versus(w: &World, weapon: WeaponId, t: Target) -> Option<glasnost_num::F64> {
    let r = w.rules();
    let h = r.combat.warhead_of(weapon)?;
    let class = armour_class(w, t)?;
    Some(h.versus(class))
}

/// The armour class of a target (S-0305): an object's type's class; a cell
/// has none.
pub fn armour_class(w: &World, t: Target) -> Option<usize> {
    let Target::Object(x) = t else { return None };
    let o = w.obj(x)?;
    let r = w.rules();
    match o.kind {
        Kind::Terrain => r.terrain.get(o.type_index as usize).map(|t| t.armour),
        _ => veterancy::type_combat(w, x).map(|tc| tc.armour),
    }
}

/// A water target of table P row 17 (S-0308 Terms): an object that is not
/// in the air, not on a bridge deck, on a water or beach cell.
fn water_target(w: &World, t: Target) -> bool {
    let Target::Object(x) = t else { return false };
    let Some(o) = w.obj(x) else { return false };
    // spec: S-0308 Terms (water target: not airborne in S-0308's sense; #178)
    if airborne(w, x) || o.layer == Layer::Deck {
        return false;
    }
    let l = w.cells.cell(o.pos.cell()).map_or(land::CLEAR, |c| c.land);
    // spec: S-0308 Terms (water target: land type water or beach)
    l == land::WATER || l == land::BEACH
}

/// Table N: water targets by the firer's naval-targeting mode (S-0308).
fn table_n(w: &World, firer: ObjectId, t: Target) -> usize {
    let mode = veterancy::type_combat(w, firer).map_or(0, |tc| tc.naval_targeting);
    let Target::Object(x) = t else { return 0 };
    let tc = veterancy::type_combat(w, x);
    let underwater = tc.is_some_and(|c| c.underwater);
    let organic = tc.is_some_and(|c| c.organic);
    let unnatural = tc.is_some_and(|c| c.unnatural);
    // The target's speed type is its locomotion class (S-0308 Import mapping, S-0211).
    let hover = w
        .obj(x)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .is_some_and(|u| u.class == Some(glasnost_rules::sim::LocomotionClass::Hover));
    // spec: S-0308 table N (modes 0 … 7; "no weapon" answers are slot 0)
    match mode {
        // spec: S-0308 table N (mode 1: 1 against an underwater type)
        1 => usize::from(underwater),
        // spec: S-0308 table N (mode 3: 1 against an organic or unnatural type)
        3 => usize::from(organic || unnatural),
        // spec: S-0308 table N (mode 4: 1 when the target's speed type is not hover and its type is not organic)
        4 => usize::from(!hover && !organic),
        _ => 0,
    }
}

/// The slot an object of a deploy-fire infantry type uses (S-0308 claim 8).
fn deployed_infantry_slot(w: &World, firer: ObjectId) -> Option<usize> {
    let o = w.obj(firer)?;
    if o.kind != Kind::Infantry {
        return None;
    }
    let tc = veterancy::type_combat(w, firer)?;
    if !tc.deploy_fire {
        return None;
    }
    let deployed = o
        .infantry
        .as_ref()
        .and_then(|s| s.seq)
        .is_some_and(crate::infantry::Seq::is_deploy);
    // spec: S-0308 claim 8 (deploying: the deploy weapon index; in an open-topped transport the passenger index; else 0)
    let i = if deployed {
        tc.deploy_fire_weapon
    } else {
        // spec: S-0308 claim 8 (not deploying, inside an open-topped transport with a passenger weapon index not −1: that index; else 0)
        crate::transport::passenger_weapon_index(w, firer).map_or(0, |x| x as i32)
    };
    // spec: S-0308 Edge cases (an index outside 0 … 17 is treated as 0, a departure)
    Some(if (0..SLOTS as i32).contains(&i) {
        i as usize
    } else {
        0
    })
}

/// The weapon slot an object would use against a target (S-0308 claim 3,
/// table P; claims 8 and 9 before it: deployed infantry and deployed
/// vehicles of a deploy-fire type).
// spec: S-0308 F2 and table P rows 13, 18 (slot 2s + a; land-targeting mode 2)
pub fn choose_slot(w: &World, firer: ObjectId, target: Option<Target>) -> usize {
    if let Some(i) = deployed_infantry_slot(w, firer) {
        return i;
    }
    let Some(tc) = veterancy::type_combat(w, firer) else {
        return 0;
    };
    // spec: S-0308 claim 9 (a deployed vehicle whose type has the deploy-fire flag: its deploy weapon index against every target, table P not applied; #177)
    if w.obj(firer).is_some_and(|o| o.kind == Kind::Vehicle)
        && tc.deploy_fire
        && crate::econ::inplace::deployed(w, firer)
    {
        let i = tc.deploy_fire_weapon;
        // spec: S-0308 Edge cases (deploy weapon index not checked: an index outside 0 … 17, −1 included, is treated as 0, a departure)
        return if (0..SLOTS as i32).contains(&i) {
            i as usize
        } else {
            0
        };
    }
    let r = w.rules();
    let q = r.quirks;
    // spec: S-0308 table P row 1 (a switching type: its weapon number)
    if tc.multi_weapon && !tc.gattling {
        return weapon_number(w, firer);
    }
    // spec: S-0308 table P row 2 (a garrisoned building whose occupants fire: slot 0; a switching type follows row 1 above)
    if crate::structures::garrison::fires(w, firer) {
        return 0;
    }
    let passenger = crate::transport::passenger_weapon_index(w, firer);
    if !q.passenger_weapon_needs_secondary
        && let Some(i) = passenger
    {
        // spec: S-0308 Edge cases (passenger_weapon_needs_secondary off: row 4 applies before row 3)
        return i;
    }
    let p = slot_weapon(w, firer, 0);
    let s = slot_weapon(w, firer, 1);
    let sec_never = s.is_some_and(|s| r.combat.weapon(s).never_use);
    // spec: S-0308 table P row 3 (slot 1 or 0 empty, secondary never-use, or no target: 0)
    let (Some(pw), Some(sw), Some(t)) = (p, s, target) else {
        return 0;
    };
    if sec_never {
        return 0;
    }
    if let Some(i) = passenger {
        // spec: S-0308 table P row 4 and claim 10 (a passenger of an open-topped transport whose type's passenger weapon index is not −1: that index)
        return i;
    }
    // spec: S-0308 Terms and S-0609 Terms (a gattling type is a gattling-flagged type that is also a multi-weapon type; only it reads the stage)
    if tc.multi_weapon && tc.gattling {
        // spec: S-0308 claim 7 (slot 2s + 1 against an object in the air when the anti-air test passes, else 2s)
        let stage = w.obj(firer).map_or(0, |o| o.combat.gattling_stage).max(0) as usize;
        let test_slot = if q.gattling_air_test_slot_one {
            Some(sw)
        } else {
            // spec: S-0308 Edge cases (gattling_air_test_slot_one off: the slot 2s + 1)
            slot_weapon(w, firer, 2 * stage + 1)
        };
        let aa = test_slot
            .and_then(|x| r.combat.projectile_of(x))
            .is_some_and(|p| p.aa);
        // spec: S-0308 claim 7 and Terms (an object airborne in S-0308's sense; #178)
        let air = matches!(t, Target::Object(x) if airborne(w, x));
        return (2 * stage + usize::from(air && aa)).min(SLOTS - 1);
    }
    let sec_h = r.combat.warhead_of(sw);
    let pri_h = r.combat.warhead_of(pw);
    let target_obj = match t {
        Target::Object(x) => w.obj(x),
        Target::Cell(_) => None,
    };
    let target_building = target_obj.is_some_and(|o| o.kind == Kind::Building);
    // spec: S-0308 table P row 6 (an airstrike secondary decides alone: 1 against a building that accepts charges and is not both a resource gatherer and a resource destination)
    if sec_h.is_some_and(|h| h.airstrike) {
        let accepts = matches!(t, Target::Object(x) if target_building && {
            let bc = &r.buildings[w.obj(x).map_or(0, |o| o.type_index) as usize].building;
            bc.can_c4 && !bc.gatherer_destination
        });
        return usize::from(accepts);
    }
    // spec: S-0308 table P row 7 (a locomotor primary against a building: 1)
    if pri_h.is_some_and(|h| h.is_locomotor) && target_building {
        return 1;
    }
    // spec: S-0308 table P row 8 (a draining secondary against a drainable, not allied target, while the firer is draining nothing at present, S-0608 claim 4)
    if r.combat.weapon(sw).drain
        && !crate::drain::draining(w, firer)
        && let Target::Object(x) = t
        && veterancy::type_combat(w, x).is_some_and(|c| c.drainable)
        && !w.allied(
            w.obj(firer).and_then(|o| o.owner),
            w.obj(x).and_then(|o| o.owner),
        )
    {
        return 1;
    }
    // spec: S-0308 table P row 9 (an area-fire secondary while the current behaviour is unload)
    if r.combat.weapon(sw).area_fire
        && w.obj(firer).and_then(|o| o.beh.seen()) == Some(Behaviour::Unload)
    {
        return 1;
    }
    // spec: S-0308 table P row 10 (the firer is an overpowered building, S-0518 claim 13: 1)
    if w.obj(firer).is_some_and(|o| o.combat.overpowered) {
        return 1;
    }
    // spec: S-0308 table P row 11 (an allied building target of an overpowerable type and an electric-charge secondary: 1)
    if let Target::Object(x) = t
        && target_building
        && sec_h.is_some_and(|h| h.electric_assault)
        && w.allied(
            w.obj(firer).and_then(|o| o.owner),
            w.obj(x).and_then(|o| o.owner),
        )
        && crate::linked::overpowerable(w, x)
    {
        return 1;
    }
    // Row 12 (kamikaze aircraft) needs the aircraft specs.
    if let Target::Cell(c) = t
        && tc.land_targeting == 2
    {
        // spec: S-0308 table P row 13 (a cell target with land mode 2: not water and outside the water role, or a bridge for a naval firer)
        let cell = w.cells.cell(c);
        let land_type = cell.map_or(land::CLEAR, |c| c.land);
        let water_start = w
            .catalogue
            .as_deref()
            .map_or(i32::MIN, |cat| cat.roles.water);
        let tile = cell.map_or(-1, |c| c.tile);
        // spec: S-0308 Constants (water role range: 14 tile indices)
        let in_water_role = water_start >= 0 && (water_start..water_start + 14).contains(&tile);
        let bridge = w.cells.idx(c).is_some_and(|i| w.cells.deck[i]);
        if (land_type != land::WATER && !in_water_role) || (bridge && tc.naval) {
            return 1;
        }
    }
    // spec: S-0308 table P row 14 (a target that is not a target object: 0)
    if !is_target_object(w, t) {
        return 0;
    }
    let zero = |x: Option<glasnost_num::F64>| x.is_some_and(|v| v.eq_num(glasnost_num::F64::ZERO));
    // spec: S-0308 table P row 15 (the secondary's versus value is zero: 0)
    if zero(versus(w, sw, t)) {
        return 0;
    }
    // spec: S-0308 table P row 16 (the primary's versus value is zero: 1)
    if zero(versus(w, pw, t)) {
        return 1;
    }
    // spec: S-0308 table P row 17 (a water target: table N)
    if water_target(w, t) {
        return table_n(w, firer, t);
    }
    // spec: S-0308 table P rows 18 and 19 and Terms (in the air: airborne in S-0308's sense; #178)
    let air = matches!(t, Target::Object(x) if airborne(w, x));
    // spec: S-0308 table P row 18 (not in the air, land mode 2: 1)
    if !air && tc.land_targeting == 2 {
        return 1;
    }
    let sec_aa = r.combat.projectile_of(sw).is_some_and(|p| p.aa);
    let pri_aa = r.combat.projectile_of(pw).is_some_and(|p| p.aa);
    // spec: S-0308 table P row 19 (in the air and an anti-air secondary: 1; quirks.secondary_wins_against_air off: only when the primary is not anti-air)
    if air && sec_aa && (q.secondary_wins_against_air || !pri_aa) {
        return 1;
    }
    // spec: S-0308 table P row 20 (otherwise 0)
    0
}

/// The weapon an object would use against a target (S-0308, S-0304
/// Terms): the weapon in the slot of [`choose_slot`].
pub fn would_use(w: &World, firer: ObjectId, target: Option<Target>) -> (usize, Option<WeaponId>) {
    let slot = choose_slot(w, firer, target);
    (slot, slot_weapon(w, firer, slot))
}

/// The mean weapon damage M (S-0308 claim 15, F1).
pub fn mean_damage(w: &World, id: ObjectId) -> i32 {
    let r = w.rules();
    // spec: S-0308 F1 (dᵢ = damage + ambient damage, signed 32-bit)
    let d = |x: Option<WeaponId>| {
        x.map(|x| {
            let wr = r.combat.weapon(x);
            wr.damage.wrapping_add(wr.ambient)
        })
    };
    if switching(w, id) {
        // spec: S-0308 claim 15 (a switching type: d of its weapon number's slot, 0 when empty)
        return d(slot_weapon(w, id, weapon_number(w, id))).unwrap_or(0);
    }
    match (d(slot_weapon(w, id, 0)), d(slot_weapon(w, id, 1))) {
        // spec: S-0308 F1 (M = trunc((d₀ + d₁) / 2) when both hold a weapon)
        (Some(a), Some(b)) => a.wrapping_add(b) / 2,
        (Some(a), None) | (None, Some(a)) => a,
        (None, None) => 0,
    }
}

/// A healer for acquisition (S-0308 claim 15): mean weapon damage below 0.
pub fn healer_for_acquisition(w: &World, id: ObjectId) -> bool {
    mean_damage(w, id) < 0
}

/// Armed in the sense of S-0214 (Terms): a weapon in the slot it currently
/// uses (the slot of table P with no target).
pub fn armed(w: &World, id: ObjectId) -> bool {
    would_use(w, id, None).1.is_some()
}
