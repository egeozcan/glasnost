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
| 9: S-0309 claim 30's listed causes, through the same function | tested | `claims_s0309_r120::listed_causes_request_guard_the_takeover_drops_only` and `a_trigger_owner_change_drops_target_and_destination` (guard current or queued; they do not pin the readiness) |
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

GOLDENS-PLACEHOLDER

## Lanes

LANES-PLACEHOLDER

## Seams and open points

- S-0111 claim 20's Open question (an ordinary route-mode click released over a highlighted or
  refused point) is not built, as above.
- S-0601 MC4 parts (a) and (b) need fire frames for the firing kind. The synthetic WALKER has
  none, so part (b) sets the attack state by hand, as `claims_s0301::attacking` does.
