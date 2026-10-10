---
id: S-0313
title: Firing positions (where a shot starts, burst alternation, garrison muzzles, spray attack)
topic: combat
revision: 1
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: []
supersedes: []
---

# S-0313: Firing positions (where a shot starts, burst alternation, garrison muzzles, spray attack)

## Summary

The world position at which a shot starts (its *fire point*), computed from the firer's position,
its weapon slot's muzzle position, its turret displacement and facings, and its burst counter; the
special fire points of garrisoned buildings, buildings with pixel muzzle shifts and passengers of
open-topped transports; and the redirected targets of spray-attack firers. The fire point is game
state: projectiles start there (S-0306), so it changes flight times and what a projectile passes.

## Terms

Glossary terms are used as defined there (*lepton*, *facing*, *direction*, *reference point*,
*foundation*, *top corner*, *world height*, *vehicle*, *infantry*, *building*, *aircraft*,
*locomotor*, *weapon*, *target*, *scenario random stream*). From S-0105: *table trigonometry* (F1),
*32-step facing* *f*₃₂, *body transform* *D* (claim 7), the rotation *Rz* and the right-multiplying
composition convention (F2), the *model frame* (+x forward, +y to the left, +z up). From S-0301:
*firer*, *shot*, *burst counter* *k*, *garrisoned building*. From S-0310: *aiming facing*. From
S-0202: R(*a*, *b*). In addition:

- **Fire point.** The world position, in leptons (x, y and world height), at which a shot's
  projectile starts.
- **Base point.** For a building, the top corner of its foundation, at the building's world height.
  For every other firer, its position (its reference point, with its world height).
- **Muzzle position (F, L, H).** Three integers in leptons that the art data gives per weapon slot of
  a type (primary, secondary, their elite versions and the numbered weapons): forward, lateral
  (positive to the left) and up. Which slot a shot uses is S-0308's.
- **Turret displacement** *δ*. The type's imported turret displacement, in leptons along the
  forward axis (the same field S-0105 uses for drawing).
- **Alternate muzzle positions.** Five extra muzzle positions (numbered 0 to 4) that a transport's art
  data can give; passengers of an open-topped transport fire from them (claim 5).
- **Pixel shift.** A pair of integers in screen pixels in the art data, converted to a lepton shift
  by F3.
- **Muzzle point** *k* **of a garrisonable building.** The *k*-th pixel shift of the building type's
  list of garrison muzzle points (claim 6).
- **Primary fire pixel shift.** A pixel shift that a building type's art data can give for the
  shots of the building (claim 7).
- **Dual-shift flag.** A building type's imported flag that makes its primary fire pixel shift
  apply to the fire point of claim 1 instead of to the base point (claim 7).
- **A voxel turret, a voxel barrel.** A building's turret, or its barrel, that its type's art data marks
  as a voxel model (claim 8).
- **Turret anchor's pixel shift.** A pixel shift that a building type's art data gives for its
  turret (claim 8).
- **Turret pivot.** The point from which a firer with a muzzle position of (0, 0, 0) would fire by
  claim 1: its base point shifted by the turret displacement in the frame of *M* (claims 1 and 5).
- **Spray-attack type.** A type with the imported spray-attack flag (claim 9).

## Behaviour

1. **Fire point.** Unless claims 5 to 8 replace it, the fire point of a shot is its firer's base
   point plus a lepton shift. The shift is given by the translation (*u*, *v*, *w*), read in the
   model frame, of the transform *T* of F1. *T* is the product, in this order, of four factors:
   - the firer's frame *M*: for a firer that has a locomotor (vehicles, infantry, aircraft), the
     body transform *D* that its locomotor gives (S-0105 claim 7, with S-0106 tilt); for any other
     firer, the identity,
   - a translation by the turret displacement *δ* along +x,
   - the rotation about +z by the angle *θ* of claim 3,
   - a translation by the weapon slot's muzzle position (*F*, *L*, *H*), with the lateral part *L*
     negated when the firer's burst counter is odd (claim 2).

   The shift is (trunc *u*, trunc(−*v*), trunc *w*) leptons along map x, map y and up. Formulas F1
   and F2 give the arithmetic and its rounding. The model frame's +y is to the left of the forward
   axis, so map y, which grows towards map south, takes −*v*.
   - Backing: `measurable` (scenario P1)
   - Confidence: high (composition, the sign of *L*, the base point); medium (bit-exact single
     precision, which follows S-0105 F0)

2. **Burst alternation.** The lateral part of the muzzle position alternates within a burst: shots
   made with an even burst counter (the first, third, … shot of a burst; the counter a shot uses is
   the number of earlier shots in its burst, S-0301 claim 4) use +*L*, those with an odd counter
   −*L*. Because the counter
   restarts at 0 after each burst, a weapon with an odd burst count starts each burst on the same
   side. A weapon with burst count 1 always uses +*L*.
   - Backing: `measurable` (scenario P2)
   - Confidence: high

3. **The rotation angle.** *θ* depends on the firer's aiming facing *a* and, for a firer with a
   locomotor, on its body facing *b*, whose rotation *D* already holds. *a*₃₂ and *b*₃₂ are their
   32-step facings.

   | Firer | Aiming facing *a* | *θ* |
   |---|---|---|
   | vehicle with the turret flag (S-0310, claim 1) | its turret facing | −(π/16) × (*a*₃₂ − *b*₃₂) |
   | vehicle without the turret flag (S-0310, claim 1) | its body facing | 0 |
   | infantry | its body facing, always | 0 |
   | aircraft | its turret facing, always (S-0310, claim 5) | −(π/16) × (*a*₃₂ − *b*₃₂) |
   | turreted building (its type or an installed upgrade has the turret flag, S-0310 claim 4) | its turret facing | −(π/16) × (*a*₃₂ − 8) |
   | other building, with a target | the direction of its current target from its reference point | −(π/16) × (*a*₃₂ − 8) |
   | other building, with no target | its turret facing | −(π/16) × (*a*₃₂ − 8) |

   The angle is a double, narrowed to single precision before the table trigonometry of S-0105 F1
   is applied (F2).
   - Backing: `measurable` (scenarios P1 and P3, part b)
   - Confidence: high (vehicles, infantry, buildings); medium (aircraft)

4. **The building turret displacement does not turn** (`quirks.building_fire_point_shift_fixed`).
   For a building, *M* is the identity, so the move by *δ* is along map x (towards map east)
   whatever the turret's facing, and only the muzzle position is rotated. This differs from how a
   building turret is drawn (S-0105 claim 9).
   - Backing: `measurable` (scenario P3)
   - Confidence: medium

5. **Passengers of open-topped transports.** An infantry passenger firing from inside an
   open-topped transport fires from the transport's fire point computed with the transport's
   alternate muzzle position *n* − 1 in place of the weapon slot's, where *n* is the passenger's
   position (1, 2, …) in the transport's passenger order (the transport specs own that order).
   Claim 1 applies with the transport's own position, facings, turret displacement and burst
   counter. A passenger at position 6 or later fires from a muzzle position of (0, 0, 0) put through the
   same transform, so from the transport's turret pivot.
   - Backing: `measurable` (scenario P4)
   - Confidence: medium (the slot numbering and the transport's burst counter); low (the
     passenger order, which this spec does not own)

6. **Garrisoned buildings.** A garrisoned building fires from its base point shifted by its muzzle
   point *k* (F3, map x and y only; the world height is the base point's), where *k* is the
   building's *muzzle index*. This claim owns that index; S-0308 (claim 13) cites it. The index
   starts at 0, and after every shot it makes (a launch that succeeds, S-0301 claim 15) it becomes
   (*k* + 1) mod *n*, *n* being the occupant count then. Exception: a shot of a disc laser weapon
   (S-0301 claim 10) does not advance the index, so a garrison whose occupant weapon is a disc
   laser keeps the same muzzle point and the same occupant from shot to shot. It is set to 0 when
   the occupants leave or are killed. The same index picks which occupant's weapon fires (S-0308). A muzzle point that the art data does not give, or gives as (65535, 65535), is
   (0, 0).
   - Backing: `measurable` (scenario P5)
   - Confidence: high (the cycle and the reset when emptied); medium (the shift arithmetic, F3)

7. **Buildings with a primary fire pixel shift.** A building that is not garrisoned and whose type
   gives a primary fire pixel shift fires, with any weapon, from its base point shifted by that
   pixel shift (F3), at the base point's world height. If its type also has the dual-shift flag, the
   pixel shift is added to the fire point of claim 1 instead of to the base point. A pixel shift of
   (65535, 65535) counts as not given.
   - Backing: `measurable` (scenario P6)
   - Confidence: medium

8. **Buildings with a voxel turret or barrel.** A building that is not garrisoned and has no
   primary fire pixel shift fires as follows. With a voxel barrel, a further rule that follows the
   barrel gives its fire point (Open questions), whether or not its turret is a voxel turret. With a
   voxel turret and no voxel barrel, it fires from the fire point of claim 1 shifted by its turret
   anchor's pixel shift (F3, map x and y only). With neither, it fires from the fire point of
   claim 1. With claims 6 and 7, every building falls in exactly one row:

   | Building | Fire point |
   |---|---|
   | garrisoned | claim 6 |
   | not garrisoned; primary fire pixel shift given; no dual-shift flag | base point shifted by the pixel shift (F3), at the base point's world height (claim 7) |
   | not garrisoned; primary fire pixel shift given; dual-shift flag | fire point of claim 1 shifted by the pixel shift (F3) (claim 7) |
   | not garrisoned; no primary fire pixel shift; voxel barrel | the barrel rule (Open questions) |
   | not garrisoned; no primary fire pixel shift; no voxel barrel; voxel turret | fire point of claim 1 shifted by the turret anchor's pixel shift (F3, map x and y only) |
   | not garrisoned; no primary fire pixel shift; neither voxel barrel nor voxel turret | fire point of claim 1 |

   - Backing: `measurable` (scenario P6, with its part b)
   - Confidence: medium (voxel turret); low (voxel barrel)

9. **Spray attack.** Every shot of a spray-attack firer is aimed not at its target but at the cell
   that holds the point at one of eight fixed lepton shifts from the firer's position (F4). The
   shift number *s* is drawn as R(0, 7) from the scenario random stream for the first shot of a
   burst (burst counter 0), and for each later shot of the burst it advances by trunc(8 / *B*)
   modulo 8, *B* being the burst count. That cell becomes the shot's target; how the projectile
   uses it (its stored target point and speed follow the cell, while the launch still points at
   the firer's own target) is S-0306's (claim 3 a). No stock type has the flag.
   - Backing: `measurable` (scenario P7, test data)
   - Confidence: high (shifts, steps, and that the cell replaces the target)

## Formulas

**F1. Matrix composition** (claim 1). All matrices are 3 × 4 single-precision matrices, composed by
multiplying on the right as in S-0105 F2, each step's entries stored in single precision as S-0105
F0 states (truncated toward zero under `quirks.chop_rounding`, S-0205). Tr(*x*, *y*, *z*) is the
translation by (*x*, *y*, *z*) leptons in the model frame:

> *T* = *M* · Tr(*δ*, 0, 0) · Rz(*θ*) · Tr(*F*, *σL*, *H*), with *σ* = +1 for an even burst counter
> and −1 for an odd one,
> (*u*, *v*, *w*) = the translation column of *T* (that is, *T* applied to the origin),
> shift = (trunc *u*, trunc(−*v*), trunc *w*), integers in leptons.

*δ*, *F*, *σL* and *H* are converted from integers to single precision first (exact for magnitudes
below 2²⁴). With *M* the identity this is

> *u* = *δ* + *F* cos *θ* − *σL* sin *θ*, *v* = *F* sin *θ* + *σL* cos *θ*, *w* = *H*,

with cos and sin from S-0105 F1 and the rounding of F0 after each product and sum.

**F2. Angle** (claim 3). With *c* = −0.19634954084936207 (the double nearest −π/16) and *s*₅₃ as in
F3:

> *θ* = *s*₅₃(*s*₅₃((*a*₃₂ − 8) × *c*) − *s*₅₃((*b*₃₂ − 8) × *c*)) for a firer with a locomotor,
> *θ* = *s*₅₃((*a*₃₂ − 8) × *c*) for a building,

then narrowed to single precision before the table trigonometry.

**F3. Pixel shift to leptons** (claims 6 to 8). For a pixel pair (*p*, *q*), each first converted to
single precision:

> Δx = trunc(*s*₂₄(*s*₅₃(*s*₅₃(*β* × *q*) + *s*₅₃(*α* × *p*))))
> Δy = trunc(*s*₂₄(*s*₅₃(*s*₅₃(*β* × *q*) + *s*₅₃(−*α* × *p*))))

where *α* = 4.26669979095459 and *β* = 8.53339958190918 are the single-precision constants of the
original (near 256/60 and 512/60), *s*₅₃ is S-0205's RZ₅₃ for each double result and *s*₂₄ the
narrowing to single precision (round toward zero in both under `quirks.chop_rounding`). The world
height is unchanged. A pixel to the right on screen moves the point towards map east and map north
equally; a pixel down on screen moves it towards map east and map south.

**F4. Spray shifts** (claim 9). Leptons, map x and y; the world height is unchanged.

| *s* | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|---|
| Δx | 256 | 180 | 0 | −180 | −256 | −180 | 0 | 180 |
| Δy | 0 | 180 | 256 | 180 | 0 | −180 | −256 | −180 |

> *s*₁ = R(0, 7); *s*ⱼ₊₁ = (*s*ⱼ + trunc(8 / *B*)) mod 8 for the following shots of the burst.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Angle per facing step | −0.19634954084936207 | radians per 32-step facing | F2 (double, −π/16) |
| Pixel-to-lepton factor *α* | 4.26669979095459 | leptons per pixel | F3 (single) |
| Pixel-to-lepton factor *β* | 8.53339958190918 | leptons per pixel | F3 (single) |
| Alternate muzzle positions | 5 | positions | Claim 5 |
| Garrison muzzle points read | up to the type's maximum occupant count | points | Claim 6; see Edge cases |
| Garrison muzzle points the original supports | 10 | points per building type | Edge cases (departure above it) |
| "Not given" pixel pair | (65535, 65535) | pixels | Claims 6, 7 |
| Spray shift length | 256 straight, 180 diagonal | leptons | F4 |
| Spray directions | 8 | shifts | F4 |

## State and ordering

| State | Event | Result |
|---|---|---|
| garrisoned building, muzzle index *k* | a shot is launched, other than a disc laser shot | *k* ← (*k* + 1) mod occupant count |
| garrisoned building, muzzle index *k* | a disc laser shot is launched | *k* unchanged |
| garrisoned building | occupants leave or are killed | *k* ← 0 |
| spray-attack firer, burst counter 0 | a shot | *s* ← R(0, 7), one draw before the projectile's draws (S-0301 claim 18) |
| spray-attack firer, burst counter above 0 | a shot | *s* ← (*s* + trunc(8 / *B*)) mod 8, no draw |

A shot's fire point uses the firer's facings at the moment the shot is made (in the firer's own
share of the tick, S-0203) and the burst counter
that the shot uses (claim 2: the number of earlier shots in its burst).

## Edge cases and quirks

- **Fixed building turret displacement** (`quirks.building_fire_point_shift_fixed`, default on,
  claim 4). With the switch off, a building's turret displacement turns with its aiming angle:
  *T* = Rz(*θ*) · Tr(*δ*, 0, 0) · Tr(*F*, *σL*, *H*), composed and rounded as in F1.
- **The transport's burst counter steers a passenger's side** (claim 5). Original behaviour; no
  switch.
- **More than ten garrison muzzle points** (*departure*). The original supports at most ten
  garrison muzzle points per building type, but reads as many as the type's maximum occupant
  count. With a maximum occupant count above ten its result is unspecified: reading the extra
  points changes other art settings of the type. Our engine uses every muzzle point up to the
  maximum occupant count, and the importer warns for a maximum occupant count above ten.
- **Spray steps for long bursts.** A burst count above 8 gives a step of 0 (every shot of the burst
  at the same shift); a burst count of 0 or less is S-0301's (treated as 1, with a warning).

## Import mapping

| Key (file, section) | Concept | Form | Absent |
|---|---|---|---|
| `PrimaryFireFLH`, `SecondaryFireFLH`, `ElitePrimaryFireFLH`, `EliteSecondaryFireFLH` (art, type image) | muzzle positions (*F*, *L*, *H*) of the primary and secondary slots | three integers, leptons | 0, 0, 0 |
| `Weapon`*n*`FLH` and its elite form, the same key with `Elite` in front (art, type image) | muzzle positions of numbered weapons | three integers | 0, 0, 0 |
| `TurretOffset` (art, type image) | turret displacement *δ* | integer, leptons | 0 |
| `AlternateFLH` followed by 0 to 4 (art, transport image) | alternate muzzle positions | three integers | 0, 0, 0 |
| `MuzzleFlash` followed by 0, 1, … (art, building image) | garrison muzzle points | two integers, pixels | (0, 0) |
| `MaxNumberOccupants` (rules, building type) | how many garrison muzzle points are read | integer | 0 |
| `PrimaryFirePixelOffset` (art, building image) | primary fire pixel shift | two integers, pixels | not given |
| `PrimaryFireDualOffset` (art, building image) | dual-shift flag (claim 7) | boolean | no |
| `TurretAnimX`, `TurretAnimY` (art, building image) | turret anchor's pixel shift (claim 8) | integers, pixels | 0 |
| `TurretAnimIsVoxel`, `BarrelAnimIsVoxel` (art, building image) | voxel turret, voxel barrel | boolean | no |
| `SprayAttack` (rules, type) | spray-attack type | boolean | no |
| `Turret` (rules, type) | turret flag (claim 3) | boolean | no |

## Measurement scenarios

Stock data unless a test file is named; director and mods off; a small flat test map; any game
speed. The fire point is read as the start position of the projectile in the measurement setup's
object-state recorder (or, for instant weapons, from the start of the drawn beam, to the pixel).

**P1. Vehicle fire point.**
- Setup: a stock turreted vehicle with known muzzle position and turret displacement, on flat
  ground, body at facings 0, 16384, 32768 and 49152 (map north, east, south and west, in facing
  units), turret aimed at targets in four directions; a turret-less vehicle at the same body
  facings.
- Observable: projectile start positions relative to the vehicle's position.
- Expected: F1 with *D* the flat body rotation; the turret-less vehicle's shift rotates with the
  body only.
- Falsified if: any shift differs by more than 1 lepton from F1 (exact values with nearest
  rounding differ by at most 1).
- Runs: 1 (no random draw decides the observable).

**P2. Burst alternation.**
- Setup: a test rules file gives a stock vehicle with a non-zero lateral muzzle position a burst
  count of 2, then 3.
- Observable: start positions of consecutive shots.
- Expected: burst 2: +*L*, −*L*, +*L*, −*L* …; burst 3: +*L*, −*L*, +*L*, then +*L* again for the
  next burst.
- Falsified if: the sides do not follow the burst counter.
- Runs: 1 per burst count (no random draw decides the observable).

**P3. Building turret displacement.**
- Setup: a test art file gives a stock turreted defence a turret displacement of 512 leptons and a
  zero muzzle position; targets placed east, south, west and north of it.
- Observable: start positions.
- Expected: always 512 leptons towards map east of the foundation's top corner (at the building's
  height), whatever the target's direction.
- Falsified if: the shift turns with the turret.
- Part b: the same defence with a turret displacement of 0 and a muzzle position with non-zero
  forward and lateral parts (a test art file); a second, non-turreted defence given the same muzzle
  position. Expected: shifts as F1 with *M* the identity and *θ* from the building rows of claim
  3: the turreted one's from its turret facing, the other's from the direction of its target.
  Falsified if: any shift differs by more than 1 lepton from F1.
- Runs: 1 per target position (no random draw decides the observable).

**P4. Open-topped passengers.**
- Setup: a stock open-topped transport with five or six infantry passengers attacking a target.
- Observable: start positions of each passenger's shots.
- Expected: each passenger's shots start at the transport's fire point for its alternate muzzle
  position; a sixth passenger fires from the turret pivot.
- Falsified if: passengers fire from their own positions.
- Runs: 1 (no random draw decides the observable).

**P5. Garrison muzzles.**
- Setup: a stock garrisonable building with 1, 3 and its maximum number of occupants attacking a
  target.
- Observable: start positions and which occupant fired, shot by shot.
- Expected: the muzzle points cycle 0, 1, …, *n* − 1, 0, …; positions follow F3 from the
  foundation's top corner.
- Falsified if: the cycle skips or the positions differ from F3 by more than 1 lepton.
- Part b: repeat with an occupant type whose occupying weapon is a disc laser weapon (a test rules
  file). Expected: the index does not move, so every shot starts at the same muzzle point and
  comes from the same occupant. Falsified if the muzzle points or occupants cycle.
- Runs: 1 per occupant count (no random draw decides the observable).

**P6. Building pixel shifts.**
- Setup: a stock building with a primary fire pixel shift; a test art file adding the dual-shift
  flag; a stock voxel-turret defence.
- Observable: start positions.
- Expected: the stock building fires from its base point shifted by its primary fire pixel shift
  (F3), at the base point's world height; with the dual-shift flag, from the fire point of claim 1
  shifted by that pixel shift; the voxel-turret defence from the fire point of claim 1 shifted by
  its turret anchor's pixel shift (F3, map x and y only) (claims 7 and 8).
- Falsified if: the shifts are not applied as stated.
- Part b: a test art file marks the barrel of a voxel-turret defence with a non-zero turret anchor
  pixel shift as a voxel model. Expected: its shots do not start where the voxel-turret row of
  claim 8 puts them. Falsified if: they start at the fire point of claim 1 shifted by the turret
  anchor's pixel shift.
- Runs: 1 per building (no random draw decides the observable).

**P7. Spray attack.**
- Setup: a test rules file gives a stock vehicle the spray-attack flag and a weapon with burst
  count 4, attacking an enemy; the random-state recorder runs.
- Observable: the impact cells of each shot and the recorded draw.
- Expected: the first shot hits the cell at shift *s*₁ = the recorded draw; the next three at
  *s*₁ + 2, + 4, + 6 (mod 8).
- Falsified if: impacts follow the target, or the steps differ.
- Runs: 1 (the random-state recorder reads the draw).

## Open questions

- The fire point of a building whose barrel is a voxel model follows the barrel's elevation
  (claim 8); its rule is not yet worked out.
- The passenger order of an open-topped transport (claim 5) belongs to the transport specs.
- For infantry and aircraft, *D* is the locomotor's body transform; whether any of their
  locomotors adds a translation to *D* that would move the fire point (as a hover bob would) is not
  checked.
