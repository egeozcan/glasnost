# S-0309 r70: claim ledger, M8 step A3 intake 68

S-0309 r53 to r70 (commit c7b7ca1) adds claim 29, the periodic held-target check, and its
Constants row (the held-target check period, 16 ticks), with scenario A6 part p (runs p1 to p6)
and new readings of parts f and m. Claims 4, 19 and 28 gain claim 29's consequences, and the
Open question *An earlier drop* is replaced by six narrower ones. Eight Changes-for-other-specs
entries come with it (S-0304 claim 2, S-0306 claim 23, S-0215 claim 32, S-0516 claim 14, S-0501
take-off, S-0310 claim 11 twice, S-0513 claim 39, and S-0513 claims 48, 51 and 52 with S-0203
claim 4). Intake 52b built r53 ([ledger](intake52b-S-0309-S-0405-M-0005-M-0006-M-0016.md)).
Requests filed: **Q-2261** to **Q-2264**.

## Built

- **The periodic check** (claim 29). `combat::held::periodic_check`, called from
  `tick::object_turn` in every object turn. In a tick whose tick number is a multiple of 16
  (`held::check_tick`, the global tick, so every object checks in the same ticks), a covered
  holder asks `check::check` about its target, without the range test, for the slot
  `weapon::would_use` gives. On *illegal* or *impossible* it drops the target through
  `spawn::clear_own_target` (claim 24's consequences: the spawner's conditional pending-target
  clear and the airstrike cancel) and resets its burst, as S-0304 claim 2's drop did. The drop
  takes no draw, reverts nothing (no `behaviour::revert`) and changes no behaviour; the next
  *attack* processing finds no target and ends the attack.
- **Coverage.** Infantry, vehicles, aircraft and buildings of a player house
  (`acquire::player_owned`, AI houses included, as claim 29 gives for our engine), whose current
  behaviour is *attack* or *guard*. *Capture* and *sabotage* make none. Other behaviours make none
  (interim Q-2262). The neutral and special houses make none (interim Q-2263). The target may
  come from an order or a search; our engine does not record which, and does not record a
  retaliation or revert either (Q-2263).
- **Place in the turn** (interim Q-2261). Right after the warp update, so the warp turns that
  S-0513 claim 51 makes the whole member turn take no check, and in the arrival hold it comes
  after the hold-end test. It comes before the drainer's position check, the behaviour start,
  the behaviour processing, the searches, movement and the firing step, and before a
  building's pending delayed shot counts down (`delayed_shot_turn`), so in the tick that ends
  the delay the check still answers *rearming* and the target is kept (claim 29, S-0301 F21).
- **Turns cut short.** A holder being erased (`temporal::erased`, a lock on it) makes no check;
  an infantry unit, vehicle or landed aircraft being erased already drops its target in each of
  its turns (`temporal::turn`, S-0215 claim 2, S-0514 claim 17). A materialising vehicle or
  aircraft with no beam on it makes the check, and `check::check`'s phasing-in *rearming* keeps
  the target where it is the answer. Tunnel travel, a vehicle's death frames and a
  materialising building do not arise in our engine (check.rs: ranks 1 and 3 need states no
  topic sets; no tunnel is built), so nothing is cut short for them.
- **S-0304 claim 2** (Changes). `process::process_attack` drops an *illegal* target at its first
  check only for an actor that claim 29 does not cover (`held::periodic_covers` false: the
  neutral and special houses, interim Q-2263). A covered vehicle, infantry unit or aircraft now
  drops it at claim 29's check tick. A building in *attack* keeps S-0215 claim 32's drop at its
  own processing (`process::building_attack`), unchanged, so it normally drops there first.
- **S-0310 claim 11** (Changes, infantry healers). `behave::infantry_step` now replies to
  *illegal* in its firing step when the infantry unit's default weapon heals: it drops a target
  that is not infantry or is at full health, and keeps an infantry target below full health. The
  table is the vehicle healer's (`behave::vehicle_step`) with *infantry* in place of *a vehicle*.
  A kept target lasts to claim 29's next check tick, as for the vehicle healer of T10 part d.
- **S-0513 claim 39** (Changes). No code beyond claim 29: a holder of a warp member, whose
  firing check answers *illegal* from the launch to the departure, drops it at its first
  checking turn in a check tick after the launch. The interim keep that intake 60 tested through
  the leaving stage now ends there.
- **Spawner keep** (claim 4). `periodic_check` has no spawner exception, so claim 4's keep of an
  *impossible* target ends at the first check tick. A vehicle launching spawned aircraft gets
  *busy* (rank 2) and keeps its target.

## Design choices

- The check tick is `w.tick % 16 == 0` read in the holder's own turn. An order given between
  steps while `w.tick` is *n* is applied in tick *n* after the turn pass (S-0203 claim 7), so its
  drop comes 1 to 16 ticks later; the tests count from that tick.
- In our engine an attack order queues *attack* behind the current *guard*, which starts it at
  its next processing. In run p1 a check tick can come while *guard* is still current; both
  behaviours check, so the drop tick is the same.
- The behaviour gate reads `beh.current` at the start of the turn, before `process::head` can
  start a queued behaviour (Q-2261 question 3).
- The neutral and special houses keep S-0304 claim 2's attack-processing drop rather than losing
  every drop of an *illegal* force-fire target. That keeps their behaviour as before r70.

## Claims

| claim | status | tests or reason |
|---|---|---|
| 29, Phase: the drop comes in the holder's turn of the first tick whose tick number is a multiple of 16 after the order's tick: 15, 11, 7 and 3 ticks after an order applied at remainders 1, 5, 9 and 13; no shot (part p run p1) | tested | `p1_an_ordered_illegal_target_is_dropped_at_the_first_check_tick_by_a_vehicle`, `p1_an_ordered_illegal_target_is_dropped_at_the_first_check_tick_by_infantry` (`claims_s0309_r70.rs`): three runs per remainder, no drop before the expected tick, no shot |
| 29, The drop: no targeting-timer draw or reset (run p1, timer above 10 ticks) | tested | the p1 tests (timer set to 200 ticks; it counts on through the drop), and `the_check_and_its_drop_take_no_draw` (the stream against a clone with no target) |
| 29, The drop: no behaviour change by itself; the *attack* ends at its next processing | tested | `p1_the_attack_ends_at_the_processing_after_the_drop` (processing held back past the drop: *attack* still current after it; then it ends) |
| 29, The drop: no revert (S-0215 claim 10) | tested | `the_drop_reverts_no_override` (an override in effect; nothing reverts at the drop) |
| 29, Answers: a running rearm countdown (rank 5) keeps a rank-9 refusal to the first check tick after it runs out (run p2) | tested | `p2_a_running_countdown_keeps_an_illegal_target_to_the_first_check_tick_after_it` (a 40-tick countdown, vehicle) |
| 29, Answers: the rank-5 keep for an infantry holder | partly | Built by the same check; no test, as the spec rates it low |
| 29, Answers: the weapon it would use decides; its other weapon's refusal does not drop the target (run p3) | tested | `p3_the_weapon_it_would_use_decides` (100 ticks, kept at every check tick, fires its second weapon) |
| 29, Answers: a rank-4 refusal (an airborne target the projectile cannot hit, S-0306 claim 23) outranks the countdown (run p4) | tested | `p4_a_rank_4_refusal_is_dropped_while_the_countdown_runs` (a cell target in the air with a ground-only projectile, 200-tick countdown) |
| 29, Answers: *impossible* drops the target (run p5) | tested | `p5_an_impossible_answer_drops_the_target_at_the_check_tick` (a weapon slot with no weapon: rank 4 *impossible*). The hidden-target refusal of the run is S-0516 claim 24's, built at its own intake |
| 29, Answers: no range test | by construction | `check(…, false)`; no refusal ranked below range is reachable, as the spec says |
| 29, Answers: a vehicle launching spawned aircraft (*busy*) keeps its target | by construction | `check::check` answers *busy* first for a vehicle with a launching count; not tested |
| 29, Phase: holders made in different ticks check in the same ticks, vehicle and infantry (run p6) | tested | `p6_holders_made_in_different_ticks_check_in_the_same_ticks` (holders made at remainders 5 and 11, ordered at remainder 1; both drop 15 ticks later) |
| 29, a searched (opportunity) target in *guard* is dropped no later than the first check tick after the refusal begins (part m) | tested | `m_a_searched_target_in_guard_is_dropped_at_the_check_tick_before_the_next_search` (vehicle; the next search held off) |
| 29, a searched target by an infantry holder; *guard*'s cadence and shared phase | by construction | The same gate and the same tick test; the spec rates these `unmeasured` |
| 29, Behaviours: *capture* and *sabotage* make no check | tested | `capture_and_sabotage_make_no_check` |
| 29, Behaviours: the others (open) | interim, tested | Q-2262: no check. `other_behaviours_make_no_check_interim` (*hunt*, *area guard*, *sticky*, *move*) |
| 29, holders of the neutral and special houses, and targets from retaliation or a revert (open) | interim, tested | Q-2263: no check for those houses, which keep S-0304 claim 2's attack-processing drop (`a_neutral_holder_keeps_the_attack_processing_drop_interim`); a player house checks whatever gave the target |
| 29, Place in the turn: a building checks before its pending delayed shot counts down, so a delay ending in a check tick keeps the target | tested | `a_pending_delayed_shot_ending_in_a_check_tick_keeps_the_target`, with a control in which the target is dropped at a check tick with no pending shot |
| 29, Place in the turn otherwise; a refusal that begins in the holder's own turn; which behaviour decides (open) | interim | Q-2261: the start of the turn, after the warp update and claim 27's turn-start drop; a refusal begun later in that turn waits for the next check tick; the behaviour current at the turn's start decides. Not tested beyond the placement tests above |
| 29, building holders | tested | `a_building_drops_an_illegal_target_at_the_check_tick` (the building's processing held back; the drop is the periodic one) |
| 29, aircraft holders | built | The same gate covers aircraft; no test gives an aircraft a refused target, as no part does. S-0501's take-off keep (`air.rs`) is a movement rule and the check runs alongside it |
| 29, Turns cut short: a holder being erased makes no check and, if mobile, drops its target each turn | by construction | `temporal::erased` gate; `temporal::turn`'s drop is S-0514 claim 17's, tested at its intake |
| 29, Turns cut short: tunnel travel, the last death-frame turn, the earlier death-frame turns' *impossible*, a building being materialised | seam | No tunnel, vehicle death frames or materialising building in our engine |
| 29, a holder that misses its turn in a check tick acts at its next checking turn | by construction | The check runs only in the holder's own turn; nothing carries over |
| 29, materialising holders (open) | interim | Q-2264: a holder with a beam locked counts as being erased (no check, the per-turn drop); one with no beam makes the check, and the hidden-target *impossible* and the projectile *illegal* come before the phasing-in *rearming* in our firing check, so it drops. Not tested |
| 29, Warp members: no check in the warp turns; in the arrival hold after the hold-end test (S-0513 claim 51, design) | by construction | The call follows `warp::turn`'s early return. Intake 60's warp tests still pass |
| Constants, held-target check period 16 ticks | tested | `held::CHECK_PERIOD`, used by every test above |
| 4 r70, the drop on an unusable target can come earlier at a check tick | tested | through the part m test |
| 4 r70, a spawner's keep of an *impossible* target ends at the first check tick | by construction | No spawner exception in `periodic_check`; no test (the spec rates the keep `unmeasured`) |
| 4 r70 and A6 part f's new reading: a firer that is not a spawner drops an *impossible* target at a search and picks it again in that search (the hidden span with no check tick) | partly | Built at intake 52b and earlier (`acquire`'s claim 4 drop, table C not rejecting on *impossible*), no change. The spec rates the case `unmeasured`, since no setup is shown to put a search in a hidden span with no check tick; no new test |
| 4 and 19 r70, Backing and Confidence wording (the spawner's keep, its controller reset and released missiles now `unmeasured`, medium) | wording only | No code |
| 19 r70, an opportunity target's drop can come at a check tick before the next search | tested | `m_a_searched_target_in_guard_is_dropped_at_the_check_tick_before_the_next_search` |
| 28 r70, a holder kept under an exception can still lose the target at claim 29's check | by construction | The check reads the firing check; the cloak keep is S-0516 claim 14's. Intake 52b's `claim_28_cloak_drops_reset_the_timer` still passes |
| Changes, S-0304 claim 2: a covered actor drops the force-fire *illegal* target at the first check tick after the order's tick (1 to 16 ticks), not at its attack processing; after a countdown, at the first check tick after it | tested | the p1 and p2 tests; `an_attack_on_an_illegal_target_never_fires` and `an_illegal_target_is_kept_while_rearming` (`claims_s0304_s0305.rs`) now expect a check tick |
| Changes, S-0304 claim 2: a building in *attack* also drops at its own processing (S-0215 claim 32), whichever comes first | by construction | `process::building_attack` unchanged; its tests still pass |
| Changes, S-0306 claim 23: a covered unit drops the first row's refused target at claim 29's check | tested | `p4_a_rank_4_refusal_is_dropped_while_the_countdown_runs` |
| Changes, S-0215 claim 32: no change | — | unchanged |
| Changes, S-0516 claim 14 and CS9 part c: a kept cloaking target can be lost at a check tick | by construction | See claim 28 above |
| Changes, S-0501 take-off: the take-off keep has claim 29's exception | by construction | The check is independent of `air.rs`'s keep; not tested |
| Changes, S-0310 claim 11 and T10 part d: a vehicle healer's keep lasts to the next check tick | tested | `a_vehicle_healers_keep_lasts_to_the_check_tick` |
| Changes, S-0310 claim 11 and T10 parts g and h: an infantry healer drops a non-infantry or full-health target in its firing step | tested | `an_infantry_healer_drops_a_vehicle_target_in_its_firing_step` |
| Changes, S-0310 claim 11: an infantry healer keeps a damaged infantry target that answers *illegal*, to the next check tick | tested | `an_infantry_healers_keep_lasts_to_the_check_tick` (a healing warhead with versus 0) |
| Changes, S-0513 claim 39: a holder of a warp member drops it at its first checking turn in a check tick after the launch | tested | `claim39_whole_tick_holder_first_then_the_interim_drop` (`claims_s0513_r43.rs`), updated (below) |
| Changes, S-0513 claims 48, 51 and 52 and S-0203 claim 4: the check is among the work the warp turn suppresses | by construction | see *Warp members* above |

## Tests changed

- `claims_s0304_s0305.rs`: `an_attack_on_an_illegal_target_never_fires` now expects the drop in
  a check tick, and `an_illegal_target_is_kept_while_rearming` expects it at the first check
  tick after the countdown runs out (S-0304 claim 2 as r70 changes it). These are expectation
  updates.
- `claims_s0513_r43.rs`: `claim39_whole_tick_holder_first_then_the_interim_drop`. The infantry
  holder in *guard* used to keep the leaving member through the whole leaving stage. It now drops
  it at the first check tick after the launch, with no draw (S-0513 r62 claim 39). The test then
  gives it the member again after the last check tick before the departure, and the departure
  part runs as before. It asserts that the departure tick is not a check tick in this setup.
- `claims_s0607_release.rs`: `claim_9_no_destination_flies_to_the_found_point`. **This is a
  setup change, not only an expectation update.** The lifter used to drop the airborne victim,
  which its firing check answers *illegal* for, at its attack processing. It now keeps it until
  the check tick (tick 96 in this test). That shifts its fall-back to *area guard* and that
  behaviour's search, which picks the victim again (tick 156) and grips it as it lands (tick
  185), so the victim ends up near the lifter instead of at its found point. That follows from
  claim 29 and the existing search rules. The claim under test is the victim's own jumpjet stop
  (S-0607 claim 9), so the test now removes the lifter from play once the grip has ended.
  Nothing tests what the lifter does after a release with r70's timing.

## Earlier rows

Intake 52b's row for claim 28's cloak drops ("a holder kept by an exception takes none") stands,
with r70's addition that such a holder can still lose the target at claim 29's check. The row is
updated there.

## Requests

- **Q-2261**: where in a holder's turn the check falls, whether a refusal begun in the holder's
  own turn of a check tick is acted on in that turn, and which behaviour decides when a queued
  one starts. Interim: the start of the turn (above), the next check tick, the behaviour current
  at the turn's start.
- **Q-2262**: whether holders in behaviours other than *attack* and *guard* (and *capture* and
  *sabotage*, which make none) make the check. Interim: none do.
- **Q-2263**: the neutral and special houses, and targets from retaliation or a revert.
  Interim: those houses make no check and keep S-0304 claim 2's drop; a player house checks
  whatever gave the target.
- **Q-2264**: a materialising vehicle or aircraft, with and without a beam locked on it.
  Interim: with one, as being erased; without one, the firing check's order as built.

Older requests checked against what is built:

- **Q-2006** (a target that has just left play) and **Q-2114** (the targets of a crushed unit),
  answered by S-0309 r51 and r53 claim 27 (intake 52b): r70 changes nothing there. A removed
  target is dropped inside the removal, so claim 29 never sees it.
- **Q-2112** (ground hunters against hovering jumpjets, answered in part): its hunters are in
  *hunt*, which claim 29 does not cover in our engine (Q-2262), so r70 gives them no new drop or
  target. Question 1's occasion is still open, and DISK against FV still fails.

## C7

No quirk added; r70 adds none.

## Formats

No new state: the check reads `w.tick` and the existing target and behaviour. The save format
(92), the AI game format (54) and the wire version are unchanged. `GLASNOST_UPDATE_GOLDEN=1` on
the gate crate moved seven goldens, all state-hash lists that change from the first check tick
at which a holder drops a refused target: `duel.txt`, `duel-diagonal.txt`, `duel-m6c.txt`,
`duel-m7a.txt`, `duel-m7b.txt`, `duel-naval.txt` and `skirmish.txt`.

## Lanes

Before is main at 746ba32: intake 67, the capture seam (S-0215 claim 28) and intake 69, which
changes only the client. Before is a fresh run of 746ba32: `local-lane/m8/intake68/main-lanes`
for the sweeps, the arena and the AI tests, and `local-lane/m8/intake68/main-quick` for the quick
lane. The capture seam's own after run (`local-lane/m8/seam-capture/after`) came from before that
branch's rebase. The fresh run matches it exactly: the same sweep counts and byte-identical arena
csvs. After is this branch rebased onto 746ba32 (`local-lane/m8/intake68/after2`, the code of
546adc1). Outputs are in `local-lane/m8/`, which is git-ignored.

| Lane | Before | After |
|---|---|---|
| M6c sweep, seeds 1–16 (29 counts) | passes | passes; 25 counts move. Lower minimums: garrison orders 13 → 10, tech building income cycles 2 346 → 1 372, jumpjet orders 21 → 2, jumpjet flight 8 984 → 8 135, jumpjet units built and flown 8 → 6, radiation in the field 5 129 → 1 890, chrono erasure orders 20 → 19, prism buildings ordered 10 → 7, charger orders 9 → 6, support beams 66 → 0, suicide orders 14 → 6, sabotage orders 10 → 3, air-strike orders 5 → 3, air strikes called 8 → 6, crate orders 73 → 65, crates picked up 11 → 9 |
| M7a sweep, seeds 1–16 (26 counts) | passes | passes; 24 counts move. Lower minimums: berserk weapon orders 11 → 7, units made berserk 18 → 14, slave-miner buildings placed or deployed 13 → 5, vehicles bunkered 5 → 4, bunker exit orders 4 → 3, seized units dropped 5 → 4 |
| M7b sweep, seeds 1–16 (34 counts) | passes | passes; 10 counts move, with only drained-building ticks lower at the minimum (19 182 → 19 169) |
| Arena `--record all`, scored as intake 47 / A6: M-0001 | 9 of 96 failing | 9 of 96, `ours-M-0001.csv` byte-identical |
| M-0002 | 1 of 24 | 1 of 24, identical |
| M-0003 | 1 of 35 | 1 of 35, identical |
| Arena total | 11 of 155 | 11 of 155 |
| `glasnost-ai --include-ignored` (release) | 239 pass | 239 pass |
| AI quick lane, seeds 7–10, maps 0, 2 and 8 | 36 games, all ended; house 0 won 15, house 1 won 21; median 42 716 ticks | 36 games, all ended; none end at the same tick; house 0 won 18, house 1 won 18; median 42 455 ticks |

The sweep games are AI games, and a drop that moves by up to 15 ticks changes their course, so
the counts move both ways. Every sweep test passes with its minimums.

The M6c minimums fall further on this base than on the first one, and most of the fall comes from
seed 3. Its smallest count falls from 4 on main to 0: support beams 0 and jumpjet orders 2. The
other fifteen seeds keep a smallest count of 3 or 4. The sweep test still passes, because it
accepts a count below 3 in a single seed, as the M7a sweep on main already shows in several seeds.
It is still the largest move in this intake, so it is a candidate for a follow-up look at that
game. It was not traced in this intake.

On the first base (7f7ce87, intake 67, without the capture seam) the same change passed every
sweep too. The M6c sweep had fewer lower minimums there, chiefly garrison orders 22 → 13 and
crate orders 78 → 37. M7a and M7b were identical to the runs above, the arena was unchanged at
11 of 155, and the quick lane went from a median of 45 620 to 46 317 ticks.

The 11 failing arena pairings, each unchanged (same rows, same verdict): DISK–FV, LTNK–E1,
CAOS–APOC, CAOS–TTNK, TELE–APOC, TELE–TTNK, TELE–V3, CAOS–E1, CAOS–V3, INIT–E2 and ROBO–HTK.
The three `ours-M-000x.csv` files are byte-identical before and after: no arena game's outcome
changes.

## Checks

- `cargo test -p glasnost-sim` (debug): passes.
- `cargo test --release -p glasnost-sim -- --include-ignored`: only the two expected failures,
  `pathing_c7p4::diag` and `claims_s0213_r89::claim_16b_a_vehicle_with_a_copied_capture_never_attacks_x`.
- `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`: clean.
- `cargo xtask check`: stock-guard's 105 known problems, nothing else. `cargo xtask gate`: ok.
- `cargo test --workspace`: **not run to the end.** The run was stopped by the session's memory
  guard while other intakes were building, and was not restarted. The other crates' changes are
  the seven goldens, which `xtask gate` and the gate crate's own tests cover.
