---
id: Q-3330
title: A walker in area guard that reaches the building of its queued enter, switch off
status: open
related: [S-0208, S-0215, S-0605]
answered-by: []
---

## Need

S-0208's Edge case `quirks.walker_search_ignores_queued_enter` (r75), off form: while a walker's
queued behaviour is *enter* and its current behaviour has no mission target, *enter*'s mission
target is read, the building's cell reads *clear* by row WP3, the search finds a path into it, "and
the walker goes in under S-0605 claim 6 once *enter* is current". The Changes for S-0605 claim 16
say that with the switch off a released passenger of the neutral house walks back in.

That holds for a walker in *guard*: S-0215 claim 11 makes infantry in *guard* ready while it moves,
so the queued *enter* becomes current before its walk reaches the building. It does not hold in our
engine for a walker in *area guard*. Infantry in *area guard* is not ready while it moves (S-0215
claim 11), so *enter* stays queued until the walk ends, and the walk ends in one of the building's
foundation cells. S-0605 claim 6's arrival at the end of that step needs *enter* current, so the
walker does not go in.

An armed infantry unit of the neutral house rests in *area guard* when the area-guard threshold is
0 (S-0215 claim 14 row 9), and a player's infantry rests in *area guard* when it has the guard-area
ability or its type defaults to it (rows 11 and 12). In our test scene (synthetic data, threshold
0) the passenger walks into a foundation cell in *area guard*. In the same update a scatter request
there (S-0208 claim 11) is accepted, which queues *move* in place of *enter*. It walks out, falls
back to *area guard* again, and the re-run of the enter-nearest search sends it back, over and over.
It never goes in.

## Questions

1. With the switch off, what should happen when a walker in *area guard* (or another current
   behaviour that is not ready while it moves) reaches a foundation cell of the building of its
   queued *enter* before *enter* is current? Does it go in once *enter* becomes current, or
   does it stay out?
2. If it should go in: by what rule, given that S-0605 claim 6's arrival comes at the end of a
   walking step and the walker makes no further step there?
3. Should the off form instead make the building's cell read *blocked* (row WP19) for a walker
   whose current behaviour is not ready while it moves, so that it takes a substitute cell and
   waits there for *enter* to become current, as before r75?

## Possible observation

None: the off form is an engine design choice (`unmeasured`).

## Current workaround

The off form reads the queued *enter*'s mission target whatever the current behaviour, as the Edge
case gives. `claim_16_back_in_after_release_both_positions` in
`crates/deterministic/sim/tests/claims_s0605.rs` uses an area-guard threshold of 1, so the neutral
passenger rests in *guard* as S-0605 claim 16 describes. The area-guard case is
`claim_16_switch_off_back_in_from_area_guard`, ignored with this request's ID.
