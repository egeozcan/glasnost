# M6 step 2: movement, rendering and order spec revisions of the 03:27 round

This sorts every changed hunk in `git diff 8c9ed36 main` for S-0105, S-0106, S-0107, S-0109,
S-0111, S-0010, S-0206, S-0207, S-0209, S-0210, S-0211, S-0212, S-0214 and S-0215, plus the
answer to Q-0600. The round is commits 6c5ec1b to 4b0377e. S-0207 has two commits in it:
128c047 (rev 18 to 19) and d4f5f8e (rev 19 to 20, the Q-0600 answer).

Buckets: **(a)** changes or clarifies behaviour the engine already implements.
**(a, already done)** means the code already matches. **(a, seam)** means the target code does not
exist yet. Plain **(a)** means code must change. **(b)** is a pointer into S-0501, S-0504, S-0506 or
S-0614. **(c)** is editorial.

Evidence comes from committed `main` at ab7dd88, which includes the lane-B1 (S-0614) merge
1c636d7. Line numbers are `git show main:<path>` lines. The diff from 4b0377e to main adds only
new request files under `spec/`, so `8c9ed36..main` equals the round for these 14 specs.

Every file's front matter changed (revision; approved-on for most; S-0010 `sources-class` gains
`design`; S-0207 `requests` gains Q-0600). That is (c) and is not repeated below.

Several hunks repoint text from "the cloaking / visibility / special-abilities / bridge /
transport specs" to the new S-0516 (cloak, disguise, sensors), S-0511 (bridges), S-0507
(transports) and S-0503 (hover). Those specs are not in this sort's (b) list. None of their
mechanisms exists in the sim or the app yet: no cloak state, no disguise, no hover locomotor, no
transport contents and no bridge destruction. Hunks that only change such a pointer are (c).
Hunks that state a new behaviour through such a pointer are (a, seam).

---

## S-0105 Rendering of voxel models (rev 13 to 14)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Cloak state* | (c) | The four states are now S-0516 claim 1's, and S-0516 owns the transitions. Same states, same "only uncloaked is fully visible" rule. | No cloak state is modelled (`combat/check.rs:312`). |

## S-0106 Slope shapes, surface height and voxel tilt (rev 6 to 7)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Tilt orientation* | (c) | Points at "C3's construction rule" in place of "table C3". | — |
| Terms, *Powered down* | (a, seam) | Hover height is world height minus the surface height at the unit's position, minus 416 more on a bridge deck, in exact integers. The unit is powered while switched on or while hover height > 0. Settling and switching off belong to S-0503 claims 8 and 10. | No hovering locomotor exists in `sim/src`. |
| 4 Fixed tilt per shape (corrected angles; the old angles kept as an oracle table) | **(a)** | Unconditional presentation correction with no switch. Nominal tilt is arctan(1/√6) for shapes 1 to 4 and π/6 for shapes 5 to 16, with table-trigonometry precision. The original angles (29.3115°, 21.3574°) are for oracle comparison only. | `crates/render/src/tilt.rs:39-62` `tilt_orientation` returns C3's historical numeric rows, and its tests (`tilt.rs:247-300`: 9.7559°, 19.5996°, 29.3555°, 22.0749°, 7.1054°) assert original-angle values. See F2/C3/F3 below. Settled *R*ₛ (C4) and tilted drawing are not wired (`tilt.rs:11-14`, `app/src/mapview/quirks.rs:223`), so nothing on screen changes yet. |
| 5 table, hovering row | (a, seam) | "Positive height" becomes "strictly positive hover height". A switched-off hover tilts only once it settles to 0. Confidence raised. | No hover locomotor. |
| 7 (endpoint differences) | (c) | The numeric examples are now marked as original-angle examples. | — |
| F2 corrected-angle paragraph; τ_A and τ_B now "oracle reference only" in Constants | **(a)** | Corrected *R*ₛ = F2 with arctan(1/√6) and π/6 in double precision, rounded under S-0105 F0, rotations unchanged. One angle convention is used for body, turret, barrel, shadow and transition orientations. | Same as claim 4. When C4 is implemented it must use F2 with the corrected angles, not the C4 rows. |
| C3 heading and note; C4 heading and note | **(a)** | The numeric rows are oracle references. Engine orientations come from C3's construction rule (axis (cos α, sin α, 0) normalised with the table square root; half-angle sin/cos from the S-0105 F1 table) using the corrected τ. | `tilt.rs:41-62` must build *q*ₛ by that rule. Keep the current rows as a separate oracle function. |
| F3 start/end orientations | **(a)** | Both F3 endpoints use the corrected-angle construction. Oracle comparisons use the original rows. | `tilt.rs` `TiltState::orientation` / `interpolate` callers take `tilt_orientation`, so they follow once it changes. |
| Edge cases, *Shape-dependent angles*; *Interpolated endpoints* | (c) for this table (the behaviour is covered by claim 4) | The withdrawn-switch note becomes "corrected unconditionally". No switch is added. | No quirk field is involved. |
| Scenario preamble, *corrected renders*; T7 (deck control, re-enabling, spin); T11 engine expectation | (c) | Scenario text. | — |
| Claim 5 confidence; Open questions (hover power resolved by S-0503) | (c) | — | — |

New quirk switches: none (an angle switch is explicitly not added).

## S-0107 Infantry sprite sequences (rev 10 to 11)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *drawn as its disguise for a viewing house* (new) | (a, seam) | Presentation test of S-0516 claim 37, separate from the game-state test *appears disguised* (S-0516 claim 35). | No disguise anywhere (`app/src/mapview/quirks.rs:103-104`: "Disguise is not modelled"). |
| 13 Disguise | (a, seam) | The disguise type's sequence table is used when the unit is *drawn as its disguise* for the local house. Otherwise the unit's own type is used. Presentation only. | Same. |
| M13 | (c) | Scenario. It adds the owner-view blink (mod 256 in 8 to 55) from S-0516 claim 37. | — |

## S-0109 Animation playback (rev 19 to 20)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *building fade level* | (a, seam) | The fade level now has a defined schedule: +1 per tick while cloaking, −1 while uncloaking, 15 ticks each way (S-0516 claim 18). | The app draws slot animations at fade level 0 only (`app/src/mapview/slots.rs:252`). No cloak state. |
| Terms, *cloak*, *sensors*, *cloaked from the viewer* | (c) | The same viewer rule (owner, sensed cell, mutual alliance), now by pointer to S-0516 claims 18, 21 and 28. | — |
| 20 (source of *t*; fade 11 to 14 sentence; confidence) | (c) | The parenthetical "from fade level 11 in the original" becomes S-0516 claim 28's rule. Same content. | `quirks.anim_low_detail_shows_cloaked` is unchanged (`app/src/mapview/quirks.rs:280`). |
| Open questions (fade-level item removed) | (c) | — | — |

## S-0111 In-game overlays (rev 9 to 10)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 3 When the health bar shows (disguise case) | (a, seam) | A hovered unit that *appears disguised* to the local house (S-0516 claim 35) shows a bar only when its disguise has a house (claim 36). Selected units, and hovered units that do not appear disguised, are not restricted. Confidence becomes high. | The hover bar is drawn without a disguise test (`app/src/mapview/live.rs:949`, `game/session.rs:883`). No disguise state exists, so the current code is correct for every reachable state. |
| 18 Insignia (disguise / cloak, buildings at fade 11 to 15, fading units) | (a, seam) | Insignia are hidden while the unit *appears disguised* (not "drawn as"), and while cloak presentation hides the object (S-0516 claim 28). A building at fade levels 11 to 15 follows the fully-cloaked rule. Fading units keep their insignia except in the non-owner hidden band. Skirmish only. | `app/src/mapview/combat.rs:235` `insignia` and `live.rs:1001` have no cloak or disguise input. Nothing cloaks, so the current output is right for every reachable state. |
| Table "Object state", hovered row | (a, seam) | Hover: bar "if claim 3 allows", pips "if claims 3 and 9 allow". The pip scale on hover now follows the same disguise gate. | Same seam as claim 3. |
| OV3, OV13 | (c) | Scenarios. | — |
| Open questions (two items removed) | (c) | — | — |

## S-0010 Theater data, tilesets and terrain values (rev 15 to 16)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Scope note (two corrections); Behaviour preamble (inherited confidence) | (c) | — | — |
| 6 backing row and confidence | (c) | — | — |
| 10 Ground smoothing (the R4 overlap fixture is now a fixed result; no general ordering claim) | (a, already done) | For R4's fixture (rough and sand transitions sharing *L* … *L*+15) the loaded centre is *G* and all four arms are *L*+15. Other overlaps are outside the claim. | `map/src/terrain.rs:443-500` `smooth_ground` runs the pairs in turn on one tile. Worked through: an arm gets rough *L*+11, then sand *L*+15 (the centre *G*ᵣ is not in the sand pair). The centre keeps *G*ᵣ, whatever order the cells are visited in. No test pins R4 yet; a small synthetic test would be cheap. |
| 13 Variant choice: every sub-tile, 0 included, uses the divided origin | **(a)** | Unconditional drawing correction with no switch. **`quirks.variant_origin_mismatch` is withdrawn.** | `convert/src/theater.rs:161-174` `variant_image(..., origin_mismatch, ...)` uses (x, y) for t = 0 while the flag is on. The flag defaults to on (`theater.rs:69-72, 88`). The caller is `app/src/mapview/build.rs:1349` (`q.variant_origin_mismatch`). The flag is also listed in the quirk catalogue at `rules/src/quirks.rs:2562`. |
| 13 alternate-image state (bridge damage/repair of S-0511 as examples) | (c) | Pointer only. No event sets the state yet. | `theater.rs` `variant_image` returns 0 for a marked cell. |
| 13 backing and confidence | (c) | — | — |
| 16 backing and confidence (G2 rewritten) | (c) | — | — |
| 24 Missing tile cells: fall back to the plain image's tile cell | **(a)** | Unconditional, no switch. **`quirks.variant_cell_missing` is withdrawn.** The fallback paints the plain tile cell at *t* mod (*w*×*h*), with its diamond, extra image and depth. The selected variant still supplies the radar colour, or S-0410 claim 9's dark grey. If the plain image also has no cell there, nothing is painted and the background is left black. | `app/src/mapview/build.rs:1366-1378`: the fallback runs only with the flag off (`None if !q.variant_cell_missing`), and it indexes `plain.cells.get(s.max(0))` rather than `s.rem_euclid(w × h)`. The radar key is already the shown variant's (`build.rs:1364-1365`, `radar.rs:57`): (a, already done) for that part. |
| 24 backing and confidence; claim 25 backing/confidence table and the S-0511 pointer | (c) | — | — |
| Formulas, Variant (and the negative-origin note) | **(a)** | Same as claim 13. | Same. |
| Constants, transition pairs note | (c) | — | — |
| Edge cases: the two variant switches replaced by "unconditional corrections" | **(a)** | Removal of both quirk names. | `convert/src/theater.rs:69-76, 88-89` (`TheaterQuirks` fields), `rules/src/quirks.rs:2561-2568` (catalogue entries), the `theater.rs` tests that build `TheaterQuirks { .. }` with these fields (around `theater.rs:804, 946`), and `build.rs:1269, 1349, 1369`. |
| Scenarios X4 (engine comparison), V1 part B, V2, R4, G2, L-series shadow run 4 | (c) | Scenario text. X4's and V1 B's engine comparisons describe the corrected behaviour above. | — |
| Open questions (bridges now S-0511; overlap note) | (c) | — | — |

Quirk switches removed: `quirks.variant_origin_mismatch`, `quirks.variant_cell_missing`. Added: none.

## S-0206 Turn rates and turning (rev 10 to 11)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Turret facing* (turret-less unseen facing) | (a, already done) | The unseen turret facing takes the body's target when a facing error re-aims the body. Equality during other movement is no longer claimed. | `combat/behave.rs:65-72` turns the turret to `body.target` in that case only. |
| 4, 6 backing | (c) | — | — |
| 9 priority preamble; turret requirement; turret-locked row reworded ("neither turret-locked nor omni-fire") | (a, already done) | Priority is spin, then target aiming, then facing error. Target-aiming and rest rows need a turret; the spin row does not. "No change" rows supply no target. | `turret.rs:31-66`: target rows and rest rows only when `u.turret`. The spin row runs on `u.turret_spins` alone (`turret.rs:62`). `combat/behave.rs:107-140` `turret_target_rows` pushes facing error, then target aiming. Turret-locked weapons are not imported (`behave.rs:133` comment), a pre-existing seam not changed by this hunk. |
| 9 "Several rows in one tick" and F5 (explicit composition order; *s*ᵢ semantics; Δᵢ = δ(*s*ᵢ₋₁, τᵢ)) | (a, already done) | The order is facing error, then target aiming, then spin (lowest to highest priority). Each replacement turn starts from *s*ᵢ₋₁, and the same target again keeps *s*ᵢ = *s*ᵢ₋₁. This states the existing rule explicitly. | `turret.rs:57-66`: `turn_to` per row in that order, then the spin row from `turret.value(t)`. Test `turret.rs:87-` (R4 run f example). |
| 9 backing (unmeasured items) and confidence | (c) | — | — |
| 10 (a), (b) result priority by S-0301 claim 9; (b) turret-less body re-aimed only when a facing error is given | (a, already done) | Wording. A higher-priority refusal wins over out-of-range or turning. | `combat/check.rs` runs ranks in S-0301 claim 9 order (cited at `check.rs:349`). |
| 10 (c) visceroid weapons never give a facing error | (a, already done) | New sentence, by pointer to S-0310 claim 2. | `combat/check.rs:107-108` (`wr.omni_fire \|\| u.visceroid`). |
| 10 backing | (c) | — | — |
| 11 Spin with target, scope narrowed (turret, valid weapon, not locked or omni) | (a, already done) | The with-target consequences apply only when target rows can act. | Follows from `turret_target_rows` needing `tw.is_some() && !omni`. |
| 13 Hovering locomotor (body aims at a look-ahead point; heading drives travel) | (a, seam) | The body aims one neighbour step beyond the heading point while throttle > 0 (S-0503 claim 5). The body does not steer. | No hover locomotor. |
| 14, 15, 17 backing | (c) | — | — |
| F1 (same target again leaves the turn unchanged); F2 (ω ≥ 1) | (c) | Restates claim 3 and the domain. | `turning.rs` `turn_to` already ignores a repeated target (claim 3, as tested in `turret.rs`). |
| Edge cases, `quirks.turret_spin_quantised` off: "at ω = 0 this alternative does not spin" | **(a)** | Clarifies the off branch. | `rules/src/quirks.rs:103` declares the field, but **no code reads it**: `turret.rs:12-16` `spin_target` always rounds and adds 2048. The off branch (target = facing + 256 × min(ω, 8), no rounding; no spin at ω = 0) is missing in `turret::tick` (`turret.rs:62-65`). |
| Edge cases, the remaining text; Measurement scenarios (preamble, hover heading scenario, R2 d/e, R3 e/f, R4 h, R6 i, R8, R9, R12); Open questions | (c) | — | — |

New quirk switches: none.

## S-0207 Ground vehicle movement (rev 18 to 20)

### Commit 128c047 (rev 18 to 19)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *heading point* (hover = next cell centre of S-0503 claim 5) | (a, seam) | Hover heading point defined. | No hover locomotor. `path::heading_point` serves driving only. |
| 15 table T4, *cloaked blocker* row (and the T-table summary row) | (a, seam) | Every cloaked object in the cell's ground layer, of any house, starts uncloaking for every house (S-0516 claim 11 e), in place of "visible to the mover's house". Path handling is unchanged. | `drive.rs:574-575`, `drive.rs:655-656`: "nothing is cloaked … nothing to reveal". Hook the uncloak there once S-0516 exists. |
| 16 confidence (hover heading point) | (c) | — | — |
| 19 (d) hovering arrival-time stop (an already computed final step allowed; no later travel) and the T-table row | (a, seam) | The hover may finish an already computed positive step after the stop. | No hover locomotor. |
| Scenarios D15 (f), run 4, D21 runs; Open questions (heading points) | (c) | — | — |

### Commit d4f5f8e (rev 19 to 20): the Q-0600 answer

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 23 Driving wakes, *current speed* | **(a)** | *Moving now* = body turning, or a heading point and **integer rate *s* > 0**. Here *s* = trunc(*s*₂ × ρ) is claim 2's rate **evaluated with the speed fraction and factors in force at the wake check**. The spec rules out ρ itself, accumulated progress, distance travelled, and "a remembered rate from the last moving tick". A positive *s* with no position change still qualifies. | `drive.rs:1436-1439` reads `d.step`, the *s* stored by `record_step` (`drive.rs:355-358`), which `drive::tick` sets only at `drive.rs:1309-1310` and `1355-1356`. That diverges in two cases. (1) When a second track begins in the same tick (`drive.rs:1315-1320`, `1360-1366`), `update_rho` runs again but `step` is not re-recorded, so the wake uses the *s* of the earlier ρ. (2) On a tick where the locomotor runs but earns nothing new, the value is stale from the last moving tick. Fix: in `driving_wake`, compute *s* fresh with `progress(w, id, u)` (= `speed_estimate`, `drive.rs:368-376`, which already evaluates F2 with the current ρ and has the same "no flag carrier" note). Keep `d.track.is_some()` as the heading-point test. Leave `DriveState::step` in place for S-0402 claim 5a (`econ/harvest.rs:573`). |
| 23 heading point = active track target, not a retained path destination | (a, already done) | Waiting or being between tracks does not qualify, but turning in place still can. | `drive.rs:1438` (`d.track.is_some()`) and `o.body.turning(w.tick)` (`drive.rs:1436`). |
| 23 timing: post-movement state, newly begun track included; post-move position; inside the vehicle's own locomotor turn, after movement's draws; not an end-of-tick pass | (a, already done) | Confirms the interim placement. | `tick.rs:145-149` calls `driving_wake` right after `drive::tick` in the vehicle's turn. `driving_wake` reads `o.pos`, `o.layer`, track and body after the move (`drive.rs:1434-1450`). A completed track with no successor leaves `track = None` (via `arrive_or_rest`), so no wake, as claim 23 requires. |
| 23 "its animation enters the update order under S-0203 claim 4" | (a, seam) | — | Shared with every sim-created animation. `combat/effects.rs:115-127` takes only the creation and start draws (S-0109 claim 3) and places no animation in the update order. This seam predates the hunk and is not specific to wakes. |
| 23 draws: only the animation's creation draws, after any movement draws | (a, already done) | — | `combat::effects::anim` → `load::anim_created_at` (`effects.rs:121`), called after `drive::tick`. |
| 23 backing (D25a, Q-0600) | (c) | — | — |
| Import mapping, new "No data key" row | (c) | No new field, and no wake-speed key. | — |
| Scenario D25a (new) | (c) | — | — |

Held vehicles: on main (`tick.rs:139-144`, from the lane-B1 merge) a held drive vehicle takes the
`Some(Locomotor::Drive) if held => false` arm. Neither `drive::tick` nor `driving_wake` runs for
it. That is consistent with S-0614 claim 8 (a held unit's driving locomotor is not processed)
and S-0207 claim 23 (the wake is created "as part of its locomotor movement"). After item 1's
fix this placement matters more. A held vehicle that still has a track and ρ > 0 would make
wakes if `driving_wake` were ever moved out of the non-held arm. Keep it inside.

New quirk switches: none.

## S-0209 Occupancy and crowding (rev 26 to 27)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 7 cloaked-blocker search | (a, seam) | The search sees the ground-layer uncloaking of claim 9. Deck objects are not uncloaked. | No cloak state. `walk.rs:477, 513` handle `CloakedBlocker` as a path outcome only. |
| 9 *Cloaked blocker* (global, ground-layer-only uncloaking) | (a, seam) | Every cloaked ground-layer object in the blocking cell, of any house, starts uncloaking for every house on each such answer (S-0516 claim 11 e). | Same. `occupancy.rs:5`: "nothing is cloaked". |
| 9 backing and confidence | (c) | — | — |
| State table, cloaked-blocker row | (a, seam) | Same as claim 9. | — |
| Scenarios C8 (f), C19 (d), (d2) | (c) | — | — |

## S-0210 Crushing (rev 18 to 19)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 11 Automatic crushing: "does not appear friendly" becomes "does not appear disguised" (S-0516 claims 35-36); not applied to a drive-through or to the omni-crusher / attack-target variants | (a, seam) for the automatic choice; (a, already done) for the variants | The disguise test belongs only to the automatic choice. | The automatic choice of the first list is not implemented: `general.player_auto_crush` is imported (`sim/src/rules.rs:491, 873`) but nothing reads it in `sim/src`. The omni-crusher and attack-target variants in `approach.rs:175-215` (`crushing_approach`) have no disguise test, as the claim now says. No disguise state exists. **M8 A3 intake 52a:** S-0210 r29 and S-0215 r88 claim 39 (Q-2005) make the automatic choice the *crush response* to a hit, built in `crush::respond_to_hit` with the disguise test; `general.player_auto_crush` is now read, behind `quirks.auto_crush_computer_only` ([ledger](../m8-claims/intake52a-S-0210-S-0215-S-0216.md)). |
| 11 backing; scenario K13 (new); Open questions (appears-friendly item removed) | (c) | — | — |

## S-0211 Land types and passability (rev 15 to 16)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 15 (d) and backing | (c) | The affected cells are now S-0511 claims 5 and 19's. The backing is inherited from them. | High-bridge destruction is not implemented (`bridges.rs:8-9`). |
| 15 (a), rev 17 (historical responsibility) | (a) | A section's own cell selects itself; another marked cell selects its last coverer; no nearby substitute. | `Cells::section_self`, `Cells::covered_by`, `Cells::responsible`; `marking_keeps_historical_responsibility` in `claims_s0211.rs`. |

## S-0212 Pathfinding (rev 30 to 31)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 3 bridge links in play (named cells from S-0511 claims 12 and 19; low bridges name none; table wording; claim 5 runs after low-bridge changes too) | (a, seam) | A successful direct attempt on a high bridge names one cell. Repair names each restored pier's reference cell, the far ramp, and the cell before it. Low-bridge events name no cell but still update connectivity. | `bridges.rs:1-9`: links are built once at game start, and "changes of links in play … wait for the bridge specs". `bridges.rs:113` `link_within` is ready for the lookup. |
| 3 backing | (c) | — | — |
| 22 Blocked next step extended to hovering (moving-blocker timer; immediate mode-0 search on first *blocked* or *cloaked blocker*; a repeated cloaked blocker keeps the destination) | (a, seam) | The hover rules (owned by S-0503 claim 12). The driving rule is unchanged, now scoped "for a driving unit". | No hover locomotor. The driving path is unchanged (`drive.rs:574`, S-0212 claim 22 cites). |
| 22 backing and confidence; P18 hover variant (new); Open questions (bridge item removed) | (c) | — | — |

## S-0214 Order targeting rules (rev 26 to 27)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *deploy readiness* (vehicle-transport unload exclusions are S-0507 claim 7's: bridge, a deck on the cell or a straight neighbour, water, a non-bunker building cell, disabled, tunnel; the water refusal applies to human and AI player houses, while neutral/special keep the exemption) | (a, seam) | The details were low-confidence placeholders before. | `targeting.rs:1035-1064` `vehicle_deploy`: readiness is only "not on a tunnel cell" (`targeting.rs:1040`). Transports never hold passengers (`targeting.rs:1063`, row VD6), so the transport rows are unreachable. Implement with S-0507. |
| Terms, *Bridge repair hut* (cursor test S-0511 claim 21; entry effects check no owner or alliance; `quirks.repair_consumes_engineer_always`) | (a, seam) | — | `targeting.rs:864-870` row I3 always returns *no-repair* ("no bridge is destroyed in our engine yet"), with no owner test, which is consistent. The quirk is S-0511's (not in `rules/src/quirks.rs`). |
| Terms, *Can be selected* (fully cloaked, unsensed, non-owned objects cannot be clicked; fading objects can; units that appear disguised as terrain/overlay cannot be selected) | (a, seam) | — | `targeting::selectable` (development.md "Selection") has no cloak or disguise input. Nothing cloaks or disguises. |
| Terms, *Docking answer* (capacity refusal vs eligibility *neither*; the vehicle-transport and air-transport *neither* cases; *neither* yields no entry verdict) | (a, seam) | — | `targeting.rs:15-16`: no contact exchange exists, so the answer is always *neither*. `targeting.rs:834, 1105` (rows x3/x4, n1-n3) wait for it. |
| 7, 8 backing | (c) | — | — |
| 12 cloak-driven selection loss (at cloak start, every non-owner loses the object; later, a fully cloaked unsensed object is removed) | (a, seam) | Client selection. | No cloak state. |
| Table I row I3; table IC row IC2 | (a, seam) | The cursor test is S-0511 claim 21's, with no owner or alliance test. | `targeting.rs:864` (I3). IC2 needs fog of war, which does not exist. |
| Table VD rows VD7 and VD8 | (a, seam) | VD7 is independent of S-0507 readiness. VD8 uses S-0507 claim 7 for transports. | `targeting.rs:1063` (transports hold nothing). |
| Cross-spec input-check paragraph; scenario T5 / T10 text; Open questions | (c) | — | — |

## S-0215 Behaviours (rev 26 to 27)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Cloak, sensor* | (c) | Pointer to S-0516 claims 1-9 and 21-23. | — |
| 13 (unload *guard* allowance owned by S-0507 claim 11) | (c) | Pointer. | — |
| 29 Enter (in the transport loading exchange only acceptance or an existing tether keeps *enter*; any other reply ends contact and falls back to rest) | (a, seam) | Confidence high for loading replies. | No transport entry exchange. `process.rs:577` handles *enter* for harvesters only. |
| 40 pointer list (*unload* to S-0507 claims 7-13) | (c) | — | — |

---

## Hunk count per bucket

Counted by table row above, excluding the front-matter note.

| Bucket | Rows |
|---|---|
| (a) needs code change | 10 (S-0106: 4 rows, one change; S-0010: 4 rows, two changes; S-0206: 1; S-0207: 1, the claim 23 rate) |
| (a, already done) | 10, plus S-0210 claim 11's variants part |
| (a, seam) | 27 (S-0210 claim 11's automatic-choice part included) |
| (b) | 0 |
| (c) | 43 |

No hunk in these specs points into S-0501, S-0504, S-0506 or S-0614 as new behaviour. Claim 23
still names S-0504 claim 2 as owner of the ship wake rule, and that was already there.

## New quirk switch names

- None added in these specs.
- **Withdrawn:** `quirks.variant_origin_mismatch` (S-0010 claim 13) and
  `quirks.variant_cell_missing` (S-0010 claim 24). Both are in code:
  `convert/src/theater.rs:69-76, 88-89` and `rules/src/quirks.rs:2562-2567`.
- Referenced, owned elsewhere: `quirks.repair_consumes_engineer_always` (S-0214 Terms; owned by
  S-0511 claim 17). It is not in `rules/src/quirks.rs`, and its target code does not exist yet.
- Existing but unimplemented: `quirks.turret_spin_quantised` (S-0206) is declared at
  `rules/src/quirks.rs:103`, and nothing reads it.

## (a) items that need work

1. **S-0207 claim 23 (Q-0600), wake rate.** In `drive::driving_wake` (`drive.rs:1423-1451`),
   replace `d.track.is_some() && d.step > 0` with `d.track.is_some() && progress(w, id, u) > 0`.
   `progress` is the same F2 evaluation as `speed_estimate`, with the ρ in force after the
   tick's movement. Update the three citations that say "open in Q-0600"
   (`drive.rs:1433-1435`, `tick.rs:147`, `tests/claims_s0207.rs:1885-1887`) to cite S-0207
   claim 23 and Q-0600 as answered. `git grep Q-0600` on main finds no other code site, and
   no ship-wake site in `naval.rs`. Update the
   test `claims_s0207.rs:1886-1928`, which sets `step` directly. It should drive ρ instead,
   with one case of ρ > 0 but *s* = 0 (no wake), one case of positive *s* after a second track
   begins in the tick, and one held or stale case. Size: small (about 5 lines of code plus
   test changes).
2. **S-0207 claim 23 docs.** Retire the `drive::driving_wake` interim row in
   `docs/development.md` (line 256 on main). Replace it with a plain description that cites
   S-0207 claim 23 rev 20. Size: trivial.
3. **S-0010 claims 13 and 24: withdraw two quirks and make the corrections unconditional.**
   - Remove `TheaterQuirks::variant_origin_mismatch` and `variant_cell_missing`
     (`convert/src/theater.rs:69-76, 88-89`) and their catalogue entries
     (`rules/src/quirks.rs:2561-2568`).
   - Drop the `origin_mismatch` parameter of `variant_image` (`theater.rs:161-174`), so
     every *t* uses the divided origin.
   - In `app/src/mapview/build.rs:1366-1378`, always fall back to the plain image's tile cell.
     Index it with `s.rem_euclid(plain.width * plain.height)` instead of `s.max(0)`.
     If that cell is missing too, paint nothing and keep the existing `drop_object` note.
   - Fix the theater tests that set the fields (around `theater.rs:804, 946`) and the app
     tests for V1 B / X4, if any.

   - Add both names to the `WITHDRAWN` list of `every_spec_quirk_has_a_switch`
     (`rules/src/quirks.rs` tests), following the S-0413 precedent.

   Neither CI check fails today. That test only checks the spec-to-code direction, and xtask
   has no quirk lint. So this is spec conformance work, not a CI break. Size: small to medium
   (mechanical, several files).
4. **S-0106 claim 4, F2, C3, F3: corrected tilt angles.** In `crates/render/src/tilt.rs`:
   - Make `tilt_orientation` build *q*ₛ by C3's construction rule from α_s (F2's table) and the
     corrected τ (arctan(1/√6) for shapes 1-4, π/6 for 5-16). Use double precision rounded per
     S-0105 F0, half-angle sin/cos from `voxel::table_sin_cos`, and the axis normalised with the
     table square root.
   - Keep the current numeric rows as an oracle-only function, e.g. `tilt_orientation_original`.
   - Re-label the existing degree tests as oracle tests on the original rows, and add
     corrected-angle tests derived from the formula.
   - When settled *R*ₛ (C4) is implemented later, it must be F2 with the corrected angles.

   Nothing is drawn tilted yet, so there is no visible change. Size: medium.
5. **S-0206 Edge cases, `quirks.turret_spin_quantised`.** Implement the off branch in
   `turret::tick` (`turret.rs:62-65`): target = facing + 256 × min(ω, 8) with no rounding,
   and no spin at ω ≤ 0. Add a test. The field exists (`quirks.rs:103`) but nothing reads it.
   Size: small. The gap predates the round; the round's hunk only clarifies ω = 0.

Seams worth noting for later milestones, with no work now:
- cloak and disguise (S-0516) under S-0107 13, S-0109 20, S-0111 3/18, S-0207 T4/T5,
  S-0209 7/9, S-0210 11, S-0214 Terms/12;
- hover (S-0503) under S-0106 5, S-0206 13, S-0207 Terms/19 d, S-0212 22;
- bridge links in play (S-0511) under S-0212 3 and S-0214 I3/IC2;
- transports (S-0507) under S-0214 Terms/VD and S-0215 29;
- the automatic crush choice of S-0210 claim 11, which is not implemented at all
  (`player_auto_crush` is unused); built in M8 A3 intake 52a as the crush response to a hit
  (S-0215 claim 39).

Optional: a synthetic test for S-0010 claim 10's R4 overlap fixture. The code already gives
*G* and *L*+15.

## Requests answered

- **Q-0600** (`spec/requests/Q-0600-wake-current-speed.md`): `status: open` became
  `answered`, `answered-by: [S-0504, S-0207]`. For drive vehicles, S-0207 claim 23 rev 20 answers
  the questions as follows:
  1. *s* > 0 evaluated with the current ρ, not ρ itself.
  2. No wake when ρ > 0 but *s* = 0, unless the body is turning.
  3. Ship rate: S-0504 claim 2 (outside this sort).
  4. After the movement update, at the post-move position, inside the vehicle's locomotor turn,
     after movement's draws.
  5. The active track's target only.

  S-0207 front matter now lists Q-0600. The interim rule in `docs/development.md` (the
  `drive::driving_wake` row, line 256) can be retired once item 1 lands. Its timing half is
  already correct; its rate half ("latest drive tick earned *s*", `DriveState::step`) is the part
  the answer rules out ("not a remembered rate from the last moving tick").
- No other request in `spec/requests` changed `answered-by` in the round for these specs. Q-0610
  to Q-0616 are new, open, S-0506 requests. No other interim row in `docs/development.md` ties
  to these revisions.
