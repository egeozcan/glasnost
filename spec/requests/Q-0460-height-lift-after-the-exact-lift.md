---
id: Q-0460
title: Which height lift the sight centre uses now that S-0101's lift is exact, and stale lift references in S-0102 and S-0409
status: answered
related: [S-0101, S-0102, S-0409]
answered-by: [S-0409, S-0101]
---

## Need

S-0101 revision 8 replaced the object height lift *L(z)* of F2 with an exact
rational lift, ⌊(30·z + 104)/208⌋, as a drawing departure (claim 3), and
retired the switch `quirks.height_lift_float_scale`. Two other specs still
lean on the old lift:

- S-0409 F2 (sight centre) says "*L* is S-0101 F2", with
  *k* = trunc(*L*(*Z*) / 30). The sight centre is game state: it decides
  which cells a unit reveals, so it must be the same on every peer and
  cannot follow a drawing departure by accident. S-0409's note under F2
  also says *L*(104*h*) = 15*h* "up to level 19 (S-0101 claim 4)", which
  matches the old lift, not the new one (the exact lift gives 15*h* at
  every level).
- S-0102's Edge cases entry for `quirks.overlay_lift_exact_levels` says the
  overlay lift and the object lift "disagree by a few units" at levels
  ≥ 20 and points to "S-0101 quirk `quirks.height_lift_float_scale`",
  which S-0101 no longer has. With the exact lift both lifts are 15 per
  whole level, so the switch changes nothing at whole levels.

The code is `crates/deterministic/sim/src/econ/shroud.rs` (`height_lift`,
used by `centre`) for the sight centre, and
`crates/app/src/mapview/quirks.rs` (`overlay_depth_lift`,
`NOT_IMPLEMENTED`) for the overlay switch.

## Questions

1. Does S-0409 F2's sight centre use the game's own floating-point lift
   (the one S-0101 F2 defined before revision 8), or the exact lift that
   S-0101 F2 now defines? If the former, where is that lift defined now
   that S-0101 no longer gives it?
2. Is S-0409's "up to level 19" note still right for the lift F2 uses?
3. Should S-0102's `quirks.overlay_lift_exact_levels` entry drop the
   reference to `quirks.height_lift_float_scale`, and does the switch keep
   any visible effect (for example between whole levels, or for overlay
   shadows) now that the object lift is exact?

## Possible observation

For question 1: a unit hovering or standing at a height where the two lifts
differ in trunc(*L* / 30) (for example just below a multiple of 30 pixels of
lift, at a height of 20 levels or more) next to a shroud edge; which cells it
reveals tells the two lifts apart.

## Current workaround

The simulation keeps the floating-point lift of S-0101 F2 as it stood before
revision 8 (`glasnost_sim::econ::shroud::height_lift`, unchanged), because
changing game state on a drawing departure is not safe without an answer.
The renderer uses the exact lift everywhere. `height_lift_float_scale` is
listed in `NOT_IMPLEMENTED` (it is still named by S-0102's text) with this
request as the reason; `overlay_lift_exact_levels` is kept as a field and
gives the same lift in both settings at whole levels.
