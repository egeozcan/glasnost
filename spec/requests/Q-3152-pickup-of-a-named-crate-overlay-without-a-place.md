---
id: Q-3152
title: Whether the pickup of a named crate overlay that holds no place and is no map crate removes it
status: open
related: [S-0520, S-0319, S-0202]
answered-by: []
---

## Need
S-0520 claim 11 (revision 12) says: "A continuing pickup frees the occupied place with the lowest
priority number among those whose cell is the crate's cell, if there is one ... Freeing it removes the
crate when the crate's overlay is a named crate overlay, as at an expiry (claim 29); a crate of another
overlay type stays (Edge cases, unsafe data), except a map's own crate (below)." The Edge cases entry
for unsafe data says of an overlay that is not a named crate overlay that "neither its pickup nor an
expiry removes it".

A named crate overlay can stand on a cell with no crate place of that cell and without being a map's
own crate: an overlay placement in play (S-0202 claim 10) can make one, as the resource bounce of
S-0319 claim 10 does when its overlay draw lands on a named crate overlay's place in modded data. With
`quirks.crate_place_by_cell` off the same arises whenever a named crate overlay belongs to no place.
The first sentence then frees no place, so the second sentence's "freeing it" removes nothing, while
the Edge cases entry implies that a named crate overlay is removed by its pickup. (Issue #284.)

## Questions
1. Is a named crate overlay that holds no place and is no map's own crate removed by its pickup?
2. If it is not removed, can it be collected again, as a crate of another overlay type can?

## Possible observation
None in the original's ordinary play: a resource bounce over stock data does not place a crate overlay,
and a skirmish has no map crates. A test file whose bounce overlay list ends on the wooden-crate
overlay, with the crate setting on and every crate place free, would show whether a unit that drives
onto the bounced crate removes it and whether a random replacement appears.

## Current workaround
The crate is removed, as before revision 12 (`crates::pickup` in
`crates/deterministic/sim/src/crates.rs`): a pickup removes every named crate overlay, whether or not
it frees a place.
