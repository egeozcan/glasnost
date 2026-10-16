//! Tests of the planted-bomb, charge and air-strike fields of the `sim`
//! section (S-0519 Import mapping) on invented data (every name and number
//! here is made up).

use glasnost_rules::sim::SimData;

use crate::inputs::{InputFile, Inputs};
use crate::layers::LayerQuirks;
use crate::{ImportOptions, convert};

fn sim(base: &str) -> SimData {
    let inputs = Inputs {
        base: Some(InputFile {
            name: "rules.ini".into(),
            origin: "test".into(),
            bytes: base.as_bytes().to_vec(),
        }),
        ..Inputs::default()
    };
    let opts = ImportOptions {
        quirks: LayerQuirks::default(),
        ..ImportOptions::default()
    };
    let import = convert(&inputs, &opts);
    glasnost_rules::resolve_ruleset(Some((import.tree.clone(), import.provenance.clone())), &[])
        .unwrap_or_else(|e| panic!("{e}"))
        .ruleset
        .sim
}

const BASE: &str = "[CombatDamage]\nIvanWarhead=Boom\nIvanDamage=120\nIvanTimedDelay=300\n\
    IvanIconFlickerRate=6\nCanDetonateTimeBomb=yes\nCanDetonateDeathBomb=yes\n\
    C4Warhead=Demo\nC4Delay=0.05\n\
    [Warheads]\n1=Boom\n2=Demo\n3=Plant\n4=Disarm\n5=Strike\n\
    [Boom]\n[Demo]\n[Plant]\nIvanBomb=yes\n[Disarm]\nBombDisarm=yes\n[Strike]\nAirstrike=yes\n\
    [InfantryTypes]\n1=Bomber\n2=Sabo\n[Bomber]\nIvan=yes\nDoggie=yes\n\
    [Sabo]\nC4=yes\nPrimary=Kamikaze\n\
    [AircraftTypes]\n1=Jet\n2=EliteJet\n\
    [Jet]\n[EliteJet]\n\
    [VehicleTypes]\n1=Desig\n2=Det\n\
    [Desig]\nAirstrikeTeam=3\nAirstrikeTeamType=Jet\nAirstrikeRechargeTime=200\n\
    [Det]\nBombSight=4\n\
    [Kamikaze]\nSuicide=yes\n";

// spec: S-0519 Import mapping (the combat-damage keys: the bomb warhead name, damage, delay and flicker rate as integers, both permissions and the charge warhead; the charge delay a decimal)
#[test]
fn the_game_wide_bomb_and_charge_keys() {
    let g = sim(BASE).general;
    assert_eq!(g.ivan_warhead.as_deref(), Some("Boom"));
    assert_eq!(
        (g.ivan_damage, g.ivan_timed_delay, g.ivan_icon_flicker_rate),
        (Some(120), Some(300), Some(6))
    );
    assert_eq!(g.can_detonate_time_bomb, Some(true));
    assert_eq!(g.can_detonate_death_bomb, Some(true));
    assert_eq!(g.c4_warhead.as_deref(), Some("Demo"));
    assert_eq!(
        g.c4_delay.as_ref().map(|d| d.0.as_str()),
        Some("5e-2"),
        "the decimal as the importer holds it"
    );
}

// spec: S-0519 Import mapping (absent keys stay absent so that the engine defaults apply: no key is invented by the importer)
#[test]
fn absent_keys_stay_absent() {
    let g = sim("[CombatDamage]\n").general;
    assert!(g.ivan_damage.is_none() && g.ivan_timed_delay.is_none() && g.c4_delay.is_none());
    assert!(g.ivan_warhead.is_none() && g.c4_warhead.is_none());
}

// spec: S-0519 Import mapping (the warhead flags IvanBomb, BombDisarm and Airstrike; infantry Ivan and C4; the weapon flag Suicide; all booleans, absent no)
#[test]
fn the_flags() {
    let s = sim(BASE);
    assert_eq!(s.warheads["Plant"].ivan_bomb, Some(true));
    assert_eq!(s.warheads["Disarm"].bomb_disarm, Some(true));
    assert_eq!(s.warheads["Strike"].airstrike, Some(true));
    assert_eq!(s.warheads["Boom"].ivan_bomb, None);
    assert_eq!(s.infantry["Bomber"].ivan, Some(true));
    assert_eq!(s.infantry["Bomber"].doggie, Some(true));
    assert_eq!(s.infantry["Sabo"].c4, Some(true));
    assert_eq!(s.weapons["Kamikaze"].suicide, Some(true));
}

// spec: S-0519 Import mapping (BombSight an integer in cells; the strike keys: size, type and recharge, the elite values left absent so that the engine takes the normal ones)
#[test]
fn bomb_sight_and_strike_keys() {
    let s = sim(BASE);
    assert_eq!(s.vehicles["Det"].combat.bomb_sight, Some(4));
    let d = &s.vehicles["Desig"].combat;
    assert_eq!(d.airstrike_team, Some(3));
    assert_eq!(d.airstrike_team_type.as_deref(), Some("Jet"));
    assert_eq!(d.airstrike_recharge_time, Some(200));
    assert_eq!(
        (
            d.elite_airstrike_team,
            d.elite_airstrike_team_type.clone(),
            d.elite_airstrike_recharge_time
        ),
        (None, None, None)
    );
}

fn warnings(base: &str) -> Vec<String> {
    let inputs = Inputs {
        base: Some(InputFile {
            name: "rules.ini".into(),
            origin: "test".into(),
            bytes: base.as_bytes().to_vec(),
        }),
        ..Inputs::default()
    };
    convert(&inputs, &ImportOptions::default())
        .warnings
        .into_iter()
        .map(|w| w.message)
        .collect()
}

// spec: S-0519 F3 and Edge cases (a timed delay below 6, a flicker rate of 0 and a designator without a strike aircraft type are warned about as departures)
#[test]
fn bad_data_is_warned_about() {
    let w = warnings(
        "[CombatDamage]\nIvanTimedDelay=4\nIvanIconFlickerRate=0\n\
         [VehicleTypes]\n1=Desig\n[Desig]\nAirstrikeTeam=2\n",
    );
    assert!(
        w.iter().any(|m| m.contains("IvanTimedDelay 4 is below 6")),
        "{w:?}"
    );
    assert!(
        w.iter().any(|m| m.contains("IvanIconFlickerRate 0 is 0")),
        "{w:?}"
    );
    assert!(
        w.iter().any(|m| m.contains("no AirstrikeTeamType")),
        "{w:?}"
    );
    let ok = warnings(BASE);
    assert!(!ok.iter().any(|m| m.contains("S-0519")), "{ok:?}");
}
