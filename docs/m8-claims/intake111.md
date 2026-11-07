# S-0601 r23 and S-0111 r49: claim ledger, M8 step A3 intake 111

- **S-0601** r21 to r23 (commit 617b08a, from r21, the revision main built; intake 107 left Q-2717's interim on it). r23 answers Q-2717:
  the readiness that a release's guard request reads is the victim's after both clears (the target
  and the destination). For an infantry victim that holds a target, the target clear is a change of
  target (S-0215 claim 11, S-0107 claim 11) that ends an attack state at the clear, with its
  request for *stand*, *prone* or *deployed*, so that state no longer holds the start back. An
  infantry victim in the attack state with no target has no change at the clear and is still held
  back. A walking infantry victim under *attack* is then *attack* with no target, which its
  movement does not hold back, so with an interruptible sequence it starts *guard* at once. Every
  other readiness condition applies as usual. r23 adds MC4 parts (a) and (b), and the Backing and
  Confidence that go with them. No quirk is added or changed.
- **S-0111** r40 to r49 (commit 05ade81, from the r40 that intake 109 built). r49 answers Q-2736
  (claim 20, *Blocked*). In the route mode (S-0213 claim 16) a held right button blocks the tip as
  it does outside the mode, and so does a held left button whose press is an ordinary click. A held
  left button does not block it when the route mode refused its press at the press (for kind,
  S-0213 claim 16d, or by the point-click test, claim 16b), or when the press took a point, a join
  or a loop close, or, with nothing selected, took an own point that the click then selects
  (claims 16b and 16e). The block ends at the release, in the route mode too. The release over a
  highlighted or refused point after an ordinary click stays an Open question. r49 also updates
  the Backing and Confidence and scenario NT1 part (h). App only. No quirk is added or changed.

No request filed (block Q-2756 to Q-2765 unused).

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

### S-0601 r23 (claim 9)

- `mind::release_orders` now runs in claim 9's order:
  - the target clear, the burst counter included;
  - the change-of-target test (`infantry::watch_test`, S-0107 r46 claim 11), which ends an
    infantry victim's attack state and makes its request;
  - the destination clear (`econ::assign_destination(None)`);
  - then the readiness read (`process::ready`) and the guard request with its immediate start
    (`queue_and_start`, S-0215 claim 7).
- Intake 107's interim, which read the readiness before the target clear, is gone, and so is its
  `Q-2717` line.
- The simple-deployer skip (a vehicle in *unload*) comes first, as before.
- S-0309 claim 30's listed causes (intake 110: Capture, Trigger, Occupation and the trigger
  action's mobile branch) call the same function, so they take the same order.

### S-0111 r49 (claim 20, *Blocked*)

- `Session::route_press` now returns whether a route-mode left press is an ordinary click:
  - `routes::press` gives `Press::Ordinary`: an ordinary click. This covers a band box, a click
    that adds a point or selects at its release, and a press that the route mode refuses only at
    its release.
  - A kind refusal at the press (claim 16d), `Press::Refused` (the point-click test) or
    `Press::Take` (a point, a join, a loop close, or an own point taken with nothing selected): not
    an ordinary click.
- `Session::pointer_down` clears `view_button_held` when the press is not an ordinary click, so the
  tip's wait runs with the button held. The release clears the block as before.
- A right press in the route mode is never tested (claim 16d, Q-2193) and keeps the block.
- Watch mode (no local house) lays no route, and its press counts as an ordinary click.
- The `design: Q-2736` line is removed.

## Claims

| Claim | Status | Tests and notes |
|---|---|---|
| S-0601 9: the readiness read after both clears, a standing vehicle | tested | `claims_s0601::claim_9_release` (guard current, nothing queued) |
| 9 and MC4 (b): a standing infantry victim in the attack state with a target; the clear ends the state, guard current, nothing queued | tested | `claims_s0601::claim_9_mc4_b_a_firing_victim_starts_guard_at_once`; fails with the r21 order |
| 9 and MC4 (a): a walking infantry victim under *attack*, guard current, nothing queued | tested | `claims_s0601::claim_9_mc4_a_a_victim_walking_to_attack_starts_guard_at_once`; fails with the r21 order |
| 9: an infantry victim moving under *move*, guard queued, started later | tested | `claims_s0601::claim_9_a_victim_walking_under_move_keeps_guard_queued` |
| 9: the attack state with no target holds the start back | tested | `claims_s0601::claim_9_the_attack_state_with_no_target_keeps_guard_queued` |
| 9: the request for *stand*, *prone* or *deployed* at the clear | partly | made by `infantry::target_changed` (intake 107's tests); the release tests check only that the state ended |
| 9: a simple-deployer vehicle in *unload* skips the request | tested | `claims_s0601::claim_6_simple_deployer_in_unload_keeps_its_behaviour`, unchanged |
| 9: S-0309 claim 30's listed causes, through the same function | partly | `claims_s0309_r120::listed_causes_request_guard_the_takeover_drops_only` and `a_trigger_owner_change_drops_target_and_destination` still state the spec, but they accept guard current or queued and so cannot tell the two orders apart; no test pins r23's order for a capture, a trigger or an occupation |
| S-0111 r49 20: in the route mode a held right button blocks | tested | `client::name_tip_tests::claim_20_route_mode_an_ordinary_press_or_the_right_button_blocks` |
| 20: an ordinary left click blocks | tested | the same test |
| 20: a kind refusal at the press does not block; the tip shows after 1000 ms of rest with the button held | tested | `claim_20_route_mode_a_press_refused_or_taking_a_point_does_not_block` |
| 20: a refusal by the point-click test does not block | tested | the same test |
| 20: a press that takes a point does not block | tested | the same test |
| 20: with nothing selected, a press that takes an own point does not block | tested | the same test |
| 20: a join or a loop close | partly | the same `Press::Take` path; no test of its own |
| 20: a press refused only at its release (route state, holder limit) blocks | partly | `routes::press` gives `Ordinary` at the press; no test of its own |
| 20: the block ends at the release, in the route mode too | tested | both route-mode tests (no tip until the pointer moves and rests again) |
| 20: the Open question (a block kept after an ordinary click released over a point) | no change | not built: our block always ends at the release, as the claim states |

## Records

S-0601 r23 reaches the M-0001 and M-0020 rows whose games put units under mind control (MIND and
YURI pairings), and M-0021 through any release. Arena `--record all --threads 2`, main 841f16d
(before) against this branch on 841f16d (after):

| Record | Row | Before | After |
|---|---|---|---|
| M-0001 | MIND pairings (the MIND rows of the table, against every column) | as main | **unchanged**: `ours-M-0001.csv` is byte-identical, every game's winner and end the same |
| M-0020 | Part 1: in all four YURI games every unit under mind control at the first such tick is of the opposing group and owned by the YURI's house | holds | **holds** |
| M-0020 | Part 2: YURI against GGI and SHK, every arm (12 of 12 to side A) | holds but arm S against SHK | **the same**: arm S against SHK still fails (11 to 1), the other eleven arms hold |
| M-0021 | Every row | 8 of 50 fail | **the same 8**, word for word (`oracle-M-0021.txt` identical) |

Every arena output is byte-identical before and after. M-0001 has 11 of 96 failing, M-0002 2 of
24, M-0003 1 of 35, M-0019 0 of 6, M-0020 1 of 25 and M-0021 8 of 50. No arena game has a release
whose readiness differs between the two orders.

## Goldens

No golden changes. Regenerated on the rebased tree (main 6b78a92 with this branch) with
`GLASNOST_UPDATE_GOLDEN=1 cargo test --release -p glasnost-sim-gate -- --include-ignored`: every
test passes and no file under `crates/sim-gate/golden/` changes, because no golden game has a
release whose readiness differs between the two orders. The formats still move, because the order
changes what a saved game of another game replays to (the quick lane below shows two such
games): save 111 to 112 and AI game 76 to 77, one past main 6b78a92 (intake 110 took 111 and
76). Wire format unchanged.

## Lanes

Before: binaries built on main 841f16d. After: this branch's code on 841f16d (commit afe0e05,
before the rebase). The sweeps, the arena and the quick lane use those two bases. The branch was
then rebased onto main 6b78a92, which merges intake 110 (S-0208 r52, S-0309 r120, S-0508 r13,
S-0506 r34; its S-0309 claim 30 causes now call `mind::release_orders`). On that base the AI
suite, the goldens, the render lane and the final checks were run again. The sweep, arena and
quick-lane figures predate intake 110, so no lane covers its causes taking the r23 order; only the
workspace and AI tests on the rebased tree do. Local lane; install values are not quoted.

- **Sweeps, seeds 1 to 16.** Five of the six runs are identical before and after, apart from
  timings. M7a with the play fixes changes one counter of one seed: seed 13's berserk weapon
  orders go from 78 to 79. Its smallest counter and its minimum are unchanged.
  - M6c reference: smallest minimum 0. M6c play fixes: 0.
  - M7a reference: 1. M7a play fixes: 2.
  - M7b reference and play fixes: drain minimum 3.
- **Arena, `--record all --threads 2`.** Every output file is byte-identical before and after (the
  `ours-*.csv` tables and every `oracle-*.txt`): M-0001 11 of 96 failing, M-0002 2 of 24, M-0003
  1 of 35, M-0019 0 of 6, M-0020 1 of 25 (YURI against SHK, arm S) and M-0021 8 of 50.
- **`cargo test --release -p glasnost-ai -- --include-ignored`** (`--no-fail-fast`). Before, on
  841f16d: 33 binaries, 263 pass and 0 fail. After, on 841f16d: 33 binaries, 263 pass and 0 fail.
  On the rebased tree (6b78a92 with this branch): 33 binaries, 263 pass and 0 fail
  (`plays_to_the_last_side` included). Main 6b78a92 alone was not run here.
- **AI quick lane, seeds 7 to 10, maps 0, 2 and 8** (`GLASNOST_AI_QUICK=1`).
  - 2 of 36 games change, both at seed 8, and **no winner changes**. All 36 games end before and
    after, and house 0 wins 24 of them both times.
    - Seed 8, map 2, sides 1 and 2: same winner (house 1) and end (60873), with different detail
      (house 0 gives one more bomb order).
    - Seed 8, map 8, sides 0 and 2: the same winner (house 1) at 49320 in place of 51434. A
      dominator launch at tick 45210 takes a different object.
  - Seeds 7, 9 and 10 are identical line for line, apart from timings.
  - **Intake 107's changed winner (seed 9, map 0, sides 1 and 2)** does not change. On main
    841f16d it ends at 76670, house 0 winning, before and after, and the game's whole report is
    identical. It no longer passes through the release that intake 107 traced. A run of that game
    alone (`GLASNOST_AI_SIDES=1,2`, `GLASNOST_AI_WATCH=8993,39440,39470`,
    `GLASNOST_AI_DUMP=39451,39452,39453`, before binary) has no object 8993 in those ticks: the
    watch prints nothing and the dumps do not list it. The game has moved on since intake 107
    (intakes 108 and 109 came between), so the one-tick gap that intake 107 saw is not in it.
- **Render lane** (`r8_render_lane.rs`, `i111_the_name_tip_with_a_button_held_in_the_route_mode`,
  local only). The route-mode key is held throughout, and each scene presses the left button,
  moves onto a unit and rests 1200 ms:
  - 1, a kind refusal (a building selected);
  - 2, a press that takes another unit's route point;
  - 3, an ordinary click on open ground with a unit selected.

  Before (main 841f16d): no tip in any scene. After (rebased tree): the tip shows in scenes 1 and
  2 while the button is held, none shows in scene 3, and none shows right after any release.
  Frames are in `local-lane/m8/i111/render/before/` and `after/`.

## Seams and open points

- S-0111 claim 20's Open question (an ordinary route-mode click released over a highlighted or
  refused point) is not built, as above.
- S-0601 MC4 parts (a) and (b) need fire frames for the firing kind. The synthetic WALKER has
  none, so part (b) sets the attack state by hand, as `claims_s0301::attacking` does.
