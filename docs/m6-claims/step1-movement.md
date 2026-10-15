# M6 step 1: movement, orders, economy and production spec revisions since 8d4181a

Classification of every changed hunk in `git diff 8d4181a HEAD` for the specs below.
Buckets: **(a)** changes or clarifies behaviour the engine already implements;
**(a, already done)** the committed code already does what the new text says;
**(b)** pointer into S-0501 (aircraft), S-0504 (naval), S-0506 (missiles / spawn
controller) or S-0614 (parasites), effective only once that feature exists;
**(c)** editorial, no behavioural change.

M6a: the (b) pointers into S-0501 are now implemented in `glasnost_sim::air` (lane A2); see
[S-0501.md](S-0501.md) for their status. The evidence column below is unchanged.

Evidence is taken from committed code at HEAD (d158117). The working tree also holds
uncommitted work from another session (`sim/src/air.rs`, `naval.rs`, `parasite.rs`,
`rocket.rs`, `spawn.rs`, plus new quirk fields in `rules/src/quirks.rs`). Those files
are not counted as evidence here, but the (b) items will probably land in them.

Every file's front matter (revision, approved-on) changed. That is (c) and is not
repeated below.

---

## S-0206 Turn rates (rev 9 to 10)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 12 (ship sentence and backing) | (b) S-0504 claim 1 | The turning rules apply unchanged to the ship locomotor on ordinary tracks, backed by S-0504 N1. | `turning.rs` needs nothing. The ship locomotor's turn will reuse it (`naval.rs`, uncommitted). |
| F5 (several turret rows) | (c) | The same state relation, restated as a three-row table (τᵢ = τᵢ₋₁: no change; Θᵢ > 0: start from *s*ᵢ = (τᵢ − βᵢΘᵢ) mod 65536; Θᵢ = 0 or ω ≤ 0: at once), with the same spin-row rule and the same R4 run f example. | `turret.rs` stays as it is. |

New quirk switches: none.

## S-0207 Ground vehicle movement (rev 17 to 18)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *heading point* | (b) S-0504 claim 1 | The ship locomotor has the driving heading point. Hovering and other locomotors stay open. | `path::heading_point` (path.rs:985), to be extended for ships. |
| Terms, *Sinking* | (b) S-0504 claims 4, 5; S-0614 claim 11 | Sinking is also started by a grappler's cull (no hit, no health loss). While sinking, the locomotor is not processed. | Nothing in drive.rs. Death and sinking will live in `combat::death` and `naval.rs`. |
| 6, Launching hold (+ backing D26) | (b) S-0506 claim 10 (with S-0504 claim 3) | A driving vehicle that is *launching* does not begin a new track. It keeps its destination and resumes when launching ends. A track already begun finishes. | Would go in `drive::tick` / `begin_track` (drive.rs:444, 1275). No launching state exists yet: `combat/check.rs:38,251` notes "launching spawned aircraft" as unset. It needs the spawn controller. |
| 10 (preamble, case 3 marked unreachable, backing and confidence) | (a, already done) | Sinking vehicles get no locomotor processing, so case 3 (sinking deceleration) is never reached. The ship locomotor uses the same speed-fraction rules. | `drive::update_rho` (drive.rs:259-320) has no sinking row: crush, braking, then cases 4 to 6. That is consistent. Ship reuse is (b) S-0504. |
| 12 (worked example) | (a, already done) | The "sinking vehicle farther than *D*ₛ gets *F*₀.₁" example is removed: no movement update happens while sinking. | Same evidence as claim 10. |
| 16 (confidence) | (b) S-0504 claim 1 | Ship scatter centre = driving heading point. | Scatter in `scatter.rs` / `drive::request_scatter`. |
| 20 (a) | (b) S-0614 claims 7, 8 | "Under a paralysing effect" becomes "is *held*" (S-0614 hold timer). A held object drops the destination. | Done: `teleport::take_destination` drops it for a held object; test `claim_20_held_object_drops_the_destination` (M6b). |
| 20 (d) / (e) | (b) S-0614 claims 12, 13 | The parasite exit moves from (e) ("thrown out at *L*") to (d): the parasite exits *before* the host leaves its old position. | Done: `parasite::host_jumps` runs before the host leaves its old position in `teleport::jump`; the stale comment is gone (M6b). |
| 23 Driving wakes (new; + Import rows `[General] Wake`, vehicle `Underwater`; scenario D25) | **(a)** | New: a drive vehicle that is *moving now* creates one game-wide wake animation at its own position on ticks divisible by 10. Conditions: off a bridge deck, its cell's land type is water, and the animation is defined. *Moving now* = body turning, or a heading point and current speed > 0. There is no underwater-type exclusion, and the only draws are the animation's creation draws (S-0109 claim 3). | `drive::tick` (drive.rs:1275), or a per-tick hook after the drive update, calling `combat::effects::anim` (combat/effects.rs:118). Import: `[General] Wake` is currently unsupported (`convert/src/coverage.rs:249` lists it under S-0312; `refs.rs:361` knows it as an Anim reference). Map it into the sim's general rules (`convert/src/sim.rs`). `Underwater` is already imported (`rules/src/sim.rs:1292`) and must *not* gate this. `combat::effects::anim` takes the S-0109 claim 3 creation draws through `load::anim_created_at`, as claim 23 requires. **Blocked in part:** S-0207 does not define "current speed" (ρ > 0 or integer step *s* > 0). S-0504 claim 2 uses the same words, and Q-0590 was answered for the harvesting sprite only, so that answer must not be borrowed. The speed half of "moving now" waits on a spec request covering both S-0207 claim 23 and S-0504 claim 2. The turning half, the tick cadence and the deck, water and animation conditions can be implemented now. |
| Open questions (sinking events, heading point, ship speed fraction) | (c) | Updated pointers. Ship speed and heading point are settled by S-0504. | — |

New quirk switches: none.

## S-0209 Occupancy and crowding (rev 25 to 26)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Parasite, victim* | (b) S-0614 claims 1-5, 13 | Adds *host*. Either an infantry or a vehicle mover can be a parasite. WP1/VP2 apply to entry while moving, not to the parasite's exit test (S-0614 claim 13, where the host blocks its own cell). | occupancy.rs:5 ("nothing is … a parasite"). No parasite rows yet. |
| 4 row WP1, 5 row VP2 | (b) S-0614 claim 13 | Scope narrowed to "asks to enter while moving, not for exit". | Same. |
| U2 (unmeasured note) | (c) | States that the uncertainty is about movement entry only. | — |

New quirk switches: none.

## S-0210 Crushing (rev 17 to 18)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 9 (ship locomotor wording) | (b) S-0504 claim 1 | Ship speed-fraction updates now come from S-0207 claim 10 via S-0504 claim 1. The crush slowdown rule itself is unchanged (min(ρₜ, 0.2), accelerating and not passive only). | drive.rs:287-291 is already correct for the driving locomotor. The ship case will be in `naval.rs`. |
| Open questions (ship speed outside the state) | (c) | Removed, now settled by S-0504. | — |

New quirk switches: none.

## S-0212 Pathfinding (rev 29 to 30)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 25 table, new row "Parasite exit", plus note | (b) S-0614 claim 13 | Nearby search for a parasite exit: centre at the host's cell, with the host's class (*winged* counted as *track*) and the host's zone, reach region of the centre for that zone (deck when on a deck), bridge exclusion when the host is on a deck, no overlay test. Alternative: `quirks.parasite_exit_search_host_mobility` (owned by S-0614) uses the parasite's class and zone instead. | `path.rs` nearby-search table, called from `parasite.rs` when it exists. |
| 25 timing list, backing | (b) S-0614 claim 13 | Adds the exit-search tick and the P4 backing. | — |

New quirk switches referenced: `quirks.parasite_exit_search_host_mobility` (owned by S-0614 claim 13).

## S-0213 Human orders (rev 25 to 26)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 3 (precedence restated, between-house rule) | (a, already done) | Results and checks are against the effects of preceding orders. The non-behaviour group comes before the behaviour group, so the behaviour order decides any shared actor state. Between houses, creation order: a later house's successful changes prevail, and an order whose actor an earlier house removed fails its actor check. | `orders::apply_pending` (orders.rs:194-215): houses in order, non-behaviour orders, then behaviour orders, each in issue order. Actor checks run at application (`live_on_map`, `valid`, orders.rs:218-226). |
| 3, production-start draw assignment | (a, already done) | Restated as "association of starts with values follows causing-order precedence". No new rule. | Same loop. Production starts are made inside `apply_other` in issue order. |
| 4 backing | (c) | Validity of cells and non-unit references is marked `unmeasured`. | — |
| 6 (attack-move rows, distinction from the standing *attack move*) | (c) | Clarifies that losing eligibility can expose the ineligible-mobile column, and that the building and neither-reference cells have no input path. Target-only and destination-only orders give *attack* and *move*, not the standing *attack move*. The translation is unchanged. | orders.rs:468 (translation at application). |
| 7 (judged state for group destinations) | (a, already done) | Members and cells include this house's non-behaviour orders but none of its behaviour orders. | orders.rs:211 `group_destinations` runs after the non-behaviour loop and before any behaviour order is applied. |
| 7 backing | (c) | Look-class priority, nearest-member priority and last-cell fallback are marked `unmeasured`. | — |
| 19 (cheer) backing | (c) | Qualification about a dropped cheer order. | — |
| Formulas (leader / destination priority) | (c) | Wording: "destination priority by increasing *d*", the same ordering. | — |
| Ordering consequences | (a, already done) | Adds "successful" and "an order that fails its application checks does not overwrite state changed by a preceding order". | Same as claim 3. |
| Departures, the click marker | (c) | Now points to S-0214 claim 30 for eligibility and placement. It stays drawing-only with no scenario draw. | See S-0214 claim 30. |
| Scenario O21 (status font) expected / falsifier | (a, already done) | Line origins are 3 px right of the battlefield's left edge. | `app/src/game/status_font.rs:31` (`LEFT = 3`). docs/development.md says "3 pixels in from the battlefield's top left". |
| Open questions (S-0417 revision cite), Questions for the project lead (S-0215 claim 5 companion) | (c) | — | — |

New quirk switches: none.

## S-0214 Order targeting rules (rev 24 to 26)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Parasite* | (b) S-0614 claims 7, 8 | The parasite's *hold* is S-0614's hold timer, separate from *paralysed*. | — |
| Terms, *Movable* | (b) S-0614 claims 7, 8; S-0506 claim 10 | "Not held still by a parasite's paralysis timer" becomes "not held". The launching exclusion becomes "launching count not strictly between 0 and spawn count" (S-0506 counts, not aircraft away). | `targeting::movable` (targeting.rs:356-372) has neither test yet. Infantry and vehicles return `true`. |
| Terms, *Docking answer* | (b) S-0504 claim 7 | A building refuses when exactly one of it and the unit is naval, unless the unit's zone is *amphibious*. This only removes the naval-mismatch refusal. | No docking answer exists yet (targeting.rs:15, 797, 814, 1085: "no contact exchange"). |
| 4 backing, 8 backing, scenario intro paragraph | (b) | Cite S-0506 R6, S-0614 P1/P9 and S-0504 N7. | — |
| 14 (marker sentence replaced; confidence) | (a) | The marker rule moves to claim 30. One acknowledgement per click is unchanged. | See claim 30. |
| 30 Move-target marker (new; + Import row `[General] MoveFlash`; scenario T22) | **(a)** | New client feedback. One marker per battlefield **cell** click, and only when the *first* selected object is a mobile actor whose action is *move*. That includes force-move with action *move* and a move turned into attack-move to a place by claim 5, and an accepted route-mode cell click. Never for *no-move*, object clicks, building rally or guard-spot clicks, clicks refused before dispatch, or other houses' or AI orders. Position: the clicked cell's centre (256x + 128, 256y + 128), exact integers. Height: the terrain surface height (S-0106 claim 2), + 416 on any intact high-bridge deck cell. The original's unmarked-deck defect is corrected without a quirk. The marker is drawing-only (no scenario draw), uses the imported `MoveFlash` animation, is unattached, has no loop delay and repeat 1, and plays per S-0109 claims 1, 5, 7, 16 and 26. | Nothing exists yet (no marker, no `MoveFlash` import: grep finds none). Input: `app/src/game/play.rs` `Play::apply_click` (play.rs:416), which needs the first object's resolved action from `targeting::click` (sim `targeting.rs`). Key the marker on that action being *move*, not on `r.orders` being non-empty: route-mode clicks issue no immediate order, and the marker promises no reachable destination. Route mode does not exist in the client yet. Drawing: a client-side animation in `app/src/mapview/live.rs` / `build.rs`, kept out of `World` state. Import: `[General] MoveFlash` in `convert/src/sim.rs` or the render/presentation data. Deck height: `sim/src/bridges.rs` / `surface.rs`. |
| Import mapping `SpawnsNumber` | (b) S-0506 claim 10 | Renamed "spawn count". | — |

New quirk switches: none (the deck correction explicitly has no switch).

## S-0215 Behaviours (rev 25 to 26)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Sources list | (c) | Adds S-0501. | — |
| Terms, *Resting behaviour* | (b) S-0501 claim 18 | Aircraft rest choice and early exits come from S-0501. | Aircraft get no behaviour turn: process.rs:81, 412. |
| 1 backing | (c) | Human-owner set-up comparison marked `unmeasured`. | — |
| 3 (b) | (b) S-0501 claim 31 | An aircraft's end-of-update start is processed on *t* + 1. | — |
| 5 (attack move from ordinary input) | (a, already done) | Clarifies that ordinary eligible input gives *attack* / *move*, not the standing *attack move*. Only data establishes the standing behaviour. | orders.rs:468 translation. No change needed. |
| 11 (aircraft per-tick start and readiness; backing; confidence) | (b) S-0501 claim 31 | Aircraft start after processing and movement, plus during the locomotor step and at the end of a fall-back. Readiness: not *sticky*/*rescue*, no run lock except in spy-plane approach, no cleared aircraft ready mark. | `process::ready_to_change` returns `false` for aircraft (process.rs:81). |
| 12 (aircraft start; attack-move clarification; backing) | (b) S-0501 claim 31; rest (a, already done) | Aircraft use S-0501's three opportunities. The attack-move paragraph repeats claim 5's clarification. | orders.rs:468. |
| 13 (aircraft early exits; second fall-back) | (b) S-0501 claim 18 | Three early-exit rows skip the common effects and the queued start. | — |
| 34 Pointers (missile/paradrop/spy-plane, *retreat*, aircraft processing; backing; confidence) | (b) S-0501 claims 12-18, 24, 29-34 | Ownership is re-pointed. Aircraft *hunt* queues *attack* without an immediate start. | — |
| 38 backing | (c) | Centre-precedence and start-assigned record cases marked `unmeasured`. | — |
| Measurement-scenario common setup, B15 b, B32, B33-B38 | (c) | "Director" placeholders replaced, wording and run counts changed. | — |
| Open questions, cross-spec changes | (c) / (b) | Aircraft readiness is now owned by S-0501 claim 31. | — |

New quirk switches: none.

## S-0402 Harvest cycle (rev 21 to 22)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms *Capacity C*, 1, Edge cases, Import `Storage`, H1 d, Questions | **(a)** | New `quirks.zero_capacity_harvester_cycles` (default on = original empty cycle). Off: a **vehicle** harvester's imported capacity 0 becomes effective *C* = 1 everywhere (fill ratio, full test, load step, payment). Slave capacity is unchanged. The importer warns about vehicle capacity 0 in both settings. | Effective capacity at rules build: `econ/harvest.rs:200` (vehicle `capacity: u.storage.unwrap_or(0)`; line 187 is the infantry slave and stays as is). Also check `full` (harvest.rs:479-487) and the capacity-0 branch (harvest.rs:1119-1123). **Importer warning missing:** no capacity-0 warning in `convert/src/sim_econ.rs` (only `dock_unload_warning`, line 838), although the earlier revision already required one. |
| 4 (ring-search preference) | (a, already done) | Preference within the ring is by value per bale × bales present, not either factor alone. | `resource::worth` = V × (d + 1) (econ/resource.rs:272-278), used at harvest.rs:673-694. |
| 5 (timing qualifiers), F1 | (c) | "*C* ≥ 1" and "*L* > 0" qualifiers. | — |
| 5a, visibility (Q-0590) | **(a)** | The ground-driver movement test is now: an active track with integer step *s* = trunc(*s*₂ × ρ) > 0 (S-0207 claim 2), not ρ > 0. With ρ > 0 and *s* = 0 the sprite shows. With *s* > 0 it is suppressed even if no track point was bought. | `econ::harvest::display_suppressed` (harvest.rs:554-559) uses `d.rho.gt(F64::ZERO)`. Replace that with a test on the tick's step *s* from S-0207 claim 2: record *s* in the drive state when `drive::tick` earns it, or share `drive::progress` (drive.rs:323). Do not gate on `drive::speed_estimate` (drive.rs:358). That function is S-0207 claim 18's lead estimate (owned by S-0306) and may drift from claim 2. Boundary case: at a track boundary ρ can be updated a second time (S-0207 claim 12), so the ρ held after the tick is not necessarily the ρ at which that tick's *s* was earned. Test this against H26's zero-step control. Update the comment and the docs/development.md interim note. |
| 5a backing, confidence; H26 additions (owner-colour, shadowed-cell, erasing, no-draw, zero-step controls) | (c) | Scenarios and confidence only. | — |
| Import mapping, harvesting asset row | (c), with a citation fix | The file name was removed from S-0402. The fixed asset is still named in S-0102's Import mapping (spec/rendering/S-0102-draw-order.md:1418). | `app/src/mapview/harvesting.rs:16` and `build.rs:3007` cite "S-0402 Import mapping (… OREGATH.SHP)". Re-point them to S-0102's Import mapping. |
| 8, 10, Edge cases, Import `DockUnload`, H10, Questions | **(a)** | New `quirks.non_unloading_refinery_accepts` (default on = original stall). Off: a dock-list building without the dock-unload flag is excluded from both the normal and waiting refinery choice, and refuses refinery contact, including through an explicit enter order. Other unit-entry exchanges are unchanged, and the importer warns in both settings. | `econ::harvest::accepts` (harvest.rs:761-782), used by `choose_refinery` (harvest.rs:897) and `process_enter` (harvest.rs:1263). Add `!quirk && !dock_unload → false` (dock flag at harvest.rs:86/230). An explicit enter order queues *enter* (orders.rs:489). The harvester's contact request then goes through `process_enter`, which calls `accepts` (harvest.rs:1272, 1286), so the refusal covers that path too. Verify with a test. The warning already exists (`convert/src/sim_econ.rs:838-864`). |
| 8 backing (placement versus production-start join) | (c) | — | — |
| 10 / 13 / 26 / 27 backing | (c) | `unmeasured` qualifications. | — |
| 26, 27, 28, Open-question / Q-0371 sentence-case fixes | (c) | Capitalisation only. | — |
| H4 cross-type run, H5 C/D, H6 f, H7 f, H12 | (c) | Scenario rewrites. | — |
| Requests hand-off (Q-0580, Q-0590) | (c) | Q-0590 is answered by claim 5a and H26. | — |

New quirk switches: `quirks.zero_capacity_harvester_cycles`, `quirks.non_unloading_refinery_accepts` (both default original). Both fields already appear, uncommitted, in the working tree's `rules/src/quirks.rs`, with no logic yet.

## S-0404 Production queue and payment (rev 14 to 15)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 3 table, "Any other line state" | (a, already done) | The dock-bound aircraft queue test is S-0407 claim 16's house-wide pad-capacity test (S-0501 claim 30). It is distinct from, and does not replace, the free-dock factory eligibility. | Separate tests: factory and free dock at production.rs:999. Queue test `tech::queue_refused` at production.rs:1040 / tech.rs:1170. |
| 9 (legal build order) | (a, already done) | Same distinction: free dock at an eligible factory versus house-wide queue capacity. | Same. |
| 15 (pad positions) | (b) S-0501 claim 30 | The pad-capacity test is not a release test. | — |

New quirk switches: none.

## S-0407 Tech tree (rev 13 to 14)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 16 (pad total *T*; backing; confidence) + Import `NumberOfDocks` + T12 run 7 | **(a)** | *T* = the house's pad total. Each placed building **with the helipad flag** adds its docking-place count, whose absent default is now **1** (was 0); an explicit 0 adds 0. Placement adds, owner change transfers, and removal subtracts without taking the total below 0. | `econ/tech.rs:351-355` (`number_of_docks.unwrap_or(0)` must become 1; the helipad gate is already there). The total is a wrapping sum over on-map owned buildings (tech.rs:1199-1206). That matches add/transfer/subtract for non-negative counts but not for negative explicit counts. The spec defines a running total: placement adds, owner change transfers, removal subtracts with a floor at 0. Implement it as a per-house running total updated on placement, owner change and removal. That total is simulation state, so it must be serialized and included in the state hash. `econ/release.rs:58, 1008` already use default 1 for S-0417/S-0404. **Done (M6a lane A2):** `HouseTech::pad_total`, `tech::pad_placed` / `pad_owner_changed` / `pad_removed`; ledger `docs/m5-claims/S-0407.md` row 16. |
| 16 + Quirk switches list | **(a)** | New `quirks.pad_limit_counts_list_only` (owned by S-0501 claim 30, default on = original). Off: *D* also counts the requested type's present objects and line copies when it is not on the pad list, never twice. | `tech::queue_refused` *D* loop (tech.rs:1186-1197). **Done (M6a lane A2):** `claim_16_pad_limit_counts_list_only`. |
| Quirk list, `dock_bound_aircraft_ignores_limit` "owned here" | (c) | — | tech.rs:1210-1215 unchanged. |

New quirk switches: `quirks.pad_limit_counts_list_only` (owned by S-0501 claim 30; the field already appears, uncommitted, in the working tree).

## S-0415 MCV deploy / undeploy (rev 11 to 12)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Scenario part f (1) | (b) S-0614 claim 12 | Records the parasite's release and the end of the hold. Scenario text only. | — |
| Scenario part k | (b) S-0614 claim 12 / (c) | Attachment alone prevents deploy readiness. Release from a living host is S-0614's. | `econ::deploy` already has no parasite state. |

New quirk switches: none.

## S-0417 Factory unloading (rev 9 to 10)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 11 (destination and pad contact; take-off hypothesis; backing; confidence) | (b) S-0501 claim 8 | An aircraft on a pad ends its pad contact when it takes a destination other than the pad, which can be before lift-off. The tether is lost by take-off at the latest. | Release tethering exists (`econ/release.rs:916`). Aircraft never move or take destinations in the sim yet. |
| U12, Open question 1 | (c) | Scenario and question rewording. | — |

New quirk switches: none.

## Q-0590 (request)

| Item | Bucket | Change | Code location / evidence |
|---|---|---|---|
| status open to answered, answered-by S-0402 | (a) | Answered by S-0402 claim 5a: integer step *s* > 0. | See the S-0402 5a row. docs/development.md, "Harvesting sprite" bullet: the interim rule "a ground driver on a track with speed fraction ρ above 0 counts as moving" is now answered and must be replaced. |

## Glossary (rev 25 to 27)

| Term | Bucket | Note |
|---|---|---|
| Planned-spec note (S-0516), batch note (S-0501/S-0504/S-0506/S-0614 drafts) | (c) | — |
| *aircraft* (changed) | (b) S-0501 claims 1-10; S-0506 claims 14-21 | — |
| *cruise height* (new) | (b) S-0501 claim 2 | — |
| *in-the-air answer* (new) | (b) S-0501 claim 18 | — |
| *locomotor* (changed) | (b) S-0504 claim 1, S-0501, S-0506 | — |
| *pad, pad reservation*, *pad-bound* (new) | (b) S-0501 claims 18, 21-24 | — |
| *pad total* (new) | (a) S-0407 claim 16 (owner S-0501 claim 30) | See the S-0407 row. |
| *service step* (new) | (b) S-0501 claims 26, 27 | — |
| *sinking descent* (new) | (b) S-0504 claim 4 | Affects S-0207 Terms *Sinking*. |
| *speed fraction* (new) | (c) for ground vehicles; (b) S-0501 claim 5, S-0504 claim 1 | — |
| *submerged* (new) | (b) S-0504 claim 6 | — |
| *visiting aircraft* (new) | (b) S-0501 claims 1, 10 | — |
| *wake* (new) | (b) S-0504 claims 2, 4 | It says "created by a vessel". S-0207 claim 23 (a) also creates the same wake animation from drive vehicles. The wording is narrower than S-0207, but there is no behavioural conflict. |
| *aim point* (new) | (b) S-0506 claim 14 | — |
| *airborne* (changed) | (b) S-0506 / S-0501 claim 18 | The S-0307 height test excepts the V3-rocket and dreadnought-missile types (S-0307 claim 26). `combat::weapon::in_air` is false for now. |
| *attack run, firing pass* (new) | (b) S-0501 claims 12-15 | — |
| *drone parasite, grappling parasite* (new) | (b) S-0614 claims 6, 10, 11 | — |
| *hold, held* (new) | (b) S-0614 claims 7, 8, 13 | Affects S-0214 *movable* and S-0207 claim 20 (a). |
| *latching weapon* (new) | (b) S-0614 claim 1 | — |
| *launch pass* (new) | (b) S-0506 claim 2 | — |
| *launching count* (new) | (b) S-0506 claim 10 | Affects S-0214 *movable* and the S-0207 claim 6 hold. |
| *parasite, victim, host* (changed) | (b) S-0614 claims 1-15 | — |
| *release, flight start, detachment* (new) | (b) S-0506 claims 8, 9, 12, 14 | — |
| *rocket missile* (new) | (b) S-0506 claims 14-21 | — |
| *run lock* (new) | (b) S-0501 claim 31 | Affects S-0215 claim 11. |
| *silent removal* (new) | (b) S-0614 claims 4, 13, 15 | — |
| *spawn controller, spawner*, *spawn slot …*, *spawned type, spawn count* (new) | (b) S-0506 claims 1-9; S-0501 claim 35 | Affects the S-0214 `SpawnsNumber` row. |
| *strafing weapon* (new) | (b) S-0501 claims 11, 13, 15 | — |

---

## New quirk switch names introduced or referenced by these hunks

- `quirks.zero_capacity_harvester_cycles` (S-0402 claim 1, default original), (a)
- `quirks.non_unloading_refinery_accepts` (S-0402 claims 8 and 10, default original), (a)
- `quirks.pad_limit_counts_list_only` (S-0407 claim 16; owned by S-0501 claim 30, default on = original), (a)
- `quirks.parasite_exit_search_host_mobility` (S-0212 claim 25 row; owned by S-0614 claim 13), (b)

## (a) items that need work

1. S-0207 claim 23: driving wakes, plus the `[General] Wake` import. The speed half of the "moving now" test is blocked (see the open item below).
2. S-0214 claim 30: move-target marker, plus the `[General] MoveFlash` import (client-side, drawing only).
3. S-0402 claim 5a (Q-0590): integer-step movement test in `display_suppressed`, plus the docs update.
4. S-0402 claim 1: `quirks.zero_capacity_harvester_cycles`, plus the missing capacity-0 importer warning.
5. S-0402 claims 8 and 10: `quirks.non_unloading_refinery_accepts` in `accepts`.
6. S-0407 claim 16: pad total (`NumberOfDocks` default 1, a serialized per-house running total with a floor at 0) and `quirks.pad_limit_counts_list_only`.

Open item: a spec request is needed for "current speed" in S-0207 claim 23 and S-0504 claim 2 (ρ > 0 or *s* > 0). It blocks only the speed half of item 1.

Small follow-up with no behaviour change: re-point the `OREGATH.SHP` citations (harvesting.rs:16, build.rs:3007) to S-0102's Import mapping.

## Interim rules in docs/development.md now answered

- "Harvesting sprite" bullet, *Interim (Q-0590)*: replaced by S-0402 claim 5a's integer-step test.
- No other listed interim rule (Q-0241, Q-0193, Q-0172, the client table) is answered by these revisions. S-0215's Q-0241 bootstrap stays open.
