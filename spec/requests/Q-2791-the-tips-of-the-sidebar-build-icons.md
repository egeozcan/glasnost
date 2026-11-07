---
id: Q-2791
title: The tips of the sidebar's build icons
status: open
related: [S-0411, S-0111, S-0004, S-0015]
answered-by: []
---

## Need

S-0111 gives the name tip for objects on the battlefield. It states that a tip of a sidebar
control is not its subject (claim 20 and the scope note near its start), and S-0411 gives no tip
for the sidebar's build icons. So our sidebar shows no name for an icon: a player cannot learn
what an icon builds, or what it costs, without building it. Seam sweep 6 left the row "the
sidebar's names through the table" (S-0111, S-0015) for this reason.

The display name is already resolved through the string table (S-0004 claim 17, S-0015), so
only the sidebar's own behaviour is missing. The app code is the sidebar in
`crates/app/src/mapview/sidebar.rs` and the name tip in `crates/app/src/game/client.rs`.

## Questions

1. When the pointer rests on a build icon in the sidebar strip, does the game show a tip? If so,
   after what wait, and does that wait start again when the pointer moves within the same icon, or
   onto another icon?
2. What does the tip show: the type's display name only, or also its cost? In what format is the
   cost written, and does it change with the owner's cost modifiers?
3. Where is the tip drawn relative to the pointer or the icon, with what font, colours, box and
   border? Is it clipped at the screen's edges, and if so how?
4. Does an icon that cannot be built now (a missing prerequisite shown greyed, a building in
   progress, a ready building waiting to be placed, a superweapon icon charging or ready) show the
   same tip, a different one, or none?
5. Do the sidebar's other controls (the tab buttons, the repair, sell, power and waypoint buttons,
   the strip's scroll buttons) show tips? If so, what text, and from which labels of the string
   table?
6. When does the tip disappear: on a press, on leaving the icon, after a time, or when the icon's
   state changes under it?

## Possible observation

In a skirmish, rest the pointer on each kind of icon (an infantry unit, a vehicle, a building, a
superweapon) in each of its states, and on each sidebar button. Record the tip's appearance time,
text, position and when it goes away, at two screen resolutions.

## Current workaround

None in code: the sidebar shows no tip, and no `todo!()` stands for it. The app is unchanged
until this is answered.
