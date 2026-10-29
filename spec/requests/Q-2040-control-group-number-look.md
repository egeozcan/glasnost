---
id: Q-2040
title: How the control-group number looks: font, alignment, border, colour sample and shading
status: answered
related: [S-0111, S-0103]
answered-by: [S-0111]
---

## Need

S-0111 claim 17 fixes the control-group number's digit ((*γ* + 1) mod 10) and its placement point
((−4, −39) from the pip-scale origin *o* for infantry, (−4, −36) for others, buildings included).
It says the digit is drawn in the owner's house colour inside a small border of the same colour.
The Open questions leave the font, how the text is aligned to the point and the border's size
unestablished, and the colour is of medium confidence.

Our client now draws the number (M8 A4 round 5 seams, `crates/app/src/mapview/combat.rs`,
`group_number_image` and `Cache::group_number_item`, drawn from `Live::draw_status`). Every choice
below the placement point is ours and marked design. A spec answer would replace them.

## Questions

1. Font: which glyphs does the game draw the digit with (the size of a digit in pixels, and
   whether all ten digits are the same width)? Are they the game's small in-game text font?
2. Alignment: which point of the drawn text, or of its border, sits at the claim-17 placement point:
   the top-left corner, the top centre, the baseline's left end, or something else?
3. Border: how wide is the border line, and how far is it from the glyph on each side? Is it a
   closed rectangle around the digit alone, and is the space inside it filled or left clear?
4. Colour: which sample of the owner's house colour do the digit and the border use? Is it the
   house's displayed (radar/text) colour of S-0103 claim 15, the brightest entry of its remap ramp,
   or another entry? Do the digit and the border use the same entry?
5. Shading: claim 2 lists the status images that the cell's lighting shades, and the group number
   is not among them. Is the number drawn unshaded (the same brightness under any light)?
6. Overlap with other objects: when two selected objects' status extras overlap on the screen, does
   one object's group number cover the other object's health bar or pips, in an order that follows
   claim 2's object order, or is the number drawn above every object's status extras?

## Possible observation

Scenario OV12 with close-up screenshots at 100% scale: a vehicle in groups 1, 5 and 10 of a
red-remap house and of a blue-remap house, on lit and dark ground. Read off the glyph pixels, the
border pixels and their colour, and the offset of the top-left lit pixel from the anchor. Add two
selected grouped vehicles placed one cell apart, so that one's number overlaps the other's health
bar.

## Current workaround

Our own design, marked `// design:` in the code. Digits are our own 3 by 5 pixel glyphs inside a
1-pixel border with a 1-pixel gap (a 7 by 9 pixel image). The image's top-left corner sits at the
claim-17 placement point. The digit and border take the owner's displayed house colour (the RGB
the radar and texts use, S-0103 claim 15), or white for an object with no owner. It is drawn
unshaded, after the object's own health bar and pips (so it covers them, as claim 10 says), in the
status group, in the object order of the other status extras. No simulation code depends on any
of this.
