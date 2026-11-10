//! Claim tests of S-0309 revision 124 (Q-2747 answered), claim 31: the
//! orders reset at an owner change. Every cause that claim 30 lists, the
//! vehicle-like takeover and the contacts that follow a building clear the
//! destination and the guard spot (with a factory's rally point and a
//! harvester's remembered cell), request *guard*, apply the *rescue* rule and
//! fall back to rest; the override in effect and the attack-move memory are
//! kept (`quirks.owner_change_keeps_override`,
//! `quirks.owner_change_keeps_attack_move`, both positions). Built on
//! scenario A19, on the synthetic rules of `structures_common`: invented
//! types and numbers, no game data. Ledger: `docs/m8-claims/intake112.md`.

mod deploy_common;
mod structures_common;

use glasnost_map::triggers::{Action, Event, TagDef, TriggerDef, Triggers};
use glasnost_rules::schema::Ruleset;
use glasnost_sim::econ::harvest::HarvestState;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::ownership::{self, Cause};
use glasnost_sim::world::{AttackMoveMemory, ObjectId, Target, World};
use structures_common::*;

/// A map whose tag `TAG` changes its carriers' owner to Beta (house value
/// 1) when event 8 fires.
fn change_owner_triggers() -> Triggers {
    let t = TriggerDef {
        events: vec![Event {
            code: 8,
            kind: 0,
            value: 0,
            type_name: None,
        }],
        actions: vec![Action {
            code: 14,
            number: 1,
            ..Default::default()
        }],
        ..TriggerDef::created(b"CHOWN".to_vec())
    };
    Triggers {
        triggers: vec![t],
        tags: vec![TagDef {
            id: b"TAG".to_vec(),
            repeat: 0,
            name: Vec::new(),
            first: Some(0),
            chain: vec![0],
        }],
        ..Default::default()
    }
}

/// Raises event 8 on `holder`'s tag, so the change-owner action gives it
/// to Beta.
fn raise_change(w: &mut World, holder: ObjectId) {
    let tag_id = *w.triggers.objects.get(&holder).unwrap();
    glasnost_sim::triggers::raise(
        w,
        tag_id,
        glasnost_sim::triggers::Raise {
            code: Some(8),
            causer: None,
            carrier: Some(glasnost_sim::triggers::Carrier::Object(holder)),
        },
    );
}

/// A world on `r` with the change-owner trigger, and a `name` object of
/// `kind` of Alpha's at the map's middle carrying its tag.
fn tagged(r: &Ruleset, kind: Kind, name: &str) -> (World, ObjectId) {
    let mut w = game_on(r, change_owner_triggers());
    let id = put(&mut w, kind, name, 0, 0, 0);
    tag(&mut w, id, "TAG");
    (w, id)
}

fn beh(w: &World, id: ObjectId) -> (Option<Behaviour>, Option<Behaviour>) {
    let o = w.obj(id).unwrap();
    (o.beh.current, o.beh.queued)
}

fn guarded(w: &World, id: ObjectId) -> bool {
    let (c, q) = beh(w, id);
    c == Some(Behaviour::Guard) || q == Some(Behaviour::Guard)
}

/// [`ruleset`] with KITE, an unarmed landable aircraft (invented numbers).
fn kite_rules() -> Ruleset {
    let mut r = ruleset();
    let mut kite = glasnost_rules::sim::UnitType {
        strength: Some(500),
        speed: Some(20),
        rot: Some(12),
        locomotor: Some(glasnost_rules::sim::Locomotor::Fly),
        ..Default::default()
    };
    kite.econ.landable = Some(true);
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("KITE".into());
    r.sim.aircraft.insert("KITE".into(), kite);
    r
}

// spec: S-0309 claim 31 (every cause that claim 30 lists and the takeover of a vehicle-like building, S-0509 claim 17, request guard; Q-2747 answered: the takeover is no longer drop only) and claim 30 (each drops the target)
#[test]
fn every_cause_and_the_takeover_request_guard() {
    for cause in [
        Cause::Capture,
        Cause::Trigger,
        Cause::Occupation,
        Cause::Takeover,
        Cause::MindControl,
    ] {
        let mut w = game();
        let tower = put(&mut w, Kind::Building, "TOWER", 0, 0, 0);
        let foe = put(&mut w, Kind::Infantry, "WALKER", 1, 3, 0);
        w.obj_mut(tower).unwrap().target = Some(Target::Object(foe));
        w.obj_mut(tower).unwrap().beh.current = Some(Behaviour::Sleep);
        ownership::change(&mut w, tower, Some(B), cause);
        assert_eq!(w.obj(tower).unwrap().target, None, "{cause:?}: dropped");
        assert!(guarded(&w, tower), "{cause:?}: guard requested");
    }
}

// spec: S-0309 claim 31 (a factory forgets its rally point, S-0213 claim 20: the guard spot is forgotten at the change); scenario A19 part f
#[test]
fn a_captured_factory_forgets_its_rally_point() {
    let mut w = game();
    let k = put(&mut w, Kind::Building, "WARF", 0, 0, 0);
    w.obj_mut(k).unwrap().guard_spot = Some(Target::Cell(at(6, 6)));
    ownership::change(&mut w, k, Some(B), Cause::Capture);
    assert_eq!(w.obj(k).unwrap().guard_spot, None);
}

// spec: S-0309 claim 31 (a building requests its idle body state and queues guard without an immediate start at the fall-back; S-0108 claim 6: a building already idle is not changed)
#[test]
fn a_captured_building_requests_its_idle_body() {
    let mut w = game();
    let tower = put(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    let idle = w.obj(tower).unwrap().econ.idle_body;
    w.obj_mut(tower).unwrap().econ.idle_body = None;
    ownership::change(&mut w, tower, Some(B), Cause::Capture);
    assert!(
        w.obj(tower).unwrap().econ.idle_body.is_some(),
        "the idle body is requested"
    );
    // A building already idle keeps its stage and start.
    let mut w = game();
    let tower = put(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    let before = w.obj(tower).unwrap().econ.idle_body;
    assert!(before.is_some() || idle.is_none(), "set-up");
    ownership::change(&mut w, tower, Some(B), Cause::Capture);
    assert_eq!(w.obj(tower).unwrap().econ.idle_body, before);
}

// spec: S-0309 claim 31 (r126, Q-2801 answered: an object whose current behaviour is selling gets no guard request and keeps selling; the fall-back still requests the building's idle body, and S-0215 claim 7 refuses its queued guard while selling is current)
#[test]
fn a_building_being_sold_keeps_selling() {
    let mut w = game();
    let tower = put(&mut w, Kind::Building, "TOWER", 0, 0, 0);
    {
        let o = w.obj_mut(tower).unwrap();
        o.beh.current = Some(Behaviour::Selling);
        o.beh.queued = None;
        o.econ.idle_body = None;
    }
    ownership::change(&mut w, tower, Some(B), Cause::Capture);
    assert_eq!(beh(&w, tower), (Some(Behaviour::Selling), None));
    assert!(w.obj(tower).unwrap().econ.idle_body.is_some());
}

// spec: S-0309 claim 31 (a vehicle: the destination cleared, the guard spot forgotten, guard requested; a harvester forgets its remembered cell, S-0402 claim 3); scenario A19 part e
#[test]
fn a_trigger_change_forgets_destination_guard_spot_and_remembered_cell() {
    let (mut w, v) = tagged(&ruleset(), Kind::Vehicle, "CRUSHER");
    {
        let o = w.obj_mut(v).unwrap();
        o.dest = Some(Target::Cell(at(5, 5)));
        o.guard_spot = Some(Target::Cell(at(6, 6)));
        o.econ.harvest = Some(HarvestState {
            remembered: Some(at(7, 7)),
            ..Default::default()
        });
    }
    raise_change(&mut w, v);
    assert_eq!(owner(&w, v), Some(B), "the action ran");
    let o = w.obj(v).unwrap();
    assert_eq!(o.dest, None, "the destination is cleared");
    assert_eq!(o.guard_spot, None, "the guard spot is forgotten");
    assert_eq!(
        o.econ.harvest.as_ref().and_then(|h| h.remembered),
        None,
        "the remembered cell is forgotten"
    );
    assert!(guarded(&w, v));
    // An object with no harvest state is given none.
    let (mut w, v) = tagged(&ruleset(), Kind::Vehicle, "CRUSHER");
    raise_change(&mut w, v);
    assert!(w.obj(v).unwrap().econ.harvest.is_none());
}

// spec: S-0309 claim 31 (an infantry unit forgets its guard spot before the fall-back, so S-0215 claim 13's guard-spot row gives it no destination there)
#[test]
fn an_infantry_unit_does_not_return_to_its_guard_spot() {
    let (mut w, u) = tagged(&ruleset(), Kind::Infantry, "WALKER");
    w.obj_mut(u).unwrap().guard_spot = Some(Target::Cell(at(6, 6)));
    raise_change(&mut w, u);
    let o = w.obj(u).unwrap();
    assert_eq!((o.dest, o.guard_spot), (None, None));
    assert_ne!(o.beh.queued, Some(Behaviour::Move), "no move to the spot");
}

// spec: S-0309 claim 31 (rescue: an object whose current behaviour is rescue then has guard made current at once, without the readiness test)
#[test]
fn rescue_current_is_replaced_by_guard() {
    let (mut w, v) = tagged(&ruleset(), Kind::Vehicle, "CRUSHER");
    {
        let o = w.obj_mut(v).unwrap();
        o.beh.current = Some(Behaviour::Rescue);
        o.beh.queued = None;
    }
    raise_change(&mut w, v);
    assert_eq!(beh(&w, v).0, Some(Behaviour::Guard));
}

// spec: S-0309 claim 31 (r126, Q-2801 answered: S-0215 claim 7 refuses the guard request of an object in wait, and the rescue rule's force of guard is refused while wait is current, S-0215 claim 9, so wait stays current and rescue queued)
#[test]
fn wait_with_rescue_queued_keeps_both() {
    let (mut w, v) = tagged(&ruleset(), Kind::Vehicle, "CRUSHER");
    {
        let o = w.obj_mut(v).unwrap();
        o.beh.current = Some(Behaviour::Wait);
        o.beh.queued = Some(Behaviour::Rescue);
    }
    raise_change(&mut w, v);
    assert_eq!(beh(&w, v), (Some(Behaviour::Wait), Some(Behaviour::Rescue)));
}

/// The A19 part h set-up: an engaged vehicle with an attack-move memory to
/// a remembered destination, then the trigger's change.
fn engaged_change(keep: bool) -> (World, ObjectId) {
    let mut r = ruleset();
    r.globals.quirks.owner_change_keeps_attack_move = keep;
    let (mut w, v) = tagged(&r, Kind::Vehicle, "CRUSHER");
    {
        let o = w.obj_mut(v).unwrap();
        o.attack_move = Some(AttackMoveMemory::Dest(Target::Cell(at(8, 0))));
        o.combat.engaged = true;
        // Set-up: the placement's fall-back used the refresh of this tick
        // (claim 23 (b), at most once per tick); the change comes later.
        o.beh.last_common_effects = None;
    }
    raise_change(&mut w, v);
    assert_eq!(owner(&w, v), Some(B), "the action ran");
    (w, v)
}

// spec: S-0309 claim 31 (attack-move memory: the change keeps it and its engaged mark; the fall-back's refresh, claim 23 (b), resumes it for an engaged object: move queued with an immediate start and the remembered destination its destination again); scenario A19 part h; Edge cases (quirks.owner_change_keeps_attack_move on)
#[test]
fn an_engaged_vehicle_resumes_its_attack_move() {
    let (w, v) = engaged_change(true);
    let o = w.obj(v).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Move), "resumed");
    assert!(!o.combat.engaged, "resuming ends the engaged mark");
    assert_eq!(
        o.dest,
        Some(Target::Cell(at(8, 0))),
        "its destination again"
    );
}

// spec: S-0309 Edge cases (quirks.owner_change_keeps_attack_move off: the change removes the memory with its engaged mark before the guard request, so the fall-back does not resume it)
#[test]
fn quirk_owner_change_keeps_attack_move_off_forgets_the_memory() {
    let (w, v) = engaged_change(false);
    let o = w.obj(v).unwrap();
    assert_eq!(o.attack_move, None, "the memory is removed");
    assert!(!o.combat.engaged, "with its engaged mark");
    assert_eq!(o.dest, None, "no resumed destination");
    assert!(guarded(&w, v));
}

// spec: S-0309 claim 31 (an object whose refresh has already been used on that tick, claim 23 (b), keeps its memory as it was)
#[test]
fn a_refresh_used_on_the_tick_keeps_the_memory() {
    let (mut w, v) = tagged(&ruleset(), Kind::Vehicle, "CRUSHER");
    let t = w.tick;
    {
        let o = w.obj_mut(v).unwrap();
        o.attack_move = Some(AttackMoveMemory::Dest(Target::Cell(at(8, 0))));
        o.combat.engaged = true;
        o.beh.last_common_effects = Some(t);
    }
    raise_change(&mut w, v);
    let o = w.obj(v).unwrap();
    assert_eq!(
        o.attack_move,
        Some(AttackMoveMemory::Dest(Target::Cell(at(8, 0))))
    );
    assert!(o.combat.engaged);
    assert_eq!(o.dest, None, "not resumed");
}

/// A vehicle with an override in effect (*hunt* suspended, a target and a
/// destination put aside), then the trigger's change.
fn overridden_change(keep: bool, stale: bool) -> (World, ObjectId, ObjectId) {
    let mut r = ruleset();
    r.globals.quirks.owner_change_keeps_override = keep;
    r.globals.quirks.stale_suspended_revert = stale;
    let (mut w, v) = tagged(&r, Kind::Vehicle, "CRUSHER");
    let foe = put(&mut w, Kind::Infantry, "WALKER", 1, 4, 0);
    {
        let o = w.obj_mut(v).unwrap();
        o.beh.current = Some(Behaviour::Attack);
        o.beh.queued = None;
        o.beh.suspended = Some(Behaviour::Hunt);
        o.beh.aside = Some((Some(Target::Object(foe)), Some(Target::Cell(at(5, 5)))));
    }
    raise_change(&mut w, v);
    assert_eq!(owner(&w, v), Some(B), "the action ran");
    (w, v, foe)
}

// spec: S-0309 claim 31 (override in effect: the change does not empty the suspended slot, so any object other than an aircraft keeps the override in effect after the change); Edge cases (quirks.owner_change_keeps_override on, quirks.stale_suspended_revert on)
#[test]
fn a_vehicle_keeps_its_override_in_effect() {
    let (w, v, foe) = overridden_change(true, true);
    let o = w.obj(v).unwrap();
    assert_eq!(o.beh.suspended, Some(Behaviour::Hunt));
    assert_eq!(
        o.beh.aside,
        Some((Some(Target::Object(foe)), Some(Target::Cell(at(5, 5)))))
    );
}

// spec: S-0309 claim 31 (with quirks.stale_suspended_revert off, the first start-next or force at or after the change empties the slot: for an object whose guard starts at the request, that start ends it within the change); Edge cases (quirks.owner_change_keeps_override on)
#[test]
fn with_stale_revert_off_a_started_guard_ends_the_override() {
    let (w, v, _) = overridden_change(true, false);
    let o = w.obj(v).unwrap();
    if o.beh.current == Some(Behaviour::Guard) {
        assert_eq!(o.beh.suspended, None, "the start emptied the slot");
    } else {
        assert_eq!(o.beh.suspended, Some(Behaviour::Hunt), "guard queued: kept");
    }
}

// spec: S-0309 Edge cases (quirks.owner_change_keeps_override off: the change empties the suspended slot and forgets the target and destination the override put aside, whichever way quirks.stale_suspended_revert is set)
#[test]
fn quirk_owner_change_keeps_override_off_empties_the_slot() {
    for stale in [true, false] {
        let (w, v, _) = overridden_change(false, stale);
        let o = w.obj(v).unwrap();
        assert_eq!(o.beh.suspended, None, "stale {stale}");
        assert_eq!(o.beh.aside, None, "stale {stale}");
    }
}

/// An aircraft in *attack* with an override in effect (*move* suspended,
/// the target `foe` and a cell put aside), then the trigger's change.
fn overridden_aircraft(keep: bool) -> (World, ObjectId, ObjectId) {
    let mut r = kite_rules();
    r.globals.quirks.owner_change_keeps_override = keep;
    let (mut w, a) = tagged(&r, Kind::Aircraft, "KITE");
    let foe = put(&mut w, Kind::Infantry, "WALKER", 1, 4, 0);
    {
        let o = w.obj_mut(a).unwrap();
        o.beh.current = Some(Behaviour::Attack);
        o.beh.queued = None;
        o.beh.suspended = Some(Behaviour::Move);
        o.beh.aside = Some((Some(Target::Object(foe)), Some(Target::Cell(at(5, 5)))));
    }
    raise_change(&mut w, a);
    assert_eq!(owner(&w, a), Some(B), "the action ran");
    (w, a, foe)
}

// spec: S-0309 claim 31 (an aircraft whose override is in effect at the change takes the first row of S-0501 table T2 at the change's fall-back: its suspended behaviour current again, and the target and destination put aside come back, whatever their relation to the new owner); Edge cases (quirks.owner_change_keeps_override on)
#[test]
fn an_aircraft_gets_its_override_back_at_the_fall_back() {
    let (w, a, foe) = overridden_aircraft(true);
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Move), "restored");
    assert_eq!(o.target, Some(Target::Object(foe)), "the target comes back");
    assert_eq!(o.dest, Some(Target::Cell(at(5, 5))), "the destination too");
}

// spec: S-0309 Edge cases (quirks.owner_change_keeps_override off: the override is no longer in effect, so the aircraft's fall-back takes the rows of S-0501 table T2 after the first)
#[test]
fn quirk_owner_change_keeps_override_off_no_revert_for_an_aircraft() {
    let (w, a, _) = overridden_aircraft(false);
    let o = w.obj(a).unwrap();
    assert_ne!(o.beh.current, Some(Behaviour::Move), "not restored");
    assert_eq!(o.target, None);
    assert_eq!(o.beh.suspended, None);
}

// spec: S-0309 claim 31 (each object that changes owner together with a building it is in contact with, S-0509 claim 14, gets the reset: its guard spot forgotten and guard requested; in contact with a vehicle factory it gets no fall-back) and S-0509 claim 14 (a contact of a vehicle factory follows it without a distance test)
#[test]
fn a_contact_that_follows_the_building_is_reset() {
    let mut r = ruleset();
    r.sim.buildings.get_mut("WARF").unwrap().weapons_factory = Some(true);
    let mut w = game_on(&r, Default::default());
    let k = put(&mut w, Kind::Building, "WARF", 0, 0, 0);
    let v = put(&mut w, Kind::Vehicle, "CRUSHER", 0, 4, 0);
    {
        let o = w.obj_mut(v).unwrap();
        o.contact = Some(k);
        o.dest = None;
        o.guard_spot = Some(Target::Cell(at(6, 6)));
        o.beh.current = Some(Behaviour::Sleep);
        o.beh.queued = None;
    }
    ownership::change(&mut w, k, Some(B), Cause::Capture);
    assert_eq!(owner(&w, v), Some(B), "it follows the factory");
    let o = w.obj(v).unwrap();
    assert_eq!(o.guard_spot, None, "the guard spot is forgotten");
    assert!(guarded(&w, v), "guard requested");
}

// spec: S-0309 claim 31 (a vehicle in the middle of deploying or undeploying in place, S-0517 Terms, claims 4 and 6, keeps its guard spot at the change; a mobile vehicle of the same type forgets it) (seam sweep 7a: intake 112's row)
#[test]
fn seams7a_a_vehicle_deploying_in_place_keeps_its_guard_spot() {
    use glasnost_sim::econ::inplace::{self, Form};
    let size = deploy_common::SIZE;
    for undeploy in [false, true] {
        let mut w = glasnost_sim::testkit::game_with_triggers(
            &deploy_common::ruleset(),
            size,
            7,
            change_owner_triggers(),
        );
        glasnost_sim::testkit::run(&mut w, 4);
        // ATANK deploys in place with a presentation: its transitions last
        // several ticks (S-0517 claim 7).
        let v = deploy_common::put(&mut w, "ATANK", Kind::Vehicle, 0, 0, 0);
        let m = deploy_common::put(&mut w, "ATANK", Kind::Vehicle, 0, 4, 0);
        tag(&mut w, v, "TAG");
        tag(&mut w, m, "TAG");
        deploy_common::deploy(&mut w, v);
        if undeploy {
            for _ in 0..60 {
                if inplace::form(&w, v) == Form::Deployed {
                    break;
                }
                glasnost_sim::tick::step(&mut w);
            }
            assert_eq!(inplace::form(&w, v), Form::Deployed);
            deploy_common::deploy(&mut w, v);
        }
        for _ in 0..40 {
            if inplace::in_transition(&w, v) {
                break;
            }
            glasnost_sim::tick::step(&mut w);
        }
        let form = inplace::form(&w, v);
        if undeploy {
            assert!(matches!(form, Form::Undeploying { .. }), "{form:?}");
        } else {
            assert!(matches!(form, Form::Deploying { .. }), "{form:?}");
        }
        assert_eq!(inplace::form(&w, m), Form::Mobile);
        let spot = Some(Target::Cell(glasnost_sim::testkit::mid(size, 6, 6)));
        w.obj_mut(v).unwrap().guard_spot = spot;
        w.obj_mut(m).unwrap().guard_spot = spot;
        raise_change(&mut w, v);
        assert_eq!((owner(&w, v), owner(&w, m)), (Some(B), Some(B)));
        assert!(inplace::in_transition(&w, v), "still in transition");
        assert_eq!(w.obj(v).unwrap().guard_spot, spot, "kept in transition");
        assert_eq!(
            w.obj(m).unwrap().guard_spot,
            None,
            "a mobile one forgets it"
        );
    }
}
