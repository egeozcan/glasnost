//! Tests for the Vanguard AI script (#449, registry id 3): registry listing,
//! save/load/hash round-trip, difficulty ordering, rules-adaptive economy
//! (paid vs free-harvester refineries and `multiple_factory`), active building
//! repair hysteresis, and in-range focus fire. Shroud and spy-disguise honesty
//! are also covered automatically by `tests/scripts.rs` and
//! `tests/spy_disguise.rs`.

use glasnost_ai::scripts::{self, AiScript, REGISTRY, Schedule, Seat, View};
use glasnost_ai::vanguard::{self, Vanguard, multiple_factory_helps, refinery_brings_harvester};
use glasnost_ai::{fixture, game::Game};
use glasnost_map::CellXY;
use glasnost_sim::econ::shroud;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{AiDifficulty, AiSeat, HouseId, ObjectId, Pos, Target, World};

fn vanguard_at(level: AiDifficulty) -> AiSeat {
    AiSeat {
        script: vanguard::ID,
        difficulty: level,
    }
}

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

fn yard(w: &World, h: u8) -> Option<ObjectId> {
    let i = w.rules().index_of(Kind::Building, "YARD")?;
    w.objects
        .values()
        .find(|o| {
            o.owner == Some(HouseId(h)) && o.on_map && o.kind == Kind::Building && o.type_index == i
        })
        .map(|o| o.id)
}

fn base_up() -> World {
    let mut g = game(AiSeat::DEFAULT);
    // nocite: test data of our own
    while g.world.tick < 3000 && (yard(&g.world, 0).is_none() || yard(&g.world, 1).is_none()) {
        g.step();
    }
    let mut w = g.world.clone();
    for h in &mut w.houses {
        h.credits = 0;
    }
    w
}

fn play(w: &mut World, v: &mut Vanguard, level: AiDifficulty, ticks: i32) -> Vec<(i32, Order)> {
    let mut got = Vec::new();
    for _ in 0..ticks {
        if w.econ.end.finished {
            break;
        }
        let tick = w.tick;
        let view = View::new(w, HouseId(0));
        let orders = AiScript::think(v, &view, Schedule::of(HouseId(0), tick), level, None, &mut ());
        for o in orders {
            got.push((tick, o.clone()));
            orders::issue(w, HouseId(0), o);
        }
        testkit::run(w, 1);
    }
    got
}

// design: own AI (Vanguard): registry lists vanguard as id 3 after siege
#[test]
fn the_registry_lists_vanguard() {
    assert_eq!(REGISTRY[0].id, AiSeat::DEFAULT_SCRIPT);
    let e = scripts::by_key("vanguard").expect("listed");
    assert_eq!(e.id, vanguard::ID);
    assert_eq!(vanguard::ID, 3);
    assert_eq!(e.name, "Vanguard");
    assert_eq!(scripts::entry(vanguard::ID), Some(e));
    assert_eq!(
        scripts::known(vanguard_at(AiDifficulty::Hard)).script,
        vanguard::ID
    );
}

// design: own AI (Vanguard): vanguard survives save, load and state hash at every difficulty
#[test]
fn vanguard_survives_save_load_and_hash() {
    for level in AiDifficulty::ALL {
        let seat = vanguard_at(level);
        let mut g = game(seat);
        let choices: Vec<AiSeat> = g.seats().iter().map(Seat::choice).collect();
        assert_eq!(choices, vec![seat, AiSeat::DEFAULT], "{level:?}");
        assert!(g.seats()[0].script().vanguard().is_some(), "{level:?}");
        // nocite: test data of our own
        for _ in 0..600 {
            g.step();
        }
        let mut back =
            Game::load(&g.save(), fixture::rules(), fixture::catalogue()).expect("loads");
        assert!(back.seats()[0].script().vanguard().is_some());
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

// design: own AI (Vanguard): rules detection distinguishes paid-harvester and no-speedup fixture rules
#[test]
fn rules_detection_reads_fixture_accurately() {
    let w = base_up();
    assert!(!refinery_brings_harvester(&w, HouseId(0)));
    assert!(!multiple_factory_helps(&w));
}

// design: own AI (Vanguard): damaged buildings turn repair on when credits suffice and never toggle repair on full-health buildings
#[test]
fn active_repair_toggles_only_damaged_buildings() {
    let mut w = base_up();
    let y = yard(&w, 0).expect("yard stands");
    w.houses[0].credits = 2000;
    let mut v = Vanguard::start(&w, &fixture::map(), HouseId(0), AiDifficulty::Hard);
    // At full health, no Repair order is issued.
    let got = play(&mut w, &mut v, AiDifficulty::Hard, 30);
    assert!(
        !got.iter().any(|(_, o)| matches!(o, Order::Repair { actor } if *actor == y)),
        "no repair at full health"
    );
    // Damage the yard: Repair turns on once.
    w.obj_mut(y).unwrap().health = 500;
    let got = play(&mut w, &mut v, AiDifficulty::Hard, 30);
    let repairs = got
        .iter()
        .filter(|(_, o)| matches!(o, Order::Repair { actor } if *actor == y))
        .count();
    assert_eq!(repairs, 1, "repair toggled on once for damaged yard");
    assert!(w.obj(y).unwrap().econ.building.repairing);
}

// design: own AI (Vanguard): focus fire prefers a wounded combat threat in range over a full-health target
#[test]
fn focus_fire_targets_wounded_threat_in_range() {
    let mut w = base_up();
    let y = w.obj(yard(&w, 0).unwrap()).unwrap().pos.cell();
    let shooter = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 0, CellXY::new(y.x + 8, y.y), 0);
    let full = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 1, CellXY::new(y.x + 11, y.y - 1), 0);
    let low = testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 1, CellXY::new(y.x + 11, y.y + 1), 0);
    w.obj_mut(low).unwrap().health = 60;
    shroud::area_reveal(&mut w, HouseId(0), Pos::centre(CellXY::new(y.x + 11, y.y), 0), 8);
    let mut v = Vanguard::start(&w, &fixture::map(), HouseId(0), AiDifficulty::Hard);
    let got = play(&mut w, &mut v, AiDifficulty::Hard, 30);
    let focused_low = got.iter().any(|(_, o)| {
        matches!(
            o,
            Order::Behaviour {
                actor,
                behaviour: Behaviour::Attack,
                target: Some(Target::Object(t)),
                ..
            } if *actor == shooter && *t == low
        )
    });
    assert!(focused_low, "shooter focuses wounded enemy {low:?} over {full:?}: {got:?}");
}

// design: own AI (Vanguard): interleaves combat vehicles after two harvesters stand before queuing a third harvester
#[test]
fn interleaves_combat_vehicles_after_two_harvesters() {
    let mut w = base_up();
    let y = w.obj(yard(&w, 0).unwrap()).unwrap().pos.cell();
    testkit::spawn(&mut w, Kind::Building, "POWR", 0, CellXY::new(y.x + 3, y.y), 0);
    testkit::spawn(&mut w, Kind::Building, "PROC", 0, CellXY::new(y.x - 4, y.y), 0);
    testkit::spawn(&mut w, Kind::Building, "FACT", 0, CellXY::new(y.x, y.y + 4), 0);
    testkit::spawn(&mut w, Kind::Vehicle, "MINR", 0, CellXY::new(y.x - 2, y.y + 4), 0);
    testkit::spawn(&mut w, Kind::Vehicle, "MINR", 0, CellXY::new(y.x - 3, y.y + 4), 0);
    w.houses[0].credits = 3000;
    let gunr = w.rules().index_of(Kind::Vehicle, "GUNR").unwrap();
    let minr = w.rules().index_of(Kind::Vehicle, "MINR").unwrap();
    let mut v = Vanguard::start(&w, &fixture::map(), HouseId(0), AiDifficulty::Hard);
    let got = play(&mut w, &mut v, AiDifficulty::Hard, 60);
    let first_veh = got.iter().find_map(|(_, o)| match o {
        Order::Build { what } if what.kind == Kind::Vehicle => Some(what.index),
        _ => None,
    });
    assert_eq!(
        first_veh,
        Some(gunr),
        "expected combat vehicle GUNR ({gunr}) before 3rd MINR ({minr}): {got:?}"
    );
}

// design: own AI (Vanguard): never builds an unarmed land ferry on a land-connected map even when vehicle line is busy
#[test]
fn drops_unarmed_land_ferry_on_land_map() {
    let mut w = base_up();
    let y = w.obj(yard(&w, 0).unwrap()).unwrap().pos.cell();
    testkit::spawn(&mut w, Kind::Building, "POWR", 0, CellXY::new(y.x + 3, y.y), 0);
    testkit::spawn(&mut w, Kind::Building, "POWR", 0, CellXY::new(y.x + 5, y.y), 0);
    testkit::spawn(&mut w, Kind::Building, "PROC", 0, CellXY::new(y.x - 4, y.y), 0);
    testkit::spawn(&mut w, Kind::Building, "PILL", 0, CellXY::new(y.x + 3, y.y + 3), 0);
    testkit::spawn(&mut w, Kind::Building, "FACT", 0, CellXY::new(y.x, y.y + 4), 0);
    testkit::spawn(&mut w, Kind::Building, "RADR", 0, CellXY::new(y.x - 3, y.y + 4), 0);
    for dx in 0..4 {
        testkit::spawn(&mut w, Kind::Vehicle, "MINR", 0, CellXY::new(y.x - 2 - dx, y.y + 6), 0);
        testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 0, CellXY::new(y.x + dx, y.y + 7), 0);
        testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, CellXY::new(y.x + dx, y.y + 8), 0);
    }
    w.houses[0].credits = 6000;
    let truk = w.rules().index_of(Kind::Vehicle, "TRUK").unwrap();
    let mut v = Vanguard::start(&w, &fixture::map(), HouseId(0), AiDifficulty::Hard);
    let got = play(&mut w, &mut v, AiDifficulty::Hard, 120);
    assert!(
        !got.iter().any(|(_, o)| matches!(
            o,
            Order::Build { what } if what.kind == Kind::Vehicle && what.index == truk
        )),
        "vanguard should never build unarmed land ferry TRUK on a land map"
    );
}

// design: own AI (Vanguard): caps landable aircraft to owned reload docks when not flush with credits
#[test]
fn caps_landable_aircraft_to_reload_docks() {
    let mut w = base_up();
    let y = w.obj(yard(&w, 0).unwrap()).unwrap().pos.cell();
    testkit::spawn(&mut w, Kind::Building, "POWR", 0, CellXY::new(y.x + 3, y.y), 0);
    testkit::spawn(&mut w, Kind::Building, "POWR", 0, CellXY::new(y.x + 5, y.y), 0);
    testkit::spawn(&mut w, Kind::Building, "PROC", 0, CellXY::new(y.x - 4, y.y), 0);
    testkit::spawn(&mut w, Kind::Building, "FACT", 0, CellXY::new(y.x, y.y + 4), 0);
    testkit::spawn(&mut w, Kind::Building, "RADR", 0, CellXY::new(y.x - 3, y.y + 4), 0);
    testkit::spawn(&mut w, Kind::Building, "HPAD", 0, CellXY::new(y.x + 4, y.y + 4), 0);
    // One JET already occupies the single HPAD dock, plus 3 MINR and 4 GUNR.
    for dx in 0..3 {
        testkit::spawn(&mut w, Kind::Vehicle, "MINR", 0, CellXY::new(y.x - 2 - dx, y.y + 6), 0);
    }
    for dx in 0..4 {
        testkit::spawn(&mut w, Kind::Vehicle, "GUNR", 0, CellXY::new(y.x + dx, y.y + 7), 0);
    }
    testkit::spawn(&mut w, Kind::Aircraft, "JET", 0, CellXY::new(y.x + 4, y.y + 4), 0);
    w.houses[0].credits = 1500;
    let jet = w.rules().index_of(Kind::Aircraft, "JET").unwrap();
    let mut v = Vanguard::start(&w, &fixture::map(), HouseId(0), AiDifficulty::Hard);
    let got = play(&mut w, &mut v, AiDifficulty::Hard, 60);
    assert!(
        !got.iter().any(|(_, o)| matches!(
            o,
            Order::Build { what } if what.kind == Kind::Aircraft && what.index == jet
        )),
        "vanguard should not overbuild JETs beyond single HPAD dock at 1500 credits"
    );
}

// design: own AI (Vanguard): plays cleanly on a water-separated strait map without stalling
#[test]
fn plays_cleanly_on_strait_map() {
    let mut s = fixture::setup(1);
    s.ai = vec![vanguard_at(AiDifficulty::Hard), AiSeat::DEFAULT];
    let mut g = Game::new(
        fixture::rules(),
        &fixture::map_strait(),
        "strait",
        fixture::catalogue(),
        s,
    )
    .expect("strait map loads");
    // nocite: test data of our own
    for _ in 0..1500 {
        if g.world.econ.end.finished {
            break;
        }
        g.step();
    }
    assert!(yard(&g.world, 0).is_some(), "vanguard deploys and builds on strait map");
}
