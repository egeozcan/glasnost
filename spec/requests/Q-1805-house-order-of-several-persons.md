---
id: Q-1805
title: House order of several human players, and of a human whose colour is on random
status: answered
related: [S-0009, S-0412]
answered-by: [S-0009]
---

## Need
M9 step N2 builds the network lobby: a room holds several human players and computer players,
and every peer must build the same game setup from it, with the houses in the same order. The
order matters to the game: it decides the start positions of the players left on "random"
(S-0009 claim 19), the order of S-0412 claim 7's colour draws in the original, and the order of
the defeat tests (S-0414).

S-0009 claim 19 says houses are created with the human players first, in increasing order of
their chosen colour index (on a tie, the earlier lobby slot first), then the computer players in
lobby-slot order. It also says that the order among several humans is unmeasured, since a
skirmish has one human. S-0412 covers neither network games nor several humans.

Code: `crates/ui/src/setup.rs` (`Setup::house_order`, used by `Setup::lobby`, which builds the
players of the game setup), and the network lobby's slot table in `crates/net/src/lobby/room.rs`
(`Room::start`), which must follow the same order.

## Questions
1. In a game with two or more human players, are the human players' houses created in
   increasing order of their colour index, before every computer player's house, as S-0009
   claim 19 says?
2. A human player whose colour is left on "random" gets a colour slot from S-0412 claim 7's draws
   when the game is started from the lobby. Does that human's place in the house order follow the
   colour slot it was given, as if it had chosen that slot? If not, where does it go among the
   humans with a chosen colour?
3. Two humans can only share a colour index when both are on "random" before the draws. Does
   the "earlier lobby slot first" tie rule apply anywhere else, for example to observers, which
   take no colour slot (S-0412 claim 7)?

## Possible observation
A network game of three humans in lobby slots 1, 2 and 3 with chosen colours 5, 1 and 3 on a map
with four start positions, every start on "random": the start position each human gets, and
which human's house is listed first in the score list, against S-0009 claim 19's farthest-sum
rule for each possible order. Then the same with the second human's colour on "random", over
several seeds, recording the colour slot it was given.

## Current workaround
`Setup::house_order` follows S-0009 claim 19 as written for humans with a chosen colour: the
humans in increasing order of their colour slot, the earlier row first on a tie, then the
computer rows in row order. Humans whose colour is on "random" come after the humans with a
chosen colour, in row order, because our engine resolves random colours after the house order is
fixed (S-0412 claim 7 lets our engine choose how and in which order it draws them). With one
human, as in a skirmish, the order is the row order as before.
