---
id: S-0106
title: Slope shapes, terrain surface height and voxel tilt on slopes
topic: rendering
revision: 8
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0106: Slope shapes, terrain surface height and voxel tilt on slopes

## Summary

The 21 slope shapes a cell can have, the height of the terrain surface at any point of a cell, and
how a voxel-drawn unit is tilted to match the slope it stands on: the fixed tilt of each shape, the
three-tick transition when the shape under a unit changes, which locomotors tilt the body, and the
tilt of voxel shadows. Together with S-0105 this fixes the body transform *D* of a voxel unit
on the ground layer when the unit is not rocking. Rocking, and the causes of pitch and roll, belong to the vehicle
specs. This spec covers skirmish; network games and campaigns are out of scope.

## Terms

*Cell*, *lepton*, *height level*, *world height*, *slope*, *tick*, *drawn frame*, *facing*,
*locomotor*, *voxel model*, *quirk switch*, *departure*, *importer warning* and *round-toward-zero
mode* are used as in the glossary. *Body transform D*, *model frame*, *model unit*, *pixels per
lepton ρ*, *table trigonometry*, *table square root*, *fresh draw*, *image cache*, *ramp state*
(claim 26 of S-0105: which settled tilt a body has, as far as the image cache is concerned) and
*settled* are S-0105's terms. The terms below are specific to this spec.

- **In-cell position (u, v).** For a world point (X, Y) in leptons, u = X mod 256 and v = Y mod 256,
  integers 0 … 255. (0, 0) is the cell's top corner, u grows towards map east and v towards map south.
- **Corners of a cell.** Named by where they appear on the screen: the *top* corner (u, v) = (0, 0)
  (the glossary's top corner); the *right* corner (256, 0); the *bottom* corner (256, 256); the
  *left* corner (0, 256). A corner "raised by k" lies k height levels above the cell's height level.
- **Slope shape** *s*. An integer 0 … 20 that every cell carries. 0 is flat. The cell's terrain tile
  gives it (S-0010 claim 6; unusable-tile cases and their overlay exception are
  S-0010 claims 7 and 8); this spec says what each value means. Shapes are grouped as:
  - *edge slopes* 1 … 4: two neighbouring corners raised by 1,
  - *corner slopes* 5 … 8: one corner raised by 1,
  - *valley slopes* 9 … 12: three corners raised by 1 (one corner low),
  - *steep slopes* 13 … 16: one corner raised by 2, the corner opposite it at 0, the other two
    raised by 1,
  - *saddle shapes* 17 … 20: the four remaining shapes. Their corner pattern is not established by
    this spec (Open questions); their surface height and tilt are given in claims 2 and 4.
- **Level height in model units** *η* = 15 / sin 60° = 10√3 ≈ 17.32 model units: the height in
  the model frame that S-0105's view (F2, nominal tilt 60°) projects to the 15 screen pixels by
  which S-0101 claim 4 lifts terrain per height level. A cell edge is 256*ρ* ≈ 42.43 model units
  (S-0105 F3), which the view projects to the step of 30 pixels across and 15 pixels down that one
  cell makes on the screen (S-0101 claim 1). These two numbers give the angle of a slope *as drawn*: arctan(*η* / 256*ρ*) = arctan(1/√6) ≈ 22.21°
  across an edge slope, and arctan(*η* / 128√2·*ρ*) = arctan(1/√3) = 30° along the diagonal of a
  corner, valley or steep slope (rising one level over half a diagonal, or two over a whole one).
- **Uphill direction.** For shapes 1 … 16, the map direction in which the surface rises (Constants,
  table C1).
- **Surface height** *H*(*x*, *y*, *u*, *v*). The world height, in leptons, of the terrain surface at
  in-cell position (u, v) of cell (x, y) (claim 2).
- **Tilt transform** *R*<sub>s</sub>. A fixed 3 × 3 rotation for each slope shape (claim 4), in
  the model frame of S-0105 (model +x = map east, +y = map north, +z = up).
- **Tilt orientation** *q*<sub>s</sub>. A fixed, approximately unit quaternion (x, y, z, w) for each slope shape,
  used only while a tilt changes (claim 7, Constants C3's construction rule). It describes nearly, but not exactly,
  the same rotation as *R*<sub>s</sub>.
- **Tilt state** of a unit with a tilting locomotor: its *current shape* *s*<sub>c</sub>, its
  *previous shape* *s*<sub>p</sub>, and the tick *t*<sub>0</sub> at which the current shape was last
  changed by a transition (claim 6). A restarted transition also has a *start orientation*:
  the quaternion of the interrupted transition at the tick of the change, excluding rocking
  and body facing. This preserves continuity of slope tilt without preserving unrelated rocking.
- **Transition fraction** *p*. 1 when no transition is running, else (*t* − *t*<sub>0</sub>) / 3 for
  ticks *t* = *t*<sub>0</sub>, *t*<sub>0</sub> + 1, *t*<sub>0</sub> + 2 (claim 6).
- **Rocking.** A pitch or roll of a unit's body, of 0.005 radians or more, that its locomotor applies
  for reasons outside this spec (firing, being hit, crushing and so on). S-0105 calls a body that is
  rocking or in a transition *not settled*.
- **Mechanical walking locomotor.** A walking locomotor distinct from the infantry walking
  locomotor, selectable by a type's locomotor field like the others (glossary: locomotor). For this
  spec it behaves like the walking locomotor.
- **Tilting locomotor.** The driving locomotor (wheeled and tracked vehicles) and the naval
  locomotor (ships), which keep a tilt state (claim 6).
- **Drop-pod locomotor, tunnelling locomotor.** Two further locomotors selectable by a type's
  locomotor field: the one that brings a unit down from the sky in a pod, and the one that digs into
  the ground, travels underground and surfaces again. The tunnelling locomotor's *surface state* is
  the state in which its unit is above ground and neither digging in nor surfacing. Their movement belongs to
  the movement specs; stock data gives neither of them to a voxel unit.
- **Teleporting locomotor, rocket locomotor.** Two further locomotors selectable by a type's
  locomotor field, named after how they move their units. A *rocket* is a unit with the rocket
  locomotor. Their movement belongs to the movement and aircraft specs.
- **Powered down.** A hovering locomotor is *powered* while its unit is switched on, or while its
  hover height is strictly positive; otherwise it is *powered down*. Hover height is the unit's
  world height minus the terrain surface height at its own position (claim 2), minus a further
  416 leptons when it is on a bridge deck. These are exact integer subtractions in leptons,
  with no rounding. S-0503 claims 8 and 10 own settling, switching off and re-enabling: a
  switched-off hovering unit remains powered until its hover height reaches 0. Loss of the
  control building on which the type depends can switch it off.

## Behaviour

### Slope shapes and the terrain surface

1. **Corner heights of the shapes.** The slope shapes 1 … 16 raise the corners listed in Constants
   table C1. Their surface is planar where it lies between 0 and its maximum height, and is clipped
   flat at 0 and at the maximum (claim 2). So a corner slope is flat at the cell's level over the
   half of the cell away from its raised corner, and a valley slope is flat one level up over the
   half towards its three raised corners.
   - Backing: `measurable` (scenario T1)
   - Confidence: high

2. **Surface height.** For a cell at height level ℓ with slope shape *s*, the surface height at
   in-cell position (u, v) is *H* = T(*B*(ℓ) + *h*<sub>s</sub>(u, v)) (Formulas F1), where
   *B*(ℓ) is the level base and *h*<sub>s</sub> the shape's rise. The rise of shapes 1 … 16 is the
   clipped plane of table C2, so for example a shape-1 cell rises from 0 at its west edge to
   103 leptons at u = 255 (not 104: the rise is truncated, so the next cell's floor, a full level up,
   is 1 lepton higher than the last lepton of the slope). Shapes 17 … 20 rise by exactly 52 leptons
   everywhere in the cell: the surface is flat at half a level, whatever the saddle's corner
   pattern (Open questions).
   The cell's centre height, which much of the game uses as "the height of the cell", is *H* at
   (128, 128): ℓ × 104 + 52 for edge slopes and saddle shapes, ℓ × 104 for corner slopes and
   ℓ × 104 + 104 for valley and steep slopes (for ℓ ≥ 0).
   - Backing: `measurable` (scenario T1). Each recorded position tests F1, including its
     truncation and clipping. A finite run confirms only its samples, not an exhaustive
     equivalence over all positions; absence of exhaustive sampling does not make the
     remaining surface heights unobservable.
   - Confidence: high (the formula reproduces the original's arithmetic exactly for all 21
     shapes); medium that every ground object stands exactly at *H* (the movement specs own which
     objects follow the surface, and how)

3. **Level base below level 0.** The level base is *B*(ℓ) = 104ℓ for ℓ ≥ 0 but 104ℓ + 1 for
   ℓ < 0: it is 104ℓ + 0.5 truncated toward zero (F1). A flat cell at level −1 has
   its surface at −103 leptons, at level −2 at −207 (`quirks.negative_level_floor_raised`). S-0009
   claim 9 states the same values and points here for the surface height.
   - Backing: `measurable` (scenario T1, negative-level part)
   - Confidence: medium (the truncation is certain; that the objects a measurement can see stand at
     *H* is the medium part of claim 2)

### Tilt of each shape

4. **Fixed tilt per shape.** A unit whose body is tilted to slope shape *s* (claims 5 to 9) is drawn
   with the corrected tilt transform *R*<sub>s</sub> of Formulas F2. Its up axis leans away from the
   uphill direction, so the uphill end of the body is raised. The nominal tilt follows the surface
   as drawn: arctan(1/√6) radians for edge slopes and π/6 radians for corner, valley and steep
   slopes, with the table-trigonometry precision of F2. This is a presentation-only correction:
   surface heights, movement, collision and firing calculations are unchanged. In particular,
   gas slope deflection retains the original transforms of table C4, not the corrected drawing
   transforms (S-0613 claim 10). There is no switch for the drawing correction.
   The original used mismatched shape-dependent angles, making voxel bodies and shadows appear
   too steep on edge slopes and too shallow on corner and valley slopes.
   For oracle comparisons only, its angles were:

   | Shapes | Tilt of the up axis | Angle of the surface as drawn |
   |---|---|---|
   | 1 … 4 (edge) | 29.3115° | arctan(1/√6) ≈ 22.21° |
   | 5 … 12 (corner and valley) | 21.3574° | 30° |
   | 13 … 16 (steep) | 29.3115° | 30° |
   | 0 and 17 … 20 | none (identity) | 0 for flat terrain / saddle tile geometry unresolved |

   A corner slope and the valley slope with the same uphill direction (5 and 9, 6 and 10,
   7 and 11, 8 and 12) give identical tilts in both renderings. Flat and saddle shapes retain
   identity tilt; unresolved saddle artwork does not define a replacement orientation.
   - Backing: `measurable` (T2: original transforms against the oracle, corrected transforms
     as engine conformance). Exact floating-point values that give the same captured pixels
     are `unmeasured`: T2 cannot separate them. Gas deflection is `measurable` through
     S-0613 claim 10's scenario G8, using its F3 with C4 unchanged. Other unchanged game calculations are
     `unmeasured` here: the scenario compares images, not every possible gameplay outcome.
   - Confidence: high for the original transforms and the correction's definition; medium
     for the surface-angle comparison, which uses S-0105 F2 and F3 and S-0101 claims 1 and 4

### Which bodies tilt

5. **Locomotors.** Whether and how a voxel body is tilted depends on its locomotor:

   | Locomotor | Body tilt |
   |---|---|
   | driving, naval | tilt state with transitions (claims 6 and 7) |
   | hovering | *R*<sub>s</sub> of the cell under the unit, applied at once, only while the unit is not rocking and its locomotor is powered down (Terms: a unit with strictly positive hover height counts as powered, so a switched-off hovering unit is tilted only once it settles to hover height 0); a powered hovering unit is never tilted by slopes |
   | tunnelling | *R*<sub>s</sub> of the cell under the unit, applied at once, while the locomotor is in its surface state; in its other states (digging in, underground, surfacing) no slope tilt (the pitch it shows while digging in and surfacing belongs to the vehicle specs) |
   | walking, mechanical walking, drop-pod, teleporting, jumpjet | never tilted by slopes (the jumpjet's rocking aside, claim 9) |
   | flying, rocket | no tilt from slopes |

   "The cell under the unit" is the cell that contains the unit's centre (its height ignored).
   - Backing: `measurable`: scenario T4 for the locomotors that stock data gives to voxel units
     (driving, teleporting, powered hovering, jumpjet, flying, rocket), T6 for the others
     (walking, mechanical walking, drop-pod, tunnelling, naval, given to a voxel vehicle by named
     test data) and T7 for a powered-down hovering unit
   - Confidence: high for driving, naval, walking, mechanical walking, drop-pod, teleporting and
     jumpjet; high for hovering's powered-height test (S-0503 claims 8 and 10),
     medium for tunnelling (its surface states belong to the vehicle specs),
     medium for flying and rocket, whose bodies take their orientation from rules
     of the aircraft specs that this spec does not fix

6. **Tilt state of the tilting locomotors.** On each ground-layer movement tick on which the slope shape of the cell
   that contains the unit's centre differs from its current shape, a unit with a tilting locomotor
   starts a transition of 3 ticks, at that tick *t*<sub>0</sub>, from its current shape to the
   cell's shape: from then on its previous shape is the shape that was current, and its current
   shape is the cell's shape. A change while a transition is still
   running starts a new three-tick transition from the interrupted transition's orientation
   at the tick of the change, evaluated by F3, to the new shape's orientation. The slope tilt
   therefore stays continuous on restart; the previous shape remains a record of the old
   current shape, not the restart's interpolation endpoint.
   The original instead restarted from the old current shape, causing a presentation-only jump.
   There is no switch for that jump. When the unit is
   placed on the map, its current and previous shapes are both set to the shape of its cell and no
   transition runs, so it starts settled. This claim covers ground-layer units only.
   A stationary unit whose cell changes shape and a unit on a bridge deck are left open.
   - Backing: `measurable` for ordinary transitions (T3), settled placement (T2's
     placement part and T6), and interrupted transitions (T11's restart part: oracle
     comparison for the original jump, engine conformance for continuous restart).
   - Confidence: high

7. **Drawn tilt during a transition.** In the drawn frames of tick *t*<sub>0</sub> + *j*, for
   *j* = 0, 1, 2, a non-rocking unit's body is drawn with *D* = *M* · Rz(*θ*<sub>body</sub>), the
   product, in that order, of the rotation *M* of the spherical interpolation from
   the start orientation to *q*<sub>s<sub>c</sub></sub> at fraction *p* = *j*/3 (Formulas F3).
   For an uninterrupted transition the start orientation is *q*<sub>s<sub>p</sub></sub>.
   For a restart it is claim 6's interrupted orientation. This interpolation is combined
   with its 32-step body rotation, as in claim 8. From tick *t*<sub>0</sub> + 3
   on, the body is settled and drawn with *R*<sub>s<sub>c</sub></sub> (claim 8). Because the
   interpolated orientations describe slightly different rotations from the table transforms, small
   endpoint differences can occur in an uninterrupted transition. The following numeric examples
   describe the original angles only, not claim 4's corrected rendering: at *j* = 0 the old tilt is drawn as
   *q*<sub>s<sub>p</sub></sub> (for an edge slope 29.3555° instead of 29.3115°), and at
   *t*<sub>0</sub> + 3 the tilt snaps from the two-thirds orientation to the table transform. For
   example, driving from flat ground onto an edge slope the drawn tilt is 0°, 9.756°, 19.600°, then
   29.3115°. During the transition S-0105 claim 26 draws every part fresh.
   - Backing: `measurable` (T3 and T11's pair part) for transition images and timing,
     including a change between equal orientations. Restart continuity is engine conformance
     in T11, not an oracle expectation. Exact component values that produce
     identical pixels are `unmeasured`, for the arithmetic reason of claim 4.
   - Confidence: high for the three-tick timing and the interpolation endpoints; high for the
     intermediate rotations (the sample values below follow from F3 and tables C3 and S-0105 F1),
     and medium for which tick's drawn frames show *j* = 0 (whether the drawn frames of tick
     *t*<sub>0</sub> already show the transition, Open questions)

8. **Settled tilt.** A body that is settled (no transition running and no rocking) is drawn with
   *D* = *R*<sub>s<sub>c</sub></sub> · Rz(*θ*<sub>body</sub>), the product of the shape's tilt
   (left factor) and the body's 32-step rotation of S-0105 claim 3 (right factor). The tilt therefore does not depend on the body facing.
   The body's ramp state (S-0105 claim 26) is the shape whose tilt it shows: *s*<sub>c</sub> for
   the tilting locomotors, and the shape of the cell under the unit for a powered-down hovering
   unit and for a tunnelling unit in its surface state. Walking, mechanical walking,
   drop-pod, teleporting, jumpjet and powered hovering bodies have no ramp state.
   Full reuse keys for flying and rocket bodies remain open in S-0105; no complete key
   for those bodies is asserted here. For a body with ramp state, an image made on one
   shape is never reused on another, although shapes 5 … 8 and 9 … 12 give identical images.
   - Backing: `measurable` (T2 for driving, T6 for tunnelling and naval, T7 for hovering)
     for the settled transform and rejection of a flat image on shapes 1 … 16. T12 tests
     separate reuse classes for equal-transform shapes and reuse across shapes for
     untilted bodies, through fresh-draw shading rather than silhouette alone.
   - Confidence: high

9. **Rocking.** While a unit with the driving or naval locomotor rocks, its body transform is built
   from its pitch and roll, the tilt of *p* (the table transform when *p* = 1, else the interpolated
   orientation of claim 7) and its body rotation. How pitch and roll arise, and how they combine
   with the tilt, belong to the vehicle specs. The jumpjet locomotor's body shows only its rocking,
   never a slope, and a hovering unit shows no slope tilt while it rocks (claim 5). That a rocking
   body of the driving or naval locomotor is drawn fresh is S-0105 claim 26's claim (claim 13 for
   the other locomotors); this claim adds only that the tilt of *p* stays in the transform.
   - Backing: `measurable` (T2's rocking part and T11's rocking part) for driving,
     naval, hovering and jumpjet cases, including rocking during a transition.
     Exact pitch/roll composition remains outside this claim and belongs to the vehicle specs.
   - Confidence: medium

10. **No per-type opt-out.** The vehicle types' imported tilter flag is read (Import mapping) but
    nothing in the original uses it: a vehicle with the flag set to no tilts exactly like one with
    it set to yes (`quirks.tilter_flag_ignored`).
    - Backing: `measurable` (scenario T5)
    - Confidence: medium (no use of the flag was found; an indirect use cannot be ruled out without
      the measurement)

### Shadows

11. **Shadow tilt.** For every locomotor, whenever a voxel shadow is drawn, the transform of its
    unmasked footprint, *S* of S-0105 claim 30, is *R*<sub>s</sub> · Rz(*θ*<sub>body</sub>), where *s* is the slope
    shape of the cell that contains the unit's centre (its height ignored), taken at the moment of
    the draw: there is no transition, and the unit's own tilt state is not used. So the shadow of a
    walking, teleporting or powered hovering unit on a slope is tilted although its body is not,
    the shadow of a jumpjet, an aircraft or a rocket in flight is tilted by the shape of the cell
    below it, and while a driving unit's body is in a transition its shadow already shows the new
    shape. One exception: an aircraft with the flying locomotor whose type has the dropship flag
    (Import mapping) always gets the flat shadow (*s* taken as 0). For aircraft, which of the
    aircraft's facings gives *θ*<sub>body</sub> belongs to the aircraft specs (Open questions).
    These tilt assertions describe the unmasked footprint, not every darkened screen pixel.
    Every vehicle shadow, fresh or reused, excludes the vehicle's current completed own-part
    image (S-0105 claim 39). Only the unmasked footprint is cached, so an earlier pose or
    another object of the same type and key supplies no retained exclusions.
    The original reused shadows with old-pose cutouts, a presentation glitch fixed by that claim.
    - Backing: `measurable`: scenario T4 for the driving, teleporting, hovering, jumpjet, flying
      and rocket locomotors, T6 for the walking, mechanical walking, drop-pod, tunnelling and naval
      locomotors, T3 for the shadow during a transition and T8 for the dropship exception
    - Confidence: high for the transform of every locomotor; medium for which drawn objects have a
      shadow at all (a rocket, a tunnelling unit below ground), which S-0105 and the aircraft and
      vehicle specs own, and for the flying locomotor's facing

12. **Shadow images and the cache.**
    - **Ramp state of the shadow.** The ramp state in a shadow's image-cache key (S-0105 claim 31)
      is the *s* of claim 11, for every locomotor (always 0 for an aircraft with the dropship
      flag). So an unmasked shadow footprint made on flat ground is never reused on a slope.
      - Backing: `measurable` (scenario T3 and T6) when the two shapes have visibly
        different footprints: a flat footprint must not reappear on shapes 1 … 16 at
        the same facing. Separate shadow reuse classes for equal-transform shapes
        are `unmeasured`: unlike body shading in T12, the unmasked shadow has no
        fresh-draw shading difference, and current masking agrees on every appearance,
        so no scenario here distinguishes their separate keys.
      - Confidence: high
    - **Unsettled shadows.** While a driving or naval unit's body is not settled, and while a
      hovering unit rocks, its shadow image is made fresh: it is never taken from the image cache
      and is not added to it, although the shadow transform is the same as when it is settled.
      If eligible under S-0105 claim 31's cloak and no-shadow conditions, such a shadow is always
      drawn, so the skip on a cache miss that S-0105 claim 31 leaves open never applies to it.
      S-0105 claim 31 includes this unsettled-shadow freshness case. No shadow enables
      depth mode, regardless of its type's depth-buffer option or whether it is fresh.
      Only a fresh depth-option part draw uses that mode (S-0105 claim 32); shadows
      ordinarily preserve retained depths, apart from the initial clear (S-0105 claim 34).
      Another object's depth look cannot change because of an unsettled shadow
      (S-0105 claim 33). The original could leave depth mode enabled after a depth-option
      shadow, a presentation glitch fixed by those claims.
      - Backing: `unmeasured` for hovering, driving and naval shadow freshness and
        non-retention. The key covers every input of the unmasked footprint, so fresh and
        reused footprints have the same pixels; current own-image exclusions also agree
        on every appearance (S-0105 claims 31 and 39). T10 therefore tests corrected
        masking and mode isolation, not freshness or storage. Exemption from an original
        cache-miss skip is also `unmeasured`: the skip's enabling condition is unknown
        in S-0105, so no practical comparison can force it while changing only settledness.
        Masking and depth-mode isolation are `measurable` through T10's engine-conformance
        comparisons, following S-0105 claims 32, 33, 34 and 39.
      - Confidence: high

13. **Fresh bodies of other locomotors.** Besides the driving and naval cases of S-0105 claim 26,
    the original draws a unit's body fresh, and keeps no image of it, while a hovering unit rocks,
    while a jumpjet rocks, and while the digging pitch of a tunnelling unit is changing (as it digs
    in or surfaces). This is also specified in S-0105 claim 26.
    - Backing: `measurable` (scenario T9)
    - Confidence: high for the hovering and jumpjet cases; medium for which tunnelling states
      change the pitch (the states belong to the vehicle specs)

## Formulas

**F1. Surface height (claims 2 and 3).** For a cell at height level ℓ (integer) with slope shape
*s*, and an in-cell position (u, v):

> *B*(ℓ) = T(104ℓ + 0.5)
>
> *h*<sub>s</sub>(u, v) = min(max(13·(*a*<sub>s</sub>·u + *b*<sub>s</sub>·v) / 32 + *c*<sub>s</sub>, 0), *d*<sub>s</sub>) for 1 ≤ *s* ≤ 20, and *h*<sub>0</sub> = 0
>
> *H* = T(*B*(ℓ) + *h*<sub>s</sub>(u, v)). Unit: leptons.

T truncates toward zero. *a*, *b*, *c*, *d* are the integers of table C2, and 13/32 = 104/256 is the
rise per lepton of one level across a cell. The original works in double precision. Every value
here is exact in double precision, so the result does not depend on the round-toward-zero mode, and
exact rational arithmetic reproduces it. For ℓ ≥ 0, *B*(ℓ) = 104ℓ and *H* = ⌊104ℓ + *h*⌋; for
ℓ < 0, *B*(ℓ) = 104ℓ + 1, and *H* is *B* + *h* truncated toward zero.

**F2. Tilt transforms (claim 4).**

> *R*<sub>s</sub> = Rz(*α*<sub>s</sub>) · Rx(*τ*<sub>s</sub>) · Rz(−*α*<sub>s</sub>)

with the rotations, the right-handed convention and the right-multiplication of S-0105 (F2), table
trigonometry for every sine and cosine (S-0105 F1) and single-precision stores truncated toward zero
(S-0105 F0). *α*<sub>s</sub> (radians) and *τ*<sub>s</sub> are:

| Shapes | *α* | *τ* |
|---|---|---|
| 1, 2, 3, 4 | 3π/2, π, π/2, 0 | *τ*<sub>A</sub> |
| 5, 6, 7, 8 and 9, 10, 11, 12 | 5π/4, 3π/4, π/4, 7π/4 | *τ*<sub>B</sub> |
| 13, 14, 15, 16 | 5π/4, 3π/4, π/4, 7π/4 | *τ*<sub>A</sub> |
| 0, 17, 18, 19, 20 | *R* is the identity | |

For the corrected rendering, replace the angle column above with arctan(1/√6) radians for
shapes 1 … 4 and π/6 radians for shapes 5 … 16. Evaluate these mathematical angles in double
precision, rounded toward zero under S-0105 F0 (to nearest with its rounding switch off).
No integer rounding is applied to these angles. The rotation operations and single-precision stores
still follow S-0105 F0 and F2. Thus the nominal angle matches the drawn terrain, subject only
to the shared table-trigonometry resolution, rather than to the coarse arctangent grid.
Flat and saddle shapes keep identity. Use these corrected angles for bodies, turrets, barrels,
shadows and the construction of transition orientations alike; drawing never mixes the two
angle conventions. This correction does not apply to simulation gas deflection (S-0613 claim 10),
which uses the original transforms of C4. The original-angle column and C3's numeric rows are
oracle references for drawing; C4 additionally remains a simulation constant table.
Corrected drawing orientations are defined by C3's construction rule with the corrected angles.
Corrected drawing matrices are defined by this rotation formula.

For the original-angle references only, the two angles are values of the original's table
arctangent arctan<sub>t</sub>, a tabulated
arctangent on a grid of arguments with step *Δ* = 0.024413354694843292 (a single-precision number:
4096 steps reach an argument of about 100, 4096*Δ* ≈ 99.997):

> arctan<sub>t</sub>(*x*) = sgn(*x*) · *A*<sub>*i*</sub>, with *i* = T(|*x*| / *Δ*), for *i* ≤ 4096
>
> arctan<sub>t</sub>(*x*) = sgn(*x*) · 1.5707963705062866, for *i* > 4096

where *A*<sub>*i*</sub> (radians) is the table's angle for *i* steps and 1.5707963705062866 rad is
π/2 rounded to single precision. No argument in this spec reaches the second case. Only two
entries occur here, so they are given as constants:

> *τ*<sub>A</sub> = arctan<sub>t</sub>(208 / 362.0386657714844) = *A*<sub>23</sub> = 0.51163476705551147 rad (29.3145°)
>
> *τ*<sub>B</sub> = arctan<sub>t</sub>(104 / 256) = *A*<sub>16</sub> = 0.37238854169845581 rad (21.3363°)

Here 362.0386657714844 (leptons) is the table square root of 2 × 256² (S-0105 F1), and 104 is the height
level in leptons. The exact arctangents would be 29.88° and 22.11°. The tilt of the up axis in
claim 4 (29.3115°, 21.3574°) differs again from *τ* because the three rotations are made with table
trigonometry. Constants table C4 gives the resulting matrices.

**F3. Interpolated tilt (claim 7).** Let *q*<sub>0</sub> be the start orientation
(*q*<sub>s<sub>p</sub></sub> from C3's corrected-angle construction for an uninterrupted
transition, or claim 6's interrupted quaternion for a restart), *q*<sub>1</sub> = *q*<sub>s<sub>c</sub></sub>
(C3's construction rule using claim 4's corrected angle), and *t* = *p* stored in single precision.
For the uninterrupted start, likewise use that construction rule rather than C3's historical
numeric row. Oracle comparisons instead use the original numeric rows for both endpoints. Let *δ* be the four-component dot product *q*<sub>0</sub> · *q*<sub>1</sub>, stored in single precision. Define the interpolation weights by

> *(*k*<sub>0</sub>, *k*<sub>1</sub>)* = (*1 − t*, *t*) when *1 − δ ≤ 10⁻⁵*,
>
> otherwise, let *θ* = acos<sub>t</sub>(*δ*) and set
>
> *k*<sub>0</sub> = sin((*1 − t*)·*θ*) / sin *θ*,   *k*<sub>1</sub> = sin(*t*·*θ*) / sin *θ*.
>
> The interpolated orientation is *q* = *k*<sub>0</sub>·*q*<sub>0</sub> + *k*<sub>1</sub>·*q*<sub>1</sub>, with each component stored in single precision and without renormalisation.

The sines are table sines (S-0105 F1). acos<sub>t</sub>(*x*) = π/2 − *S*(⌊2048(*x* + 1)⌋), with π/2
in double precision, where *S*(*m*), for *m* = 0 … 4096, is the exact arcsine of *m*/2048 − 1
truncated toward zero to single precision (4097 fixed values). F3 holds for *δ* + 1 > 10⁻⁵, which
every pair of tilt orientations satisfies (all have *w* > 0.96). For *δ* + 1 ≤ 10⁻⁵ the original's
result differs and is not given here. The
drawn transform is the matrix that the unit-quaternion formula gives for *q* as it is (*q* is not
renormalised, so its norm can differ from 1 by about 2 × 10⁻⁴ and the matrix is then not exactly a
rotation):

| | x | y | z |
|---|---|---|---|
| row x | 1 − 2(*y*² + *z*²) | 2(*xy* − *wz*) | 2(*xz* + *wy*) |
| row y | 2(*xy* + *wz*) | 1 − 2(*x*² + *z*²) | 2(*yz* − *wx*) |
| row z | 2(*xz* − *wy*) | 2(*yz* + *wx*) | 1 − 2(*x*² + *y*²) |

At *p* = 0 it is the matrix of the start orientation itself. Floating-point work follows
S-0105 F0 (the round-toward-zero mode of S-0205 claim 5, switch `quirks.chop_rounding`).

Sample values (tilt of the up axis, taken as the arccosine of the row z, column z entry; they
reproduce the original's arithmetic): flat to shape 1 at *p* = 1/3,
2/3: 9.7559°, 19.5996°; shape 1 to shape 4 at *p* = 1/3 and 2/3: 22.0749° both, the up axis leaning
towards 243.27° and then 206.73° clockwise from map north (between the westward lean of shape 1 and
the southward lean of shape 4); shape 5 to shape 7: 7.1054° at both fractions, leaning north-west
and then south-east.

## Constants

**C1. Shapes 1 … 16.**

| Shape | Raised corners (by levels) | Uphill direction |
|---|---|---|
| 1 | right 1, bottom 1 | map east |
| 2 | bottom 1, left 1 | map south |
| 3 | left 1, top 1 | map west |
| 4 | top 1, right 1 | map north |
| 5 | bottom 1 | map south-east |
| 6 | left 1 | map south-west |
| 7 | top 1 | map north-west |
| 8 | right 1 | map north-east |
| 9 | right 1, bottom 1, left 1 (top low) | map south-east |
| 10 | bottom 1, left 1, top 1 (right low) | map south-west |
| 11 | left 1, top 1, right 1 (bottom low) | map north-west |
| 12 | top 1, right 1, bottom 1 (left low) | map north-east |
| 13 | bottom 2, right 1, left 1 | map south-east |
| 14 | left 2, top 1, bottom 1 | map south-west |
| 15 | top 2, right 1, left 1 | map north-west |
| 16 | right 2, top 1, bottom 1 | map north-east |

**C2. Rise of each shape** (F1). Units: *a*, *b* dimensionless; *c*, *d* leptons.

| *s* | *a* | *b* | *c* | *d* | | *s* | *a* | *b* | *c* | *d* |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 1 | 0 | 0 | 104 | | 11 | −1 | −1 | 208 | 104 |
| 2 | 0 | 1 | 0 | 104 | | 12 | 1 | −1 | 104 | 104 |
| 3 | −1 | 0 | 104 | 104 | | 13 | 1 | 1 | 0 | 208 |
| 4 | 0 | −1 | 104 | 104 | | 14 | −1 | 1 | 104 | 208 |
| 5 | 1 | 1 | −104 | 104 | | 15 | −1 | −1 | 208 | 208 |
| 6 | −1 | 1 | 0 | 104 | | 16 | 1 | −1 | 104 | 208 |
| 7 | −1 | −1 | 104 | 104 | | 17 | 0 | 0 | 52 | 52 |
| 8 | 1 | −1 | 0 | 104 | | 18 | 0 | 0 | 52 | 52 |
| 9 | 1 | 1 | 0 | 104 | | 19 | 0 | 0 | 52 | 52 |
| 10 | −1 | 1 | 104 | 104 | | 20 | 0 | 0 | 52 | 52 |

**C3. Original tilt orientations and the construction rule.** *q*<sub>s</sub> = (x, y, z, w),
single precision, dimensionless. The numeric rows are oracle references only; corrected engine
orientations use the rule following the table with F2's corrected angles.

| Shapes | *q* |
|---|---|
| 0, 17 … 20 | (0, 0, 0, 1) |
| 1 | (0, −0.2533820, 0, 0.9673663) |
| 2 | (−0.2533820, 0, 0, 0.9673663) |
| 3 | (0, 0.2533820, 0, 0.9673663) |
| 4 | (0.2533820, 0, 0, 0.9673663) |
| 5 and 9 | (−0.1310319, −0.1310319, 0, 0.9826817) |
| 6 and 10 | (−0.1310319, 0.1310319, 0, 0.9826817) |
| 7 and 11 | (0.1310319, 0.1310319, 0, 0.9826817) |
| 8 and 12 | (0.1310319, −0.1310319, 0, 0.9826817) |
| 13 | (−0.1791736, −0.1791736, 0, 0.9673663) |
| 14 | (−0.1791736, 0.1791736, 0, 0.9673663) |
| 15 | (0.1791736, 0.1791736, 0, 0.9673663) |
| 16 | (0.1791736, −0.1791736, 0, 0.9673663) |

Each is the rotation about the horizontal axis (cos *α*<sub>s</sub>, sin *α*<sub>s</sub>, 0) of F2's
construction by the angle *τ*<sub>s</sub> of its shape: (x, y, z) = sin(*τ*<sub>s</sub>/2) times the
normalised axis, and *w* = cos(*τ*<sub>s</sub>/2),
with the sine and cosine of the half angle taken from the table trigonometry of S-0105 F1 (so the
half angle is effectively rounded to a whole step of 1/8192 turn). The axis is normalised with the
table square root of S-0105 F1, so a diagonal axis has components ±0.7071284 rather than ±1/√2,
which is why the x and y entries of shapes 5 … 16 are not the half-angle sine divided by √2. Values to
7 decimals; the original's are the single-precision numbers nearest to them.

**C4. Original tilt transforms** (drawing oracle references and simulation gas-deflection
constants, S-0613 claim 10), rows listed top to bottom, single precision, dimensionless.
Corrected drawing *R*<sub>s</sub> is F2 with claim 4's angles, not these numeric rows.
Gas deflection retains these rows. Shapes
9 … 12 equal 5 … 8; shapes 0 and 17 … 20 are the identity.

| *s* | row 1 | row 2 | row 3 |
|---|---|---|---|
| 1 | 0.8719697, −0.00133758574, −0.489557832 | 0.00153398002, 0.999998808, 1.19903125·10⁻¹⁶ | 0.489557236, −0.000750971958, 0.871970773 |
| 2 | 0.999998808, −0.00153398002, 5.99515626·10⁻¹⁷ | 0.00133758574, 0.8719697, 0.489557832 | −0.000750971958, −0.489557236, 0.871970773 |
| 3 | 0.8719697, −0.00133758574, 0.489557832 | 0.00153398002, 0.999998808, −5.99515626·10⁻¹⁷ | −0.489557236, 0.000750971958, 0.871970773 |
| 4 | 1, 0, 0 | 0, 0.871970773, −0.489557832 | 0, 0.489557832, 0.871970773 |
| 5 | 0.965714812, 0.0328553095, −0.257517487 | 0.0358179212, 0.965609491, 0.257517487 | 0.257122159, −0.257912219, 0.931326687 |
| 6 | 0.965609491, −0.0358179212, 0.257517487 | −0.0328553095, 0.965714812, 0.257517487 | −0.257912219, −0.257122159, 0.931326687 |
| 7 | 0.965714812, 0.0328553095, 0.257517487 | 0.0358179212, 0.965609491, −0.257517487 | −0.257122159, 0.257912219, 0.931326687 |
| 8 | 0.965609491, −0.0358179212, −0.257517487 | −0.0328553095, 0.965714812, −0.257517487 | 0.257912219, 0.257122159, 0.931326687 |
| 13 | 0.936082423, 0.0625787526, −0.346169651 | 0.0654503107, 0.935886025, 0.346169651 | 0.345638216, −0.346700251, 0.871970773 |
| 14 | 0.935886025, −0.0654503107, 0.346169651 | −0.0625787526, 0.936082423, 0.346169651 | −0.346700251, −0.345638216, 0.871970773 |
| 15 | 0.936082423, 0.0625787526, 0.346169651 | 0.0654503107, 0.935886025, −0.346169651 | −0.345638216, 0.346700251, 0.871970773 |
| 16 | 0.935886025, −0.0654503107, −0.346169651 | −0.0625787526, 0.936082423, −0.346169651 | 0.346700251, 0.345638216, 0.871970773 |

The small off-axis entries (for example 0.00133758574 in shape 1) come from table trigonometry,
which does not give exact zeros at multiples of π/2 (S-0105 F1). They are part of the original.

**Other constants.**

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Slope shapes | 21 | shapes | 0 … 20 |
| Rise per lepton | 13/32 | leptons of height per lepton | One level (104) across a cell edge (256) |
| Saddle rise | 52 | leptons | Shapes 17 … 20, claim 2 |
| Transition length | 3 | ticks | Claims 6 and 7 |
| Table arctangent step | 0.024413354694843292 | argument units per entry | F2; 4096 steps reach about 100 (99.997) |
| Original *τ*<sub>A</sub> | 0.51163476705551147 | radians | F2, oracle reference only |
| Original *τ*<sub>B</sub> | 0.37238854169845581 | radians | F2, oracle reference only |
| Interpolation linear threshold | 10⁻⁵ | dimensionless | F3 |
| Rocking threshold | 0.005 | radians | S-0105 claim 26 |

## State and ordering

Ground-layer tilt state of a unit with a tilting locomotor (claims 6 to 8), excluding
stationary cell-shape changes and multiple-border crossings left open below.
*σ* is the slope shape of the cell that contains the unit's centre.

| State | Event | Result |
|---|---|---|
| any | the unit is placed on the map | *s*<sub>c</sub> = *s*<sub>p</sub> = *σ*; no transition running |
| no transition running, on *s*<sub>c</sub> | a tick on which *σ* ≠ *s*<sub>c</sub> | in transition from the former *s*<sub>c</sub> to *σ*: *s*<sub>p</sub> is the former *s*<sub>c</sub>, *s*<sub>c</sub> is *σ*, *t*<sub>0</sub> is this tick |
| in transition | a tick on which *σ* ≠ *s*<sub>c</sub> | new three-tick transition towards *σ*, starting from the interrupted orientation at this tick (F3); previous/current shapes and start tick change as above; slope tilt is continuous |
| in transition | tick *t*<sub>0</sub> + 3 begins with *σ* = *s*<sub>c</sub> | no transition running, on *s*<sub>c</sub> |

Rocking (claim 9) is not part of this state: the rows hold whether or not the unit rocks. A body
is *settled* in S-0105's sense when no transition is running and it does not rock.

The table specifies the shape observed at a tick, not the sampling of an unobserved cell
crossed between ticks. Multiple-border crossings are left open below. Backing and confidence
for the table are those of claims 6 and 7.

## Edge cases and quirks

All switches default to the original behaviour.

- **Shape-dependent angles.** Claim 4 corrects this drawing artefact unconditionally under
  the presentation-only bug policy. The reference is the terrain as drawn, not its angle in
  world leptons: the comparison concerns visual alignment in model space. F2 and C3 define
  the corrected rendering; their historical values and C4 support drawing oracle comparisons.
  C4 also supplies the unchanged simulation gas deflection of S-0613 claim 10, so the drawing
  correction must not replace that table's simulation values.
- **`quirks.negative_level_floor_raised`.** Original: below level 0 the surface is 1 lepton higher
  than 104ℓ (claim 3). Fixed: *B*(ℓ) = 104ℓ for every level.
- **Continuous restart.** Claim 6 removes the original's presentation-only restart jump
  unconditionally. T11 separates its oracle comparison from engine conformance.
- **`quirks.tilter_flag_ignored`.** Original: the tilter flag has no effect (claim 10). Fixed: a
  vehicle whose flag is no is never tilted by slopes (its body and shadow drawn as on flat ground).
- **Interpolated endpoints.** The small endpoint differences of claim 7 arise from the
  quaternion construction versus the settled matrix construction. Both use the same corrected
  nominal angles in our engine. Historical numeric examples use C3/C4's original angles.
  These numerical rendering differences get no switch of their own.
- **Shape values outside 0 … 20.** If the tile cell that gives a cell its slope shape (S-0010
  claim 6) carries a slope code outside 0 … 20 (the slope code of S-0010's Terms), the original's
  result is unspecified. Our engine treats such a cell as flat (shape 0) and the importer warns
  about the tile (a departure). This spec owns that departure; S-0010's Edge cases point here.
- **Rounding.** Claims 2 and 3 are exact and do not depend on the round-toward-zero mode. The tilt
  transforms and interpolations follow S-0105 F0; the mode and its switch `quirks.chop_rounding` are owned by S-0205 (claim 5).

## Import mapping

| User data | Key | Feeds | Form | Absent |
|---|---|---|---|---|
| rules, vehicle type section | `IsTilter` | read, no effect (claim 10, `quirks.tilter_flag_ignored`) | boolean (S-0003) | yes |
| rules, type sections (read for every type kind; only aircraft with the flying locomotor use it) | `IsDropship` | flat shadow for an aircraft (claim 11) | boolean (S-0003) | no |
| theater tile files | each tile cell's slope code | the cell's slope shape (claim 1; how a cell gets it from its tile is S-0010 claims 6 to 8) | integer | — |

The locomotor of a type, which selects the row of claim 5, is imported by the movement specs.

## Measurement scenarios

All run under the glossary's measurement setup with stock data, on purpose-made test maps in one
theater whose tileset data includes slope tiles of every shape used (the tilesets of the theater's
ramp roles, S-0010 claims 4 and 11). A test cell's slope shape is the one it holds after the map is
loaded, that is, after the ramp smoothing of S-0010 claims 11 and 12. A *recorder* that reads an object's world position (x, y, height) is used where stated.
Game speed 2, resolution 800 × 600, director and mods off except for the explicitly named
test data. The observed area is the test cells and routes; one bystander computer player is kept
outside it, with the glossary's void rule. Manual play or scripted input and tick-indexed image
capture are used.
*Runs:* one per scenario part and control, except T6 uses one per named locomotor,
T11 uses one per tested pair or locomotor, and T12 one per locomotor and shape pair. Repeat only void
or inconclusive runs. Every comparison uses the recorded positions, cells, facings and ticks. Where
an event that a scenario needs may fail to happen, the scenario says what is recorded instead.

*Renders* below are golden images made with S-0105's pipeline and the transforms of this spec at
the stated facing: the *C4 render* uses table C4 (and F3 with C3's original numeric endpoints
 during an uninterrupted transition).
The *corrected renders* use claim 4's angles for F2 and for C3's construction, and F3
with those corrected endpoints during transitions. The *geometric comparison renders*
replace only the original F2's nominal tilt angle: one uses
arctan(1/√6) for shapes 1 … 4 and π/6 for shapes 5 … 16, the other uses
arctan(104/256) and arctan(208/(256√2)), respectively. These angles are double-precision
radians, rounded as S-0105 F0 specifies; all other F2 operations are unchanged. They are
falsifying hypotheses for the oracle. The first is also the corrected engine's settled
render; the second is not an engine alternative. A *C4 shadow render* is
S-0105 claim 30's footprint with *S* = *R*<sub>s</sub> · Rz(*θ*<sub>body</sub>). The *flat image*
of a unit is its capture on flat ground at the same facing.

Shadow shape comparisons in T3, T4, T6 and T7 use that unmasked footprint only as the set
of potential shadow pixels. Golden images also apply S-0105 claim 39's vehicle own-image
exclusions from the current completed own-part image on every appearance; screen darkening
follows S-0102. Corrected reuse comparisons are engine conformance, not oracle checks.
A missing or retained pixel explained by those rules does not falsify the slope transform.

Unless a part explicitly says engine conformance, its numeric angles, C4 images and F3
endpoint rows below describe oracle expectations using the original angles. Replay each
image-comparison part on our engine with the same recorded pose and controls, replacing only
the historical tilt matrices and endpoint orientations with the corrected renders. Expected:
every eligible body and shadow matches the corrected render, including transitions and
settled placement. An image matching only the original-angle render, an incorrect transition
duration, or a body/shadow using different angle conventions falsifies the corrected rule.
Pixel-identical renders are inconclusive. Corrected restarts also use the continuous-start
rule already specified in T11; no oracle run is expected to reproduce either correction.

**T1. Surface height (claims 1 to 3).**
- *Setup:* a map with one cell of each slope shape 1 … 20 at height level 2, each surrounded by
  cells that fit it, plus flat cells at levels 0, 2, −1 and −2 (the map gives the negative levels
  directly, S-0009 claim 7).
  - For each test cell, a vehicle of one stock type with the driving locomotor is ordered to stop
    on the cell (it parks at the cell centre).
  - For each test cell, five infantry units of one stock type are ordered into the cell, so that
    they stand at different places in it.
  - A vehicle with the driving locomotor drives in a straight line across a saddle cell and across
    a shape-1 cell, at right angles to the shape-1 cell's uphill direction and then along it.
- *Observable:* the recorder's (x, y, height) of each parked vehicle, of each infantry unit once it
  stands still, and of the moving vehicle on every tick. For every sample, (u, v) = (x mod 256,
  y mod 256).
- *Expected:*
  - at the centre, 208 + 52 = 260 for shapes 1 … 4 and 17 … 20, 208 for 5 … 8 and 312 for 9 … 16,
    0 at level 0, 208 at level 2, −103 at level −1 and −207 at level −2,
  - every infantry sample equals *H* of F1 at its own (u, v): 260 anywhere in a saddle cell,
    208 + ⌊13u/32⌋ in a shape-1 cell, and so on through table C2,
  - the moving vehicle's samples follow the same values. When a moving unit's height is updated
    relative to its position belongs to the movement specs, so a vehicle sample that matches *H* at
    the position of the previous or the next tick is also accepted.
- *Falsified if:* any centre height differs from these values; an infantry sample in a saddle cell
  is not 260 (the saddles are not flat at half a level) or an infantry sample in another cell
  differs from F1 at its (u, v); −104 or −208 at the negative levels (no raised level base); or
  any moving vehicle sample fails F1 at its current, preceding and following recorded positions.
  The centre alone cannot tell a flat saddle from one interpolated between its corners, since both give
  52 there; the off-centre samples can. *Control:* the saddle samples count only if the infantry
  samples in the shape-1 cell of the same run vary with u as F1 predicts. If they do not (for
  example, if every infantry unit stands at its cell's centre height), the saddle part is
  inconclusive, not confirmed.

**T2. Tilt of each shape, ramp state and rocking (claims 4, 8 and 9).**
- *Setup:* the map of T1; one stock voxel vehicle with the driving locomotor and a long, asymmetric
  body. Placement part: place a separate vehicle of that type on a shape-1 cell at facing 0
  and capture its first drawn appearance; it must already match the settled C4 render, without
  intermediate tilts (claim 6). The travelling vehicle is first shown on flat ground at body
  facings 0, 8, 16 and 24 (32-step), then parked in
  turn on each slope cell at the same four facings, settled (no firing, no damage). Rocking part:
  parked settled on a shape-1 cell, the vehicle is hit by a stock weapon whose warhead rocks its
  target.
- *Observable:* screenshots of the body; in the rocking part, one capture per tick until the
  rocking ends.
- *Expected:* every settled screenshot matches the C4 render; shapes 5 and 9 (and 6 and 10, 7 and
  11, 8 and 12) give identical images; shapes 17 … 20 give the flat image; every other slope shows
  its C4 render, not the flat image already made at that facing (the ramp state is in the image
  cache's key). While it rocks, the body stays leaning towards map west by about the shape-1 tilt,
  with the rocking added, and returns to the shape-1 C4 render once the rocking ends.
- *Falsified if:* any settled screenshot fails to match the C4 render (in particular if it
  matches a geometric comparison render instead); a corner and a valley slope with the same uphill
  direction look different; a shape-1 … 16 cell shows the flat image; the placed vehicle starts flat or in an intermediate
  tilt; the rocking body is drawn as on flat ground with only the rocking. Comparisons of
  renders that produce identical pixels are inconclusive for their differing arithmetic,
  not confirmation of either formula.

**T3. Transition (claims 6, 7 and 11).**
- *Setup:* a straight route over flat ground, then three consecutive shape-1 cells (uphill east),
  then flat ground at the higher level. A stock voxel vehicle with the driving locomotor drives the
  route heading map east. A tick-by-tick recorder of drawn images (one capture per tick).
- *Observable:* for each tick, the tick number, the cell containing the vehicle's centre (recorder),
  the body image and the shadow image.
- *Expected:*
  - on the tick the centre first lies in a slope cell, the body is still drawn flat; the next two
    ticks show two intermediate tilts (the renders at *p* = 1/3 and 2/3 of F3 match); from the
    third tick after the crossing the body shows the C4 shape-1 image. Leaving the slope, the same
    pattern runs from the shape-1 image (as the 29.3555° orientation at *j* = 0) back to flat,
  - the shadow matches the C4 shadow render of shape 1 from the first tick on which the centre lies
    in a slope cell, while the body is still flat, and is flat from the first tick on which the
    centre lies on flat ground again. It shows no intermediate tilt.
- *Falsified if:* the body's tilt changes at once, takes a different number of ticks, or the
  intermediate images fail to match F3 at the recorded fraction; the shadow passes through intermediate
  tilts, or changes on a different tick from the one on which the centre changes cell.

**T4. Locomotors of stock voxel units, and their shadows (claims 5 and 11).**
- *Setup:* the map of T1, plus a row of cells that alternates flat and shape-1 cells. On a shape-1
  cell and on a shape-5 cell in turn, at body facings 0 and 8: a stock voxel vehicle with the
  driving locomotor; a stock voxel vehicle with the teleporting locomotor; a stock voxel vehicle
  with the hovering locomotor, switched on and hovering; a stock voxel vehicle of a jumpjet type,
  hovering over the cell. Along the alternating row: a stock voxel aircraft with the flying
  locomotor flying along it at a steady heading, and a stock voxel rocket with the rocket locomotor
  fired so that its path runs along it.
- *Observable:* screenshots of body and shadow; for the aircraft and the rocket, one capture per
  tick.
- *Expected:* only the driving vehicle's body is tilted (its C4 render). The teleporting, hovering
  and jumpjet bodies show their flat images. The bodies of the aircraft and the rocket change
  nothing on the ticks on which their centre passes from a flat cell to a slope cell or back,
  beyond the change of the ticks before and after. Every shadow that is drawn matches the C4 shadow
  render of the cell under the unit's centre: over the alternating row, the aircraft's and the
  rocket's shadows switch between flat and shape-1 on the ticks on which their centre changes cell.
- *Falsified if:* a teleporting, powered hovering or jumpjet body is tilted; the aircraft's or the
  rocket's body takes on a C4 tilt over a slope cell; a shadow that is drawn stays flat over a slope
  cell. (A rocket that casts no shadow at all tests only its body.)

**T5. Tilter flag (claim 10).**
- *Setup:* the map of T1 with an override section that sets the tilter flag of one stock driving
  vehicle type to no (named test data in the map). One vehicle of that type and one of a type whose
  flag is yes are parked on shape-1 cells with the same facing.
- *Observable:* screenshots of both bodies.
- *Expected:* the vehicle with the flag set to no is tilted exactly as in T2.
- *Falsified if:* its body fails to match its own C4 render, whether flat or tilted differently.
  The yes-flag vehicle is a control against a failed slope setup, not a pixel-for-pixel comparison
  between two different models.

**T6. Test locomotor assignments and naval transitions (claims 5, 6 and 11).**
- *Setup:* the map of T1 and the route of T3. Named test data: for each run, a test-map override
  section sets the locomotor field of one stock voxel vehicle type that has the driving locomotor
  to another locomotor, in turn walking, mechanical walking, drop-pod, tunnelling and naval.
  Apart from the naval legality controls below, it changes nothing else. In each run the map places vehicles of that type on a shape-1 cell, on a
  shape-5 cell and on flat ground, each at body facings 0 and 8.
- *Procedure:* capture every placed vehicle after placement. Naval run: a further vehicle of the
  type drives T3's route, captured on every tick. Tunnelling run: the vehicle on the shape-1 cell is
  ordered to a cell far enough away that it digs in, and the vehicle on flat ground to a shape-5
  cell far enough away that it travels underground and surfaces there; both are captured on every
  tick.
- *Observable:* body and shadow images.
- *Expected:*
  - walking, mechanical walking and drop-pod: every body shows its flat image, and every shadow
    matches the C4 shadow render of its cell,
  - for every locomotor, the shadow on a slope cell is not the flat footprint already drawn on
    flat ground at the same facing (claim 12), and for the naval and tunnelling locomotors the
    body on a slope cell is not the flat image already drawn (claim 8),
  - naval: on the slope cells the body shows its C4 render from the first capture on (settled at
    placement, claim 6); on the route the body and shadow behave tick for tick as in T3,
  - tunnelling: on the slope cells the body shows its C4 render from the first capture on; while
    digging in and surfacing no slope tilt shows (only the pitch of digging, about the body's own
    lateral axis); once surfaced on the shape-5 cell, the body shows its C4 render from the first
    tick of the surface state, with no intermediate tilts; shadows, while drawn, match the C4 shadow
    render of the cell.
- *Falsified if:* a walking, mechanical walking or drop-pod body is tilted; a drawn shadow is flat
  on a slope cell; the naval vehicle is placed flat on a slope or its route differs from T3; the
  tunnelling vehicle is placed flat on a slope, shows a slope tilt while digging or surfacing, or passes through intermediate tilts after surfacing.
- *If a run cannot be made:* if the game refuses placement or movement, record that as
  inconclusive, not confirmation. For the naval run, named test-map data also gives the
  subject the amphibious movement zone and a nonzero land multiplier on the route's land
  type, without changing its naval locomotor. Verify placement and movement before judging
  tilt. If these controls still cannot make the required pose, that locomotor's affected
  subclaim is `unmeasured` for the demonstrated setup limitation, with medium confidence.

**T7. A powered-down hovering unit (claim 5).**
- *Setup:* the map of T1. A house owns the building on which a stock voxel hovering vehicle type
  depends (a type whose units are switched off when their owner loses that building; stock data
  has one), and one unit of that type, parked on a shape-1 cell at body facing 0, and one on flat
  ground at the same facing.
- *Procedure:* the building is sold or destroyed. Capture and record each tick from before the
  loss until the units have rested at hover height 0 for some seconds. Repeat the flat-ground
  control on a high bridge deck. Restore the control building and record the surviving units
  rising again (S-0503 claims 8 and 10).
- *Observable:* body and shadow images, world position and height, terrain surface height at
  that position, deck state, and current body facing per tick. Compute hover height as in Terms.
  Switch-off may spin the body (S-0503 claim 10), so compare renders at each recorded facing,
  not necessarily the initial facing 0.
- *Expected:* while switched on, the body on the slope shows its flat image. After the switch-off,
  the body still shows the flat image while its height above the surface is positive; from the
  first tick on which that height is 0, it shows the C4 render of shape 1, with no intermediate
  tilts. The shadow matches the C4 shadow render of shape 1 throughout. On the deck control,
  hover height 0 means world height equals terrain surface height plus 416 leptons, not terrain
  surface height alone. Re-enabling removes the slope tilt at once, even before lift has
  raised the unit above hover height 0. This control asserts no slope-tilt rule for a driving
  unit on a bridge deck (claim 6 leaves that separate case open).
- *Falsified if:* the body is tilted while switched on or while its height is positive; it stays
  flat after resting at height 0 above the surface switched off; intermediate tilts appear; the
  shadow is flat; the deck control settles to a different world height; or the slope body
  remains tilted after re-enabling. (If the switched-off unit never comes down to height 0 above the surface, the
  scenario tests only the first half,
  and records the height it keeps.)

**T8. Dropship flag (claim 11).**
- *Setup:* the map of T4. Named test data: a test-map override section sets the dropship flag
  (Import mapping) of one stock voxel aircraft type with the flying locomotor to yes. A second stock
  voxel aircraft type with the flying locomotor keeps its default. Both fly along the alternating
  row of T4 at a steady heading.
- *Observable:* the shadow of each aircraft, one capture per tick.
- *Expected:* the flagged aircraft's shadow is the flat footprint over every cell; the other
  aircraft's shadow switches between flat and shape-1 as in T4.
- *Falsified if:* the flagged aircraft's shadow tilts over a slope cell, or the other aircraft's
  shadow stays flat there. If the flag also changes how the flagged aircraft flies or lands,
  record that; only the shadow is compared.

**T9. Fresh bodies of other locomotors (claim 13).**
- *Principle and test data:* those of S-0105 MS-17. Probe P (cache-disable option, normal index
  85) keeps changing the shade of index 85 as it turns between body facings 8 and 16. Each subject
  gets a single-section test body model with normal mode 1, filled with voxels of normal index 85
  and one colour index *χ*, and keeps caching on, so its body pixels show whether each image is
  fresh (it follows P) or reused (it keeps the row it was made with).
- *Setup:* a flat test map with P and subjects, each given the test model by named test
  data: a stock voxel hovering vehicle type, a stock voxel vehicle of a jumpjet type,
  the tunnelling vehicle of T6, and separate driving and naval subjects. The naval subject
  uses T6's legality controls. Nothing else on the map is drawn as a voxel model.
- *Procedure:* P turns back and forth without stopping. The hovering, jumpjet, driving
  and naval subjects are each hit by a stock weapon whose warhead rocks its target.
  The tunnelling vehicle is ordered far enough away that it digs in. Driving and naval
  subjects also traverse T11's slow shape-pair route while P keeps turning. One capture
  per drawn frame. Use a separate run for each subject and a cache-disabled subject control.
- *Observable:* the palette index of each subject's body pixels, drawn frame by drawn frame, against P's.
- *Expected:* while a subject visibly rocks, changes digging pitch, or runs a driving/naval
  tilt transition, its body's row follows P's row (allowing one drawn frame of lag): each
  image is fresh. Once settled at a previously captured facing, shape and HVA frame,
  its row no longer follows P. The cache-disabled control keeps following P.
- *Falsified if:* a subject's row stays fixed while it visibly rocks, changes digging
  pitch or runs a verified tilt transition, while the cache-disabled control follows P.
  No rocking when hit, indistinguishable probe rows or no accepted movement tests nothing
  for that part and is recorded as inconclusive.

**T10. Current shadow masking and depth-mode isolation (claims 11 and 12).**
- *Setup:* S-0105 MS-27's named two-frame test body, hole-to-filled comparison,
  asymmetric frame marker and shadow-cache-disabled control. Use its skirmish setup,
  speed index 2 and bystander void rule at 800 × 600. Body caching is disabled.
  Shadow caching is enabled except in the control. Run separately for a hovering
  vehicle, a driving vehicle and T6's naval subject, using its legality controls.
  These are engine-conformance runs for the lead-confirmed presentation fixes.
- *Procedure:* capture frame 0, then frame 1 at the same shadow key. Cause rocking
  without changing frame, facing or slope. For driving/naval subjects, also use T11's
  route after another same-type frame-0 vehicle has shown the destination-shape
  footprint at the matching facing. Capture each transition and the settled return.
  Reverse frame order and repeat with a second same-type object sharing the key.
  Use one run per locomotor and part, with a fresh-shadow and a no-shadow control.
- *Observable:* tick-indexed part/shadow images, frame marker, facing, cell shape and
  visible rocking or transition. Judge overlap pixels that lie within the unmasked
  footprint and the current completed own image at the recorded pose.
- *Expected:* filled own-image pixels are undarkened on every appearance, before,
  during and after rocking or a transition, matching both controls. A current hole
  permits qualifying scene darkening even if an earlier frame or another object
  filled that position. No old-pose exclusion persists on returning to settledness.
- *Falsified if:* a qualifying current own-image pixel is darkened, or a current hole
  retains an earlier pose's exclusion. Wrong frame, changed key, absent required pose
  or no qualifying overlap makes that comparison inconclusive. Fresh-shadow overlaps
  can also be checked against the oracle; corrected reuse is checked against the spec.
- *Mode-isolation part:* run S-0105 MS-9's depth-option and non-depth probe comparison,
  adding the above driving/naval transitions and hovering/driving/naval rocking poses.
  Named test data gives the subject the depth-buffer option; retain MS-9's probe models
  and controls. Capture one run per locomotor/pose and control. Expected: each probe
  keeps the look selected by its own type option regardless of the subject's shadow
  freshness or settledness. A non-depth probe acquiring the depth look, or a depth-option
  probe losing it, falsifies isolation. This is engine conformance for S-0105 claims
  32 and 33; retained depths between non-depth draws remain unmeasured as in claim 34.
- *Limit:* neither corrected masking nor mode isolation distinguishes fresh from
  reused footprints, or storage from non-storage, when the key is unchanged. No part
  of T10 is evidence for unsettled-shadow freshness or the unknown cache-miss skip.

**T11. Shape-pair transitions, restarts and rocking (claims 6, 7 and 9).**
- *Setup:* named test files are copies of theater tile files whose plain tile cells
  differ only in their slope codes. Use otherwise passable terrain at a common height
  level, outside the theater's ramp-smoothing ranges, so traffic does not replace the
  test shapes. The deliberately mismatched artwork is not a surface-height test.
  Record the loaded shape of each route cell. Use a long asymmetric voxel body, no
  turret, and two locomotor runs: driving and naval, with T6's naval legality controls.
- *Pair part:* drive across a border from a settled shape to another, allowing at
  least 3 ticks on each side. Cover every ordered pair of distinct shapes 0 … 20,
  including 5 to 9, whose orientations coincide. Record centre cell, facing, tick,
  body and shadow in every drawn frame. Expected: F3 at fractions 0, 1/3 and 2/3,
  then F2's destination tilt, with immediate destination shadow tilt. For 5 to 9 the
  near-equal interpolation case is used, not an angular sweep. Any different visible
  pose or duration falsifies the corresponding pair. Identical pixels cannot decide
  between different arithmetic yielding that image.
- *Restart part:* use consecutive shapes 0, 1 and 4. Named test-map data increases
  only the subject's speed until the recorder verifies that entry into shape 4
  follows entry into shape 1 by 1 or 2 ticks. Start settled on shape 0. Record
  centre cell, facing, tick, body and shadow in every drawn frame.
  Oracle expectation: on the second crossing, the body shows shape 1's quaternion
  orientation, not the partially tilted flat-to-1 pose, then follows F3 from 1 to 4
  for a new 3-tick transition. A continuous continuation from the interrupted pose
  or retention of the first settling deadline falsifies that original-behaviour account.
  Engine-conformance expectation: use corrected-angle endpoints throughout. At the second
  crossing, evaluate the first transition's quaternion at its recorded elapsed fraction,
  use that as the new F3 start orientation,
  and settle to shape 4 three ticks later. A jump to shape 1's full tilt, any image
  inconsistent with that restarted interpolation, or a different settling deadline
  falsifies the corrected rule. Shadow tilt remains immediate in both comparisons.
  If the required crossings are not achieved, record the actual intervals and do not
  judge restart.
- *Rocking part:* on shape 1 and on flat ground, compare driving and naval subjects,
  a powered-down hovering subject, and a jumpjet subject. A named test-map weapon
  rocks the subjects with a small amplitude, without moving or destroying them.
  The recorder reads pitch and roll in radians as well as positions and tick numbers.
  Judge captures with a verified angle at least 0.005 radians and both absolute
  angles at most 0.01 radians. Use matching flat-ground controls at those angles and
  facings, and calibrated long-axis features to separate a slope-sized bias from
  the small rocking. Expected: driving/naval keep the approximately 29.31° slope
  bias in their up axes, whereas hovering and jumpjet match the flat-ground rocking
  control with no slope-sized bias. During a driving/naval flat-to-1 transition,
  arrange a hit so small rocking is still present at fraction 1/3 or 2/3. Expected:
  the up-axis bias is approximately 9.76° or 19.60°, respectively, not zero or the
  full 29.31°. A discrepancy exceeding 2° after projection calibration falsifies
  the appropriate inclusion/exclusion rule. This tolerance tests presence of slope
  tilt, not exact composition of pitch and roll. No qualifying amplitude, missed
  timing, rejected placement or a control that cannot resolve 2° makes that part
  inconclusive, not confirmation. Scripted input can time the hit; it never writes the rocking state.

**T12. Equal-transform reuse classes (claim 8).**
- *Setup:* use S-0105 MS-17's named body and shade probe P. The subject's body uses
  normal mode 1, with out-of-table normal index 85 and colour index χ. P disables
  caching and sets that index's shade at fixed facing 8 or 16. Verify that these
  two probe facings give distinguishable palette rows before judging any result.
  Choose χ so its two rows remain distinguishable after the scene's palette treatment.
  Start with shape 5 and shape 9, supplied by T11's named tile files. Repeat for
  pairs 6/10, 7/11 and 8/12, and for every pair of distinct shapes among 0 and
  17 … 20, whose fixed transforms also coincide. Keep the
  subject's body facing, HVA frame and alternative-model state unchanged in judged
  captures. Test driving, naval, surface-state tunnelling and powered-down hovering
  subjects; also test untilted walking, mechanical walking, drop-pod, teleporting,
  jumpjet and powered hovering subjects, in separate runs. Use T6's locomotor overrides
  and placement controls where needed.
- *Procedure:* place two same-type subjects, one on shape 5 and one on shape 9,
  far enough apart that their images cannot be drawn in the same view. With P at
  facing 8, view and capture only the settled shape-5 subject, making an image with
  row A. Turn P to facing 16 and verify its new row B. Remove P from the view before
  each judged subject capture, so it cannot obscure that subject's shade effect.
  Pan to reveal the shape-9 subject for the first time, then return to shape 5.
  No subject need move; this also permits powered-down hovering and drop-pod tests.
  A recorder verifies the common facing and HVA frame. Run a cache-disabled control.
- *Observable:* subject body palette indices, P's stable row, body facing and frame,
  loaded shape, and settledness. For a repeated pair, replace shape 5 and shape 9
  below by that pair's first and second shapes.
- *Expected:* tilting subjects show A on the first shape-5 capture, B on settled
  shape 9, and A on returning to shape 5. Thus identical transforms still have
  different reuse classes. Every untilted subject keeps A on both shapes: its
  body has no shape component in reuse. Every cache-disabled control follows
  B once P changes, on either shape.
- *Falsified if:* a settled tilting subject keeps A on shape 9 with the unchanged
  inputs, or an untilted subject instead adopts B there. A return that adopts B
  while an independent same-key settled control still shows A also falsifies reuse.
  A changed body frame/facing, global invalidation, loss of the powered-down/surface
  state, inability to return with the same key, or indistinguishable rows makes
  that part inconclusive, not evidence for either reuse rule.

## Open questions

- The corner patterns of the saddle shapes 17 … 20, and which tiles of the stock ramp tilesets
  carry them. The original's surface height and tilt treat all four alike (claim 2), so only their
  images show the pattern. Until the patterns are known this spec proposes no switch for the flat
  saddle surface: an alternative that follows the corners cannot be defined without them.
- Where an airborne unit's shadow lands vertically (also open in S-0105), and which of an
  aircraft's facings turns its shadow (claim 11).
- Which states of the tunnelling locomotor count as its surface state (claim 5). This belongs
  to the vehicle specs. Hovering power and the height used by its test are resolved by
  S-0503 claims 8 and 10.
- Whether a stationary unit reacts to a change of its cell's slope shape, and whether
  a unit on a high bridge deck tilts to the ground cell below it. These cases are not
  fixed by claim 6 or the state table in this revision.
- Whether a cell crossed between two recorded ticks can affect tilt when a unit crosses
  multiple borders in one tick. No rule for that sampling is asserted here.
- Whether any sloped cell exists that a ship can enter with stock data. If none does, a ship's tilt
  state never leaves flat in stock play, and only T6's test data shows claim 6 for the naval
  locomotor.
- Whether the drawn frames of tick *t*<sub>0</sub> already show *p* = 0 (claim 7) or the old settled
  tilt; both look nearly the same.

