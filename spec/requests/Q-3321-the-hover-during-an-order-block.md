---
id: Q-3321
title: Is the object under the cursor looked up again during an order block, and what a held left press keeps
status: answered
related: [S-0214, S-0201, S-0111]
answered-by: [S-0214]
---

## Need

S-0214 claim 35 (e) says the cursor's object is found again once for every drawn frame, and that
"while a left press is held, and while the game takes no player input, the hover is not looked up
again". Our client now keeps the hover while a left press is held, while the options menu is open
and while the window lacks the input focus (S-0201 Terms, *cycle* and *stand-still*; claims 8, 9,
15 and 16). Our engine has no freeze in its client loop, so frozen cycles (S-0201 claim 10) do not
arise yet.

Two readings of "takes no player input" meet at an order block (S-0201 Terms; the first ticks of a
game, claim 14, and a block made by the map's trigger actions, a chrono flash's included, claim 11):

- S-0201 Terms say a cycle that is not frozen takes the player's input, and an order block's
  cycles are not frozen: by this reading the hover is looked up in every drawn frame of a block.
- S-0111 claim 20 (*Blocked*) says an order block "locks the input and hides the cursor": by this
  reading the game takes no player input during a block, and the hover is held.

The client code is `Session::frame_hover` and `Session::hover_held` in `crates/app/src/game/session.rs`.

A second, smaller point: claim 35 (c) finds the object through the cell under the cursor. While a
left press is held and the pointer moves (a band-box drag, claim 18) or the view moves under a
still pointer, our client keeps the whole lookup of the last frame before the hold, the cell
included, and so the cursor's action and the hover mark stay those of the held object or held cell.

## Questions

1. During an order block (the first ticks of a skirmish, or a chrono flash), with the pointer still
   over the battlefield view and no button held, when a unit moves into or out of the pointer's
   region, does the hover (the cursor's image over it, its hover health bar) change in the frame the
   unit moves, or only once the block ends?
2. While a left press is held and the pointer is dragged to another cell, does the cursor keep
   showing the action for the object or cell under the pointer when the press began, or does it
   follow the cell now under the pointer (with the held object, or none)?
3. While a left press is held, when the hovered object leaves play (destroyed) or becomes one the
   viewer cannot find (claim 35 (d), for example it cloaks fully), does the hover health bar and the
   cursor's action on it stay until the release?
4. When the pointer enters the battlefield view while a left press that began elsewhere (over the
   sidebar) is still held, is an object under it looked up once, or is there no hover until the
   release?

## Possible observation

A skirmish on a small flat map. For question 1: at the start of the game, before the first order
can be given, a unit of the player's moves under a still pointer (or a chrono flash trigger runs
while a unit drives under the pointer); record the cursor image and the hover health bar per drawn
frame. For questions 2 to 4: hold the left button over one of the player's units, then drag the
pointer to an empty cell, and separately have the unit destroyed or cloaked under the held press;
record the cursor image and the health bars per drawn frame until the release.

## Current workaround

1. The hover is looked up in every drawn frame of an order block, as outside one (S-0201 Terms:
   a cycle that is not frozen takes the player's input). Marked `design: render (Q-3321)`.
2. to 4. While a left press is held the last lookup is kept whole, object and cell, as claim 35 (e)
   reads; an object that has left play is not shown by it (the hover mark and the cursor's target
   test whether the object is still in play). A hold that begins with no hover (the pointer off the
   battlefield view) makes one lookup when the pointer comes over the view and keeps that.
