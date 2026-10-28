# S-0417 r64: claim ledger, M8 step A3 intake 35

This intake takes in one spec revision. It answers Q-1850 and the walk-out part of Q-1757 (play
note 7, [a4-play-notes.md](a4-play-notes.md)): an infantry unit released with no rally point no
longer keeps its factory's contact place for good, so later units come out.

- **S-0417 revision 64** (spec commit 25cbcd1). The code was built at r20 and r21 (r21 changed
  only the scenarios' preamble). The whole r21 to r64 diff was taken in:
  - Summary and *Paired hand-offs*: the Q-1757 and Q-1850 answers; wording.
  - Claim 10: the no-jump retention of a teleporting unit needs a refused request, one with no
    destination or one it never jumps to; a no-rally walking or teleporting unit in *guard* is
    sent off by claim 17; a cell of the foundation rectangle that the foundation word leaves out
    does not hold K; a rally point set after the release does not move a resting unit.
  - Claim 12: a no-rally unit's scatter draw belongs to its first turn, in the next tick.
  - Claim 14: a later release is busy only before the earlier unit's first arrival; one line's
    items are at least 54 ticks apart (S-0404 claim 7, S-0403 F7); rated medium.
  - Claim 15: Open question 3 is now about jumpjet infantry only.
  - Claim 16: a factory-built chrono miner installs temporary driving again on every later
    destination that is not a qualified docking assignment (S-0207 claim 28).
  - Claim 17 (new): the scatter request to resting infantry on a building cell.
  - Import mapping (contact retention row), scenarios U10 b, e, f and U13 a, Open questions 3
    and 10 to 15, Changes for other specs (S-0208 claims 11, 12, 13, 23; S-0404 claim 13 and P12
    part f; S-0215 claims 11, 24, 26).

No new quirk switch. The cross-specs it names were read as they stand: S-0208 claims 11 to 13,
19, 23, 24 and Terms, S-0212 claims 6, 23, 25, S-0209 claim 4 (rows WP2, WP7, WP8, WP10, WP11),
S-0215 claims 11, 24, 26, S-0203 claims 2, 7, 10, S-0404 claims 7, 13, S-0513 claims 48, 51,
S-0514 claim 17, S-0207 claims 19, 20 d, 22. None was revised for this intake. S-0207 r82 and
S-0212 r82 are on main and belong to other intakes; not taken in here.

**Code.**
- `scatter::resting_turn` (sim), called at the end of an infantry unit's turn in
  `tick::object_turn` (after the late start, the behaviour processing, the movement and the
  infantry firing and sequence rules). A unit on the map, not inside and not riding, whose
  current behaviour is *guard* or *area guard* and whose cell (under its position, ground
  layer, any height) lists a building that counts, is sent a forced, move-anywhere request with
  no threat point. The turns that end early (being erased, a warp's leaving stage and earlier
  boundaries) never get there.
- A building counts unless `occupancy::building_gives_none` (rows WP7, WP11, WP10, now shared
  with the walker's entry test, same order, same outcomes), it is a laser-fence section, or it
  is the unit's master and the cell is that master's unloading cell (WP2).
- `scatter::request_resting`: S-0208 claims 11 and 13 as before, but a destination from the
  nearby-cell search leaves the behaviour as it was (no *move* queued); a fall-back neighbour
  queues *move*. A teleporting unit given a search destination gets `teleport::tick` at once and
  jumps in that turn. Every other scatter caller is unchanged (`scatter::request`).
- Comments only: `econ::release` (module header, the no-rally arm), the AI's rally-point rule
  (`scripted.rs` 1b; the AI still sets a rally point, now as a design choice, so its infantry
  gather toward the goal).
- No new saved or hashed state: save format 82, AI game format 44, unchanged.

**Design.**
- The request's place in the turn is the spec's own design rule (claim 17, Open question 11).
- Laser-fence sections: our engine models no section states, so every section is taken to be in
  the down, no-beam states and never counts (claim 17's design, Open question 14), as for
  S-0613 claim 10 and S-0317 claim 19. The walker's entry test has no WP8 row yet, so an
  infantry unit cannot rest on a section anyway.
- A walking unit that holds a destination with no path (its path failed at the reachability
  gate, or a fresh search waits for the retry delay): claim 17 leaves open whether it reports
  movement in progress. For claim 17's request ours does (`scatter::answer`, Q-1980), so each
  later request is refused with no draw and the destination is kept, as for a fall-back
  destination whose *move* is not yet current. A first build accepted it again: in the M7a fix
  sweep, seed 1, a slave stuck on a bio-reactor cell (below) took a draw and a new destination
  in every tick, and the new destination never got searched because the retry delay was still
  running. Every other S-0208 claim 11 caller is unchanged.
- Jumpjet infantry released with no rally point (Open question 3): our engine rests it in
  *guard* like the others, so the rule applies by its conditions. Its ascent report (claim 15)
  ends contact first in practice.

**Seam.** Claim 16's chrono-miner paragraph: a later destination that is not a qualified docking
assignment does not install temporary driving again. That is S-0207 r82 claim 28, which a later
intake takes in.

| Claim or item | Status | Notes |
|---|---|---|
| 10 (no rally: no destination at the release, rests on K in contact) | tested | `claim_10_no_rally_no_destination` still passes; `claim_17_first_turn_request` (no request at the release). |
| 10 (a rally point set later does not move a resting unit) | tested | `claim_10_later_rally_point`. |
| 10 (foundation word cells that do not hold K) | by construction | The cell test reads the cell's ground list, which only the foundation word's cells hold. |
| 10 (teleporting no-jump retention) | by construction | Contact ends only at a cell-arrival update; unchanged. `unmeasured` in the spec. |
| 12 (the draw belongs to the first turn, next tick) | tested | `claim_17_first_turn_request` (the answer's tick), `claim_17_one_draw_then_refusals` (one ranged draw). |
| 14 (busy only before the first arrival; 54 ticks) | tested | `claim_17_walk_off_frees_k` (busy before, released after), `claim_17_unit_after_unit` (releases 54 or more ticks apart). `claim_13_infantry_exit_list` and `claim_14_infantry_contact_places` (busy with no ticks between) still pass. |
| 15 (Open question 3 for jumpjets only) | text | No rule change; `claims_s0417_r19.rs` passes. |
| 16 (chrono miner re-installs driving) | seam | Above. |
| 17 (conditions: in play on the map, guard or area guard, a building that counts) | tested | `claim_17_behaviours` (guard and area guard sent, sleep not), `claim_17_buildings_that_count`. |
| 17 (exempt buildings WP7, WP8, WP10, WP11) | tested | `claim_17_buildings_that_count` (invisible, laser fence, firestorm wall, open gate; a plain building and a closed gate count). |
| 17 (WP2: enslaved unit at its master's unloading cell) | tested | `claim_17_enslaved_unit_at_its_master`. |
| 17 (forced, move-anywhere, no threat point; accepted at rest) | tested | `claim_17_first_turn_request`. The deploy-kind and fraidycat cases of S-0208 claim 11: by construction (`scatter::answer`, S-0208's tests). |
| 17 (the place in the turn; not while erased, warp, tunnel) | by construction | The call is the last step of the infantry turn; the turns that end early return before it. No tunnel is built. |
| 17 (*During the walk-off*: guard kept, later requests refused with no draw, destination kept) | tested | `claim_17_first_turn_request`, `claim_17_one_draw_then_refusals`, `claim_17_walk_off_frees_k` (one accepted request over the walk). |
| 17 (fall-back neighbours queue move) | by construction | `scatter::answer`; no setup where the search finds nothing (spec: `unmeasured`). |
| 17 (teleporting unit jumps in the request's turn) | tested | `claim_17_teleporting_unit_jumps_in_the_request_turn` (it jumps and reports clear in that tick). |
| 17 (*Unit after unit*, Q-1850) | tested | `claim_17_unit_after_unit` (five built by production orders, all released, no refund, five different cells, none on K). Lane `barracks-no-rally`. |
| 17 (a destination with no path: the gate failed, or the retry delay runs) | design, Q-1980 | `claim_17_destination_without_a_path_is_kept` (later requests refused, the destination kept, then it walks off). |
| 17 (jumpjet infantry) | design | Above. |
| Import mapping (contact retention row) | text | No import change. |
| Scenarios U10 b, e, f, U13 a | not ours | Oracle scenarios. |
| Open questions 3, 10 to 15 | design | As above, or no state of ours depends on them. |
| Changes for other specs (S-0208, S-0404, S-0215) | named, left | Text for other specs. Built here: claim 17's request and its walk-off rule. |

**Tests changed.** Claim 17 now sends guard infantry off a building cell, so two tests that had
put an infantry unit on a building cell in *guard* to test something else now put it in *sleep*:
`claims_s0606.rs` (`claim_6_chosen_only_both_positions`, `claim_6_far_point_both_positions`)
and `claims_s0513_storm.rs` (`building_strike_debris`). What they check is unchanged.

**Goldens, arena, sweeps.** Before the rebase no golden moved. After the rebase onto main
21651b9 (intake 34 in), `duel-m6c` moves from its line 746 (tick 7460 to 7470): at tick 7467 a
garrison infantry in *guard* stands on a cell of a 2x2 hut of house 2. Claim 17's request is
accepted once and the unit walks to the next cell, after which each request is refused while it
walks. The goldens are regenerated; no other golden moved (`cargo xtask gate` ok, native and
wasm32). Main against this branch, after the rebase: arena M-0001 30 of 96 pairings failing,
M-0002 13 of 24, M-0003 2 of 35, before and after; the arena folders are identical. Seed sweeps
1 to 16, reference and with `GLASNOST_PLAY_FIXES=1`, all exit 0. M7a and M7b are identical.
M6c (both, the same changes) differs: jumpjet orders median 51 to 52 (max 77 to 132), jumpjets
built and flown median 9 to 10, deploy orders min 62 to 65 and median 106 to 102, sabotage
orders median 17 to 15, crates picked up median 17 to 16, radiation exits max 54 to 94. The
minimum and median of every counter are in `local-lane/m8/a3-35/sweep-tables-rebased.md`.

Before the rebase (main a2584dd), M7a also differed (reference seeds 1 and 2, play fixes seed
1), with M6c reference jumpjet orders median 47 to 53 and M7a play fixes seed 1's infantry
absorbed 6 to 2 (`sweep-tables.md`). The tracing below was done on that base and was not
repeated after the rebase. Each change traced to claim 17 firing where the spec says it should,
after which the game goes its own way:
- M6c, reference, seeds 1 to 16 traced: in 11 seeds, one to four units met claim 17's
  conditions, mostly for 1 to 30 ticks. Most are jumpjet infantry in *guard* over a pillbox, a
  prism tower or a yard, at 300 to 470 leptons (claim 17 ignores the unit's height). The others
  are infantry standing on a building cell of their own or the neutral house. One walker in seed
  5 came back to a pillbox's cell over 2 500 ticks and was sent off each time. The M6c play
  fixes sweep was not traced; its changes are of the same size.
- M7a, reference, seed 1: an enemy walker in *guard* on a bio reactor's cell is sent off at
  tick 7856. Seed 2: one slave on a slave-miner building's cell takes a request about once in
  every ten of its turns over 1 900 ticks.
- M7a, play fixes, seed 1: slave 797 (below) is sent a request on a bio reactor's cell. Under
  the first build it was accepted in every tick from 7706 on (4 270 draws). Under the Q-1980
  design it is accepted 430 times, once in each of its own retry cycles, and refused 3 843
  times. Against the first build the design also changed M6c reference and M7a reference seed
  2, where units held a destination with no path; M6c play fixes and M7b did not change.

**Found, not ours to fix.** In M7a play fixes seed 1 on main a2584dd, slave 797 walks onto its
own house's bio reactor's cell under an entry order (S-0605) and stays there in *guard* from
about tick 7740 to the end of the game. Its search back to its master's unloading cell (S-0402
claim 14) fails in every retry cycle. The same happens with claim 17 switched off, so it
predates this intake. It is a play-fix candidate: the AI gives an absorber entry order to an
enslaved unit, and the slave does not get out of the building's cells. Not checked again after
the rebase, where M7a no longer differs.

**Local lane.** Scene `barracks-no-rally` (`m7b_lane.rs`): the first infantry factory of the
local house (a 3x2 barracks), no rally point, five releases 60 ticks apart. All five were
released at the same contact cell; each unit left it 29 to 50 ticks after its release, so
every later release found the place free. They rest on five different cells next to the
barracks, none on it, in *guard*, out of contact. Screens 001, 030, 120, 300 and end.
The same run after the Q-1980 design gives the same output (`lane2/`).

Quick skirmish (`glasnost-headless ai-sides 120000 7 <map>` with `GLASNOST_AI_QUICK=1`; maps 8,
2 and 0; Yuri as house 1 against Allied and against Soviet; reference and with the play-fixes
layer; 12 games, `local-lane/m8/a3-35/quick/`): every game is over with a winner, between 26 199
and 65 571 ticks, and none panicked.

**Requests.** Q-1980 (claim 17's request to a walking unit that holds a destination with no
path), with the interim above. Q-1981 to Q-1984 unused.
