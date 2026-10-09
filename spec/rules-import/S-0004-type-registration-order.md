---
id: S-0004
title: Type registration order
topic: rules-import
revision: 3
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-09
measurements: []
requests: []
supersedes: []
---

# S-0004: Type registration order

## Summary

How the game builds its registries of object types from the rules data: which types exist, the index each one gets, what happens to types that are referenced but never listed, and how the order of the type lists does (and does not) affect what the player sees, including the order of the build sidebar. How several rules files are stacked on top of each other is in S-0006.

## Terms

The glossary defines *type kind*, *registry*, *registration index*, *type name*, *type list*, *game-wide section*, *base rules*, *rules layer*, *game*, *game start*, *departure* and *measurement setup*. This spec adds:

- **Layer.** In this spec, short for *rules layer*.
- **Creation by reference.** A type is created the first time a field names it, when no type of that kind and name exists yet. The field may belong to another type or to a game-wide section.
- **Reading pass.** The part of processing one layer in which each existing type of a kind reads its own section from that layer (claim 6). The second reading that warheads get in a layer with the special-weapon settings section (claim 6) is not a reading pass.
- **Early and late game-wide sections.** The two groups of game-wide sections that a layer reads before and after its reading passes (claim 4). Import mapping names them.
- **Sidebar tab.** One of the four columns of build icons (cameos) on the sidebar.
- **Engine defaults.** The values a type's fields have when the type is created, before it reads any section.
- **Display name.** The text a type shows the player, for example in its sidebar tooltip. It is resolved from the type's display-name field.

## Behaviour

Registration indices are non-negative integers. Indices are dense: the *n*-th type created for a kind has index *n* − 1, and indices are never reused within a game.

1. **Registries are rebuilt at every game start.** At each game start, every type registry is emptied except tiberiums (claim 10). Sides are emptied too, with one difference in multiplayer and skirmish games: when processing of the base rules begins, the side registry already holds the sides that the base rules' countries' side fields name, in country-list order (claim 9). The registries are then refilled by processing the layers in order (S-0006). Two games started from the same data therefore get identical registration indices. The exception is tiberiums: a tiberium that an earlier game's map or game-mode file added keeps its index in later games of the same session (claim 10).

   The original also builds a provisional set of buildings and animations at program start. It is thrown away at the first game start and has no effect on any game, so an implementation need not build it.
   - Backing: `measurable` for the rebuilt registries (scenario G, second game) and for tiberiums being kept (S-0006, scenario G). `measurable` for sides being emptied (scenario E, E5).
   - Confidence: high (medium for sides)

2. **A type list registers names in entry order.** The entries of a type list take effect in the order they appear in the file. Each entry's value is a type name, and its effect depends on that name:

   | Entry value | Effect |
   |---|---|
   | empty | none |
   | `none` or `<none>`, in any letter case | none; no type is created |
   | the name of an existing type of the kind, compared without regard to letter case | none; the existing type keeps its index |
   | any other name | a new type with that name is created and gets the next index of its kind |

   Wherever a type name is looked up with creation allowed, `none` and `<none>` likewise mean "no type". There is one exception: where a particle system names the particle type it emits, only `<none>` means no type. There, `none` names a particle type called none, and an empty value names a particle type with an empty name. Either is created if it doesn't exist (see S-0006, claim 5).

   The entry's key (the text left of `=`) plays no part for ordinary lists. Only its position counts. The exceptions are sides (claim 9) and tiberiums (claim 10). Which of two entries with the same key survives is defined in S-0002, and case rules for section and key names in S-0001.
   - Backing: `measurable` for the existing-name and reserved-name rows (scenario A). The empty row is `unmeasured`: a skipped empty entry and a type with an empty name look the same in play, since no section can carry an empty name. The particle exception: the empty-name case is measurable through S-0006 scenario D3. The `none` case is `measurable` (scenario A, A3). The particle type it creates reads no section in the layer that creates it (claim 6: particles come before particle systems), but it does read its section in every later layer (claims 6 and 7), so a particle system that names it in the base rules shows the particle section a test map gives it.
   - Confidence: high

3. **Some kinds have no list at all, and some only a list.** Weapons and projectiles have no type list. They come into existence only by reference, for example when a vehicle names a weapon or a weapon names a projectile. Every other kind in the glossary's *type kind* list has a type list.
   - Superweapons and tiberiums are created only by their own lists. A rules field that names an unknown superweapon (for example a building's superweapon field) means "none"; it does not create one. Likewise, an unknown name in a field that lists countries (for example a type's owner list) is ignored and creates no country.
   - Countries are created by the country list. Outside the rules layers, a mission's own house definitions that name an unknown country also create it. That happens after all layers, so such a country reads no rules section and keeps the engine defaults.
   - Sides are created by the side list or by a country's own side field (claim 9).
   - All other kinds can also gain types by reference.
   - Backing: `measurable` for weapons and projectiles by reference and for the unknown superweapon (scenario B). `unmeasured` for the unknown country in a field and for countries created by mission house definitions: a country created that way could not be picked by any player and reads no section, so no skirmish observable separates "ignored" from "created and unused", and the mission case needs a purpose-made campaign mission whose observable belongs to the country spec.
   - Confidence: high (medium for countries created by mission house definitions)

4. **Listed types come before referenced types of the same layer.** Within a layer, the index a new type gets depends only on what first names it. Within each kind, every type in an earlier row of this table has a lower index than every type in a later row:

   | Order | Types | Read their section in the same layer? |
   |---|---|---|
   | 1 | types the layer lists (every type list except the tiberium list) | yes |
   | 2 | types first named by the early game-wide sections: general settings, AI settings, AI intelligence levels, power-ups, land types, jumpjet controls and multiplayer dialog settings | yes |
   | 3 | types first created during the reading passes (claim 6) | only if created no later than their own kind's pass, except warheads in a layer that has the special-weapon settings section (claim 6) |
   | 4 | types first named by the late game-wide sections: difficulty levels, crate rules, combat damage, radiation, elevation and wall models, audio-visual settings and special weapons | no, except warheads in a layer that has the special-weapon settings section (claim 6) |

   Where a section sits in the file plays no part. Only the row that first names a type counts. Within rows 2 to 4 the order is partly fixed:
   - Row 2: types first named by the AI settings come before types first named by the general settings. Of the early sections, only these two name types.
   - Row 3: types first created during an earlier pass (in the rank order of claim 6) come before types first created during a later pass. Within one pass, types first created while a lower-indexed type reads its section come before types first created while a higher-indexed type reads its section.
   - Row 4: the late sections take effect in the order the table lists them (difficulty levels, crate rules, combat damage, radiation, elevation and wall models, audio-visual settings, special weapons), and a type first named by an earlier one of them comes first.
   - The order among several types first created by the same section (one type's section, or one game-wide section) is not specified here (open question 4).
   - Types first created while warheads read their section a second time (claim 6) belong to none of the four rows. Their place in the order is not specified here (open question 5).

   Sides follow claim 9 instead of this table. The tiberium list takes effect after all four rows, at the end of the layer (claim 10).

   Across layers, types first named by an earlier layer have lower indices than types first named by a later one. The order is the rules-layer order (S-0006): the base rules, then the language rules file if a valid one exists, then the game-mode file, then the map. A later layer can add types at the end of a registry. It can never remove a type, rename it or move it.

   The index of overlay types is visible, because a map's overlay data refers to overlay types by registration index (S-0009, claim 12). Overlays are created only by their list and by the crate rules, so they can show orders 1 and 4 and the cross-layer append.
   - Backing: `measurable` for orders 1 and 4 and for the cross-layer rule (scenario G). `unmeasured` for the position of orders 2 and 3: no kind whose index a game shows is created by the early game-wide sections or during the reading passes, so those two rows, and the order within rows 2 to 4, can be seen only through claim 12 in other specs.
   - Confidence: high (medium for the order within rows 2 to 4)

5. **Names are limited to 24 bytes.** Name lengths in this spec are counted in bytes, since rules documents are byte strings with no character encoding (S-0001, claim 1). A name written in plain ASCII has one byte per letter. A type stores at most the first 24 bytes of its name. A type reads its properties from the section whose name equals its stored name (claim 7). So a type listed under a longer name picks up only a section named with its first 24 bytes.

   Before a type-list value is looked up, it is cut to its first 31 bytes, and spaces at the end of the cut text are removed. This shows only in one case: a value made of an existing type's name of at most 24 bytes, followed by spaces that run past the 31st byte and then more text, finds that existing type. A value whose extra text starts at or before the 31st byte does not (claim 8).
   - Backing: `measurable` (scenario C, C1 and C3)
   - Confidence: medium

6. **The rank of a type's kind decides whether a referenced type gets its properties.** Each kind has a fixed rank, and within each layer the reading passes of the kinds take place in rank order:

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

   Animations read from the art file in every layer. An animation first created after the animations' pass of a layer, that is, during the pass of a later-ranked kind or by the late game-wide sections, does not read its art in that layer, but it does in the next layer. The map layer is always the last layer and always follows the base rules, so only an animation first created after the animations' pass of the map layer stays without its art.

   Proposed switch: `quirks.unlisted_reference_unread` (default **on** = the YR behaviour above). With it off, a type created by reference still reads its section in the layer that created it, wherever its kind falls in the order. Stock data is written around this behaviour, so turning it off should not change stock games, but mods may rely on either form.
   - Backing: `measurable` (scenario B)
   - Confidence: high for the pass order and the unread consequence. Medium for the exact list of kinds each game-wide section can create.

7. **A type with no section of its own is left untouched.** If a layer has no section named after a type, that type reads nothing in that layer. Its values stay as they were, and nothing is reset. Section lookup uses the stored name, which may be truncated (claim 5). The stored spelling is the one the type was first registered with. Type-name lookup ignores letter case (claim 2), but section names are matched exactly (S-0001). So a section spelled in a different case from the first registration is not found.
   - Backing: `measurable` (scenario H)
   - Confidence: high

8. **Long names duplicate types.** Lookups compare the name being looked up against the stored, truncated name. The looked-up name is first cut to the length the field allows (31 bytes for a type-list value, claim 5; longer for many other fields). So a name longer than 24 bytes never matches the type created from it, and every later lookup of that name that may create a type creates yet another type of the kind. Each copy has the same truncated name and gets a new index. A lookup that may not create a type finds nothing.

   Proposed switch: `quirks.long_name_duplicates` (default **on**). With it off, names are truncated to 24 bytes before every comparison, so all references resolve to one type.
   - Backing: `measurable` (scenario C, C2)
   - Confidence: medium

9. **Sides come from the side list and from each country's side field.** Every country records one side as its own (none until something sets it), and every side keeps a member list of countries. Side names are compared without regard to letter case.
   - **Side list.** Each entry's *key* is the side's name and its *value* is a list of countries. The side is created if no side of that name exists. Its member list is replaced by the listed countries, and each listed country records that side as its own.
   - **Country side field.** When a country reads its section (claim 6), a side field that is present names the country's side. An unknown side name creates a new side. If the side differs from the country's current side, the country is moved to the new side's member list (see the quirk below). An absent field keeps the current side.
   - **Order within a layer.** Within one layer a country's side field wins over the side list: the result is as if the layer's side list took effect before the countries' side fields of the same layer. A later layer's side list wins over an earlier layer's side field. A country that a later side list leaves out keeps the side it had: leaving a country out of a side's list does not take the side away from it.
   - **Names in the side list's value.** Each name is looked up as a country, matching either the country's type name or its name field, without regard to case. A name that is not a country but is the name of an existing side stands for all of that side's current members. Any other name is skipped. The countries must already exist, from the country list of the same layer or an earlier layer.
   - **Indices.** A side's index is the order in which sides are first created. In a campaign mission the base rules' side list comes first, then sides that only countries' side fields name. In a multiplayer or skirmish game, the side indices instead follow the order in which the base rules' countries' side fields first name each side, taken in country-list order, and the side list only adds sides no country names. This is because in those games, when processing of the base rules begins, the side registry already holds the sides that the base rules' countries' side fields name, in country-list order (claim 1). With data in which both orders agree, as in stock data, the result is the same.

   Sides are cleared at every game start, like the other registries (for multiplayer and skirmish games, see claim 1).

   Proposed switch: `quirks.side_change_removes_by_position` (default **on**). When a country's side field moves it away from a side, the member removed from the old side's member list is the one whose position in that list equals the country's registration index. If the list has no such position, no member is removed. So a different country may be removed, and the moved country may stay in the old list. With the switch off, the moved country itself is removed from the old list. Where the member list is used is outside this spec.
   - Backing: `measurable` (scenario E) for the side list and the country side field. `measurable` for the difference between the two index orders (scenario E, E4, with a loose copy of the base rules in which the two orders disagree). `unmeasured` for the quirk: whether it can be observed depends on where member lists are used (open question 1).
   - Confidence: medium. Low for the quirk, whose observable depends on how member lists are used.

10. **Tiberiums are matched by key number, not by name.** In the tiberium list, each entry's key is read as an integer *k* by the rule in Formulas (a leading decimal number; a key without one gives 0). The value is cut to 23 bytes. Let *n* be the number of tiberiums that exist when the entry takes effect. The entry's effect is:

    | Entry | Effect |
    |---|---|
    | empty value, any key | none; the entry is skipped |
    | non-empty value, 0 ≤ *k* < *n* | the tiberium with index *k* is used; the name in the value is ignored |
    | non-empty value, *k* ≥ *n* | a new tiberium, named by the value, is created at index *n*, the next free index. That index may differ from *k*: if two tiberiums exist and the entry's key is 5, the new tiberium gets index 2 |
    | non-empty value, *k* < 0 | see below |

    Each tiberium named by an entry reads its section immediately, at the end of processing the layer. Unlike every other kind, a tiberium reads its section in a layer only if that layer's tiberium list has an entry for it: a layer that contains a tiberium's section but no list entry for it leaves the tiberium unchanged. A negative key refers outside the registry; the result is undefined in the original (likely a crash). This is a departure: our engine ignores such an entry, and the importer warns. Tiberiums are never removed, not even between games (S-0006, claim 10).

    Proposed switch: `quirks.tiberium_index_by_key` (default **on**). With it off, tiberiums are matched by name like other kinds.
    - Backing: `measurable` (scenario D)
    - Confidence: medium (high for the key parse)

11. **The sidebar order does not depend on registration index.** Each buildable item goes to a fixed sidebar tab:

    | Item | Tab |
    |---|---|
    | buildings whose build category is "combat" (defences), and superweapons | 2nd |
    | other buildings | 1st |
    | infantry | 3rd |
    | vehicles and aircraft | 4th |

    A tab holds at most 76 items. An item that becomes available while its tab is full is not shown. An item already shown (the same type) is not added twice. Two different types are two items even if their names are equal.

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
    4. Tech level (an integer), lowest first.
    5. Cost to the local player at the time the item is added, in whole credits, lowest first. This is the price the player would pay, country cost multipliers included; how it is computed belongs to the production spec.
    6. Display name, compared as in Formulas. When the names are equal too, the newly added item goes first. So among items identical in every key above, the one that became available last is shown first.

    The order is fixed when an item is added. Items are not re-sorted later, for example when a cost changes.

    Proposed switches: `quirks.sidebar_tie_newest_first` (default **on**; with it off, a full tie places the new item after the existing equal items) and `quirks.sidebar_both_flags_rank` (default **on** = the two-column rank above; with it off, an item with both flags ranks as air, 1, in both columns).
    - Backing: `measurable` (scenario F)
    - Confidence: medium

    These sidebar rules are user-interface behaviour. They stay in this spec for now; a future UI spec may take them over, and this claim will then point there.

12. **"The first type that…" means the lowest registration index.** Wherever another spec says that the game picks "the first type of a kind that satisfies a condition", first means the lowest registration index under claims 2 to 4. With unchanged data, that is the order of the base rules' type lists, followed by types created by reference.
    - Backing: `unmeasured`. This is a definition. Each spec that uses it states its own observable.
    - Confidence: high

## Formulas

**Key of a tiberium-list entry (claim 10).** The key *s* (already trimmed, S-0001) gives the integer *k*. Split *s* as *s* = *w* *p* *D* *r*, where *w* is the whole run of whitespace at the start of *s* (possibly empty), *p* is one sign byte `+` or `-` or nothing, *D* = *d*₁ *d*₂ … *d*<sub>ℓ</sub> is the longest run of decimal digit bytes `0` to `9` that follows *p* (ℓ ≥ 0), and *r* is the rest of *s*, which plays no part.

> *k* = wrap₃₂(σ × *d*), with *d* = Σ<sub>*i*=1…ℓ</sub> *d*<sub>*i*</sub> × 10<sup>ℓ−*i*</sup> (so *d* = 0 when ℓ = 0), σ = −1 if *p* is `-` and σ = +1 otherwise.
> - wrap₃₂(*x*) is the integer in [−2³¹, 2³¹ − 1] that is congruent to *x* modulo 2³², so a key too large for 32 bits gives the wrapped value.
> - Exact integer arithmetic, then the wrap; no rounding applies. Number domain: signed 32-bit integers. Unit: none (a registration index).
> - No other form is recognised: `12abc` gives 12, `+7` gives 7, `-1` gives −1, and a key written in hexadecimal gives only its leading decimal digits (so a leading `0` followed by a letter gives 0).

This differs from the integer value forms of S-0003. Keys are never read in those forms.

**Superweapon recharge time as a sort key (claim 11, rule 1).** The recharge-time field *m* is read as a decimal number in game minutes and held as a double-precision floating-point number.

> *t* = trunc(RZ(*m* × 900)). Unit: ticks; *t* is an integer.
> - *m* × 900 is a floating-point product. RZ rounds the exact product to double precision, toward zero (the original's round-toward-zero mode).
> - trunc truncates toward zero to an integer.
> - A field value of exactly 0 keeps the earlier value (S-0006).

The round-toward-zero mode is owned by S-0205 (number domains, planned), together with its switch. So when the exact product of the stored double and 900 lies just below a whole number, the product rounds down and *t* is one tick less than round-to-nearest arithmetic gives. For example, a field of 0.3 is stored as a double slightly below 0.3; the product rounds toward zero to just under 270, so *t* = 269, where round-to-nearest gives exactly 270 and *t* = 270.

Two superweapons are compared by *t*, an integer, so two fields that truncate to the same tick count tie and fall through to the display name.

**Display-name order (claim 11, rule 6).** Display names are compared as sequences of unsigned 16-bit character codes, from the first character on. The first differing code decides: the lower code sorts first. If one name is a prefix of the other, the shorter one sorts first. Equal sequences are a tie.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| stored name length | 24 | bytes | longer names are truncated (claims 5 and 8) |
| type-list value read length | 31 | bytes | cut before trailing spaces are removed and before lookup (claim 5) |
| tiberium list value length | 23 | bytes | claim 10 |
| reserved "no type" names | `none`, `<none>` | type names | any letter case |
| sidebar tabs | 4 | tabs | claim 11 |
| items per sidebar tab | 76 | items | further items are not shown |
| recharge-time factor | 900 | ticks per game minute | claim 11, Formulas |

## State and ordering

| State | Event | Result |
|---|---|---|
| any | game start | all registries except tiberiums are emptied, then filled layer by layer (for sides in multiplayer and skirmish games, see claim 1) |
| type list entry names an unknown type | layer registration | new type appended, next index |
| type list entry names a known type (any case) | layer registration | nothing; index unchanged |
| a field names an unknown type during the reading passes | reading | new type appended; it reads its section in this layer only if its kind's pass has not yet run, or is running now, or it is a warhead and the layer has the special-weapon settings section (claim 6) |
| layer has no section for a type | reading | type unchanged |
| item becomes buildable | sidebar update | inserted in sorted position in its tab, unless the tab is full or the item is already there |

Observable ordering:
- Within a layer, every listed type of a kind has a lower index than every type of that kind first created by reference in the same layer, wherever the sections sit in the file. Sides (claim 9) and tiberiums (claim 10) follow their own claims instead.
- Later layers only append.

## Edge cases and quirks

- `quirks.unlisted_reference_unread`: claim 6.
- `quirks.long_name_duplicates`: claim 8.
- `quirks.tiberium_index_by_key`: claim 10.
- `quirks.sidebar_tie_newest_first` and `quirks.sidebar_both_flags_rank`: claim 11.
- `quirks.side_change_removes_by_position`: claim 9.
- `quirks.particle_none_is_a_name` (default **on**): where a particle system names its emitted particle type, `none` names a particle type called none (claim 2). With it off, `none` means "no particle type" there as everywhere else. The empty-value case belongs to S-0006, claim 5.
- Tiberiums kept between games: switch `quirks.persist_tiberium_overrides` in S-0006, claim 10.
- The superweapon recharge-time sort key depends on the *round-toward-zero mode* (Formulas). Its switch, `quirks.chop_rounding`, belongs to S-0205 (number domains, planned); this spec defines none.
- A side list naming a country that is not registered: the name is skipped (claim 9), unless it is a side's name.
- The reserved country name for a random choice, written in a side list, refers outside the country registry in the original. The result is undefined. Departure: our engine skips the name, and the importer warns.
- A negative key in the tiberium list: undefined in the original. Departure: our engine ignores the entry and the importer warns (claim 10).
- In one particular game-wide mode, the original adds no items to the sidebar at all. Which game option selects that mode, and whether it is reachable in normal play, is not established (open question 3).

## Import mapping

| Rules section | Kind | Entry form |
|---|---|---|
| `[Countries]` | countries | any key = country name |
| `[Sides]` | sides | side name = comma-separated country names (claim 9) |
| `[SuperWeaponTypes]` | superweapons | any key = name |
| `[BuildingTypes]` | buildings | any key = name |
| `[VehicleTypes]` | vehicles | any key = name |
| `[AircraftTypes]` | aircraft | any key = name |
| `[InfantryTypes]` | infantry | any key = name |
| `[Warheads]` | warheads | any key = name |
| `[Animations]` | animations | any key = name |
| `[VoxelAnims]` | voxel animations | any key = name |
| `[Particles]` | particles | any key = name |
| `[ParticleSystems]` | particle systems | any key = name |
| `[TerrainTypes]` | terrain objects | any key = name |
| `[SmudgeTypes]` | smudges | any key = name |
| `[OverlayTypes]` | overlays | any key = name |
| `[Tiberiums]` | tiberiums | integer key = name (claim 10) |
| none | weapons, projectiles | reference only |

Within one layer, every list above except `[Tiberiums]` is registered before the reading passes. `[Tiberiums]` takes effect at the end of the layer, after the reading passes (claim 10). Every kind has its own registry, so the order among the other lists has no visible effect, with one exception: `[Countries]` is registered before `[Sides]`, so a side list can name countries listed in the same layer (claim 9).

Game-wide sections (claim 4):
- Early: `[General]`, `[AI]`, `[IQ]`, `[JumpjetControls]`, `[MultiplayerDialogSettings]`, the power-up section and the twelve land-type sections (S-0006). Of these, only `[General]` and `[AI]` name types in the original.
- Late: `[Easy]`, `[Normal]`, `[Difficult]`, `[CrateRules]`, the combat-damage section, `[Radiation]`, `[ElevationModel]`, `[WallModel]`, `[AudioVisual]` and `[SpecialWeapons]`. `[SpecialWeapons]` is the special-weapon settings section whose presence in a layer gives every warhead its second reading (claim 6).
- Animations read their sections from the art file, not from the layer (claim 6, rank 3; S-0006, claim 8).

Country keys (claim 9): side field = `Side` in a country's section (absent = keep); name field = `Name`, which side lists may use in place of the country's type name. Country lists in other fields, such as `Owner`, ignore unknown names (claim 3).

Reference fields used in the scenarios: a building's superweapon field = `SuperWeapon` (find only, claim 3); a vehicle's deploy-into building = `DeploysInto`; a projectile's airburst and shrapnel weapons = `AirburstWeapon`, `ShrapnelWeapon`; a particle system's emitted particle type = `HoldsWhat` (claim 2); the crate overlays = `CrateImg`, `WoodCrateImg`, `WaterCrateImg` in `[CrateRules]` (overlay names, created by reference in the late phase); an object type's artwork = `Image`; a tiberium's credit value = `Value`.

Sidebar keys (claim 11): build category = `BuildCat` (combat = `Combat`); base-planning side = `AIBasePlanningSide`; counts as aircraft = `ConsideredAircraft`; naval = `Naval`; tech level = `TechLevel`; display name = `UIName` (resolved to text); superweapon recharge time = `RechargeTime` (game minutes).

## Measurement scenarios

Every scenario runs under the measurement setup (glossary): an unmodified install with no engine add-ons, and no loose override files except the named test files a sub-test lists, which are removed after that sub-test. Where a scenario needs extra rules entries, they go in the override sections of a purpose-made test map in the multiplayer map folder, never in the stock files. Sub-tests A3, E4 and E5 also use named test files: a loose copy of the base rules, or loose side archives (S-0005). Run one skirmish per scenario at any game speed, with crates off, unless stated. Test names use plain ASCII letters, so each letter or space is one byte.

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
- D2 setup: a test map that changes the first ore type's credit value in that ore's own section, with no tiberium list in the map. Expected: the gain is unchanged, because an unlisted tiberium does not read the map. Falsified by the changed gain.
- D3 setup: as D1, but the key is written `0abc`. Expected: as D1 (the key reads as 0). Falsified by: a gain matching the new section's value.

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
- E4 procedure: (i) a skirmish on a stock map, the local player playing an Allied country; (ii) the first Allied campaign mission. Check first that the mission's map has no side list and no country side fields of its own; if it has either, use another Allied mission without them. Each run in a fresh session.
- E4 observable: the look of the sidebar's radar frame, or whether the game reaches play.
- E4 expected: (i) the Allied look. In a skirmish the Allied side keeps index 0, because the indices follow the countries' side fields. (ii) The Soviet look. In a campaign mission the indices follow the side list, so the Allied side has index 1, which mounts the Soviet side's group (S-0005, claim 13a).
- E4 falsified by: the Soviet look in (i), or the Allied look in (ii). If a run does not reach play, it is inconclusive: a swapped side list may break a mission for other reasons.
- E5 (sides are emptied at every game start, claim 1). Let *S* be the number of sides a fresh skirmish has: the distinct side names that the base rules' side list and the base rules' countries' side fields name (count them in the file). A new side created by a map then gets index *S* if the side registry was emptied at game start, and *S* + 1 if a new side from the previous game is still there.
- E5 named test files: a loose side archive for index *S* (its archive number from S-0005, claim 13a) that is a copy of the stock Allied side's archive, and one for index *S* + 1 that is a copy of the stock Soviet side's archive.
- E5 setup: test map 1 sets the side field of a country nobody plays to a new side name *T1*. Test map 2 sets the side field of the country the local player plays to a different new side name *T2*.
- E5 procedure: in one session, play a skirmish on map 1 until it reaches play, return to the menu, then play a skirmish on map 2 as the country map 2 changes. Control: map 2 alone, in a fresh session.
- E5 observable: the look of the sidebar's radar frame on map 2.
- E5 expected: the Allied look in both runs, because the registry was emptied and *T2* got index *S*.
- E5 falsified by: the Soviet look after map 1, which would mean *T1* was kept and *T2* got index *S* + 1. If the control does not show the Allied look, or does not reach play, the run is inconclusive.

**F. Sidebar order (claim 11).**
- F1 setup: a skirmish with stock data. Screenshot each tab once all tech is available, for example after a long game.
- F1 observable: the order of the icons.
- F1 expected: the order follows the rules of claim 11 (superweapons first, then side, category, tech level, cost, name).
- F1 falsified by: any adjacent pair whose order contradicts those rules, given the items' fields and the order in which they became available.
- F2 setup, for ties: a test map defining two vehicles identical in tech level, cost and display name, unlocked by different prerequisite buildings.
- F2 procedure: build the two prerequisites in either order, in two runs.
- F2 expected: the vehicle unlocked last appears first.
- F2 falsified by: list order deciding the order, or the earlier-unlocked vehicle appearing first.
- F3 setup, for the recharge-time rounding (Formulas): a test map whose superweapon list adds two new superweapons, each a copy of an existing one, granted by two new buildable buildings. Superweapon *A* has a recharge time of 0.3 minutes and superweapon *B* one of 0.30001 minutes, and their display names are chosen so that *B* sorts first by display name.
- F3 procedure: build both buildings and look at the superweapon icons once both are shown, in two runs that build them in either order.
- F3 expected: *A* before *B* in both runs, because *A* has 269 ticks and *B* 270.
- F3 falsified by: *B* before *A*, which would mean the two tie at 270 ticks and the display name decided (round-to-nearest).

**G. Overlay indices: listed before late references, later layers append (claims 1 and 4).**
- Setup: a test map with format version 2 or higher (S-0009, claim 11) in a theater where two existing overlay types have clearly different visible artwork, *P* and *Q*, neither of them a crate type. Let *N* be the number of entries in the base rules' overlay list (count them in the file). Check that the chosen game mode's file adds no overlay types; the scenario needs *N* + 1 ≤ 254.
  - The map's crate-rules section names a new overlay name *Y* as a crate overlay. Put that section *above* the overlay list in the file. *Y* gets no section.
  - The map's overlay list names two new overlay types, *X1* then *X2*, with sections that copy the keys of *P*'s and *Q*'s sections respectively and also set the image field explicitly to *P*'s and *Q*'s type names, so that *X1* looks like *P* and *X2* like *Q*. Choose *P* and *Q* among simple decorative overlays, not walls, crates or resources.
  - The map's overlay data places overlay index *N* on one marked cell, *N* + 1 on a second marked cell, and *N* − 1 on a third.
- Second map (claim 1): a second test map, same theater and format, whose overlay list names one new overlay type *Z* set up to look like *Q* (as above), with overlay index *N* on one marked cell. It has no crate-rules section.
- Procedure: start a skirmish on the first map and look at the three cells. Then return to the menu and, in the same session, start a skirmish on the second map and look at its cell.
- Observable: the artwork on each marked cell.
- Expected: on the first map, cell *N* shows *P*'s artwork (*X1*), cell *N* + 1 shows *Q*'s artwork (*X2*), and cell *N* − 1 shows the last overlay of the base list. On the second map, cell *N* shows *Q*'s artwork (*Z*), because the registry was rebuilt and *Z* is the first type past the base list.
- Falsified by: on the first map, *P*'s artwork at cell *N* + 1, or nothing at cell *N* + 1 while *P*'s artwork is at cell *N*. The first would mean *Y* took an index before both listed types, the second that it took one between them. *Q*'s artwork at cell *N* also falsifies: it would mean *X2* took the first new index, against entry order (claim 2). On the second map, *P*'s artwork at cell *N* (the first map's *X1* kept) falsifies claim 1. If *P*'s and *Q*'s artwork appear on no cell of the first map, the run is inconclusive (the artwork did not bind), not a falsification.

**H. Section names match exactly (claim 7).**
- Setup: a test map with a section for an existing buildable vehicle, spelled in a different letter case from the vehicle's name in the base rules' list, that only points its display name at another existing text. Control map: the same section spelled exactly as in the list.
- Observable: the vehicle's display name in its sidebar tooltip, and its health when built.
- Expected: on the first map the stock display name and stock health; on the control map the new display name and stock health.
- Falsified by: the new display name on the first map, or any change other than the display name on the control map.

## Open questions

1. Where a side's member list (as opposed to each country's own side) is used in play is not covered here. It decides whether the quirk of claim 9 is observable.
2. The default of the counts-as-aircraft flag for aircraft types decides whether aircraft sort as air or ground in claim 11. It belongs in the aircraft spec.
3. Which game option selects the mode in which no sidebar items are added (Edge cases) is not established. Confidence: low.
4. The order among several types first created by the same section is not stated (claim 4): several types of one kind first named by one game-wide section, or by one type's section during a reading pass. Claim 12's "first type" depends on it for any type created by reference. The answer has to be an observable order (for example the order of the fields in the import mapping) or an explicit statement that the order is unspecified, with our engine's deterministic choice; it must not be written as the order in which the original reads its fields.
5. In a layer that has the special-weapon settings section, every warhead reads its section a second time, after the late game-wide sections (claim 6). Two things about types first created during that second reading are not stated: where they fall in index order (claim 4 places them in none of its rows), and whether they read their own section in that layer. Claim 12's "first type" depends on the first for any such type.
