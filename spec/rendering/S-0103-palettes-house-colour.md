---
id: S-0103
title: Palettes and house colour
topic: rendering
revision: 8
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0214, Q-0280]
supersedes: []
---

# S-0103: Palettes and house colour

## Summary

This spec covers skirmish rendering; network games and campaigns are out of scope. It describes how the palette files become palettes, how a colour definition becomes the 16-entry house-colour ramp, which colour definition each house shows, and which palette each kind of drawn object uses. It also says which palette indices keep their colour under coloured light, and gives the fixed colours of the skirmish setup screen's colour swatches. How a palette is shaded by light into rows of hi-colour pixels is in S-0104 (lighting). This spec only names the inputs to that shading.

## Terms

Glossary terms used here include *palette*, *theater*, *house*, *house colour*, *local player*, *rules layer*, *base rules*, *map layer*, *override section*, *lobby*, *measurement setup* and *quirk switch*.

- **Palette file.** Supplies 256 ordered colours, each with red, green and blue components represented by byte values intended to range from 0 to 63.
- **Unit palette.** The theater's palette for units, infantry, aircraft and buildings. House-colour palettes are built from it.
- **Tile palette.** The theater's palette for terrain tiles.
- **Theater palette.** The theater's general-purpose palette. The temperate theater palette read at program start supplies the start-up resource table (S-0112, Terms and claims 1 and 2); later theater loads do not change that table.
- **Anim palette.** The palette for animations that use neither the tile palette nor a house-colour palette. It is the same in every theater (claim 1).
- **Colour definition.** A named colour given as three bytes (H, S, V): hue, saturation, value. The hue runs 0–255 over the full circle, so 0 and 255 are both red.
- **Definition position.** Colour definitions are numbered from 0 in the order the game creates them in each game: the entries of the base rules' colour section in their order, then each new name that a later rules layer adds, in order. A later layer's entry whose name already exists creates nothing (claim 5). A line with an empty value is no entry at all (S-0001, claim 11), so it creates no definition and takes no position: the definitions after it each move one position lower. The **first colour definition** is the one at position 0.
- **Remap range.** Palette indices 16–31. A house-colour palette replaces exactly these entries.
- **House-colour palette.** The unit palette with the remap range replaced by the ramp of one colour definition.
- **Shade table, shade row, shade count.** The lit form of a palette (S-0104): *N* rows of 256 hi-colour pixels. Row *j* is the palette at the *j*-th brightness step; which row a brightness selects is S-0104, claim 15.
- **Single-shade and 53-shade palette.** The two house-colour palettes of one colour definition, whose shade tables have 1 and 53 rows (claim 5).
- **Tint-exempt index.** A palette index whose shade-table entries ignore the light's colour. They do not follow brightness either, except at the darkest rows: in a 53-row shade table such an entry rises from black over rows 0 to 6 and shows its full palette colour in every row from 6 on (S-0104, claim 6 and Formulas, "Shade table").
- **Hi-colour pixel.** A 16-bit packed RGB value in the display surface's pixel layout.
- **Lobby colour slot.** The colour a player picks in the skirmish lobby, numbered 0–7 in the order the lobby lists the eight colours.
- **Lobby swatch.** The coloured interior of a fixed-colour choice in the setup screen's colour control, both in its open list and when that choice is selected. Text, borders and the drop-down arrow are not part of this interior.
- **Vein animation.** An animation whose art marks it as a vein animation (Import mapping).
- **Terrain animation.** An animation that the game creates as part of a cell's terrain when the map's terrain is set up or when the cell is recalculated in play, rather than one created by an object or an event.
- **Object's own animation.** An animation that belongs to one object and shows part of that object: an animation a building plays as part of itself, the animation of a unit deploying or undeploying, and the parachute that carries an object down, unless that object is a projectile (claim 8).
- **Projectile.** What a weapon launches, from launch until it detonates. The combat specs own its rules. Here it matters only because a projectile can come down by parachute (claim 8).
- **Test lighting.** S-0104's neutral test lighting, with every value it changes set by the test map itself: the map's lighting values, and the three extra-light fields set to 0 in the test map's override section for the game-wide visual settings. Under it objects and tiles are drawn at brightness 1000 (row 26 of a 53-row shade table, S-0104).
- **Lighting terms of S-0104.** *Light level*, *brightness*, *light colour*, *scaled* and *unscaled ground level*, *map tint*, *light source* and its *light radius*, *map ambient*, *global effect* (the ion storm among them), *neutral brightness*, the *light-alpha value*, a *lighting set* with its *ground bias* and *level factor*, and the rules' *extra unit*, *extra infantry* and *extra aircraft light* (the three extra-light fields, S-0104, claim 13) are used here only as S-0104 defines them, units included.

## Behaviour

1. **Palette files are multiplied by 4 in 8-bit arithmetic.** Each palette loaded from a palette file (the unit, tile and theater palettes of a theater, and the anim palette) turns a file component *c* into the palette component (4·*c*) mod 256. Well-formed files (*c* ≤ 63) therefore give components 0, 4, …, 252: every component loaded from a file is a multiple of 4 and at most 252. A component above 63 wraps (64 → 0). The anim palette is read once, when the program starts, from its own file (Constants), and is not reloaded when the theater changes.
   - Backing: `measurable` (PA for the ×4, PB for the anim palette, PG for the wrap)
   - Confidence: high

2. **Palette files per theater, and missing files.** For a theater with palette base name *B* and suffix *X* (Constants), the palette files are named as follows:

   | Palette | File name | Temperate example |
   |---|---|---|
   | unit palette | `UNIT`*X*`.PAL` | `UNITTEM.PAL` |
   | tile palette | `ISO`*X*`.PAL` | `ISOTEM.PAL` |
   | theater palette | *B*`.PAL` | `TEMPERAT.PAL` |

   For each of the three files that is missing when a theater's palettes are loaded (claim 3), the palette in force afterwards is:

   | Missing file | Palette in force afterwards |
   |---|---|
   | theater palette | entry *i* = (*i*, 255 − *i*, (4·*i*) mod 256), for *i* = 0 … 255 |
   | unit palette | the unit palette in force before the load, with each component *c* replaced by (4·*c*) mod 256 |
   | tile palette | the tile palette in force before the load, unchanged |

   - Before the first theater load of a session, the unit palette in force is the snow theater's unit palette, loaded from its file (×4, claim 1) when the program starts. The tile palette in force is all black: every component 0.
   - The unit-palette rule applies on every load. If the unit palette file stays missing for *n* loads in a row, a component that was *c* in the file last loaded becomes (4^(*n*+1)·*c*) mod 256. From *n* = 3 on, every component is 0. Before the first load, the "file last loaded" is the snow unit palette file.
   - House-colour palettes are built from whichever unit palette is in force (claim 3).
   - Backing: `measurable` (PB) for the unit and tile palette names in all six theaters. `measurable` (PH) for the missing unit and tile palette files, the palettes in force before the first load and the repeated multiplication. `unmeasured` for the theater palette's per-theater file names and missing-file row during theater loading, because no drawing subject here is established to show that loaded palette directly. The resource table shows the temperate palette read at program start instead (S-0112, claims 1 and 2).
   - Confidence: high for the names, the table and the repeated multiplication. Medium for the palettes in force before the first load.

3. **When palettes are loaded and house-colour palettes built.** The unit, tile and theater palettes are reloaded only when a game starts in a different theater from the previous game of the session. The first game of a session always loads them. Every game, whatever its theater, builds all its house-colour palettes afresh from that game's colour definitions and the unit palette then in force. When the theater's unit palette file is found, that palette is the one in force; when it is missing, the missing-file rule in claim 2 determines which palette is in force. The anim palette never changes during a session (claim 1).
   - Backing: `measurable` (PB for the theater change and PI for fresh house-colour palettes in successive games; PH for the rule that a game in the same theater keeps the loaded palettes rather than reading the files again, which is visible only through claim 2's missing-file behaviour)
   - Confidence: high

4. **Parsing a colour definition.** The value is read as up to three decimal integers separated by commas. Each integer may carry a sign and may have spaces before it. A comma must follow an integer directly.
   - Reading stops at the first character that does not fit this form. Every integer not read by then is 0. So `10,abc,20` gives (10, 0, 0), and `10 ,20,30` gives (10, 0, 0).
   - Each integer read is stored as its value mod 256, taken as a non-negative remainder: 300 becomes 44, −1 becomes 255 and −86 becomes 170.
   - Characters after the 63rd character of the value have no effect.
   - Backing: `measurable` (PI). With base rules that supply all selected positions and the light-grey name (claim 10; PF confirms the name first), a house shows only definitions the base rules create, so a test map cannot supply a malformed one; PI uses a loose copy of the base rules instead.
   - Confidence: high. Medium for integers outside −2 147 483 648 … 2 147 483 647, which are expected to reduce the same way, mod 256; this overflow extension is `unmeasured`, because PI does not test overflowing integers and their parsing is not isolated here.

5. **Two palettes per colour definition, and the first definition of a name wins.** Each colour definition has a single-shade and a 53-shade house-colour palette with the same ramp (S-0104 shades them). A house's units, infantry, aircraft and buildings are drawn with its 53-shade palette, except where claim 8 says otherwise (claims 8 and 10). Among the battlefield subjects covered by claim 8, the single-shade palette is shown only by animations flagged for the alternative palette. Names are compared ignoring case. When any later rules layer (the language rules file, the game-mode file or the map) defines a colour name that already exists, the existing definition is kept with its original value. Every house that shows that name keeps the earlier ramp, and nothing at a later position is created. This is a specific exception to the general layer-override rule, also identified in S-0006 claim 2.
   - Backing: `measurable` (PE for the two variants, PF for the shown-name rule). Definition positions are tested by PC; the statement that higher-layer names append and duplicates create no hidden extra definition is `unmeasured`: stock skirmish lobby choices expose the existing base-rules positions, not the count or positions of newly appended definitions.
   - Confidence: high

6. **The house-colour ramp.** For a definition (H, S, V), entry 16 + *i* for *i* = 0 … 15 is the hue H with saturation *S_i* and value *V_i* (Formulas), converted to RGB by the integer HSV conversion in Formulas. Entry 16 is the brightest, and the value falls with *i*.
   - **Entry 31 is always pure black (0, 0, 0)**, whatever the definition, because its value multiplier is effectively zero.
   - The other 240 entries are those of the unit palette.
   - The ramp has full 8-bit precision. Unlike file-loaded components (claim 1), its components need not be multiples of 4.
   - Backing: `measurable` (PC)
   - Confidence: high

7. **Tint-exempt indices.** In the shade tables of house-colour palettes, indices 240–254 are tint-exempt and indices 1–239 and 255 follow the light's colour. Index 0 is transparent in every shade row (S-0104). In the shade tables of the tile palette no index is tint-exempt.
   - **Brightness.** A tint-exempt entry ignores brightness too, except in the darkest rows. In a 53-row table it is at its full palette colour in row 6 and every later row (S-0104, claim 6). At the default light-alpha value, row 6 is reached at a brightness of 236 ‰ (S-0104, claim 15). So at any brightness from 236 ‰ up, a tint-exempt pixel looks the same however bright or dark the rest of the object is drawn.
   - **When the exemption shows.** A 53-shade house-colour palette starts with white light. Global-effect tint changes follow S-0104 claims 2, 3 and 5: they occur at the specified lighting re-evaluations, not simply whenever an effect is in force. In particular a chrono flash alone does not change an existing palette's tint, and a tint applied by a re-evaluation during that flash can remain after it ends.
   - A trigger action that sets one map-tint channel applies the whole new map tint immediately and makes it the palette's own tint (S-0104, claim 17), even during a global effect. A later lighting re-evaluation may replace the displayed tint with the then-selected effect tint, or restore the palette's own tint when no effect wins. The exemption is visible whenever one of these events has left a non-white displayed tint.
   - Light-source colours never reach a house-colour palette, and neither does the map tint before such a trigger action. Single-shade house-colour palettes are never tinted (S-0104, claim 3).
   - Backing: `measurable` (PD for exemption under storm and trigger tint; S-0104 L2 and L21 for re-evaluation timing, overlaps and trigger actions; PD's low-light part for rows 0–6)
   - Confidence: high

8. **Which palette an object uses.**
   - **Terrain tiles** use the tile palette, shaded by their own cell's light colour (S-0104).
   - **Units, infantry, aircraft and buildings** use their owner's 53-shade house-colour palette, except as below.
   - **A building flagged in its art to use the terrain palette** uses the tile palette instead. Its body and sprite parts are shaded with the light colour and scaled ground level of the cell under the building's centre and show no house colour (S-0104, claim 13, including the larger-coordinate cell at an even foundation's centre). S-0104 also owns the brightness of its voxel parts.
   - **Animations** take their palette from the first row of this table that applies to them. An animation that meets several conditions uses the earliest row it meets.

   | Precedence | Animation | Palette | Light and brightness |
   |---|---|---|---|
   | 1 | vein animation | the 53-shade house-colour palette of the **local player**, the same palette the local player's own units use, whoever owns the animation | its cell's scaled ground level (S-0104) |
   | 2 | terrain animation | the tile palette | its cell's light colour, at its cell's scaled ground level (S-0104) |
   | 3 | an object's own animation that is drawn with its object (see below the table) | the owner's 53-shade house-colour palette, even when the object is a building that uses the terrain palette | a brightness of its own, which depends on the kind of animation (see below the table) |
   | 4 | flagged for the alternative palette | the **single-shade** house-colour palette of the **first colour definition**, whatever the owner's colour | one row, so it ignores brightness and never takes an effect tint (S-0104, claim 3) |
   | 5 | any other | the anim palette | its cell's unscaled ground level (S-0104); the anim palette's shade table is never tinted (S-0104, claims 3 and 6) |

   - **Which own animations are drawn with their object (row 3).**
     - A unit's deploy or undeploy animation always is.
     - The parachute of any object other than a projectile always is.
     - A building's own animation is when the animation's art flag for drawing with its object is set, and the flag is set when absent (Import mapping). With the flag off, the animation takes its palette from row 4 or 5, by its own art flags.
     - A projectile coming down by parachute is the exception. It gets a different parachute animation (Import mapping), which is not drawn with it: that animation takes its palette and brightness from row 4 or 5, by its own art flags, like an animation that belongs to no object.
   - **Brightness of row 3.**
     - A building's own animation is drawn at the building's brightness: the unscaled ground level of its top-left foundation cell (S-0104, claim 13), even for a terrain-palette building. Its brightness is set when the animation starts and set again on every tick, so it follows changes to the building's light. S-0104 owns the alternative cell source under its building-light quirk switch. How flashing, or an effect that changes the building's brightness, changes it is not established (Open questions).
     - A parachute is drawn at the unscaled ground level of the carried object's cell on the tick the parachute is created (S-0104, claim 13). It is not updated during the fall (Edge cases).
     - A deploy or undeploy animation is drawn at brightness 1000 (neutral), whatever the cell's light (Edge cases).
   - An animation flagged in its art to use normal light is drawn at brightness 1000 instead of the brightness in the last column. This changes nothing for row 4.
   - **Ore and gem overlays outside deck and marker cells** are drawn at neutral brightness and ignore the cell's light level (S-0104, claim 13). They use the start-up resource table: a 53-row untinted table built at program start from the temperate theater palette and unchanged when theaters are loaded (S-0112, claims 1 and 2). A drawn overlay in a deck cell instead uses the cell's tile shade table and scaled raised light level, whatever its type. S-0112 claim 1 owns this precedence and the marker-cell exclusion. Shroud cover pieces and light-glow images can still change ore and gem shade rows through the per-pixel light-alpha value (S-0112, claim 1; S-0110, claims 5 and 6).
   - **Other overlays, smudges and terrain objects.** S-0112 claims 1 to 3 own their palettes, tint and brightness.
   - Backing: `measurable` (S-0112 scenarios OV1, OV2 and OV18 for ore and gem palette, tint, theater independence and per-pixel shade-row changes, and OV7 part C for deck-cell precedence; S-0104 L23 parts B and E for the building animation's top-left source, tick updates and the terrain-palette building's centre-cell source; PE for the object kinds and rows 1 to 5, the parachute, the normal-light flag and the brightness of row 3, a light change during a parachute's descent and during a building's own animation included; PJ for a building's own animation with the drawing-with-object flag off and for the precedence of rows 1, 2 and 3 over row 4). `unmeasured` for the projectile's parachute, because the measurement setup offers no known way to make a projectile come down by parachute (Open questions). `unmeasured` for the precedence between rows 1, 2 and 3, because no way is known to create an animation that meets two of those conditions; it matters only if one exists. The terrain-palette building's own-animation source is `unmeasured` until a suitable stock subject for the L23 repeat is identified. The distinction between the parachute's creation tick and a later visible opening tick is `unmeasured`: PE tests brightness kept during the fall but establishes no separate, observable opening event.
   - Confidence: high for tiles, terrain-palette buildings, units, infantry, aircraft and buildings, and rows 1, 4 and 5. High for the palette of deploy, undeploy and parachute animations and for the projectile exception. Medium for rows 2 and 3 otherwise, for the brightness of row 3 and for the precedence between rows. High for the ore and gem overlay palette and tint outside deck and marker cells, as specified by S-0112 claims 1 and 2; medium for deck-cell precedence, as specified by S-0112 claim 1.

9. **Output pixel format.** Shading turns every palette into 16-bit hi-colour pixels in the display surface's pixel layout. Each 8-bit component keeps only its top bits: 5 bits red, 6 green and 5 blue in the 5-6-5 layout, and 5 bits each in the 5-5-5 layout. A 24- or 32-bit display therefore shows only colours that survive this 16-bit step.
   - Backing: `measurable` (PA) for 5-6-5 output. The 5-5-5 variant is `unmeasured`: PA fixes a 5-6-5 display path, and no supported path exposing 5-5-5 is established for the measurement setup here.
   - Confidence: high that the shade tables are 16-bit. Medium for which layout a given display path presents.

10. **Which colour definition a house shows.** In a skirmish:
    - A player house shows the definition at a fixed position chosen by its lobby colour slot *s*: position *P*(*s*) in the table below.
    - Both non-player houses (the neutral house and the special house) show the definition with a fixed name, the light-grey name in Import mapping.
    - When the base rules supply all positions selected by the lobby table, a later rules layer cannot change which definition a player house shows. Whenever the base rules define the light-grey name, a later layer cannot change the non-player houses' colour either (claim 5).

    | Lobby colour slot *s* | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
    |---|---|---|---|---|---|---|---|---|
    | Definition position *P*(*s*) | 1 | 5 | 10 | 14 | 6 | 12 | 8 | 7 |

    - Backing: `measurable` (PC)
    - Confidence: high for the table and the non-player houses. Medium that the slot numbering matches the lobby's listing order.

11. **Setup-screen swatches are fixed RGB colours, not shades of a definition.** Each fixed lobby colour slot has the RGB colour in Constants. These are engine constants, independent of every colour definition and of the house-colour ramp. Changing a definition can change the battlefield house colour without changing its setup-screen swatch. There is no HSV conversion, ramp entry selection, palette-file multiplication or lighting adjustment for a swatch. Each component is reduced directly to the display's hi-colour precision by the 16-bit-step formula, with no subtraction of 1.
    - Backing: `measurable` (PK, including the named base-rules test for independence)
    - Confidence: high

12. **A swatch is a flat rectangular fill.** The coloured interior uses one solid colour, not an image tinted or recoloured by the slot colour. The open list and the selected choice use the same slot colour. Text and surrounding control decoration may cover parts of the fill and are excluded from the flat-colour rule. This claim covers enabled fixed-colour choices; the random choice is not a ninth player colour slot.
    - Backing: `measurable` (PK)
    - Confidence: high

## Formulas

All arithmetic is on integers unless stated otherwise. "÷" is integer division truncating toward zero; every dividend below is non-negative, so it is also the floor. "mod" gives a non-negative remainder.

**Palette file load** (claim 1): component = (4 · *c*) mod 256, for a file byte *c* from 0 to 255. Exact integer arithmetic.

**Missing unit palette file, repeated** (claim 2): component = (4^(*n*+1) · *c*) mod 256 after *n* loads in a row without the file, where *c* is the component in the file last loaded. Exact integer arithmetic.

**Colour definition parse** (claim 4): each stored byte = *v* mod 256, for the integer *v* read, with a result from 0 to 255.

**Ramp** (claim 6). For *i* = 0 … 15:

> *S_i* = ⌊σ_i · S⌋, *V_i* = ⌊γ_i · V⌋

- σ_i and γ_i are the single-precision constants in the Constants table: the original's single-precision values of the sine and cosine of the angles below, each angle first rounded to the nearest 1/8192 of a turn (halves rounded up). The original computes the products in floating point. Each product is exact in double precision, so the floating-point rounding mode does not change it, and ⌊ ⌋ truncates it to an integer from 0 to 255.

> β_i = 50° + *i* · (40/15)° (for σ_i = sin β_i)
>
> α_0 = 11.25°, α_i = 20° + *i* · (70/15)° for *i* ≥ 1 (for γ_i = cos α_i)

An implementation should use the tabulated values. Recomputing them in floating point may differ in the last bit and change ⌊·⌋.

**Integer HSV → RGB** (claim 6), for bytes (H, S, V), all values non-negative integers:

> *f* = (6H) mod 255, *k* = (6H) ÷ 255 (0 … 6)
>
> *p* = ((255 − S) · V) ÷ 255
>
> *q* = ((255 − (*f* · S) ÷ 255) · V) ÷ 255
>
> *t* = ((255 − ((255 − *f*) · S) ÷ 255) · V) ÷ 255

| *k* | (R, G, B) |
|---|---|
| 0 or 6 | (V, *t*, *p*) |
| 1 | (*q*, V, *p*) |
| 2 | (*p*, V, *t*) |
| 3 | (*p*, *q*, V) |
| 4 | (*t*, *p*, V) |
| 5 | (V, *p*, *q*) |

The hue scale is 255 per full circle, divided into six sectors of 42.5. H = 255 gives *k* = 6, which is treated as sector 0 (red), so H = 255 and H = 0 give the same ramp.

**Worked examples** (8-bit entries 16–31, before shading):

- (H, S, V) = (0, 255, 255): (250,58,58), (231,48,48), (222,40,40), (211,32,32), (199,25,25), (185,20,20), (170,15,15), (154,10,10), (137,7,7), (119,4,4), (101,2,2), (81,1,1), (61,0,0), (41,0,0), (20,0,0), (0,0,0).
- (H, S, V) = (170, 128, 200): (120,120,196), (109,109,181), (102,102,174), (95,95,165), (88,88,156), (80,80,145), (72,72,133), (64,64,121), (56,56,107), (48,48,93), (40,40,79), (32,32,64), (24,24,48), (16,16,32), (8,8,16), (0,0,0).

**16-bit step** (claim 9): in the 5-6-5 layout, red₅ = ⌊R / 8⌋, green₆ = ⌊G / 4⌋, blue₅ = ⌊B / 8⌋ for the 8-bit shaded components R, G, B. In the 5-5-5 layout green₅ = ⌊G / 8⌋ instead. Exact integer arithmetic.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| remap range | 16 … 31 | palette index | 16 entries |
| tint-exempt range (house-colour palettes) | 240 … 254 | palette index | none in tile palettes |
| shade counts per colour definition | 1 and 53 | rows | S-0104 |
| palette file size | 768 | bytes | 256 × 3, components meant 0–63 |
| anim palette file | `ANIM.PAL` | file name | read once per session |
| lobby slot → definition position | 1, 5, 10, 14, 6, 12, 8, 7 | definition position | for slots 0 … 7, claim 10 |
| σ_0 … σ_15 | 0.7661539316177368, 0.7949070930480957, 0.8224136829376221, 0.8481203317642212, 0.8719707727432251, 0.8935688734054565, 0.9135870337486267, 0.9316057562828064, 0.9473402500152588, 0.9612804651260376, 0.9731168746948242, 0.9826816916465759, 0.9902727603912354, 0.9956966042518616, 0.9989056587219238, 1.0 | dimensionless | single-precision values, written exactly |
| γ_0 … γ_15 | 0.9807852506637573, 0.9088482856750488, 0.8719707727432251, 0.8289040923118591, 0.7807372212409973, 0.7274125814437866, 0.6692825555801392, 0.606731116771698, 0.5395258069038391, 0.469366192817688, 0.3961058259010315, 0.3202287256717682, 0.24223609268665314, 0.16188637912273407, 0.08121144026517868, 1.2246063538223773e-16 | dimensionless | γ_15 is not exactly 0, but ⌊γ_15 · V⌋ = 0 for every byte V |

**Lobby swatch colours (claim 11).** Each tuple is (red, green, blue), in integer component units from 0 to 255 before the 16-bit step. These values are fixed by the engine, not imported rules values.

| Lobby colour slot | RGB |
|---|---|
| 0 | (221, 226, 13) |
| 1 | (255, 25, 25) |
| 2 | (42, 116, 226) |
| 3 | (62, 209, 46) |
| 4 | (255, 160, 25) |
| 5 | (50, 215, 230) |
| 6 | (149, 40, 189) |
| 7 | (255, 154, 235) |

Theaters. These names are engine constants of the original, not stock data. Each theater has an identifier, which the map's theater value names, a palette base name *B* and a suffix *X*. All four columns are names, not quantities, so they carry no unit.

| Theater | Identifier | *B* | *X* |
|---|---|---|---|
| Temperate | TEMPERATE | TEMPERAT | TEM |
| Snow | SNOW | SNOW | SNO |
| Urban | URBAN | URBAN | URB |
| Desert | DESERT | DESERT | DES |
| New Urban | NEWURBAN | URBANN | UBN |
| Lunar | LUNAR | LUNAR | LUN |

## State and ordering

| State | Event | Result |
|---|---|---|
| program started | — | the unit palette in force is the snow theater's unit palette, loaded from its file (×4); the tile palette in force is all black; the anim palette is loaded from its file (×4) |
| palettes for theater A in force | a game in theater A starts | no palette file is read; house-colour palettes are built from that game's colour definitions and A's unit palette |
| palettes for theater A in force (or none loaded yet, as after program start, even when B is snow) | a game in theater B starts | the palettes for theater B are loaded under claim 2, including its missing-file rules; house-colour palettes are built from that game's colour definitions and the unit palette then in force |
| rules layers read | each new colour name | a single-shade and a 53-shade house-colour palette exist under that name, at the next definition position |
| rules layers read | a colour name that already exists | nothing changes |

## Edge cases and quirks

- **Component wrap.** A palette-file component above 63 wraps after the ×4 (claim 1). `quirks.palette_component_wrap` (default on). Off: such a component becomes 252.
- **Missing unit palette multiplies again.** With its file missing, the unit palette in force is multiplied by 4 again on each theater change, and turns black after a few (claim 2). `quirks.missing_unit_palette_remultiplied` (default on). Off: a missing unit palette file leaves the unit palette in force unchanged, as a missing tile palette file does.
- **Missing palette files at the first load.** On the first load of a session, a missing unit palette file leaves the unit palette as the snow unit palette file multiplied by 4 twice, (16·*c*) mod 256, and a missing tile palette file leaves the tile palette all black (claim 2). This follows from the two rules above, so it gets no switch of its own. With `quirks.missing_unit_palette_remultiplied` off, a missing unit palette file leaves the snow unit palette file multiplied by 4 once.
- **Black last ramp entry.** Entry 31 is always black (claim 6). This is how the original behaves, not a bug, so it gets no switch.
- **The first definition of a name wins.** A later rules layer cannot change an existing colour definition (claim 5). `quirks.colour_redefinition_ignored` (default on). Off: a later layer's value replaces the earlier one for that name and keeps its position, and every house showing the name shows the new ramp.
- **Alternative-palette animations use the first colour definition, not the owner's** (claim 8, row 4). `quirks.alt_palette_first_definition` (default on). Off: the animation uses its owner's single-shade house-colour palette, and the first colour definition's when it has no owner.
- **Parachutes keep the light of their creation tick** (claim 8, row 3). If the light changes while an object comes down, its parachute keeps its creation-tick brightness while the object follows the new light. `quirks.parachute_brightness_fixed` (default on). Off: the parachute follows the unscaled ground level of the carried object's current cell, as a building's own animation follows its building.
- **Deploy and undeploy animations ignore the light** (claim 8, row 3). They are drawn at brightness 1000, so at night a deploying unit's animation is brighter than the unit. S-0104's `quirks.deploy_animation_unlit` (default on) owns this rule. Off: they take the unscaled ground level of the unit's cell (S-0104, claim 13 and Edge cases), without extra unit light or a height term.
- **Vein animations use the local player's colour** (claim 8, row 1). Different players can see them in different colours. `quirks.veins_local_player_colour` (default on). Off: vein animations use the anim palette, like any other animation (row 5), so every player sees the same colours.

## Import mapping

The lobby swatch's RGB and flat fill (claims 11 and 12) have no data-file selector or absent-key default: they are fixed interface behaviour. In particular, the colour-definition values below feed battlefield palettes, not swatches.

| Source | Key | Parse | Feeds | Absent |
|---|---|---|---|---|
| rules `[Colors]` | each key | name = key, compared ignoring case; value as in claim 4 | one colour definition at the next definition position, if the name is new (claims 4–6 and 10); never the setup swatch RGB or fill (claims 11 and 12) | missing integers = 0; a line with an empty value creates no definition (S-0001, claim 11) |
| rules `[Colors]` | `LightGrey` | name, ignoring case | the definition both non-player houses of a skirmish show (claim 10) | engine constant name |
| art, building entry | `TerrainPalette` | boolean | building uses the tile palette (claim 8) | no |
| art, animation entry | `AltPalette` | boolean | alternative palette (claim 8, row 4) | no |
| art, animation entry | `IsVeins` | boolean | vein animation (claim 8, row 1) | no |
| art, animation entry | `ShouldUseCellDrawer` | boolean | a building's own animation drawn with its building (claim 8, row 3); deploy, undeploy and parachute animations ignore it | yes |
| art, animation entry | `UseNormalLight` | boolean | drawn at brightness 1000 (claim 8) | no |
| rules `[General]` (a game-wide section) | `Parachute` | animation name | the parachute of an object other than a projectile (claim 8, row 3) | not established |
| rules `[General]` | `BombParachute` | animation name | the parachute of a projectile; not drawn with it (claim 8) | not established |
| map `[Map]` | `Theater` | theater identifier (Constants) | which palette files are used (claim 2) | (see S-0009) |
| overlay, smudge and terrain-object data | keys and value forms in S-0112 Import mapping | as specified there | palette, tint and brightness selection delegated by claim 8 to S-0112 claims 1–3, including deck-cell precedence | defaults in S-0112 Import mapping |

## Measurement scenarios

S-0104 scenario L23 part B tests the building animation's top-left foundation cell and per-tick light following; part E tests the terrain-palette building's centre-cell source. Repeat part B's two light placements with a terrain-palette building that plays an own animation drawn with it: the animation must still match the top-left reference, even when the body's light follows the centre cell. Record equivalent animation image frames; a match to the centre-cell reference instead falsifies the own-animation source. If no suitable stock subject is found, this last comparison remains unmeasured. Their setups, pixel observables, expected reference matches and falsifiers apply to claim 8 here.

Scenario labels PA to PJ are local to this spec; they are not measurement-record IDs. The ore and gem rules of claim 8 use S-0112's scenarios OV1 (palette and theater independence), OV2 (brightness and tint), OV7 part C (deck-cell precedence) and OV18 (shroud and glow shade-row changes), with the setup, procedure, observables, expected results and falsifiers given there.

All scenarios run under the *measurement setup* (glossary): an unmodified install, stock data and no engine add-ons, with the allowances the glossary lists. Every scenario that reads pixels may use a presentation-only display compatibility layer (Display path below). PF and scenarios PG to PJ add named test files, and each says which: test language and game-mode rules copies (PF), a loose palette file that overrides a shipped one on purpose (PG), loose copies of stock archives with the named palette files left out to make them absent (PH), a loose copy of the base rules (PI), or a copy of the art database in a loose test archive that wins over the stock archives (PJ; S-0005 and S-0007 say how such an archive ranks). The test files are used only for the scenario that names them and are removed after the run.

**Common setup and recording.** Use game speed index 2, detail level 2 and 800×600, with the director and unrelated mods off. Each skirmish has a local human-controlled player and one bystander computer player in a separated start area; where a scenario names computer-owned subjects, those are exceptions to the separation. The observed area contains only the named test subjects, and the glossary's bystander void rule applies. Human-owned map objects removed by skirmish loading must instead be created in play, or placed under a retained owner for tests that do not depend on ownership. Use scripted input and lossless, tick-labelled screen capture; any missing harness support is a proposal for the user, not assumed installed. Unless cover or glow is explicitly tested by the cited scenario, sample explored pixels far from shroud edges and glow images, with light-alpha 127. Pause only after a required event has completed; timing tests need a capture for each tick.

Every scenario that reads pixels shares these rules:
- **Display path.** Record how the game's 16-bit output reaches the screen (the display path) and the pixel layout it presents. If a display compatibility layer is used, the measurement record names it generically, by what it does (for example "presents the game's 16-bit surface as 32-bit pixels, unscaled, without waiting for the monitor's refresh"), not by product. Such a layer must not scale, filter or colour-convert beyond widening each 16-bit pixel. Take screenshots at the game's own resolution, unscaled and unfiltered.
- **Reading a pixel.** Sample only identified, non-transparent pixels, not palette index 0 or shadow pixels. For each screenshot pixel, use only the top bits of each 8-bit channel: red₅ = ⌊R/8⌋, green₆ = ⌊G/4⌋ and blue₅ = ⌊B/8⌋ in the 5-6-5 layout, and red₅ = ⌊R/8⌋, green₅ = ⌊G/8⌋ and blue₅ = ⌊B/8⌋ in the 5-5-5 layout. PA needs a 5-6-5 display path. Where another scenario writes expected top bits in the 5-6-5 layout (the worked examples of PC, and PG), a 5-5-5 display path shows green₅ = ⌊green₆ / 2⌋ instead, with red and blue unchanged. These are the hi-colour pixel's own components, whether the display path fills the low bits with zeros or with copies of the top bits.
- **Expected values.** Each one comes from the formulas here and in S-0104, applied to stock files that the measurer reads at measurement time: palette files and colour definitions. The spec gives no stock values.
- **Lit components.** Under test lighting, a tinted entry with 8-bit component *u* is shown as *u* − 1 for *u* ≥ 1 and as 0 for *u* = 0 (S-0104, claim 7). A tint-exempt entry, and every entry of the anim palette, is shown as *u* (S-0104, claims 6 and 7).
- **Runs.** The palette and brightness rules tested here are deterministic. Other play, such as an explosion's animation choice, may use random draws; compare identified image frames and palette indices, not unqualified screenshot equality. One run per configuration is enough unless the scenario says otherwise.

**PA: palette ×4 and the 16-bit output (claims 1 and 9).**
- *Setup:* a purpose-made flat temperate map with test lighting, and a display path that presents a 5-6-5 surface.
- *Procedure:* start a skirmish, pause, and screenshot a stretch of flat clear terrain.
- *Observable:* for pixels whose tile-palette index *n* is known from the tile's image, the top bits of each channel.
  - Let (*r*, *g*, *b*) be entry *n* of the temperate tile palette file. A component of 0 is shown as 0 in every channel below.
  - Otherwise the expected top bits are red₅ = ⌊(4*r* − 1) / 8⌋, green₆ = *g* − 1 and blue₅ = ⌊(4*b* − 1) / 8⌋.
  - Also, across the whole screenshot: the full 8-bit value of each channel is the same for every pixel with the same top bits.
- *Runs:* 1.
- *Expected:* every examined pixel matches.
- *Falsified if:*
  - Any pixel's top bits differ from the expected values. Scaling the file components by 255/63 instead of 4 would show green₆ ≥ *g* for *g* ≥ 21.
  - Two pixels share their top bits but differ in their low bits. Then the output is not 16-bit, or the display path is not pixel-exact.
  - If S-0104's claim 7 (the −1) is falsified by its own scenario, recompute the expectation with 4*c* in place of 4*c* − 1 before judging this one.

**PB: theater palette files, theater change and the anim palette (claims 1, 2 and 3).**
- *Setup:* six purpose-made flat maps with test lighting, one for each theater in Constants. Each has the same human-owned infantry unit on open ground, and a pre-placed unit of the computer player for the human to destroy.
- *Procedure:* in one session, play temperate, snow, urban, desert, new urban, lunar, then temperate again. In each, screenshot the infantry unit, flat clear terrain, and the explosion when the computer's unit is destroyed.
- *Observable:* top bits of the infantry's pixels outside the remap range, of tile pixels, and of explosion pixels whose palette indices are known from their image frames.
- *Runs:* 1 sequence of seven games.
- *Expected:*
  - Infantry pixels match the PA formula applied to the unit palette file of that game's theater, named by claim 2 and Constants. At the tint-exempt indices 240–254 they show the top bits of 4*c* without the −1 (Lit components).
  - Tile pixels match it applied to that theater's tile palette file, named by claim 2 and Constants.
  - Explosion pixels show the top bits of 4*c* exactly (no −1), with *c* from the anim palette file, for the same image frame and palette index in all seven games.
  - The last game's identified pixels match the first.
- *Falsified if:* any pixel follows another file, an explosion pixel differs between theaters, or the last game's identified pixels differ from the first.

**PC: house-colour ramp and lobby colours (claims 6 and 10).**
- *Setup:* a purpose-made flat temperate map with test lighting. The human player and a pre-placed human-owned infantry unit stand on open flat ground. One pre-placed building is owned by the neutral house and another by the special house. Ensure the special house is retained by skirmish loading (S-0009); if it is absent, that part cannot yield a measurement. (Infantry are sprites. A voxel vehicle's face shading changes which palette index is drawn, S-0105.)
- *Procedure:* one game per lobby colour slot *s* = 0 … 7, each with the human player in slot *s*. In each, screenshot the infantry and both non-player buildings.
- *Observable:* top bits of the infantry's pixels whose palette index *n* is in the remap range (known from the sprite's image frames), and the same for both non-player buildings.
- *Runs:* 8 games (one per slot).
- *Expected:*
  - For slot *s*, read (H, S, V) of the colour definition at position *P*(*s*) of the stock base rules' colour section. Count only the lines that create a definition: a line with an empty value takes no position (Terms, *definition position*). Compute entry *n* by the ramp formulas, take the lit component (−1 rule), then the top bits.
  - Both non-player buildings' remap pixels follow the same computation for the light-grey definition (Import mapping).
  - Entry 31 shows (0, 0, 0).
  - To check the computation itself, the worked example (0, 255, 255) lit and packed to 5-6-5 is (31,14,7), (28,11,5), (27,9,4), (26,7,3), (24,6,3), (23,4,2), (21,3,1), (19,2,1), (17,1,0), (14,0,0), (12,0,0), (10,0,0), (7,0,0), (5,0,0), (2,0,0), (0,0,0).
  - For (170, 128, 200) it is (14,29,24), (13,27,22), (12,25,21), (11,23,20), (10,21,19), (9,19,18), (8,17,16), (7,15,15), (6,13,13), (5,11,11), (4,9,9), (3,7,7), (2,5,5), (1,3,3), (0,1,1), (0,0,0).
- *Falsified if:* any remap pixel differs from its computed value, or a slot shows a definition other than the one at *P*(*s*). A slot whose colour matches a different position, while every pixel matches that position's ramp, falsifies only the slot table and the slot numbering.

**PD: tint-exempt indices (claim 7).**
- *Setup:*
  - Test lighting. A human-owned building whose art uses indices 240–254 (lights), on flat open ground.
  - The test map's own lighting section sets the ion storm's lighting set to test values: ambient 0.60, tint (0.50, 1.00, 1.00), ground bias and level factor 0. Under the storm the building is then drawn about 600 ‰ bright (row 15 of 53) instead of 1000 (row 26). Both brightnesses are above the 236 ‰ of claim 7.
  - The test map starts an ion storm on a fixed tick through its own trigger, and the storm lasts long enough for the current ambient to reach the storm's ambient (S-0104, claim 4).
- *Procedure:*
  - Run 1: screenshot the building before the storm, and again during the storm once its pixels outside 240–254 have stopped changing.
  - Run 2: play the same map with its map tint set strongly red (for example red 1.5, green 0.5, blue 0.5), no storm and no trigger action that changes the tint, and screenshot the building.
  - Run 3: play the map with test lighting and no storm, plus a trigger action that sets the map's red tint to 0.50 on a fixed tick. Screenshot the building before and after the action.
- *Observable:* top bits of the building's pixels at indices 240–254 and at other indices.
- *Runs:* 3 tint runs, plus the low-light part below.
- *Expected:*
  - Run 1: every pixel at indices 240–254 is identical before and during the storm, at its full palette colour (Lit components). The other indices become darker and shift towards the storm tint: their red falls further than their green and blue.
  - Run 2: the building takes no red cast. Any change from its test-lighting colours is the same proportion in all three channels, within one step of the top bits.
  - Run 3: from the tick of the action, the pixels at indices 240–254 are identical to those before it. At the other indices red is reduced as S-0104's Scenario L21 gives (for example 248 → 123), and green and blue stay as before.
- *Falsified if:*
  - In run 1, any pixel at indices 240–254 differs between before and during the storm. Darker pixels there mean they follow brightness; a shift towards the storm tint means they are tinted.
  - In run 2, the building's pixels turn redder relative to their green and blue.
  - In run 3, any pixel at indices 240–254 changes at the action, or the other indices keep their colour (the tint set by the trigger action does not reach the house-colour palette).
  - Run 1 says nothing about claim 7 if the other indices do not change during the storm, because then the storm was not in force or its set was not applied. Check the map instead.
- *Low-light part:* repeat the building test with no global effect or tint-changing trigger, under neutral test lighting except map ambients 0.00, 0.10, 0.18, 0.23, 0.24 and 0.30. These select rows 0, 2, 4, 5, 6 and 7 respectively at light-alpha 127. Record the same identified tint-exempt pixels in six games. Expected: the rows below 6 follow S-0104's tint-exempt shade formula, row 0 is black, and rows 6 and 7 both show the full palette colour. Any different computed pixel value, or continued brightening between rows 6 and 7, falsifies the respective rule. Choose non-black palette entries so the comparison is discriminating.

**PE: palette by object kind (claims 5 and 8).** One map, purpose-made.
- *Setup:*
  - Two houses: the human player (lobby slot *a*) and an allied computer player (slot *b* ≠ *a*).
  - The computer player owns a vehicle, an infantry unit, an aircraft, a building with an animation of its own whose art leaves the drawing-with-object flag at its default, and a building flagged to use the terrain palette.
  - The human player owns a stock vehicle that plays an animation when it deploys and when it undeploys. Prefer one whose deploy animation's image frames use indices in the remap range.
  - A trigger action of the test map drops a team of stock infantry by parachute for the computer player, onto open flat ground, on a fixed tick.
  - A vein field with its vein animations; whoever owns it, it is not the human player.
  - A building with a strongly coloured light (S-0104). Both a terrain tile that the theater animates and the terrain-palette building stand inside its light radius.
  - A stock animation flagged for the alternative palette, played on the map by its trigger or by the object that normally creates it.
  - Two stock animations that take row 5 (no vein, alternative-palette or terrain role, and not an object's own animation), played on open flat ground by trigger actions: one whose art sets the use-normal-light flag, and one without it.
  - Two lighting configurations: test lighting, and test lighting with map ambient 0.43.
- *Procedure:*
  - Screenshot every item under both lighting configurations. Deploy the human player's vehicle and screenshot its deploy animation, then undeploy it and screenshot its undeploy animation. Screenshot the parachutes while the infantry come down.
  - Play again with the human player in slot *b* and the computer player in slot *a*, and screenshot the vein field.
  - Light-change run: test lighting, with the ambient change step and rate set in the test map's game-wide visual section so that a full fade (S-0104, claim 4) ends well before the parachutes land, and the paratroopers dropped from as high as the trigger action allows. A trigger action lowers the base ambient to 0.43 (S-0104, claim 14) on a tick after the parachute animations have been created, while the building's own animation is playing. Once the fade has ended and before the parachutes land, screenshot the parachutes, the falling infantry, the building and its own animation.
- *Observable:* top bits of each item's pixels, compared with the ramps computed as in PC, and with the unit palette and anim palette files (Lit components). For the palette of an animation without remap pixels, use indices at which the unit palette file and the anim palette file differ. For voxel subjects, determine the output palette index with S-0105's face-shading formulas for the recorded image and facing before comparing palette values; compare equivalent faces and facings across runs.
- *Runs:* 2 lighting configurations, plus 1 game with the slots swapped, plus the light-change run.
- *Expected:*
  - The vehicle, the infantry, the aircraft and the building show slot *b*'s ramp, and all four darken at ambient 0.43 (53-shade). Under both configurations the vehicle and the infantry are drawn at the cell's unscaled ground level, because test lighting sets the extra-light fields to 0 (S-0104, claim 13).
  - The building's own animation shows slot *b*'s ramp, its other pixels follow the unit palette file, and it darkens at ambient 0.43 to the same shade row as the infantry.
  - The deploy and undeploy animations show slot *a*'s ramp where they have remap pixels, and their other pixels follow the unit palette file. They show the same pixels under both lighting configurations (brightness 1000), while the vehicle itself darkens at 0.43.
  - The parachutes show slot *b*'s ramp where they have remap pixels, and their other pixels follow the unit palette file. They darken at ambient 0.43 to the same shade row as the infantry on the ground.
  - The terrain-palette building shows no ramp and takes the light's colour like the tiles around it.
  - The animated tile takes the coloured light like the tiles around it.
  - The vein animations show the human player's ramp: slot *a*'s, then slot *b*'s after the swap.
  - The alternative-palette animation shows the ramp of the first colour definition (position 0) and shows exactly the same pixels under both lighting configurations.
  - Both row-5 animations follow the anim palette file. The one with the use-normal-light flag shows the same pixels under both configurations; the one without it darkens at 0.43.
  - In the light-change run, the falling infantry, the building and the building's own animation darken to the 0.43 shade row, while the parachutes keep the pixels they showed under test lighting.
- *Falsified if:* any item differs from its expectation. In particular:
  - the building's own animation, the deploy or undeploy animation or a parachute follows the anim palette file, or shows a ramp other than its owner's,
  - the deploy or undeploy animation darkens at 0.43 (its brightness is not fixed at 1000), or a parachute or the building's own animation keeps its test-lighting pixels at 0.43 (its brightness is not the cell's),
  - the flagged row-5 animation darkens at 0.43, or the unflagged one does not,
  - in the light-change run, the parachutes darken with the infantry, or the building's own animation keeps its earlier pixels while the building darkens,
  - or the animated tile does not take the coloured light.
- *Scope of a failure:* the building's own animation, the deploy and undeploy animations and the parachutes falsify only row 3 and its brightness rules; the animated tile falsifies only row 2.

**PF: the first definition of a name wins (claim 5).**
- *Setup:* the PC map. Run 1 has an override colour section in the map that defines the light-grey name again, spelled exactly as in the base rules, as (0, 255, 255). Confirm first that the base rules define that name. Run 2 repeats it with mixed letter case. Run 3 removes the map override and puts the mixed-case redefinition in a named test copy of the selected game-mode file. Run 4 instead puts it in a named valid language rules test file, with the matching integrity digest required by S-0006. All test files contain their respective colour section, and the language file must pass validation so the run does not merely skip it. Keep a bystander alive after taking the screenshot.
- *Procedure:* screenshot the neutral building in each run. Then play an unaltered PC map in the same session and screenshot it again, to test that the previous game's definitions do not leak into this game. Remove the named test rules files before that following game.
- *Observable:* top bits of the neutral building's remap pixels in both games of each sequence.
- *Runs:* 4 sequences of two games, plus the PC reference.
- *Expected:* every sampled ramp is identical to PC, not the red test ramp. The mixed-case entries keep the same definition as the exact-case entry. The following game also shows PC's ramp.
- *Falsified if:* any sampled pixel differs from PC, in particular if a valid higher layer supplies the red ramp, mixed case creates a different shown definition, or the following game's ramp depends on the preceding map.

**PG: component wrap (claim 1). Test data: a loose palette file.**
- *Setup:* the PA map, with a loose copy of the temperate tile palette file whose entry for one known terrain index has components 64, 65 and 100.
- *Procedure:* as in PA.
- *Observable:* top bits at that index.
- *Runs:* 1.
- *Expected:* the palette components are 0, 4 and 144, so the 5-6-5 top bits after the −1 are red₅ 0, green₆ 0 and blue₅ 17.
- *Falsified if:* any of them shows as a clamped 252 (top bits 31, 62 and 31), or as any other value.

**PH: missing palette files (claims 2 and 3). Test data: loose copies of stock archives with named members left out.**
- *Setup:*
  - Make the desert unit palette file and the desert tile palette file absent: for the stock nested archive that holds them, place a loose copy of it, minus those two files (S-0005, claim 3). If the two files are held by different archives, or a file is also held by another archive or as a loose file, every such copy is replaced or removed the same way, so that neither file can be found at all. Keep every other member, the desert theater palette file included.
  - Make the urban unit palette file absent by the same method. Keep the urban tile palette present.
  - The snow unit palette file must stay, because it is read when the program starts.
  - Use the temperate, desert and urban maps of PB.
- *Procedure:*
  - In one session, play the temperate map, then the desert map, and screenshot the PB items.
  - In a new session, play the desert map first, then the desert map again.
  - In a third session, play temperate, desert, temperate and desert again.
  - In a fourth session, play desert, urban and desert, without a successful unit palette load in between.
- *Observable:* top bits of infantry pixels outside the remap range and of tile pixels; sample remap pixels separately to confirm that the house-colour ramp is still computed as in PC.
- *Runs:* 1 per session.
- *Expected:*
  - Temperate then desert: on the desert map, infantry show the temperate unit palette file with each component (16·*c*) mod 256, and tiles keep the temperate tile colours.
  - Desert first: infantry show the snow unit palette file with each component (16·*c*) mod 256, and tiles are black.
  - In the third session, the second desert game again shows the temperate unit palette file multiplied by 16, not by 64.
  - In the second session, the second game on the desert map shows the same pixels as the first.
  - In the fourth session, the unit palette's non-remap entries come from the snow file with multipliers 16, 64 and 256 modulo 256 respectively; thus they are all black in the third game. House-colour ramps remain those computed in PC. The first game's tiles are black, the urban game's tiles follow its file, and the last game's tiles keep the urban tile colours.
- *Falsified if:* any of these differs.

**PI: colour definition parsing (claim 4). Test data: a loose copy of the base rules.**
- *Setup:* a loose copy of the base rules whose colour definitions at positions *P*(0) … *P*(5) are `300,255,255`, `-86,255,255`, `10,abc,20`, `10, 128, 200`, `10 ,128,200` and a value of 66 characters: `10,128,` followed by 56 zeros and then `200`. No line before any of these positions (the highest is *P*(3) = 14) has an empty value, so every line holds its intended position (Terms, *definition position*). Use the PC map.
- *Procedure:* as in PC, slots 0 to 5 in successive games of one session, then repeat slot 0. Keep the loose base-rules copy unchanged through these seven games. Before an eighth game in the same theater, change only the definition at position *P*(0) to `170,128,200`, and select slot 0 again.
- *Observable:* top bits of remap pixels.
- *Runs:* 8 games.
- *Expected:*
  - Slot 0 shows the ramp of (44, 255, 255), a yellow.
  - Slot 1 shows the ramp of (170, 255, 255).
  - Slots 2 and 4 show an all-black remap range.
  - Slot 5 shows an all-black remap range too: only the first 63 characters are read, which end inside the zeros, so the value is (10, 128, 0).
  - Slot 3 shows the ramp of (10, 128, 200).
  - The repeated slot 0 again shows the (44, 255, 255) ramp, not the previous game's slot-5 ramp. In the eighth game it instead shows the (170, 128, 200) worked-example ramp, confirming that the new game takes fresh definitions even without a theater change.
- *Falsified if:*
  - slot 0 shows red (clamped to 255) or black (value rejected),
  - slot 1 shows red (clamped to 0),
  - slot 2 or 4 shows a non-black ramp (a bad character skipped),
  - slot 3 is black (spaces rejected),
  - slot 5 shows the ramp of (10, 128, 200), like slot 3 (characters after the 63rd read),
  - the repeated slot 0 inherits a ramp from the preceding game,
  - or the eighth game keeps the earlier slot-0 ramp despite the changed base-rules definition.

**PJ: drawing-with-object flag off, and precedence (claim 8). Test data: a copy of the art database in a test expansion archive.**
- *Setup:*
  - A test expansion archive (S-0005; S-0007 calls it the *test expansion archive*) holding a copy of the art database with four changes, each to an animation used by nothing else on the test map:
    - Building B1's own animation: the drawing-with-object flag set to no, and the alternative-palette flag left off.
    - Building B2's own animation: the alternative-palette flag set to yes, and the drawing-with-object flag left at its default.
    - The vein animation: the alternative-palette flag set to yes.
    - The animation of the animated terrain tile: the alternative-palette flag set to yes.
  - The PE map and houses: B1 and B2 are owned by the computer player (slot *b*), the vein field is not the human player's (slot *a*), and the animated tile stands inside the coloured light, as in PE. Choose B1 and B2 so that their own animations have pixels in the remap range.
  - The two lighting configurations of PE.
- *Procedure:* screenshot B1's and B2's own animations, the vein animations and the animated tile under both configurations.
- *Observable:* top bits of their pixels, compared with the ramps computed as in PC, with the unit palette and anim palette files, and with the single-shade ramp of the first colour definition.
- *Runs:* 2 lighting configurations.
- *Expected:*
  - B1's animation follows the anim palette file (row 5) and shows no ramp: its pixels at remap indices show the anim palette file's entries there. It darkens at ambient 0.43.
  - B2's animation shows slot *b*'s 53-shade ramp and darkens at 0.43 (row 3 is met before row 4).
  - The vein animations show slot *a*'s 53-shade ramp and darken at 0.43 (row 1 is met before row 4).
  - The animated tile still follows the tile palette and takes the coloured light like the tiles around it (row 2 is met before row 4).
- *Falsified if:*
  - B1's animation shows slot *b*'s ramp, or any ramp (the flag is ignored),
  - B2's animation or the vein animations show the first colour definition's ramp, or keep their pixels at 0.43 (row 4 is met first),
  - or the animated tile shows the first colour definition's single-shade ramp, or stops taking the coloured light.

**PK: fixed setup-screen swatch colours and flat fills (claims 11 and 12).**
- *Setup:* the measurement setup, 800×600, an unscaled and unfiltered display path as above, director and unrelated mods off. Open the skirmish setup screen with the other player entries empty so all eight fixed colours are available. No map is running, so game speed, battlefield lighting and tick recording do not apply. Set the speed control to index 2 for the later battlefield comparison.
- *Procedure:* capture the open colour list with the pointer outside the sampled interiors. Select each fixed colour in turn and capture its closed control. Repeat in a fresh program run with one named loose copy of the base rules, changing only the colour definition at position P(0) to the invented value (170, 128, 200). Keep all names, positions and other entries unchanged. Start the PC test map with slot 0 to confirm that the changed definition supplies the worked-example battlefield ramp, then remove the test file.
- *Observable:* the top bits of every unobscured interior pixel in each open-list choice and selected choice, plus the slot-0 battlefield remap pixels in the test-file run. Exclude text, arrow, border and pixels covered by the pointer. Use the display-path and pixel-reading rules above, but not the battlefield lit-component subtraction for swatches.
- *Runs:* one stock program run and one named-test-file program run, each capturing all eight fixed choices.
- *Expected:* every interior pixel for a slot matches that slot's Constants RGB after the 16-bit step; its open-list and selected-control colours match. There is no interior image pattern or gradient. The modified definition changes the battlefield ramp as in PC but leaves all swatch pixels unchanged. For comparison, compute the definition's own HSV-derived RGB and all 16 ramp entries from the data read at measurement time: no equality caused by pixel quantisation establishes that the swatch was derived from that entry.
- *Falsified if:* any unobscured interior pixel differs from the packed constant, the interior has an image-dependent pattern, the open and selected fills differ, or the changed definition changes the swatch. A failure to obtain the changed battlefield ramp invalidates the independence sub-test rather than confirming it.

## Open questions

Q-0280 is resolved by claims 11 and 12: the setup swatch is a flat fill of a fixed RGB constant, not the definition's own colour or a ramp entry. S-0412 claims 1 and 7 continue to own slot selection and random-colour resolution; neither determines a swatch shade.

Q-0214 is resolved by claim 8: S-0112 claims 1–3 own the established overlay, smudge and terrain-object palette rules. No residual palette question for those subjects remains here; the deck and marker exceptions still apply to ore and gems.

- **Saved games.** Whether loading a saved game makes the next game reload the palette files, whatever its theater, is not established.
- **Snow unit palette missing at program start.** What the game does if the snow unit palette file itself is missing when the program starts is not established.
- **The theater palette after theater loading.** The start-up resource table uses the temperate theater palette read at program start (S-0112, claims 1 and 2). Whether any drawing depends directly on the theater palette loaded later for the current theater remains open. That loaded palette is also an input to shade-table building (S-0104), with no visible effect established on 16-bit output.
- **Rows 2 and 3 of claim 8.** Which animations count as terrain animations, beyond animated terrain tiles, is not fully established. The list of an object's own animations (claim 8, row 3) may also be incomplete.
- **Brightness of a building's own animation.** How a building's flashing, or an effect that changes the building's brightness, changes the brightness its own animation takes (claim 8) is not established (S-0104, claim 13).
- **Projectiles by parachute.** What makes a projectile come down by parachute, and whether stock play ever does, is not established. Until a way is known, the projectile exception of claim 8 cannot be measured. What happens when the rules name no parachute animation (the "not established" defaults in Import mapping) is not established either.
- **Disguise.** An object whose disguise is in effect against the local player (glossary, *disguise*) may be drawn in the disguise house's colour. That belongs with the disguise rules and is not claimed here.
- **Types with their own palette.** A type may be drawn with a palette of its own, recoloured per house. Whether stock data uses this, and which key selects it, is open.
- **Houses defined by a map.** How houses defined by a map (campaign-style maps) choose their colour is outside this spec's skirmish scope.
