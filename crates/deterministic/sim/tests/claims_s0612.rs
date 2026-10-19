//! Claim tests of S-0612 (the psychic domination superweapon) and of the
//! S-0104 lighting state it times, on an empty synthetic map with invented
//! types: every name and number here is made up; no game data. Houses
//! Alpha (0) and Beta (1) of the testkit's sandbox game, not allied, on the
//! superweapon ruleset of `sw_common`. The ledger is
//! `docs/m7-claims/S-0612.md`.
//!
//! Added on top of `sw_common`:
//! - superweapon DOM (the psychic-domination kind word), its building DOMB
//!   (one cell);
//! - animations LEAD (8 image frames, no rate field: one stage per update)
//!   and SWEEP (12 image frames), the lead-in and sweep types;
//! - warhead PsiW (versus 1, cell spread 1), the domination warhead when a
//!   test wants damage (none otherwise);
//! - vehicles BOX (unarmed, strength 1000), STOIC (psionics-immune),
//!   BALLOON (balloon hover); infantry GRUNT (unarmed, strength 1000);
//!   building PAD (two by two cells);
//! - the activation sound `Psi`.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_num::Rounding;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, BuildingType, Decimal, WarheadType};
use glasnost_rules::sim_econ::SuperweaponType;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::domination::{self, DomEvent, Phase};
use glasnost_sim::lighting::{self, Choice};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::superweapons::{ProtectKind, Protection, SwEvent, Tint};
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, World};
use glasnost_sim::{mind, objects, save, testkit};
use sw_common::{ALPHA, BETA, at, fire, fire_now, ready_tick, rec};

/// DOM's list position (after `sw_common`'s twelve).
const DOM: u32 = 12;
/// STORM's list position in `sw_common`.
const STORM: u32 = sw_common::STORM;
/// The lead-in and sweep sprites' image-frame counts.
const LEAD_N: i32 = 8;
const SWEEP_N: i32 = 12;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn reg(r: &mut Ruleset, registry: &str, name: &str) {
    let l = r.registries.entry(registry.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The domination ruleset: no warhead, so the sweep deals no damage
/// (S-0303 claim 2), unless a test names one.
fn rules() -> Ruleset {
    let mut r = sw_common::ruleset();
    r.sim.infantry.get_mut("WALKER").unwrap().primary = None;
    r.sim.general.max_damage = Some(1_000_000);
    r.sim.superweapons.insert(
        "DOM".into(),
        SuperweaponType {
            kind: Some("PsychicDominator".into()),
            action: Some("PsychicDominator".into()),
            recharge_time: Some(d("0.1")),
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
    r.sim.buildings.insert(
        "PAD".into(),
        BuildingType {
            strength: Some(500),
            foundation: Some([2, 2]),
            ..Default::default()
        },
    );
    reg(&mut r, "buildings", "PAD");
    for (n, f) in [("LEAD", LEAD_N), ("SWEEP", SWEEP_N)] {
        r.sim.animations.insert(
            n.into(),
            AnimationType {
                frame_count: Some(f),
                ..Default::default()
            },
        );
        reg(&mut r, "animations", n);
    }
    r.sim.warheads.insert(
        "PsiW".into(),
        WarheadType {
            cell_spread: Some(d("1")),
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    let base = r.sim.vehicles["BUGGY"].clone();
    for n in ["BOX", "STOIC", "BALLOON"] {
        let mut t = base.clone();
        t.strength = Some(1000);
        t.primary = None;
        t.secondary = None;
        match n {
            "STOIC" => t.combat.immune_to_psionics = Some(true),
            "BALLOON" => t.balloon_hover = Some(true),
            _ => {}
        }
        r.sim.vehicles.insert(n.into(), t);
        reg(&mut r, "vehicles", n);
    }
    let mut g = r.sim.infantry["WALKER"].clone();
    g.strength = Some(1000);
    g.primary = None;
    r.sim.infantry.insert("GRUNT".into(), g);
    reg(&mut r, "infantry", "GRUNT");
    let s = &mut r.sim.general.superweapons;
    s.dominator_first_anim = Some("LEAD".into());
    s.dominator_second_anim = Some("SWEEP".into());
    s.psychic_dominator_activate_sound = Some("Psi".into());
    r
}

fn step(w: &mut World) {
    w.effects.clear();
    sw_common::step(w);
}

/// A game under `r` with a ready DOM for Alpha (its building at (−6, −6)).
fn armed(r: &Ruleset) -> World {
    let mut w = sw_common::game(r);
    sw_common::spawn(&mut w, "DOMB", ALPHA, -6, -6);
    ready_tick(&mut w, ALPHA, DOM, 400);
    w
}

fn spawn(w: &mut World, name: &str, h: HouseId, dx: i32, dy: i32) -> ObjectId {
    sw_common::spawn(w, name, h, dx, dy)
}

/// Infantry `name` of house `h` at standing position `p` of cell `c`.
fn put(w: &mut World, name: &str, h: HouseId, c: CellXY, p: u8) -> ObjectId {
    let ti = w.rules().index_of(Kind::Infantry, name).unwrap();
    let id = objects::create(w, Kind::Infantry, ti, Some(h));
    let (u, v) = glasnost_sim::walk::standing_point(p);
    let z = w.surface_at(256 * c.x + u, 256 * c.y + v);
    objects::place(w, id, Pos::new(256 * c.x + u, 256 * c.y + v, z), 0);
    id
}

fn dom_events(w: &World) -> Vec<DomEvent> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Domination { event, .. } => Some(*event),
            _ => None,
        })
        .collect()
}

/// Fires DOM for Alpha at `c` and steps the tick that applies it; returns
/// that tick *L*.
fn launch(w: &mut World, c: CellXY) -> i32 {
    fire_now(w, ALPHA, DOM, c);
    w.tick - 1
}

/// Steps until `ev` happens; returns the tick it happened in.
fn until_event(w: &mut World, ev: fn(&DomEvent) -> bool, max: i32) -> i32 {
    for _ in 0..max {
        step(w);
        if dom_events(w).iter().any(ev) {
            return w.tick - 1;
        }
    }
    panic!("event not reached in {max} ticks");
}

fn sweep_tick(w: &mut World) -> i32 {
    until_event(w, |e| *e == DomEvent::Sweep, 200)
}

fn owner(w: &World, id: ObjectId) -> Option<HouseId> {
    w.obj(id).and_then(|o| o.owner)
}

fn phase(w: &World) -> Option<Phase> {
    w.domination.current.as_ref().map(|d| d.phase)
}

// ---------------------------------------------------------------------------
// Claim 1: the launch.

// spec: S-0612 claim 1 (in the tick it is applied: a radar event at c, a domination with house H and cell c, the lead-in at the centre of c at the terrain surface, a lighting re-evaluation, the activation announcement and the positional activation sound)
#[test]
fn claim_1_launch() {
    let r = rules();
    let mut w = armed(&r);
    let c = at(0, 0);
    w.effects.clear();
    let l = launch(&mut w, c);
    let centre = glasnost_sim::superweapons::cell_point(&w, c, false);
    assert_eq!((centre.x, centre.y), (256 * c.x + 128, 256 * c.y + 128));
    let d = w.domination.current.clone().expect("a domination");
    assert_eq!((d.house, d.cell, d.launched), (ALPHA, c, l));
    assert_eq!(d.phase, Phase::LeadIn { first: false });
    let lead = d.lead.expect("the lead-in");
    assert_eq!(w.domination.anims[&lead].pos, centre);
    assert!(sw_common::anims(&w).contains(&"LEAD".into()));
    let ev: Vec<(SwEvent, Option<Pos>)> = w
        .effects
        .iter()
        .filter_map(|e| match e {
            Effect::Superweapon { event, pos, sw, .. } if *sw == Some(DOM) => Some((*event, *pos)),
            _ => None,
        })
        .collect();
    assert!(ev.contains(&(SwEvent::Radar, Some(centre))), "{ev:?}");
    assert!(ev.contains(&(SwEvent::Launch, None)), "{ev:?}");
    assert!(
        ev.contains(&(SwEvent::DominatorSound, Some(centre))),
        "{ev:?}"
    );
    assert!(dom_events(&w).contains(&DomEvent::Started));
    // S-0104 claim 5: the re-evaluation chose the dominator set.
    assert_eq!(w.lighting.choice, Choice::Dominator);
    assert_eq!(w.lighting.target, w.lighting.map.dominator.ambient);
    assert!(!rec(&w, ALPHA, DOM).ready, "the charge is spent");
}

// ---------------------------------------------------------------------------
// Claim 2: a missing animation type.

// spec: S-0612 claim 2 (either type unset: the charge is spent, the announcement, sound and radar event happen, no domination and no re-evaluation; another launch is accepted; the importer warns)
#[test]
fn claim_2_missing_type() {
    for unset_lead in [true, false] {
        let mut r = rules();
        let s = &mut r.sim.general.superweapons;
        if unset_lead {
            s.dominator_first_anim = None;
        } else {
            s.dominator_second_anim = Some("NOSUCH".into());
        }
        let mut w = armed(&r);
        assert!(
            w.rules()
                .superweapons
                .warnings
                .iter()
                .any(|m| m.starts_with("S-0612 claim 2")),
            "{:?}",
            w.rules().superweapons.warnings
        );
        sw_common::spawn(&mut w, "DOMB", BETA, 6, 6);
        ready_tick(&mut w, BETA, DOM, 400);
        w.effects.clear();
        let before = w.lighting.clone();
        launch(&mut w, at(0, 0));
        assert!(!domination::in_force(&w));
        assert!(!rec(&w, ALPHA, DOM).ready, "the charge is spent");
        let ev = sw_common::events(&w);
        for e in [SwEvent::Radar, SwEvent::Launch, SwEvent::DominatorSound] {
            assert!(ev.contains(&(e, Some(ALPHA), Some(DOM))), "{e:?} {ev:?}");
        }
        assert_eq!(w.lighting.choice, before.choice, "no re-evaluation");
        assert_eq!(w.lighting.target, before.target);
        fire_now(&mut w, BETA, DOM, at(2, 2));
        assert!(!rec(&w, BETA, DOM).ready, "Beta's launch is accepted");
    }
}

// ---------------------------------------------------------------------------
// Claim 3: in force, refusal.

// spec: S-0612 claim 3 and S-0512 claim 8 (while a domination is in force every launch of this kind by any house is refused: charge and ready state kept, a refusal for its own player; accepted again once it is over)
#[test]
fn claim_3_refused_while_in_force() {
    let r = rules();
    let mut w = armed(&r);
    sw_common::spawn(&mut w, "DOMB", BETA, 6, 6);
    ready_tick(&mut w, BETA, DOM, 400);
    launch(&mut w, at(0, 0));
    assert!(domination::in_force(&w));
    w.effects.clear();
    fire_now(&mut w, BETA, DOM, at(3, 3));
    assert!(rec(&w, BETA, DOM).ready, "Beta keeps its charge");
    assert!(
        sw_common::events(&w).contains(&(SwEvent::Refused, Some(BETA), Some(DOM))),
        "{:?}",
        sw_common::events(&w)
    );
    until_event(&mut w, |e| *e == DomEvent::Over, 400);
    assert!(!domination::in_force(&w));
    fire_now(&mut w, BETA, DOM, at(3, 3));
    assert!(!rec(&w, BETA, DOM).ready, "accepted once over");
    assert_eq!(w.domination.current.as_ref().map(|d| d.house), Some(BETA));
}

// ---------------------------------------------------------------------------
// Claim 4 and F1: when the sweep happens.

// spec: S-0612 claim 4 and F1 (the first domination update cannot sweep; the sweep comes at the first later update whose lead-in stage meets F1, before that tick's turn pass: L + 2 at the earliest)
#[test]
fn claim_4_sweep_timing() {
    // p = 0 meets F1 at stage 0: the earliest sweep, L + 2.
    let mut r = rules();
    r.sim.general.superweapons.dominator_fire_at_percentage = Some(0);
    let mut w = armed(&r);
    let l = launch(&mut w, at(0, 0));
    assert_eq!(sweep_tick(&mut w), l + 2);
    // p = 50 with N = 8 needs stage 4: the stage read at tick L + j is j − 2.
    let r = rules();
    let mut w = armed(&r);
    let l = launch(&mut w, at(0, 0));
    assert_eq!(sweep_tick(&mut w), l + 6);
    // p = 75: stage 6.
    let mut r = rules();
    r.sim.general.superweapons.dominator_fire_at_percentage = Some(75);
    let mut w = armed(&r);
    let l = launch(&mut w, at(0, 0));
    assert_eq!(sweep_tick(&mut w), l + 8);
}

// spec: S-0612 F1 (exact equality 100k = pN: no sweep under round-toward-zero for p = 1, 2, 4, 8, 11, 16, 19, 22, 32, 38, 44, 51, 63, 64, 76, 88, 89; under round-to-nearest for p = 35, 41, 47, 57, 69, 70, 82, 83, 94, 95; a sweep for every other p from 0 to 100; elsewhere the integer test)
#[test]
fn f1_exact_equality_cases() {
    let rz = [
        1, 2, 4, 8, 11, 16, 19, 22, 32, 38, 44, 51, 63, 64, 76, 88, 89,
    ];
    let rn = [35, 41, 47, 57, 69, 70, 82, 83, 94, 95];
    for (mode, list) in [
        (Rounding::TowardZero, &rz[..]),
        (Rounding::Nearest, &rn[..]),
    ] {
        for p in 0..=100 {
            // N = 100, k = p: 100k = pN exactly.
            let got = domination::f1(p, p, 100, mode);
            assert_eq!(got, !list.contains(&p), "{mode:?} p = {p}");
            // N = 200, k = 2p: the same equality, for any N.
            assert_eq!(domination::f1(p, 2 * p, 200, mode), got, "{mode:?} p = {p}");
        }
        for (p, k, n) in [(50, 3, 8), (50, 5, 8), (75, 5, 8), (75, 7, 8), (10, 1, 12)] {
            assert_eq!(
                domination::f1(p, k, n, mode),
                100 * k >= p * n,
                "{mode:?} {p} {k} {n}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Claim 5 and F2: final phase and end.

// spec: S-0612 claim 5 and F2 (t₁ the first update after the sweep with k₂ ≥ N₂ − 10; the final phase at the first update after t₁ with k₂ ≥ N₂ − 1, with a re-evaluation; over at a later update at which the current ambient equals the target, with no re-evaluation)
#[test]
fn claim_5_final_phase_and_end() {
    for skip in [true, false] {
        let mut r = rules();
        r.globals.quirks.update_order_removal_skip = skip;
        let mut w = armed(&r);
        launch(&mut w, at(0, 0));
        let s = sweep_tick(&mut w);
        // The sweep animation's stage read at tick S + j is j − 1; N₂ = 12:
        // t₁ at j = 3 (stage 2), the final phase at j = 12 (stage 11). The
        // lead-in (N = 8, swept at L + 6) ends in the turn pass of S + 3;
        // with S-0203 claim 10's skip that removal costs the sweep
        // animation its turn there, one tick later.
        step(&mut w);
        step(&mut w);
        assert_eq!(phase(&w), Some(Phase::Swept { near: false }));
        step(&mut w);
        assert_eq!(phase(&w), Some(Phase::Swept { near: true }), "t₁ = S + 3");
        let f = until_event(&mut w, |e| *e == DomEvent::Final, 100);
        assert_eq!(f, s + SWEEP_N + i32::from(skip), "skip {skip}");
        // S-0104 claim 5: the final phase's re-evaluation chose normal.
        assert_eq!(w.lighting.choice, Choice::Normal);
        assert_eq!(w.lighting.target, w.lighting.base);
        assert!(
            domination::in_force(&w),
            "still in force in its final phase"
        );
        let over = until_event(&mut w, |e| *e == DomEvent::Over, 100);
        assert!(over > f);
        assert_eq!(w.lighting.current, w.lighting.target);
        assert!(!domination::in_force(&w));
    }
}

// spec: S-0612 F2 (the final phase starts at least two domination updates after the sweep, even for a short sweep animation)
#[test]
fn f2_at_least_two_updates() {
    let mut r = rules();
    r.sim.animations.get_mut("SWEEP").unwrap().frame_count = Some(2);
    let mut w = armed(&r);
    launch(&mut w, at(0, 0));
    let s = sweep_tick(&mut w);
    let f = until_event(&mut w, |e| *e == DomEvent::Final, 100);
    assert_eq!(f, s + 2);
}

// ---------------------------------------------------------------------------
// Claim 6: animations that end too early.

// spec: S-0612 claim 6 (a lead-in removed before F1's threshold: the sweep at the next domination update; a sweep animation removed before F2's final threshold: the final phase at the next update; the importer warns about both)
#[test]
fn claim_6_ended_animations() {
    let mut r = rules();
    // p = 100 with N = 8: stage 8 is never read (the lead-in ends at it).
    r.sim.general.superweapons.dominator_fire_at_percentage = Some(100);
    // A shadow type's stages end at N / 2.
    r.sim.animations.get_mut("SWEEP").unwrap().shadow = Some(true);
    let mut w = armed(&r);
    let warn = &w.rules().superweapons.warnings;
    assert!(
        warn.iter()
            .any(|m| m.starts_with("S-0612 claim 6: DominatorFireAtPercentage")),
        "{warn:?}"
    );
    assert!(
        warn.iter()
            .any(|m| m.starts_with("S-0612 claim 6: the sweep animation")),
        "{warn:?}"
    );
    let l = launch(&mut w, at(0, 0));
    // The lead-in's advance to stage 8 in the turn pass of L + 9 ends it;
    // the update of L + 10 finds it removed and sweeps.
    let s = sweep_tick(&mut w);
    assert_eq!(s, l + 10);
    // SWEEP as a shadow type ends at stage 6 (turn pass of S + 6), before
    // stage 11: the final phase at S + 7.
    let f = until_event(&mut w, |e| *e == DomEvent::Final, 100);
    assert_eq!(f, s + 7);
}

// ---------------------------------------------------------------------------
// Claim 7: one domination; the zero-rate freeze.

// spec: S-0612 claim 7 and S-0104 Edge cases (zero ambient change rate: the current ambient never moves; over at the first update after the final phase starts at which the target equals the frozen value, the next update in the plain case; quirks.zero_rate_freezes_ambient off: the ambient jumps to the target)
#[test]
fn claim_7_zero_rate() {
    let mut r = rules();
    r.sim.general.ambient_change_rate = Some(d("0"));
    let mut w = armed(&r);
    let base = w.lighting.current;
    launch(&mut w, at(0, 0));
    let f = until_event(&mut w, |e| *e == DomEvent::Final, 200);
    assert_eq!(w.lighting.current, base, "frozen through the domination");
    let over = until_event(&mut w, |e| *e == DomEvent::Over, 10);
    assert_eq!(over, f + 1, "the plain case: the next domination update");

    // A storm in force when the final phase starts: the target is the ion
    // ambient, never the frozen value, so the domination stays in force.
    let mut r2 = r.clone();
    r2.sim.general.superweapons.lightning_deferment = Some(0);
    r2.sim.general.superweapons.lightning_storm_duration = Some(10_000);
    let mut w = armed(&r2);
    sw_common::spawn(&mut w, "WEATHERB", BETA, 6, 6);
    sw_common::spawn(&mut w, "DOMB", BETA, 7, 7);
    ready_tick(&mut w, BETA, DOM, 400);
    ready_tick(&mut w, BETA, STORM, 400);
    launch(&mut w, at(0, 0));
    fire(&mut w, BETA, STORM, at(4, 4));
    until_event(&mut w, |e| *e == DomEvent::Final, 200);
    sw_common::run(&mut w, 100);
    assert!(domination::in_force(&w), "stays in force");
    assert_ne!(w.lighting.target, w.lighting.current);
    fire_now(&mut w, BETA, DOM, at(3, 3));
    assert!(rec(&w, BETA, DOM).ready, "no house can launch this kind");

    // Switch off: the current ambient jumps straight to each target.
    let mut r3 = r.clone();
    r3.globals.quirks.zero_rate_freezes_ambient = false;
    let mut w = armed(&r3);
    launch(&mut w, at(0, 0));
    step(&mut w);
    assert_eq!(w.lighting.current, w.lighting.map.dominator.ambient);
}

// ---------------------------------------------------------------------------
// Claim 8: the blast ring and the sweep animation.

// spec: S-0612 claim 8 (at the sweep, before the damage: a blast ring at the centre of the target cell, updated in row 15, removed at its 80th update, so present for 79; no draws; the sweep animation starts there)
#[test]
fn claim_8_blast_ring() {
    let r = rules();
    let mut w = armed(&r);
    let c = at(0, 0);
    launch(&mut w, c);
    let s = sweep_tick(&mut w);
    let centre = glasnost_sim::superweapons::cell_point(&w, c, false);
    assert_eq!(w.domination.rings.len(), 1);
    assert_eq!(w.domination.rings[0].pos, centre);
    assert_eq!(
        w.domination.rings[0].updates, 1,
        "updated in its own tick's row 15"
    );
    assert!(sw_common::anims(&w).contains(&"SWEEP".into()));
    let sweep = w.domination.current.as_ref().unwrap().sweep.unwrap();
    assert_eq!(w.domination.anims[&sweep].pos, centre);
    while w.tick - 1 < s + 78 {
        step(&mut w);
    }
    assert_eq!(w.domination.rings.len(), 1, "present after its 79th update");
    step(&mut w);
    assert!(w.domination.rings.is_empty(), "removed at its 80th");
    assert!(dom_events(&w).contains(&DomEvent::RingEnded));
    // No draws: the ring's updates leave the stream alone.
    let mut a = w.clone();
    a.domination.rings.push(domination::BlastRing {
        pos: centre,
        updates: 0,
    });
    let mut b = w.clone();
    for _ in 0..5 {
        step(&mut a);
        step(&mut b);
    }
    assert_eq!(a.rng, b.rng);
}

// ---------------------------------------------------------------------------
// Claims 9 and 10: area damage, then takeovers.

fn damaging() -> Ruleset {
    let mut r = rules();
    let s = &mut r.sim.general.superweapons;
    s.dominator_warhead = Some("PsiW".into());
    s.dominator_damage = Some(100);
    r
}

// spec: S-0612 claim 9 (area damage at the centre of the target cell at its terrain surface, the imported amount and warhead, no source object; the attacking house is the current storm owner, none before any request: quirks.dominator_damage_storm_house; off: the dominating house)
#[test]
fn claim_9_area_damage() {
    for quirk in [true, false] {
        let mut r = damaging();
        r.globals.quirks.dominator_damage_storm_house = quirk;
        let mut w = armed(&r);
        let c = at(0, 0);
        let weak = spawn(&mut w, "WALKER", BETA, 0, 0);
        w.obj_mut(weak).unwrap().health = 50;
        let tough = spawn(&mut w, "BOX", BETA, 1, 0);
        let full = w.obj(tough).unwrap().health;
        launch(&mut w, c);
        sweep_tick(&mut w);
        let centre = glasnost_sim::superweapons::cell_point(&w, c, false);
        let det: Vec<(Pos, i32)> = w
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::Detonation { pos, damage, .. } => Some((*pos, *damage)),
                _ => None,
            })
            .collect();
        assert_eq!(det, vec![(centre, 100)]);
        assert!(
            w.obj(tough).unwrap().health < full,
            "the area damage reached it"
        );
        assert!(w.obj(weak).is_none_or(|o| o.combat.killed), "killed");
        // No storm was ever requested: the storm owner is none.
        let kills = w.houses[usize::from(ALPHA.0)].combat.kills;
        assert_eq!(kills, i32::from(!quirk), "quirk {quirk}");
    }
}

// spec: S-0612 claim 9 (a pending storm's requesting house is the attacking house, sampled before the storm's own update of the tick)
#[test]
fn claim_9_pending_storm_owner() {
    let mut r = damaging();
    r.sim.general.superweapons.lightning_deferment = Some(500);
    let mut w = armed(&r);
    sw_common::spawn(&mut w, "WEATHERB", BETA, 6, 6);
    ready_tick(&mut w, BETA, STORM, 400);
    fire_now(&mut w, BETA, STORM, at(5, 5));
    assert_eq!(w.sw.storm.owner, Some(BETA));
    let weak = spawn(&mut w, "WALKER", BETA, 0, 0);
    w.obj_mut(weak).unwrap().health = 50;
    launch(&mut w, at(0, 0));
    sweep_tick(&mut w);
    assert_eq!(
        w.houses[usize::from(ALPHA.0)].combat.kills,
        0,
        "not Alpha's kill"
    );
}

// spec: S-0612 claim 10 (every effect of the damage precedes the first takeover: a destroyed object is not taken over; a survivor is taken over with its post-damage health)
#[test]
fn claim_10_damage_before_takeover() {
    let r = damaging();
    let mut w = armed(&r);
    let weak = spawn(&mut w, "WALKER", BETA, 0, 0);
    w.obj_mut(weak).unwrap().health = 50;
    let tough = spawn(&mut w, "BOX", BETA, 1, 0);
    launch(&mut w, at(0, 0));
    sweep_tick(&mut w);
    let taken: Vec<ObjectId> = dom_events(&w)
        .into_iter()
        .filter_map(|e| match e {
            DomEvent::Taken(id) => Some(id),
            _ => None,
        })
        .collect();
    assert_eq!(taken, vec![tough]);
    assert!(!mind::permanent(&w, weak));
    // The damage event counts one survivor hurt and one destroyed.
    assert!(dom_events(&w).contains(&DomEvent::Damage {
        damaged: 1,
        killed: 1
    }));
    let o = w.obj(tough).unwrap();
    assert_eq!(o.owner, Some(ALPHA));
    assert!(o.health < 1000, "post-damage health kept");
}

// ---------------------------------------------------------------------------
// Claim 11 and F3: swept cells.

// spec: S-0612 claim 11 and F3 (the T1 cells with ring metric at most min(r, 10) in scan order: 1, 9, 21, 37, 61, 89, 121, 161, 205, 253, 309 cells; rings beyond 10 never; a negative range sweeps nothing and warns; the ground layer only, quirks.dominator_ground_layer_only; off, each cell's deck after its ground)
#[test]
fn claim_11_swept_cells() {
    let counts = [1, 9, 21, 37, 61, 89, 121, 161, 205, 253, 309];
    for (range, want) in (0..=12).map(|r| (r, counts[r.min(10) as usize])) {
        let mut rr = rules();
        rr.sim.general.superweapons.dominator_capture_range = Some(range);
        let w = sw_common::game(&rr);
        let cells = domination::swept(&w, at(0, 0));
        assert_eq!(cells.len(), want, "r = {range}");
        assert!(cells.iter().all(|(_, l)| *l == Layer::Ground));
        assert_eq!(cells[0].0, at(0, 0), "ring 0 first");
    }
    let mut rr = rules();
    rr.sim.general.superweapons.dominator_capture_range = Some(-1);
    let w = armed(&rr);
    assert!(domination::swept(&w, at(0, 0)).is_empty());
    assert!(
        w.rules()
            .superweapons
            .warnings
            .iter()
            .any(|m| m.starts_with("S-0612 claim 11"))
    );
    let mut rr = rules();
    rr.globals.quirks.dominator_ground_layer_only = false;
    let w = sw_common::game(&rr);
    let cells = domination::swept(&w, at(0, 0));
    assert_eq!(cells.len(), 2 * 21);
    assert_eq!(cells[0], (at(0, 0), Layer::Ground));
    assert_eq!(cells[1], (at(0, 0), Layer::Deck));
    // Range 1 by sweep: ring 1 taken, ring 2 not.
    let mut rr = rules();
    rr.sim.general.superweapons.dominator_capture_range = Some(1);
    let mut w = armed(&rr);
    let near = spawn(&mut w, "BOX", BETA, 1, 1);
    let far = spawn(&mut w, "BOX", BETA, 2, 0);
    launch(&mut w, at(0, 0));
    sweep_tick(&mut w);
    assert_eq!(owner(&w, near), Some(ALPHA));
    assert_eq!(owner(&w, far), Some(BETA));
}

// ---------------------------------------------------------------------------
// Claim 12: which objects of a cell are examined.

// spec: S-0612 claim 12 (the start object is the nearest to the cell's top corner by S-0207 F8, the first listed on a tie; examination runs forward from it, so an infantry unit that entered after a nearer one escapes; quirks.dominator_sweep_from_nearest off: every infantry, vehicle and aircraft)
#[test]
fn claim_12_start_object() {
    use glasnost_sim::cells::position;
    for quirk in [true, false] {
        let mut r = rules();
        r.globals.quirks.dominator_sweep_from_nearest = quirk;
        let mut w = armed(&r);
        let c = at(0, 0);
        // TOP_LEFT is nearer the top corner than BOTTOM_RIGHT; the later
        // arrival is listed first.
        let near = put(&mut w, "GRUNT", BETA, c, position::TOP);
        let late = put(&mut w, "GRUNT", BETA, c, position::BOTTOM);
        let list = w.cells.list(c, Layer::Ground).to_vec();
        let ip = |x| list.iter().position(|&y| y == x).unwrap();
        assert!(ip(late) < ip(near), "{list:?}");
        let ex = domination::examined(&w, c, Layer::Ground);
        if quirk {
            assert_eq!(ex, vec![near]);
        } else {
            assert_eq!(ex, vec![late, near]);
        }
        launch(&mut w, c);
        sweep_tick(&mut w);
        assert_eq!(owner(&w, near), Some(ALPHA));
        assert_eq!(owner(&w, late), Some(if quirk { BETA } else { ALPHA }));
    }
}

// spec: S-0612 claim 12 (a building's position is its foundation centre: in a cell of a building with an even side the building is the start object, and the units listed before it are not examined; off, they are)
#[test]
fn claim_12_building_start_object() {
    for quirk in [true, false] {
        let mut r = rules();
        r.globals.quirks.dominator_sweep_from_nearest = quirk;
        let mut w = armed(&r);
        let pad = spawn(&mut w, "PAD", BETA, 0, 0);
        let g = put(
            &mut w,
            "GRUNT",
            BETA,
            at(1, 1),
            glasnost_sim::cells::position::TOP,
        );
        let ex = domination::examined(&w, at(1, 1), Layer::Ground);
        if quirk {
            assert_eq!(ex, vec![pad], "the building is the start object");
        } else {
            assert_eq!(ex, vec![g]);
        }
        launch(&mut w, at(0, 0));
        sweep_tick(&mut w);
        assert_eq!(owner(&w, g), Some(if quirk { BETA } else { ALPHA }));
        assert_eq!(owner(&w, pad), Some(BETA), "buildings are never taken");
    }
}

// ---------------------------------------------------------------------------
// Claim 13: who is taken over.

// spec: S-0612 claim 13 (not taken: a building, psionics immunity, protection, balloon hover, airborne; taken: own, allied and already permanent objects; no owner, alliance or prior-control test)
#[test]
fn claim_13_who_is_taken() {
    let r = rules();
    let mut w = armed(&r);
    sw_common::ally(&mut w, ALPHA, BETA);
    sw_common::ally(&mut w, BETA, ALPHA);
    let stoic = spawn(&mut w, "STOIC", BETA, 0, 0);
    let balloon = spawn(&mut w, "BALLOON", BETA, 1, 0);
    let shielded = spawn(&mut w, "BOX", BETA, -1, 0);
    let t = w.tick;
    w.obj_mut(shielded).unwrap().sw.protection = Some(Protection {
        start: t,
        duration: 10_000,
        kind: ProtectKind::Curtain,
        tint: Tint::default(),
    });
    let own = spawn(&mut w, "BOX", ALPHA, 0, 1);
    let allied = spawn(&mut w, "BOX", BETA, 0, -1);
    let again = spawn(&mut w, "BOX", ALPHA, 1, 1);
    w.mind.permanent.insert(again);
    launch(&mut w, at(0, 0));
    sweep_tick(&mut w);
    for id in [stoic, balloon, shielded] {
        assert_eq!(owner(&w, id), Some(BETA), "{id:?}");
        assert!(!mind::permanent(&w, id), "{id:?}");
    }
    for id in [own, allied, again] {
        assert_eq!(owner(&w, id), Some(ALPHA), "{id:?}");
        assert!(mind::permanent(&w, id), "{id:?}");
    }
}

// spec: S-0612 claim 13 (an object still in play while dying is taken over: quirks.dominator_takes_dying; off, it is skipped)
#[test]
fn claim_13_dying() {
    for quirk in [true, false] {
        let mut r = rules();
        r.globals.quirks.dominator_takes_dying = quirk;
        let mut w = armed(&r);
        let v = spawn(&mut w, "GRUNT", BETA, 0, 0);
        let l = launch(&mut w, at(0, 0));
        // Dying from just before the sweep tick L + 6 (its update in row 3
        // precedes the unit's own turn).
        while w.tick < l + 6 {
            step(&mut w);
        }
        w.obj_mut(v).unwrap().combat.killed = true;
        step(&mut w);
        assert!(dom_events(&w).contains(&DomEvent::Sweep));
        assert!(w.obj(v).is_some(), "dying, in play");
        assert_eq!(owner(&w, v), Some(if quirk { ALPHA } else { BETA }));
    }
}

// spec: S-0612 claim 13 (airborne objects are not taken over: S-0601 claim 13's in-the-air test)
#[test]
fn claim_13_airborne() {
    let r = rules();
    let mut w = armed(&r);
    let p = spawn(&mut w, "SPYP", BETA, 0, 0);
    let o = w.obj_mut(p).unwrap();
    o.pos.z += 2000;
    let airborne = glasnost_sim::combat::weapon::in_air(&w, p);
    launch(&mut w, at(0, 0));
    sweep_tick(&mut w);
    assert!(airborne, "in the air");
    assert!(!mind::dominable(&w, p));
    assert_eq!(owner(&w, p), Some(BETA));
}

// ---------------------------------------------------------------------------
// Claim 14: the takeover.

// spec: S-0612 claim 14 (released by its temporary controller first, then the owner change, then permanent control with one ring; an own object is marked too: quirks.dominator_marks_own_objects; off, left unchanged)
#[test]
fn claim_14_takeover() {
    for quirk in [true, false] {
        let mut r = rules();
        r.globals.quirks.dominator_marks_own_objects = quirk;
        let mut w = armed(&r);
        let own = spawn(&mut w, "BOX", ALPHA, 0, 0);
        let enemy = spawn(&mut w, "BOX", BETA, 1, 0);
        launch(&mut w, at(0, 0));
        sweep_tick(&mut w);
        assert_eq!(owner(&w, own), Some(ALPHA));
        assert_eq!(mind::permanent(&w, own), quirk);
        assert_eq!(owner(&w, enemy), Some(ALPHA));
        assert!(mind::permanent(&w, enemy));
        assert_eq!(
            w.mind.permanent.iter().filter(|&&x| x == enemy).count(),
            1,
            "one ring"
        );
    }
}

// ---------------------------------------------------------------------------
// Claim 15: what the sweep does not reach.

// spec: S-0612 claim 15 (only examined objects are direct recipients: objects outside the swept cells and airborne objects are untouched; a passenger keeps its owner)
#[test]
fn claim_15_not_reached() {
    let r = rules();
    let mut w = armed(&r);
    let outside = spawn(&mut w, "BOX", BETA, 4, 0);
    launch(&mut w, at(0, 0));
    sweep_tick(&mut w);
    assert_eq!(owner(&w, outside), Some(BETA));
    let taken = dom_events(&w)
        .iter()
        .filter(|e| matches!(e, DomEvent::Taken(_)))
        .count();
    assert_eq!(taken, 0);
}

// ---------------------------------------------------------------------------
// Claim 16: orders after a computer house's takeover.

// spec: S-0612 claim 16 (hunt as the next behaviour after all takeovers only for the neutral or a special dominating house; a player or AI house gives none: a departure)
#[test]
fn claim_16_hunt() {
    let r = rules();
    for (house, hunt) in [(HouseId(2), true), (HouseId(3), true), (ALPHA, false)] {
        let mut w = testkit::game_with_non_players(&r, sw_common::SIZE, 1);
        let v = spawn(&mut w, "BOX", BETA, 0, 0);
        domination::launch(&mut w, house, DOM, at(0, 0));
        for _ in 0..20 {
            step(&mut w);
            if owner(&w, v) == Some(house) {
                break;
            }
        }
        assert_eq!(owner(&w, v), Some(house));
        let o = w.obj(v).unwrap();
        let queued =
            o.beh.queued == Some(Behaviour::Hunt) || o.beh.current == Some(Behaviour::Hunt);
        assert_eq!(queued, hunt, "{house:?}");
    }
}

// ---------------------------------------------------------------------------
// Claim 17: the dominating house is fixed at launch.

// spec: S-0612 claim 17 (losing the launching building after the launch changes nothing: the sweep comes on time and its takeovers go to the dominating house)
#[test]
fn claim_17_fixed_house() {
    let r = rules();
    let mut a = armed(&r);
    let v = spawn(&mut a, "BOX", BETA, 0, 0);
    let mut b = a.clone();
    let la = launch(&mut a, at(0, 0));
    let lb = launch(&mut b, at(0, 0));
    let dombs: Vec<ObjectId> = b
        .objects
        .values()
        .filter(|o| o.kind == Kind::Building && o.owner == Some(ALPHA))
        .map(|o| o.id)
        .collect();
    for id in dombs {
        sw_common::remove(&mut b, id);
    }
    assert_eq!(sweep_tick(&mut a) - la, sweep_tick(&mut b) - lb);
    assert_eq!(owner(&b, v), Some(ALPHA));
}

// ---------------------------------------------------------------------------
// Claim 18: other effects.

// spec: S-0612 claim 18 and S-0104 claims 2 and 5 (a domination coexists with a storm: its own claims are unchanged, and the storm outranks it in the global choice)
#[test]
fn claim_18_with_a_storm() {
    let mut r = rules();
    r.sim.general.superweapons.lightning_deferment = Some(0);
    r.sim.general.superweapons.lightning_storm_duration = Some(10_000);
    let mut w = armed(&r);
    sw_common::spawn(&mut w, "WEATHERB", BETA, 6, 6);
    ready_tick(&mut w, BETA, STORM, 400);
    fire_now(&mut w, BETA, STORM, at(8, 8));
    assert_eq!(w.lighting.choice, Choice::Ion);
    let v = spawn(&mut w, "BOX", BETA, 0, 0);
    let l = launch(&mut w, at(0, 0));
    assert_eq!(
        w.lighting.choice,
        Choice::Ion,
        "the storm outranks the dominator"
    );
    assert_eq!(
        sweep_tick(&mut w),
        l + 6,
        "the same timing as without a storm"
    );
    assert_eq!(owner(&w, v), Some(ALPHA));
}

// ---------------------------------------------------------------------------
// Save and load.

// spec: S-0612 Terms (the domination is game state) and S-0203 (the state hash survives a save/load round trip with a domination in force)
#[test]
fn save_load_mid_domination() {
    let r = rules();
    let mut w = armed(&r);
    spawn(&mut w, "BOX", BETA, 0, 0);
    launch(&mut w, at(0, 0));
    let bytes = save::save(&w);
    let mut l = save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("load");
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
    for _ in 0..40 {
        step(&mut w);
        step(&mut l);
    }
    assert_eq!(save::state_hash(&l), save::state_hash(&w));
}

// ---------------------------------------------------------------------------
// S-0104: the lighting state.

// spec: S-0104 Formulas, Ambient fade (A' = max(target, 0) within one step, else one step towards it)
#[test]
fn s0104_ambient_step() {
    assert_eq!(lighting::ambient_step(100, 150, 10), 110);
    assert_eq!(lighting::ambient_step(145, 150, 10), 150);
    assert_eq!(lighting::ambient_step(150, 100, 10), 140);
    assert_eq!(lighting::ambient_step(5, -20, 10), 0);
    assert_eq!(lighting::ambient_step(30, -20, 10), 20);
}

// spec: S-0104 claims 4 and 5 (a launch's re-evaluation forces a fade step on the next tick; then the dominator rate, stored in thousandths and used as ticks: quirks.dominator_rate_thousandths; off, ⌊v⌋ ticks)
#[test]
fn s0104_dominator_fade_rate() {
    for quirk in [true, false] {
        let mut r = rules();
        r.globals.quirks.dominator_rate_thousandths = quirk;
        let mut w = armed(&r);
        // A map value of 3.0: 3000 thousandths, 3 whole ticks.
        w.lighting.map.dominator_rate = 3000;
        w.lighting.map.dominator_rate_whole = 3;
        let start = w.lighting.current;
        launch(&mut w, at(0, 0));
        step(&mut w);
        assert_eq!(w.lighting.current, start + 10, "the forced step");
        let gap = if quirk { 3000 } else { 3 };
        assert_eq!(w.lighting.interval, gap);
        let t0 = w.lighting.last_step;
        for _ in 0..3 {
            step(&mut w);
        }
        assert_eq!(w.lighting.last_step, if quirk { t0 } else { t0 + 3 });
    }
}

// spec: S-0104 claim 5, Nuke flash timing (the detonation chooses the nuke set with a forced step; on tick 31 the fading phase re-evaluates; a step on tick 47 still takes the nuke rate)
#[test]
fn s0104_nuke_flash() {
    let r = rules();
    let mut w = sw_common::game(&r);
    let d = w.tick;
    lighting::nuke_detonated(&mut w);
    assert_eq!(w.lighting.choice, Choice::Nuke);
    assert_eq!(w.lighting.target, lighting::NUKE_SET.ambient);
    assert!(w.lighting.forced);
    while w.tick <= d + 30 {
        step(&mut w);
    }
    assert_eq!(
        w.lighting.choice,
        Choice::Nuke,
        "brightening through tick 30"
    );
    step(&mut w);
    assert_eq!(w.lighting.choice, Choice::Normal, "tick 31 re-evaluates");
}

// spec: S-0104 claim 14 (the set-base-ambient action sets the base, and the target unless an ion storm is in force)
#[test]
fn s0104_set_base() {
    let r = rules();
    let mut w = sw_common::game(&r);
    lighting::set_base(&mut w, 57);
    assert_eq!((w.lighting.base, w.lighting.target), (57, 57));
}

// spec: S-0612 Import mapping and Constants (absent keys: fire-at percentage 50, capture range 2, damage 50, no warhead, no animation types, no sound)
#[test]
fn absent_key_defaults() {
    let w = sw_common::game(&sw_common::ruleset());
    let d = &w.rules().superweapons.domination;
    assert_eq!((d.percent, d.range, d.damage), (50, 2, 50));
    assert!(d.warhead.is_none() && d.lead.is_none() && d.sweep.is_none() && d.sound.is_none());
}
