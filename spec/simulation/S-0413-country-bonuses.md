---
id: S-0413
title: Country bonuses (what each country field does, which spec owns it, and the fields with no effect)
topic: simulation
revision: 2
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: []
supersedes: []
---

# S-0413: Country bonuses (what each country field does, which spec owns it, and the fields with no effect)

## Summary

This spec lists every field of a country's rules section that the original reads, says what each does in play and which spec owns the rule. It owns the leftovers: the factors that are read but change nothing, the aircraft speed factor, the country's colour field, the wall-owner rewrite at game start, and the rule that every house of a country gets the same advantages whoever plays it. It names no stock country and says nothing about which country has which advantage: that is stock data.

## Terms

Glossary terms are used as defined there, in particular *country*, *side*, *house*, *type*, *type
kind*, *field*, *rules layer*, *absent-key default*, *AI house*, *difficulty row*, *normal row*,
*veterancy rank*, *departure*, *importer warning*, *quirk switch* and *game start*. S-0009's terms
*passive house*, *neutral house*, *special house*, *base unit* and *owner list* (the countries that
own a type, S-0009 claim 45) are used as S-0009 defines them, and so is S-0207's *progress unit* (the
unit in which a vehicle's progress along a track, and so its speed, is counted; it has nothing to do
with the glossary's build *progress*). In addition:

- **Country field.** One setting in a country's own rules section.
- **Factor kind.** For the per-kind country factors, every infantry, vehicle, aircraft or
  building type (and so every object of it) belongs to one of five kinds: infantry, vehicles (ground and naval alike), aircraft,
  ordinary buildings and defences. A building type is a defence when its build category says so
  (S-0403, Terms; S-0403 claim 3 and S-0404 claim 2 use the same split).
- **Per-kind factor.** A country field that holds one factor for one kind, in single precision.
- **General factor.** A country field that holds one factor for every kind, in double precision.
  The original has seven: firepower, rate of fire, armour, cost, build time, ground speed and air
  speed.
- **Country colour field.** The colour scheme that a country's section names.
- **Wall-owner flag.** The country flag of S-0210 (claim 14) that lets a house of the country own
  walls the map places.

## Behaviour

1. **Every country field and its owner.** The original reads the fields below from a country's
   section, each by the layering rules of S-0006. A field that a rules layer omits keeps the value
   in force (S-0006 claim 3); the absent-key default (Import mapping, last column, and Constants)
   applies only when no layer gives it. Two fields are exceptions. The parent-country field
   returns to the country's own name in every layer that has the country's section and omits it.
   The name prefix is read as given: a prefix key with an empty value gives an empty prefix, where
   the glossary's rule for a key with an empty value would keep the value in force (claim 4).

   | Field (by role) | Effect in play | Owner |
   |---|---|---|
   | side | the side the country belongs to | S-0004 claim 9; S-0005 claim 13 |
   | firepower (general factor) | multiplies the house's firepower, together with the difficulty row's firepower factor | S-0302 claim 3 |
   | rate of fire (general factor) | multiplies the house's rate-of-fire factor, together with the difficulty row's | S-0301 claim 3 |
   | armour, one per kind | divides the damage that objects of that kind take | S-0302 claim 5 (and its Formulas F2) |
   | cost, one per kind | multiplies the price of items of that kind | S-0404 claim 2 (draft of this batch) |
   | build time, one per kind | multiplies the build time of items of that kind | S-0403 claim 3 (draft of this batch) |
   | income | multiplies each harvest payment made into the balance (S-0402 claim 12); in the available-credits sum it also multiplies the value of resources held in house storage, which is always 0 in a skirmish (S-0404 claim 4) | S-0402 claim 12 and S-0404 claim 4 (drafts of this batch) |
   | speed, for vehicles | multiplies a vehicle's speed | S-0207 claim 2 |
   | speed, for infantry | multiplies an infantry unit's walking speed | S-0208 claim 7 |
   | speed, for aircraft | multiplies an aircraft's speed | claim 3 |
   | veteran lists for infantry, vehicles and aircraft | types created at the veteran rank | S-0311 claim 11 |
   | passive for multiplayer | the house takes no start position, no starting units, never loses (S-0414) | S-0009 (Terms, claims 19, 28, 33, 45); S-0414 claim 1 |
   | wall owner | may own the map's walls | S-0210 claim 14; claim 6 |
   | multiplayer listing flag | filters explicit countries offered by the standard skirmish lobby; also decides whether a house of the country appears in the in-game player lists and the score screen | claim 4; S-0412 claim 15 |
   | colour | presentation only | claim 5 |
   | armour, cost, build time, ground speed, air speed (general factors) | none | claim 2 |
   | parent country, name prefix, name suffix, smart-computer flag | none known | claim 4 |

   - Backing: `measurable` for each field's effect, through the scenarios of the owning specs and
     scenarios C1 to C3 here; `unmeasured` for the completeness of the list (a claim that no other
     field of a country's section is read, which no scenario can show; a spec that finds another
     field adds it here)
   - Confidence: high for each effect as its owner gives it; medium that the list is complete

2. **General factors with no effect.** The general armour, cost, build-time, ground-speed and
   air-speed factors of a country change nothing in play: no damage, price, build time or speed
   depends on them. Only the general firepower and rate-of-fire factors act (claim 1). The house's
   difficulty row has the same five factors and they change nothing either (S-0412 claim 5; S-0302
   claim 3; S-0403 claim 9; S-0404 claim 2). The importer warns when a rules file gives one of the
   five country factors a value other than 1.0, since a mod author may expect an effect.
   - Backing: `measurable` (scenario C1; S-0302 scenario D19 part b already covers the general
     armour factor, S-0404 scenario P2 the cost factor, S-0403 scenario B8 the build-time factor)
   - Confidence: high for armour, cost and build time (their owners); medium for ground and air
     speed (every house gets its country's general speed factors however the game is set up, but
     no speed rule uses them: the original's speed rules, S-0207 and S-0208, use only the
     per-kind factors)

3. **The aircraft speed factor.** An aircraft's speed is multiplied by the speed factor for
   aircraft of its owner's country, in the same way as S-0207 (claim 2) multiplies a vehicle's
   speed by the vehicle factor, with the same factors, rounding and order (Formulas, *Aircraft
   speed*, and S-0207 Formulas F2). The formula fixes the result, not an order of work: under
   round-toward-zero (S-0205 claim 7), RZ₅₃(RZ₅₃(*V* × *κ*) × *b*) and RZ₅₃(RZ₅₃(*V* × *b*) × *κ*)
   can be different whole speeds after truncation when the object's speed multiplier *b* differs
   from 1.0, and the speed is the first of the two. With *b* = 1.0 the two are equal, and no known
   source gives an aircraft another *b* (backing below), so the grouping has no observable effect
   on aircraft unless such a source exists. The aircraft's flight rules own the rest (planned
   S-0501).
   Each object uses the factor of its own kind only: an infantry unit the infantry factor, a
   vehicle the vehicle factor, an aircraft the aircraft factor. A building has no speed.
   - Backing: `measurable` for the kind selection and the aircraft factor (scenario C2, which also
     runs the vehicle and infantry factors); `unmeasured` for the order of the country factor and
     the object's speed multiplier, with this reason: the only known source of a speed multiplier
     other than 1.0 is the speed power-up, which does not reach aircraft (S-0207 claim 13), and so,
     as far as is known, an aircraft's multiplier stays 1.0, so with *b* = 1 the two orders give the same whole
     speed and no test can tell them apart. The order is therefore a rule of the shared formula
     that would matter only if another source of *b* for aircraft exists (Open questions)
   - Confidence: high for the kind selection; medium that aircraft use the same formula as
     vehicles; low that every aircraft locomotion uses this speed (the flight rules have not been
     examined yet; planned S-0501); medium for the order (it is S-0207's order for vehicles)

4. **Lobby and display fields with no effect on game state.** The multiplayer listing flag is
   required for a country to be offered explicitly by the standard skirmish lobby (S-0412 claim
   15, which owns the full filtering rule). It also decides whether a house of the country is
   shown in the in-game player lists and the score screen. It does not limit which countries the
   lobby's random choice can pick with the original fixed-range rule (S-0412 claim 6). The parent-country field, the
   name prefix, the name suffix and the smart-computer flag are read, but no rule of play is known
   to use them. Our engine reads all five and gives them no effect on game state. The prefix is
   at most one character and the suffix at most three: from a longer value the field keeps its
   first one or three characters, with any whitespace at the ends of the kept characters removed,
   and the original gives no message. Our importer cuts the same way and gives an importer warning
   for a longer value. The parent-country field keeps its first 24 characters, also without a
   message. An empty prefix value gives an empty prefix (the second exception of claim 1), and
   an empty suffix value is ignored and keeps the value in force. The absent-key defaults are in
   the Import mapping.
   - Backing: `measurable` (S-0412 scenario K11 part a for explicit-country filtering; scenario
     C3 part a for what the listing flag does to the two displays); `unmeasured` for the absence of any effect on game state, for the listing flag as
     for the other four (a claim about the absence of any effect; each topic that could use them
     would state it; a comparison of game state between two runs of the original needs the same
     seed, which the measurement setup cannot set), and for the cutting of the prefix, the suffix
     and the parent-country field and the empty-prefix rule (a field with no observable effect
     cannot show what was kept)
   - Confidence: high for the listing flag's explicit-country filtering; medium for what it does
     to the displays; low that the listing flag
     and the other four have no effect on game state; medium for the cutting of the prefix and
     suffix, the 24-character cut and the empty-prefix rule

5. **The country's colour field.** The field decides no game state: house colours come from the
   lobby colour slots through S-0103's table (claim 10). In the original it colours only the text
   of the alliance messages (S-0414 claims 12 and 13), which take the colour field of the issuing
   house's country and read it as a colour definition position. Those messages are presentation.
   From the moment the player houses are created to the end of the game, a country's colour field
   holds:

   | Country | Colour field |
   |---|---|
   | played by at least one player house | the lobby colour slot number *s* (S-0103 claim 10; 0 to 7) of the last player house of that country in house order (S-0009 claim 19) |
   | played by no player house | the imported value, a colour definition position |

   Before the player houses are created, every country's colour field holds its imported value.
   Countries are built afresh at every game start (S-0004 claim 1), so nothing carries over to a
   later game. So an alliance message is drawn in the colour definition at position *s*, not at
   position *P*(*s*), the colour definition position that S-0103's table gives slot *s*, and it
   does not show the issuing player's colour; and every player house of one country issues its
   alliance messages in the same colour, the one that the last of them in house order decides.
   The colour is a defect of the original, so it is a quirk: `quirks.alliance_message_slot_colour`, default on (the original: the message
   is drawn in the colour definition at position *s*). Off, an alliance message is drawn in the
   issuing house's own colour (position *P*(*s*) of S-0103's table). Whether our engine keeps the
   switch or fixes the colour with no switch is Questions for the project lead, 2.
   - Backing: `measurable` (scenario C3, part b)
   - Confidence: high for the slot number that the field holds and its use by the alliance messages,
     medium for the resulting colour on screen

6. **Wall-owner flags rewritten at game start.** In a skirmish, the wall-owner flag of each
   country changes at game start as follows, and keeps that value for the rest of the game:

   | Country | Wall-owner flag from then on |
   |---|---|
   | played by at least one house of the game, not passive | on |
   | played by at least one house of the game, passive | off |
   | played by no house of the game | unchanged (the imported value) |

   Every house of the game counts as playing its country, the neutral and special houses
   included. The rewrite changes no wall's owner: a wall that the map places keeps the owner that
   S-0210 (claim 14 a) gives it, whatever the new flag says. Our engine does the same.
   - Backing: `unmeasured`: no later use of the flag in a skirmish is known, so nothing observable
     follows from it (the measurement setup records no country data). A spec that finds a later use
     gives the scenario.
   - Confidence: medium (the rewrite itself is clear; low that the neutral and special houses are
     included)

7. **Advantages through types.** Most country-specific advantages in the original come from types,
   not from country fields: a type's owner list, and its lists of required and forbidden countries,
   decide which countries' houses may build it (S-0407, claims 5 and 11), and a building
   type can carry a superweapon (planned S-0512) or give a free unit. The base unit a house receives
   is the first type of the base-unit list whose owner list includes its country (S-0009 claim 45).
   A unique unit's own abilities belong to the specs of those abilities.
   - Backing: `measurable` through the owning specs
   - Confidence: high

8. **The same advantages for every player.** Every rule in this spec and in the specs it points
   to applies to a house because of its country alone. In our engine every house, AI or human, uses
   the normal difficulty row unless its player sets the handicap (S-0412 claim 5), so an AI house
   of a country gets exactly what a human house of that country gets, and two player houses of one
   country differ in their firepower and rate-of-fire factors only through a handicap. The neutral
   and special houses keep the original's rule: their countries' per-kind factors and lists apply
   to them like to any house, but their firepower and rate-of-fire factors are exactly 1.0, with
   no row and no general country factor (S-0412 claim 5).
   - Backing: `unmeasured` (a rule of our engine, fixed by decision of the project lead; in the
     original the country part of every factor is also the same for human and computer houses, and
     only the difficulty row differs, S-0412 claim 4)
   - Confidence: high

## Formulas

**House factors at set-up.** When the houses are created, a player house's firepower factor and
its rate-of-fire factor are each the product of the difficulty row's factor and the country's
general factor (S-0412 claims 4 and 5; the neutral and special houses have exactly 1.0, claim 8).
The other five general factors act nowhere (claim 2):

> *h* = RZ₅₃(*r* × *g*)

with *r* the row's factor and *g* the country's general factor, both doubles and dimensionless
(S-0205; for the rate of fire, S-0301 Formulas F2; for firepower, S-0302 claim 3), and *h* the
double result, dimensionless. In our engine *r* is the normal row's factor, or the chosen row's
when the handicap is set. The per-kind factors are used directly by their owners, in single
precision.

**Aircraft speed** (claim 3). The symbols are those of S-0207 Formulas F2. *V* is the aircraft
type's speed, a whole number in progress units per tick (taken as S-0207 Formulas F1 takes it for
a vehicle; the aircraft's own conversion is the flight rules', planned S-0501, low confidence).
*κ* is the country's aircraft factor, a single-precision value widened to a double, dimensionless.
*b* is the object's speed multiplier, a double, dimensionless. *ν* is the veteran speed factor, a
double, dimensionless. *ρ* is the object's speed fraction, a double, dimensionless. All products are
formed in doubles under round-toward-zero (RZ₅₃), and trunc truncates toward zero:

> *s*₁ = trunc(RZ₅₃(RZ₅₃(*V* × *κ*) × *b*)), a whole number in progress units per tick
> *s*₂ = trunc(RZ₅₃(*s*₁ × *ν*)) for a veteran that has the faster ability, otherwise *s*₁
> *s* = trunc(RZ₅₃(*s*₂ × *ρ*)), the whole speed in progress units per tick

This is S-0207's rule for vehicles with the vehicle factor replaced by the aircraft factor; S-0207
Formulas F2 owns the order of the factors and the rounding (including its flag-carrier halving,
which is a vehicle rule).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| General factors, absent-key default | 1.0 | factor | claim 1 |
| Per-kind factors, absent-key default | 1.0 | factor | claim 1 |
| Veteran lists, absent-key default | empty | type list | S-0311 |
| Side, absent-key default | none | side | claim 1; S-0004 claim 9 |
| Listing flag and smart-computer flag, absent-key default | off | flag | claim 4 |
| Colour field, absent-key default | position 0 | colour definition position | claim 5 |
| Name prefix, absent-key default | the character A | string | claim 4 (no effect) |
| Name suffix, absent-key default | empty | string | claim 4 (no effect) |
| Prefix and suffix characters kept | 1 and 3 | characters | claim 4 |
| Parent-country characters kept | 24 | characters | claim 4; Import mapping |

## State and ordering

The country fields are read with the rules layers at every game start (S-0006). The house factors
built from them are fixed when the houses are created (S-0412 table "Game start") and do not change
during a game. The wall-owner rewrite of claim 6 leaves every wall that the map places with the
owner S-0210 (claim 14 a) gives it. Of several player houses of one country, the last in house order decides the
colour of the alliance messages of every house of that country (claim 5).

## Edge cases and quirks

One quirk switch is proposed here (the alliance-message colour, below); the other decision that
might add one is for the project lead (Questions for the project lead).

- **General factors with no effect** (claim 2). Our engine gives them no effect either, as the
  original does. This is a property of the rules' data, not a defect that a mod would turn off, so
  no switch is named. The importer warns when one of the five differs from 1.0. Whether a switch
  that gives them an effect is wanted is question 1.
- **The colour field shared by every house of a country** (claim 5). The field holds a lobby
  colour slot number, which the alliance messages read as a colour definition position, and the
  last player house of the country in house order decides it.
  Presentation only. Switch: `quirks.alliance_message_slot_colour`, default on (the original's
  colour); off, the message uses the issuing house's own colour. Whether the switch is kept is
  question 2.
- **Wall-owner rewrite** (claim 6). Ordinary behaviour, reproduced; not a quirk, because no later
  use of the flag is known.

## Import mapping

Country sections are the sections named in the rules' country list (S-0004). Parse forms follow
S-0003. Keys of the per-kind cost and build-time factors and of the income factor are in the import
mappings of S-0404, S-0403 and S-0402; the per-kind armour keys in S-0302; the veteran lists in
S-0311; the passive flag in S-0009; the wall-owner flag in S-0210.

Every key keeps the value in force when a layer omits it (S-0006), except `ParentCountry` (claim 1). `Prefix` differs in the other case: an empty value is not ignored (claim 1).
The last column is the absent-key default, the value when no layer gives the key.

| Key | Concept | Parse form | Absent-key default |
|---|---|---|---|
| `Side` | side (S-0004) | side name | no side (S-0004 claim 9) |
| `Firepower` | general firepower factor (S-0302 claim 3) | decimal, double | 1.0 |
| `ROF` | general rate-of-fire factor (S-0301 claim 3) | decimal, double | 1.0 |
| `Armor`, `Cost`, `BuildTime`, `Groundspeed`, `Airspeed` | general factors with no effect (claim 2); importer warning when not 1.0 | decimal, double | 1.0 |
| `SpeedUnitsMult` | vehicle speed factor (S-0207) | decimal, single | 1.0 |
| `SpeedInfantryMult` | infantry speed factor (S-0208) | decimal, single | 1.0 |
| `SpeedAircraftMult` | aircraft speed factor (claim 3) | decimal, single | 1.0 |
| `Multiplay` | multiplayer listing flag (claim 4) | boolean | off |
| `Color` | country colour field (claim 5) | name of a colour definition, matched without regard to letter case; an unknown name keeps the value in force | the colour definition at position 0 |
| `ParentCountry` | parent country, no effect (claim 4) | country name, at most 24 characters kept | the country's own name, also in every layer that has the country's section but omits the key |
| `Prefix` | name prefix, no effect (claim 4) | string; the first character is kept; an empty value gives an empty prefix | the character A |
| `Suffix` | name suffix, no effect (claim 4) | string; the first three characters are kept; an empty value is ignored | empty |
| `SmartAI` | smart-computer flag, no effect (claim 4) | boolean | off |

## Measurement scenarios

All scenarios run under the glossary's *measurement setup* in a skirmish on a purpose-made flat
map of clear ground, at any game speed (glossary G3: game speed does not reach game state), with a
test rules file whose values are invented test values; no stock value is needed. No expected
result depends on a random draw, so each variant runs once. Every variant is a separate run of the
whole game. The *recorder* reads each tick the positions of the test objects. "Cell *k*" and
"arrival" are as in S-0207 scenario D1 (the first tick at whose end the object is at or past the
centre of the *k*-th cell of its route), and the *interval* is the arrival tick at cell 20 minus
the arrival tick at cell 10.

**Test objects (C1 and C2).** The human house owns one test vehicle (S-0207's D1 setup: speed field
20, wheel class, clear ground at 100 %, a straight route of 30 cells), one test aircraft whose speed
field converts to a speed *V* in progress units per tick (the conversion of an aircraft's speed
field is the flight rules', planned S-0501; the test picks a field value that gives *V* of at
least 4, and *V* is known from that conversion once S-0501 exists) flying a straight route of 30 cells, and one test infantry unit on a straight route of
30 cells. Each object has its own route, starts at full health with no veterancy, and is ordered
to move at the same tick. In every variant, every country factor not named is 1.0.

**C1: general factors with no effect.** Runs: 2. Run 1: all factors 1.0 (baseline). Run 2: the
human's country general ground-speed and air-speed factors 2.0, with every per-kind factor 1.0.
Observable: the interval of the test vehicle, of the test aircraft and of the test infantry unit.
Expected: all three intervals equal their run-1 values. Falsified if any of the three intervals
differs between the runs.

**C2: the aircraft speed factor and the kind selection.** Runs: 4. Run 1: baseline. Run 2: aircraft
factor 0.5. Run 3: vehicle factor 0.5. Run 4: infantry factor 0.5. Observable: the interval of
each of the three test objects (*T*ₐ for the aircraft, *T*ᵥ for the vehicle, *T*ᵢ for the infantry
unit), per run.
- *Expected:* in run 2 the aircraft's speed is trunc(*V* / 2) (claim 3 Formulas with *κ* = 0.5 and
  *b* = 1), so its interval is *V* / trunc(*V* / 2) times its run-1 value, within 2 ticks (exactly
  2 times for an even *V*, and at most 2.5 times for an odd *V*, which is at least 5; each interval
  is read to the tick); the vehicle's and the
  infantry unit's intervals equal their run-1 values. In run 3 the vehicle's interval is 66 ± 1
  (S-0207 scenario D12 run 1); the aircraft's and the infantry unit's equal their run-1 values.
  In run 4 the infantry unit's interval is longer than in run 1 (it walks at the factor times its
  own speed, S-0208 claim 7), and the aircraft's and the vehicle's equal their run-1 values.
- *Falsified if:* in run 2 the aircraft's interval differs from *V* / trunc(*V* / 2) times its
  run-1 value by more than 2 ticks; or any object that is not of the halved kind has an interval that differs from
  its run-1 value in any run; or the infantry unit's interval does not grow in run 4.
- *Not covered:* the order of the factors for aircraft (claim 3: unmeasured, since *b* is 1).

**C3: listing flag and colour field.**
- *Part a.* Runs: 2, for one human and one computer player of different countries. Run 1
  (baseline): the listing flag of the computer's country on. Run 2: a named test map override
  turns the flag off when the map loads. Keep the flag on in the base rules in both runs, so
  the country can be chosen explicitly before loading (S-0412 claim 15). The seed differs between
  runs and is not fixed; nothing below depends on a random draw, because only the two displays
  are compared. Observable: whether the computer's house appears in the in-game player list, and
  whether it appears in the score screen, at the end of a game that the human ends by resigning
  at tick 300. Expected: in run 1 the computer's house is listed in both places; in run 2 it is in
  neither. Falsified if the house is missing from either place in run 1, or appears in either
  place in run 2. Not covered: whether the flag changes any game state (credits, objects, random
  streams, who wins). Claim 4 marks that unmeasured, because two runs of the original differ in
  seed, and with a computer player in the game also in play, whatever the flag says. A statistical
  comparison over many runs could only show a large effect.
- *Part b.* Runs: 1. A skirmish of three players with the allies-allowed and ally-change-allowed
  flags on (S-0414 claim 14). A test rules file gives the colour definitions at positions 1, 2
  and 10 (counted as S-0103's table counts positions) three clearly different colours. The human
  has lobby colour slot 0 and one computer player of the same country has slot 2 (the computer
  comes later in house order). S-0103's table gives *P*(0) = 1 and *P*(2) = 10, so the slot
  number 2 differs from both. The second computer player is of another country. During play the
  human makes peace with the second computer and then breaks the alliance. Observable: the colour
  of the two alliance messages. Expected: the colour definition at position 2, the computer
  player's slot number (S-0103 claim 10), which is neither the colour at position 1 (the human's
  displayed colour) nor the colour at position 10 (the computer's). Falsified if the messages show
  any colour other than the colour at position 2 (in particular the colour at position 1 or at
  position 10).

## Cross-spec hand-off

S-0412 claim 15 owns the standard lobby's explicit-country filtering; claim 4 here now includes
that use of the listing flag. Joint sign-off must verify these claims against the exact versions
handed off together. The original random-country rule still ignores the flag (S-0412 claim 6).

## Open questions

- Whether any rule of play uses the parent-country field, the name prefix or suffix, or the
  smart-computer flag, or uses the multiplayer listing flag beyond explicit-country filtering
  and the two displays (claim 4).
- Whether every aircraft locomotion uses the speed rule of claim 3.
- Whether any source gives an aircraft a speed multiplier other than 1.0 (claim 3); if one exists,
  the order of the two factors becomes measurable.
- Whether anything reads the wall-owner flag after game start in a skirmish (claim 6), and whether the rewrite includes the neutral and special houses.

## Questions for the project lead

1. **A switch for the general factors with no effect (claim 2)?** The original reads the general
   armour, cost, build-time, ground-speed and air-speed factors and uses none of them. Should our
   engine keep it that way with no such option (this draft's assumption), or add a `quirks.<name>`
   switch whose alternative gives them effect? The alternative would be new behaviour that needs
   its own design: which rule each factor feeds, in which precision and in which order of
   rounding.
2. **The alliance-message colour (claim 5).** The original draws the messages in the colour at the
   position given by the last same-country player's slot number, which is not that player's
   colour. This draft proposes `quirks.alliance_message_slot_colour` with the original's colour as
   its default, and the issuing house's own colour when it is off. Does the lead accept
   the quirk (a quirk's default is always the original behaviour), or should our engine fix the
   colour as a departure with no quirk (presentation only), or reproduce the original with no
   quirk?
