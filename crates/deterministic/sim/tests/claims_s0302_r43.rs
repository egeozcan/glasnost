//! S-0302 r43 claim 10 (M8 intake 104, Q-2626): the sound reactions of a
//! berserk hit. Although it reports *unchanged*, it plays no damage sound
//! (S-0701 claim 20), and of S-0701 claim 28's announcements it can make
//! only the harvester one: the base and ally announcements come only from
//! hits on buildings, which a berserk hit leaves *unaffected*. So the hit
//! reports no `Effect::Hit` (the event the damage sound and the attack
//! announcements read), only `Sound::HarvesterAttacked`. Synthetic data
//! only: the combat ruleset of `testkit` with an invented berserk warhead.
//! Ledger: `docs/m8-claims/intake104.md`.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, WarheadType};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::combat::data::WarheadId;
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::world::{HouseId, ObjectId, World};

const SIZE: i32 = 24;

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.warheads.insert(
        "MAD".into(),
        WarheadType {
            verses: Some(vec![Decimal("0.5".into()); 11]),
            psychedelic: Some(true),
            ..Default::default()
        },
    );
    r
}

fn mad(w: &World) -> WarheadId {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "MAD")
        .expect("the invented berserk warhead") as u32
}

/// A subject of house `owner` (kind and type as given), an attacker of
/// house 0 when `owner` is 1, of house 1 otherwise.
fn world(kind: Kind, name: &str, owner: u8) -> (World, ObjectId, ObjectId) {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 3);
    let s = spawn(&mut w, kind, name, owner, mid(SIZE, 0, 0), 0);
    let foe = if owner == 1 { 0 } else { 1 };
    let a = spawn(&mut w, Kind::Vehicle, "CART", foe, mid(SIZE, -3, 0), 64);
    (w, s, a)
}

fn berserk_hit(w: &World, attacker: ObjectId) -> Hit {
    let house = w.obj(attacker).and_then(|o| o.owner);
    Hit {
        amount: 100,
        warhead: Some(mad(w)),
        distance: 0,
        attacker: Some(attacker),
        house,
        ignore_defences: false,
        no_escape: false,
    }
}

fn sounds(w: &World) -> Vec<Sound> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Sound { sound, .. } => Some(*sound),
            _ => None,
        })
        .collect()
}

// spec: S-0302 claim 10 (r43, Q-2626: a berserk hit reports unchanged but makes no sound reaction of the hit: the target type's damage sound, which S-0701 claim 20 gives other unchanged hits, is not played; of S-0701 claim 28's announcements only the harvester one can follow, so no hit event for the attack announcements either), for a vehicle and an infantry unit of either house
#[test]
fn claim_10_a_berserk_hit_reports_no_hit_event() {
    for (kind, name) in [(Kind::Vehicle, "CART"), (Kind::Infantry, "WALKER")] {
        for owner in [0u8, 1] {
            let (mut w, s, a) = world(kind, name, owner);
            w.effects.clear();
            let h = berserk_hit(&w, a);
            assert_eq!(damage::apply(&mut w, s, h), HitResult::Unchanged);
            assert!(
                !w.effects.iter().any(|e| matches!(e, Effect::Hit { .. })),
                "{name} of house {owner}: no hit event, so no damage sound"
            );
            assert_eq!(
                sounds(&w),
                vec![Sound::HarvesterAttacked],
                "{name} of house {owner}: only the harvester announcement's event, which the audio makes only for the local player's harvester"
            );
        }
    }
}

// spec: S-0302 claim 10 (r43: the control: an ordinary hit that takes health and leaves the target alive reports its hit event, the one S-0701 claim 20's damage sound reads, where the berserk hit above reports none)
#[test]
fn claim_10_an_ordinary_hit_reports_its_hit_event() {
    let (mut w, s, a) = world(Kind::Vehicle, "CART", 1);
    w.effects.clear();
    let he = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "HE")
        .expect("the combat ruleset's warhead") as u32;
    let h = Hit {
        amount: 50,
        warhead: Some(he),
        distance: 0,
        attacker: Some(a),
        house: Some(HouseId(0)),
        ignore_defences: false,
        no_escape: false,
    };
    assert_ne!(damage::apply(&mut w, s, h), HitResult::Unaffected);
    assert!(!damage::dead(w.obj(s).expect("the subject")));
    assert!(w.effects.iter().any(|e| matches!(e, Effect::Hit { .. })));
}
