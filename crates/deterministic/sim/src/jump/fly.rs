//! The flight update of a jumpjet (S-0502): the physics step and the
//! outcome of each flight state.

use super::*;

/// Removes the old placement before a physics step (claims 4, 22 and 26):
/// the mark and the cell listing go, and the cell of the position is
/// recomputed, even high above it or without horizontal travel.
fn remove_placement(w: &mut World, id: ObjectId) {
    let Some(cell) = w.obj(id).map(|o| o.pos.cell()) else {
        return;
    };
    lift_mark(w, id);
    unlist(w, id);
    // spec: S-0502 claim 26 and S-0211 claim 5 (in every state but hovering and cruising, the cell before every physics step is recomputed on removal of the old placement)
    crate::load::recalc_cells(w, &[cell]);
}

/// The new placement after a physics step (claims 4 and 26): the on-deck
/// state ends at the first rise; a position that qualifies is listed (on the
/// deck when the on-deck state is set, on the ground otherwise) and marked,
/// and its cell is recomputed.
fn place_after_step(w: &mut World, id: ObjectId) {
    // spec: S-0502 claim 4 (at the first rise from the surface the on-deck state ends)
    if height_above(w, id) > 0 && st(w, id).is_some_and(|j| j.on_deck) {
        upd(w, id, |j| j.on_deck = false);
        if let Some(o) = w.obj_mut(id) {
            o.layer = Layer::Ground;
        }
    }
    // spec: S-0502 claim 4 (the new position is eligible for listing only when it qualifies; a jumpjet does not use the aircraft's positive-listing-height exclusion)
    if !placement_qualifies(w, id) {
        return;
    }
    let layer = if st(w, id).is_some_and(|j| j.on_deck) {
        Layer::Deck
    } else {
        Layer::Ground
    };
    list_at(w, id, layer);
    // spec: S-0502 claim 4 (actual-position marking follows the ordinary kind-specific rules of S-0209 claim 1, here the mark of the unit at its position)
    lay_mark(w, id);
    if let Some(cell) = w.obj(id).map(|o| o.pos.cell()) {
        // spec: S-0502 claim 26 (the resulting cell is also recomputed when the new position qualifies for ground-layer placement)
        crate::load::recalc_cells(w, &[cell]);
    }
}

/// Everything a physics step does to the jumpjet: speed, height, step,
/// facing and occupancy (claims 4, 6, 8, 10, 22 and 26).
fn physics(w: &mut World, id: ObjectId) {
    let mode = w.rules().mode;
    let Some(f) = fields(w, id) else { return };
    // spec: S-0502 claims 4 and 22 (hovering and cruising physics make no new listing or mark and leave an existing listing alone; every other state removes its own listing before the clearance is evaluated)
    let placing = st(w, id).is_some_and(|j| !matches!(j.phase, Phase::Hovering | Phase::Cruising));
    if placing {
        remove_placement(w, id);
    }
    let Some(mut j) = st(w, id).cloned() else {
        return;
    };
    let tick = w.tick;
    if j.phase == Phase::Crashing && seized(w, id) {
        // spec: S-0607 claim 15 (while a seized unit is crashing, its jumpjet target height is 0 in every update)
        j.ht = 0;
    }
    speed_step(&mut j, &f, mode);
    let mut z = height_step(w, id, &mut j, &f, mode);
    let p = pos(w, id);
    let heading = j.heading.value(tick);
    let (nx, ny) = step_position(w, p, heading, j.vc.to_i32_common(), mode);
    let crashing = j.phase == Phase::Crashing;
    let entering = j.crash_started && !matches!(j.phase, Phase::Crashing | Phase::Crashed);
    j.crossed_deck = false;
    let mut dropped = None;
    let held = seized(w, id);
    if crashing || entering {
        // spec: S-0502 claim 18 (the extra crash drop k after the ordinary height step, the resulting world height truncated toward zero; in the update that starts the crash the ordinary physics still used the preceding state)
        let before = z;
        let rate = if held { crate::lift::rate(w, id) } else { None };
        let k = rate.map_or(f.crash, |r| r.to_f64());
        z = int(z, mode).sub(k, mode).to_i32_common();
        if let Some(r) = rate {
            // spec: S-0502 claim 18 (magnetic lift: the crash rate grows by 1 lepton per tick, each tick, each new rate narrowed to single precision; S-0607 claim 15: the extra drop uses the current rate, and only after that drop does the rate grow by 1.0)
            let grown = r.to_f64().add(int(1, mode), mode).to_f32(mode);
            if let Some(o) = w.obj_mut(id) {
                o.lift.rate = Some(grown);
            }
        }
        let deck = w.surface_at(p.x, p.y) + BRIDGE_HEIGHT;
        // spec: S-0502 claim 18 (over a high-bridge cell the drop ends the fall when its height before is at or above the deck and after it below)
        j.crossed_deck = w.high_bridge(p.cell()) && before >= deck && z < deck;
        dropped = Some((before, z));
    }
    if crashing && !held {
        // spec: S-0502 claim 18 (magnetic lift: a seized unit does not spin) and S-0206 claim 14 (the heading's target is 11520 facing units clockwise ahead of its current value, turning at the heading's rate)
        let target = (i32::from(heading) + CRASH_SPIN).rem_euclid(65536) as Facing;
        j.heading.turn_to(target, tick, f.omega);
    }
    // spec: S-0502 claim 10 (a hovering infantry jumpjet with the jumpjet-turn flag and a target sets its movement heading at once to the direction of the target, S-0204 claim 8; its body facing is not set from the heading in that step)
    let turn_flag = w.obj(id).is_some_and(|o| {
        o.kind == Kind::Infantry
            && w.rules()
                .unit(o.kind, o.type_index)
                .is_some_and(|u| u.vehicle.jump_jet_turn)
    });
    let target = w.obj(id).and_then(|o| o.target);
    let snap = turn_flag && j.phase == Phase::Hovering && target.is_some();
    if snap && let Some(tg) = target {
        let t = crate::process::target_pos(w, tg);
        let d = direction(
            i64::from(t.x - p.x),
            i64::from(t.y - p.y),
            w.rules().quirks.table_direction,
        );
        j.heading = Turning::at(d);
    }
    let keeps = w.rules().quirks.set_facing_keeps_target;
    // The ordinary step ends at the height before the crash drop.
    let step_z = dropped.map_or(z, |(before, _)| before);
    if let Some(o) = w.obj_mut(id) {
        o.jump = Some(j);
        o.pos.z = step_z;
        if !snap && !held {
            // spec: S-0502 claim 10 (the body facing is set to the movement heading in each physics step; a seized unit keeps its body facing, S-0607 claim 7)
            o.body.set(heading, tick, keeps);
        }
    }
    if (nx, ny) != (p.x, p.y) {
        move_to(w, id, nx, ny);
    }
    if placing {
        place_after_step(w, id);
    }
    if let Some((_, after)) = dropped {
        // spec: S-0502 claim 26 and S-0211 claim 5 (a crash's additional drop likewise recomputes its pre-drop cell and supplies another placement and recompute only when the new position qualifies under claim 4; the events of one tick are not collapsed)
        if placing {
            remove_placement(w, id);
        }
        if let Some(o) = w.obj_mut(id) {
            o.pos.z = after;
        }
        if placing {
            place_after_step(w, id);
        }
    }
}

// ---------------------------------------------------------------------------
// State outcomes (claims 5, 7 and 11 to 15)

fn dest_point(w: &World, id: ObjectId) -> Option<Pos> {
    st(w, id).and_then(|j| j.dest)
}

fn turn_towards(w: &mut World, id: ObjectId, d: Pos) {
    let Some(f) = fields(w, id) else { return };
    let p = pos(w, id);
    let target = direction(
        i64::from(d.x - p.x),
        i64::from(d.y - p.y),
        w.rules().quirks.table_direction,
    );
    let tick = w.tick;
    upd(w, id, |j| {
        j.heading.turn_to(target, tick, f.omega);
    });
}

fn set_phase(w: &mut World, id: ObjectId, phase: Phase) {
    upd(w, id, |j| j.phase = phase);
}

/// Sets the maximum speed to an integer speed.
fn set_vm(w: &mut World, id: ObjectId, v: i32) {
    let mode = w.rules().mode;
    upd(w, id, |j| j.vm = int(v, mode));
}

/// Hovers over the present cell, or, when another jumpjet holds the slot,
/// takes a neighbour step (claims 7, 12 and 14).
fn hover_or_step(w: &mut World, id: ObjectId) {
    let c = pos(w, id).cell();
    if slot_held(w, id, c) {
        neighbour_step(w, id);
    } else {
        claim_slot(w, id, c);
        set_phase(w, id, Phase::Hovering);
    }
}

fn start_descent(w: &mut World, id: ObjectId) {
    upd(w, id, |j| {
        // spec: S-0502 claim 15 (the target height is 0: the jumpjet descends by c per tick)
        j.ht = 0;
        j.phase = Phase::Descending;
    });
}

fn grounded_outcome(w: &mut World, id: ObjectId) {
    let Some(f) = fields(w, id) else { return };
    if !moving(w, id) {
        return;
    }
    let mode = w.rules().mode;
    // spec: S-0502 claim 5 (in its first update with a move order a grounded jumpjet: speed fraction 1, current and maximum speeds 0, target height H, ascending)
    upd(w, id, |j| {
        // spec: S-0502 claims 5 and 25 (the speed fraction becomes 1, except 0 for S = 0: engine design rule, no quirk switch)
        j.fraction = if f.speed == 0 {
            F64::ZERO
        } else {
            int(1, mode)
        };
        j.vc = F64::ZERO;
        j.vm = F64::ZERO;
        j.ht = f.height;
        j.phase = Phase::Ascending;
    });
    took_off(w, id);
}

fn ascending_outcome(w: &mut World, id: ObjectId) {
    let h = adjusted_height(w, id);
    if let Some(j) = st(w, id)
        && w.obj(id).is_some_and(|o| o.contact.is_some())
    {
        // spec: S-0417 claim 15 and S-0502 claim 7 (the factory ascent report: the integer window trunc(ht / q) < h < ht after this update's physics, q = 2 for a balloon-hover vehicle and 4 otherwise; with the design fallback, the update in which h reaches or passes ht reports too)
        let q = if balloon_vehicle(w, id) { 2 } else { 4 };
        let ht = j.ht;
        if (ht / q < h && h < ht) || h >= ht {
            crate::econ::release::ascent_report(w, id);
        }
    }
    let Some(f) = fields(w, id) else { return };
    let Some(j) = st(w, id).cloned() else { return };
    let r = height_above(w, id);
    // spec: S-0502 claim 7 (one quarter of the target height, one half for a balloon-hover vehicle, each truncated toward zero)
    let threshold = if balloon_vehicle(w, id) {
        j.ht / 2
    } else {
        j.ht / 4
    };
    if h < j.ht {
        if h > threshold {
            // spec: S-0502 claim 7 (past the threshold the maximum speed becomes S and the movement heading starts turning towards the destination)
            set_vm(w, id, f.speed);
            if let Some(d) = j.dest {
                turn_towards(w, id, d);
                let over = d.cell() == pos(w, id).cell();
                if over && !balloon(w, id) {
                    // spec: S-0502 claim 7 and quirks.jumpjet_bridge_early_cruise_height (over its destination cell and not of a balloon-hover type: the target height becomes the present height r, or with the switch off h, and it cruises)
                    let keep = if w.rules().quirks.jumpjet_bridge_early_cruise_height {
                        r
                    } else {
                        h
                    };
                    upd(w, id, |j| {
                        j.ht = keep;
                        j.phase = Phase::Cruising;
                    });
                }
            }
        }
    } else {
        // spec: S-0502 claim 7 (once h reaches the target height: a neighbour step and cruising when another jumpjet holds the slot of its cell, otherwise it claims the slot and hovers)
        hover_or_step(w, id);
    }
}

/// The horizontal distance ρ from the position to a point, the table square
/// root truncated (S-0204 claim 5).
fn rho(w: &World, p: Pos, d: Pos) -> i32 {
    let (dx, dy) = (i64::from(d.x - p.x), i64::from(d.y - p.y));
    let mode = w.rules().mode;
    let r = root_floor(
        (dx * dx + dy * dy) as u64,
        w.rules().quirks.table_root_distance,
        mode,
    );
    i32::try_from(r).unwrap_or(i32::MAX)
}

fn cruising_outcome(w: &mut World, id: ObjectId) {
    let Some(f) = fields(w, id) else { return };
    let Some(d) = dest_point(w, id) else { return };
    let mode = w.rules().mode;
    turn_towards(w, id, d);
    let p = pos(w, id);
    let r = rho(w, p, d);
    // spec: S-0502 claim 11 (ρ < 20 leptons: the jumpjet arrives)
    if r < ARRIVAL_RADIUS {
        arrive(w, id, d);
        return;
    }
    let Some(j) = st(w, id).cloned() else { return };
    let none = !has_target(w, id);
    let s = f.speed;
    let t = f.turn_field;
    let tick = w.tick;
    // spec: S-0502 F6 (the remaining turn δ = (desired heading − current heading) mod 65536; D = ((⌊δ / 128⌋ + 1) div 2) mod 256; quirks.jumpjet_turn_slowdown_one_sided off: the smaller of the clockwise and anticlockwise differences)
    let delta = (i32::from(j.heading.target) - i32::from(j.heading.value(tick))).rem_euclid(65536);
    let delta = if w.rules().quirks.jumpjet_turn_slowdown_one_sided {
        delta
    } else {
        delta.min(65536 - delta) // spec: S-0502 F6 and Edge cases (the folded difference 0 … 32768)
    };
    let dd = ((delta / 128 + 1) / 2) % 256; // spec: S-0502 F6 (D = ((⌊δ / 128⌋ + 1) div 2) mod 256)
    let one = int(1, mode);
    // spec: S-0502 F6 and Constants (the cruise bands: speed divisors 8, 4, 10, 2 and 5, distance factor 50, the middle band's height factor 0.75, the low bands' height divisor 2)
    let (vm, ht) = if r < s {
        // spec: S-0502 F6 (ρ < S: S / 8, and H / 2 when it has no target)
        (int(s / 8, mode), none.then_some(f.height / 2))
    } else if r < 2 * s {
        // spec: S-0502 F6 (the second band: ρ < 2S)
        // spec: S-0502 F6 (ρ < 2S: S / 4, and max(S / 10, 1.0) when D > T; H / 2 when it has no target)
        let v = if dd > t {
            int(s / 10, mode).max(one)
        } else {
            int(s / 4, mode)
        };
        (v, none.then_some(f.height / 2)) // spec: S-0502 F6 (H / 2 when it has no target)
    } else if t > 0 && r < (50 * s) / t {
        // spec: S-0502 F6 and Constants (the third band: ρ < (50 × S) div T, band distance factor 50)
        // spec: S-0502 F6 (ρ < (50 × S) div T: S / 2, and max(S / 5, 1.0) when D > 5T; trunc(H × 0.75) when it has no target; T = 0 gives an empty band, S-0502 claim 25)
        let v = if dd > 5 * t {
            int(s / 5, mode).max(one)
        } else {
            int(s / 2, mode)
        };
        (v, none.then_some(f.height * 3 / 4)) // spec: S-0502 F6 (trunc(H × 0.75) when it has no target)
    } else {
        // spec: S-0502 F6 (otherwise S at the full height H, whether or not it has a target)
        (int(s, mode), Some(f.height))
    };
    // spec: S-0502 claim 25 (S = 0: every cruise-band speed is 0, a speed floor included; T = 0 leaves the middle band empty above)
    let vm = if s == 0 { F64::ZERO } else { vm };
    upd(w, id, |j| {
        j.vm = vm;
        if let Some(h) = ht {
            j.ht = h;
        }
    });
    // spec: S-0502 claim 11 (full height kept: a balloon-hover type, a destination cell of water or beach, a deploy-to-land vehicle)
    let wet = w
        .cells
        .cell(d.cell())
        .is_some_and(|c| c.land == land::WATER || c.land == land::BEACH);
    if balloon(w, id) || wet || deploy_to_land(w, id) {
        upd(w, id, |j| j.ht = f.height);
    }
}

fn arrive(w: &mut World, id: ObjectId, d: Pos) {
    // spec: S-0502 claim 12 (the current and maximum speeds become 0 and the position moves exactly onto the destination point's horizontal position)
    upd(w, id, |j| {
        j.vc = F64::ZERO;
        j.vm = F64::ZERO;
    });
    move_to(w, id, d.x, d.y);
    let target = has_target(w, id);
    let balloon = balloon(w, id);
    if seized(w, id) {
        // spec: S-0502 claim 12 (held by magnetic lift: target height 0 and descending; S-0607 claim 8 owns the arrival drop)
        start_descent(w, id);
        // spec: S-0607 claims 8 and 15 (in the same update the lifter's grip ends; a dropped unit's detour ends over its cell centre and its fall begins)
        crate::lift::arrived(w, id);
    } else if !target && !balloon && !deploy_to_land(w, id) {
        // spec: S-0502 claim 12 (no target, not a balloon-hover type, not a deploy-to-land vehicle: target height 0 and descending)
        start_descent(w, id);
    } else {
        // spec: S-0502 claim 12 (a deploy-to-land vehicle with no target, or a unit with a target or of a balloon-hover type: it claims its cell's hover slot and hovers, or takes a neighbour step when another jumpjet holds the slot)
        hover_or_step(w, id);
    }
}

fn hovering_outcome(w: &mut World, id: ObjectId) {
    if !moving(w, id) {
        return;
    }
    let Some(f) = fields(w, id) else { return };
    let p = pos(w, id);
    let Some(j) = st(w, id).cloned() else { return };
    // spec: S-0502 claim 13 (row 1: the destination's horizontal position differs from its own: the heading turns towards it, the slot is released and it cruises)
    if let Some(d) = j.dest
        && (d.x, d.y) != (p.x, p.y)
    {
        turn_towards(w, id, d);
        release_slot(w, id);
        set_phase(w, id, Phase::Cruising);
        return;
    }
    // spec: S-0502 claim 13 (row 2: a target: it keeps hovering)
    if has_target(w, id) {
        return;
    }
    let held = seized(w, id);
    if deploy_to_land(w, id) && !held && !j.landing_request {
        // spec: S-0502 claim 13 (row 3: a deploy-to-land vehicle with no landing request keeps hovering, its target height set to H; S-0517 claim 5)
        upd(w, id, |j| j.ht = f.height);
        return;
    }
    if simple_deployer(w, id) && !held {
        // spec: S-0502 claim 13 (row 4: any other simple-deployer vehicle releases its slot and descends)
        release_slot(w, id);
        start_descent(w, id);
        return;
    }
    if balloon(w, id) {
        // spec: S-0502 claim 13 (row 5: a balloon-hover type keeps hovering)
        return;
    }
    // spec: S-0502 claim 13 (row 6: otherwise it releases its slot and descends)
    release_slot(w, id);
    start_descent(w, id);
}

/// The cell layer a landing at `d` ends on (claim 15): the deck of a
/// high-bridge cell when the point is above the terrain.
fn landing_layer(w: &World, d: Pos) -> Layer {
    if w.high_bridge(d.cell()) && d.z > w.surface_at(d.x, d.y) {
        Layer::Deck
    } else {
        Layer::Ground
    }
}

/// The landing check of claim 15: whether the destination cell passes; on a
/// pass the landing reservation is made.
fn landing_check(w: &mut World, id: ObjectId, d: Pos) -> bool {
    let Some(j) = st(w, id).cloned() else {
        return true;
    };
    let holds = j.reservation.is_some();
    let layer = landing_layer(w, d);
    let cell = d.cell();
    let m = mark_at(w, id, d, layer);
    let infantry = m.position.is_some();
    // spec: S-0502 claim 15 (infantry: the standing position at the destination point is free, or the jumpjet already holds its landing reservation; a vehicle always passes this part)
    if infantry
        && let Some(p) = m.position
        // spec: S-0502 claim 4 (a descent can supply an actual-position mark of its own as well as the landing reservation): the unit's own mark does not take the standing position from it
        && w.cells.occ(cell, layer).is_some_and(|r| {
            r.marks[usize::from(p)].is_some_and(
                |k| !matches!(k.by, crate::cells::Marker::Infantry(who, _) if who == id),
            )
        })
        && !holds
    {
        return false;
    }
    // spec: S-0502 claim 15 (unless the cell has a high bridge, the entry outcome for the jumpjet, S-0209 claims 4 and 5, is clear or cloaked blocker, or moving blocker when it already holds its reservation)
    if !w.high_bridge(cell) {
        let reference = crate::occupancy::reference_level(w, id);
        let out = if infantry {
            crate::occupancy::infantry_outcome(w, id, cell, reference)
        } else {
            crate::occupancy::vehicle_outcome(w, id, cell, reference)
        };
        use crate::occupancy::Outcome as O;
        let ok = matches!(out, O::Clear | O::CloakedBlocker) || (out == O::MovingBlocker && holds);
        if !ok {
            return false;
        }
    }
    if !holds {
        // spec: S-0502 claim 15 (on a pass it makes its landing reservation)
        set_mark(w, id, m);
        upd(w, id, |j| j.reservation = Some(m));
    }
    true
}

fn descending_outcome(w: &mut World, id: ObjectId) {
    let Some(f) = fields(w, id) else { return };
    let Some(d) = dest_point(w, id) else { return };
    let wet = w
        .cells
        .cell(d.cell())
        .is_some_and(|c| c.land == land::WATER || c.land == land::BEACH);
    let skip = entering(w, id) || seized(w, id);
    if wet && !skip {
        // spec: S-0502 claim 14 (a descending jumpjet whose destination is water or beach, neither entering nor seized, does not land: it claims the hover slot of that cell, sets its target height to H and hovers; a held slot gives a neighbour step and cruising)
        upd(w, id, |j| j.ht = f.height);
        let c = d.cell();
        if slot_held(w, id, c) {
            neighbour_step(w, id);
        } else {
            claim_slot(w, id, c);
            set_phase(w, id, Phase::Hovering);
        }
        return;
    }
    if !skip && !landing_check(w, id, d) {
        // spec: S-0502 claim 15 (a failed landing check stops the movement, claim 17)
        stop(w, id);
        return;
    }
    if simple_deployer(w, id) && !seized(w, id) {
        // spec: S-0502 claim 15 (a simple-deployer vehicle not seized starts to turn its heading towards the deploy direction: (8192 × b) mod 65536, integer arithmetic)
        let b = i64::from(w.rules().econ.production.deploy_dir);
        let target = (8192 * b).rem_euclid(65536) as Facing; // spec: S-0502 claim 15 (the target heading (8192 × b) mod 65536 facing units)
        let tick = w.tick;
        upd(w, id, |j| {
            j.heading.turn_to(target, tick, f.omega);
        });
    }
    // spec: S-0502 claim 15 (descent: the target height is 0)
    upd(w, id, |j| j.ht = 0);
    // spec: S-0502 claim 15 (touchdown: the bridge-adjusted height h reaches 0)
    if adjusted_height(w, id) <= 0 {
        touchdown(w, id, d);
    }
}

fn touchdown(w: &mut World, id: ObjectId, d: Pos) {
    let layer = landing_layer(w, d);
    let alive = w.obj(id).is_some_and(|o| o.health > 0);
    release_reservation(w, id);
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0502 claim 15 (its position becomes the destination point, on the deck when it is above the terrain on a high-bridge cell; its destination and move order are cleared)
        o.pos = d;
        o.dest = None;
    }
    upd(w, id, |j| {
        j.fraction = F64::ZERO;
        j.dest = None;
        j.order = None;
        j.moving = false;
        j.phase = Phase::Grounded;
        j.ht = 0;
        j.landing_request = false;
    });
    release_slot(w, id);
    touchdown_registration(w, id, alive);
    settle(w, id, layer);
    if alive {
        // spec: S-0502 claim 15 (still in play and alive: its sight is refreshed at its new position, it releases its hover slot and enters grounded, it attempts crate pickup, a deploy-to-land vehicle's landing request ends)
        crate::econ::shroud::refresh_sight(w, id);
        crate_pickup(w, id);
    }
    // spec: S-0607 Terms and claim 17 (a seized unit that touches down is set down and no longer dropped or crashing)
    crate::lift::touched_down(w, id);
    // spec: S-0502 claim 15 and S-0516 claims 13 and 22 (a completed touchdown is a cell-arrival update: the sensor disc transfers to the landing cell and the adjacency-reveal test applies)
    crate::cloak::machine::cell_movement_completed(w, id);
}

/// Whether a touchdown ends the unit's registration as a tracked flying
/// object (S-0502 claim 29): it does, before the unit enters *grounded*,
/// except with `quirks.jumpjet_falling_touchdown_tracked` on for a unit in
/// the bridge or parachute falling state of claim 4, or no longer alive or
/// in play, which stays registered.
fn touchdown_registration(w: &mut World, id: ObjectId, alive: bool) {
    let in_play = w.obj(id).is_some_and(|o| o.on_map);
    let falling = crate::fall::falling(w, id);
    // spec: S-0502 claim 29 and Edge cases (quirks.jumpjet_falling_touchdown_tracked on: a touchdown while falling, or no longer alive or in play, leaves it registered; off: every touchdown ends registration)
    let keeps =
        w.rules().quirks.jumpjet_falling_touchdown_tracked && (falling || !alive || !in_play);
    if !keeps {
        // spec: S-0502 claim 29 (revision 18, Q-1500 answered: registration ends in the touchdown tick, before it enters grounded)
        crate::combat::flying::unregister(w, id);
    }
}

fn crashing_outcome(w: &mut World, id: ObjectId) {
    let Some(j) = st(w, id).cloned() else { return };
    if j.crash_started && !matches!(j.phase, Phase::Crashing | Phase::Crashed) {
        // spec: S-0502 claim 18 (the crash enters crashing with target height −5 leptons; the new target first affects the ordinary height control in the following update)
        upd(w, id, |j| {
            j.phase = Phase::Crashing;
            j.ht = CRASH_TARGET;
            j.crash_started = false;
        });
        // spec: S-0502 claim 18 (an infantry jumpjet switches to its air-death-start sequence, S-0107 claim 21)
        if w.obj(id).is_some_and(|o| o.kind == Kind::Infantry) {
            crate::infantry::request(w, id, crate::infantry::Seq::AirDeathStart, true);
        }
    }
    if st(w, id).is_none_or(|j| j.phase != Phase::Crashing) {
        return;
    }
    // spec: S-0502 claim 18 (the ordinary crash end: the height above the surface below 1 lepton, or a drop that crosses a deck)
    if height_above(w, id) < 1 || j.crossed_deck {
        if crate::lift::dropped(w, id) {
            // spec: S-0502 claim 18 and S-0607 claims 15 to 17 (a dropped seized unit's fall ends: what lies underneath, then water, beach, clear or blocked, in the same update)
            crate::lift::fall_ended(w, id);
            return;
        }
        release_slot(w, id);
        upd(w, id, |j| {
            j.vm = F64::ZERO;
            j.ht = 0;
            j.phase = Phase::Crashed;
        });
        // spec: S-0502 claim 29 (revision 18, Q-1500 answered: registration ends when it enters crashed, before S-0312 claim 16's impact, so its own impact never finds it through the flying-object search)
        crate::combat::flying::unregister(w, id);
        impact(w, id);
    }
}

/// The impact of an ordinary crash (S-0312 claim 16): a shot-down jumpjet's
/// death weapon goes off at the impact point and it leaves play; one that
/// still lives is destroyed by damage equal to its health, with the
/// demolition warhead, ignoring defences, with the no-escape marker and no
/// attacker.
pub fn impact(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    // spec: S-0312 claim 16 (the impact is on the ground: the unit is at the surface under it, which also ends the crash for a unit that a deck crossing stopped above the terrain)
    let surface = w.surface_at(o.pos.x, o.pos.y);
    if let Some(ob) = w.obj_mut(id) {
        ob.pos.z = surface;
    }
    let at = Pos::new(o.pos.x, o.pos.y, surface);
    if damage::dead(&o) {
        // spec: S-0312 claim 16 (shot down: its death weapon goes off at the impact point, and it leaves play)
        crate::combat::death::impact_death_weapon(w, id, at);
        crate::combat::death::leave_play(w, id);
        return;
    }
    let warhead = w.rules().combat.general.c4_warhead;
    let amount = o.health;
    // spec: S-0312 claim 16 (alive: destroyed by damage equal to its health, the demolition warhead, ignoring defences, no-escape, no attacker)
    damage::apply(
        w,
        id,
        Hit {
            amount,
            warhead,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: true,
        },
    );
    if let Some(h) = warhead {
        // spec: S-0312 claim 16 (the explosion for that amount, S-0303 F5, and a detonation with no source object or house at the impact point)
        crate::combat::projectile::detonate_area(w, h, amount, at, None, None, true);
    }
}

/// A jumpjet above the surface that has been killed, or a crash started
/// without a kill, starts to crash (S-0312 claim 16): the passengers die,
/// then the three tumbling draws (none for infantry); the fall begins in the
/// next update (S-0502 claim 18). Returns whether a crash started.
pub fn start_crash(w: &mut World, id: ObjectId, hit: Option<&Hit>) -> bool {
    if !is_jumpjet(w, id) || height_above(w, id) <= 0 {
        return false;
    }
    if crashing(w, id) {
        return true;
    }
    if seized(w, id) {
        // spec: S-0502 claim 18 (a unit held by magnetic lift does not start this fall)
        return false;
    }
    let none = Hit {
        amount: 0,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    };
    // spec: S-0312 claim 16 (an object that starts to crash loses all its passengers, credited to the attacker)
    crate::transport::kill_passengers(w, id, hit.unwrap_or(&none));
    let infantry = w.obj(id).is_some_and(|o| o.kind == Kind::Infantry);
    if !infantry {
        let mode = w.rules().mode;
        let half = w.rules().quirks.ranged_draw_half_span;
        // spec: S-0312 claim 16 (tumbling, unless infantry: one unit-interval value, one ranged draw over 0 to 1, one more unit-interval value)
        let u1 = w.rng.unit(mode);
        let sense = w.rng.ranged(0, 1, half);
        let u3 = w.rng.unit(mode);
        // spec: S-0113 claim 16 (the three draws set the crashing body's rates; the view keeps them, presentation only)
        w.effects
            .push(crate::combat::effects::Effect::CrashTumble { id, u1, sense, u3 });
    }
    upd(w, id, |j| j.crash_started = true);
    true
}

// ---------------------------------------------------------------------------
// The update

/// One update of a jumpjet in its turn (S-0502 claim 3); whether it moved.
pub fn tick(w: &mut World, id: ObjectId) -> bool {
    if !is_jumpjet(w, id) {
        return false;
    }
    if seized(w, id) {
        // spec: S-0607 claims 7 and 9 (a seized unit's destination is its lifter or nothing)
        crate::lift::steer(w, id);
    }
    // An order-level destination changed without the assignment hook (a rule that writes the field) is a new order.
    let dest = w.obj(id).and_then(|o| o.dest);
    if st(w, id).is_some_and(|j| j.order != dest) {
        take_destination(w, id);
    }
    let Some(j) = st(w, id) else { return false };
    let now = !matches!(j.phase, Phase::Grounded | Phase::Hovering);
    // spec: S-0502 claim 3 (a jumpjet that is neither moving nor moving now takes no update: its height, wobble phase and speed fraction do not change); a pending aerial crash start also guarantees the next update, a safety departure and engine design rule (claim 3, S-0312 claim 16)
    if !(j.moving || now || j.crash_started) {
        return false;
    }
    let before = pos(w, id);
    let start = j.phase;
    // spec: S-0102 claim 4 (the layer is evaluated just before the tick's movement step)
    let heights = layer_heights(w, id);
    physics(w, id);
    if w.obj(id).is_none() {
        return false;
    }
    match phase(w, id) {
        Some(Phase::Grounded) => grounded_outcome(w, id),
        Some(Phase::Ascending) => ascending_outcome(w, id),
        Some(Phase::Hovering) => hovering_outcome(w, id),
        Some(Phase::Cruising) => cruising_outcome(w, id),
        Some(Phase::Descending) => descending_outcome(w, id),
        Some(Phase::Crashing | Phase::Crashed) | None => {}
    }
    // The crash start and the fall's end are judged after the state's outcome (claim 18).
    if w.obj(id).is_some() {
        crashing_outcome(w, id);
    }
    if matches!(start, Phase::Ascending | Phase::Cruising) {
        // spec: S-0502 claim 29 (revision 18, Q-1500 answered: in each update of ascending or cruising over a different cell, the registration moves to the new cell's sector with S-0303 claim 11's arrival order; hovering, descending and crashing updates do not move it)
        crate::combat::flying::moved(w, id);
    }
    let end = phase(w, id);
    // spec: S-0102 claim 4 (a crash fall tick re-files: the update entering crashing, each crashing update and the one ending the fall)
    let crash_tick = start == Phase::Crashing
        || end == Some(Phase::Crashing)
        || (end == Some(Phase::Crashed) && start != Phase::Crashed);
    if crash_tick {
        crate::combat::effects::layer_refile(w, id);
    } else {
        // spec: S-0102 claim 4 (and just after it: re-filed only when the two evaluations differ)
        crate::combat::effects::layer_check(w, id, heights);
    }
    let after = pos(w, id);
    (after.x, after.y) != (before.x, before.y)
}
