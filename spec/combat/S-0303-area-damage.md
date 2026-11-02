---
id: S-0303
title: Area damage (spread, falloff, cells scanned, walls, bridges and detonation effects)
topic: combat
revision: 32
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-02
measurements: []
requests: [Q-0020, Q-0060, Q-0061, Q-0614]
supersedes: []
---

# S-0303: Area damage (spread, falloff, cells scanned, walls, bridges and detonation effects)

## Summary

How one detonation of a warhead at a point spreads its damage: which cells and objects it reaches,
the distance each object is measured at, how the damage falls off, which objects are left out, the
order of the hits, and what the detonation does to walls, bridges, resource cells and exploding
overlays; it also gives the explosion animation and the debris of a projectile's detonation. What
one hit does to one object (armour, multipliers, the health change) is damage resolution, S-0302,
and what happens when an object dies is S-0312. This revision covers skirmish; network games
and campaigns are out of scope.

Revision 25 takes in the S-0303 entries of the Changes for other specs of S-0318 and S-0317. From
S-0318: the detonation order table gains row 1a for the fragments of S-0318, and a sentence after
the table gives the draws that follow each detonation that is not an airburst (S-0306 claim 21).
From S-0317: the voxel-model debris of claim 22 and the barrel debris of claim 19 are pieces of
S-0317, objects of play whose creation and later behaviour are S-0317's; claim 23 names a piece's
expiry damage beside the disc laser. Row 1a and the sentence after the order table are
cross-references rated by S-0318 and S-0306.

Against revision 19, revision 25 also makes one claim of its own: claim 19's barrel debris
piece, when one is chosen, takes its creation draws (S-0317 claim 3 or 4) after the draw that
chose it and before the barrel particle system's percent draw, where revision 19 listed no draws
between the two. That placement is new behaviour of this revision, rated medium in claim 19 until
scenario A13 variant d runs. Against revision 24, revision 25 changes no behaviour.

## Terms

Glossary terms are used as defined there, in particular *warhead*, *weapon*, *cell*, *lepton*,
*world height*, *height above the terrain*, *bridge deck*, *high bridge*, *cell layer*, *foundation*,
*aircraft*, *building*, *overlay*, *overlay data value*, *scenario random stream*, *draw*, *ranged
draw*, *round-toward-zero mode*, *departure*, *importer warning*, *house*, *known attacker*. In
addition:

- **Detonation.** One application of area damage: a *detonation point* *P* = (*x*, *y*, *z*) in
  leptons, an integer *damage amount* *D* (the amount the producer passes in, before any armour step of the
  target: S-0302's delivered damage *d*₀ for every hit of the detonation), a warhead *W*, an optional
  *source object* (the object that caused it, for example the firer of a projectile), an optional
  *source house*, and whether its producer allows resource reduction (claim 17; resource-explosion
  detonations of S-0401 claim 20 forbid it). Projectiles (S-0306, S-0203 claim 5), damaging animations (S-0109 claim 17), death
  weapons and crashes (S-0312), superweapons and some trigger actions (their specs) produce
  detonations; each of those specs says which *P*, *D*, source object and source house it passes.
- **Spread** *S*. The warhead's spread field, a single-precision decimal number of cells (S-0003).
- **Edge fraction** *p*. The warhead's edge-percentage field, a single-precision decimal fraction
  (S-0003): the share of the damage left at the edge of the spread.
- **Spread radius** *R*. The distance in leptons within which objects are hit (Formulas F1).
- **Scan radius** *k* and **scanned cells**. The cells whose contents the detonation looks at
  (Formulas F2 and table T1), listed in the **scan order** of table T1.
- **Centre cell.** The cell that contains *P* (S-0204 claim 2: coordinates truncated toward zero).
- **Ring metric** *m*(Δ*x*, Δ*y*) = max(|Δ*x*|, |Δ*y*|) + ⌊min(|Δ*x*|, |Δ*y*|) / 2⌋ for a cell
  displaced by (Δ*x*, Δ*y*) cells from the centre cell.
- **Epicentre distance** *d*. The distance in leptons at which one hit measures an object (claim 7).
- **Defended amount** *d*₃. S-0302's amount after the target's defences (its Formulas F2: the house
  and object armour, the veteran armour factor, the prone factor and the minimum of 1). It equals
  *D* when the house and the object armour are both 1.0, the target is not prone and has no veteran
  armour bonus, and can differ from *D* otherwise. The falloff of claim 12 works on it, not on *D*.
- **Undemolishable building.** S-0302's term (its claim 7). This spec uses only its rule that every
  hit costs such a building at least 1 health point.
- **In the air.** The test of S-0308 (Terms, *airborne*): the state in which the original counts an
  object as in the air, whose exact test the aircraft and movement specs own. Claim 10 uses it. It is
  not the height test of S-0307 and S-0306.
- **Hit.** One delivery of the detonation's damage to one object at one epicentre distance. Damage
  resolution (S-0302) turns a hit into a health change. One object can receive several hits from
  one detonation (claim 8).
- **Projectile position** *Q* = (*q*ₓ, *q*ᵧ, *q*_z). The position of the projectile object itself
  when it detonates (S-0306). It can differ from the detonation point *P*: S-0306 claim 20 can move
  the detonation point away from it, and the further detonations of a cluster are at other points
  (S-0306 claim 21). Claims 21 and 22 use *Q*, not *P*, for the land type, the heights, the bridge
  and naval tests and the position of the debris.
- **Naval vehicle.** A vehicle whose type has the naval flag (Import mapping). An **underwater
  type** is a type with the underwater flag (Import mapping; S-0308 uses the same flag).
- **Small detonation.** A detonation whose warhead has *S* ≤ 0.5.
- **Curtain protection.** The invulnerability effect of the iron-curtain kind, as opposed to the
  force-shield kind of the same effect. The superweapon specs own both; here only the difference
  matters (claim 15).
- **Self-damage flag.** A yes/no field of a unit or building type that lets its own detonations hit
  it (Import mapping).
- **Wall hit.** The damage of one detonation to the wall overlay of one cell (claim 16).
- **Nullified detonation.** A detonation that claim 15 restricts to objects under the invulnerability
  effect.
- **Tracked flying object.** An object registered for the flying-object search: an aircraft-kind
  object in ordinary flight (S-0501 claim 47 gives the events for an aircraft), a unit moving
  with the jumpjet locomotor,
  including a seized unit (S-0502 claim 29), or a rocket missile from the end of its tilt or raise
  phase until it detonates (S-0506 claims 15 and 19). Also included is any unit placed on the map
  while airborne whose type has the counts-as-aircraft flag (Import mapping; S-0502 claim 29). Such
  placement registration lasts until it goes off the map, unless its movement ends registration
  earlier. An aircraft placed in the air with that flag ends it at touchdown like any other aircraft
  (S-0501 claim 47). Registration during jumpjet movement follows all the boundaries and exceptions
  of S-0502 claim 29, not merely its height or flight state. Rocket missiles cease to be tracked
  before their own detonation's area damage. Each is registered in a *sector* (Formulas F7), updated
  at the movement events that its movement spec defines; S-0502 claim 29 owns these events for
  jumpjet movement.
- **Sector.** One of the 20 × 20 rectangles of whole cells into which F7 divides the map for the
  flying-object search.
- **Turn.** In claim 1, an object's share of one tick (S-0203, Terms), not an amount of rotation.

## Behaviour

### When area damage happens and what it reaches

1. **One detonation, all at once.** A detonation applied during an object's turn completes
   within that turn (for a projectile, S-0203 claim 5), before any other object acts. A detonation
   applied outside all object turns completes at that same out-of-turn place in the tick
   (S-0203 claim 1; S-0312 claim 1), before any object takes its next turn. Before
   the detonation has finished, every hit, wall hit and other effect of claims 3 to 20 has taken
   effect. If a hit sets off another detonation (an object's death weapon, S-0312, or an exploding
   overlay, claim 19), its effects occur before the outer detonation's later hit effects. Thus
   nested effects can change the outcome of later hits of the outer detonation (claim 11).
   - Backing: `measurable` (scenario A11: a death weapon going off between two hits of one
     detonation; S-0203 scenario O10 for out-of-turn damage)
   - Confidence: high

2. **Detonations that do nothing.** The area damage of a detonation whose damage amount is 0, or
   whose warhead is absent, has no effect at all: no hit, no wall hit, no bridge or overlay effect,
   no particle system, and its result is *missed* (claim 23). On a map whose inert flag is set
   (Import mapping), the area damage of every detonation is like this. The explosion animation and
   debris of claims 21 and 22 belong to a projectile's detonation, not to the area damage. The
   inert flag does not suppress them. The animation is not chosen when the damage amount is 0 (F5),
   but the debris depends only on the warhead and is thrown whatever the damage amount. A
   projectile that has no warhead is a departure (Edge cases).
   A projectile whose warhead has the parasite flag does not apply this area damage: it makes
   no area hits, whether or not attachment succeeds (S-0614 claim 1). Its later explosion
   animation and debris are not suppressed by that exception. This is a projectile-producer
   rule, not an exclusion for other producers using the same warhead.
   A projectile whose warhead has the air-strike flag likewise makes no area hits, whether
   or not it calls or redirects a strike (S-0519 claim 16). Neither this exception nor the
   parasite exception suppresses radiation creation, the explosion animation, warhead debris
   or the light flash where configured. The air-strike exception is also a projectile-producer
   rule, not an exclusion for other producers using the same warhead.
   A projectile detonation that selects the electric-charge effect likewise bypasses ordinary
   area damage at any target (S-0518 claim 11), even when it registers no charger. This bypass
   does not suppress shared effects: radiation creation before effect selection, or the later
   explosion animation, warhead debris and light flash where configured. It is not a general
   immunity to effects of that shot, nor an exclusion for other producers using the same warhead.
   - Backing: `measurable` (scenario A1, variant e, for the inert flag; A14, variant f, for the
     damage amount 0; S-0614 scenarios P1 and P3 for parasite-projectile area-hit suppression,
     and A14 variant o for their later animation and debris; S-0518 scenarios TC1 and TC4 for
     electric-charge area-damage bypass and preserved common effects; S-0519 scenario T16
     for air-strike area-hit suppression and preserved common effects),
     `unmeasured` for area damage without a warhead (no producer with a
     reproducible named-data setup for that case is established) and for the projectile without
     a warhead (the original stops, so there is no effect value to compare)
   - Confidence: high

3. **Spread radius and scanned cells.** Objects can be hit only if they are in a scanned cell (or
   are tracked flying objects, claim 6) and their epicentre distance is at most the spread radius *R* =
   trunc(256*S*) leptons (F1). The scanned cells are the cells whose ring metric from the centre cell
   is at most the scan radius *k* = trunc(*S* + 0.99) (F2), for *k* from 0 to 10; table T1 gives the
   exceptions for *k* = 11. The two limits are separate: *R* follows *S* exactly, while *k* rounds
   *S* up to the next whole cell when its fraction exceeds 0.01. An object at a distance at most *R*
   in a cell that is not scanned is not hit, and an object in a scanned cell further than *R* is
   not hit either. This has no switch, as decided at the delegated sign-off of 2026-10-10.
   - Backing: `measurable` (scenario A2, variant b for ring 11)
   - Confidence: high

4. **Cell layer.** When the centre cell carries a high bridge and *P* lies more than 208 leptons
   (half the height of a bridge deck, 416 leptons, S-0102 and glossary) above the terrain at the
   horizontal position of *P*, the detonation looks at the bridge-deck layer of every scanned cell,
   otherwise it looks at the ground layer of every scanned cell. It never looks at both. So a
   detonation on a bridge deck hits only objects on bridge decks, and one under or beside the deck
   hits only objects in ground layers, even objects standing on a bridge deck that is within *R*.
   This has no switch, as decided at the delegated sign-off of 2026-10-10.
   - Backing: `measurable` (scenario A4)
   - Confidence: high

5. **Order of the objects within a cell.** Within one cell layer the objects are taken in the order
   in which the cell holds them. That order is the *listing order* of the cell layer (S-0209 claim 13),
   which S-0209 owns; this spec only relies on it being fixed.
   - Backing: `measurable` (scenario A11, variant c: two objects in the same cell, one of which
     has a death weapon that kills the other)
   - Confidence: medium

6. **Flying objects.** When the world height of *P* is greater than the terrain height at the
   centre of the centre cell (S-0106; the centre of the cell, not the horizontal position of *P*),
   the detonation also considers the *tracked flying objects* (Terms) that the flying-object search
   of Formulas F7 finds around the centre cell. Their hits come before those of the objects in the
   scanned cells (claim 11). Such an object takes part
   only while it is alive, on the map and has health above 0, and only if its epicentre distance,
   measured from *P* to its position (claim 7), is at most *R*. The search is coarse: it reads
   whole sectors of a 20 × 20 division of the map, and for a spread of several cells it can miss a
   tracked flying object that lies within *R* but in a sector off the four map axes and off the
   near diagonals (F7, `quirks.flying_search_sectors`). A detonation at or below that terrain
   height never hits a flying object through this rule, whatever *R* is
   (`quirks.ground_detonation_misses_aircraft`). A rocket missile in rocket-flight movement
   is never listed in a scanned cell, in any flight phase, including pause, tilt and raise
   (S-0506 claim 23). Before its tracking starts it is therefore absent from both candidate
   searches; afterwards only the flying-object search can find it. Its tracking starts and
   ends at the boundaries of S-0506 claims 15 and 19, not according to its height or its
   in-the-air answer.
   An object found by both the flying-object search and a scanned cell is a candidate twice,
   without merging its entries (`quirks.flying_listed_double_hit`). Low jumpjets can have
   this dual membership (S-0502 claim 31). Each eligible entry delivers its own hit, sized
   at that entry's epicentre distance (claim 7), with the flying entry first. The second
   entry does nothing if the first killed the object, because claim 9's eligibility tests
   apply again. With that switch off, an object found by both searches keeps only its flying
   entry and receives no scanned-cell entry.
   - Backing: `measurable` (scenario A5 for the height condition; scenario A16 for the sectors,
     and scenario A21 for dual membership, per-entry damage and lethal-first-hit exclusion), and
     S-0506 scenario R10 for rocket-missile registration at the end of tilt or raise, as
     S-0506 claim 15 specifies; S-0506 scenario R17 for rocket-missile exclusion from cell
     scans in every phase (S-0506 claim 23); `unmeasured` for the full sector membership list: A16 samples
     only three distant sectors rather than every F7 sector. Rocket-missile removal from the
     search before its own area damage follows S-0506 claim 19 but is `unmeasured` here:
     R11 records impact damage and effects, not the missile's eligibility for a nested flying-object
     search during that damage
   - Confidence: high for the height condition and rocket-missile cell exclusion; medium for
     the set of tracked objects and for F7; high for unmerged entries and their per-hit eligibility

### Distance, exclusions and order

7. **Epicentre distance.** Every distance here is a spatial distance (S-0204 claims 5 and 7):
   ⌊*r*(*n*)⌋ with *n* the sum of the squared differences of the three coordinates and *r* the table
   square root of S-0204 (F1), with no building reduction *β*. Every distance here follows
   `quirks.table_root_distance` (S-0204), which owns what replaces the table when the switch is
   off; this spec defines no switch of its own for it.
   - An object that is not a building is measured from *P* to its reference point (S-0204 claim 4),
     which for such an object is its position.
   - A building is measured once for each scanned cell of its foundation, from *P* to that cell's
     centre at the height of the ground surface at the cell centre (S-0106). In the centre cell the
     distance is 0, unless *P* lies more than 208 leptons above that cell centre, in which case it
     is the measured distance minus 208 leptons.
   - Backing: `measurable` (scenarios A1 and A3)
   - Confidence: high

8. **Buildings take one hit per scanned foundation cell.** A building appears in every cell of its
   foundation, and the detonation hits it once for each of those cells that is scanned and whose
   epicentre distance (claim 7) is at most *R*, each hit with its own distance. So a building with
   many foundation cells inside the spread takes several hits from one detonation, and a large
   building takes more damage from a wide spread than a small one with the same armour
   (`quirks.area_damage_per_foundation_cell`). Once a hit has destroyed it, its remaining hits from
   the same detonation do nothing (claim 9), except that a building kept alive at 1 health by a
   delayed kill remains eligible for later hits of the same detonation, as S-0302 claim 16
   specifies. Claim 9's alive, positive-health, on-map and in-play tests still apply.
   With S-0302's minimum for undemolishable buildings
   (claim 7), even hits that the falloff or the versus value bring to 0 cost such a building 1
   health point each.
   - Backing: `measurable` (scenario A3; S-0302 scenario D13 part d for the delayed-kill exception)
   - Confidence: high for per-cell hits and exclusions; medium for the delayed-kill exception,
     as S-0302 claim 16

9. **Objects left out.** A candidate object receives no hit when any of these holds:
   - it is the source object, unless the source object's type has the self-damage flag set, or
     the warhead is the game-wide crushing warhead (Import mapping),
   - at the moment of its hit it is not alive, has health 0 or less, is not on the map, or is held
     out of play (for example inside a transport, or dead and awaiting its removal, S-0312),
   - it is a vehicle whose type is in the game-wide harvester list while the map's
     harvester-immunity flag is set (Import mapping),
   - it is a building whose type is marked invisible in game (Import mapping),
   - its epicentre distance, after claim 10, is greater than *R*.
   Ownership plays no part here: objects of the source house and its allies are hit like any other
   (claim 14 gives the one owner rule, in damage resolution). The first and third conditions apply
   only to objects found in scanned cells: a tracked flying object found by claim 6 is not left out
   for being the source object or a listed harvester (`quirks.flying_candidates_unfiltered`).
   - Backing: `measurable` (scenario A6 for the source object, the self-damage flag and the
     harvester flag and the crushing-warhead exception; A3 for the destroyed building, and
     A16, variants c and c2, for the flying source object and listed harvester)
   - Confidence: high; medium for the exemption of tracked flying objects

10. **Aircraft in the air count at half distance.** An aircraft object that is in the air (Terms:
    S-0308's test) when it is
    hit has its epicentre distance halved, truncated toward zero, before the comparison with *R*
    and before the falloff of claim 12. This holds whether it was found by claim 6 or in a scanned
    cell. Jumpjets and other flying objects that are not of the aircraft kind keep their full
    distance.
    - Backing: `measurable` (scenario A5, variant c)
    - Confidence: high

11. **Observable order of hit effects.** The objects present when the detonation begins determine
    its candidate entries. An object that first appears in a scanned cell while hit effects are
    occurring receives no entry for that detonation (State and ordering). Across conforming
    implementations, the externally observable effects and scenario-random-stream draws have
    one consistent order. When hit effects from tracked flying objects of claim 6 can be
    distinguished from effects of scanned-cell objects, the flying-object effects and draws
    are observable before the scanned-cell effects and draws. Among flying objects, effects
    and draws follow F7's sector order and this claim's within-sector order. Among scanned-cell
    objects, effects and draws follow the cells' T1 order and each cell's listing order of
    claim 5 (S-0209 claim 13). A building's eligible foundation-cell hits have the corresponding
    observable positions in that sequence. For any two successive distinguishable entries,
    the damage (S-0302), resulting death effects (S-0312), nested detonations and draws of the
    earlier entry are observable before those of the later entry. Thus an earlier entry's death
    weapon can kill an object before its later entry, making that later entry ineffective for
    the object (claim 9); damage without death can instead change its remaining health.
    Within a sector, a newly tracked object follows every object already registered there.
    A tracked object crossing into another sector likewise follows every object already in its
    destination sector, regardless of when it first became tracked. Objects remaining in the
    old sector keep their relative order. Movement within the same sector leaves the order
    unchanged. Leaving and later returning to a sector puts the returning object after its
    current occupants. These rules apply to every tracked flying kind, including rocket missiles
    (S-0506 claims 15 and 19); the movement owner determines when tracking starts and ends.
    Thus a resident's earlier hit can trigger a death effect before a later-arriving object's
    outer hit, even when that arriving object first became tracked earlier than the resident.
    With `quirks.flying_search_sectors` off, the order instead follows the start of each object's
    current uninterrupted period of tracking, globally across all sectors. Sector crossings
    never change that order. Ending tracking removes the object from it; starting tracking
    again places the object after all currently tracked objects. This off-state rule is a
    design choice of the alternative search, not an observation of the original.
    - Backing: `measurable` (scenario A11, including variant d for newly placed objects, and
      scenario A20 for sector arrival, departure, return and same-sector ordering), and
      `unmeasured` for flying-object effects preceding scanned-cell effects and for the complete
      order between sectors: A11 and A20 do not compare flying and ground hits, and A16 tests
      membership, not every pairwise sector hit order. The extension of within-sector ordering
      to aircraft and rocket missiles is also `unmeasured`: A20 uses only jumpjets and supplies
      no corresponding kind-specific fixture. The off-state order is a design rule
      (A20's replay check).
    - Confidence: high for cells, hits and within-sector registration changes; medium for the
      complete order between sectors (F7)

### How much damage

12. **Falloff.** A hit with damage amount *D* > 0 at epicentre distance *d* (0 ≤ *d* ≤ *R*)
    first goes through the target's defences, which S-0302 owns (its claim 5): the defended amount
    *d*₃ is *D* divided by the armour and never below 1 (Terms). The falloff then works on *d*₃,
    not on *D*. Formulas F3 narrows the defended amount and its edge share to single precision
    (*D*ₛ and *E*), interpolates between those rounded quantities in double precision, truncates
    toward zero to give *D*₁, and floors the result at 0. When the rounded edge share *E* equals
    *D*ₛ, or *R* is 0, the hit carries the integer *d*₃ unchanged instead. Thus an edge fraction
    of exactly 1 bypasses interpolation; other fractions can do so too when the rounded quantities
    coincide. An edge fraction above 1 normally makes the damage rise towards the edge. The result goes on to the versus step of S-0302 (its claim 6): the original
    multiplies it by the versus value of the object's armour class, truncates toward zero and caps
    it at the game-wide maximum damage (F3, last line). S-0302 owns that step, every other
    multiplier and the minimum. So with a house and an object armour of 1.0, a target that is not
    prone and no veteran armour bonus, *d*₃ = *D*; in the other cases the order can matter, because *d*₃ is truncated
    before the falloff. Each hit passes its integer epicentre distance *d* in leptons, after
    claim 10's aircraft halving, as S-0302 claim 6's distance *r*. The falloff radius is F1's
    *R* = trunc(256 × *S*), not the distinct whole-cell radius used for the delayed-kill
    countdown of S-0302 claim 16.
    - Backing: `measurable` (scenario A1, variants a to d, g and h)
    - Confidence: high

13. **Healing detonations reach only the point itself.** A hit with damage amount *D* < 0 heals
    only an object whose epicentre distance is less than 8 leptons, by the full amount −*D*, with no
    falloff, no versus value and no cap; every other object of the spread gets nothing from it.
    (An object must also pass claim 9, so with *R* below 8 only objects at *d* ≤ *R* qualify.) How
    the healing changes health is S-0302's. Exception: an undemolishable building takes at least 1
    from every hit (S-0302 claim 7), so each of its hits from a healing detonation, at any distance
    up to *R*, costs it 1 health point instead.
    - Backing: `measurable` (scenario A7)
    - Confidence: high

14. **Friendly damage.** Area damage hits allies and the source house's own objects. The one
    exception is in damage resolution (S-0302 claim 8, the allies row of stage 2): when the warhead's
    affects-allies field is off and the hit carries an attacker, an object whose house is allied with
    the attacker's house takes no damage from the hit. What a detonation carries is S-0302 claim 1's
    source-object and source-house rule for its producer. For a projectile's detonation, the firing
    object and its owner are carried unless, between the shot and detonation, the firing object
    was killed, removed from the game by any other means, or started to cloak; in any of those
    cases neither is carried. A death-weapon detonation carries the dying object and its house even
    though the object is killed. A firer loaded into a closed transport is believed to remain
    carried despite being out of play; a firer that starts to cloak is believed to stop being
    carried. These cases retain the confidence qualifications of S-0302 claim 1. This spec adds
    nothing to the rule and has no switch for it.
    - Backing: `measurable` (scenario A8; S-0302 scenarios D17 g2–g5 and D24 b2–b5 for the
      death sequence, closed transport, death weapon and cloak); `unmeasured` for removal by
      another means, as S-0302 claim 1: no reproducible producer is identified
    - Confidence: medium for the killed-firer and death-weapon cases; low for closed loading
      and starting to cloak, as S-0302 claim 1

15. **Curtain protection nullifies a small detonation.** A small detonation becomes *nullified*
    when it finds a curtain-protected object (under the iron-curtain kind of invulnerability, not
    the force-shield kind) in either of these ways. In both, the epicentre distance is the one of
    claim 7, before the halving of claim 10:
    - a tracked flying object of claim 6 whose epicentre distance is below 85 leptons; like every
      candidate of claim 6 it must also be within *R* (for an aircraft in the air, at the halved
      distance of claim 10, as for every candidate).
    - an object of any kind in the centre cell whose epicentre distance is below 85 leptons. *R*
      plays no part here: with a spread radius below 85 leptons an object beyond *R* still
      triggers it. An object that claim 9 leaves out for being the source object, a listed
      harvester or not alive cannot trigger it.

    A nullified detonation hits only objects under the invulnerability effect, of either kind
    (whose protection then cancels the damage, S-0302), and all other objects in the spread take
    nothing. It also skips claims 18 to 20 and makes the projectile show the game-wide nullify
    animation instead of its explosion and debris (claim 21). Wall hits and resource reduction
    (claims 16 and 17) still happen in every scanned cell, as in a detonation that is not nullified
    (`quirks.curtain_nullifies_small_detonation`).
    - Backing: `measurable` (scenario A9)
    - Confidence: medium (the test and its effect are clear; the curtain and force-shield
      distinction is the superweapon specs')

### Cells, walls, bridges and overlays

16. **Walls.** For every scanned cell, in scan order and before any hit is delivered, a cell that
    holds a wall overlay takes a wall hit when the warhead has the wall flag, or has the wood flag
    and the wall's overlay type has the wood armour class (S-0305), or has the absolute
    wall-destroyer flag. The wall hit uses the full damage amount *D* (not 0, claim 2), with no
    falloff and no distance test, in every scanned cell. With the absolute wall-destroyer flag the
    wall hit is *absolute*: it always succeeds and removes the wall, whatever the sign of *D*
    (F4). A hit that is not absolute and has *D* < 0 never damages the wall but still takes a
    draw (F4). The one exception is *D* = −1: the original treats a wall hit of exactly −1 as
    absolute (`quirks.wall_hit_minus_one_is_absolute`). Formulas F4 gives the outcome, which
    agrees with S-0210 (claim 8, F3) for the hits on neighbouring walls; those always have the
    amount 200, so the quirk of −1 does not touch them.
    - Backing: `measurable` (scenario A10; the draws only statistically)
    - Confidence: high for which cells and the full amount; medium for the stage rules of F4

17. **Resource cells.** For every scanned cell, in scan order, a cell loses trunc(*D* / 10)
    resource units (nothing when the result is 0 or less) only if it holds a resource overlay
    whose type has the chain-reaction flag, the warhead has the resource flag, and the detonation
    producer allows resource reduction. Resource-explosion detonations produced by an animation's
    resource chain reaction forbid this reduction (S-0401 claim 20), so neighbouring scanned
    resource cells lose nothing through this rule and make no spread-offer draws through it.
    The chain reaction's own emptying of its source cell remains S-0401's rule. Crate gas
    detonations also forbid resource reduction (S-0520 claim 23). Crate explosion and napalm
    area damage allow it (S-0520 claims 16 and 17). The other producers named here allow
    reduction. Craters remove resources by a rule of their own (S-0109 claim 18).
    - Backing: `measurable` (scenario A10, variant d, with a test overlay type; S-0401 scenario
      E17 and A10 variant h for the resource-explosion exception; S-0520 scenarios CR12 and
      CR17 for crate permissions), and `unmeasured` for resource permission of the remaining
      producers: these scenarios do not test each death, crash, damaging animation, superweapon
      and trigger producer
    - Confidence: high for the resource-explosion and crate permissions; medium for the remaining producers

18. **Bridges.** When the bridge-destruction game option is on and the warhead has the wall
    flag, a detonation may make destroy attempts at its centre cell. In a skirmish the setup
    screen's option replaces the map's destroyable-bridges setting (S-0511 claim 1).
    Eligible tests and their chance draws follow this order: high bridge-role family, high
    wooden-bridge-role family, low wooden-bridge-role family, low bridge-role family. S-0511
    claim 1 defines membership: a high test accepts the family's section on a high-bridge cell
    or its pier in state 0 to 3; a low test accepts only a low piece not yet destroyed.
    Only a cell marked as high bridge imposes a height window: *P* must lie above
    (*h* − 2) × 104 + 416 and at or below (*h* + 1) × 104 + 416 leptons, where *h* is that
    cell's integer height level. The expressions use exact integer arithmetic. A pier cell
    without that marking, like a low-bridge cell, has no height exclusion.
    Each eligible test passing the height condition takes its own ranged draw *r* over 1 to
    the game-wide bridge strength (inclusive), and makes an attempt exactly when *r* < *D*.
    The game-wide ion-cannon warhead bypasses those chance draws and always makes the attempt.
    For a high test only, it follows a failed attempt with further attempts at the same cell,
    with no new chance draw, up to four attempts in all, stopping at the first success.
    A low test makes just one attempt, including for that warhead. Other warheads make one
    attempt per successful chance test. An attempt's bridge changes, occupant effects and
    resulting draws are those of S-0511 claims 1 to 15; a failed attempt can still damage a
    piece, so the high-test retries can finish a destruction that one attempt leaves unfinished.
    - Backing: `measurable` (scenario A12 for chance, high-only repeats, height and skirmish
      gate; S-0511 scenarios B1, B7 and B11 for membership); `unmeasured` for the ordering of
      multiple eligible tests on one cell: no established fixture distinguishes every pair
    - Confidence: medium

19. **Exploding overlays.** When the centre cell holds an overlay whose type has the explodes flag
    (such as a barrel), that overlay is removed, the game-wide barrel-explosion animation appears at
    *P*, and a nested detonation follows at *P* with the game-wide ammunition-crate damage as its
    damage amount, the game-wide demolition warhead, no source object and the outer detonation's
    source house. After the nested detonation, at most one barrel debris piece appears, and
    then possibly the barrel particle system. With *n* the length of the barrel debris list and
    percent draws *r*₀, *r*₁, … (each a ranged draw over 0 to 99) belonging to its entries in list
    order, the debris piece is of the type of entry *i*\*, the least *i* with *r*ᵢ < 15, and the draws
    taken are *r*₀ … *r*ᵢ\*, that is *i*\* + 1 draws; when no *r*ᵢ is below 15, all *n* draws are
    taken and no debris appears. The barrel debris list names voxel-model animation types, so the
    debris is a piece of S-0317, an object of play created at *P*: its creation takes the draws of
    S-0317 claim 3 (or claim 4 for a type with the meteor flag) right after *r*ᵢ\*, and what it
    does afterwards is S-0317's. One further percent draw *r* follows, and the game-wide barrel
    particle system appears at *P* when *r* < 25. This happens whatever the sign of *D*. Only the centre
    cell is affected, so a barrel next to the centre cell is set off only by its own detonation
    chain. This has no switch, as decided at the delegated sign-off of 2026-10-10.
    - Backing: `measurable` (scenario A13; variant d for the piece's creation draws between *r*ᵢ\*
      and *r*)
    - Confidence: high for the trigger and the nested detonation; medium for the draws. The
      medium for the barrel debris piece's creation draws is S-0317's: claims 3 and 4 there rate
      the draw count medium until S-0317 scenario V1 runs, and also rate medium the order among
      those draws. The place of those creation draws, after *r*ᵢ\* and before *r*, is medium until
      A13 variant d runs

20. **Particle systems and rocking.** A warhead that names a particle system creates it at *P* at
    the end of the detonation, owned by the source house. When the system has a held particle type,
    it immediately receives one particle of that type at *P*, subject to the terrain floor of S-0613
    claim 5; without a held type no initial particle appears (S-0613 claim 4). The system takes its
    type's lifetime, whose absent default is −1 ticks, meaning it never times out (S-0613 claim 4).
    This is the system's lifetime, not the particle's. A warhead with the rocking flag rocks the
    bodies of the objects drawn as voxel models (of any kind) in the 7 × 7 cells around the centre
    cell, in the same cell layer, when the rocking strength min(*D* × 0.01, 4.0) exceeds 0.3. The
    strength is formed from the integer *D* and rounded as S-0113 F2 sets; whatever the rounding, it
    exceeds 0.3 exactly when *D* ≥ 31, and it reaches its cap of 4.0 at *D* ≥ 400. Objects in the
    centre cell are pushed away from the source object when there is one. The others, and the centre
    cell's objects when there is no source object, are pushed away from *P*, and only when the
    spread *S* (Terms) is above 0. Rocking sets only the body-rocking rates of S-0113 (divided by
    the type's weight) and takes no draw. The rocked angles reach game state as S-0113 claim 19
    decides (S-0113 claims 9, 11 and 19).
    - Backing: `measurable` for the particle system (scenario A13, variant c; S-0613 scenario G3 for
      the initial particle, absent held type and system lifetime), S-0113 scenario RK1 for the
      rocking push's direction and strength (its damage amount whose strength is not exact in single
      precision for the rounding; its part d for a centre-cell object when there is no source
      object), and `unmeasured` for the rocking threshold, the 7 × 7 area and the cell-layer limit:
      no scenario varies them. `unmeasured` that rocking takes no draw: no scenario records the
      scenario random stream around a rocking detonation
    - Confidence: high; medium for the rocking threshold, the 7 × 7 area and the cell-layer limit,
      and for rocking's taking no draw; medium for the angles' reaching game state, as S-0113 claim
      19 rates it, and for the weight's division of the push on infantry and buildings drawn as
      voxel models, as S-0113 claim 10 rates it

### Detonation effects of a projectile's warhead

21. **Explosion animation.** After a projectile's area damage, its warhead's explosion animation is
    chosen by Formulas F5 from its then-current damage amount (S-0306 claim 21, including
    degeneration under claim 18 there), the warhead's animation list and
    the land type *L* of the projectile position *Q* (Terms), and created at the detonation point
    *P*. *L* is the land type of the cell that contains *Q* when the projectile's height above the
    terrain is below 208 leptons, and "air" (no land type) otherwise. Over water the splash list
    replaces the warhead's own list (F5), except when the area damage hit something and the first
    object in the ground layer of the cell of *Q* is a naval vehicle that is not an underwater type
    (Terms): then *L* counts as air and the warhead's own list is used. The bridge and height tests
    of F5 are taken at *Q* too. For a projectile of the invisible kind (S-0306) the animation is
    placed at the nearby point N(*P*, 32, *b*) of S-0312 (F2), with *b* = *u* mod 256 for one raw
    value *u* of the scenario random stream (the non-negative remainder, 0 … 255, whether *u* is
    read as signed or unsigned) and no snapping to the cell centre, instead of at *P*. On the
    scenario random stream that raw value comes after every draw of the area damage and before the
    draw of F5. An electromagnetic warhead takes the one draw of F5's electromagnetic case whenever
    F5 reaches that case with a non-empty animation list, whatever the result of the area damage,
    so also when the detonation was nullified. When the area damage was nullified (claim 15), the
    game-wide nullify animation appears instead of the chosen one (at the same point), and claim 22
    does not happen. If the projectile is no longer in play after its area damage, none of this and
    nothing of claim 22 happens (S-0306 says when a projectile leaves play).
    Crate explosions also use F5 for their blast animations (S-0520 claim 16), with their
    supplied damage amount and demolition warhead, but always with a non-water land type:
    even a blast over water never selects the splash case. This use of F5 does not imply a
    projectile, its invisible-kind scatter, its nullify-animation substitution or its debris.
    - Backing: `measurable` (scenario A14; S-0520 scenario CR12 for crate blast selection; variant e for the invisible projectile, through the
      recorder's stream position; variant h for the draw of a nullified electromagnetic
      detonation; variant i for *Q* against *P*); `unmeasured` for the projectile that leaves play
      (how that can happen is not established)
    - Confidence: high for F5; medium for the land-type, naval, *Q* and invisible-projectile rules,
      low for the projectile that leaves play

22. **Warhead debris.** A warhead with a positive maximum debris count throws debris when a
    projectile detonates: Formulas F6 gives the count and the types, all created at the projectile
    position *Q* (the metallic debris of F6 raised by 20 leptons), not at *P*. This happens
    whatever the damage amount, on an inert map as well, and not when the detonation was nullified
    (claim 21). The voxel-model debris that F6 gives consists of pieces of S-0317: objects of
    play, created at *Q* with the draws of S-0317 claims 3 and 4, whose flight, impacts and end
    are S-0317's.
    - Backing: `measurable` (scenario A15, statistical; variant c for the position), and
      `measurable` for signed maxima and a zero allocation divisor (A19 variants b and c), and
      `unmeasured` for F6's signed-magnitude exceptional raw value and its overflowing quotient:
      the setup cannot select the seed to make that one value in 2³² occur, as S-0202 claim 6
      explains. The safe results of arithmetic departures are design rules, not oracle values
    - Confidence: high for the count draw; medium for the distribution over types and the
      position; medium for the pieces' creation draws, as S-0317 claim 3 rates them until its
      scenario V1 runs

23. **Result of a detonation.** A detonation reports one of three results to whatever produced it:
    *hit* (at least one object received a hit), *missed* (none did, or claim 2 applied) or
    *nullified* (claim 15). The projectile detonation uses it in claim 21; other producers say in
    their own specs whether they use it. A disc laser's detonation (S-0615 claim 16) applies
    only the area damage of claims 1 to 20 and ignores its result. It produces none of the
    projectile-only effects of claims 21 and 22: no explosion animation, warhead debris or
    nullify animation, and no animation-selection draw of F5. It also produces no light flash.
    The area damage's own effects, including an exploding overlay's animation and the
    warhead's particle system, still follow their rules above. The end of a debris piece of
    S-0317 (claim 11 there) is a second such producer: its expiry damage applies the area damage
    of claims 1 to 20 and none of the projectile-only effects of claims 21 and 22. Only a piece
    whose type names an expiry animation, and that ends not on water or on water at deck height,
    makes this area damage, as S-0317 claim 11 gives its cases. Unlike the disc laser, it also
    makes the warhead's light flash when the warhead has one. Both happen also when the type's
    damage amount is 0 (S-0317 claim 11); the area damage then has no effect (claim 2).
    - Backing: `measurable` through claim 21 (scenario A14, variants b and c), and S-0615
      scenario DL7 for disc-laser area effects and absence of projectile-only effects; S-0317
      scenario V8 for a piece's expiry: its runs with and without an expiry animation (the area
      damage only with one, the flash and the warhead's particle system, and no explosion
      animation of the warhead's list and no warhead debris of claim 22, V8 using a falling test
      type). The area damage and the light flash at a damage amount of 0 are `unmeasured`: the
      area damage then has no effect to observe (claim 2), and the flash inherits S-0317 claim
      11's rating and reason for that case
    - Confidence: high, also for a piece's end; medium for which ends on water make the area
      damage (at deck height or not), as S-0317 claim 11 rates its meteor water row and the exact
      deck height; medium for the area damage and the light flash at a damage amount of 0, which
      S-0317 claim 11 leaves `unmeasured`. Claim 23 asserts nothing else of a piece's end: the
      order of its expiry animation and area damage, the area damage's lack of a source object and
      its effect on resources keep S-0317 claim 11's own ratings

24. **Defaults of bridge and barrel damage.** At a fresh program start the game-wide bridge
    strength is 1000 bridge-strength points and the ammunition-crate damage is 100 damage points.
    Omitting either field from a rules layer keeps its value in force, including a value left by
    an earlier game of the same session (S-0006). Neither omission disables the corresponding
    effect: claim 18 still tests bridge destruction, and claim 19 still produces its nested
    detonation with that damage amount. The damage is an integer delivered amount, before the
    demolition warhead's falloff and the target's damage resolution.
    - Backing: `measurable` (scenario A17)
    - Confidence: high

25. **Animation and debris list items.** S-0003 claim 19 owns their shared parsing rule. In this
    spec's explosion-animation, splash, barrel-debris,
    metallic-debris and warhead voxel-debris lists, an item naming no type is omitted, not kept as
    an empty place. The no-type spellings are given in Import mapping. The remaining items retain
    their relative order. Every list length and entry number in claims 19, 21 and 22 and F5 and F6
    is taken after those omissions. Thus an omitted first item does not prevent the next animation
    from being selected at entry 0, does not enlarge a random choice's range and takes no barrel
    debris chance draw. Empty comma-separated items are also skipped, as S-0003 claim 12 says.
    - Backing: `measurable` (scenario A18)
    - Confidence: high

26. **Debris maxima are decimal-prefix items.** S-0003 claim 20 owns their shared parsing rule.
    Each non-empty item in the warhead's maxima list
    contributes one signed 32-bit integer: leading whitespace, an optional sign and decimal digits
    are read up to the first non-digit. An item with no readable decimal digits contributes 0
    and keeps its place. Hexadecimal prefix and suffix forms of a scalar integer field do not
    apply. Empty comma-separated items contribute no place. Arithmetic of the conversion follows
    S-0003's decimal integer form, including its overflow rule. The types list and maxima list
    are read independently: dropping a no-type item from the types does not drop the corresponding
    maxima item. F6 pairs the first retained type with the first maxima item, and so on.
    - Backing: `measurable` (scenario A19); `unmeasured` for the separate 511-byte value-reader
      limit: S-0001 claim 3's physical-line limit cuts the value sooner once the key and separator
      are counted, as S-0003 claim 20 explains
    - Confidence: high

## Formulas

**Notation.** RZ₂₄ and RZ₅₃ are S-0205's roundings toward zero to single and double precision,
trunc is truncation toward zero to an integer (S-0205 claim 7). In the original every floating-point
step below is rounded toward zero (the round-toward-zero mode, S-0205 claim 5); with
`quirks.chop_rounding` off, RN₂₄ and RN₅₃ replace RZ₂₄ and RZ₅₃. This spec defines no switch of its
own for the rounding mode.

**F1. Spread radius** (claim 3).

> *R* = trunc(256 × *S*)

*S* is single precision; the product is exact. Unit: leptons. Integer result.

**F2. Scan radius** (claim 3).

> *k* = trunc(RZ₅₃(*S* + *c*)), *c* = the double nearest 0.99 (just below 0.99)

Examples: *S* = 1.0 gives *k* = 1; *S* = 1.01 (single) gives 1; *S* = 1.02 gives 2; *S* = 0.01 gives
0; *S* = 0.02 gives 1. No single-precision *S* lies close enough to a whole number minus *c* for the
rounding mode to change *k*. Unit: cells (ring metric). Valid range: 0 ≤ *k* ≤ 11 (Edge cases).

**F3. Falloff** (claim 12). *D* is the amount the detonation delivers. For a hit with *D* > 0, the
input is the defended amount *d*₃ of S-0302's F2 (equal to *D* when the house and the object
armour are both 1.0, the target is not prone and has no veteran armour bonus, and possibly different
otherwise). At epicentre distance *d* (an integer,
0 ≤ *d* ≤ *R*):

> *D*ₛ = RZ₂₄(*d*₃)  (exact for *d*₃ ≤ 2²⁴)
>
> *E* = RZ₂₄(RZ₅₃(*d*₃ × *p*))
>
> *D*₁ = trunc(RZ₅₃(RZ₅₃(RZ₅₃(RZ₅₃(*D*ₛ − *E*) × (*R* − *d*)) / *R*) + *E*)) when *E* ≠ *D*ₛ and
> *R* ≠ 0, and
>
> *D*₁ = *d*₃ when *E* = *D*ₛ or *R* = 0.
>
> *D*₂ = max(*D*₁, 0)
>
> versus step (S-0302 F3): *m*₀ = min(trunc(RZ₅₃(*D*₂ × *v*)), *M*)

*p* is the edge fraction (single), *v* the versus value of the hit object's armour class (a double,
S-0302 claim 6) and *M* the game-wide maximum damage. (*R* − *d*) and *R* are exact integers
in leptons; *R* is F1's spread radius and *d* includes claim 10's halving. This is the same
falloff and versus computation owned by S-0302 claim 6: its *f*, *a*, *g* and *r* correspond to
*D*ₛ, *E*, *D*₁ and *d* here. Both single-precision narrowings are required, but the edge product
uses the integer *d*₃, not the already narrowed *D*ₛ. The interpolation's arithmetic uses the
double-precision roundings shown above. Damage quantities are in health points.
S-0302 claim 6 uses the same integer *d*₃ for its edge share. Using *f* instead would make
large defended amounts disagree by a health point.
Example with exact arithmetic (no rounding at these values), for *d*₃ = 100, *p* = 0.5, *R* = 512:
*d* = 0 gives 100, *d* = 256 gives 75, *d* = 511 gives 50, *d* = 512 gives 50. The order of the
steps matters: with *D* = 1000, an armour divisor of 3 (*d*₃ = 333), *p* = 0.5 and *R* = 1024, the
falloff at *d* = 512 gives 249; applying it to 1000 first (750) and dividing afterwards would give
250.

For *D* < 0 (claim 13): *D*₁ = *D* when *d* < 8, else 0; the defences leave the amount unchanged
(S-0302 claim 5) and there is no versus step and no cap.

**F4. Wall hit** (claim 16). Let *L* be the wall type's damage-stage count, *T* its wall strength, *s* the cell's damage stage and *v* its overlay data value, with *s* = ⌊*v* / 16⌋ (S-0210 claim 8). Let *a* be the hit amount: *a* = *D* for a non-absolute hit, except that *a* = −1 makes the hit absolute when `quirks.wall_hit_minus_one_is_absolute` is on. An absolute hit always succeeds; otherwise a hit with *a* ≥ *T* succeeds without a draw, and a hit with *a* < *T* succeeds exactly when one ranged draw *r* over 0 … *T*, inclusive, gives *a* > *r*. The draw is still taken for a negative *a*. For 0 ≤ *a* < *T*, the success probability is *a* / (*T* + 1); a negative *a* other than the absolute −1 case cannot succeed.

The state and effects of a successful hit are:

| Result of successful hit | Behaviour |
|---|---|
| Overlay data value and damage stage | *v* increases by 16; the resulting stage is ⌊*v* / 16⌋, one greater than the prior stage. |
| Removal | The wall is removed when the hit was absolute, the resulting stage is at least *L*, or the resulting stage is *L* − 1 and *v* mod 16 = 0. A wall at stage *L* − 1 that still links to a neighbour remains until a further successful hit. |
| Neighbour hits | If the resulting stage is *L* − 1 and *L* > 2, each orthogonal neighbour, in map-north, map-east, map-south, map-west order, that holds a same-type wall at stage 0 takes a wall hit of 200. Each neighbour hit follows these same rules and may take its own draw. |

Comparisons and state values use exact integer arithmetic. Units: *D*, *a* and *T* are wall-hit points; *r* is a ranged-draw result; *s* and *L* are damage stages; *v* is overlay data value, in increments of 16 per stage.

**F5. Explosion animation** (claim 21). Let *D* be the producer's supplied damage amount,
*W* the warhead, *L* the land type (water, another land type, or none for air), and
*Q* = (*q*ₓ, *q*ᵧ, *q*_z) the position used for the bridge and height tests. For projectiles,
claim 21 supplies their then-current damage amount, land type and projectile position. For
crate explosions (S-0520 claim 16), *Q* is the blast point and *L* is always non-water. With *D* = 0 there is no animation and no draw. Otherwise the
animation is given by the first case of this table whose condition holds; *n*ₛ is the length of the
game-wide splash list and *n*_W the length of *W*'s animation list.

| Case | Condition | Animation | Draws |
|---|---|---|---|
| Splash | *L* is water, *W* has the conventional flag, the cell of *Q* carries no bridge, and *q*_z < (terrain height at *Q*) + 208 | none when *n*ₛ = 0, else entry trunc(min(*D*, 35*n*ₛ − 1) / 35) of the splash list | none |
| Lightning | *W* is the game-wide lightning warhead | the game-wide lightning-bolt explosion animation | none |
| Electromagnetic | *W* has the electromagnetic flag | none when *n*_W = 0, else entry R(0, *n*_W − 1) of *W*'s list | one ranged draw when *n*_W > 0, none when *n*_W = 0 |
| Graded | any other | none when *n*_W = 0, else entry trunc(min(*D*, 25*n*_W − 1) / 25) of *W*'s list | none |

A case that gives no animation still decides the result: a splash case with an empty splash list
shows nothing and does not fall through to *W*'s own list.

Integer division truncated toward zero; entries counted from 0. For −24 ≤ *D* ≤ −1 (−34 ≤ *D* ≤ −1
for the splash list) the entry is 0, a valid entry. For smaller *D* the entry is negative, which
lies outside the list (Edge cases).

**F6. Warhead debris** (claim 22). With *X* the warhead's maximum debris count (*X* > 0) and *Y* its minimum debris count, the total number of pieces is *N* = R(*Y*, *X* − 1), one ranged draw whose bounds may come in either order (S-0202 claim 4). A non-positive *N* creates no pieces and takes no allocation or type-choice draws. This non-positive-total case is `unmeasured`: A15 and A19 use positive totals, not a setup producing a non-positive total. Confidence: high. All debris is created at the projectile position *Q* (claim 22).

For a nonempty retained type list of *m* types, let *t*ᵢ and *u*ᵢ be the type and signed maximum at index *i*, with indices counted from 0. The initial remaining count is *N*. Each allocation uses one scenario-random-stream value *raw*. The allocation state is:

| Allocation state | Selected type | Remaining count before allocation | Random value | Allocation and next state |
|---|---|---|---|---|
| Initial, with *N* > 0 | *t*₀ | *N* | One *raw* value | Apply the quantity table below. |
| Following allocation | The type after the previously selected type in list order, wrapping from *t*ₘ₋₁ to *t*₀ | The preceding allocation's resulting remaining count | One new *raw* value | Apply the quantity table below only when the remaining count is positive. |

The quantities determining an allocation and the remaining count are:

| Quantity | Meaning |
|---|---|
| *a* = wrap(|*raw*|) | Magnitude quantity, with the signed 32-bit wrap described below |
| *b* = wrap(*u*ᵢ + 1) | Divisor associated with the selected type |
| *g* = min(*a* rem *b*, *ℓ*) | Number assigned to the selected type, capped by the currently remaining count *ℓ* |
| New remaining count | wrap(*ℓ* − *g*) |

“Wrap” is S-0205 claim 1's signed 32-bit wrap. The remainder uses signed division truncated toward zero (S-0205 claims 2–3), so its sign follows *a*, not *b*. This is the magnitude-first rule of S-0202 claim 6, extended here to signed divisors. Thus a maximum below −1 normally behaves like the non-negative maximum −*u* − 2: −2 assigns zero pieces, while −4 assigns 0 to 2 pieces. A maximum of 2147483647 gives the wrapped divisor −2147483648, not a zero divisor. A maximum of −1 gives a zero divisor, and its departure is specified under Edge cases. For the exceptional raw value −2147483648, *a* stays negative under the default of S-0202 claim 6; signed remainder and the wrap of the remaining count still apply, and a non-positive allocation creates no pieces. With `quirks.random_magnitude_overflow` off (S-0202 claim 6), *a* is instead the exact magnitude |*raw*|, including 2147483648; the remainder is then non-negative. The magnitude and its quotient may use that wider non-negative integer domain, while the divisor and remaining count retain signed 32-bit wrap.

The allocation's result contributes max(*g*, 0) pieces of its selected type. Debris allocation ends when the resulting remaining count is 0 or less; the random value used for that allocation is included. Pieces assigned by an allocation are created before the random value for the next allocation is drawn. Draws caused by creating pieces occur between allocation values and are not themselves allocation values. If no allocation reduces the remaining count to 0 or less, the non-terminating case is described under Edge cases: debris that can never be placed.

If the warhead lists no debris types, the detonation creates *N* animations, each selected by
entry R(0, *c* − 1) of the game-wide metallic debris list (*c* its length), at *Q* raised by
20 leptons. Arithmetic is exact integer arithmetic, with the signed-magnitude exception owned
by S-0202 claim 6; counts are pieces and the position raise is in leptons.

**F7. Flying-object search** (claims 6 and 11). Let *W* and *H* be the map's width and height
(S-0009) and *B* = *W* + *H* − 1, the largest cell coordinate of the map along either axis. Sector
size, the same along both axes, and the sector of a cell (*x*, *y*):

> *w* = *h* = trunc(*B* / 20)
>
> σₓ(*x*) = min(trunc(max(*x*, 0) / *w*), 19), σᵧ(*y*) = min(trunc(max(*y*, 0) / *h*), 19), and
> σ(*x*, *y*) = (σₓ(*x*), σᵧ(*y*))

Integer division truncated toward zero; a sector is a pair (*i*, *j*) with 0 ≤ *i*, *j* ≤ 19, and
"the sector (*i* + *a*, *j* + *b*)" below always means that pair with each coordinate clamped to 0 …
19. For *B* < 20 the sector size is 0 (Edge cases, departure). With (*c*ₓ, *c*ᵧ) the centre cell
and the search radius ρ = max(trunc(*S*), 1) cells, let *C* = (*C*ₓ, *C*ᵧ) = σ(*c*ₓ, *c*ᵧ).

There are four *runs*, one along each map axis direction from *C*. A run is the straight line of
sectors from its *far sector*, the sector of the cell ρ cells from the centre cell in that
direction, inwards to the neighbour of *C* on that side, taken in that order (farthest first). Its
*count* is the number of sectors in it:

| Run | Direction from *C* | Far sector | Count |
|---|---|---|---|
| east | map east (increasing *x*) | (σₓ(*c*ₓ + ρ), *C*ᵧ) | *e* = σₓ(*c*ₓ + ρ) − *C*ₓ |
| north | map north (decreasing *y*) | (*C*ₓ, σᵧ(*c*ᵧ − ρ)) | *n* = *C*ᵧ − σᵧ(*c*ᵧ − ρ) |
| south | map south (increasing *y*) | (*C*ₓ, σᵧ(*c*ᵧ + ρ)) | *s* = σᵧ(*c*ᵧ + ρ) − *C*ᵧ |
| west | map west (decreasing *x*) | (σₓ(*c*ₓ − ρ), *C*ᵧ) | *u* = *C*ₓ − σₓ(*c*ₓ − ρ) |

A run with a count of 0 (its far sector is *C* itself) is empty. There are also four *diagonal groups*, one per quarter: for the
quarter of two runs with counts *a* and *b* and diagonal step (*d*ₓ, *d*ᵧ), the min(*a*, *b*)
sectors *C* + *k*(*d*ₓ, *d*ᵧ) for *k* = 1, 2, …, each clamped as above, and, when *a* > 1 and
*b* > 1, the two sectors *C* + (2*d*ₓ, *d*ᵧ) and *C* + (*d*ₓ, 2*d*ᵧ). The quarters are, in this
order: north and west with (−1, −1), south and west with (−1, +1), north and east with (+1, −1),
and south and east with (+1, +1).

The ordered sector set consists of *C*, followed by the nonempty east, north, south and west runs, then the four diagonal groups, in the orders defined above. Within each diagonal group, sectors follow the stated outward-diagonal order and then the stated order of its two extra sectors. Every tracked flying object registered in one of these sectors is a candidate exactly once. Candidates in different sectors follow the sector order; candidates within one sector follow arrival order as defined by claim 11, not the order in which they first became tracked. No tracked flying object registered in a sector outside this set is a candidate under this search. For example, (−3, −1) from *C* is not searched when every run is at least 3 long. Units: cells, sectors.

With `quirks.flying_search_sectors` off, the candidates are every tracked flying object, each
once, in the global order of starting their current uninterrupted tracking periods (claim 11).
Crossing a sector boundary does not change this alternative order.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Spread scale | 256 | leptons per cell of spread | F1 |
| Scan rounding addend | the double nearest 0.99 | cells | F2 |
| Largest scan radius | 11 | ring metric | table T1; above it is a departure |
| Small detonation limit | 0.5 (inclusive) | cells of spread | claim 15 |
| Curtain nullify distance | 85 (exclusive) | leptons | claim 15 |
| Deck test height | 208 (exclusive) | leptons above the terrain | claim 4; half of the 416-lepton deck height |
| Raised-detonation building allowance | 208 | leptons | claim 7 |
| Height level | 104 | leptons | claim 18 (glossary) |
| Healing reach | 8 (exclusive) | leptons | claim 13 |
| Resource reduction divisor | 10 | damage per resource unit | claim 17 |
| High-test repeats for the ion-cannon warhead | 3 | further attempts | claim 18; scenario A12 variant b; low tests never repeat |
| Neighbour wall hit | 200 | wall-hit points | F4, as S-0210 |
| Stage step in the overlay data value | 16 | data value per stage | F4, as S-0210 |
| Barrel debris chance per entry | 15 | percent (draw < 15) | claim 19 |
| Barrel particle chance | 25 | percent (draw < 25) | claim 19 |
| Rocking strength factor | 0.01 | strength per damage point | claim 20 |
| Rocking strength cap | 4.0 | strength | claim 20 |
| Rocking strength threshold | 0.3 (exclusive) | strength | claim 20 |
| Rocking area | 7 × 7 | cells around the centre cell | claim 20 |
| Splash list step | 35 | damage per entry | F5 |
| Animation list step | 25 | damage per entry | F5 |
| Air land-type height | 208 (exclusive) | leptons above the terrain | claim 21 |
| Metallic debris raise | 20 | leptons | F6 |
| Wall hit amount treated as absolute | −1 | wall-hit points | claim 16, F4 |
| Sectors per map axis | 20 | sectors | F7 |
| Smallest flying-search radius | 1 | cells | F7 |
| Smallest map size with a sector size above 0 | 20 | cells (*B*) | F7; below it is a departure |
| Invisible-projectile animation scatter | 32 | leptons | claim 21 |
| Edge fraction, absent-key default | 1.0 | fraction | Import mapping |
| Spread, absent-key default | 0 | cells | Import mapping |
| Bridge strength, initial default | 1000 | bridge-strength points | claim 24 |
| Ammunition-crate damage, initial default | 100 | damage points | claim 24 |

**Table T1. Scanned cells and scan order.** Ring *r* is the set of cells displaced (Δ*x*, Δ*y*) from
the centre cell with *m*(Δ*x*, Δ*y*) = *r*. A detonation with scan radius *k* scans rings 0, 1, …, *k*
in that order. The number of cells scanned for *k* = 0 … 11, counting the cell that ring 11 scans
twice as two, is 1, 9, 21, 37, 61, 89, 121, 161, 205, 253, 309, 369 (368 distinct cells for
*k* = 11).

- Rings 0 and 2 to 10: the cells of the ring in order of increasing Δ*y*, and within one Δ*y* in
  order of increasing Δ*x*.
- Ring 1: (1, −1), (0, −1), (−1, −1), (−1, 0), (1, 0), (−1, 1), (0, 1), (1, 1). Only its first row
  is in decreasing Δ*x*.
- Ring 11 (60 entries, not the cells of metric 11; `quirks.spread_ring_eleven`): (0, 11), (0, −11),
  (−1, 11), (1, 11), (−1, −11), (1, −11), (−2, 11), (2, 11), (−2, −11), (2, −11), (−3, 11),
  (3, 11), (−3, −11), (−3, 11), (−4, 9), (4, 9), (−4, −9), (4, −9), (−5, 9), (5, 9), (−5, −9),
  (5, −9), (−6, 8), (6, 8), (−6, −8), (6, −8), (−7, 8), (7, 8), (−7, −8), (7, −8), (−8, 7), (8, 7),
  (−8, −7), (8, −7), (−8, 6), (8, 6), (−8, −6), (8, −6), (−9, 5), (9, 5), (−9, −5), (9, −5), (−9, 4),
  (9, 4), (−9, −4), (9, −4), (−10, 3), (10, 3), (−10, −3), (10, −3), (−10, 2), (10, 2), (−10, −2),
  (10, −2), (−11, 1), (11, 1), (−11, −1), (11, −1), (11, 0), (−11, 0). The cell (−3, 11) appears
  twice, so it is scanned twice: its wall overlay takes two wall hits (claim 16), its resource
  overlay is reduced twice (claim 17), and each object there that passes claim 9 is a candidate
  twice, so it takes two hits, at the two places of the order (claim 11; the second does nothing
  if the first destroyed it, with claim 8's delayed-kill exception). Cells that are not in the list: (3, −11) and the metric-11 cells
  (±2, ±10) and (±3, ±10). The list also holds seven cells of metric 12: (±2, ±11), (−3, 11),
  (3, 11) and (−3, −11).

A scanned cell outside the map has no contents and takes part in nothing.

## State and ordering

**Observable state and draws within one detonation** (claims 1, 11 and 15 to 20). The following
outcomes can be distinguished through the random-state recorder and nested detonations:

- By the time the first object hit is observable, the resource reductions and wall hits for every
  scanned cell have already taken effect (claims 17 and 16). The recorded wall-hit draws (F4),
  including draws for any neighbour hits they set off (S-0210 F3), occur before the first
  object-hit draw and match T1 scan order.
- An object that first appears in a scanned cell while this detonation's hit effects are
  occurring receives no hit from this detonation (claim 11).
- For successive entries in claim 11's observable order, the damage resolution, death effects,
  nested-detonation effects and recorded draws of one entry are observable before those of the
  next.
- A nullified detonation (claim 15) has no later bridge draw, exploding-overlay effect, warhead
  particle system or rocking.
- For a detonation that is not nullified, the recorded applicable bridge-test draws (claim 18:
  one ranged draw for each applicable test, none for the ion-cannon warhead) occur after
  hit-related draws and before centre-cell exploding-overlay effects. The overlay's nested
  detonation and percent draws (claim 19) occur before creation of the warhead's particle system
  (claim 20).
- Rocking (claim 20) takes no draw (`unmeasured`, medium, as claim 20 rates it). A detonation's
  rocking comes after all of its object hits, with their damage resolution, death effects and the
  nested detonations those make, and before its bridge-test draws, its centre-cell exploding-overlay
  effects and its particle system. Since only the last push before a body's turn counts (S-0113
  claim 2), a body that this detonation and a nested detonation from a death both push keeps this
  detonation's push, and one that the exploding overlay's nested detonation also pushes keeps that
  one's, in each case when the later push changes the rates: a push that changes nothing leaves the
  earlier rates (S-0113 claims 2 and 3). These last-push outcomes rest on this place, which is
  `unmeasured` (medium): no scenario here pushes one body by two nested detonations. The
  resource-before-object timing above and the post-hit ordering of bridge, overlay and particle
  effects are `unmeasured` except for the wall-draw ordering tested by A11 variant e: no scenario
  combines those other effects with an identifiable object-hit draw to distinguish their order.
  Confidence: medium.

**Order of a projectile's detonation** (claims 21 to 23, with S-0306 for the rest). The random-state
recorder sees the draws of a projectile's detonation on the scenario random stream in this order:

| Order | Source of the draws | Draws on the scenario random stream |
|---|---|---|
| 1 | the effects that the special-warhead specs own (radiation, mind control and the like, which can replace the area damage) | as their specs say |
| 1a | the fragments of S-0318 (claims 3 and 4), for a projectile with a fragment weapon, when S-0318 claim 1 lets it burst | for choosing the fragments' targets: two ranged draws per fragment sent to a random cell, none for a fragment sent to an object (whether a fragment's launch takes scatter draws is open in S-0318) |
| 2 | the area damage above | as in *Order inside one detonation* |
| 3 | the nearby point of claim 21, for a projectile of the invisible kind only | one raw value |
| 4 | the choice of the explosion animation (F5) | as F5: one ranged draw when F5 reaches its electromagnetic case with a non-empty list, whatever the result of row 2, nullified included; none otherwise |
| 5 | the warhead debris (F6), only when the table below creates it | as F6 |

The table covers one detonation. After each detonation that is not an airburst, S-0306 claim
21's ranged draw R(256, 512) and one raw value follow (S-0318 State and ordering). Row 1a is
numbered so that the other rows keep the numbers other specs cite. Row 1a and the sentence on
S-0306 claim 21's draws after a detonation are cross-references: their backing and ratings are
those of S-0318 (claims 3 and 4, State and ordering) and S-0306 claim 21, and this spec adds none
of its own.

Before row 1, the warhead's screen-shake ranges take one ranged draw each, horizontal then vertical,
for each range that is not zero at both ends, from the unsynchronised random stream, so they reach
no game state and do not shift the scenario random stream. Backing: `measurable` (A14 variant n
for stream, count and order); confidence: high for those draws. Their never reaching game state
is `unmeasured`, confidence medium: no scenario compares later gameplay with and without shake.
The result of the area damage decides
what is created:

| Detonation result | Explosion animation | Debris |
|---|---|---|
| hit or missed | the one chosen by F5 (none when *D* = 0), created | F6 |
| nullified | game-wide nullify animation | none |

A projectile that is no longer in play after its area damage creates nothing (claim 21).

## Edge cases and quirks

- **`quirks.area_damage_per_foundation_cell`** (default: on, the original). A building takes one hit
  for each scanned foundation cell within *R* (claim 8). Off: a building takes one hit only, at the
  smallest epicentre distance among its scanned foundation cells, at the place in the order of its
  first such cell.
- **`quirks.spread_ring_eleven`** (default: on). A scan radius of 11 uses the ring-11 list of table
  T1, which scans one cell twice, misses eight cells of metric 11 and scans seven cells of metric 12.
  Off: ring 11 is the set of cells with ring metric 11, in the order of increasing Δ*y* then Δ*x*.
- **`quirks.ground_detonation_misses_aircraft`** (default: on). Tracked flying objects are
  considered only when *P* lies above the terrain at the centre of the centre cell (claim 6). Off:
  they are considered for every detonation.
- **`quirks.flying_search_sectors`** (default: on). The flying-object search reads only the sectors
  of F7, so it can miss a flying object within *R*. Off: every tracked flying object is a
  candidate once, in claim 11's global order of starting the current tracking period; sector
  changes do not reorder it.
- **`quirks.flying_listed_double_hit`** (default: on, the original). An object found through
  both the flying-object search and scanned cells keeps both entries and can take two hits,
  the flying entry first, each at its own distance (claim 6). Off: it keeps only its flying
  entry, giving one candidate per object found by both searches. This does not merge a
  building's foundation entries or repeated cell scans for objects absent from the flying
  search, which remain governed by their own switches.
- **`quirks.flying_candidates_unfiltered`** (default: on). A tracked flying object found by claim 6
  is not left out for being the source object or a listed harvester (claim 9). Off: the same two
  exclusions apply to it as to objects in scanned cells.
- **`quirks.curtain_nullifies_small_detonation`** (default: on). Claim 15. Off: a curtain-protected
  object only cancels its own hits, and the other objects of the spread are hit normally.
- **`quirks.wall_hit_minus_one_is_absolute`** (default: on). Claim 16. A wall hit of exactly −1
  damage points, which only a warhead with the wall flag (or the wood flag on a wooden wall) and
  a damage amount of −1 produces, always succeeds with no draw and removes the wall, as an
  absolute hit does. Off: it is like any other negative amount: it takes a draw and never damages
  the wall.
- **Healing is a point effect** (claim 13). Not a switch: it is the original's rule for negative
  damage, which the healing specs rely on.
- **Falloff beyond the edge.** F3 would extrapolate below *E* for *d* > *R*; claims 3, 9 and 10 never
  deliver such a hit, so the case does not arise in area damage. A producer that uses F3 directly
  states its own distance range.
- **Departure: scan radius out of range** (no switch). For *S* of 11.01 or more (*k* ≥ 12), or
  *S* ≤ −1.99 (*k* ≤ −1), the original's result is undefined. Our engine clamps *k* to 0 … 11 and
  the importer warns about a spread outside 0 to 11.
- **Departure: a small map in the flying-object search** (no switch). On a map with *B* < 20 (F7:
  *W* + *H* − 1 < 20) the sector size is 0, and the original stops with a program error the first
  time a flying object is registered or searched for. Our engine applies S-0205's rule for a zero
  divisor (claim 3: the quotient is 0), so every cell lies in sector (0, 0), which is always
  searched, and the search finds every tracked flying object in registration order. The importer
  warns about a map with *W* + *H* − 1 < 20.
- **Departure: a projectile without a warhead** (no switch). The original stops with a program
  error when a projectile with no warhead detonates. Our engine treats such a detonation as one that
  does nothing: no area damage, no animation, no debris, and the result *missed*. The importer warns
  about a weapon whose warhead is absent or names nothing.
- **Departure: empty metallic debris list** (no switch). In F6, when the warhead lists no debris
  types, *N* > 0 and the game-wide metallic debris list is empty, the original takes the ranged
  draw R(0, −1) for each piece, and the debris it creates is undefined. Our engine takes the same
  draws and creates nothing, and the importer warns about a warhead with a positive maximum debris count and
  nothing to throw.
- **Departure: a zero debris-allocation divisor** (claim 22; no switch). A retained maximum of
  −1 is parsed and kept, but when F6 reaches that type with pieces remaining, the original
  takes its allocation raw value and stops with a program error. Our engine applies S-0205
  claim 3: the remainder is *a*ⱼ, capped by the remaining count as usual. No extra draw is
  taken, and the importer warns about a retained maximum of −1. The same claim supplies a
  remainder of 0 for the overflowing quotient when the maximum is −2 and the raw value is
  −2147483648; the original stops in that exceptional case. Other negative maxima are not
  rejected or clamped: their signed-remainder behaviour is F6. Backing: `measurable` for the
  zero-divisor stop (A19 c), `unmeasured` for the exceptional raw-value stop because the
  recorder cannot force that value; departure results are design rules. Confidence: high.
- **Departure: debris that can never be placed** (no switch). In F6, when every paired maximum
  is 0 or −2 and *N* > 0, every allocation remainder is 0 (apart from the exceptional
  raw-value stop above), so the original never finishes. Our engine stops after one full pass
  over those types, having taken that pass's raw values, and the importer warns. This guard
  applies only to that all-zero-remainder configuration, not to a chance pass of zero
  allocations when some type can receive pieces. A maxima list shorter than the types list is
  bad data: our engine treats the missing maxima as 0 and the importer warns.
  Backing: `unmeasured` for the original's nontermination and missing-maxima result: no finite
  observation proves nontermination, and no isolated missing-maxima setup is specified.
  Confidence: high for nontermination, low for the original's missing-maxima result.
- **Departure: negative damage and the animation list** (no switch). F5 with *D* ≤ −25 (*D* ≤ −35
  for the splash list) gives a negative entry, for which the original's result is undefined. Our
  engine shows no animation for a negative entry, and the importer warns about a weapon with
  negative damage whose warhead has an animation list.
- **Wall hits of healing detonations.** A wall hit with *D* < 0, other than −1 and other than an
  absolute hit, takes a draw (F4) and does nothing else (claim 16). Not a switch: it only matters
  for the order of draws, which our engine reproduces. *D* = −1 and the absolute flag are claim
  16's.

The original's undefined results for an out-of-range scan radius, an empty metallic debris list,
and a negative animation entry remain `unmeasured`: no isolated scenario here exercises those
invalid inputs. Confidence: medium. Their clamping, suppressed creation, preserved draws and
warnings above are design rules, not claimed oracle observations.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| rules, warhead section, `CellSpread` | spread *S* | decimal (single) | 0 |
| no user-data key | within-sector arrival order and alternative global tracking order (claim 11, F7) | fixed behaviour; `quirks.flying_search_sectors` is an engine option, not an imported field | sector search on |
| rules, unit type section, `ConsideredAircraft` | counts-as-aircraft flag: registration when placed airborne (Terms; S-0502 claim 29) | boolean | yes for an aircraft type, no for every other type (S-0501 Import mapping) |
| rules, warhead section, `Parasite` | parasite flag: projectile area damage replaced by attachment (claim 2; S-0614 claim 1) | boolean | no |
| rules, warhead section, `ElectricAssault` | electric-charge effect: when selected, replaces projectile area damage but preserves shared effects (claim 2; S-0518 claim 11) | boolean | no |
| rules, warhead section, `Airstrike` | air-strike flag: designation projectiles make no area hits but preserve shared effects (claim 2; S-0519 claim 16) | boolean (S-0003) | no |
| rules, warhead section, `PercentAtMax` | edge fraction *p* | decimal (single) | 1.0 |
| rules, warhead section, `Verses` | versus values *v* (F3; S-0003 claim 13 owns the parse, S-0305 the order of the armour classes) | percent list | 100 % each |
| rules, warhead section, `Wall`, `Wood`, `WallAbsoluteDestroyer` | wall flags (claim 16) | boolean | no |
| rules, warhead section, `Tiberium` | resource flag (claim 17) | boolean | no |
| rules, warhead section, `AffectsAllies` | affects-allies field (claim 14) | boolean | yes |
| rules, warhead section, `Rocker` | rocking flag (claim 20) | boolean | no |
| rules, warhead section, `Particle` | particle system (claim 20) | particle-system name | none |
| rules, warhead section, `Conventional` | conventional flag (F5) | boolean | no |
| rules, warhead section, `EMEffect` | electromagnetic flag (F5) | boolean | no |
| rules, warhead section, `AnimList` | animation list (F5) | animation-name list; claim 25 | empty |
| rules, warhead section, `MaxDebris`, `MinDebris` | *X*, *Y* (F6) | integer | 0, 0 |
| rules, warhead section, `DebrisTypes` | debris types (F6) | voxel-animation-name list; claim 25; 127-byte text limit | empty |
| rules, warhead section, `DebrisMaximums` | maxima (F6) | decimal-prefix integer list, not the scalar hexadecimal forms; claim 26; 511-byte text limit | empty |
| rules, warhead section, `ShakeXlo`, `ShakeXhi`, `ShakeYlo`, `ShakeYhi` | screen-shake ranges (State and ordering) | integer | 0 |
| rules, unit or building type section, `DamageSelf` | self-damage flag (claim 9) | boolean | no |
| rules, unit type section, `Naval`, `Underwater` | naval flag, underwater flag (claim 21; the same keys as S-0308 and S-0311) | boolean | no |
| rules, building type section, `InvisibleInGame` | invisible in game (claim 9) | boolean | no |
| rules, overlay section, `Wall`, `Armor` | wall overlay, armour class (F4, claim 16) | boolean, armour name | owned by the overlay specs |
| rules, overlay section, `Strength`; art data, the overlay type's image section, `DamageLevels` | *T*, *L* (F4) | integer, integer | owned by S-0210 (claim 13, Import mapping): *L* comes from the art data, not the rules section; engine defaults 1 and 1 |
| rules, overlay section, `ChainReaction`, `Tiberium` | chain-reaction flag, resource overlay (claim 17) | boolean | no |
| rules, overlay section, `Explodes` | explodes flag (claim 19) | boolean | no |
| rules, the combat-damage section, `CrushWarhead` | crushing warhead (claim 9) | warhead name | none |
| rules, the combat-damage section, `C4Warhead` | demolition warhead (claim 19) | warhead name | none |
| rules, the combat-damage section, `IonCannonWarhead` | ion-cannon warhead (claim 18) | warhead name | none |
| rules, the combat-damage section, `BridgeStrength` | bridge strength (claims 18, 24) | integer | initially 1000 bridge-strength points; a missing key keeps the value in force |
| rules, the combat-damage section, `AmmoCrateDamage` | ammunition-crate damage (claims 19, 24) | integer | initially 100 damage points; a missing key keeps the value in force |
| rules, the combat-damage section, `MaxDamage` | maximum damage *M* (F3; S-0302) | integer | S-0302 |
| rules, the combat-damage section, `SplashList` | splash list (F5) | animation-name list; claim 25 | empty |
| rules, the general section, `LightningWarhead`, `WeatherConBoltExplosion` | lightning warhead and its animation (F5) | warhead name, animation name | none |
| rules, the general section, `HarvesterUnit` | harvester list (claim 9) | vehicle-type list | empty |
| rules, the general section, `BarrelExplode`, `BarrelDebris`, `BarrelParticle` | barrel effects (claim 19) | animation name, voxel-animation-name list (claim 25), particle-system name | none; the list starts empty |
| rules, the general section, `MetallicDebris` | metallic debris list (F6) | animation-name list; claim 25 | empty |
| rules, the general section, `WeaponNullifyAnim` | nullify animation (claims 15 and 21) | animation name | none |
| map, the special-flags section, `Inert` | inert flag (claim 2) | boolean | no |
| map, the special-flags section, `HarvesterImmune` | harvester immunity (claim 9) | boolean | no |
| rules, `MultiplayerDialogSettings`, `BridgeDestruction` | default of the bridge-destruction game option (claim 18; S-0511 claim 1) | boolean | yes |
| map, the special-flags section, `DestroyableBridges` | ignored in skirmish: replaced by the setup screen's bridge-destruction option (claim 18; S-0511 claim 1) | boolean | not used in skirmish |

For `AnimList`, `SplashList`, `BarrelDebris`, `MetallicDebris` and `DebrisTypes`, the
case-insensitive whole-item spellings `none` and `<none>` are omitted (claim 25). Names otherwise
follow S-0003 claims 12 and 19's list splitting and type-name matching, including the
untrimmed-item quirk. S-0003 claims 19–20 also own missing/empty-key retention, replacement by
present non-empty values and initially empty lists. These
name lists have a 127-byte text limit. `DebrisMaximums` has a 511-byte text limit and the same
comma splitting, but uses S-0003 claim 20 and claim 26's conversion: `12abc`, `3.9`, `$10`, `10h`, `abc` give
12, 3, 0, 10, 0 respectively, each retaining one place. Numeric warnings follow S-0003's decimal
form, not its scalar hexadecimal forms.

The importer warns about a spread outside 0 to 11 cells, about a map with *W* + *H* − 1 < 20, and
about the debris (including an empty metallic debris list) and animation-list cases of Edge cases
and quirks.

## Measurement scenarios

All scenarios run under the glossary's *measurement setup*, with no engine add-on and no mods (the named test files of a scenario excepted),
on purpose-made flat test maps unless a bridge or shore is named. Each
skirmish has one human player and one bystander computer player kept outside the observed area
(the test objects and all cells within reach of their effects), with the glossary's void rule.
Use game speed 2 unless stated otherwise. Test warheads and weapons are given in the test map's
override sections (named test files); each scenario names the fields it sets, with invented values.
Unless specified otherwise, cause one detonation per run and use three runs per deterministic
variant, recording positions, health and effect creation with a read-only recorder. Statistical
runs have their counts below; the random-state recorder can also compare their actual draws.
No measurement has been recorded for this revision.

**A1. Falloff (claims 7, 12; claim 2 variant e).** *Setup:* a test weapon with damage 1000, a test
warhead with spread 4 (*R* = 1024), edge fraction 0.5 and versus 100 % for the target armour class, no
other effect; a static launcher (a stock building weapon replaced through the override) fires once
at a fixed ground cell, with a projectile that has no inaccuracy, so that the detonation point *P*
is the centre of that cell. Stationary target vehicles of one stock type with health above 1000
(raised in the override), each standing on the centre of a cell, so that the distances from *P* are
those of whole-cell displacements (*a*, *b*): 256·√(*a*² + *b*²) through the table root of S-0204 claim 7. The
distances reachable this way are 0 (0, 0), 256 (1, 0), 362 (1, 1), 512 (2, 0), 572 (2, 1), 724
(2, 2), 768 (3, 0), 923 (3, 2), 1024 (4, 0, exactly *R*) and 1055 (4, 1, beyond *R*); the recorder
reads each target's position and *P*, and the expected values use the distances it reads. The
targets stand in distinct cells and none dies (health above 1000). They have armour 1.0, no
veteran bonus, and their owner's country has the factor 1.0 for vehicles. *Observable:* health lost by each target. *Expected
(variant a):* trunc(1000 − 500 × *d* / 1024) for *d* ≤ 1024 (F3; 1000, 875, 823, 750, 720, 646, 625,
549 and 500), and 0 at 1055. *Falsified if:* any value differs, or the 1055 target is hit.
*Variant b:* edge fraction 1.0 (expected 1000 for all within 1024). *Variant c:* edge fraction 1.5,
with the targets' health raised above 1500 (expected: F3's value at each recorded *d*, rising with
*d*; falsified if any value differs from F3). *Variant d (order of the defences and the falloff):* the country of the
targets' owner has the vehicle armour factor 3.0 (S-0302 claim 3) and everything else as in variant a,
expected 333, 291, 274, 249, 239, 215, 208, 182, 166 (the falloff on the defended amount 333), where
a falloff on 1000 followed by the division would give 250, 240 and 183 at distances 512, 572 and
923; any loss different from F3 on the defended amount falsifies the rule. *Variant e:* the same map as a
with the inert flag set, and a wall cell inside the spread with the wall flag set on the warhead
(expected: no health lost, the wall's stage unchanged). Also name a particle system on the
warhead and put an exploding overlay in the centre cell: it stays, no warhead particle system
appears, and no area-effect draw is taken. Any contrary effect falsifies the inert-area rule. *Variant f (off-lattice
distances, 1023 and 1025):* the same warhead fired from the vertical test projectile of S-0306
claim 11, whose type's detonation altitude field is varied from run to run; the recorder reads *P*
and the targets' positions, and a target displaced by (3, 2) cells from the cell under *P* (horizontal distance 923) gives
distances √(923² + *h*²) for the height *h* of *P* above the target's position: a run with
*h* from 441 to 445 gives *d* from about 1023 to 1025 depending on the exact height reached (the projectile
climbs in steps, so the height comes from the recorded *P*; runs are repeated with neighbouring
altitude values until distances of 1023, 1024 and 1025 are all recorded). Expected: the loss
follows F3 at every recorded *d* up to 1024 and is 0 at 1025. *Runs:* 1 each (no draw involved for
vehicles; check S-0302 adds none).

*Variant g (large-amount narrowings):* named map overrides set the delivered damage to
16777219 health points, spread to 1 cell, edge fraction to 0.75, the relevant versus value to
100 % and the game-wide maximum damage above the delivered amount. Use a non-veteran target
vehicle with strength and health 30000000, no armour modifiers and country armour factor 1.0.
Place it exactly 256 leptons from the recorded detonation point, in a scanned cell. Fire once
in each of three runs and record its health loss. Expected: *d*₃ = 16777219,
*D*ₛ = 16777218 and *E* = 12582914, hence a loss of 12582914 health points at the edge.
A loss of 12582913 would instead indicate that the edge product incorrectly used *D*ₛ.
Any other loss also falsifies F3 for this setup. These are invented test values, not stock data.

*Variant h (bypass cases):* use g's damage, target health, armour and cap settings, but an
edge fraction of 1.0. Expected: the target loses 16777219 health points, not the narrowed
16777218. Separately use edge fraction 0.75 and spread 0, aiming exactly at the target's
recorded position so that its distance is 0 and its cell is scanned. Expected: again a loss
of 16777219 because the zero radius bypasses interpolation. Use three fresh runs of each
case. Any other loss falsifies the respective bypass rule.

**A2. Scanned cells versus radius (claim 3).** Wall hits (claim 16) take no distance test, so a
wall shows directly whether its cell is scanned. *Setup:* a field of stock wall cells around the
detonation cell; a test warhead with the wall flag and a damage amount at least the wall strength (no
draws, F4), fired at the centre of the centre cell, with spreads 0.01, 0.02, 1.01, 1.02 and 3.5.
Separately, a vehicle on the centre of a ring-1 cell (256 leptons away) with spreads 0.99 (*R* =
253) and 1.0 (*R* = 256), and a warhead without the wall flag. *Observable:* which wall cells change
stage; whether the vehicle is hit. *Expected:* spread 0.01 changes only the centre cell; 0.02 and
1.01 rings 0 and 1; 1.02 rings 0 to 2; 3.5 every cell of ring metric 4 or less, including (3, 3) at
4.24 cells, but not (4, 2) at 4.47 cells. The vehicle is not hit at 0.99 (scanned but beyond *R*)
and hit at 1.0. *Variant b (ring 11):* spread 11 and walls at every cell in and just outside
T1, each below its strength so that a hit takes a draw; use the recorder's starting random state
to predict each F4 success and neighbour hit. Expected: exactly the T1 entries contribute wall
hits in its stated order, including two hits on (−3, 11) and none on the omitted cells. Compare
recorded wall stages and stream state with that prediction. Any disagreement falsifies the list
or its order. *Falsified if:* any main-part wall or vehicle outcome differs, or b's wall stages
or draws differ from T1 and F4.

**A3. Buildings: one hit per foundation cell (claims 7, 8, 9).** *Setup:* test weapon with damage 100
and a test warhead with spread 3 (*R* = 768), edge fraction 1.0, versus 100 % for the building's
armour class, buildings with armour 1.0 and an owner whose country factor for buildings is 1.0; one
1 × 1 and one 3 × 3 stock building of the same armour class, each fully inside the spread of
separate detonations at the same distance from their reference points. *Variant b:* the 1 × 1
building only, edge fraction 0.5, and a detonation above the centre of its cell (the vertical test
projectile of S-0306 claim 11, with a detonation altitude field chosen by dry runs until the
recorder reads *P* at about 300 leptons above the terrain at that cell's centre; the projectile type is
not of the airburst kind). *Observable:* health lost. *Expected:* the 1 × 1 building loses 100; the
3 × 3 building loses 900 when all nine cells are within *R* (S-0302's per-hit rules applied to each
hit). Variant b: F3 with *d* = ⌊*r*(*n*)⌋ − 208 for the distance the recorder gives; with a
measured distance of 300 that is *d* = 92 and a loss of 94, against 80 if the full distance were
used and 100 if *d* were 0. Repeat b at recorded heights 208 and 209 leptons, keeping *P*
horizontally at the cell centre: expected *d* = 0 and 1 respectively, and losses 100 and 99.
A different loss falsifies the centre-cell height boundary.
*Variant c:* repeat with the can-be-demolished flag off and a zero
versus value: each eligible foundation entry costs 1 health point. Then restore versus 100 % and
lower the building's starting health below one hit, with the warhead's delayed-kill flag off.
It dies once and later entries produce no further death effects. The delayed-kill exception is
measured by S-0302 scenario D13 part d: an eligible multi-cell building remains at 1 health
and each later lethal foundation-cell hit credits another destruction under the default quirk,
without the death effects that await actual destruction. *Falsified if:* any main-part loss differs from 100 per eligible cell,
any variant-b loss differs from F3 at the recorded reduced distance, variant c does not lose
1 per entry with zero versus, or the destroyed building produces repeated death effects.

**A4. Bridge-deck layer (claim 4).** *Setup:* a high bridge; one vehicle on the deck and one on the
ground under the deck in the same cell; detonations (spread 2, edge fraction 1.0) once at the deck
surface and once at ground level. *Observable:* the health of each vehicle. *Expected:* the deck detonation hits only the deck vehicle, the
ground detonation only the ground vehicle. *Variant b (height boundary):* use the vertical test
projectile of A3 b to obtain recorded heights of 208 and 209 leptons above the terrain at *P*,
keeping *P* in the bridge cell. Expected: at 208 only the ground vehicle is hit; at 209 only the
deck vehicle is hit. Repeat with *P* in an adjacent cell that carries no bridge and with both
vehicles still within *R*: only the ground vehicle is hit, even when *P* is above deck height.
*Falsified if:* either detonation hits the vehicle of the other layer, fails to hit the vehicle
of its own layer, or any variant-b layer selection differs.

**A5. Flying aircraft (claims 6, 10).** *Setup:* a stock aircraft hovering or circling at known
height near a ground target; (a) a ground-level detonation with spread 10 directly below it; (b) a
detonation at a set height with the same spread, from the vertical test projectile of S-0306 claim
11 (its detonation altitude field set, and not of the airburst kind of S-0306 claim 21, which would
launch child projectiles); (c) the same projectile with the aircraft at a known distance and edge
fraction 0.5. *Observable:* health lost by the aircraft. *Expected:* (a) no damage to the aircraft,
(b) damage; (c) damage as F3 with half the measured distance. *Falsified if:* (a) damages it, (b)
leaves it undamaged, or (c) differs from F3 at the halved distance. *Variant d:* repeat b and c
with a jumpjet instead of an aircraft-kind object. Expected: it is reached through the flying
search but uses its full distance in F3. Any other health loss falsifies that distinction.
*Runs:* 3 each.

**A6. Source object and flags (claim 9).** *Setup:* a vehicle that fires a test weapon (spread 2) at
a cell next to itself; variant b gives its type the self-damage flag; variant c places a stock
harvester-list vehicle in the spread with the map's harvester-immunity flag set, and variant d
without it. *Variant e:* a building whose type is invisible in game, inside the spread; *variant f:* a vehicle
inside a transport (held out of play) that is inside the spread. *Observable:* health lost by the
firer and the harvester (variants e and f: by the building and the passenger). *Expected:* the firer is not hit (a), is hit (b); the harvester is not hit (c), is hit
(d); the invisible building and the passenger are not hit (e, f). *Variant g:* a's setup with
the test warhead selected as the game-wide crushing warhead, self-damage still off: the firer
is hit. *Falsified if:* any differs.

**A7. Healing point (claim 13).** *Setup:* a test weapon with damage −50, spread 3, fired at a
damaged vehicle; other damaged vehicles at 300 leptons from the detonation point and, in repeated
runs, the vertical test projectile of S-0306 claim 11 detonated 7 and then 8 leptons above a damaged
vehicle (the recorder reads the distance, so cell-centred vehicles need not stand at 7 and 8
leptons). *Observable:* each vehicle's health. *Expected:* only targets at *d* < 8 heal, each by 50
(vehicles only). *Variant b:* a building with the can-be-demolished flag off, enough health to
survive all entries, and eligible foundation cells both below and above 8 leptons: it loses
1 health point per eligible entry rather than healing (S-0302 claim 7). *Falsified if:*
any vehicle at 8 or 300 heals, one below 8 does not heal by exactly 50, or the building's loss
differs from the eligible-entry count.

**A8. Allies (claim 14).** *Setup:* a test warhead with the affects-allies field off; an allied
house's vehicle in the spread. (a) The firer alive at detonation; (b) a slow projectile whose firer
is destroyed before it lands. *Observable:* the ally's health. *Expected:* (a) no damage to the ally; (b) the ally is damaged.
Repeat a with the affects-allies field on, and with a same-house target distinct from the
firer: it is damaged with the field on and protected with the field off and attacker present.
*Falsified if:* any expected health outcome differs.

**A9. Curtain protection (claim 15).** *Setup:* a vehicle under the iron-curtain effect, and an
unprotected vehicle within *R* of the detonation point (not necessarily in the same cell, so that
S-0209's occupancy rules are not strained); (a) a small detonation (spread 0.5) within 85 leptons of
the protected vehicle; (b) spread 0.6; (c) the same as (a) under a force shield instead.
*Observable:* the unprotected vehicle's health and the animations the recorder sees created.
*Expected:* (a) the unprotected vehicle takes no damage and the nullify animation shows; (b) and
(c) it takes damage. *Variant d:* the protected object is a tracked flying object, with the
detonation raised above the terrain as in scenario A5 (b), spread 0.5, and recorded distances
84, 85 and 86 leptons from *P*: nullified at 84, not at 85 or 86. *Variant e:* the protected vehicle in the centre cell 84 leptons
from *P* with spread 0.3 (*R* = 76, so the vehicle is beyond *R*): still nullified.
*Variant f (cell and post-hit effects):* repeat a on separate copies with scanned wall and resource
cells configured as A10, and with a centre-cell exploding overlay and a named warhead particle
system as A13. Expected: the walls and resource cells change just as without protection, but
the exploding overlay remains and no warhead particle system appears. Also repeat at an A12
bridge-test cell with a protected centre-cell vehicle: no bridge draw or destruction occurs.
*Falsified if:* any differs.

**A10. Walls and resources (claims 16, 17).** *Setup:* a line of stock walls through the spread,
(a) a warhead with the wall flag and damage at least the wall strength (no draws): the stage of
every scanned wall cell rises by one, whatever its distance; (b) damage below the strength:
statistical over 200 runs, share of walls damaged ≈ *a* / (*T* + 1) within 3 standard errors,
(c) the absolute flag: every scanned wall cell is removed; (d) a test overlay type with the
chain-reaction and resource flags and resource amount, and a warhead with the resource flag (claim
17): it loses trunc(*D*/10) units; (e) healing
warheads with the wall flag (damage −50 and then −1) and one with the absolute flag and damage −50:
the wall's stage is unchanged at −50 (a draw is taken, seen through the recorder), and the wall is
removed at −1 and with the absolute flag. Repeat d with each required flag separately off and
with damage 9 and −50: no resource reduction. Use resource amounts above trunc(*D*/10) to avoid
exhaustion, disable growth for the observation, and record the amount immediately before and after
the detonation. In d with positive damage, the expected loss is capped by the available amount.
*Observable:* wall stages and removals, resource amount and attributed wall-hit draws. *Falsified if:* a wall outside the scanned cells changes, the shares differ,
variant d differs from that capped loss, any negative control reduces resources, or variant e
differs in wall state or draw count. *Variant f:* wall flag off, wood flag on, with wooden and
non-wooden wall types: only the wooden wall takes a hit. Both flags off: neither does; the absolute
flag alone removes both. *Variant g (F4 stages):* named test wall art and rules give a stage
count above 2. Test a linked wall and an isolated wall at the penultimate-stage transition, with
same-type stage-0 neighbours. Record overlay data and draws before and after one certain hit.
Expected: the stage rises by 1, stage-0 orthogonal neighbours get F4's 200-point hits in map
north, east, south, west order, the isolated wall disappears and a linked penultimate-stage wall
stays until its next successful hit. Different state or draw outcomes falsify F4.
*Variant h (resource-explosion permission):* use S-0401 E17's animation chain reaction on a
resource-filled centre cell, with neighbouring resource cells satisfying d's overlay flags and
an overridden demolition warhead with the resource flag, positive damage and spread reaching
those neighbours. Suppress growth, resource spreading and crater effects during the observation.
Record resource amounts immediately before and after the chain reaction and the scenario stream.
Expected: the source cell is emptied by the chain reaction, but the neighbours retain their
amounts and take no spread-offer draws through scanned-cell removal. Any neighbouring removal
or such draw attributable to that detonation falsifies the permission exception.

**A11. Order of hits and nested detonations (claims 1, 11).** *Setup:* a test warhead with spread
3 and edge fraction 1.0 fired by a stock vehicle *F*; vehicle *A* in the centre cell, with a test
death weapon (S-0312) of spread 1.5 whose damage kills vehicle *B*; *A*'s health is below the outer
damage; *B* stands on the centre of a ring-1 cell, 256 leptons from *A*, within the death weapon's radius, with health below both
amounts. The outer detonation reaches *A* first (ring 0) and *B* later (ring 1). *Observable:*
whether *B*'s kill is credited to *F* (its experience or kill count, S-0311). *Expected:* *A* dies,
its death weapon kills *B* during the outer detonation, and the outer hit on *B* does nothing, so
*B*'s kill is not credited to *F*. *Falsified if:* *B*'s kill is credited to *F*. Variant b swaps
the cells (*B* in the centre cell, *A* in the ring-1 cell): *B*'s kill is credited to *F*.
*Variant c (order inside one cell, claim 5):* *A* and *B* both in the centre cell, as two objects
that can share a cell (for example two infantry units, if S-0209 allows it), *A* with the test
death weapon; the observable is again whether *B*'s kill is credited to *F*. The expected result
follows from S-0209's listing order (claim 13): credited if *B* is listed before *A*, not credited
if *A* is listed first. The variant is falsified if the credit disagrees with that order.
*Variant d (new object):* the first outer hit kills a non-exploding transport with a surviving
passenger and no death weapon. Give the passenger enough health to survive the outer amount and
ensure its escape cell is a scanned cell later in T1 order, and discard runs where placement
fails. Record passenger health immediately after escape and after the outer detonation. Expected:
no outer-hit health loss for that passenger; any loss attributable to the outer detonation
falsifies the fixed-entry rule. *Variant e (cell effects before object hits):* use a death weapon
that makes one identifiable electromagnetic animation-choice draw, with scanned wall cells whose
F4 hits take draws. Record the random state and wall stages. Expected: all outer wall-hit draws,
including neighbour-hit draws where F4 applies, precede that death-weapon draw. A reversed order
falsifies State and ordering. Resource-before-hit timing is `unmeasured`: these scenarios read
resource totals after detonation, not during a hit effect.

**A12. Bridges (claim 18).** *Setup:* destroyable bridges on; a low bridge and a high bridge; a
warhead with the wall flag and damage *D*. Statistical over 200 runs per value, at a bridge cell
where the recorder sees one bridge draw per detonation (one bridge test applies) and where a single
certain attempt destroys a section (variant b's single-attempt detonation shows which cells these
are): share destroyed ≈
(*D* − 1) / bridge strength for 1 ≤ *D* ≤ strength, within 3 standard errors; an ion-cannon
warhead attempts the destruction every time, so it destroys a section whenever the bridge allows
one. *Observable:* whether the section is destroyed. *Falsified if:* outside, or an ion-cannon
detonation on a bridge that allows destruction leaves the section standing. *Variant b (the
repeats):* for each cell position of a high bridge and of a low bridge, one detonation of a warhead
with the wall flag and a damage amount above the bridge strength (so that its single attempt is
certain to be made) on one copy of the map, and one detonation of the ion-cannon warhead on another. The
observable is the state of the bridge cells afterwards. *Expected:* an intact high section becomes damaged after the single ordinary attempt but is
lost after the ion-cannon's retries (S-0511 claim 3). An intact low strip becomes damaged after
either detonation; an already-damaged low strip becomes destroyed after either (S-0511 claim 13).
The low comparisons use ordinary straight strips away from end pieces and neighbouring damage.
*Falsified if:* the ion-cannon leaves the intact high section merely damaged, or destroys an
intact low strip that the ordinary attempt only damages. *Variant c (height window):* use a high-bridge cell where
variant b's certain single attempt destroys a section, with a wall-capable warhead and damage
above bridge strength. Vary the recorded detonation height across both endpoints of claim 18,
including each endpoint and a point 1 lepton on either side; use the vertical test projectile
and discard runs that do not attain the requested height. Expected: no high-bridge draw or
attempt at the lower endpoint, an attempt 1 lepton above it and at the upper endpoint, and none
1 lepton above the upper endpoint. Repeat over a low bridge and a pier cell without the
high-bridge marking: no such height exclusion. Any different recorded attempt or draw outcome
falsifies the height rule. *Variant d (skirmish gate):* repeat a certain attempt on fresh copies
of each family and bridge kind with the setup option on and off, crossed with the map setting
on and off. Record bridge changes and chance draws. Expected: attempts and their chance draws
follow only the setup option, with none when it is off, regardless of the map setting. A map
setting that overrides the option falsifies the gate. *Variant e (membership):* repeat on a
low piece already in the destroyed class and on intact or damaged pieces of both families,
with ordinary warheads. Expected: no chance draw or attempt for the destroyed piece; one chance
draw per eligible low test otherwise, as S-0511 claim 1. Any contrary draw count falsifies it.

**A13. Exploding overlays (claims 19, 20).** *Setup:* a stock exploding overlay (barrel) in the
centre cell of a detonation with a warhead that cannot damage overlays otherwise, and another in a
neighbouring scanned cell. *Observable:* whether each barrel explodes, the health lost by the
vehicles near the centre barrel, and the debris pieces and particle systems that the recorder
sees created. *Expected:* the centre barrel explodes (nested detonation damages nearby
vehicles by the ammunition-crate damage scaled as F3 with the demolition warhead); the neighbour
barrel explodes only if the nested detonation's own centre cell holds it (it does not), so it stays.
Statistical over 100 runs: debris appears in a share 1 − 0.85ⁿ of the runs (*n* the length of
the barrel debris list) and the particle system in 25 %. Variant c: a warhead naming a particle
system shows it at *P*. For initial-particle and lifetime variants, use S-0613 scenario G3:
record the initial count with and without a held type, and compare an unlimited gas system with
one whose lifetime is positive. Expect exactly one initial particle when a held type exists,
none otherwise, and no timeout with the absent lifetime default. Variant d, with the
random-state recorder, over runs until one shows a debris piece: expect the piece's creation draws
(S-0317 claim 3) after the percent draw that chose its entry and before the particle system's
percent draw. *Falsified if:* the neighbour barrel explodes from the outer detonation, or
the share of runs with debris differs from 1 − 0.85ⁿ, or the share with the barrel particle system
differs from 25 %, by more than 3 standard errors, or in variant c the warhead's particle system
does not appear at *P*, the initial particle count differs from the held-type prediction,
or the finite/unlimited system-lifetime outcomes differ from G3, or in variant d the piece's
creation draws come after the particle system's percent draw or are missing.

**A14. Explosion animation (claims 21, 23).** *Setup:* a test warhead with an animation list of 4
distinct stock animations and damage values 1, 24, 25, 74, 75, 99, 100, 1000. *Observable:* which
animation the recorder sees created for each detonation and where; in variants e and h also the
recorder's position on the scenario random stream; in variant f also the debris. *Expected:* entries
0, 0, 1, 2, 3, 3, 3, 3 (F5). Variant b: over water with the conventional flag and a splash list of 3:
entries ⌊min(*D*, 104)/35⌋. Variant c: the same over water with a stock naval vehicle in the cell under the
projectile (the first object listed there) that the detonation hits: the warhead's own list is
used. Variant d: the electromagnetic flag,
statistical over 100 runs: all four entries appear. Variant e: a weapon whose projectile is of
the invisible kind, with the recorder reading the scenario random stream's position before and
after the detonation: one raw value more than the same detonation with a visible projectile, and the
animation within 32 leptons of *P*. *Variant f (damage amount 0, claim 2):* the same
warhead with damage 0 and a maximum debris count above 0: no explosion animation, and debris is
still thrown. Also set the wall flag, name a particle system and place a centre-cell barrel,
scanned walls and a healthy target. Expected: no target health loss, wall-stage change, barrel
removal, warhead particle system or area-effect draw. Any such effect falsifies claim 2.
*Variant g (negative damage):* damage −24 with the animation list of 4: entry 0.
*Variant h (nullified, electromagnetic):* variant a of scenario A9 with an electromagnetic warhead
whose projectile hits: the recorder's stream position advances by one raw value even though the
nullify animation is the one that shows. *Variant i (*Q* against *P*):* a projectile with cluster
count 3 whose own position is over land at the shore, repeated until the recorder reads further
detonation points over water, with a splash list and the conventional flag: every detonation's
animation comes from the warhead's own list (the land type is that of the projectile's own
position), and each is shown at its own detonation point. *Variant j (the air rule, claim 21):* over water with the conventional flag, a splash list of 3, and
the vertical test projectile of S-0306 claim 11 detonating 207 and then 208 leptons above the
terrain (the recorder reads the height): the splash animation at 207, the warhead's own list at
208. *Variant k:* select the test warhead as the game-wide lightning warhead and specify a
distinct lightning explosion type; on land it shows that type without an animation-choice draw.
*Variant l:* the water setup with a bridge in the projectile's cell, then with an underwater
naval vehicle, then with no object hit. Expected: the bridge uses the warhead list; the underwater
and missed cases use the splash list. *Variant m (degeneration):* a degenerating projectile
starts with damage 80; vary flight length so the recorder gives its then-current damage on both
sides of an animation-entry boundary. Record health loss of an armour-1 target at distance 0.
Expected: the animation entry and health loss both follow the amount at detonation, with the
5-point degeneration floor, not 80. *Falsified if:* any entry, position, health loss or attributed
draw count differs from F5 and these predictions (including the single extra raw value in e
and the one electromagnetic choice in h). In d, compare each actual choice with F5 using the
recorded ranged draw, not merely whether every entry appears. *Variant n (screen shake):* use a
warhead with horizontal range 2 to 3 and vertical range 7 to 8, then repeat with each range and
both ranges zero at both ends. Record both random streams immediately before the detonation and
the screen displacement. Expected: one unsynchronised ranged draw for each nonzero range,
horizontal then vertical, and no additional scenario-stream draw. Any count, stream or axis-choice
order different from those predictions falsifies the draw rule.
*Variant o (parasite-projectile visuals, claim 2):* use the successful and failed latches of
S-0614 P1 and P3, with named map overrides giving the latching weapon damage 25, a warhead
animation list of two distinct stock types, minimum debris 2, maximum debris 3 and no voxel
debris types. Keep a nonempty metallic debris list. Record attachment outcome, created effects,
their positions and random state at detonation. Expected in both latch outcomes: no area hit,
but the animation at entry 1 and two metallic debris animations, following F5 and F6 at the
recorded *P* and *Q*. Suppressing either visual effect solely because the parasite flag is set,
or any different type, count or position, falsifies this part.

**A15. Warhead debris (claim 22).** *Setup:* a test warhead with minimum debris 2, maximum 6 and no
debris types; counted over 200 detonations. *Observable:* the number, types and positions of the
debris animations that the recorder sees created by each detonation. *Expected:* counts 2 to 5 only (R(2, 5)), roughly
uniform. Variant b: two debris types with maxima 1 and 3, with the recorder reading the raw values
of the scenario random stream, including its state before and after each piece's creation.
Expected: the two types together number 2 to 5 pieces per detonation, and each type's count is the
one F6 gives for the allocation values read. Creation draws for the pieces of one assignment
precede the next assignment's allocation value; any reversed order falsifies F6's ordering. Variant c (position, claim 22): the projectile
of variant i of scenario A14 with this warhead: the recorder reads the debris positions, which
equal the projectile's own position for every detonation of the cluster (the metallic ones 20
leptons higher). *Falsified if:* any count is outside 2 to 5, any type or position differs from F6 at the
recorded random state, or the count frequencies differ from uniform by more than 3 standard
errors (each count has expected share 0.25). Repeat a on an inert map and during A9's nullified
detonation: expected the same count rule on the inert map and no debris when nullified.
Either different outcome falsifies claim 22.

**A16. Flying-object search sectors (claims 6, 9, 11).** *Setup:* a purpose-made test map whose
width and height add up to 61 to 80 (*W* + *H* − 1 from 60 to 79), so the sector size of F7 is
*w* = 3 cells; a test warhead with spread 9 (*R* = 2304 leptons, scan radius 9, inside table T1),
edge fraction 1.0 and versus 100 % for the aircraft's armour class, fired with the vertical test projectile of S-0306 claim 11 (detonation altitude field set; not of
the airburst kind of S-0306 claim 21), whose detonation point the recorder reads and which is
placed by dry runs so that it lies over a cell (*c*ₓ, *c*ᵧ) that is the middle cell of its
sector *C* along both axes, at least 15 cells from every map edge, and in a sector with
3 ≤ *C*ₓ ≤ 16 and 3 ≤ *C*ᵧ ≤ 16, so the four runs are each 3 sectors long (the last sector of each
axis holds every cell from 57 up to *B*, so the distance from the edge alone does not ensure this). Two stock aircraft hovering at the height of the detonation: one over the cell (*c*ₓ + 8,
*c*ᵧ − 2), which lies in the sector three east and one north of *C* (off the axes, off the
diagonal and not one of the two extra sectors; about 8.25 cells away, within *R*), the other over
(*c*ₓ + 5, *c*ᵧ − 5), in the sector two east and two north (on the diagonal). Variant b: an
aircraft over (*c*ₓ + 5, *c*ᵧ − 2), in the sector two east and one north (an extra sector of the
north-east group). Variant c: the firer is itself a hovering aircraft within *R* of the detonation point, over a
cell of sector *C*, so that its sector is always searched.
*Observable:* health lost by each aircraft. *Expected:* the off-diagonal aircraft takes no damage,
the diagonal one takes the full damage (edge fraction 1.0); b: that aircraft is damaged; c: the
firing aircraft is damaged. *Variant c2:* place that aircraft type in the game-wide harvester
list and enable the harvester-immunity flag: it is still damaged. *Falsified if:* c2 is immune,
the off-diagonal aircraft is damaged, the diagonal or
variant-b aircraft is not, or c leaves the firer undamaged. *Runs:* 3 each.

**A17. Absent bridge and barrel values (claim 24; Q-0060).** *Setup:* a named loose test rules
file with both fields removed, and no language, mode or map layer giving either field. Restart
the program before each run. Use A12's single-test low-bridge cell, a wall-capable test warhead
and damage 501, and separately A13's exploding overlay with a test demolition warhead of edge
fraction 1 and versus 100 %, and vehicles of armour 1 at recorded positions inside its spread.
Use the measurement setup, no engine add-on and no mods (the named test files of a scenario excepted), game speed 2, one human player and one bystander
computer player kept outside the test area. *Procedure:* cause one bridge detonation or one barrel
detonation per run. *Observable:* bridge destructions and vehicle health loss, using the recorder.
*Runs:* 400 bridge runs and 3 barrel runs, each at a fresh program start. *Expected:* bridge
destruction share 0.5 within 3 standard errors, and each qualifying barrel target loses 100
health points, after accounting for any separately recorded outer hit. *Falsified if:* the share
is outside that band, no bridge attempt occurs, or barrel health loss differs. *Layer variant:*
a named map gives bridge strength 2000 and ammunition-crate damage 140, followed without restarting
by the same test with both fields absent everywhere. Expected: the second game retains those
values, giving bridge destruction probability 0.25 and barrel loss 140; resetting to 1000 and 100
or suppressing either effect falsifies the retention rule.

**A18. No-type items in effect lists (claim 25; Q-0061).** *Setup:* the measurement setup and
houses of A17; named test maps give each list in turn an omitted first item and one valid stock
type, with no whitespace around the comma. Test both no-type spellings of Import mapping,
including mixed case. For the explosion and splash lists use damage 1 and the applicable F5
case. For metallic debris use no warhead voxel types and a positive fixed piece count. For
voxel debris use a positive maximum for the sole retained type. For barrel debris use A13's
centre barrel. *Procedure:* detonate once per run; record created effect types and the scenario
random state. *Runs:* 3 per deterministic case and 200 for barrel debris. *Expected:* graded and
splash selections show the valid type at entry 0; metallic and voxel pieces are of the valid
type only. Barrel debris has one entry, one chance draw and appearance probability 0.15 within
3 standard errors, not two chance draws or probability 1 − 0.85². *Falsified if:* a first-entry
selection shows nothing, a list choice includes an empty place, or the barrel draw count or
appearance share differs. Repeat with the omitted item last to test that valid entries do not
move relative to one another.

**A19. Maxima parsing and independent list positions (claim 26; Q-0061).** *Setup:* the
measurement setup and houses of A17, with named test maps giving a warhead two distinct stock
voxel-debris types, a fixed positive total piece count, and maxima pairs whose first item is
in turn a positive decimal with a leading plus sign and trailing text, a fractional decimal, a hexadecimal-prefix-looking
item, a hexadecimal-suffix-looking item, or unreadable text. Keep the second maximum positive.
Use the spellings and expected integers of Import mapping. *Procedure:* detonate and record the
raw scenario values and each type's piece count. *Runs:* 10 per pair. *Expected:* every allocation
matches F6 with the parsed maxima, including zero in place for unreadable text. A no-type item
prepended to the types list changes neither of the two retained types' maxima. *Falsified if:*
any allocation or draw count differs from F6 at the recorded raw values and the predicted parsed
maxima, including hexadecimal conversion or omission of a retained zero maxima item. *Text-limit variant:* prepend enough
zero maxima items that a positive maximum falls beyond byte 127 but within the value retained
under S-0001 claim 3's physical-line limit, and give matching valid repeated types. Use long
runs of zero digits within a few maxima items so that the types list still fits its own
127-byte limit. Expected: F6 reaches that positive maximum and finishes. A read-only recorder
can confirm the list length before detonation. Truncation at 127 value bytes falsifies claim
26's import mapping. The separate 511-byte value-reader limit cannot be reached by file input
because the physical-line limit also counts the key and separator; it remains unmeasured.
Do not detonate if the recorder shows only zero maxima, because the original would not finish
(Edge cases).

*Variant b (signed maxima, claim 22 and F6):* repeat the two-type allocation test with the
first maximum −2, then −4, then −2147483648, then 2147483647, keeping the second maximum 3.
Use a fixed total of 4 pieces (minimum 4, maximum 5), ten detonations per pair, and record
allocation raw values separately from creation draws. Expected: for ordinary raw values the
first type's allocation is respectively 0, min(|*raw*| rem 3, pieces remaining),
min(|*raw*| rem 2147483647, pieces remaining), and
min(|*raw*| rem 2147483648, pieces remaining); the second type follows F6. Any count or stream
consumption different from F6 falsifies the signed-divisor rule. These cases keep a type that
can receive pieces, avoiding an all-zero-remainder hang. They do not claim to force the rare
minimum signed raw value.

*Variant c (zero divisor, claim 22):* on an isolated copy of that test map give the first type
maximum −1 and the second 3, with the same fixed total 4. Start a fresh program for each of
three runs, fire once and record the random stream until program termination. Expected: the
first allocation takes one raw value, creates no voxel pieces and ends the original with a
program error. Continued play, creation of a voxel piece or a different attributed allocation
draw count falsifies that prediction. This test deliberately stops the original; do not use an
unsaved game. Our engine's warned, continuing result under S-0205 claim 3 is a departure and
has no oracle value to compare.

**A20. Sector arrival order (claim 11; Q-0614).** *Setup:* A16's sector geometry,
with two stock jumpjet infantry types, *A* and *B*, kept airborne in one sector and away from
other units. A named test map gives *A* a death weapon and the exploding-on-death flag under
S-0312 claim 6. Its death weapon reaches and kills *B*. An unrelated human-owned launcher *F*
fires an elevated test detonation reaching both with lethal damage, edge fraction 1 and no
other effects. Use the recorder to verify positions, tracking periods and the outer detonation
point; both objects must qualify for claim 6 and neither may appear in a scanned cell layer.
Void runs where either lands, takes an unrelated hit, or moves out of reach. Use game speed 2,
the measurement setup, no engine add-on and no mods (the named test files of a scenario excepted), and a bystander computer
player outside the test area. *Procedure:* make *A* become tracked first in an adjacent sector,
then make *B* become tracked in the destination sector, then fly *A* across into *B*'s sector.
Detonate after the crossing. *Observable:* whether *B*'s destruction is credited to *F*, read
from its kill count or experience, together with recorded health and positions. *Expected:*
*B* precedes the later arrival *A*, so *F* kills *B* before *A*'s death weapon can do so.
Keeping first-tracking order would instead let *A*'s death weapon kill *B* without credit to *F*.
*Variants:* start both in one sector with *A* tracked first, then move *A* between cells within
that sector: no credit to *F* for *B*. Send *A* out of that sector and back, without ending
tracking: credit to *F* for *B*. Add a third harmless airborne object between *A* and *B* in
arrival order, send it out, and repeat the pair test: the remaining *A*, *B* order is unchanged.
*Runs:* three qualifying runs per variant, with scripted ordinary movement and firing input
where useful. *Falsified if:* kill credit differs from these predictions at the recorded positions.
The same arrival rule applies to aircraft and rocket missiles; these jumpjets separate its
hit-order consequence from aircraft crash timing.

*Alternative replay check (not an oracle measurement):* replay the main crossing and the
out-and-back variant on our engine with `quirks.flying_search_sectors` off. *A* stays before
*B*, so *B* is killed by *A*'s death weapon, not credited to *F*. Ending *A*'s tracking and
starting it again while *B* remains tracked instead makes *B* precede *A*. This validates
claim 11's design rule, not any off-state behaviour of the original.

**A21. Flying and cell-listed entries (claim 6).** *Setup:* use S-0502 scenario J28's
low-jumpjet descent and take-off fixtures, including the elevated decoy detonation for take-off.
Use a named map override with delivered damage 100, spread 3 cells, edge fraction 0.5,
versus 100 % for the target's armour, no other warhead effects, and target health 10000.
The target is non-veteran, not prone, with object and house armour 1.0. Keep the source
object distinct from the target. Use the common skirmish settings above, game speed 2,
director and other mods off. *Procedure:* cause one detonation while the read-only recorder
confirms the target is both tracked and listed in the scanned cell layer. Record both
candidate distances, health changes and their order. Repeat with the target's health below
the first hit's predicted damage. Use three qualifying runs of each take-off, descent and
lethal variant, voiding runs with unrelated damage or absent dual membership.
*Expected:* in the surviving variants, two successive hits, flying first, each losing the
amount F3 gives at its own recorded claim-7 distance; no assumption that their amounts are
equal. In the lethal variant, one effective hit and one destruction, with no second damage
or death effect. A single hit on a surviving dual-listed target, a wrong per-hit amount,
reversed distinguishable entry effects, or a second effective hit after destruction falsifies
claim 6. S-0502 scenario J28 supplies the grounded and high-flight single-entry controls.
*Engine-only replay:* with `quirks.flying_listed_double_hit` off, each dual-membership variant
has only the flying entry and its predicted damage. This is the alternative's design rule,
not an oracle measurement.

## Open questions
- The order of several eligible bridge tests on one cell (claim 18) lacks a fixture that
  distinguishes every pair. Bridge-piece acceptance and attempt effects are specified by
  S-0511 claims 1 to 15, not open here.
- The boundary with S-0302: this spec states the falloff on the defended amount and, for completeness,
  the versus step and cap as the original computes them in one expression (F3); S-0302 owns the
  defences (its F2), the versus step and the cap, and must agree.
- S-0306 claim 21 supplies the projectile's damage amount at detonation, including the loss
  during earlier non-detonating turns under its claim 18 (with its floor of 5 damage points).
  F5 also uses that then-current amount, not the amount at creation. Separately, whether a
  producer can change a per-projectile scaling factor from its initial 1.0 is not established.
  No rule here assumes that damage lost during flight is such a factor change; *D* remains
  the amount supplied by the producer. The possible factor change is `unmeasured`, confidence
  low: no producer or named-data setup that changes it is known.
- How a projectile can leave play during its own area damage (claim 21), which S-0306 owns.

## Questions for the project lead

No question for the project lead remains open in this revision.
