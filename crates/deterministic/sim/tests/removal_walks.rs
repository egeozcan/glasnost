//! The object-table walks of a removal and a killing hit (#402): the
//! passes that now serve several back-reference kinds at once, and the
//! walks a removal skips, must leave every back-reference changed as the
//! separate walks did. Synthetic data.

use glasnost_sim::combat::damage::Hit;
use glasnost_sim::rules::Kind;
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
    testkit::spawn(w, Kind::Infantry, "WALKER", house, testkit::mid(SIZE, dx, 0), 0)
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
        assert_eq!(w.obj(a).unwrap().beh.aside, Some((None, elsewhere)), "{how:?}");
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
