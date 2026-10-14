---
id: Q-0417
title: Radar colour of a house: which 8-bit triple of its colour definition
status: answered
related: [S-0410, S-0103]
answered-by: [S-0103, S-0410]
---

## Need

S-0410 claim 8 draws a shown object in "the 8-bit triple of the house's
colour definition, S-0103, converted as Terms *radar colour* says". A colour
definition (S-0103 claim 4) is three integers that the house-colour ramp
reads as hue, saturation and value, not a red-green-blue colour. It is not
clear whether the radar uses those three stored values directly as red,
green and blue, or a red-green-blue colour derived from the definition (for
example one entry of its ramp, and which one). The code is
`crates/app/src/mapview/radar.rs` (`HOUSE_COLOURS`, a placeholder).

## Questions

1. Which red-green-blue triple does a house's object show on the radar: the
   definition's three stored values taken as red, green and blue, or a colour
   computed from them? If computed, which one (for example which ramp entry,
   before or after the unit palette is applied)?
2. Do a house's buildings and its other objects show the same colour?

## Possible observation

A skirmish with players in several lobby colours, and a loose rules copy that
changes one colour definition to a value whose stored triple differs
clearly from its ramp colours (for example a low hue with full saturation).
Read the radar pixels of that player's objects.

## Current workaround

The radar shows each house's colour as the definition's stored triple read
as hue, saturation and value and converted by the same conversion as the
ramp's brightest entry (S-0103), our reading, marked provisional. Listed in
docs/development.md (interim rendering rules).
