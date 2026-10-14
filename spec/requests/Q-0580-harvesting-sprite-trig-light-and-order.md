---
id: Q-0580
title: The harvesting sprite's sine, light, depth and order against the shadow
status: open
related: [S-0402, S-0205, S-0105, S-0208, S-0102, S-0109]
answered-by: []
---

## Need

S-0402 claim 5a and F8 (r20, answering Q-0550) give the vehicle harvester's harvesting sprite.
Our engine draws it (`mapview::harvesting`, `mapview::build`). Five points are open.

1. **Sine.** F8 writes θ = RZ₅₃((*f̂* − 16383) × *c*) and then plain sin θ and cos θ, "in the
   original's floating-point round-toward-zero mode, not integer lookup distances". S-0205 claim 6
   says the original computes no sine or cosine of game state with a library function and that our
   engine must use the tables wherever a spec gives one. S-0208 F3, which has the same angle form,
   writes the table sine and cosine sinT and cosT of S-0105 F1 explicitly. The two readings differ
   by one lepton in the anchor at some facings: with `quirks.table_direction` on and *q* = 2, the
   exact sine gives *X* − 1 where sinT gives *X*; with it off and *q* = 0, (*X* + 29, *Y* − 1)
   against (*X* + 30, *Y*).
2. **Light.** Claim 5a gives the palette (the animation palette, no house remap) but not the
   brightness the image is drawn with: the cell's light, the vehicle's, or none.
3. **Depth.** S-0102 claim 13's depth rows name none for this image, so whether it is depth-tested
   against the terrain and other objects, and at what depth, is open.
4. **Order against the shadow.** "Composited before the vehicle's body": is it also before the
   vehicle's shadow, or between the shadow and the body?
5. **Current speed.** Claim 5a suppresses the sprite for a ground driver with "an active track with
   positive current speed". S-0207 has a speed fraction ρ (Terms) and a per-tick step
   s = trunc(s₂ × ρ), which is 0 while ρ is small but positive.

## Questions

1. Does F8's anchor use the exact sine and cosine rounded toward zero, or the table sine and
   cosine of S-0105 F1 (as S-0208 F3 does)?
2. With what brightness is the harvesting sprite drawn: the light of the vehicle's cell, of the
   vehicle, full brightness, or another rule?
3. Is the sprite depth-tested, and if so at what depth (for instance the vehicle's own depth, or
   its anchor's)? Can terrain or a neighbouring object in front of the harvester cover it?
4. Is the sprite drawn before or after the vehicle's shadow?
5. Is "positive current speed" a speed fraction ρ above 0, or a positive per-tick step s? Is the
   sprite shown on a tick where the harvester is on a track with ρ > 0 but s = 0?

## Possible observation

A harvester loading on a lit and an unlit cell, facing each of the 8 directions, captured one
frame per facing: compare the sprite's pixel offset with both anchor readings, and its colours
with the cell and the vehicle light. Put a tall building just south of the harvesting cell and see
whether it covers the sprite. A harvester with a large shadow shows whether the shadow covers the
sprite.

## Current workaround

1. The exact sine and cosine, each rounded once toward zero (`glasnost_num::bigmath::sin_cos`),
   following F8's text; noted in `mapview::harvesting::compute_offset`.
2. The cell's unscaled ground light.
3. Drawn upright at the vehicle's depth less 2, depth-tested, marked provisional.
4. Before every item of the vehicle, its shadow included.
5. ρ > 0 on an active track suppresses it (`glasnost_sim::econ::harvest::display_shown`).
