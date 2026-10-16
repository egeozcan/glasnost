//! Radiation fields (S-0525): the field a radiation-weapon detonation
//! leaves, the radiation it spreads over the cells around its centre, its
//! decay and end (row 13 of the tick, S-0203), the hit it deals to mobile
//! units standing in it (in the unit's own turn), and the light it casts.
//!
//! Cell radiation is a running sum of doubles (S-0525 F1 to F3, F5): every
//! floating-point operation is formed in the order the formulas name, with
//! the engine's soft double and its rounding mode (S-0205). No site takes a
//! scenario random draw (S-0525 claim 12).

use std::collections::BTreeMap;

use glasnost_map::{CellXY, Theater};
use glasnost_num::geom::root_floor;
use glasnost_num::{F64, Rounding};
use serde::{Deserialize, Serialize};

use crate::combat::damage::{self, Hit};
use crate::combat::data::WeaponId;
use crate::combat::veterancy::type_combat;
use crate::rules::Kind;
use crate::world::{ObjectId, World};

/// The light source of a field (S-0525 claims 13 to 15, F7).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldLight {
    /// The current intensity, ‰.
    pub intensity: i32,
    /// The intensity drop of one light step, Δ*I* (F2).
    pub step: i32,
    /// The tint at the field's making or latest top-up, ‰ per channel.
    pub tint_start: [i32; 3],
    /// The current tint, ‰ per channel.
    pub tint: [i32; 3],
}

/// One radiation field (S-0525 Terms).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Field {
    /// The centre cell.
    pub centre: CellXY,
    /// The spread *s*, whole cells (claim 1); negative bad data reaches no
    /// cell.
    pub spread: i32,
    /// The level *L* set at the making or the latest top-up.
    pub level: i32,
    /// The duration *N*, ticks.
    pub duration: i32,
    /// The time left *T*, ticks.
    pub left: i32,
    /// The tick the level-decay timer last started.
    pub level_start: i32,
    /// The tick the light-fading timer last started.
    pub light_start: i32,
    /// The level steps *n*<sub>L</sub> (F2).
    pub level_steps: i32,
    /// The decay steps taken since the making or the latest top-up.
    pub steps_taken: i32,
    /// The reach *R*, leptons (F1), fixed at the making: a top-up keeps the
    /// spread and the light's radius (claim 4).
    pub reach: i32,
    /// The light.
    pub light: FieldLight,
}

/// The light of a field as the map view draws it (claim 13): the centre
/// point's cell, the radius in leptons, the intensity and the tint, in ‰.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightSource {
    /// The centre cell.
    pub centre: CellXY,
    /// The light radius (leptons); a field of negative spread lights no
    /// cell (Edge cases).
    pub radius: i32,
    /// The intensity, ‰.
    pub intensity: i32,
    /// The tint, ‰ per channel.
    pub tint: [i32; 3],
}

/// All radiation of a game.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RadWorld {
    /// The fields in creation order (row 13 updates the most recently
    /// created first, S-0203).
    pub fields: Vec<Field>,
    /// The cell radiation *C*; a cell not listed holds 0.
    pub cells: BTreeMap<CellXY, F64>,
}

fn big(n: i64, mode: Rounding) -> F64 {
    F64::from_i64(n, mode)
}

fn zero() -> F64 {
    F64::ZERO
}

/// Stores a cell's radiation, dropping the entry at 0.
fn store(cells: &mut BTreeMap<CellXY, F64>, c: CellXY, v: F64) {
    if v.eq_num(zero()) {
        cells.remove(&c);
    } else {
        cells.insert(c, v);
    }
}

/// *C* ← *C* + *c* (F1).
fn add_to(cells: &mut BTreeMap<CellXY, F64>, c: CellXY, amount: F64, mode: Rounding) {
    if amount.eq_num(zero()) {
        return;
    }
    let cur = cells.get(&c).copied().unwrap_or(zero());
    store(cells, c, cur.add(amount, mode));
}

/// *C* ← max(0, *C* − *a*) (F3, F5).
fn take_from(cells: &mut BTreeMap<CellXY, F64>, c: CellXY, amount: F64, mode: Rounding) {
    let Some(cur) = cells.get(&c).copied() else {
        return;
    };
    let v = cur.sub(amount, mode);
    // spec: S-0525 claim 5 (a cell's radiation that would fall below 0 becomes 0)
    store(cells, c, if v.lt(zero()) { zero() } else { v });
}

/// Whether a cell is a map cell (Edge cases of F1: other cells take and
/// keep no radiation).
fn map_cell(w: &World, c: CellXY) -> bool {
    w.cells.idx(c).is_some()
}

/// The cells of a field's square (F1: |x − x₀| ≤ s and |y − y₀| ≤ s), kept
/// to the cells the map has.
fn square(w: &World, centre: CellXY, s: i32) -> Vec<CellXY> {
    if s < 0 {
        return Vec::new();
    }
    let fp = w.cells.footprint();
    let lim = fp.width.saturating_add(fp.height).saturating_add(2);
    let lo = |v: i32| v.saturating_sub(s).max(0);
    let hi = |v: i32| v.saturating_add(s).min(lim);
    let mut out = Vec::new();
    for x in lo(centre.x)..=hi(centre.x) {
        for y in lo(centre.y)..=hi(centre.y) {
            let c = CellXY::new(x, y);
            if map_cell(w, c) {
                out.push(c);
            }
        }
    }
    out
}

/// The ground height at the centre point of a cell (F1).
fn ground(w: &World, c: CellXY) -> i32 {
    // spec: S-0525 F1 (h, the ground height at the cell's centre point, S-0204)
    w.surface_at(256 * c.x + 128, 256 * c.y + 128)
}

/// The contribution of F1 of a field (centre, reach *R*, level *L*) to the
/// cell `c`.
fn contribution(w: &World, centre: CellXY, reach: i32, level: i32, c: CellXY) -> F64 {
    let r = w.rules();
    let mode = r.mode;
    // spec: S-0525 F1 (n = |P − P₀|², P the centre points with their ground heights)
    let dx = i64::from(c.x) - i64::from(centre.x);
    let dy = i64::from(c.y) - i64::from(centre.y);
    let dz = i64::from(ground(w, c)) - i64::from(ground(w, centre));
    let n = (dx * 256).pow(2) + (dy * 256).pow(2) + dz.pow(2);
    // spec: S-0525 F1 (d = ⌊√ₜ(n)⌋, the table square root of S-0204 F1)
    let d = root_floor(n as u64, r.quirks.table_root_distance, mode) as i64;
    let big_r = i64::from(reach);
    // spec: S-0525 F1 (c = ((R − d)/R) × L when d ≤ R, else 0; the quotient first, then the product)
    if d > big_r || big_r == 0 {
        return zero();
    }
    big(big_r - d, mode)
        .div(big(big_r, mode), mode)
        .mul(big(i64::from(level), mode), mode)
}

/// The field level of F4's integer form (claim 7): ⌊*L*·*T*/*N*⌋, 0 for
/// *N* < 1.
pub fn field_level(f: &Field) -> i32 {
    // spec: S-0525 claim 7 and F4 (trunc(T × L / N) in 32-bit integers; 0 when N < 1)
    if f.duration < 1 {
        return 0;
    }
    f.left.wrapping_mul(f.level) / f.duration
}

/// The field level of the field centred on a cell, if there is one
/// (S-0215 claim 31: only a field centred on the unit's cell counts).
pub fn centre_field_level(w: &World, c: CellXY) -> Option<i32> {
    w.radiation
        .fields
        .iter()
        .find(|f| f.centre == c)
        .map(field_level)
}

/// A cell's radiation *C* as the hit reads it.
pub fn cell_radiation(w: &World, c: CellXY) -> F64 {
    w.radiation.cells.get(&c).copied().unwrap_or(zero())
}

/// The exposure level ℓ of a cell (F6): trunc(min(*C*, *C*<sub>max</sub>)).
pub fn exposure(w: &World, c: CellXY) -> i32 {
    let r = w.rules();
    let mode = r.mode;
    let cap = big(i64::from(r.combat.general.radiation.level_max), mode);
    let cur = cell_radiation(w, c);
    // spec: S-0525 F6 (the comparison is exact; ℓ = trunc(min(C, C_max)))
    (if cur.gt(cap) { cap } else { cur }).to_i32_common()
}

/// The light of a field made or topped up for level `level` (F7, claims 13
/// to 15).
fn make_light(w: &World, level: i32, duration: i32) -> FieldLight {
    let r = w.rules();
    let mode = r.mode;
    let g = &r.combat.general.radiation;
    let cap = big(2000, mode); // spec: S-0525 Constants (light cap 2000 ‰)
    let upper = |v: F64| if v.gt(cap) { cap } else { v };
    // spec: S-0525 F7 (I = trunc(min(L × k_I, 2000)), one rounded multiplication)
    let intensity = upper(big(i64::from(level), mode).mul(g.light_factor, mode)).to_i32_common();
    let mut colour = g.colour;
    if w.theater == Theater::Snow {
        // spec: S-0525 claim 15 (on a snow-theater map the red and blue channels are −128 before the conversion)
        colour[0] = -128;
        colour[2] = -128;
    }
    let mut tint = [0; 3];
    for (t, k) in tint.iter_mut().zip(colour) {
        // spec: S-0525 F7 (q_c = trunc(1000 × κ_c / 255), toward zero)
        let q = 1000 * k / 255;
        // spec: S-0525 F7 (t_c = trunc(min(q_c × k_T, 2000)); an upper cap only)
        *t = upper(big(i64::from(q), mode).mul(g.tint_factor, mode)).to_i32_common();
    }
    // spec: S-0525 F2 (n_I = trunc(N / D_I); ΔI = trunc(I / n_I), 0 for a divisor of 0, claim 16)
    let steps = duration.checked_div(g.light_delay).unwrap_or(0);
    let step = intensity.checked_div(steps).unwrap_or(0);
    FieldLight {
        intensity,
        step,
        tint_start: tint,
        tint,
    }
}

/// The level steps of a duration (F2, claim 16: a divisor of 0 gives 0).
fn level_steps(w: &World, duration: i32) -> i32 {
    // spec: S-0525 F2 (n_L = trunc(N / D_L)) and claim 16 (a level delay of 0: n_L = 0)
    duration
        .checked_div(w.rules().combat.general.radiation.level_delay)
        .unwrap_or(0)
}

/// What a decay step takes from a cell: *c*/*n*<sub>L</sub> (F3).
fn step_amount(c: F64, steps: i32, mode: Rounding) -> F64 {
    // spec: S-0525 F3 (the quotient c / n_L with n_L converted exactly)
    c.div(big(i64::from(steps), mode), mode)
}

/// A detonation of `weapon` at `cell` leaves radiation there (S-0525
/// claims 1 and 2). Does nothing for a weapon of radiation level 0 or less.
pub fn detonation(w: &mut World, weapon: WeaponId, cell: CellXY) {
    let rules = w.rules_arc();
    let level = rules.combat.weapon(weapon).rad_level;
    if level <= 0 {
        // spec: S-0525 Import mapping (a radiation level of 0 or less leaves no radiation)
        return;
    }
    // spec: S-0525 claim 1 (s = trunc of the warhead's cell spread)
    let spread = rules
        .combat
        .warhead_of(weapon)
        .map_or(0, |h| h.spread.to_i32_common());
    // spec: S-0525 claim 2 (a detonation on the centre cell of a live field adds to it, whatever spread it has; otherwise a new field)
    if let Some(i) = w.radiation.fields.iter().position(|f| f.centre == cell) {
        top_up(w, i, level);
    } else {
        make(w, cell, spread, level);
    }
}

/// Makes a new field (claim 2).
fn make(w: &mut World, cell: CellXY, spread: i32, level: i32) {
    let rules = w.rules_arc();
    let g = &rules.combat.general.radiation;
    let mode = rules.mode;
    // spec: S-0525 F1 and F2 (R = 256 s + 128, wrapping; N = L × M, wrapping, T starts at N)
    let reach = spread.wrapping_mul(256).wrapping_add(128);
    let duration = level.wrapping_mul(g.duration_multiple);
    let f = Field {
        centre: cell,
        spread,
        level,
        duration,
        left: duration,
        level_start: w.tick,
        light_start: w.tick,
        level_steps: level_steps(w, duration),
        steps_taken: 0,
        reach,
        light: make_light(w, level, duration),
    };
    // spec: S-0525 claim 3 (the contribution of F1 is added to every cell the field reaches)
    for c in square(w, cell, spread) {
        let amount = contribution(w, cell, reach, level, c);
        add_to(&mut w.radiation.cells, c, amount, mode);
    }
    w.radiation.fields.push(f);
}

/// Adds radiation of level `added` to the field `i` (claim 4).
fn top_up(w: &mut World, i: usize, added: i32) {
    let rules = w.rules_arc();
    let g = &rules.combat.general.radiation;
    let mode = rules.mode;
    let f = w.radiation.fields[i].clone();
    // spec: S-0525 claim 4 and F4 (L′ = trunc(L × T / N) + L_w in floating point: L and T converted, the product, then the quotient)
    let current = if f.duration < 1 {
        0
    } else {
        big(i64::from(f.level), mode)
            .mul(big(i64::from(f.left), mode), mode)
            .div(big(i64::from(f.duration), mode), mode)
            .to_i32_common()
    };
    let level = current.wrapping_add(added);
    let cells = square(w, f.centre, f.spread);
    // spec: S-0525 claim 4 and claim 16 (the old field's remaining radiation is taken from every cell of its square; with no level steps nothing is removed)
    if f.level_steps != 0 {
        let steps = f.level_steps;
        let times = if rules.quirks.radiation_add_overdraw {
            // spec: S-0525 F5 (r = (c / n_L) × (⌊T / D_L⌋ + 1): one decay step more than remains when ρ ≥ σ; quirks.radiation_add_overdraw)
            f.left
                .checked_div(g.level_delay)
                .unwrap_or(0)
                .wrapping_add(1)
        } else {
            // spec: S-0525 F5 (the field's own remaining contribution, c × (1 − k / n_L); quirks.radiation_add_overdraw off)
            (steps - f.steps_taken).max(0)
        };
        for &c in &cells {
            let amount = contribution(w, f.centre, f.reach, f.level, c);
            let r = step_amount(amount, steps, mode).mul(big(i64::from(times), mode), mode);
            take_from(&mut w.radiation.cells, c, r, mode);
        }
    }
    // spec: S-0525 claim 4 (N′ = T′ = L′ × M; both timers start again now; the steps are recomputed for N′; the spread and light radius unchanged)
    let duration = level.wrapping_mul(g.duration_multiple);
    let nf = Field {
        level,
        duration,
        left: duration,
        level_start: w.tick,
        light_start: w.tick,
        level_steps: level_steps(w, duration),
        steps_taken: 0,
        // spec: S-0525 claim 4 (its light set afresh from L′ with its old radius)
        light: make_light(w, level, duration),
        ..f
    };
    // spec: S-0525 claim 4 (it then adds the contribution of L′ over its old reach)
    for &c in &cells {
        let amount = contribution(w, nf.centre, nf.reach, nf.level, c);
        add_to(&mut w.radiation.cells, c, amount, mode);
    }
    w.radiation.fields[i] = nf;
}

/// Row 13 of the tick (S-0203): the field updates, the most recently
/// created first (S-0525 claims 5, 6 and 14). Takes no draw (claim 12).
pub fn maintenance(w: &mut World) {
    for i in (0..w.radiation.fields.len()).rev() {
        update(w, i);
    }
}

fn update(w: &mut World, i: usize) {
    let rules = w.rules_arc();
    let g = &rules.combat.general.radiation;
    let mode = rules.mode;
    let t = w.tick;
    let mut f = w.radiation.fields[i].clone();
    // spec: S-0525 claim 5 (each field update lowers the time left by 1)
    f.left = f.left.wrapping_sub(1);
    // spec: S-0525 claim 5 and F2 (a decay step falls due at the first update with t − t_s ≥ D_L, and restarts the timer)
    if t.wrapping_sub(f.level_start) >= g.level_delay {
        if f.level_steps != 0 {
            for c in square(w, f.centre, f.spread) {
                let amount = contribution(w, f.centre, f.reach, f.level, c);
                // spec: S-0525 F3 (C ← max(0, C − c / n_L))
                let step = step_amount(amount, f.level_steps, mode);
                take_from(&mut w.radiation.cells, c, step, mode);
            }
        }
        f.level_start = t;
        f.steps_taken = f.steps_taken.wrapping_add(1);
    }
    // spec: S-0525 claim 14 and F7 (a light step at the first update with t − t_s ≥ D_I: the intensity drops by ΔI and each tint channel becomes trunc(t_c × T / N), integers)
    if t.wrapping_sub(f.light_start) >= g.light_delay {
        f.light.intensity = f.light.intensity.wrapping_sub(f.light.step);
        for k in 0..3 {
            f.light.tint[k] = f.light.tint_start[k]
                .wrapping_mul(f.left)
                .checked_div(f.duration)
                .unwrap_or(0);
        }
        f.light_start = t;
    }
    // spec: S-0525 claim 6 (a field ends in the update in which its time left reaches 0, after that update's decay step)
    if f.left <= 0 {
        if !rules.quirks.radiation_residue {
            // spec: S-0525 claim 6 and Edge cases (quirks.radiation_residue off: each cell loses what remains of the field's contribution, not below 0)
            let remain = if f.level_steps == 0 {
                None
            } else {
                Some((f.level_steps - f.steps_taken).max(0))
            };
            for c in square(w, f.centre, f.spread) {
                let amount = contribution(w, f.centre, f.reach, f.level, c);
                let r = match remain {
                    None => amount,
                    Some(times) => step_amount(amount, f.level_steps, mode)
                        .mul(big(i64::from(times), mode), mode),
                };
                take_from(&mut w.radiation.cells, c, r, mode);
            }
        }
        w.radiation.fields.remove(i);
    } else {
        w.radiation.fields[i] = f;
    }
}

/// Whether an object is airborne in the sense of S-0525 Terms: on the map
/// and at least 208 leptons above its surface, with the in-the-air answer
/// of S-0501 claim 18 for the two game-wide rocket aircraft types.
fn airborne(w: &World, id: ObjectId) -> bool {
    // spec: S-0525 Terms (airborne: the height test of S-0307; the flight phase for the two game-wide rocket aircraft types, S-0506 claim 22)
    if w.obj(id).is_some_and(|o| o.kind == Kind::Aircraft) {
        crate::combat::weapon::in_air(w, id)
    } else {
        crate::combat::weapon::airborne_by_height(w, id)
    }
}

/// The radiation hit of an object's own turn (claims 8 to 11): on a tick
/// that is a multiple of the application delay, an infantry unit, vehicle
/// or aircraft on the map that is not airborne and not immune takes one hit
/// of trunc(ℓ × *k*<sub>L</sub>) from the radiation of its own cell when ℓ
/// is at least 1.
pub fn hit_turn(w: &mut World, id: ObjectId) {
    if w.radiation.cells.is_empty() {
        return;
    }
    let rules = w.rules_arc();
    let g = &rules.combat.general.radiation;
    // spec: S-0525 claim 16 (an application delay of 0: fields damage nothing, tick 0 included) and claim 9 (t mod A = 0; a negative A acts as its absolute value)
    if g.application_delay == 0 || w.tick.checked_rem(g.application_delay) != Some(0) {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    // spec: S-0525 claim 8 (infantry units, vehicles and aircraft on the map; buildings and objects off the map are never hit)
    if !o.on_map || !matches!(o.kind, Kind::Infantry | Kind::Vehicle | Kind::Aircraft) {
        return;
    }
    // spec: S-0525 claim 8 (the immunity test of the type's own flag, applied before any hit is made)
    if type_combat(w, id).is_none_or(|tc| tc.immune_radiation) {
        return;
    }
    if airborne(w, id) {
        return;
    }
    // spec: S-0525 claim 8 (the cell of the object's position, on whatever layer it stands)
    let cell = o.pos.cell();
    let l = exposure(w, cell);
    // spec: S-0525 claim 9 (a hit only when ℓ ≥ 1)
    if l < 1 {
        return;
    }
    // spec: S-0525 Edge cases (radiation-site warhead absent: fields make no hits, a departure)
    let Some(site) = g.site_warhead else { return };
    // spec: S-0525 F6 (the amount trunc(ℓ × k_L), one rounded multiplication)
    let amount = big(i64::from(l), rules.mode)
        .mul(g.level_factor, rules.mode)
        .to_i32_common();
    // spec: S-0525 claim 10 (the radiation-site warhead at distance 0, defences applied, no attacker and no attacking house, the no-escape marker; delivered even when the amount is 0)
    damage::apply(
        w,
        id,
        Hit {
            amount,
            warhead: Some(site),
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: true,
        },
    );
}

/// The lights of the live fields, in creation order (claim 13): what the
/// map view adds to its cell lights. A field of negative spread has the
/// radius of its reach and lights no cell.
pub fn lights(w: &World) -> Vec<LightSource> {
    w.radiation
        .fields
        .iter()
        .map(|f| LightSource {
            centre: f.centre,
            // spec: S-0525 claim 13 and Edge cases (the light's radius is the field reach; a negative spread lights no cell, a departure)
            radius: if f.spread < 0 { 0 } else { f.reach },
            intensity: f.light.intensity,
            tint: f.light.tint,
        })
        .collect()
}
