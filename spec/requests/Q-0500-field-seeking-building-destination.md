---
id: Q-0500
title: Which destination a field-seeking slave-master building is offered, and how a selling building refuses it
status: open
related: [S-0402, S-0415, S-0215, S-0213]
answered-by: []
---

## Need

S-0402 claim 27's new row for a building whose slave group is in field seeking says that a found
site is offered "as a building destination under its ordinary destination rules", that *move* is
queued without an immediate start, and that "a selling building can refuse the destination and
queued behaviour under its ordinary rules". The engine (`econ::slaves::building_update`) must
store that site somewhere. Two places are possible in our state: the object's destination (what a
vehicle master gets under claim 22) and the building's guard spot, which claim 28 now says is the
building's one remembered destination (read by S-0415 claim 9, S-0408 claim 17 and claim 28's
directed site rule). S-0215 says elsewhere that a building has no destination for the stop order
to clear. No spec states a rule by which a building refuses a destination.

## Questions

1. When a field-seeking building's site search succeeds, does the site become the building's
   guard spot (so that a later pack-up heads there, and S-0415 claim 9's guard-spot release rule
   sees it), or a separate destination that the pack-up and claim 28 do not read?
2. Under which conditions does a building refuse that destination? Is it only while *selling* is
   current, and does the refusal leave any earlier guard spot or destination unchanged?
3. Does the queued *move* on a building that accepts it have any observable effect before the
   next group update puts the group at rest?

## Possible observation

S-0402 scenario H25's field-seeking case with a site, recording the building's guard spot and
then selling it so that it packs up: the vehicle's destination shows which one was set. Repeat
while the building is already selling.

## Current workaround

The site is given as the object's destination with *move* queued, as for a vehicle master
(`slaves::move_to`). Selling refuses only the queued *move* (S-0215 claim 7); the destination is
always set. The guard spot is not changed.
