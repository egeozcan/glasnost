//! Crushing (S-0210): crushing at cell centres, overlays and walls, the
//! warning ahead and the crush-slowdown state of the driving locomotor.
//!
//! Body tilt (claims 6, 8, 9 and 12) only changes how a vehicle is drawn
//! and is left to the renderer. Train damage (claim 10) needs damage
//! (milestone M4) and is not applied yet.

use glasnost_map::{CellXY, NO_OVERLAY};
use glasnost_rules::sim::MovementZone;

use crate::occupancy;
use crate::rules::Kind;
use crate::scatter::{self, ScatterRequest};
use crate::world::{Layer, ObjectId, Target, World};

/// A crusher with the driving locomotor begins a track whose first cell's
/// layer holds infantry: a cell-wide scatter request (S-0210 claim 7).
pub fn warn_ahead(w: &mut World, id: ObjectId, first: CellXY) {
    let Some(layer) = w.obj(id).map(|o| o.layer) else {
        return;
    };
    let infantry = w
        .cells
        .list(first, layer)
        .iter()
        .any(|&x| w.obj(x).is_some_and(|o| o.kind == Kind::Infantry));
    if infantry {
        // spec: S-0210 claim 7 (no threat point, forced, not move-anywhere)
        scatter::cell_request(
            w,
            first,
            layer,
            ScatterRequest {
                threat: None,
                forced: true,
                anywhere: false,
            },
        );
    }
}

/// A driving vehicle drives into a new cell (S-0210 claim 9, driving
/// locomotor case).
pub fn entered_cell(w: &mut World, id: ObjectId, c: CellXY) {
    let Some(o) = w.obj(id) else { return };
    let Some(u) = w.rules().unit(o.kind, o.type_index) else {
        return;
    };
    let crush_track = o
        .drive
        .as_ref()
        .and_then(|d| d.track.as_ref())
        .is_some_and(|t| t.crush);
    // spec: S-0210 claim 9 (movement zone crusher-all, on a track in crush mode)
    if u.zone != MovementZone::CrusherAll || !crush_track {
        return;
    }
    let q = w.rules().quirks;
    // spec: S-0210 claim 9 (the ground layer is looked at, quirks.crush_slowdown_ground_layer)
    let layer = if q.crush_slowdown_ground_layer {
        Layer::Ground
    } else {
        o.layer
    };
    let me = id;
    let own_layer = o.layer;
    let vehicle = w.cells.list(c, layer).iter().any(|&x| {
        // spec: S-0210 claim 9 (the vehicle itself counts, quirks.crush_slowdown_counts_self)
        (x != me || (q.crush_slowdown_counts_self && own_layer == layer))
            && w.obj(x).is_some_and(|ob| ob.kind == Kind::Vehicle)
    });
    if vehicle && let Some(ob) = w.obj_mut(id) {
        ob.crush_slowdown = true;
    }
}

/// A driving vehicle reaches the centre of a cell (S-0210 claims 3, 5 and
/// 8; `quirks.crush_slowdown_persists` off for the extra end of claim 9).
pub fn at_cell_centre(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    let q = w.rules().quirks;
    let c = o.pos.cell();
    if !q.crush_slowdown_persists && o.crush_slowdown {
        // spec: S-0210 Edge cases (quirks.crush_slowdown_persists off: ends at a centre whose layer holds no other vehicle)
        let other = w
            .cells
            .list(c, o.layer)
            .iter()
            .any(|&x| x != id && w.obj(x).is_some_and(|ob| ob.kind == Kind::Vehicle));
        if !other && let Some(ob) = w.obj_mut(id) {
            ob.crush_slowdown = false;
        }
    }
    if !occupancy::is_crusher(w, id) {
        return;
    }
    crush_overlay(w, id, c);
    // spec: S-0210 claim 3 (every crushable object of the cell layer less than 128 leptons from the crusher)
    let listed: Vec<ObjectId> = w.cells.list(c, o.layer).to_vec();
    for x in listed {
        if x == id || w.obj(x).is_none() {
            continue;
        }
        let Some(cr) = w.obj(id).cloned() else { return };
        let rp = w.reference_point(x);
        let dx = i64::from(rp.x - cr.pos.x);
        let dy = i64::from(rp.y - cr.pos.y);
        // spec: S-0210 F1 (Δx² + Δy² < 16384, exact integers)
        if dx * dx + dy * dy >= 16384 {
            continue;
        }
        if !occupancy::crushable_by(w, id, x) {
            continue;
        }
        let r = w.rules_arc();
        let ob = w.obj(x).cloned();
        let Some(ob) = ob else { continue };
        let thief = r
            .unit(ob.kind, ob.type_index)
            .is_some_and(|u| u.vehicle_thief);
        let train = r.unit(cr.kind, cr.type_index).is_some_and(|u| u.train);
        if thief && ob.dest == Some(Target::Object(id)) && !train {
            // spec: S-0210 claim 5 (the crusher is taken over instead: the thief disappears, the owner changes at once)
            crate::objects::remove(w, x);
            if let Some(c2) = w.obj_mut(id) {
                c2.owner = ob.owner;
            }
            continue;
        }
        // spec: S-0210 claim 4 (removed on the same tick, no damage)
        crate::objects::remove(w, x);
    }
}

/// Crushes a cell's overlay (S-0210 claim 8).
fn crush_overlay(w: &mut World, id: ObjectId, c: CellXY) {
    let r = w.rules_arc();
    let Some(cell) = w.cells.cell(c).cloned() else {
        return;
    };
    if cell.overlay == NO_OVERLAY {
        return;
    }
    let Some(ov) = r.overlays.get(usize::from(cell.overlay)).cloned() else {
        return;
    };
    let all = w
        .obj(id)
        .and_then(|o| r.unit(o.kind, o.type_index))
        .is_some_and(|u| u.zone == MovementZone::CrusherAll);
    // spec: S-0210 claim 8 (a crushable overlay, or a wall overlay for a crusher-all movement zone)
    if !(ov.crushable || (ov.wall && all)) {
        return;
    }
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0210 claim 8 (crushing an overlay ends the crush-slowdown state)
        ob.crush_slowdown = false;
    }
    if !ov.wall && r.quirks.crushable_overlay_stays {
        // spec: S-0210 claim 8 (a crushable overlay that is not a wall stays, quirks.crushable_overlay_stays)
        return;
    }
    // spec: S-0210 claim 8 (stage s = ⌊v / 16⌋)
    let stage = i32::from(cell.overlay_data) / 16;
    let d = ov.damage_levels;
    let hits = ov.wall && d > 2 && stage == d - 2; // spec: S-0210 claim 8 (a type with D > 2 at damage stage D − 2)
    remove_overlay(w, c);
    if hits {
        // spec: S-0210 F3 (neighbours in the order north, east, south, west)
        for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            let n = CellXY::new(c.x + dx, c.y + dy);
            let Some(nc) = w.cells.cell(n) else { continue };
            // spec: S-0210 claim 8 (an undamaged wall of the same type takes the hit)
            if nc.overlay != cell.overlay || nc.overlay_data / 16 != 0 {
                continue;
            }
            // spec: S-0210 F3 (S > 200: a ranged draw over 0 … S succeeds below 200; otherwise no draw)
            let ok = if ov.strength > 200 {
                w.rng.ranged(0, ov.strength, r.quirks.ranged_draw_half_span) < 200
            } else {
                true
            };
            if ok {
                // What a successful hit does to a wall belongs to the overlay specs (not yet written).
                w.note("S-0210 claim 8: a wall hit succeeded; its effect on the wall waits for the overlay specs");
            }
        }
    }
}

/// Removes a cell's overlay and updates passability (S-0211, S-0212).
pub fn remove_overlay(w: &mut World, c: CellXY) {
    let wall = w
        .cells
        .cell(c)
        .is_some_and(|x| crate::walls::is_wall(w, x.overlay));
    if let Some(cell) = w.cells.grid.get_mut(c) {
        cell.overlay = NO_OVERLAY;
        cell.overlay_data = 0;
    }
    if wall {
        // spec: S-0112 claim 7 (a wall removed by a wall hit, crushing included, recomputes every cell within two edge steps)
        crate::walls::wall_removed(w, c);
    }
    if let Some(i) = w.cells.idx(c) {
        // spec: S-0210 claim 14 (c) (a wall that is removed has no owner from then on)
        w.cells.wall_owner[i] = None;
    }
    crate::load::recalc_cells(w, &[c]);
    crate::regions::cell_changed(w, c);
}

/// Gives each wall the map places its owner (S-0210 claim 14 (a)): the
/// owner of the nearest building that is on the map and whose owner's
/// country has the wall-owner flag, by the 3-D distance of S-0204 from the
/// building's reference point to the wall cell's centre at the surface
/// height there, truncated to whole leptons; on equal distances the
/// building created first. With no such building the wall has no owner.
pub fn assign_wall_owners(w: &mut World) {
    let r = w.rules_arc();
    let buildings: Vec<(ObjectId, crate::world::HouseId, crate::world::Pos)> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.on_map && o.health > 0)
        .filter_map(|o| {
            let h = o.owner?;
            let country = w.houses.get(usize::from(h.0))?.country as usize;
            // spec: S-0210 claim 14 (a) (the owner's country has the wall-owner flag)
            r.countries
                .get(country)
                .is_some_and(|c| c.wall_owner)
                .then(|| (o.id, h, w.reference_point(o.id)))
        })
        .collect();
    let walls: Vec<CellXY> = w
        .cells
        .grid
        .iter()
        .filter(|(_, cell)| {
            cell.overlay != NO_OVERLAY
                && r.overlays
                    .get(usize::from(cell.overlay))
                    .is_some_and(|o| o.wall)
        })
        .map(|(c, _)| c)
        .collect();
    for c in walls {
        let cx = 256 * c.x + 128; // spec: S-0204 claim 3 (a cell centre: (256x + 128, 256y + 128))
        let cy = 256 * c.y + 128; // spec: S-0204 claim 3 (a cell centre)
        // spec: S-0210 claim 14 (a) (the wall cell's centre at the terrain surface height there, S-0106)
        let cz = w.surface_at(cx, cy);
        let mut best: Option<(u64, ObjectId, crate::world::HouseId)> = None;
        // Q-0036: claim 14 states no distance limit, scenario K12 implies one; none here.
        for &(id, h, p) in &buildings {
            let dx = i64::from(p.x - cx);
            let dy = i64::from(p.y - cy);
            let dz = i64::from(p.z - cz);
            // spec: S-0210 claim 14 (a) (S-0204's 3-D distance without the building reduction, truncated to whole leptons)
            let d = glasnost_num::geom::root_floor(
                (dx * dx + dy * dy + dz * dz) as u64,
                r.quirks.table_root_distance,
                r.mode,
            );
            // spec: S-0210 claim 14 (a) (equal distances: the building created earliest)
            if best.is_none_or(|(bd, bid, _)| d < bd || (d == bd && id < bid)) {
                best = Some((d, id, h));
            }
        }
        if let Some(i) = w.cells.idx(c) {
            w.cells.wall_owner[i] = best.map(|b| b.2);
        }
    }
}
