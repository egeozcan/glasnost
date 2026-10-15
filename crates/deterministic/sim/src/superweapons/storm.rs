//! The weather storm (S-0513 claims 14 to 21, table T1, F4 to F6).
//!
//! One game-wide storm ([`super::Storm`]) and its present clouds
//! ([`super::Cloud`]). The storm's update is row 3 of the tick (S-0203
//! claim 1), before the turn pass. Cloud playback follows Q-0732's interim
//! rule: the cloud's frame in tick *t* is ⌊(*t* − *t*₀) / *d*⌋ with *d* its
//! drawn stage delay (1 when none is drawn), and its frame count is *F* =
//! 2μ for the animation's middle frame μ (0 when μ is unknown).

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::{F64, Rounding};

use super::{Cloud, StormState, SwEvent};
use crate::combat::area::{Detonation, detonate};
use crate::rules::{Kind, nearest_double};
use crate::world::{HouseId, Layer, ObjectId, Pos, World};

/// The storm warning interval (Constants).
// spec: S-0513 Constants (storm warning interval 225 ticks; claim 14)
pub const WARNING_INTERVAL: i32 = 225;

/// Scatter tries per period (Constants).
// spec: S-0513 Constants (scatter tries per period: 3; claim 16)
const SCATTER_TRIES: i32 = 3;

/// The height of one level, leptons.
// spec: S-0011 Constants (height level: 104 leptons); S-0513 F4 (the cell's height level times the level height)
const LEVEL: i32 = 104;

/// The bridge lift of F4's *b*₄: 4 levels on a high-bridge cell.
// spec: S-0513 F4 (b₄, the lift of 4 levels on a high-bridge cell)
const BRIDGE_LIFT: i32 = 4 * LEVEL;

/// The bolt image height *H* the simulation assumes (Q-0739 interim).
// spec: S-0513 F4 (H, the pixel height of the first bolt animation's image; not imported; Q-0739 interim: 0)
pub const BOLT_HEIGHT: i32 = 0;

/// *h*_c of F4 for a bolt image height of `h` pixels.
pub fn cloud_lift(h: i32, mode: Rounding) -> i32 {
    // spec: S-0513 F4 (k = 362.0386657714844 / (0.865897536277771 × 60), in double precision with S-0205 claim 5's rounding)
    let k = nearest_double("362.0386657714844").div(
        nearest_double("0.865897536277771").mul(F64::from_i64(60, mode), mode),
        mode,
    );
    // spec: S-0513 F4 (h_c = trunc((trunc(H / 2) − 0.5) × k), trunc(H / 2) an integer division toward zero)
    F64::from_i64(i64::from(h / 2), mode)
        .sub(nearest_double("0.5"), mode)
        .mul(k, mode)
        .to_i32_common()
}

/// The height level of a cell (0 off the map).
fn level(w: &World, c: CellXY) -> i32 {
    w.cells.cell(c).map_or(0, |x| i32::from(x.level))
}

/// The cell's height-level point at its centre plus `lift` on a
/// high-bridge cell (claim 18's strike point; F4 without *h*_c).
fn level_point(w: &World, c: CellXY, lift: i32) -> Pos {
    let cc = Pos::centre(c, 0);
    let b = if w.high_bridge(c) { lift } else { 0 };
    Pos::new(cc.x, cc.y, level(w, c) * LEVEL + b)
}

/// A storm request (claim 14; S-0524 claim 1): the launching house or the
/// trigger's owner, the target cell, the duration and the countdown *L*.
pub fn request(w: &mut World, owner: Option<HouseId>, cell: CellXY, duration: i32, countdown: i32) {
    let s = &mut w.sw.storm;
    if matches!(s.state, StormState::Raging | StormState::Clearing) {
        // spec: S-0513 claim 14 (a request while the storm is raging or clearing does nothing)
        return;
    }
    // spec: S-0513 claim 14 (centre, owner and duration are replaced by the request's)
    s.centre = cell;
    s.owner = owner;
    s.duration = duration;
    if countdown > 0 {
        // spec: S-0513 claim 14 (the remainder becomes L if none was running, else the smaller of the running remainder and L)
        s.countdown = if s.state == StormState::Counting && s.countdown > 0 {
            s.countdown.min(countdown)
        } else {
            countdown
        };
        s.state = StormState::Counting;
    } else {
        // spec: S-0513 claim 14 (with L = 0 the storm starts at once); T1 (idle or counting down, countdown 0: raging)
        start(w);
    }
}

/// The start of a storm (claim 15).
fn start(w: &mut World) {
    let t = w.tick;
    let (owner, centre, duration) = {
        let s = &mut w.sw.storm;
        s.state = StormState::Raging;
        s.start = t;
        s.countdown = 0;
        (s.owner, s.centre, s.duration)
    };
    // spec: S-0513 claim 15 (every house not defeated and not allied with the owner, every such house with no owner, starts a radar outage of the storm's duration; no local-player exception, departure)
    for i in 0..w.houses.len() {
        let h = HouseId(i as u8);
        if crate::econ::victory::defeated(w, h) {
            continue;
        }
        if owner.is_some_and(|o| w.allied(Some(o), Some(h))) {
            continue;
        }
        crate::econ::radar::start_outage(w, h, duration);
    }
    let p = super::cell_point(w, centre, false);
    // spec: S-0513 claim 15 (a radar event at the centre cell; the lighting re-evaluation is presentation, read from the storm state)
    super::event(w, SwEvent::Radar, owner, None, Some(p));
    if w.rules().superweapons.storm.print_text {
        // spec: S-0513 claim 15 (with the storm-message flag: the storm sound and the storm-start message for every player)
        super::event(w, SwEvent::StormStart, owner, None, Some(p));
    }
}

/// A storm-stop request (claim 20, trigger action 45): clearing begins,
/// only while raging.
pub fn stop(w: &mut World) {
    let t = w.tick;
    let s = &mut w.sw.storm;
    if s.state != StormState::Raging {
        // spec: S-0513 claim 19 (a stop request begins clearing only while raging; it cancels no countdown)
        return;
    }
    s.state = StormState::Clearing;
    // The weather update in which clearing began: this tick's when it already ran, else the one before.
    s.clearing_since = if s.last_update == t { t } else { t - 1 };
}

/// The storm-start trigger action (claim 20, action 44): a storm of
/// `duration` ticks with the game-wide countdown, at a random legal cell
/// and no owner.
pub fn start_action(w: &mut World, duration: i32) {
    if matches!(w.sw.storm.state, StormState::Raging | StormState::Clearing) {
        // spec: S-0513 claim 20 (like claim 14, nothing while raging or clearing)
        return;
    }
    let fp = *w.cells.footprint();
    let n = fp.width + fp.height - 1;
    let half = w.rules().quirks.ranged_draw_half_span;
    let cell = loop {
        // spec: S-0513 claim 20 (a row, then a column, each R(0, W + H − 1), until the cell is a legal map cell)
        let y = w.rng.ranged(0, n, half);
        let x = w.rng.ranged(0, n, half);
        let c = CellXY::new(x, y);
        if fp.is_legal(c) {
            break c;
        }
    };
    let def = w.rules().superweapons.storm.deferment;
    // spec: S-0513 claim 20 (no owner, the game-wide countdown)
    request(w, None, cell, duration, def);
}

/// A cloud made by a trigger action (claim 20; S-0524 claim 2), in any
/// storm state.
pub fn cloud(w: &mut World, cell: CellXY) {
    make_cloud(w, cell);
}

/// A cloud over `cell` (claim 17).
fn make_cloud(w: &mut World, cell: CellXY) {
    let list = w.rules().superweapons.storm.clouds.clone();
    if list.is_empty() {
        // spec: S-0513 claim 21 (an empty cloud animation list: no cloud is made)
        return;
    }
    // spec: S-0513 claim 17 (one raw value of the scenario stream modulo the cloud list's length)
    let k = w.rng.list_pick(list.len() as u32) as usize;
    let name = list[k].clone();
    let mode = w.rules().mode;
    let h_c = if w.rules().superweapons.storm.bolts.is_empty() {
        // spec: S-0513 claim 21 (an empty bolt list: h_c = 0)
        0
    } else {
        // spec: S-0513 F4 (h_c from the first bolt animation's image height H; Q-0739 interim H)
        cloud_lift(BOLT_HEIGHT, mode)
    };
    let mut pos = level_point(w, cell, BRIDGE_LIFT);
    // spec: S-0513 F4 (z = h_level + b₄ + h_c)
    pos.z += h_c;
    // spec: S-0513 claim 17 (its creation then takes the animation's own creation draws, S-0109)
    let delay = crate::load::anim_created_at(w, &name, Some((pos.x, pos.y, pos.z)));
    w.effects.push(crate::combat::effects::Effect::Anim {
        name: name.clone(),
        pos,
        delay,
    });
    let t = w.tick;
    w.sw.clouds.push(Cloud {
        cell,
        anim: name,
        pos,
        created: t,
        // spec: S-0513 claim 18 (Q-0732 interim: one frame per drawn stage delay, 1 when none is drawn)
        delay: delay.unwrap_or(1).max(1),
        pending: true,
    });
}

/// A cloud's frame count *F* (Q-0732 interim: 2μ, 0 when μ is unknown).
fn frames(w: &World, c: &Cloud) -> i32 {
    // spec: S-0513 claim 18 (F, the frame count of the cloud's image; Q-0732 interim F = 2μ, μ unknown: 0)
    w.rules()
        .animation(&c.anim)
        .and_then(|a| a.mu)
        .map_or(0, |mu| 2 * mu)
}

/// A cloud's current frame in tick `t` (Q-0732 interim).
fn frame(c: &Cloud, t: i32) -> i32 {
    // spec: S-0513 claim 18 (Q-0732 interim: frame ⌊(t − t₀) / d⌋)
    (t - c.created) / c.delay.max(1)
}

/// The storm's update in row 3 of the tick (claims 14 to 19, T1).
pub fn update(w: &mut World) {
    let t = w.tick;
    w.sw.storm.last_update = t;
    {
        let s = &mut w.sw.storm;
        if s.state == StormState::Clearing && t > s.clearing_since && w.sw.clouds.is_empty() {
            // spec: S-0513 claim 19 (idle in the first weather update after clearing began that finds no present cloud at its start; owner none; the lighting re-evaluation is presentation)
            s.state = StormState::Idle;
            s.owner = None;
        }
    }
    // spec: S-0513 claim 18 and T1 (in every weather update, in every storm state, pending clouds strike first, newest first)
    for i in (0..w.sw.clouds.len()).rev() {
        let c = w.sw.clouds[i].clone();
        let f = frames(w, &c);
        // spec: S-0513 claim 18 (strikes once its frame exceeds ⌊F / 2⌋; F ≤ 2 never strikes)
        if c.pending && f > 2 && frame(&c, t) > f / 2 {
            if let Some(x) = w.sw.clouds.get_mut(i) {
                x.pending = false;
            }
            strike(w, c.cell);
        }
    }
    // spec: S-0513 claim 16 (a cloud is present until its animation reaches its last frame; claim 19: removed in that update)
    let keep: Vec<bool> =
        w.sw.clouds
            .iter()
            .map(|c| frame(c, t) < frames(w, c) - 1)
            .collect();
    let mut it = keep.into_iter();
    w.sw.clouds.retain(|_| it.next().unwrap_or(false));
    match w.sw.storm.state {
        StormState::Counting => {
            let left = w.sw.storm.countdown - 1;
            // spec: S-0513 claim 14 (the countdown decreases by 1 in every weather update)
            w.sw.storm.countdown = left;
            if left <= 0 {
                // spec: S-0513 claim 14 (at 0 the storm starts in that update and makes no cloud in it)
                start(w);
            } else if left % WARNING_INTERVAL == 0 && w.rules().superweapons.storm.print_text {
                // spec: S-0513 claim 14 (a positive multiple of 225 with the storm-message flag: the storm-approach announcement and message for every player)
                let owner = w.sw.storm.owner;
                super::event(w, SwEvent::StormApproach, owner, None, None);
            }
        }
        StormState::Raging => {
            let s = &w.sw.storm;
            // spec: S-0513 claim 19 (clearing in the first weather update in which start + duration is below the current tick; −1 is endless; no cloud in that update)
            if s.duration != -1 && s.start.saturating_add(s.duration) < t {
                let s = &mut w.sw.storm;
                s.state = StormState::Clearing;
                s.clearing_since = t;
            } else {
                raging_clouds(w);
            }
        }
        StormState::Idle | StormState::Clearing => {}
    }
}

/// The clouds of a raging storm's update (claim 16, F6).
fn raging_clouds(w: &mut World) {
    let t = w.tick;
    let (hit, scatter, spread, sep) = {
        let s = &w.rules().superweapons.storm;
        (s.hit, s.scatter, s.spread, s.separation)
    };
    let centre = w.sw.storm.centre;
    // spec: S-0513 claim 16 (a centre cloud in every tick whose absolute number is a multiple of the hit period; claim 21: a period of 0 or less makes none)
    if hit > 0 && t % hit == 0 {
        make_cloud(w, centre);
    }
    // spec: S-0513 claim 16 (a scatter cloud in every tick whose number is a multiple of the scatter period; claim 21)
    if scatter > 0 && t % scatter == 0 {
        // spec: S-0513 F6 (r = ⌊s / 2⌋ toward minus infinity)
        let r = spread.div_euclid(2);
        let half = w.rules().quirks.ranged_draw_half_span;
        for _ in 0..SCATTER_TRIES {
            // spec: S-0513 F6 (Δx = R(−r, r), then Δy = R(−r, r))
            let dx = w.rng.ranged(-r, r, half);
            let dy = w.rng.ranged(-r, r, half);
            let c = CellXY::new(centre.x + dx, centre.y + dy);
            // spec: S-0513 claim 16 (a try fails when its cell is not a legal map cell)
            if !w.cells.footprint().is_legal(c) {
                continue;
            }
            // spec: S-0513 F6 (fails when |x − x′| + |y − y′| < separation for any present cloud cell, the same tick's centre cloud and trigger clouds included)
            let near =
                w.sw.clouds
                    .iter()
                    .any(|k| (k.cell.x - c.x).abs() + (k.cell.y - c.y).abs() < sep);
            if near {
                continue;
            }
            // spec: S-0513 claim 16 (the first try that does not fail makes one cloud; at most one per period)
            make_cloud(w, c);
            break;
        }
    }
}

/// What the debris test compares before and after a strike's damage
/// (claim 18; Q-0740 interim): the cell's first building, its nearest
/// unit and its height level.
struct CellState {
    building: Option<ObjectId>,
    unit: Option<ObjectId>,
    infantry: bool,
    level: i32,
    empty: bool,
}

fn cell_state(w: &World, c: CellXY) -> CellState {
    let list = w.cells.list(c, Layer::Ground);
    let centre = Pos::centre(c, 0);
    let mut unit: Option<(i64, ObjectId)> = None;
    let mut building = None;
    for &id in list {
        let Some(o) = w.obj(id) else { continue };
        match o.kind {
            Kind::Building => {
                building = building.or(Some(id));
            }
            Kind::Infantry | Kind::Vehicle | Kind::Aircraft => {
                let (dx, dy) = (i64::from(o.pos.x - centre.x), i64::from(o.pos.y - centre.y));
                let d = dx * dx + dy * dy;
                // Q-0740 interim: the smallest planar squared distance to the centre, ties to the first listed.
                if unit.is_none_or(|(b, _)| d < b) {
                    unit = Some((d, id));
                }
            }
            _ => {}
        }
    }
    let unit = unit.map(|u| u.1);
    CellState {
        building,
        unit,
        infantry: unit.is_some_and(|u| w.obj(u).is_some_and(|o| o.kind == Kind::Infantry)),
        level: level(w, c),
        empty: list.is_empty(),
    }
}

/// A strike at a cloud's cell (claim 18).
fn strike(w: &mut World, cell: CellXY) {
    let (bolts, sounds, damage, warhead) = {
        let s = &w.rules().superweapons.storm;
        (s.bolts.clone(), s.sounds.clone(), s.damage, s.warhead)
    };
    let half = w.rules().quirks.ranged_draw_half_span;
    let surface = super::cell_point(w, cell, false);
    if !bolts.is_empty() {
        // spec: S-0513 claim 18 (a bolt animation chosen by one raw value modulo the bolt list's length, at the cell centre at the terrain surface, with its creation draws; claim 21: none for an empty list)
        let k = w.rng.list_pick(bolts.len() as u32) as usize;
        crate::combat::effects::anim(w, &bolts[k], surface);
    }
    if !sounds.is_empty() {
        // spec: S-0701 claim 25 and F5 (one raw value modulo the lightning sound list's length, after the bolt's draws; none for an empty list)
        let k = w.rng.list_pick(sounds.len() as u32);
        let owner = w.sw.storm.owner;
        super::event(w, SwEvent::Lightning(k), owner, None, Some(surface));
    }
    // spec: S-0513 claim 18 (the cell centre at the cell's height level, plus the bridge lift on a high-bridge cell)
    let point = level_point(w, cell, crate::teleport::DECK_HEIGHT);
    let before = cell_state(w, cell);
    let land_type = w.cells.cell(cell).map_or(land::CLEAR, |c| c.land);
    // spec: S-0513 claim 19 (credited to the storm owner at that moment)
    let house = w.sw.storm.owner;
    if let Some(h) = warhead
        && damage != 0
    {
        // spec: S-0513 claim 18 (the explosion animation's draws before the damage draws; the storm explosion, or a splash on water, S-0303 F5)
        if let Some(name) = crate::combat::projectile::explosion_anim(w, h, damage, point, false) {
            crate::combat::effects::anim(w, &name, point);
        }
        // spec: S-0513 claim 18 (area damage of the storm damage with the storm warhead, no attacker object, affecting resources)
        let _ = detonate(
            w,
            Detonation {
                point,
                damage,
                warhead: Some(h),
                source: None,
                house,
                resources: true,
            },
        );
    }
    let after = cell_state(w, cell);
    let not_infantry = !before.infantry;
    let changed = before.building != after.building
        || before.unit != after.unit
        || before.level != after.level;
    // spec: S-0513 claim 18 (an empty cell whose land type is road, rock, wall or weeds, S-0211)
    let bare = before.empty
        && matches!(
            land_type,
            land::ROAD | land::ROCK | land::WALL | land::WEEDS
        );
    // spec: S-0513 claim 18 (debris when the nearest unit before the damage was not infantry and something changed, or the cell held nothing on bare land; Q-0740 interim details)
    if not_infantry && (changed || bare) {
        // spec: S-0513 claim 18 (R(2, 4) pieces)
        let n = w.rng.ranged(2, 4, half);
        let list = w.rules().combat.general.metallic_debris.clone();
        for _ in 0..n {
            // spec: S-0513 claim 18 (each chosen by R(0, n − 1) over the metallic debris list, then its animation-creation draws)
            let k = w.rng.ranged(0, list.len() as i32 - 1, half);
            if let Some(name) = list.get(k as usize) {
                // Q-0740 interim: at the strike point.
                crate::combat::effects::anim(w, name, point);
            }
        }
    }
}
