# S-0203 r53 and S-0107 r46: claim ledger, M8 step A3 intake 107

- **S-0203** r50 to r53 (commit ce50a38, from the r50 that intake 105 built, 5534a23). r53 answers
  Q-2691 (claim 13 (f): a sweep reaches every leftover at its place, whatever removed it, a sweep
  or a death weapon of the same sweep included), Q-2692 (claim 13 (g): a leftover with health left
  takes a killing hit; sinking and crashing at that hit) and Q-2693 (claim 13 (h): every leftover
  holds its place in S-0414 claim 15's creation order, in both positions of the switch). It
  rewrites claim 13 (c) (the hit goes to every leftover vehicle of the house, with or without
  health), (b) (a passenger killed with its transport keeps its health), (e) (a vessel kept to the
  end of the game), the Term *leftover*, the Edge case of `quirks.defeat_sweep_leftovers`, scenario
  O18 parts f to j, and the Changes for other specs (Glossary, S-0414 claims 5, 11, 15 and 21,
  S-0312 claims 1 and 32, S-0504 claim 4, S-0302 claim 8).
- **S-0107** r40 to r46 (commit 651f294, from the r40 that intake 99 built, 55f236c). r46 answers
  Q-2616 with claim 11's *Changes of target*: every change of an infantry unit's target (another
  target, the loss of its target, or one gained when it had none) is a change of target for the
  attack state's ending and its request, at the moment it is made, whatever rule makes it and in
  whichever actor's turn; the attack-move discard and put-back of S-0309 claim 23 are the only
  exception. No quirk is added or changed.

Requests filed (block Q-2716 to Q-2725): **Q-2716** (whether a leftover keeps its tag, so that
claim 29's raises come at the sweep's killing hit; interim: removal detaches the tag, no raise) and
**Q-2717** (whether a mind-control release reads the guard request's readiness before or after its
target clear, now that the clear ends the attack state; interim: before). Q-2718 to Q-2725 unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

### S-0203 r53 (claim 13)

- **The store** (`glasnost_sim::leftover`). `Leftover` is now the object and a `kept` mark; the
  death mark of intake 105 is gone, since the hit no longer depends on how the vehicle left.
  `leftover::clear` (the start of `tick::step`) drops every leftover that is not kept, so a kept
  vessel stays in the store, out of play, off the map, with its place, to the end of the game
  (claim 13 (e), (g)). It is saved and hashed with the rest.
- **The passenger's health** (claim 13 (b)). `transport::died_with_transport` (used by
  `kill_passengers` and claim 12's failed escape) reads the passenger's health before its death and
  sets the leftover's health back to it (`leftover::keep_health`). Infantry passengers are offered
  no hit (claim 13 (c)), so only a vehicle passenger's killing hit follows from it.
- **The sweep's reach** (claim 13 (c), (f)). `econ::victory::destroy_all` already walked the
  house's objects and every leftover together in creation order; the leftovers an earlier sweep of
  the same house update or a death weapon earlier in the same sweep made are reached at their places
  (the intake 105 interim of Q-2691, now the spec). Under `quirks.defeat_sweep_leftovers`
  `leftover::sweep_hit` now reaches every leftover vehicle of the house, whatever removed it:
  a death, a sale, a deploy, an absorption, the grinder, a crush, an erasure or a passenger's death
  with its transport.
- **The hit** (`leftover::sweep_hit`). Its amount is the leftover's health. `Effect::LeftoverHit`
  now carries the health before and after (`(i32, i32)`). With no health: no health taken, nothing
  registered, the death effects once more (as intake 105). With health left: health to 0, a
  killing hit whose destruction `damage::credit` registers once more with no attacker and no house
  (a loss, no kill; S-0414 claim 21), with the leftover put back among the objects for the call
  alone (`leftover::with_restored`), then the death effects as at any killing hit.
- **Sinking** (claim 13 (g)). `death::repeat_plan` reads S-0312 claim 10's conditions
  (`naval::sinks_at_kill`) for a leftover with health left only; a leftover with no health starts
  no sinking. One that meets them takes claim 10's branch: no claim 9 animation or pick, health back
  to 1, marked sinking (`naval::start_sinking`, the leftover put back for the call), and, of a type
  that cannot crash, kept (`Repeat::kept`), with no removal and so no crate. One whose type can
  crash fails its crash start, is removed again (its crate placed, S-0312 claim 14) and ends with
  its tick.
- **Crashing** (claim 13 (g)). A leftover of a type that can crash above the terrain at the hit
  (`Repeat::crash_start`) starts a crash after its other effects: it holds no passengers in our
  engine (`death::leave_play` dealt with them) and takes S-0312 claim 16's three tumbling draws
  (unit interval, ranged 0 to 1, unit interval); taking no turn, it does not fall, so no removal and
  no crate, and it ends with its tick. One on the ground shows claim 9's animations and starts no
  crash.
- **Places** (claim 13 (h)). Every leftover of the four kinds holds its place for claim 15's
  cancellation skip in both positions of the switch, as built at intake 105; the cite now names
  claim 13 (h) in place of the Q-2693 interim.
- **The M-0021 recorder** (`oracle_m0021`) reads the hit's health pair directly.
- **Formats.** Save format 108 to 109, AI game format 72 to 73 (`Leftover`'s fields), one past main
  07d549c.

### S-0107 r46 (claim 11, *Changes of target*)

- `infantry::watch_test` compares the unit's target with its baseline (`InfantryState::watch`):
  any difference, the gaining of a target from none included, is a change (`target_changed`: the
  attack state ends and the unit asks, not forced, for *deployed*, *prone* or *stand*). The intake
  99 test ignored a gain from none; that exception is gone.
- The rules that write an infantry unit's target call it right after the write, so the change takes
  effect when it is made, in whichever actor's turn: orders (`orders::apply_pending`, both places
  that set the baseline before an order now test), a retaliation or other override
  (`combat::acquire::override_attack_on`, in the deliverer's turn), a berserk hit's drop
  (`psychic::drop_target`), a guard reset and a domination takeover (`econ::power::guard_reset`), a
  mind-control release (`mind::release_orders`), the reverts that put an earlier target back when an
  override ends (`cloak::machine`, `teleport`, `bridges`), an open-topped transport's clear and
  hand-down (`transport::clear_passenger_targets`, `hand_down_target`), and a healer's drop in its
  own firing step (`combat::behave::infantry_step`). The target leaving play already called
  `target_changed` (`held::holders_drop`).
- The test at the start of the unit's turn (`tick::object_turn`) now tests instead of only setting
  the baseline, so a rule that writes a target without testing ends the state before any start of
  the unit's update.
- The lazy ending at the firing turn (the Q-2616 interim in `combat::behave::infantry_step`) is
  removed.
- A release reads the readiness of its guard request before its target clear (interim Q-2717), then
  clears, tests and makes the guard request.

## Claims

| Claim | Status | Tests and notes |
|---|---|---|
| S-0203 13 (b): a passenger killed with its transport keeps its health | tested | `claims_s0203_r53::a_passenger_killed_with_its_transport` |
| 13 (c): the hit to every leftover vehicle of the house, whatever removed it | tested | `claims_s0203_r50::o18_e_the_sweep_hit_raises_no_tag_and_only_a_killing_hit_counts` (a removal with health left) |
| 13 (e): a kept vessel survives a save and load | tested | `claims_s0203_r53::a_kept_vessel_survives_a_save_and_load` |
| 13 (f): a death weapon of an earlier place, reached at the later place (O18 f) | tested | `claims_s0203_r53::o18_f_a_death_weapon_destroys_a_later_place` (both creation orders, both positions) |
| 13 (f): two sweeps in a surrender's house update (O18 g) | tested | `claims_s0203_r53::o18_g_two_sweeps_in_a_surrender` (both positions) |
| 13 (f): three sets of effects for a vehicle removed earlier in that tick | partly | follows from the same walk; no test sets up both at once (the spec leaves it `unmeasured`) |
| 13 (g): a killing hit for health left, a loss and no kill | tested | `o18_e_the_sweep_hit_raises_no_tag_and_only_a_killing_hit_counts`, `o18_h_a_vessel_whose_descent_ends` |
| 13 (g): claim 29's raises at that hit | seam (interim Q-2716) | the tag is detached at removal, so none comes |
| 13 (g), Sinking: claim 10's branch, the vessel kept (O18 h) | tested | `claims_s0203_r53::o18_h_a_vessel_whose_descent_ends` |
| 13 (g), Sinking: a crashable vessel removed again, ends with its tick | tested | `claims_s0203_r53::o18_h_a_crashable_vessel_ends_with_its_tick` |
| 13 (g), Sinking: no health, no sinking; the bomb mark's wake and splash | partly (seam) | no sinking without health is built; S-0312 claim 11's wake and splash is not built in our engine (seam, as at the first death) |
| 13 (g), Crashing: a crash ended on the ground (O18 i) | tested | `claims_s0203_r53::o18_i_a_vehicle_whose_crash_ended` |
| 13 (g), Crashing: above the terrain, three draws, no fall, no crate | tested | `claims_s0203_r53::a_leftover_above_the_terrain_starts_a_crash` |
| 13 (h): places in the cancellation skip, both positions | tested | `claims_s0414::claim_15_cancellation_skip_falls_on_a_leftover` |
| Edge cases: the switch off (O18 j) | tested | `claims_s0203_r53::o18_j_switch_off_keeps_no_vessel`, the off halves of f and g; C7 verdict in [c7-play-fixes.md](c7-play-fixes.md) (rejected, unchanged) |
| S-0107 r46 11, *Changes of target*: a retaliation's override | tested | `claims_s0301::s0107_claim_11_attack_ends_on_a_new_target` |
| 11: a berserk hit's drop | tested | the same test |
| 11: a target gained from none | tested | the same test |
| 11: no change, no ending (the same target) | tested | the same test |
| 11: a write that tests nothing, caught at the start of the turn; no lazy ending | tested | `claims_s0301::s0107_claim_11_a_change_written_elsewhere_ends_before_the_turn` |
| 11: orders, the target leaving play, own processing and search | tested | intake 99's tests in `claims_s0215_r118.rs`, unchanged |
| 11: guard reset, takeover, release, reverts, transport clears | partly | hooked; no test of their own (the hook is the same call) |
| 11: the attack-move exception | tested | intake 99's tests, unchanged |

## Records

LANES_RECORDS

## Goldens

GOLDENS

## Lanes

LANES

## Seams and open points

- Q-2716 (tags at the killing hit), Q-2717 (release readiness): interims above.
- S-0312 claim 11's wake and splash for a bomb-marked vehicle low over water is not built in our
  engine, at the first death or at the sweep's repeat.
- Infantry, aircraft and building leftovers are offered no hit (claim 13 (c); S-0203 Open
  questions).
