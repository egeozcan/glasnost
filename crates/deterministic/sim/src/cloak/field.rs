//! Cloak fields (S-0516 claims 19 and 20): a generator's circle grows one
//! radius per tick while it is in operation and shrinks one radius per tick
//! while it is not; removal, sale, destruction and capture collapse it at
//! once.

use std::collections::{BTreeMap, BTreeSet};

use glasnost_map::CellXY;

use super::{FieldPhase, Generator, centre_cell, field_circle, type_radius};
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
            // spec: S-0516 claim 20 (a newly created generator is idle at radius 0 with no covered cells; becoming in operation makes growth pending, so radius 1 comes on the first growth update, in the turn that detects the change)
            phase: FieldPhase::Idle,
            was_on: false,
        });
    }
}

/// Re-evaluates every object listed in the given cells (claim 20).
fn reevaluate(w: &mut World, cells: &[CellXY]) {
    for &c in cells {
        super::machine::reevaluate_cell(w, c, false);
    }
}

/// The other generators of a house, for the fixed behaviour of
/// `quirks.cloak_field_overlap_loss`: their centres and radii, and one
/// circle per distinct radius. Taken once before a pass of clears, which
/// change no generator (#389).
struct Others {
    generators: Vec<(CellXY, i32)>,
    circles: BTreeMap<i32, BTreeSet<(i32, i32)>>,
}

impl Others {
    /// The generators of house `h` other than `id` with a positive radius;
    /// none when the switch is on, as `clear` then never asks.
    fn of(w: &World, id: ObjectId, h: HouseId) -> Option<Self> {
        // spec: S-0516 claim 20 and quirks.cloak_field_overlap_loss (a flag, not a count: shared cells leave the field)
        if w.rules().quirks.cloak_field_overlap_loss {
            return None;
        }
        let mut generators = Vec::new();
        let mut circles = BTreeMap::new();
        for o in w.objects.values() {
            if o.id == id {
                continue;
            }
            if let Some(g) = o.cloak.field
                && g.house == h
                && g.radius > 0
            {
                generators.push((g.centre, g.radius));
                circles
                    .entry(g.radius)
                    .or_insert_with(|| field_circle(g.radius));
            }
        }
        Some(Self {
            generators,
            circles,
        })
    }

    /// Whether one of them covers `c` now.
    fn cover(&self, c: CellXY) -> bool {
        self.generators.iter().any(|&(centre, r)| {
            self.circles
                .get(&r)
                .is_some_and(|s| s.contains(&(c.x - centre.x, c.y - centre.y)))
        })
    }
}

/// Takes cell `c` out of house `h`'s field (claim 20, overlaps); whether it
/// was cleared. `others` is [`Others::of`] for the clearing generator.
fn clear(w: &mut World, h: HouseId, others: Option<&Others>, c: CellXY) -> bool {
    // spec: S-0516 Edge cases (quirks.cloak_field_overlap_loss off: a cell stays in the field while any powered generator of the house covers it)
    if others.is_some_and(|o| o.cover(c)) {
        return false;
    }
    w.cloak.fields.remove(&(h.0, c))
}

/// A generator's turn (claim 20): the ordinary building turn first detects a
/// change in operation, which can make growth or shrinking pending (the
/// table of claim 20's pending changes), then a growth update runs while
/// growth is pending and a shrink update while shrinking is. Staying in
/// operation (or out of it) requests nothing, and pending growth has no
/// power gate.
pub fn turn(w: &mut World, id: ObjectId) {
    let Some(g) = w.obj(id).and_then(|o| o.cloak.field) else {
        return;
    };
    let full = full_radius(w, id);
    let on = crate::econ::power::in_operation(w, id);
    let interrupt = w.rules().quirks.cloak_field_building_interrupt;
    let phase = if on == g.was_on {
        // spec: S-0516 claim 20 (merely remaining in operation, or out of it, requests no growth or shrinking; an idle field below its full radius stays idle)
        g.phase
    } else {
        match (on, g.phase) {
            // spec: S-0516 claim 20 (idle or shrinking, radius unequal to the type radius, a change to being in operation: growing from the current radius, no wait for the shrink to reach 0)
            (true, FieldPhase::Idle | FieldPhase::Shrinking) if g.radius != full => {
                FieldPhase::Growing
            }
            // spec: S-0516 claim 20 (idle or growing at a positive radius, a change to being out of operation: shrinking from the current radius; at radius 0 every phase is unchanged)
            (false, FieldPhase::Idle | FieldPhase::Growing) if g.radius > 0 => {
                FieldPhase::Shrinking
            }
            // spec: S-0516 claim 20 (shrinking at the full radius stays shrinking on a change to being in operation; growing stays growing; a pending phase survives its own kind of change)
            (_, p) => p,
        }
    };
    if let Some(ob) = w.obj_mut(id)
        && let Some(f) = ob.cloak.field.as_mut()
    {
        f.was_on = on;
    }
    set_phase(w, id, phase);
    match phase {
        FieldPhase::Growing if g.radius >= full => {
            // spec: S-0516 claim 20 (growing at full radius: the next growth update ends growth; reaching the radius alone has not made it idle)
            set_phase(w, id, FieldPhase::Idle);
        }
        FieldPhase::Growing => {
            // spec: S-0516 claim 20 (growth: radius 1 on the first growth update, then 2, through r, one per tick; the radius increases even when the update is interrupted)
            let r = g.radius + 1;
            set_radius(w, id, r);
            // spec: S-0516 claim 20 (the generator's own centre is not exempt, including on the first radius-1 update)
            let own = if interrupt {
                own_centres(w, g.house)
            } else {
                BTreeSet::new()
            };
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
        }
        FieldPhase::Shrinking if g.radius <= 0 => {
            // spec: S-0516 claim 20 (at radius 0 the following shrink update stops without retrying the omitted cells or the restart)
            set_phase(w, id, FieldPhase::Idle);
        }
        FieldPhase::Shrinking => {
            // spec: S-0516 claim 20 (shrinking: one radius per tick to 0; cells outside the shrinking circle leave at once)
            let r = g.radius - 1;
            set_radius(w, id, r);
            let keep: BTreeSet<CellXY> = cells(g.centre, r).into_iter().collect();
            let own = if interrupt {
                own_centres(w, g.house)
            } else {
                BTreeSet::new()
            };
            let others = Others::of(w, id, g.house);
            let mut gone = Vec::new();
            let mut interrupted = false;
            // spec: S-0516 claim 20 (a shrink update clears its own ring; cells an earlier interrupted shrink left are not retried)
            for c in cells(g.centre, g.radius) {
                if keep.contains(&c) {
                    continue;
                }
                gone.push(c);
                // spec: S-0516 claim 20 and quirks.cloak_field_building_interrupt (a shrink update stops after clearing an own building's centre, the generator's own centre included)
                if clear(w, g.house, others.as_ref(), c) && own.contains(&c) {
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
        FieldPhase::Idle => {}
    }
}

fn set_phase(w: &mut World, id: ObjectId, p: FieldPhase) {
    if let Some(ob) = w.obj_mut(id)
        && let Some(g) = ob.cloak.field.as_mut()
    {
        g.phase = p;
    }
}

fn set_radius(w: &mut World, id: ObjectId, r: i32) {
    if let Some(ob) = w.obj_mut(id)
        && let Some(g) = ob.cloak.field.as_mut()
    {
        g.radius = r;
    }
}

/// The neighbour restart of claim 20: every other in-play generator in
/// operation and idle (neither growth nor shrinking pending), of any owner,
/// whose centre cell lies strictly within Δ*X*² + Δ*Y*² < 4(*r* + 2)² of
/// this one's, *r* this generator's type radius: at its full radius it
/// restarts from 0, otherwise its radius decreases by one, floored at 0;
/// growth becomes pending either way.
fn restart_neighbours(w: &mut World, id: ObjectId) {
    let Some(g) = w.obj(id).and_then(|o| o.cloak.field) else {
        return;
    };
    // spec: S-0516 claim 20 (a zero or negative type radius is treated as 0: ordinary shrinking never reaches this test from radius 0, but an immediate collapse performs it, with strict reach ΔX² + ΔY² < 16)
    let r = full_radius(w, id);
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
        // spec: S-0516 claim 20 (in play, in operation and idle: no growth or shrinking pending, whatever the radius; a zero-radius generator can receive a restart)
        if g2.phase != FieldPhase::Idle || !crate::econ::power::in_operation(w, x) {
            continue;
        }
        // spec: S-0516 claim 20 (at its full radius the neighbour restarts from 0; otherwise its radius decreases by one, floored at 0; growth becomes pending)
        let next = if g2.radius == full2 {
            0
        } else {
            (g2.radius - 1).max(0)
        };
        set_radius(w, x, next);
        set_phase(w, x, FieldPhase::Growing);
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
        let others = Others::of(w, id, g.house);
        for &c in &gone {
            clear(w, g.house, others.as_ref(), c);
        }
        set_radius(w, id, 0);
        reevaluate(w, &gone);
    }
    set_phase(w, id, FieldPhase::Idle);
    // spec: S-0516 claim 20 (an immediate collapse performs the same neighbour-restart test, also when both the current and the type radius are 0 and nothing was covered)
    restart_neighbours(w, id);
}

/// An accepted manual power-off order (S-0516 claim 20, Manual power-off;
/// S-0405 claim 10 owns which orders are accepted): shrinking becomes
/// pending at once, and with `quirks.cloak_generator_zero_radius_shutdown`
/// a current radius of 0 becomes the type radius first, without adding
/// coverage. The recorded operation state is left alone, so the next turn's
/// change detection finds nothing new if the building is switched on again
/// before it.
pub fn switched_off(w: &mut World, id: ObjectId) {
    let Some(g) = w.obj(id).and_then(|o| o.cloak.field) else {
        return;
    };
    let zero_shutdown = w.rules().quirks.cloak_generator_zero_radius_shutdown;
    // spec: S-0516 claim 20 and quirks.cloak_generator_zero_radius_shutdown (on: shrinking pending even at radius 0; off: only at a positive current radius, as for an ordinary loss of operation)
    if g.radius <= 0 && !zero_shutdown {
        return;
    }
    if g.radius <= 0 {
        // spec: S-0516 claim 20 (a current radius of 0 becomes the type radius before the shrink; it never invents a positive radius for a zero-radius type, and adds no coverage)
        let full = full_radius(w, id);
        set_radius(w, id, full);
    }
    set_phase(w, id, FieldPhase::Shrinking);
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
    let on = crate::econ::power::in_operation(w, id);
    if let Some(ob) = w.obj_mut(id) {
        match ob.owner {
            Some(h) => {
                if let Some(g) = ob.cloak.field.as_mut() {
                    g.house = h;
                    // spec: S-0516 claim 20 (on capture the field grows again for the new owner if the building is in operation; otherwise it stays idle until a change in operation)
                    g.phase = if on {
                        FieldPhase::Growing
                    } else {
                        FieldPhase::Idle
                    };
                    g.was_on = on;
                }
            }
            None => ob.cloak.field = None,
        }
    }
}
