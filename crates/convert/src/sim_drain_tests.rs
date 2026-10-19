//! Import tests of S-0608 (drain) on invented data: every name and number
//! here is made up.

use glasnost_rules::sim::SimData;

use crate::inputs::{InputFile, Inputs};
use crate::{ImportOptions, convert};

fn file(name: &str, text: &str) -> InputFile {
    InputFile {
        name: name.into(),
        origin: "test".into(),
        bytes: text.as_bytes().to_vec(),
    }
}

fn sim(base: &str) -> (SimData, crate::Import) {
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    let r = glasnost_rules::resolve_ruleset(
        Some((import.tree.clone(), import.provenance.clone())),
        &[],
    )
    .unwrap_or_else(|e| panic!("{e}"));
    (r.ruleset.sim, import)
}

const RULES: &str = "[CombatDamage]\nDrainMoneyFrameDelay=-7\nDrainMoneyAmount=11\nDrainAnimationType=Suck\n\
    [Animations]\n1=Suck\n[Suck]\nRate=10\n\
    [VehicleTypes]\n1=Saucer\n[Saucer]\nSecondary=Sip\n\
    [BuildingTypes]\n1=Mill\n2=Hut\n[Mill]\nDrainable=yes\nResourceDestination=yes\n[Hut]\nStrength=10\n\
    [Weapons]\n1=Sip\n[Sip]\nDrainWeapon=yes\n";

/// spec: S-0608 Import mapping (DrainMoneyFrameDelay and DrainMoneyAmount integers, a negative period kept as given; DrainAnimationType an animation name; DrainWeapon, Drainable and ResourceDestination booleans, absent: no)
#[test]
fn drain_keys() {
    let (s, _) = sim(RULES);
    let g = &s.general;
    assert_eq!(g.drain_money_frame_delay, Some(-7));
    assert_eq!(g.drain_money_amount, Some(11));
    assert_eq!(g.drain_animation_type.as_deref(), Some("Suck"));
    assert_eq!(s.weapons["Sip"].drain_weapon, Some(true));
    assert_eq!(s.buildings["Mill"].combat.drainable, Some(true));
    assert_eq!(s.buildings["Mill"].econ.resource_destination, Some(true));
    assert_eq!(s.buildings["Hut"].combat.drainable, None);
    assert_eq!(s.buildings["Hut"].econ.resource_destination, None);
}

/// spec: S-0608 Import mapping (absent keys stay absent: period 100, amount 0 and no animation in the simulation)
#[test]
fn absent_drain_keys() {
    let (s, _) = sim("[General]\nX=1\n");
    let g = &s.general;
    assert!(
        g.drain_money_frame_delay.is_none()
            && g.drain_money_amount.is_none()
            && g.drain_animation_type.is_none()
    );
}

/// spec: S-0608 Import mapping (the keys are converted fields of S-0608)
#[test]
fn drain_coverage() {
    let (_, i) = sim(RULES);
    let row = |cat: &str, key: &str| {
        i.coverage
            .iter()
            .find(|r| r.category == cat && r.key == key)
            .map(|r| (r.outcome, r.spec.clone()))
            .unwrap_or_else(|| panic!("{cat} {key}"))
    };
    use crate::coverage::Outcome::Field;
    for key in [
        "DrainMoneyFrameDelay",
        "DrainMoneyAmount",
        "DrainAnimationType",
    ] {
        assert_eq!(
            row("global:CombatDamage", key),
            (Field, "S-0608".into()),
            "{key}"
        );
    }
}

/// spec: S-0608 Questions for the project lead (a payout period of exactly 0 is warned about; the proposed departure makes no payouts) and F1 (a negative period is valid data)
#[test]
fn a_zero_period_is_warned_about() {
    let warned = |text: &str| {
        sim(text)
            .1
            .warnings
            .into_iter()
            .any(|w| w.message.contains("DrainMoneyFrameDelay"))
    };
    assert!(warned("[CombatDamage]\nDrainMoneyFrameDelay=0\n"));
    assert!(!warned("[CombatDamage]\nDrainMoneyFrameDelay=-7\n"));
    assert!(!warned("[CombatDamage]\nDrainMoneyAmount=0\n"));
}
