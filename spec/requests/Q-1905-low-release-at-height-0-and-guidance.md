---
id: Q-1905
title: Does an aircraft in the low release turn toward its destination while it goes on at height 0?
status: open
related: [S-0501]
answered-by: []
---

## Need

S-0501 revision 55, claim 48 (c), describes the *low release*: an aircraft released at most 20
leptons above its pad reaches height 0 in its first movement update. With a target speed fraction
below 1 its held height stays 0, and "it goes on at height 0, listed in the ground layer of each
cell it is in, until claim 6's landing trigger". Claim 48 (e) and scenario AC30 part k (7) (b) build
on this: the aircraft is expected at height 0 "at the end of every update until its landing
starts".

Claim 42 gives guidance (the body facing requested toward the flight point) only after a movement
tick of an aircraft that "is above the terrain". At height 0 the low release is not above the
terrain, so by claim 42 nothing turns it. It goes straight on along the body facing it had at its
release. If the destination is not straight ahead, it passes beside it, the landing trigger of
claim 6 does not fire, and once claim 5 makes its target fraction 1 (its distance having grown past
the slowdown distance) it climbs by claim 48 (b)'s third case and is guided from then on. AC30 part
k (7) (b) does not say in which direction the destination lies relative to the released aircraft's
facing, so the scenario's expected outcome holds only if the destination happens to be ahead of
it, or if some rule turns the aircraft at height 0.

The engine has to decide whether the low release is steered while it stays at height 0.

## Questions

1. While a low-released aircraft goes on at height 0 with a target fraction below 1, does its body
   facing turn toward its destination? If it does, from which movement update on, and by which
   rule (claim 42's guidance at height 0, or another)?
2. If it does not turn: with a destination 512 to 767 leptons away and not straight ahead, does it
   go straight on until its target fraction reaches 1 and then climb, as claims 5, 42 and 48 (b)
   give together? Or does something else happen first (a landing, a stop)?
3. Does the outcome depend on the facing the aircraft has at its release and on the destination's
   direction, so that AC30 part k (7) (b) needs a stated direction?

## Possible observation

AC30 part k (7) (b) as written, run twice from a fresh release: once with the destination straight
ahead along the released aircraft's facing, and once with it at a right angle to that facing, both
512 to 767 leptons away. Record per tick the aircraft's height, position and body facing until its
landing starts or it climbs.

## Current workaround

Claim 42 as written, which is our design until this is answered: no guidance at height 0, so the
low release goes straight on along its body facing. If it passes close enough over its destination
cell, claim 6's trigger starts its landing at height 0. Otherwise it climbs once its target
fraction reaches 1 (claim 48 (b)). Its object-present flag follows claim 48 (e) either way.
Tests: `claim_48_e_low_release_at_height_0_carries_its_flag` (destination straight ahead) and
`claim_48_e_low_release_off_its_heading_goes_straight_on` in
`crates/deterministic/sim/tests/claims_s0501_released.rs`.
