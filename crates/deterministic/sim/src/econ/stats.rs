//! The result screen's statistics (S-0414 claims 19 to 23): the built
//! total, kills, losses, the running score and the displayed score.
//!
//! These are the result screen's own counters, apart from the
//! destroyed-object counts and the kill-value score of S-0302 claim 14 and
//! S-0311 claim 7 (`combat::state::HouseCombat`): S-0414 claim 21 gives no
//! kill for a destruction with neither an attacking object nor an
//! attacking house, adds ownership gains, and excludes not-scored victims
//! and insignificant buildings from the kills; claim 22 scores a
//! destruction credited to a house alone with no flag exclusion.
//!
//! The counters are kept while play runs ([`destruction`],
//! [`owner_changed`], [`resource`]); [`finish`] fixes the displayed scores
//! once, after the simulation has stopped, taking the winner-score draws
//! of claim 23 on the scenario stream (S-0202 claim 16).

use glasnost_num::F64;
use serde::{Deserialize, Serialize};

use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, World};

/// A house's result-screen counters (S-0414 claims 21 to 23).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    /// Kills (claim 21), a signed 32-bit count that wraps.
    pub kills: i32,
    /// Losses (claim 21), a signed 32-bit count that wraps.
    pub losses: i32,
    /// The running score *p* (claim 22).
    pub score: i32,
    /// The displayed score, fixed once when play has stopped (claim 23).
    pub shown: Option<i32>,
}

fn stats_mut(w: &mut World, h: HouseId) -> Option<&mut Stats> {
    w.houses
        .get_mut(usize::from(h.0))
        .map(|x| &mut x.econ.result.stats)
}

/// The house's counters.
pub fn of(w: &World, h: HouseId) -> Stats {
    w.houses
        .get(usize::from(h.0))
        .map(|x| x.econ.result.stats.clone())
        .unwrap_or_default()
}

/// The built total (claim 20): the sum of the four kinds' ever-built
/// counts, with the signed 32-bit wrap.
pub fn built_total(w: &World, h: HouseId) -> i32 {
    // spec: S-0414 claim 20 (the sum of the ever-built counts, signed 32-bit wrap, no rounding)
    w.houses.get(usize::from(h.0)).map_or(0, |x| {
        x.econ
            .tech
            .ever_built
            .values()
            .fold(0i32, |a, &b| a.wrapping_add(b))
    })
}

fn not_scored(w: &World, id: ObjectId) -> bool {
    crate::combat::veterancy::type_combat(w, id).is_some_and(|c| c.dont_score)
}

fn insignificant_building(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    o.kind == Kind::Building
        && w.rules()
            .buildings
            .get(o.type_index as usize)
            .and_then(|b| b.raw.combat.insignificant)
            .unwrap_or(false)
}

/// The type's current price for a house (S-0404 claim 2), 0 without one.
fn current_price(w: &World, id: ObjectId, house: Option<HouseId>) -> i32 {
    let (Some(o), Some(h)) = (w.obj(id), house) else {
        return 0;
    };
    super::production::price(w, h, super::tech::type_of(o))
}

/// Who a destruction is credited to (S-0302 claim 14), as the result
/// screen's counters tell the cases apart (S-0414 claims 21 and 22).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Credit {
    /// Neither an attacking object nor an attacking house.
    Nobody,
    /// An attacking object: by a hit whose kill S-0302 claim 14 credits to
    /// it, or by another destruction S-0311 claim 9 credits to it.
    Object(ObjectId),
    /// An attacking house alone: a hit with an attacking house and no
    /// attacking object, or one whose attacking house differs from its
    /// attacking object's owner.
    House(HouseId),
}

impl Credit {
    /// The credit of a destruction to an attacking object, if any (the
    /// callers whose rule allows no house credit, S-0312 claim 30).
    pub fn object(object: Option<ObjectId>) -> Self {
        object.map_or(Self::Nobody, Self::Object)
    }
}

/// A destruction (S-0414 claims 21 and 22), at the registration of S-0302
/// claim 14, credited as `credit` says. Called before the destruction's
/// other bookkeeping, with the victim still in the world.
pub fn destruction(w: &mut World, victim: ObjectId, credit: Credit) {
    let Some(v) = w.obj(victim).cloned() else {
        return;
    };
    let scored = !not_scored(w, victim);
    // spec: S-0414 claim 21 (an insignificant building gives neither a kill nor a loss, whether or not it is also not scored; an insignificant unit still does)
    let significant = !insignificant_building(w, victim);
    let (scorer, by_house) = match credit {
        Credit::Nobody => (None, false),
        Credit::Object(a) => (w.obj(a).and_then(|a| a.owner), false),
        Credit::House(h) => (Some(h), true),
    };
    // spec: S-0414 claim 21 (a loss to the victim's house, also with neither an attacking object nor an attacking house; a not-scored victim gives none, except by a hit whose kill S-0302 claim 14 credits to the attacking house)
    if significant
        && (scored || by_house)
        && let Some(vh) = v.owner
        && let Some(s) = stats_mut(w, vh)
    {
        s.losses = s.losses.wrapping_add(1);
    }
    // spec: S-0414 claims 21 and 22 (a destruction with neither an attacking object nor an attacking house: no kill and no destruction score)
    let Some(scorer) = scorer else {
        return;
    };
    // spec: S-0414 claim 22 (the victim's rank: 1 rookie, 2 veteran, 3 elite)
    let mult = match crate::combat::veterancy::rank_of(v.combat.experience) {
        crate::combat::veterancy::Rank::Rookie => 1,
        crate::combat::veterancy::Rank::Veteran => 2,
        crate::combat::veterancy::Rank::Elite => 3,
    };
    // spec: S-0414 claim 22 (0 when the scoring house allies the victim's house, as the scoring house sees its alliances; else the victim type's current price for the victim's house times the rank multiplier)
    let value = if w.allied(Some(scorer), v.owner) {
        0
    } else {
        current_price(w, victim, v.owner).wrapping_mul(mult)
    };
    if let Some(s) = stats_mut(w, scorer) {
        if scored && significant {
            // spec: S-0414 claim 21 (kills: credited destructions of every victim house, allies and the same house included; to the attacking house for a hit S-0302 claim 14 credits to it; a not-scored victim and an insignificant building give none, whoever is credited)
            s.kills = s.kills.wrapping_add(1);
        }
        if scored || by_house {
            // spec: S-0414 claim 22 (the object-credit row excludes not-scored victims; the two attacking-house rows exclude nothing; an insignificant building still gives destruction score)
            s.score = s.score.wrapping_add(value);
        }
    }
}

/// An object changed owner from `old` to its present owner (S-0414 claims
/// 21 and 22): the gaining house's ownership-gain kill and score.
pub fn owner_changed(w: &mut World, id: ObjectId, old: Option<HouseId>) {
    let Some(new) = w.obj(id).and_then(|o| o.owner) else {
        return;
    };
    // spec: S-0414 claim 21 (a change of owner to a different house)
    if old == Some(new) {
        return;
    }
    // spec: S-0414 claim 22 (the type's current price for the old owner, no allied or rank multiplier, no flag exclusion)
    let value = current_price(w, id, old);
    let kill = w.rules().quirks.ownership_gain_counts_as_kill && !not_scored(w, id);
    if let Some(s) = stats_mut(w, new) {
        if kill {
            // spec: S-0414 claim 21 and quirks.ownership_gain_counts_as_kill (one kill for a not-scored-unmarked object, insignificant included; the losing house no loss)
            s.kills = s.kills.wrapping_add(1);
        }
        s.score = s.score.wrapping_add(value);
    }
}

/// A resource amount `a` (a single, held as a double) credited to a house
/// by unloading or conversion (S-0414 claim 22): *p* = trunc(*p* + 5 × *a*).
pub fn resource(w: &mut World, h: HouseId, a: F64) {
    let m = w.rules().mode;
    let Some(s) = stats_mut(w, h) else { return };
    // spec: S-0414 claim 22 (trunc(p + 5 × a), double multiplication and addition, S-0205's conversion toward zero)
    s.score = F64::from_i64(i64::from(s.score), m)
        .add(F64::from_i64(5, m).mul(a, m), m) // spec: S-0414 claim 22 (5 score points per credited resource unit)
        .to_i32_common();
}

/// The displayed score of a house with running score `p` (claim 23), with
/// `r` the house's draw, or none for a defeated house.
pub fn displayed(p: i32, r: Option<i32>) -> i32 {
    // spec: S-0414 claim 23 (b = max(p, 0); a defeated house b; an undefeated b + ⌊b/2⌋ + r, signed 32-bit wrap)
    let b = p.max(0);
    match r {
        None => b,
        Some(r) => b.wrapping_add(b / 2).wrapping_add(r), // spec: S-0414 claim 23 (b + ⌊b/2⌋ + r)
    }
}

/// Fixes the displayed scores once, after the simulation has stopped
/// (S-0414 claim 23; S-0202 claim 16): one ranged draw R(⌊b/2⌋, b) on the
/// scenario stream per undefeated listed house, in house order. Later
/// calls change nothing.
pub fn finish(w: &mut World, listed: &[HouseId]) {
    if listed.iter().any(|&h| of(w, h).shown.is_some()) {
        return;
    }
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut order = listed.to_vec();
    // spec: S-0414 claim 23 (in house order, not derived from displayed scores)
    order.sort();
    for h in order {
        let b = of(w, h).score.max(0); // spec: S-0414 claim 23 (b = max(p, 0))
        let r = if super::victory::defeated(w, h) {
            // spec: S-0414 claim 23 (defeated houses request no draw)
            None
        } else {
            // spec: S-0414 claim 23 and S-0202 claim 16 (an inclusive ranged draw from ⌊b/2⌋ through b, the stream continuing from play)
            Some(w.rng.ranged(b / 2, b, half))
        };
        let shown = displayed(of(w, h).score, r);
        if let Some(s) = stats_mut(w, h) {
            s.shown = Some(shown);
        }
    }
}
