//! S-0312 r91 claim 16 and S-0107 r55 claim 17a (Q-3239, #252): the health
//! of a killed infantry unit left in play, a falling jumpjet infantry
//! unit's further killing hits, the infantry crash end in a high-bridge
//! cell, the turned-down *air-death-end* request, and the three switches
//! `quirks.falling_infantry_regains_health`,
//! `quirks.crash_end_bridge_height` and
//! `quirks.refused_air_death_end_keeps_unit`.
//!
//! Synthetic types on the jumpjet fixtures of `tests/jump_common`:
//! - JJEND, the fixtures' crashable JJCRASH with an *air-death-end* entry
//!   of 4 frames;
//! - JJNOEND, JJCRASH as it is: no sequence entries, so every request is
//!   turned down (S-0107 claim 6, row 1);
//! - JJBOOM, JJNOEND exploding, with the fixtures' `Rifle` as its death
//!   weapon;
//! - JJSEQ, JJCRASH with *air-death-start*, *air-death-fall* and *cheer*
//!   entries and no *air-death-end* entry;
//! - GRUNT, a walking infantry type with no sequence entries and a slow
//!   arcing gun, LOBBER; BOX, a sturdy vehicle to shoot at;
//! - BALCRASH, a crashable balloon-hover vehicle with `Cannon` as its
//!   death weapon, not exploding;
//! - the warhead KILL1 (infantry death kind 1) and the animation BODYA, the
//!   general corpse list.
//!
//! Every name and number is invented; no game data.

mod jump_common;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    AnimationType, Decimal, ProjectileType, SequenceEntry, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::infantry::Seq;
use glasnost_sim::jump::{self, Phase};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, Target, World};
use jump_common::*;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn entry(count: i32) -> SequenceEntry {
    SequenceEntry {
        start: 0,
        count,
        multiplier: 0,
        direction: None,
    }
}

fn game(f: impl FnOnce(&mut Ruleset)) -> World {
    let mut r = ruleset();
    register(&mut r, "animations", "BODYA");
    r.sim.animations.insert(
        "BODYA".into(),
        AnimationType {
            frame_count: Some(10),
            ..Default::default()
        },
    );
    r.sim.general.dead_bodies = Some(vec!["BODYA".into()]);
    let base = r.sim.infantry["JJCRASH"].clone();
    let mut end = base.clone();
    end.sequences.insert("air_death_end".into(), entry(4));
    let mut boom = base.clone();
    boom.combat.explodes = Some(true);
    boom.combat.death_weapon = Some("Rifle".into());
    let mut seqs = base.clone();
    seqs.sequences.insert("air_death_start".into(), entry(6));
    seqs.sequences.insert("air_death_fall".into(), entry(6));
    seqs.sequences.insert("cheer".into(), entry(40));
    for (n, t) in [
        ("JJEND", end),
        ("JJNOEND", base),
        ("JJBOOM", boom),
        ("JJSEQ", seqs),
    ] {
        register(&mut r, "infantry", n);
        r.sim.infantry.insert(n.into(), t);
    }
    register(&mut r, "infantry", "GRUNT");
    r.sim.infantry.insert(
        "GRUNT".into(),
        UnitType {
            strength: Some(100),
            speed: Some(4),
            locomotion_class: Some(glasnost_rules::sim::LocomotionClass::Foot),
            locomotor: Some(glasnost_rules::sim::Locomotor::Walk),
            primary: Some("LOBBER".into()),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "LOBBER".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(30),
            range: Some(d("6")),
            speed: Some(4),
            projectile: Some("SLOWLOB".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    r.sim.projectiles.insert(
        "SLOWLOB".into(),
        ProjectileType {
            arcing: Some(true),
            ..Default::default()
        },
    );
    register(&mut r, "vehicles", "BOX");
    r.sim.vehicles.insert(
        "BOX".into(),
        UnitType {
            strength: Some(30000),
            locomotion_class: Some(glasnost_rules::sim::LocomotionClass::Wheel),
            locomotor: Some(glasnost_rules::sim::Locomotor::Drive),
            ..Default::default()
        },
    );
    let mut bal = r.sim.vehicles["JV"].clone();
    bal.balloon_hover = Some(true);
    bal.combat.explodes = Some(false);
    bal.combat.death_weapon = Some("Cannon".into());
    register(&mut r, "vehicles", "BALCRASH");
    r.sim.vehicles.insert("BALCRASH".into(), bal);
    let ones = Some((0..11).map(|_| Decimal("1".into())).collect());
    r.sim.warheads.insert(
        "KILL1".into(),
        WarheadType {
            verses: ones,
            inf_death: Some(1),
            ..Default::default()
        },
    );
    f(&mut r);
    let mut w = testkit::game_with(&r, SIZE, 7);
    testkit::run(&mut w, 4);
    w
}

/// A unit of type `name` hovering `h` leptons above the ground at (0, 0).
fn hover(w: &mut World, name: &str, kind: Kind, h: i32) -> ObjectId {
    let u = put(w, name, kind, 0, 0, 0);
    jump::start_in_flight(w, u, Phase::Hovering, h);
    u
}

/// The cell at (0, 0) carries a high bridge.
fn deck(w: &mut World) {
    let i = w.cells.idx(at(0, 0)).expect("cell");
    w.cells.deck[i] = true;
    w.cells.deck_entry[i] = true;
    w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::X;
}

/// A killing hit with no warhead, ignoring defences, by `attacker`.
fn kill_by(w: &mut World, id: ObjectId, attacker: Option<ObjectId>) {
    let health = w.obj(id).unwrap().health.max(1);
    damage::apply(
        w,
        id,
        Hit {
            amount: health * 1000,
            warhead: None,
            distance: 0,
            attacker,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
}

fn kill(w: &mut World, id: ObjectId) {
    kill_by(w, id, None);
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).unwrap().health
}

fn seq(w: &World, id: ObjectId) -> Option<Seq> {
    w.obj(id).unwrap().infantry.as_ref().unwrap().seq
}

fn detonations(w: &World) -> Vec<Pos> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Detonation { pos, .. } => Some(*pos),
            _ => None,
        })
        .collect()
}

fn corpses(w: &World) -> usize {
    w.effects
        .iter()
        .filter(|e| matches!(e, Effect::Anim { name, .. } if name == "BODYA"))
        .count()
}

/// Runs until the unit is in *crashed* or has left play, at most 600 ticks;
/// whether it is in *crashed*.
fn to_crash_end(w: &mut World, u: ObjectId) -> bool {
    for _ in 0..600 {
        if w.obj(u).is_none() {
            return false;
        }
        if jump::phase(w, u) == Some(Phase::Crashed) {
            return true;
        }
        step(w);
    }
    panic!("the crash did not end");
}

/// Its height above the terrain under it.
fn above_terrain(w: &World, u: ObjectId) -> i32 {
    let p = pos(w, u);
    p.z - w.surface_at(p.x, p.y)
}

fn on_deck(w: &World, u: ObjectId) -> bool {
    w.obj(u).unwrap().jump.as_ref().unwrap().on_deck
}

// spec: S-0107 claim 17a (A crashing jumpjet infantry unit: killed in another object's turn, it has a health of 1 from its next update on, through its fall; it stays killed) and S-0312 claim 16 (Health while it falls)
#[test]
fn q3239_a_falling_unit_killed_in_another_turn_regains_a_health_of_1() {
    let mut w = game(|_| {});
    let u = hover(&mut w, "JJNOEND", Kind::Infantry, 500);
    kill(&mut w, u);
    assert!(jump::crashing(&w, u), "it starts to crash");
    assert_eq!(health(&w, u), 0, "the killing hit leaves 0");
    step(&mut w);
    assert_eq!(phase(&w, u), Phase::Crashing);
    assert_eq!(health(&w, u), 1, "a health of 1 from its next update");
    assert!(w.obj(u).unwrap().combat.killed, "it stays killed");
    while phase(&w, u) == Phase::Crashing {
        assert_eq!(health(&w, u), 1, "through its fall");
        step(&mut w);
    }
}

// spec: S-0107 Edge cases (quirks.falling_infantry_regains_health off: claim 17a does not apply to a jumpjet infantry unit from its killing hit through crashing, up to its crash end; from its crash end on it applies as usual)
#[test]
fn q3239_falling_infantry_regains_health_off_keeps_0_until_the_crash_end() {
    let mut w = game(|r| r.globals.quirks.falling_infantry_regains_health = false);
    let u = hover(&mut w, "JJNOEND", Kind::Infantry, 500);
    kill(&mut w, u);
    step(&mut w);
    while phase(&w, u) == Phase::Crashing {
        assert_eq!(health(&w, u), 0, "no health while it falls");
        step(&mut w);
    }
    assert_eq!(phase(&w, u), Phase::Crashed);
    step(&mut w);
    assert!(
        w.obj(u).is_some(),
        "left in play by the turned-down request"
    );
    assert_eq!(health(&w, u), 1, "from its crash end on, claim 17a applies");
}

// spec: S-0312 claim 16 (Health while it falls: the request for air-death-start is not forced, so S-0107 claim 6 can turn it down; the unit then keeps its current sequence) and S-0502 claim 18
#[test]
fn q3239_air_death_start_is_not_forced() {
    for busy in [true, false] {
        let mut w = game(|_| {});
        let u = hover(&mut w, "JJSEQ", Kind::Infantry, 500);
        if busy {
            // Cheer is not interruptible (S-0107 table T1).
            assert!(glasnost_sim::infantry::request(&mut w, u, Seq::Cheer, true));
        }
        kill(&mut w, u);
        step(&mut w);
        assert_eq!(phase(&w, u), Phase::Crashing);
        let want = if busy {
            Some(Seq::Cheer)
        } else {
            Some(Seq::AirDeathStart)
        };
        assert_eq!(seq(&w, u), want, "busy: {busy}");
    }
}

// design: S-0107 claim 17a (a killed unit with no health that plays air-death-start takes its follow-up to air-death-fall, S-0107 claims 8 and 21, so that the general follow-up can end air-death-fall) with quirks.falling_infantry_regains_health off, which keeps it at 0 through the fall
#[test]
fn q3239_a_faller_without_health_goes_on_to_air_death_fall() {
    let mut w = game(|r| r.globals.quirks.falling_infantry_regains_health = false);
    let u = hover(&mut w, "JJSEQ", Kind::Infantry, 500);
    kill(&mut w, u);
    step(&mut w);
    assert_eq!(seq(&w, u), Some(Seq::AirDeathStart));
    let mut fell = false;
    while phase(&w, u) == Phase::Crashing {
        assert_eq!(health(&w, u), 0);
        fell |= seq(&w, u) == Some(Seq::AirDeathFall);
        step(&mut w);
    }
    assert!(
        fell,
        "air-death-start completed and air-death-fall followed"
    );
}

// spec: S-0312 claim 16 (Infantry crash end: in a high-bridge cell it is put at the deck's height above the terrain with its on-deck state unset, listed on the ground layer, whether its fall ended by crossing the deck or beneath it) and Edge cases (quirks.crash_end_bridge_height off: at height 0 above the surface the fall ended over: on the deck with the on-deck state set after crossing it, on the terrain beneath it)
#[test]
fn q3239_the_infantry_crash_end_in_a_high_bridge_cell() {
    for bridge_height in [true, false] {
        for crossing in [true, false] {
            let mut w = game(|r| r.globals.quirks.crash_end_bridge_height = bridge_height);
            deck(&mut w);
            let u = hover(
                &mut w,
                "JJEND",
                Kind::Infantry,
                if crossing { 500 } else { 300 },
            );
            kill(&mut w, u);
            assert!(to_crash_end(&mut w, u), "in play in crashed");
            assert_eq!(pos(&w, u).cell(), at(0, 0), "it fell straight down");
            let got = (
                above_terrain(&w, u),
                on_deck(&w, u),
                w.obj(u).unwrap().layer,
                height(&w, u),
            );
            let want = match (bridge_height, crossing) {
                (true, _) => (416, false, Layer::Ground, 416),
                (false, true) => (416, true, Layer::Deck, 0),
                (false, false) => (0, false, Layer::Ground, 0),
            };
            assert_eq!(
                got, want,
                "bridge height {bridge_height}, crossing {crossing}"
            );
            assert_eq!(seq(&w, u), Some(Seq::AirDeathEnd));
        }
    }
}

// spec: S-0312 claim 16 (Infantry crash end in a high-bridge cell: the on-deck state unset, so listed on the ground layer, S-0502 claim 4; A turned-down air-death-end request: it stays in play) and Edge cases (quirks.crash_end_bridge_height off: listed on the deck after crossing it, on the ground beneath it)
#[test]
fn q3239_the_infantry_crash_end_in_a_high_bridge_cell_lists_it_on_its_layer() {
    for bridge_height in [true, false] {
        for crossing in [true, false] {
            let mut w = game(|r| r.globals.quirks.crash_end_bridge_height = bridge_height);
            deck(&mut w);
            let u = hover(
                &mut w,
                "JJNOEND",
                Kind::Infantry,
                if crossing { 500 } else { 300 },
            );
            kill(&mut w, u);
            assert!(to_crash_end(&mut w, u), "in play in crashed");
            testkit::run(&mut w, 30);
            let cell = pos(&w, u).cell();
            let (want, other) = if !bridge_height && crossing {
                (Layer::Deck, Layer::Ground)
            } else {
                (Layer::Ground, Layer::Deck)
            };
            let case = format!("bridge height {bridge_height}, crossing {crossing}");
            assert_eq!(w.obj(u).unwrap().layer, want, "{case}");
            assert!(w.cells.list(cell, want).contains(&u), "listed: {case}");
            assert!(!w.cells.list(cell, other).contains(&u), "not listed: {case}");
        }
    }
}

// spec: S-0312 claim 16 (Vehicle crash end: in a high-bridge cell it is put on the terrain beneath the deck, whether its fall ended by crossing the deck or beneath it, so its impact effects come there) and Edge cases (quirks.crash_end_bridge_height off: on the deck when its fall ended by crossing it, and a balloon-hover type's death weapon comes there)
#[test]
fn q3239_the_vehicle_crash_end_in_a_high_bridge_cell() {
    for bridge_height in [true, false] {
        for crossing in [true, false] {
            let mut w = game(|r| r.globals.quirks.crash_end_bridge_height = bridge_height);
            deck(&mut w);
            let u = hover(
                &mut w,
                "BALCRASH",
                Kind::Vehicle,
                if crossing { 500 } else { 300 },
            );
            let ground = {
                let p = pos(&w, u);
                w.surface_at(p.x, p.y)
            };
            w.effects.clear();
            kill(&mut w, u);
            assert!(detonations(&w).is_empty(), "it does not explode at death");
            assert!(!to_crash_end(&mut w, u), "it leaves play at its crash end");
            let d = detonations(&w);
            assert_eq!(d.len(), 1, "its death weapon at the crash end");
            let want = if !bridge_height && crossing { 416 } else { 0 };
            assert_eq!(
                d[0].z - ground,
                want,
                "bridge height {bridge_height}, crossing {crossing}"
            );
        }
    }
}

/// A JJNOEND unit shot down over open ground and left in play at its crash
/// end, after one more update.
fn left_in_play(f: impl FnOnce(&mut Ruleset)) -> (World, ObjectId) {
    let mut w = game(f);
    let u = hover(&mut w, "JJNOEND", Kind::Infantry, 500);
    kill(&mut w, u);
    w.effects.clear();
    assert!(to_crash_end(&mut w, u), "in play in crashed");
    step(&mut w);
    (w, u)
}

// spec: S-0312 claim 16 (A turned-down air-death-end request: nothing removes the unit at its crash end; it stays in play for good in crashed, with a health of 1 and no corpse, braking to rest, and no order moves it) and S-0107 claim 17a (a crashed unit left in play keeps a health of 1); quirks.refused_air_death_end_keeps_unit on
#[test]
fn q3239_a_turned_down_air_death_end_keeps_the_unit_for_good() {
    let (mut w, u) = left_in_play(|_| {});
    assert_eq!(health(&w, u), 1);
    assert!(w.obj(u).unwrap().combat.killed);
    testkit::run(&mut w, 150);
    let rest = pos(&w, u);
    // An order: accepted, since it has health, and it moves nothing.
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    let issued = orders::issue(
        &mut w,
        HouseId(0),
        Order::Behaviour {
            actor: u,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at(8, 0))),
        },
    );
    assert!(issued, "a living unit takes orders");
    testkit::run(&mut w, 150);
    assert!(w.obj(u).is_some(), "still in play");
    assert_eq!(phase(&w, u), Phase::Crashed);
    assert_eq!(health(&w, u), 1);
    assert_eq!((pos(&w, u).x, pos(&w, u).y), (rest.x, rest.y), "at rest");
    assert_eq!(height(&w, u), 0);
    assert_eq!(corpses(&w), 0);
}

// spec: S-0312 claim 16 (r91, A turned-down air-death-end request: it stays in play) and S-0107 claim 17a (a living unit for every rule: it can be targeted, so it is listed in its cell, on the ground layer since its on-deck state is unset, S-0502 claim 4)
#[test]
fn q3239_a_unit_kept_for_good_is_listed_in_its_cell() {
    let (mut w, u) = left_in_play(|_| {});
    testkit::run(&mut w, 30);
    let cell = pos(&w, u).cell();
    assert!(
        w.cells.list(cell, Layer::Ground).contains(&u),
        "listed on the ground layer"
    );
    assert!(!w.cells.list(cell, Layer::Deck).contains(&u));
}

// spec: S-0312 claim 16 (A turned-down air-death-end request: a later killing hit removes it at once, with no corpse, when its height above the surface is below 1 lepton at the hit, since a crash starts only from a positive height above the surface)
#[test]
fn q3239_a_later_killing_hit_on_the_ground_removes_it_at_once() {
    let (mut w, u) = left_in_play(|_| {});
    assert_eq!(height(&w, u), 0);
    kill(&mut w, u);
    assert!(w.obj(u).is_none(), "removed at once");
    assert_eq!(corpses(&w), 0);
}

// spec: S-0312 claim 16 (A turned-down air-death-end request: at a high-bridge deck's height with the on-deck state unset, a killing hit starts a crash which never runs, since the unit stays in crashed; its health comes back to 1 at its next update) and Edge cases (quirks.crash_end_bridge_height off: put on the deck at height 0 above the surface, so a later killing hit removes it)
#[test]
fn q3239_a_killing_hit_at_the_decks_height() {
    for bridge_height in [true, false] {
        let mut w = game(|r| r.globals.quirks.crash_end_bridge_height = bridge_height);
        deck(&mut w);
        let u = hover(&mut w, "JJNOEND", Kind::Infantry, 500);
        kill(&mut w, u);
        assert!(to_crash_end(&mut w, u), "in play in crashed");
        step(&mut w);
        assert_eq!(health(&w, u), 1);
        kill(&mut w, u);
        if bridge_height {
            assert!(w.obj(u).is_some(), "a crash that never runs");
            step(&mut w);
            assert_eq!(phase(&w, u), Phase::Crashed);
            assert_eq!(health(&w, u), 1, "back to 1 at its next update");
            assert_eq!(above_terrain(&w, u), 416);
        } else {
            assert!(w.obj(u).is_none(), "on the deck: removed at once");
        }
    }
}

// spec: S-0312 claim 16 (Health while it falls: a further killing hit while it falls is a killing hit like the first, with every death effect again, an exploding unit's death weapon included; its crash start takes the crash-start effects again but the unit stays in crashing: its fall is not started again)
#[test]
fn q3239_a_further_killing_hit_while_it_falls() {
    let mut w = game(|_| {});
    let u = hover(&mut w, "JJBOOM", Kind::Infantry, 500);
    w.effects.clear();
    kill(&mut w, u);
    assert_eq!(
        detonations(&w).len(),
        1,
        "its death weapon at the first kill"
    );
    step(&mut w);
    step(&mut w);
    assert_eq!(phase(&w, u), Phase::Crashing);
    assert_eq!(health(&w, u), 1);
    let z = pos(&w, u).z;
    w.effects.clear();
    kill(&mut w, u);
    assert_eq!(detonations(&w).len(), 1, "its death weapon again");
    assert!(w.obj(u).is_some(), "it stays in play");
    assert_eq!(
        phase(&w, u),
        Phase::Crashing,
        "its fall is not started again"
    );
    assert!(
        !w.obj(u).unwrap().jump.as_ref().unwrap().crash_started,
        "no pending crash start"
    );
    step(&mut w);
    assert!(pos(&w, u).z < z, "it goes on falling");
    assert_eq!(health(&w, u), 1, "and regains a health of 1 again");
}

// spec: S-0107 claim 17a (A turned-down death request: quirks.refused_death_keeps_unit leaves the unit in play; it gets a health of 1 at its first update after the killing hit and is a living unit for every rule: it can be ordered and fires; a projectile it fired before the hit carries no attacker, S-0302 claim 1, while a shot it fires after it has regained health carries it and its house) and S-0312 claim 16 (Killed and alive again)
#[test]
fn q3239_a_refused_death_leaves_a_living_unit_whose_new_shots_carry_it() {
    let mut w = game(|_| {});
    let g = put(&mut w, "GRUNT", Kind::Infantry, 0, 0, 0);
    put(&mut w, "BOX", Kind::Vehicle, 1, 4, 0);
    let fired = |w: &World| w.projectiles.values().any(|p| p.attacker == Some(g));
    assert!(until(&mut w, 600, fired), "it fires before its death");
    // Let the shot leave the unit's cell first: once the firer is killed the shot has no
    // firer and can go off where it is (S-0306 T1), and the revived unit has a health of 1.
    let home = pos(&w, g).cell();
    let away = |w: &World| {
        w.projectiles
            .values()
            .filter(|p| p.attacker == Some(g))
            .all(|p| p.pos.cell() != home)
    };
    assert!(until(&mut w, 600, away), "the shot is past the unit's cell");
    assert!(fired(&w), "a shot of it is in flight at the kill");
    let kill1 = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "KILL1")
        .expect("the KILL1 warhead") as u32;
    let before: Vec<ObjectId> = w.projectiles.keys().copied().collect();
    damage::apply(
        &mut w,
        g,
        Hit {
            amount: 100_000,
            warhead: Some(kill1),
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
    assert!(
        w.obj(g).is_some(),
        "the turned-down die-1 leaves it in play"
    );
    assert_eq!(health(&w, g), 0);
    for p in &before {
        if let Some(p) = w.projectiles.get(p) {
            assert_eq!((p.attacker, p.house), (None, None), "fired before the kill");
        }
    }
    step(&mut w);
    assert_eq!(health(&w, g), 1, "a health of 1 at its next update");
    assert!(w.obj(g).unwrap().combat.killed, "it stays killed");
    let new = |w: &World| {
        w.projectiles
            .iter()
            .any(|(id, p)| !before.contains(id) && p.attacker == Some(g))
    };
    assert!(until(&mut w, 600, new), "it fires again");
    let (_, p) = w
        .projectiles
        .iter()
        .find(|(id, p)| !before.contains(id) && p.attacker == Some(g))
        .unwrap();
    assert_eq!(p.house, Some(HouseId(0)), "with its house");
}
