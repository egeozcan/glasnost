//! Claim tests of S-0212 r82's threat records (claim 16a) and of the
//! corridor exemption that buildings and aircraft give (claim 13), on
//! synthetic rules: invented types and numbers, no game data. The shapes
//! follow scenario P30's parts where a synthetic scene can stand in for
//! them.

mod structures_common;

use glasnost_map::CellXY;
use glasnost_map::triggers::Triggers;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::UnitType;
use glasnost_sim::corridor;
use glasnost_sim::econ::victory;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::garrison;
use glasnost_sim::threat::{self, Block, block_of};
use glasnost_sim::world::{HouseId, HouseRole, HouseSetup, ObjectId, Target, World};
use structures_common::*;

/// Invented threat fields.
const BUGGY_THREAT: i32 = 40;
const HUT3_THREAT: i32 = 36;
const PER_OCCUPANT: i32 = 7;

/// The garrison ruleset with invented threat fields: BUGGY 40, HUT3 36,
/// the per-occupant threat 7, and the aircraft type KITE (field 24).
fn threat_ruleset() -> Ruleset {
    let mut r = ruleset();
    let s = &mut r.sim;
    s.vehicles.get_mut("BUGGY").unwrap().combat.threat_posed = Some(BUGGY_THREAT);
    s.buildings.get_mut("HUT3").unwrap().combat.threat_posed = Some(HUT3_THREAT);
    s.general.threat_per_occupant = Some(PER_OCCUPANT);
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("KITE".into());
    let mut kite = UnitType {
        strength: Some(100),
        speed: Some(20),
        ..Default::default()
    };
    kite.combat.threat_posed = Some(24);
    s.aircraft.insert("KITE".into(), kite);
    r
}

fn threat_game() -> World {
    game_on(&threat_ruleset(), Triggers::default())
}

/// The records of house `h` in the 5 × 5 blocks round block `b`, row by
/// row.
fn window(w: &World, h: HouseId, b: Block) -> Vec<Vec<i32>> {
    (-2..=2)
        .map(|dy| {
            (-2..=2)
                .map(|dx| threat::record(w, h, (b.0 + dx, b.1 + dy)))
                .collect()
        })
        .collect()
}

/// The spread of amount `a` centred in the 5 × 5 window (claim 16a).
fn pattern(a: i32) -> Vec<Vec<i32>> {
    (-2i32..=2)
        .map(|dy| {
            (-2i32..=2)
                .map(|dx| match (dx.abs(), dy.abs()) {
                    (0, 0) => a,
                    (0, 1) | (1, 0) => a.div_euclid(2),
                    (1, 1) => a.div_euclid(4),
                    _ => 0,
                })
                .collect()
        })
        .collect()
}

fn empty() -> Vec<Vec<i32>> {
    pattern(0)
}

/// Every record of house `h`, summed.
fn total(w: &World, h: HouseId) -> i32 {
    w.threat
        .records
        .get(usize::from(h.0))
        .map_or(0, |r| r.values().sum())
}

// spec: S-0212 claim 16a (an enemy's object adds its amount at its block, half at the four edge neighbours and a quarter at the four diagonal ones, in the records of every house that does not ally its owner, never its owner's), scenario P30 part a
#[test]
fn claim_16a_an_enemy_vehicle_is_spread_over_3_by_3_blocks() {
    let mut w = threat_game();
    let c = at(0, 0);
    put(&mut w, Kind::Vehicle, "BUGGY", 1, 0, 0);
    let b = block_of(c);
    assert_eq!(window(&w, A, b), pattern(BUGGY_THREAT));
    assert_eq!(
        window(&w, CIV, b),
        pattern(BUGGY_THREAT),
        "the civilian house records it too"
    );
    assert_eq!(
        total(&w, B),
        0,
        "the owner's records never hold its own objects"
    );
    assert_eq!(threat::value(&w, A, c), BUGGY_THREAT);
    assert_eq!(
        threat::value(&w, A, CellXY::new(c.x + 4, c.y)),
        BUGGY_THREAT / 2
    );
}

// spec: S-0212 Terms (threat block (⌊x/4⌋, ⌊y/4⌋)) and claim 16a (a building's block holds its top-left foundation cell, not its centre), scenario P30 part a2
#[test]
fn claim_16a_a_large_building_counts_at_its_top_left_cell() {
    let mut w = threat_game();
    // at(-1, 0) lies in the last column of its block; the 3 × 2 building's
    // centre lies in the next block.
    let tl = at(-1, 0);
    assert_eq!(tl.x.rem_euclid(4), 3);
    put(&mut w, Kind::Building, "HUT3", 1, -1, 0);
    assert_eq!(window(&w, A, block_of(tl)), pattern(HUT3_THREAT));
    assert_ne!(block_of(tl), block_of(at(0, 0)));
}

// spec: S-0212 claim 16a (nothing but the threat field enters the amount: not the weapons; a type with no threat field adds nothing), scenario P30 part b
#[test]
fn claim_16a_an_armed_type_without_a_threat_field_adds_nothing() {
    let mut w = threat_game();
    put(&mut w, Kind::Vehicle, "CART", 1, 0, 0);
    put(&mut w, Kind::Building, "TOWER", 1, 4, 4);
    assert_eq!(total(&w, A), 0);
}

// spec: S-0212 claim 16a (leaving the map takes the amount last added away where it was added), scenario P30 part d
#[test]
fn claim_16a_removal_takes_the_amount_away() {
    let mut w = threat_game();
    let b = put(&mut w, Kind::Building, "HUT3", 1, 0, 0);
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 1, 2, 0);
    assert!(total(&w, A) > 0);
    glasnost_sim::objects::remove(&mut w, b);
    assert_eq!(window(&w, A, block_of(at(2, 0))), pattern(BUGGY_THREAT));
    glasnost_sim::objects::remove(&mut w, v);
    assert_eq!(total(&w, A), 0);
    assert!(w.threat.records[0].is_empty(), "no zero record is kept");
}

// spec: S-0212 claim 16a (a vehicle's amount moves in the tick its recorded cell enters a cell of another threat block, and not while it moves within one), scenario P30 part e
#[test]
fn claim_16a_a_moving_vehicle_moves_its_amount_between_blocks() {
    let mut w = threat_game();
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 1, -8, 0);
    glasnost_sim::orders::issue(
        &mut w,
        B,
        Order::Behaviour {
            actor: v,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at(8, 0))),
        },
    );
    let mut blocks = vec![block_of(at(-8, 0))];
    for _ in 0..600 {
        glasnost_sim::tick::step(&mut w);
        let o = w.obj(v).unwrap();
        // The cell the vehicle is listed in, last entered.
        let listed: Vec<CellXY> = (-10..=10)
            .flat_map(|dx| (-2..=2).map(move |dy| at(dx, dy)))
            .filter(|&c| w.cells.list(c, o.layer).contains(&v))
            .collect();
        let t = &w.threat.tracks[&v];
        assert!(
            listed.iter().any(|&c| block_of(c) == t.block),
            "the amount's block is that of a cell the vehicle is listed in"
        );
        assert_eq!(window(&w, A, t.block), pattern(BUGGY_THREAT));
        if blocks.last() != Some(&t.block) {
            blocks.push(t.block);
        }
        if o.pos.cell() == at(8, 0) && o.dest.is_none() {
            break;
        }
    }
    assert_eq!(w.obj(v).unwrap().pos.cell(), at(8, 0), "arrived");
    assert_eq!(
        blocks,
        (4..=8).map(|x| (x, 6)).collect::<Vec<_>>(),
        "one move per block crossed"
    );
}

// spec: S-0212 claim 16a (a building with occupants: their number × the per-occupant threat, figured again at each entry; the owner changes of S-0508 claims 7 and 8 take away and add the same amount at the same block for a house that records both owners), scenario P30 part f
#[test]
fn claim_16a_occupants_set_a_buildings_amount() {
    let mut w = threat_game();
    let hut = put(&mut w, Kind::Building, "HUT3", 2, 0, 0);
    let b = block_of(at(0, 0));
    assert_eq!(window(&w, A, b), pattern(HUT3_THREAT));
    for n in 1..=3 {
        let u = put(&mut w, Kind::Infantry, "GARRI", 0, -6, -6);
        assert!(garrison::attempt(&mut w, u, hut));
        assert_eq!(garrison::count(&w, hut), n);
        // Beta records both the civilian house's and Alpha's objects.
        assert_eq!(window(&w, B, b), pattern(n * PER_OCCUPANT), "{n} occupants");
        glasnost_sim::tick::step(&mut w);
        assert_eq!(window(&w, B, b), pattern(n * PER_OCCUPANT), "{n} occupants");
    }
    assert_eq!(owner(&w, hut), Some(A), "the first turn after an entry");
    assert_eq!(total(&w, A), 0, "Alpha's own building now");
    assert!(garrison::evict(&mut w, hut, garrison::Evict::PLAIN));
    for _ in 0..30 {
        glasnost_sim::tick::step(&mut w);
    }
    assert_eq!(garrison::count(&w, hut), 0);
    assert_eq!(owner(&w, hut), Some(CIV));
    assert_eq!(
        window(&w, B, b),
        pattern(HUT3_THREAT),
        "its own field again"
    );
    assert_eq!(window(&w, A, b), pattern(HUT3_THREAT));
}

// spec: S-0212 claim 16a (taking away subtracts the same three amounts and a record falls no lower than 0) and quirks.negative_threat_field (on: a negative field takes its amount away when placed and adds it back when removed)
#[test]
fn negative_threat_field_on_leaves_a_record_higher() {
    for on in [true, false] {
        let mut r = threat_ruleset();
        r.sim.vehicles.get_mut("CART").unwrap().combat.threat_posed = Some(-20);
        r.globals.quirks.negative_threat_field = on;
        let mut w = game_on(&r, Triggers::default());
        let p = put(&mut w, Kind::Vehicle, "BUGGY", 1, 0, 0);
        let n = put(&mut w, Kind::Vehicle, "CART", 1, 1, 1);
        let b = block_of(at(0, 0));
        assert_eq!(block_of(at(1, 1)), b);
        let both = if on { 20 } else { BUGGY_THREAT };
        assert_eq!(window(&w, A, b), pattern(both), "on {on}");
        glasnost_sim::objects::remove(&mut w, p);
        glasnost_sim::objects::remove(&mut w, n);
        let left = if on { pattern(20) } else { empty() };
        assert_eq!(window(&w, A, b), left, "on {on}");
    }
}

/// A game of the threat ruleset whose third house, Gamma, is a player
/// (computer-controlled when `computer`, an AI house when `ai`), with
/// alliance orders allowed.
fn three_houses(computer: bool, ai: bool) -> World {
    three_houses_on(threat_ruleset(), computer, ai)
}

fn three_houses_on(mut r: Ruleset, computer: bool, ai: bool) -> World {
    r.sim.general.economy.allies_allowed = Some(true);
    let map = glasnost_sim::testkit::empty_map(SIZE);
    let mut s = glasnost_sim::testkit::setup(7);
    s.houses.push(HouseSetup {
        name: "Gamma".into(),
        country: "Betaland".into(),
        human: !computer,
        start: None,
        role: HouseRole::Player,
        ai,
    });
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-test"));
    let mut w = glasnost_sim::load::new_game(rules, &map, glasnost_sim::testkit::catalogue(), &s)
        .expect("synthetic test game loads");
    glasnost_sim::testkit::run(&mut w, 4);
    w
}

const G: HouseId = HouseId(2);

// spec: S-0212 claim 16a (an alliance order rebuilds the issuing house's records: every building whoever owns it, and a unit not its own when the house does not ally its owner or is computer-controlled, an AI house counted as computer-controlled; the incremental rules leave those friendly contributions in place after the objects leave) and quirks.threat_rebuild_counts_friends, scenario P30 parts c and g
#[test]
fn claim_16a_a_rebuild_counts_friends() {
    for (human, ai) in [(true, false), (false, false), (true, true)] {
        let computer = !human || ai;
        let mut w = three_houses(!human, ai);
        let own = put(&mut w, Kind::Building, "HUT3", 2, -8, -8);
        let ally = put(&mut w, Kind::Vehicle, "BUGGY", 1, 8, -8);
        put(&mut w, Kind::Vehicle, "BUGGY", 0, 0, 8);
        assert!(total(&w, G) > 0);
        assert_eq!(
            window(&w, G, block_of(at(-8, -8))),
            empty(),
            "its own: never"
        );
        victory::alliance(&mut w, G, B);
        assert!(w.allied(Some(G), Some(B)));
        // Its own building counts in the rebuild.
        assert_eq!(window(&w, G, block_of(at(-8, -8))), pattern(HUT3_THREAT));
        // Its ally's vehicle only when Gamma is computer-controlled.
        let expect = if computer {
            pattern(BUGGY_THREAT)
        } else {
            empty()
        };
        assert_eq!(
            window(&w, G, block_of(at(8, -8))),
            expect,
            "computer {computer}"
        );
        // The enemy's vehicle, as before.
        assert_eq!(window(&w, G, block_of(at(0, 8))), pattern(BUGGY_THREAT));
        // Stale threat: nothing takes the friendly contributions away.
        glasnost_sim::objects::remove(&mut w, own);
        glasnost_sim::objects::remove(&mut w, ally);
        assert_eq!(window(&w, G, block_of(at(-8, -8))), pattern(HUT3_THREAT));
        assert_eq!(window(&w, G, block_of(at(8, -8))), expect);
    }
}

// spec: S-0212 Edge cases (quirks.threat_rebuild_counts_friends off: the rebuild leaves out the house's own objects and those of houses it allies)
#[test]
fn threat_rebuild_counts_friends_off_leaves_friends_out() {
    let mut r = threat_ruleset();
    r.globals.quirks.threat_rebuild_counts_friends = false;
    let mut w = three_houses_on(r, true, false);
    put(&mut w, Kind::Building, "HUT3", 2, -8, -8);
    put(&mut w, Kind::Vehicle, "BUGGY", 1, 8, -8);
    put(&mut w, Kind::Vehicle, "BUGGY", 0, 0, 8);
    victory::alliance(&mut w, G, B);
    assert_eq!(window(&w, G, block_of(at(-8, -8))), empty());
    assert_eq!(window(&w, G, block_of(at(8, -8))), empty());
    assert_eq!(window(&w, G, block_of(at(0, 8))), pattern(BUGGY_THREAT));
}

// spec: S-0212 claim 16a (the recording house's own side of the relation decides: when only A allies B, B's objects stay out of A's records while A's are in B's)
#[test]
fn claim_16a_a_one_sided_alliance() {
    let mut w = three_houses(false, false);
    victory::alliance(&mut w, G, B);
    assert!(!w.allied(Some(B), Some(G)), "one way");
    put(&mut w, Kind::Vehicle, "BUGGY", 1, 8, 0);
    put(&mut w, Kind::Vehicle, "BUGGY", 2, -8, 0);
    assert_eq!(
        window(&w, G, block_of(at(8, 0))),
        empty(),
        "Gamma allies Beta"
    );
    assert_eq!(
        window(&w, B, block_of(at(-8, 0))),
        pattern(BUGGY_THREAT),
        "Beta does not ally Gamma"
    );
}

// spec: S-0212 claim 16a (breaking an alliance: the order ends it in both directions and rebuilds the records of each house whose alliance ended)
#[test]
fn claim_16a_breaking_an_alliance_rebuilds_both_houses() {
    let mut w = three_houses(false, false);
    victory::alliance(&mut w, G, B);
    victory::alliance(&mut w, B, G);
    put(&mut w, Kind::Vehicle, "BUGGY", 1, 8, 0);
    put(&mut w, Kind::Vehicle, "BUGGY", 2, -8, 0);
    assert_eq!(total(&w, G), 0);
    assert_eq!(window(&w, B, block_of(at(-8, 0))), empty());
    victory::alliance(&mut w, G, B);
    assert!(!w.allied(Some(G), Some(B)) && !w.allied(Some(B), Some(G)));
    assert_eq!(window(&w, G, block_of(at(8, 0))), pattern(BUGGY_THREAT));
    assert_eq!(window(&w, B, block_of(at(-8, 0))), pattern(BUGGY_THREAT));
}

// spec: S-0212 claim 16a (an aircraft's amount is added at the block of its placement cell and does not follow it in flight or to a landing elsewhere; it is taken away at the block of its recorded cell) and quirks.aircraft_threat_stays_put, scenario P30 part h1
#[test]
fn aircraft_threat_stays_put() {
    for on in [true, false] {
        let mut r = threat_ruleset();
        r.globals.quirks.aircraft_threat_stays_put = on;
        let mut w = game_on(&r, Triggers::default());
        let k = put(&mut w, Kind::Aircraft, "KITE", 1, -8, 0);
        let start = block_of(at(-8, 0));
        assert_eq!(window(&w, A, start), pattern(24));
        let grounded = w.obj(k).unwrap().air.as_ref().is_some_and(|a| a.ground);
        let t = w.threat.tracks[&k].clone();
        assert_eq!(t.cell.is_some(), grounded || !on, "the recorded cell");
        // A landing elsewhere.
        let there = at(8, 0);
        w.obj_mut(k).unwrap().pos = glasnost_sim::world::Pos::centre(there, 0);
        threat::landed(&mut w, k);
        if on {
            assert_eq!(
                window(&w, A, start),
                pattern(24),
                "stays at its placement block"
            );
            assert_eq!(window(&w, A, block_of(there)), empty());
        } else {
            assert_eq!(window(&w, A, start), empty());
            assert_eq!(
                window(&w, A, block_of(there)),
                pattern(24),
                "moved as a vehicle's"
            );
        }
        assert!(
            corridor::exempt(&w, CellXY::new(there.x + 1, there.y)),
            "exempt round the landing cell"
        );
        assert!(!corridor::exempt(&w, there), "not the landing cell itself");
        threat::leaving(&mut w, k);
        if on {
            // Taken away at the landing block, where it was never added.
            assert_eq!(
                window(&w, A, start),
                pattern(24),
                "stale at its placement block"
            );
            assert_eq!(window(&w, A, block_of(there)), empty());
        } else {
            assert_eq!(total(&w, A), 0);
        }
    }
}

// spec: S-0212 Terms (exemption rectangle: the foundation grown by one cell on every side) and claim 13 (every building gives the exemption to its rectangle from the moment it is on the map, whatever its owner; taken away when it leaves the map; sources add up)
#[test]
fn claim_13_a_buildings_exemption_rectangle() {
    let mut w = threat_game();
    let tl = at(0, 0);
    let b = put(&mut w, Kind::Building, "HUT3", 1, 0, 0);
    let exempt: Vec<(i32, i32)> = (-3..=6)
        .flat_map(|dy| (-3..=6).map(move |dx| (dx, dy)))
        .filter(|&(dx, dy)| corridor::exempt(&w, CellXY::new(tl.x + dx, tl.y + dy)))
        .collect();
    let rect: Vec<(i32, i32)> = (-1..=2)
        .flat_map(|dy| (-1..=3).map(move |dx| (dx, dy)))
        .collect();
    assert_eq!(
        exempt, rect,
        "3 × 2 grown by one: 5 × 4 from one lower in x and y"
    );
    // A second building overlapping the rectangle: sources add up.
    let b2 = put(&mut w, Kind::Building, "HUT1", 0, 4, 0);
    glasnost_sim::objects::remove(&mut w, b);
    assert!(
        corridor::exempt(&w, CellXY::new(tl.x + 3, tl.y)),
        "still held by the second"
    );
    assert!(!corridor::exempt(&w, CellXY::new(tl.x - 1, tl.y - 1)));
    glasnost_sim::objects::remove(&mut w, b2);
    assert!(w.threat.rects.is_empty());
}

// spec: S-0212 claim 13 (the exemption does not follow an owner change: every building gives it whatever its owner) and claim 16a (an owner change moves the amount between the recording houses)
#[test]
fn claim_13_and_16a_an_owner_change() {
    let mut w = threat_game();
    let b = put(&mut w, Kind::Building, "HUT3", 1, 0, 0);
    let rects = w.threat.rects.clone();
    let blk = block_of(at(0, 0));
    assert_eq!(window(&w, A, blk), pattern(HUT3_THREAT));
    assert_eq!(total(&w, B), 0);
    glasnost_sim::mind::set_owner(&mut w, b, Some(A));
    assert_eq!(w.threat.rects, rects);
    assert_eq!(total(&w, A), 0, "Alpha's own now");
    assert_eq!(window(&w, B, blk), pattern(HUT3_THREAT));
    assert_eq!(window(&w, CIV, blk), pattern(HUT3_THREAT));
}

/// The records each house would hold from the tracked objects' amounts
/// last added, with no rebuild (claim 16a's incremental rules), by house
/// and block.
fn summed(w: &World) -> Vec<std::collections::BTreeMap<Block, i32>> {
    let side = (i32::from(w.cells.footprint().coordinate_side()) + 3).div_euclid(4);
    let mut out = vec![std::collections::BTreeMap::new(); w.houses.len()];
    for (&id, t) in &w.threat.tracks {
        let Some(o) = w.obj(id) else { continue };
        for (r, rec) in out.iter_mut().enumerate() {
            let r = HouseId(r as u8);
            if o.owner
                .is_none_or(|own| own == r || w.allied(Some(r), Some(own)))
            {
                continue;
            }
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let b = (t.block.0 + dx, t.block.1 + dy);
                    if b.0 < 0 || b.1 < 0 || b.0 >= side || b.1 >= side {
                        continue;
                    }
                    let v = match dx.abs() + dy.abs() {
                        0 => t.amount,
                        1 => t.amount / 2,
                        _ => t.amount / 4,
                    };
                    *rec.entry(b).or_insert(0) += v;
                }
            }
        }
    }
    for rec in &mut out {
        rec.retain(|_, v| *v > 0);
    }
    out
}

// spec: S-0212 claim 16a (the records change at once at each event of the table: after every tick of a fight with movement and deaths, each vehicle's and infantry unit's amount sits at the block of a cell it is listed in, and every house's records are the sum of the amounts last added, with no rebuild in this game)
#[test]
fn claim_16a_the_records_follow_the_objects_in_a_fight() {
    let mut r = glasnost_sim::testkit::combat_ruleset();
    for (i, t) in r.sim.vehicles.values_mut().enumerate() {
        t.combat.threat_posed = Some(10 + 7 * i as i32);
    }
    for t in r.sim.infantry.values_mut() {
        t.combat.threat_posed = Some(13);
    }
    for t in r.sim.buildings.values_mut() {
        t.combat.threat_posed = Some(21);
    }
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-combat"));
    let rep = glasnost_sim::testkit::combat_replay(11, 900);
    let mut w = glasnost_sim::load::new_game(
        rules,
        &glasnost_sim::testkit::map(),
        glasnost_sim::testkit::catalogue(),
        &rep.setup,
    )
    .expect("the scenario loads");
    let start = w.threat.tracks.len();
    assert!(start > 4);
    let mut moves = 0;
    let mut last: std::collections::BTreeMap<ObjectId, Block> = std::collections::BTreeMap::new();
    rep.run_on(&mut w, 900, |w| {
        for (&id, t) in &w.threat.tracks {
            let o = w.obj(id).expect("a tracked object exists");
            if matches!(o.kind, Kind::Vehicle | Kind::Infantry) {
                let c = o.pos.cell();
                let listed = (-1..=1)
                    .flat_map(|dy| (-1..=1).map(move |dx| CellXY::new(c.x + dx, c.y + dy)))
                    .filter(|&n| w.cells.list(n, o.layer).contains(&id))
                    .any(|n| block_of(n) == t.block);
                assert!(listed, "{id:?} at tick {}", w.tick);
            }
            if last.insert(id, t.block).is_some_and(|b| b != t.block) {
                moves += 1;
            }
        }
        let have: Vec<_> = (0..w.houses.len())
            .map(|h| w.threat.records.get(h).cloned().unwrap_or_default())
            .collect();
        assert_eq!(have, summed(w), "tick {}", w.tick);
    });
    assert!(moves > 10, "the units crossed blocks: {moves}");
    assert!(w.threat.tracks.len() < start, "some died");
}

/// The cells of the path computed for a fresh vehicle of type SHY (BUGGY
/// with threat coefficient `k`, no threat field of its own) of Alpha from
/// at(−12, 0) to at(12, 0), with Beta's BUGGY (field 40) standing at
/// at(0, 3) beside the row when `enemy`.
fn shy_path(k: &str, enemy: bool) -> Vec<CellXY> {
    let mut r = threat_ruleset();
    r.registries.get_mut("vehicles").unwrap().push("SHY".into());
    let mut shy = r.sim.vehicles["BUGGY"].clone();
    shy.combat.threat_posed = None;
    shy.threat_avoidance_coefficient = Some(d(k));
    r.sim.vehicles.insert("SHY".into(), shy);
    let mut w = game_on(&r, Triggers::default());
    if enemy {
        put(&mut w, Kind::Vehicle, "BUGGY", 1, 0, 3);
    }
    let v = put(&mut w, Kind::Vehicle, "SHY", 0, -12, 0);
    w.obj_mut(v).unwrap().dest = Some(Target::Cell(at(12, 0)));
    assert_eq!(
        glasnost_sim::path::compute(&mut w, v, glasnost_sim::path::Mode::Fresh),
        glasnost_sim::path::Search::Found
    );
    let mut c = at(-12, 0);
    let mut cells = Vec::new();
    for &s in &w.obj(v).unwrap().path.steps {
        let (dx, dy) = glasnost_sim::regions::STEPS[usize::from(s % 8)];
        c = CellXY::new(c.x + dx, c.y + dy);
        cells.push(c);
    }
    cells
}

/// The largest distance of a path's cells from the row y = at(0, 0).y.
fn off_row(cells: &[CellXY]) -> i32 {
    cells
        .iter()
        .map(|c| (c.y - at(0, 0).y).abs())
        .max()
        .unwrap_or(0)
}

// spec: S-0212 claim 13 and F5 (a unit whose coefficient exceeds 0.00001 adds the coarse threat term of its owner's records to the entry cost of sectors at levels 1 and 2, so its route goes round threatened blocks), F7 and Edge cases (a zero coefficient, or one not above the binary64 value nearest 0.00001, ignores threat), scenario P10's shape
#[test]
fn claim_13_a_unit_with_a_threat_coefficient_goes_round_threat() {
    for k in ["0", "0.000001"] {
        assert_eq!(off_row(&shy_path(k, true)), 0, "k {k} drives straight");
    }
    assert_eq!(off_row(&shy_path("1", false)), 0, "no threat: straight");
    let bent = shy_path("1", true);
    assert!(off_row(&bent) >= 4, "k 1 bends round the blocks: {bent:?}");
}
