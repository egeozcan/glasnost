# S-0314 r88: claim ledger, M8 step A3 intake 30

This intake takes in one spec revision. It answers Q-1876, which intake 18 filed with an interim
([intake18-S-0314.md](intake18-S-0314.md)).

- **S-0314 revision 88** (spec commit ec15057). It covers r78 to r88 against r77, which intake 18
  built (`git diff 0f8d429 ec15057 -- spec/combat/S-0314-healing-repair.md`).
  - Terms, *Settled vehicle*, gains four sub-entries: *Away from a depot*, *Settled away, then at a
    depot*, *Settled at a depot* and *Backing*. A vehicle is **settled at a depot** while it is
    settled and the event that last made it settled was its movement arrival, a settling
    processing, or a disabling of S-0405 claim 20 while it had contact with a depot or while a
    depot occupied its cell. A disabling anywhere else makes it settled but not settled at a depot,
    and reaching a depot's cell later changes nothing by itself.
  - Terms, *What ends it*: a destination received while a depot or tank bunker occupies the cell
    ends being settled only for a vehicle whose locomotor counts as switched off. A vehicle that
    does not hover always counts. A hovering one counts only once it is down: above the ground it is
    powered in S-0503's sense, so a destination it receives then ends nothing, wherever it is. The
    same condition holds for a disabling's guard reset.
  - Claim 21, *Over water, at a naval yard*, and the Edge case *Hovering vehicle lost at a naval
    yard*: with `quirks.docked_hover_water_loss` off, only contact or being settled at a depot
    spares the water test. A hover disabled over open water away from any depot is tested in both
    positions (decided at the delegated sign-off of 2026-10-25; Q-1876 answered).
  - Claim 22 (*Settled vehicles*), the sale passage, *Disabling while docked*'s cleared
    destination, the Import mapping confidence rows and Open questions follow the new end
    condition.
  - Scenario H4 part i: runs 4 and 5 record the height without judging it, and run 7 is new (a hover
    disabled over open water at least 10 cells from any depot).
  - Changes for other specs: S-0503 claim 11's wording (settled at a depot) and S-0405 claim 20
    (a destination in a tank bunker's cell switches a disabled hover on, with the scatter
    exception). Neither has landed in S-0503 or S-0405, so both are seams (below).
  - The rest is wording only ("is no longer" becomes "has stopped", "Confirm" becomes "Check").

**Code.**
- `depot::Settled` (new): `No`, `AtDepot`, `Away`, replacing the `bool` in
  `DepotState::settled` (serde alias `loco_off` kept). `depot::settled` is "not `No`", and
  `depot::settled_at_depot` (new) is `AtDepot`. The arrival and the settling processing
  (`depot::settle`) set `AtDepot`. A disabling (`depot::disabled`) sets `AtDepot` when the vehicle
  has a depot link or a depot (`unit_repair`) occupies its cell, and `Away` otherwise; it
  overwrites, so the event that last made it settled decides.
- `depot::destination_given` returns before any end unless `depot::counts_switched_off` holds: not
  a hover, or a hover that is not powered (`!hover::powered`: switched off and at height 0 or
  below). This covers a person's order, a scatter's destination and the guard reset alike.
- `hover::sink_test`: with the switch off, spared only by `depot::link` or
  `depot::settled_at_depot`. The Q-1876 interim comment is gone.
- Doc text of `quirks.docked_hover_water_loss` and `quirks.depot_loss_strands_hover` in
  `quirks.rs` and the ruleset schema follows r88.
- Save format 80 (was 79) and AI game format 42 (was 41): the saved settled state now has three
  values. `No` and `AtDepot` keep the old bool's bytes.

**Tests.** `claims_s0314_naval_hover.rs` (synthetic lake; types SKIMMER, NAVYARD, PLANT, CENTRE and
the new BUNKER, a 1 × 1 tank bunker) and `claims_s0314_depot.rs` (the SKIMMER at a land depot).
Every test runs on synthetic data.

**Goldens, arena and sweeps.** Before = main 5b83f94, after = this branch.
With the switch on (the default) the change is byte-identical in every run.
- Goldens: unchanged. `cargo xtask gate` passes on all nine; none regenerated. The hashes would
  move if a gate game disabled a vehicle away from a depot (the new `Away` value is saved), so
  none does, and none gives a hover a destination that matters while it sinks at a depot.
- Oracle arena (local lane): the CSVs and oracle reports are byte-identical before and after.
  - M-0001: 31 of 96 pairings failing; 151 of 172 unanimous stock outcomes reproduced.
  - M-0002: 13 of 24 failing; 43 of 47 reproduced.
  - M-0003: 2 of 35 failing; 61 of 64 reproduced.
- M6c, M7a and M7b sweeps 1 to 16 (`GLASNOST_PLAY_FIXES=1`): the outputs are identical before and
  after, so every counter keeps its minimum, median and maximum. M6c's smallest minimum is 4, M7a's
  is 1 (psychic sensor orders and takeover orders), and M7b's smallest drain minimum is 3.
- Outputs in the local-only `local-lane/m8/a3-30/before/` and `after/`.

| Claim or item | Status | Notes |
|---|---|---|
| Overview, the switch's limit to contact or settled at a depot (Q-1876) | followed | As claim 21 below. |
| Terms, *Settled vehicle*, the three ways (arrival, settling processing, a disabling wherever it is) | followed (unchanged) | Intake 6's `settle` and `disabled`. |
| Terms, being settled seen only through its effects, its ends and, with the switch off, the water test; an engine with no such state conforms only at the switch's default | followed | `Settled` is that state; the default reads only `settled`, the switch off reads `settled_at_depot`. |
| Terms, *Settled at a depot*: arrival and settling processing | built, tested (new) | `settle` sets `Settled::AtDepot`. `skimmer_sinking` asserts it after the arrival; the end-to-end naval yard test asserts contact or settled at a depot at the spin's end. |
| Terms, *Settled at a depot*: a disabling in contact with a depot | built, tested (new) | `disabling_in_contact_away_from_the_yard_is_settled_at_a_depot` (both positions): contact by an entry request 10 cells from the vehicle; settled at a depot; switch off, no bomb mark. |
| Terms, *Settled at a depot*: a disabling while a depot occupies its cell, no contact | built, tested (new) | `disabling_in_a_yard_cell_without_contact_is_settled_at_a_depot` (both positions). |
| Terms, *Away from a depot*: a disabling elsewhere, settled but not at a depot | built, tested (new) | `run_7_disabled_over_open_water_is_lost_in_both_positions`, `settled_at_a_depot_then_disabled_away_is_tested` (the last event decides). |
| Terms, *Away from a depot*: only the enabling ends it while it has no contact and no depot or bunker in its cell | tested (new) | `settled_away_on_open_ground_only_the_enabling_ends_it`: down on open ground, a destination ends nothing; the enabling ends it. |
| Terms, *Away from a depot*: in a tank bunker's cell a destination that takes effect while its locomotor counts as switched off ends it and switches a hover on though still disabled | built, tested (new) | `settled_away_in_a_tank_bunker_cell`: while it is still above the ground after the disabling, a destination and a cleared one end nothing; down, it ends and the hover is switched on while still disabled. |
| Terms, *Settled away, then at a depot*: reaching a depot's cell changes nothing by itself | built, tested (new) | `settled_away_then_in_a_yard_cell_is_still_tested` (switch off): the yard comes to stand in its cell during the spin; still settled away; marked. |
| Terms, *Settled away, then at a depot*: a contact gained later and the arrival make it settled at a depot again | followed | Through `settle` at the arrival (claim 17), unchanged code; no new test. |
| Terms, *Settled away, then at a depot*: down in a depot's cell, a destination ends it; a guard reset's disabling makes it settled again on the same tick, and settled at a depot | built | `destination_given` ends it, and `disabled` then finds the depot in its cell (`occupied_by`) and sets `AtDepot`. Covered in part by `disabling_in_a_yard_cell_without_contact_is_settled_at_a_depot`. |
| Terms, a guard reset in a tank bunker's cell | followed (as built at intake 6) | *What ends it* names only a depot for the guard reset, while *Away from a depot* names a depot or tank bunker. `destination_given` serves both and counts the bunker. In that cell the same disabling settles it again (away) on the same tick, so only the hover's switch-on and switch-off inside that tick differ. No request. |
| Terms, *Backing* (the away rule, the bunker end and no end above the ground `unmeasured`, medium; the late arrival at a depot `unmeasured`, low; *Settled at a depot* our engine's term) | followed | No code. |
| Terms, *What ends it*: a destination while a depot or tank bunker occupies the cell, only with the locomotor counting as switched off; a hover only once down | built, tested (new) | `depot::counts_switched_off`. `terms_a_destination_while_a_hover_is_still_above_the_ground_ends_nothing`: above the ground a destination and a cleared one end nothing, the lift stays off and it sinks; down, the destination ends it and switches it on. |
| Terms, *What ends it*: the same condition for a disabling's guard reset | built | Through `destination_given`, which the guard reset's cleared destination reaches. |
| Terms, a hover switched on again with no end of being settled (S-0606's bunker release) | design | `// design:` in `counts_switched_off`: it is powered, so it does not count as switched off. The spec names only "down" and "above the ground". |
| Claim 21, *Over water, at a naval yard*, switch off: only contact or settled at a depot spares it | built, tested | `sink_test`. `quirk_off_settled_without_contact_is_spared` (settled at a depot), `quirk_off_settled_away_without_contact_is_tested` (new). |
| Claim 21: a hover disabled over open water away from any depot is tested in both positions (H4 part i run 7 in our engine) | built, tested (new) | `run_7_disabled_over_open_water_is_lost_in_both_positions`: a control centre switched off over the lake. The disabling adds to its tick exactly the raw values of the switch-off's two ranged draws: measured against a copy of the world, which also shows one raw value on that tick that is not the vehicle's, and a ranged draw can take a third raw value by rejection (S-0202 claim 4). It is settled, not at a depot, with no contact, and it is marked and lost in both positions. Replaces intake 18's `q1876_interim_disabled_far_from_a_depot`. Also in the local lane (`hover-away`, below). |
| Claim 21, *Disabling while docked*: the cleared destination switches on a hover that is down; one above the ground is powered already | built | The same `counts_switched_off` gate. |
| Claim 22, *Settled vehicles*: a move order in the sinking window ends nothing; down, a settled hover stays put; a new destination in the depot's cells ends it | tested (new) | `claim_22_move_order_in_the_sinking_window_then_again_once_down`. The order releases the untethered contact (claim 20) and it stays settled. The same destination again is rejected by S-0207 claim 27 and ends nothing. A new destination ends it and it leaves. |
| Claim 22, the sale: a vehicle in a sold depot's cells stays settled after the removal only if no destination it received ended its being settled (one above the ground does not) | built | Same gate; the sale tests in `claims_s0314_depot.rs` pass unchanged. |
| Edge cases, *Hovering vehicle lost at a naval yard*, the switch's limit to service at a depot | built | As claim 21. |
| Import mapping, confidence rows (the switch's limit; the original's loss away from a depot `measurable`; no end above the ground `unmeasured`, medium) | followed | No code. |
| Scenario H4 part i runs 4 and 5: the height recorded, not judged | followed | No test judged the height there. |
| Scenario H4 part i run 7 | tested in our engine, local lane | The oracle run is the lead's. See *Local lane* below. |
| Open questions, the order in the sinking window | followed, no request | Our hover takes the move while powered (S-0503 claim 4), as intake 6 built. |
| Changes for other specs, S-0503 claim 11 (settled at a depot) | seam | Not landed in S-0503. The code follows S-0314. |
| Changes for other specs, S-0405 claim 20 (a tank bunker's cell switch-on; the scatter exception) | seam | Not landed in S-0405. The switch-on is S-0314's end (`unsettle`). The scatter's own case reads `hover::powered`, so it already follows. |
| Wording-only edits ("has stopped", "Check") | no change | — |
| Save state | built, tested (new) | `settled_where_round_trips`: `AtDepot` and `Away` survive a save and load; the state hash holds. |

## Local lane

New scenes `hover-yard` and `hover-away` in `crates/app/tests/m6b_lane.rs`, each run once with
the switch on and once with it off, on the first offered map. A local rules copy gives the hover
type (found by its locomotor) the powered-unit flag, makes a small building type with a base drain
its control centre (S-0405 claim 13), and sets the switch. The power orders go to that centre. The
traces (cell, hover height, spin, bomb mark, settled, settled at a depot, contact, disabled,
switched off, on every change) and the screens are in the local-only
`local-lane/m8/a3-30/lane/` (`run.txt`).

- `hover-yard` (H4 part i run 6 in our engine). The damaged hover is clicked onto a repairing naval
  yard on water. It docks: in contact, settled at a depot, and down by tick 209. Then come a
  disabling (settled at a depot, with no draw and no spin, since it is down), an enabling (no
  longer settled, and it rises to about 84 leptons) and a second disabling above the water
  (settled at a depot again through its contact, and a spin). The spin ends 5 leptons up.
  - Switch on: bomb mark, and it is lost on the next ticks.
  - Switch off: no mark. It comes down and stays docked until the service ends at tick 700. Then
    it is released and switched on while still disabled (claim 21), and it rises. It is alive at
    the end of the run.
- `hover-away` (H4 part i run 7 in our engine). The hover stands over open water at least 10 cells
  from the map's only depot (measured to the depot's reference cell, not to the nearest cell of its
  foundation), then its centre is switched off. The disabling makes it settled, not
  at a depot, with no contact, and starts a spin. The spin ends at height 0, and in both positions
  it gets the bomb mark and is lost on the next tick.

## Requests

None. Q-1950 to Q-1954 are unused.

## Seams

- S-0503 claim 11 and S-0405 claim 20: S-0314 r88's Changes for other specs, not landed.
- Play: a hover ordered away while it is still sinking at a depot stays settled. With
  `quirks.depot_loss_strands_hover` on (the default), once it is down it stays put until a new
  destination comes while a depot occupies its cell. If it drifts out of the depot's cells
  before it comes down, only an enabling frees it, and a hover that is not disabled has none.
  This is the spec's rule (Terms, claim 22). The play fixes do not change it.
