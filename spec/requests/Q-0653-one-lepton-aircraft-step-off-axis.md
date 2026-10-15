---
id: Q-0653
title: Whether an aircraft's one-lepton forward step moves it when its heading is just off an axis
status: open
related: [S-0501]
answered-by: []
---

## Need

S-0501 revision 9, claim 5, says that an ordinary attacker with *S* > 0 and a converted speed *V*
of at least 10 slows until *d* < 86 and *d* < *S* / 10, then reaches fraction 0 short of its
point. AC24 variant (b), with converted speed about 12 and *S* = 1000, expects fraction 0 at
about 80 to 85 leptons, then *firing* and a shot.

Built literally, F3 and F4 do not always give that. With *S* = 1000 and *V* = 12, F4 brings the
fraction to about *d* / 1000. At *d* = 166 the fraction is 0.166, and F3's step is
⌊12 × 0.166⌋ = 1. In our synthetic test the body heading was 16892, slightly south of east,
because the aircraft was 11 leptons off its point's row. Both F3 components of that step are
then positive and below 1. Each new coordinate is trunc(coordinate + component), which leaves
both coordinates unchanged. The aircraft makes no forward progress, *d* stays 166, the fraction
stays 0.166, and it hovers with a positive fraction. It never reaches *firing* and fires no shot
in 2000 ticks.

The same setup flying west, so that the 1-lepton step has a negative x component, advances one
lepton per tick and behaves as AC24 (b) describes. In general the stall seems possible whenever
the fraction F4 settles at, about *d* / *S*, gives a 1-lepton step before *d* falls below 86.
That is when 2*S* / *V* > 86, at a heading whose step components are both non-negative and below
one lepton.

## Questions

1. With converted speed about 12, *S* = 1000, and an approach heading a few degrees off due east
   (or due south), does an ordinary attacker still reach fraction 0 at about 80 to 85 leptons
   and fire? Or does it hover about 166 leptons short with a positive fraction and never fire?
2. If it does advance in the original, what in the step or the slowing differs from F3 and F4
   as specified? For example, is the step taken in a way that keeps the fractional lepton, or is
   a 1-lepton step otherwise never lost to truncation?
3. Does claim 5's "*V* at least 10" condition need an extra condition on *S* / *V* or on the
   heading?

## Possible observation

Use AC24's map with variant (b). Start the aircraft so that its approach heading is a few
hundred facing units clockwise of due east (both step components positive). Log the position,
fraction and distance per tick from 300 leptons in, and repeat the run heading due west.

## Current workaround

F3 and F4 are followed literally, so the stall described above happens. No code change. The
claim 42 tests use the westward heading for variant (b). The off-axis case is not asserted
either way.
