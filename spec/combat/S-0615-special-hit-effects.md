---
id: S-0615
title: Special-hit effects (death-kind animations, the burning walker and the disc laser)
topic: combat
revision: 1
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0615: Special-hit effects (death-kind animations, the burning walker and the disc laser)

## Summary

S-0107 claim 17 chooses an infantry death kind and hands most kinds to "animations that other specs
describe". This spec owns those remaining producers, and one weapon whose hit is itself a special
effect. It covers:

- what each death kind creates, when, at which position and with which owner (claims 1 to 3),
- the *burning walker*: a burning-infantry animation that runs about, prefers water and draws random
  numbers (claims 4 to 11),
- the *disc laser*: the ring of beams that a disc-laser weapon draws before its damage lands 17
  ticks after the shot, with its range re-check, cancellation and detonation (claims 12 to 18).

This spec covers skirmish only; network games and campaigns are out of scope.

This spec does not restate these:

- death-kind selection: S-0107 claim 17,
- infantry death and removal: S-0312 claim 8,
- generic animation playback, events and damage over time: S-0109 claims 13 to 17,
- particle systems: S-0303 claim 20,
- creation draws: S-0202 claim 15,
- art lookup: S-0007 claims 28 and 29.

The virus death (kind 8) belongs to S-0613, the mutation death (kind 9) to S-0611 and the disc's
draining weapon to S-0608.

## Terms

*Tick*, *update*, *turn pass*, *animation*, *stage*, *advance*, *repeat factor*, *loop delay*,
*image frame*, *land type*, *height level*, *bridge deck*, *cell layer*, *step direction*,
*8-direction*, *n-step facing*, *direction*, *facing*, *scenario random stream*, *ranged draw*,
*fire point*, *detonation*, *hit*, *in play*, *off the map* and *removed* are used as in the glossary,
S-0109 and S-0203. *Disc laser* is as in S-0203 (Terms). *Shot damage D*ₛ is as in S-0302 claim 2.

| Term | Meaning |
|---|---|
| death kind *w* | The integer that S-0107 claim 17 selects for a dying infantry unit. |
| game-wide death animation | One of the animation types that the rules data names once for the whole game, one per death kind 3, 4, 6, 7 and 10 (Import mapping). Kind 5 uses a type chosen by registration index (claim 1). |
| burning walker | An animation whose current type has the burning-walker flag (Import mapping). It replaces S-0109's normal playback with claims 4 to 11. |
| running frames *R* | The burning-walker type's imported running-frame count: the number of image frames per direction in its running cycle. |
| walker destination | A world point that a burning walker is heading to, or *none*. |
| retry count | The number of times a burning walker has reached a destination and picked another one. It starts at 0. |
| expiring | The final state of a burning walker, in which it plays its dying frames and is then removed (claim 11). |
| ring point | One of the 16 points around a disc-laser firer between which the disc laser's arc beams run (claim 17). |
| arc step *k* | A disc laser's step counter, from 0 to 8 (claim 13). |
| beam | A drawing-only line of laser light with a duration in ticks, an intensity that fades over that duration and three colours: inner, outer and spread. It has no effect in play. The look of a beam is shared with ordinary laser weapons and is not owned here (Open questions). |

## Behaviour

### Death kinds

1. **What each death kind creates.** These rows apply when S-0107 claim 17 uses the death kind *w*
   in the ordinary human death cases of that claim's table, after its higher-priority water,
   cyborg, jumpjet, own-animation-list and non-human cases have been excluded. Each dispatch
   below creates at most one animation; any extra virus effect of a parachuting victim belongs
   to S-0613 claim 3.

   | *w* | Effect |
   |---|---|
   | 0, or 11 and above | No animation. The unit is removed at once (claim 3). |
   | 1, 2 | Death sequence 1 or 2 of the unit itself (S-0107 claim 17). |
   | 3 | The game-wide explosion death animation. |
   | 4 | The game-wide burning death animation. Whether its type is a burning walker is determined by the burning-walker flag (Import mapping; claims 4 to 11). |
   | 5 | The animation type at registration index 1 (S-0004 claim 4), not necessarily the second listed type. No dedicated key names it. |
   | 6 | The game-wide head-pop death animation. |
   | 7 | The game-wide nuked death animation. |
   | 8 | Virus death: S-0613. |
   | 9 | Mutation: S-0611. |
   | 10 | The game-wide crushing-blow death animation. |

   S-0107 claim 17 sends a unit killed by a laser fence to kind 5 and a parachuting unit to kind 3,
   so those deaths also create these animations.
   - Backing: `measurable` (scenario DK1)
   - Confidence: high

2. **How a death animation is created.** For kinds 3 to 7 and 10:
   - **Position.** One animation of the type in claim 1 is created at the dying unit's position.
   - **Settings.** No loop delay, repeat factor 1, no creator-requested reversal, and no forced depth adjustment.
     The type's own reverse flag and depth adjustment still apply (S-0109 claims 8 and 26).
   - **No owner house.** The animation belongs to no house and is attached to no object. The
     virus-death animation's different ownership belongs to S-0613 claim 2.
   - **Removal.** The unit is removed in the same processing (S-0312 claim 8). The crash exception
     of S-0107 claim 17 applies.
   - **Missing type.** When the rules data names no type for the kind, or the animation cannot be
     created, the unit is still removed with no animation.
   - **Draws and effects.** The dispatch itself adds no draw. Creation and immediate start-event effects
     have their usual conditional draws (S-0202 claim 15; S-0109 claim 13). From then on the animation plays by S-0109 like any other, with its middle
     event effects (scorch marks, craters, particles) and its damage over time. The exception is a
     burning-walker type, which plays by claims 4 to 11.
   - Backing: `measurable` (DK1, DK2)
   - Confidence: high

3. **Kinds without an animation.** For kind 0 and for kinds 11 and above, no animation is created
   and the unit is removed in the same processing, or begins its crash if its type has the crash
   flag (S-0107 claim 17). The death-kind dispatch adds no draw for these kinds; this does not exclude draws
   from the unit's other death effects.
   - Backing: `measurable` (DK1)
   - Confidence: high

### The burning walker

4. **Frozen playback and the image frame shown.** A burning walker's own processing (claims 5
   to 11) comes at the start of each of its updates, before S-0109's playback steps.
   - **Frozen stage.** While the walker is not expiring, its stage does not advance. So its updates produce no advance-triggered middle
     event, follow-up change or damage over time (S-0109 claims 14 to 17). The creation start
     event still applies, including an immediate middle event when applicable (S-0109 claim 13).
   - **Running frame.** The image frame shown is Formulas F1. It combines the 8-direction from the
     walker's position towards its walker destination with the game's tick number, so each
     direction's running cycle advances one frame every 3 ticks. With no destination the walker
     shows its first running cycle.
   - **First update.** A new walker has no destination. Its first update chooses one by claims 5
     and 6, without claim 9's retry and water tests, and the walker does not move in that update.
     If the choice gives *none*, the walker starts expiring at once (claim 11). A walker that is
     falling does not choose.
   - Backing: `measurable` (FW1)
   - Confidence: high

5. **Choosing a destination: the water search.** A destination is chosen from the walker's current
   cell (*x*₀, *y*₀). The search considers the 100 cells with *x*₀ − 5 ≤ *x* ≤ *x*₀ + 4 and
   *y*₀ − 5 ≤ *y* ≤ *y*₀ + 4.
   - **Candidates.** A cell is a candidate when all of these hold:
     - it lies in the usable area,
     - its land type is *water*,
     - it carries no bridge deck,
     - neither neighbour in step direction 6 nor neighbour in step direction 0 carries a bridge
       deck.
   - **Best cell.** Each candidate is scored by Formulas F2, and the candidate with the lowest
     score is the best cell. On a tie the earlier one in scan order wins. Scan order runs through
     *x* from low to high and, for each *x*, through *y* from low to high.
   - **Bias.** The score favours cells to the south-east: a cell with a larger *x* or a larger *y*
     than the walker's cell scores 3 lower.
   - Backing: `measurable` (FW1)
   - Confidence: high

6. **Choosing a destination: the step and the random fallback.**
   - **Step towards the best cell.** If a best cell exists, the three candidate cells are the
     diagonal neighbour one step towards it (both coordinates changed by the signs of their
     differences), the neighbour with only *x* changed, and the neighbour with only *y* changed.
     The selected cell is the first passable candidate in that priority order. The destination is
     its centre at height 0. If the best cell is the walker's own cell, all three candidates are
     that cell.
   - **Random fallback.** If no best cell exists or none of those three candidates is passable,
     the destination is determined as follows:

     | Outcome | Result |
     |---|---|
     | A ranged draw from the scenario random stream gives *r* in [0, 7], and at least one neighbouring cell is passable in cyclic step-direction order beginning at *r* | The first such neighbour is selected; its centre at height 0 is the destination. |
     | No neighbouring cell is passable in that cyclic order | The destination is *none*. |

     The cyclic order is *r*, *r* + 1, …, *r* + 7, modulo 8. This is the destination choice's
     only ranged draw; animation creation and event effects retain their own draws.
   - Backing: `measurable` (FW1, FW2)
   - Confidence: high

7. **Passable cells for a burning walker.** Let *z* be the walker's height and *L* the height level
   of the cell under test.
   - **On a bridge deck.** If the cell carries a bridge deck and *z* − 104 *L* > 208, the walker
     counts as on the deck. The cell is passable when its deck layer holds no vehicle, aircraft or
     building. Infantry does not block it.
   - **Otherwise,** the cell is passable when all of these hold:
     - its land type is not *rock*, *tunnel* or *wall*,
     - it has no wall overlay,
     - its ground layer holds no vehicle, aircraft or building,
     - 104 *L* − *z* ≤ 208, so the ground there is at most two height levels above the walker.

     Infantry does not block in this case either.
   - Backing: `measurable` (FW1, FW2)
   - Confidence: high (the tests); medium (that infantry never blocks)

8. **Running.** While the walker is not expiring and its walker destination is not *none*:
   - **Moving.** If the horizontal table-root distance (S-0204 claim 5) from its position to the
     destination is 19 leptons or more, it takes a nominal 18-lepton step along the direction
     (S-0204 claim 8) towards the destination. The step is computed in floating point and
     truncated (Formulas F3).
   - **Height.** Its new height is the terrain height at the new point. If its old height was more
     than 208 leptons above the terrain height there, it is 416 leptons above that terrain height
     instead: a walker on a bridge deck stays on the deck.
   - **Arrival.** Otherwise it has arrived, and claim 9 applies, unless the walker is falling: a
     falling walker that has arrived does nothing more in that update (claim 10).
   - Backing: `measurable` (FW1)
   - Confidence: high

9. **Arrival.** When a walker arrives, it first chooses a new destination by claims 5 and 6, from
   the cell it is in. Choosing may consume the fallback draw. The walker then starts expiring
   (claim 11), and does not move this update, if any of these holds:
   - the new destination is *none*,
   - its retry count is already 7 or more,
   - the old destination's cell has land type *water* or *beach*, and the walker's height above
     the terrain at its current position is
     at most 104 leptons, one height level.

   Otherwise:
   - its retry count rises by 1,
   - it is placed exactly on the old destination, with its height set by claim 8's height rule,
   - the new destination becomes its walker destination.

   So a walker that keeps finding destinations runs to at most 8 destinations in all, counting the
   first. It expires at the latest when it arrives at the eighth. Counting the choice at its first
   update, it makes at most 9 choices, so at most 9 fallback draws.
   - Backing: `measurable` (FW1, FW3)
   - Confidence: high (destination-cell land test, draw order and retry limit); medium
     (height above the terrain)

10. **Falling off a deck.** After a running step or a placement on arrival (claims 8 and 9), a walker
    that is not falling starts to fall when all of these hold:
    - its height is at least 416 leptons above the terrain at its position,
    - neither its cell nor its neighbours in step directions 6 and 0 carry a bridge deck.

    This happens, for example, when its deck has been destroyed under it, or when it runs off the
    end of a deck: claim 7 lets a walker at deck height into a cell without a deck, and claim 8
    keeps its height. Its descent follows S-0511 claim 10. While falling, it still takes running
    steps towards its destination, but it does not handle arrivals and does not choose
    destinations.
    The running step's height placement precedes descent on that update. Thus a moving
    walker can be raised back to deck height repeatedly while falling; descent becomes
    uninterrupted when it stops taking running steps near its destination.
    Landing starts expiry. Landing on water also creates the first game-wide splash animation
    at its position plus 3 leptons of height. This uses the splash list whose selection in
    other deaths is owned by S-0312 claim 11; it uses the first entry, not that claim's last entry.
    - Backing: `measurable` (FW4)
    - Confidence: high for continued running and landing expiry; medium for the combined path

11. **Expiring and removal.** The frame and lifetime below describe forward playback with
    start frame 0, no earlier pass end and no follow-up type. Other playback combinations are
    not established here.
    - **Start.** Expiry from claims 4 or 9 clears the walker destination; expiry on landing can
      retain it, but no further running step occurs. The stage is set to 8 *R* + 1, one frame
      beyond the first frame after the eight running cycles. From
      then on it has a one-update stage delay, not normalised for game speed. The first
      advance is on a later update, not the update that starts expiry.
    - **Removal.** At the start of each update, an expiring walker whose stage is at least
      ⌊*N* / 2⌋ − 1 is removed, where *N* is its sprite's image-frame count.
    - **No middle event.** The middle image frame (S-0109 claim 14) is never reached, so an
      expiring walker produces no middle event either.
    - **Damage.** Expiring advances deliver ordinary damage over time when enabled
      (S-0109 claim 17). A running walker does not advance, so it delivers none.
    - Backing: `measurable` (FW3)
    - Confidence: high for threshold and enabled damage; medium for lifetime with altered art

### The disc laser

12. **The shot.** When an object fires a weapon that has the disc-laser flag, the shot creates a
    disc laser instead of a projectile.
    - **Shot.** The disc laser holds the firer, the target, the weapon and the shot damage *D*ₛ.
      *D*ₛ is computed exactly as S-0302 claim 2 computes it for a projectile, with the same
      garrison, bunker and open-topped factors.
    - **Not at the shot.** No projectile, no report sound and no ammunition use (S-0301 claim 10).
      The burst count and the rearm countdown follow S-0301 (claim 8: no berserk halving), and the
      garrison fire-point index does not advance (S-0313 claim 6).
    - **Start point.** The disc laser's start ring point is *i*₀ (Formulas F4): the ring point on
      the side of the firer away from the target.
    - Backing: `measurable` (DL1, DL5, DL6)
    - Confidence: high

13. **Schedule.** Disc lasers are updated in their own part of each tick, before the turn pass
    (S-0203 row 12). Their arc step *k* starts at 0, and a step happens on every second update.
    - **When.** For a shot made in tick *T*'s turn pass, step *k* (0 to 7) happens in tick
      *T* + 1 + 2*k*. The final step, with the damage, happens in tick *T* + 17, before that tick's
      turn pass. The disc laser is removed in its update in tick *T* + 18.
    - **Re-check.** Every step, including the final one, first makes claim 14's re-check.
    - **Order.** Several disc lasers update in the order S-0203 gives. Each one's steps are
      independent of the others.
    - Backing: `measurable` (DL1)
    - Confidence: high

14. **Range re-check and cancellation.**
    - **Range.** At each step, the disc laser measures the table-root distance in three dimensions
      from the firer's position to the target's position. The vertical difference is taken as 0
      when the firer is in the air. For a building target, 64 (*W* + *H*) leptons is subtracted
      (foundation width *W* and height *H* in cells), clamping at 0.
    - **Cancelled by range.** If the distance exceeds the weapon's range field (leptons, with no
      additional range term of S-0307), the disc laser is removed without damage and without
      drawing that step.
    - **Cancelled by crash.** It is also cancelled if the firer has begun a crash (S-0312 claim
      16).
    - **Cancelled by loss.** It is cancelled, without damage, as soon as its firer or its target
      leaves play: goes off the map, is removed, or otherwise stops being a valid object to point
      at.
    - **Re-check only.** Facing, line of fire and the firing check are not re-tested.
    - Backing: `measurable` (DL2, DL3)
    - Confidence: high (range test and the loss of firer or target); medium (the exact distance
      form for a building target)

15. **The arc.** At arc step *k* from 0 to 7, after a passing re-check:
    - **Charge-up sound.** At *k* = 0 only, the rules data's disc-laser charge-up sound plays, if
      given (S-0701 owns how it is placed).
    - **Two beams.** Two beams are drawn:
      - one from ring point *a* = (*i*₀ + *k*) mod 16 to ring point *a* + 1 (mod 16),
      - one from ring point *b* = (*i*₀ − *k*) mod 16 to ring point *b* − 1 (mod 16).

      So the two arcs start on the far side of the firer and close on the side facing the target.
    - **Beam settings.** Each beam lasts 8 − *k* ticks and fades from full intensity to half
      intensity. Its colours are the weapon's inner, outer and spread laser colours.
    - **House colours.** When the weapon has the house-colour flag, the inner colour is the
      firer's house laser colour and the outer colour is that colour with each channel halved,
      toward zero, using integer channel arithmetic. The weapon's spread colour is kept.
    - **Ring position.** The ring points are placed around the firer's fire point for weapon slot
      0 (S-0313), taken at the time of the step, at that point's height.
    - **Next step.** *k* then rises by 1 and the next step comes two updates later.
    - Backing: `measurable` (DL1 for the timing; DL5 visual)
    - Confidence: high (indices, durations, timing); medium (colour derivation)

16. **The final step.** At *k* = 8 the two arcs meet: *a* = *b* = *i*₀ + 8 (mod 16), the ring point
    facing the target as of the shot. After a passing re-check, these effects occur in that update:
    - **Beam.** A beam is drawn from ring point *a* to the target's target point (where shots aimed
      at it detonate). It lasts the weapon's laser-duration field in ticks, fades from full
      intensity to none, and uses claim 15's colours.
    - **Detonation.** A detonation is made at the target's target point (S-0303). The amount is
      *D*ₛ and the warhead is the weapon's warhead. The firer is the attacker, there is **no
      attacking house**, and the detonation may reduce resource cells (S-0303 claim 17).
    - **Report sound.** The weapon's report sound plays (S-0701 owns how it is placed). Its variant
      is chosen by S-0701 claim 21 from the firer's own stored value.
    - **Removal.** The disc laser is then removed at its next update.

    The firer is the attacker, so veterancy and retaliation follow it (S-0302). Effects that specifically require
    an explicit attacking house see none (`quirks.disc_laser_no_house`). An absent explicit
    house does not prevent ordinary kill credit derived from the attacker (S-0302 claim 14).
    - Backing: `measurable` (DL1, DL4). The absent explicit attacking house is `unmeasured`:
      ordinary kill credit cannot distinguish it from the firer's house, and no discriminating
      house-only effect is identified here.
    - Confidence: high

17. **Ring points.** The 16 ring points lie on a circle of radius 240 leptons in the horizontal
    plane around the ring centre. Point 0 is due north of the centre (smaller *y*), and the points
    run clockwise at the whole-degree approximations to sixteenth-turn spacing in F6.
    Their displacements from the centre are whole leptons.
    - Backing: `measurable` (DL5)
    - Confidence: high (radius, order); see Edge cases for points 14 and 15

18. **Draws.** A disc laser draws nothing from either random stream itself. Its detonation draws as
    S-0303 says, and its report variant uses a value stored at the firer's creation (S-0701 claim
    21).
    - Backing: `measurable` (DL1 with the stream recorder)
    - Confidence: high

## Formulas

**F1. Burning-walker image frame** (claim 4). *t* is the game's tick number (S-0201). *d* is the
8-direction of the direction *f* from the walker's position to its destination:
*d* = ⌊(⌊*f* / 4096⌋ + 1) / 2⌋ mod 8, integer arithmetic (half-way rounds up). With no destination,
use *d* = 7.

> frame = *R* × (7 − *d*) + (⌊*t* / 3⌋ mod *R*)

The cycles are stored counter-clockwise from north-west in the sprite: north (*d* = 0) uses cycle 7
and north-west (*d* = 7) cycle 0. The expression yields frames 0 to 8*R* − 1.

**F2. Water-search score** (claim 5). For a candidate (*x*, *y*) with Δ*x* = *x* − *x*₀ and
Δ*y* = *y* − *y*₀ (cells):

> score = 0 for the walker's own cell; otherwise
> score = trunc(√ₜ(Δ*x*² + Δ*y*²)) − (3 if Δ*x* > 0 or Δ*y* > 0, else 0)

The differences and sum of squares are exact integers in cells and square cells. √ₜ is the
floating-point table square root of S-0204 F1. Truncation is toward zero to a whole-cell score,
then the bias is subtracted in integer arithmetic. Lowest
score wins, first in scan order on ties. The search starts from a score of 10000, which every
candidate beats, so that value only marks that no candidate has been found yet.

**F3. Running step** (claim 8). With *f* the direction from the walker's position (*x*, *y*) to its
destination (S-0204 F3), first take signed facing *s* = *f* for *f* < 32768 and
*s* = *f* − 65536 otherwise, in integer arithmetic. Then in floating point (double precision,
table trigonometry of S-0105 F1):

> θ = (*s* − 16383) × (−2π / 65534)
> *x*′ = trunc(*x* + 18 cos θ), *y*′ = trunc(*y* − 18 sin θ)

Truncation is toward zero after adding the floating-point displacement to the whole-lepton
coordinate. The nominal step length is 18 leptons, not a guarantee of exactly 18 leptons after
coordinate truncation. North decreases *y* and east increases *x*. The zero angle is at *f* = 16383, and the scale is 2π / 65534, not
2π / 65536: both are the original's constants and are kept because they change the truncated step. The comparison with 19 leptons
uses the horizontal table-root distance of S-0204 claim 5 in whole leptons.

**F4. Disc-laser start point** (claim 12). With *f* the direction from the firer to the target
(S-0204 claim 8) at the shot:

> *i*₀ = (⌊(⌊*f* / 2048⌋ + 1) / 2⌋ mod 16 + 8) mod 16

That is, the 16-step facing of the target from the firer (S-0204 F4), turned half a turn.
All divisions, floors and residues here are integer operations.

**F5. Disc-laser schedule** (claim 13). Shot in tick *T*'s turn pass; arc step *k* (0 ≤ *k* ≤ 7) in
tick *T* + 1 + 2*k*, final step in tick *T* + 17, removal in tick *T* + 18. Arc beams of step *k* last
8 − *k* ticks. Tick and step arithmetic here is exact integer arithmetic; the drawing
lifetime boundary is shared with ordinary beams and is not established here.

**F6. Ring point positions** (claim 17). For point *i* (0 to 15), the angle in whole degrees is

> φᵢ = (270 + ⌊22.5 *i*⌋) mod 360

so the odd-numbered points use the half-degree angle rounded down (292°, 337°, 22°, …). Then

> Δ*x*ᵢ = trunc(240 cos φᵢ), Δ*y*ᵢ = trunc(240 sin φᵢ) leptons, Δ*z* = 0

with φᵢ converted to radians in double precision, table trigonometry (S-0105 F1) and truncation
toward zero. With *y* growing southward, φ = 270° is north and φ = 0° is east. Points 14 and 15 are
at 225° and 247° (Edge cases).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Death kinds with a game-wide animation | 3, 4, 6, 7, 10 | — | Kind 5 by registration index 1 |
| Height level | 104 | leptons | Glossary |
| Walker step | 18 | leptons per update | Claim 8 |
| Walker arrival radius | 19 | leptons | Distance below which it has arrived |
| Walker deck lift | 416 | leptons | Four height levels |
| Walker climb limit | 208 | leptons | Two height levels |
| Walker search square | 10 × 10 | cells | From −5 to +4 cells |
| Walker south-east bias | 3 | cells of score | F2 |
| Walker search start score | 10000 | — | F2 |
| Walker retry limit | 7 | retries | Claim 9 |
| Walker frame period | 3 | ticks per running frame | F1 |
| Walker expiring start stage | 8 *R* + 1 | stage | Claim 11 |
| Ring points | 16 | points | Claim 17 |
| Ring radius | 240 | leptons | Claim 17 |
| Arc steps | 8 | steps | Plus a final step |
| Updates per arc step | 2 | updates | Claim 13 |
| Shot to damage | 17 | ticks | F5 |
| Arc beam intensity | 1.0 to 0.5 | fraction | Claim 15 |
| Final beam intensity | 1.0 to 0 | fraction | Claim 16 |

## State and ordering

**Burning walker** (one update):

| State | Event | Result |
|---|---|---|
| running, no destination | update | choose destination (claims 5, 6); expire if *none* |
| running | distance ≥ 19 | move 18 leptons (claim 8); frame by F1 |
| running | distance < 19 | claim 9: new destination or start expiring |
| running, not falling | 416 or more above terrain, no deck at cell or neighbours 6, 0 | start falling (claim 10) |
| falling, distance ≥ 19 | update | running step and height placement, then descent (claims 8, 10) |
| falling, distance < 19 or no destination | update | descent without choosing another destination |
| falling | landing | begin expiring; water splash as in claim 10 |
| expiring | stage ≥ ⌊*N* / 2⌋ − 1 at update start | removed |
| expiring | otherwise | advance one stage |

**Disc laser:**

| State | Event | Result |
|---|---|---|
| created at shot (*k* = 0) | update in tick *T* + 1 | re-check; charge-up sound; two beams; *k* = 1 |
| waiting one update | update | wait ends |
| arc step *k* < 8 due | update | re-check; two beams of 8 − *k* ticks; *k* + 1 |
| final step due (*k* = 8) | update | re-check; beam to target; detonation; report sound |
| finished | next update | removed |
| any | firer or target leaves play | removed, no damage |
| any step | re-check fails | removed, no damage, nothing drawn that step |

Ordering consequences:
- The disc laser's damage lands before any object's turn in tick *T* + 17. A target that would move
  out of range in that tick's turn pass is still hit.
- A disc laser fired in the same tick as an ordinary shot whose projectile is instant still lands 17
  ticks later.
- Its report sound comes with the damage, not with the shot.
- A burning walker's fallback draw happens in its own turn in the update order (S-0203), so it
  interleaves with other objects' draws by update order.

## Edge cases and quirks

- **Disc laser without an attacking house** (`quirks.disc_laser_no_house`, default on). The
  detonation names the firer as attacker but no attacking house. With the switch off, the
  attacking house is the firer's owner, as for a projectile's detonation (S-0302 claim 1).
- **Ring points 14 and 15 misplaced (display only, no switch).** Points 14 and 15 are placed by
  F6, at 225° and 247°. The original uses 235° and 257°, so those two ring points and the beams
  that touch them sit about 42 leptons off the circle's even spacing.
- **Laser duration over 127 (display only, no switch).** The final beam lasts the laser-duration
  field as given. The original keeps that value as a signed 8-bit number, so a field above 127
  wraps to a negative duration and the beam is cut short.
- **Start point fixed at the shot.** *i*₀ is set once. If the target moves round the firer during
  the 17 ticks, the arcs still close at the old side, and the final beam runs from that ring point
  to the target's current target point.
- **Firer moving.** The ring follows the firer's current fire point at each step. Beams already
  drawn stay where they were drawn.
- **Walker with no water nearby.** Each destination comes from the random fallback, so the walker
  stumbles to random neighbouring cells, drawing once per destination, until 7 retries are used or
  no neighbour is passable.
- **Walker on a bridge deck.** The search rejects water cells under or next to a deck, so a walker
  on a deck over water does not run to the water below; it uses the fallback and the deck test of
  claim 7.
- **Kind 5 by registration index.** Changing registration index 1 changes the kind 5 death animation,
  and the laser-fence death with it. Registration follows S-0004 claim 4, including types
  first named outside the list.

## Import mapping

| Concept | Rules key (section) | Parse form | Absent |
|---|---|---|---|
| Death kind *w* of a warhead | `InfDeath` (warhead) | integer | S-0107 |
| Kind 3 animation | `InfantryExplode` ([General]) | animation type name | none: no animation |
| Kind 4 animation | `FlamingInfantry` ([General]) | animation type name | none |
| Kind 6 animation | `InfantryHeadPop` ([General]) | animation type name | none |
| Kind 7 animation | `InfantryNuked` ([General]) | animation type name | none |
| Kind 8 animation | `InfantryVirus` ([General]) | S-0613 | |
| Kind 9 | `InfantryMutate` ([General]) | S-0611 | |
| Kind 10 animation | `InfantryBrute` ([General]) | animation type name | none |
| Kind 5 animation | `[Animations]` and other animation references | registration index 1, S-0004 claim 4 | no dedicated key |
| Burning-walker flag | `IsFlamingGuy` (animation type, art data) | boolean | no |
| Running frames *R* | `RunningFrames` (animation type, art data) | integer | 0 frames; a burning-walker type needs a positive count |
| Disc-laser flag | `DiskLaser` (weapon) | boolean | no |
| Charge-up sound | `DiskLaserChargeUp` ([AudioVisual]) | sound name | none: silent |
| Range | `Range` (weapon) | range field of S-0307 | S-0307 |
| Final beam duration | `LaserDuration` (weapon) | integer ticks | 10 ticks |
| Inner, outer and spread colours | `LaserInnerColor`, `LaserOuterColor`, `LaserOuterSpread` (weapon) | three integer colour channels, S-0003 | each defaults to (0, 0, 0) |
| House-colour flag | `IsHouseColor` (weapon) | boolean | no |
| Report sound | `Report` (weapon) | sound list | S-0701 |

Defaults here apply to newly created types or a fresh rules database. An omitted game-wide
key can retain an earlier value under S-0006 claims 1 and 10. Invalid or zero running-frame
counts are outside claim 4's verified scope.

## Measurement scenarios

**Common setup.** Use tiny purpose-made skirmish maps, one human-controlled house and one enemy
bystander computer player kept away under the glossary's void rule. Use game speed index 4,
800 × 600, director and mods off, and stock data except each scenario's named test-map overrides
or test art file. Remove named test data afterwards. Run each case 10 times unless stated
otherwise. Use video, scripted input and read-only recording of ticks, positions, health,
animation types, stages, owners and both random streams. Endpoint and beam-setting recording,
where needed below, is a proposed read-only recorder capability, not an engine modification.
Separate producer draws from creation, death, sound and detonation draws. A repeatable mismatch
with any predicted outcome below falsifies the corresponding rule.

**DK1. Death kinds (claims 1 and 3).** Kill one ordinary human infantry type with no own death
animation list, not a cyborg or jumpjet, on dry ground. A named test-map warhead override supplies
each death kind 0 through 10 and 11. Record animation type, position and removal tick. Expect
claim 1's table, including no animation for kinds 0 and 11 and no extra dispatch draw. The
ordinary animation cases remove immediately; sequence and mutation cases follow their owning
specs. Repeat kind 3 while parachuting. A named test animation-list override changes registration
index 1, with a control changing another index: expect only the index-1 change to replace kind
5's animation. Different type, removal or dispatch draw falsifies the corresponding rule.

**DK2. Creation settings (claim 2).** Repeat DK1 kinds 3 through 7 and 10, recording owner,
attachment, position, frame history and draws. Expect no house or attachment, the victim's
position, no creator delay, and ordinary playback with repeat factor 1. Contrast kind 8 killed
by an attacker object: its animation has the attacker's house (S-0613 claim 2).
A named test art file varies the type's reverse flag, depth adjustment, loop count, start sound,
particles, scorch/crater flags and damage one at a time: expect S-0109's corresponding effects
and S-0202's conditional creation/event draws, not forced forward playback or one total pass.
A fresh-session named test rules file omitting one game-wide death type must still remove the
victim, with no animation. Different ownership, playback or dispatch draws falsify claim 2.

**FW1. Water choice and path (claims 4, 5, 7, 8 and 9).** Kill an infantry with a kind-4
warhead near a known water layout. Predict the best water cell using all of F2, not simply nearest
distance. Record first-update position, every later position, shown frame, destination and expiry.
Expect no first-update movement, then F1 and F3 with claim 8's height rule. Include trajectories
in all eight directions to distinguish signed-facing conversion and whole-coordinate truncation.
Repeat at speed indices 1 and 6: the running cycle still changes every 3 ticks. Separate maps
place water at search boundaries −5, +4 and +5, equal-score candidates that distinguish scan
order, and candidates that distinguish the south-east bias. Water under or beside bridge decks
must be excluded exactly as in claim 5. Repeat arrival on water, beach and a deck over water:
the old destination's land and the current height decide expiry. A different selected cell,
rounded coordinate, frame, height or expiry falsifies the respective rule.

**FW2. Cell entry and fallback (claims 6 and 7).** On a dry map observe 30 kind-4 deaths.
Predict each fallback choice from the recorded incoming stream state: one ranged draw over
[0, 7], followed by cyclic neighbour trials. Alternative layouts place vehicles, buildings,
infantry, walls, rock and tunnel tiles in those neighbours. Repeat on a bridge deck, with
occupants above versus below it, and beside terrain steps. Compare each observed height against
the 208-lepton inequalities in claim 7. Expect infantry not to block, the listed other occupants
to block on the tested layer, and the first passable neighbour to win. Box in all neighbours:
expect none and immediate expiry. A water-map variant blocks the diagonal step, then also the
x-only step: expect x-only, then y-only before any fallback draw. Wrong neighbour, an extra draw,
or a draw during running alone falsifies the corresponding rule.

**FW3. Lifetime and damage (claims 9 and 11).** On dry, passable ground record every arrival,
choice, stage and removal. Expect expiry at the eighth arrival after its new choice, at most
nine fallback draws including the initial choice, and no snap on the expiring arrival. Expect
stage 8*R* + 1 at expiry, the first advance on a later update, one stage per update thereafter,
and removal on an update starting at or above ⌊*N* / 2⌋ − 1. A named test art file keeps claim
11's forward/start/end conditions, enables positive damage and a middle-event particle effect,
and places a durable victim near the final point. Expect no running damage, S-0109 claim 17
damage on expiring advances and no middle-event particles. Different retries, draws, removal
frame, running damage or missing expiry damage falsifies these rules. Repeat at speed indices
1 and 6 to distinguish the fixed expiry delay from normalised playback.

**FW4. Lost deck (claim 10).** Create a walker on a high bridge, then destroy the deck after
it has a destination. Record horizontal position, world and terrain height and landing, over
land and over water. Expect running and claim 8's height placement while distance is at least
19 leptons, followed by descent in the same update. Near the destination expect uninterrupted
descent without destination choices. Landing starts expiry; water landing adds a splash
3 leptons above it. No running throughout the fall, new choices while falling, a different
height sequence or no expiry on landing falsifies claim 10.

**DL1. Disc laser timing.** Tiny map, game speed fixed, a Disc firing once at a stationary
building in range. Record the tick of the shot (burst count change), the tick the target's health
drops, and the tick of the report sound. Expected: health drops in tick *T* + 17, before that
tick's object turns; report in the same tick; the charge-up sound in *T* + 1. Repeat 10 times.
Record every arc step: expect *T* + 1, +3, …, +15, then the final step in *T* + 17 and
removal in *T* + 18. Record both random streams: expect no draws attributable to the disc laser
itself, apart from ordinary detonation consequences and sound-system draws owned by S-0701.
Repeat with two shots in the same tick: expect newest disc laser first, as S-0203 claim 1 says.
A different tick, order or extra producer draw falsifies claims 13 or 18.

**DL2. Range cancellation (claim 14).** Move the target out of range between steps and just
before the final step. Expect cancellation on the first due step beyond range, no beam that step
and no damage. Equality at the boundary passes; an excursion entirely between steps does not
cancel. Named test-map overrides give a ground firer the disc weapon and an additional range
bonus. Compare its height-sensitive check with the airborne Disc's horizontal check, and compare
building foundations of different sizes: expect table-root distance minus 64 leptons times
(width + height in cells), clamped to zero, and no additional range bonus. Turn away or introduce an obstacle after
the shot while staying in range: expect no facing or line-of-fire re-check. Opposite outcomes
falsify the respective parts of claim 14.

**DL3. Cancellation (claim 14).** Repeat DL1, killing the firer or target at *T* + 5. Also
start the firer's crash without immediate removal, and load a named test-map ground disc firer
into a closed transport. Expect no final damage after either object leaves play, and cancellation
at the next due step for a crash. A final hit falsifies the corresponding cancellation rule.

**DL4. Damage and attacker (claim 16).** A disc laser kills an enemy unit. Record health,
retaliation, veterancy and house statistics: expect the firer as attacker and ordinary kill
credit, which can include its house. A named test-map warhead varies versus and spread; place
resource beside the target. Expect stored shot damage and S-0303 armour/falloff and permitted
resource-reduction rules. Wrong damage, attacker credit or resource outcome falsifies these
parts of claim 16. Kill statistics cannot establish the absent explicit attacking house.

**DL5. Ring drawing (visual).** Screenshots of a Disc firing at a target due east, at ticks
*T* + 1 to *T* + 17. Expected: arcs start west of the fire point and close east; each step adds two
segments; point positions by F6 within one pixel. Under the original, points 14 and 15 visibly off the even
spacing (Edge cases). Record whole-lepton endpoints and beam settings as well as screenshots:
expect durations 8 − *k*, intensity parameters 1.0 to 0.5, and the final beam's imported duration
and 1.0 to 0 parameters. Named test-map weapon colours distinguish inner, outer and spread,
then enable house colours: expect house inner, integer-halved outer and unchanged spread.
Move firer and target while charging: new segments follow the current slot-0 fire point, old
segments stay put, the closing index stays fixed and the final endpoint follows the target.
A different endpoint, colour, duration or index falsifies claims 15 or 17, except the two original
display errors explicitly noted above. Exact beam rasterisation remains outside this spec.

**DL6. Shot state and defaults (claim 12, Import mapping).** A named test-map vehicle clone has
a disc weapon, finite ammunition and a multi-shot burst. Record projectile count, ammunition,
burst and rearm state and victim health. Expect no projectile or ammunition use, no report at
the shot, normal burst/rearm and no berserk halving (S-0301 claim 8). Repeat with a garrison
(S-0313 claim 6): its fire-point index must not advance. Compare damage against S-0302 claim 2,
including bunker and open-topped factors. Any different state or damage falsifies claim 12.
Fresh named weapon sections omitting duration and colours must show Import mapping's defaults.
Fresh non-walker animation sections can expose the default running-frame count safely; omitted
walker and disc flags must leave ordinary playback and projectile shots respectively.

## Open questions

- **Beam look.** The rasterisation of a beam (thickness, how inner, outer and spread colours and the
  intensity combine) is shared with ordinary laser weapons. No approved spec owns it; this spec
  only gives the parameters.
- **Water and beach height test.** Claim 9's test of height above the terrain; confirm by FW1 with
  a walker arriving on a deck over water (expected: no expiry from this test).
- **Unusual walker art.** Full expiry behaviour for nonzero start frames, reversed playback,
  early end limits and follow-up types remains untraced; claim 11 does not prescribe those
  combinations.
- **Absent attacking house.** A discriminating house-only effect is still needed for claim 16:
  kill statistics alone cannot establish its explicit house metadata.
- **Zero running frames.** The engine default is zero, but a walker needs a positive count for
  its frame cycle; failure and safe handling of invalid counts remain outside the verified scope.
