//! S-0204 Edge cases (revision 14, answers Q-3103), sim issues S21 (#294):
//! the Off form of `quirks.table_direction` for arguments that are not a
//! number, infinite arguments and two zero arguments. Ledger:
//! `docs/m8-claims/sim-issues.md`, *Batch S21*.

use glasnost_num::geom::{direction, direction_real_b};
use glasnost_num::{F64, Rounding};

const NAN: F64 = F64(0x7ff8_0000_0000_0000);
const INF: F64 = F64(0x7ff0_0000_0000_0000);
const NEG_INF: F64 = F64(0xfff0_0000_0000_0000);

fn off(x: F64, y: F64) -> u16 {
    direction_real_b(x, y, false, Rounding::TowardZero)
}

fn n(v: i64) -> F64 {
    F64::from_i64(v, Rounding::TowardZero)
}

// spec: S-0204 Edge cases (revision 14: with quirks.table_direction off, two zero arguments give 16384, map east on the exact scale, wherever a rule takes a direction by F3)
#[test]
fn issue_294_two_zero_arguments_give_east() {
    assert_eq!(direction(0, 0, false), 16384);
    assert_eq!(off(n(0), n(0)), 16384);
    assert_eq!(off(F64(1 << 63), n(0)), 16384, "a negative zero");
}

// spec: S-0204 Edge cases (revision 14: an argument that is not a number counts as zero; each infinite argument then counts as +1 or -1 by its sign and each finite one as 0; X = +inf with Y finite gives 16384, X = +inf with Y = -inf gives 24576)
#[test]
fn issue_294_arguments_that_are_not_finite() {
    assert_eq!(off(INF, n(5)), 16384);
    assert_eq!(off(INF, NEG_INF), 24576);
    assert_eq!(off(NAN, INF), 0, "not a number counts as zero");
    assert_eq!(off(NEG_INF, n(-7)), 49152);
    assert_eq!(off(n(3), NEG_INF), 32768);
    assert_eq!(off(NAN, NAN), 16384, "both zero after the not-a-number rule");
    assert_eq!(off(NAN, n(3)), 0, "a finite argument with a zero");
}
