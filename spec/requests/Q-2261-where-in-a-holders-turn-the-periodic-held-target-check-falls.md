---
id: Q-2261
title: Where in a holder's turn the periodic held-target check falls
status: open
related: [S-0309, S-0215, S-0301, S-0203]
answered-by: []
---

## Need
S-0309 claim 29 (revision 70) drops, in a holder's own turn of every tick whose tick number is a
multiple of 16, a held target that the firing check (without the range test) answers *illegal* or
*impossible* for. The claim fixes its place only against a building's pending delayed shot (the
check comes first). Its Open questions leave the rest open: *Place of the periodic check* and *A
refusal that begins in the holder's turn*. Our engine must call the check at one place in the
object turn (`tick::object_turn`, `combat::held::periodic_check`).

## Questions
1. In a check tick, does a holder make the check before or after its behaviour processing, its
   automatic searches and its firing step in that turn?
2. When a refusal begins inside the holder's own turn of a check tick (through its own behaviour
   processing or firing step, for example a target that it makes illegal by its own action), does
   that turn's check act on it, or only the check of the next check tick?
3. When a queued behaviour becomes current in a check tick's turn (*capture* or *sabotage*
   replacing *attack*, for example), which of the two behaviours decides whether the holder makes
   the check?

## Possible observation
S-0309 scenario A6 part p with a refusal started by the holder's own processing in a check tick:
record, with tick numbers, the holder's target, its current behaviour before and after its turn,
and whether the drop is seen at the end of that tick or of the next check tick.

## Current workaround
The check comes at the start of the turn: after the warp update (S-0513 claim 51) and S-0309
claim 27's turn-start drop, before the behaviour start, the behaviour processing, the searches,
movement and the firing step, and so before a building's pending delayed shot counts down. A
refusal that begins later in the holder's own turn of a check tick is acted on at the next check
tick. The behaviour current at the start of the turn decides.
