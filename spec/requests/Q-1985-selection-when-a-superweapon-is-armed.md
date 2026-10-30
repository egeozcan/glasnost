---
id: Q-1985
title: What happens to the unit selection when a superweapon is armed from its icon?
status: answered
related: [S-0513, S-0512, S-0214]
answered-by: [S-0512]
---

## Need

S-0513 claim 1 says: "A non-empty unit selection takes precedence: while units are selected, the
cursor offers their actions, not the superweapon's." S-0512 claim 16 arms a type when its icon is
selected, and claim 23 says a recognised action "enters aiming". Neither spec says whether
selecting the icon changes the unit selection.

If the selection is kept, a player who had units selected arms the superweapon, and the next
battlefield click acts for the units. The aim image of S-0214 table CS never shows, and the
superweapon cannot be fired until the player deselects by hand (a right click cancels the aim
under S-0214 claim 28, so it has to be a deselecting left click or a key). Play testing (M8 A4
round 5, note 27) read this as "can't fire with a unit selected".

The client has to choose between keeping the selection and clearing it when the icon arms
the type.

## Questions

1. When the player selects a ready superweapon icon that enters aiming, while units are
   selected, are the units still selected afterwards?
2. If they stay selected: does the battlefield cursor show the units' actions until the
   selection is emptied, and how does a player normally empty it without cancelling the aim
   (which input, and does that input cancel the aim)?
3. Does the answer differ between the mouse and a keyboard shortcut that selects the icon,
   or between a type that aims and a type with no aim action (which fires at once, S-0512
   claim 23)?
4. Does a teleport's source pick (S-0513 claim 33), which arms the second stage, change the
   selection?

## Possible observation

Select two of your own tanks, then select a ready nuclear-strike icon and hover the battlefield:
note the cursor image and whether the tanks' selection marks stay. Click empty ground and note
whether the tanks move or the strike fires. Repeat with the icon selected by its keyboard
shortcut, if it has one.

## Current workaround

Our own design: selecting an icon that enters aiming clears the unit selection, on the mouse
and keyboard paths alike, so the next battlefield click aims (S-0513 claim 1's precedence is kept
for a selection made after arming). A type with no aim action and a source pick leave the
selection alone. The code is `Play::cameo_click` in `crates/app/src/game/play.rs`, marked
`design:` with this request.
