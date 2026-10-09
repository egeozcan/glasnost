//! Hostile-input smoke test (stable, runs in CI): the JSONC reader, its
//! writer and the resolve pipeline get the synthetic ruleset's files, their
//! prefixes and mutated copies. Deeper fuzzing runs with cargo-fuzz
//! (`fuzz/`, target `jsonc`).

// Tests read fixture files from disk and use a seeded generator of their own.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::path::PathBuf;

use glasnost_rules::hostile::run;

fn seeds() -> Vec<Vec<u8>> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data/synthetic/ruleset");
    let mut out: Vec<Vec<u8>> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| std::fs::read(e.unwrap().path()).unwrap())
        .collect();
    out.sort();
    out.push(br#"{"objects": {"a": {"inherits": ["b"]}, "b": {"inherits": ["a"]}}, "types": {"x": {"^y": {"ini": {"+": "1"}}}}}"#.to_vec());
    out
}

/// Bytes that matter to the parser, inserted by the mutator.
const INSERTS: &[u8] = b"{}[]\",:/*-0123456789en\\\n ";

#[test]
fn reader_and_pipeline_survive_mutations() {
    let mut state = 0x5EEDu64;
    let mut next = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 33) as usize
    };
    for seed in seeds() {
        run(&seed);
        for cut in (0..seed.len()).step_by(7) {
            run(&seed[..cut]);
        }
        for _ in 0..300 {
            let mut b = seed.clone();
            for _ in 0..1 + next() % 4 {
                let i = next() % b.len().max(1);
                match next() % 3 {
                    0 if !b.is_empty() => b[i] = INSERTS[next() % INSERTS.len()],
                    1 => b.insert(i.min(b.len()), INSERTS[next() % INSERTS.len()]),
                    _ if !b.is_empty() => {
                        b.remove(i);
                    }
                    _ => {}
                }
            }
            run(&b);
        }
    }
}

#[test]
fn deep_and_wide_input_is_bounded() {
    run("[".repeat(10_000).as_bytes());
    run(format!("{{\"a\": {}}}", "[".repeat(200)).as_bytes());
    let wide: String = (0..5000).map(|i| format!("\"k{i}\": {i},")).collect();
    run(format!("{{{wide}}}").as_bytes());
}
