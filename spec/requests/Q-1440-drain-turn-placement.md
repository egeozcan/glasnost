---
id: Q-1440
title: Where in the turn the drain tests and the drain payout come
status: answered
related: [S-0608, S-0203, S-0405]
answered-by: [S-0608]
---

## Need
S-0608 says the alliance test (claim 11) and the position check (claim 12) happen "in every turn
of the drainer", and the payout (claim 7) "in its own turn" of the drained building, but not at
which point of that turn. S-0203 orders many other steps of an object's turn (behaviour
processing with its shots, the warp-only and erasure returns, the overload update, movement,
the building's power and economy step). The order is observable when a link ends or a payout
falls in the same tick as something else the turn does: a drainer that is pulled off the
building and checks in the same tick, a building that is erased, sold or captured in the same
tick as a payout, or a power update that follows the payout in the same turn.

## Questions
1. In the drainer's turn, do the alliance test and the position check come before or after its
   behaviour processing (its shots, including a new drain attempt) and its movement in that tick?
   Does a drainer that is being erased, or that only gets a warp update in that tick, still run
   them?
2. In the drained building's turn, does the payout come before or after its behaviour processing,
   its own power and economy step (S-0405 claims 7 and 9), and its erasure step? Does a building
   whose turn ends early (being erased) make the payout before it ends?
3. When a drainer's position check ends a link in the tick its house would make a power update
   (S-0405 claim 7), does that update already see the link as ended?

## Possible observation
S-0608 scenario D5 and D6 with the recorder at single-tick resolution: move the Disc off the
building so that its first tick outside the foundation is a tick with n mod 16 = 2, and record
whether the link ends in that tick; lock a drained refinery with a temporal weapon on a payout
tick and record whether that tick pays.

## Current workaround
Interim placement, cited as Q-1440 in `crates/deterministic/sim/src/tick.rs`: the drainer's tests
run right after the warp-only return and before the rest of its turn (so before its behaviour
processing and movement; an erased drainer still runs them); the payout runs in the building
section of the turn after the erasure return and the prism support step, before the building's
own power and economy step.
