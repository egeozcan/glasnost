---
id: Q-3019
title: Which objects are drawn with a named palette, and how is such a palette built?
status: open
related: [S-0103, S-0104]
answered-by: []
---

## Need

S-0104 claims 3, 5 and 17 make the *named palettes* (S-0104 Terms) tintable tables: a global
lighting effect rebuilds them with its set's tint, the choice "normal" gives them their own tint
back, and a trigger-action tint change retints them. S-0104 Terms says they are house-colour
palettes built from a palette file other than the unit palette, "for objects drawn with a palette
of their own", and that which objects use them and how they are built is covered neither there nor
in S-0103. S-0103's Open questions ("Types with their own palette") leave open whether stock data
uses this and which key selects it.

Render issue #75 (GitHub, render audit #154) asks that every tintable table follow the global
effects. The client (`crates/app/src/mapview/build.rs`, the house-colour tables and
`Painter::retint_houses`) now retints every 53-shade house-colour table it draws with. It builds no
named palette, because no spec says which object draws with one, so it cannot retint one either.

## Questions

1. Which objects are drawn with a palette of their own rather than the unit palette: which kinds
   (infantry, vehicles, aircraft, buildings, animations, projectiles, overlays, terrain objects),
   and which key of which file (rules or art, and in which section) selects the palette file?
2. How is such a palette built: from which palette file, found how (the file name as given, with a
   theater suffix, through the usual file search), with how many shade rows, and with which
   house-colour recolouring (the same range and remap as the unit palette's 53-shade tables, or
   another)?
3. When is it built: once per palette file and house colour when the game starts, or the first time
   an object needs it? If it can be built later, which tint does it take while a global effect is in
   force (S-0104 claim 2's third column, or the global choice)?
4. What is drawn when the named file cannot be found?

## Possible observation

Give a stock vehicle type and a stock infantry type a palette key naming a test palette file in a
loose test archive (one whose colours differ visibly from the unit palette), in each candidate file
and section. Capture the objects under neutral lighting, under an ion storm and under a nuke flash,
and compare their pixels with shade tables built from the unit palette and from the test palette.
Repeat with the test palette absent.

## Current workaround

No object is drawn with a named palette: every object takes the palette its kind uses today
(S-0103), and only those tables follow the global effects (S-0104 claims 3 and 5). The part of
#75 that concerns named palettes is left open in `docs/m8-claims/render-issues.md` (batch R5, #75).
