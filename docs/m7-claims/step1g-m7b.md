# Spec revisions after M7b: S-0611 r3, S-0402 r33, S-0506 r11 (creation sets)

This step takes in the five spec revisions that landed after M7b. Two of them were already taken in at M7b step 1e ([step1e-m7b.md](step1e-m7b.md)). The other three revisions describe one rule from three sides, so they share a ledger. Classes are as in [step1b-m7b.md](step1b-m7b.md): *changed*, *satisfied*, *seam*, *(c)*.

| Spec | Revision | Answers | Outcome |
|---|---|---|---|
| S-0405 power | 21 (from 20), aba2af6 | Q-1505 | already taken in at step 1e; nothing more |
| S-0107 infantry sprite sequences | 18 (from 17), 98e3b55 | none | already taken in at step 1e; nothing more |
| S-0611 mutation | 3 (from 2), fba573c | Q-1460 (question 3) | changed |
| S-0402 harvest cycle | 33 (from 32), cd325aa | none | changed |
| S-0506 missiles | 11 (from 10), bcbf1d8 | none | changed |

**Format.** The order change matters only for a type that names both a spawned type and a slave type. The new groups matter only for an infantry or aircraft type that names a slave type. No synthetic gate setup has either, so the save format, the AI game format and the goldens do not move.

**New request:** Q-1530 (does the object that closes a cycle of spawned and slave types keep an empty controller and group). Interim: it keeps neither.

## S-0611 revision 3 (from 2)

| Claim | Class | Notes |
|---|---|---|
| 14, *Objects created with the made unit* (Q-1460 question 3): the creation set, one raw draw then one number per member; the made unit first, then its slot objects in slot order, then its slaves in place order, depth first; the position-choice draw after the whole set; every member owned by the attempt's house; a refused set never enters play and nothing of it is undone | changed | `objects::create` now runs `objects::fill`, which fills the spawn slots before the slave group (the order used to be the other way round). `harvest::Rules::group_of` gives infantry and aircraft types their slave group too (before, only vehicles and buildings had one), so a made infantry type with a slave type now creates its slaves. `mutation::discard` also drops the launcher records of a refused set's slot objects. Tests: `claim_14_creation_set_draws_and_numbers`, `claim_14_position_choice_after_the_set`, `claim_14_weapon_controllers_take_nothing` (`claims_s0611.rs`). |
| 14, the weapon controllers (mind control, parasite, temporal, airstrike designator) take no draw and no number | satisfied | None of them creates an object at creation. Test: `claim_14_weapon_controllers_take_nothing`. |
| F5, the sequence and its totals | changed | Tests: `claims_creation_set.rs` (`s0506_claim_1_slot_objects_before_slaves`, `s0402_claim_19_slaves_after_nested_slot_sets`). |
| F5, a cycle of spawned and slave types: the importer warns, and the type that closes the cycle gets no initial fill inside the chain | changed | Before this change such data recursed without end. `objects::fill` carries the chain of creations in progress, and `objects::creation_cycles` fills `MissileRules::warnings`. Q-1530 interim: the closing object gets no controller and no group either. Test: `s0611_f5_cycles_end`. |
| 11 and the State table rows (the creation set of a refused attempt), Import mapping rows, MU12, Open questions | (c) | |

## S-0402 revision 33 (from 32)

| Claim | Class | Notes |
|---|---|---|
| 19, each initial slave takes a number after its own draw, after the master's and the initial spawn-slot objects' (with their descendants), in place order | changed | As S-0611 F5 above. Tests: `s0402_claim_19_slaves_after_nested_slot_sets`, `s0402_claim_19_infantry_master_gets_its_group`. Pack-up (claim 19a, `deploy::pack_up`) still creates the return vehicle's slaves and no spawn slots. That gap predates this revision and is unchanged. |
| H15 scenario text | (c) | |

## S-0506 revision 11 (from 10)

| Claim | Class | Notes |
|---|---|---|
| 1, each initial slot object takes a raw draw and a number after the spawner's, in increasing slot index; each slot object's own set comes before the next; all come before the spawner's slaves | changed | As S-0611 F5 above (the slot objects used to come after the slaves). Test: `s0506_claim_1_slot_objects_before_slaves`. |
| R1 scenario text, backing and confidence | (c) | |
