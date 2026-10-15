---
id: Q-0717
title: Which section an attempt affects when several high-bridge overlays mark the cell
status: open
related: [S-0511, S-0211]
answered-by: []
---

## Need

S-0511 claim 2 sends an attempt on a high-bridge cell to "the section whose overlay marks it".
Each overlay marks four cells (S-0211 claim 15), so a cell can be marked by its own overlay and by
overlays one or two steps away in the marking direction, or one step against it. On an ordinary
bridge the sections lie across that direction and no cell is marked twice, but on map data with
neighbouring parallel bridges, or with an overlay that marks cells already marked by another, the
choice matters. The code in `crates/deterministic/sim/src/bridges/high.rs` (`section_of`) needs
the order.

## Questions

1. When several high-bridge overlays mark the attempted cell, which section is affected?
2. Is a cell marked as a high-bridge cell but with no overlay marking it at all (for example after
   the marking overlay was replaced by map data) affected in any way?

## Possible observation

A named test map with two high bridges along x one cell apart across, so that the outer marked
cells of one bridge are marked by the other bridge's overlays too; fire at that cell and watch
which section changes.

## Current workaround

The cell's own overlay is taken first, then the overlay one step against the marking direction,
two steps against it, and one step along it; the first high-bridge overlay whose marked cells
include the attempted cell is the section. A cell with no such overlay takes no effect. The choice
is marked in the code with this request's ID.
