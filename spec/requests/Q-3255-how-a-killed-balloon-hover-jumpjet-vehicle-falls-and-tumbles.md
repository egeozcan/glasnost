---
id: Q-3255
title: How a killed balloon-hover jumpjet vehicle falls and tumbles
status: open
related: [S-0502, S-0113, S-0312]
answered-by: []
---

## Need

An issue report says that in a skirmish on an imported install, a killed airship-type bomber falls and tumbles the way a rocketeer-type unit does. The lead is sure that in the original it falls **straight down** while **spinning about its vertical axis**.

We traced the crash on synthetic data: a crashable balloon-hover vehicle with the jumpjet locomotor, killed once while cruising at full speed and once while hovering with no order. Our engine matches the claims as written:

- **S-0502 claim 18 (Fall).** Each tick of *crashing* applies the extra drop *k* after F4. The movement heading's target is set 11 520 facing units ahead, so the heading spins at S-0206 claim 14's rate. The current speed keeps evolving under F1, and the unit keeps stepping along the heading (F5). The body facing follows the heading each step used (claim 10).
  - Killed while moving: the current speed falls toward the crash-time maximum speed and stays above 0. The unit steps along its spinning heading every tick, so it drifts in a circle-like path while it falls.
  - Killed while hovering: the current speed is 0, so it falls straight down while its body spins.
- **S-0113 claim 16 (rocking while crashing).** A non-infantry object gets a roll rate and a pitch rate of 0.1 × *u*₃ ≥ 0. Each angle grows with no limit. For a balloon-hover type, the pitch is raised to −π/4 when below it, "it has no upper bound", and the roll is held within ±π/4. So the pitch grows without limit and the body turns nose over tail during a long fall: a tumble. The claim rates the balloon-hover lower pitch bound as unmeasured.
- **S-0312 claim 16 and its table.** A balloon-hover type's crash end sets off its death weapon and shows none of claim 9's animations.

So the two observations differ from the claims in two places:

- the drift of a unit killed while moving;
- the unbounded pitch.

We do not change either without a spec.

## Questions

1. Does a crashing balloon-hover jumpjet vehicle keep stepping along its movement heading (S-0502 claim 18, F1 and F5)? Or is its horizontal motion stopped when the crash starts, or when it enters *crashing*? If it is stopped, is that true of every jumpjet vehicle, or only of balloon-hover types? Does the current speed go to 0 at once or brake under F1?
2. For a balloon-hover type, does the pitch have an upper bound while it crashes (S-0113 claim 16)? For example, is it held within ±π/4 like the roll? Or is the pitch rate not applied to such a type, so that it stays level while it spins?
3. Is the spin seen from above the movement heading's spin of S-0502 claim 18 drawn as the body facing? Or does a balloon-hover type turn its body by some other rule while it crashes?
4. Do the answers depend on whether the unit was moving, hovering or attacking when it was killed?

## Possible observation

Scenario J12 for a balloon-hover jumpjet vehicle, killed twice: once while cruising at full speed to a far cell, and once while hovering over a target. Record each tick from the kill to the crash end:

- the world position and height;
- the current speed and movement heading;
- the drawn body facing, roll and pitch.

## Current workaround

None. Our engine follows S-0502 claim 18 and S-0113 claim 16 as written: a unit killed while moving drifts, and its pitch has no upper bound. We change nothing until this is answered.
