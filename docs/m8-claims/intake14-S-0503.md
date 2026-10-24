# S-0503 r27: claim ledger, M8 step A3 intake 14

This intake takes in one spec revision, which answers one request, so one interim is retired:
Q-1773 (a hover's stored path after a deploy order).

- **S-0503 revision 27** (spec commit 4763d7c; Q-1773 answered in 2d77c7a). Against revision 16
  it adds claim 19, the deploy order of a moving hover, with scenario H16 and two quirk
  switches. No claim of revision 16 changes. The Summary lists corrections to S-0213 claim 10,
  S-0415 claim 2 and S-0517 claims 4 and 9. They are the other specs' to make, and claim 19
  states each of them for a hover.

Both switches were already on main (7e83a23): `quirks.hover_landing_request_stands` and
`quirks.hover_deployed_keeps_moving`. They are now consulted, and their doc comments (in
`quirks.rs` and the ruleset schema) no longer say "Not consulted yet". No other quirk is named.
`every_spec_quirk_has_a_switch` passes.

**Code.**
- `orders.rs`, the deploy order: when the order clears a hover's nonempty order-level
  destination, it makes `hover::stop_order`'s point-equality request (claim 16). The stored path
  goes when *D* ≠ *P* and stays when they are equal (`hover::keeps_path`). The Q-1773 interim
  (a hover always kept its stored path) is retired. When the order-level destination is already
  empty, nothing of the hover's movement is touched.
- `jump::deploy_clear_rejected`: S-0213 claim 10's balloon-hover retention, now also for a
  hovering type with the balloon-hover field (the deploy order only). Its rows are
  `jump::retention_rejects`, taken out of `clear_rejected` without change.
- `process::moving`, readiness (S-0215 claim 11): a vehicle whose current locomotor is
  hovering reports `hover::moving_now` (a point and throttle ≠ 0). Before this intake it
  reported `occupancy::in_progress`.
- `econ::deploy::attempt`, S-0415 claim 2's moving rule: a hover is moving while it holds *D*
  or *P* (`hover::moving`), whatever its throttle.
- `econ::placement::ignored_claims`: the deploying vehicle's own hover claims in the cell
  (ground layer) are ignored, as its driving claims already were. **Bug fix.** Before this
  intake, a hover's own vehicle-present flag failed S-0415 claim 3's cell test, so a hover's
  deploy into a building always failed.
- `econ::inplace`: claim 4's height is the hover height for a hover (`airborne`). A
  landing-first hover above the ground makes the landing request (`hover::request_landing`,
  new hashed field `HoverState::landing_request`). While the request stands, a deploy step does
  not deploy it (`quirks.hover_landing_request_stands`). With that switch off, once the hover is
  not moving, the next step treats the request as met (`hover::landing_met`) and takes the
  height-0 rows. A standing request keeps the 1-tick delay.
- `hover::deploying`, called by `inplace::set_form` on becoming deploying or deployed. An
  order-level destination that the locomotor has taken as *D* becomes a retained *D* of the
  locomotor, so S-0517 claim 9's `enforce` (the order-level destination replaced by none) does
  not stop the hover. With `quirks.hover_deployed_keeps_moving` off, becoming deployed drops *D*
  and the stored path and keeps *P*.
- Save format 74 → 75, AI game format 35 → 36 (`HoverState::landing_request`).

**Tests.** `crates/deterministic/sim/tests/claims_s0503_r27.rs`, new. It uses synthetic types:
HOVER, plus the H16 variants HOVA (A), HOVB (B), HOVC (C) and HOVD (D), and HOVE (B with the
balloon-hover field).
- `claim_19_midway_drops_d_and_the_path`
- `claim_19_arrival_tick_and_the_tick_before`
- `claim_19_final_leg_keeps_d`
- `claim_19_repeated_order_on_unchanged_points_changes_nothing`
- `claim_19_no_heading_point_stops_in_its_cell`
- `claim_19_deploy_into_a_building_waits_for_the_stop`
- `claim_19_targetless_attack_makes_unload_current_while_moving`
- `claim_19_readiness_asks_moving_now`
- `claim_19_in_place_by_hover_height`
- `quirk_hover_landing_request_stands` (both positions, plus a save round trip of the request)
- `quirk_hover_deployed_keeps_moving` (both positions)
- `claim_19_balloon_hover_retention_into_a_building`

| Claim or item | Status | Notes |
|---|---|---|
| S-0503 claim 19, table row *D* ≠ *P* (midway) | built, tested (new) | At the order, *D* and the path are dropped, and heading point, throttle, hover speed and ρ are unchanged. The hover stops at the next cell centre, within claim 7's final step. H16 part a. |
| Claim 19, the order on the arrival tick and on the tick before | built, tested (new) | The order applies after the turn pass (S-0203 claim 7). On the arrival tick it sees the new point and goes one cell further; on the tick before, it stops at the point it is reaching. H16 parts b and c. |
| Claim 19, table row *D* = *P* (final leg, equal heights) | built, tested (new) | *D* is kept and the move completes at the destination, from four cells away and from one (H16 parts d and e). |
| Claim 19, a final leg with unequal heights takes the unequal row | built (by the rule) | `stop_order` compares full points. No test lifts *P* by 416 leptons on flat ground. |
| Claim 19, table row with no heading point | built, tested (new) | A synthetic waiting state (*D* present, no point). *D* is cleared and the hover stays in its cell. H16 part g. |
| Claim 19, the suppressed assignment | built, tested (new) | A second order on the next tick, after the first emptied the order-level destination, leaves *D*, path and *P* unchanged. H16 parts f and h. |
| Claim 19, the order on changed points (open) | interim | Our order touches nothing whenever the order-level destination is empty. The spec leaves the case open, and no stock setup reaches it. No request filed: the code needs no answer to go on. |
| Claim 19, balloon-hover retention for a hovering type | built, tested (new) | Variant D: with a target ahead (S-0502 claim 28's direction row), the order keeps the order-level destination, *D* and path. The hover travels to the end of the move, and no attempt is made while it moves. H16 part i. |
| Claim 19, deploying into a building: no attempt while moving, attempt at the stopping cell | built, tested (new) | Variant A. *unload* is queued under *move* while the hover moves now, and the attempt comes once *D* and *P* are absent, at the cell where it stopped. This needed the placement fix (Code). |
| Claim 19, the targetless attack makes *unload* current while moving | built already, tested (new) | S-0215 claim 11's exception. *unload* is current while the hover still moves, and no attempt is made until both points are absent (variant A's attack repeat). |
| Claim 19 and S-0215 claim 11, readiness from the moving-now report | built, tested (new) | A tick with throttle 0 while the heading point is present leaves the hover ready (H16 part j run 3, in its synthetic form). |
| Claim 19, in place: height 0 at the first deploy step | built, tested (new) | Deploys as S-0517 claim 4 gives. The tests put the hover on its surface before each tick, standing in for part j's climb. |
| Claim 19, in place: above 0, not landing-first | built, tested (new) | *guard* replaces the order, and a later height of 0 does not deploy it. With `quirks.airborne_deploy_order_dropped` off, the order is kept and the first step at 0 deploys it. |
| Claim 19, in place: above 0, landing-first (the landing request) | built, tested (new) | `quirks.hover_landing_request_stands` on: it never deploys, even at height 0, and *unload* stays current with a deploy step each tick. Off: once the hover is not moving, the next step deploys it while still above the ground. |
| Claim 19, the deploy step reads the previous turn's hover height | built already | Behaviour processing comes before movement in a vehicle's turn. The tests rely on it (Notes above). |
| Claim 19, a deployed hover finishes the movement it holds | built, tested (new) | `quirks.hover_deployed_keeps_moving` on: it deploys at height 0 mid-move (balloon retention keeps *D*), then reaches the old destination deployed. Off: it stops short, path empty. H16 part j run 2. |
| Claim 19, no movement along the old path afterwards | tested (new) | The midway test runs 300 more ticks after the stop and the hover stays in its cell. The building test's hover deploys at its stopping cell. |
| `quirks.hover_landing_request_stands` | consulted, tested (both) | As the in-place row above. |
| `quirks.hover_deployed_keeps_moving` | consulted, tested (both) | As the deployed-hover row above. |
| H16, parts a to j | tested (engine) in part | Synthetic versions as above. Part g uses a set state in place of the blocked gap. Part j uses a forced height in place of the bank, and run 3 checks readiness only. Oracle runs are the lead's. |
| Summary, S-0213 claim 10 cites claim 19 | no change (other spec) | Built here under claim 19. |
| Summary, S-0415 claim 2's moving rule for a hovering deployer | built | `econ::deploy::attempt` uses S-0503 Terms *moving* for a hover. |
| Summary, S-0517 claim 4's row "height 0, a landing request stands" | built (hover only) | A jumpjet is unchanged, as the Summary says. A jumpjet's request ends on landing, so the new row never applies to it. |
| Summary, S-0517 claim 9's last sentence | built | `hover::deploying`. |
| Q-1773 | answered, interim retired | The `orders.rs` interim and the docs that cited it now cite claim 19. |

## Seams

- A deployed hover's later S-0212 search, for a retained equal *D* that is not completed on
  arrival, runs as for any retained *D*. The spec leaves open whether S-0517 claim 9 reaches it,
  and no stock setup gets there.
- `hover::deploying` turns an order-level destination that the locomotor holds as *D* into a
  retained *D*. An object destination becomes the point of the destination at that moment, and
  later arrivals and searches target that point's cell.
- With `quirks.hover_deployed_keeps_moving` off, *D* is dropped on becoming deployed, not on
  becoming deploying. During the deploying presentation the hover keeps *D* (retained, as
  above).
- The 416-lepton lift of a final leg with unequal heights has no test on flat ground.
- Under `quirks.hover_landing_request_stands` (default on), nothing clears a hover's landing
  request. The jumpjet's request is cleared by its own locomotor rules (S-0502 claim 16's
  re-ascent, a landing, S-0607's seizure and drop). Claim 19 knows no hovering rule that ends
  it, and S-0517 claim 5 and S-0213 claim 10 name no order that ends it. So a landing-first
  in-place hover whose first deploy step found it above the ground never deploys again in that
  game, even after later move, stop or deploy orders. Its later deploy orders make *unload*
  current, and the deploy step returns at once.
- The readiness change (`process::moving`) reaches every queued behaviour of a hover, not
  only *unload*. A hover with a point but throttle 0 (a sharp corner) is now ready, and a
  queued behaviour can become current there. This is the widest change of the intake, and the
  likeliest source of differences in play outside H16.

## Requests

None filed (block Q-1841 to Q-1844 unused). Q-1773 answered and its interim retired. The two
cases claim 19 leaves open (changed points, a deployed hover's later search) are reached by no
stock setup, and the code keeps to the stated rule.

## Measurements

None in this intake.
