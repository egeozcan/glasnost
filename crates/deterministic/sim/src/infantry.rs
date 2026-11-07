//! Infantry sequences, idle actions and fear (S-0107 claims 1, 4 to 10, 14
//! and 15). Sequences are part of game state: they decide readiness (S-0215
//! claim 11) and gate idle actions, which draw from the scenario random
//! stream. Drawing them is the renderer's (S-0107 claims 2, 3).

use glasnost_num::F64;
use glasnost_rules::sim::MovementZone;
use serde::{Deserialize, Serialize};

use crate::rules::Kind;
use crate::world::{Layer, ObjectId, Target, World};

/// The 42 sequence kinds (S-0107 Terms), with our snake-case names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum Seq {
    Stand,
    Guard,
    Prone,
    Walk,
    FireStanding,
    GoProne,
    Crawl,
    GetUp,
    FireProne,
    IdleA,
    IdleB,
    Die1,
    Die2,
    Die3,
    Die4,
    Die5,
    Tread,
    Swim,
    WetIdleA,
    WetIdleB,
    WetDie1,
    WetDie2,
    WetFire,
    Hover,
    Fly,
    Tumble,
    FireAirborne,
    Deploy,
    Deployed,
    DeployedFire,
    DeployedIdle,
    Undeploy,
    Cheer,
    Parachute,
    AirDeathStart,
    AirDeathFall,
    AirDeathEnd,
    Panic,
    Shovel,
    Carry,
    SecondaryFire,
    SecondaryFireProne,
}

impl Seq {
    /// Our name (the key of a type's `sequences`).
    pub fn key(self) -> &'static str {
        use Seq::*;
        match self {
            Stand => "stand",
            Guard => "guard",
            Prone => "prone",
            Walk => "walk",
            FireStanding => "fire_standing",
            GoProne => "go_prone",
            Crawl => "crawl",
            GetUp => "get_up",
            FireProne => "fire_prone",
            IdleA => "idle_a",
            IdleB => "idle_b",
            Die1 => "die_1",
            Die2 => "die_2",
            Die3 => "die_3",
            Die4 => "die_4",
            Die5 => "die_5",
            Tread => "tread",
            Swim => "swim",
            WetIdleA => "wet_idle_a",
            WetIdleB => "wet_idle_b",
            WetDie1 => "wet_die_1",
            WetDie2 => "wet_die_2",
            WetFire => "wet_fire",
            Hover => "hover",
            Fly => "fly",
            Tumble => "tumble",
            FireAirborne => "fire_airborne",
            Deploy => "deploy",
            Deployed => "deployed",
            DeployedFire => "deployed_fire",
            DeployedIdle => "deployed_idle",
            Undeploy => "undeploy",
            Cheer => "cheer",
            Parachute => "parachute",
            AirDeathStart => "air_death_start",
            AirDeathFall => "air_death_fall",
            AirDeathEnd => "air_death_end",
            Panic => "panic",
            Shovel => "shovel",
            Carry => "carry",
            SecondaryFire => "secondary_fire",
            SecondaryFireProne => "secondary_fire_prone",
        }
    }

    /// (interruptible, base delay b, normalised) of S-0107 table T1.
    pub fn table(self) -> (bool, i32, bool) {
        use Seq::*;
        // spec: S-0107 Constants table T1 (interruptible, base delay, normalised by game speed)
        match self {
            Stand | Guard => (true, 0, false),
            Prone => (true, 6, false),
            Walk => (true, 3, false),
            FireStanding | Crawl | FireProne | Swim | WetFire | Fly | Tumble | FireAirborne => {
                (true, 1, false)
            }
            GoProne | GetUp => (false, 1, false),
            IdleA | IdleB | WetIdleA | WetIdleB => (true, 3, true),
            Die1 | Die2 | Die3 | Die4 | Die5 | WetDie1 | WetDie2 => (false, 1, false),
            Tread => (true, 3, false),
            Hover => (true, 2, true),
            Deploy | Undeploy => (false, 1, false),
            Deployed | DeployedFire | DeployedIdle | Parachute => (true, 1, false),
            Cheer => (false, 3, true),
            AirDeathStart | AirDeathEnd => (false, 3, false),
            AirDeathFall => (false, 1, false),
            Panic => (true, 4, false),
            Shovel => (true, 6, false),
            Carry => (true, 3, false),
            SecondaryFire | SecondaryFireProne => (true, 1, false),
        }
    }

    /// Whether the kind is one of the deploy kinds (S-0107 Terms).
    pub fn is_deploy(self) -> bool {
        matches!(
            self,
            Seq::Deploy | Seq::Deployed | Seq::DeployedFire | Seq::DeployedIdle
        )
    }
}

/// An infantry unit's sequence and fear state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfantryState {
    /// Current sequence.
    pub seq: Option<Seq>,
    /// Tick at which the stage last restarted.
    pub seq_t0: i32,
    /// Stage delay d.
    pub seq_delay: i32,
    /// Fear level, 0 … 300.
    pub fear: i32,
    /// Prone state.
    pub prone: bool,
    /// Attack state (S-0107 claim 11, combat).
    pub attack: bool,
    /// Tick at which the idle timer has run out (S-0107 claim 14).
    pub idle_until: i32,
    /// The water record of an amphibious-destroyer type (S-0107 claim 7).
    pub wet: Option<bool>,
    /// The target as the last change-of-target test left it (S-0107 claim
    /// 11, S-0215 claim 11): [`watch_set`] and [`watch_test`].
    #[serde(default)]
    pub watch: Option<Target>,
}

impl InfantryState {
    /// The stage seen after the updates of tick `t` (S-0107 claim 4).
    pub fn stage(&self, t: i32) -> i32 {
        // spec: S-0107 claim 4 (stage = ⌊(t − tᵣ) / d⌋; a delay of 0 freezes it at 0)
        if self.seq_delay >= 1 {
            t.wrapping_sub(self.seq_t0).max(0) / self.seq_delay
        } else {
            0
        }
    }
}

/// The game-speed normalisation N(b, s) of S-0107 F3.
pub fn normalised(b: i32, s: u8) -> i32 {
    // spec: S-0107 Constants table T3 (A(b, s) for 1 ≤ b ≤ 4)
    const A: [[i32; 8]; 4] = [
        [2, 2, 1, 1, 1, 1, 1, 1],
        [3, 3, 3, 2, 2, 2, 1, 1],
        [5, 4, 4, 3, 3, 2, 2, 1],
        [7, 6, 5, 4, 4, 4, 3, 2],
    ];
    let s = usize::from(s.min(7)); // spec: S-0107 F3 (s above 7 reads column 7)
    match b {
        // spec: S-0107 F3 (N(0, s) = 0; table for 1 … 4; ⌊8b / (s + 1)⌋ from 5)
        0 => 0,
        1..=4 => A[(b - 1) as usize][s], // spec: S-0107 F3 (table T3 for b = 1 … 4)
        _ => 8 * b / (s as i32 + 1),     // spec: S-0107 F3 (⌊8b / (s + 1)⌋ from b = 5)
    }
}

/// A sequence entry's frame count C for a unit (0 when absent).
fn count(w: &World, id: ObjectId, q: Seq) -> i32 {
    let Some(o) = w.obj(id) else { return 0 };
    w.rules()
        .unit(o.kind, o.type_index)
        .and_then(|u| u.sequences.get(q.key()))
        .map_or(0, |e| e.count)
}

fn entry_dir(w: &World, id: ObjectId, q: Seq) -> Option<u8> {
    let o = w.obj(id)?;
    w.rules()
        .unit(o.kind, o.type_index)
        .and_then(|u| u.sequences.get(q.key()))
        .and_then(|e| e.direction)
}

/// Whether the unit's cell is a wet cell (S-0107 Terms) and it is not on a
/// bridge deck.
fn on_wet(w: &World, id: ObjectId) -> bool {
    use glasnost_map::terrain::land;
    let Some(o) = w.obj(id) else { return false };
    o.layer == crate::world::Layer::Ground
        && w.cells
            .cell(o.pos.cell())
            .is_some_and(|c| c.land == land::WATER || c.land == land::BEACH)
}

/// The substitution of S-0107 claim 7 for a requested kind.
pub(crate) fn substitute(w: &World, id: ObjectId, q: Seq) -> Seq {
    let Some(o) = w.obj(id) else { return q };
    let Some(u) = w.rules().unit(o.kind, o.type_index) else {
        return q;
    };
    let fear = o.infantry.as_ref().map_or(0, |s| s.fear);
    // spec: S-0107 claim 7 (carrying: enslaved units only; none in M3)
    // spec: S-0107 claim 7 (water: amphibious-destroyer types on a wet cell)
    if u.zone == MovementZone::AmphibiousDestroyer && on_wet(w, id) {
        let wet = match q {
            Seq::Walk | Seq::Crawl => Some(Seq::Swim),
            Seq::Prone | Seq::Stand => Some(Seq::Tread),
            Seq::IdleA => Some(Seq::WetIdleA),
            Seq::IdleB => Some(Seq::WetIdleB),
            Seq::Die1 => Some(Seq::WetDie1),
            Seq::Die2 => Some(Seq::WetDie2),
            Seq::FireStanding | Seq::FireProne => Some(Seq::WetFire),
            _ => None,
        };
        if let Some(x) = wet {
            return x;
        }
    }
    // spec: S-0107 claim 7 and Constants (hovering: stand becomes hover when the unit is airborne, 208 leptons above the surface, and not on a bridge deck, and its hover entry has S ≥ 1; quirks.hover_needs_nonzero_start off: C > 0 instead)
    if q == Seq::Stand
        && o.layer != Layer::Deck
        && crate::combat::weapon::airborne_by_height(w, id)
        && u.sequences.get(Seq::Hover.key()).is_some_and(|e| {
            if w.rules().quirks.hover_needs_nonzero_start {
                e.start >= 1
            } else {
                e.count > 0
            }
        })
    {
        return Seq::Hover;
    }
    // spec: S-0107 claim 7 (panic: walk becomes panic above fear 199)
    if q == Seq::Walk && fear > 199 {
        return Seq::Panic;
    }
    q
}

/// Destination-driven get-up (S-0107 claim 15, S-0213 claim 21): called
/// after a destination is assigned to `id`, with the destination it held
/// before. A prone infantry unit of a human-controlled owner whose type is
/// neither a fraidycat nor a cyborg asks (not forced) for *get-up* when the
/// non-empty destination assigned is the one it already had (the same cell
/// or the same object). A first or different destination and a cleared one
/// make no request. The request is not saved when refused.
pub fn destination_assigned(w: &mut World, id: ObjectId, before: Option<Target>) {
    if let Some(ob) = w.obj_mut(id) {
        // spec: S-0417 claim 17 (No path: a destination accepted after a failed search reaches the walk locomotor, which then has movement in progress until it is searched for; the destination it kept, given again, keeps the delay running, S-0212 claim 21, and its movement in progress is our design, S-0417 Open question 16)
        ob.path.failed_for = None;
    }
    // spec: S-0212 claim 21 (for the driving and walk locomotors a destination changed to a different one or cleared ends a running retry delay, so the first search for it is not held back)
    crate::path::destination_changed(w, id, before);
    // spec: S-0207 claim 27 (an assignment takes effect when it changes the order-level destination or a destination-refresh state is present)
    let effective = w.obj(id).is_some_and(|o| o.dest != before) || crate::refresh::pending(w, id);
    // spec: S-0207 claim 27 (a vehicle's effective assignment runs the dock test before the locomotors take the destination)
    let token = crate::refresh::begin(w, id, before);
    // spec: S-0207 claim 20 (a) (the teleporting locomotor takes a destination at once)
    crate::teleport::take_destination(w, id);
    // spec: S-0314 Terms and claim 22 (a destination that takes effect while a depot or tank bunker occupies the cell ends being settled)
    crate::depot::destination_given(w, id, effective);
    // spec: S-0503 claim 4 (the hovering locomotor takes a move: ρ = 1 when powered and not moving now)
    crate::hover::take_destination(w, id);
    // spec: S-0502 claim 16 (the jumpjet locomotor takes a destination point: the nearby-cell search, the move state and the re-ascent of a descending one)
    crate::jump::take_destination(w, id);
    // spec: S-0207 claim 25 (a driving or ship locomotor installs the supplied point at the assignment, before any path search)
    crate::drive::take_destination(w, id);
    // spec: S-0207 claim 27 (the refresh this assignment consumed is gone, unless the dock fallback newly enabled it here)
    crate::refresh::finish(w, id, token);
    let Some(o) = w.obj(id) else { return };
    // spec: S-0107 claim 15 (clearing a destination makes no request)
    let Some(new) = o.dest else { return };
    if o.kind != Kind::Infantry || !o.infantry.as_ref().is_some_and(|i| i.prone) {
        return;
    }
    // spec: S-0213 claim 21 (revision 34: repeating the cell it occupies requests get-up, unless the jumpjet own-cell filter of S-0502 claim 23 suppresses the assignment)
    if crate::jump::own_cell_filtered(w, id, new) {
        return;
    }
    // spec: S-0107 claim 15 (only the current destination assigned again; quirks.prone_get_up_repeated_destination off: any non-empty one)
    if w.rules().quirks.prone_get_up_repeated_destination && before != Some(new) {
        return;
    }
    let human = o.owner.is_some_and(|h| w.houses[usize::from(h.0)].human);
    let exempt = w
        .rules()
        .unit(o.kind, o.type_index)
        .is_some_and(|u| u.fraidycat || u.cyborg);
    // spec: S-0107 claim 15 (a human-controlled owner's unit, not a fraidycat or a cyborg, asks, not forced; claim 6 may refuse)
    if human && !exempt {
        request(w, id, Seq::GetUp, false);
    }
}

/// A start request (S-0107 claim 6). Returns whether the kind started.
pub fn request(w: &mut World, id: ObjectId, q: Seq, forced: bool) -> bool {
    let tick = w.tick;
    let speed = w.game_speed;
    let normalise = w.rules().quirks.speed_normalised_delays;
    // spec: S-0107 claim 6 (row 1: the requested kind's own entry has C = 0)
    if count(w, id, q) == 0 {
        return false;
    }
    // spec: S-0107 claim 6 (row 2: the current sequence is parachute and the unit is still falling; S-0522 claim 12)
    if w.obj(id).is_some_and(|o| {
        o.fall.falling && o.infantry.as_ref().and_then(|s| s.seq) == Some(Seq::Parachute)
    }) {
        return false;
    }
    let crawls = w
        .obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .is_some_and(|u| u.crawls);
    // spec: S-0107 claim 6 (row 3: go-prone for a type that cannot crawl)
    if q == Seq::GoProne && !crawls {
        return false;
    }
    let resulting = substitute(w, id, q);
    let wet = on_wet(w, id);
    let Some(s) = w.obj_mut(id).and_then(|o| o.infantry.as_mut()) else {
        return false;
    };
    // spec: S-0107 claim 7 (the water record is updated by every request that rows 1 to 3 do not turn down)
    s.wet = Some(wet);
    // spec: S-0107 claim 6 (row 4: the resulting kind is the current one)
    if s.seq == Some(resulting) {
        return false;
    }
    // spec: S-0107 claim 6 (row 5: not forced, and the current kind is not interruptible)
    if !forced && s.seq.is_some_and(|c| !c.table().0) {
        return false;
    }
    let (_, b, norm) = resulting.table();
    // spec: S-0107 claim 5 (normalised kinds take N(b, s); every other kind b)
    s.seq_delay = if norm && normalise {
        normalised(b, speed)
    } else {
        b
    };
    s.seq = Some(resulting);
    s.seq_t0 = tick;
    // spec: S-0107 Terms (the prone state begins with go-prone and ends with get-up or deploy)
    match resulting {
        Seq::GoProne => s.prone = true,
        Seq::GetUp | Seq::Deploy => s.prone = false,
        _ => {}
    }
    true
}

/// The per-tick sequence rules of one infantry unit, after its movement for
/// the tick (S-0107 claims 8 to 10 and 15), with no firing between them:
/// [`fear_turn`], then [`sequence_turn`]. The tick runs the firing rules
/// (S-0107 claim 11) between the two. `moving_now`: physically moving on
/// this tick; `in_progress`: has movement in progress.
pub fn update(w: &mut World, id: ObjectId, moving_now: bool, in_progress: bool, has_dest: bool) {
    fear_turn(w, id, in_progress, has_dest);
    sequence_turn(w, id, moving_now, in_progress);
}

/// The fear rules of a tick (S-0107 claim 15) and the parachute landing
/// request (State and ordering): the part of the sequence rules that comes
/// before the firing rules of the same tick (S-0107 State and ordering: a
/// sequence the fear rules start already holds for the firing rules).
pub fn fear_turn(w: &mut World, id: ObjectId, in_progress: bool, has_dest: bool) {
    let Some(o) = w.obj(id) else { return };
    let Some(u) = w.rules().unit(o.kind, o.type_index).cloned() else {
        return;
    };
    let human = o.owner.is_some_and(|h| w.houses[usize::from(h.0)].human);
    let Some(s) = o.infantry.clone() else { return };
    // Fear (S-0107 claim 15).
    // spec: S-0107 claim 15 (f′ = f − 1 when f > 0 and the type is not fearless)
    let f = s.fear;
    let f2 = if f > 0 && !u.fearless { f - 1 } else { f };
    let deploying = s.seq.is_some_and(Seq::is_deploy);
    if let Some(st) = w.obj_mut(id).and_then(|o| o.infantry.as_mut()) {
        st.fear = f2;
    }
    if f > 0 {
        // spec: S-0107 claim 15 (lie down at f′ ≥ 50, at rest for a human-controlled owner or an AI house, moving or not for the neutral and special houses; get up below 50)
        // Destination-driven get-up is `destination_assigned`.
        if !s.prone
            && !deploying
            && !u.fraidycat
            && (!human || (!has_dest && !in_progress))
            && f2 >= 50
        {
            request(w, id, Seq::GoProne, false);
        } else if s.prone && !deploying && f2 < 50 {
            request(w, id, Seq::GetUp, false);
        }
    }
    // spec: S-0107 State and ordering (on every tick a unit whose current sequence is parachute and that is no longer falling asks for stand, forced); S-0522 claim 12
    if w.obj(id).is_some_and(|o| {
        !o.fall.falling && o.infantry.as_ref().and_then(|s| s.seq) == Some(Seq::Parachute)
    }) {
        request(w, id, Seq::Stand, true);
    }
}

/// The end of the attack state (S-0107 claim 11) with the request of a
/// not-ready fire frame or a change of target: *deployed* when deploying,
/// *prone* when prone, *stand* otherwise, not forced (claim 6 decides).
/// With `ask` false only the state ends.
pub(crate) fn end_attack(w: &mut World, id: ObjectId, ask: bool) {
    let Some(s) = w.obj_mut(id).and_then(|o| o.infantry.as_mut()) else {
        return;
    };
    s.attack = false;
    let (prone, deploying) = (s.prone, s.seq.is_some_and(Seq::is_deploy));
    if ask {
        // spec: S-0107 claim 11 (the unit asks, not forced, for deployed when deploying, prone when prone and stand otherwise)
        let q = if deploying {
            Seq::Deployed
        } else if prone {
            Seq::Prone
        } else {
            Seq::Stand
        };
        request(w, id, q, false);
    }
}

/// A change of an infantry unit's target (S-0107 r46 claim 11, *Changes of
/// target*; S-0215 r118 claim 11, *Other endings*): the attack state ends,
/// and the unit asks (not forced) for *deployed*, *prone* or *stand*,
/// whether or not it was in the attack state.
pub(crate) fn target_changed(w: &mut World, id: ObjectId) {
    if !w
        .obj(id)
        .is_some_and(|o| o.kind == Kind::Infantry && o.infantry.is_some() && o.health > 0)
    {
        return;
    }
    // spec: S-0107 r46 claim 11 (a change of target ends the attack state and asks, not forced, for deployed, prone or stand, in or out of the attack state) and S-0215 r118 claim 11 (Other endings)
    end_attack(w, id, true);
    if let Some(o) = w.obj_mut(id) {
        let t = o.target;
        if let Some(s) = o.infantry.as_mut() {
            s.watch = t;
        }
    }
}

/// Takes the unit's target as the baseline of its next change-of-target
/// test ([`watch_test`]).
pub(crate) fn watch_set(w: &mut World, id: ObjectId) {
    if let Some(o) = w.obj_mut(id) {
        let t = o.target;
        if let Some(s) = o.infantry.as_mut() {
            s.watch = t;
        }
    }
}

/// The change-of-target test (S-0107 r46 claim 11, *Changes of target*): a
/// unit whose target differs from the baseline (another target, none, or
/// one gained when it had none) has had its target changed
/// ([`target_changed`]). The baseline becomes the target it holds now.
///
/// The rules that change an infantry unit's target call it right after
/// the change, so the ending and the request come at the moment the change
/// is made, in whichever actor's turn: an order, a retaliation or other
/// override (`acquire::override_attack_on`), a berserk hit's drop
/// (`psychic`), a guard reset (`econ::power::guard_reset`, a takeover's
/// included), a release, a revert that puts the earlier target back, an
/// open-topped transport's clear, and the unit's own turn at its places in
/// `tick::object_turn`. The target's leaving play calls [`target_changed`]
/// itself (`held::holders_drop`). Our engine also tests at the start of the
/// unit's turn, before any start of its update, for a rule that writes a
/// target without calling it.
pub(crate) fn watch_test(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    let Some(s) = o.infantry.as_ref() else { return };
    // spec: S-0107 r46 claim 11, Changes of target (a target that differs from the one held, the loss of the target, and the gaining of a target by a unit that had none)
    let changed = s.watch != o.target;
    if changed {
        target_changed(w, id);
    } else {
        watch_set(w, id);
    }
}

/// The attack-move rule's exception (S-0215 r118 claim 11, S-0309 claim
/// 23): a discard of the target with no other found, or a remembered target
/// put back, is no change of target. `before` is the target the unit held
/// just before that rule wrote it; when no change is pending since the
/// baseline, the baseline follows the rule's write.
pub(crate) fn watch_carry(w: &mut World, id: ObjectId, before: Option<Target>) {
    let Some(o) = w.obj_mut(id) else { return };
    let t = o.target;
    if let Some(s) = o.infantry.as_mut()
        && s.watch == before
    {
        // spec: S-0215 r118 claim 11 (the attack-move rule of S-0309 claim 23 discards a target, and puts a remembered target back, without the change-of-target ending)
        s.watch = t;
    }
}

/// The stage-delay-0 ending (S-0107 r40 claim 11): a unit in the attack
/// state whose update began with a current sequence of stage delay 0
/// (`began_frozen`, taken at the start of its turn) leaves the state and
/// asks (not forced) for *stand*. S-0215 r118 claim 11 places it after the
/// update's processing and before its late start.
pub(crate) fn stage_delay_zero_end(w: &mut World, id: ObjectId, began_frozen: bool) {
    let Some(s) = w.obj(id).and_then(|o| o.infantry.as_ref()) else {
        return;
    };
    if !(began_frozen && s.attack) {
        return;
    }
    // spec: S-0215 r118 claim 11 (Other endings: a current sequence whose stage delay is 0, after the update's processing and before its late start, so only the first start sees the state) and S-0107 r40 claim 11 (a tick that begins with a current sequence whose stage delay is 0 ends the attack state; the unit also asks, not forced, for stand)
    if let Some(s) = w.obj_mut(id).and_then(|o| o.infantry.as_mut()) {
        s.attack = false;
    }
    request(w, id, Seq::Stand, false);
}

/// Whether the unit's update begins with a current sequence whose stage
/// delay is 0 (S-0107 claim 11; only *stand* and guard have one, T1). A
/// unit with no current sequence reads the delay 0 that its stage rule
/// uses then (claim 4), as before.
pub(crate) fn begins_frozen(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.infantry.as_ref())
        .is_some_and(|s| s.seq_delay == 0)
}

/// The completion follow-up (S-0107 claim 8) and the movement rule (claim
/// 10) of a tick: the part of the sequence rules that comes after the
/// firing rules of the same tick (S-0107 State and ordering: the fear and
/// firing rules are judged against the sequence current before that tick's
/// completion follow-up; a follow-up already holds for the movement rule).
pub fn sequence_turn(w: &mut World, id: ObjectId, moving_now: bool, in_progress: bool) {
    let tick = w.tick;
    let keeps = w.rules().quirks.set_facing_keeps_target;
    let Some(o) = w.obj(id) else { return };
    if w.rules().unit(o.kind, o.type_index).is_none() || o.infantry.is_none() {
        return;
    }
    // Completion follow-up (S-0107 claim 8 and State and ordering).
    let s = w
        .obj(id)
        .and_then(|o| o.infantry.clone())
        .unwrap_or_default();
    let c = s.seq.map_or(0, |q| count(w, id, q));
    let complete = s.seq.is_none() || s.stage(tick) >= c;
    if complete {
        let cur = s.seq;
        if s.prone
            && matches!(cur, Some(Seq::FireProne | Seq::SecondaryFireProne))
            && !w.rules().quirks.prone_attack_state_persists
            && let Some(st) = w.obj_mut(id).and_then(|o| o.infantry.as_mut())
        {
            // spec: S-0107 r40 claim 11 and Quirks (quirks.prone_attack_state_persists off: a prone unit's attack state ends when fire-prone, or secondary-fire-prone that is not repeated for an airstrike, completes; no airstrike repeat is built, seam)
            st.attack = false;
        }
        let follow = match cur {
            Some(Seq::Deploy) => Some(Seq::Deployed),
            Some(Seq::Undeploy) => Some(Seq::Stand),
            Some(Seq::AirDeathStart) => Some(Seq::AirDeathFall),
            Some(Seq::Die1 | Seq::Die2 | Seq::Die3 | Seq::Die4 | Seq::Die5)
            | Some(Seq::WetDie1 | Seq::WetDie2 | Seq::AirDeathEnd)
            | Some(Seq::Parachute) => None,
            _ => {
                // spec: S-0107 State and ordering ("not moving": no movement in progress)
                let moving = in_progress;
                Some(match (moving, s.prone, cur) {
                    (false, _, Some(Seq::Deployed | Seq::DeployedFire | Seq::DeployedIdle)) => {
                        Seq::Deployed
                    }
                    (false, true, _) => Seq::Prone,
                    (false, false, _) => Seq::Stand,
                    (true, true, _) => Seq::Crawl,
                    (true, false, _) => Seq::Walk,
                })
            }
        };
        // spec: S-0107 claim 9 (a completed kind's declared direction is snapped in the "any other kind" rows)
        let generic = !matches!(
            cur,
            Some(
                Seq::Die1
                    | Seq::Die2
                    | Seq::Die3
                    | Seq::Die4
                    | Seq::Die5
                    | Seq::WetDie1
                    | Seq::WetDie2
                    | Seq::AirDeathEnd
                    | Seq::Deploy
                    | Seq::Undeploy
                    | Seq::AirDeathStart
                    | Seq::Parachute
                    | Seq::Shovel
            )
        );
        if generic
            && let Some(q) = cur
            && let Some(d) = entry_dir(w, id, q)
        {
            // spec: S-0107 claim 9 (body facing set at once to D × 8192)
            if let Some(o) = w.obj_mut(id) {
                o.body.set(u16::from(d) * 8192, tick, keeps);
            }
        }
        if let Some(q) = follow {
            request(w, id, q, true);
        }
    }
    // Movement rule (S-0107 claim 10).
    let s = w
        .obj(id)
        .and_then(|o| o.infantry.clone())
        .unwrap_or_default();
    if !moving_now {
        let rep = match s.seq {
            Some(Seq::Walk | Seq::Fly | Seq::Hover) => Some(Seq::Stand),
            Some(Seq::Crawl) => Some(Seq::Prone),
            Some(Seq::Swim) => Some(Seq::Tread),
            _ => None,
        };
        if let Some(q) = rep {
            request(w, id, q, false);
        }
    } else {
        if let Some(fraction) = crate::jump::speed_fraction(w, id) {
            // spec: S-0107 claim 10 (moving now: a jumpjet asks for nothing while in the attack state, otherwise for hover at a speed fraction of at most 0.8 and fly above it; the double nearest 0.8)
            if !s.attack {
                let q = if fraction.le(crate::rules::nearest_double("0.8")) {
                    Seq::Hover
                } else {
                    Seq::Fly
                };
                request(w, id, q, false);
            }
        } else {
            // spec: S-0107 claim 10 (moving: crawl when prone, walk otherwise)
            let q = if s.prone { Seq::Crawl } else { Seq::Walk };
            request(w, id, q, false);
        }
    }
}

/// The idle timer length T of S-0107 F4 from a drawn unit-interval value.
pub fn idle_length(w: &World, u: F64) -> i32 {
    let mode = w.rules().mode;
    let r = w.rules().general.idle_frequency;
    // spec: S-0107 F4 (P = R × 1800, Q = R × 450, T = trunc(Q + (P − Q) × u), all rounded by the mode)
    let p = r.mul(F64::from_i64(1800, mode), mode);
    let q = r.mul(F64::from_i64(450, mode), mode); // spec: S-0107 F4 (Q = R × 450)
    q.add(p.sub(q, mode).mul(u, mode), mode).to_i32_common()
}

/// An idle-action attempt (S-0107 claim 14), from a behaviour processing.
pub fn idle_attempt(w: &mut World, id: ObjectId, in_progress: bool) {
    let tick = w.tick;
    let q = w.rules().quirks;
    let mode = w.rules().mode;
    let keeps = q.set_facing_keeps_target;
    let Some(o) = w.obj(id) else { return };
    let Some(s) = o.infantry.clone() else { return };
    let is_cow = w
        .rules()
        .unit(o.kind, o.type_index)
        // spec: S-0107 claim 14, Import mapping (Hard-coded identifiers: the cow is the infantry
        // type whose registry name is COW, compared case-insensitively; other names do not count)
        .is_some_and(|u| u.name.eq_ignore_ascii_case("COW"));
    // spec: S-0107 claim 14 (conditions: timer run out, no movement in progress, not prone, not attacking, stand/guard/tread)
    let goes = tick >= s.idle_until
        && !in_progress
        && !s.prone
        && !s.attack
        && matches!(s.seq, Some(Seq::Stand | Seq::Guard | Seq::Tread));
    if !goes {
        return;
    }
    // spec: S-0107 F4 (the timer draw: R(0, 2147483646) times c)
    let u = w.rng.unit(mode);
    let t = idle_length(w, u);
    if let Some(st) = w.obj_mut(id).and_then(|o| o.infantry.as_mut()) {
        st.idle_until = tick.wrapping_add(t);
    }
    // S-0107 claim 14: an owner that is not human-controlled is one of the neutral and special
    // houses in our engine (an AI house counts as human-controlled, glossary *AI house*).
    let fraidycat = w
        .obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .is_some_and(|u| u.fraidycat);
    let not_human = w
        .obj(id)
        .and_then(|o| o.owner)
        .is_some_and(|h| !w.houses[usize::from(h.0)].human);
    let idle_scatter = crate::scatter::ScatterRequest {
        threat: None,
        forced: true,
        anywhere: false,
    };
    // spec: S-0107 claim 14 (a fraidycat above fear 50 whose owner is not human-controlled scatters, and nothing else happens: the timer draw only)
    if fraidycat && not_human && s.fear > 50 {
        crate::scatter::request(w, id, idle_scatter);
        return;
    }
    // spec: S-0107 claim 14 (r and e drawn from 0 … 10)
    let mut r = w.rng.ranged(0, 10, q.ranged_draw_half_span);
    if is_cow && q.cow_idle {
        let e = w.rng.ranged(0, 10, q.ranged_draw_half_span); // spec: S-0107 claim 14 (the cow's e drawn from 0 … 10)
        // spec: S-0107 claim 14 (the cow: when e < 5 the outcome of r = 8)
        if e < 5 {
            r = 8;
        }
    }
    match r {
        // spec: S-0107 claim 14 (r = 1, 2 or 7: idle B)
        1 | 2 | 7 => {
            request(w, id, Seq::IdleB, false);
        }
        // spec: S-0107 claim 14 (r = 3 … 5: idle A)
        3..=5 => {
            request(w, id, Seq::IdleA, false);
        }
        // spec: S-0107 claim 14 (δ × 8192)
        6 | 8..=10 => {
            // spec: S-0107 claim 14 (r = 6 or 8 … 10: a new facing)
            // spec: S-0107 claim 14 (δ drawn from 0 … 7, the body facing set at once to δ × 8192)
            let d = w.rng.ranged(0, 7, q.ranged_draw_half_span);
            if let Some(o) = w.obj_mut(id) {
                o.body.set((d as u16) * 8192, tick, keeps); // spec: S-0107 claim 14 (δ × 8192)
            }
            // spec: S-0107 claim 14 (r = 8 also scatters the cow, and a fraidycat whose owner is not human-controlled)
            if r == 8 && ((is_cow && q.cow_idle) || (fraidycat && not_human)) {
                crate::scatter::request(w, id, idle_scatter);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0107 F3 (the normalisation table and the formula from b = 5)
    #[test]
    fn normalisation() {
        assert_eq!(normalised(0, 3), 0);
        assert_eq!(normalised(3, 0), 5);
        assert_eq!(normalised(3, 6), 2);
        assert_eq!(normalised(4, 0), 7);
        assert_eq!(normalised(5, 0), 40);
        assert_eq!(normalised(2, 7), 1);
    }

    // spec: S-0107 claim 4 (stage timing)
    #[test]
    fn stage() {
        let s = InfantryState {
            seq: Some(Seq::Walk),
            seq_t0: 10,
            seq_delay: 3,
            ..Default::default()
        };
        assert_eq!(s.stage(10), 0);
        assert_eq!(s.stage(12), 0);
        assert_eq!(s.stage(13), 1);
        let z = InfantryState {
            seq_delay: 0,
            ..Default::default()
        };
        assert_eq!(z.stage(100), 0);
    }
}
