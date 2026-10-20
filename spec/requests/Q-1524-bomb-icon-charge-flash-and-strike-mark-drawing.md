---
id: Q-1524
title: How a planted bomb's icon, a charged building's flash and an air-strike mark are drawn
status: open
related: [S-0519, S-0104, S-0102, S-0103, S-0111]
answered-by: []
---

## Need
S-0519 says when three things are shown and leaves out what they look like:
- claim 10 and F3 give the bomb icon's visibility and its image frame (0 to 11), but no spec names
  the icon's image file, its palette, where it is placed over the carrier, or its layer;
- claim 14 and F4 say a building on which a charge is planted "flashes for *L* ticks", but not
  what the flash does to the building's picture;
- claims 17 and 19 say a marked building "is tinted with the game-wide laser-target colour"
  (a position in the colour-adjustment table) and that S-0104 owns the brightness, while S-0104
  lists both the flash and the strike mark among its Open questions.

The game client (`crates/app/src/mapview/live.rs`, `draw_effects`) drew our own placeholders for
all three (a red and white square, an orange diamond, a cyan ring). The lead's review of
2026-10-19 rejected them ("the charge is just an orange opaque shape", "just a blue circle, looks
cheap").

## Questions
1. Which image file is the bomb icon drawn from, with which palette and brightness? Where is it
   placed relative to the carrier (its reference point, its health bar, its top), and in which
   layer and order relative to the carrier and the status extras of S-0111? Is it drawn over
   other carriers' icons in any particular order?
2. While a charged building flashes, what changes in its picture: its brightness (to which
   value, and does it alternate between frames), a colour, or something else? Which parts are
   affected (body, slot animations, voxel turret), and does the flash alternate on a tick
   pattern? How does it combine with protection (S-0102 claim 34) and with damage states?
3. How is a marked building tinted with the laser-target colour: is the colour-adjustment entry
   applied as S-0102 claim 34's colour mask (bitwise union), as a brightness change, or
   otherwise? Which parts of the building take it, and does it pulse or stay constant while the
   mark lasts?
4. Are any of the three drawn differently for the planter's or striker's own house and for other
   houses (beyond claim 10's visibility rule)?

## Possible observation
Plant a bomb on a test vehicle with named test data giving a long timed delay; capture every
frame at the highest detail and compare the icon pixels with candidate image files under the
anim, unit and overlay palettes. Plant a charge on a test building and capture each frame of the
flash, with and without the building damaged and with a force shield. Designate a test building
for an air strike and capture it with the laser-target colour set to two different table
positions in named test rules.

## Current workaround
Interim rule (docs/development.md, *Visual review 2026-10-19*): the game client draws nothing for
the bomb icon, the charge flash or the strike mark; the viewer's debug layer keeps our
placeholders. The simulation state (bomb, charge countdown, mark) is unchanged.
