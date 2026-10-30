---
id: S-0612
title: Psychic Dominator superweapon
topic: superweapons
revision: 4
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-30
measurements: []
requests: []
supersedes: []
---

# S-0612: Psychic Dominator superweapon

## Summary

The psychic domination superweapon: what its launch does and when, the game-wide domination it
starts and the phases it goes through, the area damage and the takeover sweep that happen at one
moment of it, which objects the sweep takes over and how, who owns the result, and how a domination
coexists with other effects. These claims cover skirmish only; network games and campaigns are out
of scope.

**What this spec does not restate.**
- **Charging, readiness, granting, fire orders and the refusal of an order while a domination is in
  force:** S-0512 claims 1 to 8 and 16 to 18. Claim 3 below only defines when a domination is in
  force, which S-0512 claim 8 reads.
- **Aiming, the ready-record guard, the shared launch announcements and the absence of game-rule
  draws after a launch:** S-0513 claims 1 to 3, 5 and 6. What protection refuses: S-0513 claim 27.
- **Area damage itself** (scanned cells, spread, warhead factors, credit, friendly damage): S-0303
  and S-0302. Claim 9 below gives only the call: point, amount, warhead and houses.
- **Lighting:** the dominator lighting set, the ranking of global effects, the fade and the
  re-evaluations are S-0104 (claims 1 to 5, 10, 13 and 16). This spec gives the moments that
  S-0104's dominator phases start and end at (claims 3 to 5).
- **Announcements and sounds:** the announcement rules and mute after defeat are S-0701 claims 28
  and 36.
- **The permanently controlled state** (what it means for every consumer, the release by a
  temporary controller first, the permanent ring and its height): S-0601 claims 9, 13 and 18.
- **Psionics immunity:** S-0610 claim 1.
- **What a direct owner change does to a vehicle in a bunker:** S-0606 claim 15.

## Terms

- **Domination.** The game-wide state that a launch of this superweapon starts (claim 1). There is
  at most one at a time, for all houses together.
- **Domination in force.** From the launch that starts a domination until it is over (claim 5).
  This is the same span as S-0104's dominator being *in force*.
- **Dominating house.** The house whose launch started the domination, fixed at launch (claim 17).
- **Domination update.** The once-per-tick step of a domination in force. It runs in S-0203's row 3
  (the weather storm's update), before the storm's own update of that tick. Row 3 comes before the
  chrono flash step (row 4) and the turn pass (row 5).
- **Lead-in animation, sweep animation.** The two animations of a domination: the lead-in animation
  starts at launch, the sweep animation at the sweep. Their types come from the rules (Import
  mapping). For an animation, *k* is its stage and *N* its sprite's image-frame count, both as in
  S-0109 (Terms). *k* is the stage, not the image frame shown.
- **Sweep.** The single moment of a domination at which its area damage and its takeovers happen
  (claims 4 and 8 to 15).
- **Swept cells.** The cells the sweep visits (claim 11).
- **Start object.** In a swept cell, the object the sweep examines first (claim 12).
- **Examined object.** An object the sweep reaches in a swept cell and tests for takeover
  (claims 12 and 13).
- **Airborne.** In this spec, S-0308's sense: the state in which the original counts an object as in
  the air (S-0308 Terms; for aircraft, S-0501 claim 18's *in-the-air answer*).

## Behaviour

### Launch

1. **Launch.** A launch of the psychic domination kind at target cell *c* by house *H*, accepted
   under S-0512 claim 8 and S-0513 claim 3, has these effects in the tick it is applied:
   - a radar event of the same kind as the invulnerability launch's at *c*,
   - when both the lead-in and the sweep animation types are set (claim 2), a domination starts:
     its dominating house is *H*, its target cell is *c*, the lead-in animation starts at the centre
     of *c* at the terrain surface, playing once, and a lighting re-evaluation happens (S-0104
     claim 5),
   - the activation announcement for every player, unless launch announcements are muted for that
     player (S-0701 claim 28),
   - the activation sound at the centre of *c*, heard by players near it,
   - for the launching player, the end of aiming mode and the silencing of the type's ready
     announcement (S-0513 claim 5).

   For S-0513 claim 5's table this adds the row: *psychic domination: every player: activation
   announcement; activation sound at the target; radar event at the target cell*.
   - Backing: `measurable` (PD1)
   - Confidence: high

2. **Missing animation types.** When either animation type is not set, the launch still has every
   effect of claim 1 other than the domination: the charge is spent (S-0512 claim 8), the
   announcement, sound and radar event happen, and no domination starts. No domination is then in
   force, so another launch of any house is accepted. No lighting re-evaluation happens. Our
   importer warns about a psychic domination superweapon whose rules leave either type unset.
   - Backing: `measurable` (PD8, named test data)
   - Confidence: high

### Phases

3. **In force.** A domination is in force from the launch that starts it until it is over (claim 5).
   While it is in force every launch of this kind by any house is refused as S-0512 claim 8 states.
   - Backing: `measurable` (PD2)
   - Confidence: high

4. **When the sweep happens.** The first domination update after the launch cannot sweep.
   The lead-in animation is already playing from the launch. The sweep happens at the first later domination update at
   which the lead-in animation's stage satisfies the threshold of F1, read as it stands at that
   update (before that tick's turn pass). A launch applied in tick *L* (S-0203 claim 7) therefore
   sweeps in tick *L* + 2 at the earliest.
   - Backing: `measurable` (PD3). The exact equality cases of F1 are `measurable` only with named
     test data whose fire-at percentage and frame count give 100*k* = *pN* (PD3 part c)
   - Confidence: high

5. **Final phase and end.** After the sweep the sweep animation is the one that times the
   domination (the lead-in animation keeps playing to its own end). The domination's final phase
   (S-0104 Terms) starts at the domination update of F2, and a lighting re-evaluation happens then
   (S-0104 claim 5). The domination is over at a later domination update at which the current
   ambient equals the target ambient (S-0104 claims 4 and 5), with no re-evaluation. In S-0104's
   words "its animation reaches its last image frame" means the sweep animation's stage reaching
   *N* − 1 as F2 states.
   - Backing: `measurable` (PD4)
   - Confidence: high for the thresholds; medium for the tick of the final phase (PD4 resolves it to
     the tick)

6. **Animations that end too early (departure).** The original keeps timing the domination by an
   animation after that animation has been removed, which is undefined: it happens when the lead-in
   animation ends before F1's threshold is met (a fire-at percentage near or above 100), or when the
   sweep animation ends without its stage reaching *N* − 1 (a type whose image frames are half
   shadow frames, or one with a set end frame). In our engine:
   - when the lead-in animation is removed before the sweep, the sweep happens at the next
     domination update,
   - when the sweep animation is removed before F2's final threshold is met, the final phase starts
     at the next domination update.

   Our importer warns about a fire-at percentage above 100 × (*N* − 1) / *N* for the lead-in type,
   and about a sweep animation type whose stages cannot reach *N* − 1.
   - Backing: `unmeasured`: the original's behaviour here is undefined, so there is nothing stable
     to observe
   - Confidence: high that the original is undefined; the replacement is design

7. **One domination for all houses.** There is never more than one domination. A second launch while
   one is in force is refused (claim 3). With a zero ambient change rate the current ambient never
   moves (S-0104 claim 4, `quirks.zero_rate_freezes_ambient`). The domination is then over at the
   first domination update after its final phase starts at which the target ambient equals that
   frozen current ambient:
   - in the plain case the final phase's re-evaluation sets the target back to the same base ambient
     the game started with, so the domination is over at the next domination update,
   - when the target then differs from the frozen value (a storm in force, or a base ambient changed
     during the game), the domination stays in force until a later lighting re-evaluation brings the
     target back to the frozen value, and for the rest of the game if none does. While it stays in
     force no house can launch this kind.
   - Backing: `measurable` (PD2 part c, named test data)
   - Confidence: high

### The sweep

8. **Blast ring and animation at the sweep.** At the sweep, before the damage, a blast ring appears
   at the centre of the target cell, and the sweep animation starts there, playing once. The blast
   ring deals no damage, starts no animation of its own and takes no draws. It is
   updated with the other blast rings (S-0203 row 15) and is removed at its 80th update, so it is
   present for 79 of them. While it lasts it sets the body-rocking angles of objects near it, by
   a formula not established here (S-0113 Open questions).
   - Backing: `measurable` (PD3 part b for the ring's lifetime; PD9 for the absence of draws),
     `unmeasured` for the rocking angles: no scenario records them
   - Confidence: high; low for the rocking angles

9. **Area damage.** At the sweep, before any takeover, area damage (S-0303) is dealt:
   - at the centre of the target cell at the height of its terrain surface (no bridge height is
     added, so S-0303 claim 4 gives the ground layer and claim 6 adds no flying objects),
   - of the imported domination damage, with the imported domination warhead,
   - with no source object, and with the reduction of resources on (S-0303),
   - with the attacking house being the current storm owner (S-0513 claims 14 and 19), including
     the latest requesting house while a storm is pending, and no house after storm cleanup or
     before any request (`quirks.dominator_damage_storm_house`, default on = the original).

   The sweep uses the storm owner before that tick's own storm update. Thus a storm that clears
   later in the same update still supplies its owner for this damage.

   With no source object S-0302's allies row does not apply, so the attacking house affects only who
   is credited with kills and losses. Off: the attacking house is the dominating house. An object
   destroyed by this damage takes no turn in this tick unless S-0312 keeps it in play (S-0203
   claim 6).
   - Backing: `measurable` (PD5; part b for the attacking house)
   - Confidence: high

10. **Damage comes before takeover.** Every effect of the area damage of claim 9 is complete before
    the first takeover of the sweep. An object destroyed by it is not taken over. An object that
    survives it is taken over with the health it has after the damage. The takeovers of claims 13
    and 14 never use the damage's attacking house.
    - Backing: `measurable` (PD5)
    - Confidence: high

11. **Swept cells.** The swept cells are the cells of S-0303 table T1 with ring metric at most
    min(*r*, 10) from the target cell, visited in T1's scan order, where *r* is the imported capture
    range. Rings beyond 10 are never swept, whatever *r* is. Only the ground layer of each swept cell
    is visited, so an object on a bridge deck is never reached
    (`quirks.dominator_ground_layer_only`, default on = the original; off: the bridge-deck layer of
    a swept cell is visited after its ground layer, in the same way). A cell outside the map holds
    no objects. A negative capture range is undefined in the original (it can crash or sweep
    nothing). In our engine it sweeps no cell, and our importer warns (departure).
    - Backing: `measurable` (PD6)
    - Confidence: high

12. **Which objects of a cell are examined.** In each swept cell the sweep finds the *start
    object*: among the infantry, vehicles, aircraft and buildings listed in the cell's ground layer,
    the one with the smallest S-0207 F8 distance (residues measured from the top corner of the cell
    that holds its position, with the table square root, compared as integers), the one listed first
    (S-0209 claim 13) winning a tie. From the start object it examines objects forward in listing
    order and stops before the first listed object that is none of those four kinds (a terrain
    object, for example) or at the end of the list. The swept layer (glossary) is the ground layer.
    - Objects listed before the start object are never examined, nor are objects after a stop.
    - A building's position for this distance is the centre of its foundation, so along a
      foundation side with an even number of cells its residue is 0, and along an odd side 128. In
      any cell of a building with at least one even side, the building's distance is at most 128,
      less than the 181 of an object at a cell centre. Such a building is then the start object
      unless some object of that cell is nearer, and because buildings are listed after every
      non-building, the infantry, vehicles and aircraft of that cell are not examined. Units
      standing in the cells of such a building, on a pad or docked, for example, are therefore
      usually not taken over.
    - A building whose sides are both odd has the distance 181 of a cell centre, so an object at the
      cell centre ties with it and, being listed first, is the start object.
    - A building can be the start object in several swept cells, but buildings are never taken
      (claim 13).

    `quirks.dominator_sweep_from_nearest` (default on = the original). Off: every listed infantry,
    vehicle and aircraft of the cell's ground layer is examined, in listing order.
    - Backing: `measurable` (PD7 part a; PD10 part b for the building case). The stop before a
      non-combatant listing is `unmeasured`: such objects (terrain objects) are placed before units
      enter, so they are listed after every unit, and the stop then skips only buildings, which are
      never taken anyway
    - Confidence: high

13. **Who is taken over.** An examined object is taken over unless any of these holds:
    - it is a building,
    - its type has psionics immunity (S-0610 claim 1),
    - it is protected (S-0513 claim 27),
    - its type hovers like a balloon (S-0601 Import mapping),
    - it is airborne.

    These are the conditions of S-0601 claim 13 and no others. In particular there is no test of
    owner, alliance, dying, bunker link or prior control:
    - objects of the dominating house itself are taken over (claim 14 says what that changes),
    - allied and neutral objects are taken over,
    - an object still in play while dying (an infantry unit in its death sequence, a sinking vessel:
      S-0203 claim 6) is taken over (`quirks.dominator_takes_dying`, default on = the original; off:
      a dying object is not examined),
    - an already permanently controlled object is taken over again,
    - a vehicle in a bunker is reached when it is the start object or follows it. With a bunker of
      two by two cells the vehicle stands exactly at the bunker's foundation centre, so it has the
      same F8 distance as the bunker (0) and, listed before it, is the start object of that cell and
      is taken over. This is not an exception to claim 12's building case: the vehicle is not
      farther than the building. The bunker link stays (S-0606 claim 15).
    - Backing: `measurable` (PD7, PD10)
    - Confidence: high for the conditions and the own-house, allied and repeat cases; medium for the
      dying and bunker cases

14. **Takeover.** An object taken over:
    - is first released by its temporary controller, if it has one (S-0601 claim 9, with all its
      effects),
    - then changes owner to the dominating house, with every change-of-owner consumer (S-0601
      claim 6). When it already belongs to the dominating house, no owner change happens and none of
      those consumers runs,
    - then becomes permanently controlled and gets the permanent control ring (S-0601 claims 13 and
      18), even when it already belonged to the dominating house
      (`quirks.dominator_marks_own_objects`, default on = the original; off: an object that already
      belongs to the dominating house is left unchanged).

    An object taken over again keeps exactly one permanent control ring. The original attaches a
    second ring and keeps the first, so it shows two.
    - Backing: `measurable` (PD7 parts b and c, PD10)
    - Confidence: high

15. **What the sweep does not reach.** Only examined objects are direct takeover recipients.
    Indirect effects of changing a recipient's owner still follow claim 14:
    - passengers of a taken transport and occupants of a garrisoned building keep their owners, and
      no passenger or occupant changes owner as a consequence,
    - objects on a bridge deck (claim 11) and airborne objects (claim 13) are untouched,
    - a temporary controller that is taken over keeps its victims (S-0601 claim 11, under its
      `quirks.mind_control_owner_change_keeps_victims`).
    - Backing: `measurable` (PD10)
    - Confidence: high

16. **Orders after a computer house's takeover (departure).** In the original, when the dominating
    house is computer-controlled, every object taken over by the sweep is given *hunt* as its next
    behaviour, after all takeovers of the sweep. In our engine an AI house follows the
    human-controlled rule (no additional hunt order is given), and the hunt order is kept only when the dominating
    house is the neutral or a special house (glossary, *AI house*, *computer-controlled*). The AI
    module may give its own orders.
    - Backing: `measurable` for the original's hunt (PD7 part d). Our engine's rule is design
    - Confidence: high

### Ownership and coexistence

17. **The dominating house is fixed at launch.** Losing, selling or having the launching building
    captured after the launch changes nothing about the domination: the sweep still happens on time,
    and its takeovers still go to the dominating house. If the dominating house is defeated before the
    sweep, the takeovers still go to it (Open questions).
    - Backing: `measurable` (PD2 part b)
    - Confidence: high for loss, sale and capture; medium for defeat

18. **Other effects.** A domination coexists with a weather storm, a nuclear strike and a chrono
    flash. Its own claims do not depend on them, except that the area damage's attacking house
    depends on the current storm owner (claim 9). How the lighting of overlapping effects combines is
    S-0104 claims 2 and 5.
    - Backing: `measurable` (PD5 part b and PD11)
    - Confidence: high

## Formulas

**F1 Sweep threshold** (claim 4). Let *p* be the imported fire-at percentage (an integer), and *k*
and *N* the lead-in animation's stage and image-frame count at the domination update. The original
compares in double precision, with a 53-bit significand, as two separately rounded values:

> sweep ⇔ RZ₅₃(*p* × *c*) ≤ RZ₅₃(*k* / *N*)

where *c* is the double nearest to 0.01, *p* and *k* and *N* convert to doubles exactly, and RZ₅₃
rounds toward zero to 53 bits as in S-0205 claim 5 (`quirks.chop_rounding`; off, RN₅₃,
rounding to nearest with ties to even, replaces RZ₅₃). The
result equals the exact integer test

> sweep ⇔ 100 *k* ≥ *p N*

except when 100 *k* = *p N* exactly. At exact equality the sweep does not happen for these *p*, and
does for every other *p* from 0 to 100 (for any *N*):
- under RZ₅₃: *p* = 1, 2, 4, 8, 11, 16, 19, 22, 32, 38, 44, 51, 63, 64, 76, 88 or 89,
- under RN₅₃: *p* = 35, 41, 47, 57, 69, 70, 82, 83, 94 or 95.

**F2 Final phase** (claim 5). Let *k*₂ and *N*₂ be the sweep animation's stage and image-frame count,
read at each domination update. Let *t*₁ be the first domination update after the sweep at which
*k*₂ ≥ *N*₂ − 10. The final phase starts at the first domination update after *t*₁ at which
*k*₂ ≥ *N*₂ − 1. So it starts at least two domination updates after the sweep. Integer arithmetic.

**F3 Swept cell count** (claim 11). With *r'* = min(*r*, 10) for *r* ≥ 0, the number of swept cells
is T1's count for ring *r'*: 1, 9, 21, 37, 61, 89, 121, 161, 205, 253 or 309 for *r'* = 0 to 10.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Lead-in minimum | 2 | ticks | Earliest sweep is tick *L* + 2 for a launch applied in tick *L* (claim 4) |
| Near-end threshold | 10 | stages | F2, *k*₂ ≥ *N*₂ − 10 |
| Final threshold | 1 | stages | F2, *k*₂ ≥ *N*₂ − 1 |
| Percentage scale | 0.01 | per cent | The double nearest to 0.01 (F1) |
| Blast ring lifetime | 79 | blast-ring updates | Removed at the 80th (claim 8) |
| Largest swept ring | 10 | ring metric | Capture range above it is treated as 10 (claim 11) |
| Default fire-at percentage | 50 | per cent | Engine default when absent |
| Default capture range | 2 | cells (ring metric) | Engine default when absent |
| Default domination damage | 50 | health points | Engine default when absent |
| Default warhead, animation types, sound, permanent ring | none | — | Engine defaults when absent |

## State and ordering

| State | Event | Result |
|---|---|---|
| no domination | launch accepted, both animation types set | domination in force, lead-in started, lighting re-evaluated (claim 1) |
| no domination | launch accepted, a type unset | no domination; charge spent; announcements, sound and radar event (claim 2) |
| in force, before sweep | first domination update after launch | no sweep; lead-in continues (claim 4) |
| in force, before sweep | later domination update, F1 holds | blast ring, sweep animation, area damage, then takeovers in swept-cell order (claims 8 to 15) |
| in force, after sweep | domination update, F2 holds | final phase starts; lighting re-evaluated (claim 5) |
| final phase | later domination update, current ambient = target ambient | over; no re-evaluation (claim 5) |
| in force, any phase | launch of this kind by any house | refused (S-0512 claim 8) |

Ordering, as observable consequences:
- The sweep happens in S-0203 row 3 of its tick, before the turn pass, so a unit taken over in tick
  *t* takes its turn of tick *t* under the dominating house.
- An object killed by the sweep's area damage cannot become a takeover recipient, and a surviving
  recipient keeps its post-damage health. Effects of takeovers follow T1's cell order and claim 12's
  within-cell order. A temporary controller's release of a victim (S-0601 claim 9) has completed
  when that victim becomes permanently controlled, so a later cell's takeover of the controller
  cannot affect that already-released victim.
- Claim 16's hunt orders are available before the recipients' turns of that tick; they cannot
  influence which objects the sweep takes over.

## Edge cases and quirks

- `quirks.dominator_damage_storm_house` (claim 9). Default on: the area damage uses the current
  storm owner, sampled before the storm's own update, including a pending storm's requesting house
  and no house after cleanup. Off: it uses the dominating house.
- `quirks.dominator_ground_layer_only` (claim 11). Default on: bridge decks are never swept. Off:
  each swept cell's deck layer is swept too.
- `quirks.dominator_sweep_from_nearest` (claim 12). Default on: only the start object and the objects
  listed after it are examined, up to the first non-combatant listing. Consequences: an infantry unit
  that entered a cell after another infantry unit standing nearer the cell's top corner escapes; and
  units in the cells of a building with an even foundation side usually escape, because the building
  is the start object. Off: every infantry, vehicle and aircraft of the ground layer is examined.
- `quirks.dominator_marks_own_objects` (claim 14). Default on: the dominating house's own objects
  become permanently controlled with a ring. Off: they are left alone.
- `quirks.dominator_takes_dying` (claim 13). Default on: a dying object still in play is taken over.
  Off: it is skipped.
- **Double ring (fixed, no switch).** Claim 14: one ring per object. The original leaves the old ring
  in place when it adds a new one.
- **Ended animations, negative range, missing types (departures).** Claims 2, 6 and 11.
- **Bunkered vehicle.** Claim 13. With bunkers of other foundation sizes the vehicle and the bunker
  can have different F8 distances, so the vehicle is reached only when it is the start object or
  follows it.

## Import mapping

| Key | Section | Form | Feeds | Absent |
|---|---|---|---|---|
| `Type` | superweapon type | kind name | the psychic domination kind, by the value S-0512's Import mapping gives | — |
| `DominatorFirstAnim` | `[General]` | animation type name | lead-in animation type | none (claim 2) |
| `DominatorSecondAnim` | `[General]` | animation type name | sweep animation type | none (claim 2) |
| `DominatorFireAtPercentage` | `[General]` | integer | *p* of F1 | 50 |
| `DominatorCaptureRange` | `[General]` | integer | *r* of claim 11 | 2 |
| `DominatorDamage` | `[General]` | integer | area damage amount (claim 9) | 50 |
| `DominatorWarhead` | `[General]` | warhead name | area damage warhead (claim 9) | none |
| `PsychicDominatorActivateSound` | `[AudioVisual]` | sound name | activation sound (claim 1) | none |
| `PermaControlledAnimationType` | `[CombatDamage]` | animation type name | permanent control ring (S-0601 owns it) | none |
| `ImmuneToPsionics` | object type | boolean | psionics immunity (S-0610 owns it) | S-0610 |
| `BalloonHover` | object type | boolean | balloon hover (S-0601 owns it) | S-0601 |
| `MindControlRingOffset` | object type | integer, leptons | ring height (S-0601 owns it) | S-0601 |

With the warhead absent the original's area damage call has no warhead. S-0303 owns what a missing
warhead does.

## Measurement scenarios

All scenarios: stock data except the explicitly named test data below, no engine add-on and no mods (the named test files of a scenario excepted), a small purpose-made skirmish map, game speed index 3, 800 × 600
presentation. Use scripted input and a read-only recorder when needed to deliver and observe the
stated setup on the stock executable. Record the seed rather than assuming it can be set. Repeat each deterministic part three times. Include one bystander computer
player, kept outside the test area under the glossary's void rule, except where a part names a
computer launcher. Record tick counts, owners, health and orders using read-only observation, and
capture frames for presentation. Unless a part gives a narrower falsifier, any observed departure
from its expected outcome falsifies that part. A setup not achievable with stock data or the
scenario's explicitly named test data is not a successful measurement.

- **PD1. Launch.** A player with a ready psychic domination launches at an empty cell in view.
  Record the radar event, the announcement, the activation sound's position, the end of aiming mode,
  the silenced ready announcement and the lead-in animation's first frame. Expected: all in the
  launch's application tick, the animation centred on the cell. Falsifier: any missing, or the
  animation appearing elsewhere.
- **PD2. In force and refusal.** (a) Two players with ready dominations: the first launches, the
  second tries in every tick until it is accepted. Expected: refused (charge kept) while the first
  domination is in force at order application, and accepted in the same tick whose domination
  update ends it, since that update precedes order application. (b) The first player sells, loses or has
  captured its launching building in the tick after the launch. Expected: sweep at the same tick as
  an unchanged control run, takeovers to the first player. Repeat with that player defeated between
  launch and sweep, keeping two other opposing contenders alive so observation continues. Record
  ownership at the sweep itself separately from later defeat cleanup. Expected: the original
  launching house still receives the takeovers. (c) Named test data with a zero ambient
  change rate, as S-0104 scenario L16 part C. Run 1, base ambient unchanged: the second player's
  launch is accepted from the domination update after the final phase starts. Run 2, a trigger
  changes the base ambient before the launch: the second player remains refused while the recorded
  target differs from the frozen current ambient. Later restore the base ambient to the frozen
  value with a trigger. Expected: refusal ends at the next domination update, not before it.
- **PD3. Sweep timing.** (a) Launch in tick *L* with stock data. Record the tick of the first owner
  change and of the sweep animation's first frame. Expected: F1 with stock values, and never before
  *L* + 2. (b) Record the tick the blast ring appears and the tick it is removed.
  Expected: creation in the sweep tick, removal 79 ticks later, since its first update is in its
  creation tick. (c) Named test map overrides select a lead-in type and fire-at percentage so that
  100*k* = *pN* is met at some stage, with one-stage advances and no early removal. Run once with
  a percentage in the RZ exception set and once outside it. Record each run's equality tick rather
  than comparing absolute sweep ticks. Expected: no sweep at equality in the exception run,
  then a sweep at its first later qualifying update; a sweep at equality in the other run.
  Falsifier: either run disagrees with its own equality prediction.
- **PD4. Final phase.** As PD3 part a, recording the tick the tint returns from the dominator set
  (S-0104 L16) and the tick a second launch is first accepted. Expected: F2 for the first and the
  first domination update after the final phase starts with current ambient equal to the target
  for the end of refusal. An order applied later in that same tick is accepted. Record animation
  stages and both ambient values each tick to distinguish this from a one-tick-late acceptance.
- **PD5. Damage.** (a) Units of several armour types at the target cell and nearby, with recorded
  health before and after the sweep tick. Expected: S-0303 area damage of the domination amount and
  warhead from the cell centre on the ground, applied before any owner change. A unit killed by it
  is not taken over. (b) A third player requests a weather storm, then the domination kills an
  enemy unit. In separate runs the sweep falls during the storm countdown, during the raging
  storm, and after its cleanup. Use a named test map override extending the countdown if necessary
  to fit a sweep before it ends. Expected: credit to the requesting house in the first two runs,
  and to nobody after cleanup. A run with no storm request also credits nobody. Align another
  sweep with the update that clears the last storm: expected credit still goes to the storm owner,
  because the sweep precedes cleanup in that update. Record storm state, owner and kill credit at
  the sweep. Credit retained after cleanup, missing during the countdown or credited to the
  dominating house falsifies the corresponding part.
- **PD6. Range.** A grid of enemy infantry, one per cell out to ring 12. Named test map overrides
  set domination damage to zero to isolate takeovers; use the stock capture range, then named
  overrides of the capture range at 0, 10 and 12. Expected: exactly the cells of rings 0 to min(*r*, 10)
  change owner. A unit on a bridge deck over a swept cell keeps its owner.
- **PD7. Examined objects.** Named test map overrides set domination damage to zero to isolate
  takeovers. In one swept cell: (a) enemy infantry only. Infantry A first takes the
  spot of the cell nearest its top corner, then infantry B moves into a spot farther from it.
  Expected: A changes owner, B (listed before A) keeps its owner. Control: B moves in first and A
  second. Expected: both change owner. (b) A
  unit of the dominating house and an allied unit. Expected: both get the permanent ring; the allied
  unit changes owner, the own unit does not. (c) A unit already permanently controlled by a third
  house. Expected: it changes owner again and shows one ring (our engine) or two (the original). (d)
  A computer house launches. Expected: every taken unit is given *hunt*.
- **PD8. Missing types.** Named test rules files omit the lead-in type, the sweep type and both,
  in separate fresh program sessions, so omitted keys have their initial defaults rather than
  values retained from an earlier game. Expected: charge
  spent, announcement, sound and radar event, no lighting change, no owner change, and a second
  launch accepted at once.
- **PD9. Blast ring draws.** Launch at an empty area with no object within the damage spread or
  swept cells. Named test art and rules files supply the two timing animations with fixed playback,
  no random rate, random start, sounds, particles, smudges, damage, follow-up or other side effects,
  and omit the activation sound. Choose frame counts and timing that let both animations finish
  safely under F1 and F2. Keep every computer house's base far from the target so that S-0513
  claim 6's base-defence draw does not occur. Compare scenario-stream draws with a no-launch
  control, accounting for animation creation draws under S-0109 claims 1 to 4 and S-0202 claim 15.
  Expected: no additional draw attributable to the blast ring's creation or lifetime. Falsifier:
  any excess draw after animation draws and unrelated activity have been accounted for.
- **PD10. Special recipients.** Named test map overrides set domination damage to zero to isolate
  takeovers. Put each subject in its own swept cell except the linked pairs explicitly named.
  (a) In swept cells: a loaded transport, a garrisoned building, a vehicle
  in a two by two bunker, an enemy temporary controller with a victim outside the area, a victim
  inside the area of a controller outside it, a protected unit, a psionics-immune unit, an aircraft
  in flight and an infantry unit in its death sequence. Expected: the transport and the vehicle in
  the bunker change owner (the bunker link stays); passengers and occupants keep theirs; the
  controller changes owner and keeps its victim; the victim is released and then taken by the
  dominating house; the protected, immune and flying units are unchanged; the dying infantry unit
  changes owner. Repeat for a sinking vessel, a neutral unprotected non-immune unit, and a landed
  balloon-hover type. Where stock types combine immunity or flight with balloon hover, named test
  overrides disable immunity and permit a stationary ground-layer subject. Expected: the sinking
  and neutral subjects change owner, the balloon-hover subject does not. Record positions and cell
  listings to establish that the bunker vehicle is actually reached, rather than assuming it.
  (b) In the cells of two stock building types, one with an even foundation side
  and one with both sides odd, put an enemy unit on a cell of the building (on a pad, docked, or
  otherwise standing in its foundation). Expected: the unit in the even-sided building's cell keeps
  its owner, the unit at the centre of the odd-sided building's cell changes owner. Use subjects
  at recorded cell-centre positions with no other nearer non-building in either listing. If no
  stock pad or dock supplies both cases, named test map overrides make two buildings passable to
  the test unit without changing their foundation or reference point.
- **PD11. Coexistence.** Repeat PD3 with a storm, a nuclear detonation far from the test subjects,
  or a trigger-started chrono flash overlapping the domination, separately and then together.
  Keep subjects and the launching building outside unrelated damage. Record the domination's
  stages, sweep tick and recipients, and the other effects' continuation. Expected: the same F1
  and F2 thresholds and takeover recipients as the control, with no cancellation of either effect.
  Lighting follows S-0104 claims 2 and 5 and may change the end tick through the ambient equality
  test. A missing sweep, cancelled overlapping effect or altered eligibility falsifies claim 18.

## Open questions

- **Defeated dominating house.** When the dominating house is defeated between launch and sweep,
  the takeovers still go to it (claim 17). What S-0414 then does with objects that arrive at a
  defeated house is not established here.
- **Bunkers of other sizes.** Claim 13's bunker case is reasoned from the two by two foundation and
  S-0209's listing order. PD10 should confirm it, and bunkers of other foundation sizes are not
  covered.
- **Dying objects.** Claim 13's dying case relies on S-0203 claim 6's description of objects still
  in play. PD10 should confirm it for infantry and for a sinking vessel.
