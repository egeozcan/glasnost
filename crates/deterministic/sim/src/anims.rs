//! Ordinary animations in play (S-0109; S-0203 claim 2): every animation
//! that no specialised model plays (burning walkers, infantry-making,
//! domination, depot slot and virus-death animations, storm clouds) is an
//! object of the update order here. It is created at the event that makes
//! it and appended to the update order (S-0109 claim 29), takes its creation
//! draws and, with no loop delay, its start event at once (claims 13 and
//! 28; S-0202 claim 10), then plays the stage clock of
//! [`crate::anim_clock`] in its own turns with its draws from the scenario
//! stream (claim 28): the middle event when μ > 0 (claim 14), damage over
//! time (claim 17, F5), the start event at the end of a loop delay and at a
//! follow-up switch (claims 11, 13 and 15), and its removal at its end
//! (claim 16), a removal from the update order (S-0203 claim 10). An
//! attached animation follows its owner and is deleted at its next update
//! once the owner has left the game (claim 22).
//!
//! An animation whose type is a bouncing type at its creation also carries
//! a flight (S-0319, [`crate::bounce`]): its creation places it, and each
//! of its turns takes the motion step first, ending it at a landing with
//! the landing's effects; otherwise the trailer of any type that names one
//! (S-0319 claim 12) and then the stage update follow.

use std::collections::BTreeMap;

use glasnost_num::F64;
use serde::{Deserialize, Serialize};

use crate::anim_clock::{AnimQuirks, Creation, Env, Playback, Player};
use crate::combat::area::{self, Detonation};
use crate::rules::{AnimRules, Kind};
use crate::world::{HouseId, ObjectId, Pos, World};

/// The ordinary animations in play.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimWorld {
    /// The animations, by number in the update order.
    pub anims: BTreeMap<ObjectId, Anim>,
    /// The animation taking its turn, while its playback runs (it is out of
    /// [`AnimWorld::anims`] then), its role, and whether something removed
    /// it during that turn. Never set between turns.
    #[serde(skip)]
    current: Option<(ObjectId, Role, bool)>,
}

/// What an animation is to the rest of the game, beyond its playback.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// An ordinary animation.
    #[default]
    Ordinary,
    /// A tile animation started before the second recalculation pass of
    /// map loading: none survives loading (S-0010 claim 19).
    FirstPassTile,
    /// A paratrooper's parachute (S-0522 claim 13): its end detaches it
    /// from the falling unit.
    Parachute,
    /// A house's teleport source marker (S-0513 claims 33 and 34).
    SourceMarker(HouseId),
    /// A drainer's drain animation (S-0608 claims 5 and 14).
    Drain(ObjectId),
    /// A grappling parasite's grab animation (S-0614 claim 25), with the
    /// house whose house-colour palette and the cell whose unscaled ground
    /// level the parasite last gave it.
    Grab {
        /// The grappling parasite.
        parasite: ObjectId,
        /// The parasite's house, for its palette.
        house: Option<HouseId>,
        /// The host's cell, for its brightness.
        cell: Option<glasnost_map::CellXY>,
    },
}

/// One ordinary animation in play.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anim {
    /// Its current type, by registration index (it changes at a follow-up
    /// switch); `None` for a follow-up name no type is registered under.
    pub anim: Option<u32>,
    /// Its stage clock.
    pub player: Player,
    /// Its position; `None` for a resource glint, whose position no spec
    /// gives (S-0202 claim 14).
    pub pos: Option<Pos>,
    /// The owner it is attached to and its displacement from the owner's
    /// position at attachment, height included (S-0109 claim 22).
    pub owner: Option<(ObjectId, Pos)>,
    /// The damage accumulator *A* of F5.
    pub acc: F64,
    /// What it is to the rest of the game.
    pub role: Role,
    /// Its flight, when its type was a bouncing type at its creation (S-0319
    /// claim 1): it moves by S-0319 for as long as it is in play, whatever
    /// type it switches to (claim 11).
    #[serde(default)]
    pub flight: Option<crate::bounce::Flight>,
}

/// What a creator gives a new ordinary animation besides its type and
/// place.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Spawn {
    /// Initial loop delay, repeat factor and reverse request (S-0109 claims
    /// 7, 8, 11 and 29).
    pub creation: Creation,
    /// The owner it is attached to and its displacement (S-0109 claim 22).
    pub owner: Option<(ObjectId, Pos)>,
    /// Its role.
    pub role: Role,
}

/// A created animation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Spawned {
    /// Its number in the update order; `None` when the rules list no such
    /// type (S-0004 claim 3: no fields, no draw, nothing in play).
    pub id: Option<ObjectId>,
    /// The stage delay the creation's ranged draw gave (S-0109 claim 3),
    /// when one was taken.
    pub delay: Option<i32>,
}

/// The damage factor *c* of F5 for an animation attached to a terrain
/// object.
const TERRAIN_FACTOR: i64 = 5; // spec: S-0109 claim 17, F5 and Constants (5δ when attached to a terrain object)

/// Creates an ordinary animation of type `name` at `at` (S-0109 claim 29):
/// its creation draws (S-0202 claim 10), its start event when it has no
/// loop delay (claim 13), then it enters the update order at the end
/// (S-0203 claims 3 and 4).
pub fn spawn(w: &mut World, name: &str, at: Option<Pos>, s: Spawn) -> Spawned {
    let rules = w.rules_arc();
    let Some(index) = anim_index(w, name.as_bytes()) else {
        return Spawned::default();
    };
    let Some(a) = rules.animations.get(index as usize) else {
        return Spawned::default();
    };
    // spec: S-0109 claim 29 and S-0202 claim 10 (the creation draws are taken at the creation)
    let (drawn, raws) = crate::load::creation_draws(w, a);
    // spec: S-0319 claims 1 to 3 (a bouncing type at creation: the first velocity from the three raw values; a falling animation is placed at its creation point, a meteor animation at its start position)
    let (at, flight) = match (raws, at) {
        (Some(r), Some(p)) => {
            let (placed, f) = crate::bounce::create(w, a, r, p);
            (Some(placed), Some(f))
        }
        // An animation whose position is not carried (a resource glint) has no point to fly from.
        _ => (at, None),
    };
    if s.creation.loop_delay <= 0 {
        // spec: S-0109 claim 13 and State and ordering (created with no loop delay: the start event in the same tick)
        crate::load::start_event(w, a, at.map(|p| (p.x, p.y, p.z)), true);
    }
    // A creator's loop delay defers the start event to the delay's end (claim 11).
    let play = crate::mutation::playback(w, a);
    let aq = AnimQuirks::from(&w.rules().quirks);
    let t = w.tick;
    let mut env = AnimEnv {
        w: &mut *w,
        pos: at,
        anim: Some(index),
        acc: F64::ZERO,
        factor: 1,
    };
    let player = Player::create(a.name.as_bytes(), play, t, drawn, s.creation, &mut env, &aq);
    let mode = rules.mode;
    // spec: S-0109 claim 17 and F5 (A₀ = 1.0; quirks.anim_damage_starts_at_one off: 0.0)
    let acc = if rules.quirks.anim_damage_starts_at_one {
        F64::from_i64(1, mode)
    } else {
        F64::ZERO
    };
    let id = w.new_id();
    // spec: S-0319 claim 11 and Edge cases (a bouncing animation whose placement point is exactly (0, 0, 0) is not placed: its creation draws are taken, but it never takes a turn, never moves, lands or ends by itself and is never drawn; design rule, no switch)
    // design: as for a debris piece at that point (`pieces::create`), its start event is still made at its creation when it has no loop delay (a deferred one never comes: it takes no turn) and its number is still its own; no record of it is kept, as nothing reads it again.
    if flight.is_some() && at == Some(Pos::new(0, 0, 0)) {
        return Spawned {
            id: Some(id),
            delay: drawn,
        };
    }
    w.anims.anims.insert(
        id,
        Anim {
            anim: Some(index),
            player,
            pos: at,
            owner: s.owner,
            acc,
            role: s.role,
            flight,
        },
    );
    // spec: S-0109 claim 29 and S-0203 claims 3 and 4 (appended; eligible for its first update later in the same pass)
    w.update.enter_new(id);
    Spawned {
        id: Some(id),
        delay: drawn,
    }
}

/// The registration index of animation type `name` (a byte-wise,
/// case-insensitive match, as the rules' lookup).
fn anim_index(w: &World, name: &[u8]) -> Option<u32> {
    w.rules()
        .animations
        .iter()
        .position(|a| a.name.as_bytes().eq_ignore_ascii_case(name))
        .and_then(|i| u32::try_from(i).ok())
}

/// Removes animation `id` from play now (a removal from the update order,
/// S-0203 claim 10), when another rule ends it: a drain link's end, a new
/// source pick or a warp launch, the end of loading for a first-pass tile
/// animation.
pub fn remove(w: &mut World, id: ObjectId) {
    let present = w.anims.anims.remove(&id).is_some();
    let current = match &mut w.anims.current {
        Some((c, _, gone)) if *c == id => {
            *gone = true;
            true
        }
        _ => false,
    };
    if present || current {
        let skip = w.rules().quirks.update_order_removal_skip;
        w.update.leave(id, skip);
    }
}

/// The animations of role `role` in play, in update-order number order,
/// the one taking its turn included (a drain link that its own damage ends
/// removes it, S-0608 claim 14).
pub fn with_role(w: &World, role: Role) -> Vec<ObjectId> {
    let mut ids: Vec<ObjectId> = w
        .anims
        .anims
        .iter()
        .filter(|(_, a)| a.role == role)
        .map(|(&id, _)| id)
        .collect();
    if let Some((id, r, false)) = w.anims.current
        && r == role
    {
        ids.push(id);
        ids.sort_unstable();
    }
    ids
}

/// The warhead of a delivery of S-0109 claim 17 by an animation of current
/// type `a`: the demolition warhead for the built-in invisibility-animation
/// type, the game-wide flame warhead for every other type; `None` when the
/// chosen one is not given (the amount is still taken, no damage is done).
pub fn damage_warhead(
    rules: &crate::rules::SimRules,
    a: &AnimRules,
) -> Option<crate::combat::data::WarheadId> {
    if a.invisibility {
        // spec: S-0109 claim 17 and Import mapping (C4Warhead for the built-in invisibility-animation type, chosen on each delivery by the current type's name)
        rules.combat.general.c4_warhead
    } else {
        // spec: S-0109 claim 17 and Import mapping (FlameDamage2 for every other type, and always when no type has exactly that name)
        rules.combat.general.flame_damage2
    }
}

/// Removes every first-pass tile animation (S-0010 claim 19: none that the
/// first recalculation pass started survives loading).
pub fn drop_first_pass_tiles(w: &mut World) {
    for id in with_role(w, Role::FirstPassTile) {
        // spec: S-0010 claim 19 (none that the first pass started survives loading)
        remove(w, id);
    }
}

/// A paratrooper `trooper` has landed (S-0522 claim 13): a parachute still
/// attached to it makes its current pass its last.
pub fn parachute_landed(w: &mut World, trooper: ObjectId) {
    for a in w.anims.anims.values_mut() {
        if a.role == Role::Parachute && a.owner.is_some_and(|(o, _)| o == trooper) {
            // spec: S-0522 claim 13 (still present on landing: its remaining passes become 0, making the current pass its last)
            a.player.last_pass();
        }
    }
}

/// The parachute attached to `trooper`, if one still plays.
pub fn parachute_of(w: &World, trooper: ObjectId) -> Option<ObjectId> {
    w.anims
        .anims
        .iter()
        .find(|(_, a)| a.role == Role::Parachute && a.owner.is_some_and(|(o, _)| o == trooper))
        .map(|(&id, _)| id)
}

/// The stage clock's surroundings for an ordinary animation: follow-up
/// types from the rules, draws from the scenario stream (S-0109 claim 28),
/// damage over time (claim 17) and the middle and start events of the
/// current type (claims 13 and 14).
struct AnimEnv<'a> {
    w: &'a mut World,
    pos: Option<Pos>,
    anim: Option<u32>,
    acc: F64,
    /// The factor *c* of F5.
    factor: i64,
}

impl AnimEnv<'_> {
    fn current(&self) -> Option<std::sync::Arc<crate::rules::SimRules>> {
        self.anim.map(|_| self.w.rules_arc())
    }

    /// The damage of one advance (claim 17, F5).
    fn damage(&mut self, a: &AnimRules) {
        // spec: S-0109 claim 17 (a type with δ > 0 that is not bouncing delivers on every advance)
        if a.bouncing || !a.damage.gt(F64::ZERO) {
            return;
        }
        let rules = self.w.rules_arc();
        let mode = rules.mode;
        let one = F64::from_i64(1, mode);
        // spec: S-0109 F5 (B = A + cδ, c = 1 unattached, 5 attached to a terrain object; binary64 toward zero)
        let add = if self.factor == 1 {
            a.damage
        } else {
            F64::from_i64(self.factor, mode).mul(a.damage, mode)
        };
        let b = self.acc.add(add, mode);
        if !b.ge(one) {
            // spec: S-0109 F5 (B < 1.0: nothing delivered, A = B)
            self.acc = b;
            return;
        }
        // spec: S-0109 F5 (n = trunc(B) delivered, A = B − n)
        let n = b.to_i32_common();
        self.acc = b.sub(F64::from_i64(i64::from(n), mode), mode);
        let Some(point) = self.pos else {
            self.w.note("S-0109 claim 17: an animation whose position is not carried (a resource glint) takes its damage from the accumulator but delivers none");
            return;
        };
        // spec: S-0109 claim 17 and Import mapping (one of two game-wide warheads by the current type's name; absent: the amount is taken, no damage)
        let warhead = damage_warhead(&rules, a);
        // spec: S-0109 claim 17 and S-0302 claim 1 (an area detonation with resource reduction allowed, no attacker and no attacking house)
        area::detonate(
            self.w,
            Detonation {
                point,
                damage: n,
                warhead,
                source: None,
                house: None,
                resources: true,
            },
        );
    }
}

impl Env for AnimEnv<'_> {
    fn follow_up(&mut self, name: &[u8]) -> Playback {
        self.anim = anim_index(self.w, name);
        let rules = self.w.rules_arc();
        match self.anim.and_then(|i| rules.animations.get(i as usize)) {
            Some(a) => crate::mutation::playback(self.w, a),
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
        let Some(rules) = self.current() else { return };
        let Some(a) = self.anim.and_then(|i| rules.animations.get(i as usize)) else {
            return;
        };
        // spec: S-0109 claim 17 and State and ordering (Q-1716: the damage due on the advance first, its whole detonation, draws and kills included, the ending advance and the advance before a follow-up switch too)
        self.damage(a);
        if self.left_play() {
            // spec: S-0109 State and ordering (an animation that leaves play in its own detonation: nothing more of its update, no middle event, pass end, follow-up switch or end; our engine's rule)
            return;
        }
        // spec: S-0109 claim 14 and State and ordering (then the middle event, on every advance after which a + k = μ, μ ≠ 0; not for a bouncing type; also on an ending advance, before the end or a follow-up switch)
        if !a.bouncing && a.mu.is_some_and(|m| m != 0 && p.start.wrapping_add(k) == m) {
            let at = self.pos.map(|q| (q.x, q.y, q.z));
            crate::load::middle_event(self.w, a, at);
        }
    }

    fn left_play(&self) -> bool {
        matches!(self.w.anims.current, Some((_, _, true)))
    }

    fn switched(&mut self, _name: &[u8]) {
        // spec: S-0109 claim 15 and F5 (the accumulator restarts at 0.0 after a follow-up switch)
        self.acc = F64::ZERO;
        let Some(rules) = self.current() else { return };
        if let Some(a) = self.anim.and_then(|i| rules.animations.get(i as usize)) {
            // spec: S-0109 claims 13, 15 and 28 (the new type's start event after its delay choice)
            let at = self.pos.map(|q| (q.x, q.y, q.z));
            crate::load::start_event(self.w, a, at, true);
        }
    }

    fn delay_ended(&mut self, _p: &Playback) {
        let Some(rules) = self.current() else { return };
        if let Some(a) = self.anim.and_then(|i| rules.animations.get(i as usize)) {
            // spec: S-0109 claims 11 and 13 (the start event on the update that brings a loop delay to 0)
            let at = self.pos.map(|q| (q.x, q.y, q.z));
            crate::load::start_event(self.w, a, at, true);
        }
    }
}

/// An ordinary animation's turn (S-0109 claims 5, 11 and 13 to 22; the
/// state table).
pub fn turn(w: &mut World, id: ObjectId) {
    let Some(mut a) = w.anims.anims.remove(&id) else {
        return;
    };
    let skip = w.rules().quirks.update_order_removal_skip;
    let mut factor = 1;
    if let Some((owner, d)) = a.owner {
        let Some(o) = w.obj(owner) else {
            // spec: S-0109 claim 22 (an owner removed from the game: deleted at its next update, which does nothing else)
            ended(w, id, &a);
            w.update.leave(id, skip);
            return;
        };
        // spec: S-0109 claim 22 (its position is the owner's plus its displacement at attachment, height included)
        // spec: S-0109 Open questions (an owner its own detonation removes: the update goes on at the owner's last position; our engine's design)
        a.pos = Some(Pos::new(o.pos.x + d.x, o.pos.y + d.y, o.pos.z + d.z));
        if o.kind == Kind::Terrain {
            // spec: S-0109 claim 17 and F5 (c = 5 when attached to a terrain object)
            factor = TERRAIN_FACTOR;
        }
    }
    let rules = w.rules_arc();
    let missing;
    // spec: S-0319 claim 11 (claims 4 and 7 to 10 read the type the animation has now; a follow-up name no type is registered under has absent-key defaults)
    let current = match a.anim.and_then(|i| rules.animations.get(i as usize)) {
        Some(t) => t,
        None => {
            missing = AnimRules::default();
            &missing
        }
    };
    if let (Some(f), Some(p)) = (a.flight.as_mut(), a.pos.as_mut()) {
        // spec: S-0319 claim 4 and State and ordering (the motion step first, in every turn, the first update and a loop delay's included)
        if let Some(landing) = crate::bounce::step(w, f, p, current) {
            let at = *p;
            // spec: S-0319 claims 7 and 8 (the landing's effects at the landing point, then the end in the same turn: no stage advance, trailer or follow-up switch)
            crate::bounce::land(w, current, at, landing);
            ended(w, id, &a);
            // spec: S-0319 State and ordering and S-0203 claim 10 (it leaves play: a removal from the update order)
            w.update.leave(id, skip);
            return;
        }
    }
    if let Some(p) = a.pos
        && let Some(name) = crate::bounce::trailer_due(current, w.tick)
    {
        // spec: S-0319 claim 12 (every animation whose type names a trailer, after its motion step and before its stage update, at its position in play, with S-0109 claim 29's trailer settings)
        crate::combat::effects::trailer(w, &name, p);
    }
    w.anims.current = Some((id, a.role, false));
    let aq = AnimQuirks::from(&w.rules().quirks);
    let t = w.tick;
    let mut env = AnimEnv {
        w: &mut *w,
        pos: a.pos,
        anim: a.anim,
        acc: a.acc,
        factor,
    };
    let playing = a.player.update(t, &mut env, &aq);
    a.anim = env.anim;
    a.acc = env.acc;
    let gone = w.anims.current.take().is_some_and(|(_, _, g)| g);
    if gone {
        // Removed by another rule during its own turn (a drain link ended by its damage, say): its
        // playback stopped there (S-0109 State and ordering).
        ended(w, id, &a);
        return;
    }
    if playing {
        w.anims.anims.insert(id, a);
        return;
    }
    ended(w, id, &a);
    // spec: S-0109 claim 16 (the last pass ends with no follow-up type: removed in that update); S-0203 claim 10 (a removal from the update order)
    w.update.leave(id, skip);
}

/// What an animation's end tells the rest of the game.
fn ended(w: &mut World, _id: ObjectId, a: &Anim) {
    if a.role == Role::Parachute
        && let Some((owner, _)) = a.owner
        && let Some(o) = w.obj_mut(owner)
    {
        // spec: S-0522 claim 13 (a parachute whose playback ends before landing is no longer attached: the no-parachute recurrence of S-0511 F3 from then on)
        o.fall.chute = false;
    }
}

/// The drawn state of animation `id`: its type name, image frame and
/// position (presentation).
pub fn shown(w: &World, id: ObjectId) -> Option<(&[u8], i32, Option<Pos>)> {
    let a = w.anims.anims.get(&id)?;
    Some((&a.player.name, a.player.frame(), a.pos))
}
