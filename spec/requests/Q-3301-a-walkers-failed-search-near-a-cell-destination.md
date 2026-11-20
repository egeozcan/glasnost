---
id: Q-3301
title: A walker's failed search near a cell destination, and a standing walker with one
status: answered
related: [S-0208, S-0212, S-0209, S-0417]
answered-by: [S-0208]
---

## Need

S-0208 claim 25 (r68) stands under the heading *Searches for an object destination* and opens with
"While a walker's destination names an object". Its two later paragraphs read more generally:

- *A failed search near the locomotor destination*: "When a walker's fresh search finds no path
  and the reachability gate has passed ..., the walker loses its destination in the same
  locomotion when" its distance *D* to its locomotor destination is less than the close-enough
  distance and it is not held by a tether link. Where S-0212 claim 23's third row keeps the
  destination, "it has no locomotor destination from then on, so it makes no further search until
  its destination is assigned again".
- *A walker standing with no stored path*: the cell that decides whether anything is left to walk
  is its locomotor destination's cell; with movement in progress it searches for that cell, and in
  that cell the search fails and the rule above decides.

The engine applies both paragraphs only while the destination names an object. For a cell
destination it keeps its earlier rules: a failed fresh search is decided by S-0212 claim 23 alone,
a walker kept by row 3 searches again after the retry delay, and a standing walker with no stored
path in its destination cell at the right height has its move finished at once with no search.

## Questions

1. Does the failed-search clearing (*D* less than the close-enough distance, no tether link) also
   apply to a walker whose destination is a cell?
2. For a walker with a cell destination that S-0212 claim 23's third row keeps, does it also have no
   locomotor destination from then on and make no further search until the cell is assigned again?
   Or does it search again once the retry delay has run, as S-0212 claim 21 reads?
3. Does a standing walker with no stored path whose cell destination is its own cell (within two
   height levels of claim 18's point) make the fresh search, or is its move finished with no
   search?

## Possible observation

S-0208 scenario W18 with a cell in place of the object: a walker given a cell next to a wall of
blocked cells so that its fresh search fails past the gate, once within and once beyond the
close-enough distance; record its destination and its later searches for some hundreds of ticks.
For question 3, a walker standing on a free standing position given its own cell.

## Current workaround

Cell destinations keep the rules they had before r68: S-0212 claim 23 alone decides a failed
search, row 3's walker searches again after the retry delay, and the own-cell shortcut finishes the
move of a standing walker.
