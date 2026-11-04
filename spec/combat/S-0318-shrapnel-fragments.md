---
id: S-0318
title: Shrapnel fragments (when a detonation bursts into further shots, how many, and where they go)
topic: combat
revision: 29
sources-class: [decomp, design, measurement]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-04
measurements: [M-0021]
requests: [Q-1714, Q-1825]
supersedes: []
---

# S-0318: Shrapnel fragments (when a detonation bursts into further shots, how many, and where they go)

## Summary

A projectile type may name a *fragment weapon* and a *fragment count*. When such a projectile
detonates, it first launches up to that many *fragments*, each a projectile of the fragment weapon,
and only then does its own area damage. Fragments go first to objects in the cells around the
burst that the firer's house does not ally (enemy, neutral, civilian and terrain objects alike),
nearest ring first, each to the object listed first in its cell. Fragments that find
no such target go to random cells near the burst, whatever stands there.

So, under the conditions below, one shot can damage its own target and up to *n* further objects,
with the timing of claim 5 when the fragments are invisible. The shot detonates once and is not
an airburst projectile (S-0306 claim 21). Its warhead and its fragment warhead each damage a
single object. Its fragments do not burst again (claim 7), each detonates once, and none is an
airburst projectile. Here *n* is the fragment count when that is 0 or more; for a negative count,
*n* is at most the larger of its magnitude and 3 (claim 2). A fragment carries the fragment
weapon's damage field unscaled, and the projectile's attacker as its own.

This spec gives the rules that decide how many objects one shot's fragments damage, which bears
on Q-1714. M-0021 measures that count on a test pairing built after Q-1714's. For the stock
pairing that Q-1714 names, the count is our engine's own comparison (Open questions). This spec
owns the selection of the fragments' targets, their draws and their launch, which S-0306 leaves open
(Changes for other specs). The look of a fragment's laser beam or electric bolt belongs to S-0518
and S-0316; this spec says which fragments make one (claim 8).

## Terms

- **Fragment weapon, fragment count.** The two fields of a projectile type that this spec is about
  (Import mapping). A projectile type without a fragment weapon launches no fragments.
- **Fragment.** One projectile launched by claim 1. Its type is the fragment weapon's projectile.
- **Burst point, burst cell.** The detonating projectile's own position when it detonates, and the
  cell containing it. This is its position after its moves (S-0306 T1 and T2) and after S-0306
  claim 20's raise (a) and placement on the target (b), but before that claim's change (c): (c) moves only the
  detonation point, where the projectile's own area damage and explosion take place, and never
  the burst point (claim 1). For a projectile that detonates several times (S-0306 claim 21
  clusters), each detonation bursts from that same position, not from the cluster's later
  detonation points.
- **First-listed object.** The object listed first in the ground layer of a cell, in S-0209's
  listing order (S-0209 claim 13): the most recent non-building entrant first, buildings last. A
  cell with no listed object has none. A cell outside the map has none.
- **Firer's view.** The alliance test of claim 3 is made from the house of the projectile's
  attacker as it is at the burst: that house allies the object's owning house (glossary
  *alliance*). An object without an owning house, such as a terrain object, is never allied.
- **Fragment range *k*.** The fragment weapon's range in leptons, divided by 256 and truncated
  toward zero: whole cells (F2).
- **Chain level.** The projectile that bursts first, which is not itself a fragment, has level 0,
  and a fragment launched by a projectile of level *d* has level *d* + 1 (claim 7).

## Behaviour

### When fragments are launched

1. **Which detonations burst.** A detonation of a projectile whose type has a fragment weapon
   launches fragments by claims 2 to 5, unless one of these holds:
   - (a) the detonation selects one of the special hits of S-0514 claim 20's table, which replace
     the area damage (S-0303 claim 2 and its order table, row 1): takeover, bomb planting,
     electric charging, parasite attachment, temporal erasure, locomotor replacement, air-strike
     designation, direct rocking at a vehicle target, bomb disarming, disguise-making and
     nuke-making. As in that table, a row is selected by its warhead flag whether or not its
     effect can succeed. The radiation of the weapon that the bursting projectile carries (S-0525
     claim 1), which is created before this test, and the electromagnetic-pulse warhead do not
     stop fragments. A fragment carries no weapon (claim 5), so a fragment that bursts again
     leaves no radiation before its own fragments.
   - (b) the burst cell has no first-listed object, or its first-listed object is a building.
   - (c) the fragment count *n* of claim 2 is 0 or less.

   When it holds, nothing of this spec happens and no draw is taken for fragments. The fragments
   of one detonation are launched after the warhead's radiation effect and before the area damage
   (S-0303 order table, between rows 1 and 2), so their draws come before every draw of the
   area damage and of the rows after it. Each detonation of a cluster tests (a) to (c) afresh and
   bursts afresh. An airburst projectile (S-0306 claim 21) detonates once, and that detonation
   tests (a) to (c) like any other. Its fragments are launched before its area damage, and so
   before the nine projectiles of its airburst weapon, which it launches after the area damage.
   Each of those nine is a projectile of its own, which bursts by this spec at its own detonation
   when its type has a fragment weapon.

   Conditions (a) to (c) are the only ones. A detonation whose own area damage does nothing (S-0303
   claim 2: a damage amount of 0, or a map whose inert flag is set) still bursts, and its
   fragments are launched as usual. A fragment's detonation is an ordinary detonation, so on a map
   whose inert flag is set the fragments' own area damage does nothing either. A projectile
   without a warhead is S-0303's departure and is not covered here.

   The burst cell of (b), the ring centre of claim 3, the distance of claim 2 and the fragments'
   launch direction (claim 6) all use the burst point (Terms), which S-0306 claim 20 (c) does not
   move. The fragments' starting position (claim 5) is the burst point too, except in height: a
   burst point that is not above the terrain is raised as claim 5 gives (S-0306 claim 8). The
   projectile's own area damage and explosion use the detonation point that (c) gives.
   The two differ only where (c) changes the point, as for a shot at a building whose art
   gives it a non-zero aim shift (S-0306 Terms, *target point*): a shot placed on the building's
   reference point by S-0306 claim 20 (b) does its area damage at the building's target point,
   but bursts from the reference point, inside the building's own cell, where (b) above stops the
   burst when the building is first-listed there.
   - Backing: `measurable` (scenario SH1 parts a, b, c and l; SH1 part o for a count of 0 and
     SH2 part c for a negative one; SH1 part n runs 1 and 2 for the electromagnetic-pulse warhead
     and radiation; SH1 part c with the random-state recorder for the place before the draws that
     follow the area damage, and SH1 part p, whose area damage takes a wall-hit draw, for the place
     before the area damage itself; SH1 part n run 3 for the bomb-planting row of (a); SH5 part b
     for clusters and SH5 part c for an airburst; SH1 part r for the burst point against the
     detonation point; SH1 part s for a damage amount of 0 and for the inert flag). `measured`
     (M-0021) for a burst at a positive count by a shot at a vehicle target, every judged shot of
     its test pairing bursting, and for no fragments from a projectile that names no fragment
     weapon (its part A0). M-0021 records no burst cell or burst tick, so a target listed in the
     burst cell is a case it does not observe: that case is `measurable` (SH1 part a).
     `unmeasured` for the other special-hit rows of (a): each needs the firer and target that its
     own spec gives it before its row is selected (S-0514), so no part here tests them, and they
     rest on the same row order of S-0303's table as the bomb-planting row
   - Confidence: high for conditions b and c, for the place before the area damage and before
     the draws that follow it, and for a burst whose area damage does nothing; medium for the
     burst point against the detonation point, which rests on S-0306 claim 20 (medium), until SH1
     part r runs; medium for the list in (a), whose other special-hit rows no part
     tests, and for the cluster and airburst cases

2. **How many.** Let *c* be the fragment count field.
   - *c* ≥ 0: *n* = *c*.
   - *c* < 0 and the projectile still has its attacker (S-0302 claim 1): *n* = |*c*| − ⌊*d* / 256⌋,
     where *d* is the spatial distance in leptons from the burst point to the attacker's reference
     point, taken with S-0204's table square root (F1). A shot fired from further away bursts
     into fewer fragments, and none at |*c*| cells or more.
   - *c* < 0 and the attacker is gone: *n* = 3.
   - Backing: `measured` (M-0021) for *n* = *c* at a count of 3: every judged burst of its test
     pairing launched 3 fragments. `measurable` (scenario SH2: parts a to c for the count, part d
     for the truncation of a distance that is not a whole number of cells; part e, with a one-cell
     building that house A builds during the run, for the absence of a building adjustment; SH3
     part b for the attacker gone). `unmeasured` for the height term of the
     distance and for the reference point of a vehicle attacker or of a larger building: SH2
     runs on flat ground with a one-cell building, so it cannot tell the spatial distance from
     the planar one, nor a vehicle's reference point from another of its points at ground level,
     and S-0204 claim 4 adds nothing to the position of a one-cell building. A part on
     raised ground would need the height of one cell level in leptons and the burst point's
     height, which this spec does not fix
   - Confidence: high for the count, its three cases and the truncation; medium for the absence
     of a building adjustment, until SH2 part e runs; medium for the height term and the
     reference point

### Where fragments go

3. **Fragments at enemies.** While the projectile still has its attacker, the cells of rings 1 to
   *k* (F2) around the burst cell are visited in the scan order of S-0303 table T1 (ring 1 first; the
   burst cell itself, ring 0, is not visited). In each visited cell, the first-listed object is a
   *candidate* when it is not the attacker itself and it is not allied from the firer's view. Each
   candidate receives one fragment, at once and in visiting order, until *n* fragments have been
   launched or the last ring has been visited. Only the first-listed object of a cell can be a
   candidate, so a cell whose first-listed object is the firer's own or allied gives no fragment,
   even if an enemy is listed after it. No other test applies. A candidate need not be a legal
   target of the fragment weapon: S-0306 claim 23's anti-air and anti-ground rows and S-0304's
   zero versus value play no part, and neither does whether the firer's house can see the object
   (shroud, cloaking). The alliance test reads the object's true owning house, so a disguise plays
   no part either. A candidate whose armour the fragment warhead cannot damage still receives its
   fragment, which then does what S-0302 and S-0303 say for that warhead. Only an object that
   S-0209 claim 13 lists in the cell's ground layer can be a candidate, so an ordinary aircraft at
   positive listing height, which that claim does not list, never is. Consequences:
   - neutral and civilian objects are candidates unless the firer's house allies their owning
     house (S-0412 claim 8), and terrain objects, which have no owning house, always are.
   - a building is listed last in each of its foundation cells, so it is a candidate in every
     visited cell where nothing else is listed: one building can receive several fragments.
   - an object is a candidate at most once per visit of a cell, so a vehicle or infantry unit,
     which is listed in one cell, receives at most one fragment from one burst's scan, with one
     exception: S-0303 table T1 lists one cell of ring 11 twice, and that cell is visited twice,
     so its first-listed object, if it is a candidate, receives a fragment at each visit, two in
     all (as it takes two hits of area damage there, S-0303). Nothing in the scan remembers an
     object already given a fragment in the same burst. The ring order is S-0303 table T1's, so
     S-0303's `quirks.spread_ring_eleven` governs it here too (Edge cases).
   - when the projectile's own target is listed in the burst cell (as it is after an invisible
     projectile's detonation, S-0306 claim 9), it receives no fragment from the scan. A projectile that bursts
     short of its target, or a target that has moved on, can leave the target in a visited cell,
     and then it is a candidate like any other object.

   When the projectile no longer has its attacker, no cell is visited and every fragment goes by
   claim 4.
   - Backing: `measured` (M-0021) for 3 fragments to 3 distinct enemy vehicles, none to the
     shot's own target, when 3 or more candidates stand in the rings around the target's cell as
     read from positions at the shot's tick. M-0021 records no burst cell, so the target's
     exclusion as an object listed in the burst cell is `measurable` (SH1 part a), not measured
     there.
     M-0021 settles neither the visiting order nor the alliance test: in its judged shots no cell
     holding only the firer's vehicles came before the third candidate.
     `measurable` (scenario SH1 parts a, d, e and f; parts g and h for an alliance in
     one direction only; part i for a fragment range that is not a whole number of cells; part j
     for ring 11's twice-visited cell; part k for one building receiving several fragments; part
     q for a candidate that the fragment warhead cannot damage; SH3 for the attacker gone; SH2
     part a for the attacker itself; SH8 parts a and b for the absence of a visibility test, part
     c for disguise, part d for the anti-air and anti-ground rows, part e for aircraft at positive
     listing height). A house counts as allied with itself (glossary *alliance*), so the alliance
     test alone already excludes the attacker; SH2 part a backs the outcome, which is the same
     under either reading.
   - Confidence: high for the scan, the first-listed rule, the alliance test and the attacker
     itself; medium for the absence of a visibility test, for disguise, for the anti-air and
     anti-ground rows and for aircraft, until SH8 runs

4. **Fragments at random cells.** When claim 3 has launched *t* < *n* fragments, the remaining
   *n* − *t* fragments are launched one after another. For each, two ranged draws (S-0202 claim 4)
   are taken from the scenario random stream, Δ*x* = R(−2, 2) and then Δ*y* = R(−2, 2), and the
   fragment is aimed at the cell (*x* + Δ*x*, *y* + Δ*y*), where (*x*, *y*) is the burst cell. The
   aim is the cell itself, not an object in it: such a fragment detonates as a projectile aimed at
   that cell (S-0306), whoever stands there, the firer's own objects included, and its warhead does
   what S-0302 and S-0303 say for such a detonation. The draws of one fragment are taken before
   the next fragment's. The burst cell can be drawn (Δ*x* = Δ*y* = 0).
   - Backing: `measurable` (scenario SH1 parts c and i, with the random-state recorder; SH3)
   - Confidence: high for the draws, their range and their count; medium for their order (Δ*x*
     before Δ*y*) and their pairing per fragment, which show only when an object stands in a drawn
     cell, and no part places one there; medium for what the fragment does on landing, which
     follows S-0306 and S-0303

### What a fragment is

5. **A fragment's fields.** A fragment is a projectile of the fragment weapon's projectile type. It
   carries:
   - the fragment weapon's damage field as its damage, unscaled: none of the factors of S-0302
     claim 2 (firepower multipliers, veteran combat factor, garrison, bunker or open-topped
     multipliers) applies, because a fragment is not a shot.
   - the fragment weapon's warhead and its brightness flag.
   - as its speed, the fragment weapon's weapon speed *S*<sub>w</sub> of S-0306 claim 1: the
     imported speed field when the fragment's projectile type has a non-zero turn rate, and
     *v*(*R*₀, *g*) from the fragment weapon's range when the turn rate is 0, in which case the
     imported speed field plays no part; *g* is S-0306's floater gravity *g*<sub>f</sub> when the
     fragment's projectile type is a floater, as in S-0306 claim 1. Neither the shot speed of S-0306 claim 1 nor the cap of
     its claim 2 applies, because a fragment is not a shot.
   - the projectile's attacker as its attacker (S-0302 claim 1), so the hits it delivers are
     credited as the projectile's own would be.

   It carries no weapon. So its detonation leaves no radiation, even when the fragment weapon has
   a radiation level: S-0525 claim 1 leaves radiation only where the detonating projectile carries
   a radiation weapon, as for the nine children of an airburst projectile.

   It is placed at the burst point, raised as S-0306 claim 8 gives: a fragment whose burst point is
   not above the terrain starts 1 lepton above it, at the same *x* and *y*, so a burst at height 0
   gives fragments that start at height 1. It enters play at once (S-0306 claim 8): it joins the
   end of the update order during the bursting projectile's turn, the fragments of one burst in
   launch order (State and ordering), so it is eligible for a turn later in the same turn pass.
   That projectile then does its area damage and leaves play in the same turn, which is one
   counting removal of S-0203 claim 10: the object that directly follows it at the end of that
   turn misses its turn in this tick. When nothing entered play after the projectile before its
   fragments, that object is the first fragment. An invisible fragment (S-0306 claim 9) whose
   projectile type has an arm time of 0 (S-0306 claim 9: the arm field's absent-key default)
   detonates in its first turn (S-0306, State and ordering) and leaves play in it, which is again
   one counting removal, so the fragment that directly follows it misses its turn in that tick
   too. With S-0203's removal skip (`quirks.update_order_removal_skip`, on by default) and nothing
   else entering or leaving play, the fragments of one burst therefore detonate as follows,
   counted in launch order:
   - in the tick of the burst, the 2nd, 4th, 6th … fragments, and
   - in each later tick, of the fragments still waiting, the 1st, 3rd, 5th … in launch order,
     each one's removal costing the next waiting fragment its turn.

   So with 3 such fragments, the 2nd detonates in the tick of the burst, the 1st one tick later
   and the 3rd two ticks later; with 8, the last detonates three ticks after the burst. Every one
   of these detonations comes after the area damage of the projectile that launched them. When
   another object directly follows the projectile (a shot animation that entered play after it,
   S-0109 claim 29), that object misses the turn instead and the pattern starts one place later:
   the 1st, 3rd, 5th … fragments detonate in the tick of the burst. A kill by a fragment, or any
   other counting removal in a fragment's turn, costs further turns by S-0203 claim 10, which
   decides every case not listed here. With the removal skip off, every such fragment detonates
   in the tick of the burst, in launch order. With a positive arm field, when it detonates
   follows S-0306's arm-time rows, which this spec does not restate.
   - Backing: `measured` (M-0021) for the fragment weapon's warhead and for the attacker object:
     in its test pairing's part A, each judged shot's 3 further hits carry the fragment weapon's
     warhead, which the firer never fires itself, every such hit names an object of the firer's
     house as its attacker, and each judged shot's 3 name its firer. `measurable`
     (scenario SH1 part a, both runs, for the timing; SH4: part a for the damage, part c for the
     fragment weapon's warhead, part b for the credit to the attacker's house). `unmeasured` for
     the brightness flag (an invisible fragment draws nothing, and no scenario captures a visible
     fragment's brightness) and for the speed (an invisible
     fragment's speed has no effect, and SH7 judges only the horizontal direction); `measurable` for
     the absence of radiation (scenario SH1 part n, run 4). The raise of a burst point that is not
     above the terrain is `unmeasured`, as S-0306 claim 8 gives: no scenario places a fragment's
     start at or below the ground
   - Confidence: high for the damage, the warhead and the credit to the attacker's house; medium
     for the place in the update order until SH1 part a runs (its run 1 alone gives the same result
     for a place right after the bursting projectile when that projectile is last; run 2, whose
     shot animation directly follows the projectile, tells the two places apart, since fragments
     placed before the animation would give run 1's ticks), which follows S-0203 claim 4 as S-0109 claim 29 does for
     an animation that enters play in a projectile's turn; medium for the turn that the bursting
     projectile's removal costs, as S-0203 claim 10 rates its causes; medium for the resulting ticks until SH1 part a runs,
     since they also rest on the first-turn detonation of S-0306; medium for the brightness
     flag; high for the attacker object; high for the absence of radiation; low for the speed,
     as in claim 6, and medium for the raise of a burst point that is not above the terrain, as
     S-0306 claim 8 rates it

6. **Launch direction.** A fragment starts from its starting position (claim 5) and is launched
   towards its aim point's reference point, climbing at a fixed elevation of about an eighth of a
   turn (45°), not at the pitch of S-0306 claim 6 (Changes for other specs). The direction is taken
   from the burst point as it is, not from the raised starting position (S-0306 claim 8). The two
   differ only in height, and only for a burst point that is not above the terrain. For an
   invisible fragment the velocity has no observable effect (S-0306 claim 9 moves it at once).
   Only the direction is established; the elevation is not measured. The velocity's length is
   believed to follow claim 5's speed, with a length of 100 in place of a speed of 0 as for a
   shot (S-0306 claim 2), but how the velocity of a fragment
   that is not invisible is formed, its length included, and whether its flight rule keeps that
   velocity in its first turn, are not established (Open questions).
   - Backing: `measurable` (scenario SH7) for the horizontal direction. `unmeasured` for the
     elevation: SH7's first move also depends on the flight rule's first turn, which is not
     established. `unmeasured` for the velocity's length and the length 100 at a speed of 0,
     which SH7 does not measure. `unmeasured` for the use of the unraised burst point, as S-0306
     claim 8 gives for its claims 5 to 7: for a burst point at the surface the two points differ by
     1 lepton, which no recorded direction tells apart.
   - Confidence: low, except medium for the use of the unraised burst point, as S-0306 claim 8
     rates it

7. **Fragments of fragments.** A fragment whose own projectile type has a fragment weapon bursts
   in its turn by claims 1 to 6, with the original attacker still as attacker; its fragments are
   one chain level below it (Terms). No depth limit of the original's own is known: a chain of
   fragment weapons bursts at every level that the data names and that passes claim 1. A
   fragment weapon that names itself, directly or through a chain, is an edge case (Edge cases),
   where our engine sets a provisional bound.
   - Backing: `measurable` (scenario SH5 part a, for two levels). `unmeasured` for the absence of
     any limit: a finite run shows only the levels it sets up, and SH5 sets up two
   - Confidence: high for two levels; medium for the absence of a depth limit of the original's
     own; low for whether anything else bounds a chain that names itself (Edge cases)

8. **Laser lines.** When the fragment weapon has the laser flag, each fragment launched while the
   projectile still has its attacker makes a laser beam from the burst point to the fragment's
   target (the object of claim 3, or the cell of claim 4). The beam's look is the one S-0518
   claim 18 gives for a laser shot of the fragment weapon by the attacker: its duration and
   colours are the fragment weapon's, and with the fragment weapon's house-colour flag its inner
   colour is the attacker's house laser colour. Three things differ from claim 18's table for
   the attacker's own laser shot: the beam starts at the burst point, not at a fire point; its
   thickness is 1 whatever the attacker's type (claim 18 gives 2 to a vehicle's shot with the
   house-colour flag, and more to a prism building), which shows only in the house-colour
   drawing (S-0518 claim 22); and when the attacker is a building, terrain, cliffs, overlays and
   buildings drawn by the cell pass hide the beam exactly where they would hide a beam from a
   non-building source at the burst point (*a* = 0 in S-0518 claim 20): the beam is not drawn in
   front of more of the scene near its start, as the building-source adjustment draws a
   building's own laser shot. The fragment's bolt differs here: S-0316 claim 8 gives it the
   building-source adjustment, with the burst point in place of the fire point, so near the
   burst point a building attacker's fragment bolt can show where its fragment beam is hidden.
   The colours and the duration come from the fragment weapon even when a building attacker
   fires another slot, so claim 18's building-look bug does not arise; the duration follows claim
   18's long-duration bullet (the original's bug, and the fixed behaviour that claim states). A fragment launched after the attacker has gone makes no beam.

   **Electric bolts.** When the fragment weapon has the electric-bolt flag, each fragment launched
   while the projectile still has its attacker makes one bolt from the burst point to its target,
   with its spark system, as S-0316 claim 8 gives it; a fragment launched after the attacker has
   gone makes no bolt. The bolt's side strands take the colour that the fragment weapon's own
   alternate-colour flag selects (S-0316 claim 10); the flag of the weapon that fired the
   bursting projectile plays no part. The two flags are tested separately for each fragment, so when both hold,
   each such fragment makes a beam and a bolt. This differs from a shot, where the laser flag wins
   and the shot makes no bolt (S-0316 claim 2).
   - Backing: `measurable` (scenario SH1 part m with frame capture, for the beams and their
     look, and SH3 part b with frame capture, for no beam after the attacker has gone; SH1 part m
     run 2 and SH3 part d for the bolts; SH1 part m run 3 for both flags; run 4 for the bolt's
     colour; run 5, with buildings that house A builds during the run, for the building case; run 6
     for the thickness with a
     vehicle attacker, whose own shot would be thicker)
   - Confidence: medium

## Formulas

**F1. Fragment count.** With integer fragment count field *c*:

> *n* = *c* when *c* ≥ 0.
>
> *n* = |*c*| − ⌊*d* / 256⌋ when *c* < 0 and the attacker is known, with *d* = ⌊*r*(Δ*x*² + Δ*y*² + Δ*z*²)⌋.
>
> *n* = 3 when *c* < 0 and the attacker is gone.

Δ*x*, Δ*y*, Δ*z* are the integer lepton differences between the burst point and the attacker's
reference point, and *r* is the table square root of S-0204 (claim 5 and its F1): this is
S-0204's spatial distance (S-0204 claim 6) between two positions, with no building adjustment
(*β* = 0 whatever the attacker; claim 2 rates the height term, the end point and this absence).
The root is truncated toward zero to an integer, and the division by 256 truncates toward zero.
F1 inherits S-0204 F2's arithmetic domain; separations outside it are not specified here.
No fragments are launched when *n* ≤ 0. On flat ground at whole cells apart in one axis, and two
cells apart in both, the table root and the exact root give the same *d* (S-0204 claim 5), so
SH2 tests the count and its truncation but not the choice of root.

**F2. Fragment range** (claim 3). *k* = ⌊*R* / 256⌋, truncated toward zero, with *R* the fragment
weapon's range in leptons (S-0307 owns its import). *k* < 1 visits no ring, so every fragment goes
by claim 4. Backing: `measurable` (scenario SH1 part i). Confidence: high.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| First ring visited | 1 | ring | claim 3; ring 0 (the burst cell) is never visited |
| Highest ring visited | 11 | ring | departure for larger *k* (Edge cases) |
| Random-cell displacement range | −2 to 2, inclusive | cells per axis | claim 4 |
| Fragment count without attacker, negative field | 3 | fragments | claim 2 |
| Fragment elevation | about π/4 (45°) | radians | claim 6; `unmeasured` |
| Leptons per cell in F1 and F2 | 256 | leptons | |
| Fragment chain bound (our engine) | 16 | chain level | a projectile of level *d* bursts only when *d* < 16 (Terms, Edge cases); design, provisional until the lead decides |

## State and ordering

**Order inside one burst.** For one detonation that passes claim 1:

| Order | Event | Target-selection draws on the scenario random stream |
|---|---|---|
| 1 | the warhead's radiation effect, if any (S-0303 row 1) | as its spec says |
| 2 | fragments at candidates (claim 3), in T1 visiting order, each launched at once | none |
| 3 | fragments at random cells (claim 4), one after another | 2 per fragment, Δ*x* then Δ*y* |
| 4 | the projectile's area damage and what follows it (S-0303 rows 2 to 5) | as S-0303 says |
| 5 | for an airburst projectile, its nine children (S-0306 claim 21) | one R(0, 32) per child |

The table counts only the draws that choose the fragments' targets, and row 5's draws of the
airburst children, which S-0306 claim 21 gives. Whether a fragment's launch
also takes S-0306's launch scatter draws is open (Open questions).

A fragment's laser beam is made as the fragment is launched (claim 8). It is presentation and
takes no draw from the scenario random stream. A fragment's electric bolt is made at the same
moment; the bolt takes no draw from that stream, but its spark system's turns do, later (S-0316
Terms *bolt*, S-0616 claim 2).

**Draws after the area damage.** The draws that follow the area damage of a burst are S-0303's
and S-0306's: the one raw value of S-0303's order table row 3 for a projectile of the invisible
kind, the draws of rows 4 and 5 where they apply, and then, after each detonation that is not an
airburst, S-0306 claim 21's ranged draw R(256, 512) and one raw value for the next nearby point
(taken after the last detonation too). The two specs describe successive stages and do not
conflict: S-0303's table orders the draws inside one detonation, and S-0306 claim 21's two draws
follow it (Changes for other specs). Backing: `measurable` (scenario SH1 parts c and l, and SH5
part c for an airburst, with the random-state recorder). Confidence: high for their place after
the fragment draws; the draws themselves keep the ratings that S-0303 and S-0306 give them.

Each fragment then takes its turns as an ordinary projectile: the fragments join the end of the
update order in launch order during the bursting projectile's turn, and they are eligible for
turns later in the same turn pass (claim 5). So for invisible fragments whose projectile type has an arm
time of 0 (claim 5), the projectile's area damage comes first, and each fragment detonates in its first actual turn. With S-0203's removal
skip on, those turns are spread over several ticks: the bursting projectile's own removal costs
the object directly after it its turn, and each fragment's removal costs the next fragment its
turn, so with nothing else entering or leaving play the 2nd, 4th, 6th … fragments detonate in
the tick of the burst and the others in later ticks, in the pattern of claim 5 (for 3 fragments:
the 2nd in the tick of the burst, the 1st one tick later, the 3rd two ticks later). With the
switch off, all of them detonate in the tick of the burst, in launch order. The ticks of the
hits are `measurable` (SH1 part a, both runs; SH4 parts a and b). The order of the hits that
fall in one tick is `unmeasured`: SH1 part a's ordered log records it, but no part states an
expected order or is falsified by another. Confidence: medium that the hits are spread in this way rather than all falling in the
tick of the burst, which follows S-0203 claim 10, whose causes it rates medium, and S-0306's
first-turn detonation; medium for
the exact ticks until SH1 part a runs; medium for the order within a tick.

## Edge cases and quirks

- **First-listed object only** (`quirks.shrapnel_first_listed_only`, default on). Only the
  first-listed object of a visited cell can be a candidate, so an enemy behind an own unit in the
  same cell is passed over, and an own unit or allied unit listed first blocks the cell. Fixed
  behaviour: the first object in the cell's listing order that passes claim 3's test is the
  candidate.
- **Terrain objects take fragments** (`quirks.shrapnel_hits_terrain`, default on). A terrain
  object, such as a tree, has no owning house and so is a candidate. Fixed behaviour: objects
  without an owning house are not candidates.
- **One building, several fragments** (`quirks.shrapnel_building_per_cell`, default on). A
  building is a candidate once per visited cell where it is first-listed (claim 3). Fixed behaviour:
  each object receives at most one fragment from one burst's scan; a cell whose candidate has
  already received one from that scan gives none, and the scan moves on. With this switch off,
  the rule covers the whole scan, the second visit of ring 11's twice-visited cell included, so
  that cell's candidate receives one fragment, not two; with `quirks.spread_ring_eleven` off the
  cell is not visited at all. When `quirks.shrapnel_first_listed_only` is off as well, an object
  that has already received a fragment fails claim 3's test, so a later-listed object of the same
  cell can be the candidate. The fragments of claim 4 aim at cells, not objects, so this switch
  does not touch them: an object standing in a drawn cell is hit by them whatever it has already
  received.
- **Ring 11 visits one cell twice** (S-0303's `quirks.spread_ring_eleven`, default on, the same
  switch). The fragment scan uses table T1's ring-11 list, so with a fragment range of 11 cells
  or more it visits one cell twice, misses eight cells of metric 11 and visits seven of metric
  12; the twice-visited cell's first-listed candidate receives two fragments (claim 3). Off: ring
  11 is S-0303's fixed set of metric-11 cells, each visited once.
- **Building at the burst.** No fragments burst when the burst cell's first-listed object is a
  building (claim 1 b). A shot at a building bursts in a cell of its foundation, where the
  building is listed last (S-0209 claim 13): when nothing else is listed there, the shot launches no
  fragments, even with enemy vehicles beside the building; when another object, such as an
  infantry unit, is listed in that cell, that object is first-listed and the shot bursts by the
  ordinary rules. This is the rule, not a defect; no switch.
- **Fragments at own units.** Random-cell fragments (claim 4) can land on the firer's own and
  allied objects. Whether they hurt them is S-0302's and S-0303's ordinary rule for the warhead.
- **Fragment range above 11 cells.** S-0303 table T1 ends at ring 11, and for *k* > 11 this spec does
  not characterise the original's choice of cells. This is a departure, as for area
  damage (S-0303 Constants, largest scan radius): our engine visits rings 1 to min(*k*, 11) only,
  with ring 11 as S-0303 table T1 lists it.
- **A fragment weapon that names itself.** When a fragment weapon's projectile names the same
  fragment weapon, directly or through a chain, every fragment that passes claim 1 bursts again.
  With invisible fragments each level bursts in the ticks that claim 5 gives, and the
  chain ends only when no detonation of it passes claim 1, for example when the objects it hits
  are destroyed and leave burst cells with no first-listed object. No limit on the number
  of levels is known; whether anything bounds such a chain, for example
  when no further projectile can be made, is not established. Confidence: medium that the
  original has no depth limit of its own (claim 7); low for whether anything else bounds such a
  chain, `unmeasured`: a run that tests a chain whose warhead destroys nothing may stop the
  original from responding.
  Our engine's rule, provisional until the lead decides (design): a projectile of chain level
  *d* (Terms) launches fragments only when *d* < 16, so a fragment of level 16 detonates by
  S-0306 and S-0303 without bursting, and a chain has at most 16 levels of fragments. A chain whose
  data names at most 16 fragment weapons one after another, without repeating one, therefore
  bursts as in the original unless it has a limit of its own below 16 levels. Backing:
  `unmeasured` (our engine's design).
  Whether to keep, change or drop this bound is the lead's call (Questions for the project
  lead).
- **High bridges.** Both the test of claim 1 (b) and the visits of claim 3 look only at a cell's
  ground layer (S-0209 Terms *cell layer*). An object on a high-bridge deck is never a candidate,
  and a projectile that bursts at a deck object launches fragments only when the ground layer
  below lists a first object that is not a building. Backing: `measurable` (scenario SH8 part
  f). Confidence: medium, until SH8 part f runs.
- **Firer killed in flight.** A projectile whose attacker has gone (S-0302 claim 1) still bursts,
  but every fragment goes to a random cell (claim 4), and with a negative fragment count exactly 3
  do.

## Import mapping

| Source | Key | Concept | Form | Absent |
|---|---|---|---|---|
| rules, projectile section | `ShrapnelWeapon` | fragment weapon | weapon reference, a creating field (S-0004 claim 13; claims 4 and 6 for its index and reading) | none: no fragments |
| rules, projectile section | `ShrapnelCount` | fragment count *c* | integer | 0 |
| rules, fragment weapon's section | `Damage`, `Warhead`, `Speed`, `Projectile`, `Range` | the fragment's damage and warhead; its speed through the weapon speed of S-0306 claim 1 (the speed field counts only for a projectile with a non-zero turn rate, claim 5); the fragment's projectile type; *R* of F2 | `Damage` as S-0302 imports a weapon's damage field (*D* of its F1); `Warhead`, `Speed` and `Projectile` as S-0306 imports them; `Range` as S-0307 imports it | as those specs say |
| rules, fragment weapon's section | `Bright` | the fragment's brightness flag (claim 5) | as S-0312 imports it | as S-0312 says |
| rules, fragment weapon's section | `IsLaser` and the laser look fields | the laser beam of claim 8 | as S-0518 imports them | as S-0518 says |
| rules, fragment weapon's section | `IsElectricBolt`, `IsAlternateColor` | the bolt of claim 8 and its side-strand colour | as S-0316 imports them | as S-0316 says |

S-0004 claim 6 applies: a weapon first created in a layer's projectile reading pass, as a
projectile's fragment weapon, does not read its section in that layer.

## Measurement scenarios

All under the glossary's measurement setup, with named test files (loose copies of the rules file
changing only the fields named) supplied before each run and removed after it. A read-only recorder
reads every object's health and cell at the start of every tick, and the random-state recorder
reads the scenario random stream where stated. The *test weapon* is a copy of an existing direct-fire
weapon, changed so that its warhead damages a single object (no area spread), it has neither the
laser flag nor the electric-bolt flag, it names no shot animation (except in SH1 part a run 2),
and its projectile is a test projectile type that is invisible and names a *test fragment weapon*. The test weapon's warhead and the test fragment
weapon's warhead are test warheads that name no explosion animation and throw no debris, so that
rows 4 and 5 of S-0303's order table take no draw among the draws that the parts count (SH1 parts
c, l, o, p and s, SH5 part c). The test fragment weapon has an invisible test
projectile that is neither inaccurate nor flak-scattering, a single-target warhead, a damage field that is visibly smaller than the test weapon's,
no laser flag (except in SH1 part m runs 1, 3, 5 and 6 and SH3 part b), no radiation level
(except in SH1 part n run 4), no electric-bolt flag (except
in SH1 part m runs 2, 3 and 4 and SH3 part d), and a range of 3 cells. A test vehicle
type carries the test weapon as its primary weapon and the test fragment weapon as its secondary
weapon (so that the fragment weapon is read, S-0004 claim 6); it is never ordered to use the
secondary. Every target object is of a test type whose health is above all the damage it can take
in its scenario, so that nothing dies, except where a part says otherwise (SH4 part b). Every
test projectile type has no arm field, so its arm time is 0 (S-0306 claim 9).
Everything is on flat open ground, away from cliffs, bridges, terrain objects and the map edge,
with nothing within 12 cells of *T* other than what the scenario or part places. A laser duration
given to a test weapon below is from 1 to 127 ticks, and its house-colour flag is off except in
SH1 part m runs 5 and 6. The test fragment weapon's alternate-colour flag is off except in SH1 part m
run 4, and the test weapon's is off throughout.

*Buildings of house A.* House A is human-controlled, so the test map cannot give it a building
(S-0009 claim 27). Where a part names a house-A building, house A builds it during the run, and
that part's game has the bases setting on and a unit-count setting of 0 (S-0412 Terms), so house A starts with
its base unit, a mobile construction vehicle, and no other unit (S-0009 claim 45); house B's base
unit falls under the void rule of the glossary's *bystander computer player*. The part's named
test file makes each named building type buildable by house A from the start, with no
prerequisite, no power drain and a cost within house A's starting credits. The player deploys the
construction vehicle and builds and places each named building on its named cell; any further
building needed to reach that cell stands more than 3 cells from *T*'s cell. A run is void if a
named building cannot be placed on its named cell. A test defence building is ordered to attack
*T* once it is complete; its first shot is the one judged, and if that shot is at another object,
the run is void.

**SH1. Count, candidates and order (claims 1, 3, 4 and 8, F2).**
- Setup: fragment count 3. One test vehicle (house A) and, 6 cells away, an enemy vehicle *T*
  (house B). Around *T*'s cell, enemy vehicles displaced by (0, −1), (1, 0), (0, 2), (3, 0) and
  (2, 2). In T1 order these are ring 1's (0, −1) then (1, 0), ring 2's (0, 2), and ring 3's (3, 0)
  then (2, 2) (S-0303 ring metric). The test vehicle stands on the side away from them. *T* and
  the surrounding enemies are unarmed test copies of a vehicle type, given no orders, so that
  they neither fire nor move; if the recorder shows any of them off its cell at the shot, the run
  is void.
- Procedure: order the test vehicle to attack *T* and let it fire one shot; record health at the
  start of the tick of the shot and at the start of each of the 5 ticks after it, which covers the
  tick of the shot and the next 4 ticks (enough for the latest fragment hit of claim 5 with the
  largest fragment count that any part uses). Each tick's reading is taken at the start of the
  following tick, after every object's turn in that tick, so a loss in tick 3 after the shot shows
  in the reading taken at the start of tick 4; "a loss in tick *k*" in the parts below means the
  change between the readings taken at the starts of ticks *k* and *k* + 1.
- Part a. *Expected:* *T* loses the test weapon's damage after its warhead, and exactly the first 3
  of the surrounding vehicles in T1 order lose the fragment's damage after its warhead; the
  others are unhurt. The losses fall in these ticks (claim 5): *T*'s and the vehicle at (1, 0)'s
  in the tick of the shot, the vehicle at (0, −1)'s one tick later and the vehicle at (0, 2)'s
  two ticks later. *Falsified by:* any other set of damaged vehicles, or any loss in another
  tick; in particular, all four losses in the tick of the shot, which the removal skip of
  S-0203 claim 10 switched off would give. Run 2: the named test file also gives the test weapon
  a shot animation, an existing sprite animation type, so that the animation enters play directly
  after the projectile (S-0109 claim 29). *Expected:* the same set of losses, with *T*'s and the
  vehicles at (0, −1) and (0, 2) in the tick of the shot and the vehicle at (1, 0)'s one tick
  later. *Falsified by:* the ticks of the first run, or any other ticks. In this part the
  recorder also logs, in the order they happen, every object entering or leaving play and every
  health change. If the recorder shows any
  object other than the projectile, the fragments and (in run 2) the shot animation entering or
  leaving play from the shot to the last expected loss, the run is void; run 2 is also void if
  the shot animation leaves play before the vehicle at (1, 0) loses health, since its removal
  would cost the waiting fragment its turn (S-0203 claim 10).
- Part b. As a, but *T* is replaced by an enemy building of one-cell foundation, of a type whose art
  gives it no aim shift (S-0306 Terms, *target point*). *Expected:* only the building is damaged. *Falsified by:* any surrounding vehicle losing health.
- Part c. As a with fragment count 8 (more than the 5 candidates), with the random-state
  recorder. *Expected:* each of the 5 surrounding vehicles takes one fragment; 6 ranged draws
  R(−2, 2) at the burst give 3 cells (*x* + Δ*x*, *y* + Δ*y*), each taken in Δ*x*, Δ*y* pairs; 3
  more invisible detonations at those cells, so a vehicle (*T* or a surrounding one) standing in a
  drawn cell takes one more fragment's damage per time its cell is drawn. The 6 draws come
  first at the burst, before the draws that follow the area damage (State and ordering: S-0303's
  row 3 raw value for the invisible test projectile, then S-0306 claim 21's R(256, 512) and one
  raw value), so the drawn cells are those that the burst's first 6 ranged draws give.
  *Falsified by:* another number of draws, another range, fragment damage that the drawn cells
  do not predict, or drawn cells that match only the values after the first raw value that
  follows the area damage.
- Part d. As a, with an own (house A) vehicle at (0, −1). *Expected:* that vehicle is unhurt,
  and the fragments go to (1, 0), (0, 2) and (3, 0). *Falsified by:* the own vehicle being hit, or
  any other set of three enemies being hit.
- Part e. As a, with (0, −1) holding an enemy infantry unit and an own infantry unit, both placed
  there by the test map, the own one placed after the enemy one so that it is listed first (S-0209
  claim 13). *Expected:* nothing at (0, −1) is hit, and the fragments go to (1, 0), (0, 2) and
  (3, 0). *Falsified by:* the enemy infantry unit being hit. If the recorder shows the two not
  sharing the cell at the shot, the part is void. The part rests on S-0209 claim 13's listing
  order for map-placed objects, which that spec rates medium: a run in which the enemy unit is hit
  is reported against S-0209 claim 13 as well, and falsifies the first-listed rule of claim 3
  only once that order is confirmed.
- Part f. As a, with a tree (a terrain object) at (0, −1) instead of the vehicle there.
  *Expected:* the tree receives the first fragment (and takes damage if its terrain type is not
  immune), and the fragments go to the tree, (1, 0) and (0, 2), so (3, 0) is unhurt. *Falsified
  by:* (3, 0) losing the fragment's damage, which a skipped tree would give it as the third
  fragment.
- Part g. As a, with the vehicle at (0, −1) owned by a third player house C. The part's named test
  file sets both rules flags of S-0414 claim 14 on, so that the alliance order can be given; like
  every named test file it is supplied before the run and removed after it, which restores the
  rules in use. After the game has started
  and before the shot, house A's player gives the alliance order towards C, so that A allies C
  and C does not ally A (S-0414 claim 12). The recorder reads both houses' alliance state; if it
  shows C allying A at the shot, the part is void. *Expected:* the C vehicle is unhurt, and the
  fragments go to (1, 0), (0, 2) and (3, 0). *Falsified by:* the C vehicle being hit.
- Part h. As a, with the vehicle at (0, −1) placed by the test map for the special house, so that
  the special house allies A and A does not ally the special house, as S-0412 claim 8 makes it
  when the special house's country is of the civilian side. The recorder reads both houses'
  alliance state; if it shows A allying the special house, or the special house not allying A,
  the part is void. *Expected:* the special-house vehicle receives the first fragment, then
  (1, 0) and (0, 2). *Falsified by:* the special-house vehicle unhurt.
- Part i. As a, with the test fragment weapon's range set to 2.75 cells (704 leptons, so *k* = 2)
  and fragment count 4, with the random-state recorder. *Expected:* (0, −1), (1, 0) and (0, 2)
  each take one fragment, (3, 0) takes none, and 2 ranged draws R(−2, 2) at the burst give one
  cell, where the fourth fragment detonates. It hits a vehicle only when the drawn cell is that
  vehicle's: a first hit for *T* when the cell (0, 0) is drawn and for the vehicle at (2, 2) when
  (2, 2) is drawn, and a second hit for the vehicle at (0, −1), (1, 0) or (0, 2) when its cell is
  drawn. *Falsified by:* (3, 0) hit, no draws, a vehicle hit more often than the drawn cell gives,
  or (2, 2) hit when its cell is not drawn (which ring 3 being visited would give).
- Part j. As a, but with the test fragment weapon's range set to 11 cells, fragment count 2, and
  the surrounding vehicles replaced by a single enemy vehicle *V* at (−3, 11) from *T*'s cell (the
  cell that S-0303 table T1 lists twice), with the random-state recorder. *Expected:* *V* takes the
  fragment's damage twice, and no fragment draws are taken. *Falsified by:* *V* hit fewer than
  twice, or any draw of claim 4. With `quirks.spread_ring_eleven` off our engine does not visit
  (−3, 11), a cell of metric 12, so there *V* is unhurt and 4 draws are taken.
- Part k. As a, with fragment count 4, the surrounding vehicles removed, and an enemy test
  building of 2 × 2 foundation covering (1, 0), (2, 0), (1, 1) and (2, 1) from *T*'s cell, with the
  random-state recorder. *Expected:* the building takes the fragment's damage four times, and no
  fragment draws are taken. *Falsified by:* fewer than four fragment hits on the building, or any
  draw of claim 4.
- Part l. As a, without *T*: the test vehicle is given a forced attack on *T*'s empty cell, with the
  random-state recorder. *Expected:* no vehicle loses health, and at the burst the recorder sees
  only the one raw value that the invisible test projectile's own detonation takes (S-0303 order
  table, row 3), then S-0306 claim 21's ranged draw R(256, 512) and one raw value, and no draw
  of claim 4 before them. *Falsified by:* any surrounding vehicle losing health, or a pair of
  R(−2, 2) draws before that raw value. If the test vehicle does not fire, the part is void.
- Part m. As a, with the test fragment weapon given the laser flag and a laser duration, and
  frame capture. *Expected:* in the tick of the shot, three beams appear, each from
  *T*'s position to one of the three hit vehicles, in the test fragment weapon's laser colours
  (S-0518 claim 18), and none from the test vehicle. *Falsified by:* no beams, beams from the test
  vehicle, or beams in other colours. Run 2: the test fragment weapon has the electric-bolt flag
  instead of the laser flag. *Expected:* three bolts, each from *T*'s position to one of the three
  hit vehicles, each with sparks (S-0316 claim 8). *Falsified by:* no bolts, another number, or
  bolts from the test vehicle. Run 3: the test fragment weapon has both flags. *Expected:* three
  beams and three bolts, one of each from *T*'s position to each of the three hit vehicles.
  *Falsified by:* beams without bolts (which the shot ranking of S-0316 claim 2 would give), or
  bolts without beams. Run 4: as run 2, with the test fragment weapon's alternate-colour flag on.
  *Expected:* each bolt's two side strands in the alternate colour entry (S-0316 claim 10).
  *Falsified by:* side strands in the ordinary entry, which the test weapon's flag (off) would
  give. Run 5: as run 1, with the test fragment weapon's house-colour flag on, and the test
  vehicle replaced by a house-A test defence building of one-cell foundation carrying the test
  weapon, which house A builds 6 cells due south of *T* (*Buildings of house A*). A house-A test
  building at least 3 levels tall, which house A also builds, stands with its foundation just
  south-west of *T*'s cell, so that the first part of the beam
  to (0, −1)'s neighbour cells passes in front of it on screen. As an object of the firer's own
  house it is never a candidate (claim 3), so the three fragments still go to the vehicles at
  (0, −1), (1, 0) and (0, 2), and as no fragment goes to a random cell it cannot be hit. *Expected:* three beams, each drawn
  with thickness 1 in the house-colour drawing (S-0518 claim 22: a centre line and at most one
  pair of side lines), and their hidden parts near the burst
  point are those that a source depth bias with *a* = 0 gives (S-0518 claim 20, F9). *Falsified
  by:* thicker beams, or hidden parts that match the building-source adjustment instead. The two
  predictions are those of S-0518 claim 20 (F9) for each choice of *a*, worked out for the run's
  positions before it is judged. Run 6 (a vehicle attacker): as run 1, with the test fragment
  weapon's house-colour flag on, in S-0518's fine mode, and house A of a colour whose house laser
  colour, read from the rules files in use, has a channel of at least 32, so that a beam of
  thickness 2 would draw a second pair of side lines (S-0518 claim 22's stop threshold).
  *Expected:* three beams, each at most 3 pixels across where it runs horizontally or vertically
  on screen, and at most one pair of side lines wherever it runs. *Falsified by:* a second pair
  of side lines, which the thickness 2 of a vehicle's own house-colour shot (S-0518 claim 18)
  would give.
- Part n. As a, with the random-state recorder, in four runs: (1) the test weapon's warhead also
  has the electromagnetic-pulse flag, (2) the test weapon has a positive radiation level
  (S-0525) instead, (3) the test weapon's warhead has the bomb-planting flag instead, and *T*
  carries no bomb, so the shot is not illegal (S-0519 claim 3), (4) the test fragment weapon has
  a positive radiation level instead, and health is recorded for twice S-0525's application
  delay in the rules in use after the shot. In run 4 the surrounding vehicles are of a type that
  is not immune to radiation (S-0525 claim 8). *Expected:* in runs 1 and 2 the
  fragments go as in part a, in the ticks of part a. In run 2 a radiation loss is told from a
  fragment's by its amount and by S-0525's timing. In run 3 planting is selected, does no area
  damage and, since the firer is not an infantry unit, plants no bomb (S-0519 claim 2): no
  vehicle loses health, and no draw R(−2, 2) is taken. In run 4 the three vehicles hit by
  fragments lose the fragment's damage once, in the tick that part a gives each, and no health after it
  (claim 5: a fragment leaves no radiation). *Falsified by:* no fragment damage in run 1
  or run 2, any fragment damage in run 3, or in run 4 a later loss of health of a vehicle hit by
  a fragment on a tick of S-0525's application cadence (claim 9 there). If the test vehicle does not fire in run 3, that run
  is void.
- Part o. As c, with fragment count 0. *Expected:* no surrounding vehicle loses health, and at the
  burst the recorder sees no draw R(−2, 2) before the raw value of the test projectile's own
  detonation (S-0303 order table, row 3). *Falsified by:* any fragment damage, or such draws.
- Part p. As c, but the test weapon's warhead has the wall flag and a spread that reaches ring 1
  (S-0303), and a wall overlay stands at (0, 1) from *T*'s cell, of a wall type whose strength is
  above the test weapon's damage, so that its wall hit takes one draw R(0, *T*<sub>w</sub>), with
  *T*<sub>w</sub> that strength (S-0303 claim 16, F4). Every vehicle's health is above the area
  damage it can take. *Expected:* the 3 random-cell fragments land at the cells that the burst's
  first 6 draws give, taken as R(−2, 2) in Δ*x*, Δ*y* pairs, and the wall's outcome (damaged or
  not) agrees with the 7th draw as the wall hit's draw R(0, *T*<sub>w</sub>). *Falsified by:*
  landing cells that match draws 2 to 7 instead, with the wall's outcome agreeing with the 1st
  draw, which a wall-hit draw taken before the fragment draws would give.
- Part q. As a, with the vehicle at (0, −1) of a test type whose armour class differs from the
  other surrounding vehicles', and named test rules giving the test fragment weapon's warhead a
  versus value of exactly 0 against that class (S-0304). *Expected:* that vehicle is unhurt, and
  the fragments go to it, (1, 0) and (0, 2), so (3, 0) is unhurt. *Falsified by:* (3, 0) losing
  the fragment's damage, which a test of the fragment weapon's legality would give it as the third
  fragment.
- Part r (burst point against detonation point, claim 1). As b, in two runs, with a visible test
  projectile: a slow straight projectile, neither invisible nor inaccurate nor airburst, still
  naming the test fragment weapon; the test weapon's warhead lacks the electromagnetic flag (S-0303
  F5). The test vehicle fires from due west of *T*. In run 1 the building at *T*'s cell has an aim
  shift of 256 leptons along the *x* axis, towards (1, 0), set by named test art (a loose copy of
  the art file changing only that field, supplied and removed like the test files; S-0306 Import
  mapping), so that its target point is the centre of the cell (1, 0), where an enemy vehicle
  stands. The shot reaches the building's reference point and is placed on it (S-0306 claim 20 b).
  As the building's spatial distance there is 0, (c) then moves the detonation point to the target
  point in (1, 0). Run 2 (control): as a, with the same visible projectile at the vehicle *T*.
  *Expected:* in run 1 no object loses the fragment's damage, though the vehicle at (1, 0) may lose
  the test weapon's damage from the shot's own area damage there: the burst cell is the
  building's cell, where the building is first-listed. In run 2 the fragments go as in part a.
  *Falsified by:* in run 1, any object losing the fragment's damage, which a burst from the
  detonation point in (1, 0), whose first-listed object is an enemy vehicle, would give. If no
  fragment damage appears in run 2, the part is void.
- Part s (detonations whose area damage does nothing, claim 1). As c (fragment count 8, with the
  random-state recorder), in two runs: (1) the test weapon's damage field is 0, while the test
  fragment weapon keeps its positive damage field (the setup's comparison of the two damage fields
  does not apply to this run); (2) the test map has its inert flag set (S-0303 claim 2).
  *Expected:* in run 1 *T* loses no health other than one fragment's damage for each time its cell
  is drawn, and the 5 surrounding vehicles and the vehicles in the drawn cells take the fragment
  damage as in part c.
  In run 2 no object loses health, and the 6 ranged draws R(−2, 2) are still taken at the burst.
  *Falsified by:* in run 1, no fragment damage, or a loss of health of *T* that the drawn cells
  do not give; in run 2, no draws R(−2, 2) at the burst. If the
  test vehicle does not fire, that run is void.

**SH2. Negative count (claim 2, F1).**
- Setup: as SH1 part a, but the fragment count field is −6 and the test weapon's range is at
  least 7 cells. In parts a to c the test vehicle stands due west of *T*, at a whole number of
  cells from it.
- Procedure: one shot from 2 cells (part a), one from 5 cells (part b) and one from 7 cells
  (part c), and one with the test vehicle at (−2, −2) cells from *T*'s cell (part d, about 2.83
  cells), each in its own run.
- *Expected:* a: 4 fragments (|−6| − 2), and the test vehicle itself, standing at (−2, 0) inside
  the fragment range, is unhurt, so the four go to (0, −1), (1, 0), (0, 2) and (3, 0); b: 1
  fragment; c: no fragment and no fragment draws; d: 4 fragments (the distance of about 724 leptons
  truncates to 2 cells). *Falsified by:* other counts, in particular 3 fragments in part d (a
  distance rounded to the nearest cell), or in part a the test vehicle hit or (3, 0) unhurt. The
  distance is spatial, so on flat ground it is the planar distance between the burst point and the
  firer's reference point.
- Part e: the test vehicle is replaced by a house-A test defence building of one-cell foundation
  carrying the test weapon, which house A builds 2 cells due west of *T* (*Buildings of house A*).
  *Expected:* 4 fragments: the distance from the
  burst point to the centre of the building's cell is 512 leptons and takes no building
  adjustment. *Falsified by:* 5 fragments, which S-0204's building adjustment of 128 leptons for a
  one-cell foundation would give (⌊384 / 256⌋ = 1), or any other count.

**SH3. Attacker gone (claims 2, 3, 4 and 8).**
- Setup: as SH1 part a, except for where the surrounding enemy vehicles stand: only at (3, 0),
  (0, −3) and (−3, 0) from *T*'s cell, inside the fragment range of 3 cells but outside the random
  displacement of claim 4 (at most 2 cells per axis), and the test vehicle stands at (0, 6). The test
  projectile is a slow straight, not invisible, projectile, and a second enemy unit with an
  instant weapon, standing at least 4 cells from *T*'s cell, is ordered to destroy the test vehicle
  while the shot flies. Fragment count 3 (part a) and −6 (part b, with the fragment weapon given the
  laser flag and a laser duration, and frame capture). Part c is a control: as part a,
  but the test vehicle is not destroyed. Part d: as part a, with the fragment weapon given the
  electric-bolt flag, and frame capture.
- *Expected:* in parts a and b, the random-state recorder sees 6 ranged draws R(−2, 2) at the
  burst (3 fragments each), and the 3 fragments detonate at the cells (*x* + Δ*x*, *y* + Δ*y*)
  that those draws give, taken in Δ*x*, Δ*y* pairs; an object standing in a drawn cell (*T*
  itself when a pair is (0, 0)) takes the fragment's damage after its warhead, and none of the
  three surrounding vehicles is damaged by a fragment; in part b, the frame capture shows no beam
  at the burst, and in part d no bolt at the burst. In part c, the three surrounding vehicles each
  take one fragment and no fragment draws are taken. *Falsified by:* fragment damage that the
  drawn cells do not predict, a surrounding vehicle damaged in part a, b or d, another number of
  draws, a beam at the burst in part b, or a bolt at the burst in part d.

**SH4. Unscaled damage, timing and attacker (claim 5).**
- Part a. Setup: as SH1 part a, with the test vehicle at the elite veterancy rank and holding a
  firepower bonus from a firepower crate it collected before the shot. *Expected:* *T*'s loss
  scaled by S-0302 claim 2's factors; each fragment's target loses exactly the unscaled fragment
  damage after its warhead; each loss in the tick that SH1 part a gives it. SH1 part a's void rule
  applies: the run is void if the recorder shows any object other than the projectile and the
  fragments entering or leaving play from the shot to the last expected loss. *Falsified by:* a
  scaled fragment loss, or a loss in another tick.
- Part b. Setup: as SH1 part a, with the vehicle at (0, −1) given health no greater than the
  fragment's damage after its warhead, so that its fragment destroys it, and *T* given health
  above the test weapon's damage. The read-only recorder reads each house's count of enemy units
  it has destroyed (the count the end-of-game score shows). *Expected:* house A's count rises by
  one in the tick after the shot, when the first fragment detonates (claim 5), and no other
  house's count changes. A repeat as SH3 part a, with the
  doomed vehicle standing in a cell that a recorded draw pair gives, credits no house. *Falsified
  by:* no rise for house A in the first run, or a rise for house A in the repeat.
- Part c. Setup: as SH1 part a, with named test rules giving the test fragment weapon its own test
  warhead, single-target, whose percentage against the surrounding vehicles' armour is 50, while
  the test weapon's warhead has 100 against it. *Expected:* each fragment's target loses half the
  fragment damage (S-0302's rounding). *Falsified by:* a loss of the whole fragment damage, which
  the parent's warhead would give.

**SH5. Clusters, airbursts and fragments of fragments (claims 1 and 7).**
- Part a: as SH1 part a, but the test fragment weapon's projectile names a second test fragment
  weapon, with a fragment count of 1 on that projectile. The second test fragment weapon has
  range 2 cells, a damage field different from the first fragment weapon's, and an invisible test
  projectile that names no fragment weapon. It is also carried by a second test vehicle type, so
  that it is read. *Expected:*
  each first-level fragment bursts again at its target's cell, by claim 3 seen from that cell:
  the burst at (0, −1) sends its fragment to *T*, the burst at (1, 0) to the vehicle at (0, −1),
  and the burst at (0, 2) to *T*. So *T* takes the second fragment weapon's damage twice, the
  vehicle at (0, −1) once, and nothing else takes it. *Falsified by:* any other set of
  second-level damage, including none.
- Part b: as SH1 part a, but the test projectile has a cluster count of 2 (S-0306 claim 21).
  *Expected:* 6 fragment hits in total (3 per detonation, the same 3 candidates twice, each from the
  projectile's own position). *Falsified by:* 3 hits, or hits from the cluster's second point.
- Part c: as SH1 part c (fragment count 8), with the random-state recorder, but the test
  projectile is airburst (S-0306 claim 21) and names a test airburst weapon, whose damage field
  differs from both test weapons', with a single-target warhead and a projectile that names no
  fragment weapon. The test airburst weapon is also carried by a second test vehicle type, so
  that it is read. *Expected:* the 5 surrounding vehicles each take one fragment, in the tick of
  the shot or a later recorded tick (claim 5); at the burst the 6 ranged draws R(−2, 2) come first, then the draws that follow the
  area damage within the detonation (S-0303 order table), then the nine draws R(0, 32) of the
  airburst children, and no draw R(256, 512). *Falsified by:* no fragment damage, or the draws
  R(0, 32) before the fragment draws. The airburst and invisible flags are independent (S-0306
  Terms); if neither the nine draws R(0, 32) nor the airburst weapon's damage appears, the part is
  void.

**SH6. The Q-1714 pairing (claims 1 to 5).** In the measurement harness's stock two-group fight of
Q-1714, watched for the first 600 frames, record for each tick the number of shots of each side
that detonate and the number of objects of each side whose health drops. Counting detonations
needs a reading that the general setup's recorder does not take: for this scenario the read-only
recorder also logs each detonation of a projectile as it happens, with its tick and the side
whose object fired it (an invisible projectile detonates between two start-of-tick readings, so
a reading taken only at the start of a tick would miss it). A shot is a projectile fired by an
object; the detonation of a projectile launched by a burst (claim 1) is a fragment's, and the
recorder logs it apart, with its tick and the side of its attacker (claim 5). The recorder tells
the two apart by projectile type, so the run is void if, in the
files in use, a fragment weapon's projectile type is also the projectile of a weapon that either
group fires. Before the run, record
from the rules files in use which group's weapons (ordinary and elite) have projectiles that
name a fragment weapon; that group is the *fragment side*. The run does not assume that only
one group has one: if both do, the comparison below is made for each, and if neither does, the
run is void. Also record from those files the fragment side's fragment count field *c*, so that
*n*ₘₐₓ = *c* for *c* ≥ 0 and *n*ₘₐₓ = the larger of |*c*| and 3 for *c* < 0 bounds claim 2's count, whether its
warhead and fragment warhead have an area spread, and whether its fragment projectiles name a
fragment weapon of their own and are invisible (claim 5), and whether the fragment side's
projectiles and its fragment projectiles detonate once and are not airburst projectiles (S-0306 claim 21). Two shots that detonate in the same tick cannot be told apart, so
the run counts objects per tick, not per shot. Only *counted ticks* are judged: ticks before the
first object of either side is destroyed (so no death explosion adds damage), in which no shot
and no fragment of the other side detonates. A fragment need not detonate in the tick of its
burst (claim 5), so a counted tick can hold fragment hits of a shot that detonated earlier. *Expected:* counted ticks in which more objects of the other side lose
health than shots of the fragment side detonate, when the targets stand within the fragment range
of each other. Where the recorded fields give no area spread to either warhead and no fragment
weapon to the fragment projectiles, the fragment projectiles are invisible, and the fragment side's
projectiles and its fragment projectiles detonate once and are not airburst projectiles, two
upper bounds are judged: every counted tick has at most *s* + *f* objects of the other side
losing health, where *s* and *f* are the fragment side's shots and fragments logged as
detonating in that tick; and over the whole run the fragment side's logged fragment
detonations number at most *n*ₘₐₓ times its logged shot detonations. A run with no counted tick in which more objects of the other side lose health than shots of
the fragment side detonate is inconclusive, since claim 1 may have stopped every burst. *Falsified
by:* where the upper bounds are judged, a counted tick with more than *s* + *f* objects of the
other side losing health, or more than *n*ₘₐₓ fragment detonations per shot detonation of the
fragment side over the run.

**SH7. Launch direction (claim 6).** As SH1 part a, but the test fragment weapon's projectile is a
slow straight, not invisible, projectile, and the recorder reads every projectile's position in
leptons each tick. *Expected:* each fragment's first recorded move goes from its starting
position (claim 5: the burst point, raised in height only when it is not above the terrain)
horizontally towards its target's reference point, and its climb is recorded. *Falsified by:* a first move that heads elsewhere horizontally.

**SH8. What the scan ignores (claim 3, Edge cases).** Each part is as SH1 part a,
with the random-state recorder, except as it says.
- Part a (cloaking). The vehicle at (0, −1) is of a house-B test type that cloaks by itself
  (S-0516 claims 2 and 5), idle since the start of the game, with no sensor of house A anywhere on
  the map. The recorder reads its cloak state; unless it is fully cloaked at the shot, the part is
  void. *Expected:* it receives the first fragment (and starts uncloaking, S-0516 claim 12), the
  others go to (1, 0) and (0, 2), (3, 0) is unhurt, and no draw R(−2, 2) is taken. *Falsified by:*
  (3, 0) hit with the cloaked vehicle unhurt.
- Part b (shroud). Fragment count 4, the shroud on, and the test vehicle at (−6, 0) from *T*'s cell,
  of a test type whose sight range is 7 cells, with nothing else of house A on the map, so that
  house A has never seen the cell (3, 0). The recorder reads house A's view of that cell; if house A
  has seen it before the shot, the part is void. *Expected:* (3, 0) receives the fourth fragment,
  and no draw R(−2, 2) is taken. *Falsified by:* (3, 0) unhurt with a pair of draws R(−2, 2) taken
  at the burst.
- Part c (disguise). The vehicle at (0, −1) is replaced by a house-B infantry unit of a test type
  that can disguise, given house A as its disguise house through S-0516 claim 31 before the shot.
  The recorder reads its disguise house and its cell; unless they are house A and (0, −1) at the
  shot, the part is void. *Expected:* it receives the first fragment, and (3, 0) is unhurt.
  *Falsified by:* (3, 0) hit with the disguised unit unhurt.
- Part d (anti-air and anti-ground flags). Two runs: the test fragment weapon's projectile has the
  anti-ground flag off and the anti-air flag on (run 1), and both flags off (run 2). *Expected:* in
  both runs no draw R(−2, 2) is taken at the burst, because the three fragments go to (0, −1),
  (1, 0) and (0, 2). *Falsified by:* six draws R(−2, 2) at the burst, which leaving out targets
  that the flags make illegal would give. What each fragment then does to its vehicle is S-0306's
  and S-0303's and is not judged here.
- Part e (aircraft). The vehicle at (0, −1) is removed, and an unarmed house-B aircraft of a test
  type that stays in the air over a cell while idle (S-0501) is ordered to (0, −1) and is there,
  in the air, at the shot. The recorder reads its cell and its height; unless it is over (0, −1)
  and above the ground at the shot, the part is void. *Expected:* the aircraft is unhurt, the
  fragments go to (1, 0), (0, 2) and (3, 0), and no draw R(−2, 2) is taken. *Falsified by:* the
  aircraft hit, or (3, 0) unhurt.
- Part f (high bridges). On a test map with a high bridge over water (the general setup's
  distance from bridges does not apply). Run 1: *T* stands on the bridge deck over a water cell
  whose ground layer lists nothing, and the surrounding enemy vehicles stand on land within 3
  cells of *T*'s cell. *Expected:* *T* is damaged, no surrounding vehicle is, and no draw
  R(−2, 2) is taken (claim 1 b). *Falsified by:* any fragment damage, or such draws. Run 2: *T*
  stands on land beside the bridge, fragment count 1, with the only other house-B vehicle on the
  deck over a water cell of ring 1 from *T*'s cell. *Expected:* the deck vehicle receives no
  fragment from the scan, and 2 draws R(−2, 2) are taken at the burst for the fragment's random
  cell. *Falsified by:* the deck vehicle hit with no draws taken.

## Changes for other specs

- **S-0306 (approved, revision 34), claim 21.** "A projectile with a shrapnel weapon also launches
  fragments (Open questions)" can point to S-0318 instead of its Open questions.
- **S-0306, Open questions.** The item on "the selection of targets, the draws and the flight of
  the fragments of a shrapnel weapon" is answered by S-0318 for the selection, the draws and the
  launch (claims 1 to 6 and the order table), and can be dropped or narrowed to the launch velocity
  of a fragment that is not invisible (S-0318 Open questions).
- **S-0306, Import mapping.** The sentence that the original also reads "a shrapnel weapon and
  count", which "are not imported here", can say instead that S-0318 imports them.
- **S-0303 (approved, revision 19), State and ordering, *Order of a projectile's detonation*.** A
  row for fragments between rows 1 and 2: "the fragments of S-0318 (claims 3 and 4), for a
  projectile with a fragment weapon | two ranged draws per fragment sent to a random cell, none
  for a fragment sent to an object". A sentence after the table, for clarity: the table covers one
  detonation; after each detonation that is not an airburst, S-0306 claim 21's ranged draw
  R(256, 512) and one raw value follow (S-0318 State and ordering).
- **S-0306, claim 20.** The point that (a) and (b) leave is also the burst point of S-0318 (Terms):
  (c) moves the detonation point of the projectile's own area damage and explosion, not the point
  from which its fragments burst (S-0318 claim 1).
- **S-0203 (approved, revision 39), claim 5.** "unless a removal costs it that turn (claim 10),
  in which case its first turn is in tick *n* + 1" can allow a later tick in the same way: its
  first turn is in the first tick in which no removal costs it the turn, which for the third of
  three fragments is tick *n* + 2 (S-0318 claim 5). Claim 10 itself needs no change: its
  objects that entered play during the turn already include fragments.
- **S-0306, State and ordering, *Ordering*.** "in which case its first turn is in tick *n* + 1"
  can allow a later tick: a removal in tick *n* + 1 can cost the projectile that turn as well,
  as it does for the third of three fragments (S-0318 claim 5), so its first turn is in the
  first tick in which no such removal costs it the turn.
- **S-0306, claims 1, 2 and 6.** A fragment is not a shot: its speed is the fragment weapon's
  weapon speed *S*<sub>w</sub> of claim 1, without claim 1's shot speed or claim 2's cap, and it
  is launched in its own direction rather than at the pitch of claim 6, believed to be a fixed
  elevation of about an eighth of a turn (S-0318 claims 5 and 6; the elevation is rated low
  there and is not measured). Claim 6's "Every projectile, whatever its kind" can except fragments,
  pointing to S-0318.
- **S-0316 (approved, revision 27), claim 8 and Open questions.** "the fragment rules, which have
  no owner yet (S-0306 Open questions)" can point to S-0318. Whether a fragment launched after
  the projectile's firer has gone still makes a bolt is answered by S-0318 claim 8: it makes none.
- **S-0316, Open questions, *Fragment colour*.** Answered by S-0318 claim 8: the fragment
  weapon's own alternate-colour flag selects a fragment bolt's side-strand colour; the flag of the
  weapon that fired the bursting projectile plays no part. Claim 8's "Which weapon's
  alternate-colour flag gives the bolt its colour ... are open" can point to S-0318.
- **S-0316, claim 2.** The ranking of the line-effect flags (the laser flag wins over the
  electric-bolt flag) can say that it is for shots: a fragment weapon with both flags makes both
  a beam and a bolt for each fragment (S-0318 claim 8).

## Open questions
- **Launch velocity of a non-invisible fragment** (claim 6). The direction is known and the
  elevation is believed, but the exact components, whether the fragment's flight rule keeps them in its first
  turn, and whether such a fragment takes S-0306's launch scatter draws (claims 4 and 9) for
  inaccurate or flak-scattering fragment projectiles, are not established.
- **Whether anything bounds a fragment chain** (claim 7, Edge cases). No limit of the original's
  own on the chain level is known, and whether anything, for example running out of room for
  further projectiles, ends a chain that names itself is not established. Our engine's
  provisional bound is the lead's call (Questions for the project lead).
- **Random cell outside the map** (claim 4). Where a fragment aimed at a drawn cell beyond the map
  edge goes is not established.
- **How many objects one shot damages in Q-1714's stock pairing.** On M-0021's test pairing,
  built after Q-1714's with a fragment count of 3, single-object warheads and fragments that do
  not burst again, every judged shot damaged its target and 3 further enemy vehicles, with 3 or
  more candidates in the rings each time: 1 + *n* objects, as claims 1 to 5 give. Claims 1 to 5
  bound one burst at 1 + *n* objects only for a shot that detonates once and is not an airburst
  projectile, whose warhead and fragment warhead each damage a single object and whose fragments
  do not burst again, detonate once and are not airburst projectiles. Whether the stock pairing's
  weapons (SH6 names a fragment side from the files in use) meet those conditions in each
  veterancy rank, and so what the count is for that pairing, is not established here: it is our
  engine's own comparison, the same rules applied to the stock data, and SH6 has not been run.
- **The rest of Q-1714.** For its test pairing, M-0021 also records which group fires first and
  at what distance (question 1), the shortest gap between one vehicle's shots, as a relation to
  the weapon's rate of fire and the house's rate-of-fire factor (question 2, first half), and
  when the stage-raising group reaches its top stage against the first vehicle lost (question
  3). S-0307, S-0301 and S-0609 own those rules. Whether one group spreads its fire (question 4)
  is not in M-0021 and stays open; S-0309 owns target choice. For the stock pairing none of these
  is established here.

## Questions for the project lead

- **A fragment chain that never ends** (Edge cases). Our engine provisionally lets a projectile of
  chain level *d* (Terms) burst only when *d* < 16, so a fragment of level 16 detonates without
  bursting. Whether to keep that bound, choose another, or follow the original without one, is
  the lead's call.
