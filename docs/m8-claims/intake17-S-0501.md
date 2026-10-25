# S-0501 r52: claim ledger, M8 step A3 intake 17

This intake takes in one spec revision. It answers one request, so one interim is retired:
Q-1841 (where an aircraft docked at a positive docking height is listed).

- **S-0501 revision 52** (spec commit c418c13; Q-1841 answered in ca3cd50). It covers r46 to r52
  against r45, which intake 16 built.
  - One behaviour changes: docking by landing no longer puts the aircraft at its docking position
    at touchdown. Instead, while it lands, it moves toward that position horizontally, at most 5
    leptons per map axis per processing, and keeps the landing height (claim 24, AC14).
  - It adds claim 48 on an aircraft released above its pad, claim 47's exceptions for it, the
    switch `quirks.aircraft_released_above_pad`, AC30 part k and Open question 25.
  - Terms now says that the on-the-ground test reads height. Claim 7 says when the render list is
    matched to the height.
  - Import mapping now reads `DockingOffset` from art.
  - Changes for other specs adds S-0309 claims 7 and 9 and S-0404 claim 15, both clarifications.

The switch was already on main. It is now consulted, and its doc comment (in `quirks.rs` and the
ruleset schema) no longer says "Not consulted yet". Claim 48 (a) also names the renderer's switch
`quirks.layer_change_on_events_only` (S-0102). The renderer did **not** do what claim 48 (a) states:
its placement re-file read the post-tick height, so a released aircraft at 100 leptons was filed in
the top layer. It now files such an aircraft in the ground layer (below).

**Code.**
- `air.rs`:
  - `AirState` gains `released` (left above its pad by a factory release, until the held height of
    ordinary flight, a landing, a take-off or a crash) and `ground_filed` (still in the ground
    render layer's list from its ground placement, with no re-file since). Both are saved and
    hashed: save format 77, AI game format 39.
  - `released_docked`:
    - With the switch on, the aircraft is put at the docking position, height included. At a
      positive height it is not settled, not landing and with no take-off. It holds height 0, both
      speed fractions are 0, and it is in neither cell layer (`sync_listing`), not tracked, and
      `released` and `ground_filed` are set.
    - With the switch off, it is settled at the landing height above the terrain at the docking
      position's horizontal location.
  - `dock` no longer moves the aircraft. The new `docking_step` moves it toward the docking
    position (a non-building contact gives its reference point) by the difference clamped to −5 …
    5 per axis, keeping its height.
  - `enter`: the processing that finds the aircraft landing (or on the ground) in contact on its
    pad sets the docking stage and returns 3. Each later processing returns 1 and either makes a
    docking step (while landing) or docks (once down).
  - `point_with`: pad guidance replaces only the horizontal location of the building point.
  - `tick_flight`:
    - The take-off test reads `on_ground` (height) besides the settled and landing flags.
    - A released aircraft with no destination and a current fraction of 0 rests: no step, no
      height change.
    - Claim 48 (b) case 3 is applied at the start of the update, case 2 at its end.
    - The interim of Q-1875 clears its mark at its first update toward a destination.
  - `sector_update` takes in a released aircraft whatever its height, with the corner switch still
    applying.
  - `guard`: the claim 29 pad search tests `h != cruise`, and the cruise-height branch tests
    `h == cruise` (claim 48 (d), equality only).
  - `destination_taken`, new: it ends a released aircraft's pad contact when it takes a destination
    other than the pad and its cell belongs to the pad.
  - `touchdown`, `take_off`, `start_landing` and `impact` clear `released`.
- `econ::assign_destination` and the behaviour orders in `orders.rs` call
  `air::destination_taken`. This covers a person's order and the rally point's move order at the
  release.
- `combat::effects::layer_refile` clears `ground_filed` at every re-file moment.
- `combat::acquire::search`: the map-wide ground-layer pass also takes an aircraft with
  `ground_filed`.
- `crates/app/src/mapview/layers.rs`: `Layers::evaluate_mobiles` files a newly seen aircraft with
  `ground_filed` in the ground layer. With the S-0102 switch off, the sweep then moves it to the
  top layer.

**Tests.**
- `crates/deterministic/sim/tests/claims_s0501_released.rs`, new, with synthetic pads:
  - HIPAD: displacement 0, 0, 100;
  - LOWPAD: 0, 0, 15;
  - OFFPAD: 64, 64, 0;
  - FLATPAD.

  And synthetic aircraft:
  - JET: armed, pad-bound;
  - NEAR: JET with a slowdown distance of 2000;
  - SLOW: JET at 5 leptons per tick;
  - SCOUT: the unarmed control.
- In `claims_s0501.rs`, two comments were updated to the new docking.
- `crates/app/src/mapview/layers.rs`: `claim4_released_aircraft_stays_in_the_ground_layer`, new.

**Goldens, arena and sweeps.**
- *Before* is main 23d450b; *after* is this branch.
- Goldens regenerated: duel, duel-diagonal, duel-naval, duel-m6c and skirmish. Unchanged: scripted,
  combat, duel-m7a and duel-m7b.
- How the causes were found: in a scratch copy, the two new fields were left out of the hash and
  the changes were reverted one by one.
  - duel, duel-diagonal, duel-naval and skirmish:
    - Their hashes change from the first aircraft, because of the two new saved fields.
    - Their play changes because of claim 24's docking steps and claim 40's horizontal guidance.
      With those reverted they match the old goldens.
  - duel-m6c: diverges from tick 510, by the take-off test that reads height (Terms row above). It
    matches only with that reverted too.
  - Claim 48 (d)'s equality-only height tests change no scenario. No gate pad docks above the
    terrain, so no release rests above a pad.
- Oracle arena (local lane): the CSVs are byte-identical before and after.
  - M-0001: 31 of 96 pairings failing; 152 of 172 unanimous stock outcomes reproduced.
  - M-0002: 13 of 24 failing; 43 of 47 reproduced.
  - M-0003: 2 of 35 failing; 61 of 64 reproduced.
- M7a and M7b sweeps 1 to 16: every counter has the same minimum, median and maximum before and
  after. M7a's smallest minimum is 1 (takeover orders); M7b's smallest drain minimum is 3.
- M6c sweep 1 to 16: the counters change. The take-off test is the cause in the gate's seed
  (duel-m6c above). In the other seeds, claim 24's docking may add to it; this was not separated. The smallest minimum falls from 4 to 2 (seed 3: charger orders 2;
  seed 12: jumpjet orders 2), and no counter reaches 0. Minimum / median, before → after:
  - garrison orders 9/28 → 12/30; occupants 4/7 → 5/6; engineer 6/6 → 6/6; captures 4/4 →
    4/4;
  - spy 5/6 → 5/6; infiltrations 4/6 → 4/6; tech 4/4 → 4/4; income cycles 2338/2344 → 2166/2344;
  - jumpjet orders 14/51 → 2/51; jumpjet flight 9108/9116 → 8829/9116; jumpjets built 7/8 → 7/9;
  - deploy orders 52/122 → 60/116; changes of form 4/7 → 4/6;
  - radiation exits 5/19 → 13/27; radiation ticks 4936/7216 → 4561/6851;
  - erasure orders 17/33 → 21/33; erasures begun 5/10 → 5/11;
  - prism 10/11 → 7/11; charger 7/17 → 2/18; support beams 54/98 → 11/94;
  - bomb planter 27/51 → 31/49; bombs planted 7/17 → 9/18; disarm 5/10 → 4/9; suicide 12/21 →
    12/20; sabotage 5/16 → 3/18;
  - air-strike orders 4/17 → 11/17; strikes called 9/17 → 10/17;
  - crate orders 64/136 → 77/124; crates picked up 8/16 → 11/18.

| Claim or item | Status | Notes |
|---|---|---|
| Overview, Q-1841 in full (1) to (3) | followed | (1) and (3) by claim 24's rows below; (2) by S-0209 claim 13's listing, unchanged. |
| Terms, *on the ground* reads height, never the filed list | built, tested | `air::on_ground` reads height. Claim 18's fall-back rows and claim 47's start already read it. `claim_48_leaving_far` (no take-off at the order). |
| Terms and claim 8, the take-off test reads height | built, tested (new) | Code change: `tick_flight`'s take-off test was `ground \|\| landing` and is now `ground \|\| landing \|\| on_ground`. An aircraft at height 0 that is neither settled nor landing now takes off at a new destination. `claim_8_take_off_test_reads_height` fails on the old test. In the gate's duel-m6c, an air-strike aircraft at height 0 in flight (ticks 517 and 637) now takes off; that changes its golden and the M6c sweep (below). |
| Claim 1, the release moves the aircraft to its docking position, height included | built, tested (new) | `released_docked`; `claim_48_release_rests_above_its_pad` checks the position against `pad_position`. |
| Claim 4, the docking approach is a separate movement | built | The descent nudge is still skipped in *enter* toward a helipad; `docking_step` is the separate move. |
| Claim 7, the render list matched to the height only at S-0102 claim 4's moments | built, tested (new) | The renderer's placement keeps a `ground_filed` aircraft in the ground layer; `layer_refile` clears the flag. `claim4_released_aircraft_stays_in_the_ground_layer`. |
| Claim 8 and S-0417 claim 11, the contact ends at the order (claim 48 (c)) | built, tested (new) | `destination_taken`. `claim_48_leaving_far` (a person's order) and `claim_48_rally_point_at_the_release` (the rally point's move order). |
| Claim 8, a docking position outside the pad's foundation keeps the contact | built | `pad_cell` test in `destination_taken`. Not tested: no synthetic pad puts its docking position outside its foundation, and AC30 does not either. |
| Claim 24, horizontal docking steps, clamped −5 … 5 per axis, the height never applied | built, tested (new) | `enter`, `docking_step`, `dock`. `claim_24_docking_steps_are_clamped` (AC14 shape): no step in the first 2 ticks after the landing starts, the first step 3 to 6 ticks after it, then exactly the clamped difference each tick, with both axes limited and one axis already used up. `claim_24_docking_at_a_raised_pad_rests_at_the_landing_height` (AC30 k (1)). The earlier docking tests of `claims_s0501.rs` still pass: with no displacement the steps end at the pad position. |
| Claim 24, an aircraft already on the ground and not landing at the docking stage is not moved | built | `enter` docks it without a step. |
| Claim 24, the stage delays 1, 3 and 1 ticks | built, tested | As before, with the docking stage now returning 1 at each step; `claim_24_docking_follows_touchdown`. |
| Claim 40, pad guidance by the horizontal location only | built | `point_with`. Covered by the raised-pad docking test, whose pad has a height component of 100. |
| Claim 47 (a), the released aircraft starts no take-off and is first tracked at a movement update | built, tested (new) | `claim_48_leaving_far`: not tracked at the order nor in the first update, tracked in the second. |
| Claim 47 (b), the released aircraft is in neither cell layer; the low release's overlap | built, tested (new) | `assert_resting` in every rest test. `claim_48_low_release`: listed at height 0 after the first update, tracked at the start of the second. |
| Claim 47 (c), a released aircraft is taken in whatever its height; the corner switch applies | built, tested (new) | `sector_update`. Tested outside sector (0, 0) only. |
| Claim 48 (a), resting: in flight, not landing, no take-off, neither cell layer, not tracked, ground render layer | built, tested (new) | `claim_48_release_rests_above_its_pad` (300 ticks: height and position unchanged; *guard*; area damage misses it while the landed control is hit). `claim_48_an_unarmed_release_rests_too`. |
| Claim 48 (a), range and area searches pass over it | built | By construction: no cell lists it (S-0309 claim 10), and the airborne scan reads only tracked flying objects (claim 9). |
| Claim 48 (a), the map-wide search's two passes | built, tested (new) | The aircraft pass is unchanged. The ground-layer pass reads `ground_filed`. `claim_48_map_wide_ground_pass_takes_the_released_aircraft`: a vehicle's map-wide search finds it, and finds nothing once the flag is cleared. |
| Claim 48 (a), "with S-0102's switch off only the aircraft pass considers it" | seam | The renderer's switch is client state (EXTERNAL_QUIRKS) and does not reach the simulation, whose map-wide pass reads `ground_filed` whatever the client's setting. Only the drawing follows the switch. |
| Claim 48 (b), the held height from the placement; the three cases | built, tested (new) | Case 1 by `take_off`. Case 2 is tested by `claim_48_leaving_far` (the held height is 500 at the end of the first update) and `claim_48_leaving_near`. Case 3 is tested by `claim_48_low_release`. |
| Claim 48 (c), leaving far: the first update with no forward step, the F2 descent to 80, the 5-lepton nudge, fraction 0.1; the second update tracked, a forward step, the F1 climb | built, tested (new) | `claim_48_leaving_far` (AC30 k (3)), and `claim_48_rally_point_at_the_release` for the rally order. |
| Claim 48 (c), the low release | built, tested (new) | `claim_48_low_release` (a target fraction of 1). The lower-fraction branch (it goes on at height 0) is built by the same rules but not tested; the spec rates the low release low. |
| Claim 48 (c), the ground render layer kept while it leaves | built, tested (new) | `claim_48_leaving_far` checks `ground_filed` until its landing starts; the renderer test checks the drawing. |
| Claim 48 (d), *guard* by T2's ground row, the rally order, contact held until a destination | built, tested (new) | `claim_48_release_rests_above_its_pad`, `claim_48_rally_point_at_the_release`. |
| Claim 48 (d), equality-only height tests in *guard* and *area guard* | built | `guard` now uses `h != cruise` and `h == cruise`; `area_guard` already compared for equality (claim 45). Unarmed: 45 ticks; armed for a human player with the in-the-air answer *no*: the standard delay. |
| Claim 48 (d), an armed aircraft of a human player with the in-the-air answer *yes* | seam | S-0215 claim 24's *guard* with its rearm and distributed delays is not built for aircraft; our *guard* gives the standard delay. Not reachable on AC30's pad (below 208 leptons). |
| Claim 48 (d), a computer-controlled house (*guard* in a team with its house's target search, *area guard* outside one) | seam | Our own AI's houses take the human-player rows here, as elsewhere in S-0501. The area-guard rows are built (`process_area_guard`), but no test releases an armed aircraft for an AI house. |
| Open question 25 (a release at exactly the cruise height; a destination inside the halving zone; the region test of a non-aircraft map-wide searcher) | interim, no request | The spec already lists these as untraced, so no request is filed. At the cruise height, *guard* and *area guard* take the cruise outcomes by the equality tests, and the speed fractions start at 0 as at any other height. A near destination runs through the same rules (`claim_48_leaving_near`). The vehicle's region test applies as for any target. |
| Formulas and Constants, the docking approach step −5 … 5 and the 3-tick stage start | built, tested (new) | `docking_step`, `enter`; `claim_24_docking_steps_are_clamped`. |
| T2, "flying to the pad (*enter*)": lands; *sleep*, *guard* queued (no longer put on its docking position) | followed | `dock` no longer moves it. |
| Edge cases, `quirks.aircraft_released_above_pad` | built, tested (new) | On: the rest tests. Off: `claim_48_quirk_off_settles_at_the_landing_height` (settled at 0, listed, hit, then an ordinary take-off, tracked from it). |
| Edge cases, `quirks.aircraft_corner_sector_untracked`, now naming the released aircraft | built | `sector_update`'s condition includes it. |
| Import mapping, `DockingOffset` from art | followed | The importer already read it from art (`crates/convert/src/sim_econ.rs`; `sim_econ_tests.rs`). |
| AC14 | tested on synthetic data | `claim_24_docking_steps_are_clamped`: displacement 64, 64, 0. The stock pad copy and its recording are the lead's. |
| AC30 part k (1), docking at the raised pad | tested on synthetic data | `claim_24_docking_at_a_raised_pad_rests_at_the_landing_height`. |
| AC30 part k (2), the release | tested on synthetic data | `claim_48_release_rests_above_its_pad`. The render layer is tested in the renderer test. |
| AC30 part k (3), leaving far | tested on synthetic data | `claim_48_leaving_far`. |
| AC30 part k (4), leaving near | tested on synthetic data | `claim_48_leaving_near` (a destination 512 to 767 leptons away, slowdown distance 2000, target fraction below 1, climb in the second update). |
| AC30 part k (5), the airborne scan | by construction | The scan's ground-layer test is `temporal::ground_layer` (the height's layer), never `ground_filed`, and the aircraft is tracked from its second update. No synthetic defence test. |
| Changes for other specs, S-0309 claims 7 and 9 | followed | The pass reads the filed list (`ground_filed`), the scan reads the height's layer. |
| Changes for other specs, S-0404 claim 15 | followed | The docking position's height is applied only at the factory release (`released_docked`). Depot, lift and ownership callers of `pad_position_at` are unchanged. |
| Changes for other specs, S-0303 Terms (the released aircraft among the start events) | followed | As claim 47 above. |
| Claim 47 (b) of intake 16, "interim, Q-1841" | retired | Updated in [intake16-S-0501.md](intake16-S-0501.md). |

## Requests

Q-1875 was filed (block Q-1875 to Q-1879). It asks where the object-present mark of an aircraft
released above its pad is, and when it is cleared. Interim: the aircraft keeps its ground
placement's mark, moved to its docking cell, until its first movement update toward a destination.
Q-1876 to Q-1879 are unused. Q-1841 was answered, and its interim is retired.
