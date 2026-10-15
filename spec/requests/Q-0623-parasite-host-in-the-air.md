---
id: Q-0623
title: What counts as a host "in the air" in the parasite host-cell test?
status: open
related: [S-0614, S-0307, S-0501]
answered-by: []
---

## Need

S-0614 claim 13's host-cell test requires that "the host is not in the air"; claim 15 reuses the
test for host loss. No spec defines that state for this test. S-0307 Terms defines *airborne*
as a height of at least 208 leptons above the surface, and our engine also has a separate
in-flight state for aircraft (S-0501). Our engine needs one test in
`glasnost_sim::parasite::host_cell_ok`.

## Questions

1. Is a host "in the air" for this test when it is airborne by S-0307's height rule, when it is
   a flying aircraft or a jumpjet in flight, or something else?
2. Does a host on a high bridge deck (416 leptons above the ground) count as in the air?
3. Does a host that is jumping, falling or being carried count?

## Possible observation

A drone-type parasite on a jumpjet infantry or hover host released by a sonic hit at several
heights (on the ground, on a bridge deck, rising, at cruise height); record whether it comes
out or is removed silently.

## Current workaround

An interim rule, noted in docs/development.md: the host is in the air when S-0307's airborne
rule holds (height above its surface at least 208 leptons) or it is in flight as an aircraft.
