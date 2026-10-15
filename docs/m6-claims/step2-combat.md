# M6 step 2: combat, economy and stream spec revisions of the 03:27 round

Scope: `git diff 8c9ed36 main` for S-0301, S-0302, S-0303, S-0308, S-0309, S-0311, S-0312,
S-0313 (spec/combat), S-0202 (spec/simulation), S-0011 (spec/triggers), S-0405 (spec/economy),
S-0410 (spec/ui) and S-0417 (spec/structures). The round is commits 6c5ec1b to 4b0377e
(2026-10-15 03:27 to 03:33).

Buckets as in step 1:
- **(a)** changes or clarifies behaviour of something the engine already implements.
  "(a, already done)": the code already matches. "(a, seam)": the target code does not exist yet.
  Plain "(a)": code must change.
- **(b)** pointer into S-0501, S-0504, S-0506 or S-0614.
- **(c)** editorial: revision/date, backing, confidence, scenario text, rewording.

Every spec moved its `revision:` up by one; S-0313, S-0011, S-0405 and S-0410 also moved
`approved-on:` to 2026-10-15 (c, not repeated below). No spec in this list changed its
front-matter `requests:` line.

Most of this round points at four specs added in the same round and not yet implemented:
S-0516 (cloak, stealth, disguise), S-0511 (bridges), S-0507 (transports) and S-0503 (hover).
Those hunks are "(a, seam)" when the claim already exists in our engine but its new content
needs the missing feature.

---

## S-0301 Firing cycle (r7 -> r8)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 9, *impossible* row | a, seam | *impossible* now includes a hidden target that fails the new test (S-0516 claim 24). | No cloaking: `combat/check.rs:321` ("no object cloaks yet"). |
| 9, new *Hidden-target refusal* paragraph | a, seam | A target hidden from the firer's house gives *impossible* when the firer's mean weapon damage (S-0308 claim 15, ambient part included, truncated) is above 0 or the target is not allied. Hidden = fully cloaked without the house's sensor coverage; a building also at fade levels 11 to 15 without coverage (S-0516 claim 18). Applies to forced attacks on own objects too. | Hook exists: `combat/weapon.rs:338` `mean_damage` (already used at `combat/acquire.rs:176`). The test goes in rank 4 of `combat/check.rs` once S-0516 cloak and sensor state exists. |
| 9 rank table, rank 4 | a, seam | Rank 4 lists the hidden-target *impossible*. | As above. |
| 9 backing | c | Adds F15 part f and S-0516 CS9 part c. | — |
| 9 confidence | c | High for the hidden-target refusal using mean weapon damage. | — |
| Scenario F15 part f (new) | c | Hidden-target test procedure. | Test oracle for the seam. |

New quirk switches: none.

## S-0302 Damage resolution (r20 -> r21)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 1 table, elimination row | a, already done | The row now cites S-0414 claim 5; the values (ignores defences, no-escape yes, no attacker, no house) are unchanged. | `econ/victory.rs:664-672` (`ignore_defences: true, no_escape: true, attacker: None, house: None`). |
| 1 table, new bridge-destruction row | a, seam | Ground-layer hits, landing hits after a deck drop and low-strip kill checks (S-0511 claims 9, 10, 15): ignore defences, no-escape marker, no attacker, no house. | No bridge destruction: `combat/area.rs:436`. |
| 1 backing and confidence | c | Bridge row now backed by S-0511 B3, B5, B6; the `unmeasured` and low-confidence bridge notes are removed. | — |
| 10/11 hand-off note (S-0011 claim 27) | c | "S-0011 claim 27 must replace its airborne wording" becomes "which S-0011 claim 27 also states". No behaviour change. | — |
| 12 (disguise owner) | a, seam | Disguise loss and re-disguise eligibility now owned by S-0516 claims 33-34. | No disguise in the engine (`targeting.rs:14`). |
| 12 backing and confidence | c | The disguise-disruption countdown (2*m*′ ticks) is now `measurable` (D23 part h, S-0516 CS14 part c); confidence medium. | — |
| 16 (bridge repair hut at expiry) | a, seam (plus a comment fix) | The hut now **demolishes** its bridge (S-0511 claim 16) instead of repairing it. It still clears the countdown and planter, with no expiry hit or combat death. | `combat/damage.rs:742-751` already clears `delay_kill_until` and returns with no hit. The comment at line 747 still says "its bridge-repair action is the structures specs' seam"; it should say bridge demolition (S-0511 claim 16). The demolition itself waits for S-0511. |
| Scenario D16 part g, final variant | c | Expected outcome is bridge demolition. | — |
| Scenario D23 setup and new part h | c | Spy with the permanent-disguise flag cleared; countdown test. | — |

New quirk switches: none.

## S-0303 Area damage (r9 -> r10)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 18 Bridges (rewritten) | a, seam | The gate is now the bridge-destruction **game option**: in skirmish the setup option replaces the map's `DestroyableBridges` (S-0511 claim 1). Test order: high bridge-role, high wooden, low wooden, low bridge. A high test accepts the family's section on a high-bridge cell or its pier in state 0 to 3; a low test accepts a low piece not yet destroyed. The height window ((*h* − 2) × 104 + 416, (*h* + 1) × 104 + 416] applies only to a cell marked high bridge (piers without the mark have none). One draw R(1, strength) per eligible test, attempt when *r* < *D*. The ion cannon skips the draws; it retries up to 4 attempts for a **high test only**, and a low test makes one attempt. Effects are S-0511 claims 1 to 15. | Not implemented: `combat/area.rs:436`. Its comment ("need the map's destroyable-bridges setting") is now wrong and should name the game option and S-0511. |
| 18 backing and confidence | c | A12 parts and S-0511 B1, B7, B11; confidence medium. | — |
| F6 allocation paragraph | c | Reworded intro to the transition table. No behaviour change. | — |
| Constants, ion-cannon repeats | a, seam | Renamed "High-test repeats": 3 further attempts, low tests never repeat. | As claim 18. |
| Import mapping, new `BridgeDestruction` row | a, seam | `[MultiplayerDialogSettings] BridgeDestruction` is the default of the game option (boolean, absent yes). | Not imported (no `BridgeDestruction` in `crates/`). Add it to the setup/rules import with S-0511. |
| Import mapping, `DestroyableBridges` row | a, seam | Ignored in skirmish. | Only listed in `convert/src/coverage.rs:231`; nothing reads it. When S-0511 is implemented, do not read it in skirmish. |
| Scenario A12 variants b to e | c | New expected outcomes; skirmish gate and membership variants. | — |
| Open questions | c | Bridge-piece rules now S-0511; only the test order stays open. | — |

New quirk switches: none.

## S-0308 Weapon choice (r4 -> r5)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Mode index* | c | Also used when a passenger is put back after a failed exit. | — |
| 6, new *Failed-exit exception* | a, seam | When a passenger fails to exit and stays at position 1 (S-0507 claim 10), the gunner hand-over runs again for it even with others aboard. The weapon number becomes its mode index, subject to claim 5's clamping and charge-turret exception (`quirks.unload_failed_exit_gunner_handover`). | No transports. `combat/weapon.rs:92` `set_weapon_number` (clamping, charge-turret) exists and has no callers. Quirk field exists: `rules/src/quirks.rs:1271` (owner S-0507, "not implemented yet"). |
| 6 backing and confidence | c | Part i and S-0507 T10 part e; high for the failed-exit weapon number, low for other hand-over effects. | — |
| Table (gunner weapon number) | a, seam | New row for the failed exit; all changes subject to claim 5; with the quirk off a failed exit changes nothing. | As claim 6. |
| Edge cases, new quirk entry | a, seam | Documents `quirks.unload_failed_exit_gunner_handover` (default on, shared with S-0507). | Field exists (`quirks.rs:1271`); its doc comment cites only S-0507 and should also cite S-0308 claim 6 when it is implemented. |
| Scenario W4 part i (new) | c | Failed-exit test. | — |

New quirk switches: `quirks.unload_failed_exit_gunner_handover` is new to S-0308. It is already declared (S-0507).

## S-0309 Target acquisition (r12 -> r13)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Sensed cell* | c (a, seam) | Now defined by S-0516 (Terms, claims 21-23). | No sensors. |
| Terms, *Fully cloaked* | c (a, seam) | The *cloaked* state of S-0516 (Terms, claim 1). | Table C's cloak row has no producer (no cloaking). |
| Terms, *Appears disguised to a house* (new) | a, seam | Game-state test of S-0516 claim 35. Disguise-detector coverage (S-0516 claim 40) is separate from sensor coverage. | No disguise. |
| Scenario part g | c | Sensor pointer to S-0516 claims 21-23. | — |

New quirk switches: none.

## S-0311 Veterancy (r3 -> r4)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Ability table, *cloak* row | a, seam | The object can start and retain cloaking under the ability rules of S-0516 claims 2, 5, 9. | `combat/data.rs:335` `Ability::Cloak` is imported; no consumer. |
| Ability table, *sensors* row | a, seam | The ability forces an adjacent enemy fully cloaked mobile object to uncloak (S-0516 claims 13, 22). **It grants no sensor coverage.** | `combat/data.rs:341` `Ability::Sensors`; no consumer. When S-0516 is done, the sensors ability must not add coverage. |

New quirk switches: none.

## S-0312 Death and destruction (r18 -> r19)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Bomb mark* | a, seam | The mark now causes landing damage when the fall ends (S-0511 claim 10). Stock sources: a hover vehicle's switch-off fall over water (S-0503 claim 11) and a bridge-deck drop (S-0511 claim 9). | No bomb mark or fall (`grep -i bomb` finds only planted-bomb seams). |
| 11 (sources, thresholds) | a, seam | Lists those stock sources. The hover's strict 104-lepton fall start is separate from the inclusive 10-lepton death-effects threshold, and the mark alone does not suppress ordinary death effects at greater heights. | Not modelled in `combat/death.rs`. |
| 11 backing and confidence | c | Now `measurable` (D8 marked-water variant, S-0503 H7, S-0511 B3); the 10-lepton threshold stays `unmeasured`. | — |
| Scenario D8 text and new marked-water variant | c | — | — |
| Open questions | c | Bomb-mark question removed. | — |

New quirk switches: none.

## S-0313 Firing positions (r1 -> r2)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Overview | c | Cites S-0306 claim 1; skirmish-only scope. | — |
| Terms, *Pixel shift* | c | "imported" instead of "in the art data". | — |
| Terms, *voxel turret/barrel*, *Turret anchor's pixel shift* | **a** | These are now **rules** data (building type), not art. The same pair places the drawn turret (S-0105 claim 9). | See Import mapping below. |
| 1 backing | c | P1 and P2; locomotor translations `unmeasured`. | — |
| 3 backing | c | Building no-target row and aircraft turret/body distinction `unmeasured`. | — |
| 5 (passenger order, transport frame) | a, seam | Position 1 is the most recently boarded passenger; a failed exit goes back to position 1 (S-0507 claims 3, 10). The passenger's own facing, turret displacement and burst counter do not change the muzzle; the transport's burst parity decides the lateral sign. | No transports. `combat/position.rs:107` `fire_point` has no passenger path. |
| 5 confidence | c | High for the passenger order. | — |
| 6 (garrison reset wording) | c (a, seam) | "the building is emptied by occupants leaving or being killed". Same rule. | No garrisons in the engine. |
| 8 backing | c | The voxel-barrel row is an explicit gap. | `combat/position.rs:221` already uses the claim-1 point for a voxel barrel and comments the gap. |
| 9 (spray wording) | a, already done | "redirects its shot target to the cell". | `combat/fire.rs:131-147` sets `shot_target` to the shifted cell. |
| F4 integer note | a, already done | Integer shifts and indices; trunc(8 / *B*) for positive *B*; sum mod 8. | `combat/fire.rs:139` `(spray_shift + 8 / b).rem_euclid(8)`. |
| State table, garrison reset row | c | Same wording change as claim 6. | — |
| Edge cases, burst-counter entry removed | c | The "no switch" entry for the transport's burst counter is folded into claim 5. | — |
| Edge cases, more than ten muzzle points | c | Backing note only. | Importer warning already exists: `convert/src/sim_combat.rs` `building_combat`, `MaxNumberOccupants`. |
| Import mapping, `TurretAnimX`, `TurretAnimY`, `TurretAnimIsVoxel`, `BarrelAnimIsVoxel` | **a** | Source moves from (art, building image) to **(rules, building type)**. | Importer still reads art: `convert/src/sim_combat.rs:1993-2011` (`art_bool` for the two voxel flags, `art_int` for `TurretAnimX/Y`), and the keyed-coverage lists at `sim_combat.rs:1336-1369` file them under `"art"`. Doc comments `rules/src/sim.rs:1833-1847` say "Art". The drawing side already reads rules (`app/src/mapview/build.rs:2658-2662`; `convert/src/art.rs:742-743`). Test `convert/src/sim_combat_tests.rs:755-797` puts them in art and must move. **Conflict:** S-0310's Import mapping (`spec/combat/S-0310-turrets-facing-for-fire.md:401-402`) still lists `TurretAnimIsVoxel`, `TurretAnimX`, `TurretAnimY` as art, and `combat/check.rs:74` uses `turret_anim` for the S-0310 claim 4 aiming point. |
| Measurement scenarios (common procedure, P1 to P7) | c | Rewritten procedures; exact integer falsifiers in place of the 1-lepton tolerance; new parts. | Test oracles. |
| Open questions | c | Passenger order removed (S-0507 owns it). | — |

New quirk switches: none.

## S-0202 Random streams (r20 -> r21)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 10 (scenario stream) | a, seam | Exception: low-bridge repair's piece choices use the map-generator stream (claim 13). | No bridge repair. |
| 13 (map-generator stream in play) | a, seam | Low-bridge repair (S-0511 claim 20): per strip that needs a random intact piece, unsigned raw *g*, *r* = trunc(*g* × 4 × *c*) with *c* the double nearest 1 / (2³² − 1), both multiplications in double; reject *r* > 3 and draw again. Strips are taken in increasing bridge-axis coordinate; strips with no random choice take nothing. This is not claim 4's ranged draw. | The world has one scenario `Rng` (`rng.rs:28`); there is no map-generator stream in the sim. Needs a second seeded stream in state (serialised, hashed) plus soft-double arithmetic (`glasnost_num` F64, as `rng.rs:139` `unit` does), not host floats. |
| 13 table and backing | c | R9 c limited to play without low-bridge repair; new row for repair choices. | — |
| Scenario R9 c note and new part f | c | — | — |

New quirk switches: none.

## S-0011 Map triggers (r16 -> r17)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Hand-off note | c | Check claims 20 and 27 with S-0511 claims 1, 7, 8 at sign-off. | — |
| 20 (bridge-destroyed event bands, rewritten) | a, seam | Event judging unchanged: satisfied in every raise with its code. New: bands from S-0511 claims 7, 8 (not only collapsed cells). High band: first cell after the start piece to the far piece; low band: first to last low piece of the whole run, destroyed ones included. 4 cells across (own, +1, −1, −2 on the cross-coordinate), raised in increasing along-bridge order then that cross order. Last centre-row cell and its cross-cells skipped; one-cell band raises nothing (`quirks.bridge_event_band_short`). **Each raise has no causing object and no cell.** One raise per qualifying tagged cell. A destroyed low strip gives one band pass; a high span with *n* isolated losses gives *n* + 1 passes; no missing section, none. | Judging already done: `triggers.rs:694-695`. The raise itself is a seam (`triggers.rs:16`). Note for the implementer: `Raise` (`triggers.rs:184-192`) uses `carrier: Some(Carrier::Cell(..))` both as the raising tag's carrier and as "the raise carries a cell". Claim 20 raises come from tagged cells yet carry no cell, so claim 27's cell test needs a separate field (or `carrier: None` for these raises). |
| 27 (destroy attached objects: cell attempts) | a, seam | After the object hits, if the raise carries a cell, up to 4 bridge destroy attempts at that cell, stopping at the first success, **ignoring the bridge-destruction setting** (S-0511 claims 1, 2). A raise with no cell makes none; claim 20 raises carry none. | Action 32 is not implemented at all: `triggers.rs:933-935` (note only). |
| 27 backing and confidence | c | T9 E and S-0511 B10; low confidence for the cell effect. | — |
| Edge cases, new `quirks.bridge_event_band_short` entry | a, seam | Default on: skip the last band cell and its cross-cells. Off: include them. | Field exists: `rules/src/quirks.rs:1287` (owner S-0511 claim 8, "not implemented yet"). |
| Scenarios T9 variant E (new) and T13 (rewritten) | c | — | — |
| Open questions | c | "Destroy attached objects on a cell" removed (answered by claim 27). | — |

New quirk switches: `quirks.bridge_event_band_short` is new to S-0011. It is already declared (S-0511).

## S-0405 Power (r8 -> r9)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 13 (disabled vehicles) | a, seam | "Does nothing until enabled" becomes "inactive until enabled". A disabled hover vehicle can still spin, settle and keep residual travel, and over water fall and be destroyed. Lift returns on re-enabling (S-0503 claims 8, 10, 11). | `econ/power.rs:845` `set_disabled` sets `unit_disabled`. No hover locomotor exists, so nothing to change now. When S-0503 lands, its hover update must keep running for a disabled vehicle. |
| 13 backing | c | Hover effects backed by S-0503 H7. | — |
| Scenario W8 expected (a) | c | "stop" becomes "are disabled", hover exception. | — |

New quirk switches: none.

## S-0410 Radar and minimap (r15 -> r16)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 7 (not owned: cloaked objects) | a, seam | A fully cloaked object of another house, **allies included**, is listed only when *H*'s **own** sensor coverage covers its cell (S-0516 claims 21, 27). Allied coverage does not count. This is necessary, not sufficient. Owned objects need no sensors. | `econ/minimap.rs:509` notes cloaking as a seam. |
| 7 backing and confidence | c | R9 part D; medium for the own-sensor requirement. | — |
| 8 (disguise colour) | a, seam | For a disguised unit, S-0516 claim 38 picks the colour house: the owner if it allies *H* (or is *H*), else the disguise house. This is separate from the game-state and image disguise tests. No disguise house: open. | No disguise; `econ/minimap.rs` uses owner colour only. |
| 8 backing and confidence | c | S-0516 CS16 c, CS17; disguise-house selection medium. | — |
| Scenario R9 C note and new part D | c | — | — |
| Open questions | c | Cloak question narrowed to underground and radar-invisible exceptions; disguise question narrowed to "no disguise house". | — |

New quirk switches: none.

## S-0417 Factory unloading (r10 -> r11)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 5 (hover exit fraction) | a, seam | ρ = ½ after the exit instruction is unchanged. For a hover vehicle, S-0503 claim 4 owns later changes: a stalled start (provisional step 0, heading exactly at the heading point) resets ρ to 1. That resets the fraction only; hover speed does not jump to full. | `econ/release.rs:1389-1392` sets `drive.rho = HALF` (already done for the ρ = ½ part). No hover locomotor. Note: the doc comment of `quirks.hover_stalled_start_reset` (`rules/src/quirks.rs:1280`) says it "sets the throttle and fraction to 1 at once". Check that against S-0503 claim 4 when hover is implemented: S-0417 says the reset does not set hover speed to 1. |
| 5 backing and confidence | c | U6 and S-0503 H8; low for how long the hover fraction lasts. | — |
| Scenario U6 hover recording | c | — | — |

New quirk switches: none.

---

## (a) items that need work

1. **S-0313 Import mapping and Terms (claim 8): read `TurretAnimX`, `TurretAnimY`,
   `TurretAnimIsVoxel` and `BarrelAnimIsVoxel` from the rules building type, not the art image.**
   Where: `crates/convert/src/sim_combat.rs:1993-2011` (`building_combat`: replace `art_bool` /
   `art_int` for these four keys with rules-section reads; keep `PrimaryFireDualOffset`,
   `IsAnimDelayedFire` and `PrimaryFirePixelOffset` on art). Also the coverage lists at
   `sim_combat.rs:1336-1369`, the doc comments at `crates/deterministic/rules/src/sim.rs:1833-1847`,
   and the test `crates/convert/src/sim_combat_tests.rs:755-797`. The drawing side already reads
   rules. Size: small (about 20 lines plus a test). **Blocker:** S-0310's Import mapping still lists
   `TurretAnimIsVoxel`, `TurretAnimX` and `TurretAnimY` as art, and the same imported field drives
   the S-0310 claim 4 aiming point (`combat/check.rs:74`). The two specs now contradict each other.
   Under CLAUDE.md, write a spec request asking which source S-0310 claim 4 uses, and make the
   change once that is answered.
2. **Comment fixes (no behaviour change), small:**
   - `crates/deterministic/sim/src/combat/damage.rs:747`: the hut's expiry action is bridge
     demolition (S-0302 claim 16, S-0511 claim 16), not repair.
   - `crates/deterministic/sim/src/combat/area.rs:436`: claim 18's gate is the bridge-destruction
     game option (S-0511 claim 1), not the map's destroyable-bridges setting.

Everything else in (a) is "already done" (S-0302 claim 1 elimination row, S-0313 claim 9
wording and F4, the S-0417 ρ = ½ part, the S-0011 claim 20 judging) or "seam". The seams are
waiting on S-0516 (hidden-target refusal, sensors and cloak ability, radar cloak and disguise
rules, disguise loss), S-0511 (bridge attempts, bridge hit metadata, band raises, the
destroy-attached cell attempts, low-bridge repair on the map-generator stream, the bomb mark from
deck drops), S-0507 (failed-exit gunner hand-over, passenger muzzles) and S-0503 (hover while
disabled, hover exit fraction, hover bomb mark). Ready hooks: `combat::weapon::mean_damage`
(`combat/weapon.rs:338`) and `combat::weapon::set_weapon_number` (`combat/weapon.rs:92`).
Declared quirks: `unload_failed_exit_gunner_handover` (`quirks.rs:1271`) and
`bridge_event_band_short` (`quirks.rs:1287`). The trigger `Raise` struct (`triggers.rs:184`)
will need a "carries a cell" field separate from its carrier for S-0011 claims 20 and 27.

## Requests answered

- **Q-0600** (which speed makes a vehicle or ship "moving now" for wakes): `status: answered`,
  `answered-by: [S-0504, S-0207]` (commit 4b0377e). Neither spec is on this list. The interim
  rule in `docs/development.md:256` (`drive::driving_wake`) is tied to it and is a candidate for
  retirement. Whoever owns S-0207 should check it first: S-0207 claim 23 now defines *s* as
  trunc(*s*₂ × ρ) at the wake check, with the current fraction, not a remembered rate. The code
  (`drive.rs:1433-1438`) reads `DriveState::step` from the latest drive tick, which matches only
  if `step` is always this tick's post-movement rate.
- No request is answered by the 13 specs on this list: none changed its `requests:` front matter.
  Q-0610 to Q-0616 were added during the round by the S-0506 lane and are still `open`
  (`answered-by: []`), so their interim rules (`docs/development.md:350-356`) stay.
