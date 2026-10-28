---
id: Q-1836
title: A debris piece off the map, and one whose placement fails
status: answered
related: [S-0317, S-0106, Q-1702]
answered-by: [S-0317]
---

## Need
S-0317 lists as open (Open questions, from Q-1702 question 4) whether a piece that flies off the
map is removed for that, what its ground impacts and its end read there as its cell, and what
happens to a piece whose placement at creation fails (for example a meteor piece whose start
position lies off the map). `glasnost-sim` `pieces::fly` and `pieces::end` read a ground height,
a height level, a land type and a high-bridge flag for the piece's cell in every turn, so they
need values for a cell off the map.

## Questions
1. Is a piece that leaves the map in flight removed from play for that, at once or later?
2. While a piece is off the map: what ground height does its contact test meet (so where does it
   bounce, if anywhere), is it ever in water (claims 8 and 11), and does its end make its
   expiry animation, damage, further pieces and resource cell there?
3. A piece whose start position lies off the map at its creation (claim 4's meteor piece, moved
   back along its velocity): is it made at all, does it take its creation draws, and if it is made,
   does it take turns and end as any other piece?
4. Does any of this differ for a position outside the playable area but inside the map's full
   cell grid?

## Possible observation
A meteor piece type with a long lifetime and a large horizontal speed thrown near a map edge, with
the read-only recorder logging its position and counter each tick, its draws, and the effects of
its end (scenario V4's recorder with V9's draw log).

## Current workaround
Every piece is placed and takes its turns wherever it is; none is removed for being off the map.
Off the map the ground height is 0, the height level 0, the land type not water and there is no
high bridge, so a piece bounces at height 0 there and ends as on dry land. Each cites Q-1836 in
`crates/deterministic/sim/src/pieces.rs`.
