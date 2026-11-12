---
id: S-0103
title: Palettes and house colour
topic: rendering
revision: 25
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-12
measurements: []
requests: [Q-0214, Q-0280, Q-0275, Q-0416, Q-0417, Q-0419, Q-3019]
supersedes: []
---

# S-0103: Palettes and house colour

## Summary

This spec covers skirmish rendering and the colour import needed to inspect map-defined owners without skirmish reassignment; network games and campaign gameplay are out of scope. It describes how the palette files become palettes, how a colour definition becomes the 16-entry house-colour ramp, which colour definition each house shows, and which palette each kind of drawn object uses. It also says which palette indices keep their colour under coloured light, and gives the fixed colours of the skirmish setup screen's colour swatches. How a palette is shaded by light into rows of hi-colour pixels is in S-0104 (lighting). This spec only names the inputs to that shading.

## Terms

Glossary terms used here include *palette*, *theater*, *house*, *house colour*, *local player*, *rules layer*, *base rules*, *map layer*, *override section*, *lobby*, *measurement setup* and *quirk switch*.

- **Palette file.** Supplies 256 ordered colours, each with red, green and blue components represented by byte values intended to range from 0 to 63.
- **Unit palette.** The theater's palette for units, infantry, aircraft and buildings. House-colour palettes are built from it.
- **Tile palette.** The theater's palette for terrain tiles.
- **Theater palette.** The theater's general-purpose palette. The temperate theater palette read at program start supplies the start-up resource table (S-0112, Terms and claims 1 and 2); later theater loads do not change that table.
- **Anim palette.** The palette for animations that use neither the tile palette nor a house-colour palette. It is the same in every theater (claim 1).
- **Icon palette.** The dedicated palette for sidebar build icons, loaded at program start and unchanged during the session (claim 13).
- **Named palette.** A palette built like a 53-shade house-colour palette, but from a palette
  file that a type names in its art in place of the unit palette (claims 16 and 17). It is not a
  house-colour palette: a statement here about house-colour palettes covers named palettes only
  where it says so. S-0104 calls these *named palettes* too.
- **Colour definition.** A named colour given as three bytes (H, S, V): hue, saturation, value. The hue runs 0–255 over the full circle, so 0 and 255 are both red.
- **House display RGB.** The display-precision RGB sample of a house's colour used as an input to radar drawing and house laser colour (claim 15), not the definition's stored HSV triple or a setup-screen swatch.
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

1. **Palette files are multiplied by 4 in 8-bit arithmetic.** Each palette loaded from a palette file (the unit, tile and theater palettes of a theater, the anim palette, and the icon palette) turns a file component *c* into the palette component (4·*c*) mod 256. Well-formed files (*c* ≤ 63) therefore give components 0, 4, …, 252: every component loaded from a file is a multiple of 4 and at most 252. A component above 63 wraps (64 → 0). The anim palette is read once, when the program starts, from its own file (Constants), and is not reloaded when the theater changes.
   - Backing: `measurable` (PA for the ×4, PB for the anim palette, PG for the wrap, PL for the icon palette)
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

3. **When palettes are loaded and house-colour palettes built.** The unit, tile and theater palettes are reloaded only when a game starts in a different theater from the previous game of the session. The first game of a session always loads them. Every game, whatever its theater, builds all its house-colour palettes afresh from that game's colour definitions and the unit palette then in force. Named palettes are built as claim 17 describes, which differs in the original. When the theater's unit palette file is found, that palette is the one in force; when it is missing, the missing-file rule in claim 2 determines which palette is in force. The anim palette never changes during a session (claim 1).
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
   - **Units, infantry, aircraft and buildings** use their owner's 53-shade house-colour palette,
     except as below. A type with a named palette uses that instead, for the objects and
     animations that claim 16 lists.
   - **A building flagged in its art to use the terrain palette** uses the tile palette instead. Its body and sprite parts are shaded with the light colour and scaled ground level of the cell under the building's centre and show no house colour (S-0104, claim 13, including the larger-coordinate cell at an even foundation's centre). S-0104 also owns the brightness of its voxel parts.
   - **Animation art settings** for palette selection, drawing with a building, normal light and tiling come only from the animation type's own art section at every type read (S-0007 claim 34). A different image name, including a snow-suffixed name, does not redirect these settings. Missing or empty values keep the held setting, initially the defaults in Import mapping; they never fall back to the image-named section. This applies regardless of whether a preceding read found a sprite.
   - **Animations** select their ordinary palette and brightness from the first applicable row below. An animation that meets several conditions uses the earliest row it meets. Tiled copies have the palette exception below the table.

   | Precedence | Animation | Palette | Light and brightness |
   |---|---|---|---|
   | 1 | vein animation | the 53-shade house-colour palette of the **local player**, the same palette the local player's own units use when their type has no named palette (claim 16), whoever owns the animation | its cell's scaled ground level (S-0104) |
   | 2 | terrain animation | the tile palette | its cell's light colour, at its cell's scaled ground level (S-0104) |
   | 3 | an object's own animation that is drawn with its object (see below the table) | the object's named palette when its type has one (claim 16), otherwise the owner's 53-shade house-colour palette, even when the object is a building that uses the terrain palette | a brightness of its own, which depends on the kind of animation (see below the table) |
   | 4 | flagged for the alternative palette | the **single-shade** house-colour palette of the **first colour definition**, whatever the owner's colour | one row, so it ignores brightness and never takes an effect tint (S-0104, claim 3) |
   | 5 | any other | the anim palette | its cell's unscaled ground level (S-0104); the anim palette's shade table is never tinted (S-0104, claims 3 and 6) |

   - **Which own animations are drawn with their object (row 3).**
     - A shot animation is not an object's own animation drawn with its object, attached or not. It takes row 4 or 5 by its own art flags (S-0301 claim 22).
     - A unit's deploy or undeploy animation always is.
     - The parachute of any object other than a projectile always is, when a parachute animation type is held. A genuinely absent type produces no visible parachute. An ordinary unlisted animation name creates a type rather than selecting this absent-type case; missing art is not absence of the type. S-0522 claim 13 owns the release, playback and fall-rate consequences, including the distinction between a genuinely absent type and an unlisted name.
     - A building's own animation is when the animation's art flag for drawing with its object is set, and the flag is set when absent (Import mapping). With the flag off, the animation takes its palette from row 4 or 5, by its own art flags.
     - A projectile coming down by parachute is the exception. It gets a different parachute animation (Import mapping), which is not drawn with it: that animation takes its palette and brightness from row 4 or 5, by its own art flags, like an animation that belongs to no object.
   - **Brightness of row 3.**
     - A building's own animation is drawn at the building's brightness: the unscaled ground level of its top-left foundation cell (S-0104, claim 13), even for a terrain-palette building. Its brightness is set when the animation starts and set again on every tick, so it follows changes to the building's light. S-0104 owns the alternative cell source under its building-light quirk switch. How flashing, or an effect that changes the building's brightness, changes it is not established (Open questions).
     - A parachute is drawn at the unscaled ground level of the carried object's cell on the tick the parachute is created (S-0104, claim 13). It is not updated during the fall (Edge cases).
     - A deploy or undeploy animation is drawn at brightness 1000 (neutral), whatever the cell's light (Edge cases).
   - An animation flagged in its art to use normal light is drawn at brightness 1000 instead of the brightness in the last column. An untiled row-4 animation still has only one shade row, so the flag cannot change its pixels.
   - **Tiled copies** all use the anim palette, even when the ordinary selection would be a vein, terrain, owner's or alternative palette. They retain the selected brightness, including an own animation's brightness and the normal-light override; tiling does not force brightness 1000. For a tiled alternative-palette animation with no earlier row applying, the retained brightness is its cell's unscaled ground level unless the normal-light flag overrides it. The anim palette remains untinted, so a tiled terrain animation retains its scaled ground-level brightness but not the cell's coloured tint. S-0109 claim 27 owns copy placement, count and spacing, and S-0102 claim 13 owns their depth.
   - **Ore and gem overlays outside deck and marker cells** are drawn at neutral brightness and ignore the cell's light level (S-0104, claim 13). They use the start-up resource table: a 53-row untinted table built at program start from the temperate theater palette and unchanged when theaters are loaded (S-0112, claims 1 and 2). A drawn overlay in a deck cell instead uses the cell's tile shade table and scaled raised light level, whatever its type. S-0112 claim 1 owns this precedence and the marker-cell exclusion. Shroud cover pieces and light-glow images can still change ore and gem shade rows through the per-pixel light-alpha value (S-0112, claim 1; S-0110, claims 5 and 6).
   - **Particles** drawn as sprites (gas, smoke, fire and no kind) use the anim palette at brightness 1000 ‰ (neutral), whatever their cell's light level, height or lighting, and never a house-colour palette; row 5 of the table does not apply to them. The anim palette is never tinted. Shroud and fog pieces and light-glow images can still change their shade rows through the per-pixel light-alpha value (S-0104 claim 15; S-0110 claims 5 and 6). Spark and railgun particles are single pixels in their own colour and use no palette (S-0616 claims 24 and 28).
   - **Projectiles.** A sprite body uses the anim palette when its image has the anim-palette flag; otherwise, when its type has the firer's-palette flag, it uses the 53-shade house-colour palette of the house that owned the firer at creation, or the local player's current house palette when created without a firer; otherwise it uses the general palette defined by S-0306. A voxel body uses the palette selected by its type's colour definition, ordinarily the 53-shade palette, with the retained-palette and initial single-shade exception of S-0306 claim 30. S-0306 claims 30 and 31 own these selections and their fixed brightness of 1000 ‰ through the per-pixel light-alpha value.
   - **Target line.** A vehicle's target line (S-0301 claim 24) is drawn in palette index 8 of the general palette (S-0306 Terms; the file of S-0111's overlay palette), in its neutral shade row 26 of 53: never lit, tinted or house-coloured.
   - **Other overlays, smudges and terrain objects.** S-0112 claims 1 to 3 own their palettes, tint and brightness.
   - Backing: `measurable` (S-0301 scenario F20 part h for target-line palette index, neutral shade row and independence from house colour and lighting; S-0616 scenarios P25 and P29 for particle palette, brightness and pixel colours; S-0301 scenario F19 part h for shot-animation palette rows; S-0306 scenario P42 parts e, f and h for projectile palettes and brightness; S-0007 scenario A25 for own-section selection at every type read and missing/empty-key behaviour; S-0112 scenarios OV1, OV2 and OV18 for ore and gem palette, tint, theater independence and per-pixel shade-row changes, and OV7 part C for deck-cell precedence; S-0104 L23 parts B and E for the building animation's top-left source, tick updates and the terrain-palette building's centre-cell source; S-0522 scenario P8 for absent-type and unlisted-name parachutes; PE for the object kinds and rows 1 to 5, the parachute, the normal-light flag and the brightness of row 3, a light change during a parachute's descent and during a building's own animation included; PJ for a building's own animation with the drawing-with-object flag off and for the precedence of rows 1, 2 and 3 over row 4; PO for tiled copies' forced anim palette and retained brightness). `unmeasured` for the projectile's parachute, because the measurement setup offers no known way to make a projectile come down by parachute (Open questions). `unmeasured` for the precedence between rows 1, 2 and 3, because no way is known to create an animation that meets two of those conditions; it matters only if one exists. The terrain-palette building's own-animation source is `unmeasured` until a suitable stock subject for the L23 repeat is identified. The distinction between the parachute's creation tick and a later visible opening tick is `unmeasured`: PE tests brightness kept during the fall but establishes no separate, observable opening event.
   - Confidence: high for own-section imports, held-value retention and initial defaults. High for tiles, terrain-palette buildings, units, infantry, aircraft and buildings, and rows 1, 4 and 5. High for the palette of deploy, undeploy and parachute animations, the projectile exception, and tiled copies' palette and retained brightness. Medium for rows 2 and 3 otherwise, for the brightness of row 3 and for the precedence between rows. High for the ore and gem overlay palette and tint outside deck and marker cells, as specified by S-0112 claims 1 and 2; medium for deck-cell precedence, as specified by S-0112 claim 1. High for sprite particles' palette and fixed brightness; medium for their light-alpha response. Medium for shot-animation palette rows, voxel projectile palette selection and the no-firer fallback; high for the sprite projectile palette precedence and fixed brightness, and for the target-line palette index and fixed neutral shade row.

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

13. **Sidebar icon palette conversion and missing file (Q-0275).** The icon palette is read once at program start from the fixed file named in Import mapping, resolved by S-0005. Its components receive claim 1's multiplication and byte wrap. Every sidebar icon uses this palette, with no house-colour replacement even at indices 16–31. It is independent of the map's theater, battlefield light, global-effect tint and selected house colour. The palette in force at program start remains in force in every game of that session, even if a different side's archives subsequently provide a different copy of the same palette file.
    - An ordinary nonzero icon index shows its converted palette colour packed by claim 9's 16-bit step, without the neutral-light subtraction of 1. Index 0 is transparent. The icon's untinted shade table has 53 rows, with the middle row (26) reproducing the converted colour exactly. S-0411 claim 16 owns image-frame selection, placement and the separate darkening mask; that mask halves the already-packed destination channels, rather than selecting a darker palette row.
    - If the icon palette file cannot be resolved at program start, the original crashes before a skirmish can begin. It does not substitute the unit, anim or general overlay palette. **Our engine's departure:** use an all-black icon palette, keeping index 0 transparent, and issue an importer warning naming the missing file. There is no switch to reproduce the crash.
    - Backing: `measurable` (PL for conversion, transparency, independence, session lifetime and missing-file failure; S-0411 S11 for the mask). The existence of every unused shade row is `unmeasured`: sidebar states covered here show the middle row and a separate mask, not those rows.
    - Confidence: high

14. **Imported colour of a map-defined owner (Q-0416 and Q-0419).** For map inspection without skirmish reassignment, a house inherits its country's imported colour definition. Its own map section can replace that choice by naming a colour definition. Both selectors are names, compared without regard to letter case, not lobby slots or numeric definition positions. Only the first 31 characters of a selector are used. An absent or empty house selector keeps the inherited choice; an unknown name also keeps it, rather than selecting a new fallback colour. A country's absent selector keeps the value from an earlier rules layer; with no earlier value, its colour is the first definition, position 0. An unknown country selector likewise keeps the value in force. A numeric-looking selector is a name, not an index.
    - S-0009 claim 47 owns how a listed house selects its country, including the absent-selector country at registration index 0 and the creation of a default country for an ordinary unknown nonempty name. That newly created country's initial colour is definition position 0 here; the house's valid colour selector can still replace it. A house's name is not itself a country selector.
    - S-0009 claim 48 owns owner resolution: a country name supplies no colour unless it matches an actually created house. With a nonempty house list, a country-only or unknown owner is unresolved and its object is dropped, so it has no original remap colour. With an empty house list, houses are created for the registered countries and a matching owner inherits that actual house's colour, including its own section's valid override. Any optional diagnostic colour for an unresolved entry retained by a viewer is presentation-only, not an original remap colour, a colour-definition fallback or a sandbox house assignment; identify the entry as unresolved.
    - The selected definition supplies the house-colour ramp of claim 6 to both units and buildings under claim 8's palette rules. There is no cycling of lobby colours according to the order in which map objects first mention their owners.
    - This is colour import only, not a rule to create additional houses in a skirmish. S-0009 claim 27 still owns which owners survive skirmish loading, and claim 10 here still determines the colours of the resulting skirmish houses. S-0413 claim 5 already covers the country's field in a skirmish; it does not supply this map-house override rule.
    - Backing: `measurable` (PM, including its country-identity extension, an isolated import probe rather than a campaign-play acceptance test; S-0009 M27 for country selection, owner rejection and empty-list creation). An optional viewer diagnostic colour has no original pixel value to measure; it is not an original-semantics claim.
    - Confidence: high for inheritance, name lookup and defaults; medium for the resulting map-owner pixels before measurement

15. **House display RGB supplied to radar and house laser colour (colour-definition part of Q-0417).** A house's display RGB is sampled from palette index 16, the brightest entry of its selected definition's ramp, in row 26 of that definition's 53-shade palette when the house's colour is set up. It is not the three stored HSV bytes read as RGB, and is not a direct conversion of the unmodified HSV definition. The unit palette's file colours do not enter this sample: index 16 has already been replaced by the ramp.
    - With ordinary initial white tint, each ramp component *u* becomes max(0, *u* − 1), then receives claim 9's 16-bit step. To express this packed sample as an 8-bit RGB triple, widen each channel with zeros in its discarded low bits (Formulas). No extra multiplication by 4 applies. The subtraction belongs to shade-table conversion, not to HSV conversion.
    - The sample is fixed at house-colour setup. Later battlefield brightness, global-effect tint and trigger tint changes do not resample it. S-0410 claim 8 owns which house supplies a shown object's colour, including disguise, and whether it is shown at all; its drawing rules consume this sample. A house's sample itself has no building-versus-unit variant.
    - The same widened display RGB is the input to the house laser colour, computed once at house-colour setup by S-0518 claim 17 and its formula F5; it is not a separate HSV conversion or a later lighting sample.
    - Backing: `measurable` (PN; S-0518 scenario LB1 for the house laser colour input)
    - Confidence: high

16. **Which objects are drawn with a named palette, and a missing file (Q-3019).** Only an
    infantry, vehicle, aircraft or building type can name a palette file of its own, in its art
    section (Import mapping). Animation, projectile, particle, overlay, smudge, terrain-object
    and terrain-tile types never select a named palette by their own art, whatever it says. An
    animation takes a named palette only as an object's own animation drawn with that object's
    palette (*Its own animations* below).
    - **What uses it.** When an object's type has a named palette in the current theater (claim
      17), the object is drawn with the 53-shade named palette of its owner's colour definition
      (claims 10 and 14) instead of its owner's 53-shade house-colour palette. In the original
      this holds where no colour name is repeated in the read before the owner's definition
      position, and the other case is open (Open questions). Its brightness and shade row are
      chosen as before (S-0104). This covers infantry, aircraft and buildings, their voxel parts
      included, and vehicles drawn as sprites without a turret.
    - **Its own animations.** The object's own animations that are drawn with the object's
      palette (claim 8, row 3) use its named palette: a building's own animations drawn with it,
      deploy and undeploy animations and the parachute of a carried object.
    - **Vehicles that ignore it.** A vehicle drawn as a voxel model, or as a sprite with a
      turret, is drawn in its owner's ordinary 53-shade house-colour palette, even when its type
      has a named palette. Its own animations drawn with it still use the named palette.
    - **Terrain-palette buildings.** A building flagged to use the terrain palette keeps the tile
      palette for its body and sprite parts (claim 8). Its own animations drawn with it use the
      named palette.
    - **Disguise.** An infantry, aircraft or building object, or a vehicle drawn as a sprite
      without a turret, whose type has a named palette is drawn with that named palette at its
      real owner's colour definition even while it is drawn as its disguise for a viewing house.
      Unlike other disguised objects (S-0516 claim 37), it does not show the disguise house's
      colour. A disguised vehicle drawn as a voxel model or as a sprite with a turret, and an
      object disguised as an overlay or terrain object, are not covered here (Open questions).
      `quirks.named_palette_true_owner` (Edge cases).
    - **Missing file.** When the file for the current theater is not found at the read that claim
      17 describes, that file is not searched for again until the next theater change, even if it
      appears meanwhile, and the type has no named palette from it until then. Whether a file is
      missing is settled at that read, not when an object is drawn. Its objects and their
      animations are then drawn exactly as if its art named no palette file (claim 8; S-0516
      claim 37 for disguise). A file for another theater does not substitute, and a file name
      without the theater suffix is never tried.
    - Backing: `measurable` on PQ part A for the named palette of infantry, aircraft, building
      bodies drawn as sprites and turretless sprite vehicles, the building's own animation and
      any captured deploy, undeploy or parachute animation, the voxel vehicle drawn in its
      owner's ordinary palette, the key for a short value without an extension, and the exclusion
      of animations, projectiles and overlays. On PQ part C for the disguise rule of infantry. On
      PQ parts D and E for the missing-file rule, its other-theater and unsuffixed cases, and a
      file that stays missing until the next theater change. `unmeasured` for the other excluded
      kinds (particles, smudges, terrain objects and terrain tiles), vehicles drawn as sprites
      with a turret, the voxel parts of buildings, terrain-palette buildings, the own animations
      of a vehicle that ignores its named palette, and the disguise rule of aircraft, buildings
      and turretless sprite vehicles: PQ has no such subject, each would need a test type or
      fixture of its own, and each follows the same drawing choice as a measured subject.
      `unmeasured` for the key's 31-byte limit and a value ending in a file extension
      (Import mapping): PQ's test name is short and has no extension, and these details change
      only which file is searched (claim 17).
    - Confidence: high for the four kinds that can name a palette, the key selecting a named
      palette for a short value without an extension, the exclusion of animations, projectiles
      and overlays, the infantry and aircraft bodies, building bodies drawn as sprites and the
      missing-file rule. Medium for the other excluded kinds, the 31-byte limit and a value
      ending in an extension, the vehicle exception, the voxel parts of buildings,
      terrain-palette buildings, the list of own animations and the disguise rule.

17. **How and when a named palette is built, and its tint (Q-3019).** The file searched for is
    the value named in the art followed by the current theater's suffix *X* (Constants) and
    `.PAL`. It is resolved like any other data file (S-0005), and its components receive claim
    1's multiplication and byte wrap.
    - **Construction.** For every colour definition, a 53-shade named palette is built exactly as
      that definition's 53-shade house-colour palette (claims 6 and 7), with the named file in
      place of the unit palette. Entries 16 to 31 are the definition's ramp (claim 6). The other
      240 entries come from the named file, and indices 240 to 254 are tint-exempt (claim 7).
    - **When.** In the original and in our engine alike, each file named for the current theater
      is read when that theater's palette files are loaded (claim 3), and a file that no earlier
      game of the session named in that theater, such as one first named by a later game's map
      data, is read at the start of the game that names it. A later game in the same theater uses
      the file content read before.
      - *In our engine*, every game builds its named palettes when it starts, with its
        house-colour palettes, from the named files held for its theater, from that game's colour
        definitions and with white tint. No named palette is built during play, so none is built
        while a global effect is in force or after a trigger-action tint change (design rule,
        decided at the delegated sign-off, 2026-11-12).
      - *In the original*, the named palettes of files already named are built only when a game's
        theater differs from the previous game's, before play, from the colour definitions read
        for the previous game or at program start. A game in the same theater keeps the previous
        game's named palettes. Those of a file first named in a later game are built at that
        game's start, and show the colour definitions in force at the point where that game's
        data names the file.
    - **Tint.** In the original and in our engine alike, a named palette takes no light-source
      colour, and no map tint before a trigger-action tint change, and a named palette that an
      event retints takes the same tint as the 53-shade house-colour palettes (S-0104 claims 3, 5
      and 17). *In our engine*, every lighting re-evaluation, every choice of "normal" and every
      trigger-action tint change retints every named palette. *In the original*, such an event
      retints only the named palettes built for the first ⌊*N*/2⌋ colour keys read for the game,
      where *N* counts every colour key read in every rules layer, repeats included. Where no
      colour name is repeated before a definition position, those are the named palettes of the
      definition positions below ⌊*N*/2⌋. The event does not retint the others.
    - Backing: `measurable` on PQ part A for the file name of a short value without an extension
      and the construction, on PQ part B for the tint at its captures and the original's partial
      retint, on PQ part D for a file read on a theater change and its content reused by a later
      game in the same theater, and on PQ part E for a missing file staying missing until the
      next theater change. `unmeasured` for a file first named by a later game in the same
      theater: PQ part D reuses only a file the first game named, and naming a new file in a
      later game needs map art data of its own. `unmeasured` for the original's builds (only on a
      theater change or at a file's first read, from the previous game's colour definitions or
      those read so far, and kept by a later game in the same theater): a skirmish house shows
      only definitions that the base rules create (claim 10), the same in every read, and what a
      kept palette shows at the start of a same-theater game is open (Open questions). Our
      engine's build at every game start is a design rule. `unmeasured` for the 31-byte limit and
      a value ending in an extension, for the reason in claim 16. `unmeasured` for the tint taken
      back at a "normal" re-evaluation after a trigger-action tint change: PQ part B makes no
      such re-evaluation after its tint change, and the named palettes follow the house-colour
      palettes there as everywhere else (S-0104 claim 17).
    - Confidence: high for the file name of a short value without an extension, the construction,
      and the reading of a file on a theater change and the reuse of its content by a later game
      in the same theater when an earlier game named it, and the tint at PQ part B's captures.
      High for our engine's build at every game start, a design rule. Medium for the tint taken
      back at a "normal" re-evaluation after a trigger-action tint change. Medium for a file
      first named by a later game in the same theater, the 31-byte limit and a value ending in an
      extension, the original's partial retint (PQ part B is inconclusive when no definition
      position reaches ⌊*N*/2⌋), and the original's builds: when it builds, from which colour
      definitions, and that it keeps them across games in the same theater.

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

**Icon shade table** (claim 13). For converted 8-bit component *u* and row *j* = 0 … 52, the component before packing is min(255, ⌊*u* · *F_j* / 65536⌋), where *F_j* = ⌊131072 · *j* / 52⌋. Exact integer arithmetic, no floating-point operations. At *j* = 26, *F_j* = 65536 and the component is exactly *u*. Pack with the 16-bit step above. Index 0 is not painted.

**House display RGB** (claim 15). For an 8-bit ramp component *u*, ordinary initial white tint and a display channel of *k* bits:

> *a* = ⌊max(0, *u* − 1) / 2^(8 − *k*)⌋
>
> display RGB component = *a* · 2^(8 − *k*)

Exact non-negative integer arithmetic. Use *k* = 5, 6, 5 for red, green, blue in 5-6-5, or 5 for all three in 5-5-5. Packing the widened component again preserves *a* exactly; it does not apply another subtraction. For any non-white tint already present at colour setup, sample row 26 using S-0104 claim 6 instead; the widening rule is unchanged.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| remap range | 16 … 31 | palette index | 16 entries |
| tint-exempt range (house-colour palettes) | 240 … 254 | palette index | none in tile palettes |
| shade counts per colour definition | 1 and 53 | rows | S-0104 |
| palette file size | 768 | bytes | 256 × 3, components meant 0–63 |
| anim palette file | `ANIM.PAL` | file name | read once per session |
| named palette file | named value + *X* + `.PAL` | file name | claim 17; *X* from the theater table below |
| icon shade count | 53 | rows | middle row 26 reproduces converted file colour, claim 13 |
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
| palettes for theater A in force | a game in theater A starts | no palette file is read, except a named file that no earlier game of the session named in theater A (claim 17); house-colour palettes are built from that game's colour definitions and A's unit palette; named palettes as in claim 17 |
| palettes for theater A in force (or none loaded yet, as after program start, even when B is snow) | a game in theater B starts | the palettes for theater B are loaded under claim 2, including its missing-file rules; house-colour palettes are built from that game's colour definitions and the unit palette then in force; named palettes as in claim 17 |
| any | a game starts | named palettes are built for every colour definition of that game from the named files held for its theater, with white tint (claim 17, our engine); a named file is read when its theater's palettes are loaded or at the start of the first game of the session that names it in that theater |
| rules layers read | each new colour name | a single-shade and a 53-shade house-colour palette exist under that name, at the next definition position |
| rules layers read | a colour name that already exists | nothing changes |
| program starts with an icon palette file available | file is resolved | icon colours are fixed for the session by claim 13 |
| icon colours fixed for the session | another game, theater or side is selected | the icon palette does not change |

## Edge cases and quirks

- **Component wrap.** A palette-file component above 63 wraps after the ×4 (claim 1). `quirks.palette_component_wrap` (default on). Off: such a component becomes 252.
- **Missing unit palette multiplies again.** With its file missing, the unit palette in force is multiplied by 4 again on each theater change, and turns black after a few (claim 2). `quirks.missing_unit_palette_remultiplied` (default on). Off: a missing unit palette file leaves the unit palette in force unchanged, as a missing tile palette file does.
- **Missing palette files at the first load.** On the first load of a session, a missing unit palette file leaves the unit palette as the snow unit palette file multiplied by 4 twice, (16·*c*) mod 256, and a missing tile palette file leaves the tile palette all black (claim 2). This follows from the two rules above, so it gets no switch of its own. With `quirks.missing_unit_palette_remultiplied` off, a missing unit palette file leaves the snow unit palette file multiplied by 4 once.
- **Black last ramp entry.** Entry 31 is always black (claim 6). This is how the original behaves, not a bug, so it gets no switch.
- **The first definition of a name wins.** A later rules layer cannot change an existing colour definition (claim 5). `quirks.colour_redefinition_ignored` (default on). Off: a later layer's value replaces the earlier one for that name and keeps its position, and every house showing the name shows the new ramp.
- **Alternative-palette animations use the first colour definition, not the owner's** (claim 8, row 4). `quirks.alt_palette_first_definition` (default on). Off: the animation uses its owner's single-shade house-colour palette, and the first colour definition's when it has no owner.
- **Parachutes keep the light of their creation tick** (claim 8, row 3). If the light changes while an object comes down, its parachute keeps its creation-tick brightness while the object follows the new light. `quirks.parachute_brightness_fixed` (default on). Off: the parachute follows the unscaled ground level of the carried object's current cell, as a building's own animation follows its building.
- **Deploy and undeploy animations ignore the light** (claim 8, row 3). They are drawn at brightness 1000, so at night a deploying unit's animation is brighter than the unit. S-0104's `quirks.deploy_animation_unlit` (default on) owns this rule. Off: they take the unscaled ground level of the unit's cell (S-0104, claim 13 and Edge cases), without extra unit light or a height term.
- **Disguise does not hide a named palette's owner** (claim 16). An object covered by claim 16's
  disguise rule whose type has a named palette shows its real owner's colour definition even
  while drawn as its disguise.
  `quirks.named_palette_true_owner` (default on). Off: while drawn as its disguise for a viewing
  house, the object ignores its named palette and is drawn as S-0516 claim 37 says, in the
  disguise house's 53-shade house-colour palette.
- **Named-palette build and retint departures** (claim 17). Three departures from the original
  only change displayed colours, so they get no switch: our engine retints every named palette
  where the original retints only some, builds them from each game's own colour definitions where
  the original uses the previous game's or those read so far, and rebuilds them at every game
  start where the original keeps them across games in the same theater. Claim 17 states the fixed
  behaviour and records the original's in one sentence each (decided at the delegated sign-off,
  2026-11-12).
- **Vein animations use the local player's colour** (claim 8, row 1). Different players can see them in different colours. `quirks.veins_local_player_colour` (default on). Off: vein animations use the anim palette, like any other animation (row 5), so every player sees the same colours.

## Import mapping

The lobby swatch's RGB and flat fill (claims 11 and 12) have no data-file selector or absent-key default: they are fixed interface behaviour. In particular, the colour-definition values below feed battlefield palettes, not swatches.

| Source | Key | Parse | Feeds | Absent |
|---|---|---|---|---|
| fixed asset `CAMEO.PAL` | no INI selector | 768-byte palette file, components as in claim 1; resolution by S-0005 at program start | dedicated sidebar icon palette (claim 13), not the general overlay palette | original crashes at program start; our engine uses an all-black palette with transparent index 0 and an importer warning |
| rules `[Colors]` | each key | name = key, compared ignoring case; value as in claim 4 | one colour definition at the next definition position, if the name is new (claims 4–6 and 10); never the setup swatch RGB or fill (claims 11 and 12) | missing integers = 0; a line with an empty value creates no definition (S-0001, claim 11) |
| rules `[Colors]` | `LightGrey` | name, ignoring case | the definition both non-player houses of a skirmish show (claim 10) | engine constant name |
| rules layers, country's section | `Color` | colour-definition name, case-insensitive, first 31 characters, not a number | inherited map-owner colour (claim 14); skirmish country-field uses remain S-0413 claim 5's | keep earlier value; initially definition position 0; unknown name also keeps earlier value |
| map, house's section named by `[Houses]` | `Color` | colour-definition name, same lookup as the country selector | replace inherited country colour for that map-defined house (claim 14), not skirmish lobby assignment | absent, empty or unknown name keeps inherited country colour |
| map house's own section | `Country` | country-name selector, matching and truncation as in S-0009 claim 47 and Import mapping; not the house name | country whose colour claim 14 inherits | absent or empty: country registration index 0; ordinary unknown nonempty name: newly created default country, initially colour definition position 0; invalid reserved selectors and empty registry follow S-0009 claim 47's warned departures |
| map `[Houses]` and object-list owner values | list and owner fields in S-0009 Import mapping | actual-house creation and exact owner matching under S-0009 claim 48 | whether an original owner and remap colour exist at all (claim 14) | empty list creates houses for countries; unmatched owner drops the object, with no original colour; no data-file default selects a viewer diagnostic colour |
| colour definition selected for a house | no separate radar or house-laser key | ramp index 16, shade row 26, packed then widened by claim 15 | house display RGB supplied to S-0410 claim 8 and S-0518 claim 17 | no independent display-colour default; use the selected house definition |
| art, the art section (S-0007 Terms) of an infantry, vehicle, aircraft or building type | `Palette` | text; bytes after the 31st are ignored, which changes only which file is searched (claim 17); the theater suffix and `.PAL` are always appended, even to a value that already ends in a file extension | named palette (claims 16 and 17) | absent or empty at a type's first read of the game: no named palette; absent or empty at a later read: the earlier value is kept, an empty value counting as no entry (S-0001 claim 11; S-0007 claim 2); what the original shows after a later read without the key is not established (Open questions) |
| art, building entry | `TerrainPalette` | boolean | building uses the tile palette (claim 8) | no |
| art, animation type's own section at every type read (S-0007 claim 34), never image-named fallback | `AltPalette` | boolean | alternative palette (claim 8, row 4) | retain held value; initially no |
| art, animation type's own section at every type read (S-0007 claim 34), never image-named fallback | `IsVeins` | boolean | vein animation (claim 8, row 1) | retain held value; initially no |
| art, animation type's own section at every type read (S-0007 claim 34), never image-named fallback | `ShouldUseCellDrawer` | boolean | a building's own animation drawn with its building (claim 8, row 3); deploy, undeploy and parachute animations ignore it | retain held value; initially yes |
| art, animation type's own section at every type read (S-0007 claim 34), never image-named fallback | `UseNormalLight` | boolean | drawn at brightness 1000, including tiled copies (claim 8) | retain held value; initially no |
| art, animation type's own section at every type read (S-0007 claim 34), never image-named fallback | `Tiled` | boolean | copies force the anim palette but retain selected brightness (claim 8); placement, count and spacing in S-0109 claim 27 | retain held value; initially no |
| rules `[General]` (a game-wide section) | `Parachute` | creating animation reference, case-insensitive; ordinary unlisted names create a type; `none` and `<none>` clear it (S-0004 claims 2 and 13) | the parachute of an object other than a projectile (claim 8, row 3; S-0522 claim 13) | initially none; omitted or empty layered value retains the held type |
| rules `[General]` | `BombParachute` | creating animation reference, with the same lookup and reserved no-type names as above | the parachute of a projectile; not drawn with it (claim 8) | initially none; omitted or empty layered value retains the held type |
| map `[Map]` | `Theater` | theater identifier (Constants) | which palette files are used (claim 2) | (see S-0009) |
| overlay, smudge and terrain-object data | keys and value forms in S-0112 Import mapping | as specified there | palette, tint and brightness selection delegated by claim 8 to S-0112 claims 1–3, including deck-cell precedence | defaults in S-0112 Import mapping |

## Measurement scenarios

For claim 8's additional drawing subjects, S-0616 scenarios P25 and P29 supply the particle setups, pixel observables, expected colours and falsifiers; S-0301 scenario F19 part h supplies the attached and unattached shot-animation palette controls; and S-0306 scenario P42 parts e, f and h supply the projectile palette, initial and retained selection, no-firer fallback and light-alpha tests. Their named fixtures, procedures, expected results and falsifiers apply here. S-0301 scenario F20 part h tests the target-line addition: compare its pixels against general-palette index 8 in neutral shade row 26 of 53, repeating with different house colours, lighting and coloured light. Its named fixtures, distinguishable test-palette control and falsifiers apply here; a different colour or a response to those changes falsifies claim 8's target-line rule. S-0518 scenario LB1 likewise tests claim 15's widened display RGB as the house laser colour input, including its named colour-definition controls. These are proposed measurements, not recorded results.

S-0007 scenario A25 tests claim 8's own-section imports. Its named art and sprite fixtures, conflicting own/image-section controls, absent and empty own keys, and preceding-sprite/read-count variants apply to all five animation-property rows here. Record palette and brightness pixels using PE and PJ, and tiled-copy pixels using PO. Each result must match the own-section or retained-default control, never the conflicting image-section control. An image-section-dependent result falsifies the section rule; missing-key resets instead of retention falsify the retention rule. These are proposed measurements, not recorded results.

S-0522 scenario P8 tests claim 8's absent-type and unlisted-name parachute distinction: its named map overrides, release recording, expected absence of visible parachute for no type, and expected ordinary playback for a created unlisted type apply here. A visible parachute with genuinely no type, or rejection merely because an ordinary name is unlisted, falsifies the corresponding rule. Projectile release remains outside that fixture.

S-0104 scenario L23 part B tests the building animation's top-left foundation cell and per-tick light following; part E tests the terrain-palette building's centre-cell source. Repeat part B's two light placements with a terrain-palette building that plays an own animation drawn with it: the animation must still match the top-left reference, even when the body's light follows the centre cell. Record equivalent animation image frames; a match to the centre-cell reference instead falsifies the own-animation source. If no suitable stock subject is found, this last comparison remains unmeasured. Their setups, pixel observables, expected reference matches and falsifiers apply to claim 8 here.

Scenario labels PA to PJ are local to this spec; they are not measurement-record IDs. The ore and gem rules of claim 8 use S-0112's scenarios OV1 (palette and theater independence), OV2 (brightness and tint), OV7 part C (deck-cell precedence) and OV18 (shroud and glow shade-row changes), with the setup, procedure, observables, expected results and falsifiers given there.

All scenarios run under the *measurement setup* (glossary): an unmodified install, stock data and no engine add-ons, with the allowances the glossary lists. Every scenario that reads pixels may use a presentation-only display compatibility layer (Display path below). PF and scenarios PG to PJ add named test files, and each says which: test language and game-mode rules copies (PF), a loose palette file that overrides a shipped one on purpose (PG), loose copies of stock archives with the named palette files left out to make them absent (PH), a loose copy of the base rules (PI), or a copy of the art database in a loose test archive that wins over the stock archives (PJ; S-0005 and S-0007 say how such an archive ranks). The test files are used only for the scenario that names them and are removed after the run.

**Common setup and recording.** Use game speed index 2, detail level 2 and 800×600, with no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Each skirmish has a local human-controlled player and one bystander computer player in a separated start area; where a scenario names computer-owned subjects, those are exceptions to the separation. The observed area contains only the named test subjects, and the glossary's bystander void rule applies. Human-owned map objects removed by skirmish loading must instead be created in play, or placed under a retained owner for tests that do not depend on ownership. Use scripted input and lossless, tick-labelled screen capture; any missing harness support is a proposal for the user, not assumed installed. Unless cover or glow is explicitly tested by the cited scenario, sample explored pixels far from shroud edges and glow images, with light-alpha 127. Pause only after a required event has completed; timing tests need a capture for each tick.

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
- *Setup:* the measurement setup, 800×600, an unscaled and unfiltered display path as above, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Open the skirmish setup screen with the other player entries empty so all eight fixed colours are available. No map is running, so game speed, battlefield lighting and tick recording do not apply. Set the speed control to index 2 for the later battlefield comparison.
- *Procedure:* capture the open colour list with the pointer outside the sampled interiors. Select each fixed colour in turn and capture its closed control. Repeat in a fresh program run with one named loose copy of the base rules, changing only the colour definition at position P(0) to the invented value (170, 128, 200). Keep all names, positions and other entries unchanged. Start the PC test map with slot 0 to confirm that the changed definition supplies the worked-example battlefield ramp, then remove the test file.
- *Observable:* the top bits of every unobscured interior pixel in each open-list choice and selected choice, plus the slot-0 battlefield remap pixels in the test-file run. Exclude text, arrow, border and pixels covered by the pointer. Use the display-path and pixel-reading rules above, but not the battlefield lit-component subtraction for swatches.
- *Runs:* one stock program run and one named-test-file program run, each capturing all eight fixed choices.
- *Expected:* every interior pixel for a slot matches that slot's Constants RGB after the 16-bit step; its open-list and selected-control colours match. There is no interior image pattern or gradient. The modified definition changes the battlefield ramp as in PC but leaves all swatch pixels unchanged. For comparison, compute the definition's own HSV-derived RGB and all 16 ramp entries from the data read at measurement time: no equality caused by pixel quantisation establishes that the swatch was derived from that entry.
- *Falsified if:* any unobscured interior pixel differs from the packed constant, the interior has an image-dependent pattern, the open and selected fills differ, or the changed definition changes the swatch. A failure to obtain the changed battlefield ramp invalidates the independence sub-test rather than confirming it.

**PL: sidebar icon palette conversion, lifetime and missing file (claim 13).**
- *Setup:* the common skirmish setup, using six tiny flat maps, one per theater, and a retained buildable sidebar entry. Named test files: a loose icon sprite whose image frame 0 has labelled stripes of indices 0 … 255, and a loose copy of the dedicated icon palette. Set selected palette entries to invented triples (1, 2, 3), (63, 32, 16) and (64, 65, 100), including an entry in the remap range. Leave other data unchanged. Use S-0007's icon-file selector only to attach that sprite to the test entry. Still no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Sample unobscured ordinary icon pixels, not the mask, labels, clocks or mouse-over styling.
- *Procedure:* in one program session start the six maps successively, varying the local player's side and lobby colour between games. Include neutral lighting, a strongly coloured map tint and a trigger-started ion storm, and capture the icon before and during the storm. After the first game replace the loose palette with a second named version giving the sampled entries different invented triples; start another game without restarting the program, then restart and repeat. Remove the named test files afterwards.
- *Observable:* top bits of identified icon pixels, and unchanged background beneath stripe index 0. Record image index and state for each sample.
- *Expected:* each nonzero index shows the packed value of (4·*c*) mod 256 from the palette version present at program start, with no −1 subtraction, remap or battlefield tint. In 5-6-5 the three invented triples above yield channel tuples (0, 2, 1), (31, 32, 8) and (0, 1, 18). Index 0 leaves the background untouched. Theater, side, house colour and storm do not change these results. The replacement palette has no effect within the session but supplies the colours after restart. S-0411 S11 separately checks the mask's channel halving.
- *Missing-file run:* in a fresh program session, use named loose copies of the stock containing archives with every copy of the dedicated icon palette omitted, as in PH. Remove the loose test palette too, verifying that no loose or mounted copy can resolve. Keep other members unchanged. Record whether the program reaches skirmish setup and can begin a game. Expected: the original crashes during startup, before a skirmish begins. On our engine the importer warns and nontransparent icon pixels are black instead.
- *Runs:* one six-theater session plus its palette-replacement game, one restarted replacement-palette game, and one missing-file startup.
- *Falsified if:* any sampled pixel disagrees with the formula, index 0 paints, a battlefield or owner change changes colours, the palette changes within a session, a restart fails to use the replacement, or the original successfully begins a skirmish with the palette genuinely absent. Failure to expose the named test entry or remove every resolving copy makes that part inconclusive.

**PM: map-owner colour inheritance and name lookup (claim 14).**
- *Setup:* a tiny purpose-made single-player map used solely to expose map-house import, with separated visible stock infantry and buildings of each test house, test lighting, game speed index 2 and 800×600. No campaign progression or campaign gameplay rule is tested. There is no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Named test data: a loose base-rules copy giving three test countries distinct existing colour-definition names, and test map house sections selecting those countries. Give one house a different valid definition name, one the same name in mixed case, one an unknown name, one no selector and one an empty selector. Add a country with no colour selector in any layer and one with an unknown selector in every layer. Preserve a valid first definition and every name used by the test.
- *Procedure:* load the isolated test map directly through the original's single-player map entry path; record identified sprite remap pixels once import completes. Repeat with a second loose rules copy that changes the inherited country selector, without changing house selectors. For the 31-character limit, add a named colour with exactly 31 characters and give a house that name followed by additional characters. For numeric parsing, use an undefined numeric-looking name as another house selector.
- *Observable:* top bits of identified remap pixels, compared with claim 6's ramps under test lighting. Record the resolved owner of every subject. One run per rules-copy variant suffices.
- *Expected:* the explicit and mixed-case houses use their named definition; unknown, empty, absent and undefined numeric selectors inherit their country's definition and follow the country change in the repeat. Countries with no successful selection use position 0. The long selector selects the definition matching its first 31 characters. Infantry and buildings match the same selected ramp, subject to claim 8's palette exceptions.
- *Falsified if:* a selector is treated as an index, a bad name picks a different fallback, inheritance fails, case changes the selected name, truncation differs, or a sampled pixel follows another ramp. If no direct map-entry harness is available, propose one to the user; do not patch the executable or treat a skirmish that discards these houses as a test of inheritance. This probe is outside the skirmish suite and does not establish campaign support.

**PM country-identity extension (claim 14; Q-0419).**
- *Setup:* PM's isolated map-entry probe and named rules copy, retaining its lighting, speed, resolution and no-add-on conditions. Follow S-0009 M27's country and house fixtures: listed houses with unrelated names, a house named like one country but selecting another, absent and empty country selectors, an ordinary unknown nonempty country selector, and separated objects with listed, country-only and unknown owner names. Give country registration index 0 a valid colour distinct from definition position 0 and from the other test countries. Leave house colour selectors absent except for a valid override on one unknown-country house. Repeat with no house-list entries.
- *Procedure and observable:* capture identified infantry and building remap pixels after import; record which test objects exist. Compare pixels with the ramps computed by claim 6, not viewer diagnostic swatches. Use one run per fixture variant, with owner spelling and expected actual house recorded as in S-0009 M27.
- *Expected:* a listed house inherits its selected country's ramp regardless of its own name. Absent and empty country selectors inherit country registration index 0's colour; an ordinary unknown nonempty selector instead inherits colour definition position 0 until a valid house override replaces it. A country-only owner has no object in the nonempty-list run. In the empty-list repeat it shows the corresponding actually created house's ramp if its owner name matches that house. An unknown owner has no object and therefore no original remap pixels in either run. A viewer retaining a diagnostic entry must label it unresolved, not report its diagnostic colour as the oracle result.
- *Falsified if:* any existing object's remap pixels follow another computed ramp, an absent selector uses the same-name country, an unknown-country house inherits country registration index 0's different colour, or an unresolved owner produces an original object. An unavailable direct map-entry harness leaves the probe pending; propose it to the user, not an executable patch.

**PN: house display RGB, not raw HSV or an unshaded ramp (claim 15).**
- *Setup:* the common skirmish setup with functioning radar, explored separated objects of the local player and a retained opponent, and test lighting. Use a named loose base-rules copy setting the definition selected by the local player's lobby slot to invented HSV (0, 255, 255); leave definition names and positions unchanged. Include an ordinary building, infantry and vehicle, with no disguise, no combat flashing and no overlapping radar pixels. Select a 5-6-5 display path. A map trigger later starts a storm and another changes a tint channel; keep the subjects alive and radar available, using the map's forced radar setting if necessary.
- *Procedure:* capture identified radar object pixels after listing stabilises, then during the storm and after the tint action. Repeat once with a second named rules copy using (170, 128, 200). Remove the test files afterwards.
- *Observable:* radar pixel channel top bits, plus battlefield remap pixels to confirm that the intended definition was loaded. Sample interior building pixels and isolated non-building pixels. Two games suffice.
- *Expected:* brightest ramp entries are (250, 58, 58) and (120, 120, 196) respectively (claim 6). At ordinary initial white tint their radar samples in 5-6-5 are (31, 14, 7) and (14, 29, 24) in packed channel units, or zero-widened RGB (248, 56, 56) and (112, 116, 192). Buildings and other subjects of one house show the same sample when S-0410's ordinary house-colour rule applies. Later storm and tint changes leave those radar samples unchanged, although battlefield pixels change. There is no second neutral-light subtraction at radar packing.
- *Falsified if:* any sample follows raw HSV, direct HSV-to-RGB without the ramp adjustment, another ramp index, an unshaded ramp, a setup swatch, a later tint, or a different rule for one object kind. Failure to expose a stable, unflashed shown object makes that sample inconclusive. S-0410 owns the visibility and object-kind drawing tests, not this colour-definition conversion.

**PO: tiled copies force the anim palette but retain brightness (claim 8).**
- *Setup:* common skirmish setup on a tiny flat temperate map. Named test data: a test expansion archive containing an art-database copy and a labelled animation sprite with known nonzero indices, including remap indices and indices whose unit, tile and anim colours differ after packing. Use a retained opponent-owned building's own animation drawn with the building, and a separate trigger-created alternative-palette animation. Keep both opaque, non-vein and free from cover, glow and overlap; use a positive stored height for frame 0. Test map ambients 1.00 and 0.43, with other test-lighting settings unchanged. Read candidate palette entries at measurement time and choose indices whose expected packed colours discriminate both palette and shade row.
- *Procedure:* for each subject and ambient, capture the same image frame with tiling off, then with tiling on. Repeat the tiled runs with the normal-light flag on. Change only the named flags between runs. Sample unobscured pixels in the lowest visible copy, identifying each index from the test sprite; do not infer a palette from an unqualified screenshot match. Remove the named files afterwards.
- *Observable:* top bits of each identified animation pixel, compared with claim 6's selected house ramp and the anim palette's untinted shade formula in S-0104 claim 6 at the brightness and shade row selected by S-0104 claim 15.
- *Runs:* twelve configurations: two subjects, two ambients, and untiled, tiled, or tiled with normal light.
- *Expected:* the untiled own animation follows its owner's 53-shade palette and darkens at 0.43; the untiled alternative animation follows the first definition's single-shade palette and does not darken. Both tiled subjects instead follow the anim palette and darken at 0.43, retaining the building's brightness and the cell's unscaled ground level respectively. With normal light enabled, both tiled subjects show brightness 1000 at both ambients. No tiled remap-index pixel uses a house ramp.
- *Falsified if:* a tiled pixel follows the ordinary selected palette, a tiled subject remains neutral-bright at 0.43 without the override, or a normal-light tiled subject darkens. A sample whose candidate packed colours coincide is inconclusive, not confirmation.

**PQ: named palettes (claims 16 and 17; Q-3019). Test data: a copy of the art database in a test
expansion archive, loose test palette files and, if needed, a loose copy of the base rules.**
- *Setup:* the common setup and test lighting, on a tiny flat temperate map and a tiny flat snow
  map. Named test data, as in PJ: a test expansion archive holding a copy of the art database in
  which the art sections of the following test types name the invented palette name `QTEST`: an
  infantry type, a disguise-capable infantry type, an aircraft type, a building type with an own
  animation drawn with it, a vehicle type drawn as a voxel model, and a vehicle type drawn as a
  sprite without a turret (named test copies of the art database and the base rules may make a
  vehicle type so). The own art section of one trigger-created animation, and the art sections of
  one projectile type and one overlay type, name it too. Loose test palette files: `QTESTTEM.PAL`
  with invented components that differ visibly from the unit palette and from the anim palette at
  every sampled index, including indices outside 16 to 31 and in 240 to 254, and an unsuffixed
  `QTEST.PAL` with different invented components as a control. No `QTESTSNO.PAL`. Read every
  expected palette entry from the test files and stock files at measurement time.
- *Houses:* the local player and the opponent own one of each test subject. The measurer counts
  *N*, the colour keys of every rules layer read for the game, repeats included. Choose lobby
  slots whose definition positions (claim 10) lie on both sides of ⌊*N*/2⌋. No colour name may be
  repeated in the read before the highest definition position used. The measurer checks this
  while counting *N*, and a run that cannot meet it is void (Open questions, *Repeated colour
  names*). If no lobby slot gives a position at or above ⌊*N*/2⌋, add a building of the test type
  owned by a non-player house, provided the light-grey definition (claim 10) has such a position.
  If no available position is at or above ⌊*N*/2⌋, part B's partial-retint check is inconclusive,
  and the measurer records *N* and the positions.
- *Procedure:*
  - *A, neutral.* Temperate map. After each subject is placed and the building's own animation
    plays, pause and capture every subject. Capture a deploy or undeploy animation and a
    paradropped infantry's parachute if the test types offer them.
  - *B, effects.* On the temperate map, in a game that no game in the same theater precedes in
    that program session (a fresh program start, or after a snow game), so that the named
    palettes are built with white tint, a trigger starts an ion storm. Capture after the lighting
    re-evaluation that selects the storm (S-0104 Scenario L3), then after the storm ends. In a
    second run under the same condition, capture after a nuke flash's re-evaluation, and after
    the trigger-action tint change of S-0104 Scenario L21.
  - *C, disguise.* Use S-0516 Scenario CS16's disguise fixture with the opponent's
    disguise-capable test infantry disguised and viewed by the local player without detector
    coverage.
  - *D, other theater and same theater.* In the same program session, after the temperate games
    of parts A to C, replace `QTESTTEM.PAL` with a second version giving the sampled entries
    different invented components, then start the snow map, then the temperate map. End that
    temperate game during an ion storm. Replace the file with a third version, start the
    temperate map once more and capture its first drawn tick.
  - *E, missing file.* Remove `QTESTTEM.PAL`, restart the program and repeat A. Then, in the same
    program session, put the file back and start the temperate map again, then the snow map, then
    the temperate map, capturing every test subject in each.
- *Observable:* top bits of identified pixels of every subject and animation. Compare each with
  three references at its shade row and tint (S-0104): the named palette (claim 17), the owner's
  house-colour palette built from the unit palette, and the palette built from the unsuffixed
  control file. Record the lobby slots, *N* and the definition positions.
- *Expected (original):*
  - A: the infantry, aircraft, turretless sprite vehicle and building bodies, the building's own
    animation and any captured deploy, undeploy or parachute animation match the named palette at
    their owner's definition, including the ramp at 16 to 31. The voxel vehicle matches its
    owner's ordinary house-colour palette. The test animation, projectile and overlay are drawn
    as without the key (claim 8). No pixel at an index outside 16 to 31 matches the unsuffixed
    control.
  - B: named-palette pixels of definition positions below ⌊*N*/2⌋ take the same storm, flash and
    trigger tints as the house-colour pixels of that definition, except at indices 240 to 254.
    Those at or above ⌊*N*/2⌋ keep white tint throughout. On our engine all of them take the
    tints.
  - C: the disguised infantry is drawn with the named palette at its real owner's definition, not
    in the disguise house's colours.
  - D: the snow game shows every test subject in ordinary palettes. The temperate game after it
    shows the second version of the file, read on the theater change. The last game, in the same
    theater, still shows the second version, not the third. Record whether its first tick shows
    white tint or the storm tint of the game before (Open questions).
  - E: every test subject is drawn as without a named palette in the first game and in the
    temperate game started after the file is put back. The temperate game after the snow game
    shows the named palette.
- *Falsified if:* a listed subject follows the ordinary palette where the expected results give
  the named palette, the voxel vehicle, the test animation, the projectile or the overlay follows
  the named palette, a pixel at an index outside 16 to 31 follows the unsuffixed control, a
  named-palette pixel takes a tint that the expected results exclude or misses one they require,
  the disguised infantry shows the disguise house's ramp, a game without the suffixed file shows
  the named palette, the temperate game after the snow game shows the first version, the third
  version appears without a theater change, the file put back in part E shows before a theater
  change, or it does not show after one. A sample whose candidate packed colours coincide is
  inconclusive, not confirmation.

## Changes for other specs

S-0104 Terms *Named palettes*: which objects use named palettes and how they are built is covered
by claims 16 and 17 here. The sentence saying that neither S-0104 nor S-0103 covers it can point
there instead.

S-0104 claims 3 and 17, its Edge cases entry for `quirks.trigger_tint_retints_all_tables` and the
Backing of claims 3 and 17: in the original, a lighting re-evaluation, a choice of "normal" and a
trigger-action tint change retint only the named palettes of the first ⌊*N*/2⌋ colour keys read
for the game (claim 17 here). The event does not retint the others. Our engine retints every
named palette as S-0104 claims 3 and 17 describe, a display-only fix that claim 17 here gives no
quirk switch. S-0104 can say that its "every named palette" is our engine's behaviour and point
to claim 17 here for the original's. Its `unmeasured` backing for named palettes can cite
scenario PQ part B here.

S-0104 Open questions, the tint of a house-colour or named palette built after a trigger-action
tint change: no named palette is built during play, in our engine by design and in the original
only on a theater change or at the start of a game that first names a file (claim 17 here), so
the named-palette half can close. Its remark that such palettes are normally all built when the
game starts holds for named palettes only in our engine. The original keeps them across games in
the same theater (claim 17 here). The house-colour half stays open there.

S-0516 claim 37, "The disguise is drawn in its disguise house's colours": an object of a type
with a named palette, within the scope of claim 16 here, is drawn with that named palette at its
real owner's colour definition instead, unless `quirks.named_palette_true_owner` is off.

S-0522 claim 18, "an ordinary untiled parachute uses the carried infantry's house-colour unit
palette": when the carried infantry's type has a named palette, the parachute uses that named
palette (claim 16 here, *Its own animations*). The sentence can cite claim 16 here.

## Open questions

Q-0419 is answered jointly: S-0009 claims 47–48 determine country selection and owner resolution; claim 14 here supplies inherited, default-country and overridden colours. An unresolved owner has no original remap colour. Optional viewer diagnostic colours are not original colour inheritance. No original-semantics part remains open.

Q-0416's colour-selection and import questions are answered by claim 14. S-0009 claim 27 and S-0412 claims 1 and 7 already determine skirmish owner retention and lobby assignment; those rules do not assign colours to map-defined owners outside skirmish. Campaign gameplay remains out of scope.

Q-0417's colour-definition conversion is answered by claim 15. S-0410 claim 8 must consume this display RGB rather than describing the stored definition triple as RGB; object-kind equality and the radar's actual drawing remain that spec's responsibility.

Q-0275's palette conversion and missing-palette behaviour are resolved by claim 13. S-0411 claim 16 owns its image-frame, darkening-mask and placement answers; together the two specs answer the request.

Q-0280 is resolved by claims 11 and 12: the setup swatch is a flat fill of a fixed RGB constant, not the definition's own colour or a ramp entry. S-0412 claims 1 and 7 continue to own slot selection and random-colour resolution; neither determines a swatch shade.

Q-3019 is answered by claims 16 and 17: the kinds and key that select a named palette, its file
name and construction, its build before play and its tint, and the missing-file rule. The items
for it in Open questions below are edge cases of these answers, not unanswered parts of the
request.

Q-0214 is resolved by claim 8: S-0112 claims 1–3 own the established overlay, smudge and terrain-object palette rules. No residual palette question for those subjects remains here; the deck and marker exceptions still apply to ore and gems.

- **Saved games.** Whether loading a saved game makes the next game reload the palette files, whatever its theater, is not established.
- **Snow unit palette missing at program start.** What the game does if the snow unit palette file itself is missing when the program starts is not established.
- **The theater palette after theater loading.** The start-up resource table uses the temperate theater palette read at program start (S-0112, claims 1 and 2). Whether any drawing depends directly on the theater palette loaded later for the current theater remains open. That loaded palette is also an input to shade-table building (S-0104), with no visible effect established on 16-bit output.
- **Rows 2 and 3 of claim 8.** Which animations count as terrain animations, beyond animated terrain tiles, is not fully established. The list of an object's own animations (claim 8, row 3) may also be incomplete.
- **Brightness of a building's own animation.** How a building's flashing, or an effect that changes the building's brightness, changes the brightness its own animation takes (claim 8) is not established (S-0104, claim 13).
- **Projectiles by parachute.** What makes a projectile come down by parachute, and whether stock play ever does, is not established. Until a way is known, the projectile exception of claim 8 cannot be measured. The animation-reference defaults are established in Import mapping. S-0522 claim 13 resolves the non-projectile absent-type case; projectile-specific release and lifetime consequences remain unmeasured.
- **Disguise.** For an object whose type has a named palette, claim 16 states the colour it shows
  while drawn as its disguise, within the scope given there. For every other disguised object,
  the colour it is drawn in belongs with the disguise rules (S-0516 claim 37) and is not claimed
  here. Open: a disguised vehicle drawn as a voxel model or as a sprite with a turret whose type
  has a named palette, and an object of a named-palette type disguised as an overlay or terrain
  object.
- **Repeated colour names and named palettes.** A named palette is chosen by the position of the
  owner's colour definition. When a colour name is repeated in the read before a house's
  definition position, which colour that house's named-palette objects show in the original, its
  own definition's or another's, is not established, nor whether any skirmish arrangement reaches
  this.
- **A named palette dropped by a later read.** For a type whose art names a palette file at one
  read of a game and has no such key at a later read of the same game (an empty value counts as
  none, S-0001 claim 11), what the original then shows is not established: whether its objects
  keep the named palette chosen earlier, and what they show after the next theater change. Our
  engine follows S-0007 claim 2 and the Import mapping.
- **Named-palette tint at the start of a same-theater game.** Whether the original's kept named
  palettes start such a game with the tint they had when the previous game ended is not
  established (claim 17). PQ part D records it.
- **Invalid colour registries.** Claim 14 assumes at least the inherited colour definition exists. Colour selection when the rules supply no definitions, or otherwise lack a referenced inherited definition, remains unspecified here; an unknown selector in an otherwise valid registry is not this case and keeps the inherited definition.
