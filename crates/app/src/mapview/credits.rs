//! The credits counter of the local player's sidebar (S-0416): a shown
//! value that moves toward the house's available credits one step per
//! cycle, with a tick sound for each move.
//!
//! Presentation only (S-0416 claim 5): the simulation never reads the shown
//! value; spending and refunds use the house's credits.

use glasnost_convert::Import;
use glasnost_ini::value;

/// The gap divisor of a step.
const GAP_DIVISOR: i32 = 8; // spec: S-0416 Constants (gap divisor)
/// The largest step, in credits per cycle.
const LARGEST_STEP: i32 = 143; // spec: S-0416 Constants (largest step)
/// The smallest step, in credits per cycle.
const SMALLEST_STEP: i32 = 1; // spec: S-0416 Constants (smallest step)

/// Which credit tick sound a move plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tick {
    /// The list's first entry: the counter rose.
    Rising,
    /// The list's second entry: the counter fell.
    Falling,
}

/// The ticking counter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CreditCounter {
    /// The shown value *D*; 0 at game start.
    // spec: S-0416 claim 3 (the shown value starts at 0)
    pub shown: i32,
}

impl CreditCounter {
    /// The counter's update for one cycle that is not frozen, aiming at the
    /// local player's available credits `available`. Returns the tick that
    /// plays, if the counter moved.
    pub fn update(&mut self, available: i32) -> Option<Tick> {
        // spec: S-0416 claim 1 (the target is the available credits, 0 when 0 or less)
        let target = available.max(0);
        let gap = target.abs_diff(self.shown).min(i32::MAX as u32) as i32;
        if gap == 0 {
            return None;
        }
        // spec: S-0416 claim 2 (s = min(143, max(1, ⌊|G − D| / 8⌋)), never past the target)
        let step = (gap / GAP_DIVISOR).clamp(SMALLEST_STEP, LARGEST_STEP);
        if target > self.shown {
            self.shown += step;
            Some(Tick::Rising)
        } else {
            self.shown -= step;
            Some(Tick::Falling)
        }
    }
}

/// The credit tick sounds (S-0416 Import mapping): `CreditTicks` of the
/// audio-visual section, a list of sound names; empty when absent.
pub fn credit_tick_sounds(import: &Import) -> Vec<Vec<u8>> {
    import
        .outcome
        .globals
        .get(&b"AudioVisual"[..])
        .and_then(|s| s.get(&b"CreditTicks"[..]))
        .and_then(|k| k.value.as_deref())
        // spec: S-0003 claim 1 (lists read with the usual 127-byte text limit)
        .map(|v| {
            value::list(v, value::COMMON_TEXT_LIMIT)
                .into_iter()
                .map(<[u8]>::to_vec)
                .collect()
        })
        .unwrap_or_default()
}

/// The sound a tick plays from the credit tick list, at half volume:
/// the first entry rising, the second falling, none when the list has fewer
/// than two entries.
pub fn tick_sound(sounds: &[Vec<u8>], tick: Tick) -> Option<&[u8]> {
    // spec: S-0416 claim 4 (fewer than two entries: no tick sound at all)
    if sounds.len() < 2 {
        return None;
    }
    // spec: S-0416 claim 4 (first entry rising, second falling)
    Some(match tick {
        Tick::Rising => &sounds[0],
        Tick::Falling => &sounds[1],
    })
}

/// The tick sound volume, as a fraction (numerator, denominator) of full
/// volume.
pub const TICK_VOLUME: (u32, u32) = (1, 2); // spec: S-0416 Constants (tick sound volume one half)

#[cfg(test)]
mod tests {
    use super::*;

    fn run(from: i32, target: i32) -> Vec<i32> {
        let mut c = CreditCounter { shown: from };
        let mut seen = Vec::new();
        while c.update(target).is_some() {
            seen.push(c.shown);
        }
        seen
    }

    #[test]
    fn claim_1_target_floors_at_zero() {
        // spec: S-0416 claim 1
        let mut c = CreditCounter { shown: 3 };
        for _ in 0..10 {
            c.update(-500);
        }
        assert_eq!(c.shown, 0);
        assert_eq!(c.update(-1), None);
        let mut c = CreditCounter::default();
        for _ in 0..100 {
            c.update(250);
        }
        assert_eq!(c.shown, 250);
    }

    #[test]
    fn claim_2_step_rule() {
        // spec: S-0416 claim 2 (Formulas example: a rise of 1000 moves 125, 109, 95, 83, …)
        let seen = run(0, 1000);
        let steps: Vec<i32> = std::iter::once(seen[0])
            .chain(seen.windows(2).map(|w| w[1] - w[0]))
            .collect();
        assert_eq!(&steps[..4], &[125, 109, 95, 83]);
        // the last 8 credits by 1 per cycle, never overshooting
        assert!(steps[steps.len() - 8..].iter().all(|&s| s == 1));
        assert!(seen.iter().all(|&v| v <= 1000));
        assert_eq!(*seen.last().unwrap(), 1000);
        // a gap of 1144 or more closes by 143
        let seen = run(0, 5000);
        assert_eq!(&seen[..3], &[143, 286, 429]);
        // a fall
        let seen = run(1000, 0);
        assert_eq!(seen[0], 875);
        assert_eq!(*seen.last().unwrap(), 0);
        // a gap of 1 closes in one cycle
        assert_eq!(run(0, 1), vec![1]);
    }

    #[test]
    fn claim_2_target_change_while_moving() {
        // spec: S-0416 claim 2 (a change of the target changes the remaining gap)
        let mut c = CreditCounter::default();
        c.update(1000);
        assert_eq!(c.shown, 125);
        assert_eq!(c.update(0), Some(Tick::Falling));
        assert_eq!(c.shown, 125 - 15);
    }

    #[test]
    fn claim_3_starts_at_zero() {
        // spec: S-0416 claim 3 (D3: counts up from 0 by 143 at first)
        let mut c = CreditCounter::default();
        assert_eq!(c.shown, 0);
        c.update(10000);
        assert_eq!(c.shown, 143);
    }

    #[test]
    fn claim_4_tick_sounds() {
        // spec: S-0416 claim 4
        let two = vec![b"up".to_vec(), b"down".to_vec()];
        assert_eq!(tick_sound(&two, Tick::Rising), Some(&b"up"[..]));
        assert_eq!(tick_sound(&two, Tick::Falling), Some(&b"down"[..]));
        let one = vec![b"up".to_vec()];
        assert_eq!(tick_sound(&one, Tick::Rising), None);
        assert_eq!(tick_sound(&[], Tick::Falling), None);
        let mut c = CreditCounter::default();
        assert_eq!(c.update(5), Some(Tick::Rising));
        assert_eq!(c.update(0), Some(Tick::Falling));
        assert_eq!(TICK_VOLUME, (1, 2));
    }
}
