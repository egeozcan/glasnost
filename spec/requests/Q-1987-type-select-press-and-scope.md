---
id: Q-1987
title: What does the type-select role do when pressed alone, and which objects does it take?
status: open
related: [S-0214, S-0701]
answered-by: []
---

## Need

S-0214 Terms and claim 12 define the type-select role as a held role: "while it is held during a
*select* or *toggle select* click, the click applies to every object of the clicked object's type
that is on the screen", and "the type-select role pressed twice selects that type across the whole
map, instead of on the screen, when the second press comes at most 500 milliseconds after the
first". Claims 19 and 29 cover a type-select drag.

Scenario text in the claim 12 tests reads: "Press the type-select role twice with about 200 ms
between the presses, and later twice with about 1 s between them. Expected: the first pair
selects that type across the whole map, the second only on the screen." No click is named
there, which suggests that pressing the role alone already acts on the current selection's
types. Claim 12 names only clicks. Play testing (M8 A4 round 5, note 31) asked for a key that
selects every unit of the selected type, so the client has to settle this and the points below.

## Questions

1. Does pressing the type-select role alone, with no click, change the selection? If it does:
   is it every object of each selected object's type, on the screen (one press) or across the
   map (a double press within the interval), and in which addition order (selection order of
   the seeds, then candidate order or creation order)? Is it silent under S-0701 claim 17?
2. "Every object of the clicked object's type": does that take objects of the same type owned
   by an ally, or by another house the player can select (for example a captured or
   mind-controlled one of a different owner), or only objects of the clicked object's owner?
3. With the role held, a *toggle select* click on an object that is already selected: does it
   remove every selected object of that type, only the clicked one, or nothing?
4. With the role held, a *select* click: is the result exactly the type's objects in candidate
   order (claim 12's front/end rule applied), or does the clicked object come first? Does the
   selection response of S-0701 claim 17 still play for the clicked object?
5. Does the double press widen only the presses that follow it, until the role is released, or
   does the wide state persist for a while after release?
6. Does a *nothing* click (an object clicked that is already the sole selection, S-0214 row B6)
   with the role held expand the type?

## Possible observation

Select one own tank of a type with several more on the screen and a few off it. Press the
type-select key once without clicking and note the selection. Press it twice quickly and note the
selection again. Then put an allied tank of the same type on the screen and repeat with a held
click on the own tank. With the role held, shift-click (toggle select) a selected tank of a type
with three selected and note how many stay selected.

## Current workaround

Our own design, marked `design:` with this request in `crates/app/src/game/play.rs`
(`Play::expand_types`) and `crates/app/src/game/session.rs` (`Session::type_select_key`,
`type_select_click`):

- A press alone expands each type of the current selection, in seed order, over the screen
  candidates (or every object across the map after a double press), silently, by claim 12's
  front/end rule.
- Only objects of the seed's own owner join (an own tank does not take an ally's tanks).
- A held *toggle select* on a selected object removes every selected object of that type.
- A held *select* replaces the selection with the type in candidate order (the clicked object
  speaks as for the ordinary click); a held *nothing* click expands like a toggle.
- The double press widens the role for as long as that press is held.
