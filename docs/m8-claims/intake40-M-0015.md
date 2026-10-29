# M-0015: claim ledger, M8 step A3 intake 40

This intake takes in one measurement record, **M-0015**. It watches the stock M-0001 games of
TELE against BFRT unit by unit (side A, seeds 0 and 1; measurement commit ff1d17d), and it answers
part of Q-1712 question 2. We compared our games of that pairing with it fact by fact, using the
`arena --trace` tool of intake 38 extended with the magnetic lift (below). No spec revision comes
with the record. The specs it touches were read as they stand:

- S-0607 r11: claims 3, 5, 6, 9 to 13, 16 to 18 and 20, and scenario L11;
- S-0215 r76: claims 13, 19, 27 and 33;
- S-0309: table C, the *firing check* row;
- S-0207 r82: claim 16;
- S-0210.

**Result: no departure from an approved claim was found, so no simulation code changed.** Every
difference has one cause. The specs leave open the rule that would remove it, so we filed a
request for that rule (Q-2035).

## Fact by fact

Ours means the oracle arena as built (`glasnost-headless arena --trace`, local lane). It reads the
state after each tick, as the record's tick convention does. The record's numbers come from
M-0015. Every row below holds for all twelve of our games (sides A and B, seeds 0 to 5), not
only for the two games the record watched.

| M-0015 fact | stock (seed 0; seed 1) | ours | verdict |
|---|---|---|---|
| every fortress held from the first tick, by fire events in that tick | yes | yes, at tick 1 (the same labelling offset as intake 38's tick of the first gas) | agrees |
| fire events that started a hold | 56; 52 | one per fortress in each game | **differs**: cause below, Q-2035 |
| fire events at a fortress held by another Magnetron, no hold | 501; 439 | 12 (side A), 10 (side B) | follows from the cause |
| other fire events, no hold | 22; 20 | about 3 000 per game; every one meets a fortress that is still seized | follows from the cause; the record leaves its 22 and 20 unexplained |
| no fire event by a Magnetron while it holds a fortress | none | none | agrees (S-0607 claim 11) |
| damage events with a Magnetron as attacker | only the 2 of the landing crush | none | follows from the cause (no landing crush) |
| holds that end in a landing | local | every hold | compared in the local lane only; follows from the cause |
| height above the surface at release | above 0 in every one | above 0 in every one of every game | agrees that every release is a drop (claim 13) |
| carry time, release height and time to landing | local | local lane only | not compared: the record keeps the values local, and this session has none to compare with |
| damage events at a landing other than the landing crush | 0 | 0 | agrees (claim 17, the *not blocked* row: unhurt) |
| what ended each hold | not recorded | in ours the lifter accepted a destination (claim 12, row 2) | left alone: the record does not identify it |
| landing crush: two fortresses come down together; the crushing warhead kills both, attacker the later one's holder; 3 credits for 2 fortresses | 1 per seed | not reached in our games | tested synthetically (claims 16 to 18, double credit on): agrees |
| fortress hits on Magnetrons | 15; 18 | 0 | **differs**: cause below |
| Magnetrons destroyed by that fire | 0; 0 | 0 | agrees, trivially |
| Magnetrons that leave play, all with no damage event while a fortress that targeted them stood on their position (crushed, S-0210 claim 4) | local | none | **differs** (compared in the local lane): cause below |
| end of game | tick limit (6080); the units left are local | tick limit (6080); every unit left | follows from the cause |

## Cause

At tick 1 each fortress is seized (S-0607 claim 5). Its house is a test-only computer-controlled
player house, so it queues *hunt* (claim 6), and *hunt* is current from tick 2. Each lifter later
accepts a destination, which ends its grip (claim 12, row 2). The fortress is then dropped over
clear land (claim 13), descends, and is set down unhurt (claim 17). It is still *seized*: claim 20
gives a unit its own locomotor back only when it falls back to rest. Two rules keep it from ever
doing so:

- **Its hunt search finds nothing.** Each *hunt* processing runs the normal search (S-0215
  claim 27), and that search rejects every candidate at S-0309 table C's firing-check row. A
  seized unit's firing check answers *illegal* for every weapon and target (S-0607 claim 10).
- **Hunt without a target does not fall back to rest.** It only attempts the idle action, and a
  vehicle has none (S-0107 claim 14).

We found no other rule that makes such a unit fall back to rest. A scatter request queues no
behaviour (S-0207 claim 16), and claim 9 refuses every new destination.

So in our games the fortresses stay seized from tick 1 to the frame limit. They take no target,
fire no shot and never move. Every later Magnetron hit meets a seized fortress and seizes nothing
(claims 3 and 12). The trace confirms this: `seized` never clears for any fortress in any of the
twelve games.

The record shows the fortresses held again after their landings, 56 and 52 times. They also fire
at the Magnetrons and crush them. By claims 3 and 10, all of that needs the fortresses to get their
own movement back, and no approved claim says when a set-down unit in *hunt* does. Scenario L11
part c expects a unit left alone after touchdown to stay seized, but it does not name that unit's
behaviour, and part b records only the hunt queued at the seize.

That rule is open, so we built no interim rule. Any rule we picked would be fitted to M-0015
alone: **Q-2035**. Once it is answered, a restored fortress is an ordinary computer-house hunter
under the built rules: search, fire, and the attack-target crushing approach of S-0210 claim 11
(intake 38's test `m0011_a_berserk_crusher_holding_a_berserk_friendly_target_drives_over_it` shows
that approach crushing). Q-2005 may bear on the crushes too.

## Claims

| claim | status | tests or reason |
|---|---|---|
| S-0607 claims 6, 10, 17 and 20, S-0215 claim 27, S-0309 table C *firing check* (a seized unit of a computer-controlled house, set down unhurt with *hunt* current, stays seized, takes no target and fires no shot; the same unit, never seized, fires) | tested | `m0015_a_set_down_computer_unit_in_hunt_stays_seized_and_never_fires` (`claims_s0607_m0015.rs`) |
| S-0607 claims 16 to 18 and `quirks.lift_impact_double_credit` (a unit dropped onto a grounded vehicle of its own house crashes; the first detonation, crushing warhead, its dropper as attacker, kills both; 3 credits for 2 units) | tested | `m0015_landing_on_a_grounded_vehicle_of_its_own_house_kills_both_three_credits`. The test does not stage the record's exact case, two fortresses released one tick apart that come down together; it sets the grounded one there beforehand |
| S-0607 claim 20 for a set-down unit in *hunt* (when it gets its own movement back) | request | Q-2035; the built rules meanwhile |

## Tool

`glasnost-headless arena --record M-0001 --trace TELE,BFRT,A,0` (or `--trace-summary`) now also
prints:

- each unit's lifter, and its *seized*, *set down* and *dropped* changes;
- a `lift:` summary that sorts the lifters' fire events as M-0015 does (started a hold, met a unit
  held by another, else; with how many of the last met a seized unit), and counts those made while
  holding;
- every hold, with its release and landing ticks, the height above the surface at release, and
  the hits at landing;
- the ticks at which each unit was restored;
- the hits between the two groups.

It is tooling only (`crates/tools/src/oracle_trace.rs`). The trace files stay in the local lane
(`local-lane/m8/a3-40/`), because they hold heights and times that come from the stock rules.

## Arena and sweeps

No simulation code changed. See the report of this intake for the arena table and the sweeps.
