# S-0501 r45: claim ledger, M8 step A3 intake 16

This intake takes in one spec revision, which answers one request, so one interim is retired:
Q-1845 (when an aircraft is a tracked flying object).

- **S-0501 revision 45** (spec commit fc4554c; Q-1845 answered in af5bba1). It adds claim 47,
  the events that start and end an aircraft's tracking as a flying object (S-0303 Terms) and
  that bring its sector up to date, with the quirk switch
  `quirks.aircraft_corner_sector_untracked`, the counts-as-aircraft row of Import mapping and
  scenario AC30. No other claim changes. Changes for other specs names S-0303 Terms and Import
  mapping, S-0105 Import mapping, and S-0004 open question 2 and claim 11. They are the other specs'
  to make; below says which of them our code already follows.

The switch was already on main (ad4ef5d). It is now consulted, and its doc comment (in
`quirks.rs` and the ruleset schema) no longer says "Not consulted yet". No other quirk is named.

**Code.**
- `air.rs`: the Q-1845 interim (tracked exactly while on the map and in flight, checked after
  every movement update) is retired. `sync_listing` keeps only the cell listing (S-0209 claim
  13), and ends tracking when the aircraft is off the map. Tracking now changes only at claim
  47's events:
  - `take_off` starts it, before the climb, at the height the aircraft has (a). Leaving a pad
    goes through `take_off` too.
  - `placed` starts it when the in-the-air answer is yes (at least 208 leptons) and the type's
    counts-as-aircraft flag is set (a).
  - `touchdown` (accepted) ends it, after the height is set; `impact` ends it before the
    impact's own effects; `released_docked` ends it (a landed aircraft on a pad is never
    tracked); `leaving_map` and the off-map case of `sync_listing` end it (b). Removal already
    unregisters through `spawn::removing`.
  - `sector_update`, at the start of `tick_flight` before the crash block and the forward step
    (c): nothing when the carried speed fraction is 0. A tracked aircraft calls
    `flying::moved`, which moves it to the end of its new sector's list only when its sector
    changed. An untracked aircraft in flight starts tracking, except, with the quirk on, while
    its cell lies in sector (0, 0) of F7.
- `econ::tech`: the counts-as-aircraft flag left out of an aircraft type's section now reads
  *yes* (S-0501 Import mapping; S-0004 open question 2 defers to it). Infantry and vehicle types
  still read *no*. This also changes S-0004 claim 11's sidebar rank: such an aircraft ranks as
  air.
- No new state: the tracked list and its sectors were already hashed and saved, and claim 47
  needs no stored "last cell" because a sector change implies a cell change. Save format stays
  76, AI game format 37.

**Tests.** `crates/deterministic/sim/tests/claims_s0501_tracking.rs`, new, with synthetic types
APAD (an aircraft factory and pad), SCOUT (landable), HOVER (not landable), FLAGOFF (the flag
set to no) and LOW (a cruise height below 208 leptons). In `claims_s0501.rs`,
`claim_47_tracked_from_the_pad_to_the_pad` and
`claim_47_a_launched_slot_aircraft_is_tracked_from_its_take_off` are new. In `claims_s0309_airborne.rs`,
`aircraft_is_tracked_only_in_flight` is replaced by
`aircraft_is_tracked_from_take_off_to_touchdown`, and
`anti_air_building_ignores_a_landed_aircraft_until_it_leaves_the_ground` is new. In
`claims_s0411.rs`, `absent_counts_as_aircraft_ranks_an_aircraft_as_air` is new.

| Claim or item | Status | Notes |
|---|---|---|
| S-0501 claim 47 (a), start at the take-off start | built, tested (new) | `claim_47_tracking_starts_with_the_take_off`: tracked after the take-off update, listed once. In our engine the take-off and the first climb step fall in the same movement update, so the dual membership of S-0303 claim 6 (tracked and listed at once) lasts only inside that update; a search between ticks never sees it. `claim_47_pad_departure` covers the pad, and `claim_47_tracked_from_the_pad_to_the_pad` (in `claims_s0501.rs`, an armed type that spends its rounds and returns) the whole round trip. |
| Claim 47 (a), not added a second time | built, tested (new) | `flying::register` adds nothing for an aircraft already tracked. A take-off again from a landing keeps its place (`claim_47_tracking_ends_at_touchdown_not_at_the_landing_start`). |
| Claim 47 (a), start at placement in the air | built, tested (new) | `claim_47_placed_in_the_air_with_the_flag_is_tracked_at_once` (visiting placement at cruise height); with the key left out, in sector (0, 0), the control in `claim_47_corner_sector_quirk_both_ways` (AC30 h (2)). A slot aircraft launched on the ground by its carrier is first tracked at its take-off: `claim_47_a_launched_slot_aircraft_is_tracked_from_its_take_off`, which also checks the end on going back inside. |
| Claim 47 (a), the ignored-destination list | partly (during the effect) | The paralysis hold, erasure, warp and a switched-off locomotor skip the movement update that would accept the destination, and a vehicle-disabling effect refuses the order, so no take-off starts and no tracking. What happens to a destination held through the effect once it ends is not stated, and no rule is invented for it. Built by those existing holds; no test puts a landed aircraft under one of the effects. |
| Claim 47 (b), end at an accepted touchdown, not at the landing start | built, tested (new) | `claim_47_tracking_ends_at_touchdown_not_at_the_landing_start`; on a pad in `claim_47_tracked_from_the_pad_to_the_pad`; every tick of a flight in `aircraft_is_tracked_from_take_off_to_touchdown`, and of six aircraft over four rounds of crossing move orders in `claim_47_never_tracked_on_the_ground_always_in_flight`. |
| Claim 47 (b), an aircraft docked at a positive docking height | interim retired at intake 17 (Q-1841 answered by S-0501 r52) | Intake 17 ([intake17-S-0501.md](intake17-S-0501.md)): docking by landing never applies the docking height (claim 24), and a factory release above the pad follows claim 48. The interim text was: Claim 47 (b) says a landed aircraft on a pad is listed in its cell layer instead, and AC30 i (2) expects one hit from the scanned cell; S-0209 claim 13 lists no aircraft with a positive listing height, which a pad's docking displacement can give. The code follows S-0209 claim 13: such an aircraft is in neither list. The synthetic pads dock at height 0, so no test meets the case. |
| Claim 47 (b), a refused touchdown keeps it | built | Only an accepted touchdown ends tracking. Not forced from a test (the spec's AC30 has no fixture for it either). |
| Claim 47 (b), end at a crash impact; a dead falling aircraft stays tracked | built, tested (new) | `claim_47_a_falling_dead_aircraft_stays_tracked` (tracked while falling, untracked after the impact). |
| Claim 47 (b), end on leaving the map | built, tested (new) | `claim_47_going_inside_ends_tracking` (`objects::take_inside`); warp and removal through the existing paths. |
| Claim 47 (b), nothing else ends it: hovering, waiting, fraction 0 | built, tested (new) | `claim_47_hovering_stays_tracked` (AC30 b). |
| Claim 47 (c), sector update one update late | built, tested (new) | `claim_47_the_sector_follows_one_update_late` (a map with 2-cell sectors). |
| Claim 47 (c), no update at a carried fraction of 0 | built, tested (new) | `claim_47_no_sector_update_at_a_zero_fraction`. |
| Claim 47 (c), an untracked aircraft in flight joins at its first update | built, tested (new) | `claim_47_placed_in_the_air_without_the_flag_or_low_joins_at_its_first_update` (the flag off; and the low cruise height). |
| Edge cases, `quirks.aircraft_corner_sector_untracked` | built, tested (new) | `claim_47_corner_sector_quirk_both_ways`: on, an untracked aircraft in sector (0, 0) joins no list until it moves into another sector; off, it joins sector (0, 0)'s list. |
| Claim 47 (d), order of the events | built, tested (new) | No sorting; aircraft join in the order of their movement updates within the tick. `claim_47_same_tick_starts_keep_the_update_order`. |
| Claim 47 (e), the anti-air consequence | built, tested | The airborne scan already skips objects in the ground render layer (`acquire.rs`). `anti_air_building_ignores_a_landed_aircraft_until_it_leaves_the_ground` (AC30 d); the three engage tests of A4 round 3 (AC30 a) still pass. |
| Import mapping, `ConsideredAircraft` absent: yes for an aircraft type | built, tested (new) | `econ::tech`; `absent_counts_as_aircraft_ranks_an_aircraft_as_air` (AC30 j, with its control). |
| Scenario AC30 | partly | Parts a, b, d, j and the tracking state behind c, f, g, h and i (1) and (3) are covered by synthetic tests; i (2) waited for Q-1841 when a pad docks above the surface (answered; part k is taken in at intake 17); the timings and stock types are the lead's to record. |
| Changes for other specs, S-0303 Terms and Import mapping | followed | Our S-0303 code takes the events from claim 47. S-0303's Import mapping row still gives *no* as the absent default; the lead may wish to correct it. |
| Changes for other specs, S-0105 Import mapping (shadow) | not followed | The client's shadow rule (`crates/app/src/mapview/build.rs`) reads the key with S-0105's absent *no*. Left until S-0105 is revised. |
| Changes for other specs, S-0004 open question 2, claim 11 | followed | The sidebar rank reads the new default. |
