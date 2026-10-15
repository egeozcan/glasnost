---
id: Q-0715
title: Which cell of a bridge repair hut the bridge searches start from
status: open
related: [S-0511]
answered-by: []
---

## Need

S-0511 claims 16 and 18 search the 5 × 5 block "around the hut's cell" and run claim 16's
bridge-cell table from "the hut's cell" (claim 18 from the engineer's cell). The spec's open
questions leave the cell open when the hut's foundation has several cells. Claim 18 also says the
engineer's cell: the spec does not say whether that is the cell the engineer stands in when it
enters or the hut cell it enters. The code in `crates/deterministic/sim/src/bridges/hut.rs`
needs both cells.

## Questions

1. For a hut whose foundation has several cells, which cell is the centre of claim 16's block and
   the start of its bridge-cell table: the top-left foundation cell, the centre cell, or another?
2. For claim 18, is "the engineer's cell" the cell it stands in at the moment of entry, or the
   hut cell it enters?

## Possible observation

S-0511 scenario B12's search variants with a named test rules file that gives the hut a 2 × 2
foundation, and engineers entering from different sides.

## Current workaround

The hut's cell is the cell of its position (its top-left foundation cell); claim 18's engineer
cell is the engineer's own cell at entry. The choices are marked in the code with this request's
ID.
