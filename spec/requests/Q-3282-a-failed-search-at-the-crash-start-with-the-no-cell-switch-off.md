---
id: Q-3282
title: A failed search at the crash start with the no-cell switch off
status: answered
related: [S-0502, S-0607]
answered-by: [S-0502]
---

## Need

S-0502 r30 claim 18 (*Destination clear at crash start*) says that when the
crash start's search finds no cell, no re-order follows, and that claim 17's
no-cell rule acts only on a unit that still has health, so a failed search
does not destroy a killed unit again.

S-0502's Edge cases give the alternative no-cell outcome for
`quirks.jumpjet_no_cell_self_destruct` off: at 1 lepton or more above the
surface the unit re-enters *hovering* over its current cell with no movement
point and no commitment; at 0 or less it becomes *grounded*. Claim 18 does
not say whether that alternative also acts only on a unit with health.

In the engine (`crates/deterministic/sim/src/jump.rs`, `stop_with`), with the
switch off, a killed crashing unit whose search fails is set *hovering* with
target height H, no point and no commitment before its first crash update,
so that update's ordinary physics is the hovering state's, not the state it
was killed in. With the switch on nothing happens to it, as claim 18 says.
This was reachable before r30 too (the one empty assignment of r24 ran the
same stop).

## Questions

1. With the no-cell switch off, when the crash start's search finds no cell
   for a killed unit with a movement commitment, does the unit keep its
   flight state, movement point and commitment (as with the switch on), or
   does the switch-off outcome change them?
2. If it keeps them: is that because no no-cell outcome of either setting
   acts on a unit without health?

## Possible observation

None in the original, which has only the switch-on behaviour; this is a
question about the switch's design, for a unit killed in the air with every
cell around it refused (scenario J11's fully blocked setup, with a crashable
type, killed while moving).

## Current workaround

As built: the switch-off outcome applies to the killed unit. The switch is
on by default, which follows claim 18.
