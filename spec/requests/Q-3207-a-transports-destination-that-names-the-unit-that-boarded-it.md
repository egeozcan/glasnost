---
id: Q-3207
title: A closed transport whose destination names the unit that has just boarded it
status: answered
related: [S-0507, S-0312, S-0207, S-0503, S-0215]
answered-by: [S-0507]
---

## Need
S-0507 claim 5 lets a transport standing on water take a requester as its destination ("the
transport has acquired that destination"). When that unit then boards (claim 6), the transport
still holds a destination that names its own passenger. No claim says what becomes of that
destination, or which point it stands for while the unit is inside.

Since sim batch S25 (#383) the engine follows S-0312 claim 34 (b): a passenger of a closed
transport keeps the position it had when it went inside, and the transport no longer carries it
along. So the destination's point is now the cell where the unit boarded. Before S25 it was the
transport's own position. In a lane game a hover transport at rest in *guard* held such a
destination for some 30 to 80 ticks. Before S25, S-0207 claim 26's guard row removed it at once,
because the supplied point equalled the transport's own point. With the kept position the
transport is one cell off that point, finds no path and does not move. It holds the destination
until a new order replaces it. The engine needs to know which of these the game does.

## Questions
1. A transport on water takes an infantry unit as its destination by claim 5's no-acceptance
   reply. The unit then boards it. Does the transport still have a destination after the
   boarding?
2. If it keeps it: which point does that destination stand for while the unit is inside? Is it
   the cell where the unit boarded, the transport's own position (moving with it), or something
   else? Does the transport try to move toward it?
3. Is any of this different for a hover transport and for a ship, or for a vehicle transport on
   land whose destination names a unit that boards it?
4. More generally: a moving unit's destination names an object that then goes inside a closed
   transport (not this one) or a building. Does the destination stay, and what point does it
   stand for while the object is inside?

## Possible observation
A hover transport stands on water next to a shore. An infantry unit on the shore is ordered to
enter it, so the transport takes the infantry as its destination and the infantry boards. Record
each tick after the boarding: whether the transport moves, toward which cell, and when it is
next idle, which shows whether it holds a destination. Repeat with the transport one cell away
from where the infantry stood when it boarded.

## Current workaround
None needed in code. The engine reads the destination's point from the passenger's kept position
(S-0312 claim 34 (b)) and keeps the destination by the rules that exist. No rule clears it at
boarding, and S-0207 claim 26's guard row removes it only when that point equals the
transport's own. The change since S25 is recorded in `docs/m8-claims/sim-issues.md`, batch S26.
