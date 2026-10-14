---
id: Q-0226
title: In what order are the creation draws taken when several production lines start items in one tick, and when cloning buildings make copies?
status: open
related: [S-0404, S-0203, S-0202, S-0417]
answered-by: []
---

## Need

S-0404 claim 11 (revision 6) says that each item start takes one creation draw from the
scenario random stream. It adds that when several lines start items in the same tick, the
assignment of successive stream values to those creations is not established, and that the
case is open pending a measurement. S-0203 claim 1 asks the production specs to state the
observable draw order.

S-0404 claim 14 (revision 6) says that the observable order of cloning copies, and any effect
of the order in which copies are placed on which draws they take, are not established.

The simulation must take these draws in some fixed order for the state hash to be defined, and
that order should be the game's.

## Questions

1. Two or more lines of one house, or lines of different houses, start items in the same tick
   (for example, two lines finish their items in one tick and each starts the next queued type,
   or a build order and a line's own start fall in one tick). In what order do the creations take
   their draws? By the line order of S-0404 claim 1, by house, by the part of the tick in which
   each start is applied, or another order?
2. When an infantry factory's release makes copies in several cloning buildings, in what order
   are the copies created and released? Does each copy take all its draws (creation, exit
   search and the like) before the next copy is created, or are the creations taken first?
3. Does a copy's draw order depend on where it is placed, or only on the order of the cloning
   buildings?

## Possible observation

S-0404 scenario P11 with two or three lines whose items complete in the same tick, and P12 part
(g) with two or three cloning buildings, each with the random-state recorder, recording which
stream values each creation takes.

## Current workaround

Starts take their draws in the order the simulation applies them: build orders in the order
they arrive in the orders part, and line starts in S-0404 claim 1's line order in the
production part. Cloning copies are made in the order the house acquired the cloning
buildings, each copy created and released before the next. Listed as an interim row in
docs/development.md.
