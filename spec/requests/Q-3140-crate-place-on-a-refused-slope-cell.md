---
id: Q-3140
title: What a crate placement does when the slope condition refuses its overlay
status: answered
related: [S-0520, S-0009, S-0202]
answered-by: [S-0520]
---

## Need
S-0009 claim 12 makes the slope condition part of creating an overlay (the cell's slope shape is
at most 4, or the overlay type is the one at list index 178) and says "In play it applies to every
overlay placed." S-0202 claim 10 repeats that an overlay is placed only where S-0009 claim 12 allows
one, "which includes the slope condition stated there".

S-0520 claim 3 lists the tests a random crate's candidate must pass: the nearby search finds a cell,
the cell is inside the height-aware usable area, and it has no overlay. None of them reads the slope
shape, and the nearby search's settings ("no further test") do not either. Then "the first free crate
place receives random contents and a water-crate overlay on *water*, or a wooden-crate overlay
otherwise", and the lifetime request follows. The trigger-crate and death-crate placements use the
same search and tests.

So a cell of slope shape 5 or more can pass every test of claim 3 while S-0009 claim 12 refuses its
crate overlay. No claim says what then happens to the crate place and the lifetime draw. The edge case
`quirks.crate_phantom_place` covers only "a terrain object prevents its overlay appearing".

Our engine creates the crate overlay on such a cell, as if there were no slope condition in play.
(Issue #349.)

## Questions
1. When the slope condition refuses a crate overlay at a cell that passed claim 3's tests, does the
   crate place still become occupied and take its lifetime draw (as `quirks.crate_phantom_place` on
   does for a terrain object), or does the attempt fail with no place and no lifetime draw?
2. If the attempt fails, does a random placement go on to its next candidate attempt (two more ranged
   requests, within the 1000 attempts), or does the placement end there?
3. Do the trigger crate (S-0520's trigger-crate row of T5) and the death crate behave the same way?
4. Does `quirks.crate_phantom_place` govern this case too, or is the slope refusal outside it?

## Possible observation
A synthetic map with one flat cell surrounded by cells raised so that a cell next to it has slope
shape 5 or more after recalculation, and a trigger that places a crate on that sloped cell. Look for a
crate at the cell, and count crates later placed by the regeneration timer: whether the place stays
taken until expiry answers question 1. A random placement on a map of mostly steep cells answers
question 2 by where (and whether) the crate first appears.

## Current workaround
The crate overlay is created whatever the cell's slope shape (`crates::finish` calls
`overlays::place_overlay`, which has no slope test). Issue #349 is left open until this is answered.
