---
id: S-0105
title: Rendering of voxel models
topic: rendering
revision: 50
sources-class: [decomp, format-doc, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-12
measurements: []
requests: [Q-0014, Q-0056, Q-0210, Q-0211, Q-0213, Q-0215, Q-0216, Q-0470, Q-0799, Q-0950,
Q-1760, Q-1885, Q-2672, Q-2982, Q-2983]
supersedes: []
---

# S-0105: Rendering of voxel models

## Summary

This spec defines skirmish voxel rendering: model-part placement and animation,
aircraft attitude, lighting, pixel coverage, recoil, shadows and image reuse.
Slope tilt, scene compositing, palettes and map lighting belong to the related
specs; campaigns and network games are out of scope.

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
- **Rocket drawing pitch** *p*. S-0506's single-precision nose pitch in radians,
  positive nose up; it is not a raw facing. S-0506 claims 14–18 and 24 own its
  evolution. Claim 41 only specifies how that current value appears.
- **Flight drawing heading.** For the flying locomotor, the aircraft's aiming
  facing (S-0501 claims 19 and 42), rounded to the 32-step facing by the rule
  above; this need not equal the body facing that governs horizontal movement
  in S-0501 claim 4. For the rocket locomotor, use the body facing
  instead. This distinction also applies to their shadows.
- **Flight bank** *β*, **flight drawing pitch** *α*. Local x-axis and y-axis
  rotation angles in radians, respectively, for a non-crashing flying-locomotor
  body (claim 42). Positive *α* lowers the nose; these are drawing angles, not
  the rocket's nose-up pitch *p* or a change to its trajectory.
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
- **Shadow draw point.** The screen point a shadow image is placed around. Claim 40
  gives it for aircraft, claim 44 for vehicles. The
  footprint's own displacement and scale (claim 31) are relative to this point.
- **Aircraft deck mark.** The aircraft's own standing-on-deck state, whose landing,
  climbing and departure transitions are owned by S-0501 claim 43. It is not inferred
  from render layer, occupant listing or cell-layer occupancy flags.
- **Aircraft shadow height** *a*. For claim 40 only, the aircraft's signed integer
  height above its current surface, in leptons: *a = z − H − b*, where *z* is its
  world height, *H* is the terrain surface height at its exact horizontal position
  (S-0106 claim 2), and *b* is 416 leptons while the aircraft deck mark is set
  (S-0501 claim 43), otherwise 0. This is not S-0501's altitude-guidance height.
- **Aircraft-like type.** A type with the aircraft-like flag (import mapping), which an aircraft
  type has unless its data clears it; its shadow is half size (claim 31).
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
  advances for gattling-flagged types is S-0609 claim 14's rule, including its correction
  of the building's presentation-only double advance outside *attack*. Other advances belong
  to the vehicle, building and combat specs.
- **Turret anchor.** The screen point at which a building's voxel turret and barrel images are
  centred: the building's draw point moved by the building type's turret-animation x and y
  displacements, in whole screen pixels (x to the right, y downward). Both are fields of the
  building's rules section, not of its art (import mapping; S-0108 reads the same two fields for
  its turret slot animation), and both are 0 when absent (claims 9 and 37).
- **Cloak state.** S-0516 claim 1 defines the four cloak states: *uncloaked*, *cloaking* (fading
  out), *cloaked*, *uncloaking* (fading in). Only *uncloaked* counts as fully visible here
  (claim 31). S-0516 owns the cloak-state transitions.
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
  have rotating-barrel weapon stages (import mapping), whatever its body. With the voxel flag
  such a type has one turret model, and optionally one barrel model, per turret index 0, 1, …
  (file naming: S-0007 claim 16). A sprite-body vehicle (voxel flag not set) has no numbered
  models (S-0007 claim 19) and draws as claim 12's *Sprite body* says; the index and weapon
  rules of claim 43 apply to it all the same.
- **Selected turret index.** A per-object value that says which of a multi-turret type's turret
  models is in use: either a turret index (an integer, possibly outside 0 … *N* − 1, where *N* is
  the type's turret count) or *none*. Claim 43 says what sets it. It picks the turret model drawn
  (claim 12). It never selects a weapon: every test of a vehicle's current
  weapon asks the weapon S-0308 chooses, which for a multi-turret vehicle is that of its weapon
  number (S-0308 claim 5) unless S-0308 claim 9's override for a deployed vehicle applies, also
  while the index is *none* (claim 43).
- **Turret table.** A per-type table of 18 entries, one per weapon number 0 … 17 (S-0308 Terms),
  each holding a turret index or *none*. Every entry is *none* unless the import fills it, which
  happens only for one fixed vehicle type name (Import mapping).
- **Charge-turret type, gunner type, weapon number, mode index.** As in S-0308 Terms. A
  charge-turret type's turret follows its rearm countdown (S-0301 claim 16).

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
   at most 32 distinct heading orientations on flat ground. S-0106 supplies slope
   tilt; claims 41–42 supply rocket pitch and flying-locomotor pitch and bank.
   For those two locomotors, use the flight drawing heading (Terms).
   - Backing: `measurable` (MS-1; MS-31/MS-32 for rocket and flying heading sources)
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
   - **When the turn starts (Q-2982).** A non-building object's elevation is set level, and its
     turn begins (*t*₀ of F9), each time the object is placed on the map (glossary), at the tick
     number current then, except at a chronosphere warp's appearance. Nothing else sets it,
     except the end of a drop-pod stand-in below, so a turn under way goes on by F9's tick count
     through every other event. The cases:
     - A new vehicle leaving its factory, and a vehicle unloaded from a transport, start level
       and turn again, however long the vehicle was carried. So does any vehicle that leaves a
       building it was inside, since inside a building it is off the map (glossary) and leaving
       places it again.
     - A vehicle lifted by a carrier aircraft is off the map, held by the aircraft as a passenger
       until it is set down (S-0102 claim 22), though it is still drawn under the aircraft (claim
       44's lifted vehicle). The aircraft sets it down by placing it on the map, so the vehicle
       starts level and turns again, as an unloaded one does.
     - A vehicle bunkered in a tank bunker stays on the map (S-0606), so entering and leaving the
       bunker keep its elevation. So does a vehicle on a repair depot's pad, through the repair
       and leaving. Whether a harvester docked at a refinery stays on the map is not settled
       (S-0501 claim 32 and Open question 23). Its elevation follows the rule above either way:
       kept if it stays on the map, set level and turning again if docking takes it off and
       leaving places it again.
     - The exception: a chronosphere warp's member is off the map between its departure and its
       appearance at the landing point (S-0513 claim 39), but that appearance keeps the
       elevation. A member has been on the map since the launch at the latest, and S-0513 claim
       39's wait before the departure is longer than any turn of F9, so its turn has ended before
       it departs and the appearance shows its final elevation.
     - A vehicle seized by a magnetic lift stays on the map while it is pulled, held, set down or
       dropped and while it falls (the events of S-0607), so none of these, its landing included,
       restarts the turn.
     - A move by the teleporting locomotor keeps the object on the map, whether it is the
       object's own move (S-0207 claim 20) or the teleporting stand-in's move of a unit seized by
       a magnetic lift (S-0607 claims 22 and 23), so the jump and the landing keep the elevation.
     - A unit seized under S-0607 claim 23's drop-pod stand-in is, in the original, taken off the
       map at touchdown and put back at the landing point, which starts its turn again. Our
       engine gives that touchdown result at once at the seize (S-0607 claim 23) and leaves the
       unit on the map, so there the end of the stand-in sets the elevation level and starts its
       turn, at the tick number current then (design).
     - An object dropped by parachute is placed when it is dropped, so its turn runs while it
       falls, and landing does not restart it. In skirmish only infantry are dropped (S-0522),
       and they have no barrel part, so this is never seen.
     - Aircraft-kind objects have an elevation too, but claim 45 draws no barrel for them, so it
       is never seen either.
   - **First appearance.** A drawn frame shows the elevation at the tick number *n* current while
     it is drawn, so at *ε = n − t*₀ (F9). That number has already grown by one at the end of the
     tick the frame follows (S-0201 claims 1 and 6), so an object placed during a tick is first
     drawn at *ε* = 1, never at *ε* = 0. Within the placement tick the elevation is at *ε* = 0
     (S-0206), but no drawn frame shows that value. An object that the map places before the
     first tick has *t*₀ = 0. With the illustrative fire angle of 20 above, a vehicle unloaded
     during a game shows the 32-step values 8, 7, 7, 6, 6, 6 in its first six drawn frames.
   - **Saved games.** A saved game keeps each object's elevation and its turn, and the tick
     number too: after a load the tick number is the one current at the save. So every turn goes
     on where it was, and a vehicle saved during its turn is drawn neither level again nor at its
     final elevation at once. Joining a network game in progress is outside this spec, as are
     network games.
   - Backing: `measurable` (MS-2 and MS-14 for vehicles, MS-6 for buildings; MS-35 for the
     restart at a factory exit and an unload, the first appearance, the tank bunker, the repair
     depot, the chronosphere warp, the teleporting stand-in, the magnetic lift, and saved games
     with the tick number they keep). The rule that only a placement sets the elevation is
     `measurable` case by case through those parts. Apart from part e's save, those parts take
     each event after the turn has ended, so they show that the event does not restart it, not
     that a turn still under way goes on through it. That a turn under way goes on through an
     event other than a save is `unmeasured`: it follows from nothing else setting the elevation.
     Leaving another building it was inside, the carrier aircraft, the refinery dock, the
     parachute case and the drop-pod stand-in are `unmeasured`: the first follows from the
     glossary and the rule, which part b tests for the factory, a carrier aircraft's lift needs a
     start and an end that the transport specs leave outside S-0507 claim 13, the refinery dock
     waits on S-0501's Open question 23, no barrel part is ever dropped in skirmish, and the
     drop-pod stand-in needs a seizing warhead that names that locomotor (S-0607 claim 22). Our
     engine's restart at the end of that stand-in is a design rule.
   - Confidence: high (the turn timing of F9, the restart at a factory exit and an unload, and
     the first appearance at *ε* = 1); medium (the shown pitch, the rule that only a placement
     sets the elevation, a turn under way going on through other events, and leaving another
     building, the carrier aircraft, the bunker, the repair depot, the chronosphere warp, the
     teleporting locomotor, the magnetic lift, the drop-pod touchdown and the parachute case
     keeping or restarting it, and saved games with their tick number, until MS-35 part e
     runs); low (the refinery dock, as S-0501 rates it)

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

11. **Turret and barrel overlap.** The turret and barrel are separate images. At 4-step
    turret facings 1 and 2 (roughly east or south), barrel pixels win wherever the two
    images have nontransparent pixels at the same position. At facings 0 and 3, turret
    pixels win instead. Vehicles and buildings have the same visible result. Wherever
    either image has a nontransparent pixel over the body, its pixel wins over the body's.
    Transparent part-image pixels do not hide another part.
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

    A vehicle of a multi-turret type with the voxel flag draws
    the turret model and barrel model of its selected turret index *k* (claim 43) instead of the
    type's ordinary turret and barrel, which such a type never has (S-0007 claim 16). A type with
    rotating-barrel weapon stages always draws its ordinary turret and barrel, whatever its turret
    count. File naming is S-0007.
    - **No model for the index.** When the index is *none*, or *k* is outside 0 … *N* − 1 (*N* the
      turret count), the vehicle is drawn with neither a turret nor a barrel. The body is drawn
      as usual. A missing numbered barrel *k* alone leaves turret *k* drawn without a barrel.
    - **Original glitches, fixed without a switch.** In the original, a vehicle whose type has 18
      turrets and whose index is *none* shows its turret model 17 in the barrel's place; our
      engine draws nothing there. It is a drawing artefact with no gameplay impact, so it gets no
      switch. An index of 18 or more, or below −1, names no turret any type can have (it can come
      only from an imported turret table entry, Import mapping); what the original shows for it is
      *unspecified*. Our engine draws neither part, and the importer warns about such entries.
    - Backing: `measurable` (MS-11; MS-33 for the index rules and the missing-model cases).
      `unmeasured` for the original's glitches with *N* = 18 and with *k* ≥ 18 or below −1, which
      MS-33 does not set up
    - Confidence: high (missing-body fallback, no alternative without the option, and nothing
      drawn for *none* or an index from *N* to 17), medium (remaining model choices and the
      original's glitches)
    - **Sprite body (S-0007 claim 19).** A vehicle drawn with a sprite body (voxel flag not set)
      never draws numbered models, whatever its turret count, and its selected turret index (claim
      43), *none* included, selects neither its sprite frames nor its voxel barrel model. With the
      turret flag it draws its body and turret image frames as S-0102 claim 22 says, at every
      index. Of the two voxel parts S-0007 claim 19 loads for it, only the barrel is drawn. The
      voxel turret part is loaded but never drawn, and without a barrel part no voxel part is drawn
      at all; the body and turret frames are then still composed into one image (*One image*,
      *Depth*).
      - Backing: `measurable` (MS-33 part k, whose sprite runs use the fixed-name type, so the
        filled turret table takes the index through 2, 0, 1 and *none*), for the parts drawn at
        each index
      - Confidence: high (no numbered model and no voxel turret part drawn), low (the sprite turret
        shown at every index, no higher than S-0102 claim 22 rates the turret path it rests on, as
        part k shows that a sprite turret is drawn at each index, not which frame)
      - **Barrel pose.** The barrel is placed by claim 8's transform, the shift first and then the
        turn about the shifted point, but with the displacement in claim 9's unit: *d =
        trunc(δ / 8)* model units along the body's forward axis (integer division, truncated toward
        zero; *δ* the imported turret displacement in leptons). It is turned by claim 4's relative
        turret angle and pitched by claim 6's 32-step elevation about claim 10's axis, the lateral
        axis through the shifted point. In the original the pitch pivot of this barrel lies off
        claim 10's axis, at a point that also depends on the firing position of the vehicle's
        weapon in slot 0, and moves with the turn. Our engine uses claim 10's axis, as the
        departure for claim 10 does for other barrels (Lead-confirmed presentation departures),
        extended to this barrel at the delegated sign-off, 2026-10-25. At level elevation the two
        pivots give the same image.
        - Backing: `measurable` (MS-33 part k's pose runs, for the displacement and the
          turn); the pitch about claim 10's axis at an elevation other than level is our engine's
          design rule, with no run cited, since the pose runs hold the barrel level and the
          invisible runs check only the elevation step; `unmeasured` for the original's off-axis
          pivot: it changes the image only at an elevation other than level, which the pose runs
          do not set, and our engine does not reproduce it
        - Confidence: medium (the displacement and the turn, until MS-33 part k runs), medium
          (the pitch axis, a design rule), medium (the original's off-axis pivot)
        - **Invisible slot-0 projectile (original glitch, fixed without a switch).** When the
          projectile of the vehicle's weapon in slot 0 is invisible (S-0306 claim 9), the
          original draws this barrel at a distorted elevation: a raised barrel is shown below
          level and a lowered one stays below level, each at about a third of its angle from
          level, so the absent-key fire angle of claim 6 shows level. The weapon in slot 0 is
          the one S-0308 claim 2's per-slot replacement gives: while the vehicle is elite, the
          weapon of its type's elite slot 0 when that elite slot holds a weapon, and the weapon
          of slot 0 otherwise. Which slot S-0308 chooses against a target does not matter. Our
          engine draws claim 6's elevation. It is a drawing artefact with no gameplay impact, so
          it gets no switch, as decided at the delegated sign-off, 2026-10-25.
          - Backing: `measurable` (MS-33 part k's invisible runs, for the original's raised
            barrel and for our engine's elevation); `unmeasured` for the original's lowered
            barrel: the invisible runs use raised elevations only, and the case describes a
            drawing artefact our engine does not reproduce; for an elite vehicle, which slot-0
            weapon applies rests on S-0308 claim 2 and is not run here, since the invisible runs
            do not vary veterancy
          - Confidence: medium (the original's raised barrel, until MS-33 part k runs), medium
            (the original's lowered barrel), medium (the elite case, as S-0308 claim 2 states it)
      - **Frame (Q-1885 question 4).** The barrel is always drawn in HVA frame 0, whatever the
        walk-animation and animation-frame counters. `quirks.barrel_frame_zero` does not apply to
        it: with no voxel turret drawn there is no turret frame to take, so it stays in frame 0
        in both switch states (claim 16).
        - Backing: `measurable` (MS-33 part k, frame run, for the original, which is the switch's
          default state); in the other switch state, frame 0 is our engine's design rule, since
          the switch covers barrels drawn with a voxel turret, and no run tests it
        - Confidence: medium (the original, until MS-33 part k runs), medium (the other switch
          state, a design rule)
      - **One image (Q-1885 question 3).** The body frame, the barrel and the sprite turret frame
        (without a barrel part, the body frame and the sprite turret frame) are composed into one
        image, without any depth test between them and with transparent
        pixels painting nothing. The body frame comes first; at 4-step turret facings 0 and 3
        (Terms, the turret's own facing, not its facing relative to the body) the barrel follows
        and then the turret frame, and at facings 1 and 2 the turret frame and then the barrel.
        Later pixels win. So the barrel and the sprite turret both cover the body wherever they
        have pixels, and claim 11's rule by the 4-step turret facing alone decides between barrel
        and sprite turret. The composed image stands in the vehicle's render layer at the
        vehicle's place in the drawing order (S-0102), drawn at the body's draw point, so on a
        bridge deck the barrel is lifted with the body. The vehicle's own shadow (S-0102 claim 22)
        is drawn before the composed image and keeps its own row.
        - Backing: `measurable` (MS-33 part k, overlap runs)
        - Confidence: medium (the overlap rule by 4-step turret facing, which extends claim 11 to a
          sprite turret, until MS-33 part k runs)
      - **Depth (Q-1885 questions 1 and 2).** A vehicle drawn as a sprite whose type has the
        turret flag, with or without a barrel part, is drawn as one composed image (*One image*).
        The composed image has one depth bias and profile for all its pixels: S-0102 claim 13's
        row for a vehicle drawn as a voxel model, *U* with no further constant, upright, a reader,
        at every height above the surface, on a bridge deck and in the air included. The one
        exception is the split of a tall image (*Split of a tall image* below). Its rectangle,
        whose last row is the anchor row *r*₁ of S-0102 F2, is the smallest rectangle holding the
        rectangles of all its parts, each placed at the body's draw point: the body frame's stored
        rectangle, the barrel's copy rectangle (claim 25, F6), where the type has a barrel part,
        and the turret frame's stored rectangle. So while the barrel and the turret frame end
        above the body frame's last row, *r*₁ is the body frame's last row, as for a vehicle drawn
        as a sprite without the turret flag. In the original, when the barrel or the turret frame
        reaches lower, or further right, than the body frame, the rectangle ends one row (or
        column) past the last row (or column) of the lowest (or rightmost) such part. Our engine
        takes the exact rectangle: the extension moves the anchor by at most one row, a drawing
        detail with no gameplay impact, so it gets no switch, as decided at the delegated
        sign-off, 2026-10-25. So another object's pixels (a building, a tree or a wall) hide or
        show the barrel exactly as they hide or show the body and the sprite turret at the same
        screen rows.
        Neither of S-0102 claim 13's rows for a vehicle drawn as a sprite (*U* − 2 at height 0,
        −*L(z)* − 2 otherwise) applies to a vehicle with the turret flag, whether or not it has a
        barrel part. They apply to a sprite-body vehicle without the flag (Cross-spec
        reconciliation). Another vehicle, also a reader, does not hide any part of it through
        depth: their overlap follows the drawing order.
        *Split of a tall image.* In the original, the composed image of a type too big to fit
        under a bridge is split when it is taller than 16 rows and either the vehicle is near a
        bridge with its column factor 0 (the condition of S-0102 claim 13's row for such a type,
        claim 14 there) or one further situation holds that is not identified here (Open
        questions). Its lowest 16 rows then keep *U*, upright, and the rows above them take
        *U* − 5, flat. The image of a type without that flag is never split, whatever its height.
        Vehicles drawn as voxel models are split in the same way. S-0102 owns the split
        (Cross-spec reconciliation).
        - Backing: `measurable` (MS-33 part k, depth runs: at height 0 for the bias and the shared
          anchor row, with a barrel part and, in the depth run without a barrel, without one; on the
          bridge deck for the one profile of all pixels); `unmeasured` for which row applies on the
          bridge deck and in the air: there the voxel row and S-0102 claim 13's row for a sprite
          with *h* not 0 differ by about one depth unit, inside F2's rounding, so the deck runs
          cannot tell the two rows apart, and no run lifts the subject; `unmeasured` for a rectangle
          whose last row is not the body frame's, where the barrel or the turret frame reaches lower
          than the body frame: the depth runs build the test sprite file so that the body frame's
          last row is lowest, and no run makes another part reach lower; `unmeasured` for the
          original's one-row extension of the rectangle: the depth runs are built so that it does
          not arise, and our engine does not reproduce it; `unmeasured` for the split of a tall
          image: no scenario here sets up a type too big to fit under a bridge (the depth runs clear
          that flag, so no split arises in them), and the split's second condition is not identified
        - Confidence: medium (the row at height 0, with and without a barrel part, and the one
          profile on the bridge deck, until MS-33 part k runs), medium (which row applies on the
          bridge deck and in the air), medium (a rectangle whose last row belongs to the barrel or
          the turret frame), medium (the original's one-row extension), low (the split of a tall
          image, whose second condition is open)
      - **No barrel shadow.** The barrel casts no shadow of its own. The vehicle's only shadow is
        its sprite shadow frame (S-0102 claim 22), whose outline does not include the barrel.
        - Backing: `measurable` (MS-33 part k, shadow observable)
        - Confidence: medium, until MS-33 part k runs

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
    remainders of non-negative counters. **Barrels drawn together with a voxel turret, of
    vehicles and buildings alike, are always drawn in HVA frame 0** (`quirks.barrel_frame_zero`). A
    building's barrel drawn without a turret uses the building's animation-frame counter instead
    (claim 37). The barrel of a sprite-body vehicle is drawn in frame 0 in both switch states
    (claim 12, *Sprite body*). **A numbered turret model** of a multi-turret type (claim 12) is drawn in frame
    *f_b* whatever its value, so in frame 0 whenever *f_b* = 0: the animation-frame counter is
    never used for it. Claim 13 then reduces the frame modulo that model's own HVA frame count.
    Its turret image key (claim 26) records the same frame. Numbered barrels follow the barrel
    rule above. The animation-frame counter is 0 when the
    object is created. For gattling-flagged types, S-0609 claim 14 owns its advances:
    a vehicle advances once per completed firing step with positive post-update spin,
    and our engine advances a building once per tick with positive post-processing spin,
    without a separate advance for firing or leaving *attack*. Animation can therefore
    continue during spin-down after attacking stops. The original building adds a second
    advance outside *attack* when spin remains positive after passive decay; S-0609 claim 14
    corrects this presentation-only bug without a quirk switch.
    For other types, changes arise through attacking; their exact rule belongs to the
    combat, vehicle and building specs. A vehicle or building that has never attacked shows
    turret frame 0 whenever its turret frame comes from this counter.
    - Backing: `measurable` (MS-3 and MS-3b for the body, the barrel and *f_b ≠ 0*; MS-3c for
      *f_b = 0* and building turrets; MS-33 part e for numbered turret models following *f_b*).
      That the animation-frame counter is never used for a numbered turret model is `unmeasured`:
      MS-33 does not record the counter, so a counter that stayed at a multiple of the turret's 4
      HVA frames during part e's attack would give the same images.
      The exact mapping from animation-frame counter u to a vehicle turret at f_b = 0 or
      a building turret is `unmeasured`: MS-3c observes changing frames but does not record u,
      so it cannot distinguish the precise modulo mapping. The counter's initial zero and
      subsequent changes are the parts tested there; S-0609 claim 14 and its scenario G11
      cover gattling advances and the original building's double advance separately.
    - Confidence: high (barrel frame 0, body frame), medium (turret frame, numbered turret frame
      and counter advances)

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
    - Projectiles drawn as voxel models also give shades to the indices their table covers,
      by S-0306 F16 (claim 30). Debris drawn as voxel models also gives shades to the indices
      its table covers, by a rule that belongs to its own spec.
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
    file order (`quirks.voxel_section_sort_ties`). F8 gives the full-sequence priority
    predictions to test in MS-25 and MS-28, including the latter's larger mixed ties.
    There is no depth test between sections, so a farther voxel in the higher-priority
    section can cover a nearer voxel in the other section.
    - Backing: `measurable` (MS-25/MS-28) for every listed pair and common winner,
      the five-section all-equal control, absent-section controls and gap-change
      repeats in both modes; MS-5 covers distinct-depth coverage independently of ties.
      F8's relation and input sufficiency for arbitrary counts remain `unmeasured`:
      the finite scenario set cannot settle every count or comparison pattern.
      MS-28's closing sweep tests further cases, not the universal statement.
      Interim engine pixels are not evidence.
    - Confidence: high for the listed priorities; medium for the arbitrary-count relation

24. **Per-type depth buffering.** A type with the depth-buffer option draws differently in four
    ways. Claim 32 confines this mode to fresh depth-option part draws.
    - **Section ties and depth.** At coincident left pixels without right-hand
      interference, the greatest numeric voxel depth wins. Equal numeric depths
      favour the section with highest F8 overlap priority, even when section nearest
      depths differ. A passing right-hand write from a lower-priority section can
      replace a higher-priority section's pixel at that neighbour, even when the
      replaced depth is greater. The left-pixel test below still applies. Within-section
      coverage follows claim 21's depth-mode rules, not its greatest-tuple painter
      winner. These are pixel outcomes, not a required order of section drawing.
    - **Depth test.** A voxel's comparison depth is the unwrapped signed integer
      *d = ⌊pz / 256⌋*, with *pz* from F5, one model unit per depth level. Before each
      fresh depth-mode draw, every coordinate is empty, below all numeric voxel depths
      (claim 34). Clearing depth to 0 denotes that empty sentinel, logically −∞,
      not numeric voxel depth zero. A voxel passes an empty coordinate or one whose
      stored numeric depth is strictly smaller. Zero and negative depths therefore
      pass empty coordinates, and wrapping never reverses depth order. Without
      right-hand interference, equal depths favour the smallest priority tuple within
      a section (claim 21) and the highest-priority section between sections (F8).
    - **Two pixels per voxel** (`quirks.voxel_depth_double_width`). A voxel that passes the test writes its colour (0 included, claim 19) and its
      depth into its own pixel and into the pixel immediately to its right. Only the left pixel's
      stored depth is tested: the right pixel is overwritten even if it held a greater depth. At
      every image column, including 255, the right-hand pixel is *(x + 1, y)*,
      never column 0 of another row. Claim 22's retained window limits which pixels
      reach the screen, not their coverage or comparison-depth result inside it.
    - **Independent coordinates.** Horizontal arithmetic never carries into vertical
      position in either mode (F5).
    - Backing: `measurable` (MS-25/MS-28) for equal-depth section winners on the
      listed patterns. MS-5c isolates within-section equal-depth and right-write
      coverage; MS-12 isolates cross-section ties and right-write coverage. MS-5
      covers ordinary depth output. MS-12/MS-18/MS-24 provide engine conformance
      for lead-confirmed depth, clearing and coordinate departures. No internal
      drawing sequence is claimed as a measured observable.
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
    - **Rocket body:** the 32-step body heading, complete applicable HVA frame and
      pitch reuse class of claim 41, instead of a ramp state. Pitches outside its
      three eligible classes always draw fresh and are not retained.
    - **Flying-locomotor body:** the 32-step flight drawing heading, complete
      applicable HVA frame, the presence of speed-dependent pitch, and bank state
      (none, clockwise turn, counterclockwise turn; none when the configured
      bank angle is zero), as independent components.
      Claim 42 supplies the pitch/bank states. No ramp state is included. This key
      covers ordinary, non-crashing flight; crashing appearances at strictly
      positive signed height are fresh.
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
      elevation, MS-6 recoil and MS-9 mode isolation. MS-31/MS-32 propose direct
      read-only reuse observations for rocket and flying body/shadow keys, including
      their exceptional freshness conditions. Corrected cache components are
      lead-confirmed departures tested against this spec rather than the original. Global image invalidation in the original is `unmeasured`: no scenario here forces
      such an event or separates it from a new reuse class. MS-3b's rendering fixture
      tests complete building turret frames without requiring an oracle counter above
      255. The naval locomotor case remains `unmeasured`: when a ship's body tilts is S-0106's (claims 6 and 7), but stock data may give a
      ship no tilt at all (S-0106, Open questions), and none of this spec's scenarios uses
      S-0106's naval test data (its scenario T6), so none of them makes it happen. A body
      drawn fresh beside a reused turret is covered by MS-17 (turret's body facing, run 2). The point at which the original empties its caches is not a claim of this spec (a
      departure, above), and whether a new game empties them is an open question.
    - Confidence: high (complete frames, separate alternative state, current elevation
      and body facing in the turret key; rocket and flying body keys, MS-31/MS-32), medium (ramp state and unsettled tilt, whose details
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
    body do not change the unmasked footprint's outline; this includes the rocket's
    nose pitch and the flying body's pitch and bank (claims 41–42). Flying shadows
    use the flight drawing heading, rocket shadows the body heading (Terms).
    They can change the own-image
    exclusions of claim 39.
    - Backing: `measurable` (MS-8) for ordinary footprint positions and present-column output.
      Unwrapped right-pixel adjacency is a lead-confirmed departure tested by MS-24's
      enlarged-footprint variant. Independence of the outline from body height
      and rocking is also `unmeasured` for vehicles: MS-8 has no independently
      controlled comparison of either input. MS-31/MS-32 test rocket pitch and
      ordinary flying pitch/bank independence separately.
    - Confidence: medium (high for the two pixels per column and for present columns without
      voxels)

31. **Shadow shift and scale.** The footprint is shifted 3 pixels to the right on screen (exactly
    *−6 × L_x*, stored truncated as 2.9999964, before the footprint's positions are truncated, F10).
    For aircraft-like types (by default every aircraft type, Import mapping), the footprint is
    scaled by 1/2 about the shadow's draw point, and the 3-pixel shift is not halved (F10). The
    shadow is drawn with the ordinary shadow treatment (S-0102, S-0104). It is skipped whenever the
    object's cloak state is not *uncloaked* (that is, while cloaking, cloaked or uncloaking), and
    when the type has the no-shadow option. Shadow images are cached per type like part images (key:
    ramp, 32-step body facing; for the flying locomotor substitute its 32-step aiming facing, claims
    41–42), unless the type has the shadow-cache-disable option. They are also neither looked up nor
    stored, but drawn fresh, while a driving or naval unit's body is not settled or while a hovering
    unit rocks (S-0106, claim 12). These freshness rules do not override the cloak and no-shadow
    suppression above. For cache-eligible shadows, the key covers every input of the unmasked
    footprint (frame 0, the type's shadow section and scale are fixed per type). Cache only the
    unmasked footprint: current own-image exclusions are recomputed on every appearance (claim 39),
    and no shadow enables depth mode (claim 32). Freshness still matters where the original skips a
    shadow on a cache miss (open questions). That cache-miss skip does not apply to the unsettled
    shadows just described. Until those cache-miss cases are known, our engine makes the shadow on
    every cache miss.
    - Backing: `measurable` (MS-8) for the shift, scale, cloak and no-shadow conditions. The
      aircraft-like flag's absent default for an aircraft type is `measurable` through S-0501
      scenario AC30 parts h (2) and j, and rated medium, as S-0501 rates it. MS-9 tests corrected
      mode isolation; MS-27 tests current-image exclusion with caching enabled and disabled, as
      lead-confirmed departures. Hovering unsettled shadow freshness is `unmeasured`: corrected
      exclusions agree for cached and fresh footprints, so a rocking comparison does not
      distinguish freshness. The driving/naval unsettled case and exemption from cache-miss skip
      remain `unmeasured`: no scenario here isolates their freshness or the miss-skip event from
      changed keys and other fresh draws. Also `unmeasured` in two places. The look of a reused
      unmasked footprint: its key covers every input of that footprint, so fresh and reused unmasked
      footprints have the same pixels. Vehicle own-image exclusions also agree for fresh and reused
      appearances (claim 39). The ramp-key component is `measurable` through the changed footprint
      in S-0106 T3 and T6. The body-facing component is `unmeasured`: a new body facing also gives
      the body image a new key, so no scenario here separates their fresh-draw side effects. The
      skip on a cache miss is not a claim yet (open questions).
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
      Depth preservation between non-depth draws and the initial depth clear are
      `unmeasured`: every fresh depth draw clears depth completely, so screenshots
      cannot distinguish retained or initially cleared depth before that draw.
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
      marker and attacks, not u, and therefore tests only the initial frame and subsequent
      changes, not the exact counter-to-frame mapping (gattling advances: S-0609 claim 14).
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
    undarkened. This applies on flat ground, on slopes and to hovering vehicles, wherever
    claim 44 places the shadow. Claim 44 gives the original's comparison, which is not always
    at the same screen position. Suppression and depth tests still apply (claim 31; S-0102
    claim 18).
    - **Reuse.** Cache only the unmasked footprint. Exclude this object's current
      completed part image on every appearance, even when another object supplied the
      cached footprint. No old-pose cutouts persist.
    - Backing: `measurable` (MS-27; fresh exclusions of a vehicle at height 0 above the
      surface are oracle-testable, corrected reuse is a lead-confirmed departure tested
      against this spec, and the comparison above the surface is claim 44's correction,
      tested by MS-34 against this spec)
    - Confidence: high

40. **Aircraft shadow anchor (Q-0799).** For an aircraft drawn as a voxel model,
    let *P* be the screen projection of its horizontal position at terrain height
    *H*, using S-0101 claims 1 and 3. Its shadow draw point is *P*, except that a
    qualifying high bridge raises that point by exactly 60 screen pixels. Claim 31's
    footprint shift and scale and F10's copy placement still apply around this anchor.
    The anchor is not the body's lifted point or the centre of its visible silhouette.
    The deck mark used here is S-0501 claim 43's aircraft deck mark, not a render
    layer or occupancy inference: landing can set it, ordinary climbing clears it,
    and horizontal flight or hovering alone leaves its previous value unchanged.
    - **Bridge qualification.** The current cell must be a high-bridge cell
      (S-0211 claim 15), and so must its neighbour one cell towards *y − 1* for a
      bridge whose axis is along cell x, or towards *x − 1* for an axis along cell y.
      The bridge axis is the one used by S-0212 claim 3's bridge links. In addition,
      *a ≥ 416* leptons when the aircraft is not marked on the deck, or *a ≥ 0*
      when it is marked on the deck. Otherwise use terrain. Thus a flight above a
      continuous deck can cast its shadow at deck height, but not every marked edge
      cell qualifies. A destroyed deck that has lost its high-bridge marking cannot
      qualify. No building roof or helipad surface is substituted for terrain.
    - **Height dependence.** Away from a change in the bridge qualification,
      changing world height alone does not move the anchor. It changes the separation
      from the body, not the shadow's surface point. The terrain height is sampled at
      the exact horizontal position, including its slope, not just the cell's whole
      height level. There is no clamp to the aircraft's own world height.
    - **Landing and pads.** The same rule applies during take-off, hovering above a
      pad, landing and rest, including *a = 0* and *a < 0*. Negative height does not
      raise the terrain anchor with the body or itself suppress the shadow. A
      non-deck aircraft below deck height uses terrain even over a high-bridge cell.
      A deck-marked aircraft at *a = 0* can still qualify for the deck.
    - **Locomotors.** Our engine uses this anchor for every aircraft locomotor,
      whenever it supplies an otherwise eligible voxel shadow (claim 31); locomotor
      eligibility and shadow shape are not made universal by this placement rule.
      The original uses the stated anchor for the flying locomotor, but for the
      other locomotors it places the anchor Λ(*a*) rows below *P* even over a
      qualifying deck, Λ being the original's own drawing lift (claim 44, *Which
      lift*); this height-dependent presentation glitch is corrected here without
      a switch.
    All height subtractions and threshold tests above are exact signed integer
    operations; 60 is an integer pixel translation, with no further rounding.
    S-0101 claim 3 owns the projection's height rounding. S-0102 claim 18 still owns
    the shadow's render layer, scene-depth test and own-body overlap, not its anchor.
    - Backing: `measurable` (MS-30; the non-flying placement correction is engine
      conformance, not a prediction of identical original pixels)
    - Confidence: high

### Aircraft body attitude

41. **Rocket pitch is drawn throughout flight.** A rocket missile's body is drawn
    with *V · Rz(θ_body) · Ry(−p)*, before its section HVA transforms (claim 13).
    The heading angle is claim 3's 32-step body heading. Pitch is about the body's
    local lateral axis, so positive *p* raises the nose in every heading. There is
    no slope tilt and no phase-specific drawing override: pause, tilt, raise,
    ascent, cruise and descent all show the current pitch owned by S-0506.
    Cruise is level only when that pitch is zero, not merely because the phase is
    named cruise. Height placement is independent of this rotation.
    - **Pitch resolution.** The current single-precision *p* is negated and used
      directly as the y-rotation angle. There is no rounding to 32 pitch facings,
      nor a fixed set of initial, final and level pictures. F1's table
      trigonometry quantises *−p* to its 8192 sine-table indices, with the exact
      signed rounding and end-of-turn rule given there. Thus there are at most
      8192 rotation-table choices per heading, not necessarily 8192 visibly
      distinct images: the model, rasterisation and pitches actually reached
      can give fewer. Single-precision matrix stores follow F0.
    - **Reuse classes.** Zero pitch is eligible for the level class. For nonzero
      pitch, equality with *pᵢ* gives the initial class; otherwise equality with
      *p𝒇* gives the final class. All other pitches require fresh draws, even
      when their table angle or rendered pixels equal an earlier pitch's.
      Define *pᵢ = RZ₅₃(Pᵢ × c)* and
      *p𝒇 = RZ₂₄(RZ₅₃(P𝒇 × c))*, where *Pᵢ*, *P𝒇* are the imported
      single-precision initial/final pitch fractions of the selected rocket set,
      widened exactly, and *c = 1.5707963267948966* radians per quarter turn
      (binary64). RZ₅₃ and RZ₂₄ mean round toward zero to double and single
      precision, respectively. Compare the current single pitch widened exactly
      with those values, with no tolerance. In particular, the initial comparison
      does not first narrow its product to single precision, so a pitch assigned
      from an initial fraction need not qualify for the initial class. Coincident
      initial and final values favour the initial class, except zero remains
      level. These classes are not pitch quantisation: a fresh draw still uses
      the current *p*.
    - **Shadow.** When eligible under claim 31, the rocket's shadow has
      *S = R_s · Rz(θ_body)*, where *s* is the current cell's slope shape
      (S-0106 claim 11), regardless of pitch or flight phase. Its reuse key is
      slope shape and 32-step body heading, with no pitch class. Intermediate
      rocket pitch alone does not force a fresh shadow. Claim 40 owns its anchor,
      S-0102 claim 18 owns scene-depth and own-body overlap.
    - Backing: `measurable` (MS-31, including its reuse probe). The precise
      table entries that yield identical captured pixels remain `unmeasured`
      for F1's stated reason, not an unknown pitch-step count.
    - Confidence: high

42. **Flying bodies pitch and bank.** A non-crashing aircraft with the flying
    locomotor uses *D = Rz(θ_flight) · Ry(α) · Rx(β)* and is drawn with *V · D*,
    before the HVA. Here *θ_flight* uses the 32-step flight drawing heading,
    not the other body-facing direction. There is no slope tilt.
    - At strictly positive height above the current surface (the signed height
      used in claim 40), if the current speed fraction is strictly greater than
      the imported drawing-pitch threshold, *α* is the imported drawing-pitch
      angle. Otherwise *α = 0*. Compare in double precision, with no rounding
      to a facing. Positive imported angles lower the nose.
    - Under that same strict height and speed condition, *β* is the imported
      bank angle during an unfinished clockwise aiming-facing turn, its negative
      during an unfinished counterclockwise turn, and zero otherwise. The turn
      direction is S-0206's signed shorter turn, positive facing change meaning
      clockwise. Bank is a discrete state, not proportional to heading error
      or turn progress. Pitch does not require a turn; bank does require the
      speed condition, even when the pitch angle is zero. In claim 26's key,
      pitch is present only when these height/speed tests pass and the configured
      pitch angle is nonzero; a configured zero angle has the no-pitch state.
    - At zero or negative height, or speed fraction equal to or below the
      threshold, both angles are zero. Take-off and landing do not separately
      force a nose angle: the same tests apply. This claim does not replace
      S-0312's crash attitude rules; crashing flying bodies at strictly
      positive signed height are not reusable.
    - The angle rotations use F1 and F0, after narrowing the configured double
      angles to single precision toward zero. The shadow uses the current
      slope transform and 32-step flight drawing heading (S-0106 claim 11),
      never these pitch or bank angles; the dropship flat-shadow exception
      remains. Its key has slope shape and that heading, not body bank or pitch.
    - Backing: `measurable` (MS-32)
    - Confidence: high

### Multi-turret selection

43. **Selected turret index (Q-1760).** This claim says which index claim 12 draws for a vehicle
    of a multi-turret type (Terms). State table S3 summarises it.
    - **Taken from the turret table.** Each time the vehicle's weapon number is set (S-0308
      claims 5 and 6), its selected turret index becomes the turret table entry for the resulting
      weapon number, in the same tick. This includes a request outside 0 … 17, which S-0308
      claim 5 turns into weapon number 0.
      The model shown is therefore not the one numbered like the weapon number. It is the one
      the table maps that number to, and no turret at all when the table holds *none* for it.
      Only one fixed type name ever has a filled table (Import mapping), so a vehicle of any
      other multi-turret type that is not a charge-turret type shows no turret in any state,
      unless it has a sprite body (*Any body*, below; `quirks.turret_table_fixed_type_only`).
    - **Not a weapon choice.** The index decides what is drawn, not which weapon is used.
      Firing, target and threat choice, the bunker test of S-0606 claim 1 and the armed tests of
      S-0209 all ask the weapon S-0308 chooses, which is that of the vehicle's weapon number
      (S-0308 claim 5) unless S-0308 claim 9's override for a deployed vehicle applies, whatever
      the index holds. So a multi-turret vehicle whose index is *none*
      for its whole life still fires, enters a vehicle bunker and counts as armed through the
      weapon S-0308 chooses: outside S-0308 claim 9, that of its weapon number, which is 0 from
      creation unless S-0308 claims 5 and 6 set it otherwise.
    - **When the weapon number is set.** A vehicle of a gunner type has its weapon number set to 0
      when it is created, whether it is built, placed by the map or made in any other way, and
      again at every gunner hand-over of S-0308 claim 6. Its model changes in the same tick as its
      weapon number. A vehicle whose type is neither a gunner type nor a
      charge-turret type never has its weapon number set, so its index stays *none* for its whole
      life.
    - **Charge-turret types.** Their index never comes from the turret table, because the type
      ignores every request to change its weapon number (S-0308 claim 5). A charge-turret type
      here is a multi-turret type, so its turret count *N* is above 0 and it has no
      rotating-barrel weapon stages (Terms): this is exactly the scope of S-0301 claim 16, which
      covers only such types. In each of the vehicle's
      turns its index becomes S-0301 claim 16's turret number *c*. Right after a shot with a
      positive delay *D* it is *N* − 1. It steps down through the turret models while the rearm
      countdown runs, and it is 0 once the countdown is over and whenever *D* ≤ 0, which is the
      case from creation until the first shot. Ammunition has no direct effect: with a full,
      partly spent or empty magazine (S-0301 claims 10 and 11), and in the turn a round comes
      back, the index is what the countdown gives.
    - **From creation (fixed, no switch).** Our engine gives a charge-turret vehicle index 0
      from its creation, so it shows turret 0 in every drawn frame until its first shot. In the
      original, a vehicle released by a person's place order (S-0404 claim 7) is shown in one
      drawn frame with neither a turret nor a barrel, and with turret 0 from its first turn on; a
      vehicle placed by the map is shown the same way in the game's first drawn frame, if that
      frame shows it. It is a drawing artefact with no gameplay impact (the index selects no
      weapon, above), so it gets no switch.
    - **Nothing else changes it.** The index is kept through veterancy changes, through being
      carried in a transport and unloaded from it, and through attacking, apart from a
      charge-turret type's countdown.
    - **Any body.** Everything in this claim holds for a sprite-body vehicle too: its index is set
      by the same rules and selects no weapon. Only what is drawn differs (claim 12, *Sprite
      body*): the drawn frame without a turret before a charge-turret vehicle's first turn, above,
      does not occur on a sprite body. On a sprite body the index reaches what is drawn only
      through the barrel's image reuse (claim 26, whose barrel key holds it), so that its index
      follows these rules is `unmeasured`: no scenario here tells it apart. MS-33 part k shows only
      that nothing drawn follows it. Rated medium, from the same rules being applied whatever the
      body.
    - Backing: `measurable` (MS-33, parts a to d, f, h, i, j, k and l; part i for the index
      selecting no weapon in firing and the bunker test; part j for the original's drawn frame
      without a turret before a charge-turret vehicle's first turn; part k for what a sprite body
      draws at every index; part l for ammunition having no direct effect). `unmeasured` for
      *none* for life on a type with neither flag and a filled turret table, and for the armed
      tests of S-0209, which no part sets up.
    - Confidence, table lookup: high (the model chosen from the turret table at every setting
      of the index). This rating does not cover the next line.
    - Confidence, other parts: medium (only the fixed type name having a filled table, which
      MS-33 part d checks against one other name only; the index
      selecting no weapon, until MS-33 part i runs, and for the armed tests of S-0209, which no
      part sets up; *none* for life without a gunner or
      charge-turret type; a sprite body's index following these rules (*Any body*); the
      charge-turret rule, which S-0301 claim 16 rates medium, and ammunition having no direct
      effect on it, until MS-33 part l runs; creation by
      every route, keeping the index through veterancy and transport; the original's drawn
      frame without a turret before the first turn, until MS-33 part j runs)

### Vehicle shadow anchor

44. **Vehicle shadow anchor (Q-2672).** For a vehicle drawn as a voxel model, let *B* be its body's
    draw point (Terms) and *a* its signed integer height above the surface, in leptons (glossary):
    its world height minus the terrain surface height at its exact horizontal position (S-0106 claim
    2), minus a further 416 leptons while it is on a bridge deck. Its shadow draw point is *B* moved
    down by the lift of *a*, in rows, using the same lift that lifts *B* by the vehicle's whole
    world height. *B* is lifted by its world height and the anchor is lowered by its height above
    the surface, so, when its locomotor applies no draw shift to *B* (Terms, claim 25), the shadow
    lies at the projection of its horizontal position at the height of its surface, to within the
    rounding of the lifts, not at the body's lifted point (except at a height whose lift is 0). A
    hovering vehicle, at rest or moving, on land or on water, casts its shadow on the surface under
    it, and the shadow shows wherever the vehicle's own image does not cover it. Claim 31's shift
    and scale and F10's copy placement apply around this anchor, and claim 39 excludes the vehicle's
    own image at the same screen position.
    - **Which lift.** Our engine uses S-0101 claim 3's signed lift *L*, following that claim's
      lead-confirmed departure, for both *B* and the anchor. On a surface at a whole height level
      the anchor is then exactly the surface projection (S-0101 F2 gives 15 pixels per level), and
      elsewhere, on a slope, within one row of it. The original uses its own drawing lift Λ
      (S-0313's term) for both, so its anchor is *B* moved down by Λ(*a*) rows. On a surface at
      level 0 that is exactly the ground projection whatever Λ is, for a locomotor that applies no
      draw shift to *B*, since *B*'s lift and the anchor's lowering are then both Λ(*a*). Elsewhere
      its distance from the surface projection depends on Λ: under S-0313 claim 11b's hypothesis
      that Λ is S-0409 F2's lift, it is within one row on a surface at whole levels 1 to 13 for
      *a* ≥ 0 and not bounded here elsewhere, as on higher levels and slopes.
    - **Surface.** Over water the surface is the water cell's terrain surface: there is no
      separate water height. On a bridge deck it is the deck. A vehicle under a high bridge,
      not on its deck, uses the terrain. No building roof is substituted.
    - **Height and rocking.** Changing *a* alone, as a hovering vehicle's bob does (S-0503
      claim 8), moves the body, while the anchor stays within the rounding band above: exactly
      fixed in our engine on a surface at a whole level, and in the original on level 0. Rocking
      (S-0106) does not enter the anchor or the footprint's outline (claim 30): it changes only
      the body's own image, and so which shadow pixels claim 39 excludes. The ramp tilt of the
      cell under the vehicle enters the footprint (claim 30), not the anchor.
    - **Other heights.** The rule is the same at *a* = 0, where the anchor is *B*, at *a* < 0, where
      the signed lift places the anchor at or above *B*, and on a slope's upper edge, with whatever
      height above the surface the vehicle has there.
    - **Lifted vehicle.** While a carrier aircraft lifts the vehicle, the anchor is raised by
      14 rows. When in addition the vehicle is not on a bridge deck, *a ≥ 416* and claim 40's
      bridge qualification holds for its current cell and that cell's neighbour, the anchor is
      raised by a further 60 rows. When the lifted state begins and ends is not given here: it
      belongs to the transport specs.
    - **Locomotors.** The original uses this anchor for every vehicle locomotor except the flying
      one: a vehicle with the flying locomotor gets *B* itself. A vehicle whose type has the
      hunter-seeker property (Import mapping) gets the projection of its horizontal position at
      terrain height, with the original's lift Λ, moved down by 2·Λ(*a*) rows with a non-flying
      locomotor. Our engine uses the rule above for the flying locomotor too, and claim 40's *P*
      itself for a hunter-seeker type, which is at terrain height under claim 40's bridge
      qualification, so a hunter-seeker type on a deck below that qualification gets the terrain
      point. Both are presentation glitches, corrected here without a switch.
    - **Own-image comparison in the original.** The original compares each shadow pixel with the
      own-image pixel at the same position relative to *B* moved down by Λ(*a*) rows, plus one row
      while *a* > 0, whatever anchor it actually uses. So for a vehicle above its surface it tests
      the own-image pixel one row below each shadow pixel, and for a lifted vehicle, a hunter-seeker
      type or a vehicle with the flying locomotor it is also off by the raise or by the anchor's
      difference (for the flying locomotor, whose anchor is *B*, by Λ(*a*) rows plus that one row).
      Our engine compares at the same screen position (claim 39), correcting this presentation
      glitch without a switch. All height subtractions above are exact signed integer operations,
      and 14 and 60 are integer pixel translations with no further rounding. S-0101 claim 3 owns the
      rounding of *L*, and S-0102 claim 18 owns the shadow's render layer and scene-depth test. A
      sinking vehicle draws no shadow at all (S-0504 claim 16).
    - Backing: `measurable` (MS-34 parts a, b and d for the original's anchor on a level-0 surface,
      for the surface choice on and under a deck, and through the bob and rocking; MS-34 part c for
      the original's own-image comparison). Our engine's lift, the corrections of the
      flying-locomotor and hunter-seeker anchors and of the own-image comparison are engine
      conformance, tested by MS-34 part e against this spec. `unmeasured` for *a* < 0 and for a
      lifted vehicle: no situation this spec covers draws a vehicle below its surface, and a lifted
      vehicle needs a carrier aircraft lifting it, whose start and end are the transport specs' to
      set up, not a shadow scenario's. `unmeasured` for the original's flying-locomotor and
      hunter-seeker anchors: our engine corrects both, so no result of ours depends on them.
      `unmeasured` for the anchor's distance below *B* off a level-0 surface: MS-34 records it, but
      the original's lift there is known only under S-0313 claim 11b's hypothesis, so no run can
      tell a departure of the anchor from one of the lift.
    - Confidence: high (the anchor of a vehicle with a non-flying locomotor and without the
      hunter-seeker property, not lifted, at *a* ≥ 0, on a level-0 surface, as *B* moved down by the
      lift of *a* with *B*'s own lift and so, for a locomotor that applies no draw shift, the exact
      ground projection, on land, on water and under a bridge over level-0 ground; the surface
      choice on and under a bridge deck; rocking and the bob not entering it; our engine's forms:
      S-0101's lift, the rule above for the flying locomotor, *P* for a hunter-seeker type and the
      comparison at the same screen position), medium (whether the hovering locomotor applies a draw
      shift to *B* (S-0313 Open questions), the anchor's distance below *B* off a level-0 surface,
      which MS-34 records but which only S-0313 claim 11b's hypothesis would falsify, *a* < 0, the
      lifted vehicle's raises, the original's flying-locomotor and hunter-seeker anchors, and the
      original's own-image comparison, until MS-34 part c runs), low (the original's row bounds off
      a level-0 surface, which rest on S-0313 claim 11b's hypothesis, itself low)

### Aircraft parts

45. **Aircraft draw no turret or barrel (Q-2983).** An aircraft-kind object (glossary *aircraft*
    and *rocket missile*: an ordinary aircraft, a rocket missile or a spawned aircraft) is drawn
    with its body part only, in every attitude: level, pitched or banked in flight (claim 42),
    pitched as a rocket (claim 41) and tumbling while it crashes (S-0113 claim 20). Its type's
    turret flag, turret model and barrel model are never drawn, whatever their values. So claims
    4, 8, 10 and 11, and the turret and barrel keys of claim 26, never apply to such an object:
    there is no turret angle relative to the drawn heading, no turret or barrel key under pitch,
    bank or tumble, and no turret-first overlap test for it. The object's kind decides, not its
    locomotor. Whether the turret flag changes how an aircraft aims or fires is for the combat
    specs.
    - Backing: `measurable` (MS-36: part a for an ordinary aircraft with the flying locomotor and
      a rocket missile, part b for a spawned aircraft, part c for an aircraft-kind object with
      another locomotor)
    - Confidence: high (an ordinary aircraft with the flying locomotor and a rocket
      missile); medium (a spawned aircraft and an aircraft-kind object with another
      locomotor, until MS-36 parts b and c run)

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

**F8. Section overlap priorities (claims 23, 24; Q-0210, Q-0215).** Nearest depths are single-precision
view-space values under F0, in model units, compared exactly without tolerance or conversion
to integer comparison depths. Greater nearest depth means higher painter overlap priority.
The following relations fix both painter coverage and equal-comparison-depth coverage in
claim 24. Names stand for sections in the listed file order, not required storage positions.

| File order | Nearest-depth relation | Overlap priority, lowest to highest |
|---|---|---|
| A, B | A = B | A, B |
| A, B, C | A = B > C | C, B, A |
| A, B, C | A = B = C | A, B, C |
| A, B, C, D | A = B = C = D | A, B, C, D |
| A, B, C, D | A = B > C = D | C, D, A, B |
| A, B, C | A < B = C | A, B, C |
| A, B, C | A > B = C | B, C, A |
| A, B, C | A = C > B | B, A, C |
| A, B, C | A = C < B | A, C, B |
| A, B, C | A = B < C | A, B, C |
| A, B, C, D | A = B = C > D | D, B, C, A |
| A, B, C, D | A > B = C = D | B, C, D, A |
| A, B, C, D | A = C > B = D | B, D, C, A |
| A, B, C, D | A = D > B = C | B, C, A, D |

An entire part with any positive number of sections at one equal nearest depth has
file-order overlap priority: the later file section wins any pair overlap and the
last file section wins a common pixel. For every table row, the section farther right
in the priority column wins a pair overlap; the last section wins an all-section
common pixel. Outside depth mode this applies regardless of numeric voxel depth.
In depth mode it applies to coincident left pixels at equal numeric depth, provided
that depth exceeds the initial depth and no right-hand overwrite interferes (claim 24).
These are coverage requirements, not section drawing sequences. For finite nearest
depths, the priority depends only on the full file-order section sequence and its
exact nearest-depth comparison relations, not on the numerical gaps between depths, colours,
occupied voxel positions, or earlier part draws. It is not a pair-local rule: A = B alone
gives A, B, but adding a lower-depth C after them gives C, B, A. Consequently sections
without pixels at a particular overlap can still change which tied section wins there.
The section count and the number of intervening sections are already part of that full
sequence, not extra inputs.

**Question 7: one candidate relation for every count.** The following mathematical
priority relation supplies the table's coverage predictions without requiring any
particular sorting or drawing implementation. For a section sequence *S*, its strict
record-low subsequence *R* contains
the first section and every later section whose nearest depth is strictly smaller than
that of every preceding section. Equality never creates a record low. The lowest-priority
section of *S* is the last member of *R*. The relative priorities of the remaining
sections equal their priorities in the residual sequence: omit the first position of *S*,
and at the position of each later member of *R* substitute its predecessor in *R*.
All other positions retain their sections. The empty sequence has no priority; a
one-section sequence has only that section. This recursively defines a unique priority
permutation for every finite sequence. Depth comparisons use F0 single precision exactly.
Positions and ranks are integers, with no rounding. Only the resulting pixel coverage is
required, not residual sequences or a sequence of exchanges inside an implementation.

For example, A = B > C has record lows A,C, lowest priority C, and residual B,A,
hence priority C,B,A. All-equal sequences have just their first section as a record low
and therefore retain file-order priority. A stable file-order tie rule is not an
alternative description of these predictions: it disagrees at MS-25's A/B control
and MS-28's B/D, A/E and B/E discriminators in rows 2, 3 and 4 respectively.

Backing: `measurable` (MS-25/MS-28) for every listed pair and common winner, the
five-section all-equal part, the absent-C control and the four larger rows with
gap/absence repeats, in both modes. Common winners for this table are
B,A,C,D,B,C,A,C,B,C,A,A,A,D. Each pair must be probed separately, not inferred
from the common winner. The five-section all-equal prediction is the later file
section at every pair and E at the common pixel. A = B > C with C absent at A/B
predicts A, whereas A = B alone predicts B.
Backing: `unmeasured` for the universal arbitrary-count relation, the all-equal
case beyond supplied scenario counts and input sufficiency outside those patterns:
a finite scenario set cannot distinguish all possible extensions. MS-28's closing
sweep probes further cases. Confidence: high for the listed predictions, medium
for the general relation. Equal-depth winners do not establish a physical drawing
order, and no such order is required here.

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
| Turret table size | 18 | entries, one per weapon number 0 … 17 | Terms, claim 43; every entry *none* unless imported |
| Aircraft-like shadow scale | 1/2 | dimensionless (scale factor) | Claim 31 |
| Lifted-vehicle shadow raise | 14 | pixels, screen y (upward) | Claim 44 |
| Lifted-vehicle bridge raise | 60 | pixels, screen y (upward) | Claim 44, with claim 40's bridge qualification |
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

**S3. Selected turret index of a multi-turret vehicle (claim 43).** *T* is the type's turret
table, *w* the resulting weapon number, *c* S-0301 claim 16's turret number.

| Type | Event | Selected turret index |
|---|---|---|
| any, except our engine's charge-turret vehicles | the vehicle is created | *none* |
| gunner type, not charge-turret | creation completes (weapon number set to 0) | *T*[0] |
| not charge-turret | its weapon number is set (S-0308 claims 5, 6) | *T*[*w*], even when *w* is unchanged |
| charge-turret | each of its turns | *c* (our engine: *c* from creation) |
| any | veterancy change, transport carriage, anything else | unchanged |

For a vehicle with the voxel flag, claim 12 draws no turret or barrel for *none* or for an index
outside 0 … *N* − 1 (a sprite body: claim 12, *Sprite body*).

**Ordering consequences.**
- Within one part outside depth-buffered mode: claim 23's section overlap priority and
  claim 21's voxel priority decide which pixels show. A nearer voxel of a lower-priority
  section can be covered by a farther voxel of a higher-priority section, because there is
  no depth test between sections (`quirks.voxel_no_intersection_depth`). Claim 24 owns the
  depth-buffered exception.
- Between parts of one vehicle: nontransparent turret and barrel pixels cover body
  pixels at the same position; claim 11 selects the turret/barrel overlap winner.
  Parts never depth-test against each other.
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
  with "÷ 8, truncated" instead of the vehicle factor *ρ* (≈ ÷ 6.03), along the turret's
  own forward axis. Fixed: *d_b = δ × ρ*, rounded to single precision as for vehicles
  (F3), still along that axis. A building has no separate body facing to move along.
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
- **`quirks.barrel_frame_zero`** (default: on). Barrels drawn together with a voxel turret always
  use HVA frame 0 (claim 16; a building barrel without a turret is not affected, claim 37, and a
  sprite-body vehicle's barrel stays in frame 0 in both states, claim 12). Fixed: use the voxel
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
- **`quirks.voxel_section_sort_ties`** (default: on). Equal nearest depths have the
  overlap priorities of claim 23 (F8), not necessarily file-order priority. The listed
  patterns are measurable by MS-25/MS-28; the arbitrary-count relation is unmeasured.
  Fixed: among sections with equal nearest depth, the later file section has higher
  overlap priority. Distinct-depth priority is unchanged. For file order A, B, C
  with nearest depths 2, 2, 1 model units, B then wins an A/B overlap rather than A.
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
- **`quirks.turret_table_fixed_type_only`** (default: on). Claim 43. Only the fixed type name of
  Import mapping reads the turret table; every other type keeps an all-*none* table, so a
  multi-turret type of any other name that is not a charge-turret type shows no turret unless it
  has a sprite body. Fixed: every multi-turret type reads the same pairs of keys from its own
  section, by the same rules for the table (order, layering and the per-stem absent-key index),
  and entries no pair sets stay *none*. A pair with no weapon number still changes nothing in the
  table, and `quirks.turret_pair_no_weapon_sets_charge_flag` still applies to the fixed type name
  only, so no other type's charge-turret flag changes. Decided (delegated sign-off, 2026-10-25).
  Confidence medium; `measurable` (MS-33 part d) for the default.
- **Round-toward-zero mode** (switch `quirks.chop_rounding`, owned by S-0205 (claim 5); not defined here). Every floating-point operation and every single-precision store in
  this spec rounds toward zero (F0). With that switch off they round to nearest. Conversions from floating point to integers are still truncations toward zero, since
  each formula states them. The tabulated sine values of F1 do not change. In the table square
  root the stored input follows the switch, as every store does, while the final cut of the
  result stays a truncation. Example: F2's row z, x entry becomes 0.611342072 instead of 0.611342013.
- **`quirks.turret_pair_no_weapon_sets_charge_flag`** (default: on). Observable result: the
  fixed type name whose rules section fills the turret table (Import mapping) has charge-turret
  behaviour exactly when, in a rules layer that has that section, a pair with no weapon number
  (key absent or −1) gives a turret index that is not 0 and not a multiple of 256; such a pair
  overrides that layer's own charge-turret key, and an index of 0 or a non-zero multiple of 256
  in it turns the behaviour off. Such a pair leaves the turret table unchanged. The import
  accepts any integer for the index key, so a user's file can give a multiple of 256, and the
  importer must apply this rule to reproduce the original's behaviour. With several such pairs
  in one layer, the last one in the Import mapping order decides. The flag then governs S-0308
  claim 5 (weapon-number requests ignored) and claim 43 (turret from the rearm countdown), so
  this has gameplay effect. Fixed: such a pair changes nothing. Confidence medium; `measurable`
  (MS-33 part g) for one pair with turret index 1 and one with 0; `unmeasured` for indices that
  are other multiples of 256 and for several such pairs in one layer, which part g does not set
  up.

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
| `HunterSeeker` | Rules [type], vehicles | boolean | no | Hunter-seeker property (S-0307 owns it); here only the shadow anchor, the original's and ours (claim 44) |
| `ConsideredAircraft` | Rules [type] | boolean | yes for an aircraft type, no for every other type (S-0501 Import mapping; medium, as S-0501 rates it) | Half-size shadow (claim 31) |
| `DisableVoxelCache`, `DisableShadowCache` | Art [image] | boolean | no | Disable reuse of part images and of shadow images (claims 26, 31) |
| `FireAngle` | Rules [type] | integer, 1/256 turn (256 facing units), used modulo 256 | 8 | Barrel elevation target (claim 6) |
| `NoSpawnAlt` | Rules [type] | boolean | no | The alternative-body option: alternative body while none of the vehicle's spawned aircraft is docked (claim 12) |
| `TurretCount` | Rules [type] | integer | 0 | Turret count; above 0 makes a multi-turret type unless the type has rotating-barrel weapon stages (Terms, claim 12) |
| `IsGattling` | Rules [type] | boolean | no | Rotating-barrel weapon stages; here only its effect on the multi-turret rule (claim 12) |
| `Gunner` | Rules, vehicle type section | boolean | no | Gunner type (S-0308 owns it); here only the weapon-number setting at creation that selects turret table entry 0 (claim 43) |
| `IsChargeTurret` | Rules [type] | boolean | held value; default no (S-0308) | Charge-turret type (S-0308, S-0301 claim 16 own it); here only the countdown-driven turret index (claim 43). For the fixed type name below it can be overwritten (`quirks.turret_pair_no_weapon_sets_charge_flag`) |
| `IFVMode` | Rules [type], passenger types | integer | held value; default 0 (S-0308) | Mode index (S-0308 owns it); reaches the turret only through the weapon number (claim 43) |
| `NormalTurretIndex` and `NormalTurretWeapon`, and the same pair of keys with `Normal` replaced by each of the stems `Repair`, `MachineGun`, `Flak`, `Pistol`, `Sniper`, `Shock`, `Explode`, `BrainBlast`, `RadCannon`, `Chrono`, `TerroristExplode`, `Cow`, `Initiate`, `Virus`, `YuriPrime`, `Guardian`, in that order | Rules [type], read only when the type name is `FV` (letters in either case); every other type keeps an all-*none* turret table (`quirks.turret_table_fixed_type_only`) | integers: a turret index, −1 meaning *none*; a weapon number | index: 1 for the `Repair` stem, 2 for `MachineGun`, 3 for `Flak`, 0 for every other stem; weapon number: −1 | Turret table (Terms, claim 43); the pair rules follow the table |
| `UseTurretShadow` | Art [image], vehicles | boolean | no | Read, no effect (`quirks.no_turret_shadow`) |
| `Locomotor` | Rules [type] | class identifier in hyphen-separated hexadecimal groups of 8, 4, 4, 4 and 12 digits, enclosed in paired brace characters, letters in either case (S-0207 claim 14) | keeps value in force; engine default teleporting locomotor (S-0207 claim 14) | Selects body attitude and heading source (claims 41–42); original aircraft displacement differs between flying and other locomotors, while the corrected anchor is independent of this selection (claim 40) |
| `V3RocketPitchInitial`, `DMislPitchInitial`, `CMislPitchInitial`; `V3RocketPitchFinal`, `DMislPitchFinal`, `CMislPitchFinal` | Rules [General] | real quarter-turn fractions, single precision | selected set's defaults and layering as S-0506 Import mapping | current pitch is governed by S-0506 claims 14–18 and 24; the same selected set supplies claim 41's exact initial/final reuse comparisons; no separate drawing-pitch key |
| `PitchAngle` | Rules [type] | real degrees; −1 keeps the value in force; otherwise multiply by binary64 π/180 and store in double precision toward zero | 20 degrees (engine initial value 0.3490658503988659 radians) | flying body's drawing-pitch angle, claim 42 |
| Roll&#65;ngle | Rules [type] | real degrees, same conversion and −1 sentinel as the preceding row | 30 degrees (engine initial value 0.5235987755982988 radians) | flying body's bank angle, claim 42 |
| `PitchSpeed` | Rules [type] | real, double-precision speed fraction | 0.25 | strict speed threshold for pitch and bank, claim 42; not leptons per tick |
| No data key | rocket/flying drawing | fixed transform and reuse rules | not imported | claims 41–42: no pitch-image count, pitch-facing count or banking-progress setting |
| none: terrain and high-bridge marks | Map terrain and overlay data | terrain height and slope, high-bridge cells and bridge axis, as S-0106 claim 2, S-0211 claim 15 and S-0212 claim 3 specify | no deck where no intact high bridge marks the cell | Surface anchor and neighbour qualification in claim 40; no shadow-height INI key or helipad-height substitute |

**Turret table pairs.** The pairs of the turret table row above (claim 43) follow these rules.
- Each pair sets the entry for its weapon number to its turret index.
- Pairs apply in the order listed, so a later pair naming the same weapon number wins.
- The table is never cleared: entries set in one rules layer that has the section stay unless a
  pair of a later such layer sets the same entry.
- In every layer that has the section, a pair whose index key is absent uses its stem's default
  index, never an index an earlier layer gave that pair. So a later layer that sets only the
  weapon-number key of a pair sets that entry to the stem's default.
- Both orders (a later pair wins, an entry is kept across layers) and the per-layer default are
  `measurable` through MS-33 part h.
- The index defaults are constants of the original for an absent key, not values of the user's
  rules, and stating them here was decided at the delegated sign-off, 2026-10-25. Their numbers
  are `unmeasured`, medium, apart from the `Normal` stem's 0 that part h checks: every other
  part of MS-33 sets the index key of each pair it uses.
- A weapon number of −1, or an absent weapon-number key:
  `quirks.turret_pair_no_weapon_sets_charge_flag`.
- Any other weapon number outside 0 … 17: the original's result is *unspecified* (`unmeasured`,
  medium: no part of MS-33 supplies such a pair). Our engine ignores the pair and the importer
  warns (design).
- Turret indices outside −1 … *N* − 1 are kept and draw nothing (claim 12); the importer warns.

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
| Section sequence and bounds | VXL model | ordered sections and single-precision bounds | none; required model data | full-sequence nearest-depth relations and F8 priorities (claims 23, 24); Q-0215 question 7's candidate general relation uses no additional input or absent-key default; arbitrary-count backing remains unmeasured |
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
void rule. Use game speed 6 unless a scenario says otherwise, with no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). A screen recorder with read-only tick counts supplies captures; any
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
  - *Importer-warning conformance:* For named inputs in MS-11, MS-19, MS-20,
    MS-22, MS-23, MS-24 and MS-26, also import the files in our engine and record
    warnings, independently of oracle captures. Each condition in Import mapping's
    warning table must warn at the specified file/type/section/column, without
    changing that input's specified rendering. Include a vehicle with the
    alternative-body option but no spawned aircraft: it warns and shows its main
    body. Missing warnings, warnings naming the wrong input, or changed rendering
    falsify the importer rule. MS-29 separately tests oversized-warning deduplication.
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
    either winner. Facings 12 and 28 also straddle a 4-step boundary, but the winner
    is the same on both sides of it,
  - nontransparent turret and barrel pixels cover the body at shared positions,
    while transparent pixels leave the next part visible (claim 11),
  - the barrel rises as claim 6 says (default fire angle): turret directions first drawn in the
    first tick after placement show it level, and directions first drawn from the second tick on
    show it one step above level, including directions already cached at level,
  - the image matches claim 10's turret-origin pivot and barrel frame 0. Pivot and
    current-elevation reuse checks are lead-confirmed departures, engine conformance.
- *Falsified if:* the barrel pivots away from the turret origin, overlap differs,
  body pixels hide nontransparent turret/barrel pixels, transparent part pixels hide
  another part, or a second-tick-or-later appearance retains an earlier level pitch.

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
  for a few hundred ticks, then let it attack a target. Capture every drawn frame. For
  gattling-flagged variants, remove the target after spin-up and capture through spin-down.
  Use S-0609 claim 14's scenario G11 to distinguish the original building's double advance
  from the corrected engine result.
- *Observable:* the turret frame shown in each capture.
- *Expected:*
  - Before its first attack, each turret shows frame 0 in every capture, however far the
    vehicle drives.
  - Once attacking starts, each turret's frame changes. For types without the gattling flag,
    changes occur only through attacking. For gattling-flagged variants, continued animation
    during spin-down follows S-0609 claim 14, including its building correction.
- *Falsified if:* a turret shows frame 1 or 2 before its owner's first attack, the vehicle's
  turret frame changes with driving alone (a sign of the walk-animation counter), a turret
  cycles through its frames before any attack (a sign of an independently playing HVA),
  a gattling variant disagrees with S-0609 claim 14's applicable original or corrected result,
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
  - T1: A has higher F8 overlap priority than B, so those pixels show *X*'s colour.
  - T2: B has higher priority under F8 and under stable file-order priority alike,
    so those pixels show *Y*'s colour.
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
  - **Body overlap (claim 11).** Nontransparent turret and barrel pixels cover the
    body, while transparent part pixels do not hide another part.
  - **Overlap (claim 11).** The overlap shows *Y* (barrel wins) at 32-step turret facings
    5–19, and *X* (turret wins) at 21–31 and 0–3. Facings 4 and 20 straddle a 4-step
    boundary at which the winner changes and may show either.
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
  - body pixels hide nontransparent turret/barrel pixels, or transparent part pixels
    hide another part,
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
  lossless tick-labelled captures, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted).
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
  - MS-33's subject (the vehicle type with the fixed type name of Import mapping, with MS-33's
    test override, base-copy deletion and test models) as the multi-turret type, with a mode-1
    and a mode-2 passenger type as in MS-33, so that a gunner hand-over changes its selected
    turret index between two models that exist (claim 43),
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
  - Change the multi-turret vehicle's selected turret index through a gunner hand-over (claim 43),
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
  - the multi-turret vehicle shows the turret and barrel models of its selected turret index:
    turret 0 with barrel 0 for the mode-1 passenger and turret 1 with barrel 1 for the mode-2
    passenger (MS-33 part b),
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
  - **Two-pixel test.** Two sections, each containing just one voxel, with visibly
    distinct output colours X and Y. X's left pixel is immediately left of Y's left
    pixel on the same row; X has smaller positive numeric depth than Y. Both right
    pixels lie in the copy rectangle, and no other pixel can interfere. In variant 1,
    X's section has lower F8 overlap priority than Y's, verified from transformed
    bounds. In variant 2, change only unoccupied bounds so X's section has higher
    overlap priority, retaining both projected voxel positions and numeric depths.
- *Observable:* pixel crops at a fixed body facing (east, except where stated).
- *Expected:*
  - **Equal depths.** Only colour *Y* shows: the second file section has higher
    F8 overlap priority.
  - **Depth 0.** Isolated zero-depth voxels appear. Add negative-depth isolated
    markers and a coincident pair at depths 255 and 256, verified from F5: negative
    markers pass empty coordinates and depth 256 wins the pair.
  - **Two-pixel test.** X's left pixel shows X and Y's right pixel shows Y in both
    variants. Their shared pixel shows X in variant 1 despite Y's greater depth,
    and Y in variant 2. The bounds-only change therefore changes the shared winner.
  - **All three models.** The golden image matches claim 24 exactly. Testing both
    pixels or giving X the equal-depth overlap differs in at least one pixel.
- *Falsified if:* X shows in the equal-depth model, an isolated zero/negative marker
  disappears, depth 255 wins over 256, the shared pixel in the two-pixel test shows
  Y in variant 1 or X in variant 2, or verified tie/right-write coverage otherwise disagrees.

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
  - Before the first attack the marker shows frame 0 in every capture. For a type
    without the gattling flag it changes only through attacking; a gattling-flagged
    variant follows S-0609 claim 14 through spin-down, checked with its scenario G11.
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
  (h): the greater numeric depth of the nearer section wins the overlap even though
  its palette index is transparent (claim 24). The overlap is transparent, and the
  farther section shows only outside it.
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

Expected pair winners are listed below for independent checking in both modes.
No completed measurement is claimed. A verified contrary pair falsifies its F8
priority even when the common-pixel winner agrees. Repeat every mixed-depth row
with changed gaps and with other sections absent at its pair probe, retaining
those sections and bounds: each listed pair winner must remain unchanged.

| F8 row | Expected pair winners, both modes |
|---|---|
| 1 | AB:B |
| 2 | AB:A AC:A BC:B |
| 3 | AB:B AC:C BC:C |
| 4 | AB:B AC:C AD:D BC:C BD:D CD:D |
| 5 | AB:B AC:A AD:A BC:B BD:B CD:D |
| 6 | AB:B AC:C BC:C |
| 7 | AB:A AC:A BC:C |
| 8 | AB:A AC:C BC:C |
| 9 | AB:B AC:C BC:B |
| 10 | AB:B AC:C BC:C |
| 11 | AB:A AC:A AD:A BC:C BD:B CD:C |
| 12 | AB:A AC:A AD:A BC:C BD:D CD:D |
| 13 | AB:A AC:A AD:A BC:C BD:D CD:C |
| 14 | AB:A AC:A AD:D BC:C BD:D CD:D |

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
runs, or a reversal for ramps, falsify claim 39. A hovering vehicle is above its surface, so
in the original its poses follow claim 44's own-image comparison (MS-34 part c), and for them
this part is engine conformance for claim 39.

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
Rocking freshness itself is not distinguishable by this corrected-output test.

**MS-28, larger mixed-depth ties (claims 23, 24; Q-0215 question 7).** The four
rows and repeats below are proposed stock-oracle tests, not completed measurements.
The universal arbitrary-count relation remains unmeasured.
- *Setup:* Common setup, speed index 2, one human-owned parked stock turretless vehicle
  and one distant bystander computer player. Named test VXL/HVA files and an art database
  copy give the vehicle five or six sections, disable part caching and suppress shadows.
  All other data is stock; no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Use the pairwise and common
  marker construction of MS-25, with different nontransparent colours for every section.
  Bounds set exact finite nearest-depth equality groups independently of the markers.
  In depth mode all coincident left marker contributions at each probe have identical
  nonzero comparison depth. Different probes need not share a depth, provided
  right-hand writes cannot reach another probe.
- *Procedure and observable:* Restart the program for each model and mode. Capture one
  paused lossless image in normal mode and one with the depth option. Record every pair
  winner, not just the common-pixel winner. Repeat each model with unequal depth gaps
  changed while preserving the relations. Repeat with sections outside one tested pair
  retaining their bounds but contributing no pixel at that pair's probe.
- *Expected discriminators:* Names are in file order; each priority is lowest to
  highest. All pair winners must equal the later section in that priority in both
  modes, subject to the depth controls above. Common-pixel predictions are C,B,A,B.

  | File order | Nearest-depth relation | Overlap priority | Expected common-pixel winner, both modes |
  |---|---|---|---|
  | A, B, C, D, E | A = C > B = E > D | D, B, E, A, C | C |
  | A, B, C, D, E | B = D > A = E > C | C, A, E, D, B | B |
  | A, B, C, D, E, F | A = E > C = F > B = D | B, D, C, F, E, A | A |
  | A, B, C, D, E, F | B = E > A = C > D = F | D, F, A, C, E, B | B |

- *Expected pairs, both modes:* Row 1: AB:A AC:C AD:A AE:A BC:C BD:B BE:E CD:C
  CE:C DE:E. Row 2: AB:B AC:A AD:D AE:E BC:B BD:B BE:B CD:D CE:E DE:D.
  Row 3: AB:A AC:A AD:A AE:A AF:A BC:C BD:D BE:E BF:F CD:C CE:E CF:F DE:E
  DF:F EF:E. Row 4: AB:B AC:C AD:A AE:E AF:A BC:B BD:B BE:B BF:B CD:C CE:E
  CF:C DE:E DF:F EF:E.
- *Expected repeats:* Changed unequal depth gaps must preserve every pair and common
  winner in both modes. Also remove all probes except private section legends and
  tied-pair probes, retaining every section and its bounds. Expected tied-pair winners:
  row 1 AC:C BE:E, row 2 AE:E BD:B, row 3 AE:A BD:D CF:F, row 4 AC:C BE:B DF:F.
  These absent-at-probe repeats have no common probe. Match each probe's colour
  against its section's private legend in the same lossless capture so display-path
  colour conversion cannot be mistaken for a different winner. Legends and probes
  must be separated enough to exclude right-hand interference.
- *Falsifying results:* A verified contrary pair or common-pixel winner falsifies
  that row's relation; changed winners under either repeat falsify F8's input
  sufficiency for that configuration. Stable file-order priority instead predicts
  D,E,E for row 2 B/D, row 3 A/E and row 4 B/E, rather than B,A,B. Common-pixel
  captures must not replace pair probes. Incorrect equality or coincidence,
  indistinguishable legend colours, or interfering right writes make the affected
  run inconclusive; disagreement with the prediction alone does not.
- *Closing question 7:* Test F8's candidate relation on every equality/order pattern
  for five sections, then on six-section patterns containing three equality groups and
  on larger parts with intervening sections absent at probes. Use the setup, every-pair
  observable, both modes and gap/absence repeats above. Preserve captures with the
  predicted winner for each probe: any verified mismatch falsifies the candidate.
  Include near-equal but distinct finite depths under F0 to test exact comparison
  rather than a tolerance. These additional finite cases are `measurable`; the
  universal arbitrary-count relation and depth-gap independence beyond scenario
  coverage remain `unmeasured`, because no finite sweep distinguishes every possible
  extension. Agreement supports the relation but does not prove it for every count.
  No stable priority is substituted for the original default. Neither the closing
  sweep nor its near-equal-depth variants are completed measurements here.

**MS-29 Bounded part and shadow images (claim 22, F6; Q-0470).** Engine
conformance for lead-confirmed departures, not an oracle experiment.
- *Setup:* Use an engine rendering fixture with named sparse VXL/HVA models on a
  stock turreted vehicle type, neutral lighting and fixed flat-ground poses. Render
  part images and unmasked shadows separately, then together. Use the common skirmish
  houses, speed 6, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted); a direct rendering fixture
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

**MS-30 — Aircraft surface anchors (claim 40; Q-0799).**

- *Setup:* the common skirmish measurement setup, speed index 2, 800×600,
  no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), one human house and one bystander computer player outside
  the observed area. Use a tiny purpose-made map with flat ground at two levels,
  a slope, an intact high bridge in each axis with visible terrain beneath, and
  a stock landing pad. Observe a stock voxel plane and a stock rotor aircraft that
  cast shadows. Run three times per type and route. Use screen capture plus a
  read-only recorder of tick, horizontal position, world height and deck mark.
- *Procedure:* record cruise over each terrain level, the slope and each bridge,
  record descent, pad hovering, landing, rest and take-off. Include the outermost
  marked bridge row with no qualifying neighbour and the first interior row.
  Compare equal horizontal positions at different heights where available. Add
  S-0501 claim 43's landing and take-off transition test: on an interior bridge
  cell with the qualifying neighbour, fly over unmarked, land from above, rest,
  then take off. Use a human-owned landable aircraft with ordinary flying movement
  and, if needed, a named test-map override making it landable and not pad-bound.
  Repeat this route five times, with a non-bridge control. Never write the mark
  with the recorder or assume a pad can be built on a bridge. Record
  a collapse control only after the deck's marks are removed. Void a run if the
  bystander affects the observed area.
- *Observable:* recover the shadow anchor by matching the known claim 30/F10
  footprint, rather than equating a silhouette's centre with the object's point.
  Record its screen coordinates, terrain projection, bridge qualification and
  aircraft height for every selected drawn frame. During the transition route,
  also record per tick the aircraft deck mark, world height, terrain height,
  render layer, both occupant lists and cell-layer occupancy flags separately.
  Scene-depth occlusion may hide
  footprint pixels; a frame without enough visible pixels is inconclusive.
- *Expected:* the anchor is the terrain projection or exactly 60 rows above it
  according to claim 40, never the body's lifted point. Over ordinary terrain it
  stays fixed at equal horizontal positions as height changes. The bridge threshold
  includes equality; its neighbour test rejects the unpaired edge row. A pad adds
  no new surface. A missing deck returns to terrain. The transition route starts
  unmarked on flyover, acquires the mark on settling at deck height, rests with
  *a = 0* and ground rendering, and clears the mark on the first ordinary climbing
  tick. At that interior cell the shadow remains at the deck anchor across the
  landing and take-off transitions whenever claim 40's height test still qualifies,
  regardless of render-layer or occupant-list changes.
- *Named test-data extensions:* when stock routes cannot sample exact thresholds, use a test map and
  named type overrides for landing/cruise positions, and a named placement fixture for signed
  heights, without altering the executable. Sample *a = −1, 0, 1* on terrain and *415, 416, 417*
  over the bridge, with and without the deck mark where legal. A state that normal placement cannot
  produce is instead an engine-conformance fixture, not an oracle measurement. Replay aircraft with
  the jumpjet and rocket locomotors and each further supported locomotor using named test types,
  comparing only appearances that actually cast voxel shadows. The corrected-engine expectation is
  always claim 40; record the original's non-flying anchor, Λ(*a*) rows below *P* (claim 40),
  separately, not as its target, with *a* from the recorder.
- *Falsified if:* an eligible flying-locomotor shadow's recovered anchor differs from the surface
  rule, moves with height away from a bridge threshold, uses a pad roof, clamps to the body, or
  selects a deck without the specified neighbour. A transition contrary to S-0501 claim 43 falsifies
  that transition rule; a shadow anchor contrary to claim 40 using the recorded mark falsifies this
  anchor rule. The engine-conformance extension fails if a non-flying aircraft uses a different
  anchor under equal surface inputs. A reproducible original non-flying anchor not straight below
  *P* falsifies that recorded original rule, not the corrected surface rule. Its distance below *P*
  is recorded against S-0409 F2's lift of *a*: a departure bears on S-0313 claim 11b's hypothesis
  for Λ, not on claim 40. Negative-height and artificial-locomotor variants that cannot be created
  in the original remain conformance tests only.

**MS-31 — Rocket body pitch, shadow and reuse (claims 26, 30, 41; Q-0950).**
Use the common skirmish setup, stock data, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), speed index 2,
800×600 captures and one bystander computer player outside the observed area.
On a tiny flat test map, launch each stock rocket kind toward a distant stationary
legal target, with eight launch headings, then repeat over a slope. Capture every
tick from placement to detonation, recording position, body heading, phase and
single-precision pitch beside each image using a proposed read-only recorder.
Repeat each launch twice in one game, and once after restarting the game.
Record the body silhouette separately from shadow pixels and reconstruct the
unmasked footprint using claim 40's anchor and S-0102 claim 18's occlusion rules.
Expected: the model matches *V · Rz(θ_body) · Ry(−p)* in every phase, including
intermediate tilt and descent pitches. It is not limited to 32 pitches. On equal
heading and ground shape, the unmasked shadow stays unchanged as pitch changes.
A level body at a nonzero resolvable pitch, a 32-step pitch snap rather than F1,
a pitch sign/order mismatch, or a pitched unmasked shadow falsifies.

For reuse, propose extending the read-only recorder to report completed-image
reuse versus a fresh body/shadow appearance; it must not alter game state or
rendering. Compare repeated level, initial, final and intermediate poses at equal
heading and HVA frame, computing both endpoint comparisons exactly as claim 41.
Expected: only the three eligible pitch classes can reuse a body; all other
pitches stay fresh. Shadows can reuse across all pitches at the same shape and
heading, independently of body freshness. A contrary recorded reuse decision
falsifies. Without this proposed observation, identical pixels cannot establish
reuse or non-retention; screenshots alone test attitude and footprint, not the
exact equality-class contract. Named test-data endpoint fractions may supplement
the stock runs to distinguish double initial comparison from single final
comparison; keep these runs separate from the stock observations. No such
recorder results or completed measurements are claimed here.

**MS-32 — Flying pitch and banking (claims 26, 30, 42; Q-0950).**
Same setup and capture settings as MS-31, with a stock flying-locomotor aircraft
on a tiny flat map and a landing pad, plus one bystander kept out. Record take-off,
straight flight, clockwise and counterclockwise unfinished turns, turn completion,
slowdown and landing once at each of eight headings. Record current speed fraction,
aiming facing, body facing, signed height and turn direction alongside each
capture. Expected: level at nonpositive height or speed at/below the threshold,
otherwise constant configured pitch, plus positive bank on clockwise turns,
negative bank on counterclockwise turns, and none after turn completion. Compare
against claim 42's composed transform, not pitch in screen space. Repeat over a
slope: body attitude is unchanged; the unpitched shadow follows the ground shape
and flight heading. A sign/order error, use of the other facing, gradual bank
proportional to turn progress, or a separate take-off/landing pitch falsifies.
A supplemental named map-only test-data run gives distinguishable drawing angles
and arranges speed samples equal to, immediately below and immediately above the
threshold; −1 angle overrides must retain earlier angles. Use MS-31's proposed
read-only reuse observation to distinguish the heading/pitch/bank body classes
and the shadow's pitch-independent class. Repeat the same rounded heading and
complete HVA frame in each pitch/bank state, then return to a previously drawn
state. Also repeat with zero configured pitch and, separately, zero configured
bank: a zero angle must not create its corresponding active reuse state. Hold
the aiming-facing bucket fixed while changing only the body-facing bucket to
distinguish the heading source for both body and shadow. If normal flight cannot
produce that separation, the heading-source comparison remains `unmeasured`
until a legal route does so; a forced-pose engine fixture alone is not oracle
backing. In a separate run, destroy the airborne aircraft with ordinary weapon
fire and capture crashing appearances at strictly positive signed height:
each must be fresh.

A recorded reuse across different applicable heading, HVA, pitch or bank classes,
no reuse on return to an eligible retained class without invalidation, a bank
when its angle is zero, pitch at nonpositive height or speed at/below threshold,
a changed unmasked shadow with pitch/bank alone, or reuse of a positive-height
crashing body falsifies the corresponding rule. A −1 override changing an
earlier angle also falsifies the import rule. Missing reuse records leave the
exact reuse contract inconclusive: pixel matches alone do not settle it.

**MS-33 Selected turret index of multi-turret vehicles (claims 11, 12, 16, 43).**
- *Setup:*
  - A tiny flat test map with a war factory, an infantry barracks, a transport able to
    carry vehicles and a vehicle bunker (S-0606), all for the tester's house, and one
    stationary enemy target in range.
  - **Subject.** A test map override section for the vehicle type with the fixed type name of
    Import mapping. It sets the turret flag, a turret count of 3, the gunner flag, a weapon
    count of 18 with a weapon in every slot, and every one of the 17 turret table pairs with an
    explicit weapon number, in the Import mapping order of the stems: `Normal` maps weapon 0 to
    turret 2, `Repair` weapon 1 to turret 0, `MachineGun` weapon 2 to turret 1, `Flak` weapon 3
    to turret 5 (at or above the count, below 18), and the other 13 pairs map weapon numbers 4
    to 16, in order, to turret 0. Weapon number 17 gets no entry here.
  - **Base copy.** In the base rules copy (S-0004), every turret table key of that type's
    section is deleted, so no earlier rules layer gives any entry and every expected value
    below follows from the test override alone.
  - Test models under the names S-0007 gives: a body whose HVA has 4 frames, turret models 0, 1
    and 2 that look clearly different from each other and whose HVAs have 4 clearly different
    frames, and barrel models 0 and 1 only.
  - **Control.** A second test vehicle type with another name and the same fields, keys and
    models, added to the vehicle list of the base rules copy (S-0004) as in MS-11.
  - **Charge subject.** A third test vehicle type, added in the same way, with the turret flag, the charge-turret flag,
    a turret count of 3, the same kind of turret models and one weapon whose rate-of-fire field
    is long enough to give many ticks per turret model.
  - **Magazine subject (part l only).** A test copy of the charge subject, added in the same
    way, whose weapon launches a projectile and whose section gives it a maximum ammunition of 2
    and leaves its initial-ammunition field out, so it starts full (S-0301 claim 10). It lacks
    the manual-reload flag, and its reload fields give a reload delay many times its weapon's
    rate-of-fire field, at every ammunition count (S-0301 claim 11).
  - **Sprite runs (part k only).** Only the fixed type name has a filled turret table (Import
    mapping), so the sprite case uses the subject itself, in runs of their own. A test art file
    clears the voxel flag in the art section of the subject's image (S-0007 claim 4). Its rules override stays as above, so its turret table, turret flag, gunner flag,
    turret count and weapons are unchanged: the table is read by type name whatever the body.
    Its body and turret image frames come from a test sprite file under its image name, with
    turret frames clearly different from the test models. Test models with the single turret
    and single barrel names that S-0007 claim 19 gives are added, clearly different from each
    other and from the numbered models, which stay supplied. A second sprite run is the same
    except that the single barrel model is withheld.
  - **Sprite-barrel runs (part k only).** Further runs of the first sprite run's subject, each
    with the changes named, its test override setting all four depth fudges to 0 as in S-0102
    D33 and its fire angle to a test value of 0, so that its barrel stands level (claim 6):
    - *depth runs:* the map also holds a wall segment, a building, a tree, a voxel vehicle of the
      tester's house and a bridge whose deck the subject can drive onto. A depth control, a further
      test vehicle type with the subject's sprite file and fields but without the turret flag, is
      added as the control is. The test override clears the subject's too-big-for-a-bridge flag,
      which the depth control shares, so that no split of a tall image (claim 12, *Depth*) arises
      for either. A further depth run, the *depth run without a barrel*, is the same with the single
      barrel model withheld, as in the second sprite run, and has only the drive past the wall
      segment at height 0. The test sprite file gives every body frame a stored rectangle whose last
      row lies below the last row of every turret frame's stored rectangle and below the barrel's
      copy rectangle (claim 25, F6) at the held turret facing, so that the subject's composed image
      and the depth control's body frame share their last row (claim 12, *Depth*). The *compared
      pixels* are the body pixels that neither the barrel nor the sprite turret covers at the held
      facing and that lie over opaque pixels of the wall segment at every distance from −1 to 4 rows
      (Procedure), in at least three consecutive rows, each holding at least one such pixel.
    - *pose runs:* the subject's turret displacement field set to the test values 0, 200, −100
      and −12 leptons, one run each.
    - *frame run:* the single barrel model's HVA has 4 clearly different frames.
    - *invisible runs:* the projectile of the subject's slot-0 weapon has the invisible flag, with
      the fire angle left out (absent default 8) in one run and set to a test value of 20 in a
      second; a third run, the visible control, sets 20 without the invisible flag. For these
      runs the base rules copy (S-0004) also deletes the fire-angle key of the subject type's
      section, besides its turret table keys, so that in the first run no rules layer gives a
      fire angle and the absent default is the value imported.
  - Passenger types: override sections giving five infantry types the mode indices 0, 1, 2, 3
    and 17, and a sixth one mode index 20.
- *Procedure and observable:* capture the turret and barrel area of every drawn frame, with the
  tick number, at a held body and turret facing unless stated.
  - *Part a, creation:* build one subject and one control, and place one subject on the map.
  - *Part b, hand-over:* load the mode-1 passenger into an empty subject, then a mode-2 passenger
    as second passenger, then remove them with S-0308 scenario W4's load and unload procedure,
    which isolates the first-passenger and last-exit hand-overs. Repeat with the passengers of modes 2, 3, 17 and 20
    entering first. Record the tick of each entry and exit.
  - *Part c, kept:* with the mode-1 passenger aboard, promote the subject by kills, then carry
    it in the transport and unload it.
  - *Part d, control:* repeat part b with the control.
  - *Part e, frame:* with the mode-1 passenger aboard, drive the subject straight through at
    least 8 walk-counter steps, recording the body frame *f_b* (from the four distinct body
    frames) in every drawn frame. Then let it attack the target while standing still with its
    body in frame 0; if it stops in another body frame, move it and stop again until it shows
    frame 0.
  - *Part f, charge:* let the charge subject fire one shot and capture every tick until its
    rearm countdown is over.
  - *Part g, absent weapon key:* repeat parts a and b with a subject whose override omits only
    the weapon-number key of the `MachineGun` pair (turret index 1, not 0). In a mirror run, the
    override sets the charge-turret flag on and omits only the weapon-number key of the `Repair`
    pair (turret index 0).
  - *Part h, layers and repeated weapon numbers:* repeat part b with two changes. The base
    copy's section of the fixed-name type gets one pair only, the `Normal` pair, which maps weapon
    17 to turret 1.
    In the test override the `Pistol` pair, which follows the `Flak` pair in the Import mapping
    order, maps weapon 3 to turret 1 instead of weapon 4 to turret 0. In a second run of part h,
    the first run's `Pistol` change is undone and the test override's `Normal` pair instead gives
    weapon number 17 and no turret index key, the base copy staying as in the first run.
  - *Part i, weapon not chosen by the index:* give the control's weapons in slots 0 and 1 two
    clearly different projectiles. With no passenger, order the control into the vehicle bunker
    and record whether it is taken in; then, outside the bunker, let it attack the target.
    Load the mode-1 passenger and let it attack again. Record which projectile each shot
    creates.
  - *Part j, before the first turn:* build a charge subject at the war factory for the tester's
    house, which a person controls, and capture every drawn frame from the tick its place order
    is given; place a second charge subject on the map, in view of the starting position, and
    capture the game's first drawn frame. A run in which the factory's image hides the built
    subject's turret area in the first drawn frame that shows the subject is inconclusive.
    Repeat it with a test copy of the war factory whose imported exit position puts a released
    vehicle outside its foundation.
  - *Part k, sprite body:* in each sprite run, repeat parts a and b with the subject, so that
    its index goes through 2, 0, 1 and *none* as in part b. Then, in the first sprite run, with
    no passenger, turn the subject's turret through all 4-step turret facings while it attacks
    targets placed round it, capturing every drawn frame. A run in which the subject is drawn
    with voxel parts as in part a (the voxel flag not cleared) is void. In the sprite-barrel runs,
    with no passenger:
    - *depth:* as S-0102 D33 (a), the subject and then the depth control drive toward the viewer
      past the wall segment one step at a time on the same path, with no view jump between the
      two drives, their images overlapping the wall's, the subject's turret held at one 4-step
      facing. In every drawn frame record the distance *b* − *b*<sub>wall</sub> in rows (*b* the
      shared last row of the subject's composed image or of the control's body frame), which
      compared pixels are shown, and which other pixels of the overlap are shown. The switch
      distance of each is the first distance at which all its compared pixels are shown. A run
      in which two consecutive drawn frames differ by more than one row of *b* within three
      rows of a switch is inconclusive. Then the subject stands still
      while the building, the tree and the voxel vehicle are placed in turn so that their images
      overlap its barrel from map north, south, east and west, at height 0 and again with the
      subject on the bridge deck. Record which pixels of the overlap are shown, and the
      vehicle's shadow region.
    - *pose:* with the body facing held, turn the turret through the eight step directions and
      capture the barrel at each.
    - *frame:* drive the subject through at least 8 walk-counter steps, then let it attack the
      target at rest, capturing the barrel in every drawn frame.
    - *invisible:* place the subject and capture its barrel from placement until at least
      8 ticks after it.
  - *Part l, magazine:* let the magazine subject attack the target while standing still until
    its magazine is empty. Go on capturing every tick, recording its ammunition, its rearm
    countdown and its reload countdown, until it has regained a round and fired it.
- *Expected:*
  - Part a: both subjects show turret model 2 and no barrel (barrel model 2 is absent). The
    control shows no turret and no barrel.
  - Part b: mode 1 gives turret 0 with barrel 0, from the tick of entry. The second passenger
    changes nothing, and the last exit brings back turret 2. Mode 2 gives turret 1 with barrel 1.
    Mode 3 shows neither turret nor barrel, and neither does mode 17. Mode 20 shows turret 2.
  - Part c: turret 0 throughout.
  - Part d: no turret and no barrel in any state. With `quirks.turret_table_fixed_type_only`
    off (engine conformance, not the original) the control shows what part b shows for the
    subject.
  - Part e: in every drawn frame the numbered turret shows the body's current frame *f_b*. While
    attacking at rest with *f_b* = 0 it stays in frame 0 throughout.
  - Part f: model 2 in the ticks whose remaining countdown *r* is at least 2*D*/3, model 1 for
    *D*/3 ≤ *r* < 2*D*/3, and model 0 for smaller *r* and after the countdown.
  - Part g: the subject behaves as a charge-turret type: hand-overs change neither weapon nor
    turret. In the mirror run the flag
    set in the override is cleared and hand-overs change the turret again, as in part b, except
    that weapon number 1 has no entry, so the mode-1 passenger shows neither turret nor barrel.
  - Part h: mode 17 gives turret 1 with barrel 1 (the base copy's entry is kept, because the
    override sets no pair for weapon 17), and mode 3 gives turret 1 with barrel 1 (the later
    pair for weapon 3 wins). The other modes are as in part b. In the second run, mode 17 gives
    turret 0 with barrel 0 (the stem's default index, not the base copy's 1), and the subject
    shows no turret at creation and after the last exit (weapon 0 has no entry).
  - Part i: the control, which shows no turret throughout, is taken into the bunker, fires the
    slot-0 projectile without a passenger and the slot-1 projectile with the mode-1 passenger.
  - Part j (original): the first drawn frame that shows the built charge subject shows neither a
    turret nor a barrel, and every drawn frame after its first turn and before its first shot
    shows turret model 0.
    The map-placed charge subject shows neither in the game's first drawn frame, if that frame
    shows it, and turret model 0 after its first turn. Our engine shows turret model 0 in all of these frames.
  - Part k: in every drawn frame and with every passenger, at indices 2, 0, 1 and *none* alike, the
    subject shows a sprite turret frame, and a hand-over does not change which model or sprite
    frame is selected. Which frame shows for a turret facing is S-0102 claim 22's rule and is
    judged there, not here. In the first sprite run it also shows
    the single barrel model at the turret, turned with it, winning over the sprite turret where
    they overlap at 4-step turret facings 1 and 2 and losing at 0 and 3 (claim 11). In the second
    sprite run it shows no voxel part. Neither run ever shows the single turret model or a numbered
    model. In the sprite-barrel runs:
    - *depth:* from S-0102 F2 with the shared last row (*U* = −1 for the subject and *U* − 2 =
      −3 for the depth control, against the wall's −2, upright), the depth control switches at
      0 to 2 rows, as in D33 (a), and the subject at 2 or 3 rows: the pair of switch distances
      (control, subject) is (0, 2), (1, 3) or (2, 3), whichever the path's rows give. These pairs
      follow from S-0102 *Comparing two drawings*: the control is shown once (4/3)·(*b* −
      *b*<sub>wall</sub>) > −1 and the subject once it is > +1, two depth units (1.5 rows) later,
      each within F2's rounding. At 3 rows the subject's margin, 4 units against a difference of
      1, exceeds what the rounding Q and the floors can remove on any row, so the subject's
      switch is never later than 3 rows, and its switch comes 1 or 2 rows after the control's.
      Taking every alignment of the shared last row and the wall's last row modulo 3, with at
      least three consecutive compared rows, gives exactly these three pairs, so a pair such as
      (2, 4) does not arise. In the depth run without a barrel, the same holds for the drive.
      At every distance, each compared pixel shown for the subject is also shown for the control; at
      least one distance shows all the control's compared pixels and not all the subject's; and
      once all the subject's compared pixels are shown, they stay shown at every greater
      distance up to 4 rows. In every drawn frame, where pixels of another object that share
      one writer depth (S-0102 F2) overlap the composed image on one screen row, the composed
      image's pixels under them are shown or hidden alike, whether they belong to the barrel,
      the body or the sprite turret. Against the building and the tree, from every side, the
      barrel is hidden or shown exactly where the body and the sprite turret would be on the same
      rows. The voxel vehicle
      never cuts the subject along a row: whichever is drawn later covers the other, barrel
      included. On the deck the same holds. The barrel never darkens or adds to the shadow
      region.
    - *pose:* the barrel's origin sits 0, 25, −12 and −1 model units ahead of the body's origin
      in the four pose runs, on the body's forward axis, and the barrel turns with the turret's
      32-step facing relative to the body.
    - *frame:* frame 0 of the barrel's HVA in every drawn frame.
    - *invisible (original):* once placement's turn is over, the barrel shows level with the
      fire angle left out, and one step (11.25°) below level with 20, while the visible control
      shows two steps above level. Our engine shows claim 6's elevation in all three runs: one
      step above level, and two steps above level twice.
  - Part l: model 0 from the subject's first turn until its first shot, with a full magazine.
    After each shot, the shot made with the last round included, the steps of part f for that
    shot's countdown. Model 0 in every tick in which the magazine is empty and the countdown is
    over, with no change in the tick a round comes back. The shot made with the regained round
    starts again at model 2.
- *Falsified if:* a turret numbered like the weapon number appears where the table maps
  elsewhere; the control shows a turret; a model shows for weapon number 3 or 17 outside part h; in
  part h, mode 17 shows no turret (an entry lost between layers) or mode 3 shows none (the
  earlier pair won); the model
  changes in a tick other than the weapon number's; promotion or transport carriage changes the
  model; a numbered turret shows a frame other than *f_b*; the charge subject's steps differ from
  S-0301 claim 16; part g keeps gunner behaviour, or its mirror run keeps charge-turret behaviour, while the
  quirk is on; in part h's second run, mode 17 shows turret 1 (a default taken from an earlier
  layer); in part i, the control is refused by the bunker, does not fire, or fires a projectile
  other than its weapon number's (the index choosing the weapon); in part j, the built charge
  subject shows a turret in the first drawn frame that shows it, or shows no turret in a later
  frame before its first shot; in part k, the subject shows a voxel turret model, a
  numbered model, or no sprite turret in some frame, what is drawn changes at a hand-over, or
  its barrel's overlap with the sprite turret follows another rule than claim 11's; in part k's
  sprite-barrel runs, the subject's compared pixels are shown or hidden as the control's at
  every distance (the sprite row *U* − 2), the pair of switch distances is none of (0, 2),
  (1, 3) and (2, 3), a compared pixel is shown for the subject at a distance where it is hidden
  for the control, a compared pixel of the subject is hidden again at a greater distance after
  all were shown, barrel pixels on a row are hidden where body pixels on the same row, under
  pixels of the same writer depth, are shown or the other way round, the voxel vehicle cuts the
  barrel along a row, in the depth run without a barrel the subject's compared pixels are shown
  or hidden as the control's at every distance, the barrel is
  shifted by *δ* × ρ (about 33.1, −16.6 and −2.0 model units), rounded down (−13 for −100) or
  not shifted, it shows a frame other than 0, the shadow region shows a barrel outline, or the
  invisible runs show the visible control's elevation in the original; in part l,
  some tick shows a model other than the countdown gives, such as a turret other than 0, or no
  turret, while the magazine is empty and the countdown is over, or the model changes in the
  tick a round comes back.
- *Tool:* the measurement harness, with the read-only recorder for weapon number, countdowns and
  (part l) ammunition.

**MS-34 — Vehicle shadow anchor (claims 39, 44; Q-2672).**

- *Setup:* the common setup, speed index 2, 800×600, one human house and one bystander
  computer player outside the observed area. A tiny purpose-made map with flat clear ground
  at level 0 and at one whole level from 1 to 9, a slope between them, a water area at level
  0, and an intact high bridge over water and one over level-0 land, with visible terrain
  beneath each. A stock hovering vehicle type and a stock driving vehicle type, both drawn as
  voxel models that cast shadows. Screen capture plus a read-only recorder of tick,
  horizontal position, world height, terrain surface height and whether the vehicle is on a
  bridge deck.
- *Procedure:* (a) rest the hovering vehicle on flat ground at each level, on water, on each
  bridge deck and under each bridge, at two facings each, and capture several consecutive
  drawn frames through its bob. Repeat while it moves along a straight route over each
  surface. (b) Rest the driving vehicle on flat ground at each level, on the slope (its
  lower half and its upper edge) and on a deck. (c) Give the hovering type MS-27's test body
  (art database copy, test models, cache controls), whose own image has a horizontal lower
  or upper edge inside the shadow footprint, and capture it at rest, with MS-27's no-shadow
  control. From the test data, select the footprint pixels where the own image is opaque one
  row below but transparent at the same position, and those where it is opaque at the same
  position but transparent one row below. (d) Set the resting hovering vehicle rocking with
  a hit (S-0106) and capture while it rocks.
- *Observable:* the anchor recovered by matching the known footprint (claims 30 and 31, F10),
  as MS-30 does, against the body's draw point and the projection at the surface height,
  with *a* from the recorder, for every selected frame. In part c, whether each selected
  footprint pixel is darkened against the control.
- *Expected:* parts a, b and d, in the original: the anchor lies exactly at the ground projection on
  a level-0 surface (flat ground, water, under a bridge over level-0 land) and stays there through
  the bob and the rocking while the body moves. On the higher level and on a deck the anchor's
  distance from the surface projection and below the body's draw point is recorded for each frame:
  under S-0313 claim 11b's hypothesis it is within one row of the projection and S-0409 F2's lift of
  *a* below the body. The footprint's outline does not change with the rocking. The driving vehicle
  has *a* = 0 and the anchor at its body's point on flat ground and on the deck, and on the slope
  the anchor lies below its body's point by the lift of the recorder's *a* (0 where *a* is 0). Part
  c, in the original: a selected pixel stays undarkened exactly where the own image is opaque one
  row below it, so the two selected sets give opposite results.
- *Falsified if:* on a level-0 surface, an anchor off the ground projection by any row (for a
  vehicle whose body is drawn at the projection of its position and world height, so with no draw
  shift), or one moving with the bob or the rocking, falsifies claim 44. On the higher level and on
  a deck, a recorded distance outside one row of the projection, or a distance below the body other
  than S-0409 F2's lift of *a*, bears on S-0313 claim 11b's hypothesis and the row bounds that rest
  on it, not on claim 44. A deck, water or under-bridge anchor at another surface falsifies the
  surface rule. In part c, both sets darkened or both undarkened, or the set opaque at the same
  position undarkened, falsifies claim 44's original comparison, not the engine rule.
- *Engine conformance (part e):* named test types with the flying locomotor and with the
  hunter-seeker property, and fixtures for *a* < 0, for a lifted vehicle, for a slope and for a
  hunter-seeker type on a deck below claim 40's bridge qualification, run in our engine only: every
  anchor follows claim 44's rule with S-0101's lift (for the hunter-seeker fixtures, claim 40's *P*,
  so the terrain point on that deck; for other fixtures that are not lifted, exactly the surface
  projection at whole levels and within one row on the slope; for the lifted fixture, that anchor
  raised by 14 rows, and by a further 60 rows in claim 44's bridge case), and every own-image
  comparison is at the same screen position. This is not an oracle check.
- *Tool:* the measurement harness, with the read-only recorder named in the setup.

**MS-35 — When the barrel-elevation turn starts (claim 6; Q-2982).**
- *Test data:* as MS-14: a test map override section sets the fire angle of one stock vehicle
  type with a barrel part to 20, and the art database copy gives its image the cache-disable
  option. Parts b and e instead set the fire angle to 64 (a turn of 21 ticks, from level to
  straight up, ending at step 8), so that the turn is still under way when the factory first
  shows the barrel and when the save is made. For part f, the override section also has the stock
  magnetic-lift weapon's seizing warhead name the teleporting locomotor, so that a seize uses the
  teleporting stand-in (S-0607 claims 22 and 23).
- *Setup:* the common setup. A stock vehicle transport, a stock factory that builds the test
  type, a stock tank bunker that the test type can enter, a stock repair depot, a chronosphere
  ready for the human house, and a stock magnetic-lift unit of the human house with a test
  vehicle of a second house in its reach. The test map places one test vehicle of the human house
  with less than full health. A read-only recorder of the tick number and of each object's
  presence on the map, beside per-drawn-frame captures.
- *Procedure:* in every part except b and e, the unload, entry, docking, warp or seize comes at
  least 30 ticks after the vehicle's previous placement. In (b) the built vehicle has no earlier
  placement, and it is first listed on the map at tick 30 or later, so that a turn counted from
  the game's start would show step 8 in every frame. In (e) the unload is the placement, and the
  save comes during the turn that it starts.
  - (a) Load a vehicle into the transport, wait, and unload it on open ground.
  - (b) Build the vehicle at the factory. If no frame whose tick number is below *t*₀ + 18 shows
    its barrel, where *t*₀ is the tick in which the recorder first lists the vehicle on the map,
    the run is void and is repeated with the factory's exit in view. The void rule reads only
    tick numbers and whether the barrel is in view, never the step shown, and F9 gives a step
    below 8 to every such frame.
  - (c) Order a vehicle into the tank bunker, wait, and sell the bunker, which lets the vehicle
    out (S-0606 claim 13).
  - (d) Warp a vehicle with the chronosphere to open ground.
  - (e) Unload a vehicle as in (a). While its last drawn frame shows a step from 2 to 6, open the
    options menu (no tick runs while it is open), save, and later load that save. If the last
    frame before the save shows another step, the run is void and is repeated.
  - (f) In a run with part f's test data, let the magnetic-lift unit seize the test vehicle, and
    capture it through the teleporting stand-in's jump and landing.
  - (g) Let the magnetic-lift unit seize the test vehicle, and during the pull give the lifter a
    move order, which ends the grip (S-0607 claim 12), so that the vehicle is dropped over clear
    land (S-0607 claim 13).
  - (h) Send the test vehicle with less than full health to the repair depot, let it be repaired
    on the depot, and let it leave.
- *Observable:* the barrel's 32-step pitch step (0 = level, 1 = 11.25° up, and so on) in each
  drawn frame with the tick number current while it is drawn, the tick numbers at which the
  recorder lists the vehicle on the map and off it, and in (e) the recorder's tick number just
  before the save and just after the load.
- *Expected:*
  - (a): from the first drawn frame after the tick in which the recorder first lists the vehicle
    on the map again, the steps are 0, 1, 1, 2, 2, 2, …: step 0 once, step 1 for exactly two
    frames.
  - (b): the factory may hide the first frames. Every frame that shows the barrel, up to step 8,
    shows the step that F9 gives for the fire angle of 64 at *ε = n − t*₀, where *n* is the
    frame's tick number and *t*₀ the tick in which the recorder first lists the vehicle on the
    map.
  - (c), (d), (f), (g) and (h): step 2 in every frame, through entering and leaving, the warp,
    the seize, jump, pull, drop, fall and landing, and the docking, repair and leaving. In (f),
    (g) and (h) the recorder lists the vehicle on the map in every tick.
  - (e): the tick number after the load is the one just before the save. Every frame before the
    save and after the load, up to step 8, shows the step that F9 gives for the fire angle of 64
    at *ε = n − t*₀, where *n* is the frame's tick number and *t*₀ the tick in which the recorder
    first lists the vehicle on the map after the unload. So the steps after the load go on as if
    there had been no save.
- *Falsified if:* in (a), the steps start 0, 0, 1, 1 (first appearance at the placement tick
  itself) or step 2 shows in the first frame (no restart); in (b), any frame that shows the
  barrel at a step other than F9's for its *ε*; in (c), (d), (f), (g) or (h), any frame below
  step 2, or in (f), (g) or (h) any tick in which the recorder does not list the vehicle on the
  map; in (e), a tick number after the load other than the one before the save, or any frame
  after the load whose step differs from F9's step for its *ε*.
- *Tool:* the measurement harness, with the recorder above.

**MS-36 — Aircraft draw no turret or barrel (claim 45; Q-2983).**
- *Test data:* a test map override section gives the turret flag, a nonzero test drawing-pitch
  angle and a nonzero test bank angle to one stock aircraft type with voxel art and the flying
  locomotor (part a), the turret flag to one stock rocket type (part a) and to the stock spawned
  aircraft type of a stock carrier with voxel art (part b). The art database copy gives each of
  these images a test turret model and a test barrel model: one section each, filled with a
  colour index that no other model in view uses, and sized to stick out of the body on every
  side. For part c, a second run of part a's aircraft also has its locomotor field set to the
  jumpjet locomotor. As a positive control, the same test turret and barrel models go on a stock
  vehicle type that has the turret flag.
- *Setup:* the common setup, with the rocket's stock launcher and the carrier for the human
  house, and a computer house's defences that can shoot the aircraft down.
- *Procedure:*
  - (a) Capture the aircraft flying level, turning (banked), and level again; capture the rocket
    from launch to impact; let the defences shoot the aircraft down and capture its crash.
  - (b) Order the carrier to attack, and capture its spawned aircraft from launch to their
    return.
  - (c) In the second run, capture part a's aircraft on the ground and while it moves, whether or
    not it leaves the ground.
  - Capture the control vehicle on the ground in each run.
- *Observable:* whether any pixel shows a colour that the test colour index gives under the
  lighting in view: the palette colours of that index in every shade row of the lighting table
  (claim 38) as claim 19 applies it. The index is chosen under *Discriminating colours* so that
  no other object in view shows any of those colours.
- *Expected:* the control shows the test turret and barrel in those colours; no capture of an
  aircraft-kind object in any part shows any of them.
- *Falsified if:* any pixel in those colours appears on an aircraft-kind object in any part. If
  the control shows no turret or barrel, the run is void (the test models did not load).
- *Tool:* screen capture.

## Open questions

- **Second condition of the tall-image split.** Claim 12 (*Depth*) splits the composed image of
  a type too big to fit under a bridge when it is taller than 16 rows, near a bridge with its
  column factor 0, or in one further situation that has not been identified. Until it is, that
  situation is rated low and no scenario sets it up.
- **Turretless vehicle with a barrel part.** A vehicle with a barrel part but no turret is drawn
  differently, with a shift derived from its weapon position and a pitch correction. Not yet
  understood; left out.
- **Own-image exclusion outside vehicles.** Claim 39 fixes vehicles, including hovering
  vehicles, only. Whether other voxel object kinds also exclude their own images is not
  determined here; S-0102 owns the aircraft observable.
- **Shadow on a cache miss.** When a shadow image is not yet cached, the shadow sometimes appears
  to be skipped for that draw instead of made. In which situations, and whether a shadow can be
  missing for one drawn frame, has not been determined.
- **Animation-frame counter.** S-0609 claim 14 owns advances for gattling-flagged types,
  including continued animation during spin-down and the corrected building double advance.
  Exact advances for other types belong to the combat, vehicle and building specs; MS-3c
  tests the initial frame and subsequent changes, not the exact counter-to-frame mapping.
- **Cache keys of other locomotors.** Claims 26 and 41–42 settle ordinary flying
  and rocket body keys and their pitch/bank freshness conditions. The tunnelling
  body's complete key and how these locomotors combine their keys with a vehicle
  turret key, including removal of body facing at turret displacement 0, remain
  undetermined. Hovering, jumpjet and tunnelling freshness during rocking or
  digging pitch is settled by S-0106 claim 13.
- **Image invalidation.** What observable event makes all earlier images ineligible for reuse
  in the original, and whether a new game does so, remain open (claim 26). Our engine retains
  reuse throughout a game and starts each game without earlier images. Invalidation bounds how
  long the frame and elevation reuse artefacts persist in the original.
- **Other section ties (Q-0215 question 7).** F8's candidate general relation gives
  all MS-25/MS-28 pair and common-pixel predictions, including gap/absence repeats.
  Stock-oracle measurements and an assigned M-ID are still required. The scenarios
  discriminate stable file-order priority, including MS-28 row 4's B/E probe.
  Arbitrary-count validation and input sufficiency beyond the supplied patterns
  remain `unmeasured`; no finite capture set establishes that universal rule.
  MS-28's closing sweep proposes further count and near-equal-depth tests. Interim
  engine pixels are not reference results.

## Cross-spec reconciliation

- **S-0102 claim 18 and its open questions, S-0106 T10 and Open questions (Q-2672).** Claim 44 here
  gives the vehicle shadow draw point: above, on and below the surface, on water, on and under a
  bridge deck, and for a lifted vehicle. S-0102 claim 18's "vehicle shadow points remain unresolved
  (Open questions)" should cite claim 44, and its open question on the vehicle voxel shadow point
  should be dropped, keeping its sentence on own-image overlap. S-0106's open question on where an
  airborne vehicle's shadow lands vertically should point to claim 44, and neither of those changes
  behaviour. S-0102 D23 (d)'s hovering poses do change: its expectation that every own-body overlap
  pixel of a hovering vehicle equals the no-shadow control, with "a reversal on slopes or hovering"
  as a falsifier and "fresh variants remain oracle-testable", is our engine's rule (claim 39) but
  not the original's, which compares one row lower while the vehicle is above its surface (claim
  44). D23 (d) should keep that expectation for the hovering poses as engine conformance, take the
  hovering vehicle out of its fresh oracle variants, and point to MS-34 part c for the original's
  comparison. S-0106 T10's separate hovering-vehicle run, expecting filled own-image pixels
  undarkened on every appearance and offering fresh-shadow overlaps as an oracle check, likewise
  tests our engine's same-position comparison: it should keep that expectation as engine
  conformance, drop the oracle check for a hovering vehicle, and point to MS-34 part c for the
  original's comparison.
- **S-0106 claims 5, 8, 11 and 12.** Claims 26 and 41–42 here resolve the flying/rocket
  body-attitude and reuse questions. Claim 11 there and its aircraft heading open question must be
  updated: flying shadows use the 32-step aiming facing, rocket shadows the 32-step body facing,
  and neither includes body pitch or bank. Claim 12's shadow key uses the same heading source.
  Retain that spec's slope transform and dropship flat-shadow exception. Its current unresolved
  heading statement is not an alternative to claims 30 and 42 here.
- **S-0506 claims 15 and 17.** Add a pointer to claim 41 here for the body's
  appearance throughout the flight phases and its level/endpoint reuse classes:
  current pitch is drawn without 32-step pitch quantisation and does not pitch
  the shadow. Flight-pitch evolution and movement formulas are unchanged.
- **S-0308 claims 5 and 6, S-0301 claim 16 (Q-1760).** Claim 43 here adds only the turret-table
  lookup on top of S-0308's weapon number and cites S-0301 claim 16 for charge turrets; S-0308
  needs no change for it. S-0301 claim 16 says every shot of a charge-turret vehicle uses its
  slot-0 weapon. In the original, S-0308 claim 9's override for a deployed vehicle whose type has
  the deploy-fire flag applies to a charge-turret type too: such a vehicle, while deployed, fires
  its type's deploy weapon index, whatever turret image is shown. No charge-turret type of the
  stock data has the deploy-fire flag, so the two specs differ only for a type from a user's file
  (`unmeasured`, medium: no scenario sets one up). S-0301 claim 16 should read "uses the weapon
  S-0308 chooses: its slot-0 weapon, unless S-0308 claim 9 applies". S-0308 should note `quirks.turret_pair_no_weapon_sets_charge_flag`, which can
  switch the charge-turret flag of the fixed type name. If S-0308 scenario W4 part h builds its
  charge-turret gunner from that type through a map override, an omitted weapon-number key in
  that override changes the flag too.
- **S-0606 claim 1 and S-0209.** S-0606 claim 1 asks for "the weapon of its current turret
  index", and S-0209's Terms entry *Armed*, in its *has a primary weapon* rule, for "the weapon
  of its currently selected turret". In the original those tests, like every other weapon test, ask the weapon
  S-0308 chooses and never the selected turret index (claim 43, *Not a weapon choice*; MS-33
  part i for the bunker test). Both phrases should become "the weapon S-0308 chooses: that of
  its weapon number (S-0308 claim 5), unless S-0308 claim 9's override for a deployed vehicle
  applies". This changes no behaviour of the original, which
  is what both specs describe; it corrects the wording. Read literally beside claim 43, the old
  wording would differ for a multi-turret type whose index is *none*, such as any such type
  other than the fixed type name: S-0606's "it has a turret weapon" would then find no weapon
  and refuse it. In the original such a vehicle passes that test whenever the weapon S-0308
  chooses holds a weapon (outside S-0308 claim 9, the slot of its weapon number, 0 from
  creation), and the new wording says so. For the fixed
  type name, too, the weapon is that of the weapon number wherever its table maps that number.
- **Hover tip name (pointer only).** This spec does not specify hover tips. The interface spec
  that takes them should check whether a multi-turret type's hover-tip name follows its selected
  turret index (claim 43).
- **S-0007 claims 16, 17 and 19.** Claims 16 and 17 are unchanged: a multi-turret type with the
  voxel flag has no ordinary turret or barrel, which claim 12 here relies on. Claim 19 is
  unchanged as a loading rule: a sprite-body vehicle with the turret flag loads the single voxel
  turret and barrel whatever its turret count, and no numbered models. Its backing should note
  that of these two parts only the barrel is ever drawn (claim 12 here, *Sprite body*), so its
  scenario A14 can see the barrel lookup but never the voxel turret lookup. This changes no
  behaviour; it narrows what A14 can show.
- **S-0102 colour-selector table and S-0610 claim 7 table (wording only).** Their rows for a
  vehicle drawn as a sprite say "including a voxel turret on a sprite body". By claim 12 here
  (*Sprite body*), only the voxel barrel is drawn on a sprite body. The rows should read "including
  a voxel barrel on a sprite body". Neither row's colouring changes. The amended rows should also
  note that this barrel's overlap with the sprite turret follows claim 12 here, which extends
  claim 11's overlap rule, by the 4-step turret facing, to a sprite turret. MS-2 and MS-6 back
  claim 11 for voxel turrets only, so the extension is rated medium until MS-33 part k runs.
- **S-0102 claims 13, 14 and 22 (Q-1885).** A vehicle drawn as a sprite whose type has the turret
  flag, with or without a barrel part, is drawn as one composed image (claim 12 here, *Sprite body*:
  body frame, voxel barrel where the type has one, turret frame), placed with the row of a vehicle
  drawn as a voxel model: *U*, upright, a reader, at every height. S-0102 claim 13's rows "vehicle
  drawn as a sprite, *h* = 0" and "*h* not 0" should say that they apply only to a vehicle drawn as
  a sprite whose type lacks the turret flag, and the voxel row should name the turret-flag sprite
  vehicle too. Claim 14's sentence "Infantry and vehicles drawn as sprites use *U* only while their
  height above the surface is 0" should exclude it in the same way. Claim 22's "combined into one
  image" should add that the voxel barrel, where the type has one, is part of that image, and cite
  claim 12 here for its order and for its rectangle, which F2 anchors at its last row. The shadow
  row there is unchanged. The row "vehicle drawn as a sprite whose type is too big to fit under a
  bridge" (*U* − 18, flat) likewise applies only without the turret flag. With it, and for vehicles
  drawn as voxel models too, the original instead splits the composed image when it is taller than
  16 rows, under that row's condition (near a bridge, column factor 0) and in one further situation
  not identified here (Open questions): its lowest 16 rows keep *U*, upright, and the rows above
  them take *U* − 5, flat. Claim 12 here (*Depth*) rates this split low and `unmeasured`: no
  scenario here sets it up, and S-0102 owns it.

## Decided

- **Image invalidation.** Claim 26 keeps reuse for a whole game in our engine, while the original
  can invalidate earlier images together at an event not specified here. That departure, with no
  switch, was decided at the delegated sign-off, 2026-10-25.
Image enlargement and turret-origin pitching are settled as unconditional presentation
fixes, with no extra pivot import. Claim 22 settles Q-0470's bounded-image policy and
requires one importer warning per oversized model.
