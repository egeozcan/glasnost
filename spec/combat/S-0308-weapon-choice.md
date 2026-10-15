---
id: S-0308
title: Weapon choice (primary or secondary per target, elite weapons, multi-weapon types, deployed, passenger and garrison weapons)
topic: combat
revision: 8
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: [Q-0083]
supersedes: []
---

# S-0308: Weapon choice

## Summary

Every infantry unit, vehicle, aircraft and building that can attack holds its weapons in numbered
weapon slots. Against each target the original picks one slot, the *weapon it would use*, by a fixed
precedence of rules: the slot a multi-weapon type has switched to, the deployed or passenger weapon,
special warhead kinds, zero versus values, water, land and air targets. This spec gives the slots,
the elite replacement, the precedence, the rules for gattling, gunner and garrisoned objects, and the
mean weapon damage that other rules use. Whether the chosen weapon may fire is the firing check of
S-0301; its range is S-0307's.

## Terms

Glossary terms are used as defined there, in particular *object*, *type*, *field*, *weapon*,
*warhead*, *armour class*, *infantry*, *vehicle*, *aircraft*, *building*, *house*, *allied* (as
S-0214 defines it), *behaviour*, *cell*, *land type*, *bridge*, *departure*, *importer warning* and
*quirk switch*. *Versus value* and *weapon it would use* are S-0304's terms; this spec owns how the
weapon it would use is chosen. *Health ratio* and *counts as a vehicle* are
S-0214's, and *veterancy rank* is S-0311's (S-0214 only uses the words). *Deploy kinds* and *current sequence* are S-0107's. *Unload* and the other behaviour names
are S-0215's. *Garrisoned building* is S-0301's and *open-topped transport* is S-0507's, as the glossary
gives them; S-0203 claim 3 owns open-topped passengers' place in the update order. *Elite weapon* is S-0311's (claim 6), with the slots here. *Disc laser weapon* is
S-0301's term (Terms and claim 10).

- **Weapon slot.** A type holds up to 18 weapon slots, numbered 0 to 17. Each slot holds a weapon or
  nothing. Slot 0 is the *primary* slot and slot 1 the *secondary* slot; "the primary weapon" and
  "the secondary weapon" mean the weapons in those slots. A type also holds 18 *elite slots* of the
  same numbering.
- **Weapon in slot *i* of an object.** The weapon an object fires when it uses slot *i*, after the
  elite replacement of claim 2 and, for a building, the upgrade and occupant rules of claims 12 and
  13.
- **Firer.** The object whose weapon is chosen. **Target**: the object or cell it is chosen against.
- **Multi-weapon type.** A type whose imported turret count is above 0 (Import mapping). It is a
  *gattling type* when its gattling flag is set, and a *charge-turret type* when its charge-turret
  flag is set. Every multi-weapon type that is not a gattling type is a *switching type*,
  charge-turret types included (their weapon number never changes, claim 5).
- **Weapon number.** An integer per object, the slot a switching type currently uses (claim 5).
- **Gattling stage.** An integer per object of a gattling type, the stage its weapon has spun up to.
  S-0609 claims 1–4 define how it rises and falls; claims 1–9 own its ordinary updates and their
  timing. This spec only reads it.
- **Gunner type.** A vehicle type with the imported gunner flag: its weapon number is set by its
  passenger (claim 6).
- **Mode index.** A type's imported passenger mode index: the weapon number a gunner type takes when
  an object of this type becomes its first passenger, or is put back after a failed exit
  (claim 6).
- **Airborne.** An object is airborne when the original counts it as in the air. For aircraft,
  use S-0501 claim 18's *in-the-air answer*, as defined in its Terms, not its distinct *in flight*
  test. The movement specs own the exact test for other objects; S-0303 (claim 10) and S-0309 use
  the same test.
- **Water target.** A target object that is not airborne, not on a bridge deck, and whose cell has
  the land type *water* or *beach* (S-0211).
- **Kamikaze aircraft.** An aircraft in the dive by which it crashes itself into its target. The
  aircraft specs own when that state begins.
- **Overpowered building.** A building in the state defined by S-0518 claim 13. An
  *overpowerable* building type can be put in that state.
- **Installed upgrade.** A building type that has been installed on a building through one of the
  building's upgrade slots (S-0108 gives the slots). The structures specs own installing an upgrade
  and removing it; this spec only reads the building's list of installed upgrades, in the order they
  were installed.
- **Cloaking, cloaked, uncloaking.** The states an object passes through when it hides itself
  (*cloaking*, then *cloaked*) and comes into view again (*uncloaking*); an object in none of the
  three is fully visible. The cloaking specs own the transitions. S-0301 (claim 12) uses the same
  three words for "any state other than fully visible".
- **Mean weapon damage.** Claim 15.

## Behaviour

This spec covers skirmish combat and import. Network games and campaigns are out of scope.

### Weapon slots and elite weapons

1. **Slots come from the type.** Which weapons a type holds in its usable slots is read from its
   type section (Import mapping). A type whose turret count is above 0 has numbered normal and
   elite weapon fields 1 through *N* take effect, where *N* is its weapon count: each present
   field may create its named weapon type, and each resulting weapon type reads its section as
   specified by S-0004, claim 6. In our engine only fields numbered 1 through 18 fill usable slots
   0 through 17 and their elite slots; fields numbered above 18 have registration effects but no
   usable slot (claim 16). A missing numbered field creates no weapon type. A type whose turret
   count is 0 or less fills slot 0 from its primary key and slot 1 from its secondary key, and
   the elite slots 0 and 1 from the two elite keys. Neither form reads the other's keys. When a rules layer gives a type's
   section and the type's clear-weapons flag is set as that section leaves it (a layer that omits
   the key keeps the value in force), its slots 0 and 1 and its elite slots 0 and 1 are empty after
   that layer, even when the same layer names weapons for them. This holds whatever its turret
   count, so on a multi-weapon type it empties the first two numbered weapons
   (`quirks.clear_weapons_any_turret_count`, Edge cases). With turret count 0 or less and the flag
   set, the four standard weapon keys create no weapon type in that layer (S-0004, claim 13), so a
   weapon named only there does not exist unless something else creates it.
   - Backing: `measurable` (scenario W1, parts c to f; W11 and S-0004, claim 16, scenario K6,
     for registration above 18). The absence of registration from suppressed standard fields
     is `unmeasured` here: firing shows an empty slot, not whether an otherwise unused weapon
     type was registered (S-0004 claim 13 has the same qualification).
   - Confidence: high for the two forms and the registration/usable-slot distinction; medium for
     the clear-weapons flag on multi-weapon types

2. **Elite replacement.** S-0311 (claim 6) owns when an elite weapon is used; this claim restates the
   rule for the slots, and the two agree. While an object's veterancy rank is elite, the weapon in its slot *i* is
   the weapon of its type's elite slot *i* when that elite slot holds a weapon, and the weapon of
   slot *i* otherwise. The replacement is per slot: an elite primary does not imply an elite
   secondary. At lower ranks the elite slots are never used.
   - Backing: `measurable` (scenario W1, parts a and b)
   - Confidence: high

### Choosing the slot against a target

3. **The precedence.** The weapon it would use against a target is the slot given by the first row
   of table P (State and ordering) that applies to the firer and the target. Overrides for deployed
   infantry and vehicles (claims 8 and 9) come before the table. No row draws from a random stream,
   and the choice depends only on the firer, the target and their present state, so asking twice with
   unchanged state gives the same answer, even within one tick. A shot or state change between
   requests can change the answer. The rows say which of the two slots 0 and 1 is used, except rows
   1, 4 and 5, which can give other slots.
   - Backing: `measurable` (scenarios W2 to W8 test the rows: rows 6 to 8 by W2, rows 9 and 11 by
     W6 parts a and b, row 10 by W6 part e, rows 13 and 17 to 20 by W3, row 3 by W7 part b); row 12
     is `unmeasured`: it needs an aircraft in its dive, which no scenario known to us can set up
     with two distinguishable weapons, and which the aircraft specs would have to supply. Rows 14 to
     16 are tested by S-0304 scenario V2 and W3. Precedence combinations not exercised by these
     parts are `unmeasured`: the parts test individual rows and selected overlaps, not every
     possible combination of special abilities. The absence of choice-related random draws is
     `measurable` through W12. Slot 0 with no target is `unmeasured`: without a target these
     parts have no shot identifying a slot. Choices hidden by an independent refusal to fire
     (notably a cloaked water target) are also `unmeasured` unless a shot identifies them.
   - Confidence: high for the order of the rows; medium for rows 6 to 11, which depend on special
     abilities that other specs own; low for row 12

4. **Incomplete weapon pairs use slot 0.** A firer whose primary or secondary slot is empty,
   or whose secondary weapon has the never-use flag, uses slot 0 against every target, apart from claims 8 and 9 and
   rows 1 and 2 of table P. In particular rows 4 (passengers) and 5 (gattling types) apply only to firers with
   weapons in both slots 0 and 1 and a secondary without the never-use flag
   (`quirks.passenger_weapon_needs_secondary`, Edge cases).
   - Backing: `measurable` (scenario W7, parts b to d)
   - Confidence: high

### Multi-weapon types

5. **Switching types use their weapon number.** A switching type uses the slot of its weapon number
   against every target, whatever the target is: no versus, land, water or air rule applies to it.
   An object's weapon number starts at 0, and it is always one of 0 … 17: setting it to a value
   outside 0 … 17 sets it to 0. A charge-turret type ignores every request to change its weapon
   number.
   - Backing: `measurable` (scenario W4, parts a to f for the number; part h for the charge-turret
     rule; W1 part d for a switching type selecting a higher slot with slots 0 and 1 empty)
   - Confidence: high

6. **Gunner types.** For a vehicle of a gunner type:
   - when an object enters it as a passenger and is then its only passenger, its weapon number
     becomes the mode index of that passenger's type,
   - when its last passenger leaves, its weapon number becomes 0,
   - further passengers entering or successfully leaving while at least one remains change
     nothing.

   **Failed-exit exception.** When a passenger fails to exit during unloading and stays inside
   at position 1 (S-0507 claim 10), the gunner hand-over runs again for that passenger, even
   with others aboard. Its weapon number becomes that passenger's mode index, subject to
   claim 5's clamping and charge-turret exception
   (`quirks.unload_failed_exit_gunner_handover`).

   For a gunner vehicle that is also a switching type but is not a charge-turret type, the weapon
   is therefore chosen by the passenger that entered the empty vehicle, and it stays while any
   passenger is aboard, even after that passenger has left, unless a failed exit changes it as
   above. A charge-turret gunner ignores the
   weapon-number change request and remains at weapon number 0. Other gunner vehicles still follow
   table P for slot choice.
   **Temporal hand-over.** A first passenger with a temporal beam (S-0514, Terms) hands
   that beam to the vehicle whether or not it is currently erasing a target. The vehicle
   carries on any attached erasure without releasing the target or resetting its progress.
   When the last passenger leaves, the vehicle hands its temporal beam back to that passenger
   and the beam lets go. If it was the sole attacker, the target stops being erased and loses
   its progress; otherwise S-0514 claims 9 and 10 govern continuation and release.
   If the entering passenger's rearm countdown (S-0301) was still running, the vehicle's rearm
   countdown restarts with the delay of the weapon in the vehicle's slot 7 (low confidence).
   The vehicle's weapon number after entry is still the passenger type's mode index, as above.
   - Backing: `measurable` (scenario W4, parts a to f for the first-passenger rule; part g for the
     temporal hand-over, idle-beam transfer and last-passenger release; part i and S-0507 scenario T10 part e for the failed-exit weapon-number
     exception). Retention after the first passenger alone leaves is `unmeasured`
     if W4 part d cannot remove that passenger alone: evacuating and refilling does not test it.
     Other hand-over effects after a failed exit are `unmeasured`: these scenarios isolate the
     weapon number, not temporal erasure or rearm-countdown transfer.
   - Confidence: high for the first-passenger rule and failed-exit weapon-number exception,
     high for temporal beam transfer and release on last-passenger exit, low for the
     rearm-countdown transfer and temporal effects after a failed exit

7. **Gattling types.** A gattling type at gattling stage *s* uses slot 2*s* + 1 against an airborne
   target object when the weapon in its slot 1 has an anti-air projectile, and slot 2*s* otherwise.
   The anti-air test looks at slot 1, not at slot 2*s* + 1 (`quirks.gattling_air_test_slot_one`).
   A gattling type with an empty slot 1, or a never-use weapon there, uses slot 0 at every stage
   (claim 4).
   - Backing: `measurable` (scenario W5, including its third variant for the empty slot 1)
   - Confidence: high

### Deployed and passenger weapons

8. **Deployed infantry.** For an infantry type with the deploy-fire flag, table P does not apply.
   The slot it uses depends only on its own state:

   | Current sequence is a deploy kind | Rides inside an open-topped transport, and its type's passenger weapon index is not −1 | Slot |
   |---|---|---|
   | yes | any | its type's deploy weapon index |
   | no | yes | its type's passenger weapon index |
   | no | no | 0 |

   So an infantry unit of such a type never chooses its secondary by the target-based rows of table
   P.
   - Backing: `measurable` (scenario W8, parts a, b and d)
   - Confidence: high

9. **Deployed vehicles.** A vehicle that is deployed and whose type has the deploy-fire flag uses
   its type's deploy weapon index against every target; otherwise table P applies. The index is used
   as given: −1 does not mean "choose normally" here (Edge cases).
   - Backing: `measurable` (scenario W8, part c)
   - Confidence: medium (which vehicles reach the deployed state belongs to the deploy specs)

10. **Passengers of open-topped transports.** A firer riding inside an open-topped transport, whose
    own type's passenger weapon index is not −1, uses that index (row 4 of table P), subject to
    claim 4. The transport's type plays no part. −1 means "choose normally" (rows 5 onwards).
    - Backing: `measurable` (scenario W7, including part e for the transport's own index)
    - Confidence: high

### Water and land targets

11. **Water targets follow the firer's naval targeting.** Against a water target, after the rows
    above it in table P, the slot follows the firer's naval-targeting mode by table N. A result of
    "none" is slot 0 here, the same result as any other slot-0 answer: the choice returns no special
    "no weapon" value. This spec gives no rule that then forbids the shot; a refusal, if the original
    has one for these modes, would come from a rule outside the weapon choice (open question). The modes' tests look at the target's type: its underwater flag, organic flag,
    unnatural flag, its hover speed type, and, for mode 0, its present cloak state.
    - Backing: `measurable` when a shot identifies the slot (scenario W3; scenario W6, parts c
      and d); `unmeasured` for a choice hidden by a refusal to fire: the refusal alone cannot
      distinguish slot 0 from a separate prohibition outside this spec.
    - Confidence: high for modes 1, 2, 3, 5, 6 and 7; medium for modes 0 and 4

### Buildings

12. **Upgrades supply weapons.** The weapon in slot *i* of a building is the weapon in slot *i* of
    the first of its installed upgrades, in the order they were installed, whose type has a weapon
    there. It is always the upgrade type's ordinary slot *i*: an upgrade's elite slots are never
    used, whatever the building's veterancy rank. Only when no installed upgrade has one do the
    occupant rule (claim 13) and then the building's own slot *i* (with claim 2) apply.
    - Backing: `measurable` for slot 0 (scenario W9, parts c and d); `unmeasured` for higher
      slots: these parts cannot select a higher building slot through ordinary input.
    - Confidence: medium

13. **Garrisoned buildings fire their occupants' weapons.** A building whose type can be occupied
    and lets its occupants fire, and that holds at least one occupant, uses slot 0 when row 2 of
    table P applies; a switching type instead follows row 1. The weapon in that slot, and in every other slot that a rule asks about (so the building's mean
    weapon damage of claim 15 is that weapon's *d*), unless an upgrade supplies one (claim 12), comes
    from its *current occupant*: the occupant at the building's occupant index. The occupant index starts at
    0 and moves to the next occupant, in the order the building holds them, after each successful
    non-disc-laser shot (wrapping to 0 after the last). A disc-laser shot does not advance it,
    so successive such shots use the same occupant. It returns to 0 when the building is emptied
    by occupants leaving or being killed (S-0313 claim 6); this is not a rule for an individual
    departure or death while other occupants remain. A refused firing attempt or void shot does
    not advance it (S-0301 claim 15).
    The weapon is:
    - for an elite occupant: its type's elite occupying weapon if the type has one, and otherwise
      the weapon in the occupant's own slot 0 (with claim 2),
    - for any other occupant: its type's occupying weapon if the type has one, and otherwise the
      weapon in the occupant's own slot 0.

    An elite occupant whose type has an occupying weapon but no elite occupying weapon therefore
    fires its own primary, not the occupying weapon (`quirks.elite_occupant_skips_occupy_weapon`).
    S-0313 (claim 6) owns this shared index, called the muzzle index there.
    - Backing: `measurable` for slot 0 (scenario W9, parts a, b and e; S-0313 P5 part b for the disc-laser
      exception; W9 part f for the reset after evacuation). Reset after occupants are killed and
      unchanged index on a refused or void attempt are `unmeasured` here: these parts do not
      isolate killing occupants from other building changes, or an unsuccessful launch between
      distinguishable occupants. Index effects of an individual departure or death while others
      remain are not established here. Occupant substitution in
      other slots is `unmeasured`: these garrison parts choose only slot 0 for their shots.
    - Confidence: high for the rotation and the weapon rule; medium for the order of occupants

### Other uses of the weapons

14. **The weapon it would use is the one tested everywhere.** The firing check (S-0301), the
    versus bands of S-0304, the threat scoring and candidate tests of S-0309 and the attack rule of
    S-0214 all look at the weapon this spec chooses against the object concerned, asked afresh each
    time.
    - Backing: `measurable` (for the firing check, scenario W8 parts a and d, where the chosen
      slot's firing legality determines whether a shot is refused or made; for the versus choice,
      S-0304 scenario V2, where the non-zero-versus weapon is selected and the zero-versus weapon
      is not; for acquisition candidate and scoring uses, S-0309 scenarios A2 and A4, where the
      stated weapon-dependent candidate/score result determines which target is selected, and A8,
      where expected-health preference determines the selected target; for the attack rule,
      S-0214 scenario T4, where the target's legality for the weapon it would use determines the
      attack result)
    - Confidence: high

15. **Mean weapon damage.** Some rules (S-0309: who heals and who retaliates) use an object's *mean
    weapon damage* *M*, which does not depend on a target:
    - for a switching type, *M* is *d* of the slot of its weapon number (0 when that slot is
      empty),
    - for every other object, *M* is the mean of *d*₀ and *d*₁ over the slots 0 and 1 that hold a
      weapon (Formulas F1), and 0 when neither holds one,

    where *d*ᵢ is the damage field plus the ambient-damage field of the weapon in slot *i* (with
    claim 2). An object whose *M* is below 0 is a *healer for acquisition* (S-0309). This is not the
    test of S-0214's *healer*, which looks at the weapon it would use by default: the two can
    differ, for example for a type that switches to a healing weapon only through its weapon number.
    - Backing: `measurable` (scenario W10)
    - Confidence: high

16. **Registration is not limited by usable slots.** With a positive turret count, normal and elite
    numbered weapon references take effect for every positive integer number up to the imported
    weapon count, including numbers above 18. They resolve or create weapon types by S-0004
    claims 13 and 16. Newly created weapons read their sections in that same rules layer by
    S-0004 claim 6, so their own references can create further types even when the weapon has no
    safe usable slot. The normal reference precedes the elite reference at each number, and
    lower numbers precede higher numbers for registration. Missing references create nothing.
    - The original has no registration cutoff at 18, but an excessive count makes its overall
      result undefined. This does not promise that such a game completes loading or can fire.
    - **Departure for bad data.** Our engine provides only slots 0 … 17 and their elite slots,
      keeps the registration and section-reading effects of higher-numbered references, and warns
      about the excessive count. No usable firing slot is supplied for those higher references.
    - Backing: `measurable` for registration and section-reading effects (scenario W11, shared
      with S-0004 K6). Registration order is `unmeasured`: these parts expose existence through
      overlay artwork, not a weapon index or the relative order of several created weapons.
      The safe slot limit and warning are design rules, not predicted oracle results.
    - Confidence: high for registration without a cutoff; medium for observing it in an original
      game that successfully loads the unsafe test data.

## Formulas

**F1. Mean weapon damage.**

> *d*ᵢ = damage(slot *i*) + ambient damage(slot *i*)  (integers, sum in signed 32-bit)
>
> *M* = trunc((*d*₀ + *d*₁) / 2) when both slots hold a weapon, *M* = *d*ᵢ when only slot *i* does,
> and *M* = 0 when neither does

trunc is truncation toward zero (S-0205, claim 2), so a sum of −3 gives *M* = −1. Integer
arithmetic, no rounding mode involved.

**F2. Gattling slots.** *slot* = 2 × *s* + *a*, where *s* is the integer gattling stage and *a* is an integer equal to 0 or 1: *a* = 1 when the target is an airborne object and the weapon in slot 1 has an anti-air projectile, and *a* = 0 otherwise. The result is an integer slot; evaluation uses integer arithmetic and no rounding. The supported integer width and overflow behaviour of this expression are not established here and must not be inferred from this formula.

**F3. Versus test.** "Zero versus value" means *v* = 0 exactly (+0 or −0), the double-precision
versus value of the weapon's warhead for the target's armour class (S-0304, Formulas). No other
threshold enters the weapon choice.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Weapon slots per type | 18 | slots | Numbered 0 to 17; also 18 elite slots |
| Primary slot | 0 | slot | |
| Secondary slot | 1 | slot | |
| Gunner hand-over slot for a temporal weapon | 7 | slot | Claim 6, low confidence |
| Passenger weapon index absent-key default | −1 | slot index | −1 = choose normally |
| Deploy weapon index absent-key default | 1 | slot index | Claims 8 and 9 |
| Mode index absent-key default | 0 | slot index | Claim 6 |
| Land-targeting mode absent-key default | 0 | mode | Table P rows 13 and 18 test mode 2 |
| Water role range | 14 | tile indices | S-0010, table R (row 13 of table P) |
| Naval-targeting mode absent-key default | 0 | mode | Table N |
| Occupant index start | 0 | occupant | Claim 13 |

## State and ordering

**Table P: the weapon it would use** (first row that applies; claims 8 and 9 come before it).
"Target object" means the target is an infantry unit, vehicle, aircraft or building; "the
secondary" and "the primary" are the weapons in slots 1 and 0.

| Row | Firer and target | Slot |
|---|---|---|
| 1 | the firer is of a switching type (including a gunner type only when it is also a switching type) | its weapon number (claim 5) |
| 2 | the firer is a building that can be occupied, lets occupants fire, and holds occupants | 0 (claim 13) |
| 3 | slot 1 or slot 0 is empty, or the secondary has the never-use flag, or there is no target | 0 |
| 4 | the firer rides in an open-topped transport and its passenger weapon index is not −1 | that index (claim 10) |
| 5 | the firer is of a gattling type | claim 7 |
| 6 | the secondary's warhead is an airstrike warhead | 1 if the target is a building whose type accepts charges and is not both a resource gatherer and a resource destination; otherwise 0 |
| 7 | the primary's warhead is a locomotor warhead and the target is a building | 1 |
| 8 | the secondary is a draining weapon, the target's type is drainable, the firer is draining nothing at present, and the target is not allied with the firer | 1 |
| 9 | the secondary has the area-fire flag and the firer's current behaviour is *unload* | 1 |
| 10 | the firer is an overpowered building (S-0518 claim 13) | 1 |
| 11 | the target is a building allied with the firer, the secondary's warhead is an electric-charge warhead, and the target's type is overpowerable | 1 |
| 12 | the firer is a kamikaze aircraft | 1 |
| 13 | the target is a cell, the firer's land-targeting mode is 2, and either the cell's land type is not water and its tile index lies outside the range of the theater's water role (S-0010, table R: 14 tile indices from the role's start), or the cell holds a bridge and the firer's type is naval | 1 |
| 14 | the target is not a target object (a cell, a terrain object, an overlay) | 0 |
| 15 | the secondary's versus value against the target is zero | 0 |
| 16 | the primary's versus value against the target is zero | 1 |
| 17 | the target is a water target | table N |
| 18 | the target is not airborne and the firer's land-targeting mode is 2 | 1 |
| 19 | the target is airborne and the secondary has an anti-air projectile | 1 |
| 20 | otherwise | 0 |

The first row that applies decides, so the order of two rows is observable wherever both can apply
and they give different slots. Rows that give the same slot may be tested in any order. The
orderings that change outcomes include:
- row 1 before every other table row (a switching type ignores target-based choice rules; in
  particular a switching type with an empty secondary still uses its weapon number, claim 5),
- row 2 before row 3 (a garrisoned building without weapons of its own still fires),
- row 3 before rows 4 to 20 (claim 4),
- row 4 before row 5 (a gattling passenger with a passenger weapon index uses that index),
- rows 4 and 5 before rows 6 to 20,
- row 6 before rows 7 to 20 (an airstrike secondary decides alone: against anything but a
  charge-accepting building it gives 0, where rows 7 to 13 would give 1, as for a locomotor primary
  or an overpowered building, and a zero versus value does not change it),
- rows 7 to 12 before rows 14 to 20 (when applicable they give slot 1 even when its versus
  value is zero; rows 9, 10 and 12 can also apply to a cell target, where row 14 would give 0),
- row 13 before row 14 (a cell target with land mode 2 gives 1, not 0),
- row 14 before rows 15 to 20 (a target without an armour class has no versus value to test),
- row 15 before row 16 (two zero versus values give slot 0),
- rows 15 and 16 before rows 17 to 19 (a zero versus value overrides the water, land and air rules),
- row 17 before row 18 (a water target with land mode 2 follows table N, not slot 1; W3 depends on
  this).

Rows 18 and 19 cannot both apply (one needs an airborne target, the other a target that is not), so
their order does not matter. Rows 15 and 16 are S-0304 claim 6; rows 18 and 19 never
look at the primary's projectile, so
against an airborne target the secondary is chosen whenever it is anti-air, even when the primary is
anti-air too (`quirks.secondary_wins_against_air`).

**Table N: water targets by naval-targeting mode** (mode of the *firer*'s type; flags of the
*target*'s type).

| Mode | Slot |
|---|---|
| 0 | 0 for every target (the original answers "no weapon", see below, when the target's type is underwater and the target is cloaking, cloaked or uncloaking) |
| 1 | 1 if the target's type is underwater; else 0 |
| 2 | 0 for every target (the original answers "no weapon", see below, when the target's type is not underwater) |
| 3 | 1 if the target's type is organic or unnatural; else 0 |
| 4 | 1 if the target's speed type is not hover and its type is not organic; else 0 |
| 5 | 0 |
| 6 | 0 for every target (the original answers "no weapon" for every target, see below) |
| 7 | 0 |
| any other value | 0 |

Every mode gives slot 0 or 1. Where the original's choice answers "no weapon" (mode 0 against an
underwater target that is cloaking, cloaked or uncloaking, mode 2 against a target whose type is not
underwater, and mode 6), the result is slot 0, exactly as a slot-0 answer: nothing in the weapon
choice tells the two apart. The importer need not model "no weapon".

**Weapon number of a gunner vehicle.**

| State | Event | Result |
|---|---|---|
| no passenger | an object enters as passenger | weapon number = mode index of its type |
| one or more passengers | another boards, or one successfully exits and at least one remains | unchanged |
| one passenger | it successfully exits | weapon number = 0 |
| one or more passengers | position 1 fails to exit and stays inside | weapon number = that passenger's mode index, with the failed-exit quirk on |

All weapon-number changes in this table are subject to claim 5's clamping and charge-turret
exception. With the failed-exit quirk off, a failed exit leaves the gunner role and weapon
number unchanged.

**Occupant index of a garrisoned building.**

| Event | Result |
|---|---|
| a successful shot leaves the building, other than a disc-laser shot | index = (index + 1) mod occupant count |
| a disc-laser shot leaves the building | index unchanged |
| a firing attempt is refused or its shot is void | index unchanged |
| the building is emptied by occupants leaving or being killed | index = 0 |

Ordering: none of its own. The choice is made each time a rule asks for it, with the state at that
moment (for example the gattling stage and the deployed state of the same tick). S-0609 claims
1–9 own the ordinary gattling stage updates and their timing.

## Edge cases and quirks

- **Passenger weapons need a second weapon** (claim 4, row 4). A passenger of an open-topped
  transport uses its passenger weapon index only if it holds weapons in slots 0 and 1 and its
  secondary lacks the never-use flag; a single-weapon type ignores the index, except infantry of a
  deploy-fire type (claim 8). Switch `quirks.passenger_weapon_needs_secondary` (default on =
  original). Off: row 4 applies before row 3.
- **Gunner hand-over after a failed exit** (claim 6; S-0507 claim 10). Switch
  `quirks.unload_failed_exit_gunner_handover` (default on = original), shared with S-0507.
  A failed exit repeats the gunner hand-over for the passenger staying at position 1, changing
  the weapon number to its mode index even with others aboard, subject to claim 5. Off: the
  failed exit leaves the gunner role and weapon number unchanged.
- **Gattling air test reads slot 1** (claim 7). Switch `quirks.gattling_air_test_slot_one` (default
  on). Off: the anti-air test reads the weapon in slot 2*s* + 1.
- **Secondary wins against air targets** (table P, row 19). Switch
  `quirks.secondary_wins_against_air` (default on). Off: the secondary is chosen against an airborne
  target only when the primary's projectile is not anti-air.
- **Elite occupants skip the occupying weapon** (claim 13). Switch
  `quirks.elite_occupant_skips_occupy_weapon` (default on). Off: an elite occupant without an elite
  occupying weapon fires its type's occupying weapon when the type has one.
- **The airstrike row is final** (row 6). A firer whose secondary has an airstrike warhead never
  reaches the zero-versus, water, land or air rows. No switch: it is a data choice, not a defect.
- **Deploy weapon index not checked** (claim 9). A deployed deploy-fire vehicle uses its deploy
  weapon index as given. With an index outside 0 … 17 (including −1) the original's result is
  undefined. **Departure** (bad data): our engine treats such an index as 0
  and the importer warns for a deploy weapon index outside 0 … 17 on a type with the deploy-fire
  flag. The same holds for a passenger weapon index outside −1 … 17.
- **Mode index outside 0 … 17** (claim 6) gives weapon number 0, as claim 5 says. Our engine
  reproduces this and the importer warns.
- **The clear-weapons flag also empties numbered weapons** (claim 1). On a multi-weapon type the
  flag empties slots 0 and 1 (and elite slots 0 and 1) after the numbered keys filled them. Switch
  `quirks.clear_weapons_any_turret_count` (default on = original). Off: the flag acts only on types
  whose turret count is 0 or less.
- **Weapon count above 18** (claim 1). The original's result for using such a count is undefined.
  **Departure** (bad data): our engine retains registration effects from every applicable numbered
  normal and elite weapon field up to the imported weapon count, including fields above 18 and
  section reads of weapon types they create, but usable weapon slots and their elite slots remain
  limited to the first 18 (claim 16; S-0004 claims 6 and 16). Missing numbered fields create
  nothing. The importer warns for a count above 18.
- **Upgrade weapons win over occupants** (claim 12). A garrisoned building with an installed upgrade
  that supplies slot 0 fires the upgrade's weapon, not its occupants'. No switch proposed; this is a
  consequence of the slot rule.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, type section, key `TurretCount` | turret count: above 0 makes a multi-weapon type (claims 1, 5) | integer | held value; engine default 0 |
| Rules, type section, key `WeaponCount` | *N* of claim 1 | integer | held value; engine default 0 |
| Rules, type section, numbered keys `Weapon`*k* and `EliteWeapon`*k* for positive integer *k* | For a multi-weapon type, each present key with *k* ≤ *N* may create its named weapon type; only applicable keys with *k* ≤ 18 fill usable slot *k* − 1 or its elite slot in our engine. Applicable keys with *k* > 18 have registration effects but no usable slot (claims 1 and 16). | creating weapon type reference, 127-byte text limit, trimmed and resolved as S-0004 claim 13 specifies | held slot value for supported slots; engine default nothing; absent references create nothing |
| Rules, type section, keys `Primary`, `Secondary`, `ElitePrimary`, `EliteSecondary` | slots 0 and 1 and elite slots 0 and 1 of a type with turret count 0 or less (claim 1) | weapon type name | held value; engine default nothing |
| Rules, type section, key `ClearAllWeapons` | clear-weapons flag (claim 1): when set on a type with turret count 0 or less, its primary, secondary and elite keys create no weapon type in that layer (S-0004, claim 13); on every type, when set as a layer leaves the section, slots 0 and 1 and elite slots 0 and 1 are empty after that layer, even if the layer names weapons for them | boolean | held value; engine default no |
| Rules, type section, key `IsGattling` | gattling type (claim 7) | boolean | held value; default no |
| Rules, type section, key `IsChargeTurret` | charge-turret type (claim 5) | boolean | held value; default no |
| Rules, vehicle type section, key `Gunner` | gunner type (claim 6) | boolean | held value; default no |
| Rules, type section, key `IFVMode` | mode index (claim 6) | integer | held value; default 0 |
| Rules, type section, key `OpenTransportWeapon` | passenger weapon index (claims 8, 10) | integer | held value; default −1 |
| Rules, type section, key `DeployFire` | deploy-fire flag (claims 8, 9) | boolean | held value; default no |
| Rules, type section, key `DeployFireWeapon` | deploy weapon index (claims 8, 9) | integer | held value; default 1 |
| Rules, infantry type section, keys `OccupyWeapon`, `EliteOccupyWeapon` | occupying and elite occupying weapon (claim 13) | weapon type name | held value; default nothing |
| Rules, building type section, keys `CanBeOccupied`, `CanOccupyFire` | garrison firing (claim 13, row 2) | boolean | held value; default no |
| Rules, weapon section, key `NeverUse` | never-use flag (claim 4, row 3) | boolean | default no |
| Rules, weapon section, keys `Damage`, `AmbientDamage` | *d*ᵢ of F1 | integer | default 0 |
| Rules, weapon section, keys `DrainWeapon`, `AreaFire` | rows 8 and 9 | boolean | default no |
| Rules, projectile section, key `AA` | anti-air projectile (claim 7, row 19) | boolean | default no |
| Rules, warhead section, keys `Airstrike`, `IsLocomotor`, `ElectricAssault` | rows 6, 7 and 11 | boolean | default no |
| Rules, warhead section, key `Verses` | versus values (rows 15, 16; S-0304) | S-0003, claim 13 | S-0304 |
| Rules, type section, keys `Drainable`, `ResourceGatherer`, `ResourceDestination` | rows 8 and 6 | boolean | default no |
| Rules, building type section, key `CanC4` | accepts charges (row 6) | boolean | yes |
| Rules, building type section, key `Overpowerable` | overpowerable building type (row 11) | boolean | no |
| Rules, type section, key `LandTargeting` | land-targeting mode (rows 13, 18) | integer, used as given | held value; default 0 |
| Rules, type section, key `NavalTargeting` | naval-targeting mode (table N) | integer, used as given | held value; default 0 |
| Rules, type section, keys `Underwater`, `Organic`, `Unnatural` | target tests of table N | boolean | default no |
| Rules, type section, key `SpeedType` | hover test of table N (mode 4) | speed-type word (S-0211) | as S-0211 gives |
| Rules, type section, key `Naval` | row 13 | boolean | default no |

## Measurement scenarios

All scenarios run under the measurement setup with named test files: a test map whose override
sections give the test types and weapons named in each scenario. Director and all other mods
are off. Use a tiny flat skirmish map, one human observer and one bystander computer player
kept outside the observed area; a run is void if an unnamed bystander object affects that area.
Targets and firers named below are the observed area. Unless a part says otherwise, the
two weapons of a test firer are made distinguishable by giving their warheads 100 % against every
armour class and their weapons different damage fields, for example 7 and 13, so that the health a
target loses per hit identifies the weapon. Observations use a recorder that reads health values,
or the health bar at a resolution that separates the two amounts. Any game speed. One run per part
unless stated: no random draw enters the choice (claim 3), so repeated runs give the same result.

**W1: slots and elite weapons (claims 1 and 2).**
- *Setup:* firer *F*: a vehicle with turret count 0, primary *A* (damage 7), secondary *B*
  (damage 13), elite primary *A′* (damage 9) and no elite secondary. Target: a stationary enemy
  vehicle with a large health field.
- *Part a:* *F* at rookie rank, ordered to attack. *Expected:* hits of 7. *Part b:* *F* made elite
  (by the map's initial veterancy for its owner, or by kills), same order. *Expected:* hits of 9; with
  a second target against which the zero-versus row picks slot 1 (secondary *B* 100 %, *A′* and *A*
  0 % against its class), hits of 13 (the secondary is not replaced).
- *Part c:* the same type with turret count 1, weapon count 2, the first numbered weapon key = *C* (damage 5) and
  primary *A*. *Expected:* hits of 5 (slot 0 comes from the numbered key, the primary key is not
  read).
- *Part d:* the type of part c with weapon count 3, ordinary numbered weapons in slots 0, 1
  and 2 of damage 5, 13 and 11, and elite numbered weapons in slots 0 and 1 of damage 9 and 17.
  Set the clear-weapons flag; switch its weapon number to 0, 1 and 2 through gunner passengers
  (W4), at rookie and elite rank. *Expected:* no shot with weapon numbers 0 and 1 at either rank
  (ordinary and elite slots emptied), hits of 11 with 2 (no elite replacement there).
  Control with the flag clear: shots in slots 0 and 1 identify their ordinary or elite weapon.
  A slot-2 damage other than 11, or any firing through a cleared slot, falsifies this part.
- *Part e:* the type of part c with turret count 0 and the first numbered weapon key = *C* still set
  (damage 5), primary *A*. *Expected:* hits of 7 (the numbered key is not read).
- *Falsified if:* part a not 7; part b not 9, or 9 used against the second target; part c 7,
  or part d fires slot 0 or slot 1; part e 5.
- *Part f (numbered elite fields):* the type of part c, with an elite first numbered weapon of
  damage 9 and elite primary of damage 13; make the firer elite and attack. *Expected:* hits of
  9, from the numbered elite slot, not 13. Any other damage falsifies this part.

**W2: zero versus values and the airstrike, locomotor and drain rows (claim 3, rows 6 to 8, 15,
16).** S-0304 scenario V2 covers rows 15 and 16. For rows 6 to 8:
- *Part a (row 6):* a ground firer whose secondary has an airstrike warhead; targets: a
  building whose type accepts charges and is not both a resource gatherer and a resource destination,
  and an enemy vehicle. *Expected:* the
  secondary's effect against the building (the airstrike marker or its damage), the primary against
  the vehicle. Repeat with the primary's versus value against the vehicle set to zero:
  *Expected:* no shot, rather than a fallback to the legal secondary. Firing the secondary
  falsifies the airstrike row's precedence over zero-versus choice.
- *Part b (row 7):* a firer whose primary has a locomotor warhead and whose secondary is ordinary,
  against an enemy building. *Expected:* the secondary fires.
- *Part c (row 8):* a firer whose secondary is a draining weapon; target an enemy drainable
  building. *Expected:* the secondary; against an allied drainable building ordered with force-fire,
  the primary.
- *Falsified if:* any part shows the other weapon.

**W3: land, water and air rows (rows 13, 17 to 20, table N modes 1 to 3, 5 to 7).**
- *Setup:* firer *S*: a naval vehicle type with two distinguishable weapons; in its type section the
  land-targeting mode and naval-targeting mode are set per run. Targets: an enemy surface ship, an
  enemy submarine-type vehicle (underwater flag), an organic enemy (organic flag) in water, an enemy
  vehicle on land, and an airborne enemy aircraft; weapons with projectiles allowing each target.
- *Procedure:* order *S* to attack each target in turn, with force-fire where needed.
- *Expected:* as table N for the water targets and rows 18 and 19 for the land and air targets.
  With land mode 2 a land target gets slot 1, and a force-fire on a land cell gets slot 1 (row 13).
  With land mode 0 a land target and a force-fire on a land cell, and a force-fire on a terrain
  object, get slot 0 (rows 14 and 20). With land mode 2, a water target follows table N (row 17
  before row 18), not slot 1.
- *Falsified if:* any successful shot uses a different slot from the tables.
  The absence of a shot is inconclusive where another firing-legality rule could refuse it.

**W4: switching and gunner types (claims 5 and 6).**
- *Setup:* a gunner vehicle type with turret count 4 and weapon count 4, weapons 1 to 4 with damages
  3, 5, 7 and 11; two infantry types *P* and *Q* with mode indices 2 and 3; the gunner vehicle can
  carry two passengers.
- *Procedure:* (a) empty vehicle attacks; (b) *P* enters, attack; (c) *Q* enters as second
  passenger, attack; (d) *P* alone leaves, if ordinary input can make it do so, then attack.
  (e) all leave, attack; (f) give *P* a mode index of 25 in the test map, enter, attack.
- *Expected:* hits of 3, 7, 7, 7 (the first passenger's choice stays), 3, 3. If part d cannot
  remove *P* alone, leave that part unmeasured; unloading both and re-entering *Q* then *P*
  instead gives 11, but confirms only the empty-vehicle entry rule, not retention after *P* leaves.
- *Part g (temporal hand-over; countdown transfer low confidence):* the gunner vehicle's type is given weapon count 8,
  so that its slot 7 exists (claim 1; slots 4 to 6 may stay empty), and slot 7 holds a weapon with a
  rearm delay of about 300 ticks, and the weapon of mode index 2 has a delay of about 30 ticks.
  Infantry type *T* has an erasing (temporal) primary and mode index 2; it is erasing a stationary
  enemy vehicle when it is ordered into the empty gunner vehicle. Use a long passenger rearm
  delay and record the countdown at entry: the delay-transfer test applies only when it is still
  positive then. *Expected:* the enemy vehicle stays
  being erased (the erasure is carried on); the first shot the gunner vehicle fires after the entry
  comes after about 300 ticks, not 30, and then uses slot 2 (mode index 2). *Falsified if:* the first
  shot comes after about 30 ticks, or the erasure ends when *T* enters. More precisely, compare
  the observed countdown restart with the slot-7 delay from S-0301, including its ordinary delay
  factors, rather than treating the illustrative 300 ticks as an exact result. Repeat with *T*
  not erasing anything as a control; record its timing without assuming entry itself imposes a
  slot-2 delay. In that idle control, a read-only recorder logs which object holds the
  temporal beam before and after boarding: the vehicle must receive it even without a target.
  In a separate run, after boarding, let the vehicle lock a high-strength stationary target,
  then unload its sole passenger before completion. Record beam ownership and the target's
  being-erased state on exit: the passenger receives the beam but it is detached, and the
  target recovers with no saved progress. Repeat with another attacker attached: that
  attacker continues the erasure under S-0514 claims 9 and 10. A beam left on the vehicle,
  an idle beam not transferred, or a sole-attacker target remaining erased after exit
  falsifies the corresponding hand-over rule. Discard runs where another release cause
  ends the vehicle's lock before exit.
- *Part h (charge-turret rule):* the gunner vehicle of parts a to f with its charge-turret flag set.
  *Expected:* hits of 3 after *P* enters and after *Q* enters (the weapon number stays 0 whatever
  the passengers' mode indices); the same vehicle without the flag gives the results of parts a to f.
  *Falsified if:* any hit other than 3.
- *Part i (failed-exit exception):* use the two-passenger gunner of parts a to c, with *P*
  boarded first and *Q* last. Use game-speed index 2 and 800 × 600. Order an unload while
  neighbours are open; after the transport chooses its unloading cell and starts turning,
  move blockers into every exit before release, as in S-0507 scenario T10 part e. A read-only
  recorder logs passenger order, count, weapon number and tick. *Expected:* the failed exit
  leaves both inside, *Q* at position 1, but changes the weapon number from 2 to 3. Interrupt
  unloading and attack a stationary enemy target: hits of 11 rather than 7 identify the change.
  Repeat with the charge-turret flag: the weapon number remains 0 and hits remain 3.
  *Falsified if:* a reached failed exit retains weapon number 2 in the ordinary gunner, changes
  number in the charge-turret control, or the subsequent hits disagree. A run blocked before
  the unloading-cell choice is inconclusive: unloading may end without attempting an exit.
  With the quirk off in our engine, the ordinary gunner keeps number 2 and hits of 7.
- *Falsified if:* any completed part a to f differs from the listed damage, including part f
  not reverting to damage 3.

**W5: gattling types (claim 7).**
- *Setup:* a gattling vehicle type with turret count 1 and weapon count 4: slot 0 ground (damage 3),
  slot 1 anti-air (damage 5), slot 2 ground (damage 7), slot 3 anti-air (damage 11). A second
  variant in which slot 1 is not anti-air but slot 3 is. A third variant with slot 1 empty (slots 2
  and 3 as in the first variant).
- *Procedure:* attack a ground target and an airborne target at stage 0 and, after sustained fire
  has raised the stage, at stage 1 (the stage shows in the weapon change of the ground target).
- *Expected:* first variant: 3 / 5 at stage 0, 7 / 11 at stage 1. Second variant: the airborne
  target gets the ground slot (3, then 7), and since slot 2*s* is not anti-air the firing check
  decides whether it fires at all.
  Third variant: slot 0 (damage 3) against both targets at stages 0 and 1, whatever slots 2 and 3
  hold.
- *Falsified if:* any observed shot differs from the listed damage for its stage and target,
  including slot 3 against air at stage 1 in the second variant or slot 2 or 3 at stage 1 in
  the third. A refused shot alone does not identify its chosen slot.

**W6: special rows and modes 0 and 4 (rows 9 to 12, table N modes 0 and 4).**
- *Part a (row 9):* a type without the deploy-fire flag (claim 8 would otherwise decide) whose
  secondary has the area-fire flag, made to fire while its current behaviour is *unload* (an
  infantry type that deploys, ordered to deploy). *Expected:* slot 1 while *unload* is current; in
  *guard* against an ordinary target slot 0.
- *Part b (row 11):* a unit with an electric-charge secondary and an allied overpowerable building:
  ordered to attack the building. *Expected:* slot 1.
- *Part c (mode 0):* firer with naval-targeting mode 0 against an underwater enemy that is
  surfaced (uncloaked) and one that is submerged (cloaked). *Expected:* slot 0 for both; the damage
  of any shot identifies it. Record also whether a shot is fired at the submerged one (open
  question: this spec does not predict it).
- *Part d (mode 4):* firer with mode 4 against a hover vehicle on water, an organic swimmer and a
  ship. *Expected:* 0, 0, 1.
- *Part e (row 10):* an overpowerable building type with two distinguishable weapons (damages 7 and
  13) and an enemy vehicle in range against which the normal choice is slot 0. A unit with an
  electric-charge secondary charges the building as in part b, which makes it overpowered; the
  building then attacks the vehicle. *Expected:* hits of 13 while it is overpowered; hits of 7 once
  the state has ended. *Falsified if:* 7 while overpowered, or 13 afterwards.
- *Falsified if:* any observed weapon differs from the predicted slot. The absence of a shot in
  part c's submerged case is inconclusive for slot choice, because whether that shot is allowed remains
  open. Row 12 (kamikaze aircraft) has no scenario (claim 3).

**W7: passengers of open-topped transports (claims 4 and 10).**
- *Part a:* an infantry type with two weapons (damages 7 and 13) and passenger weapon index 1,
  inside an open-topped transport, the transport ordered to attack an enemy vehicle against which
  the normal choice is slot 0. *Expected:* hits of 13.
- *Part b:* the same with the secondary removed in the test map and passenger weapon index 1.
  *Expected:* hits of 7 (row 3 wins over row 4).
- *Part c:* restore the secondary and set its never-use flag. *Expected:* hits of 7.
- *Part d:* clear the never-use flag and remove the primary instead. *Expected:* no shot,
  because slot 0 is chosen and is empty, not hits of 13 from slot 1.
- *Part e:* repeat part a in a second open-topped transport type with passenger weapon index 0
  on the transport itself, leaving the passenger unchanged. *Expected:* still hits of 13
  from the passenger; its index, not the transport's, decides.
- *Falsified if:* part a or e differs from 13, b or c differs from 7, or d fires the secondary.

**W8: deployed weapons (claims 8 and 9).**
- *Part a:* an infantry type with the deploy-fire flag, deploy weapon index 1 and two weapons,
  undeployed, attacking a target against which the zero-versus row would pick slot 1 (primary 0 %).
  *Expected:* slot 0 chosen; with the primary at 0 % the firing check forbids the shot, so no hit.
- *Part b:* the same unit deployed. *Expected:* hits from slot 1.
- *Part c:* a vehicle type with the deploy-fire flag and deploy weapon index 0, deployed, against a
  target where table P would give 1. *Expected:* slot 0.
- *Part d (claim 8, passenger rows):* the infantry type of part a with passenger weapon index −1,
  riding in an open-topped transport that is ordered to attack a target against which the primary's
  versus value is 0 % and the secondary's 100 %. *Expected:* slot 0 is chosen, so no hit (the
  primary's zero versus value makes the shot illegal); the same unit without the deploy-fire flag
  gives hits of 13. With passenger weapon index 1 and a target both weapons hit, the deploy-fire
  unit gives hits of 13 too.
- *Falsified if:* part a fires slot 1, parts b and c use the table's slot, or part d hits with
  slot 1 for the deploy-fire unit with index −1.

**W9: garrisoned buildings and upgrades (claims 12 and 13).**
- *Setup:* an occupiable building that lets occupants fire; infantry type *E* with occupying weapon
  of damage 5 and primary of damage 7 and no elite occupying weapon; infantry type *G* without an
  occupying weapon, primary damage 11.
- *Part a:* occupants in this order: *E* (rookie), *G*, *E* (elite). Target a stationary enemy
  vehicle; record the damage of successive shots. *Expected:* 5, 11, 7 (or the elite primary if *E*
  has one), repeating. This part does not require an individual occupant to leave.
- *Part b:* the same with *E* given an elite occupying weapon of damage 9. *Expected:* 5, 11, 9.
- *Part c:* a building with an upgrade slot whose upgrade type holds a weapon in slot 0 (damage 17),
  garrisoned as in part a. *Expected:* every shot 17.
- *Part d (several upgrades, elite slots):* a building with two upgrade slots; upgrade type *U1*
  (ordinary slot 0 damage 17, elite slot 0 damage 21) is installed first, then *U2* (slot 0 damage
  19). *Expected:* every shot 17 (the first installed wins). Repeat with *U2* installed first:
  every shot 19. With the host building at elite rank (the map's initial veterancy for its owner)
  and *U1* installed: still 17, never 21.
- *Part e (disc-laser exception):* no upgrade; two rookie occupants entered in order, *E* then
  *G*. Give *E* a disc laser occupying weapon with damage 5, and *G* its ordinary damage-11
  primary. Order the building to attack the stationary enemy vehicle and record at least four
  successful shots, their damage and their muzzle points (as S-0313 P5). *Expected:* every
  shot uses *E*, damage 5, and the same muzzle point. Control: replace only *E*'s weapon with
  the ordinary occupying weapon of part a; damage then alternates 5, 11. Evacuate and refill:
  the first occupant fires again from index 0.
- *Part f (reset after emptying):* use part a's ordinary weapons and three occupants. Stop
  firing after one successful shot, while the index is 1; use a read-only recorder to confirm
  the index and that no further shot occurs. Evacuate all occupants through the building's
  deploy order with free exit cells (S-0508 claim 11). *Expected:* the empty building has index
  0 immediately after evacuation. Refill in the same order and attack again: the first hit is
  5, from the first occupant. Record the index before refilling so that an entry effect cannot
  masquerade as an evacuation reset. A nonzero index after evacuation, or a first shot from
  another occupant after refilling, falsifies this part.
- *Falsified if:* any damage in parts a to d differs from their expected sequences, part e
  rotates occupants or muzzles after a disc-laser shot while its ordinary-weapon control
  alternates, or part f differs from its reset and first-shot predictions.

**W10: mean weapon damage (claim 15).** Through S-0309. A test infantry type with primary damage
−5 and secondary damage 3 (*M* = trunc(−2 / 2) = −1) is a healer for acquisition: in *guard* next to
a damaged allied infantry unit it picks that unit by itself (S-0309, claim 6). With primary −3 and
secondary 3 (*M* = 0) it does not. With primary −1 and secondary 2 (*M* = trunc(1 / 2) = 0) it does
not retaliate when attacked (S-0309, claim 18), with primary −1 and secondary 4 (*M* = 1) it does.
- *Falsified if:* the unit with *M* = −1 does not pick the damaged ally, the unit with *M* = 0 does,
  or the retaliation results are reversed.
- *Additional parts:* repeat the acquisition pair with the negative contribution in the
  ambient-damage field instead of the damage field (ordinary damage 0): same acquisition result.
  Repeat using only the primary, with damage −1 then 0: acquisition then no acquisition.
  For the switching-type case use W4's gunner vehicle with slot 0 positive, slot 2 damage −1
  and slot 3 damage 0. *P* aboard selects slot 2: the vehicle seeks the damaged ally; *Q*
  aboard selects slot 3: it does not. For this vehicle test replace the damaged allied infantry
  with a damaged allied vehicle within 512 leptons, because a vehicle healer searches for vehicles.
  Keep all other acquisition eligibility conditions equal
  to the two-weapon control. Any reversed acquisition outcome falsifies the corresponding
  ambient, single-slot or switching-slot rule.
- *Negative rounding control:* repeat the two-weapon infantry acquisition setup with primary
  damage −2 and secondary damage 1, both ambient contributions 0. The sum is −1, so truncation
  toward zero gives *M* = 0: it must not acquire the damaged ally as a healer. Acquisition in
  this case, while the *M* = −1 positive control above acquires normally, falsifies the stated
  rounding (rounding downward instead would give −1).

**W11: registration beyond usable slots (claim 16; S-0004 K6).**
- *Setup:* named test files: a loose base-rules copy and art copy, tiny skirmish map, one human
  observer and one bystander computer player kept outside the test area. Director and mods off
  apart from the named test files. Any fixed game speed. A vehicle type has positive turret count
  and weapon count 19. Only its nineteenth normal weapon reference names a new weapon *W*.
  No other field names *W*. Its section names a new firing animation *A*, not listed elsewhere.
  The art section of *A* uses stock artwork and names a new decorative spawned overlay. Give the
  map that overlay's rules section and place its expected registration index as in S-0004 G3.
  Use a mode with no extra overlay registrations.
- *Procedure:* load once with this normal reference, once with only the nineteenth elite
  reference naming *W*, and once with count 18 while retaining the nineteenth reference.
  A positive control names *W* in the first slot instead. Keep the bystander away throughout.
- *Observable:* record the artwork at the expected new overlay index by screenshot. The overlay
  demonstrates that *W* read its section, created *A*, and *A* read its art in a later layer.
- *Expected:* the overlay appears in both count-19 runs and the first-slot control, not in the
  count-18 control. A successful count-19 load lacking the overlay while the first-slot control
  shows it falsifies the claim. Failure to load is inconclusive: the original's excessive count
  is unsafe. No firing through a nineteenth slot is attempted or predicted.
- *Our engine:* both count-19 imports create the overlay but provide no nineteenth firing slot,
  and issue the excessive-count warning. This checks the safety departure, not oracle behaviour.

**W12: choice does not draw randomness (claim 3).**
- *Setup:* W1's two-weapon vehicle, stationary and selected, with its rearm countdown long enough
  that no shot can occur during the observation; an enemy vehicle and an enemy building in view.
  Use an ordinary primary and an airstrike secondary so the attack-cursor test asks for different
  slots on the two targets. The random-state recorder logs both streams with tick numbers.
- *Procedure:* scripted input alternates hovering over the two targets without clicking, then
  repeats an equal-length interval hovering over empty ground. Repeat three paired intervals.
  Record cursor changes and stream advancement; discard intervals with unrelated visible random
  effects or a shot.
- *Expected:* the target-dependent cursor checks add no draws beyond the matched ground-hover
  control. A reproducible increase proportional to the number of checks falsifies the no-draw
  claim. Unexplained background advancement is inconclusive, not evidence for a choice draw.

## Open questions

- Which states make an aircraft a kamikaze aircraft (row 12): owned by the aircraft specs.
- S-0508 claim 5 gives entry order for the occupant list; W9 tests the resulting firing order.
  The index effect of an individual departure or death while other occupants remain is not
  established here. Claim 13's reset applies when the building is emptied, as S-0313 claim 6
  states. S-0311's Terms must narrow their reset wording to the same empty-building condition.
- What a new occupant's entry does to the occupant index of a garrisoned building (claim 13 and its
  state table give only shots and departures): whether the index keeps its value or returns to 0.
- The rearm-countdown transfer of claim 6, restarted from slot 7, remains low confidence
  pending W4 part g. Temporal beam transfer and exit release agree with S-0514 claims 9,
  10 and 14.
- Gattling spin is now specified by S-0609 claims 1–4, with ordinary stage-update timing in
  S-0609 claims 5–9; it is no longer an open question here.
- Whether any rule refuses a shot where table N's "no weapon" answers arise (modes 0, 2 and 6):
  the weapon choice gives slot 0 and nothing here forbids the shot.
