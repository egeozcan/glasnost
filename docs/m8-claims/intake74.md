# S-0309 r94, S-0215 r103, S-0214 r68, S-0213 r96, S-0203 r42, S-0204 r10: claim ledger, M8 step A3 intake 74

- **S-0309** r70 to r94 (28fbbcc, from 5f2b926). Answers **Q-2261** to **Q-2264**, intake 68's
  interims for claim 29 (the periodic held-target check), and adds claim 25 part k (the allied
  held-target drop) with scenario A6 part p runs p8 to p10.
- **S-0215** r91 to r103 (593d891), and r104 (5917193, pushed during the intake).
- **S-0214** r64 to r68 (24207de).
- **S-0213** r94 to r96 (529b790).
- **S-0203** r40 to r42 (f1a50d1).
- **S-0204** r9 to r10 (9fbda61).

Requests filed: none (Q-2306 to Q-2315 unused). The one order the revisions leave open in our
engine, the allied drop against the berserk countdown that also follows the first per-tick
start, changes no outcome (below), so it needs no question.

## Built

- **Place in the turn** (claim 29, Q-2261 answered). `combat::held::turn` is called in
  `tick::object_turn` right after `process::head` (the first per-tick start of S-0215 claim 11
  and the berserk countdown; an unlimited mind controller's early head included), and before a
  building's income and drain payout, `process::tail` (the behaviour processing with its
  searches and the per-tick search of S-0215 claim 33, the firing step, a building's delayed
  shot countdown) and the movement. So it follows the fall step and landing hit (`fall::turn`),
  the warp update (S-0513 claim 51) and the erasure turn of a mobile holder being erased
  (`temporal::turn`, which returns), and a building being erased (`behaves` false) reaches it
  never. Intake 68 had put it before the drainer's position check and the behaviour start.
- **Behaviours** (Q-2262 answered). Every behaviour but *capture* and *sabotage* makes the
  check, read from `beh.seen()` after the start, so the behaviour the start makes current
  decides.
- **Owners and targets** (Q-2263 answered). Every holder of every house, the neutral and special
  houses included: `held::periodic_covers` is gone. Our engine does not record what gave a
  target, and the check reads only the held target.
- **The attack processing's drop** (r94's Changes for S-0304 claim 2). `process::process_attack`
  no longer drops an illegal target for any unit; the building's own drop (S-0215 claim 32) is
  unchanged.
- **Claim 25 part k.** `held::allied_drop`, first in `held::turn`, in every turn that reaches the
  place, in any tick and any behaviour: a holder (infantry, vehicle, aircraft, building) not
  being erased, whose deciding house is not human-controlled (`HouseRole::Player` with `human`:
  every player house of our engine, AI houses included, keeps it), and whose target object is
  allied with that house (its own house included, `World::allied`), drops it with
  `spawn::clear_own_target` and a burst reset: no draw, no revert, no behaviour change. Kept when
  the holder is berserk (`psychic::berserk`), an engineer infantry (`structures::engineer`), an
  infantry that can occupy the building target (`structures::garrison::may_enter`, S-0508
  claim 1), or one whose second weapon is electric-charge with an overpowerable building target
  (`linked::overpowerable`). The deciding house of a passenger of a mind-controlled open-topped
  transport is the controller's (`acquire::controlled_carrier_house`, now `pub(crate)`).
- **S-0204 r10** (claim 5, Edge cases). `targeting::rep_distance` (S-0214 F1, now public) takes
  the root under `quirks.table_root_distance`; it used the table root always. F9's group centre
  already did.

No new state and no new switch: save format 92, AI game format 55.

## Fixtures changed by the wider check

Claim 29 now reaches every behaviour of every holder. A weaponless holder answers *impossible*
(S-0301 claim 9, "no weapon in the slot"), so it drops a held target at its first check tick.
Five synthetic tests had weaponless holders keep a cell or object target across a check tick.
They are re-armed or re-timed, and no assertion is loosened:

- **`claims_s0522`.** The paradrop plane PDPLANE is armed, which is the data model behind S-0522's
  Import mapping (its aircraft weapon rows) and S-0501 claim 20.
  `a_weaponless_paradrop_plane_drops_its_cell_and_retreats` pins the weaponless outcome: the
  plane drops its cell at the first check tick with every paratrooper on board, then retreats
  (S-0522 claim 6, no target).
- **`claims_s0506::claim_26_release_deadline_and_order`.** The rocket RKA is armed, as R21 arms
  its flagged aircraft. Its ammunition of 1 survives its turn in *r* + 2.
- **`claims_s0513_planes::claim44_no_primary_weapon_gives_no_camera_and_warns`.** The camera frame
  count is 4 and the launch comes just after a check tick, so the warning processing comes
  before the drop. With the default 16, the second approach processing always lies past a
  check tick.
- **`claims_s0406::claim_15_target_kept`.** The HUT has no build-up sprite (H = T + 3) and is
  placed just after a check tick, so no check falls in its construction. An armed HUT would
  search and take draws, which claim 17's no-draw assertion refuses.
- **`claims_s0610::m0011_a_berserk_crusher_holding_a_berserk_friendly_target_drives_over_it`.**
  The pair is placed just after a check tick. The crusher's first processing takes the held
  mite as its destination (S-0210 claim 11). The check drops the illegal target at tick 16
  (S-0610 claim 6, in *hunt* too), and the destination it leaves carries the crusher over the
  mite. There is still no shot and no damage.

## Claims

### S-0309 r94

| claim | status |
|---|---|
| 29, Place in the turn (Q-2261): after the first per-tick start, after a fall step and landing hit, before processing, searches, per-tick search, movement and firing, before a delayed shot counts down | Built (`tick::object_turn`, `held::turn`). Tested: `claims_s0309_r94::the_behaviour_made_current_by_the_first_per_tick_start_decides` (capture current, attack queued just before a check tick: the start makes attack current and the check drops the target; with nothing queued capture keeps it); `claims_s0309_r70::a_pending_delayed_shot_ending_in_a_check_tick_keeps_the_target` (unchanged) |
| 29, an aircraft: the behaviour current at its turn's start decides; after its own start-of-turn drops | Built: an aircraft's head is only its berserk countdown, so the check comes before its processing. The in-air drop of S-0501 claim 32 (`air::behaviour_turn`) comes after the check in our turn; both are draw-free clears with no revert, so the end state is the same. Seam: intake 76 (S-0501) |
| 29, Behaviours (Q-2262): every behaviour but capture and sabotage | Built. Tested: `claims_s0309_r70::other_behaviours_make_the_check` (hunt, area guard, sticky, move; was the interim test), `capture_and_sabotage_make_no_check` |
| 29, Owners and targets (Q-2263): every holder of every house | Built. Tested: `claims_s0309_r70::a_neutral_holder_makes_the_check_and_no_processing_drop`, `claims_s0309_r94::p9a_a_special_holder_in_area_guard_drops_an_illegal_target_at_the_check_tick` |
| 29, behaviours in which no weapon is aimed, a weaponless holder (*impossible*) | Built. Tested: `claims_s0522::a_weaponless_paradrop_plane_drops_its_cell_and_retreats`; fixtures above |
| 29, a second turn in a check tick checks too | Built by construction (`held::turn` runs in every turn reaching the place). Not tested: no setup here gives a holder a second turn |
| 29, Turns cut short: a building being erased makes no check; a mobile holder being erased drops its target each turn (S-0215 claim 2) | Existing; the place keeps both (`behaves`, the `temporal::turn` return) |
| 29, Turns cut short, a materialising holder with a temporal beam attached (Q-2264) | Intake 75 (S-0513 claim 39a, merged before this branch): the check skips a turn that `temporal::cut_short` cuts short, `phase_in_beam_spares_infantry` included. In the rebase the allied drop takes the same test in place of `temporal::erased`. Tested by intake 75's `claims_s0513_r86.rs` |
| 29, a materialising holder with no beam gets *rearming*, first in rank 4 | Existing answer in `check::check`; the new precedence over the other rank-4 refusals is S-0301's change (run p10a). Seam: intake 79 |
| 29, Warp members (S-0203 claim 4): no check in the warp turns | Existing (intake 71); tested by `claims_s0513_r62::claim51_sw23_f_the_members_own_periodic_check_is_suppressed_through_arrival_effects` |
| 25 part k, the allied drop: computer owners, own house included, any tick, any behaviour | Built. Tested: `claims_s0309_r94::p9b_a_computer_owned_holder_drops_a_target_of_its_own_house_off_the_check_phase` (neutral and special), `the_allied_drop_is_made_in_every_behaviour` (capture and sabotage included), `a_computer_owned_holder_keeps_an_enemy_target` |
| 25 part k, a holder of a human-controlled owner keeps it (p9b's control) | Built. Tested: `p9b_control_a_player_holder_keeps_a_target_of_its_own_house`; the marked test house drops: `a_marked_test_house_takes_the_allied_drop` |
| 25 part k, claim 29's consequences: no draw, no revert, no behaviour change | Built. Tested: `the_allied_drop_takes_no_draw` (rng and targeting timer against a control), p9b test (behaviour unchanged) |
| 25 part k, the exceptions: berserk, engineer, occupier with a building it can occupy, electric-charge second weapon with an overpowerable building | Built. Tested: `a_berserk_holder_keeps_an_allied_target`, `an_engineer_keeps_an_allied_target`, `an_occupier_keeps_a_building_it_can_occupy`, `an_electric_charger_keeps_an_overpowerable_building`, each with controls that drop |
| 25 part k, a passenger of a mind-controlled open-topped transport: the controller's house for the owner test and the alliance | Built. Tested: `a_passenger_of_a_controlled_open_topped_transport_reads_the_controlling_house` (both directions, with uncontrolled controls) |
| 25 part k, not in a warp turn (S-0203 claim 4 r42) | Built by the place. Tested: `claims_s0513_r62::claim51_sw23_f_the_members_allied_drop_is_suppressed_through_arrival_effects` (a marked house's member keeps its own-house target in every warp row off the check phase, and drops it at the hold end) |
| 25 part k, not in a turn cut short | Built (`temporal::erased` and the place) |
| 25 part k, against the berserk countdown | Our order: the countdown first (`process::head`). No outcome depends on it: a countdown that ends berserk drops the target itself (S-0610 claim 5), and one that does not leaves the holder berserk for the keep either way |
| Changes for S-0304 claim 2: the attack processing's drop is a building's only | Built (`process::process_attack`). Tested: the neutral test above (an attack holder keeps its target past its processing and drops it at the check tick) |
| 7 and 9, the layer reading | Existing (`air::ground_filed`, `temporal::ground_layer`); tested in `claims_s0501_released.rs` |
| Runs p8 to p10 | p8 and p9 as above; p10 is S-0513 claim 39a and S-0301's precedence (seams) |

### S-0215 r103

| claim | status |
|---|---|
| 1, 7 (the aircraft queue row), 12, 19, 26, 28, table rows: the aircraft items | Seam: intake 76 (S-0501 claims 18, 34, 45, 49) |
| 2, the phasing-in-with-beam exclusions and `phase_in_beam_spares_infantry` | Seam: intake 75 (S-0513 claim 39a) |
| 10, a warp member's departure does not revert an override | Existing: `superweapons::warp::depart` calls no `holders_drop` and no revert (intake 60). Tested by `claims_s0513_r62::claim39_the_periodic_drop_of_the_member_reverts_no_override` and the landing-kill revert |
| 32, the periodic check note | By construction (S-0309 claim 29 above) |
| F6, the reset at the drops of claims 27 and 28, none during game set-up | Existing (`held::timer_draw`); no holder holds a target during our set-up, so the set-up exception holds by construction |
| B16 part e: maximum ammunition 0, no initial value | Tested: `claims_s0301::s0215_b16_e_zero_maximum_ammunition_returns_to_guard` (out of ammunition from the attack order; target cleared, guard current, 1 tick, no draw) |

### S-0215 r104

r103 to r104 (5917193), pushed during the intake. It adds two sentences to claim 37, both citing
S-0607 claim 9 (r38). Backing: unmeasured, confidence medium.

| claim | status |
|---|---|
| 37 part c: the step's drop (part d) and its failed search with no nearby cell (S-0307 claim 17) give a gripped unit no destination, so while it has one they end its grip | Existing: both write an empty destination (`approach::step`, `approach::search`), and `lift::steer` (jumpjet) and `lift::enforce` (stand-in) end a grip whose unit has no destination unless part e removed it (`grip_without_dest`). Cited. Tested: `claims_s0215_r88::a_seized_unit_hunting_its_lifter_drops_it_and_the_grip_ends` (part d), new `claims_s0215_r104::a_failed_search_with_no_nearby_cell_ends_the_grip` (an armed victim in the air hunting its lifter, out of range, part e keeping the grip's destination, every cell claimed for the search: target dropped, no destination, the grip ends, the unit still seized, the lifter keeps its target) |
| 37 part e: the clearing removes the destination directly, no effective clear in S-0502 claim 24's sense, no jumpjet stop | Existing (`lift::destination_removed`). Cited. Tested: `claims_s0607_r33::claim_9_part_e_clearing_keeps_the_grip_and_the_pull`, new `claims_s0215_r104::part_e_clearing_is_no_effective_clear_and_no_stop_follows` (in the air: the grip holds, no destination, the commitment unchanged) |

### S-0214 r68

| claim | status |
|---|---|
| Terms, *can be selected*: a seized unit cannot be selected | Existing (`targeting::seized_out`, S-0607 r33); cited |
| T4 part (e), land-targeting mode 1 | Tested: `claims_s0306_modes::s0214_t4_e_mode_one_shows_attack_with_force_fire` and `claim23_cursor_offers_no_attack_to_mode_one` |
| F1 and F9 under `quirks.table_root_distance` | Built (F1); F9 existing. Tested: `claims_s0309_r94::the_representative_distance_follows_the_table_root_switch` |

### S-0213 r96

| claim | status |
|---|---|
| The pad-bound aircraft scenario | Seam: intake 76 (S-0501 claim 50, AC31 part b) |

### S-0203 r42

| claim | status |
|---|---|
| 4, the check and the allied drop suppressed in warp turns | Built (the place, after the warp return). Tested as above (SW23 part f, both) |
| 4, the erasure update of an attached beam before the warp work | Seam: intake 75 (S-0513 claim 39a) |

### S-0204 r10

| claim | status |
|---|---|
| 5 and Edge cases: the switch covers S-0214 F1 and F9 | Built and tested (S-0214 row above) |

## Lanes

The real-install lanes ran on the first base (0cb93b5), before and after this intake's code, so
they show this intake alone. The goldens, the gate and the AI tests ran again on the final base,
main 23869cf, which has intakes 75 and 76.

- **Goldens.** Seven moved against main and were regenerated: duel, duel-diagonal, duel-naval,
  duel-m6c, duel-m7a, duel-m7b and skirmish. On the first base the naval duel ran its full
  40,000 ticks undecided, where before it had ended at about 24,840. On the final base it ends
  at 2569 hash lines, against 2561 on main, so it is decided again. `xtask gate` passes, native
  and wasm.
- **Arena** (`arena --record all`). Failing pairings went from 13 to 10.
  - Now passing: CAOS–V3, TELE–TTNK, YURI–GGI and YURI–SHK.
  - Newly failing: MIND–E1 (stock 12/12, ours 11/12; one side's outcome differs).
  - Still failing: DISK–FV, TELE–V3, CAOS–E1, LTNK–E1, TELE–APOC, CAOS–APOC, CAOS–TTNK,
    INIT–E2 and ROBO–HTK.
- **Sweeps.** All exit 0. Seeds with no counter below 3: M6c 7 → 8, M7a 0 → 1, M7b 16 → 16.
  - M6c minima: garrison occupants 6 → 4, sabotage orders 0 → 8, suicide orders 4 → 11, crate
    orders 10 → 40, radiation exits 1 → 7, disarm orders 6 → 4, bombs planted 12 → 10.
  - M7a: berserk orders median 20 → 16, absorber entries min 2 → 1, grinder entries min 8 → 6,
    vehicles seized max 114 → 188. Infantry absorbed min stays 0.
  - M7b: small moves only.
- **AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games). Every game ends, before and
  after.
  - The winner changed in 8 of the 36 games.
  - Total ticks went from 1,507,624 to 1,774,001. The longest game went from 78,529 to 91,418
    ticks.
  - Paradrop launches went from 218 to 243 and spy-plane launches from 211 to 268. No warnings
    were logged. The lane has no per-mission counter, so a mission that ended early would not
    show here; no evidence of one turned up.
- **`cargo test --release -p glasnost-ai -- --include-ignored`.** First base: 246 passed, 0
  failed. Final base: 246 passed, 0 failed, in 33 binaries.

## Seams

- **Intake 75** (S-0513, S-0514, S-0610): S-0309 claim 29's materialising holder with a beam
  attached, S-0215 claim 2's phasing-in exclusions and `quirks.phase_in_beam_spares_infantry`,
  S-0203 claim 4's erasure update before the warp work. Merged on main (23869cf) before this
  branch's last rebase. Both sides are kept in `tick::object_turn`: the member's erasure update
  (`temporal::member_update`) and its early return come right after `held::turn_start_drop`
  and before the warp step, and `held::turn` stays after `process::head`. The check skips turns
  that `temporal::cut_short` cuts short, and the allied drop now does too.
- **Intake 76** (S-0501, S-0502), merged on main (3ec77d6): S-0215's aircraft items, S-0213 r96's pad-bound scenario, the
  aircraft's in-air drop against the check (S-0501 claim 32), and the aircraft cloak drop of
  S-0516 claim 25, which runs in `cloak::late_turn` after movement where S-0501 claim 32 says
  before processing (the end state against the check is the same).
- **Intake 79** (S-0301): the materialising *rearming* first in rank 4 (p10a).
