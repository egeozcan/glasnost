//! S-0612 claims 3 and 7 with a drain-mode superweapon of the psychic
//! domination kind (#290): a drain-mode order in *ready* is a launch of the
//! kind, so it is refused while a domination is in force, as an ordinary
//! order is (S-0512 claims 8 and 19). Synthetic rules only: the `sw_common`
//! superweapon ruleset plus an invented type DOM (list position 12), its
//! building DOMB and the invented animations LEAD and SWEEP; every name and
//! number is made up.

mod sw_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, BuildingType, Decimal};
use glasnost_rules::sim_econ::SuperweaponType;
use glasnost_sim::domination::{self, Phase};
use glasnost_sim::superweapons::{DrainState, SwEvent};
use glasnost_sim::world::World;
use sw_common::{ALPHA, BETA, at, fire_now, ready_tick, rec};

/// DOM's list position (after `sw_common`'s twelve).
const DOM: u32 = 12;

fn reg(r: &mut Ruleset, registry: &str, name: &str) {
    let l = r.registries.entry(registry.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn rules(drain: bool) -> Ruleset {
    let mut r = sw_common::ruleset();
    r.sim.superweapons.insert(
        "DOM".into(),
        SuperweaponType {
            kind: Some("PsychicDominator".into()),
            action: Some("PsychicDominator".into()),
            recharge_time: Some(Decimal("0.1".into())),
            use_charge_drain: Some(drain),
            ..Default::default()
        },
    );
    reg(&mut r, "superweapons", "DOM");
    let mut b = BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    b.econ.super_weapon = Some("DOM".into());
    r.sim.buildings.insert("DOMB".into(), b);
    reg(&mut r, "buildings", "DOMB");
    for (n, f) in [("LEAD", 8), ("SWEEP", 12)] {
        r.sim.animations.insert(
            n.into(),
            AnimationType {
                frame_count: Some(f),
                ..Default::default()
            },
        );
        reg(&mut r, "animations", n);
    }
    let s = &mut r.sim.general.superweapons;
    s.dominator_first_anim = Some("LEAD".into());
    s.dominator_second_anim = Some("SWEEP".into());
    r
}

/// Both houses hold a DOMB and their DOM records are ready.
fn armed(r: &Ruleset) -> World {
    let mut w = sw_common::game(r);
    sw_common::spawn(&mut w, "DOMB", ALPHA, -6, -6);
    sw_common::spawn(&mut w, "DOMB", BETA, 6, 6);
    ready_tick(&mut w, ALPHA, DOM, 400);
    ready_tick(&mut w, BETA, DOM, 400);
    w
}

/// Alpha launches at (0, 0); then Beta fires at (3, 3) while Alpha's
/// domination is in force.
fn second_launch_by_beta(drain: bool) {
    let r = rules(drain);
    let mut w = armed(&r);
    fire_now(&mut w, ALPHA, DOM, at(0, 0));
    assert!(domination::in_force(&w));
    assert_eq!(w.domination.current.as_ref().map(|d| d.house), Some(ALPHA));
    assert_eq!(w.domination.anims.len(), 1);
    let launched = w.domination.current.as_ref().map(|d| d.launched);
    let beta_before = rec(&w, BETA, DOM);
    w.effects.clear();
    fire_now(&mut w, BETA, DOM, at(3, 3));
    let d = w.domination.current.as_ref();
    // spec: S-0612 claims 3 and 7 (a second launch while one is in force is refused, for any house)
    assert_eq!(
        d.map(|d| d.house),
        Some(ALPHA),
        "Alpha's domination is still in force"
    );
    assert_eq!(d.map(|d| d.cell), Some(at(0, 0)));
    assert_eq!(d.map(|d| d.launched), launched);
    assert_eq!(w.domination.anims.len(), 1, "one lead-in animation");
    // spec: S-0512 claim 8 (a refusal because a domination is in force keeps the charge and the ready state; a refusal message for the ordering house)
    assert!(
        sw_common::events(&w).contains(&(SwEvent::Refused, Some(BETA), Some(DOM))),
        "Beta's order is refused"
    );
    let beta = rec(&w, BETA, DOM);
    assert!(beta.ready, "Beta's record stays ready");
    assert_eq!(beta.drain, beta_before.drain, "Beta's drain state is kept");
}

// Control: the ordinary type takes the refusal of the ordinary path.
// spec: S-0612 claims 3 and 7, S-0512 claim 8
#[test]
fn claim_3_ordinary_type_second_launch_is_refused() {
    second_launch_by_beta(false);
}

// spec: S-0612 claims 3 and 7, S-0512 claims 8 and 19 (#290)
#[test]
fn claim_3_drain_type_second_launch_by_another_house_is_refused() {
    second_launch_by_beta(true);
}

// spec: S-0612 claims 3 and 7, S-0512 claim 19 (ready -> active launches; active -> ready launches nothing; the next order in ready is a launch, refused while the domination is in force) (#290)
#[test]
fn claim_3_drain_type_relaunch_by_the_same_house_is_refused() {
    let r = rules(true);
    let mut w = armed(&r);
    fire_now(&mut w, ALPHA, DOM, at(0, 0));
    assert_eq!(rec(&w, ALPHA, DOM).drain, DrainState::Active);
    let launched = w.domination.current.as_ref().map(|d| d.launched);
    sw_common::step(&mut w);
    assert_eq!(
        w.domination.current.as_ref().map(|d| d.phase),
        Some(Phase::LeadIn { first: true })
    );
    // Active -> ready: no effect.
    fire_now(&mut w, ALPHA, DOM, at(0, 0));
    assert_eq!(rec(&w, ALPHA, DOM).drain, DrainState::Ready);
    w.effects.clear();
    // Ready while the domination is in force: refused.
    fire_now(&mut w, ALPHA, DOM, at(0, 0));
    assert_eq!(
        w.domination.current.as_ref().map(|d| d.launched),
        launched,
        "the first domination is still the one in force"
    );
    assert_eq!(w.domination.anims.len(), 1, "one lead-in animation");
    assert!(sw_common::events(&w).contains(&(SwEvent::Refused, Some(ALPHA), Some(DOM))));
    assert_eq!(
        rec(&w, ALPHA, DOM).drain,
        DrainState::Ready,
        "the refused order keeps the ready state"
    );
}
