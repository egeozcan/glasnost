# M6 step 1: combat and random-stream spec revisions since 8d4181a

Scope: `git diff 8d4181a HEAD` for S-0301, S-0302, S-0303, S-0306, S-0307, S-0308, S-0309,
S-0310, S-0311, S-0312, S-0314, S-0315 (spec/combat) and S-0202 (spec/simulation).

Buckets:
- **(a)** changes or clarifies behaviour of something the engine already implements.
  "(a, already done)" means the code already matches; "(a, seam)" means the spec changed but the
  target code does not exist yet, so there is no M6 work.
- **(b)** pointer into S-0501 (aircraft), S-0504 (naval), S-0506 (missiles/spawn controller) or
  S-0614 (parasites); it takes effect only when that feature exists.

M6a: the (b) pointers into S-0501 are now implemented in `glasnost_sim::air` (lane A2); see
[S-0501.md](S-0501.md) for their status. The evidence column below is unchanged.
- **(c)** editorial: revision/date, backing, confidence, scenario text, rewording.

Requests: no front-matter `requests:` line changed in any of the 13 specs, so these revisions
answer no requests. The only Q-ID in changed lines is Q-0060, in a reworded S-0301 Open-questions
line (already answered by claim 10). Every spec moved its `revision:` up by one and
`approved-on:` to 2026-10-15 (c, not repeated below).

---

## S-0301 Firing cycle (r5 -> r6)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Rearm countdown* | b | Countdown also does not block "the unchecked shots of claim 9", i.e. an aircraft's same-tick full burst (S-0501 claim 12). | No aircraft attack behaviour; `combat/check.rs:290-296` (countdown) unchanged. |
| Terms, *Spawner weapon* | b | Spawner shots launch nothing; they hand the target to the spawn controller as a pending target (S-0506 claim 3). Spawned aircraft are S-0506 slot objects; *alive* = slot not empty; *launching* = S-0506 launching count > 0 (claim 10). | No spawner in engine (`combat/check.rs:284` notes none exist). |
| Terms, *Firing check* | b | The firing check is skipped for the claim 9 unchecked shots (aircraft full burst). | — |
| 1 | b | Countdown does not hold back the later shots of an aircraft's same-tick full burst (claim 4). | — |
| 4 | b | Aircraft full burst: all *B* shots in one tick, no firing check between, each advances the burst counter and sets the countdown; in-burst delays do not separate them (S-0501 claim 12). | — |
| 9 (intro, *illegal* row) | b | *Illegal* cases now name parasite tests (S-0614 claim 5), a held organic firer (S-0614 claim 8), sinking firer or target (S-0504 claim 5). | `combat/check.rs:277` (rank 4 target tests) would gain these later. |
| 9 rank 2 | b | *busy* cites S-0506 claim 10. | `combat/check.rs:39` (`Busy` exists, never produced). |
| 9 rank 4 | b | Adds *illegal* for a sinking firer (S-0504 claim 5); spawner rearming case cites S-0506 claim 10. | `combat/check.rs:284`. |
| 9 rank 9 | b | Adds parasite tests (S-0614 claim 5), sinking target (S-0504 claim 5), held organic firer (S-0614 claim 8) as *illegal*. | `combat/check.rs:310-320`. |
| 9 exception paragraph | b | Aircraft full burst joins the SAM secondary shot as a check-free shot (S-0501 claim 12). | — |
| 9 backing | c | Adds S-0506/S-0614/S-0504/S-0501 scenarios; sinking precedence `unmeasured`. | — |
| 10 | b | Aircraft spend one round per attack run in which they fired, not per shot (S-0501 claim 15); pad servicing is S-0501 claims 26 to 28. Starting count still owned here. | `combat/fire.rs` ammunition use: no aircraft path yet. |
| State and ordering | b | Shot-interval rule adds the aircraft full-burst exception. | — |
| Open questions | c | Aircraft ammunition now points at S-0501 claims 15, 26 to 28 (Q-0060 question 1 already answered). | — |

New quirk switches: none.

## S-0302 Damage resolution (r19 -> r20)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 1 (hit metadata table) | b | New row: a parasite's delivery (S-0614 claims 6 and 10): does not ignore defences (except a drone's delivery to an infantry host, which does), no-escape marker yes, attacker = the parasite, no attacking house. | `combat/damage.rs` hit metadata; no parasite producer yet. |
| 1 backing and confidence | c | Adds parasite-delivery backing (S-0614 P1, P6, scenario D31). | — |
| Scenario D31 (new) | c | Parasite-delivery metadata scenario. | — |

New quirk switches: none.

## S-0303 Area damage (r8 -> r9)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 2 (parasite flag) | **a** (suppression), b (latch) | A projectile whose warhead has the parasite flag makes **no area hits**, whether or not the latch succeeds. Its explosion animation (F5) and debris (F6) are still created. The rule applies only to projectile detonations, not to other producers using that warhead. Until S-0614 exists every such detonation is a failed latch, so the suppression takes effect as soon as the flag is imported. The latch itself is S-0614 claim 1 (b). | Sim: `combat/projectile.rs` detonation (around `detonate_weapon`, line 1617, and the area call before the animation/debris block at 1505-1540): skip `area::detonate` when the warhead has the parasite flag, keep `explosion_anim` and debris. Data: add `parasite: bool` to the warhead record in `combat/data.rs`. Importer: new `Parasite` warhead key in `convert/src/sim_combat.rs` (warhead field table near line 290). |
| Import mapping | a | New row: rules warhead `Parasite` -> parasite flag, boolean, absent no. | `convert/src/sim_combat.rs` warhead table (around 285-300). |
| Terms, *Tracked flying object* | b | Rocket missile is tracked from the end of its tilt/raise phase until it detonates (S-0506 claims 15, 19); it stops being tracked before its own detonation's area damage. Aircraft and jumpjet boundaries unchanged (movement specs). | No missiles; `combat/area.rs` flying-object search unchanged for aircraft/jumpjets. |
| 6 backing | c | Rocket registration backed by S-0506 R10; removal-before-own-damage `unmeasured`. | — |
| 22 backing | c | Adds A19 b/c; F6 exceptional raw value `unmeasured`. | — |
| F6 non-positive *N* | a, already done | A non-positive total creates no pieces and takes no allocation or type-choice draws. | `combat/death.rs:85` (`n <= 0` returns); metallic branch `combat/projectile.rs:1530` loops `n.max(0)`. |
| F6 allocation (signed maxima, wrap) | **a** | Rewritten state machine: *a* = wrap(\|raw\|), *b* = wrap(*u* + 1), *g* = min(*a* rem *b*, ℓ) with signed truncated remainder (sign of *a*), ℓ' = wrap(ℓ − *g*) (so a negative *g* **raises** ℓ), stop when ℓ' ≤ 0. *u* = 2147483647 gives divisor −2147483648 (not 0). *u* = −4 assigns 0 to 2 pieces; −2 assigns 0. Each assignment's pieces are created before the next allocation raw value is drawn (creation draws interleave). | `combat/death.rs:83-122` `deal_debris`. Divergences: (1) `u + 1` overflows at `i32::MAX` (panic in debug; needs `wrapping_add`); (2) `left -= g.max(0)` must become `left = left.wrapping_sub(g)`; (3) the `%` needs `num::int::rem` (`crates/deterministic/num/src/int.rs:13`) to cover the zero divisor and `MIN rem -1` (Rust `%` panics there). Interleaving already matches: `effects::anim` runs inside the loop before the next `w.rng.raw()`. |
| F6 + `quirks.random_magnitude_overflow` off | **a** | With the quirk off, *a* is the exact magnitude 2147483648 in a wider non-negative domain, so the remainder is non-negative; *b* and ℓ keep i32 wrap. With it on (default), *a* stays −2147483648 and signed rules apply. | `combat/death.rs:107-111`: both branches give `i32::MIN` today (`wrapping_abs(MIN) == MIN`). The off branch needs i64: `(raw as i64).abs() % (b as i64)`, with *b* = 0 handled as S-0205 claim 3. |
| Edge cases: zero debris-allocation divisor (new departure) | **a** | Retained maximum −1: the original stops; our engine takes the raw value, remainder = *a* (S-0205 claim 3), so *g* = min(*a*, ℓ). No extra draw. *u* = −2 with raw −2147483648: remainder 0. Importer warns about a retained maximum of −1. | Sim: `combat/death.rs:112-116` currently gives `g = 0` for `u + 1 == 0`, which is wrong now. Importer: `convert/src/sim_combat.rs:1310-1348` `debris_warnings` needs a new warning for a retained −1. |
| Edge cases: debris that can never be placed | **a** | The guard now applies only when **every** paired maximum is 0 or −2 (all remainders 0). −1 no longer belongs (it places pieces). Still stop after one full pass, keeping that pass's raw values; missing maxima count as 0. | Sim: `combat/death.rs:92` `never` uses `(-2..=0)`; change it to `m == 0 \|\| m == -2`. Importer: `convert/src/sim_combat.rs:1342` checks only `m == 0`; extend it to −2 and update the message. |
| Edge cases closing paragraph | c | Undefined results (scan radius, empty metallic list, negative entry) `unmeasured`; departures are design rules. | — |
| Open questions | c | Rocket-missile boundaries now S-0506 claims 15, 19. | — |
| Scenario A3 b (heights 208/209) | c (a, already done) | Scenario adds the centre-cell height boundary: 208 -> *d* = 0, 209 -> *d* = 1. | `combat/area.rs:352-353` (`p.z - z > 208`, `dist - 208`). |
| Scenario A4 b (deck layer 208/209) | c (a, already done) | Deck layer only above 208 and only with a high bridge in the centre cell. | `combat/area.rs:279-280`. |
| Scenario A9 d (nullify 84/85/86) | c (a, already done) | Nullified at 84, not at 85 or 86 (85 exclusive). | Spec Constants "85 (exclusive)"; `combat/area.rs` claim 15 test. |
| Scenarios A14 o, A15 b, A19 b/c | c | Parasite visuals; allocation/creation ordering; signed maxima; zero divisor. | Use as test oracles for the F6 work. |

S-0312's death debris (S-0312 claim 4, F1) calls the same `deal_debris` (`combat/death.rs:62`), so
the F6 fix applies there too.

New quirk switches: none. (`quirks.random_magnitude_overflow` already exists, owned by S-0202
claim 6; F6 now gives its off branch a specific meaning, listed above.)

## S-0306 Projectiles (r7 -> r8)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Launch preamble | b | General launch rules have aircraft velocity exceptions (S-0501 claim 20). | — |
| 7 | b | Aircraft shot: with ρ = 0 the projectile takes the aircraft's body heading, pitch and ground speed; with ρ = 1 it is aimed in 3D at the target at weapon speed. This replaces the general velocity, including the claim 9 rescale. Other turn rates unchanged (S-0501 claim 20). | `combat/projectile.rs` launch; no aircraft firer. |
| 9 | b | The steering rescale to length 1 (F10) comes before any aircraft velocity replacement. | — |
| 21 | **a** (no area hit), b (latch) | A latching (parasite-warhead) weapon's detonation makes no area-damage hit, whether or not it latches. It tries to latch onto the target object at any distance; a cell target or a vanished target gives a failed latch. Later visuals unchanged (S-0614 claim 1). Same sim change as S-0303 claim 2. | `combat/projectile.rs` detonation (see S-0303 claim 2). |
| 21 backing and confidence | c | Adds S-0614 P1/P3. | — |
| P3 scenario falsifier | c (**spec defect?**) | Falsifier changed from "two draws in (e), or four in (d)" to "four draws in (e), or two in (d)". The unchanged Expected text still says (d) two draws and (e) four, so the new falsifier contradicts it. The old text was the consistent one. No code change. Consider a spec request. | `crates/deterministic/sim/tests/claims_s0306.rs` follows the claims (claim 24's order), not this falsifier. |

New quirk switches: none.

## S-0307 Range (r11 -> r12)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Airborne, floor-bound* | b | Ordinary objects keep the 208-lepton test. Aircraft use S-0501 claim 18's in-the-air answer, as qualified in claim 26. | `combat/weapon.rs:24-30` `airborne_by_height`. |
| 26 (new) | b | For aircraft, *airborne* = S-0501 claim 18's in-the-air answer. Ordinary aircraft keep the 208-lepton test. For the game-wide V3-rocket and dreadnought-missile types, the locomotor's own in-the-air answer replaces the height test, and floor-bound is its opposite. Applies to claims 3, 6, 7 and 8 b. | Ordinary aircraft already use height: `combat/weapon.rs:24`, used at `combat/range.rs:23, 71, 166`. The rocket exceptions needed S-0506 rocket locomotion (b). Built by sim issues S7 (#253): `weapon::range_airborne` and `weapon::range_floor_bound` give those two types the phase answer, read by `range` and the projectile and targeting-mode rows of `check`; test `claims_s0506.rs::a_rocket_in_ascent_below_208_is_not_measured_on_the_ground`. |

New quirk switches: none.

## S-0308 Weapon choice (r3 -> r4)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Airborne* | b | For aircraft, use S-0501 claim 18's in-the-air answer, not its *in flight* test. Other objects: movement specs. | `combat/weapon.rs:17` `in_air`. |
| 6 | a, already done (gunner flow is a seam) | A gunner vehicle that is also a charge-turret type ignores the weapon-number change request and stays at weapon number 0. Only non-charge-turret switching gunners take the first passenger's weapon. | `combat/weapon.rs:80-83` (`set_weapon_number` returns early for `charge_turret`). Gunner passenger entry/exit is not implemented (no transports), so nothing calls it yet. |

New quirk switches: none.

## S-0309 Target acquisition (r11 -> r12)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 4 (spawner *impossible*) | b | A spawner that gets *impossible* keeps the target and resets its spawn controller (S-0506 claim 11). Released missiles in flight continue to their fixed aim points (S-0506 claims 11 and 12). | `combat/acquire.rs:942-952`: no spawner case (no spawner types exist). |
| 4 (retention vs other losses) | b | Search retention does not protect against other target-loss rules: before behaviour processing, an aircraft clears even a living target whose in-the-air answer is yes (S-0501 claim 32). | — |
| 18 | b | "X is not the parasite attached to it" now cites S-0614 claim 17. | `combat/acquire.rs:146-171` retaliation; no parasites. |
| Search outcome table | b | Spawner *impossible* row: controller resets (S-0506 claim 11), released missiles fly on (S-0506 claim 12). | — |
| Scenario (part f) | c | Spawner reset and missile-in-flight variants. | — |

New quirk switches: none.

## S-0310 Turrets and facing for fire (r2 -> r3)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Scope, Terms (*healing actor*) | c | Skirmish-only scope sentence; S-0214 term renamed. | — |
| Terms (*Fighter*, *Passenger carrier*; *Flight control* removed) | b | New aircraft terms (S-0501 claims 11 and 1). | — |
| 1 backing | c | — | — |
| 4 (snap wording) | c | "no switch disabling the snap itself". | — |
| 5 | b | Non-fighter aircraft: τ = 2048 on the current turret facing, whatever the projectile. A fighter never gets *not facing* (type flag, not flight state). After the general check passes, an aircraft marked as a passenger carrier that carries none gets *out of ammunition*, ahead of the aim test. No switch (S-0501 claim 19). | `combat/check.rs:126` (aircraft τ = 2048, no fighter exemption). The fighter exemption could be added cheaply once `Fighter` is imported, but it only matters with aircraft attack behaviour. |
| 6 | a, already done | Only the difference between *not facing* and each of *must deploy*, *moving*, *still turning* shows in game state. The order among those three is free: an implementation may pick any applicable one. Empty passenger carrier's *out of ammunition* ranks after the general part and above aircraft aiming (b). | `combat/check.rs:330-357` already uses a fixed order (healer illegal, moving, still turning, not facing), which is allowed. |
| 7 | c | The locomotor's own refusal comes only after the conditions and aiming tests pass. Which states produce it is unspecified (Open questions). | `combat/check.rs:360` returns `CanFire` with no locomotor refusal. Consistent. |
| 7, 8, 9, 11 backing | c | `unmeasured` notes (0.1 boundary, combinations, healer cases). | — |
| F2 table, Constants | b | Split into non-fighter 2048 / fighter no test. | `combat/check.rs:126`. |
| Rank table | b | New aircraft passenger-carrier *out of ammunition* row; non-fighter aim row. | — |
| Rank table (vehicle *illegal* rank 3) | c | Cites S-0301 claim 9 rank 3 instead of "no spec owns it". | — |
| Ordering consequences | a, already done | A non-homing weapon on a non-spinning turret must wait until the tick after the turret stops turning (claim 3), not just until it is inside tolerance. Homing weapons and turret-less vehicles may fire in the tick the facing enters tolerance. | `combat/check.rs:345-346` (`u.turret && !u.turret_spins && !homing && turning(t-1)`). |
| Edge cases (voxel switch, aircraft) | c / b | Voxel-switch wording (snap bound's separate switch is S-0215's). The aircraft edge case is replaced by "empty former passenger carriers" (b, S-0501 claims 19 and 20). | `quirks.voxel_turret_exact_aim` at `combat/check.rs:120` unchanged. |
| Import mapping | b | New `Fighter` (rules, aircraft type) -> fighter flag (S-0501 claim 19). | Not imported. |
| Scenarios T1, T4, T5, T6, T7, T9, T10; Open questions | c | Rewritten procedures; aircraft question answered by S-0501 claim 19. | — |

New quirk switches: none.

## S-0311 Veterancy (r2 -> r3)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Missile spawn* | b | Spawning rules are S-0506 claims 1 to 13. | `combat/veterancy.rs:129` (missile spawns are a seam). |
| 2 | b | A grappling naval parasite also gains experience from a cull (S-0614 claim 16 and F5). | — |
| 3 (ability table, *faster*) | b | Faster applies to ground vehicles and infantry. Ordinary aircraft flight uses type speed without the bonus (S-0501 claim 4). | Vehicle/infantry speed unchanged. |
| Open questions | c | Parasite-experience question removed (now S-0614). | — |

New quirk switches: none.

## S-0312 Death and destruction (r17 -> r18)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Sinking, crashing* | b | These states can start without a killing hit. Owners: S-0504 claims 4-5 (vehicle sinking), S-0501 claim 9 (aircraft crash and sinking), S-0614 claim 11 (cull sinking), S-0506 claim 20 (rocket interception). | `combat/death.rs:276` (sinking not modelled). |
| 6 | b | Only a non-rocket crashing aircraft fires its death weapon at impact. Rocket interception follows S-0506 claim 20. | No crash code (`combat/death.rs` has none). |
| 10 | b | Sinking after it starts is S-0504 claims 4-5. A grappling parasite's cull starts sinking without a killing hit, whatever the weight, with health unchanged and no death effects (S-0614 claim 11). | `combat/death.rs:276`. |
| 16 | b | Aircraft crash motion is S-0501 claim 9 (the fall counter is a downward contribution, not net descent; sinking branch). Rocket missiles are excluded from the impact rules. An intercepted rocket above ground keeps rocket flight and full detonation on its next positive-speed move attempt (map-edge rejection included); at zero speed it waits; on the ground it is removed without detonation (S-0506 claims 19, 20). | No crash code. |
| State table | b | New cull-sinking row; crash rows limited to non-rocket objects. | — |
| `quirks.crash_death_weapon_twice` | b | Now "non-rocket"; the rocket quirk is S-0506's (`quirks.destroyed_rocket_still_detonates`). Existing switch, not new. | `crates/deterministic/rules/src/quirks.rs:733`; no crash code uses it yet. |
| 4 (debris via S-0303 F6) | a (via S-0303) | Unchanged text, but the F6 rewrite above changes death debris allocation. | `combat/death.rs:62` -> `deal_debris`. |
| Backing, scenarios D8 and D11, Open questions | c | Pointers to S-0504 N5/N6, S-0614 P6, S-0501 AC5, S-0506 R13. Sinking question removed. | — |

New quirk switches: none in this spec (pointer to S-0506's `quirks.destroyed_rocket_still_detonates`).

## S-0314 Healing and repair (r9 -> r10)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 4 | b | Aircraft on reload pads use the same step size. Cadence, priority and release are S-0501 claims 26-27. | `combat/heal.rs` step size; no pads. |
| 5 backing | b / c | The same cost formula for aircraft pad steps (S-0501 AC16). | — |
| 6 (depot, new waiting phase) | a, seam | Three phases now. *Waiting*: at each processing (*D* ticks apart), check that the vehicle stands still within the starting distance of its docking point: strictly < 100 leptons for ordinary locomotors, < 200 for ship and hover (S-0504 claim 8). The distance is spatial (height included), floating point, truncated to integer leptons. Success schedules *settling* 3 ticks later. Coming to rest alone takes no step. *Settling* needs the vehicle still and its locomotor settled. New constants: 100, 200, 3. | Depots are not wired (`combat/heal.rs:1-6`; `targeting.rs:1173` refuses depot entry). No M6 work. When built, the float distance needs a deterministic integer/fixed-point equivalent. The ship case is S-0504 (b). |
| 6 backing, confidence, Constants, State table | c (follows 6) | — | — |
| 8 (S-0408 sentence) | c | S-0408 claim 13 is already updated. | — |
| 15 | a, already done | Rewording: the required behaviour is the observable credit order equal to the object-turn order (S-0203 claims 3-4), not a particular internal pass. | `econ/sell.rs:364` `building_turn` -> `automatic` (390) runs in the object turn; hold release in `econ/houses.rs:35`. |
| Scenarios H3, H4; aircraft note; Open questions | c | Depot timing from the waiting gate; aircraft pad question answered by S-0501. | — |

New quirk switches: none.

## S-0315 Suppression and panic (r6 -> r7)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Leaving a living host* / *Losing the host* | b | Release events are S-0614 claim 12, exit placement S-0614 claim 13, host loss S-0614 claim 15. Unloading and consumption set 50 ticks of suppression before release. Teleport releases a naval parasite at launch. Locomotor replacement releases only a naval parasite. | No parasites; only the threshold is imported (`combat/data.rs:1216`). |
| 7 | b | Non-naval: suppressed means silent removal on living-host release. Unsuppressed exit follows S-0614 claim 13. Naval ignores suppression on living-host release. Host loss: suppression removes either kind unless the parasite is temporarily invulnerable, otherwise forced placement with no hold (S-0614 claim 15, `quirks.parasite_host_loss_suppresses_naval`). Sonic and healing hits release before the health change, so a lethal sonic hit uses the living-host rule. | — |
| Constants | b | New: 50 ticks of suppression on unloading or consumption (S-0614 claim 12). | — |
| State and ordering table and text | b | Rows split into living-host release and host loss. Release precedes the hit's health change. | — |
| Backing, scenarios P4 n/o/p, Open questions | c | — | — |

New quirk switches: `quirks.parasite_host_loss_suppresses_naval` (owned by S-0614 claim 15; already
declared at `crates/deterministic/rules/src/quirks.rs:1231`).

## S-0202 Random streams (r19 -> r20)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, *Fence post, fence line* | c | Drops "stock data has no such building type". The rule applies when imported rules define matching posts. Still a pointer to unspecified fence rules. | Fence lines are a seam (no fence code). |
| 7 backing | c | — | — |
| 10 (parasite ranged choices) | b | S-0614 claims 9-10 on the scenario stream. Drone delivery to a non-infantry host: R(0,1) after effects, before the hit; none for infantry. Grappling push start: R(0,1), then 3 wakes each R(−180,180), R(−64,64) before creation. Strike: 3 splashes each R(0,1), R(0,n−1). Next push choices only after a hit that keeps the host attached. | `rng.rs:100` `ranged`; no parasite code. |
| 10 backing table | b / c | Parasite row (S-0614 P8, b). Fence-line row `unmeasured` (c). | — |
| 14 closing paragraph | c | History rewording; ownership unchanged. | — |
| Map-loading table, *Fence lines* row | c | Same condition rewording ("if no such posts, no draws"). | Seam. |
| Scenarios R3, R7 | c | Procedure rewrites (bystander, checkpoint). | — |

New quirk switches: none.

---

## Summary

(a) items that need M6 work:
1. **S-0303 F6 debris allocation rewrite** (`combat/death.rs:83-122`, shared with S-0312 death
   debris). Covers: wrapped divisor, `num::int::rem` (zero divisor and `MIN rem -1`), ℓ wrap with
   negative *g*, the i64 quirk-off magnitude, a −1 maximum placing min(*a*, ℓ) pieces, and the
   never-placeable guard limited to {0, −2}.
2. **Importer debris warnings** (`convert/src/sim_combat.rs:1310-1348`): warn on a retained −1
   maximum; the never-placeable warning must accept −2.
3. **Parasite-flag area suppression** (S-0303 claim 2, S-0306 claim 21): import the warhead
   `Parasite` key, and have projectile detonations skip area damage while keeping animation and
   debris. The latch itself is S-0614.

(a, already done): S-0308 claim 6, S-0310 claim 6, the S-0310 ordering paragraph, S-0314 claim 15,
the F6 non-positive *N* rule, and the S-0303 scenario boundaries A3 b, A4 b, A9 d.
(a, seam, no work): S-0314 claim 6 depot waiting phase.
Possible spec defect: the S-0306 P3 falsifier contradicts its Expected text.

Quirk switches named in changed lines: `quirks.parasite_host_loss_suppresses_naval` is the only one
new to these specs (S-0614 claim 15, already declared). Existing ones re-cited:
`quirks.crash_death_weapon_twice` (now non-rocket) and `quirks.random_magnitude_overflow` (new F6
off-branch meaning). Pointer only: `quirks.destroyed_rocket_still_detonates` (S-0506 claim 20).

## Addendum: revisions after d158117

- **S-0301 r7** (a19e764): table R rank 4 gains the *impossible* answer for a spawner weapon whose mobile firer is under the parasite hold, ahead of rank 9's organic-only test. (b): pointers into S-0506 claim 10 and S-0614 claim 8, which lanes A1 and B1 implement in `combat::check`.
- **S-0306 r9** (69cf98a):
  - Claim 20: inaccurate and airburst projectiles skip only (b) and (c), and (a)'s raise still applies. (c): `projectile::detonation_point` already raises first and then returns.
  - The parasite-warhead exception moves from latching weapons to any projectile with a parasite-flag warhead (S-0303 claim 2). (b): this is lane B1's.
  - Attribution is lost when the firer is removed by any means (believed: when it starts to cloak). (c): `projectile.rs` already drops a firer removed by any means, and cloaking is not modelled.
  - The first-turn exception for an earlier removal is S-0203's turn pass, already modelled. (c).
