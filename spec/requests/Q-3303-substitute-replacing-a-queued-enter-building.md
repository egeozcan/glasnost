---
id: Q-3303
title: A substitute cell that replaces a walker's building destination before its queued enter starts
status: answered
related: [S-0208, S-0212, S-0215, S-0605]
answered-by: [S-0208]
---

## Need

S-0208 claim 26 (r71, Q-3264 question 1) says that when S-0212 claim 24 adopts a substitute cell for
the cell a walker's search looks for, the walker is given that cell as its destination by an
assignment like any other, and a destination that names an object is replaced. It heads for the
object again only when a later assignment gives it that object again.

Built as written, this breaks S-0605 claim 16's Edge case for `quirks.enter_nearest_survives_absorption`
(on): "a neutral passenger let out walks back in at its next guard processing". The test
`claims_s0605.rs` `claim_16_back_in_after_release_both_positions` fails, and a trace on synthetic
rules shows why (building's top-left cell (24, 24), the walker let out next to it):

- tick 38: the guard processing of S-0215 claim 24 (enter-nearest instruction) gives the walker the
  building as its destination and queues *enter* without an immediate start. The walker's next
  search runs in the same tick while *guard* is still current. Its mission target (S-0209 Terms)
  is none, so row WP4 of S-0209 claim 4 does not apply and the building cell reads as *blocked*.
  Claim 24's building case adopts the substitute (23, 25). Under claim 26 that cell replaces the
  building as the walker's destination.
- tick 48: the walker reaches (23, 25).
- tick 74: its walk ends there (S-0208 claim 16), with no destination. *Enter* is now current, and
  the walker holds the building as its contact.

From then on nothing gives the building again. S-0605 claim 5's *enter* processing with a contact
only ends with the standard delay. Guard no longer runs, and S-0215 claim 24 would not give the
building again anyway, since the walker is already bound for it (in contact). The walker stays
outside for good.

Before r71 the object destination survived the substitution. *Enter* then started, WP4 made the
building cell enterable, the next search went into it and S-0605 claim 6 took the walker in.

S-0208 claim 26's own backing marks claim 24's building case `unmeasured`.

## Questions

1. Does claim 26 apply when the destination that the substitute replaces is the building the
   walker is in contact with, or bound for by an enter-nearest instruction? Or does such a walker
   keep the building as its destination?
2. If claim 26 applies there, does S-0605 claim 5's *enter* processing of a walker in contact with
   an absorbing building, and with no destination, give it the building again, as S-0402 claim 10
   row 2 does for a harvester at each processing of *enter*?
3. If neither holds, does S-0605 claim 16's Edge case yield, so that with the instruction surviving
   the released walker stays outside?
4. More generally, does the first search after an assignment that queues *enter* without an
   immediate start read the mission target of *enter*, or of the behaviour still current?

## Possible observation

A neutral infantry unit with the bio-reactor fate, let out of an absorbing building of its owner
next to it. Record each tick its destination, its current and queued behaviour, its cell and
whether it goes back in.

## Current workaround

S-0208 claim 26 is held: the engine keeps its interim rule, in which an adopted substitute replaces
only a cell destination (S-0212 claim 24, departure D3), a kept locomotor destination walks on to
the substitute's centre at claim 18's height, and an object destination stays. Claim 18 of r71 (an
object's point in a high-bridge cell, Q-3264 question 2) is built. The claim 26 tests are not
committed; their scenes were: a walker given an allied parked vehicle farther than the
close-enough distance as its destination, and a walker with no destination and a kept point on
that vehicle's cell, each expecting the substitute as its cell destination.
