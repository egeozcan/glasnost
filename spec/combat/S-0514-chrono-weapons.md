---
id: S-0514
title: Chrono weapons (temporal erasure)
topic: combat
revision: 3
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
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
  detonation effect selected. Other special-effect flags can take precedence.
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
- **Removed without death:** removal without the target's normal death animation, death weapon,
  debris, rubble, wreck or survivors. Destruction bookkeeping can still occur (claim 13).

## Behaviour

### Locks and their immediate effects

1. **Delivery.** A temporal detonation needs both its firer and its target still present.
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

4. **A successful lock.** With no existing beam, the firer becomes leader and sets *R* by
   F1. Otherwise it joins without resetting *R*. The target becomes being erased, is
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
    - Backing: `measurable` (E8).
    - Confidence: high.

### Completion and release causes

12. **Removal.** Completion plays the imported warp-away animation at the target and
    removes the target without death, with claim 13's credit. Building garrison occupants
    are removed as S-0508 claim 15 specifies; absorbed units are deleted without individual
    destruction credit. A bunker releases its vehicle; removing a bunkered vehicle clears
    its bunker. A slave master's slaves are freed. Transport passengers are recursively
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
    - Backing: `measurable` (E6, E9 parts f and g). Exhaustive score/counter equivalence is
      `unmeasured`: these scenarios do not isolate every destruction-bookkeeping consumer.
    - Confidence: high (gains, cost bases); medium (bookkeeping equivalence).

14. **Release causes.** An attached attacker lets go on arriving in a new cell, entering
    idle behaviour, delivering another erasing hit even at the same target, being taken
    under mind control or freed, being locked by another temporal beam, or being removed.
    Gunner exit follows S-0308 claim 6; claim 11 adds the leader's range release. Arrival
    by teleport follows the new-cell rule. Removing the target invalidates all attached
    beams and makes their attackers enter idle behaviour.

    Death alone is not established as a separate release: a destroyed attacker retained
    in play may continue erasing until removal, but this is a hypothesis, not a guaranteed
    lifetime rule.
    - Backing: `measurable` (E10) for ordinary cell arrival, idle, mind control, counter-lock,
      removal and retained-death tests. Teleport arrival and gunner exit are `unmeasured`
      here: their complete movement/transfer paths have not been independently retraced.
      The owners above retain their own backings.
    - Confidence: high (ordinary release paths), medium (teleport, gunner exit),
      low (continuation after death).

15. **House destruction.** S-0414 claim 5 owns surrender. A trigger that destroys all
    objects of a house detaches the beams on each erased victim and idles their attackers
    before the destructive hit. It does not hand leadership on or switch buildings back on.
    - Backing: `measurable` (E10 part g).
    - Confidence: high.

16. **Owner change.** An erased target changing owner is not exempt from ordinary attacker
    target loss. Clearing that ordinary target alone does not detach the beam. Subsequent
    behaviour can still cause claim 14's release. Whether it does so immediately is open.
    An erased target normally cannot make its own teleporting locomotor jump because its
    destination is cleared (S-0207 claim 20).
    - Backing: `measurable` (E10 part h) for target clearing without direct detachment.
      Subsequent behaviour-driven release is `unmeasured`: that transition was not traced,
      so no deadline is claimed.
    - Confidence: medium (clearing itself); low (later continuation).

### While attached

17. **Target inactivity.** Infantry, vehicles and landed aircraft being erased clear their
    target and destination and skip ordinary movement, turning and firing. Buildings skip
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
      predicts that transformation independently.
    - Confidence: high (cadence, mobile placement); medium (building placement).

19. **Attacker firing.** Holding the same target makes the firing check answer rearming
    (S-0301 claim 9), so a stationary attacker does not fire repeated erasing shots at it.
    A counter-lock releases its existing beam and prevents a new lock while attached.
    Vehicle moving-fire restrictions remain S-0310 claim 7's responsibility.
    - Backing: `measurable` (E10 parts d and f).
    - Confidence: high.

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

**F4, distance:** *Δx*, *Δy*, *Δz* are signed integer coordinate differences in leptons.

> *q* = RZ₅₃(RZ₅₃(RZ₅₃(*Δx*²) + RZ₅₃(*Δy*²)) + RZ₅₃(*Δz*²))
>
> dist = trunc(RZ₅₃(√*q*))
>
> Release when dist > 256 × *W*.

Differences are converted to floating point before squaring. Multiplications, additions and
the square root use double-significand floating point; RZ₅₃ rounds toward zero as S-0205
claim 5 specifies. The final conversion truncates toward zero to integer leptons. The warp
distance *W* is integer cells, and its threshold uses signed 32-bit integer multiplication.

**F5, direct gain:** *e* is single-precision experience, *A*ₜ and *A*ₐ are integer actual
costs in credits as S-0311 F1 defines, for target and attacker houses respectively. Veteran
ratio ρ and cap *c* are double precision.

> *s* = RZ₅₃(*e* + RZ₅₃(*A*ₜ / RZ₅₃(*A*ₐ × ρ)))
>
> *e*′ = RZ₂₄(*c*) when *c* ≤ *s* or *s* is not a number, otherwise RZ₂₄(*s*).

RZ₂₄ narrows to single precision toward zero. The cap comparison uses *s* before narrowing.
This uses S-0311 F3's arithmetic but not its kill value or recipient-cost house. The normal
credit then uses S-0311 claims 7 to 10 independently. For F4 and F5,
`quirks.chop_rounding` off selects S-0205 claim 5's round-to-nearest alternative.

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
| no beam | delivered hit | Side effects even on refusal; otherwise first lock |
| attached | another lock | Join without resetting progress |
| attached | erasure update | Range release, or reduction and possible completion |
| attached | leader releases with others left | Newest remaining joiner leads with unchanged amount |
| attached | last release | Progress lost; target resumes; building switched on |
| attached | completion | Target removed, credit applied, attackers idle |
| attached | other target removal | Attachments invalidated, attackers idle |

A refused lock can free controlled units without erasing their controller. Two temporal firers
whose pending shots reach each other cannot establish reciprocal locks: the first successful
lock releases its victim's beam and causes that victim's later lock attempt to be refused.
The direct experience gain can remain on a passenger while ordinary credit goes to its transport, or exist when ordinary allied/unscored credit gives none. The direct gain precedes ordinary kill experience, and each gain is capped separately. Both gains are available by the target's removal.

## Edge cases and quirks

- `quirks.erasure_refused_hit_side_effects`, default original: claim 2's target side effects
  survive refusal. Off: refused locks do not clear spawners or free controlled units.
  the firer's previous beam still lets go.
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
| `Temporal`, warhead | Temporal effect, beam supplied by primary warhead | boolean | no |
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

Use the glossary measurement setup: stock executable and data, director/mods/add-ons off,
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
  (b) Join two/three attackers, including elite and firepower-bonus variants: expect raw
  chosen-weapon sums, not multiplied damage. Named zero/negative-damage variants test no
  reduction/increasing amount; a read-only amount record tests signed wrap separately.
  (c) In separate runs make the local house own the harvester or building target; clear
  announcement cooldowns. Expect ordinary eligible notices on first lock, none on joining.
  Repeat with named 1 × 1 test building art, once with and once without a pack-up vehicle.
  Expect no base notice only in the packing variant; a notice there falsifies the exclusion.
  Other first-lock silence is inconclusive until normal notice suppression is excluded.
- **E2, health (claim 6).** Repeat E1 after damaging the victim below half health.
  Expect identical duration; a shorter duration falsifies the type-strength rule.
- **E3, joins/releases (claims 4, 9, 10).** A locks, B joins later, then A stops.
  Record amount and completion date: expect unchanged amount at join and leadership transfer,
  with only the subsequent damage sum changed. Stop B too and relock: expect a fresh full
  amount. A reset on joining/handover or retained progress after last release falsifies it.
- **E4, gates (claims 1, 3).** Compare protected and unprotected targets, a named not-erasable
  type, factory-contact vehicles before and after leaving, landed and airborne aircraft,
  jumpjet infantry before take-off, during ascent and after landing, and ground force-fire.
  Record each object's render layer at impact as well as height and jumpjet flight state.
  Expect refusal in the listed protected cases, no effect on targets outside the ground
  render layer, and ground force-fire only releasing the old lock without damage. An
  otherwise eligible ground-layer jumpjet locks regardless of its flight-state label.
  a lock outside that layer or refusal caused only by flight state falsifies the layer gate.
  Use a named slow temporal projectile already in flight when its firer becomes
  locked: expect refusal when it arrives. A lock in a refused case falsifies that gate.
- **E5, refused side effects (claim 2).** Hit a protected mind controller holding two units
  and a protected carrier with aircraft out. Expect control freed/spawner slots cleared
  without a lock. Repeat with a firer already erasing another victim: that old beam releases.
  Retained control/slots or attachment falsifies the corresponding claim.
- **E6, experience (claim 13, F5).** Named test data sets actual attacker/rookie victim costs
  to 1000/500 credits, ratio 4 and cap 2.5. Expect direct 0.125 plus ordinary 0.125 per erasure,
  veteran after four and elite after eight. Repeat with differing house cost factors, allied
  and unscored victims, and a leader passenger in a trainable open-topped transport.
  Expect F5 to use the leader's own-house cost, survive allied/unscored exclusions and stay
  on the passenger while ordinary credit goes to the transport. A missing or redirected
  direct gain, wrong rounding or wrong cost factor falsifies these parts.
- **E7, building power state (claims 4, 10).** Lock a locally owned power-draining building
  switched off before the hit. Record switch, house totals and radar through release.
  Expect no output/drain while locked and switch on after last release, with totals updated
  in the next eligible house update. A restored off state falsifies the original quirk.
- **E8, range (claim 11, F4).** Named warp distance 3 cells. A passenger leads, then its
  transport drives away: expect release at the first target update beyond 768 truncated
  3-D leptons and no reduction that update. Repeat with an on-foot leader and passenger
  joiner: expect no passenger release. Stop the leader so the passenger inherits leadership
  beyond the limit: its next check releases. Include a height difference and exact-boundary
  positions. Release at equality or checking the joiner falsifies the corresponding rule.
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
  change the erased victim's owner. Expect releases in a to d, no further shots in f,
  detached/idle attackers before g's destructive hit, and ordinary target clearing without
  detachment by that clearing alone in h. Record later behaviour-driven release without a
  predicted deadline; refused takeover is inconclusive. In e, continuation until removal
  is the low-confidence hypothesis; earlier release falsifies it. Also remove a locked
  target by a trigger and verify all attackers idle. Missing release/idle falsifies that part.
- **E11, target and presentation (claims 4, 17, 18).** (a) Lock moving/firing infantry,
  vehicles, landed aircraft and armed buildings: expect ordinary movement/turn/fire to stop.
  (b) Begin long locks at three global tick residues. Expect mobile sparkle creation on
  global multiples of 24, with claim 18's mobile displacement. Named building art with two
  distinct fire positions gives residues 0 and 23; no positions gives residue 0. A different
  cadence/phase or mobile position falsifies that part. Building coordinate conversion is
  recorded without a predicted result. (c) Select an own target before lock: expect
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

## Open questions

- Later behaviour-driven detachment after an erased victim changes owner.
- Whether death paths always release before removal, despite no release in beam cleanup itself.
- Exact world placement of building sparkles from their drawing-coordinate positions.
- Ordinary beam release's possible firing-side rearm restart; release alone does not establish it.
