//! The weather storm (S-0513 claims 14 to 21, table T1, F4 to F6).
//!
//! One game-wide storm ([`super::Storm`]) and its present clouds
//! ([`super::Cloud`]). The storm's update is row 3 of the tick (S-0203
//! claim 1), before the turn pass. A cloud plays as an ordinary forward
//! one-pass animation (S-0513 claim 46, S-0109 claims 2 to 5): its stage
//! advances every effective stage delay *d* from the first advance
//! *a* = *c* + *d*, and it is tested against the frame count *F* of its image.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::geom::root_floor;
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
    // spec: S-0513 claim 15 (a radar event at the centre cell)
    super::event(w, SwEvent::Radar, owner, None, Some(p));
    // spec: S-0513 claim 15 and S-0104 claim 5 (an ion storm starting re-evaluates the lighting)
    crate::lighting::reevaluate(w);
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

/// The effective stage delay of an animation created now (S-0109 claims 2
/// to 4): the ranged draw `drawn` of a random rate, else its constant
/// delay, else the rate field's (1 when not given), then the game-speed
/// normalisation of a normalised type.
pub(crate) fn stage_delay(w: &World, a: &crate::rules::AnimRules, drawn: Option<i32>) -> i32 {
    let q = &w.rules().quirks;
    let (d1, d2) = a.random_delay;
    let d = if let Some(x) = drawn {
        // spec: S-0109 claim 3 (d₁′ ≠ d₂′: the ranged draw is the stage delay)
        x
    } else if (d1, d2) != (0, 0) {
        // spec: S-0109 claim 3 (d₁′ = d₂′, not both 0: no draw, the delay is that value)
        d1
    } else {
        match a.rate {
            // spec: S-0109 claim 2 (a type without the rate field has d = 1)
            None => 1,
            // spec: S-0109 claim 2 and F1 (d = ⌊900 / ρ⌋ for ρ > 0, 0 for ρ ≤ 0; a delay of 0 freezes the stage, it is not replaced by 1)
            Some(r) => crate::rules::anim_delay(r, q.anim_fast_rate_freezes),
        }
    };
    if a.normalized && q.speed_normalised_delays {
        // spec: S-0109 claim 4 (a normalised type: N(d, s) with s the game speed when the delay is chosen)
        crate::infantry::normalised(d, w.game_speed)
    } else {
        d
    }
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
    let rules = w.rules_arc();
    let Some(anim) = rules.animation(&name).filter(|a| a.frames.is_some()) else {
        // spec: S-0513 claim 21 (a selected cloud type with no image: no cloud is made, a bad-data departure; the importer's warning is the type's missing art)
        w.note(format!(
            "S-0513 claim 21: the cloud type {name} has no image; no cloud is made"
        ));
        return;
    };
    let h_c = match rules.superweapons.storm.bolts.first() {
        // spec: S-0513 claim 21 (an empty bolt list: h_c = 0)
        None => 0,
        Some(bolt) => {
            // spec: S-0513 F4 (h_c from the canvas height H of the first bolt animation's image header)
            let Some(h) = rules.animation(bolt).and_then(|b| b.canvas_height) else {
                // spec: S-0513 claim 21 and F4 (a nonempty bolt list whose first type has no image: no cloud is made, a bad-data departure, no switch)
                w.note(format!(
                    "S-0513 claim 21: the first bolt type {bolt} has no image; no cloud is made"
                ));
                return;
            };
            cloud_lift(h, mode)
        }
    };
    let mut pos = level_point(w, cell, BRIDGE_LIFT);
    // spec: S-0513 F4 (z = h_level + b₄ + h_c)
    pos.z += h_c;
    // spec: S-0513 claim 17 (its creation then takes the animation's own creation draws, S-0109)
    // The cloud's playback is the storm's model (claim 46, `Cloud`), so it is not an ordinary animation in play here.
    let delay = crate::load::anim_created(w, &name, Some((pos.x, pos.y, pos.z)), true);
    w.effects.push(crate::combat::effects::Effect::Anim {
        name: name.clone(),
        pos,
        delay,
        reverse: false,
        z_adjust: None,
        id: None,
    });
    let t = w.tick;
    let d = stage_delay(w, anim, delay);
    w.sw.clouds.push(Cloud {
        cell,
        anim: name,
        pos,
        created: t,
        // spec: S-0513 claim 46 (the effective stage delay of the ordinary playback; 0 freezes the stage)
        delay: d,
        pending: true,
    });
}

/// A cloud's frame count *F* (claim 46): the image-frame count of its
/// sprite.
fn frames(w: &World, c: &Cloud) -> i32 {
    // spec: S-0513 claim 46 (F is the frame count of the cloud's current image, not twice a middle field)
    w.rules()
        .animation(&c.anim)
        .and_then(|a| a.frames)
        .unwrap_or(0)
}

/// A cloud's stage as the weather update of tick `t` finds it (claim 46):
/// the advances made before it. The cloud is created before the turn pass
/// (the weather update, or a polled trigger), so its first update is in its
/// creation tick; with *d* ≥ 1 the first advance is in tick *a* = *c* + *d*
/// and advance *j* in *a* + (*j* − 1)*d*; weather precedes the animation
/// turns of its tick.
fn stage(c: &Cloud, t: i32) -> i32 {
    // spec: S-0513 claim 46 (a zero stage delay freezes the stage; it is not replaced by 1 tick)
    if c.delay < 1 {
        return 0;
    }
    // spec: S-0513 claim 46 and S-0109 claim 5 (the first advance is max(c + d, u + 1) = c + d with u = c; advance j ≥ 1 in a + (j − 1)d)
    let a = c.created.wrapping_add(c.delay);
    if t <= a { 0 } else { (t - 1 - a) / c.delay + 1 }
}

/// The storm's update in row 3 of the tick (claims 14 to 19, T1).
pub fn update(w: &mut World) {
    let t = w.tick;
    w.sw.storm.last_update = t;
    {
        let s = &mut w.sw.storm;
        if s.state == StormState::Clearing && t > s.clearing_since && w.sw.clouds.is_empty() {
            // spec: S-0513 claim 19 (idle in the first weather update after clearing began that finds no present cloud at its start; owner none)
            s.state = StormState::Idle;
            s.owner = None;
            // spec: S-0513 claim 19 and S-0104 claim 5 (an ion storm ending re-evaluates the lighting; this tick's fade already stepped towards the old target)
            crate::lighting::reevaluate(w);
        }
    }
    // spec: S-0513 claim 18 and T1 (in every weather update, in every storm state, pending clouds strike first, newest first)
    for i in (0..w.sw.clouds.len()).rev() {
        let c = &w.sw.clouds[i];
        // spec: S-0513 claim 18 (strikes once its cloud stage exceeds ⌊F / 2⌋, F the frame count of its image; an ordinary cloud with F ≤ 2 is removed first and never strikes)
        if c.pending && stage(c, t) > frames(w, c) / 2 {
            let cell = c.cell;
            if let Some(x) = w.sw.clouds.get_mut(i) {
                x.pending = false;
            }
            strike(w, cell);
        }
    }
    // spec: S-0513 claims 16 and 46 (a cloud is present until a weather update finds its cloud stage at least F − 1; claim 19: removed in that update)
    let keep: Vec<bool> =
        w.sw.clouds
            .iter()
            .map(|c| stage(c, t) < frames(w, c) - 1)
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
/// (claim 18): the cell's first ground-layer building, its strike witness
/// and its height level.
struct CellState {
    building: Option<ObjectId>,
    witness: Option<ObjectId>,
    infantry: bool,
    level: i32,
}

fn cell_state(w: &World, c: CellXY) -> CellState {
    let table = w.rules().quirks.table_root_distance;
    let mode = w.rules().mode;
    let mut witness: Option<(u64, ObjectId)> = None;
    let mut building = None;
    // spec: S-0513 claim 18 (the witness is selected only from the cell's ground-layer listing, buildings and landed aircraft included, terrain objects excluded; deck objects and aircraft not in that listing do not count)
    for &id in w.cells.list(c, Layer::Ground) {
        let Some(o) = w.obj(id) else { continue };
        if !matches!(
            o.kind,
            Kind::Building | Kind::Infantry | Kind::Vehicle | Kind::Aircraft
        ) {
            continue;
        }
        if o.kind == Kind::Building {
            building = building.or(Some(id));
        }
        // spec: S-0513 claim 18 (x and y the reference point's integer lepton coordinates modulo 256, a building's being its foundation centre (Q-0800, S-0204 claim 4) and any other object's its position; the distance trunc(r(x² + y²)) by S-0204 claim 5's table square root; the remainders are compared with (0, 0), height and the building-size distance reduction do not enter)
        let p = w.reference_point(id);
        let (x, y) = (p.x.rem_euclid(256) as u64, p.y.rem_euclid(256) as u64);
        let d = root_floor(x * x + y * y, table, mode);
        // spec: S-0513 claim 18 (the smallest integer distance wins; ties go to the first candidate in S-0209 claim 13's listing order)
        if witness.is_none_or(|(b, _)| d < b) {
            witness = Some((d, id));
        }
    }
    let witness = witness.map(|u| u.1);
    CellState {
        building,
        witness,
        infantry: witness.is_some_and(|u| w.obj(u).is_some_and(|o| o.kind == Kind::Infantry)),
        level: level(w, c),
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
    // spec: S-0513 claim 18 (the pre-damage strike witness is not infantry; no witness also passes)
    let not_infantry = !before.infantry;
    // spec: S-0513 claim 18 (the first ground-layer building, the strike witness or the cell height level differs after damage; mere health loss without a changed witness, building or height does not)
    let changed = before.building != after.building
        || before.witness != after.witness
        || before.level != after.level;
    // spec: S-0513 claim 18 ("held nothing" is neither a ground-layer building nor a strike witness, not an empty object list: a terrain object alone does not prevent it; the land type is road, rock, wall or weeds, S-0211)
    let bare = before.building.is_none()
        && before.witness.is_none()
        && matches!(
            land_type,
            land::ROAD | land::ROCK | land::WALL | land::WEEDS
        );
    // spec: S-0513 claim 18 (debris as above)
    if not_infantry && (changed || bare) {
        // spec: S-0513 claim 18 (R(2, 4) pieces)
        let n = w.rng.ranged(2, 4, half);
        let list = w.rules().combat.general.metallic_debris.clone();
        for _ in 0..n {
            // spec: S-0513 claim 18 (each chosen by R(0, n − 1) over the metallic debris list, then its animation-creation draws)
            let k = w.rng.ranged(0, list.len() as i32 - 1, half);
            if let Some(name) = list.get(k as usize) {
                // spec: S-0513 claim 18 (every debris animation is created at the strike point: the cell centre at its height level plus the high-bridge lift, no additional 20-lepton lift, no horizontal displacement)
                crate::combat::effects::anim(w, name, point);
            }
        }
    }
}
