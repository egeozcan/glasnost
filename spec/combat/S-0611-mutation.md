---
id: S-0611
title: Mutation
topic: combat
revision: 4
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-20
measurements: []
requests: [Q-1461, Q-1460, Q-1530]
supersedes: []
---

# S-0611: Mutation

## Summary

How an infantry unit killed by a mutating warhead is replaced by a new infantry unit: which deaths
convert and which do not, the conversion animation, and the owner, type, place, state and timing of
the infantry it creates. It covers both deliveries: the genetic mutation superweapon (its launch
and the two ways it reaches infantry) and any weapon whose warhead has the mutating death kind. It
also owns the general rule by which an animation that makes infantry creates it at its end.
Skirmish only; network games and campaigns are out of scope.

**What this spec does not restate.**
- **Amount of a mutating hit** on infantry above the surface: S-0302 claim 17. Ordinary damage,
  defences, credit and killing: S-0302. Area damage: S-0303.
- **Which death case applies** to a killed infantry unit, including the die-2 fallback when the
  conversion does not take place: S-0107 claim 17 (row 7). This spec only says when row 7's
  replacement effect takes place and what it does.
- **The ordinary death hand-off** (shared death effects, passengers, bomb, credit): S-0312 claim 8.
- **Animation playback and lifetime:** S-0109 (claim 16 for the end of an animation).
- **Availability, charge, readiness, refusal and sidebar of the superweapon:** S-0512 claims 1 to 8
  and 16 to 18. **Aiming, the readiness guard at launch, the shared announcement rule, the
  computer-house draw after a fire order and the effect block:** S-0513 claims 1, 3, 5, 6 and 4.
- **Protection:** S-0513 claims 23 and 27 (infantry is never protected).
- **Standing-position choice and entry outcomes:** S-0208 claim 3, S-0209 claims 4 and 5, and
  their placement use in S-0614 claim 20.
- **Copies made by a cloning building** are production (S-0404, S-0417), not mutation.
- **Spawn controllers and slave groups** of a made type: S-0506 claim 1 and S-0402 claim 19.
  This spec only counts and orders their creation draws and numbers within an attempt (claim 14).

## Terms

- **Mutating death kind.** Infantry-death kind 9 (S-0107, Terms). A **mutating warhead** is a
  warhead whose infantry-death kind is 9. S-0302 claim 17 already uses this term.
- **Conversion.** The replacement effect of S-0107 claim 17, row 7: instead of a death sequence the
  victim leaves play at once unless a crash begins (claim 7), and a conversion animation starts at its position in either case. A **victim** is the
  killed infantry unit; the conversion is **refused** when claim 6's test fails, and the death then
  takes row 7's die-2 request.
- **Conversion animation.** The animation type that the rules' general mutation key names (Import
  mapping). Whether it or a follow-up makes infantry depends on the imported animation
  settings and animation-to-infantry list (claims 9 and 10).
- **Infantry-making animation.** An animation whose type has a make-infantry index other than −1
  (the "none" value; S-0109 Import mapping). The index selects an entry of the rules'
  animation-to-infantry list, the **made type**.
- **Genetic mutation superweapon.** The superweapon kind whose launch is claims 2 to 4. Its record,
  charge and readiness are S-0512's; *launch*, *launching house* and *target cell* are S-0513's
  (Terms).
- **Explosion mode, cell mode.** The two ways a genetic mutation launch reaches infantry, chosen by
  one game-wide boolean of the rules (Import mapping): *explosion mode* when it is on, *cell mode*
  when it is off. The **explosion warhead** and the **cell warhead** are the two warheads the rules'
  special-weapons section names for them.
- **Mutation point.** The world point of a launch, F1.
- **Creation set.** The objects one creation of a made unit brings into existence: the unit,
  the initial slot objects of its spawn controller (*spawn controller* and *slot*: S-0506),
  the initial slaves of its slave group (*slave place*: S-0402), and recursively theirs
  (claim 14, F5).
- **Civilian house, effect block, swept layer, occupancy record, attacking house, damage that
  ignores defences:** glossary.

## Behaviour

### Which hits convert

1. **Only a death converts.** A mutating warhead converts an infantry unit only when its hit kills
   it and S-0107 claim 17 reaches row 7 for it, that is, when the death kind *w* is 9 and none of
   that claim's earlier rows applies. So there is no conversion when:
   - the hit does not kill (a surviving target keeps its type and owner but takes the ordinary health change),
   - the death kind is replaced (3 for a unit whose sequence is *parachute*, 5 for a laser-fence
     killer),
   - an earlier row of S-0107 claim 17 applies: the water-splash case, a prone cyborg, a jumpjet
     type, a type with its own death-animation list, or a type marked "not human",
   - the victim is not infantry (vehicles, aircraft and buildings never convert; they take the
     warhead's damage like any other).

   Delivery plays no part beyond this: a superweapon hit (claims 3 and 4) and a weapon shot whose
   warhead has the mutating death kind lead to the same test (claim 6) and the same result
   (claims 7 to 13). The amount a mutating hit delivers to infantry above the surface is S-0302
   claim 17's.
   - Backing: `measurable` (scenarios MU1, MU3 and MU6). The laser-fence killer substitution
     is `unmeasured` here: these scenarios do not isolate a fence-attributed mutating hit.
     It inherits S-0107 claim 17's death-kind rule
   - Confidence: high

### The superweapon

2. **Launch.** A carried-out genetic mutation fire order whose record is ready (S-0513 claim 3)
   does all of the following in the tick of the launch, whether or not any infantry is reached:
   - the animation that the rules' ion-blast key names is created at the mutation point raised by
     5 leptons (F1), as an ordinary animation (S-0109),
   - the genetic-mutation activation announcement plays for every player (S-0701 claim 28, which
     also mutes it after the local player's house is defeated),
   - the activation sound (Import mapping) plays as a positional sound at the mutation point,
   - a radar event of the kind the invulnerability launch uses is raised at the target cell,
   - the effect of claim 3 or claim 4 is delivered,
   - for the launching player, aiming ends and the type's ready announcement is silenced (S-0513
     claim 5).

   The target cell is not checked again (S-0513 claim 3). The superweapon type's range and line
   fields (Import mapping) play no part in the effect. S-0513 claim 6's computer-house draw after
   the fire order is a computer-player rule and is left out in our engine, as that claim says.
   - Backing: `measurable` (scenarios MU1 part a and MU11 for launch feedback, coordinates
     and unused type fields)
   - Confidence: high

3. **Explosion mode.** In explosion mode the launch delivers one area hit (S-0303) centred on the
   mutation point, of amount 10 000 hit points, with the explosion warhead, with no attacker object
   and with the launching house as attacking house. It does not ignore defences. Which objects it
   reaches, its falloff, versus values and credit are S-0303's and S-0302's. An infantry death
   caused by it converts only when claim 1 permits it. Naming a warhead for this delivery does
   not itself give that warhead the mutating death kind.
   - Backing: `measurable` (scenarios MU1, MU2 and MU11)
   - Confidence: high

4. **Cell mode.** In cell mode the launch reaches the infantry units listed in the swept layer of
   each cell of the target cell's effect block (S-0513 F1), with the same layer rule as S-0513
   claim 4: in a high-bridge cell the deck layer only, otherwise the ground layer only, under the
   same switch `quirks.superweapon_block_one_layer`. Each reached infantry unit, of any house,
   receives one hit of its type's whole strength (F2), distance 0 leptons, with the cell warhead, ignoring
   defences, with no attacker object and with the launching house as attacking house. Neither
   delivery sets the no-escape marker (S-0302 claim 1). No other object receives a direct cell-mode
   hit; collateral death effects still follow their own rules. The observable order of hits is
   S-0513 F1's cell order and, within a cell, S-0209 claim 13's listing order.

   Every infantry unit listed in a cell when that cell is reached is hit, even though each
   converted victim leaves play at once: S-0513's `quirks.superweapon_list_cut_by_removal` does
   not apply to this effect. For ordinary infantry whose health does not exceed its type's
   strength, the hit is lethal unless S-0302 claim 8 refuses it. Conversion still requires claim 1, including a mutating
   cell warhead. With such a warhead a reached infantry unit above the surface takes no damage
   (S-0302 claim 17); assigning a different death kind removes that height rule.
   - Backing: `measurable` (scenarios MU4 and MU11)
   - Confidence: high for the recipients, the hit and the absence of a list cut; medium for the
     order within a cell (S-0513's own scenario covers it)

5. **Infantry the effect does not reach.** Infantry that is not listed in any cell is never a
   victim of a launch in either mode: passengers inside a transport, occupants of a garrisoned
   building, infantry held by an absorbing building (S-0605) or already inside a grinder (S-0603).
   These are exclusions from direct hits, not immunity to collateral effects such as the
   destruction of a carrier or building. A unit still on the map on its way to enter one of them
   is listed and is eligible like any target; a successful conversion ends its entry. Infantry
   is never protected (S-0513 claim 23), so protection
   never stops a conversion. A jumpjet type takes S-0107 claim 17's row 2 and a parachutist
   death kind 3, so neither converts. Height alone is not an absolute conversion ban:
   S-0302 claim 17 resets a mutating hit's amount above the surface, but a hit that does not
   ignore defences can still become lethal through the minimum damage. An ordinary human
   type flying without either exclusion can then convert, including claim 7's crash case.
   - Backing: `measurable` (scenarios MU2, MU5 and MU13)
   - Confidence: high for garrison, transport and flight; medium for absorber and grinder
     entrants on their way

### The conversion

6. **Conversion test.** At a victim's death the conversion takes place unless one of these
   outcomes refuses it. The test looks at the victim's cell (the cell of its position) with the
   victim's own entry in the occupancy record already cleared:

   | Case | Conversion | Standing-position draw |
   |---|---|---|
   | The land multiplier of the cell's land type for the *foot* class (S-0211 claim 2) is exactly 0, and the victim is not on a high-bridge deck | refused | none |
   | Otherwise, S-0208 claim 3's position choice, with the victim's position as desired point and the cell's **ground** occupancy record, gives no position | refused | as S-0208 claim 3 gives (one when the point's class is centre and the vehicle and object rows do not apply) |
   | Otherwise, a building is listed in the cell's ground layer | refused | as in the row above: the choice and its draw were made |
   | Otherwise | converts | as in the row above |

   The land multiplier is a single-precision floating-point value, compared exactly with zero
   without integer conversion or rounding. The foot column is used whatever the victim's own
   locomotion class, so a swimming amphibious infantry unit on water with a zero foot multiplier
   is refused when its death reaches row 7. A negative multiplier is not 0.
   The position this test chooses is not kept: the animation starts at the victim's own position
   (claim 8), and the new unit's position is chosen again when it is placed (claim 10). The
   ground record is read even for a victim on a high-bridge deck, so infantry or a vehicle beneath
   the bridge can refuse a deck victim's conversion, and the deck's own record plays no part
   (`quirks.mutation_test_ground_record`, Edge cases).
   - Backing: `measurable` (scenarios MU2 and MU7); the position choice's object-present
     refusal is `unmeasured`, as in S-0208 claim 3: no practical fixture isolates that flag
     from the other refusals
   - Confidence: high for the land, building and position rows; medium for the ground record on a
     deck and for the draw retained when a building refuses

7. **What becomes of the victim.** Unless a crash begins as described below, a converted victim
   leaves play in the tick of its death, with no death sequence and no corpse, after the shared
   death effects and credit of S-0312 claim 8 (so the
   kill is credited to the attacker or attacking house exactly as for any death).
   The crash exception changes only the victim's removal, not its conversion: the conversion
   animation still starts in the death tick, with claim 8's owner, and makes infantry under
   claims 9 to 11, independently of the victim's subsequent crash. A crash begins only when
   the type has the crash flag and the victim's height above its surface is greater than
   0 leptons (S-0312 claim 16); the flag alone on open ground does not prevent removal.
   For a victim that starts a crash, the standing position is marked taken again in the death
   tick, not left free as for a refused conversion. The conversion animation also marks that
   position immediately. Its later marks and freeing follow claim 8: the position is not
   guaranteed to remain taken until the crash ends if the animation ends sooner.
   Crash motion and eventual victim removal belong to the movement and death rules, not to
   the conversion animation. A victim that held control of other units loses it as
   on any death (S-0601), and a controlled victim's control ends with it. A refused conversion
   takes S-0107 claim 17's die-2 request.

   The refused victim's standing position is shown free in the occupancy record from the tick of
   its death, while its die-2 sequence still plays, so another walker may choose it
   (`quirks.refused_mutation_frees_position`, Edge cases).
   - Backing: `measurable` (scenarios MU1, MU7 part c and MU13)
   - Confidence: high for the removal, animation surviving the crash exception, crash-start
     height test and the die-2 request; medium for the occupancy consequences and
     the second freeing at removal (Edge cases)

8. **The conversion animation.** A conversion creates one conversion animation at the victim's
   position (world point, height included) in the tick of the death. Its owner is:
   - the current owner of the attacker object when there is one (for a firer under mind control
     that is the controlling house),
   - otherwise the attacking house,
   - otherwise no house.

   When it has an owner, the animation is drawn in that owner's colour, including after
   claim 9's fallback changes ownership. Without an owner it uses its ordinary animation palette. The original leaves the old colour on an animation waiting to place a unit after
   that change, a display-only defect fixed without a quirk switch. It plays as any animation
   (S-0109), including its start sound. It marks its standing position immediately at
   conversion. Ordinary immediate victim removal clears that mark again, but a victim retained
   by a crash does not perform that removal in the death tick (claim 7).
   While it plays, an infantry-making animation holds its own standing position: in
   each of its updates its position is marked taken in the occupancy record of its surface, and the
   cell's infantry house becomes the animation's owner. For a victim removed immediately,
   between the death and its first update the position is free. At its end the position is shown free again, whatever else has marked it
   meanwhile (`quirks.infantry_animation_occupancy`, Edge cases). The animation has no health and
   is not a target.
   - Backing: `measurable` (scenarios MU1, MU6, MU8, MU9, MU10 and MU13)
   - Confidence: high for the owner rule; medium for the occupancy marks

### Infantry made by an animation

9. **Creation at the end.** The rules in claims 9 to 11 apply to every infantry-making animation,
   the conversion animation included. When its last pass ends and its type names no follow-up type
   (S-0109 claim 16), it tries to create one infantry unit of the made type, then is removed. When
   its type's make-infantry index *i* does not satisfy 0 ≤ *i* < *n* (F4), nothing is created.

   The unit's house is the animation's owner, unless the animation has no owner or its owner is
   defeated by then: the animation itself changes owner to the civilian house and the unit goes
   to that house. If placement then fails, subsequent occupancy marks use this new owner.
   If there is no civilian house, an ownerless animation creates nothing and a defeated owner
   keeps the unit.
   - Backing: `measurable` (scenarios MU1, MU8 and MU10). The no-civilian-house fallback is
     `unmeasured`: no normal-input skirmish fixture without that house is established. Invalid
     indices are a design departure where the original result is undefined
   - Confidence: high

10. **The new unit.** The new unit is a fresh object of the made type: full health (its type's
    strength), starting experience under S-0311 claim 11, no passengers and no inherited order.
    Thus it is a rookie unless the new owner's country infantry-veteran list or infiltrated
    infantry-factory flag grants veteran experience. The victim's experience is irrelevant.
    Nothing of the victim passes to it: not its health, rank, owner, orders, selection or group. It faces
    24 576 facing units (F3, map south-east). Its placement asks for the animation's world point
    under S-0614 claim 20's rules for infantry:
    - when the point is at terrain height inside the usable area, it is the desired point of the
      occupant-aware position choice of S-0208 claim 3 (with its draw for a centre-class point), and the unit
      stands at the position that choice gives, which may differ from the point,
    - when the point is not at terrain height, there is no position choice and the unit stands at
      the point itself; this includes the ordinary deck case,
    - in both cases the entry outcome must be *clear*, judged on the ground record as S-0614
      claim 20 says, so a vehicle beneath a bridge refuses a deck placement (claim 11),
    - when the animation's cell carries a high bridge and the point is above the cell's terrain
      height, the unit stands on the deck.

    A new unit owned by a house that is not human-controlled is given the *hunt* behaviour in the
    original. In our engine this holds for the neutral and special houses only (lead rule on AI
    houses, glossary *AI house*, *Player houses only*). A unit of a player house, human or AI,
    gets no behaviour from its creation: an AI house may order *hunt* through the ordinary order
    rules (documented departure, no switch).
    - Backing: `measurable` (scenarios MU1, MU7, MU8 and MU10); the position choice's
      object-present refusal inherits claim 6's `unmeasured` qualification and reason.
      The departure is a design rule
    - Confidence: high for health, starting experience, facing and the deck; medium for the
      placement rules (from S-0614 claim 20's placement use)

11. **Placement refused.** When the placement of claim 10 fails (for example, a vehicle has entered
    the position's cell), no unit appears and the animation stays: it steps back one frame and tries
    again when that frame is next due (S-0109 timing), and so on until a placement succeeds or the
    animation is removed otherwise. Each attempt takes its own position-choice draw when the
    point is at terrain height inside the usable area, its class is centre and no earlier
    position-choice refusal applies. The owner of claim 9 is resolved again at each attempt.
    In the original each failed attempt also builds a unit that is never placed, together
    with any objects that unit's type creates with it (claim 14); our engine builds none of
    them (documented departure, no switch; Open questions), but retains claim 14's
    scenario-stream consumption and creation-number reservations for all of them.
    - Backing: `measurable` (scenarios MU9 and MU10 for delay and retry; MU12 for creation
      draws and number consumption). Other consequences of retaining never-placed objects
      are `unmeasured`: no isolating fixture for their possible house-count or build-limit
      effects is established (Open questions)
    - Confidence: medium

12. **When the new unit appears.** The new unit appears in the update in which the conversion
    animation ends, with lifetime governed by S-0109 claims 5 and 16, including its first-update
    tick rather than assuming an update in its creation tick. Claim 11 adds delay when placement
    is refused. Until then the attacking side has no new unit. The victim has already left play,
    subject to claim 7's crash exception.
    - Backing: `measurable` (scenario MU1 part b)
    - Confidence: high

### Repeated and overlapping hits

13. **Each victim converts once.** A victim leaves play in its death tick, so a second mutating hit
    in the same or a later tick cannot reach it, and it starts exactly one conversion animation
    (subject to claim 7's crash exception and the absent-animation departure below).
    Two launches whose areas overlap convert each infantry unit at most once. The conversion
    animation is not an object that hits reach, so a launch during it changes nothing about it.
    A unit created by a conversion is an ordinary infantry unit: a later mutating hit converts it
    again under claims 1 to 12 (when S-0107 claim 17 reaches row 7 for its type), to the house of
    that later hit.
    - Backing: `measurable` (scenario MU5 part c)
    - Confidence: high

14. **Draws and numbering of creation attempts (Q-1460).** Every end attempt that has a
    valid made type and a resolved owner (claim 9), successful or refused, takes one raw
    creation draw from the scenario random stream for the made unit (S-0202 claim 10), plus
    the draws of the rest of its creation set below. Its standing-position
    choice uses the state left by that draw: when claim 10's position choice applies, it
    takes one ranged draw over the inclusive integer range 0 to 3 only for a centre-class
    point with no earlier vehicle/object refusal (S-0208 claim 3). Thus a vehicle blocking
    the ground record gives one raw creation draw and no position-choice draw, even at a
    centre-class point (for a made type whose creation set is the unit alone; otherwise the
    whole set's draws replace the single raw value in this paragraph). A centre-class choice with all selectable infantry positions taken
    still takes the ranged draw and then fails. A clear centre-class placement takes the
    raw draw followed by that ranged draw. Side-class and non-terrain-height placements
    take the creation draw but no position-choice draw. None of these draws is undone by
    refusal. These are the creation and placement draws, not a count of unrelated animation
    events or effects elsewhere in the same tick.
    The object-present refusal inherits S-0208 claim 3's `unmeasured` backing: no practical
    fixture isolates that flag from other refusals; MU12 tests the vehicle refusal instead.

    In the original the attempted infantry unit itself also consumes one number from the
    shared object creation counter, even when it is never placed. Refusal neither returns that
    number nor restores the counter. It assigns no control group and advances no automatic
    control-group numbering: new infantry starts ungrouped.

    **Objects created with the made unit.** A made type can name objects that every creation
    of it brings along. An attempt, refused or successful, creates this **creation set**:
    - the made infantry unit itself,
    - when its type names a spawned type and a spawn count *N* ≥ 1, the *N* initial slot
      objects of its spawn controller (S-0506 claim 1),
    - when its type names a slave infantry type and a slave count *M* ≥ 1, the *M* initial
      slaves of its slave group (S-0402 claim 19),
    - in turn, the initial slot objects and slaves of each of those objects' own types, and
      so on.

    Each member takes one raw creation draw and one number from the shared creation counter,
    its number after its own draw, and the created objects start ungrouped. F5 gives the
    order: the made unit's draw and number come first, then its slot objects in slot order,
    then its slaves in place order, and claim 10's position-choice draw, when it applies,
    comes after the whole set. The observable draw and number order is recursive: each slot object's own slot objects and slaves take their draws and numbers before the next slot object's draws and numbers; after all slot objects and their descendants, each slave's own slot objects and slaves take their draws and numbers before the next slave's draws and numbers. All members are
    owned by the house resolved for the attempt (claim 9). A branch contributes no children
    when its type reference is absent or its corresponding count is 0 or less. Only when
    neither branch contributes children is the set the made unit alone. After a successful
    attempt the slot objects and slaves stay off the map as
    S-0506 claim 1 and S-0402 claim 19 describe.
    For bad-data type cycles, F5 overrides the ordinary spawn-controller and slave-group
    creation rules: the cycle-closing object still takes its draw and number but gets neither
    spawn controller nor slave group, so neither can fill it later (Q-1530, design rule).
    Every other controller that a type's settings give an object at its creation takes no
    draw and no number: the takeover controller of a mind-control primary weapon (S-0601),
    the latching controller of a parasite primary weapon (S-0614), the temporal controller of
    a temporal primary weapon (S-0514) and the airstrike designator (S-0519).

    On a refused attempt the whole set stays off the map in the original. None of it ever
    enters play, so none of it, nor the controllers it carries, acts or draws in a later tick.
    No draw or number of the set is undone, and the next attempt creates a fresh set.

    Our engine keeps its own deterministic creation counter rather than matching the
    original's global numbers (the departure also used by S-0306 claim 13). It reserves one
    number per member of the creation set, at each attempt, refused or successful, in the order
    above and without reuse. Avoiding unplaced objects under claim 11 must not undo those
    reservations or any creation draw of the set. When claim 9 creates nothing
    because the index is invalid or no owner can be resolved, no unit-creation draw or
    unit number is consumed in our engine. The invalid-index case is F4's safe design
    departure, not a guarantee about the original's undefined access.
    - Backing: `measurable` (scenario MU12) for original draw composition, creation-set
      draws and numbers, counter advance and absence of a control group; `unmeasured` for no
      resolved owner, because no normal-input skirmish without a civilian house is established
      (claim 9). Our counter reservation without unplaced objects and invalid-index handling
      are design
    - Confidence: high for the made unit's own draw, number and order; high for slot objects
      and slaves taking a draw and a number each after the made unit and before the position
      choice; medium for the depth-first order of nested sets and for the absence of later
      activity of a refused set

## Formulas

- **F1. Mutation point.** For target cell (*x*, *y*):
  *P* = (256*x* + 128, 256*y* + 128, *h*(*c*) + *b*), with *h*(*c*) the terrain surface height
  at the cell centre and *b* the bridge lift of S-0513 F2 (416 on a high-bridge cell, 0
  otherwise). Integer leptons, exact. The launch animation of claim 2 is created at *P* +
  (0, 0, 5) leptons. The area hit of claim 3 is centred on *P* itself.
- **F2. Cell-mode hit.** *d* = *S*, the reached unit's type strength in hit points, exact integer
  assignment with no rounding, delivered at distance 0 leptons, ignoring defences. S-0302 claims
  8, 9 and 17 still govern hit refusals, skipped defences and the mutating-warhead height rule.
- **F3. Facing of a made unit.** 24 576 facing units (96 × 256), exact.
- **F4. Made type.** With *n* the length of the rules' animation-to-infantry list and *i* the
  animation type's make-infantry index (integer), the made type is entry *i* (counting from 0)
  when 0 ≤ *i* < *n*. Otherwise nothing is made. (The original's own bound lets *i* = *n* through
  and reads beyond the list: undefined in the original, nothing in our engine, with an importer
  warning. Negative values other than −1 are treated the same way.)
- **F5. Creation-set sequence (claim 14).** For an object *o* of type *T*, let *S*(*o*) be
  its *N* initial slot objects in slot order when *T* names a spawned type and *N* ≥ 1 (else
  none), and *V*(*o*) its *M* initial slaves in place order when *T* names a slave type and
  *M* ≥ 1 (else none). The creation sequence is
  *C*(*o*) = ⟨raw(*o*), num(*o*)⟩ · *C*(*s*₁) · … · *C*(*s*_N) · *C*(*v*₁) · … · *C*(*v*_M),
  where · is concatenation, raw is one raw scenario-stream value and num one creation number.
  An end attempt with a valid made type and a resolved owner produces *C*(*u*) for the made
  unit *u*, followed by claim 10's position-choice draw when that choice applies. Its totals
  are |*C*(*u*)| raw values and as many numbers, where |*C*(*o*)| counts the pairs: |*C*(*o*)| = 1 + Σ |*C*(*s*ᵢ)| + Σ |*C*(*v*ⱼ)|.
  Integer counts, exact. A chain of spawned and slave types that leads back to a type already
  in that same chain is bad data: the original's result is undefined. Our engine's importer
  warns. Our rule (Q-1530), not original behaviour, is to create the cycle-closing object with
  its one raw creation draw and one creation number, but give it neither a spawn controller
  nor a slave group, regardless of its type's settings. It has no slots and no slave places,
  not empty ones: nothing regenerates or is replaced for it later. For this object both
  *S*(*o*) and *V*(*o*) are empty and *C*(*o*) contains only its own draw-number pair.
  The exception applies to that object, not every object of its type. This makes each creation
  set finite and deterministic and prevents unbounded growth from later fill of a
  cycle-closing object; no separate count clamp is needed for this cycle rule.
  - Backing for the cycle rule: `unmeasured`, design; the original has no defined outcome to
    compare. Engine-only scenario MU14 checks the rule
  - Confidence: high for the design rule

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Mutating death kind | 9 | infantry-death kind | Claim 1; S-0107, S-0302 |
| Explosion-mode amount | 10 000 | hit points | Claim 3 |
| Launch animation lift | 5 | leptons | Claim 2, F1 |
| Made-unit facing | 24 576 | facing units | F3 |
| Cell-mode block | 9 | cells | Claim 4, S-0513 F1 |

## State and ordering

| State | Event | Result |
|---|---|---|
| infantry alive | killed, S-0107 claim 17 reaches row 7, claim 6 test passes | leaves play in the same tick subject to claim 7's crash exception, shared death effects and credit (S-0312 claim 8); conversion animation starts at its position |
| infantry alive | killed, row 7 reached, claim 6 test refuses | die-2 request of S-0107 claim 17; position shown free (claim 7) |
| conversion animation playing | each update | its position marked taken, cell's infantry house set to its owner (claim 8) |
| conversion animation playing | last pass ends, no follow-up type, index valid, owner resolved, placement succeeds | creation-set draws and numbers consumed (claim 14), unit placed; animation removed |
| conversion animation playing | same creation conditions, placement refused | creation-set draws and numbers consumed, but our engine retains none of those objects (claim 14); no unit appears; one frame stepped back; retry when the frame is next due (claim 11) |
| conversion animation playing | same, no owner and no civilian house, or index invalid | animation removed, nothing created |

**Ordering consequences.**
- Every effect of a launch (animation, announcement, sound, radar event, hits) happens in the launch
  tick; every successful conversion removes its victim in that tick, subject to claim 7's crash
  exception, and the new units appear only when
  their animations end, so no unit made by a launch can be hit by the same launch.
- Credit for a victim's death is given in its death tick, before the new unit exists.
- In cell mode every listed infantry unit of the block is hit, in S-0513 F1's cell order, regardless of how
  many of them convert.

## Edge cases and quirks

- **`quirks.mutation_test_ground_record`** (claim 6; default on, the original). On: the position
  choice of the conversion test reads the cell's ground record even for a victim on a high-bridge
  deck, so occupants beneath the bridge can refuse the conversion and the deck's record is not read.
  Off: a victim on a deck is tested against the deck record and the ground record plays no part.
- **`quirks.refused_mutation_frees_position`** (claim 7; default on, the original). On: a refused
  victim's standing position is free from its death tick while it plays die-2, and its removal when
  die-2 completes frees that position again, even if a walker has taken it meanwhile (that walker
  loses its mark). Off: the position stays taken until the victim is removed, as for any other
  death, and the removal frees only the victim's own mark.
- **`quirks.infantry_animation_occupancy`** (claim 8; default on, the original). On: an
  infantry-making animation marks its position taken and sets the cell's infantry house to its own
  owner in every update, and frees the position at its end whatever else holds it, so a walker
  that took the position in the gap after the death loses its mark when the animation ends. Off:
  the animation holds its position only while no other unit holds it, never changes the cell's
  infantry house while other infantry stands in the cell, and at its end frees only its own mark.
- **Cell mode beneath a bridge.** With `quirks.superweapon_block_one_layer` on (S-0513), infantry
  on the ground or water beneath a high bridge in the block is not reached in cell mode, while
  infantry on the deck is. Explosion mode is S-0303's and has no such rule.
- **Absent conversion animation.** Our engine removes an otherwise converted victim with no
  replacement and issues an importer warning. The original does not establish a safe result for
  this missing required type (documented bad-data departure). Backing: `unmeasured`: no safe
  original outcome is established for comparison. Confidence: low for the original outcome.
- **Animation without a make-infantry index.** A conversion animation whose type makes no infantry
  plays and leaves nothing. A follow-up type (S-0109) is followed first; only the last type's index
  counts.
- **Hunt for computer owners.** Claim 10's departure: kept for the neutral and special houses,
  dropped for player houses.
- **Never-placed objects.** Claim 11's departure.

## Import mapping

| Source | Key | Concept | Parse | Default when absent |
|---|---|---|---|---|
| Rules, `[General]` | `MutateExplosion` | explosion mode (on) or cell mode (off), claims 3 and 4 | boolean (S-0003) | no (cell mode) |
| Rules, `[SpecialWeapons]` | `MutateExplosionWarhead` | explosion warhead, claim 3 | warhead reference (S-0004) | none (see below) |
| Rules, `[SpecialWeapons]` | `MutateWarhead` | cell warhead, claim 4 | warhead reference (S-0004) | none (see below) |
| Rules, `[General]` | `InfantryMutate` | conversion animation type, claims 8 to 12 | animation reference (S-0004) | none: our engine leaves no animation or unit and warns (departure above) |
| Rules, `[General]` | `AnimToInfantry` | animation-to-infantry list, F4 | infantry type list (S-0004) | empty: no animation makes infantry |
| Art, animation section | `MakeInfantry` | make-infantry index *i*, F4 (S-0109 maps the key) | integer | −1 (none) |
| Rules, `[General]` | `IonBlast` | launch animation of claim 2 (shared with other users of the key) | animation reference (S-0004) | none: no launch animation |
| Rules, `[AudioVisual]` | `GeneticMutatorActivateSound` | activation sound of claim 2 | sound name | none: no sound |
| Rules, infantry type section | `Crashable` | permits the victim's crash instead of immediate removal, but never suppresses its conversion animation (claim 7) | boolean (S-0003) | false at type creation; omitted layered values retain the value in force |
| Rules, warhead section | `InfDeath` | the mutating death kind is the value 9 (S-0107 maps the key) | integer | 0 |
| Rules, superweapon section | `Type` | the genetic mutation superweapon is S-0512's genetic-conversion kind (S-0512 maps the key and its kind words) | name | as S-0512 |
| Rules, superweapon section | `Range`, `LineMultiplier` | not read by the effect (claim 2) | — | — |
| Rules, made infantry type section (and each created object's type) | `Spawns`, `SpawnsNumber` | spawned type and spawn count *N* of F5; slot objects join the creation set of claim 14 (S-0506 maps the keys) | as S-0506 | as S-0506 (none, 0: no slot objects) |
| Rules, made infantry type section (and each created object's type) | `Enslaves`, `SlavesNumber` | slave type and slave count *M* of F5; slaves join the creation set of claim 14 (S-0402 maps the keys) | as S-0402 | as S-0402 (none, 0: no slaves) |
| Rules, weapon's warhead section and type section | `MindControl`, `Parasite`, `Temporal` (primary weapon's warhead), `AirstrikeTeam` | controllers made at creation that take no draw and no number (claim 14); S-0601, S-0614, S-0514 and S-0519 map the keys | boolean, integer | as their owners |

- A warhead reference left empty: what the original does with a missing explosion or cell warhead
  is not established (Open questions). Our engine delivers no hits for that mode and the importer
  warns once.
- An index *i* with *i* ≥ *n*, or negative other than −1: nothing is made and the importer warns
  (F4). Claims 11 and 14's retry draws and number reservations use the made type selected by
  the existing animation/list mapping, and the spawn and slave settings of that type (and of
  the types it creates) in the rows above set the size of each attempt's creation set (F5).
- Cycles formed by `Spawns` and `Enslaves` with positive corresponding counts are bad data.
  The importer warns; F5 suppresses both the spawn controller and slave group only on each
  cycle-closing object, without changing the imported type references or counts. The closing
  object's creation draw and number are retained. This is our design rule, not an absent-key
  default or original behaviour.
- The civilian house is found by the side named Civilian, an engine constant (S-0414 Import
  mapping), not stock data.
- Announcement and radar-event bindings are S-0701's and S-0410's.

## Measurement scenarios

All scenarios use stock data with unrelated mods and autonomous gameplay intervention disabled,
normal game speed, a small purpose-made skirmish map, and a recorder of health, owner, type,
position and occupancy for listed units and of the scenario random stream. Scripted measurement
input and read-only recording are permitted. One bystander computer player is kept outside the observed area under the
glossary's void rule. Run each deterministic sub-test once with the seed
and stream recorded; repeat a centre-class test over 16 runs to distinguish draw-dependent choices.
Named map overrides below are test data, removed after each run; MU10 also names a temporary art
test file. The mode is set explicitly in each scenario by a rules
override in the test map (the general mode key on or off), so no scenario depends on the stock
value. "The made type" means the type the animation-to-infantry entry of the conversion animation's
index names.

- **MU1 (basic conversion, claims 1, 2, 7, 8, 9, 10, 12).** House A owns a ready genetic mutation
  superweapon; house B has three human-type infantry of a type that S-0107 claim 17 sends to row 7,
  standing at side positions of the target cell and two of its neighbours, plus one vehicle. Run once per mode,
  targeting the middle infantry's cell.
  (a) Record the launch tick: launch animation at the target point 5 leptons up, activation
  announcement, positional sound, radar event. (b) Record for each infantry: death tick, no corpse,
  removal in the death tick, a conversion animation at its exact position drawn in A's colour, and
  a unit of the made type owned by A appearing at that position facing 24 576, at full health and
  the starting rank of S-0311 claim 11, when the animation ends (S-0109 claim 16 lifetime).
  Repeat with a wounded elite victim selected and assigned to a group, and with the made type in
  A's country infantry-veteran list, then with A's infiltrated-infantry-factory flag set and a
  trainable made type. These named map overrides yield veteran new units, not the victim's elite
  rank. The new unit is unselected, ungrouped and has no inherited destination or passengers.
  Record health before death: earlier surviving hits reduce it without replacing it.
  The vehicle is unharmed if
  the warhead's versus value for its armour is 0. Kill counts credit A.
  *Falsified if* a corpse appears after a successful conversion, the unit belongs to B or the
  neutral house, appears at a different time, inherits the victim's state, or fails the new
  owner's starting-experience rules. Missing launch feedback, a wrong animation point or
  colour, wrong health or facing, or missing kill credit also falsifies the corresponding rule.
- **MU2 (refusals and flight, claims 5 and 6).** Explosion mode. (a) An amphibious infantry unit
  swimming on water: no new unit; the death shown is S-0107 claim 17's (splash or wet die-2). (b) Infantry in a cell where
  a building is listed (an open gate cell): die-2, no new unit. (c) A rocketeer in flight and a
  paratrooper under its parachute in the area: no new unit (S-0302 D14 records the damage).
  (d) Infantry garrisoned in a civilian building and passengers of a transport inside the area:
  unchanged. *Falsified if* any of these produces a new unit.
- **MU3 (non-lethal hit, claim 1).** Map override giving a rifle infantry's weapon warhead
  infantry-death kind 9 and a low damage. Fire at a full-health enemy infantry: it survives with
  its type and owner; only the shot that kills it converts it. *Falsified if* a surviving target
  changes. Repeat lethal shots against a prone cyborg, a type with a personal death-animation
  list and a type marked not human, using named map overrides where required to isolate each
  property. None converts, even with the mutating death kind. A conversion in any excluded case
  falsifies claim 1; compare the shown death with S-0107 claim 17. Also kill an aircraft and a
  building with that warhead, using named test damage sufficient for each: neither creates a
  conversion animation or made infantry. Either replacement falsifies the non-infantry exclusion.
- **MU4 (cell mode, claim 4).** Cell mode. Three enemy infantry in the target cell and one in each of
  two neighbouring cells, one infantry two cells away, and one vehicle in the block. All five block
  infantry convert in the launch tick (the three in one cell show the absence of a list cut); the
  outer one and the vehicle are untouched. Repeat with the target on a high-bridge cell, with
  infantry on the deck and beneath it: deck infantry converts, infantry beneath does not (switch
  on). Include an allied infantry unit and one of the launching house; they are hit too.
  Record the scenario stream and hit/death records to compare with S-0513 F1's cell order and
  S-0209 claim 13's within-cell order. Use centre-class victims with no early position-choice
  refusal so draw assignments distinguish the order. Repeat with named map overrides replacing
  the cell warhead by a non-mutating warhead: ordinary deaths replace conversions and the special
  height rule no longer applies. *Falsified if* any eligible block infantry survives, an outer
  unit or vehicle receives a direct hit, ground infantry beneath the bridge is hit, or the
  hit/draw order differs.
- **MU5 (entrants, repeats and overlap, claims 5 and 13).** (a) An infantry unit walking to a
  grinder and one walking to an absorbing building, inside the area: both convert, neither enters.
  (b) Units already inside: unchanged. (c) Two launches by two houses at the same cell in one tick,
  and a second launch while the conversion animations play: each victim converts once; the second
  launch then converts the new units to its own house after they appear. *Falsified if* an
  entrant on its way survives unconverted or enters, a unit inside changes, or any victim leaves
  two animations or two units.
- **MU6 (weapon delivery and owner, claims 1 and 8).** Map override giving a vehicle weapon's
  warhead infantry-death kind 9. (a) The firer of house A kills a B infantry: new unit owned by A.
  (b) The firer is mind-controlled by house C: new unit owned by C. (c) The firer dies before the
  animation ends: new unit still owned by A. *Falsified if* (a) or (c) gives a unit not owned
  by A, or (b) gives a unit owned by A or B. (d) Kill a mind-controlled victim, then a victim
  holding control of another unit: the victim's old control links end, and the made infantry
  inherits none. A remaining link falsifies claim 7. (e) Kill the firer while its projectile
  is still flying: when S-0302 claim 1 leaves neither attacker nor attacking house, the conversion
  animation starts ownerless with the ordinary palette and its made infantry goes to the civilian
  house. Ownership by the old firer house falsifies claim 8.
- **MU7 (test details, claims 6, 7, 10).** (a) A victim moving through a cell centre (centre class)
  when killed: one ranged draw taken by the test, visible in the recorded stream; a victim at a side
  position: no draw. (b) A victim on a high-bridge deck with a vehicle standing beneath the bridge
  in the same cell: refused, die-2 (switch on). (c) A refused victim's position read from the
  occupancy record during its die-2 sequence: free. (d) The new unit of a computer player house:
  in the original it starts hunting; record its behaviour. A different initial behaviour
  falsifies this original-only rule. *Falsified if* a side-position
  victim consumes a draw, a centre-class victim does not, (b) produces a new unit, or (c) shows
  the position taken. Also let an allied walker take the refused victim's freed position while
  die-2 still plays in a traversable gate cell. Its mark must be cleared again when the victim
  is removed; retention falsifies the second-freeing quirk. Use an open-gate centre-class victim
  to check that building refusal retains the draw when position choice otherwise succeeds.
  Named map overrides setting the foot land multiplier to zero, then a negative value on
  otherwise equivalent dry cells must refuse only zero at the land test. An early refusal for
  the negative value falsifies claim 6. Place three infantry at all selectable side positions
  while a centre-class victim crosses the cell: no free selectable position refuses conversion
  even when the top position is free; conversion in that recorded configuration falsifies it.
- **MU8 (owner at the end, claims 8, 9).** Three houses: A, C allied with A, and B, so that two
  alliances remain when A is defeated (S-0414 claim 6). A owns the ready superweapon building and a
  power plant only. A launches at B's infantry, and a map trigger destroys all of A's objects in
  the launch tick. S-0414 decides A's defeat tick; record it. Expected: when A is defeated before
  the animations end, the new units belong to the civilian (neutral) house; when not, to A. Also
  record the occupancy record of each victim's position during the animation (taken, infantry
  house A from the animation's first update). *Falsified if* a unit appearing after A's defeat
  belongs to A, C or B, if the position is not marked during animation updates, or if its infantry
  house is not the animation's owner. For the occupancy quirk, use scripted movement and the
  recorder to find a run where a walker marks that same position after the death and before the
  animation's first update. Its mark must be cleared at the animation's end; retention falsifies
  the quirk.
- **MU9 (placement refused, claim 11).** After a conversion, drive a vehicle into the victim's cell
  before the animation ends and hold it there for a known number of ticks: no unit appears while
  it stays; the unit appears at the next due retry after departure, within one animation frame
  delay, not necessarily a full delay after the vehicle leaves. Repeat on a
  high-bridge deck with the vehicle beneath the bridge. *Falsified if* the unit appears while the
  vehicle is still in the cell, or the animation is removed without a unit. Repeat with MU8's
  defeat setup while placement is blocked: after the first refused end attempt following defeat,
  the animation belongs to the civilian house and its later occupancy marks name that house.
  The original's colour remains A's; our engine corrects this display-only defect to the civilian
  colour. A different owner, different occupancy house or renewed A-owned placement falsifies
  claim 9.

- **MU10 (generic infantry creation, claims 8 to 11, F4).** Use a temporary named art test file
  that gives a non-conversion animation a valid infantry-making index, then one with no index,
  and a pair of follow-up types with distinct valid indices. Start the animation by a map trigger
  at a clear side position. Record owner, frame advances, occupancy and unit counts. An ownerless
  animation creates for the civilian house. No index creates nothing. A follow-up suppresses
  creation by the preceding type; only the terminal type makes its selected infantry. Repeat
  valid indices selecting two distinct types from a named map override of the list. A different
  type, premature creation, more than one unit or a non-civilian owner falsifies the corresponding
  rule. Compare centre-class and deck placements with MU7 and MU9: only terrain-height eligible
  centre choices consume a position draw. Invalid-index behaviour is not an oracle comparison:
  our engine must create nothing and warn, rather than reproduce an undefined result.
- **MU11 (delivery and launch constants, claims 2 to 4).** Named map overrides give an ordinary
  human infantry type strength 30 000 hit points, no damage immunity and no death weapon, and
  give the explosion warhead a versus factor of 1 for that type's armour. Place a full-health
  unit at the target centre. Explosion mode leaves 20 000 health when all other damage factors
  are 1 and centre falloff is 1; cell mode delivers the whole type strength and kills it. These
  are invented test values, not stock values. A different health delta falsifies the amount.
  Repeat the launch on elevated terrain and on a high bridge: record the effect centre and
  animation point against F1. Vary the superweapon type's range and line fields by named map
  overrides; recipients and amount must not change. Record aiming and ready-announcement state
  for a local launch, and sound/radar output with no victims present. A changed effect, missing
  launch feedback or a launch animation anywhere other than F1 falsifies claim 2.

- **MU12 (attempt draws and numbering, claims 11 and 14).** Use MU9's setup with the
  autonomous gameplay intervention disabled and the read-only random-state recorder. Record
  each terminal attempt's
  incoming/outgoing scenario state, made-unit creation number and group, and whether a unit
  enters play. Use MU10's trigger-created infantry-making animation at a centre-class point,
  with a named test art file removing random-rate and start/middle-event effects. Run these variants once each, recording the
  seed: vehicle blocking the cell; all selectable infantry positions occupied without a
  vehicle/object refusal; clear centre-class point; clear side-class point; and a point
  above terrain height. Observe at least three refused retries, then clear the blocker and
  observe success. Expected per attempt: vehicle blocker, one raw value only; infantry
  crowding, one raw value followed by one ranged choice over 0 to 3; clear centre, the same
  draw composition with successful placement; side or raised point, one raw value only.
  Use S-0202 claims 4 and 5 to predict the ranged result from the state after the raw value,
  and the corresponding position under S-0208 claim 3. Repeat the clear-centre variant over
  16 recorded seeds to distinguish reversed draw allocation. No creation draw, reversed
  allocation, or an extra position draw under the vehicle blocker falsifies claim 14.
  Record the original's creation counter immediately before and after each refused attempt:
  it increases by one, with no rollback. Create a later factory unit and account separately
  for intervening creations: its number includes the consumed attempt numbers. Any rollback
  falsifies the numbering rule. Each attempted infantry object has no control group.
  A recorded group assignment falsifies that rule.
  Creation-set variants (claim 14, F5): named map overrides give the made type invented
  settings: a spawned type with count *k* = 2; a slave type with count *m* = 3; both
  together; a spawned type whose own type names a slave type with count 1; and, separately,
  a mind-control, parasite or temporal primary weapon only, and a type with only an
  airstrike designator. Test each controller-only variant separately. Block placement with
  the vehicle and
  record at least two refused attempts, then a successful one. Expected per refused attempt:
  1 + *k* + *m* raw values (with nesting, 1 + *k*(1 + 1) for the nested variant) before any
  position-choice draw, and the counter advancing by the same amount; the made unit's
  number is the lowest, then the slot objects' numbers, then the slaves'. In the nested
  variant each slot object's slave number directly follows that slot object's number. The
  controller-only variants add nothing to the ordinary totals. Between attempts no further draw
  is attributable to the refused sets. A successful attempt gives the same composition,
  followed by the position-choice draw. A different count, any draw of the set after the
  position choice, a slave number before a slot number of the same set, or an extra draw
  for any controller-only variant falsifies the creation-set rule.
  Engine-only check: refused attempts create no retained object but still consume every
  draw of their creation set and one deterministic creation number per member; none is
  returned for later reuse.
- **MU13 (crash-flag conversion, claims 7 and 8; Q-1461).** Normal-speed skirmish with
  autonomous gameplay intervention disabled, house A's mutating weapon and house B's ordinary
  human infantry type with
  no personal death-animation list, no cyborg or jumpjet type flag, and a named map override
  enabling its crash flag. On dry open ground, kill it at a side position. Record death tick,
  animation creation, owner, occupancy marks, victim removal and made-unit appearance.
  Expected: no crash at surface height, immediate victim removal and one A-owned conversion
  animation, later making A's infantry under claims 9 to 11. Repeat with positive recorded
  height above the surface, not merely above terrain on a bridge: a named map override gives
  the same non-jumpjet type jumpjet movement, allowing scripted flight. Accept the fixture
  only when the recorded death case reaches row 7 of S-0107 claim 17, rather than a parachute
  or jumpjet-type case. Give the airborne victim 1 remaining health and use a non-ignoring
  mutating hit whose minimum damage is lethal (S-0302 claims 5 and 17); do not use cell-mode
  delivery, which does no damage there. Expected: the victim starts a crash, yet the
  conversion animation starts at its death position in that same tick and follows ordinary
  ownership and end-placement rules. Its standing position is marked taken on entry to
  the crash, not left free by conversion. Record subsequent animation marks and freeing
  separately from victim removal; do not assume that their end ticks coincide. Repeat with
  A defeated before animation expiry under MU8's setup: the made unit goes to the civilian
  house, even though the victim crashed. Run once per variant with the seed recorded.
  Missing conversion animation, suppressed creation solely because the victim crashed,
  wrong owner, or a position left unmarked at crash entry falsifies the respective rule.

- **MU14 (cycle termination, claim 14 and F5; Q-1530; engine only).** Use a small normal-speed
  skirmish with autonomous gameplay intervention and unrelated mods disabled except named
  test data. Give an infantry-making
  animation a made type whose spawned/slave descendants contain a repeated type along one
  chain, using positive invented counts. Test a spawn-only cycle, a slave-only cycle and a
  mixed cycle. In the mixed case give the repeated type both spawned and slave settings. Trigger one
  successful creation at a clear side position and record each object's type, creation draw,
  number, spawn controller, slave group, slots and slave places. Expect an importer warning and a finite
  creation set in F5 order: each cycle-closing object exists, takes its own draw and number,
  but has neither spawn controller nor slave group, zero slots and zero slave places. Keep it alive past
  several configured regeneration/replacement delays, destroying other descendants where
  necessary to exercise normal replacement. No later fill may be attributed to that closing
  object. Repeat from the same recorded seed: counts, draws and numbers must match. Any
  missing closing object, lost draw or number, retained spawn controller or slave group,
  later fill from that object, nontermination or differing replay falsifies our rule.
  Also test an acyclic chain whose siblings share a descendant type: each sibling's
  descendant must retain its ordinary controllers and children, because a repeated type
  outside its own ancestry is not a cycle. Suppression there falsifies the per-chain rule.
  Repeat the cyclic variants with a vehicle blocking placement for three attempts, then
  clear it. Each refusal must reserve the same finite F5 sequence without retaining any
  attempted objects, and success must consume a fresh sequence. Reused numbers, missing
  draws or objects retained after refusal falsify claims 11 and 14's engine rule.
  No original-game comparison is proposed for undefined cyclic data.

## Open questions

The following unresolved original outcomes are `unmeasured` (low confidence): no safe missing-type
fixture or observable consequence of an unplaced unit has been established, and no stock death
effect fixture is known for the sweep interruption. They do not override the defined safe results.

1. What the original does when the explosion or cell warhead is missing (no stock data reaches it).
2. Whether the unit the original builds on each refused placement attempt (claim 11), and the
   slot objects and slaves of its creation set (claim 14), affect anything observable, such as
   house unit counts or build limits. Our engine builds none of them.
3. Whether a death effect of a cell-mode victim that removes the next listed infantry unit of the
   same cell can stop the sweep of that cell (no stock case known).
