# M7b step 1e claim ledger (depot disabling and the crashing mutation victim)

This is M7b step 1e. It takes in the two revisions that step 1d listed as not taken in: S-0405 revision 21 (aba2af6) and S-0107 revision 18 (98e3b55). They answer Q-1505 and confirm S-0611 revision 2's crash rule from the death-sequence side. `git log c059ee2..main -- spec/ measurements/` was empty at the start and at the end of this step, so nothing else is new. Every changed claim gets one class, as in [step1b-m7b.md](step1b-m7b.md):
- *changed*: code changed, with the test;
- *satisfied*: already true, with the test that shows it (a new test where there was none);
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

**Format.** No code changed behaviour, so the save format stays 65, the AI game format 26, and the goldens do not move.

| Spec | Revision | Answers |
|---|---|---|
| S-0405 power | 21 (from 20) | Q-1505 |
| S-0107 infantry sprite sequences | 18 (from 17) | none (S-0611 claim 7's crash, seen from claim 17 row 7) |

**Retired interims** (in code comments, `docs/development.md` and the ledgers):
- Q-1505 (a vehicle on a repair depot at the last control centre's loss). Replaced by S-0405 claim 13's *Repair depots* paragraph. The interim said "passed over when our depot exchange tethers it (a pad depot does, an ordinary depot does not)". The answer reads the tether only: a pad depot can still serve an untethered vehicle, which is disabled. `power::tethered_in_own_cell` already tested exactly that (the tether to the contact building, and that building's foundation holding the vehicle's cell), so no code changed. The comment in `econ::power::set_disabled`, the S-0405 paragraph in `docs/development.md` and row 13 of `docs/m5-claims/S-0405.md` now cite revision 21.

**New requests:** none. Q-1505's request file is already marked answered by S-0405 (aba2af6).

## S-0405 revision 21 (from 20)

| Claim | Class | Notes |
|---|---|---|
| 13, *Repair depots* (Q-1505): passed over at the 1-to-0 transition only if tethered to that depot standing in the vehicle's cell; being serviced or in contact is not enough; the test reads neither the pad flag nor the repair phase | satisfied | `power::tethered_in_own_cell` (`depot::tethered` holds only for a tether to the contact depot, `depot::link`). New file `claims_s0405_depot.rs`, scenario W8 part (m) on our own types, every state reached through the depot's exchanges (click, entry request, repair), never by writing the tether: `claim_13_ordinary_depot_customer_is_disabled` (an ordinary depot: serviced, untethered, disabled), `claim_13_untethered_pad_customer_is_disabled` (a one-place pad depot reached by a straight approach: serviced, untethered, disabled), `claim_13_tethered_pad_customer_is_passed_over` (a two-place pad depot whose place 0 is held far away, so the repeated request tethers the drone in the centre cell: passed over). |
| 13, no new disabling event when repair finishes or contact or tether end; a passed-over vehicle leaving a depot is disabled at its first completed arrival in a cell without a building | satisfied | `power::arrival` (built at step 1c). Test: `claim_13_tethered_pad_customer_is_passed_over` (repaired, sent away, still enabled on the depot; disabled only once off the foundation). |
| 13, the arrival exception needs no tether | satisfied | `power::arrival` tests the building in the cell, not a tether (step 1c, `claim_13_cell_entry_rule`). |
| Import mapping, `UnitRepair` and `Helipad` row | satisfied | Both already imported for S-0314 (`unit_repair`, `helipad`); claim 13 reads neither. |
| Scenario W8 part (m), backing, front matter | (c) | |

## S-0107 revision 18 (from 17)

| Claim | Class | Notes |
|---|---|---|
| 17, row 7: for a successful replacement a crash changes only the victim's removal; the conversion animation still starts in the death tick; the standing position is marked taken again at crash entry; the animation's later freeing need not wait for the crash to end | satisfied | Built at step 1d for S-0611 revision 2 (`mutation::convert`, `mutation::mark_now`). Tests: `claim_7_crash_victim_still_converts` and the new `s0107_claim_17_row_7_crash_changes_only_the_removal` (`claims_s0611.rs`: the animation ends after the same span from the death tick as on the ground, while the victim is still crashing, and makes its unit). |
| Front matter | (c) | |
