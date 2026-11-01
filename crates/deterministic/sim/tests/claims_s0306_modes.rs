//! Claim tests for the firing check's water and land rows (S-0306 claim 23,
//! revision 33) and their consequences for retaliation (S-0309 claim 18,
//! revision 38), the order cursor (S-0214 claim 6 (e)) and a building's
//! *attack* (S-0215 claim 32), on synthetic data (M8 step A3 intake 5,
//! answering Q-1761 and Q-1763).
//!
//! Every type, weapon, warhead and number here is invented. The ledger is
//! `docs/m8-claims/intake5-S-0306-S-0309-S-0107.md`.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    ArmourClass, BuildingType, Decimal, LocomotionClass, Locomotor, ProjectileType, UnitType,
    WarheadType, WeaponType,
};
use glasnost_sim::cloak::CloakState;
use glasnost_sim::combat::acquire;
use glasnost_sim::combat::check::{Answer, check};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Target, World};

const SIZE: i32 = 30;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn weapon(projectile: &str) -> WeaponType {
    WeaponType {
        damage: Some(10),
        rof: Some(20),
        range: Some(d("5")),
        speed: Some(100),
        projectile: Some(projectile.into()),
        warhead: Some("Even".into()),
        ..Default::default()
    }
}

fn vehicle(primary: Option<&str>) -> UnitType {
    let mut t = UnitType {
        strength: Some(200),
        speed: Some(5),
        rot: Some(5),
        sight: Some(9),
        primary: primary.map(Into::into),
        locomotion_class: Some(LocomotionClass::Wheel),
        locomotor: Some(Locomotor::Drive),
        ..Default::default()
    };
    t.combat.armor = Some(ArmourClass::Light);
    t
}

/// The testkit's combat ruleset plus
/// - projectiles `Sky` (anti-air, not anti-ground) and `Ground` (anti-ground,
///   not anti-air), warhead `Even` (every versus value 1),
/// - weapons `Flak` (projectile `Sky`) and `Plain` (projectile `Ground`),
///   both of range 5 cells, and `Long` (`Plain` with range 9 cells),
/// - vehicles `AAV` (primary `Flak`), `AGV` (primary `Plain`), `TGT` (an
///   unarmed target), `SUB` (unarmed, underwater), `FAR` (primary `Long`),
/// - buildings `AAB` (primary `Flak`, sight 9) and `SAMB` (`AAB` with the
///   SAM flag).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    s.projectiles.insert(
        "Sky".into(),
        ProjectileType {
            aa: Some(true),
            ag: Some(false),
            inviso: Some(true),
            ..Default::default()
        },
    );
    s.projectiles.insert(
        "Ground".into(),
        ProjectileType {
            aa: Some(false),
            ag: Some(true),
            inviso: Some(true),
            ..Default::default()
        },
    );
    s.warheads.insert(
        "Even".into(),
        WarheadType {
            verses: Some(vec![d("1"); 11]),
            ..Default::default()
        },
    );
    s.weapons.insert("Flak".into(), weapon("Sky"));
    s.weapons.insert("Plain".into(), weapon("Ground"));
    s.weapons.insert(
        "Long".into(),
        WeaponType {
            range: Some(d("9")),
            ..weapon("Ground")
        },
    );
    let mut sub = vehicle(None);
    sub.combat.underwater = Some(true);
    for (name, t) in [
        ("AAV", vehicle(Some("Flak"))),
        ("AGV", vehicle(Some("Plain"))),
        ("TGT", vehicle(None)),
        ("SUB", sub),
        ("FAR", vehicle(Some("Long"))),
    ] {
        s.vehicles.insert(name.into(), t);
        r.registries
            .get_mut("vehicles")
            .expect("vehicles registry")
            .push(name.into());
    }
    let mut aab = BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        primary: Some("Flak".into()),
        ..Default::default()
    };
    aab.combat.armor = Some(ArmourClass::Light);
    aab.econ.sight = Some(9);
    let samb = BuildingType {
        sam: Some(true),
        ..aab.clone()
    };
    for (name, b) in [("AAB", aab), ("SAMB", samb)] {
        s.buildings.insert(name.into(), b);
        r.registries
            .get_mut("buildings")
            .expect("buildings registry")
            .push(name.into());
    }
    r
}

/// The rules with `ty`'s land-targeting and naval-targeting modes set.
fn modes(ty: &str, land_mode: i32, naval_mode: i32) -> Ruleset {
    let mut r = rules();
    let c = match r.sim.vehicles.get_mut(ty) {
        Some(v) => &mut v.combat,
        None => &mut r.sim.buildings.get_mut(ty).expect("type").combat,
    };
    c.land_targeting = Some(land_mode);
    c.naval_targeting = Some(naval_mode);
    r
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 13)
}

fn put(w: &mut World, kind: Kind, ty: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, ty, house, at(dx, dy), 0)
}

fn set_land(w: &mut World, c: CellXY, l: u8) {
    w.cells.grid.get_mut(c).expect("legal cell").land = l;
}

/// The firing check of `f`'s primary against `t`, without the range test.
fn answer(w: &World, f: ObjectId, t: Target) -> Answer {
    check(w, f, Some(t), 0, false)
}

fn obj(x: ObjectId) -> Target {
    Target::Object(x)
}

/// A firer of type `ty` (land mode `lm`, naval mode `nm`) at the centre and
/// an enemy `target` vehicle two cells east, on a cell of land type `l`.
fn duel(ty: &str, lm: i32, nm: i32, target: &str, l: u8) -> (World, ObjectId, ObjectId) {
    let mut w = game(&modes(ty, lm, nm));
    let f = put(&mut w, Kind::Vehicle, ty, 0, 0, 0);
    let t = put(&mut w, Kind::Vehicle, target, 1, 2, 0);
    set_land(&mut w, at(2, 0), l);
    (w, f, t)
}

fn lift(w: &mut World, x: ObjectId, dz: i32) {
    let p = w.obj(x).unwrap().pos;
    let ground = w.surface_at(p.x, p.y);
    w.obj_mut(x).unwrap().pos.z = ground + dz;
}

// ---------------------------------------------------------------------------
// The land row

// spec: S-0306 claim 23 (land row: a firer whose type has land-targeting mode 1 is refused a floor-bound infantry, vehicle or building on a cell that is neither water nor beach, with any weapon; modes 0 and 2, and any other value, allow it; the anti-ground flag is not tested for object targets; scenario P16 parts f (1) and h)
#[test]
fn claim23_land_row_refuses_ground_objects_by_the_firers_type() {
    for lm in [0, 1, 2, 3, -1] {
        for ty in ["AAV", "AGV"] {
            let mut w = game(&modes(ty, lm, 0));
            let f = put(&mut w, Kind::Vehicle, ty, 0, 0, 0);
            let v = put(&mut w, Kind::Vehicle, "TGT", 1, 2, 0);
            let i = put(&mut w, Kind::Infantry, "WALKER", 1, 0, 2);
            let b = put(&mut w, Kind::Building, "TOWER", 1, -3, 0);
            for (what, x) in [("vehicle", v), ("infantry", i), ("building", b)] {
                let a = answer(&w, f, obj(x));
                assert_eq!(
                    a == Answer::Illegal,
                    lm == 1,
                    "{ty}, land mode {lm}, {what}: {a:?}"
                );
            }
        }
    }
}

// spec: S-0306 claim 23 (land row: a floor-bound object on a bridge is refused to mode 1 whatever the cell below; an object on a water or beach cell, not on a bridge, is not covered by the land row; an airborne object meets neither row)
#[test]
fn claim23_land_row_bridges_water_and_airborne_objects() {
    for l in [land::WATER, land::BEACH] {
        let (mut w, f, t) = duel("AAV", 1, 0, "TGT", l);
        assert_ne!(answer(&w, f, obj(t)), Answer::Illegal, "on land type {l}");
        w.obj_mut(t).unwrap().layer = Layer::Deck;
        assert_eq!(
            answer(&w, f, obj(t)),
            Answer::Illegal,
            "on a bridge over {l}"
        );
    }
    // Airborne over clear land: neither row; the anti-air projectile may fire.
    let (mut w, f, t) = duel("AAV", 1, 6, "TGT", land::CLEAR);
    lift(&mut w, t, 400);
    assert_ne!(answer(&w, f, obj(t)), Answer::Illegal);
    // Airborne over water with naval mode 6: the water row needs a target that is not airborne.
    set_land(&mut w, at(2, 0), land::WATER);
    assert_ne!(answer(&w, f, obj(t)), Answer::Illegal);
}

// spec: S-0306 claim 23 (land row for a cell: mode 1 refuses a cell whose land type is neither water nor beach; the third row still refuses any cell to a projectile that is not anti-ground; scenario P16 part f (4))
#[test]
fn claim23_land_row_refuses_land_cells() {
    for lm in [0, 1, 2] {
        let mut w = game(&modes("AGV", lm, 0));
        let f = put(&mut w, Kind::Vehicle, "AGV", 0, 0, 0);
        set_land(&mut w, at(2, 1), land::WATER);
        set_land(&mut w, at(2, -1), land::BEACH);
        let cell = |c| answer(&w, f, Target::Cell(c));
        assert_eq!(
            cell(at(2, 0)) == Answer::Illegal,
            lm == 1,
            "clear, mode {lm}"
        );
        assert_ne!(cell(at(2, 1)), Answer::Illegal, "water, mode {lm}");
        assert_ne!(cell(at(2, -1)), Answer::Illegal, "beach, mode {lm}");
    }
    // The anti-air-only weapon is refused every cell (third row), whatever the mode.
    let mut w = game(&modes("AAV", 0, 0));
    let f = put(&mut w, Kind::Vehicle, "AAV", 0, 0, 0);
    set_land(&mut w, at(2, 1), land::WATER);
    assert_eq!(answer(&w, f, Target::Cell(at(2, 0))), Answer::Illegal);
    assert_eq!(answer(&w, f, Target::Cell(at(2, 1))), Answer::Illegal);
}

// ---------------------------------------------------------------------------
// The water row

// spec: S-0306 claim 23 (water row: an object on a water or beach cell, not airborne and not on a bridge, is refused when the firer's naval-targeting mode gives "none" by S-0308 table N: mode 2 against a type that is not underwater, mode 6 against every target, mode 0 against an underwater target that is cloaking, cloaked or uncloaking; the row does not depend on the weapon; scenario P16 part f (3))
#[test]
fn claim23_water_row_by_naval_mode() {
    for ty in ["AAV", "AGV"] {
        for l in [land::WATER, land::BEACH] {
            for nm in 0..=7 {
                for target in ["TGT", "SUB"] {
                    let (w, f, t) = duel(ty, 0, nm, target, l);
                    let refused = match nm {
                        2 => target == "TGT",
                        6 => true,
                        _ => false,
                    };
                    assert_eq!(
                        answer(&w, f, obj(t)) == Answer::Illegal,
                        refused,
                        "{ty}, land type {l}, naval mode {nm}, {target}"
                    );
                }
            }
        }
    }
    // Mode 0: an underwater target that is cloaking, cloaked or uncloaking.
    for (state, refused) in [
        (CloakState::Uncloaked, false),
        (CloakState::Cloaking, true),
        (CloakState::Cloaked, true),
        (CloakState::Uncloaking, true),
    ] {
        let (mut w, f, t) = duel("AAV", 0, 0, "SUB", land::WATER);
        w.obj_mut(t).unwrap().cloak.state = state;
        let a = answer(&w, f, obj(t));
        if state == CloakState::Cloaked {
            // A fully cloaked target is hidden from the firer's house: the hidden-target impossible comes first (rank 4).
            assert!(glasnost_sim::cloak::hidden_from(&w, t, Some(HouseId(0))));
            assert_eq!(a, Answer::Impossible, "{state:?}");
            continue;
        }
        assert_eq!(a == Answer::Illegal, refused, "{state:?}: {a:?}");
    }
    // A bridge over water: not the water row.
    let (mut w, f, t) = duel("AAV", 0, 6, "TGT", land::WATER);
    w.obj_mut(t).unwrap().layer = Layer::Deck;
    assert_ne!(answer(&w, f, obj(t)), Answer::Illegal);
}

// spec: S-0306 claim 23 (mode 1 against an object on water or beach that its naval mode does not refuse: no row refuses it)
#[test]
fn claim23_land_mode_one_on_water_without_a_naval_refusal() {
    let (w, f, t) = duel("AAV", 1, 0, "TGT", land::WATER);
    assert_ne!(answer(&w, f, obj(t)), Answer::Illegal);
    let (w, f, t) = duel("AAV", 1, 2, "TGT", land::WATER);
    assert_eq!(answer(&w, f, obj(t)), Answer::Illegal);
}

// ---------------------------------------------------------------------------
// Rank

// spec: S-0306 claim 23 (rank of the water and land rows: rank 4 of S-0301 claim 9, ahead of rearming (rank 5), out of ammunition (rank 7) and out of range (rank 10))
#[test]
fn claim23_rows_outrank_rearming_ammunition_and_range() {
    let (mut w, f, t) = duel("AAV", 1, 0, "TGT", land::CLEAR);
    let far = put(&mut w, Kind::Vehicle, "TGT", 1, 12, 0);
    assert_eq!(check(&w, f, Some(obj(far)), 0, true), Answer::Illegal);
    let tick = w.tick;
    w.obj_mut(f).unwrap().combat.rearm_until = tick + 50;
    assert_eq!(answer(&w, f, obj(t)), Answer::Illegal, "rearming");
    w.obj_mut(f).unwrap().combat.ammo = 0;
    assert_eq!(answer(&w, f, obj(t)), Answer::Illegal, "out of ammunition");
    // The control: the same states with mode 0.
    let (mut w, f, t) = duel("AAV", 0, 0, "TGT", land::CLEAR);
    let far = put(&mut w, Kind::Vehicle, "TGT", 1, 12, 0);
    assert_eq!(check(&w, f, Some(obj(far)), 0, true), Answer::OutOfRange);
    let tick = w.tick;
    w.obj_mut(f).unwrap().combat.rearm_until = tick + 50;
    assert_eq!(answer(&w, f, obj(t)), Answer::Rearming);
}

// spec: S-0306 claim 23, The firer's gripped unit (no row refuses the firer's own gripped unit: every row yields to the rearming answer of S-0301 claim 9 and S-0607 claim 11; any other airborne object asked about with a projectile that is not anti-air is illegal, one the firer holds as its target included)
#[test]
fn claim23_the_firers_gripped_unit_is_never_refused_by_a_row() {
    // The land row (mode 1) and the airborne row, each against the control of the same target not gripped.
    for (lm, airborne) in [(1, false), (0, true)] {
        let (mut w, f, t) = duel("AGV", lm, 0, "TGT", land::CLEAR);
        if airborne {
            lift(&mut w, t, 600);
        }
        assert_eq!(
            answer(&w, f, obj(t)),
            Answer::Illegal,
            "control {lm} {airborne}"
        );
        w.obj_mut(f).unwrap().lift.gripped = Some(t);
        w.obj_mut(t).unwrap().lift.lifter = Some(f);
        assert_eq!(
            answer(&w, f, obj(t)),
            Answer::Rearming,
            "gripped {lm} {airborne}"
        );
        // Another object of the same kind is still refused.
        let other = put(&mut w, Kind::Vehicle, "TGT", 1, 3, 0);
        if airborne {
            lift(&mut w, other, 600);
        }
        assert_eq!(
            answer(&w, f, obj(other)),
            Answer::Illegal,
            "other {lm} {airborne}"
        );
    }
}

// spec: S-0306 claim 23 (within rank 4 the rows come after the impossible answer of an absent weapon and of a hidden target)
#[test]
fn claim23_rows_yield_to_earlier_rank_four_answers() {
    let (mut w, f, t) = duel("AAV", 1, 6, "TGT", land::CLEAR);
    // Slot 1 holds no weapon: impossible.
    assert_eq!(check(&w, f, Some(obj(t)), 1, false), Answer::Impossible);
    // A cloaked target hidden from the firer's house, with mean weapon damage above 0: impossible.
    w.obj_mut(t).unwrap().cloak.state = CloakState::Cloaked;
    assert!(glasnost_sim::cloak::hidden_from(&w, t, Some(HouseId(0))));
    assert_eq!(answer(&w, f, obj(t)), Answer::Impossible);
}

// ---------------------------------------------------------------------------
// Retaliation (S-0309 claim 18)

fn retaliated(w: &World, a: ObjectId, x: ObjectId) -> bool {
    let o = w.obj(a).unwrap();
    o.target == Some(obj(x))
        && (o.beh.current == Some(Behaviour::Attack) || o.beh.queued == Some(Behaviour::Attack))
}

// spec: S-0309 claim 18 (first case: an object whose type has land-targeting mode 1 never fires back at a floor-bound attacker on land, whatever its rearm countdown; third case: an anti-air-only object with mode 0 or 2 retaliates against a ground attacker; the same for a human-controlled and a computer-controlled owner; S-0306 claim 23; scenario A5 part b, S-0306 P16 part g)
#[test]
fn claim18_land_row_stops_retaliation_and_mode_zero_fires_back() {
    for computer in [false, true] {
        for (lm, fires) in [(0, true), (1, false), (2, true)] {
            for rearming in [false, true] {
                let (mut w, a, x) = duel("AAV", lm, 0, "AGV", land::CLEAR);
                w.houses[0].human = !computer;
                if rearming {
                    let tick = w.tick;
                    w.obj_mut(a).unwrap().combat.rearm_until = tick + 50;
                }
                acquire::retaliate(&mut w, a, x);
                assert_eq!(
                    retaliated(&w, a, x),
                    fires,
                    "computer {computer}, land mode {lm}, rearming {rearming}"
                );
            }
        }
    }
}

// spec: S-0306 P16 part g (revision 63: retaliation does not restart the copy's processing timer, so its first processing in attack can wait until the guard processing delay then running ends) and S-0215 claim 10 (neither an override nor a revert restarts the processing timer, claim 3) and S-0309 claim 18 (the override to attack with the attacker as target)
#[test]
fn claim18_retaliation_keeps_the_guard_processing_timer() {
    let (mut w, a, x) = duel("AAV", 0, 0, "AGV", land::CLEAR);
    w.houses[0].human = false;
    // Let the copy's guard processing run, so that its timer holds a delay.
    testkit::run(&mut w, 8);
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Guard), "in guard");
    assert_eq!(o.target, None, "holding no target");
    let timer = (o.beh.last, o.beh.expired);
    assert!(timer.0.is_some(), "a guard processing has run");
    acquire::retaliate(&mut w, a, x);
    assert!(retaliated(&w, a, x), "mode 0 retaliates");
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.suspended, Some(Behaviour::Guard), "an override");
    assert_eq!(
        (o.beh.last, o.beh.expired),
        timer,
        "the processing timer is not restarted"
    );
}

// spec: S-0309 claim 18 (the water row stops retaliation against an attacker on water when the naval mode gives "none"; a mode-1 object retaliates against an attacker on water that its naval mode does not refuse)
#[test]
fn claim18_water_row_and_mode_one_on_water() {
    for (lm, nm, fires) in [(0, 6, false), (0, 2, false), (1, 0, true), (1, 6, false)] {
        let (mut w, a, x) = duel("AAV", lm, nm, "AGV", land::WATER);
        acquire::retaliate(&mut w, a, x);
        assert_eq!(retaliated(&w, a, x), fires, "land {lm}, naval {nm}");
    }
}

/// Runs `n` ticks, counting the shots `f` makes: each shot starts a new
/// rearm countdown (S-0301 claim 1).
fn run_counting(w: &mut World, f: ObjectId, n: i32) -> usize {
    let mut last = w.obj(f).unwrap().combat.rearm_until;
    let mut shots = 0;
    for _ in 0..n {
        testkit::run(w, 1);
        let now = w.obj(f).unwrap().combat.rearm_until;
        if now != last {
            shots += 1;
            last = now;
        }
    }
    shots
}

// spec: S-0309 claim 18 (second case: a building whose type has the SAM flag never fires at a ground attacker; its attack processing drops a target that is not S-0215 claim 32's suitable target) and S-0215 claim 32
#[test]
fn claim18_sam_building_never_fires_at_a_ground_attacker() {
    let mut w = game(&modes("SAMB", 0, 0));
    let b = put(&mut w, Kind::Building, "SAMB", 0, 0, 0);
    let x = put(&mut w, Kind::Vehicle, "AGV", 1, 3, 0);
    acquire::retaliate(&mut w, b, x);
    let shots = run_counting(&mut w, b, 60);
    assert_eq!(shots, 0);
    assert_ne!(
        w.obj(b).unwrap().target,
        Some(obj(x)),
        "the target is dropped"
    );
}

// spec: S-0309 claim 18 (third case: a building whose type has land-targeting mode 0 and no SAM flag, its projectiles anti-air only, fires back at a ground attacker in range; mode 1 does not) and S-0215 claim 32
#[test]
fn claim18_building_third_case_fires_back() {
    for (lm, fires) in [(0, true), (1, false)] {
        let mut w = game(&modes("AAB", lm, 0));
        let b = put(&mut w, Kind::Building, "AAB", 0, 0, 0);
        let x = put(&mut w, Kind::Vehicle, "TGT", 1, 3, 0);
        acquire::retaliate(&mut w, b, x);
        assert_eq!(retaliated(&w, b, x), fires, "land mode {lm}");
        let shots = run_counting(&mut w, b, 60);
        assert_eq!(shots > 0, fires, "land mode {lm}: {shots} shots");
    }
}

// spec: S-0309 claim 18 (Q-1761 question 1: a building of the third case retaliates against an attacker beyond its weapon's range but within its sight limit, S-0215 claim 18 and F7, and drops it without firing at its first processing in attack, which answers out of range; the same for a computer-controlled owner, for which F7's sight limit does not apply; scenario A5 part b Variant 3)
#[test]
fn claim18_third_case_drops_an_attacker_beyond_its_range() {
    for computer in [false, true] {
        let mut w = game(&modes("AAB", 0, 0));
        w.houses[0].human = !computer;
        let b = put(&mut w, Kind::Building, "AAB", 0, 0, 0);
        // 7 cells: beyond the 5-cell weapon range, within the 9-cell sight (F7).
        let x = put(&mut w, Kind::Vehicle, "TGT", 1, 7, 0);
        acquire::retaliate(&mut w, b, x);
        assert!(retaliated(&w, b, x), "computer {computer}");
        let shots = run_counting(&mut w, b, 30);
        assert_eq!(shots, 0, "computer {computer}");
        assert_ne!(w.obj(b).unwrap().target, Some(obj(x)), "dropped");
        // Beyond the sight limit: F7 refuses it for a human-controlled owner only.
        let far = put(&mut w, Kind::Vehicle, "TGT", 1, 0, 11);
        acquire::retaliate(&mut w, b, far);
        assert_eq!(
            retaliated(&w, b, far),
            computer,
            "computer {computer}: beyond sight"
        );
    }
}

// ---------------------------------------------------------------------------
// Other routes (Q-1763 question 3) and the cursor (S-0214 claim 6 (e))

// spec: S-0306 claim 23 (Q-1763 question 3: a target kept from an order passes the firing check on every shot; with mode 1 the land row refuses a ground object and the attack drops it, S-0304 claim 2; with mode 0 an anti-air-only weapon fires at it)
#[test]
fn claim23_an_ordered_ground_target_is_refused_to_mode_one_only() {
    for (lm, fires) in [(0, true), (1, false)] {
        let (mut w, a, x) = duel("AAV", lm, 0, "TGT", land::CLEAR);
        acquire::override_attack(&mut w, a, x);
        let shots = run_counting(&mut w, a, 60);
        assert_eq!(shots > 0, fires, "land mode {lm}: {shots} shots");
        if !fires {
            assert_ne!(w.obj(a).unwrap().target, Some(obj(x)), "dropped");
        }
    }
}

// spec: S-0214 claim 6 (e) and S-0306 claim 23 (scenario P16 part h: without force-fire, the mode-1 copy is offered no attack on an enemy ground vehicle within or beyond its range; the mode-0 copy of an anti-air-only weapon is offered it at both distances)
#[test]
fn claim23_cursor_offers_no_attack_to_mode_one() {
    for (lm, offered) in [(0, true), (1, false)] {
        let (mut w, a, near) = duel("AAV", lm, 0, "TGT", land::CLEAR);
        let far = put(&mut w, Kind::Vehicle, "TGT", 1, 9, 0);
        for (what, x) in [("near", near), ("far", far)] {
            let act =
                targeting::action(&w, HouseId(0), a, ClickOn::Object(x), Modifiers::default());
            assert_eq!(
                act == Action::Attack,
                offered,
                "land mode {lm}, {what}: {act:?}"
            );
        }
    }
}

// spec: S-0306 claim 23 (rank of the water and land rows: rank 4 of S-0301 claim 9, ahead of cloaked (rank 8): a cloaked firer is answered illegal against a target the land row refuses, and cloaked against the same target with mode 0)
#[test]
fn claim23_rows_outrank_a_cloaked_firer() {
    for state in [
        CloakState::Cloaking,
        CloakState::Cloaked,
        CloakState::Uncloaking,
    ] {
        let (mut w, f, t) = duel("AAV", 1, 0, "TGT", land::CLEAR);
        w.obj_mut(f).unwrap().cloak.state = state;
        assert_eq!(answer(&w, f, obj(t)), Answer::Illegal, "{state:?}");
        // The control: the same state with mode 0 gives the rank 8 answer.
        let (mut w, f, t) = duel("AAV", 0, 0, "TGT", land::CLEAR);
        w.obj_mut(f).unwrap().cloak.state = state;
        assert_eq!(answer(&w, f, obj(t)), Answer::Cloaked, "{state:?}");
    }
}

// spec: S-0306 claim 23 (rank of the water and land rows: ahead of the late target tests of rank 9: a seeker type with land-targeting mode 1 is answered illegal against a target the land row refuses; with mode 0 the seeker's out of range, whatever the distance) and S-0301 claim 9 rank 9
#[test]
fn claim23_rows_outrank_the_late_target_tests() {
    for (lm, expected) in [(1, Answer::Illegal), (0, Answer::OutOfRange)] {
        let mut r = modes("AGV", lm, 0);
        r.sim.vehicles.get_mut("AGV").unwrap().hunter_seeker = Some(true);
        let mut w = game(&r);
        let f = put(&mut w, Kind::Vehicle, "AGV", 0, 0, 0);
        let t = put(&mut w, Kind::Vehicle, "TGT", 1, 2, 0);
        assert_eq!(answer(&w, f, obj(t)), expected, "mode {lm}");
        assert_eq!(
            check(&w, f, Some(obj(t)), 0, true),
            expected,
            "mode {lm}, with the range test"
        );
    }
}
