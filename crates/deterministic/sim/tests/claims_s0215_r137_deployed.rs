//! Claim tests for S-0215 r137, sim issues S11: deployed and deploying
//! infantry in the behaviour processings. *Attack*'s deployed row with or
//! without a target and for the owners it covers (claim 23, #230), the
//! area-fire deployer in *unload* (claim 30, #232), the own-cell shot of a
//! radiation-immune deployed unit (claim 31, #233) and deployed infantry of
//! the houses that are not human-controlled in *move* (claim 22, #235).
//! Synthetic data of `deploy_common`; every name and number is invented.
//! Ledger: `docs/m8-claims/sim-issues.md`, *Batch S11*.

mod deploy_common;

use deploy_common::*;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ProjectileType, WarheadType, WeaponType};
use glasnost_sim::infantry::Seq;
use glasnost_sim::radiation;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseRole, ObjectId, Target, World};

fn seq(w: &World, i: ObjectId) -> Option<Seq> {
    w.obj(i).unwrap().infantry.as_ref().and_then(|s| s.seq)
}

fn deployed(w: &World, i: ObjectId) -> bool {
    seq(w, i).is_some_and(Seq::is_deploy)
}

/// Deploys `i` and steps until its *deploy* sequence has ended in
/// *deployed*.
fn deploy_and_wait(w: &mut World, i: ObjectId) {
    deploy(w, i);
    for _ in 0..200 {
        if seq(w, i) == Some(Seq::Deployed) {
            return;
        }
        step(w);
    }
    assert_eq!(seq(w, i), Some(Seq::Deployed), "the infantry deployed");
}

/// Makes house `h` a test-only computer-controlled player house (S-0215
/// Terms), or the neutral house when `neutral`.
fn computer(w: &mut World, h: usize, neutral: bool) {
    w.houses[h].human = false;
    if neutral {
        w.houses[h].role = HouseRole::Neutral;
    }
}

/// Steps until `i` has been processed at or after tick `from`; returns the
/// delay that processing chose.
fn next_processing(w: &mut World, i: ObjectId, from: i32) -> i32 {
    for _ in 0..40 {
        step(w);
        if let Some((at, d)) = w.obj(i).unwrap().beh.last
            && at >= from
        {
            return d;
        }
    }
    panic!("never processed");
}

// ---------------------------------------------------------------------------
// #230: claim 23's deployed row

// spec: S-0215 claim 23 (third infantry row: a deployed unit of a human-controlled owner runs its deployed-firing rule, with or without a target) and S-0517 claim 16 (a jumpjet type left targetless does not fall back; scenario P7 f)
#[test]
fn issue_230_targetless_deployed_jumpjet_keeps_attack() {
    let mut w = game();
    let i = put(&mut w, "DEPJET", Kind::Infantry, 0, 0, 0);
    deploy_and_wait(&mut w, i);
    let tick = w.tick;
    let o = w.obj_mut(i).unwrap();
    o.target = None;
    o.beh.force(Behaviour::Attack, tick, true);
    for n in 0..200 {
        step(&mut w);
        assert!(deployed(&w, i), "still deployed at step {n}");
        assert_eq!(
            behaviour(&w, i),
            (Some(Behaviour::Attack), None),
            "step {n}: a targetless deployed jumpjet type does not fall back"
        );
    }
}

// spec: S-0215 claim 23 (third infantry row, with or without a target) and S-0517 claim 16 (a range search at once; a found target becomes its target)
#[test]
fn issue_230_targetless_deployed_unit_in_attack_searches() {
    let mut w = game();
    let i = put(&mut w, "DEPINF", Kind::Infantry, 0, 0, 0);
    deploy_and_wait(&mut w, i);
    let near = put(&mut w, "TANK", Kind::Vehicle, 1, 3, 0);
    let tick = w.tick;
    let o = w.obj_mut(i).unwrap();
    o.target = None;
    o.beh.force(Behaviour::Attack, tick, true);
    let mut found = false;
    for n in 0..80 {
        step(&mut w);
        assert_eq!(
            behaviour(&w, i),
            (Some(Behaviour::Attack), None),
            "step {n}: attack stays current, nothing queued (target {:?})",
            w.obj(i).unwrap().target
        );
        if w.obj(i).unwrap().target == Some(Target::Object(near)) {
            found = true;
            break;
        }
    }
    assert!(found, "the range search found the enemy in range");
}

// spec: S-0215 claim 23 (the third infantry row is for a human-controlled owner; Terms, houses in our engine: a test-only computer-controlled player house keeps the original's rules) and claim 37 (g) (original only: a deployed unit of a computer-controlled owner whose type fires when deployed undeploys when its target is out of range)
#[test]
fn issue_230_deployed_row_is_not_for_a_computer_controlled_owner() {
    let mut w = game();
    let i = put(&mut w, "DEPINF", Kind::Infantry, 0, 0, 0);
    deploy_and_wait(&mut w, i);
    let far = put(&mut w, "TANK", Kind::Vehicle, 1, 14, 0);
    computer(&mut w, 0, false);
    let tick = w.tick;
    let o = w.obj_mut(i).unwrap();
    o.target = Some(Target::Object(far));
    o.beh.force(Behaviour::Attack, tick, true);
    next_processing(&mut w, i, tick);
    assert_eq!(
        seq(&w, i),
        Some(Seq::Undeploy),
        "the general rules' approach step: it undeploys"
    );
    assert_eq!(w.obj(i).unwrap().target, Some(Target::Object(far)));
}

// ---------------------------------------------------------------------------
// #232: claim 30's area-fire deployer

/// AREA: a deployer infantry type that does not deploy to fire, with a
/// negative undeploy delay; its secondary is an area-fire copy of the
/// rifle. With `spray`, the area-fire weapon is its primary and its type
/// has the spray-attack flag.
fn area_rules(spray: bool, named: bool) -> Ruleset {
    let mut r = ruleset();
    r.globals.quirks.named_deploy_area_fire = named;
    let mut area = r.sim.weapons["Rifle"].clone();
    area.area_fire = Some(true);
    r.sim.weapons.insert("AreaRifle".into(), area);
    let mut t = r.sim.infantry["DEPINF"].clone();
    t.deploy_fire = None;
    if spray {
        t.primary = Some("AreaRifle".into());
        t.spray_attack = Some(true);
    } else {
        t.secondary = Some("AreaRifle".into());
    }
    r.registries
        .get_mut("infantry")
        .unwrap()
        .push("AREA".into());
    r.sim.infantry.insert("AREA".into(), t);
    r
}

/// Deploys an AREA unit and returns (its target, its cell, the delay of
/// the *unload* processing) once that processing has run.
fn area_unload(r: &Ruleset) -> (Option<Target>, glasnost_map::CellXY, i32) {
    let mut w = testkit::game_with(r, SIZE, 7);
    testkit::run(&mut w, 4);
    let i = put(&mut w, "AREA", Kind::Infantry, 0, 0, 0);
    let from = w.tick;
    deploy(&mut w, i);
    for _ in 0..10 {
        step(&mut w);
        let o = w.obj(i).unwrap();
        if seq(&w, i) == Some(Seq::Deploy)
            && let Some((at, d)) = o.beh.last
            && at >= from
        {
            assert_eq!(o.beh.current, Some(Behaviour::Guard), "unload forces guard");
            return (o.target, o.pos.cell(), d);
        }
    }
    panic!("unload was never processed");
}

// spec: S-0215 claim 30 (an undeployed unit whose deploy weapon, the secondary, has the area-fire flag takes its own cell as its target; any other type: 450 ticks) and Edge cases (quirks.named_deploy_area_fire off: no type is singled out by name); Q-3100 (interim: no type is singled out with the switch on either)
#[test]
fn issue_232_area_fire_deployer_takes_its_own_cell() {
    for named in [false, true] {
        let (target, cell, d) = area_unload(&area_rules(false, named));
        assert_eq!(
            target,
            Some(Target::Cell(cell)),
            "named_deploy_area_fire={named}: its own cell is its target"
        );
        assert_eq!(d, 450, "named_deploy_area_fire={named}: 450 ticks");
    }
}

// spec: S-0215 claim 30 and Terms (the deploy weapon is the primary for a type with the spray-attack flag)
#[test]
fn issue_232_spray_attack_deploy_weapon_is_the_primary() {
    let (target, cell, d) = area_unload(&area_rules(true, false));
    assert_eq!(target, Some(Target::Cell(cell)));
    assert_eq!(d, 450);
}

// spec: S-0215 claim 30 (a deploy weapon without the area-fire flag: no new target; the area-fire primary of a type without the spray-attack flag is not its deploy weapon)
#[test]
fn issue_232_area_fire_primary_without_spray_attack_aims_at_nothing() {
    let mut r = area_rules(true, false);
    r.sim.infantry.get_mut("AREA").unwrap().spray_attack = None;
    let (target, _, d) = area_unload(&r);
    assert_eq!(target, None);
    assert_eq!(d, 450);
}

// ---------------------------------------------------------------------------
// #233: claim 31's own-cell shot

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

/// IMM: a radiation-immune deploy-fire infantry type whose secondary
/// weapon has a radiation level, an arcing visible projectile, and a fire
/// point 150 leptons above the unit.
fn immune_world() -> World {
    let mut r = ruleset();
    let mut imm = r.sim.infantry["DEPINF"].clone();
    imm.immune_to_radiation = Some(true);
    imm.secondary = Some("RadRifle".into());
    imm.combat.secondary_fire_flh = Some([0, 0, 150]);
    r.sim.infantry.insert("IMM".into(), imm);
    r.registries.get_mut("infantry").unwrap().push("IMM".into());
    r.sim.projectiles.insert(
        "Arc".into(),
        ProjectileType {
            arcing: Some(true),
            ..Default::default()
        },
    );
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
            projectile: Some("Arc".into()),
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

// spec: S-0215 claim 31 (Q-0983: the own cell is the target for the firing check and this immediate shot; the processing clears it after firing) and S-0306 claim 3 (the launch is computed towards the reference point of the firer's target)
#[test]
fn issue_233_the_own_cell_shot_is_launched_towards_the_own_cell() {
    let mut w = immune_world();
    let i = put(&mut w, "IMM", Kind::Infantry, 0, 0, 0);
    deploy_and_wait(&mut w, i);
    assert!(w.obj(i).unwrap().target.is_none(), "it holds no target");
    let cell = w.obj(i).unwrap().pos.cell();
    let mut prev = w.obj(i).unwrap().beh.last;
    let mut firing = None;
    for _ in 0..600 {
        step(&mut w);
        let last = w.obj(i).unwrap().beh.last;
        if last != prev {
            prev = last;
            // The firing row's delay: the deployed-fire frame count, 2.
            if last.is_some_and(|l| l.1 == 2) {
                firing = last;
                break;
            }
        }
    }
    let (tick, _) = firing.expect("a processing that takes the firing row");
    assert_eq!(w.obj(i).unwrap().last_fired, tick, "the shot was made");
    assert!(
        w.obj(i).unwrap().target.is_none(),
        "the own cell is cleared after the shot"
    );
    for _ in 0..300 {
        step(&mut w);
    }
    assert!(
        radiation::centre_field_level(&w, cell).is_some(),
        "a field is centred on its cell"
    );
}

// ---------------------------------------------------------------------------
// #235: claim 22's deployed infantry in move

/// DEPINF of house 0, deployed, in *move* with a destination three cells
/// east; house 0 is made computer-controlled (`computer`, neutral when
/// `neutral`) and the type's undeploy delay set to `delay`. Returns (its
/// sequence, whether it has a destination, the delay chosen) after its
/// first *move* processing.
fn deployed_move(owner: Option<bool>, delay: i32) -> (Option<Seq>, bool, i32) {
    let mut r = ruleset();
    r.sim.infantry.get_mut("DEPINF").unwrap().undeploy_delay = Some(delay);
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    let i = put(&mut w, "DEPINF", Kind::Infantry, 0, 0, 0);
    if let Some(s) = w.obj_mut(i).unwrap().infantry.as_mut() {
        s.seq = Some(Seq::Deployed);
    }
    if let Some(neutral) = owner {
        computer(&mut w, 0, neutral);
    }
    let s = w.tick;
    let o = w.obj_mut(i).unwrap();
    o.beh.force(Behaviour::Move, s, true);
    o.dest = Some(Target::Cell(at(3, 0)));
    let d = next_processing(&mut w, i, s);
    let o = w.obj(i).unwrap();
    (seq(&w, i), o.dest.is_some(), d)
}

// spec: S-0215 claim 22 (Infantry: a deployed unit of a human-controlled owner clears its destination and chooses 1 tick; one of a computer-controlled owner whose undeploy delay is negative starts its undeploy sequence with that sequence's frame count as the delay) and Terms (houses in our engine: the neutral and special houses and a test-only computer-controlled player house keep the original's rules)
#[test]
fn issue_235_deployed_infantry_of_a_computer_owner_in_move_undeploys() {
    let (s, dest, d) = deployed_move(None, -1);
    assert!(s.is_some_and(Seq::is_deploy), "player house: {s:?}");
    assert!(!dest, "player house: the destination is cleared");
    assert_eq!(d, 1, "player house: 1 tick");
    for neutral in [true, false] {
        let (s, _, d) = deployed_move(Some(neutral), -1);
        // The undeploy sequence of DEPINF has 6 frames (deploy_common).
        assert_eq!((s, d), (Some(Seq::Undeploy), 6), "neutral={neutral}");
    }
}

// spec: S-0215 claim 22 (Infantry: a deployed unit of a computer-controlled owner whose undeploy delay is 0 or more follows the general rule: it keeps its destination and takes the standard delay)
#[test]
fn issue_235_deployed_infantry_of_a_computer_owner_with_a_delay_follows_the_general_rule() {
    let (s, dest, _) = deployed_move(Some(true), 20);
    assert!(s.is_some_and(Seq::is_deploy), "it stays deployed: {s:?}");
    assert!(dest, "the general rule keeps the destination");
}
