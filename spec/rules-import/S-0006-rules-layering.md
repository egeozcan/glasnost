---
id: S-0006
title: Rules layering
topic: rules-import
revision: 15
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0006, Q-0007]
supersedes: []
---

# S-0006: Rules layering

## Summary

At each game start the game applies its rules layers in a fixed order on top of the rules state it already holds (base rules, language rules file if valid, game-mode layer, the map's own rule sections; a campaign mission's own file acts like a layer of the previous game), and this spec covers what a later layer can change, the per-key rule "an absent key keeps the held value" with its exceptions, the interaction with the type lists, the rule data that survives from one game to the next, and the mode list that names each game mode and its game-mode file. Type registration within a layer is in S-0004, and which copy of a file wins is in S-0005.

## Terms

Shared terms used here are defined in the glossary: *base rules*, *colour-adjustment table*, *departure*, *game*, *game minute*, *game-mode file*, *game start*, *game-wide section*, *importer warning*, *language rules file*, *map layer*, *measurement setup*, *override section*, *recorder*, *reset on override*, *rules layer*, *session*, *type list*, *type section*, *unspecified*.

- **Layer.** In this spec, short for the glossary's *rules layer*; it never means a render layer. Every layer is treated alike: its type lists add types (S-0004, claims 2 to 4), its game-wide sections are read, and each existing type reads its type section from it, subject to the rule of S-0004, claim 6, on which types are read in the layer that creates them.
- **Game-mode file.** Narrowed here from the glossary's entry: every listed mode names one (claim 13). Where the glossary says that a mode may have none, this spec reads it as a mode whose named file cannot be resolved, so that its game-mode layer is empty (claim 1; S-0005, claim 19).
- **Not given.** For a few numeric keys, one special value that the original treats as if the key were absent (claim 3).
- **Held value.** The value a key has in the rules state at the moment a layer is read. It is what earlier layers of this game set. If no layer of this game has set the key yet, it is what the previous game of the session left (claim 10) for a game-wide key or a tiberium's key, as changed by the mission-named file in a campaign mission (claim 14), and the engine default for any other key. In the session's first game, and for a tiberium first created by this game's layers, nothing is left over from an earlier game, so it is the engine default, as changed by the mission-named file where claim 14 applies.
- **Engine default.** A value fixed by the original's program rather than read from data: the value that a key has when no data gives it, the glossary's *absent-key default*. It always means this and nothing else. The value that a reset-on-override key takes when a layer omits it is called its *reset value* (claim 5); the two can differ for the same key.
- **Early game-wide section.** One of these game-wide sections: the general settings, AI settings, AI intelligence levels, power-ups, land types, jump-jet controls and multiplayer dialog settings (S-0004, Terms and claim 4, row 2). A type that one of them names first reads its own section in the layer that names it. The other game-wide sections (difficulty levels, crate rules, combat damage, radiation, elevation and wall models, audio-visual settings and special weapons) are the *late* ones (S-0004, claim 4, row 4). A type that a late section names first is a reference that S-0004, claim 6, leaves unread in that layer, except a warhead in a layer that contains the special-weapon settings section; claim 7 applies to it accordingly.
- **Tiberiums.** Resource types (one of the glossary's *type kinds*), such as ore or gems. The *tiberium list* is the type list of that kind.
- **Locomotion class.** As in the glossary and S-0211, which own the list; claim 5 uses it for the land-type multipliers, and *winged* is the further class there.
- **Mode list.** The data file that lists the multiplayer and skirmish game modes (S-0005, Import mapping). Each listed mode has a *mode number*, a *mode kind*, a name and a tooltip, a game-mode file, a map filter and a flag (claim 13).
- **Mode number.** The whole number that identifies a listed mode. The lobby lists modes in order of mode number, and a game refers to its mode by that number (claim 13).
- **Mode kind.** One of six fixed kinds of game mode. The section of the mode list that holds a mode's entry sets its kind. What each kind changes in play (alliances, teams, starting units and the like) is outside this spec.
- **Crate effect.** One of the 19 fixed effects that a crate can give, each with an animation and an integer relative weight in the power-up section (claim 5). A relative-weight unit is dimensionless, not a percent: weights describe relative selection chances. Full crate selection and effect rules are outside this spec.
- **String table.** The game's table of display texts, each found by its label. A mode's name and tooltip are labels in it (claim 13).
- **Mission-named file.** The rules file that a campaign mission applies besides its rules layers, with the effects of a layer of the previous game (claim 14). Its name is formed from the mission's map file name as the Import mapping gives (S-0002, claim 6). It is the earlier file of S-0002's layered load; this spec covers only what it does as a rules file.
- **Type read.** As in S-0007, Terms: one reading of a type's settings in a rules layer. An animation has one type read, from the art file, in every applied layer, except a layer in which the animation is first created after that layer's animation reading (S-0004, claim 6, last paragraph: such an animation reads its art from the next layer on) (claim 8). S-0007, claim 2, makes the number of an animation's type reads in a game observable.

## Behaviour

Implementation requirements cover skirmish only. Campaign descriptions, including claim 14 and scenario U, preserve deferred original context, not requirements for this milestone. Network games are out of scope.

1. **Layer order at each game start.** At every game start, these rules layers are applied on top of the rules state already held, from lowest to highest precedence:
   - the base rules,
   - the language rules file, only if one exists and either passes its check or holds no section (claim 12; stock data has none),
   - the game-mode layer, in every multiplayer or skirmish game (but see claim 12) and never in a campaign mission. It is the selected mode's game-mode file. Every listed mode names a file (claim 13), so the layer is empty only when that file cannot be resolved or holds no section (claim 11; S-0005, claim 19),
   - the map layer.

   These are all the rules layers of a game. In a campaign mission, the mission-named file also takes effect, but it is not one of these layers: it acts like a layer of the previous game, and claim 14 gives what it changes.

   An empty layer (a game-mode layer as above, or a language rules file with no section) changes no rules value, but it is still an applied layer: every animation that is not first created after that layer's animation reading gets one type read in it, from the art file (Terms; claim 8). So under stock data an animation that the base rules' animation list names has three type reads in every multiplayer or skirmish game, whether or not the mode's file can be resolved, and two in a campaign mission (base rules and map). The mission-named file adds none that counts in the mission (claim 14). A valid language rules file, or one with no section, adds one. The consequence of the count belongs to S-0007 (claim 2).

   Where more than one applicable layer gives the same key, the value from the highest-precedence layer is the value held after layering (claims 2 to 5 give the exact rule, including exceptions). Type data and game-wide data start differently:
   - Every type registry except the tiberiums' starts each skirmish empty, so it holds only the types created for this game (S-0004, claims 1 and 3); tiberiums carry over (claim 10). The original's deferred campaign exception is sides newly created by the mission-named file: they remain ahead of the base rules' sides for that mission (claim 14; S-0004, claims 1 and 9). No side of the previous game survives. This exception records original context only; campaigns and network games are out of scope for the skirmish milestone.
   - Game-wide sections are not cleared. Each game-wide key starts the game with the value the previous game of the session left (the engine default in the session's first game), as changed by the mission-named file in a campaign mission (claim 14), and keeps it unless a layer of this game gives the key (claim 10). The exceptions restored at every game start are listed in claim 10.

   The players' houses (countries, colours, starting credits and so on) reflect the final values of all layers: overrides of country properties made by a game-mode file or a map apply to them.
   - Backing: `measurable` for base, game-mode file and map (scenarios A, B, L). `measurable` for the position of the language rules file (scenario S, which uses a language rules file with a valid digest formed as S-0005, claim 21b, gives). `measurable` for skirmish type reads (local scenarios T and Q7). The deferred campaign count is `unmeasured` in this milestone because campaign execution is outside scope.
   - Confidence: high

2. **A layer may override any section.** A later layer can change every game-wide section and every type section, with these exceptions:
   - The maximum number of players is taken from the base rules only. A later layer cannot change it.
   - In the colour-definition section, a later layer's entry for a name already defined keeps the earlier definition, comparing names without regard to letter case. It creates no additional definition or position. New names append in layer and entry order. S-0103 claim 5 owns this rule and `quirks.colour_redefinition_ignored` (default on = keep the first definition; off = replace its value while keeping its position).
   - The language rules file cannot change the colour-adjustment table at a position that the base rules' table gives (see below). Game-mode and map layers can change it.
   - Tiberiums read their sections only from layers whose tiberium list names them (S-0004, claim 10). A tiberium section in a layer without a matching list entry is ignored.

   **Colour-adjustment table.** The table has 16 entries. A layer's colour-adjustment section changes the table by position, not by key name: the *n*-th entry of the section replaces the *n*-th entry of the table, whatever its key is called. Entries past the section's last entry keep their current value. At each game start, once the base rules and the language rules file (if valid) have been applied, each position *n* from 1 to 16 holds the value this table gives (except in the case of claim 12, where all 16 positions are zero):

   | Base rules' section has an *n*-th entry | A valid language rules file's section has an *n*-th entry | Position *n* holds |
   |---|---|---|
   | yes | yes or no | the base rules' *n*-th entry |
   | no | yes | the language rules file's *n*-th entry |
   | no | no, or there is no valid language rules file | zero (red, green and blue all 0, which adds no tint) |

   So nothing a previous game's game-mode file or map put into the table survives into the next game, unlike other game-wide values (claim 10). The game-mode and map layers then change the table by position as above. A section with more than 16 entries is bad data. In the original, the entries after the 16th overwrite other rules values; which values change is *unspecified*. This is a *departure*: our engine ignores every entry after the 16th, and the importer warns.

   Sections a layer doesn't contain leave the state unchanged. A type whose section is absent from a layer is not touched in that layer.
   - Backing: `measurable` for sampled type-section overrides, the game-mode and map colour-adjustment table, and the tiberium rule (A, M, N). `unmeasured` for overriding every remaining section individually: these scenarios sample the general rule, not every consumer; topic specs must test their own keys. `measurable` for the language rules file's part of the colour-adjustment rule (scenario S). `unmeasured` for the maximum number of players: it limits the lobby, which is set up before later layers are read, and no later effect has been identified (open question 6). `unmeasured` for an overlong colour-adjustment section: the original supplies no stable defined result to compare; ignoring the excess and warning is our design departure. The colour-definition exception is `measurable` by S-0103 scenario PF for retention across later layers and case-insensitive names; appending new names without extra positions for duplicates is `unmeasured` for the reason S-0103 claim 5 gives.
   - Confidence: high

3. **An absent key keeps the held value.** Subject to claim 2's section-specific exceptions, within a section that a layer contains, each key that is given replaces the held value. Each key that is absent keeps the held value (Terms):
   - in a type section, the value an earlier layer of this game gave, or the engine default of its kind if none did; for a tiberium, if none did, the value the previous game of the session left (claim 10), as changed by the mission-named file in a campaign mission (claim 14), and the engine default only for a tiberium first created in this game, as changed by the mission-named file where claim 14 applies,
   - in a game-wide section, the value an earlier layer of this game gave, or else the value the previous game of the session left (claim 10), as changed by the mission-named file in a campaign mission (claim 14). It is the engine default only until a layer of the session, or a mission-named file, first sets the key (claim 10).

   An entry with no value after its separator, or with only spaces or a comment as its value, is absent: it creates no entry (S-0001, claim 11). In a layer it therefore keeps the held value exactly as a missing key does; it never resets the held value to the key's reset value or engine default. A section whose entries are all absent in this way, or that has no entries, does not exist in that layer (S-0001, claim 12). Claim 5's resets therefore do not apply to it, and a type that reads that layer finds no section there (claim 7).

   Claim 5 lists the exceptions: keys that reset on override, values that are truncated when re-read, and an unset terrain type strength that takes the game-wide tree strength. The terrain strength's −1 is not a "not given" value: it requests that fill rather than keeping the held strength.

   The original's omitted infantry pip fields also drift on repeated section reads, as recorded in claim 5. In our engine both the passenger pip colour and occupant pip image frame keep the held value, with no quirk switch: this is a display-only correction, not another exception to retention (S-0003 claim 16; S-0111 claim 12).

   Some numeric keys have a "not given" value: written explicitly, it keeps the held value. For every key below except the voxel animation's two angular velocities, that is exactly what an absent key does. Those two also reset when absent (claim 5), and for them the "not given" value still keeps the held value. These are all of them:

   | Where | Key (our name) | Value form | "Not given" value |
   |---|---|---|---|
   | a building, vehicle, aircraft or infantry type's section | pitch angle; roll angle, in degrees (their use belongs to the movement and rendering specs) | decimal | −1 |
   | a superweapon's section | recharge time | decimal | 0 |
   | a voxel animation's section | minimum angular velocity; maximum angular velocity | decimal | 0 (see claim 5: these keys also reset when absent) |
   | an animation's art section (claim 8) | rate; each of the two numbers of the random rate | integer | −1 |

   The comparison is made on the number the key's value form gives (S-0003), before any later conversion of units, and it is exact. So for an integer key every spelling that reads as −1 counts, including `$FFFFFFFF`, `$-1`, `-1h` and `4294967295` (S-0003, claims 3 and 4). For a decimal key, `-1`, `-1.0` and `-100%` read as −1 (S-0003, claims 6 and 7) and count, while `$FFFFFFFF` is not a decimal and follows S-0003, claim 8. For the animation keys the art file is the same in every layer, so "not given" there means that the value from before the reading stands. No game-wide section has a key with a "not given" value.
   - Backing: `measurable` (scenario A for keep and replace in a type section; scenario O for a game-wide key across games; scenario J for the "not given" value of the recharge time; scenario R for the animation rate, including the `$FFFFFFFF` spelling; scenario D7 for the voxel animation's 0; scenarios D3c and D3d for an empty value and a section of empty lines; scenario AA for the original pip drift). Pip retention in our engine is a display-only design correction, with AA's separate engine acceptance check, not an oracle expectation. `unmeasured` for pitch and roll angles: their uses and observables are not established here (open question 8). `unmeasured` for the animation random-rate pair individually: R tests the single rate only; a fixed-seed duration test that isolates each endpoint is still needed.
   - Confidence: high that keeping the held value is the general rule for numeric and flag keys of every type kind and every game-wide section. Medium that claim 5 lists every exception: the exceptions other than fixed-default resets (the barrel and light values) were found late. High that the table of "not given" values above is complete for numeric keys: the numeric and flag keys of every type kind and every game-wide section were checked for every fixed default and every special value. Medium for text and list keys: the type-reference and list keys of buildings, vehicles, aircraft, infantry, weapons, warheads, countries, superweapons, particles and particle systems were checked, as were the value forms shared by many keys (ranges, speeds, coordinate triples, categories, movement zones, owner lists and prerequisites); other text keys were not checked one by one (open question 3).

4. **List-valued keys are replaced whole.** When a later layer gives a list-valued key, such as prerequisites, owner countries or a country's veteran-type lists, the new list replaces the old one entirely. Nothing is appended or merged. When the key is absent, the old list is kept.
   - Backing: `measurable` (scenario A)
   - Confidence: high

5. **Exceptions to keeping the held value.** There are four kinds: keys that reset to a fixed reset value, keys that reset to another key's value, values that are truncated when re-read, and an unset terrain type strength that is filled from a game-wide value.

   **Keys that reset on override.** In the following places, if a layer contains the section but omits the key, the value goes back to the reset value listed. It does not keep the held value. A reset value is not the key's engine default (Terms); where the two differ, the paragraph on the switch below says which one applies.

   | Where | Key (our name) | Value after a reset |
   |---|---|---|
   | a country's section | parent country | the country itself |
   | a warhead's section | damage modifiers versus each of the 11 armour classes | 100 % for all 11 |
   | each difficulty section | firepower, ground speed, air speed, armour, rate-of-fire and cost multipliers; build-time multiplier | 1.0 each |
   | each difficulty section | repair delay | 0.02 game minutes |
   | each difficulty section | build delay | 0.03 as a dimensionless imported scalar; no gameplay duration is specified here |
   | each difficulty section | build slowdown | off |
   | each difficulty section | destroys walls | on |
   | each difficulty section | content scan | off |
   | each of the 12 land-type sections | movement multiplier per locomotion class (foot, track, wheel, hover, float, amphibious, float-on-beach) | 100 % |
   | each of the 12 land-type sections | buildable | no |
   | a particle system's section | the particle type it emits | the particle type with an empty name (see below) |
   | a particle system's section | behaviour | none |
   | a particle's section | behaviour | none |
   | the power-up section | the weight of each of the 19 crate effects | 0 relative-weight units (dimensionless integer) |
   | the power-up section | the animation of each of the 19 crate effects | no animation (see Import mapping for the one exception) |
   | a voxel animation's section | minimum angular velocity; maximum angular velocity | −1, in the unit in which the key is written (the conversion to the stored value, and what the value does, are outside this spec; see open question 10) |
   | a voxel animation's section | whether it shares another model's data instead of loading its own | no (it loads its own model) |

   **The empty-named particle type.** A particle system's emitted particle type is a reference by name, and the reference creates a particle type of that name if none exists yet. When a layer contains a particle system's section but omits the key, the system emits the particle type whose name is empty. Only the explicit no-type value of the Import mapping avoids this (S-0004, claim 2). If no particle type with the empty name exists yet in this game, that layer creates it; its place in the particle registry follows S-0004, claim 4. Every particle system of the game that omits the key emits this same type, so it exists at most once per game. Like every type created by a reference that its creating layer leaves unread (claim 7), it reads no section in the layer that creates it. In each later layer of the same game it reads that layer's section with the empty name, if there is one (S-0001, claim 6: a header with nothing between the brackets); otherwise it keeps the engine defaults of its kind. The particle-system row applies to the base rules too: a particle system whose base-rules section names no emitted particle type creates the empty-named particle type in the base rules. Stock data names the particle type in every listed particle system, so with stock data this does not happen through the list.

   For the land-type rows, a movement multiplier that is given is capped at 100 %, and one further locomotion class not listed above, winged, is fixed at 100 % whenever a land-type section is read. A land type whose section has never been read has the startup row specified by the movement spec (S-0211 claim 2). Both rules belong to the movement spec (S-0211).

   **The power-up rows.** A crate effect whose entry the power-up section omits gets weight 0 and no animation; its other two values (whether it can appear on water, and its argument) keep the held value. An effect with weight 0 is expected never to be drawn when a crate is picked up (the full crate draw is outside this spec), so a layer whose power-up section lists only some effects is expected to leave every other effect out of the crate draw.

   **The voxel-animation rows.** These rows apply to the base rules too. The two angular velocities have 0 as their "not given" value (claim 3) and reset to −1 when absent: a layer whose section omits either key sets that key to −1, and a layer whose section gives it as 0 keeps the held value. So the engine default before any reading (0 for the minimum, 10 in the written unit for the maximum), which differs from the reset value, remains only for a voxel animation that reads no section at all, or whose every section that it reads gives that key as 0. The sharing value is yes only when the layer's section gives at least one of the three sharing keys of the Import mapping as yes; otherwise it is no in that layer, whatever an earlier layer gave.

   So a map that changes a single key of a warhead, a difficulty level, a land type or the power-up section silently resets the rest of that row's keys. For the warhead, that means the damage modifiers against all armour classes.

   Proposed switch: `quirks.override_resets_keys` (default **on** = reset as above). With it off, these keys keep the held value like every other key. The held value of a reset key that no earlier layer of this game has given is, as for every key (claim 3), its engine default, and never its reset value. This is the value that decides the outcome in the layer that creates a type, and in the first layer that holds a section of a game-wide row:
   - **Types created by a layer.** The held value in the layer that creates the type, and in every later layer until a layer gives the key, is the type's engine default. For a warhead, the damage modifier against every armour class is 100 %. For a particle system, the behaviour is none and the emitted particle type is *no type*. For a particle, the behaviour is none. For a voxel animation, the angular velocities are 0 and 10 in the written unit, and it loads its own model. For a country, the parent country is the country itself (not read out; this spec fixes it at the reset value, open question 11). Only the emitted particle type and the two angular velocities differ from their reset values. So with the switch off a particle system that omits the emitted type emits nothing, and the empty-named particle type is never created through that omission. An empty written value is still an absent key (claim 3), so it cannot create that type with this reset disabled. A voxel animation whose section omits an angular velocity keeps 0 or 10 (or the value an earlier layer gave); it is never −1 through omission.
   - **Game-wide rows (difficulty, land types, power-up effects).** The held value is the value the previous game of the session left, as for every game-wide key (claim 10). In the session's first game it is the engine default of the key. For these rows the engine defaults were not read out key by key, and this spec fixes each of them at the reset value of the table (a *design choice*, only reachable with the switch off, with no further switch). With the switch off the restoration of claim 10 for these rows no longer applies in full: the base rules restore only the keys they give, and an omitted key keeps what an earlier game left, like any other game-wide key (with `quirks.persist_global_overrides` off it is the engine default of the key, that is, the reset value of the table).
   - These switch-off defaults are design choices, not measurements of the original. The original's particle-system and voxel-animation resets are `measurable` by D3 and D7 with original behaviour; no original-game run can test our alternative.

   **Pip colours: a display-only departure.** An infantry type's passenger pip colour and occupant pip image frame keep the held value when a layer contains its section but omits the corresponding key, including in the base rules; before any reading their defaults are image frames 1 and 7 respectively. The original instead shifts these omitted values through the pip-colour vocabulary on successive reads, a display-only bug. Our engine fixes it without a quirk switch, independently of `quirks.override_resets_keys`. S-0111 claims 11 and 12 own the displayed pips, and claim 12 must state this fixed retention rule; S-0003 claim 16 already records the correction and applies its missing named-value rule unchanged in our engine. Joint hand-off must align S-0111 claim 12 and its pip-default description with S-0003 claim 16 and this departure rather than retain a separate pip-default switch.

   **A reset rule outside the rules sections.** One more reset on override belongs to the standing-order processing: a behaviour's anti-air rate is set to its rate as it stands after the layer when a layer contains the behaviour's section and gives no anti-air rate, or gives 0. S-0215, claim 21, owns this rule and its quirk switch, `quirks.anti_air_rate_layer_reset`; it is not part of `quirks.override_resets_keys`.

   **Unset terrain strength takes the game-wide tree strength.** A terrain type begins with strength −1 health points, meaning unset. When a layer contains its section, a strength that is −1 after the layer's integer value has been read becomes the game-wide tree strength in force in that layer, including a value that the same layer gives. Explicit −1 therefore replaces an earlier strength with the current game-wide tree strength, rather than keeping the earlier strength. An omitted key triggers the fill only while the held strength is −1; otherwise it keeps the held strength. An absent section triggers no fill. These are integer health-point values, copied exactly with no rounding. S-0102, claim 23, owns the rule and `quirks.tree_strength_fixed_at_first_section` (default on = this behaviour; off = a type with no explicit strength uses the final layered game-wide tree strength). It is not part of `quirks.override_resets_keys`.

   **Keys that reset to another key's value.** In every layer that has a section for a type with barrel recoil values (S-0105 draws them for buildings), each of its four barrel recoil values (travel, compress, hold and recover) that the layer omits is set to the turret's corresponding value as it stands after that layer, replacing any barrel value an earlier layer gave. S-0105, claim 27, owns this rule, its consequences and its quirk switch, `quirks.barrel_recoil_layer_reset`.

   **Values truncated when re-read.** A building type's light intensity and its three light tints, and the three extra-light values of the audio-visual settings, are held in thousandths. When a layer contains the section but omits such a key, the held value is not kept: it becomes the value that a written value equal to its whole part would give (S-0104, claim 12), the whole part being the held value in whole units, truncated toward zero. For a held value of *t* thousandths, with *k* = trunc(*t* / 1000), the new value *t*′ is 1000 × *k* for the extra-light values; for the building values it is 1000 × *k* when *k* ≥ 0 and 1000 × *k* + 1 when *k* < 0 (Formulas). This is not a reset to a fixed default. A whole extra-light value, and a whole building value that is not negative, is unchanged. A negative building value loses up to a whole unit at every such layer: a written intensity of −3.0 is held as −2999 thousandths, and a later layer that restates the building without the key makes it −1999. For building types it can happen only in a layer after the one that created the type. For the audio-visual settings, which are game-wide, it happens in every layer that contains that section and omits the key, including the base rules of a later game when they omit it, since the held value then comes from the previous game (claim 10). S-0104, claims 12 and 13, owns these values and their quirk switch, `quirks.reread_truncates_light_values`.
   - Backing, row by row:
     - `measurable`: the warhead row (scenario C); the difficulty firepower multiplier (scenario D1); the difficulty rate-of-fire multiplier (scenario D9, which uses the effect that S-0301, claim 3, gives it); the land-type movement multipliers (scenario D2); the two particle-system rows (scenario D3); the particle row (scenario D5); the empty-named particle type reading a later layer's section with the empty name (scenario D4); the crate-effect weight reset (scenario D6); the voxel-animation angular velocities (scenario D7); the voxel-animation sharing row (scenario D8); and the application of the particle-system, voxel-animation and difficulty rows in the base rules layer (scenario D10).
     - `unmeasured`: the converted voxel-animation angular velocities and the rounding of their conversion. No observable of the stored value alone exists before the voxel-animation spec uses it, and scenario D7 compares spin rates in the written unit only (open question 10). The reset to −1 and the comparison with 0 are the measurable parts.
     - `unmeasured`: the parent-country row. S-0413 claim 4 identifies no rule of play using it, so no observable distinguishes resetting from keeping its name (open question 5).
     - `unmeasured`: the difficulty armour multiplier. S-0302 (claim 3 and its scenario D19, part b) shows that no damage rule uses it, so no scenario can tell a reset value from a kept one; the reset is recorded only because the original does it.
     - `unmeasured`: the difficulty ground-speed and air-speed multipliers, repair and build delays, build slowdown, destroys walls and content scan, and land-type buildable. No local scenario isolates their reset. The repair delay participates in original autonomous computer building-repair scheduling, not a player-order delay, and has no effect in our engine (S-0412 claim 5). The build delay is preserved only as a parsed scalar: no consuming gameplay timing or time conversion is specified. Remaining uses are open question 7.
     - `unmeasured`: the difficulty cost and build-time multiplier resets. Prices and build times do not use them (S-0404 claim 2 and S-0403 claim 9), so those observables cannot distinguish reset from retention.
     - `unmeasured`: omitted crate-effect animation reset and water-flag and argument retention. D6 tests weight only; zero weight prevents an ordinary pickup from showing an omitted effect's animation or other parameters. A discriminating scenario is still needed.
     - Which difficulty row a house takes is not decided here. In our engine every house, human or AI, takes the normal row by default (S-0301 and S-0302, claim 3; the glossary's *difficulty skill block*), so for the factors it uses a reset of the normal section matters in every game, and a reset of the easy or difficult section matters only to the optional per-player handicap setting. The original's choice of row by difficulty is set-up behaviour recorded in those claims.
     - The pip-colour omission bug is `measurable` by local AA, with high confidence; retention in our engine is a display-only departure tested against this spec, not an expected original-game result.
     - The terrain-strength fill is `measurable` by local V, with high confidence (S-0102 claim 23).
     - The barrel and light parts are `measurable` by local W and X, including the negative building light value in P. The anti-air-rate reset is `measurable` by Z, with high confidence (S-0215 claim 21).
   - Confidence: high that every row of the table resets as listed. Medium for the visible consequence of the parent-country reset, because it depends on where the parent country is used. Medium that the empty-named particle type reads a later layer's section with the empty name: this follows from the general rules of claim 7 and S-0001, claim 6, and was not checked for this type on its own. The barrel and light parts carry the confidence their own specs give; the +1 for a negative building light value was checked separately here, with high confidence. Medium for the visible consequences given for the power-up and voxel-animation rows, which depend on the crate and voxel-animation specs.

6. **Later layers can add types but not remove or reorder them.** A later layer's type lists add new names at the end of each registry. Re-listing an existing name does nothing. Leaving a name out of a later layer's list does not remove the type. Order and indices follow S-0004, claims 2 to 4.
   - Backing: `measurable` (local Y for overlay append, retention and re-listing). `unmeasured` for other type kinds individually: Y observes overlay indices only, and other kinds need their own observable-index tests.
   - Confidence: high

7. **A type never reads a section from a layer before the one that created it.** In each layer, a type reads its section from that layer only. Which layer it reads first depends on how it came into existence:
   - A type created by a layer's type list, by an early game-wide section (Terms; S-0004, claim 4, row 2), or by a reference that S-0004, claim 6, lets be read in the same layer, reads its section from the layer that created it and from every later layer.
   - A type created by a reference that S-0004, claim 6, leaves unread in that layer reads its section only from the later layers. If the creating layer is the map layer, it reads no section at all and keeps the engine defaults of its kind for every key.

   In no case does a type read the section of the same name in a layer that was applied before it existed. So a map that lists a new type must also give that type's full definition. Keys it omits take the engine defaults for that kind, not values from a base-rules section of the same name.
   - Backing: `measurable` (local E for a listed type and a sampled unread reference). `unmeasured` for remaining individual rank relations: E does not isolate them, as S-0004 claim 6 also records.
   - Confidence: high

8. **Animation properties cannot be overridden by layers.** Animations read their properties from the art file, and the art file is the same for every layer. A game-mode file or map can add an animation name to the animation list. That animation then gets its properties from the art file's section. A rules layer cannot change any animation property.
   - Backing: `measurable` (scenario F)
   - Confidence: high

9. **Projectile speed follows its final layered field.** A weapon's effective projectile speed uses its final layered speed field, not an earlier layer's value. Its observable result is the same whether computed during layering or after layering. This claim does not generalise to every derived property: fills and re-read exceptions follow claim 5, and other derived properties belong to their topic specs.
   - Backing: `measurable` (scenario K)
   - Confidence: medium

10. **Game-wide values and tiberiums carry over between games in one session.** The rules state is kept for the whole session. A game start rebuilds the type registries other than the tiberiums' (S-0004, claim 1), but it does not reset game-wide values or tiberiums. Each layer then changes only what it gives (claims 3 to 5).

    Colour definitions are rebuilt for each game under S-0103 claims 3 and 5, rather than carrying over as game-wide values below.

    **Game-wide values.** Apart from those colour definitions, this holds for every game-wide section (general settings, combat damage, AI settings, audio-visual settings, jump-jet controls, crate rules, radiation, special weapons, multiplayer dialog settings and the like), and for every kind of key in them: numbers, flags, text, lists and type references. Every game-wide key that the base rules give, or a language rules file that passes its check, is restored at every game start. A game-wide key that neither of them gives, but that an earlier game's game-mode layer or map set, keeps that value in every later game of the same session, until a layer sets it again. So the absent-key default of a game-wide key holds only in the session's first game, or until a layer first sets the key. A fresh program start clears the carry-over. The carry-over also reaches a later game in a mode whose file does not give the key: a game in one mode can change the rules of a later game in another mode. A campaign mission's mission-named file leaves game-wide values behind in the same way (claim 14). Stock game-mode files give no game-wide key that the stock base rules lack, and stock data has no mission-named file (S-0002, claim 6), so with stock files a carry-over can come only from a map.

    These game-wide values are restored at every game start whatever earlier games did:
    - the colour-adjustment table, which depends only on this game's layers, never on earlier games (claims 2 and 12),
    - the maximum number of players, which comes from the base rules only (claim 2),
    - the keys of the difficulty and land-type sections in claim 5, and the crate effects' weights and animations of the power-up section, provided the base rules contain each of those sections, as the stock base rules do. The base rules then restore every one of those values, given or reset.

    **Type references in game-wide keys.** A game-wide key can name a type, or a list of types (for example an animation, a weapon or a warhead named in the general or audio-visual settings). If the reference comes only from an earlier game's layer, or from a mission-named file (claim 14), it names a type that the rebuild has deleted. The original keeps a reference to the deleted type, so its result is *unspecified*. This is a *departure*: in our engine such a reference names a type by its name only. Once all layers of the new game are applied, the key refers to the type of that name if the new game has one; otherwise it refers to no type, and a list leaves that entry out. The reference never creates a type. The importer warns about every game-mode file, mission-named file or map that gives, in a game-wide section, a key naming a type or a list of types that the base rules do not give, since only such a key can leave a reference behind.

    **Tiberiums.** Tiberiums are not rebuilt at game start (S-0004, claim 10). At the next game start the base layer re-reads the tiberiums its own tiberium list names (S-0004, claim 10: an unlisted tiberium reads nothing), and every key the base rules give is restored. A key that the base rules do *not* give, but that an earlier game's map or game-mode file set, keeps that value into later games of the same session. A tiberium that only an earlier game's map or game-mode file added stays in the registry, with all its values, for the rest of the session. A mission-named file's tiberium changes behave the same way (claim 14). A fresh program start clears both. Sides do not carry over: they are rebuilt with the other registries (the new sides that a mission-named file creates stay for that mission, claim 14).

    Proposed switches, both default **on** (the original's behaviour):
    - `quirks.persist_global_overrides`. With it off, every game-wide key starts each game from its absent-key default instead of the value the previous game left (in a campaign mission, as the mission-named file then changes it, claim 14), so a game's rules never depend on earlier games of the session. Type references then never outlive their game.
    - `quirks.persist_tiberium_overrides`. With it off, tiberiums are rebuilt from the layers at every game start like other kinds: nothing a tiberium got in an earlier game, from a map, a game-mode file or a mission-named file, reaches a later game, and a key that no layer of this game gives has its engine default (Formulas; claim 14).
    - Backing: `measurable` (scenario O for game-wide values and the restored exceptions; scenario G for tiberiums). Scenario O's difficulty check uses a key that the base rules give, so for the difficulty and land-type exception it confirms only that given keys are restored. That the base rules also reset omitted keys is `measurable` for the difficulty row by D10c; for land-type and power-up rows it is `unmeasured` here: D2 and D6 show map-layer resets only, not base-layer resets after carry-over. `unmeasured` also covers carry-over of unsampled game-wide flags, text and lists and newly added tiberium types: O and G do not isolate these cases, so further consumer-specific scenarios are required. The type-reference case is `unmeasured`: the original's result is *unspecified*, supplying no stable expected observable; our engine's choice is a design decision.
    - Confidence: high for game-wide values and the restored exceptions; medium for tiberiums

11. **The game-mode file is fixed at program start.** For the whole session, every game in a mode uses that mode's file as it was at program start. This holds for every mode, whether or not it has been played yet. Edits to the file while the program runs have no effect until a restart. If the file named by the mode does not exist at program start, the mode layer is empty and changes no rules value for the whole session (it is still an applied layer, claim 1).
    - Backing: `measurable` (scenario H)
    - Confidence: high

12. **The language rules file needs a valid digest.** The language rules file is optional; stock data has none. A file that holds at least one section is applied only if it carries a valid integrity digest section. What makes a digest valid (what the checksum covers, its algorithm and how it is written) is given by S-0005, claims 21 and 21b. A file that holds no section at all, an empty file for example, is accepted without any check and changes no rules value; it is still an applied layer (claim 1). It never triggers either case below.

    A file that holds at least one section but fails the check has these effects:
    - At program start, the original refuses to start: start-up fails and the main menu is never reached. This is a *departure*, with no switch: our engine starts as though the file were absent at program start, and the importer warns. The file is still checked again at every game start, as below, so while it keeps failing, every game takes the game-start result.
    - At game start, the file is resolved and checked again for every game (S-0005, claim 21). So this case arises when a file that fails the check first appears after program start, and also when a file that was valid at program start has since been edited or replaced so that it fails. For that game the file is ignored, and so is the whole game-mode layer. So every animation has one type read fewer than claim 1 gives (for an animation that the base rules' animation list names, two instead of three), which S-0007, claim 2, makes visible. The colour-adjustment table is left with zero in all 16 positions, so only the entries the map layer gives (claim 2) are non-zero. Game-wide values are not affected otherwise: they keep the values held (claim 10), apart from what the base rules and the map give.

    In both cases the importer warns. Proposed switch: `quirks.bad_language_rules_skips_mode` (default **on**). It covers only the game-start case. With it off, a language rules file that fails the check at game start is ignored, the game-mode layer is still applied, and the colour-adjustment table is filled from the base rules as in claim 2. The program-start case has no switch; it is the departure above. The behaviour is irrelevant for stock data.
    - Backing: `measurable` (scenario I, which measures the original, including its start-up failure; I4 for the type read lost with the game-mode layer). Our engine's program-start result is a departure, a design decision.
    - Confidence: medium

13. **The mode list names each mode, its kind and its game-mode file.** The mode list is read once, at program start (S-0005, claim 17). It holds six fixed sections, one for each mode kind (Import mapping), whose names must match exactly, letter case included (S-0001, claim 13). Its other sections are ignored. Every entry of those six sections is a candidate mode, including every repeat of a key. A key written twice in one section therefore gives two candidates, and both carry the single value that S-0002 (claims 1 and 4) gives that key, so they are identical.

    **Mode number.** The entry's key, read as a decimal integer in the decimal form of S-0003, claim 3: leading whitespace and a sign are allowed, reading stops at the first non-digit, a key without leading digits gives 0, and overflow wraps (S-0003, claim 4). The `$` and `h` forms are not recognised here, so `$5` gives 0 and `5h` gives 5.

    **Items of the value.** The value is split at commas. Empty items are skipped, as in S-0003, claim 12: consecutive commas count as one separator and leading commas are ignored, so an empty item moves every later item one place earlier than it is written (the third written item counts as the second, and so on). Unlike a list value, each item has its leading and trailing spaces removed. Only the space byte is removed; a tab or any other byte stays part of the item. The items are, in order:

    | Item | Meaning | Needed for the mode to be listed |
    |---|---|---|
    | 1 | the label of the mode's name in the string table | yes; an item made only of spaces counts as present |
    | 2 | the label of the mode's tooltip in the string table | yes; as item 1 |
    | 3 | the name of the game-mode file | yes, and it must not be empty once its spaces are removed |
    | 4 | the map filter (its use belongs to the map selection spec) | yes, as item 3 |
    | 5 | a flag, off only when the item is `false`, ignoring case; on when it is anything else or absent (its use belongs to the lobby spec) | no |

    Items after the fifth are ignored. An entry that lacks a needed item is not listed. The original drops it silently; the importer warns.

    A label that the string table lacks does not stop the mode from being listed: the name or tooltip then shows the string table's placeholder text for a missing label.

    These item rules, with the mode number above, are the format of the mode list. They define how every mode list is read, stock data and mods alike, in the way S-0003 defines the value forms of rules keys, and they are not quirks: there is no switch. Where a file depends on them in a way an author may not expect, the importer warns (Bad data below).

    **The game-mode file.** Item 3 is used exactly as written, once its spaces are removed: no extension or folder is added. It is resolved like any other data file (S-0005, including its case-insensitive name lookup), at program start (claim 11). If it cannot be resolved, the mode is still listed and its game-mode layer is empty (S-0005, claim 19). No spelling of item 3 means "no file": an entry without a file name is not a mode at all. Two modes that name the same file both get that file's contents as they were at program start (claim 11), and nothing else about them is linked.

    **Order and identity.** The lobby lists the modes in ascending order of mode number. Modes with equal numbers keep their reading order: sections in the order of the Import mapping, and within a section the entry order. A game identifies its mode by its mode number. Where several listed modes share a number, the number always refers to the first of them in list order, so choosing one of the later ones plays the first one's kind and game-mode file. Proposed switch: `quirks.mode_number_first_wins` (default **on** = as just stated). With it off, a game identifies its mode by the mode's place in the list rather than by its number, so choosing any listed mode plays that mode's own kind and game-mode file. The list and its order, the default mode and the remembered-mode rule above are the same either way, and a remembered number still selects the first listed mode with that number. Where no two listed modes share a number, both settings give the same result.

    **Default mode and the remembered mode.** The skirmish screen preselects a mode, and that mode is the one played if the player does not choose another. The preselected mode comes from a number *r* that the player's settings file remembers for skirmish games (Import mapping). The rule that turns *r* into a mode is a rule of this spec, because it decides which game-mode file the game uses. With *M* the count of listed modes, and the *first listed mode* the first in list order, which is the one with the lowest number:

    | Remembered number *r* | Preselected mode |
    |---|---|
    | none (the settings file remembers no number) | the first listed mode |
    | *r* < 1, or *r* > *M* (the bound is the count of listed modes, not the highest mode number) | the first listed mode |
    | 1 ≤ *r* ≤ *M*, and at least one listed mode has number *r* | the first listed mode, in list order, that has number *r* |
    | 1 ≤ *r* ≤ *M*, and no listed mode has number *r* (a gap in the numbering) | the first listed mode |

    The preselected mode is also the one whose number the screen remembers from then on. When listed modes share a number, the number refers to the first of them (see above), so the remembered number cannot select a later one.

    How the screen shows the preselection is for the interface spec. Network and online lobbies are outside this spec: the network games are out of scope (S-0201, claim 13).

    **Bad data.** The importer warns about every entry that is not listed, about mode numbers that several modes share, about mode numbers that are not exactly 1 to *N* for *N* listed modes (the original's remembered-mode check compares the remembered number with the count of listed modes, and its other uses of mode numbers assume that numbering), about an entry with an empty item (consecutive or leading commas), about an item that begins or ends with a tab or another whitespace byte other than the space, which stays part of the item, about a fifth item that is neither `true` nor `false` (ignoring case), and about an entry with more than five items.
    - Backing, per item:
      - `measurable`: the items, the empty item, the dropped entries, a missing label, a tab at the edge of an item, the order, shared numbers, the default mode and the remembered-mode rule (scenario Q, parts Q1 to Q11); the `$` and `h` spellings of the mode number, the overflow of the mode number, the ignoring of items after the fifth, a section name of the wrong letter case, and a key written twice (scenario Q12, which uses its own copy of the mode list); a mode's file being read at program start (local H). The fifth item's flag is `unmeasured`: its consumer is outside this spec and Q6 is exploratory, not an established falsifying test. Importer warnings are design requirements, not original-game measurements.
    - Confidence: high for the sections, the mode number, the items and which entries are listed, the order and the default mode. High for the rule that turns a remembered number into a mode (backing `measurable`, scenario Q8, not yet run). Medium for what a shared mode number does in the lobby.

14. **A campaign mission's mission-named file acts like a layer of an earlier game.** At the start of every campaign mission, and of no other game, the mission-named file (Terms) is resolved like any other data file (S-0005). If it exists, it is applied as a rules file with the effects below, but it is not one of the mission's rules layers (claim 1). Stock data has no such file (S-0002, claim 6).

    Its effects are exactly those that a layer of the previous game of the session would have left (claim 10), plus its sides:
    - **Game-wide values.** Each game-wide key it gives takes its value. The base rules, a valid language rules file and the map then apply as usual, so the mission-named file's value is in force in the mission only where none of them gives the key, and it carries into later games of the session like any game-wide value (claim 10). The values that claim 10 restores at every game start (the colour-adjustment table, the maximum number of players, and the reset keys of the difficulty, land-type and power-up sections while the base rules contain those sections) never keep its values.
    - **Tiberiums.** Its tiberium list and tiberium sections act as in any layer (S-0004, claim 10), and the result survives the rebuild: a tiberium it adds stays for the rest of the session, and a key it sets that no later layer gives stays in force. With `quirks.persist_tiberium_overrides` off the rebuild deletes the tiberiums like any other kind, so every tiberium change of the mission-named file is lost, and the importer warns about a mission-named file that holds a tiberium list or a tiberium section, as it does for other type data; its game-wide values still carry over. In the session's first game every entry of its tiberium list creates a tiberium, and these tiberiums take the lowest indices, ahead of any that the mission's layers create; the base rules' tiberium list then uses them by index, whatever names they carry (S-0004, claim 10).
    - **Other types.** Every change it makes to any other type kind (its type lists, its type sections, and types that its game-wide keys create by reference) is lost: the rebuild deletes those types. The new sides of the next bullet are the one exception. None of the rest has any effect in the mission, and it gives no animation a type read that counts in the mission (claim 1).
    - **Sides.** No side of the previous game reaches the mission. A side that the mission-named file's side list or a country's side field creates remains at this game start, comes before the base rules' sides in side order, and stays for the mission (S-0004, claim 9). This is the deferred campaign exception recorded in S-0004, claims 1 and 9, and in claim 1 above. Such a side's member list can name only the previous game's countries, which the rebuild deletes; in the original those entries stay in the list, and what they denote in the mission is *unspecified*. This is a *departure*: our engine empties the member list of every side at the rebuild, so that only the side lists and country side fields of this game's layers fill it, as S-0004, claim 9, gives. This is bad data that stock data never has; the importer warns about every mission-named file that has a side list or a country section.

    The importer also warns about every mission-named file that holds a type list other than the tiberium list and the side list, or a type section other than a tiberium's, since in the original none of it takes effect. For the tiberium list and sections, and for the side list, the importer warns as the bullets above give.

    Proposed switch: `quirks.mission_rules_before_reset` (default **on** = as above). With it off, the mission-named file is applied as an ordinary rules layer after the language rules file and before the map layer, in the place that the game-mode layer takes in other games, so that all its changes take effect in the mission, its sides follow S-0004, claim 9, like any layer's, and it gives animations a type read. Its game-wide values and tiberiums still carry over like any layer's (claim 10).
    - Backing: `unmeasured` in this milestone: these effects require campaign execution, outside scope. Deferred U proposes a future campaign test. The member-list departure and switch-off alternative are design decisions; the original's member-list result is *unspecified*.
    - Confidence: high that every rules layer of the mission ranks above it and that its type changes are lost; medium for the game-wide and tiberium consequences and for the side order.

## Formulas

The layered value *v* of a key *k* in a section *s*, after layers *L₁ … Lₙ* (base first, map last):

> *v* = the value of *k* in *s* in the last layer *Lᵢ* that gives *k* inside section *s*. If no layer of this game does, *v* = *v₀*, the value held at the start of the game:
> - for a type section other than a tiberium's, *v₀* is the engine default of *k* for the type's kind,
> - for a game-wide section, *v₀* is the value *k* had at the end of the previous game of the session (claim 10), and the engine default of *k* in the session's first game,
> - for a tiberium, *v₀* is likewise the value at the end of the previous game, or the engine default for a tiberium first created in this game (with `quirks.persist_tiberium_overrides` off, always the engine default; below),
> - in a campaign mission, for a game-wide key and (while `quirks.persist_tiberium_overrides` is on) for a tiberium, *v₀* is the value after the mission-named file has been applied to the value just described (claim 14), when that file exists.
>
> A key written with its "not given" value (claim 3) does not count as given.
>
> **Reset-on-override keys.** For a reset-on-override key, a layer that has no section for the key's section leaves the held value unchanged. In a layer that has the section, an ordinary value replaces the held value, and the key's “not given” value keeps the held value. If the section is present and the key is absent, the key takes its reset value. For barrel recoil, that reset value is the turret's corresponding value after the same layer (S-0105, claim 27). Light values truncated on re-read follow the light formula below instead.
>
> | Section in this layer | Key in this layer | Result |
> |---|---|---|
> | absent | any | held value |
> | present | ordinary value | given value |
> | present | “not given” value | held value |
> | present | absent | reset value |
>
> With `quirks.override_resets_keys` off, an absent key in a present section keeps the held value instead of taking its reset value. The held value at the start of layering is the value specified for *v₀* above. The switch does not change the “not given” rule or the separate barrel-recoil and light-value rules.
>
> With `quirks.persist_tiberium_overrides` off, every tiberium is rebuilt from the layers at each game start: *v₀* for a tiberium key is the engine default of *k* in every game, whatever earlier games left; a tiberium first named by this game's layers is created in this game, the tiberiums of earlier games are deleted, and so are the tiberiums that only an earlier game's layers or a mission-named file added. The mission-named file does not enter *v₀* of a tiberium either (claim 14). The indices of the tiberiums then follow S-0004, claim 10, as in a session's first game.
>
> With `quirks.persist_global_overrides` off, the value that the previous game left is replaced by the engine default for every game-wide key, so *v₀* is the engine default, or in a campaign mission the value after the mission-named file has been applied to it. With `quirks.mission_rules_before_reset` off, the mission-named file does not enter *v₀*; it is one of the layers *L₁ … Lₙ*, after the language rules file (claim 14).

For a type section, the layers considered are only those in which the type reads its section (claim 7). For a tiberium, only the layers whose tiberium list has an entry for it count (S-0004, claim 10). Colour definitions follow S-0103 claim 5 instead of this formula: the first definition of a name wins by default (claim 2). The colour-adjustment table follows claims 2 and 12 instead of this formula, and so does the maximum number of players (base rules only). Terrain type strength follows claim 5's unset-strength fill instead: after any layer containing its section, a resulting integer strength of −1 health points becomes that layer's game-wide tree strength. This exact copy uses no arithmetic or rounding.

Layering selects values. Its only arithmetic is the truncation of the light values in claim 5, where *t* is the held value in thousandths, an integer, and *k* = trunc(*t* / 1000) is an integer division rounding toward zero:

> *t*′ = 1000 × *k* for the extra-light values. For the building light values, *t*′ = 1000 × *k* when *k* ≥ 0 and *t*′ = 1000 × *k* + 1 when *k* < 0 (the conversion of a written value of S-0104, claim 12, applied to *k* whole units). Units: thousandths.

For the building light values, this formula assumes that S-0104 claim 12 converts a written value *v* as trunc(*v* × 1000 + 0.1), with truncation toward zero. S-0104 claim 12 must be aligned to state that rule in the same hand-off. Under that conversion, a written whole-unit value gives the integer results above regardless of how the floating-point operations round, so they do not depend on the round-toward-zero mode. No other rounding applies. Each value keeps the number domain of its key, stated in the topic specs. The reset values 1.0, 0.02 and 0.03 of the difficulty sections, 100 % for warhead modifiers and 100 % for land-type multipliers are floating-point numbers in the original (100 % is stored as 1.0).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| armour classes per warhead | 11 | classes | all reset to 100 % (claim 5) |
| land-type sections | 12 | sections | claim 5; named in the Import mapping |
| difficulty levels | 3 | levels | claim 5 |
| difficulty multiplier reset | 1.0 | ratio (factor) | floating point in the original |
| difficulty repair delay reset | 0.02 | game minutes | Double precision. Used in original autonomous computer building-repair scheduling; not a player-order delay or a rule of our engine (claim 5). |
| difficulty build delay scalar reset | 0.03 | dimensionless imported scalar | Double precision. Preserves the parsed scalar and reset, not a duration; no gameplay consumer or time conversion is established (claim 5). |
| land movement multiplier reset | 1.0 (100 %) | ratio (factor) | floating point |
| warhead modifier reset | 1.0 (100 %) | ratio (factor) | floating point |
| colour-adjustment table size | 16 | entries | claim 2 |
| colour-adjustment empty entry | 0, 0, 0 | red, green, blue components | adds no tint (claims 2 and 12) |
| "not given" value of the pitch and roll angles | −1 | degrees | claim 3 |
| "not given" value of an animation's rate and random rate | −1 | the unit in which the key is written | claim 3 |
| "not given" value of a superweapon's recharge time | 0 | game minutes | claim 3 |
| "not given" value of a voxel animation's angular velocities | 0 | the unit in which the key is written | claim 3; compared in the written unit before angular-unit conversion |
| voxel-animation angular velocity reset | −1 | the unit in which the key is written | claim 5; conversion to an internal angular unit is outside this spec |
| voxel-animation angular velocity engine defaults before any reading | 0 (minimum), 10 (maximum) | the unit in which the key is written | claim 5; not the reset value, which is −1 |
| crate effects in the power-up section | 19 | effects | claim 5 |
| crate effect weight reset | 0 | relative-weight units (dimensionless integer) | Not a percentage, duration or effect count; zero excludes an effect from the weighted choice (claim 5). |
| mode-list sections (mode kinds) | 6 | sections | claim 13 |
| items per mode-list entry | 4 needed, 5 read | items | claim 13 |

## State and ordering

| State | Event | Result |
|---|---|---|
| any | game starts | every type registry except the tiberiums' holds only the types created for this game; game-wide values and tiberiums are those the previous game left (claim 10), as changed by the mission-named file in a campaign mission (claim 14) and then by this game's layers; the colour-adjustment table holds only what claim 2 gives. The layers, from lowest to highest precedence: base, language (if present and it passes its check, or holds no section), game-mode layer (every multiplayer and skirmish game, empty when the mode's file cannot be resolved; skipped if a language rules file with sections fails its check, claim 12), map |
| ordinary key given in a later layer | layer applied | value replaced |
| colour name already defined, ignoring letter case | later layer applied | earlier definition and its position kept by default (claim 2; S-0103 claim 5) |
| key absent, section present, ordinary key | layer applied | held value kept |
| key given with its "not given" value (claim 3), section present, any key | layer applied | held value kept |
| key absent, section present, reset-on-override key | layer applied | value set to its reset value (a fixed value, or the turret's value for a barrel value) |
| key absent, section present, light value of claim 5 | layer applied | held value cut to a whole unit as claim 5 gives |
| terrain type section present, strength is −1 after reading the layer's value | layer applied | strength becomes the game-wide tree strength in force in that layer, including any value that layer gives (claim 5; S-0102, claim 23) |
| section absent | layer applied | everything in it kept |
| any | program starts | the mode list is read and every listed mode's file contents are fixed for the session as they are then (claims 11 and 13) |
| campaign mission, mission-named file exists | game starts | the file acts like a layer of the previous game: its game-wide values and tiberium changes take effect where no rules layer of the mission gives the key, apart from the values claim 10 restores at every game start, and carry over to later games; its new sides stay and come before the base rules' sides; its other type changes have no effect (claim 14, `quirks.mission_rules_before_reset`) |
| language rules file with sections fails its check | program starts | original: start-up fails; our engine: starts as though the file were absent, importer warns (claim 12) |

Observable ordering:
- Where they give the same key, a map's values win over the game-mode file's, and the mode file's over the base rules', except the maximum number of players, which only the base rules give, and a tiberium's section in a layer whose tiberium list has no entry for it, which is ignored (claim 2).
- A valid language rules file's values beat the base rules' (except at the colour-adjustment table positions that the base rules give, claim 2) and lose to the game-mode file's and the map's. Colour definitions are another exception to both override statements: by default the first definition of a name wins, ignoring letter case (claim 2; S-0103 claim 5).
- Player houses see the final country values of all layers.
- The lobby lists the modes in ascending mode number; a mode number shared by several modes refers to the first of them (claim 13, `quirks.mode_number_first_wins`).

## Edge cases and quirks

- `quirks.colour_redefinition_ignored`: S-0103 claim 5 owns the colour-definition exception and both switch settings (claim 2).
- `quirks.override_resets_keys`: claim 5.
- Pip-colour omission: fixed without a switch as a display-only departure (claim 5); it is not controlled by `quirks.override_resets_keys`.
- `quirks.persist_global_overrides`: claim 10.
- `quirks.persist_tiberium_overrides`: claim 10; its effect on a tiberium's starting value is in the Formulas and on the mission-named file in claim 14.
- `quirks.override_resets_keys` off: an omitted key keeps the held value. The engine default applies only if no earlier applicable layer or carried-over game supplied a value (claim 5), not as an unconditional reset.
- `quirks.bad_language_rules_skips_mode`: claim 12.
- A language rules file that fails its check at program start: the original refuses to start; our engine starts without it and warns, a *departure* without a switch; see claim 12.
- `quirks.barrel_recoil_layer_reset` and `quirks.reread_truncates_light_values`: owned by S-0105 and S-0104 (claim 5).
- `quirks.tree_strength_fixed_at_first_section`: owned by S-0102, claim 23; the unset-strength fill of claim 5 is separate from "not given" values and from `quirks.override_resets_keys`.
- A colour-adjustment section with more than 16 entries: the original overwrites other rules values with the extra entries, with an *unspecified* result; our engine ignores the extra entries and the importer warns, a *departure* (claim 2).
- A game-wide type reference left over from an earlier game or from a mission-named file: the original keeps a reference to a deleted type, with an *unspecified* result; our engine matches it by name against the new game's types and the importer warns about the keys that can cause it, a *departure* (claim 10).
- A negative building light value loses up to a whole unit at every later layer that restates the building without that key (claim 5); S-0104 owns the switch.
- A power-up section that lists only some crate effects gives every other effect weight 0 and no animation (claim 5); covered by `quirks.override_resets_keys`.
- A voxel animation's angular velocities: 0 means "not given", and an absent key sets −1 (claims 3 and 5); the reset is covered by `quirks.override_resets_keys`, and with it off an absent key keeps the held value.
- `quirks.mode_number_first_wins`: claim 13.
- The mode list's format (claim 13): an empty item moves every later item one place earlier than it is written, only the space byte is removed around an item, a fifth item other than `false` (for example `no`) leaves the flag on, and an entry that lacks a needed item is not listed. These are not quirks: they define the file format, as the value forms of S-0003 do, and have no switch. The importer warns as claim 13 (Bad data) lists.
- A side that a mission-named file creates keeps member-list entries that name deleted countries in the original, with an *unspecified* result; our engine empties every side's member list at the rebuild and the importer warns, a *departure* (claim 14).
- `quirks.mission_rules_before_reset`: claim 14. A campaign mission's mission-named file acts like a layer of the previous game, so only its game-wide values, its tiberium changes and its new sides take effect; the importer warns about its type data (its tiberium data only with `quirks.persist_tiberium_overrides` off), side list and country sections. Stock data has no such file (S-0002, claim 6).

## Import mapping

| Layer | Source |
|---|---|
| base rules | `RULESMD.INI` (which copy wins: S-0005, including the start-up picker of S-0005 claim 20) |
| language rules | `LANGRULE.INI`: a file with at least one section only with a valid `[Digest]` section (S-0005, claims 21 and 21b); a file with no section is accepted and changes nothing |
| game-mode layer | the game-mode file named by item 3 of the selected mode's entry in `MPModesMD.INI` (claim 13 and the mode-list table below); an empty layer when the file cannot be resolved |
| map layer | the map file being played |
| mission-named file (campaign missions only, claim 14) | the mission's map file name with its extension replaced by `.INI` (S-0002, claim 6), resolved as S-0005 gives; stock data has none |
| animation properties | `ARTMD.INI`, the same in every layer; T uses `Image`, `Theater` and `NewTheater` as mapped by S-0007 |

Keys and sections named in this spec:

| Our name | Rules key or section |
|---|---|
| maximum number of players (base rules only) | `[Maximums]` `Players` |
| colour-adjustment table | `[ColorAdd]`: each entry's value is red, green, blue; key names are ignored, only the entry's position counts |
| colour-definition section | `[Colors]`: each key names a definition, compared ignoring case; value parsing and defaults are in S-0103 Import mapping. An existing name keeps its earlier definition by default (claim 2; S-0103 claim 5) |
| parent country | `ParentCountry` in a country's section; when absent in a layer that has the section, it becomes the country's own name |
| warhead damage modifiers | `Verses` in a warhead's section; when absent in a layer that has the section, `100%` × 11 |
| difficulty sections | `[Easy]`, `[Normal]`, `[Difficult]` |
| AI intelligence levels (an early game-wide section, Terms) | `[IQ]` |
| difficulty keys and reset values | `FirePower`, `Groundspeed`, `Airspeed`, `Armor`, `ROF`, `Cost`, `BuildTime` = 1.0; `RepairDelay` = 0.02 game minutes; `BuildDelay` = 0.03 as an imported scalar only. Numeric entries use S-0003's decimal/percent form, held in double precision. No build-delay duration conversion is specified. `BuildSlowdown` = no; `DestroyWalls` = yes; `ContentScan` = no, all using the boolean form |
| land-type sections | the 12 land-type sections, a list fixed in the engine: `[Clear]`, `[Road]`, `[Water]`, `[Rock]`, `[Wall]`, `[Tiberium]`, `[Beach]`, `[Rough]`, `[Ice]`, `[Railroad]`, `[Tunnel]`, `[Weeds]`. Each is a game-wide section; `[Tiberium]` here is the land type, not a resource type. Which terrain uses which land type belongs to the movement spec |
| land-type keys and reset values | `Foot`, `Track`, `Wheel`, `Hover`, `Float`, `Amphibious`, `FloatBeach` = 100%; `Buildable` = no |
| particle-system keys that reset | `HoldsWhat` (empty name; only an explicit `<none>` avoids it, S-0004, claim 2, where `none` also names a particle type), `BehavesLike` (none) |
| particle key that resets | `BehavesLike` (none) |
| infantry pip-colour omission (claim 5) | `Pip` and `OccupyPip`: named values using S-0111's Import mapping vocabulary, ignoring ASCII case. An absent key keeps the held value; initial defaults are image frames 1 and 7 respectively. An unrecognised word gives frame 1. The original's omitted-key shift is fixed without a switch |
| owner lists (example of a list key) | `Owner`, `RequiredHouses`, `ForbiddenHouses`, `SecretHouses` |
| prerequisites (example of a list key) | `Prerequisite` |
| crate-effect keys of the power-up section (claim 5) | `Money`, `Unit`, `HealBase`, `Cloak`, `Explosion`, `Napalm`, `Squad`, `Darkness`, `Reveal`, `Armor`, `Speed`, `Firepower`, `ICBM`, `Invulnerability`, `Veteran`, `IonStorm`, `Gas`, `Tiberium`, `Pod`. Each value is weight, animation, whether it can appear on water (`yes` or `no`), argument. When a key is absent in a layer that has the section, it reads as `0,NONE`: weight 0 and the animation named `NONE`, which is no animation unless the data defines an animation type of that name (matched ignoring case; the lookup never creates one); the water flag and the argument keep the held value |
| "not given" keys (claim 3) | `PitchAngle` and the roll-angle key, whose name is `PitchAngle` with `Pitch` replaced by `Roll`, letter case as shown (−1 each), in a building, vehicle, aircraft or infantry section; `RechargeTime` (0) in a superweapon's section; `MinAngularVelocity`, `MaxAngularVelocity` (0) in a voxel animation's section; `Rate` (−1) and both numbers of `RandomRate` (−1 each) in an animation's section of `ARTMD.INI`. No key of a game-wide section has one |
| voxel-animation keys that reset (claim 5) | `MinAngularVelocity`, `MaxAngularVelocity` (−1 when absent); `ShareBodyData`, `ShareTurretData`, `ShareBarrelData`, read as flags with absent = no, and the voxel animation shares another model's data when at least one of them is yes |
| barrel recoil values that reset to the turret's (S-0105, claim 27) | `BarrelTravel`, `BarrelCompressFrames`, `BarrelHoldFrames`, `BarrelRecoverFrames`, reset to `TurretTravel`, `TurretCompressFrames`, `TurretHoldFrames`, `TurretRecoverFrames` |
| terrain strength and its game-wide fill (S-0102, claim 23) | `Strength` in a terrain type's section: integer health points, initially −1 (unset). When its section is read and the resulting strength is −1, it takes `[General]` `TreeStrength` as then held, an integer in health points. The latter's absent-key default and persistence are in S-0102, claim 23 |
| building light values truncated on re-read (S-0104, claim 12) | `LightIntensity`, `LightRedTint`, `LightGreenTint`, `LightBlueTint` in a building type's section |
| extra-light values truncated on re-read (S-0104, claims 12 and 13) | `ExtraUnitLight`, `ExtraInfantryLight`, `ExtraAircraftLight` in `[AudioVisual]` |
| mode-list sections, one per mode kind, in reading order (claim 13) | `[Battle]`, `[ManBattle]`, `[Siege]`, `[Unholy]`, `[FreeForAll]`, `[Cooperative]` in `MPModesMD.INI` |
| the remembered skirmish mode (claim 13) | key `GameMode` in the `[Skirmish]` section of the player's settings file `RA2MD.INI`: the mode number last chosen; absent, the number of the first listed mode |
| a mode-list entry (claim 13) | key: the mode number (decimal); value: up to five comma-separated items, name label, tooltip label, game-mode file name, map filter, flag (`false` = off) |

## Measurement scenarios

The colour-definition exception in claim 2 uses S-0103 scenario PF: its named map, game-mode and valid language test files re-define the non-player houses' colour with exact and mixed-case names. Its pixel recording, reference-ramp expectation and falsifier apply here; no completed measurement is claimed.

All active scenarios use game speed 2, one human-controlled test house and one bystander computer player isolated from the observed area, unless a scenario requires the computer house's difficulty row. The observed area consists of all cells containing the named test objects and their paths or effects. Void any run with an unnamed bystander object entering or affecting that area, or bystander defeat before observation ends. Director and mods are off apart from each scenario's named test data; the executable remains stock. A video capture, scripted input and the read-only recorder may be used as described; no patched launcher is required.

Every scenario runs under the glossary's *measurement setup*, with stock rules, art and mode files. Overrides go in the rule sections of a purpose-made test map in the multiplayer map folder, never in the stock files. Scenarios D1 and D9 (only where they say so), D4, D10, E, H, I, P, Q, R, S, T, W, X, Y and Z need named test files and say so. U is deferred campaign context, not a skirmish run. A loose test file is a named test file of the measurement setup, placed in a scratch copy of the installation, used only by the scenario that names it, and removed with the copy afterwards. A scenario that needs a loose copy of the base rules runs its test map and its control on that same copy, so that the two runs differ only in what the scenario says. Tick counts and damage may be read with a recorder that reads the running game's state, where one fits. A skirmish on a fixed map with fixed start positions is deterministic for what these scenarios record, so each test and control run is done once, then repeated once to confirm; a disagreement between the two repeats voids the run. Game-wide values carry over between games of a session (claim 10), so a run that follows a game on a map that gives a game-wide section can inherit that map's values. Every run, and every confirming repeat, therefore starts from a fresh program start, unless the scenario itself prescribes a sequence of games in one session (scenarios D10c, G, H, O, Q and U1).

**A. Keep, replace and list replacement (claims 1 to 4).**
- Setup: in the test map, override one numeric key of one vehicle type (its health), and replace that vehicle's prerequisite list with a single building it doesn't normally need. Leave all its other keys out.
- Observable: the vehicle's health, read from the health bar or with a recorder that reads the running game's state; whether its normal prerequisites still matter; one key left out, such as its speed in cells per tick on clear ground.
- Expected: health as overridden; only the new prerequisite is required; the left-out key equals the base value.
- Falsified by: the left-out key changing, the health not changing, or the old prerequisites still being required.

**B. Country overrides reach the players (claim 1).**
- Setup: in the test map, override a country's cost multiplier for one category with a value clearly different from the base value. Control: the same item bought on a stock map by the same country.
- Observable: the price of an item of that category, in credits, for a human player of that country.
- Expected: the price reflects the override on the test map and the base multiplier on the control map.
- Falsified by: equal prices on both maps.

**C. Warhead modifier reset (claim 5).**
- Setup: in the test map, give a warhead's section a single key that doesn't affect damage, for example a visual flag. Pick a warhead whose base modifier against some armour class is below 100 %.
- Procedure: in a controlled engagement, have a weapon with that warhead fire at a target of that armour class. Run on the test map and on a control map without the section.
- Observable: health lost per shot.
- Expected: full, unmodified damage on the test map; reduced damage on the control map.
- Falsified by: equal damage on both maps.

**D. Difficulty, land-type and particle resets (claim 5).**
- D1 setup: a test map with a difficulty section holding one key only, one that does not affect damage (for example the destroys-walls flag). Check the base rules and choose a difficulty level whose firepower multiplier is not 1.0. A house that takes that level's row (in the original, a computer house at that lobby difficulty; for the normal level, a human-controlled house, which the original gives the normal row) fires a weapon at a fixed target in a controlled engagement; control: the same on a stock map. The country's firepower factor must be 1.0: check the base rules, and if it is not 1.0 for that country, both the test run and the control run use one loose copy of the base rules (a loose test file) in which only that factor is changed to 1.0.
- D1 observable: health lost per shot from the weapon of the house that takes that level's row.
- D1 expected: on the test map, the damage of a firepower multiplier of 1.0 (the same damage the weapon does for a house whose multiplier is 1.0); on the control map, damage scaled by the base multiplier.
- D1 falsified by: equal damage on test and control maps.
- D9 (the rate-of-fire multiplier, claim 5; S-0301, claim 3 and scenario F2). Setup: as D1, with the one-key difficulty section on the test map and the same stock-map control, for a level whose rate-of-fire multiplier is not 1.0 in the base rules; if every level has 1.0, both the test run and the control run use one loose copy of the base rules (a loose test file) that gives the level a rate-of-fire multiplier of 0.5 and leaves every other key as its base value (and, if the country's rate-of-fire factor is not 1.0, sets it to 1.0 in the same copy). The house that takes that level's row is as in D1 (the normal level with a human-controlled house is the simplest), the country's rate-of-fire factor is 1.0, and a stationary enemy target of high strength stands in range of a unit with a single-shot weapon (burst count 1) whose rate-of-fire field is 60 or more ticks, so that a change of the multiplier is far larger than the ranged draw of one tick. Run the game at one game speed.
- D9 observable: shot ticks over 20 shots, and the recorder's draws of S-0301, scenario F1, from which each interval is compared with the full delay *D*₁ of S-0301, Formulas F1.
- D9 expected: on the test map every interval is that of a rate-of-fire factor of 1.0, the field plus the draw; on the control, every interval follows the base (or the loose copy's) multiplier.
- D9 falsified by: equal intervals on both maps (a map that gives one key of the section keeps the held rate-of-fire multiplier instead of resetting it).
- D2 setup: check the base rules and choose a land type and a locomotion class whose base movement multiplier is below 100 %. If no such pair exists, D2 cannot discriminate and is skipped. A test map gives that land type's section only the buildable key; a control map has the same terrain without the section. A vehicle of that locomotion class crosses a straight strip of that land type, of fixed length.
- D2 observable: the crossing time, in ticks.
- D2 expected: on the test map, the time at a 100 % multiplier, shorter than on the control map.
- D2 falsified by: equal crossing times on both maps.
- D2 is run once for each of the 12 land-type section names of the Import mapping for which the base rules give a suitable pair, so that it also confirms each name. A name for which the test map's section changes nothing, although a suitable pair exists, falsifies that name.
- D3 setup: a test map that gives a visible particle system (for example a smoke trail) a section with one unrelated key only. Control: the same event on a stock map.
- D3a isolates the emitted-type reset: repeat D3 but explicitly retain the usual system behaviour in the map. D3b isolates the behaviour reset: repeat D3 but explicitly retain its usual emitted particle type. Both controls have the usual type and behaviour.
- D3 observable: whether the system draws any particles, from a recording, and whether the game keeps running. Apply the same observation, expectation and falsifier separately to D3a and D3b.
- D3 expected: no visible particles on the test map, including D3a and D3b (respectively no image on the empty-named particle and no system behaviour); the control draws its usual particles.
- D3 falsified by: the usual particles appearing on the test map. If the game stops instead, record that: the claim's reset stands, but its visible consequence must be revised.
- D3c setup (the empty value, claim 3): a test map as in D3 whose section for the system holds the one unrelated key and, besides it, the particle-type key written with an empty value. Observable and expected: as D3 (no visible particles), because the empty value is an absent key: it neither keeps the held value nor means the explicit no-type value. Falsified by: the usual particles.
- D3d setup (a section of empty lines only, claim 3): a test map as in D3 whose section for the system holds only the particle-type key written with an empty value. Observable: as D3. Expected: the usual particles, as on the control map, because such a section does not exist in that layer and claim 5's reset does not apply to it. Falsified by: no visible particles.
- D4 setup (optional, needs a loose test file): a scratch copy of the installation, as in scenario H, with a loose copy of the game-mode file of one mode. In that copy, give the visible particle system of D3 a section with one unrelated key only, so that the game-mode layer creates the empty-named particle type. Both maps also give the particle system its usual behaviour explicitly, while omitting the emitted-type key, so it retains the empty-named reference through the reset but can draw particles. Test map 1 has a section with the empty name giving the full visible particle definition copied from the system's usual particle, including image and behaviour. Test map 2 has no empty-named section. Play both in that mode.
- D4 observable: whether the system draws particles, from a recording.
- D4 expected: particles with that image on map 1; none on map 2, as in D3.
- D4 falsified by: no particles on map 1 while map 2 behaves as D3 expects.
- D5 setup: a test map that gives the particle type emitted by the visible particle system of D3 a section with one unrelated key only. Check the base rules first and choose a system whose particle type gives a behaviour. Control: the same event on a stock map.
- D5 observable: how the system's particles look and move, from a recording, and whether the game keeps running.
- D5 expected: the particles differ from the control, because their type has no behaviour.
- D5 falsified by: particles identical to the control. If the game stops instead, record that: the reset stands, but its visible consequence must be revised.
- D6 setup: crates turned on in the game options. A test map whose power-up section gives one crate effect only, one whose result is easy to see (for example the money effect), with the same weight as in the base rules, and no other key. Control: a stock map. On each map, a unit picks up 20 crates, one at a time.
- D6 observable: the effect of each crate (credits gained, a unit added, a speed change and so on), from a recording. This tests weights, not an omitted effect's animation or retained parameters.
- D6 expected: on the test map every crate gives the listed effect; on the control map other effects appear as well.
- D6 falsified by: any other effect on the test map. If the given effect never appears either, record that: the reset stands, but the crate spec must say what happens when other conditions rule an effect out.
- D7 setup: check the base rules and choose a voxel animation that gives both angular velocities with values other than −1 and 0 and is easy to cause (for example a piece of debris thrown by a destroyed vehicle). Test map 1 gives its section one unrelated key only. Test map 2 gives both angular velocities as 0. Test map 3 gives both as −1 explicitly. Control: a stock map. Use the same fixed seed and destruction event for each run, matching the debris piece.
- D7 observable: how fast the piece spins while it flies, read with a recorder that reads the running game's state, or from a recording by counting frames per turn.
- D7 expected: map 2 equals the control (0 is "not given"); map 1 matches map 3's spin rate (explicit −1), unlike the control.
- D7 falsified by: map 1 differing from map 3, map 1 equal to the control when map 3 differs (no reset), or map 2 differing from the control (0 applied). If −1 and the control cannot be distinguished, the reset test is inconclusive.
- D8 setup: check the base rules and choose a voxel animation whose section gives one of the three sharing keys as yes. A test map gives that section one unrelated key only. Control: a stock map.
- D8 observable: how the animation is drawn, from a recording, and whether the game keeps running.
- D8 expected: on the test map the animation no longer uses the shared model; it shows its own model, or nothing if it has none. On the control map it looks as usual.
- D8 falsified by: the same picture on both maps. If the game stops instead, record that: the reset stands, but its visible consequence must be revised.
- D10 (the rows in the base rules layer, claim 5; optional, needs a loose test file). Each part uses a loose copy of the base rules from which one key has been removed, with the control on the stock rules, and a stock map in both runs.
  - D10a: remove the particle-type key from the section of the visible particle system of D3. Observable and expected as D3: no visible particles, because the base rules' own layer creates and uses the empty-named particle type. Falsified by: the usual particles.
  - D10b: remove both angular velocity keys from the section of the voxel animation of D7. Observable: the spin, as in D7. Expected: the spin rate of −1 in the written unit, unlike the control. Falsified by: the same spin as the control.
  - D10c (the difficulty row, with the carry-over of claim 10): remove the firepower multiplier from the section of the normal difficulty level. Procedure: start the program fresh; game 1 on a test map whose normal-level section gives the firepower multiplier 0.5 alone; return to the menu; game 2 on a stock map. Observable: health lost per shot as in D1, for a human-controlled house with the country's factor 1.0 (as D1 gives). Expected: game 2 shows the damage of a multiplier of 1.0 (the base rules' layer resets the missing key), not that of 0.5. Falsified by: game 2 showing the damage of 0.5 (the held value was kept).

**E. No earlier definition and an unread reference (claim 7).**
- Setup: a named loose base-rules copy gives a new vehicle name a full usable section with an explicitly chosen test health value, but leaves it unlisted and unreferenced. The test map lists that name and gives its full usable section with health omitted. A control map gives a different, explicitly chosen test health value. No mode layer names the vehicle.
- Procedure and observable: build the vehicle on each map; record maximum health with the recorder. If the omission run cannot build or place it, record that outcome rather than inventing a health value.
- Expected: the omission run uses the kind's engine default, not 700; the control has 300 health points. Falsified by: 700 in the omission run or a control other than 300. If default health prevents placement, the health test is inconclusive.
- Unread-reference setup: a visible particle system of D3 references a new, unlisted particle in the test map. That map gives the new particle a full visible definition copied from the usual particle. Control: also list the new particle. Cause the event creating the system and record its particles.
- Expected: no usual particles in the reference-only run; usual particles in the listed control. Falsified by: usual particles in the reference-only run. Failure of the listed control makes the test inconclusive.

**F. Animations ignore rules layers (claim 8).**
- Setup: in the test map, give an existing animation's name a section that changes its playback rate. Control: the same animation on a stock map.
- Observable: the animation's duration in drawn frames and in ticks, from a recording or a recorder that reads the running game's state.
- Expected: equal durations on both maps.
- Falsified by: a different duration on the test map.

**G. Carry-over between games (claim 10).**
- Setup: test map 1 overrides a tiberium key that the base rules do not give. Check the base file first and choose such a key, for example one of the spread or growth keys if absent. Map 1 must also have a tiberium list whose entry is keyed with that tiberium's existing index (S-0004, claim 10); without it the override is ignored. Map 2 is a stock map.
- Procedure: start the program fresh, play map 1, return to the menu without quitting, then play map 2. Repeat map 2 after a fresh program start.
- Observable: ore growth or spread over a fixed number of ticks, by cell counts.
- Expected: map 2 shows map 1's value in the first session and the stock value after a restart.
- Falsified by: map 2 showing the stock value in the first session, or map 1's value after the restart.

**H. Mode file fixed at program start (claim 11).**
- Setup: a scratch copy of the installation with a loose copy of one mode's game-mode file. This needs a writable copy of the game folder, and the measurement is optional.
- Procedure: start the program. Before playing any game, edit the loose copy so that it changes one visible value. Play a skirmish in that mode. Edit again and play a second one. Then restart the program and play a third.
- Observable: the chosen value in each game (for example a price in credits or a unit's health).
- Expected: the first two games show the value the file had at program start; the third shows the latest edit.
- Falsified by: the first or second game showing an edited value, or the third not showing the latest edit.

**I. Language rules digest (claim 12).** Optional: stock data has no such file. Every part uses a scratch copy of the installation. I1 and I2 use a loose language rules file that has no digest section and changes one visible value.
- I1 (program start). Setup: the file is present when the program starts. Observable: whether the main menu is reached. Expected: it is not; start-up fails. Falsified by: the main menu appearing.
- I2 (game start). Setup: start the program without the file; at the main menu, add the file. Start a skirmish in the mode chosen for scenario L, on a test map that gives no colour-adjustment section. Observables: the value the language file changes; the key that the mode's game-mode file gives (scenario L); whether a building under the invulnerability effect shows its usual tint.
- I2 expected: the language file's value is not applied; the mode key has its base value; the building shows no tint.
- I2 falsified by: the mode key showing the game-mode file's value, the language value being applied, or the usual tint appearing.
- I3 (file with no section). Setup: a loose language rules file holding only comment lines, present when the program starts. Start a skirmish in the mode chosen for scenario L on a stock map. Observables: whether the main menu is reached; the key that the mode's game-mode file gives. Expected: the menu is reached and the mode key shows the game-mode file's value. Falsified by: start-up failing, or the mode key showing its base value.
- I4 (optional, the type read lost with the game-mode layer). Setup: local T's animation test data, and the loose language rules file of I2, added at the main menu as in I2. Start a skirmish on T's test map. Observable: the animation's picture. Expected: invisible, because the game has two type reads instead of three. Falsified by: the usual picture. Run once with the file and once without it; the control shows the usual picture as T expects.

**J. "Not given" recharge time (claim 3).**
- Setup: test map 1 gives a superweapon's section a recharge time of 0. Test map 2 gives it a recharge time clearly different from the base value. Control: a stock map.
- Observable: ticks from the completion of the superweapon's building until the weapon is ready.
- Expected: map 1 equals the control; map 2 differs from it in line with the new value.
- Falsified by: map 1 differing from the control (an explicit 0 applied), or map 2 equal to it.

**K. Derived values follow the final values (claim 9).**
- Setup: test map gives a weapon's projectile speed field a value clearly different from its base value. Control: a stock map.
- Observable: the projectile's flight time, in ticks, over a fixed distance to a stationary target.
- Expected: the test map's flight time matches the overridden speed; it differs from the control.
- Falsified by: equal flight times on both maps.

**L. Base, game-mode file and map order (claim 1).**
- Setup: check the stock game-mode files and choose a mode whose file gives a key *k* with a visible effect (for example a price in credits or a health value) and a value different from the base rules'. Choose a second mode whose file does not give *k*, or cannot be resolved.
- Procedure: run 1, a stock map in the second mode; run 2, the same map in the chosen mode; run 3, a test map that gives *k* a third value, in the chosen mode.
- Observable: the effect of *k* in each run.
- Expected: run 1 shows the base value, run 2 the mode file's value, run 3 the map's value.
- Falsified by: run 2 showing the base value, or run 3 showing the mode file's value.

**M. Colour-adjustment table by position (claim 2).**
- Setup: a test map whose colour-adjustment section has 16 entries, every one with the same vivid colour (for example pure blue) and with key names that do not occur in the base rules' section. Control: a stock map.
- Observable: the tint of a building under the invulnerability effect.
- Expected: blue on the test map; the usual tint on the control map.
- Falsified by: the usual tint on the test map (entries matched by name instead of position).

**N. A tiberium section needs a list entry (claim 2).**
- Setup: test map 1 gives an existing tiberium's section a changed growth or spread key, with no tiberium list. Test map 2 is the same plus a tiberium list whose entry is keyed with that tiberium's existing index (S-0004, claim 10). Control: a stock map. Each run starts a fresh program.
- Observable: ore growth or spread over a fixed number of ticks, by cell counts.
- Expected: map 1 equals the control; map 2 shows the changed value.
- Falsified by: map 1 showing the changed value, or map 2 equal to the control.

**O. Game-wide values carry over between games (claims 3, 10 and 2).**
- Setup: check the base rules and choose a game-wide key *k* that they do not give and whose effect can be measured, for example the harvester's load rate in the general settings, if the base rules do not give it. Test map 1 gives *k* a value clearly different from its absent-key default. Map 1 also gives a colour-adjustment section with 16 entries of one vivid colour (as in scenario M) and a difficulty section holding only the firepower multiplier, set to a value clearly different from the base value for that level (as in scenario D1). Map 2 is a stock map that gives none of these sections. Use one game mode throughout.
- Procedure: start the program fresh. Game 1: map 2. Return to the menu without quitting. Game 2: map 1. Return to the menu. Game 3: map 2. Quit, start the program fresh, game 4: map 2. Optionally, game 5 in the same session as game 4: map 1, then game 6: map 2 in another game mode.
- Observable: the effect of *k* in each game (for the load rate, the ticks a harvester takes to fill from empty on a fixed ore field); the tint of a building under the invulnerability effect; health lost per shot from the weapon of a house that takes that level's row, as in D1.
- Expected: *k* shows its absent-key default in games 1 and 4, map 1's value in game 2, and map 1's value again in game 3 (and in game 6). The tint is the vivid colour in game 2 and the usual tint in games 1, 3 and 4. The damage reflects map 1's firepower multiplier in game 2 and the base multiplier in games 1, 3 and 4.
- Falsified by: game 3 showing *k*'s absent-key default (game-wide values are reset), game 4 showing map 1's value (the carry-over survives a restart), game 6 showing the default (the carry-over is limited to one mode), or game 3 showing the vivid tint or map 1's firepower (the restored exceptions are not restored).

**P. A negative building light value re-read (claim 5).**
- Setup: a loose copy of the base rules (a named test file of the measurement setup, removed after the run) in which one light-emitting building type, whose section no stock game-mode file gives, has its light intensity set to −3.0, with its other keys unchanged. Test map: a section for that building type holding one key that has no effect on lighting. Control map: the same map without that section. On each map, one such building stands on flat ground, and the same cell is also observed in a run without the building.
- Observable: how much the building lowers the light level of the cell at its centre (the level with the building minus the level without it), read with a recorder that reads the running game's state; and a screenshot of the darkened area.
- Expected: −2999 thousandths on the control map and −1999 on the test map; the area is visibly less dark on the test map.
- Falsified by: −2000 on the test map (the re-read gains no thousandth), or equal values on both maps (no truncation on re-read).

**Q. The mode list (claim 13).** A scratch copy of the installation with a loose copy of the stock mode list and two loose test game-mode files, all named test files of the measurement setup: T1 raises one unit's cost by 1 credit and T2 by 2 credits (the pattern of S-0005, scenario M2 (b)). The copy of the mode list keeps every stock entry and adds the entries below. *N* is the highest stock mode number. Items 1, 2 and 4 of each added entry are copied from one stock entry of the standard battle kind, so the added entries show that stock entry's name: each is told apart by its place in the list and by the price when it is played. Item 3 is given here.
- Q1: in the standard battle section, number *N* + 1, naming T1, with spaces around every item and an extra comma before item 3, so that one empty item appears.
- Q2: in the standard battle section, number *N* + 2, naming T2, with item 4 left out.
- Q3: in the standard battle section, number *N* + 3, whose item 3 is only spaces, followed by item 4.
- Q4: in the free-for-all section, number 0, naming T2.
- Q5: number *N* + 4 twice, first in the standard battle section naming T1, then in the free-for-all section naming T2.
- Q6 (optional): two further entries in the standard battle section, numbers *N* + 5 and *N* + 6, naming T1, with fifth item `false` and `no`.
- Q7 (optional, an empty game-mode layer still gives a type read, claim 1): number *N* + 7 in the standard battle section, naming a file that exists nowhere. With local T's animation test data, play its test map in the Q7 mode.
- Q8 (the remembered mode, claim 13): no new entry. It uses the other entries of Q and *M* for the number of listed modes. *M* is *N* + 10 if the stock numbering is 1 to *N*, because the Q1, Q4, Q5 (twice), Q6 (twice), Q7, Q9, Q10 and Q11 entries are listed and Q2 and Q3 are not; count the list on screen instead of assuming it. Q11's number, *N* + 20, is above *M* although a listed mode has it, so Q8c can tell the count of listed modes from the highest mode number.
- Q9 (a tab at the edge of an item, claim 13): in the standard battle section, number *N* + 8, whose item 3 is a single tab, followed by item 4.
- Q10 (a missing label, claim 13): in the standard battle section, number *N* + 9, naming T1, whose item 1 is a label that the string table lacks.
- Q11 (a listed number above the count, for Q8c): in the standard battle section, number *N* + 20, naming T1. It only needs to be listed; it is not played.
- Procedure: start the program with a settings file that remembers no mode. Open the skirmish screen and note the preselected mode and the order of the mode list. Then play a short skirmish in each listed test mode except Q11, and in the second of the two Q5 modes as listed. For Q8, close the program and set the remembered skirmish mode number in the settings file, one value per fresh program start, and note the preselected mode each time: (a) 0 (Q8a); (b) −5 (Q8b); (c) *N* + 20, the number of the Q11 mode, which is above *M* (Q8c); (d) *N* + 2, a number no listed mode has although it is not above *M* (Q8d); (e) *N* + 4, the number of the two Q5 modes (Q8e); (f) 1, which the stock list has (Q8f, the control). For Q8e, also play a short skirmish in the preselected mode.
- Observables: which test modes are listed, and in which order (screenshot); the preselected mode; the price of the chosen unit, in credits, in each mode; for Q7, the test animation's picture; for Q8, the preselected mode in each part; for Q9 and Q10, whether the mode is listed, the mode's name text, and the price in a skirmish; for Q6, whether computer players can be added (the use that the mode-list row of S-0005's Import mapping names for the flag, whether AI players are allowed; no claim of S-0005 or of this spec states that use, open question 9).
- Expected: Q1 is listed and its price is 1 credit higher; Q2 and Q3 are not listed; the Q4 mode is listed first and is preselected; both Q5 modes are listed, and playing the second one gives the price 1 credit higher (T1, the first one's file); for Q6, computer players can be added with `no` but not with `false`; Q7 is listed, and its skirmish shows the test animation's usual picture (local T), as with a resolved file, because the empty layer still gives the third type read; Q8a to Q8d preselect the Q4 mode (the first listed; Q8a cannot tell the rule "a number below 1 gives the first listed mode" from "the number selects the mode numbered *r*", because the Q4 mode has number 0 and is first: only Q8b tells them apart), Q8e preselects the first of the two Q5 modes (the standard battle one, with the 1-credit price when played), and Q8f preselects the stock mode numbered 1; Q9 is listed, with the base price (its file name is a tab, which no file has, so its layer is empty); Q10 is listed, its name shows the placeholder text for a missing label, and its price is 1 credit higher; Q11 is listed, last.
- Falsified by: Q1 missing or at the base price (empty item or spaces not handled as stated); Q2 or Q3 listed; the Q4 mode not first, or not preselected; the second Q5 mode giving the 2-credit price; for Q6, `no` behaving like `false`; Q7 missing, or its animation invisible (the empty layer gives no type read). For Q8, any of a to d preselecting another mode (for c, the Q11 mode would mean that the bound is the highest mode number, not the count of listed modes), or e preselecting the free-for-all one (the later of the two Q5 modes), or f preselecting the Q4 mode; for Q9, the mode missing, or listed with the 1-credit or 2-credit price (the tab removed); for Q10, the mode missing or its name text empty. If neither Q6 entry limits computer players, the flag has another use and Q6 cannot test it. If Q11 is not listed, Q8c cannot test the bound.

**Q12. Mode number spellings, overflow, more than five items, section case and a repeated key (claim 13).** A separate run with its own scratch copy of the installation and its own loose copy of the stock mode list, which keeps every stock entry and adds the entries below; the loose files T1 and T2 are those of scenario Q. *N* is the highest stock mode number (check that the stock numbering is 1 to *N*) and *B* = *N* + 10, so that *B* has at least two digits. Items 1, 2 and 4 of each added entry are copied from one stock entry of the standard battle kind, so each added entry is told apart by its place in the list and by the price when it is played.
- Entries, all in the standard battle section unless stated: (a) key `$2`, naming T1; (b) key *B* followed by `h`, naming T2; (c) key *B* + 1, naming T1; (d) the decimal spelling of 4294967296 + *B* + 2, naming T2; (e) key *B* + 3, naming T1; (f) key *B* + 4, naming T2, with a sixth item added after the fifth; (g) key *B* + 5, naming T2, in a section whose name is the standard battle section's name in a different letter case; (h) key *B* + 6 written twice in one section, the first line naming T1 and the second T2.
- Procedure: start the program with a settings file that remembers no mode. Note the order of the mode list on the skirmish screen, then play a short skirmish in each added mode that is listed.
- Observables: the order of the list and which added modes are listed; the price of the chosen unit, in credits, in each added mode.
- Expected: (a) is listed first, before the stock mode numbered 1, and its price is 1 credit higher (its number is 0); the stock modes follow; then (b), (c), (d), (e) and (f) in that order, with (b), (d) and (f) at 2 credits higher and (c) and (e) at 1 credit higher; (g) is not listed; (h) is listed twice, next to each other after (f), and both play at the same price, 1 or 2 credits higher, whichever line the lookup returns (S-0002, claim 1).
- Falsified by: (a) listed anywhere other than first (the `$` form recognised, which would put it at the place of 2); (b) listed after (c) (the digits read as hexadecimal, which would put it after every other added mode, or the `h` read as a hexadecimal marker); (d) listed anywhere other than between (c) and (e) (an overflow that clamps instead of wrapping would put it after (e)); (f) not listed, or at the base price (the extra item makes the entry invalid, or is read); (g) listed; (h) listed once, or the two entries at different prices.

**R. "Not given" for an integer key (claim 3).** A scratch copy of the installation with a loose copy of the art file (a named test file of the measurement setup). Choose an animation that is easy to cause and whose art section does not give the rate key.
- Runs: R0, the loose copy unchanged; R1, the rate key added as `$FFFFFFFF`; R2, the rate key added as `-1`; R3, the rate key added with a value that clearly changes the speed of the animation. Each run is a fresh program start on the same stock map.
- Observable: the animation's duration, in ticks, from a recorder that reads the running game's state or from a recording.
- Expected: R1 and R2 equal R0; R3 differs.
- Falsified by: R1 or R2 differing from R0 (the value was applied), or R3 equal to R0 (the key is not read at all).

**S. The language rules file's place (claims 1 and 2).** Optional: stock data has no such file. A scratch copy of the installation with a loose language rules file, a named test file of the measurement setup, that carries a valid integrity digest section formed as S-0005, claim 21b, and its Formulas give. Besides the digest, the file gives the key *k* of scenario L a fourth value, different from the base value, the game-mode file's and the test map's, and a colour-adjustment section of 16 entries, every one the same vivid colour (for example pure green), with key names that do not occur in the base rules' section. Use the two modes and the test map of scenario L.
- Runs: S1, a stock map in the second mode of scenario L; S2, the same map in the chosen mode; S3, scenario L's test map in the chosen mode. Each run is a fresh program start.
- Observable: the effect of *k* in each run; the tint of a building under the invulnerability effect.
- Expected: S1 shows the language file's value of *k*, S2 the game-mode file's, S3 the map's. In all three the tint is the usual one: that tint shows in stock play, so the base rules' section gives its position, and the language rules file cannot change a position that the base rules give.
- Falsified by: S1 showing the base value (the language file is not applied, or ranks below the base rules), S2 showing the language file's value (it ranks above the game-mode layer), S3 not showing the map's value, or the vivid tint in any run.
- S4 (optional, a position the base rules lack): add a loose copy of the base rules (a named test file) whose colour-adjustment section keeps only its first *m* entries, with *m* the largest number for which a control run without the language rules file shows the building under the invulnerability effect with no tint (its position then holds zero). Then repeat S1 with the language rules file. Expected: the vivid tint. Falsified by: no tint (the language file does not fill a position the base rules lack) or the usual tint.

**T. Skirmish animation read count (claims 1 and 12; I4 and Q7).**
- Setup: named loose art copy and test expansion archive, temperate map. Choose a stock theater-flag animation X named by the base animation list, with its stock temperate sprite available; place or trigger X in view. In the art copy X keeps its theater flag and names invented image `ZZQ`; the new `ZZQ` section clears the theater flag. Neither section sets the new-theater flag. The archive contains a distinguishable `ZZQ.tem`, but no `ZZQ.shp` or theater-letter variant. No language file is present in the control.
- Procedure: from fresh starts cause X in an ordinary mode, in Q7's unresolved-file mode, and with I4's invalid language file added after startup; record its picture at the same age.
- Expected: ordinary mode and Q7 show X's usual stock picture (three reads); I4 is invisible (two reads). None shows the invented picture. Falsified by: either three-read run invisible or showing the invented sprite, or I4 visible. Repeat with S's valid language file, then with a comment-only language file containing no sections: expect invisibility in both (one added read), falsified by any visible sprite.
- This discriminator follows S-0007 claim 2. It tests the stated counts through their odd/even consequence, not arbitrary extra pairs of reads.

**V. Terrain-strength fill (claim 5).**
- Setup: named test map, flat ground, one stock tree whose base section omits strength. Base game-wide tree strength must be at least 2 health points; otherwise use a named loose base copy setting it to 2 in every run. No mode changes the tree.
- Procedure: capture its starting image frame on four variants: (a) tree strength −1 and game-wide tree strength 1; (b) tree strength −1 and game-wide tree strength 2; (c) game-wide tree strength 1, no tree section; (d) game-wide tree strength 1 and a tree section with an unrelated key but strength omitted.
- Expected: frame 1 in (a), frame 0 in (b), (c), (d). Falsified by: any different frame, particularly frame 1 in (d), indicating omission refills an already set strength.

**W. Barrel reset to turret values (claim 5).**
- Setup: named loose base copy; stock building with visible voxel turret and barrel, recoil enabled, turret travel 0 model units, barrel travel 3 model units, barrel compress 2 ticks, hold 3 ticks, recover 4 ticks. Remove other turret recoil keys. Mode has no building section. A stationary enemy target stands in range; stop repeat firing after one shot with scripted input.
- Procedure: record turret and barrel displacement per tick on three maps. Control has no building section; turret-only map sets turret travel 6 model units and compress, hold, recover to 2, 3, 4 ticks, omitting barrel values; section-only map restates recoil enabled but omits all distances and durations.
- Expected: control turret stays still, barrel slides 1.5, 3, 3, 3, 3, 2.25, 1.5, 0.75, 0 model units. Turret-only gives both parts 3, 6, 6, 6, 6, 4.5, 3, 1.5, 0. Section-only gives neither part any slide. Allow the sequence to begin on the shot tick or the next tick, consistently with S-0105.
- Falsified by: any different sequence after alignment, notably the barrel retaining its control sequence on turret-only or moving on section-only. These sequences test all four copied values.

**X. Light-value truncation (claim 5).**
- Setup: named loose base copy gives one light-emitting building intensity and each tint 1.5, and each game-wide extra-light value 1.5. No mode changes the sections. Flat test maps contain the building and a vehicle, infantry and aircraft. For each field compare a map with its section containing one unrelated key only against a map without that section; keep other fields identical.
- Procedure: record the affected value in thousandths with the read-only recorder, removing unrelated lighting contributions; also capture matched screenshots. For extra-light use the matching object's lighting contribution.
- Expected: 1500 thousandths in controls, 1000 after section-present omission. Falsified by: retention of 1500 or any other value after unrelated contributions are removed. Repeat P for each negative building tint, recording tint instead of intensity: −2999 becomes −1999, falsified by equality or −2000. For negative extra-light use −3.0: −3000 stays −3000, falsified by a change.

**Y. Type append, retention and re-listing (claim 6).**
- Setup: named loose mode copy adds fully defined new overlays A and B in that order, using distinguishable art in a named art copy and test expansion archive. Let N be the number of base-rule overlays; account for all base references, not just list entries. Names are collision-free and at most 24 bytes. Map overlay list re-lists B then A, omits base types and adds fully defined C last with a third image.
- Procedure: place overlays by index at N, N + 1, N + 2 and one known base index; record the four cells. Control map re-lists A then B and adds C.
- Expected: A, B, C at the new indices on both maps, base overlay unchanged. Falsified by: exchanged A and B, missing base overlay, C at another index, or different pictures between maps. Unexpected overlays from base references invalidate the assumed N; correct the setup before comparing.

**Z. Anti-air-rate layer reset (claim 5; S-0215 claim 21).**
- Setup: named loose base copy gives guard rate 0.07 game minutes, anti-air rate 0.2. Mode has no guard section. Human-owned armed building in guard, no target; bystander isolated from it. Map variants give guard rate 0.2 and anti-air rate (a) omitted, (b) 0.07, (c) 0. Game speed 2; no other rate modifiers. Void runs with a target or behaviour change.
- Procedure: record 20 consecutive guard-processing intervals in ticks with the read-only recorder, aligned to processing turns, not shots.
- Expected: (a) and (c) 180, 181 or 182 ticks; (b) 63, 64 or 65. Falsified by: any interval outside its range, especially 63 to 65 in (a) or (c), indicating retention rather than reset. This tests the reset, not random draw counts.

**AA. Pip-colour omission (claim 5; display-only departure).**
- Setup: measurement setup, one tiny flat skirmish map, game speed 2, director and mods off. Place an own stock infantry passenger, a compatible own transport and an own occupiable building. A named loose test copy of the base rules explicitly gives that infantry type passenger pip image frame 1 and occupant pip image frame 7; choose a mode whose file has no section for the type. Compare a control map with no infantry-type section to a test map whose section gives one unrelated valid field but omits both pip keys. Use identical unit placements and orders, and one run per map.
- Procedure: load the passenger into the transport and, separately, occupy the building with an infantry unit of the same type. Select each host and capture its pips; compare the image frames using the stock pip sprites.
- Observable: passenger and occupied-slot pip image frames, not passenger capacity or loading legality.
- Expected original result: the control shows frames 1 and 7; the test map shows frames 2 and 9. Retention of frames 1 and 7 on the test map falsifies the original omission-shift finding.
- Engine acceptance check: both maps show frames 1 and 7, regardless of the override-reset switch. Any shift on the test map fails the fixed behaviour. This is not an oracle expectation.

**U. The mission-named file (claim 14; deferred campaign proposal, not run in this milestone).** Optional: stock data has no such file. A scratch copy of the installation with a loose mission-named file for the first mission of the Allied campaign, a named test file of the measurement setup (the file of S-0002's scenario D5, row D-h, but with rules sections only).
- U1 setup: check first that the first Allied mission's map gives no section or key for any of the items below; if it does, choose another item. The file gives (a) a game-wide key *k* that the base rules do not give, chosen as in scenario O but such that its effect shows both in that mission and in a skirmish, with a value clearly different from its absent-key default; (b) a game-wide key *g* that the base rules give, with a visible effect and a clearly different value; (c) a section for a vehicle type that the player has in that mission, giving its health a clearly different value; (d) a tiberium list keyed with an existing tiberium's index (S-0004, claim 10) and that tiberium's section with a changed key that the base rules do not give, chosen as in scenario G.
- U1 procedure: start the program fresh and play a short skirmish on a stock map, so that the tiberiums exist when the file is applied. Return to the menu without quitting and start the first Allied mission. Then return to the menu again and play a skirmish on a stock map. Control: the same sequence without the file, from a fresh program start.
- U1 observables: the effects of *k* and *g* in the mission and in the second skirmish; the vehicle's health in the mission (health bar, or a recorder that reads the running game's state); ore growth or spread in the second skirmish over a fixed number of ticks, by cell counts.
- U1 expected: *k* shows the file's value in the mission and again in the second skirmish, while the control shows the absent-key default in both; *g* and the vehicle's health show the base values, as in the control; the tiberium shows the changed value in the second skirmish.
- U1 falsified by: *k* at its default in the mission or in the second skirmish (the file is not applied, or its game-wide values do not survive); *g* or the vehicle's health showing the file's value (the file acts as a layer above the base rules); the tiberium unchanged in the second skirmish.
- U2 setup (the side order): a second test file of the same name whose only section is a side list naming the Soviet side first and the Allied side second, each with the countries that the base rules' side list gives it. Procedure: start the first Allied mission from a fresh program start. Observable: the look of the player's interface, Allied or Soviet, as in S-0004, scenario E4. Expected: the Soviet look, because the Allied side then has index 1 (S-0005, claim 13a). Falsified by: the Allied look. If the mission does not reach play, the run is inconclusive, as in S-0004, scenario E4.

## Open questions

1. Campaign play is not otherwise covered by this spec: claim 14 covers only the mission-named file's effect on the rules state.
2. Stock maps (claim 10): whether any stock map gives a game-wide key that the stock base rules lack, and so causes a carry-over in stock play, has not been checked. The stock game-mode files cause none.
3. Not every text key was checked for further reset-on-override cases beyond claim 5. Not covered: art-related text keys read from rules sections, smudge, terrain, overlay and voxel-animation text keys, and the text keys of game-wide sections. Each topic spec should confirm, for its own keys, that an absent key keeps the earlier value.
4. Section and key name matching (letter case) across layers follows S-0001. A map section whose name differs only in case from the type's name is matched or not according to that spec.
5. Parent country (claim 5): S-0413 claim 4 identifies no play consumer and gives the field no effect in our engine. Its reset remains unmeasured unless a future consumer exposes it.
6. Maximum number of players (claim 2): whether the value has any effect after a game has started is not established.
7. Unsampled resets (claim 5): ground-speed, air-speed, build slowdown, destroys walls, content scan and land-type buildable need identified consumers and discriminating scenarios. This spec establishes how the difficulty speed fields are layered and reset, not whether they affect movement. S-0207's open question about other country or difficulty speed fields remains with the movement spec; the reset rules here do not settle it. Cost and build-time factors have no price or production-time effect (S-0404 claim 2 and S-0403 claim 9). Repair delay is in game minutes but controls original autonomous computer repair scheduling, not our player rules (S-0412 claim 5); its reset remains unmeasured here. Build delay is preserved as a dimensionless imported scalar only: no gameplay duration or conversion is claimed until a consuming use is established. Firepower and rate-of-fire resets have local D1 and D9; armour has no damage use (S-0302 claim 3).
8. Pitch and roll angles (claim 3): no spec yet says what these values do, so their "not given" value remains unmeasured until the spec that uses them supplies a scenario.
9. Mode kinds and the mode list (claim 13): what each of the six mode kinds changes in play, what the map filter and the fifth item's flag do, and what the lobby offers when no mode is listed at all (S-0005, Open questions) are outside this spec.
10. The voxel-animation angular velocities (claims 3 and 5): the number domain of the stored value and the rounding of the conversion product are not established, and no approved spec owns the stored unit or the use of the value. The voxel-animation spec that comes to use them must state both, with floating point in the original called out and the round-toward-zero mode of S-0205 considered. Until then the reset is `unmeasured` as to the stored product and `measurable` only as to the observable spin rate relative to the control (scenario D7), which does not need the stored value. Confidence in the stored product stays low.
11. Reset keys with the switch off (claim 5): original program-start values of difficulty, land-type and power-up keys, and the parent of a new country, were not established here. Our switch-off defaults are design choices. A future original-game test must distinguish constructor defaults without invoking an original reset; omitting a key in a present map section cannot do so.
