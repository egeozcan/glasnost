# M8 A3 intake 119: S-0501 r176 and r178

Brief: `.claude/worktrees/M8-A3-INTAKE119-BRIEF.md`. Spec: S-0501 r176 (b15de27) and r178
(58a3096), after intake 118's r172 (401710c). Answered requests: **Q-2866** (the pad last heading
for after a contact granted in *sticky*: r176, claims 21, 24 and 29), **Q-2867** (the moment with
no destination in T2's in-flight pad row: r176, claim 18 and T2) and **Q-2868** (how a kamikaze
is removed after its shot: r178, claim 20 and scenario AC36). Intake 118's interims for Q-2866 and
Q-2867 are replaced, and no `// design:` note for them is left. No new request. r176 and r178 add
no quirk name; every S-0501 switch is checked, with C7 verdicts in
[c7-play-fixes.md](c7-play-fixes.md).

## Ledger

| Claim part | Status | Where, tests |
|---|---|---|
| S-0501 r176 claim 21, *The pad last heading for*: every search records its result (the pad kept, the pad chosen, or none), pad-bound or not, in every behaviour and at every use | built, tested | `air::pad_search` (records; `air::find_pad` stays a pure choice) used by T2's in-flight pad row, the after-the-row search, claim 29's search below cruise height and S-0507's drop-off search (`transport`); `claims_s0501_r176.rs` `claim_21_every_search_records_its_result` |
| S-0501 r176 claim 21: a pad-bound aircraft keeps its recorded pad while it accepts, even with a nearer free pad; one that is not pad-bound takes the nearer | built (already), tested | `air::find_pad`; `claim_21_a_pad_bound_aircraft_keeps_its_recorded_pad` |
| S-0501 r176 claims 21 and 23: a contact granted after the search records nothing, in *sticky* or not | built, tested | `air::request_contact` (the Q-2866 interim's write removed); `claim_21_a_contact_records_nothing` |
| S-0501 r176 claim 21: the destruction or removal of the recorded building leaves no pad recorded | built, tested | `air::heading_pad_gone`, from `combat::death::mark_killed` and `objects::remove_with`; `claim_21_a_destroyed_or_removed_pad_leaves_no_record` |
| S-0501 r176 claim 21 and S-0404 claims 13 and 15: a factory release at a pad's docking position records that pad | built, tested | `econ::release::aircraft`; `claim_21_a_factory_release_at_a_pad_records_the_pad` |
| S-0501 r176 claim 24: at every processing of *enter*, an aircraft in flight whose destination is a building other than its record records that building, of its dock types or not | built, tested | `air::enter`; `claim_24_enter_in_flight_records_its_destination_building` |
| S-0501 r176 claim 24: the record rule on the ground (a pad-bound aircraft on the ground records its destination building, a cell records none) | built (already) | `air::bound_away`; unchanged |
| S-0501 r176 claim 18, *The moment with no destination*, and T2: every use of the in-flight pad row gives no destination after the search and before the contact request, in any behaviour, found or not, granted or not | built, tested | `air::seek_pad`, `air::fall_back_rows`; `claim_18_the_in_flight_pad_row_gives_no_destination_for_a_moment` (*guard*, *sticky*, *move*, *enter* with no destination; the moment's draws over a refused cell, none over an accepted one) |
| S-0501 r176 claim 18: no other row has it, the after-the-row search included; *enter* with a destination takes the guard row | built, tested | `claim_18_the_after_the_row_search_has_no_moment` |
| S-0501 r176 claim 18 and T2: pad-bound with no grant: none, then the crash, which begins with the flight point the moment left | built, tested | `air::fall_back_rows`, `air::crash_fall_back`; `claim_18_a_pad_bound_crash_begins_with_the_moments_flight_point` |
| S-0501 r176 T2: not pad-bound with no grant: none, then the F8 spot and *move* | built | `air::fall_back_rows` |
| S-0501 r176 Edge cases, `quirks.unarmed_pad_bound_hover` off: the unarmed row's search has the moment too | built, tested | `air::fall_back_rows`; `quirk_unarmed_pad_bound_hover_off_has_the_moment` (both positions) |
| S-0501 r178 claim 20, *Kamikaze removal*: a kamikaze is removed without dying at the end of its first firing pass with an ordinary attempt (an ordinary shot with or without a projectile, a drain attempt after its continuation, a suicide attempt after its hit; not a drop attempt; a refused check is no attempt) | built, tested | `air::attack_stage`, `air::full_burst`, `air::not_a_drop`, `air::kamikaze_removal`; `claims_s0501_r178.rs` `ac36_a_a_kamikaze_is_removed_at_the_end_of_its_first_pass` (with the control not enrolled) |
| S-0501 r178 claim 20: the removal comes after the pass's last attempt; it neither cuts the pass short nor adds to it | built, tested | `ac36_b_the_whole_burst_comes_before_the_removal` |
| S-0501 r178 claim 20: the removal is no death: no hit, death effects, kill or loss, S-0011 raise or round; the tag stays; a planted bomb is removed silently; it leaves the group and is a leftover for the rest of the tick | built, tested in part | `objects::remove` (the removal path every removed object takes); `ac36_a_…` (no `Killed` effect, totals unchanged, ammunition kept, gone in *s* + 1). The tag, the bomb and the raises are `unmeasured` and not set up here |
| S-0501 r178 claim 20: the pass's projectiles have no firer, so a kill they make credits no house and the target's house takes a loss | built, tested | `air::kamikaze_removal` (attacker and house cleared); `ac36_c_a_kill_by_the_pass_credits_no_house` |
| S-0501 r178 claim 20: a kamikaze not of a rocket type left its slot at enrollment, so the removal's loss notification changes no slot | built (already) | `spawn::lost`; unchanged |
| Intake 118: *sticky* and *guard* at a pad (claim 29), the contact at cruise height, claim 48 (b)'s third case, the missile-spawn ground rows (AC35) | checked, stands | `claims_s0501_r167.rs` and `claims_s0501_r150.rs` pass on this tree |
| Intake 118 rows for Q-2866, Q-2867 and Q-2868 | replaced | `intake118.md` rows note it |

## Quirks

All 23 S-0501 switches keep their default, on, as the spec gives it. Only
`unarmed_pad_bound_hover` (its Edge case) and `aircraft_descent_touchdown_untested` (named in the
moment's text) are touched by the delta; both are judged again in
[c7-play-fixes.md](c7-play-fixes.md). Neither is admitted.

## Formats

Save format 122 → 123, AI game format 88 → 89, one past intake 120's (main 16df36f). The pad
record of every search, the moment and the kamikaze removal change the hashed state: on 2b0e419,
before the rebase, the sim-gate skirmish golden first differed after tick 1115, and the skirmish
and the three duels (`duel`, `duel-diagonal`, `duel-naval`) moved while combat and the scripted
goldens did not. Goldens regenerated again on the tree rebased onto 16df36f.

## Lanes

The lanes were run before the rebase onto 16df36f (intake 120). Before: main 2b0e419. After: this
branch's code on 2b0e419 (code commit b02475f, format bump 3e20417, then save 121 and AI game 87).
They were not run again on 16df36f; the final checks were.

- Seed sweeps, seeds 1 to 16: M6c, M7a and M7b are identical before and after, minimum,
  median and maximum of every counter.
- Arena (`--record all --threads 2`): every output file in `--out` is byte-identical. M-0001 fails
  12 of 96, M-0002 2 of 24, M-0003 2 of 35, M-0019 0 of 6, M-0020 1 of 25 and M-0021 11 of 50,
  before and after.
- `glasnost-ai --include-ignored` (release): 264 pass before and after.
- AI quick lane, seeds 7 to 10 on maps 0, 2 and 8 (36 games): no winner changes. House 0 wins 22
  and house 1 wins 14, before and after. 15 games change length (seed 7: five, seed 8: three, seed
  9: three, seed 10: four), which fits a change to how aircraft fall back. No game changed its
  winner, so none was traced.
