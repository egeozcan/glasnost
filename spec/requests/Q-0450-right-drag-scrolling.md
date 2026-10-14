---
id: Q-0450
title: How does the view move during a right-button drag?
status: open
related: [S-0214, S-0101]
answered-by: []
---

## Need

S-0214 claim 27 and F8 fix when a right-button press becomes a scroll drag
and that its release neither clears the selection nor cancels a targeting
mode. The claim leaves the scrolling itself to S-0101 ("the scrolling rate,
direction, pointer recentering and view limits are companion rules of
S-0101"), but S-0101's Open questions say right-button drag scrolling is not
specified by its claims 12 to 15. The game client (`game::session`, the
right-press anchor and `scroll`) can classify the gesture but cannot move
the view.

## Questions

1. While a right-button scroll drag is active, how far does the view move,
   and in which direction, for a given pointer displacement from the press
   pixel? Is the movement once per drawn frame, per input sample or per
   game tick, and does game speed or the scroll-speed setting change it?
2. Does the pointer stay where it is, return to the anchor (recentering), or
   is it hidden while dragging?
3. How do the three values of the scrolling-method preference (S-0214
   Scenario T19 mentions values 0, 1 and 2) differ in the movement?
4. Do the scroll limits, the order block of S-0201 claim 14 and frozen
   cycles (S-0101 claim 15) apply to drag scrolling as they do to edge and
   arrow scrolling?
5. Is the cursor image during a scroll drag one of the edge-scroll images of
   S-0213 claim 24, or another?

## Possible observation

S-0214 Scenario T19's right-drag runs, recording the view origin per drawn
frame and per tick against the pointer path, for each scrolling-method value
and two scroll-speed settings, with the view far from and at a scroll limit.

## Current workaround

The client classifies a right-button drag by F8 (|Δx| > 8 or |Δy| > 8
pixels from the press pixel), keeps it a drag until release and ignores its
release, but the view does not move during it. Arrow and edge scrolling are
unchanged.
