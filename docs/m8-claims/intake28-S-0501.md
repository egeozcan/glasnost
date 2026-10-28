# S-0501 r59: claim ledger, M8 step A3 intake 28

This intake takes in one spec revision, which answers one request, so one interim is retired:
Q-1905 (does an aircraft in the low release turn toward its destination while it goes on at
height 0?).

- **S-0501 revision 59** (spec commit 7ee94f4; Q-1905 answered in 0510ab7). It covers r56 to r59
  against r55, which intake 23 built.
  - One part is new: claim 48 (c) *At height 0*. Nothing steers the low release while it is at
    height 0. Guidance's distance bands and claim 6's close-approach trigger do not apply there.
    It lands, or a pad-bound one falls back to rest, only in its destination cell's halving zone.
    Otherwise it goes on past the destination and climbs in the update after the first one that
    leaves a target fraction of 1.
  - The Overview gains Q-1905's answer in full and its ratings. Claim 48's backing and confidence
    name AC30 part k (7) (b) and (c).
  - The Edge cases row of `quirks.aircraft_released_above_pad` says that the same switch governs
    the unsteered run. With it off, the aircraft leaves by an ordinary take-off, which claim 8
    steers. There is no new switch.
  - The Import mapping's `PoseDir` row also names the facing of a release onto a pad, which AC30
    part k (7)'s named test rules set to 64 (map east).
  - AC30 part k (7) is rewritten as three runs, (a), (b) and (c), with voids and falsifiers.
  - Open question 26 is new: a docking position outside the pad's foundation.

The cross-specs it names were read as they stand: S-0404 claim 13 (a facing value *v* is *v* × 256
facing units), S-0204 claim 5 (the table square root distance) and S-0206 claim 1 (turning). None
of them changed.

**Code** (`crates/deterministic/sim/src/air.rs`).
- `low_release_at_height_0` (new): the low release while it stays at height 0, read as the
  released mark (claim 48) together with height 0. All three gates below test it, and only it.
  Claims 5, 6 and 8 apply as written to every other aircraft, at any height (Q-1941 interim).
- `tick_flight`: claim 5's approach target replacement (0 below 128 leptons, 0.5 below 512, 0.75
  below 768) is skipped for the low release at height 0. The spec calls the replacement
  guidance's distance bands, and claim 42 confines guidance to an aircraft above the terrain. For
  the low release, then, the target stays claim 5's ordinary min(1, *d* / *S*). Its
  start-of-update climb test, held height 0 with a target fraction of 1 (claim 48 (b), third
  case), sees that target. Before this change, the 0.75 band held a low release that was 512 to
  767 leptons from its destination at height 0.
- `landing_trigger`:
  - The close-approach trigger (below 128 leptons, current fraction below 0.05) is skipped for the
    low release at height 0, because it is a request of guidance. The exact-cell trigger is claim
    6's own and still applies to it.
  - The no-destination landing, claim 8's "in flight", is skipped for the low release at height 0,
    which is on the ground by Terms. Only the low release after its fall-back to rest reaches this.
    It now stays at height 0 in its cell even if a positive current fraction is left after the
    fall-back. Before, each update would have fallen back again.
  - The pad-bound branch carries the `// design:` comment for Open question 26. Its logic is
    unchanged.
- Nothing else changes. Guidance (`guidance`) already returned at height 0, claim 4's descent
  nudge already needed a descent, and claim 4's edge rule already applied at any height.

**Scope (Q-1941).** Another aircraft can fly at height 0 without landing. Claim 6's
holding-height approach holds, below 768 leptons from the destination, the destination's height
minus the terrain there. For a point accepted with no target (a move order, so not raised) and a
target with ammunition picked up on the way, that held height is 0. The determinism gate's
`duel-m6c` has such an aircraft. A first build that gated on height alone dropped claim 5's
replacement for it, and the golden hash moved at line 51 (a changed target fraction in the
hashed state). *At height 0*'s subject is the low release, and claims 5 and 6 carry no height
condition, so the gates now test the low release only. The reason the spec gives (guidance runs
only above the terrain) would cover that aircraft too, so Q-1941 asks which is meant.

**Design (Decisions of 2026-10-25), Open question 26.** A pad-bound low release from a docking
position outside its pad's foundation keeps its contact (claim 48 (c), *Pad contact*).
- Its destination cell holds no building it is in contact with. In the halving zone it falls back
  to rest by T2's ground row, and T2's after-the-row entry gives it *enter*. Our design: that
  *enter* is claim 24's, as for any aircraft in contact on the ground off its pad. It takes the pad
  as its destination and takes off toward it.
- Its destination cell is a cell of the pad it is in contact with. Claim 6's pad-bound restriction
  does not hold there. Our design: claim 6 as written. It starts to land (claim 7), and its
  touchdown in a cell of its contact pad docks it there, outside *enter* (claim 24).
- No request was filed for these: both follow from claims 6, 7, 24 and T2 with no new rule.
- Where the contact is held: only for a destination given without a person's order (in the tests,
  `assign`, the economy's destination assignment). A person's move order ends the contact of a
  pad-bound type wherever its docking position lies (S-0213 T3 (b), since S-0417 claim 11 leaves
  it untethered). That contradicts claim 48 (c)'s "still holds its contact", so Q-1940 asks; its
  interim is T3 (b) as written.

**Tests** (`crates/deterministic/sim/tests/claims_s0501_released.rs`).
- `rules_k7`: AC30 part k (7)'s named test rules, with the pose direction 64, so that every release
  faces 16384 (map east). The low-release runner asserts that facing, as the scenario's void.
- New synthetic pads:
  - SIDEPAD (128, 28, 10) docks 100 leptons beside a foundation cell's middle row.
  - OUTPAD (384, 128, 10) docks in the middle of the cell east of the foundation.
  - WESTPAD (−640, 128, 10) docks two cells west of the foundation, so that the cell straight ahead
    of it at 512 leptons is a foundation cell.
- New synthetic type NEAR6: JET with a slowdown distance of 600.
- The recorder (`Up`) adds, per update, the horizontal position, body facing, speed fractions,
  behaviour, destination and contact. It also checks that every cell the aircraft is in is usable,
  as AC30 part k (7)'s void at the map's edge.
- Run (b) uses MIDPAD (128, 128, 10) rather than a copy of the first pad with a horizontal
  displacement of 0. MIDPAD's docking position is a cell's centre, so a destination centre lies on
  the line ahead. With (0, 0, 10) the docking position is a cell corner, and every straight-ahead
  centre lies 128 leptons off the line, outside the 20-lepton allowance. Its docking position is
  still in a foundation cell, as part k requires.

**Goldens, arena and sweeps.** Before is main 0510ab7, after is this branch. Outputs are in the
local-only `local-lane/m8/a3-28/before/` and `after/`.
- The determinism gate's goldens are unchanged, and the gate with its ignored sweeps passes. A
  first build gated the claim 5 replacement on height alone. It moved the `duel-m6c` golden: an
  attacker at height 0 from claim 6's holding approach lost its band value. Gating on the low
  release (Q-1941 interim) restored it.
- Arena CSVs ours-M-0001, M-0002 and M-0003 are byte-identical before and after: M-0001 31 of 96
  failing and 151 of 172 reproduced, M-0002 13 of 24 and 43 of 47, M-0003 2 of 35 and 61 of 64.
- The M6c, M7a and M7b sweeps over seeds 1 to 16 with `GLASNOST_PLAY_FIXES=1` exit 0. Their seed
  lines and minimum and median counters are identical before and after.
- Save format 79 and AI game format 41 are unchanged: the build adds no state.

**Local lane.** New scenes `low-release-ahead` and `low-release-abeam` in
`crates/app/tests/m6c_lane.rs`. A local rules copy puts every docking position of the loaded
pad-bound aircraft's pad 10 leptons up at a cell's centre, and sets the aircraft's slowdown
distance to 2000 (AC30 part k (7)'s NEAR). A person's move order then goes to a cell three cells
from the docking cell, outside the foundation and inside *S*. Outputs are in
`local-lane/m8/a3-28/lane/` (`run.txt`, per-update traces and screens).
- Ahead: at height 0 from the first update, facing unchanged, the speed falling with the distance.
  It falls back to rest in *guard* in the destination cell and stays there at height 0 for
  30 updates.
- Abeam: claim 4's 5-lepton descent nudge toward the destination in the first update, then a run
  along the released facing at height 0, unturned, for about 100 updates. It climbs once the
  distance is about 2000 and turns back toward the destination.
- An earlier run with destinations one cell away showed Q-1942: those cells are in the pad's
  foundation, F7 refuses them, and claim 40's replacement at the move's first processing makes the
  aircraft at height 0 take off.

| Claim or item | Status | Notes |
|---|---|---|
| Front matter, revision 59, Q-1905 in `requests` | followed | |
| Overview, Q-1905 in full and its ratings | followed | Ratings; the behaviour is claim 48 (c) below. |
| Claim 48 heading (Q-1841, Q-1905); (c) "goes on at height 0 … as *At height 0* below states" | followed | Wording. |
| Claim 48 (c) *At height 0*: nothing steers it (claim 42 guidance only above the terrain; claim 8 only in a take-off; a landing's steering only once it has started; claim 4's nudge only above its held height, with no turn); its body keeps its released facing (S-0404 claim 13) | retired at intake 31 (S-0501 r62: a low release on a move takes off before its second update; see [intake31-S-0501.md](intake31-S-0501.md)) | `guidance` returns at height 0 (unchanged). `unsteered_through` asserts facing 16384 at every update at height 0 in every run below. |
| *At height 0*: each update steps it straight along that facing by claim 4, with claim 5's fractions from its distance to the flight point, the cell's centre for a move order | retired at intake 31 | `claim_48_c_low_release_at_a_right_angle_goes_past_then_climbs` asserts x growing at every update before the climb. |
| *At height 0*: guidance's distance bands and close-approach trigger do not apply at height 0, so a destination 512 to 767 leptons away is treated like any other below *S* | retired at intake 31 (Q-1941: claims 5 and 6 gate on height for every aircraft) | The replacement and the close trigger are skipped for the low release at height 0 (`low_release_at_height_0`). `claim_48_c_no_distance_bands_at_height_0` (NEAR6, *S* = 600, 512 leptons abeam): the target fraction reaches 1 at a distance of 600 to 767, and the next update climbs. On the interim the 0.75 band held it until 768: the test fails on the intake 23 code (checked). |
| *Into the halving zone*: a step ends in the zone only within min(86, *S* / 10) of the point, inside the destination cell; target fraction 0 there and claim 6's exact-cell trigger with its planned-route and pad-bound restrictions | retired at intake 31 | Zone membership is read from the recorded positions as *d* < 86 and 10 *d* < *S* (`in_zone`). |
| *Into the halving zone*, not pad-bound: starts to land there (claim 7) | retired at intake 31 | `claim_48_e_low_release_at_height_0_carries_its_flag` (DRIFT, MIDPAD, 512 leptons straight ahead): the landing starts in the first update that ends in the zone, in the destination cell, and no earlier update ends in it. It fails on the intake 23 code (checked). |
| *Into the halving zone*, pad-bound, docking position in the foundation: no contact, falls back to rest by T2's ground row (forgets target and destination, *guard* for a human-controlled house); with full ammunition no pad search; it stays at height 0 in that cell, listed there | retired at intake 31 | `claim_48_c_low_release_straight_ahead_falls_back_in_the_halving_zone` (AC30 k (7) (b)): before the zone, height 0, facing 16384, destination and *move* kept; in the first zone update τ = 0, destination none, *guard*; for 30 updates after it, height 0 in that cell, listed, no landing, no contact, no *enter*, current fraction 0 and the position unchanged. `landing_trigger`'s no-destination arm now skips the low release at height 0. The test fails on the intake 23 code (checked). |
| *Into the halving zone*, pad-bound, docking position outside the foundation: contact held; with no building it is in contact with in the destination cell, it falls back by the ground row, then T2 gives it *enter*; what *enter* does is open (Open question 26) | retired at intake 31 (no low release reaches the zone at height 0; the test is removed) | `open_question_26_outside_the_foundation_falls_back_then_enters` (OUTPAD; the destination given by `assign`, without a person's order, Q-1940): contact held through the run, the fall-back in the zone in the destination cell, then *enter*, which takes the pad as its destination and takes off. It fails on the intake 23 code (checked). |
| *Into the halving zone*, destination cell on the pad it holds contact with: the restriction does not hold; what it does is open (Open question 26) | superseded at intake 31 (the test now takes off first, then lands and docks) | `open_question_26_destination_on_its_pad_lands_and_docks` (WESTPAD; the destination given by `assign`): it starts to land in the zone in that foundation cell, and touches down there at height 0 with its contact kept. |
| *Past the destination*: passing through the destination cell with a positive target fraction starts no landing | retired at intake 31 (r62 drops *Past the destination*) | `claim_48_c_positive_fraction_through_the_destination_cell_starts_no_landing` (DRIFT, SIDEPAD, a path 100 leptons beside the centre): in the destination cell, closer than 128 leptons and never in the zone, with τ > 0; no landing, and it climbs later. The test fails on the intake 23 code, where the 0 band landed it there (checked). |
| *Past the destination*: the first update that ends with the distance at least *S* leaves a target fraction of 1; in the next its held height becomes that of ordinary flight at the start (third case of (b)) and it climbs by F1; from then on guidance turns it (S-0206 claim 1) | retired at intake 31 | `climbs_after_full_fraction`, used by the right-angle, bands, side-pass and off-heading tests: no climb up to the first update with τ = 1, and a climb in the next. The right-angle test also checks that *d* ≥ *S* exactly at that update, and that the facing has left 16384 and is closer to the bearing 80 updates after the climb. |
| *Past the destination*: claim 5's positive-fraction hovers and claim 4's edge rule apply at height 0 as anywhere | followed | No height test in either (unchanged code); not separately tested. |
| *At height 0*: the run is the original's; no switch of its own; `quirks.aircraft_released_above_pad` off gives an ordinary take-off that claim 8 steers | built, tested | Doc comment in `quirks.rs` and the schema text updated. `claim_48_quirk_off_takes_off_toward_a_right_angle_destination`: off, it climbs in its first update and its body turns toward a right-angle destination. `every_spec_quirk_has_a_switch` green. |
| Claim 48 backing and confidence, AC30 k (7) (b) and (c), Open question 26 `unmeasured` | followed | Ratings only. |
| Edge cases row of the switch | built, tested | As above. |
| Import mapping, `PoseDir` also the facing of a release onto a pad, set to 64 in AC30 k (7)'s test rules | followed, tested | Already imported (S-0404, `pose_dir`); `rules_k7` sets it, and the runner asserts the released facing 16384. |
| AC30 part k (7) (a) | tested on synthetic data | `claim_48_e_low_release_climbs_and_clears`, now under `rules_k7`. |
| AC30 part k (7) (b) | superseded at intake 31 (`ac30_k7_b_...`) | `claim_48_c_low_release_straight_ahead_falls_back_in_the_halving_zone` (NEAR, pad-bound, armed, *S* 2000, speed 30, on MIDPAD; see above). |
| AC30 part k (7) (c) | superseded at intake 31 (`ac30_k7_c_...`) | `claim_48_c_low_release_at_a_right_angle_goes_past_then_climbs` (NEAR on MIDPAD, a destination two rows north, 512 leptons abeam). |
| *Pad contact* with *At height 0*, a pad-bound type outside the foundation "still holds its contact", against S-0213 T3 (b) for a person's order | retired at intake 31 (Q-1940 answered by r62: a behaviour order ends the contact wherever it docks, as built) | T3 (b) as written: a person's move order ends the contact of an untethered (pad-bound, S-0417 claim 11) type wherever it docks, so it falls back to rest in *guard* with no *enter*. `claim_48_c_person_order_ends_a_pad_bound_contact_outside_the_foundation` (OUTPAD, a person's move order): contact none in every update after the order, the fall-back in the zone of the destination cell, then 30 updates at height 0 there with no destination and no *enter*. It fails on the intake 23 code (checked). `destination_taken` carries the interim comment. |
| *At height 0*'s exemptions (bands, close trigger) beyond the low release | retired at intake 31 (Q-1941 answered by r62: the gates are on height for every aircraft; the test is renamed `..._has_no_approach_replacement`) | Low release only; claims 5, 6 and 8 as written for any other aircraft. `q1941_ordinary_flight_at_height_0_keeps_the_approach_replacement` (`claims_s0501.rs`): a move order, then a target with ammunition, so claim 6's holding approach brings it to height 0 at about 500 leptons; at every update there at height 0, the target fraction is the band's (0.5, then 0). It fails with the gates on height alone (checked). The `duel-m6c` golden is the second guard. |
| Claim 40's replacement of the low release's move destination at the move's first processing, after it reaches height 0, against claim 48 (c)'s "a take-off, which it does not make" | retired at intake 31 (Q-1942 answered by r62: the move's first processing takes it off) | Claims 40 and 8 as written: the replacement is a destination it gets on the ground, so it takes off. No code change. Seen in the local lane with a destination in the pad's own foundation. |
| Local lane, real install | checked | `low-release-ahead` and `low-release-abeam` (see *Local lane* above). |
| Open question 26 | design | As above; no request for the design itself. |
| Claim 48 (c) row of intake 23's ledger, "interim, Q-1905" | retired | Updated in [intake23-S-0501.md](intake23-S-0501.md). |

## Requests

Three filed (block Q-1940 to Q-1944):
- Q-1940: does a person's move order end the pad contact of a pad-bound aircraft released at a
  docking position outside its pad's foundation (claim 48 (c) against S-0213 T3 (b) and S-0417
  claim 11)? Interim: T3 (b) as written.
- Q-1941: do *At height 0*'s exemptions from guidance's distance bands and close-approach trigger
  apply to any aircraft at height 0, or only to the low release? Interim: the low release only.
- Q-1942: when claim 40 replaces the low release's move destination at the move's first
  processing, after it has reached height 0, does claim 8 make it take off? Interim: claims 40 and
  8 as written (it takes off).

Q-1943 and Q-1944 are unused. Q-1905 was answered, and its interim is retired. Q-1940, Q-1941 and
Q-1942 were answered by S-0501 r62, and their interims are retired at intake 31
([intake31-S-0501.md](intake31-S-0501.md)).
