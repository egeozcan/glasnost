---
id: Q-0411
title: A deployed slave miner whose field runs out with no other field or site, and the vehicle's destination after a pack-up
status: answered
related: [S-0402, S-0415]
answered-by: [S-0402]
---

## Need

S-0402 claim 25 leaves three cases open, and `econ::slaves` (the building
group update, `building_update`, and the end of an automatic pack-up) must
do something in each:

- a deployed slave miner whose short scan finds no field and whose long scan
  finds none either;
- a deployed slave miner whose long scan finds a field but for which claim
  24's site search finds no site near it;
- the vehicle made by a successful automatic pack-up, between the pack-up and
  its first field-seeking group update: does it have a destination?

## Questions

1. A deployed slave miner's field is exhausted and no other resource cell
   lies within its long scan range. Does it stay deployed? Do its slaves
   still leave it? Does it pack up? If it packs up, where does the vehicle
   go, and what does it do once it gets there?
2. The same, but a field lies within the long scan range and every cell near
   it fails the site test of claim 24 (for example, walls on every free
   cell). Does it stay, pack up, or something else? If it packs up, where
   does the vehicle go?
3. Right after an automatic pack-up succeeds, before the group's next update,
   does the new vehicle move, and towards which cell?
4. In cases 1 and 2, if the miner stays, does it try again later, and how
   often?

## Possible observation

S-0402's scenario H19: a deployed slave miner on a small field, its slaves
working, with (a) no other field on the map, (b) one far field whose
surroundings are blocked for its foundation, (c) one far field with free
ground. Record the miner's object (building or vehicle), cell, behaviour and
destination each tick from the field's exhaustion until 1000 ticks after.

## Current workaround

Cases 1 and 2: the miner stays deployed and keeps working (no pack-up, no
retry beyond its ordinary group updates). Case 3: the new vehicle gets no
destination of its own; it keeps the remembered cell of claim 22, and its
first field-seeking update decides. No outcome is invented beyond leaving
things as they are. Listed in docs/development.md (slave miners).
