---
id: Q-2687
title: The name tip's open points - wrapping, inset, blocking states and where the pointer rests
status: open
related: [S-0111, S-0213, S-0214]
answered-by: []
---

## Need

S-0111 claims 20 to 26 (the answer to Q-2673) give the name tip. `crates/app/src/game/name_tip.rs`
and `Session::update_name_tip` implement them. A few points the claims leave open, or that our
engine has no direct counterpart for, still need a decision. Each one is marked
`design: Q-2687` in the code.

## Questions

1. **Wrapping (claim 24, Open questions).** A line wider than the battlefield view is wrapped.
   Where does it break: before the text unit that would take it past the width, at the last space
   before that, or somewhere else? Does a wrapped line's first unit drop a leading space?
2. **Text inset (claim 25, Open questions).** How many pixels from the box's left and top edges
   does the first glyph's top-left pixel sit?
3. **A box as wide as the view or wider (claim 24, Open questions).** Where is it placed, and is
   it cut?
4. **The further view state that blocks (claim 20, Open questions).** Which player-visible state
   is it: for example while a building is being placed, while a superweapon is being aimed, or
   while the radar is being clicked or dragged?
5. **Locked input and hidden cursor (claim 20).** Which ordinary skirmish situations, if any, lock
   the input or hide the cursor while the pointer is over the battlefield view? Is the options
   menu, open over a paused skirmish, one of them? Is a tip that shows when the options menu opens
   hidden, kept under the menu, or still drawn over it?
6. **A blocked wait end (claim 20).** When the wait ends while blocked (for example during a
   scroll), is the rest's one choice used up, so no tip shows until the pointer moves and rests
   again? Or does the tip show once the block ends, with the pointer still resting?
7. **Resting on the battlefield view (claim 20).** Does the wait run, and does a tip show, while
   the pointer rests over the battlefield view but on an interface element drawn over it (for
   example a message line, S-0213 claim 25), or while it rests over the sidebar or outside the
   window? Does the pointer leaving the window count as a movement?

## Possible observation

Scenario NT1 and NT6 setups of S-0111: a probe display name wider than the view, with spaces, for
1 to 3; a building placement, a superweapon aim and a radar drag held through the wait for 4; the
options menu opened during the wait and while a tip shows for 5; an edge scroll that ends while the
pointer stays still for 6; the pointer resting over a status message line and over the sidebar,
then moved back onto the view, for 7.

## Current workaround

1. A line breaks before the text unit that would take it past the view's width, keeping at least
   one unit per line; no space is dropped.
2. An inset of 2 pixels from the left and top edges, inside the 1-pixel frame.
3. No special case: claim 24's placement applies, and the box is clipped to the view.
4. Nothing else blocks.
5. Our engine never hides the cursor over the battlefield view and has no locked input. A wait
   that ends while the options menu is open shows no tip, and a showing tip is not drawn while the
   menu is open (it stays, and shows again when the menu closes if its lifetime is not over).
6. The rest's choice is used up: nothing shows until the pointer moves and rests again.
7. A tip shows only when the pointer, at the wait's end, is inside the battlefield view and over
   no interface target drawn on it; otherwise the rest's choice is used up and nothing shows. The
   pointer leaving the window hides a tip and stops the wait, as a button does.
