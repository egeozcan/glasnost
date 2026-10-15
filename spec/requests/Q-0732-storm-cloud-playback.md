---
id: Q-0732
title: Frame count and frame timing of a storm cloud for its strike and end
status: answered
related: [S-0513, S-0109]
answered-by: [S-0513]
---

## Need
S-0513 claims 16 to 19 say:
- a cloud strikes once its animation's current frame exceeds ⌊F / 2⌋, F being its image's
  frame count;
- a cloud counts as present until it reaches its last frame.

The simulation keeps no animation playback clock and no image frame counts. The rules give it
only the middle frame μ (S-0109 claims 14 and 25) and the creation draws. Our storm needs two
ticks for each cloud: the tick of its strike, and the tick it stops being present.

## Questions
1. A cloud animation is created in tick t. In which tick does its current frame first exceed
   ⌊F / 2⌋, as a function of its imported rate and frame count?
2. In which tick does it reach its last frame, so that it stops being present?
3. Is the middle frame μ that S-0109 uses always ⌊F / 2⌋ for these images?

## Possible observation
Run S-0513 scenario SW7 part d with cloud images of known frame counts and rates. Record the
strike tick and the tick the cloud disappears.

## Current workaround
Interim rule (docs/development.md, *Simulation (M6b: superweapons)*):
- F = 2μ, so a cloud strikes when its frame exceeds μ.
- The cloud advances one frame per stage delay d. d is the creation's drawn stage delay when one
  is drawn, and 1 otherwise.
- The frame in tick t is ⌊(t − t₀) / d⌋.
- A cloud type whose μ is unknown has F = 0: it never strikes and is gone at once.
