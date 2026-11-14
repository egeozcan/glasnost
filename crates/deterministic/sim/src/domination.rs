//! The psychic domination superweapon (S-0612): its launch (claims 1 and
//! 2), the game-wide domination and its phases (claims 3 to 7), the sweep
//! with its blast ring, area damage and takeovers (claims 8 to 16), and the
//! fixed dominating house (claim 17).
//!
//! A domination is world state ([`DominationWorld`]): at most one at a
//! time for all houses. Its two animations play with the simulation's
//! stage clock (`crate::anim_clock`, S-0109) in the update order, their
//! playback draws taken from the scenario stream (S-0109 claim 28), because
//! their stages time the sweep (F1) and the final phase (F2). The domination
//! update runs once per tick in S-0203 row 3, before the weather storm's
//! own update ([`update`], called from `tick`). The lighting it starts and
//! ends is [`crate::lighting`] (S-0104).

use std::collections::{BTreeMap, BTreeSet};

use glasnost_map::CellXY;
use glasnost_num::F64;
use serde::{Deserialize, Serialize};

use crate::anim_clock::{AnimQuirks, Fields, Playback, Player};
use crate::combat::data::{CombatRules, WarheadId};
use crate::combat::effects::Effect;
use crate::rules::{AnimRules, Kind};
use crate::world::{HouseId, HouseRole, Layer, ObjectId, Pos, World};

/// F2's near-end threshold, stages before *N*.
pub const NEAR_END: i32 = 10; // spec: S-0612 F2 and Constants (near-end threshold k₂ ≥ N₂ − 10)
/// F2's final threshold, stages before *N*.
pub const FINAL: i32 = 1; // spec: S-0612 F2 and Constants (final threshold k₂ ≥ N₂ − 1)
/// The largest swept ring (claim 11).
pub const LARGEST_RING: i32 = 10; // spec: S-0612 claim 11 and Constants (largest swept ring 10)
/// The blast ring is removed at this update (claim 8).
pub const RING_UPDATES: i32 = 80; // spec: S-0612 claim 8 and Constants (removed at its 80th update, present for 79)
/// The double nearest to 0.01 (F1's *c*).
pub const PERCENT: F64 = F64(0x3F84_7AE1_47AE_147B); // spec: S-0612 F1 and Constants (c, the double nearest to 0.01)

/// The game-wide domination values (S-0612 Import mapping).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DominationRules {
    /// The lead-in animation type (claims 1 and 4).
    pub lead: Option<String>,
    /// The sweep animation type (claims 5 and 8).
    pub sweep: Option<String>,
    /// *p* of F1.
    pub percent: i32,
    /// *r* of claim 11.
    pub range: i32,
    /// The area damage amount (claim 9).
    pub damage: i32,
    /// The area damage warhead (claim 9).
    pub warhead: Option<WarheadId>,
    /// The activation sound (claim 1).
    pub sound: Option<String>,
}

impl Default for DominationRules {
    fn default() -> Self {
        Self {
            lead: None,
            sweep: None,
            percent: 50, // spec: S-0612 Import mapping and Constants (DominatorFireAtPercentage absent: 50)
            range: 2, // spec: S-0612 Import mapping and Constants (DominatorCaptureRange absent: 2)
            damage: 50, // spec: S-0612 Import mapping and Constants (DominatorDamage absent: 50)
            warhead: None,
            sound: None,
        }
    }
}

impl DominationRules {
    /// Resolves the values, warning about the bad-data cases of claims 2,
    /// 6 and 11 when some superweapon type is of the psychic domination
    /// kind (`dominator`).
    pub fn new(
        r: &glasnost_rules::schema::Ruleset,
        combat: &CombatRules,
        animations: &[AnimRules],
        dominator: bool,
        warnings: &mut Vec<String>,
    ) -> Self {
        let g = &r.sim.general.superweapons;
        let d = Self::default();
        let warhead = g.dominator_warhead.as_ref().and_then(|n| {
            combat
                .warheads
                .iter()
                .position(|h| h.name.eq_ignore_ascii_case(n))
                .and_then(|i| WarheadId::try_from(i).ok())
        });
        let out = Self {
            // spec: S-0612 Import mapping (DominatorFirstAnim, DominatorSecondAnim absent: none)
            lead: g.dominator_first_anim.clone(),
            sweep: g.dominator_second_anim.clone(),
            percent: g.dominator_fire_at_percentage.unwrap_or(d.percent),
            range: g.dominator_capture_range.unwrap_or(d.range),
            damage: g.dominator_damage.unwrap_or(d.damage),
            // spec: S-0612 Import mapping (DominatorWarhead absent: none; S-0303 owns what a missing warhead does)
            warhead,
            // spec: S-0612 Import mapping (PsychicDominatorActivateSound absent: none)
            sound: g.psychic_dominator_activate_sound.clone(),
        };
        if !dominator {
            return out;
        }
        let find = |n: &Option<String>| {
            n.as_ref()
                .and_then(|n| animations.iter().find(|a| a.name.eq_ignore_ascii_case(n)))
        };
        let (lead, sweep) = (find(&out.lead), find(&out.sweep));
        if lead.is_none() || sweep.is_none() {
            // spec: S-0612 claim 2 (our importer warns about a psychic domination superweapon whose rules leave either animation type unset)
            warnings.push("S-0612 claim 2: [General] DominatorFirstAnim or DominatorSecondAnim names no animation type; a psychic domination launch starts no domination".to_owned());
        }
        if let Some(a) = lead {
            let n = i64::from(a.frames.unwrap_or(0));
            // spec: S-0612 claim 6 (our importer warns about a fire-at percentage above 100 × (N − 1) / N for the lead-in type)
            if i64::from(out.percent) * n > 100 * (n - 1) {
                warnings.push(format!("S-0612 claim 6: DominatorFireAtPercentage {} is above 100 × (N − 1) / N for the lead-in animation {} (N = {n}); the sweep then follows the animation's end (a departure)", out.percent, a.name));
            }
        }
        if let Some(a) = sweep {
            let n = a.frames.unwrap_or(0);
            let end = play_fields(a, n).end;
            // spec: S-0612 claim 6 (our importer warns about a sweep animation type whose stages cannot reach N − 1)
            if a.reverse || end < n {
                warnings.push(format!("S-0612 claim 6: the sweep animation {} cannot reach stage N − 1 (N = {n}); the final phase then follows the animation's end (a departure)", a.name));
            }
        }
        if out.range < 0 {
            // spec: S-0612 claim 11 (a negative capture range sweeps no cell in our engine, and our importer warns; departure)
            warnings.push(format!(
                "S-0612 claim 11: [General] DominatorCaptureRange {} is negative; a domination sweeps no cell (a departure)",
                out.range
            ));
        }
        out
    }
}

/// The fields in play of type `a` with sprite count `n`, without the
/// rate (for the importer's reach test).
fn play_fields(a: &AnimRules, n: i32) -> Playback {
    let f = Fields {
        start: a.start,
        end: a.end,
        loop_end: a.loop_end,
        loop_start: a.loop_start,
        loop_count: a.loop_count,
        reverse: a.reverse,
        ping_pong: a.ping_pong,
        shadow: a.shadow,
        new_theater: a.new_theater,
        normalized: a.normalized,
        random_loop_delay: a.random_loop_delay,
        next: None,
    };
    // nocite: the importer's warning is a design aid and reads the default switches
    Playback::of_fields(&f, n, 1, (0, 0), &AnimQuirks::default())
}

/// The phase of a domination (S-0612 claims 4 and 5; S-0104 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Before the sweep. `first` is set once the first domination update
    /// after the launch has run (claim 4: it cannot sweep).
    LeadIn {
        /// The first domination update has run.
        first: bool,
    },
    /// After the sweep, before the final phase. `near` is set at F2's *t*₁.
    Swept {
        /// F2's near-end update *t*₁ has passed.
        near: bool,
    },
    /// The final phase (S-0104 Terms): the dominator no longer counts for
    /// the global choice, but is still in force.
    Final,
}

/// The domination in force (S-0612 Terms).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Domination {
    /// The dominating house, fixed at launch (claim 17).
    pub house: HouseId,
    /// The target cell.
    pub cell: CellXY,
    /// The superweapon type launched, by list position.
    pub sw: u32,
    /// The lead-in animation, while in play.
    pub lead: Option<ObjectId>,
    /// The sweep animation, while in play.
    pub sweep: Option<ObjectId>,
    /// The phase.
    pub phase: Phase,
    /// The launch tick.
    pub launched: i32,
}

/// One of a domination's animations in play (S-0109 playback).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomAnim {
    /// The current type by registration index (`None` after a follow-up
    /// switch to a name no type is registered under).
    pub anim: Option<u32>,
    /// Its stage clock.
    pub player: Player,
    /// Its world point.
    pub pos: Pos,
}

/// A blast ring of claim 8: presentation only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlastRing {
    /// Where.
    pub pos: Pos,
    /// Its updates so far.
    pub updates: i32,
}

/// The domination state of the game.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DominationWorld {
    /// The domination in force, if any (claims 3 and 7: at most one).
    pub current: Option<Domination>,
    /// The domination animations in play, by number in the update order.
    pub anims: BTreeMap<ObjectId, DomAnim>,
    /// Blast rings present (S-0203 row 15).
    pub rings: Vec<BlastRing>,
}

/// What happened to a domination (presentation and counting).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomEvent {
    /// A domination started (claim 1).
    Started,
    /// The sweep happened (claim 8): the blast ring appeared.
    Sweep,
    /// The sweep's area damage (claim 9) hurt `damaged` objects that
    /// survived it and destroyed `killed` (presentation and tallies only).
    Damage {
        /// Objects whose health fell and that survived.
        damaged: u32,
        /// Objects the damage destroyed.
        killed: u32,
    },
    /// An object was taken over (claim 14).
    Taken(ObjectId),
    /// The final phase started (claim 5).
    Final,
    /// The domination is over (claim 5).
    Over,
    /// A blast ring was removed at its 80th update (claim 8).
    RingEnded,
}

/// Whether a domination is in force (claim 3; S-0512 claim 8 reads it).
pub fn in_force(w: &World) -> bool {
    w.domination.current.is_some()
}

/// Whether a domination is in force in an active phase other than its
/// final phase (S-0104 claim 2's global choice).
pub fn active(w: &World) -> bool {
    w.domination
        .current
        .as_ref()
        .is_some_and(|d| d.phase != Phase::Final)
}

fn push(w: &mut World, event: DomEvent, house: Option<HouseId>, pos: Pos) {
    w.effects.push(Effect::Domination { event, house, pos });
}

/// The psychic domination launch (claims 1 and 2) by house `h` with type
/// `i` at target cell `cell`.
pub fn launch(w: &mut World, h: HouseId, i: u32, cell: CellXY) {
    let rules = w.rules_arc();
    let dr = &rules.superweapons.domination;
    // spec: S-0612 claim 1 (the centre of c at the terrain surface; no bridge height)
    let centre = crate::superweapons::cell_point(w, cell, false);
    // spec: S-0612 claim 1 (a radar event of the same kind as the invulnerability launch's at c)
    crate::superweapons::event(
        w,
        crate::superweapons::SwEvent::Radar,
        Some(h),
        Some(i),
        Some(centre),
    );
    let types = (
        dr.lead.as_deref().and_then(|n| anim_index(w, n)),
        dr.sweep.as_deref().and_then(|n| anim_index(w, n)),
    );
    if let (Some(lead), Some(_)) = types {
        let t = w.tick;
        w.domination.current = Some(Domination {
            house: h,
            cell,
            sw: i,
            lead: None,
            sweep: None,
            phase: Phase::LeadIn { first: false },
            launched: t,
        });
        // spec: S-0612 claim 1 (the lead-in animation starts at the centre of c at the terrain surface, playing once)
        let id = start_anim(w, lead, centre);
        if let Some(d) = w.domination.current.as_mut() {
            d.lead = id;
        }
        push(w, DomEvent::Started, Some(h), centre);
        // spec: S-0612 claim 1 and S-0104 claim 5 (a lighting re-evaluation; the first fade step on the next tick)
        crate::lighting::reevaluate_forced(w);
    }
    // spec: S-0612 claim 2 (a type unset: every other effect of claim 1 and no domination, no re-evaluation)
    // spec: S-0612 claim 1 and S-0701 claim 28 (the activation announcement for every player, unless muted)
    crate::superweapons::event(
        w,
        crate::superweapons::SwEvent::Launch,
        Some(h),
        Some(i),
        None,
    );
    // spec: S-0612 claim 1 (the activation sound at the centre of c, heard near it; the launching player's aiming ends and its ready announcement is silenced, S-0513 claim 5)
    crate::superweapons::event(
        w,
        crate::superweapons::SwEvent::DominatorSound,
        Some(h),
        Some(i),
        Some(centre),
    );
}

/// The registration index of animation type `name`.
fn anim_index(w: &World, name: &str) -> Option<u32> {
    w.rules().anim_index(name.as_bytes())
}

/// Starts a domination animation of type `anim` at `pos` in the update
/// order (S-0109 claims 1, 5 and 29).
fn start_anim(w: &mut World, anim: u32, pos: Pos) -> Option<ObjectId> {
    let rules = w.rules_arc();
    let a = rules.animations.get(anim as usize)?;
    // spec: S-0109 claim 29 and S-0202 claim 10 (the creation and start-event draws at the creation)
    let drawn = crate::load::anim_created(w, &a.name, Some((pos.x, pos.y, pos.z)), true);
    let play = crate::mutation::playback(w, a);
    let id = w.new_id();
    let aq = AnimQuirks::from(&w.rules().quirks);
    let t = w.tick;
    let mut env = crate::mutation::SimEnv {
        w: &mut *w,
        pos,
        anim: Some(anim),
    };
    // spec: S-0109 claims 1 and 7 (no loop delay, repeat factor 1: played once)
    let player = Player::new(a.name.as_bytes(), play, t, drawn, &mut env, &aq);
    w.domination.anims.insert(
        id,
        DomAnim {
            anim: Some(anim),
            player,
            pos,
        },
    );
    w.effects.push(Effect::Anim {
        name: a.name.clone(),
        pos,
        delay: drawn,
        reverse: false,
        z_adjust: None,
        id: None,
    });
    // spec: S-0109 claim 5 (an animation updates in its own turn; created outside the turn pass, its first turn comes in the next turn pass)
    w.update.enter(id);
    Some(id)
}

/// A domination animation's turn (S-0109 claims 5, 15 and 16).
pub fn anim_turn(w: &mut World, id: ObjectId) {
    let Some(mut a) = w.domination.anims.remove(&id) else {
        return;
    };
    let aq = AnimQuirks::from(&w.rules().quirks);
    let t = w.tick;
    let mut env = crate::mutation::SimEnv {
        w: &mut *w,
        pos: a.pos,
        anim: a.anim,
    };
    let playing = a.player.update(t, &mut env, &aq);
    a.anim = env.anim;
    if playing {
        w.domination.anims.insert(id, a);
        return;
    }
    let skip = w.rules().quirks.update_order_removal_skip;
    // spec: S-0109 claim 16 (the last pass ends with no follow-up type: removed); S-0203 claim 10 (a removal from the update order)
    w.update.leave(id, skip);
}

/// The stage *k* and sprite count *N* of a domination animation in play,
/// `None` once it has been removed.
fn stage_of(w: &World, id: Option<ObjectId>) -> Option<(i32, i32)> {
    let a = w.domination.anims.get(&id?)?;
    let n = a
        .anim
        .and_then(|i| w.rules().animations.get(i as usize))
        .and_then(|r| r.frames)
        .unwrap_or(0);
    Some((a.player.stage(), n))
}

/// F1: whether the lead-in's stage `k` of `n` meets fire-at percentage
/// `p`, compared in double precision as two separately rounded values.
pub fn f1(p: i32, k: i32, n: i32, mode: glasnost_num::Rounding) -> bool {
    // spec: S-0612 F1 (sweep ⇔ R(p × c) ≤ R(k / N), each rounded in the mode of quirks.chop_rounding; p, k and N convert exactly)
    let lhs = F64::from_i64(i64::from(p), mode).mul(PERCENT, mode);
    let rhs = F64::from_i64(i64::from(k), mode).div(F64::from_i64(i64::from(n), mode), mode);
    // A sprite of no frames divides by zero: k > 0 gives infinity (sweep), k = 0 a NaN (no sweep), as in double arithmetic.
    lhs.le(rhs)
}

/// The domination update (S-0612 claims 4 to 7), in S-0203 row 3 before
/// the storm's own update.
pub fn update(w: &mut World) {
    let Some(mut d) = w.domination.current.clone() else {
        return;
    };
    let mode = w.rules().mode;
    match d.phase {
        Phase::LeadIn { first: false } => {
            // spec: S-0612 claim 4 (the first domination update after the launch cannot sweep)
            d.phase = Phase::LeadIn { first: true };
            w.domination.current = Some(d);
        }
        Phase::LeadIn { first: true } => {
            let p = w.rules().superweapons.domination.percent;
            let fire = match stage_of(w, d.lead) {
                // spec: S-0612 claim 4 and F1 (the lead-in's stage as it stands at this update, before the turn pass)
                Some((k, n)) => f1(p, k, n, mode),
                // spec: S-0612 claim 6 (the lead-in removed before the sweep: the sweep happens at the next domination update; a departure)
                None => true,
            };
            if fire {
                sweep(w, &mut d);
            }
            w.domination.current = Some(d);
        }
        Phase::Swept { near } => {
            let enter_final = match stage_of(w, d.sweep) {
                // spec: S-0612 F2 (the final phase starts at the first update after t₁ at which k₂ ≥ N₂ − 1)
                Some((k, n)) if near => k >= n - FINAL,
                Some((k, n)) => {
                    // spec: S-0612 F2 (t₁: the first update after the sweep at which k₂ ≥ N₂ − 10)
                    if k >= n - NEAR_END {
                        d.phase = Phase::Swept { near: true };
                    }
                    false
                }
                // spec: S-0612 claim 6 (the sweep animation removed before F2's final threshold: the final phase starts at the next domination update; a departure)
                None => true,
            };
            let pos = crate::superweapons::cell_point(w, d.cell, false);
            let house = d.house;
            if enter_final {
                d.phase = Phase::Final;
            }
            w.domination.current = Some(d);
            if enter_final {
                push(w, DomEvent::Final, Some(house), pos);
                // spec: S-0612 claim 5 and S-0104 claim 5 (the final phase starts with a lighting re-evaluation; this tick's fade already stepped towards the old target)
                crate::lighting::reevaluate(w);
            }
        }
        Phase::Final => {
            let l = &w.lighting;
            // spec: S-0612 claims 5 and 7 and S-0104 claim 5 (over at a later domination update at which the current ambient equals the target ambient; no re-evaluation)
            if l.current == l.target {
                let pos = crate::superweapons::cell_point(w, d.cell, false);
                w.domination.current = None;
                push(w, DomEvent::Over, Some(d.house), pos);
            }
        }
    }
}

/// Whether an object is dying while still in play (S-0612 claim 13;
/// S-0203 claim 6): an infantry unit in its death sequence, a sinking
/// vessel.
fn dying(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.combat.killed) || crate::naval::sinking(w, id)
}

/// The objects the sweep examines in cell `c`, layer `l` (claim 12).
pub fn examined(w: &World, c: CellXY, l: Layer) -> Vec<ObjectId> {
    let list = crate::superweapons::listed(w, c, l);
    let kind = |id: ObjectId| w.obj(id).map(|o| o.kind);
    if !w.rules().quirks.dominator_sweep_from_nearest {
        // spec: S-0612 claim 12 and Edge cases (quirks.dominator_sweep_from_nearest off: every listed infantry, vehicle and aircraft, in listing order)
        return list
            .into_iter()
            .filter(|&id| {
                matches!(
                    kind(id),
                    Some(Kind::Infantry | Kind::Vehicle | Kind::Aircraft)
                )
            })
            .collect();
    }
    let combatant = |id: ObjectId| {
        matches!(
            kind(id),
            Some(Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building)
        )
    };
    let mut best: Option<(u64, usize)> = None;
    for (i, &id) in list.iter().enumerate() {
        if !combatant(id) {
            continue;
        }
        // spec: S-0612 claim 12 (a building's position is the centre of its foundation)
        let p = w.reference_point(id);
        // spec: S-0612 claim 12 and S-0207 F8 (residues from the top corner of the cell holding the position, with the table square root, compared as integers)
        let dist = crate::cloak::machine::sensor_candidate_distance(w, p.x, p.y);
        // spec: S-0612 claim 12 (the smallest distance; the one listed first wins a tie, S-0209 claim 13)
        if best.is_none_or(|(b, _)| dist < b) {
            best = Some((dist, i));
        }
    }
    let Some((_, start)) = best else {
        return Vec::new();
    };
    // spec: S-0612 claim 12 (from the start object forward in listing order, stopping before the first listed object of none of the four kinds)
    list[start..]
        .iter()
        .copied()
        .take_while(|&id| combatant(id))
        .collect()
}

/// The swept cells and layers in visiting order (claim 11).
pub fn swept(w: &World, centre: CellXY) -> Vec<(CellXY, Layer)> {
    let range = w.rules().superweapons.domination.range;
    let q = w.rules().quirks;
    let mut out = Vec::new();
    if range < 0 {
        // spec: S-0612 claim 11 (a negative capture range sweeps no cell; a departure)
        return out;
    }
    // spec: S-0612 claim 11 and F3 (the cells of S-0303 T1 with ring metric at most min(r, 10), in T1's scan order)
    for ring in 0..=range.min(LARGEST_RING) {
        for (dx, dy) in crate::combat::area::ring(ring, q.spread_ring_eleven) {
            let c = CellXY::new(centre.x + dx, centre.y + dy);
            // spec: S-0612 claim 11 (only the ground layer of each swept cell; quirks.dominator_ground_layer_only)
            out.push((c, Layer::Ground));
            if !q.dominator_ground_layer_only {
                // spec: S-0612 claim 11 and Edge cases (quirk off: the deck layer of a swept cell after its ground layer, in the same way)
                out.push((c, Layer::Deck));
            }
        }
    }
    out
}

/// The sweep (claims 8 to 16).
fn sweep(w: &mut World, d: &mut Domination) {
    let rules = w.rules_arc();
    let dr = &rules.superweapons.domination;
    let q = rules.quirks;
    let centre = crate::superweapons::cell_point(w, d.cell, false);
    // spec: S-0612 claim 8 (a blast ring at the centre of the target cell, before the damage; presentation only, no draws)
    w.domination.rings.push(BlastRing {
        pos: centre,
        updates: 0,
    });
    push(w, DomEvent::Sweep, Some(d.house), centre);
    if let Some(a) = dr.sweep.as_deref().and_then(|n| anim_index(w, n)) {
        // spec: S-0612 claim 8 (the sweep animation starts there, playing once)
        d.sweep = start_anim(w, a, centre);
    }
    d.phase = Phase::Swept { near: false };
    // The objects already dying before the damage (claim 10: one destroyed by the damage is not taken over).
    let before: BTreeSet<ObjectId> = w
        .objects
        .keys()
        .copied()
        .filter(|&id| dying(w, id))
        .collect();
    // The health of every object in play before the damage, for the
    // damage event's counts (presentation only).
    let health: Vec<(ObjectId, i32)> = w
        .objects
        .values()
        .filter(|o| o.on_map)
        .map(|o| (o.id, o.health))
        .collect();
    let house = if q.dominator_damage_storm_house {
        // spec: S-0612 claim 9 (the attacking house is the current storm owner, sampled before this tick's storm update; quirks.dominator_damage_storm_house)
        w.sw.storm.owner
    } else {
        // spec: S-0612 claim 9 and Edge cases (quirk off: the dominating house)
        Some(d.house)
    };
    // spec: S-0612 claim 9 (area damage at the centre of the target cell at its terrain surface, the imported amount and warhead, no source object, the reduction of resources on)
    crate::combat::area::detonate(
        w,
        crate::combat::area::Detonation {
            point: centre,
            damage: dr.damage,
            warhead: dr.warhead,
            source: None,
            house,
            resources: true,
        },
    );
    let (mut damaged, mut killed) = (0u32, 0u32);
    for (id, h) in health {
        match w.obj(id) {
            Some(o) if o.on_map && !(dying(w, id) && !before.contains(&id)) => {
                damaged += u32::from(o.health < h);
            }
            _ => killed += 1,
        }
    }
    if damaged + killed > 0 {
        push(
            w,
            DomEvent::Damage { damaged, killed },
            Some(d.house),
            centre,
        );
    }
    // spec: S-0612 claim 10 (every effect of the damage is complete before the first takeover)
    let mut taken = Vec::new();
    for (c, l) in swept(w, d.cell) {
        for id in examined(w, c, l) {
            let Some(o) = w.obj(id) else { continue };
            if !o.on_map {
                continue;
            }
            let owner = o.owner;
            if dying(w, id) {
                if !before.contains(&id) {
                    // spec: S-0612 claim 10 (an object destroyed by the damage is not taken over)
                    continue;
                }
                if !q.dominator_takes_dying {
                    // spec: S-0612 claim 13 and Edge cases (quirks.dominator_takes_dying off: a dying object is skipped)
                    continue;
                }
            }
            if owner == Some(d.house) && !q.dominator_marks_own_objects {
                // spec: S-0612 claim 14 and Edge cases (quirks.dominator_marks_own_objects off: an object of the dominating house is left unchanged)
                continue;
            }
            // spec: S-0612 claims 13 and 14 (the conditions of S-0601 claim 13 and no others; release, owner change, permanent control with one ring)
            if crate::mind::dominate(w, id, d.house) {
                taken.push(id);
                let pos = w.obj(id).map_or(centre, |o| o.pos);
                push(w, DomEvent::Taken(id), Some(d.house), pos);
            }
        }
    }
    let role = w.houses.get(usize::from(d.house.0)).map(|x| x.role);
    if matches!(role, Some(HouseRole::Neutral | HouseRole::Special)) {
        // spec: S-0612 claim 16 (after all takeovers, hunt as the next behaviour only for the neutral or a special house; an AI house follows the human rule, a departure)
        for id in taken {
            if let Some(o) = w.obj_mut(id) {
                o.beh.queue(Some(crate::rules::Behaviour::Hunt));
            }
        }
    }
}

/// The blast rings' update (S-0203 row 15; S-0612 claim 8): each is
/// removed at its 80th update.
pub fn rings(w: &mut World) {
    let mut ended = Vec::new();
    for r in &mut w.domination.rings {
        r.updates += 1;
        if r.updates >= RING_UPDATES {
            ended.push(r.pos);
        }
    }
    // spec: S-0612 claim 8 (removed at its 80th update, so present for 79 of them)
    w.domination.rings.retain(|r| r.updates < RING_UPDATES);
    for pos in ended {
        push(w, DomEvent::RingEnded, None, pos);
    }
}

/// What the sweep of a domination launched now by `h` at `cell` would
/// take over, as things stand (for the own AI's choice): the examined
/// objects that pass S-0601 claim 13's tests, before any damage. The
/// takeover itself waits for the sweep (claim 4).
pub fn preview(w: &World, h: HouseId, cell: CellXY) -> Vec<ObjectId> {
    let q = w.rules().quirks;
    let mut out = Vec::new();
    for (c, l) in swept(w, cell) {
        for id in examined(w, c, l) {
            let Some(o) = w.obj(id) else { continue };
            if !o.on_map || (dying(w, id) && !q.dominator_takes_dying) {
                continue;
            }
            if o.owner == Some(h) && !q.dominator_marks_own_objects {
                continue;
            }
            if crate::mind::dominable(w, id) {
                out.push(id);
            }
        }
    }
    out
}
