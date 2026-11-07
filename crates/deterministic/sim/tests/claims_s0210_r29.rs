//! Claim tests for S-0210 revision 29 claim 11 and S-0215 revision 88
//! claim 39 (M8 step A3 intake 52a; answers Q-2005): the crush response, by
//! which a crusher drives at a crushable enemy that hit it from close by;
//! who may make it (the original's computer-controlled row, kept by the
//! neutral and special houses and by test-only computer-controlled player
//! houses, and `quirks.auto_crush_computer_only` for player houses); and
//! its order within the hit (after a berserk entry and after retaliation).
//!
//! Synthetic data only: the testkit's combat ruleset, its crusher made
//! unarmed (S-0210 scenario K7's *unarmed*) and durable, with an invented
//! berserk warhead and a detector building; every number is ours. Ledger:
//! `docs/m8-claims/intake52a-S-0210-S-0215-S-0216.md`.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, WarheadType};
use glasnost_sim::cloak::{Disguise, DisguiseAs};
use glasnost_sim::combat::area::{self, Detonation};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::psychic;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;
/// The neutral and special houses of `testkit::game_with_non_players`.
const NEUTRAL: u8 = 2;
const SPECIAL: u8 = 3;
/// The crush range of the engine default (S-0210 F2: 1.5 cells).
const RANGE: i32 = 384;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The combat ruleset with the crusher unarmed and durable, a crushable
/// vehicle BUGGY, a berserk-causing area warhead MAD (spread 4 cells) and
/// a disguise-detector building EYE (radius 3 cells).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    let c = s.vehicles.get_mut("CRUSHER").unwrap();
    c.primary = None;
    c.strength = Some(30_000);
    s.warheads.insert(
        "MAD".into(),
        WarheadType {
            verses: Some(vec![d("0.5"); 11]),
            psychedelic: Some(true),
            cell_spread: Some(d("4")),
            percent_at_max: Some(d("1")),
            ..Default::default()
        },
    );
    let mut eye = BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    eye.combat.detect_disguise = Some(true);
    eye.detect_disguise_range = Some(3);
    s.buildings.insert("EYE".into(), eye);
    register(&mut r, "buildings", "EYE");
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

/// House `h` made a test-only computer-controlled player house (glossary)
/// with skill level `skill` and difficulty index `difficulty`.
fn computer(w: &mut World, h: u8, skill: i32, difficulty: i32) {
    let house = &mut w.houses[usize::from(h)];
    house.human = false;
    house.skill = skill;
    house.econ.setup.difficulty = difficulty;
}

fn warhead(w: &World, name: &str) -> u32 {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == name)
        .unwrap() as u32
}

/// A hit of 10 with warhead SA from `attacker`, naming it.
fn hit_from(w: &World, attacker: ObjectId) -> Hit {
    Hit {
        amount: 10,
        warhead: Some(warhead(w, "SA")),
        distance: 0,
        attacker: Some(attacker),
        house: w.obj(attacker).unwrap().owner,
        ignore_defences: false,
        no_escape: false,
    }
}

/// Applies [`hit_from`] `attacker` to `c` through the hit rules (S-0302).
fn hit(w: &mut World, c: ObjectId, attacker: ObjectId) -> damage::HitResult {
    let h = hit_from(w, attacker);
    damage::apply(w, c, h)
}

/// Moves `x` to `dx` leptons east of `of`'s centre, keeping it listed in
/// the cell it was spawned in (which must be the cell of that point).
fn east_of(w: &mut World, x: ObjectId, of: ObjectId, dx: i32) {
    let p = w.obj(of).unwrap().pos;
    let o = w.obj_mut(x).unwrap();
    let q = Pos::new(p.x + dx, p.y, p.z);
    assert_eq!(q.cell(), o.pos.cell(), "stays in its cell");
    o.pos = q;
}

/// Whether `c` made the crush response to `a`: the destination names `a`
/// and *move* is queued.
fn responded(w: &World, c: ObjectId, a: ObjectId) -> bool {
    let o = w.obj(c).unwrap();
    o.dest == Some(Target::Object(a)) && o.beh.queued == Some(Behaviour::Move)
}

/// A computer-owned crusher (skill 2, difficulty index 1) at (0, 0) and a
/// crushable infantry unit of BETA at (1, 0), one cell away.
fn scene() -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    computer(&mut w, ALPHA, 2, 1);
    let c = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
    let a = put(&mut w, Kind::Infantry, "WALKER", BETA, 1, 0);
    (w, c, a)
}

// ---------------------------------------------------------------------------
// The occasion and the effect (K7 cases 1 and 18)

// spec: S-0215 claim 39 (an enemy's hit from close by: the attacker object becomes the destination, not a cell; move is queued with no immediate start; the target is not changed) and S-0210 claim 11 (scenario K7 case 1: the computer-controlled row, skill level at least the threshold, difficulty index not 2)
#[test]
fn k7_case_1_a_hit_gives_the_attacker_as_destination_and_queues_move() {
    let (mut w, c, a) = scene();
    let before = w.obj(c).unwrap().beh.current;
    hit(&mut w, c, a);
    let o = w.obj(c).unwrap();
    assert_eq!(o.dest, Some(Target::Object(a)), "the attacker object");
    assert_eq!(o.beh.queued, Some(Behaviour::Move), "move queued");
    assert_eq!(o.beh.current, before, "no immediate start");
    assert_eq!(o.target, None, "the target is not changed");
}

// spec: S-0215 claim 39 (K7 case 1 played out: no response before the first hit; from the tick of that hit the destination names the infantry, move becomes current, the target stays empty) and S-0210 claims 3 and 4 (the crusher drives over the infantry, which leaves play with the crush sound and no damage event after the first hit)
#[test]
fn k7_case_1_the_crusher_drives_over_the_infantry_that_shoots_it() {
    let (mut w, c, a) = scene();
    w.obj_mut(a).unwrap().beh.current = Some(Behaviour::Sticky);
    while w.tick <= w.orders_blocked_through {
        glasnost_sim::tick::step(&mut w);
    }
    assert_eq!(w.obj(c).unwrap().dest, None, "no hit yet");
    let order = Order::Behaviour {
        actor: a,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(c)),
        dest: None,
    };
    assert!(orders::issue(&mut w, HouseId(BETA), order), "order taken");
    let mut hit_tick = None;
    let mut moved = false;
    let mut crushed = false;
    for _ in 0..600 {
        glasnost_sim::tick::step(&mut w);
        for e in std::mem::take(&mut w.effects) {
            match e {
                Effect::Hit { id, attacker, .. } if id == c && attacker == Some(a) => {
                    hit_tick.get_or_insert(w.tick);
                }
                Effect::Sound {
                    sound: Sound::Crush,
                    ..
                } => crushed = true,
                _ => {}
            }
        }
        if w.obj(a).is_none() {
            break;
        }
        let o = w.obj(c).unwrap();
        assert_eq!(o.target, None, "the target stays empty");
        if hit_tick.is_none() {
            assert_ne!(o.dest, Some(Target::Object(a)), "no response before a hit");
        } else {
            assert_eq!(o.dest, Some(Target::Object(a)), "the infantry object");
            moved |= o.beh.current == Some(Behaviour::Move);
        }
    }
    assert!(hit_tick.is_some(), "the infantry hit the crusher");
    assert!(moved, "move became current");
    assert!(w.obj(a).is_none(), "the infantry left play");
    assert!(crushed, "crushed (the crush sound)");
}

// spec: S-0215 claim 39 (the response is the only occasion: no hit, no response, in guard and in hunt; scenario K7 case 18 runs g and h)
#[test]
fn k7_case_18_no_hit_no_response_in_guard_or_hunt() {
    for b in [Behaviour::Guard, Behaviour::Hunt] {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        computer(&mut w, ALPHA, 2, 1);
        let c = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
        let a = put(&mut w, Kind::Infantry, "WALKER", BETA, 1, 0);
        w.obj_mut(a).unwrap().beh.current = Some(Behaviour::Sticky);
        let t = w.tick;
        w.obj_mut(c).unwrap().beh.force(b, t, true);
        for _ in 0..600 {
            glasnost_sim::tick::step(&mut w);
            assert_ne!(
                w.obj(c).unwrap().dest,
                Some(Target::Object(a)),
                "{b:?}: no destination naming the idle infantry"
            );
        }
        assert!(w.obj(a).is_some(), "{b:?}: not crushed");
    }
}

// spec: S-0215 claim 39 (a hit that names no attacker never gives the response; S-0302 claim 1)
#[test]
fn a_hit_naming_no_attacker_gives_no_response() {
    let (mut w, c, a) = scene();
    let mut h = hit_from(&w, a);
    h.attacker = None;
    damage::apply(&mut w, c, h);
    // The same hit sends the computer-owned crusher the scatter from damage (S-0215 claim 19
    // case 2: no known attacker, guard with the scatter property), whose cell destination
    // queues nothing; the crush response would name the infantry and queue move.
    assert!(
        !matches!(w.obj(c).unwrap().dest, Some(Target::Object(_))),
        "no destination naming an object"
    );
    assert!(!responded(&w, c, a));
    assert_eq!(w.obj(c).unwrap().beh.queued, None);
}

// spec: S-0215 claim 39 (an unaffected result gives the response too: here a hit of amount 0, stopped at S-0302 claim 8's stage 4)
#[test]
fn an_unaffected_hit_gives_the_response() {
    let (mut w, c, a) = scene();
    let mut h = hit_from(&w, a);
    h.amount = 0;
    let r = damage::apply(&mut w, c, h);
    assert_eq!(r, damage::HitResult::Unaffected);
    assert!(responded(&w, c, a));
}

// ---------------------------------------------------------------------------
// Range (K7 cases 2, 8, 15 and 16)

// spec: S-0210 claim 11 and F2 (the centre distance in three dimensions, by the approximate root of S-0208 F8, at most the crush range: the default 384 leptons; about 1.4 cells and exactly the range respond; 2 leptons over the range, about 1.6 cells and two cells do not, the approximate root being at most one below the exact root; scenario K7 cases 2 and 16)
#[test]
fn k7_cases_2_8_16_the_default_range() {
    for (cell, dx, expect) in [
        (1, 358, true),
        (2, RANGE, true),
        (2, RANGE + 2, false),
        (2, 410, false),
        (2, 512, false),
    ] {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        computer(&mut w, ALPHA, 2, 1);
        let c = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
        let a = put(&mut w, Kind::Infantry, "WALKER", BETA, cell, 0);
        east_of(&mut w, a, c, dx);
        hit(&mut w, c, a);
        assert_eq!(responded(&w, c, a), expect, "{dx} leptons");
    }
}

// spec: S-0210 claim 11 and F2 (scenario K7 case 8: an imported range of 2.0 cells is 512 leptons, and an infantry unit two cells away along a map axis is at exactly that distance, the approximate root being exact there, and so responds)
#[test]
fn k7_case_8_exactly_at_the_imported_range() {
    let mut r = rules();
    r.sim.general.crush_range = Some(d("2.0"));
    let mut w = testkit::game_with(&r, SIZE, 1);
    assert_eq!(w.rules().general.crush_range, 512);
    computer(&mut w, ALPHA, 2, 1);
    let c = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
    let a = put(&mut w, Kind::Infantry, "WALKER", BETA, 2, 0);
    hit(&mut w, c, a);
    assert!(responded(&w, c, a));
}

// spec: S-0210 claim 11 and F2 (the range in force: an imported 2.5 cells is 640 leptons; 0.9 of it responds, 1.1 does not; scenario K7 case 15's two placements)
#[test]
fn k7_case_15_the_imported_range() {
    let mut r = rules();
    r.sim.general.crush_range = Some(d("2.5"));
    for (cell, dx, expect) in [(2, 576, true), (3, 704, false)] {
        let mut w = testkit::game_with(&r, SIZE, 1);
        computer(&mut w, ALPHA, 2, 1);
        let c = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
        let a = put(&mut w, Kind::Infantry, "WALKER", BETA, cell, 0);
        east_of(&mut w, a, c, dx);
        hit(&mut w, c, a);
        assert_eq!(responded(&w, c, a), expect, "{dx} leptons");
    }
}

// ---------------------------------------------------------------------------
// The owner (K7 cases 1, 3, 4, 13, 14 and 17; the switch)

// spec: S-0210 claim 11 (the computer-controlled row: skill level at least the auto-crush threshold, absent 2, and difficulty index not 2; scenario K7 case 4 for index 2)
#[test]
fn k7_cases_1_and_4_the_computer_row() {
    for (skill, difficulty, expect) in [(2, 0, true), (5, 1, true), (1, 0, false), (2, 2, false)] {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        computer(&mut w, ALPHA, skill, difficulty);
        let c = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
        let a = put(&mut w, Kind::Infantry, "WALKER", BETA, 1, 0);
        hit(&mut w, c, a);
        assert_eq!(
            responded(&w, c, a),
            expect,
            "skill {skill}, difficulty {difficulty}"
        );
    }
}

// spec: S-0210 claim 11 and Import mapping ([IQ] AutoCrush: the threshold; absent 2) and Constants
#[test]
fn the_imported_threshold() {
    let mut r = rules();
    assert_eq!(
        glasnost_sim::rules::SimRules::new(&r, "t")
            .general
            .iq_auto_crush,
        2
    );
    r.sim.general.iq_auto_crush = Some(4);
    let mut w = testkit::game_with(&r, SIZE, 1);
    assert_eq!(w.rules().general.iq_auto_crush, 4);
    computer(&mut w, ALPHA, 3, 1);
    let c = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
    let a = put(&mut w, Kind::Infantry, "WALKER", BETA, 1, 0);
    hit(&mut w, c, a);
    assert!(!responded(&w, c, a), "skill 3 is below 4");
}

/// A player-house (human-controlled) crusher hit from close by, with the
/// switch, the auto-crush flag and the player-auto-crush rule as given.
fn player_responds(switch_on: bool, flag: bool, rule: bool) -> bool {
    let mut r = rules();
    r.sim.vehicles.get_mut("CRUSHER").unwrap().auto_crush = Some(flag);
    r.sim.general.player_auto_crush = Some(rule);
    r.globals.quirks.auto_crush_computer_only = switch_on;
    let mut w = testkit::game_with(&r, SIZE, 1);
    assert!(w.houses[usize::from(ALPHA)].human, "a player house");
    let c = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
    let a = put(&mut w, Kind::Infantry, "WALKER", BETA, 1, 0);
    hit(&mut w, c, a);
    responded(&w, c, a)
}

// spec: S-0210 claim 11 and Edge cases (quirks.auto_crush_computer_only on, the default and the original: a player house's crusher never makes the crush response, with the flag and the rule off, the flag on or the rule on; scenario K7 cases 3, 13 and 14) and S-0215 claim 39 (in our engine every player house counts as human-controlled)
#[test]
fn k7_cases_3_13_14_switch_on_no_response_for_a_player_house() {
    assert!(
        glasnost_rules::quirks::Quirks::default().auto_crush_computer_only,
        "default on"
    );
    assert!(!player_responds(true, false, false), "case 3");
    assert!(!player_responds(true, true, false), "case 13");
    assert!(!player_responds(true, false, true), "case 14");
}

// spec: S-0210 claim 11 (our engine's rule for player houses with quirks.auto_crush_computer_only off: the response when the player-auto-crush rule is on or the type has the auto-crush flag; K7's expected engine results for cases 3, 13 and 14) and S-0215 claim 39
#[test]
fn k7_cases_3_13_14_switch_off_the_flag_or_the_rule() {
    assert!(!player_responds(false, false, false), "case 3");
    assert!(player_responds(false, true, false), "case 13");
    assert!(player_responds(false, false, true), "case 14");
}

// spec: S-0210 claim 11 (a test-only computer-controlled player house keeps the computer-controlled row whatever the switch: the flag and the rule play no part for it)
#[test]
fn a_test_only_computer_house_keeps_the_computer_row_whatever_the_switch() {
    for switch_on in [true, false] {
        for (skill, expect) in [(2, true), (0, false)] {
            let mut r = rules();
            r.sim.vehicles.get_mut("CRUSHER").unwrap().auto_crush = Some(true);
            r.sim.general.player_auto_crush = Some(true);
            r.globals.quirks.auto_crush_computer_only = switch_on;
            let mut w = testkit::game_with(&r, SIZE, 1);
            computer(&mut w, ALPHA, skill, 1);
            let c = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
            let a = put(&mut w, Kind::Infantry, "WALKER", BETA, 1, 0);
            hit(&mut w, c, a);
            assert_eq!(
                responded(&w, c, a),
                expect,
                "switch {switch_on}, skill {skill}"
            );
        }
    }
}

// spec: S-0210 claim 11 (the neutral and special houses: difficulty index 1 and skill level 0, so they fail the row with threshold 2 and pass it with threshold 0; scenario K7 case 17)
#[test]
fn k7_case_17_the_neutral_and_special_houses() {
    for house in [NEUTRAL, SPECIAL] {
        for (threshold, expect) in [(2, false), (0, true)] {
            let mut r = rules();
            r.sim.general.iq_auto_crush = Some(threshold);
            let mut w = testkit::game_with_non_players(&r, SIZE, 1);
            assert!(!w.houses[usize::from(house)].human);
            assert_eq!(w.houses[usize::from(house)].skill, 0);
            let c = put(&mut w, Kind::Vehicle, "CRUSHER", house, 0, 0);
            let a = put(&mut w, Kind::Infantry, "WALKER", BETA, 1, 0);
            assert!(
                !w.allied(Some(HouseId(house)), Some(HouseId(BETA))),
                "hostile"
            );
            hit(&mut w, c, a);
            assert_eq!(
                responded(&w, c, a),
                expect,
                "house {house}, threshold {threshold}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The attacker (crushable, alliance, disguise; K13 part a)

// spec: S-0210 claim 11 (the attacker is crushable by the crusher, claim 2: an uncrushable vehicle gives no response, a crushable one does)
#[test]
fn only_a_crushable_attacker_gives_the_response() {
    for (name, expect) in [("CART", false), ("BUGGY", true)] {
        let mut w = testkit::game_with(&rules(), SIZE, 1);
        computer(&mut w, ALPHA, 2, 1);
        let c = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
        let a = put(&mut w, Kind::Vehicle, name, BETA, 1, 0);
        hit(&mut w, c, a);
        assert_eq!(responded(&w, c, a), expect, "{name}");
    }
}

// spec: S-0210 claim 1 (only a crusher makes the response: a vehicle without the crusher flag does not)
#[test]
fn only_a_crusher_makes_the_response() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    computer(&mut w, ALPHA, 2, 1);
    let c = put(&mut w, Kind::Vehicle, "CART", ALPHA, 0, 0);
    let a = put(&mut w, Kind::Infantry, "WALKER", BETA, 1, 0);
    hit(&mut w, c, a);
    assert!(!responded(&w, c, a));
}

// spec: S-0215 claim 39 (its owner is not allied with the attacker's owner; a house counts as allied with itself, so its own house's unit never gets the response)
#[test]
fn an_allied_or_own_attacker_gives_no_response() {
    let (mut w, c, _) = scene();
    let own = put(&mut w, Kind::Infantry, "WALKER", ALPHA, 0, 1);
    hit(&mut w, c, own);
    assert!(!responded(&w, c, own), "its own house");
    let (mut w, c, a) = scene();
    w.houses[usize::from(ALPHA)].allies.push(HouseId(BETA));
    w.houses[usize::from(ALPHA)].allies.sort();
    hit(&mut w, c, a);
    assert!(!responded(&w, c, a), "an ally");
}

// spec: S-0210 claim 11 and scenario K13 part a (an attacker that appears disguised to the crusher's house gives no response; with that house's disguise-detector coverage on its cell it does) and S-0516 claim 35
#[test]
fn k13_a_disguise_blocks_the_response_and_detection_lifts_it() {
    for detector in [false, true] {
        let (mut w, c, a) = scene();
        let walker = w.rules().index_of(Kind::Infantry, "WALKER").unwrap();
        let t = w.tick;
        w.obj_mut(a).unwrap().cloak.disguise = Some(Disguise {
            what: Some(DisguiseAs::Infantry(walker)),
            house: Some(HouseId(ALPHA)),
            since: t,
        });
        w.obj_mut(a).unwrap().beh.current = Some(Behaviour::Sticky);
        if detector {
            put(&mut w, Kind::Building, "EYE", ALPHA, 1, 2);
        }
        // The detector coverage is kept by the tick (S-0516 claim 35).
        testkit::run(&mut w, 2);
        let o = w.obj_mut(c).unwrap();
        o.dest = None;
        o.beh.queued = None;
        assert_eq!(
            glasnost_sim::cloak::appears_disguised(&w, a, Some(HouseId(ALPHA))),
            !detector
        );
        hit(&mut w, c, a);
        assert_eq!(responded(&w, c, a), detector, "detector {detector}");
    }
}

// ---------------------------------------------------------------------------
// The crusher's state (busy, tethered, refused destination, later hits)

// spec: S-0215 claim 39 (the response is made in every current behaviour, whether or not the vehicle holds a target or a destination: the destination is replaced, the target kept)
#[test]
fn a_busy_crusher_still_responds_and_keeps_its_target() {
    let (mut w, c, a) = scene();
    let other = put(&mut w, Kind::Vehicle, "CART", BETA, 6, 6);
    let t = w.tick;
    let o = w.obj_mut(c).unwrap();
    o.beh.force(Behaviour::Attack, t, true);
    o.target = Some(Target::Object(other));
    o.dest = Some(Target::Cell(at(-5, 0)));
    hit(&mut w, c, a);
    let o = w.obj(c).unwrap();
    assert!(responded(&w, c, a));
    assert_eq!(o.target, Some(Target::Object(other)), "target kept");
}

// spec: S-0215 claim 39 (a tethered vehicle makes no response; glossary *tether link*, here a factory release's)
#[test]
fn a_tethered_crusher_makes_no_response() {
    let (mut w, c, a) = scene();
    let k = put(&mut w, Kind::Building, "EYE", ALPHA, -3, 0);
    let o = w.obj_mut(c).unwrap();
    o.econ.exit = Some(glasnost_sim::econ::release::Exit {
        factory: k,
        tethered: true,
        place: 0,
    });
    o.contact = Some(k);
    assert!(glasnost_sim::econ::tethered(&w, c));
    hit(&mut w, c, a);
    assert_eq!(w.obj(c).unwrap().dest, None);
    assert_eq!(w.obj(c).unwrap().beh.queued, None);
}

// spec: S-0215 claim 39 (the destination is given under the rules of every new destination: a seized vehicle ignores it, S-0607 claim 9; move is queued only when the destination is accepted)
#[test]
fn a_seized_crusher_ignores_the_destination_and_queues_nothing() {
    let (mut w, c, a) = scene();
    w.obj_mut(c).unwrap().lift.seized = true;
    hit(&mut w, c, a);
    assert_eq!(w.obj(c).unwrap().dest, None);
    assert_eq!(w.obj(c).unwrap().beh.queued, None);
}

// spec: S-0215 claim 39 (a later qualifying hit replaces the destination with the newer attacker; several qualifying hits in one tick each respond in the order applied)
#[test]
fn a_later_hit_replaces_the_destination() {
    let (mut w, c, a) = scene();
    let b = put(&mut w, Kind::Infantry, "WALKER", BETA, 0, 1);
    hit(&mut w, c, a);
    assert!(responded(&w, c, a));
    hit(&mut w, c, b);
    assert!(responded(&w, c, b), "the newer attacker");
}

// spec: S-0215 claim 39 (an ordinary hit first makes the vehicle retaliate, claim 18: it holds the attacker as its target under an attack override, and the response's move is queued behind it)
#[test]
fn retaliation_comes_first_and_move_is_queued_behind_it() {
    let mut r = rules();
    r.sim.vehicles.get_mut("CRUSHER").unwrap().primary = Some("Gun".into());
    let mut w = testkit::game_with(&r, SIZE, 1);
    computer(&mut w, ALPHA, 2, 1);
    let c = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
    let a = put(&mut w, Kind::Infantry, "WALKER", BETA, 1, 0);
    w.obj_mut(a).unwrap().beh.current = Some(Behaviour::Sticky);
    testkit::run(&mut w, 2);
    assert_eq!(w.obj(c).unwrap().beh.current, Some(Behaviour::Guard));
    w.obj_mut(c).unwrap().target = None;
    hit(&mut w, c, a);
    let o = w.obj(c).unwrap();
    assert_eq!(o.target, Some(Target::Object(a)), "retaliation");
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Attack),
        "the attack override"
    );
    assert!(responded(&w, c, a), "then the response");
}

// ---------------------------------------------------------------------------
// A berserk crusher (K15)

/// A computer-owned crusher (difficulty index `difficulty`) at (0, 0) and
/// a crushable gas firer BUGGY of `firer_house` at (`dx`, 0) that sets off
/// the berserk area warhead at its own position, naming itself as the
/// attacker (S-0302 claim 1).
fn gassed(dx: i32, firer_house: u8, difficulty: i32) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    computer(&mut w, ALPHA, 2, difficulty);
    let c = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
    let g = put(&mut w, Kind::Vehicle, "BUGGY", firer_house, dx, 0);
    w.obj_mut(c).unwrap().beh.queued = None;
    let mad = warhead(&w, "MAD");
    let point = w.obj(g).unwrap().pos;
    let house = w.obj(g).unwrap().owner;
    area::detonate(
        &mut w,
        Detonation {
            point,
            damage: 100,
            warhead: Some(mad),
            source: Some(g),
            house,
            resources: true,
        },
    );
    (w, c, g)
}

// spec: S-0215 claim 39 (order within the hit: the berserk entry drops the target and queues hunt, S-0610 claim 4, then the response queues move in its place; the destination names the gas firer, the target is not it) and S-0210 claim 11 (berserk play no part; scenario K15 part a); S-0210 claims 3 and 4 (it then drives over the firer, which leaves play with the crush sound)
#[test]
fn k15_a_the_berserk_crusher_drives_over_the_gas_firer() {
    let (mut w, c, g) = gassed(1, BETA, 1);
    assert!(psychic::berserk(&w, c), "berserk");
    let o = w.obj(c).unwrap();
    assert_eq!(o.beh.queued, Some(Behaviour::Move), "move in place of hunt");
    assert_eq!(o.dest, Some(Target::Object(g)), "the firer object");
    assert_ne!(o.target, Some(Target::Object(g)), "not its target");
    let mut crushed = false;
    for _ in 0..300 {
        glasnost_sim::tick::step(&mut w);
        crushed |= std::mem::take(&mut w.effects).iter().any(|e| {
            matches!(
                e,
                Effect::Sound {
                    sound: Sound::Crush,
                    ..
                }
            )
        });
        if w.obj(g).is_none() {
            break;
        }
    }
    assert!(w.obj(g).is_none(), "the firer left play");
    assert!(crushed, "crushed");
}

// spec: S-0215 claim 39 and S-0210 claim 11 (scenario K15 part b: beyond the crush range the crusher goes berserk, hunt stays queued and the hit gives no destination)
#[test]
fn k15_b_beyond_the_range() {
    let (w, c, g) = gassed(3, BETA, 1);
    assert!(psychic::berserk(&w, c));
    let o = w.obj(c).unwrap();
    assert_eq!(o.beh.queued, Some(Behaviour::Hunt));
    assert_ne!(o.dest, Some(Target::Object(g)));
}

// spec: S-0215 claim 39 (scenario K15 part c: the crusher's own house's firer: no berserk, S-0302 claim 10, and no response, a house being allied with itself)
#[test]
fn k15_c_its_own_house() {
    let (w, c, g) = gassed(1, ALPHA, 1);
    assert!(!psychic::berserk(&w, c));
    assert_ne!(w.obj(c).unwrap().dest, Some(Target::Object(g)));
}

// spec: S-0215 claim 39 and S-0210 claim 11 (scenario K15 part d: difficulty index 2: berserk, hunt queued, not move, and no destination from the hit)
#[test]
fn k15_d_difficulty_index_2() {
    let (w, c, g) = gassed(1, BETA, 2);
    assert!(psychic::berserk(&w, c));
    let o = w.obj(c).unwrap();
    assert_eq!(o.beh.queued, Some(Behaviour::Hunt));
    assert_ne!(o.dest, Some(Target::Object(g)));
}

// spec: S-0215 claim 39 (a berserk hit on a vehicle that is already berserk queues nothing of its own, and the response queues move in the same way)
#[test]
fn an_already_berserk_crusher_responds_to_a_further_berserk_hit() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    computer(&mut w, ALPHA, 2, 1);
    let c = put(&mut w, Kind::Vehicle, "CRUSHER", ALPHA, 0, 0);
    let g = put(&mut w, Kind::Vehicle, "BUGGY", BETA, 1, 0);
    psychic::enter(&mut w, c, 10_000);
    w.obj_mut(c).unwrap().beh.queued = None;
    let h = Hit {
        amount: 100,
        warhead: Some(warhead(&w, "MAD")),
        distance: 0,
        attacker: Some(g),
        house: Some(HouseId(BETA)),
        ignore_defences: false,
        no_escape: false,
    };
    damage::apply(&mut w, c, h);
    assert!(responded(&w, c, g));
}
