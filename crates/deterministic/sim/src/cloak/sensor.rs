//! Sensor coverage and disguise-detector coverage (S-0516 claims 21 to 23
//! and 40).

use glasnost_map::CellXY;

use super::{Cover, centre_cell, type_radius};
use crate::combat::veterancy::type_combat;
use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, World};

/// Which per-house count a change goes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    /// Sensor coverage (claim 21).
    Sensors,
    /// Disguise-detector coverage (claim 40).
    Detectors,
}

/// Adds `delta` to house `h`'s counts over the disc of radius `r` around
/// `centre`; `floor` keeps a count from going below 0 (claim 22). Every
/// sensor change re-evaluates the mobile objects of the cell's ground layer
/// (claim 21).
pub fn change(
    w: &mut World,
    which: Coverage,
    h: HouseId,
    centre: CellXY,
    r: i32,
    delta: i32,
    floor: bool,
) {
    let mut cells = Vec::new();
    for c in disc_on_map(w, centre, r) {
        let map = match which {
            Coverage::Sensors => &mut w.cloak.sensors,
            Coverage::Detectors => &mut w.cloak.detectors,
        };
        let now = map.get(&(h.0, c)).copied().unwrap_or(0);
        let mut v = now.saturating_add(delta);
        if floor {
            // spec: S-0516 claim 22 (a mobile sensor's removal never takes a count below 0)
            v = v.max(0);
        }
        if v == 0 {
            map.remove(&(h.0, c));
        } else {
            map.insert((h.0, c), v);
        }
        cells.push(c);
    }
    if which == Coverage::Sensors {
        for c in cells {
            // spec: S-0516 claim 21 (each vehicle, infantry unit and aircraft of the cell's ground layer is re-evaluated)
            super::machine::reevaluate_cell(w, c, true);
        }
    }
}

/// The map's cells of the sensor disc of radius `r` around `centre`, in
/// the order of [`super::sensor_disc`] (rows of dy, each in increasing dx). Only
/// the part of the disc's square that holds legal cells is walked, so a
/// huge radius costs no more than the map (#170).
fn disc_on_map(w: &World, centre: CellXY, r: i32) -> Vec<CellXY> {
    let mut out = Vec::new();
    if r <= 0 {
        return out;
    }
    let r = i64::from(r);
    let r2 = r * r;
    let side = i64::from(w.cells.footprint().coordinate_side());
    let (cx, cy) = (i64::from(centre.x), i64::from(centre.y));
    // spec: S-0516 Terms (sensor disc: −R ≤ dx ≤ R − 1, −R ≤ dy ≤ R − 1, dx² + dy² < R²); only cells inside the coordinate square 0 … side − 1 can be on the map (S-0009 claim 2)
    let (dx0, dx1) = ((-r).max(-cx), (r - 1).min(side - 1 - cx));
    let (dy0, dy1) = ((-r).max(-cy), (r - 1).min(side - 1 - cy));
    for dy in dy0..=dy1 {
        for dx in dx0..=dx1 {
            if dx * dx + dy * dy >= r2 {
                continue;
            }
            let (Ok(x), Ok(y)) = (i32::try_from(cx + dx), i32::try_from(cy + dy)) else {
                continue;
            };
            let c = CellXY::new(x, y);
            if w.cells.idx(c).is_some() {
                out.push(c);
            }
        }
    }
    out
}

/// A mobile object's sensors-sight *R* (claim 22).
fn sight(w: &World, id: ObjectId) -> i32 {
    // spec: S-0516 Import mapping (SensorsSight absent: 0)
    type_combat(w, id).map_or(0, |t| t.sensors_sight)
}

/// A mobile sensor entered play (claim 22): its disc is added for its
/// owner.
pub fn mobile_entered(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let Some(h) = o.owner else { return };
    let r = sight(w, id);
    // spec: S-0516 claim 22 (a vehicle, infantry unit or aircraft with a non-zero sensors-sight)
    if !o.kind.is_mobile() || r == 0 || o.cloak.disc.is_some() {
        return;
    }
    let c = centre_cell(w, id);
    change(w, Coverage::Sensors, h, c, r, 1, false);
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.disc = Some((h, c));
    }
}

/// A mobile sensor's cell-arrival update (claim 22): the disc is removed
/// around the last recorded coverage cell and added around the current one,
/// even when the two are the same cell (each change re-evaluates its cells,
/// claim 21).
pub fn mobile_moved(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let Some((h, old)) = o.cloak.disc else { return };
    let c = centre_cell(w, id);
    let r = sight(w, id);
    change(w, Coverage::Sensors, h, old, r, -1, true);
    change(w, Coverage::Sensors, h, c, r, 1, false);
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.disc = Some((h, c));
    }
}

/// A mobile sensor leaves play (claim 22): with
/// `quirks.mobile_sensor_removal_centre` the disc is removed around its
/// current cell, which may differ from where it was last added (the drift
/// of a sensor that crossed a boundary without completing the movement);
/// off, around the cell where it was last added.
pub fn mobile_left(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let Some((h, added)) = o.cloak.disc else {
        return;
    };
    let c = if w.rules().quirks.mobile_sensor_removal_centre {
        // spec: S-0516 claim 22 and quirks.mobile_sensor_removal_centre (removed around the current position cell, destruction and boarding alike)
        centre_cell(w, id)
    } else {
        // spec: S-0516 Edge cases (mobile_sensor_removal_centre off: remove the disc actually last added)
        added
    };
    let r = sight(w, id);
    // spec: S-0516 claim 22 (removal never takes a mobile sensor's count below 0)
    change(w, Coverage::Sensors, h, c, r, -1, true);
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.disc = None;
    }
}

/// A mobile sensor changed owner (claim 22): the disc moves from the old
/// owner to the new one.
pub fn mobile_owner_changed(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let Some((h, c)) = o.cloak.disc else { return };
    let new = o.owner;
    let r = sight(w, id);
    change(w, Coverage::Sensors, h, c, r, -1, true);
    match new {
        Some(n) => {
            change(w, Coverage::Sensors, n, c, r, 1, false);
            if let Some(ob) = w.obj_mut(id) {
                ob.cloak.disc = Some((n, c));
            }
        }
        None => {
            if let Some(ob) = w.obj_mut(id) {
                ob.cloak.disc = None;
            }
        }
    }
}

/// A building type's array and detector radii: (sensor array's
/// sensors-sight, detector's detection range), `None` where the type is
/// not one.
fn building_radii(w: &World, id: ObjectId) -> (Option<i32>, Option<i32>) {
    let Some(o) = w.obj(id) else {
        return (None, None);
    };
    if o.kind != Kind::Building {
        return (None, None);
    }
    let Some(b) = w.rules().buildings.get(o.type_index as usize) else {
        return (None, None);
    };
    let tc = type_combat(w, id);
    // spec: S-0516 Import mapping (SensorArray absent: no; SensorsSight absent: 0)
    let array = b
        .raw
        .sensor_array
        .unwrap_or(false)
        .then(|| tc.map_or(0, |t| t.sensors_sight));
    // spec: S-0516 Import mapping (DetectDisguise absent: no; DetectDisguiseRange absent: 0)
    let detector = tc
        .is_some_and(|t| t.detect_disguise)
        .then(|| b.raw.detect_disguise_range.unwrap_or(0));
    (array, detector)
}

/// Adds a building's coverage for its owner and records it.
fn activate(w: &mut World, id: ObjectId) {
    let Some(h) = w.obj(id).and_then(|o| o.owner) else {
        return;
    };
    let c = centre_cell(w, id);
    let (array, detector) = building_radii(w, id);
    if let Some(r) = array
        && w.obj(id).is_some_and(|o| o.cloak.array.is_none())
    {
        change(w, Coverage::Sensors, h, c, r, 1, false);
        if let Some(ob) = w.obj_mut(id) {
            ob.cloak.array = Some(Cover {
                house: h,
                centre: c,
                radius: r,
            });
        }
    }
    if let Some(r) = detector
        && w.obj(id).is_some_and(|o| o.cloak.detector.is_none())
    {
        change(w, Coverage::Detectors, h, c, r, 1, false);
        if let Some(ob) = w.obj_mut(id) {
            ob.cloak.detector = Some(Cover {
                house: h,
                centre: c,
                radius: r,
            });
        }
    }
}

/// Undoes a building's recorded coverage exactly.
fn deactivate(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let (array, detector) = (o.cloak.array, o.cloak.detector);
    if let Some(a) = array {
        change(w, Coverage::Sensors, a.house, a.centre, a.radius, -1, false);
    }
    if let Some(d) = detector {
        change(
            w,
            Coverage::Detectors,
            d.house,
            d.centre,
            d.radius,
            -1,
            false,
        );
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.array = None;
        ob.cloak.detector = None;
    }
}

/// A building's first placement (claims 23 and 40): a sensor array or
/// detector in operation adds its coverage for its owner, once.
pub fn building_placed(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Building || o.cloak.placed {
        return;
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.placed = true;
    }
    // spec: S-0516 claim 23 and quirks.sensor_building_activation_once (at the first placement, only if in operation then)
    if w.rules().quirks.sensor_building_activation_once && crate::econ::power::in_operation(w, id) {
        activate(w, id);
    }
}

/// A building's turn with `quirks.sensor_building_activation_once` off:
/// its coverage follows *in operation* (S-0516 Edge cases).
pub fn building_turn(w: &mut World, id: ObjectId) {
    if w.rules().quirks.sensor_building_activation_once {
        return;
    }
    let (array, detector) = building_radii(w, id);
    if array.is_none() && detector.is_none() {
        return;
    }
    let on = crate::econ::power::in_operation(w, id);
    let has = w
        .obj(id)
        .is_some_and(|o| o.cloak.array.is_some() || o.cloak.detector.is_some());
    // spec: S-0516 Edge cases (sensor_building_activation_once off: coverage follows in operation)
    if on && !has {
        activate(w, id);
    } else if !on && has {
        deactivate(w, id);
    }
}

/// A sensor array or detector leaves play (claims 23 and 40).
pub fn building_removed(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Building || !o.cloak.placed {
        return;
    }
    let q = w.rules().quirks;
    if !q.sensor_count_negative {
        // spec: S-0516 Edge cases (sensor_count_negative off: removal undoes exactly what was added)
        deactivate(w, id);
        return;
    }
    let Some(h) = o.owner else {
        deactivate(w, id);
        return;
    };
    let ty = o.type_index;
    let c = centre_cell(w, id);
    let (array, detector) = building_radii(w, id);
    if let Some(r) = array {
        // spec: S-0516 claim 23 and quirks.sensor_array_removal_radius (the current owner's counts over the cloak-radius disc); quirks.sensor_count_negative (no floor)
        let radius = if q.sensor_array_removal_radius {
            type_radius(w, ty)
        } else {
            r
        };
        change(w, Coverage::Sensors, h, c, radius, -1, false);
    }
    if let Some(r) = detector {
        // spec: S-0516 claim 40 (removal over the detection range, no floor)
        change(w, Coverage::Detectors, h, c, r, -1, false);
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.cloak.array = None;
        ob.cloak.detector = None;
    }
}

/// A sensor array or detector changed owner (claims 23 and 40): with
/// `quirks.sensor_building_activation_once` the old owner keeps the
/// coverage; off, it moves with the building (its turn adds it again).
pub fn building_owner_changed(w: &mut World, id: ObjectId) {
    // spec: S-0516 claim 23 (capture: the old owner keeps the coverage, the new owner gets none)
    if w.rules().quirks.sensor_building_activation_once {
        return;
    }
    // spec: S-0516 Edge cases (sensor_building_activation_once off: coverage moves to the new owner on capture)
    deactivate(w, id);
    building_turn(w, id);
}
