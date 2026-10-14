---
id: Q-0442
title: Slave-group states that S-0402 claim 27 does not list for a building master, and the automatic remembered cell as a guard spot
status: answered
related: [S-0402, S-0415, S-0408, S-0213]
answered-by: [S-0402]
---

## Need

S-0402 claim 27 gives the busy-state table of a slave-master building's
group update: at rest, working, checking the field and packing. Claim 22
says a travelling group whose master has become a building is put at rest.
A group can also be in field seeking or in the deployment retry state when
its vehicle deploys (for example when a deploy order is given while the
group seeks a field, or when a retry succeeds through the player's own
deploy order). Separately, an automatic pack-up remembers the building's
top-left cell (claim 25) and claim 28 speaks of the building's "remembered
destination"; S-0415 claim 9 and S-0408 claim 17 also read a guard spot. The
code is `econ::slaves::building_update` and
`econ::slaves::remembered_destination`.

## Questions

1. When a slave-miner vehicle becomes a building while its group is in
   field seeking or in the deployment retry state, what does the building's
   next group update do: put the group at rest (as for travelling), go to
   checking the field, or something else?
2. Is the cell remembered by an automatic pack-up the same thing as the
   building's guard spot for S-0415 claim 9 (whether the building packs up
   rather than being sold) and for S-0408 claim 17? For example: a slave
   miner whose automatic pack-up has been requested is then sold by the
   player before the build-down ends; does it pack up toward its old cell?
3. For claim 28, when the building has both a guard spot from a move or
   harvest order and an automatic remembered cell, which one is the
   remembered destination?

## Possible observation

A purpose-made map with a slave miner given a deploy order at the moment
its group starts seeking a field (field far away), then watched for one
group-update period after the deployment. A second scenario: an automatic
pack-up requested on a field that runs out, then a sell order given during
the build-down.

## Current workaround

1: field seeking and retrying are treated like travelling: the building's
next group update puts the group at rest. 2 and 3: the automatic remembered
cell is kept apart from the guard spot; claim 28's remembered destination
is the automatic cell when there is one, else the guard spot; S-0415 claim
9 and S-0408 claim 17 read only the guard spot. Listed in
docs/development.md (interim decisions).
