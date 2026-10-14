---
id: S-0102
title: Draw order, render layers, depth compositing, sprite placement and shadow pixels
topic: rendering
revision: 10
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0011, Q-0015, Q-0021, Q-0023, Q-0050, Q-0051, Q-0213]
supersedes: []
---

# S-0102: Draw order, render layers, depth compositing, sprite placement and shadow pixels

## Summary

In which order the battlefield is painted in each drawn frame, how objects are grouped into five
render layers and ordered inside each layer, and how a per-pixel depth buffer decides which pixels of
terrain, overlays, buildings, units, projectiles and animations hide each other. It also says where
a sprite's image frame lands on the screen relative to the object that draws it (infantry, vehicles
drawn as sprites, terrain objects, overlays and smudges), which image frame a vehicle drawn as a
sprite shows, and what a shadow does to the pixels under it. Positions, world pixels and the height
lift *L(z)* are those of S-0101. This spec covers skirmish; network games and campaigns are out
of scope.

**Cross-spec wording correction required.** The glossary's *sprite* and *aircraft* entries still
say aircraft can be drawn as sprites. They must instead point to claim 21: aircraft bodies are
voxel models only, and an aircraft type without voxel art draws no body. This is a vocabulary
correction to match the existing rule, not a new rendering rule.

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
| door sprite | An optional flat sprite that a building's art can name, drawn throughout unloading, including while its door is closed, opening, open or closing (S-0108, claim 23). |
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
| canvas, *W* × *H* | The full width *W* and height *H*, in pixels, that a shape file states once for all its image frames. |
| frame rectangle | The rectangle that a shape file stores for one image frame: its width and height and the position of its top-left pixel inside the canvas (a column and a row counted from the canvas's top-left pixel, both 0 or more). Pixels of the canvas outside the frame rectangle are transparent. |
| draw point | The screen pixel at which an object's sprite is placed (claims 19 to 25). |
| centred placement | Placing an image frame so that the point (⌊*W*/2⌋, ⌊*H*/2⌋) of its canvas lies on the draw point (claim 19). |
| hi-colour pixel | As in S-0103: a 16-bit packed colour in the display surface's pixel layout, with its red, green and blue *channels* (5, 6 and 5 bits in the 5-6-5 layout). |
| shadow image frame | An image frame of a sprite that is drawn as a shadow: it is applied with the darkening of claim 26 instead of being drawn with its colours. Which image frame is the shadow of which belongs to the sprite specs (S-0107, S-0108, S-0109); this spec gives where it lands and what it does to the pixels. |
| darkening | What a shadow image frame does to each screen pixel it is applied to (claim 26). |
| smudge | A flat mark on the ground of one or more cells (a crater, a scorch mark and the like), stored per cell with the number of the cell's piece. A smudge that covers several cells is one image; each covered cell knows which piece of it it holds, counted row by row across the smudge's width. |
| ion-blast wobble | A signed number of screen rows by which a vehicle or infantry unit's sprite is drawn higher while an ion-cannon blast (claim 2) shakes the area around it; 0 at all other times. Which blasts set it, and to what, is not specified here (Open questions). |
| walk counter | A per-unit integer that the movement of a vehicle advances (claim 22). It picks the walking image frame of a vehicle drawn as a sprite. |
| firing counter, death counter | Per-unit integers of a vehicle drawn as a sprite. While one of them is 0 or more it selects the vehicle's firing or death image frames (claim 22). How the death counter runs is in claim 22; which shots start the firing counter is not settled (Open questions). |
| standing-pose state | A per-unit on/off state of a vehicle drawn as a sprite. It chooses between the rows of claim 22's image-frame table. It is on when the vehicle is created, turns off when the vehicle starts a movement step and on again when the step ends (claim 22). |
| overlay shadow level *ℓ*<sub>s</sub> | The height level of the cell that holds an overlay, without the 4 levels that a bridge deck adds to *ℓ* (claim 13). It sets the depth bias of the overlay's shadow image frame. |
| game-wide tree strength | A game-wide integer, in health points, that a terrain type takes as its strength when no rules layer gives the type a strength (claim 23, Import mapping). |

## Behaviour

### Structure of a drawn frame

1. **Order of a drawn frame.** Each drawn frame starts from the terrain picture, after the cell pass
   has repainted whatever parts of it were marked for redraw (claim 16 gives the precedence inside
   the cell pass). The object pass then draws the five layers in the fixed order underground, surface,
   ground, air, top, and within a layer in layer-list order. So wherever two objects overlap and both
   pass the depth test, the one in the later layer, or later in the same list, is visible. After the
   last layer, the status extras of every vehicle, infantry, aircraft and building drawn in that drawn
   frame are painted, so every status extra appears above every object of every layer.
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
     - building light effects: D19 (e).
     - terrain objects on the ground, for animated terrain objects: D33 (c).

     `unmeasured`:
     - drop-pod movement: no stock situation that creates a drop-pod unit has been found.
     - tunnel movement, both rows: whether a tunnelling unit is drawn at all while underground is not
       settled (Open questions).
     - terrain objects on the ground that are not animated: they are painted in the cell pass, not
       from their layer list (claim 16), so their place in the ground list has no visible effect.
     - buildings, terrain objects and other objects in the air, the global cruise height and its
       400-lepton default: no stock situation lifts such an object off the ground.
     - the bridge rule for units with jumpjet movement (*State and ordering*): the moments at which
       such a unit's layer is
       evaluated in flight are not settled (Open questions), so no setup can make the rule show at a
       known moment.
     - particle systems: no drawing of a particle system's own, apart from its particles, has been
       identified, so there is nothing to observe.
     - the veinhole monster: stock data has a terrain type marked as a veinhole monster, but whether
       placing it creates a veinhole monster apart from an ordinary terrain object, and which
       drawing would then belong to which, is not settled (Open questions), so no expected picture
       can be stated.
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
   - Backing: `measurable` (scenarios D3, D4 (a); the gate term D4 (b), which gives the gate flag
     to a building with a voxel turret through the override section; the animation sort adjustment
     D21)
   - Confidence: high
6. **Ground-layer drawing order.** A newcomer appears immediately before the first object in
   the existing drawing order whose sort key is strictly greater than its own, or after all objects
   if there is none. In an already sorted order it follows every equal-key object. In an unsorted
   order it need not reach its fully sorted place: existing keys 5, 10, 3 leptons and a newcomer
   with key 4 leptons give 4, 5, 10, 3 leptons, not 3, 4, 5, 10 leptons.

   The drawing order has one ordering transition per tick, with the following exact result.
   Number the objects in the order before that transition from 1 to *N*, and let *k*ᵢ be object
   *i*'s key in leptons. Its resulting drawing position is the integer given by this table.
   Every position and every comparison refers to that same pre-transition order, not to an order
   partly changed during the transition. This specifies which object covers which in an overlap,
   not how an implementation stores or rearranges objects.

   | Condition for object *i* | Resulting drawing position (1 is drawn earliest) |
   |---|---|
   | some earlier object has a key strictly greater than *k*ᵢ | *i* − 1 |
   | no earlier key is strictly greater, and *j* is the first later position with *k*ⱼ ≥ *k*ᵢ | *j* − 1 |
   | no earlier key is strictly greater, and no later key is at least *k*ᵢ | *N* |

   Arithmetic on positions is exact integer arithmetic, without rounding. For example, keys
   10, 5, 3 leptons become 5, 3, 10 leptons, not the fully sorted 3, 5, 10 leptons.
   Equal-key objects retain their relative order through this transition.

   An increase of a key can therefore move an object any number of drawing positions later in
   one tick, while a decrease moves it at most one position earlier per tick. For a mover passing
   behind several stationary objects in an otherwise sorted order, it can remain drawn over some
   of them for several ticks, crossing one drawing position per transition. If each key change
   crosses at most one stationary object's key, that multi-tick settling is not visible.
   A movement-caused key change in tick *n* first affects drawing order in the picture after tick
   *n* + 1: the transition uses the keys before the current tick's movement, not after it.
   - Backing: `measurable` (D3 (a), (b) for movement ordering and lag; D3 (c) for equal-key
     stability). Entry placement is `unmeasured`: D3 observes already-placed objects, and no
     scenario here controls a newcomer's appearance while the existing order still has inversions.
   - Confidence: high for entry placement and the ordering result; medium for the exact one-tick lag
7. **Other layers are never sorted.** Objects entering the underground, surface, air or top layer
   are appended at the end of the list and never sorted. Among aircraft, for instance, the one that
   most recently entered or re-entered the layer is drawn over the others regardless of screen
   position. Take-off is one such event, not necessarily the last one during flight (claim 4).
   - Backing: `unmeasured` for absence of sorting in every non-ground layer: D5 is exploratory
     for air/top order, but flight re-entry events are unresolved (claim 4), so an order change
     cannot yet be distinguished from re-entry. No scenario here isolates underground or surface
     insertion order either.
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
    the terrain under it instead. Over ordinary terrain, the generic shadow of an infantry unit whose
    height above the surface *h* is greater than 0 uses −*L*(*z* − *h*) (claim 13). The generic
    shadow above a high-bridge cell at or above the deck is excluded from this rule: its point and
    depth depend on the unresolved case in S-0107 claim 19 (claim 20; Open questions).
    An object's body at any height is therefore depth-compared as if it stood at its
    ground reference row: a flying aircraft is hidden only by terrain, overlays or buildings that lie
    in front of the point below it, never by terrain behind that point even though it is drawn over
    that terrain. Overlays and their shadows use the exact 15 pixels per level instead of *L(z)* (claim 13).
    - Backing: `measurable` (D7 for aircraft bodies; the sprite, projectile, animation, overlay
      and ordinary-terrain shadow terms use the scenarios assigned in claim 13). The excluded
      above-deck shadow is `unmeasured` for the reason in claim 20.
    - Confidence: high for body anchoring; the per-kind biases retain claim 13's confidence
12. **Depth profile per drawing.** Ground-lying images (projectile sprites and shadows, the other shadows drawn as sprites,
    flat animations, ore and gems, overlays whose type is drawn flat, the building bib, under-door
    sprites and door sprite) use the flat profile; vehicles, infantry, aircraft, voxel models, building
    bodies, terrain objects, walls, overlays whose type is not drawn flat, and ordinary and stacked
    animations use the upright profile. The voxel shadows of vehicles and aircraft are the
    exception among shadows: they use the upright profile (claim 18). The profiles are defined in F2. Overlay types are drawn flat
    unless their type says otherwise (absent-key default: flat), except walls, which are always
    upright. A vehicle drawn as a sprite whose type is too big to fit under a bridge uses the flat
    profile near a bridge while its column factor is 0 (claim 13). Smudges have no profile: they take no part in the depth test
    (claim 25).
    - Backing: `measurable` (scenarios D8, D11, D15; voxel shadows D23 (c); vehicles drawn as
      sprites D33 (a)); the too-big vehicle is `unmeasured` (claim 13)
    - Confidence: high for the profiles of vehicles, infantry, aircraft, building bodies,
      projectiles, overlays, voxel shadows, animations and terrain objects (trees), and for the flat
      profile of the bib and the under-door sprites (their biases are medium, claim 13); low for
      the door sprite and for the flat profile of the vehicle too big to fit under a bridge, as in
      claim 13
13. **Depth bias per kind.** The depth bias added to the row depth is given below. **Every value is
    final:** it already includes every constant that applies to that kind of drawing, including the
    −2 that applies to every image of a vehicle, infantry unit or building drawn as a sprite and the
    further −10 of an infantry body. Nothing further is added. *U* is the unit depth adjustment of
    claim 14; *z* is the world height of the object's position; *h* is the object's height above the
    surface (glossary); all values are integers in depth units.

    | Drawing | Depth bias (final) | Profile | Writes depth |
    |---|---|---|---|
    | vehicle drawn as a sprite, *h* = 0 | *U* − 2 | upright | no |
    | vehicle drawn as a sprite, *h* not 0 | −*L(z)* − 2 (*U* does not apply) | upright | no |
    | vehicle drawn as a sprite whose type is too big to fit under a bridge, while it is near a bridge and its column factor (claim 14) is 0 | *U* − 18 | flat | no |
    | infantry body, *h* = 0 | *U* − 12 | upright | no |
    | infantry body, *h* not 0 | −*L(z)* − 12 (*U* does not apply) | upright | no |
    | vehicle or aircraft drawn as a voxel model | *U* (no further constant) | upright | no |
    | voxel shadow of a vehicle or aircraft (claim 18) | *U* of the object casting it (no further constant) | upright | no |
    | own shadow image frame of an infantry unit or a vehicle drawn as a sprite (claims 20, 21; only while *h* = 0) | −*L(z)* − 4 (*U* does not apply) | flat | no |
    | generic shadow of an infantry unit with *h* > 0 over ordinary terrain (claim 20) | −5 − *L*(*z* − *h*) | flat | no |
    | generic shadow of an infantry unit above a high-bridge cell at or above the deck, not on the deck (S-0107 claim 19) | not specified; see Open questions | flat | no |
    | shadow of an infantry unit falling in a drop pod | *U* − 2 | flat | no |
    | shadow of a vehicle drawn as a sprite whose type has a turret (claim 22) | *U* − 2 | flat | no |
    | smudge (claim 25) | none: no depth test and no depth write | — | no |
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
    | shadow image frame of an overlay, every kind (ore and gems, walls, crates, bridge decks, rubble, other flat and upright types) | −2 − 15 × *ℓ*<sub>s</sub> | flat | yes |

    Here *ℓ* is the cell's height level, plus 4 if the cell carries a bridge deck, and *ℓ*<sub>s</sub>
    is the cell's height level alone (Terms): an overlay's shadow never takes the deck's 4 levels nor
    the −15 of an upright type. So on a cell without a bridge deck the shadow has the same bias as a
    flat overlay body there, and on a deck cell it is 60 depth units farther than the deck's own
    pixels, which it therefore never darkens (claim 27). The vertical draw shift is in pixels and
    counts as depth units here.

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
    - Backing: `measurable`. Coverage by row: infantry body with *h* = 0 and the own shadow of
      infantry D8, D15, D20, D31; generic shadow of infantry with *h* > 0 D25 (b) (placement only)
      and D31 (e), on ordinary terrain only; voxel units
      D7, D9, D10 (relative tests only); voxel shadow D23 (profile; its bias only relative to the
      body's); voxel turret D22 (d), including the absent-key default 0 of the turret slot depth
      adjustment; gate body, building shadow, bib,
      under-door sprites, non-animated terrain object and its shadow D15; projectile sprite,
      projectile shadow,
      ordinary, flat and tiled animations and animation shadow D15; overlays, wall, rock, ore and gems
      D11, D15; overlay shadows D35; vehicle drawn as a sprite with *h* = 0 and its own shadow D33 (a), (b); shadow of a
      vehicle drawn as a sprite whose type has a turret D29 (c) (its place under the vehicle's
      image, and its bias through a bib); animated terrain object D33 (c) (that it is a reader, and
      its bias against a wall). `unmeasured`:
      - the generic infantry shadow above a high-bridge cell at or above the deck: S-0107 claim 19
        leaves the neighbouring-cell condition unresolved, so its point and depth cannot be
        predicted for a controlled setup here.
      - normal and unloading building-body biases: D15 does not isolate their values from the
        unresolved companion depth image and the different unloading sprite rectangles. Image
        selection is covered by S-0108 B10, not an absolute depth-bias test here.
      - building parts drawn as sprites and the door sprite: the only writer they overlap is their
        own body, whose companion depth image has reference points that are not yet settled (F5,
        Open questions).
      - animations drawn entirely as a shadow: the state cannot be set up on purpose.
      - the row of a vehicle drawn as a sprite whose type is too big to fit under a bridge: it
        applies only on cells that count as near a bridge, and which cells those are, in terms of
        the map's terrain and overlays, is not settled (Open questions), so no layout can be stated
        in which the row is known to apply. Under the bridge itself the deck hides the vehicle
        whatever its profile (claim 15).
      - bodies with *h* not 0 (infantry and vehicles drawn as sprites): in the air the body's
        depth differs from its value on the ground by about one depth unit only (−*L(z)* − 12
        against *U* − 12 with *s* = −1, and likewise with −2), inside the rounding of F2.
      - the drop-pod shadow: no stock situation that creates a drop-pod unit has been found
        (claim 3).
      - the two remembered-building rows: a probe unit of the local player standing close enough
        to overlap a fogged building would normally bring it back in sight, so no probe can be
        compared with the remembered body; and the remembered animation is
        tested only against drawings ranked before it, mainly its own body, whose companion depth
        image has reference points that are not yet settled (F5). D17 tests where these drawings rank (claim 16), not their
        biases.
    - Confidence: high for projectile sprites, overlays, overlay shadows, animations, building bodies, infantry
      bodies, the own shadows of sprite units, the smudge row, non-animated terrain objects and their
      shadows, and the −*L(z)* term; medium for the rows with *h* not 0, the generic infantry shadow,
      vehicles drawn as sprites, voxel units, the voxel shadow, the voxel turret row and its absent-key
      default, building shadow, bib,
      unloading, under-door and gate rows, the choice of unloading image, the remembered building
      rows, animated terrain objects and projectile shadows; low for building parts drawn as
      sprites, the door sprite, animations drawn entirely as a shadow, the drop-pod shadow, the
      turret shadow and the row of the type too big to fit under a bridge
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
    Infantry and vehicles drawn as sprites use *U* only while their height above the surface is 0.
    In the air their bodies use −*L(z)* alone, without *s* and *F* (claim 13). A vehicle or
    aircraft drawn as a voxel model uses *U* at every height.
    - Backing: `measurable` (scenarios D9, D16; the absent-key defaults D9 run A0 and D16 runs with
      the key absent)
    - Confidence: high for the cliff fudge and the ordinary *s* = −1; medium for column, tunnel and
      bridge detection
15. **Bridges.** A bridge deck is an overlay drawn in the cell pass. Its world height is 4 levels
    above the terrain of its cell, and its depth uses that 4-level elevation (claim 13). Its screen
    draw point is given by claim 24: the image is lifted by 15 rows per level of the cell's terrain
    and receives the deck-cell shift, without an extra 60-row lift for the deck's 4 world-height
    levels. It has a flat profile and depth is written (bias per claim 13). A unit under the bridge is
    drawn later with the depth of its own ground reference row; the deck pixels covering it belong to
    ground rows nearer to the viewer, so the deck hides it. A unit on the deck is lifted by its height
    and has the bias of claim 13, which contains −*L(z)*; the deck pixels behind and around its
    image belong to ground rows behind its ground point, so the unit passes the depth test and appears
    on the deck (its lowest one or two rows can be lost to the deck's −2 bias, the unit's own small
    constants and the rounding of the upright profile). The bridge fudge (claim 14) exists for tall
    units passing under.
    - Backing: `measurable` (scenario D10 for unit occlusion, D26 for screen placement)
    - Confidence: medium
16. **Precedence inside the cell pass.** Within each area it repaints, the cell pass ranks its
    drawings as follows:

    | Rank | Drawings |
    |---|---|
    | 1 | terrain tiles, each followed at once by the pieces of the smudges on its cell (claim 25), which take no part in the depth test |
    | 2 | with fog of war on, the remembered images of fogged buildings, one building after another, nearest first (measure below); each building's group ranks its body, shadow and bib before its remembered animations |
    | 3 | overlays, then overlay shadows: every overlay of the repainted area ranks before every overlay shadow of that area, so no overlay's shadow ranks before a neighbouring overlay |
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
    ranks 1 (smudges), 3 and 4 as outside fog. Which objects stay hidden under fog belongs to the visibility
    specs.

    The ranking is visible through these consequences, and only these:
    - Where a tile pixel and another cell-pass writer give a pixel exactly the same depth, the tile
      shows. Where two other cell-pass writers tie, the one ranked earlier shows. So where a
      building's body and its own shadow, bib or under-door sprite tie, the body pixel shows, and
      where its bib and an under-door sprite tie, the bib pixel shows. Likewise, where an overlay's
      shadow falls on an ore, gem or other flat overlay at the same height level, both have the
      same depth (claim 13) and the overlay pixel shows undarkened, whichever of the two cells comes
      first.
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
    - Backing: `measurable` (scenario D17: trees in (a), a second remembered building in (b); the
      overlay shadows ranking after every overlay, through the tie with a neighbouring flat overlay,
      D35 (b)); the live-body case is `unmeasured` (D17 (c) gives the reason). The other tie rules,
      and with them the order inside a live building's group, are `unmeasured`, because exact depth
      ties between those writers cannot be set up on purpose. That terrain objects, overlays and smudges are never remembered is
      `unmeasured`: it would show when one of them changes inside a fogged area, but whether such a
      change repaints that area at once is not settled (claim 8 lists the redraw events at
      medium confidence), so no expected result can be stated.
    - Confidence: high for tiles first, for buildings being the only remembered objects, for the
      measure *N*, for overlay shadows ranking after all overlays and for the order inside a live
      building's group; medium for the order of the later steps

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
    the next ordering transition on. The turret slot's depth adjustment is also the depth adjustment of a
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
    - an aircraft's shadow, drawn in the top layer, darkens ground-layer object pixels it falls on
      wherever its depth test passes and its own body does not cover them. A ground vehicle's
      shadow similarly darkens qualifying pixels of ground-layer objects before the vehicle in
      the ground list, and is covered by those after it wherever their own tests pass.
    - where the shadow falls on a writer that is nearer than its depth (a wall, a cliff face, a
      building's body), it is hidden or shown as a whole, like an upright drawing, not cut along a
      fixed row.

    Where an aircraft's shadow and its own body overlap and both pass the depth test, the body is
    shown undarkened: the shadow never paints over its own aircraft. For vehicles, a freshly made
    shadow excludes non-transparent pixels of the completed body, turret and barrel image,
    independently of their scene-depth acceptance (S-0105, claim 39). Thus qualifying own-image
    overlaps are undarkened on flat ground, on slopes and for hovering vehicles. This is not an
    unconditional shadow-before-body rule: a reused shadow retains its earlier own-image exclusions
    and can darken current own-image pixels that filled an earlier hole, or leave a gap where an
    earlier part pixel has disappeared. S-0105 claim 39 owns those exclusions and their reuse,
    including `quirks.voxel_shadow_cached_cutouts` (default original; off: exclude the current
    completed part image on every appearance).
    - Backing: `measurable` (scenario D23, including D23 (d) for Q-0213). The bias value itself is tested only relative to the
      body's (both use *U*); whether an aircraft's body shows over its own shadow is `unmeasured`, because an
      aircraft's shadow and its own body overlap only when it is on or very near the ground, where
      the shadow point is not settled.
    - Confidence: high for the drawing inside the object's own drawing (and so its layer and list
      place), the reader-only test, the bias *U*, the upright profile and the fresh vehicle
      own-image exclusions and their retention; medium for visible vehicle overlap and for an
      aircraft's body showing over its own shadow

### Placement of sprite images

19. **Centred placement.** Every sprite drawing named in this spec is placed centred on its draw
    point. Let the shape file's canvas be *W* × *H* and let the image frame's rectangle have its
    top-left pixel at column *c* and row *r* inside the canvas. With the draw point at screen pixel
    (*x*<sub>p</sub>, *y*<sub>p</sub>), the image frame's top-left pixel lands at
    (*x*<sub>p</sub> − ⌊*W*/2⌋ + *c*, *y*<sub>p</sub> − ⌊*H*/2⌋ + *r*) (F6). The image frame's own
    width and height play no part, and every image frame of the file shares the same canvas, so
    image frames of different sizes stay aligned. For an odd canvas size the integer half is rounded
    down, not up: a canvas 61 pixels wide has its draw point on column 30, not 31.
    This is the meaning of "drawn centred" in S-0108 claim 8. It applies to building images (body,
    build-up, bib, shadow, unloading, under-door and door sprites), infantry, vehicles drawn as
    sprites, terrain objects, overlays, smudges, every shadow image frame, and also to projectile and
    animation sprites, whose draw points their own specs give.
    - Backing: `measurable` (scenario D24)
    - Confidence: high
20. **Infantry.** An infantry unit's draw point is the screen pixel of its position (S-0101 F1,
    which includes *L(z)*), moved up by its ion-blast wobble. Its body image frame and, while its
    height above the surface is 0 and its type does not suppress shadows, its own shadow image
    frame (S-0107 claim 19) are both placed
    centred on that one draw point; the shadow is drawn right after the body (claim 28). While its
    height above the surface *h* is greater than 0 it draws no shadow from its own sprite but the
    generic infantry shadow of S-0107 claim 19. On ordinary terrain, the shadow is centred on the
    point *L*(*h*) rows below its draw point, about where the ground under it appears. For a unit
    above a high-bridge cell at or above the deck, and not on the deck, S-0107 claim 19 says the
    deck can count as the ground under an unresolved neighbouring-cell condition. This claim does
    not specify the shadow point or depth anywhere in that unresolved case. The ordinary-terrain
    depth bias is in claim 13.
    - Backing: `measurable` (scenario D25 (a) for the body and own shadow, (b) for the generic
      shadow over ordinary terrain). The above-deck case is `unmeasured`: its neighbouring-cell
      condition is not identified, so no scenario can predict the shadow point there. The ion-blast
      wobble: `unmeasured`, because no stock weapon or event that makes an ion-cannon blast is known
      (claim 2).
    - Confidence: high for the body and its own shadow; medium for the generic shadow's
      ordinary-terrain point; low for the unresolved above-deck case
21. **Vehicles drawn as sprites; aircraft.** A vehicle drawn as a sprite has the same draw point as
    an infantry unit: the screen pixel of its position, moved up by its ion-blast wobble. Its body
    image frame (claim 22) is placed centred on it. Its own shadow image frame is the body frame
    plus ⌊*N*/2⌋, *N* being the number of image frames in its shape file (the second half of the
    file holds the shadows), placed centred on the same draw point and drawn right after the body.
    The shadow is drawn only while the vehicle's height above the surface is 0, its type does not
    suppress shadows, and its type is not marked as either of the two visceroid kinds (Import
    mapping). A vehicle type with a turret drawn as a sprite is drawn differently (claim 22).
    The original has no way to draw an aircraft as a sprite: an aircraft is always drawn as a voxel
    model (S-0105), and an aircraft type whose art does not mark it as a voxel model shows no body
    of its own, even though its sprite file is looked up (S-0007).
    - Backing: `measurable`: scenario D29 (b) for the placement, the shadow frame and the three
      suppressing conditions (each set through the override section); D34 for the aircraft rule,
      with a named loose copy of the art file.
    - Confidence: high for the draw point, the shadow frame and the aircraft rule; medium for the
      conditions
22. **Which image frame a vehicle drawn as a sprite shows.** Write *F*<sub>w</sub>, *F*<sub>f</sub>,
    *F*<sub>s</sub>, *F*<sub>d</sub> for the type's walking, firing, standing and death frame counts,
    *S*<sub>w</sub>, *S*<sub>f</sub>, *S*<sub>s</sub>, *S*<sub>d</sub> for its walking, firing,
    standing and death start frames and *R*<sub>d</sub> for its death frame rate (Import mapping,
    with the derived defaults).
    - **Facing part.** If the type's facing count is 8 and the vehicle is not disguised,
      *d* = (*f*<sub>8</sub> + 1) mod 8, where *f*<sub>8</sub> is the 8-step facing of its body
      facing (glossary, *n-step facing*: halves round toward the next step clockwise). So *d* = 0
      for map north-west, 1 for map north, 2 for map north-east and so on clockwise: the art stores
      the eight facings clockwise starting with map north-west. Otherwise *d* = 0: a type whose
      facing count is not 8 always shows its first facing (`quirks.sprite_vehicle_eight_facings_only`).
      The off state's facing part, including counts 2 and 4, is defined in *Edge cases and quirks*.
      It is an alternative of our engine, not behaviour of the original (Q-0050 part 2).
    - **Image frame.** The first row that applies gives the image frame (integers; ⌊·⌋ and mod act on
      values that are 0 or more):

      | Vehicle state | Image frame |
      |---|---|
      | firing: its firing counter *c* is 0 or more | *S*<sub>f</sub> + *d*·*F*<sub>f</sub> + ⌊*c*/2⌋ |
      | moving, or its type's idle rate is not 0 | *S*<sub>w</sub> + *d*·*F*<sub>w</sub> + (*w* mod *F*<sub>w</sub>), *w* its walk counter |
      | dying: its death counter *k* is 0 or more (only for a type with *F*<sub>d</sub> > 0, see below) | *S*<sub>d</sub> + min(⌊*k*/*R*<sub>d</sub>⌋, *F*<sub>d</sub> − 1) |
      | its standing-pose state is off | *d* |
      | *F*<sub>s</sub> > 0 | *S*<sub>s</sub> + *d*·*F*<sub>s</sub> |
      | otherwise | *S*<sub>w</sub> + *d*·*F*<sub>w</sub> |

    - **Walk counter.** A vehicle's walk counter grows by 1 in a tick when it is moving and the
      tick number (glossary; 0 while the first tick runs) is a multiple of its type's walk rate
      (ticks), or when it is not moving, its type's idle rate is not 0 and the tick number is a
      multiple of the idle rate.
    - **Death counter.** The death counter is below 0 until the vehicle is destroyed. It is
      started, at 0, only when the vehicle is destroyed and its type's death frame count
      *F*<sub>d</sub> is greater than 0; from then on it grows by 1 per tick until the vehicle is
      removed (removal belongs to S-0312). So the dying row applies only to types with
      *F*<sub>d</sub> > 0, and its image frames run from *S*<sub>d</sub> upward, one every
      *R*<sub>d</sub> ticks, holding the last one. With the absent-key default
      *F*<sub>d</sub> = 0 (and so *S*<sub>d</sub> = −1) the dying row never applies.
    - **Standing-pose state.** It is on when the vehicle is created, so also when the map places it,
      and nothing turns it off before the vehicle first moves. With driving, hovering, mech or ship
      movement it turns off when the vehicle starts a movement step (with mech movement only once
      any turning on the spot before the step has finished) and on again when that step ends: for driving and ship movement at the end
      of every step from one cell to the next, so it goes off and on at each cell of a route; for
      hovering movement when the vehicle comes within its arrival distance of the point it is
      moving to; for mech movement at the end of each step. It is also turned on when the vehicle's
      movement type changes, when a superweapon moves the vehicle to a new place, and at one further
      event that re-places the vehicle (not identified). Other movement types never turn it off.
      Consequences:
      - a vehicle placed by the map shows, from the first drawn frame and before any tick has run,
        the row for *F*<sub>s</sub> > 0 if its type has standing frames and otherwise the last row
        (*S*<sub>w</sub> + *d*·*F*<sub>w</sub>), never the row *d*, unless its type's idle rate is
        not 0 (then the moving row).
      - a vehicle that has ended a move shows the same rows.
      - the row *d* needs the state off while the vehicle does not count as moving, that is, a
        vehicle standing still part-way through a movement step with an idle rate of 0, not firing
        and not dying. No stock situation that produces this has been identified (Open questions).
    - When the firing counter runs, and one further case in which the walk counter grows while the
      vehicle is in the air, are not settled (Open questions).
    - A vehicle whose type has a turret and is drawn as a sprite draws its body frame as above and
      a turret image frame ((*t*<sub>32</sub> + 4) mod 32) + 8·*F*<sub>w</sub>, *t*<sub>32</sub>
      being the 32-step facing of its turret facing, combined into one image. Its shadow, the body
      frame plus ⌊*N*/2⌋, has the bias of claim 13 and lies under that combined image: unlike
      claim 28, the shadow never darkens the vehicle's own body or turret pixels.
    - Backing: `measurable`: the facing part and the walking and standing rows D29 (a), the
      standing-pose state at placement and after a move D29 (f); the turret
      path D29 (c) (the turret flag set through the override section); facing counts other than 8
      D29 (d) and the dying row and death counter D29 (e), both with a named loose copy of the art
      file. `unmeasured` for the firing row: which shots start the firing counter is not settled
      (Open questions), so no setup can make the row show at a known tick. `unmeasured` for the row
      *d* and for the events of the standing-pose state other than creation and movement steps
      (change of movement type, superweapon move): no stock situation stops a vehicle part-way
      through a step, so the state cannot be seen off while the vehicle is not moving.
    - Confidence: high for the facing part and the image-frame formulas, medium for the walk counter, the
      death counter and the standing-pose state (high that it is on at creation and after a
      completed step; medium for the other events that turn it on), low for the turret path
23. **Terrain objects.** A terrain object (a tree and the like) stands at the centre of its cell, at
    the terrain surface height there (S-0106). Its draw point is the screen pixel of that position,
    and a terrain object whose type spawns ore is drawn 16 rows higher. It is an animated terrain
    object (glossary), drawn in the object pass instead of the cell pass (claims 10, 16), when its
    type is marked animated and also once its destruction sequence has begun; a burning terrain
    object whose destruction sequence has not begun stays in the cell pass. Its body image frame is:
    - for a type marked animated, its current animation stage, whatever its health. The stage is 0
      when the object is placed, so every animated terrain object first shows image frame 0. Only
      an ore mine (a terrain object whose type has the spawns-resources flag, S-0401 claim 15)
      advances its stage: it shows image frame *k* during the *k*-th rate interval of an animation
      that S-0401 claim 15 starts and times, for *k* = 0 to ⌊*N*/2⌋ − 1, and its stage returns to 0
      in the same turn in which it reaches ⌊*N*/2⌋, so image frame ⌊*N*/2⌋ is never shown as its
      body. A mine with ⌊*N*/2⌋ = 0 never returns: once its animation starts, its image frame only
      grows (S-0401 claim 15). An animated terrain object whose type lacks the spawns-resources flag takes no turns
      (S-0203 claim 2), so its stage stays 0 and it shows image frame 0 for as long as it stands
      (until its destruction sequence, below). It is still drawn in the object pass as an animated
      terrain object (claims 10, 16).
    - during its destruction sequence, an image frame that depends on that sequence (not specified here),
    - otherwise image frame 0 while its health is 2 or more and image frame 1 when it is below 2.
      A terrain object starts with its type's strength as its health, so it first shows image
      frame 0, unless that strength is below 2.

    **Strength of a terrain type.** A terrain type's strength is unset until a rules layer gives
    it. Each time a rules layer that holds the type's section has been read (S-0006), a strength
    that is still unset is set to the game-wide tree strength as it stands at that moment. Within a
    layer the general settings, an early game-wide section (S-0006, Terms; S-0004, claim 4), are
    read before the type sections, so a game-wide tree strength that the same layer gives counts. Writing −1 as the type's strength (in any spelling that reads
    as −1, S-0003) unsets it, so that layer then sets it to the game-wide tree strength again.
    Once set, the strength keeps its value in later layers that give the type's section without a
    strength, even if such a layer changes the game-wide tree strength
    (`quirks.tree_strength_fixed_at_first_section`). How an empty or unreadable strength value is
    read is S-0003's and S-0006's rule for integer keys. A terrain type whose section no layer holds
    keeps its strength unset: its objects start with health −1 and show image frame 1. The
    game-wide tree strength's absent-key default is 25 health points; as a game-wide key it carries
    over between the games of a session (S-0006, claim 10).

    Its shadow image frame is the body frame plus ⌊*N*/2⌋, placed centred on the same draw point.
    In the object pass its shadow can darken its own body (claim 30); the ore-spawning raise and
    destruction state do not change that ordering.
    The 16-row raise of an ore-spawning object moves its whole drawing rectangle, and with it the
    anchor row of its upright depth profile (F2): every pixel of its image has a depth 16 units
    greater (farther) than the same pixel would have without the raise. Its shadow, being flat, has
    the depth of the screen rows it lies on, as for any terrain object.
    - Backing: `measurable` (scenario D27) for the placement, the 16-row raise and its depth
      consequence, the initial image frame and image frame 1 (D27 (iii): the type's strength set to
      1 through the override section, which pre-placed objects already follow, S-0009), the
      strength of a terrain type and its quirk (D27 (iv) and (v)), and the image frames of animated
      terrain objects (D27 (vi)); animated terrain objects by the type's flag D33 (c). `unmeasured`
      for destruction-frame selection and timing, because the destruction sequence is not settled
      (Open questions), so no expected image-frame sequence can be stated. D36 (c) tests the
      body/shadow ordering during visible destruction independently of that timing (claim 30),
      not the exact transition tick from cell pass to object pass; that transition remains
      `unmeasured` because no scenario here isolates it. Also `unmeasured` for a
      terrain type whose section no layer holds: the original's type lists name only types that
      have sections in stock data, and whether a test map can add one without a section is not
      known.
    - Confidence: medium for which terrain objects are animated terrain objects (the type's flag:
      high); high for the position, the draw point and image frames 0 and 1, for the stage being 0 at
      placement, for the strength rule and its default and for the image frames of animated objects.
      Medium for the depth consequence of the raise and for the type whose section no layer holds.
      low for the destruction frames
24. **Overlays.** The draw point of an overlay on cell (*x*, *y*) at height level ℓ is the screen
    pixel of the world pixel (30(*x* − *y*), 15(*x* + *y*) + 15 − 15ℓ + *s*): the cell's centre at
    level 0, lifted by exactly 15 rows per level (S-0101 claim 11), plus a vertical shift *s* in
    rows (negative is up):

    > *s* = *s*<sub>type</sub> + *s*<sub>rail</sub> + *s*<sub>cell</sub> (integers, screen rows)

    | Term | Condition | Value (rows) |
    |---|---|---|
    | *s*<sub>type</sub> | the overlay type with index 126 | −13 |
    | | otherwise, ore or gems, a wall or a crate | −12 |
    | | otherwise | 0 |
    | *s*<sub>rail</sub> | the overlay type's land type is railroad | −1 |
    | | otherwise | 0 |
    | *s*<sub>cell</sub> | the cell carries a bridge deck and has an overlay data value of 9 to 17 | −31 |
    | | otherwise, the cell carries a bridge deck | −16 |
    | | otherwise (no bridge deck), the overlay type with index 239 | −15 |
    | | otherwise | 0 |

    The three terms are independent. Indices count from 0 in the overlay type list, in the order of
    S-0004. For example, the overlay type with index 126 gets −13 − 1 = −14 if its land type is
    railroad and −13 otherwise, and any overlay on a deck cell with an overlay data value of 9 to 17
    gets −31 plus its *s*<sub>type</sub> and *s*<sub>rail</sub> (the index-239 value never applies on
    a deck cell).

    There is no horizontal shift. The two indices and the data range 9 to 17 are engine
    constants. The overlay's shadow image frame is placed on the same draw point, except on a cell
    that carries a bridge deck and has an overlay data value of 9 to 17, where it is placed 15
    columns to the left and 7 rows below that draw point.
    - Backing: `measurable` (scenario D26)
    - Confidence: high for the base point and for −12 of *s*<sub>type</sub> (ore and gems, walls,
      crates and the index-126 type); medium for the index-126 type's further −1, for
      *s*<sub>rail</sub>, for *s*<sub>cell</sub> and for the shadow's displacement
25. **Smudges.** A smudge is painted in the cell pass, right after the terrain tile of each cell it
    covers (claim 16, rank 1), with image frame 0. For a cell (*x*, *y*) at level ℓ holding piece *p*
    of a smudge *w* cells wide, let *i* = *p* mod *w* and *j* = ⌊*p*/*w*⌋ (integers). The draw
    point is the world pixel (30((*x* − *i*) − (*y* − *j*)), 15((*x* − *i*) + (*y* − *j*)) − 15ℓ):
    the top corner of the smudge's first cell, lifted by the level of the cell being painted. Each
    covered cell paints the whole image again, clipped to the area being repainted; where covered
    cells lie at different levels, the last one painted decides. A smudge neither reads nor writes
    the depth buffer: all of its pixels are painted, and the stored depth stays that of the tile.
    So a smudge only recolours the terrain picture. Everything painted later is tested against the
    tile's depth as if the smudge were not there, and a terrain tile painted later over the same
    screen pixel replaces it.
    - Backing: `measurable` (scenario D28) for the draw point and the first-cell anchor, and
      `unmeasured` for the depth rule, because whether a later tile covers a smudge depends on the
      order among terrain tiles, which is not known (Open questions).
    - Confidence: high for the depth rule and the image frame; medium for the draw point

### Shadow pixels

26. **Darkening.** Wherever a non-transparent pixel of a shadow image frame passes its depth test
    (claims 10 to 13), the screen pixel under it, a hi-colour pixel, is replaced by the same pixel
    with each channel halved and rounded down (F7). Nothing else enters:
    - every non-transparent pixel of the shadow frame acts the same, whatever its palette index
      (index 0 is transparent and leaves the pixel alone),
    - the palette, the house colour, the light at the shadow's cell or at the object (S-0104), the
      object's brightness and the light-alpha values of S-0110 play no part,
    - a translucent object's shadow is a plain darkening, not translucent.

    The pixel under the shadow already carries the lighting and fog darkening it was drawn with, so
    a shadow halves the lit pixel. The rule is the same for every shadow: the own shadows of infantry
    and vehicles drawn as sprites, the generic infantry shadow, building, terrain-object, overlay,
    projectile and animation shadows, and the shadows of voxel models (S-0105 claim 31).
    - Backing: `measurable` (scenario D30: the pixel rule, the palette index and the light in (a) to
      (c), the house colour in (d), the palette in (e)); the translucency bullet is `unmeasured`,
      because no stock situation is known in which a sprite unit is drawn translucent while it casts
      its own shadow.
    - Confidence: high
27. **Overlapping shadows.** Each shadow drawing darkens every pixel where it passes its depth test,
    so a pixel under two shadows is darkened twice (each channel ⌊⌊*v*/2⌋/2⌋) when both pass. Within
    one drawing a pixel is darkened at most once; a voxel model's shadow counts as one drawing.
    Whether the second shadow passes follows from claims 10 and 13:
    - shadows drawn in the object pass are readers and store nothing,
    - shadows painted in the cell pass (building, terrain-object and overlay shadows, and the
      remembered shadows of fogged buildings) are writers: where one darkens a pixel it also stores
      its own depth there (`quirks.cell_pass_shadows_write_depth`). A shadow drawn later darkens that
      pixel again only where its depth is strictly less.

    Consequences on flat ground at one height level, with the biases of claim 13:

    | Shadow drawn later | Over the earlier shadow or flat writer | Pixel darkened |
    |---|---|---|
    | building shadow (−4) | another building's shadow (−4) | once: equal depths |
    | building shadow (−4) | terrain-object shadow (−3) | twice |
    | own shadow of an infantry unit or vehicle drawn as a sprite (−4) | building shadow (−4) | once: the unit's shadow does not show inside the building's shadow |
    | the same | terrain-object shadow (−3) | twice |
    | the same | bib (−3), ore or gem overlay (−2) | once: the unit's shadow is drawn over them |
    | the same | another sprite unit's own shadow | twice |
    | animation shadow (−2) | building shadow or terrain-object shadow | once: the animation's shadow does not show there |
    | animation shadow (−2) | bib, ore or gems | not at all: the animation's shadow does not show there |
    | projectile shadow (−10, over ground of level 0) | building shadow or terrain-object shadow | twice |
    | generic shadow of an infantry unit in the air over ordinary terrain (−5 − *L*(*z* − *h*), over ground of level 0: −5) | building shadow or terrain-object shadow | twice |
    | overlay shadow (−2) | bare terrain tile | once |
    | overlay shadow (−2) | an ore, gem or other flat overlay on a cell of the same level (drawn earlier, claim 16) | not at all: the overlay shows undarkened |
    | overlay shadow (−2) | another overlay's shadow on the same level | once: equal depths |
    | overlay shadow (−2) | a wall, an upright overlay or a bridge deck on a cell of the same level | not at all: those pixels are nearer |
    | terrain-object shadow (−3) or building shadow (−4) | overlay shadow | twice |
    | own shadow of an infantry unit or vehicle drawn as a sprite (−4), projectile shadow (−10) | overlay shadow | twice |
    | animation shadow (−2) | overlay shadow | once: the animation's shadow does not show there |

    - Backing: `measurable` (scenario D31; the overlay-shadow rows D35)
    - Confidence: high for the mechanism and for the overlay-shadow rows; medium for the rows that
      involve the building shadow and the bib, whose biases are medium (claim 13)
28. **A sprite unit's shadow over its own body.** An infantry unit or vehicle drawn as a sprite
    draws its own shadow right after its body, and the body stores no depth. So where its shadow
    image frame overlaps its own body frame, the shadow darkens the body's pixels (wherever it passes
    its depth test against the stored depth). A building's body is a writer painted before its
    shadow, so the building's shadow darkens the body only where the shadow's depth is strictly less
    than the body's stored depth.
    - Backing: `measurable` (scenario D32) for sprite units; the building part is `unmeasured`,
      because the body's depth comes with its companion depth image, whose placement is not settled
      (F5).
    - Confidence: high for sprite units; medium for buildings

29. **Sprite vehicle facings with the switch off.** With
    `quirks.sprite_vehicle_eight_facings_only` off, the facing part is the exact integer formula
    in *Edge cases and quirks*: turn the full body facing clockwise by 8192 facing units, then
    round to the imported facing count, with halves going clockwise. This includes counts 2 and 4,
    not a rounded rotation applied after choosing the coarse facing. Disguise and unsupported
    counts still give part 0. This is our engine's alternative, not behaviour of the original
    (Q-0050 part 2); revision 5 already defined it.
    - Backing: `unmeasured`, because the original has no such alternative to observe; engine test
      D37 checks the definition. The original's rule is claim 22, tested by D29.
    - Confidence: high, as the definition of the alternative
30. **Animated terrain body and shadow ordering.** In the object pass an animated terrain
    object's body and its own shadow form one group at that object's place in its layer list:
    the body ranks before its shadow, with no other object's drawing between them. There is no
    separate terrain-shadow pass. Both are depth readers (claims 10 and 13), so where the
    shadow image frame overlaps a body pixel and passes its depth test against the terrain
    picture, it darkens that already-painted body pixel by claim 26's channel halving. The body
    does not shield itself through depth. Another object later in the layer list paints over
    both wherever its own depth test passes.

    The ordering is identical for an ore mine, whose body and shadow both receive the 16-row
    raise of claim 23, and for a terrain object whose destruction sequence has begun. The
    destruction state affects the body-frame selection of claim 23 and makes an otherwise
    non-animated type use the object pass; it does not reverse or separate body and shadow.
    This claim fixes ordering, not the still-open destruction-frame timing.
    - Backing: `measurable` (D36, including the mine and destruction cases)
    - Confidence: high

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
`quirks.upright_depth_anchor_after_clip`). Backing: `measurable` (D38) for the clipping
consequence on sprite readers; confidence: high. Its persistence on cell-pass writers is
`unmeasured`: no controlled writer-repaint test is given here; confidence: medium. The slope
profile is `unmeasured`: no stock drawing using it is identified; confidence: high for the formula.

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
confidence). Backing: `unmeasured` for this displacement, its constants, the width threshold and
shared-image selection: the reference points are unresolved, so no scenario here predicts the
per-pixel body depth needed to isolate those values.


**F6. Centred placement** (integers, screen pixels). For a canvas *W* × *H*, an image frame whose
rectangle starts at column *c* and row *r* of the canvas, and a draw point (*x*<sub>p</sub>,
*y*<sub>p</sub>):

> top-left pixel of the image frame = (*x*<sub>p</sub> − ⌊*W*/2⌋ + *c*, *y*<sub>p</sub> − ⌊*H*/2⌋ + *r*)

*W* and *H* are positive, so ⌊·/2⌋ is the integer half rounded down (truncation gives the same).
The image frame's pixel at (*u*, *v*) inside its rectangle lands at the top-left pixel plus (*u*, *v*).

**F7. Darkening** (integers). For a hi-colour pixel with channels *r*, *g*, *b* (as stored: in the
5-6-5 layout *r* and *b* in 0…31 and *g* in 0…63):

> *r*′ = ⌊*r*/2⌋, *g*′ = ⌊*g*/2⌋, *b*′ = ⌊*b*/2⌋

Each channel is halved separately; no bit moves from one channel into another. The 5-5-5 layout
works the same way. Two darkenings give ⌊⌊*v*/2⌋/2⌋ = ⌊*v*/4⌋ per channel.

An engine that keeps 8-bit channels and packs them to hi-colour by keeping each channel's top bits
(S-0103 claim 9) gets exactly the same packed result by halving the 8-bit channel (rounding down)
before packing, also after several darkenings: ⌊⌊*C*/8⌋/2⌋ = ⌊*C*/16⌋ = ⌊⌊*C*/2⌋/8⌋ for an 8-bit
red or blue *C*, and likewise with 4 for green.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| depth clear value | 65535 | depth units | Fill after a full redraw, after a view jump, and for exposed strips and areas marked for redraw |
| depth reference start | 32768 | depth units | Depth of the view's top row in the first drawn frame after a reset of *C* (F1) |
| depth value range | 0…65535 | depth units | Values wrap modulo 65536 |
| projectile depth bias | −30 | depth units | Plus −*L(z)*; final |
| animation depth bias | −2 (ordinary), −3 (flat), −50 (tiled) | depth units | Plus type and instance adjustments and −*L(z)*; final |
| shadow depth bias | −2 | depth units | Animation shadows, plus −*L(z)*. Own shadows of sprite units: −4 − *L(z)*; generic infantry shadow over ordinary terrain: −5 − *L*(*z* − *h*) (claim 13) |
| building shadow bias | −4 | depth units | Plus −*L(z)* (−2 for remembered fogged images); final |
| terrain object bias | −12 (body), −3 (shadow) | depth units | Plus −*L(z)*; final; non-animated and animated terrain objects alike |
| projectile shadow bias | −10 | depth units | Plus −*L* of the world height of the terrain under the projectile; final (claim 13) |
| other building image biases | −3 (bib), −2 (under-door sprites), −7 (door sprite), −22 (deploying sprite), −42 (roof-deploying sprite) | depth units | Plus −*L(z)*; final; the deploying and roof-deploying values replace the normal depth adjustment (claim 13) |
| generic infantry shadow bias | −5 | depth units | Over ordinary terrain only, plus −*L*(*z* − *h*); final (claim 13). Above-deck case unresolved |
| own-shadow bias of sprite units | −4 | depth units | Plus −*L(z)*; final (claim 13) |
| remembered-building ranking weights | 37962 per cell of *x*, 37814 per cell of *y* | dimensionless, per cell | *N* in claim 16 |
| sprite constant | −2 | depth units | Already contained in the claim 13 rows for vehicles, infantry and buildings drawn as sprites; listed for reference only |
| infantry body constant | −10 | depth units | Added to an infantry body on top of the sprite constant (−12 in all); already contained in claim 13 |
| too-big vehicle constant | −16 | depth units | Added to a vehicle drawn as a sprite whose type is too big to fit under a bridge, near a bridge (−18 in all, flat profile); already contained in claim 13 |
| ordinary unit situational term | −1 | depth units | *s* in claim 14 |
| overlay depth bias | −2 − 15 × level (−17 − 15 × level for upright non-wall, non-rock types) | depth units | Level + 4 for bridge decks |
| overlay shadow depth bias | −2 − 15 × level | depth units | Every overlay kind; the cell's level alone, never + 4 for a bridge deck; flat, writer (claim 13) |
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
| ground-layer ordering transitions | 1 per tick | transitions | Drawing-position result of claim 6 |
| centring | ⌊*W*/2⌋, ⌊*H*/2⌋ | pixels | Canvas point placed on the draw point (F6) |
| ore-spawning terrain object raise | 16 | screen rows | Draw point moved up (claim 23) |
| terrain object damaged frame threshold | 2 | hit points | Image frame 1 below it (claim 23) |
| game-wide tree strength default | 25 | health points | Absent-key default of a game-wide key; carries over between games of a session (S-0006, claim 10) (claim 23) |
| unset terrain type strength | −1 | health points | A terrain type's strength before any layer gives it; writing it unsets the strength; the starting health of objects of a type whose section no layer holds (claim 23) |
| animated terrain object stage at placement | 0 | image frames | Claim 23 |
| overlay shifts | *s*<sub>type</sub>: −13 (overlay type index 126), −12 (ore and gems, walls, crates); *s*<sub>rail</sub>: −1 (railroad land type); *s*<sub>cell</sub>: −31 (bridge-deck cell with data value 9 to 17), −16 (other bridge-deck cells), −15 (index 239, cell without a bridge deck) | screen rows | Claim 24; the three terms add; indices count from 0 in the order of S-0004 |
| overlay shadow displacement on deck cells with data 9 to 17 | 15 left, 7 down | pixels | Claim 24 |
| vehicle sprite facing count that rotates | 8 | facings | Any other count shows facing part 0 (claim 22) |
| vehicle sprite facing rotation | +1 | 8-step facings | *d* = (*f*<sub>8</sub> + 1) mod 8 (claim 22) |
| vehicle firing frame step | ½ | image frames per count of the firing counter | ⌊*c*/2⌋ (claim 22) |
| turret frame rotation and base | +4 (32-step facings); 8 × walking frame count | image frames | Claim 22 |
| generic infantry shadow image frame | 1 | image frame | Claim 20 (S-0107 claim 19) |
| darkening | ½ per channel, rounded down | dimensionless factor on each hi-colour channel value | F7 |

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
| any | tick | drawing positions change by claim 6's result, using the keys before that tick's movement |
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

- **`quirks.ground_layer_single_pass_sort`** (default: on). For every switch in this section with a stated default, that default is the original YR behaviour; an alternative explicitly identified as our engine's alternative is not behaviour of the original. The ground layer has the limited settling of claim 6's drawing-position result. Alternative: fully sort the ground layer every tick (stable, by key), using the same pre-movement keys and visibility lag. With the quirk on, fast units moving up the screen can be drawn over units they have already passed behind for a few ticks.
- **`quirks.unsorted_upper_layers`** (default: on). Underground, surface, air and top layers keep
  insertion order (claim 7). Alternative: sort them like the ground layer. With the quirk on,
  overlapping aircraft are ordered by their latest layer entry or re-entry, not by screen position.
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
- **`quirks.overlay_lift_exact_levels`** (default: on). Overlay depth, and that of overlay shadows, uses 15 per level while object
  depth uses −*L(z)*; at levels ≥ 20 the two disagree by a few units (see S-0101 quirk
  `quirks.height_lift_float_scale`). Alternative: use the same lift for both.
- **`quirks.sprite_vehicle_eight_facings_only`** (default: on). A vehicle drawn as a sprite turns its
  image only when its type's facing count is exactly 8; with any other count it always shows the
  facing part 0 (claim 22). With the absent-key defaults, a type with no firing frames and no turret
  has a facing count of 1, so it never turns its image unless its art sets the count to 8.
  Alternative (off): with a count *n* that is a power of two from 2 to 256, the facing part *d* is
  the *n*-step facing (glossary; rounding of S-0204, claim 10 and F4) of the body facing turned
  clockwise by an eighth of a turn, 8192 facing units. For *n* = 8 this is exactly
  (*f*<sub>8</sub> + 1) mod 8, the formula of the on state; for *n* ≥ 8 it is
  (*f*<sub>n</sub> + *n*/8) mod *n*; for *n* = 2 and *n* = 4 the turned facing is rounded to *n*
  steps by the same rule, halves going to the next step clockwise. Explicitly, for body facing
  *f* in 0…65535 facing units, let *g* = (*f* + 8192) mod 65536. Then
  *d* = ⌊*g*·*n*/65536 + ½⌋ mod *n*, using exact rational arithmetic and returning an integer
  facing-part number in 0…*n* − 1. Thus at *n* = 2 map north gives part 0 and map east gives part 1.
  At *n* = 4 map north gives part 1 and map north-west gives part 0. Map north lies exactly at
  a rounding boundary for *n* = 4. In both states a count that is not a power of two from 2 to 256, and a
  disguised vehicle, give *d* = 0.
- **`quirks.cell_pass_shadows_write_depth`** (default: on). Shadows painted in the cell pass store
  their depth where they darken (claim 27). So two building shadows overlapping on the same level
  darken once, a sprite unit's own shadow does not show inside a building's shadow, and a unit
  standing on a building shadow is depth-tested against the shadow (claim 13). Alternative: these
  shadows darken without storing depth; then overlapping shadows always darken twice and units are
  tested against what lies under the shadow.
- **`quirks.tree_strength_fixed_at_first_section`** (default: on). A terrain type that no layer
  gives a strength takes the game-wide tree strength in force when the first layer holding its
  section is read, and keeps it (claim 23). So a map whose override section changes only the
  game-wide tree strength does not change the strength of terrain types that the base rules
  already describe. Alternative: such a type takes the game-wide tree strength in force after the
  last layer of the game has been read. D27 (v) tells the two apart.
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
| rules, building type | the key for the searchlight flag; no stock type sets it | the building has a building light effect, created when the building is placed (claim 3); used by D19 (e) | boolean | no |
| art, building image | `NormalZAdjust` | building's normal depth adjustment (body in the cell pass) | integer, depth units | 0 |
| art, building image | `ZShapePointMove` | depth-image displacement (*m<sub>x</sub>*, *m<sub>y</sub>*) in F5 | two integers, pixels | 0, 0 |
| art, building image | `SpecialZOverlay`, `SpecialZOverlayZAdjust` | an image and an adjustment that are read but have no known effect on drawing: no drawing of this image was found | image name; integer (no unit stated, since no effect is known) | none; 0 |
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
| rules, overlay type | `Wall` | overlay drawn upright with −2 bias regardless of the flat setting; −12 placement shift (claim 24) | boolean | no |
| rules, overlay type | `Tiberium` | ore and gems: flat profile, −2 bias; −12 placement shift (claim 24) | boolean | no |
| rules, overlay type | `Crate` | −12 placement shift (claim 24) | boolean | no |
| rules, overlay type | `Land` | a railroad land type gives the further −1 placement shift (claim 24) | land-type name | as S-0211 says |
| art, vehicle or aircraft image | `Voxel` | drawn as a voxel model instead of a sprite (S-0007, claim 4); an aircraft type without it shows no body (claim 21); used by D34 | boolean | no |
| art, vehicle image | `Facings` | facing count of a vehicle drawn as a sprite; only 8 turns the image with the switch on (claim 22); counts 2 to 256 that are powers of two use claim 29 with the switch off | integer, facings | 8, but 1 when the type has no firing frames and no turret |
| none | none | claim 29 reads no additional key: its switch selects the alternative and the facing count is imported above | not applicable | switch on (original) |
| none | none | claim 30 reads no ordering key; the animated and ore-spawning flags below select the drawing and its raise, and destruction is an object's state, not an imported ordering setting | not applicable | body before its own shadow |
| art, vehicle image | `WalkFrames`, `FiringFrames` | walking and firing frame counts *F*<sub>w</sub>, *F*<sub>f</sub> (claim 22) | integer, image frames | 12; 0 |
| art, vehicle image | the keys for the standing frame count, the death frame count, the death frame rate and the four start frames (standing, walking, firing, death); no stock file sets them (claim 22) | *F*<sub>s</sub>, *F*<sub>d</sub>, *R*<sub>d</sub>, *S*<sub>s</sub>, *S*<sub>w</sub>, *S*<sub>f</sub>, *S*<sub>d</sub> | integer: counts and start frames in image frames, *R*<sub>d</sub> in ticks per image frame | *F*<sub>s</sub>: 0, or 1 when *F*<sub>f</sub> > 0. *F*<sub>d</sub>: 0. *R*<sub>d</sub>: 1, and a value below 1 is raised to 1. With *n* the facing count after reading: *S*<sub>w</sub>: 0. *S*<sub>s</sub>: *F*<sub>w</sub>·*n* if *F*<sub>s</sub> > 0, else *S*<sub>w</sub>. *S*<sub>f</sub>: (*F*<sub>w</sub> + *F*<sub>s</sub>)·*n* if *F*<sub>f</sub> > 0, else *S*<sub>s</sub>. *S*<sub>d</sub>: (*F*<sub>f</sub> + *F*<sub>w</sub> + 1)·*n* if *F*<sub>d</sub> > 0, else −1. Each default is computed only when its own key is absent |
| rules, vehicle or infantry type | `WalkRate`, `IdleRate` | walk counter of a vehicle drawn as a sprite (claim 22) | integer, ticks | 1; 0 |
| rules, any unit type | `NoShadow` | suppresses the own shadow of a sprite unit (claims 20, 21), or the voxel shadow (claim 18; S-0105 claim 31) | boolean | no |
| art, vehicle image | `DisableShadowCache`, `DisableVoxelCache` | reuse controls for D23 (d); full mapping and defaults owned by S-0105 | boolean | S-0105 Import mapping |
| none | none | vehicle own-image exclusion has no ordering key (claim 18); S-0105 claim 39 owns the cached-cutout quirk and model inputs | not applicable | fresh own-image exclusions; retain exclusions on reuse |
| rules, vehicle type | `SmallVisceroid`, `LargeVisceroid` | either one suppresses the own shadow of a vehicle drawn as a sprite (claim 21) | boolean | no |
| rules, vehicle type | `TooBigToFitUnderBridge` | the *U* − 18 flat row near a bridge (claim 13) | boolean | no |
| rules, vehicle type | `Turret` | the turret path of a vehicle drawn as a sprite (claim 22); also a default of `Facings` | boolean | no |
| rules, terrain type | `SpawnsTiberium` | the 16-row raise (claim 23); the only animated terrain objects whose stage advances (ore mines, S-0401 claim 15) | boolean | no |
| rules, terrain type | `IsAnimated` | body frame from the animation stage, drawn in the object pass (claim 23); the stage advances only for an ore mine (S-0401 claim 15) | boolean | no |
| rules, terrain type | `Strength` | the terrain type's strength: starting health, and so image frame 0 or 1 (claim 23); −1 unsets it | integer, health points | unset (−1); after the first layer holding the type's section is read, the game-wide tree strength then in force (claim 23) |
| rules, terrain type | `AnimationRate`, `AnimationProbability` | the timing and start of an ore mine's animation (S-0401 claim 15); used by D27 (vi) | integer, ticks; decimal | S-0401 |
| rules, `[General]` | `TreeStrength` | game-wide tree strength (claim 23) | integer, health points | 25, until a rules layer first gives the key in the session; after that a game-wide key keeps the value an earlier game of the session gave it (S-0006, claim 10) |

## Measurement scenarios

**Common setup.** The measurement setup of the glossary: an unmodified install with no loose
override files other than a scenario's named test files and no engine add-ons; stock data only; purpose-made
test maps. Where a scenario changes a
type's field, it does so through the test map's override section, or, for art data, through a
named loose copy of the art file (the glossary's named test files; test data, removed afterwards).
Skirmish only, with the director and mods off. Use one bystander computer player at the farthest
start position, unreachable from the observed test area during the run, and void the run if it
enters that area, affects a test object or is defeated before observation ends. Reveal the test
area to the local player before recording; no shroud covers the compared pixels. Screen resolution
800×600, game speed 2 unless a scenario says otherwise, fog of war off unless stated. The *test house* is the
local player, whose view the captures show. The observable is read from
screenshots or a capture of the battlefield view, one image per drawn frame; a unit's position is
read from its cell and the ticks it has moved at a known speed. Where a scenario counts ticks, each captured
picture is matched to its tick by capturing at a game speed at which the number of pictures over a
long interval equals the number of ticks (see the glossary's *drawn frame* and *display path*, and
S-0201, claim 6).
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
  the flying aircraft, if the flat projectile is drawn over the vehicle, or if the non-flat
  projectile is covered by the vehicle.
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
  in the second picture after that tick (the outcome of a full sort), or if in (a) the first picture
  showing the mover on top is not the picture after tick *n* + 1, where tick *n* is the first tick
  whose movement makes its key exceed the other's. A fast-run picture whose mover order differs
  from claim 6's drawing-position table also falsifies it. If the movement rules never permit a
  drop past two keys in one tick, (b)'s multi-position settling is `unmeasured` because this setup
  cannot distinguish it from a full sort.
  (c) Equal-key stability. Use two map-placed sprite vehicles at (x, y) and (x + 1, y − 1),
  whose cell-centre keys are equal. Named test files replace their sprites with two distinct,
  opaque test squares on 129 × 129-pixel canvases and set their art to sprite drawing with one
  walking frame, one facing and no firing frames. Set their idle rates to 0 and suppress their
  shadows through the map override. These test images overlap despite the 60-column separation.
  After at least ten stationary ticks, record which colour covers the overlap. Keep both still
  for twenty more ticks, then drive a third vehicle past both in each direction without touching
  their cells. Expected: the two equal-key colours keep the recorded overlap priority whenever
  not covered by the third vehicle. Any reversal falsifies equal-key stability. Initial priority
  is recorded, not predicted; this does not test entry placement.
- **D4, building keys (claim 5).** Setup: a flat map; a building with a voxel turret whose turret
  image overlaps the route below, and a vehicle driving straight down the column of cells just west of
  the foundation (cells with x one less than the foundation's first column), from north of the
  building to south of it. A turret drawn as a sprite is not a building part but a slot animation
  with its own key (claim 17), tested by D22. (a) The building as stock data has it (a type that is
  not a gate). Observable: the vehicle's key (X + Y, from its position) at the tick from which it is drawn over the building's
  voxel turret. Expected: the switch happens at the first tick at which the vehicle's key is strictly
  greater than the building's key of F4, i.e. 256·(*c<sub>x</sub>* + *c<sub>y</sub>*) + 32, and
  becomes visible one tick later (claim 6).

  A vehicle with key *K* where 256·(*c<sub>x</sub>* + *c<sub>y</sub>*) ≤ *K* < 256·(*c<sub>x</sub>* +
  *c<sub>y</sub>*) + 32 is drawn under the voxel turret. Falsified if the switch happens near the
  key of the foundation's centre (256·(*c<sub>x</sub>* + *c<sub>y</sub>*) + 128·(*w* + *h*)) instead.

  (b) Gate term. The same layout, with the building type given the gate flag through the test
  map's override section (a rules key, Import mapping). The vehicle and the building belong to the
  test house, and the vehicle's route stays off the foundation, so it never passes through the
  building. Observable and procedure as in (a). Expected: the switch happens at the first tick at
  which the vehicle's key is strictly greater than 256·(*c<sub>x</sub>* + *c<sub>y</sub>*) + 16
  (F4 with *t* = 32 and *g* = 16), visible one tick later, so a vehicle with key *K* where
  256·(*c<sub>x</sub>* + *c<sub>y</sub>*) + 16 < *K* ≤ 256·(*c<sub>x</sub>* + *c<sub>y</sub>*) + 32
  is drawn over the voxel turret here and under it in (a). Choose a vehicle speed at which some
  tick's key falls in that range. Falsified if the switch happens at the key of (a), + 32. A run in
  which the building is drawn opening or closing as a gate (claim 13) is void; if every run opens
  it, (b) is not measurable.
- **D5, air/top insertion order (claim 7).** Setup: two aircraft of the same type take off one
  after the other and fly crossing paths. Observable: which aircraft is on top where they overlap, and
  which is lower on screen. Expected between layer re-entries: the aircraft that entered
  later stays on top, even when the screen-position order reverses. An order reversal without
  re-entry would falsify claim 7. However, flight re-entry events remain unresolved (claim 4),
  so an order change during straight flight is inconclusive, not itself a falsifier: record it
  to help settle that question. Take-off order alone does not predict all later overlaps.
  This is an exploratory scenario until those events can be identified independently.
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
  depth unit of the switch, Comparing two drawings); its shadow (flat, −4 against the wall's −2) is
  cut along a fixed row, 0 to 2 rows above the wall's last row, whatever the infantry's position,
  because a flat drawing's depth depends only on the screen row. Falsified if a cut row sweeps up the infantry body over several
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
- **D15, depth biases by comparison (claims 10, 12, 13, 27).** Setup: a flat map at level 0; an
  infantry unit as probe (sprite, upright, body bias *U* − 12 = −13 with all fudge factors 0 and
  *s* = −1; its own shadow flat, −4), and animation types chosen with no vertical draw shift and no
  depth adjustment in their art. Each case is predicted with *Comparing two drawings* (F2) and the
  final biases of claim 13; the observable and the predicted value are given per case. Rows are
  counted from the bottom of the probe's drawing rectangle (F2).
  - Flat writers under the probe's shadow (the probe stands so that its own shadow falls on the
    image, away from any body; flat over flat, so the comparison is the same on every row),
    observable: whether the probe's shadow darkens the image's pixels. Expected: inside a building
    shadow (−4) the probe's shadow does not show (the pixels stay darkened once); on a bib (−3), a
    terrain-object shadow (−3), under-door sprites (−2) and ore and gems (−2) it shows. The probe's
    body loses no rows on any of these flat writers (its bias −13 is nearer than all of them).
  - Upright writers in front of the probe (the probe walks toward the viewer past the image, one step
    at a time), observable: the switch distance *b*<sub>probe</sub> − *b*<sub>image</sub> at which the
    probe stops being hidden (all of its rows in the overlap shown). Expected: wall, rock and gate
    body while opening (−2, for a type whose normal depth adjustment is absent) −7 to −6 rows, that
    is, the probe shows over them while its last row is still 6 or 7 rows above theirs; a terrain
    object that is not animated (−12) 0 to 2 rows; a non-flat overlay that is neither wall nor rock
    (−17) 3 or 4 rows (D11). Normal and unloading building bodies are excluded from this numerical
    comparison: F2's comparison without a companion depth image cannot predict their cutoffs,
    and different body sprites can have different rectangle bottoms. Their biases remain
    `unmeasured` here for those reasons (claim 13).
  - Upright readers behind a wall overlay (−2), observable: the smallest distance
    *b*<sub>reader</sub> − *b*<sub>wall</sub> at which the reader shows over the wall. Expected: an
    ordinary animation (−2) shows from 1 to 3 rows; a tiled animation (−50) shows already from −35
    to −33 rows, i.e. while its last row is up to 33 to 35 rows above the wall's.
  - Flat readers passing behind a wall overlay (−2), observable: the row distance above the wall's
    last row from which the reader is hidden. Expected: projectile at height 0 (−30) 20 or 21 rows,
    about (3/4)·*L(z)* rows more at height *z*; projectile shadow (−10, over ground of level 0) 5 or 6
    rows; the probe's own shadow (−4) 0 to 2 rows; flat animation (−3) 0 or 1 row; animation shadow
    (−2) 0 rows.

  Falsified if any observed count or distance lies outside the range the formula gives for *q* in
  0…2 (the ranges above), allowing one more row where the reader's or writer's image is transparent
  at the predicted row, or if a flat-over-flat case of the first part goes the other way.
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
  rows; C at 300 leptons, 43 rows; B at 500 leptons, 72 rows; each within ±2 rows. Falsified if any run
  differs from its expected lift by more than 2 rows; in particular, if A's
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
  - (e) Building light effects: no stock building type casts a searchlight, so a stock building
    type is given the searchlight flag through the test map's override section (Import mapping)
    and placed by the map near the vehicle, so that its beam sweeps across the vehicle and the
    aircraft's path. Expected: air: the beam covers the vehicle and is covered by the flying
    aircraft. Falsified if the vehicle covers the beam or the beam covers the flying aircraft. Not
    measurable if no beam is drawn with the test lighting of S-0104 (then repeat with a darker
    ambient light).

- **D20, depth reference reset at view jumps (claim 8, F1).** Setup: a flat map at level 0; a wall
  segment (upright writer, −2), and an infantry unit of the test house standing still beside it so
  that its own shadow (flat reader, −4) covers the wall's last three rows over part of their width.
  Record only drawn frames in which the probe shows the same image frame. Let *s* be the screen row
  of the wall image's last rectangle row, counted from the battlefield view's top row (0), in the
  first drawn frame after a view jump. Procedure: (i) scroll the view by single small steps (scrolls
  only) through at least six positions while the wall stays in view; (ii) twelve times, make a view
  jump to a distant part of the map, then a view jump back that brings the wall into view. The jump
  back must land at varying pixel positions: a jump to a cell (for example a click on the radar map)
  may always land with the same top row mod 3, so use a jump whose landing follows a unit's exact
  position, such as centring the view on a selected vehicle that drives slowly to and fro near the
  wall, and record *s* after each landing. Observable: the number *k* of the wall's bottom rows,
  counted up from its last row, whose pixels inside the shadow's outline are darkened, after each
  scroll step and after each jump back, with *s* for each jump back. Expected (F1, F2 with *C* =
  32768 + the world pixel row of the view's top row after the jump; the shadow's rows at distance
  *j* above the wall's last row show iff *j* + ⌊*j*/3⌋ < 2 − *q*, with *q* = (−*s*) mod 3 the amount
  the rounding Q removes from the wall's anchor): in (i), *k* never changes; in (ii), *k* = 2 − ((−*s*)
  mod 3), so *k* is 2 when *s* mod 3 = 0, 0 when it is 1 and 1 when it is 2, the same for every jump
  with the same *s* mod 3. Falsified if *k* is the same after every jump although *s* mod 3 took all
  three values (the alternative of `quirks.depth_reference_reset_on_jump`), if *k* changes during the
  scrolls of (i), or if two jumps with the same *s* mod 3 give different *k*. Inconclusive, not
  falsified, if fewer than three values of *s* mod 3 occur in the twelve jumps (then repeat with
  another way of jumping), or if the wall's last rectangle row is transparent where the shadow
  covers it (then choose another place along the wall).
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
    run differs from the run with 0. Repeat the explicit-zero-slot run with a named loose copy
    of the art file changing only this animation type's depth adjustment to +60 depth units.
    For this repeat, choose a qualifying pixel visible in the zero-slot control that would be
    hidden by an added +60 depth units under F2; otherwise reposition the wall and repeat both
    controls. Expected: identical qualifying pixels to the unchanged-art zero-slot run, because the slot
    replaces the type's value. Any changed depth acceptance falsifies replacement. Keep the
    animation's vertical draw shift, image and slot displacement unchanged in this comparison.
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

  - (d) Own vehicle image (Q-0213). Use S-0105 MS-27's named test model, cache controls and
    no-shadow comparison at speed index 2, with one bystander computer player outside the observed
    area. Capture each matching pose twice on flat ground, on a ramp and for a hovering vehicle
    resting and rocking. Expected: with fresh shadows, every qualifying own-body overlap pixel
    equals the no-shadow control, and qualifying footprint pixels outside the own image darken.
    With shadow reuse, a newly filled body pixel can darken and an earlier exclusion can persist
    as a gap, including sharing between same-type objects. Repeat the two frame orders after
    program restart as MS-27 specifies. Darkened own pixels in the fresh runs, a reversal on
    slopes or hovering, or loss of retained exclusions with verified reuse falsifies the claim.
    Changed keys or invalidated caches void the reuse comparison. The full frame and rocking
    controls, turret/barrel inclusion and freshness rules are owned by S-0105 claim 39 and MS-27.

- **D24, centred placement (claim 19).** Setup: a flat map at level 0 with a stock building, a
  stock infantry unit standing alone in a cell (its position from the standing-position rules of
  S-0208) and a stock tree. Prefer sprites whose canvas width or height is odd. For each, read from
  its shape file the canvas *W* × *H* and the rectangle of the image frame shown (format data, not
  copied into any spec). Observable: the screen column and row of one chosen non-transparent pixel
  (*u*, *v*) of the image frame. Expected: F6 with the draw point of S-0108 claim 8, claim 20 and
  claim 23. Falsified if any pixel is off by one or more columns or rows, in particular by one for
  an odd canvas size (the result of rounding the half up), or by the frame rectangle's position (the
  result of centring the image frame instead of the canvas).
- **D25, infantry placement (claim 20).** Setup: a flat map at level 0. (a) An infantry unit
  standing still. Observable: the screen position of a chosen pixel of its body frame and of its
  own shadow frame. Expected: both by F6 on the one draw point, the screen pixel of its position.
  (b) A jumpjet infantry unit hovering still at its cruise height *h* over a flat cell (D18).
  Observable: the screen row of a chosen pixel of the generic shadow, and the screen row of the
  unit's health bar compared with a standing unit on the same cell, which gives its lift. Expected:
  the generic shadow image frame 1 placed by F6 on the point *L*(*h*) rows below the unit's draw
  point. Falsified if either part is off by more than one row or one column. No high-bridge cell
  is used in either part; the unresolved above-deck shadow is not tested.
- **D26, overlay placement (claim 24).** Setup: test maps with flat cells at level 0 and at level 2,
  carrying a wall, an ore field, a crate, a fence (an overlay of none of the shifted kinds), an
  overlay type whose land type is railroad, the overlay types with indices 126 and 239 (counted from
  0) in the stock overlay list (whatever types the stock rules put there; read them, copy no value), and a high
  bridge whose deck cells carry overlay data values inside and outside 9 to 17. Observable: the
  screen row and column of a chosen pixel of each overlay's image frame and, where it has one, of
  its shadow frame. Expected: claim 24 with F6. Falsified if any shift differs from the claim by one
  row or more, or if the horizontal displacement differs from claim 24 (0 normally; 15 columns
  left for the specified deck-cell shadow).
- **D27, terrain objects (claim 23).** Setup: a stock tree type on a flat level-0 cell, the same
  type on a flat level-2 cell and on a sloped cell, and a stock terrain type that spawns ore on a
  flat level-0 cell. Observable: (i) the screen position of a chosen pixel of each object's image
  frame and shadow frame, and which image frame is shown at the start; (ii) an infantry probe (D15)
  walks toward the viewer past the ore-spawning object, one step at a time, recording the switch
  distance *b*<sub>probe</sub> − *b*<sub>image</sub> measured from the drawn image's last row.
  Expected: (i) F6 on the screen pixel of the cell centre at its surface height (S-0106), 16 rows
  higher for the ore spawner, image frame 0; (ii) 0 to 2 rows, the same as for a tree that is not
  raised (D15), because the depth anchor moved up with the image. Falsified if part (i) is off by one
  or more pixels or shows another image frame, or if in part (ii) the switch comes about 12 rows later than
  that (the anchor left at the unraised row). (iii) Image frame 1: the tree type of (i) on a flat
  level-0 cell, in two games: its strength set through the test map's override section to 1 in
  one and to 2 in the other (pre-placed objects follow the override, S-0009). Observable: the image
  frame shown at the start. Expected: image frame 1 with strength 1, image frame 0 with strength 2.
  Falsified if the two games show the same image frame.
  (iv) Strength of a terrain type. A stock tree type whose section in the stock rules gives no
  strength (check the stock rules; copy no value), on a flat level-0 cell, in two games: the test
  map's override section sets that type's strength to −1 and the game-wide tree strength to 1 in
  one, and to −1 and 2 in the other. Observable: the image frame shown at the start. Expected:
  image frame 1, then image frame 0 (the map's layer sets the unset strength to its own game-wide
  tree strength). Falsified if both games show the same image frame.
  (v) The quirk `quirks.tree_strength_fixed_at_first_section`. The same tree type and cell; the
  test map's override section sets only the game-wide tree strength, to 1, and nothing in the tree
  type's section. Observable: the image frame shown at the start. Expected: image frame 0 if the
  stock rules' game-wide tree strength is 2 or more (the type took it in the base rules and keeps
  it); image frame 1 is the quirk-off result. Not measurable if the stock game-wide tree strength
  is below 2. When the stock value is 2 or more, any starting image frame other than 0 falsifies
  the retained-strength rule.
  (vi) Animated terrain objects. Part 1: a stock terrain type marked animated that has the
  spawns-resources flag (an ore mine), placed by the map on a flat cell, with its animation
  probability set to 1 and its animation rate to 4 ticks through the override section, so that its
  animation starts in its first turn. Observable: per tick, the image frame shown (matched against
  its shape file of *N* image frames). Expected: image frame 0 in the first drawn frame; then image
  frames 0, 1, …, ⌊*N*/2⌋ − 1, each for 4 ticks, then image frame 0 again, the sequence repeating:
  the stage returns to 0 in the turn that would show image frame ⌊*N*/2⌋ and a new animation starts in
  the next turn, so between two sequences image frame 0 shows for 5 ticks. Image frame ⌊*N*/2⌋ never
  appears. Part 2: the same type
  with its spawns-resources flag cleared through the override section. Expected: image frame 0 in
  every drawn frame over at least 1000 ticks (and no 16-row raise). Falsified if part 1 shows another
  sequence, or image frame ⌊*N*/2⌋, or if part 2 ever shows another image frame. Not measurable if no stock
  animated type can be placed on a test map.
- **D28, smudges (claim 25).** Setup: test maps with flat cells at level 0 and at level 2, a
  one-cell smudge and a smudge several cells wide, placed by the map's smudge list. Observable: the
  screen position of a chosen pixel of each smudge's image frame. Expected: image frame 0 placed by
  F6 on the top corner of the smudge's first cell, lifted 15 rows per level. Falsified if it is
  centred on the cell centre (15 rows lower), on each piece's own cell, or differs from F6 by any
  other pixel displacement.
- **D29, vehicles drawn as sprites (claims 13, 21, 22).** Setup: a stock land vehicle type drawn as
  a sprite whose facing count is 8 (its art's facing count, or the absent-key default of 8 for a
  type with firing frames or a turret, Import mapping), with several walking frames and an idle
  rate of 0 (choose it by reading the stock rules and art; copy no value; if its idle rate is not
  0, set it to 0 through the test map's override section). Not measurable if stock data has no
  such type. A flat map at level 0; the vehicle belongs to the test house.
  - (a) Order the vehicle to move one cell in each of the eight directions and to stop.
    Observable: per tick, the image frame shown (matched against the image frames of its shape file) and
    its body facing. Expected: the facing part *d* = (*f*<sub>8</sub> + 1) mod 8; while moving, the
    walking frames of that facing in turn, one step in each tick whose tick number is a multiple of
    its walk rate; after stopping, *S*<sub>s</sub> + *d*·*F*<sub>s</sub> if *F*<sub>s</sub> > 0,
    otherwise *S*<sub>w</sub> + *d*·*F*<sub>w</sub> (the standing-pose state is on again, claim
    22). Falsified if the facing parts run counter-clockwise, start at another direction than map
    north-west, if the walking image frame steps at another rate, or if an image frame shown after
    stopping is another image frame, in particular image frame *d* where it differs from the
    expected one.
  - (b) The vehicle stands still, alone, on open ground. Observable: the screen position of a
    chosen pixel of its body frame and of its shadow frame. Expected: both by F6 on the screen
    pixel of its position, the shadow being the body frame plus ⌊*N*/2⌋. Three further games, each
    with one flag set for the type through the override section: the shadow-suppressing flag, the
    small-visceroid flag, the large-visceroid flag (Import mapping). The visceroid flags also switch
    on behaviour of their own, so the unit is kept alone and only the first drawn frames after the
    start, while it stands on the ground, are read. Expected: no shadow frame is drawn in any of the
    three games. Falsified if either position is off by one or more pixels, another shadow frame
    shows, or a shadow shows in any of the three flagged games.
  - (c) Turret path. The same type with the turret flag set through the override section, standing
    still; enemy units are placed in turn in each of the eight step directions from it, on cells
    in line with it, within reach of its weapon, so that it turns its turret toward each. Observable:
    the two image frames that make up its image (body and turret, matched against the shape file),
    and the body's pixels inside the outline of its shadow frame. Expected: once the turret has
    stopped turning toward a target in step direction *j* (glossary; its 32-step facing is 4*j*), the
    turret frame is ((4*j* + 4) mod 32) + 8·*F*<sub>w</sub>, drawn over the body frame; the body's
    pixels inside the shadow's outline are not darkened (unlike D32). Then, with the fudges set to
    0 as in D33, the vehicle stands so that its shadow falls on a bib, away from any body, and in a
    second position on a tree's shadow: the shadow (flat, *U* − 2 = −3) ties with both (−3) and
    darkens neither, while the same vehicle without the turret flag darkens both with its own
    shadow (−4, D33 (b), D31 (c)). Falsified if the turret frame differs, if the shadow darkens the
    body or turret pixels, or if the turret vehicle's shadow darkens the bib or the tree's shadow. Not measurable if the shape file
    has fewer than 8·*F*<sub>w</sub> + 32 image frames before its shadow half, or if the vehicle
    does not turn its turret toward the targets.
  - (d) Facing counts other than 8. A named loose copy of the art file in which the type's facing
    count is 16. The vehicle moves as in (a). Expected: the facing part is 0 in every direction:
    while moving, the walking frames *S*<sub>w</sub> + (*w* mod *F*<sub>w</sub>), whatever the body
    facing. Falsified if the image turns with the body facing (the alternative of
    `quirks.sprite_vehicle_eight_facings_only`).
  - (e) Dying row and death counter. A named loose copy of the art file in which the type's death
    frame count is 4, its death start frame is 0 and its death frame rate is 2 (keys by role in the
    Import mapping). The vehicle stands still and is destroyed by the weapon of a second house,
    and a second game uses the stock art. Observable: per tick, from the tick of destruction until the
    vehicle is gone, the image frame shown. Expected: image frames 0, 1, 2 and 3 in that order,
    each shown for 2 ticks (the first may show for 1 tick only, depending on where in the tick
    the destruction falls), then 3 until the vehicle is removed, whatever its body facing; in the
    stock-art game no dying frames are shown (S-0312). Falsified if the image frames depend on the body
    facing, run in another order or at another rate, or if the stock-art game shows the dying row.

  - (f) Standing-pose state. The vehicle of (a), of a type with firing frames, so that
    *F*<sub>s</sub> > 0 by the absent-key defaults (Import mapping; if the chosen type has none,
    give it standing frames through a named loose copy of the art file), placed by the map facing a
    direction with *d* ≠ 0. Observable: the image frame in the first drawn frame of the game, in
    every drawn frame while it turns on the spot before its first step toward a cell behind it,
    and in the first drawn frame after it has driven one cell and stopped. Expected:
    *S*<sub>s</sub> + *d*·*F*<sub>s</sub> at the start and after stopping, *d* being the facing part
    of its body facing at that moment; while it turns, the moving row or this standing row, never
    image frame *d* alone. Falsified if any of these pictures shows image frame *d* where it differs
    from the expected image frames (image frame *d* while turning would show a state turned off before the
    turn).

- **D30, darkening (claim 26).** Setup: a flat map at level 0 with the test lighting of S-0104 and a
  display path that presents a 5-6-5 surface (S-0103, *Reading a pixel*). An infantry unit stands
  still on bare ground. Procedure: capture the ground pixels inside its shadow's outline; move the
  unit away and capture the same pixels without the shadow. Repeat (b) with another ambient light
  level and (c) with a coloured light source close by. Observable: per pixel, the 5-6-5 channels
  with and without the shadow. Expected: every shadowed channel is ⌊*v*/2⌋ of the unshadowed value,
  at every pixel of the outline whatever the shadow frame's palette index there, in all three
  lightings. (d) Two infantry units of the same type belonging to houses of different colours, each
  casting its shadow on the same kind of ground in turn: the shadowed values are identical. (e) A
  tree's shadow and an infantry unit's shadow on the same ground pixels in turn (the tree's sprite
  and the soldier's sprite are drawn with different palettes): the shadowed values are identical.
  Falsified if any channel differs from ⌊*v*/2⌋ (for example a rounded-up half, a darker palette
  shade, or a result that follows the light), or if parts (d) or (e) give different values.
- **D31, overlapping shadows (claim 27).** Setup and reading as in D30. Each part compares a pixel
  under the overlap with the same pixel with no shadow (value *v* per channel).
  - (a) Two infantry units in one cell whose own shadows overlap. Expected ⌊*v*/4⌋.
  - (b) An infantry unit standing so that its own shadow reaches into a building's shadow.
    Expected ⌊*v*/2⌋ inside the building's shadow (the unit's shadow does not show there).
  - (c) An infantry unit whose own shadow falls on a tree's shadow, and one whose shadow falls on a
    bib. Expected ⌊*v*/4⌋ on the tree's shadow and ⌊*v*/2⌋ of the bib's own pixel on the bib.
  - (d) A building placed so that its shadow overlaps a tree's shadow. Expected ⌊*v*/4⌋.
  - (e) Two buildings whose shadows overlap. Expected ⌊*v*/2⌋; and a jumpjet infantry unit hovering
    over ordinary terrain (no high-bridge cell), so that its generic shadow falls inside a
    building's shadow: ⌊*v*/4⌋.
  - (f) A stock projectile with a shadow passes over a building's shadow, then a tree's shadow,
    on level-0 ordinary terrain. Compare only pixels outside the projectile body and other
    effects. Expected ⌊*v*/4⌋ in either overlap. A run with no qualifying overlap is inconclusive.
  - (g) A stock animation with a shadow, no vertical draw shift and no depth adjustment, created
    at height 0 as in D15, overlaps in turn a building's shadow, a tree's shadow, a bib, and ore
    or gems. Record pixels inside its shadow but outside its body. Expected: the earlier shadows
    remain ⌊*v*/2⌋, and bib/ore/gem pixels retain their control-game values without the animation
    shadow. A part with no qualifying overlap is inconclusive.

  Falsified if any part gives another value. A result of ⌊*v*/4⌋ in (b) or in the two-building part
  of (e) is the alternative of `quirks.cell_pass_shadows_write_depth`.
- **D32, a sprite unit's shadow over its own body (claim 28).** Setup and reading as in D30. Run 1:
  a stock infantry type whose shadow frames overlap its body frames (check in its shape file), one
  unit standing still. Run 2: the same type with its shadow suppressed through the test map's
  override section, the unit at the same place and facing. Observable: the body's pixels inside the
  shadow frame's outline. Expected: in run 1 each channel is ⌊*v*/2⌋ of the run 2 value. Falsified
  if any qualifying channel differs from ⌊*v*/2⌋, including unchanged body pixels where the
  expected halving differs from *v*.
- **D33, depth of vehicles drawn as sprites and of animated terrain objects (claims 3, 12, 13).**
  Setup: a flat map at level 0, the reading of D15, and the vehicle type of D29 with all four
  fudges set to 0 through the override section (its body bias *U* − 2 = −3 with *s* = −1; its own
  shadow flat, −4).
  - (a) Body. The vehicle drives toward the viewer past a wall segment (upright writer, −2), one
    step at a time, its image overlapping the wall's. Observable: the switch distance
    *b*<sub>vehicle</sub> − *b*<sub>wall</sub> at which the vehicle stops being hidden (all of its
    rows in the overlap shown). Expected (Comparing two drawings): 0 to 2 rows; the infantry probe
    of D15 (−13) switches at −7 to −6 rows against the same wall. Falsified if the vehicle's switch
    distance lies outside 0 to 2 rows, allowing one more row where an image is transparent at the
    predicted row (for example −7 to −6, which an infantry body's −10 would give).
  - (b) Own shadow. The vehicle stands so that its own shadow falls into a building's shadow, away
    from any body, and in a second position on a bib. Observable: whether its shadow darkens the
    pixels there (reading of D30). Expected: inside the building's shadow it does not (the pixels
    stay darkened once); on the bib it does. Falsified if either goes the other way.
  - (c) Animated terrain object. A stock terrain type marked animated (Import mapping; choose it by
    reading the stock rules; copy no value), placed by the map on a flat cell where the theater
    allows it. Not measurable if no such type can be placed on a test map. An infantry probe (D15)
    walks straight toward the viewer past it, one step at a time, its image overlapping the
    object's; then a wall segment is placed in front of the object, its image overlapping the
    object's. Observable: per position, which of the two is on top in the overlap, the probe's key
    (X + Y from its position) and the object's key (X + Y of its cell's centre); and the switch
    distance *b*<sub>object</sub> − *b*<sub>wall</sub> at which the object's pixels inside the wall's
    image are all shown (the object stands still, so choose the wall's cell to reach the distances
    around the switch, one map per distance). Expected: the probe and the object, both readers in
    the ground list, never hide each other through depth: the probe is drawn under the object while
    its key is smaller than the object's and over it once its key is greater, the change visible
    one tick after the key passes (claim 6), whatever their rows. Against the wall the object
    (−12) shows from a switch distance of −6 rows (one row either way allowed for transparency).
    Falsified if the probe's order against the object follows the rows instead of the keys (the
    result of a writer, as for a non-animated tree in D15), or if the wall switch distance differs
    from −6 rows by more than one row.
- **D34, aircraft without a voxel model (claim 21).** Setup: a flat map at level 0; a named loose
  copy of the art file in which the voxel flag of one stock aircraft type's art section is cleared
  (Import mapping), so that its sprite file is looked up instead (S-0007). The aircraft is placed
  on the map, belongs to the test house, takes off and flies across open ground; a second game
  uses the stock art. Observable: whether any image of the aircraft's body appears at its
  position; its status extras, with the aircraft selected, show where it is. Expected: no body
  image in the first game, while the selected aircraft's status extras move across the map; in the
  stock-art game the voxel body shows. Falsified if any sprite image of the aircraft appears in the
  first game. (Whether its shadow shows is not part of the claim.) Not measurable if the original does not
  start a game with the changed file.

- **D35, overlay shadows (claims 13, 16, 27).** Setup and reading as in D30, on a flat test map
  at level 0 and on a copy with the same layout at level 2. Each part compares a ground pixel with
  the same pixel in a control game whose map lacks the overlay that casts the shadow (value *v* per
  channel). Choose the overlays by reading the stock data (copy no value); a part is not measurable
  if its overlay's sprite has no shadow image frames or its shadow does not reach the pixels named.
  - (a) An ore overlay whose shadow falls partly on bare ground. Expected ⌊*v*/2⌋ there. Further
    pixels of that shadow are covered in turn by: a tree's shadow (terrain object placed by the
    map), a building's shadow, the own shadow of an infantry unit standing still, and the shadow of
    an animation type with the shadow flag, created as in D15 at height 0 above the terrain. Expected ⌊*v*/4⌋ under the tree's,
    the building's and the infantry unit's shadows, and ⌊*v*/2⌋ under the animation's shadow. The
    tree part shows the bias is above −3 and the animation part that it is at most −2, so together
    they fix −2.
  - (b) Two ore cells side by side such that the shadow of one falls on the ore pixels of the
    other. Expected: those ore pixels are the same as in a game without the first cell's ore (not
    darkened), read in the first drawn frames, before resource growth or spread changes either
    cell. Where the two cells' shadows overlap on bare ground: ⌊*v*/2⌋.
  - (c) An upright overlay type that is neither a wall nor a rock (its type not drawn flat), whose
    shadow overlaps a tree's shadow on bare ground. Expected ⌊*v*/4⌋ in the overlap (with the
    −17 of the upright body it would be ⌊*v*/2⌋).
  - (d) Parts (a) and (c) on the level-2 map. Expected: the same results (the shadow's bias follows
    the cell's level as the other drawings do).
  - (e) A high bridge over dry ground, a deck cell whose shadow (with the displacement of claim 24
    where it applies) falls on ground outside the deck's pixels, and a tree on a cell of the same
    height level as the deck cell's own cell (the low ground under the deck, not a bank 4 levels
    up), placed so that its shadow also falls there. Not measurable if no such layout can be
    built. Expected ⌊*v*/4⌋ in the overlap (with the deck's 4 levels added the overlay
    shadow would be 60 depth units nearer, and the tree's shadow would not darken it again:
    ⌊*v*/2⌋).

  Falsified if any part gives another value.

- **D36, animated terrain body/shadow overlap (claim 30; Q-0051).** Use the common
  measurement setup, game speed 2, a fixed view, no fog and a flat level-0 test area with no
  overlays or buildings beside the test object. Capture lossless pixels in the 5-6-5 display
  layout as in D30. One run per part; keep the bystander outside this area.
  - (a) Place a stock terrain type marked animated. Choose the type and a displayed body frame
    by reading its shape file: require at least one non-transparent body pixel inside the
    non-transparent outline of its corresponding shadow frame. Record the displayed body frame
    and the pixels in that overlap, excluding pixels covered by another effect. Compare each
    channel with the body's lit colour before shadow darkening, calculated with S-0104 and
    S-0112 claim 3. Expected: where both depth tests pass, the result is the channel's rounded-down
    half (claim 26), not the undarkened body colour. Ground pixels outside the body but inside
    the shadow provide a control for shadow visibility and darkening.
  - (b) Repeat with a stock ore mine; capture each distinct displayed body frame while its
    animation runs. Match the body and shadow at the common draw point raised by 16 rows.
    Expected: the same rounded-down half in every qualifying overlap, not an undarkened body
    or a shadow left at the unraised draw point.
  - (c) Place a stock destructible terrain object, and have a human-controlled stock armed unit
    destroy it by firing at its cell. Capture every drawn frame through destruction. Identify
    each displayed body frame from the shape file rather than assuming the still-open sequence
    timing. For each visible destruction frame with qualifying body/shadow overlap, compare
    the pixels as in (a), excluding weapon, fire and other effect pixels. Expected: the same
    channel halving where both tests pass, including an otherwise non-animated type once it
    uses the object pass. Record which frames qualify; a case with no qualifying visible
    overlap is inconclusive, not evidence for the opposite order.
  - Falsified if any qualifying overlap shows the undarkened body where shadow darkening should
    pass, or if the mine or destruction state reverses the result. Use pixels whose unshadowed
    channel value is nonzero, so that the two outcomes differ. A separate-shadow-pass
    alternative can also be tested by placing a second animated terrain object later in the
    ground list whose body overlaps the first object's shadow: its pixels must remain
    undarkened by the first object's shadow wherever its own body passes depth. Record both
    objects' positions and placement order and check claim 6's list order. Falsified if the
    first object's shadow darkens a qualifying body pixel of the later object.

- **D37, off-state sprite vehicle facings (claim 29; Q-0050 part 2).** This is an engine
  test, not a measurement of the original. Use an undisguised sprite vehicle, turn the switch
  off and record its integer facing part for every body facing in 0…65535, once for each
  supported facing count. Compare with the exact rational formula in *Edge cases and quirks*.
  Boundary checks: count 2 gives part 0 at facing 8191 and part 1 at 8192, part 1 at 40959 and
  part 0 at 40960. Count 4 gives part 0 at 65535 and part 1 at 0, part 1 at 16383 and part 2 at
  16384. Counts 8 and above must equal the glossary's coarse facing plus one eighth of the
  count, modulo the count. Repeat disguised and with unsupported counts; expected part 0.
  Any different part falsifies the alternative's implementation, not a claim about the original.

- **D38, upright reader clipped at the bottom (F2).** Use the common setup and a sprite vehicle
  with all terrain fudges 0, on ordinary level-0 terrain. Put an upright overlay that is neither
  wall nor rock in front of it (set its flat flag off through the map override if needed), with
  bias −17 and no companion depth image. The vehicle has bias −3. Choose a placement whose
  unclipped rectangle bottoms differ by at least 15 rows and whose non-transparent pixels
  overlap. First capture both rectangles fully inside the view: the vehicle must show throughout
  the overlap by F2. Scroll, without a view jump, until the overlay's last row lies 5 rows above
  the view's bottom and the vehicle's last row lies below the view. Record a qualifying shared
  pixel still inside the view and the rectangle rows in both captures. Expected: in the second
  capture, the reader's anchor is the view's bottom, only 5 rows below the writer's anchor, so
  its pixels in the overlap are hidden. The alternative using the unclipped anchor leaves them
  visible. Falsified if a qualifying pixel stays visible in the second capture. Inconclusive if
  no non-transparent shared pixel survives clipping; choose a different stock sprite or placement.
  This tests a reader, not the still-unconfirmed repaint persistence of a clipped writer or the
  clipping of a reused voxel image.

## Open questions

- **Generic infantry shadow above a high bridge:** S-0107 claim 19 leaves unresolved the
  neighbouring-cell condition under which the deck counts as ground for a unit above a high-bridge
  cell at or above the deck and not on it. Claims 11, 13 and 20 here do not specify the shadow
  point or depth for that whole case. Determine the condition, placement and final depth together.
  Do not infer a deck-relative formula from the ordinary-terrain formula. Backing: `unmeasured`,
  because a controlled scenario cannot predict the result until that condition is known.
  Confidence: low.

- **Building images:** the building shadow's bias (−4 − *L(z)*) has been tied to how building and unit
  images drawn as sprites get their shadow in general, not yet to the live building drawing with
  certainty; the slot animations are separate animation objects (claim 17), but whether a building
  draws any image in the object pass besides its voxel turret and its door sprite is not fully
  mapped; the construction state's build-up sprite is not yet placed. The special overlay image named in a building's art is read, but no drawing of it
  was found; medium confidence.
- **Building parts drawn as sprites in the object pass, and the door sprite:** the −2 is inferred
  from how other building and unit sprites are drawn. Their depth rules remain low confidence
  (claim 13). Door-sprite visibility is settled by S-0108 claim 23: it is drawn throughout
  unloading, including the closed state.
- **Terrain objects:** the image frames of the destruction sequence (claim 23) are not mapped,
  nor whether an object whose destruction has begun takes turns. The image frames of animated
  terrain objects, the strength of a terrain type and the 16-row raise of ore-spawning objects
  with its depth consequence are settled (claim 23).
- **Sprite placement and image frames, leftovers (claims 20 to 22, 25):**
  - which ion-cannon blasts set the ion-blast wobble, and to how many rows.
  - a per-unit state, set and cleared in the course of an aircraft carrying units, in which the own
    shadow of a sprite unit is drawn 14 rows higher than its draw point; no stock situation that
    sets it is known.
  - which shots start the firing counter of a vehicle drawn as a sprite. A shot of a type with
    *F*<sub>f</sub> > 0 can set it to 2*F*<sub>f</sub> − 1, after which it falls by 1 per tick to
    −1, so the firing image frames would show from the last to the first, each for two ticks; but
    some shots leave it alone, and which ones is not settled. Low confidence.
  - the standing-pose state of a vehicle drawn as a sprite (claim 22): which stock situation, if
    any, leaves a vehicle standing still part-way through a movement step, so that the row *d*
    shows, and the one event that turns the state on without being identified; and a case in which
    its walk counter also grows while it is in the air.
  - the turret path of a vehicle drawn as a sprite (low confidence; no stock type has it, D29 (c)
    gives the turret flag to one).
  - which cells count as near a bridge for the row of a vehicle drawn as a sprite whose type is
    too big to fit under a bridge (claim 13). It depends on the bridge state of the vehicle's own
    cell and of its neighbours, which has not been tied to the map's terrain and overlays.
  - the order among terrain tiles, which also decides which of two overlapping smudge paintings, or
    a smudge and a neighbouring tile, shows (claims 16, 25).
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
  been determined. Vehicle own-image overlap is settled by S-0105 claim 39 and claim 18 here:
  fresh exclusions leave own pixels undarkened, but retained cached exclusions can differ from
  the current image. This spec owns the drawing order and scene depth of the shadow; the point itself depends on the
  movement types (S-0106 and the movement specs).

- **Situational unit term *s*:** besides the ordinary −1, these cases are known but only partly
  determined, low confidence: −2 when the unit faces toward certain raised neighbouring tiles; −3 for
  a unit inside certain structures; −14 for a unit standing on a cell of certain special building
  types; +2 for a unit on a sloped cell next to a rock overlay. Their triggering conditions must be
  pinned down before scenarios can be written (slope, garrison and rock setups).
- **The veinhole monster:** it is in no render layer and no step of a drawn frame was found to paint
  it. Whether it is visible at all in the original, and through which other drawing, is not
  settled; stock data has a terrain type marked as a veinhole monster, and whether placing it
  creates a veinhole monster apart from an ordinary terrain object is not settled either. Medium
  confidence.
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
