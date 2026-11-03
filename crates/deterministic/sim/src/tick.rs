//! The tick pipeline (S-0203): the parts of a tick in their fixed places.
//!
//! The parts that are not built yet (chrono flashes, diving aircraft,
//! teams and pulses) are empty seams in their places
//! of S-0203 claim 1, so that their draws land where the original takes them
//! once they are written. Resource growth (row 9), the production lines (row
//! 6) and the houses (row 7) run the M5a topics (`crate::econ`). Radiation
//! maintenance (row 13, `crate::radiation`) takes no draws.

use crate::probe::{Phase, Probe};
use crate::rules::{Kind, Locomotor};
use crate::world::{ObjectId, World};
use crate::{drive, infantry, occupancy, orders, process, turret, walk};

/// The number of ticks that have run: the tick number, and one more once
/// the final tick has run, since that tick adds nothing to the tick number
/// (S-0201 claim 1).
pub fn ticks_run(w: &World) -> i32 {
    // spec: S-0201 claim 1 (the final tick runs but leaves the tick number at itself)
    w.tick.wrapping_add(i32::from(w.econ.end.finished))
}

/// Runs one tick (S-0203 claim 1, the draw order of its rows).
pub fn step(w: &mut World) {
    step_with(w, &mut ());
}

/// [`step`], telling `p` where the tick is (the tools' benchmark,
/// `crate::probe`); `p` reads nothing and changes nothing.
pub fn step_with<P: Probe>(w: &mut World, p: &mut P) {
    let on = p.on();
    let mark = |p: &mut P, phase| {
        if on {
            p.mark(phase);
        }
    };
    mark(p, Phase::Triggers);
    // spec: S-0203 claim 1 (draw order: row 1, 9, 2, 10, 11, 12, 3, 14, then the turn pass, row 5; row 13, radiation maintenance, takes no draws)
    // spec: S-0203 row 1 and S-0011 claim 6 (the poll of every polled tag, at the start of the tick)
    crate::triggers::poll(w);
    // spec: S-0203 row 9 (growth and spread of resources, S-0401)
    mark(p, Phase::Resources);
    crate::econ::resource::part(w);
    // Row 2 (planted-bomb maintenance, S-0519 claim 10; no draws) is presentation only (visibility, icon,
    // sound): nothing of it is state. Timed bombs detonate in their carrier's turn in row 5 (S-0203 claim 1,
    // S-0519 claim 4), not here.
    // spec: S-0506 claim 26 and S-0203 claim 1 row 17 (the shared kamikaze re-order cycle: once per tick, outside object turns, after planted-bomb maintenance (row 2), before diving aircraft (row 10) and the turn pass (row 5); it takes no draws)
    mark(p, Phase::Kamikaze);
    crate::spawn::kamikaze_orders(w);
    // Rows 10 and 11 have no topic yet: empty seams.
    // spec: S-0203 claim 1 row 12 and S-0615 claim 13 (disc lasers, the most recently fired first, after rows 10 and 11 and before row 3 and the turn pass)
    mark(p, Phase::Discs);
    crate::disc::update(w);
    // spec: S-0104 claims 4 and 5 (the ambient fade and the nuke flash's phase change; the fade work may run anywhere in the tick and takes no draws: here, just before the domination update that reads it)
    mark(p, Phase::Lighting);
    crate::lighting::update(w);
    // spec: S-0612 Terms (the domination update runs in S-0203 row 3, before the storm's own update of that tick) and claims 4 to 7
    mark(p, Phase::Domination);
    crate::domination::update(w);
    // spec: S-0203 claim 1 row 3 (the weather storm's own update and bolt strikes, after rows 10 to 12 and before row 14 and the turn pass; S-0513 claims 14 to 19)
    mark(p, Phase::Storm);
    crate::superweapons::storm::update(w);
    // spec: S-0203 claim 1 row 13 and S-0525 claims 5, 6 and 11 (radiation maintenance: after the storm, before the turn pass; the most recently created field first; no draws)
    mark(p, Phase::Radiation);
    crate::radiation::maintenance(w);
    // Rows 14 (and row 4) have no topic yet: empty seams.
    // spec: S-0203 claim 1 (other work once per tick, no draws: the regrowth of shroud; S-0409 claim 18)
    // spec: S-0409 claim 18 and State and ordering (a regrowth step due in the same tick as the re-cover sweep, tick 0 included, has its effect before the sweep)
    mark(p, Phase::ShroudRegrowth);
    crate::econ::shroud::regrowth(w);
    // spec: S-0409 claim 13 (the re-cover sweep, before the turn pass of its tick)
    mark(p, Phase::ShroudSweep);
    crate::econ::shroud::sweep(w);
    mark(p, Phase::TurnPass);
    turn_pass(w, p);
    // spec: S-0203 claim 1 (after the turn pass: rows 15, 16, 6, 7, then the orders, row 8)
    // spec: S-0203 claim 1 row 15 and S-0612 claim 8 (blast rings, after the turn pass; no draws)
    mark(p, Phase::Rings);
    crate::domination::rings(w);
    // spec: S-0203 claim 1 row 16 and S-0520 claim 4 (crate expiry and replacement, after the turn pass and before the production lines)
    mark(p, Phase::Crates);
    crate::crates::update(w);
    // spec: S-0203 claim 8 and row 6 (the production lines see the whole turn pass)
    mark(p, Phase::Production);
    crate::econ::production::part(w);
    // spec: S-0203 claim 8 and row 7 (the houses, in house order, after the production lines)
    mark(p, Phase::Houses);
    crate::econ::houses::part(w);
    // spec: S-0203 claim 7 and S-0213 claim 1 (orders take effect after the turn pass)
    mark(p, Phase::Orders);
    orders::apply_pending(w);
    // spec: S-0011 claim 11 (a one-time tag that fired is deleted at the end of the tick)
    mark(p, Phase::EndTick);
    crate::triggers::end_tick(w);
    if w.econ.end.over == Some(w.tick) || w.econ.end.finished {
        // spec: S-0201 claim 1 and S-0203 State and ordering (the final tick runs in full but does not add 1: afterwards the tick number is the last tick's; no next tick runs, S-0414 claim 8)
        w.econ.end.finished = true;
    } else {
        // spec: S-0203 claim 1 and S-0201 claim 1 (after the last part the tick number grows by 1, except in the final tick)
        w.tick = w.tick.wrapping_add(1);
    }
    // Derived data, not state: the regions are brought up to date between
    // ticks, so that input rules can read them (crate::targeting).
    mark(p, Phase::Regions);
    crate::regions::ensure(w);
    mark(p, Phase::Done);
}

/// The turn pass (S-0203 claims 2 to 4, 10 and 11).
fn turn_pass<P: Probe>(w: &mut World, p: &mut P) {
    let on = p.on();
    w.update.begin_pass();
    while let Some(id) = w.update.next_turn() {
        if on {
            p.mark(Phase::Turn(crate::probe::classify(w, id)));
        }
        object_turn(w, id, p);
        if on {
            p.mark(Phase::StrikeSweep);
        }
        // spec: S-0519 claim 19 (revision 3: a designator's change of target cancels its strike in the tick it applies, also a change after the designator's own turn, not at its next look or turn)
        crate::strike::sweep(w);
        if on {
            p.mark(Phase::TurnPass);
        }
    }
    w.update.end_pass();
}

/// One object's turn. Within a turn: the behaviour part (S-0215 claims 2,
/// 11 and 33), then the locomotor, then the infantry sequence rules
/// (S-0107 claims 8 to 10 and 15 run after the movement of the tick).
fn object_turn<P: Probe>(w: &mut World, id: ObjectId, p: &mut P) {
    let on = p.on();
    let t = w.tick;
    if w.projectiles.contains_key(&id) {
        // spec: S-0203 claim 5 (a shot is resolved in its projectile's own turn)
        crate::combat::projectile::turn(w, id);
        return;
    }
    if w.anims.anims.contains_key(&id) {
        // spec: S-0109 claim 5 and S-0203 claim 2 (an animation updates in its own turn)
        crate::anims::turn(w, id);
        return;
    }
    if w.gas.systems.contains_key(&id) {
        // spec: S-0613 claim 6 and S-0203 claim 2 (a particle system takes one turn per tick)
        crate::gas::system_turn(w, id);
        return;
    }
    if w.gas.anims.contains_key(&id) {
        // spec: S-0109 claim 5 (an animation updates in its own turn)
        crate::gas::anim_turn(w, id);
        return;
    }
    if w.walkers.contains_key(&id) {
        // spec: S-0615 claim 4 and S-0109 claim 5 (a burning walker updates in its own turn)
        crate::walker::turn(w, id);
        return;
    }
    if w.mutation.makers.contains_key(&id) {
        // spec: S-0611 claim 8 and S-0109 claim 5 (an infantry-making animation updates in its own turn)
        crate::mutation::turn(w, id);
        return;
    }
    if w.domination.anims.contains_key(&id) {
        // spec: S-0612 claims 1 and 8 and S-0109 claim 5 (a domination animation updates in its own turn)
        crate::domination::anim_turn(w, id);
        return;
    }
    if w.depot_anims.anims.contains_key(&id) {
        // spec: S-0108 claim 20 and S-0109 claim 5 (a repair depot's slot animation updates in its own turn)
        crate::depot_anim::anim_turn(w, id);
        return;
    }
    if w.pieces.contains_key(&id) {
        // spec: S-0317 claim 6 and S-0203 claim 2 (a debris piece takes its turn in the update order)
        crate::pieces::turn(w, id);
        return;
    }
    if w.waves.contains_key(&id) {
        // spec: S-0607 claim 21 (a magnetic wave updates in its own turn of the update order)
        crate::lift::wave_turn(w, id);
        return;
    }
    let Some(o) = w.obj(id) else { return };
    // A building in a delayed-kill countdown is killed but alive (S-0302 claim 16): it takes its whole turn.
    let dead = crate::combat::damage::dead(o);
    let Some(o) = w.obj(id) else { return };
    let kind = o.kind;
    if let Some(ob) = w.obj_mut(id) {
        ob.last_turn = t;
    }
    if dead {
        // spec: S-0309 claim 15 (the expected remaining health changes in the object's own turn)
        crate::combat::acquire::expected_health_turn(w, id);
        // spec: S-0513 claim 29 (the protection tint's phases advance in the object's turns; its draw is game state)
        crate::superweapons::protect::tint_turn(w, id);
        // spec: S-0519 State and ordering (the flash count and the mark sequence step right after the protection sequence)
        crate::mark::turn(w, id);
        if kind == Kind::Building {
            // spec: S-0312 claim 21 (a building awaiting removal: the general part of its turn, then its removal)
            process::turn(w, id);
            crate::combat::death::removal_turn(w, id);
            return;
        }
        if w.obj(id).is_some_and(|o| o.rocket.is_some()) {
            // spec: S-0506 claim 20 (a destroyed rocket missile above the terrain keeps the rocket-flight movement mode)
            crate::rocket::tick(w, id);
            return;
        }
        if crate::jump::crashing(w, id) {
            // spec: S-0502 claim 18 and S-0312 claim 16 (a killed jumpjet that started to crash keeps its jumpjet movement until it hits the ground)
            crate::jump::tick(w, id);
            return;
        }
        if crate::air::crashing(w, id) {
            // spec: S-0501 claim 46 and S-0519 claim 4 (a bomb on an aircraft falling from the fall-back crash stays attached: its timer test comes before the aircraft's movement in its turn)
            crate::bomb::turn(w, id);
            // spec: S-0501 claim 9 (a crashing aircraft keeps its flying movement until it hits the ground)
            crate::air::tick(w, id);
            crate::air::end_turn(w, id);
            return;
        }
        if crate::naval::sinking(w, id) {
            // spec: S-0312 claims 15 and 32 (a vessel destroyed while sinking stays in play at 0 health because it is sinking); S-0504 claim 4 (each own update of a sinking vessel descends, and it leaves play below −400 leptons); S-0504 claim 13 (no ordinary living updates are asserted for it, so it takes no others)
            crate::naval::sinking_turn(w, id);
            return;
        }
        // A killed object that stays in play (an infantry unit in its death sequence) only plays it out.
        crate::combat::death::infantry_death_turn(w, id);
        return;
    }
    // spec: S-0616 claim 1 (a deactivated vehicle's spark step comes first in its turn, before every other draw of that turn, also in turns that end early for another reason)
    crate::emitters::deactivated_sparks(w, id);
    // spec: S-0309 claim 27 (DECIDED: a held object a revert gave back that is off the map, or destroyed and still in play, is dropped at the start of the holder's next turn; no draw); S-0513 r89 claim 39a (The firer: an infantry or vehicle firer whose beam is attached to a departed member keeps it; a building firer and a firer beyond range: interim Q-2376, kept too; an aircraft firer: the aircraft specs')
    crate::combat::held::turn_start_drop(w, id);
    let member = crate::superweapons::warp::is_member(w, id);
    if member {
        // spec: S-0513 claim 39a (Q-2316: a member's sparkle, with or without a beam, decided by its state before the turn's warp step) and claim 51 (an intrinsic exception made before the warp work, its draws before the warp step's); its order against the erasure update is open (S-0513 Open questions, S-0514's): interim Q-2377, before it, as in an ordinary turn
        crate::temporal::turn_sparkle(w, id);
    }
    // spec: S-0513 claim 39a and S-0203 claim 4 (a warp member's erasure update comes before its warp step in each of its turns; a member that update removes takes no warp step, the no-step rule)
    let erasure_made = crate::temporal::member_update(w, id);
    if erasure_made && w.obj(id).is_none() {
        return;
    }
    // spec: S-0513 State and ordering and T2 (a warp member's warp runs in its own updates); S-0513 claims 48 and 51 and S-0203 claim 4 (a member still leaving takes no other part of its turn, confirmed; engine design for the other stages before the arrival hold: no ordinary healing, behaviour, movement, firing or turret turning, and S-0203 claim 4 revision 32: the warp update is the whole member turn, with the intrinsic exceptions)
    if crate::superweapons::warp::turn(w, id) {
        // spec: S-0513 claim 39a (Q-2316: the drop after the warp step of a member still being erased or cut short) and claim 51
        crate::temporal::member_drop(w, id);
        // spec: S-0203 claim 4 and S-0301 claim 11 (the reload countdown's elapsed time still passes in an excluded warp turn; no grant, no restart)
        crate::combat::fire::reload_elapsed(w, id);
        return;
    }
    // spec: S-0608 claims 12 and 15 (Q-1440: a vehicle drainer's position check samples its position before that turn's behaviour start, behaviour processing, movement and firing, and before its erasure exit; the warp-only stages, which returned above, exclude it, S-0513 claim 51; a pulse-disabled drainer still checks)
    crate::drain::position_check(w, id);
    if !crate::temporal::cut_short(w, id) {
        // spec: S-0608 claims 11 and 15 (the alliance test precedes behaviour processing, its shots, movement, separate firing and the controller's overload update; the erasure exit skips it, S-0513 claim 39a's cut-short phasing-in turns included)
        crate::drain::alliance_test(w, id);
    }
    // spec: S-0203 claim 4 (expected-health bookkeeping and protection-brightness phase processing are ordinary work: suppressed through arrival effects, eligible in the hold after its hold-end test, with no catch-up)
    // spec: S-0309 claim 15 (the expected remaining health changes in the object's own turn)
    crate::combat::acquire::expected_health_turn(w, id);
    // spec: S-0513 claim 29 (the protection tint's phases advance in the object's turns; its draw is game state)
    crate::superweapons::protect::tint_turn(w, id);
    // spec: S-0519 State and ordering (the flash count and the mark sequence step right after the protection sequence)
    crate::mark::turn(w, id);
    // The berserk countdown (S-0610 claim 5) runs in the behaviour part, after the first per-tick start (`process::head`).
    // spec: S-0601 claim 16 (revision 9: the overload update runs in exactly the updates in which the berserk countdown runs: not in an excluded update, S-0215 claim 2's being erased (a tunnel is not built), nor in a warp-only update, which returned above, S-0513 claim 51 and S-0203 claim 4)
    // spec: S-0610 r12 claim 5 (the excluded updates include a superweapon-warp member's phasing-in updates that S-0215 claim 2 excludes, S-0513 claim 39a)
    let early = crate::mind::unlimited(w, id) && !crate::temporal::cut_short(w, id);
    if early {
        // spec: S-0601 claim 16 and State and ordering (revision 9: the overload update follows the berserk countdown of S-0610 claim 5, which follows the first per-tick start, and precedes self-healing, behaviour processing, movement and firing; so an unlimited controller's first per-tick start and countdown come here, ahead of its self-healing, an order S-0610 claim 6 and S-0314 claim 9 leave to the engine)
        process::head(w, id);
        crate::mind::overload_turn(w, id);
        // spec: S-0601 claim 16 (a lethal pulse ends the update's remaining work: no self-healing, behaviour, movement or firing follows; for a controller that is not a vehicle the same rule, unmeasured)
        if w.obj(id).is_none_or(crate::combat::damage::dead) {
            return;
        }
    }
    // spec: S-0504 claim 4 (a sinking vessel's own update first descends, and may leave play at once)
    if crate::naval::sinking_turn(w, id) {
        return;
    }
    // spec: S-0314 claims 9 and 10 (self-healing, in the object's own turn) and claim 19 (a sinking vehicle still self-heals; health does not end sinking)
    crate::combat::heal::turn(w, id);
    let sinking = crate::naval::sinking(w, id);
    // spec: S-0511 claim 10 (a dropped object falls on each of its own updates; the fall and any landing hit precede that turn's due behaviour processing, ordinary locomotion advance and firing check)
    let fell = crate::fall::falling(w, id) && crate::fall::turn(w, id);
    if w.obj(id).is_none_or(crate::combat::damage::dead) {
        return;
    }
    // spec: S-0514 claims 14, 17 and 18 (an attached attacker lets go on arriving in a new cell; a target being erased takes its erasure update and sparkle in its own turn and skips ordinary movement, turning, firing and behaviour; a building skips behaviour processing and firing; a phasing-in unit sparkles)
    crate::temporal::holder_turn(w, id);
    if !member {
        // spec: S-0514 claim 18 (one sparkle per turn of an object being erased, phasing in or both); a warp member's came before its warp step (S-0513 claim 39a)
        crate::temporal::turn_sparkle(w, id);
    }
    if crate::temporal::turn(w, id, !erasure_made) {
        // spec: S-0525 claims 9 and 11 (the radiation hit in the object's own turn, after its self-healing; S-0514 claim 17's erasure skips behaviour, movement, turning and firing, not the hit)
        crate::radiation::hit_turn(w, id);
        return;
    }
    if w.obj(id).is_none() {
        return;
    }
    if on {
        p.mark(Phase::Behaviour);
    }
    let behaves = early || !crate::temporal::building_erased(w, id);
    if behaves && !early {
        // The behaviour start (the first per-tick start and the countdown); an unlimited controller ran it before its overload update.
        process::head(w, id);
    }
    if behaves {
        // spec: S-0309 claim 29 (Place in the turn, revision 94, Q-2261 answered: after the turn's first per-tick start of an infantry unit, vehicle or building, S-0215 claim 11, so the behaviour it makes current decides; after a falling holder's fall step and landing hit; before the behaviour processing, its searches, the per-tick search, the movement and the firing step, and before a building's pending delayed shot counts down; an aircraft, which has no start here, before its processing, the behaviour current at its turn's start deciding) and claim 25 part k (the allied drop at that place, ahead of the check); S-0513 claim 51 with S-0203 claim 4 revision 42 (both suppressed in the warp turns above; in the arrival hold after the hold-end test); a turn cut short returned above (S-0215 claim 2) or is a building being erased (`behaves` false). The berserk countdown, which also follows the first per-tick start (S-0610 claim 5), comes before it here; the order changes no outcome, since a countdown that ends berserk drops the target itself (S-0610 claim 5) and one that does not leaves the holder berserk for claim 25 part k's keep either way
        crate::combat::held::turn(w, id);
    }
    if kind == Kind::Building && w.obj(id).is_some() {
        // spec: S-0515 claim 5 (the income cycle, in the building's own turn) and S-0608 claim 15 (Q-1440: the payout follows the due periodic income, so income available in that turn can be taken by it)
        crate::structures::tech::turn(w, id);
        // spec: S-0608 claims 7 and 15 (Q-1440: a drained resource-destination building's payout, in its own turn after its behaviour start and periodic income and before its behaviour processing, behaviour-issued firing, sale progression and the later health-change power recheck; claim 10: not while being erased or materialising)
        crate::drain::building_turn(w, id);
    }
    if behaves {
        // The behaviour processing.
        process::tail(w, id);
    }
    if on {
        p.mark(Phase::BeforeMove);
    }
    if w.obj(id).is_none() {
        return;
    }
    // spec: S-0616 claim 1 (Q-1404 answered: the damage-sparks draws come after the due behaviour processing with its shots, the mind-control overload and the self-healing, and before the radiation hit, ordinary movement, the parasite step and the separate infantry firing)
    crate::emitters::damage_sparks(w, id);
    // spec: S-0525 claims 9 and 11 (the radiation hit, in the object's own turn after its self-healing) and S-0616 claim 1 (after the damage sparks, so after the behaviour processing); a kill ends the turn as S-0312 says
    crate::radiation::hit_turn(w, id);
    if w.obj(id).is_none_or(crate::combat::damage::dead) {
        return;
    }
    // spec: S-0607 claim 12 (row 2: a mobile lifter that accepted another non-empty destination by any rule this turn lets go)
    crate::lift::lifter_destination(w, id);
    if kind == Kind::Building {
        // spec: S-0609 claims 9 and 14 (a gattling building's animation advance and its passive spin-down follow the tick's behaviour processing)
        crate::gattling::building_turn(w, id);
        // spec: S-0108 claims 12 and 20 (a repair depot's every-tick slot checks; our placement, after its behaviour processing)
        crate::depot_anim::building_turn(w, id);
    }
    // spec: S-0519 claim 4 (a planted bomb is checked in its carrier's own turn, after the behaviour processing; not in a pass of its own)
    crate::bomb::turn(w, id);
    // spec: S-0519 claim 19 (a designator's change of target cancels its strike: looked at in its own turn)
    crate::strike::turn(w, id);
    if w.obj(id).is_none() {
        return;
    }
    // spec: S-0506 claim 2 (a spawner's launch pass, when one is due, in its own turn; claim 24: a vehicle or ship spawner's own firing step comes later in the turn, so its shot waits for the next pass)
    crate::spawn::turn(w, id);
    if w.obj(id).is_none() {
        return;
    }
    if kind.is_mobile() {
        // spec: S-0516 claim 42 and S-0203 (the ordinary cloak update follows due behaviour processing, its shots included, and precedes ordinary locomotion and the separate infantry and vehicle firing)
        crate::cloak::turn(w, id);
    }
    if kind == Kind::Building {
        // spec: S-0513 claims 8 and 13 (a silo's accepted launch leaves on its next update)
        crate::superweapons::nuke::silo_turn(w, id);
        if !crate::temporal::building_erased(w, id) {
            crate::combat::behave::delayed_shot_turn(w, id);
            // spec: S-0518 claim 6 (a recruited supporter's beam comes in its own turn)
            crate::linked::supporter_turn(w, id);
        }
        // spec: S-0301 claim 10 (a building's refill, in its own turn after its own firing in that tick: after behaviour processing with both attempts of a paired firing, and after its animation-delayed shot)
        crate::combat::fire::building_refill(w, id);
        // spec: S-0518 claims 12 and 13 (a building's charger list and overpowered state, in its own turn)
        crate::linked::building_turn(w, id);
        // spec: S-0405 claims 7 and 9, S-0409 claim 8 (the building's own turn)
        crate::econ::events::turn(w, id, false);
        // spec: S-0501 claim 26 (a pad's service step, in the pad's own turn)
        crate::air::pad_turn(w, id);
        // spec: S-0302 claim 16 (the countdown's expiry hit, in the building's own turn, after the actions due earlier in that turn: self-healing, behaviour, ordinary and animation-delayed shots; Q-0170)
        crate::combat::damage::delay_kill_expiry_turn(w, id);
        return;
    }
    if on {
        p.mark(Phase::Movement);
    }
    // design: C7-P5 yield (a unit giving way resumes its order before its movement, once the other side has passed)
    crate::path::yield_turn(w, id);
    if kind == Kind::Vehicle {
        // spec: S-0203 claim 12 (a vehicle master's due group update comes after that turn's behaviour start and processing, whose destination and group-state changes it sees, and before its movement; a destination it assigns is available to that movement)
        crate::econ::slaves::group_turn(w, id);
        if w.obj(id).is_none() {
            return;
        }
    }
    let loco = w
        .obj(id)
        .and_then(|o| w.rules().unit(o.kind, o.type_index))
        .map(|u| u.locomotor);
    let rocket = w.obj(id).is_some_and(|o| o.rocket.is_some());
    // spec: S-0614 claim 8 (a held unit does not move under the driving, ship, flying, teleporting and tunnelling locomotors; walking and rocket ignore the hold)
    let held = crate::parasite::held(w, id);
    // spec: S-0511 claim 10 (while still falling after the fall step, no ordinary locomotion advance and no firing; a surviving landing lifts both refusals in the same turn)
    let falling = crate::fall::falling(w, id);
    let stand_in = crate::lift::moving_stand_in(w, id);
    let moved = match loco {
        // spec: S-0506 Terms (a rocket missile moves in the rocket-flight movement mode, claims 14 to 19)
        _ if rocket => crate::rocket::tick(w, id),
        _ if falling => fell,
        // spec: S-0507 claim 15 and S-0203 claim 3 (a passenger in play inside an open-topped transport has no movement of its own; its position is the transport's)
        _ if crate::transport::riding(w, id).is_some() => false,
        // spec: S-0504 claim 5 (a sinking vessel makes no progress by its locomotor, turning and speed-fraction updates included)
        _ if sinking => false,
        // spec: S-0607 claims 5 and 7 (a seized unit moves with its temporary jumpjet locomotor, whatever its own)
        _ if crate::lift::seized(w, id) && crate::lift::stand_in(w, id).is_none() => {
            crate::jump::tick(w, id)
        }
        // spec: S-0607 claim 23 (a unit seized with another stand-in moves under that stand-in's ordinary rules; a drop-pod stand-in has ended and the unit's own locomotor moves it)
        _ if stand_in.is_some() => {
            crate::lift::stand_in_tick(w, id, stand_in.unwrap_or(crate::lift::StandIn::Walk), held)
        }
        Some(Locomotor::Drive) if held => false,
        Some(Locomotor::Drive) => {
            let moved = drive::tick(w, id);
            // spec: S-0207 claim 23 (a drive vehicle's wake, created right after its drive update; Q-0600 answered)
            drive::driving_wake(w, id);
            moved
        }
        Some(Locomotor::Walk) => walk::tick(w, id),
        // spec: S-0503 claims 2 to 11 (the hovering locomotor; S-0614 claim 8 lists no hold for it)
        Some(Locomotor::Hover) => crate::hover::tick(w, id),
        // spec: S-0417 claim 5 (a tunnelling or teleporting vehicle drives temporarily for its factory exit)
        _ if crate::econ::release::temporary_drive(w, id) => !held && drive::tick(w, id),
        // spec: S-0207 claim 20 (d) (the jump on the first movement update after L is fixed; the behaviour is processed before it)
        Some(Locomotor::Teleport) => crate::teleport::tick(w, id),
        // spec: S-0502 claims 3 to 21 (the jumpjet locomotor; S-0614 claim 8 lists no hold for it)
        Some(Locomotor::Jump) => crate::jump::tick(w, id),
        // spec: S-0501 claims 1 to 10 (aircraft flight)
        Some(Locomotor::Fly) => !held && crate::air::tick(w, id),
        // A rocket-locomotor type outside the rocket-flight mode has no movement of its own (S-0506 Terms).
        // spec: S-0504 claim 1 (the ship locomotor)
        Some(Locomotor::Ship) => !held && crate::naval::tick(w, id),
        _ => false,
    };
    if on {
        p.mark(Phase::AfterMove);
    }
    // spec: S-0507 claim 15 (whenever the transport moves to a new position, its passengers' positions become its position at once); S-0314 claim 23 (with the position change, before the queued-entry retry)
    crate::transport::sync(w, id);
    // spec: S-0215 claim 11 (the late start of an infantry unit or vehicle: after this update's movement, before its separate firing; a tunnel, erasing, leaving-stage or warp-only update never gets here, claim 2 and S-0513 claim 51)
    process::late_start(w, id);
    // spec: S-0605 claim 4 (in each of its own turns, after its behaviour processing and its movement, an actor waiting in line asks for contact: S-0314 claim 23's turn position) and claim 17 (quirks.absorber_stale_contact_kept off: the contact check at the same place)
    crate::structures::absorb::waiting_turn(w, id);
    if kind == Kind::Vehicle && !crate::teleport::phasing_in(w, id) {
        // spec: S-0314 claim 23 (the queued-entry retry, once per ordinary living vehicle turn, after that turn's locomotor movement, self-healing and due behaviour processing; not before movement and not an extra behaviour processing; skipped, the queued entry kept, only while chrono phase-in is still active, not for a hold, pulse disable, deactivation, lift or locomotor replacement; before the attached parasite's step)
        crate::depot::vehicle_turn(w, id);
    }
    // spec: S-0614 claim 18 (an attached parasite acts late in its host's turn: after self-healing, behaviour processing and movement, before the separate firing update; a host killed by it does nothing more)
    crate::parasite::host_turn(w, id);
    if w.obj(id).is_none_or(crate::combat::damage::dead) {
        return;
    }
    // spec: S-0405 claim 13, S-0409 claim 8 (the unit's own turn, after its movement of the tick)
    crate::econ::events::turn(w, id, moved);
    if kind == Kind::Aircraft {
        // spec: S-0501 claims 15, 31 and 37 (the end of an aircraft's update)
        crate::air::end_turn(w, id);
    }
    if kind == Kind::Vehicle {
        crate::combat::fire::reload_turn(w, id);
        crate::combat::fire::firing_frame_turn(w, id);
        // spec: S-0310 claim 9 (the vehicle's own firing step each tick it has a target); S-0301 claim 9 rank 4 and S-0511 claim 10 (a unit still falling cannot fire; the check refuses it)
        let facing_error = crate::combat::behave::vehicle_step(w, id);
        // The turret's target at the end of the tick (S-0206 claim 9), after the movement.
        turret::tick(w, id, facing_error);
    }
    if kind == Kind::Infantry && w.obj(id).is_some() {
        let in_progress = occupancy::in_progress(w, id);
        let has_dest = w.obj(id).is_some_and(|o| o.dest.is_some());
        // spec: S-0107 State and ordering (the fear rules of a tick come before its firing rules: a sequence they start already holds for the firing rules)
        infantry::fear_turn(w, id, in_progress, has_dest);
        // spec: S-0107 claim 11 (firing); S-0107 State and ordering (judged against the sequence current before the tick's completion follow-up, so a fire frame equal to the count still releases the shot on the completion tick); S-0301 claim 9 rank 4 and S-0511 claim 10 (a unit still falling cannot fire, even with a firing sequence begun before the drop)
        crate::combat::behave::infantry_step(w, id);
        // spec: S-0107 State and ordering (the completion follow-up after the firing rules; the movement rule after the follow-up)
        infantry::sequence_turn(w, id, moved, in_progress);
        // spec: S-0417 claim 17 (a resting unit on a cell that holds a building is sent a scatter request after the rest of its turn: after its late start, its behaviour processing with that processing's draw and its movement, design; turns that end early above make none: being erased, S-0514 claim 17, a warp's leaving stage and earlier update boundaries, S-0513 claim 51; no tunnel is built)
        crate::scatter::resting_turn(w, id);
    }
    // spec: S-0607 claim 20 (revision 14: the restoration test at the end of each of the unit's turns, after its movement and after the point at which it may process its behaviour, whether or not it did, whatever its behaviour and house; the turns that end early above have none: lost, dead, being erased, and the warp-only updates, design for the phase-in that the spec leaves open)
    crate::lift::end_of_turn(w, id);
}
