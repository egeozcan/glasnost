//! `quirks_with` reads a layer's `globals.quirks` as the resolve pipeline
//! reads it (#331): a `null` switch takes its default, a `null` `quirks` or
//! `globals` resets every switch, and a `quirks` or `globals` that is not an
//! object is refused.

use glasnost_rules::layers::quirks_with;
use glasnost_rules::quirks::Quirks;
use glasnost_rules::resolve::Source;
use glasnost_rules::resolve_ruleset;

const BASE: &str = r#"{"schema": 1}"#;
const OFF: &str = r#"{"globals": {"quirks": {"deadlock_no_yield": false}}}"#;

fn src<'a>(file: &'a str, text: &'a str) -> Source<'a> {
    Source {
        layer: "test",
        file,
        text,
    }
}

/// The pipeline's switches for the base tree and these layers.
fn pipeline(layers: &[&str]) -> Result<Quirks, String> {
    let names: Vec<String> = (0..layers.len()).map(|i| format!("l{i}.jsonc")).collect();
    let mut srcs = vec![src("base.jsonc", BASE)];
    for (t, f) in layers.iter().zip(&names) {
        srcs.push(src(f, t));
    }
    resolve_ruleset(None, &srcs)
        .map(|r| r.ruleset.globals.quirks)
        .map_err(|e| format!("{e}"))
}

/// `quirks_with` over the defaults, one layer after another.
fn direct(layers: &[&str]) -> Result<Quirks, String> {
    let mut q = Quirks::default();
    for t in layers {
        q = quirks_with(q, &src("l.jsonc", t))?;
    }
    Ok(q)
}

fn off() -> Quirks {
    Quirks {
        deadlock_no_yield: false,
        ..Quirks::default()
    }
}

// nocite: our own rules format (PLAN.md 4.6: a layer sets a switch), #331 (control)
#[test]
fn a_plain_switch_agrees() {
    assert_eq!(pipeline(&[OFF]).unwrap(), off());
    assert_eq!(direct(&[OFF]).unwrap(), off());
}

// nocite: our own rules format (PLAN.md 4.6: `null` removes a key, so the switch takes its default), #331
#[test]
fn a_null_switch_takes_its_default() {
    let null = r#"{"globals": {"quirks": {"deadlock_no_yield": null}}}"#;
    assert_eq!(pipeline(&[OFF, null]).unwrap(), Quirks::default());
    assert_eq!(direct(&[OFF, null]).unwrap(), Quirks::default());
}

// nocite: our own rules format (PLAN.md 4.6: `null` removes a key, so every switch takes its default), #331
#[test]
fn a_null_quirks_or_globals_resets_every_switch() {
    for null in [
        r#"{"globals": {"quirks": null}}"#,
        r#"{"globals": null}"#,
    ] {
        assert_eq!(pipeline(&[OFF, null]).unwrap(), Quirks::default(), "{null}");
        assert_eq!(direct(&[OFF, null]).unwrap(), Quirks::default(), "{null}");
    }
}

// nocite: our own rules format (PLAN.md 4.6: `globals` and `globals.quirks` are objects), #331
#[test]
fn a_quirks_or_globals_that_is_not_an_object_is_refused() {
    for bad in [
        r#"{"globals": {"quirks": false}}"#,
        r#"{"globals": {"quirks": [1]}}"#,
        r#"{"globals": 3}"#,
    ] {
        assert!(pipeline(&[bad]).is_err(), "pipeline: {bad}");
        assert!(direct(&[bad]).is_err(), "quirks_with: {bad}");
    }
}
