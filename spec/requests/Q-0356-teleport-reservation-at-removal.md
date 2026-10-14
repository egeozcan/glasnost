---
id: Q-0356
title: Which teleport reservations a vehicle, or an aircraft with a pending destination, clears when it is removed
status: answered
related: [S-0207, S-0209]
answered-by: [S-0207, S-0209]
---

## Need

S-0207 claim 20 a and e (with S-0209 claim 1, *Teleporting vehicles* and
*Teleporting aircraft*) give the reservation lifecycle of a teleporting vehicle
and aircraft under `quirks.teleport_vehicle_ground_reservation` and
`quirks.teleport_aircraft_ground_reservation`: a ground reservation at the
remembered landing point, kept after a completed jump, and a second mark in the
deck record after a deck landing.

For an aircraft, the specs say what removal clears when it stands on the deck
with no replacement destination: the deck flag only. They do not say:

- what a **vehicle** clears when it is destroyed or otherwise taken off the map
  (a) while it stands on a deck after a teleport landing, with both its ground
  reservation and its deck mark set, (b) after a landing off a bridge, and
  (c) while a landing point is fixed but not yet jumped to;
- what an **aircraft** standing on the deck clears when it is removed after a
  replacement destination has reserved a new landing point's ground record but
  before the jump;
- what a teleporting vehicle that stands on a deck after a teleport landing
  clears when it is handed driving locomotion for a factory exit (S-0417 claim
  5), and which record its own cell mark is in when it gets its own locomotor
  back (S-0207 claim 21 says only that a retained reservation is not moved).

The code is `teleport::removal_marks`, used by `objects::remove` and
`drive::release_claims`.

## Questions

1. A teleporting vehicle lands on an empty high-bridge cell (both records
   now have the vehicle-present flag) and is destroyed there. Afterwards, is
   the vehicle-present flag set in the ground record under the bridge, in the
   deck record, in both, or in neither?
2. The same vehicle, destroyed on the deck after it has been given a new
   destination elsewhere and has reserved it but not yet jumped: which of the
   old deck record, the old ground record and the new landing cell's ground
   record keep the flag?
3. For a teleporting aircraft in the situation of question 2, which of the old
   deck flag and the new landing cell's ground object-present flag remain after
   it is destroyed?
4. A teleporting vehicle that landed on a deck leaves a factory-exit track with
   driving locomotion and later gets its teleporting locomotor back. Are the
   ground and deck records of its former landing cell still flagged afterwards?

## Possible observation

The S-0207 scenario D22 setup (a high bridge over level ground, a teleporting
vehicle and aircraft) with a destruction at each of the moments above, probed
afterwards with a ground vehicle approaching under the bridge (moving blocker
from the ground flag, S-0209 claim 5 row VR5) and a walker taking a standing
position there (refused by an object-present flag, S-0209 claim 6).

## Current workaround

An interim rule in docs/development.md: removal clears every record the
vehicle holds (its pending or remembered reservation and its deck mark). An
aircraft on the deck with no replacement destination clears only the deck
flag, as specified. One with a replacement reservation pending clears both
the deck flag and that reservation.
