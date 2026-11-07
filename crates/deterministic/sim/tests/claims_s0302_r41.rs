//! S-0302 r41 claim 10 (M8 intake 100): the reactions of a berserk hit.
//! A hit that makes its target berserk (or renews the state) changes no
//! health and reports *unchanged*. It sets off none of the hit's own damage
//! step reactions (anger, retaliation, scatter, trigger event, damage
//! effect, disguise loss), but the reactions a mobile target makes after
//! that step do follow: the no-threat fall-back (S-0215 claim 16), the
//! change of fear of an infantry unit (S-0107 claim 16), the crush
//! response of a vehicle (S-0215 claim 39) and the announcement that a
//! harvester is under attack (S-0701 claim 28). Request B of S-0208 claim
//! 15 and the scatter from damage are not built for any hit (seam).
//! Synthetic data only: the combat ruleset of `testkit` with an invented
//! berserk warhead and an invented no-threat behaviour. Ledger:
//! `docs/m8-claims/intake100.md`.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BehaviourProps, Decimal, WarheadType};
use glasnost_sim::combat::damage::{self, Hit, HitResult};
use glasnost_sim::combat::data::WarheadId;
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::rules::{Behaviour, Kind};
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
    // Scenario B5's behaviour file: `harmless` is the no-threat behaviour.
    r.sim.behaviours.insert(
        "harmless".into(),
        BehaviourProps {
            no_threat: Some(true),
            zombie: Some(false),
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

/// A subject of house 1 (kind and type as given), an attacker `A` of house 0.
fn world(kind: Kind, name: &str) -> (World, ObjectId, ObjectId) {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 3);
    let s = spawn(&mut w, kind, name, 1, mid(SIZE, 0, 0), 0);
    let a = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, -3, 0), 64);
    (w, s, a)
}

fn force(w: &mut World, id: ObjectId, b: Behaviour) {
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    w.obj_mut(id).unwrap().beh.force(b, t, stale);
}

fn berserk_hit(w: &World, attacker: Option<ObjectId>, house: Option<HouseId>) -> Hit {
    Hit {
        amount: 100,
        warhead: Some(mad(w)),
        distance: 0,
        attacker,
        house,
        ignore_defences: false,
        no_escape: false,
    }
}

fn slots(w: &World, id: ObjectId) -> (Option<Behaviour>, Option<Behaviour>) {
    let o = w.obj(id).unwrap();
    (o.beh.current, o.beh.queued)
}

/// The fall-back of an infantry unit of a player house queues *guard*
/// (S-0215 claim 14); nothing else queues it in these worlds (the berserk
/// entry queues *hunt*).
fn fell_back(w: &World, id: ObjectId) -> bool {
    slots(w, id).1 == Some(Behaviour::Guard)
}

// spec: S-0302 claim 10 (r41: a berserk hit changes no health, reports unchanged) and S-0215 claim 16 (the fall-back follows a berserk hit from a known attacker on an actor whose current behaviour has the no-threat property; the Hunt of the berserk entry is replaced by the behaviour the fall-back queues, S-0610 claim 4)
#[test]
fn claim_10_a_berserk_hit_makes_a_no_threat_infantry_unit_fall_back() {
    let (mut w, s, a) = world(Kind::Infantry, "WALKER");
    force(&mut w, s, Behaviour::Harmless);
    let h0 = w.obj(s).unwrap().health;
    let h = berserk_hit(&w, Some(a), Some(HouseId(0)));
    assert_eq!(damage::apply(&mut w, s, h), HitResult::Unchanged);
    let o = w.obj(s).unwrap();
    assert_eq!(o.health, h0, "no health changes");
    assert!(o.combat.berserk.is_some(), "the unit is berserk");
    assert!(fell_back(&w, s), "the no-threat fall-back ran");
    assert_eq!(
        slots(&w, s).1,
        Some(Behaviour::Guard),
        "guard replaced the queued hunt (claim 14, an infantry unit of a player house)"
    );
}

// spec: S-0302 claim 12 (r41: a berserk hit has none of the reactions below, whatever it reports, claim 10, so a target whose type can disguise itself but not permanently keeps its disguise and takes no disruption countdown; an ordinary hit that reports unchanged loses it) and claim 10 (r41: no disguise loss) (seam sweep 6: intake 100's row)
#[test]
fn seams6_a_berserk_hit_keeps_the_disguise() {
    use glasnost_sim::cloak::{Disguise, DisguiseAs};
    let mut r = rules();
    let mut disg = r.sim.infantry["WALKER"].clone();
    disg.combat.can_disguise = Some(true);
    r.sim.infantry.insert("DISG".into(), disg);
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("DISG".into());
    for berserk in [true, false] {
        let mut w = testkit::game_with(&r, SIZE, 3);
        let s = spawn(&mut w, Kind::Infantry, "DISG", 1, mid(SIZE, 0, 0), 0);
        let a = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, -3, 0), 64);
        let walker = w.rules().index_of(Kind::Infantry, "WALKER").unwrap();
        let t = w.tick;
        let o = w.obj_mut(s).unwrap();
        o.cloak.disguise = Some(Disguise {
            what: Some(DisguiseAs::Infantry(walker)),
            house: Some(HouseId(0)),
            since: t,
        });
        o.cloak.lockout_until = 0;
        let h = if berserk {
            berserk_hit(&w, Some(a), Some(HouseId(0)))
        } else {
            let sa = w
                .rules()
                .combat
                .warheads
                .iter()
                .position(|x| x.name == "SA")
                .expect("the combat ruleset's SA") as u32;
            Hit {
                amount: 4,
                warhead: Some(sa),
                ..berserk_hit(&w, Some(a), Some(HouseId(0)))
            }
        };
        let res = damage::apply(&mut w, s, h);
        assert_eq!(res, HitResult::Unchanged, "berserk {berserk}");
        let o = w.obj(s).unwrap();
        if berserk {
            assert!(
                o.combat.berserk.is_some(),
                "set-up: the hit made it berserk"
            );
            assert!(o.cloak.disguise.is_some(), "the disguise is kept");
            assert_eq!(o.cloak.lockout_until, 0, "no disruption countdown");
        } else {
            assert!(
                o.cloak.disguise.is_none(),
                "control: an ordinary hit loses it"
            );
            assert!(o.cloak.lockout_until > t, "control: the countdown");
        }
    }
}

// spec: S-0302 claim 10 (r41: each reaction that needs a known attacker needs one here too)
#[test]
fn claim_10_no_fall_back_without_a_known_attacker() {
    let (mut w, s, _) = world(Kind::Infantry, "WALKER");
    force(&mut w, s, Behaviour::Harmless);
    let h = berserk_hit(&w, None, Some(HouseId(0)));
    assert_eq!(damage::apply(&mut w, s, h), HitResult::Unchanged);
    assert!(w.obj(s).unwrap().combat.berserk.is_some());
    assert!(!fell_back(&w, s), "no attacker, no fall-back");
    assert_eq!(
        slots(&w, s).1,
        Some(Behaviour::Hunt),
        "the berserk entry's hunt stays"
    );
}

// spec: S-0215 claim 16 (only a no-threat behaviour falls back)
#[test]
fn claim_10_no_fall_back_without_the_no_threat_property() {
    let (mut w, s, a) = world(Kind::Infantry, "WALKER");
    force(&mut w, s, Behaviour::Guard);
    let h = berserk_hit(&w, Some(a), Some(HouseId(0)));
    assert_eq!(damage::apply(&mut w, s, h), HitResult::Unchanged);
    assert!(!fell_back(&w, s));
}

// spec: S-0302 claim 10 (r41: the three excepted cases, here an allied attacking house, report unaffected and nothing happens: no berserk, no fall-back, no fear)
#[test]
fn claim_10_an_allied_attacker_house_stops_the_hit_before_every_reaction() {
    let (mut w, s, a) = world(Kind::Infantry, "WALKER");
    force(&mut w, s, Behaviour::Harmless);
    // The attacker's house is the target's own, so allied.
    let h = berserk_hit(&w, Some(a), Some(HouseId(1)));
    assert_eq!(damage::apply(&mut w, s, h), HitResult::Unaffected);
    let o = w.obj(s).unwrap();
    assert!(o.combat.berserk.is_none());
    assert!(!fell_back(&w, s));
    assert_eq!(o.infantry.as_ref().unwrap().fear, 0, "no change of fear");
}

// spec: S-0302 claim 10 (r41) and S-0107 claim 16 (the change of fear follows a berserk hit from a known attacker: a unit below 100 that is neither fraidycat nor fearless goes to 100)
#[test]
fn claim_10_a_berserk_hit_changes_the_fear_of_an_infantry_unit() {
    let (mut w, s, a) = world(Kind::Infantry, "WALKER");
    let h = berserk_hit(&w, Some(a), Some(HouseId(0)));
    assert_eq!(damage::apply(&mut w, s, h), HitResult::Unchanged);
    assert_eq!(w.obj(s).unwrap().infantry.as_ref().unwrap().fear, 100);
}

// spec: S-0302 claim 10 (r41) and S-0107 claim 16 (a hit with no known attacker raises fear by the plain increment, F5: 25 above the red ratio, halved above the yellow ratio: 12 at full health)
#[test]
fn claim_10_a_berserk_hit_without_an_attacker_raises_fear_by_the_increment() {
    let (mut w, s, _) = world(Kind::Infantry, "WALKER");
    let h = berserk_hit(&w, None, Some(HouseId(0)));
    assert_eq!(damage::apply(&mut w, s, h), HitResult::Unchanged);
    assert_eq!(w.obj(s).unwrap().infantry.as_ref().unwrap().fear, 12);
}

// spec: S-0610 Edge cases (quirks.berserk_zero_duration, both positions) and S-0302 claim 10 (r41: a berserk hit with T = 0: on, the state starts and the reactions follow; off, the hit has no effect on the target, so no reaction follows; the interim rule of docs/m8-claims/intake100.md)
#[test]
fn claim_10_a_zero_duration_berserk_hit_follows_the_switch() {
    for on in [true, false] {
        let mut r = rules();
        r.globals.quirks.berserk_zero_duration = on;
        let mut w = testkit::game_with(&r, SIZE, 3);
        let s = spawn(&mut w, Kind::Infantry, "WALKER", 1, mid(SIZE, 0, 0), 0);
        let a = spawn(&mut w, Kind::Vehicle, "CART", 0, mid(SIZE, -3, 0), 64);
        force(&mut w, s, Behaviour::Harmless);
        let mut h = berserk_hit(&w, Some(a), Some(HouseId(0)));
        h.amount = 0;
        assert_eq!(damage::apply(&mut w, s, h), HitResult::Unchanged);
        assert_eq!(w.obj(s).unwrap().combat.berserk.is_some(), on);
        assert_eq!(
            fell_back(&w, s),
            on,
            "reactions only when the hit took effect"
        );
        assert_eq!(w.obj(s).unwrap().infantry.as_ref().unwrap().fear == 100, on);
    }
}

// spec: S-0302 claim 10 (r41: no anger change and no retaliation from a berserk hit: the target holds no target after it, the berserk entry having dropped it)
#[test]
fn claim_10_a_berserk_hit_causes_no_retaliation() {
    let (mut w, s, a) = world(Kind::Vehicle, "CART");
    let h = berserk_hit(&w, Some(a), Some(HouseId(0)));
    assert_eq!(damage::apply(&mut w, s, h), HitResult::Unchanged);
    assert!(w.obj(s).unwrap().target.is_none(), "no retaliation target");
    assert_ne!(slots(&w, s).0, Some(Behaviour::Attack));
}

// spec: S-0302 claim 10 (r41: a further berserk hit renews the state and the reactions follow it too)
#[test]
fn claim_10_a_renewing_berserk_hit_also_makes_the_actor_fall_back() {
    let (mut w, s, a) = world(Kind::Infantry, "WALKER");
    let h = berserk_hit(&w, Some(a), Some(HouseId(0)));
    assert_eq!(damage::apply(&mut w, s, h), HitResult::Unchanged);
    force(&mut w, s, Behaviour::Harmless);
    assert!(!fell_back(&w, s));
    assert_eq!(damage::apply(&mut w, s, h), HitResult::Unchanged);
    assert!(fell_back(&w, s), "the renewing hit triggers the fall-back");
}

// spec: S-0302 claim 10 (r41) and S-0701 claim 28 (the harvester-under-attack announcement is made for a berserk hit: the hit gives a presentation event naming the target's type, owner and position)
#[test]
fn claim_10_a_berserk_hit_gives_the_harvester_announcement_event() {
    let (mut w, s, a) = world(Kind::Vehicle, "CART");
    let pos = w.obj(s).unwrap().pos;
    w.effects.clear();
    let h = berserk_hit(&w, Some(a), Some(HouseId(0)));
    assert_eq!(damage::apply(&mut w, s, h), HitResult::Unchanged);
    let n = w
        .effects
        .iter()
        .filter(|e| {
            matches!(e, Effect::Sound { sound: Sound::HarvesterAttacked, of: Some(t), house: Some(hh), pos: Some(p) }
                if t.kind == Kind::Vehicle && *hh == HouseId(1) && *p == pos)
        })
        .count();
    assert_eq!(n, 1);
    // A berserk hit that reports unaffected gives none.
    let (mut w2, s2, a2) = world(Kind::Vehicle, "CART");
    w2.effects.clear();
    let h2 = berserk_hit(&w2, Some(a2), Some(HouseId(1)));
    assert_eq!(damage::apply(&mut w2, s2, h2), HitResult::Unaffected);
    assert!(!w2.effects.iter().any(|e| matches!(
        e,
        Effect::Sound {
            sound: Sound::HarvesterAttacked,
            ..
        }
    )));
}
