---
id: Q-3249
title: What the game shows where a tree beside a large building's west corner overlaps the building's body
status: open
related: [S-0102, S-0112]
answered-by: []
---

## Need

Issue #437: in a native skirmish on a snow map the project lead saw a snow-covered pine tree drawn
over the left (west) side of an iron curtain building's raised base, where it looked as if it
belonged behind the building. The tree's cell relative to the building's foundation was not
recorded.

The tree (a terrain object that is not animated) and the building's body are both depth writers
(S-0102 claim 10), so each pixel where they overlap goes to the nearer of the two, the earlier
one on a tie (claim 16). The tree's depth is upright from its last row with a bias of −*L(z)* − 12
(claims 13 and 23, F2). The body's depth is its unrounded upright anchor *d*(*r*₁) − *L(z)* − 2
minus the shared companion depth image's value at each pixel (claims 13 and 31, F5).

Our engine follows these rules exactly; a trace on a synthetic map with an invented companion
image (`crates/app/src/mapview/tree_depth_tests.rs`) gives, for a 4 × 4 foundation whose top-left
cell is (*x*, *y*), the CPU and GPU paths alike:

- a tree on (*x* − 1, *y* + 3), the cell north-west of the west corner cell (*x*, *y* + 3), is
  behind the foundation and the body covers it wherever they overlap;
- a tree on (*x* − 1, *y* + 4), the cell west of the corner cell (the same *x* + *y*), or on
  (*x*, *y* + 4), south-west of it, covers the body's west corner wherever they overlap: its
  last row is lower on screen than the body's front edge there, and its bias is 10 nearer.

So with the specified rules a tree beside the west corner is drawn over the body's corner, as
the lead saw, unless the real companion image makes the body nearer there. We cannot tell from
the specs whether that is what the game shows, and it looked wrong in play.

## Questions

1. A tree on the cell west of a large building's west corner cell (the same *x* + *y* as the
   corner cell), its crown reaching over the building's body: does the game draw the crown over
   the body there, or does the body hide it?
2. The same for a tree on the cell north-west of the corner cell (*x* + *y* one less): is it
   hidden by the body wherever they overlap?
3. If the game hides the tree in question 1: what makes the body nearer there? The values of the
   shared building depth image at the body's west corner, a depth adjustment of the building type
   (for example its normal depth adjustment), or something else in the order of the cell pass?

## Possible observation

On a snow map, place an iron curtain (or another building of a foundation at least 3 × 3 cells)
with its top-left cell at (*x*, *y*) and a tall tree on (*x* − 1, *y* + *h*) (west of the corner
cell, *h* the foundation's height in cells), then a second map with the tree on
(*x* − 1, *y* + *h* − 1). Take a screenshot of the corner in each, and note whether the tree's crown
shows over the body. For the first case, the depth image values at the body's west corner
(S-0102 F5's sample there) would settle question 3.

## Current workaround

None needed: the engine draws both cases as S-0102 claims 10, 13, 16, 23 and 31 give, checked by
the guard tests in `tree_depth_tests.rs` (#437).
