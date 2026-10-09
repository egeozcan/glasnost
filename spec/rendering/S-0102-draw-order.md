---
id: S-0102
title: Draw order, render layers and depth compositing
topic: rendering
revision: 2
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-09
measurements: []
requests: []
supersedes: []
---

# S-0102: Draw order, render layers and depth compositing

## Summary

In which order the battlefield is painted in each drawn frame, how objects are grouped into five
render layers and ordered inside each layer, and how a per-pixel depth buffer decides which pixels of
terrain, overlays, buildings, units, projectiles and animations hide each other. Positions, world
pixels and the height lift *L(z)* are those of S-0101.

## Terms

Shared terms (cell, lepton, height level, world height, height above the terrain, foundation, drawn
frame, tick, render layer, fog of war, locomotor, jumpjet, jumpjet type, high bridge, bridge deck,
sprite, detail level, local player, override section, measurement setup and others) are
used as defined in the glossary. Positional terms (world pixel, battlefield view, view origin,
height lift) are those of S-0101; the battlefield view's pixels are screen pixels and their rows are
screen rows.

| Term | Meaning |
|---|---|
| layer | In this spec, *layer* on its own always means *render layer* (glossary), never *rules layer*. |
| layer list | The ordered sequence of objects in one render layer. Objects are drawn in list order, so a later object paints over an earlier one where both pass the depth test. |
| movement type, *X* movement | The object's locomotor (glossary); "jumpjet movement" means the jumpjet locomotor, and so on. A unit "with jumpjet movement" is any unit currently using the jumpjet locomotor, whether or not its type has the imported jumpjet flag; the layer rules (claim 3, *State and ordering*) depend on the locomotor only. Where this spec needs the flag instead (the choice of unloading image, claim 13) it says *jumpjet type* (glossary). |
| veinhole monster | A stationary map object that the original supports as the source of a vein field (a ground cover that harms units on it). It is listed here only to say that it belongs to no layer (claim 3). |
| building light effect | The moving searchlight beam that some buildings cast on the ground. It is an object of its own, separate from the building that casts it. |
| light flash | The short, round flash of light drawn where certain warheads hit, and by some particle effects. Whether it is shown depends on the detail level and on the rate of drawn frames. |
| balloon-hover type | A type with the imported balloon-hover flag (Import mapping). Used here only to choose a building's unloading image (claim 13). |
| gate | A building whose type has the imported gate flag: a wall-like building that opens to let units pass. |
| construction state | The body animation state *construction* of S-0108. A building is in it while it plays its build-up sequence after being placed, and again while it is being sold, when the sequence plays backwards. *Build-up* and *selling* below are these two parts of it. |
| unloading | The state of a building while it releases a unit it holds, for example a factory letting a finished unit out. The unit being released is the building's *unloading unit*. |
| bib | The building's bib sprite (glossary): an optional flat sprite that a building's art can name, drawn on the ground under and around the building. It is drawn except while the building is in the construction state (so never during build-up or selling) and while a gate is opening or closing (claim 13; S-0108, claims 4 and 7). |
| deploying sprite, roof-deploying sprite | Optional sprites (glossary) that a building's art can name. While the building is unloading, one of them is drawn in place of its body sprite (claim 13). |
| under-door sprites | The under-door sprite and the under-roof-door sprite (glossary): optional flat sprites that a building's art can name, drawn under the opening door while the building is unloading. |
| door sprite | An optional flat sprite that a building's art can name, drawn while the building is unloading and its door is opening, open or closing. |
| slot animation | An animation that a building keeps in one of its slots (S-0108). It is an animation object of its own, created at the building but not attached to it (claim 17). A building's turret drawn as a sprite is one. |
| turret slot depth adjustment | The depth adjustment of a building type's turret slot (Import mapping). It is the depth adjustment of the turret slot animation (claim 17) and also of a voxel turret (claim 13). |
| voxel shadow | The flat shadow image that a vehicle or aircraft drawn as a voxel model casts (S-0105, claims 29 to 31). |
| voxel copy rectangle | The screen rectangle into which a voxel model's finished image is copied: S-0105, claim 25 and formula F6. |
| sort key | Integer used to order the ground layer list (claim 5). |
| cell pass | The part of the drawing that paints the persistent terrain picture: terrain tiles, overlays, non-animated terrain objects, the buildings' bodies with their shadows, bibs and under-door sprites (claim 16) and, with fog of war on, the remembered images of fogged buildings. It is not repeated in every drawn frame; it repaints only areas marked for redraw and newly scrolled-in strips. |
| terrain picture | The persistent image (with its depth buffer) produced by the cell pass, copied as the background of every drawn frame. |
| object pass | The part of a drawn frame that paints the five layer lists, after the cell pass. |
| remembered image | What a player sees of a building that is hidden under fog of war: a still copy of how it looked when last seen (body, shadow, bib and animations), painted in the cell pass. Only buildings have remembered images (claim 16). |
| status extras | The markers drawn for a vehicle, infantry, aircraft or building on top of its image: health bar, pips, selection marker and similar. |
| information tip | Text or markers a building shows next to its image (for example while the pointer rests on it). |
| depth buffer | One integer in 0…65535 per screen pixel of the battlefield view. **Smaller means nearer to the viewer.** |
| depth unit | The unit of depth values. Along a flat ground line, one depth unit is one screen pixel row (F1). |
| depth reader / depth writer | A drawing that tests its pixels against the depth buffer; a writer also stores its own depth where it draws. |
| row depth | The depth value of one row of a drawing before per-pixel adjustment (F2). |
| depth bias | A signed integer, in depth units, added to the row depth of a whole drawing. The biases in claim 13 are final values. |
| depth profile | How row depth changes from row to row within one drawing: **flat** (lying on the ground), **slope** (45°), **upright** (standing vertically). |
| companion depth image | An optional second image with one integer in −128…127 per pixel, subtracted from the row depth pixel by pixel (used by some buildings). |
| tile depth data | The per-pixel depth values, integers in 0…255, stored with each terrain tile image. |
| world pixel row | The row coordinate of a world pixel (S-0101). Increasing downward on screen. |
| ground reference row | For a point, the world pixel row it would have at world height 0 (S-0101 F1 without *L*). |
| *L(z)* | Height lift of S-0101 F2, in pixels. S-0101 computes it in floating point and rounds it to an integer; this spec uses only that integer result, which is not the same as 15/104 × *z* rounded (for example *L*(400) = 57). |
| depth reference *C* | The integer that ties depth values to world pixel rows (F1). It is reset at the events listed in F1 and otherwise stays fixed. |
| view jump | A change of the battlefield view's position between two drawn frames by more than the view's width across or more than its height up or down, for example after a click on a distant point of the radar map. Smaller changes are *scrolls*. Both sizes are in screen pixels. |
| global cruise height | A game-wide height above the terrain, in leptons, that separates the air layer from the top layer for objects whose layer is not decided by their own movement type (Import mapping). |
| jumpjet cruise height | A per-type height above the terrain, in leptons, that separates the air layer from the top layer for units with jumpjet movement (Import mapping, *State and ordering*). |
| terrain fudge | A per-type depth value, in depth units, that pushes a vehicle, infantry or aircraft farther from the viewer near cliffs, bridge columns, tunnel entrances or high bridges (claim 14). |
| unit depth adjustment *U* | The depth bias of a vehicle, infantry or aircraft before the constants of claim 13 (claim 14). |

## Behaviour

### Structure of a drawn frame

1. **Order of a drawn frame.** Each drawn frame starts from the terrain picture, after the cell pass
   has repainted whatever parts of it were marked for redraw (claim 16 gives the precedence inside
   the cell pass). The object pass then draws the five layers in the fixed order underground, surface,
   ground, air, top, and within a layer in layer-list order. So wherever two objects overlap and both
   pass the depth test, the one in the later layer, or later in the same list, is visible. After the
   last layer, the status extras of every vehicle, infantry, aircraft and building drawn in that frame
   are painted, so every status extra appears above every object of every layer.
   - Backing: `measurable` (scenarios D1, D6, D7)
   - Confidence: high
2. **Effects around the layers.** Ion-cannon blast effects are drawn before the layers, so objects of
   every layer can paint over them. Light flashes, beam effects (lasers, electric bolts, radiation
   beams), range circles and map beacons are drawn after all five layers and after the status extras,
   so no object paints over them and they paint over status extras. Building information tips are
   drawn right after the ground layer, so objects of the air and top layers and all status extras can
   paint over them, but no object of the underground, surface or ground layer does.
   - Backing: `measurable` (scenario D13: map beacons and information tips in (a), beam effects in
     (b), range circles in (c), light flashes in (d)). Ion-cannon blast effects: `unmeasured`, because
     no stock weapon or game event that produces one is known.
   - Confidence: high for the order of the effect groups relative to the layers and the status
     extras; high for map beacons, information tips and the three kinds of beam; medium for light
     flashes; low for range circles (which drawing is the range circle is inferred)
3. **Layer of each object kind.** An object's layer is determined by its kind and state as in the
   table under *State and ordering*. In short: vehicles, infantry and buildings on the ground are in
   the ground layer; aircraft and units with flying movement are in the top layer while in the air (an
   aircraft with rocket movement is in the air layer); units with jumpjet movement are in the air
   layer below their jumpjet cruise height and in the top layer at or above it; projectiles are in the air
   layer unless their image is marked flat (then surface); animations attached to an object are in the
   ground layer, other animations use their type's layer (default air); particles, voxel animations,
   wave effects and building light effects are in the air layer; particle systems in the ground
   layer; units digging through a tunnel evaluate to the underground layer; the veinhole monster is
   in no layer.
   - Backing: `measurable`. Coverage by row of the table:
     - walking, driving, hovering, ship, mech and teleport movement: D2, D3, D14, D19 (d).
     - flying movement, in the air and on the ground: D2, D5, D12, D14.
     - jumpjet movement, all three rows, and the jumpjet cruise height with its minimum and default:
       D2, D12, D18.
     - rocket movement: D19 (a).
     - buildings on the ground: D4.
     - projectiles: D2 (b).
     - attached animations: D14 (c); unattached animations: D19 (b).
     - particles, voxel animations and wave effects: D19 (c).

     `unmeasured`:
     - drop-pod movement: no stock situation that creates a drop-pod unit has been found.
     - tunnel movement, both rows: whether a tunnelling unit is drawn at all while underground is not
       settled (Open questions).
     - terrain objects and other objects on the ground: non-animated terrain objects are painted in
       the cell pass, not from their layer list; animated or transient terrain objects (Open
       questions) are drawn from the ground list, but no scenario here targets them yet.
     - buildings, terrain objects and other objects in the air, the global cruise height and its
       400-lepton default: no stock situation lifts such an object off the ground.
     - the bridge rule for units with jumpjet movement (*State and ordering*): the moments at which
       such a unit's layer is
       evaluated in flight are not settled (Open questions), so no setup can make the rule show at a
       known moment.
     - building light effects: making one on purpose needs a building type that casts a
       searchlight, and whether stock data has one that a test map can use has not been established.
     - particle systems: no drawing of a particle system's own, apart from its particles, has been
       identified, so there is nothing to observe.
     - the veinhole monster: creating one needs map data whose stock support has not been
       established.
   - Confidence: high for the table, except: the tunnel rows (low: tunnelling units may in practice
     stay in the ground list, see Open questions); the jumpjet bridge rule, the jumpjet cruise
     height minimum and the cruise height being also the hover height (medium); the veinhole monster (medium: it is in no layer, and no drawing of it in
     a drawn frame was found); building light effects (medium)
4. **When an object changes layer.** An object's layer is evaluated, and the object put into that
   layer's list, when it is placed on the map, when an animation is attached to an owner object, and
   at take-off and landing of flying, jumpjet and rocket movement (and possibly at further steps of
   flight, see Open questions). Its layer is the one the table gives for its state and height at that
   moment. Between those moments the
   object stays in the list it was put into, even if its height has crossed a cruise height since.
   Every evaluation places the object anew in the list of the layer it evaluates to, even when that
   is the layer it was already in. In the underground, surface, air and top layers it becomes the
   last object of the list, so it is drawn over every other object of that layer wherever both pass
   the depth test; in the ground layer it is placed as in claim 6. So an evaluation that keeps an
   aircraft in the top layer still brings it in front of every other object of the top layer. While an object is falling (dropped or parachuted), its layer
   is re-evaluated every tick and changed as soon as it differs.
   - Backing: `measurable` (scenarios D5, D12, D14) for placement, attachment, take-off and landing,
     `unmeasured` for the falling rule: vehicles and infantry take their layer from their
     movement type alone, so falling never changes their layer, and no stock situation drops an
     object whose layer follows its height
   - Confidence: medium for placement, attachment and the falling rule; low for exactly which steps
     of flying and jumpjet movement re-evaluate the layer

### Order inside a layer

5. **Sort key.** The sort key of an object is X + Y of its position in leptons, with these
   exceptions:
   - a building uses the top corner of its foundation (the corner with the smallest x and y), i.e.
     X + Y of the top-left foundation cell's corner (cell index × 256, without the +128 centre), not
     its centre; it then adds 32 if its turret is a voxel model and subtracts 16 if it is a gate,
   - an animation adds its sort adjustment: its type's, except for a building's slot animation,
     which adds its slot's y-sort value instead (claim 17).

   Height plays no part in the key.
   - Backing: `measurable` (scenarios D3, D4; the animation sort adjustment D21), except the gate
     term, which is `unmeasured`: a
     building's place in the list shows only through the parts of it drawn in the object pass, no
     stock gate is known to have such parts, and an override section cannot give it any. D4 does
     not test it.
   - Confidence: high
6. **The ground layer is kept nearly sorted.** An object entering the ground layer is placed
   immediately before the first object in list order whose sort key is strictly greater than its own, or at the
   end if there is none. On a sorted list this puts it after all objects with an equal
   key; on a list that is not fully sorted it can land earlier than its sorted place (for keys
   5, 10, 3, a newcomer with key 4 goes first). Once per tick the ground layer list receives one pass
   of adjacent swaps from front to back: each neighbouring pair is swapped when the later object's key
   is strictly smaller than the earlier one's. Consequences:
   - An object whose key grew can travel any number of places toward the back (drawn later) in one
     tick, but an object whose key shrank moves at most one place toward the front per tick. When an
     object's key drops past the keys of several others in one tick (possible only when their keys
     are closer together than its change of key per tick), it stays drawn over some of them for up to
     one tick per object passed. At ordinary unit speeds against units standing a cell apart, a key
     never passes more than one other key per tick, and the limit is not visible.
   - Objects with equal keys keep their relative order.
   - A reorder caused by movement in tick *n* is first visible in the picture drawn after tick
     *n* + 1: the pass of each tick uses each object's key as it stood in the picture drawn after
     the previous tick, not the key it has after its movement in this tick.
   - Backing: `measurable` (scenario D3)
   - Confidence: high for the insertion and swap rules; medium for the exact one-tick lag
7. **Other layers are never sorted.** Objects entering the underground, surface, air or top layer
   are appended at the end of the list and never reordered. Among aircraft, for instance, the one that
   most recently took off is drawn over the others regardless of screen position.
   - Backing: `measurable` (scenario D5)
   - Confidence: high

### Depth compositing

8. **Depth buffer life cycle.** The depth buffer belongs to the terrain picture and persists from one
   drawn frame to the next. It is filled with 65535 when the battlefield is fully redrawn, and after
   every view jump (Terms). When the view scrolls (moves by at most its width across and at most its
   height up or down), the existing depth values move with the picture and the newly exposed strip
   is filled with 65535 and repainted by the cell pass; depth values stay consistent in world terms
   across scrolls (F1). Areas marked for redraw (terrain or overlay changes
   such as ore growth, bridge destruction and repair, cliff destruction, objects placed on or removed
   from the map, a building changing damage state) are filled with 65535 and repainted by the cell
   pass before the next object pass. Drawings of the object pass never write depth, so moving units
   leave no trace in the depth buffer. Because cell-pass writers store a value only where they pass
   the depth test, a pixel keeps the nearest depth that any writer has given it since its area was
   last cleared.
   A view jump also resets the depth reference *C* of F1. Its visible effect: the rows that a
   rounding-sensitive overlap loses (for example an infantry unit standing on a building shadow) can
   change after a view jump, while scrolls never change them.
   - Backing: `unmeasured` for the buffer itself (internal state; its effects are covered by D6–D8,
     D11 and D15); `measurable` for the reset of *C* at view jumps and its visible effect (scenario
     D20)
   - Confidence: high for the mechanism and for the reset at view jumps; medium for the full list of
     events that mark areas, and for the reset when the depth buffer is created (whether the move
     to the starting position at the start of a game is itself a view jump depends on the map)
9. **Terrain tiles write depth.** Each pixel of a terrain tile gets the depth of F3. A tile pixel is
   drawn, and its depth stored, only when its depth is **less than or equal to** the stored depth.
   Tiles without tile depth data are drawn without touching the depth buffer. Tile depth does not
   depend on the tile's height level: raised terrain is depth-compared as if it lay at level 0, the
   shape of cliffs and slopes being carried entirely by the tile depth data.
   - Backing: `measurable` (scenario D6)
   - Confidence: high
10. **Objects read depth; only some write it.** A sprite or voxel image drawn in the object pass is
    a depth reader: each of its pixels is drawn only if its depth (F2, minus the companion depth image
    value if any) is **strictly less** than the stored depth. Readers do not change the depth buffer.
    The cell pass paints mostly writers, which store their pixel depth where they pass: terrain tiles
    (claim 9), overlays (walls, ore and gems, bridge decks, fences and the like), the building
    images (body with its companion depth image, building shadow, bib, under-door sprites, a gate's
    body), the terrain objects that are not animated (trees and the like) with their shadows, and
    the remembered images of fogged buildings (body, shadow and bib). The remembered images of
    building animations under fog are readers. Everything drawn in the object pass is a reader
    only: vehicles, infantry, aircraft, voxel models (including building turrets), the building
    parts drawn there (including the door sprite), animated terrain objects, projectiles and
    animations. Consequently, units never hide each other through depth; overlap between two units
    is decided by draw order alone, while terrain, overlays, trees and buildings can hide units.
    - Backing: `measurable` (scenarios D6, D7, D15, D17)
    - Confidence: high for tiles, overlays, building bodies, non-animated terrain objects and
      units; medium for the other building images
11. **Depth is anchored to the ground footprint.** Every object's depth bias contains −*L(z)* of its
    own position, which cancels its height lift; a projectile's shadow uses −*L* of the world height of
    the terrain under it instead (claim 13). An object at any height is therefore depth-compared as if it stood at its
    ground reference row: a flying aircraft is hidden only by terrain, overlays or buildings that lie
    in front of the point below it, never by terrain behind that point even though it is drawn over
    that terrain. Overlays use the exact 15 pixels per level instead of *L(z)* (claim 13).
    - Backing: `measurable` (scenario D7)
    - Confidence: high
12. **Depth profile per drawing.** Ground-lying images (projectile sprites and shadows, the other shadows drawn as sprites,
    flat animations, ore and gems, overlays whose type is drawn flat, the building bib, under-door
    sprites and door sprite) use the flat profile; vehicles, infantry, aircraft, voxel models, building
    bodies, terrain objects, walls, overlays whose type is not drawn flat, and ordinary and stacked
    animations use the upright profile. The voxel shadows of vehicles and aircraft are the
    exception among shadows: they use the upright profile (claim 18). The profiles are defined in F2. Overlay types are drawn flat
    unless their type says otherwise (absent-key default: flat), except walls, which are always
    upright.
    - Backing: `measurable` (scenarios D8, D11, D15; voxel shadows D23 (c))
    - Confidence: high for vehicles, infantry, aircraft, buildings, projectiles, overlays, voxel shadows,
      animations and terrain objects (trees)
13. **Depth bias per kind.** The depth bias added to the row depth is given below. **Every value is
    final:** it already includes every constant that applies to that kind of drawing, including the
    −2 that applies to images of vehicles, infantry, aircraft and buildings drawn as sprites. Nothing
    further is added. *U* is the unit depth adjustment of claim 14; *z* is the
    world height of the object's position; all values are integers in depth units.

    | Drawing | Depth bias (final) | Profile | Writes depth |
    |---|---|---|---|
    | vehicle, infantry or aircraft drawn as a sprite | *U* − 2 | upright | no |
    | vehicle or aircraft drawn as a voxel model | *U* (no further constant) | upright | no |
    | voxel shadow of a vehicle or aircraft (claim 18) | *U* of the object casting it (no further constant) | upright | no |
    | infantry shadow | *U* − 2 | flat | no |
    | building body (cell pass) | −*L(z)* − 2 + the building's normal depth adjustment; companion depth image F5 | upright | yes |
    | building body while unloading, drawn as its deploying sprite | −*L(z)* − 22, replacing the normal depth adjustment; companion depth image as for the normal body | upright | yes |
    | building body while unloading, drawn as its roof-deploying sprite | −*L(z)* − 42, replacing the normal depth adjustment; companion depth image as for the normal body | upright | yes |
    | gate body while opening or closing | −*L(z)* − 2 + the normal depth adjustment; no companion depth image | upright | yes |
    | building shadow | −*L(z)* − 4; the normal depth adjustment does not apply (remembered image under fog: −*L(z)* − 2) | flat | yes |
    | building bib (except in the construction state and while a gate opens or closes; remembered image under fog: the same bias) | −*L(z)* − 3 | flat | yes |
    | building under-door sprites (while unloading) | −*L(z)* − 2 | flat | yes |
    | building parts drawn in the object pass as voxel models (turret) | −*L(z)* + the turret slot depth adjustment | upright | no |
    | building parts drawn in the object pass as sprites (not the slot animations, which use the animation rows, claim 17) | −*L(z)* − 2 | upright | no |
    | building door sprite (object pass, while unloading) | −*L(z)* − 7 | flat | no |
    | building body, remembered image under fog | −*L(z)* − 2 + the normal depth adjustment; companion depth image F5 | upright | yes |
    | building animation, remembered image under fog | animation depth adjustment − *L(z)* − 2 (no vertical draw shift, and −2 for flat types too) | flat for flat types, otherwise upright | no |
    | terrain object that is not animated (tree), cell pass | −*L(z)* − 12 | upright | yes |
    | its shadow | −*L(z)* − 3 | flat | yes |
    | animated terrain object (object pass) | −*L(z)* − 12 (its shadow: −*L(z)* − 3, flat) | upright | no |
    | projectile sprite | −30 − *L(z)* | flat | no |
    | projectile shadow (types with a shadow, while the projectile's height above the terrain is greater than 0) | −10 − *L*(world height of the terrain under it) | flat | no |
    | animation, ordinary (also slot animations, claim 17) | type vertical draw shift + animation depth adjustment − *L(z)* − 2 | upright | no |
    | animation, flat | type vertical draw shift + animation depth adjustment − *L(z)* − 3 | flat | no |
    | animation, tiled (stacked) | as ordinary but −50 instead of −2, and each further copy stacked above has its bias reduced by the number of rows it is shifted up | upright | no |
    | animation shadow (types with a shadow) | −2 − *L(z)* | flat | no |
    | animation drawn entirely as a shadow (a per-animation state) | type vertical draw shift − *L(z)* | flat | no |
    | overlay, flat type, ore and gems, wall, rock | −2 − 15 × *ℓ* | see claim 12 | yes |
    | overlay, any other upright type | −17 − 15 × *ℓ* | upright | yes |

    Here *ℓ* is the cell's height level, plus 4 if the cell carries a bridge deck. The vertical
    draw shift is in pixels and counts as depth units here.

    **Which unloading image is drawn.** While a building is unloading, its body is drawn as follows:

    | Unloading unit's type | Art names a roof-deploying sprite | Art names a deploying sprite | Body drawn as |
    |---|---|---|---|
    | jumpjet type or balloon-hover type | yes | any | roof-deploying sprite |
    | jumpjet type or balloon-hover type | no | any | body sprite |
    | neither | any | yes | deploying sprite |
    | neither | any | no | body sprite |

    A body drawn as its body sprite keeps its normal row of the table above. The under-door sprites
    and the door sprite are drawn in addition to the body.

    The animation depth adjustment is the animation's own value: its type's, unless the creator of
    the animation replaced it. A building's slot animation has its slot's depth adjustment instead
    (claim 17).

    The animation's vertical draw shift appears in the bias because shifting an image down by *n*
    rows would otherwise make it *n* depth units nearer; the bias keeps its depth tied to its position.
    - Backing: `measurable`. Coverage by row: sprite units and infantry shadow D8, D15; voxel units
      D7, D9, D10 (relative tests only); voxel shadow D23 (profile; its bias only relative to the
      body's); voxel turret D22 (d), including the absent-key default 0 of the turret slot depth
      adjustment; building body, both unloading bodies, gate body, shadow, bib,
      under-door sprites, non-animated terrain object and its shadow D15; projectile sprite,
      projectile shadow,
      ordinary, flat and tiled animations and animation shadow D15; overlays, wall, rock, ore and gems
      D11, D15. `unmeasured`:
      - building parts drawn as sprites and the door sprite: the only writer they overlap is their
        own body, whose companion depth image has reference points that are not yet settled (F5,
        Open questions).
      - animations drawn entirely as a shadow: the state cannot be set up on purpose.
      - animated terrain objects: no stock terrain type that is animated and stands where a probe
        can reach it has been identified.
      - the two remembered-building rows: a probe unit of the local player standing close enough
        to overlap a fogged building would normally bring it back in sight, so no probe can be
        compared with the remembered body; and the remembered animation is
        tested only against drawings ranked before it, mainly its own body, whose companion depth
        image has reference points that are not yet settled (F5). D17 tests where these drawings rank (claim 16), not their
        biases.
    - Confidence: high for projectile sprites, overlays, animations, building bodies, sprite units,
      the infantry shadow, non-animated terrain objects and their shadows, and the −*L(z)*
      term; medium for voxel units, the voxel shadow, the voxel turret row and its absent-key
      default, building shadow, bib,
      unloading, under-door and gate rows, the choice of unloading image, the remembered building
      rows, animated terrain objects and projectile shadows; low for building parts drawn as
      sprites, the door sprite and animations drawn entirely as a shadow
14. **Unit depth adjustment.** For vehicles, infantry and aircraft,
    *U* = −*L(z)* + *s* + *F*, an integer in depth units, where:
    - *s* is a situational term, −1 in the ordinary case. A few special situations give other values
      (Open questions); this spec does not fix them.
    - *F* is the largest of four terrain fudges *F* = max(*F*<sub>column</sub>, *F*<sub>tunnel</sub>,
      *F*<sub>cliff</sub>, *F*<sub>bridge</sub>), each being the unit type's fudge value (depth units)
      times an integer factor from its surroundings (positive values push the unit farther away, so it
      is hidden sooner):
      - cliff: the factor depends on the cells one step (x + 1, y + 1) and two steps (x + 2, y + 2)
        toward the viewer from the unit's cell (x, y); a cell is *high* when it is more than 3 levels
        higher than the unit's cell.

        | Unit on a bridge | Two-step cell high | One-step cell high | Cliff factor |
        |---|---|---|---|
        | yes | any | any | 0 |
        | no | yes | any | 1 |
        | no | no | yes | 2 |
        | no | no | no | 0 |

      - column: only near a bridge; factor = min(1, *c*(x, y + 1) + *c*(x + 1, y)) + *c*(x + 1, y + 1),
        where *c* of a cell is 1 if it holds a bridge support column tile and 0 otherwise (range 0–2).
      - tunnel: factor 1 next to a tunnel entrance, else 0.
      - bridge: factor 1 when the unit is under or beside a high bridge, else 0.

    Absent fudge values default to cliff 10, column 5, tunnel 10, bridge 0 depth units. Units with
    tunnel movement get a further state-dependent term while digging in or out (Open questions).
    - Backing: `measurable` (scenarios D9, D16; the absent-key defaults D9 run A0 and D16 runs with
      the key absent)
    - Confidence: high for the cliff fudge and the ordinary *s* = −1; medium for column, tunnel and
      bridge detection
15. **Bridges.** A bridge deck is an overlay drawn in the cell pass, lifted 4 levels above
    its cell, with a flat profile and depth written (bias per claim 13). A unit under the bridge is
    drawn later with the depth of its own ground reference row; the deck pixels covering it belong to
    ground rows nearer to the viewer, so the deck hides it. A unit on the deck is lifted by its height
    and has the bias of claim 13, which contains −*L(z)*; the deck pixels behind and around its
    image belong to ground rows behind its ground point, so the unit passes the depth test and appears
    on the deck (its lowest one or two rows can be lost to the deck's −2 bias, the unit's own small
    constants and the rounding of the upright profile). The bridge fudge (claim 14) exists for tall
    units passing under.
    - Backing: `measurable` (scenario D10)
    - Confidence: medium
16. **Precedence inside the cell pass.** Within each area it repaints, the cell pass ranks its
    drawings as follows:

    | Rank | Drawings |
    |---|---|
    | 1 | terrain tiles |
    | 2 | with fog of war on, the remembered images of fogged buildings, one building after another, nearest first (measure below); each building's group ranks its body, shadow and bib before its remembered animations |
    | 3 | overlays with their shadows |
    | 4 | terrain objects that are not animated (trees and the like), with their shadows |
    | 5 | the ground layer's buildings that are not hidden under fog, one building after another; each building's group ranks its body (or the image drawn in its place: a deploying sprite, a roof-deploying sprite or a gate image, claim 13), then its shadow, then its bib, then its under-door sprites |

    **Measure for "nearest first".** For a fogged building whose top-left foundation cell is
    (*x*, *y*), let *N* = 37962·*x* + 37814·*y*, an exact integer. Remembered buildings are ranked
    by decreasing *N*. Two buildings never share a top-left cell, so there are no ties. Because
    *N* = 37814·(*x* + *y*) + 148·*x*, this is the same as ranking by decreasing *x* + *y*, and
    among equal *x* + *y* by decreasing *x*, whenever the two buildings' *x* values differ by less
    than 256 cells.

    Only buildings have remembered images. Terrain objects, overlays and smudges are never
    remembered: wherever they are drawn under fog, they are drawn as they are at that moment, in
    ranks 3 and 4 as outside fog. Which objects stay hidden under fog belongs to the visibility
    specs.

    The ranking is visible through these consequences, and only these:
    - Where a tile pixel and another cell-pass writer give a pixel exactly the same depth, the tile
      shows. Where two other cell-pass writers tie, the one ranked earlier shows. So where a
      building's body and its own shadow, bib or under-door sprite tie, the body pixel shows, and
      where its bib and an under-door sprite tie, the bib pixel shows.
    - The remembered image of a fogged building's animation is a reader. It is tested against the
      depth left by the tiles, by the remembered images ranked before it (its own building's body
      among them) and by nothing else. Any cell-pass writer ranked after it whose pixel passes the
      depth test paints over it, even a writer that stands farther from the viewer than the
      animation. So, where they overlap outside the building's body:
      - a tree standing behind a fogged building appears over the building's remembered animation,
      - the remembered body of a second fogged building that stands farther back (smaller *N*)
        appears over the nearer building's remembered animation, and
      - a live building body (rank 5) standing behind appears over it.

      When the building is not fogged, its animation is an object of the object pass, and it appears
      over all three of these wherever it passes the depth test (quirk
      `quirks.remembered_animation_in_cell_pass`).
    - Apart from ties and these remembered animations, the result does not depend on the ranking,
      because every other cell-pass drawing is a writer that keeps the nearest depth.
    - Backing: `measurable` (scenario D17: trees in (a), a second remembered building in (b)); the
      live-body case is `unmeasured` (D17 (c) gives the reason). The tie rules, and with them the
      order inside a live building's group, are `unmeasured`, because exact depth ties between two
      writers cannot be set up on purpose. That terrain objects, overlays and smudges are never remembered is
      `unmeasured`: it would show when one of them changes inside a fogged area, but whether such a
      change repaints that area at once is not settled (claim 8 lists the redraw events at
      medium confidence), so no expected result can be stated.
    - Confidence: high for tiles first, for buildings being the only remembered objects, for the
      measure *N* and for the order inside a live building's group; medium for the order of the
      later steps

### Building slot animations and voxel shadows

17. **Building slot animations.** The animations a building keeps in its slots (S-0108), its
    turret drawn as a sprite among them, are animation objects of their own. They are created at
    the building's image anchor moved by the slot's displacement (S-0108, claim 10), but they are
    not attached to the building, so each is in the layer of an unattached animation: its type's
    layer (claim 3, *State and ordering*). They are drawn in the object pass as depth readers, with
    the animation rows of claim 13 (ordinary, flat, tiled, shadow), not with the rows for building
    parts. Two values of the slot definition **replace** the animation type's own; they are not
    added to them:
    - the slot's depth adjustment is the animation depth adjustment in its depth bias (claim 13).
    - the slot's y-sort value is the sort adjustment in its sort key (claim 5), so the key is
      X + Y of the animation's position plus the slot's y-sort.

    An absent slot key counts as 0, so a slot animation whose slot sets neither value draws and
    sorts as if its type had no depth adjustment and no sort adjustment, whatever its type says
    (`quirks.slot_values_replace_animation_adjustments`). The slot's values take effect only once the
    animation is in its layer. So a slot animation that enters the ground layer is placed in the
    list (claim 6) by a key with its type's own sort adjustment, and the slot's y-sort counts from
    the next sorting pass on. The turret slot's depth adjustment is also the depth adjustment of a
    voxel turret (claim 13), and the turret slot's keys come from the building's rules section
    (Import mapping).
    - Backing: `measurable` (scenario D22 (a) to (c)). The insertion with the type's own value is
      `unmeasured`: it shows only in the first tick after a slot animation is created, and the
      tick at which a building starts a slot animation is set by the building behaviours of S-0108,
      not by a setup this spec can fix.
    - Confidence: high for the layer rule and for the replacement of both values; medium for the
      insertion with the type's value
18. **Shadows of voxel models.** The voxel shadow of a vehicle or aircraft is drawn as part of that object's
    own drawing in the object pass, so it is drawn in the object's own layer at the object's place
    in that layer's list. It is not drawn in a pass of its own, and an aircraft's shadow is not
    drawn in the ground layer. It is drawn only when the object's movement type gives it a shadow,
    around the shadow draw point that the movement type gives (S-0105 Terms; Open questions). It
    is a depth reader only, with the same depth bias *U* as the object's body (claim 13) and the
    upright profile of F2, anchored at the last row of the shadow image's own copy rectangle, not
    the flat profile of the other shadows. So:
    - an aircraft's shadow, drawn in the top layer, darkens every ground-layer object it falls on,
      and a ground vehicle's shadow darkens the ground-layer objects before the vehicle in the
      ground list and is covered by those after it.
    - where the shadow falls on a writer that is nearer than its depth (a wall, a cliff face, a
      building's body), it is hidden or shown as a whole, like an upright drawing, not cut along a
      fixed row.

    Where an aircraft's shadow and its own body overlap and both pass the depth test, the body is
    shown undarkened: the shadow never paints over its own aircraft. For a vehicle it is not settled
    whether its shadow paints over its own body where they overlap (Open questions).
    - Backing: `measurable` (scenario D23). The bias value itself is tested only relative to the
      body's (both use *U*); whether an aircraft's body shows over its own shadow is `unmeasured`, because an
      aircraft's shadow and its own body overlap only when it is on or very near the ground, where
      the shadow point is not settled.
    - Confidence: high for the drawing inside the object's own drawing (and so its layer and list
      place), the reader-only test, the bias *U* and the upright profile; medium for an aircraft's
      body showing over its own shadow

## Formulas


**F1. Screen-row depth.** Every screen row *r* of the battlefield view has a screen-row depth
*d*(*r*), an integer modulo 65536 that decreases by 1 per row downward. It is tied to the world:

> *d* = (*C* − *w*) mod 65536, where *w* is the world pixel row shown on screen row *r* and *C* is the
> depth reference. All values are integers.

*C* changes only at a **reset**. A reset happens when the depth buffer is created for a game, and at
every view jump (Terms). At a reset, *C* = 32768 + *w*<sub>top</sub>, where *w*<sub>top</sub> is the
world pixel row of the view's top row in the first drawn frame after the reset; that row then has
depth 32768. Scrolls do not change *C*, so a fixed world pixel row keeps its depth while the view
scrolls. A full redraw without a view move (claim 8) does not change *C* either.

*C* matters beyond differences of depth. The rounding Q of F2 acts on absolute depth values, so the
amount it removes depends on *C* mod 3. Results that depend on that amount (the ranges with "*q* in
0…2" below) can therefore change after a view jump, even though nothing on the map has moved. A
reimplementation reproduces this with `quirks.depth_reference_reset_on_jump` on (Edge cases and
quirks).

How long *C* − *w* stays inside 0…65535: it starts at 32768 for the top row after a reset, and the
world pixel rows of a map differ by far less than 32768, so no visible row leaves the range in play.

**F2. Row depth of an object drawing** (integers). Let the drawing occupy screen rows *r*₀ (first)
to *r*₁ (last) of its drawing rectangle, both after clipping to the view, with depth bias *β*. The
drawing rectangle may include transparent rows:
- for a sprite, it is the rectangle stored for that image frame in its file, placed as the sprite
  specs say.
- for a voxel model (vehicle, aircraft or building turret), it is the voxel copy rectangle of
  S-0105 (claim 25, F6): *h* = trunc(*y*<sub>max</sub> − *y*<sub>min</sub>) + 8 rows starting at
  screen row *y*<sub>p</sub> + trunc(*c*<sub>y</sub>) − ⌊*h* / 2⌋, where *y*<sub>p</sub> is the
  draw point's row and *y*<sub>min</sub>, *y*<sub>max</sub>, *c*<sub>y</sub> are the vertical bounds
  and centre of the model's transformed section boxes. Those bounds are floating-point values, as
  S-0105 defines them; *h* and the first row are integers obtained by truncation toward zero, and
  ⌊*h* / 2⌋ is an integer halving of a positive number. So *r*₁ = *r*₀ + *h* − 1 before clipping,
  3 or 4 rows below the lowest projected corner of the boxes (the copy margin); the formula above
  gives the exact row.

  A consequence: a voxel model's upright anchor is not its ground point but the bottom of its
  boxes' projection plus the margin, which moves when the facing, turret facing or tilt changes
  the boxes' projection.

Q(*v*) denotes trunc₀(*v* / 3) × 3 (truncation toward zero; *v* is positive in practice).

- **flat**: depth(*r*) = *d*(*r*) + *β*. It follows the ground row by row.
- **slope**: depth(*r*₀ + *j*) = Q(*d*(*r*₀) + *β*) − ⌊2*j* / 3⌋ for *j* = 0, 1, 2, … (it loses 1 on the
  second and third of every three row steps).
- **upright**: depth(*r*) = Q(*d*(*r*₁) + *β*) − ⌊(*r*₁ − *r*) / 3⌋. The bottom row has the (rounded)
  depth of the ground row it stands on, and the image becomes nearer by 1 for every 3 rows upward.

Pixel depth = row depth − *a*, where *a* is the companion depth image's signed value for that pixel
(0 when there is none). **When a companion depth image is used, the row depth does not step from row
to row:** every row of the drawing uses the anchor value alone, i.e. *d*(*r*₀) + *β* for flat,
Q(*d*(*r*₀) + *β*) for slope, and *d*(*r*₁) + *β* without the rounding Q for upright, so the whole depth
shape comes from the companion image. The pixel is drawn iff pixel depth < stored depth; a writer then
stores the pixel depth, reduced modulo 65536. *d* is reduced modulo 65536 before *β* is added; the
comparison is between plain integers. In play the values never leave 0…65535, so the reduction on
storing has no visible effect.

No stock drawing has been confirmed to use the slope profile; it is listed because the original
supports it.

Note: clipping at the view's bottom edge moves *r*₁, so an upright image partly below the view is
depth-anchored at the view's last row instead of at its own base (quirk
`quirks.upright_depth_anchor_after_clip`).

**Comparing two drawings** (consequences of F2, used by the measurement scenarios; *b* is the last
row *r*₁ of a drawing, *β* its bias, all in integers, ±1 for the rounding Q and the floors; valid only
for drawings without a companion depth image):

- Upright reader over upright writer: the reader's pixels inside the overlap are drawn iff
  (4/3)·(*b*<sub>reader</sub> − *b*<sub>writer</sub>) > *β*<sub>reader</sub> − *β*<sub>writer</sub>. The
  reader is shown or hidden as a whole, not cut along a row, except within 2 rows of the switch
  distance, where single rows of the overlap can differ because of the floors.
- Upright reader standing on a flat writer: the reader loses its bottom *k* rows, *k* being the
  number of row distances *j* = 0, 1, 2, … above its base with *j* + ⌊*j* / 3⌋ ≤ *β*<sub>reader</sub> −
  *β*<sub>writer</sub> − *q*, where *q* in 0…2 is the amount Q removed from the reader's anchor.
- Flat reader over upright writer: the reader is cut along a row; its pixels at distance *j* above the
  writer's last row are hidden iff *j* + ⌊*j* / 3⌋ ≥ *β*<sub>writer</sub> − *β*<sub>reader</sub> − *q*,
  with *q* in 0…2 the amount Q removed from the writer's anchor.

**F3. Terrain tile pixel depth** (integers). For a tile whose ground reference top row is at screen
row *t* (the tile's top row as drawn plus 15 × its height level), and tile depth data value *e* at a
pixel:

> depth = (*e* + *d*(*t* + 30)) mod 65536

Here 30 is a fixed shift of 30 screen rows. The pixel is drawn and stored iff depth ≤ stored depth. (For a flat tile, *e* decreases from top to
bottom so that the result follows *d* row by row.)

**F4. Sort key** (integers, leptons):

> key = X + Y + adjustments (claim 5)

For a building whose top-left foundation cell is (*c<sub>x</sub>*, *c<sub>y</sub>*):
key = 256·*c<sub>x</sub>* + 256·*c<sub>y</sub>* + *t* − *g*, where *t* = 32 if its turret is a voxel
model and 0 otherwise, and *g* = 16 if it is a gate and 0 otherwise.

**F5. Building companion depth image** (integers, pixels). The building body uses one shared depth
image (a game data file) as its companion depth image, the first image in that file, for every
building whose foundation is less than 8 cells wide (wider buildings get none). It is positioned
relative to the body sprite by a displacement of the form

> (*K<sub>x</sub>* + *m<sub>x</sub>* − 30·(*w* − *h*),  *K<sub>y</sub>* + *m<sub>y</sub>* − 15·(*w* + *h* − 2)) pixels

where *w*, *h* are the foundation width and height in cells, (*m<sub>x</sub>*, *m<sub>y</sub>*) is the
building art's depth-image displacement (Import mapping; it is not applied while the building is
being built up or sold), and 30·(*w* − *h*), 15·(*w* + *h* − 2) is the screen displacement of the
foundation's last cell from its first. The engine constants (*K<sub>x</sub>*, *K<sub>y</sub>*) are
(198, 446) for live buildings and (144, 172) for the remembered image of a fogged building; the two
constants are measured from different reference points of the body sprite, and which reference point
each uses (and so the sign convention of the displacement) is not yet confirmed. Low confidence.
This spec owns the question (Open questions). No building's art supplies a depth image of its own:
every building narrower than 8 cells uses the shared image, live and remembered alike (high
confidence).


## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| depth clear value | 65535 | depth units | Fill after a full redraw, after a view jump, and for exposed strips and areas marked for redraw |
| depth reference start | 32768 | depth units | Depth of the view's top row in the first drawn frame after a reset of *C* (F1) |
| depth value range | 0…65535 | depth units | Values wrap modulo 65536 |
| projectile depth bias | −30 | depth units | Plus −*L(z)*; final |
| animation depth bias | −2 (ordinary), −3 (flat), −50 (tiled) | depth units | Plus type and instance adjustments and −*L(z)*; final |
| shadow depth bias | −2 | depth units | Animation shadows; infantry shadows use *U* − 2 |
| building shadow bias | −4 | depth units | Plus −*L(z)* (−2 for remembered fogged images); final |
| terrain object bias | −12 (body), −3 (shadow) | depth units | Plus −*L(z)*; final; non-animated and animated terrain objects alike |
| remembered-building ranking weights | 37962 per cell of *x*, 37814 per cell of *y* | dimensionless, per cell | *N* in claim 16 |
| sprite constant | −2 | depth units | Already contained in the claim 13 rows for vehicles, infantry, aircraft and buildings drawn as sprites; listed for reference only |
| ordinary unit situational term | −1 | depth units | *s* in claim 14 |
| overlay depth bias | −2 − 15 × level (−17 − 15 × level for upright non-wall, non-rock types) | depth units | Level + 4 for bridge decks |
| companion-image width limit | 8 | cells | Foundations this wide or wider get no companion depth image |
| companion-image constants (*K<sub>x</sub>*, *K<sub>y</sub>*) | (198, 446) live, (144, 172) remembered | pixels | F5 |
| voxel-turret sort bonus | +32 | leptons (sort key) | Buildings |
| gate sort penalty | −16 | leptons (sort key) | Buildings |
| slope / upright quantum | 3 | depth units | Rounding of the anchor row depth |
| tile depth row shift | 30 | screen rows | F3 |
| cliff fudge test | more than 3 | height levels | Difference to the cell 1 or 2 steps toward the viewer |
| default fudges | cliff 10, column 5, tunnel 10, bridge 0 | depth units | Absent-key defaults |
| global cruise height default | 400 | leptons | Absent-key default of a game-wide key: it holds only until a layer first sets the key in the session (S-0006, claim 10) |
| slot depth adjustment and y-sort default | 0 | depth units; leptons (sort key) | Absent-key default of every slot of a building type, the turret slot included (claims 13, 17) |
| jumpjet cruise height default | 500 | leptons | Absent-key default of a type's jumpjet cruise height |
| jumpjet cruise height minimum | 208 | leptons | 2 height levels; a smaller configured value is raised to this |
| bridge deck height | 416 | leptons | 4 height levels |
| ground-layer sort passes | 1 per tick | passes | One front-to-back adjacent-swap pass |

## State and ordering

**Layer by object kind and state** (evaluated at the moments listed in claim 4):

| Object kind | Condition | Layer |
|---|---|---|
| vehicle or infantry, walking / driving / hovering / ship / mech / teleport movement | always | ground |
| aircraft or unit with flying movement | height above the terrain > 0 | top |
| aircraft or unit with flying movement | on the ground | ground |
| unit with jumpjet movement | on the ground, or in the air at height 0 | ground |
| unit with jumpjet movement | in the air, height ≥ its jumpjet cruise height | top |
| unit with jumpjet movement | in the air, height below it | air |
| unit with rocket or drop-pod movement | always | air |
| unit with tunnel movement | while digging underground | underground |
| unit with tunnel movement | otherwise | ground |
| building, terrain object, any object kind not listed in this table | not in the air | ground |
| same | in the air, height ≥ global cruise height | top |
| same | in the air, below it | air |
| projectile | image marked flat | surface |
| projectile | otherwise | air |
| animation | attached to an owner object | ground |
| animation | otherwise | its type's layer (default air) |
| particle, voxel animation, wave effect, building light effect | always | air |
| particle system | always | ground |
| veinhole monster | never in any layer; no step of a drawn frame was found to paint it | — |

Heights in this table are heights above the terrain (glossary), in leptons. A unit's jumpjet cruise
height is its type's configured value, raised to 208 leptons if smaller (absent-key default 500); it
is also the height at which a unit with jumpjet movement hovers.

**A unit with jumpjet movement over a high bridge's cell.** The table's comparisons for such a unit
use its effective
height *h*′, where *h* is its height above the terrain (over a high bridge's cell, the ground under
the deck). Both are in leptons; the arithmetic is exact on integers.

> *h*′ = *h* − 416 if the unit is over a high bridge's cell, is not itself on the bridge, is not
> falling, and *h* ≥ 416 (the deck height); otherwise *h*′ = *h*.

So above a deck its layer follows its height above the deck.

**Ground-layer order maintenance:**

| State | Event | Result |
|---|---|---|
| any | object enters the ground layer | placed just before the first object, counting from the front, whose key is strictly greater than its own; at the end if there is none |
| any | tick | one front-to-back pass; swap neighbours where later key < earlier key |
| any | object leaves the layer | removed; the order of the rest is unchanged |

**Painting order of a drawn frame** (each later step can paint over the earlier ones, subject to
the depth test where one applies):

| Step | What is painted |
|---|---|
| 1 | terrain picture (cell pass, claim 16) |
| 2 | ion-cannon blast effects |
| 3 | underground, surface and ground layers, in list order |
| 4 | building information tips |
| 5 | air and top layers, in list order |
| 6 | status extras of all drawn vehicles, infantry, aircraft and buildings |
| 7 | light flashes, beam effects, range circles, map beacons |

**Observable consequences.** A building's body is painted in the cell pass with its depth written,
so whether a unit appears in front of or behind it is decided by depth, not by the unit's place in
the ground list: a unit standing on the foundation's front cells passes the depth test against the
body and is drawn over it, while a unit standing behind the building (map north-west of it, up the screen) fails the depth
test where the body is nearer and appears hidden behind it. The ground-list order relative to the
building matters only for the building parts drawn in the object pass (voxel turrets, the door
image and the like): those are drawn over units earlier in the list and under units later in it.
A building's slot animations, its turret drawn as a sprite among them, are objects of their own
with their own sort keys and layers (claim 17), so their order against a unit does not follow the
building's key. Between two units, the one later in the list is drawn on top wherever they overlap.


## Edge cases and quirks

- **`quirks.ground_layer_single_pass_sort`** (default: on). The ground layer gets one adjacent-swap
  pass per tick (claim 6). Alternative: fully sort the ground layer every tick (stable, by key).
  With the quirk on, fast units moving up the screen can be drawn over units they have already passed
  behind for a few ticks.
- **`quirks.unsorted_upper_layers`** (default: on). Underground, surface, air and top layers keep
  insertion order (claim 7). Alternative: sort them like the ground layer. With the quirk on,
  overlapping aircraft are ordered by take-off time, not by screen position.
- **`quirks.layer_change_on_events_only`** (default: on). Layer membership is updated only at the
  moments of claim 4. Alternative: re-evaluate every tick. Affects, for example, objects whose
  height crosses the cruise height without one of those moments.
- **`quirks.cliff_fudge_far_cliff_weaker`** (default: on). A high cell two steps in front gives the
  cliff fudge factor 1 and overrides a high cell one step in front, which alone gives factor 2.
  Alternative: factor 2 whenever either cell qualifies.
- **`quirks.upright_depth_anchor_after_clip`** (default: on). The upright profile is anchored at the
  last *visible* row (F2). Alternative: anchor at the image's own last row. With the quirk on, the depth
  of tall images cut by the bottom of the view shifts as the view scrolls. For a cell-pass writer the
  depth it was painted with stays in the depth buffer until its area is repainted (claim 8), so a
  reimplementation that rebuilds the terrain picture from scratch in every drawn frame does not
  reproduce the quirk-on result.
- **`quirks.depth_reference_reset_on_jump`** (default: on). The depth reference *C* is reset at every
  view jump and at the creation of the depth buffer (F1), so the amount that the rounding Q removes,
  and with it the rows lost in rounding-sensitive overlaps, can change after a view jump.
  Alternative: one *C* for the whole game that does not depend on the view, for example
  *C* = 32768, so that those overlaps depend only on world positions. D20 tells the two apart.
- **`quirks.remembered_animation_in_cell_pass`** (default: on). A fogged building's remembered
  animations are painted inside the cell pass, ranked as in claim 16, so later cell-pass writers
  (trees, overlays, the remembered bodies of buildings farther back, live building bodies) paint over
  them even when they stand farther from the viewer. Alternative: paint each remembered animation
  after the whole cell pass of its area, as a reader against the finished depth buffer, so that
  under fog it is covered only by drawings nearer than it, as when the building is visible. D17 tells
  the two apart.
- **`quirks.slot_values_replace_animation_adjustments`** (default: on). A building's slot
  animation takes the slot's depth adjustment and y-sort in place of its type's depth adjustment
  and sort adjustment, an absent slot key counting as 0 (claim 17). Alternative: add the slot's
  values to the type's, so that an animation type's own adjustments also apply when a building
  plays it. D22 (b) tells the two apart.
- **`quirks.overlay_lift_exact_levels`** (default: on). Overlay depth uses 15 per level while object
  depth uses −*L(z)*; at levels ≥ 20 the two disagree by a few units (see S-0101 quirk
  `quirks.height_lift_float_scale`). Alternative: use the same lift for both.
- **Depth ties.** A tile pixel replaces a stored value equal to its own (≤), an object pixel does not
  (<). Two units at exactly the same depth do not hide each other in any case (readers only).
- **Depth wrap.** Depth values are taken modulo 65536 before the bias is added; in normal play the
  reference never comes close to wrapping.

## Import mapping

| Source | Key | Feeds | Parse | Default when absent |
|---|---|---|---|---|
| rules, `[JumpjetControls]` | `CruiseHeight` | global cruise height: air/top threshold for objects without own movement rules | integer, leptons | 400, until a rules layer first gives the key in the session; after that a game-wide key keeps the value an earlier game of the session gave it (S-0006, claim 10) |
| rules, unit type | `JumpjetHeight` | jumpjet cruise height of that type (air/top threshold); raised to 208 if smaller | integer, leptons | 500 |
| rules, building type | `Gate` | gate sort penalty applies | boolean | no |
| rules, building type | `TurretAnimIsVoxel` | voxel-turret sort bonus applies; the turret slot depth adjustment is added to the bias of the building's voxel turret | boolean | no |
| rules, building type | `TurretAnimZAdjust`, `TurretAnimYSort` | the turret slot's depth adjustment (also the voxel turret's, claim 13) and y-sort (claim 17) | integer, depth units; integer, leptons | 0; 0 |
| rules, building type | `TurretAnim`, `TurretAnimX`, `TurretAnimY` | the turret slot's animation name and displacement (S-0108); used by D22 | animation name; integers, pixels | S-0108 |
| art, building image | the other slots' `…ZAdjust` and `…YSort` keys (the slot base keys are listed in S-0108), and `PowerUpNLocZZ`, `PowerUpNYSort` for the upgrade slots (digit N in place of the letter) | the slot's depth adjustment and y-sort, replacing the animation type's values (claim 17) | integer, depth units; integer, leptons | 0; 0 |
| art, building image | `NormalZAdjust` | building's normal depth adjustment (body in the cell pass) | integer, depth units | 0 |
| art, building image | `ZShapePointMove` | depth-image displacement (*m<sub>x</sub>*, *m<sub>y</sub>*) in F5 | two integers, pixels | 0, 0 |
| art, building image | `SpecialZOverlay`, `SpecialZOverlayZAdjust` | an image and an adjustment that are read but have no known effect on drawing: no drawing of this image was found | image name; integer | none; 0 |
| art, building image | `BibShape` | the building's bib | image name | none (no bib) |
| art, building image | `DeployingAnim` | the deploying sprite | image name | none |
| art, building image | `RoofDeployingAnim` | the roof-deploying sprite | image name | none |
| art, building image | `UnderDoorAnim`, `UnderRoofDoorAnim` | the under-door sprites | image name | none |
| art, building image | `DoorAnim` | the door sprite | image name | none |
| rules, infantry or vehicle type | `JumpJet` | jumpjet type (glossary); selects the roof-deploying sprite when such a unit is unloaded (claim 13). The layer rules do not read it: they follow the movement type | boolean | no |
| rules, infantry or vehicle type | `BalloonHover` | balloon-hover type; selects the roof-deploying sprite when such a unit is unloaded | boolean | no |
| rules, vehicle, infantry or aircraft type | `Locomotor` | movement type (locomotor); used by D18 to give a type jumpjet movement | locomotor identifier | not used here (movement specs) |
| rules, vehicle, infantry or aircraft type | `ZFudgeCliff`, `ZFudgeColumn`, `ZFudgeTunnel`, `ZFudgeBridge` | the four terrain fudge values | integer, depth units | 10, 5, 10, 0 |
| art, animation | `Layer` | layer of unattached animations | one of `underground`, `surface`, `ground`, `air`, `top` | air |
| art, animation | `YSortAdjust` | animation sort adjustment (copied to each animation; a building's slot animation has its slot's y-sort instead, claim 17) | integer, leptons | 0 |
| art, animation | `ZAdjust` | animation depth adjustment (copied to each animation; the creator may replace it, and a building's slot animation has its slot's depth adjustment instead, claim 17) | integer, depth units | 0 |
| art, animation | `YDrawOffset` | vertical draw shift (also enters the depth bias) | integer, pixels | 0 |
| art, animation | `Flat` | flat profile and −3 bias | boolean | no |
| art, animation | `Tiled` | stacked drawing with −50 bias | boolean | no |
| art, animation | `Shadow` | shadow drawn with −2 bias | boolean | no |
| art, projectile image | `Flat` | projectile goes to the surface layer | boolean | no |
| art, projectile image | `Shadow` | projectile shadow drawn while the projectile's height above the terrain is greater than 0 | boolean | yes |
| rules, overlay type | `DrawFlat` | overlay drawn with the flat profile and −2 bias; otherwise upright | boolean | yes |
| rules, overlay type | `IsARock` | an upright overlay keeps the −2 bias instead of −17; rocks next to a sloped cell change the unit term *s* | boolean | no |
| rules, overlay type | `Wall` | overlay drawn upright with −2 bias regardless of the flat setting | boolean | no |
| rules, overlay type | `Tiberium` | ore and gems: flat profile, −2 bias | boolean | no |

## Measurement scenarios

**Common setup.** The measurement setup of the glossary: an unmodified install with no loose
override files and no engine add-ons, stock data, purpose-made test maps. Where a scenario changes a
type's field, it does so through the test map's override section (test data, removed afterwards).
Screen resolution 800×600, any game speed, fog of war off unless stated. The *test house* is the
local player, whose view the captures show. The observable is read from
screenshots or a capture of the battlefield view, one image per drawn frame; a unit's position is
read from its cell and the ticks it has moved at a known speed. Where a scenario counts ticks, each captured
picture is matched to its tick by capturing at a game speed at which the number of pictures over a
long interval equals the number of ticks (see the glossary's open question on drawn frames per tick).
Every scenario is deterministic: one run per variant is enough unless stated. Within a run, after
the view has been brought to the test area, the view moves only by scrolls (Terms), never by a view
jump, so that the depth reference *C* (F1) stays fixed; D20 is the only scenario that jumps on
purpose. Rows and row distances are screen pixel rows; *b* is the
last row of a drawing's rectangle as in F2 (for a sprite, read from the image's stored rectangle in its file).

- **D1, status extras above all layers (claim 1).** Setup: a flat map; a damaged vehicle,
  selected, standing still; an aircraft flying across it so that the aircraft's image overlaps the
  vehicle's health bar. Observable: in the drawn frames where they overlap, whether the health bar or
  the aircraft is visible in the shared pixels. Expected: the health bar is drawn over the aircraft.
  Falsified if the aircraft covers any part of the health bar.
- **D2, layer membership (claim 3).** Setup: a flat map. (a) A jumpjet infantry hovering below
  its cruise height (climbing, or a type whose cruise height is set high through the override
  section) over a vehicle on the ground, and an aircraft flying across both. (b) A projectile whose
  image is marked flat, and one whose image is not, each passing over a vehicle on the ground (not
  measurable on stock data if no stock projectile image is marked flat). Observable: which image is
  visible where they overlap. Expected: (a) the aircraft (top) covers the jumpjet and the vehicle.
  The jumpjet (air) covers the vehicle wherever they overlap, whatever their screen positions.
  (b) The flat projectile (surface) is covered by the vehicle; the other projectile (air) covers the
  vehicle. Falsified if the vehicle is drawn over the hovering jumpjet, if the jumpjet is drawn over
  the flying aircraft, or if the flat projectile is drawn over the vehicle.
- **D3, ground-layer sorting (claim 6).** Setup: a flat map. (a) Two identical vehicles on
  neighbouring cell columns (screen columns 30 pixels apart), the first starting two cells higher on
  screen; order the first to drive straight down-screen past the second so that their images overlap.
  (b) Five vehicles standing on consecutive cells of one map row, (x, y) to (x + 4, y), so their keys
  are 256 apart; a sixth vehicle drives along the neighbouring row from (x + 5, y − 1) toward
  (x, y − 1), its image overlapping each standing vehicle in turn. Its speed field is set through the
  override section so that its key drops by more than 512 leptons per tick (not measurable if the
  movement rules cannot reach that speed); a second run uses an ordinary speed. Observable: per
  picture, which vehicle is on top in each overlap, and per tick the mover's key (X + Y) computed from
  its position. Expected: (a) the mover is drawn under the other until its key exceeds the other's,
  then on top, the change being visible one tick after the tick in which its key passes (claim 6).
  (b) Fast run: after a tick in which the mover's key drops past the keys of *m* ≥ 2 standing
  vehicles, the mover goes under them one at a time, one per tick, so for *m* − 1 ticks it is still
  drawn over some of them. Ordinary run: the mover goes under each standing vehicle one tick after its
  key drops below that vehicle's key. Falsified if in the fast run the mover is under all *m* vehicles
  in the second picture after that tick (the outcome of a full sort), or if in (a) the mover goes on
  top before its key exceeds the other's.
- **D4, building keys (claim 5).** Setup: a flat map; a building with a voxel turret whose turret
  image overlaps the route below, and a vehicle driving straight down the column of cells just west of
  the foundation (cells with x one less than the foundation's first column), from north of the
  building to south of it. A turret drawn as a sprite is not a building part but a slot animation
  with its own key (claim 17), tested by D22. The gate term is not tested (claim 5). Observable:
  the vehicle's key (X + Y, from its position) at the tick from which it is drawn over the building's
  voxel turret. Expected: the switch happens at the first tick at which the vehicle's key is strictly
  greater than the building's key of F4, i.e. 256·(*c<sub>x</sub>* + *c<sub>y</sub>*) + 32, and
  becomes visible one tick later (claim 6).

  A vehicle with key *K* where 256·(*c<sub>x</sub>* + *c<sub>y</sub>*) ≤ *K* < 256·(*c<sub>x</sub>* +
  *c<sub>y</sub>*) + 32 is drawn under the voxel turret. Falsified if the switch happens near the
  key of the foundation's centre (256·(*c<sub>x</sub>* + *c<sub>y</sub>*) + 128·(*w* + *h*)) instead.
- **D5, air/top insertion order (claim 7).** Setup: two aircraft of the same type take off one
  after the other and fly crossing paths. Observable: which aircraft is on top where they overlap, and
  which is lower on screen. Expected: the one that took off later is on top, whichever is lower on
  screen. Falsified if the one that took off earlier is drawn on top in any overlap. If the order
  changes during straight flight with no take-off or landing, flight moves re-place aircraft (Open
  questions, layer re-evaluation in flight).
- **D6, tile depth (claims 9, 10).** Setup: a test map with a cliff more than 4 levels high
  facing the camera. (a) A vehicle driving along the low ground behind (map north-west of, up the screen from) the high ground's
  cliff top. (b) A vehicle driving along the cliff top. Observable: which of the vehicle's pixels are
  missing. Expected: (a) the vehicle's pixels disappear exactly where the cliff tile's image covers
  them; (b) the vehicle on the cliff top is never hidden by the lower terrain in front of the cliff.
  Falsified if in (a) the vehicle shows over the cliff face, or if in (b) any lower terrain covers it.
- **D7, depth anchoring (claims 10, 11, 13).** Setup: an aircraft flying north over a cliff wall
  that faces the camera. Observable: in which drawn frames the aircraft's pixels are hidden by the
  cliff, and the position of the point below it. Expected: the aircraft is hidden only while the
  point below it is behind the cliff edge and the cliff image overlaps the aircraft's image; it is
  never hidden by terrain behind its ground point. Falsified if it is hidden while the point below it
  is still in front of the cliff edge, or by terrain behind that point.
- **D8, flat against upright profile (claim 12).** Setup: a flat map with a wall segment; an
  infantry unit walks straight toward the wall from behind (map north-west of, up the screen from) it, then past it, one step at a
  time. Observable: for each position, which rows of the infantry body and of its shadow are hidden
  inside the wall's image. Expected: the infantry body (upright) inside the overlap is either hidden
  as a whole or shown as a whole (allow rows to differ only at the one or two positions within one
  depth unit of the switch, Comparing two drawings); its shadow (flat) is cut along a fixed row, at or
  one row above the wall's last row, whatever the infantry's position, because a flat drawing's depth
  depends only on the screen row. Falsified if a cut row sweeps up the infantry body over several
  positions, or if the shadow's cut row moves with the infantry.
- **D9, cliff fudge (claim 14).** Setup: three test maps, each with a parking cell (x, y) on level
  0 and a vehicle type whose cliff fudge is set through the override section; all other fudges 0.
  Map A: the cell (x + 1, y + 1) is more than 3 levels higher than the parking cell, the cell
  (x + 2, y + 2) is **not** (a one-cell-deep ridge). Map B: both (x + 1, y + 1) and (x + 2, y + 2) are
  more than 3 levels higher (a plateau). Maps A and B use the same tiles at the parking cell and at
  (x + 1, y + 1), and with the fudge set to 0 the vehicle is hidden by the same rows in both (so only
  the face of (x + 1, y + 1) covers it). Map C: the parking cell is on a bridge deck near the end
  of a bridge that crosses a valley toward the viewer, and the bank cells (x + 1, y + 1) and
  (x + 2, y + 2) are more than 3 levels higher than the cell under the deck. Runs: map A with fudge 20,
  map B with fudge 20 and 40, map C with fudge 0 and 40, and run A0: map A with the cliff fudge key
  absent (the stock rules set it for no type; the other three fudges are still set to 0 through the
  override section). Observable: the rows of the vehicle hidden by the terrain in front, per run.
  Expected: map A with 20 pushes the vehicle 40 depth units farther, the same as map B with 40, so
  those two runs hide the same rows, and more than map B with 20. Run A0 uses the default 10, so it
  pushes the vehicle 2 × 10 = 20 depth units farther and hides the same rows as map B with 20. Map C
  hides the same rows with 0 and 40. Falsified if map A with 20 and map B with 40 differ by more than
  one row, if map A with 20 equals map B with 20 while map B with 40 hides more (the alternative of
  `quirks.cliff_fudge_far_cliff_weaker`), if run A0 differs from map B with 20 by more than one row,
  or if map C changes with the fudge.
- **D10, bridges (claim 15).** Setup: a high bridge; a vehicle driving under it while another
  vehicle crosses on top. Observable: hidden pixels of each vehicle. Expected: the lower vehicle is
  hidden by the deck where they overlap; the upper vehicle is visible on the deck except at most its
  lowest two rows. Falsified if the lower vehicle shows through the deck, or the upper vehicle loses
  more than its lowest two rows to the deck.
- **D11, overlay biases (claims 12, 13).** Setup: a flat map with a wall segment, and the same map
  with an overlay type that is neither drawn flat nor a wall nor a rock (if no stock type qualifies,
  one set through the override section). An infantry unit walks straight toward the viewer from
  behind each overlay to in front of it, one step at a time. Observable: for each overlay, the row
  distance *b*<sub>infantry</sub> − *b*<sub>overlay</sub> at which the infantry stops being hidden by
  the overlay's image. Expected (Comparing two drawings, upright over upright): the switch for the
  other overlay type comes about (3/4) × 15 ≈ 11 rows farther toward the viewer than for the wall,
  because its bias is 15 depth units nearer. Falsified if the two switch distances differ by less
  than 8 or more than 15 rows.
- **D12, jumpjet threshold (claims 3, 4).** Setup: a flat map; a ground vehicle standing still,
  a jumpjet infantry that lifts off from a cell next to the vehicle, climbs to its cruise height and
  then hovers there with its image overlapping the vehicle's; an aircraft flying across both; and a
  second jumpjet, of a type whose cruise height is set high through the override section, still
  climbing (air layer) where its image overlaps the first jumpjet's. Observable: per drawn frame,
  which image is on top where they overlap, and whether the first jumpjet is climbing or hovering.
  Expected: while climbing, the first jumpjet is in the air layer: it covers the ground vehicle and
  is covered by the aircraft. Hovering at cruise height it is in the top layer, ordered against the
  aircraft by insertion order (claim 7), and drawn over the climbing second jumpjet. Falsified if
  the climbing jumpjet is drawn over the flying aircraft, or if the hovering jumpjet is covered by
  the climbing second jumpjet. If instead the first jumpjet stays under the ground vehicle throughout
  its climb, its layer is evaluated only at lift-off, at height 0, where it evaluates to ground. That
  answers the open question on layer re-evaluation in flight, and claims 3 and 4 then get a new
  revision.
- **D13, effects and status extras (claim 2).** Setup: a flat map; a selected, damaged vehicle
  standing still (the *target vehicle*) and an aircraft that repeatedly flies across it. Each part
  adds one effect so that it overlaps both the target vehicle's health bar and the aircraft's path.
  Observable: in the shared pixels, which is visible. Common falsifier, for every effect except the
  information tip of (a): the part's effect is covered by the aircraft's image or by the health bar
  in any shared pixel.
  - (a) Map beacon and information tip. A map beacon placed so that its marker overlaps the health
    bar; a building showing its information tip, with a vehicle driving past it on the ground and
    the aircraft flying across it. Expected: the beacon marker is drawn over the aircraft and over
    the health bar; the information tip is drawn over the ground vehicle's image, under the aircraft
    and under the ground vehicle's status extras. Falsified also if the ground vehicle's image
    covers the information tip, or if the information tip is drawn over the aircraft's image or over
    the ground vehicle's status extras.
  - (b) Beam effects. In turn, a stock weapon whose shot is drawn as a laser beam, one drawn as an
    electric bolt and one drawn as a radiation beam, each fired by a unit or building of the test
    house at a target placed so that the beam crosses the health bar and the aircraft's path
    (choose the weapons by reading the stock rules and art; copy no value). Expected: each beam is
    drawn over the aircraft and over the health bar. Not measurable for a kind of beam that no stock
    weapon draws.
  - (c) Range circle. A building of a type that shows a circle of its range on the ground while it
    is selected, if stock data has one, placed so that the circle crosses the health bar (select the
    building and the target vehicle together). Expected: the circle is drawn over the aircraft and
    over the health bar. Not measurable if no stock building shows such a circle.
  - (d) Light flashes. Detail level 2 (high). A stock weapon whose warhead makes a light
    flash where it hits, fired at the ground beside the target vehicle so that the flash covers the
    health bar while the aircraft passes. Expected: the flash brightens the aircraft's pixels and
    the health bar's pixels where they overlap it, in the same way as the bare ground. Falsified if
    the aircraft or the health bar shows unbrightened inside the flash. Not measurable if no stock
    warhead makes a flash.

  Ion-cannon blast effects are not tested (claim 2).
- **D14, layer changes at take-off, landing and attachment (claims 3, 4).** Setup: a flat map.
  (a, b) An aircraft on its landing pad, and a vehicle (ground layer) that stands so that its image overlaps the aircraft's
  and that comes later in the ground list (larger key). The aircraft takes off, circles, and lands
  again on the same pad. Observable: which image is on top in each drawn frame, and whether the
  aircraft is on the ground or in the air. Expected: on the ground the aircraft is under the vehicle.
  From the take-off it is on top of the vehicle, until it has landed again, when it goes under the
  vehicle. Falsified if the aircraft is drawn under the vehicle while clearly in the air, or on top of
  it after landing has finished. (c) Attachment: an animation attached to a ground vehicle (for
  example damage smoke or fire), of a type whose own layer in the art data is air or absent, and a
  jumpjet hovering below its cruise height over the vehicle. Expected: the attached animation (ground
  layer) is covered by the jumpjet (air layer) where they overlap. Falsified if the attached
  animation is drawn over the jumpjet.
- **D15, depth biases by comparison (claims 10, 12, 13).** Setup: a flat map at level 0; an
  infantry unit as probe (sprite, upright, bias *U* − 2 = −3 with all fudge factors 0 and *s* = −1),
  and animation types chosen with no vertical draw shift and no depth adjustment in their art. Each case
  is predicted with *Comparing two drawings* (F2) and the final biases of claim 13; the
  observable and the predicted value are given per case. Rows are counted from the bottom of the
  probe's drawing rectangle (F2).
  - Flat writers under the probe (the probe stands on the image, away from any body), observable: the
    number of the probe's bottom rows lost. Expected: the larger of the bare-ground count (the same
    probe on bare ground beside the image) and the count predicted for an upright reader on a flat
    writer: building shadow (−4) 0 to 2 rows, bib (−3) and terrain-object shadow (−3) 0 or 1,
    under-door sprites (−2) and ore and gems (−2) none.
  - Upright writers in front of the probe (the probe walks toward the viewer past the image, one step
    at a time), observable: the switch distance *b*<sub>probe</sub> − *b*<sub>image</sub> at which the
    probe stops being hidden (all of its rows in the overlap shown). Expected: wall, rock and gate
    body while opening (−2, for a type whose normal depth adjustment is absent) 0 to 2 rows; a
    terrain object that is not animated (−12) 7 to 9 rows; a non-flat overlay that is neither wall
    nor rock (−17) 10 to 12 rows (D11). Building body with a companion depth image:
    only the relative result is predicted, for a type whose normal depth adjustment is absent. While
    it unloads a unit of an ordinary type, the body drawn as its deploying sprite (−22) switches about
    15 rows farther toward the viewer than the same body in its normal state (−2). While it unloads a
    unit of a jumpjet type, the body drawn as its roof-deploying sprite (−42) switches about 30 rows
    farther (each only for a building whose art names that image).
  - Upright readers behind a wall overlay (−2), observable: the smallest distance
    *b*<sub>reader</sub> − *b*<sub>wall</sub> at which the reader shows over the wall. Expected: an
    ordinary animation (−2) shows from 1 to 3 rows; a tiled animation (−50) shows already from −35
    to −33 rows, i.e. while its last row is up to 33 to 35 rows above the wall's.
  - Flat readers passing behind a wall overlay (−2), observable: the row distance above the wall's
    last row from which the reader is hidden. Expected: projectile at height 0 (−30) 20 or 21 rows,
    about (3/4)·*L(z)* rows more at height *z*; projectile shadow (−10, over ground of level 0) 5 or 6
    rows; flat animation (−3) and infantry shadow (−3) 0 or 1 row; animation shadow (−2) 0 rows.

  Falsified if any observed count or distance lies outside the range the formula gives for *q* in
  0…2 (the ranges above), allowing one more row where the reader's or writer's image is transparent
  at the predicted row.
- **D16, column, bridge and tunnel fudges (claim 14).** Setup: test maps with (a) a vehicle beside
  a bridge support column, on the cells listed for the column factor, (b) a vehicle under a high
  bridge, (c) a vehicle next to a tunnel entrance (not measurable on stock data if the theater has
  none). Use a vehicle type for which the stock rules set none of the four fudge keys (check by
  reading the stock rules; copy no value). Runs per case: the fudge of that case set through the
  override section to 0, to 40, and to its absent-key default (column 5, bridge 0, tunnel 10), and a
  run with that key left absent; the other three fudges are set to 0 in every run. Observable: the
  number of the vehicle's rows hidden, per run. Expected: more rows hidden with 40 than with 0 in
  each case; the run with the key absent hides the same rows as the run with the default set
  explicitly; for column and tunnel, the absent run hides more rows than the run with 0, and for
  the bridge it hides the same rows as with 0. In (a), a cell layout with column tiles at both
  (x, y + 1) and (x + 1, y) hides the same as one with only one of them. Falsified if 40 makes no
  difference, if the absent run differs from the explicit-default run by more than one row, or if
  in (a) the two-tile layout hides more than the one-tile layout.
- **D17, remembered animations inside the cell pass (claim 16).** Setup: fog of war on. The
  buildings below belong to a second house and have an animation that shows above their body
  silhouette. A unit of the test house first sees them and then moves away, so that fog of war
  covers them; their remembered images are then viewed. Each part is viewed once while visible and
  once under fog. Observable: in the shared pixels, which is visible.
  - (a) A tree standing behind such a building (map north-west of it, up the screen), its image
    overlapping the animation above the body. Expected: under fog, the tree is drawn over the
    animation wherever they overlap. While visible, the result follows *Comparing two drawings*
    (upright over upright) with the animation's bias from claim 13 and the tree's −12: for an
    animation with no vertical draw shift and no depth adjustment (−2), the animation shows over the
    tree when its last row is 8 to 10 rows lower on screen than the tree's or more, so choose a
    layout with a margin of at least 15 rows. Falsified if under fog the animation shows over the
    tree, or if in that layout the visible animation is hidden by the tree.
  - (b) Two such buildings: a near one, and a far one whose top-left foundation cell is smaller in
    both *x* and *y* (so it ranks later, claim 16), with the far body's image overlapping the
    near building's animation above the near body. Choose a layout in which, while both are
    visible, the animation shows over the far body in the shared pixels. Expected: under fog, the far
    building's remembered body is drawn over the near building's remembered animation there.
    Falsified if under fog the animation still shows over the far body. That outcome would also fit
    a far-first ranking of remembered buildings, or the alternative of
    `quirks.remembered_animation_in_cell_pass`.
  - (c) A live building body behind a fogged building's animation is not tested: no layout is known
    in which a building that the local player sees stands that close behind a building it does not
    see.
- **D18, jumpjet cruise height minimum and default (claim 3, *State and ordering*).** Setup: a
  flat map at level 0, one run per part, each with one jumpjet hovering still over a flat cell after
  its climb:
  - run A: a jumpjet type whose cruise height is set to 100 leptons through the override section.
  - run C: the same type, in a separate game, with 300 leptons.
  - run B: a type that sets no jumpjet cruise height in the stock rules (check by reading the stock
    rules; copy no value), given jumpjet movement and the jumpjet flag through the override section.
    If no such type can be made to fly this way, run B is not measurable.

  Observable: the hover lift, measured on something that moves with the unit's draw point whatever
  image frame is shown: the screen row of the unit's health bar (select the unit) while it hovers,
  minus the same row while it stands on the same cell. Infantry jumpjets use different image
  sequences in the air and on the ground, so the body sprite itself is not compared. Before relying
  on it, confirm in a control run that the health bar is drawn at the lifted position: it must move
  up when the unit lifts off. If the hover moves up and down, record the mean over at least 100
  drawn frames. Expected (lifts from S-0101 F2): A hovers at 208 leptons, a lift of *L*(208) = 30
  rows; C at 300 leptons, 43 rows; B at 500 leptons, 72 rows; each within ±2 rows. Falsified if A's
  lift is near *L*(100) = 14 rows (no minimum), or if B's lift is near *L*(400) = 57 rows (the
  global cruise height used instead of the type's default), or more than 2 rows from 72. That the
  same value is also the air/top threshold is tested by D12.
- **D19, remaining layer rows (claim 3).** Setup: a flat map with a ground vehicle standing still
  and an aircraft that repeatedly flies across it. Each part places one more drawing so that its
  image overlaps both the vehicle and the aircraft's path. Observable: in the shared pixels, which
  image is on top. A drawing in the air layer must cover the vehicle (ground) and be covered by the
  flying aircraft (top), whatever their screen positions.
  - (a) Rocket movement: a unit with rocket movement in flight, launched by a weapon that fires one.
    Expected: air. Falsified if the vehicle covers it or it covers the flying aircraft.
  - (b) Unattached animations: animations created at a ground point and not attached to an object,
    for example by a trigger of the test map that plays an animation at a waypoint, or a weapon's
    impact animation. Use one type whose art sets its layer to surface (chosen from the stock art
    data) and one whose art sets no layer. Expected: the surface type is covered by the vehicle and
    by the aircraft; the type with no layer set is in the air layer. Falsified if the surface type
    covers the vehicle, or if the type with no layer set is covered by the vehicle. If no stock
    animation type sets its layer to surface, test another value the stock art uses, with its own
    expected order.
  - (c) Particles, voxel animations and wave effects: the particles of a particle system created by
    a weapon or warhead (not the smoke or fire animations attached to a damaged object, which are
    attached animations in the ground layer, D14 (c)), debris voxel animations thrown when a vehicle
    is destroyed, and the wave effect of a weapon whose shot is drawn as a wave. Expected: air, for each. Falsified if, for any of them, the vehicle covers it
    or it covers the flying aircraft.
  - (d) Ground layer for other movement types: in place of the standing vehicle, in turn, a hovering
    vehicle, a ship on water, a unit with mech movement and a unit with teleport movement after its
    jump (each only if stock data has a type with that movement). Over each, a projectile whose image
    is not marked flat (air layer, D2). Expected: the projectile covers each of them, and each is
    covered by the flying aircraft. Falsified if any of them covers the projectile.

- **D20, depth reference reset at view jumps (claim 8, F1).** Setup: a flat map at level 0; a
  building of the second house whose shadow lies on open ground; an infantry unit of the test house
  as probe (bias −3, as in D15), standing still on the building's shadow, away from the body, so
  that its bottom rows overlap the shadow (flat writer, −4). Record only drawn frames in which the
  probe shows the same image frame. Let *s* be the screen row of the probe's last rectangle row,
  counted from the battlefield view's top row (0), in the first drawn frame after a view jump.
  Procedure: (i) scroll the view by single small steps (scrolls only) through at least six
  positions while the probe stays in view; (ii) twelve times, make a view jump to a distant part
  of the map, then a view jump back that brings the probe into view. The jump back must land at
  varying pixel positions: a jump to a cell (for example a click on the radar map) may always land
  with the same top row mod 3, so use a jump whose landing follows a unit's exact position, such as
  centring the view on a selected vehicle that drives slowly to and fro near the probe, and record
  *s* after each landing. Observable: the
  number *k* of the probe's bottom rows lost, after each scroll step and after each jump back,
  with *s* for each jump back. Expected (F1, F2 with *C* = 32768 + the world pixel row of the view's
  top row after the jump): in (i), *k* never changes; in (ii), *k* = 2 − ((2 − *s*) mod 3), so *k*
  is 2, 1 or 0, the same for every jump with the same *s* mod 3, and different between the three
  residues (*k* may be larger where the probe on bare ground beside the shadow loses more, D15).
  Falsified if *k* is the same after every jump although *s* mod 3 took all three values (the
  alternative of `quirks.depth_reference_reset_on_jump`), if *k* changes during the scrolls of (i),
  or if two jumps with the same *s* mod 3 give different *k*. Inconclusive, not falsified, if fewer
  than three values of *s* mod 3 occur in the twelve jumps; then repeat with another way of
  jumping.
- **D21, animation sort adjustment (claim 5).** Setup: a flat map; a waypoint *W*; two stock
  animation types that the stock art puts in the ground layer: type 1 with a nonzero sort adjustment
  *A* whose magnitude is at least 64 and less than 256 leptons, so that the switch below happens
  while the images overlap, and type 0 with no sort adjustment (read both from the stock art at
  test time; copy no value). Two runs, one per type: a trigger of the test map plays the animation,
  not attached to any object, at *W*, and plays it again whenever it ends if the type does not
  repeat. In each run a vehicle of the same type drives the same straight route across *W*'s cell
  in the direction of increasing X + Y, its image overlapping the animation's. Observable: per tick,
  the vehicle's key (X + Y from its position), and the key *K*<sub>0</sub> or *K*<sub>1</sub> at the
  first tick from whose following picture on the vehicle is drawn over the animation. Expected:
  *K*<sub>1</sub> − *K*<sub>0</sub> = *A*, within one tick's movement of the vehicle (claims 5 and
  6; the animation's own position cancels). Falsified if *K*<sub>1</sub> and *K*<sub>0</sub> are equal
  within one tick's movement, or differ by more than one tick's movement from *A*. Not measurable if
  no two such stock animation types can be played this way; then the claim falls back to
  `unmeasured`.
- **D22, building slot animations and turret depth adjustments (claims 13, 17).** Setup: a flat map
  at level 0. Parts (a) to (c) use a stock building type whose turret is drawn as a sprite and whose
  turret animation's type the stock art puts in the ground layer (choose both by reading the stock
  rules and art; copy no value; not measurable if there is none). Part (d) uses a stock building
  type whose turret is a voxel model (not measurable if there is none). The turret slot's keys are
  rules keys, so every value below is set through the test map's override section. A slot key
  that a run leaves absent is absent from the override section and from the stock rules of that
  type (check by reading; otherwise that run is not possible).
  - (a) Slot y-sort. A vehicle drives in a straight line in the direction of increasing X + Y so
    that its image passes over the turret animation's image. Runs: the turret slot's y-sort set to
    0, and set to *Y* with 64 ≤ *Y* < 256 leptons. Observable: per tick the vehicle's key (X + Y
    from its position), and the key *K*<sub>0</sub> or *K*<sub>*Y*</sub> at the first tick from
    whose following picture on the vehicle is drawn over the turret animation. Expected:
    *K*<sub>*Y*</sub> − *K*<sub>0</sub> = *Y*, within one tick's movement of the vehicle.
    Falsified if the two keys are equal within one tick's movement, or differ from *Y* by more.
  - (b) Replacement, not addition. Only if the turret animation's type has a nonzero sort
    adjustment *A* in the stock art (read it; copy no value); otherwise not measurable. The turret
    slot's displacement is set to 0, 0 and its y-sort is left absent, so the animation stands at
    the building's image anchor, whose X + Y is 256·(*c<sub>x</sub>* + *c<sub>y</sub>*) for the
    top-left foundation cell (*c<sub>x</sub>*, *c<sub>y</sub>*). The vehicle drives as in (a), on a
    route on which its image overlaps the animation's. Expected: the vehicle is drawn over the
    animation from the first tick at which its key is strictly greater than
    256·(*c<sub>x</sub>* + *c<sub>y</sub>*), visible one tick later. Falsified if the switch
    happens near 256·(*c<sub>x</sub>* + *c<sub>y</sub>*) + *A* instead (the alternative of
    `quirks.slot_values_replace_animation_adjustments`).
  - (c) Slot depth adjustment. A wall segment placed so that its image overlaps part of the turret
    animation's image outside the building's body, in a layout where
    |(4/3)·(*b*<sub>animation</sub> − *b*<sub>wall</sub>) − *s*| < 40 rows, *s* being the
    animation type's vertical draw shift (from the stock art; set the slot displacement through
    the override section to reach such a layout). Runs: the turret slot's depth adjustment left
    absent, set to 0, set to −60 and set to +60. Observable: which of the animation's pixels inside
    the wall's image are visible. Expected (Comparing two drawings, upright over upright): with
    −60 the animation is shown in the overlap and with +60 hidden; the run with the key absent is
    identical to the run with 0. Falsified if −60 and +60 give the same result, or if the absent
    run differs from the run with 0.
  - (d) Turret drawn as a voxel model. A wall segment placed so that its image overlaps part of the voxel turret's
    image outside the building's body. Runs: the turret slot's depth adjustment left absent, set to
    0, set to −60 and set to +60. Observable: which of the turret's pixels inside the wall's image
    are visible. Expected: −60 shows at least the pixels that 0 shows, +60 at most those, and −60
    and +60 differ; the absent run is identical to the run with 0 (absent-key default 0). If −60
    and +60 do not differ, move the wall one cell and repeat. Falsified if the absent run differs
    from the run with 0, or if −60 and +60 never differ.
- **D23, voxel shadows (claim 18).** Setup: a flat map at level 0, open ground, voxel vehicles and
  a voxel aircraft of stock types that cast a shadow. Observable: inside the outline of the shadow,
  which pixels are darkened.
  - (a) Layer of an aircraft's shadow. A ground vehicle stands still; an aircraft flies across it at
    a steady height so that the aircraft's shadow passes over the vehicle's image. Expected: the
    vehicle's pixels inside the shadow's outline are darkened in every drawn frame of the overlap.
    Falsified if any of them shows undarkened while the aircraft's own image does not cover it
    (that would be a shadow drawn under ground-layer objects).
  - (b) Order of a vehicle's shadow in the ground list. Two vehicles A and B of the same type; B
    stands so that A's shadow falls on B's image outside A's own image. Run 1: B's key smaller than
    A's; run 2: B's key larger than A's (the same relative screen layout as far as possible).
    Expected: run 1, B's pixels inside A's shadow are darkened; run 2, they are not (B covers A's
    shadow). Falsified if the two runs give the same result.
  - (c) Profile. A vehicle drives step by step from behind a wall segment toward the viewer, its
    shadow overlapping the wall's image. Observable per position: which of the wall's pixels inside
    the shadow's outline, and outside the vehicle's own image, are darkened. Expected: inside the
    overlap the shadow is shown as a whole or hidden as a whole, switching between two positions
    (allow single rows to differ at the one or two positions next to the switch), like an upright
    drawing. Falsified if the shadow is cut along a row that stays fixed while the vehicle moves
    (the result of the flat profile, as for the infantry shadow in D8).

## Open questions


- **Building images:** the building shadow's bias (−4 − *L(z)*) has been tied to how building and unit
  images drawn as sprites get their shadow in general, not yet to the live building drawing with
  certainty; the slot animations are separate animation objects (claim 17), but whether a building
  draws any image in the object pass besides its voxel turret and its door sprite is not fully
  mapped; the construction state's build-up sprite is not yet placed. The special overlay image named in a building's art is read, but no drawing of it
  was found; medium confidence.
- **Building parts drawn as sprites in the object pass, and the door sprite:** the −2 is inferred
  from how other building and unit sprites are drawn, and when exactly the door sprite shows (which
  door stages) is not fully mapped. Low confidence.
- **Terrain objects:** which terrain objects count as animated, and so are drawn in the object pass
  as readers rather than in the cell pass (burning or falling trees, for example), is not fully
  mapped. Terrain objects whose type spawns ore are drawn 16 rows higher than other terrain
  objects; whether their depth anchor moves up with the image (which would make them about 5 depth
  units farther) was not checked. Medium confidence.
- **Tunnel layer:** tunnelling units have not been seen to be moved into the underground layer; they
  may stay in the ground layer while underground. Settle with the tunnel movement spec.
- **Effect groups** in claim 2: the three kinds of beam, light flashes and map beacons are
  identified; which drawing is the range circle is inferred (low confidence), and one further
  drawing in the same group has not been identified.
- **Tunnel digging term** of the unit depth adjustment and its interaction with the underground
  layer; to be settled with the tunnel movement spec.
- **Companion depth images (owned here):** which reference points of the body sprite the constants
  of F5 are measured from, and so the sign convention of the displacement, are not confirmed. That
  no building's art supplies its own depth image is settled (F5). S-0108 leaves depth images to
  this spec.
- **Shadow point of voxel models:** where the shadow draw point of a vehicle or aircraft lies (claim 18), in
  particular how high an airborne unit's shadow is drawn, is given by its movement type and has not
  been determined; nor has whether a vehicle's shadow paints over its own body where they overlap.
  This spec owns the drawing order and depth of the shadow; the point itself depends on the
  movement types (planned S-0106 and the movement specs).

- **Situational unit term *s*:** besides the ordinary −1, these cases are known but only partly
  determined, low confidence: −2 when the unit faces toward certain raised neighbouring tiles; −3 for
  a unit inside certain structures; −14 for a unit standing on a cell of certain special building
  types; +2 for a unit on a sloped cell next to a rock overlay. Their triggering conditions must be
  pinned down before scenarios can be written (slope, garrison and rock setups).
- **The veinhole monster:** it is in no render layer and no step of a drawn frame was found to paint
  it. Whether it is visible at all in the original, and through which other drawing, is not
  settled. Medium confidence.
- **Layer re-evaluation in flight:** with jumpjet and flying movement, an object's layer is
  re-evaluated and the object put back into a layer list (claim 4) at take-off and landing and
  at least once during flight; whether this happens at every move in flight (which would move it to
  the end of its list each time) is not settled. D5 and D12
  answer it. Low confidence.
- **Reused voxel images:** the original can show a voxel model with the image it made in an earlier
  drawn frame instead of rendering it again. The depth of such a reused image was checked to have
  the anchor, rounding and profile of F2, but whether it clips at the view's bottom edge in the same
  way (`quirks.upright_depth_anchor_after_clip`) was not confirmed. Medium confidence.
- **Cell pass:** the cell pass has two further parts that have not been identified, one ranked
  before the terrain tiles and one between the terrain objects and the building bodies (claim 16). The order among terrain tiles
  themselves (which matters only where two tile pixels tie) is not known.
