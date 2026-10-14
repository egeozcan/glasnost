---
id: Q-0233
title: What a teleporting vehicle's own locomotor holds when it gets it back after a temporary drive
status: answered
related: [S-0207, S-0417, S-0215]
answered-by: [S-0207]
---

## Need

S-0417 claim 5 gives a teleporting vehicle driving locomotion for its factory exit, and
S-0215 claim 13 row 1 gives it its own locomotor back at a fall-back to rest (confidence low).
S-0207 claim 20 (a) keeps a landing point *L* and a vehicle-present mark on *L*'s cell from the
moment a destination is taken until the jump. None of these says what the returned teleporting
locomotor holds: a landing point fixed before the temporary drive, a phasing-in time, or where
the occupancy mark is. Our engine joins them in `econ::release::restore_locomotor` and
`teleport::is_teleporter`.

## Questions

1. When a teleporting vehicle gets its own locomotor back after a temporary drive, does that
   locomotor still hold a landing point it had fixed before, and is it jumped to on the next
   movement update?
2. Is the vehicle-present mark then set on the vehicle's own cell, or on the cell of such a
   landing point?
3. Does a phasing-in state from before the temporary drive carry over?
4. While the vehicle drives temporarily, does a destination it is given move a teleporting
   mark at all, or only the driving claims of S-0209?

## Possible observation

S-0417 scenario U6 with the read-only recorder: produce a teleporting vehicle, record the
occupancy marks around the factory and the vehicle's position on each tick from the exit track
until the first jump after rest, and give it a move order before and after it reaches rest.

## Current workaround

While it drives temporarily, the vehicle is not a teleporter: it takes no landing point and does
not jump. When it gets its own locomotor back, that locomotor holds no landing point, no phasing in
and no remembered destination, and the mark is set on its own cell. Interim row in
`docs/development.md`.
