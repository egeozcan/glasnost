# S-0501 r55: claim ledger, M8 step A3 intake 23

This intake takes in one spec revision. It answers one request, so one interim is retired:
Q-1875 (where the object-present mark of an aircraft released above its pad is, and when it is
cleared).

- **S-0501 revision 55** (spec commit 9cb7800; Q-1875 answered in fca3ff2). It covers r53 to r55
  against r52, which intake 17 built.
  - One behaviour is new: claim 48 (e), the object-present flag of an aircraft released by a
    factory or in the low release. From the release tick on it holds the flag in no cell while it
    is above height 0. The ground record of the docking position's cell ends the release with no
    flag, even if another object had set it. At the end of a movement update that leaves it at
    height 0 (its touchdown, or the low release) it holds the flag in the cell that then holds it,
    and in no other.
  - The Edge cases row of `quirks.aircraft_released_above_pad` now says that with the switch off
    the aircraft holds the flag in its cell.
  - AC30 part k gains (6), the flag recorder, and (7), the low release on a second pad.
  - The rest is wording: the Overview drops the per-revision history and points to claim 48 (e),
    claim 47's backing notes that AC30 k (7) sets up the low release, and F7 says "this spec's"
    for "this revision's".

The switch was already consulted (intake 17). Its doc comment in `quirks.rs` and the ruleset
schema now name the flag held with the switch off, and its `// spec:` line cites claim 48 (e).

**Code** (`crates/deterministic/sim/src/air.rs`).
- `released_docked`:
  - Released above the terrain with the switch on, it sets no flag. The `clear_mark` before the
    move still takes its ground placement's flag. The ground record of the docking position's
    cell is then cleared outright (`others = 0`). Our record counts writers, but S-0209 claim 1
    says the flags are not counts, and claim 48 (e) says the record ends clear whatever another
    object had contributed.
  - At or below the terrain, or with the switch off, it still sets the flag in its cell.
- `tick`: before `tick_flight` it notes whether the aircraft is `released`. After the update's
  `sync_listing`, `released_mark` brings the flag in line:
  - above height 0, no flag;
  - at height 0, the flag in the cell that then holds it, with the record claim 44 chooses (the
    ground record in the low release). A mark in another cell is cleared first.

  This runs at the true end of the update, after every early return of `tick_flight`
  (touchdown included). It covers the update in which (b)'s third case or a landing ends the
  released state.
- The Q-1875 interim (the mark kept on the docking cell and cleared at the first update toward a
  destination) is gone from `released_docked` and `tick_flight`.
- `landing_cell_ok` (F7): a cell whose first listed occupant of the four kinds is the lander
  itself is accepted, as F7's second case states. Before, our search skipped the lander, so a
  cell holding only the lander and its own flag was refused. The low release reaches this at
  touchdown, being listed at height 0 with its flag. Claim 40's initial test and replacement
  search now also accept the cell a landed aircraft stands in. No gate scenario changes.
- A touchdown refused under claim 7 clears the flag as take-off restarts. Only a low release
  that carried its flag into its landing has one to clear; for any other lander this does
  nothing.

**Tests** (`crates/deterministic/sim/tests/claims_s0501_released.rs`).
- New synthetic pads:
  - TENPAD (0, 0, 10): AC30 k (7)'s second pad.
  - MIDPAD (128, 128, 10): docks above a cell's middle, so that a destination straight ahead is a
    cell whose centre it passes over.
- New synthetic aircraft DRIFT: SCOUT with a slowdown distance of 2000 (target fraction below 1
  near the destination; not pad-bound, so the trigger lands it).
- The recorder reads the object-present flag of both records (`others`) and both occupant lists.
  It covers a square of 7 by 7 cells around the pad, which takes in the foundation and the cells
  next to it, plus every cell the aircraft is in.

**Goldens, arena and sweeps.**
- *Before* is main a7478bb; *after* is this branch. Outputs in
  `local-lane/m8/a3-23/before/` and `after/`.
- Goldens: no change. No gate pad docks above the terrain, so no release is raised, and only a
  raised release takes the new paths.
- Oracle arena (local lane): the CSVs are byte-identical before and after.
  - M-0001: 31 of 96 pairings failing; 151 of 172 unanimous stock outcomes reproduced.
  - M-0002: 13 of 24 failing; 43 of 47 reproduced.
  - M-0003: 2 of 35 failing; 61 of 64 reproduced.
- M6c, M7a and M7b sweeps 1 to 16: every seed's counters are identical before and after. The
  smallest minimum is 2 for M6c (jumpjet orders, charger orders), 1 for M7a (psychic sensor and
  takeover orders), and M7b's smallest drain minimum is 3.

| Claim or item | Status | Notes |
|---|---|---|
| Front matter, revision 55 and Q-1875 in `requests` | followed | |
| Overview, "from the release tick on it holds the object-present flag in no cell, and none while above height 0" | built, tested (new) | As claim 48 (e) below. |
| Overview, ratings "(2) to (6)" and the per-revision history dropped | followed | Wording; no behaviour. |
| Claim 47's backing, the low release's tracking start noted as set up but not recorded by AC30 k (7) | followed | Wording; tracking is unchanged (intake 17's `claim_48_low_release`). |
| Claim 48 (e), scope: the factory release and the low release; claim 44 governs rest and take-off and chooses the record | built | `released_mark` runs only for an aircraft that was `released` at the start of the update; `set_mark` and `clear_mark` choose the record by claim 44's height test. |
| Claim 48 (e), *After the release*: no flag in either record of any cell from the release tick on, while it rests; listed in no cell layer | built, tested (new) | `released_docked` sets none. `claim_48_e_no_flag_from_the_release_on` (AC30 k (6)): every recorded cell clear, and the aircraft in neither list, on every tick before the release, on the release tick, for 20 ticks at rest, at the order and through the first three movement updates of a far move order. It fails on the interim, which kept the mark on the docking cell. |
| Claim 48 (e), *After the release*: the release leaves the flag clear before any order, so no later event clears one (Q-1875 question 2) | followed | The interim's clear at the first update is removed. |
| Claim 48 (e), *The docking cell's ground record*: no flag after a release, even if another object had contributed one (S-0209 claim 1) | built, tested (new) | `released_docked` sets `others = 0` there. `claim_48_e_release_clears_the_docking_cells_ground_flag`: a contribution written into that record before the build stands for another object's. It is still set on every tick before the release, and clear after it. The spec rates this `unmeasured`. |
| Claim 48 (e), *Leaving*: no flag while above height 0, resting or leaving; the next flag at the end of an update that leaves it at height 0, in the cell that then holds it (its touchdown, or the low release) | built, tested (new) | `released_mark`. Far: the k (6) test above. Touchdown: `claim_48_e_low_release_at_height_0_carries_its_flag` ends with the flag in the touchdown cell only. After a far release, the touchdown long after is claim 44's `touchdown`. |
| Claim 48 (e), *Low release (c)*, first update: at height 0, in its cell's ground layer, its ground flag set | built, tested (new) | `claim_48_e_low_release_climbs_and_clears` (AC30 k (7) (a), TENPAD, JET, far order): flag in its cell and in no other recorded cell. |
| Claim 48 (e), *Low release (c)*, a climb in the second update (target fraction 1): no flag and no listing anywhere at its end | built, tested (new) | Same test: height positive, every recorded cell clear, unlisted, `mark` none. |
| Claim 48 (e), *Low release (c)*, staying at height 0 (target fraction below 1): the flag in the ground record of its current cell at the end of each update, in no other cell; a cell it has left is clear (Q-1875 question 3) | built, tested (new) | `claim_48_e_low_release_at_height_0_carries_its_flag` (AC30 k (7) (b), MIDPAD, DRIFT, a destination 640 leptons straight ahead): at every update until its landing starts, height 0, listed in its cell's ground layer, the flag there and in no other recorded cell; it crosses a cell boundary at height 0. `claim_48_e_low_release_off_its_heading_goes_straight_on` carries the flag across cells as well and clears it at the climb. |
| Claim 48 (e), with the switch off: at the landing height, listed in the ground layer of that cell, holding its flag there | built, tested (new) | Unchanged code (`set_mark` after the settle). `claim_48_quirk_off_settles_at_the_landing_height` now checks the flag in the docking cell's ground record and in no other recorded cell, at the release and after 20 ticks. It also checks that the flag is cleared after the take-off (claim 44). |
| Claim 48, a release at or below the terrain (docking height 0, switch on) | followed | Outside claim 48: it rests on its pad by claim 44, with its flag. `claim_8_take_off_test_reads_height` (FLATPAD) now checks the flag in its cell only. |
| Claim 48 (c), "goes on at height 0 … until claim 6's landing trigger"; claim 42 guidance only above the terrain | interim, Q-1905 | Our design: claim 42 as written, so at height 0 nothing turns it. With a destination off its heading it goes straight on, and climbs by (b)'s third case once claim 5 makes its target fraction 1 (`claim_48_e_low_release_off_its_heading_goes_straight_on`). With one straight ahead, the trigger lands it near it. AC30 k (7) (b) gives no direction. |
| F7, the found object may be the lander itself (text unchanged; the wording change "this spec's" is r55's) | built, tested (code change) | `landing_cell_ok` accepts a cell whose first listed occupant is the lander: the low release at touchdown, and claim 40's tests on the cell a landed aircraft stands in. Without it, `claim_48_e_low_release_at_height_0_carries_its_flag` stays landing at height 9 to 10: refused, rising and refused again. |
| Claim 7 and claim 48 (e), a refused touchdown | built | The flag is cleared as take-off restarts (above height 0 it holds none). Only a low release has a flag here; not separately tested. |
| Claim 48 backing and confidence, AC30 k (6) and (7), the docking cell's record `unmeasured` | followed | Ratings only. |
| Edge cases, `quirks.aircraft_released_above_pad`, off: "holding the object-present flag in that cell (claim 48 (e))" | built, tested | Doc comment and schema updated; both positions tested (on: the claim 48 (e) tests; off: `claim_48_quirk_off_settles_at_the_landing_height`). |
| AC30 part k (6) | tested on synthetic data | `claim_48_e_no_flag_from_the_release_on`. The stock pad copy and its recording are the lead's. |
| AC30 part k (7) (a) | tested on synthetic data | `claim_48_e_low_release_climbs_and_clears` (TENPAD, displacement height 10). |
| AC30 part k (7) (b) | tested on synthetic data | `claim_48_e_low_release_at_height_0_carries_its_flag`. It uses MIDPAD and a destination straight ahead (Q-1905), and DRIFT, which is not pad-bound, in place of the armed pad-bound type, so that the trigger lands it rather than sending it back to a pad. |
| Claim 48 (e) of intake 17's ledger, "interim, Q-1875" | retired | Updated in [intake17-S-0501.md](intake17-S-0501.md). |

## Requests

Q-1905 was filed (block Q-1905 to Q-1909). It asks whether the low release turns toward its
destination while it goes on at height 0, since claim 42 guides only above the terrain. Interim
(our design): claim 42 as written. Q-1906 to Q-1909 are unused. Q-1875 was answered, and its
interim is retired.
