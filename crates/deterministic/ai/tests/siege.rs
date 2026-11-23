//! The siege script (#449, registry id 2) on the synthetic skirmish: it is
//! listed and chosen like the default, plays at every level and survives a
//! save and a load; its army waits at home below its count, goes out as one
//! attack-moving strike that leaves a guard, and is called home when it is
//! outmatched; its levels are ordered. The shroud test of `tests/scripts.rs`
//! covers it with every registry entry.

use std::collections::BTreeSet;

use glasnost_ai::scripts::{self, AiScript, REGISTRY, Schedule, Seat, View};
use glasnost_ai::siege::{self, HELD, Siege, plan_for, tuning};
use glasnost_ai::{fixture, game::Game};
use glasnost_map::CellXY;
use glasnost_sim::econ::shroud;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, FactoryKind, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{AiDifficulty, AiSeat, HouseId, ObjectId, Pos, Target, World};

fn game(seat: AiSeat) -> Game {
    let mut s = fixture::setup(5);
    s.ai = vec![seat, AiSeat::DEFAULT];
    Game::new(
        fixture::rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        s,
    )
    .expect("the synthetic skirmish loads")
}

fn siege_at(level: AiDifficulty) -> AiSeat {
    AiSeat {
        script: siege::ID,
        difficulty: level,
    }
}

fn yard(w: &World, h: u8) -> Option<ObjectId> {
    let i = w.rules().index_of(Kind::Building, "YARD")?;
    w.objects
        .values()
        .find(|o| {
            o.owner == Some(HouseId(h)) && o.on_map && o.kind == Kind::Building && o.type_index == i
        })
        .map(|o| o.id)
}

/// The skirmish once house 0's yard stands, with no money left to either
/// house (so no army is built but the test's) and no AI thinking after.
fn base_up() -> World {
    let mut g = game(AiSeat::DEFAULT);
    // nocite: test data of our own (the skirmish's yards stand well before)
    while g.world.tick < 3000 && (yard(&g.world, 0).is_none() || yard(&g.world, 1).is_none()) {
        g.step();
    }
    let mut w = g.world.clone();
    assert!(yard(&w, 0).is_some(), "house 0's yard stands");
    for h in &mut w.houses {
        h.credits = 0;
    }
    w
}

/// `n` gun vehicles of house 0 on free cells south of its yard.
fn tanks(w: &mut World, n: usize) -> Vec<ObjectId> {
    let at = w.obj(yard(w, 0).unwrap()).unwrap().pos.cell();
    let mut out = Vec::new();
    // nocite: test data of our own (cells a few south of the yard)
    for (i, dx) in (-6..=6).enumerate() {
        if out.len() == n {
            break;
        }
        let c = CellXY::new(at.x + dx, at.y + 5 + (i as i32 % 2));
        if w.cells.usable(c) && w.objects.values().all(|o| o.pos.cell() != c) {
            out.push(testkit::spawn(w, Kind::Vehicle, "GUNR", 0, c, 0));
        }
    }
    assert_eq!(out.len(), n, "room for the tanks");
    out
}

/// Runs `w` for at most `ticks` steps, house 0 played by `s` at `level`,
/// and stops once the game is over (after its final tick the tick number
/// no longer grows, S-0201 claim 1 and S-0414 claim 8); returns every order
/// it gave, by tick.
fn play(w: &mut World, s: &mut Siege, level: AiDifficulty, ticks: i32) -> Vec<(i32, Order)> {
    let mut got = Vec::new();
    for _ in 0..ticks {
        if w.econ.end.finished {
            break;
        }
        let tick = w.tick;
        let schedule = Schedule::of(HouseId(0), tick);
        let view = View::new(w, HouseId(0));
        let o = AiScript::think(s, &view, schedule, level, None, &mut ());
        for o in o {
            got.push((tick, o.clone()));
            orders::issue(w, HouseId(0), o);
        }
        testkit::run(w, 1);
    }
    got
}

/// The units of `ids` that stand idle (no behaviour, or guarding where
/// they stand or their area): those the brain has not sent on a part of
/// its own.
fn waiting(w: &World, ids: &[ObjectId]) -> usize {
    ids.iter()
        .filter(|&&id| {
            w.obj(id).is_some_and(|o| {
                matches!(
                    o.beh.current,
                    None | Some(Behaviour::Guard | Behaviour::AreaGuard)
                )
            })
        })
        .count()
}

/// The units of `ids` given an attack-move order in `got`.
fn attack_moved(got: &[(i32, Order)], ids: &[ObjectId]) -> BTreeSet<ObjectId> {
    got.iter()
        .filter_map(|(_, o)| match o {
            Order::Behaviour {
                actor,
                behaviour: Behaviour::Move,
                attack_move: true,
                ..
            } if ids.contains(actor) => Some(*actor),
            _ => None,
        })
        .collect()
}

// design: own AI (#449): the registry lists the siege script after the default, under the next id, with a key for the tools and a name for the setup screen
#[test]
fn the_registry_lists_the_siege_script() {
    assert_eq!(REGISTRY[0].id, AiSeat::DEFAULT_SCRIPT);
    let e = scripts::by_key("siege").expect("listed");
    assert_eq!(e.id, siege::ID);
    assert_eq!(siege::ID, AiSeat::DEFAULT_SCRIPT + 1);
    assert_eq!(e.name, "Siege");
    assert_eq!(scripts::entry(siege::ID), Some(e));
    assert_eq!(
        scripts::known(siege_at(AiDifficulty::Hard)).script,
        siege::ID
    );
}

// design: own AI (#449): a setup that chooses the siege script plays it, at each level, and the choice and the script's state survive a save and a load with the hash
#[test]
fn a_setup_choosing_siege_plays_it_at_every_level() {
    for level in AiDifficulty::ALL {
        let seat = siege_at(level);
        let mut g = game(seat);
        let choices: Vec<AiSeat> = g.seats().iter().map(Seat::choice).collect();
        assert_eq!(choices, vec![seat, AiSeat::DEFAULT], "{level:?}");
        assert!(g.seats()[0].script().siege().is_some(), "{level:?}");
        assert!(g.seats()[1].script().brain().is_some(), "{level:?}");
        // nocite: test data of our own (a few think periods)
        for _ in 0..600 {
            g.step();
        }
        let mut back =
            Game::load(&g.save(), fixture::rules(), fixture::catalogue()).expect("loads");
        assert!(back.seats()[0].script().siege().is_some());
        assert_eq!(back.seats()[0].choice(), seat);
        assert_eq!(back.state_hash(), g.state_hash(), "{level:?}");
        // nocite: test data of our own
        for _ in 0..300 {
            g.step();
            back.step();
        }
        assert_eq!(back.state_hash(), g.state_hash(), "{level:?}");
    }
}

// design: own AI (#449): the siege's plan holds the brain's waves at every level, so that the army is the siege's own, and keeps the level's harvesters
#[test]
fn the_brains_waves_are_held_at_every_level() {
    for level in AiDifficulty::ALL {
        let p = plan_for(level);
        assert_eq!(p.wave, HELD, "{level:?}");
        assert_eq!(p.harvesters, tuning(level).harvesters, "{level:?}");
    }
}

// design: own AI (#449): the levels are ordered: easy earns from fewer harvesters, strikes with fewer and acts at a third of the rate; normal and hard put no money into superweapons; only hard weighs the enemy army it remembers, calls a strike home, goes for the enemy economy and hunts
#[test]
fn the_levels_play_worse_to_better() {
    let (easy, normal, hard) = (
        tuning(AiDifficulty::Easy),
        tuning(AiDifficulty::Normal),
        tuning(AiDifficulty::Hard),
    );
    assert!(easy.harvesters < normal.harvesters);
    assert!(easy.every > normal.every && normal.every == hard.every);
    assert!(easy.strike < normal.strike);
    assert!(!normal.superweapons && !hard.superweapons);
    for t in [easy, normal] {
        assert!(!t.hunts && t.edge == 0 && t.recall == 0 && !t.economy && t.patience == 0);
    }
    assert!(hard.hunts && hard.edge > 100 && hard.recall > 0 && hard.economy);
    assert!(hard.join < normal.join);
}

// design: own AI (#449): the army waits at home while fewer than the level's strike count stand idle there; at the count it goes out as one strike of attack-move orders, and the guard nearest home stays
#[test]
fn the_strike_goes_at_its_count_and_leaves_the_guard() {
    let level = AiDifficulty::Hard;
    let t = tuning(level);
    let mut w = base_up();
    let mut s = Siege::start(&w, &fixture::map(), HouseId(0), level);
    let first = tanks(&mut w, t.strike - 1);
    // nocite: test data of our own (several think periods)
    let got = play(&mut w, &mut s, level, 600);
    assert!(
        attack_moved(&got, &first).is_empty(),
        "no strike below the count"
    );
    assert_eq!(s.strikes(), 0);
    assert!(waiting(&w, &first) < t.strike);
    // The brain keeps some for its own parts (a scout, a guard of the
    // base): three more make the count with room for them.
    let more = tanks(&mut w, 3);
    let all: Vec<ObjectId> = first.iter().chain(&more).copied().collect();
    let before = waiting(&w, &all);
    assert!(before >= t.strike);
    // nocite: test data of our own
    let got = play(&mut w, &mut s, level, 600);
    let sent = attack_moved(&got, &all);
    assert_eq!(s.strikes(), 1);
    assert_eq!(
        sent.len(),
        before - t.guard,
        "all but the guard go: {sent:?}"
    );
    assert_eq!(s.out(), &sent);
    // Every one goes towards the enemy start, the only goal it knows.
    let enemy = fixture::at(fixture::START_DX, 0);
    for (_, o) in &got {
        if let Order::Behaviour {
            actor,
            attack_move: true,
            dest: Some(Target::Cell(c)),
            ..
        } = o
            && sent.contains(actor)
        {
            assert!(
                (c.x - enemy.x).abs() <= 12,
                "{c:?} by the enemy start {enemy:?}"
            );
        }
    }
}

/// `n` riflemen of house 0 on free cells north of its yard.
fn soldiers(w: &mut World, n: usize) -> Vec<ObjectId> {
    let at = w.obj(yard(w, 0).unwrap()).unwrap().pos.cell();
    let mut out = Vec::new();
    // nocite: test data of our own (cells a few north of the yard)
    for dy in [-5, -6, -7] {
        for dx in -6..=6 {
            let c = CellXY::new(at.x + dx, at.y + dy);
            if out.len() < n && w.cells.usable(c) && w.objects.values().all(|o| o.pos.cell() != c) {
                out.push(testkit::spawn(w, Kind::Infantry, "WALKER", 0, c, 0));
            }
        }
    }
    assert_eq!(out.len(), n, "room for the soldiers");
    out
}

// design: own AI (#449): at hard a new strike also needs its units worth the level's floor: past the count, with no enemy army remembered, cheap soldiers wait at home, and go once tanks join them
#[test]
fn a_strike_below_the_floor_waits_at_hard() {
    let level = AiDifficulty::Hard;
    let t = tuning(level);
    assert!(t.floor > 0);
    assert_eq!(tuning(AiDifficulty::Easy).floor, 0);
    assert_eq!(tuning(AiDifficulty::Normal).floor, 0);
    let mut w = base_up();
    let mut s = Siege::start(&w, &fixture::map(), HouseId(0), level);
    // The count and room for the brain's own parts, worth less than the floor.
    let few = soldiers(&mut w, t.strike + 3);
    let worth: i64 = few.iter().map(|&id| siege::worth(&w, id)).sum();
    assert!(worth < t.floor, "{worth} under the floor");
    // nocite: test data of our own (several think periods)
    let got = play(&mut w, &mut s, level, 600);
    assert!(waiting(&w, &few) >= t.strike, "the count waits");
    assert!(
        attack_moved(&got, &few).is_empty(),
        "no strike below the floor"
    );
    assert_eq!(s.strikes(), 0);
    // Tanks join them: the units that would go pass the floor.
    let more = tanks(&mut w, 8);
    let all: Vec<ObjectId> = few.iter().chain(&more).copied().collect();
    // nocite: test data of our own
    let got = play(&mut w, &mut s, level, 600);
    assert_eq!(s.strikes(), 1);
    assert!(!attack_moved(&got, &all).is_empty(), "the strike goes");
}

// design: own AI (#449): at hard a strike worth less than the level's share of the enemy army the house has seen is called home with plain move orders, and the army gathers again
#[test]
fn an_outmatched_strike_is_called_home() {
    let level = AiDifficulty::Hard;
    let t = tuning(level);
    let mut w = base_up();
    let mut s = Siege::start(&w, &fixture::map(), HouseId(0), level);
    // The strike's count and room for the brain's own parts.
    tanks(&mut w, t.strike + 3);
    // nocite: test data of our own
    play(&mut w, &mut s, level, 600);
    assert_eq!(s.strikes(), 1);
    let out: Vec<ObjectId> = s.out().iter().copied().collect();
    assert!(!out.is_empty());
    // A large enemy army, seen, ahead of the strike.
    let lead = w.obj(out[0]).unwrap().pos.cell();
    let mut n = 0;
    // nocite: test data of our own (three times the strike, a few cells ahead)
    for dy in -4..=4 {
        for dx in 6..=9 {
            let c = CellXY::new(lead.x + dx, lead.y + dy);
            if n < 3 * t.strike
                && w.cells.usable(c)
                && w.objects.values().all(|o| o.pos.cell() != c)
            {
                testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 1, c, 0);
                n += 1;
            }
        }
    }
    // nocite: test data of our own
    shroud::area_reveal(&mut w, HouseId(0), Pos::centre(lead, 0), 14);
    let home = s.out().clone();
    // nocite: test data of our own (one think period)
    let got = play(
        &mut w,
        &mut s,
        level,
        glasnost_ai::scripted::THINK_PERIOD + 1,
    );
    assert_eq!(s.recalls(), 1);
    assert!(s.out().is_empty());
    let called: BTreeSet<ObjectId> = got
        .iter()
        .filter_map(|(_, o)| match o {
            Order::Behaviour {
                actor,
                behaviour: Behaviour::Move,
                attack_move: false,
                ..
            } if home.contains(actor) => Some(*actor),
            _ => None,
        })
        .collect();
    assert!(!called.is_empty(), "the strike is called home: {got:?}");
}

// design: own AI (#449): a strike that reaches its goal, a seen enemy building, attacks it: an AI house counts as human-controlled (glossary), so its units' own searches pass a building with no weapon by (S-0309 table C, row "harmless building"), and the strike stood beside the enemy base
#[test]
fn the_strike_attacks_the_building_it_reaches() {
    let level = AiDifficulty::Hard;
    let t = tuning(level);
    let mut w = base_up();
    let target = yard(&w, 1).expect("house 1's yard stands");
    let at = w.obj(target).unwrap().pos.cell();
    // nocite: test data of our own (house 0 sees house 1's base)
    shroud::area_reveal(&mut w, HouseId(0), Pos::centre(at, 0), 10);
    let mut s = Siege::start(&w, &fixture::map(), HouseId(0), level);
    // The strike's count and room for the brain's own parts.
    let ids = tanks(&mut w, t.strike + 3);
    // nocite: test data of our own (time to drive there and fire)
    let got = play(&mut w, &mut s, level, 3000);
    assert!(s.strikes() >= 1, "a strike goes");
    let attackers: BTreeSet<ObjectId> = got
        .iter()
        .filter_map(|(_, o)| match o {
            Order::Behaviour {
                actor,
                behaviour: Behaviour::Attack,
                target: Some(Target::Object(e)),
                ..
            } if *e == target && ids.contains(actor) => Some(*actor),
            _ => None,
        })
        .collect();
    assert!(
        !attackers.is_empty(),
        "the strike attacks the yard it reaches"
    );
}

/// House 0's factory buildings on the map that make `kind` (not naval).
fn factories(w: &World, kind: FactoryKind) -> usize {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(0)) && o.on_map && o.kind == Kind::Building)
        .filter(|o| {
            w.rules()
                .buildings
                .get(o.type_index as usize)
                .is_some_and(|b| b.factory == Some(kind) && !b.naval)
        })
        .count()
}

/// House 0's infantry and vehicle factories after `ticks` of play at
/// `level` from the base, with `credits` in hand at the start. House 1's
/// yard has a guard of tanks, so that hard's first soldiers (trained beside
/// its harvesters) do not end the game before the factories come.
fn factories_when_rich(level: AiDifficulty, credits: i32, ticks: i32) -> (usize, usize) {
    let mut w = base_up();
    w.houses[0].credits = credits;
    let at = w.obj(yard(&w, 1).unwrap()).unwrap().pos.cell();
    // nocite: test data of our own (six tanks beside house 1's yard)
    for dy in [-4, -2, 0, 2, 4, 6] {
        testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "GUNR",
            1,
            CellXY::new(at.x - 4, at.y + dy),
            0,
        );
    }
    let mut s = Siege::start(&w, &fixture::map(), HouseId(0), level);
    play(&mut w, &mut s, level, ticks);
    (
        factories(&w, FactoryKind::Infantry),
        factories(&w, FactoryKind::Vehicle),
    )
}

/// House 0's base with a barracks, a refinery and a war factory standing
/// and no harvester (fewer than any level's plan keeps), `credits` in hand;
/// returns the infantry types (by index) house 0 is ordered to build over
/// one think period at `level`.
fn soldiers_while_short(level: AiDifficulty, credits: i32) -> Vec<u32> {
    let mut w = base_up();
    let at = w.obj(yard(&w, 0).unwrap()).unwrap().pos.cell();
    // nocite: test data of our own (free cells around the yard)
    for (name, dx, dy) in [("PILL", -6, -6), ("PROC", 4, -7), ("FACT", -7, 4)] {
        testkit::spawn(
            &mut w,
            Kind::Building,
            name,
            0,
            CellXY::new(at.x + dx, at.y + dy),
            0,
        );
    }
    let minr = w.rules().index_of(Kind::Vehicle, "MINR").unwrap();
    assert!(
        !w.objects.values().any(|o| o.owner == Some(HouseId(0))
            && o.kind == Kind::Vehicle
            && o.type_index == minr),
        "no harvester yet"
    );
    w.houses[0].credits = credits;
    let mut s = Siege::start(&w, &fixture::map(), HouseId(0), level);
    // nocite: test data of our own (one think period and a tick)
    let got = play(
        &mut w,
        &mut s,
        level,
        glasnost_ai::scripted::THINK_PERIOD + 1,
    );
    got.iter()
        .filter_map(|(_, o)| match o {
            Order::Build { what } if what.kind == Kind::Infantry => Some(what.index),
            _ => None,
        })
        .collect()
}

// design: own AI (#449): at hard, while the plan's harvesters do not stand, the brain's army round waits for them; the siege trains a soldier on the free infantry line when the money in hand pays for the next harvester and the soldier both, and not when it pays for less; normal and easy wait
#[test]
fn hard_trains_soldiers_beside_the_harvesters() {
    assert!(tuning(AiDifficulty::Hard).beside);
    assert!(!tuning(AiDifficulty::Normal).beside && !tuning(AiDifficulty::Easy).beside);
    // nocite: test data of our own (the fixture's harvester costs 1000 and its soldier 200)
    let (rich, poor) = (20_000, 1_199);
    let hard = soldiers_while_short(AiDifficulty::Hard, rich);
    assert!(!hard.is_empty(), "hard trains a soldier while short");
    assert!(
        soldiers_while_short(AiDifficulty::Hard, poor).is_empty(),
        "not with money for the harvester alone"
    );
    assert!(
        soldiers_while_short(AiDifficulty::Normal, rich).is_empty(),
        "normal waits for its harvesters"
    );
}

/// House 0's base with a power plant, a refinery, a barracks and a war
/// factory standing and three harvesters by them (the plan's count at
/// normal and hard), `credits` in hand; returns house 0's refineries on the
/// map after `ticks` of play at `level`.
fn refineries_standing(level: AiDifficulty, credits: i32, ticks: i32) -> usize {
    let mut w = base_up();
    let at = w.obj(yard(&w, 0).unwrap()).unwrap().pos.cell();
    // nocite: test data of our own (free cells around the yard)
    for (name, dx, dy) in [
        ("POWR", -6, 4),
        ("PILL", -6, -6),
        ("PROC", 4, -7),
        ("FACT", -7, 4),
    ] {
        testkit::spawn(
            &mut w,
            Kind::Building,
            name,
            0,
            CellXY::new(at.x + dx, at.y + dy),
            0,
        );
    }
    // nocite: test data of our own (three harvesters south of the yard)
    for dx in [-2, 0, 2] {
        testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "MINR",
            0,
            CellXY::new(at.x + dx, at.y + 6),
            0,
        );
    }
    w.houses[0].credits = credits;
    let proc = w.rules().index_of(Kind::Building, "PROC").unwrap();
    let mut s = Siege::start(&w, &fixture::map(), HouseId(0), level);
    play(&mut w, &mut s, level, ticks);
    w.objects
        .values()
        .filter(|o| {
            o.owner == Some(HouseId(0))
                && o.on_map
                && o.kind == Kind::Building
                && o.type_index == proc
        })
        .count()
}

// design: own AI (#449): at hard, once the plan's harvesters stand, the siege orders more refineries on the free building line while the house owns fewer than the level's count, so its credits do not run out with fields unworked; normal leaves it to the brain's own rule (a new field once the near one runs low)
#[test]
fn hard_builds_more_refineries() {
    // nocite: test data of our own (money for the refineries and more; a few minutes of play)
    let (credits, ticks) = (20_000, 4500);
    let hard = refineries_standing(AiDifficulty::Hard, credits, ticks);
    let normal = refineries_standing(AiDifficulty::Normal, credits, ticks);
    assert!(hard > normal, "hard {hard} refineries, normal {normal}");
}

// design: own AI (#449): at hard, with money to spare in hand, the siege adds factories of the army's kinds (S-0403 claim 6: more factories of a kind, shorter build times), up to three of each; normal and easy do not
#[test]
fn hard_adds_factories_when_rich() {
    // nocite: test data of our own (money enough for the base, the army and more factories; a few minutes of play)
    let (credits, ticks) = (40_000, 6000);
    let (inf, veh) = factories_when_rich(AiDifficulty::Hard, credits, ticks);
    assert!(
        inf.max(veh) >= 2,
        "hard adds a factory when rich: {inf} infantry, {veh} vehicle"
    );
    // nocite: design: own AI (#449): three of each kind at most
    assert!(inf <= 3 && veh <= 3, "{inf} infantry, {veh} vehicle");
    let (n_inf, n_veh) = factories_when_rich(AiDifficulty::Normal, credits, ticks);
    assert!(
        n_inf.max(n_veh) <= 1,
        "normal adds none: {n_inf} infantry, {n_veh} vehicle"
    );
    assert_eq!(tuning(AiDifficulty::Normal).rich, 0);
    assert_eq!(tuning(AiDifficulty::Easy).rich, 0);
}
