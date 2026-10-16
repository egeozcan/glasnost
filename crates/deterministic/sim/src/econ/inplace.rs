//! Vehicles that deploy in place (S-0517 claims 3 to 15): the in-place
//! cycle of a vehicle whose type has the in-place deploy flag (mobile,
//! deploying, deployed, undeploying and, for a landing-first jumpjet, landing
//! for deploy), the deploy-to-fire cell test, the firing refusals and the
//! drawing helpers. The deploy-fire shot (claim 2) is in `econ::deploy`.

use glasnost_map::CellXY;
use serde::{Deserialize, Serialize};

use super::TypeRef;
use crate::combat::effects::{Effect, Sound};
use crate::rules::{AnimRules, Behaviour, Kind};
use crate::world::{Layer, ObjectId, Target, World};

/// The resting and changing states of an in-place deploying vehicle
/// (S-0517 Terms): a new vehicle is mobile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Form {
    /// Mobile.
    #[default]
    Mobile,
    /// Deploying since the tick `since`: the presentation plays forward.
    Deploying {
        /// The tick of the deploy step that started the transition (*t*₀).
        since: i32,
    },
    /// Deployed.
    Deployed,
    /// Undeploying since the tick `since`: the presentation plays in reverse.
    Undeploying {
        /// The tick of the undeploy step that started the transition.
        since: i32,
    },
}

/// The delay after a step that leaves a transition or a landing running.
// spec: S-0517 Constants (transition processing delay 1 tick, no draw) and claim 3
const TRANSITION_DELAY: i32 = 1;

/// The delay after a step that leaves no transition.
// spec: S-0517 Constants (450 ticks, the general default of S-0215 claim 4) and claim 3
const DONE_DELAY: i32 = 450;

fn unit_of(w: &World, id: ObjectId) -> Option<&crate::rules::UnitRules> {
    let o = w.obj(id)?;
    w.rules().unit(o.kind, o.type_index)
}

/// Whether the object is an in-place deploying vehicle (Terms: a vehicle
/// whose type has the in-place deploy flag).
pub fn is_in_place(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle)
        && unit_of(w, id).is_some_and(|u| u.simple_deployer)
}

/// The form of an in-place deploying vehicle (mobile for any other).
pub fn form(w: &World, id: ObjectId) -> Form {
    w.obj(id).map_or(Form::Mobile, |o| o.econ.deploy.form)
}

/// Whether the vehicle is deploying or undeploying (*in transition*,
/// Terms).
pub fn in_transition(w: &World, id: ObjectId) -> bool {
    matches!(
        form(w, id),
        Form::Deploying { .. } | Form::Undeploying { .. }
    )
}

/// Whether the vehicle is deployed.
pub fn deployed(w: &World, id: ObjectId) -> bool {
    form(w, id) == Form::Deployed
}

/// Whether a destination given to the vehicle is replaced by none (claim
/// 9): it is deployed or in transition.
pub fn blocks_destination(w: &World, id: ObjectId) -> bool {
    // spec: S-0517 claim 9 (a deployed vehicle or one in transition cannot receive a destination; the seized exception is S-0607's, not built)
    matches!(
        form(w, id),
        Form::Deploying { .. } | Form::Deployed | Form::Undeploying { .. }
    )
}

/// Enforces claim 9 on a destination written without the assignment
/// (an approach step, a rule that sets the field directly): called at the
/// start of the vehicle's behaviour turn.
pub fn enforce(w: &mut World, id: ObjectId) {
    if blocks_destination(w, id)
        && w.obj(id).is_some_and(|o| o.dest.is_some())
        && let Some(o) = w.obj_mut(id)
    {
        o.dest = None;
    }
}

/// The body of the vehicle is not drawn: the deploying presentation stands in
/// for it (claim 12).
pub fn body_hidden(w: &World, id: ObjectId) -> bool {
    // spec: S-0517 claim 12 (while a vehicle is in transition its own body is not drawn)
    in_transition(w, id)
}

/// The type whose image draws the deployed vehicle (claim 12): its deployed
/// image type, when the vehicle is deployed and the type names one.
pub fn deployed_image(w: &World, id: ObjectId) -> Option<u32> {
    // spec: S-0517 claim 12 (while deployed, the image of the deployed image type when the type names one, the own image otherwise; the vehicle keeps its own type for every rule)
    if !deployed(w, id) {
        return None;
    }
    let name = unit_of(w, id)?.unloading_class.as_deref()?;
    w.rules().index_of(Kind::Vehicle, name)
}

// ---------------------------------------------------------------------------
// The presentation (F1, F2)

/// The deploying presentation of the vehicle's type (claims 4 and 6).
fn presentation(w: &World, id: ObjectId) -> Option<&AnimRules> {
    let name = unit_of(w, id)?.deploying_anim.as_deref()?;
    w.rules().animation(name)
}

/// The base stage delay *d* of an animation type (S-0109 claim 2, F1): the
/// rate-derived delay before random rate selection and game-speed
/// normalisation (S-0517 claim 7: the vehicle's completion clock does not
/// follow the playback delay).
fn stage_delay(w: &World, a: &AnimRules) -> i32 {
    let q = &w.rules().quirks;
    match a.rate {
        // spec: S-0109 claim 2 (a type without the rate field has d = 1)
        None => 1,
        // spec: S-0109 claim 2 and F1 (d = floor(900 / rho) for rho > 0, 0 for rho <= 0; quirks.anim_fast_rate_freezes off: a positive rate gives at least 1) and S-0517 claim 7 (the base delay, without random rate selection or normalisation)
        Some(r) => crate::rules::anim_delay(r, q.anim_fast_rate_freezes),
    }
}

/// The end field *E* of an animation type (S-0109 claim 25): the field as
/// written unless it is absent or −1, then the sprite's: its frame count, or
/// half of it for a shadow type.
fn end_field(a: &AnimRules) -> i32 {
    match a.end {
        Some(e) if e != -1 => e,
        _ => a.frames.map_or(0, |n| if a.shadow { n / 2 } else { n }), // spec: S-0109 F2 (E_s(n) = n, or ⌊n / 2⌋ for a shadow type)
    }
}

/// Whether a transition begun at `since` has completed at the processing of
/// tick `now` (F1 and F2): `less` is 1 for a deploy and 2 for an undeploy.
fn completed(w: &World, a: &AnimRules, since: i32, less: i32) -> bool {
    // spec: S-0517 claim 7 (a transition never finishes in the processing that started it)
    if now_of(w) <= since {
        return false;
    }
    let (d, e) = (stage_delay(w, a), end_field(a));
    let stages = e - less;
    // spec: S-0517 F1 and F2 (with E ≤ 1, or E ≤ 2 for an undeploy, completion is at t₀ + 1)
    if stages <= 0 {
        return true;
    }
    if d == 0 {
        // spec: S-0517 Edge cases (quirks.deploy_stall_zero_delay on: with d = 0 no stages advance and the transition never completes; off: any zero-delay transition completes at the next processing)
        return !w.rules().quirks.deploy_stall_zero_delay;
    }
    // spec: S-0517 F1 and F2 and claim 7 (completion on t₀ + (E − 1) d + 1, or t₀ + (E − 2) d + 1 for an undeploy, in both directions: the stage count that reaches the threshold in tick T is seen by the processing of T + 1)
    now_of(w).wrapping_sub(since) > stages.wrapping_mul(d)
}

fn now_of(w: &World) -> i32 {
    w.tick
}

// ---------------------------------------------------------------------------
// Steps

fn set_form(w: &mut World, id: ObjectId, f: Form) {
    if let Some(o) = w.obj_mut(id) {
        o.econ.deploy.form = f;
    }
}

/// Requests the type's deploy or undeploy sound at the vehicle's position
/// (claim 12).
fn sound(w: &mut World, id: ObjectId, s: Sound) {
    let Some(o) = w.obj(id) else { return };
    let (of, pos) = (
        TypeRef {
            kind: o.kind,
            index: o.type_index,
        },
        o.pos,
    );
    w.effects.push(Effect::Sound {
        sound: s,
        of: Some(of),
        house: None,
        pos: Some(pos),
    });
}

/// Starts the deploying presentation at the vehicle's position, forward or
/// in reverse.
fn start_presentation(w: &mut World, id: ObjectId, reverse: bool) {
    let Some(name) = unit_of(w, id).and_then(|u| u.deploying_anim.clone()) else {
        return;
    };
    let Some(pos) = w.obj(id).map(|o| o.pos) else {
        return;
    };
    crate::combat::effects::anim_reversible(w, &name, pos, reverse);
}

/// Whether the vehicle is above the ground (claim 4's height).
fn airborne(w: &World, id: ObjectId) -> bool {
    crate::jump::is_jumpjet(w, id) && crate::jump::height_above(w, id) > 0
}

/// The deploy step (claims 4 and 5) of a mobile vehicle; the deploy sound
/// request of claim 12.
fn deploy_step(w: &mut World, id: ObjectId) {
    let repeats = w.rules().quirks.deploy_sound_repeats;
    if airborne(w, id) {
        let landing_first = unit_of(w, id).is_some_and(|u| u.deploy_to_land);
        if landing_first {
            // spec: S-0517 claim 5 (a landing-first type above the ground requests landing for deploy; the jumpjet locomotor then lands, S-0502 claim 15)
            let first = !crate::jump::landing_requested(w, id);
            crate::jump::request_landing(w, id);
            if repeats || first {
                // spec: S-0517 claim 12 (every deploy step requests the deploy sound; off: once, at the first step that starts landing)
                sound(w, id, Sound::Deploy);
            }
        }
        // spec: S-0517 claim 5 (a type that is not landing-first: nothing)
        return;
    }
    // spec: S-0517 claim 12 (the deploy sound at the vehicle's position at each deploy step, or once with the switch off: at an immediate deploy or the step that starts a transition)
    if repeats {
        sound(w, id, Sound::Deploy);
    }
    let now = now_of(w);
    if presentation(w, id).is_some() {
        // spec: S-0517 claim 4 (height 0 and a deploying presentation: it starts at the vehicle's position, playing forward, and the vehicle becomes deploying)
        if !repeats {
            sound(w, id, Sound::Deploy);
        }
        start_presentation(w, id, false);
        set_form(w, id, Form::Deploying { since: now });
    } else {
        // spec: S-0517 claim 4 (height 0 and no presentation: it becomes deployed in that step)
        if !repeats {
            sound(w, id, Sound::Deploy);
        }
        set_form(w, id, Form::Deployed);
    }
}

/// The deploying vehicle's step (claim 4, third row): deployed when the
/// completion time of F1 has passed, otherwise nothing changes.
fn deploying_step(w: &mut World, id: ObjectId, since: i32) {
    if w.rules().quirks.deploy_sound_repeats {
        // spec: S-0517 claim 12 (each deploy step, the completion processing included, requests the deploy sound)
        sound(w, id, Sound::Deploy);
    }
    let done = presentation(w, id).is_none_or(|a| completed(w, a, since, 1));
    if done {
        set_form(w, id, Form::Deployed);
    }
}

/// The undeploy step (claim 6) of a deployed vehicle, and of one that is
/// undeploying.
fn undeploy_step(w: &mut World, id: ObjectId) {
    let repeats = w.rules().quirks.deploy_sound_repeats;
    let now = now_of(w);
    if repeats {
        // spec: S-0517 claim 12 (each undeploy step requests the undeploy sound)
        sound(w, id, Sound::Undeploy);
    }
    match form(w, id) {
        Form::Undeploying { since } => {
            // spec: S-0517 claim 6 (undeploying: it becomes mobile when the completion time of F2 has passed)
            if presentation(w, id).is_none_or(|a| completed(w, a, since, 2)) {
                set_form(w, id, Form::Mobile);
                landing_first_take_off(w, id);
            }
        }
        _ => {
            // spec: S-0517 claim 6 (when it starts, the undeploy step removes a disguise the vehicle wears)
            crate::cloak::disguise::clear(w, id);
            if !repeats {
                sound(w, id, Sound::Undeploy);
            }
            if presentation(w, id).is_some() {
                // spec: S-0517 claim 6 (a presentation: it starts in reverse and the vehicle becomes undeploying)
                start_presentation(w, id, true);
                set_form(w, id, Form::Undeploying { since: now });
            } else {
                // spec: S-0517 claim 6 (no presentation: it becomes mobile in that step, and no take-off destination is assigned)
                set_form(w, id, Form::Mobile);
            }
        }
    }
}

/// A landing-first vehicle that completes an animated undeploy is given a
/// destination from a nearby-cell search centred on its position, so that
/// its locomotor takes off again (claim 6; the search's settings are S-0502 claim 23's last row).
fn landing_first_take_off(w: &mut World, id: ObjectId) {
    let landing_first = unit_of(w, id).is_some_and(|u| u.deploy_to_land);
    if !landing_first || presentation(w, id).is_none() {
        return;
    }
    let Some(o) = w.obj(id) else { return };
    let Some(u) = unit_of(w, id) else { return };
    // spec: S-0517 claim 6 and S-0502 claim 23 (S-0212 claim 25's search centred on the vehicle's position: the type's own class with winged counted as track, its own movement zone, not simplified, the reach region of the centre cell for that zone read for its deck when the vehicle is on a deck (none when the read gives none), deck mode when on a deck, no height condition, no reference cell, bridge cells allowed, no further test)
    let at = o.pos.cell();
    let deck = o.layer == Layer::Deck;
    let class = match u.class {
        Some(glasnost_rules::sim::LocomotionClass::Winged) => {
            Some(glasnost_rules::sim::LocomotionClass::Track)
        }
        c => c,
    };
    let zone = u.zone;
    crate::regions::ensure(w);
    let reach = crate::regions::reach_read(w, at, zone, deck);
    let reach = (reach != crate::regions::Reach::None).then_some(reach);
    let s = crate::path::NearbySettings {
        class,
        zone,
        reach,
        deck,
        height: false,
        reference: None,
        no_bridges: false,
        no_overlay: false,
    };
    if let Some(c) = crate::path::nearby(w, at, crate::path::NearbyUse::Settings(s)) {
        super::assign_destination(w, id, Some(Target::Cell(c)));
    }
}

/// One processing of *unload* of an in-place deploying vehicle (claims 3 to
/// 8): the step, then the delay and the behaviour that follows it.
pub fn process(w: &mut World, id: ObjectId) -> i32 {
    let stale = w.rules().quirks.stale_suspended_revert; // spec: S-0215 Edge cases (quirks.stale_suspended_revert)
    let was = form(w, id);
    match was {
        Form::Mobile => deploy_step(w, id),
        Form::Deploying { since } => deploying_step(w, id, since),
        Form::Deployed | Form::Undeploying { .. } => undeploy_step(w, id),
    }
    let landing = crate::jump::landing_requested(w, id) && airborne(w, id);
    if in_transition(w, id) || landing {
        // spec: S-0517 claim 3 (in transition or landing for deploy: 1 tick, no draw: processing every tick until the change finishes)
        return TRANSITION_DELAY;
    }
    // spec: S-0517 claim 5 and Edge cases (quirks.airborne_deploy_order_dropped off: an order to an airborne in-place type that cannot land is retained until the vehicle is on the ground)
    if airborne(w, id) && !w.rules().quirks.airborne_deploy_order_dropped {
        return TRANSITION_DELAY;
    }
    let t = w.tick;
    let discard = w.rules().quirks.deploy_transition_discards_orders;
    if let Some(o) = w.obj_mut(id) {
        if matches!(was, Form::Deploying { .. } | Form::Undeploying { .. }) && !discard {
            // spec: S-0517 claim 8 (quirks.deploy_transition_discards_orders off: completion starts the pending behaviour, or guard when none is pending)
            if o.beh.queued.is_none() {
                o.beh.queue(Some(Behaviour::Guard));
            }
            o.beh.start_next(t, stale);
        } else {
            // spec: S-0517 claim 3 (otherwise guard replaces any queued behaviour and is started at once; the delay below overrides the start, S-0215 claim 3c)
            o.beh.queue(Some(Behaviour::Guard));
            o.beh.start_next(t, stale);
        }
    }
    DONE_DELAY
}

/// Whether an in-place deploying vehicle may start a different behaviour:
/// not while deploying or undeploying (claim 8, S-0215 claim 11).
pub fn ready(w: &World, id: ObjectId) -> bool {
    // spec: S-0517 claim 8 (an ordinary order cannot start a different behaviour during deploying or undeploying: the vehicle is not ready to change)
    !in_transition(w, id)
}

/// The move fallback of claim 10: *move* processed for a vehicle that is
/// deployed or in transition queues *guard* and chooses 1 tick.
pub fn move_fallback(w: &mut World, id: ObjectId) -> Option<i32> {
    // spec: S-0517 claim 10 (move processed for a deployed vehicle or one in transition: guard becomes its queued behaviour, 1 tick)
    if !blocks_destination(w, id) {
        return None;
    }
    if let Some(o) = w.obj_mut(id) {
        o.beh.queue(Some(Behaviour::Guard));
    }
    Some(TRANSITION_DELAY)
}

// ---------------------------------------------------------------------------
// Deploy-to-fire (claims 13 and 14)

/// The deploy-to-fire cell test of claim 14 for cell `c`: flat, no building
/// among its occupants, and a land type with the buildable flag.
pub fn deploy_test_cell(w: &World, c: CellXY) -> bool {
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    // spec: S-0517 claim 14 (its slope shape is flat, S-0406)
    if cell.slope != 0 {
        return false;
    }
    // spec: S-0517 claim 14 (no building is among its occupants)
    let building = w
        .cells
        .list(c, Layer::Ground)
        .iter()
        .any(|&x| w.obj(x).is_some_and(|o| o.kind == Kind::Building));
    if building {
        return false;
    }
    // spec: S-0517 claim 14 (its land type has the buildable flag B, S-0211 claim 2 c)
    w.rules()
        .land
        .get(usize::from(cell.land))
        .is_some_and(|l| l.buildable)
}

/// Whether a vehicle's own cell passes the deploy-to-fire test.
pub fn deploy_test(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| deploy_test_cell(w, o.pos.cell()))
}

/// Whether a type is a deploy-to-fire vehicle (Terms).
pub fn is_deploy_to_fire(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.kind == Kind::Vehicle)
        && unit_of(w, id).is_some_and(|u| u.vehicle.deploy_to_fire)
}

/// Whether a deploy-to-fire vehicle queues *unload* instead of firing at a
/// *can fire* or *not facing* answer (claim 13): always while mobile; when
/// deployed only with `quirks.deploy_to_fire_repacks` (the default), the
/// type then also having the in-place flag.
pub fn queues_unload(w: &World, id: ObjectId) -> bool {
    if !is_deploy_to_fire(w, id) {
        return false;
    }
    // spec: S-0517 claim 13 and Edge cases (a deployed vehicle queues unload again with quirks.deploy_to_fire_repacks on; with it off it fires normally and only a mobile one queues unload)
    !deployed(w, id) || w.rules().quirks.deploy_to_fire_repacks
}

// ---------------------------------------------------------------------------
// Firing refusals (claim 11)

/// Whether the firing check refuses every weapon and target of the vehicle:
/// it is in transition (claim 11, *illegal* at S-0301 claim 9's rank 4).
pub fn refuses_fire(w: &World, firer: ObjectId) -> bool {
    // spec: S-0517 claim 11 (a vehicle's transition supplies an illegal refusal for every weapon and target at rank 4)
    in_transition(w, firer)
}

/// Whether a magnetic-beam weapon is refused against `target`: a vehicle in
/// transition, or an in-place deploying vehicle whose current behaviour is
/// *unload* (claim 11).
pub fn beam_refused(w: &World, target: ObjectId) -> bool {
    // spec: S-0517 claim 11 (a weapon whose warhead carries its target off with a locomotor gets illegal against a target vehicle that is in transition or is an in-place deploying vehicle whose current behaviour is unload, landing for deploy and the first processing included)
    in_transition(w, target)
        || (is_in_place(w, target)
            && w.obj(target)
                .is_some_and(|o| o.beh.current == Some(Behaviour::Unload)))
}
