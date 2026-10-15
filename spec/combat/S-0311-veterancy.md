---
id: S-0311
title: Veterancy (experience from kills, ranks, abilities, elite weapons, starting experience)
topic: combat
revision: 5
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
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
only where no other spec does. This spec covers skirmish; network games and campaigns are out of scope.

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
- **Trainable.** A type flag (Import mapping). Claim 8 tests it for transport, credited-object
  and launcher recipients, but not for the selected garrison occupant. The infiltrated-factory
  rules of claim 11 test it too.
- **Naval.** A type flag (Import mapping), tested by the infiltrated-factory rules of claim 11.
- **Unscored.** A type flag (Import mapping): the destruction of an object of such a type gives no
  experience to anyone (claim 7).
- **Missile spawn.** A type flag of the missiles that a launching object creates and steers (for
  example a submarine's or a launcher's missile); the launching object is the missile's
  *launcher*. S-0506 claims 1–13 own the spawning rules.
- **Open-topped transport, passenger firing from it.** As S-0302 (Terms): a transport whose
  passengers fire from inside it.
- **Garrisoned building, firing occupant.** As S-0302 (Terms) for the garrisoned building. Its
  occupants fire through the shared occupant/muzzle index owned by S-0313 claim 6 and used by
  S-0308 claim 13. It advances after each successful non-disc-laser shot, stays unchanged after
  a disc-laser shot, and returns to 0 when occupants leave or are killed. Its *firing occupant*
  at a given moment is the occupant at the index then, that is, the one whose weapon the building's next
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
- **Infiltrated infantry factory, infiltrated vehicle factory.** Two yes/no states belonging to
  the infiltrating house, not the victim house. A house's spy can set the corresponding state
  by entering another house's infantry or vehicle factory (S-0510 claim 5). Claim 11 reads
  these states from the new object's own house.
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
   - Backing: `measurable` for rank thresholds and losses (X1) and negative rookie rank (X8).
     No decay is measurable for X1's wait intervals; the unrestricted no-decay assertion is
     `unmeasured`: finite waits cannot test every duration or interaction.
   - Confidence: high

2. **Experience changes only by the rules of this spec.** Experience changes by kills (claims 7
   to 10), is set at creation (claim 11), by the armory (claim 12) and by a deployment (claim 13),
   and is changed by crates (the crate spec). A grappling naval parasite also gains experience
   from a cull under S-0614 claim 16 and F5. No other change is known.
   - Backing: the named changes inherit their claims' backing. The assertion that no other
     change exists is `unmeasured`: a finite set of scenarios cannot exclude every interaction.
     Crates and parasite culls are delegated to their owners, not tested here.
   - Confidence: medium (no other in-scope change was found)

### Abilities and elite weapons

3. **Which abilities a rank grants.** An object of veteran rank has the abilities of its type's
   veteran list. An object of elite rank has the abilities of the veteran list together with those
   of the elite list. A rookie has none. So listing an ability on the veteran list alone gives it
   at both raised ranks, and an ability is never applied twice because it is on both lists: each
   ability is either present or absent. Every rule of the original that tests an ability uses this
   same test.
   - Backing: `measurable` for firepower (X5). Other abilities inherit each written owner's
     backing. Applying this test to every original ability rule is `unmeasured`: X5 tests one
     ability, not every rule, and no effect parts for the future owners are supplied here.
   - Confidence: high

4. **What each ability does, and where it is owned.**

   | Ability | Effect while present | Owner |
   |---|---|---|
   | faster | in ground vehicle and infantry movement, speed is multiplied by the veteran speed factor and truncated; ordinary aircraft flight uses type speed directly, without this bonus | S-0207 (vehicles, claim 2), S-0208 (infantry, claim 7 and F5), S-0501 (aircraft, claim 4) |
   | stronger | the damage the object takes is divided by the veteran armour factor | S-0302 (claim 5, F2) |
   | firepower | the object's shot damage is multiplied by the veteran combat factor | S-0302 (claim 2, F1) |
   | scatter | the object accepts the scatter requests that otherwise only a player setting allows | S-0208 (claims 11 to 15) |
   | rate of fire | the delay between shots is multiplied by the veteran rate-of-fire factor | S-0301 (claim 6) |
   | sight | the nonzero veteran sight factor multiplies the radius before its visibility limit | this spec, claim 5; S-0409 claim 4 |
   | cloak | the object can start and retain cloaking under the ability-dependent rules | S-0516 claims 2, 5 and 9 |
   | resource-proof | no effect found (claim 5) | this spec |
   | vein-proof | the object is not harmed by veins | resource specs (future) |
   | self-heal | the object heals itself like a self-healing type | S-0314 (claim 9) |
   | explodes | the object explodes on death | S-0312 |
   | radar-invisible | the radar display applies the ability-dependent hiding rule | S-0410 claim 7 |
   | sensors | the object can force an enemy fully cloaked mobile object entering a neighbouring cell to uncloak, subject to the adjacency candidate test; the ability alone grants no sensor coverage | S-0516 claims 13 and 22 |
   | fearless | damage raises the object's fear only in the fraidycat case; its fear still decays | S-0107 (claims 15 and 16), S-0315 |
   | explosive charges | the infantry unit can plant explosive charges | S-0214 (row I16) |
   | resource-heal | no effect found (claim 5) | this spec |
   | area guard | an object that falls back to rest rests in *area guard* instead of *guard* | S-0215 (claim 14 row 11 for infantry, claim 15 row 10 for vehicles; the behaviour itself is claim 26) |
   | crusher | the object crushes as a crusher type does | S-0210, S-0214 (table V) |

   - Backing: written owners supply their effects' backing. X5 tests firepower, and S-0302
     D29 part c tests stronger. Future-owner descriptions are `unmeasured`: no effect parts for
     them are supplied here. Resource abilities have the split backing of claim 5.
   - Confidence: high for the rank test; medium for the future-owner descriptions

5. **The abilities owned here.**
   - *Sight.* When the object has the sight ability and the veteran sight factor is not exactly 0,
     the radius computed from its sight field and height is multiplied by the factor and
     truncated toward zero (Formulas F4), before S-0409 claim 4 applies its radius limit.
     A factor of exactly 0 leaves that intermediate radius unchanged. S-0409 owns sight
     application and removal.
   - *Resource-proof and resource-heal.* The original accepts both on the lists, but no rule of
     the original is known to test them, and no effect was found for the type flag and game-wide
     value of the same name for healing on resources either. Our engine gives them no effect.
   - Backing: `measurable` for sight (X10) and no resource healing during X11's observation
     interval. Resource-proof and the assertion that resource-heal has no effect in any rule
     are `unmeasured`: no resource-proof part is supplied, and a finite interval cannot exclude
     every resource-heal interaction. Giving them no effect in our engine is the design rule here.
   - Confidence: medium

6. **Elite weapons.** Only the elite rank uses elite weapons: the veteran rank never changes
   weapons, and an object that is not elite never uses an elite slot. Since the rank is derived
   when asked, a unit that becomes elite uses its elite weapons from its next shot. Which weapon
   each slot then holds belongs to S-0308: an elite slot with a weapon supplies that weapon,
   even when the ordinary slot is empty; without an elite weapon the ordinary slot's weapon
   or emptiness is kept (claim 2), with two
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
   - Backing: `measurable` (X1 and X2). One-way alliances, house-less victims and scoring
     without an experience recipient are `unmeasured` here: these parts use ordinary owned
     victims, symmetric alliances and a trainable killer, and supply no isolated variants.
   - Confidence: high

8. **The recipient.** A passenger firing from inside a trainable open-topped transport gives
   its kill experience to the transport, even when the passenger itself is trainable.
   Outside that transport rule, a trainable credited object gains the experience itself.
   An untrainable missile-spawn credited object gives its experience to its launcher only when
   the launcher exists and is trainable; otherwise nobody gains it, with no garrison fallback.
   An untrainable credited object that is not a missile spawn and is a garrisoned building whose
   occupants fire gives the experience to its firing occupant at destruction time (Terms),
   regardless of the occupant's trainability (`quirks.occupant_experience_untrained`,
   `quirks.occupant_experience_next_in_turn`). Any credited object not covered by these rules
   gives experience to nobody.

   A trainable garrisoned building therefore takes the experience itself, not through an occupant.
   A trainable passenger firing from an untrainable open-topped transport gains it itself.
   A trainable aircraft launched by a carrier, which is not a missile spawn, also gains it itself.

   For the garrison rule, the recipient at destruction is the occupant selected by the shared
   index (S-0313 claim 6; S-0308 claim 13). A successful non-disc-laser launch changes which
   occupant the index selects; a disc-laser shot leaves that selection unchanged. Consequently,
   with no intervening launch, occupant departure or death changing the selection, a destruction
   after an ordinary launch is credited to the next occupant, whereas after a disc-laser shot it
   is credited to that shot's shooter. With one occupant, that occupant is selected. A hit
   resolved during an ordinary launch before the index changes would name the shooter.
   No such weapon case is established here.
   - Backing: `measurable` for the recipient cases and ordinary/disc-laser comparison (X3).
     Timing of a hit resolved during launch is `unmeasured`: X3 uses hits after launch, and no
     synchronous-hit part is supplied. Carrier aircraft gaining for themselves is `unmeasured`
     here: no carrier-aircraft part is supplied. A missing missile launcher and a case where
     no recipient rule applies are `unmeasured`: X3 supplies neither. The trainable missile's
     gain itself remains `unmeasured` if it is removed before X3 b can sample it.
   - Confidence: high (transport, self and launcher rules and their priority); medium (the
     garrison current-index rule)

9. **Which destructions are credited.** A destruction by a hit is credited as S-0302 claim 14 says,
   and only a credited attacker object leads to experience: a kill credited to a house alone gives
   no experience.

   For a building in a delayed-kill countdown, the initial lethal hit and every further lethal
   hit credit a kill as S-0302 claim 16 says: each credited object supplies another kill value
   to its recipient (claims 7, 8 and 10) and another addition to its house's score. Healing and
   nonlethal hits do not credit a kill. At ordinary countdown expiry there is no attacker or
   attacking house: the initial hit's attacker is not retained. Nobody therefore gains further
   experience, score or an attacking-house kill count at expiry, although the victim's loss
   bookkeeping and destroyed-object events run again when the expiry hit is lethal. A separately
   recorded demolition-charge planter can instead be the expiry hit's attacker; that distinct
   case follows the same credit and recipient rules, not automatic credit to the original firer.
   S-0302 claim 16 owns expiry delivery and `quirks.delay_kill_credited_again` (default original):
   with that switch off, no further destruction is registered during the countdown or at expiry.

   The original also credits an object in other kinds of destruction:
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
     for transport and absorbed passengers; X12 for repeated lethal hits and ordinary expiry).
     Passengers still inside other destroyed buildings, absorbed passengers that cannot be
     placed, healing/nonlethal follow-up hits and a separately recorded charge planter at expiry
     are `unmeasured`: X3 does not isolate those passenger cases, and S-0302 claim 16 leaves
     the follow-up/planter cases unmeasured. No dedicated parts are supplied here
   - Confidence: high (hits, the lack of retained attacker at ordinary expiry, and the credit of
     passengers killed with a transport or inside a building); medium (repeated lethal-hit credit,
     erasure and absorbed passengers); low (healing/nonlethal follow-ups and charge-planter expiry)

10. **Experience gained.** The recipient's experience *e* becomes *e*′ of Formulas F3: the kill
    value divided by the product of the recipient type's actual cost and the game-wide veteran
    ratio, added to *e*, and limited to the game-wide veteran cap. The recipient type's actual cost
    is computed for the destroyed object's house, not the recipient's own
    (`quirks.experience_cost_victim_house`). The cap limits only this gain: experience above the
    cap that was set by claims 11 to 13 is not lowered by it until the object is given a kill value
    (a kill value of 0 included). That gain sets experience to the cap when its computed sum is
    at least the cap, as it is for nonnegative gains from experience above the cap; negative gains
    can instead leave a sum below it (F3). With the absent-key default of the cap, 1.0, kills
    cannot raise a non-elite object to elite.
    - Backing: `measurable` (X1, X4, X7 and X13). Negative ratios and caps are `unmeasured`
      here: none of these parts supplies negative gain parameters.
    - Confidence: high (the formula, the cap, the house used for the cost); medium (the exact
      rounding, which shows only in borderline cases such as scenario X1 part c; it depends on the
      round-toward-zero mode of S-0205, so with `quirks.chop_rounding` off our engine rounds each
      step of F3 to nearest instead)

### Starting experience and other changes

11. **Experience at creation.**
    - An infantry unit is created as a veteran (experience 1) when its type is in its house's
      country veteran list for infantry, or when its own house has the infiltrated-infantry-factory
      flag and its type is trainable.
    - A vehicle is created as a veteran when its type is in the country list for vehicles, or when
      its own house has the infiltrated-vehicle-factory flag and its type is trainable and not naval.
    - An aircraft is created as a veteran when its type is in the country list for aircraft. An
      infiltrated factory never promotes aircraft.
    - A building is created as a veteran when its own house has the infiltrated-vehicle-factory
      flag and its type is trainable, not naval, and undeploys into a vehicle (a construction yard,
      for example).
    - An object placed by the map takes its experience from the map's veterancy value *n* (Import
      mapping): *e* = *n* × 0.01 (Formulas F5). This replaces whatever the rules above gave, so a
      map value of 0 makes even a type of the country list a rookie, and it is not limited by the
      cap: a map value of 200 makes an elite object even when the cap is 1.0.
    - Starting units of a skirmish are created as elites when a game-mode setting of the original
      is on (low: which setting this is was not identified; it is believed not to be reachable from
      the stock lobby).
    - Backing: `measurable` (X7) for country lists, map values and infiltrated-factory branches
      (also S-0510 scenario I4). The observer builds for the infiltrating house, whose flags
      supply the starting experience, not for the enemy victim house. Starting-unit elites are
      `unmeasured`: their enabling setting is unidentified, so no setup can be given
      (Open questions)
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
    - Backing: `measurable` for deploy and undeploy (X9). Ownership changes are `unmeasured`
      here: X9 has no mind-control, thief or capture part. The assertion that no other transfer
      exists is `unmeasured`: a finite test cannot exclude every interaction.
    - Confidence: high (deploy and undeploy); medium (ownership changes and exhaustiveness)

14. **Promotion feedback.** When an object's rank changes to veteran or to elite, the local player
    hears the matching promotion sound and announcement if the object is theirs; an object that
    becomes elite also starts an elite flash on screen whatever its owner (the rendering specs own
    the flash, whose length is a game-wide field). The feedback follows the new rank, not the
    direction of the change: a fall from elite to veteran plays the veteran feedback. A fall to
    rookie, and the rank an object has when it first appears, give no feedback. This is
    presentation only.
    - Backing: `measurable` (X1 parts a, d and e for raised-rank feedback, a fall to veteran,
      and silent initial/rookie ranks). Nonlocal ownership and elite flashes are `unmeasured`
      here: no part isolates their owner-dependent presentation.
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

The comparison is exact and uses *s* before narrowing. With nonnegative cost and ratio, a divisor
of positive zero (a recipient of actual cost 0, or ρ = 0) gives a positive infinite quotient for
*V* > 0, so *e*′ is the cap; for *V* = 0 it gives a quotient that is not a number, so *e*′ is
again the cap (`quirks.zero_kill_value_caps`). Negative parameters follow F3's signed
floating-point arithmetic instead; its positive-infinity explanation does not cover them. Example: ρ = 4,
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
  original). The garrison rule of claim 8 gives experience to the firing occupant without testing its type's
  trainable flag. Switch off: an occupant whose type is not trainable gains nothing.
- **The next occupant in turn gets the garrison's experience**
  (`quirks.occupant_experience_next_in_turn`, default the original). The garrison rule of claim 8 names the
  occupant whose turn to fire it is when the destruction happens. After an ordinary launch this
  is normally the next occupant; a disc-laser shot does not advance the turn (claim 8). Later
  shots or occupant changes can change that turn before the kill. Switch off: the occupant
  that fired the killing shot gains the experience (when it is still inside).
- **A kill value of 0 against an actual cost of 0** (`quirks.zero_kill_value_caps`, default the
  original). F3 then sets the recipient's experience to the cap, for example when an object of
  actual cost 0 destroys an allied object. Switch off: a kill value of 0 never changes experience.
  An actual cost of 0 with a positive kill value also reaches the cap, in both settings, because the
  gain is unbounded; that is the original's arithmetic and needs no switch.
- **Ability list items are not trimmed** (`quirks.list_items_untrimmed`, default on, the original,
  shared with S-0003 claim 12). Items are compared with the ability names exactly as written
  between commas, ignoring letter case. Whitespace before or after a name prevents the match,
  so that item is ignored like an unknown name. With the switch off, items use S-0003's
  whitespace trim before matching; whitespace-only items become empty and are skipped.
  Unknown non-empty names remain ignored. The importer warns for surrounding whitespace while
  the switch is on, and for each non-empty item that matches no ability in either setting.
  Original parsing is `measurable` (X14), confidence high; the alternative and warnings are
  design rules, not oracle predictions.
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
| Rules, type section, `VeteranAbilities`, `EliteAbilities` | veteran list and elite list (claim 3) | text cut to 127 bytes (S-0003 claim 1), split at commas with empty items skipped, each item compared ignoring letter case and without trimming while `quirks.list_items_untrimmed` is on (trimmed as S-0003 claim 12 when off) with the names `faster`, `stronger`, `firepower`, `scatter`, `rof`, `sight`, `cloak`, `tiberium_proof`, `vein_proof`, `self_heal`, `explodes`, `radar_invisible`, `sensors`, `fearless`, `c4`, `tiberium_heal`, `guard_area`, `crusher` (written here in lower case, since case does not matter; our names in Terms, same order); a non-empty value replaces the whole list | empty list; an empty or missing value keeps the list in force |
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
Where insignia alone cannot distinguish the result, record exact experience, rank, health and
tick numbers under the glossary's measurement setup. Use a skirmish at game speed 2, 800 × 600 pixels,
with one human observer and one enemy bystander computer player, plus an allied bystander for
X2. Director and other mods are off, crates off, no superweapon use. Test objects and their
firing/production area are the observed area. Keep all unnamed bystander objects outside it,
and void and repeat a run if one affects it or a bystander is defeated before observation ends.
Give each house survival buildings outside the test area. Unless a part specifies otherwise,
perform one valid fresh-game run per part and variant. Test costs, damages and factors below
are invented inputs in named test overrides, not stock data. All cost factors relevant to F1
are explicitly 1 unless a part names another value.

**X1: kill gain, threshold and cap (claims 1, 7, 10, 14).**
- *Setup:* ratio ρ = 4, cap 2.5. Killer *K*: a trainable vehicle type of cost 1000 owned by the human
  house, with a weapon that kills each target with one shot. Targets: unarmed enemy vehicles of a
  type of cost 500, all rookies, owned by a computer house whose country and difficulty give cost
  factors of 1 for vehicles.
- *Part a:* *K* kills targets one by one. *Observable:* the kill after which the veteran insignia
  and the announcement appear, and the kill after which the elite insignia appears. *Expected:*
  veteran after the 8th kill, elite after the 16th, and experience capped at 2.5 after the
  20th kill. Wait 2000 ticks without combat before the 8th and after the 20th kill:
  experience must stay unchanged during each wait. Record the promotion sound and announcement
  at both thresholds. *Falsified if:* other counts, a different recorded cap, missing promotion
  feedback, or experience changes during either wait.
- *Part b:* as part a with the cap key removed in a loose copy of the rules file (named test file).
  *Expected:* veteran after the 8th kill and experience stays at 1 after further kills.
  Stop after the 24th kill. Any different promotion count, experience above 1, or elite rank
  within this interval falsifies the default-cap prediction.
- *Part c (rounding):* ρ = 1.5, target cost 500, killer cost 1000. Each gain is RZ₅₃(1/3), and F3
  gives experience 0.33333331…, 0.66666662… and 0.99999994… after the first three kills and
  1.3333332… after the fourth. *Expected:* veteran after the 4th kill, not the 3rd. *Falsified if:*
  any recorded experience differs from F3 or the veteran rank appears at a kill other than the
  4th. Veteran after the 3rd is what rounding to nearest would give (the three narrowed values
  would then reach exactly 1); that result would falsify F3's rounding and bear on
  S-0205's `quirks.chop_rounding`.
- *Part d (feedback follows the new rank):* cap 1.5, and *K* placed by the map with veterancy
  value 250 (elite). *K* kills one target. *Expected:* *K* shows the veteran insignia and the
  veteran announcement plays (experience becomes the cap, 1.5). Record experience and feedback.
  Any other experience/rank or missing veteran feedback falsifies this part. No announcement
  would indicate silent demotion; staying elite would indicate that the cap did not apply.
- *Part e (silent initial and rookie ranks):* start fresh runs with *K* map-placed at veterancy
  100 and 200, and record sound from its first appearance. Neither initial rank gives promotion
  feedback. Then use the elite copy with cap 0.5 and kill one target. Expected: experience 0.5,
  rookie insignia state, no promotion sound or announcement. Any initial promotion feedback or
  feedback on the fall to rookie falsifies claim 14.

**X2: victim rank multiplier and allied kills (claim 7).**
- *Setup:* as X1 part a, but the targets are placed by the map with veterancy values 100 (veteran)
  or 200 (elite), and a second set of targets belongs to an allied computer house.
  A third set of targets, owned by the enemy computer house, is of a type made unscored in the
  test map.
- *Expected:* with veteran targets *K* becomes a veteran after the 4th kill; with elite targets
  after the 3rd (3 × 0.375 ≥ 1); killing allied targets or unscored targets never promotes *K*.
- Record *K*'s experience and its house's score delta per destruction as well as insignia.
  Expected: kill values 1000 and 1500 credits for the veteran and elite enemy victims,
  respectively; 0 experience gain and 0 score delta for allied or unscored victims.
- *Falsified if:* other counts or score deltas, or any experience gain from allied or unscored
  kills (even when too small to promote).

**X3: recipients (claims 8, 9).**
- *Part a (open-topped):* a trainable open-topped transport type carrying one trainable infantry
  passenger that fires from inside; the passenger kills targets of cost equal to the transport's.
  *Expected:* the transport's insignia rises, the passenger's never does. Second run with the
  transport's type made not trainable: the passenger's insignia rises.
- *Part b (missile):* a trainable launcher whose missiles (missile-spawn type, made not trainable
  in the test map) kill the targets. *Expected:* the launcher rises. Second run with the launcher
  not trainable: nobody rises. Third run with the launcher trainable and the missile type made
  trainable in the test map: *Expected:* the launcher does not rise, because the missile's own
  trainability gives it the experience instead. *Falsified if:* the launcher rises in the
  third run (launcher credit would override the trainable credited object's gain).
- *Part c (garrison):* a garrisoned building whose type is made not trainable in the test map,
  with one infantry occupant whose type is also made not trainable. *Expected:* the occupant, once
  it leaves the building, shows a raised rank after enough kills. *Falsified if:* it does not (that
  would falsify the quirk's default). Sub-run: the building's type trainable (its absent-key
  default). *Expected:* the building's insignia rises and the occupant's does not (the
  building's own trainability takes priority over its occupant).
- *Part c2 (next occupant in turn):* the not-trainable garrisoned building of part c with two
  infantry occupants of not-trainable types: *A* of cost 100, which enters first, and *B* of cost
  1000. Targets of cost 500 that each die from one shot, ρ = 4, cap 2.5, and a weapon whose delay between
  shots is long enough that every kill lands before the next shot. Run 1: the occupants are
  unloaded right after the first kill. Run 2: right after the second kill. *Expected:* run 1, *A*
  (which fires the first shot) is still a rookie (the kill went to *B*, whose gain of 0.125 does
  not show); run 2, *A* is a veteran (500 / 400 ≥ 1, credited while it was *A*'s turn).
  Record both experience values at each kill. Expected: run 1, *A* = 0 and *B* = 0.125.
  In run 2, *A* = 1.25 and *B* = 0.125. Any different value or rank falsifies this part.
  *A* being veteran in run 1 would indicate shooter credit.
- *Part c3 (disc laser):* repeat c2 run 1 with only *A*'s occupying weapon changed to a disc
  laser that kills the same victim with one shot. Record which occupant fires and both
  experience values at the destruction, then unload them. Expected: index remains on *A*,
  *A* gains 1.25 and becomes veteran, *B* stays at 0. Any different experience value or rank
  falsifies the disc-laser recipient prediction. The ordinary-shot control c2 run 1 instead
  gives *B* 0.125 and leaves *A* at 0. Void if another shot or an occupant change precedes the
  destruction. S-0313 P5 part b separately tests repeated disc-laser index retention.
- *Part d (erasure):* a trainable unit of cost 1000 with an erasing weapon erases cost-500
  rookie targets, ratio 4 and cap 2.5 as X1. *Expected:* veteran after 8 erasures, elite after 16.
- *Part e (passengers killed with their transport):* a transport vehicle type of cost 500 that
  explodes on death carries two passengers of cost 500 each (rookies), and is shot dead by *K* (the
  cost-1000 trainable killer of X1, ρ = 4, cap 2.5); repeat with fresh transports. *Expected:* each
  destruction gives *K* three kill values (the transport and both passengers, 1500 in all, so
  0.375 of experience): veteran after the 3rd destruction, elite after the 6th. Record experience
  after each destruction. Any different delta or promotion count falsifies this part.
  Veteran only after the 8th would indicate no passenger credit.
- *Part f (absorbed passengers with no crew):* a building that absorbs infantry, with the map's
  no-crew setting, holding two absorbed infantry units, is destroyed by *K*; no charge was planted.
  *Expected:* *K* gains the building's kill value only, and no insignia changes on any other
  object. Repeat with a charge planted by a trainable infantry unit *P* of the human house
  before *K*'s killing hit. Give the enemy building and its two enemy passengers cost 500
  each, *P* cost 1000, ratio 4 and cap 2.5. *Expected:* *P* gains 0.25 for the two passengers
  and *K* gains 0.125 for the building alone. *Falsified if:* either experience delta differs.
- In parts a, b and c (not c2/c3), use ratio 1 and cap 2, enough fresh targets to cross a rank
  threshold, and record each named recipient's experience delta per kill. Use equal recipient
  and victim costs so the predicted gain is 1. A delta on a different object, no delta on the
  named recipient, or any gain in a nobody-rises variant falsifies that part. For b's trainable
  missile variant, record its experience before removal if possible; if the missile is removed
  before it can be sampled, only the launcher's lack of gain is tested, not the missile's gain.
- *Falsified if, in parts a to d:* a different object's experience or insignia rises.

**X4: recipient cost uses the destroyed object's house (claim 10, quirk).**
- *Setup:* as X1 part a, with the target house's country given a vehicle cost factor of 2 in the
  test map and the killer's country a factor of 1.
- *Expected:* the kill value doubles (each target is worth 1000) and the killer's cost also doubles
  (2000), so each gain is 0.125 and the killer becomes a veteran after the 8th kill, as in X1.
  Record experience after every kill. Any different delta or promotion count falsifies this
  part; veteran after the 4th would indicate use of the killer's own house for its cost.

**X5: abilities by rank (claims 3, 4).**
- *Setup:* a killer type whose weapon does 100 damage, with firepower on the elite list only and
  the veteran combat factor 2; targets with high health, versus values 100 %, all other
  damage factors 1 and no independent defence or healing. Map-placed killers with veterancy values
  0, 100 and 200.
- *Observable:* health lost per hit. *Expected:* 100, 100, 200. Second run with firepower on the
  veteran list only: 100, 200, 200. Third run with firepower on both lists: 100, 200, 200 (not 400).
- *Falsified if:* other values.

**X6: elite weapons (claim 6).**
- *Setup:* a type with a primary weapon of damage 10 and an elite primary of damage 50, and a second
  type with no elite primary. Map-placed copies at veterancy 100 and 200. Both ability lists
  are empty, target versus values are 100 %, other damage factors 1, no independent defence.
  Add a third type with an empty ordinary primary and elite primary of damage 50. Expected:
  its veteran cannot shoot from that slot, while its elite hits for 50. A veteran shot or no
  elite damage-50 shot falsifies the slot replacement for that variant.
- *Expected:* the first type's veteran hits for 10 and its elite for 50; the second type's elite
  hits for 10. *Falsified if:* otherwise.
- *Promotion sub-run:* put the first type at map veterancy 150, cost 1000, ratio 1, cap 2.5.
  Let its damage-10 shot kill an enemy rookie cost-500 target with health 10, then attack a
  fresh high-health target. With versus 100 % and other damage factors 1, expected: the kill
  raises experience to 2 and the very next shot deals 50. A next shot of 10, or any other
  damage, falsifies the immediate elite-weapon rule.

**X7: starting experience (claim 11).**
- *Part a (country list and map value):* map-placed vehicles of a type in the human house's
  country veteran list for vehicles, with map veterancy values 0, 50, 150 and 250, cap 1.0 (cap
  key removed, named test file). Also build one such vehicle from a factory, and one infantry
  unit and one aircraft of types in the country's lists for infantry and for aircraft.
  *Expected:* insignia rookie, rookie, veteran, elite for the placed ones; veteran for each built
  one. Record experience too: 0, 0.5, 1.5 and 2.5 for the placed vehicles, 1 for each built
  unit. *Falsified if:* any experience or insignia differs from those results.
- *Part b (country lists without trainability):* repeat the three factory-built country-list
  types of part a with their trainable flags off. Expected: all still start at experience 1.
  Any rookie result falsifies the country-list rule's lack of a trainability test.
- *Part c (infiltrating-house flags):* use S-0510 scenario I4's skirmish setup and procedure,
  including empty country veteran lists. Record experience and rank of our newly built infantry,
  ground vehicles and aircraft before and after each spy entry. Expected: experience 0 before
  infiltration; after the barracks entry, infantry experience 1 and vehicles 0; after the naval-yard
  entry, ground vehicles 1 and aircraft 0. These are our units, not the enemy victim's.
  In separate fresh runs of that setup, named test-map overrides add untrainable infantry,
  untrainable ground vehicles, trainable naval vehicles and two trainable non-naval building
  types, one that undeploys into a vehicle and one that does not. Build and place the buildings
  normally, not as map-placed objects; keep all objects out of combat. After both entries,
  expected experience: 0 for the untrainable infantry, untrainable ground vehicles and naval
  vehicles; 1 for the building that undeploys and 0 for the other. Repeat the undeployable
  building variant with its trainable flag off, then with its naval flag on: both start at 0.
  Perform one valid fresh-game run per variant at game speed 2 with director and mods off,
  using the read-only recorder for experience where insignia is insufficient.
  *Falsified if:* any experience or rank differs from these predictions.

**X8: armory (claim 12, F6).**
- *Setup:* a building type made an armory in the test map, owned by the human house; a rookie and
  a veteran trainable infantry unit (map veterancy 0 and 150); infantry repair rate 0.0078125 and
  the rate of the *repair* behaviour 0.015625 in the test file.
- *Observable:* the rank when each unit comes out, and the ticks from entry to exit.
- *Expected:* both elite; 112 ticks from entry to exit, within one processing (14 ticks) for the
  phase of the entry. A second run with the *repair* rate 0.03125 (*D* = 28): 224 ticks.
- Add a third infantry unit map-placed with veterancy −25. Expected: it leaves with experience
  exactly 1 (veteran), while the nonnegative units leave with exactly 2. Record experience and
  rank at exit for all three.
- *Falsified if:* any exit experience/rank differs, or duration is outside one processing of
  112 ticks in the first run or 224 in the second. For example 21 ticks would match the
  hospital's tick count instead (14 + 8 − 1).

**X9: deploying (claim 13).** A map-placed vehicle that deploys into a building (a construction
vehicle) with veterancy 150. Deploy it, then undeploy the building. Record experience before
and after both changes. *Expected:* experience 1.5 throughout and veteran insignia on both
forms. *Falsified if:* either experience changes or either form has another rank.

**X10: sight factor (claim 5).** A ground infantry type with sight field 4 and the sight ability
on its veteran list, placed at veterancy 0 and 100, veteran sight factor 2, on level-0 ground
with shroud and no other sight covering the sampled cells. *Observable:* the radius of
revealed cells around each. *Expected:* S-0409's radius-4 shape for the rookie and radius-8 shape
for the veteran; a second run with factor 0 gives radius 4 for both. A third run with factor
1.375 gives radius 5 for the veteran (truncation of 5.5), radius 4 for the rookie.
*Falsified if:* any sampled revealed-cell set differs from those S-0409 shapes.

**X11: no resource healing (claim 5).** A damaged vehicle of a type with the resource-heal ability
on both lists and its resource-heal type flag enabled, placed at veterancy 200 on resource cells.
Set the game-wide resource-heal value to the invented test value 1, with no other healing source.
Record its health at placement and throughout the interval. *Expected:* its health does not rise within 2000 ticks. *Falsified if:* its health
rises at any time (resource healing would exist, and claim 5 would be wrong).

**X12: delayed-kill credit and expiry (claim 9).** Reuse S-0302 D13 parts c1, c2 and d
for repeated lethal hits, and part e for ordinary expiry: their named test map, one-cell and
multi-cell buildings, human-owned trainable attacker and enemy-owned victims, with no planted
charge. Run as a skirmish at game speed 2 with the director and mods off, under the measurement
setup. Perform one valid run per sub-test.
Record the attacker's experience, house score and kill count, the victim house's loss bookkeeping,
and destroyed-object events at each lethal hit and expiry. Expected: the repeated lethal hits
add kill values and attacker credit again; ordinary lethal expiry adds no further experience,
score or attacking-house kill count, but repeats victim loss bookkeeping and destroyed-object
events. Falsified by missing repeated lethal-hit credit, credit to the original firer at ordinary
expiry, or missing victim loss/events at lethal expiry. On our engine repeat with
`quirks.delay_kill_credited_again` off: only the initial lethal hit registers destruction.

**X13: zero kill value and zero divisor (claim 10, F3).** Use named map overrides with ratio 4,
cap 2.5, rookie human-owned trainable killer and no country veteran list. In separate fresh
runs, set killer cost to 0 and force-fire at (a) a cost-500 human-owned rookie vehicle and (b) a
cost-500 enemy rookie vehicle. Record experience and victim alliance at destruction. Expected:
experience becomes 2.5 in both runs, from the zero-over-zero and positive-over-zero cases.
Repeat with killer cost 1000, ratio 0: the same two results. Control: killer cost 1000, ratio 4,
human-owned victim, initial killer map veterancy 250 and cap 1.5. Expected: even this zero kill value lowers
experience to 1.5. Any different experience falsifies F3 for that part. Our-engine comparison
with `quirks.zero_kill_value_caps` off: allied zero-value kills leave experience unchanged,
while positive-value zero-divisor kills still reach the cap.

**X14: ability-list parsing (Import mapping and whitespace quirk).** Reuse X5's damage-100
weapon, veteran combat factor 2, human-owned map-placed veteran at experience 1, enemy high-health
target, versus 100 %, other damage factors 1 and no defence or healing. Use an empty elite list.
In separate fresh runs give the veteran list these exact texts: `firepower,sight`,
`firepower ,sight`, `sight, FIREPOWER`, `sight,FIREPOWER`, `firepower,,sight`, and
`firepower,not-an-ability`. Record health lost to one shot in each run. Expected: respectively
200, 100, 100, 200, 200 and 200. Any other loss falsifies the corresponding spacing,
case-insensitivity, empty-item or unknown-item rule. For layering, use a named test rules file
that gives the veteran list firepower, then a map override giving sight alone: expected loss
100 (replacement, not union). Repeat with that map value empty and then missing: expected
200 in both (retention). Any different loss falsifies the corresponding layering rule.
On our engine repeat the two whitespace variants with `quirks.list_items_untrimmed` off:
expected loss 200 for both. The off-state and warnings are design checks, not oracle measurements.

**Coverage outside numbered claims.** Import parsing and warning predictions are inherited from
the referenced import specs where applicable. Ability-list whitespace, case matching, empty and
unknown items, replacement and empty/missing retention are `measurable` (X14), confidence high.
The ability lists' specific text limit, same-tick sequential capping, out-of-scope team-created
experience values and negative-data outcomes beyond X8's negative map value are `unmeasured`
here: no part isolates those limits, ordering or inputs. Confidence: high for list parsing and
capping, medium for the bad-data descriptions and team-created values. Warnings and
out-of-scope AI policies are design rules, not oracle predictions. X1's wait tests absence of decay for its
2000-tick intervals, not for every possible duration.

## Open questions

- Which object a destruction that claim 9 does not list credits (erasure is only medium; other
  paths, such as occupants killed on their own, were not traced).
- Which game-mode setting makes skirmish starting units elite, and whether it can be reached.
- Whether the resource-proof ability has any effect in the original (one candidate rule was not
  checked).
