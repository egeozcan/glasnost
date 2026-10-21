# M7b step 1j claim ledger, sprites and effects group (intake 14B)

This is the sprites and effects group of step 1j (intake 14B). It covers these approved revisions: S-0519 revision 6, S-0610 revision 11, S-0107 revision 19, S-0108 revisions 16 and 17, S-0111 revision 14, S-0109 revision 32, and the S-0102 claim 34 rows and F9 that name them. Every changed claim gets one class, as in [step1q-m7a.md](step1q-m7a.md) and [step1j-m7b-sim.md](step1j-m7b-sim.md):
- *changed*: code changed, with the test;
- *satisfied*: already true, with the test that shows it (a new test where there was none);
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

The charge flash count, the strike mark's phase sequence and the repair depot's slot animations are now simulation state. The save format goes from 67 to 68; the AI game format stays 29. Every sim-gate golden was regenerated, and only the per-tick state hash lines changed. Default play changes in two places, both on the scenario stream:
- a depot slot animation's creation draws (S-0109 claim 28), where depots service vehicles;
- the mark's one ranged draw at the end of Bright hold, where strikes mark buildings.

No quirk switch is new.

| Spec | Revision | Answers |
|---|---|---|
| S-0519 terror and attack animals | 6 | Q-1524 (bomb icon, charge flash, strike mark) |
| S-0610 psychic weapons | 11 (from 10) | Q-1526 (berserk tint) |
| S-0107 infantry sprite sequences | 19 | Q-1525 (generic infantry shadow) |
| S-0108 building sprites | 16 and 17 | Q-1550 (depot slots) |
| S-0111 in-game overlays | 14 (from 13) | none (bomb icon anchor and order, sprite table) |
| S-0109 animation playback | 32 (from 31) | none (editorial for this group) |
| S-0102 draw order | claim 34 rows, F9 | none |

**Retired interims** (the rows under *Visual review 2026-10-19* in `docs/development.md`):
- Q-1524: `live::draw_effects` drew provisional markers for a bomb, a charge and a mark. They are gone. Now the bomb icon is a status extra (S-0519 claim 24), and the charge flash and the mark go through the building's brightness and colour mask (claims 25 and 26, F6 and F7).
- Q-1525: `build::infantry_items` drew no shadow for infantry in the air. Now it draws the generic infantry shadow (S-0107 claim 19).
- Q-1526: berserk units were drawn without a colour. Now they take the berserk mask (S-0610 claim 7, S-0102 claim 34).
- Q-1550: `mapview::slot_play` played the depot's slot rows on the client. The production and special slots are now simulation state (`glasnost_sim::depot_anim`). Only *idle* and *active-1* stay on the client, started and stopped by `Effect::DepotSlot`.

The step-1j sim group's slot seam is filled: `depot::slot_running` now asks `depot_anim::running`.

**New requests** (block Q-1620 to Q-1629; Q-1623 to Q-1629 are unused):
- Q-1620: what the depot's slots do after a send-away step that finds no contact. Interim: the slot change of S-0108 claim 20 happens once, at the send-away step (`DepotState::sent_away`). A later processing without contact changes no slot.
- Q-1621: what a marked building shows when the laser-target selector is outside 0 to 15. Interim: no mask (S-0102 F9's "no selected adjustment"). The importer warns once, as it does for the protection selectors. The mark's brightness pulse is unaffected.
- Q-1622: on the local lane's install, the bomb icon's palette `MOUSE.PAL` resolves nowhere in S-0005's search order, at any game stage or for any side. A scan of every archive, nested ones included, also misses it; `BOMBCURS.SHP` does resolve. Interim: no icon is drawn and a note says so once at load, as for a missing sprite (S-0111). So on a stock install no bomb icon shows. Placement, frame and order are tested with synthetic assets.

## S-0519 revision 6

| Claim | Class | Notes |
|---|---|---|
| Front matter, Terms (flash count, mark clock, mark factor) | (c) | |
| 24: bomb icon image and palette (the bomb-icon image at F3's frame, the fixed cursor palette, no remap, no translucency) | changed | `combat::Cache::bomb_item`, `combat::BOMB_SPRITE`, `MOUSE_PALETTE`. Test: `bomb_icon_claim24`. On a stock install the palette is missing (Q-1622). |
| 24: brightness neutral 1000 ‰; only S-0111 claim 2's light-alpha shading darkens it | changed (brightness), seam (light alpha) | drawn at 1000 ‰. The light-alpha shading is not built for any status extra (the pips do not have it either). |
| 24: placement (a unit's position with height; a building's projected top foundation corner at its cell level) | changed | `Live::draw_status`. The building anchor uses `projection::LEVEL_LEPTONS`. Test: `bomb_icon_claim24` (the unit anchor, centred, no depth test). The building anchor has no test of its own. |
| 24: order (first of the carrier's status extras, so its bar and marks paint over it; after the effects) | changed | the first extra per carrier in `draw_status`. The provisional effect markers that came before the status extras are removed. Test: `bomb_icon_claim24` (the bar and bracket follow the icon). Carrier order follows `world.objects` id order, as for all status extras: a seam shared with them. |
| 24: shown only when claim 10 lets the icon show | satisfied | `bomb::visible`, as built, is the gate in `draw_status`. Test: `visibility_follows_the_planter_and_detectors` (`claims_s0519.rs`). |
| 25: planting sets the flash count to *L*; each building turn lowers it by 1; a later flash replaces the count | changed | `mark::start_flash`, `mark::turn`. Tests: `claim_25_the_flash_count_and_pattern`, `claim_25_the_flash_outlives_the_countdown_and_is_replaced`. |
| 25: flashing body and slot animations; no colour mask | changed | `BuildingLook::brightness` on the body and slot rows. Test: `flash_claim25_body_and_slot_rows`. |
| 26: phase sequence, restart, the one ranged draw, M thresholds, factor table | changed | `mark::turn`, `mark::factor`, `mark::restart` (called from `strike` at a start or redirect). Tests: `claim_26_the_phases_and_the_one_draw`, `claim_26_unmarked_early_makes_no_draw`, `claim_26_the_mark_clock_phases`, `claim_26_the_factors`, `claim_26_restart_by_start_and_redirect_not_by_a_shared_mark`, `claim_26_the_protection_draw_comes_first`. |
| 26: the marked body, voxel turret and barrel, and slot animations take the laser-target mask; the mask applies for every viewer | changed | `Masks::building_look`. Tests: `mark_claim26_body_and_slot_rows`, `building_look_and_mask_tables`. No voxel-turret test: that path is the same code as the protection mask's. |
| 26: map-placed buildings drawn live while flashing or marked | changed | `Live::building_look(o).active()` puts them in the live redraw. |
| F3 (bomb icon frame: q = ⌊D/6⌋, flicker bit, min 11) | changed | `combat::bomb_frame`, in the shown tick. Our engine's choices: q = 0 → 1, F = 0 → b = 0. Test: `bomb_icon_frame_f3`. |
| F6 (flash ±500 ‰ around 1500, on when c mod 4 ≥ 2) | changed | `mark::flash_on`, `mark::flashed`. Test: `f6_the_flash_step`. |
| F7 (b₁ flash, b₂ protection, marked min(2000, ⌊m b₂ / 256⌋)) | changed | `mark::Adjust::brightness`. Tests: `f7_the_combined_brightness`, `mark_claim26_body_and_slot_rows`. |
| S-0513 claim 29's air-strike row (protection combined with the mark's brightness; the protection draw before the mark draw), the 14S seam | changed | `mark::Adjust::brightness` takes the protection brightness as b₂; `mark::turn` runs after the protection turn. Tests: `f7_the_combined_brightness`, `claim_26_the_protection_draw_comes_first`. |
| State and ordering (mark turn right after the protection envelope's turn) | changed | `tick` calls `mark::turn` after the protection turn. Test: `claim_26_the_protection_draw_comes_first`. |
| Determinism (flash and mark survive save and load) | changed | `CombatState::look`, save format 68. Test: `flash_and_mark_survive_a_save_and_load`. |
| Import mapping: bomb-icon image (missing: no icon, the importer warns) | changed | note `BOMB_ICON_MISSING`, given at load. Test: `bomb_icon_claim24` (without the sprite). |
| Import mapping: laser-target selector | changed | `LASER_TARGET_SELECTOR`, read with the protection selectors. Test: `laser_and_berserk_selectors`. Out of range: Q-1621. |
| T25, T26, T27 scenarios | (c) | the lane shots below are our engine's side. |

## S-0610 revision 11

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision, Q-1526), Terms (berserk selector, berserk mask) | (c) | |
| 7: berserk voxel vehicle body and voxel turret and barrel: berserk mask, united with the curtain mask | changed | `build::vehicle_voxels` mask = curtain ∪ berserk. Test: `berserk_claim7_rows`. |
| 7: berserk sprite vehicle and voxel turret on a sprite body: none | changed | `Masks::berserk_of(.., SpriteVehicle, ..)` gives 0. Test: `berserk_claim7_rows`. |
| 7: berserk infantry sprite body: the mask, none while its cell centre is shrouded for the viewer | changed | `Mobile::berserk` carries the shroud test (`econ::shroud::explored`). Test: `berserk_claim7_rows`. |
| 7: aircraft and buildings: no mask; no brightness change | changed | Tests: `building_look_and_mask_tables` (aircraft), `berserk_claim7_rows` (no brightness change). |
| Edge cases: selector outside 0 to 15 gives no mask and the importer warns | changed | `glasnost_convert::sim`, next to the protection selectors. Test: `laser_and_berserk_selectors`. |
| P4 scenario text | (c) | |

## S-0107 revision 19

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision, Q-1525) | (c) | |
| 19 (a): own shadow only at h = 0 | satisfied | `infantry_items`, as built. Test: `generic_shadow_height_claim19b` (no generic shadow at h = 0). |
| 19 (b): shadow height hₛ over a high bridge (h − 416 when the lower neighbour is also a deck) | changed | `live::generic_shadow`, using `cells.axis`, `high_bridge` and `hover::DECK`. Test: `generic_shadow_height_claim19b`. |
| 19 (c): drawn exactly when hₛ > 0; never for drop-pod movement | changed (generic), seam (drop-pod shadow) | drop-pod types get no generic shadow. The drop-pod shadow of S-0102 claim 13 is not drawn, since drop-pod movement is not modelled. |
| 19 (d): image frame 1 of `INFSHDW.SHP`, a darkening, centred L(hₛ) rows below the draw point, right before the body; fewer than 2 frames draws nothing, without a warning | changed | `build::GENERIC_SHADOW_FILE`, `GENERIC_SHADOW_FRAME`. Test: `generic_infantry_shadow_claim19`. |
| Import mapping (`INFSHDW.SHP` by name lookup) | changed | in `wanted_files`. |
| M20 scenario text | (c) | |

## S-0108 revisions 16 and 17

| Claim | Class | Notes |
|---|---|---|
| Front matter | (c) | |
| 2 (r17): a protected building's bib, under-door, replacement and door images take S-0513 claim 29b's appearance | satisfied | `accessory_row`, as built. Test: `protection_claim29b_accessory_rows`. |
| 12: condition swap restarts the running modelled slots | changed | `depot_anim::swap`, checked at a slot start and in the depot's own turn (our placement). Test: `claim_12_condition_swap_restarts_the_modelled_slots`. |
| 20: working entry starts special-1 and stops active-1 and idle | changed | `depot_anim::working_entered`. Test: `claim_20_working_entry_starts_special_1`. |
| 20: special-1 ends in repair and contact → special-2; otherwise idle | changed | `depot_anim::anim_turn`. Test: `claim_20_special_1_ends_into_special_2`. |
| 20: end of service: production and special-2 stop, special-3 and active-1 start; special-3 ending starts idle | changed | `depot_anim::service_ended`. Test: `claim_20_done_ends_the_service`. |
| 20: every-tick row (not naval, not in repair, production or special-2 running) | changed | `depot_anim::building_turn`. Test: `claim_20_every_tick_row_and_the_naval_exception`. |
| 20 (r17): stopping or replacing special-1 triggers no follow-up | changed | `stop` and `start` remove without the follow-up. Test: `claim_20_special_1_ends_into_special_2`. |
| 20 (r17): slot durations are deterministic simulation state; the settling wait does not depend on drawing | changed | `DepotAnimWorld`, in the update order and saved. Tests: `claim_22_settling_hold_end_to_end`, `depot_slots_survive_a_save_and_load`, `a_slot_creation_draws_on_the_scenario_stream`. |
| 20: idle and active-1 (presentation) | changed | `Effect::DepotSlot` → `Slots::depot` on the client. |
| 13, 14: power rules and the special-outage slot for depot slots | seam | not modelled for the depot's simulation slots (`depot_anim` module comment). |
| Import mapping (the slot names; empty damaged name takes the normal one) | changed | `sim_econ` `slot_names`. Test: `slot_model_names_are_converted`. |
| B18 scenario parts | (c) | |
| S-0314 claim 6 (production hold) and claim 22 (settling waits for special-3), the 14S seam | changed | `depot::slot_running` → `depot_anim::running`. Tests: `claim_6_production_hold`, `claim_22_settling_hold_end_to_end`. |
| S-0504 claim 8 (a naval yard takes claim 20's slot rows), the 14S seam | changed | a yard runs through `depot_anim` as a depot does, with claim 20's naval exception. Test: `claim_20_every_tick_row_and_the_naval_exception` (a depot and a yard type); lane `ship-repaired-at-yard`. |

## S-0111 revision 14

| Claim | Class | Notes |
|---|---|---|
| Front matter | (c) | |
| 1: the bomb icon is placed from the carrier's draw point, not the status anchor; it is the first of the carrier's extras | changed | as S-0519 claim 24 above. Test: `bomb_icon_claim24`. |
| 1: culling against the widened view | seam | not built for any status extra. |
| Import mapping table (`BOMBCURS.SHP`, `MOUSE.PAL`) | changed | both in `wanted_files`. `MOUSE.PAL` is not found on the lane install (Q-1622). |
| Backing and confidence | (c) | |

## S-0109 revision 32 and S-0102 claim 34

| Claim | Class | Notes |
|---|---|---|
| S-0109 front matter, Terms (being erased), claim 21 (slot-animation erasure only; the body's own translucency is S-0514 claim 27's) | (c) | the drawing already splits them this way. |
| S-0109 claim 9 table (the shot animation's creator settings) and backing | (c) | for this group; the projectile group owns them. |
| S-0102 claim 34: berserk rows | changed | as S-0610 claim 7. |
| S-0102 claim 34: marking rows (body in both passes, voxel turret and barrel, building animations over the marked cell by the cell rule; the union with the force-shield mask) | changed | `Masks::building_look`; the slot-animation mask uses the cell's building look. Tests: `mark_claim26_body_and_slot_rows`, `building_look_and_mask_tables`. |
| S-0102 claim 34: curtain adds no mask to a marked building; the charge flash adds no mask; shadows keep plain darkening | satisfied | `building_look_and_mask_tables`; shadows are `Paint::Shadow`. |
| S-0102 F9 (no selected adjustment: K = 0) | satisfied | `protection_selector` returns no mask outside 0 to 15. |

## Lane pictures

These were written to `local-lane/visual-fix-4/sprites/` by `m6c_lane` (scene `terror`), `m6b_lane` (`depot`, `yard`, `paratroopers-dropping`) and `m7a_lane` (`berserk`, the new `berserk-mask`). The terror scene now also saves:
- the charge flash on and off;
- the mark in Bright hold, near the top of a Pulse rise and near the bottom of a Pulse fall;
- the bomb 40 ticks after planting.

The depot scene counts any health rise as a repair step, because the vehicle repairs while its link stays untethered. It also prints the simulation's depot slot animations and saves the depot after the service ends.

- `terror-charge-flash-on.png` and `-off.png`: the charged building is visibly brighter while the count is 11 (on) than at 9 (off).
- `terror-strike-bright-hold.png`, `-pulse-rise.png` (factor 480) and `-pulse-fall.png` (factor 288): the marked building is red from the laser-target mask, and brightest in the hold. The fall is clearly darker than the rise.
- `terror-bomb.png` and `terror-bomb-later.png`: no bomb icon, because `MOUSE.PAL` is missing (Q-1622). The note is present.
- `paratroopers-descending.png`: each descending paratrooper casts a small dark generic shadow on the ground far below it. A run with the shadow switched off changed exactly those pixels: 6 blobs of 33 pixels each.
- `berserk-mask.png`: the berserk vehicle and the berserk infantry are red, and their plain twins are not. The dark, shadow-shaped pixels under the berserk infantry's feet turn red too. They belong to the body image, not the shadow item. Switching the own-shadow item off changed no pixel, and this type's shadow frames decode empty. So they are opaque body pixels that F9's union recolours.
- `vehicle-repaired-at-depot.png` and `-after.png`: the arm reaches down to the vehicle (simulation slot *special-2*) during the repair, and is raised (*special-3*) after it.
