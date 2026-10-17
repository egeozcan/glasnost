---
id: S-0413
title: Country bonuses (what each country field does, which spec owns it, and the fields with no effect)
topic: simulation
revision: 5
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-17
measurements: []
requests: [Q-0478]
supersedes: []
---

# S-0413: Country bonuses (what each country field does, which spec owns it, and the fields with no effect)

## Summary

This spec catalogues the country fields read by the original and their rule owners for skirmish; network games and campaigns are out of scope. Map-owner colour inheritance is an import-only pointer, not gameplay in another mode. It owns the leftovers: the factors that are read but change nothing, the aircraft speed factor, the country's colour field, the wall-owner rewrite at game start, and the rule that every house of a country gets the same advantages whoever plays it. It names no stock country and says nothing about which country has which advantage: that is stock data.

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
   applies only when no layer gives it. The parent-country field is an exception:
   it returns to the country's own name in every layer that has the country's section and omits it
   (S-0006 claim 5). Empty values create no entries for any key (S-0001 claim 11).
   In particular, an empty name-prefix line behaves exactly as omission and keeps the held prefix
   (claim 4). A country section containing only such a line is absent (S-0001 claim 12), so
   it changes no country field and triggers no parent-country reset (S-0006 claims 2 and 3).

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

   - Backing: each row inherits the backing of its owning claim, including the `unmeasured`
     portions of claims 3, 4 and 6. Scenarios C1 to C3 cover only the portions they identify.
     Completeness is `unmeasured`: no finite scenario can show that no other country field is
     read; a spec that finds another field adds it here.
   - Confidence: each effect inherits its owner's confidence; medium that the list is complete

2. **General factors with no effect.** The general armour, cost, build-time, ground-speed and
   air-speed factors of a country change nothing in play: no damage, price, build time or speed
   depends on them. Only the general firepower and rate-of-fire factors act (claim 1). The house's
   difficulty row's armour, cost and build-time factors also change nothing (S-0412 claim 5,
   S-0302 claim 3, S-0403 claim 9 and S-0404 claim 2). The importer warns when a rules file gives one of the
   five country factors a value other than 1.0, since a mod author may expect an effect.
   - Backing: `measurable` (scenario C1; S-0302 scenario D19 part b already covers the general
     armour factor, S-0404 scenario P2 the cost factor, S-0403 scenario B8 the build-time factor).
     The importer warning is `unmeasured` in the original: it is our engine's design requirement,
     not an original message. An engine import check must warn for each of the five factors set
     to the invented value 2.0, and not warn for 1.0; either opposite result fails that check
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
   - Backing: `measurable` for kind selection and a qualitative aircraft slowdown (scenario C2).
     The exact aircraft formula, its speed-field conversion and its applicability to every
     aircraft locomotion are `unmeasured`: no validated flight conversion and integration rule
     yet connects the formula's speed to a tick-by-tick position prediction. C2 cannot establish
     the truncations or an exact interval ratio; planned S-0501 must supply that test.
     `unmeasured` for the order of the country factor and
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
   to use them. Our engine reads these four fields without a game-state effect. The listing flag
   retains its lobby and display effects, but does not change play after setup. The prefix is
   at most one character and the suffix at most three: from a longer value the field keeps its
   first one or three characters, with any whitespace at the ends of the kept characters removed,
   and the original gives no message. Our importer cuts the same way and gives an importer warning
   for a longer value. The parent-country field keeps its first 24 characters, also without a
   message. An empty value for either affix creates no entry and keeps the value in force, exactly as omission. With no earlier prefix value, the prefix remains the character A; with no earlier suffix value, the suffix remains empty. A section containing only empty-valued lines is absent and changes neither field. These are applications of S-0001 claims 11 and 12 and S-0006 claims 2 and 3, not key-specific lexical exceptions. The absent-key defaults are in the Import mapping.
   - Backing: `measurable` (S-0412 scenario K11 part a for explicit-country filtering; scenario
     C3 part a for what the listing flag does to the two displays); `unmeasured` for the absence of any effect on game state, for the listing flag as
     for the other four (a claim about the absence of any effect; each topic that could use them
     would state it; a comparison of game state between two runs of the original needs the same
     seed, which the measurement setup cannot set), and for the cutting of the prefix, the suffix
     and the parent-country field, the retained prefix and suffix values, and their defaults
     (a field with no observable effect cannot show what was kept). The lexical absence of empty
     entries and entryless sections inherits S-0001 claims 11 and 12's backing; its scenario L9d
     tests the name-prefix key without depending on a prefix display. C4 supplies an engine import
     acceptance check for the field consequences, not an oracle measurement. Importer warnings are also
     `unmeasured` in the original: they are design requirements of our engine, not original
     messages. An engine import check must warn for an invented two-character prefix or
     four-character suffix, retain only the stated characters, and not warn for a prefix or
     suffix within its limit; a wrong retained value or warning outcome fails that check
   - Confidence: high for the listing flag's explicit-country filtering; medium for what it does
     to the displays; low that the listing flag
     and the other four have no effect on game state; medium for the cutting of the prefix and
     suffix and the 24-character cut; high for empty-line omission and prefix retention

5. **The country's colour field.** In a skirmish the field decides no game state: player-house
   colours come from the lobby colour slots through S-0103 claim 10. Alliance messages
   (S-0414 claims 12 and 13) are drawn in the issuing house's own colour, position *P*(*s*)
   of that table, with no quirk switch for a presentation-only defect. For map inspection without
   skirmish reassignment, a house inherits its country's imported colour, subject to its own
   map-section override (S-0103 claim 14); that colour-import rule does not add houses to a skirmish.
   In the original skirmish, the country's field colours only alliance-message text: the original's
   bug is to use position *s* from the last same-country player in house order, rather than the
   issuing house's position *P*(*s*), so same-country players share the wrong message colour.
   From the moment the player houses are created to the end of a skirmish, the original's
   country colour field holds:

   | Country | Colour field |
   |---|---|
   | played by at least one player house | the lobby colour slot number *s* (S-0103 claim 10; 0 to 7) of the last player house of that country in house order (S-0009 claim 19) |
   | played by no player house | the imported value, a colour definition position |

   Before the player houses are created, every country's colour field holds its imported value.
   Countries are built afresh at every game start (S-0004 claim 1), so nothing carries over to a
   later game.
   - Backing: `measurable` (scenario C3 part b for the original message colour; S-0103 scenario
     PM for map-house inheritance and override). The corrected presentation is a design rule,
     `unmeasured` in the original; C3 part b also gives an engine acceptance check. The values
     held by an unplayed country's field and their lifetime are `unmeasured`: they supply no
     message observable in a skirmish, and C3 tests only the played-country result.
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
   - Backing: `measurable` for build eligibility through S-0407's scenarios for claims 5 and 11,
     and for the base-unit choice through S-0009's scenario for claim 45. The building-carried
     superweapon and free-unit examples are `unmeasured` here: this catalogue gives no scenario
     for their operation, which needs the owning ability or production spec.
   - Confidence: high for build eligibility and base-unit choice; medium for the two examples

8. **The same advantages for every player.** The country-derived factors and type lists in
   this catalogue do not depend on whether a person or an AI plays the house. In our engine every house, AI or human, uses
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

with *r* the row's factor and *g* the country's general factor, both doubles and dimensionless.
RZ₅₃ rounds the product toward zero to double precision, as in S-0205 claim 7.
*h* is the dimensionless double result. S-0301 Formulas F2 owns its rate-of-fire use and
S-0302 claim 3 its firepower use. In our engine *r* is the normal row's factor, or the chosen row's
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
owner S-0210 (claim 14 a) gives it. Alliance messages use the issuing house's own colour
(claim 5), independently of other houses of the same country.

## Edge cases and quirks

No quirk switch is proposed here. The general-factor question remains for the project lead
(Questions for the project lead).

- **General factors with no effect** (claim 2). Our engine gives them no effect either, as the
  original does. This is a property of the rules' data, not a defect that a mod would turn off, so
  no switch is named. The importer warns when one of the five differs from 1.0. Whether a switch
  that gives them an effect is wanted is question 1.
- **Alliance-message colour** (claim 5). Messages use the issuing house's own colour. This is
  a presentation-only correction with no switch.
- **Wall-owner rewrite** (claim 6). Ordinary behaviour, reproduced; not a quirk, because no later
  use of the flag is known.

## Import mapping

Country sections are the sections named in the rules' country list (S-0004). Parse forms follow
S-0003. Keys of the per-kind cost and build-time factors and of the income factor are in the import
mappings of S-0404, S-0403 and S-0402; the per-kind armour keys in S-0302; the veteran lists in
S-0311; the passive flag in S-0009; the wall-owner flag in S-0210.

Every country field keeps the value in force when a rules layer omits it (S-0006 claim 3), except
the parent-country field, which resets as claim 1 describes. Empty values create no entry for
any key (S-0001 claim 11). In particular, `Prefix=`, `Prefix=   ` and `Prefix= ; note`
keep the held prefix; alone they create no country section (S-0001 claim 12).
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
| `Prefix` | name prefix, no effect (claim 4) | string; the first character is kept; an empty value creates no entry and keeps the held prefix | the character A |
| `Suffix` | name suffix, no effect (claim 4) | string; the first three characters are kept; an empty value is ignored | empty |
| `SmartAI` | smart-computer flag, no effect (claim 4) | boolean | off |

## Measurement scenarios

All scenarios run under the glossary's *measurement setup* in a skirmish on a purpose-made flat
map of clear ground, at game speed index 2 and 800 × 600 resolution, with no engine add-on and no
mods (the named test files of a scenario excepted). Their changed values are invented
test values; no stock value is needed. No expected
result depends on a random draw, so each variant runs once. Every variant is a separate run of the
whole game. The *recorder* reads each tick the positions of the test objects. "Cell *k*" and
"arrival" are as in S-0207 scenario D1 (the first tick at whose end the object is at or past the
centre of the *k*-th cell of its route), and the *interval* is the arrival tick at cell 20 minus
the arrival tick at cell 10.

**Test objects (C1 and C2).** A named test map places three objects for the human house: a
vehicle using S-0207's D1 straight-route setup (invented speed field 20, wheel class, clear ground
at 100 %), a stock fixed-wing aircraft with a named map override giving its speed field the
invented value 20, and a stock walking infantry type with the same invented speed field.
Each has a separate straight route of 30 cells towards map north, starts at full health without
veterancy, and receives a move order at tick 100 through scripted input. Place the infantry at
the right standing position so its empty-cell entries do not need a random position choice
(S-0208 claim 3). Keep their routes free
of obstructions and combat. The aircraft's starting facing points along its route; its route
must allow takeoff before cell 10 and continued flight past cell 20 without a turn. No aircraft
speed conversion is assumed or needed for these comparisons. Record the selected stock type
names with the measurement. Every country factor not named is 1.0.
Include one bystander computer player under the glossary's void rule. The observed area is the
three routes. Void a run if an object turns, fights or is obstructed on its measured segment.
Record until each reaches cell 20 or tick 3000, whichever comes first. Failure of a baseline
object to reach cell 20 makes the fixture invalid, not evidence about its country factor.

**C1: general factors with no effect.** Runs: 2. Run 1: all factors 1.0 (baseline). Run 2: the
human's country general ground-speed and air-speed factors 2.0, with every per-kind factor 1.0.
Observable: the interval of the test vehicle, of the test aircraft and of the test infantry unit.
Expected: all three intervals equal their run-1 values. Falsified if any of the three intervals
differs between the runs, or a run-2 object fails to reach cell 20 by tick 3000.

**C2: the aircraft speed factor and the kind selection.** Runs: 4. Run 1: baseline. Run 2: aircraft
factor 0.5. Run 3: vehicle factor 0.5. Run 4: infantry factor 0.5. Observable: the interval of
each of the three test objects (*T*ₐ for the aircraft, *T*ᵥ for the vehicle, *T*ᵢ for the infantry
unit), per run.
- *Expected:* run 2 has a longer aircraft interval than run 1, while the vehicle and infantry
  intervals are unchanged. This tests only a qualitative slowdown, not an exact speed or interval
  ratio. In run 3 the vehicle interval is 66 ± 1 ticks (S-0207 scenario D12 run 1), while the
  aircraft and infantry intervals are unchanged. In run 4 the infantry interval grows
  (S-0208 claim 7), while the aircraft and vehicle intervals are unchanged.
- *Falsified if:* the aircraft interval does not grow in run 2, the vehicle interval is outside
  65 to 67 ticks in run 3, the infantry interval does not grow in run 4, or any unaffected kind's
  interval differs from its baseline. A changed object that has not reached cell 20 by tick
  3000 is recorded as stalled and fails the predicted continued movement.
- *Not covered:* the exact aircraft formula, truncation sites, grouping of factors or coverage
  of every aircraft locomotion (claim 3's `unmeasured` portions). There is no justified exact
  arrival-interval prediction until the flight rules supply conversion and integration.

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
  position 10). Replay the same setup in our engine: both messages must use position 1,
  the issuing human house's displayed colour. A match to position 2 or 10 falsifies the fixed
  presentation rule.

**C4: empty name-affix importer acceptance check (claim 4, Q-0478).**
This is an engine import test, not a measurement of the original: no known play display uses
these fields. Use invented country names and an earlier rules layer giving prefix Z and suffix XYZ. Apply a later layer in separate variants: a nonempty country section omitting both affixes; the same section with an empty prefix value, a three-space prefix value, or a prefix value consisting only of whitespace and a comment; and a section containing only each of those three empty prefix entries. Spell the entries as Import mapping specifies and terminate every line with a line feed. Inspect the imported country fields after layering. All seven variants must retain prefix Z and suffix XYZ; the last three must supply no section. Repeat without earlier affix entries: every variant must retain prefix A and an empty suffix. A cleared or replaced prefix, changed suffix, or retained entryless section fails the check. For an oracle test of the lexical part alone, use S-0001 scenario L9d, whose observable is a sidebar price, not a prefix field.

## Cross-spec hand-off

**Q-0478 correction.** S-0001 claims 11 and 12 already answered the lexical questions:
empty values have no key-specific exceptions, and a prefix-only empty section is absent.
S-0006 claims 2, 3 and 5 already give section absence, ordinary retention and the separate
parent-country reset. Claims 1 and 4 here and Import mapping now remove the contradictory
empty-prefix exception: empty prefix lines retain the held prefix. No part of Q-0478 remains
open; the lack of a gameplay observable limits backing, not the import rule.

**Presentation-policy replacement within S-0413.** Claim 5 and Edge cases apply the project
lead's 2026-10-14 policy for obvious presentation-only defects, also stated in the template:
use the fixed message colour without a switch. This explicitly replaces the previously approved
requirement for `quirks.alliance_message_slot_colour` and resolves its pending lead question.
The old original-default switch must not survive this hand-off. No gameplay rule changes.

S-0412 claim 15 owns the standard lobby's explicit-country filtering; claim 4 here now includes
that use of the listing flag. Joint sign-off must verify these claims against the exact versions
handed off together. The original random-country rule still ignores the flag (S-0412 claim 6).

## Open questions

- Whether any rule of play uses the parent-country field, the name prefix or suffix, or the
  smart-computer flag, or uses the multiplayer listing flag beyond explicit-country filtering
  and the two displays (claim 4).
- The aircraft speed-field conversion and flight integration needed for an exact tick-by-tick
  test of claim 3's formula; whether every aircraft locomotion uses that formula.
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
