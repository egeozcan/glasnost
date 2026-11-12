# M8 A3 intake 124: S-0217 (new) and S-0508 r39 (Q-2951 answered)

Brief: `.claude/worktrees/M8-A3-INTAKE124-BRIEF.md`.

**S-0217** r20, new (commit 5a39a6d): the house-wide hunt order and the computer player's periodic
checks, original only. **S-0508** r19 → r39 (commit 4fc7e34). Both answer **Q-2951** (e7820bf,
c81fd46), which intake 123 filed. The S-0508 delta: claim 4's *No hunt after a refusal* now names the
owner's own checks as a source of *hunt*; claim 13's house-wide hunt row names S-0217 claim 1 and
its checks; scenario G21 is rewritten around the checks; the Open questions entry on Q-2951. Neither
delta adds a quirk name (no `quirks.` token; `every_spec_quirk_has_a_switch` passes).

The measurement records M-0004, M-0005, M-0007, M-0008, M-0013, M-0015, M-0016, M-0020, M-0021,
M-0024 and M-0025 were revised (2d4c965 to d80a644). Read diff by diff: the changes move values to
`committed` visibility or say what a record does not report. None changes a value or an outcome
that the arena oracle reads. Two code comments that quoted the old wording were brought in line:
`crates/tools/src/oracle_m0024.rs` (the record does not report the counts and ticks) and
`crates/tools/src/oracle_m0025.rs` (three "kept local" notes now read "not reported").

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## What changed in the code

- **The house-wide hunt order (S-0217 claim 1).** New `computer::hunt_order`: every infantry unit,
  vehicle and aircraft of the house on the map, not riding a transport, has *hunt* queued without
  an immediate start (`air::queue`, S-0215 claim 7, with S-0501 claim 34's row for aircraft); then
  `garrison::hunt_order` evicts the occupants of the house's buildings with hunting and without
  inside fallback (S-0508 claims 11 and 13). `garrison::hunt_order` now takes the buildings in
  reverse creation order and only those the house owns at that moment. Map action code 6
  (`triggers.rs`) now calls `computer::hunt_order`: before, it reached only garrisons.
- **The checks (S-0217 claims 2 to 4).** New `computer::house_update`, called in a house's update
  (S-0203 row 7, `econ/houses.rs`) right after the raise of its tags (claim 2's engine rule). Only a
  test-only computer-controlled player house makes checks: a player house whose `human` flag is
  clear and that was not removed (S-0215 Terms). Our AI houses and the player are `human`, so our
  AI is unchanged. A check at tick *t*: held when *t* ≤ *L* + 900; not held and no factory building
  on the map gives every own building on the map with health left a sell order in the order of
  ownership (`combat.acquired`), then the order; a house with no building other than occupied
  civilian buildings and no construction vehicle gets the order at every check. Then one R(1, 7)
  draw sets the next check to *t* + 105 + *R*.
- **The last base hit tick *L* (claim 3).** `computer::building_hit`, called from
  `combat::damage::apply_hit` after the health change, sets the owner's *L* when the hit has an
  attacker that is an object in the world and the building is not one that undeploys into a vehicle
  and covers one cell. A building's own damage without the self-damage mark is already stopped by
  S-0302 claim 8's first stage. Which other hits count is open: **Q-2986** (below).
- **State.** `House::checks` (`computer::Checks { next, last_base_hit }`), both 0 at game start, saved
  with the house.

Formats: save 128 → **129**, AI game 95 → **97** (every game's draws change: each test-only house
draws at tick 0 and every 106 to 112 ticks after, and the new field is in every state hash). Goldens:
regenerated on the final tree (see the report).

Ten older sim tests used computer-controlled player houses with no building, which claim 4 now sends
hunting at tick 0. Each holds that house's next check past the test's ticks
(`w.houses[i].checks.next = i32::MAX`, cited to S-0217 claims 2 and 4): `claims_orders`,
`claims_s0210_r29`, `claims_s0306_modes`, `claims_s0215_scatter_damage` (two),
`claims_s0402_slaves`, `claims_s0508_r19`.

## Ledger

| Claim part | Status | Where, tests |
|---|---|---|
| S-0217 1: each mobile actor on the map has *hunt* queued without an immediate start; other houses' actors are not reached | tested | `computer::hunt_order`; `claims_s0217::claim_1_mobile_actors_have_hunt_queued_without_a_start` |
| S-0217 1: the actor leaves its team | no change | our engine has no teams (S-0215 Terms) |
| S-0217 1: units inside a transport are not reached | tested in code, no test | `transport::riding` filter; no fixture has a transport |
| S-0217 1: occupants come out hunting, without inside fallback, only through a building the house owns at that moment | tested | `garrison::hunt_order`; `claim_1_occupants_come_out_hunting_only_from_an_owned_building` |
| S-0217 1: buildings in reverse creation order; the later one's occupants take the earlier draws | tested | `claim_1_the_building_created_later_is_evicted_first` (state hash against the other order) |
| S-0217 1 and S-0011 claim 23: map action code 6 gives the order, units outside garrisons included | tested (changed) | `triggers.rs` action 6; `claim_1_map_action_6_reaches_units_outside_garrisons` |
| S-0217 2: a check at tick 0, then every 106 to 112 ticks; one R(1, 7) draw at the end of the check | tested | `computer::house_update`; `claim_2_one_draw_sets_the_next_check`, `claim_2_checks_from_tick_0_every_106_to_112_ticks` |
| S-0217 2: human-controlled houses (our AI's among them), the neutral and the special house make no checks | tested | `computer::makes_checks`; `claim_2_no_check_for_other_houses` |
| S-0217 2: the check after the raise of the house's tags; row 7's house order | built, no own test | call site in `econ/houses.rs`; row 7's loop order as built |
| S-0217 3: *L* is set by a hit with an attacker, whatever the attacker's house | tested (Q-2986 interim) | `computer::building_hit`; `claim_3_a_hit_with_an_attacker_sets_the_last_base_hit_tick` |
| S-0217 3 and Import mapping: a one-cell building that undeploys into a vehicle; the building's own damage only with the self-damage mark | tested | `claim_3_vehicle_like_and_self_damage` |
| S-0217 3: not held, no factory: sell orders in the order of ownership, then the order | tested | `computer::sell_all`; `claim_3_sale_then_order_when_not_held` |
| S-0217 3: held at *t* ≤ *L* + 900; held at every check up to tick 900 | tested | `claim_3_held_until_l_plus_900`, `claim_3_held_up_to_tick_900` |
| S-0217 3: a factory building of any kind stops the sale and the order | tested | `claim_3_a_factory_building_stops_both` |
| S-0217 4: no building and no construction vehicle: the order at every check, held or not | tested | `claim_4_no_building_gives_the_order_while_held` |
| S-0217 4: a construction vehicle stops claim 4's order | tested | `claim_4_a_construction_vehicle_stops_the_order_while_held` |
| S-0217 4: occupied civilian buildings do not count; occupiers come out at a held check | tested | `claim_4_occupiers_come_out_at_a_held_check` |
| S-0217 4 with 3: not held, the occupied civilian building gets the sell order first | tested | `claim_4_occupied_civilian_building_sold_when_not_held` |
| S-0508 4 (r39) and S-0217 4: a refused unit has *hunt* queued at its house's next check | tested | `claim_4_a_refused_unit_has_hunt_queued_at_the_next_check` |
| S-0508 13 (r39): the house-wide hunt row names S-0217 claim 1 and its checks | tested | as S-0217 claim 1 |
| S-0508 G21 (r39) and S-0217 H1 to H3 | not run (oracle scenarios) | the lead's |
| S-0217 Open questions: the money step, a check against other draws of the update, which hits set *L*, the order's reach to a walker's queued *enter*, the units the original's order destroys | seam (spec open) | Q-2986 asks the hits; the rest are left as the spec leaves them |

Requests: **Q-2986** (which hits set *L*), **Q-2987** (the arena rows, below), **Q-2988** (M-0024's
row on an approach destination left when the target dies in the same tick, below). Q-2989 to
Q-2995 are unused.

Later (sim issues S3, [sim-issues.md](sim-issues.md), *Batch S3*): S-0217 r22 answered Q-2986. Every
hit with an attacker that reaches a building sets *L*, whatever the hit then does, so the interim
"lowers its health" above is gone (`claim_3_hits_that_change_no_health_set_l`). S-0309 r133 and the
M-0024 revision answered Q-2987 and Q-2988's questions 1 and 2; Q-2988 stays open for question 3.

## Play-fixes layer

The checks reach only test-only computer-controlled player houses: the arena's and the oracle
lanes'. Our AI houses and the player are human-controlled, so no normal game gets a check, and no
design switch is added (brief). Claim 1 is our engine's rule for every house that receives the
order; in a normal game only map action code 6 gives it.

## Arena

Base main 2831b4e against this branch, `arena --record all --threads 2` (12 games per pairing,
seeds 0 to 5 on each side):

| Record | Failing before | Failing after |
|---|---|---|
| M-0001 | 14 of 96 | 11 of 96 |
| M-0002 | 5 of 24 | 4 of 24 |
| M-0003 | 2 of 35 | 2 of 35 |
| M-0019 | 0 of 6 | 0 of 6 |
| M-0020 | 1 of 25 | 0 of 25 |
| M-0021 | 11 of 50 | 10 of 50 |

Every arena game changes from tick 0: both houses now make a check in their house update of tick 0,
and its delay draw moves the scenario stream for every later draw. So rows far from the bunkers
move too, within noise. In M-0001, LTNK–APOC, LTNK–E1, YTNK–E1, TELE–E1, TELE–E2 and CAOS–APOC now
pass; MIND–E1, CAOS–V3, CAOS–E1, CAOS–E2 and DISK–E2 now fail; CAOS–TTNK, DISK–FV, TELE–BFRT,
TELE–ROBO, TELE–APOC and TELE–V3 still fail. In M-0002, YURI–SHK passes; INIT–E1 (side A now),
INIT–E2, YURI–E1 and YURI–E2 still fail. M-0020's one row (part 2, YURI against SHK) passes. In
M-0021, one part B row passes; the others still fail.

### Q-2951's rows, 64 seeds (0 to 63) on the side named

| A6b row | Pairing, side | Record | Before | After | Verdict |
|---|---|---|---|---|---|
| 10 | M-0001 TELE–E2, B | col | none 64 | col 64 | fixed |
| 18 | M-0002 INIT–E1, B | row | col 54, none 10 | row 63, col 1 | fixed |
| 20 | M-0002 YURI–E1, B | col | row 46, col 16, none 2 | row 63, col 1 | worse; **Q-2987** |
| 9 | M-0001 TELE–E1, B | col | col 31, none 33 | col 64 | fixed |
| 21 | M-0002 YURI–E2, B | col | col 36, row 26, none 2 | row 49, col 15 | worse, now systematic; **Q-2987** |
| 4 | M-0001 YTNK–E1, B | row | row 41, col 23 | row 63, col 1 | fixed |

### Rows newly failing, 64 seeds (a row is within noise when our share *p* gives *p*⁶ ≥ 0.01)

| Pairing, side | Record | After | Verdict |
|---|---|---|---|
| M-0001 CAOS–E1, B | col | row 57, col 5, none 2 | systematic; **Q-2987** |
| M-0001 CAOS–E1, A | col | row 58, col 6 (A6b: col 21, row 32, none 11) | systematic before and after; **Q-2987** |
| M-0001 CAOS–E2, B | col | row 53, col 10, none 1 | systematic; **Q-2987** |
| M-0001 CAOS–TTNK, A | none | none 63, row 1 | within noise |
| M-0001 CAOS–TTNK, B | none | none 48, row 16 (*p*⁶ = 0.18) | within noise |
| M-0001 MIND–E1, A | row | row 60, col 2, none 2 | within noise |
| M-0001 DISK–E2, B | row | row 60, none 4 | within noise |
| M-0002 INIT–E1, A | row | row 38, col 24, none 2 (*p*⁶ = 0.044; before: row 62, none 2) | within noise, but moved; named in Q-2987 |
| M-0001 CAOS–V3 (win test; M-0016's stock rows: row 2 of 6 on A, 5 of 6 on B) | – | A: row 13, col 1, none 50; B: row 16, col 3, none 45 (before, B: row 6, col 1, none 57) | side B was already far from stock before; the win test flips with the 12 games' draws. Q-2159's pairing, not this intake's |

**The mechanism (trace, M-0001 CAOS–E1, side B, seed 0).** Before: E1 won at tick 304. After: the
first difference is at tick 0 (the checks' draws), and the first behavioural difference that
decides the game is the E1 house's first check after tick 0, at tick 111. The E1 that entered
bunkers #258 and #259 in *capture* (from tick 89) are put out with *hunt* next (S-0217 claims 1 and
4, S-0508 claim 13). At tick 112 *hunt* is current and gives A#438 and the others the bunker they
left as target; they go for it in *capture* again, meet the CAOS gas outside, turn berserk and
shoot one another. CAOS wins at tick 912. M-0002 YURI–E1, side B, seed 0, plays the same way: the
occupiers come out at tick 110, and YURI wins at tick 912 holding 20 units. Every step is a claim
as built, so here the spec and the records disagree. Under PROVENANCE the measurement wins:
**Q-2987** names the rows and asks what the original's occupiers do at their house's checks. That
part stops there; nothing was tuned toward the records.

**Oracle lanes** (`cargo test --release -p glasnost-tools --test arena m002`): M-0023 skipped (no
local values) before and after; M-0025 0 of 6 failing before and after; M-0020 passes now (it failed
on the YURI–SHK row before); M-0021 fails as before (its 10 of 50 above); M-0024 fails 1 of 7 before
and after, but a different row. Before: *Part f: every judged hit naming no attacker classed
neighbouring destination*. After: that row holds, and *Parts f and fc: un-hit free vehicles classed
neighbouring destination: 0* fails on one vehicle-tick. A temporary diagnostic, not committed, found
it: part f, F on B, seed 0, tick 633. That is the run's last tick, when the last F vehicle dies.
An un-hit G vehicle in *hunt* with no target gains a destination in a neighbouring cell. No check
runs at that tick (both houses' next checks are at 652 and 653), no hit lands on it, and nothing is
queued for it. The record's houses are computer-controlled with no building, so their checks run
there too, but on vehicles already in *hunt* with nothing queued they change nothing (S-0215 claim
7). The run reaches this tick because the checks' draws move every later draw. A second
diagnostic, also not committed, traced the vehicle over ticks 628 to 633. In that tick, its *hunt*
processing takes the last F vehicle as target (S-0215 claim 27), and its approach step finds an
approach position in the next cell (S-0215 claim 37 part f, S-0307). Then another G vehicle
destroys the target, and S-0309 claim 27 drops the target but not the destination. So the
claims as built produce the row, and the record states 0 such rows (exact): **Q-2988** asks
whether the original keeps such an approach destination. That part stops there.

Figures: `local-lane/m8/i124/arena-base.out`, `arena-after.out`, `base-sr-*.out`,
`after-sr-*.out`, `base-trace-caos-e1-b0.out`, `after-trace-caos-e1-b0.out`,
`after-trace-yuri-e1-b0.out`, `base-oracles-m0023-25.out`, `after-oracles-m0023-25.out`,
`diag-m0024.out`, `diag-m0024-b.out`.
