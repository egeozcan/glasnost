//! Claim tests for the radiation-immune deployed rows of S-0215 claim 31
//! with the field level of S-0525 claim 7 (scenario RF4 part d, S-0215
//! scenario B19) on synthetic data.

#[path = "deploy_common/mod.rs"]
mod deploy_common;

use deploy_common::*;
use glasnost_rules::sim::{WarheadType, WeaponType};
use glasnost_sim::radiation;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::World;

fn dec(s: &str) -> glasnost_rules::sim::Decimal {
    glasnost_rules::sim::Decimal(s.into())
}

/// The deploy ruleset plus IMM, a deployed infantry type that fires and is
/// immune to radiation, with the radiation weapon RadRifle (level 900,
/// spread 0) in its second slot: R = trunc(900 / 3) = 300.
fn world() -> World {
    let mut r = ruleset();
    let mut imm = r.sim.infantry["DEPINF"].clone();
    imm.immune_to_radiation = Some(true);
    imm.secondary = Some("RadRifle".into());
    r.sim.infantry.insert("IMM".into(), imm);
    r.registries.get_mut("infantry").unwrap().push("IMM".into());
    r.sim.warheads.insert(
        "RADW".into(),
        WarheadType {
            cell_spread: Some(dec("0")),
            verses: Some(vec![dec("0"); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "RadRifle".into(),
        WeaponType {
            damage: Some(0),
            rof: Some(30),
            range: Some(dec("4")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("RADW".into()),
            rad_level: Some(900),
            omni_fire: Some(true),
            ..Default::default()
        },
    );
    let g = &mut r.sim.general;
    g.rad_duration_multiple = Some(2);
    g.rad_application_delay = Some(15);
    g.rad_level_max = Some(10_000);
    g.rad_level_delay = Some(60);
    g.rad_light_delay = Some(60);
    g.rad_level_factor = Some(dec("0.0625"));
    g.rad_light_factor = Some(dec("0.5"));
    g.rad_tint_factor = Some(dec("1.0"));
    g.rad_color = Some([0, 255, 0]);
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

fn deployed(w: &World, i: glasnost_sim::world::ObjectId) -> bool {
    w.obj(i)
        .unwrap()
        .infantry
        .as_ref()
        .and_then(|s| s.seq)
        .is_some_and(glasnost_sim::infantry::Seq::is_deploy)
}

fn ready_unit(w: &mut World) -> glasnost_sim::world::ObjectId {
    let i = put(w, "IMM", Kind::Infantry, 0, 0, 0);
    deploy(w, i);
    for _ in 0..200 {
        if deployed(w, i) {
            break;
        }
        step(w);
    }
    assert!(deployed(w, i));
    i
}

// spec: S-0215 claim 31 and S-0525 claim 7 (RF4 part d, B19 c: a deployed immune unit on a clean cell fires at its own cell, which makes a field centred there; the interval equals the deployed-fire sequence's frame count; then the field's level, at least R, keeps it from firing: the interval is D_r + U(10, 20))
#[test]
fn an_immune_deployer_fires_until_its_cell_holds_a_centre_field() {
    let mut w = world();
    let i = ready_unit(&mut w);
    let cell = w.obj(i).unwrap().pos.cell();
    assert_eq!(radiation::centre_field_level(&w, cell), None);
    let p = processings(&mut w, i, 900);
    assert!(
        radiation::centre_field_level(&w, cell).is_some() || w.radiation.fields.is_empty(),
        "it fired at its own cell"
    );
    let delays: Vec<i32> = p.iter().map(|x| x.1).collect();
    // The shot interval is the frame count of the deployed-fire sequence (2).
    assert!(delays.contains(&2), "{delays:?}");
    // Afterwards the delays are D_r + 10 to 20 (a guard rate delay of 1 and more).
    let dr = delays
        .iter()
        .copied()
        .filter(|&d| d != 2)
        .min()
        .unwrap_or(0);
    assert!(dr >= 10, "{delays:?}");
}

// spec: S-0215 claim 31 and S-0525 claim 7 (RF4 part d: exposure that comes from a field centred on a neighbour does not suppress the shot; only a field centred on the unit's cell does)
#[test]
fn a_neighbouring_centre_does_not_suppress_the_shot() {
    let mut w = world();
    let i = ready_unit(&mut w);
    let cell = w.obj(i).unwrap().pos.cell();
    let rr = w.rules().combat.weapon_index["RadRifle"];
    let neighbour = glasnost_map::CellXY::new(cell.x + 1, cell.y);
    radiation::detonation(&mut w, rr, neighbour);
    assert_eq!(radiation::centre_field_level(&w, cell), None);
    processings(&mut w, i, 600);
    assert!(
        radiation::centre_field_level(&w, cell).is_some(),
        "it fired at its own cell: a field is centred there"
    );
}

// spec: S-0215 claim 31 and S-0525 claim 7 (a field centred on the unit's cell whose level is at least R keeps it from firing, and the level falls with the time left until it is below R)
#[test]
fn a_centre_field_at_or_above_the_threshold_withholds_the_shot() {
    let mut w = world();
    let i = ready_unit(&mut w);
    let cell = w.obj(i).unwrap().pos.cell();
    let rr = w.rules().combat.weapon_index["RadRifle"];
    radiation::detonation(&mut w, rr, cell);
    let p = processings(&mut w, i, 700);
    assert!(!p.is_empty());
    assert!(p.iter().all(|x| x.1 >= 10), "no shot: {p:?}");
    assert_eq!(w.radiation.fields.len(), 1, "no top-up either");
}

// spec: S-0215 claim 31 (revision 51, answers Q-0983) and S-0525 claims 1 and 4: the own-cell shot is made in the processing that decides to fire, with no wait for any fire frame of the deployed-fire sequence; the own-cell target is gone at the end of that processing and the field centred there exists in the same tick; the next processing comes after the sequence's frame count
#[test]
fn the_own_cell_shot_is_made_in_the_deciding_processing() {
    let mut w = world();
    let i = ready_unit(&mut w);
    let cell = w.obj(i).unwrap().pos.cell();
    assert_eq!(radiation::centre_field_level(&w, cell), None);
    let mut prev = w.obj(i).unwrap().beh.last;
    let mut shot = None;
    for _ in 0..600 {
        step(&mut w);
        let last = w.obj(i).unwrap().beh.last;
        if last != prev {
            prev = last;
            if last.is_some_and(|l| l.1 == 2) {
                shot = last;
                break;
            }
        }
    }
    let (tick, delay) = shot.expect("a processing that fires");
    assert_eq!(delay, 2, "the frame count of the deployed-fire sequence");
    assert_eq!(
        tick + 1,
        w.tick,
        "found in the step that made it (the world tick has moved on)"
    );
    assert!(
        radiation::centre_field_level(&w, cell).is_some(),
        "the field exists in the tick of the shot"
    );
    assert!(
        w.obj(i).unwrap().target.is_none(),
        "the own-cell target is dropped in the same processing"
    );
}

// spec: S-0525 claims 1 and 4 (revision 2, scenario RF4 part e) and S-0215 claim 31: with a live field centred on the unit's cell whose level has fallen below the firing threshold, the own-cell shot's top-up is effective in the tick of the deciding processing, not at a later tick or animation frame
#[test]
fn the_own_cell_top_up_is_effective_in_the_deciding_tick() {
    let mut w = world();
    let i = ready_unit(&mut w);
    let cell = w.obj(i).unwrap().pos.cell();
    let rr = w.rules().combat.weapon_index["RadRifle"];
    radiation::detonation(&mut w, rr, cell);
    let mut prev = w.obj(i).unwrap().beh.last;
    let mut before = radiation::centre_field_level(&w, cell).expect("a field");
    let mut topped = false;
    for _ in 0..3000 {
        step(&mut w);
        let last = w.obj(i).unwrap().beh.last;
        let level = radiation::centre_field_level(&w, cell);
        if last != prev {
            prev = last;
            if last.is_some_and(|l| l.1 == 2) {
                let after = level.expect("the field is still there");
                assert!(after > before, "topped up in the tick of the shot");
                assert!(
                    w.obj(i).unwrap().target.is_none(),
                    "the own-cell target is dropped in that processing"
                );
                // The deployed-fire sequence makes no second shot: the level only falls.
                for _ in 0..3 {
                    step(&mut w);
                }
                let later = radiation::centre_field_level(&w, cell).expect("field");
                assert!(later <= after, "no extra shot from the sequence");
                topped = true;
                break;
            }
        }
        before = level.expect("the field persists until the shot");
    }
    assert!(topped, "the unit fired once the level had fallen");
}
