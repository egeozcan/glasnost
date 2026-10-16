//! Spies and infiltration (S-0510): the single effect an infiltrated
//! building has, chosen by its type (claim 3), the money theft (claim 6),
//! the infiltrated-factory flags (claim 5), the used-up spy (claim 2) and the
//! announcements' event (claim 11).

use crate::combat::effects::{Effect, InfiltrationRow};
use crate::econ::{TypeRef, tech};
use crate::rules::{FactoryKind, Kind};
use crate::world::{HouseId, ObjectId, World};

/// A spy type (S-0510 Terms, `Agent`).
pub fn is_spy(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .filter(|o| o.kind == Kind::Infantry)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        // spec: S-0510 Import mapping (Agent absent: no)
        .is_some_and(|u| u.raw.agent.unwrap_or(false))
}

/// The effect rows of a building's type, in table order (claim 3). With
/// `quirks.infiltration_single_effect` on only the first matching row
/// applies; off, every matching row does, and E3 requires only tech-lab-list
/// membership.
pub fn rows(w: &World, b: ObjectId) -> Vec<InfiltrationRow> {
    let Some(o) = w.obj(b) else {
        return Vec::new();
    };
    if o.kind != Kind::Building {
        return Vec::new();
    }
    let r = w.rules();
    let idx = o.type_index as usize;
    let single = r.quirks.infiltration_single_effect;
    let t = TypeRef {
        kind: Kind::Building,
        index: o.type_index,
    };
    let tt = r.econ.tech.of(t).cloned().unwrap_or_default();
    let radar = r.econ.shroud.radar.get(idx).copied().unwrap_or(false);
    // spec: S-0510 claim 3 (row E2: the type's own base power output above 0; upgrades do not count)
    let output = r.econ.power.buildings.get(idx).map_or(0, |p| p.output);
    let building = &r.buildings[idx];
    let mut out = Vec::new();
    let mut add = |row: InfiltrationRow, matches: bool| {
        if matches && (!single || out.is_empty()) {
            out.push(row);
        }
    };
    // spec: S-0510 claim 3 (E1: the radar flag)
    add(InfiltrationRow::Radar, radar);
    // spec: S-0510 claim 3 (E2: base power output above 0)
    add(InfiltrationRow::Power, output > 0);
    // spec: S-0510 claim 3 (E3: on the tech-lab list, without the radar flag and with base power output 0 or below; with the single-effect switch off, list membership alone)
    add(
        InfiltrationRow::Tech,
        tt.tech_lab && (!single || (!radar && output <= 0)),
    );
    // spec: S-0510 claim 3 (E4: a first superweapon; the second is never reset)
    add(
        InfiltrationRow::Superweapon,
        r.superweapons
            .building_sw
            .get(idx)
            .is_some_and(|f| f[0].is_some()),
    );
    // spec: S-0510 claim 3 (E5: storage above 0)
    add(InfiltrationRow::Cash, building.raw.storage.unwrap_or(0) > 0);
    // spec: S-0510 claim 3 (E6: a factory building of vehicles, naval yards included; E7: of infantry; aircraft and building factories give nothing)
    add(
        InfiltrationRow::VehicleFactory,
        building.factory == Some(FactoryKind::Vehicle),
    );
    add(
        InfiltrationRow::InfantryFactory,
        building.factory == Some(FactoryKind::Infantry),
    );
    out
}

/// A spy's entry into a building after the building's entry raise (claims
/// 1 to 9): the effect of the building's type against the building's owner,
/// then the spy's destroyed raise and its removal. Returns whether the spy
/// left the map.
pub fn infiltrate(w: &mut World, spy: ObjectId, b: ObjectId) -> bool {
    let (Some(s), Some(bo)) = (w.obj(spy), w.obj(b)) else {
        return w.obj(spy).is_none_or(|o| !o.on_map);
    };
    // spec: S-0510 claim 9 (the infiltrating house is the spy's owner when the effect is selected, after the building's entry raise; the victim house the building's owner at that same point)
    let (infiltrator, victim, pos) = (s.owner, bo.owner, bo.pos);
    // spec: S-0510 claim 3 (the infiltrating house is the victim house itself: no effect, the spy is still used up; only identities are compared, so an allied building suffers in full)
    let applied = if infiltrator.is_some() && infiltrator == victim {
        Vec::new()
    } else {
        rows(w, b)
    };
    let satellite = victim.is_some_and(|v| crate::econ::shroud::spy_satellite_active(w, v));
    if let (Some(t), Some(v)) = (infiltrator, victim)
        && t != v
    {
        for &row in &applied {
            apply(w, row, b, t, v);
        }
    }
    // spec: S-0510 claim 11 (the infiltrated radar event and the announcement lines for the local player when the two houses differ, also when no row applies)
    if infiltrator != victim && (infiltrator.is_some() || victim.is_some()) {
        w.effects.push(Effect::Infiltrated {
            building: b,
            infiltrator,
            victim,
            rows: applied,
            satellite,
            pos,
        });
    }
    // spec: S-0510 claim 9 (after the effect and before its removal the spy's own tag is raised with the destroyed-by-anything code)
    crate::triggers::destroyed_by_anything(w, spy);
    // spec: S-0510 claim 2 (every entry removes the spy in the same update: not a death, no killing hit, no experience, no death sequence or survivor)
    crate::objects::remove(w, spy);
    true
}

/// One effect row (claims 3 to 6).
fn apply(w: &mut World, row: InfiltrationRow, b: ObjectId, thief: HouseId, victim: HouseId) {
    match row {
        // spec: S-0510 claim 3 row E1 (the victim's whole map is re-covered, unless the victim has an active spy satellite, S-0409 claim 15)
        InfiltrationRow::Radar => crate::econ::shroud::radar_infiltrated(w, victim),
        // spec: S-0510 claim 3 row E2 and claim 4 (a timed blackout of the victim of the game-wide spy blackout length, starting at the spy's update of the arrival tick, S-0405 claim 6)
        InfiltrationRow::Power => {
            let d = w.rules().econ.power.spy_blackout;
            crate::econ::power::start_blackout(w, victim, d);
        }
        // spec: S-0510 claim 3 row E3 (the infiltrating house gains the stolen-tech flag the type's base-planning side chooses, S-0407 claim 6)
        InfiltrationRow::Tech => {
            let exclusions = w.rules().quirks.infiltration_single_effect;
            tech::infiltrated_with(w, thief, b, exclusions);
        }
        // spec: S-0510 claim 3 row E4 (the victim's record for the building type's first superweapon is reset, S-0512 claim 15)
        InfiltrationRow::Superweapon => crate::superweapons::charge::spy_reset(w, b),
        InfiltrationRow::Cash => steal(w, thief, victim),
        // spec: S-0510 claim 5 (the infiltrating house's infiltrated-vehicle-factory flag is set and stays set)
        InfiltrationRow::VehicleFactory => {
            if let Some(h) = w.houses.get_mut(usize::from(thief.0)) {
                h.infiltrated_vehicle_factory = true;
            }
        }
        // spec: S-0510 claim 5 (the infiltrated-infantry-factory flag)
        InfiltrationRow::InfantryFactory => {
            if let Some(h) = w.houses.get_mut(usize::from(thief.0)) {
                h.infiltrated_infantry_factory = true;
            }
        }
    }
}

/// The theft amount *s* of F1 for a victim balance of available credits
/// `b` (the thief gains it, the victim pays it).
pub fn theft_amount(w: &World, b: i32) -> i32 {
    let mode = w.rules().mode;
    let f = w.rules().econ.power.spy_steal;
    // spec: S-0510 F1 (the product B × f in double precision, rounded under quirks.chop_rounding, the single widened exactly)
    // spec: S-0510 F1 (s: the common conversion of S-0205 claim 7: truncated toward zero, reduced modulo 2³² when the magnitude is below 2⁶³, zero otherwise)
    glasnost_num::F64::from_i64(i64::from(b), mode)
        .mul(f, mode)
        .to_i32_common()
}

/// Money theft (claim 6, F1): the infiltrating house's balance rises by *s*
/// and the victim pays *s*, a payment above its balance emptying it.
fn steal(w: &mut World, thief: HouseId, victim: HouseId) {
    let available = crate::econ::production::available_credits(w, victim);
    let mut s = theft_amount(w, available);
    let v = w.houses.get(usize::from(victim.0)).map_or(0, |h| h.credits);
    if !w.rules().quirks.spy_theft_unbacked_gain {
        // spec: S-0510 Edge cases (quirks.spy_theft_unbacked_gain off: the amount is clamped to 0 ≤ s ≤ max(0, the victim's credit balance) before either balance changes)
        s = s.clamp(0, v.max(0));
    }
    if let Some(h) = w.houses.get_mut(usize::from(thief.0)) {
        // spec: S-0510 F1 (T′ is the signed 32-bit result congruent to T + s modulo 2³²)
        h.credits = h.credits.wrapping_add(s);
    }
    if let Some(h) = w.houses.get_mut(usize::from(victim.0)) {
        // spec: S-0510 F1 (V′ is the signed 32-bit result congruent to V − min(s, V) modulo 2³²)
        h.credits = h.credits.wrapping_sub(s.min(v));
    }
}

/// Whether the sidebar shows the alternate icon of building, vehicle or
/// infantry type `t` to `house` (S-0007 claim 25, the part the simulation
/// answers): the type is trainable and the house has infiltrated a barracks
/// and the type is an infantry type, or it has infiltrated a war factory and
/// the type is a vehicle type or a building type with an undeploy-into
/// vehicle. Aircraft and other building types always show the main icon.
pub fn alternate_icon(w: &World, house: HouseId, t: TypeRef) -> bool {
    let Some(h) = w.houses.get(usize::from(house.0)) else {
        return false;
    };
    let r = w.rules();
    let (trainable, undeploys) = match t.kind {
        Kind::Infantry => (
            r.infantry.get(t.index as usize).map(|u| u.combat.trainable),
            false,
        ),
        Kind::Vehicle => (
            r.vehicles.get(t.index as usize).map(|u| u.combat.trainable),
            false,
        ),
        Kind::Building => (
            r.buildings
                .get(t.index as usize)
                .map(|b| b.combat.trainable),
            r.buildings
                .get(t.index as usize)
                .is_some_and(|b| b.raw.undeploys_into.is_some()),
        ),
        _ => return false,
    };
    // spec: S-0007 claim 25 (the type's trainable flag is set)
    if !trainable.unwrap_or(false) {
        return false;
    }
    match t.kind {
        // spec: S-0007 claim 25 (the house has infiltrated a barracks and the type is an infantry type)
        Kind::Infantry => h.infiltrated_infantry_factory,
        // spec: S-0007 claim 25 (it has infiltrated a war factory and the type is a vehicle type or a building type whose undeploy-into vehicle is set)
        Kind::Vehicle => h.infiltrated_vehicle_factory,
        _ => h.infiltrated_vehicle_factory && undeploys,
    }
}
