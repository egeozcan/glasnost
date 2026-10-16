# M6c step 1e claim ledger (spec revisions that landed on main after step 1d)

Part 5 of step 1. Each spec is compared with the revision the code had taken in; the old state is the spec file at the commit named in the table (`git diff <old> HEAD -- spec/<path>`: the commit that introduced the previous revision). Every changed claim is classified as *satisfied* (already true, with the test that shows it), *changed* (code changed, with the test), *pointer* (the claim only points at another spec's claim) or *not built* (with the reason); *(c)* is editorial (front matter, scenario text, rewording). Per-spec ledgers: [S-0502.md](S-0502.md), [S-0514.md](S-0514.md), [S-0517.md](S-0517.md), [S-0520.md](S-0520.md), [S-0614.md](S-0614.md).

| Spec | Revision | Old state | Answers |
|---|---|---|---|
| S-0514 chrono weapons | 6 (from 5) | 8dcc6b8 | |
| S-0517 deployable units | 8 (from 7) | b4e90bf | |
| S-0614 parasites | 9 (from 8) | 6e1861b | Q-1032 |
| S-0520 crates | 5 (from 3; 4 landed between) | 292cb87 | Q-1033 |
| S-0207 ground vehicle movement | 31 (from 30) | f92f3e2 | |
| S-0502 movement of jumpjets | 7 (from 6) | 800a588 | Q-1030, Q-1031 |
| S-0006 rules layering | 20 (from 18; 19 landed between) | 5cffdc6 | Q-1034 (reverses revision 18's answer) |
| S-0213 human orders | 36 (from 35) | 228fd1c | |
| S-0215 behaviours | 53 (from 52) | 3ff0fe6 | |

Requests answered and their interim rules retired: Q-1030 and Q-1031 (S-0502 revision 7), Q-1032 (S-0614 revision 9), Q-1033 (S-0520 revision 5) and Q-1034 (S-0006 revision 20 with S-0520 revision 5). One new request: Q-1090 (the moment of the teleport-capable dock fallback of S-0207 claim 27 and whether placement's idle setup assigns a destination), with an interim rule.

## S-0514 revision 6

| Claim | Class | Notes |
|---|---|---|
| 20, table row 4 (parasite attachment: S-0614 claims 1 to 4 and 24) | pointer | the row's selection is as built (`special_of`); the fate of the carried parasite is S-0614 claim 24, changed below. |
| 20, the paragraph on a carried parasite when takeover, bomb planting or charging suppresses attachment | changed | same code and tests as S-0614 claim 24 (`parasite::carrier_ended`, `quirks.parasite_suppressed_latch_strands_unit`). |
| E-scenario text for the carried-parasite fate | (c) | measurement text. |

## S-0517 revision 8

| Claim | Class | Notes |
|---|---|---|
| 5 (the deploy order's one destination-clear request, before the target is cleared and *unload* queued; destination handling per S-0502 claims 24, 27 and 28; balloon-hover rejection retains even a nonempty destination, can make *attack* current before *unload* is queued; no deploy-enabled refresh, no retry after target clearing; it can end with no target and a nonempty destination) | changed | `orders` (`Order::Deploy`: `jump::clear_rejected` before the target is cleared; on a rejection neither the destination nor the reroute is touched). Tests `claim_28_deploy_keeps_the_destination_without_a_retry` and `claim_28_deploy_without_a_qualifying_row_clears` (`claims_s0502_r7.rs`); the non-balloon rows stay as `claims_s0517_r7.rs`. |
| 5, backing text (J24, J25) | (c) | |

## S-0614 revision 9 (answers Q-1032)

| Claim | Class | Notes |
|---|---|---|
| Terms (latching onto / failing to latch; latching weapon and building-targeted detonations) | (c) | wording; claim 24 owns suppression. |
| 1 (a higher selected effect: claim 24 owns the carried parasite's fate, not claim 4) | changed | `parasite::carrier_ended` no longer sends the suppressed parasite through the failed-latch return. |
| 24 (new): takeover, bomb planting or charging selected at a detonation strands the carried parasite: off the map, unattached, unable to act, no return and no recovery timer, no death, kill experience or replacement; whatever the target's latchability (legal host, non-latchable, building, cell, lost target) and whether or not the higher effect succeeds; a shot refused before launch never creates one. `quirks.parasite_suppressed_latch_strands_unit` (default original) off: the surviving parasite attempts claim 4's return, never attaching and never any other suppressed effect; independent of `quirks.parasite_carrier_loss_strands_unit` | changed | `parasite::carrier_ended` (a detonation with `Special::Takeover`, `Bomb` or `Charge`: return when the switch is on; `fail_latch` when off). The new quirk is in `rules::quirks`, the schema and the quirk list. Tests `claim_24_a_higher_hit_strands_the_carried_parasite` (all three rows against a latchable host, a cell and a lost target: off the map, extant, health kept, no hold timer, still stranded after 300 ticks), `claim_24_switch_off_returns_the_parasite_without_attaching`, `claim_24_control_selected_attachment_to_a_lost_target_returns`. |
| State table row (stranded regardless of latchability) | changed | same. |
| Import mapping (`MindControl`, `IvanBomb`, `ElectricAssault`; the fixed engine behaviour row for claim 24) | satisfied | the importer already reads the three warhead flags (S-0514 step 1c, `WarheadType::mind_control`, `ivan_bomb`, `electric_assault`). |
| Edge cases (the new quirk) | changed | as above. |
| P14 (extended oracle text), Open questions | (c) | |

## S-0520 revision 5 (answers Q-1033; with S-0006 revision 20, Q-1034)

Revision 4 was never taken in on its own.

| Claim | Class | Notes |
|---|---|---|
| 28 (the layered table: with `quirks.override_resets_keys` on, a layer holding the section resets every omitted entry's weight to zero and its animation to no animation, keeping the water flag and argument; the reserved bare no-type name cannot be registered through the animation list or an ordinary creating reference and a section alone creates no type, so the reset always gives no animation; with the switch off an omitted entry keeps all four held values, including the built-in startup values when no layer mentioned it, section presence never substituting zero; with `quirks.persist_global_overrides` off the startup table is restored before each game's layers) | changed | `convert::sim_econ::crate_import`: an entry starts at the built-in weight (50, 20, 1, 3, 5, 5, 20, 1, 1, 10, 10, 10, 1, 3, 1, 1, 1, 1, 1), no animation, water off, argument 0.0; the reset event (recorded by `convert::layers` only with the switch on) gives weight 0 and no animation without any lookup. Tests `the_power_up_table_with_the_reset_switch_off` (the unmentioned entry keeps its startup weight and equals the fixture with no section: scenario AB), `the_power_up_table_is_layered` (A, B, C and the empty-entry section), `the_reset_never_gives_an_animation_whatever_is_named_none` (CR19's reserved-name controls: a list entry, a section alone, mixed letter case, `<none>`, a name given in an entry; the ordinary-name positive control). The later-game carry-over (claim 10) is a property of a session, which a single import does not model: the persistence-off restore of the startup table is each import's start. |
| Import mapping (`[Powerups]` row: the switch-on and switch-off wording; the new reserved-name controls row) | changed | as above; the reserved-name controls row is covered by the same test. |
| CR19 (more controls), Named test data | (c) | measurement text. |

Retired interim rules: Q-1033 (the importer's lookup of NONE finding no type: now a fixed no-animation) and Q-1034 (weight 0 for an unmentioned entry with the switch off: now the startup table).

## S-0006 revision 20 (answers Q-1034; reverses revision 18's answer)

| Claim | Class | Notes |
|---|---|---|
| 5, the power-up rows (animation: no animation for supported skirmish imports; the reserved no-type name; omission reset only with the switch on) | changed | as S-0520 claim 28. |
| 5, game-wide rows (a power-up entry keeps its built-in startup weight with the switch off whatever the section's presence; the fixed reset-value fallback applies only to keys with no specified startup value, i.e. difficulty and land types) | changed | the power-up half is S-0520 claim 28's code. The difficulty and land-type half is as built (`convert::layers`: with the switch off no reset events are recorded, and the importer's defaults for those keys are the S-0211 and S-0004 ones). |
| Import mapping row for the power-up keys | changed | as above. |
| AB (new scenario), CR19 extension, open question 11 | (c) | engine acceptance check: covered by `the_power_up_table_with_the_reset_switch_off`. |

## S-0207 revision 31

| Claim | Class | Notes |
|---|---|---|
| Terms, scope list, Import mapping rows (`Teleporter`, `DockUnload`) | satisfied | both keys are imported already (S-0214 and S-0402 mappings). |
| 27 (new): the destination-refresh state makes an otherwise eligible unchanged assignment effective, it starts absent, successful placement enables it only during the initial idle-behaviour setup; the later activation is the qualified teleport-capable dock fallback, with the separate one-application point suppression; consumption rules | not built (activation) | The placement interval has no observable effect (a freshly placed vehicle has no movement commitment, and our placement's idle setup makes no destination assignment), and the engine has no step that evaluates the dock predicates; the spec itself calls the qualified fallback `unmeasured` and gives a conditional diagnostic only (D32). Neither pending state ever occurs; an unchanged vehicle assignment is always ignored (`jump::take_destination`, `drive::take_destination`). Request Q-1090 (where the fallback is tested, what "temporary locomotion can finish" means, whether placement assigns a destination, the drive locomotor's point suppression) with that interim rule. The ordinary-order nonactivation half of the claim (stop, move, deploy and behaviour changes do not enable refresh) is satisfied: `claim_24_effective_clear_then_suppressed_assignment` and `claim_28_stop_without_a_rejection_reroutes_once` show the suppressed repeat. |
| D32 (new diagnostic) | (c) | measurement text; no fixture is established. |

## S-0502 revision 7 (answers Q-1030 and Q-1031)

| Claim | Class | Notes |
|---|---|---|
| Terms (destination-refresh state, teleport-capable flag) | pointer | claim 27 above. |
| 24, redundant assignments (equality is the same cell or the same object, not objects sharing a cell; a balloon-hover rejection retains the destination, it does not substitute one; a suppressed assignment does not reroute or erase the point) | satisfied | `Target` equality is cell-with-cell and object-with-object (`claim_24_effective_clear_then_suppressed_assignment`, `claim_28_stop_without_a_rejection_reroutes_once`); the substitution never existed. |
| 24, infantry reassignment (Q-1030): an infantry unit of a jumpjet type has no unchanged-destination suppression; the same nonempty destination elsewhere (cell or object) repeats the adjustment and its ranged draw, an empty-to-empty assignment is an effective clear (reroute with a commitment, otherwise nothing); a stop of a balloon-hover type supplies two empty assignments; a balloon-hover infantry unit has no vehicle equality suppression; the rule is per assignment | changed | `jump::take_destination` skips the equality test for infantry; `orders` (`Order::Stop`) makes the second request for a balloon-hover type (`jump::balloon_hover_type`). Tests `claim_24_infantry_repeats_a_nonempty_assignment` (cell and object, vehicle control), `claim_24_infantry_empty_to_empty_reroutes_every_time`, `claim_24_stop_of_balloon_hover_infantry_reroutes_twice` (draws of one stop: ordinary one reroute, balloon-hover two). The pre-draw refusals and the own-cell filter stay as before (`claims_s0502_r6.rs`). |
| 24, deploy while hovering (a qualifying balloon-hover rejection retains the nonempty destination) | changed | as S-0517 claim 5. |
| 24, backing text (J23) | (c) | |
| 27 (new, when a vehicle needs destination refresh) | not built (activation) | as S-0207 claim 27: Q-1090. |
| 28 (new, balloon-hover destination retention): a balloon-hover vehicle rejects an empty assignment when it has a nonempty destination and a target and the destination is the target object, or a cell whose first ground-layer listed object is the target, or the target direction is within 16384 facing units of the turret facing (signed wrapped difference, inclusive, integer arithmetic, the turret facing even without a turret); the rejection keeps destination, point, commitment and reservation, takes no draw, makes *attack* current (queued behaviour emptied) unless the current, or with none the queued, behaviour is attack; nonempty assignments bypass it; an empty destination or an absent target cannot be retained; reachable by move then attack; stop requests the clear while the target is present and again after it is cleared, deploy once | changed | `jump::clear_rejected` (the predicate and the behaviour result via `Behaviour::force`), called by `orders` where the request is made: the behaviour order after its target is installed (`apply_behaviour`), `Order::Stop` (first request before the target is cleared, second after, for the type) and `Order::Deploy` (once), and by `combat::acquire::override_attack` for the behaviour-processing request. Tests `claim_28_the_angle_bound_is_inclusive` (0, 16383, 16384, -16384 retain; 16385, -16385, 32768 do not; no draw), `claim_28_identity_rows_and_the_controls` (both identity rows with the turret facing away, the controls: neither row, empty destination, no target, a nonempty request, an ordinary jumpjet vehicle), `claim_28_the_behaviour_result`, `claim_28_stop_ends_empty_after_a_rejection`, `claim_28_stop_without_a_rejection_reroutes_once`, `claim_28_deploy_keeps_the_destination_without_a_retry`, `claim_28_deploy_without_a_qualifying_row_clears`, `claim_28_behaviour_processing_clear_can_be_rejected`. Limits: other behaviour-processing sites that write an empty destination directly (the fall-backs and deploy-related clears of `process`) do not call the rejection, because they do not pass the locomotor's assignment hook today (the locomotor sees the clear at its next update as before) and the spec names no site beyond the override and the orders; the spec marks a stock deployable balloon-hover fixture `unmeasured`. |
| Import mapping rows (`BalloonHover` with retention; `Teleporter`; `DockUnload`; the "no additional key" row) | satisfied | all keys imported. |
| J23 to J25 (new scenarios), the J22 text | (c) | measurement text. |

## S-0213 revision 36

| Claim | Class | Notes |
|---|---|---|
| 8 (stop: the exception list; infantry has no equality suppression; stop's two requests for a balloon-hover type; the interim attack surviving; per assignment) | changed | as S-0502 claims 24 and 28 (`orders`, `Order::Stop`). |
| 10 (deploy: one request before target clearing, the destination becomes empty except for the balloon-hover retention; infantry empty-to-empty is effective; no refresh) | changed | as S-0517 claim 5 (`orders`, `Order::Deploy`). |
| 21 (the destination-request table: stop and deploy rows; the filters for jumpjets: infantry repeats, refresh, the balloon-hover rejection; backing) | changed | as S-0502 claims 24, 27 and 28; the refresh rows are Q-1090. |
| Scenario text (J23 to J25 pointers) | (c) | |

## S-0215 revision 53

| Claim | Class | Notes |
|---|---|---|
| 13 (a destination clear: infantry of a jumpjet type has no unchanged-destination suppression, also when behaviour processing assigns; the balloon-hover rejection by behaviour processing, forcing *attack* with claim 9's start effects; "a requested clear does not guarantee an empty destination") | changed | infantry: `jump::take_destination`; the rejection: `jump::clear_rejected` in `combat::acquire::override_attack` (the override installs its target and requests the clear); see the limits under S-0502 claim 28. Test `claim_28_behaviour_processing_clear_can_be_rejected`. |
| 13, backing (J22 to J25) | (c) | |

## Result

Code: the importer's power-up table, `parasite::carrier_ended`, `jump::{take_destination, clear_rejected, balloon_hover_vehicle, balloon_hover_type}`, `orders` (behaviour order, stop, deploy) and `combat::acquire::override_attack`. New quirk `parasite_suppressed_latch_strands_unit`, default on. No simulation state changed (no field added: save format stays 44, AI game format 12). The sim-gate goldens hold unchanged (`-- --include-ignored` passes).
