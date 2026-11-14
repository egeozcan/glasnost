---
id: Q-3109
title: Which cell and layer table C's zone row reads in an area search
status: open
related: [S-0309, S-0212]
answered-by: []
---

## Need

GitHub issue #245. S-0309 gives an area search two region tests:

- Claim 11 skips every visited cell that is not in the searcher's reach region, with its own read: "Both the visited cell and the searcher's own cell are read for their decks when they are bridge cells, and the regions are compared as in claim 7 (S-0212 claim 4)".
- Table C's row *zone* rejects a candidate *T* whose "cell is outside the connected region of *A*'s movement zone that holds *A*'s own cell, in the searches that claims 7 and 11 restrict (compared as claim 7 gives: two cells of no reach region of that zone count as the same region)".

Claim 7 reads the object's cell "in the cell layer the object occupies". The engine now builds claim 11's cell skip as written (the visited cell read for its deck when it is a bridge cell), and keeps table C's row in the area search with claim 7's read (the object's position cell, in the layer the object occupies). The two reads differ for an object on the ground under a high bridge whose deck holds no object, and for a building offered from a foundation cell other than its position cell.

Where they differ, the row can reject an object whose visited cell claim 11 does not skip. Example: a land searcher on the bank, in the region of the bridge link's first end; an enemy on the ground under the bridge, on ground of another reach region or of none. Claim 11 reads the visited cell for its deck, gets the bank's region and does not skip the cell. Table C's row with claim 7's read reads the ground and rejects the enemy. With claim 11's read the row would let it through.

## Questions

1. In an area search, which read does table C's row *zone* use for *T*: claim 7's (*T*'s cell in the layer *T* occupies), claim 11's (the visited cell, read for its deck when it is a bridge cell), or does the row not apply in an area search beyond claim 11's skip?
2. If the row uses claim 7's read in an area search, is the cell *T*'s position cell or the visited cell that offered *T* (they differ for a building offered from another foundation cell)?

## Possible observation

The example above: a high bridge over ground that is its own reach region for the searcher's zone (walled off from both banks), an armed land unit in *area guard* on the bank next to the bridge's first end, and an unarmed enemy vehicle on the ground under the empty deck, within the area search's radius. Does the unit pick the enemy? Repeat with the searcher on the walled-off ground off the bridge cells (claim 11 then skips the cell; the expected answer is no pick) as the control.

## Current workaround

Claim 11's cell skip is built as written (`combat::acquire::search`). Table C's row *zone* keeps claim 7's read in both the map-wide and the area search (`combat::acquire::same_zone`), as before this issue.
