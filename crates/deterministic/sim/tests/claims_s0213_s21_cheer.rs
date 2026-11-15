//! S-0213 claim 19 and Formulas (a cheer is accepted when t > L + R; the
//! sum and the comparison are exact integer operations), sim issues S21
//! (#419). Synthetic data only. Ledger: `docs/m8-claims/sim-issues.md`,
//! *Batch S21*.

mod s21_targeting_common;

use glasnost_sim::orders::Order;
use glasnost_sim::targeting::CheerMemory;
use glasnost_sim::world::World;
use s21_targeting_common::*;

fn game_with_rate(rate: i32) -> World {
    let mut r = ruleset();
    r.sim.general.maximum_cheer_rate = Some(rate);
    game_of(&r)
}

// spec: S-0213 Formulas (t > L + R exactly: L = -100 at program start, claim 19, and R at the 32-bit minimum give a sum far below every tick)
#[test]
fn issue_419_a_rate_at_the_32_bit_minimum_accepts() {
    let w = game_with_rate(i32::MIN);
    let mut m = CheerMemory::default();
    assert_eq!(m.cheer(&w), Some(Order::Cheer));
}

// spec: S-0213 Formulas (t > L + R exactly: L = 5 and R at the 32-bit maximum give a sum above every tick)
#[test]
fn issue_419_a_rate_at_the_32_bit_maximum_refuses() {
    let w = game_with_rate(i32::MAX);
    let mut m = CheerMemory { last: 5 };
    assert_eq!(m.cheer(&w), None);
    assert_eq!(m.last, 5);
}
