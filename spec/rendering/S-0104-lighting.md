---
id: S-0104
title: Lighting
topic: rendering
revision: 4
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-09
measurements: []
requests: []
supersedes: []
---

# S-0104: Lighting

## Summary

How the map's ambient light, tint and height factors, the light sources and the global screen effects (ion storm, nuke flash, chrono flash, psychic dominator) give each cell a light level and a tint, and how the ambient light fades over time. How a palette is shaded into rows of 16-bit pixels for a tint, and how a drawn object's brightness picks a row (palette loading and the house-colour ramp are in S-0103).

## Terms

- **Light level.** A per-mille brightness, usable range 0–2000. 1000 is unlit-neutral and 2000 is double brightness.
- **Tint.** A per-mille triple (r, g, b). (1000, 1000, 1000) is white light.
- **Raw cell tint.** The map tint plus the light-source tints reaching a cell, per channel in ‰, as computed at the cell's last full recompute (claim 9, Formulas, "Cell light"). It can lie outside [0, 2000].
- **Normalised tint.** The raw cell tint after clamping and normalisation (claim 8): either white, or a tint whose largest channel is set to 1000. The other channels are at most 1000, with one exception: when green is the largest channel and blue ties with it at an odd value from 3 to 13 ‰, blue comes out 1001 to 1003 (Formulas, "Tint normalisation"). Quantisation clamps it to 1000, so the excess never shows.
- **Quantised tint.** The normalised tint after quantisation by detail level (claim 8). Each channel is a multiple of 128, 64 or 32 at detail level 0, 1 or 2, so a quantised tint is never exactly white.
- **Quantised white.** The quantisation of white: (896, 896, 896), (960, 960, 960) or (992, 992, 992) at detail level 0, 1 or 2.
- **Neutral tile table.** The one tile shade table built for white (1000, 1000, 1000), with 53 rows and no quantisation. It is shared by every cell that uses it (claim 8).
- **Named palettes.** House-colour palettes built from a palette file other than the unit palette, for objects drawn with a palette of their own. Which objects use them and how they are built is not covered by this spec or by S-0103. For this spec they behave like the 53-shade house-colour palettes (claims 3 and 17).
- **Tint-exempt indices.** The palette indices that S-0103 exempts from tinting. In a shade table they follow brightness but ignore the tint (claim 6). Defined in S-0103.
- **53-shade and single-shade house-colour palettes.** The two kinds of house-colour palette of S-0103. Here the 53-shade ones are tintable (claim 3) and the single-shade ones are never tinted. Defined in S-0103.
- **Vein animations, terrain animations, anim-palette animations.** The animations of rows 1, 2 and 5 of the table in S-0103, claim 8 (claim 13). Defined in S-0103.
- **16.16 factor.** An integer *F* meaning the multiplier *F*/65536.
- **Base ambient, current ambient, target ambient.** Three ambient light values in hundredths (100 = 1.00).
  - *Base*: the value set by the map or a trigger action.
  - *Target*: where the light is heading.
  - *Current*: what is shown. It steps towards the target.
- **Global effect.** One of: *nuke flash* (brightening phase), *chrono flash*, *ion storm*, *psychic dominator* (any phase), or none. This is narrower than the glossary's entry: here the nuke flash counts as a global effect only in its brightening phase.
- **Ion storm.** The weather superweapon's storm. Its superweapon or a trigger action starts it. A start may be delayed, and the storm then starts on the tick the delay runs out. A start while a storm is in force does not start a second one. The storm is *in force* from the tick it starts to the tick it ends. It ends once its duration has run out and none of its lightning clouds is still pending. Both of those ticks are lighting re-evaluations (claim 5). How long a storm lasts and what it does apart from lighting belong to the superweapons specs.
- **Chrono flash.** The full-screen chrono warp effect that a trigger action starts, counted as in force from the tick the action runs until the effect finishes. It is **not** the chronosphere superweapon: a chronosphere jump does not start it and causes no lighting re-evaluation.
- **Nuke flash phases.** The *brightening phase* starts on the tick a nuclear missile detonates. The *fading phase* follows it. Then the nuke flash is over (claim 5).
- **Psychic dominator phases.** The dominator is *in force* from the tick it is launched until it is over. Its *final phase* starts when its animation reaches its last image frame. The dominator is over on the first tick after that on which the current ambient equals the target ambient (claim 5). The phases before the final one are its *active phases*.
- **Lighting re-evaluation.** An event at which the global choice of claim 2 is made again. The target ambient and the tint of every tintable shade table are set from it, and every cell gets an incremental update (claims 5 and 9).
- **Lighting set.** For "normal" and for each global effect: ambient, tint (r, g, b), ground bias and level factor.
- **Light source.** A point light with a position (leptons), a radius (leptons), an intensity (‰, may be negative) and a tint (‰ per channel, may be negative).
- **Shade table, shade row, shade count *N*.** See claim 6.
- **Tile shade table.** A shade table of the tile palette. Each one except the neutral tile table is built for one quantised tint and is shared by all cells that use that quantised tint (claim 8).
- **Detail level.** The player's visual-detail option, kept in the settings file (glossary; Import mapping): 0 = low, 1 = medium, 2 = high. The in-game options switch only between 0 and 2, so detail level 1 is reached only by setting it in the settings file before the game starts.
- **Light-alpha value.** A per-screen-pixel integer, 127 by default, that light-glow images can change locally. Values other than 127 come only from light-glow images, and their range is open (Open questions). It modulates object brightness (claim 15).
- **Cell light level (ground), cell light level (raised).** Two light levels per cell (claim 9). The first is for things at ground height. The second is computed for things four height levels up. The overlays of deck cells read it (claim 13); whether anything else does is open (Open questions).
- **Scaled and unscaled ground level.** The two forms of a cell's ground light level that drawn objects read (Formulas, "Cell light"): the *unscaled* one ignores the raw cell tint, the *scaled* one is multiplied by the tint-normalisation factor of claim 8.
- **Light-glow image.** An image that some objects and animations draw into the light-alpha values instead of onto the screen, to brighten or darken the screen pixels under it locally (a lamp's glow, for example). Only its effect on brightness belongs to this spec (claim 15). How it writes the values is open.
- **Shadowed cell.** A cell marked as lying under overhanging terrain, which darkens vehicles and infantry in it (claim 13). Two things mark cells: an intact high bridge (one with a raised deck) marks the cells it spans, and the mark is cleared when the bridge is destroyed; and a terrain tile whose tileset (glossary) is declared a shadow caster in the theater's data marks a fixed set of cells around it (Import mapping). Which cells a shadow-casting tile marks is open.
- **Height above the surface.** An object's world height minus the height of the surface below it, in leptons. The surface is the bridge deck for an object on a bridge deck, and the terrain of the object's cell otherwise. A bridge deck is 4 height levels (416 leptons) above the terrain of its cell.
- **Bridge deck.** The raised deck of a high bridge, 4 height levels above the terrain of its cell (S-0102, claim 15). An object "on a bridge deck" is standing or driving on it, as opposed to passing under it. A *deck cell* is a cell that the raised deck covers. The deck itself is drawn as overlays in the deck cells (S-0102).
- **Terrain-palette building.** A building whose art flags it to use the terrain palette (S-0103, claim 8). Every other building is an *ordinary building* here.
- **Ore-spawning terrain object.** A terrain object (glossary) whose type is flagged to spawn ore around it (Import mapping). Other terrain objects, such as ordinary trees, are *plain terrain objects*.
- **Wall overlay.** An overlay whose type is flagged as a wall (Import mapping).

## Behaviour

1. **Map lighting values and their precision.** The map supplies a lighting set for normal conditions, the ion storm and the psychic dominator.
   - Ambient and tint channels are kept in **hundredths**.
   - Ground bias and level factor are kept in **thousandths**.

   A decimal value *v* from the map is stored as ⌊*v* × 100 + 0.01⌋ (hundredths) or ⌊*v* × 1000 + 0.01⌋ (thousandths), where *v* is the single-precision decimal of S-0003, claim 6. The calculation is in double precision, truncating toward zero, under the original's round-toward-zero mode (glossary; S-0003, claim 17), whose switch `quirks.chop_rounding` belongs to S-0205 (number domains, planned) (Formulas, "Precision of decimal conversions"). The nuke flash set is fixed and cannot be changed by the map. Absent keys take the engine defaults in Constants.
   - Backing: `measurable` (Scenario L1)
   - Confidence: high

2. **Which lighting set is in force.** Three decisions choose a set, and each ranks the effects differently. In each decision the highest-ranked effect that is in force wins:

   | Rank | Global choice, made at each lighting re-evaluation (target ambient and every shade table's tint) | Per-cell height terms (claims 9–10) | Shade tables built while an effect is in force (claims 3, 6, 8) |
   |---|---|---|---|
   | 1 | nuke flash brightening phase, or chrono flash | ion storm | ion storm |
   | 2 | ion storm | psychic dominator, in **every** phase, the final one included | nuke flash brightening phase, or chrono flash |
   | 3 | psychic dominator, in an active phase **other than its final phase** | nuke flash brightening phase | psychic dominator, in any phase |
   | 4 | normal | normal | no override |

   The global choice takes effect only at a lighting re-evaluation (claim 5); between two re-evaluations the target ambient and the tints stay as the last one set them, whatever effects start or end in between. Trigger actions are the exception: one that changes the base ambient can set the target (claim 14), and one that changes the map tint retints every tintable table (claim 17). The height terms and the choice for a newly built table follow the effects in force at the moment the cell is updated or the table is built. The chrono flash does not affect the height terms. A table built while an effect is in force (e.g. a tile table first needed during a storm) takes that effect's tint. Existing tables keep the one from the last global choice.

   So when a storm and a dominator overlap with a nuke flash, the tint follows the nuke and the height terms follow the storm or dominator. A tile table first created during a storm and nuke overlap gets the storm tint while the rest of the map shows the nuke tint. In the dominator's final phase, the tint and target ambient have already reverted (to normal, or to the storm's while a storm is in force), while the height terms, the fade interval (claim 4) and newly built tables still use the dominator's values. A chrono flash changes the target and the tints only if a lighting re-evaluation happens while it is in force (claim 5).
   - Backing: `measurable` (Scenarios L2, L16)
   - Confidence: high for each of the three orders and for the extent of the dominator's final phase

3. **Who takes which tint.**
   - **Outside global effects, only terrain takes colour.** The map tint and light-source tints reach only the tile shade tables (claim 8), so only what is drawn with a tile shade table takes them, including terrain tiles, terrain animations, plain terrain objects and terrain-palette buildings (claim 13, S-0103). Whether overlays are drawn with one, and so take these tints, is open (Open questions; their palettes belong to S-0103). The house-colour palettes and named palettes (Terms) are shaded with white light (1000, 1000, 1000) until a trigger action changes the map tint (claim 17), so units, infantry, aircraft and ordinary buildings change brightness with the cell but never colour. Vehicles drawn from voxel models are included: after their per-face shading their pixels go through the same house-colour palette (S-0105).
   - **Global effects replace the tints.** When a lighting re-evaluation chooses an effect's set (claim 5), the tile shade tables, the 53-shade house-colour palettes and the named palettes (the *tintable* tables) are rebuilt with that set's tint, whatever tint they had before. Coloured light sources stop showing their colour on the ground until the next re-evaluation that chooses "normal": every cell shows the effect tint. Their brightness still counts, because it goes into the cell light level, not the tint. When the global choice at a re-evaluation is "normal", every table goes back to its own tint: the tint it was built for, or the map tint a trigger action last gave it (claim 17).
   - **Never tinted:** the single-shade house-colour palettes (S-0103, claim 5) and the anim palette. The anim palette has 53 rows built without any tint (claim 6, "untinted table"), so anim-palette animations follow brightness but never take the map tint, light colours or effect tints.
   - Backing: `measurable` (Scenarios L3, L5 "anim palette", L21 and L23, and S-0103's scenario PD)
   - Confidence: high for the effect replacement and for house-colour palettes staying white; medium for the anim palette: its table is built untinted, but which drawing paths use that table is open (Open questions)

4. **Ambient fade.** On any tick on which the current ambient differs from the target and at least one fade interval (a whole number of ticks) has passed since the last step, the current ambient moves one *step* towards the target. A step never overshoots: within one step of the target, the current ambient becomes exactly the target. A negative target counts as 0: when a step falls due, the target is raised to 0 and the step is taken towards 0. Each step starts a new fade interval.

   The step is the rules' ambient change step, in hundredths (⌊value × 100⌋, Formulas, "Precision of decimal conversions"). The length of each interval depends on the effects in force at the step that starts it (Formulas, "Ambient fade"): the nuke-flash rate during either nuke flash phase or a chrono flash, and also for a step taken on the tick the nuke flash ends (tick 47 in claim 5, "Nuke flash timing"); failing that, the dominator rate while the dominator is in any phase, the final one included; otherwise the rules' ambient change rate in game minutes × 900 ticks per game minute, truncated. The ion storm uses the normal rate. If the rules' ambient change rate is 0, the current ambient never moves, even during nuke and dominator fades (Edge cases, "Zero change rate freezes ambient").
   - Backing: `measurable` (Scenarios L4, L12)
   - Confidence: high

5. **Lighting re-evaluations and what the global effects set.** At a lighting re-evaluation, the global choice of claim 2 is made from the effects in force at that moment. The target ambient becomes the chosen set's ambient, or the base ambient when the choice is "normal". Every tintable shade table (claim 3) takes the chosen set's tint, or goes back to its own tint when the choice is "normal". Every cell gets an incremental update (claim 9). The tint change is visible on the tick of the re-evaluation; the current ambient then fades towards the new target (claim 4).
   - **Events that re-evaluate:** a nuclear missile detonating (the brightening phase starts); the start of the nuke flash's fading phase; an ion storm starting; an ion storm ending; a psychic dominator being launched; the dominator's final phase starting.
   - **Events that do not:** the end of the nuke flash's fading phase; the dominator being over; a chrono flash starting or finishing. At these, the target ambient and the tints stay as they are. Only what follows the effects in force from moment to moment changes with them: the interval chosen at the next fade step (claim 4), the height terms at each cell's next update (claim 16), and the tint of tables built from then on.
   - **Overlaps.** The chosen set belongs to the highest-ranked effect in force in claim 2's first column. The nuke flash brightening phase and the chrono flash give the nuke set, an ion storm the ion set, and a dominator in an active phase the dominator set. With none of these in force the choice is base: the base ambient, and every table's own tint. For the overlaps that matter, this gives the following, with no nuke brightening phase and no chrono flash in force unless the row names one:

     | Event | Effects still in force after it | Target ambient and tint from then on |
     |---|---|---|
     | a nuclear missile detonates | anything | nuke set |
     | nuke flash enters its fading phase | ion storm (with or without a dominator) | ion set |
     | nuke flash enters its fading phase | dominator in an active phase | dominator set |
     | nuke flash enters its fading phase | dominator in its final phase, or nothing | base |
     | ion storm ends | dominator in an active phase | dominator set |
     | ion storm ends | dominator in its final phase, or nothing | base |
     | ion storm ends | nuke flash brightening phase | nuke set (no visible change) |
     | dominator is launched, or enters its final phase | ion storm | ion set (no visible change) |
     | dominator enters its final phase | nothing else | base |
     | any re-evaluation | chrono flash | nuke set; it stays after the chrono flash finishes, until the next re-evaluation (Edge cases, "Chrono flash lighting persists") |
     | nuke flash is over, dominator is over, chrono flash starts or finishes | anything | unchanged |

   - **Same-tick order.** When a re-evaluation comes from an effect's own timing (the nuke flash entering its fading phase, an ion storm ending, the dominator entering its final phase), the fade on that tick still steps towards the old target. The first step towards the new target comes on the next tick at the earliest.
   - **Nuke flash timing.** On the tick a nuclear missile detonates (tick 0), the nuke set is applied, and the first fade step comes on tick 1, however much of the current fade interval has passed. The brightening phase lasts 31 ticks, ticks 0 to 30. The fading phase lasts 16 ticks, ticks 31 to 46; on tick 31 the re-evaluation changes the tint back (to whatever it chooses), and the first darkening step is on tick 32 at the earliest. The nuke flash is over from tick 47 on: a fade step taken on tick 47 still gets the nuke rate as its interval, and one taken on tick 48 or later does not (claim 4).
   - **Dominator timing.** When a dominator is launched, the first fade step comes on the next tick in the same way. The dominator's final phase starts when its animation reaches its last image frame. The dominator is over on the first tick after its final phase starts on which the current ambient equals the target ambient (on the tick of the fade step that reaches the target, if that step comes later). Its being over causes no lighting re-evaluation and no incremental update (claim 16).
   - Backing: `measurable` (Scenarios L2, L4, L12, L16)
   - Confidence: high for which events re-evaluate and which do not, and for the nuke flash ticks; medium for the image frame at which the dominator's final phase starts (Scenario L16 Part B checks it only to the tick)

6. **Building a shade table.** A shade table turns a palette, a tint (r, g, b) and a shade count *N* into *N* rows of 256 16-bit pixels (Formulas, "Shade table"):
   - Each tint channel is clamped to [0, 2000] ‰ and converted to a 16.16 factor.
   - With *N* ≥ 2, row *j* multiplies the tint by 2*j*/(*N* − 1). Row 0 is black, the middle row is roughly the tint itself, and the last row is twice the tint.
   - Tint-exempt indices (S-0103) ignore the tint. They darken linearly over the first few rows and are at full palette colour from then on.
   - Entry 0 of every row is transparent.
   - Every 16.16 factor *F* that scales a palette component, the tint factors and the tint-exempt brightness factor alike, takes effect as ⌊*F*/16⌋·16: it is rounded down to a multiple of 16 (its low 4 bits are dropped).
   - Each resulting 8-bit channel is capped at 255 and packed into the display's 16-bit format.
   - With *N* = 1, the single row is the palette times the tint.
   - **Untinted table** (the anim palette): *N* = 53 rows, row *j* is the palette times 2*j*/(*N* − 1) exactly, with no tint, no tint-exempt indices and no dropped low bits, so the middle row reproduces the palette exactly.
   - Backing: `measurable` (Scenario L5; the untinted table: Scenario L5 "anim palette")
   - Confidence: high.
   - **Host-dependent in the original; a departure here.** The original's shading result depends on the host machine. With a pixel layout of 5 bits per channel (15-bit) or a 16-bit layout that gives its channels 5, 6 and 5 bits in any arrangement (5-6-5, 5-5-6 or 6-5-5), on a processor that has the 64-bit multimedia integer instructions (every x86 processor in use today), it is the result with the dropped low bits described above. On an older processor without them, or with any other pixel layout, every factor is used in full instead: the bullet on dropped low bits does not apply (Formulas, "Shade table", the full-factor form). **Our engine** always gives the dropped-low-bits result, whatever the host and output format, and has no switch for the full-factor form (glossary, *departure*). Scenario L5 checks the original's result on a current processor, which is the one our engine fixes.

7. **Neutral light is very slightly dark.** The conversion of a tint channel to a 16.16 factor truncates a product that falls just below the exact value. So 1000 ‰ becomes 65535, not 65536 (see Formulas). Under neutral white light every non-zero colour channel comes out one less than its palette value, before 16-bit packing (this holds with and without the dropped low bits of claim 6). Tint-exempt indices at full brightness are not affected. The untinted anim-palette table is not affected either.
   - Backing: `measurable` (Scenario L5)
   - Confidence: medium. The result needs floating-point rounding toward zero at that point, which is the original's round-toward-zero mode (glossary; S-0003, claim 17), whose switch `quirks.chop_rounding` belongs to S-0205 (number domains, planned). It follows that switch: with `quirks.chop_rounding` off, the conversion gives the exact factor (Formulas, "Shade table"). Green and blue are certainly affected. Red is affected unless something outside the game changes the rounding mode, which is possible in principle.

8. **Tile shade tables are shared by tint, normalised and quantised.** The terms raw cell tint, normalised tint, quantised tint, quantised white and neutral tile table are defined in Terms.
   - **Normalisation.** At each full recompute, the cell's raw cell tint is clamped and *normalised* (Formulas, "Tint normalisation"). The normalised tint has its largest channel set to 1000, and the cell's light levels are scaled by the old largest channel / 1000 (the factor *K*).
   - **Quantisation.** The normalised tint is *quantised* by detail level: each channel is clamped to [0, 1000] and rounded down to a multiple of 128 (low), 64 (medium) or 32 (high). So 1000 becomes 896, 960 or 992.
   - **Table choice.** When a cell's table is chosen afresh, the test is made on the **normalised** tint, before quantisation:
     - A cell whose normalised tint is exactly (1000, 1000, 1000) uses the neutral tile table.
     - Every other cell uses the tile shade table of its quantised tint, which all cells with that quantised tint share.

     A table has 53 shades, or 27 if the three quantised channels sum to less than 2000. A normalised tint is white for every raw cell tint that is white already, for every near-black raw cell tint (below), and for every grey raw cell tint (*X*, *X*, *X*) whose *X* is not in the set 𝒟 (below, and Formulas). A cell's table is chosen afresh when the cell's lighting is first built, for every cell when the player changes the detail level, and at a light change whenever the kept-table rule below does not apply.
   - Backing: `measurable` (Scenarios L6, L18, L19)
   - Confidence: high
   - **Kept table at a light change.** A full recompute caused by a light-source change (claim 12) keeps the cell's current table if that table's own tint (the tint it was built for, or the map tint a trigger action last gave it, claim 17; not an effect tint it shows during a global effect) equals the cell's new quantised tint. A quantised tint is never white, so, trigger-action tint changes aside (claim 17), the neutral tile table is never kept this way, and in all but one case the kept table is the one a fresh choice would give. The exception is a cell on the quantised-white table whose new normalised tint is exactly white. It stays on the quantised-white table instead of moving to the neutral tile table, until its table is next chosen afresh. Example: a slightly off-white light is removed, and the cells it reached return to a white raw cell tint. `quirks.tile_table_kept_on_relight`.
     - Backing: `measurable` (Scenario L20)
     - Confidence: high
   - **Near-black tint.** A raw cell tint whose largest channel is 1 ‰ or less (after clamping) normalises to white (1000, 1000, 1000) with *K* = 65536 (a scale of 1). So when its table is chosen afresh, the cell uses the neutral tile table. Its scaled ground level becomes 0 at that full recompute. Its unscaled ground and scaled raised levels are not zeroed. The next incremental update restores the scaled ground level, because the stored *K* is then 65536 (Formulas, "Incremental update"). So a map whose tint is (0, 0, 0) is lit white, not black, and its ground looks exactly like ground under a white map tint once the scaled level is back.
     - Backing: `measurable` (Scenario L18)
     - Confidence: high
   - **Which channel is largest changes the rounding.** Write *X* for the largest channel of the clamped raw cell tint (among tied channels red counts as largest, then green, then blue) and *c* for one of the other two. The exact quotient is ⌊1000·*c*/*X*⌋. Which channel is largest decides how the original divides (Formulas, "Tint normalisation"):
     - **Red largest:** the channel comes out as the exact quotient, except that it is 1 lower when 1000·*c* is a multiple of *X* (and *c* > 0) and *X* is in the set 𝒟 of Formulas (`quirks.red_normalisation_divisor`). Of the 2 002 998 possible (*X*, *c*) pairs with 2 ≤ *X* ≤ 2000 and 0 ≤ *c* ≤ *X*, 1435 are 1 lower.
     - **Green or blue largest:** the channel is divided by the truncated scale *K*, so it comes out as the exact quotient or higher: 1 higher for 4920 pairs, 2 higher for 3 and 3 higher for 1 (`quirks.normalisation_truncated_scale`).
     - **Grey tints.** The largest visible effect is on grey raw cell tints. Since ties count for red, (*X*, *X*, *X*) with *X* in 𝒟 normalises to (1000, 999, 999), not to white. So it is quantised and uses a quantised table (quantised white) instead of the neutral tile table. Map tints alone can produce this: the grey map tints whose ‰ value is in 𝒟 are 10, 20, 40, 80, 110, 160, 190, 220, 320, 380, 440, 510, 630, 640, 760, 880, 890, 1010, 1020, 1260, 1270, 1280, 1510, 1520, 1530, 1760, 1770, 1780 and 1790 ‰. A grey map tint of 1.01 therefore gives quantised-white ground, while 1.00 and 1.03 give the neutral tile table.
     - **Other tints.** Apart from the neutral-table choice, these differences change the quantised tint only rarely. Compared with quantising the exact quotient, the red-largest result differs for 53 pairs at high detail and none at medium or low; the green-or-blue-largest result differs for 48 pairs at high detail, 32 at medium and 16 at low. The two rules applied to the same (*X*, *c*) pair give different quantised values for 101 pairs at high detail, 32 at medium and 16 at low, and the red-largest value is then always the lower one. No pair whose two values are both multiples of 10 ‰ is among any of these, so a map tint alone never shows them: a light-source tint is needed (Scenario L22).
     - Backing: `measurable` (Scenario L19 for grey tints, Scenario L22 for the green-or-blue-largest rule)
     - Confidence: high

9. **Cell light level.** Each cell has a ground light level, a raised light level and a raw cell tint (Formulas, "Cell light"):
   - The **ground** level is the current ambient (in ‰), plus the sum of all light-source intensities reaching the cell, plus the level factor × the cell's height level, minus the ground bias.
   - The **raised** level is the same with the height level increased by 4.
   - The raw cell tint is the map tint (in ‰) plus the light sources' tints.
   - Both levels are scaled by the tint-normalisation factor *K* of claim 8 and clamped to [0, 2000]. An unscaled copy of the ground level is also kept (Formulas).

   **Two kinds of update.** What a cell shows can depend on which kind of update last touched it. A *full* recompute evaluates all of the above. After every ambient step, at every lighting re-evaluation (claim 5), and after every trigger action that changes the lighting (the trigger specs), every cell gets an *incremental* update instead: its light-source sum, raw cell tint, normalised tint, normalisation factor and tile shade table stay as the last full recompute left them, and only the ambient and height terms are evaluated again. The two kinds differ in small ways (claim 10 and Formulas, "Incremental update").
   - Backing: `measurable` (Scenarios L7, L9, L17) for the ground levels and the raw cell tint, and (Scenario L23 Part C) for the raised level, which the overlays of deck cells read (claim 13).
   - Confidence: high; medium for the raised level, because only the deck overlays are known to show it

10. **Dominator height bug.** When a cell's lighting is fully recomputed while the psychic dominator is in force (any phase, the final one included) and no ion storm is in force, its **ground** level uses the *nuke flash* level factor with the dominator's ground bias. Its **raised** level correctly uses the dominator's level factor. Full recomputes happen when light sources around the cell change, when the cell's lighting is first built, and **for every cell on the map when the player changes the detail level**. The incremental update of claim 9 uses the dominator's level factor correctly. So which factor a cell shows depends on when it was last fully recomputed. With the engine defaults (nuke level factor 0.100 per level, dominator 0), affected high ground shows 100 ‰ brighter per height level until its next incremental update. While an ion storm is in force it outranks the dominator in the per-cell ranking (claim 2), so a full recompute then uses the ion storm's level factor and ground bias for both levels, and the bug does not show.
    - Backing: `measurable` (Scenario L8) for the ground level and the ion-storm exception, and (Scenario L8 Part A, the bridge deck) for the raised level's use of the dominator's level factor.
    - Confidence: high

11. **Light-source falloff.** A light source affects a cell when the horizontal distance *d* from the light's position to the cell centre is at most the radius *R*. Height is ignored. *d* is the Euclidean distance in leptons, truncated to an integer: the largest integer whose square is at most Δx² + Δy² (Formulas, "Cell light"). The falloff is *f* = ⌊1000·(*R* − *d*)/*R*⌋ ‰, so 1000 at the centre and 0 at the edge. The cell gains ⌊*I*·*f*/1000⌋ light level and ⌊*t*<sub>*c*</sub>·*f*/1000⌋ on each tint channel *c* (truncation toward zero, so negative lights darken symmetrically). Contributions from several lights add up.
    - A light source counts only when the detail level is at least its required level. For building lights this is 2 (high). **At low and medium detail, building lights contribute nothing.**
    - **Radius 0 (bad data; a departure).** A light source with radius 0 reaches a cell only if it sits exactly on the cell's centre (*d* = 0), and the falloff then divides by zero. In the original this ends the game with a crash at the first full recompute of that cell that counts the light. A building's light sits at the building's centre, which is a cell centre when the foundation's width and depth are both odd (claim 13), so a light-emitting building type with radius 0 and such a foundation crashes the original once its light is on at detail level 2. With an even width or depth the light reaches no cell. **Our engine:** a light source with radius 0 reaches no cell, wherever it sits, and the importer warns about every building type with a non-zero light intensity and a radius of 0 (glossary, *importer warning*). No switch reproduces the crash.
    - Backing: `measurable` (Scenario L7; Scenario L24 for radius 0)
    - Confidence: high for the formula; high that building lights require detail level 2; medium for the radius-0 crash

12. **Building light sources.**
    - A building type with a non-zero light intensity gets one light source at the building's centre. Its radius is the light-visibility value in leptons (default 5000), and its intensity and tints are as imported.
    - Intensity and tints are stored in thousandths: ⌊*v* × 1000 + 0.1⌋.
    - When a tint key is absent, the tint defaults to **1000.0**, not 1.0. As a stored value that is 1 000 000 ‰. Such a channel adds ⌊1 000 000·*f*/1000⌋ = 1000·*f* ‰ (claim 11), so in every cell with falloff *f* ≥ 2 ‰ it reaches the 2000 clamp, provided that channel of the map tint is not negative and no other light source subtracts from it. When all three tint keys are absent, such a cell's clamped raw cell tint is (2000, 2000, 2000). That is a grey tint whose value is not in 𝒟, so it normalises to white with *K* = 131072 (a scale of 2): the cell uses the neutral tile table and its scaled ground level is exactly twice its ground level, clamped to 2000 (claim 9). The unscaled level that vehicles, infantry and anim-palette animations use is not doubled.
    - **No delay.** When a light source is added, removed or changed, every cell it reaches is fully recomputed on that same tick, and all of them show their new values together from that tick on, however many cells and light sources the change involves. The original never spreads a relighting over several ticks and never limits it by a real-time budget, on any path that data, maps or play can reach, so nothing about it depends on real time or on the speed of the machine. Our engine's rule that a relighting completes within the tick it is requested is therefore the original's behaviour, not a departure. When a building's light is switched on or off is a matter of the building's own state (the structures specs); it need not be the tick the building is placed or destroyed.
    - **Re-reading truncates.** When a section is read again by any later rules layer (a game-mode file or a map), an absent key's default is the value already held, integer-divided by 1000 and so truncated to a whole unit. A later rules layer that restates part of a light-emitting building type without its light keys therefore turns an intensity of 0.5 into 0 (no light) and a tint of 0.75 into 0. The same applies to the extra-light values of claim 13 when a later rules layer has its own audio-visual rules section. A value that is already a whole number (such as the 1000.0 tint default) survives unchanged. This is an exception to the glossary's rule that a later rules layer changes only what it mentions, and it is not a *reset on override* (glossary): the value is not reset to a fixed default but truncated.
    - Backing: `measurable` (Scenarios L7, L9, L11)
    - Confidence: medium for the tint default's visible effect, which depends on which ground level each drawn thing reads (claim 13); high that every affected cell changes on the tick of the light change; medium for re-read truncation (how each later rules layer reaches a building type's section is the subject of S-0006)

13. **Object brightness.** An object's brightness *b* (‰) is built from its cell's light levels. In this claim *U* is the cell's unscaled ground level, *h* is the object's height above the surface in leptons, and λ is the level factor of the set chosen by the per-cell ranking of claim 2 unless stated otherwise. "*h* ÷ 208" is integer division truncated toward zero, so the height term grows by one level factor for every 2 height levels (208 leptons, 2 × the 104 leptons of a height level).
    - **Vehicles** (every vehicle kind, naval and hovering ones included): *b* = *U* + the rules' extra unit light, plus 4λ on a bridge deck. A vehicle in a shadowed cell, not on a bridge deck and less than 2 height levels above the surface, gets −500 added instead: *b* = *U* − 500 + extra unit light.
    - **Infantry:** *b* = *U* + the rules' extra infantry light, with one of these terms, whose cases exclude each other (as in Formulas, "Object brightness"):
      - on a bridge deck: add (*h* ÷ 208) × λ. Here *h* is measured from the deck, so infantry standing on a deck get nothing extra, unlike vehicles.
      - not on a bridge deck, in a shadowed cell, more than 208 leptons above the surface: add (*h* ÷ 208) × λ.
      - not on a bridge deck, in a shadowed cell, at most 208 leptons above the surface: add −500.

      Anywhere else infantry ignore their height: infantry falling or flying above an ordinary cell are drawn as if on the ground.
    - **Aircraft:** *b* = *U* + the rules' extra aircraft light + (*h* ÷ 208) × λ<sub>air</sub>. λ<sub>air</sub> is the ion storm's level factor while a storm is in force and the normal level factor otherwise, never the nuke or dominator factor.
    - **Buildings.** Write *T* for the building's top-left foundation cell (glossary, foundation). The building's light comes from *T* alone, whatever the light on its other foundation cells (`quirks.building_light_from_top_left_cell`). No extra-light value and no height term is added.
      - An ordinary building: *b* = *U* of *T*, for every part of it: the body painted in the cell pass, the parts drawn as sprites in the object pass, and the parts drawn as voxel models (turrets and the like).
      - A terrain-palette building: its body and its parts drawn as sprites take the **scaled** ground level of the cell under the building's centre, the cell whose tile shade table they are drawn with (S-0103, claim 8). With an even foundation width or depth the centre lies on a cell edge, and the cell with the larger coordinate counts. Its parts drawn as voxel models still take *U* of *T*.
    - **Terrain objects.** A plain terrain object, such as a tree: its cell's **scaled** ground level, drawn with its cell's tile shade table, so it takes the cell's tint as terrain does (claim 3). An ore-spawning terrain object: its cell's **unscaled** ground level *U*, drawn with a palette that is not its cell's tile shade table; which palette, and whether it takes any tint, is not covered here (Open questions).
    - **Overlays:**
      - Ore and gem overlays: always 1000, whatever the cell's light.
      - Wall overlays: the cell's **unscaled** ground level *U*.
      - The overlays of a deck cell, which draw the bridge deck: the cell's **scaled raised** level (claim 9), whatever their type.
      - Every other overlay: the cell's **scaled** ground level.
    - **Animations** (palette choice in the table of S-0103, claim 8): vein animations (row 1) and terrain animations, which are created as part of a cell's terrain (row 2), take the cell's **scaled** ground level. Anim-palette animations (row 5) take the **unscaled** ground level *U*. Alternative-palette animations (row 4) are drawn with a single-shade table, so brightness has no effect on them. An object's own animation that is drawn with its object (row 3) takes a brightness that is stored with the animation:
      - A building's own animation: the building's brightness, *U* of the building's top-left foundation cell (as for an ordinary building, even when the building uses the terrain palette). It is set when the animation starts and set again on every tick, so it follows the building's light.
      - A parachute: *U* of the carried object's cell on the tick the parachute is created. It is not updated during the fall.
      - A unit's deploy or undeploy animation: 1000, whatever the cell's light (`quirks.deploy_animation_unlit`).
      - Any other animation of this row: not established (Open questions).

      An animation flagged to use normal light is drawn at 1000 in every case, row 3 included.
    - **Not covered here:** changes to an object's brightness while it flashes, while it is protected by the invulnerability superweapon, or while it is the target of an air strike. These apply on top of the values above (Open questions).
    - The extra-light values are stored as ⌊*v* × 1000⌋ ‰ with no rounding allowance, where *v* is the single-precision decimal of S-0003, so a value such as 0.57 becomes 569.
    - All additions are integer additions. *b* is not clamped here; the row mapping of claim 15 treats any *b* ≤ 0 as 0.
    - Backing: `measurable` (Scenarios L9 for the animations, L10, L14, L15 for vehicles and infantry in shadowed cells, L23 for buildings, terrain objects, wall and other overlays, deck overlays and the animations of row 3, L3 Setup A for the tint of plain terrain objects)
    - Confidence: high for vehicles on and off bridge decks, for the −500 being added (not replacing the brightness) and for ore and gems; high for the 208-lepton height step and the infantry and aircraft rules; high for an ordinary building's top-left cell; medium for the animation rules, terrain-palette buildings, terrain objects, wall, deck and other overlays; medium for which cells count as shadowed (Terms)

14. **Trigger-action ambient changes.** The effect lighting sets are read once with the map. Trigger actions that change the base ambient set the target as well, unless an ion storm is in force, in which case only the base changes. Fading then proceeds as in claim 4.
    - Backing: `measurable` (Scenario L13)
    - Confidence: medium (trigger actions are covered by the trigger specs)

15. **Brightness to shade row.** An object drawn at brightness *b* (‰) with a shade table of *N* rows uses, per pixel, row ⌊*a*·*ρ*·(*N* − 1)/32258⌋, capped at *N* − 1.
    - *ρ* = min(254, ⌊max(*b*, 0)·261/2048⌋).
    - *a* is the pixel's light-alpha value, 127 unless a light-glow image covers it.

    So *b* = 1000 gives row 26 of 53 (neutral), *b* = 2000 gives row 52, and *b* ≤ 0 gives row 0, which is entirely black (tint-exempt indices included).
    - Backing: `measurable` (Scenario L5)
    - Confidence: high for the mapping; medium that all object drawing goes through it

16. **Dominator height terms linger.** The dominator stays in force through its final phase, and its being over causes no update of the cells (claim 5). Its final phase normally ends with the fade step that brings the current ambient to the target, and the incremental update of that step still uses the dominator's level factor and ground bias. If the ambient is already at the target when the final phase starts, the last update is the one at the re-evaluation that starts the final phase, which also still uses them. So after a dominator, every cell keeps the dominator's height terms until its next incremental update or full recompute: the next ambient step, the next lighting re-evaluation, a trigger action that changes the lighting, a light change around it, or a detail-level change. With the absent-key defaults (dominator level factor and ground bias 0, normal 8 ‰ per level and 50 ‰), flat ground stays 50 ‰ brighter than normal and high ground loses its height brightening until then. `quirks.dominator_height_terms_linger`.
    - If the rules' ambient change rate is 0 (claim 4) and the current ambient differs from the target when the final phase starts, the dominator is never over: its height terms, its fade interval and its tint for newly built tables stay in force for the rest of the game.
    - Backing: `measurable` (Scenario L16; Part C for the zero-rate case)
    - Confidence: high for the order of the last step and the end of the dominator; medium for the zero-rate consequence

17. **Trigger-action tint changes.** A trigger action that sets one channel of the map tint (the trigger specs) changes that channel of the map tint to the action's value, in hundredths. On the same tick it retints every tintable table (claim 3) to the whole new map tint, in ‰ (hundredths × 10). That covers every tile shade table, the neutral tile table included, the 53-shade house-colour palettes and the named palettes. The new map tint becomes each of these tables' own tint, and it replaces any effect tint they were showing. Every cell then gets an incremental update (claim 9), which keeps its table, raw cell tint and normalised tint.
    - So from the action on, units, infantry, aircraft and buildings take the map tint as well, unlike claim 3's normal case. All ground shows the map tint uniformly, without the colour of light sources and without quantisation, until the cell's table is chosen again (claim 8).
    - A later lighting re-evaluation that chooses an effect shows the effect tint as usual. One that chooses "normal" returns every table to its own tint, which is now the map tint.
    - The action does not change the target ambient, the base ambient or any cell's brightness.
    - Which table a cell gets when its table is chosen again after such an action, before the next detail change, is open (Open questions). After a detail change, every table is built afresh and claim 8 applies with the new map tint in the raw cell tints.
    - `quirks.trigger_tint_retints_all_tables`.
    - Backing: `measurable` (Scenario L21)
    - Confidence: high for the retint of every tintable table and the change of their own tint; medium for the action's value being in hundredths, which Scenario L21 checks only for one value. How a map states the action belongs to the trigger specs.

## Formulas

Integer arithmetic truncating toward zero unless marked. "÷" is integer division, truncated toward zero. ⌊ ⌋ also means truncation toward zero, not rounding toward minus infinity, wherever a passage does not say otherwise.

**Precision of decimal conversions.** Every decimal read from the rules or the map is first the single-precision value of S-0003, claim 6, widened to double. Every conversion of such a value to an integer in this spec (map lighting values, the ambient change step and rate, the nuke and dominator rates, the extra-light values, the building light values) is computed in double precision. Each floating-point multiplication and addition rounds toward zero, under the original's round-toward-zero mode (glossary; S-0003, claim 17), whose switch `quirks.chop_rounding` belongs to S-0205 (number domains, planned), and the final conversion to an integer truncates toward zero. The 16.16 factors of the shade tables and the normalisation scale are computed the same way (below). The light distance is the only other floating-point step, and its integer form is exact (Cell light).

**Map value storage** (claim 1): *x*₁₀₀ = ⌊*v*·100 + 0.01⌋, *x*₁₀₀₀ = ⌊*v*·1000 + 0.01⌋. Double precision. In the original, rounding is toward zero in every floating-point step. Per-mille from hundredths: ‰ = *x*₁₀₀ · 10.

**Ambient fade** (claim 4): step = ⌊*s*·100⌋ hundredths, where *s* is the rules' change step. Fade interval *P* in ticks:

| Condition | *P* |
|---|---|
| nuke flash (either phase, or tick 47 of claim 5, the first tick after the fading phase) or chrono flash | nuke rate, stored as ⌊*v*⌋ |
| not the above, and the dominator in force (any phase, the final one included) | dominator rate, stored as ⌊*v*·1000 + 0.01⌋ and **used directly as ticks** |
| neither of the above | ⌊*m*·900⌋, *m* = rules' change rate in game minutes |

With *A*ₜ = max(target ambient, 0), a step takes the current ambient *A* to *A*′ = *A*ₜ if |*A*ₜ − *A*| ≤ step, otherwise *A*′ = *A* + step · sgn(*A*ₜ − *A*). All values in hundredths, integer arithmetic.

**Shade table** (claims 6–7). For each channel *c* of tint (*r*, *g*, *b*), clamped to [0, 2000]:

> *M*<sub>*c*</sub> = ⌊*t*<sub>*c*</sub>·1000·κ⌋, κ = the double nearest to 0.065536 (slightly below it), computed in double precision rounding toward zero (the round-toward-zero mode, Precision of decimal conversions). For every integer *t* in 0 … 2000 this equals the integer form
>
> *M*(*t*) = ⌊8192·*t* / 125⌋ − ε(*t*), where ε(*t*) = 1 when *t* is a positive multiple of 125, and ε(*t*) = 0 otherwise
>
> so *M*(1000) = 65535, *M*(500) = 32767, *M*(2000) = 131071, *M*(999) = 65470. (Under round-to-nearest the correction term ε vanishes: *M*(*t*) = ⌊8192·*t* / 125⌋ for every *t*. So with `quirks.chop_rounding` off (S-0205, number domains, planned), 1000 ‰ maps to exactly 65536, 500 ‰ to 32768 and 2000 ‰ to 131072, neutral light reproduces the palette exactly (claim 7), and the expected values of the scenarios change accordingly, for example 248 → 124 instead of 123 in Scenario L21.)

With *L* = min(max(⌊30*N*/200⌋ − 1, 0), ⌊(*N* − 1)/2⌋) (*L* = 6 for *N* = 53, *L* = 3 for *N* = 27), for row *j* = 0 … *N* − 1 (*N* ≥ 2):

> *T*<sub>*c*</sub>(*j*) = ⌊2*j*·*M*<sub>*c*</sub> / (*N* − 1)⌋
>
> *W*(*j*) = ⌊65536·*j*/*L*⌋ for *j* ≤ *L*, and *W*(*j*) = 65536 for *j* > *L* (the tint-exempt brightness factor)

For *N* = 1: *T*<sub>*c*</sub> = *M*<sub>*c*</sub>, *W* = 65536. For palette entry *i* ≥ 1 with components *u*<sub>*c*</sub>, the output channel *o*<sub>*c*</sub> of row *j* is:

> *o*<sub>*c*</sub> = min(255, ⌊*u*<sub>*c*</sub>·16·⌊*T*<sub>*c*</sub>(*j*) / 16⌋ / 65536⌋) for normal indices; min(255, ⌊*u*<sub>*c*</sub>·16·⌊*W*(*j*) / 16⌋ / 65536⌋) for tint-exempt indices.

(The original's other, host-dependent result, the *full-factor form* of claim 6, omits the inner ⌊· / 16⌋·16: *o*<sub>*c*</sub> = min(255, ⌊*u*<sub>*c*</sub>·*T*<sub>*c*</sub>(*j*) / 65536⌋), and likewise with *W*(*j*) for tint-exempt indices. Our engine never uses it (claim 6, departure). The two differ by at most 1 in an 8-bit channel, and only rarely after 16-bit packing: under neutral tint a single (row, component) pair in 53 × 256 changes a 5-bit channel, row 11 with component 227.)

**Untinted table** (anim palette, claim 6): *o*<sub>*c*</sub> = min(255, ⌊*u*<sub>*c*</sub>·⌊2*j*·65536 / (*N* − 1)⌋ / 65536⌋), *N* = 53. Entry 0 is transparent here too, so its value never shows.

Entry 0 is transparent. The three outputs are packed by keeping each channel's top bits (5-6-5 for the usual 16-bit format). (*N* from 2 to 13 would give *L* = 0, for which *W* is undefined. Such counts never occur: only 1, 27 and 53 are used.)

**Tint normalisation** (claim 8). Each channel of the raw cell tint is first clamped to [0, 2000]. *X* is the largest channel; among tied channels red counts as the largest, then green, then blue. The scale *K* (a 16.16 factor) and the normalised tint are:

| Clamped raw cell tint | *K* | Normalised tint |
|---|---|---|
| (1000, 1000, 1000) | 65536 | (1000, 1000, 1000) |
| *X* ≤ 1 (near-black) | 65536 | (1000, 1000, 1000), and the cell's **scaled** ground level (below) is 0; the unscaled ground level and the raised level are unaffected (`quirks.near_black_tint_zeroes_scaled_ground`) |
| *X* ≥ 2, green or blue largest | ⌊8192·*X* / 125⌋ | largest channel 1000; each other channel *c* becomes ⌊*c*·65536 / *K*⌋ (`quirks.normalisation_truncated_scale`). The original divides in floating point, and the result always equals this integer division by the integer *K*. |
| *X* ≥ 2, red largest | ⌊8192·*X* / 125⌋ | red 1000; each other channel *c* becomes *n*(*c*, *X*) below (`quirks.red_normalisation_divisor`) |

⌊8192·*X* / 125⌋ is exactly ⌊*X*·65.536⌋. The original computes *K* in double precision, and the result equals this integer form for every *X* in 0 … 2000.

For red-largest tints, with *c* from 0 to *X* and *X* from 2 to 2000:

> *n*(*c*, *X*) = ⌊1000·*c* / *X*⌋ − 1 if *c* > 0, 1000·*c* is a multiple of *X*, and *X* ∈ 𝒟; otherwise *n*(*c*, *X*) = ⌊1000·*c* / *X*⌋.

This integer form is exact. It comes from floating point in the original: the divisor there is not *K* but *D*(*X*) = *X*·65536·δ, where δ is the double nearest to 0.001 (slightly above it), and the channel is ⌊*c*·65536 / *D*(*X*)⌋. The product and the quotient are each rounded toward zero in double precision before the final truncation (the round-toward-zero mode, Precision of decimal conversions). 𝒟 is the set of *X* for which *D*(*X*) > *X*·65.536. Of the 1999 values of *X* from 2 to 2000, 284 are in 𝒟 (the smallest are 2 to 8, 10, 12, 14, 16, 20, 24, 28, 32, 40 and 47); *D*(*X*) is below *X*·65.536 for 1699 values and equal to it for 16.

So an exact quotient can come out one lower: *X* = 2, *c* = 1 gives 499, and a second channel equal to the largest gives 999 instead of 1000 whenever *X* ∈ 𝒟 (e.g. *X* = 2 … 8). In particular a grey tint (*X*, *X*, *X*) with *X* ≥ 2 normalises to white exactly when *X* ∉ 𝒟. An implementation can decide membership of 𝒟 by computing *D*(*X*) in double precision in the rounding mode in force, or from a 2001-entry table. With `quirks.red_normalisation_divisor` off, *n*(*c*, *X*) = ⌊1000·*c* / *X*⌋ for every *c*, so every grey tint normalises to white. That switch changes only the red-largest rule. 𝒟 and the counts above (284 values of *X*, 1435 pairs) are the original's, under the round-toward-zero mode. *D*(*X*) and the quotient are rounded in whatever mode is in force: with `quirks.chop_rounding` off (S-0205, number domains, planned) they round to nearest, which changes the set of affected (*X*, *c*) pairs (it is not the 𝒟 given here). With `quirks.red_normalisation_divisor` off the result is the exact quotient in either mode.

For green- or blue-largest tints the divisor *K* is truncated, so *K* ≤ *X*·65.536 and ⌊*c*·65536 / *K*⌋ is never below the exact quotient ⌊1000·*c* / *X*⌋; it is above it for 4924 of the 2 002 998 (*X*, *c*) pairs (by 1 for 4920, by 2 for 3, by 3 for 1). A channel that ties with the largest (only blue can tie with a largest green) gives ⌊*X*·65536 / *K*⌋, which is above 1000 for *X* = 3, 5, 7, 9, 11 and 13 (1003, 1002, 1001, 1001, 1001, 1001); quantisation clamps these to 1000. With `quirks.normalisation_truncated_scale` off, every channel other than the largest becomes the exact quotient ⌊1000·*c* / *X*⌋ when green or blue is largest.

**Quantisation** (claim 8): for each channel *c* of the normalised tint, *c*′ = *m* − (*m* mod *Q*), where *m* = min(max(*c*, 0), 1000) and *Q* = 128, 64, 32 for detail levels 0, 1, 2. Integer arithmetic, exact.

**Tile table choice** (claim 8). Chosen afresh: the neutral tile table if the normalised tint is (1000, 1000, 1000), otherwise the table of the quantised tint, with 53 rows if *r*′ + *g*′ + *b*′ ≥ 2000 and 27 otherwise. Kept at a light change: the cell's current table if its own tint equals the new quantised tint, otherwise chosen afresh.

**Cell light** (claims 9–11). With current ambient *A* (hundredths), the set's level factor λ and ground bias *G* (thousandths, i.e. ‰ per level and ‰), cell height level *z* (in height levels), and light sources *k*:

> *d*<sub>*k*</sub> = ⌊√(Δx² + Δy²)⌋ leptons (cell centre = cell coordinate × 256 + 128); light *k* reaches the cell when *d*<sub>*k*</sub> ≤ *R*<sub>*k*</sub>. The original takes the square root in double precision and truncates it, and the result always equals the integer square root: the largest integer *d* with *d*² ≤ Δx² + Δy².
>
> *f*<sub>*k*</sub> = ⌊1000·(*R*<sub>*k*</sub> − *d*<sub>*k*</sub>) / *R*<sub>*k*</sub>⌋
>
> *S* = Σ ⌊*I*<sub>*k*</sub>·*f*<sub>*k*</sub> / 1000⌋, raw tint<sub>*c*</sub> = 10·(map tint<sub>*c*</sub>) + Σ ⌊*t*<sub>*k*,*c*</sub>·*f*<sub>*k*</sub> / 1000⌋ (sums over the lights that reach the cell; map tint in hundredths, so the raw cell tint is in ‰)
>
> ground = *A*·10 + *S* + λ·*z* − *G*
>
> raised = *A*·10 + *S* + λ·(*z* + 4) − *G*

*K* and the normalised tint come from the raw cell tint (Tint normalisation), and the tile table from the normalised tint (Tile table choice). A full recompute gives each cell three light values, each finally clamped to [0, 2000]:

- *unscaled ground* = ground (used by vehicles, infantry, aircraft, ordinary buildings, the voxel parts of terrain-palette buildings, ore-spawning terrain objects, wall overlays, anim-palette animations and the own animations of claim 13),
- *scaled ground* = ⌊min(ground, 2000)·*K* / 65536⌋ (used by vein animations, by animations created as part of a cell's terrain, by plain terrain objects, by the body and sprite parts of terrain-palette buildings and by overlays other than walls, ore, gems and deck overlays, claim 13),
- *scaled raised* = ⌊raised·*K* / 65536⌋ (no clamp before scaling; used by the overlays of deck cells, claim 13).

For negative values the rounding direction of these three divisions never shows: the final clamp turns any negative result into 0.

**Incremental update** (claim 9): ground and raised are re-evaluated from the stored light-source sum *S* with the current ambient and the set's λ and *G*; the stored *K*, raw cell tint, normalised tint and tile table are kept. Differences from the full recompute:
- the scaled ground is ⌊ground·*K* / 65536⌋ from the **unclamped** ground, so it differs from the full recompute when ground > 2000 and *K* < 65536.
- the zeroing of the scaled ground for a near-black raw cell tint (*X* ≤ 1 above) is not repeated, because the stored *K* is then 65536: the scaled ground equals the ground again after the next incremental update.
- the dominator's level factor is used correctly (claim 10).

Which of the ground values the terrain tile itself is drawn with is open (Open questions); Scenario L9's secondary observable decides it.

**Object brightness** (claim 13), all integer, with *U* the unscaled ground level and *h* the height above the surface in leptons:

| Object | *b* (‰) |
|---|---|
| vehicle on a bridge deck | *U* + 4λ + *e*<sub>unit</sub> |
| vehicle not on a bridge deck, in a shadowed cell, with *h* < 208 | *U* − 500 + *e*<sub>unit</sub> |
| any other vehicle | *U* + *e*<sub>unit</sub> |
| infantry on a bridge deck | *U* + (*h* ÷ 208)·λ + *e*<sub>inf</sub> |
| infantry not on a bridge deck, in a shadowed cell, with *h* > 208 | *U* + (*h* ÷ 208)·λ + *e*<sub>inf</sub> |
| infantry not on a bridge deck, in a shadowed cell, with *h* ≤ 208 | *U* − 500 + *e*<sub>inf</sub> |
| any other infantry | *U* + *e*<sub>inf</sub> |
| aircraft | *U* + (*h* ÷ 208)·λ<sub>air</sub> + *e*<sub>air</sub> |
| ordinary building, every part | *U* of its top-left foundation cell |
| terrain-palette building, body and sprite parts | scaled ground of the cell under its centre |
| terrain-palette building, voxel parts | *U* of its top-left foundation cell |
| plain terrain object | scaled ground of its cell |
| ore-spawning terrain object | *U* of its cell |
| ore and gem overlays | 1000 |
| wall overlay | *U* of its cell |
| overlay in a deck cell | scaled raised level of its cell |
| any other overlay | scaled ground of its cell |
| a building's own animation drawn with it | *U* of the building's top-left foundation cell, every tick |
| parachute | *U* of the carried object's cell when the parachute is created |
| deploy or undeploy animation | 1000 |

For each object kind the rows exclude each other. Where a row names no cell, *U* and the scaled ground level are those of the object's own cell. *e*<sub>unit</sub>, *e*<sub>inf</sub> and *e*<sub>air</sub> are the stored extra-light values, ⌊*v*·1000⌋ ‰. 208 leptons is 2 height levels.

**Brightness to row** (claim 15): *ρ* = min(254, ⌊max(*b*, 0)·261 / 2048⌋), row = min(*N* − 1, ⌊*a*·*ρ*·(*N* − 1) / 32258⌋).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| neutral | 1000 | ‰ | |
| light level range | 0 … 2000 | ‰ | |
| fade game minutes to ticks | 900 | ticks per game minute | |
| default ambient change rate | 0.2 | game minutes | 180 ticks |
| default ambient change step | 0.10 | ambient factor | 10 hundredths |
| default ambient (normal) | 1.00 | factor | stored 100 hundredths |
| default tint (normal) | 1.00, 1.00, 1.00 | factor | |
| default ground bias (normal) | 0.050 | factor | subtracted once, not per level (claim 9); stored 50 thousandths = 50 ‰ |
| default level factor (normal) | 0.008 | per height level | 8 ‰ per level |
| default ion ambient / tint | 0.87 / 0.30, 0.40, 0.75 | factor | engine default for absent keys |
| default ion ground / level | 0 / 0 | thousandths | |
| nuke flash ambient / tint | 2.00 / 1.75, 1.50, 1.25 | factor | fixed |
| nuke flash ground / level | 0.100 / 0.100 | factor / factor per height level | fixed; stored 100 / 100 thousandths (100 ‰, 100 ‰ per level) |
| default nuke fade interval | 1 | tick | |
| default dominator ambient / tint | 1.50 / 0.85, 0.20, 0.30 | factor | |
| default dominator ground / level | 0 / 0 | thousandths | |
| default dominator fade interval | 1 | tick | stored value used as ticks |
| nuke flash phase lengths | 31, then 16 | ticks | brightening (ticks 0–30), fading (ticks 31–46), claim 5 |
| building light radius default | 5000 | leptons | |
| building light intensity default | 0 | ‰ | 0 = no light |
| building light tint default | 1000.0 | factor | 1 000 000 ‰ |
| building light required detail | 2 | detail level | |
| extra unit/infantry/aircraft light default | 0 | ‰ | |
| shadowed-cell penalty | −500 | ‰ | added to the brightness of vehicles and infantry (claim 13) |
| height-term step | 208 | leptons | 2 height levels of 104 leptons (claim 13) |
| bridge deck height | 4 | height levels | 416 leptons; vehicles on a deck add 4 × the level factor |
| raised step | 4 | height levels | |
| tint quantum | 128 / 64 / 32 | ‰ | detail 0 / 1 / 2 |
| tile shade counts | 53 (27 if quantised tint sum < 2000) | rows | |
| default light-alpha | 127 | dimensionless | |
| row mapping | 261/2048, cap 254, divisor 32258 | dimensionless | claim 15 |
| near-black threshold | 1 | ‰ | a clamped raw cell tint whose largest channel is at most this normalises to white (claim 8) |
| red-largest divisor factor δ | the double nearest to 0.001 (slightly above it) | dimensionless | Formulas, "Tint normalisation" |
| shading factor granularity | 16 | 16.16 units | factors are rounded down to a multiple of this before use (claim 6) |
| anim palette rows | 53 | rows | untinted table |

## State and ordering

| State | Event | Result |
|---|---|---|
| any | nuclear missile detonates (tick 0) | lighting re-evaluation, which chooses the nuke set: target = nuke ambient, all tintable tables take the nuke tint on this tick; first fade step on tick 1; brightening phase |
| nuke brightening | tick 31 | fading phase (16 ticks); lighting re-evaluation (claim 5 chooses the set); fade interval still the nuke rate |
| nuke fading | tick 47 | nuke flash over; no re-evaluation, target and tints unchanged; from tick 48 on the fade interval no longer uses the nuke rate |
| any | ion storm starts, or ion storm ends | lighting re-evaluation |
| any | dominator launched | dominator in an active phase; lighting re-evaluation; first fade step on the next tick |
| dominator active phase | animation reaches its last image frame | final phase; lighting re-evaluation (the dominator no longer counts for it) |
| dominator final phase | current ambient = target ambient | dominator over; no re-evaluation and no cell update (claim 16) |
| any | chrono flash starts or finishes | no re-evaluation; target and tints unchanged; fade interval and new-table tint follow it while it is in force |
| current ≠ target | fade interval elapsed | current moves one step towards target; interval restarts with the rate chosen at that moment; every cell gets an incremental update |
| rules change rate = 0 | any | current never moves (quirk) |
| any | trigger action changes the map tint | on this tick every tintable table takes the new map tint as its own tint, replacing any effect tint; every cell gets an incremental update; target ambient unchanged (claim 17) |
| any | light source added, removed or changed | every affected cell fully recomputed on this tick; all of them change together (claim 12) |

**Ordering consequence.** Tints change on the tick of a lighting re-evaluation. Brightness follows over the fade, so a nuke flash shows the warm tint immediately and the brightening gradually.

## Edge cases and quirks

- **Dominator height bug** (claim 10). `quirks.dominator_ground_uses_nuke_level` (default on). Off: use the dominator's level factor in both levels.
- **Zero change rate freezes ambient** (claim 4). If the rules' ambient change rate is 0, the current ambient never moves, even for nuke and dominator fades (and a dominator in its final phase may then never be over, claim 16). `quirks.zero_rate_freezes_ambient` (default on). Off: jump straight to the target.
- **Dominator fade rate scale.** The dominator rate is stored ×1000 but used as ticks (claim 4). A map value of 1 means 1000 ticks, and the default is 1 tick. `quirks.dominator_rate_thousandths` (default on). Off: the map value *v* is a number of ticks, truncated to whole ticks as the nuke rate is (⌊*v*⌋), so a map value of 1 means 1 tick; the absent-key default stays 1 tick.
- **Nuke fade rate truncated** to whole ticks. An edge case, not a quirk: no switch.
- **Three different effect priorities** (claim 2): the tint/target order, the height-term order and the new-table order. The dominator counts in the first only in its active phases, and in the other two in every phase. `quirks.split_effect_priority` (default on). Off: the tint/target order, with the dominator counting only in its active phases, applies everywhere.
- **Neutral light darkens by one** (claim 7). This spec has no switch of its own for it: it is a result of the round-toward-zero mode, and follows `quirks.chop_rounding` (S-0205, number domains, planned). With that switch off, *M*(*t*) = ⌊8192·*t* / 125⌋ for every tint value (Formulas, "Shade table"): every positive multiple of 125 ‰ gives one more than in the original, and neutral light reproduces the palette exactly.
- **Detail level hides building lights and coarsens tints** (claims 8, 11). This is original behaviour, not a bug. An implementation may expose it as a setting.
- **Untinted building lights default to tint 1000.0** (claim 12). `quirks.building_light_tint_default_1000` (default on). Off: default 1.0.
- **Aircraft ignore nuke and dominator level factors** (claim 13). `quirks.aircraft_height_light_ion_only` (default on). Off: aircraft use the level factor chosen by the per-cell ranking of claim 2 (ion storm, then dominator in any phase, then nuke flash brightening phase, then normal), as vehicles on bridge decks and infantry do.
- **A building is lit from its top-left cell** (claim 13). An ordinary building, every part of it and its own animations take the light of its top-left foundation cell only, so a light that reaches only another part of the foundation does not brighten the building. `quirks.building_light_from_top_left_cell` (default on). Off: they take the unscaled ground level of the cell under the building's centre, the cell a terrain-palette building already uses.
- **Deploy and undeploy animations ignore the light** (claim 13). They are drawn at 1000 whatever the cell's light, while the unit beside them follows it. `quirks.deploy_animation_unlit` (default on). Off: they take the unscaled ground level of the unit's cell, as a parachute does.
- **Infantry height light only on bridges and in shadowed cells** (claim 13). Infantry above an ordinary cell ignore their height, and infantry standing on a bridge deck get no deck brightening, unlike vehicles. `quirks.infantry_height_light_limited` (default on). Off: infantry add (*h* ÷ 208) × λ wherever they are, with *h* their height above the terrain (so a deck counts 2λ), and the shadowed-cell penalty still applies at *h* ≤ 208.
- **Extra-light values lose a unit when truncated** (claim 13). `quirks.extra_light_truncation` (default on). Off: round to nearest.
- **Near-black tint turns white and drops the scaled level** (claim 8, near-black threshold). A raw cell tint whose largest channel is 1 ‰ or less normalises to white, and the cell's scaled ground level becomes 0 at each full recompute. The unscaled ground level (vehicles, infantry, anim-palette animations) and the raised level keep their values, and the scaled ground level comes back at the next incremental update (Formulas). `quirks.near_black_tint_zeroes_scaled_ground` (default on). Off: the normalised tint is still white with scale factor 1, and the scaled ground level equals the clamped ground level, as after an incremental update.
- **Red-largest tints normalise one lower** (claim 8). When red is the largest channel, the divisor is a slightly inexact double, so an exact quotient can come out 1 ‰ low (Formulas, "Tint normalisation"). Grey tints with *X* ∈ 𝒟 therefore miss the neutral tile table. `quirks.red_normalisation_divisor` (default on). Off: when red is the largest channel, every other channel becomes the exact quotient ⌊1000·*c* / *X*⌋, so every grey tint normalises to white and uses the neutral tile table. Green- and blue-largest tints are governed by the next switch.
- **Green- and blue-largest tints normalise high** (claim 8). When green or blue is the largest channel, the other channels are divided by the truncated scale *K*, so they can come out 1 to 3 ‰ above the exact quotient, and a blue that ties with green at a small odd value comes out above 1000 (Formulas, "Tint normalisation"). `quirks.normalisation_truncated_scale` (default on). Off: those channels become the exact quotient ⌊1000·*c* / *X*⌋, as for red-largest tints with `quirks.red_normalisation_divisor` off.
- **Chrono flash lighting persists** (claims 2, 5). The chrono flash ranks with the nuke flash brightening phase, but neither its start nor its finish re-evaluates the lighting. A re-evaluation while it is in force (an ion storm starting or ending, a dominator being launched or entering its final phase, the nuke flash entering its fading phase) applies the nuke set's ambient and tint, and they stay after the chrono flash finishes until the next re-evaluation. `quirks.chrono_flash_no_reevaluation` (default on). Off: the chrono flash's start and finish are lighting re-evaluations too, so it shows the nuke set exactly while it is in force.
- **Dominator height terms linger** (claim 16). After a dominator is over, cells keep its level factor and ground bias until their next update. `quirks.dominator_height_terms_linger` (default on). Off: the dominator's being over is followed by an incremental update of every cell with the normal (or ion) height terms.
- **Shading factors lose their low 4 bits** (claim 6). In the original this depends on the processor and the pixel layout. A departure, with no switch: our engine always drops the low 4 bits, which is what the original does on current processors with the standard 16-bit layouts.
- **Radius-0 light** (claim 11). The original crashes when a radius-0 light sits on a cell centre. A departure for bad data, with no switch: in our engine a radius-0 light reaches no cell, and the importer warns.
- **Full and incremental cell updates disagree slightly** (claim 9, Formulas). `quirks.incremental_relight_differs` (default on). Off: the incremental update gives exactly what a full recompute would (for the dominator, see `quirks.dominator_ground_uses_nuke_level`).
- **Re-reading a section truncates light values** (claim 12). `quirks.reread_truncates_light_values` (default on). Off: an absent key keeps the value already held, exactly.
- **Quantised-white table kept at a light change** (claim 8). A cell on the quantised-white table whose normalised tint becomes exactly white through a light change keeps the quantised-white table until its table is next chosen afresh. `quirks.tile_table_kept_on_relight` (default on). Off: at every light change the table is chosen afresh, so such a cell moves to the neutral tile table.
- **Trigger-action tint changes retint every table** (claim 17). A trigger action that changes the map tint gives the new map tint to every tintable table, house-colour and named palettes included, and makes it their own tint. `quirks.trigger_tint_retints_all_tables` (default on). Off: the action changes only the map tint. Every cell is then fully recomputed with it, and house-colour and named palettes stay white.

## Import mapping

| Source | Key | Parse | Feeds | Absent |
|---|---|---|---|---|
| map `[Lighting]` | `Ambient`, `Red`, `Green`, `Blue` | decimal → hundredths (claim 1) | normal ambient (base, current and target at start), normal tint | 1.00 each |
| map `[Lighting]` | `Ground`, `Level` | decimal → thousandths | normal ground bias, level factor | 0.050, 0.008 |
| map `[Lighting]` | `IonAmbient`, `IonRed`, `IonGreen`, `IonBlue` | hundredths | ion set | 0.87, 0.30, 0.40, 0.75 |
| map `[Lighting]` | `IonGround`, `IonLevel` | thousandths | ion set | 0, 0 |
| map `[Lighting]` | `DominatorAmbient`, `DominatorRed`, `DominatorGreen`, `DominatorBlue` | hundredths | dominator set | 1.50, 0.85, 0.20, 0.30 |
| map `[Lighting]` | `DominatorGround`, `DominatorLevel` | thousandths | dominator set | 0, 0 |
| map `[Lighting]` | `DominatorAmbientChangeRate` | thousandths, used as ticks | dominator fade interval | 1 tick |
| map `[Lighting]` | `NukeAmbientChangeRate` | decimal truncated to integer ticks | nuke fade interval | 1 tick |
| rules `[AudioVisual]` | `AmbientChangeRate` | decimal game minutes | normal fade interval (×900) | 0.2 (→ 180 ticks) |
| rules `[AudioVisual]` | `AmbientChangeStep` | decimal → hundredths ⌊×100⌋ | fade step | 0.1 (→ 10 hundredths) |
| rules `[AudioVisual]` | `ExtraUnitLight`, `ExtraInfantryLight`, `ExtraAircraftLight` | decimal → ‰ ⌊×1000⌋ | object brightness additions | 0 |
| rules, building type | `LightVisibility` | integer leptons | light radius; 0 reaches no cell in our engine, and a non-zero intensity with radius 0 gives an importer warning (claim 11, radius 0) | 5000 |
| rules, building type | `LightIntensity` | decimal → ‰ ⌊×1000 + 0.1⌋ | light intensity; 0 = no light | 0 |
| rules, building type | `LightRedTint`, `LightGreenTint`, `LightBlueTint` | decimal → ‰ ⌊×1000 + 0.1⌋ | light tint | 1000.0 (→ 1 000 000 ‰) |
| art, animation entry | `UseNormalLight` | boolean | animation drawn at 1000 regardless of cell | no |
| art, building image | `TerrainPalette` | boolean | terrain-palette building: scaled ground level of the centre cell (claim 13; palette in S-0103) | no |
| rules, terrain object type | `SpawnsTiberium` | boolean | ore-spawning terrain object: unscaled ground level (claim 13) | no |
| rules, overlay type | `Wall` | boolean | wall overlay: unscaled ground level (claim 13) | no |
| rules, overlay type | `Tiberium` | boolean | ore and gem overlays: brightness 1000 (claim 13) | no |
| theater's data, tileset section (glossary: tileset) | the tileset's boolean shadow-caster key (Open questions: its spelling) | boolean | tiles of this set mark nearby cells as shadowed (claim 13, Terms) | no |
| settings file (glossary), section `[Options]` | `DetailLevel` | integer; values above 2 become 2 and values below 0 become 0 | detail level: tint quantum (claim 8), building lights only at 2 (claim 11) | the detail level keeps the value it had before the file was read; scenarios set the key |

The nuke flash set has no keys (fixed constants). The rows whose source is the rules are read again by each later rules layer, so a test map can set those keys in its own override sections (claim 12 says how an omitted light or extra-light key behaves when it does).

## Measurement scenarios

**Common setup.** Every scenario runs under the glossary's *measurement setup*: an unmodified install with no loose override files and no engine add-ons, stock data, plus the test data the scenario names. Test data is a purpose-made tiny flat map with its own lighting section and, where stated, override sections of its own: for the game-wide visual settings (the audio-visual section of Import mapping) or for a stock building type. No scenario needs a loose file: none uses a loose palette or a loose copy of a rules file. The test map is removed afterwards. The detail level is set with the settings-file key of Import mapping before the game starts, or with the in-game options where a scenario toggles it. Game speed and screen resolution are free. The display path (glossary) may include a presentation-only display compatibility layer, as the measurement setup allows; the measurement record names the display path used, and any such layer generically, by what it does. Screenshots must keep the game's 16-bit output exactly, unscaled and unfiltered, and saved losslessly; S-0103's scenario PA checks that the display path used does so, and is run with the same display path. The original's lighting has no random element, so **one run** of each scenario is enough unless the scenario says otherwise. "Per tick" recording means a capture in which the tick of every recorded drawn frame is known, with at least one drawn frame per tick. "Row *n*" is the shade row a brightness gives by claim 15 (1000 → 26, 950 → 24, 500 → 12, 1075 → 27, 1076 → 28, 1400 → 36, 1500 → 39 for 53-row tables).

**Test light building.** A stock building type with a 1×1 foundation and no light keys in any stock rules layer (the measurer picks one and checks this in the stock rules at measurement time), whose section the test map restates with the light values the scenario gives. Unless a scenario says a key is omitted, the test map states all five light keys of Import mapping: intensity, radius and the three tints. It is placed with the map, and its light sits at the centre of its cell, so a cell whose centre is *k* cells away along a cell axis is 256·*k* leptons from the light.

**Neutral test lighting** (used below and by S-0103's scenarios): map ambient 1.00, tint 1.00 / 1.00 / 1.00, ground bias 0, level factor 0, every cell at height 0, and extra unit, infantry and aircraft light all 0 (in the test map's audio-visual override section, which states all three keys). Objects and tiles then draw at brightness exactly 1000 (row 26 of 53). With the default ground bias they would draw at 950, which is row 24.

**Scenario L1: storage precision** (claim 1). Three runs: the test run and two comparison runs.
- *Setup:* test run: map ambient 0.995 and red tint 0.574, otherwise neutral test lighting. Comparison run 1: map ambient 0.99 and red tint 0.57. Comparison run 2: map ambient 1.00 and red tint 0.58. Detail high in all three (at low detail 570 ‰ and 580 ‰ quantise alike, claim 8).
- *Observable:* the pixels of one plain ground tile and of an infantry unit on flat ground, compared across the runs.
- *Expected:* the test run is identical to comparison run 1 (99 and 57 hundredths: brightness 990, row 25; red 570 ‰, quantised 544). It differs from comparison run 2 (brightness 1000, row 26; red 580 ‰, quantised 576).
- *Falsified if:* the test run differs from comparison run 1 (for example, it matches comparison run 2).

**Scenario L2: effect priority, re-evaluations and overlaps** (claims 2, 5). One run per part. Superweapons and storms are started by trigger actions of the test map at fixed ticks, so that each overlap is reproducible.
- *Setup:* a map with a 4-level plateau next to low ground, map level factor 0.100, the effects' own lighting values left at their absent-key defaults except where a part says otherwise, extra light 0. An infantry unit stands on low ground well away from every superweapon target.
- *Observable:* per tick, the ground tint (pixel colours of a chosen low-ground tile and of a plateau tile), the brightness of the plateau relative to the low ground, and the tick on which each changes.
- *(a) Nuke during an ion storm.* Start an ion storm, let the ambient settle, then detonate a nuclear missile off screen. *Expected:* from the detonation tick to 30 ticks after it the tint is the nuke tint, while the plateau's extra brightness follows the storm's level factor (0 by default, so the plateau is no brighter than the low ground). On the 31st tick after detonation the tint changes to the ion tint, and the ambient then fades back towards the ion ambient, not the base ambient. *Falsified if:* the plateau follows the nuke's level factor during the brightening phase, or the tint or the target return to normal while the storm is still in force.
- *(b) Ion storm ending during a dominator.* Launch a dominator, start an ion storm during its active phases, and let the storm end before the dominator's animation ends. *Expected:* during the overlap the tint is the ion tint. When the storm ends the tint becomes the dominator tint on that tick, and the ambient fades towards the dominator ambient. *Falsified if:* the tint becomes normal, or the ambient fades towards the base ambient, while the dominator's animation is still running.
- *(c) Dominator ending during an ion storm.* Launch a dominator during an ion storm that outlasts it. *Expected:* the tint is the ion tint throughout, and the target stays the ion ambient when the dominator's animation ends. *Falsified if:* the dominator tint or the normal tint appears at any point during the storm.
- *(d) Chrono flash.* Run 1: a trigger action starts a chrono flash alone. Run 2: the same, and a second trigger action a few ticks later, while the chrono flash is still in force, starts an ion storm that lasts at least 300 ticks after the chrono flash finishes. *Expected:* run 1: the ground pixels after the chrono flash are identical to those before it, and no fade step happens. Run 2: from the storm's start the tint is the nuke tint and the ambient fades towards 2.00. This stays after the chrono flash has finished, until the storm ends, and then the tint and ambient return to normal. The plateau's height brightening follows the storm's level factor throughout. *Falsified if:* run 1 changes the tint or the ambient; or in run 2 the ion tint shows while the storm is in force, or the nuke tint ends when the chrono flash finishes.
- *(e) Table built during an overlap.* As (a), but during the brightening phase change the visual detail level, with the game paused if need be. The change rebuilds every tile shade table and fully recomputes every cell (claim 10). *Expected:* from the detail change until the 31st tick after the detonation, the ground shows the ion tint while the infantry unit shows the nuke tint. *Falsified if:* the ground keeps the nuke tint after the detail change.

**Scenario L3: who takes which tint** (claim 3). One run per part.
- *Setup A:* a test light building (Common setup) with intensity 0.50, radius 2560 leptons, red tint 1.00, and green and blue tints stated as 0 (an absent tint key would default to 1000.0, claim 12), detail high, otherwise neutral test lighting. A plain terrain object (a stock tree) stands 2 cells from the light building. Start an ion storm, let it end, and wait for the ambient to settle.
- *Setup B:* two runs of a map with an infantry unit and an ordinary building (Terms) on flat ground: neutral test lighting, and the same with map tint 1.00 / 0.50 / 0.50. Then, in the second run, start an ion storm.
- *Observable:* ground pixels near and far from the light building, and the tree's pixels, before, during and after the storm. Infantry and building pixels in both runs of B, before, during and after the storm.
- *Expected:* A: the red cast near the light building disappears during the storm (the ground shows the uniform storm tint), while brightness near the light building stays higher than far away. The red cast returns after the storm. The tree shows the same red cast as the ground around it before and after the storm, and the storm tint during it (claim 13). B: infantry and building pixels are identical in the two runs before the storm, although the ground is red-shifted in the second one. During the storm, their colours shift to the storm tint's colour balance. After the storm, they are identical to their pre-storm pixels again.
- *Falsified if:* the red cast persists during the storm; or the tree shows no red cast outside the storm; or the infantry or building differ between the white and red map tints; or they keep their colours during the storm; or they do not return to their pre-storm pixels.

**Scenario L4: fade timing** (claim 4). One run per rate setting.
- *Setup:* neutral test lighting, with change rate *m* game minutes and change step *s* stated in the test map's audio-visual override section. A trigger action sets the base ambient to 0.50 at a fixed tick. Settings: (*m*, *s*) = (0.1, 0.1), (0.2, 0.05) and (0, 0.1). A separate run with (*m*, *s*) = (0.2, 0.1), map dominator ambient 1.50 and map dominator rate 0.01, launching a psychic dominator instead of the trigger action.
- *Observable:* per tick, the ticks on which a chosen tile pixel changes value.
- *Expected:* one change of ⌊100*s*⌋ hundredths every ⌊900*m*⌋ ticks until the target is reached: 10 hundredths every 90 ticks with (0.1, 0.1), 5 hundredths every 180 ticks with (0.2, 0.05). With *m* = 0, no change ever. With dominator rate 0.01 (stored 10, used as 10 ticks), changes 10 ticks apart while the dominator is in force.
- *Falsified if:* the intervals or step sizes differ, the *m* = 0 run changes, or the dominator run's changes are not 10 ticks apart.

**Scenario L5: shade rows and neutral −1** (claims 6, 7, 15). One run per part.
- *Setup:* neutral test lighting.
- *Subject:* an infantry unit on flat ground (it draws at exactly the cell's ground level plus the extra infantry light, 1000 here). Vehicles are voxel objects whose per-face shading changes the palette index (S-0105), so they are unsuitable for exact values.
- *Observable:* screenshot of an infantry pixel whose palette value has a channel that is a multiple of 8 (e.g. 248).
- *Expected:* that channel shows the 16-bit value of 247 (one 5-bit step lower than 248 would give).
- *Falsified if:* it shows 248's step.
- *Also:* vary the map ambient across 0.5–1.5 in steps of 0.1 (one run each). Expected: in each run the infantry pixel shows the row claim 15 gives for the brightness (ambient × 1000 ‰). Falsified if any run shows another row.
- *Dropped low bits (claim 6):* neutral test lighting except for the map tint and ambient below, detail high. The tint's red channel is 1000 ‰, so *K* = 65536, the scaled and unscaled ground levels are equal, and the tiles use the table of the quantised tint. Each run lists one 8-bit tile-palette component *u* whose green or blue output differs after packing with and without the dropped low bits (Formulas, "Shade table"):
  - Run A: map tint (1.00, 0.87, 0.87), quantised (992, 864, 864); ambient 0.30 (row 7). *u* = 172: with the dropped bits 39 (6-bit green 9, 5-bit blue 4), without them 40 (green 10, blue 5).
  - Run B: map tint (1.00, 0.77, 0.77), quantised (992, 768, 768); ambient 0.90 (row 23). *u* = 212: 143 (green 35, blue 17), without them 144 (green 36, blue 18).
  - Run C: map tint (1.00, 0.74, 0.74), quantised (992, 736, 736); ambient 0.80 (row 20). *u* = 212: 119 (green 29, blue 14), without them 120 (green 30, blue 15).
  - *Subject:* a plain tile whose image shows a tile-palette entry with green or blue component 172 (run A) or 212 (runs B and C). The measurer finds such an entry in the stock tile palette of the theater used, and such a tile, at measurement time; any theater may be used.
  - *Observable:* the packed green and blue values of that entry's pixels.
  - *Expected:* the values with the dropped bits in every run.
  - *Record:* the measurement record names the processor and the 16-bit pixel layout the game runs with (with a display compatibility layer, the layout the game is given), since the original's result depends on both (claim 6).
  - *Falsified if:* any run shows the values without the dropped bits.
- *Anim palette:* an anim-palette animation under neutral test lighting shows its palette colours exactly (no −1), and keeps them during an ion storm apart from brightness. Falsified if it shows the −1 or takes the storm tint.

**Scenario L6: tint quantisation** (claim 8). One run per detail level.
- *Setup:* map tint (1.00, 0.90, 0.80), otherwise neutral test lighting.
- *Procedure:* run at detail low, medium and high, each set in the settings file before the game starts (the in-game options cannot select medium).
- *Observable:* ground pixel colours.
- *Expected:* every channel quantised to multiples of 128 / 64 / 32 ‰ (red 1000 → 896 / 960 / 992, green 900 → 896 / 896 / 896, blue 800 → 768 / 768 / 800). The normalised tint is (1000, 900, 800), which is not white, so the neutral tile table is not used.
- *Falsified if:* the colours vary continuously, or match other quanta.

**Scenario L7: light falloff, detail and relighting timing** (claims 9, 11, 12). One run per detail level, and five runs of the timing part.
- *Setup:* a test light building (Common setup) with intensity 0.50, radius 2560 leptons and all three tints 0, in darkness (map ambient 0.30), map tint white, ground bias, level factor and extra light 0.
- *Observable:* the shade row of an infantry unit standing on the centre of a cell at each distance 0–10 cells, at detail high, medium and low, each set in the settings file before the game starts (the in-game options cannot select medium). Infantry use the unscaled ground level, which does not depend on the light's tint, so the tint keys do not matter for this part.
- *Expected:* at high detail the infantry's brightness is 300 ‰ + ⌊500·*f*/1000⌋ with *f* = ⌊1000·(*R* − *d*)/*R*⌋, and its row follows claim 15. At medium and low detail there is no light (brightness 300 everywhere).
- *Falsified if:* the profile differs, or the light shows at medium or low detail.
- *Timing part (claim 12), five runs.* The relighting is made large, so that work spread over ticks by a real-time budget would take many ticks and show.
  - *Setup:* a flat 100 × 100-cell test map, detail high, map ambient 0.30, map tint white, ground bias, level factor and extra light 0. Twenty-five test light buildings in a 5 × 5 grid across the map, each with intensity 0.10, radius 25600 leptons (100 cells) and all three tints 0, so that every light reaches almost every cell. Trigger actions of the test map destroy all twenty-five on the same fixed tick. The game speed is the fastest, and the run is recorded on the slowest machine available; the record names the machine.
  - *Observable:* per tick, from screenshots or a capture of the whole visible area scrolled over the map in successive runs (the same tick in each run), the ticks on which the ground pixels of each visible cell change, counted from the trigger tick.
  - *Expected:* every cell that was lit changes on one and the same tick, and the number of ticks from the trigger tick to that tick is the same in all five runs and in every scrolled view.
  - *Falsified if:* cells change on different ticks (a front spreading over several ticks, which is what a real-time budget would produce), or the number of ticks from the trigger tick differs between runs or views.

**Scenario L8: dominator height bug** (claim 10). One run per part.
- *Common setup:* a map with a 4-level cliff plateau next to low ground (height 0), neutral test lighting (so map level factor 0 and ground bias 0), dominator level factor and ground bias left at their absent-key defaults (0), detail high. One infantry unit stands on the plateau top and one on the low ground, both on flat cells away from shadow-casting tiles (claim 13) and far from the dominator's target. Infantry draw at the cell's unscaled ground level, which the full recompute computes with the nuke flash level factor (claim 10).
- *Part A, detail change.* Dominator ambient left at its default (1.50).
  - *Procedure:* launch the psychic dominator. During its active phases, once the ambient has stopped changing, toggle the detail level from high to low, which fully recomputes every cell.
  - *Observable:* the shade rows of the two infantry units before and after the toggle.
  - *Expected:* before the toggle both units are at row 39 (brightness 1500). After it, the plateau unit is at row 49 (1500 + 4 × 100 = 1900) and the low-ground unit stays at row 39. This lasts until the cells' next incremental update: the next ambient step or lighting re-evaluation, at the latest when the dominator's final phase starts.
  - *Raised level (claim 10):* the map also has a high bridge whose deck cells are all at height 0, in view. Its deck is at row 39 (raised level 1500 + 4 × 0) before and after the toggle, because the raised level uses the dominator's level factor.
  - *Falsified if:* both units are at the same row after the toggle, or the deck goes to row 49 (1900) after the toggle.
- *Part B, light change.* Dominator ambient stated as 1.00 in the test map, so the dominator causes no ambient step. A test light building (Common setup) with intensity 0.30, radius 2560 leptons and all three tints 0, so the raw cell tint stays white, stands on low ground, with the two infantry units 4 cells (1024 leptons) from it on either side: one on the plateau top, one on low ground. A trigger action destroys the building at a fixed tick that falls in the dominator's active phases.
  - *Procedure:* launch the psychic dominator, and let the trigger action destroy the building.
  - *Observable:* per tick, the shade rows of the two infantry units from before the launch until after the dominator's final phase starts.
  - *Expected:* before the launch, and during the active phases before the building is destroyed, both units are at the same row. With the light at the building's cell centre, *f* = 600 and the light adds 180 ‰, so both are at row 30 (brightness 1180). On the tick the building's light goes out (claim 12: every relit cell changes on that tick), the plateau unit goes to row 36 (1000 + 4 × 100 = 1400) and the low-ground unit to row 26 (1000). Both are at row 26 from the tick the dominator's final phase starts.
  - *Falsified if:* after the light goes out, both units are at the same row, or the plateau unit is at any row other than 36 while the low-ground unit is at row 26.
- *Part C, ion storm in force.* Ion ambient stated as 1.00 in the test map, ion level factor and ground bias left at their absent-key defaults (0).
  - *Procedure:* start an ion storm that lasts until the end of the run. Once the ambient has settled, launch the psychic dominator. During its active phases, toggle the detail level from high to low.
  - *Observable:* the shade rows of the two infantry units before and after the toggle.
  - *Expected:* both units at row 26 before and after the toggle: the full recompute uses the ion storm's level factor (0), not the nuke flash's (claim 10).
  - *Falsified if:* the plateau unit is brighter than the low-ground unit after the toggle.

**Scenario L9: untinted light default and which ground level each drawn thing reads** (claims 9, 12, 13). Two runs.
- *Setup:* map ambient 0.50, otherwise neutral test lighting, detail high. A test light building (Common setup) with intensity 0.30 and radius 2560 leptons, in the middle of a flat area. Run 1 omits all three tint keys from the restated section, so the tints take the 1000.0 default (claim 12). Run 2 states all three tints as 0. The chosen building type must have no tint keys in any stock rules layer, so that the omitted keys re-read as 1000.0 and are not truncated (claim 12). Subjects, each standing on (or created at) a cell whose centre is 2, 4 or 6 cells from the light along a cell axis:
  - a terrain animation: a tile that the theater animates, placed in that cell,
  - a vein animation, from a vein field placed in that cell (as in S-0103's scenario PE),
  - an anim-palette animation (row 5 of the table in S-0103, claim 8), played in that cell by a trigger action,
  - a stock animation whose art sets the use-normal-light flag and which S-0103 places in row 5, played in that cell by a trigger action,
  - an infantry unit,
  - a plain ground tile (secondary observable).
- *Observable:* the shade row of each subject in each run.
- *Expected:* at 2, 4 and 6 cells the falloff is *f* = 800, 600 and 400 ‰, the light adds 240, 180 and 120 ‰, and the ground level is 740, 680 and 620 ‰ (rows 19, 17 and 16) in both runs. In run 1 every channel of the raw cell tint is clamped to 2000, so the cells use the neutral tile table with *K* = 131072 and their scaled ground level is 1480, 1360 and 1240 (rows 38, 35 and 32). So:
  - the terrain animation and the vein animation are at rows 38, 35 and 32 in run 1, and at rows 19, 17 and 16 in run 2,
  - the anim-palette animation and the infantry unit are at rows 19, 17 and 16 in both runs,
  - the use-normal-light animation is at row 26 in both runs, at every distance.
- *Falsified if:* in run 1 the terrain or vein animation is at the same row as in run 2 (no doubling: the absent tint is not 1000.0, or these animations do not read the scaled level); or the anim-palette animation or the infantry unit differ between the runs; or the use-normal-light animation is at any row other than 26.
- *Secondary observable:* the plain tile is at rows 38, 35 and 32 in run 1 if terrain tiles are drawn with the scaled ground level, and at rows 19, 17 and 16 if they use the unscaled one. Either result answers the open question; neither falsifies a claim.

**Scenario L10: object brightness** (claim 13). One run per part.
- *Setup:* neutral test lighting, except where a part says otherwise.
- *Extra light:* extra infantry light 0.076 in the map's audio-visual override section. Observable: an infantry unit on flat ground. Expected: row 27 (extra light 75 ‰, brightness 1075). Falsified if row 28 (76 ‰, brightness 1076).
- *Vehicle on a bridge deck:* map level factor 0.100. Observable: a vehicle on a bridge deck, compared with the same vehicle at the same facing on flat ground under map ambient 1.40 in a second run. Expected: identical pixels (+400 ‰). Falsified if the deck vehicle matches ambient 1.00 instead.
- *Infantry on a bridge deck:* map level factor 0.100. Observable: an infantry unit standing on a bridge deck, compared with the same unit at the same facing on flat ground. Expected: the same pixels as on flat ground (no deck brightening). Falsified if brighter.
- *Infantry in the air:* map level factor 0.100. Paratroopers dropped over ordinary flat cells. Observable: per tick, a paratrooper's pixels during the fall, compared with its pixels once landed. Expected: the same pixels throughout the fall as on the ground. Falsified if they are brighter while high.
- *Aircraft:* map level factor 0.100. Observable: an aircraft taking off, compared with itself landed at the same facing. Expected: its brightness rises in steps of 100 ‰, one step for every 208 leptons of height. During an ion storm (ion level factor 0) the flying aircraft is as bright as the landed one. During a psychic dominator (dominator level factor 0) it keeps the 100 ‰ steps. Falsified if the steps follow another height interval, vanish during the dominator, or remain during the storm.

**Scenario L11: re-read truncation** (claim 12). Four runs, one per map.
- *Setup:* a skirmish in a game mode without its own rules sections. The measurer picks, from the stock base rules, a light-emitting building type whose intensity *v* satisfies 0 < |*v*| < 1 and whose stock light changes the row of an infantry unit standing 1 cell from it under neutral test lighting (computed from the stock values by claims 11 and 15). The measurer also reads the stock extra infantry light *w*.
  - Map 1, under neutral test lighting, places that building type and restates its section with one key that is not a light key (for example its strength, at its stock value) and none of its light keys. Detail high.
  - Map 1′ (control) is map 1 without the restated section.
  - Map 2 uses neutral test lighting, except that its audio-visual override section states no extra-light keys (it still states the change rate, so the section exists).
  - Map 2′ (control) is map 2 without any audio-visual override section.
- *Observable:* maps 1 and 1′: the shade row of an infantry unit standing 1 cell from the building. Maps 2 and 2′: an infantry unit on flat ground.
- *Expected:* on map 1 the intensity is 1000·(*I* ÷ 1000) ‰, where *I* = ⌊1000·*v* + 0.1⌋ is the stored stock value, so the light is gone and the infantry unit is at row 26, while on map 1′ it is at the row the stock light gives. On map 2 the extra infantry light is 1000·(*E* ÷ 1000) ‰ with *E* = ⌊1000·*w*⌋, so a value with |*w*| < 1 gives brightness 1000 and row 26, while map 2′ shows brightness 1000 + *E*.
- *Falsified if:* the infantry unit is at the same row on map 1 as on map 1′, or (when *E* changes the row) at the same row on map 2 as on map 2′.

**Scenario L12: nuke flash phases and nuke fade rate** (claims 4, 5). One run per setting.
- *Setup:* neutral test lighting, an infantry unit in view, change step 0.1 (10 hundredths) and change rate 0.2 game minutes (180 ticks), both stated in the test map's audio-visual override section. Fire a nuclear missile at a target off screen.
- *Observable:* per tick, the ticks on which a chosen bright pixel of the infantry unit changes value. Tick 0 is the tick on which the tint first changes (the detonation).
- *Expected, default nuke rate (1 tick):* changes on tick 0 (tint), ticks 1 to 10 (ten brightening steps, from 100 to 200 hundredths), none on ticks 11 to 30, a change on tick 31 (tint back to normal: the fading phase starts), ten darkening steps on ticks 32 to 41, and no change after that.
- *Expected, map nuke rate 3.9:* the rate truncates to 3 ticks. Brightening steps on ticks 1, 4, 7, …, 28 (ten steps). Tint back on tick 31. Darkening steps on ticks 32, 35, 38, 41, 44, 47 and 50, which brings the ambient to 130 hundredths. The step on tick 50 is the first taken after the nuke flash is over, so the next interval is the normal 180 ticks: the remaining three steps come on ticks 230, 410 and 590.
- *Falsified if:* the first step is not on tick 1; the tint returns on any tick other than 31; a 3.9 rate gives 4-tick steps; or the 3-tick steps continue after tick 50 (or stop before it).

**Scenario L13: trigger-action ambient changes during an ion storm** (claim 14). Two runs.
- *Setup:* neutral test lighting and a trigger that sets the base ambient to 0.50 at a fixed time.
- *Procedure:* run 1: start an ion storm before the trigger fires and keep it going for at least 300 ticks after it. Run 2: the same with no storm.
- *Observable:* per tick, the brightness of a tile pixel.
- *Expected:* run 2: the fade towards 0.50 starts after the trigger fires. Run 1: during the storm the brightness stays at the storm's ambient after the trigger fires. When the storm ends, it fades towards 0.50, not back to 1.00.
- *Falsified if:* in run 1 the brightness moves towards 0.50 during the storm, or returns to 1.00 after it.

**Scenario L14: ore and gem brightness** (claim 13). Two runs.
- *Setup:* a field of ore and one of gems. Run 1 with neutral test lighting, run 2 the same with map ambient 0.50.
- *Observable:* pixels of an ore cell, a gem cell and the bare tile next to each.
- *Expected:* the ore and gem pixels are identical in both runs. The bare tiles are darker in run 2.
- *Falsified if:* the ore or gem pixels differ between the runs.

**Scenario L15: shadowed cells** (claim 13). One run per numbered run of each part.
- *Part A, vehicles under a bridge.* A high bridge over water, with a hover or naval vehicle that can pass under it, under neutral test lighting. Naval and hovering vehicles are vehicles for claim 13.
  - *Procedure:* run 1: stop the vehicle under the bridge. Run 2: the same vehicle at the same facing on open water under map ambient 0.50 (brightness 500). Run 3: as run 1, after the bridge has been destroyed.
  - *Observable:* the vehicle's pixels in each run.
  - *Expected:* run 1 matches run 2 (brightness 1000 − 500 = 500, row 12). Run 3 matches the vehicle on open water under neutral test lighting (row 26).
  - *Falsified if:* run 1 shows no change (no penalty), or the vehicle under the bridge is completely black (−500 used as the brightness itself, row 0), or run 3 is still darkened.
- *Part B, infantry under a bridge.* Neutral test lighting. The test map's high bridge spans at least one passable land cell (for example a shore cell beside the water at one end of the deck). Run 1: an infantry unit stands on that land cell under the deck. Run 2: the same unit at the same facing on open ground under map ambient 0.50.
  - *Observable:* the infantry unit's pixels in each run.
  - *Expected:* run 1 matches run 2 (row 12).
  - *Falsified if:* the infantry unit under the deck is at row 26 or row 0.
- *Part C, infantry falling over bridge cells.* Neutral test lighting except map level factor 0.100. Paratroopers are dropped over the cells the high bridge spans, and land on its deck.
  - *Observable:* per tick, the brightness of a falling paratrooper and its height above the terrain.
  - *Expected:* while it is more than 208 leptons above the terrain and not yet on the deck, its brightness is 1000 + 100·(*h* ÷ 208) ‰ with *h* its height above the terrain, so it darkens in steps of 100 ‰ as it falls. Once on the deck it is at row 26 (claim 13: on a deck *h* is measured from the deck). Over ordinary cells (Scenario L10, "infantry in the air") it stays at row 26 throughout.
  - *Falsified if:* the paratrooper stays at row 26 throughout the fall over the bridge cells.
- *Part D, a shadow-casting tile.* This part can run only in a theater whose data declares at least one tileset a shadow caster; the measurer checks the theater's data (stock data) at measurement time. Neutral test lighting. One tile of such a tileset is placed on flat ground. A ground vehicle is stopped, at the same facing, on the centre of every cell within 3 cells of the tile that it can reach.
  - *Observable:* the vehicle's pixels at each cell, matched against two references: the vehicle on open ground under neutral test lighting (row 26) and under map ambient 0.50 (row 12).
  - *Expected:* every cell matches one of the two references, and at least one cell matches the row-12 reference. The set of row-12 cells is recorded: it answers the open question on which cells such a tile marks.
  - *Falsified if:* a cell matches neither reference, or no cell matches the row-12 reference.

**Scenario L16: dominator final phase and lingering height terms** (claims 2, 5, 16). One run per part.
- *Setup:* a map with a 4-level plateau next to low ground. Map ambient 1.00, map level factor 0 and ground bias 0, so the plateau and the low ground are equally bright under normal lighting. Dominator ambient 1.00, dominator level factor 0.100 (equal to the nuke flash's fixed level factor, so that the full-recompute bug of claim 10 cannot change the result), dominator ground bias 0, dominator rate left at its default. Change step 0.1 and change rate 0.2 game minutes (180 ticks), stated in the test map's audio-visual override section. Extra light 0. One infantry unit on the plateau and one on the low ground, both on flat cells away from shadow-casting tiles (claim 13) and far from the dominator's target.
- *Part A, lingering:* launch the dominator at a target off screen and wait until well after its animation has ended. Then change the visual detail level.
  - *Observable:* the shade rows of the two infantry units: before the launch, during the dominator, after it, and after the detail change.
  - *Expected:* before the launch both at row 26. From the launch on, the plateau unit at row 36 (brightness 1000 + 4 × 100 = 1400), the low-ground unit at row 26. This stays after the dominator's animation ends, because the ambient is already at the target and no further update happens. After the detail change both are at row 26 again.
  - *Falsified if:* the plateau unit returns to row 26 when the dominator's animation ends, or stays at row 36 after the detail change.
- *Part B, final-phase split:* the same map, but dominator ambient 1.50 and dominator rate 0.1 (100 ticks per step).
  - *Observable:* per tick, the ground tint and the brightness of a low-ground tile.
  - *Expected:* during the active phases the tint is the dominator tint and the ambient steps up every 100 ticks. On the tick the animation reaches its last image frame, the tint returns to normal. From then on the ambient steps back towards 1.00, still one step every 100 ticks rather than every 180, and the plateau unit stays at its extra 400 ‰ above the low-ground unit throughout. If the visual detail level is changed during this time, the ground takes the dominator tint while the infantry keep their normal colours (new tile tables take the dominator's tint).
  - *Falsified if:* the tint stays the dominator's after the animation ends; or the steps after it are 180 ticks apart; or the plateau loses its extra brightness before the ambient has reached 1.00.
- *Part C, zero change rate:* the same map, but dominator ambient 1.50, and change rate 0 stated in the test map's audio-visual override section. A trigger action sets the base ambient to 0.50 before the launch. With a change rate of 0 the current ambient stays at 1.00 for the whole run, while the target moves (claims 4 and 14).
  - *Procedure:* launch the dominator. Wait until well after its animation has ended, then toggle the detail level from high to low and back.
  - *Observable:* the shade rows of the two infantry units, and the ground tint, before the launch, during the dominator, after its animation ends, and after the toggle.
  - *Expected:* before the launch both units at row 26. From the launch on, the plateau unit at row 36 and the low-ground unit at row 26. When the animation ends, the ground tint returns to normal, but the dominator is never over, because the current ambient (1.00) never reaches the target (0.50). So after the toggle the plateau unit is still at row 36, and the ground shows the dominator tint (tables built while the dominator is in force take its tint, claim 2), while the infantry units show their normal colours.
  - *Falsified if:* after the toggle both units are at the same row, or the ground shows the normal tint.

**Scenario L17: incremental update scales the unclamped ground** (claim 9, Formulas, "Incremental update"). One run.
- *Setup:* map ambient 2.50, map tint (0.80, 0.80, 0.80), ground bias 0, level factor 0, flat map with at least one tile that the theater animates, detail high, extra light 0. Normalisation gives *K* = 52428 and a white normalised tint (800 is not in 𝒟, claim 8), so the cells use the neutral tile table, which has 53 rows. Change step 0.1 and change rate 0.2 game minutes (180 ticks), stated in the test map's audio-visual override section. A trigger action lowers the base ambient to 2.40 at a fixed tick.
- *Subject:* an animation created as part of a cell's terrain (claim 13), which certainly uses the scaled ground level. If Scenario L9 has shown that terrain tiles also use it, a plain tile serves as well.
- *Observable:* the subject's shade row at three times: before the trigger action (after a full recompute, for example after toggling the detail level from high to low and back), after the ambient step, and after a further toggle to low and back.
- *Expected:* before the step, scaled ground ⌊2000 × 52428 / 65536⌋ = 1599, row 41. After the step, ⌊2400 × 52428 / 65536⌋ = 1919, row 49 (the unclamped ground is scaled). After the detail-level changes, 1599 and row 41 again, although the ambient is still 2.40.
- *Falsified if:* the row after the step is 41 (the ground was clamped before scaling), or the row after the detail-level changes is still 49.

**Scenario L18: near-black map tint** (claim 8, near-black tint and table choice). One run per part, plus a control run for Part B.
- *Part A, scaled ground:*
  - *Setup:* neutral test lighting, except that the map tint is (0, 0, 0). Detail high. Change step 0.1 and change rate 0.2 game minutes (180 ticks), stated in the test map's audio-visual override section. A trigger action lowers the base ambient to 0.90 at a fixed tick, which is one step. The map includes a tile that the theater animates, so that an animation created as part of a cell's terrain is in view, and an infantry unit stands near it.
  - *Observable:* the colour balance of the ground and the shade rows of the infantry unit and of the terrain animation: after toggling the detail level from high to low and back (a full recompute of every cell), and after the ambient step.
  - *Expected:* the ground never shows a colour cast (its normalised tint is white), and after the ambient step it is lit, not black. After the detail toggle the terrain animation is at row 0 (scaled ground 0), while the infantry unit stays at row 26. After the ambient step the terrain animation is at row 23 (900 ‰) and the infantry unit is at row 23 as well.
  - *Falsified if:* the ground is black or coloured after the ambient step, or the terrain animation is not at row 0 after the detail toggle while the infantry unit is lit.
- *Part B, table choice on the normalised tint:*
  - *Setup:* neutral test lighting, except that the map tint is (0, 0, 0) and the map ambient is 1.10. Detail low from the start of the game (set with the settings-file key before it starts, Common setup). The same animated tile as in Part A, and the same change step and rate. A trigger action lowers the base ambient to 1.00 at a fixed tick, which is one step. *Control run:* the same map with neutral test lighting (map tint white, ambient 1.00), detail low.
  - *Observable:* once the ambient step has happened, the pixels of a plain ground tile, and of the terrain animation (one chosen image frame) if it is drawn at low detail. The plain tile is enough on its own, because after the step its scaled and unscaled ground levels are equal. Compare them with the same tile and image frame in the control run.
  - *Expected:* identical pixels. After the step both the scaled and the unscaled ground level are 1000 in both runs. Both runs use the neutral tile table, because the test is on the normalised tint, which is white.
  - *Falsified if:* Part B's pixels are darker than the control's. In particular, a channel that shows *u* − 1 in the control run would show ⌊*u* × 58720 / 65536⌋ (248 → 222). That is the quantised-white table (896, 896, 896) at row 26, which the run would show if the table were chosen by the raw cell tint (0, 0, 0) instead.

**Scenario L19: grey map tints and the red-largest rounding** (claim 8, red-largest rounding). Three runs.
- *Setup:* neutral test lighting (map ambient 1.00), except for the map tint. Detail low from the start of the game (set with the settings-file key before it starts, Common setup). Subject: a plain ground tile. The tints are chosen so that the tile is at row 26 whether terrain tiles use the scaled or the unscaled ground level (Open questions).
  - Run 1: map tint (1.01, 1.01, 1.01). 1010 is in 𝒟, so the normalised tint is (1000, 999, 999). *K* = 66191, scaled ground 1009 and unscaled ground 1000, both row 26.
  - Run 2: map tint (1.03, 1.03, 1.03). 1030 is not in 𝒟, so the normalised tint is white. *K* = 67502, scaled ground 1029 and unscaled ground 1000, both row 26.
  - Run 3 (control): map tint white. Ground 1000, row 26.
- *Observable:* the subject's pixels, compared across the three runs.
- *Expected:* run 2 is identical to the control (neutral tile table). Run 1 is darker. Each channel that shows *u* − 1 in the control shows ⌊*u* × 58720 / 65536⌋ instead (248 → 222), which is the quantised-white table (896, 896, 896) at row 26.
- *Falsified if:* run 1 is identical to the control (grey 1010 normalised to white, as with `quirks.red_normalisation_divisor` off), or run 2 differs from the control.

**Scenario L20: quantised-white table kept at a light change** (claim 8, kept table). One run.
- *Setup:* neutral test lighting, except map ambient 1.40, so every cell's ground level is 1400 (row 36) once the light below is gone. Detail high. A test light building (Common setup) with intensity 0.01, radius 2560 leptons, red tint 0.01, and green and blue tints 0 stands on flat ground in a large field of one plain tile type. A trigger action destroys it at a fixed tick.
  - While the building stands, a cell whose centre is *n* cells (256·*n* leptons) from it, with *n* from 2 to 8, has a raw cell tint of (1000 + 10 − *n*, 1000, 1000): red from 1008 down to 1002.
  - Their normalised tints run from (1000, 992, 992) to (1000, 998, 998), so they use the quantised-white table (992, 992, 992).
  - Cells more than 10 cells away use the neutral tile table.
- *Observable:* pixels of the plain tile in that ring and far away. Record them after the building's light has gone out, and again after toggling the detail level from high to low and back.
- *Expected:* after the destruction, ring and far cells have the same brightness, 1400 (row 36). In some channels the ring pixels are one 5- or 6-bit step darker than the far pixels, and never brighter. They are what the (992, 992, 992) table gives at row 36: for example, a channel of 168 shows 230 (5-bit value 28) in the ring and 232 (29) far away. After the detail toggle, ring and far pixels are identical.
- *Falsified if:* right after the destruction the ring pixels are identical to the far pixels (the neutral tile table chosen afresh), or they still differ after the detail toggle.

**Scenario L21: trigger-action tint change** (claim 17). One run.
- *Setup:* neutral test lighting, detail high, an infantry unit and an ordinary building (Terms) on flat ground. A trigger action sets the map's red tint to 0.50 (50 hundredths) at a fixed tick. Later, a second trigger action starts an ion storm, which then ends.
- *Observable:* pixels of the infantry unit, the building and a plain ground tile at four times: before the action, after it, during the storm and after the storm. Then toggle the detail level from high to low and back, and record the pixels again.
- *Expected:*
  - Before the action, every non-zero channel shows *u* − 1 (claim 7).
  - From the tick of the action, the infantry, the building (outside its tint-exempt indices) and the ground all show the tint (500, 1000, 1000) at row 26. Red becomes ⌊*u* × 32752 / 65536⌋ (248 → 123), and green and blue stay at *u* − 1. The ground's red is not quantised: 123, not the 119 that the quantised tint (480, 992, 992) would give.
  - The brightness does not change.
  - During the storm everything shows the storm tint. After the storm the infantry, the building and the ground return to the red-reduced pixels.
  - After the detail toggle, the ground uses the table of its quantised tint (480, 992, 992), so 248 → 119. The infantry and the building stay at 123.
- *Falsified if:* the infantry or building pixels do not change at the action, or they return to *u* − 1 after the storm, or the ground's red is 119 before the detail toggle.

**Scenario L22: green-largest tints normalise high** (claim 8, which channel is largest). Two runs.
- *Setup:* neutral test lighting except map ambient 1.01, detail high, a large field of one plain tile type, and a test light building (Common setup) with intensity 0.001 (stored 1 ‰, which adds nothing at the measured cells) and radius 2560 leptons. The measured cells are those of the four cells whose centres are 1 cell (256 leptons) from the light along a cell axis that the building's image does not cover on screen; there *f* = 900, so a light tint stored as *t* ‰ adds ⌊0.9·*t*⌋ ‰.
  - Run R: map tint (0.99, 0.82, 0.82) and light tints (0.005, 0.008, 0.008). The measured cells' raw cell tint is (990 + 4, 820 + 7, 820 + 7) = (994, 827, 827). Red is largest; 994 is not in 𝒟, so green and blue become the exact quotient 831, quantised to 800.
  - Run G: map tint (0.82, 0.99, 0.82) and light tints (0.008, 0.005, 0.008). The raw cell tint is (827, 994, 827). Green is largest, *K* = 65142, and red and blue become ⌊827·65536 / 65142⌋ = 832, quantised to 832.
  - In both runs *K* = 65142, the unscaled ground level is 1010 and the scaled one 1003, both row 26, and the table has 53 rows.
- *Observable:* the blue channel of the plain tile's pixels in the measured cells.
- *Expected:* a blue component *u* shows ⌊*u*·52416 / 65536⌋ in run R and ⌊*u*·54512 / 65536⌋ in run G (the 16.16 factors of 800 ‰ and 832 ‰ with their low bits dropped): for example 248 → 198 (5-bit value 24) in run R and 206 (5-bit value 25) in run G.
- *Falsified if:* the blue channels are the same in both runs: both from 832 ‰ (the red-largest rule also divides by the truncated scale) or both from 800 ‰ (the green-largest rule gives the exact quotient).

**Scenario L23: buildings, terrain objects, overlays and own animations** (claims 3, 9, 13). One run per numbered run of each part.
- *Part A, scaled or unscaled ground level.*
  - *Setup:* three runs of one map. Run 1: map tint (0.80, 0.80, 0.80) and map ambient 1.00, otherwise neutral test lighting. Run 2: neutral test lighting except map ambient 0.80. Run 3: neutral test lighting. Detail high in all three. In run 1 the raw cell tint is (800, 800, 800), which normalises to white with *K* = 52428 (800 is not in 𝒟, claim 8), so every cell uses the neutral tile table, its unscaled ground level is 1000 (row 26) and its scaled ground level is ⌊1000 × 52428 / 65536⌋ = 799 (row 20). In run 2 both levels are 800 (row 20) and in run 3 both are 1000 (row 26). The neutral tile table is used in all three runs.
  - *Subjects,* on flat ground, each on its own cells: an ordinary building, a terrain-palette building, a plain terrain object (a tree), an ore-spawning terrain object, a wall overlay segment, an overlay that is neither a wall nor ore or gems and is not part of a bridge, an ore cell and an infantry unit. The measurer picks stock types for each at measurement time (stock data), and the map places them.
  - *Observable:* each subject's pixels, compared across the three runs.
  - *Expected:*
    - Unscaled group (the ordinary building, the ore-spawning terrain object, the wall and the infantry unit): run 1 is identical to run 3, and darker in run 2.
    - Scaled group (the terrain-palette building, the tree and the other overlay): run 1 is identical to run 2, and brighter in run 3.
    - The ore cell: identical in all three runs.
  - *Falsified if:* any subject of one group behaves like the other group, or the ore cell differs between the runs.
- *Part B, the top-left foundation cell.*
  - *Setup:* neutral test lighting except map ambient 0.50, detail high. An ordinary building of a stock type with a 2 × 2 foundation and no light keys in any stock rules layer, which the measurer picks at measurement time. A test light building (Common setup) with intensity 0.50, radius 300 leptons and all three tints 0. Its light reaches only the cells whose centres are within 300 leptons of its own cell's centre: its own cell and the four cells next to it along the cell axes, at 256 leptons, where *f* = ⌊1000 × 44 / 300⌋ = 146 and the light adds 73 ‰ (brightness 573, row 14). Unlit cells are at 500 (row 12).
  - *Run 1:* the light building stands next to the 2 × 2 building's top-left cell along a cell axis, outside the foundation, so that of the foundation cells only the top-left one is lit.
  - *Run 2:* the light building stands next to the opposite foundation cell (the one with the largest x and y) along a cell axis, so that only that cell is lit. For a 2 × 2 foundation that cell is also the cell under the building's centre (claim 13).
  - *Reference runs:* the same building with no light building, under map ambient 0.50 (row 12) and under map ambient 0.57 (brightness 570, row 14).
  - *Observable:* the pixels of the building's body, and of its own animation drawn with it if the chosen type plays one while the run is recorded.
  - *Expected:* run 1 is identical to the ambient-0.57 reference (row 14). Run 2 is identical to the ambient-0.50 reference (row 12). The building's own animation, if there is one, follows the body in both runs.
  - *Falsified if:* run 1 matches the ambient-0.50 reference, or run 2 matches the ambient-0.57 reference.
  - *Run 3, own animation following the light:* neutral test lighting, with change step 0.1 and change rate 0.015625 game minutes (⌊0.015625 × 900⌋ = 14 ticks) stated in the test map's audio-visual override section. An ordinary building of a stock type that plays an animation of its own drawn with it all the time (the measurer picks one at measurement time), and an infantry unit on open flat ground. A trigger action lowers the base ambient to 0.90 at a fixed tick, which is one fade step. *Observable:* per tick, the pixels of the building's own animation and of the infantry unit. *Expected:* the animation darkens to row 23 within one tick of the tick the infantry unit does. *Falsified if:* the animation keeps its earlier pixels after that, until it next starts again.
- *Part C, bridge deck and raised level.*
  - *Setup:* two runs of a map with a high bridge whose deck cells are all at height 0, for example over water, and a plain ground tile at height 0 next to it. Run 1: neutral test lighting except map level factor 0.100. Run 2 (reference): neutral test lighting except map ambient 1.40.
  - *Observable:* the pixels of the deck and of the plain tile in both runs.
  - *Expected:* the deck is identical in the two runs: in run 1 its raised level is 1000 + 100 × (0 + 4) = 1400 (row 36), with the level factor stored as 100 ‰ per level. In run 1 the plain tile is at 1000 (row 26), darker than in run 2.
  - *Falsified if:* the deck in run 1 is darker than in run 2 (for example at row 26, as if it read the ground level).
- *Part D, own animations.*
  - *Setup:* two runs, neutral test lighting and neutral test lighting with map ambient 0.50. Paratroopers are dropped over open flat ground. A stock vehicle whose art gives it a deploy animation (the measurer picks one at measurement time) deploys on open flat ground.
  - *Observable:* the pixels of a parachute during the fall and of the deploy animation, in both runs.
  - *Expected:* the parachute is darker in the ambient-0.50 run (row 12 against row 26). The deploy animation is identical in both runs (brightness 1000), while the vehicle itself is darker in the ambient-0.50 run.
  - *Falsified if:* the parachute is identical in both runs, or the deploy animation is darker in the ambient-0.50 run.
  - *Run 3, parachute fixed at creation:* the first run's map, with change step 0.1 and change rate 0.015625 game minutes (14 ticks) stated in the test map's audio-visual override section, and a trigger action that lowers the base ambient to 0.90 a few ticks after the paratroopers are dropped, so that the fade step comes while they are still falling. *Observable:* per tick, the pixels of a parachute and of the infantry unit it carries. *Expected:* on the step tick the carried infantry unit darkens to row 23 (infantry over ordinary cells follow their cell, claim 13), while the parachute keeps its row-26 pixels until it is gone. *Falsified if:* the parachute darkens during the fall.
- *Part E, voxel parts and the centre cell of a terrain-palette building.* The light geometry, the reference runs and the rows are those of Part B (map ambient 0.50, detail high, a test light building with intensity 0.50, radius 300 leptons and all three tints 0, standing next to one foundation cell along a cell axis, outside the foundation, so that it lights that foundation cell alone).
  - *Runs 1 and 2, voxel parts:* an ordinary building of a stock type that has parts drawn as voxel models, such as a turret, a foundation at least 2 cells wide and deep, and no light keys in any stock rules layer (the measurer picks one at measurement time). No enemy is placed, and every run is recorded at the same facing of the voxel part. Run 1 lights the top-left foundation cell alone; run 2 lights the foundation cell with the largest x and y alone. *Observable:* the pixels of the voxel part, compared with the two reference runs of the same building. *Expected:* run 1 is identical to the ambient-0.57 reference, run 2 to the ambient-0.50 reference. *Falsified if:* run 1 matches the ambient-0.50 reference, or run 2 matches the ambient-0.57 reference.
  - *Runs 3 and 4, terrain-palette building with an even foundation:* a terrain-palette building of a stock type whose foundation has an even width or depth (the measurer picks one at measurement time; if stock data has none, the measurement record says so and this sub-claim stays at medium confidence). Write *C* for the foundation cell under the building's centre with the larger coordinate (claim 13); for a 2 × 2 foundation it is the cell with the largest x and y. Run 3 lights *C* alone; run 4 lights the top-left foundation cell alone (choose a type for which that cell is not *C*). The light's tints are 0, so every cell keeps a white raw cell tint, uses the neutral tile table and has equal scaled and unscaled ground levels. *Observable:* the pixels of the building's body, compared with the two reference runs of the same building. *Expected:* run 3 is identical to the ambient-0.57 reference, run 4 to the ambient-0.50 reference. *Falsified if:* run 3 matches the ambient-0.50 reference, or run 4 matches the ambient-0.57 reference (as if the building took its top-left cell, or the centre cell with the smaller coordinate).

**Scenario L24: radius-0 light** (claim 11, radius 0). Two runs.
- *Setup:* neutral test lighting except map ambient 0.50. A test light building (Common setup, 1 × 1 foundation, so its light sits on its cell's centre) with intensity 0.50, radius 0 and all three tints 0. Run 1 starts at detail high, run 2 at detail low.
- *Procedure:* run 1: if the game reaches play, switch the detail level to low and back to high with the in-game options; a detail change fully recomputes every cell (claim 10). Run 2: record the shade row of an infantry unit on a cell next to the light building, then switch the detail level to high with the in-game options.
- *Observable:* whether and when the game crashes, and in run 2 the infantry unit's shade row before the switch.
- *Expected:* run 1 crashes when the game starts or, at the latest, at the switch back to high. Run 2 runs normally at detail low with no light anywhere (the infantry unit at brightness 500, row 12), because a building light is not counted below detail level 2, and crashes at the switch to high.
- *Falsified if:* no crash follows a start at, or a switch to, detail high in either run (the record then notes what the light's cell shows), or run 2 crashes before the switch.

## Open questions

- Which cells a shadow-casting terrain tile marks as shadowed (Terms, claim 13).
- How light-glow images write the light-alpha value, and the range of values it can then take (Terms, claim 15).
- Which drawing paths use the anim palette's untinted table, and whether any of them apply a tint elsewhere.
- Whether any overlay type outside the four overlay cases of claim 13 is drawn with another light value; none is known.
- The palette of ore-spawning terrain objects, and whether it takes any tint (claim 13).
- Which shade table overlays are drawn with, and so whether they take the map tint and light colours (claim 3). Their palettes belong to S-0103.
- The brightness of an object's own animation drawn with its object (S-0103, claim 8, row 3) other than a building's own animation, a parachute, and a deploy or undeploy animation (claim 13).
- How an object's brightness changes while it flashes, while it is protected by the invulnerability superweapon, or while it is the target of an air strike (claim 13). These changes apply to buildings and to their own animations as well.
- The brightness of other drawn things that claim 13 does not list, such as smudges and animations drawn from voxel models.
- What lighting the original gives a position outside the map's legal cells (S-0009, claim 1).
- Which cell light value terrain tiles are drawn with (scaled or unscaled ground). Scenario L9's secondary observable answers it.
- Which tile shade table a cell gets when its table is chosen again after a trigger-action tint change, before the next detail change (claim 17).
- What tint a house-colour or named palette takes if it is built after a trigger-action tint change (claim 17). Such palettes are normally all built when the game starts; whether any is built later is not established.
- Whether anything other than the overlays of deck cells reads a cell's raised light level (claims 9 and 10).
- The exact spelling of the theater's tileset key that declares a shadow caster (Import mapping). It is one word, in the theater's data; it is to be added to Import mapping at sign-off.

## Questions for the project lead

- Scope: light sources other than buildings (for example radiation sites) are not covered by this spec. Should they be added here, or left to another spec?
- Consistency with the glossary, for sign-off (delete this item once the glossary is corrected): the project lead's ruling on host- and real-time-dependent behaviour assumed that the original spreads a relighting of map cells over several ticks by a real-time budget. It does not (claim 12), so this spec states the same-tick relighting as the original's behaviour and needs no departure for it. The glossary's G3 confidence note still cites claim 12 as such a relighting and calls it a departure of this spec; that sentence needs correcting before both are signed off.
