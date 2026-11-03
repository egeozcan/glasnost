//! Claim tests for S-0202 claim 14's house-creation draws (revision 62,
//! Q-2336) on synthetic data: our engine's analogue of scenario R17, a
//! flat clear map with no object, no overlay and no trigger, four start
//! waypoints, the players and the neutral and special houses, the bases
//! setting off and the unit-count setting 0. No stock data.

mod common;

use common::Scene;
use glasnost_map::Fill;
use glasnost_rules::sim::Country;
use glasnost_sim::rng::Rng;
use glasnost_sim::world::{HouseRole, HouseSetup, World};

/// R17's skirmish scene with `p` players (one human, the rest computer),
/// each at the fixed start of its index, then the neutral and the special
/// house (S-0009 claim 19's order), seed `seed`.
fn r17(p: usize, seed: u32) -> Scene {
    let mut s = Scene::new();
    s.setup.sandbox = false;
    s.setup.seed = seed;
    // Invented passive countries for the neutral and special houses, so that
    // neither takes a start position (S-0009 claim 19).
    for name in [glasnost_sim::load::NEUTRAL, glasnost_sim::load::SPECIAL] {
        s.register("countries", name);
        s.ruleset.sim.countries.insert(
            name.into(),
            Country {
                multiplay_passive: Some(true),
                multiplay_passive_base: Some(true),
                ..Default::default()
            },
        );
    }
    for (i, (x, y)) in [(20, 40), (60, 40), (40, 20), (40, 60)]
        .into_iter()
        .enumerate()
    {
        s.map.waypoints.insert(i as u16, (x, y));
    }
    let countries = ["Alphaland", "Betaland"];
    s.setup.houses = (0..p)
        .map(|i| HouseSetup {
            name: format!("Player {i}"),
            country: countries[i % 2].into(),
            human: i == 0,
            start: Some(i as u32),
            role: HouseRole::Player,
            ai: i != 0,
        })
        .collect();
    for (name, role) in [
        (glasnost_sim::load::NEUTRAL, HouseRole::Neutral),
        (glasnost_sim::load::SPECIAL, HouseRole::Special),
    ] {
        s.setup.houses.push(HouseSetup {
            name: name.into(),
            country: name.into(),
            human: false,
            start: None,
            role,
            ai: false,
        });
    }
    // spec: S-0202 R17 (the bases setting off and the unit-count setting 0)
    s.setup.bases = false;
    s.setup.unit_count = 0;
    s
}

/// One house-creation round of `h` houses on `r` (S-0202 claim 14).
fn round(r: &mut Rng, h: usize, half: bool) {
    for _ in 0..h {
        // spec: S-0202 claim 14 (one ranged draw R(450, 1800) per house created)
        let _ = r.ranged(450, 1800, half);
    }
}

/// The starting-unit step's final draw (S-0009 claim 20 and Constants).
fn last(r: &mut Rng, half: bool) {
    // spec: S-0009 claim 20 and Constants (the final ranged draw over 0 … 65535)
    let _ = r.ranged(0, 65535, half);
}

fn half(w: &World) -> bool {
    w.rules().quirks.ranged_draw_half_span
}

/// spec: S-0202 claim 14 (house creation: in a skirmish, 2 × h draws R(450, 1800), two rounds in S-0009 claim 19's order, then the final R(0, 65535) and no other draw before tick 0; R17 Expected)
#[test]
fn claim_14_two_house_rounds_then_the_final_draw() {
    for p in 2..=4 {
        for seed in 1..=5 {
            let s = r17(p, seed);
            let w = s.build();
            let h = w.houses.len();
            assert_eq!(h, p + 2, "the p players and the neutral and special houses");
            let mut want = Rng::new(seed);
            round(&mut want, h, half(&w));
            round(&mut want, h, half(&w));
            last(&mut want, half(&w));
            assert_eq!(w.rng, want, "p {p}, seed {seed}");
        }
    }
}

/// spec: S-0202 claim 14 (variant C: the human's random-position draw over 0 … 3 comes after the first round and before the second, and is the only other draw; S-0009 claim 19)
#[test]
fn claim_14_variant_c_position_draw_between_the_rounds() {
    for p in 2..=4 {
        for seed in 1..=5 {
            let mut s = r17(p, seed);
            for hs in &mut s.setup.houses {
                hs.start = None;
            }
            let w = s.build();
            let h = w.houses.len();
            let mut want = Rng::new(seed);
            round(&mut want, h, half(&w));
            // spec: S-0009 claim 19 (the first player with no position takes one ranged draw over the 4 positions)
            let pos = want.ranged(0, 3, half(&w));
            round(&mut want, h, half(&w));
            last(&mut want, half(&w));
            assert_eq!(w.rng, want, "p {p}, seed {seed}");
            assert_eq!(w.houses[0].start, Some(pos as u32), "p {p}, seed {seed}");
        }
    }
}

/// spec: S-0202 claim 14 (variant B: a pre-placed vehicle of the neutral house takes its creation draw after all the house draws; claim 10's creation draw)
#[test]
fn claim_14_variant_b_a_neutral_vehicle_after_both_rounds() {
    for seed in 1..=5 {
        let mut s = r17(2, seed);
        s.vehicle("TRUCK", common::unit_type(common::TRACK, 4, 5));
        s.map.objects.units.push(glasnost_map::Unit {
            placed: glasnost_map::objects::Placed {
                key: b"1".to_vec(),
                owner: glasnost_sim::load::NEUTRAL.as_bytes().to_vec(),
                type_name: b"TRUCK".to_vec(),
                health: 256,
                cell: common::cell(0, 0),
                facing: 0,
                raw: Vec::new(),
            },
        });
        let w = s.build();
        let truck = w
            .objects
            .values()
            .find(|o| o.kind == glasnost_sim::rules::Kind::Vehicle)
            .expect("the neutral vehicle is placed");
        assert_eq!(
            w.houses[usize::from(truck.owner.unwrap().0)].role,
            HouseRole::Neutral
        );
        let mut want = Rng::new(seed);
        round(&mut want, w.houses.len(), half(&w));
        round(&mut want, w.houses.len(), half(&w));
        // spec: S-0202 claim 10 (the creation draw: the next raw value, its low 16 bits the report value)
        assert_eq!(
            truck.report_value,
            (want.raw() & 0xFFFF) as u16,
            "seed {seed}: its creation draw directly after the second round"
        );
    }
}

/// spec: S-0202 claim 14 (variant D: the water fill's draws over 0 … 3 come directly after the second round's last house draw; S-0009 claim 6)
#[test]
fn claim_14_variant_d_fill_after_the_second_round() {
    for seed in 1..=5 {
        let mut s = r17(2, seed);
        s.map.fill = Fill::Water;
        let w = s.build();
        let h = w.houses.len();
        let cells = s
            .map
            .cells
            .visiting_order(w.rules().quirks.narrow_map_partial_pass)
            .len();
        let mut want = Rng::new(seed);
        round(&mut want, h, half(&w));
        round(&mut want, h, half(&w));
        for _ in 0..cells {
            // spec: S-0009 claim 6 (one draw over 0 … 3 per visited cell)
            let _ = want.ranged(0, 3, half(&w));
        }
        last(&mut want, half(&w));
        assert_eq!(w.rng, want, "seed {seed}");
    }
}

/// spec: S-0202 claim 14 (the two rounds sit on either side of the early phase); S-0009 Edge cases (quirks.double_start_resolution off: one list, after the terrain, so no early phase and the rounds come together, before the fill, and the position draw after the fill)
#[test]
fn claim_14_without_the_early_phase_the_rounds_are_adjacent() {
    for seed in 1..=5 {
        let mut s = r17(3, seed);
        s.map.fill = Fill::Water;
        for hs in &mut s.setup.houses {
            hs.start = None;
        }
        s.ruleset.globals.quirks.double_start_resolution = false;
        let w = s.build();
        let h = w.houses.len();
        let cells = s
            .map
            .cells
            .visiting_order(w.rules().quirks.narrow_map_partial_pass)
            .len();
        let mut want = Rng::new(seed);
        round(&mut want, h, half(&w));
        round(&mut want, h, half(&w));
        for _ in 0..cells {
            // spec: S-0009 claim 6 (one draw over 0 … 3 per visited cell)
            let _ = want.ranged(0, 3, half(&w));
        }
        // spec: S-0009 claim 19 (the first player with no position takes one ranged draw over the 4 positions, here in the late phase)
        let pos = want.ranged(0, 3, half(&w));
        last(&mut want, half(&w));
        assert_eq!(w.rng, want, "seed {seed}");
        assert_eq!(w.houses[0].start, Some(pos as u32), "seed {seed}");
    }
}

/// spec: S-0202 claim 14 (the house draws belong to a skirmish or network game start; our viewer's sandbox start is neither and takes none, design: only the final R(0, 65535))
#[test]
fn claim_14_sandbox_takes_no_house_draw() {
    let mut s = r17(2, 7);
    s.setup.sandbox = true;
    let w = s.build();
    let mut want = Rng::new(7);
    last(&mut want, half(&w));
    assert_eq!(w.rng, want);
}
