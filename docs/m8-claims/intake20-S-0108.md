# S-0108 r75: claim ledger, M8 step A3 intake 20

This intake takes in one spec revision. It answers one request, so one interim is retired:
Q-1772 (the slot animations of a building that needs an engineer before and at its ownership
changes).

- **S-0108 revision 75** (spec commit ecb951c; Q-1772 answered in 49fe9cb). It covers r61 to r75
  against r60, which intake 6 built.
  - Claim 13 now reads the powered state as the in-operation result (every row of S-0405 claim 8),
    reacts once to each change, and says what a newly placed or completed building does. The start
    rule reads "not in operation" in place of "unpowered". The end of a pulse re-enables a building
    whatever its switch.
  - New claims 26 (buildings that need an engineer), 27 (a building never powered) and 28 (the
    working animations of S-0405 claim 9, row by row).
  - Claim 10: slots are hidden from the first tick of the build-down, a pack-up likewise, and a
    type with no build-up sprite starts no sale. Claim 11 adds trigger action code 125's grand
    opening and a code-130 replacement opening at its completion. Claims 16 and 19: the per-tick
    frame wins over a hold.
  - `quirks.enable_unfreezes_unpowered` names the pulse end and every ownership change, and its
    off state now reads "only when the building is in operation right after it".
  - Scenarios B11 (pulse run), B14 (pack-up) and the new B22; Changes for other specs (S-0405
    claims 9 and 10, S-0402 Terms and claim 18, S-0515 TB3); two Open questions; the lead's
    question on claim 6 is decided.

**The lead's ruling** (docs/m8-plan.md, S-0108: "a captured tech building, or one that changes
owner, restarts its production slots as in the original, with no quirk switch"). r75 meets it:
every ownership change re-enables the building and its frozen slots run again (claim 26), and the
revision adds no switch for it. The slots *resume* from the frame they held rather than start
again, which is what r75 states as the original's behaviour; in the held-at-creation case the two
show the same frame. `quirks.enable_unfreezes_unpowered` is an older switch, and only governs
whether that happens while the building is not in operation; its default (on) is the ruling's.
No request.

**Code** (client only; the simulation gains one method and no state).
- `mapview/slots.rs`: `SlotDef::power`, `PowerFlags` (*freeze-unpowered* on by default) and
  `Counting` (the first flag on). `read_slot` reads `Powered`, `PoweredLight`, `PoweredEffect`
  and `PoweredSpecial` when a name is non-empty, and not for *pre-production*, *production* and
  *turret*.
- `mapview/slot_play.rs`:
  - `Running::frozen`; `Slots` keeps the disabled state, what the last update saw (owner, switch,
    pulse), the powered state since the grand opening and the outage marks.
  - `Slots::prepare` takes in, before the slots play and before any start, whether the type
    needs power, whether the building is in operation, and the switch.
  - Starts and the damage swap's restarts are frozen by claim 13's start rule
    (`starts_frozen`).
  - `enablings`: an ownership change re-enables; a switch-off and a pulse disable, a switch-on
    and a pulse end re-enable. A disabling freezes every running *freeze-unpowered* slot; a
    re-enabling unfreezes every frozen slot, or with the switch off only when the building is in
    operation.
  - The grand opening of a map-placed building that is not manned is followed by a disabling.
  - `power`: claim 13's columns for a type that needs and draws power, once to each change of
    the in-operation result, with the newly completed building treated as not powered, so one in
    operation gets the return reaction in its first update. The repair depot's simulation-owned
    slots are left out.
- `anim_clock::Player::update_held` (simulation crate): `update` with a hold. A held animation
  still skips its first update and counts down a loop delay, with the start event, but does not
  advance; its clock keeps running. `update` is `update_held(…, false)`.
- `mapview/live.rs`:
  - `update_slots` fills the new `TypeKind` and `State` fields (in operation by
    `econ::power::in_operation`, the switch, `under_pulse`, the owner, manned), calls `prepare`,
    and plays each slot held while it is frozen or its building is being erased.
  - `completed` now reads `econ::placement::seen_behaviour`, so a building placed in play, whose
    *construction* is queued while none is current, does not open until its construction ends.
  - Q-1772's interim is gone: `Live::unmanned_tech` and `redraw`'s `still_map` are removed, and
    `LiveBuilding::slots` is a plain list. A map building with running slots is drawn live from
    the start, whether or not it is manned.
  - `Live::frozen_slots` for tests and lanes.
- `mapview/quirks.rs`: `ViewQuirks::enable_unfreezes_unpowered` (default on), in `named()`; its
  `NOT_IMPLEMENTED` entry and the `docs/development.md` row are removed.
- Notes and docs: `notes::SLOT_ANIMS_LIVE`, the `anims.rs` and `slots.rs` module docs, the rules
  crate's `EXTERNAL_QUIRKS` entry.

**Tests.**
- `slots.rs`: `power_flags`.
- `slot_play.rs`: `a_switch_off_freezes_and_a_switch_on_resumes`,
  `starts_while_disabled_follow_the_start_rule`,
  `an_unmanned_map_building_holds_until_its_owner_changes`,
  `the_enable_quirk_in_both_positions`, `a_pulse_end_resumes_a_switched_off_building`,
  `a_building_first_seen_switched_off_is_disabled` (Q-1890's interim), `the_power_columns`,
  `a_building_never_powered`, `a_delayed_fire_outage`.
- `mapview/anims.rs` (for `anim_clock::Player::update_held`): `a_held_animation_keeps_its_clock`.
- `capture_tests.rs` (live drawing, synthetic HUTX and SLOTL):
  - reworked: `a_captured_map_building_animates` (drawn live from the start, held at its first
    frame, plays after the capture), `a_damaged_unmanned_tech_building_animates_in_the_neutral_colour`
    (the damage swap restarts the slot unfrozen), `a_chipped_unmanned_tech_building_looks_undamaged_and_holds_its_slot`
    (was `..._but_animates`), `an_unmanned_tech_building_holds_its_slots_whatever_its_look` (was
    `..._drawn_live_for_its_look_keeps_its_slots_still`; now also with no act);
  - new: `a_light_unpowered_slot_of_an_unmanned_tech_building_plays`,
    `a_capture_resumes_a_held_slot_from_its_held_frame` (both switch positions; a slot held at a
    frame other than its first by a switch-off resumes from it at a capture, or stays held with
    the switch off), `a_building_placed_in_play_opens_at_its_completion`.

| Claim or item | Status | Notes |
|---|---|---|
| Summary, claims 26 to 28 answer Q-1772 | followed | Q-1772's interim is retired (above). |
| Claim 2, no build-down without a build-up sprite; a pack-up has one | no change | The sale's (S-0408 claim 4, S-0415 claim 9); the body frames are unchanged. |
| Claim 10, hidden from the first tick of the build-down; the pack-up | built, tested | Already so (`slot_draws` in `BuildDown`); `a_sold_buildings_slots_stay_until_its_build_down`, `a_packing_buildings_slots_stay_until_its_build_down`. |
| Claim 10, a type with no build-up sprite starts no sale; a firestorm-wall one is removed | followed | The sale is the simulation's (S-0408 claim 4); no slot rule here. |
| Claim 11, a building placed in play opens at its completion | built, tested (new) | Code change: `completed` reads the seen behaviour, so the queued *construction* counts. `a_building_placed_in_play_opens_at_its_completion`. |
| Claim 11, a code-125 building opens in the action's tick, with its free unit | seam | The simulation has no trigger action 125 (only 130). Nothing to draw. |
| Claim 11, a code-130 replacement opens at the completion of its construction | built | The replacement enters by `begin_construction` (queued *construction*), so the change above covers it. Not tested through the trigger. |
| Claim 13, the counting flag; slots without power-flag keys count *freeze-unpowered* | built, tested (new) | `PowerFlags::counting`, `read_slot`; `power_flags`. The upgrade slots are not played. |
| Claim 13, the powered state is the in-operation result; once to each change; next update | built, tested (new) | `Slots::power` compares `econ::power::in_operation` with the last update's. `the_power_columns`. The reaction is in the same tick's update (the client runs it after the tick). |
| Claim 13, a newly completed building: no loss reaction; in operation, the return reaction in its first update | built, tested (new) | `a_building_never_powered`. |
| Claim 13, the loss and return columns for *freeze*, *light* and *effect*; normal or damaged variant | built, tested (new) | `the_power_columns` (the occupied variant never used). Only for a type that needs and draws power. |
| Claim 13, the delayed-fire *special-1* exceptions | built, tested (new) | `a_delayed_fire_outage`. |
| Claim 13, *super-3* and *super-low-power* | seam | The superweapon slots are not played by the client (claim 18). |
| Claim 13, the start rule (disabled or not in operation, only for a type that needs power) | built, tested (new) | `starts_frozen`; `starts_while_disabled_follow_the_start_rule`. |
| Claim 13, a disabling freezes every running *freeze-unpowered* slot whatever the type | built, tested (new) | `disable`; `a_switch_off_freezes_and_a_switch_on_resumes`. |
| Claim 13, a re-enabling unfreezes; a pulse end re-enables whatever the switch | built, tested (new) | `enable`, `enablings`; `a_pulse_end_resumes_a_switched_off_building`. The pulse itself never comes: `econ::power::under_pulse` is a stub (no pulse weapon in the simulation), so the pulse rows are reachable in unit tests only. |
| Claim 14, special outage and the *low-power* slot | seam | Not played (as before). |
| Claim 16, the silo-fill frame wins over a hold | seam | Silo fill is not played by the client, and no skirmish path fills storage. |
| Claim 19, the turret frame wins over a hold | seam | The turret slot is not played by the client. |
| Claim 26, not manned does not hide; map-placed: disabled from placement, the set-up's *freeze-unpowered* slots hold the frame they were created with, whatever the type; other slots play | built, tested (new) | `an_unmanned_map_building_holds_until_its_owner_changes`; live: `an_unmanned_tech_building_holds_its_slots_whatever_its_look`, `a_light_unpowered_slot_of_an_unmanned_tech_building_plays`. |
| Claim 26, the *guard* order unfreezes nothing | built, tested | No slot start comes from it; the unit test's second update and the live tests' later redraws. |
| Claim 26, the turret and silo slots' per-tick frames win | seam | As claims 16 and 19. |
| Claim 26, placed any other way: not disabled; a needs-power type's slots start frozen | built | Only the map's buildings (`setup`) are disabled at placement; the start rule freezes a needs-power type's slots while it is not manned (not in operation). |
| Claim 26, a code-130 replacement disabled at placement | interim, no request | Open question of the spec. Ours: not disabled (it is not a map building). |
| Claim 26, starts while not manned (the damage swap restarts a non-power type's slots unfrozen) | built, tested | `starts_while_disabled_follow_the_start_rule`; live: `a_damaged_unmanned_tech_building_animates_in_the_neutral_colour`. |
| Claim 26, a pulse on a building not manned | built, tested (new) | `the_enable_quirk_in_both_positions` (unit only: no pulse in the simulation). |
| Claim 26, every ownership change re-enables and resumes from the held frame, keeping the variant, whether or not in operation; first eligible update | built, tested (new) | `an_unmanned_map_building_holds_until_its_owner_changes`; live: `a_captured_map_building_animates`, `a_capture_resumes_a_held_slot_from_its_held_frame` (held at a frame other than its first by a switch-off; the first redraw after the capture shows the next frame). `update_held` keeps the clock running. |
| Claim 26, an erasure is an independent hold | built | `update_slots` holds a slot while `superweapons::being_erased`, separately from `frozen`. No live test of a capture during an erasure (S-0509 refuses it). |
| Claim 26, afterwards: claims 13 and 14 apply to the change in the next update; a short-power capture is no change | built, tested (new) | `the_enable_quirk_in_both_positions` (capture into short power: the slots stay unfrozen with the switch on); the powered state starts "not in operation" at the opening. |
| Claim 27, a building never powered keeps its *light* and *effect* slots | built, tested (new) | `a_building_never_powered`; live: `a_light_unpowered_slot_of_an_unmanned_tech_building_plays`. |
| Claim 28, *switched on* row: switch-off disables, switch-on re-enables | built, tested (new) | `a_switch_off_freezes_and_a_switch_on_resumes`; live: `a_capture_resumes_a_held_slot_from_its_held_frame`. |
| Claim 28, a building placed switched off | interim (our design), Q-1890 | No rule. Our design (docs/m8-plan.md, *Decisions of 2026-10-25*): first seen switched off (or pulsed), it is disabled from then on, so its freeze slots hold as after a switch-off. `a_building_first_seen_switched_off_is_disabled`. |
| Claim 28, same-tick disablings and re-enablings | interim (our design), Q-1890 | Our design: ownership change, then switch, then pulse, each one change; the last decides the state. |
| Claim 28, *not under a pulse* row | built (unit only) | As claim 13. |
| Claim 28, *not building or selling*: the sale changes no slot before its build-down | built, tested | The sale's loss of operation reaches claim 13 only for a type that needs and draws power. The sale tests above (HUTX needs no power). |
| Claim 28, *no blackout* (claim 14), *alive* | seam / followed | Claim 14 is not played; *alive* adds no rule. |
| Edge cases, `quirks.enable_unfreezes_unpowered` | built, tested (new) | `ViewQuirks::enable_unfreezes_unpowered`, consulted in `Slots::enable`. On and off: `the_enable_quirk_in_both_positions`, `a_capture_resumes_a_held_slot_from_its_held_frame`. Off: "until power comes back" applies through claim 13's return column, which only a type that needs and draws power has. |
| Import mapping, the four power flags | built, tested (new) | `power_flags`. *special-outage* is read and not consulted (claim 14). |
| B11 (pulse run), B14 (pack-up), B22 | tested on synthetic data, partly | B22 (a), (b), (d), (k) by the live tests (on HUTX, a type that does not need power); (e), (f), (g) by the unit tests; (c) by the unit test's second change; (h) by the sale tests; (i) and (j) seams (turret, code 125). The lead's runs are theirs. |
| Changes for other specs (S-0405 claims 9 and 10, S-0402, S-0515 TB3) | followed | Spec-side changes; nothing to build here. |
| Open questions (code-125 construction behaviour; code-130 disabled at placement) | noted | No code-125 in the simulation; code 130 as above. |
| Questions for the lead (claim 6's *full* range decided) | no change | Built at intake 6. |

## Requests

Q-1890 was filed (block Q-1890 to Q-1894): the slots of a building the map places switched off,
and same-tick disablings and re-enablings. Interims as in the table. Q-1891 to Q-1894 are unused.
Q-1772 was answered, and its interim is retired.

## Checks

- The simulation gains `anim_clock::Player::update_held` and no state; `update` is unchanged in
  behaviour, so the save format stays 78 and the AI game format 40.
- Oracle arena (before and after on base a32dbdc): the CSVs are byte-identical (M-0001 31 of 96
  pairings failing, 151 of 172 unanimous outcomes reproduced; M-0002 13 of 24, 43 of 47; M-0003
  2 of 35, 61 of 64). The M6c, M7a and M7b seed sweeps 1 to 16 print the same lines (smallest
  minimum M6c 2, M7a 1, M7b drain 3). The sim-gate goldens hold without the update flag; none
  was regenerated.
- Lanes `derrick-look` and `derrick-paths` (`local-lane/m8/a3-20/before` and `after`), map 0: a
  fresh derrick was in the cell pass (1 image set over 24 ticks) and is now drawn live with 1 of
  its 2 slots frozen (9 image sets: its second slot has no power flag on); protection and a
  charge flash 1 image set each before, 8 and 9 after; a 1-point hit 13 before, 9 after (its
  freeze slot stays held); damaged by half and each capture (277 and 263 ticks) unchanged, 0
  slots frozen, the capturer's colour. The slot power flags read: type 0 slot 0 *freeze*, slot
  1 none; type 1 slot 1 *freeze*, slot 2 none.
