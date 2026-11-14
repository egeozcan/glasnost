//! `quirks.decimal_conversion_rounding` (S-0003 Edge cases, #324) as the
//! simulation reads a ruleset: on (the default), every decimal by claim 6's
//! rule; off, correctly rounded to the nearest single, ties to even,
//! subnormals included. Versus items (claim 13, already doubles written as
//! their exact decimals by the importer) are read as before in both.

use glasnost_num::{F64, Rounding};
use glasnost_rules::jsonc;
use glasnost_rules::resolve::Source;
use glasnost_rules::schema::SCHEMA_VERSION;

const RZ: Rounding = Rounding::TowardZero;
const RN: Rounding = Rounding::Nearest;

fn resolve(text: &str) -> glasnost_rules::Resolved {
    let base = jsonc::object([(
        "schema".to_owned(),
        jsonc::integer(i128::from(SCHEMA_VERSION)),
    )]);
    glasnost_rules::resolve_ruleset(
        Some((base, Default::default())),
        &[Source {
            layer: "mod 1: test",
            file: "test.jsonc",
            text,
        }],
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

/// A layer setting the switch (or leaving it absent) and four decimals.
fn layer(switch: Option<bool>, values: [&str; 4]) -> String {
    let quirks = match switch {
        Some(b) => format!("\"quirks\": {{\"decimal_conversion_rounding\": {b}}}"),
        None => "\"quirks\": {}".to_owned(),
    };
    format!(
        "{{\"globals\": {{{quirks}}}, \"sim\": {{\"general\": {{\"close_enough\": \"{}\", \"path_delay\": \"{}\", \"crush_range\": \"{}\", \"guard_mode_stray\": \"{}\"}}, \"warheads\": {{\"w\": {{\"verses\": [\"0.1\", \"0.299999999999999988897769753748434595763683319091796875\"]}}}}}}}}",
        values[0], values[1], values[2], values[3]
    )
}

fn values(r: &glasnost_rules::Resolved, mode: Rounding) -> [F64; 4] {
    let g = &r.ruleset.sim.general;
    [
        &g.close_enough,
        &g.path_delay,
        &g.crush_range,
        &g.guard_mode_stray,
    ]
    .map(|d| d.as_ref().expect("set").value(mode))
}

fn int(v: i64) -> F64 {
    F64::from_i64(v, RZ)
}

/// The single 2^-149, the smallest subnormal.
// nocite: binary64 layout of the smallest binary32 subnormal
fn smallest_subnormal() -> F64 {
    F64::from_parts(false, 1, -149, RZ)
}

// spec: S-0003 Edge cases (quirks.decimal_conversion_rounding off: correctly rounded, ties to even, subnormals included; 16777219 gives 16777220)
#[test]
fn off_reads_every_decimal_correctly_rounded() {
    let r = resolve(&layer(
        Some(false),
        ["16777219", "8388609.5", "33554435", "1e-45"],
    ));
    for mode in [RZ, RN] {
        assert_eq!(
            values(&r, mode),
            [
                int(16777220),
                int(8388610),
                int(33554436),
                smallest_subnormal()
            ],
            "{mode:?}"
        );
    }
    let r = resolve(&layer(
        Some(false),
        ["-8388609.5", "8388609.5%", "8388608.5", "0.1"],
    ));
    let [a, b, c, d] = values(&r, RZ);
    assert_eq!(a, int(-8388610));
    // spec: S-0003 claim 7 (with `%`, the single times the percent factor, rounded by the mode)
    assert_eq!(
        b,
        int(8388610).mul(glasnost_num::decimal::percent_factor(), RZ)
    );
    // A tie with an even kept part stays below in both settings.
    assert_eq!(c, int(8388608));
    assert_eq!(
        d,
        glasnost_num::decimal::parse("0.1", Default::default(), RZ).unwrap()
    );
    // The single-precision reading narrows the same single.
    let g = &r.ruleset.sim.general;
    assert_eq!(
        g.close_enough.as_ref().unwrap().single_value(RZ),
        int(-8388610)
    );
}

// spec: S-0003 Edge cases (quirks.decimal_conversion_rounding on, the default: claim 6's rule), S-0003 Formulas
#[test]
fn on_and_absent_keep_claim_six() {
    for switch in [None, Some(true)] {
        let r = resolve(&layer(
            switch,
            ["16777219", "8388609.5", "33554435", "1e-45"],
        ));
        assert_eq!(
            values(&r, RZ),
            [int(16777218), int(8388609), int(33554432), F64::ZERO],
            "{switch:?}"
        );
    }
}

// spec: S-0003 claim 13 (versus items are the importer's doubles, written as exact decimals), S-0003 Edge cases
#[test]
fn versus_items_keep_their_text() {
    for switch in [None, Some(false)] {
        let r = resolve(&layer(switch, ["1", "1", "1", "1"]));
        let w = &r.ruleset.sim.warheads["w"];
        let texts: Vec<&str> = w
            .verses
            .as_ref()
            .unwrap()
            .iter()
            .map(|d| d.0.as_str())
            .collect();
        assert_eq!(
            texts,
            [
                "0.1",
                "0.299999999999999988897769753748434595763683319091796875"
            ],
            "{switch:?}"
        );
    }
}
