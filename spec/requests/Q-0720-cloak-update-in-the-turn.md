---
id: Q-0720
title: Where in an object's turn the cloak update runs
status: answered
related: [S-0516, S-0203]
answered-by: [S-0516]
---

## Need

S-0516 says the draws of claims 5, 8 and 33 are taken in the object's own turn (State and
ordering). It does not say where in the turn the cloak update falls relative to the object's
movement, firing and behaviour. It also does not say whether, on one tick of a cloaking object,
the claim 8 abort test comes before or after the stage step. The code needs the rule in two places:
`crates/deterministic/sim/src/cloak/machine.rs` (`turn`, `step`) and the call in
`crates/deterministic/sim/src/econ/events.rs` (`turn`).

## Questions

1. In a unit's turn, does the cloak update (start, stage step, keep test) come before or after the
   unit moves? Does it come before or after the unit fires on that tick?
2. In a building's turn, does it come before or after the building's behaviour (firing)?
3. On a tick when a cloaking unit is at low health (claim 8), is the abort draw taken before that
   tick's stage step? In other words, can an abort happen on the tick the stage would reach ⌈N/2⌉?

## Possible observation

- A self-cloaking unit at full health is ordered to fire at a target in range on the tick its cloak
  delay ends. Record the tick it fires and the tick it starts cloaking.
- A cloaking unit at yellow-to-red health, with a stage count that gives c = 1, repeated many
  times. Does an abort ever happen on the completing tick?

## Current workaround

- The cloak update runs at the end of the object's turn: for units after movement and firing, for
  buildings after their behaviour.
- The claim 8 abort test comes before the stage step.

The choice is marked in the code with this request's ID.
