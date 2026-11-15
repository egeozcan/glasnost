//! The object-table walks of a removal and a killing hit (#402): the
//! passes that now serve several back-reference kinds at once, and the
//! walks a removal skips, must leave every back-reference changed as the
//! separate walks did. Synthetic data.

#[path = "lift_common/mod.rs"]
mod lift_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType};
use glasnost_sim::bomb::Bomb;
use glasnost_sim::combat::damage::Hit;
use glasnost_sim::lift;
use glasnost_sim::rules::Kind;
use glasnost_sim::spawn::SpawnController;
use glasnost_sim::testkit;
use glasnost_sim::world::{AttackMoveMemory, ObjectId, Target, World};

const SIZE: i32 = 40;

/// How the object leaves: removed, or killed by a hit (a synthetic
/// infantry unit then leaves play inside the hit).
#[derive(Clone, Copy, Debug)]
enum How {
    Removed,
    Killed,
}

/// A hit no unit survives.
fn killing_hit() -> Hit {
    Hit {
        amount: 1_000_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    }
}

fn game() -> World {
    testkit::game_with(&testkit::ruleset(), SIZE, 1)
}

fn walker(w: &mut World, house: u8, dx: i32) -> ObjectId {
    testkit::spawn(
        w,
        Kind::Infantry,
        "WALKER",
        house,
        testkit::mid(SIZE, dx, 0),
        0,
    )
}

// spec: S-0309 claim 27a (every holder whose override goes on loses the object from its put-aside target and destination) and claim 23 (d) (a remembered target lost turns into a remembered destination at its cell)
#[test]
fn one_pass_clears_put_aside_slots_and_converts_memories() {
    for how in [How::Removed, How::Killed] {
        let mut w = game();
        let gone = walker(&mut w, 1, 6);
        let other = walker(&mut w, 1, 8);
        let a = walker(&mut w, 0, -6);
        let b = walker(&mut w, 0, -4);
        let c = walker(&mut w, 0, -2);
        let d = walker(&mut w, 0, 0);
        let cell = w.obj(gone).unwrap().pos.cell();
        let it = Some(Target::Object(gone));
        let elsewhere = Some(Target::Cell(testkit::mid(SIZE, 3, 3)));
        w.obj_mut(a).unwrap().beh.aside = Some((it, elsewhere));
        w.obj_mut(b).unwrap().beh.aside = Some((Some(Target::Object(other)), it));
        w.obj_mut(c).unwrap().attack_move = Some(AttackMoveMemory::Target(Target::Object(gone)));
        {
            let o = w.obj_mut(d).unwrap();
            o.beh.aside = Some((it, it));
            o.attack_move = Some(AttackMoveMemory::Target(Target::Object(gone)));
        }
        let kept = Some(AttackMoveMemory::Target(Target::Object(other)));
        w.obj_mut(other).unwrap().attack_move = kept;
        match how {
            How::Removed => glasnost_sim::objects::remove(&mut w, gone),
            How::Killed => glasnost_sim::combat::death::kill(&mut w, gone, &killing_hit()),
        }
        let converted = Some(AttackMoveMemory::Dest(Target::Cell(cell)));
        assert_eq!(
            w.obj(a).unwrap().beh.aside,
            Some((None, elsewhere)),
            "{how:?}"
        );
        assert_eq!(
            w.obj(b).unwrap().beh.aside,
            Some((Some(Target::Object(other)), None)),
            "{how:?}"
        );
        assert_eq!(w.obj(c).unwrap().attack_move, converted, "{how:?}");
        assert_eq!(w.obj(d).unwrap().beh.aside, Some((None, None)), "{how:?}");
        assert_eq!(w.obj(d).unwrap().attack_move, converted, "{how:?}");
        assert_eq!(w.obj(other).unwrap().attack_move, kept, "{how:?}");
    }
}

/// testkit's ruleset with PLANE, an invented flying aircraft type.
fn with_plane() -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("PLANE".into());
    r.sim.aircraft.insert(
        "PLANE".into(),
        UnitType {
            strength: Some(100),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(Locomotor::Fly),
            ..Default::default()
        },
    );
    r
}

/// One holder of each rarer back-reference to an object: a bomb it
/// planted, a charger list it is on, a unit it dropped, a spawn
/// controller's pending target and another's controller target, and an
/// aircraft's destination.
struct Holders {
    bomb: ObjectId,
    charger: ObjectId,
    dropped: ObjectId,
    pending: ObjectId,
    controller: ObjectId,
    plane: ObjectId,
}

/// Makes each kind of [`Holders`] refer to `gone`; the charger list also
/// names `other`.
fn hold(w: &mut World, gone: ObjectId, other: ObjectId) -> Holders {
    let it = Some(Target::Object(gone));
    let h = Holders {
        bomb: walker(w, 0, -6),
        charger: walker(w, 0, -4),
        dropped: walker(w, 0, -2),
        pending: walker(w, 0, 0),
        controller: walker(w, 0, 2),
        plane: testkit::spawn(w, Kind::Aircraft, "PLANE", 0, testkit::mid(SIZE, 0, -6), 0),
    };
    w.obj_mut(h.bomb).unwrap().combat.bomb = Some(Bomb {
        planter: Some(gone),
        house: None,
        planted: 0,
    });
    w.obj_mut(h.charger).unwrap().combat.chargers = vec![gone, other];
    w.obj_mut(h.dropped).unwrap().lift.dropper = Some(gone);
    w.obj_mut(h.pending).unwrap().spawn = Some(SpawnController {
        pending: it,
        ..Default::default()
    });
    w.obj_mut(h.controller).unwrap().spawn = Some(SpawnController {
        target: it,
        ..Default::default()
    });
    assert!(
        w.obj(h.plane).unwrap().air.is_some(),
        "void: PLANE has no aircraft state"
    );
    w.obj_mut(h.plane).unwrap().dest = it;
    h
}

// spec: S-0519 claim 8 (a bomb whose planter leaves forgets it), S-0518 claim 12 (a charger that leaves play leaves every charger list at once), S-0607 claim 14 (a dropper that leaves play: its dropped units no longer have a dropper), S-0506 claims 11 and 25 (a lost pending target is cleared; a lost controller target is cleared), S-0501 claim 53 (an aircraft does not hold the gone object as its destination)
#[test]
fn every_rarer_back_reference_is_dropped_after_a_removal_without_them() {
    for how in [How::Removed, How::Killed] {
        let mut w = testkit::game_with(&with_plane(), SIZE, 1);
        let gone = walker(&mut w, 1, 6);
        let other = walker(&mut w, 1, 8);
        // A first removal while no object holds any of them.
        let first = walker(&mut w, 1, 10);
        glasnost_sim::objects::remove(&mut w, first);
        let h = hold(&mut w, gone, other);
        match how {
            How::Removed => glasnost_sim::objects::remove(&mut w, gone),
            How::Killed => glasnost_sim::combat::death::kill(&mut w, gone, &killing_hit()),
        }
        assert!(
            w.obj(gone).is_none(),
            "void: {how:?} left the object in play"
        );
        let o = |id: ObjectId| w.obj(id).unwrap();
        assert_eq!(o(h.bomb).combat.bomb.unwrap().planter, None, "{how:?}");
        assert_eq!(o(h.charger).combat.chargers, vec![other], "{how:?}");
        assert_eq!(o(h.dropped).lift.dropper, None, "{how:?}");
        assert_eq!(
            o(h.pending).spawn.as_ref().unwrap().pending,
            None,
            "{how:?}"
        );
        assert_eq!(
            o(h.controller).spawn.as_ref().unwrap().target,
            None,
            "{how:?}"
        );
        assert_eq!(o(h.plane).dest, None, "{how:?}");
    }
}

// spec: S-0607 claims 12 and 14 (a lifter leaving play drops its gripped unit just before; then, as a dropper that leaves play, its dropped unit no longer has a dropper)
#[test]
fn a_drop_made_inside_the_removal_is_still_undone() {
    let mut w = lift_common::world(|_| {});
    let l = lift_common::spawn(&mut w, Kind::Vehicle, "LIFTER", 0, lift_common::at(0, 0));
    let v = lift_common::spawn(&mut w, Kind::Vehicle, "VICT", 1, lift_common::at(6, 0));
    lift_common::beam(&mut w, l, v);
    assert!(lift::seized(&w, v), "void: not seized");
    let c = lift_common::at(3, 0);
    assert!(
        lift_common::run_until(&mut w, 400, |w| w.obj(v).is_some_and(|o| o.pos.cell() == c)
            && lift_common::height(w, v) > 300),
        "void: the victim was not pulled over the cell"
    );
    assert_eq!(
        w.obj(v).unwrap().lift.dropper,
        None,
        "void: dropped already"
    );
    glasnost_sim::objects::remove(&mut w, l);
    assert!(lift::dropped(&w, v), "void: not dropped");
    assert_eq!(w.obj(v).unwrap().lift.dropper, None);
}

// Our engine's bookkeeping, not game behaviour (#402): the census of the
// rarer back-references lives only inside the action it was taken for,
// so none is left to be saved, hashed or read by a later action.
#[test]
fn no_census_outlives_its_action() {
    for how in [How::Removed, How::Killed] {
        let mut w = testkit::game_with(&with_plane(), SIZE, 1);
        let gone = walker(&mut w, 1, 6);
        let other = walker(&mut w, 1, 8);
        hold(&mut w, gone, other);
        assert!(w.census.is_none(), "{how:?}: before");
        match how {
            How::Removed => glasnost_sim::objects::remove(&mut w, gone),
            How::Killed => glasnost_sim::combat::death::kill(&mut w, gone, &killing_hit()),
        }
        assert!(w.census.is_none(), "{how:?}: after the action");
        testkit::run(&mut w, 1);
        assert!(w.census.is_none(), "{how:?}: after a tick");
    }
}

// The census is neither saved nor hashed (#402): a game saved and loaded between two removals goes on exactly as the one never saved.
// spec: S-0519 claim 8, S-0518 claim 12, S-0607 claim 14, S-0506 claims 11 and 25, S-0501 claim 53 (as above)
#[test]
fn a_save_and_load_between_removals_changes_nothing() {
    let mut w = testkit::game_with(&with_plane(), SIZE, 1);
    let gone = walker(&mut w, 1, 6);
    let other = walker(&mut w, 1, 8);
    let first = walker(&mut w, 1, 10);
    glasnost_sim::objects::remove(&mut w, first);
    let h = hold(&mut w, gone, other);
    testkit::run(&mut w, 1);
    let bytes = glasnost_sim::save::save(&w);
    let mut loaded =
        glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue.clone().unwrap()).unwrap();
    assert_eq!(
        glasnost_sim::save::state_hash(&loaded),
        glasnost_sim::save::state_hash(&w)
    );
    for x in [&mut w, &mut loaded] {
        glasnost_sim::objects::remove(x, gone);
        testkit::run(x, 1);
        glasnost_sim::objects::remove(x, other);
    }
    assert_eq!(
        glasnost_sim::save::state_hash(&loaded),
        glasnost_sim::save::state_hash(&w)
    );
    assert_eq!(
        loaded.obj(h.charger).unwrap().combat.chargers,
        Vec::<ObjectId>::new()
    );
    assert_eq!(
        w.obj(h.charger).unwrap().combat.chargers,
        Vec::<ObjectId>::new()
    );
}
