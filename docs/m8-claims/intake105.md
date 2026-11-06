# S-0203 r44 to r50: claim ledger, M8 step A3 intake 105

- **S-0203** r44 to r50 (commit 5534a23, from the r43 that intake 81 built, b520795). r50 answers
  Q-2636, which intake 101 filed from M-0021's rows: how long the original keeps an object after
  its removal and what can still reach it. It adds claim 13 (a leftover is kept to the end of its
  tick; what a leftover is; the defeat sweep reaches leftovers; the shots of a vehicle in the tick
  it is destroyed; our engine's freedom), the Term *leftover*, the State and ordering row of a
  leftover at the tick's end, scenario O18 (parts a to e), the quirk *Defeat sweep on leftovers*
  (`quirks.defeat_sweep_leftovers`, default on), Changes for other specs (the Glossary, S-0414
  claims 5 and 15, S-0312 claims 1 and 32, S-0302 claim 8), Open questions on leftovers with
  health left and leftovers of other kinds, and `measurements: [M-0021]`.

Requests filed (commit d07f0fe): Q-2691 (whether a sweep reaches the leftovers an earlier sweep
of the same house update, or its own death weapons, made), Q-2692 (a leftover vehicle that meets
S-0312's sinking or crash conditions at the sweep's hit) and Q-2693 (whether leftovers that take no
hit hold places in S-0414 claim 15's cancellation skip, in each position of the switch). Q-2694 to
Q-2700 unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

- **The leftover store** (`glasnost_sim::leftover`, `World::leftovers`). `objects::remove` keeps
  every infantry unit, vehicle, aircraft and building it takes out of the game, as it was when it
  left, off the map, with its owner and health (claim 13 (a), (b)). Projectiles and animations are
  not objects of `w.objects` in our engine and nothing reaches their leftovers, so none is kept.
  `death::leave_play` marks a leftover that left play at its own death; a passenger killed with
  its carrier, a crushed or erased object and any other removal stay unmarked. The immediate
  deletion of a production item (`production::delete_item_object`) keeps none (claim 13 (a)), and
  neither does the arena's *Removal before the groups* (`objects::take_out_quietly`, S-0216 claim
  8), which runs before play. `tick::step_with` empties the store before row 1, so a leftover of
  tick *t* is gone in every part of *t* + 1.
- **Nothing else finds a leftover** (claim 13 (e)). The store is read only by the sweep and, outside
  the simulation, by the arena's recorder (`leftover::object`). Every hold, target and count is
  dropped at the removal as before (S-0309, S-0414 claim 4), so claim 13 (b)'s consequences are the
  removal's.
- **The sweep** (`econ::victory::destroy_all`). It walks the house's objects in play and the tick's
  leftovers together in creation order (S-0414 claim 15). At a leftover's place it counts the
  cancellation skip as at any place, then calls `leftover::sweep_hit`, with the skip tracking of an
  ordinary hit around it. Every leftover of the four kinds holds its place in both positions of the
  switch (interim Q-2693). Leftovers that an earlier sweep of the same house update made (a
  surrender followed by the defeat) and those that the sweep's own death weapons made at later
  places are reached (interim Q-2691, the literal rule).
- **The sweep's hit** (`leftover::sweep_hit`, claim 13 (c)). Under `quirks.defeat_sweep_leftovers`
  it reaches a leftover vehicle owned by the house that left play at its own death with no health.
  It pushes `Effect::LeftoverHit` (the demolition warhead, the health of none, no attacker) and
  calls `death::repeat`: debris (`death::throw_debris`, S-0312 claim 4, with its draws), the death
  weapon of an exploding vehicle at its last position (claim 6, `projectile::detonate_weapon` with
  the leftover as source and its owner as house), the claim 9 animations (`death::show_animations`),
  no crew (claim 13 with the no-escape marker) and a crate it carries (claim 14,
  `crates::place_death`). It changes no health and calls neither the credit nor the destruction
  raises, so no kill, no loss and no tag. `death::repeat_plan` reads these with the leftover put
  back among the objects for the read alone (`leftover::read_restored`, which gives the reader the
  world for reading only). A leftover that meets S-0312's sinking or crash conditions takes no hit
  (interim Q-2692). Off: no leftover takes a hit.
- **The split death effects.** `death::debris`, `death::vehicle_animations` and
  `crates::death_crate` are now each a plan read from the world and an act (`debris_plan` and
  `throw_debris`, `animation_plan` and `show_animations`, `death_crate_plan` and `place_death`), so
  the first death and the repeat share them. The first death's order and draws are unchanged.
- **Claim 13 (b)'s same-tick defeat** was already the engine's: a house whose last vehicle the turn
  pass destroys fails its defeat test in that tick's house update (row 7).
- **Claim 13 (d)** (shots of a vehicle in the tick it is destroyed) restates claim 6 (a) and S-0302
  claim 1, which the engine already follows: no change.
- **The M-0021 recorder** (`oracle_m0021`). The per-tick rows and the off-map set read a vehicle
  that is no object of the world from the leftover store, and `Effect::LeftoverHit` is recorded as
  a hit with no attacker and the health before and after equal. The Q-2636 seam texts are gone.
- **Presentation.** `Effect::LeftoverHit` draws and sounds nothing of its own (`mapview::live`,
  `glasnost-audio`); the repeated death effects make theirs through their own effects.
- **Formats.** Save format 104 → 105, AI game format 68 → 69 (`World::leftovers`).

## Claims

Tests are in `crates/deterministic/sim/tests/claims_s0203_r50.rs`.

| Claim (r50) | Status | Tests and notes |
|---|---|---|
| 13 (a): kept to the end of the tick, gone in *t* + 1 | tested | `o18_a_the_last_vehicle_destroyed_in_the_turn_pass` (listed after *t*, gone after *t* + 1), `o18_b_no_hit_on_a_vehicle_destroyed_in_the_tick_before` |
| 13 (a): other kinds and causes kept | partly | infantry and vehicles removed with health left are kept and take no hit (`o18_e_the_sweep_hit_raises_no_tag_and_counts_nothing`); projectiles and animations are not kept (no reader) |
| 13 (a): the immediate deletion leaves no leftover | no change | `production::delete_item_object` removes from `w.objects` directly |
| 13 (b): off the map, owner and health kept | tested | `o18_a_the_last_vehicle_destroyed_in_the_turn_pass` |
| 13 (b): defeated in the same tick (O18 part a) | tested | `o18_a_the_last_vehicle_destroyed_in_the_turn_pass`; already the engine's |
| 13 (c): the sweep's hit, the demolition warhead, no health (O18 part a) | tested | `o18_a_the_last_vehicle_destroyed_in_the_turn_pass` (both positions) |
| 13 (c): the death effects once more (a second crate) | tested | `o18_a_the_last_vehicle_destroyed_in_the_turn_pass` (two crates on, one off), `o18_e_the_sweep_hit_raises_no_tag_and_counts_nothing` |
| 13 (c): no tag raise, no kill, no loss (O18 part e) | tested | `o18_e_the_sweep_hit_raises_no_tag_and_counts_nothing` |
| 13 (c): no hit on a vehicle destroyed in the tick before (O18 part b) | tested | `o18_b_no_hit_on_a_vehicle_destroyed_in_the_tick_before` |
| 13 (c): no hit to leftovers with health left or of other kinds | tested | `o18_e_the_sweep_hit_raises_no_tag_and_counts_nothing` |
| 13 (c): debris, death weapon and animations repeated | partly | built through the first death's own plan and act; the synthetic tests exercise the crate only |
| 13 (c): a leftover that meets the sinking or crash conditions | seam (interim Q-2692) | no hit |
| 13 (c): leftovers of an earlier sweep in the same house update | seam (interim Q-2691) | reached, the literal rule |
| 13 (c) with S-0414 claim 15: leftover places in the cancellation skip | seam (interim Q-2693) | every leftover holds its place, both positions |
| 13 (d): shots before the destroying hit | no change | claim 6 (a) and S-0302 claim 1 as built |
| 13 (e): our form | tested | `leftovers_survive_a_save_and_load`; the store is read only by the sweep |
| Edge cases: `quirks.defeat_sweep_leftovers` | tested | both positions in `o18_a_the_last_vehicle_destroyed_in_the_turn_pass`; C7 verdict in [c7-play-fixes.md](c7-play-fixes.md) (rejected) |

## Records

Q-2636 touched two M-0021 rows (intake 101 named them) and no M-0020 row. Arena `--record all
--threads 2`, main d9ad36c against this branch on the same base:

| Record | Row | Main | Intake 105 |
|---|---|---|---|
| M-0021 | Reading points: a vehicle destroyed by a hit at *t* has a per-tick row stamped *t*, off the map, with no health left | fails (0 of 123 destroyed vehicles; seam, Q-2636) | **holds** (the leftover's row, claim 13 (a) and (e)) |
| M-0021 | Reading points: in every part B run the last F vehicle destroyed takes one more hit in its death tick, after its destroying hit, naming no attacker, with no test warhead, taking no health | fails (no hit in any of the six runs; seam, Q-2636) | **holds** (the sweep's hit, claim 13 (c)) |

M-0021 fails 10 of 50 on main and 8 of 50 here; the other eight failing rows are word for word
main's. No other record row changes: M-0001 12 of 96, M-0002 2 of 24, M-0003 1 of 35, M-0019
0 of 6 and M-0020 1 of 25 (YURI against SHK, arm S), each as on main, and every other line of the
arena's output is main's apart from timings and paths.

## Goldens

All nine regenerated with `GLASNOST_UPDATE_GOLDEN=1 cargo test --release -p glasnost-sim-gate --
--include-ignored`: `combat`, `duel`, `duel-diagonal`, `duel-naval`, `duel-m6c`, `duel-m7a`,
`duel-m7b`, `scripted` and `skirmish`. Every hash line changes, because the state now holds
`World::leftovers` (serialized even when empty). Every file keeps its line count, and the six seed sweeps match main's output line for line, so the
change is the new state, not a changed course of play. `cargo xtask gate` matches them on
wasm32.

## Lanes

Before: main d9ad36c, built from a `git archive` export of that commit. After: this branch on
d9ad36c.

| Lane | Main | Intake 105 |
|---|---|---|
| M6c sweep, seeds 1 to 16, ref | smallest minimum 2 (seed 8, radiation exits 2) | same, output identical |
| M6c sweep, play-fixes | 3 | same, output identical |
| M7a sweep, ref | 1 | same, output identical |
| M7a sweep, play-fixes | 1 | same, output identical |
| M7b sweep, ref | smallest drain minimum 3 | same, output identical |
| M7b sweep, play-fixes | 3 | same, output identical |
| Arena `--record all --threads 2` | M-0001 12/96, M-0002 2/24, M-0003 1/35, M-0019 0/6, M-0020 1/25, M-0021 10/50 | M-0021 8/50, the rest unchanged |
| `glasnost-ai --include-ignored` | 255 passed, 0 failed | 255 passed, 0 failed |
| AI quick lane, seeds 7 to 10, maps 0, 2, 8 | 36 games, 35 won, seed 7's third game not over at 120000 ticks | every game's winner and end tick as on main |

No winner changes in the quick lane, and the arena's printed results (each failing pairing's win
counts, M-0020's differing games) are main's. The per-game arena tables were not compared file by
file. The sweep's hit comes in the tick of a defeat, so in a two-house game it lands after the game
is decided.
