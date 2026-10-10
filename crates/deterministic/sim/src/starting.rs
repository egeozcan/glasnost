//! Starting units (S-0009 claims 45 and 46): each house's base unit and the
//! units bought with its budget, placed around its base cell while the map
//! loads, with their draws in the order claim 46 gives.

use glasnost_map::CellXY;

use crate::objects;
use crate::regions::STEPS;
use crate::rules::{Behaviour, Kind};
use crate::walk;
use crate::world::{GameSetup, HouseId, HouseRole, Layer, ObjectId, Pos, World};

/// Failed placements after which a house stops buying (S-0009 claim 45).
const FAILURE_LIMIT: u32 = 20; // spec: S-0009 Constants (starting-unit failure limit)

/// Purchases in a row that do not lower the budget after which our engine
/// stops a house's buying (S-0009 Edge cases: a limit of our engine).
const NON_FALLING_LIMIT: u32 = 1000; // spec: S-0009 Constants (purchases without a fall in the budget)

/// The largest radius of the placement search (S-0009 claim 46).
const LAST_RADIUS: i32 = 31; // spec: S-0009 Constants (placement search radii: first radius … 31)

/// The first radius of a budget unit's placement search (S-0009 claim 46).
const UNIT_RADIUS: i32 = 4; // spec: S-0009 Constants (first radius 4 for budget units)

/// The first radius of a base unit's search when its direct placement
/// fails (S-0009 claim 46).
const BASE_RADIUS: i32 = 1; // spec: S-0009 Constants (first radius 1 for a base unit)

/// The tech level of the neutral and special houses (S-0009 Terms).
const NON_PLAYER_TECH: i32 = 1; // spec: S-0009 Constants (tech level of the neutral and special houses)

/// The displacement direction threshold of the second pass (S-0009 claim
/// 46).
const TOWARDS_HIGHER: i32 = 50; // spec: S-0009 Constants (displacement direction threshold 50)

/// Whether a cell accepts a starting unit (S-0009 claim 46): it is usable
/// (the height-aware test) and holds no infantry, vehicle, aircraft or
/// building, or the unit is infantry and the cell's occupant nearest to
/// its centre is infantry. Interim (request Q-0045): the ground layer's
/// list is read on every cell, bridge cells included, and the nearest
/// occupant is the one with the smallest squared distance from its
/// position to the cell centre, the first listed on ties. With
/// `quirks.starting_infantry_share_centre` off an infantry unit also needs
/// a free standing position (S-0208 claim 3, centre class, with its draw);
/// the position found is returned.
fn accepts(w: &mut World, c: CellXY, infantry: bool) -> Option<u8> {
    if !w.cells.usable(c) {
        return None;
    }
    let centre = Pos::centre(c, 0);
    let mut nearest: Option<(i64, Kind)> = None;
    for &id in w.cells.list(c, Layer::Ground) {
        let Some(o) = w.obj(id) else { continue };
        if !matches!(
            o.kind,
            Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
        ) {
            continue;
        }
        let dx = i64::from(o.pos.x - centre.x);
        let dy = i64::from(o.pos.y - centre.y);
        let d = dx * dx + dy * dy;
        if nearest.is_none_or(|(b, _)| d < b) {
            nearest = Some((d, o.kind));
        }
    }
    // spec: S-0009 claim 46 (no occupant, or an infantry unit whose nearest occupant is infantry)
    let ok = match nearest {
        None => true,
        Some((_, k)) => infantry && k == Kind::Infantry,
    };
    if !ok {
        return None;
    }
    if !infantry || w.rules().quirks.starting_infantry_share_centre {
        // spec: S-0009 claim 46 (quirks.starting_infantry_share_centre: the centre position, no draw)
        return Some(crate::cells::position::CENTRE);
    }
    // spec: S-0009 Edge cases (switch off: the standing position of S-0208 claim 3 for class centre, with its draw)
    if crate::occupancy::no_position_by_flags(w, c, Layer::Ground) {
        return None;
    }
    let q = w.rules().quirks;
    let k = w.rng.ranged(0, 3, q.ranged_draw_half_span); // spec: S-0208 claim 3 (centre class: one ranged draw over 0 … 3)
    crate::occupancy::first_free(
        w,
        c,
        Layer::Ground,
        walk::row_for(crate::cells::position::CENTRE, k),
    )
}

/// The placement search of S-0009 claim 46 around `b` with first radius
/// `r0`, with its draws: the start element, then for each radius the draw
/// k = R(0, 7), the eight first-pass candidates and the eight second-pass
/// candidates with four draws each. Returns the accepting cell and the
/// standing position for an infantry unit.
fn search(w: &mut World, b: CellXY, r0: i32, infantry: bool) -> Option<(CellXY, u8)> {
    let q = w.rules().quirks;
    // spec: S-0009 claim 46 (the start element: b itself, no draw)
    if let Some(p) = accepts(w, b, infantry) {
        return Some((b, p));
    }
    let fp = *w.cells.footprint();
    // spec: S-0009 Formulas (clamp(v) = min(max(v, 1), W + H − 1))
    let clamp = |v: i32| v.max(1).min(fp.width + fp.height - 1);
    for r in r0..=LAST_RADIUS {
        // spec: S-0009 claim 46 (the radius start: one draw k = R(0, 7))
        let k = w.rng.ranged(0, 7, q.ranged_draw_half_span);
        let mut first = [CellXY::new(0, 0); 8]; // spec: S-0009 Constants (8 candidates per pass)
        for (j, f) in first.iter_mut().enumerate() {
            // spec: S-0009 claim 46 (b moved r cells along step direction (k + j) mod 8, clamped)
            let (sx, sy) = STEPS[((k as usize) + j) % 8];
            *f = CellXY::new(clamp(b.x + r * sx), clamp(b.y + r * sy));
        }
        for &c in &first {
            // spec: S-0009 claim 46 (a candidate equal to b other than the start element never takes the unit)
            if c != b
                && let Some(p) = accepts(w, c, infantry)
            {
                return Some((c, p));
            }
        }
        for &f in &first {
            // spec: S-0009 claim 46 (four draws: a_x = R(0, 1), p_x, a_y = R(0, 1), p_y)
            let ax = w.rng.ranged(0, 1, q.ranged_draw_half_span);
            let px = w.rng.percent();
            let ay = w.rng.ranged(0, 1, q.ranged_draw_half_span);
            let py = w.rng.percent();
            // spec: S-0009 Formulas (towards higher values when the percent draw is below 50)
            let x = clamp(if px < TOWARDS_HIGHER {
                f.x + ax
            } else {
                f.x - ax
            });
            let y = clamp(if py < TOWARDS_HIGHER {
                f.y + ay
            } else {
                f.y - ay
            });
            let c = CellXY::new(x, y);
            if c != b
                && let Some(p) = accepts(w, c, infantry)
            {
                return Some((c, p));
            }
        }
    }
    None
}

/// Puts a created starting unit on cell `c` at standing position `p`, with
/// body facing 0, at the terrain's height there, in the guard behaviour;
/// a vehicle of a visceroid type takes its draw R(0, 29) (S-0009 claim 46).
fn put(w: &mut World, id: ObjectId, c: CellXY, p: u8) {
    let Some(kind) = w.obj(id).map(|o| o.kind) else {
        return;
    };
    let (u, v) = if kind == Kind::Infantry {
        walk::standing_point(p)
    } else {
        // spec: S-0009 claim 46 (the centre of the cell)
        (128, 128)
    };
    let (x, y) = (256 * c.x + u, 256 * c.y + v); // spec: S-0204 claim 3 (a position within the cell)
    let pos = Pos::new(x, y, w.surface_at(x, y));
    let t = w.tick;
    if let Some(o) = w.obj_mut(id) {
        // spec: S-0009 claim 45 (our engine starts every house's starting units in guard)
        o.beh.force(Behaviour::Guard, t);
    }
    // spec: S-0009 claim 46 (body facing 0)
    objects::place(w, id, pos, 0);
    objects::visceroid_draw(w, id);
}

/// Gives every house its starting units (S-0009 claims 45 and 46). `bases`
/// holds each house's base cell. Draws in house order; after the last
/// house one ranged draw over 0 … 65535 whose value has no effect, unless
/// the step stops early.
pub fn give(w: &mut World, bases: &[Option<CellXY>], setup: &GameSetup) {
    let r = w.rules_arc();
    let q = r.quirks;
    let passive = |w: &World, h: usize| {
        r.countries
            .get(w.houses[h].country as usize)
            .is_none_or(|c| c.passive)
    };
    // spec: S-0009 claim 45 (the budget pool: types allowed to start, owned by the country of a house that is not passive, tech level at most the game's, base units left out)
    let live_countries: Vec<u32> = (0..w.houses.len())
        .filter(|&h| !passive(w, h))
        .map(|h| w.houses[h].country)
        .collect();
    let mut n: i32 = 0;
    let mut cost: i32 = 0;
    for (kind, list) in [(Kind::Vehicle, &r.vehicles), (Kind::Infantry, &r.infantry)] {
        for (i, u) in list.iter().enumerate() {
            let base = kind == Kind::Vehicle && r.base_units.contains(&(i as u32));
            if u.allowed_to_start
                && u.owners.iter().any(|c| live_countries.contains(c))
                && u.tech_level <= r.game_tech_level
                && !base
            {
                n += 1;
                cost = cost.wrapping_add(u.cost);
            }
        }
    }
    // spec: S-0009 Formulas (n taken as 1 when the pool is empty; B = trunc((trunc(n / 2) + C) / n) × U, wrapping)
    let n = n.max(1);
    let budget = (n / 2) // spec: S-0009 Formulas (trunc(n / 2))
        .wrapping_add(cost)
        .wrapping_div(n)
        .wrapping_mul(setup.unit_count);
    for h in 0..w.houses.len() {
        // spec: S-0009 claim 45 (each house in house order that is not passive receives starting units around its base cell)
        let Some(b) = bases.get(h).copied().flatten() else {
            continue;
        };
        if passive(w, h) {
            continue;
        }
        let owner = HouseId(h as u8);
        let country = w.houses[h].country;
        // spec: S-0009 Terms (a player's house has the game's tech level; the neutral and special houses 1)
        let tech = if w.houses[h].role == HouseRole::Player {
            r.game_tech_level
        } else {
            NON_PLAYER_TECH
        };
        if setup.bases {
            // spec: S-0009 claim 45 (the first type of the base-unit list whose owner list includes the house's country)
            let Some(&bt) = r
                .base_units
                .iter()
                .find(|&&t| r.vehicles[t as usize].owners.contains(&country))
            else {
                // spec: S-0009 claim 45 (early stop: no base-unit type qualifies)
                return;
            };
            let id = objects::create(w, Kind::Vehicle, bt, Some(owner));
            // spec: S-0009 claim 46 (the base unit goes directly to its base cell's centre; a base cell that is not legal: the search with first radius 1, a departure)
            let at = if w.cells.idx(b).is_some() {
                Some((b, crate::cells::position::CENTRE))
            } else {
                search(w, b, BASE_RADIUS, false)
            };
            let Some((c, p)) = at else {
                // spec: S-0009 claim 45 (early stop: the base unit cannot be placed; it is removed again)
                w.objects.remove(&id);
                return;
            };
            put(w, id, c, p);
        }
        // spec: S-0009 claim 45 (the start pools, in registration order: allowed to start, owned by the house's country, tech level at most the house's, base units left out)
        let pool = |kind: Kind| -> Vec<u32> {
            let list = if kind == Kind::Vehicle {
                &r.vehicles
            } else {
                &r.infantry
            };
            list.iter()
                .enumerate()
                .filter(|(i, u)| {
                    u.allowed_to_start
                        && u.owners.contains(&country)
                        && u.tech_level <= tech
                        && !(kind == Kind::Vehicle && r.base_units.contains(&(*i as u32)))
                })
                .map(|(i, _)| i as u32)
                .collect()
        };
        let vehicles = pool(Kind::Vehicle);
        let infantry = pool(Kind::Infantry);
        if budget > 0 && vehicles.is_empty() && infantry.is_empty() {
            // spec: S-0009 claim 45 (early stop: B > 0 and both pools empty)
            return;
        }
        let mut left = budget;
        let mut failures = 0;
        let mut flat_run = 0;
        // spec: S-0009 claim 45 (buy while R > 0 and fewer than 20 placements have failed)
        while left > 0 && failures < FAILURE_LIMIT && flat_run < NON_FALLING_LIMIT {
            // spec: S-0009 claim 45 (the vehicle pool while R > trunc(B / 3) and it is not empty; the infantry pool otherwise)
            let use_vehicles = (left > budget / 3 && !vehicles.is_empty()) || infantry.is_empty(); // spec: S-0009 Edge cases (an empty infantry pool when infantry is due: the vehicle pool, a departure)
            let (kind, list) = if use_vehicles {
                (Kind::Vehicle, &vehicles)
            } else {
                (Kind::Infantry, &infantry)
            };
            // spec: S-0009 claim 45 (the type: one ranged draw R(0, pool size − 1))
            let i = w
                .rng
                .ranged(0, list.len() as i32 - 1, q.ranged_draw_half_span);
            let t = list[i as usize];
            let id = objects::create(w, kind, t, Some(owner));
            // spec: S-0009 claim 46 (every other starting unit: the placement search with first radius 4)
            match search(w, b, UNIT_RADIUS, kind == Kind::Infantry) {
                Some((c, p)) => {
                    put(w, id, c, p);
                    let c = r.unit(kind, t).map_or(0, |u| u.cost);
                    // spec: S-0009 Formulas (R falls by the type's cost, wrapping)
                    let next = left.wrapping_sub(c);
                    flat_run = if next < left { 0 } else { flat_run + 1 };
                    left = next;
                }
                None => {
                    // spec: S-0009 claim 45 (a failed placement: the unit is removed, its creation draw stays taken)
                    w.objects.remove(&id);
                    failures += 1;
                }
            }
        }
        if left > 0 {
            // spec: S-0009 claim 45 (a positive budget left is given to the house as credits)
            w.houses[h].credits = w.houses[h].credits.wrapping_add(left);
        }
    }
    // spec: S-0009 claim 20 (after the last house one ranged draw over 0 … 65535 whose value has no effect)
    let _ = w.rng.ranged(0, 65535, q.ranged_draw_half_span);
}
