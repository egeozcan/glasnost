---
id: S-0303
title: Area damage (spread, falloff, cells scanned, walls, bridges and detonation effects)
topic: combat
revision: 3
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: [Q-0020]
supersedes: []
---

# S-0303: Area damage (spread, falloff, cells scanned, walls, bridges and detonation effects)

## Summary

How one detonation of a warhead at a point spreads its damage: which cells and objects it reaches,
the distance each object is measured at, how the damage falls off, which objects are left out, the
order of the hits, and what the detonation does to walls, bridges, resource cells and exploding
overlays; it also gives the explosion animation and the debris of a projectile's detonation. What
one hit does to one object (armour, multipliers, the health change) is damage resolution, S-0302,
and what happens when an object dies is S-0312.

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
  *source house*, and whether its producer allows resource reduction (claim 17; every producer
  named in this spec allows it). Projectiles (S-0306, S-0203 claim 5), damaging animations (S-0109 claim 17), death
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
- **Tracked flying object.** An object that the game registers as flying from the moment it lifts
  off until it lands: an aircraft-kind object in flight, a jumpjet between its lift-off and its landing, and a
  rocket-propelled missile object in flight (the movement specs own when each lifts off and lands).
  Each is registered in the *sector* (Formulas F7) of the cell it is over, which the game updates
  as it flies.
- **Sector.** One of the 20 × 20 rectangles of whole cells into which F7 divides the map for the
  flying-object search.

## Behaviour

### When area damage happens and what it reaches

1. **One detonation, all at once.** A detonation applies all of its effects at the moment it
   happens, inside the turn of whatever detonated (for a projectile, S-0203 claim 5), before any
   other object acts. Every hit, wall hit and other effect of claims 3 to 20 completes before the
   detonation ends, including any further detonation that a hit sets off (an object's death weapon,
   S-0312, or an exploding overlay, claim 19): such a nested detonation runs to its end at the
   moment it is set off, in the middle of the outer detonation's order (claim 11).
   - Backing: `measurable` (scenario A11: a death weapon going off between two hits of one
     detonation)
   - Confidence: high

2. **Detonations that do nothing.** The area damage of a detonation whose damage amount is 0, or
   whose warhead is absent, has no effect at all: no hit, no wall hit, no bridge or overlay effect,
   no particle system, and its result is *missed* (claim 23). On a map whose inert flag is set
   (Import mapping), the area damage of every detonation is like this. The explosion animation and
   debris of claims 21 and 22 belong to a projectile's detonation, not to the area damage. The
   inert flag does not suppress them. The animation is not chosen when the damage amount is 0 (F5),
   but the debris depends only on the warhead and is thrown whatever the damage amount. A
   projectile that has no warhead is a departure (Edge cases).
   - Backing: `measurable` (scenario A1, variant e, for the inert flag; A14, variant f, for the
     damage amount 0); `unmeasured` for the projectile without a warhead (the original stops, so
     there is no value to compare)
   - Confidence: high

3. **Spread radius and scanned cells.** Objects can be hit only if they are in a scanned cell (or
   are tracked flying objects, claim 6) and their epicentre distance is at most the spread radius *R* =
   trunc(256*S*) leptons (F1). The scanned cells are the cells whose ring metric from the centre cell
   is at most the scan radius *k* = trunc(*S* + 0.99) (F2), for *k* from 0 to 10; table T1 gives the
   exceptions for *k* = 11. The two limits are separate: *R* follows *S* exactly, while *k* rounds
   *S* up to the next whole cell when its fraction exceeds 0.01. An object at a distance at most *R*
   in a cell that is not scanned is not hit, and an object in a scanned cell further than *R* is
   not hit either.
   - Backing: `measurable` (scenario A2)
   - Confidence: high

4. **Cell layer.** When the centre cell carries a high bridge and *P* lies more than 208 leptons
   (half the height of a bridge deck, 416 leptons, S-0102 and glossary) above the terrain at the
   horizontal position of *P*, the detonation looks at the bridge-deck layer of every scanned cell,
   otherwise it looks at the ground layer of every scanned cell. It never looks at both. So a
   detonation on a bridge deck hits only objects on bridge decks, and one under or beside the deck
   hits only objects in ground layers, even objects standing on a bridge deck that is within *R*.
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
   (`quirks.ground_detonation_misses_aircraft`).
   - Backing: `measurable` (scenario A5 for the height condition; scenario A16 for the sectors)
   - Confidence: high for the height condition; medium for the set of tracked objects and for F7
     (Open questions: whether a tracked flying object can also appear in a scanned cell)

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
   the same detonation do nothing (claim 9). With S-0302's minimum for undemolishable buildings
   (claim 7), even hits that the falloff or the versus value bring to 0 cost such a building 1
   health point each.
   - Backing: `measurable` (scenario A3)
   - Confidence: high

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
     harvester flag; A3 for the destroyed building; A16, variant c, for the flying source object)
   - Confidence: high; medium for the exemption of tracked flying objects

10. **Aircraft in the air count at half distance.** An aircraft object that is in the air (Terms:
    S-0308's test) when it is
    hit has its epicentre distance halved, truncated toward zero, before the comparison with *R*
    and before the falloff of claim 12. This holds whether it was found by claim 6 or in a scanned
    cell. Jumpjets and other flying objects that are not of the aircraft kind keep their full
    distance.
    - Backing: `measurable` (scenario A5, variant c)
    - Confidence: high

11. **Order of the hits.** The candidates are fixed when the detonation happens, from the objects
    present then: an
    object that appears in a scanned cell while the hits are being delivered gets no hit from this
    detonation (State and ordering). They are hit one after another in this order: the tracked
    flying objects of claim 6, in the order of F7, then the objects of the scanned cells in scan
    order (table T1), within a cell in the order of claim 5.
    A building gets its hits at the places where its foundation cells come in that order. Each hit
    is resolved completely (S-0302) before the next, including any death it causes and the
    effects of that death (S-0312). So a death weapon set off by an earlier hit can kill or damage
    an object that a later hit of the same detonation would have reached; that later hit then does
    nothing to it (claim 9). The order also fixes the order of every random draw the hits take.
    - Backing: `measurable` (scenario A11)
    - Confidence: high for the order of cells and hits; medium for the order among several tracked
      flying objects (F7)

### How much damage

12. **Falloff.** A hit with damage amount *D* > 0 at epicentre distance *d* (0 ≤ *d* ≤ *R*)
    first goes through the target's defences, which S-0302 owns (its claim 5): the defended amount
    *d*₃ is *D* divided by the armour and never below 1 (Terms). The falloff then works on *d*₃,
    not on *D*, and gives the damage *D*₁ of Formulas F3: *d*₃ at the centre, falling linearly to the
    edge share *d*₃ × *p* at *d* = *R*, truncated toward zero, and never below 0. When *p* is exactly
    1, or *R* is 0, the hit carries *d*₃ unchanged. An edge fraction above 1 makes the damage rise
    towards the edge. The result goes on to the versus step of S-0302 (its claim 6): the original
    multiplies it by the versus value of the object's armour class, truncates toward zero and caps
    it at the game-wide maximum damage (F3, last line). S-0302 owns that step, every other
    multiplier and the minimum. So with a house and an object armour of 1.0, a target that is not
    prone and no veteran armour bonus, *d*₃ = *D*; in the other cases the order can matter, because *d*₃ is truncated
    before the falloff. The falloff is computed in floating point (F3).
    - Backing: `measurable` (scenario A1, variants a to d)
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
    the attacker's house takes no damage from the hit. What a detonation carries is S-0302's claim
    1: the firing object and its owner when the firing object is still in play at the detonation,
    otherwise neither. So when the firer no longer exists (for example a projectile whose firer has
    been destroyed, S-0203 claim 6d), the hit has no attacker and allies are damaged even by such a
    warhead. This spec adds nothing to the rule and has no switch for it.
    - Backing: `measurable` (scenario A8)
    - Confidence: medium (the test is clear; which detonations still carry their source object
      when the firer is gone is owned by the specs that create them)

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

17. **Resource cells.** For every scanned cell, in scan order, a cell whose overlay type has the
    chain-reaction flag loses trunc(*D* / 10) units of resource (nothing when the result is 0 or
    less), provided the overlay type is not a resource type or the warhead has the
    resource flag, and provided the producer of the detonation allows it (every producer named in
    this spec allows it). Only a cell that actually holds resource loses anything, so in effect the
    rule needs a resource overlay type with the chain-reaction flag and a warhead with the resource
    flag. (Craters remove resources by a rule of their own, S-0109 claim 18.)
    - Backing: `measurable` (scenario A10, variant d, with a test overlay type)
    - Confidence: medium

18. **Bridges.** When the map's destroyable-bridges setting is on and the warhead has the wall
    flag, a detonation whose centre cell is part of a bridge may destroy a section of it. For a high
    bridge, *P* must lie in a height window from (*h* − 2) × 104 + 416 (exclusive) to (*h* + 1) ×
    104 + 416 (inclusive) leptons, where *h* is the height level of the bridge cell; for a low
    bridge there is no height test. A destruction is attempted when a ranged draw *r* over 1 to the
    game-wide bridge strength (inclusive) gives *r* < *D*, or without a draw when the warhead is the
    game-wide ion-cannon warhead. Each bridge test that applies to the cell takes its own draw.
    Each attempt either succeeds or fails. Which it does depends on the bridge piece in the centre
    cell, whatever the draw gave, and a failed attempt can still change the bridge: for some pieces
    it alters the cell without succeeding. The ion-cannon warhead follows a failed attempt with
    further attempts at the same cell and no new draw, up to four attempts in all, stopping at the
    first that succeeds, so that it can complete a destruction that one attempt leaves unfinished.
    Any other warhead makes one attempt after its draw. Which attempts succeed, and what each
    attempt changes, belong to the bridge specs, which are not written.
    - Backing: `measurable` (scenario A12; statistical for the chance, and variant b for the
      repeats)
    - Confidence: medium (the chance and the flags); low (the exact set of bridge cells each test
      accepts, the height window, and the repeats)

19. **Exploding overlays.** When the centre cell holds an overlay whose type has the explodes flag
    (such as a barrel), that overlay is removed, the game-wide barrel-explosion animation appears at
    *P*, and a nested detonation follows at *P* with the game-wide ammunition-crate damage as its
    damage amount, the game-wide demolition warhead, no source object and the outer detonation's
    source house. After the nested detonation, at most one barrel debris animation appears, and
    then possibly the barrel particle system. With *n* the length of the barrel debris list and
    percent draws *r*₀, *r*₁, … (each a ranged draw over 0 to 99) belonging to its entries in list
    order, the debris animation is of the type of entry *i*\*, the least *i* with *r*ᵢ < 15, and the draws
    taken are *r*₀ … *r*ᵢ\*, that is *i*\* + 1 draws; when no *r*ᵢ is below 15, all *n* draws are
    taken and no debris appears. One further percent draw *r* follows, and the game-wide barrel
    particle system appears at *P* when *r* < 25. This happens whatever the sign of *D*. Only the centre
    cell is affected, so a barrel next to the centre cell is set off only by its own detonation
    chain.
    - Backing: `measurable` (scenario A13)
    - Confidence: high for the trigger and the nested detonation; medium for the draws

20. **Particle systems and rocking.** A warhead that names a particle system creates it at *P* at
    the end of the detonation, owned by the source house. A warhead with the rocking flag rocks the
    bodies of the objects drawn as voxel models (of any kind) in the 7 × 7 cells around the centre
    cell, in the same cell layer, when the rocking strength min(*D* × 0.01, 4.0) exceeds 0.3.
    The strength is formed in double precision from the integer *D*, rounded as S-0205 sets for
    floating point; whatever the rounding, it exceeds 0.3 exactly when *D* ≥ 31, and it reaches
    its cap of 4.0 at *D* ≥ 400. Objects in the centre cell are pushed away from the source object
    when there is one, the others away from *P* and only when *S* > 0. Rocking sets only the
    body-rocking rates that S-0106 draws (scaled by the type's weight), takes no draw, and is
    believed not to reach game state. The strength is passed on to that presentation as a number
    whose precision this spec does not fix.
    - Backing: `measurable` for the particle system (scenario A13, variant c); `unmeasured` for
      the rocking strength (presentation only)
    - Confidence: high for the particle system; low that rocking never reaches game state

### Detonation effects of a projectile's warhead

21. **Explosion animation.** After a projectile's area damage, its warhead's explosion animation is
    chosen by Formulas F5 from the projectile's damage amount *D*, the warhead's animation list and
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
    - Backing: `measurable` (scenario A14; variant e for the invisible projectile, through the
      recorder's stream position; variant h for the draw of a nullified electromagnetic
      detonation; variant i for *Q* against *P*); `unmeasured` for the projectile that leaves play
      (how that can happen is not established)
    - Confidence: high for F5; medium for the land-type, naval, *Q* and invisible-projectile rules,
      low for the projectile that leaves play

22. **Warhead debris.** A warhead with a positive maximum debris count throws debris when a
    projectile detonates: Formulas F6 gives the count and the types, all created at the projectile
    position *Q* (the metallic debris of F6 raised by 20 leptons), not at *P*. This happens
    whatever the damage amount, on an inert map as well, and not when the detonation was nullified
    (claim 21).
    - Backing: `measurable` (scenario A15, statistical; variant c for the position)
    - Confidence: high for the count draw; medium for the distribution over types and the position

23. **Result of a detonation.** A detonation reports one of three results to whatever produced it:
    *hit* (at least one object received a hit), *missed* (none did, or claim 2 applied) or
    *nullified* (claim 15). The projectile detonation uses it in claim 21; other producers say in
    their own specs whether they use it.
    - Backing: `measurable` through claim 21 (scenario A14, variants b and c)
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

> *D*ₛ = RZ₂₄(*d*₃)  (exact for *d*₃ < 2²⁴)
>
> *E* = RZ₂₄(*D*ₛ × *p*)
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
S-0302) and *M* the game-wide maximum damage. (*R* − *d*) and *R* are integers. Unit: health points.
Example with exact arithmetic (no rounding at these values), for *d*₃ = 100, *p* = 0.5, *R* = 512:
*d* = 0 gives 100, *d* = 256 gives 75, *d* = 511 gives 50, *d* = 512 gives 50. The order of the
steps matters: with *D* = 1000, an armour divisor of 3 (*d*₃ = 333), *p* = 0.5 and *R* = 1024, the
falloff at *d* = 512 gives 249; applying it to 1000 first (750) and dividing afterwards would give
250.

For *D* < 0 (claim 13): *D*₁ = *D* when *d* < 8, else 0; the defences leave the amount unchanged
(S-0302 claim 5) and there is no versus step and no cap.

**F4. Wall hit** (claim 16). Let *L* be the wall type's damage-stage count, *T* its wall strength, *s*
the cell's damage stage and *v* its overlay data value, with *s* = ⌊*v* / 16⌋ (S-0210 claim 8).

- *Success test.* An absolute wall hit always succeeds, with no draw, whatever the sign of the damage
  amount *D*. A wall hit that is not absolute has the amount *a* = *D*, except that *a* = −1 makes
  it absolute (claim 16). A wall hit of amount *a* ≥ *T* succeeds with no draw. A wall hit of
  amount *a* < *T* takes one ranged draw *r* over 0 to *T* (inclusive) and succeeds when *a* > *r*,
  so with probability *a* / (*T* + 1) for 0 ≤ *a* < *T*, and never for *a* < 0 other than −1 (the
  draw is still taken).
- *Success.* *v* increases by 16, so *s* increases by 1. If the new stage is *L* − 1 and *L* > 2,
  each of the four orthogonal neighbours (map north, map east, map south, map west, in that order)
  that holds a wall of the same overlay type at stage 0 takes a wall hit of 200 (this is S-0210 F3's
  neighbour hit, which may take its own draw).
- *Destruction.* After a success the wall is removed from the cell when the hit was absolute, when
  *s* ≥ *L*, or when *s* = *L* − 1 and *v* mod 16 = 0 (a wall with no link to a neighbour). A wall at
  stage *L* − 1 that still links to a neighbour stays until a further successful hit.

Exact integer arithmetic. Units: wall-hit points, damage stages.

**F5. Explosion animation** (claim 21). Let *D* be the projectile's damage amount, *W* the warhead,
*L* the land type of claim 21 (water, another land type, or none for air) and *Q* = (*q*ₓ, *q*ᵧ,
*q*_z) the projectile position. With *D* = 0 there is no animation and no draw. Otherwise the
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

**F6. Warhead debris** (claim 22). With *X* the warhead's maximum debris count (*X* > 0) and *Y* its
minimum debris count:

> *N* = R(*Y*, *X* − 1)  (one ranged draw; the bounds may come in either order, S-0202 claim 4)

All the debris below is created at the projectile position *Q* (claim 22).

If the warhead lists debris types *t*₀ … *t*ₘ₋₁ with maxima *u*₀ … *u*ₘ₋₁, the *N* pieces are dealt
out to the types in turn, *t*₀, *t*₁, … and round again after *t*ₘ₋₁, until none is left. Each turn
takes one raw value from the scenario random stream, read as a signed integer, and gives type *t*ᵢ

> *q*ᵢ = min(|raw| mod (*u*ᵢ + 1), pieces still left)

pieces (the "magnitude first" remainder of S-0202 claim 6), each one debris animation of that type.
If the warhead lists no debris types, the detonation creates *N* animations, each of the type at
entry R(0, *c* − 1) of the game-wide metallic debris list (*c* its length), at *Q* raised by 20
leptons. Exact integer arithmetic.

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

The candidates are the tracked flying objects registered in these sectors, sector after sector in
the order *C*, east run, north run, south run, west run, then the four diagonal groups (within a
group the diagonal sectors from *C* outwards, then the two extra sectors in the order given), and
within a sector in the order in which the objects were registered there. Since no diagonal group
reaches further than its two runs, which stop at the map's edges, the clamping never makes two of
these sectors the same, and each candidate appears once. Sectors further from *C* than these, for example
(−3, −1) from *C* when every run is at least 3 long, are not searched. Units: cells, sectors.

With `quirks.flying_search_sectors` off, the candidates are every tracked flying object, in the
order of registration, each once.

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
| Bridge repeats for the ion-cannon warhead | 3 | further attempts | claim 18; scenario A12 variant b |
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
  if the first destroyed it). Cells that are not in the list: (3, −11) and the metric-11 cells
  (±2, ±10) and (±3, ±10). The list also holds seven cells of metric 12: (±2, ±11), (−3, 11),
  (3, 11) and (−3, −11).

A scanned cell outside the map has no contents and takes part in nothing.

## State and ordering

**Order inside one detonation** (claims 1, 11 and 15 to 20). The following orders are observable
through the draws on the scenario random stream (the random-state recorder) and through nested
detonations:

- Before the first hit, every scanned cell has had its resource reduction (claim 17) and its wall
  hit (claim 16). The wall hits' draws (F4), and the draws of any neighbour hits they set off
  (S-0210 F3), all come before any draw of the first hit. The wall hits follow scan order (table
  T1).
- The candidates are fixed before the first hit (claim 11), so an object that appears in a scanned
  cell while the hits are delivered gets no hit from this detonation.
- The hits follow the order of claim 11. All draws and effects of one hit come before any draw of
  the next. These include the hit's damage resolution (S-0302), a death it causes and that death's
  effects (S-0312), and any nested detonation, which runs to its end.
- A nullified detonation (claim 15) does nothing after its hits: no bridge draw, no exploding
  overlay, no warhead particle system and no rocking.
- In any other detonation, these follow the last hit, in this order: the bridge draws (claim 18:
  one ranged draw for each bridge test that applies, none for the ion-cannon warhead); the exploding
  overlay of the centre cell, with its nested detonation and then its percent draws (claim 19); and
  last, the creation of the warhead's particle system (claim 20).
- Rocking (claim 20) takes no draw and is presentation only, so its place in this order is not
  observable and is not specified.

**Order of a projectile's detonation** (claims 21 to 23, with S-0306 for the rest). The random-state
recorder sees the draws of a projectile's detonation on the scenario random stream in this order:

| Order | Source of the draws | Draws on the scenario random stream |
|---|---|---|
| 1 | the effects that the special-warhead specs own (radiation, mind control and the like, which can replace the area damage) | as their specs say |
| 2 | the area damage above | as in *Order inside one detonation* |
| 3 | the nearby point of claim 21, for a projectile of the invisible kind only | one raw value |
| 4 | the choice of the explosion animation (F5) | as F5: one ranged draw when F5 reaches its electromagnetic case with a non-empty list, whatever the result of row 2, nullified included; none otherwise |
| 5 | the warhead debris (F6), only when the table below creates it | as F6 |

Before row 1, the warhead's screen-shake ranges take one ranged draw each, horizontal then vertical,
for each range that is not zero at both ends, from the unsynchronised random stream, so they reach
no game state and do not shift the scenario random stream. The result of the area damage decides
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
  of F7, so it can miss a flying object within *R*. Off: every tracked flying object is a candidate once, in registration order.
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
- **Departure: debris that can never be placed** (no switch). In F6, when every listed maximum is 0
  and *N* > 0, the original never finishes. Our engine stops after one full pass over the types that
  places nothing (having taken that pass's raw values), and the importer warns. A maxima list
  shorter than the types list is bad data: our engine treats the missing maxima as 0 and the
  importer warns.
- **Departure: negative damage and the animation list** (no switch). F5 with *D* ≤ −25 (*D* ≤ −35
  for the splash list) gives a negative entry, for which the original's result is undefined. Our
  engine shows no animation for a negative entry, and the importer warns about a weapon with
  negative damage whose warhead has an animation list.
- **Wall hits of healing detonations.** A wall hit with *D* < 0, other than −1 and other than an
  absolute hit, takes a draw (F4) and does nothing else (claim 16). Not a switch: it only matters
  for the order of draws, which our engine reproduces. *D* = −1 and the absolute flag are claim
  16's.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| rules, warhead section, `CellSpread` | spread *S* | decimal (single) | 0 |
| rules, warhead section, `PercentAtMax` | edge fraction *p* | decimal (single) | 1.0 |
| rules, warhead section, `Verses` | versus values *v* (F3; S-0003 claim 13 owns the parse, S-0305 the order of the armour classes) | percent list | 100 % each |
| rules, warhead section, `Wall`, `Wood`, `WallAbsoluteDestroyer` | wall flags (claim 16) | boolean | no |
| rules, warhead section, `Tiberium` | resource flag (claim 17) | boolean | no |
| rules, warhead section, `AffectsAllies` | affects-allies field (claim 14) | boolean | yes |
| rules, warhead section, `Rocker` | rocking flag (claim 20) | boolean | no |
| rules, warhead section, `Particle` | particle system (claim 20) | particle-system name | none |
| rules, warhead section, `Conventional` | conventional flag (F5) | boolean | no |
| rules, warhead section, `EMEffect` | electromagnetic flag (F5) | boolean | no |
| rules, warhead section, `AnimList` | animation list (F5) | animation-name list | empty |
| rules, warhead section, `MaxDebris`, `MinDebris` | *X*, *Y* (F6) | integer | 0, 0 |
| rules, warhead section, `DebrisTypes`, `DebrisMaximums` | debris types and maxima (F6) | voxel-animation list, integer list | empty |
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
| rules, the combat-damage section, `BridgeStrength` | bridge strength (claim 18) | integer | owned by the bridge specs |
| rules, the combat-damage section, `AmmoCrateDamage` | ammunition-crate damage (claim 19) | integer | owned by the crate specs |
| rules, the combat-damage section, `MaxDamage` | maximum damage *M* (F3; S-0302) | integer | S-0302 |
| rules, the combat-damage section, `SplashList` | splash list (F5) | animation-name list | empty |
| rules, the general section, `LightningWarhead`, `WeatherConBoltExplosion` | lightning warhead and its animation (F5) | warhead name, animation name | none |
| rules, the general section, `HarvesterUnit` | harvester list (claim 9) | vehicle-type list | empty |
| rules, the general section, `BarrelExplode`, `BarrelDebris`, `BarrelParticle` | barrel effects (claim 19) | animation name, voxel-animation list, particle-system name | none |
| rules, the general section, `MetallicDebris` | metallic debris list (F6) | animation-name list | empty |
| rules, the general section, `WeaponNullifyAnim` | nullify animation (claims 15 and 21) | animation name | none |
| map, the special-flags section, `Inert` | inert flag (claim 2) | boolean | no |
| map, the special-flags section, `HarvesterImmune` | harvester immunity (claim 9) | boolean | no |
| map, the special-flags section, `DestroyableBridges` | destroyable bridges (claim 18); the skirmish lobby's bridge setting is the map specs' | boolean | owned by the map specs |

The importer warns about a spread outside 0 to 11 cells, about a map with *W* + *H* − 1 < 20, and
about the debris (including an empty metallic debris list) and animation-list cases of Edge cases
and quirks.

## Measurement scenarios

All scenarios run under the glossary's *measurement setup* on purpose-made flat test maps, in a
skirmish with one human player and one passive computer player far away, at any game speed. Test
warheads and weapons are given in the test map's override sections (named test files); each scenario
names the fields it sets, with invented values. Health values are read with a *recorder* that reads
the game's state. Scenarios whose result depends on random draws are statistical, as stated.

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
923; falsified if the values at those three distances are those. *Variant e:* the same map as a
with the inert flag set, and a wall cell inside the spread with the wall flag set on the warhead
(expected: no health lost, the wall's stage unchanged). *Variant f (off-lattice
distances, 1023 and 1025):* the same warhead fired from the vertical test projectile of S-0306
claim 11, whose type's detonation altitude field is varied from run to run; the recorder reads *P*
and the targets' positions, and a target displaced by (3, 2) cells from the cell under *P* (horizontal distance 923) gives
distances √(923² + *h*²) for the height *h* of *P* above the target's position: a run with
*h* from 441 to 445 gives *d* from about 1023 to 1025 depending on the exact height reached (the projectile
climbs in steps, so the height comes from the recorded *P*; runs are repeated with neighbouring
altitude values until distances of 1023, 1024 and 1025 are all recorded). Expected: the loss
follows F3 at every recorded *d* up to 1024 and is 0 at 1025. *Runs:* 1 each (no draw involved for
vehicles; check S-0302 adds none).

**A2. Scanned cells versus radius (claim 3).** Wall hits (claim 16) take no distance test, so a
wall shows directly whether its cell is scanned. *Setup:* a field of stock wall cells around the
detonation cell; a test warhead with the wall flag and a damage amount at least the wall strength (no
draws, F4), fired at the centre of the centre cell, with spreads 0.01, 0.02, 1.01, 1.02 and 3.5.
Separately, a vehicle on the centre of a ring-1 cell (256 leptons away) with spreads 0.99 (*R* =
253) and 1.0 (*R* = 256), and a warhead without the wall flag. *Observable:* which wall cells change
stage; whether the vehicle is hit. *Expected:* spread 0.01 changes only the centre cell; 0.02 and
1.01 rings 0 and 1; 1.02 rings 0 to 2; 3.5 every cell of ring metric 4 or less, including (3, 3) at
4.24 cells, but not (4, 2) at 4.47 cells. The vehicle is not hit at 0.99 (scanned but beyond *R*)
and hit at 1.0. *Falsified if:* any wall outside those rings changes, a wall inside them does not,
(4, 2) changes at 3.5, or the vehicle result differs.

**A3. Buildings: one hit per foundation cell (claims 7, 8, 9).** *Setup:* test weapon with damage 100
and a test warhead with spread 3 (*R* = 768), edge fraction 1.0, versus 100 % for the building's
armour class, buildings with armour 1.0 and an owner whose country factor for buildings is 1.0; one
1 × 1 and one 3 × 3 stock building of the same armour class, each fully inside the spread of
separate detonations at the same distance from their reference points. *Variant b:* the 1 × 1
building only, edge fraction 0.5, and a detonation above the centre of its cell (the vertical test
projectile of S-0306 claim 11, with a detonation altitude field chosen by dry runs until the
recorder reads *P* at about 300 leptons above the ground over that cell; the projectile type is
not of the airburst kind). *Observable:* health lost. *Expected:* the 1 × 1 building loses 100; the
3 × 3 building loses 900 when all nine cells are within *R* (S-0302's per-hit rules applied to each
hit). Variant b: F3 with *d* = ⌊*r*(*n*)⌋ − 208 for the distance the recorder gives; with a
measured distance of 300 that is *d* = 92 and a loss of 94, against 80 if the full distance were
used and 100 if *d* were 0. *Falsified if:* both buildings lose the same amount (main part), or in
variant b the loss matches the full distance or *d* = 0.

**A4. Bridge-deck layer (claim 4).** *Setup:* a high bridge; one vehicle on the deck and one on the
ground under the deck in the same cell; detonations (spread 2, edge fraction 1.0) once at the deck
surface and once at ground level. *Observable:* the health of each vehicle. *Expected:* the deck detonation hits only the deck vehicle, the
ground detonation only the ground vehicle. *Falsified if:* either detonation hits both.

**A5. Flying aircraft (claims 6, 10).** *Setup:* a stock aircraft hovering or circling at known
height near a ground target; (a) a ground-level detonation with spread 10 directly below it; (b) a
detonation at a set height with the same spread, from the vertical test projectile of S-0306 claim
11 (its detonation altitude field set, and not of the airburst kind of S-0306 claim 21, which would
launch child projectiles); (c) the same projectile with the aircraft at a known distance and edge
fraction 0.5. *Observable:* health lost by the aircraft. *Expected:* (a) no damage to the aircraft,
(b) damage; (c) damage as F3 with half the measured distance. *Falsified if:* (a) damages it, (b)
leaves it undamaged, or (c) matches the full distance. *Runs:* 3 each.

**A6. Source object and flags (claim 9).** *Setup:* a vehicle that fires a test weapon (spread 2) at
a cell next to itself; variant b gives its type the self-damage flag; variant c places a stock
harvester-list vehicle in the spread with the map's harvester-immunity flag set, and variant d
without it. *Variant e:* a building whose type is invisible in game, inside the spread; *variant f:* a vehicle
inside a transport (held out of play) that is inside the spread. *Observable:* health lost by the
firer and the harvester (variants e and f: by the building and the passenger). *Expected:* the firer is not hit (a), is hit (b); the harvester is not hit (c), is hit
(d); the invisible building and the passenger are not hit (e, f). *Falsified if:* any differs.

**A7. Healing point (claim 13).** *Setup:* a test weapon with damage −50, spread 3, fired at a
damaged vehicle; other damaged vehicles at 300 leptons from the detonation point and, in repeated
runs, the vertical test projectile of S-0306 claim 11 detonated 7 and then 8 leptons above a damaged
vehicle (the recorder reads the distance, so cell-centred vehicles need not stand at 7 and 8
leptons). *Observable:* each vehicle's health. *Expected:* only targets at *d* < 8 heal, each by 50
(vehicles only: an undemolishable building would lose 1 instead, S-0302 claim 7). *Falsified if:*
the target at 8 or 300 heals, or a target below 8 leptons does not heal by exactly 50.

**A8. Allies (claim 14).** *Setup:* a test warhead with the affects-allies field off; an allied
house's vehicle in the spread. (a) The firer alive at detonation; (b) a slow projectile whose firer
is destroyed before it lands. *Observable:* the ally's health. *Expected:* (a) no damage to the ally; (b) the ally is damaged.
*Falsified if:* (a) damages it or (b) does not.

**A9. Curtain protection (claim 15).** *Setup:* a vehicle under the iron-curtain effect, and an
unprotected vehicle within *R* of the detonation point (not necessarily in the same cell, so that
S-0209's occupancy rules are not strained); (a) a small detonation (spread 0.5) within 85 leptons of
the protected vehicle; (b) spread 0.6; (c) the same as (a) under a force shield instead.
*Observable:* the unprotected vehicle's health and the animations the recorder sees created.
*Expected:* (a) the unprotected vehicle takes no damage and the nullify animation shows; (b) and
(c) it takes damage. *Variant d:* the protected object is a tracked flying object, with the
detonation raised above the terrain as in scenario A5 (b), spread 0.5, 84 and then 86 leptons from
*P*: nullified at 84, not at 86. *Variant e:* the protected vehicle in the centre cell 84 leptons
from *P* with spread 0.3 (*R* = 76, so the vehicle is beyond *R*): still nullified. *Falsified if:*
any differs.

**A10. Walls and resources (claims 16, 17).** *Setup:* a line of stock walls through the spread,
(a) a warhead with the wall flag and damage at least the wall strength (no draws): the stage of
every scanned wall cell rises by one, whatever its distance; (b) damage below the strength:
statistical over 200 runs, share of walls damaged ≈ *a* / (*T* + 1) within 3 standard errors,
(c) the absolute flag: every scanned wall cell is removed; (d) a test overlay type with the
chain-reaction and resource flags and resource amount, and a warhead with the resource flag (claim
17): it loses trunc(*D*/10) units; (e) healing
warheads with the wall flag (damage −50 and then −1) and one with the absolute flag and damage −50:
the wall's stage is unchanged at −50 (a draw is taken, seen through the recorder), and the wall is
removed at −1 and with the absolute flag. *Observable:* wall stages and removals, and the
resource amount. *Falsified if:* a wall outside the scanned cells changes, the shares differ,
variant d loses an amount other than trunc(*D*/10), or variant e differs.

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
observable is the state of the bridge cells afterwards. *Expected:* for some cell position the
ion-cannon detonation leaves more destroyed than the single attempt. *Falsified if:* the two agree at
every position (the repeats are then unobservable, and the constant for them goes).

**A13. Exploding overlays (claims 19, 20).** *Setup:* a stock exploding overlay (barrel) in the
centre cell of a detonation with a warhead that cannot damage overlays otherwise, and another in a
neighbouring scanned cell. *Observable:* whether each barrel explodes, the health lost by the
vehicles near the centre barrel, and the debris animations and particle systems that the recorder
sees created. *Expected:* the centre barrel explodes (nested detonation damages nearby
vehicles by the ammunition-crate damage scaled as F3 with the demolition warhead); the neighbour
barrel explodes only if the nested detonation's own centre cell holds it (it does not), so it stays.
Statistical over 100 runs: debris appears in a share 1 − 0.85ⁿ of the runs (*n* the length of
the barrel debris list) and the particle system in 25 %. Variant c: a warhead naming a particle
system shows it at *P*. *Falsified if:* the neighbour barrel explodes from the outer detonation, or
the share of runs with debris differs from 1 − 0.85ⁿ, or the share with the barrel particle system
differs from 25 %, by more than 3 standard errors, or in variant c the warhead's particle system
does not appear at *P*.

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
still thrown. *Variant g (negative damage):* damage −24 with the animation list of 4: entry 0.
*Variant h (nullified, electromagnetic):* variant a of scenario A9 with an electromagnetic warhead
whose projectile hits: the recorder's stream position advances by one raw value even though the
nullify animation is the one that shows. *Variant i (*Q* against *P*):* a projectile with cluster
count 3 whose own position is over land at the shore, repeated until the recorder reads further
detonation points over water, with a splash list and the conventional flag: every detonation's
animation comes from the warhead's own list (the land type is that of the projectile's own
position), and each is shown at its own detonation point. *Variant j (the air rule, claim 21):* over water with the conventional flag, a splash list of 3, and
the vertical test projectile of S-0306 claim 11 detonating 207 and then 208 leptons above the
terrain (the recorder reads the height): the splash animation at 207, the warhead's own list at
208. *Falsified if:* any entry differs, variant e takes no extra
raw value, variant j differs, variant f shows an animation or no debris, variant h takes no draw, or a splash
animation appears in variant i.

**A15. Warhead debris (claim 22).** *Setup:* a test warhead with minimum debris 2, maximum 6 and no
debris types; counted over 200 detonations. *Observable:* the number, types and positions of the
debris animations that the recorder sees created by each detonation. *Expected:* counts 2 to 5 only (R(2, 5)), roughly
uniform. Variant b: two debris types with maxima 1 and 3, with the recorder reading the raw values
of the scenario random stream; expected: the two types together number 2 to 5 pieces per
detonation, and each type's count is the one F6 gives for the raw values read. Variant c (position, claim 22): the projectile
of variant i of scenario A14 with this warhead: the recorder reads the debris positions, which
equal the projectile's own position for every detonation of the cluster (the metallic ones 20
leptons higher). *Falsified if:* a count of 6 appears, or 1, or in variant b a type's count differs
from F6 for the recorded raw values, or the debris of a further detonation appears at that
detonation's point.

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
firing aircraft is damaged. *Falsified if:* the off-diagonal aircraft is damaged, the diagonal or
variant-b aircraft is not, or c leaves the firer undamaged. *Runs:* 3 each.

## Open questions

- Whether a tracked flying object can also be listed in a scanned cell's layer while it flies (it
  would then be hit twice, once at the halved distance if it is of the aircraft kind). Scenario A5
  with an aircraft directly over a scanned cell, and A16 with a jumpjet, would settle it.
- Exactly when each kind of flying object is registered and unregistered (lift-off, landing,
  leaving the map), which the movement specs own; F7 relies only on the registration order. In
  particular, where an object that crosses into a sector in flight comes in that sector's order
  (after the objects already registered there, or by its first registration), which fixes the
  hit order among tracked flying objects (claim 11, medium confidence).
- The exact set of bridge cells each bridge test of claim 18 accepts, what destroying a section
  does, which attempts fail and what a failed attempt leaves behind (claim 18); this belongs to the
  bridge specs, which are not written.
- The boundary with S-0302: this spec states the falloff on the defended amount and, for completeness,
  the versus step and cap as the original computes them in one expression (F3); S-0302 owns the
  defences (its F2), the versus step and the cap, and must agree.
- The original hands a projectile's damage to the area damage after scaling it by a per-projectile
  factor, a multiple of 1/256, which is 1.0 when the projectile is
  created; the explosion animation (F5) uses the unscaled damage. This spec takes *D* as passed to
  the area damage and assumes the factor stays at 1.0, so that both are the same amount. Whether
  any producer changes the factor is for S-0306 to say.
- How a projectile can leave play during its own area damage (claim 21), which S-0306 owns.
- Rocking (claim 20): whether a rocked body's tilt can reach game state (for example through a
  firing angle).

## Questions for the project lead

- The allies exception has no switch. If the lead wants one for a detonation whose firer is gone,
  S-0302 owns it (S-0302 claim 1 and the allies row of its claim 8), for example a switch that keeps
  the firing house recorded at launch.
- Three behaviours of the original could each get a quirk switch whose default is the original: an object
  within *R* in a cell that is not scanned is missed (claim 3); a detonation beside a high bridge
  misses objects on its deck (claim 4); only a barrel in the centre cell explodes (claim 19). They
  are fixed behaviour until the lead decides.
