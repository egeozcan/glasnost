//! A per-unit trace of one oracle arena game (M8 step A3 intake 38, for
//! M-0011): what each unit of the two groups does tick by tick, with the
//! facts M-0011 tabulates for CAOS against BFRT (berserk entries, their
//! remaining time at each hit, resets, fire of one unit at another of its
//! own group, and how each unit leaves play). Since intake 40 also the
//! magnetic-lift facts M-0015 tabulates for TELE against BFRT: each unit's
//! lifter (the record's *holder*), whether it is seized, set down or
//! dropped, the record's sorting of the lifters' fire events, every hold
//! with its release and landing, the restorations, and the hits between
//! the two groups.
//!
//! Tooling only: it plays the game exactly as [`oracle::play`] does and
//! reads state after each tick; it touches no simulation state. Our own
//! diagnostic; no spec applies.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::world::{ObjectId, Target, World};

use crate::oracle::{self, Arena, FRAME_LIMIT, Ground, SAMPLE_PERIOD, Side};

/// One unit's state as the trace compares it from tick to tick.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Seen {
    health: i32,
    cell: (i32, i32),
    z: i32,
    beh: String,
    target: Option<Target>,
    dest: Option<Target>,
    berserk: Option<i32>,
    /// Magnetic lift (M-0015): its lifter (the record's *holder*), and
    /// whether it is seized, set down or dropped (S-0607 Terms).
    lifter: Option<ObjectId>,
    seized: bool,
    set_down: bool,
    dropped: bool,
    /// Height above the surface (S-0607 claim 13).
    above: i32,
    /// Disabled by the control-centre rule (S-0405 claim 13; M-0005).
    disabled: bool,
}

fn seen(w: &World, id: ObjectId) -> Option<Seen> {
    let o = w.obj(id)?;
    Some(Seen {
        health: o.health,
        cell: (o.pos.cell().x, o.pos.cell().y),
        z: o.pos.z,
        beh: format!("{:?}", o.beh.current),
        target: o.target,
        dest: o.dest,
        berserk: o.combat.berserk,
        lifter: o.lift.lifter,
        seized: o.lift.seized,
        set_down: o.lift.set_down,
        dropped: o.lift.dropped,
        above: glasnost_sim::jump::height_above(w, id),
        disabled: glasnost_sim::econ::power::unit_disabled(w, id),
    })
}

/// Per-unit counts the summary gives.
#[derive(Default, Clone, Debug)]
struct Tally {
    first_berserk: Option<i32>,
    berserk_entries: Vec<(i32, i32)>,
    berserk_exits: Vec<i32>,
    /// Ticks at which the remaining time was set anew while berserk, with
    /// the value before and after.
    resets: Vec<(i32, i32, i32)>,
    /// Ticks with no change of the remaining time while berserk.
    held: Vec<i32>,
    gas_hits: u32,
    shots_at_own_group: u32,
    shots_at_other_group: u32,
    shots_other: u32,
    /// Ticks at which it took an object of the other group as target
    /// while berserk.
    berserk_other_targets: u32,
    /// Ticks at which it took an object of its own group as target while
    /// berserk.
    berserk_own_targets: u32,
    left: Option<String>,
}

fn label(units: &[Vec<ObjectId>; 2], id: ObjectId) -> String {
    for (s, ids) in units.iter().enumerate() {
        if ids.contains(&id) {
            return format!("{}#{}", ["A", "B"][s], id.0);
        }
    }
    format!("#{}", id.0)
}

fn side_of(units: &[Vec<ObjectId>; 2], id: ObjectId) -> Option<usize> {
    units.iter().position(|ids| ids.contains(&id))
}

fn target_text(units: &[Vec<ObjectId>; 2], t: Option<Target>) -> String {
    match t {
        None => "-".into(),
        Some(Target::Object(x)) => label(units, x),
        Some(Target::Cell(c)) => format!("cell({}, {})", c.x, c.y),
    }
}

/// Plays one game of the oracle arena and returns the per-unit trace
/// (`detail`: every change of every unit, else only the summary).
pub fn trace(
    ground: &Ground,
    row: &str,
    col: &str,
    row_side: Side,
    seed: u32,
    detail: bool,
) -> Result<String, String> {
    let types = match row_side {
        Side::A => [row, col],
        Side::B => [col, row],
    };
    let Arena {
        world: mut w,
        houses,
        units,
        ..
    } = oracle::setup(ground, types, row_side, seed)?;
    let mut out = String::new();
    let all: Vec<ObjectId> = units.iter().flatten().copied().collect();
    let mut prev: BTreeMap<ObjectId, Option<Seen>> =
        all.iter().map(|&id| (id, seen(&w, id))).collect();
    let mut tally: BTreeMap<ObjectId, Tally> =
        all.iter().map(|&id| (id, Tally::default())).collect();
    let _ = writeln!(
        out,
        "trace: {row} against {col}, row on side {}, seed {seed}; side A {} ({} units), side B {} ({} units)",
        row_side.text(),
        types[0],
        units[0].len(),
        types[1],
        units[1].len()
    );
    for &id in &all {
        let crusher = glasnost_sim::occupancy::is_crusher(&w, id);
        let _ = writeln!(
            out,
            "  {} start {:?}; crusher {crusher}",
            label(&units, id),
            prev[&id]
        );
    }
    // Which other-side units each crusher may crush, at the start.
    for &a in &all {
        if !glasnost_sim::occupancy::is_crusher(&w, a) {
            continue;
        }
        let n = all
            .iter()
            .filter(|&&b| b != a && glasnost_sim::occupancy::crushable_by(&w, a, b))
            .count();
        let _ = writeln!(out, "  {} may crush {n} of the units", label(&units, a));
    }
    // M-0015: every unit's lifter and height above the surface at the end
    // of each tick (index 0 the state before the first tick), the fire
    // events and the hits.
    let mut lift = LiftLog::default();
    lift.record(&w, &all);
    let end = loop {
        glasnost_sim::tick::step(&mut w);
        let t = w.tick;
        let effects = std::mem::take(&mut w.effects);
        lift.record(&w, &all);
        let mut crush_sounds = 0;
        for e in &effects {
            match e {
                Effect::Hit {
                    id,
                    result,
                    attacker,
                    ..
                } if all.contains(id) => {
                    lift.hits.push((t, *id, *attacker, format!("{result:?}")));
                }
                Effect::Detonation { .. } if detail => {
                    let _ = writeln!(out, "  {t}: {e:?}");
                }
                _ => {}
            }
            match e {
                Effect::Shot { firer, .. } if all.contains(firer) => {
                    let tg = w.obj(*firer).and_then(|o| o.target);
                    if let Some(Target::Object(x)) = tg {
                        lift.fires.push((t, *firer, x));
                    }
                    let (fs, ts) = (
                        side_of(&units, *firer),
                        match tg {
                            Some(Target::Object(x)) => side_of(&units, x),
                            _ => None,
                        },
                    );
                    let tl = tally.get_mut(firer).expect("a unit of a group");
                    match ts {
                        Some(s) if Some(s) == fs => tl.shots_at_own_group += 1,
                        Some(_) => tl.shots_at_other_group += 1,
                        None => tl.shots_other += 1,
                    }
                    if detail {
                        let _ = writeln!(
                            out,
                            "  {t}: {} fires at {}",
                            label(&units, *firer),
                            target_text(&units, tg)
                        );
                    }
                }
                Effect::GasHit { id, .. } if all.contains(id) => {
                    tally.get_mut(id).expect("a unit").gas_hits += 1;
                    if detail {
                        let _ = writeln!(out, "  {t}: gas hit on {}", label(&units, *id));
                    }
                }
                Effect::Hit {
                    id,
                    result,
                    attacker,
                    ..
                } if detail && all.contains(id) => {
                    let _ = writeln!(
                        out,
                        "  {t}: hit on {} by {} result {result:?}",
                        label(&units, *id),
                        attacker.map_or("-".into(), |a| label(&units, a))
                    );
                }
                Effect::Killed { id, .. } if detail && all.contains(id) => {
                    let _ = writeln!(out, "  {t}: {} killed", label(&units, *id));
                }
                Effect::Sound {
                    sound: Sound::Crush,
                    ..
                } => crush_sounds += 1,
                _ => {}
            }
        }
        for &id in &all {
            let now = seen(&w, id);
            let before = prev[&id].clone();
            if now == before {
                continue;
            }
            let tl = tally.get_mut(&id).expect("a unit");
            match (&before, &now) {
                (Some(b), None) => {
                    // Who stood on its cell in the tick before.
                    let on: Vec<String> = all
                        .iter()
                        .filter(|&&x| x != id)
                        .filter_map(|&x| prev[&x].as_ref().map(|s| (x, s)))
                        .filter(|(_, s)| s.cell == b.cell)
                        .map(|(x, s)| {
                            format!(
                                "{} (z {}, target {})",
                                label(&units, x),
                                s.z,
                                target_text(&units, s.target)
                            )
                        })
                        .collect();
                    let why = format!(
                        "gone at {t} from cell {:?} health {}; crush sounds this tick {crush_sounds}; on its cell the tick before: [{}]",
                        b.cell,
                        b.health,
                        on.join(", ")
                    );
                    if detail {
                        let _ = writeln!(out, "  {t}: {} {why}", label(&units, id));
                    }
                    tl.left = Some(why);
                }
                (Some(b), Some(n)) => {
                    match (b.berserk, n.berserk) {
                        (None, Some(v)) => {
                            tl.first_berserk.get_or_insert(t);
                            tl.berserk_entries.push((t, v));
                        }
                        (Some(_), None) => tl.berserk_exits.push(t),
                        (Some(x), Some(y)) if y != x - 1 => {
                            if y == x {
                                tl.held.push(t);
                            } else {
                                tl.resets.push((t, x, y));
                            }
                        }
                        _ => {}
                    }
                    if n.berserk.is_some() && n.target != b.target {
                        match n.target {
                            Some(Target::Object(x))
                                if side_of(&units, x) == side_of(&units, id) =>
                            {
                                tl.berserk_own_targets += 1
                            }
                            Some(Target::Object(_)) => tl.berserk_other_targets += 1,
                            _ => {}
                        }
                    }
                    if detail {
                        let mut parts = Vec::new();
                        if b.health != n.health {
                            parts.push(format!("health {} -> {}", b.health, n.health));
                        }
                        if b.beh != n.beh {
                            parts.push(format!("behaviour {} -> {}", b.beh, n.beh));
                        }
                        if b.target != n.target {
                            parts.push(format!(
                                "target {} -> {}",
                                target_text(&units, b.target),
                                target_text(&units, n.target)
                            ));
                        }
                        if b.dest != n.dest {
                            parts.push(format!(
                                "destination {} -> {}",
                                target_text(&units, b.dest),
                                target_text(&units, n.dest)
                            ));
                        }
                        if b.berserk != n.berserk
                            && !matches!((b.berserk, n.berserk), (Some(x), Some(y)) if y == x - 1)
                        {
                            parts.push(format!("berserk {:?} -> {:?}", b.berserk, n.berserk));
                        }
                        if b.lifter != n.lifter {
                            parts.push(format!(
                                "lifter {} -> {}",
                                b.lifter.map_or("-".into(), |x| label(&units, x)),
                                n.lifter.map_or("-".into(), |x| label(&units, x))
                            ));
                        }
                        if (b.seized, b.set_down, b.dropped) != (n.seized, n.set_down, n.dropped) {
                            parts.push(format!(
                                "seized/set down/dropped {:?} -> {:?}",
                                (b.seized, b.set_down, b.dropped),
                                (n.seized, n.set_down, n.dropped)
                            ));
                        }
                        if b.disabled != n.disabled {
                            parts.push(format!("disabled {} -> {}", b.disabled, n.disabled));
                        }
                        if b.cell != n.cell {
                            parts.push(format!("cell {:?} -> {:?}", b.cell, n.cell));
                        }
                        if b.z != n.z && (b.z - n.z).abs() > 8 {
                            parts.push(format!("z {} -> {}", b.z, n.z));
                        }
                        if !parts.is_empty() {
                            let _ = writeln!(
                                out,
                                "  {t}: {} at {:?}: {}",
                                label(&units, id),
                                n.cell,
                                parts.join("; ")
                            );
                        }
                    }
                }
                _ => {}
            }
            prev.insert(id, now);
        }
        let finished = w.econ.end.finished;
        let frame = if finished {
            (t + SAMPLE_PERIOD - 1) / SAMPLE_PERIOD * SAMPLE_PERIOD
        } else {
            t
        };
        if !finished && (frame <= 0 || frame % SAMPLE_PERIOD != 0) {
            continue;
        }
        let o = houses.map(|h| oracle::owned(&w, h));
        match (o[0].0 > 0, o[1].0 > 0) {
            (true, true) if frame < FRAME_LIMIT && !finished => {}
            _ => break (frame, o),
        }
    };
    let _ = writeln!(
        out,
        "end at {}: A owns {} (value {}), B owns {} (value {})",
        end.0, end.1[0].0, end.1[0].1, end.1[1].0, end.1[1].1
    );
    for (s, ids) in units.iter().enumerate() {
        let _ = writeln!(out, "side {} ({}):", ["A", "B"][s], types[s]);
        for id in ids {
            let tl = &tally[id];
            let _ = writeln!(
                out,
                "  {}: first berserk {:?}; entries {:?}; exits {:?}; resets {} {:?}; held {:?}; gas hits {}; shots at own group {}, other group {}, else {}; berserk targets own {} other {}; {}",
                label(&units, *id),
                tl.first_berserk,
                tl.berserk_entries,
                tl.berserk_exits,
                tl.resets.len(),
                tl.resets.iter().take(4).collect::<Vec<_>>(),
                tl.held.iter().take(6).collect::<Vec<_>>(),
                tl.gas_hits,
                tl.shots_at_own_group,
                tl.shots_at_other_group,
                tl.shots_other,
                tl.berserk_own_targets,
                tl.berserk_other_targets,
                tl.left.as_deref().unwrap_or("in play at the end")
            );
        }
    }
    lift.summary(
        &mut out,
        &units,
        match row_side {
            Side::A => 0,
            Side::B => 1,
        },
    );
    let notes: Vec<&String> = w.notes.iter().collect();
    for n in notes {
        let _ = writeln!(out, "  note: {n}");
    }
    Ok(out)
}

/// One unit's lift state at the end of a tick: its lifter, its height above
/// the surface, and whether it is seized.
type LiftSeen = (Option<ObjectId>, i32, bool);

/// The magnetic-lift facts M-0015 tabulates for TELE against BFRT, read
/// from the state after each tick.
#[derive(Default)]
struct LiftLog {
    /// Per tick (index 0 before the first tick), the units in play.
    ticks: Vec<BTreeMap<ObjectId, LiftSeen>>,
    /// Fire events at an object: tick, firer, target.
    fires: Vec<(i32, ObjectId, ObjectId)>,
    /// Hits: tick, target, attacker, result.
    hits: Vec<(i32, ObjectId, Option<ObjectId>, String)>,
}

impl LiftLog {
    fn record(&mut self, w: &World, all: &[ObjectId]) {
        let m = all
            .iter()
            .filter_map(|&id| {
                let o = w.obj(id)?;
                Some((
                    id,
                    (
                        o.lift.lifter,
                        glasnost_sim::jump::height_above(w, id),
                        o.lift.seized,
                    ),
                ))
            })
            .collect();
        self.ticks.push(m);
    }

    fn holder(&self, t: i32, id: ObjectId) -> Option<ObjectId> {
        let t = t.clamp(0, self.ticks.len() as i32 - 1) as usize;
        self.ticks[t].get(&id).and_then(|s| s.0)
    }

    /// `mag`: the side of the row group (the lifters in TELE against BFRT).
    fn summary(&self, out: &mut String, units: &[Vec<ObjectId>; 2], mag: usize) {
        let other = 1 - mag;
        let last = self.ticks.len() as i32 - 1;
        let mut started = 0;
        let mut met_held = 0;
        let mut free = 0;
        let mut free_seized = 0;
        let mut while_holding = 0;
        for (i, &(t, firer, x)) in self.fires.iter().enumerate() {
            if side_of(units, firer) != Some(mag) || side_of(units, x) != Some(other) {
                continue;
            }
            // The firer held a unit at the end of the tick before.
            if self.ticks[(t - 1).max(0) as usize]
                .values()
                .any(|s| s.0 == Some(firer))
            {
                while_holding += 1;
            }
            let next = self.fires[i + 1..]
                .iter()
                .find(|f| f.1 == firer)
                .map_or(i32::MAX, |f| f.0);
            let window = (t + 10).min(next - 1).min(last);
            let before = self.holder(t - 1, x);
            if before != Some(firer) && self.holder(window, x) == Some(firer) {
                started += 1;
            } else if self.holder(t, x).is_some_and(|h| h != firer) {
                met_held += 1;
            } else {
                free += 1;
                let t2 = t.clamp(0, last) as usize;
                if self.ticks[t2].get(&x).is_some_and(|s| s.2) {
                    free_seized += 1;
                }
            }
        }
        let _ = writeln!(
            out,
            "lift: magnetron fire events at fortresses {}: started a hold {started}; met a fortress held by another {met_held}; else {free} (of those, target seized at the end of the fire tick {free_seized}); made while holding {while_holding}",
            started + met_held + free
        );
        // Holds: runs of ticks with a lifter.
        for &x in &units[other] {
            let mut t = 0;
            while t <= last {
                let h = self.holder(t, x);
                if h.is_none() || !self.ticks[t as usize].contains_key(&x) {
                    t += 1;
                    continue;
                }
                let start = t;
                while t <= last && self.holder(t, x) == h {
                    t += 1;
                }
                let release = t;
                let above = self.ticks[(release.min(last)) as usize]
                    .get(&x)
                    .map(|s| s.1);
                let landing = (release..=last)
                    .find(|&u| self.ticks[u as usize].get(&x).is_none_or(|s| s.1 <= 0));
                let gone = landing.is_some_and(|u| !self.ticks[u as usize].contains_key(&x));
                let hits: Vec<String> = landing
                    .map(|u| {
                        self.hits
                            .iter()
                            .filter(|h| h.0 == u && h.1 == x)
                            .map(|h| {
                                format!("{} {}", h.2.map_or("-".into(), |a| label(units, a)), h.3)
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let _ = writeln!(
                    out,
                    "  hold {} by {}: ticks {start}..{release} ({}); above at release {above:?}; landing {landing:?}{}; hits at landing [{}]",
                    label(units, x),
                    h.map_or("-".into(), |a| label(units, a)),
                    if release > last {
                        "running at the end"
                    } else {
                        "released"
                    },
                    if gone { " (left play)" } else { "" },
                    hits.join(", ")
                );
            }
        }
        for &x in &units[other] {
            let restored: Vec<usize> = (1..self.ticks.len())
                .filter(|&u| {
                    self.ticks[u - 1].get(&x).is_some_and(|s| s.2)
                        && self.ticks[u].get(&x).is_some_and(|s| !s.2)
                })
                .collect();
            let _ = writeln!(
                out,
                "  {}: restored (seized cleared) at ticks {:?}",
                label(units, x),
                restored
            );
        }
        let fortress_hits_on_magnetrons = self
            .hits
            .iter()
            .filter(|h| side_of(units, h.1) == Some(mag))
            .filter(|h| h.2.is_some_and(|a| side_of(units, a) == Some(other)))
            .count();
        let magnetron_attacks = self
            .hits
            .iter()
            .filter(|h| h.2.is_some_and(|a| side_of(units, a) == Some(mag)))
            .count();
        let _ = writeln!(
            out,
            "lift: hits on magnetrons by fortresses {fortress_hits_on_magnetrons}; hits with a magnetron as attacker {magnetron_attacks}"
        );
        for h in &self.hits {
            if h.2.is_some_and(|a| side_of(units, a) == Some(mag)) {
                let _ = writeln!(
                    out,
                    "  {}: hit on {} by {} result {}",
                    h.0,
                    label(units, h.1),
                    label(units, h.2.expect("an attacker")),
                    h.3
                );
            }
        }
    }
}
