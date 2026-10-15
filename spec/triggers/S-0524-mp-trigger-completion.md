---
id: S-0524
title: Multiplayer trigger completion (remaining stock actions and entry raises)
topic: triggers
revision: 3
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0524: Multiplayer trigger completion (remaining stock actions and entry raises)

## Summary

Extends S-0011's non-team action subset for official multiplayer maps, including the loose official maps, and specifies additional object-entry raises. Superweapon grant, set-charge and reset effects belong to S-0512 claims 12 to 15; this spec's action table does not cover those actions. Storm-start and storm-stop actions (44 and 45) belong to S-0513 claim 20, and the waypoint curtain action belongs to S-0513 claim 22: no launching house, ground layer only, without the leaving-stage skip. These rules cover skirmish only; network games, campaigns, the team system and computer-player targeting policy are out of scope.

## Terms

- **The waypoint point.** S-0011 claim 28's cell-centre point at the terrain surface, including its 416-lepton bridge lift. Claims 1, 2 and 4 use a cell, not this point.
- **Demolition warhead.** The rules' demolition-charge warhead, named in Import mapping.
- **Starting-building record.** A snapshot of each house's owned building types and cells, taken before the first tick (claim 9). It records neither health nor tags.
- **Weather storm.** The weather superweapon's storm, called the *ion storm* in S-0104. S-0513 owns its effects.
- **Entry raise.** An entered-by raise on the entered object's tag, with the entrant as causing object. S-0011 claim 18 supplies event matching and the triggering house; claims 8 to 12 there supply evaluation, repetition and nested-raise suppression.
- **Projectile turn.** One projectile's share of a tick, counted from 1 at its first turn, not from the action's firing tick (S-0306 claim 26).

## Behaviour

1. **Lightning storm at a waypoint (code 102).** An active weather storm blocks this action. Otherwise it requests a storm centred on the waypoint-slot cell, with the imported duration and deferment, owned by the trigger's owner house, possibly no house. A deferred storm does not block another request: the later request replaces the centre, owner and duration. Its remaining deferment is the smaller of the existing positive remaining deferment and the new deferment; if none is pending, it uses the new deferment. A zero deferment starts the storm immediately. The waypoint's bridge lift has no effect on the centre cell. S-0513 claims 14 and 19 own the shared countdown and storm-owner rules; claim 20 owns the other storm controls.
   - Backing: `measurable` (C1). Subsequent storm effects inherit S-0513's backing; they are not newly specified here.
   - Confidence: high

2. **Lightning strike at a waypoint (code 90).** Each action creates a storm cloud at the waypoint-slot cell's cloud point, whether or not a storm is active. Two actions at the same waypoint create two clouds: there is no previous-strike exclusion. Each cloud takes one scenario-stream list pick over the imported cloud-animation list, before its animation-creation draws (S-0202 claims 6 and 10; S-0109 claims 3 and 13).

   Pending clouds produce bolts through the weather effect even with no active storm. Their damage uses the storm owner in force when the bolt occurs, not necessarily the trigger owner or the house in force at cloud creation. With no storm since game setup that owner is absent. Adding a cloud to an active or clearing storm can delay its return to idle until the cloud is no longer present (S-0513 claims 18 and 19). Cloud height follows S-0513 claim 17, not the waypoint-point formula; unsafe animation lists follow S-0513 claim 21.
   - Backing: `measurable` (C2) for repeated clouds, the absent-storm bolt and draw order. Attribution after an intervening storm-owner change, delayed clearing and image-dependent cloud height inherit S-0513 claims 17 and 19's `measurable` backing (scenario SW8).
   - Confidence: high for repeat acceptance and the list pick; medium for bolt processing and attribution

3. **Nuclear strike at a waypoint (code 95).** The action creates a projectile of the fixed-name nuclear payload weapon (Import mapping), with that weapon's damage, projectile type and warhead, no firer and no owning house. The target is the waypoint-slot cell. Its start is 20 000 leptons above the cell centre at the surface height, with a 416-lepton lift on a high-bridge deck cell. Its projectile speed is the engine constant 50 leptons per tick, not the weapon's speed; its initial velocity is the downward vector in Formulas. These are distinct inputs to subsequent flight (S-0306 claims 10 to 15). S-0306 claim 26 gives qualifying vertical-flight predictions: the nuclear wait starts in projectile turn 202 over empty flat ground, turn 199 over a centred stationary ground vehicle, and turn 201 over an empty high-bridge cell at deck height minus 100 leptons. These predictions are conditional on the projectile properties and absence of intervening detonation conditions specified there, not universal timings for every payload configuration. Nuclear-warhead detonation follows S-0306 claim 25. The number slot is unused.

   If no weapon has the fixed name, the original does not safely reject it. Our engine gives the action no effect and the importer warns (departure).
   - Backing: `measurable` (C3; S-0306 claim 26, scenario P40) for the launch inputs, qualifying vertical flight, target, damage, lack of kill credit and unused number slot. These are proposed tests, not completed measurements.
   - Confidence: high for the launch inputs; medium for the subsequent flight hand-over

4. **Create a crate at a waypoint (code 108).** The action uses S-0212 claim 25's single-cell nearby search centred on the waypoint-slot cell: *float* locomotion class if that centre is water, *track* otherwise, *normal* movement zone, no reach region, deck mode off, no height or further test, no no-overlay condition, bridges allowed and no reference cell. The search takes no random draw; its result depends on the action's tick as specified there.

   No crate is placed when all 256 scenario crate places are occupied. Placement also fails if the selected cell is outside the height-aware usable area or holds any overlay; it does not retry another candidate merely because the selected cell has an overlay. A placed crate uses the water-crate overlay on water and the wooden-crate overlay otherwise. Placement redraws an already visible cell but does not explore or reveal it for any house.

   The action works with the crate setting off. A successful placement takes the lifetime draw in Formulas, after any draws caused by overlay placement. With crates off, expiry processing is disabled: an uncollected trigger crate remains even after that lifetime elapses. With crates on, expiry and replacement are deferred to planned S-0520.

   Number-slot value 20 leaves the crate random. Every other value is reduced modulo 256 to a value in 0 … 255. Values 0 … 18 select a fixed crate kind; values 19 … 255 instead use the random pickup selection. Pickup effects and numbering are deferred to planned S-0520. Reduction is exact integer arithmetic, including for negative inputs.
   - Backing: `measurable` (C4) for search, placement, images, settings, lack of reveal, expiry gating, lifetime draw and fixed versus random contents. The exact effects of each numbered kind are deferred to planned S-0520.
   - Confidence: high

5. **Demolition blast at a waypoint (code 63).** The number slot names a waypoint. At its waypoint point *P* the action deals five successive area damages, each 100 hit points, with the demolition warhead, no attacker and no attacking house (S-0303 claims 1 and 2). The points are *P*, *P* + (85, 85, 0), *P* + (−85, 85, 0), *P* + (85, −85, 0), and *P* + (−85, −85, 0), in integer leptons. Their observable precedence is centre, increasing-x/increasing-y, decreasing-x/increasing-y, increasing-x/decreasing-y, decreasing-x/decreasing-y: a death or trigger action caused by an earlier blast can change what the later blasts hit.

   After those damages, a defined waypoint receives up to four direct bridge-destruction attempts at its cell, stopping at the first success (S-0511 claims 1 and 2). These direct attempts ignore the bridge-destruction setting; bridge effects within each area damage retain S-0303 claim 18's own gate. Thus the bridge can fall even when that setting is off.

   An absent waypoint marker has cell (0, 0), as S-0011 claim 33 says, but the damage point's height there is not established. Our engine gives codes 63 and 66 no effect for an absent marker and warns (safe bad-data departure). Invalid waypoint numbers follow S-0011 claim 33's departure too.
   - Backing: `measurable` (C5) for damage points and direct bridge attempts. Exact precedence among the five damages is `unmeasured`: C5 checks health totals, not a discriminating intervening death/trigger effect. The absent-marker point is `unmeasured` because its height has not been traced and no stable expected height is available.
   - Confidence: high for amounts, points and direct-attempt count; medium for precedence

6. **Large light flash at a waypoint (code 66).** The number slot names a waypoint. The action requests a drawing-only flash at its waypoint point when the demolition warhead has the bright flag and the presentation gate allows reduced-detail effects: nonzero detail level and the adaptive drawn-frame-rate test. The action itself does not bypass either gate. A positive warhead flash-size fraction supplies the flash-size level as given in Formulas. Otherwise size 300 gives integer level 63 (Formulas). The level is dimensionless, not a radius in cells or pixels; its precise visible footprint is not specified here. Neither the action nor the flash takes a random draw or deals damage. An undefined marker follows claim 5.
   - Backing: `measurable` (C6) for the point, fallback versus maximum-size image, bright/detail gates and absence of damage or draws. Exact numeric size-level calibration and the visible footprint are `unmeasured`: no independent pixel-to-level calibration is available. The adaptive frame-rate gate is `unmeasured`: C6 deliberately uses a host above its threshold and does not exercise its hysteresis boundaries.
   - Confidence: high for size and gates; medium for the presentation-only classification

7. **Set base ambient light (code 73).** The integer number slot becomes base ambient in hundredths, without division or rounding. It also sets target ambient unless a weather storm is active. The fade and global-effect exceptions are S-0104 claims 4 and 14.
   - Backing: `measurable` (S-0104 scenario L13, using code 73)
   - Confidence: high

8. **Resize local area (code 40).** Action items four to seven become the local area's left, top, width and height, as signed integers in the local area's screen-aligned cell-width/cell-height units, not map-axis cell coordinates (S-0009 claim 4). Subsequent usable-area tests, scrolling and the radar rectangle use the new area (S-0009 claims 4 and 5; S-0101 claim 8). Every legal cell receives terrain recalculation under S-0010 claims 8 and 19; this may create a previously eligible tile animation, not an unconditional restart of existing animations. Pathfinding connectivity is rebuilt for the new area (S-0212 claims 3 and 4). Building radar tracking is refreshed for the new rectangle; gameplay effects on a building excluded by shrinking the area remain open.
   - Backing: `measurable` (C7) for the rectangle and observable scroll/order/radar results. Animation creation and pathfinding rebuild details are `unmeasured` here: C7 has no animated-tile fixture or connectivity probe that separates a rebuild from updated area tests. Excluded-building effects are `unmeasured` because their effect is not traced.
   - Confidence: high for the rectangle assignment and recalculation; medium for subsequent effects

9. **Restore starting buildings (code 130).** Before the first tick, after tag setup and the game-start terrain pass, each house's owned buildings are recorded by type and cell. Buildings created during play are not added. The action resolves the number slot as a house value (S-0011 claim 23); an unresolved house gives no effect.

   A recorded cell holding a building of the recorded type and house receives a healing hit of minus the type's strength, distance 0 leptons, demolition warhead, no attacker or attacking house, neither ignoring defences nor preventing escape. S-0302 claims 5 to 8 and 11 decide its outcome: an ordinary demolishable, unprotected building returns to full health, even with zero versus; immunity can prevent the hit and an undemolishable building can instead lose health. Restoration is not an unconditional health assignment.

   Otherwise a building of that type and house is created at the cell. Its foundation overlays are removed, and mobile occupants are displaced to nearby cells if placement succeeds. If a blocking building prevents placement, or placement otherwise fails, no replacement remains. Displaced objects are then offered their original positions; any whose return placement fails are removed (Edge cases). Removed foundation overlays are not restored on failure. Restoration supplies no recorded tag to a newly created building.
   - Backing: `measurable` (C8) for the snapshot, healing and ordinary displacement. Processing order, exact nearby-search options, failure-side overlay persistence and failed-return removal are `unmeasured`: C8 does not provide a discriminating ordered multi-building fixture or a guaranteed failed-return fixture. They must not be inferred from its ordinary successful placement.
   - Confidence: high for healing-hit metadata and the snapshot; medium for forced placement

10. **Engineer capture and tag transfer.** S-0509 claims 2 and 8 give a capture two entry raises: the general building-entry raise and the capture raise. Both precede capture's own ownership and tag changes. Without intervening trigger changes they see the old owner and tag; trigger actions can change that state, and the later raise sees those changes. The general raise can also change which engineer outcome applies (S-0509 claim 3).

    If the engineer still carries a tag whose trigger chain includes the carry-over flag, the captured building receives that tag, replacing its previous tag and dropping one of the old tag's carriers (S-0011 claim 2). Losing its last carrier does not by itself delete the old tag. Capture gives the building to the engineer's house. No relative order between tag carry-over and ownership change is prescribed here; both follow the entry raises and precede the engineer's destruction raise (S-0509 claim 8).

    The engineer's own tag gets one destroyed-by-anything raise with the engineer as causing object before the engineer is removed. When the tag carried over, this raise acts on the tag now shared with the captured building; actions selecting that tag's present carriers can affect the captured building. A damage-only attempt has the general entry raise and the engineer's destruction raise, but no second entry raise or carry-over transfer. That outcome cannot occur in skirmish because its enabling rule is always off (S-0509 claims 2 and 6). Capture eligibility belongs to S-0509 claims 3 and 7, not a universal health gate here.
    - Backing: `measurable` (C9; S-0509 claim 8, scenarios E2 and E6) for capture raises, transfer, established ownership precedence and destruction raises. Survival of a replaced tag with no other carrier is `unmeasured`: C9 does not poll that orphan tag independently. Damage-only notifications are `unmeasured` because that outcome is unreachable in skirmish; the relative order of capture's tag carry-over and ownership change remains unresolved.
    - Confidence: high

11. **Vehicle hijacking and tag transfer.** A vehicle thief's takeover raises the vehicle's existing tag with the thief as causing object. That raise sees the vehicle's old owner and tag. The carry-over rule of claim 10 then replaces the vehicle's tag when applicable, and takeover gives it the thief's house and records the thief's type as its hijacker type (S-0312 claim 13). The thief is removed without a destroyed-by-anything raise on its own tag. The crew effects on later vehicle destruction belong to S-0312 claim 13. This claim covers vehicles only. A vehicle-like building instead follows S-0509 claim 17: its entering unit does receive a destroyed-by-anything raise, and any tag carry-over follows the ownership change. Its eligibility, backing and low confidence remain those of that claim.
    - Backing: `measurable` (C9 variant B)
    - Confidence: high

12. **Other object entries.** Garrison entry, infantry entry into its ordered target, sabotage entry into its target building, and vehicle entry into its ordered target building can raise the entered object's tag with the entrant as causing object. The entered-by action effects can therefore occur before the entrant's garrison, sabotage or building-entry effect. This is not a rule that merely receiving an enter order raises the tag, nor that every entry consumes its entrant.

    Spy infiltration is a consuming entry (S-0510 claim 9): the building's entry raise precedes the infiltration effect, and the spy's own tag receives destroyed-by-anything with the spy as causing object after that effect and before removal. This is S-0509 claim 2's same general entry raise, not an extra one. S-0510 claim 1 owns eligibility and its vehicle-like destination exception.

    Some other building-entry paths that consume infantry or vehicles also raise destroyed-by-anything on the entrant's own tag, with itself as causing object, before removal. No universal consumption notification is established: vehicle hijacking is the explicit exception of claim 11, and the remaining entry paths have not all been classified.
    - Backing: `measurable` (C10) for garrison entry; spy raise counts and order inherit S-0510 claim 9's `measurable` backing (scenario I8). Its effect-relative state and presence qualifications retain that claim's `unmeasured` backing. Other entry classifications, consumed-entrant notifications and their precise eligibility are `unmeasured`: a complete mapping to data-only fixtures has not been established. C10's remaining additional fixtures are conditional proposals, not evidence of a complete list.
    - Confidence: medium for garrison and the ordered-target paths; low for the complete entry classification

## Formulas

**Trigger crate lifetime (claim 4).** Use S-0202 claim 6's unit-interval draw *u*, not a raw value times 2⁻³¹. It takes R(0, 2147483646), including any rejected raw values. With imported double-precision regeneration field *C* in game minutes:

> *L* = RZ₅₃(450 × *C*), *H* = RZ₅₃(1800 × *C*)
>
> *T* = trunc(RZ₅₃(*L* + RZ₅₃(RZ₅₃(*H* − *L*) × *u*))) ticks

RZ₅₃ denotes double-precision rounding under S-0205 claim 5 (`quirks.chop_rounding`); trunc converts to a signed integer toward zero. The original uses floating point at each multiplication, subtraction and addition. For positive finite *C* within the integer conversion domain, the bounds before rounding are half and twice *C* game minutes, with 900 ticks per game minute. Lifetime assignment does not imply expiry while crates are off. Out-of-domain conversions follow S-0205 claim 7's common-form wrap and huge/non-finite-result rules; there is no added clamp.

**Flash-size level (claim 6).** Without a positive imported size, exact integer arithmetic gives

> *r* = min(63, max(21, ⌊300 × 64 / 256⌋)) = 63 flash-size levels.

For positive imported single-precision fraction *f*, level = trunc(RZ₅₃(63 × min(*f*, 1))) (dimensionless). The original multiplies in floating point under S-0205 claim 5, then uses claim 7's common integer conversion. This path has an upper bound of 63 but no lower bound of 21.

**Nuclear start and velocity (claim 3).** Start = target surface-centre point + bridge lift + (0, 0, 20 000), in signed integer leptons, with no rounding. Let *p* be the stored double constant 1.5707483884323126 radians (slightly below π/2). Let *S*(*p*) and *C*(*p*) denote, respectively, the sine and cosine values supplied by S-0105 F1's table trigonometry. Initial velocity components, in double-precision leptons per tick, are

> *V*ₓ = RZ₅₃(RZ₅₃(*C*(*p*) × *C*(*p*)) × (−100))
>
> *V*ᵧ = RZ₅₃(RZ₅₃(*S*(*p*) × *C*(*p*)) × (−100))
>
> *V*_z = RZ₅₃(*S*(*p*) × (−100)).

The table's cosine at this angle is not exactly zero, so the horizontal components are tiny but not exactly zero. Subsequent movement and integer position conversion are S-0306 claims 10 to 15's, not a constant-speed 400-tick fall inferred from the stored speed of 50.

**Blast displacements (claim 5).** Adding (±85, ±85, 0) uses signed integer leptons, no rounding. Ambient assignment (claim 7), action rectangle items (claim 8) and crate-kind reduction (claim 4) also use integer arithmetic only.

## Constants

| Name (ours) | Value | Unit |
|---|---|---|
| Damage per demolition blast | 100 | hit points |
| Blast count and displacement | 5, 85 | blasts; leptons along each map axis |
| Maximum direct bridge attempts | 4 | attempts per action |
| Large flash size | 300 | flash-size units |
| Fallback flash-size bounds | 21 … 63 | dimensionless levels |
| Nuclear drop height | 20 000 | leptons |
| Nuclear projectile speed; velocity scale | 50; 100 | leptons per tick |
| Crate capacity | 256 | crates per scenario |
| Explicit random-crate action value | 20 | number-slot value |
| Fixed crate kinds | 0 … 18 | kind numbers |
| Crate lifetime factors | 450; 1800 | ticks per game minute of regeneration field |

## State and ordering

| State | Event | Result |
|---|---|---|
| no active or deferred storm | code 102 | request storm at waypoint cell |
| storm deferred, not active | code 102 | later centre/owner/duration, no extension of the remaining positive deferment |
| storm active | code 102 | no effect |
| any storm state | code 90 twice at one waypoint | two clouds; their strikes follow S-0513 claim 18, subject to claim 21's unsafe-list departures |
| crates off | code 108 succeeds | crate appears and is not removed by lifetime expiry |
| recorded building still present, same type/house | code 130 | healing hit, subject to ordinary damage rules |
| recorded building absent or different | code 130 | attempted replacement |

Observable precedence:
- Code 63's five damage effects precede its direct bridge attempts. With the game's bridge-destruction setting off, a deck occupant can be damaged before the direct attempt drops the deck.
- A code-90 cloud's list pick precedes its creation draws, so the recorder identifies those draws in that order.
- Capture's two entry raises precede its own ownership/tag changes; trigger actions can change the state seen by the later raise (S-0509 claims 2 and 8). Vehicle hijack entry precedes takeover. The consumed engineer's destruction actions can already act on the newly tagged captured building.
- Spy entry actions precede the infiltration effect; the spy's destruction raise follows the effect and precedes removal (S-0510 claim 9).
- A building created after the first tick begins is absent from the restoration record.

## Edge cases and quirks

- **Failed restoration return** (`quirks.restoration_failed_return_removes_unit`, default on, original behaviour). If forced building placement fails and a displaced unit cannot be put back, it is removed. With the switch off, failed restoration preserves displaced units by retaining them for deterministic nearby placement rather than deleting them. The exact original failed-return fixture and relocation options remain unmeasured (claim 9).
- **Restoration and healing protections.** Claim 9 delivers an ordinary healing hit, not unconditional restoration. S-0302 claim 7 owns the undemolishable-healing quirk and its alternative.
- **Repeated code-90 actions.** Repeated actions at the same waypoint are accepted by default. `quirks.repeated_lightning_strike_suppressed` (default: off) controls this compatibility behaviour. When on, after one cloud has been created by code 90 at a waypoint, further code-90 actions at that waypoint create no cloud.
- **Crate settings.** By default, trigger-crate placement ignores the crate setting, while expiry processing is disabled when crates are off. `quirks.trigger_crate_respects_setting` (default: off) makes code 108 have no effect when crates are off. `quirks.trigger_crate_expiry_while_disabled` (default: off) allows an uncollected trigger crate to expire while crates are off, after its assigned lifetime. These switches affect only the stated behaviours.
- **Empty storm animation lists.** S-0513 claim 21 owns these invalid-data departures. An empty cloud list divides by zero in the original; our engine makes no cloud and warns. An empty bolt list is also unsafe in the original: our importer warns, the cloud-height term is 0, and strikes make no bolt animation but otherwise happen.
- **No nuclear weapon; absent blast/flash marker.** Safe no-effect departures are stated in claims 3 and 5. They require importer warnings, not quirk switches.
- **Old tag without carriers.** Tag transfer changes carrier counts but does not itself delete the replaced tag. Ordinary firing and expiry remain S-0011's.

Unsafe missing-weapon, empty-animation-list and missing-overlay outcomes have `unmeasured` backing and medium confidence: no proposed scenario deliberately runs the original into unsafe data. Their warning/no-effect alternatives are design, not measured original behaviour.

## Import mapping

**I1. Action items.** Add these rows to S-0011's action table. Its integer parsing, house resolution, waypoint decoding and parameter kinds remain authoritative.

| Code | Reads | Claim |
|---|---|---|
| 40 | fourth to seventh items: left, top, width, height | 8 |
| 63 | number slot as waypoint number | 5 |
| 66 | number slot as waypoint number | 6 |
| 73 | number slot as integer hundredths | 7 |
| 90 | waypoint slot | 2 |
| 95 | waypoint slot, number slot unused | 3 |
| 102 | waypoint slot | 1 |
| 108 | number slot as crate kind, waypoint slot | 4 |
| 130 | number slot as house value | 9 |

**I2. Rules fields.** Initial absent-key defaults below are engine defaults, not stock values. A missing value in a later layer keeps the held field (S-0006 claims 1 and 10).

| Section, key | Parse | Feeds | Initially absent |
|---|---|---|---|
| `[General]` `LightningStormDuration` | signed integer, ticks | claim 1 duration | 900 ticks |
| `[General]` `LightningDeferment` | signed integer, ticks | claim 1 deferment | 250 ticks |
| `[CrateRules]` `CrateRegen` | double-precision decimal or percent, game minutes | *C* | 10 game minutes |
| `[CrateRules]` `WoodCrateImg` | overlay type name | land crate | none |
| `[CrateRules]` `WaterCrateImg` | overlay type name | water crate | none |
| `[CombatDamage]` `C4Warhead` | warhead type name | demolition warhead | none (S-0303 claim 19) |
| weapon section `[NukePayload]` | fixed weapon type name, not a configurable key | claim 3 payload | unsafe original lookup; claim 3 departure |
| demolition warhead section, `Bright` | boolean | flash permission | false |
| demolition warhead section, `CombatLightSize` | decimal or percent, narrowed to single precision | flash-size fraction | 0 |

Initial absent-key defaults have high confidence and `unmeasured` backing: the supplied stock layers set these fields, and omitting a later override retains the held value rather than exposing the initial default. No stock-layer-removal fixture is proposed.

S-0513 claims 17, 20 and 21 supply cloud-point height, storm controls and unsafe animation-list handling; its Import mapping supplies the cloud and bolt lists. Planned S-0520 will specify crate-kind numbering and pickup effects. A missing crate overlay is unsafe original data: our engine warns and gives the placement no effect.

## Measurement scenarios

Use the glossary's measurement setup: stock data, director and mods off except explicitly named test-map overrides. Use a tiny purpose-made skirmish map, one human and one bystander computer player kept away from the observed test area, standard mode, 800 × 600, game-speed index 2. Apply the bystander void rule. Use one-time tags unless a variant explicitly needs repeated firings; arm the tested action with an elapsed event after loading. Record actual firing ticks, health and positions with a read-only recorder and tick-labelled captures. Repeat each deterministic variant twice; random variants use 30 runs with each run's observed seed and stream state recorded, not a seed written into the oracle.

- **C1 — storm requests (claim 1).** Fire code 102 at W, then at W2 during the positive deferment, and finally at W3 while the storm is active. Record the centre cell, owner, remaining deferment and activation tick read-only. Expected: W2/its trigger owner replaces W/its owner without lengthening the countdown; W3 does not replace it. Falsified by a deferred request being blocked, a longer countdown, or an active storm changing centre/owner. Use named map overrides for positive duration/deferment and a zero-deferment control. The control starts immediately. Two different owner-house variants check attribution of enemy damage, with S-0513's rules for protection.
- **C2 — repeated lightning (claim 2).** With no storm, fire two code-90 actions at W in one trigger; compare one action and two distinct-waypoint actions. Observe clouds, bolts, target health and draws. Expected: two clouds and two bolts in both two-action runs; each cloud list pick precedes its creation draws. Isolate the list pick using a named map override giving one cloud animation with no optional random creation effects, then a two-animation list; account for S-0109's unavoidable creation draws. Falsified by suppression at a repeated waypoint, absence of bolts merely because no storm is active, or a list pick occurring after creation. Read kill credit: with no storm since setup no house receives bolt kill credit. Different credit falsifies that attribution subcase.
- **C3 — nuclear payload (claim 3).** Fire code 95 beside an enemy vehicle; repeat with different number-slot values but the same waypoint and stock payload. Record start position, velocity, impact cell, projectile turns, arrival/flash/damage ticks, health changes and kill credit. Expected: same target and effect for all number-slot values, payload damage subject to S-0302/S-0303, no house credited; a bridge-cell run starts 416 leptons higher than its terrain surface. A different target, number-dependent effect or house credit falsifies those parts. For flight, use S-0306 claim 26's qualifying vertical setup and P40's named test-map overrides and controls, including detonation altitude above the start and no intervening detonation condition. Repeat empty flat ground, a centred stationary ground vehicle and an empty high-bridge cell twice each. Expected integer displacement is (0, 0, −100) leptons per non-detonating turn; the nuclear wait starts in turns 202, 199 and 201 respectively, at the surface, vehicle position and deck height minus 100 leptons. Turn 1 is the first projectile turn: record its tick instead of assuming it is the action tick. Another wait turn or point, horizontal drift or a 50-lepton descent in a qualifying fixture falsifies the prediction; an unrelated detonation condition voids that timing fixture. Other projectile configurations use their applicable S-0306 flight rules. These are proposed tests, not completed measurements.
- **C4 — trigger crates (claim 4).** With crates off, create crates at empty land and water cells, a building-occupied centre and a centre with an overlay. Predict the selected cell from S-0212 claim 25 at the recorded action tick; if it holds an overlay, expect no crate rather than another search. Record cell, image and stream draws. Put a waypoint inside shroud and out of every sight source: placement must not explore it; moving a unit there later reveals the crate. Any exploration change caused by placement falsifies the no-reveal rule. Keep one crate uncollected beyond 1800·*C* ticks: with crates off it remains. In a separate crates-on run, record the assigned duration and exact lifetime draw using the random-state recorder and Formulas; ignore replacement crates when tracking the original. Falsified by a different duration, a draw on a failed placement, or expiry with crates off. Test kinds 0 … 18 separately, and compare 19, 20, 255 and 256: 256 gives kind 0, while 19/20/255 use random pickup selection. Check against the expected effects and draws specified by planned S-0520. For capacity, place 256 spaced trigger crates with crates off; the 257th must fail without a lifetime draw. A crate or extra draw then falsifies capacity gating.
- **C5 — demolition damage (claim 5).** Put test infantry at recorded positions around W on flat ground. Predict health loss from the five points and S-0303, accounting for deaths between hits. Compare a bridge W with bridge destruction off; expect direct attempts still to destroy a destructible span after the damages. Falsified by losses incompatible with those points/amounts or a direct attempt blocked solely by the setting. A span already destroyed supplies the unsuccessful-attempt control. This does not distinguish the exact order of the five damages.
- **C6 — flash (claim 6).** Named test-map overrides set the demolition warhead bright flag on and its flash-size fraction to 0. At high detail and above the reduced-effects frame-rate threshold, fire code 66 at W. Capture the flash and health, and record both random streams. Expect a flash at W and no health or stream changes attributable to the action. Compare bright off, detail 0 and invented positive size fractions below, at and above 1: no flash in either disabled case; fallback and fractions at or above 1 give the same flash image under otherwise identical conditions, while a sufficiently small positive fraction gives a smaller flash. A contrary image, gate or health/stream result falsifies that subclaim. These image comparisons do not independently calibrate the exact numeric size levels. Host-frame-rate hysteresis is not tested.
- **C7 — area resize (claim 8).** Enlarge a small local rectangle with code 40 after tick 30, using unequal left/top and width/height so map-axis interpretation gives a different result. Record scrolling, radar border and a move order into the added strip; calculate bounds from S-0009 claim 4 and S-0101 claim 8. Expected: all use the new screen-aligned rectangle. Falsified by old bounds or map-axis interpretation. Shrink a separate empty strip as a control. Effects on newly excluded buildings, animation creation and zone rebuilding are exploratory only.
- **C8 — restoration (claim 9).** Pre-place a demolishable, unprotected human building B, and create a second building during play by a trigger. Damage B, then restore that house; expect B healed but the later building, if subsequently removed, not recreated. Sell B, park a vehicle in its foundation and restore again: expect B and a nearby displaced vehicle. Capture B for another house before restoring: the same-type wrong-owner building must not be healed as if it were the recorded house's building. Use named test-map warhead overrides to compare zero versus and ordinary versus: the ordinary healing outcome is the same. Compare an undemolishable B under S-0302 claim 7: it loses 1 hit point instead. A map-override immune building retains its damaged health under S-0302 claim 8. Any contradictory health, later-building restoration or missing ordinary displacement falsifies the corresponding subclaim. Read restored building tags: no recorded tag is assigned to a new replacement. Failed-return removal remains unmeasured without a guaranteed fixture.
- **C9 — capture and hijack (claims 10, 11).** A tagged enemy building's repeating entered-by trigger changes its carriers to a third house that remains hostile to the engineer, and produces a counter calibrated to distinguish two firings in one tick. A human engineer carries a repeating tag with carry-over and a destroyed-by-anything event that produces a separate calibrated counter. Capture the building. Expected: two building entry firings before capture's own transfer, with the second able to act on the building already transferred by the first; capture leaves the human owner. The engineer destruction counter fires once, and later building destruction reacts through the carried-over tag. Compare an engineer without carry-over: no tag transfer, but still two entry firings and one engineer destruction firing. Use S-0509 claim 8's E6 variant in which the engineer destruction action changes all its tag's carriers to a fourth house: expected final building owner is that house, showing that both capture and carry-over precede destruction actions. Falsified by other counts, owners or later-tag effects. These tests do not distinguish tag carry-over before ownership change from carry-over after it. No damage-only fixture is proposed: S-0509 claim 6's enabling rule cannot be turned on in skirmish. Variant B uses a vehicle thief and tagged enemy vehicle: expect one old-tag entry action, human takeover, transfer when flagged and no thief destruction firing. Kill the captured vehicle later; the hijacker-type crew follows S-0312 claim 13. A consumption firing or missing remembered hijacker contradicts claim 11. Vehicle-like buildings use S-0509 claim 17's E12 instead, not variant B.
- **C10 — other entries (claim 12).** Order a stock garrison-capable infantry unit into a tagged occupiable building. Its entered-by action creates a calibrated visible counter effect. Expected: one entry firing before garrisoned fire can occur; no destroyed-by-anything firing merely for being retained inside the garrison. Falsified by no entry firing or a raise merely on receiving the order before arrival. For a spy, use S-0510 claim 9's I8 fixture: expected one building entry firing, then one spy destruction firing in the arrival tick, followed by removal. A missing, extra or reversed firing falsifies that notification rule. I8 does not independently discriminate the effect-relative state or the spy's presence during the destruction raise. Additional proposed runs use tagged entry targets for a saboteur and vehicle entering a consuming building, with a counter on each entrant's tag. These remain conditional until the responsible ability spec establishes valid stock or named test-data fixtures; no universal consumption count is claimed from them.

## Open questions

- Results of the qualifying vertical-flight tests in S-0306 claim 26, scenario P40, and C3 remain outstanding. Predictions are turns 202 over empty ground, 199 over a centred stationary ground vehicle, and 201 over an empty high-bridge cell at deck height minus 100 leptons; none is a completed measurement. Other projectile configurations, including steering or speed changes, require their applicable flight rules.
- A discriminating observable for the relative order of engineer capture's tag carry-over and ownership change (S-0509 claim 8).
- Full classification of object-entry paths and consumption notifications (claim 12).
- Results for buildings excluded by a local-area shrink (claim 8).
- Exact relocation settings and a guaranteed failed-return fixture for restoration (claim 9).
- Stable surface-height behaviour of absent markers for codes 63 and 66 (claim 5).
- Exact visible flash footprint and independent pixel-to-size-level calibration (claim 6).
- Results of S-0513 scenario SW8's proposed tests for strike attribution after an intervening owner change, cloud height and storm clearing.
