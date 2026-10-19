---
id: Q-1505
title: Is a vehicle docked on a repair depot passed over when the last control centre stops
status: answered
related: [S-0405, S-0314]
answered-by: [S-0405]
---

## Need
S-0405 claim 13 (revision 20) passes over, at the 1-to-0 transition, a vehicle that is
tethered to the building standing in its own cell. Only the factory case is established. Its
Open questions leave open whether a vehicle docked on a repair depot is tethered to it at that
moment, and so passed over. Our engine decides this in
`crates/deterministic/sim/src/econ/power.rs` (`set_disabled`, `tethered_in_own_cell`).

## Questions
1. A powered-unit vehicle stands docked on its own house's repair depot, being repaired, when
   the house's last control centre stops. Is it disabled at that moment, or does it stay
   enabled?
2. Does the answer differ between a depot whose type has the pad flag and an ordinary repair
   depot?
3. If it stays enabled, is it disabled on its first arrival in a cell without a building after
   it leaves the depot?

## Possible observation
A house with one control centre, one repair depot and one damaged powered-unit vehicle. Order
the vehicle onto the depot. While it is being repaired, switch the centre off (or sell it).
Record whether the vehicle is disabled at once, and what happens when it drives off the depot.
Repeat with a depot type that has the pad flag.

## Current workaround
The literal reading of claim 13 with our depot model: a vehicle counts as tethered when our
depot exchange has tethered it (S-0314 claim 17: a pad depot tethers, an ordinary depot does
not). Such a vehicle is passed over. A vehicle on an ordinary depot is disabled, since our
model does not tether it.
