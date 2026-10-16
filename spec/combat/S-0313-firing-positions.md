---
id: S-0313
title: Firing positions (where a shot starts, burst alternation, garrison muzzles, spray attack)
topic: combat
revision: 8
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-16
measurements: []
requests: [Q-0601, Q-0670]
supersedes: []
---

# S-0313: Firing positions (where a shot starts, burst alternation, garrison muzzles, spray attack)

## Summary

The world position at which a shot starts (its *fire point*), computed from the firer's position,
its weapon slot's muzzle position, its turret displacement and facings, and its burst counter; the
special fire points of garrisoned buildings, buildings with pixel muzzle shifts and passengers of
open-topped transports; and the redirected targets of spray-attack firers. The fire point is game
state: projectiles normally start there (S-0306 claim 8), so it changes flight times and what a projectile passes.
The placement exceptions are owned by S-0306 claim 5, including dropping projectiles, which
start at the firer's own position rather than its fire point. Disc lasers create no projectile
(S-0615 claim 12); their rings follow the current slot-0 fire point as S-0615 claim 15 specifies.
This spec covers skirmish; network
games and campaigns are out of scope.

## Terms

Glossary terms are used as defined there (*lepton*, *facing*, *direction*, *reference point*,
*foundation*, *top corner*, *world height*, *vehicle*, *infantry*, *building*, *aircraft*,
*locomotor*, *weapon*, *target*, *scenario random stream*). From S-0105: *table trigonometry* (F1),
*32-step facing* *f*₃₂, *body transform* *D* (claim 7), the rotation *Rz* and the right-multiplying
composition convention (F2), the *model frame* (+x forward, +y to the left, +z up). From S-0301:
*firer*, *shot*, *burst counter* *k*, *garrisoned building*. From S-0310: *aiming facing*. From
S-0202: R(*a*, *b*). In addition:

- **Fire point.** The world position, in leptons (x, y and world height), at which a shot's
  projectile normally starts, subject to S-0306 claim 5's placement exceptions.
- **Base point.** For a building, the top corner of its foundation, at the building's world height.
  For every other firer, its position (its reference point, with its world height).
- **Muzzle position (F, L, H).** Three integers in leptons that the art data gives per weapon slot of
  a type (primary, secondary, their elite versions and the numbered weapons): forward, lateral
  (positive to the left) and up. Which slot a shot uses is S-0308's.
- **Turret displacement** *δ*. The type's imported turret displacement, in leptons along the
  forward axis (the same field S-0105 uses for drawing).
- **Alternate muzzle positions.** Five extra muzzle positions (numbered 0 to 4) that a type's art
  data can give. Passengers of an open-topped transport fire from them (claim 5); they also
  anchor mind-control link lines (S-0601 claim 17).
- **Pixel shift.** A pair of imported integers in screen pixels, converted to a lepton shift
  by F3.
- **Muzzle point** *k* **of a garrisonable building.** The *k*-th pixel shift of the building type's
  list of garrison muzzle points (claim 6).
- **Primary fire pixel shift.** A pixel shift that a building type's art data can give for the
  shots of the building (claim 7).
- **Dual-shift flag.** A building type's imported flag that makes its primary fire pixel shift
  apply to the fire point of claim 1 instead of to the base point (claim 7).
- **A voxel turret, a voxel barrel.** A building's turret, or its barrel, that its type's rules data marks
  as a voxel model (claim 8).
- **Turret anchor's pixel shift.** A pixel shift that a building type's rules data gives for its
  turret (claim 8); the same imported pair places the drawn turret (S-0105 claim 9).
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
   - Backing: `measurable` (scenarios P1 and P2) for the stated test transforms,
     `unmeasured` for locomotor translations outside those tests: their complete reachable
     conditions have not been established (Open questions). Intermediate single-precision
     store placement not distinguished by these integer launch positions is also `unmeasured`:
     no separating launch-position case is established here.
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
   | vehicle with the turret flag (S-0310, claim 1) | its turret facing | F2, locomotor case |
   | vehicle without the turret flag (S-0310, claim 1) | its body facing | 0 |
   | infantry | its body facing, always | 0 |
   | aircraft | its turret facing, always (S-0310, claim 5) | F2, locomotor case |
   | turreted building (its type or an installed upgrade has the turret flag, S-0310 claim 4) | its turret facing | F2, building case |
   | other building, with a target | the direction of its current target from its reference point | F2, building case |
   | other building, with no target | its turret facing | F2, building case |

   F2 defines the exact angle arithmetic: in the locomotor case the two facing products are
   separately rounded as doubles before their difference is rounded as a double. The result
   is narrowed to single precision before the table trigonometry of S-0105 F1 is applied.
   Replacing this with a single product of the facing difference is not the specified arithmetic.
   When aiming and body facings agree, the locomotor case gives 0, including the two zero rows.
   - Backing: `measurable` (scenarios P1 and P3 parts b and c) for the reachable shots tested.
     The building's no-target row is `unmeasured`: no reachable shot without a target has
     been established to distinguish it. The aircraft's turret-versus-body distinction is
     `unmeasured` until P1 part b establishes an ordinary attack with those facings different.
     The exact double intermediate rounding in F2 is `unmeasured`: P1 and P3 compare integer
     launch positions, and no launch-position case separating algebraic rearrangements has
     been established.
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
   position (1, 2, …) in the transport's passenger order (S-0507 claim 3). Position 1 is
   the passenger that boarded most recently; a passenger whose exit fails goes back to
   position 1 (S-0507 claims 3 and 10). Thus alternate muzzle position 0 belongs to
   position 1, not to the first passenger that boarded.
   Claim 1 applies with the transport's own position, facings, turret displacement and burst
   counter. This is a transport-frame rule, not an independently switched defect: the passenger
   selects a muzzle, but its own facing, turret displacement and burst counter do not change
   that muzzle's position on the transport. In particular, two passengers firing at the same
   transport burst parity use the same lateral sign even if their own burst parities differ.
   A passenger at position 6 or later fires from a muzzle position of (0, 0, 0) put through the
   same transform, so from the transport's turret pivot.
   These five alternate positions are not exclusive to transport passengers: the same imported
   triples also anchor a controller's mind-control link lines. S-0601 claim 17 owns those
   lines and their anchor selection; passenger order does not select a control-link anchor.
   - Backing: `measurable` (scenario P4 for passengers, P9 for shared link-anchor data)
   - Confidence: medium (the slot numbering and the transport's burst counter); high (the
     passenger order, owned by S-0507 claim 3, and shared control-link anchor data)

6. **Garrisoned buildings.** A garrisoned building fires from its base point shifted by its muzzle
   point *k* (F3, map x and y only; the world height is the base point's), where *k* is the
   building's *muzzle index*. This claim owns that index; S-0308 (claim 13) cites it. The index
   starts at 0, and after every shot it makes (a launch that succeeds, S-0301 claim 15) it becomes
   (*k* + 1) mod *n*, *n* being the occupant count then. Exception: a shot of a disc laser weapon
   (S-0615 claim 12) does not advance the index, so a garrison whose occupant weapon is a disc
   laser keeps the same muzzle point and the same occupant from shot to shot. After a completed
   whole-building eviction or clearing action, the index is 0. Reoccupation therefore starts at 0.
   An additional entry does not change the index (S-0508 claim 18).
   Only this completed-action result is required, not a reset before the first individual
   release or removal (Q-0670). S-0308 claim 13, S-0311 claim 15 and S-0508 claim 18 agree.
   No reachable shot or kill credited to the building during the action is established that
   distinguishes intermediate index values; no such intermediate reading has a specified
   result. Evacuation's release choices and assault's occupant-animation positions do not
   use this firing index. For assault, occupant list position *i*, counted from 0, uses muzzle
   point *i* without wrapping by the number of configured points (S-0508 claim 14). Supported
   assault point numbers are 0 through 9; a missing point keeps its imported default. No safe
   original animation position is specified for position 10 or later.
   Assault's reversed kill credit names the individual occupant,
   not a firing occupant selected through the building (S-0508 claims 10, 11 and 14,
   and S-0311 claim 9). These effects therefore impose no reset-before-release requirement.
   S-0311 claim 15 describes the limited individual set-up removal case that retains the
   numeric index without reducing it modulo the remaining count in the original. Separately,
   defeat or surrender of an occupant's own house can kill it inside a building belonging to
   a surviving house (S-0508 claim 19). Only the defeated house's occupants are hit, so
   differently owned occupants can survive in a mixed-house garrison.

   **Original retained places.** These deaths retain both the numeric index and the occupant
   places: *n* still includes dead places, without compaction, renumbering or index repair
   (S-0508 claim 20). The original supplies no next-living substitution. The numeric muzzle
   calculation still selects point *k* independently of occupant health, but does not guarantee
   a safe launch, successful-shot advancement or experience selection through a dead choice.
   These unsafe continuations have no dependable original result specified here.

   **Engine safety departure.** S-0508 claim 23 replaces that retention, without a quirk switch.
   Any occupant death inside removes its place in the same update, retaining the survivors'
   relative order and numbering them from 0. With *n* the new count, retain numeric *k* when
   *k* < *n*; otherwise set it to 0, including when *n* = 0. This is an integer range check,
   not reduction modulo *n*. With *n* > 0, the repaired index selects both the surviving
   occupant's weapon (S-0308 claim 13) and muzzle point. A successful non-disc-laser shot
   advances that index by (*k* + 1) mod *n*, using the post-removal count; disc laser shots
   still leave it unchanged. No shot uses a removed place. With *n* = 0 there is no occupant
   weapon or garrison muzzle selection, and civilian abandonment follows S-0508 claim 8,
   subject to S-0508 claim 24. Experience selection remains S-0311 claim 8's rule among the
   remaining occupants, not a new muzzle-selection fallback.
   A muzzle point that
   the art data does not give, or gives as (65535, 65535), is (0, 0).
   - Backing: `measurable` (scenario P5) for the cycle, completed-action reset, reoccupation,
     additional entry and muzzle shifts. Intermediate reset timing is `unmeasured`: no
     validated fixture interleaves a shot or building-credited destruction within the action.
     It is not an implementation constraint. The index-independent release and assault
     effects inherit their cited claims' backing; P5's clearing variant checks the assault
     distinction without inventing an interleaved shot. The limited set-up removal remains `unmeasured`
     for the fixture limitation in S-0311 claim 15: no validated data-only selective-removal
     scenario is supplied. Defeat hits and numeric-index retention are `measurable` (P5's
     defeat variant). Mixed-house selective survival is `unmeasured`: no validated fixture
     supplies differently owned entrants. Safe continuation through a dead firing choice is
     `unmeasured`: no safe result is established or unsafe confirming attack proposed.
     Engine death removal, repaired-index muzzle selection and post-removal advancement are
     `measurable` on our engine (P5's engine-only variant and S-0508 claim 23); these are design
     requirements, not predictions for the original.
   - Confidence: high (the cycle, action reset, additional entry, engine safety rule and defeat's off-map hits),
     medium (the shift arithmetic, F3, limited set-up removal and defeat index retention)

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

   - Backing: `measurable` (scenario P6) for the rows without a voxel barrel,
     `unmeasured` for the barrel-following description: its transform is unresolved, so no
     predicted start position or distinguishing scenario is established. The barrel row is
     an explicit gap, not a complete fire-point rule.
   - Confidence: medium (voxel turret); low (voxel barrel)

9. **Spray attack.** Every shot of a spray-attack firer redirects its shot target to the cell
   that holds the point at one of eight fixed lepton shifts from the firer's position (F4). The
   shift number *s* is drawn as R(0, 7) from the scenario random stream for the first shot of a
   burst (burst counter 0), and for each later shot of the burst it advances by trunc(8 / *B*)
   modulo 8, *B* being the burst count. That cell becomes the shot's target; how the projectile
   uses it (its stored target point and speed follow the cell, while the launch still points at
   the firer's own target) is S-0306's (claim 3 a).
   - Backing: `measurable` (scenario P7, test data)
   - Confidence: high (shifts, steps, and that the cell replaces the target)

10. **Building turret settings have one rules source.** The turret anchor's pixel shift and
    the voxel-turret and voxel-barrel flags come only from the building type's rules section,
    through building-type rules import. The art section named by its image does not supply,
    override or provide a fallback for these settings, even when the image name differs from
    the type name. Rules layers apply as in S-0006 claim 1, keeping the current setting when
    a later layer omits a key. The anchor pair used for firing in claim 8 is the same imported
    pair used for aiming in S-0310 claim 4. The voxel-turret flag used in claim 8 is likewise
    the flag used for that aiming tolerance. Neither use imports a separate art value.
    The voxel-barrel flag selects claim 8's barrel row from the same rules source, although
    that row's exact fire-point transform remains unresolved.
    - Backing: `measurable` (scenario P8)
    - Confidence: high

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

Each subtraction of 8 is exact integer arithmetic. The products and the locomotor-case
difference use double precision with *s*₅₃ rounding at the places shown. The resulting *θ*,
in radians, is narrowed by *s*₂₄ (F3) before the table trigonometry. Thus both double arithmetic
and the final single-precision narrowing follow S-0205 claim 5's rounding mode.

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
The shifts and direction indices are integers. For positive integer burst count *B*, divide 8
by *B* with truncation toward zero; the direction sum wraps modulo 8 with no further rounding.

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
| garrisoned building | a whole-building eviction or clearing action completes | *k* = 0; reoccupation starts at muzzle 0 |
| garrisoned building | an additional occupant enters (S-0508 claim 18) | *k* unchanged |
| original garrisoned building belonging to a surviving house | an occupant's own house is defeated or surrenders (S-0508 claims 19 and 20) | killing hits retain dead places in *n* and numeric *k* unchanged; no safe later launch is promised |
| garrisoned building in our engine | any occupant dies inside (S-0508 claim 23) | remove its place in the same update, retain survivor order; keep *k* when less than new *n*, otherwise set 0; later muzzle selection and advancement use repaired *k* and new *n* |
| spray-attack firer, burst counter 0 | a shot | *s* ← R(0, 7), one draw before the projectile's draws (S-0301 claim 18) |
| spray-attack firer, burst counter above 0 | a shot | *s* ← (*s* + trunc(8 / *B*)) mod 8, no draw |

A shot's fire point uses the firer's facings at the moment the shot is made (in the firer's own
share of the tick, S-0203) and the burst counter
that the shot uses (claim 2: the number of earlier shots in its burst).

## Edge cases and quirks

- **Fixed building turret displacement** (`quirks.building_fire_point_shift_fixed`, default on,
  claim 4). With the switch off, a building's turret displacement turns with its aiming angle:
  *T* = Rz(*θ*) · Tr(*δ*, 0, 0) · Tr(*F*, *σL*, *H*), composed and rounded as in F1.
- **More than ten garrison muzzle points** (*departure*). The original supports at most ten
  garrison muzzle points per building type, but reads as many as the type's maximum occupant
  count. With a maximum occupant count above ten its result is unspecified: reading the extra
  points changes other art settings of the type. Our engine uses every muzzle point up to the
  maximum occupant count, and the importer warns for a maximum occupant count above ten.
  Backing: `unmeasured` for the original's unsupported excess entries: no stable observable
  result is predicted for them. The deterministic extension and warning are design requirements.
- **Spray steps for long bursts.** A burst count above 8 gives a step of 0 (every shot of the burst
  at the same shift); a burst count of 0 or less is S-0301's (treated as 1, with a warning).

## Import mapping

| Key (file, section) | Concept | Form | Absent |
|---|---|---|---|
| `PrimaryFireFLH`, `SecondaryFireFLH`, `ElitePrimaryFireFLH`, `EliteSecondaryFireFLH` (art, type image) | muzzle positions (*F*, *L*, *H*) of the primary and secondary slots | three integers, leptons | 0, 0, 0 |
| `Weapon`*n*`FLH` and its elite form, the same key with `Elite` in front (art, type image) | muzzle positions of numbered weapons | three integers | 0, 0, 0 |
| `TurretOffset` (art, type image) | turret displacement *δ* | integer, leptons | 0 |
| `AlternateFLH` followed by 0 to 4 (art, type image) | alternate muzzle positions, also used as mind-control link anchors (S-0601 claim 17) | three integers, leptons | 0, 0, 0 |
| `MuzzleFlash` followed by 0, 1, … (art, building image) | garrison muzzle points | two integers, pixels | (0, 0) |
| `MaxNumberOccupants` (rules, building type) | how many garrison muzzle points are read | integer | 0 |
| No user-data field | shared occupant/muzzle index and completed-action reset (claim 6; Q-0670) | runtime integer, not imported | initially 0; no imported intermediate-reset setting |
| `PrimaryFirePixelOffset` (art, building image) | primary fire pixel shift | two integers, pixels | not given |
| `PrimaryFireDualOffset` (art, building image) | dual-shift flag (claim 7) | boolean | no |
| `TurretAnimX` (rules, section named by building type) | horizontal component of turret anchor's pixel shift (claims 8 and 10; also S-0310 claim 4) | integer, pixels | 0 |
| `TurretAnimY` (rules, section named by building type) | vertical component of turret anchor's pixel shift (claims 8 and 10; also S-0310 claim 4) | integer, pixels | 0 |
| `TurretAnimIsVoxel` (rules, section named by building type) | voxel turret (claims 8 and 10; also S-0310 claim 4) | boolean | no |
| `BarrelAnimIsVoxel` (rules, section named by building type) | voxel barrel (claims 8 and 10) | boolean | no |
| `SprayAttack` (rules, type) | spray-attack type | boolean | no |
| `Turret` (rules, type) | turret flag (claim 3) | boolean | no |

All four turret-animation keys above are read by building-type rules import from each
applicable rules layer, not by image-art import. A missing key retains its current value,
initially the Absent value above. Entries with these names in art, whether under the building's
type name or its image name, have no effect (claim 10). The primary fire pixel shift and
its dual-shift flag remain art settings, not members of this four-key rules group.

## Measurement scenarios

Use a small purpose-made skirmish map with a flat observed firing area, one human-controlled
house owning the firers and one bystander computer player kept away as in the glossary. Place
neutral durable targets in the observed area. Use game speed index 2 and 800 × 600 pixels,
director, unrelated mods and engine add-ons off. Stock data applies except the named test rules
and art files of each scenario, removed after that scenario.

Procedure for every part: position the firer, load passengers or occupants where named, order an
attack through ordinary input, and record complete bursts after the required facings settle.
Where a part needs a changing facing, record that facing at each shot rather than inferring it
from the order. A read-only object-state recorder logs ticks, firer and projectile positions,
facings, burst counters, selected weapon slot and passenger or occupant identities. Compare the
projectile's launch position before its first movement with F1 or F3 in integer leptons. A beam
image can corroborate placement but cannot establish lepton-exact arithmetic. For disc lasers
record the effect's origin. A missing recorder capability is a proposed harness addition for
the user, not a request to alter the game.

**P1. Vehicle fire point.**
- Setup: a stock turreted vehicle with known muzzle position and turret displacement, on flat
  ground, body at facings 0, 16384, 32768 and 49152 (map north, east, south and west, in facing
  units), turret aimed at targets in four directions; a turret-less vehicle at the same body
  facings.
- Observable: projectile start positions relative to the vehicle's position.
- Expected: F1 with *D* the flat body rotation; the turret-less vehicle's shift rotates with the
  body only.
- Falsified if: any recorded integer launch shift differs from F1; a one-lepton difference is
  significant and must not be hidden by a tolerance.
- Part b: named test art gives distinct non-zero forward, lateral and height muzzle components
  and a non-zero turret displacement. Repeat across all 32 body and aiming steps reachable by
  ordinary attacks, including steps around the facing wrap. Compute the predictions with F2's
  separately rounded facing products and final single-precision narrowing, not a simplified
  facing-difference product. Repeat with infantry and aircraft
  (named test rules may give the aircraft an ordinary visible projectile). Record both body and
  turret facings. Expected: claim 3's infantry row has no relative rotation, while the aircraft
  row uses turret minus body facing even without a turret flag. Falsified by any F1 mismatch or
  by aircraft starts following the body instead of the recorded turret facing when they differ.
  If the ordinary attack cannot separate the aircraft facings, that case is inconclusive.
- Part c: repeat with a driving vehicle stationary on each reachable slope orientation of the
  test map, recording its settled body transform (S-0106). Expected: F1 tilts the muzzle and
  turret displacement with that transform. Falsified by any different integer start position.
- Runs: one full burst per facing/transform combination; no random draw decides the start shift.

**P2. Burst alternation.**
- Setup: named test art gives a stock vehicle a non-zero lateral muzzle position; named test
  rules give its weapon a burst count of 1, then 2, then 3.
- Observable: start positions of consecutive shots.
- Expected: burst 1 always uses +*L*; burst 2: +*L*, −*L*, +*L*, −*L* …; burst 3:
  +*L*, −*L*, +*L*, then +*L* again for the next burst.
- Falsified if: the sides do not follow the burst counter.
- Runs: 1 per burst count (no random draw decides the observable).

**P3. Building turret displacement.**
- Setup: a test art file gives a stock turreted defence a turret displacement of 512 leptons and a
  zero muzzle position; targets placed east, south, west and north of it. Omit its primary fire
  pixel shift and, in named test rules, disable voxel turret and barrel flags to isolate F1.
- Observable: start positions.
- Expected: always 512 leptons towards map east of the foundation's top corner (at the building's
  height), whatever the target's direction.
- Falsified if: the shift turns with the turret.
- Part b: the same defence with a turret displacement of 0 and a muzzle position with non-zero
  forward and lateral parts (a test art file); a second, non-turreted defence given the same muzzle
  position. Expected: shifts as F1 with *M* the identity and *θ* from the building rows of claim
  3: the turreted one's from its turret facing, the other's from the direction of its target.
  Falsified if: any integer shift differs from F1.
- Part c: named test rules give an upgrade the turret flag and install it on an otherwise
  non-turreted test building. Keep primary pixel shifts and voxel barrels absent. Expected:
  claim 3's turreted-building row follows the recorded turret facing. Falsified if the target
  direction is used instead when the recorded facings differ.
- Runs: 1 per target position (no random draw decides the observable).

**P4. Open-topped passengers.**
- Setup: a stock open-topped transport and distinguishable infantry passengers. Named test
  rules give it room for six passengers; named test art gives its five alternate muzzle positions
  distinct non-zero forward, lateral and height components and gives it a non-zero turret
  displacement. Board passengers one at a time, recording identities and boarding order, then
  order an attack.
- Observable: start positions of each passenger's shots and its current passenger position.
- Expected: each passenger's shots start at the transport's fire point for its alternate muzzle
  position. The last passenger boarded is at position 1 and uses alternate muzzle position 0.
  Earlier passengers follow in reverse boarding order (S-0507 claim 3). A passenger at
  position 6 fires from the turret pivot.
- Failed-exit variant: block exits during unloading as in S-0507 claim 10, then resume attacking
  without anyone leaving. Expected: the passenger whose exit failed is still at position 1 and
  uses alternate muzzle position 0.
- Counter variant: named test rules give the transport its own burst weapon and give a passenger
  a burst of at least two shots. Record attacks during which the transport's burst parity is
  different from the passenger's. Expected: the passenger's muzzle uses the transport's parity,
  displacement and facings in F1. Repeat with the transport not firing, its counter at zero:
  every passenger shot uses +*L*, including the passenger's odd-numbered counter. Falsified if
  the passenger's counter negates the lateral component or any integer start differs from F1.
  If no differing-parity shot occurs, repeat; equal parities alone are inconclusive.
- Falsified if: passengers fire from their own positions, their muzzle assignment follows
  first-boarded-first order, the failed-exit passenger loses position 1, or a sixth passenger
  does not fire from the recorded turret pivot.
- Runs: one completed boarding/attack/unload sequence per variant, with at least two full
  passenger bursts and one differing-parity shot in the counter variant.

**P5. Garrison muzzles.**
- Setup: a stock garrisonable building with 1, 3 and its maximum number of occupants attacking a
  target.
- Observable: start positions and which occupant fired, shot by shot.
- Expected: the muzzle points cycle 0, 1, …, *n* − 1, 0, …; positions follow F3 from the
  foundation's top corner.
- Falsified if: the cycle skips or any integer start position differs from F3.
- Evacuation and reoccupation variant: use a civilian garrisonable building with free exits
  and three distinguishable occupants entered one at a time. Fire exactly one ordinary shot,
  then stop the attack through normal input. Verify a non-zero index and give the building a
  deploy order to evacuate it (S-0508 claim 13). Record index 0 after completion, reoccupy in
  the original order, and attack again. Expected: the first shot after reoccupation uses the
  first occupant and muzzle 0. A non-zero completed-action index, a different first shooter
  or a different muzzle falsifies the corresponding rule. Void and repeat a run if another
  shot intervenes before evacuation.
- Additional-entry variant: begin with two distinguishable occupants, fire once, stop and admit
  a third without another intervening shot. Expected: the index stays 1 and the next shot uses
  the second occupant at muzzle 1. A different index, shooter or muzzle falsifies retention.
  Void and repeat a run if another shot intervenes before entry.
- Clearing variant: named test map `firing-index-assault.map` gives a stock infantry type the
  assaulter and engineer capabilities but not the occupier capability (S-0508 claim 14).
  Enter three distinguishable ordinary occupants owned by the human house into a civilian
  building. A timed building-only ownership-change trigger transfers that building to a
  hostile house, leaving the occupants inside under their original owner (S-0508 claim 17).
  Give their garrison weapon harmless test damage so it cannot kill the entrant. Bring a
  human-owned durable vehicle into range for the hostile building to acquire normally. Let
  the building fire one shot at it, then move that target out of range through ordinary input.
  Verify a non-zero index and no further shot before clearing. Order the human-owned test
  assaulter to capture the hostile building through ordinary input. Verify that arrival clears
  the occupants without consuming the assaulter. Record the completed-action index, then
  reoccupy with three distinguishable replacements in the original entry order after the
  building returns to civilian ownership. Expected: completed-action index 0, first shooter
  the first replacement and first muzzle 0. A different index, shooter or muzzle falsifies
  the clearing-reset rule. A run with an intervening shot or no completed assault is
  inconclusive for reset; no weapon that kills occupants through the building is assumed.
- Repeat evacuation, additional entry and clearing in three fresh games each. These observations
  test completed actions, not the index between individual releases or removals within an action.
  In the clearing variant also record assault credit and occupant-animation positions.
  Expected: credit names the occupants under S-0508 claim 14, not an occupant selected by
  the building's pre-action firing index. Their animation positions are not selected by that
  index: for three occupants the animation points are 2, 1, 0 in reverse list order
  (S-0508 claim 14). Repeat with one and two occupants, expecting points 0 and 1, 0.
  Named test art gives points 0, 1 and 2 distinct small pixel pairs. Repeat with point 2
  omitted: position 2 uses the absent default (0, 0), not point 0's configured nonzero pair.
  Compare integer positions using F3; record event-time credit/removal as in S-0508 claim 14's
  scenario G17 if available. A point selected through the building's firing index, wrapping
  by configured-point count, or credit selected through the firing index falsifies this
  distinction. Stay within the supported assault point numbers. No shot or credited
  destruction is artificially inserted between removals: no validated ordinary-input fixture
  reaches such an intermediate reading (Q-0670).
- Defeat variant: in a named `firing-index-defeat.map`, add a third isolated living contender.
  The human house enters three identifiable occupants, fires exactly once and stops at index 1.
  A timed building-only ownership trigger transfers the building to a surviving hostile house,
  leaving the occupants under their original owner. Verify those owners and index 1, keep the
  building from firing again, and surrender the human house through ordinary input. Record
  the surrender-processing tick, building health/owner, occupant health and numeric index
  read-only, before any later firing uses a dead choice. Expected: the occupants receive killing
  hits despite being off-map, the foreign-owned building survives without eviction, and the
  index remains 1. Exclusion of the hits, eviction solely from the occupants' defeat, or an
  index reset falsifies that part. Repeat three fresh games; void a run with an intervening
  shot or whole-building action. This tests S-0508 claim 19's numeric retention, not a safe
  later muzzle or a validated mixed-house selective-survival fixture. Also record retained
  occupant count: the original still has three places, including the dead ones (S-0508 claim 20).
  A count decrement falsifies original retention. Do not resume attacks through the dead
  choices as a confirming test.
- Engine-only death-removal variant: use the same defeat fixture on our engine, expecting count
  0 and index 0 in the death update and no occupant shot thereafter. Also use S-0508 claim 23's
  scenario G18 with identified occupants U, V, W in entry order. At index 1 kill V inside:
  expect order U, W, count 2, index 1, then W's weapon at muzzle 1 on its next successful
  non-disc-laser shot, advancing the index to 0. Start separately at index 2 and kill V,
  then repeat with W killed instead: expect index 0, not modulo reduction to 1. Record death
  tick, count, survivor order, index, shooter and integer launch position; compare F3.
  Repeat each case in three fresh runs. Retained dead places, reordered survivors, a different
  repair, a removed-place shot, or advancement using the old count falsifies this engine rule.
  These deterministic engine fixtures are not an original mixed-house setup or a proposal to
  continue unsafe original firing.
- Data variant: named test art supplies distinct muzzle pixel pairs, including a missing entry,
  the not-given pair from Constants, and positive and negative unequal components. Expected:
  missing and not-given entries use (0, 0); others give exactly F3 at unchanged world height.
  Repeat with a building on raised flat terrain. Falsified by any different start or height.
- Part b: repeat with an occupant type whose occupying weapon is a disc laser weapon (a test rules
  file). Expected: the index does not move, so every shot starts at the same muzzle point and
  comes from the same occupant. Falsified if the muzzle points or occupants cycle.
- Runs: one run per occupant count and variant, recording at least two complete muzzle cycles
  (or two disc-laser shots). No random draw decides the predicted muzzle.

**P6. Building pixel shifts.**
- Setup: named test rules and art create building variants with primary fire pixel shift
  present/absent, dual-shift flag on/off, voxel turret on/off, and no voxel barrel. Give distinct
  muzzle and turret displacement values and a non-zero turret-anchor pair. Set the anchor and
  voxel flags in rules; put conflicting anchor/flag values in art as a negative control.
- Observable: start positions.
- Expected: the primary-shift variant fires from its base point shifted by its primary fire pixel shift
  (F3), at the base point's world height; with the dual-shift flag, from the fire point of claim 1
  shifted by that pixel shift; the no-primary voxel-turret variant from the fire point of claim 1 shifted by
  its turret anchor's pixel shift (F3, map x and y only) (claims 7 and 8).
- Falsified if: any integer shift differs from claims 7 and 8, an art-only anchor/voxel-flag
  change affects the result, or a rules-anchor change does not affect the voxel-turret result.
- Part b: for the primary-shift variants, fire with primary and secondary weapons, then their
  elite versions, using named test weapons with different muzzle triples. Give the primary
  pixel pair positive and negative unequal components in turn, then omit it, then use the
  not-given pair. Expected: with the dual-shift flag off, all weapon slots start at base + F3
  and ignore F1; with it on, each starts at its own F1 + F3. Absent and not-given select the
  appropriate no-primary row of claim 8. Repeat on raised flat terrain: the F3 addition never
  changes world height. Falsified by any slot-dependent shift in the first case, any ignored
  F1 shift in the second, or any non-zero F3 height change.
- Runs: one full burst per building/data/weapon variant. No barrel-position prediction is made
  while that rule remains an open question.

**P7. Spray attack.**
- Setup: named test rules give a stock vehicle the spray-attack flag and a weapon with burst
  count 4, then 3, then 9, attacking a target away from the eight shifted cells. Use a visible
  inaccurate arcing projectile in a separate draw-order variant; the random-state recorder runs.
- Observable: the projectile's stored target cell and target point, not its impact cell, plus
  firer position, burst counters and all scenario draws. Impacts alone are not evidence of the
  redirected target because launch aim can still follow the firer's target (S-0306 claim 3 a).
- Expected: the first shot's cell contains the firer's position plus F4 shift *s*₁ = R(0, 7).
  Burst 4 advances by 2 per shot, burst 3 by 2, and burst 9 by 0; all wrap modulo 8. The next
  burst takes a new ranged draw. The recorded random stream contains exactly one spray ranged draw per burst, before any projectile scatter draws, and no spray draw for later shots in that burst. Count underlying random draws as S-0202 claim 4 specifies for a ranged draw. Other launch draws follow S-0301 claim 18.
- Falsified if: a stored target cell differs from F4, later shots consume a spray draw, a new
  burst reuses its previous shift without a draw, or spray selection follows projectile draws.
- Runs: record at least two full bursts per burst count, and repeat until the recorded first
  shifts cover all eight directions. Use recorded values, not a chosen seed; report any
  directions not yet observed as incomplete coverage.

**P8. Rules source versus art source (Q-0601).**
- Setup: use the common skirmish setup and named turret-source test rules and art files.
  Give a turreted building a distinct image name, no primary fire pixel shift, and valid
  turret and barrel models. Test each of the four settings independently: a nonzero
  horizontal anchor, a nonzero vertical anchor, the voxel-turret flag, and the voxel-barrel
  flag. For each setting use rules-only, art-only, conflicting rules/art, and omitted cases.
  Include both the art section named by the image and the art section named by the type.
- Procedure: start a fresh game for each case and order attacks on close neutral targets.
  The read-only recorder logs the building type's effective anchor pair and both voxel
  flags, turret facings, and projectile launch positions. For anchor and voxel-turret cases,
  omit the voxel-barrel flag so claim 8's fully specified rows apply. Repeat the anchor
  cases with and without a primary fire pixel shift to check S-0310 claim 4's precedence.
  A further pair of test maps overrides each setting in rules, then omits it in an otherwise
  present building-type section, to test retention of lower-layer settings.
- Observable: the four effective imported settings, plus launch positions and the aiming
  direction at the shot. The imported-settings recorder is a proposed harness addition.
- Expected: only the rules type section supplies the four settings. Conflicting art loses,
  art-only equals omitted, and changing the image name changes none of these four settings.
  Rules overrides replace lower-layer values, and omission retains them. Launch positions
  follow claim 8's non-barrel rows and aiming follows S-0310 claim 4 using the same anchor
  pair and voxel-turret flag. For the barrel flag, compare the recorded effective flag only:
  this scenario does not predict the unresolved barrel transform.
- Falsified if: any effective setting follows art, an art-only value replaces the absent
  default, or firing and aiming use different imported anchor pairs or voxel-turret flags.
- Runs: two fresh games per file/setting variant, recording at least one full burst each.

**P9. Shared alternate positions for control links.**
- Setup: use the common skirmish setup with a multi-victim controller and six capturable
  neutral vehicles. Named link-anchor test art gives its five alternate muzzle positions
  distinct invented triples, different from its primary weapon's muzzle position.
- Procedure: acquire the six victims through ordinary attacks, select the controller, and
  capture the visible lines. Repeat with only the five alternate triples changed.
- Observable: each line's starting pixel relative to the controller, with its position,
  facings and burst counter recorded read-only.
- Expected: the line starts follow the alternate triples transformed by claim 1, with anchor
  selection as in S-0601 claim 17. Changing the alternate triples moves the corresponding
  starts even though the primary weapon muzzle is unchanged.
- Falsified if: link starts ignore the alternate triples or instead follow the primary muzzle.
- Runs: one six-victim acquisition per art variant, at unchanged settled facings.

## Open questions

- The fire point of a building whose barrel is a voxel model follows the barrel's elevation
  (claim 8); its rule is not yet worked out.
- For infantry and aircraft, *D* is the locomotor's body transform; whether any of their
  locomotors adds a translation to *D* that would move the fire point (as a hover bob would) is not
  checked.
