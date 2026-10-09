---
id: S-0006
title: Rules layering
topic: rules-import
revision: 4
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-09
measurements: []
requests: []
supersedes: []
---

# S-0006: Rules layering

## Summary

At each game start the game applies several rules files (rules layers) in a fixed order on top of the rules state it already holds: the base rules, then the language rules file if a valid one exists, then the game-mode layer, then the map's own rule sections. This spec covers that order, what a later layer can change, the per-key rule "an absent key keeps the held value" and its exceptions, how later layers interact with the type lists, and the rule data that survives from one game to the next (game-wide values and tiberiums); type registration within a layer is in S-0004, and which copy of a file wins (loose file or archive) is in S-0005.

## Terms

Shared terms used here are defined in the glossary: *base rules*, *colour-adjustment table*, *departure*, *game*, *game minute*, *game-mode file*, *game start*, *game-wide section*, *importer warning*, *language rules file*, *map layer*, *measurement setup*, *override section*, *recorder*, *reset on override*, *rules layer*, *session*, *type list*, *type section*, *unspecified*.

- **Layer.** In this spec, short for the glossary's *rules layer*; it never means a render layer. Every layer is treated alike: its type lists add types (S-0004, claims 2 to 4), its game-wide sections are read, and each existing type reads its type section from it, subject to the rule of S-0004, claim 6, on which types are read in the layer that creates them.
- **Not given.** For a few numeric keys, one special value that the original treats as if the key were absent (claim 3).
- **Held value.** The value a key has in the rules state at the moment a layer is read. It is what earlier layers of this game set. If no layer of this game has set the key yet, it is what the previous game of the session left (claim 10) for a game-wide key or a tiberium's key, and the engine default for any other key. In the session's first game, and for a tiberium first created in this game, nothing is left over, so it is the engine default.
- **Engine default.** A value fixed by the original's program rather than read from data. For a key that no data gives, it is the glossary's *absent-key default*. In claim 5 it also names the reset value that the table lists for a key; some of those reset values (the country itself, the empty-named particle type) are not fixed numbers.
- **Early game-wide section.** One of these game-wide sections: the general settings, AI settings, AI intelligence levels, power-ups, land types, jump-jet controls and multiplayer dialog settings (S-0004, Terms and claim 4, row 2). A type that one of them names first reads its own section in the layer that names it. The other game-wide sections (difficulty levels, crate rules, combat damage, radiation, elevation and wall models, audio-visual settings and special weapons) are the *late* ones (S-0004, claim 4, row 4). A type that a late section names first is a reference that S-0004, claim 6, leaves unread in that layer, except a warhead in a layer that contains the special-weapon settings section; claim 7 applies to it accordingly.
- **Tiberiums.** Resource types (one of the glossary's *type kinds*), such as ore or gems. The *tiberium list* is the type list of that kind.
- **Locomotion class.** One of the categories of mobile types for which each land type gives its own movement multiplier (claim 5). The movement spec owns the list and how a type's class is chosen.

## Behaviour

1. **Layer order at each game start.** At every game start, these rules layers are applied on top of the rules state already held, from lowest to highest precedence:
   - the base rules,
   - the language rules file, only if one exists and passes its check (claim 12; stock data has none),
   - the game-mode layer, in every multiplayer or skirmish game (but see claim 12) and never in a campaign mission. It is the selected mode's game-mode file, and it is empty when the mode names no file or its file cannot be resolved (claim 11; S-0005, claim 19). An empty layer changes no rules value, but it is still an applied layer: every animation gets its reading from the art file in it, as in every applied layer (claim 8; S-0004, claim 6),
   - the map layer.

   Where two layers give the same key, the later one in this list wins (claims 3 to 5 give the exact rule), as if each layer had been processed completely before the next. Type data and game-wide data start differently:
   - Every type registry except the tiberiums' starts each game empty, so it holds only the types created for this game (S-0004, claims 1 and 3); tiberiums carry over (claim 10).
   - Game-wide sections are not cleared. Each game-wide key starts the game with the value the previous game of the session left (the engine default in the session's first game), and keeps it unless a layer of this game gives the key (claim 10). The exceptions restored at every game start are listed in claim 10.

   The players' houses (countries, colours, starting credits and so on) reflect the final values of all layers: overrides of country properties made by a game-mode file or a map apply to them.
   - Backing: `measurable` for base, game-mode file and map (scenarios A, B, L). `unmeasured` for the position of the language rules file: confirming it needs a language rules file with a valid digest, and the digest rule is not yet specified (S-0005, claim 21).
   - Confidence: high

2. **A layer may override any section.** A later layer can change every game-wide section and every type section, with these exceptions:
   - The maximum number of players is taken from the base rules only. A later layer cannot change it.
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
   - Backing: `measurable` for the general rule, the colour-adjustment table as changed by the game-mode and map layers, and the tiberium rule (scenarios A, M, N). `unmeasured` for the language rules file's part of the colour-adjustment rule, for the reason given in claim 1. `unmeasured` for the maximum number of players: it limits the lobby, which is set up before any later layer is read, and no effect of it after a game has started has been identified (open question 6).
   - Confidence: high

3. **An absent key keeps the held value.** Within a section that a layer contains, each key that is given replaces the held value. Each key that is absent keeps the held value (Terms):
   - in a type section, the value an earlier layer of this game gave, or the engine default of its kind if none did; for a tiberium, if none did, the value the previous game of the session left (claim 10), and the engine default only for a tiberium first created in this game,
   - in a game-wide section, the value an earlier layer of this game gave, or else the value the previous game of the session left (claim 10). It is the engine default only in the session's first game.

   Claim 5 lists the exceptions: keys that reset on override, and values that are truncated when re-read.

   Some numeric keys have a "not given" value: written explicitly, it keeps the held value, exactly as if the key were absent. That value is −1 for angle and animation-rate keys, and 0 for a superweapon's recharge time. Each topic spec states its own such values.
   - Backing: `measurable` (scenario A for keep and replace in a type section; scenario O for a game-wide key across games; scenario J for the "not given" value of the recharge time; the −1 values are measured by the specs of their topics)
   - Confidence: high that keeping the held value is the general rule for numeric and flag keys of every type kind and every game-wide section. Medium that claim 5 lists every exception: the first check of the numeric keys looked for keys reset to a fixed default, and missed two exceptions of other forms that the topic specs found later (the barrel values and the light values of claim 5). Medium for text and list keys. The type-reference and list keys of buildings, vehicles, aircraft, infantry, weapons, warheads, countries, superweapons, particles and particle systems were checked, as were the value forms shared by many keys: ranges, speeds, coordinate triples, categories, movement zones, owner lists and prerequisites. That check found the particle rows of claim 5. Other text keys were not checked one by one (open question 3).

4. **List-valued keys are replaced whole.** When a later layer gives a list-valued key, such as prerequisites, owner countries or a country's veteran-type lists, the new list replaces the old one entirely. Nothing is appended or merged. When the key is absent, the old list is kept.
   - Backing: `measurable` (scenario A)
   - Confidence: high

5. **Exceptions to keeping the held value.** There are three kinds: keys that reset to an engine default, keys that reset to another key's value, and values that are truncated when re-read.

   **Keys that reset on override.** In the following places, if a layer contains the section but omits the key, the value goes back to the engine default listed. It does not keep the held value.

   | Where | Key (our name) | Value after a reset |
   |---|---|---|
   | a country's section | parent country | the country itself |
   | a warhead's section | damage modifiers versus each of the 11 armour classes | 100 % for all 11 |
   | each difficulty section | firepower, ground speed, air speed, armour, rate-of-fire and cost multipliers; build-time multiplier | 1.0 each |
   | each difficulty section | repair delay | 0.02 game minutes |
   | each difficulty section | build delay | 0.03 game minutes |
   | each difficulty section | build slowdown | off |
   | each difficulty section | destroys walls | on |
   | each difficulty section | content scan | off |
   | each of the 12 land-type sections | movement multiplier per locomotion class (foot, track, wheel, hover, float, amphibious, float-on-beach) | 100 % |
   | each of the 12 land-type sections | buildable | no |
   | a particle system's section | the particle type it emits | the particle type with an empty name (see below) |
   | a particle system's section | behaviour | none |
   | a particle's section | behaviour | none |

   **The empty-named particle type.** A particle system's emitted particle type is a reference by name, and the reference creates a particle type of that name if none exists yet. When a layer contains a particle system's section but omits the key, the system emits the particle type whose name is empty. Only the explicit no-type value of the Import mapping avoids this (S-0004, claim 2). If no particle type with the empty name exists yet in this game, that layer creates it; its place in the particle registry follows S-0004, claim 4. Every particle system of the game that omits the key emits this same type, so it exists at most once per game. Like every type created by a reference that its creating layer leaves unread (claim 7), it reads no section in the layer that creates it. In each later layer of the same game it reads that layer's section with the empty name, if there is one (S-0001, claim 6: a header with nothing between the brackets); otherwise it keeps the engine defaults of its kind. The particle-system row applies to the base rules too: a particle system whose base-rules section names no emitted particle type creates the empty-named particle type in the base rules. Stock data names the particle type in every listed particle system, so with stock data this does not happen through the list.

   For the land-type rows, a movement multiplier that is given is capped at 100 %, and one further locomotion class not listed above is fixed at 100 % in every land type. Both belong to the movement spec.

   So a map that changes a single key of a warhead, a difficulty level or a land type silently resets the rest of that row's keys. For the warhead, that means the damage modifiers against all armour classes.

   Proposed switch: `quirks.override_resets_keys` (default **on** = reset as above). With it off, these keys keep the held value like every other key.

   **Keys that reset to another key's value.** In every layer that has a section for a type with barrel recoil values (S-0105 draws them for buildings), each of its four barrel recoil values (travel, compress, hold and recover) that the layer omits is set to the turret's corresponding value as it stands after that layer, replacing any barrel value an earlier layer gave. S-0105, claim 27, owns this rule, its consequences and its quirk switch, `quirks.barrel_recoil_layer_reset`.

   **Values truncated when re-read.** A building type's light intensity and its three light tints, and the three extra-light values of the audio-visual settings, are held in thousandths. When a layer contains the section but omits such a key, the held value is not kept: it becomes the value that a written value equal to its whole part would give (S-0104, claim 12), the whole part being the held value in whole units, truncated toward zero. For a held value of *t* thousandths, with *k* = trunc(*t* / 1000), the new value *t*′ is 1000 × *k* for the extra-light values; for the building values it is 1000 × *k* when *k* ≥ 0 and 1000 × *k* + 1 when *k* < 0 (Formulas). This is not a reset to a fixed default. A whole extra-light value, and a whole building value that is not negative, is unchanged. A negative building value loses up to a whole unit at every such layer: a written intensity of −3.0 is held as −2999 thousandths, and a later layer that restates the building without the key makes it −1999. For building types it can happen only in a layer after the one that created the type. For the audio-visual settings, which are game-wide, it happens in every layer that contains that section and omits the key, including the base rules of a later game when they omit it, since the held value then comes from the previous game (claim 10). S-0104, claims 12 and 13, owns these values and their quirk switch, `quirks.reread_truncates_light_values`.
   - Backing, row by row:
     - `measurable`: the warhead row (scenario C); the difficulty firepower multiplier (scenario D1); the land-type movement multipliers (scenario D2); the two particle-system rows (scenario D3); the particle row (scenario D5); the empty-named particle type reading a later layer's section with the empty name (scenario D4).
     - `unmeasured`: the parent-country row, because its observable effect belongs to another spec (open question 5).
     - `unmeasured`: the difficulty ground-speed, air-speed, armour, rate-of-fire, cost and build-time multipliers, the repair and build delays, build slowdown, destroys walls and content scan, and the land-type buildable row. No effect of these values in play has been identified, so no scenario can yet tell a reset value from a kept one. The specs that would use them own that question (open question 7).
     - The barrel and light parts are measured by the scenarios of S-0105 and S-0104, except the negative building light value, which scenario P measures.
   - Confidence: high that every row of the table resets as listed. Medium for the visible consequence of the parent-country reset, because it depends on where the parent country is used. Medium that the empty-named particle type reads a later layer's section with the empty name: this follows from the general rules of claim 7 and S-0001, claim 6, and was not checked for this type on its own. The barrel and light parts carry the confidence their own specs give; the +1 for a negative building light value was checked separately here, with high confidence.

6. **Later layers can add types but not remove or reorder them.** A later layer's type lists add new names at the end of each registry. Re-listing an existing name does nothing. Leaving a name out of a later layer's list does not remove the type. Order and indices follow S-0004, claims 2 to 4.
   - Backing: `measurable` (S-0004, scenario A)
   - Confidence: high

7. **A type never reads a section from a layer before the one that created it.** In each layer, a type reads its section from that layer only. Which layer it reads first depends on how it came into existence:
   - A type created by a layer's type list, by an early game-wide section (Terms; S-0004, claim 4, row 2), or by a reference that S-0004, claim 6, lets be read in the same layer, reads its section from the layer that created it and from every later layer.
   - A type created by a reference that S-0004, claim 6, leaves unread in that layer reads its section only from the later layers. If the creating layer is the map layer, it reads no section at all and keeps the engine defaults of its kind for every key.

   In no case does a type read the section of the same name in a layer that was applied before it existed. So a map that lists a new type must also give that type's full definition. Keys it omits take the engine defaults for that kind, not values from a base-rules section of the same name.
   - Backing: `measurable` (scenario E; the unread-reference case is S-0004, scenario B)
   - Confidence: high

8. **Animation properties cannot be overridden by layers.** Animations read their properties from the art file, and the art file is the same for every layer. A game-mode file or map can add an animation name to the animation list. That animation then gets its properties from the art file's section. A rules layer cannot change any animation property.
   - Backing: `measurable` (scenario F)
   - Confidence: high

9. **Derived values follow the final layered values.** Values that the engine computes from rules values, such as a weapon's effective projectile speed, depend only on the final layered values. Computing them after every layer and computing them once after the last layer give the same result, and the claim is stated so that an implementation may do either.
   - Backing: `measurable` (scenario K)
   - Confidence: medium

10. **Game-wide values and tiberiums carry over between games in one session.** The rules state is kept for the whole session. A game start rebuilds the type registries other than the tiberiums' (S-0004, claim 1), but it does not reset game-wide values or tiberiums. Each layer then changes only what it gives (claims 3 to 5).

    **Game-wide values.** This holds for every game-wide section (general settings, combat damage, AI settings, audio-visual settings, jump-jet controls, crate rules, radiation, special weapons, multiplayer dialog settings and the like), and for every kind of key in them: numbers, flags, text, lists and type references. Every game-wide key that the base rules give, or a language rules file that passes its check, is restored at every game start. A game-wide key that neither of them gives, but that an earlier game's game-mode layer or map set, keeps that value in every later game of the same session, until a layer sets it again. So the absent-key default of a game-wide key holds only in the session's first game, or until a layer first sets the key. A fresh program start clears the carry-over. The carry-over also reaches a later game in a mode whose file does not give the key: a game in one mode can change the rules of a later game in another mode. Stock game-mode files give no game-wide key that the stock base rules lack (checked by key name), so with stock files a carry-over can come only from a map (or from a campaign mission's file, see Edge cases).

    These game-wide values are restored at every game start whatever earlier games did:
    - the colour-adjustment table, which depends only on this game's layers, never on earlier games (claims 2 and 12),
    - the maximum number of players, which comes from the base rules only (claim 2),
    - the keys of the difficulty and land-type sections in claim 5, provided the base rules contain each of those sections, as the stock base rules do. The base rules then restore every key of the section, given or reset.

    **Type references in game-wide keys.** A game-wide key can name a type, or a list of types (for example an animation, a weapon or a warhead named in the general or audio-visual settings). If the reference comes only from an earlier game's layer, it names a type of that earlier game, which the rebuild has deleted. The original keeps a reference to the deleted type, so its result is *unspecified*. This is a *departure*: in our engine such a reference names a type by its name only. Once all layers of the new game are applied, the key refers to the type of that name if the new game has one; otherwise it refers to no type, and a list leaves that entry out. The reference never creates a type. The importer warns about every game-mode file or map that gives, in a game-wide section, a key naming a type or a list of types that the base rules do not give, since only such a key can leave a reference behind.

    **Tiberiums.** Tiberiums are not rebuilt at game start (S-0004, claim 10). At the next game start the base layer re-reads the tiberiums its own tiberium list names (S-0004, claim 10: an unlisted tiberium reads nothing), and every key the base rules give is restored. A key that the base rules do *not* give, but that an earlier game's map or game-mode file set, keeps that value into later games of the same session. A tiberium that only an earlier game's map or game-mode file added stays in the registry, with all its values, for the rest of the session. A fresh program start clears both. Sides do not carry over: they are rebuilt with the other registries.

    Proposed switches, both default **on** (the original's behaviour):
    - `quirks.persist_global_overrides`. With it off, every game-wide key starts each game from its absent-key default instead of the value the previous game left, so a game's rules never depend on earlier games of the session. Type references then never outlive their game.
    - `quirks.persist_tiberium_overrides`. With it off, tiberiums are rebuilt from the layers at every game start like other kinds.
    - Backing: `measurable` (scenario O for game-wide values and the restored exceptions; scenario G for tiberiums). Scenario O's difficulty check uses a key that the base rules give, so for the difficulty and land-type exception it confirms only that given keys are restored. That the base rules also reset the keys of those sections that they omit rests on claim 5's scenarios, which show the reset in a map layer, and on every layer being treated alike (Terms). The type-reference case has no scenario: the original's result is *unspecified*, and our engine's choice is a design decision.
    - Confidence: high for game-wide values and the restored exceptions; medium for tiberiums

11. **The game-mode file is fixed at program start.** For the whole session, every game in a mode uses that mode's file as it was at program start. This holds for every mode, whether or not it has been played yet. Edits to the file while the program runs have no effect until a restart. If the file named by the mode does not exist at program start, the mode layer is empty and changes no rules value for the whole session (it is still an applied layer, claim 1).
    - Backing: `measurable` (scenario H)
    - Confidence: high

12. **The language rules file needs a valid digest.** The language rules file is optional; stock data has none. A file that holds at least one section is applied only if it carries a valid integrity digest section. What makes a digest valid (its length, the checksum algorithm, and the open points about the bytes it covers and how it is written) is owned by S-0005, claim 21. A file that holds no section at all, an empty file for example, is accepted without any check and changes nothing. It never triggers either case below.

    A file that holds at least one section but fails the check has these effects:
    - At program start, the original refuses to start: start-up fails and the main menu is never reached. This is a *departure*, with no switch: our engine starts as though the file were absent at program start, and the importer warns. The file is still checked again at every game start, as below, so while it keeps failing, every game takes the game-start result.
    - At game start, the file is resolved and checked again for every game (S-0005, claim 21). So this case arises when a file that fails the check first appears after program start, and also when a file that was valid at program start has since been edited or replaced so that it fails. For that game the file is ignored, and so is the whole game-mode layer, including that layer's reading of animation art (claim 1). The colour-adjustment table is left with zero in all 16 positions, so only the entries the map layer gives (claim 2) are non-zero. Game-wide values are not affected otherwise: they keep the values held (claim 10), apart from what the base rules and the map give.

    In both cases the importer warns. Proposed switch: `quirks.bad_language_rules_skips_mode` (default **on**). It covers only the game-start case. With it off, a language rules file that fails the check at game start is ignored, the game-mode layer is still applied, and the colour-adjustment table is filled from the base rules as in claim 2. The program-start case has no switch; it is the departure above. The behaviour is irrelevant for stock data.
    - Backing: `measurable` (scenario I, which measures the original, including its start-up failure). Our engine's program-start result is a departure, a design decision.
    - Confidence: medium

## Formulas

The layered value *v* of a key *k* in a section *s*, after layers *L₁ … Lₙ* (base first, map last):

> *v* = the value of *k* in *s* in the last layer *Lᵢ* that gives *k* inside section *s*. If no layer of this game does, *v* = *v₀*, the value held at the start of the game:
> - for a type section, *v₀* is the engine default of *k* for the type's kind,
> - for a game-wide section, *v₀* is the value *k* had at the end of the previous game of the session (claim 10), and the engine default of *k* in the session's first game,
> - for a tiberium, *v₀* is likewise the value at the end of the previous game, or the engine default for a tiberium first created in this game.
>
> A key written with its "not given" value (claim 3) does not count as given.
>
> Exception (claim 5): if the key resets on override and at least one layer contains section *s*, let *Lₙ′* be the last layer that contains *s* at all. Then *v* = the value of *k* in *Lₙ′* if *Lₙ′* gives *k*, otherwise the reset value. If no layer contains *s*, *v* = *v₀*. For the barrel recoil values the reset value is the turret's value after *Lₙ′* (S-0105, claim 27). The light values truncated on re-read follow the light formula below instead.
>
> With `quirks.persist_global_overrides` off, *v₀* of a game-wide key is always its engine default.

For a type section, the layers considered are only those in which the type reads its section (claim 7). For a tiberium, only the layers whose tiberium list has an entry for it count (S-0004, claim 10). The colour-adjustment table follows claims 2 and 12 instead of this formula, and so does the maximum number of players (base rules only).

Layering selects values. Its only arithmetic is the truncation of the light values in claim 5, where *t* is the held value in thousandths, an integer, and *k* = trunc(*t* / 1000) is an integer division rounding toward zero:

> *t*′ = 1000 × *k* for the extra-light values; *t*′ = trunc(1000 × *k* + 0.1) for the building light values, which is 1000 × *k* when *k* ≥ 0 and 1000 × *k* + 1 when *k* < 0.

In the original both formulas are evaluated in double-precision floating point, and the result is truncated toward zero. 1000 × *k* is exact, and 1000 × *k* + 0.1 is never an integer, so the result is the same whichever way the addition rounds: it does not depend on the round-toward-zero mode. No other rounding applies. Each value keeps the number domain of its key, stated in the topic specs. The reset values 1.0, 0.02 and 0.03 of the difficulty sections, 100 % for warhead modifiers and 100 % for land-type multipliers are floating-point numbers in the original (100 % is stored as 1.0).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| armour classes per warhead | 11 | classes | all reset to 100 % (claim 5) |
| land-type sections | 12 | sections | claim 5 |
| difficulty levels | 3 | levels | claim 5 |
| difficulty multiplier reset | 1.0 | ratio (factor) | floating point in the original |
| difficulty repair delay reset | 0.02 | game minutes (unit unconfirmed) | floating point; how the value is used belongs to the repair and AI specs (open question 7) |
| difficulty build delay reset | 0.03 | game minutes (unit unconfirmed) | floating point; how the value is used belongs to the production and AI specs (open question 7) |
| land movement multiplier reset | 1.0 (100 %) | ratio (factor) | floating point |
| warhead modifier reset | 1.0 (100 %) | ratio (factor) | floating point |
| colour-adjustment table size | 16 | entries | claim 2 |
| colour-adjustment empty entry | 0, 0, 0 | red, green, blue components | adds no tint (claims 2 and 12) |
| "not given" value of angle and animation-rate keys | −1 | the key's own unit | claim 3 |
| "not given" value of a superweapon's recharge time | 0 | game minutes | claim 3 |

## State and ordering

| State | Event | Result |
|---|---|---|
| any | game starts | every type registry except the tiberiums' holds only the types created for this game; game-wide values and tiberiums are those the previous game left (claim 10), as changed by this game's layers; the colour-adjustment table holds only what claim 2 gives. The layers, from lowest to highest precedence: base, language (if present and it passes its check, or holds no section), game-mode layer (every multiplayer and skirmish game, empty when the mode has no file; skipped if a language rules file with sections fails its check, claim 12), map |
| key given in a later layer | layer applied | value replaced |
| key absent or "not given", section present, ordinary key | layer applied | held value kept |
| key absent, section present, reset-on-override key | layer applied | value set to its reset value (an engine default, or the turret's value for a barrel value) |
| key absent, section present, light value of claim 5 | layer applied | held value cut to a whole unit as claim 5 gives |
| section absent | layer applied | everything in it kept |
| any | program starts | every mode's file contents are fixed for the session as they are then |
| language rules file with sections fails its check | program starts | original: start-up fails; our engine: starts as though the file were absent, importer warns (claim 12) |

Observable ordering:
- A map's values always win over the game-mode file's, and the mode file's over the base rules'.
- A valid language rules file's values beat the base rules' (except at the colour-adjustment table positions that the base rules give, claim 2) and lose to the game-mode file's and the map's.
- Player houses see the final country values of all layers.

## Edge cases and quirks

- `quirks.override_resets_keys`: claim 5.
- `quirks.persist_global_overrides`: claim 10.
- `quirks.persist_tiberium_overrides`: claim 10.
- `quirks.bad_language_rules_skips_mode`: claim 12.
- A language rules file that fails its check at program start: the original refuses to start; our engine starts without it and warns, a *departure* without a switch; see claim 12.
- `quirks.barrel_recoil_layer_reset` and `quirks.reread_truncates_light_values`: owned by S-0105 and S-0104 (claim 5).
- A colour-adjustment section with more than 16 entries: the original overwrites other rules values with the extra entries, with an *unspecified* result; our engine ignores the extra entries and the importer warns, a *departure* (claim 2).
- A game-wide type reference left over from an earlier game: the original keeps a reference to a deleted type, with an *unspecified* result; our engine matches it by name against the new game's types and the importer warns about the keys that can cause it, a *departure* (claim 10).
- A negative building light value loses up to a whole unit at every later layer that restates the building without that key (claim 5); S-0104 owns the switch.
- Not a quirk to keep, but for the record: two further layers exist in the original but don't occur in normal play. First, in campaign missions a file named after the mission is also applied. None of its changes to types other than tiberiums take effect in the mission (open question 1). Its changes to game-wide values are not undone: where the base rules (and a valid language rules file) don't give the key, the mission file's value is in force in that mission and carries into later games of the session like any game-wide value (claim 10). Its changes to tiberiums behave like those of an earlier game's layer (claim 10). Second, a skirmish-only extra layer is never applied by the original as shipped. An implementation need not support either.

## Import mapping

| Layer | Source |
|---|---|
| base rules | `RULESMD.INI` (which copy wins: S-0005, including the start-up picker of S-0005 claim 20) |
| language rules | `LANGRULE.INI`: a file with at least one section only with a valid `[Digest]` section (S-0005, claim 21); a file with no section is accepted and changes nothing |
| game-mode layer | the override file named for the selected mode in `MPModesMD.INI` (third field of the mode's entry); an empty layer when the file cannot be resolved |
| map layer | the map file being played |
| animation properties | `ARTMD.INI`, the same in every layer |

Keys and sections named in this spec:

| Our name | Rules key or section |
|---|---|
| maximum number of players (base rules only) | `[Maximums]` `Players` |
| colour-adjustment table | `[ColorAdd]`: each entry's value is red, green, blue; key names are ignored, only the entry's position counts |
| parent country | `ParentCountry` in a country's section; when absent in a layer that has the section, it becomes the country's own name |
| warhead damage modifiers | `Verses` in a warhead's section; when absent in a layer that has the section, `100%` × 11 |
| difficulty sections | `[Easy]`, `[Normal]`, `[Difficult]` |
| AI intelligence levels (an early game-wide section, Terms) | `[IQ]` |
| difficulty keys and reset values | `FirePower`, `Groundspeed`, `Airspeed`, `Armor`, `ROF`, `Cost`, `BuildTime` = 1.0; `RepairDelay` = 0.02; `BuildDelay` = 0.03; `BuildSlowdown` = no; `DestroyWalls` = yes; `ContentScan` = no |
| land-type sections | the 12 land-type sections (`[Clear]`, `[Road]`, …), whose list is fixed in the engine and belongs to the movement spec |
| land-type keys and reset values | `Foot`, `Track`, `Wheel`, `Hover`, `Float`, `Amphibious`, `FloatBeach` = 100%; `Buildable` = no |
| particle-system keys that reset | `HoldsWhat` (empty name; only an explicit `<none>` avoids it, S-0004, claim 2, where `none` also names a particle type), `BehavesLike` (none) |
| particle key that resets | `BehavesLike` (none) |
| owner lists (example of a list key) | `Owner`, `RequiredHouses`, `ForbiddenHouses`, `SecretHouses` |
| prerequisites (example of a list key) | `Prerequisite` |
| superweapon recharge time ("0 = not given") | `RechargeTime` |
| barrel recoil values that reset to the turret's (S-0105, claim 27) | `BarrelTravel`, `BarrelCompressFrames`, `BarrelHoldFrames`, `BarrelRecoverFrames`, reset to `TurretTravel`, `TurretCompressFrames`, `TurretHoldFrames`, `TurretRecoverFrames` |
| building light values truncated on re-read (S-0104, claim 12) | `LightIntensity`, `LightRedTint`, `LightGreenTint`, `LightBlueTint` in a building type's section |
| extra-light values truncated on re-read (S-0104, claims 12 and 13) | `ExtraUnitLight`, `ExtraInfantryLight`, `ExtraAircraftLight` in `[AudioVisual]` |
| example game-wide key for scenario O | the harvester load-rate key of `[General]`, if the base rules do not give it |

## Measurement scenarios

Every scenario runs under the glossary's *measurement setup*, with stock rules, art and mode files. Overrides go in the rule sections of a purpose-made test map in the multiplayer map folder, never in the stock files. Scenarios D4, H, I and P need a loose test file and say so: these are named test files of the measurement setup, used only by the scenario that names them and removed after the run. They are run in a scratch copy of the installation, which is removed afterwards. Tick counts and damage may be read with a recorder that reads the running game's state, where one fits. A skirmish on a fixed map with fixed start positions is deterministic for what these scenarios record, so each test and control run is done once, then repeated once to confirm; a disagreement between the two repeats voids the run. Game-wide values carry over between games of a session (claim 10), so a run that follows a game on a map that gives a game-wide section can inherit that map's values. Every run, and every confirming repeat, therefore starts from a fresh program start, unless the scenario itself prescribes a sequence of games in one session (scenarios G, H and O).

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
- D1 setup: a test map with a difficulty section holding one key only, one that does not affect damage (for example the destroys-walls flag). Check the base rules and choose a difficulty level whose firepower multiplier is not 1.0. A computer house at that level fires a weapon at a fixed target in a controlled engagement; control: the same on a stock map.
- D1 observable: health lost per shot from the computer house's weapon.
- D1 expected: on the test map, the damage of a firepower multiplier of 1.0 (the same damage the weapon does for a house whose multiplier is 1.0); on the control map, damage scaled by the base multiplier.
- D1 falsified by: equal damage on test and control maps.
- D2 setup: check the base rules and choose a land type and a locomotion class whose base movement multiplier is below 100 %. If no such pair exists, D2 cannot discriminate and is skipped. A test map gives that land type's section only the buildable key; a control map has the same terrain without the section. A vehicle of that locomotion class crosses a straight strip of that land type, of fixed length.
- D2 observable: the crossing time, in ticks.
- D2 expected: on the test map, the time at a 100 % multiplier, shorter than on the control map.
- D2 falsified by: equal crossing times on both maps.
- D3 setup: a test map that gives a visible particle system (for example a smoke trail) a section with one unrelated key only. Control: the same event on a stock map.
- D3 observable: whether the system draws any particles, from a recording, and whether the game keeps running.
- D3 expected: on the test map the system draws no visible particles (the empty-named particle type has no image and the system has no behaviour); on the control map it draws its usual particles.
- D3 falsified by: the usual particles appearing on the test map. If the game stops instead, record that: the claim's reset stands, but its visible consequence must be revised.
- D4 setup (optional, needs a loose test file): a scratch copy of the installation, as in scenario H, with a loose copy of the game-mode file of one mode. In that copy, give the visible particle system of D3 a section with one unrelated key only, so that the game-mode layer creates the empty-named particle type. Test map 1 has a section with the empty name that gives the particle keys needed to draw: check the base rules and copy the image key of the particle type the system normally emits. Test map 2 has no such section. Play both maps in that mode.
- D4 observable: whether the system draws particles, from a recording.
- D4 expected: particles with that image on map 1; none on map 2, as in D3.
- D4 falsified by: no particles on map 1 while map 2 behaves as D3 expects.
- D5 setup: a test map that gives the particle type emitted by the visible particle system of D3 a section with one unrelated key only. Check the base rules first and choose a system whose particle type gives a behaviour. Control: the same event on a stock map.
- D5 observable: how the system's particles look and move, from a recording, and whether the game keeps running.
- D5 expected: the particles differ from the control, because their type has no behaviour.
- D5 falsified by: particles identical to the control. If the game stops instead, record that: the reset stands, but its visible consequence must be revised.

**E. New types read only the map section (claim 7).**
- Stock base rules contain no orphan type sections (sections with a type's keys that no list names), so the "never reads the base section" half can't be shown with stock files alone. It follows from S-0004, scenario B, which shows that a type reads only the layer it is read in.
- Setup for the defaults half: in the test map, list a new vehicle name and give its section only a display name and a cost.
- Observable: whether it can be built, and its health and speed once built.
- Expected: the engine defaults of its kind for every key not given. Those defaults are listed in the vehicle specs, and may make the vehicle unusable.
- Falsified by: values copied from some other type.

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
- Setup: check the stock game-mode files and choose a mode whose file gives a key *k* with a visible effect (for example a price in credits or a health value) and a value different from the base rules'. Choose a second mode whose file does not give *k*, or that has no file.
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
- Setup: check the base rules and choose a game-wide key *k* that they do not give and whose effect can be measured, for example the harvester's load rate in the general settings (Import mapping), if the base rules do not give it. Test map 1 gives *k* a value clearly different from its absent-key default. Map 1 also gives a colour-adjustment section with 16 entries of one vivid colour (as in scenario M) and a difficulty section holding only the firepower multiplier, set to a value clearly different from the base value for that level (as in scenario D1). Map 2 is a stock map that gives none of these sections. Use one game mode throughout.
- Procedure: start the program fresh. Game 1: map 2. Return to the menu without quitting. Game 2: map 1. Return to the menu. Game 3: map 2. Quit, start the program fresh, game 4: map 2. Optionally, game 5 in the same session as game 4: map 1, then game 6: map 2 in another game mode.
- Observable: the effect of *k* in each game (for the load rate, the ticks a harvester takes to fill from empty on a fixed ore field); the tint of a building under the invulnerability effect; health lost per shot from a computer house's weapon at that difficulty level, as in D1.
- Expected: *k* shows its absent-key default in games 1 and 4, map 1's value in game 2, and map 1's value again in game 3 (and in game 6). The tint is the vivid colour in game 2 and the usual tint in games 1, 3 and 4. The damage reflects map 1's firepower multiplier in game 2 and the base multiplier in games 1, 3 and 4.
- Falsified by: game 3 showing *k*'s absent-key default (game-wide values are reset), game 4 showing map 1's value (the carry-over survives a restart), game 6 showing the default (the carry-over is limited to one mode), or game 3 showing the vivid tint or map 1's firepower (the restored exceptions are not restored).

**P. A negative building light value re-read (claim 5).**
- Setup: a loose copy of the base rules (a named test file of the measurement setup, removed after the run) in which one light-emitting building type, whose section no stock game-mode file gives, has its light intensity set to −3.0, with its other keys unchanged. Test map: a section for that building type holding one key that has no effect on lighting. Control map: the same map without that section. On each map, one such building stands on flat ground, and the same cell is also observed in a run without the building.
- Observable: how much the building lowers the light level of the cell at its centre (the level with the building minus the level without it), read with a recorder that reads the running game's state; and a screenshot of the darkened area.
- Expected: −2999 thousandths on the control map and −1999 on the test map; the area is visibly less dark on the test map.
- Falsified by: −2000 on the test map (the re-read gains no thousandth), or equal values on both maps (no truncation on re-read).

## Open questions

1. Campaign: the file named after the mission is applied in campaign missions. Its exact name, and its effects beyond those stated in Edge cases (game-wide values kept where the base rules don't give the key, type changes other than tiberiums lost), are not established. Campaign play is not otherwise covered by this spec.
2. Stock maps (claim 10): whether any stock map gives a game-wide key that the stock base rules lack, and so causes a carry-over in stock play, has not been checked. The stock game-mode files cause none.
3. Not every text key was checked for further reset-on-override cases beyond claim 5. Not covered: art-related text keys read from rules sections, smudge, terrain, overlay and voxel-animation text keys, and the text keys of game-wide sections. Each topic spec should confirm, for its own keys, that an absent key keeps the earlier value.
4. Section and key name matching (letter case) across layers follows S-0001. A map section whose name differs only in case from the type's name is matched or not according to that spec.
5. Parent country (claim 5): where the parent country is used, and so how its reset shows in play, is not settled. A scenario will be added once the spec that uses it exists. Confidence stays medium.
6. Maximum number of players (claim 2): whether the value has any effect after a game has started is not established.
7. Difficulty and land-type values without an identified effect (claim 5): no effect in play has been identified for the repair delay, the build delay, the ground-speed, air-speed, armour, rate-of-fire, cost and build-time multipliers, build slowdown, destroys walls, content scan, or a land type's buildable value, so their resets are `unmeasured`. The unit of the two delays is given here as game minutes, but without an observable use it is not confirmed. The combat, movement, production, structures, repair and AI specs should settle whether each value has an effect, and in what unit. Once one does, a scenario on the pattern of D1 can be added here.
