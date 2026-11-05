---
id: Q-2626
title: Which sounds and attack announcements a berserk hit makes besides the harvester announcement
status: answered
related: [S-0302, S-0701]
answered-by: [S-0302]
---

## Need

S-0302 r41 claim 10 says a berserk hit sets off none of the reactions of the hit's own damage step, and that
the announcement that a harvester is under attack (S-0701 claim 28) is among the reactions that follow it.
It does not say whether the other sound reactions to a hit that reports *unchanged* are made for a berserk
hit: the damage sound of the target's type (S-0701 claim 20 gives it for *unchanged*), and the *ally under
attack* announcement that S-0701 claim 28 ties to a hit on an allied house's object. A berserk hit on a
building is *unaffected*, so the building rows do not arise. The code is in `damage::apply_hit` (the berserk
branch) and `glasnost-audio` `events.rs` (`hit`, `attack_announcements`).

## Questions

1. Does a berserk hit that reports *unchanged* play the target type's damage sound, as any other *unchanged*
   hit does?
2. Does a berserk hit on a unit of an ally of the local player (not a harvester) make the *ally under attack*
   announcement and radar event?
3. Does a berserk hit on a unit of the local player that is not a harvester make any announcement?

## Possible observation

A berserk warhead fired at a local-player unit whose type names a damage sound, and at an ally's unit, with
the sound log and the announcement queue recorded.

## Current workaround

A berserk hit gives only the harvester-under-attack announcement (a `Sound::HarvesterAttacked` event, heard
only for the target's owner when it is the local player and the type is a harvester). No damage sound and no
other announcement: the behaviour before r41, which made no sound for a berserk hit.
