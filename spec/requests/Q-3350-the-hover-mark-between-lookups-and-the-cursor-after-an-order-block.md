---
id: Q-3350
title: The hover mark between lookups, and the cursor's image after an order block hides it
status: answered
related: [S-0214, S-0111, S-0201]
answered-by: [S-0214]
---

## Need

S-0214 r96 claim 35 (e), *A held press and an order block*, says that a hover lookup that finds an
object gives it the *hover mark* (Terms), that the mark is cleared in every tick, and that only a
later lookup that finds the object gives it again. It also says that during an order block the
game hides the cursor (S-0111 claim 20) and makes no hover lookup. Our client must draw the hover
health bar (S-0111 claim 3) and the cursor (S-0214 claim 22) from these rules
(`Session::update_marks` and `Session::update_cursor` in `crates/app/src/game/session.rs`). Two
points are not stated:

- Whether a lookup that finds another object, or no object, takes the mark from the object the
  last lookup found, before the next tick clears it; and whether a frame drawn with the pointer
  off the battlefield view (where claim 35 (e) makes no lookup of the cursor's object) keeps the
  mark until the tick.
- What claim 22's animation does across the span in which an order block hides the cursor: whether
  the hidden cursor is a change of kind, like the system pointer (claim 22), so that the first
  image after the block starts at its first frame and a kept image (`Retain`) has nothing to keep,
  or whether the image shown before the block carries on.

## Questions

1. Between two ticks, while no press is held and no order block is in force, the pointer moves
   from object A's region onto object B's in one drawn frame. Does A still have the hover mark
   (and its hover health bar) until the next tick, or only B? The same with the pointer moving
   from A onto an empty cell, and with it moving from A off the battlefield view, over the
   sidebar.
2. When an order block ends with the pointer over an object whose image animates (for example
   the select image), does the first image after the block start at its first frame? And when the
   first offer after the block keeps the previous image (S-0214 claim 21, the damaging-entry
   image with `quirks.damaging_entry_cursor_retained` on), which image is kept: the one shown
   before the block, or the ordinary arrow?

## Possible observation

1. As S-0214 scenario T27, with two vehicles of the player's standing 2 cells apart and the hover
   health bar recorded per drawn frame: move the pointer from one vehicle's point to the other's
   between two ticks (the game slowed to its lowest speed), then from a vehicle to an empty open
   cell, then from a vehicle onto the sidebar.
2. As scenario T27 part (f): the pointer rests over a unit of the player's with a unit selected so
   that the select image shows; a trigger blocks the player's input for 60 ticks; record the
   cursor frame per drawn frame from the block's end.

## Current workaround

1. One object at a time has the mark: each lookup gives it to the object it finds and takes it
   from any other, and a lookup that finds nothing leaves no object with it. A frame drawn with the
   pointer off the battlefield view makes no lookup, so the mark stays until the next tick clears
   it. Marked `design: render (Q-3350)`.
2. The hidden cursor leaves claim 22's animation as it was: the image after the block is chosen as
   the next image after the one shown before it. Marked `design: render (Q-3350)`.
