//! `"inherits": null` removes the key like any other `null` (#329).

use glasnost_rules::jsonc::Node;
use glasnost_rules::resolve::{Resolver, Source};

fn run(layers: &[&str]) -> Result<Node, Vec<String>> {
    let mut r = Resolver::default();
    for (i, text) in layers.iter().enumerate() {
        let file = format!("layer{i}.jsonc");
        r.apply(&Source {
            layer: "test",
            file: &file,
            text,
        })
        .expect("layer parses");
    }
    match r.finish() {
        Ok((t, _)) => Ok(t),
        Err(e) => Err(e.into_iter().map(|x| x.message).collect()),
    }
}

/// Whether instance `id` of the objects has key `key`.
fn has(t: &Node, id: &str, key: &str) -> bool {
    t.get("objects")
        .and_then(|o| o.get(id))
        .and_then(|i| i.get(key))
        .is_some()
}

const BASE: &str = r#"{"objects": {"^t": {"speed": 9}, "a": {"inherits": ["^t"], "cost": 3}}}"#;

// nocite: our own rules format (PLAN.md 4.6: `null` removes a key), #329
#[test]
fn null_removes_inherits() {
    let t = run(&[BASE, r#"{"objects": {"a": {"inherits": null}}}"#]).expect("loads");
    assert!(has(&t, "a", "cost"));
    assert!(!has(&t, "a", "speed"));
}

// nocite: our own rules format (PLAN.md 4.6: `null` removes a key), #329
#[test]
fn null_inherits_on_a_fresh_instance() {
    let t = run(&[r#"{"objects": {"b": {"inherits": null, "cost": 1}}}"#]).expect("loads");
    assert!(has(&t, "b", "cost"));
}

// nocite: our own rules format (PLAN.md 4.6), #329 (the controls)
#[test]
fn an_empty_list_and_a_null_elsewhere_still_work() {
    let t = run(&[BASE, r#"{"objects": {"a": {"inherits": []}}}"#]).expect("loads");
    assert!(!has(&t, "a", "speed"));
    let t = run(&[BASE, r#"{"objects": {"a": {"cost": null}}}"#]).expect("loads");
    assert!(has(&t, "a", "speed") && !has(&t, "a", "cost"));
    let err = run(&[r#"{"objects": {"b": {"inherits": 3}}}"#]).expect_err("refused");
    assert!(
        err.iter().any(|e| e.contains("inherits must be a list")),
        "{err:?}"
    );
}
