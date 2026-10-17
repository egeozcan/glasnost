---
id: Q-1262
title: Which bridge-deck neighbour at the eighth direction ends an absorbing building's exit search
status: open
related: [S-0605, S-0507]
answered-by: []
---

## Need
S-0605 claim 12 says a deck neighbour is passed over and "when this happens at the last direction
of the first round, the search ends without a second round". The Edge cases entry for
`quirks.absorber_release_bridge_ends_search` says this happens "when the eighth direction of the
first round passes both cell tests but its neighbour cell carries a bridge deck". The two
wordings differ for a deck neighbour at the eighth direction whose cells do not pass the tests.
Code: `crates/deterministic/sim/src/structures/absorb.rs`, `let_out`.

## Questions
1. At the eighth first-round direction, does a deck neighbour end the search only when the
   neighbour and far cell pass both cell tests, or whenever the neighbour carries a deck?

## Possible observation
An absorbing building with a bridge deck next to its top-left cell at the eighth direction and
all other directions blocked; once with the far cell open and once with it blocked; release a
passenger and record whether it comes out through the second round.

## Current workaround
The Edge cases version: the search ends only when both cell tests pass and the neighbour carries
a deck (switch on). A deck neighbour that fails a cell test is passed over and the second round
runs.
