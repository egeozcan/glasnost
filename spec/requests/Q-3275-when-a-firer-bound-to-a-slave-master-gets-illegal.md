---
id: Q-3275
title: When a firer bound to a slave master gets illegal
status: open
related: [S-0301, S-0402, S-0309, S-0513, S-0607]
answered-by: []
---

## Need

An issue report (#450) says that in a native game on an imported install, the player's own slaves sometimes attack their own slave miner during normal harvesting, without any order.

S-0402 claim 21 says that on successful emergence a slave's target is its master. Our infantry firing step fires at any held target for which the firing check (S-0301 claim 9) answers *can fire*. Nothing in our check refuses a slave's own master. The answer that would refuse it is the *illegal* of "a firer bound to a slave master", and no spec defines it. S-0301 claim 9 names it only in its rank-4 precedence paragraphs:

- *Grip refusals within rank 4*, line 555;
- *Deactivated firers within rank 4*, line 568;
- *A materialising firer within rank 4*, line 575;
- the warp-pending paragraph, line 631;
- its open notes, lines 795 and 2772 to 2776.

S-0309 (line 5016 and lines 5035 to 5037), S-0513 (line 3938) and S-0607 claim 11 (line 613) repeat those precedence statements. S-0301's rank-4 row in the rank table does not list this reason, and no claim says what makes a firer "bound to a slave master" or which targets and weapons the refusal covers. Our check therefore leaves it out (`combat/check.rs`, the comments "not in our engine").

On synthetic data we see the result whenever the slave type has a weapon. A 3 × 3 deployed master has three slaves, armed with an invented weapon of range 3 cells against which every armour takes full damage. In 1500 ticks the slaves take the master from full strength to 10, one shot at a time. The first hit lands at tick 281, soon after the first emergence, and further hits follow throughout the harvest. On ticks with a hit, the check of a slave against its master answers *can fire*, *rearming* or *moving*, never *illegal*. With the same slave unarmed, the master's strength never changes. In our trace the slaves hold the master as their target throughout their harvesting: nothing in claim 14's search replaces it. They fire only while they stand within range and are not walking, so the damage comes and goes. That fits the report's "sometimes".

The acquisition paths that the report also suspects are closed on our side, and claim tests guard them:

- a harvesting slave does not search passively (S-0309 claim 2, `claims_s0309_slave_gate.rs`);
- it does not retaliate, against an enemy or against its own house (S-0309 claim 18, same file);
- a freed slave's target is cleared (S-0312 claims 30 and 31);
- the emergence target is the master (S-0402 claim 21, `claims_s0402_slaves.rs`).

## Questions

1. Which firers are "bound to a slave master"? A slave whose master exists, whether the slave is inside or outside, harvesting, returning, recalled or waiting? Only while it is on the map? Does a slave stop being bound once it is freed (S-0312 claims 30 and 31; S-0408 claim 10a)?
2. Which targets does the *illegal* cover? Every target, so that a bound slave never fires? Only its own master? Any object of its own house, or of a house allied with it? Does a force-fire target make a difference?
3. Does it cover every weapon of the firer, whether primary, secondary or elite?
4. Where in the firing check is it asked? The precedence paragraphs place it above the materialising firer's *rearming*. Is it also above the infantry firer's first answers (the infantry healer's *illegal* and the keep-acting *illegal*)? Since all of these are *illegal*, does the place matter for anything other than the reply, such as S-0310 claim 11's handling of a healer's *illegal*?
5. Is S-0402 claim 21's emergence target, the master, still right for an armed slave? If so, does the slave keep the master as its target until its resource search replaces it, and what does it do with that target meanwhile: approach it, turn to it, or nothing?
6. How does an *illegal* answer against the held master affect the slave? Does it drop the target, as an infantry healer does (S-0310 claim 11), or keep it?

## Possible observation

Use a slave type given a weapon in a test rules file, with every armour taking full damage and a range covering the emergence cell. Deploy a slave miner next to a field and record per tick each slave's target, its firing-check answer against its master and any shot it makes, along with the master's strength. Repeat:

- with the master as an undeployed vehicle whose slaves are out;
- with a second own-house unit standing within range;
- with a force-fire order on the master given to one slave.

## Current workaround

None: the engine fires whenever the check allows, as written. An armed slave holding its master as target can shoot it. We change nothing until this is answered.
