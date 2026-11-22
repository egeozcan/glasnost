---
id: Q-3369
title: A battlefield press that begins while an order block is in force
status: answered
related: [S-0214, S-0201, S-0111]
answered-by: [S-0214]
---

## Need

S-0214 r101 claim 35 (e), *A held press and an order block*, says that when an order block
(S-0201 Terms) begins, a battlefield press that is held is abandoned as in claim 34 (e), and that
"a button that is still down when the block ends holds no press, so the hover is looked up again
from the block's end". Its Terms entry *battlefield press* is held "until its release, until claim
34 (e) or (f) abandons it, or until an order block begins". Neither says what happens to a left or
right press that **begins** on the battlefield view while a block is already in force (the first
ticks of a game, S-0201 claim 14, or a block made by trigger actions or a chrono flash, S-0201
claim 11). Our client must decide it (`Session::pointer_down` and `Session::pointer_up` in
`crates/app/src/game/session.rs`).

Read as the Terms entry gives it, such a press is a battlefield press held until its release, since
the block's beginning is already past. Read as the sentence on a button still down at the block's
end gives it, such a press holds nothing once the block ends. The two readings differ in what the
release does after the block (a click, a band box, a right click or a scroll drag, or nothing), in
whether the hover is looked up after the block while the button stays down, and in which cursor
image shows then.

## Questions

1. While an order block is in force, the player presses the left button on the battlefield view
   over an own unit and holds it until after the block ends, then releases it at the same pixel.
   Is the unit selected at the release (a click of claims 12 and 26), or does the release give
   nothing?
2. The same with a press on an empty cell, a drag of more than claim 18's distance after the block
   ends, and a release: does a band box show and select what it holds?
3. The same with the right button, with no drag: is the release a right click (claim 26: it clears
   the selection), or nothing? And with a drag after the block beyond F8's bound: does the view
   scroll (claim 27)?
4. While such a button is held after the block, is the hover looked up for every drawn frame, and
   does the cursor follow the lookup, as with no press?
5. Does a press made during the block hide the name tip or block it (S-0111 claim 20) differently
   from one made outside it?

## Possible observation

As S-0214 scenario T27 part (m), with S selected and V standing at its cell's centre: during the
block of a game's first ticks or a trigger's input lock, press the left button at V's point 20 ticks
before the block's end and hold it until 40 ticks after the end, then release it there. Record per
tick whether V has a health bar, the cursor image, and the selection after the release. Repeat with
a drag that starts a band box after the block, and with the right button, with and without a drag.

## Current workaround

A press that begins during an order block is kept as before r101: it is a battlefield press, held
until its release, and its release acts as a release made outside a block (the simulation refuses
any order the block forbids). During the block no hover lookup is made and the cursor is hidden,
as claim 35 (e) gives for any block; after the block the held press keeps the hover and its image
until the release. Only a press held when a block begins is abandoned. Marked
`design: render (Q-3369)`.
