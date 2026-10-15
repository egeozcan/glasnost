---
id: Q-0710
title: Which cells form a bridge pier's tile, and the axis of a tile in both pier ranges
status: answered
related: [S-0511, S-0010, S-0307]
answered-by: [S-0511]
---

## Need

S-0511 Terms say a pier's state is its tile and that "changing the state replaces the tile on
every cell of the pier"; claim 6 lowers "every cell of its tile" on a collapse and claim 19
restores "all its cells". The spec's open questions leave the exact cells open. The code in
`crates/deterministic/sim/src/bridges/pieces.rs` (`tile_cells`) must pick the cells whose tile number it
rewrites and whose height it lowers.

Separately, a tile is a pier along x when *q* = *m*₁ + *k* and along y when *q* = *m*₂ + *k*
(*k* = 0 to 4). The spec does not say which axis applies when a theater's two ranges overlap.

Claim 6 says both spans next to a directly collapsed pier have claim 7's isolated-run result, but
not which span is checked first, and claim 7 does not say how the span holding a lost section finds
its start piece. `crates/deterministic/sim/src/bridges/mod.rs` (`attempt_in`) and `high.rs`
(`span_check`) need both.

## Questions

1. Which cells change tile and height when a pier changes state: the cells of the one placed
   tile that holds the reference cell (its origin found from the reference cell's sub-tile and
   the tile's width), every cell of that placed tile whose tile number is the pier's, or some
   other set?
2. Are cells of that placed tile whose tile number was changed by the map (another tile laid over
   part of it) also changed?
3. When a tile position lies in both the x and the y pier range, which axis does the pier take?
4. After a direct pier collapse, which span is checked first: the far-side span or the start-side
   span? Do their band raises interleave?
5. Is the start piece of the span checked after a section loss the nearest start piece toward the
   start side from the section cell, however far, or is the search bounded?

## Possible observation

S-0511 scenario B4 with the tile and height of every cell in the pier's tile rectangle recorded
before and after the collapse, on a stock pier and on a named test map where another tile covers
one cell of the pier's rectangle.

## Current workaround

The cells are those of the placed tile that holds the reference cell: its origin is found as in
S-0307 claim 23 from the reference cell's sub-tile and the catalogue width, and each cell of the
rectangle that still holds a tile of the same pier (any state) and whose sub-tile equals its place
in the rectangle is changed. A position in both ranges is treated as a pier along x. After a
direct pier collapse the far-side span is checked first, then the start-side span. The span check
after a section loss starts from the nearest start piece toward the start side, with no bound. The
choices are marked in the code with this request's ID.
