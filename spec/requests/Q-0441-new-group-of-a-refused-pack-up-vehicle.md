---
id: Q-0441
title: When the retained vehicle of a failed pack-up creates its new slave group
status: answered
related: [S-0415, S-0402, S-0312, S-0202]
answered-by: [S-0402]
---

## Need

S-0415 claim 11 (revision 8) says that when a slave-master building's
pack-up is refused, the retained off-map vehicle "keeps its own newly
created hidden group (S-0402 claim 19), not the building's old slaves", and
that with `quirks.failed_packup_keeps_vehicle` off "the refused vehicle and
its new group then do not remain". S-0402 claim 19 creates a group, with one
creation draw per slave (S-0202 claim 10), when a master is created. Claim
10 says a successful conversion keeps the old group. The order of these
creations relative to the old group's end, and whether a successful
conversion also creates a group that is then discarded, decides the draws
and the object numbering. The code is `econ::deploy::pack_up`.

## Questions

1. When a slave-miner building packs up and the vehicle is put on the map,
   is a new slave group (with its creation draws) also created for the
   vehicle and then thrown away, so that only the old slaves remain? Or is
   no new group created at all?
2. When the pack-up is refused, is the retained vehicle's new group created
   before or after the old group's on-map slaves are given away and its
   hidden slaves destroyed (S-0312 claim 31)?
3. With `quirks.failed_packup_keeps_vehicle` off, are the new group's
   slaves created (taking their creation draws) and then removed with the
   vehicle, or never created?

## Possible observation

Two runs from one save of a purpose-made map with a deployed slave miner:
one whose pack-up succeeds and one whose return cell is blocked. Compare
the random-number position (for example by the next scatter or idle choice)
and the identities of the new slaves against the old ones.

## Current workaround

A successful conversion creates no new group. On refusal with the quirk on,
the old group ends first and then the retained vehicle creates its new group
(with the creation draws); with the quirk off no new group is created.
Listed in docs/development.md (interim decisions).
