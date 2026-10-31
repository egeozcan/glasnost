---
id: Q-2139
title: The rest of the interface while the route-mode key is held
status: answered
related: [S-0213, S-0214, S-0512]
answered-by: [S-0213]
---

## Need
S-0213 claims 16 to 16g and table T2 say what battlefield clicks and the
guard, stop, scatter and deploy keys do while the route-mode key is held.
They do not say what the other inputs do then, nor what happens when the
key's release never arrives. The client has to decide. Code:
`crates/app/src/game/session.rs` (`route_key`, `route_click`, `key`,
`group_key`, `pointer_up`) and `crates/app/src/game/client.rs`.

## Questions
1. While the route-mode key is held, do the control-group keys (select,
   add, make, centre: S-0214 claim 33) and the type-select role (claim 32)
   act as usual? Does selecting a group clear the selected route point
   (claim 16e: "selecting any object")?
2. While the key is held, what do sidebar clicks do: arming a superweapon
   (S-0512 claim 25), starting sell, repair or power mode, or readying a
   building for placement? If one of those tools is active, does a
   battlefield click use that tool or route mode?
3. Does a band-box drag with the key held select as usual, or is it refused
   or ignored?
4. When the game window loses the keyboard focus (or the options menu is
   opened) while the key is held, are the routes committed as on a
   release, left uncommitted until the next release, or something else?
5. Does the delete key act on the selected route point only while the
   route-mode key is held, or whenever a point is selected?

## Possible observation
Hold the route-mode key with units selected and press a control-group
digit, then click a superweapon icon, then alt-tab away; release elsewhere
and come back, and check whether the units move.

## Current workaround
Question 1: group keys and the type-select role act as usual; a selection
they make does not clear the point selection (only a battlefield selecting
click and a short right click do). Question 2: sidebar clicks act as usual,
and a battlefield click with sell, repair, power, placement or superweapon
targeting active uses that tool, not route mode. Question 3: drags select
as usual. Question 4: losing the focus counts as the release and commits
(our design). Question 5: whenever a point is selected.
