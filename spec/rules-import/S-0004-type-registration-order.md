---
id: S-0004
title: Type registration order
topic: rules-import
revision: 20
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-02
measurements: []
requests: [Q-0005, Q-0083]
supersedes: []
---

# S-0004: Type registration order

## Summary

How the game builds its registries of object types from the rules data: which types exist, the index each one gets, what happens to types that are referenced but never listed, which fields create types by reference and in which order, and how the order of the type lists does (and does not) affect what the player sees, including the order of the build sidebar. How several rules files are stacked on top of each other is in S-0006.

**Paired sign-off.** Confirm claim 13 and scenario I1 against the S-0407 claim 8 (d) revision handed off with this spec: failed prerequisite-name lookup creates no type in either setting, but an invalid entry remains unmet with `quirks.prerequisite_names_dropped` off.

**Required change to S-0003 claim 14.** Its statement that both reserved names always mean no type must exclude the four particle and particle-system reference fields in claim 2 here. In those fields, bare `none` can create a type; the angle-bracketed spelling and empty values have the field-specific effects in that claim's table. S-0003 must defer those exceptions and their switches to this spec. Hand off the two revisions together after confirming that correction.

## Terms

The glossary defines *type kind*, *registry*, *registration index*, *type name*, *type list*, *game-wide section*, *base rules*, *rules layer*, *game*, *game start*, *departure* and *measurement setup*. This spec adds:

- **Layer.** In this spec, short for *rules layer*.
- **Creation by reference.** A type is created the first time a field names it, when no type of that kind and name exists yet. The field may belong to another type or to a game-wide section.
- **Reference field.** A field whose value names one type (a single reference) or a comma-separated list of types (a list reference) of a fixed kind. A *creating* reference field creates a type it names that does not exist yet. A *find-only* reference field never creates one (claim 13).
- **Reading pass.** The part of processing one layer in which each existing type of a kind reads its own section from that layer (claim 6). The second reading that warheads get in a layer with the special-weapon settings section (claim 6) is not a reading pass.
- **Early and late game-wide sections.** The two groups of game-wide sections that a layer reads before and after its reading passes (claim 4). Import mapping names them.
- **Sidebar tab.** One of the four columns of build icons (cameos) on the sidebar.
- **Engine defaults.** The values a type's fields have when the type is created, before it reads any section.
- **Display name.** The text a type shows the player, for example in its sidebar tooltip. It is resolved from the type's display-name field.
- **Theater control file.** The data file of the map's theater that declares its tilesets and the animations of its tiles. Its semantics belong to S-0010.

## Behaviour

This spec covers skirmish import. Network games and campaigns are out of scope.

Registration indices are non-negative integers. Indices are dense: the *n*-th type created for a kind has index *n* − 1, and indices are never reused within a game.

1. **Registries are rebuilt at every skirmish game start.** At each skirmish game start, every type registry is emptied except tiberiums (claim 10). Sides are emptied too, with one skirmish-specific difference: the result is as if, when processing of the base rules began, the side registry already held the sides that the base rules' countries' side fields name, in country-list order (claim 9). The registries are then refilled by processing the layers in order (S-0006). Two games started from the same data therefore get identical registration indices. The exception is tiberiums: a tiberium that an earlier game's map or game-mode file added keeps its index in later games of the same session (claim 10).
   **Deferred campaign exception.** This skirmish rule must not be read as saying that every campaign side is emptied before the base rules. In the original, a side that the mission-named file's side list or a country's side field creates remains at this game start, comes before the base rules' sides in side order, and stays for the mission (claim 9; S-0006 claim 14). No side of the previous game survives. This records the exception for consistency with S-0006, not a requirement to implement campaigns in the skirmish milestone.
   - Backing: `measurable` for the rebuilt overlay registry (scenario G, second game) and for tiberiums being kept (S-0006, scenario G). `unmeasured` for rebuilding the other registries individually: G observes only overlay indices, and no scenario part here isolates each remaining registry. `measurable` for sides being emptied (scenario E, E5). The deferred campaign side order is `unmeasured` in this milestone because campaign execution is outside scope; S-0006 scenario U2 is its future proposal.
   - Confidence: high (medium for sides, including the deferred campaign exception)

2. **A type list registers names in entry order.** The positions of a type list follow entry order in the file. At each position, the entry's key is looked up by name as S-0002 claim 4 specifies: if the key occurs more than once, the value of the winning occurrence is used at every position for that key, not necessarily the value written on that position's line. That value is a type name, and its effect depends on that name:

   | Entry value | Effect |
   |---|---|
   | empty | none |
   | `none` or `<none>`, in any letter case | none; no type is created |
   | the name of an existing type of the kind, compared without regard to letter case | none; the existing type keeps its index |
   | any other name | a new type with that name is created and gets the next index of its kind |

   Wherever a type name is looked up with creation allowed, `none` and `<none>` likewise mean "no type". There are four exceptions, all naming particles or particle systems. In each, `none` names a type called none, which is created if it doesn't exist, while `<none>` (any letter case), unlike `none`, names no type. The table gives what `<none>` and an empty value do in each:

   | Field | Kind named | `<none>` | Empty value |
   |---|---|---|---|
   | a particle system's emitted particle type | particles | no type | names a particle type with an empty name, created if needed (S-0006, claim 5) |
   | a particle type's next particle type | particles | no type | the field keeps its value |
   | an animation's spawned particle type | particles | no type | the field keeps its value |
   | a weapon's attached particle system | particle systems | the field keeps its value | the field keeps its value |

   An absent key keeps the field's value in all four, except the first, where an absent key counts as an empty value (S-0006, claim 5).

   For ordinary lists, an entry's key (the text left of `=`) does not itself name the type, but when a key is repeated it determines which value is used at each position, as S-0002 claim 4 specifies. The position of each entry counts. The side list and tiberium list are exceptions in which keys carry additional meaning (claims 9 and 10). Which occurrence of a repeated key supplies the value is defined in S-0002, and case rules for section and key names are in S-0001.
   - Backing: `measurable` for the existing-name and reserved-name rows (scenario A) and for the empty row (scenario A, A4: a section with an empty name exists when a file writes `[]` with entries under it, S-0001 claim 6). The particle exceptions: the empty-name case is measurable through S-0006 scenario D4. The `none` case is `measurable` for the emitted particle type (scenario A, A3) and for the attached particle system (scenario I, I4, which also covers its `<none>` and empty values). The emitted particle type called none reads no section in the layer that creates it (claim 6: particles come before particle systems), but it does read its section in every later layer (claims 6 and 7), so a particle system that names it in the base rules shows the particle section a test map gives it. The next-particle and spawned-particle rows are `unmeasured`: the spawned particle type is an art-file field of an animation, and a particle's next particle shows only inside an effect whose look belongs to the particle spec. Both use the same name lookup as the emitted particle type, which A3 measures.
   - Confidence: high (medium for the next-particle and spawned-particle rows)

3. **Some kinds have no list at all, and some only a list.** Weapons and projectiles have no type list. They come into existence only by reference, for example when a vehicle names a weapon or a weapon names a projectile. Every other kind in the glossary's *type kind* list has a type list.
   - Superweapons and tiberiums are created only by their own lists. A rules field that names an unknown superweapon (for example a building's superweapon field) means "none"; it does not create one. Likewise, an unknown name in a field that lists countries (for example a type's owner list) is ignored and creates no country.
   - Countries are created by the country list. Creation by campaign house definitions is deferred to the campaign specs.
   - Sides are created by the side list or by a country's own side field (claim 9).
   - Outside the rules layers, after all of them, three other inputs can create types, which then read no rules section: a map's terrain-object placements that name an unknown terrain object, the theater control file's tile animations that name an unknown animation, and, at game start, the particle system whose name the original fixes (the gas-cloud particle system, Import mapping), when no particle system of that name exists. Stock data lists that particle system. The order among these late creations is not covered here (open question 5).
   - All other kinds can also gain types by reference. Claim 13 says which fields create types and which only find them; Import mapping lists every such field.
   - Backing: `measurable` for weapons and projectiles by reference and for the unknown superweapon (scenario B). `unmeasured` for the unknown country in a field: no scenario here distinguishes an ignored name from a country created but never used by a skirmish house. `unmeasured` for the three creations outside the rules layers: such a type keeps the engine defaults of its kind, so a placed terrain object or tile animation of an unknown name shows nothing that tells "created" from "ignored", and stock data lists the fixed particle system.
   - Confidence: high (medium for the creations outside the rules layers)

4. **Listed types come before referenced types of the same layer.** Within a layer, the index a new type gets depends only on what first names it. Within each kind, every type in an earlier row of this table has a lower index than every type in a later row:

   | Order | Types | Read their section in the same layer? |
   |---|---|---|
   | 1 | types the layer lists (every type list except the tiberium list) | yes |
   | 2 | types first named by the early game-wide sections: general settings, AI settings, AI intelligence levels, power-ups, land types, jumpjet controls and multiplayer dialog settings | yes |
   | 3 | types first created during the reading passes (claim 6) | only if created no later than their own kind's pass, except warheads in a layer that has the special-weapon settings section (claim 6) |
   | 4 | types first named by the late game-wide sections: difficulty levels, crate rules, combat damage, radiation, elevation and wall models, audio-visual settings and special weapons | no, except warheads in a layer that has the special-weapon settings section (claim 6) |

   Two more groups follow row 4, in this order, at the end of the layer:
   - **Row 5:** types first created while warheads read their section a second time, in a layer that has the special-weapon settings section (claim 6). Only particle systems, animations and voxel animations can be created there. None of them reads its section in that layer, because the passes of their kinds are over; an animation reads its art in the next layer (claim 6).
   - **Row 6:** the tiberium list (claim 10), and the animations first created while a tiberium it names reads its section. Those animations likewise read no section in that layer.

   Where a section sits in the file plays no part. Only the row that first names a type counts. Within each row the order is fixed too:
   - Row 1: entry order within each type list (claim 2).
   - Row 2: types first named by the AI settings come before types first named by the general settings. Of the early sections, only these two name types.
   - Row 3: types first created during an earlier pass (in the rank order of claim 6) come before types first created during a later pass. Within one pass, types first created while a lower-indexed type reads its section come before types first created while a higher-indexed type reads its section.
   - Row 4: the late sections take effect in the order the table lists them (difficulty levels, crate rules, combat damage, radiation, elevation and wall models, audio-visual settings, special weapons), and a type first named by an earlier one of them comes first. Only the crate rules create a kind whose index a game shows (overlays), so the order among the other late sections is visible only through claim 12.
   - Row 5: types first created while a lower-indexed warhead reads its section come first.
   - Row 6: entry order of the tiberium list.
   - **Within one section.** Several types of one kind first created by the same section (one type's section, or one game-wide section) get distinct indices. This spec fixes their order in three cases only:
     - the items of a list reference field take effect in item order, and empty items and items that mean "no type" are skipped (S-0003, claim 12).
     - the crate rules name the wood-crate overlay first, then the crate overlay, then the water-crate overlay, whatever order the three keys have in the file (scenario G, G2).
     - the weapons that a type's weapon fields create follow the index order column of the weapon-slot table (claim 13).

     Only the crate-rules case can be seen in this spec (overlay indices). The other two are visible only through claim 12 in other specs, since this spec shows no weapon's index and creates no overlay from a list field. They are fixed so that a spec that picks "the first type" can rely on them.

     For every other pair of keys the relative order is not specified, but it must be the same in every game started from the same data. An implementation may choose any fixed order. A spec that picks "the first type" by index (claim 12) must not rely on the relative order of types created by different keys of one section unless it states that order itself. Where a key takes effect only under conditions (a type's weapon fields, claim 13), only the keys that take effect create types.

   Sides follow claim 9 instead of this table.

   Across layers, types first named by an earlier layer have lower indices than types first named by a later one. The order is the rules-layer order (S-0006): the base rules, then the language rules file if a valid one exists, then the game-mode file, then the map. A later layer can add types at the end of a registry. It can never remove a type, rename it or move it.

   The index of overlay types is visible, because a map's overlay data refers to overlay types by registration index (S-0009, claim 12). Overlays are created by their list (row 1), by a building's convert-to-overlay field and an animation's spawned-overlay field, both read from the art file during the reading passes (row 3), and by the crate rules (row 4). So overlays can show rows 1, 3 and 4, the order inside row 3 (earlier pass first, then lower-indexed creator first), the order within the crate rules, and the cross-layer append. In stock data every overlay these fields name is also in the overlay list.
   - Backing: `measurable` for the positions of rows 1, 3 and 4, for the cross-layer rule (scenario G, G1 and G3), for the order inside row 3 (scenario G, G4) and for the crate-rules order within a section (scenario G, G2: the three crate overlays). `measurable` for the order within row 1 (scenario G, G1: two listed overlays). `unmeasured` for the position of rows 2, 5 and 6, for the order within rows 2, 5 and 6, for the order among the late sections inside row 4 and for the list-item and weapon-slot orders within one section: no kind whose index a game shows is created by the early game-wide sections, by the second warhead reading, by the tiberium list or by a list field, and no weapon index is shown, so these can be seen only through claim 12 in other specs. The relative order of types that different keys of one section create is deliberately left unspecified (see above), so it needs no backing.
   - Confidence: high (medium for rows 5 and 6)

5. **Names are limited to 24 bytes.** Name lengths in this spec are counted in bytes, since rules documents are byte strings with no character encoding (S-0001, claim 1). A name written in plain ASCII has one byte per letter. A type stores at most the first 24 bytes of its name. A type reads its properties from the section whose name equals its stored name (claim 7). So a type listed under a longer name picks up only a section named with its first 24 bytes.

   A type-list value is looked up by its first 31 bytes, without the whitespace at the end of those 31 bytes (the text-value rule of S-0003, claim 1). This shows only in one case: a value made of an existing type's name of at most 24 bytes, followed by spaces that run past the 31st byte and then more text, finds that existing type. A value whose extra text starts at or before the 31st byte does not (claim 8). Reference fields cut their values in the same way, at their own limits (claim 13).
   - Backing: `measurable` (scenario C, C1 and C3)
   - Confidence: medium

6. **The rank of a type's kind decides whether a referenced type gets its properties.** Each kind has a fixed rank, and within each layer the result is as if the reading passes of the kinds took place in rank order:

   | Rank | Kind | Section read from |
   |---|---|---|
   | 1 | countries | this layer |
   | 2 | superweapons | this layer |
   | 3 | animations | the art file, in every layer |
   | 4 | buildings | this layer |
   | 5 | aircraft | this layer |
   | 6 | vehicles | this layer |
   | 7 | infantry | this layer |
   | 8 | weapons | this layer |
   | 9 | projectiles | this layer |
   | 10 | warheads | this layer |
   | 11 | terrain objects | this layer |
   | 12 | smudges | this layer |
   | 13 | overlays | this layer |
   | 14 | particles | this layer |
   | 15 | particle systems | this layer |
   | 16 | voxel animations | this layer |

   A type reads its properties in a layer only if it exists when its kind's pass runs. For a type created during the reading passes this gives a rule on ranks. Let a type of kind *J* be created by reference during the pass of kind *K* in some layer, that is, while a type of kind *K* reads its section. Then the new type reads its own section in that layer if rank(*J*) ≥ rank(*K*), and does not if rank(*J*) < rank(*K*), except that a warhead is read in a layer that has the special-weapon settings section (below). The case *J* = *K* is included: a type created during the pass of its own kind is read in that same pass. Each layer reads only its own file (S-0006), so a type left unread this way never reads the section that the creating layer holds for it. In particular, a type first created by reference in the base rules this way never reads its base-rules section. It keeps the engine defaults for its kind, plus any keys a later layer gives it.

   Types first named by the early game-wide sections exist before any pass, so they are read normally. Types first named by the late game-wide sections come after every pass. They are not read in that layer, with one exception: in a layer that contains the special-weapon settings section, every warhead reads its section from that layer a second time, after those settings. So in such a layer every warhead is read, including warheads first named by the late sections. A layer without that section (most maps) gives no second reading. In the original, the late sections can create vehicles, overlays, smudges, particle systems, weapons, projectiles, warheads and animations.

   Consequences:
   - Weapons named by buildings, vehicles, aircraft or infantry are always read, because weapons come later.
   - Projectiles and warheads named by weapons are always read.
   - A weapon that only a projectile names (as its airburst or shrapnel weapon) is not read, because weapons come before projectiles. It must also be named by a building, vehicle, aircraft, infantry type or another weapon to get its properties.
   - A building that only a vehicle names, for example as the structure it deploys into, is not read unless a type list or an early game-wide section names it.
   - An aircraft that only a vehicle names, for example as the aircraft it spawns, is not read unless listed.
   - A vehicle, aircraft or building that only an infantry type names is likewise not read.
   - A particle that only a particle system names is not read unless it is listed.
   - A weapon or projectile that only the combat-damage or special-weapon settings name is not read.
   - A vehicle that only the crate rules name is not read.
   - A warhead that only a particle type or a voxel animation names is not read in a layer without the special-weapon settings section.
   - A particle system that only a voxel animation names is not read unless listed.
   - Types named by a country or a superweapon are always read, and so are vehicles, aircraft, infantry, overlays and particle systems named by a building, because all of them come later in the order.

   Animations read from the art file in every layer. An animation first created after the animations' pass of a layer, that is, during the pass of a later-ranked kind or by the late game-wide sections, does not read its art in that layer, but it does in the next layer. The map layer is always the last layer and always follows the base rules, so only an animation first created after the animations' pass of the map layer stays without its art.

   Proposed switch: `quirks.unlisted_reference_unread` (default **on** = the YR behaviour above). With it off, a type created by reference still reads its section in the layer that created it, wherever its kind falls in the order. Stock data is written around this behaviour, so turning it off should not change stock games, but mods may rely on either form.
   - Backing: `measurable`: scenario B for the sampled unread consequences; I2 for a vehicle named by a building; J1 for the second warhead reading (with and without the special-weapon settings section); J2 for animation art in the next layer; G4 for animations before buildings before overlays. `unmeasured` for the remaining individual rank relations and their unsampled consequences, including same-kind creation being read in the same pass: the scenarios do not isolate them, and ordinary play exposes properties rather than all ranks. A finite same-kind reference-chain scenario is still needed; K5 tests non-completion, not the full rank table.
   - Confidence: high for the pass order and the unread consequence; medium for the second warhead reading and for animation art in the next layer (no measurement has been made yet).

7. **A type with no section of its own is left untouched.** If a layer has no section named after a type, that type reads nothing in that layer. Its values stay as they were, and nothing is reset. Section lookup uses the stored name, which may be truncated (claim 5). The stored spelling is the one the type was first registered with. Type-name lookup ignores letter case (claim 2), but section names are matched by exact bytes, except that S-0002 claim 3's name-hash identity rule treats different names with the same name hash as one name when `quirks.ini_hash_identity` is on. So a section spelled in a different case from the first registration is not found unless it shares that name hash.
   - Backing: `measurable` for exact section spelling and retention of the sampled fields (scenario H). `unmeasured` for the section-name hash exception individually: H deliberately excludes collisions, and S-0002 D4 isolates a colliding key rather than two section names. `unmeasured` for retention of every other field individually: H changes only the display name and observes health; it does not test every consumer.
   - Confidence: high

8. **Long names duplicate types.** Lookups compare the name being looked up against the stored, truncated name. The looked-up name is first cut to the length the field allows, and whitespace at the end of the cut text is removed (31 bytes for a type-list value, claim 5; the limits of reference fields are in claim 13). So a name longer than 24 bytes never matches the type created from it, and every later lookup of that name that may create a type creates yet another type of the kind. Each copy has the same truncated name and gets a new index. A lookup that may not create a type finds nothing.

   One field cuts shorter than the stored name: a weapon's attached particle system (19 bytes). For a particle-system name of 20 to 24 bytes, that field looks up only the first 19 bytes. It never finds a particle system registered under the full name; it creates one under the 19-byte name instead (once; later lookups by the same field find it), and that type reads the section named with those 19 bytes. A name of at most 19 bytes is unaffected.

   Proposed switch: `quirks.long_name_duplicates` (default **on**). With it off, names are truncated to 24 bytes before every comparison, and the attached-particle-system field reads up to 24 bytes instead of 19, so all references resolve to one type.
   - Backing: `measurable` (scenario C, C2; scenario I, I5, for the 19-byte field)
   - Confidence: medium

9. **Sides come from the side list and from each country's side field.** Every country records one side as its own (none until something sets it), and every side keeps a member list of countries. Side names are compared without regard to letter case.
   - **Side list.** Each entry's *key* is the side's name and its *value* is a list of countries. The side is created if no side of that name exists. Its member list is replaced by the listed countries, and each listed country records that side as its own.
   - **Country side field.** When a country reads its section (claim 6), a side field that is present names the country's side. An unknown side name creates a new side. If the side differs from the country's current side, the country is moved to the new side's member list (see the quirk below). An absent field keeps the current side.
   - **Order within a layer.** Within one layer a country's side field wins over the side list: the result is as if the layer's side list took effect before the countries' side fields of the same layer. A later layer's side list wins over an earlier layer's side field. A country that a later side list leaves out keeps the side it had: leaving a country out of a side's list does not take the side away from it.
   - **Names in the side list's value.** Each name is looked up as a country, matching either the country's type name or its name field, without regard to case. A name that is not a country but is the name of an existing side stands for all of that side's current members. Any other name is skipped. The countries must already exist, from the country list of the same layer or an earlier layer.
   - **Indices.** A side's index is the order in which sides are first created. In a skirmish, side indices follow the order in which the base rules' countries' side fields first name each side, taken in country-list order, and the side list only adds sides no country names. The result is as if, when processing of the base rules began, the side registry already held those country-named sides in that order (claim 1). With data in which country order and side-list order agree, as in stock data, the result is the same.

   Sides are cleared at every skirmish start, like the other registries (claim 1). The deferred campaign exception in claim 1 retains newly created mission-named-file sides ahead of any new sides from the base rules. S-0006 claim 14 owns that file's effects, its member-list safety departure and its quirk switch; campaigns remain out of scope here.

   Proposed switch: `quirks.side_change_removes_by_position` (default **on**). When a country's side field moves it away from a side, the member removed from the old side's member list is the one whose position in that list equals the country's registration index. If the list has no such position, no member is removed. So a different country may be removed, and the moved country may stay in the old list. With the switch off, the moved country itself is removed from the old list. Where the member list is used is outside this spec.
   - Backing: `measurable` (scenario E, E1 to E3) for the side list and the country side field. `measurable` for the order within a layer (scenario E, E9). `measurable` for the names in the side list's value (scenario E, E6: a country's name field; E7: a side's name; E8: an unknown name). `measurable` for ordering by countries' side fields rather than by the side list (scenario E, E4, with a loose copy of the base rules in which the two orders disagree). `unmeasured` for the quirk: whether it can be observed depends on where member lists are used (open question 1).
   - Confidence: medium. Low for the quirk, whose observable depends on how member lists are used.

10. **Tiberiums are matched by key number, not by name.** In the tiberium list, each entry's key is read as an integer *k* by the rule in Formulas (a leading decimal number; a key without one gives 0). The value is cut to 23 bytes. Let *n* be the number of tiberiums that exist when the entry takes effect. The entry's effect is:

    | Entry | Effect |
    |---|---|
    | empty value, any key | none; the entry is skipped |
    | non-empty value, 0 ≤ *k* < *n* | the tiberium with index *k* is used; the name in the value is ignored |
    | non-empty value, *k* ≥ *n* | a new tiberium, named by the value, is created at index *n*, the next free index. That index may differ from *k*: if two tiberiums exist and the entry's key is 5, the new tiberium gets index 2 |
    | non-empty value, *k* < 0 | see below |

    Each tiberium named by an entry reads its section from that layer, in entry order (claim 4, row 6): animations first created while it does so come after every animation the layer creates outside the tiberium list, and read no section in that layer. Unlike every other kind, a tiberium reads its section in a layer only if that layer's tiberium list has an entry for it: a layer that contains a tiberium's section but no list entry for it leaves the tiberium unchanged. A negative key refers outside the registry; the result is undefined in the original. This is a departure: our engine ignores such an entry, and the importer warns. Tiberiums are never removed, not even between games (S-0006, claim 10).

    Proposed switch: `quirks.tiberium_index_by_key` (default **on**). With it off, tiberiums are matched by name like other kinds.
    - Backing: `measurable` for the empty row (D2), the 0 ≤ *k* < *n* row (D1), the non-negative key parse including wrap (D3), and section-reading eligibility (D2 and D4's control). `measurable` for the *k* ≥ *n* row (D4) and the 23-byte cut (D5). S-0401, Terms, and S-0112, Terms, specify the resource-cell selection used by D4 and D5: the first registered tiberium whose image run contains the overlay index, or the first tiberium if none does. `unmeasured` for animations created by a tiberium reading only at the end of a layer and remaining unread there: D1 to D5 record resource payouts, not animation effects. `unmeasured` for failure on negative keys: no safe, defined original result exists to compare; ignoring and warning is our engine's design rule.
    - Confidence: medium (high for the key parse)

11. **Sidebar sorting does not use registration index as a key.** S-0407 claim 17 owns admission order when several types become offered at the same recheck. That order can affect the final positions of tied icons under the newest-first rule below, even though registration index is not a sorting key. Each buildable item goes to a fixed sidebar tab:

    | Item | Tab |
    |---|---|
    | buildings whose build category is "combat" (defences), and superweapons | 2nd |
    | other buildings | 1st |
    | infantry | 3rd |
    | vehicles and aircraft | 4th |

    The build category uses S-0302 claim 5's import rule: cut to 31 bytes, then trim surrounding whitespace, then compare the whole text without regard to ASCII letter case. A nonempty unknown word sets the unspecified category, not the earlier category; an absent or empty key keeps the held category, initially unspecified. Only the combat category selects the second tab; an unspecified category selects the first.

    A tab holds at most 76 items. An otherwise eligible item that becomes available while its tab is full is not shown. Eligibility therefore does not guarantee a shown icon. S-0411 claim 15 owns full-tab offer membership and order legality, capacity judgement before removals, delayed admission, and the absence of sidebar removal orders when a still-suppressed type loses availability. In particular, an icon removed at a recheck still occupies its place for that recheck's admission tests: the vacancy is not refilled at the same recheck (S-0411 claim 15, scenario S10). An item already shown (the same type) is not added twice. Two different types are two items even if their names are equal.

    Items are kept in sorted order. A newly available item is placed in front of the first item already in the tab that it sorts before. If it sorts before none of them, it is placed after all of them. "Item *a* sorts before item *b*" is decided by the first rule below that tells them apart:
    1. A superweapon sorts before anything that is not a superweapon. Two superweapons sort by recharge time in ticks (Formulas), shortest first, and then by display name as in rule 6, including its tie rule. Rules 2 to 5 do not apply to two superweapons.
    2. Of two non-superweapon items, the one whose base-planning side equals the local player's side sorts first.
    3. Category rank, lowest first. The counts-as-aircraft and naval flags count only for vehicle and aircraft types; every other item ranks as ground. An item with both flags ranks differently as the new item and as an item already in the tab:

       | Flags | Rank as the newly added item | Rank as an item already in the tab |
       |---|---|---|
       | neither (ground) | 0 | 0 |
       | counts as aircraft only (air) | 1 | 1 |
       | both | 1 | 2 |
       | naval only (naval) | 3 | 3 |

       So a new item with both flags is placed like an air item, and an item with both flags already in the tab sits between air and naval items: new ground and air items go before it, new naval items never do.

       An aircraft type whose data leaves the counts-as-aircraft key out has the flag (Import
       mapping), so, unless it also has the naval flag, it ranks as air (1), not as ground.
    4. Tech level (an integer), lowest first.
    5. Cost to the local player at the time the item is added, in whole credits, lowest first. This is the price the player would pay, country cost multipliers included; how it is computed belongs to the production spec.
    6. Display name, compared as in Formulas. When the names are equal too, the newly added item goes first. So among items identical in every key above, the one that became available last is shown first.

    The order is fixed when an item is added. Items are not re-sorted later, for example when a cost changes.

    Proposed switches: `quirks.sidebar_tie_newest_first` (default **on**; with it off, a full tie places the new item after the existing equal items) and `quirks.sidebar_both_flags_rank` (default **on** = the two-column rank above; with it off, an item with both flags ranks as air, 1, in both columns).
    - Backing: `measurable` (scenario F): F1 for the ordering cases present in stock data; F2 for the newest-first tie; F3 for the recharge-time key and S-0205 N5 for a positive wrapped key; F4 for the 76-item cap; F5 for the both-flags rank and `quirks.sidebar_both_flags_rank`; F6 for the base-planning-side rule (rule 2) ahead of tech level. F1 samples tab assignment, category, tech level, price and display-name ordering only where stock items differ in those keys. Category parsing is `measurable` by S-0302 D21 part b, recording the building's sidebar tab as well as its damage response. `unmeasured` for negative or extreme recharge-time keys beyond the positive wrapped example (no scenario here isolates them), remaining combinations, non-ASCII display-name comparisons and the same-name/different-type sidebar distinction beyond C2: no scenario isolates those cases. `unmeasured` for "items are not re-sorted later": a re-sort could show only if a sort key of an item already shown changed during a game. The other keys are fields of the type or the local player's side, which this spec treats as fixed for a game. Only the cost depends on the player's state, and whether and when it changes belongs to the production topic, so this spec has no scenario that changes a key in play.
      S-0501 scenario AC30 part j backs the air rank of an aircraft type without the
      counts-as-aircraft key.
    - Confidence: medium

    These sidebar rules are user-interface behaviour. This claim owns tab capacity, tab assignment and icon sorting and placement. S-0407 claim 17 owns simultaneous admission order; S-0411 owns icon states, clicks, removal and automatic production orders. S-0411 claim 15 owns full-tab offer membership and legality, suppressed-type removal orders and delayed admission.

12. **"The first type that…" means the lowest registration index.** Wherever another spec says that the game picks "the first type of a kind that satisfies a condition", first means the lowest registration index under claims 2 to 4. With unchanged data, that is the order of the base rules' type lists, followed by types created by reference. Between types created by different keys of one section, the order is unspecified (claim 4, "within one section"), so a spec that needs a particular one must state it.
    - Backing: `unmeasured`. This is a definition. Each spec that uses it states its own observable.
    - Confidence: high

13. **Which fields create types by reference.** Every field that names a type is either *creating* or *find-only* (Terms). Import mapping lists each one with its kind, its form and its text limit, section by section.
    - **When a field takes effect.** A field in a type's section takes effect when that type reads its section (claims 6 and 7); a field in a game-wide section takes effect when that section is read (claim 4). An absent key changes nothing and creates nothing. The one exception is the particle system's emitted particle type (claim 2): when a particle system's section is read without that key, the empty name is looked up with creation allowed, so a particle type with the empty name is registered by reference at that moment, at the end of the particle type list, unless one already exists (S-0616, claim 21). Being a particle type created during the pass of a later kind, it reads no section in that layer (claim 6).
    - **Creating fields.** A name that matches no type of the field's kind (compared as in claim 2, after the cut of claim 8) creates a new type with that name. It gets the next index of its kind by claim 4, and whether it reads its section in that layer follows claim 6. `none` and `<none>` mean "no type", except in the four fields of claim 2.
    - **List fields.** A list reference field is split into items as S-0003 claim 12 says. Each item is looked up in item order, and each item that names an unknown type creates it, so the items' new types get consecutive indices in item order (claim 4). Items that mean "no type" are skipped.
    - **Find-only fields.** These never create a type: a building's two superweapon fields; a type's prerequisite lists and the general settings' prerequisite-group lists (which also accept the generic prerequisite words, S-0003 Import mapping row C3); a type's country lists (owner, required, forbidden and secret countries; claim 3); and a voxel animation's share-source field, which can find an infantry type, a vehicle, an aircraft or a building. A name that matches no type of the kind finds nothing. For a type's prerequisite lists, the effect of an invalid entry follows S-0407 claim 8 (d) and `quirks.prerequisite_names_dropped` (default on = original): the entry is dropped with the switch on, but remains an unmet prerequisite with it off, even if a matching building is registered later. Neither setting creates a type by reference. For the other find-only fields, a failed lookup acts as if the field named no type. When types of several kinds have the share-source name, infantry win over vehicles, vehicles over aircraft and aircraft over buildings.
    - **Fields read from the art file.** Every field of an animation, and three reference fields of other kinds, are read from the art file: a building's convert-to-overlay field and the trailer animations of aircraft and projectiles. Claim 14 gives their section names and conditions. Animations read their art in every layer (claim 6). The art file is the same in every layer.
    - **Weapon slots.** Which weapon fields of a building, vehicle, aircraft or infantry type take effect depends on the type's turret-count field, its weapon-count field *c* and its clear-all-weapons flag. A weapon field that does not take effect creates nothing, even if its key is present.

      The three deciding fields act with the values the type holds once the section being read has supplied its values. A value supplied by that section takes precedence; otherwise the value already held—from an earlier layer or the engine default—decides. The fields' positions in the section do not affect the result. Thus a layer can change the turret count and weapon references together, and the references that apply under the resulting values take effect in that same layer.

      | Turret-count value | Clear-all-weapons value | References that take effect | Registration order of weapons they create |
      |---|---|---|---|
      | below 1 | off | primary, secondary, elite primary, elite secondary | primary, secondary, elite primary, elite secondary |
      | below 1 | on | none | — |
      | 1 or more | either | numbered normal and elite references from 1 through *c*; the four standard references do not take effect | normal 1, elite 1, normal 2, elite 2, …, normal *c*, elite *c* |

      Here *c* is the value of the weapon-count field. When *c* is below 1, no numbered reference takes effect. Claim 16 distinguishes registration above 18 from usable slots.

      With S-0308's `quirks.clear_weapons_any_turret_count` at its default original setting, a set clear-all-weapons value empties the first and second weapon slots and their elite slots regardless of turret count. With the switch off, it empties them only when the turret count is below 1. For a type with a turret count below 1, the slots are filled from the four standard references; with a turret count of 1 or more, they are filled from numbered references 1 and 2 and their elite counterparts. A clear-all-weapons value retained from an earlier layer also applies: a later layer's standard weapon reference does not fill a slot when the held flag remains set. A weapon type created by an applicable reference remains registered even if its slot is later emptied.
    - **Text limits.** A reference field's value is cut to the field's text limit, and whitespace at the end of the cut text is removed (S-0003, claim 1). Only then is it split into items or looked up. Every reference field has a limit of 127 bytes, which applies to the whole text of a list field, with three shorter limits, which apply to four fields: the emitted particle type, 63 bytes; a particle type's next particle type and an animation's spawned particle type, 31 bytes; a weapon's attached particle system, 19 bytes (claim 8). Type-list values are limited to 31 bytes (claim 5) and tiberium-list values to 23 (claim 10). A stored name has at most 24 bytes, so a limit above 24 never changes which type a long name matches. Such a limit shows in two ways:
      - **Single fields.** A value made of an existing type's name of at most 24 bytes, followed by spaces that run past the limit and then more text, finds that type: the cut removes the extra text, and the spaces before it go with the trailing whitespace. If the extra text starts at or before the limit, the value is a different name, and a creating field creates a new type under it (claim 8).
      - **List fields.** Items that start past byte 127 are lost, and an item that the cut splits keeps only its part before the cut (S-0003, claim 12). A creating list field then treats that part as a name of its own.

      The 19-byte limit is shorter than a stored name and shows directly (claim 8).
    - **No other field creates a type.** Fields not in the Import mapping tables create no type at import, even if their value is a type name.

    The kinds each section can create. Type sections and kinds are listed in rank order (claim 6) and game-wide sections in the order of claim 4; the listing order carries no meaning:

    | Section | Kinds it can create |
    |---|---|
    | a country | aircraft, vehicles, infantry; sides (claim 9) |
    | a superweapon | buildings, weapons |
    | an animation (art file) | animations, warheads, overlays, particles |
    | a building, vehicle, aircraft or infantry type | animations, buildings, aircraft, vehicles, infantry, weapons, particle systems, voxel animations; a building also overlays (art file) |
    | a weapon | animations, projectiles, warheads, particle systems |
    | a projectile | animations (art file), weapons |
    | a warhead | animations, particle systems, voxel animations |
    | a terrain object, a smudge | none |
    | an overlay | animations |
    | a particle type | warheads, particles |
    | a particle system | particles |
    | a voxel animation | animations, warheads, particle systems, voxel animations |
    | a tiberium | animations |
    | AI settings | buildings |
    | general settings | animations, buildings, aircraft, vehicles, infantry, weapons, warheads, terrain objects, particle systems, voxel animations |
    | crate rules | vehicles, overlays |
    | combat damage | animations, weapons, warheads, smudges, particle systems |
    | radiation | warheads |
    | audio-visual settings | animations |
    | special weapons | projectiles, warheads |
    | intelligence levels, power-ups, land types, jumpjet controls, multiplayer dialog settings, difficulty levels, elevation and wall models | none |

    Sides have no section of their own. Superweapons, tiberiums and countries are never created by a reference field (claim 3).
    - Backing: `measurable` (scenario I, which samples a creating single field, a creating list field, a find-only field, a numbered weapon field, the weapon-slot conditions (I10), `none` as a name and the 19-byte limit, and the text limits: I7 for the cut of a creating list field at 127 bytes, I8 for a single field at 127 bytes, I9 for the emitted particle type at 63 bytes; scenario G, G2 to G4, for the overlay fields). The cut of a find-only list field at 127 bytes is S-0003's scenario V1. `unmeasured` for the 31-byte limit of the next and spawned particle types, for the reasons claim 2 gives for those two fields. `measurable` for the priority among kinds in the voxel animation's share-source field (S-0007 A22 part e): distinct shared-model outcomes distinguish which kind won even though the field creates no type. `measurable` for which weapon fields take effect, and for the values the three deciding fields act with (scenario I, I3 and I10 runs a and b: the weapon a vehicle fires shows which field filled its slot), and for emptying the first normal slot by the flag, a held flag included (I10, runs c and d). `unmeasured` for the second and elite slot clearing: these runs do not make those slots' weapons fire, so no scenario part isolates their clearing. `unmeasured` for the exhaustive independence from field positions in a section: I10 tests selected positions but not every permutation. `unmeasured` for registration surviving slot clearing: the slot-firing tests do not distinguish a retained unused weapon type from a removed one. `unmeasured` for "a field that does not take effect creates nothing": a weapon that only such a field names would be carried by nothing, so the listed scenarios do not expose whether it exists. `unmeasured` for each unsampled field's creating/find-only classification, kind and form, and the completeness assertion: no scenario part here exercises those individual fields. The sampled fields do not establish the entire Import mapping table.
    - Confidence: high for the creating or find-only nature of each listed field, its kind and its form, and for the weapon-slot conditions, including the values the deciding fields act with. Medium for completeness. High for the kinds each section can create.

14. **Art-file reference sections.** A building's convert-to-overlay reference and an aircraft's trailer reference come from the art section named by the image name in force at that type read. The current layer's image field, when given, has already replaced the earlier image name. Without it, buildings and aircraft keep their earlier image name, initially their stored type name. Their snow suffix follows S-0007 claims 9 and 10: when those claims append A, the suffixed art section supplies these references in that same read. There is no separate unsuffixed lookup for these fields. Aircraft drawn as voxel models do not acquire this suffix.
    - A projectile instead takes its image name afresh from its rules section at each read, as S-0007 claim 26 specifies. Its trailer reference comes from the art section named by that non-empty image value, before any arctic change to the name used for drawing. The arctic flag therefore does not select a suffixed section for this trailer reference, even for a voxel projectile. A missing or empty image value prevents this trailer reference from being read at all in that layer: its earlier value remains, and no trailer type is created by that read. There is no fallback to the projectile's type name.
    - All three references require the current layer to contain the referring type's rules section (claims 6 and 7). The reference's absence or an empty value in the selected art section keeps its earlier value, initially no type. A reserved no-type name clears it, and any other name resolves or creates an animation or overlay by claim 13. A value in the rules section with the same key has no effect on these references.
    - Image names in these lookups are limited to 24 bytes, not Unicode characters (K7), consistent with S-0007's Terms and claims 1 and 26 and S-0001's byte-string input. The same byte limit applies to the 24-byte image-name suffix cut. No decoding is performed for these names.
    - Backing: `measurable` (K1 to K4 for the sampled section-selection, suffix and trailer gates; K7 for the byte limit). `unmeasured` for later-layer image replacement on an already arctic building under S-0007 claim 10: K2 does not isolate that later-layer case. `unmeasured` for retaining or clearing a building's earlier convert-to-overlay reference, and for the absence of another reference read when a referring rules section is absent: K4 does not isolate those effects because the art file is unchanged between layers and an already registered overlay cannot be unregistered. A scenario showing a use of the held building reference is still needed.
    - Confidence: high

15. **Unbounded creation through long names.** An animation whose own art section names a longer-than-24-byte version of its stored name as its next animation creates another copy. That copy reads the same art section during the same reading pass (claims 6 and 8), so the same reference creates another copy again. The original imposes no finite copy limit or repeated-name rejection on this process. Loading continues creating copies while resources permit instead of completing the pass. Resource exhaustion can stop further allocation or cause failure; no particular crash or final count is guaranteed.
    - **Departure for bad data.** Our engine suppresses a repeated creation within a same-layer chain of creating references. The repetition is a request for a kind and name already used to create the referring type itself or an ancestor of it in this layer, where the new type would read its section in this layer. Compare the requested names after their field text limits and trimming, without regard to letter case, not just their stored 24-byte forms. The suppressed reference means no type, receives no new index, and produces an importer warning. This leaves one new copy in the simple self-referencing example, rather than an unbounded number. The check does not suppress distinct long names sharing a stored prefix, independent references, references across layers, or references to an existing type. With the long-name quirk off, ordinary name resolution already finds the existing type and needs no suppression. This is a safety departure, not a new quirk switch.
    - Backing: `measurable` for non-completion versus a short-name control (K5). The safe result is a design rule, not an oracle claim. A final exhaustion count is intentionally unspecified.
    - Confidence: high

16. **Registration above the usable weapon-slot limit.** With a positive turret count, the numbered normal and elite weapon references take effect up to the imported weapon count, including numbers above 18. In particular, count 19 allows the nineteenth normal and elite references to create weapon types in that layer, in the interleaved order of claim 13. Their registration is not capped at 18. The original does not safely support using such an excessive count: its overall result is undefined, as S-0308 documents, so successful completion or firing cannot be promised.
    - S-0308 claim 16 and Edge cases own the safety departure: our engine retains registration effects from all applicable numbered fields, including section reads of the created weapon types by claim 6, while usable slots remain limited to the first 18. A missing numbered field creates nothing. Thus a large count need not entail processing a large number of absent fields, and a weapon above the slot limit can still name other types while its section is read. The excessive count produces the importer warning required by S-0308, not a new quirk switch.
    - Backing: `measurable` (K6, with an inconclusive result if the unsafe original fails to load). The split between safe slots and retained registrations is a design rule.
    - Confidence: high for count 19 and absence of a registration cap; medium for an observable surviving the original's unsafe input

## Formulas

**Key of a tiberium-list entry (claim 10).** The key *s* gives the integer *k*. The key is trimmed (S-0001, claim 10), and the trim always removes its leading whitespace, so *s* starts with its first non-whitespace byte. Split *s* as *s* = *p* *D* *r*, where *p* is one sign byte `+` or `-` or nothing, *D* = *d*₁ *d*₂ … *d*<sub>ℓ</sub> is the longest run of decimal digit bytes `0` to `9` that follows *p* (ℓ ≥ 0), and *r* is the rest of *s*, which plays no part.

> *k* = wrap₃₂(σ × *d*), with *d* = Σ<sub>*i*=1…ℓ</sub> *d*<sub>*i*</sub> × 10<sup>ℓ−*i*</sup> (so *d* = 0 when ℓ = 0), σ = −1 if *p* is `-` and σ = +1 otherwise.
> - wrap₃₂(*x*) is the integer in [−2³¹, 2³¹ − 1] that is congruent to *x* modulo 2³², so a key too large for 32 bits gives the wrapped value.
> - Exact integer arithmetic, then the wrap; no rounding applies. Number domain: signed 32-bit integers. Unit: none (the parsed tiberium-list key number, which is used as an index only when it falls within the existing registry).
> - No other form is recognised: `12abc` gives 12, `+7` gives 7, `-1` gives −1, and a key written in hexadecimal gives only its leading decimal digits (so a leading `0` followed by a letter gives 0).

This differs from the integer value forms of S-0003. Keys are never read in those forms.

**Superweapon recharge time as a sort key (claim 11, rule 1).** The recharge-time field gives a duration *m* in game minutes. Let *v* be the field's number read as a decimal value (S-0003, claims 6 and 7), a single-precision number, and let *c* be the double nearest to 0.01.

> *m* = *v* if the value contains no `%`, and *m* = RZ(*v* × *c*) if it contains a `%`. Unit: game minutes.
>
> *y* = RZ(*m* × 900), and *t* = 0 if |*y*| ≥ 2⁶³, *t* = wrap₃₂(trunc(*y*)) otherwise. Unit: ticks; *t* is a signed 32-bit integer.
> - *v* × *c* and *m* × 900 are floating-point products. Each RZ rounds its exact product to double precision, toward zero (the original's round-toward-zero mode, glossary; S-0003, claim 17). The two RZ are separate roundings.
> - trunc truncates toward zero to an integer; wrap₃₂ is as above, so a product whose magnitude is 2³¹ ticks or more, but below 2⁶³ ticks, wraps (S-0205, claim 7, the common conversion form; its scenario N5 measures a positive case on this sort key). A product of magnitude 2⁶³ ticks or more gives 0, whatever its value modulo 2³².
> - A field value of exactly 0 keeps the earlier value (S-0006).

The original's arithmetic runs in that mode (S-0003, claim 17). Its switch, `quirks.chop_rounding`, belongs to S-0205 (number domains). Whether it changes *t* depends on the form of the value:

- **Without a `%`,** *m* is a single (24 significant bits) and 900 needs 10, so the exact product has at most 34 significant bits and is exactly representable as a double, which has a 53-bit significand. RZ changes nothing, in either rounding mode; only the single precision of *m* shows. For example, a field of 0.3 is stored as 0.300000011920928955078125, the product is 270.0000107…, and *t* = 270. A field of 0.30001 also gives 270, so the two tie. A field of 0.7 is stored slightly below 0.7 (0.699999988079…), the product is 629.99998927…, and *t* = 629 in both rounding modes.
- **With a `%`,** *m* is a double that can use all 53 bits, and the rounding mode shows. For example, `30%` gives *m* = 0.299999999999999988897…, the double nearest to 0.3 (both modes agree on this step). The exact product 269.99999999999999000… rounds toward zero to 269.99999999999994316, so *t* = 269, where round-to-nearest gives exactly 270 and *t* = 270. Likewise `10%` gives 89 against 90, and `15%` gives 134 against 135; `50%` gives 450 in both modes.

Two superweapons are compared by *t*, an integer, so two fields that truncate to the same tick count tie and fall through to the display name.

**Display-name order (claim 11, rule 6).** Display names are compared as sequences of unsigned 16-bit character codes, from the first character on. The first differing code decides: the lower code sorts first. If one name is a prefix of the other, the shorter one sorts first. Equal sequences are a tie.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| stored name length | 24 | bytes | longer names are truncated (claims 5 and 8) |
| type-list value read length | 31 | bytes | cut before trailing spaces are removed and before lookup (claim 5) |
| tiberium list value length | 23 | bytes | claim 10 |
| reference field text limit | 127 | bytes | most reference fields, whole text of a list field (claim 13) |
| emitted particle type limit | 63 | bytes | claim 13 |
| next and spawned particle type limit | 31 | bytes | claim 13 |
| attached particle system limit | 19 | bytes | claims 8 and 13 |
| reserved "no type" names | `none`, `<none>` | type names | any letter case |
| sidebar tabs | 4 | tabs | claim 11 |
| items per sidebar tab | 76 | items | further items are not shown |
| recharge-time factor | 900 | ticks per game minute | claim 11, Formulas |

## State and ordering

| State | Event | Result |
|---|---|---|
| any | game start | all registries except tiberiums are emptied, then filled layer by layer (for sides, see claim 1) |
| type list entry names an unknown type | layer registration | new type appended, next index |
| type list entry names a known type (any case) | layer registration | nothing; index unchanged |
| a field names an unknown type during the reading passes | reading | new type appended; it reads its section in this layer only if the rank of its kind is at least the rank of the kind whose type was reading its section when the new type was created, or it is a warhead and the layer has the special-weapon settings section (claim 6) |
| layer has no section for a type | reading | type unchanged |
| otherwise eligible item not already shown | admission at an offer recheck (S-0407 claim 17; S-0411 claim 15) | if admitted under those rules, inserted in sorted position in its assigned tab; full-tab offer membership, legality, delayed admission and suppressed-type removal orders follow S-0411 claim 15 |

Observable ordering:
- Within a layer, every listed type of a kind has a lower index than every type of that kind first created by reference in the same layer, wherever the sections sit in the file. Sides (claim 9) and tiberiums (claim 10) follow their own claims instead.
- Later layers only append.

## Edge cases and quirks

- `quirks.unlisted_reference_unread`: claim 6.
- `quirks.long_name_duplicates`: claim 8.
- `quirks.tiberium_index_by_key`: claim 10.
- `quirks.sidebar_tie_newest_first` and `quirks.sidebar_both_flags_rank`: claim 11.
- `quirks.side_change_removes_by_position`: claim 9.
- `quirks.particle_none_is_a_name` (default **on**): in the four particle and particle-system fields of claim 2, `none` names a type called none. With it off, `none` means "no type" there as everywhere else, with the same effect as `<none>` in that field. The empty-value case of the emitted particle type belongs to S-0006, claim 5.
- `quirks.attached_system_none_keeps` (default **on**): a weapon's attached particle system keeps its earlier value when the field is `<none>` or empty (claim 2), so a later layer cannot remove an attached particle system by writing `<none>`. With it off, `<none>` clears the field (no attached particle system); an empty value still keeps it. The importer warns where `<none>` meets this field with the switch on, since it has no effect there.
- One audio-visual key feeds two fields (Import mapping), and each of the two is a creating field. With a name of at most 24 bytes, both fields name the same animation, which is created if no animation of that name exists. With a longer name, the two fields name two different animations with the same stored name (claim 8): each field creates its own. With `quirks.long_name_duplicates` off, both fields always name the same animation.
- Tiberiums kept between games: switch `quirks.persist_tiberium_overrides` in S-0006, claim 10.
- The superweapon recharge-time sort key depends on the *round-toward-zero mode* (Formulas). Its switch, `quirks.chop_rounding`, belongs to S-0205 (number domains); this spec defines none. The mode shows only for a field written with a `%` (Formulas).
- A side list naming a country that is not registered: the name is skipped (claim 9), unless it is a side's name.
- The reserved country name for a random choice, written in a side list, refers outside the country registry in the original. The result is undefined. Departure: our engine skips the name, and the importer warns. Backing: `unmeasured`; no safe, defined original result exists for comparison. Confidence: medium.
- A negative key in the tiberium list: undefined in the original. Departure: our engine ignores the entry and the importer warns (claim 10).
- Same-layer repeated long-name creation: the safety departure of claim 15 suppresses the repeated creation with a warning; no quirk switch.
- Excessive numbered weapon count: claim 16 preserves registrations while S-0308 caps usable slots and requires a warning; no new quirk switch.
- In one particular game-wide mode, the original adds no items to the sidebar at all. Which game option selects that mode, and whether it is reachable in normal play, is not established (open question 3). Backing: `unmeasured`, because no setup that activates it is known. Confidence: low.

## Import mapping

Key and section names in this section are the user's rules-data names, written exactly as the data spells them.

| Rules section | Kind | Entry form |
|---|---|---|
| `[Countries]` | countries | any key = country name |
| `[Sides]` | sides | side name = comma-separated country names (claim 9) |
| `[SuperWeaponTypes]` | superweapons | any key = name |
| `[Animations]` | animations | any key = name |
| `[BuildingTypes]` | buildings | any key = name |
| `[AircraftTypes]` | aircraft | any key = name |
| `[VehicleTypes]` | vehicles | any key = name |
| `[InfantryTypes]` | infantry | any key = name |
| none | weapons, projectiles | reference only |
| `[Warheads]` | warheads | any key = name |
| `[TerrainTypes]` | terrain objects | any key = name |
| `[SmudgeTypes]` | smudges | any key = name |
| `[OverlayTypes]` | overlays | any key = name |
| `[Particles]` | particles | any key = name |
| `[ParticleSystems]` | particle systems | any key = name |
| `[VoxelAnims]` | voxel animations | any key = name |
| `[Tiberiums]` | tiberiums | integer key = name (claim 10) |

Within one layer, the types that every list above except `[Sides]` and `[Tiberiums]` creates are row 1 of claim 4: they have lower indices than the types the layer creates by reference, and they read their sections in that layer. Sides follow claim 9, and `[Tiberiums]` is row 6 (claims 4 and 10). Every kind has its own registry, so the relative order of the other lists has no visible effect, with one exception: a side list can name countries that `[Countries]` lists in the same layer (claim 9).

Game-wide sections (claim 4):
- Early: `[General]`, `[AI]`, `[IQ]`, `[JumpjetControls]`, `[MultiplayerDialogSettings]`, the power-up section `[Powerups]` and the twelve land-type sections (S-0006). Of these, only `[General]` and `[AI]` name types in the original.
- Late: `[Easy]`, `[Normal]`, `[Difficult]`, `[CrateRules]`, the combat-damage section, `[Radiation]`, `[ElevationModel]`, `[WallModel]`, `[AudioVisual]` and `[SpecialWeapons]`. `[SpecialWeapons]` is the special-weapon settings section whose presence in a layer gives every warhead its second reading (claim 6).
- Animations read their sections from the art file, not from the layer (claim 6, rank 3; S-0006, claim 8).

Country keys (claim 9): side field = `Side` in a country's section (absent = keep); name field = `Name`, which side lists may use in place of the country's type name. Country lists in other fields, such as `Owner`, ignore unknown names (claim 3).

Fixed particle-system name (claim 3): the gas-cloud particle system = `GasCloudSys`. If no particle system of this name exists once all rules layers have been processed, one is created at game start, and it reads no rules section (claim 3).

Reference fields used in the scenarios: a building's superweapon field = `SuperWeapon` (find only, claim 3); a vehicle's deploy-into building = `DeploysInto`; a projectile's airburst and shrapnel weapons = `AirburstWeapon`, `ShrapnelWeapon`; a particle system's emitted particle type = `HoldsWhat` (claim 2); the crate overlays = `CrateImg`, `WoodCrateImg`, `WaterCrateImg` in `[CrateRules]` (overlay names, created by reference in the late phase); an object type's artwork = `Image`; a tiberium's credit value = `Value`; its image number selecting a resource-overlay run = `Image` (integer; absent or −1 keeps the run, initially no run; S-0112, claim 10).

**Reference fields (claims 4 and 13).** In each table, the keys of a section are grouped by the kind they name. The order of the groups and of the keys within a group carries no meaning: the tables are sets (groups in rank order, claim 6, with find-only groups last; keys alphabetical), except for the crate-rules row and the weapon-slot order of claim 13, whose order claim 4 fixes. "List" means a comma-separated list reference field; otherwise the field names one type. The text limit is 127 bytes unless an entry says otherwise. Every field in these tables creates its type unless the entry says *find only*.

Fields shared by buildings, vehicles, aircraft and infantry:

| Kind named | Keys | Form |
|---|---|---|
| animations | `DeployingAnim` (single); `DebrisAnims`, `DestroyAnim`, `Explosion` (lists) | as given |
| buildings | `DeploysInto` (single); `Dock` (list) | as given |
| aircraft | `AirstrikeTeamType`, `EliteAirstrikeTeamType`, `Spawns` | single |
| vehicles | `PowersUnit`, `UndeploysInto`, `UnloadingClass` | single |
| infantry | `Enslaves` | single |
| weapons | `DeathWeapon`, and the weapon fields of claim 13's weapon-slot table: the four standard fields `Primary`, `Secondary`, `ElitePrimary`, `EliteSecondary`, or the numbered fields `Weapon`*n* and `EliteWeapon`*n*, with turret-count field = `TurretCount`, weapon-count field *c* = `WeaponCount` and clear-all-weapons flag = `ClearAllWeapons`. Which of them take effect, and the order of the weapons they create, is that table's | single |
| particle systems | `NaturalParticleSystem`, `RefinerySmokeParticleSystem` (single); `DamageParticleSystems`, `DestroyParticleSystems` (lists) | as given |
| voxel animations | `DebrisTypes` | list |
| countries | `ForbiddenHouses`, `Owner`, `RequiredHouses`, `SecretHouses` | list, find only |
| buildings (and the generic prerequisite words) | `Prerequisite`, `PrerequisiteOverride` | list, find only |

Fields of one kind's own section:

| Section of | Keys by kind named (form) |
|---|---|
| a country | aircraft: `VeteranAircraft` (list); vehicles: `VeteranUnits` (list); infantry: `VeteranInfantry` (list); sides: `Side` (claim 9) |
| a superweapon | buildings: `AuxBuilding`; weapons: `WeaponType` |
| an animation (art file) | animations: `BounceAnim`, `ExpireAnim`, `Next`, `Spawns`, `TrailerAnim`; warheads: `Warhead`; overlays: `TiberiumSpawnType`; particles: `SpawnsParticle` (31 bytes; claim 2) |
| a building | buildings: `SecretBuilding`; vehicles: `FreeUnit`, `SecretUnit`; infantry: `SecretInfantry`; overlays: `ToOverlay` (art section selected by claim 14); superweapons, find only: `SuperWeapon`, `SuperWeapon2` |
| an aircraft | animations: `Trailer` (art section selected by claim 14) |
| a vehicle | none of its own |
| an infantry type | animations: `DeadBodies`, `DeathAnims` (lists); weapons: `EliteOccupyWeapon`, `OccupyWeapon` |
| a weapon | animations: `Anim` (list), `AssaultAnim`, `OccupantAnim`, `OpenToppedAnim`; projectiles: `Projectile`; warheads: `Warhead`; particle systems: `AttachedParticleSystem` (19 bytes; claim 2) |
| a projectile | animations: `Trailer` (art section selected by claim 14, only with a non-empty current `Image`); weapons: `AirburstWeapon`, `ShrapnelWeapon` |
| a warhead | animations: `AnimList` (list); particle systems: `Particle`; voxel animations: `DebrisTypes` (list) |
| a terrain object, a smudge | none |
| an overlay | animations: `CellAnim` |
| a particle type | warheads: `Warhead`; particles: `NextParticle` (31 bytes; claim 2) |
| a particle system | particles: `HoldsWhat` (63 bytes; claim 2) |
| a voxel animation | animations: `BounceAnim`, `ExpireAnim`, `TrailerAnim`; warheads: `Warhead`; particle systems: `AttachedSystem`; voxel animations: `Spawns`; infantry, vehicles, aircraft or buildings, find only: `ShareSource` (claim 13) |
| a tiberium | animations: `Debris` (list) |

Game-wide sections that name types, in the order claim 4 gives them (within one layer, of two types of one kind first named by two sections of this table, the one the earlier section names has the lower index):

| Section | Keys by kind named (form) |
|---|---|
| `[AI]` | buildings, all lists: `AlliedBaseDefenses`, `BuildAA`, `BuildBarracks`, `BuildConst`, `BuildDefense`, `BuildDummy`, `BuildHelipad`, `BuildNavalYard`, `BuildPDefense`, `BuildPower`, `BuildRadar`, `BuildRefinery`, `BuildTech`, `BuildWeapons`, `ConcreteWalls`, `EWGates`, `NeutralTechBuildings`, `NSGates`, `SovietBaseDefenses`, `ThirdBaseDefenses` |
| `[General]` | animations: `BarrelExplode`, `Behind`, `BombParachute`, `BridgeExplosions` (list), `ChronoBeam`, `ChronoBlast`, `ChronoBlastDest`, `ChronoPlacement`, `ChronoSparkle1`, `DamageFireTypes` (list), `DeadBodies` (list), `DominatorFirstAnim`, `DominatorSecondAnim`, `DropPod` (list), `DropZoneAnim`, `EMPulseSparkles`, `FlamingInfantry`, `ForceShieldInvokeAnim`, `InfantryBrute`, `InfantryExplode`, `InfantryHeadPop`, `InfantryMutate`, `InfantryNuked`, `InfantryVirus`, `IonBeam`, `IonBlast`, `IronCurtainInvokeAnim`, `MetallicDebris` (list), `MoveFlash`, `NukeTakeOff`, `OreTwinkle`, `Parachute`, `Wake`, `WarpAway`, `WarpIn`, `WarpOut`, `WeaponNullifyAnim`, `WeatherConBoltExplosion`, `WeatherConBolts` (list), `WeatherConClouds` (list); buildings: `GDIGateOne`, `GDIGateTwo`, `GDIPowerPlant`, `NodAdvancedPower`, `NodGateOne`, `NodGateTwo`, `NodRegularPower`, `PrismType`, `RepairBay` (list), `SecretBuildings` (list), `Shipyard` (list), `ThirdPowerPlant`, `WallTower`; aircraft: `CMislType`, `DMislType`, `PadAircraft` (list), `V3RocketType`; vehicles: `BaseUnit` (list), `HarvesterUnit` (list), `LargeVisceroid`, `PrerequisiteProcAlternate`, `SecretUnits` (list), `SmallVisceroid`; infantry: `AlliedCrew`, `AlliedDisguise`, `AllyParaDropInf` (list), `AmerParaDropInf` (list), `AnimToInfantry` (list), `Engineer`, `Paratrooper`, `Pilot`, `SecretInfantry` (list), `SovietCrew`, `SovietDisguise`, `SovParaDropInf` (list), `Technician`, `ThirdCrew`, `ThirdDisguise`, `YuriParaDropInf` (list); weapons: `DropPodWeapon`; warheads: `DominatorWarhead`, `LightningWarhead`; terrain objects: `DefaultMirageDisguises` (list), `VeinholeTypeClass`; particle systems: `BarrelParticle`; voxel animations: `BarrelDebris` (list), `ExplosiveVoxelDebris` (list), `ScrapVoxelDebris`, `TireVoxelDebris`; buildings, find only: `PrerequisiteBarracks`, `PrerequisiteFactory`, `PrerequisitePower`, `PrerequisiteProc` (lists), `PrerequisiteRadar`, `PrerequisiteTech` |
| `[CrateRules]` | vehicles: `UnitCrateType`; overlays, in this index order (claim 4): `WoodCrateImg`, `CrateImg`, `WaterCrateImg` |
| the combat-damage section, `[CombatDamage]` | animations: `ControlledAnimationType`, `DrainAnimationType`, `PermaControlledAnimationType`, `SplashList` (list); weapons: `DeathWeapon`; warheads: `C4Warhead`, `CMislEliteWarhead`, `CMislWarhead`, `CrushWarhead`, `DMislEliteWarhead`, `DMislWarhead`, `FlameDamage`, `FlameDamage2`, `IonCannonWarhead`, `IvanWarhead`, `V3EliteWarhead`, `V3Warhead`; smudges: `Scorches`, `Scorches1`, `Scorches2`, `Scorches3`, `Scorches4` (lists); particle systems: `DefaultDebrisSmokeSystem`, `DefaultFireStreamSystem`, `DefaultLargeGreySmokeSystem`, `DefaultLargeRedSmokeSystem`, `DefaultRepairParticleSystem`, `DefaultSmallGreySmokeSystem`, `DefaultSmallRedSmokeSystem`, `DefaultSparkSystem`, `DefaultTestParticleSystem` |
| `[Radiation]` | warheads: `RadSiteWarhead` |
| `[AudioVisual]` | animations: `AtmosphereEntry`, `Dig`, `DropPodPuff`, `LargeFire`, `OnFire` (list), `SmallFire`, `Smoke` (read twice, for two fields), `TreeFire` (list), `VeinAttack` |
| `[SpecialWeapons]` | projectiles: `EMPulseProjectile`, `NukeDown`, `NukeProjectile`; warheads: `EMPulseWarhead`, `MutateExplosionWarhead`, `MutateWarhead`, `NukeWarhead`; the second reading of every warhead (claim 6) also covers the warheads these keys first name |

`[IQ]`, `[JumpjetControls]`, `[MultiplayerDialogSettings]`, `[Powerups]`, the twelve land-type sections, `[Easy]`, `[Normal]`, `[Difficult]`, `[ElevationModel]` and `[WallModel]` name no types.

**Reference-section selection and safety (claims 14 to 16).**

| Input | Concept | Parse form | Absent or empty result |
|---|---|---|---|
| Rules, building or aircraft section, `Image` | art section for `ToOverlay` or `Trailer` | text, cut to 24 bytes; snow suffix as S-0007 | held image name, initially stored type name |
| Rules, projectile section, `Image` | art section for `Trailer` | text, cut to 24 bytes, afresh each read | empty image; no trailer-reference read |
| Rules, building or aircraft section, `AlternateArcticArt`; art, image section, `Voxel` | suffix eligibility for the first row | booleans; S-0007 owns their defaults and suffix timing | held values; initially no |
| Art, selected building image section, `ToOverlay` | convert-to-overlay reference | overlay name, creating, 127-byte limit | held reference, initially no type |
| Art, selected aircraft or projectile image section, `Trailer` | trailer reference | animation name, creating, 127-byte limit | held reference, initially no type |
| Art, animation's own section, `Next` | next-animation reference, including claim 15's unsafe self-reference | animation name, creating, 127-byte limit | held reference, initially no type |
| Rules, actor type section, `WeaponCount` | highest numbered weapon reference that takes effect with positive `TurretCount` | signed integer; claim 16 includes numbers above 18 for registration only | held value, initially 0 |
| Rules, actor type section, `Weapon`*n*, `EliteWeapon`*n* | normal and elite weapon registration for positive integer *n* ≤ weapon count | weapon names, creating, 127-byte limit | held references; no new creation |

Sidebar keys (claim 11): build category = `BuildCat` (combat = `Combat`; accepted words and parsing as S-0302 claim 5 and Import mapping give: 31-byte cut, then trim, whole ASCII-case-insensitive comparison; unknown words give unspecified, absent or empty keeps held, initially unspecified); base-planning side = `AIBasePlanningSide`; counts as aircraft = `ConsideredAircraft`; naval = `Naval`; tech level = `TechLevel`; display name = `UIName` (resolved to text); superweapon recharge time = `RechargeTime` (game minutes).

The absent default of the counts-as-aircraft key is *yes* for an aircraft type and *no* for a
vehicle type (S-0501 Import mapping).

## Measurement scenarios

Every scenario runs under the measurement setup (glossary): an unmodified install with no engine add-ons, and no loose override files except the named test files a sub-test lists, which are removed after that sub-test. Where a scenario needs extra rules entries, they go in the override sections of a purpose-made test map in the multiplayer map folder, never in the stock files. Sub-tests A3, D4, D5, E4, E5, F2, G2, G3, G4, I10 (run d) and J2 also use named test files: a loose copy of the base rules, a loose copy of the art file, or loose side archives (S-0005). Unless a sub-test says otherwise, run each variant once at game speed 2, crates off, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), with one human house and one bystander computer player. Keep the bystander outside the observed test objects or cells; apply its glossary void rule. Record icons and effects with screen capture, and health or credits with a read-only recorder. Start every run in a fresh session unless the sub-test explicitly prescribes consecutive games in one session; repeat unexpected results in a fresh session. For sidebar tests, grant the observing house the factories, prerequisites and credits needed by the setup. Resource tests unload exactly one full, single-resource load and use S-0402's payout formula, including any harvester income multiplier; record settled credits with no concurrent spending or other income. Test names use plain ASCII unless a test explicitly uses non-ASCII bytes.

**A. List order, re-listing and reserved names (claims 2 and 11).**
- A1 setup: a test map whose building list adds two new buildings, each a copy of an existing buildable structure, with identical tech level and cost and with different display names chosen so that the one listed second sorts first by rule 6. The list also repeats an already listed building in different letter case.
- A1 procedure: start a skirmish as a country that can build them and open the building tab.
- A1 observable: the icons in the building tab.
- A1 expected: both new buildings appear; the re-listed building appears once. The two new ones are ordered by claim 11 (here by display name, rule 6), not by list position.
- A1 falsified by: a duplicate icon for the re-listed building, or ordering by list position when the names differ.
- A2 setup: a test map whose building list has three extra entries under new keys, with the values `none`, `<NONE>` and `NONE`. The map also has sections named exactly `none`, `<NONE>` and `NONE`, each a copy of a buildable structure with its own display name.
- A2 observable: the building tab.
- A2 expected: none of the three display names appears, and the game starts normally.
- A2 falsified by: any of those display names appearing.
- A3 (the particle `none` case, claim 2 and `quirks.particle_none_is_a_name`). Named test file: a loose copy of the base rules in which one particle system changes only its emitted particle type to `none`. Choose a particle system that a stock effect easy to trigger in a skirmish visibly uses, for example one a weapon fires or one a damaged building emits. Control files: the same copy with `<none>` instead of `none`, and the same copy with a new particle name *P* instead of `none`.
- A3 setup: a test map with a particle section named exactly `none` (lower case, claim 7) that copies the keys of a clearly visible stock particle's section and sets the image field explicitly to that particle's type name, and a second section named *P* with the same keys.
- A3 precondition: with the stock files and the same map, the chosen effect visibly shows its particles. Otherwise pick another particle system.
- A3 procedure: one skirmish on the test map with each of the three files, triggering the effect.
- A3 observable: whether the effect shows particles, and whether they look like the copied particle.
- A3 expected: with `none`, the effect shows particles that look like the copied particle (the particle type named none was created in the base rules and read its section from the map). With `<none>`, it shows no particles. With *P*, it shows the same particles as with `none`.
- A3 falsified by: no particles with `none` while the *P* run shows them (that would mean `none` means "no particle type" there), or particles with `<none>`. If the *P* run shows no particles, the copied section did not bind and the run is inconclusive.
- A4 (the empty row): a test map whose building list has an extra entry under a new key with an empty value, and a section written `[]` that copies a buildable structure's keys with its own display name. Observable: the building tab. Expected: that display name does not appear, because the empty entry creates nothing. Falsified by: the display name appearing (a building with an empty name was created and read the `[]` section).

**B. Unread late references (claims 3 and 6).**
- B1 setup: in the test map, give an existing vehicle that deploys into a structure a new structure name. Define that structure's section in the map but leave it out of the map's building list. Control map: identical, but with the new name added to the building list.
- B1 procedure: deploy the vehicle on each map.
- B1 observable: whether the deployed structure has the properties written in its section (health bar, footprint, function).
- B1 expected: on the first map the structure ignores its section and has the engine defaults, which may make the deploy fail. On the control map it behaves per its section.
- B1 falsified by: both maps giving the section's properties.
- B2 (projectiles by reference, claim 3): a test-map weapon, carried by an existing unit, names a new projectile that appears in no list, and the projectile's section in the map gives it a visibly different look or trajectory. Expected: the weapon fires with that projectile's properties. Falsified by: the shots showing engine-default projectile behaviour, or the weapon not firing.
- B3 (weapon named only by a projectile, claim 6): a test map with a target building of known health on open ground, and one unit of an existing type whose primary weapon the map sets to a new test weapon. The test weapon does zero damage, and its projectile names a new weapon as its shrapnel or airburst weapon. That new weapon is named nowhere else, and its section in the map gives it a clearly visible damage. Control map: the same, plus an existing unit type that names the new weapon as its secondary weapon. Procedure: order the unit to attack the target, let it fire a fixed number of shots on each map, then stop it. Precondition: on both maps the shots must visibly fire and burst into fragments; if they don't, the run is inconclusive. Observable: the health the target lost, from its health bar or a health readout, on each map. Expected: on the first map no damage, or only engine-default fragment damage; on the control map the damage the new weapon's section implies. Falsified by: equal health loss on both maps while the precondition holds.
- B4 (unknown superweapon, claim 3): a test map lists a new building, a buildable copy of an existing structure, whose superweapon field names a name that is in no superweapon list. The map also has a section under that name copying an existing superweapon's keys. Procedure: build the new building. Observable: the superweapon tab and any countdown. Expected: no new superweapon icon or countdown appears. Falsified by: one appearing.

**C. Name length (claims 5 and 8).**
- C1 setup: a test map listing a new building with a 28-byte name. Give it two sections, one named with the full name and one named with its first 24 bytes, with different display names.
- C1 observable: which display name the sidebar shows.
- C1 expected: the one from the 24-byte section.
- C1 falsified by: the full-name section's display name.
- C2 setup: as C1, but the building list names the same 28-byte value twice, under two different keys (S-0002), and only the 24-byte section exists.
- C2 observable: the number of icons with that display name.
- C2 expected: two identical icons, one per created type.
- C2 falsified by: a single icon.
- C3 setup: a test map listing a new buildable building whose name has exactly 24 bytes, with its section. The list has a second entry under another key whose value is that name, then 7 spaces, then one more letter (32 bytes in all). Control: the same with 6 spaces (31 bytes in all).
- C3 observable: the number of icons with the building's display name.
- C3 expected: one icon on the first map (the cut at 31 bytes removes the letter, and the trailing spaces are removed). Two icons on the control map, because the letter survives the cut and the second entry creates a separate type that also reads the 24-byte section.
- C3 falsified by: two icons on the first map, or one on the control.

**D. Key numbering of tiberiums (claim 10).**
- D1 setup: a test map whose tiberium list has one entry, keyed 0, naming an unknown tiberium. Give that name's section a credit value different from the first ore type's.
- D1 observable: the credits gained when one harvester unloads one full load of the first ore type at a refinery.
- D1 expected: unchanged from a stock map, because key 0 picks the existing first tiberium and the new name is ignored.
- D1 falsified by: a gain matching the new section's value.
- D2 setup: a test map that changes the first ore type's credit value in that ore's own section, with no tiberium list in the map. Repeat with a tiberium-list entry keyed 0 whose value is empty, and with a non-empty entry keyed 0 as a positive control. Procedure and observable as D1. Expected: unchanged payout in the absent-list and empty-value runs, changed payout only in the positive control. Falsified by: the changed payout in either first run while the positive control changes it. An unchanged positive control makes the test inconclusive.
- D3 setup: as D1, but change the existing first tiberium's own section to a different credit value as well, so processing key 0 is visible. Run separate maps with keys `0abc`, `+0tail`, `word`, a key consisting of `0` followed by `x10`, `4294967296` and `-4294967296`. Procedure and observable as D1. Expected: all select index 0 and pay its changed value, not the unknown name's value. Falsified by: any different payout after a plain key `0` control pays the changed value. The two large keys test wrap modulo 2³², not negative-index access.
- D4 (the *k* ≥ *n* row). Let *L* be the tiberium of the last entry of the base rules' tiberium list, and *i* its index (its position in that list, counted from 0). Choose a purpose-made map with flat resource cells selected for *L* by the image-run rule below, and confirm that they pay *L*'s credit value in the control. Each run starts in a fresh session, because tiberiums survive between games (claim 10). Check that the chosen game mode's file and the language rules file, if any, have no tiberium list; otherwise *n* differs from the value below. Observable: the credits gained when one harvester unloads one full load from that ore field.
  - Control: stock files and a copy of the map whose tiberium list has one entry, keyed *i*, naming *L*, and whose section for *L* copies every key of *L*'s effective base-rules section, including any image-number reassignment described below, with a different credit value *v*. Expected: a gain matching *v* (the 0 ≤ *k* < *n* row: *L* reads the map's section). Use S-0401's selection rule to choose flat resource overlays in *L*'s image run, outside every earlier tiberium's run. If stock image runs provide no such cells, a named loose base-rules copy may assign *L* image number 1 and every earlier tiberium image number 3 (Import mapping; runs in S-0112), with the same assignments in all runs; use a stock resource overlay in *L*'s flat run. An unchanged control payout means the selection or harvest setup failed, so the scenario is inconclusive.
  - Named test file for the test runs: a loose copy of the same base rules used by the control, including any image-number reassignments, whose tiberium list leaves out only *L*'s entry. Then *n* = *i* when the map's list is read.
  - Run 1: the same test map as the control, but the entry is keyed *i* + 3. Expected: a gain matching *v*. The entry creates *L* at index *i*, the next free index, and *L* reads the map's section.
  - Run 2: as run 1, but keyed *i*. Expected: as run 1.
  - Falsified by: no gain, or a gain other than *v*, in run 1 while run 2 matches the control (that would mean an entry keyed above *n* is skipped, not created at the next free index). If neither run reaches play or neither pays, the run is inconclusive (an ore field without its tiberium may behave in ways this spec does not cover).

- D5 (the 23-byte value cut). Named test file: the loose base-rules copy of D4 with the last tiberium *L* omitted, retaining any image-run separation that D4 required. Each run starts in a fresh session; no intervening language or mode layer lists tiberiums. The test map lists a new tiberium under key *i*, using a 28-byte ASCII name *T*. Give both the full-name section and the section named by its first 23 bytes complete copies of *L*'s resource settings, including the same image number, but different positive credit values *v_full* and *v_cut*. Place flat resource cells in the corresponding run as D4 specifies. Procedure: harvest and unload one full load from these cells. Observable: settled credits gained, using the payout calculation stated in the preamble. Controls: separate maps list only the 23-byte name or a short full-section alias, with otherwise identical resource settings; confirm that their payouts distinguish *v_cut* from *v_full*. Expected: the long-name map pays the cut-section amount. Falsified by: the full-section amount or another amount while both controls bind and produce distinct payouts. If resource selection fails or the controls coincide, the test is inconclusive.

**E. Sides from the side list and the country field (claims 1 and 9).**
- E1 setup: a test map whose side list re-lists an existing side with one country removed.
- E1 observable: the side that country plays in a skirmish: its sidebar items and its starting units.
- E1 expected: no change. The country keeps the side it had from the base rules.
- E1 falsified by: the country losing its side's items or units.
- E2 setup: a test map that gives one country's section a side field naming a different existing side.
- E2 observable: as E1.
- E2 expected: the country plays as the new side.
- E2 falsified by: the country keeping its old side's items or units.
- E3: play a stock map afterwards in the same session. Expected: stock side membership, because the base rules set every country's side again. Falsified by: the test map's side change carrying over. (This does not tell a rebuilt side registry from a kept one; E5 does.)
- Sub-tests E4 and E5 observe a side's index through the local player's side archive number (S-0005, claim 13a): the side group mounted for the player's country is chosen by its side index, and the sidebar's radar frame shows which group it is. S-0005 scenario M24 uses the same observable. With stock data, the Allied side's group and the Soviet side's group look different.
- E4 (the two side-index orders, claim 9). Named test file: a loose copy of the base rules in which the first two entries of the side list (the Allied side and the Soviet side) swap places, with their values unchanged. Nothing else changes, so the countries' side fields still name the Allied side first.
- E4 procedure: play a skirmish on a stock map as an Allied country, once with the swapped side list and once with the unmodified list, each in a fresh session.
- E4 observable: the look of the sidebar's radar frame, or whether the game reaches play.
- E4 expected: the Allied look in both runs. The Allied side keeps index 0 because the indices follow countries' side fields, not the swapped side list.
- E4 falsified by: the Soviet look with the swapped list while the unmodified control shows the Allied look. Failure to reach play is inconclusive.
- E5 (sides are emptied at every game start, claim 1). Let *S* be the number of sides a fresh skirmish has: the distinct side names that the base rules' side list and the base rules' countries' side fields name (count them in the file). A new side created by a map then gets index *S* if the side registry was emptied at game start, and *S* + 1 if a new side from the previous game is still there.
- E5 named test files: a loose side archive for index *S* (its archive number from S-0005, claim 13a) that is a copy of the stock Allied side's archive, and one for index *S* + 1 that is a copy of the stock Soviet side's archive.
- E5 setup: test map 1 sets the side field of a country nobody plays to a new side name *T1*. Test map 2 sets the side field of the country the local player plays to a different new side name *T2*.
- E5 procedure: in one session, play a skirmish on map 1 until it reaches play, return to the menu, then play a skirmish on map 2 as the country map 2 changes. Control: map 2 alone, in a fresh session.
- E5 observable: the look of the sidebar's radar frame on map 2.
- E5 expected: the Allied look in both runs, because the registry was emptied and *T2* got index *S*.
- E5 falsified by: the Soviet look after map 1, which would mean *T1* was kept and *T2* got index *S* + 1. If the control does not show the Allied look, or does not reach play, the run is inconclusive.
- Sub-tests E6 to E8 (names in the side list's value) and E9 (the order within a layer), all for claim 9, record the side a country plays: the look of its sidebar's radar frame (as in E4) and its starting units. Each test map re-lists one stock side, with the value that side has in the base rules plus the extra names given below, and, except where E9 says otherwise, has no country sections of its own. Each map is played in a skirmish as the named country.
- E6 (a country's name field). Choose an Allied country *C* whose name field differs from its type name in more than letter case. The map re-lists the Soviet side with, added at the end, *C*'s name-field value. Control map: the same with *C*'s type name. Expected: *C* plays with the Soviet look and Soviet starting units on both maps. Falsified by: the Allied look on the first map while the control shows the Soviet look.
- E7 (a side's name). The map re-lists the Allied side with, added at the end, the Soviet side's name. Play as a Soviet country. Expected: the Allied look and Allied starting units, because the Soviet side's name stands for all its members, which then record the Allied side. Falsified by: the Soviet look.
- E8 (an unknown name). The map re-lists the Soviet side with, added at the end, a name *U* that is neither a country nor a side, then the type name of an Allied country *C*. Play as *C*. Expected: the game reaches play, and *C* plays with the Soviet look, because *U* is skipped and the rest of the value still takes effect. Falsified by: the game not starting, or *C* keeping the Allied look.
- E9 (within one layer, a country's side field wins over the side list, claim 9). Choose an Allied country *C*. The test map re-lists the Soviet side with its base-rules value plus *C*'s type name at the end, and gives *C* a section that copies every key of *C*'s base-rules section, with its side field naming the Allied side. Control map: the same side list, with no section for *C*. Play each map as *C*. Expected: the Allied look and Allied starting units on the test map, because the map's side list first moves *C* to the Soviet side and *C*'s own side field then moves it back. On the control map, the Soviet look and Soviet starting units. Falsified by: the Soviet look on the test map while the control shows the Soviet look. If the control shows the Allied look, the side list did not take effect and the run is inconclusive.

**F. Sidebar order (claim 11).**
- F1 setup: a skirmish with stock data. Screenshot each tab once all tech is available, for example after a long game.
- F1 observable: the order of the icons.
- F1 expected: the order follows the rules of claim 11 (superweapons first, then side, category, tech level, cost, name).
- F1 falsified by: any adjacent pair whose order contradicts those rules, given the items' fields and the order in which they became available.
- Category parsing: repeat S-0302 D21 part b under that scenario's setup and controls, making each tested building buildable and granting its factories and prerequisites. Record its build icon's tab in each variant as well as the damage observable. Expected: every variant parsed as combat appears in the second tab, while unknown words, including an unknown word replacing combat in a later layer, appear in the first. The opposite tab with a working buildability control falsifies claim 11's use of that category parser.
- F2 setup, for ties: a test map defining two copies of the same stock vehicle with identical sidebar sort fields, including tech level, cost and display name, unlocked by different prerequisite buildings. Named test file: a loose art copy with two new image sections copying that vehicle's art settings but selecting distinct existing stock main cameo pictures. Give each test vehicle the corresponding image name, so its icon can be identified without changing any sort field. No vehicle needs to be produced.
- F2 procedure: build the two prerequisites in either order, in two runs.
- F2 observable: the relative positions of the two vehicle icons, identified by distinct cameo pictures that do not change any sort field.
- F2 expected: the vehicle unlocked last appears first.
- F2 falsified by: list order deciding the order, or the earlier-unlocked vehicle appearing first.
- F3 setup, for the recharge-time rounding (Formulas): a test map whose superweapon list adds two new superweapons, each a copy of an existing one, granted by two new buildable buildings. Superweapon *A* has its recharge time written as `30%` (0.3 minutes, S-0003 claim 7) and superweapon *B* as `0.3`, and their display names are chosen so that *B* sorts first by display name.
- F3 procedure: build both buildings and look at the superweapon icons once both are shown, in two runs that build them in either order.
- F3 expected: *A* before *B* in both runs, because *A* has 269 ticks (the product rounded toward zero) and *B* 270 (an exact product).
- F3 falsified by: *B* before *A* in both runs, which would mean the two tie at 270 ticks and the display name decided (round-to-nearest).
- F3 control: a second pair, *C* written as `0.3` and *D* as `0.30001`, names chosen so that *D* sorts first by display name. Expected: *D* before *C* in both runs, because both have 270 ticks and the display name decides. *C* before *D* would mean the field is held at more than single precision (S-0003, claim 6).
- F4 (the 76-item cap). Setup: a test map whose building list adds 80 new buildings, each a copy of a simple stock structure that is not a defence and needs only the construction yard, each with its own display name. Procedure: deploy the starting construction vehicle, then build a power plant. Observable: the number of icons in the building tab (scroll through it), after each step. Expected: exactly 76 icons after each step; the items that become available while the tab is full, such as those the power plant unlocks, are not shown. Falsified by: more than 76 icons, or fewer than 76 when a reduced-count control shows that all added types are independently buildable.
- F5 (the both-flags rank and `quirks.sidebar_both_flags_rank`). Setup: a test map with water, where the local player owns a war factory and a naval yard from the start or builds them first. It defines two new vehicles with identical tech level, cost and base-planning side: *A*, a copy of a stock land vehicle with the counts-as-aircraft flag set and the naval flag cleared, and *V*, a copy of a stock naval vessel with both flags set. Their display names are chosen so that *V* sorts before *A* by rule 6. Each is unlocked by its own new prerequisite building. Precondition: both icons appear in the vehicle tab. Procedure: run 1 builds *V*'s prerequisite first, then *A*'s; run 2 builds *A*'s first, then *V*'s. Observable: the relative order of *A* and *V* in the vehicle tab. Expected: run 1, *A* before *V* (as the new item, *A* ranks 1 against *V*'s 2 as an item already in the tab); run 2, *V* before *A* (as the new item, *V* ranks 1, ties with *A* and sorts first by display name). Falsified by: *V* before *A* in run 1 (that is the order with the switch off, where both rank 1 in both columns), or *A* before *V* in run 2.
- F6 (the base-planning-side rule, rule 2). Setup: a test map defining two new vehicles *X* and *Y*, copies of a stock buildable vehicle with equal cost. *X*'s base-planning side is the local player's side index, and *Y*'s is the index of another side (claim 9 gives the indices). *Y*'s tech level is lower than *X*'s, both within the range of buildable stock vehicles, and their display names are chosen so that *Y* also sorts first by rule 6. Each is unlocked by its own new prerequisite building. Procedure: two runs, building the prerequisites in either order. Observable: the relative order of *X* and *Y*. Expected: *X* before *Y* in both runs, because rule 2 decides before tech level. Falsified by: *Y* before *X* in either run.

**G. Overlay indices: listed before late references, later layers append (claims 1, 4 and 13).**
- G1 setup: a test map with format version 2 or higher (S-0009, claim 11) in a theater where two existing overlay types have clearly different visible artwork, *P* and *Q*, neither of them a crate type. Let *N* be the number of entries in the base rules' overlay list (count them in the file). Check that the chosen game mode's file adds no overlay types; the scenario needs *N* + 1 ≤ 254.
  - The map's crate-rules section names a new overlay name *Y* as a crate overlay. Put that section *above* the overlay list in the file. *Y* gets no section.
  - The map's overlay list names two new overlay types, *X1* then *X2*, with sections that copy the keys of *P*'s and *Q*'s sections respectively and also set the image field explicitly to *P*'s and *Q*'s type names, so that *X1* looks like *P* and *X2* like *Q*. Choose *P* and *Q* among simple decorative overlays, not walls, crates or resources.
  - The map's overlay data places overlay index *N* on one marked cell, *N* + 1 on a second marked cell, and *N* − 1 on a third.
- G1 second map (claim 1): a second test map, same theater and format, whose overlay list names one new overlay type *Z* set up to look like *Q* (as above), with overlay index *N* on one marked cell. It has no crate-rules section.
- G1 procedure: start a skirmish on the first map and look at the three cells. Then return to the menu and, in the same session, start a skirmish on the second map and look at its cell.
- G1 observable: the artwork on each marked cell.
- G1 expected: on the first map, cell *N* shows *P*'s artwork (*X1*), cell *N* + 1 shows *Q*'s artwork (*X2*), and cell *N* − 1 shows the last overlay of the base list. On the second map, cell *N* shows *Q*'s artwork (*Z*), because the registry was rebuilt and *Z* is the first type past the base list.
- G1 falsified by: on the first map, *P*'s artwork at cell *N* + 1, or nothing at cell *N* + 1 while *P*'s artwork is at cell *N*. The first would mean *Y* took an index before both listed types, the second that it took one between them. *Q*'s artwork at cell *N* also falsifies: it would mean *X2* took the first new index, against entry order (claim 2). On the second map, *P*'s artwork at cell *N* (the first map's *X1* kept) falsifies claim 1. If *P*'s and *Q*'s artwork appear on no cell of the first map, the run is inconclusive (the artwork did not bind), not a falsification.
- G2 (order within one section: the three crate overlays). Named test file: a loose copy of the base rules whose crate-rules section names three new overlay names, *Y1* as the wood-crate overlay, *Y2* as the crate overlay and *Y3* as the water-crate overlay, with the three keys written in the file in the reverse order (water crate first). Nothing else changes. Choose three simple decorative overlays *P*, *Q* and *R* with clearly different artwork, as in G1.
  - Setup: a test map (as in G1, but the scenario needs *N* + 2 ≤ 254) whose overlay data places overlay index *N* on one marked cell, *N* + 1 on a second and *N* + 2 on a third. The map gives *Y1*, *Y2* and *Y3* sections that copy the keys of *P*, *Q* and *R* respectively and set the image field explicitly to those type names. The three are created by the late crate rules of the base rules and read nothing there, but they read the map's sections in the map layer (claims 6 and 7).
  - Observable: the artwork on the three cells.
  - Expected: *P*'s artwork at *N*, *Q*'s at *N* + 1, *R*'s at *N* + 2: wood crate, crate, water crate, whatever their order in the file.
  - Falsified by: any other assignment, in particular *R*'s artwork at *N* (file order). If none of the three artworks appears, the run is inconclusive.
- G3 (row 3 between rows 1 and 4). Named test file: a loose copy of the art file with one added section *B* that copies the art section of an existing buildable structure and adds a convert-to-overlay field naming a new overlay *W*.
  - Setup: a test map (as in G1, but the scenario needs *N* + 2 ≤ 254) whose building list adds a new building *B* with a section copying that structure's rules keys (no image field, so its art section is *B*). The map's overlay list names a new overlay *X1*, and its crate-rules section names a new overlay *Y* as a crate overlay, placed above the overlay list in the file. The map gives *X1* and *W* sections that look like *P* and *Q* (as in G1); *Y* gets no section. The overlay data places index *N* on one marked cell, *N* + 1 on a second and *N* + 2 on a third.
  - Precondition: *B* appears in the building tab with the build icon its art section in the loose copy gives it; otherwise the loose art copy did not take effect and the run is inconclusive.
  - Procedure: start a skirmish and look at the three cells. *B* need not be built: *W* is created while *B* reads its section.
  - Expected: *P*'s artwork at *N* (*X1*, row 1), *Q*'s at *N* + 1 (*W*, created during the buildings' pass, row 3, and read in the overlays' pass of the same layer), nothing at *N* + 2 (*Y*, row 4, unread).
  - Falsified by: *Q*'s artwork at *N* (row 3 before row 1), or *Q*'s artwork at *N* + 2 with nothing at *N* + 1 (row 4 before row 3). No *Q* artwork anywhere while *P*'s shows means the convert-to-overlay field did not create *W*, which falsifies claim 13 for that field; if *P*'s artwork does not show either, the run is inconclusive.

- G4 (order inside row 3). Named test file: a loose copy of the art file with these added sections, all of which copy the art section of an existing visible stock type and set their image field explicitly to its name: a new animation *AN* whose spawned-overlay field names a new overlay *W1*; two new buildings *Bb* and *Ba*, whose convert-to-overlay fields name new overlays *W2* and *W3*. Write the art section of *Ba* above that of *Bb*.
  - Setup: a test map (as in G1, but the scenario needs *N* + 2 ≤ 254). Its animation list names *AN*. Its building list names *Bb* first and *Ba* second, and the map gives each a rules section copying an existing structure's keys (no image field, so its art section is its own name). No overlay list. The map gives *W1*, *W2* and *W3* sections that look like *P*, *Q* and *R* (as in G1, G2). The overlay data places index *N* on one marked cell, *N* + 1 on a second and *N* + 2 on a third. *Bb* and *Ba* are chosen so that name order, file order of their art sections and index order disagree: *Bb* has the lower index, *Ba* sorts first by name and is first in the art file.
  - Precondition: *Bb* and *Ba* appear in the building tab with the icons their art sections give them, and *AN* is visible when played; otherwise the loose art copy did not take effect and the run is inconclusive.
  - Procedure: start a skirmish and look at the three cells. *Bb* and *Ba* need not be built: their overlays are created while they read their sections.
  - Observable: the artwork on the three cells.
  - Expected: *P*'s artwork at *N* (*W1*, created in the animations' pass), *Q*'s at *N* + 1 (*W2*, from *Bb*, the lower-indexed building) and *R*'s at *N* + 2 (*W3*, from *Ba*), all read in the overlays' pass of the map layer.
  - Falsified by: *P*'s artwork at *N* + 2 or *N* + 1 (the buildings' pass before the animations' pass); *R*'s artwork at *N* + 1 (creator order by name or by file order); or any other assignment. If one of the three artworks is missing, the run is inconclusive.

**H. Section names match exactly (claim 7).**
- Setup: a test map with a section for an existing buildable vehicle, spelled in a different letter case from the vehicle's name in the base rules' list, that only points its display name at another existing text. Choose spellings with distinct name hashes under S-0002 claim 3, so its hash-identity exception cannot hide the case distinction. Control map: the same section spelled exactly as in the list.
- Observable: the vehicle's display name in its sidebar tooltip, and its health when built.
- Expected: on the first map the stock display name and stock health; on the control map the new display name and stock health.
- Falsified by: the new display name on the first map, or any change other than the display name on the control map.

**I. Reference fields and weapon slots, sampled (claims 2, 6, 8 and 13).** Each sub-test uses one test map; new types appear in no type list unless stated, and every new section that needs artwork sets its image field explicitly to the stock type it copies.
- Share-source kind priority (claim 13): use S-0007 A22 part e's paired-kind setups, procedure and controls. Record the thrown debris model. Expected: infantry wins over vehicle, vehicle over aircraft, and aircraft over building, with the distinct model-versus-no-model outcomes that scenario specifies. Its contrary outcomes falsify the priority; no registration index need be exposed.
- I1 (find only: a prerequisite list). The map lists a new buildable building *T*, a copy of a stock structure, whose prerequisite list names only a new name *Q*. *Q* has a section copying another buildable structure, with its own display name. Control map: *T*'s prerequisite list names an existing structure the player has not built yet. Procedure: deploy the starting construction vehicle and look at the building tab. Expected with `quirks.prerequisite_names_dropped` on (default original): *T* is available at once on the first map, and *Q*'s display name never appears; on the control map *T* appears only after its prerequisite is built. Falsified by: *T* unavailable on the first map under that default setting, or *Q*'s display name appearing. With the switch off in our engine, the invalid entry remains unmet and *T* stays unavailable on the first map; *Q* is still not created. S-0407 claim 8 (d) and scenario T8 own this alternative.
- I2 (creating single field, read in the same layer). The map lists a new buildable building *F*, a copy of a stock structure, whose free-unit field names a new vehicle *V*. *V*'s section copies a stock vehicle with a different display name. Procedure: build *F*. Expected: a free unit appears with *V*'s display name (vehicles come after buildings, claim 6). Falsified by: no free unit, or one showing engine-default properties (no display name, no artwork).
- I3 (numbered weapon fields). Choose an existing vehicle whose turret-count field is 1 or more and whose weapon-count field is 1 or more. Precondition: on a stock map, that vehicle, alone and with no passengers or upgrades, visibly fires its first numbered weapon at a building; if it does not, pick another vehicle or count the run as inconclusive. The map changes that vehicle's first numbered weapon field to a new weapon *W* with a clearly visible damage, carried by no other type. Procedure: attack a target building of known health with that vehicle for a fixed number of shots. Expected: the health loss *W*'s section implies. Falsified by: the stock weapon's health loss while the precondition holds.
- I4 (`none` as a name in the attached particle system). Choose a stock weapon that has an attached particle system and a unit that fires it. The map sets that field to `none` and gives a section named exactly `none` that copies a different, clearly distinguishable stock particle system. Control runs: the same field set to `<none>`, set to an empty value, and set to a new name *S* with the same section as `none` under *S*. Expected: with `none` and with *S*, the shots show the other particle system's look; with `<none>` and with the empty value, they keep the stock look (the field keeps its value, claim 2). Falsified by: the stock look with `none` while the *S* run shows the other look, or the other look, or no particle system, with `<none>` or the empty value.
- I5 (the 19-byte limit). As I4, but the field names a new 22-byte name *L*. The map has two sections: one named with the first 19 bytes of *L*, copying a distinguishable stock particle system *A*, and one named with all of *L*, copying another, *B*. Expected: the shots show *A*'s look. Falsified by: *B*'s look.
- I6 (creating list field). The map lists a new buildable building *D*, a copy of a stock structure that emits damage smoke, whose damage particle-system list is `none,S` with a new name *S*; *S*'s section copies a clearly distinguishable stock particle system. Procedure: build *D* and damage it until it emits damage particles. Expected: *D* emits *S*'s look (the `none` item is skipped, *S* is created and read). Falsified by: no damage particles while a stock copy of the structure emits them, or the stock structure's own look.
- I7 (the 127-byte cut of a creating list field, claim 13). As I6, but *D*'s damage particle-system list is a run of commas followed by a new four-letter name *S* (empty items are skipped, S-0003 claim 12). The map has a section for *S* copying a distinguishable stock particle system *A*, and a section for the first two letters of *S*, called *S′*, copying another, *B*. Procedure and precondition as I6, in three runs whose comma runs differ in length:
  - Run 1: *S* starts at byte 128. Expected: no damage particles, since *S* lies past the cut.
  - Run 2: the first two letters of *S* are bytes 126 and 127. Expected: *B*'s look, since the cut leaves *S′*, which is created and read.
  - Run 3: *S* ends at byte 127. Expected: *A*'s look.
  - Falsified by: damage particles in run 1, or *A*'s look in run 2, when run 3 shows *A*'s look.
- I8 (the 127-byte cut of a single field, claim 13). As I2, but *F*'s free-unit field names an existing stock vehicle *E* whose type name has at most 24 bytes, followed by spaces and then one letter.
  - Run 1: the letter is byte 129, so the spaces run past byte 127. Expected: the free unit is a stock *E*, with *E*'s display name, because the cut removes the letter and the trailing spaces go with it.
  - Run 2: the letter is byte 127. Expected: no free unit with *E*'s display name, because the value names a new vehicle, which finds no section and keeps the engine defaults.
  - Falsified by: no *E* in run 1, or an *E* in run 2.
- I9 (the 63-byte cut of the emitted particle type, claims 2 and 13). Choose a stock particle system and effect as in A3, and a second, clearly distinguishable stock particle *P2* whose type name has at most 24 bytes. The map's section for that particle system sets its emitted particle type to *P2*'s name followed by spaces and one letter. Trigger the effect in each run.
  - Run 1: the letter is byte 65. Expected: *P2*'s look, as in run 3.
  - Run 2: the letter is byte 63. Expected: not *P2*'s look, because the value names a new particle type, which is created during the particle systems' pass and so reads no section in the map layer (claim 6).
  - Run 3, the control: *P2*'s name alone. Precondition: it shows *P2*'s look; otherwise pick another *P2*.
  - Falsified by: run 1 differing from run 3, or run 2 matching it.
- I10 (which values the weapon-slot conditions use, claim 13). A new weapon *W*, carried by no other type, whose section gives it a clearly visible, nonzero damage different from every stock weapon used here. Procedure as I3: attack a target building of known health for a fixed number of shots, and record the health loss.
  - Run a: choose an existing vehicle *V* whose turret-count field in the base rules is below 1 (absent or 0) and that has a primary weapon. Precondition: on a stock map, *V* visibly fires at a building. The test map's section for *V* sets, in this order in the file, its first numbered weapon field to *W*, its turret-count field to 1 and its weapon-count field to 1. It leaves the primary field alone. Expected: *W*'s health loss, because the section's own turret count decides and the numbered field fills the first slot. Falsified by: the stock primary weapon's health loss (that would mean the turret count held before the section decided). If *V* does not fire at all with the new turret count, the run is inconclusive.
  - Run b: the vehicle of I3, with a turret count of 1 or more. The test map's section for it sets its primary field to *W* and, below that line, its turret-count field to 0. Expected: *W*'s health loss, because the section's turret count of 0 makes the primary field take effect. Falsified by: the stock first numbered weapon's health loss.
  - Run c (the flag empties numbered weapon 1): the vehicle of I3, alone and with no passengers or upgrades as in I3's precondition. The test map's section for it sets only the clear-all-weapons flag. Expected: the vehicle does not fire at the building, and the building loses no health, because its first weapon slot, which holds numbered weapon 1, is empty. Falsified by: the stock first numbered weapon's health loss.
  - Run d (a held flag): named test file, a loose copy of the base rules in which the section of *V* (run a) also sets the clear-all-weapons flag. The test map's section for *V* sets only its primary field to *W*. Expected: *V* does not fire, because the flag held from the base rules keeps the primary field from taking effect and keeps its first weapon slot empty. Falsified by: *W*'s health loss, or the stock primary weapon's.

**J. Late readings: the second warhead reading and animation art (claim 6).**
- J1 (the second warhead reading). Choose an existing infantry type whose weapon leaves a radiation field, and a second infantry type that loses health when it stands in such a field in a stock game (precondition, checked on a stock map). Four test maps, each with the same small open area where one unit of the second type stands in a radiation field that a unit of the first type creates:
  - Map 1: the radiation section names a new warhead *NW* as the radiation-site warhead. *NW*'s section copies the stock radiation-site warhead's keys and sets its versus value against the second infantry type's armour to 0 %. The map has no special-weapon settings section.
  - Map 2: as map 1, plus a special-weapon settings section holding one key, with the value that key has in the base rules.
  - Map 3: as map 2, but *NW* has no section, so *NW* keeps the engine defaults even when it is read.
  - Map 4: as map 1, but a weapon carried by a unit on the map also names *NW*, so *NW* is read in the warheads' pass.
  - Observable: the health the standing unit loses in a fixed time at a fixed game speed.
  - Expected: on map 1 the unit loses health as on map 3, because *NW* is created by a late section and not read. On maps 2 and 4 it loses none, because *NW* reads its section; on map 2 that happens through the second reading.
  - Falsified by: map 2 losing health as map 3 does when map 4 loses none (no second reading), or map 1 losing none when map 3 loses health (the late-named warhead was read without the special-weapon settings section).
  - Inconclusive: map 4 losing health (the section did not bind), or map 3 losing none (the engine defaults hide the difference).
- J2 (an animation reads its art in the next layer). Named test file in every setup: a loose copy of the art file with one added section for a new animation *AN*. That section copies the art section of a clearly visible stock animation and sets its image field explicitly to that animation's type name. Choose a stock weapon that a unit fires visibly and that has an animation list.
  - Setup 1: a second named test file, a loose copy of the base rules in which that weapon's animation list is *AN* alone, and a stock map.
  - Setup 2: stock base rules, and a test map whose section for that weapon sets its animation list to *AN* alone.
  - Setup 3: as setup 2, and the test map's animation list also names *AN*.
  - Procedure: fire the weapon in a skirmish in each setup.
  - Observable: whether the weapon's firing shows the copied animation.
  - Expected: setup 1 shows it, because *AN* is created during the weapons' pass of the base rules and reads its art in the next layer. Setup 2 does not, because *AN* is created during the weapons' pass of the map layer, after the animations' pass of the last layer, and never reads its art. Setup 3 shows it, because the list creates *AN* before the passes.
  - Falsified by: setup 2 showing the animation, or setup 1 not showing it when setup 3 does. If setup 3 does not show it, the loose art copy did not take effect and the run is inconclusive.

**K. Reference-section selection and unsafe creation (claims 14 to 16; Q-0083).** All runs use a tiny purpose-made skirmish map, game speed 2, crates off, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), one human house and one bystander computer player kept outside the observed test area. Named test files below are removed after each run. Use screen capture and scripted normal input where useful. Unless stated, run each variant once and repeat any unexpected result in a fresh session.

- **K1, trailer file and section.** Named test file: a loose art copy. For an aircraft, and separately a visible, slowly travelling projectile used by a placed firer, give the map rules section an image name different from its type name. The selected art section, the type-name art section and the rules section each name a different, already listed and visibly distinguishable trailer animation. Give all selected trailers ample visibility through their art settings. Order the aircraft to fly across the observed area, or the firer to fire across it. Record the trailer pictures. Expected: the image-named art section's trailer in both cases, not the type-name or rules trailer. Swap the two art references for a control and expect the new image-section trailer. A rules trailer or type-name trailer falsifies claim 14. No trail in both test and control is inconclusive.
- **K2, building image and snow suffix.** Named test file: a loose art copy with art sections for a newly listed building's own name, an explicit image name and that image name with A appended. Each section names a different new overlay through its convert-to-overlay reference. Use the G3 overlay-index observable: overlays are unlisted, get rules sections copying three distinct decorative overlays, and the map places the first new overlay index on a marked cell. The building is first created by the map layer, sets its image explicitly and sets the arctic flag. Play temperate and snow variants. Expected: the explicit-image overlay on temperate, the suffixed-image overlay on snow, not the own-name overlay. Control: clear the flag on snow and expect the unsuffixed-image overlay. A different artwork falsifies. If no artwork binds in the control, the test is inconclusive. Repeat with no image field and matching own-name and own-name-plus-A art sections: these now supply the references. An additional later-layer image replacement on an already arctic building must follow S-0007 claim 10, not append another A.
- **K3, aircraft versus projectile suffix.** Named test file: a loose art copy whose image and image-plus-A sections name different listed trailer animations. First create each test type in the map layer, set a non-empty image and the arctic flag, and give it visible art. Fly the aircraft or fire the projectile across the observed area, once on snow and once on temperate. For the aircraft test, use sprite art. Expected: suffixed trailer on snow for the aircraft, unsuffixed trailer for the projectile on both theaters. Repeat with voxel art: both use the unsuffixed trailer. A suffixed projectile trailer or unsuffixed sprite-aircraft trailer on its first snow read falsifies. Art failure in the unsuffixed control makes the run inconclusive.
- **K4, section and image gates, held references and clearing.** Named test files: loose base-rules and art copies. Give an existing aircraft and projectile an explicit image and a clearly visible listed trailer in the corresponding art section at the base-rules read. In the map, replace the aircraft's image with another art section that has no trailer reference: expect the earlier trailer to remain. A second variant gives that selected section a reserved no-type name: expect no trailer. For the projectile, a map rules section that omits the image or gives it an empty value must retain its earlier trailer without creating a newly named trailer from any type-name art section. A non-empty new image whose art names another trailer must change the trail; an absent or empty trailer field there keeps it, and a reserved no-type name clears it. Record trail pictures during flight, with a no-override stock-image control. A changed trail without the required non-empty projectile image, removal on an absent art reference, a surviving trail after a reserved no-type value, or failure to change to the newly selected art section's non-empty trailer falsifies, provided the corresponding positive trail control works. Finally, introduce the new projectile or aircraft image only in the base rules, and a different trailer only in its art section: adding no rules section in the map must not cause another reference read. A building's held convert-to-overlay reference and its absent-rules-section gate are not tested by this trail scenario (claim 14, backing): changing an art reference cannot remove an overlay registration that an earlier layer already made.
- **K7, image-name byte limit.** Named test file: a loose art copy, written as raw bytes with no byte-order mark. Repeat K2's building-reference observable and K1's projectile-trailer observable with an image value consisting of 23 ASCII letters, bytes 195 and 169, then an ASCII letter. Provide three art sections: the first 24 bytes, the first 25 bytes, and the full 26 bytes of that value. Give them different decorative overlay references for the building, or different listed, visible trailer animations for the projectile. Use temperate theater with arctic flags clear. The two non-ASCII bytes would encode one character under UTF-8, but S-0001 gives no decoding rule. Procedure: observe the marked overlay cell at the first new overlay index, or fire the projectile. Controls: short ASCII aliases for each section with identical settings, confirming all three artworks or trails can bind and be distinguished. Expected: the 24-byte section supplies the reference, even though that byte string ends between the two non-ASCII bytes. The 25-byte section's effect falsifies the byte limit in favour of a 24-character decoded limit; the full-name effect or any other distinguished effect also falsifies. Missing artwork or trails with working controls falsifies section selection; with failed controls it is inconclusive. Repeat the projectile-trailer part with an aircraft; missing body art is acceptable only if the short-alias controls still show distinguishable trails. For a suffix-boundary test, repeat the building case on snow with its arctic flag set: appending A to an already 24-byte image name and cutting to 24 bytes still selects the same section. Supply an additional first-24-bytes-plus-A section with a fourth overlay for contrast. That fourth overlay falsifies the suffix cut; failed short-alias controls are inconclusive.
- **K5, unbounded long-name self-reference.** Named test file: a loose art copy with a new animation section whose name is exactly 24 ASCII bytes. Its next-animation reference names those same bytes followed by one additional letter. The map lists the 24-byte animation name. Control: the same file with the reference limited to the 24-byte name. Record whether loading reaches play and the elapsed loading time, once each in fresh sessions. Expected: the short-name control loads, whereas the long-name variant does not complete normally while creation continues. Stop the unsafe run externally after 60 seconds if needed; do not wait for resource exhaustion. Normal completion of the long-name variant without resource failure falsifies the unbounded-creation claim. A crash or allocation failure gives no guaranteed copy count. On our engine, both load; each independent same-layer chain in the long-name case creates one new copy and warns when its repeated reference is suppressed.
- **K6, nineteenth weapon registration.** Named test files: a loose base-rules copy and art copy. Take a vehicle with positive turret count, set its weapon count to 19, and make only its nineteenth normal weapon reference name a new weapon *W*, carried nowhere else. *W*'s section names a new animation *A* as its firing animation. *A*'s art section copies a stock animation's art, sets its image explicitly, and creates a new decorative overlay through its spawned-overlay reference. Leave *A* out of every animation list, and give the map that overlay's rules section and index placement as in G3. Choose a game mode with no extra overlay registrations. The observable is artwork at the first new overlay index: *A*, first created while *W* reads the base rules, reads art in a later layer and creates the overlay before the map's overlay reading. Control: weapon count 18, with the same nineteenth field present, must create no such overlay. Repeat with only the nineteenth elite field naming *W*. Expected: overlay in both count-19 runs and not in the count-18 control. A successful count-19 load without the overlay while a control naming *W* in the first slot shows it falsifies registration above 18. A failed original load is inconclusive, because excessive counts are unsafe. On our engine the overlay appears, but neither nineteenth reference supplies a usable firing slot, and the excessive count warns.

## Open questions

1. Where a side's member list (as opposed to each country's own side) is used in play is not covered here. It decides whether the quirk of claim 9 is observable.
2. Resolved by S-0501 (Import mapping, scenario AC30 part j): an aircraft type whose data leaves
   the counts-as-aircraft key out has the flag, so it ranks as air in claim 11 unless it also has
   the naval flag.
3. Which game option selects the mode in which no sidebar items are added (Edge cases) is not established. Confidence: low.
4. The relative order of types created by different keys of one section (claim 4, "within one section") is left unspecified, except for the crate rules (seen through overlay indices) and for list items and weapon slots (fixed, but visible only through claim 12 in other specs). If some other spec turns out to need more, a measurement through an index-visible effect would have to establish it first.
5. The order among types created after all rules layers (claim 3: map terrain-object placements, theater tile animations, the fixed particle system) is not established, nor whether the theater's tile animations are created again in every game.
6. **Resolved by S-0401 and S-0112, Terms.** Resource overlays select the first registered tiberium whose image run contains their overlay index, with a first-tiberium fallback. D4 now isolates that selection with a payout control; D5 measures the 23-byte cut through the selected tiberium's section.
7. Resolved by S-0411 claim 15: a suppressed type is absent from the offer, retries admission only at a later qualifying recheck, and gives no suppression-removal orders. F4 tests the capacity limit; S-0411 scenario S10 tests these consequences.
