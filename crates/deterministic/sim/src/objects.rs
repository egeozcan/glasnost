//! Creating objects, putting them on the map and taking them off.

use glasnost_map::objects::{
    FollowEntry, HealthKind, ListKind, facing_from_value, follow_links, resolve_upgrades,
    starting_health,
};
use glasnost_map::{CellXY, Map};
use glasnost_num::F64;
use glasnost_num::geom::Facing;

use crate::behaviour::Behaviours;
use crate::cells::OccupancyQuirks;
use crate::drive::DriveState;
use crate::infantry::InfantryState;
use crate::path::PathState;
use crate::rules::{Behaviour, Kind, Locomotor};
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
                if w.theater_snow() {
                    t.occupation.1
                } else {
                    t.occupation.0
                }
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
    // spec: S-0203 claim 2 (a terrain object takes turns exactly when its type has the spawns-resources flag)
    let takes_turns = o.kind != Kind::Terrain
        || w.rules()
            .terrain
            .get(o.type_index as usize)
            .is_some_and(|t| t.spawns_resources);
    if takes_turns {
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

/// The behaviour a map entry's behaviour name stands for (S-0009 claim 41;
/// S-0215 Import mapping: the section names, matched without regard to
/// case). A name that matches none means *none*.
pub fn behaviour_by_name(name: &[u8]) -> Option<Behaviour> {
    // spec: S-0215 Import mapping (the 32 section names, matched without regard to case)
    const NAMES: [(&str, Behaviour); 32] = [
        ("Ambush", Behaviour::Ambush),
        ("Area Guard", Behaviour::AreaGuard),
        ("Attack", Behaviour::Attack),
        ("Attack Move", Behaviour::AttackMove),
        ("Capture", Behaviour::Capture),
        ("Construction", Behaviour::Construction),
        ("Eaten", Behaviour::Eaten),
        ("Enter", Behaviour::Enter),
        ("Guard", Behaviour::Guard),
        ("Harmless", Behaviour::Harmless),
        ("Harvest", Behaviour::Harvest),
        ("Hunt", Behaviour::Hunt),
        ("Missile", Behaviour::Missile),
        ("Move", Behaviour::Move),
        ("Open", Behaviour::Open),
        ("Paradrop Approach", Behaviour::ParadropApproach),
        ("Paradrop Overfly", Behaviour::ParadropOverfly),
        ("Patrol", Behaviour::Patrol),
        ("QMove", Behaviour::QueuedMove),
        ("Repair", Behaviour::Repair),
        ("Rescue", Behaviour::Rescue),
        ("Retreat", Behaviour::Retreat),
        ("Return", Behaviour::Return),
        ("Sabotage", Behaviour::Sabotage),
        ("Selling", Behaviour::Selling),
        ("Sleep", Behaviour::Sleep),
        ("Spyplane Approach", Behaviour::SpyPlaneApproach),
        ("Spyplane Overfly", Behaviour::SpyPlaneOverfly),
        ("Sticky", Behaviour::Sticky),
        ("Stop", Behaviour::Stop),
        ("Unload", Behaviour::Unload),
        ("Wait", Behaviour::Wait),
    ];
    NAMES
        .iter()
        .find(|(n, _)| n.as_bytes().eq_ignore_ascii_case(name))
        .map(|(_, b)| *b)
}

/// The registration index of a type named `name` of `kind`, compared
/// without regard to letter case (S-0009 claims 36 and 42): the exact name
/// first, otherwise the earliest registered type whose name matches.
pub fn type_index(r: &crate::rules::SimRules, kind: Kind, name: &[u8]) -> Option<u32> {
    if let Ok(s) = std::str::from_utf8(name)
        && let Some(i) = r.index_of(kind, s)
    {
        return Some(i);
    }
    r.names
        .iter()
        .filter(|((k, n), _)| *k == kind && n.as_bytes().eq_ignore_ascii_case(name))
        .map(|(_, &i)| i)
        .min()
}

fn list_kind(k: ListKind) -> Kind {
    match k {
        ListKind::Vehicles => Kind::Vehicle,
        ListKind::Aircraft => Kind::Aircraft,
        ListKind::Infantry => Kind::Infantry,
        ListKind::Buildings => Kind::Building,
    }
}

/// The starting health of a created pre-placed object (S-0009 claim 37).
fn set_starting_health(w: &mut World, id: ObjectId, kind: Kind, value: i32) {
    let chop = w.rules().quirks.chop_rounding;
    let hk = match kind {
        Kind::Infantry => HealthKind::Infantry,
        Kind::Building => HealthKind::Building,
        _ => HealthKind::VehicleOrAircraft,
    };
    let Some(strength) = w.obj(id).map(|o| o.health) else {
        return;
    };
    // spec: S-0009 claim 37 (raised to full strength above S − 3, quirks.near_full_health_rounds_up)
    let near_full = w.rules().quirks.near_full_health_rounds_up;
    let (h, unspecified) = starting_health(hk, value, strength, near_full, chop);
    if unspecified {
        w.note("S-0009 claim 37: a pre-placed object's starting health is unspecified in the original; it starts with 1");
    }
    if let Some(o) = w.obj_mut(id) {
        o.health = h;
    }
}

/// Places the map's objects (S-0009 claims 27 and 34 to 43): terrain
/// objects, then vehicles, aircraft, infantry and buildings (each building's
/// upgrades directly after it), each kind in entry order (claim 43).
/// Smudges, placed last, hold no simulation state.
pub fn place_map_objects(w: &mut World, map: &Map, sandbox: bool) {
    let r = w.rules_arc();
    for t in &map.objects.terrain {
        // spec: S-0009 claim 42 (a name that is not yet a type creates one, which has no rules data)
        let Some(i) = type_index(&r, Kind::Terrain, &t.type_name) else {
            w.note("S-0009 claim 42: a terrain object of an unknown type creates a type without rules data; nothing is placed");
            continue;
        };
        if w.cells.idx(t.cell).is_none() {
            continue;
        }
        let id = create(w, Kind::Terrain, i, None);
        // spec: S-0009 claim 42 (a terrain object stands on its cell's centre with its type's full strength)
        let z = w.surface_at(256 * t.cell.x + 128, 256 * t.cell.y + 128); // spec: S-0009 claim 42 (world position 256·x + 128, 256·y + 128)
        place(w, id, Pos::centre(t.cell, z), 0);
    }
    let owner_of = |w: &World, name: &[u8]| -> Option<HouseId> {
        // spec: S-0009 claim 27 (owner matched exactly and case-sensitively; the first house in order)
        w.houses
            .iter()
            .position(|h| h.name.as_bytes() == name)
            .map(|i| HouseId(i as u8))
    };
    let mut bad_sub = false;
    let mut follow: Vec<FollowEntry> = Vec::new();
    // spec: S-0009 claim 43 (vehicles, then aircraft, then infantry, then buildings, each in entry order)
    for lk in ListKind::ORDER {
        let kind = list_kind(lk);
        let entries = map.objects.list(lk);
        // The upgrade fields of each entry, once the owner and type are matched (claim 40).
        let mut upgrade_fields: Vec<(bool, [Option<Vec<u8>>; 3])> = Vec::new();
        let mut matched: Vec<Option<(HouseId, u32)>> = Vec::new();
        for (n, p) in entries.iter().enumerate() {
            let fields = map.objects.fields(lk, n);
            let m = owner_of(w, &p.owner)
                .and_then(|o| type_index(&r, kind, &p.type_name).map(|t| (o, t)));
            // spec: S-0009 claims 27 and 40 (a structure dropped for a human owner gives no upgrade field)
            let human_dropped = kind == Kind::Building
                && !sandbox
                && m.is_some_and(|(o, _)| w.houses[usize::from(o.0)].human);
            upgrade_fields.push((m.is_some() && !human_dropped, fields.upgrades.clone()));
            matched.push(m);
        }
        // spec: S-0009 claim 40 (an absent upgrade field carries over, quirks.upgrade_fields_carry_over)
        let upgrades = resolve_upgrades(&upgrade_fields, r.quirks.upgrade_fields_carry_over);
        for (n, p) in entries.iter().enumerate() {
            let fields = map.objects.fields(lk, n);
            // spec: S-0009 claims 27 and 36 (an unmatched owner or type drops the entry; no creation draw)
            let Some((owner, ti)) = matched[n] else {
                continue;
            };
            // spec: S-0009 claim 27 (a skirmish drops structures owned by a human player's house)
            if kind == Kind::Building && !sandbox && w.houses[usize::from(owner.0)].human {
                continue;
            }
            if fields.short {
                // spec: S-0009 claim 36 (a short entry: dropped after one creation draw, none for a building)
                if kind != Kind::Building {
                    let _ = w.rng.raw();
                }
                w.note("S-0009 claim 36: an object entry that lacks a required field was dropped (a departure)");
                continue;
            }
            let id = create(w, kind, ti, Some(owner));
            if w.cells.idx(p.cell).is_none() {
                // spec: S-0009 claim 38 (a non-legal cell: the creation draw is taken, the object dropped)
                w.objects.remove(&id);
                w.note("S-0009 claim 38: an object entry on a non-legal cell was dropped after its creation draw (a departure)");
                if kind == Kind::Vehicle {
                    follow.push(FollowEntry {
                        value: fields.follow,
                        placed: false,
                    });
                }
                continue;
            }
            set_starting_health(w, id, kind, p.health);
            // spec: S-0009 claim 39 (body facing 256 × (v mod 256); infantry: field 8, 0 when absent)
            let facing = facing_from_value(p.facing);
            let pos = match kind {
                Kind::Infantry => {
                    let sub = map.objects.infantry.get(n).map_or(0, |i| i.sub_cell);
                    // spec: S-0208 claim 5 (the map's sub-cell value names the standing position; outside 0 … 4: centre, departure)
                    let sp = if (0..=4).contains(&sub) {
                        sub as u8
                    } else {
                        bad_sub = true;
                        0
                    };
                    let (u, v) = walk::standing_point(sp);
                    // spec: S-0009 claim 38 (infantry: (256·x + u, 256·y + v) with S-0208 table P)
                    let (x, y) = (256 * p.cell.x + u, 256 * p.cell.y + v);
                    Pos::new(x, y, w.surface_at(x, y))
                }
                _ => {
                    // spec: S-0009 claim 38 (vehicle, aircraft, building: the centre of the cell (x, y))
                    let z = w.surface_at(256 * p.cell.x + 128, 256 * p.cell.y + 128);
                    Pos::centre(p.cell, z)
                }
            };
            let t = w.tick;
            if kind == Kind::Building {
                // spec: S-0009 claim 40 (a pre-placed building is complete and starts in the guard behaviour)
                if let Some(o) = w.obj_mut(id) {
                    o.beh.force(Behaviour::Guard, t);
                }
            } else if let Some(b) = fields.behaviour.as_deref().and_then(behaviour_by_name) {
                // spec: S-0009 claim 41 (the behaviour the object starts with; none: the placement fall-back)
                if let Some(o) = w.obj_mut(id) {
                    o.beh.force(b, t);
                }
            }
            place(w, id, pos, facing);
            if kind == Kind::Vehicle {
                follow.push(FollowEntry {
                    value: fields.follow,
                    placed: true,
                });
                if fields.bridge == Some(true) {
                    w.note("S-0009 claim 38: a vehicle's bridge flag needs bridge decks (Q-0020); it stands on the ground");
                }
            }
            if kind == Kind::Aircraft {
                w.note("S-0009 claim 38: aircraft stand landed on their cell; one placed in the air needs its type's flight height (aircraft specs)");
            }
            if fields.veterancy.is_some() {
                w.note("S-0009 claim 41: veterancy values need S-0311 (M4); objects keep their type's experience");
            }
            if kind == Kind::Building {
                if !fields.switched_on() {
                    w.note("S-0009 claim 40: a building that starts switched off needs the power topic (no spec); it starts switched on");
                }
                // spec: S-0009 claim 40 (min(c, 3) upgrade fields, each naming a building type, placed right after the building)
                let c = fields.upgrade_count.unwrap_or(0).clamp(0, 3) as usize; // spec: S-0009 claim 40 (a count above 3: our engine uses 3)
                for name in upgrades[n].iter().take(c).flatten() {
                    let Some(ui) = type_index(&r, Kind::Building, name) else {
                        continue;
                    };
                    let uid = create(w, Kind::Building, ui, Some(owner));
                    // spec: S-0009 claim 40 (the upgrades start at full health, at the same cell and facing)
                    if let Some(o) = w.obj_mut(uid) {
                        o.beh.force(Behaviour::Guard, t);
                    }
                    place(w, uid, pos, facing);
                }
            }
        }
    }
    // spec: S-0009 claim 41 (the placed vehicle numbered i takes follow value i, quirks.follow_index_by_place)
    let links = follow_links(&follow, r.quirks.follow_index_by_place);
    if links.leaders.iter().any(Option::is_some) {
        w.note("S-0009 claim 41: follower-car links are worked out but an object has no leader field yet (S-0212 claim 26)");
    }
    if !links.unspecified.is_empty() || links.mixed {
        w.note("S-0009 claim 41: some vehicle entries lack the follow-index field; the vehicles beyond the follow values have no leader (a departure)");
    }
    if bad_sub {
        w.note("S-0208 claim 5: an infantry sub-cell value outside 0 … 4 was placed at the centre");
    }
}
