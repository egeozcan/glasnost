---
id: S-0514
title: Chrono weapons (temporal erasure)
topic: combat
revision: 8
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-17
measurements: []
requests: [Q-0984]
supersedes: []
---

# S-0514: Chrono weapons (temporal erasure)

## Summary

Temporal weapons attach to targets and erase them over time rather than reducing health.
This spec covers attachment, progress, interruptions, completion and its extra experience gain.
Skirmish only; network games and campaigns are out of scope.

Attacker teleport movement belongs to S-0207 claim 20; superweapon teleportation, including
jumpjet eligibility, belongs to S-0513 claims 34 to 40. Ordinary firing eligibility belongs to
S-0301 claim 9 and S-0310 claim 7, gunner hand-over to S-0308 claim 6, damage protection to
S-0302 claims 8 and 10, power totals to S-0405 claims 1, 2 and 7, bunker target redirection to
S-0309 claim 24, and destruction credit to S-0311 claims 7 to 10. Where those specs disagree
with the erasure-specific rules below, the corresponding cross-spec correction is required
before joint hand-off.

## Terms

- **Erasing shot:** a shot whose warhead has the temporal flag, when temporal erasure is the
  detonation effect selected by claim 20.
- **Temporal beam:** an attacker's persistent erasing capability, initially supplied when its
  primary weapon's warhead has the temporal flag. Gunner transfer follows S-0308 claim 6.
  One beam attaches to at most one target. It need not be visibly drawn.
- **Lock, let go:** attach a beam to a target, or detach it.
- **Leading attacker:** the first attached attacker, or its replacement under claim 9.
  Others are **joined attackers**.
- **Erasure amount:** integer remaining amount *R*, initially 10 times type strength.
- **Erasure update:** the target-turn opportunity to reduce *R*. A skipped target turn gives
  no update; a range release under claim 11 gives no reduction.
- **Being erased:** the glossary state. A beam lock sets it. The beam-free leaving stage of
  S-0513 claim 39 also counts, but is not a temporal beam lock.
- **Fall-back to rest:** the event of S-0215 claims 13 to 15, with aircraft exceptions in
  S-0501 claim 18. This is not a behaviour named idle, stop or sleep. References below to
  making an attacker fall back mean requesting this event, not assigning a fixed behaviour.
- **Removed without death:** removal without the target's normal death animation, death weapon,
  debris, rubble, wreck or survivors. Destruction bookkeeping can still occur (claim 13).

## Behaviour

### Locks and their immediate effects

1. **Delivery.** A temporal detonation needs both its firer and its target still present.
   Claim 20 decides whether temporal erasure is selected. Claims 1 to 4 describe delivery
   from a firer that holds a temporal beam; claim 21 owns the missing-beam case.
   An object target must be in the ground render layer, otherwise this effect does nothing.
   For a jumpjet this is S-0102 claim 3's render-layer test, not a test of its flight state
   (S-0502 claim 20), combat height or in-the-air state. Being in the ground render layer
   permits delivery but does not bypass claim 3's lock refusals.
   A bunkered vehicle is redirected to its bunker. A cell or a non-combat object only makes
   the firer's old beam let go. Temporal erasure replaces ordinary direct and area damage,
   but does not suppress the detonation's other presentation effects.
   - Backing: `measurable` (E1, E4).
   - Confidence: high.

2. **Refused-hit side effects.** A delivered erasing hit on infantry, a vehicle, aircraft or
   a building clears its spawner slots (S-0506 claim 13) and frees units it mind-controls.
   The firer's old beam lets go. These effects occur even if the new lock is refused,
   including a protected mind controller losing its controlled units
   (`quirks.erasure_refused_hit_side_effects`).
   - Backing: `measurable` (E5).
   - Confidence: high.

3. **Refusal.** No lock attaches when the target type is not erasable, when iron curtain or
   force shield protects it (S-0513 claim 27), or when the target is a vehicle still standing
   in the cell of the vehicle factory with which it is in contact. A beam already attached
   to the firer also refuses the new lock. This last test reads beam attachment, not the
   general being-erased state: the beam-free leaving stage alone is not this refusal.
   - Backing: `measurable` (E4). The leaving-stage distinction is `unmeasured`: no scenario
     here makes a pending shot arrive from that otherwise inactive firer during this stage.
   - Confidence: high.

4. **A successful lock.** With no beam already attached to the target, the firer becomes
   leader and sets *R* by F1. Otherwise it joins without resetting *R*. The target becomes being erased, is
   deselected and releases any beam of its own. A building is switched off and its house
   is scheduled to reconsider power and radar; power recalculation follows S-0405 claim 7,
   not an immediate recalculation inside the hit. Slot animations follow S-0109 claim 21.
   Lock and ordinary release request redraws, not flashes.

   Each successful lock, including a joining attacker, gives a gattling target one spin-down
   update with multiplier 1, not an unconditional stage loss (S-0609 claims 3 and 11).
   Spin arithmetic and the zero-amount quirk follow S-0609 claim 3; spin sound effects follow
   S-0609 claim 13.
   - Backing: `measurable` (E3, E7, E11 parts c and d).
   - Confidence: high.

5. **First-lock notices.** Only the first attachment requests attack notices: a harvester
   vehicle owned by the local player requests its harvester-under-attack announcement,
   subject to the notice's normal suppression conditions. A building that is not
   insignificant requests its base-under-attack notice unless its foundation is 1 × 1 cells
   and its type names a vehicle to pack up into (S-0415 claim 9). Joiners request neither.
   - Backing: `measurable` (E1 part c), including the packing-building exclusion.
   - Confidence: high (building exclusion); medium (announcement suppression).

### Progress and interruptions

6. **Initial amount.** F1 uses the target type's strength, not current health. Damaging a
   target before the lock does not shorten erasure.
   - Backing: `measurable` (E1, E2).
   - Confidence: high.

7. **Reduction.** Each target turn that reaches the erasure update reduces *R* by F2's sum
   of raw weapon damages. The weapon is the one each attacker would select with no target
   (S-0308 claims 3 to 10). Firepower, veterancy damage bonuses and versus multipliers do
   not scale this damage. Elite weapon replacement can still change the chosen weapon.
   Only the leader and the 51 most recent joiners contribute. Completion occurs when the
   resulting signed *R* is at most 0. With positive *R* and zero step, it never completes.
   - Backing: `measurable` (E1, E3, E12).
   - Confidence: high.

8. **Duration.** With positive *R*, positive constant step and no overflow, completion
   takes F3's number of erasure updates. For an unchanged update order, with the target
   already processed in the lock tick *n* and one subsequent update every tick, removal
   occurs in tick *n* + *k*. Normally the projectile entered play after its target, so the
   target has acted before the projectile locks (S-0203 claims 3 and 5). Skipped turns or
   re-entry into play can change this date (S-0203 claims 10 and 11).
   - Backing: `measurable` (E1).
   - Confidence: high (update count); medium (calendar-tick application).

9. **Leader replacement.** If the leader lets go while others remain, the most recent
   remaining joiner becomes leader with the same *R*. The target stays being erased and
   a building's switch is unchanged. A departing joiner simply stops contributing.
   - Backing: `measurable` (E3).
   - Confidence: high.

10. **Last release.** When the last attacker lets go before completion, the target stops
    being erased and loses all progress. A later lock starts from F1. A building is switched
    on unconditionally, even if switched off before the lock; power and radar rechecks are
    scheduled (`quirks.erasure_release_switches_on`).
    - Backing: `measurable` (E3, E7).
    - Confidence: high.

11. **Open-topped distance.** A leading attacker inside an open-topped transport lets go
    when F4's distance from its own position to the target exceeds the imported warp
    distance. That update makes no reduction, even if another attacker becomes leader.
    Joiners are not checked (`quirks.erasure_distance_leader_only`). Outside an open-topped
    transport, distance alone does not release a beam.
    - Backing: `measurable` (E8) for the range gate, height term and leader-only release.
      F4's table-root shortfall and rounding-mode dependence are `unmeasured`: E8 has no
      verified legal placement that distinguishes them from an exact root at the threshold.
      F4's out-of-domain separations and overflow thresholds are `unmeasured`: no legal
      setup here reaches those magnitudes.
    - Confidence: high.

### Completion and release causes

12. **Removal.** Completion plays the imported warp-away animation at the target and
    removes the target without death, with claim 13's credit. Building garrison occupants
    are removed as S-0508 claim 15 specifies; absorbed units are deleted without individual
    destruction credit. A bunker releases its vehicle as S-0606 claim 13 specifies.
    Removing a bunkered vehicle clears its bunker as S-0606 claim 14 specifies.
    A slave master's slaves are freed. Transport passengers are recursively
    removed too, but their destructions have no killer and award the erasing attacker no
    passenger kill credit or experience.
    - Backing: `measurable` (E9). The bunkered-vehicle branch is `unmeasured`: ordinary
      temporal delivery redirects to the bunker, and no independent reachable setup is known.
    - Confidence: high (target, occupants, passengers, slaves); medium (bunker branches).

13. **Credit and extra experience.** The leader is the credited killer for the target's
    destruction. Normal experience recipients and kill value follow S-0311 claims 7 to 10.
    Independently, a trainable leader receives F5's direct gain, using target cost for the
    target house and attacker cost for the attacker's own house. This gain has no victim-rank
    multiplier, allied zero or unscored-type exclusion. It goes to the leader itself rather
    than its transport or launcher. Thus two gains can go to the same attacker, but ordinary
    credit can instead go elsewhere (`quirks.erasure_double_experience`). Loss announcements
    and destruction-trigger events accompany completion.
    - Backing: `measurable` (E6, E9 parts f and g) for gains, recipients and cost bases.
      F5's exact rounding, unordered comparison and separately capped gain composition are
      `unmeasured`: E6's exact binary fractions do not distinguish alternative rounding or
      cap ordering, and do not produce a not-a-number sum. Exhaustive score/counter
      equivalence is `unmeasured`: these scenarios do not isolate every destruction-bookkeeping consumer.
    - Confidence: high (gains, cost bases); medium (bookkeeping equivalence).

14. **Release causes.** An attached attacker lets go on arriving in a new cell, reaching
    the common fall-back-to-rest effects, delivering another erasing hit even at the same target, being taken
    under mind control or freed, being locked by another temporal beam, or being removed.
    Gunner exit follows S-0308 claim 6; claim 11 adds the leader's range release. Arrival
    by teleport follows the new-cell rule. Removing the target invalidates all attached
    beams and makes their attackers fall back to rest.

    The fall-back release shares S-0215 claim 13's once-per-tick common-effects limit.
    Aircraft early exits that skip those effects do not cause this release. The resulting
    behaviour is chosen by S-0215 claims 14 and 15 or S-0501 claim 18, not necessarily guard.
    Merely clearing the ordinary target, assigning the stop or sleep behaviour, or processing
    either of those behaviours is not this event. A stop order clears target and destination
    without directly releasing the beam (S-0213 claim 8). In the ordinary stationary attack
    case, subsequent attack processing with no target falls back to rest and then releases.
    There is no promise of release in the tick that the order applies.

    Death alone is not established as a separate release: a destroyed attacker retained
    in play may continue erasing until removal, but this is a hypothesis, not a guaranteed
    lifetime rule.
    - Backing: `measurable` (E10) for ordinary cell arrival, fall-back, mind control, counter-lock,
      removal, stop-versus-sleep and retained-death tests. Teleport arrival and gunner exit are `unmeasured`
      here: their complete movement/transfer paths have not been independently retraced.
      The owners above retain their own backings.
    - Confidence: high (ordinary release paths), medium (teleport, gunner exit),
      low (continuation after death).

15. **House destruction.** S-0414 claim 5 owns surrender. A trigger that destroys all
    objects of a house detaches the beams on each erased victim and makes their attackers
    fall back to rest before the destructive hit. It does not hand leadership on or switch buildings back on.
    - Backing: `measurable` (E10 part g).
    - Confidence: high.

16. **Owner change.** An erased target changing owner is not exempt from ordinary attacker
    target loss. Clearing that ordinary target alone does not detach the beam. Subsequent
    behaviour can still cause claim 14's release. Whether it does so immediately is open.
    An erased target normally cannot make its own teleporting locomotor jump because its
    destination is cleared (S-0207 claim 20). Under a pending one-application point
    suppression (S-0207 claim 27) the destination is kept, and the target still cannot jump.
    - Backing: `measurable` (E10 part h) for target clearing without direct detachment.
      Subsequent behaviour-driven release is `unmeasured`: that transition was not traced,
      so no deadline is claimed.
    - Confidence: medium (clearing itself); low (later continuation).

### While attached

17. **Target inactivity.** Infantry, vehicles and landed aircraft being erased clear their
    target and destination (except that a pending point suppression, S-0207 claim 27, keeps the
    destination) and skip ordinary movement, turning and firing. Buildings skip
    ordinary behaviour processing and firing; this spec does not claim that every auxiliary
    building activity stops. Damage protection and power totals use their owning specs.
    A target with an attached beam cannot itself obtain another lock (claim 3).
    - Backing: `measurable` (E11 part a).
    - Confidence: high.

18. **Sparkles.** If the imported sparkle animation exists, mobile targets being erased or
    phasing in create it on their turns whose global tick number is divisible by 24.
    Infantry and vehicles place it 120 leptons farther along each horizontal map axis than
    their position, with unchanged height; aircraft use their own position. Skipped turns
    create nothing. Buildings use their damage-fire positions: position *i*, counted from
    0, sparkles when (*tick* + *i*) is divisible by 24. With none, they use their own position
    and tick residue 0. This is presentation only.
    - Backing: `measurable` (E11 part b) for cadence and mobile placement. Exact conversion
      of building drawing positions to world positions is `unmeasured`: no scenario here
      predicts that transformation independently. Absence of sparkles on skipped turns is
      `unmeasured`: E11 does not establish a setup that isolates a skipped turn on a sparkle tick.
    - Confidence: high (cadence, mobile placement); medium (building placement).

19. **Attacker firing.** Holding the same target makes the firing check answer rearming
    (S-0301 claim 9), so a stationary attacker does not fire repeated erasing shots at it.
    A counter-lock releases its existing beam and prevents a new lock while attached.
    Vehicle moving-fire restrictions remain S-0310 claim 7's responsibility.
    - Backing: `measurable` (E10 parts d and f).
    - Confidence: high.

### Effect selection and beam capability

20. **One selected special hit.** Of the following effects, at most one is selected by a
    projectile detonation. The first enabled row wins; all later rows are suppressed even
    when the selected effect cannot succeed, has no suitable target or has lost its firer.
    This is precedence between observable outcomes, not successive attempts to apply them.

    | Priority | Enabled effect | Detail owned elsewhere |
    |---|---|---|
    | 1 | takeover | S-0601 claim 5 |
    | 2 | bomb planting | S-0519 claim 2 |
    | 3 | electric charging | S-0518 claim 11 |
    | 4 | parasite attachment | S-0614 claims 1 to 4 and 24 |
    | 5 | temporal erasure | claims 1 to 4 and 21 |
    | 6 | locomotor replacement | selection only here |
    | 7 | air-strike designation | S-0519 claim 16 |
    | 8 | direct rocking, only when the projectile target is a vehicle | selection only here |
    | 9 | bomb disarming | S-0519 claim 7 |
    | 10 | disguise-making | selection only here |
    | 11 | nuke-making | selection only here |

    Direct rocking here means a hit directed at a vehicle, not rocking from ordinary area
    damage. Its flag with a non-vehicle target does not suppress the later rows. All other
    rows are selected by their flags regardless of target eligibility. With no selected row,
    ordinary damage applies. A warhead can therefore request several rows, but cannot both
    erase and attach a parasite, charge, designate, plant or disarm a bomb through the same
    selected hit. In particular charging and parasite attachment outrank erasure, while
    erasure outranks locomotor replacement, designation and disarming.

    When takeover, bomb planting or electric charging suppresses attachment, a carried
    parasite remains off the map, unattached and inactive, rather than making a failed-latch
    return, regardless of target latchability or the selected effect's success (S-0614
    claim 24). That claim owns `quirks.parasite_suppressed_latch_strands_unit` (default
    original) and its return-only alternative; neither setting permits fallback attachment.

    Radiation creation and the shared detonation presentation are outside this exclusion:
    radiation can accompany any selected row, and an unsuccessful selected hit still permits
    the shared presentation of claim 1. This does not make the later special rows eligible.
    - Backing: `measurable` (E13).
    - Confidence: high.

21. **Erasing delivery without a beam: safe rule.** In our engine, a firer with no temporal
    beam makes no lock and causes none of claim 2's target side effects. It does not acquire
    a beam merely by delivering an erasing shot. Ordinary damage and later selected special
    effects remain suppressed under claim 20; shared presentation remains eligible. This is
    a deterministic design departure, not an ordinary lock refusal to which claim 2 applies.
    It covers a secondary-only erasing weapon and an elite replacement that did not supply
    a beam at creation; possession after gunner transfer still follows S-0308 claim 6.

    The original has an unsafe missing-beam failure after attempting the target's spawner
    clearing and control release; it does not establish a valid beam-free lock. Whether those
    side effects become observable before termination is not established by a measurement.
    - Backing: `unmeasured` for the original failure's visible outcome: no oracle run has
      isolated this inconsistent weapon arrangement; E14 proposes one. Safe no-effect
      delivery is `design`, to be tested by the same input in our engine.
    - Confidence: high (safe rule and absence of a valid original beam-free lock), low
      (observable original side effects before termination).

22. **Boarding without gunner hand-over.** Boarding a non-gunner transport does not itself
    release or transfer the passenger's beam. Any attachment still present at boarding is
    retained and can continue reducing the target's erasure amount while the holder is a
    passenger. This does not bypass release on arrival in a new cell, fall-back to rest or
    control release during boarding: those events can already have detached it. A pending
    erasing shot that arrives after boarding can establish a lock when delivery otherwise
    qualifies. Open-topped passengers remain subject to claim 11's leader-only distance test.
    A closed transport adds no such distance test. Gunner entry and exit are separate,
    governed by S-0308 claim 6.
    - Backing: `measurable` (E15).
    - Confidence: high (boarding alone does not release), medium (isolating retained
      attachment from the preceding arrival and fall-back events).

## Formulas

*S*, weapon damage *d* and remaining *R* are signed 32-bit integers. Multiplication, addition
and subtraction wrap as S-0205 claim 1 specifies. Claims of exactness and positive durations
below assume no overflow. Numeric examples are invented, not stock data.

**F1, starting amount:** *R*₀ = 10 × *S*, exact integer erasure units.

**F2, step:** *D* = Σ *d*ᵢ, summing the leader and at most 51 most recent joiners, exact integer
erasure units per update. Raw damage is re-evaluated at every update.

**F3, progression:** *R*ⱼ = *R*ⱼ₋₁ − *D*ⱼ, where *j* counts reductions, not elapsed ticks.
Completion is the first resulting signed *R*ⱼ ≤ 0. For positive starting amount and constant
*D* > 0 without wrap, *k* = ⌈10 *S* / *D*⌉, rounding the rational quotient upward to whole
updates. For example, *S* = 100 and damages 30 and 20 require 20 updates. Zero *D* holds a
positive amount indefinitely. Negative *D* increases it until wrap or a later change.

**F4, distance:** *Δx*, *Δy*, *Δz* are signed 32-bit coordinate differences in leptons,
with subtraction wrapping under S-0205 claim 1. The points are the attacker's own position
and the target's own position, not a building's foundation-centre reference point.

> *q* = RZ₅₃(RZ₅₃(RZ₅₃(*Δx*²) + RZ₅₃(*Δy*²)) + RZ₅₃(*Δz*²))
>
> dist = trunc(*r*(*q*))
>
> Release when dist > 256 × *W*.

Differences are converted exactly to floating point before squaring. Multiplications and
additions use double-significand floating point; RZ₅₃ rounds toward zero as S-0205 claim 5
specifies. Here *r* is S-0204 claim 5's table square root, not the mathematical square root,
and the final conversion truncates toward zero to integer leptons. The stated root contract
covers squared separations below 2⁵³ square leptons, where *q* is an exact integer; larger
separations remain unresolved, not replaced by exact-integer sums. The warp distance *W*
is integer cells, and its threshold uses signed 32-bit integer multiplication with wrap.
There is no building-size distance reduction. `quirks.table_root_distance` off uses the
exact square root instead, truncated to integer leptons, under S-0204 claim 5's alternative.

**F5, direct gain:** *e* is single-precision experience, *A*ₜ and *A*ₐ are integer actual
costs in credits as S-0311 F1 defines, for target and attacker houses respectively. Veteran
ratio ρ and cap *c* are double precision.

> *s* = RZ₅₃(*e* + RZ₅₃(*A*ₜ / RZ₅₃(*A*ₐ × ρ)))
>
> *e*′ = RZ₂₄(*c*) when *c* ≤ *s* or *s* is not a number, otherwise RZ₂₄(*s*).

RZ₂₄ narrows to single precision toward zero. The cap comparison uses *s* before narrowing.
This uses S-0311 claim 10's experience arithmetic but not its kill value or recipient-cost
house. When the leader also receives ordinary kill experience, its final experience is the
ordinary S-0311 claims 7 to 10 gain evaluated with *e*′ as its starting experience, including
that gain's own cap and single-precision narrowing. The two gains are not combined into one
uncapped increment. For F4 and F5, `quirks.chop_rounding` off selects S-0205 claim 5's
round-to-nearest alternative; F4's table-root narrowing follows S-0204 claim 5.

## Constants

| Name | Value | Unit | Role |
|---|---|---|---|
| Initial multiplier | 10 | erasure units per strength point | F1 |
| Damage contributors | 52 | attackers | Leader plus 51 newest joiners |
| Sparkle period | 24 | ticks | Global phase, not time since lock |
| Mobile sparkle displacement | 120 | leptons per horizontal axis | Infantry and vehicles |
| Cell conversion | 256 | leptons per cell | F4 |
| Warp distance default | 5 | cells | Absent-key default |

## State and ordering

| State | Event | Result |
|---|---|---|
| unlocked target | delivered hit from a beam holder | Side effects even on refusal; otherwise first lock |
| firer without a beam | delivered erasing hit | No lock or claim 2 side effects, by claim 21's design rule |
| attached | another lock | Join without resetting progress |
| attached | erasure update | Range release, or reduction and possible completion |
| attached | leader releases with others left | Newest remaining joiner leads with unchanged amount |
| attached | last release | Progress lost; target resumes; building switched on |
| attached | completion | Target removed, credit applied, attackers fall back to rest |
| attached | other target removal | Attachments invalidated, attackers fall back to rest |

A refused lock can free controlled units without erasing their controller. Two temporal firers
whose pending shots reach each other cannot establish reciprocal locks: the first successful
lock releases its victim's beam and causes that victim's later lock attempt to be refused.
The direct experience gain can remain on a passenger while ordinary credit goes to its
transport, or exist when ordinary allied/unscored credit gives none. At target removal, a
leader receiving both gains has the final experience defined by F5's composition, including
the intermediate cap and narrowing rather than a single combined gain. Both recipients'
gains are present by target removal.

## Edge cases and quirks

- `quirks.erasure_refused_hit_side_effects`, default original: claim 2's target side effects
  survive refusal. Off: refused locks do not clear spawners or free controlled units.
  The firer's previous beam still lets go.
- `quirks.erasure_release_switches_on`, default original: last release switches the building
  on. Off: restore its switch state from before the first lock.
- `quirks.erasure_distance_leader_only`, default original: only an open-topped leader is
  checked. Off: apply the distance limit to every attached open-topped passenger.
- `quirks.erasure_double_experience`, default original: F5 supplements ordinary credit.
  Off: only ordinary S-0311 credit remains.
- Type strength rather than health is intentional. Zero-step indefinite attachment is retained.

## Import mapping

| Key and section | Concept | Parse form | Absent-key default |
|---|---|---|---|
| `Temporal`, warhead | Temporal effect, beam supplied by primary warhead; missing beam uses claim 21 | boolean | no |
| `MindControl`, warhead | Claim 20 takeover selection | boolean | no |
| `IvanBomb`, warhead | Claim 20 bomb-planting selection | boolean | no |
| `ElectricAssault`, warhead | Claim 20 electric-charge selection | boolean | no |
| `Parasite`, warhead | Claim 20 parasite selection | boolean | no |
| `IsLocomotor`, warhead | Claim 20 locomotor-replacement selection | boolean | no |
| `Airstrike`, warhead | Claim 20 designation selection | boolean | no |
| `directrocker`, warhead | Claim 20 direct-rocking selection on vehicle targets | boolean | no |
| `BombDisarm`, warhead | Claim 20 disarm selection | boolean | no |
| `MakesDisguise`, warhead | Claim 20 disguise selection | boolean | no |
| `NukeMaker`, warhead | Claim 20 nuke-making selection | boolean | no |
| `Primary`, `Secondary`, `ElitePrimary`, `EliteSecondary`, object type | Weapon arrangements relevant to beam creation and claim 21 | weapon names | S-0308 claims 3 to 10 |
| `Gunner`, `OpenTopped`, transport type | Hand-over versus retained passenger beam; range-gate eligibility | booleans | no for each |
| `Warpable`, object type | Erasable target | boolean | yes |
| `OpenToppedWarpDistance`, CombatDamage | *W* | integer cells | 5 |
| `WarpAway`, General | Completion animation | animation name | none |
| `ChronoSparkle1`, General | Sparkle animation | animation name | none |
| `Strength`, object type | *S* | integer | S-0302 |
| `Damage`, weapon | Raw *d* | integer | S-0301 |
| `Trainable`, object type | Direct-gain eligibility | boolean | S-0311 |
| `VeteranRatio`, `VeteranCap`, General | ρ, *c* | decimal doubles | S-0311 |
| `Harvester`, vehicle type | Harvester notice | boolean | S-0402 |
| `Insignificant`, object type | Suppress building notice | boolean | no |
| `UndeploysInto`, building type, and `Foundation`, building art | Packing-building notice exclusion | vehicle name and foundation token | S-0415 claim 9 and S-0108 |
| `IsGattling`, `RateDown`, numbered `Stage1` and `EliteStage1` keys, object type | Spin-down flag, amount and thresholds | boolean and integers | Held value from earlier layers; initially no for the flag and 0 for the amounts and thresholds |
| Numbered `DamageFireOffset0` keys, building art | Building sparkle positions | drawing-coordinate pairs | S-0302 claim 13 |

## Measurement scenarios

Use the glossary measurement setup: stock executable and data, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted),
speed index 2, 800 × 600, a small purpose-made skirmish map and a bystander computer player
kept outside the observed area. Void runs where it interferes or is defeated. Named test
overrides below are temporary map sections, or named test art where stated, not changes to
the executable. Repeat deterministic parts three times. Use video plus a read-only tick/state
recorder for attachment, position, selection, spin amount, experience and counts; script normal
input where useful. Do not infer locks from a supposed flash. Every predicted difference below
is falsified by the opposite result under the same conditions.

- **E1, timing (claims 1, 5 to 8).** A stationary erasing infantry unit attacks a stationary
  enemy vehicle. Record lock/removal ticks, health and chosen raw damage. With no skipped
  turns or re-entry, expect removal at *n* + ⌈10*S*/*d*⌉, unchanged health beforehand and
  no damage to adjacent bystanders. One tick earlier/later falsifies the calendar prediction.
  (b) Join two/three attackers, including elite and firepower-bonus variants, and compare
  named victim armour variants with different versus multipliers: expect raw chosen-weapon
  sums, not multiplied damage. Named zero/negative-damage variants test no
  reduction/increasing amount.
  (d) For signed wrap use named target strength 214748365 and raw damage 1: expect initial
  amount −2147483646 and completion at the first update. In a second variant use strength
  214748364 and damage −8: expect initial amount 2147483640, then −2147483648 and completion
  at the first update. A saturated or wider positive amount instead falsifies wrap.
  (c) In separate runs make the local house own the harvester or building target; clear
  announcement cooldowns. Expect ordinary eligible notices on first lock, none on joining.
  Repeat with named 1 × 1 test building art, once with and once without a pack-up vehicle.
  Expect no base notice only in the packing variant; a notice there falsifies the exclusion.
  Other first-lock silence is inconclusive until normal notice suppression is excluded.
- **E2, health (claim 6).** Repeat E1 after damaging the victim below half health.
  Expect identical duration; a shorter duration falsifies the type-strength rule.
- **E3, joins/releases (claims 4, 9, 10).** A locks, B joins later, then A moves into a new cell.
  Record amount and completion date: expect unchanged amount at join and leadership transfer,
  with only the subsequent damage sum changed. Move B into a new cell too and relock: expect a fresh full
  amount. A reset on joining/handover or retained progress after last release falsifies it.
- **E4, gates (claims 1, 3).** Compare protected and unprotected targets, a named not-erasable
  type, factory-contact vehicles before and after leaving, landed and airborne aircraft,
  jumpjet infantry before take-off, during ascent and after landing, and ground force-fire.
  Record each object's render layer at impact as well as height and jumpjet flight state.
  Expect refusal in the listed protected cases, no effect on targets outside the ground
  render layer, and ground force-fire only releasing the old lock without damage. An
  otherwise eligible ground-layer jumpjet locks regardless of its flight-state label.
  A lock outside that layer or refusal caused only by flight state falsifies the layer gate.
  Use a named slow temporal projectile already in flight when its firer becomes
  locked: expect refusal when it arrives. A lock in a refused case falsifies that gate.
  In separate runs remove the firer or target before impact: expect no new lock; a new
  lock falsifies the presence gate. Force-fire at a terrain object while attached elsewhere:
  expect only the old beam to release, not ordinary temporal damage. Finally attack a
  bunkered vehicle: expect its bunker, not the vehicle, to become erased. Record the
  detonation presentation in an eligible delivery: the normal animation remains present.
- **E5, refused side effects (claim 2).** Hit a protected mind controller holding two units
  and a protected carrier with aircraft out. Expect control freed/spawner slots cleared
  without a lock. Repeat with a firer already erasing another victim: that old beam releases.
  Retained control/slots or attachment falsifies the corresponding claim.
- **E6, experience (claim 13, F5).** Named test data sets actual attacker/rookie victim costs
  to 1000/500 credits, ratio 4 and cap 2.5. Expect direct 0.125 plus ordinary 0.125 per erasure,
  veteran after four and elite after eight. Repeat with differing house cost factors, allied
  and unscored victims, ranked victims, and a leader passenger in a trainable open-topped transport.
  A ranked victim still gives direct 0.125 with those cost inputs; only ordinary credit gains
  the victim-rank multiplier.
  Expect F5 to use the leader's own-house cost, survive allied/unscored exclusions and stay
  on the passenger while ordinary credit goes to the transport. A missing or redirected
  direct gain or wrong cost factor falsifies these parts. Exact rounding, unordered
  comparisons and near-cap composition are not isolated by these inputs.
- **E7, building power state (claims 4, 10).** Lock a locally owned power-draining building
  switched off before the hit. Record switch, house totals and radar through release.
  Expect no output/drain while locked and switch on after last release, with totals updated
  in the next eligible house update. A restored off state falsifies the original quirk.
- **E8, range (claim 11, F4).** Named warp distance 3 cells. A passenger leads, then its
  transport drives away: expect release at the first target update whose F4 table-root
  distance exceeds 768 leptons, and no reduction that update. Repeat with an on-foot leader
  and passenger joiner: expect no passenger release. Move the leader into a new cell so the passenger inherits leadership
  beyond the limit: its next check releases. Include a height difference and exact-boundary
  positions. Record both actors' own positions and calculate F4 for each update; use an
  on-foot leader control to verify that distance alone does not release it. Repeat with a
  large-foundation building target to distinguish its own position from its foundation
  centre and verify no building-size reduction. Release at equality, at a distance predicted
  only by a foundation shift/reduction, or checking the joiner falsifies the corresponding
  rule. This tests the range gate, not the table-root shortfall: without a verified legal
  boundary placement that distinguishes the two roots, that distinction stays unmeasured.
- **E9, completion (claims 12, 13).** Erase separately a garrison, absorbing building,
  occupied bunker, slave master and loaded transport. Record remaining objects, animation,
  announcements, death effects, losses, kills and experience. Expect garrison/absorbed units
  removed, bunker vehicle retained, slaves freed, transport passengers removed without
  extra attacker kills/experience, and warp-away without ordinary death effects. Compare
  loaded/empty transports. (f) A destruction-tag trigger must fire at completion.
  (g) Passenger losses must be registered without attacker credit. Survival, ordinary
  death effects, missing trigger or extra passenger credit falsifies the relevant outcome.
- **E10, releases (claims 14 to 16, 19).** During separate locks, (a) move into a new cell,
  (b) stop, (c) mind-control/free the attacker, (d) counter-lock it, (e) kill infantry with a
  death sequence, (f) leave it stationary through several normal firing intervals,
  (g) use a destroy-all-house trigger, (h) let a previously launched slow mind-control shot
  change the erased victim's owner. Expect direct releases in a, c and d, no further shots in f,
  detached attackers falling back before g's destructive hit, and ordinary target clearing without
  detachment by that clearing alone in h. Record later behaviour-driven release without a
  predicted deadline; refused takeover is inconclusive. In e, continuation until removal
  is the low-confidence hypothesis; earlier release falsifies it. Also remove a locked
  target by a trigger and verify all attackers fall back under their ordinary rest-choice rules.
  In b record the stop-order application, target/destination clearing, the next attack
  processing and the first common fall-back effects separately. Expect no release caused by
  the clearing itself, but release when that fall-back is reached; an intervening target
  erasure update may still reduce the amount. A release directly at clearing, or attachment
  retained through the common fall-back effects, falsifies the respective prediction.
  In a separate named map-behaviour-order variant, put an attached firer in sleep, and then
  in stop behaviour, without movement or another release event. Record attachment and
  behaviour processing: those processings alone must not detach. A release there falsifies
  that distinction. These are behaviour tests, not a spec for team scripts.
  Continue e through attacker removal:
  expect its beam gone by removal, even if it persisted during the death sequence. In a
  separate named slow-projectile run with no other attackers, allow two shots from one firer
  to be in flight before the first lock. On the second impact expect the old attachment released and a fresh F1
  amount, even at the same victim. Retained progress falsifies that delivery-release rule.
  Missing release/fall-back falsifies the corresponding release part.
- **E11, target and presentation (claims 4, 17, 18).** (a) Lock moving/firing infantry,
  vehicles, landed aircraft and armed buildings: expect ordinary movement/turn/fire to stop.
  (b) Begin long locks at three global tick residues. Expect mobile sparkle creation on
  global multiples of 24, with claim 18's mobile displacement. Named building art with two
  distinct fire positions gives residues 0 and 23; no positions gives residue 0. A different
  cadence/phase or mobile position falsifies that part. Also record phasing-in mobile
  units across three global tick residues: expect the same phase and mobile placement.
  Phase relative to arrival rather than global tick falsifies that part.
  Building coordinate conversion is recorded
  without a predicted result. (c) Select an own target before lock: expect
  deselection, no lock/release-induced flash. (d) Named gattling spin-down tests start well
  above a threshold and just above it. Expect spin reduction in both, a stage reduction only
  across the threshold. A zero decrement resets spin to 0. Record immediately, before
  unrelated later spin-down updates. Then attach a joining attacker: expect one more
  spin-down update at that lock, not continuous decay between locks. Record the spin sound
  and playing mark against S-0609 claim 13: expect looping to end and the mark to clear on
  each lock. An unconditional stage drop, missing joiner update, continuous vehicle decay
  during the lock, or retained playing mark falsifies the corresponding rule.
- **E12, contributor limit (claim 7).** Named positive, distinct damage values and sufficient
  range allow 53 attachments. Record join order and amount reductions. Expect the leader
  plus 51 newest joiners only; all 53 contributing or a different excluded joiner falsifies it.

- **E13, mixed special hits (claim 20).** Use named map weapon/warhead variants, changing
  only effect flags and the capability needed for the selected effect. Deliver single shots
  with each adjacent pair in claim 20's table enabled, and with temporal erasure paired
  separately with takeover, planting, charging, parasite, designation and disarming. Choose
  eligible targets for the winning row; for disarm tests pre-plant a long-delay bomb. Record
  owner, bomb presence, charger count, parasite host, beam attachment, locomotor state,
  designation state, disguise and nuclear effect, plus health and neighbouring health.
  Expect only the higher-priority selected outcome, with no ordinary damage. Then make
  that selected effect ineligible without changing flags: expect no lower effect and no
  ordinary damage, not fallback selection. For direct rocking, compare vehicle and building
  targets: only a vehicle makes that row suppress disarming and the later rows. Repeat
  with an imported radiation level and detonation presentation, recording the radiation
  field and presentation separately: they remain eligible with a selected special hit.
  A lower row, both selected outcomes, or ordinary damage falsifies the corresponding
  exclusion. Runs in which ordinary firing legality prevents delivery are inconclusive.
  Propose a delayed projectile whose target eligibility changes after firing, not a forced
  state write, to isolate such cases.

  For carried parasites, follow S-0614 claim 24's fate test as well. For each of takeover,
  planting and charging, retain a clear firing cell and record the carried unit's existence,
  health, map presence and attachment at detonation and for 300 ticks afterwards. Compare
  successful and refused higher effects, latchable mobile targets, targets made non-latchable
  during flight, building and forced-cell targets where firing permits, and lost-target
  detonations. Include a vehicle firer whose charging fails and named infantry-parasite
  variants able to charge or plant, plus a takeover-capable variant. Use three completed
  shots per reachable arrangement; shots removed without detonation test a different case.
  Expect the surviving carried parasite to remain extant, off-map, unattached and inactive,
  without return, recovery timer, death effects, kill experience or replacement. Record the
  higher effect's outcome separately. Return, attachment or removal caused solely by carrier
  completion falsifies this fate. Distinguish an extant off-map unit from silent removal
  with the recorder, not screen disappearance. An attachment-selected control whose target
  becomes non-latchable must instead attempt S-0614 claim 4's return. Firing refusal is
  inconclusive; change eligibility during flight rather than bypassing firing legality.
- **E14, missing beam (claim 21).** Named map variants give a non-temporal rookie primary
  and either an erasing secondary selected against the test target or an erasing elite
  replacement on a map-placed elite firer. Confirm no beam before firing, record whether
  delivery is reached, process survival, spawner slots and controlled-unit counts on the
  victim, and any lock. Compare an otherwise identical primary-temporal control. In the
  original, the hypothesis is an unsafe failure at delivery, possibly after spawner/control
  side effects; record which effects, if any, become observable. A normal continuing lock
  falsifies the no-valid-beam-free-lock conclusion. Earlier failure or failure to fire does
  not settle delivery. In our engine the expected design result is continued simulation,
  no lock and no spawner/control side effects; any such effect or ordinary damage fails
  the design rule. No original crash outcome is claimed as measured.
- **E15, passenger retention (claim 22).** Use a non-gunner transport and an erasable victim
  with sufficient strength for a long lock. Send the firer to board while a slow erasing shot
  is in flight, timed to arrive after boarding. Record all cell-arrival/fall-back events,
  boarding, beam possession/attachment and each target reduction. Expect the pending hit
  to lock and erasure to continue from inside the transport; compare open-topped and closed
  variants. In another timing variant arrange impact after the last cell-arrival event but
  before accepted boarding, and void the run if any intervening release cause occurs.
  Expect boarding itself to retain the attachment and amount. If that timing cannot be
  achieved, it does not measure retention of a pre-existing lock. Move the transport away:
  expect only an open-topped leader to face claim 11's distance release. Loss of capability
  at boarding, detachment solely at boarding, failure of otherwise eligible post-boarding
  delivery, or a closed-transport distance release falsifies the respective part.

## Open questions

- Later behaviour-driven detachment after an erased victim changes owner.
- Whether death paths always release before removal, despite no release in beam cleanup itself.
- Exact world placement of building sparkles from their drawing-coordinate positions.
- Ordinary beam release's possible firing-side rearm restart; release alone does not establish it.
- Distance arithmetic for squared separations at or above 2⁵³ square leptons.

## Cross-spec changes required before hand-off

- S-0518 claim 11: cite claim 20 for complete effect selection, including charging above
  parasite attachment and temporal erasure; failed charging does not fall through.
- S-0601 claim 5: retain takeover's existing precedence and point to claim 20 for the
  remaining effects' ordering. No correction to its takeover outcome is needed.
- S-0519 claims 2, 7 and 16: qualify planting, disarming and designation by claim 20's
  selection. Planting outranks charging and erasure; erasure outranks designation and
  disarming; locomotor replacement outranks designation; failed higher effects still
  suppress lower ones. Include direct rocking and disguise-making where applicable.
- S-0614 claims 1 and 3: qualify detonation attachment by claim 20's selection. Takeover,
  planting and charging suppress attachment; attachment suppresses erasure even on failure.
  This does not change the separate firing-time departure from the map.
- S-0215 claim 13: its existing fall-back release answers the idle-event part. Clarify
  that it tests possession of an attached temporal beam, not the currently selected weapon.
  Share its common-effects once-per-tick limit and aircraft early exits. Stop-order target
  clearing and processing stop or sleep alone do not count as fall-back (claim 14).
