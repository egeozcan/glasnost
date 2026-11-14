//! Mutation (S-0611): an infantry unit killed by a mutating warhead is
//! replaced by a new infantry unit. This module holds the conversion test
//! and the conversion (claims 6 and 7), the infantry-making animations
//! that create the new unit at their end (claims 8 to 12, for every
//! infantry-making animation, the conversion animation included) and the
//! genetic mutation superweapon's launch (claims 2 to 4).
//!
//! Which deaths reach the conversion (claim 1) is S-0107 claim 17's row 7,
//! decided in `combat::death`. An infantry-making animation plays with the
//! simulation's stage clock (`crate::anim_clock`, S-0109) in the update
//! order, its playback draws taken from the scenario stream (S-0109
//! claim 28).

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_num::geom::Facing;
use serde::{Deserialize, Serialize};

use crate::anim_clock::{AnimQuirks, Env, Fields, Playback, Player};
use crate::cells::Occupancy;
use crate::combat::damage::Hit;
use crate::combat::data::{CombatRules, WarheadId};
use crate::combat::effects::Effect;
use glasnost_rules::sim::LocomotionClass;

use crate::rules::{AnimRules, Kind};
use crate::world::{HouseId, HouseRole, Layer, ObjectId, Pos, World};

/// The mutating death kind (claim 1; S-0107 Terms).
pub const MUTATING_DEATH_KIND: i32 = 9; // spec: S-0611 Constants (mutating death kind 9)
/// The explosion-mode amount, hit points (claim 3).
pub const EXPLOSION_AMOUNT: i32 = 10_000; // spec: S-0611 claim 3 and Constants (explosion-mode amount 10 000 hit points)
/// The launch animation's lift above the mutation point, leptons (claim 2,
/// F1).
pub const LAUNCH_LIFT: i32 = 5; // spec: S-0611 F1 and Constants (launch animation lift 5 leptons)
/// The facing of a made unit (F3: map south-east).
pub const MADE_FACING: Facing = 24_576; // spec: S-0611 F3 and Constants (made-unit facing 24 576 = 96 × 256)

/// The game-wide mutation values (S-0611 Import mapping).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MutationRules {
    /// Explosion mode (claim 3) when on, cell mode (claim 4) when off.
    pub explosion: bool,
    /// The explosion warhead (claim 3).
    pub explosion_warhead: Option<WarheadId>,
    /// The cell warhead (claim 4).
    pub cell_warhead: Option<WarheadId>,
    /// The conversion animation type, by registration index (claim 8).
    pub anim: Option<u32>,
    /// The animation-to-infantry list (F4), each entry an infantry
    /// registration index (`None` for a name no infantry type has).
    pub made: Vec<Option<u32>>,
    /// The launch animation (claim 2).
    pub ion_blast: Option<String>,
    /// The activation sound (claim 2).
    pub sound: Option<String>,
}

impl MutationRules {
    /// Resolves the values, warning about the bad-data cases of the
    /// Import mapping.
    pub fn new(
        r: &glasnost_rules::schema::Ruleset,
        names: &BTreeMap<(Kind, String), u32>,
        combat: &CombatRules,
        genetic: bool,
        warnings: &mut Vec<String>,
    ) -> Self {
        let g = &r.sim.general.superweapons;
        let warhead = |n: &Option<String>| {
            n.as_ref().and_then(|n| {
                combat
                    .warheads
                    .iter()
                    .position(|h| h.name.eq_ignore_ascii_case(n))
                    .and_then(|i| WarheadId::try_from(i).ok())
            })
        };
        let anims = r.registries.get("animations").cloned().unwrap_or_default();
        let anim_index = |n: &str| {
            anims
                .iter()
                .position(|x| x.eq_ignore_ascii_case(n))
                .and_then(|i| u32::try_from(i).ok())
        };
        // spec: S-0611 Import mapping (MutateExplosion absent: no, cell mode)
        let explosion = g.mutate_explosion.unwrap_or(false);
        let explosion_warhead = warhead(&g.mutate_explosion_warhead);
        let cell_warhead = warhead(&g.mutate_warhead);
        if genetic {
            let (key, wh) = if explosion {
                ("MutateExplosionWarhead", explosion_warhead)
            } else {
                ("MutateWarhead", cell_warhead)
            };
            if wh.is_none() {
                // spec: S-0611 Import mapping (a warhead reference left empty: our engine delivers no hits for that mode and the importer warns once)
                warnings.push(format!("S-0611 Import mapping: [SpecialWeapons] {key} names no warhead; a genetic mutation launch delivers no hits"));
            }
        }
        // spec: S-0611 Import mapping (InfantryMutate absent: none)
        let anim = g.infantry_mutate.as_deref().and_then(anim_index);
        let mutating = combat
            .warheads
            .iter()
            .any(|h| h.inf_death == MUTATING_DEATH_KIND);
        if anim.is_none() && (genetic || mutating) {
            // spec: S-0611 Edge cases (absent conversion animation: the victim is removed with no replacement, with an importer warning)
            warnings.push("S-0611 Import mapping: [General] InfantryMutate names no animation type; a converted victim is removed with no replacement".to_owned());
        }
        // spec: S-0611 Import mapping (AnimToInfantry absent: empty, no animation makes infantry)
        let list = g.anim_to_infantry.clone().unwrap_or_default();
        let made: Vec<Option<u32>> = list
            .iter()
            .map(|n| names.get(&(Kind::Infantry, n.clone())).copied())
            .collect();
        for (n, t) in list.iter().zip(&made) {
            if t.is_none() {
                warnings.push(format!("S-0611 Import mapping: [General] AnimToInfantry names {n}, which is no registered infantry type; that entry makes nothing"));
            }
        }
        for n in &anims {
            let i = r
                .sim
                .animations
                .get(n)
                .and_then(|a| a.make_infantry)
                .unwrap_or(-1);
            // spec: S-0611 F4 (i ≥ n, or negative other than −1: nothing is made and the importer warns)
            if i != -1 && !(0..made.len() as i64).contains(&i64::from(i)) {
                warnings.push(format!("S-0611 F4: animation {n} has the make-infantry index {i}, outside the animation-to-infantry list of {} entries; it makes nothing", made.len()));
            }
        }
        Self {
            explosion,
            explosion_warhead,
            cell_warhead,
            anim,
            made,
            ion_blast: g.ion_blast.clone(),
            sound: g.genetic_mutator_activate_sound.clone(),
        }
    }
}

/// An infantry-making animation in play (claims 8 to 12).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Maker {
    /// The current type by registration index; `None` after a follow-up
    /// switch to a name no type is registered under.
    pub anim: Option<u32>,
    /// Its stage clock (S-0109).
    pub player: Player,
    /// Its world point.
    pub pos: Pos,
    /// Its surface (claim 8: the occupancy record of its surface).
    pub layer: Layer,
    /// Its owner (claims 8 and 9).
    pub owner: Option<HouseId>,
    /// The standing position it holds: its cell and position.
    pub spot: (CellXY, u8),
    /// A conversion animation (claim 8), not another infantry-making one.
    pub conversion: bool,
}

/// A refused victim's freed standing position (claim 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Freed {
    /// The cell.
    pub cell: CellXY,
    /// The layer.
    pub layer: Layer,
    /// The standing position.
    pub position: u8,
}

/// The mutation state of the game.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationWorld {
    /// Infantry-making animations in play, by number in the update order.
    pub makers: BTreeMap<ObjectId, Maker>,
    /// Refused victims still playing die-2, with the position their removal
    /// frees again (`quirks.refused_mutation_frees_position`).
    pub refused: BTreeMap<ObjectId, Freed>,
}

/// What became of an infantry-making animation (presentation and
/// counting).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MakerEvent {
    /// It was created.
    Made,
    /// Its type changed at a follow-up switch.
    Switched,
    /// Its owner changed (claim 9's fallback): its colour follows.
    Owner,
    /// A placement was refused; it stays (claim 11).
    Refused,
    /// It ended and was removed, with the unit it created, if any.
    Ended {
        /// The infantry unit created.
        unit: Option<ObjectId>,
        /// It was a conversion animation (claim 8).
        conversion: bool,
    },
}

/// The fields in play of animation type `a` (S-0109 F2, claim 2).
pub fn playback(w: &World, a: &AnimRules) -> Playback {
    let q = w.rules().quirks;
    let delay = match a.rate {
        // spec: S-0109 claim 2, F1 (no rate field: d = 1)
        None | Some(-1) => 1,
        Some(r) => crate::rules::anim_delay(r, q.anim_fast_rate_freezes),
    };
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
        next: a.next.as_ref().map(|n| n.as_bytes().to_vec()),
    };
    // spec: S-0109 claim 28 (a spriteless type plays with N = 0: absent end fields are 0)
    let n = a.frames.unwrap_or(0);
    Playback::of_fields(
        &f,
        n,
        delay,
        a.random_delay,
        &AnimQuirks::from(&w.rules().quirks),
    )
}

/// The registration index of animation type `name`.
fn anim_index(w: &World, name: &[u8]) -> Option<u32> {
    w.rules().anim_index(name)
}

/// Whether an animation of type `anim` is infantry-making at some type of
/// its follow-up chain (S-0611 Terms; Edge cases: only the last type's
/// index counts, but the simulation plays the whole chain).
pub fn makes_infantry(w: &World, anim: u32) -> bool {
    let r = w.rules();
    let mut cur = Some(anim);
    // A chain longer than the type list repeats a type: it never ends.
    for _ in 0..=r.animations.len() {
        let Some(a) = cur.and_then(|i| r.animations.get(i as usize)) else {
            return false;
        };
        if a.make_infantry != -1 {
            return true;
        }
        if a.next.is_none() {
            return false;
        }
        cur = a.next_index;
    }
    false
}

/// The stage clock's surroundings in the simulation: follow-up types from
/// the rules, draws from the scenario stream (S-0109 claim 28), the middle
/// and start events of the current type.
pub(crate) struct SimEnv<'a> {
    pub(crate) w: &'a mut World,
    pub(crate) pos: Pos,
    pub(crate) anim: Option<u32>,
}

impl Env for SimEnv<'_> {
    fn follow_up(&mut self, name: &[u8]) -> Playback {
        self.anim = anim_index(self.w, name);
        let rules = self.w.rules_arc();
        match self.anim.and_then(|i| rules.animations.get(i as usize)) {
            Some(a) => playback(self.w, a),
            // spec: S-0109 Edge cases (a name that names no type: absent-key defaults with no sprite)
            None => Playback::missing(),
        }
    }

    fn draw(&mut self, lo: i32, hi: i32) -> i32 {
        let half = self.w.rules().quirks.ranged_draw_half_span;
        // spec: S-0109 claim 28 (playback draws come from the scenario stream)
        self.w.rng.ranged(lo, hi, half)
    }

    fn speed(&self) -> u8 {
        self.w.game_speed
    }

    fn normalise(&self) -> bool {
        self.w.rules().quirks.speed_normalised_delays
    }

    fn advanced(&mut self, p: &Playback, k: i32) {
        let rules = self.w.rules_arc();
        let Some(a) = self.anim.and_then(|i| rules.animations.get(i as usize)) else {
            return;
        };
        // spec: S-0109 claim 14 (the middle event on every advance after which a + k = μ, μ ≠ 0; not for a bouncing type)
        if !a.bouncing && a.mu.is_some_and(|m| m != 0 && p.start.wrapping_add(k) == m) {
            let at = Some((self.pos.x, self.pos.y, self.pos.z));
            crate::load::middle_event(self.w, a, at);
        }
    }

    fn switched(&mut self, _name: &[u8]) {
        let rules = self.w.rules_arc();
        if let Some(a) = self.anim.and_then(|i| rules.animations.get(i as usize)) {
            // spec: S-0109 claims 13, 15 and 28 (the new type's start event after its delay choice)
            let at = Some((self.pos.x, self.pos.y, self.pos.z));
            crate::load::start_event(self.w, a, at, true);
        }
    }
}

/// The surface of a point: the deck when its cell carries a high bridge
/// and the point is above the cell's terrain height (claim 10).
fn surface_of(w: &World, p: Pos) -> Layer {
    if w.high_bridge(p.cell()) && p.z > w.surface_at(p.x, p.y) {
        Layer::Deck
    } else {
        Layer::Ground
    }
}

/// The standing position a point is in (S-0208 claim 2).
fn spot_of(w: &World, p: Pos) -> (CellXY, u8) {
    let top = w.rules().quirks.top_position_counts_as_centre;
    // spec: S-0204 claim 2 (the position within its cell, 0 … 255 per axis)
    let (u, v) = (p.x.rem_euclid(256), p.y.rem_euclid(256));
    (p.cell(), crate::walk::position_class(u, v, top))
}

/// Creates an animation of type `anim` at `pos` (S-0109): an
/// infantry-making one is played by the simulation in the update order
/// (claims 8 and 9), any other is an ordinary animation. Returns the
/// infantry-making animation's number.
pub fn start(
    w: &mut World,
    anim: u32,
    pos: Pos,
    layer: Layer,
    owner: Option<HouseId>,
    conversion: bool,
) -> Option<ObjectId> {
    let rules = w.rules_arc();
    let a = rules.animations.get(anim as usize)?;
    if !makes_infantry(w, anim) {
        // spec: S-0611 Edge cases (a conversion animation whose type makes no infantry plays and leaves nothing)
        crate::combat::effects::anim(w, &a.name, pos);
        return None;
    }
    // spec: S-0109 claim 29 and S-0202 claim 10 (the creation and start-event draws at the creation)
    let drawn = crate::load::anim_created(w, &a.name, Some((pos.x, pos.y, pos.z)), true);
    let play = playback(w, a);
    let id = w.new_id();
    let aq = AnimQuirks::from(&w.rules().quirks);
    let t = w.tick;
    let mut env = SimEnv {
        w: &mut *w,
        pos,
        anim: Some(anim),
    };
    let player = Player::new(a.name.as_bytes(), play, t, drawn, &mut env, &aq);
    let spot = spot_of(w, pos);
    w.mutation.makers.insert(
        id,
        Maker {
            anim: Some(anim),
            player: player.clone(),
            pos,
            layer,
            owner,
            spot,
            conversion,
        },
    );
    w.effects.push(Effect::Maker {
        id,
        name: a.name.clone(),
        pos,
        owner,
        frame: player.frame(),
        event: MakerEvent::Made,
    });
    // spec: S-0109 claim 5 (an animation updates in its own turn; created during the turn pass, its first turn comes after every object already in the order)
    w.update.enter(id);
    Some(id)
}

/// The occupancy records a victim marks: its cell and, while walking, its
/// target's cell, in its layer.
fn held_records(w: &World, id: ObjectId) -> Vec<(CellXY, Layer)> {
    let Some(o) = w.obj(id) else {
        return Vec::new();
    };
    let mut out = vec![(o.pos.cell(), o.layer)];
    if let Some(t) = o.walk.as_ref().and_then(|s| s.target) {
        out.push((t.cell(), o.layer));
    }
    out.dedup();
    out
}

/// The standing position a victim's own mark holds, else the position of
/// its point.
fn victim_spot(w: &World, id: ObjectId, records: &[(CellXY, Layer, Occupancy)]) -> Freed {
    for &(cell, layer, rec) in records {
        for (p, m) in rec.marks.iter().enumerate() {
            if matches!(m, Some(crate::cells::Mark { by: crate::cells::Marker::Infantry(who, _), .. }) if *who == id)
            {
                return Freed {
                    cell,
                    layer,
                    position: p as u8,
                };
            }
        }
    }
    let o = w.obj(id);
    let pos = o.map_or(Pos::default(), |o| o.pos);
    let (cell, position) = spot_of(w, pos);
    Freed {
        cell,
        layer: o.map_or(Layer::Ground, |o| o.layer),
        position,
    }
}

/// Whether a building is listed in cell `c`'s ground layer (claim 6).
fn building_listed(w: &World, c: CellXY) -> bool {
    w.cells
        .list(c, Layer::Ground)
        .iter()
        .any(|&x| w.obj(x).is_some_and(|o| o.kind == Kind::Building))
}

/// The conversion of a killed infantry unit `id` whose death reached
/// S-0107 claim 17's row 7 (claims 6 to 8). Returns whether it converted
/// (the victim has left play, or begun its type's crash); `false` when the
/// test refused, and the caller makes row 7's die-2 request.
pub fn convert(w: &mut World, id: ObjectId, hit: &Hit) -> bool {
    let Some(o) = w.obj(id).cloned() else {
        return false;
    };
    let rules = w.rules_arc();
    let q = rules.quirks;
    let oq = crate::objects::occ_quirks(w);
    let c = o.pos.cell();
    let deck = o.layer == Layer::Deck;
    // spec: S-0611 claim 6 (the test looks at the victim's cell with the victim's own entry in the occupancy record already cleared)
    let records: Vec<(CellXY, Layer, Occupancy)> = held_records(w, id)
        .into_iter()
        .filter_map(|(cc, l)| w.cells.occ(cc, l).map(|r| (cc, l, *r)))
        .collect();
    let spot = victim_spot(w, id, &records);
    for &(cc, l, _) in &records {
        w.cells.clear_infantry(cc, l, id, oq);
    }
    let land = w.cells.cell(c).map_or(0, |x| x.land);
    // spec: S-0611 claim 6 (the foot class's land multiplier compared exactly with 0, whatever the victim's own class; a negative value is not 0)
    let zero = rules
        .multiplier(land, LocomotionClass::Foot)
        .to_f64()
        .eq_num(F64::ZERO);
    let refused = if zero && !deck {
        // spec: S-0611 claim 6 (row 1: refused, no draw)
        true
    } else {
        let layer = if deck && !q.mutation_test_ground_record {
            // spec: S-0611 Edge cases (quirks.mutation_test_ground_record off: a victim on a deck is tested against the deck record)
            Layer::Deck
        } else {
            // spec: S-0611 claim 6 and Edge cases (the cell's ground record, even for a deck victim)
            Layer::Ground
        };
        let (_, class) = spot_of(w, o.pos);
        // spec: S-0611 claim 6 (row 2: S-0208 claim 3's position choice with the victim's position as desired point; its draw as that claim gives)
        let chosen = crate::walk::choose_position(w, c, layer, class, false);
        // spec: S-0611 claim 6 (row 3: a building listed in the cell's ground layer refuses after the choice and its draw)
        chosen.is_none() || building_listed(w, c)
    };
    if refused {
        if q.refused_mutation_frees_position {
            // spec: S-0611 claim 7 (the refused victim's standing position is shown free from the tick of its death; its removal frees it again)
            w.cells
                .clear_position(spot.cell, spot.layer, spot.position, oq);
            w.mutation.refused.insert(id, spot);
        } else {
            // spec: S-0611 Edge cases (quirks.refused_mutation_frees_position off: the position stays taken until the victim is removed)
            for (cc, l, r) in records {
                if let Some(x) = w.cells.occ_mut(cc, l) {
                    *x = r;
                }
            }
        }
        return false;
    }
    let u = rules.unit(o.kind, o.type_index);
    let mut crashed = false;
    if u.is_some_and(|u| u.vehicle.crashable) {
        // spec: S-0611 claim 7 (Q-1461: for a victim that starts a crash, the standing position is marked taken again in the death tick)
        for (cc, l, r) in records.clone() {
            if let Some(x) = w.cells.occ_mut(cc, l) {
                *x = r;
            }
        }
        // spec: S-0611 claim 7 and S-0312 claim 16 (a crash begins only when the type has the crash flag and the victim's height above its surface is greater than 0 leptons; the flag alone on open ground does not prevent removal)
        crashed = crate::jump::start_crash(w, id, Some(hit));
        if !crashed {
            for &(cc, l, _) in &records {
                w.cells.clear_infantry(cc, l, id, oq);
            }
        }
    }
    // spec: S-0611 claim 8 (the owner: the attacker's current owner, else the attacking house, else none)
    let owner = hit
        .attacker
        .and_then(|a| w.obj(a))
        .map_or(hit.house, |a| a.owner);
    if !crashed {
        // spec: S-0611 claim 7 (a converted victim leaves play in the tick of its death, with no death sequence and no corpse)
        crate::combat::death::leave_play(w, id);
    }
    match rules.superweapons.mutation.anim {
        // spec: S-0611 claim 8 (one conversion animation at the victim's position, height included, in the tick of the death) and claim 7 (Q-1461: it starts also for a victim that starts a crash, independently of the crash)
        Some(a) => {
            let made = start(w, a, o.pos, o.layer, owner, true);
            if crashed && let Some(m) = made {
                mark_now(w, m);
            }
        }
        // spec: S-0611 Edge cases (absent conversion animation: the victim is removed with no replacement)
        None => w.note("S-0611 Edge cases: no conversion animation type; the converted victim leaves no replacement"),
    }
    true
}

/// The conversion animation `id` of a victim that started a crash marks its
/// standing position at once (claims 7 and 8): no ordinary removal of the
/// victim clears that mark again in the death tick.
fn mark_now(w: &mut World, id: ObjectId) {
    let Some(m) = w.mutation.makers.get(&id).cloned() else {
        return;
    };
    let rules = w.rules_arc();
    let making = m
        .anim
        .and_then(|a| rules.animations.get(a as usize))
        .is_some_and(|a| a.make_infantry != -1);
    if making {
        let (c, p) = m.spot;
        // spec: S-0611 claim 8 (it marks its standing position immediately at conversion; a victim retained by a crash does not perform the removal that would clear that mark in the death tick)
        w.cells.mark_anim(
            c,
            m.layer,
            p,
            id,
            m.owner,
            rules.quirks.infantry_animation_occupancy,
        );
    }
}

/// A refused victim leaves play (claim 7): its removal frees its standing
/// position again, whatever holds it then
/// (`quirks.refused_mutation_frees_position`).
pub fn removing(w: &mut World, id: ObjectId) {
    let Some(f) = w.mutation.refused.remove(&id) else {
        return;
    };
    let oq = crate::objects::occ_quirks(w);
    // spec: S-0611 Edge cases (its removal when die-2 completes frees that position again, even if a walker has taken it meanwhile)
    w.cells.clear_position(f.cell, f.layer, f.position, oq);
}

/// Frees an infantry-making animation's position at its end (claim 8).
fn free_spot(w: &mut World, id: ObjectId, m: &Maker) {
    let oq = crate::objects::occ_quirks(w);
    let (c, p) = m.spot;
    if w.rules().quirks.infantry_animation_occupancy {
        // spec: S-0611 claim 8 and Edge cases (at its end the position is shown free again, whatever else has marked it)
        w.cells.clear_position(c, m.layer, p, oq);
    } else {
        // spec: S-0611 Edge cases (quirks.infantry_animation_occupancy off: it frees only its own mark)
        w.cells.clear_infantry(c, m.layer, id, oq);
    }
}

/// The outcome of one creation attempt (claims 9 to 11).
enum Attempt {
    /// A unit was created and placed.
    Made(ObjectId),
    /// Nothing is created; the animation is removed.
    Nothing,
    /// The placement was refused; the animation stays.
    Refused,
}

/// Puts a made unit on the map at an animation's point (claim 10, S-0614
/// claim 20's rules for infantry). Returns whether it was placed.
fn place(w: &mut World, unit: ObjectId, p: Pos) -> bool {
    let c = p.cell();
    let surface = w.surface_at(p.x, p.y);
    let ground_level = w.cells.cell(c).map_or(0, |x| i32::from(x.level));
    let pos = if p.z == surface && w.cells.usable(c) {
        let (_, class) = spot_of(w, p);
        // spec: S-0611 claim 10 (at terrain height inside the usable area: the occupant-aware position choice of S-0208 claim 3, with its draw for a centre-class point, on the ground record)
        let Some(k) = crate::walk::choose_position(w, c, Layer::Ground, class, false) else {
            return false;
        };
        let (u, v) = crate::walk::standing_point(k);
        // spec: S-0204 claim 2 (a position within the cell)
        Pos::new(256 * c.x + u, 256 * c.y + v, p.z)
    } else {
        // spec: S-0611 claim 10 (not at terrain height: no position choice; the unit stands at the point itself)
        p
    };
    // spec: S-0611 claim 10 and S-0614 claim 20 (the entry outcome must be clear, judged on the ground record)
    if crate::occupancy::infantry_outcome(w, unit, c, ground_level)
        != crate::occupancy::Outcome::Clear
    {
        return false;
    }
    let layer = surface_of(w, p);
    if let Some(o) = w.obj_mut(unit) {
        // spec: S-0611 claim 10 (a high-bridge cell with the point above its terrain height: the unit stands on the deck)
        o.layer = layer;
    }
    // spec: S-0611 claim 10 and F3 (it faces 24 576 facing units, map south-east)
    crate::objects::place(w, unit, pos, MADE_FACING);
    true
}

/// Takes a made unit that could not be placed out of the game again: our
/// engine keeps no never-placed object (claim 11's departure). The object
/// and its creation set (claim 14: slot objects, slaves and theirs) are
/// removed with the records their creation made; every draw and creation
/// number of the set stays consumed and is never reused (claim 14, Q-1460).
fn discard(w: &mut World, unit: ObjectId) {
    crate::objects::discard_unplaced(w, unit);
}

/// One creation attempt at the end of an infantry-making animation
/// (claims 9 to 11).
fn attempt(w: &mut World, id: ObjectId, m: &mut Maker) -> Attempt {
    let rules = w.rules_arc();
    let i = m
        .anim
        .and_then(|a| rules.animations.get(a as usize))
        .map_or(-1, |a| a.make_infantry);
    let list = &rules.superweapons.mutation.made;
    // spec: S-0611 claim 9 and F4 (the made type is entry i when 0 ≤ i < n; otherwise nothing is created)
    let Some(ty) = usize::try_from(i)
        .ok()
        .and_then(|i| list.get(i).copied())
        .flatten()
    else {
        return Attempt::Nothing;
    };
    let live = m.owner.filter(|&h| !crate::econ::victory::defeated(w, h));
    let house = match live {
        Some(h) => Some(h),
        None => match crate::structures::garrison::civilian_house(w) {
            Some(civ) => {
                // spec: S-0611 claim 9 (no owner, or a defeated owner: the animation itself changes owner to the civilian house and the unit goes to it)
                if m.owner != Some(civ) {
                    m.owner = Some(civ);
                    w.effects.push(Effect::Maker {
                        id,
                        name: m.player.name_text(),
                        pos: m.pos,
                        owner: m.owner,
                        frame: m.player.frame(),
                        event: MakerEvent::Owner,
                    });
                }
                Some(civ)
            }
            // spec: S-0611 claim 9 (no civilian house: an ownerless animation creates nothing and a defeated owner keeps the unit)
            None => m.owner,
        },
    };
    let Some(house) = house else {
        return Attempt::Nothing;
    };
    // spec: S-0611 claim 10 (a fresh object of the made type: full health, starting experience under S-0311 claim 11, no passengers, no inherited order)
    // spec: S-0611 claim 14 and F5 (the attempt creates the whole creation set, refused or not, every member owned by the attempt's house; the position-choice draw comes after the whole set)
    let unit = crate::objects::create(w, Kind::Infantry, ty, Some(house));
    if !place(w, unit, m.pos) {
        // spec: S-0611 claims 11 and 14 (no unit appears; our engine builds none of the creation set, a documented departure; the set's creation draws, then the position-choice draw where claim 10's choice applies, are not undone)
        discard(w, unit);
        return Attempt::Refused;
    }
    let role = w.houses.get(usize::from(house.0)).map(|x| x.role);
    if matches!(role, Some(HouseRole::Neutral | HouseRole::Special))
        && let Some(o) = w.obj_mut(unit)
    {
        // spec: S-0611 claim 10 and Edge cases (hunt for the neutral and special houses only; a player house's unit gets no behaviour from its creation)
        o.beh.queue(Some(crate::rules::Behaviour::Hunt));
    }
    Attempt::Made(unit)
}

/// An infantry-making animation's turn (claims 8 to 12; S-0109 claims 5,
/// 15 and 16).
pub fn turn(w: &mut World, id: ObjectId) {
    let Some(mut m) = w.mutation.makers.remove(&id) else {
        return;
    };
    let rules = w.rules_arc();
    let seize = rules.quirks.infantry_animation_occupancy;
    let making = |m: &Maker| {
        m.anim
            .and_then(|a| rules.animations.get(a as usize))
            .is_some_and(|a| a.make_infantry != -1)
    };
    if making(&m) {
        let (c, p) = m.spot;
        // spec: S-0611 claim 8 (in each of its updates its position is marked taken and the cell's infantry house becomes its owner)
        w.cells.mark_anim(c, m.layer, p, id, m.owner, seize);
    }
    let aq = AnimQuirks::from(&w.rules().quirks);
    let t = w.tick;
    let before = (m.anim, m.player.name.clone());
    let mut env = SimEnv {
        w: &mut *w,
        pos: m.pos,
        anim: m.anim,
    };
    let playing = m.player.update(t, &mut env, &aq);
    m.anim = env.anim;
    if (m.anim, m.player.name.clone()) != before {
        w.effects.push(Effect::Maker {
            id,
            name: m.player.name_text(),
            pos: m.pos,
            owner: m.owner,
            frame: m.player.frame(),
            event: MakerEvent::Switched,
        });
    }
    if playing {
        w.mutation.makers.insert(id, m);
        return;
    }
    // spec: S-0611 claim 8 (at its end the position is shown free again)
    free_spot(w, id, &m);
    let unit = match attempt(w, id, &mut m) {
        Attempt::Made(u) => Some(u),
        Attempt::Nothing => None,
        Attempt::Refused => {
            // spec: S-0611 claim 11 (the animation stays: it steps back one frame and tries again when that frame is next due)
            m.player.step_back();
            w.effects.push(Effect::Maker {
                id,
                name: m.player.name_text(),
                pos: m.pos,
                owner: m.owner,
                frame: m.player.frame(),
                event: MakerEvent::Refused,
            });
            w.mutation.makers.insert(id, m);
            return;
        }
    };
    w.effects.push(Effect::Maker {
        id,
        name: m.player.name_text(),
        pos: m.pos,
        owner: m.owner,
        frame: m.player.frame(),
        event: MakerEvent::Ended {
            unit,
            conversion: m.conversion,
        },
    });
    let skip = w.rules().quirks.update_order_removal_skip;
    // spec: S-0611 claim 9 and S-0109 claim 16 (then it is removed); S-0203 claim 10 (a removal from the update order)
    w.update.leave(id, skip);
}

/// The genetic mutation superweapon's launch (claims 2 to 4) by house `h`
/// with type `i` at target cell `cell`.
pub fn launch(w: &mut World, h: HouseId, i: u32, cell: CellXY) {
    let rules = w.rules_arc();
    let mr = &rules.superweapons.mutation;
    // spec: S-0611 F1 (P = (256x + 128, 256y + 128, h(c) + b), b the bridge lift of S-0513 F2)
    let p = crate::superweapons::cell_point(w, cell, true);
    if let Some(name) = mr
        .ion_blast
        .clone()
        .filter(|n| rules.animation(n).is_some())
    {
        // spec: S-0611 claim 2 and F1 (the ion-blast animation at P + (0, 0, 5), an ordinary animation)
        let at = Pos::new(p.x, p.y, p.z + LAUNCH_LIFT);
        crate::combat::effects::anim(w, &name, at);
    }
    // spec: S-0611 claim 2 (the genetic-mutation activation announcement for every player)
    crate::superweapons::event(
        w,
        crate::superweapons::SwEvent::Launch,
        Some(h),
        Some(i),
        None,
    );
    // spec: S-0611 claim 2 (the activation sound, positional at the mutation point; the launching player's aiming ends and its ready announcement is silenced, S-0513 claim 5)
    crate::superweapons::event(
        w,
        crate::superweapons::SwEvent::GeneticSound,
        Some(h),
        Some(i),
        Some(p),
    );
    // spec: S-0611 claim 2 (a radar event of the invulnerability launch's kind at the target cell)
    crate::superweapons::event(
        w,
        crate::superweapons::SwEvent::Radar,
        Some(h),
        Some(i),
        Some(crate::superweapons::cell_point(w, cell, false)),
    );
    if mr.explosion {
        // spec: S-0611 Import mapping (an empty warhead reference: no hits for that mode)
        let Some(warhead) = mr.explosion_warhead else {
            return;
        };
        // spec: S-0611 claim 3 (one area hit centred on P, 10 000 hit points, the explosion warhead, no attacker, the launching house; not ignoring defences)
        crate::combat::area::detonate(
            w,
            crate::combat::area::Detonation {
                point: p,
                damage: EXPLOSION_AMOUNT,
                warhead: Some(warhead),
                source: None,
                house: Some(h),
                resources: true,
            },
        );
        return;
    }
    let Some(warhead) = mr.cell_warhead else {
        return;
    };
    // spec: S-0611 claim 4 (the cells of the target cell's effect block, S-0513 F1's order)
    for (_, c) in crate::superweapons::block(w, cell) {
        // spec: S-0611 claim 4 (the swept layer: S-0513 claim 4's rule, quirks.superweapon_block_one_layer)
        for l in crate::superweapons::swept_layers(w, c) {
            // spec: S-0611 claim 4 (every infantry unit listed in the cell when it is reached is hit; quirks.superweapon_list_cut_by_removal does not apply)
            for id in crate::superweapons::listed(w, c, l) {
                let infantry = w
                    .obj(id)
                    .is_some_and(|o| o.kind == Kind::Infantry && o.on_map);
                if !infantry {
                    continue;
                }
                let strength = crate::combat::damage::strength(w, id);
                // spec: S-0611 claim 4 and F2 (one hit of the type's whole strength, distance 0, the cell warhead, ignoring defences, no attacker, the launching house, no no-escape marker)
                let hit = Hit {
                    amount: strength,
                    warhead: Some(warhead),
                    distance: 0,
                    attacker: None,
                    house: Some(h),
                    ignore_defences: true,
                    no_escape: false,
                };
                crate::combat::damage::apply(w, id, hit);
            }
        }
    }
}
