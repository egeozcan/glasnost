//! Five imported keys are stored in two fields each; a ruleset whose two
//! copies differ is refused (#323).

use glasnost_rules::resolve::Source;
use glasnost_rules::resolve_ruleset;

fn resolve(sim: &str) -> Result<(), String> {
    let text = format!(r#"{{"schema": 1, "sim": {sim}}}"#);
    let src = Source {
        layer: "test",
        file: "layer0.jsonc",
        text: &text,
    };
    resolve_ruleset(None, &[src])
        .map(|_| ())
        .map_err(|e| format!("{e}"))
}

/// Each pair: one copy alone, then both alike.
const PAIRS: [(&str, &str, &str, &str); 5] = [
    (
        r#"{"buildings": {"V": {"cloning": true}}}"#,
        r#"{"buildings": {"V": {"econ": {"cloning": true}}}}"#,
        r#"{"buildings": {"V": {"cloning": true, "econ": {"cloning": true}}}}"#,
        "cloning",
    ),
    (
        r#"{"buildings": {"V": {"naval": true}}}"#,
        r#"{"buildings": {"V": {"combat": {"naval": true}}}}"#,
        r#"{"buildings": {"V": {"naval": true, "combat": {"naval": true}}}}"#,
        "naval",
    ),
    (
        r#"{"general": {"lightning_warhead": "W"}}"#,
        r#"{"general": {"superweapons": {"lightning_warhead": "W"}}}"#,
        r#"{"general": {"lightning_warhead": "W", "superweapons": {"lightning_warhead": "W"}}}"#,
        "lightning_warhead",
    ),
    (
        r#"{"general": {"weather_con_bolt_explosion": "A"}}"#,
        r#"{"general": {"superweapons": {"weather_con_bolt_explosion": "A"}}}"#,
        r#"{"general": {"weather_con_bolt_explosion": "A", "superweapons": {"weather_con_bolt_explosion": "A"}}}"#,
        "weather_con_bolt_explosion",
    ),
    (
        r#"{"general": {"c4_warhead": "W"}}"#,
        r#"{"general": {"economy": {"c4_warhead": "W"}}}"#,
        r#"{"general": {"c4_warhead": "W", "economy": {"c4_warhead": "W"}}}"#,
        "c4_warhead",
    ),
];

// nocite: our own rules format (PLAN.md 4.6 step 5: semantic validation), #323 (the controls: both copies alike, or neither)
#[test]
fn copies_that_agree_are_accepted() {
    resolve("{}").expect("neither copy set");
    for (_, _, both, key) in PAIRS {
        resolve(both).unwrap_or_else(|e| panic!("{key}: {e}"));
    }
}

// spec: S-0214 and S-0404 Import mapping (one `Cloning` key: rally-capable, cloning building of claim 14); S-0308 Import mapping (`Naval`, row 13); S-0303 and S-0513 Import mapping (`LightningWarhead`, `WeatherConBoltExplosion`); S-0302, S-0303, S-0414 and S-0513 Import mapping (`C4Warhead`): one key each, so its two copies must agree (#323)
#[test]
fn copies_that_differ_are_refused() {
    for (one, other, _, key) in PAIRS {
        for text in [one, other] {
            let err = resolve(text).expect_err(text);
            assert!(err.contains(key), "{key}: {err}");
        }
    }
}
