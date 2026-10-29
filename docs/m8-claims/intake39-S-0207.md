# S-0207 r82: claim ledger, M8 step A3 intake 39

This intake takes in one spec revision and closes the lead's play note 8 (a chrono miner
teleports to any click; [a4-play-notes.md](a4-play-notes.md)).

- **S-0207 revision 82** (spec commit bdd7fa8, answering Q-1759). The diff against the revision
  our engine built (r41, d1943f4). The rule changes:
  - Claim 28 (new): a vehicle whose type has the teleport-capable flag drives, except on its
    docking approach. Every assignment that passes claim 27's unchanged rejection, an empty one
    included, installs temporary driving over the teleporting locomotor, unless it is a qualified
    docking assignment (claim 27's dock test without its last condition) or one of the three
    exclusions holds (warp-pending immobilization, gripped, seized). A jump's removal of its
    destination and claim 20's drops install driving from that tick. In *enter* a flagged
    vehicle's destination is tested again at each processing (the repeated dock test), so a
    miner held off an occupied docking cell jumps once the cell clears. The offer and the repeated
    test in one processing make one jump.
  - Claim 21: installing temporary driving cancels a pending jump; the landing point and its
    reservation are kept.
  - Claim 20: (a) the vehicle's own mark at the old landing point is cleared before the entry
    test of the new one; (e) a behaviour that ends with its destination gone falls back to rest at
    a later tick, not in the jump tick.
  - Claim 16 (b): the scatter refusal reads the current locomotor (scenario D17 d: "active
    movement kind").
  - Claim 27: the exclusions read "not gripped"; the wording of the fallback (it stops a driving
    vehicle, which finishes its track).
  - Quirks `teleport_aircraft_ground_reservation`, `teleport_vehicle_ground_reservation` and
    `drive_duplicate_scatter_request`: proposed to decided (no value change).
  - Scenarios D17 d, D22 (runs 1, 2 and 4), D23, D24 and D34 (new) rewritten for claim 28.
  - Unchanged: claim 19 (hover).

The cross-specs it names were read as they stand: S-0402 r76 (claims 9, 10, 10a, 11, 17, 18,
H8), S-0417 claims 4, 5, 15 and 16, S-0215 claims 9, 13 and 15, S-0607 claims 4, 5 and 9. Text
in other specs that still reads the old rule (every destination teleports) and that claim 28
overrides: S-0214 Terms; S-0417 U6 and U15; S-0215 B15 e, claims 9 and 13; S-0502 claim 27. This
intake also builds the chrono parts of S-0402 r76 that intake 36 left waiting.

**Code.**
- `refresh::assignment` (new, called from `refresh::begin` after the unchanged rejection): claim
  28's rule. `refresh::flagged` is the flag test, `excluded` the three exclusions
  (`superweapons::warp_pending_or_leaving`, `lift::lifter_of`, `lift::seized`). A qualified
  docking assignment is left to claim 27 (teleporting current: nothing; otherwise restore or
  fall back); any other assignment calls `econ::release::install_temporary_drive` when the current
  locomotor is teleporting.
- `econ::release::install_temporary_drive` cancels a pending jump and keeps the landing point and
  its reservation (claim 21).
- The removals that install driving: `teleport::jump` and `teleport::drop_destination` call
  `refresh::removed`; a destination that reaches no assignment hook (a fall-back's queued
  destination) reaches `refresh::untaken` from teleporting's catch-all, so driving gets it.
- The repeated dock test: `econ::harvest::process_enter` treats a flagged vehicle's destination
  as a new assignment (`refresh::reassign`) at each processing, after the row 2 offer.
  `path` calls `refresh::adopted` after a substitute is adopted (S-0402 claim 10a: the vehicle
  stops at its track's end).
- `teleport::fix_landing_with` clears the old mark before the entry test (claim 20 a).
- `refresh::fallback` stops driving (`drive::stop`); `drive::tick` and `fresh_search` honour the
  stop (claim 27, claim 11).
- `scatter`: claim 16 b reads the current locomotor (`teleport::is_teleporter`). A vehicle
  scatter request sets its destination without the assignment hook; for a flagged vehicle it now
  calls `refresh::adopted`, so the dock test and claim 28 act on it (*A sale of the refinery*).
- `econ::harvest::seeking` reads the drop after the adoption (S-0402 claim 17, case (c)).
- No new saved or hashed state; save format 83 and AI game format 45 are unchanged.

**Design.**
- Claim 28 names only "temporary driving". A flagged type whose own locomotor is not teleporting
  (a test type: the flag on a driving vehicle) has nothing installed; at a qualified docking
  assignment it gets claim 27's fallback, as claim 28 *An occupied docking cell* says, so it
  never docks and keeps the refinery slot. No stock type is known to have that pairing.
- With no landing cell found, `fix_landing_with` marks the old cell again rather than leave the
  vehicle with no mark.

| Claim or item | Status | Notes |
|---|---|---|
| 28 (the rule: any other assignment installs driving; no landing point) | tested | `claim_28_any_other_assignment_installs_driving` (D34 a, d; the flag-off type still jumps). |
| 28 (an empty assignment counts; the unchanged rejection first) | tested | `claim_28_an_empty_assignment_installs_driving`. |
| 28 (exclusions) | tested in part | `claim_28_a_seized_vehicle_has_nothing_installed`. Warp-pending and gripped: by construction (`refresh::excluded`). |
| 28 (placement installs driving) | tested | `claim_28_placement_installs_driving` (D34 f). |
| 28 (qualified docking assignment; the jump's removal installs driving) | tested | `claim_28_a_docking_assignment_jumps_and_the_jump_installs_driving` (D22 run 2). |
| 28 (an occupied docking cell; the repeated dock test) | tested | `claim_28_an_occupied_docking_cell_then_the_repeated_dock_test` (D34 e). |
| 28 (one jump) | tested | `claim_28_one_jump_when_the_offer_and_the_test_both_act`. |
| 28 (a destination with no assignment hook) | tested | `claim_28_a_queued_destination_is_driven` (S-0215 claim 15 row 2). |
| 28 (D34 c: the cycle) | tested | `claim_28_d34_c_the_cycle_jumps_only_to_the_docking_cell` (3 loads, 3 jumps, each to the docking cell in *enter*; no stray reservation). |
| 28 (a sale of the refinery: a full unloading miner jumps to the scatter cell) | tested | `claim_28_a_sale_jumps_to_the_scatter_cell` (D34 h runs 2 and 4: the jump in the sale's first processing tick, *harvest* current, driving again). |
| 28 (a sale: a miner not full, the remembered or ring-search cell; one still in *enter*) | tested in part (seams sweep 2) | Not full: `claim_28_a_sale_with_a_part_load_jumps_to_the_ring_search_cell` (the remembered cell dropped, the ring-search cell jumped to in the same turn, never the scatter cell), `claim_28_a_sale_with_the_remembered_cell_taken_drives_there` (a ground-layer vehicle in the remembered cell: driving takes it, the scatter jump cancelled), `claim_28_a_sale_with_nothing_found_stays_on_the_docking_cell` (*idle*, driving, no destination). A miner still in *enter*: the same assignment paths, no test of its own. |
| 28 (nested other temporary replacement) | not reachable | No other temporary replacement exists outside the exclusions. |
| 21 (install cancels a pending jump) | tested | `claim_21_installing_driving_cancels_a_pending_jump`; `claims_s0207_r34.rs` updated. |
| 20 (a) (old mark cleared first) | tested | Through the one-jump test. |
| 20 (e) (fall-back after the jump tick) | tested | `claim_20_e_the_fall_back_comes_after_the_jump_tick` (D22 run 1). |
| 16 (b) (current locomotor) | tested | `claim_16_b_reads_the_current_locomotor` (D17 d). |
| 27 (the fallback stops) | tested | `claim_27_the_fallback_stops_a_moving_miner`. |
| Quirks (decided) | by construction | No value change; the quirk lint passes. |
| 19 (hover) | unchanged | Built before (`hover.rs`). |
| S-0402 r76 Terms, 5a, 9, 10, 10a (chrono), 17 (a) to (f), 18 (chrono), H8 | tested in part | `claim_17_case_c_the_remembered_cell_is_driven_to`; the cycle and occupied-cell tests above; the rest by construction. |
| S-0402 case (e): a warp-pending miner's ring search | open | The spec leaves it open. |

Tests whose expectations moved with claim 28 (each now uses a flag-off teleporting type `TELEQ`
where the test is about teleporting, not claim 28): `claims_s0207_r34.rs`, `claims_s0207_r36.rs`,
`claims_s0402_r28.rs`, `claims_s0417_r19.rs`, `claims_s0417_r20.rs`. `claims_s0402.rs`
`claim_10_row_2_offers_reach_the_dock_test`: the flagged driving type now stays at the fallback
(the design seam above), the plain harvester docks. `claims_s0417_r19.rs` and `r20.rs`: claim 16 b
lets a driving vehicle accept S-0417 claim 15's contact-end scatter and claim 4's apron clearing,
so the factory exit point is usually replaced at once (S-0417 claim 16 row: "an accepted scatter
supplies a destination").

**Lane** (`crates/app/tests/drive_lane.rs`, local only, outputs in the git-ignored
`local-lane/m8/a3-39/`). A skirmish on the install: the deployed base, a refinery, a chrono miner
and a plain harvester sent to harvest, and four driving types sent about 20 cells off. Over 4500
ticks the chrono miner made 4 jumps, all 4 onto the refinery's docking cell in *enter*, none
elsewhere; it drove 67 cells and had teleporting current for 0 tick ends (it is teleporting only
inside a docking jump's tick). Three of the four driving types reached their cells; the fourth
did not move: its goal cell, chosen by offset, lies in no reach region of its movement zone
(S-0212 claim 4; the lane prints each driver's start and goal regions), while the other three
goals share their start's region.

**Gate and margins** (main 70eef44 before, this branch after; the sweeps and the arena ran on
the build before the scatter-request hook, which acts only on vehicles with the teleport-capable
flag: since claim 28 itself moved none of these games, none of them has such a vehicle).
- Sweeps M6c, M7a and M7b, seeds 1 to 16, reference and with the play-fixes layer: every min,
  median and max is the same as before, and so is every per-seed line. The gate games pass.
- Oracle arena (local lane): the three game files are byte for byte the same. M-0001 30 of 96
  pairings failing, 150 of 171 unanimous outcomes reproduced; M-0002 12 of 24, 42 of 47; M-0003 2
  of 35, 60 of 64.
- Goldens: all pass unchanged (`GLASNOST_UPDATE_GOLDEN=1` rewrote nothing; with the
  scatter-request hook, `cargo xtask gate` passes on native and wasm32).
- Quick skirmish (`glasnost-headless ai-sides 120000 7 <map>` with `GLASNOST_AI_QUICK=1`; maps 8,
  2 and 0; Allied as house 0 against Soviet and against Yuri; main and this branch; 12 games,
  `local-lane/m8/a3-39/quick/`): every game is over with a winner and none panicked. The Allied
  house's chrono miners now drive to the ore and jump only to the refinery. Its income, main /
  this branch (game length in ticks): map 8 against Soviet 44 161 / 30 815 (22 774 / 32 598),
  against Yuri 52 535 / 34 976 (27 688 / 47 051); map 2 against Soviet 64 376 / 68 299
  (30 770 / 48 233), against Yuri 83 201 / 35 449 (49 712 / 42 163); map 0 against Soviet
  43 437 / 53 862 (26 258 / 63 247), against Yuri 50 278 / 23 341 (40 232 / 40 533). It falls in
  4 of 6 games, and in the other two the game ran much longer. The Allied house won 4 of 6
  games on main and 2 of 6 here. Watched on map 0 against Yuri
  (ticks 16 000 to 24 000): a chrono miner loads at a field about 16 cells from its refinery,
  jumps onto the docking cell in *enter*, unloads, and drives back in about 350 ticks; a cycle
  takes about 1 300 ticks. On main the same miners jumped both ways. In that game the Allied
  house's two chrono miners made 34 jumps, all 34 onto the docking cell in *enter*. Its refinery
  was destroyed at about tick 24 500 and the AI did not build another (no refinery spending,
  credits near 0), so both miners stood in *guard* until about tick 39 800; that, not claim 28,
  is where this game's idle harvester ticks come from.
