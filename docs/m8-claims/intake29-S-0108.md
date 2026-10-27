# S-0108 r81: claim ledger, M8 step A3 intake 29

This intake takes in one spec revision. It answers one request, so one interim is retired:
Q-1890 (the slots of a building the map places switched off, and disablings and re-enablings in
one tick), which intake 20 filed.

- **S-0108 revision 81** (spec commit a9d8cf6; Q-1890 answered in 82bc10c). It covers r76 to r81
  against r75, which intake 20 built (`git diff ecb951c a9d8cf6 -- spec/rendering/S-0108-building-sprites.md`).
  - Claim 13: each disabling and re-enabling acts at once, at its place in the tick (S-0203); the
    lists of disablings and re-enablings, with an ownership change re-enabling only a type that
    needs an engineer; the cases of several in one tick; the building's own power reaction as one
    more event. Backing, confidence and a new open question.
  - Claim 27 also holds for a building placed switched off, until it first comes into operation.
  - Claim 28: the map's placement switched off (only a type with drain or the needs-power flag),
    an ownership change while switched off, and the switch-on.
  - `quirks.enable_unfreezes_unpowered`'s edge-case text, the Import mapping rows for the two map
    fields, a "Map data" paragraph, B22 parts (l) to (o) with their setup and Validity, and two
    reworded open questions.

**The lead's ruling** (docs/m8-plan.md, S-0108: a captured tech building, or one that changes
owner, restarts its production slots). It still holds: the tech buildings of the ruling need an
engineer, and every ownership change of such a type still re-enables it. r81 only narrows the
re-enabling for types that need no engineer, which the ruling does not name. No request.

**Code** (client only; no simulation change).
- `mapview/slot_play.rs`:
  - `TypeKind::needs_engineer`.
  - `enablings`: an ownership change re-enables only a type that needs an engineer; the events
    seen in one update apply in the order ownership change, pulse, switch, each acting at once;
    the power reaction (`power`) comes after them.
  - The opening of a map-placed building that is switched off is followed by a disabling, after
    the set-up and opening starts (claim 28). Q-1890's interim (a building first seen switched off
    or pulsed is disabled) is removed.
- `mapview/live.rs`: `update_slots` fills `needs_engineer` from the type's power entry.
- `mapview/quirks.rs`: `ViewQuirks::enable_unfreezes_unpowered`'s doc names r81's cases.
- The rules crate's `EXTERNAL_QUIRKS` entry for the switch (`crates/deterministic/rules/src/quirks.rs`)
  still reads "S-0108 claims 13 and 26"; it is left so, since this intake does not touch
  `crates/deterministic`. It could name claim 28 at a later simulation intake.

**Tests.**
- `slot_play.rs`, new: `a_building_placed_switched_off_holds_until_its_switch_on` (B22 (l) and
  (m): types D, E and a type with the needs-power flag and no drain),
  `a_needs_engineer_building_placed_switched_off_resumes_at_its_capture`,
  `an_ownership_change_of_another_type_is_no_re_enabling`, `several_in_one_tick`,
  `a_switch_off_close_to_an_ownership_change_or_a_pulse_end` (B22 (n) and (o)). Removed:
  `a_building_first_seen_switched_off_is_disabled` (the interim). The unmanned tests now mark
  their type as needing an engineer.
- `capture_tests.rs` (live drawing, synthetic HUTX): reworked
  `a_capture_resumes_a_held_slot_from_its_held_frame` (a tech type that needs an engineer: a
  capture, a switch-off, a recapture, B22 (k)); new
  `a_capture_of_a_switched_off_building_that_needs_no_engineer_keeps_its_slot_held`,
  `a_map_building_placed_switched_off_holds_its_slot_until_switched_on` (B22 (l): the map entry's
  switched-on field 0, HUTX given a drain; held through a capture, still held in the switch-on
  order's tick, the next frame in the next tick) and
  `a_map_building_that_draws_no_power_is_not_switched_off_by_the_map` (type Z).
- With the needs-engineer gate removed, four of these fail.

| Claim or item | Status | Notes |
|---|---|---|
| Summary, claim 28 answers Q-1890 | followed | Q-1890's interim is retired (above). |
| Claim 13, each disabling and re-enabling acts at once | built, tested (new) | `enablings` applies each event seen as its own change; `several_in_one_tick`. |
| Claim 13, the disablings (switch-off, the map's placement switched off, a pulse's start, the map's unmanned placement) | built, tested | Placement switched off is new (claim 28 row). |
| Claim 13, the re-enablings; an ownership change of a type that needs no engineer is neither | built, tested (new) | `an_ownership_change_of_another_type_is_no_re_enabling`; live: `a_capture_of_a_switched_off_building_that_needs_no_engineer_keeps_its_slot_held`. |
| Claim 13, places in the tick (polled triggers row 1; captures and pulse starts in the turn pass, update order; pulse end before the orders; power orders row 8) | built, our design for what the view cannot see | The view runs once after each tick and sees the tick's net events, not their places. Our order: ownership change, pulse, switch; the power reaction last. This gives the spec's cases that can occur: a switch-off order after an ownership change to its house leaves the slots frozen; a pulse starting in the tick of a trigger ownership change leaves them frozen. The polled-trigger switch-off cases (playing after a pulse end or a capture) cannot occur: no trigger action in the simulation switches a building. The temporal lock's switch (S-0514 claim 4, `temporal.rs::set_switch`) acts in the turn pass and is drawn as a switch. A capture and a pulse start in one turn pass: the later in update order decides, which the view cannot see; ours lets the pulse decide (seam, pulses stubbed). |
| Claim 13, a switch-on refused under a pulse changes nothing | followed | The refusal is the simulation's (S-0405 claim 10); the switch never changes, so the view sees no event. |
| Claim 13, the reaction of a type that needs and draws power as one more event | built, our design | Applied after the events in the same update; where the spec puts it before a later capture in update order, the view cannot tell (seam). |
| Claim 13, backing, confidence, open question (a power order after an ownership change away from its house) | noted | The order's acceptance is the simulation's (S-0214); nothing to draw. |
| Claim 26, re-enabling by an ownership change of a needs-engineer type (wording) | built, tested | As intake 20; the unmanned tests now set the type as needing an engineer. |
| Claim 27, also for a building placed switched off until it first comes into operation | built, tested (new) | The powered state starts "not in operation" at the opening; `a_building_placed_switched_off_holds_until_its_switch_on`. |
| Claim 28, placed switched off by the map: only a type with drain or the needs-power flag; disabled at the end of its placement, after the set-up and opening starts | built, tested (new) | The simulation's `objects` placement applies the field through `econ::power::map_off_applies`; the view disables at the opening of a set-up building whose switch is off. Unit: types D, E, needs-power-only; live: `a_map_building_placed_switched_off_holds_its_slot_until_switched_on`, `a_map_building_that_draws_no_power_is_not_switched_off_by_the_map`. |
| Claim 28, ownership change while switched off: held for a type needing no engineer, resumes for one that needs an engineer | built, tested (new) | `a_needs_engineer_building_placed_switched_off_resumes_at_its_capture`; the live capture tests. |
| Claim 28, switch-on: re-enables at once, held slots resume, no restart, variant kept; first advance next tick | built, tested (new) | `update_held` keeps the clock; the live test sees the held frame in the order's tick and the next in the next tick. |
| Claim 28, switch-on in a short house: stays out of operation, slots keep advancing | built, tested (new) | `a_building_placed_switched_off_holds_until_its_switch_on` (type E); lane `placed-off`. The return column never comes while short. |
| Claim 28, needs-power flag and no drain: no power column | built, tested (new) | Only a type that needs and draws power gets the columns (intake 20). |
| Claim 28, special-power types at the switch-on (claim 14) | seam | Claim 14 is not played. |
| Claim 28, a switch-on refused under a pulse | followed | As claim 13. |
| Edge cases, `quirks.enable_unfreezes_unpowered` | built, tested | Doc updated. On by default: the switch-on in a short house and the needs-engineer capture resume. Off: only when in operation right after (`the_enable_quirk_in_both_positions`). |
| Import mapping, the switched-on and power fields; Map data | followed | Read by the simulation's map import (as before); the view reads `switched_off` from the building. |
| B22 setup, parts (l) to (o), Validity | tested on synthetic data | (l) and (m) by the unit and live tests above; (n) and (o) by `a_switch_off_close_to_an_ownership_change_or_a_pulse_end` (unit only for the pulse: no pulse in the simulation). The lead's runs are theirs. |
| Open questions, reworded ("traced" to "known") | no change | Wording only. |

**Seams.**
- Claim 14's outage and the *low-power* slot, also at a switch-on.
- `econ::power::under_pulse` always answers no until pulses reach the simulation; the pulse rows
  are reachable in unit tests only.
- The per-tick turret and silo frames are not played (claims 16, 19, 26).
- Two toggles of the switch in one tick are not visible to the view (it sees the switch after the
  tick).
- A capture with a power reaction still due from an earlier tick depends on update order, which
  the view does not see.
- A rejoin (`session::replace_game` and the catch-up) collapses the events since the last view
  update into one update.
- The turn loss at a counting removal is S-0109's.

## Requests

None filed (block Q-1945 to Q-1949 unused). Q-1890 was answered, and its interim is retired.

## Checks

- No simulation change: `git diff main -- crates/deterministic crates/sim-gate` is empty, so the
  save format stays 79 and the AI game format 41 (those of the base), and the goldens are
  unchanged; none was regenerated.
- `glasnost-sim-gate` does not depend on `glasnost-app`, and the oracle and arena modules of
  `glasnost-tools` do not use it, so the sweeps and the arena run the same code before and after.
  After runs on base 0510ab7: oracle arena M-0001 31 of 96 pairings failing, 151 of 172 unanimous
  outcomes reproduced; M-0002 13 of 24, 43 of 47; M-0003 2 of 35, 61 of 64 (the test fails while
  the known discrepancies stand); the M6c, M7a and M7b seed sweeps 1 to 16 pass, smallest minima
  M6c 2, M7a 1, M7b drain 3. These are intake 27's numbers.
- The simulation's writers of a building's switch are the map placement (`objects.rs`), the power
  orders (`econ::power`) and the temporal lock (`temporal::set_switch`); no trigger action
  switches a building.
- Lane `placed-off` (new, `local-lane/m8/a3-29/before` and `after`, 200 maps offered): the maps
  carry few entries with the switched-on field 0, none with slot animations, so the lane switches
  off the non-player map buildings the field could switch off before the first tick. On the first
  map with such a building with a slot counting *freeze-unpowered*: before (the interim), the slot
  held from placement but played after a trigger-style ownership change while still switched off;
  after (r81), it holds through the change, holds in the switch-on order's tick, advances from
  the next tick, and keeps advancing in the house made short of power. Lane `derrick-look`: the
  same counts before and after.
