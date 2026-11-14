//! Inheriting from an instance a layer removed is an error, like an
//! unknown parent (#330).

use glasnost_rules::resolve::{Resolver, Source};

fn finish(layers: &[&str]) -> Result<(), Vec<String>> {
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
    r.finish()
        .map(|_| ())
        .map_err(|e| e.into_iter().map(|x| x.message).collect())
}

// nocite: our own rules format (PLAN.md 4.6: `inherits` names instances; `null` removes one), #330
#[test]
fn a_parent_that_never_existed_is_an_error() {
    let err = finish(&[r#"{"objects": {"^a": {"speed": 9}, "c": {"inherits": ["^a", "^b"]}}}"#])
        .expect_err("refused");
    assert!(err.iter().any(|m| m.contains("unknown parent")), "{err:?}");
}

// nocite: our own rules format (PLAN.md 4.6: `inherits` names instances; `null` removes one), #330
#[test]
fn a_parent_removed_by_a_later_layer_is_an_error() {
    let err = finish(&[
        r#"{"objects": {"^a": {"speed": 9}, "^b": {"cost": 5}, "c": {"inherits": ["^a", "^b"]}}}"#,
        r#"{"objects": {"^b": null}}"#,
    ])
    .expect_err("refused");
    assert!(
        err.iter()
            .any(|m| m.contains("objects.c") && m.contains("\"^b\"") && m.contains("removed")),
        "{err:?}"
    );
}

// nocite: our own rules format (PLAN.md 4.6: an instance is an object), #330
#[test]
fn a_parent_that_is_not_an_object_is_an_error() {
    let err = finish(&[r#"{"objects": {"^a": {"speed": 9}, "^b": 3, "c": {"inherits": ["^a", "^b"]}}}"#])
        .expect_err("refused");
    assert!(
        err.iter()
            .any(|m| m.contains("objects.c") && m.contains("\"^b\"") && m.contains("not an object")),
        "{err:?}"
    );
}
