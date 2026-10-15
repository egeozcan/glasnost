---
id: Q-0690
title: Precision of the hover bob constants and of the hover keys' engine defaults
status: answered
related: [S-0503, S-0003]
answered-by: [S-0503]
---

## Need

S-0503 F6 gives the bob as *u* = trunc(2 × sin(ϱ × 2π / *P*ᵈ) + *A*), with *P*ᵈ = *m* × *q* × 900
and *m* = 1.1 for an even object number. The formula says every operation is in double precision
under the round-toward-zero mode, but it does not say:

- which double stands for 2π (the nearest double, the double nearest π doubled, or a
  round-toward-zero value), nor which double stands for 1.1;
- the order of evaluation of ϱ × 2π / *P*ᵈ: (ϱ × 2π) / *P*ᵈ, or ϱ × (2π / *P*ᵈ);
- the order of *m* × *q* × 900.

The Import mapping also gives the engine defaults of the hover keys as 30.0, 1.3, 0.03 and 0.8
when no layer gives the key. The keys themselves are read as singles widened to double
(S-0003 claim 6). It does not say whether the engine defaults are those singles or the doubles
nearest the decimals (for 1.3 and 0.03 the two differ).

The simulation code in `crates/deterministic/sim/src/hover.rs` (the bob term) and
`crates/deterministic/sim/src/rules.rs` (the hover defaults) needs these.

## Questions

1. In F6, is 2π the double nearest 2π? Is 1.1 the double nearest 1.1, or the single nearest 1.1
   widened?
2. Is ϱ × 2π / *P*ᵈ formed as (ϱ × 2π) / *P*ᵈ, and *P*ᵈ as (*m* × *q*) × 900?
3. When no rules layer gives `HoverBob`, `HoverBoost`, `HoverAcceleration`, `HoverBrake` or
   `HoverDampen`, is the value used the single nearest the default decimal (as if the key were
   given with that text), or the double nearest it?

## Possible observation

S-0503 scenario H4 traces two resting hovers of opposite object-number parity for 2000 ticks.
A tick on which trunc(2 × sin(...) + *A*) lands within one unit in the last place of an integer
separates the readings of questions 1 and 2. For question 3, a run without the hover keys
compared with one that gives them as `HoverAcceleration=0.03` and so on: an F2 ramp over many
ticks shows a different arrival tick when the defaults differ from the singles.

## Current workaround

2π is the double nearest 2π, as S-0306 F7 forms it. 1.1 is the double nearest 1.1. The products
and quotients are formed left to right: (*m* × *q*) × 900 and (ϱ × 2π) / *P*ᵈ. The engine
defaults are read as if the key were given with the default text (singles widened to double).
Each choice is marked in the code with this request's ID.
