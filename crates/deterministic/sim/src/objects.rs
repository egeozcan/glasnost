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

/// The entered-by raise of an object's arrival update in the cell it now
/// occupies (S-0207 and S-0208 claim 22): none when the arrival's earlier
/// effects removed it, and in a high-bridge cell only for an object on the
/// deck (S-0011 claim 18).
pub(crate) fn arrival_raise(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else {
        // spec: S-0207 claim 22 (no arrival raise is promised when the arrival's effects removed the object)
        return;
    };
    if !o.on_map {
        return;
    }
    let c = o.pos.cell();
    // spec: S-0011 claim 18 (in a cell that carries a high bridge only units on the deck raise it) and S-0207 claim 22 (ground passage under a high bridge is no arrival)
    if w.cells.is_bridge(c) && o.layer != Layer::Deck {
        return;
    }
    crate::triggers::cell_entered(w, id, (c.x as i16, c.y as i16));
}

/// The occupancy switches.
pub fn occ_quirks(w: &World) -> OccupancyQuirks {
    let q = w.rules().quirks;
    OccupancyQuirks {
        side_positions_only: q.occupancy_house_side_positions_only,
        stale: q.occupancy_house_stale,
    }
}

/// Creates an object (not yet on the map) with its creation set (S-0611
/// claim 14 and F5). Every creation of an infantry, vehicle, aircraft or
/// building object takes one raw value from the scenario random stream
/// (S-0202 claim 10), then its creation number. A spawner's initial slot
/// objects (S-0506 claim 1) and then a slave type's initial slaves
/// (S-0402 claim 19) are created as part of the creation, each with its
/// own creation set before the next.
pub fn create(w: &mut World, kind: Kind, type_index: u32, owner: Option<HouseId>) -> ObjectId {
    create_in(w, kind, type_index, owner, &mut Vec::new())
}

/// The types whose creation is in progress in one creation set, outermost
/// first (S-0611 F5's chain of spawned and slave types).
pub(crate) type Chain = Vec<(Kind, u32)>;

/// [`create`] inside a creation set whose enclosing creations are `chain`.
pub(crate) fn create_in(
    w: &mut World,
    kind: Kind,
    type_index: u32,
    owner: Option<HouseId>,
    chain: &mut Chain,
) -> ObjectId {
    let id = create_converted(w, kind, type_index, owner);
    fill(w, id, chain);
    id
}

/// Takes a created object that is never placed out of the game again,
/// with its creation set: our engine keeps no never-placed object (S-0611
/// claim 11's departure; S-0216 claim 8's refused try). `unit` must be the
/// newest creation: it and every object created after it (claim 14: its
/// slot objects, slaves and theirs) are removed with the records their
/// creation made; every draw and creation number of the set stays
/// consumed and is never reused (claim 14, Q-1460).
pub fn discard_unplaced(w: &mut World, unit: ObjectId) {
    let made: Vec<ObjectId> = w.objects.range(unit..).map(|(&k, _)| k).collect();
    for id in made {
        w.mind.controllers.remove(&id);
        // spec: S-0611 claim 14 (a refused set never enters play: none of it, nor the controllers it carries, acts in a later tick)
        w.spawned.launcher.remove(&id);
        w.objects.remove(&id);
    }
    // spec: S-0611 claim 14 (one creation number reserved per member of the creation set, a refused attempt included, without reuse: the counter is not rolled back)
}

/// The initial fill of a new object `id` (S-0611 claim 14 and F5): its
/// slot objects in slot order, then its slaves in place order, each with
/// its own creation set before the next.
pub(crate) fn fill(w: &mut World, id: ObjectId, chain: &mut Chain) {
    let Some(t) = w.obj(id).map(|o| (o.kind, o.type_index)) else {
        return;
    };
    if chain.contains(&t) {
        // spec: S-0611 claim 14 and F5, S-0402 claim 19 (revision 4 and 34, Q-1530 answered: the object that closes a cycle of spawned and slave types within its own chain keeps its creation draw and number but gets neither a spawn controller nor a slave group, so no slots or places to fill later; only that object, not every object of its type; the importer warns)
        return;
    }
    chain.push(t);
    // spec: S-0611 claim 14 and F5, S-0506 claim 1 (a spawner gets its spawn controller and its full slots when it is created; the slot objects' draws and numbers come after the spawner's own, in increasing slot index)
    crate::spawn::created_in(w, id, chain);
    // spec: S-0611 claim 14 and F5, S-0402 claim 19 (the group is created as part of the object's own creation; its slaves' draws and numbers follow the master's and those of all initial slot objects and their descendants, in place order)
    crate::econ::slaves::created_in(w, id, chain);
    chain.pop();
}

/// The warnings of S-0611 F5: each spawned or slave type that closes a
/// cycle of spawned and slave types. The importer reports them with the
/// missile rules.
pub(crate) fn creation_cycles(
    missiles: &crate::spawn::MissileRules,
    harvest: &crate::econ::harvest::Rules,
) -> Vec<String> {
    let children = |t: (Kind, u32)| -> Vec<(Kind, u32)> {
        let mut v = Vec::new();
        // spec: S-0611 F5 (S(o): the slot objects of a type naming a spawned type with N ≥ 1)
        if let Some(s) = missiles.spawners.get(&t) {
            v.push((Kind::Aircraft, s.spawned));
        }
        // spec: S-0611 F5 (V(o): the slaves of a type naming a slave type with M ≥ 1)
        if let Some(g) = harvest.group_of(t.0, t.1)
            && g.count >= 1
        {
            v.push((Kind::Infantry, g.slave));
        }
        v
    };
    let mut roots: Vec<(Kind, u32)> = missiles.spawners.keys().copied().collect();
    roots.extend(harvest.masters());
    roots.sort();
    roots.dedup();
    let mut warned = std::collections::BTreeSet::new();
    for root in roots {
        let mut path = vec![root];
        // Each frame holds the children of a path entry still to visit, last first.
        let rev = |mut v: Vec<(Kind, u32)>| {
            v.reverse();
            v
        };
        let mut stack = vec![rev(children(root))];
        while let Some(kids) = stack.last_mut() {
            let Some(k) = kids.pop() else {
                stack.pop();
                path.pop();
                continue;
            };
            if path.contains(&k) {
                warned.insert(k);
                continue;
            }
            path.push(k);
            stack.push(rev(children(k)));
        }
    }
    warned
        .into_iter()
        .map(|(kind, i)| {
            format!("S-0611 F5: the {kind:?} type at index {i} closes a cycle of spawned and slave types; inside that chain it is created with neither a spawn controller nor a slave group")
        })
        .collect()
}

/// [`create`] for the object a conversion makes (S-0415 claims 6 and 10):
/// it takes the replaced object's slave group instead of creating one
/// (S-0402 claim 19).
pub fn create_converted(
    w: &mut World,
    kind: Kind,
    type_index: u32,
    owner: Option<HouseId>,
) -> ObjectId {
    let mut report_value = 0;
    if kind != Kind::Terrain {
        // spec: S-0202 claim 10 (the creation draw: one raw value)
        let x = w.rng.raw();
        // spec: S-0202 claim 10 (Q-0327: the report value r = x mod 65536, kept without another draw)
        report_value = (x & 0xFFFF) as u16;
    }
    let id = w.new_id();
    let r = w.rules_arc();
    let (health, foundation, loco) = match kind {
        Kind::Building => {
            let b = &r.buildings[type_index as usize];
            (b.strength, b.foundation, Locomotor::None)
        }
        // spec: S-0302 claim 20 and S-0009 claim 42 (a terrain object starts with its type's strength; −1 when the type's strength is unset, S-0102 claim 23)
        Kind::Terrain => (
            r.terrain
                .get(type_index as usize)
                .map_or(-1, |t| t.strength),
            (1, 1),
            Locomotor::None,
        ),
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
        report_value,
        rotation_sample: false,
        beh: Behaviours::default(),
        target: None,
        dest: None,
        guard_spot: None,
        queued_dests: Vec::new(),
        attack_move: None,
        contact: None,
        path: PathState::default(),
        // spec: S-0504 claim 1 (a ship moves by the driving rules: it carries the driving locomotor's state)
        drive: matches!(loco, Locomotor::Drive | Locomotor::Ship).then(DriveState::new),
        walk: (loco == Locomotor::Walk).then(WalkState::default),
        infantry: (kind == Kind::Infantry).then(InfantryState::default),
        depot: crate::depot::DepotState::default(),
        teleport: (loco == Locomotor::Teleport).then(crate::teleport::TeleportState::default),
        spawn: None,
        rocket: None,
        parasite: None,
        air: None,
        naval: None,
        // spec: S-0503 Terms (a hovering vehicle keeps its hover state)
        hover: (loco == Locomotor::Hover).then(|| crate::hover::HoverState::new(w.rules().mode)),
        // spec: S-0502 claim 1 and S-0206 claim 14 (a unit that gets the jumpjet locomotor: its fields are the type's, its movement heading east)
        jump: (loco == Locomotor::Jump).then(crate::jump::JumpState::new),
        transport: crate::transport::TransportState::default(),
        fall: crate::fall::FallState::default(),
        crush_slowdown: false,
        targeting_timer: 0,
        last_turn: -1,
        foundation,
        leader: None,
        // spec: S-0207 Constants (starting speed bonus factor b = 1.0); S-0208 Terms (speed multiplier 1.0 at creation)
        speed_bonus: F64::from_i64(1, w.rules().mode),
        combat: crate::combat::state::CombatState::new(
            health,
            crate::combat::fire::initial_ammo(&r, kind, type_index),
            F64::from_i64(1, w.rules().mode),
        ),
        econ: Default::default(),
        cloak: Default::default(),
        sw: Default::default(),
        structure: Default::default(),
        lift: Default::default(),
    };
    w.objects.insert(id, obj);
    veteran_at_creation(w, id);
    // spec: S-0516 claims 2 (a) and 32 (the own cloak flag and a permanent disguise at creation)
    crate::cloak::machine::created(w, id);
    // spec: S-0601 claim 1 (a controller's record is made when it is created)
    crate::mind::created(w, id);
    id
}

/// Experience at creation (S-0311 claim 11): a veteran when the type is in
/// its house's country veteran list for its kind, or when the object's own
/// house has infiltrated a factory of the kind (S-0510 claim 5): its
/// infantry flag makes a trainable infantry unit a veteran, its vehicle
/// flag a trainable vehicle that is not naval and a trainable building that
/// is not naval and undeploys into a vehicle; aircraft are never promoted
/// by an infiltrated factory.
fn veteran_at_creation(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let i = match o.kind {
        Kind::Infantry => 0,
        Kind::Vehicle => 1,
        Kind::Aircraft => 2, // spec: S-0311 claim 11 (the country lists for infantry, vehicles and aircraft)
        Kind::Building => 3, // spec: S-0311 claim 11 (buildings have no country list)
        _ => return,
    };
    let house = o.owner.and_then(|h| w.houses.get(usize::from(h.0)));
    let listed = i < 3 // spec: S-0311 claim 11 (only the three country lists)
        && house
            .and_then(|h| w.rules().combat.countries.get(h.country as usize))
            .is_some_and(|c| c.veteran[i].contains(&o.type_index));
    let trainable = crate::combat::veterancy::type_combat(w, id).is_some_and(|tc| tc.trainable);
    let (vehicle_flag, infantry_flag) = house.map_or((false, false), |h| {
        (
            h.infiltrated_vehicle_factory,
            h.infiltrated_infantry_factory,
        )
    });
    let infiltrated = match o.kind {
        // spec: S-0311 claim 11 and S-0510 claim 5 (an infantry unit: the infiltrated-infantry-factory flag of its own house and a trainable type)
        Kind::Infantry => infantry_flag && trainable,
        // spec: S-0311 claim 11 and S-0510 claim 5 (a vehicle: the infiltrated-vehicle-factory flag, a trainable type that is not naval)
        Kind::Vehicle => {
            vehicle_flag
                && trainable
                && !crate::combat::veterancy::type_combat(w, id).is_some_and(|tc| tc.naval)
        }
        // spec: S-0311 claim 11 (an infiltrated factory never promotes aircraft)
        Kind::Aircraft => false,
        // spec: S-0311 claim 11 (a building: the infiltrated-vehicle-factory flag, a trainable type that is not naval and undeploys into a vehicle)
        _ => {
            let b = w.rules().buildings.get(o.type_index as usize);
            vehicle_flag
                && trainable
                && b.is_some_and(|b| !b.naval && b.raw.undeploys_into.is_some())
        }
    };
    if listed || infiltrated {
        let mode = w.rules().mode;
        if let Some(ob) = w.obj_mut(id) {
            // spec: S-0311 claim 11 (created as a veteran: experience 1)
            ob.combat.experience = F64::from_i64(1, mode).to_f32(mode);
        }
    }
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
    place_with(w, id, pos, facing, false);
}

/// [`place`], saying whether a building is put on the map by a place order
/// (S-0407 claim 10, the `by_order` of `crate::econ::events::entered_map`).
pub fn place_with(w: &mut World, id: ObjectId, pos: Pos, facing: Facing, by_order: bool) {
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
            // spec: S-0406 claim 2 (a building is in its foundation cells only)
            touched = crate::econ::placement::foundation_cells(w, o.type_index, cell);
            for &c in &touched {
                w.cells.enter(c, Layer::Ground, id, true);
                w.cells.claim_building(c, true);
            }
            // spec: S-0314 claim 10 (a building placed for a house adds its self-heal counts)
            crate::combat::heal::building_counts(w, id, o.owner, true);
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
                // spec: S-0503 claim 13 (a newly created hover starts between steps, its flag at its actual position; Q-0691 answered)
                if let Some(h) = w.obj_mut(id).and_then(|ob| ob.hover.as_mut()) {
                    h.claims = vec![(cell, o.layer)];
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
        // spec: S-0503 claim 15 (placement, factory release and teleport replacement do not reset a hover's movement heading, and the body facing is not copied into it; a newly attached hovering locomotor starts at 0, `hover::HoverState::new`; Q-0694 answered)
    }
    if o.kind == Kind::Building {
        // spec: S-0518 claim 3 (placement into play is an acquisition by the owning house)
        crate::linked::acquire(w, id);
    }
    // spec: S-0502 claim 1 (a jumpjet is placed grounded with its mark)
    crate::jump::placed(w, id);
    // spec: S-0502 claim 29 (a unit placed on the map in the air is registered only when it is then airborne and its type has the counts-as-aircraft flag)
    crate::jump::placed_registration(w, id);
    // spec: S-0501 claim 1 (an aircraft's placement height, speed fraction and marks)
    crate::air::placed(w, id);
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
    // spec: S-0112 claim 7 (a gate or wall tower placed recomputes the wall bits of its fixed cells)
    crate::walls::building_placed(w, id);
    if o.kind.is_mobile() {
        // spec: S-0207 claim 27 (successful placement enables destination refresh during the initial idle-behaviour setup and clears it when that setup finishes)
        crate::refresh::placement_begin(w, id);
        // spec: S-0215 Terms (the placement fall-back: an actor falls back to rest when placed)
        crate::process::fall_back(w, id, true);
        crate::refresh::placement_end(w, id);
        // spec: S-0501 Terms and claim 1 (the type test that marks an aircraft visiting comes after the placement fall-back, which sees only a mark already present)
        crate::air::placed_type_test(w, id);
    }
    crate::econ::events::entered_map(w, id, by_order);
}

/// A play-time placement (S-0216 Terms): puts the created vehicle or
/// infantry unit `id`, not yet on the map, at the centre of `cell` on the
/// ground of that cell with facing value `v`, after the game's set-up.
/// Returns whether it was placed. A refused unit stays off the map, with
/// every draw its attempt took; what to do with it is the placing step's
/// (S-0216 Open questions, *A failed placement*).
///
/// The game's own rules that put units on the map during play follow their
/// own specs (S-0216 Terms): no spec yet names this placement, so its
/// callers are placing steps of our own, such as the testkit's harness
/// (`crate::testkit::spawn_in_play`). Set-up placement by the map is
/// S-0009's and S-0208's (claim 5).
pub fn place_in_play(w: &mut World, id: ObjectId, cell: CellXY, v: u8) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let kind = o.kind;
    // spec: S-0216 Terms (a newly created vehicle or infantry unit, not yet on the map)
    if o.on_map || !matches!(kind, Kind::Infantry | Kind::Vehicle) {
        return false;
    }
    // Our design: a cell that is not a legal map cell refuses (no S-0209 entry outcome exists for it).
    let Some(level) = w.cells.cell(cell).map(|c| i32::from(c.level)) else {
        return false;
    };
    let centre = Pos::centre(cell, 0);
    // spec: S-0216 Terms (at the centre of the stated cell, on the ground of that cell)
    let mut pos = Pos::new(centre.x, centre.y, w.surface_at(centre.x, centre.y));
    if kind == Kind::Infantry && w.cells.usable(cell) {
        // spec: S-0216 claim 2 (a usable cell: S-0208 claim 3's choice for a desired point of class centre, one R(0, 3) unless the vehicle-present or object-present refusal applies, on the ground record with the units already in the cell; no free position of row Rk, or no position: the placement fails, the draw kept)
        let Some(k) = walk::choose_position(
            w,
            cell,
            Layer::Ground,
            crate::cells::position::CENTRE,
            false,
        ) else {
            return false;
        };
        let (u, sv) = walk::standing_point(k);
        let (x, y) = (256 * cell.x + u, 256 * cell.y + sv); // spec: S-0204 claim 2 (a position within the cell)
        pos = Pos::new(x, y, w.surface_at(x, y));
    }
    // A cell outside the usable area: S-0216 claim 2 leaves the position and
    // the draw unestablished (PT2 part c). Our design, as S-0614 claim 20's
    // placement: no position choice and no draw, the centre, then the entry
    // test, which for a unit not yet on the map applies no usable-area check.
    // spec: S-0216 claim 5 (the unit's entry outcome for the cell, S-0209 claims 4 and 5 with S-0211 claim 10, asked without an approach direction or source elevation, against the ground occupants and record; only clear accepts, after the standing-position choice)
    let outcome = if kind == Kind::Infantry {
        crate::occupancy::infantry_outcome(w, id, cell, level)
    } else {
        crate::occupancy::vehicle_outcome(w, id, cell, level)
    };
    if outcome != crate::occupancy::Outcome::Clear {
        return false;
    }
    // spec: S-0216 claims 3 and 5 (a vehicle of a type with either visceroid flag: one R(0, 29), only once the entry test has accepted it; any other vehicle: no draw)
    visceroid_draw(w, id);
    if let Some(ob) = w.obj_mut(id) {
        ob.layer = Layer::Ground;
    }
    // spec: S-0216 claim 1 and S-0206 claim 5 (body and turret facings both 256 × v, set at once; the conversion of S-0009 claim 39)
    let facing = facing_from_value(i32::from(v));
    // Infantry facing at play time is S-0216's open question; our design gives it the same conversion.
    place(w, id, pos, facing);
    // spec: S-0216 claim 4 (a unit ready to change, S-0215 claim 11, when it is placed has the placement fall-back's resting behaviour current and nothing queued; one not ready keeps it queued)
    if crate::process::ready(w, id) {
        let (tick, stale) = (w.tick, w.rules().quirks.stale_suspended_revert); // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
        if let Some(ob) = w.obj_mut(id) {
            ob.beh.start_next(tick, stale);
        }
    }
    true
}

/// Takes an infantry unit off the map for a while without ending it: a
/// slave that goes inside its master (S-0402 claim 14). Like [`remove`] it
/// leaves the update order, a counting removal during the turn pass
/// (S-0203 claim 10), and its cell list entry and standing marks are
/// cleared; the object stays in the world, off the map, with no
/// destination, target or walk.
pub fn take_inside(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else {
        return;
    };
    if !o.on_map {
        return;
    }
    // spec: S-0309 claim 27 and S-0501 claim 32 (every holder drops as its target an object that goes off the map, a transport or a building entered, at that moment)
    crate::combat::held::holders_drop(w, id, crate::combat::held::Leaving::OffMap);
    // spec: S-0306 claim 17 (a projectile whose target goes inside: the cell it stood in when not airborne, else nothing)
    crate::combat::projectile::target_leaves(w, id);
    // spec: S-0615 claim 14 (taken off the map: lost to its disc lasers at once)
    crate::disc::object_lost(w, id);
    // spec: S-0608 claim 17 and S-0507 claim 6 (Q-1443: successful temporary off-map storage of either party ends its drain links at once, a stranded earlier link included)
    crate::drain::stored(w, id);
    // spec: S-0614 claim 15 (a host taken off the map releases its parasite at once)
    crate::parasite::leaving(w, id);
    // spec: S-0502 claims 4, 13 and 14 (a jumpjet going inside releases its marks, its landing reservation and its hover slot)
    crate::jump::leaving(w, id);
    // spec: S-0507 claim 15 (an open-topped transport leaving the map: its passengers lose their targets)
    crate::transport::leaving_map(w, id);
    crate::econ::events::leaving_map(w, id, false);
    // spec: S-0609 claims 11 and 13 (leaving the map ends the spin sound's loop and clears the playing mark)
    crate::gattling::leaving_map(w, id);
    // spec: S-0209 claim 1 (an aircraft going inside clears its object-present mark)
    crate::air::clear_mark(w, id);
    // spec: S-0501 claim 47 (b) (going inside a carrier, a transport or a building ends the tracking)
    crate::air::leaving_map(w, id);
    let skip = w.rules().quirks.update_order_removal_skip;
    // spec: S-0203 claim 10 (going inside: a removal from the update order, counting during the turn pass)
    w.update.leave(id, skip);
    let q = occ_quirks(w);
    let cell = o.pos.cell();
    w.cells.leave_both(cell, id);
    if o.kind == Kind::Vehicle {
        // A vehicle going inside (a latching parasite, S-0614 claim 1) releases its claims and its step.
        crate::drive::release_claims(w, id);
        if let Some(ob) = w.obj_mut(id)
            && ob.drive.is_some()
        {
            ob.drive = Some(crate::drive::DriveState::new());
        }
    }
    if let Some(wt) = o.walk.as_ref().and_then(|s| s.target) {
        w.cells.clear_infantry(wt.cell(), o.layer, id, q);
    }
    w.cells.clear_infantry(cell, o.layer, id, q);
    if let Some(ob) = w.obj_mut(id) {
        ob.on_map = false;
        ob.dest = None;
        crate::spawn::clear_own_target(ob);
        ob.queued_dests.clear();
        ob.path.steps.clear();
        if let Some(s) = ob.walk.as_mut() {
            *s = WalkState::default();
        }
    }
    if o.layer == Layer::Ground {
        // spec: S-0202 claim 14 (taking an object out of its occupied ground cell recalculates it)
        crate::load::recalc_cells(w, &[cell]);
    }
    crate::regions::cell_changed(w, cell);
}

/// Takes an object off the map and out of play (destroyed, removed or gone
/// inside): it leaves the update order, a counting removal during the turn
/// pass (S-0203 claim 10), its marks and claims are cleared and its cells
/// recomputed.
pub fn remove(w: &mut World, id: ObjectId) {
    remove_with(w, id, true);
}

/// Takes an object out of the game as [`remove`] does, but without the
/// recalculation of its occupied ground cells, so with no random draw and
/// no tile animation: our arena's *Removal before the groups* (S-0216
/// claim 8, design), which frees the cells and keeps the creation numbers
/// taken.
pub fn take_out_quietly(w: &mut World, id: ObjectId) {
    // spec: S-0216 claim 8 (Removal before the groups: the removal takes no random draw, starts no animation and frees the cells)
    remove_with(w, id, false);
}

/// [`remove`], with the ground-cell recalculation only when `recalc`.
fn remove_with(w: &mut World, id: ObjectId, recalc: bool) {
    let Some(o) = w.obj(id).cloned() else {
        return;
    };
    // spec: S-0309 claim 27 and S-0501 claim 32 (every holder drops as its target an object that is removed, a crush included, at that moment)
    crate::combat::held::holders_drop(w, id, crate::combat::held::Leaving::Removed);
    // spec: S-0306 claim 17 (a projectile whose target is removed from the map: the cell it stood in when not airborne, else nothing; a killed target was dealt with at the kill)
    crate::combat::projectile::target_leaves(w, id);
    // spec: S-0615 claim 14 (removed or taken off the map, a transport or building entered included: lost to its disc lasers at once)
    crate::disc::object_lost(w, id);
    // spec: S-0519 claims 7 and 8 (a removed carrier loses its bomb silently; a bomb whose planter leaves forgets it)
    crate::bomb::removed(w, id);
    // spec: S-0601 claims 10 and 12 (a removed controller releases its victims; a removed victim leaves its controller's list silently)
    crate::mind::removing(w, id);
    // spec: S-0608 claim 13 (a drainer or a drained building removed from the game in any other way ends its links at once)
    crate::drain::removing(w, id);
    // spec: S-0519 claims 19 and 20 (a strike aircraft leaving play is unlinked, a designator ends its strike, a marked building cancels)
    crate::strike::leaving(w, id);
    // spec: S-0518 claim 12 (a charger that leaves play leaves every charger list at once)
    crate::linked::leaving(w, id);
    // spec: S-0514 claims 12 and 14 (a removed target invalidates all attached beams and its attackers idle; a removed attacker's beam lets go)
    crate::temporal::leaving(w, id);
    // spec: S-0614 claim 15 (a host removed while a parasite is attached releases it at once)
    crate::parasite::leaving(w, id);
    // spec: S-0607 claims 12 and 14 (a lifter leaving play drops its gripped unit just before; a gripped unit leaving play ends the grip and clears its lifter's target; a dropper leaving play leaves its dropped units their dropper house)
    crate::lift::removing(w, id);
    // spec: S-0502 claims 4, 7, 13, 14 and 18 (a jumpjet leaving the map releases its marks, its landing reservation and its hover slot)
    crate::jump::leaving(w, id);
    // spec: S-0507 claim 16 (a transport removed with passengers inside: they die without attacker credit; a removed passenger leaves its transport's passenger order)
    crate::transport::removing(w, id);
    // spec: S-0508 claim 15 (a building that leaves play with occupants still inside removes them with it, without death and without putting them out)
    crate::structures::garrison::removing(w, id);
    // spec: S-0605 claim 15 (a passenger removed inside leaves the passenger order; a building removed with passengers deletes them; a waiting actor stops waiting)
    crate::structures::absorb::removing(w, id);
    // spec: S-0606 claims 15 and 19 (another removal: our engine ends the bunker link on both sides silently; a surviving bunker returns to clearing and queues guard)
    crate::structures::bunker::removing(w, id);
    // spec: S-0603 claim 15 (a recycling building that leaves the game: its entrants' destinations are cleared)
    crate::structures::grinder::removing(w, id);
    // spec: S-0402 claim 14 (a lost slave starts its master's regeneration delay) and S-0408 claim 10a (a removed master's slaves are dealt with)
    crate::econ::slaves::removing(w, id);
    // spec: S-0506 claims 9, 11 and 13 (a removed spawner clears its slots; a removed slot object or controller target is announced)
    crate::spawn::removing(w, id);
    // spec: S-0616 claim 4 (every particle system whose owner object is removed loses it and is marked)
    crate::particles::owner_removed(w, id);
    crate::econ::events::leaving_map(w, id, true);
    // spec: S-0609 claims 11 and 13 (leaving the map ends the spin sound's loop and clears the playing mark)
    crate::gattling::leaving_map(w, id);
    // spec: S-0011 claim 2 (detaching a carrier removes one from its tag's count)
    crate::triggers::detach_object(w, id);
    let skip = w.rules().quirks.update_order_removal_skip;
    w.update.leave(id, skip);
    let q = occ_quirks(w);
    let cell = o.pos.cell();
    // spec: S-0202 claim 14 (taking an object out of its occupied ground cells recalculates them; removing one that occupies no ground cell implies no ground-cell recalculation)
    let on_ground = o.on_map
        && o.layer == Layer::Ground
        && !(o.kind == Kind::Aircraft && crate::combat::weapon::in_air(w, id));
    let mut touched = if o.on_map { vec![cell] } else { Vec::new() };
    match o.kind {
        // An object off the map (an item in production, S-0414 claim 15) holds no cell.
        _ if !o.on_map => {}
        Kind::Building => {
            // spec: S-0314 claim 10 (a building removed or destroyed subtracts its counts, flooring at 0)
            crate::combat::heal::building_counts(w, id, o.owner, false);
            // spec: S-0406 claim 2 (a building is in its foundation cells only)
            touched = crate::econ::placement::foundation_cells(w, o.type_index, cell);
            for &c in &touched {
                w.cells.leave(c, Layer::Ground, id);
                w.cells.claim_building(c, false);
            }
        }
        Kind::Terrain => {
            w.cells.leave(cell, Layer::Ground, id);
            w.cells.clear_terrain(cell, id, q);
        }
        Kind::Infantry => {
            w.cells.leave(cell, o.layer, id);
            // spec: S-0611 claim 7 (a refused mutation victim's removal frees its standing position again)
            crate::mutation::removing(w, id);
            if let Some(wt) = o.walk.as_ref().and_then(|s| s.target) {
                w.cells.clear_infantry(wt.cell(), o.layer, id, q);
            }
            // spec: S-0208 claim 19 (a teleporting unit's reserved standing position)
            if let Some(t) = o.teleport.as_ref()
                && let Some(c) = t.mark
            {
                w.cells.clear_infantry(c, t.mark_layer, id, q);
            }
            w.cells.clear_infantry(cell, o.layer, id, q);
        }
        Kind::Vehicle => {
            w.cells.leave(cell, o.layer, id);
            crate::drive::release_claims(w, id);
        }
        // spec: S-0506 claim 23 (a rocket missile is in no cell list and marks no occupancy)
        Kind::Aircraft if o.rocket.is_some() => {}
        Kind::Aircraft => {
            // spec: S-0209 claim 13 (an aircraft is listed in the ground layer, the deck layer or neither, by its listing height)
            w.cells.leave_both(cell, id);
            // spec: S-0207 claim 20 (a) and (e) (a teleporting aircraft's object-present mark is on its landing point's cell until the jump; on the deck its removal clears the deck flag only)
            for (mc, ml) in crate::teleport::removal_marks(w, id) {
                if let Some(rec) = w.cells.occ_mut(mc, ml) {
                    rec.others = rec.others.saturating_sub(1);
                }
            }
        }
    }
    // spec: S-0309 claim 23 (d) (removal of a remembered target turns the memory into the cell of the target's last position; the engaged mark is kept)
    let remembered = crate::world::AttackMoveMemory::Target(crate::world::Target::Object(id));
    for ob in w.objects.values_mut() {
        if ob.attack_move == Some(remembered) {
            ob.attack_move = Some(crate::world::AttackMoveMemory::Dest(
                crate::world::Target::Cell(cell),
            ));
        }
    }
    let gone = w.objects.remove(&id);
    if recalc && let Some(left) = gone {
        // spec: S-0203 claim 13 (a) and (b) (a removed object is kept to the end of the tick, out of play and off the map, with its owner and health; our arena's Removal before the groups, S-0216 claim 8, runs before play and keeps none)
        crate::leftover::keep(w, left);
    }
    if on_ground && recalc {
        crate::load::recalc_cells(w, &touched);
    }
    for c in touched {
        crate::regions::cell_changed(w, c);
    }
    if o.kind == Kind::Building
        && o.health <= 0
        && let Some(j) = crate::walls::joiner(w, o.type_index)
    {
        // spec: S-0112 claim 7 (a destroyed gate or tower that leaves the map recomputes its fixed cells, without it)
        crate::walls::joiner_changed(w, j, cell, o.foundation);
        if j == crate::walls::Joiner::Tower {
            // spec: S-0112 claim 7 (a destroyed wall tower: each wall of any type in an edge neighbour of its top-left cell, at stage 0 when its hit is dealt, takes a wall hit of 200, map north, east, south, west)
            for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
                let n = CellXY::new(cell.x + dx, cell.y + dy);
                let stage0 = w.cells.cell(n).is_some_and(|x| {
                    crate::walls::is_wall(w, x.overlay) && x.overlay_data / 16 == 0 // spec: S-0210 claim 8 (s = ⌊v / 16⌋)
                });
                if stage0 {
                    crate::combat::area::wall_hit(w, n, 200, false); // spec: S-0112 claim 7 (a wall hit of 200; what it does is S-0303 F4)
                }
            }
        }
    }
}

/// Placing a vehicle of a type with either visceroid flag while the map
/// loads takes one ranged draw R(0, 29) (S-0202 claim 10, map-loading table;
/// S-0009 claim 46).
pub fn visceroid_draw(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    if o.kind != Kind::Vehicle {
        return;
    }
    let r = w.rules_arc();
    if r.unit(o.kind, o.type_index).is_some_and(|u| u.visceroid) {
        // spec: S-0202 Constants (visceroid placement draw R(0, 29))
        let _ = w.rng.ranged(0, 29, r.quirks.ranged_draw_half_span);
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
        // spec: S-0309 claim 15 (the expected remaining health starts equal to the health)
        o.combat.expected_health = h;
    }
}

/// The player-at-position names of triggers and teams, positions 0 … 7
/// (S-0009 Import mapping: fixed text of the original's program).
// spec: S-0009 Import mapping (the player-at-position names <Player @ A> … <Player @ H>)
const PLAYER_AT: [&str; 8] = [
    "<Player @ A>",
    "<Player @ B>",
    "<Player @ C>",
    "<Player @ D>",
    "<Player @ E>",
    "<Player @ F>",
    "<Player @ G>",
    "<Player @ H>",
];

/// Places the map's objects (S-0009 claims 27 and 34 to 43): terrain
/// objects, then vehicles, aircraft, infantry and buildings (each building's
/// upgrades directly after it), each kind in entry order (claim 43).
/// Smudges, placed last, hold no simulation state.
pub fn place_map_objects(w: &mut World, map: &Map, sandbox: bool) {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
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
        let exact = w
            .houses
            .iter()
            .position(|h| h.name.as_bytes() == name)
            .map(|i| HouseId(i as u8));
        if exact.is_some() || r.quirks.skirmish_drops_country_owned_objects {
            return exact;
        }
        // spec: S-0009 Edge cases (quirks.skirmish_drops_country_owned_objects off: "player at position X" to that player, a country name to its first player)
        if let Some(k) = PLAYER_AT
            .iter()
            .position(|p| p.as_bytes() == name)
            .map(|k| k as u32)
        {
            return w
                .houses
                .iter()
                .position(|h| h.start == Some(k))
                .map(|i| HouseId(i as u8));
        }
        w.houses
            .iter()
            .position(|h| {
                h.role == crate::world::HouseRole::Player
                    && r.country_names
                        .get(h.country as usize)
                        .is_some_and(|c| c.as_bytes() == name)
            })
            .map(|i| HouseId(i as u8))
    };
    let mut bad_sub = false;
    let mut follow: Vec<FollowEntry> = Vec::new();
    let mut placed_vehicles: Vec<ObjectId> = Vec::new();
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
            if kind == Kind::Building {
                let pt = crate::econ::placement::place_type(w, ti);
                if pt.wall
                    && let Some(ov) = pt.wall_overlay
                {
                    // spec: S-0112 claim 7 (a wall building of the map's building list is placed as a wall built in play: stage 0, owned by the building's owner; c and its four edge neighbours recomputed); S-0406 claim 10 (no building remains)
                    // spec: S-0210 claim 14 (a) (the building's owner is only the initial owner: the nearest-building rule replaces it before play starts, even with no owner as the result; the converted wall building cannot itself qualify; Q-0684 answered)
                    w.objects.remove(&id);
                    // As for a wall placed in play, an overlay already in the cell stays (S-0406 claim 10's interim rule).
                    if w.cells
                        .cell(p.cell)
                        .is_some_and(|x| x.overlay == glasnost_map::NO_OVERLAY)
                    {
                        crate::econ::placement::put_wall(w, p.cell, ov, Some(owner));
                    }
                    continue;
                }
            }
            // spec: S-0011 claim 34 (the object's tag after its creation draw, before the draws of placing it)
            crate::triggers::object_tag(w, id, fields.tag.as_deref());
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
                // spec: S-0009 claim 40 and S-0215 claim 1 (a pre-placed building is complete: the idle body and guard queued)
                crate::econ::body::set_up(w, id);
            } else if let Some(b) = fields.behaviour.as_deref().and_then(behaviour_by_name) {
                // spec: S-0009 claim 41 (the behaviour the object starts with; none: the placement fall-back)
                if let Some(o) = w.obj_mut(id) {
                    o.beh.force(b, t, stale);
                }
            }
            let mut pos = pos;
            if kind == Kind::Vehicle && fields.bridge == Some(true) {
                let deck = w.cells.idx(p.cell).is_some_and(|i| w.cells.deck[i]);
                if deck {
                    // spec: S-0009 claim 38 (a vehicle whose bridge flag is set stands on the bridge deck)
                    // spec: S-0207 claim 5 (on a deck: the surface height plus 416 leptons)
                    pos.z += 416;
                    if let Some(o) = w.obj_mut(id) {
                        o.layer = Layer::Deck;
                    }
                } else {
                    w.note("S-0009 claim 38: a vehicle's bridge flag on a cell without a bridge deck is not specified; it stands on the ground");
                }
            }
            place(w, id, pos, facing);
            visceroid_draw(w, id);
            if kind == Kind::Vehicle {
                placed_vehicles.push(id);
                follow.push(FollowEntry {
                    value: fields.follow,
                    placed: true,
                });
            }
            if kind == Kind::Aircraft {
                w.note("S-0009 claim 38: aircraft stand landed on their cell; one placed in the air needs its type's flight height (aircraft specs)");
            }
            if let Some(n) = fields.veterancy {
                // spec: S-0311 claim 11 and F5 (a map object's experience e = RZ₂₄(RZ₅₃(n × 0.01)), replacing the creation rules, not capped)
                let mode = w.rules().mode;
                let e = glasnost_num::F64::from_i64(i64::from(n), mode)
                    .mul(crate::rules::nearest_double("0.01"), mode)
                    .to_f32(mode);
                if let Some(o) = w.obj_mut(id) {
                    o.combat.experience = e;
                }
            }
            if kind == Kind::Building {
                if !fields.switched_on() && crate::econ::power::map_off_applies(w, ti) {
                    // spec: S-0009 claim 40, S-0405 claim 18 (switched-on field 0: switched off only for a type with a base drain above 0 or the needs-power flag)
                    if let Some(o) = w.obj_mut(id) {
                        o.econ.building.switched_off = true;
                    }
                }
                if fields.ai_repair == Some(true)
                    && let Some(o) = w.obj_mut(id)
                {
                    // spec: S-0314 claim 13 and Import mapping (the map's computer-repair field, nonzero: set)
                    o.combat.computer_repair = true;
                }
                // spec: S-0009 claim 40 (min(c, 3) upgrade fields, each naming a building type, placed right after the building)
                let c = fields.upgrade_count.unwrap_or(0).clamp(0, 3) as usize; // spec: S-0009 claim 40 (a count above 3: our engine uses 3)
                for name in upgrades[n].iter().take(c).flatten() {
                    let Some(ui) = type_index(&r, Kind::Building, name) else {
                        continue;
                    };
                    let uid = create(w, Kind::Building, ui, Some(owner));
                    // spec: S-0009 claim 40 and S-0215 claim 1 (the upgrades start at full health, at the same cell and facing, set up as buildings)
                    crate::econ::body::set_up(w, uid);
                    place(w, uid, pos, facing);
                }
            }
        }
    }
    // spec: S-0009 claim 41 (the placed vehicle numbered i takes follow value i, quirks.follow_index_by_place)
    let links = follow_links(&follow, r.quirks.follow_index_by_place);
    for (i, leader) in links.leaders.iter().enumerate() {
        // spec: S-0009 claim 41 (the placed vehicle numbered i becomes a follower car of the placed vehicle numbered n)
        let leader = leader.and_then(|n| placed_vehicles.get(n).copied());
        if let Some(&id) = placed_vehicles.get(i)
            && let Some(o) = w.obj_mut(id)
        {
            o.leader = leader;
        }
    }
    if links.leaders.iter().any(Option::is_some) {
        w.note("S-0212 claim 26: follower cars are linked to their leaders but not sent after them yet");
    }
    if !links.unspecified.is_empty() || links.mixed {
        w.note("S-0009 claim 41: some vehicle entries lack the follow-index field; the vehicles beyond the follow values have no leader (a departure)");
    }
    if bad_sub {
        w.note("S-0208 claim 5: an infantry sub-cell value outside 0 … 4 was placed at the centre");
    }
}
