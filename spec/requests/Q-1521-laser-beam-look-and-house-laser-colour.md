---
id: Q-1521
title: How a laser beam is drawn, and where a house's laser colour comes from
status: open
related: [S-0518, S-0615, S-0102, S-0103]
answered-by: []
---

## Need
Three specs ask the client to draw laser beams and none says how a beam is rasterised:
S-0518 claim 6 (a prism support beam: L_s ticks, 3 pixels thick, "the house's laser colour";
the master's own beam 5 pixels thick when it has supporters), S-0615 claims 15 and 16 (the
disc laser's arc and final beam: inner, outer and spread colours, an intensity that fades) and
S-0615's Open questions ("Beam look ... no approved spec owns it"). S-0102 claim 2 draws beam
effects after all layers. The phrase "house's laser colour" is not defined in any spec or in the
glossary, and no import mapping names its key. Our client (`crates/app/src/mapview/live.rs`,
`take_effects`) draws each beam as a row of nine small crosses, which the lead's review of
2026-10-19 rejected ("no beams rendered" on the prism towers).

## Questions
1. Where does a house's laser colour come from: a field of the house's colour definition, of its
   country, a game-wide rules value, or the house colour itself? What form does the value take,
   and what is used when it is absent?
2. How is a beam of thickness *w* pixels drawn between two projected points: which pixels does it
   cover (a line of width *w* centred on the segment, a core plus a halo), and how do the inner,
   outer and spread colours combine with what is already on screen (added with saturation,
   blended, replaced)?
3. How does a beam's intensity fade over its duration (S-0615: "from full intensity to half" and
   "to none")? Is the fade linear per tick, and does it scale the colours before they combine?
4. A prism support beam has only "the house's laser colour". Which of the inner, outer and spread
   colours does that colour fill, and what are the others?
5. Does an ordinary weapon whose shot is drawn as a laser (a prism tower's own shot, other laser
   weapons) draw a beam of the same kind? With which thickness when the master has no supporters,
   and for how long?
6. Are the beam's end points the projected fire points with the height lift (S-0101 F1, F2), and
   is a beam clipped by anything (shroud, the screen edge only)?

## Possible observation
Build three prism towers of one house next to each other and attack a target in reach so that
supporters fire. Capture each frame from the first support beam on, at the highest detail, and
compare beam pixels against the house colour and the screen pixels underneath. Repeat with a
second house of another colour, and with a floating disc's laser against a vehicle.

## Current workaround
Interim rule (docs/development.md, *Visual review 2026-10-19*): beams with a spec'd thickness and
duration are drawn as straight lines of that thickness between the projected end points, after
the layers (S-0102 claim 2), opaque, for their duration. The colour of a prism support beam is the
owner house's colour (the brightest entry of its remap ramp) until question 1 is answered; the
disc laser uses its inner colour with the outer colour as a one-pixel edge, fading by S-0615's
intensity rule as a darkening of those colours. This look is ours and provisional.
