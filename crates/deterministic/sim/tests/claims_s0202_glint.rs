//! Claim tests for S-0202 claim 14's resource-glint setup on synthetic
//! data: invented tiberiums, values and a glint animation type, no stock
//! data.

mod harvest_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::AnimationType;
use glasnost_sim::rng::Rng;
use glasnost_sim::world::World;
use harvest_common::*;

/// The ore of the tests: ORE and GEM cells at stages 0 and 3, and one
/// stage-3 ORE cell for the wrapping product.
fn ore() -> Vec<(CellXY, u8, u8)> {
    vec![
        (at(0, 0), ORE, 0),
        (at(2, 0), ORE + 3, 3),
        (at(4, 1), GEM, 0),
        (at(-3, 2), GEM + 4, 3),
        (at(1, -4), ORE + 1, 3),
        (at(-2, -1), ORE, 0),
    ]
}

/// The ruleset with glint type GLINT (one creation draw R(450, 900): ρ = 2
/// and 1) at frequency `f`; `None` configures no glint type.
fn glint_rules(f: Option<i32>) -> Ruleset {
    let mut r = ruleset();
    r.registries
        .entry("animations".into())
        .or_default()
        .push("GLINT".into());
    r.sim.animations.insert(
        "GLINT".into(),
        AnimationType {
            random_rate: Some([2, 1]),
            ..Default::default()
        },
    );
    if let Some(f) = f {
        r.sim.general.ore_twinkle = Some("glint".into());
        r.sim.general.ore_twinkle_chance = Some(f);
    }
    r
}

/// The stream after the glint setup as claim 14 predicts it, from the
/// stream `before` it: each cell of nonzero contained value, in visiting
/// order, takes R(0, f − 1) and, on 0, the glint's creation draw.
fn predicted(w: &World, before: &Rng, f: i32) -> Rng {
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut r = before.clone();
    let order = w
        .cells
        .grid
        .visiting_order(w.rules().quirks.narrow_map_partial_pass);
    for c in order {
        if glasnost_sim::econ::resource::worth(w, c) as i32 == 0 {
            continue;
        }
        if r.ranged(0, f.wrapping_sub(1), half) == 0 {
            let _ = r.ranged(450, 900, half);
        }
    }
    r
}

// spec: S-0202 claim 14 (no configured glint type: the group takes no draw; resource cells take no draw at loading, R8 C)
#[test]
fn claim_14_no_glint_type_takes_no_draw() {
    let bare = game(&glint_rules(None));
    let w = game_with_ore(&glint_rules(None), &ore(), 7);
    assert_eq!(w.rng, bare.rng);
}

// spec: S-0202 claim 14 and table *Load-time draw order* (after the game-start recalculation, before tick 0: each eligible cell's choice R(0, f − 1) in visiting order, a zero result's creation draws before the next choice)
#[test]
fn claim_14_glint_choices_and_creations() {
    let before = game_with_ore(&glint_rules(None), &ore(), 7).rng;
    for f in [2, 3, 50] {
        let w = game_with_ore(&glint_rules(Some(f)), &ore(), 7);
        assert_eq!(w.rng, predicted(&w, &before, f), "f = {f}");
        assert_ne!(w.rng, before, "f = {f}");
    }
}

// spec: S-0202 claim 14 (f = 1: the choice R(0, 0) takes no raw value and every eligible cell creates a glint, whose own draws apply)
#[test]
fn claim_14_frequency_one_creates_everywhere() {
    let before = game_with_ore(&glint_rules(None), &ore(), 7).rng;
    let w = game_with_ore(&glint_rules(Some(1)), &ore(), 7);
    let mut r = before.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    for _ in 0..ore().len() {
        let _ = r.ranged(450, 900, half);
    }
    assert_eq!(w.rng, r);
}

// spec: S-0202 claim 14 (f = 0: f − 1 = −1, the reversed choice R(0, −1) takes one raw value and creates on a result of 0; f = i32::MIN: f − 1 wraps to i32::MAX, claim 5's span)
#[test]
fn claim_14_reversed_and_wrapping_frequencies() {
    let before = game_with_ore(&glint_rules(None), &ore(), 7).rng;
    for f in [0, -5, i32::MIN] {
        let w = game_with_ore(&glint_rules(Some(f)), &ore(), 7);
        assert_eq!(w.rng, predicted(&w, &before, f), "f = {f}");
    }
}

// spec: S-0202 claim 14 (contained value V × (d + 1) as a wrapping signed 32-bit product; a zero value takes no choice)
#[test]
fn claim_14_zero_contained_value_takes_no_draw() {
    // ORE worth 2³⁰ per bale: stage 3 holds 4 bales, 2³² wraps to 0; stage 0 is eligible.
    let mut r = glint_rules(Some(1));
    r.sim.tiberiums.get_mut("ORE").unwrap().value = Some(1 << 30);
    let before = {
        let mut n = r.clone();
        n.sim.general.ore_twinkle = None;
        game_with_ore(&n, &ore(), 7).rng
    };
    let w = game_with_ore(&r, &ore(), 7);
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut p = before.clone();
    // Eligible: the two stage-0 ORE cells and both GEM cells (f = 1: a glint each, no choice draw).
    for _ in 0..4 {
        let _ = p.ranged(450, 900, half);
    }
    assert_eq!(w.rng, p);
    // A tiberium of value 0 gives no eligible cell.
    let mut r = glint_rules(Some(1));
    r.sim.tiberiums.get_mut("ORE").unwrap().value = Some(0);
    r.sim.tiberiums.get_mut("GEM").unwrap().value = Some(0);
    let w = game_with_ore(&r, &ore(), 7);
    assert_eq!(w.rng, before);
}
