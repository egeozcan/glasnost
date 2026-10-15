---
id: Q-0719
title: Which targets are dropped after a successful bridge attempt from area damage
status: open
related: [S-0511, S-0309]
answered-by: []
---

## Need

S-0511 claim 1 says that "when an attempt from area damage succeeds, every object whose target is
the centre cell itself drops that target". The spec does not say whether this is the attack target
only, or also a destination, an attack-move memory or a queued target, and whether a unit that
drops its attack target then stops firing at once or returns to its previous behaviour. The code
in `crates/deterministic/sim/src/bridges/mod.rs` (`drop_cell_targets`) needs the rule.

## Questions

1. Which of an object's references to the centre cell are cleared: its attack target, its
   destination, its attack-move memory?
2. Is a unit that loses its target this way given any new behaviour (for example back to guard)?
3. Are objects whose target is another cell of the lost section affected?

## Possible observation

S-0511 scenario B10's target variant, with a second unit ordered to move to the centre cell (on
the ground below the bridge) and a third forced to fire at a neighbouring cell of the same section.

## Current workaround

Only the attack target is cleared, exactly as when a unit's target is lost (S-0309 claim 23 is not
applied: no attack-move memory is formed); destinations and other cells are left. The choice is
marked in the code with this request's ID.
