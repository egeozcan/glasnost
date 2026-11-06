//! Claim tests for the firing cycle (S-0301), on synthetic data, plus the
//! building processing of S-0215 claim 32 and infantry firing of S-0107
//! claim 11, which drive it.
//!
//! The ruleset is `testkit::combat_ruleset()` with invented weapons added
//! here (no stock values). Most tests call the firing check
//! (`combat::check::check`) and the shot (`combat::fire::shot`) directly,
//! with the firer's state set by hand, so that each claim is observed with
//! every other reason absent. Draw counts are read from the scenario
//! random stream's `taken()` counter, and expected draws are predicted from
//! a clone of the stream.
//!
//! Not tested (seams or not implemented): garrisoned buildings and tank
//! bunkers (claim 7, F1 D₃ to D₅), the sonic-wave live effect and its gate
//! (claim 5 second part, `quirks.shared_wave_slot`; the particle emitters'
//! gates are tested in `claims_s0616_emitters.rs`), cloaking (claim 12), ranks 1
//! to 3, 8, 11 and 16 of claim 9 and its materialising and holding cases,
//! tunnels (claim 13), the charge-turret image (claim 16, F5) and the
//! campaign set-up (claim 3). See `docs/m4-claims/S-0301.md`.

use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, CombatFields, Decimal, DifficultyRow, WeaponType};
use glasnost_sim::combat::behave;
use glasnost_sim::combat::check::{Answer, check};
use glasnost_sim::combat::fire;
use glasnost_sim::infantry::Seq;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

/// Map size of every test here.
const N: i32 = 24;
/// Facing east (S-0204: 16384 facing units, a quarter turn from north).
const EAST: u16 = 16384;
/// Facing west.
const WEST: u16 = 49152;

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

/// An invented single-shot weapon: rate `rof`, range 10 cells, the
/// invisible projectile Bullet (its launch takes no draw), warhead SA.
fn zap(rof: i32) -> WeaponType {
    WeaponType {
        damage: Some(1),
        rof: Some(rof),
        range: Some(d("10")),
        speed: Some(100),
        projectile: Some("Bullet".into()),
        warhead: Some("SA".into()),
        ..Default::default()
    }
}

/// The combat ruleset with the weapon ZAP (rate 30) on the CART, a durable
/// BUGGY as the target, and a second weapon ZAP2 (rate 20) as the CART's
/// secondary.
fn base() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.weapons.insert("ZAP".into(), zap(30));
    r.sim.weapons.insert("ZAP2".into(), zap(20));
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.primary = Some("ZAP".into());
    cart.secondary = Some("ZAP2".into());
    r.sim.vehicles.get_mut("BUGGY").unwrap().strength = Some(30000);
    r
}

/// A world of `r` with an Alpha firer of `kind`/`name` two cells west of the
/// middle facing east, and a Beta BUGGY two cells east of it, set as the
/// firer's target.
fn duel_of(r: &Ruleset, kind: Kind, name: &str, seed: u32) -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with(r, N, seed);
    let a = testkit::spawn(&mut w, kind, name, 0, testkit::mid(N, -2, 0), EAST);
    let b = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 2, 0),
        WEST,
    );
    w.obj_mut(a).unwrap().target = Some(Target::Object(b));
    (w, a, b)
}

fn duel(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    duel_of(r, Kind::Vehicle, "CART", 7)
}

/// Makes a shot with `slot` at the firer's target; returns whether a shot
/// was made and how many raw values of the stream it took.
fn fire_slot(w: &mut World, a: ObjectId, slot: usize) -> (bool, u64) {
    let t = w.obj(a).unwrap().target.unwrap();
    let before = w.rng.taken();
    let made = fire::shot(w, a, slot, t);
    (made, w.rng.taken() - before)
}

/// The delay the last shot set: ticks from now until the countdown is over.
fn delay(w: &World, a: ObjectId) -> i32 {
    w.obj(a).unwrap().combat.rearm_until - w.tick
}

/// [`base`] with ZAP lacking the fire-while-moving flag, so that a
/// destination gives *moving* (S-0310 claim 7).
fn base_not_mobile() -> Ruleset {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().fire_while_moving = Some(false);
    r
}

/// The firing check for `slot` at the firer's target, with range.
fn ask(w: &World, a: ObjectId, slot: usize) -> Answer {
    check(w, a, w.obj(a).unwrap().target, slot, true)
}

/// The next ranged draw R(lo, hi) the stream will give, without taking it.
fn peek(w: &World, lo: i32, hi: i32) -> i32 {
    next(w, lo, hi).0
}

/// The next ranged draw R(lo, hi) and the number of raw values it takes
/// (S-0202 claim 4 rejects raw values above the span), without taking it.
fn next(w: &World, lo: i32, hi: i32) -> (i32, u64) {
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut s = w.rng.clone();
    let before = s.taken();
    let v = s.ranged(lo, hi, half);
    (v, s.taken() - before)
}

/// The direction from `a`'s reference point to `b`'s (S-0204 claim 8).
fn dir(w: &World, a: ObjectId, b: ObjectId) -> u16 {
    let (p, q) = (w.reference_point(a), w.reference_point(b));
    glasnost_num::geom::direction(
        i64::from(q.x - p.x),
        i64::from(q.y - p.y),
        w.rules().quirks.table_direction,
    )
}

/// The rounding mode of the rules (S-0205, `quirks.chop_rounding`).
fn mode(w: &World) -> Rounding {
    w.rules().mode
}

fn int(v: i32, m: Rounding) -> F64 {
    F64::from_i64(i64::from(v), m)
}

/// The value of a decimal field as S-0003 reads it (a double).
fn dec(text: &str, m: Rounding) -> F64 {
    glasnost_num::decimal::parse(text, glasnost_num::decimal::Field::Decimal, m).unwrap()
}

/// A seed for which the full-delay draw of the first shot of `name` in
/// `duel_of` is `u`.
fn seed_with_draw(r: &Ruleset, kind: Kind, name: &str, u: i32) -> u32 {
    (0..200)
        .find(|&s| {
            let (w, _, _) = duel_of(r, kind, name, s);
            peek(&w, 0, 2) == u
        })
        .expect("a seed with that draw")
}

// ---------------------------------------------------------------------------
// Claim 1: the rearm countdown.

// spec: S-0301 claim 1 (a firer that has never fired has no running countdown)
#[test]
fn claim_1_a_new_firer_can_fire_at_once() {
    let (w, a, _) = duel(&base());
    assert_eq!(ask(&w, a, 0), Answer::CanFire);
    assert_eq!(ask(&w, a, 1), Answer::CanFire);
}

// spec: S-0301 claim 1 (rearming for every weapon below t + D, no longer from t + D on)
#[test]
fn claim_1_rearming_until_t_plus_d_for_every_weapon() {
    let (mut w, a, _) = duel(&base());
    let t0 = w.tick;
    let (made, _) = fire_slot(&mut w, a, 0);
    assert!(made);
    let dd = delay(&w, a);
    assert!(dd >= 30, "full delay of a rate-30 weapon: {dd}");
    for k in 0..dd {
        w.tick = t0 + k;
        assert_eq!(ask(&w, a, 0), Answer::Rearming, "tick t + {k}");
        assert_eq!(
            ask(&w, a, 1),
            Answer::Rearming,
            "other weapon, tick t + {k}"
        );
    }
    w.tick = t0 + dd;
    assert_eq!(ask(&w, a, 0), Answer::CanFire);
    assert_eq!(ask(&w, a, 1), Answer::CanFire);
}

// spec: S-0301 claim 1 (D ≤ 0: the countdown is over at once)
#[test]
fn claim_1_a_delay_of_zero_or_less_is_over_at_once() {
    for rof in [0, -5] {
        let mut r = base();
        r.sim.weapons.insert("ZAP".into(), zap(rof));
        let seed = seed_with_draw(&r, Kind::Vehicle, "CART", 0);
        let (mut w, a, _) = duel_of(&r, Kind::Vehicle, "CART", seed);
        fire_slot(&mut w, a, 0);
        assert!(delay(&w, a) <= 0, "rof {rof}");
        assert_eq!(ask(&w, a, 0), Answer::CanFire, "rof {rof}");
    }
}

// spec: S-0301 claim 1 (switching weapons or targets does not change the countdown)
#[test]
fn claim_1_target_and_weapon_changes_keep_the_countdown() {
    let (mut w, a, _) = duel(&base());
    fire_slot(&mut w, a, 0);
    let until = w.obj(a).unwrap().combat.rearm_until;
    let other = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 5, 0),
        WEST,
    );
    w.obj_mut(a).unwrap().target = Some(Target::Object(other));
    assert_eq!(w.obj(a).unwrap().combat.rearm_until, until);
    assert_eq!(ask(&w, a, 1), Answer::Rearming);
    w.tick = until - 1;
    assert_eq!(ask(&w, a, 1), Answer::Rearming);
    w.tick = until;
    assert_eq!(ask(&w, a, 1), Answer::CanFire);
}

// spec: S-0301 claim 1 (counting is in ticks: game speed and quirks.speed_normalised_delays do not change D)
#[test]
fn claim_1_game_speed_and_speed_normalisation_do_not_change_d() {
    let mut delays = Vec::new();
    for normalised in [true, false] {
        for speed in [0u8, 4, 6] {
            let mut r = base();
            r.globals.quirks.speed_normalised_delays = normalised;
            let (mut w, a, _) = duel(&r);
            w.game_speed = speed;
            fire_slot(&mut w, a, 0);
            delays.push(delay(&w, a));
        }
    }
    assert!(delays.iter().all(|&x| x == delays[0]), "{delays:?}");
}

// ---------------------------------------------------------------------------
// Claims 2 and 3, Formulas F1 and F2: the full delay and the house factor.

// spec: S-0301 claim 2 and F1 (D₁ = trunc(RZ₅₃(RZ₅₃(R × h) + u)), u = R(0, 2), one draw)
#[test]
fn claim_2_full_delay_is_rate_plus_one_draw() {
    let (mut w, a, _) = duel(&base());
    for _ in 0..6 {
        let (u, n) = next(&w, 0, 2);
        let (made, draws) = fire_slot(&mut w, a, 0);
        assert!(made);
        assert_eq!(draws, n, "one draw R(0, 2)");
        assert_eq!(delay(&w, a), 30 + u);
        w.tick = w.obj(a).unwrap().combat.rearm_until;
    }
}

// spec: S-0301 claim 2 and F1 (revision 12: each floating-point operation rounds by quirks.chop_rounding and S-0205; each conversion to ticks truncates toward zero)
#[test]
fn claim_2_full_delay_rounds_by_the_rounding_switch() {
    for chop in [true, false] {
        let mut r = base();
        r.globals.quirks.chop_rounding = chop;
        r.sim.weapons.insert("ZAP".into(), zap(37));
        r.sim.difficulty.insert(
            "normal".into(),
            DifficultyRow {
                rof: Some(d("0.7")),
                ..Default::default()
            },
        );
        let (mut w, a, _) = duel(&r);
        let m = mode(&w);
        let h = w.houses[0].combat.rof_factor;
        let u = peek(&w, 0, 2);
        let want = int(37, m).mul(h, m).add(int(u, m), m).to_i32_common();
        fire_slot(&mut w, a, 0);
        assert_eq!(delay(&w, a), want, "chop_rounding {chop}");
    }
}

// spec: S-0301 claim 3 and F2 (h = RZ₅₃(ℓ × κ), the normal row's factor times the country's, for every player house)
#[test]
fn claim_3_house_factor_is_row_times_country() {
    let mut r = base();
    for (row, v) in [("easy", "0.5"), ("normal", "1.5"), ("difficult", "3")] {
        r.sim.difficulty.insert(
            row.into(),
            DifficultyRow {
                rof: Some(d(v)),
                ..Default::default()
            },
        );
    }
    r.sim.countries.get_mut("Alphaland").unwrap().rof = Some(d("0.8"));
    r.sim.countries.get_mut("Betaland").unwrap().rof = Some(d("0.7"));
    let (w, _, _) = duel(&r);
    let m = mode(&w);
    let row = dec("1.5", m);
    assert!(
        w.houses[0]
            .combat
            .rof_factor
            .eq_num(row.mul(dec("0.8", m), m))
    );
    assert!(
        w.houses[1]
            .combat
            .rof_factor
            .eq_num(row.mul(dec("0.7", m), m))
    );
}

// spec: S-0301 claim 3 and F2 (h = 1.0 exactly for the neutral and special houses, whatever the rows and their countries' factors hold; S-0412 claim 5)
#[test]
fn claim_3_non_player_houses_have_factor_one() {
    let mut r = base();
    for (row, v) in [("easy", "2"), ("normal", "1.5"), ("difficult", "0.5")] {
        r.sim.difficulty.insert(
            row.into(),
            DifficultyRow {
                rof: Some(d(v)),
                ..Default::default()
            },
        );
    }
    r.sim.countries.get_mut("Alphaland").unwrap().rof = Some(d("1.5"));
    r.sim.countries.get_mut("Betaland").unwrap().rof = Some(d("0.5"));
    let w = testkit::game_with_non_players(&r, N, 7);
    let m = mode(&w);
    // The player house of the same country: the row times the country.
    assert!(
        w.houses[0]
            .combat
            .rof_factor
            .eq_num(dec("1.5", m).mul(dec("1.5", m), m))
    );
    for h in [2, 3] {
        assert!(w.houses[h].combat.rof_factor.eq_num(int(1, m)), "house {h}");
    }
}

// spec: S-0301 F1 (the full delay takes h: D₁ = trunc(RZ₅₃(RZ₅₃(R × h) + u)))
#[test]
fn f1_full_delay_takes_the_house_factor() {
    let mut r = base();
    r.sim.weapons.insert("ZAP".into(), zap(37));
    r.sim.difficulty.insert(
        "normal".into(),
        DifficultyRow {
            rof: Some(d("1.5")),
            ..Default::default()
        },
    );
    r.sim.countries.get_mut("Alphaland").unwrap().rof = Some(d("0.8"));
    let (mut w, a, _) = duel(&r);
    let m = mode(&w);
    let h = dec("1.5", m).mul(dec("0.8", m), m);
    for _ in 0..4 {
        let u = peek(&w, 0, 2);
        fire_slot(&mut w, a, 0);
        let want = int(37, m).mul(h, m).add(int(u, m), m).to_i32_common();
        assert_eq!(delay(&w, a), want);
        w.tick = w.obj(a).unwrap().combat.rearm_until;
    }
}

// spec: S-0301 claim 2 (order 1: a building holding more than 1 round gets 1 tick with no draw, whatever its burst count or kind)
#[test]
fn claim_2_building_with_spare_rounds_takes_one_tick() {
    for sonic in [false, true] {
        let mut r = tower_rules(3);
        let wpn = r.sim.weapons.get_mut("ZAP").unwrap();
        wpn.burst = Some(2);
        wpn.is_sonic = Some(sonic);
        let (mut w, t, _) = tower_duel(&r);
        let (made, draws) = fire_slot(&mut w, t, 0);
        assert!(made);
        assert_eq!(delay(&w, t), 1, "sonic {sonic}");
        assert_eq!(draws, 0, "no draw, sonic {sonic}");
    }
}

// spec: S-0301 claim 2 (order 2: an effect weapon takes its bare rate after every shot of a burst, never R(3, 5))
#[test]
fn claim_2_effect_weapon_outranks_the_in_burst_delay() {
    let mut r = base();
    let wpn = r.sim.weapons.get_mut("ZAP").unwrap();
    wpn.burst = Some(2);
    wpn.is_sonic = Some(true);
    let (mut w, a, _) = duel(&r);
    for _ in 0..4 {
        let (_, draws) = fire_slot(&mut w, a, 0);
        assert_eq!(delay(&w, a), 30);
        assert_eq!(draws, 0);
        w.tick += 30;
    }
}

// spec: S-0301 claim 2 (order 3 before 4: inside a burst the in-burst delay, after its last shot the full delay)
#[test]
fn claim_2_in_burst_before_full_delay() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().burst = Some(2);
    let (mut w, a, _) = duel(&r);
    let u = peek(&w, 3, 5);
    fire_slot(&mut w, a, 0);
    assert_eq!(delay(&w, a), u, "in-burst R(3, 5)");
    w.tick += u;
    let u = peek(&w, 0, 2);
    fire_slot(&mut w, a, 0);
    assert_eq!(delay(&w, a), 30 + u, "full delay");
}

// ---------------------------------------------------------------------------
// Claim 4: bursts.

// spec: S-0301 claim 4 (the j-th shot leaves the counter at j mod B; in-burst R(3, 5) for j < B, full delay for j ≥ B)
#[test]
fn claim_4_burst_counter_and_in_burst_draws() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().burst = Some(3);
    let (mut w, a, _) = duel(&r);
    for round in 0..2 {
        for j in 1..=3 {
            let (lo, hi, extra) = if j < 3 { (3, 5, 0) } else { (0, 2, 30) };
            let (u, n) = next(&w, lo, hi);
            let (_, draws) = fire_slot(&mut w, a, 0);
            assert_eq!(draws, n);
            assert_eq!(delay(&w, a), extra + u, "round {round}, shot {j}");
            assert_eq!(w.obj(a).unwrap().combat.burst, j % 3);
            w.tick = w.obj(a).unwrap().combat.rearm_until;
        }
    }
}

// spec: S-0301 claim 4 (a vehicle's burst delay j for 1 ≤ j ≤ 4 when not −1: that value, no draw)
#[test]
fn claim_4_vehicle_burst_delay_fields() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().burst = Some(3);
    r.sim.vehicles.get_mut("CART").unwrap().burst_delay = Some([7, 9, -1, -1]);
    let (mut w, a, _) = duel(&r);
    for want in [7, 9] {
        let (_, draws) = fire_slot(&mut w, a, 0);
        assert_eq!((delay(&w, a), draws), (want, 0));
        w.tick += want;
    }
    // A field of −1 falls back to the draw (positions 1 and 2 of a burst of 3 here).
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().burst = Some(3);
    r.sim.vehicles.get_mut("CART").unwrap().burst_delay = Some([-1, 9, -1, -1]);
    let (mut w, a, _) = duel(&r);
    let (u, n) = next(&w, 3, 5);
    let (_, draws) = fire_slot(&mut w, a, 0);
    assert_eq!((delay(&w, a), draws), (u, n));
}

// spec: S-0301 claim 4 (burst delay fields are for vehicles: an infantry unit always draws R(3, 5))
#[test]
fn claim_4_infantry_always_draws_in_burst() {
    let mut r = base();
    r.sim.weapons.insert("RIFLE3".into(), {
        let mut z = zap(25);
        z.burst = Some(3);
        z
    });
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    walker.primary = Some("RIFLE3".into());
    walker.burst_delay = Some([7, 9, 11, 13]);
    let (mut w, a, _) = duel_of(&r, Kind::Infantry, "WALKER", 7);
    for _ in 0..2 {
        let (u, n) = next(&w, 3, 5);
        let (_, draws) = fire_slot(&mut w, a, 0);
        assert_eq!((delay(&w, a), draws), (u, n));
        w.tick += u;
    }
}

// spec: S-0301 claim 4 (the counter is shared: a weapon change mid-burst carries it over and the new weapon's B decides)
#[test]
fn claim_4_counter_carries_over_between_weapons() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().burst = Some(3);
    r.sim.weapons.get_mut("ZAP2").unwrap().burst = Some(2);
    let (mut w, a, _) = duel(&r);
    fire_slot(&mut w, a, 0);
    assert_eq!(w.obj(a).unwrap().combat.burst, 1);
    w.tick = w.obj(a).unwrap().combat.rearm_until;
    let u = peek(&w, 0, 2);
    fire_slot(&mut w, a, 1);
    // Position 2 of 2 for the burst-2 weapon: the full delay of ZAP2 (rate 20).
    assert_eq!(delay(&w, a), 20 + u);
    assert_eq!(w.obj(a).unwrap().combat.burst, 0);
}

// spec: S-0301 claim 4 (a change from one target to another keeps the burst counter)
#[test]
fn claim_4_target_change_keeps_the_counter() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().burst = Some(3);
    let (mut w, a, _) = duel(&r);
    fire_slot(&mut w, a, 0);
    let other = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 3, 1),
        WEST,
    );
    w.obj_mut(a).unwrap().target = Some(Target::Object(other));
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(a).unwrap().combat.burst, 1);
}

// spec: S-0301 claim 4 (when the firer's target is cleared, its burst counter is set to 0: a building dropping its target)
#[test]
fn claim_4_cleared_target_resets_the_counter_building() {
    let mut r = tower_rules(-1);
    r.sim.weapons.get_mut("ZAP").unwrap().burst = Some(3);
    let (mut w, t, b) = tower_duel(&r);
    fire_slot(&mut w, t, 0);
    assert_eq!(w.obj(t).unwrap().combat.burst, 1);
    w.tick = w.obj(t).unwrap().combat.rearm_until;
    // The target leaves the weapon's range: the attack processing drops it.
    let far = Pos::centre(testkit::mid(N, -11, -11), 0);
    w.obj_mut(b).unwrap().pos = far;
    attack(&mut w, t);
    assert_eq!(behave::building_process(&mut w, t), 1);
    let o = w.obj(t).unwrap();
    assert_eq!(o.target, None);
    assert_eq!(o.combat.burst, 0);
}

// spec: S-0301 claim 4 (when the firer's target is cleared, its burst counter is set to 0: a stop order, scenario F3)
#[test]
fn claim_4_stop_order_resets_the_counter() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().burst = Some(3);
    // No passive search, so nothing picks the enemy again after the stop.
    r.sim
        .vehicles
        .get_mut("CART")
        .unwrap()
        .combat
        .can_passive_aquire = Some(false);
    let (mut w, a, _) = duel(&r);
    fire_slot(&mut w, a, 0);
    assert_eq!(w.obj(a).unwrap().combat.burst, 1);
    // Orders are refused through tick 3 (S-0213 claim 1): set the clock past it.
    w.tick = 10;
    assert!(orders::issue(&mut w, HouseId(0), Order::Stop { actor: a }));
    testkit::run(&mut w, 1);
    assert_eq!(w.obj(a).unwrap().target, None);
    assert_eq!(w.obj(a).unwrap().combat.burst, 0);
}

// ---------------------------------------------------------------------------
// Claim 5: effect weapons.

// spec: S-0301 claim 5 (sonic, spark-particle, fire-particle and railgun weapons: the bare rate R, no draw, no house factor, no veteran factor)
#[test]
fn claim_5_effect_weapons_fire_at_their_bare_rate() {
    for kind in 0..4 {
        let mut r = base();
        r.sim.general.veteran_rof = Some(d("0.5"));
        r.sim.vehicles.get_mut("CART").unwrap().veteran_abilities = vec!["ROF".into()];
        r.sim.difficulty.insert(
            "normal".into(),
            DifficultyRow {
                rof: Some(d("2")),
                ..Default::default()
            },
        );
        let wpn = r.sim.weapons.get_mut("ZAP").unwrap();
        match kind {
            0 => wpn.is_sonic = Some(true),
            1 => wpn.use_spark_particles = Some(true),
            2 => wpn.use_fire_particles = Some(true),
            _ => wpn.is_railgun = Some(true),
        }
        let (mut w, a, _) = duel(&r);
        set_rank(&mut w, a, 1);
        let (made, draws) = fire_slot(&mut w, a, 0);
        assert!(made);
        assert_eq!((delay(&w, a), draws), (30, 0), "effect kind {kind}");
    }
}

// spec: S-0301 claim 5 (revision 12: a magnetic-beam weapon is not an effect weapon; it takes the full delay of claim 2 and F1, with its R(0, 2) draw)
#[test]
fn claim_5_magnetic_beam_weapon_is_not_an_effect_weapon() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().is_mag_beam = Some(true);
    let (mut w, a, _) = duel(&r);
    for _ in 0..4 {
        let (u, n) = next(&w, 0, 2);
        let (made, draws) = fire_slot(&mut w, a, 0);
        assert!(made);
        assert_eq!(draws, n, "one draw R(0, 2), as for an ordinary weapon");
        assert_eq!(delay(&w, a), 30 + u);
        w.tick = w.obj(a).unwrap().combat.rearm_until;
    }
}

// ---------------------------------------------------------------------------
// Claim 6: the veteran factor.

/// Sets the firer's experience to `e` (S-0311 claim 1: veteran from 1,
/// elite from 2).
fn set_rank(w: &mut World, a: ObjectId, e: i64) {
    let m = mode(w);
    w.obj_mut(a).unwrap().combat.experience = F64::from_i64(e, m).to_f32(m);
}

// spec: S-0301 claim 6 and F1 (D₂ = trunc(RZ₅₃(D₁ × v)); the worked example R = 30, u = 1, v = 0.5 gives D₁ = 31 and D₂ = 15)
#[test]
fn claim_6_worked_example_veteran_halves_31_to_15() {
    let mut r = base();
    r.sim.general.veteran_rof = Some(d("0.5"));
    r.sim.vehicles.get_mut("CART").unwrap().veteran_abilities = vec!["ROF".into()];
    let seed = seed_with_draw(&r, Kind::Vehicle, "CART", 1);
    let (mut w, a, _) = duel_of(&r, Kind::Vehicle, "CART", seed);
    set_rank(&mut w, a, 1);
    fire_slot(&mut w, a, 0);
    assert_eq!(delay(&w, a), 15);
}

// spec: S-0301 claim 6 and scenario F2 (v written 0.6 is the single above 0.6: D₁ = 10 gives D₂ = 6; written 60% the double below 0.6 gives 5)
#[test]
fn claim_6_decimal_and_percent_veteran_factor() {
    for (text, want) in [("0.6", 6), ("60%", 5)] {
        let mut r = base();
        r.sim.weapons.insert("ZAP".into(), zap(10));
        r.sim.general.veteran_rof = Some(d(text));
        r.sim.vehicles.get_mut("CART").unwrap().veteran_abilities = vec!["ROF".into()];
        let seed = seed_with_draw(&r, Kind::Vehicle, "CART", 0);
        let (mut w, a, _) = duel_of(&r, Kind::Vehicle, "CART", seed);
        set_rank(&mut w, a, 1);
        fire_slot(&mut w, a, 0);
        assert_eq!(delay(&w, a), want, "v = {text}");
    }
}

// spec: S-0301 claim 6 (veteran with the veteran ability, or elite with either ability: × v once; elite does not apply it twice)
#[test]
fn claim_6_which_ranks_and_abilities_take_the_factor() {
    // (experience, veteran ability, elite ability, factor applies)
    let cases = [
        (0, true, true, false),
        (1, true, false, true),
        (1, false, true, false),
        (2, true, false, true),
        (2, false, true, true),
        (2, true, true, true),
        (2, false, false, false),
    ];
    for (e, vet, elite, applies) in cases {
        let mut r = base();
        r.sim.weapons.insert("ZAP".into(), zap(40));
        r.sim.general.veteran_rof = Some(d("0.5"));
        let cart = r.sim.vehicles.get_mut("CART").unwrap();
        if vet {
            cart.veteran_abilities = vec!["ROF".into()];
        }
        if elite {
            cart.elite_abilities = vec!["ROF".into()];
        }
        let seed = seed_with_draw(&r, Kind::Vehicle, "CART", 0);
        let (mut w, a, _) = duel_of(&r, Kind::Vehicle, "CART", seed);
        set_rank(&mut w, a, e);
        fire_slot(&mut w, a, 0);
        let want = if applies { 20 } else { 40 };
        assert_eq!(delay(&w, a), want, "case {e} {vet} {elite}");
    }
}

// ---------------------------------------------------------------------------
// Claim 8: berserk halving.

fn berserk(w: &mut World, a: ObjectId) {
    w.obj_mut(a).unwrap().combat.berserk = Some(10000);
}

// spec: S-0301 claim 8 and F1 (a berserk firer: D = trunc(D₅ / 2))
#[test]
fn claim_8_berserk_halves_the_full_delay() {
    let (mut w, a, _) = duel(&base());
    berserk(&mut w, a);
    for _ in 0..4 {
        let u = peek(&w, 0, 2);
        fire_slot(&mut w, a, 0);
        assert_eq!(delay(&w, a), (30 + u) / 2);
        w.tick = w.obj(a).unwrap().combat.rearm_until;
    }
}

// spec: S-0301 claim 8 and F4 (the in-burst delay is halved: R(3, 5) gives 1 or 2)
#[test]
fn claim_8_berserk_halves_the_in_burst_delay() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().burst = Some(3);
    let (mut w, a, _) = duel(&r);
    berserk(&mut w, a);
    let u = peek(&w, 3, 5);
    fire_slot(&mut w, a, 0);
    assert_eq!(delay(&w, a), u / 2);
    assert!((1..=2).contains(&delay(&w, a)));
}

// spec: S-0301 claim 8 and Edge cases (bare-rate effect weapons and berserk: the bare rate is halved; a bare rate of 1 halves to 0)
#[test]
fn claim_8_berserk_halves_the_bare_rate() {
    for (rof, want) in [(31, 15), (1, 0)] {
        let mut r = base();
        let wpn = r.sim.weapons.get_mut("ZAP").unwrap();
        wpn.rof = Some(rof);
        wpn.is_sonic = Some(true);
        let (mut w, a, _) = duel(&r);
        berserk(&mut w, a);
        fire_slot(&mut w, a, 0);
        assert_eq!(delay(&w, a), want);
    }
}

// spec: S-0301 claim 8 (a disc laser shot is never halved)
#[test]
fn claim_8_disc_laser_is_not_halved() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().disk_laser = Some(true);
    let (mut w, a, _) = duel(&r);
    berserk(&mut w, a);
    let u = peek(&w, 0, 2);
    fire_slot(&mut w, a, 0);
    assert_eq!(delay(&w, a), 30 + u);
}

// spec: S-0301 claim 8 (stated for the firer's state: a berserk building's 1-tick delay halves to 0)
#[test]
fn claim_8_berserk_building_one_tick_halves_to_zero() {
    let (mut w, t, _) = tower_duel(&tower_rules(3));
    berserk(&mut w, t);
    fire_slot(&mut w, t, 0);
    assert_eq!(delay(&w, t), 0);
}

// ---------------------------------------------------------------------------
// Claim 9: the firing check's answers and their ranks.

// spec: S-0301 claim 9 (rank 5 rearming outranks rank 7 out of ammunition and rank 10 out of range)
#[test]
fn claim_9_rearming_outranks_ammunition_and_range() {
    let (mut w, a, b) = duel(&base());
    fire_slot(&mut w, a, 0);
    w.obj_mut(a).unwrap().combat.ammo = 0;
    w.obj_mut(b).unwrap().pos = Pos::centre(testkit::mid(N, -11, -11), 0);
    assert_eq!(ask(&w, a, 0), Answer::Rearming);
    w.tick = w.obj(a).unwrap().combat.rearm_until;
    assert_eq!(ask(&w, a, 0), Answer::OutOfAmmo);
}

// spec: S-0301 claim 9 (rank 7 out of ammunition outranks rank 10 out of range; the range test only when asked)
#[test]
fn claim_9_ammunition_then_range() {
    let (mut w, a, b) = duel(&base());
    w.obj_mut(b).unwrap().pos = Pos::centre(testkit::mid(N, -11, -11), 0);
    assert_eq!(ask(&w, a, 0), Answer::OutOfRange);
    let tg = w.obj(a).unwrap().target;
    assert_ne!(check(&w, a, tg, 0, false), Answer::OutOfRange);
    w.obj_mut(a).unwrap().combat.ammo = 0;
    assert_eq!(ask(&w, a, 0), Answer::OutOfAmmo);
}

// spec: S-0301 claim 9 (rank 10 out of range outranks the moving, still turning and not facing answers of ranks 13 to 15)
#[test]
fn claim_9_range_outranks_the_aiming_answers() {
    let (mut w, a, b) = duel(&base_not_mobile());
    let far = testkit::mid(N, -11, -11);
    w.obj_mut(b).unwrap().pos = Pos::centre(far, 0);
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.dest = Some(Target::Cell(far));
    o.turret = glasnost_sim::turning::Turning::at(0);
    assert_eq!(ask(&w, a, 0), Answer::OutOfRange);
    // Control: back in range, the moving answer shows.
    w.obj_mut(b).unwrap().pos = Pos::centre(testkit::mid(N, 2, 0), 0);
    let _ = t;
    assert_eq!(ask(&w, a, 0), Answer::Moving);
}

// spec: S-0301 claim 9 rank 4 (no weapon in the slot: impossible)
#[test]
fn claim_9_rank_4_no_weapon_is_impossible() {
    let (w, a, _) = duel(&base());
    assert_eq!(ask(&w, a, 5), Answer::Impossible);
    assert_eq!(ask(&w, a, 0), Answer::CanFire);
}

// spec: S-0301 claim 9 rank 4 (a spawner weapon none of whose spawned aircraft is alive: rearming, even when out of ammunition)
#[test]
fn claim_9_rank_4_spawner_without_aircraft_is_rearming() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().spawner = Some(true);
    let (mut w, a, _) = duel(&r);
    w.obj_mut(a).unwrap().combat.ammo = 0;
    assert_eq!(ask(&w, a, 0), Answer::Rearming);
    assert_eq!(
        ask(&w, a, 1),
        Answer::OutOfAmmo,
        "control: the other weapon"
    );
}

// spec: S-0301 claim 9 rank 4 (an illegal target of the projectile, S-0306 claim 23: a cell target of a projectile that is not anti-ground)
#[test]
fn claim_9_rank_4_projectile_illegal_target() {
    let mut r = base();
    r.sim.projectiles.get_mut("Bullet").unwrap().ag = Some(false);
    let (mut w, a, _) = duel(&r);
    fire_slot(&mut w, a, 0);
    let cell = Some(Target::Cell(testkit::mid(N, 1, 0)));
    // Rank 4 outranks rearming (rank 5).
    assert_eq!(check(&w, a, cell, 0, true), Answer::Illegal);
    let r2 = base();
    let (mut w2, a2, _) = duel(&r2);
    fire_slot(&mut w2, a2, 0);
    assert_eq!(check(&w2, a2, cell, 0, true), Answer::Rearming, "control");
}

// spec: S-0301 claim 9 rank 9 (a seeker type answers out of range whatever the distance; below out of ammunition)
#[test]
fn claim_9_rank_9_seeker_is_out_of_range() {
    let mut r = base();
    r.sim.vehicles.get_mut("CART").unwrap().hunter_seeker = Some(true);
    let (mut w, a, _) = duel(&r);
    assert_eq!(ask(&w, a, 0), Answer::OutOfRange);
    let tg = w.obj(a).unwrap().target;
    assert_eq!(check(&w, a, tg, 0, false), Answer::OutOfRange);
    w.obj_mut(a).unwrap().combat.ammo = 0;
    assert_eq!(ask(&w, a, 0), Answer::OutOfAmmo);
}

// spec: S-0301 claim 9 rank 9 (a versus value of exactly zero makes the target illegal; below out of ammunition, above out of range)
#[test]
fn claim_9_rank_9_versus_zero_is_illegal() {
    let mut r = base();
    let zero: Vec<Decimal> = (0..11).map(|_| d("0")).collect();
    r.sim.warheads.get_mut("SA").unwrap().verses = Some(zero);
    let (mut w, a, b) = duel(&r);
    assert_eq!(ask(&w, a, 0), Answer::Illegal);
    w.obj_mut(b).unwrap().pos = Pos::centre(testkit::mid(N, -11, -11), 0);
    assert_eq!(ask(&w, a, 0), Answer::Illegal, "above out of range");
    w.obj_mut(a).unwrap().combat.ammo = 0;
    assert_eq!(ask(&w, a, 0), Answer::OutOfAmmo);
}

// spec: S-0301 claim 9 rank 12 (a vehicle healer: illegal at a full-health target; below out of range, above moving)
#[test]
fn claim_9_rank_12_vehicle_healer() {
    let mut r = base_not_mobile();
    r.sim.weapons.get_mut("ZAP").unwrap().damage = Some(-5);
    let (mut w, a, b) = duel(&r);
    // The BUGGY is an enemy at full health.
    assert_eq!(ask(&w, a, 0), Answer::Illegal);
    let far = testkit::mid(N, -11, -11);
    w.obj_mut(a).unwrap().dest = Some(Target::Cell(far));
    assert_eq!(ask(&w, a, 0), Answer::Illegal, "above moving");
    w.obj_mut(b).unwrap().pos = Pos::centre(far, 0);
    assert_eq!(ask(&w, a, 0), Answer::OutOfRange, "below out of range");
    // Control: a damaged vehicle target is legal.
    let (mut w, a, b) = duel(&r);
    w.obj_mut(b).unwrap().health -= 10;
    assert_eq!(ask(&w, a, 0), Answer::CanFire);
}

// spec: S-0301 claim 9 (ranks 13 and 15: moving outranks not facing, so a vehicle with a destination facing the wrong way does not re-aim)
#[test]
fn claim_9_moving_outranks_not_facing() {
    let (mut w, a, _) = duel(&base_not_mobile());
    let o = w.obj_mut(a).unwrap();
    o.turret = glasnost_sim::turning::Turning::at(0);
    o.body = glasnost_sim::turning::Turning::at(0);
    assert_eq!(ask(&w, a, 0), Answer::NotFacing);
    w.obj_mut(a).unwrap().dest = Some(Target::Cell(testkit::mid(N, -6, 0)));
    assert_eq!(ask(&w, a, 0), Answer::Moving);
}

// ---------------------------------------------------------------------------
// Claim 10: ammunition.

// spec: S-0301 claim 10 (each shot uses one round when a > 0; a = −1 never changes; a = 0 answers out of ammunition)
#[test]
fn claim_10_rounds_are_used() {
    let mut r = base();
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.combat.ammo = Some(2);
    cart.combat.initial_ammo = Some(2);
    let (mut w, a, _) = duel(&r);
    assert_eq!(
        w.obj(a).unwrap().combat.ammo,
        2,
        "starts with the initial ammunition"
    );
    fire_slot(&mut w, a, 0);
    assert_eq!(w.obj(a).unwrap().combat.ammo, 1);
    w.tick = w.obj(a).unwrap().combat.rearm_until;
    fire_slot(&mut w, a, 0);
    assert_eq!(w.obj(a).unwrap().combat.ammo, 0);
    w.tick = w.obj(a).unwrap().combat.rearm_until;
    assert_eq!(ask(&w, a, 0), Answer::OutOfAmmo);
    // Unlimited: −1 is never changed.
    let (mut w, a, _) = duel(&base());
    assert_eq!(w.obj(a).unwrap().combat.ammo, -1);
    fire_slot(&mut w, a, 0);
    assert_eq!(w.obj(a).unwrap().combat.ammo, -1);
}

// spec: S-0301 claim 10, Formulas F6 (a₀ = A when J = −1, otherwise J; quirks.initial_ammo_above_capacity)
#[test]
fn claim_10_starting_ammunition() {
    let start = |ammo: i32, initial: Option<i32>, above: bool| {
        let mut r = base();
        r.globals.quirks.initial_ammo_above_capacity = above;
        let cart = r.sim.vehicles.get_mut("CART").unwrap();
        cart.combat.ammo = Some(ammo);
        cart.combat.initial_ammo = initial;
        let (w, a, _) = duel(&r);
        w.obj(a).unwrap().combat.ammo
    };
    // An absent initial field fills a finite magazine, including 0 for a maximum of 0.
    assert_eq!(start(3, None, true), 3);
    assert_eq!(start(0, None, true), 0);
    // An explicit 0 starts empty, even with an unlimited maximum.
    assert_eq!(start(-1, Some(0), true), 0);
    // Above the maximum: used as it stands; switch off: capped.
    assert_eq!(start(3, Some(5), true), 5);
    assert_eq!(start(3, Some(5), false), 3);
    assert_eq!(
        start(-1, Some(5), false),
        5,
        "no cap for a negative maximum"
    );
}

// spec: S-0301 claim 10 (a disc laser shot uses no round)
#[test]
fn claim_10_disc_laser_uses_no_round() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().disk_laser = Some(true);
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.combat.ammo = Some(2);
    cart.combat.initial_ammo = Some(2);
    let (mut w, a, _) = duel(&r);
    fire_slot(&mut w, a, 0);
    let o = w.obj(a).unwrap();
    assert_eq!(o.combat.ammo, 2);
    assert_eq!(o.combat.reload_left, None, "nor restarts the reload");
}

// spec: S-0301 Edge cases (ammunition below −1: never changed and never out of ammunition)
#[test]
fn edge_ammunition_below_minus_one_is_unlimited() {
    let mut r = base();
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    walker.primary = Some("ZAP".into());
    walker.combat.ammo = Some(-2);
    walker.combat.initial_ammo = Some(-2);
    let (mut w, a, _) = duel_of(&r, Kind::Infantry, "WALKER", 7);
    for _ in 0..20 {
        assert_eq!(ask(&w, a, 0), Answer::CanFire);
        fire_slot(&mut w, a, 0);
        assert_eq!(w.obj(a).unwrap().combat.ammo, -2);
        w.tick = w.obj(a).unwrap().combat.rearm_until;
    }
}

/// Rules with an Alpha building TOWERZ armed with ZAP (no turret, so no
/// aiming), maximum and initial ammunition `ammo`.
fn tower_rules(ammo: i32) -> Ruleset {
    let mut r = base();
    r.registries
        .get_mut("buildings")
        .unwrap()
        .push("TOWERZ".into());
    r.sim.buildings.insert(
        "TOWERZ".into(),
        BuildingType {
            strength: Some(800),
            foundation: Some([1, 1]),
            primary: Some("ZAP".into()),
            combat: CombatFields {
                ammo: Some(ammo),
                initial_ammo: Some(ammo),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    r
}

fn tower_duel(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    duel_of(r, Kind::Building, "TOWERZ", 7)
}

/// Puts a building in *attack* (S-0215 claim 32).
fn attack(w: &mut World, t: ObjectId) {
    let tick = w.tick;
    w.obj_mut(t)
        .unwrap()
        .beh
        .force(Behaviour::Attack, tick, true);
}

/// Runs a building's attack processing once per tick for `n` ticks from
/// now, the way S-0215 claim 32's delays would call it if it were called
/// every tick; returns the ticks of its shots.
fn building_shots(w: &mut World, t: ObjectId, n: i32) -> Vec<i32> {
    let mut shots = Vec::new();
    for _ in 0..n {
        let before = w.obj(t).unwrap().last_fired;
        if ask(w, t, 0) == Answer::CanFire {
            fire_slot(w, t, 0);
        }
        if w.obj(t).unwrap().last_fired != before {
            shots.push(w.tick);
        }
        w.tick += 1;
    }
    shots
}

// spec: S-0301 claim 10 (a building with 3 rounds fires in ticks t, t + 1, t + 2, then rearming until its countdown ends; at 0 rounds the firing check answers out of ammunition. This drives the firing step alone: revision 58's refill comes in the building's own turn, which claims_s0301_r58 drives)
#[test]
fn claim_10_building_with_three_rounds() {
    let (mut w, t, _) = tower_duel(&tower_rules(3));
    let t0 = w.tick;
    let shots = building_shots(&mut w, t, 3);
    assert_eq!(shots, vec![t0, t0 + 1, t0 + 2]);
    let until = w.obj(t).unwrap().combat.rearm_until;
    assert!(until - (t0 + 2) >= 30, "the last round gets the full delay");
    while w.tick < until {
        assert_eq!(ask(&w, t, 0), Answer::Rearming);
        w.tick += 1;
    }
    assert_eq!(ask(&w, t, 0), Answer::OutOfAmmo);
}

// spec: S-0301 claim 10 (a building with unlimited ammunition is not affected by the 1-tick rule)
#[test]
fn claim_10_unlimited_building_gets_the_full_delay() {
    let (mut w, t, _) = tower_duel(&tower_rules(-1));
    let u = peek(&w, 0, 2);
    fire_slot(&mut w, t, 0);
    assert_eq!(delay(&w, t), 30 + u);
}

// spec: S-0301 Edge cases (quirks.building_ammo_fast_rearm off: the delay claim 2's order gives without its first row)
#[test]
fn quirk_building_ammo_fast_rearm_off() {
    let mut r = tower_rules(3);
    r.globals.quirks.building_ammo_fast_rearm = false;
    let (mut w, t, _) = tower_duel(&r);
    let (u, n) = next(&w, 0, 2);
    let (_, draws) = fire_slot(&mut w, t, 0);
    assert_eq!((delay(&w, t), draws), (30 + u, n));
}

// spec: S-0301 scenario F3 building part (burst 2 with 3 rounds: ticks t, t + 1, t + 2; the third shot opens a new burst with an R(3, 5) delay)
#[test]
fn claim_2_building_burst_with_three_rounds() {
    let mut r = tower_rules(3);
    r.sim.weapons.get_mut("ZAP").unwrap().burst = Some(2);
    let (mut w, t, _) = tower_duel(&r);
    let t0 = w.tick;
    assert_eq!(fire_slot(&mut w, t, 0).1, 0);
    w.tick += 1;
    assert_eq!(fire_slot(&mut w, t, 0).1, 0);
    w.tick += 1;
    let (u, n) = next(&w, 3, 5);
    assert_eq!(fire_slot(&mut w, t, 0).1, n);
    assert_eq!(delay(&w, t), u);
    assert_eq!(w.tick, t0 + 2);
}

// ---------------------------------------------------------------------------
// Claim 11 and Formula F3: reloading.

/// The F10 vehicle: maximum and initial ammunition 4, reload 50, increment
/// 10, pip wrap `wrap`, empty reload 120.
fn reloader(wrap: i32) -> Ruleset {
    let mut r = base();
    let c = &mut r.sim.vehicles.get_mut("CART").unwrap().combat;
    c.ammo = Some(4);
    c.initial_ammo = Some(4);
    c.reload = Some(50);
    c.reload_increment = Some(10);
    c.pip_wrap = Some(wrap);
    c.empty_reload = Some(120);
    r
}

/// Runs the vehicle's reload rule turn by turn until `a` is full; returns
/// the number of turns between consecutive rounds.
fn reload_gaps(w: &mut World, a: ObjectId) -> Vec<i32> {
    let mut gaps = Vec::new();
    let mut last = 0;
    for turn in 1..=1000 {
        let before = w.obj(a).unwrap().combat.ammo;
        fire::reload_turn(w, a);
        if w.obj(a).unwrap().combat.ammo > before {
            gaps.push(turn - last);
            last = turn;
        }
        if w.obj(a).unwrap().combat.ammo == 4 {
            break;
        }
    }
    gaps
}

// spec: S-0301 claim 11 and F3 (scenario F10 a: 120 after the last shot, then 50, 60 and 60; pip wrap −2 gives the same, part f)
#[test]
fn claim_11_reload_delays_follow_f3() {
    for wrap in [2, -2] {
        let (mut w, a, _) = duel(&reloader(wrap));
        for _ in 0..4 {
            fire_slot(&mut w, a, 0);
            w.tick = w.obj(a).unwrap().combat.rearm_until;
        }
        assert_eq!(w.obj(a).unwrap().combat.ammo, 0);
        assert_eq!(
            reload_gaps(&mut w, a),
            vec![120, 50, 60, 60],
            "pip wrap {wrap}"
        );
    }
}

// spec: S-0301 F3 (delay = E when a = 0 and E ≠ −1, else L + I × q², q = 1 if W = 0, else trunc(a / W))
#[test]
fn f3_reload_delay_formula() {
    let r = glasnost_sim::rules::SimRules::new(&reloader(3), "f3");
    let i = r.index_of(Kind::Vehicle, "CART").unwrap();
    let mut tc = r.unit(Kind::Vehicle, i).unwrap().combat.clone();
    assert_eq!(fire::reload_delay(&tc, 0), 120);
    // q = trunc(a / 3): 0, 0, 1, 1, 1, 2.
    for (a, q) in [(1, 0), (2, 0), (3, 1), (5, 1), (6, 2), (-4, -1)] {
        assert_eq!(fire::reload_delay(&tc, a), 50 + 10 * q * q, "a = {a}");
    }
    tc.pip_wrap = 0;
    assert_eq!(fire::reload_delay(&tc, 2), 60, "W = 0: q = 1");
    tc.empty_reload = -1;
    assert_eq!(fire::reload_delay(&tc, 0), 60, "E = −1 at a = 0");
}

// spec: S-0301 claim 11 (each shot that leaves a < A restarts the countdown, so firing every 30 ticks keeps postponing the reload)
#[test]
fn claim_11_continuous_firing_postpones_the_reload() {
    let (mut w, a, _) = duel(&reloader(2));
    for _ in 0..3 {
        fire_slot(&mut w, a, 0);
        let before = w.obj(a).unwrap().combat.ammo;
        for _ in 0..30 {
            fire::reload_turn(&mut w, a);
            w.tick += 1;
        }
        assert_eq!(w.obj(a).unwrap().combat.ammo, before);
    }
}

// spec: S-0301 claim 11 (manual reload: no restart on firing, and a never-started countdown counts as over: the round returns in the first turn after the shot)
#[test]
fn claim_11_manual_reload_regains_in_the_first_turn() {
    let mut r = reloader(2);
    r.sim.vehicles.get_mut("CART").unwrap().combat.manual_reload = Some(true);
    let (mut w, a, _) = duel(&r);
    fire_slot(&mut w, a, 0);
    assert_eq!(w.obj(a).unwrap().combat.ammo, 3);
    assert_eq!(w.obj(a).unwrap().combat.reload_left, None);
    fire::reload_turn(&mut w, a);
    assert_eq!(w.obj(a).unwrap().combat.ammo, 4);
}

// spec: S-0301 claim 11 (a vehicle without the mobile-fire flag: no reload progress while its locomotor moves it)
#[test]
fn claim_11_reload_frozen_while_moving_without_mobile_fire() {
    for mobile in [false, true] {
        let mut r = reloader(2);
        r.sim.vehicles.get_mut("CART").unwrap().mobile_fire = Some(mobile);
        // A lone cart: nothing to shoot at.
        let mut w = testkit::game_with(&r, N, 7);
        let a = testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "CART",
            0,
            testkit::mid(N, -2, 0),
            EAST,
        );
        testkit::run(&mut w, 5);
        {
            let o = w.obj_mut(a).unwrap();
            o.combat.ammo = 2;
            o.combat.reload_left = Some(5);
        }
        let issued = orders::issue(
            &mut w,
            HouseId(0),
            Order::Behaviour {
                actor: a,
                behaviour: Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(Target::Cell(testkit::mid(N, -2, 10))),
            },
        );
        assert!(issued);
        let mut moved_ticks = 0;
        for _ in 0..40 {
            testkit::run(&mut w, 1);
            if glasnost_sim::occupancy::in_progress(&w, a) {
                moved_ticks += 1;
            }
        }
        assert!(moved_ticks >= 10, "the cart drove");
        let o = w.obj(a).unwrap();
        if mobile {
            assert!(o.combat.ammo > 2, "with mobile fire the reload goes on");
        } else {
            assert_eq!(o.combat.ammo, 2, "frozen while driving");
        }
    }
}

// spec: S-0301 claim 11 (revision 58: infantry never regain ammunition; a building regains it only by claim 10's refill, all at once, and the reload fields have no effect on either)
#[test]
fn claim_11_infantry_and_buildings_never_reload() {
    let mut r = tower_rules(2);
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    walker.primary = Some("ZAP".into());
    walker.combat.ammo = Some(2);
    walker.combat.initial_ammo = Some(2);
    walker.combat.reload = Some(1);
    r.sim.buildings.get_mut("TOWERZ").unwrap().combat.reload = Some(1);
    let (mut w, inf, _) = duel_of(&r, Kind::Infantry, "WALKER", 7);
    let tw = testkit::spawn(
        &mut w,
        Kind::Building,
        "TOWERZ",
        0,
        testkit::mid(N, -4, 4),
        0,
    );
    for id in [inf, tw] {
        w.obj_mut(id).unwrap().combat.ammo = 0;
        w.obj_mut(id).unwrap().target = None;
    }
    testkit::run(&mut w, 60);
    assert_eq!(w.obj(inf).unwrap().combat.ammo, 0);
    // spec: S-0301 claim 10 (the building's refill to its maximum, not one round per reload)
    assert_eq!(w.obj(tw).unwrap().combat.ammo, 2);
}

// ---------------------------------------------------------------------------
// Claim 13: firing frames.

/// The CART with `f` firing frames and sync frame `s0` for k mod B = 0.
fn framed(f: i32, s0: i32, rof: i32) -> Ruleset {
    let mut r = base();
    r.sim.weapons.insert("ZAP".into(), zap(rof));
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.firing_frames = Some(f);
    cart.firing_sync_frames = Some([s0, -1]);
    r
}

/// Emulates `n` turns of the vehicle in the order of its turn (the
/// firing-frame counter, then its firing step); returns the shot ticks.
fn framed_shots(w: &mut World, a: ObjectId, n: i32) -> Vec<i32> {
    let mut shots = Vec::new();
    for _ in 0..n {
        fire::firing_frame_turn(w, a);
        if ask(w, a, 0) == Answer::CanFire && fire_slot(w, a, 0).0 {
            shots.push(w.tick);
        }
        w.tick += 1;
    }
    shots
}

// spec: S-0301 claim 13 (scenario F11: the counter starts at 2F − 1 = 7; the next shot comes when it equals s = 3, then the rearm delay again)
#[test]
fn claim_13_sync_shot_ignores_the_countdown() {
    let (mut w, a, _) = duel(&framed(4, 3, 200));
    let t0 = w.tick;
    let shots = framed_shots(&mut w, a, 400);
    assert_eq!(shots[0], t0);
    assert_eq!(shots[1], t0 + 4, "sync shot when the counter equals 3");
    assert!(
        shots[2] - shots[0] >= 200,
        "then the rearm delay: {shots:?}"
    );
}

// spec: S-0301 claim 13 (a running counter not at s gives the primary weapon rearming; the secondary is not gated)
#[test]
fn claim_13_gate_holds_only_the_primary() {
    let (mut w, a, _) = duel(&framed(4, 3, 2));
    fire_slot(&mut w, a, 0);
    w.tick = w.obj(a).unwrap().combat.rearm_until + 1;
    assert_eq!(w.obj(a).unwrap().combat.firing_frame, Some(7));
    assert_eq!(ask(&w, a, 0), Answer::Rearming);
    assert_eq!(ask(&w, a, 1), Answer::CanFire);
}

// spec: S-0301 claim 13 (no sync frame given: firing frames change only the picture)
#[test]
fn claim_13_without_sync_frame_shots_follow_the_countdown() {
    let (mut w, a, _) = duel(&framed(4, -1, 2));
    let shots = framed_shots(&mut w, a, 30);
    for p in shots.windows(2) {
        assert!((2..=4).contains(&(p[1] - p[0])), "{shots:?}");
    }
}

// spec: S-0301 Edge cases (a sync frame at or above 2F is never reached: the gate holds the primary until the counter is idle)
#[test]
fn edge_sync_frame_beyond_the_start() {
    let (mut w, a, _) = duel(&framed(4, 8, 2));
    let shots = framed_shots(&mut w, a, 60);
    assert!(shots.len() >= 3);
    for p in shots.windows(2) {
        assert!(p[1] - p[0] >= 8, "{shots:?}");
    }
}

// spec: S-0301 claim 13 (a shot in the tick the counter equals s is a sync shot, which does not restart the counter, even when the countdown is already over)
#[test]
fn claim_13_sync_shot_does_not_restart_the_counter() {
    // F = 4, s = 1, rate 0 (delays 0 to 2): the shot at t starts the counter at 7; it equals 1 at t + 6 (the
    // countdown is over by then), and that sync shot leaves the counter running down to idle at
    // t + 8, when the next shot comes.
    let (mut w, a, _) = duel(&framed(4, 1, 0));
    let t0 = w.tick;
    let shots = framed_shots(&mut w, a, 9);
    assert_eq!(shots, vec![t0, t0 + 6, t0 + 8]);
}

// ---------------------------------------------------------------------------
// Claim 14: animation-delayed building shots.

fn delayed_rules(dfd: i32) -> Ruleset {
    let mut r = tower_rules(-1);
    let b = r.sim.buildings.get_mut("TOWERZ").unwrap();
    b.is_anim_delayed_fire = Some(true);
    b.delayed_fire_delay = Some(dfd);
    r
}

// spec: S-0301 claim 14 (can fire starts the delayed shot; it is made in the building's turn max(d − 1, 0) ticks later; rearming while pending; the countdown starts with the shot)
#[test]
fn claim_14_delayed_shot_timing() {
    for (dfd, wait) in [(10, 9), (1, 0), (0, 0)] {
        let (mut w, t, _) = tower_duel(&delayed_rules(dfd));
        attack(&mut w, t);
        let t0 = w.tick;
        assert_eq!(behave::building_process(&mut w, t), 1);
        assert_eq!(w.obj(t).unwrap().last_fired, -100, "no shot yet, d = {dfd}");
        assert_eq!(w.obj(t).unwrap().combat.rearm_until, i32::MIN);
        let mut fired = None;
        for k in 0..=12 {
            w.tick = t0 + k;
            if fired.is_none() && w.obj(t).unwrap().combat.delayed_shot.is_some() && k > 0 {
                assert_eq!(ask(&w, t, 0), Answer::Rearming, "pending, d = {dfd}");
            }
            behave::delayed_shot_turn(&mut w, t);
            if fired.is_none() && w.obj(t).unwrap().last_fired == t0 + k {
                fired = Some(k);
            }
        }
        assert_eq!(fired, Some(wait), "d = {dfd}");
        assert_eq!(w.obj(t).unwrap().combat.delayed_shot, None);
        assert!(w.obj(t).unwrap().combat.rearm_until > t0 + wait);
    }
}

// spec: S-0301 claim 14 (a target that left the range by the delayed turn: the shot is dropped, no countdown, no longer pending)
#[test]
fn claim_14_dropped_delayed_shot() {
    let (mut w, t, b) = tower_duel(&delayed_rules(10));
    attack(&mut w, t);
    let t0 = w.tick;
    behave::building_process(&mut w, t);
    w.obj_mut(b).unwrap().pos = Pos::centre(testkit::mid(N, -11, -11), 0);
    w.tick = t0 + 9;
    behave::delayed_shot_turn(&mut w, t);
    let o = w.obj(t).unwrap();
    assert_eq!(o.last_fired, -100);
    assert_eq!(o.combat.delayed_shot, None);
    assert_eq!(o.combat.rearm_until, i32::MIN);
}

// ---------------------------------------------------------------------------
// Claim 15: failed launches.

// spec: S-0301 claim 15 (a void shot: the countdown, burst counter and ammunition stay as they were; claim 18: no delay draw)
#[test]
fn claim_15_void_shot_changes_nothing() {
    let mut r = base();
    // An arcing projectile with a turn rate takes the weapon's speed (S-0306 claim 1); at speed 1
    // the launch pitch has no solution (S-0306 claim 6 a).
    r.sim.projectiles.insert(
        "SlowArc".into(),
        glasnost_rules::sim::ProjectileType {
            arcing: Some(true),
            rot: Some(1),
            ..Default::default()
        },
    );
    let wpn = r.sim.weapons.get_mut("ZAP").unwrap();
    wpn.projectile = Some("SlowArc".into());
    wpn.speed = Some(1);
    wpn.burst = Some(3);
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.combat.ammo = Some(3);
    cart.combat.initial_ammo = Some(3);
    let (mut w, a, _) = duel(&r);
    let before = w.obj(a).unwrap().combat.clone();
    let (made, draws) = fire_slot(&mut w, a, 0);
    assert!(!made, "an arcing shot without a pitch solution is void");
    assert_eq!(draws, 0, "no delay draw");
    assert!(w.projectiles.is_empty());
    let o = w.obj(a).unwrap();
    assert_eq!(o.combat.rearm_until, before.rearm_until);
    assert_eq!(o.combat.burst, before.burst);
    assert_eq!(o.combat.ammo, before.ammo);
}

// ---------------------------------------------------------------------------
// Claim 16: charge turrets (the weapon part).

// spec: S-0301 claim 16 (a charge-turret type's shots all use its slot-0 weapon)
#[test]
fn claim_16_charge_turret_uses_slot_zero() {
    let mut r = base();
    let cart = r.sim.vehicles.get_mut("CART").unwrap();
    cart.combat.is_charge_turret = Some(true);
    cart.turret_count = Some(4);
    let (mut w, a, b) = duel(&r);
    glasnost_sim::combat::weapon::set_weapon_number(&mut w, a, 1);
    assert_eq!(w.obj(a).unwrap().combat.weapon_number, 0);
    let tg = Some(Target::Object(b));
    assert_eq!(glasnost_sim::combat::weapon::choose_slot(&w, a, tg), 0);
}

// ---------------------------------------------------------------------------
// Claim 18: draws taken by one shot.

// spec: S-0301 claim 18 (a refused attempt takes no draw: a vehicle's firing step during its countdown)
#[test]
fn claim_18_refused_attempt_takes_no_draw() {
    let (mut w, a, _) = duel(&base());
    fire_slot(&mut w, a, 0);
    let until = w.obj(a).unwrap().combat.rearm_until;
    while w.tick < until {
        let before = w.rng.taken();
        behave::vehicle_step(&mut w, a);
        assert_eq!(w.rng.taken(), before);
        w.tick += 1;
    }
    let before = w.rng.taken();
    let (_, n) = next(&w, 0, 2);
    behave::vehicle_step(&mut w, a);
    assert_eq!(w.rng.taken(), before + n, "the shot's one delay draw");
}

// spec: S-0301 claim 18 (the delay draw comes after the projectile's own draws)
#[test]
fn claim_18_delay_draw_after_the_projectile_draws() {
    let mut r = base();
    let wpn = r.sim.weapons.get_mut("ZAP").unwrap();
    wpn.projectile = Some("Scatter".into());
    r.sim.projectiles.insert(
        "Scatter".into(),
        glasnost_rules::sim::ProjectileType {
            arcing: Some(true),
            inaccurate: Some(true),
            ..Default::default()
        },
    );
    r.sim.general.ballistic_scatter = Some(d("2"));
    // A seed for which "delay draw first" and "projectile draws first" give different delays.
    let mut checked = false;
    for seed in 0..50 {
        let (mut w, a, b) = duel_of(&r, Kind::Vehicle, "CART", seed);
        let first = peek(&w, 0, 2);
        // The projectile's draws, taken on a copy of the world.
        let mut w2 = w.clone();
        let fp = glasnost_sim::combat::position::fire_point(&w2, a, 0, 0);
        let weapon = glasnost_sim::combat::weapon::slot_weapon(&w2, a, 0).unwrap();
        let before = w2.rng.taken();
        let launched =
            glasnost_sim::combat::projectile::launch(&mut w2, a, weapon, Target::Object(b), fp, 1);
        let proj_draws = w2.rng.taken() - before;
        // A scatter beyond the arc's reach is a void shot (S-0306); take a seed that launches.
        if launched.is_none() {
            continue;
        }
        assert!(proj_draws >= 2, "the scatter draws");
        let (after, n) = next(&w2, 0, 2);
        if after == first {
            continue;
        }
        let (_, draws) = fire_slot(&mut w, a, 0);
        assert_eq!(draws, proj_draws + n);
        assert_eq!(delay(&w, a), 30 + after);
        checked = true;
        break;
    }
    assert!(checked);
}

// spec: S-0301 claim 18 (the spray pick of S-0313 comes first, only for the first shot of a burst)
#[test]
fn claim_18_spray_pick_first() {
    let mut r = base();
    r.sim.weapons.get_mut("ZAP").unwrap().burst = Some(2);
    r.sim.vehicles.get_mut("CART").unwrap().spray_attack = Some(true);
    let (mut w, a, _) = duel(&r);
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut s = w.rng.clone();
    let before = s.taken();
    let pick = s.ranged(0, 7, half);
    let u = s.ranged(3, 5, half);
    let n = s.taken() - before;
    let (_, draws) = fire_slot(&mut w, a, 0);
    assert_eq!(draws, n);
    assert_eq!(w.obj(a).unwrap().combat.spray_shift, pick);
    assert_eq!(delay(&w, a), u);
    w.tick += u;
    let (u, n) = next(&w, 0, 2);
    let (_, draws) = fire_slot(&mut w, a, 0);
    assert_eq!(draws, n, "no spray pick for the second shot");
    assert_eq!(delay(&w, a), 30 + u);
}

// ---------------------------------------------------------------------------
// S-0215 claim 32: buildings in guard and attack.

/// The TOWERZ building in guard with or without a target.
fn guard_tower(r: &Ruleset, with_target: bool) -> (World, ObjectId) {
    let (mut w, t, _) = tower_duel(r);
    if !with_target {
        w.obj_mut(t).unwrap().target = None;
    }
    (w, t)
}

/// D_aa of guard (S-0215 claims 21 and 32) for the rules of `w`.
fn aa(w: &World) -> i32 {
    w.rules().behaviour(Behaviour::Guard).aa_rate_delay
}

// spec: S-0215 claim 32 (armed, in guard: sets the ready mark; with a target it starts attack at once and chooses 1 tick)
#[test]
fn s0215_claim_32_armed_guard_with_target_starts_attack() {
    let (mut w, t) = guard_tower(&tower_rules(-1), true);
    w.obj_mut(t).unwrap().beh.ready_mark = false;
    assert_eq!(behave::building_process(&mut w, t), 1);
    let o = w.obj(t).unwrap();
    assert!(o.beh.ready_mark);
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
}

// spec: S-0215 claim 32 (armed, in guard, no target: the anti-air delay D_aa + U(0, 2))
#[test]
fn s0215_claim_32_armed_guard_without_target() {
    let (mut w, t) = guard_tower(&tower_rules(-1), false);
    let u = peek(&w, 0, 2);
    let want = aa(&w) + u;
    assert_eq!(behave::building_process(&mut w, t), want);
    assert_eq!(w.obj(t).unwrap().beh.current, Some(Behaviour::Guard));
}

// spec: S-0215 claim 32 (an electromagnetic-pulse cannon, and a building that can be occupied but has no occupants, choose the anti-air delay even with a target)
#[test]
fn s0215_claim_32_exempt_buildings_take_the_anti_air_delay() {
    for case in 0..2 {
        let mut r = tower_rules(-1);
        let b = r.sim.buildings.get_mut("TOWERZ").unwrap();
        if case == 0 {
            b.emp_cannon = Some(true);
        } else {
            b.can_be_occupied = Some(true);
        }
        let (mut w, t) = guard_tower(&r, true);
        let u = peek(&w, 0, 2);
        let want = aa(&w) + u;
        assert_eq!(behave::building_process(&mut w, t), want, "case {case}");
        assert_eq!(
            w.obj(t).unwrap().beh.current,
            Some(Behaviour::Guard),
            "case {case}"
        );
    }
}

// spec: S-0215 claim 32 (unarmed, in guard: stupid guard 100 ticks; a unit repair building the standard delay; any other 3 × D_r + U(0, 2); no ready mark)
#[test]
fn s0215_claim_32_unarmed_guard_delays() {
    for case in 0..3 {
        let mut r = tower_rules(-1);
        let b = r.sim.buildings.get_mut("TOWERZ").unwrap();
        b.primary = None;
        match case {
            0 => b.stupid_guard_mode = Some(true),
            1 => b.unit_repair = Some(true),
            _ => {}
        }
        let (mut w, t) = guard_tower(&r, false);
        w.obj_mut(t).unwrap().beh.ready_mark = false;
        let dr = w.rules().behaviour(Behaviour::Guard).rate_delay;
        let u = peek(&w, 0, 2);
        let want = match case {
            0 => 100,
            1 => dr + u,
            _ => 3 * dr + u,
        };
        assert_eq!(behave::building_process(&mut w, t), want, "case {case}");
        assert!(!w.obj(t).unwrap().beh.ready_mark, "case {case}");
    }
}

// spec: S-0215 claim 32 (in attack without a target: queues guard and starts it at once, 1 tick)
#[test]
fn s0215_claim_32_attack_without_target_returns_to_guard() {
    let (mut w, t) = guard_tower(&tower_rules(-1), false);
    attack(&mut w, t);
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(w.obj(t).unwrap().beh.current, Some(Behaviour::Guard));
}

// spec: S-0215 claim 32 (in attack: can fire fires, 1 tick; rearming rotates, 2 ticks; out of ammunition or out of range drops the target and returns to guard, 1 tick)
#[test]
fn s0215_claim_32_attack_answer_table() {
    let (mut w, t, b) = tower_duel(&tower_rules(-1));
    attack(&mut w, t);
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(w.obj(t).unwrap().last_fired, w.tick, "can fire: fires");
    assert!(w.obj(t).unwrap().beh.ready_mark);
    w.tick += 1;
    assert_eq!(behave::building_process(&mut w, t), 2, "rearming: 2 ticks");
    assert_eq!(w.obj(t).unwrap().beh.current, Some(Behaviour::Attack));
    w.tick = w.obj(t).unwrap().combat.rearm_until;
    w.obj_mut(b).unwrap().pos = Pos::centre(testkit::mid(N, -11, -11), 0);
    assert_eq!(behave::building_process(&mut w, t), 1, "out of range");
    assert_eq!(w.obj(t).unwrap().target, None);
    assert_eq!(w.obj(t).unwrap().beh.current, Some(Behaviour::Guard));
    // Out of ammunition.
    let (mut w, t, _) = tower_duel(&tower_rules(-1));
    attack(&mut w, t);
    w.obj_mut(t).unwrap().combat.ammo = 0;
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(w.obj(t).unwrap().target, None);
    assert_eq!(w.obj(t).unwrap().beh.current, Some(Behaviour::Guard));
}

// spec: S-0215 claim 32 and scenario B16 part e (revision 103: a maximum ammunition of 0 and no initial-ammunition value, so that every attempt answers out of ammunition, S-0301 claim 10; at the first such answer from the attack order: target cleared, guard current by the immediate start, a 1-tick delay with no delay draw)
#[test]
fn s0215_b16_e_zero_maximum_ammunition_returns_to_guard() {
    let mut r = tower_rules(0);
    r.sim
        .buildings
        .get_mut("TOWERZ")
        .unwrap()
        .combat
        .initial_ammo = None;
    let (mut w, t, _) = tower_duel(&r);
    assert_eq!(w.obj(t).unwrap().combat.ammo, 0);
    assert_eq!(ask(&w, t, 0), Answer::OutOfAmmo);
    attack(&mut w, t);
    let rng = w.rng.clone();
    assert_eq!(behave::building_process(&mut w, t), 1);
    let o = w.obj(t).unwrap();
    assert_eq!(o.target, None);
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert_eq!(w.rng, rng, "no delay draw");
}

/// TOWERZ with a turret (turn-rate field `rot`), voxel or sprite.
fn turret_tower(rot: i32, voxel: bool) -> Ruleset {
    let mut r = tower_rules(-1);
    let b = r.sim.buildings.get_mut("TOWERZ").unwrap();
    b.turret = Some(true);
    b.rot = Some(rot);
    b.turret_anim_is_voxel = Some(voxel);
    r
}

/// Sets the building's turret facing to `f`.
fn turret_at(w: &mut World, t: ObjectId, f: u16) {
    w.obj_mut(t).unwrap().turret = glasnost_sim::turning::Turning::at(f);
}

// spec: S-0215 claim 32 (not facing: rotates towards the target, 2 ticks)
#[test]
fn s0215_claim_32_not_facing_rotates() {
    let (mut w, t, b) = tower_duel(&turret_tower(5, false));
    attack(&mut w, t);
    turret_at(&mut w, t, 0);
    assert_eq!(ask(&w, t, 0), Answer::NotFacing);
    assert_eq!(behave::building_process(&mut w, t), 2);
    assert_eq!(w.obj(t).unwrap().turret.target, dir(&w, t, b));
    assert_eq!(w.obj(t).unwrap().last_fired, -100);
}

// spec: S-0215 claim 32 and F8 (turret snap of a voxel turret: turn-rate field 0, or ε within the bound, sets the facing onto the target and fires in the same processing)
#[test]
fn s0215_claim_32_voxel_turret_snap() {
    // Field 0: any aim error snaps.
    let (mut w, t, _) = tower_duel(&turret_tower(0, true));
    attack(&mut w, t);
    turret_at(&mut w, t, 0);
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(w.obj(t).unwrap().last_fired, w.tick);
    // Field 5: bound 1280; an aim error of 1000 snaps, one of 16384 rotates.
    for (off, snaps) in [(1000, true), (16384, false)] {
        let (mut w, t, b) = tower_duel(&turret_tower(5, true));
        attack(&mut w, t);
        let from = dir(&w, t, b).wrapping_sub(off);
        turret_at(&mut w, t, from);
        let want = if snaps { 1 } else { 2 };
        assert_eq!(behave::building_process(&mut w, t), want, "from {from}");
        assert_eq!(w.obj(t).unwrap().last_fired == w.tick, snaps);
    }
}

// spec: S-0215 Edge cases and F8 (quirks.turret_snap_bound_wraps: on, the bound reads the field modulo 256, so 200 gives 14 336 and 256 gives 0; off, b = 256 × ω with ω = 127 for both, so 32 512)
#[test]
fn s0215_quirk_turret_snap_bound_wraps_both_settings() {
    for (field, off, on_snaps, off_snaps) in [
        (200, 10_000, true, true),
        (200, 20_000, false, true),
        (256, 1000, false, true),
        (100, 20_000, true, true),
        (100, 26_000, false, false),
    ] {
        for wraps in [true, false] {
            let mut r = turret_tower(field, true);
            r.globals.quirks.turret_snap_bound_wraps = wraps;
            let (mut w, t, b) = tower_duel(&r);
            attack(&mut w, t);
            let from = dir(&w, t, b).wrapping_sub(off);
            turret_at(&mut w, t, from);
            let snaps = if wraps { on_snaps } else { off_snaps };
            let want = if snaps { 1 } else { 2 };
            assert_eq!(
                behave::building_process(&mut w, t),
                want,
                "field {field}, error {off}, wraps {wraps}"
            );
        }
    }
}

// spec: S-0215 claim 32 (rearming never snaps; a sprite turret never snaps)
#[test]
fn s0215_claim_32_no_snap_for_rearming_or_sprite_turrets() {
    // Field 10: bound 2560. A voxel turret 2300 off the target snaps and fires (control).
    let (mut w, t, b) = tower_duel(&turret_tower(10, true));
    attack(&mut w, t);
    let from = dir(&w, t, b) - 2300;
    turret_at(&mut w, t, from);
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(w.obj(t).unwrap().last_fired, w.tick);
    // The same while rearming: rotates (2 ticks), no snap.
    let (mut w, t, b) = tower_duel(&turret_tower(10, true));
    attack(&mut w, t);
    fire_slot(&mut w, t, 0);
    w.tick += 1;
    let from = dir(&w, t, b) - 2300;
    turret_at(&mut w, t, from);
    assert_eq!(behave::building_process(&mut w, t), 2);
    assert_ne!(
        w.obj(t).unwrap().last_fired,
        w.tick,
        "no shot while rearming"
    );
    // A sprite turret 2300 off (not facing, within the bound): rotates, no snap.
    let (mut w, t, b) = tower_duel(&turret_tower(10, false));
    attack(&mut w, t);
    let from = dir(&w, t, b) - 2300;
    turret_at(&mut w, t, from);
    assert_eq!(ask(&w, t, 0), Answer::NotFacing);
    assert_eq!(behave::building_process(&mut w, t), 2);
    assert_eq!(w.obj(t).unwrap().last_fired, -100);
}

/// A SAM building: TOWERZ with the SAM flag and a secondary weapon.
fn sam_rules() -> Ruleset {
    let mut r = tower_rules(-1);
    r.sim.projectiles.get_mut("Bullet").unwrap().aa = Some(true);
    let b = r.sim.buildings.get_mut("TOWERZ").unwrap();
    b.sam = Some(true);
    b.secondary = Some("ZAP2".into());
    r
}

// spec: S-0215 claim 32 (SAM attack: no suitable target, an aircraft whose height is not 0, in aiming: drops the target and returns to guard, 1 tick; the out-of-stage rule)
#[test]
fn s0215_claim_32_sam_stages() {
    let (mut w, t, _) = tower_duel(&sam_rules());
    attack(&mut w, t);
    w.obj_mut(t).unwrap().beh.progress = 0;
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(w.obj(t).unwrap().target, None);
    assert_eq!(w.obj(t).unwrap().beh.current, Some(Behaviour::Guard));
    // Out of stage: nothing, the count kept, D_aa + U(0, 2).
    let (mut w, t, _) = tower_duel(&sam_rules());
    attack(&mut w, t);
    w.obj_mut(t).unwrap().beh.progress = 3;
    let u = peek(&w, 0, 2);
    let want = w.rules().behaviour(Behaviour::Attack).aa_rate_delay + u;
    assert_eq!(behave::building_process(&mut w, t), want);
    assert_eq!(w.obj(t).unwrap().beh.progress, 3);
    assert!(w.obj(t).unwrap().target.is_some());
}

// ---------------------------------------------------------------------------
// S-0107 claim 11: infantry firing.

/// The WALKER armed with RIFLEX (rate 50) and fire frames `ff` (standing,
/// prone, secondary, secondary prone), with a firing sequence.
fn rifleman(ff: [i32; 4]) -> Ruleset {
    let mut r = base();
    r.sim.weapons.insert("RIFLEX".into(), zap(50));
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    walker.primary = Some("RIFLEX".into());
    walker.fire_frames = Some(ff);
    for k in ["stand", "prone", "fire_standing", "fire_prone"] {
        walker.sequences.insert(
            k.into(),
            glasnost_rules::sim::SequenceEntry {
                start: 0,
                count: 6,
                multiplier: 0,
                direction: None,
            },
        );
    }
    r
}

// spec: S-0107 claim 11 (with a target and the weapon ready: enters the attack state, the body facing set at once to the target's direction). The fire frame is 3, so no shot ends the state in the same step (r40, End of the attack state)
#[test]
fn s0107_claim_11_enters_attack_and_snaps_facing() {
    let r = rifleman([3; 4]);
    let (mut w, a, b) = duel_of(&r, Kind::Infantry, "WALKER", 7);
    w.obj_mut(a).unwrap().body = glasnost_sim::turning::Turning::at(0);
    behave::infantry_step(&mut w, a);
    let o = w.obj(a).unwrap();
    assert!(o.infantry.as_ref().unwrap().attack);
    assert_eq!(o.body.value(w.tick), dir(&w, a, b));
}

// spec: S-0107 claim 11 (with F = 0 the shot leaves on the tick the firing kind starts)
#[test]
fn s0107_claim_11_fire_frame_zero_shoots_at_once() {
    let r = rifleman([0; 4]);
    let (mut w, a, _) = duel_of(&r, Kind::Infantry, "WALKER", 7);
    behave::infantry_step(&mut w, a);
    assert_eq!(w.obj(a).unwrap().last_fired, w.tick);
}

/// A WALKER with fire frame `f` (standing), already in the attack state on
/// its target with *fire-standing* current at stage `stage` (stage delay
/// 3, so the frozen-stage rule does not apply) and its countdown running.
fn attacking(f: i32, stage: i32) -> (World, ObjectId, ObjectId) {
    let r = rifleman([f, 0, 0, 0]);
    let (mut w, a, b) = duel_of(&r, Kind::Infantry, "WALKER", 7);
    w.tick = 100;
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.combat.last_shot_target = o.target;
    o.combat.rearm_until = t + 50;
    let held = o.target;
    let st = o.infantry.as_mut().unwrap();
    st.watch = held;
    st.attack = true;
    st.seq = Some(Seq::FireStanding);
    st.seq_delay = 3;
    st.seq_t0 = t - 3 * stage;
    assert_eq!(st.stage(t), stage);
    (w, a, b)
}

fn in_attack(w: &World, a: ObjectId) -> bool {
    w.obj(a).unwrap().infantry.as_ref().unwrap().attack
}

// spec: S-0107 claim 11 (not ready at the fire frame: the attack state ends)
#[test]
fn s0107_claim_11_not_ready_at_the_fire_frame_ends_attack() {
    let (mut w, a, _) = attacking(2, 2);
    behave::infantry_step(&mut w, a);
    assert!(!in_attack(&w, a));
    // Control: at another stage the attack state stays.
    let (mut w, a, _) = attacking(2, 1);
    behave::infantry_step(&mut w, a);
    assert!(in_attack(&w, a));
}

// spec: S-0107 claim 11 (the attack state ends when the unit has no target)
#[test]
fn s0107_claim_11_attack_ends_without_a_target() {
    let (mut w, a, _) = attacking(2, 0);
    w.obj_mut(a).unwrap().target = None;
    behave::infantry_step(&mut w, a);
    assert!(!in_attack(&w, a));
    // Control: with the target it stays.
    let (mut w, a, _) = attacking(2, 0);
    behave::infantry_step(&mut w, a);
    assert!(in_attack(&w, a));
}

// spec: S-0107 r46 claim 11, Changes of target (Q-2616: every change of the target ends the attack state at the moment it is made, whatever rule makes it and in whichever actor's turn: a retaliation or other override's new target, the drop of the target at a berserk hit, the gaining of a target by a unit that had none; a target that does not differ is no change)
#[test]
fn s0107_claim_11_attack_ends_on_a_new_target() {
    use glasnost_sim::combat::acquire::override_attack;
    let buggy =
        |w: &mut World| testkit::spawn(w, Kind::Vehicle, "BUGGY", 1, testkit::mid(N, 2, 2), WEST);
    // A retaliation's override in another actor's turn: ended at once.
    let (mut w, a, _) = attacking(2, 0);
    let other = buggy(&mut w);
    override_attack(&mut w, a, other);
    assert_eq!(w.obj(a).unwrap().target, Some(Target::Object(other)));
    assert!(!in_attack(&w, a), "a retaliation's new target");
    // The drop of the target at a berserk hit.
    let (mut w, a, _) = attacking(2, 0);
    assert!(glasnost_sim::psychic::enter(&mut w, a, 100));
    assert_eq!(w.obj(a).unwrap().target, None);
    assert!(!in_attack(&w, a), "a berserk hit's drop");
    // The gaining of a target by a unit still in the attack state with none.
    let (mut w, a, b) = attacking(2, 0);
    {
        let o = w.obj_mut(a).unwrap();
        o.target = None;
        o.infantry.as_mut().unwrap().watch = None;
    }
    override_attack(&mut w, a, b);
    assert!(!in_attack(&w, a), "gaining a target");
    // Control: an override on the target it holds is no change.
    let (mut w, a, b) = attacking(2, 0);
    override_attack(&mut w, a, b);
    assert!(in_attack(&w, a), "the same target");
}

// spec: S-0107 r46 claim 11, Changes of target (Q-2616: a change of target ends the attack state; our engine's test at the start of the unit's turn catches a rule that writes a target without testing, and the firing turn has no lazy ending of its own)
#[test]
fn s0107_claim_11_a_change_written_elsewhere_ends_before_the_turn() {
    let (mut w, a, _) = attacking(2, 0);
    let other = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 2, 2),
        WEST,
    );
    w.obj_mut(a).unwrap().target = Some(Target::Object(other));
    // The firing turn with no test before it keeps the state (the lazy
    // ending of the Q-2616 interim is gone) ...
    let (mut w2, a2) = (w.clone(), a);
    behave::infantry_step(&mut w2, a2);
    assert!(in_attack(&w2, a2), "no lazy ending at the firing turn");
    // ... and the unit's own turn ends it, with the new target as the
    // baseline of the next test (the weapon is not ready, so no new entry).
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(o.infantry.as_ref().unwrap().watch, o.target);
    assert!(!in_attack(&w, a), "ended in the unit's turn");
}

// spec: S-0107 claim 11 (the firing kind: fire-prone for a prone unit, fire-standing otherwise; the attack state is entered even when the request is turned down)
#[test]
fn s0107_claim_11_firing_kind_by_stance() {
    for prone in [false, true] {
        let r = rifleman([0; 4]);
        let (mut w, a, _) = duel_of(&r, Kind::Infantry, "WALKER", 7);
        {
            let st = w.obj_mut(a).unwrap().infantry.as_mut().unwrap();
            st.prone = prone;
            st.seq = Some(if prone { Seq::Prone } else { Seq::Stand });
        }
        behave::infantry_step(&mut w, a);
        let want = if prone {
            Seq::FireProne
        } else {
            Seq::FireStanding
        };
        assert_eq!(w.obj(a).unwrap().infantry.as_ref().unwrap().seq, Some(want));
    }
    // A type without a fire-standing entry: the request is turned down, the
    // attack state entered. The fire frame is 3, so no shot ends it in the
    // same step (S-0107 r40 claim 11, End of the attack state).
    let mut r = rifleman([3; 4]);
    r.sim
        .infantry
        .get_mut("WALKER")
        .unwrap()
        .sequences
        .remove("fire_standing");
    let (mut w, a, _) = duel_of(&r, Kind::Infantry, "WALKER", 7);
    behave::infantry_step(&mut w, a);
    let st = w.obj(a).unwrap().infantry.clone().unwrap();
    assert!(st.attack);
    assert_ne!(st.seq, Some(Seq::FireStanding));
}

// spec: S-0107 claim 11 (the shot leaves on the tick in the attack state on which the stage of the current sequence equals the fire frame F)
#[test]
fn s0107_claim_11_shot_at_the_fire_frame() {
    let r = rifleman([2, 0, 0, 0]);
    let mut w = testkit::game_with(&r, N, 7);
    let a = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(N, -2, 0),
        EAST,
    );
    testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 2, 0),
        WEST,
    );
    let mut shot = None;
    for _ in 0..200 {
        let t = w.tick;
        testkit::run(&mut w, 1);
        let o = w.obj(a).unwrap();
        if o.last_fired == t {
            let st = o.infantry.clone().unwrap();
            // spec: S-0107 r40 claim 11 (End of the attack state: the attack state ends with the shot)
            assert!(!st.attack, "the shot ends the attack state");
            assert_eq!(st.seq, Some(Seq::FireStanding));
            assert_eq!(st.stage(t), 2);
            shot = Some(t);
            break;
        }
    }
    assert!(shot.is_some(), "the rifleman fired");
}

/// A WALKER with fire frame `f` (its firing kind has 6 image frames) at
/// range of a BUGGY; runs until it fires, giving the shot tick and the
/// unit's sequence state before that tick.
fn first_shot(f: i32) -> Option<(World, ObjectId, i32, glasnost_sim::infantry::InfantryState)> {
    let r = rifleman([f, 0, 0, 0]);
    let mut w = testkit::game_with(&r, N, 7);
    let a = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(N, -2, 0),
        EAST,
    );
    testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 2, 0),
        WEST,
    );
    for _ in 0..200 {
        let t = w.tick;
        let before = w.obj(a).unwrap().infantry.clone().unwrap();
        testkit::run(&mut w, 1);
        if w.obj(a).unwrap().last_fired == t {
            return Some((w, a, t, before));
        }
    }
    None
}

// spec: S-0107 State and ordering (the firing rules are judged against the sequence current before the tick's completion follow-up: when the fire frame equals C, the shot is still released, on the tick the sequence completes)
#[test]
fn s0107_fire_frame_equal_to_the_count_still_fires() {
    let (w, a, t, before) = first_shot(6).expect("a fire frame equal to C releases the shot");
    assert_eq!(before.seq, Some(Seq::FireStanding));
    assert_eq!(before.stage(t), 6, "the shot leaves on the completion tick");
    // The completion follow-up came after the shot on the same tick.
    let st = w.obj(a).unwrap().infantry.clone().unwrap();
    assert_eq!(st.seq, Some(Seq::Stand));
}

// spec: S-0107 r40 claim 11 (End of the attack state: a unit is in the attack state when its firing kind completes if the kind completed before its stage reached F) and State and ordering (a unit in the attack state whose firing kind completed on tick t with stand as its follow-up leaves the attack state on tick t + 1, not on t). The fire frame 7 is past the kind's 6 image frames, so the kind completes with no shot
#[test]
fn s0107_attack_state_ends_the_tick_after_completion() {
    let r = rifleman([7, 0, 0, 0]);
    let mut w = testkit::game_with(&r, N, 7);
    let a = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        0,
        testkit::mid(N, -2, 0),
        EAST,
    );
    testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(N, 2, 0),
        WEST,
    );
    let mut fired_kind = false;
    let mut completed = None;
    for _ in 0..200 {
        let t = w.tick;
        testkit::run(&mut w, 1);
        let st = w.obj(a).unwrap().infantry.clone().unwrap();
        if st.seq == Some(Seq::FireStanding) {
            fired_kind = true;
        } else if fired_kind && st.seq == Some(Seq::Stand) {
            completed = Some(t);
            assert!(
                st.attack,
                "still in the attack state on the completion tick"
            );
            break;
        }
    }
    assert!(fired_kind, "the rifleman started its firing kind");
    let t = completed.expect("the firing kind completed");
    // On t + 1 the stage-delay-0 ending comes first, and the unit, whose
    // weapon never fired and so is ready, starts a new firing kind on that
    // same tick: only a unit out of the attack state begins it.
    testkit::run(&mut w, 1);
    let st = w.obj(a).unwrap().infantry.clone().unwrap();
    assert_eq!(
        st.seq,
        Some(Seq::FireStanding),
        "a new firing kind on t + 1"
    );
    assert_eq!(st.stage(t + 1), 0, "it started on t + 1");
}

// spec: S-0215 claim 32 (SAM attack, aimed stage, can fire: fires both weapons in this processing although the primary shot starts the countdown; back to aiming)
#[test]
fn s0215_claim_32_sam_fires_both_weapons() {
    let mut r = sam_rules();
    r.registries.insert("aircraft".into(), vec!["KITE".into()]);
    r.sim.aircraft.insert(
        "KITE".into(),
        glasnost_rules::sim::UnitType {
            strength: Some(30000),
            ..Default::default()
        },
    );
    let mut w = testkit::game_with(&r, N, 7);
    let t = testkit::spawn(
        &mut w,
        Kind::Building,
        "TOWERZ",
        0,
        testkit::mid(N, -2, 0),
        EAST,
    );
    let k = testkit::spawn(
        &mut w,
        Kind::Aircraft,
        "KITE",
        1,
        testkit::mid(N, 2, 0),
        WEST,
    );
    w.obj_mut(k).unwrap().pos.z += 512;
    w.obj_mut(t).unwrap().target = Some(Target::Object(k));
    attack(&mut w, t);
    w.obj_mut(t).unwrap().beh.progress = 1;
    assert_eq!(ask(&w, t, 0), Answer::CanFire);
    let before = w.projectiles.len();
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(w.projectiles.len(), before + 2, "two shots");
    assert_eq!(w.obj(t).unwrap().beh.progress, 0);
}

// spec: S-0301 claim 17 (a type with the damage-reduces-readiness flag loses ammunition when damaged and restarts its reload countdown at once; the amount is S-0302's)
#[test]
fn claim_17_damage_empties_the_magazine() {
    let mut r = reloader(2);
    let c = &mut r.sim.vehicles.get_mut("CART").unwrap().combat;
    c.damage_reduces_readiness = Some(true);
    c.readiness_reduction_multiplier = Some(d("2"));
    let (mut w, a, b) = duel(&r);
    assert_eq!(w.obj(a).unwrap().combat.reload_left, None);
    glasnost_sim::combat::damage::apply(
        &mut w,
        a,
        glasnost_sim::combat::damage::Hit {
            amount: 150,
            warhead: None,
            distance: 0,
            attacker: Some(b),
            house: Some(HouseId(1)),
            ignore_defences: false,
            no_escape: false,
        },
    );
    let o = w.obj(a).unwrap();
    assert!(o.combat.ammo < 4, "rounds lost: {}", o.combat.ammo);
    let tc = w
        .rules()
        .unit(Kind::Vehicle, o.type_index)
        .unwrap()
        .combat
        .clone();
    assert_eq!(
        o.combat.reload_left,
        Some(fire::reload_delay(&tc, o.combat.ammo))
    );
}

// spec: S-0301 Edge cases (burst count 0 or less: treated as 1, so every shot gets the full delay and the counter stays 0)
#[test]
fn edge_burst_count_below_one_is_one() {
    for burst in [0, -2] {
        let mut r = base();
        r.sim.weapons.get_mut("ZAP").unwrap().burst = Some(burst);
        let (mut w, a, _) = duel(&r);
        for _ in 0..3 {
            let u = peek(&w, 0, 2);
            fire_slot(&mut w, a, 0);
            assert_eq!(delay(&w, a), 30 + u, "burst {burst}");
            assert_eq!(w.obj(a).unwrap().combat.burst, 0, "burst {burst}");
            w.tick = w.obj(a).unwrap().combat.rearm_until;
        }
    }
}

// ---------------------------------------------------------------------------
// S-0308 claim 12 and S-0215 claim 32: upgrade weapons and the upgrade
// weapon gate (upgrades installed by S-0406's placement, `add_upgrade`).

/// An upgrade type: (name, primary, secondary, elite slots).
type Up<'a> = (
    &'a str,
    Option<&'a str>,
    Option<&'a str>,
    Option<[&'a str; 2]>,
);

/// TOWERZ with its own `host` primary (none: unarmed), the upgrade types
/// `ups`, and the weapon ZAPB with burst count 2.
fn upgrade_rules(host: Option<&str>, ups: &[Up]) -> Ruleset {
    let mut r = tower_rules(-1);
    r.sim.weapons.insert(
        "ZAPB".into(),
        WeaponType {
            burst: Some(2),
            ..zap(25)
        },
    );
    let tower = r.sim.buildings.get_mut("TOWERZ").unwrap();
    tower.primary = host.map(Into::into);
    tower.econ.upgrades = Some(3);
    for &(name, p, s, elite) in ups {
        r.registries.get_mut("buildings").unwrap().push(name.into());
        r.sim.buildings.insert(
            name.into(),
            BuildingType {
                strength: Some(100),
                foundation: Some([1, 1]),
                primary: p.map(Into::into),
                secondary: s.map(Into::into),
                combat: CombatFields {
                    elite_weapon_slots: elite.map(|e| e.iter().map(|x| x.to_string()).collect()),
                    ..Default::default()
                },
                ..Default::default()
            },
        );
    }
    r
}

/// The tower duel with the named upgrades installed in this order.
fn upgraded(r: &Ruleset, names: &[&str]) -> (World, ObjectId, ObjectId) {
    let (mut w, t, b) = tower_duel(r);
    for n in names {
        let u = glasnost_sim::objects::type_index(w.rules(), Kind::Building, n.as_bytes()).unwrap();
        glasnost_sim::econ::placement::add_upgrade(&mut w, t, u);
    }
    (w, t, b)
}

/// A weapon by name.
fn wid(w: &World, name: &str) -> glasnost_sim::combat::data::WeaponId {
    let r = w.rules();
    (0..r.combat.weapons.len())
        .map(|i| i as u32)
        .find(|&x| r.combat.weapon(x).name == name)
        .unwrap()
}

/// The weapons of the projectiles in flight, in creation order.
fn launched(w: &World) -> Vec<glasnost_sim::combat::data::WeaponId> {
    w.projectiles.values().map(|p| p.weapon).collect()
}

// spec: S-0308 claim 12 and S-0406 claim 12 (slot i comes from the first upgrade position, in increasing position order up to the count, whose type has a weapon there; always its ordinary slot, whatever the host's rank; only then the building's own slot)
#[test]
fn s0308_claim_12_upgrades_supply_weapons() {
    use glasnost_sim::combat::weapon::slot_weapon;
    let r = upgrade_rules(
        Some("ZAP2"),
        &[
            ("U1", Some("ZAP"), None, Some(["ZAPB", "ZAPB"])),
            ("U2", Some("ZAP2"), Some("ZAPB"), None),
        ],
    );
    let (mut w, t, _) = upgraded(&r, &["U1", "U2"]);
    assert_eq!(
        slot_weapon(&w, t, 0),
        Some(wid(&w, "ZAP")),
        "slot 0 from U1"
    );
    assert_eq!(
        slot_weapon(&w, t, 1),
        Some(wid(&w, "ZAPB")),
        "slot 1 from U2"
    );
    set_rank(&mut w, t, 2);
    assert_eq!(
        slot_weapon(&w, t, 0),
        Some(wid(&w, "ZAP")),
        "never U1's elite slot"
    );
    let (w, t, _) = upgraded(&r, &["U2", "U1"]);
    assert_eq!(
        slot_weapon(&w, t, 0),
        Some(wid(&w, "ZAP2")),
        "U2 installed first"
    );
    // No upgrade: the building's own slot.
    let r = upgrade_rules(Some("ZAP2"), &[("U1", None, Some("ZAP"), None)]);
    let (w, t, _) = upgraded(&r, &["U1"]);
    assert_eq!(slot_weapon(&w, t, 0), Some(wid(&w, "ZAP2")));
    assert_eq!(slot_weapon(&w, t, 1), Some(wid(&w, "ZAP")));
}

// spec: S-0215 claim 32 and S-0406 claim 12 (the gate: position 1's upgrade, its ordinary weapons; a primary, and both slots the same type, a primary burst above 1, or a secondary burst above 1)
#[test]
fn s0215_claim_32_upgrade_gate_predicate() {
    type Case<'a> = (Option<&'a str>, Option<&'a str>, Option<[&'a str; 2]>, bool);
    let cases: [Case; 8] = [
        (Some("ZAP"), None, None, false),
        (Some("ZAP"), Some("ZAP"), None, true),
        (Some("ZAPB"), None, None, true),
        (Some("ZAP"), Some("ZAPB"), None, true),
        (Some("ZAP"), Some("ZAP2"), None, false),
        (None, Some("ZAPB"), None, false),
        // Identical elite weapons do not pass; identical ordinary ones do,
        // whatever the elite ones.
        (Some("ZAP"), Some("ZAP2"), Some(["ZAP", "ZAP"]), false),
        (Some("ZAP"), Some("ZAP"), Some(["ZAP", "ZAP2"]), true),
    ];
    for (i, (p, s, e, pass)) in cases.into_iter().enumerate() {
        let r = upgrade_rules(Some("ZAP"), &[("U1", p, s, e)]);
        let (mut w, t, _) = upgraded(&r, &["U1"]);
        assert_eq!(behave::upgrade_gate(&w, t), Some(pass), "case {i}");
        set_rank(&mut w, t, 2);
        assert_eq!(
            behave::upgrade_gate(&w, t),
            Some(pass),
            "case {i}, elite host"
        );
    }
    let r = upgrade_rules(Some("ZAP"), &[]);
    let (w, t, _) = upgraded(&r, &[]);
    assert_eq!(behave::upgrade_gate(&w, t), None, "no upgrade: no gate");
    // The first installed upgrade decides, whichever supplies the weapon.
    let r = upgrade_rules(
        None,
        &[
            ("U1", None, Some("ZAP"), None),
            ("U2", Some("ZAP"), Some("ZAP"), None),
        ],
    );
    let (w, t, _) = upgraded(&r, &["U1", "U2"]);
    assert_eq!(behave::upgrade_gate(&w, t), Some(false));
    let (w, t, _) = upgraded(&r, &["U2", "U1"]);
    assert_eq!(behave::upgrade_gate(&w, t), Some(true));
}

// spec: S-0215 claim 32 (an upgrade-only building searches and starts attack; a failed gate makes no attempt on can fire, keeps its target and attack, sets the ready mark and chooses 1 tick with no draw)
#[test]
fn s0215_claim_32_failed_gate_attacks_without_shots() {
    let r = upgrade_rules(None, &[("U1", Some("ZAP"), None, None)]);
    let (mut w, t, b) = upgraded(&r, &["U1"]);
    // Guard with a target: armed through the upgrade, so it starts attack.
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(w.obj(t).unwrap().beh.current, Some(Behaviour::Attack));
    for _ in 0..3 {
        w.obj_mut(t).unwrap().beh.ready_mark = false;
        assert_eq!(ask(&w, t, 0), Answer::CanFire);
        let before = w.rng.taken();
        assert_eq!(behave::building_process(&mut w, t), 1);
        assert_eq!(w.rng.taken(), before, "no draw");
        let o = w.obj(t).unwrap();
        assert!(launched(&w).is_empty());
        assert_eq!(o.last_fired, -100);
        assert_eq!(o.target, Some(Target::Object(b)));
        assert_eq!(o.beh.current, Some(Behaviour::Attack));
        assert!(o.beh.ready_mark);
        w.tick += 1;
    }
}

// spec: S-0215 claim 32 and S-0301 claim 9 (a passing gate: a primary attempt, then a secondary attempt with no firing check, despite the primary's rearm; slots 0 and 1 resolved by S-0308 claim 12, not necessarily from one upgrade)
#[test]
fn s0215_claim_32_passing_gate_paired_attempts() {
    let r = upgrade_rules(
        None,
        &[
            ("U1", Some("ZAPB"), None, None),
            ("U2", Some("ZAP"), Some("ZAP2"), None),
        ],
    );
    let (mut w, t, _) = upgraded(&r, &["U1", "U2"]);
    attack(&mut w, t);
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(launched(&w), [wid(&w, "ZAPB"), wid(&w, "ZAP2")]);
    // Same-weapon upgrade, single shots: both attempts though the primary
    // began its rearm countdown.
    let r = upgrade_rules(None, &[("U1", Some("ZAP"), Some("ZAP"), None)]);
    let (mut w, t, _) = upgraded(&r, &["U1"]);
    attack(&mut w, t);
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(launched(&w), [wid(&w, "ZAP"), wid(&w, "ZAP")]);
    // spec: S-0301 claim 9 (a secondary attempt with no weapon does nothing)
    let r = upgrade_rules(None, &[("U1", Some("ZAPB"), None, None)]);
    let (mut w, t, _) = upgraded(&r, &["U1"]);
    attack(&mut w, t);
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(launched(&w), [wid(&w, "ZAPB")]);
}

// spec: S-0215 claim 32 (quirks.upgrade_attack_weapon_gate off: a failed gate permits the ordinary selected-weapon attempt; a passing gate still pairs; no upgrade: the ordinary attempt either way)
#[test]
fn s0215_quirk_upgrade_attack_weapon_gate_off() {
    let mut r = upgrade_rules(
        Some("ZAP2"),
        &[
            ("U1", Some("ZAP"), None, None),
            ("U2", Some("ZAP"), Some("ZAP"), None),
        ],
    );
    r.globals.quirks.upgrade_attack_weapon_gate = false;
    let (mut w, t, _) = upgraded(&r, &["U1"]);
    attack(&mut w, t);
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(launched(&w), [wid(&w, "ZAP")]);
    let (mut w, t, _) = upgraded(&r, &["U2"]);
    attack(&mut w, t);
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(launched(&w), [wid(&w, "ZAP"), wid(&w, "ZAP")]);
    for gate in [true, false] {
        r.globals.quirks.upgrade_attack_weapon_gate = gate;
        let (mut w, t, _) = upgraded(&r, &[]);
        attack(&mut w, t);
        assert_eq!(behave::building_process(&mut w, t), 1);
        assert_eq!(launched(&w), [wid(&w, "ZAP2")], "gate {gate}");
    }
}

// spec: S-0215 claims 32 and 33 and S-0308 claim 12 (B36: a building armed only by an installed upgrade finds a target by its own per-tick search in guard, starts attack, and with a failed gate keeps attacking without shots)
#[test]
fn s0215_claim_32_upgrade_only_defence_searches_and_attacks() {
    let r = upgrade_rules(None, &[("U1", Some("ZAP"), None, None)]);
    let (mut w, t, b) = upgraded(&r, &["U1"]);
    w.obj_mut(t).unwrap().target = None;
    w.obj_mut(t).unwrap().targeting_timer = w.tick;
    glasnost_sim::combat::acquire::per_tick(&mut w, t);
    assert_eq!(w.obj(t).unwrap().target, Some(Target::Object(b)));
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(w.obj(t).unwrap().beh.current, Some(Behaviour::Attack));
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert!(launched(&w).is_empty());
    assert_eq!(w.obj(t).unwrap().beh.current, Some(Behaviour::Attack));
}

// ---------------------------------------------------------------------------
// Q-0798: the upgrade pair, the empty secondary and animation delay (S-0301
// claims 9 and 14, S-0215 claim 32), and the upgrade positions (S-0406
// claim 12).

/// `upgrade_rules` with the host building animation-delayed (20 ticks).
fn delayed_upgrade_rules(host: Option<&str>, ups: &[Up]) -> Ruleset {
    let mut r = upgrade_rules(host, ups);
    let b = r.sim.buildings.get_mut("TOWERZ").unwrap();
    b.is_anim_delayed_fire = Some(true);
    b.delayed_fire_delay = Some(20);
    r
}

// spec: S-0301 claim 9 (an attempt whose secondary slot resolves to no weapon does nothing: no shot, no burst-counter change, no rearm-countdown change, no ammunition use and no random draw; the primary shot's burst position and rearm delay stay intact); S-0215 claim 32 (F17)
#[test]
fn s0301_claim_9_empty_secondary_attempt_does_nothing() {
    let r = upgrade_rules(None, &[("U1", Some("ZAPB"), None, None)]);
    let (mut w, t, _) = upgraded(&r, &["U1"]);
    attack(&mut w, t);
    let mut only_primary = w.clone();
    let target = only_primary.obj(t).unwrap().target.unwrap();
    assert!(fire::shot(&mut only_primary, t, 0, target));
    assert_eq!(behave::building_process(&mut w, t), 1);
    let (a, b) = (w.obj(t).unwrap(), only_primary.obj(t).unwrap());
    assert_eq!(
        a.combat, b.combat,
        "burst, rearm and ammunition as after the primary alone"
    );
    assert_eq!(a.combat.burst, 1, "the first shot of a burst of 2");
    assert_eq!(
        w.rng.taken(),
        only_primary.rng.taken(),
        "no draw from the secondary"
    );
    assert_eq!(launched(&w).len(), 1);
}

// spec: S-0215 claim 32 and S-0301 claim 14 (Q-0798: a passing gate makes the pair immediately in that processing even for an animation-delayed building, starting no delayed shot; an empty secondary does nothing, also then)
#[test]
fn s0301_claim_14_passing_gate_beats_animation_delay() {
    let r = delayed_upgrade_rules(None, &[("U1", Some("ZAP"), Some("ZAP"), None)]);
    let (mut w, t, _) = upgraded(&r, &["U1"]);
    attack(&mut w, t);
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(
        launched(&w),
        [wid(&w, "ZAP"), wid(&w, "ZAP")],
        "the pair, at once"
    );
    assert_eq!(
        w.obj(t).unwrap().combat.delayed_shot,
        None,
        "no delayed shot"
    );
    // The empty secondary: only the primary, immediately, no delayed shot.
    let r = delayed_upgrade_rules(None, &[("U1", Some("ZAPB"), None, None)]);
    let (mut w, t, _) = upgraded(&r, &["U1"]);
    attack(&mut w, t);
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(launched(&w), [wid(&w, "ZAPB")]);
    assert_eq!(w.obj(t).unwrap().combat.delayed_shot, None);
}

// spec: S-0215 claim 32 and S-0301 claim 14 (a failed gate with the original quirk setting starts neither an attempt nor a delayed shot; with quirks.upgrade_attack_weapon_gate off the ordinary path applies, including the animation delay; a bypassed gate (no upgrade) uses the ordinary delayed shot)
#[test]
fn s0301_claim_14_failed_gate_and_bypass_with_animation_delay() {
    let mut r = delayed_upgrade_rules(Some("ZAP2"), &[("U1", Some("ZAP"), None, None)]);
    let (mut w, t, _) = upgraded(&r, &["U1"]);
    attack(&mut w, t);
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert!(launched(&w).is_empty());
    assert_eq!(
        w.obj(t).unwrap().combat.delayed_shot,
        None,
        "no delayed shot"
    );
    r.globals.quirks.upgrade_attack_weapon_gate = false;
    let (mut w, t, _) = upgraded(&r, &["U1"]);
    attack(&mut w, t);
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert!(
        launched(&w).is_empty(),
        "the ordinary delayed shot, not yet made"
    );
    assert_eq!(w.obj(t).unwrap().combat.delayed_shot, Some(w.tick + 19));
    let (mut w, t, _) = upgraded(&r, &[]);
    attack(&mut w, t);
    assert_eq!(behave::building_process(&mut w, t), 1);
    assert_eq!(w.obj(t).unwrap().combat.delayed_shot, Some(w.tick + 19));
}

// spec: S-0406 claim 12 and S-0215 claim 32 (the attack gate applies only with a nonzero count and an occupied position 1: a level-2 installation records position 2 and leaves the gate bypassed, a later level-less one records position 3; the provider order is 2 then 3)
#[test]
fn s0406_claim_12_level_skip_bypasses_the_gate() {
    use glasnost_sim::combat::weapon::{installed_upgrades, slot_weapon};
    let mut r = upgrade_rules(
        Some("ZAP2"),
        &[
            ("U1", Some("ZAP"), None, None),
            ("U2", Some("ZAPB"), Some("ZAPB"), None),
        ],
    );
    r.sim
        .buildings
        .get_mut("U1")
        .unwrap()
        .econ
        .powers_up_to_level = Some(2);
    let (w, t, _) = upgraded(&r, &["U1", "U2"]);
    let b = &w.obj(t).unwrap().econ.building;
    assert_eq!(b.upgrade_count, 3);
    assert_eq!(b.upgrades[0], None, "position 1 stays empty");
    assert!(b.upgrades[1].is_some() && b.upgrades[2].is_some());
    assert_eq!(
        behave::upgrade_gate(&w, t),
        None,
        "the failing U1 in position 2 imposes no gate"
    );
    let order: Vec<u32> = installed_upgrades(&w, t).collect();
    assert_eq!(order, [b.upgrades[1].unwrap(), b.upgrades[2].unwrap()]);
    assert_eq!(
        slot_weapon(&w, t, 0),
        Some(wid(&w, "ZAP")),
        "position 2 first"
    );
    assert_eq!(
        slot_weapon(&w, t, 1),
        Some(wid(&w, "ZAPB")),
        "position 3 supplies the slot 2 lacks"
    );
}

// spec: S-0406 claim 12 (count 0 after a turret-upgrade sale: retained positions supply no weapons and no gate; a new level-less installation at count 0 records position 1 and activates that position's gate, higher retained positions staying ineligible until the count reaches them, where a later installation replaces the retained type)
#[test]
fn s0406_claim_12_count_zero_retains_positions_without_effect() {
    use glasnost_sim::combat::weapon::{installed_upgrades, slot_weapon};
    let r = upgrade_rules(
        Some("ZAP2"),
        &[
            ("U1", Some("ZAP"), Some("ZAP"), None),
            ("U2", Some("ZAPB"), None, None),
            ("U3", Some("ZAP"), None, None),
        ],
    );
    let (mut w, t, _) = upgraded(&r, &["U1", "U2"]);
    let (u1, u2) = {
        let b = &w.obj(t).unwrap().econ.building;
        (b.upgrades[0].unwrap(), b.upgrades[1].unwrap())
    };
    // A turret-upgrade sale leaves count 0 and the recorded types.
    w.obj_mut(t).unwrap().econ.building.upgrade_count = 0;
    assert_eq!(
        installed_upgrades(&w, t).count(),
        0,
        "no provider at count 0"
    );
    assert_eq!(behave::upgrade_gate(&w, t), None, "no gate at count 0");
    assert_eq!(
        slot_weapon(&w, t, 0),
        Some(wid(&w, "ZAP2")),
        "the host's own weapon"
    );
    // A new installation records position 1 and the gate tests it (U3: a
    // single-shot primary only, which fails).
    let u3 = glasnost_sim::objects::type_index(w.rules(), Kind::Building, b"U3").unwrap();
    glasnost_sim::econ::placement::add_upgrade(&mut w, t, u3);
    let b = &w.obj(t).unwrap().econ.building;
    assert_eq!((b.upgrade_count, b.upgrades[0]), (1, Some(u3)));
    assert_eq!(b.upgrades[1], Some(u2), "position 2 stays recorded");
    assert_ne!(Some(u1), b.upgrades[0]);
    assert_eq!(
        behave::upgrade_gate(&w, t),
        Some(false),
        "position 1's gate"
    );
    assert_eq!(
        installed_upgrades(&w, t).collect::<Vec<_>>(),
        [u3],
        "position 2 not yet eligible"
    );
    // The next installation replaces position 2's retained type.
    let u1b = glasnost_sim::objects::type_index(w.rules(), Kind::Building, b"U1").unwrap();
    glasnost_sim::econ::placement::add_upgrade(&mut w, t, u1b);
    let b = &w.obj(t).unwrap().econ.building;
    assert_eq!((b.upgrade_count, b.upgrades[1]), (2, Some(u1b)));
}

// spec: S-0406 claim 12 (a count above 3, as on the wall tower: there is no fourth valid position; the provider search considers positions 1 to 3 only and the gate still refers to position 1)
#[test]
fn s0406_claim_12_count_above_three_has_three_positions() {
    use glasnost_sim::combat::weapon::installed_upgrades;
    let r = upgrade_rules(
        Some("ZAP2"),
        &[
            ("U1", Some("ZAP"), Some("ZAP"), None),
            ("U2", Some("ZAPB"), None, None),
            ("U3", Some("ZAP"), None, None),
        ],
    );
    let (mut w, t, _) = upgraded(&r, &["U1", "U2", "U3"]);
    w.obj_mut(t).unwrap().econ.building.upgrade_count = 4;
    assert_eq!(installed_upgrades(&w, t).count(), 3);
    assert_eq!(behave::upgrade_gate(&w, t), Some(true), "position 1 passes");
}
