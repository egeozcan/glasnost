---
id: Q-0283
title: Which periods of a flashing tab are lit, and what does the lit look look like?
status: answered
related: [S-0411]
answered-by: [S-0411]
---

## Need

S-0411 claim 11 gives the flash's timing (the look changes at ticks that are multiples of 10)
and leaves open which look belongs to which periods. The game client must draw one of them.

## Questions

1. In a period starting at tick *t* (a multiple of 10), is the tab lit when *t*/10 is even, or
   when it is odd, or does it depend on when the flash started?
2. What does the lit look look like compared with the normal tab button?

## Possible observation

S-0411 scenario S8 already records the frames; reading off the look of the first full period
settles question 1.

## Current workaround

Interim rule: the tab is drawn lit (our accent colour) in the periods where (tick / 10) is even,
and normal in the others. Recorded in docs/development.md, "Game client (M5b)".
