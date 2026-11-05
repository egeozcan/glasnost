---
id: Q-2578
title: How an aircraft flies after a refused touchdown that kept its destination
status: open
related: [S-0501]
answered-by: []
---

## Need

S-0501 r120 claim 7 says three things about a refused touchdown:

- The aircraft rises by 10 leptons and starts to take off.
- That take-off ends in the next movement update, leaving the held height at the landing's (0).
- The touchdown first forgets the destination and the flight point when the aircraft is in the
  flight point's cell, or when that cell holds its contact (or no building, with no contact).

When the touchdown forgets them, the aircraft holds 10 leptons above the cell. When it keeps them
(it has drifted out of the flight point's cell, and that cell holds a building that is not its
contact), "Its flight after a refusal that kept the destination is not traced here." Claim 24
repeats this for an aircraft in *enter*'s docking stage.

The engine has to fly such an aircraft. It still has a destination and a flight point, it is in
flight, not landing and not taking off, and its held height is the landing's 0.

## Questions

1. After the refusal's take-off ends, which held height does the aircraft use? Claim 6 gives the
   cruise height of ordinary flight, or the destination's height below 768 leptons with the
   holding-height mark. Does that apply from the next movement update, or does it keep the landing
   height 0 for a while?
2. Does it fly back toward its flight point, as in ordinary flight (claims 4 and 5, with claim 42's
   guidance), and can claim 6's landing triggers start a new landing there?
3. Does the same hold for an aircraft in *enter*'s docking stage in contact, whose next processing
   (claim 24) finds it in flight and not landing and reports entry?

## Possible observation

AC4's set-up with an occupant standing on the cell next to a building cell that is not the lander's
contact. The building is the destination. Arrange the landing to drift onto the occupied cell and
be refused there. Record the aircraft's height, position and landing state each tick after the
refusal until it touches down again.

## Current workaround

The refusal's take-off ends in the next movement update, as claim 7 gives. From that update on, the
aircraft flies in ordinary flight toward the destination it kept, and claim 6 gives its held height
(the cruise height, or the destination's height with the mark). Claim 6's triggers apply as for any
aircraft. Cited `design: S-0501 claim 7 and Q-2578`.
