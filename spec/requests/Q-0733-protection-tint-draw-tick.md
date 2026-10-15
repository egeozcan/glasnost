---
id: Q-0733
title: The exact tick of the protection tint's draw
status: open
related: [S-0513]
answered-by: []
---

## Need
S-0513 claim 29 says a protected object takes one ranged draw R(−5, 5) "about 10 ticks after
protection starts", in the turn in which the tint enters its pulsing phase. Its confidence note
says the draw comes 10 or 11 ticks after the start. The draw is game state, so our engine must
pick one tick (`superweapons::protect::tint_turn`).

## Questions
1. Protection starts in tick t₀ (a curtain launch applied in the order step, or a force shield).
   Is the draw taken in the object's turn in tick t₀ + 10 or in tick t₀ + 11?
2. Is it the same for protection started by a curtain and by a force shield, and for buildings
   and units?
3. If the object takes no turn in that tick (for example it is inside a transport) but is back in
   play while still protected, does it take the draw at its next turn, or never for that start?

## Possible observation
S-0513 scenario SW10 part e with the random-state recorder: start protection on one vehicle in a
known tick and record the tick of the extra draw.

## Current workaround
Interim rule (docs/development.md, *Simulation (M6b: superweapons)*):
- The draw is due in tick t₀ + 10.
- It is taken in the object's first turn at or after that tick, while the object is still
  protected.
- Protection that has expired takes no draw.
