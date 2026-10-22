//! The build id names the commit (M9 N8b).

#[test]
fn the_build_id_has_the_version_and_the_commit() {
    assert!(glasnost_net::BUILD.starts_with("glasnost "));
    assert!(glasnost_net::BUILD.ends_with(&format!("({})", glasnost_net::COMMIT)));
    // A short hash, with +dirty after it, or `unknown` outside git.
    let c = glasnost_net::COMMIT;
    let hash = c.strip_suffix("+dirty").unwrap_or(c);
    assert!(
        hash == "unknown" || (hash.len() >= 7 && hash.bytes().all(|b| b.is_ascii_hexdigit())),
        "{c}"
    );
}
