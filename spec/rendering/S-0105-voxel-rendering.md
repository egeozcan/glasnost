---
id: S-0105
title: Rendering of voxel models
topic: rendering
revision: 5
sources-class: [decomp, format-doc]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: [Q-0014]
supersedes: []
---

# S-0105: Rendering of voxel models

## Summary

How a voxel model (a VXL model with its HVA animation) becomes pixels: the fixed viewing
transform, how many facings each model part can show, how the body, turret and barrel are
placed (a building's barrel without a turret included), how the voxel's normal and the global light pick a shade from the voxel lighting table
(VPL), how each voxel becomes a pixel, how the parts are composited, how building turrets
recoil, how the flat voxel shadow is made, and what of earlier draws the voxel buffer keeps. Ramp tilt is left to S-0106; where the finished image
lands among other objects is in S-0102; palettes and house colours are in S-0103; the map's
lighting tint is in S-0104.

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
- **The voxel buffer.** A 256 × 256 scratch image of palette indices (0 = transparent) into which
  one part is rasterised before it is copied to the screen.
- **Shade index.** A lighting level 0–31 chosen per voxel normal; it selects one 256-entry row
  of the VPL.
- **VPL.** The voxel lighting table file: for each shade index and each voxel colour index, the
  palette index to draw. Claim 38 gives how its content is used. A *row* is the 256 entries of one shade index.
- **Table trigonometry.** Sines and cosines quantised to 1/8192 of a turn and taken from fixed
  single-precision values instead of exact functions; formula F1. It matters because every
  rotation in this spec uses it.
- **Pixels per lepton** *ρ*: the conversion from game distance to model units (see Constants and F3).
- **Image cache.** A per-type store of finished part images, keyed by a combination of the draw
  inputs (claim 26). It is shared by every object of that type.
- **Fresh draw.** A draw that makes a part image or a shadow image from the model, as opposed to
  one that reuses an image from the image cache (claims 26, 31). Only fresh draws give shades to
  normal indices (claim 20), switch the depth-buffered mode (claim 32) and use and clear the
  voxel buffer (claim 34).
- **Depth plane.** The per-pixel depths (integers 0 … 255) kept beside the voxel buffer and used
  only in the depth-buffered mode (claim 24).
- **Clear area.** The part of the voxel buffer and depth plane that a fresh draw clears before it
  starts: the previous fresh draw's copy rectangle, one pixel wider on the right and one pixel
  taller at the bottom (claim 34).
- **Recoil cycle, recoil slide.** A building turret's or barrel's slide back and return after a
  shot (claim 27); its recoil slide *λ* is how far back the part currently sits, in model units
  (F7).
- **Column.** The cells of a section that share grid indices *(i, j)*: one vertical line of the
  grid.
- **Column data.** The column's *value sequence*: the integers 0 … 255 that the public model
  format stores for one column, taken in that format's own order, between the start and the end
  the file gives for that column. In a standard file the sequence holds the column's runs, and each
  run contributes its two counts and, for each of its voxels, that voxel's colour index and normal
  index, in the format's order (the order that claim 35's *Standard files* entry relies on). A
  column the file marks as empty has no column data.
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
- **Type id.** The identifier by which S-0007 claim 16 singles out one vehicle type (claim 12,
  MS-11). Its meaning, and which type it singles out, are S-0007's.
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
- **Alternative-model state.** On while claim 12 has a vehicle draw something other than its main
  body in the body's place (its alternative body or, for a type with the turret flag, its turret
  model); off otherwise. It is part of the image-cache keys (claim 26).
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
   (11.25°) above level from the second tick on (`quirks.barrel_elevation_quantised`). With *A = 20* the turn is 5120
   facing units and takes 6 ticks at 853 facing units per tick; the 32-step values 0, 1, …, 6
   ticks after placement are 8, 8, 7, 7, 6, 6, 6 (F9). What a barrel image shows is the elevation at the time the image was made; a
   cached barrel image keeps it (claim 26, `quirks.barrel_cache_ignores_elevation`), so a barrel first drawn in the first tick
   can stay level on screen after the elevation has risen. Buildings re-aim the elevation
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

10. **Barrel placement.** With *t* the translation column of the turret transform *T* (before the
    view), the barrel transform is *B = T · Tr(−t) · Ry(φ) · Tr(t)*, where *φ* is the elevation
    angle of claim 6 and *Tr(v)* is a translation in the current local frame. The only points
    the elevation leaves in place are those of the lateral (y) line through the turret-frame point *−t*, so the barrel pitches about a
    lateral axis through *−t* in turret coordinates, not through the turret origin (`quirks.barrel_pivot_frame_mixup`).
    *t* is the turret's translation in its parent frame: for a vehicle on flat ground it is
    *δ × ρ* along the body's forward direction (plus any translation in *D*, S-0106),
    for a building it is *d_b* along the turret's own forward direction. When *t = 0* the pivot
    is the turret origin. The barrel is drawn with *V · B*.
    - Backing: `measurable` (MS-2 for vehicles, MS-6 for buildings)
    - Confidence: high (composition), medium (visible effect)

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
      the no-spawn-alternate suffix or, for the one type id that S-0007 names, the body with the
      suffix S-0007 gives for that type id. When both files exist, the no-spawn-alternate one is
      used.
    - **No option, no alternative body.** Without the alternative-body option no alternative body
      is ever drawn. A vehicle with the type id S-0007 names but without the option always shows
      its main body, even when its alternative body has loaded. So for a type without the option,
      S-0007's `quirks.apc_water_body` changes only what is loaded, not what is drawn.
    - **Option without a file.** A turretless vehicle with the option for which no alternative
      body has loaded draws no body at all while none of its spawned aircraft is docked
      (`quirks.alt_body_turret_model`). Its shadow still shows, because the shadow always comes
      from the main body (claim 29).

    A type that also has the turret flag has no alternative body file (S-0007); while none of its spawned aircraft is docked, the
    original draws the type's turret model in place of the body, posed as the body, and the
    turret as usual (`quirks.alt_body_turret_model`). A vehicle of a multi-turret type draws
    the turret model and barrel model of its selected turret index instead of the type's ordinary
    turret and barrel. A type with rotating-barrel weapon stages always draws its ordinary turret
    and barrel, whatever its turret count. File naming is S-0007.
    - Backing: `measurable` (MS-11)
    - Confidence: medium (high for the turret model in place of the body and for an alternative
      body never drawn without the option)

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

18. **Shade per normal.** For each draw of a part, the shade index of each normal index is given
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
    table file (claim 38 gives how its content is used). The file's remap range and extra value are not consulted
    when drawing: a pixel needs only the row and entry.
    - Backing: `measurable` (MS-4)
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
    - Backing: `measurable` (MS-16)
    - Confidence: high (shade 16, the stale value), medium (shade 0 at program start)

### Rasterisation

21. **Back-to-front inside a section.** Within a section, a voxel drawn later overwrites an
    earlier one in the voxel buffer, and the drawing order runs from the section's farthest
    bounding-box corner toward the nearest. The farthest corner is the one of the eight box
    corners with the smallest view-space *z*; on an exact tie the first in this list is used: on
    the bottom face (max x, max y), (max x, min y), (min x, min y), (min x, max y), then the same
    four on the top face. Count each voxel's grid position from that corner, *n_x, n_y, n_z* steps
    along the grid's x, y and z directions. The drawing order is ascending lexicographic order of
    *(n_y, n_x, n_z)*. So a section is drawn as rows of columns, a row's columns in grid-x
    order, and each column upward when the farthest corner is on the bottom face and downward
    when it is on the top face. Outside the depth-buffered mode (claim 24), where several voxels
    of one section land on the same pixel, the one with the greatest *(n_y, n_x, n_z)* shows.
    In a well-formed file the runs of every column add up to exactly *Nz* cells, including the
    empty cells above the column's top voxel. Columns that break this are covered by claim 36
    (`quirks.voxel_topdown_column_origin`).
    - Backing: `measurable` (MS-5)
    - Confidence: high

22. **One pixel per voxel.** Each voxel writes exactly one pixel of the voxel buffer, at the
    position given by formula F5. There is no splatting; a section magnified past one model unit
    per voxel shows gaps (`quirks.voxel_single_pixel`). Positions wrap modulo 256 in both directions, so a part wider
    or taller than the buffer wraps around (`quirks.voxel_buffer_wrap`; what is left of it in later draws: claim 34).
    - Backing: `measurable` (MS-5, MS-13) for one pixel per voxel and the gaps; `unmeasured` for
      the wrap. A part that wraps is wider or taller than 248 pixels, so its copy rectangle
      (claim 25) is larger than the buffer. How the original copies such a rectangle has not
      been determined, so the screen result cannot be predicted yet (open questions). Until it
      is, our engine copies only the part of such a rectangle that lies inside the voxel buffer.
    - Confidence: high

23. **Section order.** The sections of one part are drawn in ascending order of their nearest
    view-space depth (the largest *z* of their transformed bounding box), so the section whose
    nearest point is farthest is drawn first. Sections with exactly equal nearest depth do not
    keep file order (`quirks.voxel_section_sort_ties`). Formula F8 defines the exact order, ties included. For example,
    file order A, B, C with depths 2, 2, 1 draws C, B, A. Without the per-type depth option there
    is no depth test between sections: later sections simply overwrite.
    - Backing: `measurable` (MS-5)
    - Confidence: high

24. **Per-type depth buffering.** A type with the depth-buffer option draws differently in four
    ways. See claim 32 for a case where this mode is applied to types that do not have the option.
    - **Section order.** The sections are drawn in exactly the reverse of the claim 23 order
      (ties included), so the section with the nearest point is drawn first. Within a section the
      voxel order of claim 21 is unchanged.
    - **Depth test.** Every voxel-buffer pixel holds an integer depth 0 … 255 in a depth plane.
      The depth plane is not reset as a whole before each draw: at the start of a draw in this
      mode it is set to 0 over the clear area of claim 34 only, and elsewhere it keeps the depths
      that earlier draws in this mode left there (`quirks.voxel_depth_plane_partial_clear`). A voxel's depth is *⌊pz / 256⌋ mod 256*, with *pz* its z position from
      formula F5; this is about the integer part of *z + 128 − c_z*, wrapped modulo 256. A voxel
      is drawn only if its depth is strictly greater than the depth stored at its pixel. So a
      voxel whose depth is 0 is never drawn (`quirks.voxel_depth_zero_hidden`), a voxel whose pixel still holds a stale
      depth at least as great as its own is not drawn either (claim 34), and of two voxels with equal depth on one
      pixel the one drawn first stays.
    - **Two pixels per voxel** (`quirks.voxel_depth_double_width`). A voxel that passes the test writes its colour and its
      depth into its own pixel and into the pixel immediately to its right. Only the left pixel's
      stored depth is tested: the right pixel is overwritten even if it held a greater depth. At
      buffer column 255 the right-hand pixel is column 0 of the next row down (and outside the
      buffer for the last row, where the result is unspecified; our engine drops that pixel).
    - **No carry error.** The 1/256-pixel carry of `quirks.voxel_x_carry_into_y` does not occur.
    - Backing: `measurable` (MS-5, MS-12; the depth-plane clearing: claim 34, MS-18)
    - Confidence: high

25. **Copy to the screen.** After all sections are drawn, the bounding box of the transformed
    section boxes (in view-space pixels, *y* flipped) is enlarged by a 4-pixel margin on every
    side and copied to the screen around the object's draw point (formula F6). The draw point
    comes from the object position (S-0101) plus any shift its locomotor applies. Palette index 0 in
    the voxel buffer is transparent.
    - Backing: `measurable` (MS-1)
    - Confidence: high

26. **Image reuse.** Finished part images are kept in the image cache of the object's type and
    reused, by every object of that type, whenever a later draw produces the same key. The key is
    meant to capture every input of the image, but it does not:
    - **Body:** the ramp state (which settled tilt the body has, for locomotors that tilt the
      body on ramps, S-0106), the 32-step body facing, the alternative-model state, and the HVA
      frame modulo 32 (`quirks.voxel_cache_frame_mod32`).
    - **Vehicle turret:** the ramp state, the 32-step body facing, the 32-step turret facing, the
      selected turret index for a multi-turret type, and a frame value *v*. With *f* the turret
      frame, *v = f mod 256* while the alternative-model state is off and
      *v = 2⌊(f mod 256) / 2⌋ + 1* while it is on (`quirks.voxel_cache_alt_turret_frame`). So while the turret model stands in for the body (claim 12), an even
      turret frame and the odd frame after it share a key, and both share it with that odd frame
      drawn while the state is off. When the turret displacement is 0, the body facing is left out of the key. A turret
      on the body's origin then has the same orientation for every body facing with the same
      turret facing. On flat ground, and on a settled ramp, the reused image therefore differs
      from a fresh one by at most the 0.088° rounding of F1, which rarely moves a pixel. (On a ramp
      the tilt is part of the key. On a settled ramp the body transform is the cell's tilt
      followed by the body's 32-step rotation, so the tilt does not depend on the body facing.
      The tilt itself is left to S-0106.)
    - **Building turret:** the 32-step turret facing and the turret frame modulo 256.
    - **Barrel:** the turret's key (for a building's barrel drawn without a turret, claim 37
      gives the key). Barrel images are kept apart from turret images, so a
      barrel draw reuses only a barrel image. Barrel elevation is **not** part of the key, so
      once a barrel image is cached for a key, later draws with that key show the cached
      elevation (`quirks.barrel_cache_ignores_elevation`).

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

    **What reuse changes on screen.** Because only fresh draws have side effects, every key
    component shows: a draw with a key not yet in the cache is fresh, and a draw with a known key
    is not. The visible consequences are:
    - the collisions: frames *f* and *f* + 32 of a body (`quirks.voxel_cache_frame_mod32`), turret frames *f* and *f* + 256,
      the turret frames that share a key under `quirks.voxel_cache_alt_turret_frame`, and barrel elevation (`quirks.barrel_cache_ignores_elevation`),
    - a turret at displacement 0 reused across body facings, which can differ from a fresh one
      by the rounding above,
    - the side effects of fresh draws: the shades given to normal indices, which other models'
      out-of-table indices then show (claim 20), the switching of the depth-buffered mode
      (claim 32), and the clear area the next fresh draw starts from (claim 34),
    - the depth-buffered look: the key never records whether the depth-buffered mode was on, so
      an image made during a claim 32 window keeps that look when reused,
    - the absence of all of these under the cache-disable option.
    - Backing: `measurable`. MS-17 covers the body facing, the ramp state, the unsettled tilt
      of the driving locomotor and the body facing in the turret key, at both zero and non-zero
      displacement. MS-2 covers
      the turret facing. MS-3b covers the body frame modulo 32, the turret frame modulo 256 and
      the cache-disable option. MS-11 covers the alternative-model state, the selected turret
      index and the collision of `quirks.voxel_cache_alt_turret_frame`. MS-10 covers the barrel, MS-6 the recoil case and MS-9 the depth-buffered
      look. `unmeasured` in two places. The building turret's frame modulo 256: a building's
      animation-frame counter advances only through attacking, by rules other specs own, so no
      scenario here can bring a building turret past frame 255 on demand. The naval locomotor
      case: when a ship's body tilts is S-0106's (claims 6 and 7), but stock data may give a
      ship no tilt at all (S-0106, Open questions), and none of this spec's scenarios uses
      S-0106's naval test data (its scenario T6), so none of them makes it happen.
    - Confidence: high (the frame keys, the collision of `quirks.voxel_cache_alt_turret_frame`, the barrel key without elevation, the body facing
      in the turret key), medium (the ramp state and the unsettled-tilt case, whose details
      are left to S-0106)

### Recoil

27. **Building turret recoil.** When a building with the recoil option fires, its turret and its
    barrel each run a recoil cycle (formula F7, state table S1): they slide back along the
    turret's forward axis by the imported travel distance (model units, i.e. pixels; a negative
    travel slides them forward, F7), hold, then return. The turret's recoil moves the turret image only, not the barrel (`quirks.turret_recoil_leaves_barrel`). The
    barrel's recoil moves the whole barrel, pitched as claim 10 gives, back along the turret's
    forward axis: the direction of the slide does not tilt with the barrel's elevation.
    - **Timing.** A cycle started from rest lasts *max(1, t_c) + t_h′ + max(1, t_r)* ticks, where
      *t_h′ = t_h* if *t_h ≥ 2* and *t_h′ = 0* if *t_h ≤ 1*: a hold of 1 holds for no tick at all
      (`quirks.recoil_hold_one_skipped`). With the absent-key defaults (travel 2, compress 1, hold 1, recover 1) the
      recoil slide is 2 model units in the first tick of the cycle and 0 from the second on, so
      the recoiled pose shows in the drawn frames of exactly one tick.
    - **A shot during a cycle.** Every shot starts both cycles anew (a cycle whose travel is 0
      stays at rest). A cycle that is still running when the shot comes starts its new compress
      phase from its current recoil slide *λ_r* instead of from 0. The recoil slide then rises by the full compress step for
      *max(1, t_c)* ticks, to about *λ_r + ℓ*, holds, and recovers by the ordinary recover step.
      In the last recover tick it jumps to 0 from about *λ_r + ℓ / max(1, t_r)*. So the slide is
      not bounded by the travel distance: each shot during a cycle adds about *ℓ* more, and a
      building that keeps firing faster than its cycle ends slides farther with every shot
      (`quirks.recoil_refire_accumulates`).
    - **Barrel values across rules layers** (`quirks.barrel_recoil_layer_reset`). The barrel's travel, compress, hold and
      recover values each default to the turret's corresponding value, and this default is
      applied again in every rules layer that has a section for the type. After such a layer,
      each barrel value is the layer's own barrel key if it has one, and otherwise the turret's
      value as it stands after that layer (the layer's turret key, or the earlier value if the
      layer has none). A rules layer without a section for the type changes neither. Two
      consequences: a later rules layer that sets only the turret's travel makes the barrel's travel
      follow it, and a later rules layer that has a section for the type but no barrel key, even one
      that names no recoil key at all, replaces barrel values that an earlier rules layer set with the
      turret's values. This is an exception to S-0006's rule that an absent key keeps the earlier
      value (S-0006, claim 3), like the resets on override of S-0006 claim 5, except that the value
      goes back to the turret's value rather than to an engine default.
    - Backing: `measurable` (MS-6, including its hold, negative-travel, refire and layer runs)
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
    footprint is not mirrored. These positions have no 1/256-pixel carry error. Formula F10 gives the exact
    positions, the rounding and the copy to the screen. All marked pixels take one value,
    which the shadow treatment then darkens. The shadow transform is
    *V · S · H(0, shadow section)*, where *S* is the body's shadow transform: the ramp tilt of the
    cell under the unit (S-0106) and the body's 32-step rotation. Height and pitch of the body do not change the footprint's outline.
    - Backing: `measurable` (MS-8)
    - Confidence: medium (high for the two pixels per column and for present columns without
      voxels)

31. **Shadow shift and scale.** The footprint is shifted 3 pixels to the right on screen (exactly
    *−6 × L_x*, stored truncated as 2.9999964, before the footprint's positions are truncated,
    F10). For types flagged as aircraft-like, the footprint is scaled by 1/2 about the shadow's
    draw point, and the 3-pixel shift is not halved (F10). The shadow is drawn with the ordinary shadow
    treatment (S-0102, S-0104). It is skipped whenever the object's cloak state is not
    *uncloaked* (that is, while cloaking, cloaked or uncloaking), and when the type has the
    no-shadow option. Shadow images are cached per type like part images (key: ramp, 32-step body
    facing), unless the type has the shadow-cache-disable option. That key covers every input of
    the footprint (frame 0, the type's shadow section and scale are fixed per type), so a reused
    footprint looks the same as a fresh one. Whether a shadow draw is fresh still matters in two
    ways: only a fresh shadow of a depth-option type opens a claim 32 window, and on some cache
    misses the shadow may be skipped for that draw (open questions). Until those cases are known,
    our engine makes the shadow on every cache miss.
    - Backing: `measurable` (MS-8) for the shift, scale, cloak and no-shadow conditions, and
      MS-9 for the claim 32 side effect; `unmeasured` for the look of a reused footprint (its key
      covers every input of the footprint, so a reused footprint and a fresh one have the same
      pixels, and only the fresh draw's side effect is observable). The skip on a cache miss is
      not a claim yet (open questions).
    - Confidence: medium (shift, scale, cache), high (cloak and no-shadow conditions)

### Depth-mode leak

32. **Depth mode left on after a depth-option shadow.** A fresh draw of the shadow of a type with
    the depth-buffer option switches the depth-buffered mode (claim 24) on and leaves it on.
    Every later fresh draw of a voxel part then uses the depth-buffered mode, whatever its type:
    sections nearest first, depth test, two-pixel-wide voxels. This lasts until a part of a
    depth-option type (its body, turret or barrel) is next drawn fresh, which switches the mode
    off once that part is finished. Reused images switch nothing. Images made during this window
    are cached and keep the look (`quirks.depth_mode_leak`). The mode carries over from one drawn frame to the
    next. For example, take a parked depth-option type whose part images are all reused, but whose
    shadow is drawn fresh in every drawn frame because it has the shadow-cache-disable option.
    From its second drawn frame on, the mode is on for every other fresh draw, whatever the order
    in which objects are drawn.
    - Backing: `measurable` (MS-9)
    - Confidence: high

33. **The leak in stock play.** With stock data a window opens only when a depth-option type's
    shadow is drawn fresh: in every drawn frame for such a type that has the shadow-cache-disable
    option, and otherwise when it shows a ramp state or body facing not yet cached (a shadow
    skipped on a cache miss opens none, open questions). A window
    closes at the next fresh draw of any part of a depth-option type. Which other fresh draws
    fall inside a window therefore depends on the order in which objects and shadows are drawn
    within a drawn frame (S-0102), which has not been determined. So the leak can occur in a
    stock game, but how often it shows is not predicted here.
    - Backing: `unmeasured`. How often it happens depends on a cross-object draw order that no
      spec fixes yet; the mechanism itself is measured by MS-9 (claim 32).
    - Confidence: medium

### Buffer clearing and unusual model data

34. **What is cleared between draws.** The voxel buffer and its depth plane are reused from one
    fresh draw to the next, and across drawn frames and games. The first fresh draw after the
    program starts clears both completely; after that, the depth plane is never cleared as a
    whole again, and the colour plane only in the case given below. Every fresh draw (of a voxel part, of a
    shadow, and of the projectiles and debris drawn as voxel models) starts by clearing the
    *clear area*: the copy rectangle of the previous fresh draw (F6), widened by one pixel
    column on its right and one pixel row at its bottom. In voxel-buffer pixels, with that
    rectangle's corner *(x₀, y₀)* and size *w × h*, the clear area is columns *x₀ … x₀ + w* and
    rows *y₀ … y₀ + h*, both ends included.
    - **Colour.** The clear area is set to transparent (palette index 0). If it does not fit in
      the buffer (*x₀ < 0*, *y₀ < 0*, *x₀ + w > 255* or *y₀ + h > 255*), the whole buffer is set
      to transparent instead.
    - **Depth.** The depth plane is set to 0 over the clear area only, and only if the
      draw is made with the depth-buffered mode on. Every fresh draw of a part or shadow of a
      depth-option type is made with it on (S2). A draw made with the mode
      off leaves the depth plane as it is. When the clear area does not fit in the buffer, what
      this clear does to the depth plane is unspecified (it can reach pixels outside the clear
      area); our engine clears the part of the clear area that lies inside the buffer.
    - **Consequence (`quirks.voxel_depth_plane_partial_clear`).** When a fresh draw made with the mode off comes between two
      fresh draws made with it on, the later one keeps the earlier one's depths outside the
      clear area of the draw in between, and its voxels there are drawn only where their depth is
      greater than that stale depth. A parked depth-option object redrawn in every drawn frame
      after a smaller non-depth draw thus loses every voxel outside the smaller draw's clear
      area whose pixel still holds the same depth from its own previous draw. Colour residue (`quirks.voxel_buffer_wrap`) needs a pixel written outside the previous copy
      rectangle, which only a part that wraps (claim 22) produces.
    - Backing: `measurable` (MS-18) for the depth plane; `unmeasured` for the colour clear,
      whose effect shows only through the wrap of claim 22 and so waits on the same open
      question about oversized copy rectangles.
    - Confidence: medium (depth plane), high (colour clear area and its sides)

35. **Normal mode 0.** A section whose normal mode is 0 has no normal table and is drawn unlit:
    each voxel's value goes into the voxel buffer as it is, without the VPL (a value of 0 is
    written as transparent). Its column data is read as if each voxel held one value, a colour
    index, and no normal index (`quirks.voxel_mode0_misread`). The importer warns about every
    section of normal mode 0 (Import mapping, *Importer warnings*). Every column's reading starts afresh at its own
    column data (at its first value bottom-up, at its last value top-down), however the column
    before it was read.
    - **Bottom-up** (claim 21), a column's values are taken, from its first, as consecutive
      groups: an empty count *e*, a voxel count *n*, then *n* values drawn as *n* voxels upward
      from the cell after the *e* empty cells, then one value that is not used.
    - **Top-down**, the values are taken from the column's last value backward, as groups of a
      voxel count *n*, then *n* values drawn as *n* voxels downward from the top of the grid
      (positioned as the shift of `quirks.voxel_topdown_column_origin` gives), then one value that is not used, then an empty count *e*.
    - **Where a column ends.** As for the other modes (`quirks.voxel_topdown_column_origin`): at the first group boundary, the
      column's start included, at which the cells counted so far (empty counts plus voxel counts)
      are congruent to *Nz* modulo 256.
    - **Standard files.** In a standard file each voxel holds two values (a colour index and a
      normal index) and each run carries its voxel count a second time, so the groups go out of
      step after the first run of the reading direction. Bottom-up, the voxels of the first run
      show that run's voxel values in the value sequence's order, so that colour indices and
      normal indices alternate, beginning with the first voxel's colour index, and later empty and
      voxel counts are taken from whichever values come next: colour indices, normal indices or
      the file's own counts. Top-down, the first voxel shows the last voxel's normal index, the
      second the last voxel's colour index, and so on. What is drawn is thus fixed by the column data alone, as
      long as the column ends within its own column data.
    - **Reading past the column data.** If a column reaches the end of its own column data (or,
      top-down, its start) before it ends, the original reads on into the data that lies beyond,
      and what it draws from there is unspecified (as are the stray voxels of `quirks.voxel_topdown_column_origin`). Our engine, with the
      switch on, stops the column at the end of its own column data and draws nothing more for it.
      The importer warns about such a column.
    - **Positions.** A voxel is placed as F5 gives, with an empty count *e* adding *e·Δ_z*, when
      *e < 255 − Nz*. After an empty count of *255 − Nz* or more the original's position is
      unspecified (it depends on earlier draws), and for *Nz = 255* the original's preparation of a
      mode-0 section fails in an unspecified way. Our engine uses F5's *e·Δ_z* in every case, and
      draws a mode-0 section with *Nz = 255* like any other. The importer warns about a
      section with either case, since both come from the model file alone.
    - Backing: `measurable` (MS-19) for the unlit values, the bottom-up and top-down groups, the
      end of a column and positions within the bound (MS-19 (b) has an empty count of 40, below
      255 − 42); `unmeasured` for positions at or beyond the bound: there the original's position
      depends on what earlier draws left behind, which a running game does not let a test
      control, so there is no expected value to compare with.
    - Confidence: high (unlit, one value per voxel, the bottom-up groups and the end of a
      column), medium (top-down groups, the bound on empty counts)

36. **Malformed columns.** For columns whose runs do not add up to exactly *Nz* cells, the entry for
    `quirks.voxel_topdown_column_origin` gives what the original does: the position shift of a column drawn top-down, the stop at a
    pair boundary, and the stray voxels read from a neighbouring column's data.
    - **Importer warning.** The importer warns about every section with a column whose runs do not
      add up to exactly *Nz* cells, counting all its empty and voxel counts (Import mapping,
      *Importer warnings*), whatever the state of the switch. This is the condition of the
      heading. It covers the unspecified case below (a column that never reaches *Nz* at a pair
      boundary cannot add up to *Nz*) and also a column such as MS-20's, which stops cleanly at
      a pair boundary but has runs left over.
    - Backing: `measurable` (MS-20) for the stop at a pair boundary; `unmeasured` for the
      top-down shift and the stray voxels. A column that does not reach *Nz* at a pair boundary
      always goes on into a neighbouring column's data, which this spec leaves unspecified, so
      no test model can show the shift on its own. What our engine draws instead (it stops at the
      end of the column's own runs) is given in the entry for `quirks.voxel_topdown_column_origin`.
    - Confidence: medium

### Buildings with a barrel model only

37. **A barrel without a turret.** A building that has a voxel barrel model but no voxel turret
    model draws the barrel alone. S-0007 claim 20 gives when that happens; stock defences whose only
    model is in the barrel role are such buildings. The barrel is drawn as follows.
    - **Pose.** Its draw transform is *V · Rz(θ) · Ry(φ)*, where *θ* is the building turret angle
      of claim 5 and *φ* is the elevation angle of claim 6. The turret displacement plays no part:
      there is no shift by *d_b* (claim 9), and the barrel pitches about its own model origin, as
      claim 10 gives for *t* = 0.
    - **Place.** The image is centred at the turret anchor (Terms).
    - **Frame.** It is drawn in HVA frame *u mod F_bar*, where *u* is the building's
      animation-frame counter (claim 16) and *F_bar* is the barrel HVA's frame count. It is not
      held at frame 0 as barrels drawn with a turret are.
    - **No recoil.** It is never drawn displaced by recoil, whatever the building's recoil option
      and values. A running recoil cycle does not stop its image being reused either
      (`quirks.barrel_only_no_recoil`).
    - **Image reuse.** It is kept with the type's barrel images. Its key is the 32-step facing of
      claim 5 together with its frame modulo 256. The elevation is not part of the key
      (`quirks.barrel_cache_ignores_elevation`), and the cache-disable option applies as in
      claim 26.
    - Backing: `measurable` (MS-21)
    - Confidence: high (pose, frame, no recoil, key), medium (visible placement)

### The voxel lighting table file

38. **Use of the lighting table file.** The voxel lighting table file is in the public format of
    that table. Its content is a row count *R*, a remap range, one more value, a palette and *R*
    rows of 256 palette indices each, one entry for each voxel colour index. This spec fixes only
    how the content is used; where each part lies in the file, and in what order, is the format's
    own, as it is for VXL and HVA files. The file is loaded once, at start-up (claim 19).
    - **What counts.** Row *q*, numbered from 0 in file order, supplies the entries for shade
      *q*: the entry for shade *q* and colour *χ* is entry *χ* of row *q* (claim 19). Only the row
      count *R* and the rows have an effect.
    - **What does nothing.** The remap range, the extra value and the palette have no effect: not
      on drawing, not on house colouring (S-0103 fixes the remap range at 16–31 itself) and not on
      anything else. S-0103 gives the palettes drawing does use.
    - **Rows in use.** Shade indices are 0–31 in practice (F4), so rows beyond the 32nd are not
      used.
    - **Missing and short data.** Rows the file does not supply read as all zero, which is the
      transparent index, so voxels drawn with such a row leave nothing in the voxel buffer. That
      covers *R* below 32, a missing file, and a file that ends before its last row. When the
      file ends part-way through a row, the entries it does supply from that row are used and the
      rest of the row reads as zero. The game goes on starting up as if the load had succeeded
      (`quirks.vpl_unloaded_rows_blank`). Bytes after the last row are ignored.
    - **Departure: more than 128 rows.** With *R* above 128 the original's result is unspecified:
      the load can disturb unrelated game state. This is bad data, so it is a departure, with no
      switch. Our engine uses the first 128 rows, ignores the rest and carries on as for any
      other file, and the importer warns (Import mapping, *Importer warnings*).
    - Backing: `measurable` (MS-22 for the use of the rows and the unused parts; MS-23 for short,
      missing and small tables)
    - Confidence: high (rows, unused parts), medium (a file that ends part-way through a row, a
      missing file)

## Formulas

**F0. Floating-point environment.** All floating-point work in this spec runs in the original
with 53-bit (double) intermediate precision and **round toward zero** for every operation and
every store. This holds for everything this spec computes, the HVA translation scaling (claim 14)
and every draw included; the light direction (claim 17) comes out the same under either rounding
(F2). Matrices, vectors and the shade sums are stored in single
precision between steps; each such store truncates toward zero. Float-to-integer conversions are
truncations toward zero. Confidence medium: truncation is certain at every float-to-integer
conversion; that it also holds for every store between conversions is not confirmed. This is the
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

For non-negative angles this is "round to the nearest 1/8192 turn, halves up", except that an
angle that would round up to a whole number of turns gives 8191 steps (*σ* never steps on from
8191 to 0). For negative
angles it is not symmetric (`quirks.table_trig_negative_bias`). Write the exact angle as −(m + f) table steps
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

The result is never larger than the exact root, and its relative error is below 2⁻¹³.

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

**F5. Pixel position of a voxel.** Let *c* be the centre of the part's overall view-space bounding box
(*y* flipped). For a section with view-space transform *Mᵢ*, let *P₀* be the farthest box corner
transformed and *P_x, P_y, P_z* the corners adjacent to it along the grid's x, y, z directions. In
units of 1/256 pixel, truncated toward zero:

> *o = trunc(256 × (P₀ + 128 − c))* (all three components)
> *Δ_ξ = trunc(256 × (P_ξ − P₀) / N_ξ)* for *ξ* in x, y, z (the z components of *o* and of
> the *Δ_ξ* matter only in the depth-buffered mode).

A voxel *n_x, n_y, n_z* steps away from the farthest corner (counting along each axis from that
corner) lands at *p = o + n_x Δ_x + n_y Δ_y + n_z Δ_z*, each component taken modulo 65536, and
is written to voxel-buffer pixel *(⌊px / 256⌋ mod 256, ⌊py / 256⌋ mod 256)*. In the
depth-buffered mode the z component is formed the same way, and a voxel's depth (claim 24) is
*⌊pz / 256⌋ mod 256*.

Without depth buffering, *py* also receives carries from *px* (`quirks.voxel_x_carry_into_y`). Write ⟨v⟩ for
*v mod 65536* (in 0 … 65535), and ⟨w⟩ₓ for ⟨x component of the vector *w*⟩. For a voxel, let
*v* be the number of voxels drawn before it in its column, and *e₁, …, e_r* the lengths of the
runs of empty cells before it in its column. The rows of columns before the voxel's row number
*n_y*, and the columns before the voxel's column in its row number *n_x*, empty columns included.
Then

> *U = ⟨o⟩ₓ + n_y ⟨Δ_y⟩ₓ + n_x ⟨Δ_x⟩ₓ + ⟨e₁ Δ_z⟩ₓ + … + ⟨e_r Δ_z⟩ₓ + v ⟨Δ_z⟩ₓ*,

where each empty run's term reduces the whole product *e_m Δ_z* (not *e_m ⟨Δ_z⟩ₓ*), and *py* is
the value above plus *⌊U / 65536⌋*, modulo 65536. Equivalently, with the position seen as *o*
plus these terms (one per row before, one per column before, one per empty run before, one per
voxel before), *py* gains 1/256 pixel for each term whose x component, taken in 0 … 65535,
carries the running *px*, also taken in 0 … 65535, to 65536 or beyond; the count is the same
whatever order the terms are added in. With a negative x step that happens for nearly every term (all
but those that take *px* below 0); with a positive x step it happens only when *px* passes 65536
(256 pixels). The depth-buffered mode and the shadow footprint have no such carry.

**F6. Screen placement.** A model point at view-space *(x, y)* lands on the screen at the draw
point *+ trunc(c) + ⌊128 + (x, y) − c⌋ − 128*, i.e. within one pixel of *⌊(x, y)⌋*. The area that
reaches the screen, the part's *copy rectangle*, is a *w × h* rectangle. With the overall bounding box
*[x_min, x_max] × [y_min, y_max]*, *w₀ = trunc(x_max − x_min)*, *h₀ = trunc(y_max − y_min)*,
*w = w₀ + 8* and *h = h₀ + 8*. The rectangle's corner is voxel-buffer pixel
*(128 − ⌊w₀/2⌋ − 4, 128 − ⌊h₀/2⌋ − 4)*, and it lands at the draw point plus
*(trunc(c_x) − ⌊w/2⌋, trunc(c_y) − ⌊h/2⌋)*.

**F7. Recoil cycle.** The travel distance *ℓ* (model units), and the compress, hold and recover
durations *t_c, t_h, t_r* (ticks), are imported signed integers. A travel of 0 starts no cycle; a
negative travel makes the part slide forward instead of back. Write *t_c′ = max(1, t_c)*,
*t_r′ = max(1, t_r)*, and *t_h′ = t_h* if *t_h ≥ 2*, *t_h′ = 0* if *t_h ≤ 1* (`quirks.recoil_hold_one_skipped`). The
steps are

> compress step *s_c = ℓ / t_c′*, hold step 0, recover step *s_r = −(ℓ / t_r′)*.

The division is a **real (floating-point) division**, not an integer one: with *ℓ = 3* and
*t_c = 2* the step is 1.5. It is formed at 53-bit precision and stored in single precision,
truncated toward zero (F0). The recoil slide *λ* is a single-precision value, 0 at rest. In each
tick of a cycle *λ* becomes *λ + step*, stored truncated toward zero, with the step of the phase
that tick belongs to: *t_c′* compress ticks, then *t_h′* hold ticks, then *t_r′* recover ticks. In
the last recover tick *λ* is set to exactly 0 instead, and the cycle ends. So *λ* can end a
compress phase slightly short of *ℓ* (for example *ℓ = 1, t_c = 3* gives steps of
0.333333313…). The first tick of a cycle is the tick of the shot or the one after it (which of the
two is not fixed here; MS-6 does not depend on it).

A shot while a cycle runs (claim 27) starts a new cycle with *λ* left as it is: *t_c′* compress
ticks, *t_h′* hold ticks and *t_r′* recover ticks follow, with the same steps, and the last recover
tick again sets *λ* to 0. Examples, as the recoil slides of successive ticks:
- travel 3, compress 2, hold 3, recover 4, one shot: 1.5, 3, 3, 3, 3, 2.25, 1.5, 0.75, 0.
- the same with hold 1: 1.5, 3, 2.25, 1.5, 0.75, 0; with hold 2: 1.5, 3, 3, 3, 2.25, 1.5,
  0.75, 0.
- the defaults (travel 2, compress 1, hold 1, recover 1): 2, 0.
- travel 3, compress 2, hold 3, recover 4, a second shot while *λ* = 3 (during hold): after the
  last 3 shown before the rise, 4.5, 6, 6, 6, 6, 5.25, 4.5, 3.75, 0.
- travel 8, compress 8, hold 2, recover 2, a second shot 3 ticks after the first (during
  compress): 1, 2, 3, …, 11 (one more per tick, for 3 + 8 ticks), 11, 11, 7, 0.
- In general, a shot that lands in a compress phase with step *s_c* leaves the rise going at
  *s_c* per tick and makes it last *t_c′* ticks from that shot's first cycle tick, so with the first and
  last shot *n_s* ticks apart, each shot within *t_c′* − 1 ticks of the one before, the peak is
  about *(n_s + t_c′) × s_c*. A shot that lands in a hold
  at *λ* = *λ_r* raises the hold level to about *λ_r + ℓ*.

The part is drawn shifted by *−λ* model units along the turret's forward axis; the shift is not
rounded to whole pixels before the transform (the pixel position then follows F5).

**F8. Section order (claim 23).** Number the sections of a part 1 … *N* in file order, and let
*z(s)* be the nearest view-space depth of section *s* (the largest *z* of its transformed bounding
box). Define orders *π₀, π₁, …, π_N*, where *π_ϑ(i)* is the section at position *i*. *π₀* is file
order, and the draw order is *π_N*. For *ϑ* = 1 … *N*, write *π′* for *π_(ϑ−1)*. Then *π_ϑ* is
obtained from *π′* as follows:

> *i₀ = ϑ*, and *i₁ < i₂ < … < i_J* are the positions *i > ϑ* at which *z(π′(i))* is strictly
> smaller than the minimum of *z(π′(i′))* over *ϑ ≤ i′ < i* (the strict new minima after *ϑ*),
> *π_ϑ(ϑ) = π′(i_J)*, and *π_ϑ(i_m) = π′(i_(m−1))* for *m* = 1 … *J*,
> *π_ϑ(i) = π′(i)* at every other position.

So at each position the chain of strict new minima moves one place: its last member comes to *ϑ*,
and each other member takes the place of the next. When *J* = 0, *π_ϑ = π′*. *π_N* is in
ascending order of *z*. Sections with equal *z* do not necessarily keep file order (the example
in claim 23). Only the resulting order *π_N* is binding: the intermediate orders only define it, and
any way of reaching the same order meets this spec.

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
reduced modulo 65536 on its own (no carry between them, unlike F5). If the file lists the column
as present (claim 30), voxel-buffer pixel *(⌊px / 256⌋ mod 256, ⌊py / 256⌋ mod 256)* and the pixel
immediately to its right are marked, with the same rule at buffer column 255 as claim 24 (the
right-hand pixel is column 0 of the next row; past the last row the result is unspecified and our
engine drops that pixel). Every marked pixel takes one value, which the shadow treatment
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
| The voxel buffer | 256 × 256 | pixels | Wraps |
| Buffer centre | 128, 128 | pixels | F5, F6 |
| Rows kept from a lighting table file | 128 | rows | Our engine's limit; a departure for files with more rows (claim 38) |
| Copy margin | 4 | pixels | Each side |
| Clear-area extension | 1 | pixel | On the right and at the bottom of the previous copy rectangle (claim 34) |
| Sub-pixel precision | 1/256 | pixel | F5 |
| Depth levels | 256 | levels (0 … 255) | Claim 24 |
| Shadow shift | +3 (2.9999964) | pixels, screen x | Claim 31 |
| Aircraft-like shadow scale | 1/2 | dimensionless (scale factor) | Claim 31 |
| Normal tables | 16 / 36 / 64 / 245 | entries | Modes 1 / 2 / 3 / 4; see below |
| Sine table | 8192 per turn | entries | F1 |
| Turret recoil defaults | travel 2, compress 1, hold 1, recover 1 | model units, ticks | Absent-key defaults; each absent barrel value defaults to the type's turret value, again in every rules layer that has the type's section (claim 27, `quirks.barrel_recoil_layer_reset`) |
| Shortest hold that holds | 2 | ticks | A hold of 1 or less gives no hold tick (F7, `quirks.recoil_hold_one_skipped`) |
| Barrel elevation turn rate | 3 | turn rate *ω* (Terms; nominally 256 facing units per tick per unit of *ω*) | Claim 6; a turn of *Δ* facing units takes ⌊\|Δ\| / 768⌋ ticks (F9) |
| Negative-angle table bias | 2 | table steps (1/8192 turn) | F1, `quirks.table_trig_negative_bias` |
| Level-body tolerance | 0.005 | radians | Claim 26: pitch or roll of this size or more counts as an unsettled tilt; compared in floating point |

### Normal direction tables

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
| compressing | each of its *t_c′* ticks | λ grows by the compress step *s_c* (F7) |
| compressing, *t_h* ≥ 2 | its last tick ends | holding, from the next tick |
| compressing, *t_h* ≤ 1 | its last tick ends | recovering, from the next tick: no hold tick at all (`quirks.recoil_hold_one_skipped`). With the defaults (2, 1, 1, 1) λ is 2 in the first tick and 0 in the second |
| holding | each of its *t_h* ticks | λ unchanged |
| holding | its last tick ends | recovering, from the next tick |
| recovering | each of its *t_r′* ticks but the last | λ grows by the recover step *s_r*, which has the opposite sign to ℓ (F7) |
| recovering | its last tick | λ becomes exactly 0; idle |
| compressing, holding or recovering | the building fires again | compressing, with λ kept as it is: a full compress phase of *t_c′* ticks adds about ℓ to the current λ, and the last recover tick later drops λ to 0 from about *λ + ℓ / t_r′* (`quirks.recoil_refire_accumulates`) |

Turret and barrel each have their own cycle with their own imported values; both start on the
same shot.

**S2. Depth-buffered mode (claims 24, 32).** One global on/off state, off at program start. It
decides how every fresh draw of a voxel part is made, whatever the part's type.

| State | Event | Result |
|---|---|---|
| off | a fresh draw of a part (body, turret or barrel) of a depth-option type starts | on for that draw; off again once it is finished |
| off | a fresh draw of a depth-option type's shadow | on, and stays on after the shadow is finished |
| on | a fresh draw of a part of a depth-option type is finished | off |
| on | any other fresh draw, or any reused image | stays on |
| either | a new drawn frame starts | unchanged |

**Ordering consequences.**
- Within one part: sections in claim 23 order; within a section, voxels back to front (claim 21).
  A nearer voxel of an earlier-drawn section can be overwritten by a farther voxel of a later
  section, because there is no depth test between sections (`quirks.voxel_no_intersection_depth`).
- Between parts of one vehicle: body, then turret and barrel in claim 11 order, so the turret and
  barrel cover the body where they overlap it. Parts never depth-test against each other.
- The shadow is drawn separately on the ground render layer (S-0102).

## Edge cases and quirks

Every switch defaults to the original (YR) behaviour described; "Fixed" gives the alternative.

- **`quirks.vpl_unloaded_rows_blank`** (default: on). Rows that the lighting table file does not
  supply (claim 38) read as zero and draw nothing, and a missing or truncated file does not stop
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
- **`quirks.voxel_buffer_wrap`** (default: on). Parts wider or taller than the 256-pixel buffer wrap around
  it (claim 22); from about 248 pixels the copy rectangle, with its margins, is already larger
  than the buffer. Between draws only the clear area is cleared: the previous copy rectangle
  plus one column on its right and one row at its bottom, or the whole buffer when that area
  does not fit in it (claim 34). So wrapped pixels left outside the clear area can show up in a
  later part's image. Fixed: clip or enlarge, and clear everything that was written.
- **`quirks.barrel_elevation_quantised`** (default: on). The barrel's elevation shows only in 32-step
  (11.25°) increments, and the fire angle is taken modulo 256. Fixed: continuous pitch.
- **`quirks.building_turret_shift_eighths`** (default: on). Building turrets convert the turret displacement
  with "÷ 8, truncated" instead of the vehicle factor *ρ* (≈ ÷ 6.03), and move after rotating.
  Fixed: *d_b = δ × ρ*, rounded to single precision as for vehicles (F3). The order is kept: the
  turret still moves along its own forward axis, because a building has no separate body facing
  to move along. No switch is proposed for a vehicle-style order.
- **`quirks.barrel_pivot_frame_mixup`** (default: on). The barrel pitches about the turret-frame point *−t*
  (claim 10), where *t* is the turret's translation in its parent frame. In world terms the pivot
  sits at distance *|t|* from the turret origin, in a direction that turns with twice the body
  angle plus the turret's relative angle (vehicles) or with twice the turret angle (buildings).
  Fixed: pivot at the turret origin, or at an imported pivot.
- **`quirks.voxel_cache_frame_mod32`** (default: on). Reused body images are keyed by the frame number
  modulo 32 (turret images modulo 256), so a model with more HVA frames can show frame *f* where
  *f ± 32* (or *± 256*) was asked for. Fixed: key by the full frame.
- **`quirks.turret_recoil_leaves_barrel`** (default: on). Building turret recoil does not carry the barrel
  with it. Fixed: barrel follows turret recoil.
- **`quirks.vehicle_no_recoil`** (default: on). Vehicle recoil settings are read but have no visual effect.
  Fixed: apply F7 to vehicle turret and barrel.
- **`quirks.shadow_frame_zero`** (default: on). Shadows always use HVA frame 0. Fixed: current body frame.
- **`quirks.no_turret_shadow`** (default: on). No shadow from turret or barrel; the per-unit turret-shadow
  setting is read and ignored. Fixed: honour it.
- **`quirks.table_trig_negative_bias`** (default: on). Negative angles whose fractional table step is at
  least one half land two 1/8192-turn steps short of nearest rounding (F1). All negative facing
  rotations here are such angles: a body whose 32-step facing is 16 (south) rotates by −89.912°
  instead of −90°, and one at 12 (south-east) by −44.912° instead of −45°.
  A turret rotated clockwise relative to its body is likewise 0.088° short, and so is a raised
  barrel. Fixed: exact rounding.
- **`quirks.voxel_x_carry_into_y`** (default: on). In the normal (non-depth-buffered) rasteriser, every
  position addition that takes the running x position (in 1/256 pixel, 0 … 65535) to 65536 or
  beyond adds 1/256 pixel downward (F5). With
  negative x steps that is nearly every addition, so over a large section some voxels move down
  by one pixel. Fixed: independent components.
- **`quirks.voxel_mode0_misread`** (default: on). Claim 35. A section with normal mode 0 is drawn unlit,
  and its column data is read as one value per voxel. A standard file, where every voxel holds a
  colour index and a normal index, is then misread: normal indices show as colours, and after the
  first run of the reading direction later counts are taken from colour and normal indices, so the
  section draws as garbage. With the switch on (default), our engine reproduces claim 35 exactly
  while a column stays within its own column data. Where the original would read past it (claim
  35, unspecified), our engine stops that column at the end of its own column data, and the
  importer warns (Import mapping). Fixed: read
  each voxel's colour index and normal index, and draw the colour unlit.
- **`quirks.voxel_no_intersection_depth`** (default: on). No depth test between sections unless the
  per-type depth option is set; interpenetrating sections can show the farther one in front.
  Fixed: depth-test between sections.
- **`quirks.voxel_depth_double_width`** (default: on). With the depth option every voxel is two pixels wide.
  Fixed: one pixel per voxel, as without the option.
- **`quirks.barrel_cache_ignores_elevation`** (default: on). Barrel images are cached under the turret's
  key, which does not include the barrel elevation. A cached barrel image therefore keeps the
  elevation it had when first drawn, for every object of the type, until the cache entry goes.
  This applies to vehicle barrels (which change elevation after placement, claim 6) and to
  building barrels outside recoil. Fixed: include the 32-step elevation in the barrel key.
- **`quirks.barrel_frame_zero`** (default: on). Barrels drawn together with a turret always use HVA
  frame 0 (claim 16; a building barrel without a turret is not affected, claim 37). Fixed: use the
  turret's frame.
- **`quirks.depth_mode_leak`** (default: on). Claim 32: after the shadow of a depth-option type, the
  depth-buffered mode stays on for other types' fresh draws until a part of a depth-option type
  is next drawn fresh. Fixed: switch the mode off after every shadow, or never switch it on for
  shadows.
- **`quirks.voxel_topdown_column_origin`** (default: on). Claim 36. Malformed columns, whose runs do not add
  up to exactly *Nz* cells (the condition of the importer's warning, claim 36). Three things
  happen in the original:
  - *Shift.* When the section is drawn top-down (claim 21), each column is positioned from the
    top of the grid. A column whose runs cover *Nz − γ* cells is drawn *γ* cells too high (for a
    negative *γ*, a column whose runs are too long, *|γ|* cells too low). Drawn bottom-up, the
    same column is placed correctly.
  - *Where a column ends.* A column's runs come in pairs, an empty run followed by a voxel run.
    The column ends at the first pair boundary, its start included, at which the number of
    cells covered so far is congruent to *Nz* modulo 256. So a column that reaches *Nz* exactly
    at a pair boundary stops there, even if its own runs go on, and the rest of its runs are
    not drawn.
  - *Stray voxels.* A column that never hits *Nz* exactly at a pair boundary (a short one, or a
    long one that passes *Nz* inside a run) does not end with its own runs. It continues with
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
  the exchange rule of claim 23 (F8) rather than kept in file order. Fixed: stable order.
- **`quirks.voxel_cache_alt_turret_frame`** (default: on). In a vehicle turret's cache key, the
  alternative-model state and whether the turret frame is odd are not kept apart
  (claim 26): with the state on, an even turret frame and the odd frame after it share an image,
  and both share it with that odd frame drawn with the state off. Only a type with both the turret flag and the
  alternative-body option can show it (claim 12). Fixed: keep the state and the frame apart in
  the key.
- **`quirks.voxel_depth_zero_hidden`** (default: on). In the depth-buffered mode the depth plane is set to
  0 (over the clear area, claim 34) and the test is strict, so a voxel whose depth is 0 is never drawn; depths also wrap modulo
  256, so a part deeper than 256 pixels compares wrapped depths (claim 24). Fixed: start the
  plane below every possible depth and compare unwrapped depths.
- **`quirks.shadow_double_width`** (default: on). Every column of the shadow footprint marks two pixels side
  by side (claim 30, F10). Fixed: one pixel per column.

- **`quirks.voxel_depth_plane_partial_clear`** (default: on). The depth plane is cleared only over the clear
  area, and only by draws made in the depth-buffered mode (claim 34). After a draw made with the
  mode off, the next depth-buffered draw meets the depths of the one before outside the smaller
  clear area, and loses the voxels there that are not nearer than them. Fixed: clear the depth
  plane over the whole area the coming draw can write, before every depth-buffered draw.
- **`quirks.alt_body_turret_model`** (default: on). Two cases of the alternative-body option
  without an alternative body to draw (claim 12), both while none of the vehicle's spawned
  aircraft is docked. A type with both the turret flag and the option has no alternative body
  file, and draws its turret model in the body's place, posed as the body. A turretless type with
  the option whose alternative body did not load draws no body at all. Fixed: draw the main body
  whenever there is no alternative body to draw.
- **`quirks.recoil_hold_one_skipped`** (default: on). A recoil hold duration of 1 gives no hold tick, the
  same as 0, while a hold of 2 or more holds for that many ticks; no setting holds for exactly
  one tick (claim 27, F7). Fixed: hold for *t_h* ticks for every *t_h* ≥ 1.
- **`quirks.recoil_refire_accumulates`** (default: on). A shot during a running recoil cycle adds a full
  travel distance to the current recoil slide, so repeated shots slide the part farther and
  farther, and the cycle ends with a jump to 0 (claim 27, S1). Fixed: a shot during a cycle
  compresses from the current recoil slide up to the travel distance in *t_c′* ticks, and the
  recoil slide never exceeds the travel distance.
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
| `UseBuffer` | Art [type] | boolean | no | Per-type depth buffering (claims 24, 32) |
| `ShadowIndex` | Art [image] | integer | 0 | Shadow section (claim 29) |
| `NoShadow` | Rules [type] | boolean | no | Suppresses the shadow (claim 31) |
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
| A voxel model section of normal mode 0 | the model file and the section | 35 |
| A mode-0 section in which a column's reading reaches past its own column data, in which an empty count is *255 − Nz* or more, or whose *Nz* is 255 | the model file, the section and, for the first two, the column | 35 |
| A section with a column whose runs do not add up to exactly *Nz* cells (the total of all its empty and voxel counts, whatever the stop rule of claim 36 does with it) | the model file, the section and the column | 36 |
| A lighting table file that ends before the last of the rows its own row count gives | the file | 38 |
| A lighting table file with *R* below 32 or above 128 | the file | 38 |
| A lighting table file with bytes after its last row | the file | 38 |

The importer finds the model cases by reading each column as claims 35 and 36 describe. Three
cases give no warning, because they depend on the pose or size of an object at the moment of
drawing and not on the file alone, so the importer cannot tell in general whether they will
occur: the right-hand pixel written below the last buffer row (claim 24 and the shadow marking of claim 30, F10), a copy
rectangle larger than the voxel buffer (claim 22) and a clear area that does not fit in it
(claim 34). Our engine's result in each is the one the claim gives. This follows the glossary's
*unspecified* entry, which asks for an importer warning only where the cause is in the data.

The voxel lighting table is the archive file `voxels.vpl`, looked up like other game files
(S-0005). The VXL, HVA and lighting table byte layouts are public; this spec only fixes their
interpretation (claims 19 and 38).

## Measurement scenarios

**Common setup.** Every scenario uses the glossary's measurement setup: an unmodified install with
no loose override files and no engine add-ons, and stock data. On top of that: 800×600, a tiny
flat clear purpose-made map with neutral map lighting, the game paused for captures, and
screenshots saved losslessly. Record the display path used (glossary, *display path* and
*measurement setup*).

- **Golden image** means a pixel comparison between the original's screenshot and our engine's
  render of the same scene, built from this spec.
- **Runs.** Rendering is deterministic, so one run of each scenario is enough. A golden-image
  mismatch is repeated once, to rule out a capture error, before it counts.
- **Per-tick captures** mean one capture per tick, by stepping the game one tick at a time at
  the slowest game speed.
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
    show it one step above level (earlier images stay cached, `quirks.barrel_cache_ignores_elevation`),
  - a zero golden-image diff, with the pivot at *−t* (claim 10) and the barrel in HVA frame 0.
- *Falsified if:* the barrel pivots about the turret origin, the overlap rule differs, or a
  direction first drawn from the second tick on shows the barrel level.

**MS-3 HVA, scale, HVA frames (claims 13–16).**
- *Setup:* a stock unit with a multi-frame body HVA (for example a walking voxel unit or an
  animated body), filmed while it moves.
- *Observable:* the sequence of body crops, per drawn frame.
- *Expected:*
  - body frames repeat with period *F_body* in walk-counter steps,
  - they match our renders, which use translations scaled by the first section's factor,
  - any barrel part stays in HVA frame 0.
- *Falsified if:* section parts drift from their predicted spots, or a barrel animates.

**MS-3b Frame collisions and the turret frame (claims 16, 26; `quirks.voxel_cache_frame_mod32`).**
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
  - With caching, once frames 0–7 have been drawn at that facing, body frames 32–39 show the
    marker positions of frames 0–7.
  - With the cache-disable option, frames 32–39 show their own positions.
  - Whenever the body frame *f_b* is non-zero, the turret shows frame *f_b mod 3*.
- *Falsified if:* frames 32–39 show their own positions while caching is on, or show frames 0–7
  under the cache-disable option, or the turret frame does not follow the body frame while
  *f_b ≠ 0*.
- *Second model set (turret frame modulo 256).* The same type gets a body HVA and a turret HVA of
  260 frames each, and the turret, like the body, has a small marker section that takes a
  different, clearly visible position in each frame. The same procedure, for more than 260
  walk-counter steps, with and without the cache-disable option.
  - *Expected:* with caching, turret frames 32–39 show their own marker positions (the turret key
    keeps turret frames up to 255 apart, unlike the body key), and turret frames 256–259 show the
    positions of turret frames 0–3. Under the cache-disable option every turret frame shows its
    own position.
  - *Falsified if:* with caching, turret frames 256–259 show their own positions or frames 32–39
    show the positions of frames 0–7; or, under the cache-disable option, any turret frame shows
    another frame's position.

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
  - an exact golden-image match, including the 1-pixel artefacts of `quirks.voxel_x_carry_into_y` and the tie order of `quirks.voxel_section_sort_ties`
    where sections tie,
  - the depth-option type shows 2-pixel-wide voxels.
- *Falsified if:* there is a systematic one-pixel shift (carry rule wrong), or the overdraw
  order differs between tying sections.

**MS-6 Building voxel turret (claims 5, 6, 9, 10, 11, 26, 27; `quirks.recoil_hold_one_skipped`, `quirks.recoil_refire_accumulates`, `quirks.barrel_recoil_layer_reset`).**
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
  - **Barrel.** It pitches about the turret-frame point *−t* (golden image), and its pitch
    takes only 32-step values.
  - **Recoil.** After the shot, the turret's recoil slide over successive ticks is 1.5, 3, 3, 3,
    3, 2.25, 1.5, 0.75, 0 model units (F7, S1). The sequence starts on the shot tick or the next
    one (which of the two is not fixed here). The barrel follows its own cycle (here the same
    values), and turret recoil leaves the barrel in place.
  - **Hold runs.** Hold 1: 1.5, 3, 2.25, 1.5, 0.75, 0. Hold 2: 1.5, 3, 3, 3, 2.25, 1.5, 0.75, 0.
    Defaults: 2 in one tick, then 0, so the recoiled pose shows in exactly one tick (F7, `quirks.recoil_hold_one_skipped`).
  - **Negative travel.** The same magnitudes as the base sequence (1.5, 3, 3, 3, 3, 2.25, 1.5,
    0.75, 0), with turret and barrel sliding forward along the turret's forward axis instead of
    back.
  - **Hold refire.** A staircase: in the 2 ticks after each shot's cycle starts, the
    recoil slide rises by 1.5 per tick, so each shot raises the hold level by 3 (3, 6, 9 after
    three shots); it then holds. After the last shot: 2 ticks of rise to the top level *λ_top*,
    12 ticks at *λ_top*, then *λ_top* − 0.75, *λ_top* − 1.5, *λ_top* − 2.25, and then 0 in the
    next tick (a jump of *λ_top* − 2.25).
  - **Compress refire.** The recoil slide rises by 1 per tick without a break from the first
    shot until 8 ticks after the last shot's cycle starts, so its peak is *n_s* + 8, where *n_s* is
    the number of ticks between the first and the last shot. It then holds for 2 ticks, shows
    *n_s* + 4 for one tick and then 0. With two shots 3 ticks apart: 1, 2, …, 11, 11, 11, 7, 0 (F7).
  - **Layer runs.** *Control:* the turret does not recoil, and the barrel's per-tick recoil slides
    are 1.5, 3, 3, 3, 3, 2.25, 1.5, 0.75, 0. *Turret only:* turret and barrel both slide 3, 6, 6,
    6, 6, 4.5, 3, 1.5, 0 (the barrel takes the map's turret values). *Section only:* neither the
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
  - with hold 1 or the defaults, the full travel shows in two or more successive ticks, or never,
  - with negative travel, the parts slide back, or do not move,
  - in a refire run, the recoil slide never exceeds the travel distance (3 or 8), or restarts
    from 0 at a later shot, or ends by stepping down smoothly to 0 instead of jumping to 0 from
    *λ_top* − 2.25 (hold refire) or *n_s* + 4 (compress refire),
  - the barrel moves with the turret's recoil,
  - in the control layer run, the barrel does not recoil or the turret does (the base rules copy
    was not read); in the turret-only run, the barrel slides 1.5, 3, 3, 3, 3, 2.25, 1.5, 0.75, 0,
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

**MS-9 Depth-mode leak (claims 26, 31, 32).** The scenario forces a window that stays open whatever
the order in which objects are drawn, so its outcome does not depend on S-0102.
- *Test data:*
  - **Opener V:** one stock turretless vehicle type that casts a shadow gets the depth option (in
    the art section named after the type) and the shadow-cache-disable option (in its image's
    art section). Its part caches stay on. Parked, its parts are reused, so after its first
    drawn frame nothing of V switches the mode off, while its shadow, drawn fresh in every drawn
    frame, switches it on.
  - **Closer U:** a second stock vehicle type, one with a turret, gets the depth option and, in
    the test map override section, the no-shadow option, so it can only switch the mode off.
  - **Watchers W:** stock vehicle types without the depth option.
- *Setup:* a fresh game (empty image caches). The map places one U, parked, and one W. V is
  created later by a map trigger, inside the visible area, and stays parked. No object of any
  other depth-option type exists during the run; a run in which one appears (for example, built
  by the computer player) is discarded. U's body and turret keep their facings until phase 4,
  and V never turns (checked on the captures).
- *Procedure:* capture every drawn frame, in four phases.
  - *Phase 1, no opener.* W turns to a facing it has not shown before.
  - *Phase 2, opener present.* A map trigger creates V. Once V has been drawn in at least two
    drawn frames, W turns to another new facing, and a map trigger creates a vehicle of a
    second watcher type, at a facing never drawn before.
  - *Phase 3, opener gone.* A map trigger removes V from the map. W turns to another new facing.
  - *Phase 4, closer.* U's turret turns to a direction it has not shown before, while U's body
    keeps its facing (for example, through an attack order on a target in range). From the
    drawn frame after U's turret first moves, W turns to another new facing.
  - Finally W turns back to its phase 2 facing and then to its phase 1 facing.
- *Observable:* for each watcher image, whether its voxels are two pixels wide and its sections
  are drawn nearest first. Golden images are rendered both ways. A watcher image made in the
  depth look can also lack some voxels where an earlier depth-buffered draw left greater depths
  (claim 34, `quirks.voxel_depth_plane_partial_clear`); the look is judged on the pixels that do show.
- *Expected:*
  - phase 1: the normal look,
  - phase 2: the depth look, for W and for the second watcher,
  - phase 3: still the depth look (no part of a depth-option type has been drawn fresh since),
  - phase 4: the normal look (U's fresh turret draw switched the mode off, and nothing switches
    it on again),
  - on the way back, the phase 2 facing shows the depth look again and the phase 1 facing the
    normal look: these images are reused (claim 26).
- *Control:* the same run with V also given the no-shadow option. Every phase shows the normal
  look.
- *Falsified if:*
  - phase 2 or phase 3 shows the normal look,
  - phase 4 shows the depth look (as it would if only a body draw, not a turret draw, switched
    the mode off),
  - a facing seen again on the way back changes its look,
  - the control shows the depth look in any phase.

**MS-10 Barrel cache and elevation (claims 6, 26; `quirks.barrel_cache_ignores_elevation`).**
- *Setup:* a fresh game. One stock vehicle with a barrel part is placed by the map at a fixed
  facing, with its turret not turning. A few ticks later, a second vehicle of the same type is
  placed at a different facing.
- *Procedure:* capture both, then turn the first vehicle's turret through new directions.
- *Observable:* the barrel pitch of both vehicles.
- *Expected:* as long as the first vehicle's turret does not move, its barrel looks level at
  its initial key, because that cached image was made before the elevation rose. Directions
  first drawn later show the raised barrel.
- *Falsified if:* every barrel image shows the current elevation.

**MS-11 Alternative bodies and switchable turrets (claims 12, 26; `quirks.voxel_cache_alt_turret_frame`).**
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
  - **Type id.** A test vehicle type whose type id is the one S-0007 claim 16 names, added to the
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
  - the turreted spawner, while none is docked, shows its turret model in the body's place,
    posed as the body, with the turret drawn as usual,
  - with caching, while none is docked, body frames 2 and 3 show one and the same turret image.
    After a spawned aircraft has docked again, body frame 3 still shows that image, while body
    frame 2 shows turret frame 2 (`quirks.voxel_cache_alt_turret_frame`),
  - under the cache-disable option, every body frame *f_b* from 1 to 3 shows turret frame *f_b*,
  - every spawner's shadow keeps the main body's footprint throughout, whatever is drawn in the
    body's place,
  - the missing-file spawner shows its main body while a spawned aircraft is docked and no body
    at all, only its shadow, while none is,
  - the type-id vehicle shows its main body throughout the run without the option, and, in the
    run with it, its alternative body while none is docked and its main body otherwise.
- *Falsified if:*
  - the body does not switch with the docking state at the held facing,
  - the turret model does not follow the selected turret index at the held facings,
  - the rotating-barrel vehicle shows a test turret model,
  - the turreted spawner keeps its own body while none is docked,
  - with caching, body frames 2 and 3 show different turret images while none is docked,
  - with caching, after a spawned aircraft has docked again, body frame 3 shows a turret image
    other than the one body frames 2 and 3 shared while none was docked, or body frame 2 does not
    show turret frame 2,
  - under the cache-disable option, a body frame from 1 to 3 shows another turret frame,
  - a spawner's shadow changes shape when the body is replaced,
  - the missing-file spawner keeps its main body, or loses its shadow, while none is docked,
  - the type-id vehicle shows its alternative body in the run without the option, or keeps its
    main body while none is docked in the run with it.

**MS-12 Depth-buffer details (claim 24; `quirks.voxel_depth_zero_hidden`).**
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
    the next voxel's is not.
  - **Two-pixel test.** Two small overlapping sections, arranged so that the right-hand pixel of
    a farther voxel drawn first is later reached by a nearer voxel's left pixel, and the reverse.
- *Observable:* pixel crops at a fixed body facing (east, except where stated).
- *Expected:*
  - **Equal depths.** Only colour *Y* shows. Under the reversed order the second section is
    drawn first, and at equal depth the first drawn stays.
  - **Depth 0.** The rod's farthest voxels, those whose claim 24 depth is 0, are missing.
  - **All three models.** The golden image matches claim 24 exactly. Renders that test both
    pixels, use "greater or equal", keep the last drawn, or draw the sections in claim 23 order
    each differ from the capture in at least one pixel.
- *Falsified if:* colour *X* shows anywhere in the equal-depth model, the depth-0 voxels appear,
  or one of the alternative renders matches the capture where claim 24 does not.

**MS-13 Grid geometry (claims 13, 14, 15, 22).** The wrap of claim 22 is not tested here (see its backing).
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
  that every drawn frame shows the current elevation (`quirks.barrel_cache_ignores_elevation` cannot hide it).
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

**MS-18 Depth plane between draws (claim 34; `quirks.voxel_depth_plane_partial_clear`).** The outcome does not depend on the order in
which the two objects are drawn within a drawn frame.
- *Test data:* two stock turretless vehicle types get test models, the cache-disable option (in
  their images' art sections) and the no-shadow option (test map override section), so that both
  are drawn fresh in every drawn frame and draw nothing else:
  - **The probe** also gets the depth option. Its model is one flat layer of isolated voxels, spaced so
    that their pixels are at least 3 pixels apart on screen, spread over about 60 × 60 pixels,
    with no voxel of depth 0 (as in MS-12).
  - **The spoiler** has no depth option. Its model is a single voxel, so its copy rectangle is about
    9 × 9 pixels.
- *Setup:* a fresh game. The map places one probe and one spoiler, parked, both on screen, and no other
  object with a voxel model. A map trigger later removes the spoiler.
- *Procedure:* capture the probe in successive drawn frames while the spoiler is present, and after the spoiler is removed.
  Control run: the same with the spoiler also given the depth option.
- *Observable:* which of the probe's voxels show.
- *Expected:*
  - while the spoiler is present, from the probe's second fresh draw on, the probe shows only the voxels whose left pixel
    lies inside the spoiler's clear area (the spoiler's copy rectangle plus one column on the right and one row
    below, at the same voxel-buffer position; claim 34, F6). On screen that is a patch of about
    10 × 10 pixels around the point of the probe's image that the buffer centre maps to; every other
    voxel of the probe is missing,
  - after the spoiler is removed, from the probe's second fresh draw on, the probe shows all its voxels,
  - in the control run, the probe shows all its voxels from its second fresh draw on.
- *Falsified if:* with the spoiler present the probe shows voxels outside the spoiler's clear area (the depth plane was reset
  more widely), the probe shows no voxel at all, the probe stays incomplete after the spoiler is gone, or the control run
  loses voxels.

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
  - *(a)* 1 × 1 × 2 cells: one run of 0 empty cells and two voxels, both with colour index *X*
    and normal index *Y*. Read in mode 0 bottom-up, the first group covers both cells, so the
    column ends inside its own data.
  - *(b)* 1 × 1 × 42 cells: one run of 40 empty cells and two voxels, the first with colour index
    40 and normal index *Z*, the second with colour index *X* and normal index *Y*. Read in
    mode 0, the column ends inside its own data in both directions: bottom-up after one group
    (40 empty cells, then the values 40 and *Z*), top-down after one group (the values *Y* and
    *X*, then the colour index 40 taken as 40 empty cells). In a second run, model (b) gets a second HVA that turns the section
    upside down (half a turn about model x), so that its farthest corner is on its top face and
    it is drawn top-down (claim 21).
- *Observable:* the pixels of the voxels, at body facings 8 and 24, identified by grid cell on
  the golden image.
- *Expected:*
  - (a), mode 0: the lower voxel shows palette index *X* and the upper one *Y*, both unchanged by
    the VPL and the same at both facings. Mode 1: both voxels show VPL-lit versions of *X*.
  - (b), mode 0, upright (bottom-up): grid cell 40 shows index 40 and cell 41 shows *Z*, unlit,
    and nothing else of the section shows.
  - (b), mode 0, upside down (top-down): grid cell 41 shows *Y* and cell 40 shows *X*, unlit,
    and nothing else of the section shows.
  - (b), mode 1, both poses: cells 40 and 41 show VPL-lit versions of 40 and *X*.
- *Falsified if:* with mode 0 any voxel shows a VPL-lit colour or a colour that changes with the
  facing; in (a) the upper voxel shows *X*; in (b) any cell other than 40 and 41 shows a voxel, or
  cells 40 and 41 show other values than those expected for the pose (for example cell 41
  showing *X* in the upright pose, which would mean each voxel's colour and normal were read as a
  pair).

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
  In the reuse run the defence fires at the stationary target only, so that its direction stays
  the same.
- *Observable:* the barrel image per 32-step direction and per tick, and the marker position; in
  the reuse run, the palette index of the barrel's pixels in each drawn frame.
- *Expected:*
  - At each direction the image matches the golden image for *V · Rz(θ) · Ry(φ)* around the turret
    anchor: no shift with the turret displacement, and a pitch about the model origin. The pivot
    shows only while the barrel is pitched, so this comparison uses captures in which it visibly
    is: the 32-step pitch is read from the capture, and both pivots are rendered at that pitch.
  - Reuse run: while the direction stays the same, the barrel's pixels keep one row, during
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
    direction (drawn fresh, as the fixed behaviour of `quirks.barrel_only_no_recoil` would), or
    in the control it does not follow P's row.

**MS-22 Use of the lighting table file (claim 38).**
- *Test data:* the MS-4 stock vehicle on flat ground, drawn with named test copies of the lighting
  table file, each a loose file that overrides the stock one (glossary, *measurement setup*):
  - **(a) Unused parts changed.** The stock file with its remap range and its extra value changed
    to other values and its palette replaced by all zeros.
  - **(b) Extra rows.** A file made from the first 32 rows of the stock file (if it has fewer,
    padded to 32 rows with copies of its last row), then 8 added rows filled with values that
    differ from every entry of those 32, and the row count set to 40.
  - **(c) Trailing bytes.** The stock file with 100 bytes of any value appended after its last row.
- *Procedure:* with the stock file and with each copy, show the vehicle at all 32 facings and
  capture it. Read the stock file with a reader written from claim 38 and the public format, and
  draw MS-4's golden images from it.
- *Observable:* the palette index of each pixel in each capture.
- *Expected:* the stock file's captures match MS-4's golden images, drawn with row *q* of the file
  for shade *q*; the captures of (a), (b) and (c) are identical to the stock file's.
- *Falsified if:* MS-4's images match only with another choice of rows, or any of (a), (b) and (c)
  changes a pixel. A change under (a) falsifies the claim that the remap range, the extra value and
  the palette have no effect, and a change under (b) the claim that rows beyond the 32nd are not
  used.

**MS-23 Short, missing and small tables (claim 38; `quirks.vpl_unloaded_rows_blank`).** The
variants are named test files under the glossary's *measurement setup*: loose files that override
the stock one, and, in variant (d), a loose copy of the archive that holds the file, with that member left
out so that the file is absent. They are removed afterwards.
- *Setup:* the MS-4 stock vehicle on flat ground, with these variants of the file:
  - (a) the stock file with its row count set to 20, the data unchanged,
  - (b) the stock file cut after the end of its 20th row, row count unchanged,
  - (c) the stock file cut short so that it supplies no row at all and not all of its other
    content either,
  - (d) no file at all, and
  - (e) the stock file cut after the entries for colour indices 0–127 of row 19, row count
    unchanged. A second test model, as in MS-15's first-section test (one section of normal mode 2,
    voxels of normal index 16, which takes shade 19), has one voxel for each colour index 1–255,
    in any layout, drawn at body facing 8 on a stock turretless vehicle type.
- *Procedure:* start a game and show the vehicle at several facings, whose normals take shades
  throughout 0–31. For (e), show the second test model.
- *Observable:* whether the game starts, and which of the vehicle's pixels are drawn, and in variant (e) the
  palette index of the pixel of each colour.
- *Expected:* in every variant the game starts. (a): rows 0–19 act as in the stock file, so the
  vehicle's pixels of shade 20 or above are missing; (b) looks the same as (a); (c) and (d): the
  vehicle is invisible. (e): the voxels of colours 1–127 show the stock *VPL(19, χ)* and those of
  colours 128–255 are missing.
- *Falsified if:* the game fails to start, the missing rows draw as raw colour indices or as some
  other colour, or in (e) the voxels of colours 128–255 are drawn or the whole row is missing.

## Open questions

- **Turretless vehicle with a barrel part.** A vehicle with a barrel part but no turret is drawn
  differently, with a shift derived from its weapon position and a pitch correction. Not yet
  understood; left out.
- **Shadow vertical placement.** Where an airborne unit's shadow footprint lands vertically (its
  ground point) is decided by the locomotor and is left to S-0102 and S-0106; the one-pixel
  adjustment for airborne units seen in the original is unconfirmed.
- **Aircraft draw order of body and shadow** follows the same pipeline, but its exact sequence has
  not been determined.
- **Masked shadows.** A shadow can also be drawn through a per-pixel mask image, but when that
  happens is not yet clear.
- **Shadow on a cache miss.** When a shadow image is not yet cached, the shadow sometimes appears
  to be skipped for that draw instead of made. In which situations, and whether a shadow can be
  missing for one drawn frame, has not been determined.
- **Oversized parts.** How a copy rectangle larger than the 256 × 256 voxel buffer (claims 22 and
  25, `quirks.voxel_buffer_wrap`) reaches the screen, whether clipped or read past the buffer's edge, has not
  been determined. Until it is, the wrap of claim 22 and the colour residue it leaves (claim 34)
  cannot be measured.
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
- **Cache lifetime.** When cached images are discarded (budget, map change) has not been determined. It
  bounds how long the artefacts of `quirks.voxel_cache_frame_mod32` and `quirks.barrel_cache_ignores_elevation` persist.

## Questions for the project lead

- **Barrel recoil values in S-0006.** S-0006 claim 5 lists the keys that reset on override, and
  the barrel recoil values of claim 27 (`quirks.barrel_recoil_layer_reset`) are not among them. S-0006 should list them as
  a further exception (reset to the turret's value, not to an engine default), or point to this
  spec for it.
- **Type id.** Is the type id of claim 12 and MS-11 (Terms) the type name or the registration index
  of the glossary? If it is the type name, the spec should say "type name".
- **Byte layouts of public file formats.** Claim 38 now refers to the public format of the
  voxel lighting table file for its byte layout, as the spec does for VXL and HVA, and gives only
  how its content is used. The Terms entry *Column data* now defines a column's value sequence
  by reference to the public model format's own order. Claim 35 (and MS-19) cannot be
  stated without saying how that sequence is cut into groups and which value of a standard file's
  voxel comes first, and this is as far as the draft goes: it states no sizes, positions or
  byte layouts, only the order of a voxel's two values, once, in claim 35. If the lead wants even
  that moved to a formats spec of the model file, claim 35 would cite it instead. Not
  blocking.
- **Buffer-edge cases and importer warnings.** The importer warns about the model-file cases of
  claims 12, 35 and 36 (Import mapping, *Importer warnings*) and not about the three pose- or
  size-dependent cases of claims 22, 24 and 34 and F10. Should the importer also warn when a
  model's own size already guarantees an oversized copy rectangle (claim 22)?
- **Fixed behaviour of three quirk switches.** Each offers two alternatives; which one does the
  switch select when turned off?
  - `quirks.voxel_buffer_wrap`: clip, or enlarge the buffer.
  - `quirks.barrel_pivot_frame_mixup`: the turret origin, or an imported pivot (no pivot key is in
    Import mapping yet).
  - `quirks.depth_mode_leak`: switch the mode off after every shadow, or never switch it on for
    shadows.
