//! How the live game draws a projectile in flight (S-0306 claims 28 to 32,
//! F14 and F15): its type's art, its sprite image frame and its shadow
//! height. Presentation only: nothing here changes the game state.

use std::collections::BTreeMap;

use glasnost_convert::Import;
use glasnost_convert::art::ProjectileLook;
use glasnost_convert::kinds::Kind;
use glasnost_map::CellXY;
use glasnost_sim::world::{Pos, World};

/// The deck height of a high bridge over the terrain (S-0306 claim 32).
const DECK_HEIGHT: i32 = 416; // spec: S-0306 claim 32 (the deck height of 4 height levels, 416 leptons)
/// The rate field's hold when it is 0 (F15: the countdown wraps to 255).
const WRAPPED_RATE: i32 = 256; // spec: S-0306 F15 (with Rₐ = 0 the countdown wraps to 255 and a advances every 256 turns)
/// The rotating sprite's image frame base of F14.
const ROTATING_BASE: i32 = 28; // spec: S-0306 F14 (image frame = (28 − ψ) mod 32)
/// The facing steps of a rotating sprite (F14).
const STEPS: i32 = 32; // spec: S-0306 F14 (32 directions)

/// Each simulation projectile type's import registration index (by name,
/// ASCII case folded, the first registered winning: S-0003 claim 14), in
/// the order of `names`.
pub fn entries(import: &Import, names: &[Vec<u8>]) -> Vec<Option<usize>> {
    let mut by_name: BTreeMap<Vec<u8>, usize> = BTreeMap::new();
    if let Some(reg) = import.outcome.registries.get(&Kind::Projectiles) {
        for (idx, t) in reg.iter().enumerate() {
            // spec: S-0003 claim 14 (names match with ASCII case folding; the first registered wins)
            by_name.entry(t.name.to_ascii_uppercase()).or_insert(idx);
        }
    }
    names
        .iter()
        .map(|n| by_name.get(&n.to_ascii_uppercase()).copied())
        .collect()
}

/// Whether a type's image has an animation range (S-0306 claim 29: its low
/// or its high animation field is not 0).
pub fn animated(look: &ProjectileLook) -> bool {
    // spec: S-0306 claim 29 (animated when the low or the high animation field is not 0)
    look.anim_low != 0 || look.anim_high != 0
}

/// The animation counter *a* after `turns` turns (S-0306 F15): it starts at
/// 0 with the countdown at the rate; each turn the countdown decreases
/// modulo 256 and, on reaching 0, *a* steps (one up modulo 256, or to the
/// low field when above the high field) and the countdown restarts.
pub fn counter(lo: u8, hi: u8, rate: u8, turns: i32) -> u8 {
    // spec: S-0306 F15 (a step every Rₐ turns; Rₐ = 0 every 256 turns)
    let hold = if rate == 0 {
        WRAPPED_RATE
    } else {
        i32::from(rate)
    };
    let mut steps = (turns.max(0) / hold) as usize;
    // spec: S-0306 F15 (a = a + 1 modulo 256; above aₕᵢ, unsigned, it is set to aₗₒ)
    let step = |a: u8| {
        let n = a.wrapping_add(1);
        if n > hi { lo } else { n }
    };
    // The counter has at most 256 values: walk until a value repeats, then
    // skip whole cycles.
    let mut seen: [Option<usize>; 256] = [None; 256];
    let mut a = 0u8;
    let mut k = 0usize;
    while k < steps {
        if let Some(first) = seen[usize::from(a)] {
            let cycle = k - first;
            steps = k + (steps - k) % cycle;
            seen = [None; 256];
            if k >= steps {
                break;
            }
        }
        seen[usize::from(a)] = Some(k);
        a = step(a);
        k += 1;
    }
    a
}

/// A sprite body's image frame (S-0306 claim 29): the animation counter
/// when the image is animated, else (28 − ψ) mod 32 with the rotating
/// flag, else 0.
pub fn sprite_frame(look: &ProjectileLook, psi: u8, turns: i32) -> i32 {
    if animated(look) {
        // spec: S-0306 claim 29 (animated: the counter a of F15, whatever the rotating flag says)
        i32::from(counter(
            look.anim_low,
            look.anim_high,
            look.anim_rate,
            turns,
        ))
    } else if look.rotates {
        // spec: S-0306 claim 29 and F14 (rotating: (28 − ψ) mod 32, non-negative remainder)
        (ROTATING_BASE - i32::from(psi)).rem_euclid(STEPS)
    } else {
        // spec: S-0306 claim 29 (neither: image frame 0)
        0
    }
}

/// A sprite projectile's shadow height η and the world height of the
/// ground its shadow lies on (S-0306 claim 32): its height above the
/// terrain, or above the deck over a high bridge's cell once at least the
/// deck height up; `None` unless η is above 0.
pub fn shadow(w: &World, pos: Pos) -> Option<(i32, i32)> {
    let terrain = w.surface_at(pos.x, pos.y);
    let above = pos.z - terrain;
    // spec: S-0204 claim 2 (the containing cell)
    let cell = CellXY::new(pos.x.div_euclid(256), pos.y.div_euclid(256));
    // spec: S-0306 claim 32 (over a high bridge's cell at or above the deck height: above the deck, the shadow on the deck)
    let (eta, ground) = if w.high_bridge(cell) && above >= DECK_HEIGHT {
        (above - DECK_HEIGHT, terrain + DECK_HEIGHT)
    } else {
        (above, terrain)
    };
    // spec: S-0306 claim 32 (a shadow only while η is greater than 0)
    (eta > 0).then_some((eta, ground))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(lo: u8, hi: u8, rate: u8, n: i32) -> Vec<u8> {
        (0..n).map(|t| counter(lo, hi, rate, t)).collect()
    }

    /// spec: S-0306 F15 and scenario P42 part c ((2, 5, 3): 0 first, then 1 … 5, 2 … 5, each held 3 turns; (0, 3, 1): 0 to 3 each turn; (5, 2, 1): 0, 1, 2, then 5 for good; (1, 4, 0): a change every 256 turns)
    #[test]
    fn animation_counter() {
        let held = |v: &[u8]| v.iter().flat_map(|&x| [x; 3]).collect::<Vec<u8>>();
        assert_eq!(run(2, 5, 3, 30), held(&[0, 1, 2, 3, 4, 5, 2, 3, 4, 5]));
        assert_eq!(run(0, 3, 1, 9), [0, 1, 2, 3, 0, 1, 2, 3, 0]);
        assert_eq!(run(5, 2, 1, 7), [0, 1, 2, 5, 5, 5, 5]);
        assert_eq!(counter(5, 2, 1, 100_000), 5);
        assert_eq!(counter(1, 4, 0, 255), 0);
        assert_eq!(counter(1, 4, 0, 256), 1);
        assert_eq!(counter(1, 4, 0, 256 * 5), 1);
        // aₕᵢ = 255: the step after 255 gives 0, not aₗₒ.
        assert_eq!(counter(7, 255, 1, 255), 255);
        assert_eq!(counter(7, 255, 1, 256), 0);
        // Long flights skip whole cycles and agree with the plain walk.
        for t in [997, 1000, 4096, 70_001] {
            let mut a = 0u8;
            for _ in 0..t / 3 {
                let n = a.wrapping_add(1);
                a = if n > 5 { 2 } else { n };
            }
            assert_eq!(counter(2, 5, 3, t), a, "{t}");
        }
    }

    /// spec: S-0306 claim 29 (animated wins over rotating; rotating (28 − ψ) mod 32: 0 up the screen, 8 left, 16 down, 24 right; else 0)
    #[test]
    fn sprite_frames() {
        let rot = ProjectileLook {
            rotates: true,
            ..ProjectileLook::default()
        };
        assert_eq!(sprite_frame(&rot, 28, 0), 0);
        assert_eq!(sprite_frame(&rot, 8, 0), 20);
        assert_eq!(sprite_frame(&rot, 0, 0), 28);
        assert_eq!(sprite_frame(&rot, 29, 0), 31);
        let anim = ProjectileLook {
            anim_low: 2,
            anim_high: 5,
            anim_rate: 3,
            ..rot.clone()
        };
        assert_eq!(sprite_frame(&anim, 8, 0), 0);
        assert_eq!(sprite_frame(&anim, 8, 3), 1);
        assert_eq!(sprite_frame(&ProjectileLook::default(), 13, 50), 0);
    }
}
