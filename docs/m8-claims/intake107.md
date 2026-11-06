# S-0203 r53 and S-0107 r46: claim ledger, M8 step A3 intake 107

- **S-0203** r50 to r53 (commit ce50a38, from the r50 that intake 105 built, 5534a23). r53 answers
  Q-2691 (claim 13 (f): a sweep reaches every leftover at its place, whatever removed it, a sweep
  or a death weapon of the same sweep included), Q-2692 (claim 13 (g): a leftover with health left
  takes a killing hit; sinking and crashing at that hit) and Q-2693 (claim 13 (h): every leftover
  holds its place in S-0414 claim 15's creation order, in both positions of the switch). It
  rewrites claim 13 (c) (the hit goes to every leftover vehicle of the house, with or without
  health), (b) (a passenger killed with its transport keeps its health), (e) (a vessel kept to the
  end of the game), the Term *leftover*, the Edge case of `quirks.defeat_sweep_leftovers`, scenario
  O18 parts f to j, and the Changes for other specs (Glossary, S-0414 claims 5, 11, 15 and 21,
  S-0312 claims 1 and 32, S-0504 claim 4, S-0302 claim 8).
- **S-0107** r40 to r46 (commit 651f294, from the r40 that intake 99 built, 55f236c). r46 answers
  Q-2616 with claim 11's *Changes of target*: every change of an infantry unit's target (another
  target, the loss of its target, or one gained when it had none) is a change of target for the
  attack state's ending and its request, at the moment it is made, whatever rule makes it and in
  whichever actor's turn; the attack-move discard and put-back of S-0309 claim 23 are the only
  exception. No quirk is added or changed.

Requests filed (block Q-2716 to Q-2725): **Q-2716** (whether a leftover keeps its tag, so that
claim 29's raises come at the sweep's killing hit; interim: removal detaches the tag, no raise) and
**Q-2717** (whether a mind-control release reads the guard request's readiness before or after its
target clear, now that the clear ends the attack state; interim: before). Q-2718 to Q-2725 unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

### S-0203 r53 (claim 13)

- **The store** (`glasnost_sim::leftover`). `Leftover` is now the object and a `kept` mark; the
  death mark of intake 105 is gone, since the hit no longer depends on how the vehicle left.
  `leftover::clear` (the start of `tick::step`) drops every leftover that is not kept, so a kept
  vessel stays in the store, out of play, off the map, with its place, to the end of the game
  (claim 13 (e), (g)). It is saved and hashed with the rest.
- **The passenger's health** (claim 13 (b)). `transport::died_with_transport` (used by
  `kill_passengers` and claim 12's failed escape) reads the passenger's health before its death and
  sets the leftover's health back to it (`leftover::keep_health`). Infantry passengers are offered
  no hit (claim 13 (c)), so only a vehicle passenger's killing hit follows from it.
- **The sweep's reach** (claim 13 (c), (f)). `econ::victory::destroy_all` already walked the
  house's objects and every leftover together in creation order; the leftovers an earlier sweep of
  the same house update or a death weapon earlier in the same sweep made are reached at their places
  (the intake 105 interim of Q-2691, now the spec). Under `quirks.defeat_sweep_leftovers`
  `leftover::sweep_hit` now reaches every leftover vehicle of the house, whatever removed it:
  a death, a sale, a deploy, an absorption, the grinder, a crush, an erasure or a passenger's death
  with its transport.
- **The hit** (`leftover::sweep_hit`). Its amount is the leftover's health. `Effect::LeftoverHit`
  now carries the health before and after (`(i32, i32)`). With no health: no health taken, nothing
  registered, the death effects once more (as intake 105). With health left: health to 0, a
  killing hit whose destruction `damage::credit` registers once more with no attacker and no house
  (a loss, no kill; S-0414 claim 21), with the leftover put back among the objects for the call
  alone (`leftover::with_restored`), then the death effects as at any killing hit.
- **Sinking** (claim 13 (g)). `death::repeat_plan` reads S-0312 claim 10's conditions
  (`naval::sinks_at_kill`) for a leftover with health left only; a leftover with no health starts
  no sinking. One that meets them takes claim 10's branch: no claim 9 animation or pick, health back
  to 1, marked sinking (`naval::start_sinking`, the leftover put back for the call), and, of a type
  that cannot crash, kept (`Repeat::kept`), with no removal and so no crate. One whose type can
  crash fails its crash start, is removed again (its crate placed, S-0312 claim 14) and ends with
  its tick.
- **Crashing** (claim 13 (g)). A leftover of a type that can crash above the terrain at the hit
  (`Repeat::crash_start`) starts a crash after its other effects: it holds no passengers in our
  engine (`death::leave_play` dealt with them) and takes S-0312 claim 16's three tumbling draws
  (unit interval, ranged 0 to 1, unit interval); taking no turn, it does not fall, so no removal and
  no crate, and it ends with its tick. One on the ground shows claim 9's animations and starts no
  crash.
- **Places** (claim 13 (h)). Every leftover of the four kinds holds its place for claim 15's
  cancellation skip in both positions of the switch, as built at intake 105; the cite now names
  claim 13 (h) in place of the Q-2693 interim.
- **The M-0021 recorder** (`oracle_m0021`) reads the hit's health pair directly.
- **Formats.** Save format 108 to 109, AI game format 73 to 74 (`Leftover`'s fields), one past main
  01c5688 (A4 round 9b took AI game 73).

### S-0107 r46 (claim 11, *Changes of target*)

- `infantry::watch_test` compares the unit's target with its baseline (`InfantryState::watch`):
  any difference, the gaining of a target from none included, is a change (`target_changed`: the
  attack state ends and the unit asks, not forced, for *deployed*, *prone* or *stand*). The intake
  99 test ignored a gain from none; that exception is gone.
- The rules that write an infantry unit's target call it right after the write, so the change takes
  effect when it is made, in whichever actor's turn: orders (`orders::apply_pending`, both places
  that set the baseline before an order now test), a retaliation or other override
  (`combat::acquire::override_attack_on`, in the deliverer's turn), a berserk hit's drop
  (`psychic::drop_target`), a guard reset and a domination takeover (`econ::power::guard_reset`), a
  mind-control release (`mind::release_orders`), the reverts that put an earlier target back when an
  override ends (`cloak::machine`, `teleport`, `bridges`), an open-topped transport's clear and
  hand-down (`transport::clear_passenger_targets`, `hand_down_target`), and a healer's drop in its
  own firing step (`combat::behave::infantry_step`). The target leaving play already called
  `target_changed` (`held::holders_drop`).
- The test at the start of the unit's turn (`tick::object_turn`) now tests instead of only setting
  the baseline, so a rule that writes a target without testing ends the state before any start of
  the unit's update.
- The lazy ending at the firing turn (the Q-2616 interim in `combat::behave::infantry_step`) is
  removed.
- A release reads the readiness of its guard request before its target clear (interim Q-2717), then
  clears, tests and makes the guard request.

## Claims

| Claim | Status | Tests and notes |
|---|---|---|
| S-0203 13 (b): a passenger killed with its transport keeps its health | tested | `claims_s0203_r53::a_passenger_killed_with_its_transport` |
| 13 (c): the hit to every leftover vehicle of the house, whatever removed it | tested | `claims_s0203_r50::o18_e_the_sweep_hit_raises_no_tag_and_only_a_killing_hit_counts` (a removal with health left) |
| 13 (e): a kept vessel survives a save and load | tested | `claims_s0203_r53::a_kept_vessel_survives_a_save_and_load` |
| 13 (f): a death weapon of an earlier place, reached at the later place (O18 f) | tested | `claims_s0203_r53::o18_f_a_death_weapon_destroys_a_later_place` (both creation orders, both positions) |
| 13 (f): two sweeps in a surrender's house update (O18 g) | tested | `claims_s0203_r53::o18_g_two_sweeps_in_a_surrender` (both positions) |
| 13 (f): three sets of effects for a vehicle removed earlier in that tick | partly | follows from the same walk; no test sets up both at once (the spec leaves it `unmeasured`) |
| 13 (g): a killing hit for health left, a loss and no kill | tested | `o18_e_the_sweep_hit_raises_no_tag_and_only_a_killing_hit_counts`, `o18_h_a_vessel_whose_descent_ends` |
| 13 (g): claim 29's raises at that hit | seam (interim Q-2716) | the tag is detached at removal, so none comes |
| 13 (g), Sinking: claim 10's branch, the vessel kept (O18 h) | tested | `claims_s0203_r53::o18_h_a_vessel_whose_descent_ends` |
| 13 (g), Sinking: a crashable vessel removed again, ends with its tick | tested | `claims_s0203_r53::o18_h_a_crashable_vessel_ends_with_its_tick` |
| 13 (g), Sinking: no health, no sinking; the bomb mark's wake and splash | partly (seam) | no sinking without health is built; S-0312 claim 11's wake and splash is not built in our engine (seam, as at the first death) |
| 13 (g), Crashing: a crash ended on the ground (O18 i) | tested | `claims_s0203_r53::o18_i_a_vehicle_whose_crash_ended` |
| 13 (g), Crashing: above the terrain, three draws, no fall, no crate | tested | `claims_s0203_r53::a_leftover_above_the_terrain_starts_a_crash` |
| 13 (h): places in the cancellation skip, both positions | tested | `claims_s0414::claim_15_cancellation_skip_falls_on_a_leftover` |
| Edge cases: the switch off (O18 j) | tested | `claims_s0203_r53::o18_j_switch_off_keeps_no_vessel`, the off halves of f and g; C7 verdict in [c7-play-fixes.md](c7-play-fixes.md) (rejected, unchanged) |
| S-0107 r46 11, *Changes of target*: a retaliation's override | tested | `claims_s0301::s0107_claim_11_attack_ends_on_a_new_target` |
| 11: a berserk hit's drop | tested | the same test |
| 11: a target gained from none | tested | the same test |
| 11: no change, no ending (the same target) | tested | the same test |
| 11: a write that tests nothing, caught at the start of the turn; no lazy ending | tested | `claims_s0301::s0107_claim_11_a_change_written_elsewhere_ends_before_the_turn` |
| 11: orders, the target leaving play, own processing and search | tested | intake 99's tests in `claims_s0215_r118.rs`, unchanged |
| 11: guard reset, takeover, release, reverts, transport clears | partly | hooked; no test of their own (the hook is the same call) |
| 11: the attack-move exception | tested | intake 99's tests, unchanged |

## Records

S-0203 r53 reaches M-0021's two leftover rows (intake 105 built them). S-0107 r46 reaches the
M-0020 rows whose games put infantry under mind control (the takeover's guard reset and the
release now end the attack state). Arena `--record all --threads 2`, main 07d549c (before)
against this branch (after; bases under Lanes):

| Record | Row | Before | After |
|---|---|---|---|
| M-0021 | Reading points: a vehicle destroyed by a hit at *t* has a per-tick row stamped *t*, off the map, with no health left | holds | **holds** |
| M-0021 | Reading points: in every part B run the last F vehicle destroyed takes one more hit in its death tick, after its destroying hit, naming no attacker, with no test warhead, taking no health | holds | **holds** (a leftover with no health takes none, claim 13 (g)) |
| M-0020 | Part 1: in all four YURI games every unit under mind control at the first such tick is of the opposing group and owned by the YURI's house | holds | **holds** |
| M-0020 | Part 2: YURI against SHK, arm S (12 of 12 to side A) | fails (11 to 1) | fails (11 to 1), unchanged; one game ends at 304 in place of 608, same winner |

M-0021 fails 8 of 50 rows before and after, the same rows word for word. Every other M-0020 row
passes, as before, and the first-capture ticks are unchanged.

## Goldens

Eight goldens change, regenerated on main 01c5688 after the rebase: `combat`, `duel`,
`duel-diagonal`, `duel-m6c`, `duel-m7a`, `duel-m7b`, `duel-naval` and `skirmish`. Against main,
111 lines change in each direction and none is added or removed. Only state hashes change, and only
in ticks that hold a leftover: a leftover now stores whether it is kept, so its hashed form
changed. No event, count or end changes, and `scripted` is unchanged. Formats: save 108 to 109, AI
game 73 to 74, wire 12 unchanged.

## Lanes

Before: binaries built on main 07d549c. After: this branch's code on 0051f51 (origin/main then).
The commits between those two bases are intake 109's, in the app and the string table only, with
no simulation or AI code. Those bases hold no A4 round 9b, so the sweeps, the arena and the quick
lane below have not seen it. The branch was then rebased onto main 01c5688, which merges A4
round 9b (AI end-games) and spec commits (S-0208 r52, S-0506 r34). There
the AI suite and the checks were run again (the AI suite line says which base each figure is
from). Local lane; install values are not quoted.

- **Sweeps, seeds 1 to 16.** All six runs are identical before and after, apart from timings.
  - M6c reference: smallest minimum 0. M6c play fixes: 0.
  - M7a reference: 1. M7a play fixes: 2.
  - M7b reference and play fixes: drain minimum 3.
- **Arena, `--record all --threads 2`.** Every table and verdict is unchanged: M-0001 11 of 96
  failing, M-0002 2 of 24, M-0003 1 of 35, M-0019 0 of 6, M-0020 1 of 25 (YURI against SHK, arm
  S) and M-0021 8 of 50.
  - Three of our 1152 M-0001 games changed, all MIND games. One changed winner: MIND against E2,
    row on side A, seed 0, row to column. The other two changed only their end tick: MIND against
    E1, side B, seed 4 (912 to 1216), and MIND against E2, side B, seed 2 (304 to 608).
  - The changed winner, traced (`--trace MIND,E2,A,0`): the first difference is at tick 194. At
    tick 193 an E2 of side B under the MIND's control takes a guard reset that clears its target
    (behaviour *hunt* to *guard*). Under claim 11's *Changes of target* that loss of target ends
    its attack state at once, so at tick 194 it takes its next target and fires. Before, it fired
    at tick 197. This follows from S-0107 r46.
  - M-0020 arm S: one game ends at 304 in place of 608, with the same winner.
- **`cargo test --release -p glasnost-ai -- --include-ignored`** (after: `--no-fail-fast`).
  - Before: 255 pass and one fails, `skirmish::plays_to_the_last_side` ("the game is over within
    40000 ticks"). That is main's 228 pass and 1 fail up to that binary, plus `special`, `terror`
    and `water` run from the before binaries, 27 pass.
  - After: 33 binaries, 255 pass and the same test fails with the same message. A4 round 9b owns
    it. This change does not move it.
- **AI quick lane, seeds 7 to 10, maps 0, 2 and 8** (`GLASNOST_AI_QUICK=1`).
  - 19 of 36 games change: 3 at seed 7, 7 at seed 8, 6 at seed 9 and 3 at seed 10.
  - 5 change winner, and in each the house of side 2 (Yuri) is the one that loses or no longer
    holds out:
    - seed 7, map 2, sides 0 and 2: house 1 to house 0;
    - seed 7, map 2, sides 1 and 2: house 1 to house 0;
    - seed 8, map 2, sides 0 and 2: house 1 to house 0;
    - seed 9, map 0, sides 1 and 2: house 1 to house 0;
    - seed 10, map 8, sides 1 and 2: no end within 120000 ticks before, house 0 wins at 69836
      after.
  - Before, 35 of 36 games end and house 0 wins 21. After, all 36 end and house 0 wins 26. The
    longest game after runs 95455 ticks.
  - **Seed 7, map 2, sides 0 and 2, traced** (`GLASNOST_AI_TRACE`, `GLASNOST_AI_ECON`,
    `GLASNOST_AI_DUMP`, `GLASNOST_AI_WATCH`). The first difference is at tick 19848. A Yuri slave
    leaves its miner, and on emergence its target becomes the master (S-0402 claim 21), a target
    gained from none. Before, it stays in *harvest* through that tick and takes *guard* at 19849.
    After, it takes *guard* at 19848, one tick earlier.
  - Under r46 a target gained from none is a change of target, which ends the attack state and
    asks for *stand*. Infantry in the attack state is not ready to take a queued behaviour (S-0215
    r118 claim 11). The earlier *guard* is consistent with that, but the watch line does not print
    the attack state, so the trace does not single out the rule. From there on the game parts:
    house 0's credits differ by tick 20210, and house 0 wins at 57013 where house 1 won at 72778.
  - The other winner changes were not traced one by one: whole AI games part widely after any
    change to unit behaviour.

## Seams and open points

- Q-2716 (tags at the killing hit), Q-2717 (release readiness): interims above.
- S-0312 claim 11's wake and splash for a bomb-marked vehicle low over water is not built in our
  engine, at the first death or at the sweep's repeat.
- Infantry, aircraft and building leftovers are offered no hit (claim 13 (c); S-0203 Open
  questions).
