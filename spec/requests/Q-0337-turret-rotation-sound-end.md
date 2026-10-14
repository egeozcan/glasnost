---
id: Q-0337
title: How a turret's rotation sound ends, and how often it starts
status: open
related: [S-0701, S-0206]
answered-by: []
---

## Need

S-0701 claim 23 says that a turreted type's rotation sound plays as an attached sound while the
turret turns. Unlike the movement sound, the claim gives no rules for starting and ending it:

- no grace counter;
- no rule on whether the end lets the current loop finish or cuts it at once;
- no rule on what happens when a new turn starts while the sound still plays.

The turret scan in `glasnost_audio::events` needs these rules.

## Questions

1. When the turret stops turning, does the rotation sound stop at once, or finish its current
   loop pass (as the movement sound does)?
2. Is there a delay after the turn ends before the sound is ended?
3. When a new turn begins while the rotation sound is still playing, is it restarted, or does it
   simply continue?
4. Does a turn that starts and ends within one tick start the sound?

## Possible observation

Give a turreted vehicle a long, looping rotation sound. Order a short turn, then a long turn, and
then two turns back to back. Record the start and end of each sound against the turret's motion.

## Current workaround

The scan runs once per tick. While the turret turns (S-0206 *turning*) and no rotation instance of
the object exists, it requests one as an attached sound. Once the turret has stopped, the instance
is asked to finish its current loop pass, with no grace counter. A turn that goes on across ticks
keeps the one instance. A new turn after a stop requests a fresh instance, even while the
finishing one is still heard. A turn that starts and ends between two scans is not heard. Documented in docs/development.md, "Audio (M5b)".
