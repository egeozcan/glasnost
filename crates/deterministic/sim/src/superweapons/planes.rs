//! The paradrop launch and the spy plane (S-0513 claims 42 to 45; S-0522
//! claims 1 to 3 and 16). Delivery after the launch is the seam
//! [`crate::paradrop::drop_passengers`]; the camera is in `air`
//! (`spy_camera`) and [`crate::econ::shroud::camera`].

use glasnost_map::CellXY;
use glasnost_rules::sim::{LocomotionClass, MovementZone};

use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, ObjectId, Pos, Target, World};

/// Whether a cell's tile is a water-role tile (S-0522 Terms): the general
/// water test of S-0010 claim 5, over the water role's range of table R.
fn water_role(w: &World, c: CellXY) -> bool {
    let Some(cat) = w.catalogue.as_deref() else {
        // A world without a tile catalogue (synthetic maps) has no tile roles.
        return false;
    };
    let Some(tile) = w.cells.cell(c).map(|x| x.tile) else {
        return false;
    };
    // spec: S-0010 claim 4 (an unset role has the start −1; the theater with index 5's forced unset roles come from the importer)
    let start = cat.roles.water;
    if start < 0 && !w.rules().quirks.unset_role_range {
        // spec: S-0010 claim 5 (quirks.unset_role_range off: an unset role's range passes no tile)
        return false;
    }
    let start = start.max(-1); // spec: S-0010 claim 4 (the unset start −1)
    // spec: S-0010 table R (the water role's range: 14 tiles from its start; unset: tiles 0 … 12, claim 5)
    (start..start + 14).contains(&tile)
}

/// The paradrop target after the water adjustment (S-0522 claim 1).
pub fn paradrop_target(w: &mut World, cell: CellXY) -> CellXY {
    if !water_role(w, cell) {
        return cell;
    }
    crate::regions::ensure(w);
    // spec: S-0522 claim 1 (foot, normal zone, no reach region, never in deck mode, no height condition, no further test, bridges allowed, no reference cell; no draw)
    let settings = crate::path::NearbySettings {
        class: Some(LocomotionClass::Foot),
        zone: MovementZone::Normal,
        reach: None,
        deck: false,
        height: false,
        reference: None,
        no_bridges: false,
        no_overlay: false,
    };
    match crate::path::nearby(w, cell, crate::path::NearbyUse::Settings(settings)) {
        // spec: S-0522 claim 1 (the result replaces the target only when there is one and it is not itself a water-role tile)
        Some(c) if !water_role(w, c) => c,
        _ => cell,
    }
}

/// The payload list index for a launch (S-0522 claim 2): 0 to 2 the side
/// groups, 3 the American list.
pub fn payload_group(w: &World, h: HouseId, american: bool) -> usize {
    if american {
        // spec: S-0522 claim 2 (the American paradrop always uses the American payload list)
        return 3; // spec: S-0522 Import mapping (the American list, AmerParaDrop)
    }
    // spec: S-0522 claim 2 (by the side's position in the rules' side list, S-0004 claim 9)
    let side = w
        .houses
        .get(usize::from(h.0))
        .and_then(|x| {
            w.rules()
                .econ
                .tech
                .country_side
                .get(x.country as usize)
                .copied()
        })
        .flatten();
    match side {
        // spec: S-0522 claim 2 (position 0: the first side group's list)
        Some(0) => 0,
        // spec: S-0522 claim 2 (position 2: the third side group's list)
        Some(2) => 2,
        // spec: S-0522 claim 2 (any other position: the second side group's list)
        _ => 1,
    }
}

/// Sends one visiting plane of aircraft type `ti` for house `h` at
/// `target`, in behaviour `b` (S-0522 claim 3; S-0513 claim 43): its
/// creation draw, then its arrival-cell draws (S-0522 claims 4 and 16).
fn send_plane(w: &mut World, h: HouseId, ti: u32, target: CellXY, b: Behaviour) -> ObjectId {
    // spec: S-0522 claim 16 (the plane-creation draw precedes its arrival-cell draws)
    let id = crate::objects::create(w, Kind::Aircraft, ti, Some(h));
    // spec: S-0522 claims 4 and 5 (the arrival cell on the house's arrival edge, with its draws)
    let cell = crate::air::house_edge_cell(w, h);
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0522 claim 3 (a visiting aircraft, so placed at cruise height by S-0501 claim 1)
        o.air = Some(crate::air::AirState {
            visiting: true,
            ..Default::default()
        });
    }
    // spec: S-0522 claim 3 (placed at the centre of its arrival cell, facing 0); Constants (initial facing 0, map north)
    crate::objects::place(w, id, Pos::centre(cell, 0), 0);
    let (t, stale) = (w.tick, w.rules().quirks.stale_suspended_revert);
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0522 claim 3 (it starts its approach immediately, with the launch target and no destination); S-0513 claim 43
        o.target = Some(Target::Cell(target));
        o.dest = None;
        o.beh.force(b, t, stale);
    }
    // Placement cannot fail in our engine (`objects::place`), so the
    // removal of a plane whose placement failed (S-0522 claim 3, S-0513
    // claim 43) never arises.
    id
}

/// A paradrop launch (S-0513 claim 42; S-0522 claims 1 to 3 and 16).
/// Sends nothing for an illegal cell, a missing plane type or a rejected
/// payload list; the launch counts as carried out either way.
pub fn paradrop(w: &mut World, h: HouseId, cell: CellXY, american: bool) -> Vec<ObjectId> {
    let mut planes = Vec::new();
    // spec: S-0522 claim 1 (a launch at a cell that is not a legal map cell sends nothing)
    if !w.cells.footprint().is_legal(cell) {
        return planes;
    }
    // spec: S-0522 claim 1 (a water-role target is moved once, before any plane is sent)
    let target = paradrop_target(w, cell);
    let Some(plane) = w.rules().superweapons.para_plane else {
        // spec: S-0522 claim 2 and Import mapping (no paradrop plane type: no plane is sent)
        return planes;
    };
    let g = payload_group(w, h, american);
    let Some(list) = w.rules().superweapons.payloads[g].clone() else {
        // spec: S-0522 claim 2 (lists of different lengths: no plane at all)
        return planes;
    };
    // spec: S-0522 claim 2 (one plane per entry, in list order)
    for (inf, n) in list {
        let id = send_plane(w, h, plane, target, Behaviour::ParadropApproach);
        // spec: S-0522 claim 16 (the passengers' creation draws follow the plane's arrival draws)
        for _ in 0..n {
            let p = crate::objects::create(w, Kind::Infantry, inf, Some(h));
            // spec: S-0522 claim 3 (the last paratrooper created is the first released, as in S-0507 claim 3)
            crate::transport::board(w, p, id);
        }
        planes.push(id);
    }
    planes
}

/// The number of spy planes a launch sends (S-0513 claim 43).
pub fn spy_plane_count(w: &World) -> usize {
    if !w.rules().quirks.spy_plane_count_from_paradrop_list {
        // spec: S-0513 claim 43 (quirks.spy_plane_count_from_paradrop_list off: exactly one plane)
        return 1;
    }
    let (inf, num) = w.rules().superweapons.ally_para;
    // spec: S-0513 claim 43 (the length of the first side group's paradrop infantry list, for every side; none when it and its count list differ in length)
    if inf == num { inf } else { 0 }
}

/// A spy plane launch (S-0513 claim 43).
pub fn spy_plane(w: &mut World, h: HouseId, cell: CellXY) -> Vec<ObjectId> {
    let mut planes = Vec::new();
    // spec: S-0513 claim 43 (an illegal cell sends no plane; the launch still counts)
    if !w.cells.footprint().is_legal(cell) {
        return planes;
    }
    let Some(ti) = w.rules().superweapons.spy_plane else {
        // spec: S-0513 claim 43 (no spy plane type: no plane; the launch still counts)
        return planes;
    };
    for _ in 0..spy_plane_count(w) {
        // spec: S-0513 claim 43 (each a visiting aircraft with the target cell as target, no destination, spy-plane approach, placed as in S-0522 claims 4 and 5 with the same draws, facing 0, no passengers)
        planes.push(send_plane(w, h, ti, cell, Behaviour::SpyPlaneApproach));
    }
    planes
}
