# S-0607 r18: claim ledger, M8 step A3 intake 49

This intake takes in **S-0607 revision 18** (magnetic lift; spec commits a482e7b and 4cf1224).
Our engine built revision 14 ([intake 45](intake45-S-0607.md)). Revisions 15 to 17 were not
published to this repository; the diff from r14 is read as one change. r18 answers **Q-2113**
(step A6, [a6-arena-triage.md](a6-arena-triage.md), row 4: TELE against BFRT). No new quirk,
import row, formula or state comes with it.

## What r18 changes

- **Claim 11, no approach against the gripped unit.** A mobile lifter takes no approach step
  against the unit it grips while that unit is its target, in every behaviour. S-0215 claim 37
  part h's crushing and balloon-hover approaches come first and are not excepted: they give the
  lifter its gripped unit as its destination, which ends the grip (claim 12). Otherwise the range
  test, minimum range included, gives the lifter no destination towards or away from its gripped
  unit.
- **Claim 11, the per-tick drop still applies.** S-0215 claim 37 part a's per-tick rule runs
  unchanged, with its own range test after the (empty) approach step: a lifter with no
  destination that cannot reach its gripped unit's cell, with the unit out of range (minimum range
  included), drops it as its target. The grip stays (claim 12 has no row for a target change) and
  the wave ends (claim 21).
- **Claim 11, the hunter.** A computer-controlled lifter in *hunt* whose gripped unit stays
  reachable or in range takes no approach step against it, makes no fire event against it and
  keeps the grip until the unit arrives (claim 8) or another event of claim 12.
- **Claim 12, row 2** gains the same exception in its wording.
- **Claim 11's backing and confidence**, **scenario L9 part f** (records reachability and range
  on every tick; a drop on a tick with the cell unreachable and the unit out of range voids the
  part), **new scenario L17**, the **Open questions** (what ends each stock hold; whether a
  set-down cell can hold a tree or be rock) and **Changes for other specs** (S-0215 claim 37
  parts a and c) follow these.

## Built

- `approach::step`: after part h's two approaches, a lifter whose target is its gripped unit
  (`lift::gripped_unit`) returns: no keep-or-drop (part d), no destination kept or cleared (part
  e), no search (part f). Only infantry units and vehicles take the step, so the lifter is mobile.
- `approach::part_h_accepted`: when part h gives an actor its target as destination, the actor
  accepts it through `lift::lifter_accepts` (claim 12 row 2), so a lifter's grip ends at once in
  the approach step, its target cleared, and claim 13 drops or sets down the unit. Without a grip
  this does nothing.
- `approach::per_tick` is unchanged; its drop now follows the empty step for a lifter.
- No new state: the save format stays 86 and the AI game format 48.

## Design choices

| question | our choice |
|---|---|
| A part h approach gives a lifter the same destination it held when it gripped (for example an omni-crusher lifter whose crushing approach gave it its target while its seizing shot flew). Our end-of-turn test `lift::lifter_destination` ignores a destination equal to the one held at the grip (`grip_dest`) | The grip ends: claim 11 says such an approach "gives the lifter its gripped unit as its destination, which ends the grip", with no exception for a destination already held. `part_h_accepted` ends it in the approach step, ahead of the end-of-turn test. Test `claim_11_part_h_omni_crusher_ends_the_grip_at_once` |
| When in the turn a part h approach ends the grip | In the approach step itself (the behaviour processing), not at the end-of-turn test later in the same turn: the destination is accepted when it is given. The two differ only when something later in that turn ends the grip another way (for example the radiation hit removes the lifter, whose leaving play drops the unit without clearing a target) |

## Claims

| claim | status | tests or reason |
|---|---|---|
| claim 11, no approach against the gripped unit in *hunt*, computer-controlled, the unit inside the weapon's minimum range; no destination, the target kept, no fire event, the grip lasts to arrival (claim 8) | tested | `claim_11_no_approach_against_the_gripped_unit_in_hunt` (`claims_s0607_r18.rs`) |
| claim 11, in every behaviour (*attack*, *area guard*) | tested | `claim_11_no_approach_against_the_gripped_unit_in_attack_and_area_guard` |
| claim 11, a human-controlled lifter | tested | `claim_11_no_approach_for_a_human_controlled_lifter` |
| claim 11 and claim 12 row 2, the part h exception: S-0210 claim 11's attack-target crushing variant (computer-controlled) once the gripped unit comes within the crush range; the grip ends with a drop, the target cleared; control without a crushable unit lasts to arrival | tested | `claim_11_part_h_crushing_approach_ends_the_grip` |
| claim 11 and claim 12 row 2, part h's omni-crusher variant, also with the destination held at the grip | tested | `claim_11_part_h_omni_crusher_ends_the_grip_at_once` |
| claim 11 and claim 12 row 2, part h's balloon-hover approach (S-0307 claim 18) | tested | `claim_11_part_h_balloon_hover_approach_ends_the_grip` |
| claim 11, the per-tick drop of S-0215 claim 37 part a: no destination, the gripped unit's cell unreachable (water) and the unit inside the minimum range: the target is dropped, the grip stays (claim 12), the wave ends (claim 21); control over land keeps the target | tested | `claim_11_per_tick_drop_keeps_the_grip_and_ends_the_wave` |
| claim 11, the *rearming* answer and its precedence | unchanged | built at r11; its tests pass |
| claim 12, row 2's wording | tested | as the part h rows above; the comment at `lift::lifter_destination` cites the new wording |
| scenario L17 | not measured | M-0015 records neither destinations nor what ended a hold; the arena's traces below are ours |
| Q-2113 | answered | by claim 11 above; A6's request row is retired |

## Left to the spec side

- S-0215 claim 37 part c still names only the object that holds the actor, and part a does not
  yet say that its per-tick drop runs for a lifter whose gripped unit is its target. S-0607's
  Changes for other specs names the replacement texts. We follow S-0607.

## Requests

None filed. Q-2126 to Q-2130 are unused.

## Arena

Local lane, `local-lane/m8/a3-49/{before,after}/` (never committed), scored as intake 47 and A6
score them. *Before* is main 82bbb9c.

| record | before | after |
|---|---|---|
| M-0001 | 11 of 84 (3, 10); 139 of 152 | 10 of 84 (3, 9); 140 of 152 |
| M-0002 | 2 of 24 (0, 2); 45 of 47 | 2 of 24 (0, 2); 45 of 47 |
| M-0003 | 4 of 24 (0, 4); 38 of 42 | 4 of 24 (0, 4); 38 of 42 |
| all | 17 of 132 | 16 of 132 |

(Scored pairings failing, with those failing the win test and the side outcome; side-wise
unanimous stock outcomes reproduced. Not scored, before and after alike: M-0001 1 of 12 would
fail, 17 of 19 reproduced; M-0003 0 of 11, 22 of 22.)

- Only TELE games changed: 42 of M-0001's 1152 (TELE against APOC 12, BFRT 11, V3 7, SREF 6,
  TTNK 6); no game of M-0002 or M-0003.
- **TELE against BFRT now passes.** Side A ends `none` on all six seeds, as the stock's do. Over
  seeds 0 to 63 (`--seed-range TELE,BFRT,A,0-63`): `none` 60 of 64, *p*⁶ = 0.68 (before 16 of
  64, *p*⁶ < 0.001). A6's row 4 is resolved by r18.
- **TELE against APOC still fails its side-A outcome**, now by one seed (ours col 5 of 6, one
  `none`). Over 64 seeds: col 50 of 64, *p*⁶ = 0.23 (before 45 of 64, 0.12): within noise, class
  (d) as before.
- TELE against V3 (not scored) went from 1/0/11 to 0/6/6 (row/col/none); the stock's is col 12.

## M-0015 against ours (TELE against BFRT, side A, seeds 0 and 1)

The records' values that are not local, against `arena --trace-summary` (seed 0; seed 1):

| record fact | stock | before | after |
|---|---|---|---|
| fire events that started a hold | 56; 52 | 35; 25 | 58; 35 |
| fire events at a fortress held by another Magnetron | 501; 439 | 130; 163 | 462; 446 |
| other fire events | 22; 20 | 52; 61 | 171; 153 |
| hits on Magnetrons by fortress fire | 15; 18 | 18; 17 | 9; 1 |
| end of game | tick limit (6080), units left on both sides | TELE wins at 3344 in both | tick limit (6080), units left on both sides, in both |
| the landing crush: two fortresses held by two Magnetrons, released one tick apart, land in the same tick and both leave play, the later one's holder the attacker of both | in both seeds | no | in both seeds |
| other fortresses that leave play | none | 3; 3 | none; 1 (it lands on a Magnetron) |

What ended each hold, from the traces (Q-2113 question 3, scenario L17's release event):

| | before | after |
|---|---|---|
| released holds | 35; 25 | 56; 34 (2; 1 still running at tick 6080) |
| the holder took a destination at the release (claim 12 row 2) | 35; 24 | 0; 0 |
| the fortress was on its destination cell at the release (claim 8 arrival) | 0; 0 | 56; 34 |

In 1 and 2 of the after holds the holder's target had changed before the arrival. The landing
cells stay in the local lane.

## Seed sweeps

`GLASNOST_SWEEP=1-16`, release. M6c and M7b: every counter's minimum and median unchanged. M7a
(its duel has lifters): all pass; the minimums that moved went 17 to 21 (berserk weapon orders),
14 to 22 (units made berserk), 2 to 6 (psychic reveal orders and reveals fired), 12 to 13
(slave-miner buildings), 4117 to 7123 (slaves at work), 14 to 6 (gattling stage rises), 3631 to
516 (gattling at its top stage), 214 to 31 (gas clouds), 614 to 144 (gas hits), 4 to 5 and 3 to
4 (bunker entry orders, vehicles bunkered), 6 to 7 (seized units dropped); vehicles seized has
median 23 to 38.
