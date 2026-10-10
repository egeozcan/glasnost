---
id: S-0311
title: Veterancy (experience from kills, ranks, abilities, elite weapons, starting experience)
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

# S-0311: Veterancy (experience from kills, ranks, abilities, elite weapons, starting experience)

## Summary

Every infantry unit, vehicle, aircraft and building carries an experience value. Kills credited to
an object add experience to it, or to the object that the kill rules name in its place, in
proportion to the value of the victim and in inverse proportion to the recipient's own cost. Two
fixed thresholds turn experience into a veterancy rank: rookie, veteran or elite. A rank switches
on the veterancy abilities that the object's type lists for it, and the elite rank switches each
weapon slot to its elite weapon where the type has one. This spec owns the experience value, the
gain from kills, the cap, the ranks, which abilities a rank grants, the rule that only the elite
rank uses elite weapons, the ways an object starts with experience and the armory's promotion.
What each ability does is owned by the spec named for it in claim 4, and which weapon each slot
holds at the elite rank (with its exceptions) by S-0308 (claim 6 here); this spec gives an effect
only where no other spec does.

## Terms

Shared terms used here are defined in the glossary: *house*, *country*, *infantry*, *vehicle*,
*aircraft*, *building*, *type*, *field*, *weapon*, *tick*, *departure*, *importer warning*,
*quirk switch*, *round-toward-zero mode*, *rules database*, *human-controlled*, *local player*.
S-0205's notation applies: trunc, RZ₅₃, RZ₂₄, *single* and *double*. In addition:

- **Experience.** A number per object, held in single precision, that starts at 0 unless claim 11
  says otherwise. It can be negative: through a negative map veterancy value, a negative veteran
  ratio or cap (Edge cases), or the original's computer team system.
- **Veterancy rank.** Derived from experience *e* at the moment it is needed: *elite* when
  *e* ≥ 2, *veteran* when 1 ≤ *e* < 2, *rookie* otherwise, negative experience included. The
  comparisons are exact comparisons of the single value with 1 and 2. S-0214 and S-0301 use the
  same three words.
- **Veterancy ability.** One of eighteen capabilities that a type may grant at the veteran rank, at
  the elite rank or at both (claim 3). Our names, in the order of the import names: *faster*,
  *stronger*, *firepower*, *scatter*, *rate of fire*, *sight*, *cloak*, *resource-proof*,
  *vein-proof*, *self-heal*, *explodes*, *radar-invisible*, *sensors*, *fearless*, *explosive
  charges*, *resource-heal*, *area guard*, *crusher*. The import names are in Import mapping.
- **Ability lists.** Each type has a *veteran list* and an *elite list* of veterancy abilities.
- **Trainable.** A type flag (Import mapping). Of the recipient cases of claim 8, only the first,
  second and third test it; the infiltrated-factory rules of claim 11 test it too.
- **Naval.** A type flag (Import mapping), tested by the infiltrated-factory rules of claim 11.
- **Unscored.** A type flag (Import mapping): the destruction of an object of such a type gives no
  experience to anyone (claim 7).
- **Missile spawn.** A type flag of the missiles that a launching object creates and steers (for
  example a submarine's or a launcher's missile); the launching object is the missile's
  *launcher*. The spawning rules belong to a future spec.
- **Open-topped transport, passenger firing from it.** As S-0302 (Terms): a transport whose
  passengers fire from inside it.
- **Garrisoned building, firing occupant.** As S-0302 (Terms) for the garrisoned building. Its
  occupants fire in turn through the occupant index of S-0308 (claim 13): the index advances to
  the next occupant after each shot that leaves the building and returns to 0 when occupants
  leave (S-0313, claim 6, uses the same index for the muzzle). Its *firing occupant* at a given
  moment is the occupant at the index then, that is, the one whose weapon the building's next
  shot uses.
- **Actual cost.** The cost of a type for a given house: the type's cost multiplied by that
  house's cost factor for the type's kind and by the cost factor of the house's country for that
  kind, truncated (Formulas F1). The production specs own where the two factors come from; this
  spec only uses the result. The type's cost is its cost field, except that the production specs
  adjust it for some building types (for example by the cost of a unit the building comes with,
  see S-0314 claim 5). With no house, the actual cost is the type's cost.
- **Kill value.** The integer amount of experience-worth that a destruction carries (claim 7).
- **Credited object.** The object to which a destruction is credited: for a destruction by a hit,
  the attacker of S-0302 claim 14 when that claim credits the attacker; for the other
  destructions, the object named in claim 9.
- **Infiltrated infantry factory, infiltrated vehicle factory.** Two yes/no states of a house that
  an enemy infiltrator sets by entering one of the house's factories of infantry or of vehicles
  (the special-abilities specs, not yet written, own infiltration and anything shown about it).
- **Country veteran lists.** Three lists of type names per country, one each for infantry,
  vehicles and aircraft (Import mapping).
- **Armory.** A building type flag (Import mapping). S-0214 (table I, rows e1 to e3) gives which
  units may enter one.
- **Elite weapon.** For each weapon slot of a type (primary, secondary, and the numbered slots of
  a type with several weapons), an optional second weapon with its own firing positions (S-0313; Import
  mapping).

## Behaviour

### Experience and rank

1. **Experience and rank.** Every infantry unit, vehicle, aircraft and building has an experience
   value (Terms). Its veterancy rank is derived from it whenever a rule asks (Terms): rookie below
   1, veteran from 1 up to but not including 2, elite from 2 on. Ranks have no other state: an
   object whose experience is lowered below a threshold (by the cap of claim 10, for example)
   loses the rank at once.
   Experience never decays with time.
   - Backing: `measurable` (scenario X1)
   - Confidence: high

2. **Experience changes only by the rules of this spec.** Experience changes by kills (claims 7
   to 10), is set at creation (claim 11), by the armory (claim 12) and by a deployment (claim 13),
   and is changed by crates (the crate spec). No other change is known.
   - Backing: `measurable` (scenarios X1 and X8)
   - Confidence: medium (no other change was found; crates and the team system are outside this
     spec)

### Abilities and elite weapons

3. **Which abilities a rank grants.** An object of veteran rank has the abilities of its type's
   veteran list. An object of elite rank has the abilities of the veteran list together with those
   of the elite list. A rookie has none. So listing an ability on the veteran list alone gives it
   at both raised ranks, and an ability is never applied twice because it is on both lists: each
   ability is either present or absent. Every rule of the original that tests an ability uses this
   same test.
   - Backing: `measurable` (scenario X5)
   - Confidence: high

4. **What each ability does, and where it is owned.**

   | Ability | Effect while present | Owner |
   |---|---|---|
   | faster | the object's speed is multiplied by the veteran speed factor and truncated | S-0207 (vehicles, claim 2), S-0208 (infantry, claim 7 and F5) |
   | stronger | the damage the object takes is divided by the veteran armour factor | S-0302 (claim 5, F2) |
   | firepower | the object's shot damage is multiplied by the veteran combat factor | S-0302 (claim 2, F1) |
   | scatter | the object accepts the scatter requests that otherwise only a player setting allows | S-0208 (claims 11 to 15) |
   | rate of fire | the delay between shots is multiplied by the veteran rate-of-fire factor | S-0301 (claim 6) |
   | sight | the object's sight radius is multiplied by the veteran sight factor | this spec, claim 5 |
   | cloak | the object can cloak | visibility specs (future) |
   | resource-proof | no effect found (claim 5) | this spec |
   | vein-proof | the object is not harmed by veins | resource specs (future) |
   | self-heal | the object heals itself like a self-healing type | S-0314 (claim 9) |
   | explodes | the object explodes on death | S-0312 |
   | radar-invisible | the object is hidden from the radar | visibility specs (future) |
   | sensors | the object detects cloaked objects | visibility specs (future) |
   | fearless | damage raises the object's fear only in the fraidycat case; its fear still decays | S-0107 (claims 15 and 16), S-0315 |
   | explosive charges | the infantry unit can plant explosive charges | S-0214 (row I16) |
   | resource-heal | no effect found (claim 5) | this spec |
   | area guard | an object that falls back to rest rests in *area guard* instead of *guard* | S-0215 (claim 14 row 11 for infantry, claim 15 row 10 for vehicles; the behaviour itself is claim 26) |
   | crusher | the object crushes as a crusher type does | S-0210, S-0214 (table V) |

   - Backing: `measurable` through each owner's scenarios; scenario X5 here for firepower and
     stronger
   - Confidence: high for the test of claim 3 at every row; medium for the short descriptions of
     the effects owned by specs not yet written, which are pointers only

5. **The abilities owned here.**
   - *Sight.* When the object has the sight ability and the veteran sight factor is not exactly 0,
     its sight radius, as the visibility specs compute it from its type's sight field and its
     height, is multiplied by the factor and truncated toward zero (Formulas F4). A factor of
     exactly 0 leaves the radius unchanged. The same radius is used when the object's revealed
     area is later taken back.
   - *Resource-proof and resource-heal.* The original accepts both on the lists, but no rule of
     the original is known to test them, and no effect was found for the type flag and game-wide
     value of the same name for healing on resources either. Our engine gives them no effect.
   - Backing: `measurable` (scenario X10 for sight; scenario X11 for resource-heal)
   - Confidence: medium (sight: the factor and the zero case; the radius it multiplies belongs to
     the visibility specs); medium (resource-proof and resource-heal have no effect)

6. **Elite weapons.** Only the elite rank uses elite weapons: the veteran rank never changes
   weapons, and an object that is not elite never uses an elite slot. Since the rank is derived
   when asked, a unit that becomes elite uses its elite weapons from its next shot. Which weapon
   each slot then holds belongs to S-0308: the replacement per slot, where an elite slot with no
   weapon leaves the ordinary weapon and an empty slot stays empty (claim 2), with two
   exceptions that it states: an installed upgrade's elite slots are never used (claim 12), and
   an occupant that fires from a garrisoned building uses its elite occupying weapon or its own
   primary weapon (claim 13, `quirks.elite_occupant_skips_occupy_weapon`). Which keys fill the
   slots is S-0308's claim 1. The firing positions of an elite weapon are S-0313's, and which
   slot an object uses against a target is S-0308's.
   - Backing: `measurable` (scenario X6; S-0308 scenarios W1 and W9 for the slot rules)
   - Confidence: high

### Experience from kills

7. **Kill value.** The destruction of an object of an unscored type gives no experience to anyone,
   and the credited object's house adds nothing to its score total for it. Every other destruction
   credited to an object (claim 9) carries a kill value *V* (Formulas F1, F2). *V* is the actual
   cost of the destroyed object's type for the destroyed object's own house, multiplied by 1, 2 or
   3 as the destroyed object was a rookie, a veteran or elite, its rank being that of its
   experience at the moment of destruction. *V* is 0 instead when the credited object's house is
   allied with the destroyed object's house, as the credited object's house sees its alliances: a
   house counts as allied with itself, and a destroyed object with no house is never allied. The
   recipient of claim 8, if there is one, is given *V*, a *V* of 0 included, and gains experience
   by claim 10. Whether or not there is a recipient, the credited object's house adds *V* to a
   score total of its own (the score screen's, outside this spec).
   - Backing: `measurable` (scenarios X1 and X2)
   - Confidence: high

8. **The recipient.** The first of these cases that applies names the object that gains the
   experience; when none applies, nobody does.

   | Case | Credited object | Recipient |
   |---|---|---|
   | 1 | is a passenger firing from inside an open-topped transport whose type is trainable | the transport |
   | 2 | has a trainable type (and case 1 does not apply) | the credited object |
   | 3 | has a missile-spawn type | its launcher, when the launcher exists and its type is trainable; otherwise nobody |
   | 4 | is a garrisoned building whose occupants fire | its firing occupant at the moment of the destruction (Terms), whether or not the occupant's type is trainable (`quirks.occupant_experience_untrained`, `quirks.occupant_experience_next_in_turn`) |

   Case 4 is reached only when the garrisoned building's own type is not trainable; a trainable
   garrisoned building takes the experience itself by case 2. So a passenger firing from an
   open-topped transport whose type is not trainable gains experience itself when its own type is
   trainable, and an aircraft launched by a carrier (not a
   missile spawn) gains experience itself when its type is trainable.

   Because the occupant index (S-0308 claim 13) advances as soon as a shot is launched, with
   several occupants case 4 gives the experience to the occupant after the one that fired the
   building's most recent shot, which is normally not the one whose shot killed. With a single
   occupant it is that occupant.
   - Backing: `measurable` (scenario X3)
   - Confidence: high (cases 1 to 3 and their order); medium (case 4: the index advances when the
     shot is launched, before any hit it makes; whether some weapon could resolve its hit before
     the turn passes, which would credit the shooter, was not checked)

9. **Which destructions are credited.** A destruction by a hit is credited as S-0302 claim 14 says,
   and only a credited attacker object leads to experience: a kill credited to a house alone gives
   no experience. The original also credits an object in other kinds of destruction:
   - an erasure by an erasing weapon is credited to its firer.
   - passengers killed together with their transport are credited as S-0312 claim 5 says (each
     passenger's death to the attacker of the killing hit), and so are the passengers still inside
     a destroyed building (S-0312 claim 19, item h: the same credit).
   - **exception:** the absorbed passengers of a destroyed building that cannot be placed, or that
     all die because the building has no crew, are credited to the building's charge planter, or
     to no one when it has none, and never to the attacker of the killing hit (S-0312 claim 24).
     So the planter gains experience for them as for any credited kill (claim 8), and when there
     is no planter nobody does.
   - Backing: `measurable` (scenario X1 for hits; scenario X3, part d, for erasure, parts e and f
     for passengers)
   - Confidence: high (hits, and the credit of passengers killed with a transport or inside a
     building); medium (erasure and absorbed passengers)

10. **Experience gained.** The recipient's experience *e* becomes *e*′ of Formulas F3: the kill
    value divided by the product of the recipient type's actual cost and the game-wide veteran
    ratio, added to *e*, and limited to the game-wide veteran cap. The recipient type's actual cost
    is computed for the destroyed object's house, not the recipient's own
    (`quirks.experience_cost_victim_house`). The cap limits only this gain: experience above the
    cap that was set by claims 11 to 13 is not lowered by it unless the object is given a kill value
    (a kill value of 0 included), in which case the result is the cap. With the absent-key default
    of the cap, 1.0, kills can never make an object elite.
    - Backing: `measurable` (scenarios X1, X4 and X7)
    - Confidence: high (the formula, the cap, the house used for the cost); medium (the exact
      rounding, which shows only in borderline cases such as scenario X1 part c; it depends on the
      round-toward-zero mode of S-0205, so with `quirks.chop_rounding` off our engine rounds each
      step of F3 to nearest instead)

### Starting experience and other changes

11. **Experience at creation.**
    - An infantry unit is created as a veteran (experience 1) when its type is in its house's
      country veteran list for infantry, or when its house's infantry factory has been
      infiltrated and its type is trainable.
    - A vehicle is created as a veteran when its type is in the country list for vehicles, or when
      its house's vehicle factory has been infiltrated and its type is trainable and not naval.
    - An aircraft is created as a veteran when its type is in the country list for aircraft. An
      infiltrated factory never promotes aircraft.
    - A building is created as a veteran when its house's vehicle factory has been infiltrated and
      its type is trainable, not naval, and undeploys into a vehicle (a construction yard, for
      example).
    - An object placed by the map takes its experience from the map's veterancy value *n* (Import
      mapping): *e* = *n* × 0.01 (Formulas F5). This replaces whatever the rules above gave, so a
      map value of 0 makes even a type of the country list a rookie, and it is not limited by the
      cap: a map value of 200 makes an elite object even when the cap is 1.0.
    - Starting units of a skirmish are created as elites when a game-mode setting of the original
      is on (low: which setting this is was not identified; it is believed not to be reachable from
      the stock lobby).
    - Backing: `measurable` (scenario X7) for the country lists, the infiltrated factories and the
      map value; the starting-units rule is `unmeasured`, because the setting that turns it on is
      not identified and so no setup can be given (Open questions)
    - Confidence: high (country lists, the factory flags and the map value); medium (that the map
      value replaces the creation rules); low (the starting-units setting)

12. **The armory.** A unit that has entered an armory (S-0214 rows e1 to e3) stays inside for the
    armory time of Formulas F6, and is then made a veteran (experience exactly 1) if its
    experience was negative and an elite (experience exactly 2) otherwise, and is put out of the
    building. A unit that enters as a veteran with experience 1.5 therefore leaves with 2. The
    armory time is the infantry repair count *K* of S-0314 (F1) measured in processings of the
    armory's *repair* behaviour (S-0215) instead of in ticks: *K* processings, which come the rate
    delay *D* of *repair* apart. So it takes much longer than a hospital step with the same rate.
    - Backing: `measurable` (scenario X8)
    - Confidence: high (the result); medium (the timing of F6, and its phase within ±1 processing
      of the entry)

13. **Deploying keeps experience.** A vehicle that deploys into a building passes its experience to
    the building, and a building that undeploys into a vehicle passes its experience to the
    vehicle. Objects that change owner without being replaced (mind control, a vehicle thief, a
    capture) keep their experience because they stay the same object. No other passing of
    experience between objects is known.
    - Backing: `measurable` (scenario X9)
    - Confidence: high (deploy and undeploy); medium (that nothing else passes it)

14. **Promotion feedback.** When an object's rank changes to veteran or to elite, the local player
    hears the matching promotion sound and announcement if the object is theirs; an object that
    becomes elite also starts an elite flash on screen whatever its owner (the rendering specs own
    the flash, whose length is a game-wide field). The feedback follows the new rank, not the
    direction of the change: a fall from elite to veteran plays the veteran feedback. A fall to
    rookie, and the rank an object has when it first appears, give no feedback. This is
    presentation only.
    - Backing: `measurable` (scenario X1, parts a and d)
    - Confidence: medium

## Formulas

All floating-point steps follow S-0205: double results with a 53-bit significand, rounded toward
zero (RZ₅₃) under the *round-toward-zero mode* (`quirks.chop_rounding`, S-0205 claim 5), and
narrowings to single rounded toward zero (RZ₂₄).

**F1, actual cost.** *C* is the type's cost (integer; Terms), *h* the house's cost factor and *k* the
country's cost factor for the type's kind (both singles, owned by the production specs):

> *A* = trunc(RZ₅₃(RZ₅₃(*C* × *h*) × *k*)). Unit: credits. With no house, *A* = *C*.

**F2, kill value.** *A*ᵥ is F1 for the destroyed object's type and its own house:

> *V* = 0 when the credited object's house is allied with the destroyed object; otherwise
> *V* = *m* × *A*ᵥ with *m* = 1 (rookie), 2 (veteran) or 3 (elite).

Integer arithmetic. Unit: credits.

**F3, experience gained.** *e* is the recipient's experience (single), *A*ᵣ is F1 for the
recipient's type computed for the **destroyed object's** house, ρ is the veteran ratio and *c* the
veteran cap (both doubles):

> *s* = RZ₅₃(*e* + RZ₅₃(*V* / RZ₅₃(*A*ᵣ × ρ)))
>
> *e*′ = RZ₂₄(*c*) if *c* ≤ *s* or *s* is not a number, otherwise *e*′ = RZ₂₄(*s*).

The comparison is exact and uses *s* before narrowing. A divisor of 0 (a recipient of actual cost 0,
or ρ = 0) gives an infinite quotient for *V* > 0, so *e*′ is the cap; for *V* = 0 it gives a quotient
that is not a number, so *e*′ is again the cap (`quirks.zero_kill_value_caps`). Example: ρ = 4,
*A*ᵣ = 1000, a rookie victim with *A*ᵥ = 500 adds 0.125; eight such kills make a veteran.

**F4, sight with the sight ability.** *r* is the sight radius before the ability (integer, from the
visibility specs), *f* the veteran sight factor (double):

> *r*′ = trunc(RZ₅₃(*r* × *f*)) when *f* ≠ 0, otherwise *r*′ = *r*.

**F5, experience from the map.** *n* is the map's integer veterancy value:

> *e* = RZ₂₄(RZ₅₃(*n* × 0.01)), where 0.01 is the double nearest to one hundredth.

So *n* = 100 gives exactly 1 and *n* = 200 exactly 2.

**F6, armory time.** *r*ᵢ is the infantry repair rate of S-0314 (F1; a double, in game minutes) and *D* the
rate delay of the *repair* behaviour (S-0215, F1, no random draw), counted as 1 when it is 1 or
less:

> *K* = the smallest positive integer with RZ₅₃(*r*ᵢ × 900) ≤ *K* (an exact comparison; *K* = 1 when
> the product is 1 or less),
>
> armory time = *K* × *D* ticks, counted from the armory's first processing after the unit entered.

Example: with both the infantry repair rate and the rate of *repair* absent from the data (both
the double nearest 0.016), *K* = 15 and *D* = 14, so 210 ticks. With an infantry repair rate of
0.0078125 and a *repair* rate of 0.015625 (both exact in single precision), *K* = 8 and *D* = 14,
so 112 ticks.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Veteran threshold | 1 | experience | Claim 1 |
| Elite threshold | 2 | experience | Claim 1 |
| Kill value multiplier, veteran victim | 2 | factor | Claim 7 |
| Kill value multiplier, elite victim | 3 | factor | Claim 7 |
| Map veterancy scale | 0.01 | experience per map unit | Claim 11, F5 |
| Experience set by the armory | 2 (or 1 from negative) | experience | Claim 12 |
| Experience at creation by a veteran rule | 1 | experience | Claim 11 |
| Veteran ratio, absent-key default | 10.0 | factor | Claim 10 |
| Veteran cap, absent-key default | 1.0 | experience | Claim 10 |
| Veteran combat, armour, speed, rate-of-fire and sight factors, absent-key default | 1.0 | factor | Claims 4 and 5 |
| Veterancy abilities | 18 | abilities | Terms |
| Armory count advanced per processing of its *repair* behaviour | 1 | counts per processing | Claim 12, F6 |

## State and ordering

| State (rank) | Event | Result |
|---|---|---|
| any | a credited kill changes experience (claim 10) | rank re-derived; feedback when the rank changes to veteran or elite (claim 14) |
| rookie or veteran | experience reaches 2 | elite: elite list abilities added, elite weapons used from the next shot |
| inside an armory | *K* × *D* ticks have passed (F6) | elite (veteran from negative experience), put out |
| any | vehicle deploys into a building, or building undeploys | the new object has the same experience |

Ordering: the kill value uses the destroyed object's rank at its destruction, so killing an elite
counts triple even when the destruction itself also promotes the killer. Several destructions in
one tick that credit the same recipient add their gains one after another, each limited by the cap
(the order of destructions within a tick is S-0203's).

## Edge cases and quirks

- **Recipient cost uses the destroyed object's house** (`quirks.experience_cost_victim_house`, default
  the original). The recipient's actual cost in F3 is computed with the cost factors of the
  destroyed object's house and country. When the two houses have the same factors this changes
  nothing. Switch off: the recipient's own house is used.
- **Garrison occupants need not be trainable** (`quirks.occupant_experience_untrained`, default the
  original). Case 4 of claim 8 gives experience to the firing occupant without testing its type's
  trainable flag. Switch off: an occupant whose type is not trainable gains nothing.
- **The next occupant in turn gets the garrison's experience**
  (`quirks.occupant_experience_next_in_turn`, default the original). Case 4 of claim 8 names the
  occupant whose turn to fire it is when the destruction happens, which, with several occupants,
  is the one after the occupant that fired the building's latest shot. Switch off: the occupant
  that fired the killing shot gains the experience (when it is still inside).
- **A kill value of 0 against an actual cost of 0** (`quirks.zero_kill_value_caps`, default the
  original). F3 then sets the recipient's experience to the cap, for example when an object of
  actual cost 0 destroys an allied object. Switch off: a kill value of 0 never changes experience.
  An actual cost of 0 with a positive kill value also reaches the cap, in both settings, because the
  gain is unbounded; that is the original's arithmetic and needs no switch.
- **Ability list items are not trimmed.** Items of an ability list are compared with the ability
  names exactly as written between commas, ignoring letter case, so an item with a space before or
  after the name names no ability and is ignored, like any unknown name. This is an input quirk of
  the original: our engine reproduces it for all inputs and the importer warns for every list item
  that names no ability.
- **Bad data.** A negative veteran ratio or cap gives negative gains or a negative cap; the
  original does this without error and our engine reproduces it, with an importer warning. A
  negative map veterancy value gives negative experience (a rookie, and the armory then makes it a
  veteran only), reproduced, with an importer warning.
- **Team-created objects.** The original's computer team system can create objects with
  experience −0.25 or with a veteran or elite rank. That system is out of scope; our AI houses
  create objects only through the rules above.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, `[General]`, `VeteranRatio` | veteran ratio ρ (F3) | decimal (S-0003, claims 6 and 7), held as a double | 10.0; a layer without the key keeps the value in force |
| Rules, `[General]`, `VeteranCap` | veteran cap *c* (F3) | decimal, double | 1.0; keeps |
| Rules, `[General]`, `VeteranSight` | veteran sight factor *f* (F4) | decimal, double | 1.0; keeps |
| Rules, `[General]`, `VeteranCombat`, `VeteranArmor`, `VeteranSpeed`, `VeteranROF` | the factors of claim 4 (owned by S-0302, S-0207, S-0208, S-0301) | decimal, double | 1.0; keeps |
| Rules, type section, `VeteranAbilities`, `EliteAbilities` | veteran list and elite list (claim 3) | text cut to 127 characters (S-0003 claim 1), split at commas, each item compared without trimming and ignoring letter case with the names `faster`, `stronger`, `firepower`, `scatter`, `rof`, `sight`, `cloak`, `tiberium_proof`, `vein_proof`, `self_heal`, `explodes`, `radar_invisible`, `sensors`, `fearless`, `c4`, `tiberium_heal`, `guard_area`, `crusher` (written here in lower case, since case does not matter; our names in Terms, same order); a non-empty value replaces the whole list | empty list; an empty or missing value keeps the list in force |
| Rules, type section, `Trainable` | trainable flag | boolean (S-0003) | yes |
| Rules, type section, `DontScore` | unscored flag | boolean | no |
| Rules, type section, `MissileSpawn` | missile-spawn flag | boolean | no |
| Rules, type section, `Naval` | naval (claim 11) | boolean | no |
| Rules, building section, `UndeploysInto` | undeploys into a vehicle (claim 11) | type name | none |
| Rules, building section, `Armory` | armory (claim 12) | boolean | no |
| Rules, `[Repair]`, `Rate` (S-0215) | rate delay *D* of the *repair* behaviour (F6) | owned by S-0215 | S-0215 |
| Rules, `[General]`, `IRepairRate` | infantry repair rate *r*ᵢ (F6) | owned by S-0314 | S-0314 |
| Rules, type section, `Cost` | cost field *C* (F1) | integer | owned by the production specs |
| Rules, type section, `ElitePrimary`, `EliteSecondary`, `EliteWeapon1`, `EliteWeapon2` and so on | elite weapon of each slot (claim 6); which of these keys apply, by the type's turret count and the clear-weapons flag, is S-0308 claim 1 | weapon name | none: the slot has no elite weapon and the ordinary weapon is used |
| Rules, country section, keys `VeteranInfantry`, `VeteranUnits`, `VeteranAircraft` | country veteran lists for infantry, vehicles and aircraft (claim 11) | list of type names | empty |
| Map, `[Units]` and `[Aircraft]`, the ninth comma-separated value of an entry; `[Infantry]`, the tenth | map veterancy value *n* (claim 11, F5) | integer | each scenario writes it |
| Rules, `[AudioVisual]`, `EliteFlashTimer` | length of the elite flash (claim 14, presentation) | integer, ticks | owned by the rendering specs |

## Measurement scenarios

All scenarios run under the measurement setup with named test files: a flat test map whose
override sections define the test types and set the game-wide fields named. Ranks are read from
the rank insignia that the game draws on a selected object and from the promotion announcement.
Where a scenario needs the experience value itself, it is inferred from the number of kills at
which the insignia changes. Any game speed.

**X1: kill gain, threshold and cap (claims 1, 7, 10, 14).**
- *Setup:* ratio ρ = 4, cap 2.5. Killer *K*: a trainable vehicle type of cost 1000 owned by the human
  house, with a weapon that kills each target with one shot. Targets: unarmed enemy vehicles of a
  type of cost 500, all rookies, owned by a computer house whose country and difficulty give cost
  factors of 1 for vehicles.
- *Part a:* *K* kills targets one by one. *Observable:* the kill after which the veteran insignia
  and the announcement appear, and the kill after which the elite insignia appears. *Expected:*
  veteran after the 8th kill, elite after the 16th, and no change after further kills.
  *Falsified if:* other counts.
- *Part b:* as part a with the cap key removed in a loose copy of the rules file (named test file).
  *Expected:* veteran after the 8th kill and never elite. *Falsified if:* elite appears.
- *Part c (rounding):* ρ = 1.5, target cost 500, killer cost 1000. Each gain is RZ₅₃(1/3), and F3
  gives experience 0.33333331…, 0.66666662… and 0.99999994… after the first three kills and
  1.3333332… after the fourth. *Expected:* veteran after the 4th kill, not the 3rd. *Falsified if:*
  veteran after the 3rd kill, which is what rounding to nearest would give (the three narrowed
  values would then reach exactly 1); that result would falsify the rounding of F3 and bear on
  S-0205's `quirks.chop_rounding`.
- *Part d (feedback follows the new rank):* cap 1.5, and *K* placed by the map with veterancy
  value 250 (elite). *K* kills one target. *Expected:* *K* shows the veteran insignia and the
  veteran announcement plays (experience becomes the cap, 1.5). *Falsified if:* no announcement
  (that would mean a fall gives no feedback) or *K* stays elite (the cap would not apply).

**X2: victim rank multiplier and allied kills (claim 7).**
- *Setup:* as X1 part a, but the targets are placed by the map with veterancy values 100 (veteran)
  or 200 (elite), and a second set of targets belongs to an allied computer house.
  A third set of targets, owned by the enemy computer house, is of a type made unscored in the
  test map.
- *Expected:* with veteran targets *K* becomes a veteran after the 4th kill; with elite targets
  after the 3rd (3 × 0.375 ≥ 1); killing allied targets or unscored targets never promotes *K*.
- *Falsified if:* other counts, or any promotion from allied or unscored kills.

**X3: recipients (claims 8, 9).**
- *Part a (open-topped):* a trainable open-topped transport type carrying one trainable infantry
  passenger that fires from inside; the passenger kills targets of cost equal to the transport's.
  *Expected:* the transport's insignia rises, the passenger's never does. Second run with the
  transport's type made not trainable: the passenger's insignia rises.
- *Part b (missile):* a trainable launcher whose missiles (missile-spawn type, made not trainable
  in the test map) kill the targets. *Expected:* the launcher rises. Second run with the launcher
  not trainable: nobody rises. Third run with the launcher trainable and the missile type made
  trainable in the test map: *Expected:* the launcher does not rise, because case 2 comes
  before case 3 and the missile itself is the recipient. *Falsified if:* the launcher rises in the
  third run (case 3 would come before case 2).
- *Part c (garrison):* a garrisoned building whose type is made not trainable in the test map,
  with one infantry occupant whose type is also made not trainable. *Expected:* the occupant, once
  it leaves the building, shows a raised rank after enough kills. *Falsified if:* it does not (that
  would falsify the quirk's default). Sub-run: the building's type trainable (its absent-key
  default). *Expected:* the building's insignia rises and the occupant's does not (case 2 comes
  before case 4).
- *Part c2 (next occupant in turn):* the not-trainable garrisoned building of part c with two
  infantry occupants of not-trainable types: *A* of cost 100, which enters first, and *B* of cost
  1000. Targets of cost 500 that each die from one shot, ρ = 4, and a weapon whose delay between
  shots is long enough that every kill lands before the next shot. Run 1: the occupants are
  unloaded right after the first kill. Run 2: right after the second kill. *Expected:* run 1, *A*
  (which fires the first shot) is still a rookie (the kill went to *B*, whose gain of 0.125 does
  not show); run 2, *A* is a veteran (500 / 400 ≥ 1, credited while it was *A*'s turn).
  *Falsified if:* *A* is a veteran in run 1 (the shooter would be credited).
- *Part d (erasure):* a trainable unit with an erasing weapon erases targets. *Expected:* its rank
  rises after the same number of kills as in X1.
- *Part e (passengers killed with their transport):* a transport vehicle type of cost 500 that
  explodes on death carries two passengers of cost 500 each (rookies), and is shot dead by *K* (the
  cost-1000 trainable killer of X1, ρ = 4, cap 2.5); repeat with fresh transports. *Expected:* each
  destruction gives *K* three kill values (the transport and both passengers, 1500 in all, so
  0.375 of experience): veteran after the 3rd destruction, elite after the 6th. *Falsified if:*
  veteran only after the 8th (the passengers would give *K* nothing).
- *Part f (absorbed passengers with no crew):* a building that absorbs infantry, with the map's
  no-crew setting, holding two absorbed infantry units, is destroyed by *K*; no charge was planted.
  *Expected:* *K* gains the building's kill value only, and no insignia changes on any other
  object. Repeat with a charge planted by a trainable infantry unit *P* of another house before
  *K*'s killing hit: *Expected:* *P* gains for the two passengers and *K* only for the building.
  *Falsified if:* *K* gains for the passengers in either run.
- *Falsified if, in parts a to d:* a different object's insignia rises.

**X4: recipient cost uses the destroyed object's house (claim 10, quirk).**
- *Setup:* as X1 part a, with the target house's country given a vehicle cost factor of 2 in the
  test map and the killer's country a factor of 1.
- *Expected:* the kill value doubles (each target is worth 1000) and the killer's cost also doubles
  (2000), so the killer becomes a veteran after the 8th kill, as in X1. *Falsified if:* veteran after
  the 4th kill (that would mean the killer's own house is used for its cost).

**X5: abilities by rank (claims 3, 4).**
- *Setup:* a killer type whose weapon does 100 damage, with firepower on the elite list only and
  the veteran combat factor 2; targets with high health. Map-placed killers with veterancy values
  0, 100 and 200.
- *Observable:* health lost per hit. *Expected:* 100, 100, 200. Second run with firepower on the
  veteran list only: 100, 200, 200. Third run with firepower on both lists: 100, 200, 200 (not 400).
- *Falsified if:* other values.

**X6: elite weapons (claim 6).**
- *Setup:* a type with a primary weapon of damage 10 and an elite primary of damage 50, and a second
  type with no elite primary. Map-placed copies at veterancy 100 and 200.
- *Expected:* the first type's veteran hits for 10 and its elite for 50; the second type's elite
  hits for 10. *Falsified if:* otherwise.

**X7: starting experience (claim 11).**
- *Part a (country list and map value):* map-placed vehicles of a type in the human house's
  country veteran list for vehicles, with map veterancy values 0, 50, 150 and 250, cap 1.0 (cap
  key removed, named test file). Also build one such vehicle from a factory, and one infantry
  unit and one aircraft of types in the country's lists for infantry and for aircraft.
  *Expected:* insignia rookie, rookie, veteran, elite for the placed ones; veteran for each built
  one. *Falsified if:* the placed one with value 0 shows veteran (the map value would not
  replace), the one with 250 is not elite (the cap would apply), or a built one is not a veteran.
- *Part b (infiltrated factories):* a two-player game with two human-controlled houses, so that
  the second player can order an infiltrator of its own into the first house's vehicle factory
  and its infantry factory. The first house then builds, of types in none of its country veteran
  lists and with the trainable flag set in the test map as named: a trainable vehicle, a
  not-trainable vehicle, a trainable naval vehicle, a trainable infantry unit, a trainable
  aircraft, and a trainable, not-naval building that undeploys into a vehicle (built and placed,
  not deployed from a vehicle). *Expected:* the trainable vehicle, the infantry unit and the
  building are veterans; the not-trainable vehicle, the naval vehicle and the aircraft are
  rookies. *Falsified if:* otherwise.

**X8: armory (claim 12, F6).**
- *Setup:* a building type made an armory in the test map, owned by the human house; a rookie and
  a veteran trainable infantry unit (map veterancy 0 and 150); infantry repair rate 0.0078125 and
  the rate of the *repair* behaviour 0.015625 in the test file.
- *Observable:* the rank when each unit comes out, and the ticks from entry to exit.
- *Expected:* both elite; 112 ticks from entry to exit, within one processing (14 ticks) for the
  phase of the entry. A second run with the *repair* rate 0.03125 (*D* = 28): 224 ticks.
- *Falsified if:* a unit comes out with another rank, or the time does not scale with *D* (for
  example 21 ticks, which is what the hospital's count in ticks would give: 14 + 8 − 1).

**X9: deploying (claim 13).** A map-placed vehicle that deploys into a building (a construction
vehicle) with veterancy 150. Deploy it, then undeploy the building. *Expected:* the building shows
the veteran insignia, and so does the vehicle after undeploying. *Falsified if:* the building or
the vehicle shows no insignia (the experience would not be passed), or either shows the elite one
(the experience would have changed on the way).

**X10: sight factor (claim 5).** A type with the sight ability on its veteran list, placed at
veterancy 0 and 100, veteran sight factor 2, on a map with shroud. *Observable:* the radius of
revealed cells around each. *Expected:* the veteran's radius is twice the rookie's (to the cell
rounding of the visibility specs); a second run with the factor 0 gives both the same radius.
*Falsified if:* the two radii are equal with the factor 2, the veteran's is not twice the
rookie's beyond that rounding, or they differ with the factor 0.

**X11: no resource healing (claim 5).** A damaged vehicle of a type with the resource-heal ability
on both lists and the resource-heal type flag, placed at veterancy 200 on resource cells, no other
healing source. *Expected:* its health does not rise within 2000 ticks. *Falsified if:* its health
rises at any time (resource healing would exist, and claim 5 would be wrong).

## Open questions

- Which object a destruction that claim 9 does not list credits (erasure is only medium; other
  paths, such as occupants killed on their own, were not traced).
- Which game-mode setting makes skirmish starting units elite, and whether it can be reached.
- One parasite kind (a grappling naval parasite) gains experience on a path of its own, without the
  rank multiplier and the alliance test; the parasite spec should own it.
- Whether the resource-proof ability has any effect in the original (one candidate rule was not
  checked).
