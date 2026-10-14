---
id: S-0105
title: Rendering of voxel models
topic: rendering
revision: 13
sources-class: [decomp, format-doc, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0014, Q-0056, Q-0210, Q-0211, Q-0213, Q-0216, Q-0470]
supersedes: []
---

# S-0105: Rendering of voxel models

## Summary

**Revision scope.** Q-0470 adds a 4096-pixel largest side for part and shadow images,
centred copy-rectangle clipping, suppression of non-finite placement and one importer
warning per oversized model. These are lead-confirmed engine rules, tested against
this spec rather than the original. Fifteen lead-classified presentation glitches remain
fixed outright, with no switches. Q-0215 questions 1–6 are already answered by F8.
Question 7's arbitrary-section characterization is explicitly `unmeasured`, not an
implied extrapolation from that table. MS-28 proposes larger mixed-tie discriminators
and states what is still required to close that question.

How a voxel model (a VXL model with its HVA animation) becomes pixels: the fixed viewing
transform, how many facings each model part can show, how the body, turret and barrel are
placed (a building's barrel without a turret included), how the voxel's normal and the global light pick a shade from the voxel lighting table
(VPL), how each voxel becomes a pixel, how the parts are composited, how building turrets
recoil, how the flat voxel shadow is made, and what of earlier draws the voxel buffer keeps. Ramp tilt is left to S-0106; where the finished image
lands among other objects is in S-0102; palettes and house colours are in S-0103; the map's
lighting tint is in S-0104. This spec covers skirmish rendering; campaigns and network games are out of scope.

## Terms

Shared terms (facing, lepton, tick, drawn frame, locomotor, house colour, measurement setup and so
on) are in the glossary. Below, "raw facing" means a facing in the glossary's sense (1/65536 of a
turn per unit), to tell it apart from the coarser facings derived from it.

- **32-step facing** *f₃₂* of a raw facing *r* (S-0204's 32-step facing, its F4 with *n* = 32): *f₃₂ = (⌊(⌊r / 1024⌋ + 1) / 2⌋) mod 32*. This is
  *r / 2048* rounded to the nearest step, halves rounding up, with 32 wrapping to 0.
- **4-step facing** *f₄ = (⌊(⌊r / 8192⌋ + 1) / 2⌋) mod 4* (S-0204's 4-step facing, its F4 with *n* = 4: the same rule, quarter turns).
- **Model frame.** The coordinate frame a voxel model is authored in: +x forward, +y to the
  model's left, +z up. One model unit is one screen pixel before projection.
- **Section.** One independently transformed block of voxels inside a VXL file (VXL calls them
  limbs or sections). Each section has a voxel grid of size *(Nx, Ny, Nz)*, a pair of bounds
  *Bmin, Bmax* (model units), a scale factor and a normal-table mode. This spec uses
  *section* in a different sense from the glossary. The glossary's sense, a named group of
  entries in a data file, is used only where a rules file or rules layer, art file, test map or
  override section is named (the turret anchor in Terms, claim 27, `quirks.barrel_recoil_layer_reset`, the Constants table, Import mapping, Measurement
  scenarios).
- **HVA frame.** One set of per-section 3×4 transforms in the companion HVA file.
- **Frame** (in this spec). Qualified by a part or given a number ("body frame", "turret frame",
  "frame 0", "frames 0–7", "frame key"), *frame* always means an HVA frame. A screen redraw is
  always called a *drawn frame*. *Model frame*, *parent frame*, *local frame*, *part frame* and
  *turret-frame point* refer to coordinate frames.
- **Model part.** One VXL+HVA pair drawn as a unit: body, turret, barrel, or an alternative body.
- **Draw transform.** The 3×4 matrix that maps a part's model frame into view space for one draw.
- **View space.** After the view transform: *x* to the screen's right, *y* upward on screen
  (screen rows are *−y*), *z* toward the viewer (larger *z* is nearer).
- **voxel image** (also called the *voxel buffer* below). The palette-index image produced for
  a part before placement on screen. Palette index 0 is transparent. Its signed integer
  coordinates do not wrap. Its logical extent accounts for every rendered voxel and adjacent
  pixel (claims 22 and 24), without prescribing a storage size or layout. Only claim 22's
  bounded window can reach the screen.
- **Shade index.** A lighting level 0–31 chosen per voxel normal; it selects one 256-entry row
  of the VPL.
- **VPL.** The voxel lighting table file: for each shade index and each voxel colour index, the palette index to draw. Claim 38 gives how its content is used; its byte layout is given by S-0013. A *row* is the 256 entries of one shade index.
- **Table trigonometry.** Sines and cosines quantised to 1/8192 of a turn and taken from fixed
  single-precision values instead of exact functions; formula F1. It matters because every
  rotation in this spec uses it.
- **Pixels per lepton** *ρ*: the conversion from game distance to model units (see Constants and F3).
- **Image reuse, image cache.** The reuse of an earlier finished image when the draw inputs
  belong to the same equivalence class (claim 26). A *key* names that class, not a required
  data structure. Reuse crosses objects of the same type; no storage organisation is prescribed.
- **Fresh draw.** A draw that makes a part image or a shadow image from the model, as opposed to
  one that reuses an image from the image cache (claims 26, 31). Only fresh draws give shades to
  normal indices (claim 20), switch the depth-buffered mode (claim 32) and use and clear the
  voxel buffer (claim 34).
- **voxel depth** (also called the *depth plane* below). The depth associated with a voxel-image
  pixel when claim 24's depth comparison applies. Its integer range and comparison are specified
  there. It need not be stored beside the colour image.
- **Colour plane, depth plane.** Colour-image and comparison-depth effects, not required
  storage layouts. Claim 34 gives fresh-draw initialisation. *Empty comparison depth*,
  denoted 0 when discussing clearing, is a sentinel below all numeric voxel depths,
  including numeric depth zero; logically it compares as −∞ (claim 24).
- **Recoil cycle, recoil slide.** A building turret's or barrel's slide back and return after a
  shot (claim 27); its recoil slide *λ* is how far back the part currently sits, in model units
  (F7).
- **Column.** The cells of a section that share grid indices *(i, j)*: one vertical line of the
  grid.
- **Column data.** The column's *value sequence*: the integers 0 … 255 that the public model
  format stores for one column, taken in that format's own order, between the start and the end
  the file gives for that column. In a standard file the sequence encodes the column's runs and,
  for each voxel, its colour index and its normal index, laid out as the public model format gives.
  This spec does not restate that layout. A column the file marks as empty has no column data.
- **Run.** The file describes each column as a sequence of runs, each a number of consecutive
  empty cells or a number of consecutive voxels.
- **Draw point.** The screen point a part image is placed around: the object's position (S-0101)
  plus any shift its locomotor applies when drawing (claim 25).
- **Shadow draw point.** The screen point a shadow image is placed around: the point on the
  ground below the object that its locomotor gives (S-0102, S-0106; open questions).
- **Aircraft-like type.** A type with the aircraft-like flag (import mapping); its shadow is
  half size (claim 31).
- **Depth-option type.** A type with the per-type depth-buffer option (claim 24).
- **Ramp.** A sloped cell (a *slope* in the glossary's sense). A vehicle on a ramp is drawn tilted to match it; the tilt, and the
  per-cell slope that selects it, are left to S-0106. Here it matters only as an input of the body
  transform and of the image-cache key. A body's tilt is *settled* when it is the fixed tilt of the
  cell it stands on and is not changing. While the body moves from one tilt to another, or while its
  locomotor rocks it (pitches or rolls it by 0.005 radians or more), the tilt is not settled.
- **Special alternative-body type name.** The exact, case-sensitive type name singled out
  by S-0007 claim 16. It is the glossary's *type name*, not a registration index (claim 12,
  MS-11). S-0007 owns the name and the file lookup.
- **Walk-animation counter.** A per-vehicle counter that advances as the vehicle moves; it picks
  the body's HVA frame (claim 16). When it advances belongs to the vehicle specs.
- **Animation-frame counter.** A per-object counter of the object's current animation frame. It
  picks a building turret's HVA frame, and a vehicle turret's frame in one case (claim 16). When it
  advances belongs to the vehicle and building specs.
- **Turret anchor.** The screen point at which a building's voxel turret and barrel images are
  centred: the building's draw point moved by the building type's turret-animation x and y
  displacements, in whole screen pixels (x to the right, y downward). Both are fields of the
  building's rules section, not of its art (import mapping; S-0108 reads the same two fields for
  its turret slot animation), and both are 0 when absent (claims 9 and 37).
- **Cloak state.** Each object is in one of four cloak states: *uncloaked*, *cloaking* (fading
  out), *cloaked*, *uncloaking* (fading in). Only *uncloaked* counts as fully visible here
  (claim 31). Which events change the state belongs to the special-abilities specs.
- **Turn rate.** A facing that turns toward a target facing does so at a turn rate *ω*, an integer
  (values above 127 count as 127). The turn's duration and per-tick step are integers (formula
  F9): a turn of *Δ* facing units (the shorter way round) takes *Θ = ⌊|Δ| / (256 ω)⌋* ticks and
  moves by *trunc(Δ / Θ)* facing units per tick, so the facing does not move by exactly *256 ω*
  per tick. A turn shorter than *256 ω* facing units completes at once. S-0206 owns the full
  turning rule and how a type's turn-rate field becomes *ω* (its claims 1 and 6; its F1 is the
  same rule as F9); this spec uses it only for the barrel elevation (claim 6), whose rate
  S-0206 lists among its constants.
- **Spawned aircraft, docked.** Some types carry aircraft of their own, which they launch and
  recover (the *spawner* and its *spawned aircraft*; which types carry them, and how launch and
  recovery work, belong to the specs for spawned aircraft). A spawned aircraft counts as *docked*
  while it is inside its spawner, whether waiting there or reloading there. One that is taking
  off, attacking or returning is not docked, and neither is one that has been destroyed and not
  yet replaced.
- **Alternative body.** A second body model that a type can have besides its main body (file
  naming: S-0007, claim 16). It is drawn only through the alternative-body option (claim 12,
  import mapping).
- **Alternative-model state.** On while the alternative-body option is enabled and none
  of the vehicle's spawned aircraft is docked (claim 12), even when missing alternative
  art makes it show its main body instead. Otherwise off. It is a separate cache-key
  component (claim 26).
- **Multi-turret type.** A vehicle type whose imported turret count is above 0 and that does not
  have rotating-barrel weapon stages (import mapping). Such a type has one turret model, and
  optionally one barrel model, per turret index 0, 1, … (file naming: S-0007).
- **Selected turret index.** A per-object integer that says which of a multi-turret type's turret
  models is in use. What sets it (for most such types the weapon currently chosen, for some the
  progress of reloading) belongs to the vehicle and combat specs.

## Behaviour

### The view

1. **One fixed orthographic view.** Every voxel part is drawn with the same view transform
   *V = Rx(−60°) · Rz(−45°)* (formula F2), applied after the part's own transform. There is no
   perspective; there is no camera variation with screen position.
   - Backing: `measurable` (scenario MS-1)
   - Confidence: high

2. **Model forward axis.** A part whose body facing is east (*f₃₂ = 8*) is drawn unrotated: its
   model +x points east on the map (screen right-and-down), model +y points north (screen
   right-and-up), model +z points up.
   - Backing: `measurable` (MS-1)
   - Confidence: high

### Facings per part

3. **Body: 32 facings.** The body rotation is the body's 32-step facing, not its raw facing:
   angle *θ_body = −(π/16) · (f₃₂ − 8)* about model +z. A unit turning smoothly therefore shows
   at most 32 distinct body orientations on flat ground. (Ramps and pitch add tilt, S-0106.)
   - Backing: `measurable` (MS-1)
   - Confidence: high

4. **Vehicle turret: 32 facings, relative to the body.** The turret is rotated relative to the
   body by *−(π/16) · (t₃₂ − b₃₂)*, where *t₃₂* and *b₃₂* are the 32-step facings of the turret
   and the body. Its absolute orientation is thus also one of 32.
   - Backing: `measurable` (MS-2)
   - Confidence: high

5. **Building turret: 32 facings, absolute.** A building's voxel turret is rotated by
   *−(π/16) · (f₃₂ − 8)* from the building's (single) facing.
   - Backing: `measurable` (MS-6)
   - Confidence: high

6. **Barrel elevation: 32 steps.** A barrel part is pitched about the turret's lateral (y) axis
   by *+(π/16) · (e₃₂ − 8)*, where *e₃₂* is the 32-step value of the object's barrel-elevation
   facing (raw 16384 = level, smaller values raise the muzzle). When a vehicle or other
   non-building object is placed on the map, its elevation starts at 16384 and is then turned
   toward *(16384 − 256 × A) mod 65536*, where *A* is the type's imported fire angle modulo 256,
   at a fixed turn rate of 3 (see *Turn rate* and F9: a turn of *Δ* facing units takes
   *⌊|Δ| / 768⌋* ticks). With the absent-key default *A = 8* the turn is 2048 facing units and
   takes 2 ticks: the elevation's 32-step value is level after the first tick and one step
   (11.25°) above level from the second tick on (`quirks.barrel_elevation_quantised`). For an illustrative type whose imported fire-angle field is *A = 20*, the turn is 5120
   facing units and takes 6 ticks at 853 facing units per tick; the 32-step values 0, 1, …, 6
   ticks after placement are 8, 8, 7, 7, 6, 6, 6 (F9). This is an example field value, not stock data. Every appearance shows the current
   32-step elevation, including reused images, because barrel keys include elevation
   (claim 26; a lead-confirmed departure checked by MS-10 engine conformance). Buildings re-aim the elevation
   when firing; how they choose it belongs to the building combat specs, and this spec only fixes
   that the shown pitch is the 32-step value of that elevation.
   - Backing: `measurable` (MS-2 and MS-14 for vehicles, MS-6 for buildings)
   - Confidence: medium (the shown pitch); high (the turn timing of F9)

### Part placement

7. **Vehicle body transform.** *D* is the body transform its locomotor gives, without the view
   (rotation by *θ_body* on flat ground, plus tilt, S-0106). The body is drawn with *V · D*.
   - Backing: `measurable` (MS-1)
   - Confidence: high

8. **Vehicle turret placement.** The turret transform is *T = D · X(d) · Rz(θ_rel)*, where
   *X(d)* is a translation by *d* along +x of the current local frame (F2). The turret's origin
   sits *d = δ × ρ* model units along the body's forward axis from the body's origin (*δ* is the
   imported turret displacement in leptons, positive forward), and the turret turns about that
   point by the relative turret angle (claim 4). It is drawn with *V · T*.
   - Backing: `measurable` (MS-2)
   - Confidence: high

9. **Building turret placement.** The building turret transform is *T = Rz(θ) · X(d_b)*, with *θ*
   the turret angle of claim 5 and *d_b = trunc(δ / 8)* model units (integer division, truncated
   toward zero). The turret's origin sits *d_b* along the turret's own forward axis, so as the
   turret turns its origin circles the turret anchor at distance *|d_b|*. For example *δ = 20* gives 2 model units and
   *δ = −12* gives −1. Note the different unit from vehicles (`quirks.building_turret_shift_eighths`). The turret and
   barrel images are centred at the building's turret anchor (Terms), so changing the anchor's x
   and y displacements moves both images by exactly that many screen pixels and changes nothing
   else in them.
   - Backing: `measurable` (MS-6, including its anchor run)
   - Confidence: medium (high for the anchor shift)

10. **Barrel placement.** The barrel transform is *B = T · Ry(φ)*, where *T* is the
    turret transform before the view and *φ* is claim 6's elevation. Pitch about the
    lateral y axis through the turret origin; draw with *V · B*. Building recoil adds
    claim 27's displacement without changing this pivot.
    - Backing: `measurable` (MS-2 and MS-6; lead-confirmed pivot departure,
      tested against this spec rather than the original)
    - Confidence: high

11. **Turret and barrel order.** The turret and barrel are separate images; whichever is drawn
    second covers the first where they overlap. If the turret's 4-step facing is 1 or 2 (pointing
    roughly east or south), the turret is drawn first and the barrel over it; if it is 0 or 3,
    the barrel is drawn first and the turret over it. Vehicles and buildings follow the same rule.
    The body is always drawn before both, so the turret and the barrel each cover the body where
    they overlap it.
    - Backing: `measurable` (MS-2 for vehicles, MS-6 for buildings)
    - Confidence: high

12. **Alternative and multiple models.** A vehicle whose type has the alternative-body option
    draws its alternative body instead of its main body whenever none of its spawned aircraft is
    docked (Terms), and its main body while at least one is. If a type has the option but carries
    no spawned aircraft, the original's result is unspecified; our engine draws the main body,
    and the importer warns about the type (Import mapping, *Importer warnings*).
    - **Which alternative body.** It is the one S-0007 claim 16 loads for the type: the body with
      the no-spawn-alternate suffix or, for the exact type name that S-0007 names, the body with
      the suffix S-0007 gives for that name. When both files exist, the no-spawn-alternate one is
      used.
    - **No option, no alternative body.** Without the alternative-body option no alternative body
      is ever drawn. A vehicle with the type name S-0007 singles out but without the option always shows
      its main body, even when its alternative body has loaded. So for a type without the option,
      S-0007's `quirks.apc_water_body` changes only what is loaded, not what is drawn.
    - **Option without a file.** Draw the main body whenever no alternative body is
      available, including types with the turret flag, for which S-0007 loads no
      alternative body. Draw the turret normally, never as a substitute body. The
      alternative-model state remains a separate key component (Terms, claim 26).
      This fallback is a lead-confirmed departure checked by MS-11 engine conformance.

    A vehicle of a multi-turret type draws
    the turret model and barrel model of its selected turret index instead of the type's ordinary
    turret and barrel. A type with rotating-barrel weapon stages always draws its ordinary turret
    and barrel, whatever its turret count. File naming is S-0007.
    - Backing: `measurable` (MS-11)
    - Confidence: high (missing-body fallback and no alternative without the option),
      medium (remaining model choices)

### Sections and animation

13. **Section transform.** Each section *i* of a part is drawn with *M · H(f, i)*, where *M* is
    the part's draw transform (view included) and *H(f, i)* is the HVA transform of section *i* in
    frame *f mod F* (*F* = the HVA's frame count). Every section of the VXL is drawn. The VXL
    file's own per-section transform is not used.
    - Backing: `measurable` (MS-3 for the HVA, MS-13 for the unused file transform)
    - Confidence: high (HVA use), medium (file transform unused)

14. **HVA translation scale.** Every section is drawn with the translation column of its HVA
    transforms multiplied by the scale factor of the VXL's **first** section, whatever the
    section's own factor (`quirks.hva_scale_first_section`). Each product is a single-precision value truncated toward zero (F0).
    - Backing: `measurable` (MS-13)
    - Confidence: high

15. **Position of a voxel inside a section.** The voxel at grid index *(i, j, k)* occupies the box from
    *Bmin + (i, j, k) ⊙ s* to *Bmin + (i+1, j+1, k+1) ⊙ s*, with
    *s = (Bmax − Bmin) ⊘ (Nx, Ny, Nz)* (component-wise). The bounds therefore stretch or shrink
    the grid. The voxel is drawn at one corner of its box: per axis, the corner on the same side
    as the section's farthest bounding-box corner (claim 21). Viewed normally (top facing the
    viewer) that is the lower corner in z.
    - Backing: `measurable` (MS-13)
    - Confidence: medium

16. **Animation frames.** A vehicle body uses HVA frame *f_b = ν mod F_body*, where *ν* is the
    unit's walk-animation counter and *F_body* the body HVA's frame count. Whenever *f_b ≠ 0*, the
    vehicle turret's frame is *f_b* itself, not *f_b* reduced modulo the turret's frame count.
    When *f_b = 0*, the turret's frame is *u mod F_tur*, where *u* is the unit's animation-frame
    counter and *F_tur* the turret HVA's frame count, or frame 0 if the turret has no HVA. (The
    HVA transforms drawn for a turret frame at or beyond *F_tur* are those of that frame modulo
    *F_tur*, by claim 13.) A building turret uses the building's
    animation-frame counter modulo the turret HVA's frame count. All "mod" here are non-negative
    remainders of non-negative counters. **Barrels drawn together with a turret, of
    vehicles and buildings alike, are always drawn in HVA frame 0** (`quirks.barrel_frame_zero`). A
    building's barrel drawn without a turret uses the building's animation-frame counter instead
    (claim 37). The frame used by the turret models of
    a multi-turret type (claim 12) has not been determined; until it is, our engine gives them the
    vehicle turret frame above. The animation-frame counter is 0 when the
    object is created, and in the original it changes only through the object's attacking
    (firing, and, for a type with rotating-barrel weapon stages, while those stages are spun up). The exact rule belongs to the
    combat, vehicle and building specs. So a vehicle or building that has never attacked shows
    turret frame 0 whenever its turret frame comes from this counter.
    - Backing: `measurable` (MS-3 and MS-3b for the body, the barrel and *f_b ≠ 0*; MS-3c for
      *f_b = 0* and building turrets). The multi-turret frame is not a claim yet: the original's
      rule has not been determined (open questions), so there is nothing to measure against.
      The exact mapping from animation-frame counter u to a vehicle turret at f_b = 0 or
      a building turret is `unmeasured`: MS-3c observes changing frames but does not record u,
      so it cannot distinguish the precise modulo mapping. The counter's initial zero and
      attack-only changes are the parts tested there.
    - Confidence: high (barrel frame 0, body frame), medium (turret frame, and the counter
      changing only through attacking)

### Lighting

17. **Light direction.** The light is fixed for the whole game. In view space its direction
    vector is *L = (−0.49999943, −0.70710599, 0.49999943)*. Expressed in the model frame of an
    unrotated part (claim 2: *x* toward map east, *y* toward map north, *z* up) it is about
    *(0.2013, −0.9101, −0.3622)*. *L* is the direction a surface normal must point in to get the
    largest diffuse term (*a = n · L′* in F4), so for an unrotated part straight-up normals get no
    diffuse light. The specular term changes where the peak is. By F4 with the exact light and
    eye vectors, the highest shade index of an unrotated part (about 26) goes to normals pointing
    map south and about 20° above the horizon. A normal along *L′* gets about 20, and a
    straight-up normal about 4. Table normals land near, not exactly on, these values. The map's ambient light and colour tint do not change this direction (S-0104).
    - Backing: `measurable` (MS-4)
    - Confidence: high

18. **Shade per normal.** For each fresh draw of a part, the shade index of each normal index is given
    by formula F4, from:
    - the light brought into the part's model frame, *L′ = Rᵀ L*, where *R* is the rotation part
      of the part's draw transform (view included, HVA **not** included, `quirks.voxel_light_ignores_hva`),
    - a fixed "eye" vector *E = (0, 0, 1)* taken in the **model** frame (`quirks.voxel_eye_in_model_frame`),
    - the normal table selected by the normal mode of the part's **first** section (`quirks.voxel_light_first_section_mode`),
    - a fixed specular sharpness *κ = 3*.

    Every voxel of every section of that part takes the shade index that this draw gives its
    normal index (indices the table does not cover: claim 20).
    - Backing: `measurable` (MS-4 for stock models; MS-15 for the first-section mode and the
      light without the HVA)
    - Confidence: high

19. **VPL application.** The pixel written for a voxel with colour index *χ* and shade index *q*
    is *VPL(q, χ)*, the entry for colour *χ* in row *q*, a palette index. That index then goes
    through the object's palette and house colour at copy time (S-0103). The VPL is loaded once at start-up from the voxel lighting
    table file (claim 38 gives how the file's content is used). The file's remap range and extra value are not consulted
    when drawing: a pixel needs only the row and entry. A voxel whose *VPL(q, χ)* is 0 is still
    written: it sets its pixel to transparent, erasing whatever an earlier voxel left there, and in
    the depth-buffered mode it is tested and stores its depth like any other voxel (claim 24).
    - Backing: `measurable` (MS-4 for the lookup; MS-23 (a) for voxels whose entry is 0,
      MS-23 (h) for the depth such a voxel stores, and MS-23 (i) for the table being read once,
      at start-up)
    - Confidence: high

20. **Normal indices outside the table.** Normal indices 253, 254 and 255 always get shade 16.
    An index at or above the size of the part's normal table but below 253 is not given a shade
    by the draw; it shows the shade that index was last given by an earlier fresh draw (`quirks.voxel_stale_normal_shades`):
    - Only fresh draws give shades. A reused image (claim 26) gives none.
    - Fresh draws of voxel parts give shades by F4, to the indices their own table covers.
    - Projectiles and debris drawn as voxel models also give shades to the indices their table covers, by a
      rule that belongs to their own specs.
    - An index that no draw has given a shade since the program started has shade 0. Starting a
      new game does not reset the shades.
    - Backing: `measurable` (MS-16; its game-change run for shades kept across games and its
      projectile run for projectiles). Debris giving shades is `unmeasured`: that run creates
      a projectile, not voxel debris, and no scenario here controls a debris draw without other
      fresh models also changing the shades.
    - Confidence: high (shade 16, the stale value, no reset at a new game), medium (shade 0 at
      program start, the shades given by projectiles and debris)

### Rasterisation

21. **Pixel priority inside a section.** The section's farthest bounding-box corner is the
    one of the eight box corners with the smallest view-space *z*. On an exact tie, the first
    in this list is used: on the bottom face (max x, max y), (max x, min y), (min x, min y),
    (min x, max y), then the same four on the top face. Count each voxel's grid position from
    that corner as *n_x, n_y, n_z* integer grid steps along the three axes. Its priority tuple is
    *(n_y, n_x, n_z)*, compared lexicographically: compare *n_y*, then *n_x* when *n_y* ties,
    then *n_z* when both tie. These are exact integer comparisons, with no rounding.
    - **Without depth buffering.** Of the section's voxels landing on one image pixel, the
      greatest tuple supplies that pixel's palette index. A winning value of 0 makes the
      pixel transparent even if a lower-priority voxel supplied a nonzero value (claim 19).
    - **With depth buffering.** Claim 24's strict comparison and untested right-hand writes
      determine coverage. At a common left pixel, equal nonzero comparison depths favour
      the smallest tuple, provided the depth exceeds the initial stored depth and no
      right-hand write interferes. An accepted right-hand write of a greater tuple can
      replace a smaller tuple's contribution at that neighbouring pixel even when the
      replaced depth is greater. Thus the painter winner rule is not the depth-mode tie rule.

    These are coverage requirements, not a required traversal of rows, columns or voxels.
    An implementation may organise its work differently if the resulting colours and depths
    agree. A bottom-face farthest corner gives bottom-up grid positions; a top-face corner
    gives top-down positions. In a well-formed file every column's runs add up to exactly
    *Nz* cells, including the empty cells above its top voxel. Claim 36 covers columns that
    break this (`quirks.voxel_topdown_column_origin`).
    - Backing: `measurable` (MS-5 and MS-5c) for overlap priority and both positioning
      directions on the supplied models; MS-5c isolates the within-section depth tie.
      The exact farthest-corner tie list is `unmeasured`: neither scenario controls equality
      of bounding-box corner depths, so neither distinguishes that tie priority.
    - Confidence: high

22. **One pixel per voxel; no image-coordinate wrap.** Outside depth mode, each
    voxel writes exactly one pixel at F5's position. Magnification shows gaps
    (`quirks.voxel_single_pixel`). The logical image accounts for every rendered
    voxel and right-hand pixel, including out-of-box voxels from claims 35 and 36.
    No coordinate wraps or clips at an artificial 256-pixel edge.
    - **Bounded images (Q-0470).** Our engine's largest image side is 4096 pixels,
      for part images and shadow images alike. When either side of claim 25's copy
      rectangle exceeds that limit, keep its intersection with a 4096 × 4096-pixel
      window centred on the copy rectangle, as F6 defines. Pixels outside the window
      are not drawn. There is no rescaling or change in the retained pixels' screen
      positions. Coverage inside the window still follows claims 21–24, including a
      right-hand write from a voxel whose left pixel is outside it. A copy rectangle
      with both sides at most 4096 pixels is unchanged. Well-formed stock models never
      reach the limit.
    - **Non-finite placement.** A part whose placement inputs are not finite is not
      drawn: this includes its section bounds, applicable transforms and derived
      projection or copy-rectangle values containing infinity or a value that is not
      a number. Apply the same rule to a shadow image's placement inputs. Suppression
      affects only that image, not the other finite parts of the object.
    - **Warning.** The importer warns once per model whose part or shadow images
      would exceed the limit (Import mapping). The limit and suppression are
      unconditional lead-confirmed departures, with no quirk switch.
    - Backing: `measurable` (MS-5/MS-13 for pixel size; MS-24 engine conformance for
      the lead-confirmed no-wrap departure; MS-29 engine conformance for the
      lead-confirmed limit, placement suppression and warning, not oracle measurements)
    - Confidence: high

23. **Section overlap priority.** Outside depth-buffered mode, the sections of one part have
    ascending overlap priority by nearest view-space depth (the largest *z* of their transformed
    bounding box): a section with greater nearest depth covers one with smaller nearest depth
    wherever their pixels coincide. Sections with exactly equal nearest depth need not retain
    file order (`quirks.voxel_section_sort_ties`). F8 states the covered configurations,
    including an all-equal group of any size; it does not infer a general mixed-depth tie rule
    from those examples. There is no depth test
    between sections, so a farther voxel in the higher-priority section can cover a nearer voxel
    in the other section.
    - Backing: `measurable` (MS-5 for distinct depths; MS-5b and MS-25 for F8's tie configurations
      and the dependence on the full section sequence).
      A general priority relation for mixed-depth ties outside F8 is `unmeasured`:
      the listed examples do not distinguish all possible full-sequence relations, and
      no arbitrary-section coverage characterization is established here. MS-28 is the
      proposed discriminating investigation; this is a specification gap, not an
      assertion that those pixels cannot be observed.
    - Confidence: high for the stated priorities and input sufficiency; low for any
      extrapolation beyond the covered configurations

24. **Per-type depth buffering.** A type with the depth-buffer option draws differently in four
    ways. Claim 32 confines this mode to fresh depth-option part draws.
    - **Section ties and depth.** Distinct-depth sections contribute nearest first. For F8's
      configurations, equal comparison depths at a coincident pixel favour the section of
      highest overlap priority there: reversing the priorities makes that section contribute
      first, and the strict depth test retains it. F8 also lists the contributing order for
      those configurations. Within-section coverage follows claim 21's depth-mode rules,
      not its greatest-tuple painter winner.
    - **Depth test.** A voxel's comparison depth is the unwrapped signed integer
      *d = ⌊pz / 256⌋*, with *pz* from F5, one model unit per depth level. Before each
      fresh depth-mode draw, every coordinate is empty, below all numeric voxel depths
      (claim 34). Clearing depth to 0 denotes that empty sentinel, logically −∞,
      not numeric voxel depth zero. A voxel passes an empty coordinate or one whose
      stored numeric depth is strictly smaller. Zero and negative depths therefore
      pass empty coordinates, and wrapping never reverses depth order. Equal depths
      retain the first left-pixel contribution unless a right-hand write interferes.
    - **Two pixels per voxel** (`quirks.voxel_depth_double_width`). A voxel that passes the test writes its colour (0 included, claim 19) and its
      depth into its own pixel and into the pixel immediately to its right. Only the left pixel's
      stored depth is tested: the right pixel is overwritten even if it held a greater depth. At
      every image column, including 255, the right-hand pixel is *(x + 1, y)*,
      never column 0 of another row. Claim 22's retained window limits which pixels
      reach the screen, not their coverage or comparison-depth result inside it.
    - **Independent coordinates.** Horizontal arithmetic never carries into vertical
      position in either mode (F5).
    - Backing: `measurable` (MS-5/MS-12/MS-25 for coverage; MS-12/MS-18/MS-24
      engine conformance for lead-confirmed depth, clearing and coordinate departures)
    - Confidence: high

25. **Copy to the screen.** After all sections are drawn, the bounding box of the transformed
    section boxes (in view-space pixels, *y* flipped) is enlarged by a 4-pixel margin on every
    side to give the copy rectangle. Its retained window under claim 22 is copied to
    the screen around the object's draw point (formula F6). The draw point
    comes from the object position (S-0101) plus any shift its locomotor applies. Palette index 0 in
    the voxel buffer is transparent.
    - Backing: `measurable` (MS-1 for ordinary placement; MS-29 engine conformance
      for claim 22's retained-window limit)
    - Confidence: high

26. **Image reuse.** Finished part images are kept in the image cache of the object's type and
    reused, by every object of that type, whenever a later draw produces the same key. The key is
    composed of the following independent components:
    - **Body:** the ramp state (which settled tilt the body has, for locomotors that tilt the
      body on ramps, S-0106), the 32-step body facing, the alternative-model state, and the HVA
      complete applicable frame number.
    - **Vehicle turret:** the ramp state, the 32-step body facing, the 32-step turret facing, the
      selected turret index for a multi-turret type, the alternative-model state and
      the complete applicable turret-frame number as separate components. Adjacent
      frames and distinct states never collide. When the turret displacement is 0,
      the body facing is left out of the key. A turret
      on the body's origin then has the same orientation for every body facing with the same
      turret facing. On flat ground, and on a settled ramp, the reused image therefore differs
      from a fresh one by at most the 0.088° rounding of F1, which rarely moves a pixel. (On a ramp
      the tilt is part of the key. On a settled ramp the body transform is the cell's tilt
      followed by the body's 32-step rotation, so the tilt does not depend on the body facing.
      The tilt itself is left to S-0106.)
    - **Building turret:** the 32-step turret facing and complete applicable turret frame.
    - **Barrel:** the turret's key (for a building's barrel drawn without a turret, claim 37
      gives the key), plus the current 32-step barrel elevation *e₃₂*. Barrel images
      stay apart from turret images and reuse only at the same elevation. Turret keys
      themselves acquire no elevation component.

    A part image is made in a fresh draw, and neither looked up nor stored, in any of three cases:
    - the type has the cache-disable option (body, turret and barrel alike),
    - for a building's turret and barrel: while the turret's or the barrel's recoil cycle
      (claim 27) is running (not for a barrel drawn without a turret, claim 37),
    - for a vehicle with the driving locomotor or the naval locomotor (the one ships use): while
      its body's tilt is not settled (Terms: while it moves between tilts, or while it is pitched
      or rolled by 0.005 radians or more; the angles are compared in floating point). Body, turret and barrel are then all drawn fresh. When a body tilts or
      rocks is left to S-0106 and the vehicle specs. The three other locomotors whose bodies are
      drawn fresh (hovering while it rocks, jumpjet while it rocks, tunnelling while its digging
      pitch changes) are S-0106 claim 13's.

    **Independent reuse and lifetime.** Reuse is decided independently for each part and shadow:
    a fresh body does not require a fresh turret. Changed elevation may require a fresh
    barrel alongside a reused turret. Each part uses its own complete key. An earlier image can remain visible across drawn frames.
    The original can invalidate all earlier images together, making later appearances fresh
    again, but the event that triggers this is not specified here. Our engine never invalidates
    images during a game and invalidates them at game start (a departure, with no switch).
    Whether a new game invalidates images in the original remains open. No capacity, allocation
    scheme or storage lifetime is required.

    **What reuse changes on screen.** Because only fresh draws have side effects, every key
    component shows: a draw with a key not yet in the cache is fresh, and a draw with a known key
    is not. The visible consequences are:
    - distinct applicable frames, alternative-model states and elevations cannot reuse
      the wrong frame, model state or pitch,
    - a turret at displacement 0 reused across body facings, which can differ from a fresh one
      by the rounding above,
    - the side effects of fresh draws: the shades given to normal indices, which other models'
      out-of-table indices then show (claim 20), the switching of the depth-buffered mode
      (claim 32), and fresh-image initialisation (claim 34),
    - the depth look belongs to the type's own option, never to another draw,
    - under the cache-disable option, every eligible part appearance is fresh, so its
      out-of-table shades follow current fresh-draw inputs rather than an earlier image.
    - Backing: `measurable`. MS-17 covers the body facing, the ramp state, the unsettled tilt
      of the driving locomotor and the body facing in the turret key, at both zero and non-zero
      displacement. MS-2 covers
      the turret facing. MS-3b covers complete frame keys and cache disabling. MS-11
      covers separate state/frame components and selected turret index. MS-10 covers
      elevation, MS-6 recoil and MS-9 mode isolation. Corrected cache components are
      lead-confirmed departures tested against this spec rather than the original. Global image invalidation in the original is `unmeasured`: no scenario here forces
      such an event or separates it from a new reuse class. MS-3b's rendering fixture
      tests complete building turret frames without requiring an oracle counter above
      255. The naval locomotor case remains `unmeasured`: when a ship's body tilts is S-0106's (claims 6 and 7), but stock data may give a
      ship no tilt at all (S-0106, Open questions), and none of this spec's scenarios uses
      S-0106's naval test data (its scenario T6), so none of them makes it happen. A body
      drawn fresh beside a reused turret is covered by MS-17 (turret's body facing, run 2). The point at which the original empties its caches is not a claim of this spec (a
      departure, above), and whether a new game empties them is an open question.
    - Confidence: high (complete frames, separate alternative state, current elevation
      and body facing in the turret key), medium (ramp state and unsettled tilt, whose details
      are left to S-0106)

### Recoil

27. **Building turret recoil.** When a building with the recoil option fires, its turret and its
    barrel each run a recoil cycle (formula F7, state table S1): they slide back along the
    turret's forward axis by the imported travel distance (model units, i.e. pixels; a negative
    travel slides them forward, F7), hold, then return. The turret displacement is
    *−λ_turret* along its forward axis; the barrel follows that displacement in addition
    to its own, for total *−(λ_turret + λ_barrel)* along the same axis. Neither direction
    tilts with barrel elevation.
    - **Timing.** From rest, a cycle lasts *max(1, t_c) + max(0, t_h) + max(1, t_r)*
      ticks. Every positive hold duration holds for exactly that many ticks. With
      absent-key defaults (travel 2, compress 1, hold 1, recover 1), each cycle slides
      2, 2, 0 model units on successive ticks.
    - **A shot during a cycle.** Restart both cycles from their current slides, moving
      toward their configured travel over the new compress duration, without exceeding
      its magnitude. Zero travel stays at rest. F7 gives the signed clamp and rounding.
      Repeated shots cannot accumulate unbounded displacement.
    - **Barrel values across rules layers** (`quirks.barrel_recoil_layer_reset`). The barrel's travel, compress, hold and
      recover values each default to the turret's corresponding value, and this default is
      applied again in every rules layer that has a section for the type. After such a layer,
      each barrel value is the layer's own barrel key if it has one, and otherwise the turret's
      value as it stands after that layer (the layer's turret key, or the earlier value if the
      layer has none). A rules layer without a section for the type changes neither. Two
      consequences: a later rules layer that sets only the turret's travel makes the barrel's travel
      follow it, and a later rules layer that has a section for the type but no barrel key, even one
      that names no recoil key at all, replaces barrel values that an earlier rules layer set with the
      turret's values. The barrel values are thus *reset on override* (glossary), with the
      turret's current value as their reset value: an exception to S-0006's rule that an absent
      key keeps the earlier value (S-0006, claim 3), which S-0006 lists among its keys that reset
      to another key's value (S-0006, claim 5).
    - Backing: `measurable` (MS-6; barrel-following, positive holds and bounded refire
      are lead-confirmed departures tested against this spec rather than the original)
    - Confidence: high (cycle timing, step values, the refire rule and the barrel values across
      layers), medium (visible placement)

28. **Vehicles do not recoil.** A vehicle's turret and barrel are never drawn displaced by recoil,
    whatever its recoil settings (`quirks.vehicle_no_recoil`).
    - Backing: `measurable` (MS-7)
    - Confidence: high

### Shadows

29. **Shadow source.** A voxel vehicle's or aircraft's shadow is made from one section of its main
    body model, also while claim 12 draws something else in the body's place: the shadow section (imported index, default 0), always in HVA frame 0 regardless of the
    body's current frame (`quirks.shadow_frame_zero`). The turret and barrel cast no shadow (`quirks.no_turret_shadow`).
    - Backing: `measurable` (MS-8; MS-11 for the main body while another model is drawn in its
      place)
    - Confidence: high

30. **Shadow shape.** The shadow is the shadow section's footprint on its own bottom plane, drawn
    in the normal view: every column (fixed *i, j*) that the file lists as present marks two
    horizontally adjacent pixels (the second one pixel to the right, `quirks.shadow_double_width`), at the screen
    projection of
    that column's lower corner (*Bmin + (i, j) ⊙ s*) on the section's bottom face (*k = 0*
    plane). A column listed as present counts even if its runs hold no voxel; a column the file
    marks as empty marks nothing. Columns are positioned from the (min x, min y) corner, so the
    footprint is not mirrored. For vehicles, claim 39 excludes pixels covered by the object's
    current completed part image on every appearance. These positions have no 1/256-pixel carry error. Formula F10 gives the exact
    positions, the rounding and the copy to the screen. All marked pixels take one value,
    which the shadow treatment then darkens. The shadow transform is
    *V · S · H(0, shadow section)*, where *S* is the body's shadow transform: the ramp tilt of the
    cell under the unit (S-0106) and the body's 32-step rotation. Height and pitch of the
    body do not change the unmasked footprint's outline; they can change the own-image
    exclusions of claim 39.
    - Backing: `measurable` (MS-8) for ordinary footprint positions and present-column output.
      Unwrapped right-pixel adjacency is a lead-confirmed departure tested by MS-24's
      enlarged-footprint variant. Independence of the outline from body height
      and rocking is also `unmeasured`: MS-8 has no independently controlled comparison of
      either input.
    - Confidence: medium (high for the two pixels per column and for present columns without
      voxels)

31. **Shadow shift and scale.** The footprint is shifted 3 pixels to the right on screen (exactly
    *−6 × L_x*, stored truncated as 2.9999964, before the footprint's positions are truncated,
    F10). For types flagged as aircraft-like, the footprint is scaled by 1/2 about the shadow's
    draw point, and the 3-pixel shift is not halved (F10). The shadow is drawn with the ordinary shadow
    treatment (S-0102, S-0104). It is skipped whenever the object's cloak state is not
    *uncloaked* (that is, while cloaking, cloaked or uncloaking), and when the type has the
    no-shadow option. Shadow images are cached per type like part images (key: ramp, 32-step body
    facing), unless the type has the shadow-cache-disable option. They are also neither looked up
    nor stored, but drawn fresh, while a driving or naval unit's body is not settled or while a
    hovering unit rocks (S-0106, claim 12). These freshness rules do not override the cloak and
    no-shadow suppression above. For cache-eligible shadows, the key covers every input of the
    unmasked footprint (frame 0, the type's shadow section and scale are fixed per type).
    Cache only the unmasked footprint: current own-image exclusions are recomputed on
    every appearance (claim 39), and no shadow enables depth mode (claim 32). Freshness
    still matters where the original skips a shadow on a cache miss (open questions). That cache-miss skip does
    not apply to the unsettled shadows just described. Until those cache-miss cases are known,
    our engine makes the shadow on every cache miss.
    - Backing: `measurable` (MS-8) for the shift, scale, cloak and no-shadow conditions.
      MS-9 tests corrected mode isolation; MS-27 tests current-image exclusion with
      caching enabled and disabled, as lead-confirmed departures. Hovering unsettled
      shadow freshness is `unmeasured`: corrected exclusions agree for cached and fresh
      footprints, so the old rocking comparison no longer distinguishes freshness. The driving/naval unsettled case and exemption
      from cache-miss skip remain `unmeasured`: no scenario here isolates their freshness
      or the miss-skip event from changed keys and other fresh draws. Also `unmeasured` in
      two places. The look of a reused unmasked footprint: its key covers every input of that
      footprint, so fresh and reused unmasked footprints have the same pixels. Vehicle
      own-image exclusions also agree for fresh and reused appearances (claim 39). The ramp-key component is `measurable` through
      the changed footprint in
      S-0106 T3 and T6. The body-facing component is `unmeasured`: a new body facing
      also gives the body image a new key, so no scenario here separates their fresh-draw side
      effects. The skip on a cache miss is not a claim
      yet (open questions).
    - Confidence: medium (shift, scale, cache), high (cloak and no-shadow conditions)

### Depth-mode isolation

32. **Depth mode belongs to one fresh part draw (Q-0056).** Start with depth mode
    off. Enable it only during fresh depth-option part draws, and leave it off between
    draws, drawn frames and games. Shadows never enable it or ordinarily clear retained
    depths; claim 34's initial full clear remains. Non-depth parts, projectiles and
    debris inherit no depth mode from another draw. Reused images change no mode.
    - Backing: `measurable` (MS-9/MS-18; lead-confirmed departure, engine conformance)
    - Confidence: high

33. **Draw-order independence of the depth look.** Only a part's own type option
    selects its depth look, whether fresh or reused. Another fresh shadow, unsettled
    tilt or earlier draw cannot change that look. Scene compositing under S-0102 and
    claim 31's freshness rules are otherwise unchanged.
    - Backing: `measurable` (MS-9; lead-confirmed departure, engine conformance)
    - Confidence: high

### Buffer clearing and unusual model data

34. **Fresh-image initialisation.** Every fresh part, shadow, voxel projectile and
    voxel debris draw starts with completely transparent colour. Each fresh depth-mode
    draw also has empty comparison depth at every coordinate before any voxel contributes.
    Depth clearing to 0 means the empty sentinel of claim 24, below all numeric depths.
    Non-depth draws preserve retained depths except that the first fresh draw after
    program start fully clears both planes regardless of mode. Shadows never enable
    depth mode and ordinarily preserve depth. Newly added image coordinates are empty.
    No previous copy rectangle limits clearing; earlier colours never contaminate a
    fresh image and retained depths never suppress a fresh depth-option part.
    - Backing: `measurable` (MS-18/MS-24; lead-confirmed departures, engine conformance).
      Depth preservation between non-depth draws is `unmeasured`: the next depth draw
      clears it completely, so screenshots cannot distinguish preservation in that interval.
    - Confidence: high

35. **Normal mode 0.** A section whose normal mode is 0 has no normal table and is drawn unlit:
    each voxel's value goes into the voxel buffer as it is, without the VPL (a value of 0 is
    written as transparent and erases what an earlier voxel left at its pixel, as in claim 19). Its column data is read as if each voxel held one value, a colour
    index, and no normal index (`quirks.voxel_mode0_misread`). The importer warns about every
    section of normal mode 0 (Import mapping, *Importer warnings*). The output of one in-bound
    column is independent of the data or output of the preceding column.
    - **Covered output.** Standard column data can show normal indices as palette indices
      and can use colour or normal values as counts, producing a different picture from the
      normally lit column. This claim fixes the following discriminating outputs of MS-19,
      rather than prescribing a column-reading grammar. Grid-cell numbers are integers,
      counted upward from the section's bottom face.

      | Column supplied by the model | Pose | Output before screen projection |
      |---|---|---|
      | Two cells, both voxels with colour X and normal Y | bottom-up | cell 0 has palette index X, cell 1 has palette index Y |
      | e empty cells, then voxels (colour e, normal Z) and (colour X, normal Y), grid height e + 2 | bottom-up | cell e has palette index e, cell e + 1 has palette index Z |
      | The same e + 2-cell column | top-down | cell e has palette index X, cell e + 1 has palette index Y |

      MS-19 selects valid, visibly distinguishable indices and *e < 255 − Nz*. The values
      are unlit in both poses. The following colour-height relation specifies other in-bound
      outputs, rather than extrapolating from this table.
    - **Colour-height relation (Q-0211).** Treat the column's value sequence as a sequence of
      spans. A span has an empty count *e*, a colour count *n*, *n* palette indices
      *a₀ … aₙ₋₁*, and one ignored value *b*. Bottom-up, their sequence relation is
      *(e, n, a₀, …, aₙ₋₁, b)*. Top-down, the relation in forward file order is
      *(e, b, aₙ₋₁, …, a₀, n)*; spans are numbered from the column's top end in that pose.
      These are alternative interpretations of the supplied values, not the ordinary format's
      colour-and-normal voxel pairs. There is no normal associated with an output colour and
      no lighting lookup. An empty column contributes nothing. For a present column, spans
      are anchored to its own start or end, independently of every other column.

      Let *C₀ = 0*, *Cᵣ = Σₛ₌₁ʳ(eₛ + nₛ)*. Only spans through the first *r* for which
      *Cᵣ ≡ Nz (mod 256)* contribute outside depth-buffered mode. In depth-buffered mode
      the boundary condition is instead *Cᵣ ≡ Nz (mod 4294967296)*. Thus an ordinary small
      depth-buffered column ends only at *Cᵣ = Nz*, not *Nz + 256*. Within span *r*, colour *aₗ*, for
      *0 ≤ l < nᵣ*, is at traversal distance *d = Cᵣ₋₁ + eᵣ + l* cells from the far face
      bottom-up, and *d = Cᵣ₋₁ + l* top-down. All arithmetic here is exact integer
      arithmetic, with no rounding. Its grid cell is *d* bottom-up and *Nz − 1 − d*
      top-down. Thus bottom-up the span's empty count precedes its colours; top-down it
      follows them and affects only later spans. This applies to every span, including the
      topmost one, in both depth modes: there is no first-span exception (Q-0216).
      Projection uses F5's independent unwrapped components and the empty-count exception below.
      In the standard *e + 2*-cell column in the table, the top-down colours are *Y* at
      traversal distance 0 and *X* at distance 1, hence *Y* at cell *e + 1* and *X* at
      cell *e*. The remaining *e* empty cells contribute no pixel. In depth-buffered mode colours are still unlit,
      but the depth test, two-pixel output and absence of carry of claim 24 apply.
      A span with *n = 0* supplies no colour but its
      empty count still contributes to *C*. Reaching *Nz* inside a span does not cut it off.
      If no boundary is congruent to *Nz*, the bounded-data departure below applies.
      These spans are the mode-0 pairs for claim 36's visible-prefix rule. Top-down is
      anchored at the grid top, not at the height the supplied spans would have occupied:
      `quirks.voxel_topdown_column_origin` therefore applies in this mode too. With that
      switch off, positions are anchored to the decoded column's total height instead and
      only grid cells 0 … *Nz − 1* contribute. With `quirks.voxel_mode0_misread` off,
      the ordinary colour-and-normal pairs supply the colours, unlit.
    - **Reading past the column data.** If a column reaches the end of its own column data (or,
      top-down, its start) before it ends, the original reads on into the data that lies beyond,
      and what it draws from there is unspecified (as are the stray voxels of `quirks.voxel_topdown_column_origin`). Our engine, with the
      switch on, stops the column at the end of its own column data and draws nothing more for it.
      The importer warns about such a column.
    - **Positions.** Outside depth-buffered mode a voxel is placed as F5 gives, with an
      empty count *e* adding *e·Δ_z*, when *e < 255 − Nz*. After an empty count of
      *255 − Nz* or more the original's position is unspecified (it depends on earlier
      draws), and for *Nz = 255* its preparation fails in an unspecified way. In depth-buffered
      mode the bound is *e < Nz* for the x/y position, as for modes 1 to 4; the z component
      uses *e·Δ_z* for every empty count. Its preparation does not fail at *Nz = 255*.
      Our engine uses F5's *e·Δ_z* in every case, and
      draws a mode-0 section with *Nz = 255* like any other. The importer warns about a
      section with either case, since both come from the model file alone.
    - Backing: `measurable` (MS-19 for the table; MS-26 for multiple spans, independent
      columns, zero-value erasure, the prefix boundary, heights and both drawing directions).
      MS-19 (b) has an empty count of 40, below 255 − 42.
      Positions at or beyond the bound are `unmeasured`: there the original's position
      depends on what earlier draws left behind, which a running game does not let a test
      control, so there is no expected value to compare with. Reading beyond a column's own
      data is also `unmeasured`: the stray result is unspecified, so no safe expected image
      exists; our engine's bounded reading is a departure. The failed preparation at
      *Nz = 255* is also `unmeasured`: the result is unspecified and no safe predicted image
      exists. The safe behaviour of our engine is a departure, not an oracle measurement.
    - Confidence: high (unlit values, colour-height relation, prefix boundary and column
      independence), medium (the bound on empty counts)

36. **Malformed columns.** For columns whose runs do not add up to exactly *Nz* cells, the entry for
    `quirks.voxel_topdown_column_origin` gives what the original does: the position shift of a column drawn top-down, the stop at a
    pair boundary, and the stray voxels read from a neighbouring column's data.
    - **Importer warning.** The importer warns about every section with a column whose runs do not
      add up to exactly *Nz* cells, counting all its empty and voxel counts (Import mapping,
      *Importer warnings*), whatever the state of the switch. This is the condition of the
      heading. It covers the unspecified case below (a column that never reaches *Nz* at a pair
      boundary cannot add up to *Nz*) and also a column such as MS-20's, which stops cleanly at
      a pair boundary but has runs left over.
    - **Long malformed columns.** For the 260-cell column in a 4-cell grid of MS-24, bottom-up
      rendering includes voxels beyond the section grid, at the cell positions described by the
      column. Top-down rendering shifts that output 256 cells lower in grid coordinates, with
      no stray voxels in this case (`quirks.voxel_topdown_column_origin`). These voxels lie
      beyond a face of the section box; they cannot contaminate another fresh image (claim 34). This
      fixture does not establish the output of every possible long malformed column.
    - Backing: `measurable` (MS-20 for the four-cell cutoff; MS-24's upright/upside-down variants for the 260-cell
      column). MS-26 covers the additional mode-0 prefix and
      depth-mode distinction. The general cutoff and shift rule for other column data in
      the quirk entry remains `unmeasured`: these scenarios supply only the four- and 260-cell
      cases and do not discriminate other prefix totals or shifts. Also `unmeasured`
      for the stray voxels and for the shift of a column that does not end with its own runs. Such
      a column always goes on into a neighbouring column's data, which this spec leaves
      unspecified, so there is no expected value to compare with. What our engine draws instead (it stops at the
      end of the column's own runs) is given in the entry for `quirks.voxel_topdown_column_origin`.
    - Confidence: medium

### Buildings with a barrel model only

37. **A barrel without a turret.** A building that has a voxel barrel model but no voxel turret
    model draws the barrel alone. S-0007 claim 20 gives when that happens; stock defences whose only
    model is in the barrel role are such buildings. The barrel is drawn as follows.
    - **Pose.** Its draw transform is *V · Rz(θ) · Ry(φ)*, where *θ* is the building turret angle
      of claim 5 and *φ* is the elevation angle of claim 6. The turret displacement plays no part:
      there is no shift by *d_b* (claim 9), and the barrel pitches about its own model origin, as
      claim 10 specifies.
    - **Place.** The image is centred at the turret anchor (Terms).
    - **Frame.** It is drawn in HVA frame *u mod F_bar*, where *u* is the building's
      animation-frame counter (claim 16) and *F_bar* is the barrel HVA's frame count. It is not
      held at frame 0 as barrels drawn with a turret are.
    - **No recoil.** It is never drawn displaced by recoil, whatever the building's recoil option
      and values. A running recoil cycle does not stop its image being reused either
      (`quirks.barrel_only_no_recoil`).
    - **Image reuse.** It is kept with the type's barrel images. Its key is the 32-step facing of
      claim 5, its complete applicable frame and current 32-step elevation. The
      cache-disable option applies as in claim 26.
    - Backing: `measurable` (MS-21) for pose, placement, animation, lack of recoil and image
      reuse during recoil. Complete frames and current elevation are lead-confirmed
      departures tested by MS-21's engine-rendering fixtures, not the original. The exact modulo mapping
      from animation counter u to barrel-only frame is also `unmeasured`: MS-21 records the
      marker and attacks, not u, and therefore tests only the initial frame and attack-only changes.
    - Confidence: high (pose, frame, no recoil, key), medium (visible placement)

### The voxel lighting table file

38. **The lighting table file.** The voxel lighting table file is read once, at start-up (claim 19). By role, it holds a row count *R*; *R* rows, each holding one palette index for each voxel colour index; and three parts that have no effect (below): a remap range (a start and an end value), one more value (the *extra value*) and a palette. Where each part lies in the file, and in what form, is given by S-0013 (the byte layout of the voxel lighting table file). This claim fixes only what the content does. It relies on three facts of order alone, which are needed to say what a truncated file supplies: the rows come after all the other content, they are in shade order, and the entries of a row are in colour-index order.
    - **What counts.** Row *q*, numbered from 0 in file order, supplies the entries for shade
      *q*: the entry for shade *q* and colour *χ* is entry *χ* of row *q* (claim 19). Only the row
      count *R* and the rows have an effect.
    - **The row count decides.** The requested row content comes from the row count, never
      inferred from the file's length, and the file's length is not checked against it. The
      excessive-count exceptions below qualify the requested amount.
    - **What does nothing.** The remap range, the extra value and the palette have no effect: not
      on drawing, not on house colouring (S-0103 fixes the remap range at 16–31 itself) and not on
      anything else. S-0103 gives the palettes drawing does use.
    - **Rows in use.** Shade indices are 0–31 in practice (F4), so rows beyond the 32nd are not
      used.
    - **Missing and short data.** Rows the file does not supply read as all zero, which is the
      transparent index. A voxel drawn with such a row is still written (claim 19): it sets its
      pixel to transparent, so it leaves a hole where an earlier voxel would otherwise show. That
      covers *R* below 32, a missing file, and a file that ends before its last row. A file that
      ends before the first entry of row 0 supplies no row at all, whatever its row count says. When the file ends part-way through the rows, every
      whole row it holds is used, the entries it holds of a row it cuts off are used, and the rest
      reads as zero. The game goes on starting up as if the load had succeeded
      (`quirks.vpl_unloaded_rows_blank`). Anything in the file after row *R* − 1 is ignored.
    - **Departure: excessive row counts.** The row count *R* is a non-negative integer (S-0013). With a complete header and palette, let *B = (256 × R) mod 4294967296* bytes, exact integer arithmetic with a non-negative remainder and no rounding. Let *A* be the number of bytes available after that palette. When *B < 2147483648* bytes and *min(B, A) ≤ 32768* bytes, the original supplies exactly the first *min(B, A)* row bytes, unsupplied entries remain zero. In particular, *R = 16777216* rows requests zero bytes and supplies no entries even if rows follow. When *B ≥ 2147483648* bytes, or more than 32768 requested row bytes are supplied, the result is unspecified and can disturb unrelated game state. The first unsafe count is 8388608 rows, not the first count whose product wraps. Claim 6 of S-0013 owns these excessive-count exceptions to the ordinary missing and short-data rule above. Our engine never wraps the product: it uses the available prefix of the first *min(R, 128)* rows, preserves a partial row, fills unsupplied entries with zero and ignores the rest. A count above 128 rows warns (Import mapping, *Importer warnings*). This safe deterministic result is a departure, with no switch.
    - Backing: `measurable` (MS-22 for the use of the rows, the unused parts and data after the last row; MS-23 for the role of the row count, for short, missing and small tables, in its variant (g) for a safe row count above 128 in a file that holds at most 128 rows, and variant (i) for the file being read once, at start-up). Scenario L5 of S-0013 covers the safe wrapped zero-byte case. The two unsafe cases (a requested amount of at least 2147483648 bytes, or more than 32768 requested row bytes supplied) are `unmeasured`: their result is unspecified, and testing them could disturb unrelated game state.
    - Confidence: high (rows, row count, unused parts), medium (a file that ends part-way through
      a row, a file that ends before row 0, a missing file)

39. **A vehicle shadow excludes its current own image (Q-0213).** On every shadow
    appearance, fresh or reused, neither marked pixel of a footprint column remains
    where the vehicle's
    completed body, turret or barrel image has a non-transparent pixel at the same screen
    position. Each of the two shadow pixels is excluded independently. This depends on the
    part-image pixel, not on whether that pixel passes the scene's depth test. The shadow
    otherwise darkens qualifying scene pixels after the parts have contributed. Consequently,
    where both would pass, a freshly made vehicle shadow leaves its own visible part pixels
    undarkened. This applies on flat ground, on slopes and to hovering vehicles. Suppression
    and depth tests still apply (claim 31; S-0102 claim 18).
    - **Reuse.** Cache only the unmasked footprint. Exclude this object's current
      completed part image on every appearance, even when another object supplied the
      cached footprint. No old-pose cutouts persist.
    - Backing: `measurable` (MS-27; fresh exclusions are oracle-testable, corrected
      reuse is a lead-confirmed departure tested against this spec)
    - Confidence: high

## Formulas

**F0. Floating-point environment.** All floating-point work in this spec runs in the original
with 53-bit (double) intermediate precision and **round toward zero** for every operation and
every store. This holds for everything this spec computes, the HVA translation scaling (claim 14)
and every draw included; the light direction (claim 17) comes out the same under either rounding
(F2). Matrices, vectors and the shade sums are stored in single
precision between steps; each such store truncates toward zero. Float-to-integer conversions are
truncations toward zero. Confidence high for the number domain and rounding scope owned by
S-0205 claim 5; medium for the locations of the intermediate single-precision stores here.
Backing for the rounding scope is S-0205 N1 and N4; the rendering consequences are measurable
through this spec's golden-image scenarios. Exact store placement where it does not change a
captured pixel is unmeasured, because no scenario here distinguishes it. This is the
glossary's *round-toward-zero mode*. Its scope and its single switch `quirks.chop_rounding`
(default: the original behaviour) belong to S-0205 (claim 5); this spec defines no
switch of its own and only states what the mode changes here (Edge cases and quirks). With the
switch off, results differ from the original by at most one unit in the last place per stored
value. That matters only where a value lands exactly on a pixel or table boundary.

**F1. Table trigonometry.** For an angle *x* in radians, held at double precision:

> *I = trunc(x × 2607.594482421875)*, an integer in half table steps (truncation toward zero,
> the factor being 16384 / 2π half steps per radian, in single precision),
> *σ = min((trunc(I / 2) mod 8192) + (|I| mod 2), 8191)*, in table steps, where *trunc(I / 2)*
> is truncated toward zero and *mod 8192* is the non-negative remainder (0 … 8191),
> *sin x = τ(σ)*, *cos x = τ(σ + 2048)*,
>
> where *τ(n)*, for every *n* from 0 to 10239 (the range *σ + 2048* can reach), is the exact
> value of *sin(2πn / 8192)* truncated toward zero to single precision (the single-precision
> value nearest to it on the side of zero, or the value itself when it is exactly
> representable), with two exceptions:
> *τ(4096) = 1.2246063538223773 × 10⁻¹⁶* and *τ(8192) = −2.4492127076447545 × 10⁻¹⁶*, where the
> exact sine is 0. (Both are single-precision values; the decimals given identify them uniquely.)

No exact value *sin(2πn / 8192)* other than 0, 1 and −1 lies within a relative distance of
4 × 10⁻¹¹ of a single-precision value, so truncating a double-precision sine gives the same table.

Measure the angle in table steps as F1's product gives it: *ŝ = x × 2607.594482421875 / 2*, half
the product that *I* truncates. The factor is slightly below 16384 / 2π, so *ŝ* is slightly
smaller in magnitude than the exact angle, and the rules below hold for *ŝ*, not for the exact
angle. For non-negative *ŝ* this is "round to the nearest 1/8192 turn, halves up", except that an
angle that would round up to a whole number of turns gives 8191 steps (*σ* never steps on from
8191 to 0). For negative
*ŝ* it is not symmetric (`quirks.table_trig_negative_bias`). Write *ŝ* as −(m + f) table steps
(*m* whole, 0 ≤ *f* < 1):
- if *f* < 1/2, the result is −*m* steps (nearest rounding),
- if *f* ≥ 1/2 and *m* ≥ 2, the result is −(*m* − 1) steps, i.e. **two steps (0.088°) short**
  of the nearest value −(*m* + 1),
- if *f* ≥ 1/2 and *m* = 1, the result is −1 step; if *m* = 0, it is **+1** step.

Rotations by a multiple of π/16 come out slightly smaller in magnitude than the exact multiple
in the products used here. So every negative facing rotation in this spec lands exactly two steps short. For
example a body rotation for 32-step facings 9–31 is 0.088° less than the nominal angle, while
facings 0–8 are exact.

**Table square root.** Square roots used in normalisation are also tabulated. For a real input *x*:

> *√ₜ(x) = trunc₂₄(2^ζ × √m′)*, and *√ₜ(x) = 0* when *x̂* = 0,
>
> where *x̂* is *x* stored in single precision, truncated toward zero (F0); *|x̂| = m × 4^ζ*, with
> *ζ* an integer and 1 ≤ *m* < 4; *m′* is *m* truncated toward zero to 14 significant binary
> digits, that is to a multiple of 2⁻¹³ when *m* < 2 and of 2⁻¹² when *m* ≥ 2; *2^ζ × √m′* is
> exact; and *trunc₂₄* truncates toward zero to single precision (24 significant binary digits).

The result is never larger than the exact root of the magnitude, and its relative error
for nonzero inputs is below 2⁻¹³.

F1's visible rotation and shading consequences are `measurable` through MS-1, MS-4 and
MS-15, confidence high. The complete sine table, its two tiny nonzero exceptions and the
exact square-root approximation are `unmeasured`: the scenarios exercise only some inputs,
and integer pixels and shade indices cannot distinguish all last-place differences.
Confidence high for these mathematical definitions.

**F2. View transform.** *V = Rx(−60°) · Rz(−45°)* with table trigonometry (so the effective
angles are 59.985° and 44.912°, F1) and truncating single-precision stores (F0):

| | x | y | z |
|---|---|---|---|
| row x | 0.708190620 | 0.706021249 | 0 |
| row y | −0.353166908 | 0.354252070 | 0.865897536 |
| row z | 0.611342013 | −0.613220513 | 0.500221372 |

Truncation (F0) applies to every entry. With nearest rounding the row z, x entry would instead be 0.611342072; the others are the same either way.
The light vector of claim 17 is the same under both rounding modes.

Rotations use the right-handed convention: *Rz(θ)* maps +x toward +y for positive θ, and
*Ry(θ)* has rows *(cos θ, 0, sin θ)*, *(0, 1, 0)*, *(−sin θ, 0, cos θ)*. All composed transforms
multiply on the right (the new step acts in the current local frame).

**F3. Turret displacement unit.** *ρ = 60 / 362.038665771484375 = 0.16572815467691362* model units
per lepton, held at double precision. Here 60 is in model units (pixels), and 362.0386… is in
leptons and is the table square root of 2 × 256² (F1). Vehicle turret
shift *d = δ × ρ*, formed in double precision and then stored in single precision, truncated
toward zero (F0).

**F4. Shade index.** With *L′* and *E* as in claim 18, *Ĥ = (L′ + E) / |L′ + E|* (left
unnormalised if the length is 0), for each table normal *n*:

> *a = max(0, n · L′)*
> *η = n · Ĥ*, *g = max(0, η / (κ − η·κ + η))* with *κ = 3*
> *q = trunc(16 × (a + g))* (truncation toward zero).

The vector sum and the components of *Ĥ* are stored in single precision; *|L′ + E|* uses the
tabulated square root (F1). *a* stays at 53-bit precision; *g* is stored in single precision
before the clamp, and the sum and the product by 16 are formed at 53-bit precision (F0).
*q* is taken modulo 256. Because *a ≤ 1* and *g ≤ 1*, *q* is 0 … 31 in practice; 32 would
need *n*, *L′* and *Ĥ* to coincide exactly.

**F5. Pixel position of a voxel.** *y flip.* In F5 and F6 every view-space point is taken with
its *y* component negated, so that *y* grows down the screen: each transformed bounding-box corner
is stored at single precision (F0) and then has its *y* negated, which is exact. The corners
*P₀, P_x, P_y, P_z* below, the bounding boxes, *c* and the point *(x, y)* of F6 are all flipped
in this way, before any of the truncations below. Let *c* be the centre of the part's overall
view-space bounding box. For a section with view-space transform *Mᵢ*, let *P₀* be the farthest box corner
transformed and *P_x, P_y, P_z* the corners adjacent to it along the grid's x, y, z directions. In
units of 1/256 pixel, truncated toward zero:

> *o = trunc(256 × (P₀ + 128 − c))* (all three components)
> *Δ_ξ = trunc(256 × (P_ξ − P₀) / N_ξ)* for *ξ* in x, y, z (the z components of *o* and of
> the *Δ_ξ* matter only in the depth-buffered mode).

A voxel *n_x, n_y, n_z* steps from the farthest corner lands at
*p = o + n_x Δ_x + n_y Δ_y + n_z Δ_z*. Each component uses exact signed integer
arithmetic in units of 1/256 pixel, independently, without wrapping. The image pixel is
*(⌊px / 256⌋, ⌊py / 256⌋)* and the numeric depth is *⌊pz / 256⌋*, with floors toward
negative infinity, including negative coordinates. Image enlargement changes neither
these logical coordinates nor F6 placement. Horizontal arithmetic never changes *py*.
These are lead-confirmed coordinate, carry and depth departures; F0 preparation above
remains unchanged. MS-5/MS-12/MS-24 test them against this spec.

**F6. Screen placement.** A model point at view-space *(x, y)* (*y* flipped, F5) lands on the
screen at the draw point *+ trunc(c) + ⌊128 + (x, y) − c⌋ − 128*, i.e. within one pixel of
*⌊(x, y)⌋*. Floors apply to signed logical image coordinates. No point wraps at an
image edge (claim 22). The nominal area to copy before claim 22's limit,
the part's *copy rectangle*, is a *w × h* rectangle. With the overall bounding box
*[x_min, x_max] × [y_min, y_max]*, *w₀ = trunc(x_max − x_min)*, *h₀ = trunc(y_max − y_min)*,
*w = w₀ + 8* and *h = h₀ + 8*. The rectangle's corner is voxel-buffer pixel
*(128 − ⌊w₀/2⌋ − 4, 128 − ⌊h₀/2⌋ − 4)*, and it lands at the draw point plus
*(trunc(c_x) − ⌊w/2⌋, trunc(c_y) − ⌊h/2⌋)*.

**Retained window (claim 22, Q-0470).** Let the copy rectangle's logical corner be
*(a, b)* as above. Its retained width and height are *u = min(w, 4096)* and
*v = min(h, 4096)* pixels. The retained corner is
*(a + ⌊(w − u)/2⌋, b + ⌊(h − v)/2⌋)*. Keep exactly the integer pixel columns and
rows from that corner through *u − 1* and *v − 1* pixels beyond it, inclusively.
All window arithmetic is exact integer arithmetic in pixels; floors are toward
negative infinity. If centring leaves an odd pixel to discard, the right or bottom
side discards the extra pixel. The retained screen corner is the original copy
rectangle's screen corner plus the same two shifts. Thus clipping neither changes
*c* nor recentres or rescales the remaining picture. These rules also apply to
F10's shadow copy rectangle. Non-finite placement is suppressed under claim 22
rather than converted to an integer rectangle.

**F7. Recoil cycle.** Imported signed integer travel *ℓ* is in model units; signed
integer durations *t_c, t_h, t_r* are in ticks. Zero travel starts no cycle; negative
travel slides forward. Let *t_c′ = max(1, t_c)*, *t_h′ = max(0, t_h)* and
*t_r′ = max(1, t_r)*. At each shot, *λ_r* is the current slide, zero at rest.

> *s_c = (ℓ − λ_r) / t_c′*, hold step 0, *s_r = −ℓ / t_r′*, in model units per tick.

Divisions are floating point at 53-bit precision, stored in single precision truncated
toward zero (F0). The slide is single precision. Each compress tick adds *s_c*, stored
under F0 and clamped to the closed interval between 0 and *ℓ*. The last of *t_c′*
compress ticks reaches exactly *ℓ*, removing accumulated rounding error. It holds
there for exactly *t_h′* ticks. Each recover tick adds *s_r*, stored and clamped the
same way; the last of *t_r′* recover ticks sets exactly zero and ends the cycle.
A new shot restarts durations from the current slide, never adding another full travel.
Exact compress endpoint and signed clamp are part of the corrected presentation rule,
a lead-confirmed departure checked by MS-6 engine conformance. The first cycle tick
is the shot tick or the next (not fixed here).

Examples using invented values:
- travel 3, compress 2, hold 3, recover 4: 1.5, 3, 3, 3, 3, 2.25, 1.5, 0.75, 0.
- the same with hold 1: 1.5, 3, 3, 2.25, 1.5, 0.75, 0; hold 2:
  1.5, 3, 3, 3, 2.25, 1.5, 0.75, 0.
- absent-key defaults (travel 2, compress 1, hold 1, recover 1): 2, 2, 0.
- refire at travel 3: compress ticks stay at 3, then the new hold and recovery follow.
- travel 8, compress 8, hold 2, recover 2, refire at slide 3: new compress ticks
  3.625, 4.25, 4.875, 5.5, 6.125, 6.75, 7.375, 8, then 8, 8, 4, 0.

The turret shift is *−λ_turret* along its forward axis; its accompanying barrel shift
is *−(λ_turret + λ_barrel)*. Form the sum at 53-bit precision and store as F0 specifies.
No shift rounds to whole pixels before F5.

**F8. Section overlap priorities (claims 23, 24; Q-0210, Q-0215 questions 1–6).** Nearest depths are single-precision
view-space values under F0, in model units, compared exactly without tolerance or conversion
to integer comparison depths. Greater nearest depth means higher painter overlap priority.
The following relations fix both painter coverage and equal-comparison-depth coverage in
claim 24. Names stand for sections in the listed file order, not required storage positions.

| File order | Nearest-depth relation | Painter priority, lowest to highest | Depth-mode contributing order, first to last |
|---|---|---|---|
| A, B | A = B | A, B | B, A |
| A, B, C | A = B > C | C, B, A | A, B, C |
| A, B, C | A = B = C | A, B, C | C, B, A |
| A, B, C, D | A = B = C = D | A, B, C, D | D, C, B, A |
| A, B, C, D | A = B > C = D | C, D, A, B | B, A, D, C |
| A, B, C | A < B = C | A, B, C | C, B, A |
| A, B, C | A > B = C | B, C, A | A, C, B |
| A, B, C | A = C > B | B, A, C | C, A, B |
| A, B, C | A = C < B | A, C, B | B, C, A |
| A, B, C | A = B < C | A, B, C | C, B, A |
| A, B, C, D | A = B = C > D | D, B, C, A | A, C, B, D |
| A, B, C, D | A > B = C = D | B, C, D, A | A, D, C, B |
| A, B, C, D | A = C > B = D | B, D, C, A | A, C, D, B |
| A, B, C, D | A = D > B = C | B, C, A, D | D, A, C, B |

An entire part with any positive number of sections at one equal nearest depth has painter
priority in file order, and depth-mode contributing order in reverse file order. Thus the last
section in the file wins a common pixel, including an equal comparison depth in depth mode.
For every table row the last section in the painter-priority column similarly wins a pixel
shared by all sections at one comparison depth, provided that depth exceeds the initial depth
and no right-hand overwrite interferes (claim 24). These are coverage requirements; an
implementation need not sort or draw sections in these sequences if it produces the same
pixels. For finite nearest depths, the priority depends only on the full file-order section
sequence and its exact nearest-depth comparison relations, not on the numerical gaps between depths, colours,
occupied voxel positions, or earlier part draws. It is not a pair-local rule: A = B alone
gives A, B, but adding a lower-depth C after them gives C, B, A. Consequently sections
without pixels at a particular overlap can still change which tied section wins there.
The section count and the number of intervening sections are already part of that full
sequence, not extra inputs. Other mixed-depth configurations remain open; this table is
not a complete characterization for arbitrary section counts (Q-0215 question 7).
Backing: `measurable` (MS-25, MS-5b) for the table, all-equal case and input sufficiency.
Backing: `unmeasured` for an arbitrary-count mixed-tie relation, for the reason in claim 23.
Confidence: high for the stated results, low for extrapolation. MS-28 supplies proposed
outside-table observations, not a general default priority.

**F9. Turning at a turn rate (claim 6, Terms).** This is S-0206's turning rule (its claim 1 and F1), stated here for the barrel elevation. All quantities are integers. A facing turns from
a start facing *ψ₀* toward a target facing *ψ₁* at turn rate *ω* (1 … 127). Let *Δ* be *ψ₁ − ψ₀* taken as the signed
remainder in −32768 … 32767 (the shorter way round; a half-turn counts as −32768), and

> *Θ = ⌊|Δ| / (256 ω)⌋* ticks, *β = trunc(Δ / Θ)* facing units per tick (truncated toward zero).

With *ε* the number of whole ticks since the turn began, the facing is

> *(ψ₁ − β × (Θ − ε)) mod 65536* for 0 ≤ *ε* < *Θ*, and *ψ₁* from *ε = Θ* on.

If *Θ* = 0 the facing is *ψ₁* at once. Because *β × Θ* can fall short of *Δ*, the facing can jump by
the remainder *Δ − β × Θ* when the turn begins (at most *Θ − 1* facing units). Example: *ψ₀* = 16384,
*ψ₁* = 11264, *ω* = 3 gives *Θ* = 6, *β* = −853, and facings 16382, 15529, 14676, 13823, 12970,
12117, 11264 at *ε* = 0 … 6. A turn rate of 0 or less turns at once.

**F10. Shadow footprint (claims 30, 31).** Floating-point steps follow F0 (single-precision
stores truncated toward zero; sums and products at 53-bit precision). Let
*G = V · S · H(0, shadow section)*, with *S* the body's shadow transform (claim 30). For each of the four
corners *K* of the section's bottom face (*z = Bmin_z*): (min x, min y), (max x, min y),
(min x, max y) and (max x, max y):

> *Q = (μ · (G K)_x + 2.9999964, −μ · (G K)_y)*, where *μ* = 1/2 for an aircraft-like type and
> 1 otherwise, and with *G K* stored in single precision, truncated toward zero (F0), before it is
> halved and shifted (halving is exact in floating point).

So the half-size scale is about the view-space origin (the shadow's draw point), and the 3-pixel
shift is not halved. The z component of *G K* plays no part. The four points *Q* alone give the footprint's bounding
box and its centre *c*; the copy to the screen follows F6 with that box and centre, around the
shadow's draw point. Write *Q₀₀, Q₁₀, Q₀₁* for the corners (min x, min y), (max x, min y) and
(min x, max y). In units of 1/256 pixel, each result truncated toward zero:

> *o = trunc(256 × (Q₀₀ + 128 − c))*,
> *Δ_i = trunc(256 × (Q₁₀ − Q₀₀) / Nx)*, *Δ_j = trunc(256 × (Q₀₁ − Q₀₀) / Ny)*,

all per component. The column *(i, j)* lands at *p = o + i Δ_i + j Δ_j* = *(px, py)*, with each component
formed independently with exact signed integer arithmetic, no wrapping or carry.
For a present column, mark *(⌊px / 256⌋, ⌊py / 256⌋)* and
*(⌊px / 256⌋ + 1, ⌊py / 256⌋)*, floors toward negative infinity. Both belong to the
logical image, with no next-row wrap (lead-confirmed departure, MS-24 engine
conformance); claim 22 and F6 bound the retained shadow window. Every marked pixel takes one value, which the shadow treatment
darkens.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| View tilt | 60 | degrees | Rotation about view x; nominal, effectively 59.985° (F1, F2) |
| View turn | 45 | degrees | Rotation about view z; nominal, effectively 44.912° (F1, F2) |
| Facing step | π/16 | radians | 32 steps per turn |
| Unrotated body facing | 8 | 32-step facing | East |
| Light (view space) | (−0.49999943, −0.70710599, 0.49999943) | unit vector | Fixed for the game |
| Eye vector (model frame) | (0, 0, 1) | unit vector | `quirks.voxel_eye_in_model_frame` |
| Specular sharpness *κ* | 3 | dimensionless | F4 |
| Shade levels | 16 | shade indices per unit of *a + g* | F4 |
| Forced shade, normals 253–255 | 16 | shade index | Claim 20 |
| Pixels per lepton *ρ* | 0.16572815467691362 | model units / lepton | Vehicles only |
| Building turret shift divisor | 8 | leptons per model unit | Integer, truncated |
| voxel-image coordinate origin shift | 128, 128 | pixels | F5, F6 |
| Rows kept from a lighting table file | 128 | rows | Our engine's limit; a departure for files with more rows (claim 38) |
| Copy margin | 4 | pixels | Each side |
| Largest retained image side | 4096 | pixels | Part and shadow images; lead-confirmed engine rule, claim 22 |
| Sub-pixel precision | 1/256 | pixel | F5 |
| Depth-level spacing | 1 | model unit per level | Signed unwrapped depth; empty sentinel below every numeric depth (claim 24) |
| Shadow shift | +3 (2.9999964) | pixels, screen x | Claim 31 |
| Aircraft-like shadow scale | 1/2 | dimensionless (scale factor) | Claim 31 |
| Normal tables | 16 / 36 / 64 / 245 | entries | Modes 1 / 2 / 3 / 4; see below |
| Sine table | 8192 per turn | entries | F1 |
| Turret recoil defaults | travel 2, compress 1, hold 1, recover 1 | model units, ticks | Absent-key defaults; each absent barrel value defaults to the type's turret value, again in every rules layer that has the type's section (claim 27, `quirks.barrel_recoil_layer_reset`) |
| Shortest hold that holds | 1 | tick | Every positive imported hold is honoured (F7) |
| Barrel elevation turn rate | 3 | turn rate *ω* (Terms; nominally 256 facing units per tick per unit of *ω*) | Claim 6; a turn of *Δ* facing units takes ⌊\|Δ\| / 768⌋ ticks (F9) |
| Negative-angle table bias | 2 | table steps (1/8192 turn) | F1, `quirks.table_trig_negative_bias` |
| Level-body tolerance | 0.005 | radians | Claim 26: pitch or roll of this size or more counts as an unsettled tilt; compared in floating point |

### Normal direction tables

The tables below support claim 18. Their visible shade consequences are `measurable`
(MS-4 and MS-15), confidence high. Exact components of every table entry are `unmeasured`:
these scenarios do not isolate every normal, and different components can produce the same
integer shade and captured pixels. Confidence high for the listed constants.

Normal mode 0 has no table (`quirks.voxel_mode0_misread`). Modes 1–4 index the tables below by the voxel's normal
index. The values are the original's single-precision constants, in model-frame axes. Each entry is
a direction vector of about unit length; its components are dimensionless.

**Mode 1 (16 entries).**

| Normal | x | y | z |
|---|---|---|---|
| 0 | 0.549462974 | -0.000182999996 | -0.835518003 |
| 1 | 0.000144000005 | 0.549404025 | -0.835556984 |
| 2 | -0.549404025 | -6.80000012e-05 | -0.835556984 |
| 3 | 0.000106 | -0.549462974 | -0.835518003 |
| 4 | 0.949007988 | 0.000315999991 | -0.315250009 |
| 5 | -0.000186000005 | 0.948997021 | -0.315284014 |
| 6 | -0.948997021 | 0.000318000006 | -0.315284014 |
| 7 | -0.000446999999 | -0.949007988 | -0.315250009 |
| 8 | 0.95084399 | -0.000279 | 0.309671015 |
| 9 | 0.000201999996 | 0.950847983 | 0.309657007 |
| 10 | -0.950847983 | -7.00000019e-05 | 0.309657007 |
| 11 | 0.000146999999 | -0.95084399 | 0.309671015 |
| 12 | 0.552370012 | -1.10000001e-05 | 0.833598971 |
| 13 | 1.99999995e-05 | 0.552380025 | 0.833591998 |
| 14 | -0.552380025 | 5.70000011e-05 | 0.833593011 |
| 15 | -6.60000005e-05 | -0.552370012 | 0.833598971 |

**Mode 2 (36 entries).**

| Normal | x | y | z |
|---|---|---|---|
| 0 | 0.671213984 | 0.198492005 | -0.714194 |
| 1 | 0.269643009 | 0.584393978 | -0.765359998 |
| 2 | -0.040546 | 0.096988 | -0.994458973 |
| 3 | -0.572427988 | -0.0919139981 | -0.814786971 |
| 4 | -0.171400994 | -0.572709978 | -0.80163902 |
| 5 | 0.362556994 | -0.30299899 | -0.881331027 |
| 6 | 0.810347021 | -0.348971993 | -0.470697999 |
| 7 | 0.103961997 | 0.938672006 | -0.328767002 |
| 8 | -0.324046999 | 0.587669015 | -0.741375983 |
| 9 | -0.800864995 | 0.340460986 | -0.492646992 |
| 10 | -0.665498018 | -0.590147018 | -0.45698899 |
| 11 | 0.314767003 | -0.803002 | -0.506072998 |
| 12 | 0.972629011 | 0.151076004 | -0.176550001 |
| 13 | 0.680290997 | 0.68423599 | -0.262726992 |
| 14 | -0.520079017 | 0.827777028 | -0.210483 |
| 15 | -0.961643994 | -0.179001004 | -0.207846999 |
| 16 | -0.262713999 | -0.937451005 | -0.228401005 |
| 17 | 0.219706997 | -0.971301019 | 0.0911249965 |
| 18 | 0.923807979 | -0.229975 | 0.306086987 |
| 19 | -0.0824889988 | 0.970659971 | 0.225866005 |
| 20 | -0.591798007 | 0.69678998 | 0.405288994 |
| 21 | -0.925296009 | 0.36660099 | 0.0971110016 |
| 22 | -0.705051005 | -0.687775016 | 0.172828004 |
| 23 | 0.7324 | -0.680366993 | -0.0263049994 |
| 24 | 0.855162024 | 0.374581993 | 0.358310997 |
| 25 | 0.47300601 | 0.836480021 | 0.276704997 |
| 26 | -0.0976170003 | 0.654111981 | 0.750072002 |
| 27 | -0.904124022 | -0.153724998 | 0.398658007 |
| 28 | -0.211916 | -0.858089983 | 0.467732012 |
| 29 | 0.500226974 | -0.674408019 | 0.543090999 |
| 30 | 0.584538996 | -0.110248998 | 0.803840995 |
| 31 | 0.437373012 | 0.454643995 | 0.775888979 |
| 32 | -0.0424409993 | 0.0833180025 | 0.995618999 |
| 33 | -0.596251011 | 0.220131993 | 0.772028029 |
| 34 | -0.506455004 | -0.396977007 | 0.765448987 |
| 35 | 0.0705690011 | -0.478473991 | 0.875262022 |

**Mode 3 (64 entries).**

| Normal | x | y | z |
|---|---|---|---|
| 0 | 0.456510991 | -0.0739680007 | -0.886637986 |
| 1 | 0.507694006 | 0.385116994 | -0.770669997 |
| 2 | 0.0954319984 | 0.226664007 | -0.969286025 |
| 3 | -0.35876599 | 0.543187976 | -0.759100974 |
| 4 | -0.361276001 | 0.132994995 | -0.922926009 |
| 5 | -0.483117014 | -0.324066013 | -0.813374996 |
| 6 | -0.018073 | -0.197558999 | -0.980123997 |
| 7 | 0.321099997 | -0.501477003 | -0.803377986 |
| 8 | 0.799490988 | 0.0696159974 | -0.596629977 |
| 9 | 0.390971005 | 0.771305978 | -0.502224028 |
| 10 | 0.0807820037 | 0.614489973 | -0.784777999 |
| 11 | -0.732749999 | 0.411431015 | -0.542034984 |
| 12 | -0.73525399 | 0.00910199992 | -0.677730978 |
| 13 | -0.802493989 | -0.394908011 | -0.447270989 |
| 14 | -0.134130001 | -0.589155018 | -0.796809018 |
| 15 | 0.719552994 | -0.376226991 | -0.583693027 |
| 16 | 0.966875017 | 0.173593 | -0.187132001 |
| 17 | 0.760830998 | 0.519105971 | -0.38944301 |
| 18 | -0.114642002 | 0.875518978 | -0.469386011 |
| 19 | -0.532366991 | 0.768859029 | -0.354176998 |
| 20 | -0.962266982 | 0.0249770004 | -0.270958006 |
| 21 | -0.467386991 | -0.721985996 | -0.510182023 |
| 22 | 0.0584499985 | -0.85235399 | -0.519689023 |
| 23 | 0.498232991 | -0.743740022 | -0.445663005 |
| 24 | 0.939150989 | -0.270244986 | -0.212044001 |
| 25 | 0.583931983 | 0.809441984 | -0.061857 |
| 26 | 0.183797002 | 0.973227978 | -0.138007 |
| 27 | -0.884355009 | 0.452219009 | -0.115822002 |
| 28 | -0.943177998 | -0.332067013 | 0.0121379998 |
| 29 | -0.698440015 | -0.706566989 | -0.113771997 |
| 30 | -0.228411004 | -0.954706013 | -0.190694004 |
| 31 | 0.731563985 | -0.675861001 | -0.0895880014 |
| 32 | 0.969250977 | 0.0468039997 | 0.241582006 |
| 33 | 0.855647027 | 0.503476977 | 0.119915999 |
| 34 | -0.251152992 | 0.967947006 | -8.09999983e-05 |
| 35 | -0.647795022 | 0.756748974 | 0.0877119973 |
| 36 | -0.969164014 | 0.145193994 | 0.199100003 |
| 37 | -0.414793015 | -0.888966978 | 0.194125995 |
| 38 | 0.250775009 | -0.961178005 | -0.115108997 |
| 39 | 0.478628993 | -0.842593014 | 0.246883005 |
| 40 | 0.890043974 | -0.396142006 | 0.225594997 |
| 41 | 0.524051011 | 0.762359977 | 0.379707009 |
| 42 | 0.119620003 | 0.945482016 | 0.30291 |
| 43 | -0.760850012 | 0.490074992 | 0.425361991 |
| 44 | -0.869785011 | -0.202150002 | 0.450121999 |
| 45 | -0.709466994 | -0.602424026 | 0.36570701 |
| 46 | 0.0193089992 | -0.958871007 | 0.283185989 |
| 47 | 0.626112998 | -0.564677 | 0.537701011 |
| 48 | 0.769942999 | -0.126662999 | 0.625415027 |
| 49 | 0.764190972 | 0.350701988 | 0.541314006 |
| 50 | -0.00187799998 | 0.741366982 | 0.671097994 |
| 51 | -0.370880008 | 0.818368018 | 0.439000994 |
| 52 | -0.71390897 | 0.128652006 | 0.688318014 |
| 53 | -0.295165002 | -0.738663971 | 0.606014013 |
| 54 | 0.186195001 | -0.738368988 | 0.648184001 |
| 55 | 0.387522995 | -0.358783007 | 0.84917599 |
| 56 | 0.481022 | 0.124845996 | 0.86777401 |
| 57 | 0.391808003 | 0.545055985 | 0.741216004 |
| 58 | -0.00353599992 | 0.365597993 | 0.930765986 |
| 59 | -0.420498013 | 0.484961003 | 0.766807973 |
| 60 | -0.354903013 | 0.0194700006 | 0.934700012 |
| 61 | -0.547837019 | -0.359207988 | 0.755542994 |
| 62 | -0.106661998 | -0.445115 | 0.889097989 |
| 63 | 0.0867960006 | -0.0593070015 | 0.994458973 |

**Mode 4 (245 entries).**

| Normal | x | y | z |
|---|---|---|---|
| 0 | 0.526578009 | -0.359620988 | -0.770317018 |
| 1 | 0.150481999 | 0.435983986 | 0.887283981 |
| 2 | 0.414195001 | 0.738255024 | -0.532374024 |
| 3 | 0.0751520023 | 0.916248977 | -0.393498003 |
| 4 | -0.316148996 | 0.930736005 | -0.183792993 |
| 5 | -0.773819029 | 0.623333991 | -0.112510003 |
| 6 | -0.900842011 | 0.428537011 | -0.0695680007 |
| 7 | -0.998942018 | -0.0109710004 | 0.0446650013 |
| 8 | -0.979761004 | -0.157670006 | -0.123323999 |
| 9 | -0.911274016 | -0.362370998 | -0.19562 |
| 10 | -0.624068975 | -0.720941007 | -0.301301003 |
| 11 | -0.310173005 | -0.809345007 | -0.498751998 |
| 12 | 0.146613002 | -0.815819025 | -0.559414029 |
| 13 | -0.716516018 | -0.694356024 | -0.0668879971 |
| 14 | 0.503971994 | -0.114202 | -0.856136978 |
| 15 | 0.455491006 | 0.87262702 | -0.176210999 |
| 16 | -0.00500999996 | -0.114372998 | -0.993425012 |
| 17 | -0.104675002 | -0.327701002 | -0.938965023 |
| 18 | 0.56041199 | 0.752588987 | -0.345755994 |
| 19 | -0.0605759993 | 0.821627975 | -0.566796005 |
| 20 | -0.302341014 | 0.797007024 | -0.522846997 |
| 21 | -0.671543002 | 0.670740008 | -0.314862996 |
| 22 | -0.778401017 | -0.128356993 | 0.614504993 |
| 23 | -0.924049973 | 0.278382003 | -0.261985004 |
| 24 | -0.699773014 | -0.550490975 | -0.455278009 |
| 25 | -0.568247974 | -0.517189026 | -0.640007973 |
| 26 | 0.0540979989 | -0.93286401 | -0.356142998 |
| 27 | 0.758382022 | 0.572893023 | -0.310887992 |
| 28 | 0.00362000009 | 0.305025995 | -0.952337027 |
| 29 | -0.0608499981 | -0.986886024 | -0.149510995 |
| 30 | 0.635230005 | 0.0454780012 | -0.770982981 |
| 31 | 0.521704972 | 0.241309002 | -0.818287015 |
| 32 | 0.269403994 | 0.635424972 | -0.723640978 |
| 33 | 0.0456760004 | 0.67275399 | -0.738454998 |
| 34 | -0.180510998 | 0.674656987 | -0.715718985 |
| 35 | -0.397130996 | 0.636640012 | -0.661041975 |
| 36 | -0.55200398 | 0.472514987 | -0.687038004 |
| 37 | -0.772170007 | 0.0830899999 | -0.629960001 |
| 38 | -0.669818997 | -0.119533002 | -0.732840002 |
| 39 | -0.540454984 | -0.318444014 | -0.77878201 |
| 40 | -0.386135012 | -0.522789001 | -0.75999397 |
| 41 | -0.261465997 | -0.688566983 | -0.676394999 |
| 42 | -0.0194119997 | -0.696102977 | -0.717679977 |
| 43 | 0.303568989 | -0.481844008 | -0.821992993 |
| 44 | 0.681939006 | -0.195129007 | -0.704900026 |
| 45 | -0.244889006 | -0.116562001 | -0.96251899 |
| 46 | 0.800759017 | -0.0229790006 | -0.598546028 |
| 47 | -0.370274991 | 0.0955839977 | -0.923991024 |
| 48 | -0.330671012 | -0.326577991 | -0.885439992 |
| 49 | -0.163220003 | -0.52757901 | -0.83367902 |
| 50 | 0.126389995 | -0.313145995 | -0.941257 |
| 51 | 0.349548012 | -0.272226006 | -0.896498024 |
| 52 | 0.239917994 | -0.0858250037 | -0.966992021 |
| 53 | 0.390845001 | 0.0815370008 | -0.91683799 |
| 54 | 0.255266994 | 0.268696994 | -0.928785026 |
| 55 | 0.146245003 | 0.480437994 | -0.864749014 |
| 56 | -0.326016009 | 0.478455991 | -0.815348983 |
| 57 | -0.469682008 | -0.112519003 | -0.875635982 |
| 58 | 0.81844002 | -0.258520007 | -0.51315099 |
| 59 | -0.474317998 | 0.292237997 | -0.830433011 |
| 60 | 0.778943002 | 0.395841986 | -0.486371011 |
| 61 | 0.624094009 | 0.39377299 | -0.674870014 |
| 62 | 0.740885973 | 0.203833997 | -0.639953017 |
| 63 | 0.48021701 | 0.565768003 | -0.670297027 |
| 64 | 0.380930007 | 0.424535006 | -0.821377993 |
| 65 | -0.0934220031 | 0.501124024 | -0.860318005 |
| 66 | -0.236485004 | 0.29619801 | -0.925387025 |
| 67 | -0.131531 | 0.0939590037 | -0.98684901 |
| 68 | -0.823562026 | 0.295776993 | -0.484005988 |
| 69 | 0.611065984 | -0.624303997 | -0.486663997 |
| 70 | 0.0694959983 | -0.520330012 | -0.851132989 |
| 71 | 0.226521999 | -0.664879024 | -0.711775005 |
| 72 | 0.471307993 | -0.568903983 | -0.67395699 |
| 73 | 0.388424993 | -0.742623985 | -0.545560002 |
| 74 | 0.783675015 | -0.480729014 | -0.393384993 |
| 75 | 0.962393999 | 0.135675997 | -0.235349 |
| 76 | 0.876607001 | 0.172033995 | -0.449405998 |
| 77 | 0.63340503 | 0.589793026 | -0.500940979 |
| 78 | 0.182275996 | 0.800657988 | -0.570720971 |
| 79 | 0.177002996 | 0.76413399 | 0.620297015 |
| 80 | -0.544016004 | 0.675514996 | -0.497720987 |
| 81 | -0.67929697 | 0.286466986 | -0.675642014 |
| 82 | -0.59039098 | 0.0913690031 | -0.801928997 |
| 83 | -0.824360013 | -0.133123994 | -0.550189018 |
| 84 | -0.715794027 | -0.334542006 | -0.612960994 |
| 85 | 0.174285993 | -0.892484009 | 0.416049004 |
| 86 | -0.0825280026 | -0.837122977 | -0.540753007 |
| 87 | 0.283331007 | -0.880873978 | -0.379189014 |
| 88 | 0.675134003 | -0.42662701 | -0.601817012 |
| 89 | 0.843720019 | -0.512335002 | -0.160155997 |
| 90 | 0.977303982 | -0.098555997 | -0.187519997 |
| 91 | 0.846294999 | 0.522671998 | -0.102946997 |
| 92 | 0.677141011 | 0.72132498 | -0.145501003 |
| 93 | 0.320964992 | 0.870891988 | -0.372193992 |
| 94 | -0.178977996 | 0.911532998 | -0.370236009 |
| 95 | -0.447169006 | 0.826700985 | -0.341473997 |
| 96 | -0.703203022 | 0.496327996 | -0.509081006 |
| 97 | -0.977181017 | 0.0635629967 | -0.202674001 |
| 98 | -0.878170013 | -0.412937999 | 0.241455004 |
| 99 | -0.835830986 | -0.358550012 | -0.415728003 |
| 100 | -0.499173999 | -0.693432987 | -0.519591987 |
| 101 | -0.188788995 | -0.923753023 | -0.333225012 |
| 102 | 0.192254007 | -0.969361007 | -0.152896002 |
| 103 | 0.515940011 | -0.783906996 | -0.345391989 |
| 104 | 0.905924976 | -0.300951988 | -0.297870994 |
| 105 | 0.991111994 | -0.127746001 | 0.0371069983 |
| 106 | 0.995135009 | 0.0984240025 | -0.00438300008 |
| 107 | 0.760123014 | 0.64627701 | 0.0673670024 |
| 108 | 0.205220997 | 0.959580004 | -0.192590997 |
| 109 | -0.042750001 | 0.97951299 | -0.196790993 |
| 110 | -0.438017011 | 0.898926973 | 0.0084920004 |
| 111 | -0.821994007 | 0.480785012 | -0.305238992 |
| 112 | -0.899917006 | 0.0817100033 | -0.428337008 |
| 113 | -0.92661202 | -0.144618005 | -0.347095996 |
| 114 | -0.793659985 | -0.557792008 | -0.242838994 |
| 115 | -0.431349993 | -0.847778976 | -0.308557987 |
| 116 | -0.00549199991 | -0.964999974 | 0.262192994 |
| 117 | 0.58790499 | -0.804026008 | -0.088940002 |
| 118 | 0.699492991 | -0.667685986 | -0.254765004 |
| 119 | 0.889303029 | 0.359795004 | -0.282290995 |
| 120 | 0.780972004 | 0.197036996 | 0.59267199 |
| 121 | 0.520120978 | 0.506695986 | 0.687556982 |
| 122 | 0.403894991 | 0.693961024 | 0.596059978 |
| 123 | -0.154982999 | 0.899236023 | 0.409090012 |
| 124 | -0.657338023 | 0.537168026 | 0.528542995 |
| 125 | -0.746195018 | 0.334091008 | 0.575827003 |
| 126 | -0.624952018 | -0.0491439998 | 0.779115021 |
| 127 | 0.318141013 | -0.254714996 | 0.913185 |
| 128 | -0.555896997 | 0.405294001 | 0.725751996 |
| 129 | -0.794434011 | 0.0994059965 | 0.599160016 |
| 130 | -0.640361011 | -0.689463019 | 0.338494986 |
| 131 | -0.126712993 | -0.734094977 | 0.66711998 |
| 132 | 0.105457 | -0.780816972 | 0.615795016 |
| 133 | 0.407992989 | -0.480915993 | 0.776054978 |
| 134 | 0.695136011 | -0.545120001 | 0.468647003 |
| 135 | 0.973191023 | -0.00648899982 | 0.229908004 |
| 136 | 0.94689399 | 0.317508996 | -0.050799001 |
| 137 | 0.563583016 | 0.825612009 | 0.027183 |
| 138 | 0.325773001 | 0.945423007 | 0.00694900006 |
| 139 | -0.171820998 | 0.985096991 | -0.00781499967 |
| 140 | -0.670440972 | 0.739938974 | 0.0547689982 |
| 141 | -0.822981 | 0.554961979 | 0.121321999 |
| 142 | -0.96619302 | 0.117857002 | 0.229306996 |
| 143 | -0.953769028 | -0.29470399 | 0.0589450002 |
| 144 | -0.864386976 | -0.502727985 | -0.0100149997 |
| 145 | -0.530609012 | -0.842006028 | -0.0973659977 |
| 146 | -0.162617996 | -0.98407501 | 0.0717720017 |
| 147 | 0.0814469978 | -0.996011019 | 0.0364390016 |
| 148 | 0.745984018 | -0.665962994 | 0.000761999981 |
| 149 | 0.942057014 | -0.329268992 | -0.0641060024 |
| 150 | 0.939701974 | -0.281089991 | 0.194802999 |
| 151 | 0.771214008 | 0.550670028 | 0.319362998 |
| 152 | 0.641348004 | 0.730690002 | 0.234020993 |
| 153 | 0.080682002 | 0.996690989 | 0.00987899955 |
| 154 | -0.0467250012 | 0.976643026 | 0.209725007 |
| 155 | -0.531076014 | 0.821000993 | 0.209562004 |
| 156 | -0.695815027 | 0.655990005 | 0.29243499 |
| 157 | -0.976122022 | 0.216709003 | -0.0149130002 |
| 158 | -0.961660981 | -0.144128993 | 0.233313993 |
| 159 | -0.772083998 | -0.613646984 | 0.165298998 |
| 160 | -0.449600011 | -0.836059988 | 0.314426005 |
| 161 | -0.392699987 | -0.914615989 | 0.0962470025 |
| 162 | 0.390588999 | -0.919470012 | 0.0448900014 |
| 163 | 0.582529008 | -0.799197972 | 0.148127005 |
| 164 | 0.866430998 | -0.489811987 | 0.096864 |
| 165 | 0.904586971 | 0.111497998 | 0.411449999 |
| 166 | 0.953536987 | 0.232329994 | 0.191806003 |
| 167 | 0.497310996 | 0.770802975 | 0.398176998 |
| 168 | 0.194066003 | 0.956319988 | 0.218611002 |
| 169 | 0.422876 | 0.882275999 | 0.206797004 |
| 170 | -0.373796999 | 0.849565983 | 0.372173995 |
| 171 | -0.534497023 | 0.714022994 | 0.452199996 |
| 172 | -0.881826997 | 0.237159997 | 0.407597989 |
| 173 | -0.904947996 | -0.0140690003 | 0.425289005 |
| 174 | -0.751827002 | -0.512817025 | 0.414458007 |
| 175 | -0.501015007 | -0.697916985 | 0.511758029 |
| 176 | -0.235190004 | -0.92592299 | 0.295554996 |
| 177 | 0.228983 | -0.953939974 | 0.193819001 |
| 178 | 0.734025002 | -0.634898007 | 0.241062 |
| 179 | 0.913752973 | -0.0632530004 | -0.401315987 |
| 180 | 0.905735016 | -0.161486998 | 0.391874999 |
| 181 | 0.858929992 | 0.342445999 | 0.380748987 |
| 182 | 0.624486029 | 0.607581019 | 0.490776986 |
| 183 | 0.289263994 | 0.857478976 | 0.425507993 |
| 184 | 0.0699680001 | 0.902168989 | 0.425671011 |
| 185 | -0.28617999 | 0.940699995 | 0.182164997 |
| 186 | -0.574012995 | 0.805118978 | -0.149308994 |
| 187 | 0.111258 | 0.099717997 | -0.988776028 |
| 188 | -0.30539301 | -0.944227993 | -0.123159997 |
| 189 | -0.60116601 | -0.789575994 | 0.123163 |
| 190 | -0.290645003 | -0.812139988 | 0.50591898 |
| 191 | -0.0649200007 | -0.877162993 | 0.475784987 |
| 192 | 0.408300996 | -0.862215996 | 0.299789011 |
| 193 | 0.566097021 | -0.72556603 | 0.391263992 |
| 194 | 0.839363992 | -0.427386999 | 0.335869014 |
| 195 | 0.818899989 | -0.0413050018 | 0.572448015 |
| 196 | 0.719784021 | 0.414997011 | 0.556496978 |
| 197 | 0.881744027 | 0.450269997 | 0.140659004 |
| 198 | 0.401823014 | -0.898220003 | -0.178151995 |
| 199 | -0.0540199988 | 0.791343987 | 0.60898 |
| 200 | -0.293774009 | 0.763993979 | 0.574464977 |
| 201 | -0.450798005 | 0.610346973 | 0.651350975 |
| 202 | -0.638221025 | 0.186693996 | 0.746873021 |
| 203 | -0.872870028 | -0.257126987 | 0.414707989 |
| 204 | -0.587257028 | -0.521709979 | 0.618827999 |
| 205 | -0.353657991 | -0.641973972 | 0.680290997 |
| 206 | 0.0416489989 | -0.611272991 | 0.790323019 |
| 207 | 0.348342001 | -0.779182971 | 0.521086991 |
| 208 | 0.499166995 | -0.622440994 | 0.602825999 |
| 209 | 0.790018976 | -0.303831011 | 0.532500029 |
| 210 | 0.660117984 | 0.0607330017 | 0.74870199 |
| 211 | 0.604920983 | 0.294160992 | 0.739960015 |
| 212 | 0.385697007 | 0.379346013 | 0.841032028 |
| 213 | 0.239693001 | 0.207875997 | 0.948332012 |
| 214 | 0.012623 | 0.258531988 | 0.965919971 |
| 215 | -0.100556999 | 0.457147002 | 0.883687973 |
| 216 | 0.0469669998 | 0.628588021 | 0.776319027 |
| 217 | -0.430391014 | -0.445405006 | 0.785097003 |
| 218 | -0.434291005 | -0.196227998 | 0.879139006 |
| 219 | -0.256637007 | -0.336867005 | 0.905902028 |
| 220 | -0.131372005 | -0.158910006 | 0.978514016 |
| 221 | 0.102379002 | -0.208766997 | 0.972591996 |
| 222 | 0.195686996 | -0.450129002 | 0.87125802 |
| 223 | 0.627318978 | -0.423148006 | 0.653770983 |
| 224 | 0.687439024 | -0.171582997 | 0.70568198 |
| 225 | 0.275920004 | -0.0212549996 | 0.960946023 |
| 226 | 0.459367007 | 0.157465994 | 0.874177992 |
| 227 | 0.285394996 | 0.583184004 | 0.760555983 |
| 228 | -0.812174022 | 0.460303009 | 0.358460993 |
| 229 | -0.189068004 | 0.641223013 | 0.743698001 |
| 230 | -0.338874996 | 0.476480007 | 0.811251998 |
| 231 | -0.920993984 | 0.347185999 | 0.176726997 |
| 232 | 0.0406389982 | 0.0244650003 | 0.998874009 |
| 233 | -0.739131987 | -0.35374701 | 0.573189974 |
| 234 | -0.603511989 | -0.286615014 | 0.74405998 |
| 235 | -0.188676 | -0.547059 | 0.815554023 |
| 236 | -0.0260450002 | -0.397819996 | 0.917093992 |
| 237 | 0.26789701 | -0.649040997 | 0.71202302 |
| 238 | 0.518245995 | -0.284891009 | 0.806385994 |
| 239 | 0.493450999 | -0.0665329993 | 0.867224991 |
| 240 | -0.328188002 | 0.140250996 | 0.934143007 |
| 241 | -0.328188002 | 0.140250996 | 0.934143007 |
| 242 | -0.328188002 | 0.140250996 | 0.934143007 |
| 243 | -0.328188002 | 0.140250996 | 0.934143007 |
| 244 | -0.328188002 | 0.140250996 | 0.934143007 |

## State and ordering

**S1. Recoil state (buildings only).** Each row's result applies from the tick named; *λ* as in F7.

| State | Event | Result |
|---|---|---|
| idle (*λ* = 0) | the building fires and its travel ℓ is not 0 | compressing; the first compress tick is the shot tick or the next one (F7) |
| compressing | each of its *t_c′* ticks | λ moves toward ℓ with F7's bounded step; last tick reaches ℓ |
| compressing, *t_h* ≥ 1 | its last tick ends | holding, from the next tick |
| compressing, *t_h* ≤ 0 | its last tick ends | recovering, from the next tick, with no hold |
| holding | each of its *t_h* ticks | λ unchanged |
| holding | its last tick ends | recovering, from the next tick |
| recovering | each of its *t_r′* ticks but the last | λ grows by the recover step *s_r*, which has the opposite sign to ℓ (F7) |
| recovering | its last tick | λ becomes exactly 0; idle |
| compressing, holding or recovering | building fires again | compressing from current λ toward ℓ over *t_c′* ticks, bounded by travel; hold and recovery restart afterward |

Turret and barrel each have their own cycle with their own imported values; both start on the
same shot.

**S2. Depth mode (claims 24, 32, 34).** Starts off; scoped to one fresh part draw.

| State | Event | Result |
|---|---|---|
| off | fresh depth-option part starts | on for that draw, empty comparison depth everywhere |
| on | that part finishes | off |
| off | shadow, non-depth part, projectile, debris or reused image | remains off |
| off | new drawn frame or game | remains off |

All fresh draws clear colour completely. Non-depth draws preserve depth except for
claim 34's initial full clear. These are lead-confirmed departures (MS-9/MS-18).

**Ordering consequences.**
- Within one part outside depth-buffered mode: claim 23's section overlap priority and
  claim 21's voxel priority decide which pixels show. A nearer voxel of a lower-priority
  section can be covered by a farther voxel of a higher-priority section, because there is
  no depth test between sections (`quirks.voxel_no_intersection_depth`). Claim 24 owns the
  depth-buffered exception.
- Between parts of one vehicle: body, then turret and barrel in claim 11 order, so the turret and
  barrel cover the body where they overlap it. Parts never depth-test against each other.
- The shadow belongs to the object's own drawing, in its own render layer and list position,
  not to a separate ground-layer pass (S-0102, claim 18). A vehicle's freshly made shadow
  excludes its current completed part image on every appearance, including reuse,
  so its qualifying own-body overlap remains undarkened (claim 39).

## Edge cases and quirks

Every switch defaults to the original (YR) behaviour described; "Fixed" gives the alternative.

- **`quirks.vpl_unloaded_rows_blank`** (default: on). Rows that the lighting table file does not
  supply (claim 38) read as zero, so their voxels draw transparent pixels (claim 19), and a missing or truncated file does not stop
  the game. Fixed: the importer rejects such a file with an error. (A row count above 128 is not part of
  this switch: it is a departure, claim 38.)

- **`quirks.voxel_eye_in_model_frame`** (default: on). The specular term uses the eye direction (0, 0, 1)
  in the model's own frame instead of the real viewing direction brought into the model frame.
  Highlights therefore follow the model's up axis and do not move consistently as the model
  turns. Fixed: use *Rᵀ (0, 0, 1)* of the view.
- **`quirks.voxel_light_first_section_mode`** (default: on). One shade table per part, using the first
  section's normal mode, even if other sections declare a different mode. Fixed: per-section
  tables.
- **`quirks.voxel_light_ignores_hva`** (default: on). The light is brought into the part frame without the
  section's HVA rotation, so a section rotated by its HVA keeps the shading of its unrotated
  pose. Fixed: include *H(f, i)* per section.
- **`quirks.hva_scale_first_section`** (default: on). All HVA translations are scaled by the first section's
  scale factor. Fixed: each section's own factor.
- **`quirks.voxel_stale_normal_shades`** (default: on). Normal indices beyond the table (below 253) reuse a
  stale shade. Fixed: treat as shade 16.
- **`quirks.voxel_single_pixel`** (default: on). One pixel per voxel, so magnified sections show holes.
  Fixed: fill the voxel's projected box.
- **`quirks.barrel_elevation_quantised`** (default: on). The barrel's elevation shows only in 32-step
  (11.25°) increments, and the fire angle is taken modulo 256. Fixed: continuous pitch.
- **`quirks.building_turret_shift_eighths`** (default: on). Building turrets convert the turret displacement
  with "÷ 8, truncated" instead of the vehicle factor *ρ* (≈ ÷ 6.03), and move after rotating.
  Fixed: *d_b = δ × ρ*, rounded to single precision as for vehicles (F3). The order is kept: the
  turret still moves along its own forward axis, because a building has no separate body facing
  to move along. No switch is proposed for a vehicle-style order.
- **`quirks.vehicle_no_recoil`** (default: on). Vehicle recoil settings are read but have no visual effect.
  Fixed: apply F7 to vehicle turret and barrel.
- **`quirks.shadow_frame_zero`** (default: on). Shadows always use HVA frame 0. Fixed: current body frame.
- **`quirks.no_turret_shadow`** (default: on). No shadow from turret or barrel; the per-unit turret-shadow
  setting is read and ignored. Fixed: honour it.
- **`quirks.table_trig_negative_bias`** (default: on). Negative angles whose fractional table step, measured as F1's product gives it
  (*ŝ*, F1), is at least one half land two 1/8192-turn steps short of nearest rounding (F1). All negative facing
  rotations here are such angles: a body whose 32-step facing is 16 (south) rotates by −89.912°
  instead of −90°, and one at 12 (south-east) by −44.912° instead of −45°.
  A turret rotated clockwise relative to its body is likewise 0.088° short, and so is a raised
  barrel. Fixed: exact rounding.
- **`quirks.voxel_mode0_misread`** (default: on). Claim 35. A section with normal mode 0 is drawn unlit,
  and its column data is read as one value per voxel. A standard file, where every voxel holds a
  colour index and a normal index, is then misread: normal indices show as colours, and after the
  first run of the reading direction later counts are taken from colour and normal indices, so the
  section can show malformed output. With the switch on (default), claim 35's general
  colour-height relation is binding. Where the original would read past the column data (claim
  35, unspecified), our engine stops that column at the end of its own column data, and the
  importer warns (Import mapping). Fixed: read
  each voxel's colour index and normal index, and draw the colour unlit.
- **`quirks.voxel_no_intersection_depth`** (default: on). No depth test between sections unless the
  per-type depth option is set; interpenetrating sections can show the farther one in front.
  Fixed: depth-test between sections.
- **`quirks.voxel_depth_double_width`** (default: on). With the depth option every voxel is two pixels wide.
  Fixed: one pixel per voxel, as without the option.
- **`quirks.barrel_frame_zero`** (default: on). Barrels drawn together with a turret always use HVA
  frame 0 (claim 16; a building barrel without a turret is not affected, claim 37). Fixed: use the
  turret's frame.
- **`quirks.voxel_topdown_column_origin`** (default: on). Claim 36. Malformed columns, whose runs do not add
  up to exactly *Nz* cells (the condition of the importer's warning, claim 36). Three things
  happen in the original:
  - *Shift.* When the section is drawn top-down (claim 21), each column is positioned from the
    top of the grid. A column whose runs cover *Nz − γ* cells is drawn *γ* cells too high (for a
    negative *γ*, a column whose runs are too long, *|γ|* cells too low). Drawn bottom-up, the
    same column is placed correctly.
  - *Visible prefix.* Let C₀ = 0 and Cᵣ be the total cells described by the first r complete
    empty-run/voxel-run pairs in the column. Only the prefix ending at the smallest r with
    Cᵣ congruent to Nz modulo 256 contributes voxels outside depth-buffered mode; with
    depth-buffered mode on, the modulus is 4294967296 instead. Later runs contribute none. These are
    exact integer counts with a non-negative remainder, no rounding. This defines which
    supplied voxels can show, not an order of parser work. MS-20 tests the prefix ending at
    C₁ = Nz, and MS-24 the prefix ending at C₂ = Nz + 256. Other totals are unmeasured (claim 36).
  - *Stray voxels.* A column that never satisfies the visible-prefix boundary condition
    above within its own runs does not end with those runs. Passing *Nz* inside a run
    does not itself end the column; a later pair boundary can still satisfy the condition.
    If none does, the original continues with
    data that belongs to the neighbouring column (the next column when drawn bottom-up,
    the previous one when drawn top-down) as further runs, and draws whatever voxels they
    describe, at positions and in colours that depend on unrelated file contents. That part is
    unspecified. So is the position of a voxel that follows an empty run of *Nz* or more cells
    (in a section of normal mode 1 to 4; for normal mode 0, claim 35 gives the bound).

  With the switch on (default), our engine reproduces the shift and the stop at a pair boundary.
  It draws every voxel of the column's own runs up to that stop, including any beyond cell
  *Nz − 1*, at the positions the runs give, and stops at the end of the column's own runs
  instead of reading further. Fixed: position every voxel from the cell index its runs give, in
  both drawing directions, and draw only cells 0 … *Nz − 1*. Only malformed model files are
  affected.
- **`quirks.voxel_section_sort_ties`** (default: on). Sections with equal nearest depth are reordered by
  the overlap priorities of claim 23 (F8) rather than always kept in file order. F8 covers
  the requested configurations, not every possible mixed-depth tie. Fixed: a stable sort, ascending
  nearest depth with tied sections in file order (file order A, B, C at depths 2, 2, 1 then draws C,
  A, B).
- **`quirks.shadow_double_width`** (default: on). Every column of the shadow footprint marks two pixels side
  by side (claim 30, F10). Fixed: one pixel per column.

- **`quirks.barrel_recoil_layer_reset`** (default: on). After every rules layer that has a section
  for a type, each of the barrel's four recoil values is that layer's own barrel key if it has one,
  and otherwise the turret's corresponding value as it stands after that layer (claim 27). A barrel value set by an earlier rules layer is therefore lost whenever a later rules layer has
  the type's section without that barrel key. Fixed: a rules layer changes a barrel value only through
  its own barrel key; a barrel value that no rules layer sets is the turret's value after the last
  rules layer.
- **`quirks.barrel_only_no_recoil`** (default: on). A building that draws its barrel without a
  turret (claim 37) never shows recoil, and its recoil cycle does not stop image reuse. Fixed: slide
  the barrel back by its recoil slide along its own forward axis, as claim 27 does for a barrel
  drawn with a turret, and draw it fresh while its cycle runs.
- **Round-toward-zero mode** (switch `quirks.chop_rounding`, owned by S-0205 (claim 5); not defined here). Every floating-point operation and every single-precision store in
  this spec rounds toward zero (F0). With that switch off they round to nearest. Conversions from floating point to integers are still truncations toward zero, since
  each formula states them. The tabulated sine values of F1 do not change. In the table square
  root the stored input follows the switch, as every store does, while the final cut of the
  result stays a truncation. Example: F2's row z, x entry becomes 0.611342072 instead of 0.611342013.

### Lead-confirmed presentation departures

The project lead confirmed each original bug below as a presentation-only departure.
Each is fixed unconditionally, with no quirk switch.

- Original reused shadows retained old own-image cutouts instead of excluding the current pose (claim 39).
- Original voxel positions wrapped at image edges and reappeared on the opposite side (claim 22).
- Original partial colour clearing let unrelated earlier models contaminate fresh images (claim 34).
- Original barrel pivots wandered with twice the parent angle due to a misplaced coordinate frame (claim 10).
- Original frame keys reduced modulo 32 or 256 substituted earlier motion frames (claims 26 and 37).
- Original building turret recoil left its barrel behind (claim 27).
- Original horizontal position arithmetic carried into vertical position, moving some pixels down (F5).
- Original barrel keys omitted elevation, freezing an earlier pitch (claims 6, 26 and 37).
- Original depth-option shadows enabled depth mode for unrelated later draws, frames and games (claim 32).
- Original turret keys combined alternative-model state with frame parity, colliding across models and frames (claim 26).
- Original depth-zero voxels failed the strict test and wrapped depths reversed coverage order (claim 24).
- Original partial depth clearing let unrelated stale depths suppress fresh voxels (claim 34).
- Original missing-alternative-body cases drew the turret as the body or omitted the body entirely (claim 12).
- Original recoil hold duration one skipped its only hold tick (claim 27).
- Original refiring added another full travel distance, causing unbounded visual drift and a final jump (F7).

## Import mapping

All keys are optional and are read per vehicle, aircraft or building type. "Rules [type]"
means the rules file, in the type's own section. "Art [image]" means the art file, in the
section named by the type's image. "Art [type]" means the art file, in a section with the type's
own name, even when the type's image names a different section.

| Key | File and section | Parse | Default | Feeds |
|---|---|---|---|---|
| `Turret` | Rules [type] | boolean | no | Whether a turret part is placed and rotated (claims 4, 8) |
| `TurretOffset` | Art [image] | integer, leptons | 0 | Turret displacement *δ* (claims 8, 9); 0 also drops the body facing from the turret's cache key (claim 26) |
| `TurretAnimX`, `TurretAnimY` | Rules [type], buildings | integers, screen pixels (x to the right, y downward) | 0 | Turret anchor of a building's voxel turret and barrel (Terms, claims 9 and 37); the same keys place the turret slot animation of S-0108 |
| `TurretRecoil` | Rules [type] | boolean | no | Enables S1 for buildings (claims 27, 28) |
| `TurretTravel`, `TurretCompressFrames`, `TurretHoldFrames`, `TurretRecoverFrames` | Rules [type] | integers | 2, 1, 1, 1 | F7 for the turret |
| `BarrelTravel`, `BarrelCompressFrames`, `BarrelHoldFrames`, `BarrelRecoverFrames` | Rules [type] | integers | the turret's corresponding value; in each rules layer that has the type's section, an absent barrel key takes the turret's value as it stands after that layer, replacing any barrel value from an earlier rules layer (claim 27, `quirks.barrel_recoil_layer_reset`) | F7 for the barrel |
| `UseBuffer` | Art [type] | boolean | no | Per-type depth buffering for fresh part draws only (claims 24 and 32); shadows never enable it. |
| `ShadowIndex` | Art [image] | integer | 0 | Shadow section (claim 29) |
| `NoShadow` | Rules [type] | boolean | no | Suppresses the shadow (claim 31); no-shadow control for claim 39 |
| `ConsideredAircraft` | Rules [type] | boolean | no | Half-size shadow (claim 31) |
| `DisableVoxelCache`, `DisableShadowCache` | Art [image] | boolean | no | Disable reuse of part images and of shadow images (claims 26, 31) |
| `FireAngle` | Rules [type] | integer, 1/256 turn (256 facing units), used modulo 256 | 8 | Barrel elevation target (claim 6) |
| `NoSpawnAlt` | Rules [type] | boolean | no | The alternative-body option: alternative body while none of the vehicle's spawned aircraft is docked (claim 12) |
| `TurretCount` | Rules [type] | integer | 0 | Turret count; above 0 makes a multi-turret type unless the type has rotating-barrel weapon stages (Terms, claim 12) |
| `IsGattling` | Rules [type] | boolean | no | Rotating-barrel weapon stages; here only its effect on the multi-turret rule (claim 12) |
| `UseTurretShadow` | Art [image], vehicles | boolean | no | Read, no effect (`quirks.no_turret_shadow`) |

**Importer warnings.** The importer warns, naming the file and the place in it, in these cases. A
warning never changes the imported result (glossary, *importer warning*), and the warnings do not
depend on the state of any quirk switch.

| Input | Warns | Claim |
|---|---|---|
| A vehicle type with the alternative-body option whose rules give it no spawned aircraft | the type | 12 |
| A model whose part or shadow images would have a copy-rectangle side greater than 4096 pixels, from its imported bounds and applicable poses | once per model, naming the model file and an exceeding pose, whether part, shadow or both exceed the limit | 22 |
| A voxel model section of normal mode 0 | the model file and the section | 35 |
| A mode-0 section in which a column's reading reaches past its own column data, in which an empty count is *255 − Nz* or more outside depth mode or *Nz* or more in depth mode, or whose *Nz* is 255 outside depth mode | the model file, the section and, for the first two, the column | 35 |
| A section with a column whose runs do not add up to exactly *Nz* cells (the total of all its empty and voxel counts, whatever the stop rule of claim 36 does with it) | the model file, the section and the column | 36 |
| A lighting table file that ends before the first entry of row 0, so it supplies no row | the file | 38 |
| A lighting table file that ends before the last of the rows its own row count gives | the file | 38 |
| A lighting table file with *R* below 32 or above 128 | the file | 38 |
| A lighting table file with data after its last row | the file | 38 |

The importer finds model cases by reading columns as claims 35 and 36 describe.
Crossing the old 256-pixel edge alone or an old failed-clear-fit condition gives no
extra warning: claims 22 and 34 enlarge logical images and fully initialise fresh draws.
Exceeding the new 4096-pixel limit gives the single model warning above. Repeated poses,
appearances and part/shadow exceedances of the same model do not duplicate it.

No additional INI keys select section ties, mode-0 spans or the own-image exclusion.
The following model-file concepts supplement the key mapping; absence is governed by
S-0007's model-file lookup, not an INI default.

| Input concept | File | Form | Default | Feeds |
|---|---|---|---|---|
| Section sequence and bounds | VXL model | ordered sections and single-precision bounds | none; required model data | full-sequence nearest-depth relations and F8 priorities, including Q-0215 questions 1–6 (claims 23, 24); MS-28 investigates question 7, with no additional input or absent-key default |
| Section normal mode and grid height | VXL model | integers in 0 … 255 | none; required model data | mode 0 selection and Nz (claim 35) |
| Column value sequence and its start/end | VXL model | ordered values in 0 … 255, or an empty-column marker | none; required model data | colour-height relation, with top-down empty counts following each span's colours (Q-0216), and bounded-data warning (claims 35, 36) |
| Per-section pose | HVA companion | frame-indexed transforms | file lookup and failure treatment in S-0007 | transformed bounds for F8 and own-image overlap in claim 39 |
| Section bounds, scale and grid dimensions; applicable per-section pose | VXL model and HVA companion | single-precision bounds, scale and transform values; integer grid dimensions | required model data; no new INI key or absent-key default | F5/F6 part placement and F10 shadow placement, 4096-pixel retained window and oversized-model warning; non-finite placement suppresses the affected image (claim 22) |

The voxel lighting table is the archive file `voxels.vpl`, looked up like other game files (S-0005). Claim 38 gives how its content is used. Its byte layout is given by S-0013. The VXL and HVA byte layouts are public; this spec only fixes their interpretation.

## Measurement scenarios

**Common setup.** Every scenario uses the glossary's measurement setup: an unmodified install with
no loose override files and no engine add-ons, and stock data. On top of that: 800×600, a tiny
flat clear purpose-made map with neutral map lighting, the game paused for captures, and
screenshots saved losslessly. Each game is a skirmish with one human player owning the test
objects unless a scenario names another owner, and one distant bystander computer player.
The observed area is the visible test objects and their routes; apply the glossary's bystander
void rule. Use game speed 6 unless a scenario says otherwise, director off, and mods off except
the named test data. A screen recorder with read-only tick counts supplies captures; any
scripted input acts only through normal mouse and keyboard controls. Record the display path
used (glossary, *display path* and *measurement setup*).

- **Golden image** compares the original's screenshot with an engine rendering for
  unchanged behaviour. For lead-confirmed departures, engine-conformance comparisons
  instead test this spec's mathematical prediction, not the original's buggy pixels.
  In mixed scenarios, oracle backing covers only unchanged portions.
- **Runs and initial state.** Rendering is deterministic, so one run per variant is enough.
  Unless a scenario explicitly carries effects between games, each variant starts with a fresh
  program session, not merely a new game. This provides off depth mode and no earlier images
  without assuming that the original invalidates images at every game start. A golden-image
  mismatch is repeated once, to rule out a capture error, before it counts.
- **Per-tick captures** mean captures indexed by the recorder's tick count at game speed 6,
  retaining one image for each tick. Pause only after the observation for still-image inspection,
  do not assume a single-tick stepping control exists. A run missing a required tick capture is
  void and repeated.
- **Discriminating colours.** A test colour index used to distinguish named shade rows must
  have different visible palette colours in those rows under the test lighting and house colour.
  Check this when making the golden images; if the rows cannot be distinguished, that sub-test
  is inconclusive rather than a confirmation. For overlap probes, choose nontransparent colours.
- **Test data.** Some scenarios need extra data, as the glossary allows. It is removed
  afterwards, and it is placed as S-0007's scenarios place it:
  - *Test map override section:* rules fields set in the test map's own override section
    (S-0006).
  - *Base rules copy:* a copy of the stock base rules with only the changes the scenario lists,
    in the test expansion archive. The test expansion archive must be numbered above the
    expansion archive that holds the stock base rules, so that its copy wins (S-0005, claims 9 and
    18).
  - *Art database copy:* a copy of the stock art database with only the additions the scenario
    lists, in a test expansion archive, where it wins over the stock one (S-0005).
  - *Test models:* purpose-made VXL and HVA files in the same test archive, under a new image
    name, so they replace nothing.
  - *Test lighting table files:* MS-22 and MS-23 use loose copies of the stock lighting table file,
    changed as each scenario lists, that override the stock one, as the glossary's *named test
    files* allow. The install itself is not modified.
  - *Giving a stock type test models:* for a vehicle's body, turret and barrel, the test map's
    override section sets the type's image to the new image name (S-0007, claim 1), and the art
    database copy has an art section for that image. S-0007's claim 10 does not interfere: it
    matters only on snow maps, for types with the alternate-arctic-art flag, and its arctic
    suffix never applies to voxel types (S-0007, claim 9). A building's voxel turret and barrel
    are named by its turret-animation name instead (S-0007, claim 20), so for them the override
    section sets that name, and for the barrel role with the barrel flag the voxel barrel file
    name, to the new name. Art fields read from the section named after the type itself (the
    depth option, import mapping) go into that section of the art database copy.

**MS-1 Body facings and placement (claims 1–3, 7, 25).**
- *Setup:* one stock turretless voxel vehicle, placed by the map at each of the 256 map-file
  facings in turn (or 64 copies at different facings).
- *Observable:* screenshot crops of the vehicle.
- *Expected:*
  - exactly 32 distinct body images,
  - facing values within the same 32-step bucket give identical pixels,
  - the east-facing image matches our unrotated render pixel for pixel,
  - all 32 match the golden images, which include the 0.088° shortfall of facings 9–31 (F1).
- *Falsified if:* more than 32 distinct images appear, there is a constant pixel shift, or a
  mismatch goes away when we render with exact angles.

**MS-2 Turret facings, placement and order (claims 4, 6, 8, 10, 11, 26).**
- *Setup:* one stock vehicle with a turret and a barrel part and a non-zero turret displacement.
  Its body is held at one facing, and targets are placed around it so its turret points at each
  of them in turn.
- *Procedure:* start a fresh game (empty image caches). Hold the turret still for the first 2
  ticks, then turn it to directions not drawn before, and record for each direction the tick it
  was first drawn.
- *Observable:* crops per turret direction; crops in the first 3 ticks after the unit is placed.
- *Expected:*
  - 32 distinct turret images per body facing,
  - the turret moved forward by the imported displacement × *ρ*,
  - the barrel-versus-turret overlap swaps between the 4-step facings 0/3 and 1/2: the barrel
    covers the turret at 32-step turret facings 5–19, the turret covers the barrel at 21–31 and
    0–3. Facings 4 and 20 straddle a 4-step boundary at which the order changes and may show
    either order. Facings 12 and 28 also straddle a 4-step boundary, but the order is the same on
    both sides of it,
  - the barrel rises as claim 6 says (default fire angle): turret directions first drawn in the
    first tick after placement show it level, and directions first drawn from the second tick on
    show it one step above level, including directions already cached at level,
  - the image matches claim 10's turret-origin pivot and barrel frame 0. Pivot and
    current-elevation reuse checks are lead-confirmed departures, engine conformance.
- *Falsified if:* the barrel pivots away from the turret origin, overlap differs, or
  a second-tick-or-later appearance retains an earlier level pitch.

**MS-3 HVA, scale, HVA frames (claims 13–16).**
- *Setup:* a stock unit with a multi-frame body HVA (for example a walking voxel unit or an
  animated body), filmed while it moves.
- *Observable:* the sequence of body crops, per drawn frame.
- *Expected:*
  - body frames repeat with period *F_body* in walk-counter steps,
  - they match our renders, which use translations scaled by the first section's factor,
  - any barrel part stays in HVA frame 0.
- *Falsified if:* section parts drift from their predicted spots, or a barrel animates.

**MS-3b Complete cache frames and turret frames (claims 16, 26).** Complete-key
checks are engine conformance for lead-confirmed departures; frame selection remains
an unchanged oracle-checkable rule.
- *Test data:*
  - test models for one stock turreted vehicle type,
  - a body HVA of 40 frames, in which one small marker section takes a different, clearly
    visible position in each frame,
  - a turret HVA of 3 frames that look clearly different.
- *Procedure:* in a fresh game, drive the vehicle in a straight line at one facing on flat
  ground for more than 40 walk-counter steps, capturing every drawn frame. Repeat with the
  cache-disable option added to the test image's art section.
- *Observable:* the marker position and the turret frame in each capture.
- *Expected:*
  - With or without caching, frames 32–39 show their own marker positions,
    never the positions of frames 0–7.
  - Whenever the body frame *f_b* is non-zero, the turret shows frame *f_b mod 3*.
- *Falsified if:* frames 32–39 substitute frames 0–7 under either cache setting,
  or the turret frame does not follow the body frame while
  *f_b ≠ 0*.
- *Second model set (complete turret frame).* The same type gets a body HVA and a turret HVA of
  260 frames each, and the turret, like the body, has a small marker section that takes a
  different, clearly visible position in each frame. The same procedure, for more than 260
  walk-counter steps, with and without the cache-disable option.
  - *Expected:* every body/turret frame shows its own marker, including 32–39 and
    256–259, under both cache settings.
  - *Falsified if:* a requested frame substitutes any other marker.
- *Building rendering fixture:* In engine conformance, render a building turret with
  a 260-frame marker HVA at applicable frames 0–259, then repeat with caching disabled.
  Supply frames directly to the rendering test, not the oracle simulation. Every
  requested marker must match; frames 256–259 substituting 0–3 falsifies complete keys.

**MS-3c Turret frames from the animation-frame counter (claim 16).**
- *Test data:*
  - test models for one stock turreted vehicle type: a body HVA of one frame (so *f_b* is always
    0) and a turret HVA of 3 frames that look clearly different,
  - test models for one stock building with a voxel turret: a turret HVA of 3 frames that look
    clearly different.
- *Procedure:* in a fresh game, drive the vehicle around for a few hundred ticks without letting
  it attack, then order it to attack a target for a few hundred ticks. Leave the building idle
  for a few hundred ticks, then let it attack a target. Capture every drawn frame.
- *Observable:* the turret frame shown in each capture.
- *Expected:*
  - Before its first attack, each turret shows frame 0 in every capture, however far the
    vehicle drives.
  - Once attacking starts, each turret's frame changes, and it changes only while its owner is
    attacking.
- *Falsified if:* a turret shows frame 1 or 2 before its owner's first attack, the vehicle's
  turret frame changes with driving alone (a sign of the walk-animation counter), a turret
  cycles through its frames at a steady rate on its own (a sign of an independently playing HVA),
  or a turret never leaves frame 0 during a long attack.

**MS-4 Lighting (claims 17–19).**
- *Setup:* a stock vehicle on flat ground at all 32 facings, with neutral map lighting.
- *Observable:* the palette index (or colour) of each pixel.
- *Expected:* the golden images match exactly, using F4 with the constants above. Rendering
  with the eye vector *Rᵀ(0,0,1)* instead (`quirks.voxel_eye_in_model_frame` off) changes the highlights, which shows the
  quirk.
- *Falsified if:* there is any systematic shade difference.

**MS-5 Rasterisation order (claims 21–25).**
- *Setup:*
  - stock vehicles with several sections, viewed at all facings,
  - a stock type with the depth option, drawn before any depth-option shadow has appeared,
    with nothing else on screen.
- *Observable:* pixel crops.
- *Expected:*
  - exact agreement with F5's independent-coordinate prediction, claim 21's voxel
    priority and claim 23's section priority (ties in MS-5b); removed carry artefacts
    are lead-confirmed departures checked against this spec, not oracle pixels,
  - the depth-option type shows 2-pixel-wide voxels.
- *Falsified if:* any repeated golden-image mismatch remains with the model and pose verified,
  including a systematic one-pixel shift, a wrong within-section overlap winner, or coverage
  between distinct-depth sections contrary to claim 23.

**MS-5c Within-section pixel priority (claims 19, 21, 24).**
- *Setup:* Common setup, speed index 2. One human-owned parked stock turretless vehicle
  gets named test VXL/HVA models and an art database copy disabling part caching. A named
  test-map override suppresses its shadow. No other voxel object is visible. Each model
  has one well-formed section and exactly two occupied cells with distinct nontransparent
  output colours X and Y. Choose bounds and HVA scale so F5 maps both to the same left
  image pixel and, in depth mode, the same nonzero comparison depth. Check these conditions
  separately for each mode, with independent components in both. The copy rectangle contains
  both output pixels. There is no third voxel whose right-hand write can interfere.
- *Fixtures:* Define the cells by distances from the farthest corner. Use three pairs:
  X at (0, 1, 0) with Y at (1, 0, 1), X at (1, 0, 0) with Y at (0, 0, 1),
  and X at (0, 0, 1) with Y at (0, 0, 0), in *(n_x, n_y, n_z)* coordinate order.
  X has the greater priority tuple in every pair. These respectively discriminate y
  precedence over x and z, x precedence over z, and z precedence at equal x/y.
  Use valid normal indices and verify X and Y remain visually distinct after lighting.
  Repeat with an upside-down HVA pose and cells selected to retain those far-corner
  distances, so top-down positioning is exercised too.
  In a separate depth-mode fixture, retain a greater tuple for X but tune the bounds and
  pose so X's left pixel is immediately left of Y's left pixel, with X's nonzero comparison
  depth smaller than Y's. X's left pixel must not be touched by Y. Its right-hand write
  then probes replacement of Y's larger depth without a test at that neighbour.
- *Procedure and observable:* Restart the program for every fixture, pose and mode.
  Capture the first visible drawn frame losslessly and record the common pixel's colour.
  Run once without the depth option and once with it. Each fresh depth-mode image
  starts empty, below every numeric voxel depth; no shadow contributes depth.
- *Expected:* the common pixel shows X without depth buffering and Y with it. The
  depth-mode right-hand pixel also shows Y. Repeat the non-depth fixtures with X's
  lighting-table output changed to 0 by a named test table, leaving Y nonzero: the common
  pixel is transparent, not Y. In the separate right-hand-write fixture, Y's left pixel
  shows X despite Y's greater depth, while Y's right pixel retains Y. Golden-image checks
  establish projection and placement.
- *Falsified if:* a verified common pixel shows Y in the ordinary nontransparent run,
  X in the equal-depth run, or Y in the zero-output run, or Y's left pixel retains Y in
  the right-hand-write fixture. A golden-image mismatch in either
  pose also falsifies the predicted positioning. Incorrect fixture coincidences, equal
  visible colours or a missing first-frame capture make a run inconclusive, not a pass.

**MS-5b Section tie order (claim 23, F8; `quirks.voxel_section_sort_ties`).** Stock models need
not contain the one pattern in which F8 and a stable sort differ, so this scenario builds it.
- *Test data:* one stock voxel vehicle type without a turret and without the depth option (the
  test type) gets test models, one model at a time (Common setup, *Giving a stock type test
  models*). The test map override section also gives it the no-shadow option, so no shadow
  overlaps the crop. No type with the depth option is on the map. For each model the game program
  is started afresh to exclude earlier cached images. Corrected depth mode stays off
  outside fresh depth-option part draws (claim 32).
  - **Tie model (T1).** Three sections, in this file order:
    - *A:* a solid block of a few cells in every direction (for example 4 × 4 × 4 cells, every
      cell a voxel, with bounds of about one model unit per cell, claim 15), every voxel with
      colour index *X* and one normal index *K*,
    - *B:* the same grid size, bounds and voxels as A, every voxel with colour index *Y* and normal
      index *K*,
    - *C:* the same grid size and the same x and y bounds as A, with both z bounds lowered by at
      least one cell height of A, every voxel with a third colour index *W* and normal index *K*.
    All three sections have the same scale factor and the same HVA transform (the identity in
    every frame), so by claim 13 A and B are drawn with identical transforms and have exactly equal
    nearest depth. On flat ground the body turns only about the vertical axis, and a lower model z
    lowers the view-space depth (row z of F2 has a positive z entry), so C's nearest depth is
    strictly smaller than A's and B's at every body facing; the margin of at least half a model
    unit is far beyond the single-precision truncation of F0. *X* and *Y* are colour indices
    outside the house-colour range whose lighting-table entries differ in the row of the shade
    that normal *K* receives (claim 18), so the two give clearly different screen colours.
  - **Control model (T2).** T1 without section C (file order A, B).
- *Procedure:* place one test-type vehicle at body facing east on the flat map and capture it in
  the first drawn frame that shows it, once with T1 and once with T2.
- *Observable:* the colour of the pixels where A and B are drawn (their silhouettes coincide,
  since their voxels and transforms are identical).
- *Expected:*
  - T1 (F8 order C, B, A, so A is drawn last and covers B everywhere): those pixels show *X*'s
    colour.
  - T2 (order A, B under F8 and under a stable sort alike): those pixels show *Y*'s colour.
  - Both captures match the golden images, which use 1-pixel-wide voxels (a depth-buffered draw,
    claim 24, would make each voxel 2 pixels wide and so widen the silhouette's right edge).
- *Falsified if:* any qualifying A/B overlap pixel shows Y in T1 or X in T2, or differs
  from its golden image on repetition with the file order and initial mode verified. T1 showing
  Y with T2 showing Y specifically matches stable priority.
- *Invalid run:* discard only if inspection establishes incorrect file order, an unwanted
  depth-option object, or a missing first-drawn-frame capture. A wider silhouette alone is
  not a setup error: with those controls satisfied, it is a rasterisation falsifier.

**MS-6 Building voxel turret (claims 5, 6, 9–11, 26, 27).** Pivot, barrel-following,
one-tick holds and bounded refire are lead-confirmed departures, engine conformance.
Direction, shift, overlap and layer imports remain oracle-checkable.
- *Setup:*
  - a stock building with a voxel turret and a barrel: one with both a turret model and a barrel
    model, that is, a turret-animation name that has a late "TUR" in the sense of S-0007
    claim 20 (the letters "TUR" in any case, starting at the name's fifth character or later).
    A defence whose
    only model is in the barrel role does not qualify; claim 37 and MS-21 cover those,
  - if no stock building has a non-zero turret displacement, the art database copy gives it
    one: first *δ = 20*, then *δ = −12*,
  - a test map override section sets its recoil fields to travel 3, compress 2, hold 3,
    recover 4. Unless a run says otherwise, every run sets each recoil value through both the
    turret key and the barrel key, so that the barrel's values do not depend on the base rules
    (claim 27).
  - It fires at a target that moves around it. For the single-shot recoil sequence, use a shot
    that no other shot follows within 12 ticks (checked on the captures).
  - **Hold runs.** The same building with hold 1, then hold 2 (travel 3, compress 2, recover 4
    kept), then with every recoil value absent (the defaults), each firing single shots as
    above. For the defaults run, a base rules copy removes the building's eight turret and barrel
    recoil keys, the test map override section sets none of them, and the game-mode file used has
    no section for the building, so that no rules layer sets a recoil value.
  - **Negative travel run.** Travel −3, compress 2, hold 3, recover 4, single shots.
  - **Refire runs.** A stationary target that survives many shots (for example a building of an
    enemy house), and a test map override section that shortens the building's weapon's
    rate-of-fire field. A map trigger removes the target some ticks after the third shot.
    - *Hold refire:* recoil travel 3, compress 2, hold 12, recover 4, and a rate of fire that
      puts successive shots 3 to 14 ticks apart, so that every later shot lands in the hold of
      the cycle before it.
    - *Compress refire:* recoil travel 8, compress 8, hold 2, recover 2, and a rate of fire that
      puts successive shots 1 to 7 ticks apart, so that every later shot lands in a compress
      phase.
    - The shot ticks are read off the per-tick captures (the first tick in which the shot's
      projectile or muzzle flash shows).
  - **Layer runs (claim 27, `quirks.barrel_recoil_layer_reset`).** A base rules copy gives the building's section the recoil
    option on, turret travel 0 and no other turret recoil key, and barrel travel 3, compress 2,
    hold 3, recover 4; it removes any other recoil key of the building. The game mode used has a
    game-mode file without a section for the building (otherwise that layer already resets the
    barrel values). The building fires single shots as above.
    - *Control:* the test map has no section for the building.
    - *Turret only:* the test map override section sets the four turret recoil values to travel 6,
      compress 2, hold 3, recover 4, and no barrel key.
    - *Section only:* the test map override section has a section for the building that sets
      only its recoil option to yes, and no travel, compress, hold or recover key.
  - For the overlap check, a second run gives the building test models for its turret and
    barrel: each a solid block of voxels of one colour index, *X* for the turret and *Y* for the
    barrel, with the barrel's inner end inside the turret block, so that the two images overlap
    at every direction.
  - **Anchor run.** The same building with the test map override section also setting its
    turret-animation x and y displacements to 10 and −6 (import mapping), compared with a run
    in which both are 0, at the same turret directions.
- *Observable:*
  - turret and barrel position and pitch per direction,
  - per-tick captures from the shot until the recoil ends; in the refire runs, from the first
    shot until the recoil slide is back at 0,
  - in the overlap run, the colour shown where the turret and barrel images overlap, per 32-step
    turret facing.
- *Expected:*
  - **Directions.** 32 turret directions.
  - **Turret shift.** The turret sits 2 model units forward (−1 for *δ = −12*) along its own
    facing, so it circles the anchor as it turns.
  - **Barrel.** It pitches about the turret origin in 32-step values.
  - **Recoil.** After the shot, the turret's recoil slide over successive ticks is 1.5, 3, 3, 3,
    3, 2.25, 1.5, 0.75, 0 model units (F7, S1). The sequence starts on the shot tick or the next
    one (not fixed here). Each barrel cycle has the same sequence, but its displayed
    displacement is twice the turret slide, following both cycles. Add a run with
    barrel travel zero: it still follows turret recoil.
  - **Hold runs.** Hold 1: 1.5, 3, 3, 2.25, 1.5, 0.75, 0; hold 2:
    1.5, 3, 3, 3, 2.25, 1.5, 0.75, 0. Defaults: each cycle slides 2, 2, 0.
  - **Negative travel.** Each cycle has the base sequence's magnitudes (1.5, 3, 3,
    3, 3, 2.25, 1.5, 0.75, 0), with negative slides. The turret moves forward and
    the accompanying barrel moves forward by the sum of both cycle magnitudes.
  - **Hold refire.** Each cycle stays at travel 3; after the last shot, two compress
    ticks at 3, twelve hold ticks at 3, then 2.25, 1.5, 0.75, 0.
  - **Compress refire.** Restart uses F7's remaining-distance step and never exceeds
    travel 8. From slide 3: new compress ticks 3.625, 4.25, 4.875, 5.5, 6.125,
    6.75, 7.375, 8, then 8, 8, 4, 0. Also verify refire during recovery, using
    *(8 − λ_r) / 8* as the new step.
  - **Layer runs.** *Control:* the turret does not recoil, and the barrel's per-tick recoil slides
    are 1.5, 3, 3, 3, 3, 2.25, 1.5, 0.75, 0. *Turret only:* each cycle slides
    3, 6, 6, 6, 6, 4.5, 3, 1.5, 0 (barrel takes turret values); the barrel image
    displacement is twice that because it follows the turret too. *Section only:* neither the
    turret nor the barrel recoils (the barrel takes the turret's travel 0).
  - **Overlap (claim 11).** The overlap shows *Y* (barrel drawn second) at 32-step turret facings
    5–19, and *X* (turret drawn second) at 21–31 and 0–3. Facings 4 and 20 straddle a 4-step
    boundary at which the order changes and may show either.
  - **Anchor run (claim 9).** At every direction the turret and barrel images are those of the
    run with displacements 0, moved exactly 10 pixels right and 6 pixels up.
- *Falsified if:*
  - the turret shift matches *δ × ρ* (3.31 or −1.99 model units) or does not turn with the
    turret,
  - there are more than 32 turret directions,
  - the per-tick recoil slides depart from the sequences above,
  - hold 1 fails to provide exactly one hold tick or defaults differ from 2, 2, 0,
  - with negative travel, the parts slide back, or do not move,
  - a refiring cycle exceeds its travel magnitude, restarts from zero instead of its
    current slide, or differs from the remaining-distance formula,
  - the barrel fails to follow turret recoil in addition to its own cycle,
  - in the control layer run, the barrel does not recoil or the turret does (the base rules copy
    was not read); in the turret-only run, the barrel's own cycle keeps 1.5, 3, 3, 3, 3,
    2.25, 1.5, 0.75, 0,
    or in the section-only run, the barrel recoils at all (in both cases the barrel kept the base
    rules copy's values, as the fixed behaviour of `quirks.barrel_recoil_layer_reset` would),
  - in the overlap run, any facing other than 4 and 20 shows the other colour, or the same colour
    shows at every facing,
  - in the anchor run, the images move by any other amount, do not move, move by a different amount
    for the turret and the barrel, or change in shape.

**MS-7 No vehicle recoil (claim 28).**
- *Setup:* a stock vehicle whose type has the recoil option (if none exists, set it with a test
  map override section), firing.
- *Observable:* turret crop in each tick after a shot.
- *Expected:* no recoil slide.
- *Falsified if:* there is any shift.

**MS-8 Shadows (claims 29–31; `quirks.shadow_double_width`).**
- *Setup:*
  - a stock voxel vehicle with an animated body, and a stock aircraft-like type, on flat ground
    at several facings, then on a ramp,
  - a voxel vehicle that can cloak (a stock one if any exists, otherwise set the cloak field with a
    test map override section),
  - a voxel vehicle given the no-shadow option by a test map override section,
  - a stock voxel vehicle whose body has at least two sections, with its shadow section set to
    1 in the art database copy,
  - one stock turretless vehicle type given a test model whose single section has three kinds of
    column, each in a separate, clearly visible group: columns with voxels, columns the file lists
    as present whose runs hold only empty cells, and columns the file marks as empty.
- *Procedure:* capture each unit; order the cloaking vehicle to cloak and to uncloak, capturing
  every tick (per-tick captures).
- *Observable:* shadow pixels, viewed as the owning player.
- *Expected:*
  - the shadow is the 2-pixel-per-column footprint of the shadow section in HVA frame 0, 3
    pixels right of the unshifted footprint, half size for the aircraft-like type, and with no
    turret shadow,
  - on flat ground, every shadow's marked pixels match, pixel for pixel, the golden image built
    from F10 and F6 around the shadow draw point (whose position comes from S-0102 and
    S-0106). On the ramp the footprint keeps its 2-pixel-per-column form but is tilted; its exact
    pixels need the ramp tilt, which is left to S-0106, so the ramp captures are kept for
    then and are not compared here,
  - the cloaking vehicle's shadow is present only while it is fully uncloaked: absent from the
    first tick of cloaking to the last tick of uncloaking,
  - the no-shadow vehicle has no shadow,
  - the vehicle with shadow section 1 shows the footprint of its second section,
  - the test-model vehicle's shadow has pixels for both groups of present columns, those with
    voxels and those without, two pixels per column, and none for the empty columns.
- *Falsified if:*
  - the shadow-section-1 vehicle still shows the first section's footprint,
  - the shadow animates with the body or takes its shape from the light direction,
  - a shadow shows during cloaking, cloaked or uncloaking, or the no-shadow vehicle casts one,
  - a flat-ground footprint differs from the F10 golden image by a constant shift (draw point aside), by a
    one-pixel step pattern (a carry or a different rounding), or in its size (the scale applied
    after the shift, or after truncation),
  - the test-model shadow lacks the present columns without voxels, shows the empty columns, or
    is one pixel per column.

**MS-9 Depth-mode isolation (claims 26, 31–33; S2).** Engine conformance for a
lead-confirmed departure, not a comparison with the original bug.
- *Setup:* Common setup. Named art copies give depth-option turretless V fresh shadows
  every drawn frame, depth-option turreted U no shadow, and watchers W no depth option.
  V/U keep part caching enabled. No other voxel objects enter the observed area.
- *Procedure:* At speed 6 capture W at new facings before V appears by map trigger,
  while V is parked, after V is removed, and after U turns to an uncached turret
  direction. Revisit W's earlier facings. Repeat with V's shadow suppressed. Fire a
  stock voxel-model projectile from a non-depth shooter. Start a second game in the
  same program session with only W visible at an uncached facing. One run per variant,
  lossless tick-labelled captures, director and other mods off.
- *Observable:* W/projectile pixel width and section overlap against claims 21–24's
  renders; U/V part images as positive controls.
- *Expected:* W and projectiles always have their ordinary non-depth look, including
  reused directions and the new game. U/V parts alone have the depth look. Shadow
  freshness, presence or suppression never changes W.
- *Falsified if:* any W/projectile inherits depth mode or carries an inherited look
  into another game. Wrong type options, changed pose or missing captures make the
  affected comparison inconclusive.

**MS-10 Barrel cache and elevation (claims 6, 26).** Current-pitch reuse is engine
conformance for a lead-confirmed departure, not an oracle comparison.
- *Setup:* a fresh game. One stock vehicle with a barrel part is placed by the map at a fixed
  facing, with its turret not turning. A few ticks later, a second vehicle of the same type is
  placed at a different facing.
- *Procedure:* capture both, then turn the first vehicle's turret through new directions.
- *Observable:* the barrel pitch of both vehicles.
- *Expected:* Every appearance shows current 32-step elevation, at unchanged and
  new turret directions, independently for both vehicles.
- *Falsified if:* a barrel retains level pitch after its elevation changes.

**MS-11 Alternative bodies and switchable turrets (claims 12, 26).** Missing-body
fallback and collision-free reuse are lead-confirmed departures, engine conformance.
Other model-selection checks remain oracle-checkable.
- *Setup:*
  - a vehicle type with the alternative-body option that carries spawned aircraft (a stock one
    if any exists; otherwise set the option with a test map override section, and supply its
    alternative-body model as a test model under the name S-0007 gives),
  - a stock multi-turret type (Terms),
  - a stock type with rotating-barrel weapon stages, given a turret count of 2 in a test map
    override section, with test turret models for turret indices 0 and 1 (under the names S-0007
    gives) that look clearly different from its ordinary turret,
  - for the turreted case, a second type with the alternative-body option and spawned aircraft
    that return, given the turret flag in a test map override section, with test models: a body
    with an HVA of 4 frames, and a turret (under the turret's name, S-0007) whose HVA has 4
    clearly different frames and that looks clearly different from the body,
  - **Missing file.** A second copy of the first spawner's set-up in which the alternative-body
    model is absent (no test model under that name and no stock file of that name),
  - **Special type name.** A test vehicle type with the exact type name S-0007 claim 16 names, added to the
    vehicle list of the base rules copy (S-0004) with the fields of the first spawner type, so that
    it carries the same spawned aircraft. Its test models are a body and an alternative body under
    the other suffix of S-0007 claim 16, which look clearly different, and no model under the
    no-spawn-alternate suffix. It is run twice: without the alternative-body option, then with it.
- *Procedure:*
  - The spawner stands still at one body facing from before its first launch until after the
    last return. Let it launch all its spawned aircraft, and keep capturing until every one of
    them has returned or, for spawned aircraft that do not return, been replaced. Holding the
    facing matters: the main body at that facing is cached before the launch, so only a key
    that records the alternative-model state can show the alternative body there.
  - Change the multi-turret vehicle's selected turret index by the means the vehicle specs give,
    with its body and turret held at the facings they had before the change (checked on the
    captures).
  - Capture the rotating-barrel vehicle idle and while attacking.
  - The missing-file spawner and both runs of the type-id vehicle follow the spawner procedure
    above.
  - The turreted spawner stands still until all its spawned aircraft have launched. While none
    is docked, it drives straight at one body facing it has not used before, for at least 8
    walk-counter steps, capturing every drawn frame. Once a spawned aircraft has docked again, it
    drives at that facing again. The whole procedure is repeated with the cache-disable option on
    the test image.
- *Observable:* crops of the body, turret and barrel; for the turreted spawner, the turret frame
  shown at each body frame *f_b*. Only drawn frames in which the turreted spawner's turret shows
  the same 32-step direction are compared with each other.
- *Expected:*
  - the spawner shows its main body while at least one spawned aircraft is docked, and the
    alternative body while none is, at the same facing,
  - the multi-turret vehicle shows the turret and barrel models of its selected turret index,
  - the rotating-barrel vehicle always shows its ordinary turret,
  - the turreted spawner keeps its main body and draws its turret normally,
  - with caching, body frames 2 and 3 show turret frames 2 and 3 respectively in
    both docking states, with no wrong-frame substitution,
  - under the cache-disable option, every body frame *f_b* from 1 to 3 shows turret frame *f_b*,
  - every spawner's shadow keeps the main body's footprint throughout, whatever is drawn in the
    body's place,
  - the missing-file spawner keeps its main body and shadow in both docking states,
  - the type-id vehicle shows its main body throughout the run without the option, and, in the
    run with it, its alternative body while none is docked and its main body otherwise.
- *Falsified if:*
  - the body does not switch with the docking state at the held facing,
  - the turret model does not follow the selected turret index at the held facings,
  - the rotating-barrel vehicle shows a test turret model,
  - the turreted spawner substitutes a turret for its main body,
  - body frame 2 or 3 shows the wrong turret frame under caching in either state,
  - under the cache-disable option, a body frame from 1 to 3 shows another turret frame,
  - a spawner's shadow changes shape when the body is replaced,
  - the missing-file spawner loses its main body or shadow while none is docked,
  - the type-id vehicle shows its alternative body in the run without the option, or keeps its
    main body while none is docked in the run with it.

- *Separate-state key fixture:* In engine conformance, use MS-17's stale-normal
  turret and probe P at fixed facings and one fixed applicable frame. Cache the
  state-off turret with P supplying row 26, then change P to row 1 and render state
  on. It must show row 1; returning off must show cached row 26. Row 26 in both
  states falsifies their separate key component. No oracle state is altered.

**MS-12 Depth-buffer details (claim 24).** Empty initial depth and unwrapped-depth
checks are lead-confirmed departures, engine conformance; tie and right-hand-write
coverage still test unchanged original behaviour.
- *Test data:* one stock vehicle type (the test type) gets test models, one model at a time, and
  the depth option in the art database copy. The test map override section also gives it the
  no-shadow option, and no other depth-option type is on the map. For each model the game program
  is started afresh (not only a new game, since the depth plane carries over between games), and
  the model is captured in the first drawn frame that shows it at the stated facing, so no
  earlier depth-buffered draw has left depths in the depth plane where it is drawn (claim 34).
  - **Equal depths and tie order.** Two sections with identical bounds and identical voxels,
    which differ only in colour: the first section in the file uses colour index *X*, the
    second *Y*. The voxels are a few isolated ones, spaced so that their pixels are at least
    3 pixels apart on screen and none has depth 0. (Isolation keeps the right-hand pixel
    overwrites of claim 24 from lowering a stored depth between the two sections' identical
    voxels.)
  - **Depth 0.** One section of 147 × 1 × 1 cells, stretched by its bounds (claim 15) to about
    294 model units along its x axis, with voxels only in the half of the cells at the min-x
    end. It is drawn at body facing 12 (south-east), where its x axis is turned toward the
    viewer, about 30° from the view axis, so depth changes quickly along it. Fine-tune the length so that, by claim 24, the farthest voxel's depth is 0 and
    the next voxel's is not. Isolate that marker in image x/y so no nearer voxel
    can legitimately cover it.
  - **Two-pixel test.** Two small overlapping sections, arranged so that the right-hand pixel of
    a farther voxel drawn first is later reached by a nearer voxel's left pixel, and the reverse.
- *Observable:* pixel crops at a fixed body facing (east, except where stated).
- *Expected:*
  - **Equal depths.** Only colour *Y* shows. Under the reversed order the second section is
    drawn first, and at equal depth the first drawn stays.
  - **Depth 0.** Isolated zero-depth voxels appear. Add negative-depth isolated
    markers and a coincident pair at depths 255 and 256, verified from F5: negative
    markers pass empty coordinates and depth 256 wins the pair.
  - **All three models.** The golden image matches claim 24 exactly. Renders that test both
    pixels, use "greater or equal", keep the last drawn, or draw the sections in claim 23 order
    each differ from the capture in at least one pixel.
- *Falsified if:* X shows in the equal-depth model, an isolated zero/negative marker
  disappears, depth 255 wins over 256, or verified tie/right-write coverage disagrees.

**MS-13 Grid geometry (claims 13–15, 22).** Enlarged no-wrap images are tested by MS-24.
- *Test data:* one stock vehicle type gets test models, one at a time (no depth option).
  - **HVA scale.** Two sections, the first with scale factor 1, the second with scale factor 2.
    The HVA moves the second section 10 model units along model +x in every frame.
  - **Stretch and gaps.** One section of 10 × 10 × 1 cells whose bounds stretch it to 30 × 30 × 3
    model units, completely filled.
  - **File transform.** One small section, drawn twice in turn: once with the per-section
    transform in the VXL file set to the identity, once set to a 90° turn about model *z* plus a
    move of 20 model units along model +x. The HVA has one frame with the identity transform in
    both cases.
- *Observable:* pixel crops at body facing 8.
- *Expected:*
  - **HVA scale.** The second section sits 10 model units forward of where it would be with no
    translation (first section's factor 1), not 20.
  - **Stretch.** Exactly 100 lit pixels, one per voxel, with unlit gaps between neighbours
    (which are about 2 to 3 pixels apart), at the corners claim 15 gives.
  - **File transform.** The two versions give identical pixels.
  - All three match the golden images.
- *Falsified if:* the second section sits 20 units forward, the stretched plate is filled
  solid or has a different pixel count, or the second file-transform version is turned or moved
  relative to the first.

**MS-14 Barrel elevation timing (claim 6, F9).**
- *Test data:* a test map override section sets the fire angle of one stock vehicle type with a
  barrel part to 20. The art database copy gives that type's image the cache-disable option, so
  that every drawn frame is fresh. Repeat with caching on as a current-pitch
  engine-conformance check, expecting the same sequence.
- *Procedure:* the map places one such vehicle, with its turret not turning. Per-tick captures
  from the first tick the vehicle is drawn.
- *Observable:* the barrel's 32-step pitch in each capture (0 = level, 1 = 11.25° up, 2 = 22.5°
  up).
- *Expected:* starting from placement the pitch steps are 0, 0, 1, 1, 2, 2, 2, … (32-step values
  8, 8, 7, 7, 6, 6, 6). Whichever tick is captured first, pitch step 1 shows for exactly two
  consecutive ticks, and the final pitch step 2 is reached 4 ticks after placement.
- *Falsified if:* pitch step 1 shows for three consecutive ticks (a turn of exactly 768 facing
  units per tick), the final pitch is reached at a different tick, or the final pitch is not
  step 2.

**MS-15 Lighting of test models (claim 18; `quirks.voxel_light_first_section_mode`, `quirks.voxel_light_ignores_hva`).**
- *Test data:* one stock turretless vehicle type gets test models, one at a time; all voxels
  use one colour index *χ*.
  - **First-section mode.** Two separate sections, both filled with voxels of normal index 16.
    In version 1 the first section in the file has normal mode 2 and the second mode 4. Version 2
    swaps the modes.
  - **Light without the HVA.** One section with normal mode 4, filled with voxels of normal
    index 85. Its single HVA frame turns it 90° about model *z*: the rotation rows are
    (0, −1, 0), (1, 0, 0), (0, 0, 1), which take model +x to model +y, with no translation.
- *Observable:* the palette index of the section pixels, at body facing 8 on flat ground.
- *Expected:* by F4 with the light of an unrotated part, normal 16 of the mode-2 table gets shade
  19 and normal 16 of the mode-4 table gets shade 7. Normal 85 of the mode-4 table gets shade 26
  without the HVA turn and 1 with it.
  - **First-section mode.** Version 1: both sections show *VPL(19, χ)*. Version 2: both show
    *VPL(7, χ)*.
  - **Light without the HVA.** The section is drawn turned, but its pixels show *VPL(26, χ)*.
  - All match the golden images.
- *Falsified if:* in either version the two sections show different shades (per-section tables),
  or the turned section shows *VPL(1, χ)* (the light follows the HVA turn).

**MS-16 Shades of normal indices outside the table (claim 20; `quirks.voxel_stale_normal_shades`).**
- *Test data:* two stock turretless vehicle types get test models; all voxels use one colour
  index *χ*.
  - **Type A:** one section with normal mode 1 (16 entries). Some voxels have normal index 85,
    others 253, 254 and 255. The art database copy gives A's image the cache-disable option, so
    A is drawn fresh in every drawn frame.
  - **Type B:** one section with normal mode 4, with voxels of normal index 85. B keeps caching
    on.
- *Setup:* start the game program afresh (not only a new game), then a test map with one A at
  body facing 8 and nothing else that is drawn as a voxel model: no other vehicles or aircraft,
  no buildings with voxel turrets, and no attacking (voxel projectiles and debris also give
  shades, claim 20). One B, at body facing 8, is created later by a map trigger, inside the
  visible area.
- *Procedure:* capture every drawn frame before B appears and after it. Then order B to turn to
  a facing it has not shown before, and keep capturing.
- *Observable:* the palette index of A's normal-85 voxels and of its normal-253 to 255 voxels.
- *Expected:*
  - A's normal-253, 254 and 255 voxels show *VPL(16, χ)* throughout.
  - Before B appears, A's normal-85 voxels show *VPL(0, χ)*.
  - From the drawn frame after B is first drawn at the latest, they show *VPL(26, χ)*: the shade
    F4 gives normal 85 of the mode-4 table for B at body facing 8.
  - After B turns, they show the shade F4 gives normal 85 for B's new facing, and keep it while
    B's image is reused.
- *Falsified if:* A's normal-253 to 255 voxels show another row, A's normal-85 voxels do not
  follow B's fresh draws, or they change while B's image is only reused. If A's normal-85 voxels
  show a row other than 0 before B appears, only the "shade 0 at program start" part is
  falsified.
- *Game-change run (claim 20, no reset at a new game).* The first game ends, after the procedure
  above, at a moment when A's normal-85 voxels show a row other than *VPL(0, χ)*; note that row.
  A new game is then started in the same program session, on a test map that places only one A
  at body facing 8 and, like the first, nothing else that is drawn as a voxel model and no
  attacking.
  - A run in which any other object with a voxel model appears in the new game (starting units
    included), or anything attacks, is discarded.
  - *Expected:* from A's first image in the new game on, its normal-85 voxels show the noted row,
    and keep it, since no draw in the new game gives index 85 a shade.
  - *Falsified if:* they show *VPL(0, χ)* or any row other than the noted one.
- *Projectile run (claim 20, projectiles and debris).* The program is started afresh.
  - *Test data:* type A′ is A with other voxels: one voxel for each normal index 16 to 244, each
    on a pixel of its own at body facing 8, all with colour index *χ*. A stock projectile type that is
    drawn as a voxel model gets a test model of one section of normal mode 4 (245 entries, so
    its table covers every index from 16 to 244): the test map override section sets its image
    to a new image name, as the common test-data rules give for vehicles, and the art database
    copy gives that image an art section that keeps it a voxel model. The test map override
    section also gives the weapon of a stock infantry type this projectile type, which keeps its
    own flight settings. Infantry are drawn as sprites, so the shooter gives no shades.
  - *Setup:* a test map with one A′ at body facing 8, the infantry shooter and, in its range, a
    target that is drawn as sprites and survives several shots (for example an infantry unit of
    an enemy house). Nothing else on the map is drawn as a voxel model.
  - *Procedure:* capture every drawn frame before the first shot, while the projectile flies and
    after it hits.
  - *Expected:* before the first shot all of A′'s voxels show *VPL(0, χ)*. From the drawn frame
    after the projectile is first drawn at the latest, at least one of them shows another row.
    Whatever rule lights the projectile (its own specs), a lit model gives a shade above 0 to
    some of the 245 directions of its table, and every one of them is an index that A′ shows.
  - *Falsified if:* every voxel of A′ shows *VPL(0, χ)* throughout, although the projectile was
    seen on the captures. If A′'s voxels already show a row other than 0 before the first shot,
    the run is void, since something else gave shades.

**MS-17 Fresh draws and the cache key (claims 20, 26).**
- *Principle:* a part whose voxels use a normal index outside its own table shows the shade that
  index had when the part's image was made (claim 20). A fresh image takes the index's current
  shade, and a reused image keeps the old one. So such a part shows, draw by draw, whether its
  image was made fresh.
- *Test data:*
  - **Probe P:** one stock turretless vehicle type gets a test model: one section with normal
    mode 4, filled with voxels of normal index 85 and colour index *χ*, so that P's own pixels
    show the current row of index 85. The art database copy gives P's image the
    cache-disable option, so P is drawn fresh in every drawn frame and keeps setting index 85 to
    the shade F4 gives it at P's current body facing: 26 at body facing 8 and 1 at body facing 16
    (F4 with the light of claim 17; the exact values are about 26.5 and 1.4, clear of the
    truncation steps).
  - **Subject B:** one stock vehicle type that has a turret and the driving locomotor gets test
    models for its body and its turret (and its barrel, if it has one), each one section with
    normal mode 1 (16 entries), filled with voxels of normal index 85 and one colour index *χ*,
    and single-frame HVAs. B keeps caching on. In run 1 its turret displacement is non-zero (the
    art database copy sets one if the stock value is 0); in run 2 the art database copy sets it
    to 0.
- *Setup:* start the game program afresh, then a test map with flat ground and one ramp cell
  that leads up to flat ground beyond it, so that B can cross it in a straight line. The map
  places one P at body facing 8. A map trigger creates B a few seconds later, on flat ground
  inside the visible area, so that P has already set index 85 when B's first images are made.
  Nothing else on the map is drawn as a voxel model, and nothing attacks (claim 20).
- *Procedure:* capture every drawn frame.
  - After B has been drawn, P turns to body facing 16 and stays there.
  - **Body facing.** B turns its body to a 32-step facing it has not shown before, then back to
    its first facing.
  - **Ramp state.** B drives onto the ramp cell along its first facing and stops there.
  - **Turret's body facing.** A map trigger creates a second B, called B2, on flat ground at a
    body facing that B has not shown. B2's turret is brought to the 32-step direction that B's
    turret showed before P turned (checked on the captures; for example, both turned toward one
    point).
  - **Unsettled tilt.** P is ordered to turn back and forth between body facings 8 and 16 without
    stopping, so the shade of index 85 changes from drawn frame to drawn frame. Meanwhile B
    drives on along the same facing, off the ramp onto the flat ground beyond it, and stops.
- *Observable:* the palette index of B's and B2's body and turret pixels in each drawn frame.
- *Expected:*
  - Before P turns, B's parts show *VPL(26, χ)*. Right after P turns, they still show
    *VPL(26, χ)*, because their images are reused.
  - **Body facing.** At the new facing, B's body shows *VPL(1, χ)*. Back at its first facing it
    shows *VPL(26, χ)* again (the old image).
  - **Ramp state.** Once B stands on the ramp with a settled tilt, its body shows *VPL(1, χ)*:
    the ramp state is in the key, so the image there is new.
  - **Turret's body facing.** B2's body shows *VPL(1, χ)*. Its turret, at B's old turret
    direction, shows *VPL(1, χ)* in run 1 (the body facing is in the key) and *VPL(26, χ)* in run
    2 (the key leaves out the body facing, so B's old turret image is reused).
  - **Unsettled tilt.** While B's tilt changes on the way off the ramp, its body's row follows
    P's row, allowing for one drawn frame of lag: each image is fresh. Once B stands on the flat
    ground beyond with a settled tilt, its body shows *VPL(26, χ)*, the flat-ground image made
    at that facing at the start, whatever P does.
- *Falsified if:*
  - B's parts change to *VPL(1, χ)* while B changes nothing (no reuse at all),
  - the new body facing or the settled ramp shows *VPL(26, χ)*,
  - B2's turret shows *VPL(26, χ)* in run 1 or *VPL(1, χ)* in run 2,
  - while B's tilt is seen to change, P's row changes but B's body row stays the same,
  - its shade follows P's, or shows a row other than *VPL(26, χ)*, once B stands on the flat
    ground beyond.

**MS-18 Complete fresh-depth initialisation (claims 24, 32, 34).** Engine conformance
for lead-confirmed departures, not an oracle measurement of their corrections.
- *Setup:* Common setup, named models on stock turretless types. Probe P has the depth
  option, disabled part caching and no shadow; isolated voxels spread over 60 × 60
  pixels, at least three pixels apart, depths verified from F5. S is a single-voxel
  non-depth model, no shadow, caching disabled. D is a single-voxel model with part
  caching enabled and fresh shadows every drawn frame. Nothing else visible has voxel art.
- *Procedure:* At speed 6 capture P alone, P with S, and P with D. Repeat with S/D's
  depth option enabled and D's shadow caching enabled; remove spoilers by map trigger.
  Start a second game in the same program session after P appeared in the first,
  and capture P's first image there. One run per variant, lossless tick-labelled captures.
- *Observable:* P's qualifying pixels compared with the P-alone rendering.
- *Expected:* All variants show the same complete P pixel set, from the first draw
  onward and in the new game. No small previous rectangle limits clearing; a shadow
  changes neither P's mode nor coverage. Zero/negative-depth markers pass empty pixels.
- *Falsified if:* unrelated earlier draws suppress a verified marker or a fresh shadow
  changes coverage. Non-depth depth preservation and the initial full clear remain
  specified but cannot be distinguished by screenshots after full fresh-depth reset.

**MS-19 Normal mode 0 (claim 35; `quirks.voxel_mode0_misread`).**
- *Test data:* one stock turretless vehicle type gets test models, each with one section of
  normal mode 0 and a single column, and each with a twin identical except for normal mode 1.
  *X*, *Y* and *Z* are three clearly different palette indices, none of them 0, 40 or in the
  house-colour range, and *Y* and *Z* are below 16, so that they are also valid normal indices of
  the mode-1 twin. *X*, *Y*, *Z* and 40 are each chosen so that the VPL maps them to a
  different index at the shade a mode-1 voxel gets there, so that a lit voxel cannot show any of
  them unchanged. (If the VPL leaves 40 unchanged there, the test uses another empty count *e*
  (not 0, not in the house-colour range) instead of 40 throughout (b), with *Nz = e + 2* and *e* below 126,
  so that *e* stays below 255 − *Nz*.)
  - The columns below are standard columns, given by their runs and voxels; their value
    sequences follow from the public model format. The spec side has checked, from that format,
    that each column ends inside its own column data in every reading named below.
  - *(a)* 1 × 1 × 2 cells: one run of 0 empty cells and two voxels, both with colour index *X*
    and normal index *Y*. Its mode-0 output has two cells and stays within its own data.
  - *(b)* 1 × 1 × 42 cells: one run of 40 empty cells and two voxels, the first with colour index
    40 and normal index *Z*, the second with colour index *X* and normal index *Y*. Read in
    mode 0, both poses show two voxels at cells 40 and 41, without out-of-column data. In a second run, model (b) gets a second HVA that turns the section
    upside down (half a turn about model x), so that its farthest corner is on its top face and
    it is drawn top-down (claim 21).
- *Observable:* the pixels of the voxels, at body facings 8 and 24, identified by grid cell on
  the golden image.
- *Expected:* every capture matches its golden image, drawn from claim 35's explicit output
  table and F5, without assuming a general malformed-stream grammar:
  - mode 0: every voxel shows a value of the column data unchanged (unlit), the same at both
    facings,
  - (a), mode 0: two voxels show, and they show two different values, *X* and *Y* (the drawn
    values are the first voxel's colour index and normal index),
  - (b), mode 0: only cells 40 and 41 show voxels (cells *e* and *e* + 1 if another empty count
    *e* is used), in both poses, and the pair of values they show
    is palette indices 40 and Z at cells 40 and 41 upright, and X and Y at cells 40 and 41
    upside down (substitute e for 40 if that variant is used),
  - mode 1, (a) and (b): only VPL-lit colours show (none of *X*, *Y*, *Z* and 40 unchanged), at the
    cells the runs give.
- *Falsified if:* with mode 0 any voxel shows a VPL-lit colour or a colour that changes with the
  facing; any capture differs from its golden image; in (a) both voxels show *X*, or in (b) the
  upright pose shows *X* at cell 41 (each voxel's colour index and normal index read as a pair,
  with the normal index dropped); in (b) any cell other than 40 and 41 shows a voxel.

**MS-20 Column ending at a pair boundary (claim 36; `quirks.voxel_topdown_column_origin`).**
- *Test data:* one stock turretless vehicle type gets a test model with one section of
  1 × 1 × 4 cells whose single column has two pairs of runs: 2 empty cells and 2 voxels of colour
  *X*, then 0 empty cells and 2 voxels of colour *Y*. The runs cover 6 cells, and the first pair
  ends exactly at *Nz* = 4.
- *Observable:* the pixels of the section, at body facing 8.
- *Expected:* the two *X* voxels at cells 2 and 3; no *Y* voxel anywhere.
- *Falsified if:* a *Y* voxel shows (above the grid's top or elsewhere), or the *X* voxels are not
  at cells 2 and 3.

**MS-21 Building barrel without a turret (claim 37; `quirks.barrel_only_no_recoil`).**
- *Test data:* one stock defence whose only voxel model is in the barrel role (S-0007 claim 20)
  gets a test barrel model, placed as the common test-data rules give for buildings, under a new
  name that keeps the model in the barrel role (S-0007 claim 20). The model has a body section and a
  small marker section, and its HVA has 3 frames in which the marker takes clearly different
  positions. The barrel's inner end lies off the model origin, so that a pitch about the origin
  and a pitch about another point give different images. In the art database copy the defence's
  image gets a turret displacement of 20. The test map override section gives it the recoil
  option, with travel 3, compress 2, hold 3 and recover 4 set through both the turret keys and
  the barrel keys.
  - **Anchor run.** As above, with the turret-animation x and y displacements also set to 10 and
    −6 in the test map override section.
  - **Reuse run.** A second test barrel model with a single-frame HVA, whose voxels all use colour
    index *χ*, normal mode 1 and normal index 85 (outside the mode-1 table, claim 20). The map
    also has the probe P of MS-17, which turns back and forth between body facings 8 and 16
    without stopping, so the shade of index 85 changes from drawn frame to drawn frame. A run
    under the cache-disable option on the defence's image is the control. Nothing else on the
    map is drawn as a voxel model, and the defence's weapon must not fire a projectile drawn as
    one (such projectiles also give shades, claim 20).
- *Procedure:* a fresh game. The defence stands idle for a few hundred ticks, then attacks a target
  that moves around it, then a stationary target that survives many shots. Per-tick captures, from
  before the first shot until well after the last.
  In the reuse run the defence fires at the stationary target only. Compare captures
  with both direction and 32-step elevation unchanged, since both enter its key.
- *Observable:* the barrel image per 32-step direction and per tick, and the marker position; in
  the reuse run, the palette index of the barrel's pixels in each drawn frame.
- *Expected:*
  - At each direction the image matches the golden image for *V · Rz(θ) · Ry(φ)* around the turret
    anchor: no shift with the turret displacement, and a pitch about the model origin. The pivot
    shows only while the barrel is pitched, so this comparison uses captures in which it visibly
    is: the 32-step pitch is read from the capture, and both pivots are rendered at that pitch.
  - Reuse run: while direction and elevation stay the same, the barrel's pixels keep
    one row, during
    recoil cycles too, although P's row changes (the image is reused). In the control the
    barrel's row follows P's row, allowing for one drawn frame of lag.
  - Before the first attack the marker shows frame 0 in every capture. It changes only while the
    defence is attacking.
  - In the ticks after each shot the image does not move: there is no recoil slide.
  - In the anchor run every image is that of the first run, moved exactly 10 pixels right and 6
    pixels up.
- *Falsified if:*
  - the image is shifted along the barrel's facing (the 2 model units of claim 9) or pitches about
    another point,
  - the image slides in the ticks after a shot,
  - the marker stays at frame 0 throughout a long attack, or changes before the first attack,
  - the anchor run moves the image by any other amount,
  - in the reuse run, the barrel's row follows P's row during a recoil cycle at an unchanged
    direction and elevation (drawn fresh, as `quirks.barrel_only_no_recoil` off would), or
    in the control it does not follow P's row.

- *Complete-key rendering variants:* In engine conformance, give this barrel-only
  model a 260-frame marker HVA. At one facing and elevation request frames 0–259,
  with caching on and off. Every frame must show its own marker, including 256–259.
  At one fixed frame request level and raised pitch under both cache settings: each
  image must show current elevation. A substituted marker or frozen pitch falsifies
  claims 26 and 37. These rendering fixtures supply pose inputs directly; they do
  not advance the oracle attack counter or modify oracle elevation.

**MS-22 Use of the lighting table file (claim 38).** The parts of the file named below (row count, remap range start and end, extra value, palette, rows) are located as S-0013 gives.
- *Test data:* the MS-4 stock vehicle on flat ground, drawn with named test copies of the lighting
  table file, each a loose file that overrides the stock one (glossary, *measurement setup*):
  - **(a) Unused parts changed.** The stock file with its remap range start, remap range end and
    extra value each set to the value 20, and its palette replaced by all zeros. The row count
    and the rows are unchanged. Since 20 is below 32, a reader that took any of those three values
    as the row count would drop rows 20–31.
  - **(b) Extra rows.** A file made from the first 32 rows of the stock file (if it has fewer,
    padded to 32 rows with copies of its last row), then 8 added rows filled with values that
    differ from every entry of those 32, and the row count set to 40.
  - **(c) Trailing data.** The stock file with 100 bytes of any value appended after its last row.
- *Procedure:* with the stock file and with each copy, show the vehicle at all 32 facings and
  capture it. Take the rows of the stock file and draw MS-4's golden images from them, using row
  *q* for shade *q*.
- *Observable:* the palette index of each pixel in each capture.
- *Expected:* the stock file's captures match MS-4's golden images drawn with row *q* of the file
  for shade *q*; the captures of (a), (b) and (c) are identical to the stock file's.
- *Falsified if:* MS-4's images match only with another choice of rows, or any of (a), (b) and (c)
  changes a pixel. Under (a), pixels of shade 20 or above turning transparent would mean that one of the three
  changed values acts as the row count, and any other change that the remap range, the extra value
  or the palette has an effect. A change under (b) falsifies the claim that rows beyond the 32nd
  are not used, and a change under (c) the claim that data after the last row is ignored and the
  length is not checked.

**MS-23 Short, missing and small tables (claims 19, 38; `quirks.vpl_unloaded_rows_blank`).** The
variants are named test files under the glossary's *measurement setup*: loose files that override
the stock one, and, in variant (d), a loose copy of the archive that holds the file, with that member left
out so that the file is absent. They are removed afterwards. The parts named are located as in
MS-22.
- *Setup:* the MS-4 stock vehicle on flat ground, with these variants of the file:
  - (a) the stock file with its row count set to 20, everything else unchanged (so the file is still as long as before),
  - (b) the stock file cut right after the last entry of its row 19 (its 20th row), row count unchanged,
  - (c) the stock file shortened so that it ends exactly where the first entry of row 0 would begin (S-0013 gives that point), so that no entry of any row remains,
  - (d) no file at all,
  - (e) the stock file cut after the entries for colour indices 0–127 of row 19, row count
    unchanged. A second test model, as in MS-15's first-section test (one section of normal mode 2,
    voxels of normal index 16, which takes shade 19), has one voxel for each colour index 1–255,
    in any layout in which no two of them share a pixel, drawn at body facing 8 on a stock
    turretless vehicle type,
  - (f) the stock file shortened to its first byte only, so that no entry of any row remains,
  - (g) the stock file with its row count set to 200, everything else unchanged. Run it only
    if the stock file holds at most 128 rows (the requested amount is then positive and the
    supplied prefix is within claim 38's safe bound),
  - (h) the file of (a), with a second test model on a second stock turretless vehicle type,
    which gets the depth option (art database copy) and the no-shadow option (test map override
    section) and keeps image reuse. It is drawn at body facing 8. The model has two sections, each
    one flat layer of voxels of one colour index, one in front of the other at that facing and
    overlapping on screen, with no voxel of depth 0 (as in MS-12). The nearer section's voxels have
    a normal index that takes a shade of 20 or above, the farther section's voxels one that takes a
    shade below 20 (both chosen with F4), and the farther section's colour index is mapped by row
    *q* of the stock file to a non-zero index at its shade *q*. No other object on the map has the
    depth option, and the game is the first since the program started, so the depth plane holds
    only zeros before this model's first fresh draw (claim 34),
  - (i) read once: the vehicle's image gets the cache-disable option in the art database copy,
    so that every image is made fresh with the table then in use (claim 26 leaves open whether a
    new game empties the image caches). The program is started with the file of (a) in place,
    and a first game shows the vehicle as variant (a) does. That game is ended, the loose file of (a) is removed, so that the stock
    file would apply to a new load, and a second game is started in the same program session and
    shows the vehicle at the same facings. If the loose file cannot be removed while the program
    runs, the run is void.
- *Procedure:* start a game and show the vehicle at several facings, whose normals take shades
  throughout 0–31. For (e) and (h), show the second test model.
- *Observable:* whether the game starts, and which of the vehicle's pixels are drawn, and in variant (e) the
  palette index of the pixel of each colour, and in variant (h) which pixels of the overlap
  show.
- *Expected:* in every variant the game starts. (a): rows 0–19 act as in the stock file, and the
  capture matches MS-4's golden image drawn with rows 20–31 all zero: a pixel whose voxel drawn
  last there has shade 20 or above is transparent, even where a voxel of lower shade drawn earlier
  lies on the same pixel (claim 19); (b) looks the same as (a); (c), (f) and (d):
  the vehicle is invisible. (e): the voxels of colours 1–127 show the stock *VPL(19, χ)* and those of
  colours 128–255 are missing. (g): every capture matches the stock file's (the rows the file does
  not hold read as zero, and shades above 31 do not occur). (i): the captures of both games match those of (a), since the file was read only at start-up.
  (h): the sections are drawn nearest
  first (claim 24), and the nearer section's voxels pass the depth test, store their depths and
  write transparent pixels, so the farther section's voxels fail the test there: the overlap is
  transparent, and the farther section shows only outside it.
- *Falsified if:* the game fails to start, the missing rows draw as raw colour indices or as some
  other colour, (a) looks like the stock file (the number of rows taken comes from the file's length
  or from somewhere other than the row count), (c) or (f) shows any pixel of the vehicle, or in (e)
  the voxels of colours 128–255 are drawn or the whole row is missing, or any pixel of (g) differs
  from the stock file's. In (a), a pixel that shows a lower-shade voxel's colour where the golden
  image is transparent means that voxels of an all-zero row write nothing instead of
  transparent; in (h), the farther section showing inside the overlap means that such voxels
  store no depth (or write nothing). In (i), second-game captures that match the stock file's
  mean that the file is read again for a new game.

**MS-24 Colour isolation, enlarged images and malformed columns (claims 22, 34, 36).**
Colour/no-wrap checks are engine conformance for lead-confirmed departures; the
260-cell column relation still tests the original's malformed-column quirk.
- *Setup:* Common setup, two stock turretless types with named models, no shadows,
  no depth option and disabled part caches. A has one 1 × 1 × 4-cell section, bounds
  1 × 1 × 1 model units, normal mode 1, two run pairs with zero empty cells and
  128 then 132 voxels. Total 260 ends at claim 36's first qualifying pair boundary.
  B has one voxel of another output colour and a copy rectangle covering A's logical
  out-of-box positions, with B's voxel elsewhere. Nothing else visible has voxel art.
- *Variants:* A upside down by a half-turn HVA about model x; A with bounds stretched
  to 1 × 1 × 4 model units; A with an empty section enlarging its copy width to
  246, 247 and 300 pixels. Also use a well-formed sparse model whose bounds and
  occupied markers extend beyond 256 image pixels on both axes, with a copy rectangle
  containing all markers and both sides below 4096 pixels (claim 22). Repeat with the depth option and with an enlarged shadow
  footprint. Isolate markers at image column 255 to probe right-pixel adjacency.
- *Procedure:* At speed 6 capture both objects over successive drawn frames, once per
  variant. Remove B and draw A alone, then start a new game in the same session with
  B alone; capture its first image. Record marker positions, colours and copy bounds
  losslessly, against F5/F6 predictions.
- *Expected:* B always shows only its own voxel, even on the first new-game image.
  A copies only its own rectangle; its column reaches cell 259 bottom-up and starts
  at cell 3 extending downward top-down (claim 36). Enlarge A's copy rectangle with
  an empty section to observe out-of-box column markers in A itself. The well-formed
  large model shows all markers at signed unwrapped positions, including negative
  image coordinates, with no artificial edge clipping. Column 255's right pixel is
  column 256 in the same row, for depth-mode parts and shadows alike.
- *Falsified if:* B contains A-coloured residue, a verified marker wraps to an opposite
  edge or clips at 256, or a right write changes rows. Contrary column cell positions
  separately falsify claim 36, retaining its warning and malformed-column quirk.

**MS-25, requested section ties (claims 23, 24, F8).** Common setup, named test VXL/HVA
files and art database copy as in MS-5b, caching disabled, shadows suppressed, speed index 2.
Test all fourteen F8 rows and an all-equal part of five sections. Keep transformed nearest
depths exactly equal within groups, adjusting bounds independently of occupied voxels.
Provide separate coincident marker pixels for each pair and for all sections together,
in distinct colours, separated by at least three pixels. In the depth variant their
comparison depths must be equal, nonzero, and unaffected by right-pixel overwrites.
Restart the program per variant to exclude retained depths. Capture each model paused,
once without the depth option and once with it. Expected pair winner in both variants:
the later section in F8's painter-priority column. Whole-part winners in table order:
B, A, C, D, B, C, A, C, B, C, A, A, A, D. In the five-section case the last section wins,
with file-order painter priority for every pair. Repeat each mixed-depth configuration
with different strictly ordered nearest-depth gaps but the same equality groups: all pair
winners remain unchanged. Also compare A = B alone with A = B > C while C has no
pixel at the A/B marker: B wins the former, A the latter. This tests the influence of the
full section sequence independently of actual C coverage. Verify equality and pixel coincidences with a golden-image
calculation from the files. Any contrary marker with verified equal depths falsifies
the corresponding priority, rather than being discarded as a setup failure.

**MS-26, general mode-0 columns (claims 35, 36).** Common setup, named test models/HVAs,
art database copy disabling both caches, shadows suppressed, no depth option, speed index 2.
A named test lighting table changes every nonzero colour used here, distinguishing raw
from lit output. Use widely separated columns and upright/upside-down HVA poses.
Values below are invented test data, not stock values. Capture once per model and pose.

| Forward column value sequence | Nz | Bottom-up output | Top-down output |
|---|---|---|---|
| 0, 2, 10, 11, 90, 1, 2, 20, 21, 91 | 5 | cells 0, 1, 3, 4 show 10, 11, 20, 21 | separate fixture below |
| 1, 91, 21, 20, 2, 0, 90, 11, 10, 2 | 5 | separate fixture above | cells 4, 3, 2, 1 show 10, 11, 20, 21 |
| 0, 2, 10, 11, 90, 0, 1, 20, 91 | 2 | only cells 0, 1 show 10, 11 | separate fixture below |
| 0, 91, 20, 1, 0, 90, 11, 10, 2 | 2 | separate fixture above | only cells 1, 0 show 10, 11 |
| 0, 3, 10, 11, 12, 90 | 2 | cells 0, 1, 2 show 10, 11, 12 before bounded-data stop | use the separate sequence 0, 90, 12, 11, 10, 3: cells 1, 0, −1 show 10, 11, 12 before bounded-data stop |
| 0, 1, 10, 90 | 3 | cell 0 shows 10 before bounded-data stop | use 0, 90, 10, 1: cell 2 shows 10 before bounded-data stop |
| 1, 92, 30, 1, 2, 91, 21, 20, 2, 1, 90, 10, 1 | 8 | top-down fixture only | cells 7, 5, 4, 1 show 10, 20, 21, 30 |

The second row is Q-0216's two-span discriminator: 20 and 21 at cells 1 and 0
would falsify the corrected relation. The last row has three spans and nonzero empty
counts, distinguishing the uniform rule from a special first-span rule: any displacement
of 10 from cell 7 or of 20 and 21 from cells 5 and 4 falsifies it. Capture MS-19 (b)
upside down in the same program-build test series: X at cell 40 and Y at cell 41
confirms its unchanged table row, whereas X at cell 0 and Y at cell 1 falsifies it.

For the three-colour span in the two-cell grid and the one-colour span in the three-cell
grid, compare only the within-column prefix: the oracle's continuation outside the
supplied data is unspecified. Engine-conformance replays must add no further
voxels. Also supply a 260-colour column in a four-cell grid, as spans of 255 and 5 colours,
zero empty counts. Bottom-up includes all 260 cells; a separately encoded top-down equivalent
starts at cell 3 and extends downward. Scale its height so the pixels fit the copy rectangle.
The boundary is 260 ≡ 4 modulo 256, not a stop inside the first span. Exchange two companion
columns with different sequences: neither may change the probe's output. Add a later
coincident colour of palette index 0: it must erase the earlier pixel. Record marker
positions/colours after F5/F6 projection. Lit colours, a cutoff inside a span, different
height anchors, or companion-dependent prefix pixels falsify the rule. Repeat the two-span
and cutoff fixtures in depth-buffered mode after program restart: the same raw colours and
traversal distances must meet claim 24's depth and double-width rules. To discriminate the
boundary modulus, append another complete span after the 260-cell prefix in the four-cell
grid, with a visible colour at a non-overlapping pixel outside the bounding box but inside
the copy margin. Outside depth mode it contributes nothing; in depth mode it contributes
if its depth test passes. The oracle's eventual beyond-data continuation is not an expected
result. A mode-0 depth fixture of height 255 with an empty count below 255 must prepare
normally, unlike the unspecified non-depth case. Grid-cell clipping with the
malformed-column switch off and bounded reading are tested in engine-conformance
replays, not selectable oracle modes; neither clips at an image-buffer edge.

Golden-image comparisons of vehicle shadows in the earlier scenarios include claim 39's
own-image exclusions; an unmasked footprint calculation alone predicts only its potential
shadow pixels, not every pixel that will be darkened.

**MS-27, vehicle own-shadow exclusion (claim 39, Q-0213).** Common setup, speed index 2,
one bystander computer player kept outside the observed area. Supply a named test body with
two HVA frames and a shadow section whose footprint overlaps bright nonzero body pixels.
A named art database copy disables both caches. Compare with shadows suppressed by a
named test-map override. Test isolated driving vehicles on flat ground and on a ramp,
and a hovering vehicle resting and rocking. Capture one paused image per pose and control.
Calculate qualifying footprint/body overlaps from the test data and S-0102's scene-depth
rule. Expected: each own-body overlap pixel equals its no-shadow control, while qualifying
footprint pixels outside the own image are darkened. Darkened own pixels in fresh-shadow
runs, or a reversal for hovering or ramps, falsify claim 39.

For reuse, enable shadow caching only. Frame 0 has a hole at a shadow-marked pixel,
frame 1 fills it. Capture 0 first, then move to select 1 without changing facing or
ramp key; verify with an asymmetric marker. This comparison is engine conformance
for the lead-confirmed departure, not an oracle check. The filled own pixel stays
undarkened, matching the fresh-shadow control. Reverse frame order: the hole must
let the qualifying footprint darken the scene, with no old-pose gap. A second object
sharing the footprint must exclude its own current image, not inherit the first
object's cutouts. Repeat with hovering resting and rocking poses: fresh and reused
footprints have the same current exclusions. Wrong frame or changed key makes reuse
inconclusive; darkened own pixels or persistent old cutouts otherwise falsify the fix.
Rocking freshness itself is no longer distinguishable by this corrected-output test.

**MS-28, larger mixed-depth ties (claim 23, Q-0215 question 7).** This is an
investigation of an `unmeasured` general relation, not a claim that a finite table
settles every section count.
- *Setup:* Common setup, speed index 2, one human-owned parked stock turretless vehicle
  and one distant bystander computer player. Named test VXL/HVA files and an art database
  copy give the vehicle five or six sections, disable part caching and suppress shadows.
  All other data is stock; director and other mods are off. Use the pairwise and common
  marker construction of MS-25, with different nontransparent colours for every section.
  Bounds set exact finite nearest-depth equality groups independently of the markers.
  In depth mode all left marker pixels have the same nonzero comparison depth, and
  right-hand writes cannot reach another probe.
- *Procedure and observable:* Restart the program for each model and mode. Capture one
  paused lossless image in normal mode and one with the depth option. Record every pair
  winner, not just the common-pixel winner. Repeat each model with unequal depth gaps
  changed while preserving the relations. Repeat with sections outside one tested pair
  retaining their bounds but contributing no pixel at that pair's probe.
- *Proposed discriminator results:* Names are in file order; each priority is lowest to
  highest. These are finite hypotheses for this investigation, not additions to F8's
  binding coverage. At equal comparison depth, both modes should give each pair's pixel
  to the section later in the listed priority, subject to the depth controls above.

  | File order | Nearest-depth relation | Proposed painter priority | Proposed common-pixel winner, both modes |
  |---|---|---|---|
  | A, B, C, D, E | A = C > B = E > D | D, B, E, A, C | C |
  | A, B, C, D, E | B = D > A = E > C | C, A, E, D, B | B |
  | A, B, C, D, E, F | A = E > C = F > B = D | B, D, C, F, E, A | A |
  | A, B, C, D, E, F | B = E > A = C > D = F | D, F, A, C, E, B | B |

- *Expected and falsifying results:* The gap-change and absent-at-probe repeats should
  preserve every pair winner. A verified contrary pair winner falsifies that row's
  hypothesis; changed winners under either repeat falsify F8's input-sufficiency claim.
  A stable file-order tie alternative disagrees in the second row's B/D pair and the
  third row's A/E pair, so common-pixel captures alone must not replace pair probes.
- *Closing question 7:* A complete observable priority relation over the full file
  sequence and exact finite depth relations must be established separately and predict
  F8, these probes and larger parts. Test its predictions on every equality/order pattern
  for five sections, then on six-section patterns containing three equality groups and
  on larger parts with intervening sections absent at probes. Preserve all captures as
  discriminators for that relation. Agreement with finite captures supports it but does
  not alone prove an arbitrary-count rule. No such general relation is asserted in this
  revision, and no untested stable priority is substituted for the original default.

**MS-29 Bounded part and shadow images (claim 22, F6; Q-0470).** Engine
conformance for lead-confirmed departures, not an oracle experiment.
- *Setup:* Use an engine rendering fixture with named sparse VXL/HVA models on a
  stock turreted vehicle type, neutral lighting and fixed flat-ground poses. Render
  part images and unmasked shadows separately, then together. Use the common skirmish
  houses, speed 6, director off and all other mods off; a direct rendering fixture
  supplies the poses without advancing simulation. Place enough isolated, distinct
  nontransparent markers to identify each retained edge. Choose bounds and poses whose
  F6 copy widths and heights independently equal 4095, 4096, 4097, 4098 and one million
  pixels. Include odd and even sides, a long narrow rectangle, negative logical pixel
  coordinates and a displaced HVA section. These are invented test dimensions.
- *Procedure:* Import each model, request every test pose twice with caching enabled,
  and repeat with caching disabled. Compare retained pixel positions and colours
  losslessly with F5/F6 or F10 predictions, and record image extents and importer
  warnings. One replay per variant is sufficient. For the limit test use an output
  surface large enough to distinguish window clipping from ordinary screen clipping.
  For depth mode, include a voxel with its left pixel immediately outside the retained
  left edge and its right pixel inside, plus an overlapping in-window voxel: the
  expected winner is calculated by claim 24 with both voxels contributing. Repeat
  the left-edge adjacency probe for a shadow column.
- *Non-finite variants:* In separate fixtures put positive infinity, negative infinity
  and a value that is not a number in an applicable section bound or HVA transform.
  Also use finite inputs whose projection under F0 becomes non-finite. Leave another
  part finite and visibly distinct. Supply an invalid shadow placement separately
  from valid part placements. Repeat after a valid cached appearance.
- *Observable:* retained dimensions, edge-marker positions and colours, absence of
  invalid images, unaffected valid parts, and the count and model identity of warnings.
- *Expected:* both sides at most 4096 pixels are unchanged; exceeding sides retain
  exactly F6's central window, with the extra discarded pixel on the right or bottom
  for odd differences. Million-pixel rectangles retain at most 4096 × 4096 pixels,
  without wrapping, rescaling or moving retained markers. The off-window adjacency
  probes retain their right pixels with the predicted coverage. Non-finite placement
  produces no pixels of the affected part or shadow, including after earlier valid
  reuse; other finite parts still render. Importing a model with any oversized part
  or shadow pose warns once, not once per pose, appearance or image role. Models
  with all copy sides at most 4096 pixels give no size-limit warning. Also replay
  representative stock models at all body facings: clipping changes none of them.
- *Falsified if:* any retained side exceeds 4096 pixels, a verified retained marker
  moves or changes colour, odd centring selects the opposite extra pixel, a qualifying
  right-hand contribution is lost because its left pixel was outside the window,
  an invalid image draws pixels, another finite part disappears, or an oversized
  model gives zero or multiple size-limit warnings. An ordinary screen clip or an
  incorrectly constructed copy size makes that variant inconclusive rather than a pass.

## Open questions

- **Turretless vehicle with a barrel part.** A vehicle with a barrel part but no turret is drawn
  differently, with a shift derived from its weapon position and a pitch correction. Not yet
  understood; left out.
- **Shadow vertical placement.** Where an airborne unit's shadow footprint lands vertically (its
  ground point) is decided by the locomotor and is left to S-0102 and S-0106; the one-pixel
  adjustment for airborne units seen in the original is unconfirmed.
- **Own-image exclusion outside vehicles.** Claim 39 fixes vehicles, including hovering
  vehicles, only. Whether other voxel object kinds also exclude their own images is not
  determined here; S-0102 owns the aircraft observable.
- **Shadow on a cache miss.** When a shadow image is not yet cached, the shadow sometimes appears
  to be skipped for that draw instead of made. In which situations, and whether a shadow can be
  missing for one drawn frame, has not been determined.
- **Turret frame of multi-turret types.** Which HVA frame the turret models of a multi-turret
  type are drawn in (claims 12, 16) has not been determined.
- **Animation-frame counter.** Exactly how attacking advances the counter that picks turret
  frames (claim 16) belongs to the combat, vehicle and building specs; MS-3c tests only that it
  starts at 0 and changes only while attacking.
- **Cache keys of other locomotors.** For the flying, rocket and tunnelling locomotors the body
  key is made up differently, and how its parts combine with the turret key, including the
  removal of the body facing at turret displacement 0, has not been determined. Whether
  locomotors other than the driving and naval locomotors also skip the cache while the body's
  tilt is not settled is settled for the hovering, jumpjet and tunnelling locomotors by S-0106
  claim 13; for the flying and rocket locomotors it has not been determined.
- **Image invalidation.** What observable event makes all earlier images ineligible for reuse
  in the original, and whether a new game does so, remain open (claim 26). Our engine retains
  reuse throughout a game and starts each game without earlier images. Invalidation bounds how
  long the frame and elevation reuse artefacts persist in the original.
- **Other section ties (Q-0215 question 7).** F8 answers Q-0210's configurations,
  Q-0215 questions 1–6 and all-equal parts of any size. The full file-order sequence and
  exact nearest-depth relations suffice to determine the original's result, but a complete
  behavioural characterization for arbitrary mixed-depth ties remains open and is
  explicitly `unmeasured` (claim 23): finite examples cannot identify a unique general
  relation. MS-28 gives outside-table pairwise discriminators and the settlement criterion.
  The current table does not license extrapolation or a stable tie rule. Q-0215 is not
  listed as fully answered in the front matter because this part remains open.

## Questions for the project lead

- **Image invalidation.** Claim 26 keeps reuse for a whole game in our engine, while the
  original can invalidate earlier images together at an event not specified here. Is that
  departure, with no switch, acceptable?
Image enlargement and turret-origin pitching are settled as unconditional presentation
fixes, with no extra pivot import. Claim 22 settles Q-0470's bounded-image policy and
requires one importer warning per oversized model.
