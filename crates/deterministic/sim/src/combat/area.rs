//! Area damage (S-0303): the spread radius and scanned cells, the order of
//! the hits, the exclusions, wall hits, resource cells, exploding overlays
//! and the result of a detonation.

use glasnost_map::{CellXY, NO_OVERLAY};
use glasnost_num::F64;
use glasnost_num::geom::root_floor;

use crate::combat::damage::{self, Hit};
use crate::combat::data::{WarheadId, WarheadRules};
use crate::combat::effects::Effect;
use crate::rules::Kind;
use crate::world::{HouseId, Layer, ObjectId, Pos, World};

/// The result of a detonation (S-0303 claim 23).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetonationResult {
    /// At least one object received a hit.
    Hit,
    /// None did, or claim 2 applied.
    Missed,
    /// Claim 15 nullified it.
    Nullified,
}

/// One detonation (S-0303 Terms).
#[derive(Clone, Copy, Debug)]
pub struct Detonation {
    /// Detonation point P.
    pub point: Pos,
    /// Damage amount D.
    pub damage: i32,
    /// Warhead W.
    pub warhead: Option<WarheadId>,
    /// Source object.
    pub source: Option<ObjectId>,
    /// Source house.
    pub house: Option<HouseId>,
    /// The producer allows resource reduction (claim 17).
    pub resources: bool,
}

/// The ring metric of a cell displacement (S-0303 Terms).
pub fn ring_metric(dx: i32, dy: i32) -> i32 {
    let (a, b) = (dx.abs(), dy.abs());
    // spec: S-0303 Terms (m = max(|Δx|, |Δy|) + ⌊min(|Δx|, |Δy|) / 2⌋)
    a.max(b) + a.min(b) / 2
}

/// The ring-11 list of table T1 (S-0303, `quirks.spread_ring_eleven`).
// spec: S-0303 table T1 (the 60 entries of ring 11)
const RING_ELEVEN: [(i32, i32); 60] = [
    (0, 11),
    (0, -11),
    (-1, 11),
    (1, 11),
    (-1, -11),
    (1, -11),
    (-2, 11),
    (2, 11),
    (-2, -11),
    (2, -11),
    (-3, 11),
    (3, 11),
    (-3, -11),
    (-3, 11),
    (-4, 9),
    (4, 9),
    (-4, -9),
    (4, -9),
    (-5, 9),
    (5, 9),
    (-5, -9),
    (5, -9),
    (-6, 8),
    (6, 8),
    (-6, -8),
    (6, -8),
    (-7, 8),
    (7, 8),
    (-7, -8),
    (7, -8),
    (-8, 7),
    (8, 7),
    (-8, -7),
    (8, -7),
    (-8, 6),
    (8, 6),
    (-8, -6),
    (8, -6),
    (-9, 5),
    (9, 5),
    (-9, -5),
    (9, -5),
    (-9, 4),
    (9, 4),
    (-9, -4),
    (9, -4),
    (-10, 3),
    (10, 3),
    (-10, -3),
    (10, -3),
    (-10, 2),
    (10, 2),
    (-10, -2),
    (10, -2),
    (-11, 1),
    (11, 1),
    (-11, -1),
    (11, -1),
    (11, 0),
    (-11, 0),
];

/// The cells of ring r in scan order (S-0303 table T1).
pub fn ring(r: i32, ring_eleven_quirk: bool) -> Vec<(i32, i32)> {
    match r {
        0 => vec![(0, 0)],
        // spec: S-0303 table T1 (ring 1: only its first row in decreasing Δx)
        1 => vec![
            (1, -1),
            (0, -1),
            (-1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ],
        // spec: S-0303 table T1 (ring 11: the original's list)
        11 if ring_eleven_quirk => RING_ELEVEN.to_vec(),
        _ => {
            // spec: S-0303 table T1 (rings 2 to 10: increasing Δy, then increasing Δx)
            let mut out = Vec::new();
            for dy in -r..=r {
                for dx in -r..=r {
                    if ring_metric(dx, dy) == r {
                        out.push((dx, dy));
                    }
                }
            }
            out
        }
    }
}

/// The double nearest 0.99 (S-0303 F2).
fn c99() -> F64 {
    // spec: S-0303 F2 (c = the double nearest 0.99)
    crate::rules::nearest_double("0.99")
}

/// The spread radius R and scan radius k of a warhead (S-0303 F1, F2).
pub fn radii(w: &World, warhead: WarheadId) -> (i32, i32) {
    let r = w.rules();
    let mode = r.mode;
    let s = r.combat.warhead(warhead).spread;
    // spec: S-0303 F1 (R = trunc(256 × S), exact)
    let big_r = s.mul(F64::from_i64(256, mode), mode).to_i32_common();
    // spec: S-0303 F2 (k = trunc(RZ₅₃(S + c))); Edge cases (clamped to 0 … 11, a departure)
    let k = s.add(c99(), mode).to_i32_common().clamp(0, 11);
    (big_r, k)
}

/// ⌊r(n)⌋ of a squared distance.
fn root(w: &World, n: i64) -> i32 {
    let r = w.rules();
    root_floor(n as u64, r.quirks.table_root_distance, r.mode) as i32
}

/// The epicentre distance after claim 10 (S-0303): an aircraft in the air
/// (S-0308's test) counts at half its distance, truncated.
fn half_in_air(w: &World, id: ObjectId, d: i32) -> i32 {
    let aircraft = w.obj(id).is_some_and(|o| o.kind == Kind::Aircraft);
    if aircraft && crate::combat::weapon::in_air(w, id) {
        // spec: S-0303 claim 10 (the distance halved, truncated toward zero)
        d / 2
    } else {
        d
    }
}

/// The spatial distance of S-0303 claim 7 from P to a point.
fn dist(w: &World, p: Pos, q: Pos) -> i32 {
    let (dx, dy, dz) = (
        i64::from(p.x - q.x),
        i64::from(p.y - q.y),
        i64::from(p.z - q.z),
    );
    root(w, dx * dx + dy * dy + dz * dz)
}

/// The distance S-0303 claims 7 and 10 give object `id` from a detonation
/// at `p` (for a building, its nearest foundation cell): an estimate that
/// changes nothing.
pub fn object_distance(w: &World, p: Pos, id: ObjectId) -> i32 {
    object_distances(w, p, id)
        .into_iter()
        .min()
        .unwrap_or(i32::MAX)
}

/// The distance of each hit a detonation at `p` would give object `id`,
/// before the spread radius test: one for a unit (S-0303 claims 7 and
/// 10), one per foundation cell for a building (claims 7 and 8;
/// `quirks.area_damage_per_foundation_cell` off: one, at the smallest
/// distance). An estimate that changes nothing.
pub fn object_distances(w: &World, p: Pos, id: ObjectId) -> Vec<i32> {
    let Some(o) = w.obj(id) else {
        return Vec::new();
    };
    if o.kind == Kind::Building {
        let centre = p.cell();
        let all: Vec<i32> = crate::objects::foundation_cells(o.pos.cell(), o.foundation)
            .into_iter()
            .map(|c| building_cell_distance(w, p, c, centre))
            .collect();
        if w.rules().quirks.area_damage_per_foundation_cell {
            return all;
        }
        // spec: S-0303 Edge cases (area_damage_per_foundation_cell off: one hit at the smallest distance)
        return all.into_iter().min().into_iter().collect();
    }
    // spec: S-0303 claim 7 (any other object: from P to its position) and claim 10 (an aircraft in the air at half)
    vec![half_in_air(w, id, dist(w, p, w.reference_point(id)))]
}

/// The distance S-0303 claim 7 gives a building's foundation cell `c`
/// from a detonation at `p` whose centre cell is `centre`.
// spec: S-0303 claim 7 (a building: from P to the centre of this foundation cell at the surface height there; 0 in the centre cell unless P is more than 208 above)
fn building_cell_distance(w: &World, p: Pos, c: CellXY, centre: CellXY) -> i32 {
    let cc = Pos::centre(c, 0);
    let z = w.surface_at(cc.x, cc.y);
    let cp = Pos::new(cc.x, cc.y, z);
    if c == centre {
        if p.z - z > 208 {
            dist(w, p, cp) - 208
        } else {
            0
        }
    } else {
        dist(w, p, cp)
    }
}

/// A wall hit (S-0303 claim 16, F4) on the wall of cell `c`.
pub fn wall_hit(w: &mut World, c: CellXY, amount: i32, absolute: bool) {
    let rules = w.rules_arc();
    let Some(cell) = w.cells.cell(c).cloned() else {
        return;
    };
    if cell.overlay == NO_OVERLAY {
        return;
    }
    let Some(ov) = rules.overlays.get(usize::from(cell.overlay)).cloned() else {
        return;
    };
    if !ov.wall {
        return;
    }
    // spec: S-0303 F4 (a = −1 makes the hit absolute, quirks.wall_hit_minus_one_is_absolute)
    let absolute = absolute || (amount == -1 && rules.quirks.wall_hit_minus_one_is_absolute);
    let t = ov.strength;
    let success = if absolute {
        true
    } else if amount >= t {
        // spec: S-0303 F4 (a ≥ T succeeds with no draw)
        true
    } else {
        // spec: S-0303 F4 (a < T: one ranged draw over 0 … T, success when a > r; a negative amount still draws)
        let r = w.rng.ranged(0, t, rules.quirks.ranged_draw_half_span);
        amount > r
    };
    if !success {
        return;
    }
    wall_success(w, c, absolute);
}

/// A successful wall hit (S-0303 F4: the stage rises; neighbours; removal).
// spec: S-0303 F4 (neighbours when the new stage is L − 1 and L > 2)
pub fn wall_success(w: &mut World, c: CellXY, absolute: bool) {
    let rules = w.rules_arc();
    let Some(cell) = w.cells.cell(c).cloned() else {
        return;
    };
    let Some(ov) = rules.overlays.get(usize::from(cell.overlay)).cloned() else {
        return;
    };
    // spec: S-0303 F4 (success: v increases by 16, so s increases by 1)
    let v = i32::from(cell.overlay_data) + 16;
    let s = v / 16; // spec: S-0210 claim 8 (s = ⌊v / 16⌋)
    let l = ov.damage_levels;
    if let Some(g) = w.cells.grid.get_mut(c) {
        g.overlay_data = v.clamp(0, 255) as u8; // nocite: the overlay data value is one byte
    }
    if s == l - 1 && l > 2 {
        // spec: S-0303 F4 (the new stage is L − 1 and L > 2: neighbours of the same type at stage 0 take a wall hit of 200, north, east, south, west)
        for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            let n = CellXY::new(c.x + dx, c.y + dy);
            let same = w
                .cells
                .cell(n)
                .is_some_and(|nc| nc.overlay == cell.overlay && nc.overlay_data / 16 == 0);
            if same {
                wall_hit(w, n, 200, false); // spec: S-0303 Constants (neighbour wall hit 200)
            }
        }
    }
    // spec: S-0303 F4 (removed when absolute, when s ≥ L, or when s = L − 1 with no link, v mod 16 = 0)
    if absolute || s >= l || (s == l - 1 && v % 16 == 0) {
        crate::crush::remove_overlay(w, c);
    }
}

/// One candidate hit: the object, its distance (claim 7) and its cell.
struct Candidate {
    id: ObjectId,
    d: i32,
}

/// The curtain-protection trigger of S-0303 claim 15: curtain kind (not
/// the force shield's) and an epicentre distance before claim 10's halving
/// below the trigger distance.
// spec: S-0303 claim 15 and S-0513 Terms (curtain protection: the iron-curtain kind only; epicentre distance below 85 leptons, before the halving of claim 10)
fn curtain_trigger(w: &World, id: ObjectId, raw: i32) -> bool {
    raw < 85
        && crate::superweapons::protection_kind(w, id)
            == Some(crate::superweapons::ProtectKind::Curtain)
}

/// Whether an object of this kind and type is left out for harvester
/// immunity: a vehicle of a game-wide harvester type on a map that sets the
/// flag.
// spec: S-0303 claim 9 (a vehicle whose type is in the game-wide harvester list while the map's harvester-immunity flag is set)
pub fn harvester_immune(w: &World, kind: Kind, type_index: u32) -> bool {
    w.map_flags.harvester_immune
        && kind == Kind::Vehicle
        && w.rules()
            .combat
            .general
            .harvester_units
            .contains(&type_index)
}

/// Applies one detonation's area damage (S-0303 claims 1 to 20).
// spec: S-0303 claim 7 and Constants (raised-detonation building allowance 208 leptons)
pub fn detonate(w: &mut World, det: Detonation) -> DetonationResult {
    let rules = w.rules_arc();
    let q = rules.quirks;
    // spec: S-0303 claim 2 (a damage amount of 0 or no warhead: no effect at all, missed; and every detonation of an inert map is like this)
    let Some(warhead) = det.warhead else {
        return DetonationResult::Missed;
    };
    if det.damage == 0 || w.map_flags.inert {
        return DetonationResult::Missed;
    }
    let h = rules.combat.warhead(warhead).clone();
    let p = det.point;
    let (big_r, k) = radii(w, warhead);
    w.effects.push(Effect::Detonation {
        pos: p,
        radius: big_r,
        damage: det.damage,
    });
    let centre = p.cell();
    let terrain_at_p = w.surface_at(p.x, p.y);
    // spec: S-0303 claim 4 (a high bridge in the centre cell and P more than 208 above the terrain: the deck layer)
    let layer = if w.high_bridge(centre) && p.z - terrain_at_p > 208 {
        Layer::Deck
    } else {
        Layer::Ground
    };
    let mut cells = Vec::new();
    for r in 0..=k {
        for (dx, dy) in ring(r, q.spread_ring_eleven) {
            cells.push(CellXY::new(centre.x + dx, centre.y + dy));
        }
    }
    // spec: S-0303 State and ordering (every scanned cell has its resource reduction and wall hit before the first hit)
    for &c in &cells {
        if w.cells.cell(c).is_none() {
            continue;
        }
        let ov = w
            .cells
            .cell(c)
            .map(|cc| cc.overlay)
            .filter(|&o| o != NO_OVERLAY)
            .and_then(|o| rules.overlays.get(usize::from(o)).cloned());
        if let Some(ov) = &ov
            // spec: S-0303 claim 17 (a resource overlay whose type has the chain-reaction flag, a warhead with the resource flag, a producer that allows it)
            && ov.tiberium
            && ov.chain_reaction
            && h.tiberium
            && det.resources
            && det.damage / 10 > 0
        {
            // spec: S-0303 claim 17 (resource reduction by trunc(D / 10)); spec: S-0401 claim 16 (a) (taken by claim 3's rule)
            crate::econ::resource::take_bales(w, c, det.damage / 10);
        }
        if let Some(ov) = &ov
            && ov.wall
        {
            // spec: S-0303 claim 16 (the wall flag, the wood flag on a wood wall, or the absolute flag)
            let wooden = ov.armour == glasnost_rules::sim::ArmourClass::Wood.index();
            if h.wall || (h.wood && wooden) || h.wall_absolute {
                wall_hit(w, c, det.damage, h.wall_absolute);
            }
        }
    }
    let mut cands: Vec<Candidate> = Vec::new();
    // spec: S-0303 Terms and Constants (a small detonation: the warhead's spread at most 0.5 cells, inclusive)
    let small = h
        .spread
        .add(h.spread, rules.mode)
        .le(F64::from_i64(1, rules.mode));
    let mut nullified = false;
    // spec: S-0303 claim 6 (a detonation above the terrain at the centre of the centre cell also considers the tracked flying objects of the F7 search, before the scanned cells; quirks.ground_detonation_misses_aircraft off: every detonation)
    let cc = Pos::centre(centre, 0);
    if p.z > w.surface_at(cc.x, cc.y) || !q.ground_detonation_misses_aircraft {
        let spread = h.spread.to_i32_common();
        for id in crate::combat::flying::candidates(w, centre, spread) {
            if w.obj(id).is_none() {
                continue;
            }
            if !q.flying_candidates_unfiltered && Some(id) == det.source {
                // spec: S-0303 Edge cases (quirks.flying_candidates_unfiltered off: the source-object exclusion applies)
                let self_damage =
                    crate::combat::veterancy::type_combat(w, id).is_some_and(|tc| tc.damage_self);
                if !self_damage && rules.combat.general.crush_warhead != Some(warhead) {
                    continue;
                }
            }
            // spec: S-0303 Edge cases (quirks.flying_candidates_unfiltered off: the listed-harvester exclusion applies, claim 9)
            if !q.flying_candidates_unfiltered
                && w.obj(id)
                    .is_some_and(|o| harvester_immune(w, o.kind, o.type_index))
            {
                continue;
            }
            let raw = dist(w, p, w.reference_point(id));
            let d = half_in_air(w, id, raw);
            if d > big_r {
                continue;
            }
            // spec: S-0303 claim 15 (a tracked flying candidate within R, curtain-protected, raw distance below 85 leptons)
            if small && curtain_trigger(w, id, raw) {
                nullified = true;
            }
            cands.push(Candidate { id, d });
        }
    }
    // The objects that took a flying entry above.
    let flying_found: std::collections::BTreeSet<ObjectId> = cands.iter().map(|c| c.id).collect();
    let mut building_first: std::collections::BTreeMap<ObjectId, usize> = Default::default();
    for &c in &cells {
        if w.cells.cell(c).is_none() {
            continue;
        }
        let list: Vec<ObjectId> = w.cells.list(c, layer).to_vec();
        for id in list {
            let Some(o) = w.obj(id) else { continue };
            // spec: S-0303 claim 6 and Edge cases (quirks.flying_listed_double_hit on: an object found by both searches keeps both entries, unmerged; off: only its flying entry)
            if !q.flying_listed_double_hit && flying_found.contains(&id) {
                continue;
            }
            // spec: S-0303 claim 9 (the source object, unless its type has the self-damage flag or the warhead is the crushing warhead)
            if Some(id) == det.source {
                let self_damage =
                    crate::combat::veterancy::type_combat(w, id).is_some_and(|tc| tc.damage_self);
                if !self_damage && rules.combat.general.crush_warhead != Some(warhead) {
                    continue;
                }
            }
            // spec: S-0303 claim 9 (a building whose type is invisible in game)
            if o.kind == Kind::Building && rules.buildings[o.type_index as usize].invisible {
                continue;
            }
            // spec: S-0303 claim 9 (a vehicle whose type is in the harvester list while the map's harvester-immunity flag is set)
            if harvester_immune(w, o.kind, o.type_index) {
                continue;
            }
            let d = if o.kind == Kind::Building {
                // spec: S-0303 claim 7 (a building: from P to the centre of this foundation cell at the surface height there; 0 in the centre cell unless P is more than 208 above)
                building_cell_distance(w, p, c, centre)
            } else {
                // spec: S-0303 claim 7 (any other object: from P to its position)
                dist(w, p, w.reference_point(id))
            };
            let raw = d;
            let d = half_in_air(w, id, d);
            // spec: S-0303 claim 15 (an object of any kind in the centre cell, alive, raw distance below 85 leptons; R plays no part)
            if small && c == centre && curtain_trigger(w, id, raw) {
                // spec: S-0303 claim 9 and S-0504 claim 10 (alive: a sinking vessel that keeps health is alive; #172)
                let alive = w.obj(id).is_some_and(|o| {
                    o.health > 0 && o.on_map && !crate::combat::damage::dead(o)
                });
                if alive {
                    nullified = true;
                }
            }
            if d > big_r {
                continue;
            }
            if o.kind == Kind::Building && !q.area_damage_per_foundation_cell {
                // spec: S-0303 Edge cases (area_damage_per_foundation_cell off: one hit at the smallest distance, at its first place)
                if let Some(&i) = building_first.get(&id) {
                    cands[i].d = cands[i].d.min(d);
                    continue;
                }
                building_first.insert(id, cands.len());
            }
            cands.push(Candidate { id, d });
        }
    }
    // spec: S-0303 claim 15 (quirks.curtain_nullifies_small_detonation off: a curtain-protected object only cancels its own hits)
    let nullified = nullified && q.curtain_nullifies_small_detonation;
    let mut any = false;
    for cand in cands {
        if nullified && !crate::superweapons::protected(w, cand.id) {
            // spec: S-0303 claim 15 (a nullified detonation hits only objects under invulnerability, of either kind; all others take nothing)
            continue;
        }
        // spec: S-0303 claim 9 (not alive, health 0 or less, not on the map: no hit, judged at the moment of the hit)
        let Some(o) = w.obj(cand.id) else { continue };
        // spec: S-0302 claim 16 (a building in a delayed-kill countdown keeps 1 health and takes the later hits of the detonation); S-0504 claim 10 and S-0302 claim 8 (a sinking vessel with health left takes applicable area damage; #172)
        let dead = crate::combat::damage::dead(o);
        if o.health <= 0 || !o.on_map || dead {
            continue;
        }
        any = true;
        let hit = Hit {
            amount: det.damage,
            warhead: Some(warhead),
            distance: cand.d,
            attacker: det.source,
            house: det.house,
            ignore_defences: false,
            no_escape: false,
        };
        // spec: S-0303 claim 11 (each hit is resolved completely before the next)
        damage::apply(w, cand.id, hit);
    }
    if nullified {
        // spec: S-0303 claim 15 (a nullified detonation skips claims 18 to 20)
        return DetonationResult::Nullified;
    }
    if h.rocker {
        // spec: S-0303 claim 20 and Observable order (the warhead's rocking takes no draw and is reported to the drawing; it comes after all object hits with their nested detonations, and before the bridge-test draws, the centre-cell exploding overlay and the particle system)
        rock(w, &det, &h, layer);
    }
    // spec: S-0303 claim 18 and S-0511 claim 1 (with the game's bridge-destruction setting on, a warhead that damages walls makes destroy attempts on the bridge in the centre cell)
    if crate::econ::setup::bridge_destruction(w) && h.wall {
        crate::bridges::area_attempts(w, centre, p.z, det.damage, warhead);
    }
    // spec: S-0303 claim 19 (an exploding overlay in the centre cell)
    exploding_overlay(w, centre, &det);
    if let Some(name) = h.particle.clone() {
        // spec: S-0303 claim 20 and S-0613 claim 4 (a warhead that names a particle system creates it at P at the end of the detonation, owned by the source house, with its first particle)
        crate::gas::warhead_system(w, &name, p, det.house);
    }
    if any {
        DetonationResult::Hit
    } else {
        DetonationResult::Missed
    }
}

/// The least damage amount whose rocking strength min(*D* × 0.01, 4.0)
/// exceeds 0.3 (S-0303 claim 20).
// spec: S-0303 claim 20 (whatever the rounding, the strength exceeds 0.3 exactly when D ≥ 31)
const ROCKING_LEAST_DAMAGE: i32 = 31;
/// The rocking area's reach from the centre cell, cells: 7 × 7 cells.
// spec: S-0303 claim 20 and Constants (rocking area 7 × 7 cells around the centre cell)
const ROCKING_REACH: i32 = 3;

/// The rocking of a warhead with the rocking flag (S-0303 claim 20), as a
/// presentation event: the objects of `layer` in the 7 × 7 cells around
/// the centre cell, those of the centre cell pushed away from the source
/// object when there is one; the others, and the centre cell's objects
/// when there is no source object, away from *P* and only when the spread
/// is above 0. Reads only; takes no draw.
fn rock(w: &mut World, det: &Detonation, h: &WarheadRules, layer: Layer) {
    // spec: S-0303 claim 20 (only when the rocking strength exceeds 0.3)
    if det.damage < ROCKING_LEAST_DAMAGE {
        return;
    }
    let centre = det.point.cell();
    let from_source = det.source.and_then(|s| w.obj(s)).map(|o| o.pos);
    // spec: S-0303 claim 20 (the cells other than the centre cell only when S > 0)
    let spread = h.spread.gt(F64::ZERO);
    let mut seen = std::collections::BTreeSet::new();
    let mut pushed = Vec::new();
    for dy in -ROCKING_REACH..=ROCKING_REACH {
        for dx in -ROCKING_REACH..=ROCKING_REACH {
            let in_centre = dx == 0 && dy == 0;
            // spec: S-0303 claim 20 (pushed from the source object: the centre cell's objects when there is one, whatever S)
            let from_object = in_centre && from_source.is_some();
            // spec: S-0303 claim 20 (every other object, the centre cell's included when there is no source object, only when S > 0)
            if !from_object && !spread {
                continue;
            }
            let c = CellXY::new(centre.x + dx, centre.y + dy);
            if w.cells.cell(c).is_none() {
                continue;
            }
            // spec: S-0303 claim 20 (the objects in the same cell layer)
            for &id in w.cells.list(c, layer) {
                if !seen.insert(id) {
                    continue;
                }
                // spec: S-0303 claim 20 (the centre cell's objects away from the source object when there is one, the others away from P)
                // design: Q-2081 question 5 (a source that has left play by the time of the rocking counts as no source object; S-0303 r31 does not separate the two)
                let from = match from_source {
                    Some(q) if in_centre => q,
                    _ => det.point,
                };
                pushed.push((id, from));
            }
        }
    }
    if !pushed.is_empty() {
        w.effects.push(Effect::Rock {
            damage: det.damage,
            pushed,
        });
    }
}

/// An exploding overlay in the centre cell (S-0303 claim 19).
// spec: S-0303 Constants (barrel debris chance 15 percent per entry)
fn exploding_overlay(w: &mut World, centre: CellXY, det: &Detonation) {
    let rules = w.rules_arc();
    let Some(cell) = w.cells.cell(centre).cloned() else {
        return;
    };
    if cell.overlay == NO_OVERLAY {
        return;
    }
    if !rules
        .overlays
        .get(usize::from(cell.overlay))
        .is_some_and(|o| o.explodes)
    {
        return;
    }
    let g = &rules.combat.general;
    // spec: S-0303 claim 19 (the overlay is removed, the barrel explosion appears at P)
    crate::crush::remove_overlay(w, centre);
    if let Some(a) = g.barrel_explode.clone() {
        crate::combat::effects::anim(w, &a, det.point);
    }
    // spec: S-0303 claim 19 (a nested detonation with the ammunition-crate damage, the demolition warhead, no source object, the outer house)
    // spec: S-0303 claim 24 (the default ammunition-crate damage never disables the nested detonation)
    detonate(
        w,
        Detonation {
            point: det.point,
            damage: g.ammo_crate_damage,
            warhead: g.c4_warhead,
            source: None,
            house: det.house,
            resources: true,
        },
    );
    // spec: S-0303 claim 19 (percent draws r₀, r₁, …: the first entry below 15 appears; then one percent draw for the particle system, below 25)
    let debris = g.barrel_debris.clone();
    for name in &debris {
        if w.rng.percent() < 15 {
            // spec: S-0317 claim 11 (claim 19's barrel debris is a piece of S-0317) and claim 1 (a detonation's pieces have no house)
            // spec: S-0303 claim 19 (revision 25: the piece is created at P with the creation draws of S-0317 claim 3, or claim 4 for a meteor type, right after the draw rᵢ* that chose it and before the particle system's percent draw r)
            crate::pieces::create_named(w, name, det.point, None);
            break;
        }
    }
    if w.rng.percent() < 25 {
        // spec: S-0303 Constants (barrel particle chance 25 percent) and claim 19 (the particle system appears at P; S-0616 claim 1)
        crate::emitters::barrel_particle_system(w, det.point);
    }
}
