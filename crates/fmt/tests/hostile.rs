//! Hostile-input smoke test (stable toolchain, runs in CI): every parser
//! gets every synthetic fixture, every prefix of it, and many randomly
//! mutated copies. Any panic or overflow trap fails the test. Deeper
//! fuzzing runs with cargo-fuzz (`fuzz/`, nightly, not in CI).

use glasnost_fmt::fixture::{generate, registry};
use glasnost_fmt::hostile::{ALL, run};
use glasnost_fmt::synth::SynthRng;

/// Mutated copies per (fixture, target) pair.
const MUTATIONS: usize = 200;

#[test]
fn parsers_survive_mutated_fixtures() {
    let fixtures = generate(&registry()).expect("fixture paths are unique");
    let mut rng = SynthRng::new(0x5EED);
    for f in &fixtures {
        for &target in ALL {
            run(target, &f.bytes);
            // Prefixes at a stride, so large fixtures stay quick.
            let stride = (f.bytes.len() / 64).max(1);
            for cut in (0..f.bytes.len()).step_by(stride) {
                run(target, &f.bytes[..cut]);
            }
            for _ in 0..MUTATIONS {
                let mut bytes = f.bytes.clone();
                mutate(&mut bytes, &mut rng);
                run(target, &bytes);
            }
        }
    }
}

#[test]
fn parsers_survive_random_bytes() {
    let mut rng = SynthRng::new(7);
    for len in [
        0usize, 1, 2, 3, 4, 7, 8, 16, 31, 64, 92, 127, 128, 300, 768, 802, 1024, 4096,
    ] {
        for _ in 0..8 {
            let mut bytes = vec![0u8; len];
            rng.fill(&mut bytes);
            for &target in ALL {
                run(target, &bytes);
            }
        }
    }
}

/// Flips, overwrites, inserts or deletes a few bytes, or writes an extreme
/// 32-bit value (0, 1, 0x7FFF_FFFF, 0xFFFF_FFFF, ...) somewhere.
fn mutate(bytes: &mut Vec<u8>, rng: &mut SynthRng) {
    let edits = 1 + rng.next_u64() % 4;
    for _ in 0..edits {
        if bytes.is_empty() {
            bytes.push(rng.next_u64() as u8);
            continue;
        }
        let at = (rng.next_u64() % bytes.len() as u64) as usize;
        match rng.next_u64() % 5 {
            0 => bytes[at] ^= 1 << (rng.next_u64() % 8),
            1 => bytes[at] = rng.next_u64() as u8,
            2 => bytes.insert(at, rng.next_u64() as u8),
            3 => {
                bytes.remove(at);
            }
            _ => {
                const EXTREMES: [u32; 6] = [0, 1, 0xFF, 0xFFFF, 0x7FFF_FFFF, 0xFFFF_FFFF];
                let v = EXTREMES[(rng.next_u64() % 6) as usize].to_le_bytes();
                for (i, b) in v.iter().enumerate() {
                    if let Some(slot) = bytes.get_mut(at + i) {
                        *slot = *b;
                    }
                }
            }
        }
    }
}
