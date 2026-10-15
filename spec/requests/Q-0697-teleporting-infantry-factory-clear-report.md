---
id: Q-0697
title: When a teleporting infantry unit released by an infantry factory ends its factory contact
status: answered
related: [S-0417, S-0208, S-0207]
answered-by: [S-0417]
---

## Need

S-0417 claim 10 tethers a released infantry unit to its factory. The factory's contact place is
held until the unit gives its clear report, and while every place is held the next infantry
release is busy (S-0417 claim 14, S-0404 claim 13). S-0208 claim 23 gives the report on a
walker's standing-position arrival, but says it does not prescribe the teleporting case. S-0417
claim 10 says the same. S-0207 claim 22 gives an own-locomotor teleport of any object kind an
arrival update in the jump tick. It does not say whether a tethered infantry unit reports clear
on that update.

Without a rule, a released teleporting infantry unit that jumps away keeps its factory contact.
Its factory then refuses every later infantry release as busy, for the rest of the game. AI
duels on the local lane showed this.

The code in `crates/deterministic/sim/src/teleport.rs` (the jump) needs the rule.

## Questions

1. Does a tethered teleporting infantry unit, released by an infantry factory, end its factory
   contact and tether in the tick of its first jump after release?
2. If not then, on which event does it end them: when phasing in ends, when its move falls back
   to rest, or on some other event?
3. If it never jumps (for example with no rally point and no order), does it keep the contact
   as a resting walker does (S-0417 claim 10, Open question 3)?

## Possible observation

S-0417 scenario U10 (a) with a stock teleporting infantry type and a rally point several cells
away. Record the unit's position, contact and tether each tick from release through its jump
and the end of phasing in. Then order a second unit and record whether its release is busy.

## Current workaround

The jump's arrival update (S-0207 claim 22) is a tethered teleporting infantry unit's clear
report, given in the jump tick before the entered-by raise, as S-0208 claim 23's walker reports
on its arrival. The infantry factory gives no run-away reply. The choice is marked in the code
with this request's ID.
