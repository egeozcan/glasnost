//! Cloak fields (S-0516 claims 19 and 20): a generator's circle grows one
//! radius per tick while it is in operation and shrinks one radius per tick
//! while it is not; removal, sale, destruction and capture collapse it at
//! once.

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

/// The type radius of a generator, 0 or more: a zero or negative type
/// radius gives no field (S-0516 claim 20, Import mapping).
fn full_radius(w: &World, id: ObjectId) -> i32 {
    // spec: S-0516 claim 20 (a negative effective radius is invalid and treated as zero: no field, no growth or restart effects)
    w.obj(id).map_or(0, |o| type_radius(w, o.type_index)).max(0)
}

/// The cells of the circle of radius `r` around `centre`, in increasing
/// *Y*, then increasing *X* (S-0516 claim 20).
fn cells(centre: CellXY, r: i32) -> Vec<CellXY> {
    let mut v: Vec<CellXY> = field_circle(r)
        .into_iter()
        .map(|(dx, dy)| CellXY::new(centre.x + dx, centre.y + dy))
        .collect();
    // spec: S-0516 claim 20 (cells take effect in increasing Y, then increasing X)
    v.sort_by_key(|c| (c.y, c.x));
    v
}

/// The centre cells of house `h`'s buildings in play (S-0516 claim 20: an
/// own building's centre).
fn own_centres(w: &World, h: HouseId) -> BTreeSet<CellXY> {
    w.objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.on_map && o.owner == Some(h))
        .map(|o| centre_cell(w, o.id))
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
fn reevaluate(w: &mut World, cells: &[CellXY]) {
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

/// Takes cell `c` out of house `h`'s field (claim 20, overlaps); whether it
/// was cleared.
fn clear(w: &mut World, id: ObjectId, h: HouseId, c: CellXY) -> bool {
    // spec: S-0516 claim 20 and quirks.cloak_field_overlap_loss (a flag, not a count: shared cells leave the field)
    if !w.rules().quirks.cloak_field_overlap_loss && other_covers(w, id, h, c) {
        return false;
    }
    w.cloak.fields.remove(&(h.0, c))
}

/// A generator's turn (claim 20): one radius more while in operation, one
/// less while not.
pub fn turn(w: &mut World, id: ObjectId) {
    let Some(g) = w.obj(id).and_then(|o| o.cloak.field) else {
        return;
    };
    let full = full_radius(w, id);
    let on = crate::econ::power::in_operation(w, id);
    let interrupt = w.rules().quirks.cloak_field_building_interrupt;
    if on && g.radius < full {
        // spec: S-0516 claim 20 (growth: radius 1 on the first growth update, then 2, through r, one per tick; the radius increases even when the update is interrupted)
        let r = g.radius + 1;
        set_radius(w, id, r);
        let mut own = if interrupt {
            own_centres(w, g.house)
        } else {
            BTreeSet::new()
        };
        // Q-0792 (interim: the generator's own centre does not end a growth update, so growth begins with the full radius-1 circle as CS18 part D expects)
        own.remove(&g.centre);
        let mut new = Vec::new();
        // spec: S-0516 claim 20 (a growth update examines the whole circle: cells omitted earlier can be covered now)
        for c in cells(g.centre, r) {
            if !w.cloak.fields.insert((g.house.0, c)) {
                // spec: S-0516 claim 20 (already covered building centres do not interrupt again)
                continue;
            }
            new.push(c);
            // spec: S-0516 claim 20 and quirks.cloak_field_building_interrupt (the first newly covered centre cell of an own building is covered, then the rest of the update's circle is omitted)
            if own.contains(&c) {
                break;
            }
        }
        // spec: S-0516 claim 20 (every object on a newly covered cell is re-evaluated)
        reevaluate(w, &new);
    } else if !on && g.radius > 0 {
        // spec: S-0516 claim 20 (shrinking: one radius per tick to 0; cells outside the shrinking circle leave at once)
        let r = g.radius - 1;
        set_radius(w, id, r);
        let keep: BTreeSet<CellXY> = cells(g.centre, r).into_iter().collect();
        let own = if interrupt {
            own_centres(w, g.house)
        } else {
            BTreeSet::new()
        };
        let mut gone = Vec::new();
        let mut interrupted = false;
        // spec: S-0516 claim 20 (a shrink update clears its own ring; cells an earlier interrupted shrink left are not retried)
        for c in cells(g.centre, g.radius) {
            if keep.contains(&c) {
                continue;
            }
            gone.push(c);
            // spec: S-0516 claim 20 and quirks.cloak_field_building_interrupt (a shrink update stops after clearing an own building's centre, the generator's own centre included)
            if clear(w, id, g.house, c) && own.contains(&c) {
                interrupted = true;
                break;
            }
        }
        reevaluate(w, &gone);
        // spec: S-0516 claim 20 (an uninterrupted collapse reaching radius 0 restarts the neighbours; an interruption on that update prevents it)
        if r == 0 && !interrupted {
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

/// The neighbour restart of claim 20: every other in-play, powered, idle
/// generator, of any owner, whose centre cell lies strictly within
/// Δ*X*² + Δ*Y*² < 4(*r* + 2)² of this one's, *r* this generator's type
/// radius: at its full radius it restarts from 0, otherwise its radius
/// decreases by one, floored at 0, and growth resumes.
fn restart_neighbours(w: &mut World, id: ObjectId) {
    let Some(g) = w.obj(id).and_then(|o| o.cloak.field) else {
        return;
    };
    let r = full_radius(w, id);
    if r <= 0 {
        // spec: S-0516 claim 20 (a zero or negative type radius gives no restart effects)
        return;
    }
    // spec: S-0516 claim 20 and Constants (the strict integer squared-distance test ΔX² + ΔY² < 4(r + 2)²)
    let reach2 = 4 * i64::from(r + 2) * i64::from(r + 2);
    let others: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.id != id && o.on_map && o.cloak.field.is_some())
        .map(|o| o.id)
        .collect();
    for x in others {
        let Some(g2) = w.obj(x).and_then(|o| o.cloak.field) else {
            continue;
        };
        // spec: S-0516 claim 20 (X, Y the cell coordinates of each building's centre; no lepton distance, height or foundation edge)
        let (dx, dy) = (
            i64::from(g2.centre.x - g.centre.x),
            i64::from(g2.centre.y - g.centre.y),
        );
        if dx * dx + dy * dy >= reach2 {
            continue;
        }
        let full2 = full_radius(w, x);
        // spec: S-0516 claim 20 (in play, powered and idle); Q-0792 (interim: idle is in operation, at any current radius)
        if full2 <= 0 || !crate::econ::power::in_operation(w, x) {
            continue;
        }
        // spec: S-0516 claim 20 (at its full radius the neighbour restarts from 0; otherwise its radius decreases by one, floored at 0)
        let next = if g2.radius == full2 {
            0
        } else {
            (g2.radius - 1).max(0)
        };
        set_radius(w, x, next);
    }
}

/// The immediate collapse of claim 20 (removal, sale, destruction,
/// capture): the field is cleared at once, without building-centre
/// interruptions, and the neighbour-restart test is performed.
pub fn vanish(w: &mut World, id: ObjectId) {
    let Some(g) = w.obj(id).and_then(|o| o.cloak.field) else {
        return;
    };
    if g.radius > 0 {
        let gone = cells(g.centre, g.radius);
        for &c in &gone {
            clear(w, id, g.house, c);
        }
        set_radius(w, id, 0);
        reevaluate(w, &gone);
    }
    // spec: S-0516 claim 20 (an immediate collapse performs the same neighbour-restart test)
    restart_neighbours(w, id);
}

/// A captured generator (claim 20): the old field collapses at once, with
/// its neighbour restart, before it grows again for the new owner while in
/// operation.
pub fn captured(w: &mut World, id: ObjectId) {
    if w.obj(id).is_none_or(|o| o.cloak.field.is_none()) {
        return;
    }
    // spec: S-0516 claim 20 (on capture the old field's collapse and restart test come before any new-owner growth)
    vanish(w, id);
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
