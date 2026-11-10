---
id: Q-2001
title: S-0108 claims 11 and 18 for a superweapon building whose house's record is not granted yet
status: answered
related: [S-0108, S-0512]
answered-by: [S-0108]
---

## Need

S-0108 claim 11 starts *super-3* at a superweapon building's grand opening "when the weapon's
remaining charge time is below 15 ticks, otherwise *super-1*", and claim 18 compares the
charged-animation time with "the weapon's remaining charge time". Neither says what that time is
when the owner's record of the type (S-0512 Terms) is not granted at that moment.

That moment is common. S-0512 claim 2 grants a record only at the house's recheck: "for a building
placed by order, the next tick's house update". So a superweapon building that completes in play
has its grand opening before its house's record of the type is granted, unless the house already
had the type from another building. A building that a trigger creates, or that the map places at
set-up, may be in the same position. S-0108 scenario B6 expects such a building to show *super-1*
("*super-3* only if less than 15 ticks of charge remain"), but does not say whether that is
because a record that is not granted counts as not charged or because some remaining time is read
from it.

The live view plays these slots (crates/app/src/mapview/slot_play.rs, `grand_opening` and
`charge_cycle`; the record is read in crates/app/src/mapview/live.rs, `update_slots`). It must
choose a reading for a record that is not granted, including one that was granted and then lost
(S-0512 claim 7) or that a house never had (an ownerless building).

## Questions

1. A house with no record of the type completes its first superweapon building of that type. At
   its grand opening, which slot starts: *super-1* or *super-3*? Is it the same when the building
   is placed by the map at set-up, and when trigger action 125 creates it in play?
2. A house whose record of the type was granted, charged and then lost (all its giving buildings
   gone, S-0512 claim 7) builds a new building of the type. Which slot starts at its grand
   opening, and does the answer depend on how much charge the lost record had left?
3. While the record is not granted, does claim 18's check run at all (for a type with a
   charged-animation time of at most 990), and if so, what remaining time does it compare with?
   When the record is then granted with remaining time *T* (S-0512 claim 3), what happens to the
   running superweapon slot in the following ticks?
4. A superweapon building that belongs to no house (or to a house without records, for example
   a neutral or civilian house): which slot starts, and does claim 18 run for it?

## Possible observation

S-0108 scenario B6 and B9 with the recorder: build a house's first superweapon building of a type
and record its slot animations in the completion tick and the next ten ticks, with the record's
granted state and remaining time each tick. Repeat for a second building of the same type built
while the first is charged (and while it is not), after the first is sold and the record lost,
and for a map-placed one at set-up.

## Current workaround

A record that is not granted counts as not charged: the grand opening starts *super-1* (which is
what B6 expects of a building completed in play), and claim 18's check waits until the record is
granted. Both are marked `// design:` with this request's number.
