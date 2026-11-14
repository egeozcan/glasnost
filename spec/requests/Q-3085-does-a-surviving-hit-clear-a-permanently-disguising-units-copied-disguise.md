---
id: Q-3085
title: Does a surviving hit clear a permanently disguising unit's copied disguise
status: open
related: [S-0302, S-0516]
answered-by: []
---

## Need

GitHub issue #169. S-0302 and S-0516 give different results for one hit, and the engine can follow only one of them. The hit rule is `disguise::hit_taken` in `crates/deterministic/sim/src/cloak/disguise.rs`, called from the hit rules of `combat/damage.rs`.

S-0302 claim 12 (the reactions to a hit's result) says:

> This claim owns two of them: the damage effect of claim 13, and, for a target whose type can disguise itself but does not keep its disguise permanently, the loss of its current disguise and a disguise-disruption countdown of 2*m*′ ticks, *m*′ the health it lost, after any hit that reports *unchanged*, *now yellow* or *now red* (S-0516 claims 33–34 own disguise loss and re-disguise eligibility).

S-0302 scenario D23 (e) expects:

> (e) A stock type that keeps its disguise permanently, hit as in (a): the disguise is kept.

In D23 the spy is "disguised as an enemy infantry type", and (a) is a surviving hit of damage 5.

S-0516's verification table (row for claims 33 and 34) says "Damage follows S-0302 claim 12."

S-0516 claim 34 (a) says:

> **(a) Damage.** The hits of S-0302 claim 12 clear it and set the re-disguise lockout, whose length is given there. This applies to infantry and vehicles alike.

S-0516 claim 32 says:

> When a permanently disguising infantry unit is made to clear its disguise (claim 34), it returns to this default instead of becoming undisguised. Vehicles do not have this reset: clearing their disguise makes them undisguised even if their type has the permanent flag.

S-0516 scenario CS13 (c) expects:

> Disguise permanent infantry as another house, then deliver an accepted surviving hit. Expected: its disguise resets to its owner's side default, rather than preserving the copied identity or becoming undisguised.

One scenario gives different results. A permanently disguising infantry unit of house A has copied an enemy infantry unit of house E (S-0516 claim 31), so its disguise type is the enemy's type and its disguise house is E. It takes a surviving hit that reports *unchanged*.

- By S-0302 claim 12, D23 (e) and S-0516's table row, the hit does not touch its disguise. It keeps the copied identity, and it keeps appearing disguised to E (S-0516 claim 35).
- By S-0516 claims 32 and 34 (a) and CS13 (c), the hit clears it, and it returns to its side's default with disguise house A. It no longer appears disguised to anyone.

The code follows S-0302 and returns early for every type with the permanent flag. Nothing else in #169 is free of this conflict, so the issue is left open with no code change.

## Questions

1. A permanently disguising infantry unit holds a copied disguise of another house and takes a surviving hit that reports *unchanged*, *now yellow* or *now red*. Afterwards, does it hold the copied disguise, or its side's default disguise with its owner as disguise house?
2. If the hit resets it, does the hit also start the disguise-disruption countdown of 2*m*′ ticks on that unit?
3. If the hit resets it, does the same hit on a permanently disguising infantry unit that already holds its side default change anything, the countdown included?
4. A vehicle whose type can disguise and has the permanent flag holds a disguise and takes such a hit. Is it left disguised, or does it become undisguised (S-0516 claim 32's vehicle sentence)?

## Possible observation

S-0516 CS13 (c) and S-0302 D23 (e) already describe the run. Use a stock permanently disguising infantry type, make it copy an enemy infantry unit with a disguise-making warhead, and hit it for 5 with a plain warhead. Record its drawn identity to the enemy house and whether that house's units without disguise detection acquire it. Repeat with a permanently disguising vehicle type in a test file.

## Current workaround

No change. A hit leaves the disguise of a type with the permanent flag as it is (S-0302 claim 12 and D23 (e)). S-0516 CS13 (c) has no test.
