---
id: Q-1806
title: The options menu, lost focus and the game speed in a network game
status: answered
related: [S-0201, S-0414]
answered-by: [S-0201]
---

## Need
M9 step N3 plays network games: every peer runs the same ticks in lockstep, paced by its own
clock and held back only when another player's orders for a tick have not arrived. The client
must decide what the options menu, a game window without the focus and the options menu's speed
control do in such a game.

S-0201 claims 8 and 9 stop every cycle while the options menu is open and while the window lacks
the focus, and let the options menu change the game speed, but only for single-player games;
claim 13 leaves network games, "how losing focus and the pause action behave" and "the way a speed
change is sent", to a future multiplayer spec. S-0414 claim 23's active clock pauses for the
options menu and lost focus, also stated for a skirmish.

Code: `crates/app/src/game/client.rs` (`Client::update`, the network branch) and
`crates/app/src/game/session.rs` (`Session::pointer_up`, the options menu's speed control, and
`Session::network`).

## Questions
1. In a network game, while one player has the options menu open, does that player's game go on
   running ticks (the others' games too), or does every player's game stop until it is closed?
2. In a network game, while one player's game window lacks the focus, does the game go on for
   that player and for the others?
3. In a network game, does the options menu offer the speed control? If it does, does a change
   by one player change the speed of every player's game, and from which tick?
4. Does the game's elapsed-time clock shown at the end of a network game (S-0414 claim 23's active
   clock) stop while a player's options menu is open or the window lacks focus?

## Possible observation
A two-player network game on a LAN: one player opens the options menu for 30 seconds while the
other watches a unit's movement and the game clock; then the same with the first player's window
put in the background. Then the first player moves the speed control in the options menu and both
watch whether the movement speed changes.

## Current workaround
Our engine's interim rule: in a network game the options menu and lost focus never stop the
ticks (the lockstep alone holds a game back, waiting for a peer), the options menu's speed
control does nothing (every peer keeps the room's speed, set before the start), and the active
clock runs whenever the game does.
