//! Creating objects, putting them on the map and taking them off.

use glasnost_map::{CellXY, Map};
use glasnost_num::F64;
use glasnost_num::geom::Facing;

use crate::behaviour::Behaviours;
use crate::cells::OccupancyQuirks;
use crate::drive::DriveState;
use crate::infantry::InfantryState;
use crate::path::PathState;
use crate::rules::{Kind, Locomotor};
use crate::turning::Turning;
use crate::walk::{self, WalkState};
use crate::world::{HouseId, Layer, Object, ObjectId, Pos, World};

/// The occupancy switches.
pub fn occ_quirks(w: &World) -> OccupancyQuirks {
    let q = w.rules().quirks;
    OccupancyQuirks {
        side_positions_only: q.occupancy_house_side_positions_only,
        stale: q.occupancy_house_stale,
    }
}

/// Creates an object (not yet on the map). Every creation of an infantry,
/// vehicle, aircraft or building object takes one raw value from the
/// scenario random stream (S-0202 claim 10).
pub fn create(w: &mut World, kind: Kind, type_index: u32, owner: Option<HouseId>) -> ObjectId {
    if kind != Kind::Terrain {
        // spec: S-0202 claim 10 (the creation draw: one raw value)
        let _ = w.rng.raw();
    }
    let id = w.new_id();
    let r = w.rules_arc();
    let (health, foundation, loco) = match kind {
        Kind::Building => {
            let b = &r.buildings[type_index as usize];
            (b.strength, b.foundation, Locomotor::None)
        }
        Kind::Terrain => (0, (1, 1), Locomotor::None),
        _ => {
            let u = r.unit(kind, type_index).expect("unit type");
            (u.strength, (1, 1), u.locomotor)
        }
    };
    let obj = Object {
        id,
        kind,
        type_index,
        owner,
        pos: Pos::default(),
        layer: Layer::Ground,
        health,
        on_map: false,
        body: Turning::at(0),
        turret: Turning::at(0),
        // spec: S-0206 claim 9 (a vehicle that never fired counts as having last fired 100 ticks before tick 0)
        last_fired: -100,
        beh: Behaviours::default(),
        target: None,
        dest: None,
        guard_spot: None,
        queued_dests: Vec::new(),
        attack_move: None,
        contact: None,
        path: PathState::default(),
        drive: (loco == Locomotor::Drive).then(DriveState::new),
        walk: (loco == Locomotor::Walk).then(WalkState::default),
        infantry: (kind == Kind::Infantry).then(InfantryState::default),
        crush_slowdown: false,
        targeting_timer: 0,
        last_turn: -1,
        foundation,
        // spec: S-0207 Constants (starting speed bonus factor b = 1.0); S-0208 Terms (speed multiplier 1.0 at creation)
        speed_bonus: F64::from_i64(1, w.rules().mode),
    };
    w.objects.insert(id, obj);
    id
}

/// The cells a building's foundation covers.
pub fn foundation_cells(top_left: CellXY, f: (i32, i32)) -> Vec<CellXY> {
    let mut out = Vec::new();
    for dy in 0..f.1 {
        for dx in 0..f.0 {
            out.push(CellXY::new(top_left.x + dx, top_left.y + dy));
        }
    }
    out
}

/// Puts a created object on the map at `pos` facing `facing`: it enters
/// play (S-0203 claim 3), is listed in its cell layer (S-0209 claim 13),
/// marks the occupancy record (S-0209 claim 1), sets its facings at once
/// (S-0206 claim 5) and its cells are recomputed (S-0211 claim 5).
pub fn place(w: &mut World, id: ObjectId, pos: Pos, facing: Facing) {
    let tick = w.tick;
    let Some(o) = w.obj(id).cloned() else {
        return;
    };
    let top_quirk = w.rules().quirks.top_position_counts_as_centre;
    let keeps = w.rules().quirks.set_facing_keeps_target;
    let cell = pos.cell();
    let mut touched = vec![cell];
    match o.kind {
        Kind::Building => {
            touched = foundation_cells(cell, o.foundation);
            for &c in &touched {
                w.cells.enter(c, Layer::Ground, id, true);
                w.cells.claim_building(c, true);
            }
        }
        Kind::Terrain => {
            let pattern = {
                let t = &w.rules().terrain[o.type_index as usize];
                if w.theater_snow() { t.occupation.1 } else { t.occupation.0 }
            };
            w.cells.enter(cell, Layer::Ground, id, false);
            w.cells.mark_terrain(cell, id, pattern);
        }
        Kind::Infantry => {
            w.cells.enter(cell, o.layer, id, false);
            let p = walk::position_class(pos.x.rem_euclid(256), pos.y.rem_euclid(256), top_quirk); // spec: S-0204 claim 2 (the position within its cell, 0 … 255 per axis)
            if let Some(h) = o.owner {
                w.cells.mark_infantry(cell, o.layer, p, id, h);
            }
        }
        Kind::Vehicle | Kind::Aircraft => {
            w.cells.enter(cell, o.layer, id, false);
            if o.kind == Kind::Vehicle {
                w.cells.claim_vehicle(cell, o.layer, true);
                // The claim is the vehicle's own, released when it leaves the cell.
                if let Some(d) = w.obj_mut(id).and_then(|ob| ob.drive.as_mut()) {
                    d.claims = vec![cell];
                }
            } else {
                // spec: S-0209 claim 1 (an aircraft on the ground sets the object-present flag)
                if let Some(rec) = w.cells.occ_mut(cell, o.layer) {
                    rec.others = rec.others.saturating_add(1);
                }
            }
        }
    }
    if let Some(obj) = w.obj_mut(id) {
        obj.pos = pos;
        obj.on_map = true;
        // spec: S-0206 claim 5 (placing sets body and turret facings at once to the placement facing)
        obj.body.set(facing, tick, keeps);
        obj.turret.set(facing, tick, keeps);
    }
    // spec: S-0203 claim 2 (terrain objects take no turns unless they spawn resources, Q-0018)
    if o.kind != Kind::Terrain {
        w.update.enter(id);
    }
    // spec: S-0211 claim 5 (an object put into a cell recomputes it)
    crate::load::recalc_cells(w, &touched);
    for c in touched {
        crate::regions::cell_changed(w, c);
    }
    if o.kind.is_mobile() {
        // spec: S-0215 Terms (the placement fall-back: an actor falls back to rest when placed)
        crate::process::fall_back(w, id, true);
    }
}

/// Takes an object off the map and out of play (destroyed, removed or gone
/// inside): it leaves the update order, a counting removal during the turn
/// pass (S-0203 claim 10), its marks and claims are cleared and its cells
/// recomputed.
pub fn remove(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else {
        return;
    };
    let skip = w.rules().quirks.update_order_removal_skip;
    w.update.leave(id, skip);
    let q = occ_quirks(w);
    let cell = o.pos.cell();
    let mut touched = vec![cell];
    match o.kind {
        Kind::Building => {
            touched = foundation_cells(cell, o.foundation);
            for &c in &touched {
                w.cells.leave(c, Layer::Ground, id);
                w.cells.claim_building(c, false);
            }
        }
        Kind::Terrain => {
            w.cells.leave(cell, Layer::Ground, id);
            w.cells.clear_terrain(cell, id);
        }
        Kind::Infantry => {
            w.cells.leave(cell, o.layer, id);
            if let Some(wt) = o.walk.as_ref().and_then(|s| s.target) {
                w.cells.clear_infantry(wt.cell(), o.layer, id, q);
            }
            w.cells.clear_infantry(cell, o.layer, id, q);
        }
        Kind::Vehicle => {
            w.cells.leave(cell, o.layer, id);
            crate::drive::release_claims(w, id);
        }
        Kind::Aircraft => {
            w.cells.leave(cell, o.layer, id);
            if let Some(rec) = w.cells.occ_mut(cell, o.layer) {
                rec.others = rec.others.saturating_sub(1);
            }
        }
    }
    w.objects.remove(&id);
    crate::load::recalc_cells(w, &touched);
    for c in touched {
        crate::regions::cell_changed(w, c);
    }
}

/// Places the map's objects (S-0009 claim 27; S-0203 claim 3 kind order:
/// terrain objects, vehicles, aircraft, infantry, buildings).
pub fn place_map_objects(w: &mut World, map: &Map, sandbox: bool) {
    let r = w.rules_arc();
    for t in &map.objects.terrain {
        let name = String::from_utf8_lossy(&t.type_name).into_owned();
        if let Some(i) = r.index_of(Kind::Terrain, &name) {
            let id = create(w, Kind::Terrain, i, None);
            // Q-0010: a terrain object stands at its cell's centre (provisional).
            let z = w.surface_at(256 * t.cell.x + 128, 256 * t.cell.y + 128); // spec: S-0204 claim 3 (the cell centre)
            place(w, id, Pos::centre(t.cell, z), 0);
        }
    }
    let owner_of = |w: &World, name: &[u8]| -> Option<HouseId> {
        // spec: S-0009 claim 27 (owner matched exactly and case-sensitively; the first house in order)
        w.houses
            .iter()
            .position(|h| h.name.as_bytes() == name)
            .map(|i| HouseId(i as u8))
    };
    let lists: [(Kind, Vec<&glasnost_map::objects::Placed>, Vec<i32>); 4] = [ // spec: S-0009 claim 27 (the four object lists: vehicles, aircraft, infantry, structures)
        (
            Kind::Vehicle,
            map.objects.units.iter().map(|u| &u.placed).collect(),
            Vec::new(),
        ),
        (
            Kind::Aircraft,
            map.objects.aircraft.iter().map(|u| &u.placed).collect(),
            Vec::new(),
        ),
        (
            Kind::Infantry,
            map.objects.infantry.iter().map(|u| &u.placed).collect(),
            map.objects.infantry.iter().map(|u| u.sub_cell).collect(),
        ),
        (
            Kind::Building,
            map.objects.structures.iter().map(|u| &u.placed).collect(),
            Vec::new(),
        ),
    ];
    let mut bad_sub = false;
    for (kind, entries, subs) in lists {
        for (n, p) in entries.into_iter().enumerate() {
            let Some(owner) = owner_of(w, &p.owner) else {
                continue;
            };
            let name = String::from_utf8_lossy(&p.type_name).into_owned();
            let Some(ti) = r.index_of(kind, &name) else {
                continue;
            };
            // spec: S-0009 claim 27 (a skirmish drops structures owned by a human player's house)
            if kind == Kind::Building && !sandbox && w.houses[usize::from(owner.0)].human {
                continue;
            }
            let id = create(w, kind, ti, Some(owner));
            // spec: S-0204 claim 10 (a facing value v from 0 to 255 is v × 256 facing units)
            let facing = (p.facing.clamp(0, 255) as u16) * 256;
            let pos = match kind {
                Kind::Infantry => {
                    let sub = subs.get(n).copied().unwrap_or(0);
                    // spec: S-0208 claim 5 (the map's sub-cell value names the standing position; outside 0 … 4: centre, departure)
                    let sp = if (0..=4).contains(&sub) {
                        sub as u8
                    } else {
                        bad_sub = true;
                        0
                    };
                    let (u, v) = walk::standing_point(sp);
                    let (x, y) = (256 * p.cell.x + u, 256 * p.cell.y + v); // spec: S-0208 table P (a standing position within the cell)
                    Pos::new(x, y, w.surface_at(x, y))
                }
                Kind::Building => {
                    // spec: S-0204 claim 3 (a building's position is the centre of its top-left foundation cell)
                    let z = w.surface_at(256 * p.cell.x + 128, 256 * p.cell.y + 128);
                    Pos::centre(p.cell, z)
                }
                _ => {
                    let z = w.surface_at(256 * p.cell.x + 128, 256 * p.cell.y + 128); // spec: S-0204 claim 3 (the cell centre)
                    Pos::centre(p.cell, z)
                }
            };
            place(w, id, pos, facing);
        }
    }
    if bad_sub {
        w.note("S-0208 claim 5: an infantry sub-cell value outside 0 … 4 was placed at the centre");
    }
}
