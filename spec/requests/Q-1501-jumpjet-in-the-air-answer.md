---
id: Q-1501
title: The in-the-air answer of a jumpjet for weapon choice and targeting
status: open
related: [S-0308, S-0502, S-0307, S-0309]
answered-by: []
---

## Need
S-0308 Terms (*Airborne*) give the aircraft test and leave "the exact test for other objects" to
the movement specs. S-0502 Terms (*Airborne*) say that whether a jumpjet counts as in the air for
combat "is given by S-0307's height test and S-0308's in-the-air state", which points back to
S-0308. Table P rows 18 and 19 (and S-0303 claim 10, S-0309) need the answer for a jumpjet.

Our engine answered "never in the air" for every non-aircraft. The oracle arena showed the
effect: in M-0001, DISK against HTK and DISK against APOC (stock: the column wins 12 of 12)
gave our engine a row win in 12 of 12 games. The HTK and APOC kept choosing their primary slot,
which the firing check rejects against the high DISK, and never fired. Code:
`crates/deterministic/sim/src/combat/weapon.rs`, `aircraft_in_air`.

## Questions
1. Is a jumpjet (a unit of a jumpjet type moving with its jumpjet locomotor) in the air for
   weapon choice exactly when it is at least 208 leptons above the surface (S-0307's test), or by
   another test (its flight state, any height above 0)?
2. Is a jumpjet infantry unit treated the same way as a jumpjet vehicle?
3. Is a unit lifted by a magnetic beam (S-0607) in the air by the same test?

## Possible observation
A firer whose primary cannot hit air and whose secondary is anti-air, targeting a jumpjet
vehicle hovering at heights just below and just above 208 leptons, and on the ground; record
which weapon it fires.

## Current workaround
A unit of a jumpjet type with jumpjet state is in the air exactly when S-0307's height test
holds (at least 208 leptons above the surface), reading S-0502 Terms. A unit lifted by a
magnetic beam keeps the old answer (not in the air).
