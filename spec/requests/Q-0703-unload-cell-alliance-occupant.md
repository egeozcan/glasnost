---
id: Q-0703
title: Which occupant supplies the alliance test of the unloading-cell choice
status: answered
related: [S-0507]
answered-by: [S-0507]
---

## Need

S-0507 claim 9 gives a neighbour cell the open-cell score only when "the occupant used for the
alliance test is allied from the transport owner's side". The spec's Open questions leave "which
occupant supplies claim 9's alliance test on a mixed-occupancy cell" open.

The code in `crates/deterministic/sim/src/transport.rs` (`open_cell`) needs the rule.

## Questions

1. Which occupant is tested: the first infantry unit in the cell's list, the house that holds the
   cell's infantry positions, or another?
2. Does a cell with no occupant pass the test?

## Possible observation

S-0507 scenario T9: a neighbour cell holding infantry of two houses, one allied with the
transport's owner and one not, in both orders of arrival.

## Current workaround

The house recorded as holding the cell's infantry positions (its occupancy record) is tested; a
cell with no infantry passes. The choice is marked in the code with this request's ID.
