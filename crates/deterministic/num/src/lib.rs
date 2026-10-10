//! Number domains of the simulation (S-0205), computed with integers only.
//!
//! - [`soft`]: IEEE 754 binary32/binary64 values as bit patterns, with the
//!   original's round-toward-zero arithmetic or round-to-nearest
//!   (`quirks.chop_rounding`), and the conversions of S-0205 claim 7.
//! - [`decimal`]: decimal text to values as S-0003 reads them.
//! - [`geom`]: the table square root, the direction from one point to
//!   another and facing arithmetic (S-0204).
//! - [`trig`]: table trigonometry (S-0105 F1).
//! - [`int`]: integer helpers for the wrapping and division rules of S-0205.
//!
//! This is a simulation crate: no host floating point is used outside tests,
//! and every operation gives the same bits on every platform.

pub mod bigmath;
pub mod decimal;
pub mod geom;
pub mod int;
pub mod soft;
#[cfg(test)]
mod tablegen;
pub mod tables;
pub mod trig;

pub use soft::{F32, F64, Rounding};
