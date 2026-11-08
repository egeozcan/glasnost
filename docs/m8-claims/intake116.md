# M8 A3 intake 116: S-0309 r126

Brief: `.claude/worktrees/M8-A3-INTAKE116-BRIEF.md`.

S-0309 r124 to r126 (commit 4163d97, from the r124 that intake 112 built) answers **Q-2801**.
Claim 31 now covers buildings at work when their owner changes:

- A building that is being sold requests its idle body, which can cut its build-down short or
  stall it.
- A building being built up has its grand opening at the change, with no free unit.
- A vehicle factory's queued *unload* is replaced, or its current *unload* ends. The unit it had
  not yet sent out stays at its exit point, and the factory's later releases fail. The door is
  left open.
- A repair depot stops repairing, and the docked vehicle stays in *guard*.
- A psychic domination resets the orders, but adds no guard reset.

One quirk is new, `owner_change_interrupts_building_work` (default on). Its C7 verdict is in
[c7-play-fixes.md](c7-play-fixes.md): not admitted. Claims 18 and 23 (d) get text only.

New request: **Q-2841**, the entry test at a ground vehicle factory's exit point. The block is
Q-2841 to Q-2850; Q-2842 to Q-2850 are unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Ledger

| Claim part | Status | Where, tests |
|---|---|---|
| 30: a psychic domination is one of the listed causes and drops the target | tested | `mind::dominate` (the target drop was already in `set_owner`); `claims_s0309_r126::a_domination_resets_the_orders_without_a_guard_reset` |
| 31: a domination resets the orders, with no guard reset | tested | `mind::dominate` calls `reset_orders` when the owner changes (the `design: Q-2801` question 7 line is removed); `a_domination_resets_the_orders_without_a_guard_reset` (the resumed destination is kept) |
| 31: a domination by the house that already owns the object resets nothing | tested | `a_domination_by_the_owner_resets_nothing` |
| 31: release, then domination: a reset at each change, the second only if the owner changes | partly | `mind::release` (unchanged) and `mind::dominate`, which read the owner after `release_one`; no test of its own |
| 31, *Guard request*: replaces a queued *construction* or a queued *unload* | tested | `construction_queued_is_replaced_by_guard`, `a_queued_unload_is_replaced_and_the_unit_is_stranded` |
| 31, *Guard request*: an object in *selling* gets none and keeps selling | tested | `claims_s0309_r124::a_building_being_sold_keeps_selling` (now cites the answer) |
| 31, *Rescue*: refused under *wait* | tested | `claims_s0309_r124::wait_with_rescue_queued_keeps_both` (now cites the answer) |
| 31, *Fall-back*: a building requests its idle body also while selling or being built up; a deferred change | tested | `structures::ownership::fall_back`; `a_building_being_sold_keeps_selling`, the sale and build-up tests below |
| 31, *Grand opening*: a building that has not had one has it, whatever the cause and the switch: ready mark, slot animations | tested | `ownership::opening`, `Construction::opened` (new, hashed), the app's `slot_play::State::completed`; `construction_queued_is_replaced_by_guard`, `the_grand_opening_acts_once_and_whatever_the_switch` |
| 31, *Grand opening*: no free unit, now or later | tested | `deploy::process_construction` returns before `grand_opening` once opened; `assert_no_free_unit` in the three build-up tests, against `control_an_ordinary_build_up_gives_the_free_unit` |
| 31, *Grand opening*: one that has had it has none again | tested | `the_grand_opening_acts_once_and_whatever_the_switch` |
| 31, *Grand opening*: other effects of the grand opening, owned by other specs | seam | S-0309 Open questions; `design:` line in `ownership::opening`; none are made |
| 31, *Destination and guard spot* and *Grand opening*: a naval vehicle factory left with no rally point gets one again | seam | S-0309 Open questions (where the cell lies is not established); S-0108 claim 11's opening gives our naval factories none yet, so none is given; `design:` line in `ownership::opening` |
| 31, *A building being sold*: the change before the build-down leaves the sale unchanged | tested | `sell::process_selling` drops the idle body at the build-down start; `a_change_before_the_builddown_leaves_the_sale_unchanged` (same removal tick, the new owner's refund) |
| 31, *A building being sold*: during the build-down the idle range replaces it; delay 0 ends the sale at the next turn | tested | `sell::process_selling` (the new `BuildDown` arm), `body::turn`; `a_zero_delay_idle_range_ends_the_sale_at_the_next_turn` |
| 31, *A building being sold*: *c* ≥ 2 stages, delay *d* > 0: the end comes (*c* − 1) × N(*d*, *s*) after the replacement | tested | `IdleBody::sale_report` (new, hashed); `a_staged_idle_range_ends_the_sale_on_its_last_stage` |
| 31, *A building being sold*: the end is the mark set by the idle range, not a mark set before | tested | `a_mark_set_before_the_replacement_does_not_end_the_sale` |
| 31, *A building being sold*: one stage with a positive delay stalls the sale | tested | `a_one_stage_idle_range_stalls_the_sale` |
| 31, *A building being sold*: the refund goes to the owner when the sale completes | tested | `a_zero_delay_idle_range_ends_the_sale_at_the_next_turn`, `a_change_before_the_builddown_leaves_the_sale_unchanged` |
| S-0215 claim 11 as claim 31 reads it: a turreted building's body range sets the mark while its behaviour is *construction* or *selling* | partly | `body::turn`; no turreted test of its own |
| 31, *A building being built up*: *construction* queued is replaced by *guard* | tested | `construction_queued_is_replaced_by_guard` |
| 31, *A building being built up*: *construction* current and processed completes at once, without a second opening | tested | `deploy::process_construction`; `construction_processed_completes_at_once` |
| 31, *A building being built up*: *construction* current and not yet processed restarts and cancels the idle-body request | tested | `deploy::process_construction` (`body::leave_idle` at the first processing); `construction_not_yet_processed_restarts_then_guard` |
| 31, *A building being built up*: *guard* current at the next turn's second start; the idle body at the end of that turn, or from *guard*'s first processing in the third case; the construction state ends | tested | `placement::opened_turn_end` (from `process::tail`), `combat::behave::building_process` (a *guard* with no idle body requests one); the three build-up tests |
| 31, *The factory*: ready at the request, its *guard* is current at once and its unloading ends | tested | `an_ended_unloading_leaves_the_door_open` |
| 31, *A unit that the factory has not yet sent out*: it follows, keeps its contact and stays at its exit point | tested | `a_queued_unload_is_replaced_and_the_unit_is_stranded`, `an_ended_unloading_leaves_the_door_open` |
| 31, *A unit that the factory has not yet sent out*: every release gives *failed* while it stands there | tested; Q-2841 | `release::ground` and `release::stranded_at` (`design: Q-2841`: only the stated case is built, a same-house vehicle with no destination and no movement in the exit point's cell); `a_queued_unload_is_replaced_and_the_unit_is_stranded` |
| 31, *A unit that the factory has not yet sent out*: an order that moves it ends its contact, and the releases work again | tested | `an_ended_unloading_leaves_the_door_open` |
| 31, *A unit that the factory has not yet sent out*: what the factory's apron requests do to it | seam | S-0309 Open questions; nothing is built |
| 31, *The factory's door*: the change does not close it; the later unloading finds it open and does not wait | tested | `release::process_unload` (the ground starting stage moves the door only when it is not open); `an_ended_unloading_leaves_the_door_open` |
| 31, *A vehicle docked at a repair depot*: a ready depot's *guard* ends its repair inside the change | tested | no code change needed; `a_ready_depot_ends_its_repair_inside_the_change` |
| 31, *A vehicle docked at a repair depot*: a depot not ready at the change (between two ticks, its mark used by its last start) goes on repairing until its next start makes *guard* current, with no further repair step; the vehicle follows, in *guard*, docked where it stands | tested | `a_depot_stops_repairing_and_the_vehicle_stays_docked` |
| Edge cases: `owner_change_interrupts_building_work` (default on); off, no guard request and no fall-back for the sale, the vehicle factory's unloading and the depot's repair, and the other effects still apply | tested both ways | `quirks.rs`, `ownership::keeps_work`; `quirk_interrupts_building_work_off_keeps_the_builddown`, `quirk_interrupts_building_work_off_sends_the_unit_out`, `quirk_interrupts_building_work_off_keeps_the_repair`, `the_grand_opening_acts_once_and_whatever_the_switch`; C7: not admitted |
| 23 (d): an infantry unit's arrival falls back at the end of the update that ends its walk, and its rest behaviour is current at the next tick's first start | tested | built already (S-0215 claim 22's end-of-update check, `process`); `an_infantry_arrival_falls_back_at_the_end_of_its_update` |
| 18: a house counts as allied with itself; no exception for an aircraft on its pad; out of ammunition is not a refusal | no change | text only; `acquire::retaliate` already reads `w.allied` (true for the same house), refuses only *illegal* and *impossible*, and has no pad exception |
| Q-2802 interims (intake 112) | left, as the brief says | — |

## Intake 114's S-0309 uses

- Claim 18's slave, master and spawner row: r126 adds text only to the alliance line (a house is
  allied with itself) and two paragraphs after the list (no pad exception; out of ammunition is
  not a refusal). The slave, master and spawner row is unchanged. `acquire.rs` line 196 already
  reads `w.allied`, which is true for the same house. The firing-check row refuses only
  *illegal* and *impossible*, and there is no pad exception. Nothing changes.
- Claim 25 part h: the r124 to r126 delta has no claim 25 hunk. Nothing changes.
