---
id: S-0305
title: Armour classes (the class list, its order, and how a type's armour value is read)
topic: combat
revision: 1
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: []
supersedes: []
---

# S-0305: Armour classes (the class list, its order, and how a type's armour value is read)

## Summary

The original has a fixed list of eleven armour classes. Every object type belongs to exactly one,
chosen by its imported armour field, and a warhead scales its damage by the versus value it gives
that class (S-0302, S-0304). This spec owns the list, its order, how the armour field's text
becomes a class (including unknown names), the classes forced by the program, and the class's few
uses that do not go through versus values. How the warhead's list of versus values is parsed is
S-0003's (claim 13, Import mapping row C5); this spec agrees with it and does not restate it.

## Terms

Shared terms used here are defined in the glossary: *armour class*, *warhead*, *field*,
*absent-key default*, *importer warning*, *type*, *type kind*, *overlay*, *rules layer*,
*departure*. In addition:

- **Class index.** The position of an armour class in the list of claim 1, from 0 to 10. The
  *j*-th item of a warhead's versus list (S-0003 claim 13) belongs to the class with index *j* − 1.
- **Versus value.** The factor a warhead gives one armour class, a double-precision number read
  from the warhead's versus list (S-0003 claim 13). S-0302 multiplies by it and S-0304 gives the
  special meaning of small values. The glossary defines it in the same way.
- **Armour word.** The text that names a class in the data, in the form of the Import mapping.
- **Resource overlay.** An overlay type with the imported tiberium flag (ore, gems and the like).
  The economy specs own what it does.
- **Armed.** In claim 6, in the sense of S-0309, which owns the rule cited there (by the glossary,
  S-0214's sense: the object has a weapon in the weapon slot it currently uses).

## Behaviour

1. **The class list.** There are exactly eleven armour classes. In class-index order they are:
   *none* (0), *flak* (1), *plate* (2), *light* (3), *medium* (4), *heavy* (5), *wood* (6),
   *steel* (7), *concrete* (8), *special 1* (9) and *special 2* (10). The list and its order are
   fixed in the program. Data cannot add, remove, rename or reorder classes. The order is the one
   S-0003 (Import mapping row C5) gives for the items of a versus list, and the two specs agree.
   Our engine uses the same eleven classes in the same order.
   - Backing: `measurable` (scenario A1)
   - Confidence: high

2. **Every object type has one class.** Every type of the kinds whose instances are objects
   reads the armour field: infantry, vehicles, aircraft, buildings, overlays, terrain objects,
   smudges, projectiles, animations, voxel animations, particles and particle systems. Each reads
   it where it reads its other fields (S-0004 says which file that is for animations). A type that
   no rules layer has given the field has class *none*. Weapons, warheads, countries, sides,
   superweapons and tiberiums have no armour class. The class is a property of the type, so all
   objects of one type share it, and nothing in play changes it. Only the classes of infantry,
   vehicles, aircraft, buildings, overlays and terrain objects are known to matter in play
   (claim 6 and S-0302).
   - Backing: `measurable` (scenario A2 for a vehicle type with no armour field)
   - Confidence: high for infantry, vehicles, aircraft and buildings and for the absent-key
     default *none*; medium for the other kinds (they share the same reading of the field, but no
     use of their class was traced) and for no game rule changing a class in play (none was
     found)

3. **How the armour word is matched.** The field's value (trimmed as S-0001 says) is compared
   with the eleven armour words of the Import mapping as a whole, ignoring the letter case of
   A to Z only, so `Heavy`, `HEAVY` and `heavy` all give class 5. Only an exact match of the whole
   value counts: no prefix, abbreviation, number or surrounding text is accepted, so `heavy2`,
   `5` and `heavy armour` match nothing.
   - Backing: `measurable` (scenario A3)
   - Confidence: high

4. **An unknown armour word gives class *none*.** When the field is given with a value that
   matches none of the eleven words, the type's class becomes *none* (class 0). It does not keep
   the class in force from a lower rules layer. When the field is absent from a section, or given
   with an empty value (the glossary's *absent-key default* case), the class in force stays: the
   absent-key default *none* for a type that no layer has given a class, otherwise the class an
   earlier layer set (S-0006). Our engine reproduces both. The importer warns for every value that
   matches no word, naming the section, the key and the value, and saying that the type is given
   class *none*.
   - Backing: `measurable` (scenario A4)
   - Confidence: high for the unknown word; medium for the empty value (it follows the common
     text-reading rule of S-0003, under which an empty value counts as absent)

5. **Resource overlays always have class *wood*.** An overlay type with the tiberium flag has
   class *wood* whatever its armour field says: whenever its section is read (S-0006), the result
   is class *wood*, whatever the field says, so the field has no effect on such a type. Other overlay types take their class from
   the field like any other type. The importer warns when a resource overlay's section gives an
   armour word other than *wood*, saying that it is ignored.
   - Backing: `unmeasured` (no rule of the original reads a resource overlay's class: the area
     damage that reaches resources reduces them by a rule of its own that ignores armour classes,
     and the class of an overlay is read only for walls, claim 6, which are not resource overlays.
     The forced class is therefore invisible in play; it is stated so that an engine that exposes
     classes, and the importer warning, agree with the original)
   - Confidence: medium (that the forced class wins over the field is clear)

6. **Where the class is used.** The class selects one versus value of a warhead whenever damage is
   resolved against an object (S-0302) and whenever a versus value decides a targeting question
   (S-0304, and the weapon choice of S-0308). The class *wood* has two further fixed uses that do
   not go through versus values:
   - a wall overlay of class *wood* can be attacked and damaged by a warhead with the imported
     wood flag (S-0214, the attackable-cell rule; the area-damage spec S-0303 owns the damage), and
   - a terrain object of class *wood*, such as a tree, can be set burning, which takes a draw from
     the scenario random stream (the terrain specs own the burning).

   The classes *medium* (4) and *wood* (6) also have a fixed use in the *warhead's* own list: a
   warhead is *infantry-only* exactly when its versus values for class 4 and for class 6 are both
   exactly 0 (an exact comparison of the double-precision values, so a value that is merely small
   does not count; a missing list gives eleven items of 100 %, which is not 0). An armed infantry
   unit whose primary weapon's warhead is infantry-only searches for infantry targets only (S-0309,
   claim 5). The mark is a property of the warhead, fixed by its versus list as read from the
   data; it does not depend on the target's class in play.

   No other class has a fixed use: apart from *wood* and the two positions just named, the classes
   differ only through the versus values that data gives them, and their names carry no meaning to
   the program.
   - Backing: `measurable` (scenario A6 for the wall case; scenario A7 for the infantry-only mark)
   - Confidence: high (wood walls; the infantry-only mark, a single rule that S-0309 confirms
     with the same rule); medium (burning terrain; that no other class has a
     fixed use, a summary of the uses found)

## Formulas

> class(*v*) = *j*  if *f*(*v*) = *f*(*w*ⱼ) for some *j* from 0 to 10,
> class(*v*) = 0  otherwise.
>
> *v* is the armour field's value, trimmed as S-0001 says, for a field that is given with a
> non-empty value; an absent or empty value leaves the class in force (claim 4), and a resource
> overlay has class 6 whatever *v* is (claim 5). *w*₀ … *w*₁₀ are the armour words in the order of
> the Import mapping. *f* folds the letters A to Z to one case and leaves every other byte as it
> is. "=" is equality of the whole texts. Exact text comparison, so no rounding applies. Domain and
> unit: class index, an integer from 0 to 10.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Armour classes | 11 | classes | Claim 1; the same number as the items of a versus list (S-0003, Constants) |
| Class of a type without the field | 0 (*none*) | class index | Claim 2, absent-key default |
| Class of an unmatched word | 0 (*none*) | class index | Claim 4 |
| Class of a resource overlay | 6 (*wood*) | class index | Claim 5 |

## State and ordering

| State | Event | Result |
|---|---|---|
| Type with class *c* in force | A rules layer gives the armour field a matching word | Class of that word |
| Type with class *c* in force | A rules layer gives a word that matches nothing | Class *none*; importer warning |
| Type with class *c* in force | A rules layer contains the section but not the field, or gives it empty | Class *c* stays |
| Resource overlay type | Its section is read | Class *wood*, whatever the field says |

For a resource overlay the last row governs: rows 1 to 3 do not change its class (claim 5).

Observable ordering: none beyond the rules-layer order of S-0006.

## Edge cases and quirks

- **Unknown word resets to *none*** (claim 4). This is the original's behaviour and is kept; the
  importer warning tells a mod author. A layered mod that misspells the armour word in a map's
  override section silently strips the type's armour class. No switch is proposed, since a
  "fixed" behaviour (keep the class in force) would change which versus value applies and the
  importer warning already makes the case visible.
- **Resource overlays** (claim 5): reproduced, with the importer warning.
- The original's text comparison may follow the host's letter-case rules for bytes above 127 under
  some system settings. **Departure** (host-dependent, no switch): our engine folds A to Z only.
  No armour word contains such a byte, so this matters only for a value that matches nothing
  either way.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, any object type's section (infantry, vehicle, aircraft, building, overlay, terrain object, smudge and the other object kinds), key `Armor` | the type's armour class (claims 2 to 4) | one of the words `none`, `flak`, `plate`, `light`, `medium`, `heavy`, `wood`, `steel`, `concrete`, `special_1`, `special_2`, in any letter case, giving class indices 0 to 10 in that order; any other value gives 0 with an importer warning | the class in force; *none* (0) for a type no layer has given one |
| Rules, overlay type section, key `Tiberium` | makes the overlay a resource overlay, which forces class *wood* (claim 5) | boolean (S-0003) | no |
| Rules, warhead section, key `Verses` | the eleven versus values in class-index order | S-0003, claim 13 and row C5 | S-0003: eleven items of 100 % |
| Rules, warhead section, key `Wood` | the wood flag of claim 6 | boolean (S-0003) | no (S-0214) |

## Measurement scenarios

All scenarios use the measurement setup with named test files: a test map whose override sections
define the test types. Damage is read from the health bar or a recorder of health values, at any
game speed, with one human house and one computer house far away.

- **A1 (claim 1, class order).** A test warhead whose versus list gives the eleven classes the
  distinct values *k*/16 for *k* = 1 to 11 in list order, written as decimals without a percent
  sign (0.0625, 0.125, …, 0.6875, all exact in binary, so no rounding enters), on a test weapon of
  damage 160 that fires
  once at each of eleven test vehicles, one of each of eleven test vehicle types that are alike
  except that each type's armour field gives a different armour word of the Import mapping (the
  class belongs to the type, claim 2, so one type cannot supply all eleven), at full health and with enough health to survive. *Observable:* health lost by
  each vehicle. *Expected:* the vehicle with the *k*-th word of the Import mapping list loses
  10 *k*, for *k* = 1 to 11 (S-0302 gives the exact damage; no other multiplier applies to these
  houses and vehicles). *Falsified if* any two vehicles' losses swap or a loss belongs to another
  item of the list.
- **A2 (claim 2).** As A1, with a twelfth vehicle type whose section gives no armour field and whose
  stock section gives none either (a new test vehicle type). *Expected:* it loses 10 (class
  *none*, first item). *Falsified if* it loses any other amount.
- **A3 (claim 3).** As A1, with four further test vehicle types whose armour fields are written
  `Heavy`, `HEAVY`, `heavy2` and `5`, one vehicle of each. *Expected:* the first two lose 60 (class 5); the last two lose 10 (class *none*, by
  claim 4). *Falsified if* `heavy2` or `5` lose 60 or anything other than 10.
- **A4 (claim 4).** A stock vehicle type whose stock class is not *none*. Test map 1 gives its
  section the armour field with the value `nosuchclass`; test map 2 gives its section another key
  only; test map 3 gives its section the armour field with an empty value. The A1 warhead fires once at it in each map. *Expected:* map 1: the loss of
  class *none* (10); maps 2 and 3: the loss of its stock class. *Falsified if* map 1 shows the stock
  class's loss (the class in force kept) or maps 2 or 3 show class *none*.
- **A6 (claim 6, wood walls).** A test map with an enemy wall overlay type given class *wood*, and
  a unit whose warhead has the wood flag and no wall flag. *Expected:* the attack cursor appears
  on the wall and the wall is damaged; with the wall type given class *steel* instead, no attack
  cursor appears without force-fire. *Falsified if* the class makes no difference.
- **A7 (claim 6, the infantry-only mark).** Two test infantry types, each armed with a test weapon
  whose projectile can hit ground targets, in a test map with one enemy infantry unit and one enemy
  vehicle, both of class *light*, stationary and within guard range, and the vehicle placed nearer.
  The first type's warhead gives versus value 0 for classes 4 and 6 and 100 % for the rest; the
  second type's warhead is the same but gives class 4 the value 0.0001. *Observable:* which target
  each unit picks on its own (no order), over several placements with the vehicle nearer.
  *Expected:* the first unit always picks the infantry unit; the second picks the vehicle in at
  least some placements. *Falsified if* the first unit ever picks the vehicle, or the second never
  picks it.

## Open questions

- Whether any game rule changes the class of an object in play (claim 2). None was found.
- Whether the infantry-only mark (claim 6) has any use besides the infantry search of S-0309
  claim 5. None was found; S-0309 owns the search.
- Which other object kinds can take damage at all (smudges, animations) is not this spec's
  question: they have a class, but the damage specs say whether anything reaches them.
- The burning of wood terrain objects (claim 6) belongs to a future terrain spec, which should
  state its draws.
