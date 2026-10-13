---
id: Q-0193
title: What a production item killed by a defeat or surrender hit does afterwards
status: open
related: [S-0414, S-0404, S-0312]
answered-by: []
---

## Need

S-0414 claim 15 (revision 3) says that the defeat and surrender hits reach objects off the map,
production items included: an item's object is hit at its creation place and its health drops to
zero (V11 part c). The same claim says that a later factory loss can still delete such an item
through the cancellation of S-0404 claim 8, and that this deletion counts for the cancellation
exception (V11 part d). So the killed item stays in existence until something removes it.

No spec says what that killed item does in the meantime, nor what happens when nothing cancels it,
for example when the cancellation exception skips the last factory building of its kind and that
building survives the sweep. S-0312 gives the death of an object on the map; its death effects
(debris, explosions, survivors, leaving the map) have no position to use for an object that was
never put on the map. The code in `combat::death::kill` and `econ::production` must decide this.

## Questions

1. Which of S-0312's death effects take place for an item in production that a destruction hit
   kills: the death message or scoring, debris or explosion draws, survivors, other effects? Does it
   take any random draws?
2. Does the killed item stay on its production line until a cancellation removes it, or is it taken
   off the line (with or without a refund) at once or later in the same tick?
3. While it stays on its line: does its production go on, can it finish, and what does a release
   order (for a unit) or a place order (for a building) do with it? Does it count in the defeat test
   and the counts of S-0414 claim 4 until it is deleted?
4. If no cancellation ever comes (a factory building of its kind survives), what becomes of the
   item?

## Possible observation

V11 part c of S-0414 with a surviving replacement factory: arrange the cancellation exception so
that the replacement factory's place is skipped (two earlier immediate deletions just before it).
Record the held item's health, the sidebar icon of its type, the house's credits and whether a
release or place order produces anything, over the ticks after the surrender.

## Current workaround

`combat::death::kill`: an item's object off the map that a hit kills is only marked killed (the
`Killed` event, targets and projectiles dropped); it takes no other death effect and no draws, and
stays on its line with health 0. `econ::production::release` treats it as a failed release and
cancels it with the refund (S-0404 claim 13); a place order for it does nothing. It still counts
wherever objects in existence count. Marked as interim in docs/development.md.
