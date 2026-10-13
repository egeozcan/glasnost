---
id: S-0106
title: Slope shapes, terrain surface height and voxel tilt on slopes
topic: rendering
revision: 4
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: []
supersedes: []
---

# S-0106: Slope shapes, terrain surface height and voxel tilt on slopes

## Summary

The 21 slope shapes a cell can have, the height of the terrain surface at any point of a cell, and
how a voxel-drawn unit is tilted to match the slope it stands on: the fixed tilt of each shape, the
three-tick transition when the shape under a unit changes, which locomotors tilt the body, and the
tilt of voxel shadows. Together with S-0105 this fixes the body transform *D* of a voxel unit on any
cell when the unit is not rocking. Rocking, and the causes of pitch and roll, belong to the vehicle
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
  used only while a tilt changes (claim 7, Constants table C3). It describes nearly, but not exactly,
  the same rotation as *R*<sub>s</sub>.
- **Tilt state** of a unit with a tilting locomotor: its *current shape* *s*<sub>c</sub>, its
  *previous shape* *s*<sub>p</sub>, and the tick *t*<sub>0</sub> at which the current shape was last
  changed by a transition (claim 6).
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
- **Powered down.** A hovering locomotor is *powered* while its unit is switched on, and also,
  whatever that switch, while its unit has a positive height above the surface (claim 5; which
  height exactly is an open question); otherwise it is *powered down*. A unit is switched off when it is deactivated, for example when its owner loses the
  building that a powered-unit type depends on; which events switch units off and on belongs to
  the vehicle specs.

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
   - Backing: `measurable` (scenario T1) for its sampled positions. Exact arithmetic at
     unsampled positions is `unmeasured`: T1 does not exhaust all in-cell positions for every
     shape; equivalent values at its samples cannot distinguish formulas elsewhere.
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
   with the tilt transform *R*<sub>s</sub> of Formulas F2. Its up axis leans away from the uphill
   direction, so the uphill end of the body is raised. The tilt angles are not those of the surface
   of claim 2 as it is drawn (Terms, level height in model units):

   | Shapes | Tilt of the up axis | Angle of the surface as drawn |
   |---|---|---|
   | 1 … 4 (edge) | 29.3115° | arctan(1/√6) ≈ 22.21° |
   | 5 … 12 (corner and valley) | 21.3574° | 30° |
   | 13 … 16 (steep) | 29.3115° | 30° |
   | 0 and 17 … 20 | none (identity) | 0 / not planar |

   So edge slopes tilt the body more steeply than their surface, and corner and valley slopes less
   steeply (`quirks.slope_tilt_angles`). A corner slope and the valley slope with the same uphill
   direction (5 and 9, 6 and 10, 7 and 11, 8 and 12) give identical tilts. The angles come from
   arctangents taken from a table with a coarse step (Formulas F2), and the rotations use the table
   trigonometry of S-0105 (F1), which is why the tilts are 29.3115° and 21.3574° rather than the
   nominal arctangents (Formulas F2).
   - Backing: `measurable` (scenario T2) for visible transforms. Exact floating-point
     values that give the same captured pixels are `unmeasured`: T2 observes images,
     not transform components, and cannot separate such values.
   - Confidence: high (the transforms reproduce the original's own arithmetic exactly); medium
     for the reading that the angles do not match the surface (it rests on the scales of S-0105 F2
     and F3 and S-0101 claims 1 and 4, through *η* and *ρ*)

### Which bodies tilt

5. **Locomotors.** Whether and how a voxel body is tilted depends on its locomotor:

   | Locomotor | Body tilt |
   |---|---|
   | driving, naval | tilt state with transitions (claims 6 and 7) |
   | hovering | *R*<sub>s</sub> of the cell under the unit, applied at once, only while the unit is not rocking and its locomotor is powered down (Terms: a unit with a positive height counts as powered, so a hovering unit is tilted only once it rests on the surface); a powered hovering unit is never tilted by slopes |
   | tunnelling | *R*<sub>s</sub> of the cell under the unit, applied at once, while the locomotor is in its surface state; in its other states (digging in, underground, surfacing) no slope tilt (the pitch it shows while digging in and surfacing belongs to the vehicle specs) |
   | walking, mechanical walking, drop-pod, teleporting, jumpjet | never tilted by slopes (the jumpjet's rocking aside, claim 9) |
   | flying, rocket | no tilt from slopes |

   "The cell under the unit" is the cell that contains the unit's centre (its height ignored).
   - Backing: `measurable`: scenario T4 for the locomotors that stock data gives to voxel units
     (driving, teleporting, powered hovering, jumpjet, flying, rocket), T6 for the others
     (walking, mechanical walking, drop-pod, tunnelling, naval, given to a voxel vehicle by named
     test data) and T7 for a powered-down hovering unit
   - Confidence: high for driving, naval, walking, mechanical walking, drop-pod, teleporting and
     jumpjet; medium for hovering and tunnelling (which events power a hovering locomotor down
     and which height its test uses, and the states of the tunnelling locomotor, belong to the
     vehicle specs); medium for flying and rocket, whose bodies take their orientation from rules
     of the aircraft specs that this spec does not fix

6. **Tilt state of the tilting locomotors.** On every tick on which the slope shape of the cell
   that contains the unit's centre differs from its current shape, a unit with a tilting locomotor
   starts a transition of 3 ticks, at that tick *t*<sub>0</sub>, from its current shape to the
   cell's shape: from then on its previous shape is the shape that was current, and its current
   shape is the cell's shape. A change while a transition is still
   running starts a new transition from the old *current* shape, not from the orientation drawn at
   that moment, so the drawn tilt jumps (`quirks.tilt_transition_restart_jump`). When the unit is
   placed on the map, its current and previous shapes are both set to the shape of its cell and no
   transition runs, so it starts settled. Whether the unit is moving does not matter.
   - Backing: `measurable` for ordinary transitions (T3) and settled placement (T2's
     placement part and T6). Restarting a transition before it ends is `unmeasured`: T3's
     stock-speed route does not produce two shape changes less than 3 ticks apart, and no
     scenario here forces that case. Independence from motion is also `unmeasured`: no
     scenario here changes the shape under a stationary unit. Tilt on a bridge deck is
     `unmeasured`: no scenario here puts a sloped ground cell beneath an occupied deck.
   - Confidence: high

7. **Drawn tilt during a transition.** In the drawn frames of tick *t*<sub>0</sub> + *j*, for
   *j* = 0, 1, 2, a non-rocking unit's body is drawn with *D* = *M* · Rz(*θ*<sub>body</sub>), the
   product, in that order, of the rotation *M* of the spherical interpolation from
   *q*<sub>s<sub>p</sub></sub> to *q*<sub>s<sub>c</sub></sub> at fraction *p* = *j*/3 (Formulas F3)
   and its 32-step body rotation, as in claim 8. From tick *t*<sub>0</sub> + 3
   on, the body is settled and drawn with *R*<sub>s<sub>c</sub></sub> (claim 8). Because the
   interpolated orientations describe slightly different rotations from the table transforms, two
   small jumps are part of the original's look: at *j* = 0 the old tilt is drawn as
   *q*<sub>s<sub>p</sub></sub> (for an edge slope 29.3555° instead of 29.3115°), and at
   *t*<sub>0</sub> + 3 the tilt snaps from the two-thirds orientation to the table transform. For
   example, driving from flat ground onto an edge slope the drawn tilt is 0°, 9.756°, 19.600°, then
   29.3115°. During the transition S-0105 claim 26 draws every part fresh.
   - Backing: `measurable` (scenario T3) for flat-to-edge transitions and their timing.
     Other shape pairs, F3's near-equal linear case, and exact component values that
     produce identical pixels are `unmeasured`: T3 exercises only flat and shape 1,
     and its captures cannot distinguish pixel-equivalent arithmetic.
   - Confidence: high for the three-tick timing and the interpolation endpoints; high for the
     intermediate rotations (the sample values below follow from F3 and tables C3 and S-0105 F1),
     and medium for which tick's drawn frames show *j* = 0 (whether the drawn frames of tick
     *t*<sub>0</sub> already show the transition, Open questions)

8. **Settled tilt.** A body that is settled (no transition running and no rocking) is drawn with
   *D* = *R*<sub>s<sub>c</sub></sub> · Rz(*θ*<sub>body</sub>), the product of the shape's tilt
   (left factor) and the body's 32-step rotation of S-0105 claim 3 (right factor). The tilt therefore does not depend on the body facing.
   The body's ramp state (S-0105 claim 26) is the shape whose tilt it shows: *s*<sub>c</sub> for
   the tilting locomotors, and the shape of the cell under the unit for a powered-down hovering
   unit and for a tunnelling unit in its surface state. A body drawn without slope tilt by any
   other locomotor, or by a powered hovering unit, has no ramp state. So an image made on one shape
   is never reused on another shape, although shapes 5 … 8 and 9 … 12 give identical images.
   - Backing: `measurable` (T2 for driving, T6 for tunnelling and naval, T7 for hovering)
     for the settled transform and rejection of a flat image on shapes 1 … 16. Reuse being
     separated between shapes with identical transforms is `unmeasured`: identical body
     pixels do not distinguish their reuse classes, and these scenarios include no
     fresh-draw side-effect probe. The absence of a ramp reuse class for untilted bodies
     is `unmeasured` for the same reason.
   - Confidence: high

9. **Rocking.** While a unit with the driving or naval locomotor rocks, its body transform is built
   from its pitch and roll, the tilt of *p* (the table transform when *p* = 1, else the interpolated
   orientation of claim 7) and its body rotation. How pitch and roll arise, and how they combine
   with the tilt, belong to the vehicle specs. The jumpjet locomotor's body shows only its rocking,
   never a slope, and a hovering unit shows no slope tilt while it rocks (claim 5). That a rocking
   body of the driving or naval locomotor is drawn fresh is S-0105 claim 26's claim (claim 13 for
   the other locomotors); this claim adds only that the tilt of *p* stays in the transform.
   - Backing: `measurable` for driving (T2's rocking part). The naval case, and the
     exclusion of slope tilt while hovering or jumpjet units rock, are `unmeasured`: T2
     rocks only a driving vehicle, and T9 rocks the other subjects only on flat ground.
     Retaining an intermediate tilt while rocking is `unmeasured`: no scenario here
     combines rocking with a running transition. Exact composition belongs to the vehicle specs.
   - Confidence: medium

10. **No per-type opt-out.** The vehicle types' imported tilter flag is read (Import mapping) but
    nothing in the original uses it: a vehicle with the flag set to no tilts exactly like one with
    it set to yes (`quirks.tilter_flag_ignored`).
    - Backing: `measurable` (scenario T5)
    - Confidence: medium (no use of the flag was found; an indirect use cannot be ruled out without
      the measurement)

### Shadows

11. **Shadow tilt.** For every locomotor, whenever a voxel shadow is drawn, the shadow transform
    *S* of S-0105 claim 30 is *R*<sub>s</sub> · Rz(*θ*<sub>body</sub>), where *s* is the slope
    shape of the cell that contains the unit's centre (its height ignored), taken at the moment of
    the draw: there is no transition, and the unit's own tilt state is not used. So the shadow of a
    walking, teleporting or powered hovering unit on a slope is tilted although its body is not,
    the shadow of a jumpjet, an aircraft or a rocket in flight is tilted by the shape of the cell
    below it, and while a driving unit's body is in a transition its shadow already shows the new
    shape. One exception: an aircraft with the flying locomotor whose type has the dropship flag
    (Import mapping) always gets the flat shadow (*s* taken as 0). For aircraft, which of the
    aircraft's facings gives *θ*<sub>body</sub> belongs to the aircraft specs (Open questions).
    - Backing: `measurable`: scenario T4 for the driving, teleporting, hovering, jumpjet, flying
      and rocket locomotors, T6 for the walking, mechanical walking, drop-pod, tunnelling and naval
      locomotors, T3 for the shadow during a transition and T8 for the dropship exception
    - Confidence: high for the transform of every locomotor; medium for which drawn objects have a
      shadow at all (a rocket, a tunnelling unit below ground), which S-0105 and the aircraft and
      vehicle specs own, and for the flying locomotor's facing

12. **Shadow images and the cache.**
    - **Ramp state of the shadow.** The ramp state in a shadow's image-cache key (S-0105 claim 31)
      is the *s* of claim 11, for every locomotor (always 0 for an aircraft with the dropship
      flag). So a shadow footprint made on flat ground is never reused on a slope.
      - Backing: `measurable` (scenario T3 and T6: a shadow already made on flat ground at the
        same facing must not reappear on a slope)
      - Confidence: high
    - **Unsettled shadows.** While a driving or naval unit's body is not settled, and while a
      hovering unit rocks, its shadow image is made fresh: it is never taken from the image cache
      and is not added to it, although the shadow transform is the same as when it is settled.
      If eligible under S-0105 claim 31's cloak and no-shadow conditions, such a shadow is always
      drawn, so the skip on a cache miss that S-0105 claim 31 leaves open never applies to it.
      S-0105 claims 31 and 33 include this unsettled-shadow freshness case. For a type with the
      depth-buffer option, a shadow made fresh this way opens a window of S-0105 claim 32 with
      `quirks.depth_mode_leak` on (the default). With that switch off, no shadow opens a window.
      S-0105 claims 32 and 34 own its depth-plane effects. With stock data no type with the
      depth-buffer option has the driving, naval or hovering locomotor (all have the jumpjet
      locomotor, whose shadow stays cached while it rocks), so S-0105 claim 33's statement about
      stock play still holds; the unsettled-shadow case needs data that gives the depth-buffer
      option to a vehicle with one of those locomotors.
      - Backing: `unmeasured`. With the ramp state in the key as above, the key covers every input
        of the footprint, so a fresh and a reused shadow have the same pixels (S-0105 claim 31).
        With `quirks.depth_mode_leak` on (the default), the only observable difference is the
        side effect of a fresh shadow of a depth-option type (S-0105 claim 32). Whenever a shadow
        is unsettled this way, the same unit's body, turret and barrel are drawn fresh as well
        (S-0105 claim 26, and claim 13 here), and a fresh draw of a depth-option part closes the
        window again. Whether the window reaches another
        object's draw therefore depends on a draw order within and across objects that no spec
        fixes yet (S-0105 claim 33).
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

The two angles are values of the original's table arctangent arctan<sub>t</sub>, a tabulated
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

**F3. Interpolated tilt (claim 7).** With *q*<sub>0</sub> = *q*<sub>s<sub>p</sub></sub>,
*q*<sub>1</sub> = *q*<sub>s<sub>c</sub></sub> (table C3) and *t* = *p* stored in single precision:

> *δ* = *q*<sub>0</sub> · *q*<sub>1</sub> (four-component dot product, stored in single precision)
>
> (*k*<sub>0</sub>, *k*<sub>1</sub>) = (1 − *t*, *t*) when 1 − *δ* ≤ 10⁻⁵
>
> (*k*<sub>0</sub>, *k*<sub>1</sub>) = (sin((1 − *t*)·*θ*) / sin *θ*, sin(*t*·*θ*) / sin *θ*), with *θ* = acos<sub>t</sub>(*δ*), when 1 − *δ* > 10⁻⁵
>
> *q* = *k*<sub>0</sub>·*q*<sub>0</sub> + *k*<sub>1</sub>·*q*<sub>1</sub>, each component stored in single precision, not renormalised.

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

At *p* = 0 it is the matrix of *q*<sub>s<sub>p</sub></sub> itself. Floating-point work follows
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

**C3. Tilt orientations** *q*<sub>s</sub> = (x, y, z, w), single precision, dimensionless.

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

**C4. Tilt transforms** *R*<sub>s</sub> (F2), rows listed top to bottom, single precision,
dimensionless. Shapes
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
| *τ*<sub>A</sub> | 0.51163476705551147 | radians | F2 |
| *τ*<sub>B</sub> | 0.37238854169845581 | radians | F2 |
| Interpolation linear threshold | 10⁻⁵ | dimensionless | F3 |
| Rocking threshold | 0.005 | radians | S-0105 claim 26 |

## State and ordering

Tilt state of a unit with a tilting locomotor (claims 6 to 8). *σ* is the slope shape of the cell
that contains the unit's centre.

| State | Event | Result |
|---|---|---|
| any | the unit is placed on the map | *s*<sub>c</sub> = *s*<sub>p</sub> = *σ*; no transition running |
| no transition running, on *s*<sub>c</sub> | a tick on which *σ* ≠ *s*<sub>c</sub> | in transition from the former *s*<sub>c</sub> to *σ*: *s*<sub>p</sub> is the former *s*<sub>c</sub>, *s*<sub>c</sub> is *σ*, *t*<sub>0</sub> is this tick |
| in transition | a tick on which *σ* ≠ *s*<sub>c</sub> | as above: the new transition starts from the old *s*<sub>c</sub> (the drawn tilt jumps) |
| in transition | tick *t*<sub>0</sub> + 3 begins with *σ* = *s*<sub>c</sub> | no transition running, on *s*<sub>c</sub> |

Rocking (claim 9) is not part of this state: the rows hold whether or not the unit rocks. A body
is *settled* in S-0105's sense when no transition is running and it does not rock.

**Observable consequence:** If a unit's centre crosses two cell borders within one tick,
the slope shape in the intervening cell has no effect on its tilt state. Backing: `unmeasured`:
ordinary stock speeds do not produce this crossing, and no scenario here forces it. Confidence: high.

## Edge cases and quirks

All switches default to the original behaviour.

- **`quirks.slope_tilt_angles`.** Original: edge slopes tilt the body by 29.3115° and corner and
  valley slopes by 21.3574° (claim 4), steeper and shallower than their surfaces as drawn. Fixed:
  each shape tilts the body to the angle of its surface as drawn (Terms, level height in model
  units). *R*<sub>s</sub> and *q*<sub>s</sub> are built exactly as in F2 and C3, with the same
  *α*<sub>s</sub>, the same table trigonometry and the same single-precision stores, but with
  *τ* = arctan(1/√6) (0.38759669 rad, 22.2077°) for shapes 1 … 4 and *τ* = arctan(1/√3) = π/6 rad
  (30°) for shapes 5 … 16, both taken exactly in double precision instead of from the table
  arctangent. Lean directions, transitions and shadows are unchanged. Whether the fixed angles
  should instead be the arctangents in leptons is a Question for the project lead.
- **`quirks.negative_level_floor_raised`.** Original: below level 0 the surface is 1 lepton higher
  than 104ℓ (claim 3). Fixed: *B*(ℓ) = 104ℓ for every level.
- **`quirks.tilt_transition_restart_jump`.** Original: a shape change during a transition restarts
  from the old current shape, so the drawn tilt jumps (claim 6). Fixed: the new transition
  interpolates (F3) from the orientation *q* that F3 gives for the interrupted transition at the
  tick of the change, instead of from the tilt orientation of the old current shape.
- **`quirks.tilter_flag_ignored`.** Original: the tilter flag has no effect (claim 10). Fixed: a
  vehicle whose flag is no is never tilted by slopes (its body and shadow drawn as on flat ground).
- **Interpolated endpoints.** The jumps of claim 7 at *j* = 0 and on settling are part of the
  original's look. They come from the difference between C3 and C4 and get no switch of their own.
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
capture are used; no change to a harness is required by this spec.
*Runs:* 1 per scenario, except T6 uses one run per named locomotor. Every comparison is made against values that this spec gives for what the
same run records (positions, cells, facings, ticks), so a second run would test nothing new. Where
an event that a scenario needs may fail to happen, the scenario says what is recorded instead.

*Renders* below are golden images made with S-0105's pipeline and the transforms of this spec at
the stated facing: the *C4 render* uses table C4 (and F3 during a transition), the *fixed render*
uses the transforms of `quirks.slope_tilt_angles` switched off (Edge cases), and the *lepton
render* uses the alternative named in the Questions for the project lead (built as the fixed
render, but with *τ* = arctan(104/256) for shapes 1 … 4 and *τ* = arctan(208/(256√2)) for shapes
5 … 16). A *C4 shadow render* is
S-0105 claim 30's footprint with *S* = *R*<sub>s</sub> · Rz(*θ*<sub>body</sub>). The *flat image*
of a unit is its capture on flat ground at the same facing.

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
  matches the fixed or lepton render instead); a corner and a valley slope with the same uphill
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

**T6. Locomotors that stock data gives to no voxel unit (claims 5, 6 and 11).**
- *Setup:* the map of T1 and the route of T3. Named test data: for each run, a test-map override
  section sets the locomotor field of one stock voxel vehicle type that has the driving locomotor
  to another locomotor, in turn walking, mechanical walking, drop-pod, tunnelling and naval, and
  changes nothing else. In each run the map places vehicles of that type on a shape-1 cell, on a
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
- *If a run cannot be made:* if the game refuses to place a vehicle with a locomotor that its
  type's other fields do not expect, or a vehicle with the naval locomotor does not move over land,
  record that. The rows of claims 5 and 11 for that locomotor then rest on the reading of the
  original alone, and their confidence drops to medium.

**T7. A powered-down hovering unit (claim 5).**
- *Setup:* the map of T1. A house owns the building on which a stock voxel hovering vehicle type
  depends (a type whose units are switched off when their owner loses that building; stock data
  has one), and one unit of that type, parked on a shape-1 cell at body facing 0, and one on flat
  ground at the same facing.
- *Procedure:* the building is sold or destroyed. One capture per tick, and the recorder's height
  of the unit, from before the loss until the unit has rested at height 0 above the surface for
  some seconds.
- *Observable:* body and shadow images, and the unit's height above the surface, per tick.
- *Expected:* while switched on, the body on the slope shows its flat image. After the switch-off,
  the body still shows the flat image while its height above the surface is positive; from the
  first tick on which that height is 0, it shows the C4 render of shape 1, with no intermediate
  tilts. The shadow matches the C4 shadow render of shape 1 throughout.
- *Falsified if:* the body is tilted while switched on or while its height is positive; it stays
  flat after resting at height 0 above the surface switched off; intermediate tilts appear; the
  shadow is flat. (If the switched-off unit never comes down to height 0 above the surface, the
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
- *Setup:* a flat test map with P and three subjects, each given the test model by named test
  data: a stock voxel hovering vehicle type, a stock voxel vehicle of a jumpjet type, and the
  tunnelling vehicle of T6. Nothing else on the map is drawn as a voxel model.
- *Procedure:* P turns back and forth without stopping. The hovering vehicle and the jumpjet are
  each hit by a stock weapon whose warhead rocks its target, and the tunnelling vehicle is ordered
  to a cell far enough away that it digs in. One capture per drawn frame.
- *Observable:* the palette index of each subject's body pixels, drawn frame by drawn frame, against P's.
- *Expected:* while a subject visibly rocks or changes its digging pitch, its body's row follows
  P's row (allowing one drawn frame of lag): each image is fresh. Once it is still again, its row
  stops following P.
- *Falsified if:* a subject's row stays fixed while the subject visibly rocks or changes pitch.
  (A subject that does not rock when hit tests nothing, and that is recorded.)

## Open questions

- The corner patterns of the saddle shapes 17 … 20, and which tiles of the stock ramp tilesets
  carry them. The original's surface height and tilt treat all four alike (claim 2), so only their
  images show the pattern. Until the patterns are known this spec proposes no switch for the flat
  saddle surface: an alternative that follows the corners cannot be defined without them.
- Where an airborne unit's shadow lands vertically (also open in S-0105), and which of an
  aircraft's facings turns its shadow (claim 11).
- Which events switch a hovering unit off and on, which height its powered test uses, and which
  states of the tunnelling locomotor count as its surface state (claim 5). This belongs to the
  vehicle specs.
- Whether a unit on a high bridge deck is tilted by the slope shape of the ground cell under the
  deck. The tilting locomotors take the shape of the cell that contains the unit's centre, which
  for a deck cell is the ground cell below it. No scenario here tests it.
- Whether any sloped cell exists that a ship can enter with stock data. If none does, a ship's tilt
  state never leaves flat in stock play, and only T6's test data shows claim 6 for the naval
  locomotor.
- Whether the drawn frames of tick *t*<sub>0</sub> already show *p* = 0 (claim 7) or the old settled
  tilt; both look nearly the same.

## Questions for the project lead

- **Fixed behaviour of `quirks.slope_tilt_angles`.** The Edge cases propose the angles of the
  surface as drawn: arctan(1/√6) ≈ 22.21° for edge slopes and 30° for the others. The alternative is
  the arctangents in leptons, arctan(104/256) ≈ 22.11° for edge slopes and arctan(208/(256√2)) ≈
  29.88° for the others, taken exactly. Those are the nominal values of the original's two table
  angles *τ*<sub>B</sub> and *τ*<sub>A</sub>, so up to the table's rounding this alternative gives
  edge slopes the angle the original gives corner and valley slopes, and the reverse. Which should
  the switch give?
