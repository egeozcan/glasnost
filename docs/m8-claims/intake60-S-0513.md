# S-0513 r43: claim ledger, M8 step A3 intake 60

This intake takes in **S-0513 r43** (superweapon effects), from r34 (3e898b5). The revision
answers **Q-2156** (intake 52b): what the holders of a superweapon warp member do at its departure.
Its changes:

- the *leaving stage* and *arrival hold* glossary entry: the leaving stage ends at departure;
- claim 38: the fate is applied in the member's arrival effects, the update after its placement;
- claim 39: the departing member disappears from its old position and is off the map until it
  appears. The old text "every object targeting it drops it as in S-0207 claim 20 (e)" is gone.
  New paragraphs cover:
  - the departure's holders (no drop, no draw, no revert);
  - the holders of a member that the landing check kills, and the order of their draws;
  - the own jump's lack of a draw;
  - a projectile in flight towards the member;
  - the leaving-stage firing answer (*illegal*, rank 4, temporal-weapon exception);
  - the opportunity-target drop at a search, and the infantry firing step that keeps the member;
- table T2's departure row;
- scenario SW12 part (i), the holders of a warp member;
- two new Open questions (a holder's own processing after departure; where a projectile flies);
- Changes for S-0309 claim 27, S-0306 claim 17, S-0207 claim 20 (e), S-0301 (the firing check)
  and S-0215 claim 10;
- Q-2156 added to `requests`.

The revision adds no quirk (no `quirks.` name in the diff), so no switch and no C7 layer
judgement is needed. `every_spec_quirk_has_a_switch` is unchanged.

**Simulation change.**

- `superweapons::warp::depart` no longer calls `teleport::targeters_drop`. The departure clears
  no holder's target, takes no draw, reverts no override and leaves projectiles aimed at the
  member.
- `combat::check::check` gains the leaving-stage answer (`superweapons::warp_leaving`), just
  before the held-spawner *impossible*.

No hashed state was added: save format 91, AI game format 53. Goldens: see Measurements.

The tests are in `crates/deterministic/sim/tests/claims_s0513_r43.rs` (12 tests, synthetic
types from `tests/sw_common`). The fixture:

- holder *h*1, a CART, at (0,3), and holder *h*2, a WALKER, at (3,0);
- the member *m*, a BUGGY of the house BETA, at (0,0);
- a warp to (5,5).

The member's step is set apart from the holders' own processing as SW12 part (i) does: its warp
update (`superweapons::warp::turn`) is called alone between two reads. A whole-tick test makes
the holders before the member, so their turns come first (S-0203 claim 3).

`claim39_targeters_drop_the_member_at_departure` (intake 52b, `claims_s0513_warp.rs`) tested the
old text and is removed.

## Claim 39: the departure

| Item | Status | Tests / notes |
|---|---|---|
| The member disappears from its old position at *t*_L + 62 and is off the map until it appears; being erased ends; phasing in; the landing check (T2 row) | tested | Built earlier (`warp::depart`, `lift_off`). Existing tests in `claims_s0513_warp.rs`. The off-map state is `unmeasured` in the spec |
| Departure clears no holder's target and takes no holder timer draw over 4 to 8 ticks, whatever the timer shows (more than 10 ticks left and at most 10) | tested | `claim39_departure_clears_no_holder_and_takes_no_timer_draw`: *h*1 with 50 ticks left and *h*2 with 10; the member's departure update alone. Both still hold *m*, both timers are unchanged, and the stream equals that of a control world whose holders hold nothing (here the warp-out animation has a constant delay, so the update takes no draw at all) |
| A holder with an override in effect does not revert at the departure (S-0215 claim 10 Changes) and keeps the member and its suspended behaviour | tested | `claim39_departure_reverts_no_overridden_holder`. The departure is no longer among `combat::held`'s revert causes, because nothing is called there |
| A projectile in flight towards the member keeps it as its target object; its target does not become the old cell (S-0306 claim 17 Changes) | tested | `claim39_a_projectile_in_flight_keeps_the_member`: an arcing shot fired at *m* before departure still has `Target::Object(m)` after the member's departure update |
| Where that projectile flies while the member waits to land, and whether it can hit it | tested (intake 64) | **Q-2212**, answered by S-0306 r57 claim 17 and built at intake 64 ([intake64-S-0306.md](intake64-S-0306.md)): the projectile keeps the member, reads its departure position until it appears, then turns to the landing point; the area damage misses it off the map. Intake 60's interim (the projectile's own next update dropped the off-map target object) is gone |
| A holder's own processing after departure: whether it drops the member, and whether it may fire at it while it phases in | request | Open question in the spec. **Q-2211**. Interim: S-0309 claim 27's DECIDED rule. `held::firing_target` treats the off-map member as no target, and `held::turn_start_drop` drops it at the start of the holder's next turn, with no draw and no revert. `claim39_whole_tick_holder_first_then_the_interim_drop` (one infantry holder, made before the member) shows both halves: in the departure tick the holder's turn comes first and it still holds *m* at the end of the tick, with its timer unchanged; in the next tick its turn drops it, with the timer still unchanged (no draw). **Restated by S-0513 r62** ([intake 71](intake71-S-0513-S-0216.md)): S-0309 claim 27's rule does not apply to a holder that kept the member through departure, and the member's firing answer after departure stays open (Q-2211 question 3). The same drop is kept as our own interim under Q-2211 question 3, with no behaviour change |
| "A change is the member's step's only when it is made in the member's own update" | tested | The tests' reads between `warp::turn` calls (above) |
| The later warp steps (a failed or redirected landing check, placement, arrival effects) do not themselves clear holders' targets | tested | `claim39_placement_and_arrival_effects_clear_no_holder` and `claim39_a_failed_and_a_redirected_landing_check_clear_no_holder` (an unprotected TOWER on the landing cell, as in SW13 part b). Before each of the member's lone updates (departure, the failed check at departure, the redirected check, placement, arrival effects), *h*1 is set to hold *m* with 50 ticks left and *h*2 with 8. After the update both still hold it, the timers are unchanged, and the stream equals that of a control world whose holders hold nothing. Only lone updates are run, so the Q-2211 turn-start drop does not take part |

## Claim 39: a member killed by its landing check

| Item | Status | Tests / notes |
|---|---|---|
| A member that the landing check kills is a destroyed object: its holders drop it with S-0309 claim 27's timer reset, at the killing hit | tested | `claim39_holders_of_a_member_killed_by_its_landing_check_draw_after_the_warp_out`. The member is killed through `combat::death`, which calls `held::holders_drop(Destroyed)` |
| In the departure update the warp-out animation's creation draws come before those holders' draws; the holders draw among themselves in creation order | tested | Same test. The warp-out animation (WARPA, random rate [450, 225]) takes its draws first. A control world lands the member on a free cell to give the stream after the animation, and the next draws are *h*1's and *h*2's `ranged(4, 8)` in creation order (*h*1 with more than 10 ticks left, *h*2 too) |
| Within the landing check the hits follow the landing cell's listing: an object killed earlier in the listing, and its holders, draw before the member's holders | partly | Built by claim 37's listing walk (`warp::check_or_redirect`), which kills in listing order, with each kill's holders' draws at that kill. Not separately tested: the walk can be cut by the `superweapon_list_cut_by_removal` quirk, and the victims' death draws make an exact stream hard to predict. `unmeasured` in the spec |
| Where those holders' draws fall against the member's own destruction draws | open | S-0309 claim 27's open question, unchanged. The order is still `death::mark_killed`'s, as in intake 52b |

## Claim 38: the fate in the arrival effects

| Item | Status | Tests / notes |
|---|---|---|
| The fate is applied in the member's arrival effects, the update after its placement | tested | Built earlier: `warp::arrival` is the update after `Placed`. Existing claim 38 tests in `claims_s0513_warp.rs` |
| A member that the fate destroys is a destroyed object, and its holders drop it as S-0309 claim 27 gives | partly | The fate's kill goes through `combat::death`, so `holders_drop(Destroyed)` runs. Under the Q-2211 interim a holder still holds the member at that point only if it picked it up again after it appeared, so no test sets one up. `unmeasured` in the spec |
| A member that the fate sinks without destroying it: S-0504 claim 5, which leaves open when its holders drop it | open | Unchanged. The sinking path drops holders as it did before this intake |

## Claim 39: the leaving stage

| Item | Status | Tests / notes |
|---|---|---|
| Throughout the leaving stage (Leaving and Expired), a firing check asked about the member answers *illegal* unless the asking weapon is a temporal weapon | tested | `claim39_leaving_stage_answers_illegal_ahead_of_rearming_and_range`: a holder with a rearm countdown running gets *illegal*, not *rearming*, both while the wait runs and in the expiry update (Expired is still the leaving stage); a holder out of range gets *illegal*, not *out of range*. Controls: the same holder gets *rearming* for the member while it is still Pending (before its first update), and *rearming* and *out of range* for a target not in a warp. Code: `combat::check::check`, `superweapons::warp_leaving` (`Member::leaving`). **Superseded by S-0513 r62** ([intake 71](intake71-S-0513-S-0216.md)): the answer holds from the launch (the Pending stage too) for every weapon; the Pending control now expects *illegal*, and the code calls `superweapons::warp_pending_or_leaving` |
| The temporal-weapon exception | tested | `claim39_a_temporal_weapon_is_not_refused_by_the_leaving_stage`: a temporal weapon gets the rest of the check's answer. **Withdrawn by S-0513 r62** (Q-2213 answered, [intake 71](intake71-S-0513-S-0216.md)): a temporal weapon gets *illegal* too; the test is now `claim39_a_temporal_weapon_gets_illegal_too` |
| Whether a target held by a temporal beam gets the same answer | request | "Not settled here" in the S-0301 Changes. **Q-2213**. Interim: the leaving-stage reason applies whatever holds the member. The holder's own held-temporal-target *rearming* (S-0514 claim 19) keeps its earlier rank. **Answered by S-0513 r62** and built by [intake 71](intake71-S-0513-S-0216.md): another firer's beam does not change the answer; a firer whose beam is attached keeps S-0514 claim 19's *rearming*, which ranks above. Whether a beam can be established on a member after the launch is the new request Q-2276 |
| Rank 4, after every other rank-4 reason except the spawner-weapon answers, the other weapon's live-effect *impossible*, the S-0306 claim 23 rows and the magnetic-beam *rearming* against another target, which it outranks; ahead of ranks 5 and 7 to 16 | tested in part, rest by placement | `claim39_the_materialising_firer_keeps_its_earlier_rearming`: the materialising firer's early *rearming* comes first. Ranks 5 and later: the rearming test above. The reason sits just before the held-spawner *impossible* (S-0614 claim 8), so every earlier rank-4 reason in the code wins (own-state refusals, the gripped unit, the drain and berserk refusals, the hidden-target, absent-weapon and power-sensitive *impossible*, the infantry healer, the building's pending delayed shot, S-0405 T1). The sinking, falling, deploy-transition, seized-firer and S-0517 magnetic-beam refusals sit after it in the code, but they also answer *illegal*, so their order is not observable. The label and rank are `unmeasured` in the spec |
| A holder whose weapon is not a temporal weapon and whose member is an opportunity target drops it at its next search in the leaving stage (S-0309 claims 4 and 19); the drop takes no draw over 4 to 8 ticks | tested | `claim39_an_opportunity_target_is_dropped_at_the_search_without_a_drop_draw`: at the timer's expiry the search's keep test (`acquire.rs`, claim 4) gets *illegal* and drops the member. The only draws taken are the search's own, with no `ranged(4, 8)` |
| An infantry holder whose weapon does not heal keeps the member when its firing step gets this answer | tested | `claim39_whole_tick_holder_first_then_the_interim_drop`: an infantry holder with 100 ticks on its timer runs whole ticks through the leaving stage and holds *m* at the end of every tick. That the keeping follows this answer is `unmeasured` in the spec |
| The same rules for vehicle, building and aircraft holders | partly | `unmeasured` in the spec. They take the engine's existing replies to *illegal*: a vehicle holder that is not a healer keeps its target in its firing step; an attacking building drops on *illegal* (S-0215 claim 32, intake 10); the aircraft attack run answers as before. No test runs a vehicle, building or aircraft holder through the leaving stage. **Restated by S-0513 r62** ([intake 71](intake71-S-0513-S-0216.md)): every holder covered by S-0309 claim 29 drops the member at the first check tick; vehicle and infantry holders in *attack* and *guard* are tested there (SW12 part j) |
| Other drops by a holder's own processing in the leaving stage | open | Open question in the spec (with Q-2211) |

## Claim 39: the teleporting locomotor's own jump

| Item | Status | Tests / notes |
|---|---|---|
| The own jump (S-0207 claim 20 e) never gives a holder the draw over 4 to 8 ticks; a holder that drops the jumper keeps its timer whatever it shows (S-0207 claim 20 e Changes) | tested | `claim39_the_own_jump_drops_with_no_timer_draw`: a TELE vehicle with the teleporting locomotor jumps while the BETA CART at (0,9) holds it with more than 10 ticks left. The holder drops it, its timer is unchanged, and no `ranged(4, 8)` is taken. `teleport::targeters_drop` is now the own jump's only caller, and its doc comment says so |

## Changes for other specs

| Item | Status | Tests / notes |
|---|---|---|
| S-0309 claim 27: the drop, timer reset and projectile rule do not apply to the departure, then or while the member waits to land; a member destroyed by its landing check is a destroyed object | tested | The departure and killed-member tests above. `combat::held` doc comments name the departed member and the Q-2211 interim |
| S-0306 claim 17: the departure is not a loss of target | tested | The projectile test above. Flight afterwards: **Q-2212**, answered by S-0306 r57 (intake 64) |
| S-0207 claim 20 (e): the own jump's drops take no timer draw | tested | The own-jump test above. No change of behaviour (`targeters_drop` took none) |
| S-0301: the leaving-stage answer | tested | The leaving-stage tests above |
| S-0215 claim 10: the off-map revert cause does not include the departure | tested | The override test above |

## Intake 57 (run in parallel)

Intake 57 builds S-0215 r88 and S-0607 r33, which this revision names: S-0215 claim 10 (the revert
causes) and S-0607 claim 11 (the magnetic-beam *rearming* that the leaving-stage answer outranks).
Intake 57 (merged to main as 559d48c) touched neither `combat::held`'s revert nor the place of
the magnetic-beam *rearming* in `combat::check`, so no conflict and no seam. The rebase onto it
was clean (see Rebases). The two claims' later revisions (S-0607 r37, intake 61) are not this
intake's.

## Requests

- **Q-2211**: what a holder does with a departed warp member (its own processing, the holder
  kinds, firing during the phase-in, the override revert). Interim: the turn-start drop.
  Answered in part by S-0513 r62 ([intake 71](intake71-S-0513-S-0216.md)); question 3 stays
  open, and the turn-start drop is kept as its interim.
- **Q-2212**: where a projectile aimed at a departed warp member flies, whether it can hit, and
  its target after a landing-check kill. Interim: the projectile's own update drops the off-map
  target object. Answered by S-0306 r57 claim 17 and built at intake 64
  ([intake64-S-0306.md](intake64-S-0306.md)); the interim is gone.
- **Q-2213**: the leaving-stage answer for a member that a temporal beam holds. Interim: the same
  answer. Answered by S-0513 r62 and built by [intake 71](intake71-S-0513-S-0216.md).

## Measurements

Before: 590bc3f (main at the start). After: this intake's sim commits on that same base, before
the rebase, so that the comparison holds this intake's change only. Those commits are 9ec4da7 and
5874477 after the rebase (6df00a2 and 7e9e091 before it). The later test-only commit 39750a7
cannot move a lane. Outputs are in the local, ignored `local-lane/m8/intake60/{before,after}/`.

| Lane | Before | After |
|---|---|---|
| M6c seed sweep 1–16 (`m6c_seed_sweep`) | ok, 29 counter lines | ok, the same 29 lines (identical min, median and max). Minimums of 0: hostile intent seen, infantry absorbed (as before); all others at least 1 |
| M7a seed sweep 1–16 (`m7a_seed_sweep`) | ok, 26 counter lines | ok, identical. Minimums of 0: hostile intent seen, psychic reveal orders and reveals fired, gattling rises and top stage, takeover orders and takeovers, gas clouds and hits (all as before) |
| M7b seed sweep 1–16 (`m7b_seed_sweep`) | ok, 34 counter lines | ok, identical. Minimums of 0: bunker entry, bunkered, bunker exit, lift orders, seized, dropped (all as before) |
| Arena, `--record all` (M-0001) | 11 of 96 scored pairings failing (win test 3, side outcome 9); unanimous outcomes reproduced 161 of 171 | the same; `ours-M-0001.csv` byte-identical |
| Arena (M-0002) | 1 of 24 (side outcome 1); 46 of 47 | the same; CSV byte-identical |
| Arena (M-0003) | 2 of 35 (side outcome 2); 62 of 64 | the same; CSV byte-identical |
| AI quick lane (seeds 7–10, maps 0, 2 and 8; 36 games) | house 0 won 18, house 1 won 18 | identical: every winner and end tick, and every counter line (only timings differ) |
| `cargo test --release -p glasnost-ai -- --include-ignored` | 239 passed, 0 failed | 239 passed, 0 failed |

No lane moved.

Goldens: none moved. After the rebase, `cargo test --release -p glasnost-sim-gate --tests --
--include-ignored` passes with every golden as committed (`golden.rs` both tests, `m6c`, `m7a`,
`m7b`, `margins`, `play_fixes`), and `cargo xtask gate` matches every golden. Without `--tests`
the same run also builds the crate's doctest target, which failed to build here: rustdoc reported
E0463 (cannot find the crates `glasnost_ai`, `glasnost_sim`, `glasnost_rules`). In the same
session two other build steps failed because a release or debug rlib was missing from `target/`
(fixed by `cargo clean -p`).

## Rebases

The branch was rebased (never merged) once, from 590bc3f onto 506ace6, picking up intake 57
(559d48c, 6eb6c91) and intake 62 (506ace6). The rebase applied cleanly: neither intake touches
`superweapons::warp`, `combat::check`, `combat::held` or `teleport`. The goldens and the checks
below were run on the rebased branch.

## Checks (on the rebased branch)

- `cargo test --workspace --no-fail-fast`: 7091 passed, 1 failed, 115 ignored. This ran before
  the test-only commit 39750a7. Its two tests then passed in `claims_s0513_r43` (12 of 12), with
  clippy and fmt clean on `glasnost-sim`. The one failure is
  `glasnost-rules` `quirks::tests::every_spec_quirk_has_a_switch`, and it fails on main as well.
  S-0306 r53, on main since 5f0c528, names the quirks `owned_shell_ground_pass` and
  `switched_off_hover_lead`, and their switches are intake 61's work. This intake adds no quirk.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean. `cargo fmt --all -- --check`:
  clean.
- `cargo xtask check`: deps, determinism, cite and canary ok; stock-guard reports the 105 known
  problems.
- `cargo xtask gate`: ok, every golden matched natively and on wasm32.
