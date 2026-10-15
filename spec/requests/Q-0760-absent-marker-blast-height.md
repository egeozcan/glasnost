---
id: Q-0760
title: Height of a demolition blast or light flash at an absent waypoint marker
status: answered
related: [S-0524, S-0011]
answered-by: [S-0524]
---

## Need
S-0524 claim 5 says an absent waypoint marker stands for cell (0, 0) (S-0011 claim 33), but the height of the damage point there is not established. Codes 63 (demolition blast) and 66 (large light flash) at such a marker are given no effect in `crates/deterministic/sim/src/mp_triggers.rs` (`number_waypoint`), as the spec's safe departure allows. If the game's behaviour is stable, we would prefer to match it.

## Questions
1. When a code-63 action names a waypoint number that the map does not define, does any damage happen at or near cell (0, 0)? If so, at what height relative to that cell's terrain surface: the surface, height 0, or something else?
2. Is the result the same on maps whose cell (0, 0) is legal and on maps where it lies outside the legal area?
3. Does a code-66 flash appear at all for an undefined marker, and if so where?

## Possible observation
On a small test map, give cell (0, 0) (or the nearest legal cell) a raised height level and stand test infantry there and on the flat ground beside it. Fire code 63 with a waypoint number that has no marker, then record health changes. Repeat on a map with flat ground at that corner.

## Current workaround
No effect and a warning note for both codes (S-0524 claim 5's safe departure).
