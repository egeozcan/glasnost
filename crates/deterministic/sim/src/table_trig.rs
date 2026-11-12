//! The table sine and cosine of S-0105 F1 as the simulation uses them:
//! with `quirks.table_trig_negative_bias` read from the world's ruleset
//! (S-0105 F1 and Quirks; S-0208 and S-0215 name the same shared switch).

use glasnost_num::trig::{cos_tb, sin_tb};
use glasnost_num::{F64, Rounding};

use crate::world::World;

/// Whether negative angles land two table steps short (S-0105 F1).
pub fn bias(w: &World) -> bool {
    // spec: S-0105 F1 and Quirks (quirks.table_trig_negative_bias; off: exact rounding)
    w.rules().quirks.table_trig_negative_bias
}

/// sinT x under the world's `quirks.table_trig_negative_bias`.
pub fn sin_q(w: &World, x: F64, mode: Rounding) -> F64 {
    sin_tb(x, mode, bias(w))
}

/// cosT x under the world's `quirks.table_trig_negative_bias`.
pub fn cos_q(w: &World, x: F64, mode: Rounding) -> F64 {
    cos_tb(x, mode, bias(w))
}
