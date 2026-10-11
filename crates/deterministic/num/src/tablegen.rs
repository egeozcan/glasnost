//! Generation and check of the committed tables in `tables.rs`.
//!
//! `cargo test -p glasnost-num tables_match_definitions` recomputes every
//! entry from its definition (S-0105 F1, S-0204 F3) with the reference
//! arithmetic of `bigmath` and compares. `GLASNOST_UPDATE_TABLES=1 cargo test
//! -p glasnost-num regenerate_tables -- --ignored` rewrites the file.

use num_bigint::BigUint;

use crate::bigmath::{self, FRAC};
use crate::decimal;
use crate::soft::F32;

/// τ(0 … 2048): sin(2πn/8192) truncated toward zero to single precision.
pub fn sine_quarter() -> Vec<u32> {
    let pi = bigmath::pi();
    (0..=2048u32)
        .map(|n| {
            // θ = π n / 4096
            let theta = (&pi * n) / 4096u32;
            let s = bigmath::sin(&theta);
            if n == 2048 {
                // sin(π/2) = 1 exactly; the series is a hair short.
                return 0x3F80_0000;
            }
            bigmath::trunc_single(&s).0
        })
        .collect()
}

/// T(0 … 4096) = trunc24(arctan(i σ)), σ = 3276705 / 2^27.
pub fn atan_table() -> Vec<u32> {
    (0..=4096u32)
        .map(|i| {
            let x = (BigUint::from(i) * 3276705u32) << (FRAC - 27);
            bigmath::trunc_single(&bigmath::atan(&x)).0
        })
        .collect()
}

/// A_s(0 … 4096) = RZ₂₄(asin(k / 2048 − 1)) (S-0306 Terms, table arcsine).
pub fn asin_table() -> Vec<u32> {
    let s = BigUint::from(1u32) << FRAC;
    let half_pi = bigmath::pi() >> 1u32;
    (0..=4096u32)
        .map(|k| {
            let neg = k < 2048;
            let m = if neg { 2048 - k } else { k - 2048 };
            let v = if m == 2048 {
                bigmath::trunc_single(&half_pi)
            } else if m == 0 {
                F32::ZERO
            } else {
                // asin(x) = atan(x / √(1 − x²)), x = m / 2048
                let x = (BigUint::from(m) << FRAC) / 2048u32;
                let one_minus = &s - ((&x * &x) >> FRAC);
                let root = bigmath::isqrt_big(&(one_minus << FRAC));
                let ratio = (&x << FRAC) / root;
                bigmath::trunc_single(&bigmath::atan(&ratio))
            };
            // spec: S-0306 Terms (RZ₂₄ truncates toward zero, so a negative value keeps the magnitude's truncation)
            if neg && v.0 != 0 {
                v.0 | 0x8000_0000
            } else {
                v.0
            }
        })
        .collect()
}

/// The single-precision value of decimal text.
fn single(s: &str) -> u32 {
    decimal::to_single(&decimal::parse_text(s).expect("decimal")).0
}

/// The text of `tables.rs`.
pub fn render() -> String {
    let mut out = String::new();
    out.push_str(
        "//! Generated tables. Do not edit: regenerate with\n\
         //! `GLASNOST_UPDATE_TABLES=1 cargo test -p glasnost-num regenerate_tables -- --ignored`.\n\
         //! `tables_match_definitions` checks every entry against its definition.\n\n",
    );
    let emit = |out: &mut String, cite: &str, doc: &str, name: &str, v: &[u32]| {
        out.push_str(&format!(
            "// spec: {cite}\n/// {doc}\n#[rustfmt::skip]\npub static {name}: [u32; {}] = [\n",
            v.len()
        ));
        for chunk in v.chunks(8) {
            out.push_str("    ");
            for x in chunk {
                out.push_str(&format!("{x}, "));
            }
            out.push('\n');
        }
        out.push_str("];\n\n");
    };
    emit(
        &mut out,
        "S-0105 F1 (τ(n) for n = 0 … 2048: sin(2πn/8192) truncated toward zero to single precision)",
        "Quarter-wave sine table τ(0 … 2048) as single-precision bit patterns.",
        "SINE_QUARTER",
        &sine_quarter(),
    );
    emit(
        &mut out,
        "S-0204 F3 (T(i) = trunc24(arctan(i σ)) for i = 0 … 4096, the surrogate table)",
        "Arctangent surrogate table T(0 … 4096) as single-precision bit patterns.",
        "ATAN",
        &atan_table(),
    );
    emit(
        &mut out,
        "S-0306 Terms (table arcsine A_s[k] = RZ₂₄(asin(k / 2048 − 1)) for k = 0 … 4096)",
        "Table arcsine A_s(0 … 4096) as single-precision bit patterns.",
        "ASIN",
        &asin_table(),
    );
    out.push_str(&format!(
        "// spec: S-0105 F1 (τ(4096) = 1.2246063538223773 × 10⁻¹⁶, a single)\n\
         /// τ(4096) as a single-precision bit pattern.\n\
         pub const TAU_4096: u32 = {};\n\n\
         // spec: S-0105 F1 (τ(8192) = −2.4492127076447545 × 10⁻¹⁶, a single)\n\
         /// τ(8192) as a single-precision bit pattern.\n\
         pub const TAU_8192: u32 = {};\n",
        single("1.2246063538223773e-16"),
        single("-2.4492127076447545e-16"),
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tables;

    #[test]
    fn tables_match_definitions() {
        assert_eq!(tables::SINE_QUARTER.to_vec(), sine_quarter());
        assert_eq!(tables::ATAN.to_vec(), atan_table());
        assert_eq!(tables::ASIN.to_vec(), asin_table());
        assert_eq!(tables::TAU_4096, single("1.2246063538223773e-16"));
        assert_eq!(tables::TAU_8192, single("-2.4492127076447545e-16"));
    }

    #[allow(clippy::disallowed_types, clippy::excessive_precision)]
    #[test]
    fn exceptions_parse_as_host_singles() {
        assert_eq!(
            single("1.2246063538223773e-16"),
            1.2246063538223773e-16f32.to_bits()
        );
        assert_eq!(
            single("-2.4492127076447545e-16"),
            (-2.4492127076447545e-16f32).to_bits()
        );
        let _ = F32::ZERO;
    }

    #[allow(clippy::disallowed_methods)]
    #[test]
    #[ignore]
    fn regenerate_tables() {
        if std::env::var("GLASNOST_UPDATE_TABLES").is_ok() {
            let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/tables.rs");
            #[allow(clippy::disallowed_methods)]
            std::fs::write(path, render()).expect("write tables.rs");
        }
    }
}
