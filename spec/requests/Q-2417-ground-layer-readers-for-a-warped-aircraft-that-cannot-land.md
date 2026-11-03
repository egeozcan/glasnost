---
id: Q-2417
title: Target search and seizing of a warped aircraft that cannot land, flying in the ground layer
status: open
related: [S-0306, S-0102, S-0309, S-0607, S-0501]
answered-by: []
---

## Need

S-0306 claim 17 (revision 65) says that after its arrival hold a warp member whose type is an
aircraft that cannot land "keeps the ground layer while it flies, until a crash". Two rules read
that layer for it in our engine, as S-0306 and S-0514 name them: S-0306 claim 20 (c)'s
proximity threshold and S-0514 claim 1's erasure test (`superweapons::warp::in_ground_layer`).

Three other rules also speak of the ground render layer, and so far read only the object's
height (`temporal::ground_layer`). The first also reads the factory-release filing, which
S-0501 claim 48's Changes named for S-0309 claim 7 alone:

- S-0309 claim 7's second pass, which takes the objects of the ground render layer
  (`combat::acquire`);
- S-0309 claim 9's search for flying objects, which takes only objects not in the ground render
  layer (`combat::acquire`);
- S-0607 claim 3's seizing, which takes only a vehicle or aircraft in the ground render layer
  (`lift::eligible`).

No Changes entry of S-0306 r65 names them. If they read the kept layer, such an aircraft in flight is
found by claim 7's ground pass and missed by claim 9's search, and a magnetic beam can seize it.
If they read its height, the opposite holds. This is the same question that S-0501 claim 48
answered for a factory-released aircraft above its pad, but only for S-0309 claim 7.

## Questions

1. After its arrival hold, while a warped aircraft whose type cannot land flies, does a unit
   that looks for targets with S-0309 claim 7 find it in that claim's second pass, among the
   ground-layer objects?
2. Does S-0309 claim 9's search for flying objects find it, or does it skip it as an object of
   the ground layer?
3. Can a locomotor-seizing weapon (S-0607 claim 3) seize it while it flies?

## Possible observation

A test aircraft type that cannot land, put on the ground in a warp's source block (or landed by
a test map) and warped. After its hold, order it to fly. Then record which of an anti-air unit's
and a ground-only unit's target searches find it, and whether a magnetic beam unit's shot seizes
it.

## Current workaround

The three rules read the aircraft's height as before (`temporal::ground_layer`). Only
S-0306 claim 20 (c) and S-0514 claim 1 read the kept layer. Named as a seam in
`docs/m8-claims/intake84.md`; no quirk switch.
