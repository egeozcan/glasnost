//! Cloak fields (S-0516 claims 19 and 20): a generator's circle grows one
//! ring per tick while it is in operation and shrinks one ring per tick
//! while it is not; removal and capture make it vanish at once.

use std::collections::BTreeSet;

use glasnost_map::CellXY;

use super::{Generator, centre_cell, field_circle, type_radius};
use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, World};

/// Whether a building's type is a cloak generator.
fn is_generator(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.kind == Kind::Building
            && w.rules()
                .buildings
                .get(o.type_index as usize)
                // spec: S-0516 Import mapping (CloakGenerator absent: no)
                .is_some_and(|b| b.raw.cloak_generator.unwrap_or(false))
    })
}

/// The type radius of a generator, 0 or more (Q-0724 interim: a negative
/// radius gives no field).
fn full_radius(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| type_radius(w, o.type_index)).max(0)
}

/// The cells of the circle of radius `r` around `centre`.
fn cells(centre: CellXY, r: i32) -> BTreeSet<CellXY> {
    field_circle(r)
        .into_iter()
        .map(|(dx, dy)| CellXY::new(centre.x + dx, centre.y + dy))
        .collect()
}

/// A generator building entered play: its field starts empty.
pub fn placed(w: &mut World, id: ObjectId) {
    if !is_generator(w, id) {
        return;
    }
    let c = centre_cell(w, id);
    if let Some(ob) = w.obj_mut(id)
        && let Some(h) = ob.owner
        && ob.cloak.field.is_none()
    {
        ob.cloak.field = Some(Generator {
            house: h,
            centre: c,
            radius: 0,
        });
    }
}

/// Re-evaluates every object listed in the given cells (claim 20).
fn reevaluate(w: &mut World, cells: &BTreeSet<CellXY>) {
    for &c in cells {
        super::machine::reevaluate_cell(w, c, false);
    }
}

/// Whether another generator of house `h` covers `c` now (the fixed
/// behaviour of `quirks.cloak_field_overlap_loss`).
fn other_covers(w: &World, id: ObjectId, h: HouseId, c: CellXY) -> bool {
    w.objects.values().any(|o| {
        o.id != id
            && o.cloak.field.is_some_and(|g| {
                g.house == h
                    && g.radius > 0
                    && field_circle(g.radius).contains(&(c.x - g.centre.x, c.y - g.centre.y))
            })
    })
}

/// Takes cells out of house `h`'s field (claim 20, overlaps).
fn remove(w: &mut World, id: ObjectId, h: HouseId, gone: &BTreeSet<CellXY>) {
    let overlap_loss = w.rules().quirks.cloak_field_overlap_loss;
    for &c in gone {
        // spec: S-0516 claim 20 and quirks.cloak_field_overlap_loss (a flag, not a count: shared cells leave the field)
        if !overlap_loss && other_covers(w, id, h, c) {
            continue;
        }
        w.cloak.fields.remove(&(h.0, c));
    }
}

/// A generator's turn (claim 20): one ring more while in operation, one
/// ring less while not.
pub fn turn(w: &mut World, id: ObjectId) {
    let Some(g) = w.obj(id).and_then(|o| o.cloak.field) else {
        return;
    };
    let full = full_radius(w, id);
    let on = crate::econ::power::in_operation(w, id);
    if on && g.radius < full {
        // spec: S-0516 claim 20 (growth: from radius 0, one ring per tick up to the type's radius); Q-0729 (interim: no ring interruption by buildings)
        let r = g.radius + 1;
        let old = cells(g.centre, g.radius);
        let new: BTreeSet<CellXY> = cells(g.centre, r).difference(&old).copied().collect();
        for &c in &new {
            w.cloak.fields.insert((g.house.0, c));
        }
        set_radius(w, id, r);
        // spec: S-0516 claim 20 (every object on a newly covered cell is re-evaluated)
        reevaluate(w, &new);
    } else if !on && g.radius > 0 {
        // spec: S-0516 claim 20 (shrinking: one ring per tick to 0; cells outside the circle leave at once)
        let r = g.radius - 1;
        let keep = cells(g.centre, r);
        let gone: BTreeSet<CellXY> = cells(g.centre, g.radius)
            .difference(&keep)
            .copied()
            .collect();
        remove(w, id, g.house, &gone);
        set_radius(w, id, r);
        reevaluate(w, &gone);
        if r == 0 {
            restart_neighbours(w, id);
        }
    }
}

fn set_radius(w: &mut World, id: ObjectId, r: i32) {
    if let Some(ob) = w.obj_mut(id)
        && let Some(g) = ob.cloak.field.as_mut()
    {
        g.radius = r;
    }
}

/// When a shrinking generator reaches radius 0, every other powered, idle
/// generator at its full radius whose centre is closer than 2 (*r* + 2)
/// cells restarts its growth from 0, whatever its owner (claim 20). Q-0729
/// (interim: Euclidean distance between centre cells).
fn restart_neighbours(w: &mut World, id: ObjectId) {
    let Some(g) = w.obj(id).and_then(|o| o.cloak.field) else {
        return;
    };
    let r = full_radius(w, id);
    // spec: S-0516 claim 20 and Constants (generator restart reach 2 (r + 2) cells)
    let reach = i64::from(2 * (r + 2));
    let others: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.id != id && o.cloak.field.is_some())
        .map(|o| o.id)
        .collect();
    for x in others {
        let Some(g2) = w.obj(x).and_then(|o| o.cloak.field) else {
            continue;
        };
        let (dx, dy) = (
            i64::from(g2.centre.x - g.centre.x),
            i64::from(g2.centre.y - g.centre.y),
        );
        if dx * dx + dy * dy >= reach * reach {
            continue;
        }
        // spec: S-0516 claim 20 (powered and idle at its full radius: restarts from 0)
        if crate::econ::power::in_operation(w, x) && g2.radius == full_radius(w, x) && g2.radius > 0
        {
            set_radius(w, x, 0);
        }
    }
}

/// The field vanishes at once (claim 20: removal, sale, destruction,
/// capture). `restart`: the neighbours restart as when a shrinking field
/// reaches 0 (Q-0729 interim for removal).
pub fn vanish(w: &mut World, id: ObjectId, restart: bool) {
    let Some(g) = w.obj(id).and_then(|o| o.cloak.field) else {
        return;
    };
    if g.radius <= 0 {
        return;
    }
    let gone = cells(g.centre, g.radius);
    remove(w, id, g.house, &gone);
    set_radius(w, id, 0);
    reevaluate(w, &gone);
    if restart {
        restart_neighbours(w, id);
    }
}

/// A captured generator (claim 20): its field vanishes at once and grows
/// again for the new owner while it is in operation.
pub fn captured(w: &mut World, id: ObjectId) {
    if w.obj(id).is_none_or(|o| o.cloak.field.is_none()) {
        return;
    }
    vanish(w, id, false);
    if let Some(ob) = w.obj_mut(id) {
        match ob.owner {
            Some(h) => {
                if let Some(g) = ob.cloak.field.as_mut() {
                    g.house = h;
                }
            }
            None => ob.cloak.field = None,
        }
    }
}
