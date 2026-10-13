---
id: S-0403
title: Build time (progress steps, base time from cost, country and type factors, low power, several factories, walls)
topic: production
revision: 2
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: []
supersedes: []
---

# S-0403: Build time (progress steps, base time from cost, country and type factors, low power, several factories, walls)

## Summary

How long a production line takes to build one item: the 54 progress steps and their period, the
base time from the type's cost field and the game-wide build-speed field, the country's per-kind
build-time factor, the type's own build-time multiplier, the slowdown under low power, the speed-up
from several factory buildings of the same kind, the wall coefficient, and when the step period is
worked out again during a build. What a step costs, holds, cancels, queues and completion belong to
S-0404; the sidebar is S-0411. Skirmish only: network games and campaigns are out of scope.

## Terms

Glossary terms are used as defined there (*tick*, *tick number*, *house*, *country*, *type*,
*field*, *building*, *infantry*, *vehicle*, *aircraft*, *credits*, *game minute*, *difficulty row*,
*difficulty skill block*, *AI house*, *round-toward-zero mode*, *importer warning*, *quirk switch*,
*absent-key default*). From S-0205: RZ₅₃, RZ₂₄, trunc and the conversion rule (Formulas there),
with "single" and "double" as in S-0003. From S-0203: *production line* (Terms), *turn pass*. In
addition:

- **Item.** The one object a production line is building at a time. Its *type* is the type ordered.
  S-0404 owns how an item starts, is paid for, held, cancelled and completed.
- **Line kind.** Which of a house's six production lines builds a type (S-0404, claim 1): infantry,
  ground vehicles, naval vehicles, aircraft, buildings, defences. A building type is a *defence* when
  its build category field is the combat category; every other building type is an *ordinary
  building*. A vehicle type is *naval* when its naval flag is set.
- **Progress.** An integer from 0 to 54 per line: the number of progress steps the item has made.
  The item is *done* at 54.
- **Step period** *R*. The number of ticks between two progress steps of a line, an integer from 1 to
  255 (claim 1).
- **Build time** *T*. The integer, in ticks, that claims 2 to 7 compute for an item at a given
  moment. The step period is derived from it (claim 1).
- **Factory building of a kind.** A building whose type's factory field names the kind of object it
  makes: infantry, vehicles, aircraft or buildings. A factory building of vehicles is *naval* when its
  own type's naval flag is set. S-0003 claim 21 and Import mapping row C19 own the factory
  vocabulary, text limit and fallback. Which factory building releases a finished unit is S-0404's
  (claim 7).
- **Power fraction** *p*. As S-0405 defines it (its Terms and F3). S-0405 owns output, drain and the
  fraction; this spec only narrows it to single precision (claim 5).
- **Power re-evaluation.** The moment a house works out its power output and drain again. In the
  original it happens in the house's own update (S-0203, claim 8) of every tick in which something
  has made the house's power due (S-0405, claim 7; here claim 8).

## Behaviour

1. **Fifty-four steps, one every *R* ticks.** A line builds its item in 54 progress steps. The step
   period is *R* = min(255, max(1, *T* ÷ 54)), where *T* is the build time of claims 2 to 7 at the
   moment *R* is worked out and ÷ is the integer quotient. While the line runs and money suffices
   (S-0404, claim 4), a step happens every *R* ticks: the first one *R* ticks after the line starts
   or resumes, in the line's update (S-0203, claim 8). So an item whose line is started by an order
   issued in tick *n*, with *R* unchanged, enough money and no hold, makes its 54th step, and is
   done, in tick *n* + 54*R*. An uninterrupted build therefore lasts 54*R* ticks: *T* rounded down to
   a multiple of 54, at least 54 ticks and at most 54 × 255 = 13770 ticks.
   - Backing: `measurable` (scenario B1)
   - Confidence: high (the 54 steps, the period formula and its limits); medium (the tick of the
     first step relative to the order, which rests on S-0213 claim 1 and S-0203 claim 7)

2. **Base time from the cost field.** The base time is

   > *T*₀ = trunc(RZ₅₃(RZ₅₃(*C* × *S*) × *k*₀))

   where *C* is the type's cost field in credits (an integer, as written in the rules data, not the
   price the house pays), *S* is the game-wide build-speed field (a double, in game minutes per 1000
   credits) and *k*₀ is the double nearest to 0.9, whatever the rounding mode: the engine's 900 ticks
   per game minute divided by 1000, because *S* counts game minutes per 1000 credits, so that *C* × *S* is in thousandths of a
   game minute. The price a house pays (S-0404, claim 2), and therefore the country cost factors and
   the cost bonuses of buildings that lower prices, have no effect on build time.
   - Backing: `measurable` (scenario B2)
   - Confidence: high

3. **The country's per-kind build-time factor.** The country of the item's owner has one build-time
   factor for each of five kinds: infantry, vehicles, aircraft, ordinary buildings and defences.
   Ground and naval vehicles share the vehicles factor. With *c* that factor (a single):

   > *T*₁ = trunc(RZ₅₃(*c* × *T*₀))

   - Backing: `measurable` (scenario B3)
   - Confidence: high

4. **The type's build-time multiplier.** With *m* the type's own build-time multiplier (a single):

   > *T*₂ = trunc(RZ₅₃(*T*₁ × *m*))

   - Backing: `measurable` (scenario B3)
   - Confidence: high

5. **Low power.** Let *p* be the owner's power fraction (S-0405, F3) and *p*ₛ its value narrowed to
   a single, and *q*, *f*ₘᵢₙ and *f*ₘₐₓ the game-wide low-power penalty, minimum production speed
   and maximum production speed (all singles from the rules data). The production speed factor *f*
   and the time are

   > *p*ₛ = RZ₂₄(*p*) with `quirks.chop_rounding` on (the original), RN₂₄(*p*) with it off: *p* narrowed
   > to a single by S-0205's narrowing rule (claim 5 there). *p*ₛ is 1 exactly when *p* is 1 and 0
   > exactly when *p* is 0.
   >
   > *f*₁ = RZ₅₃(1 − RZ₅₃(RZ₅₃(1 − *p*ₛ) × *q*)), the unlimited speed.
   >
   > *f*₂ = min(max(*f*₁, *f*ₘᵢₙ), *f*ₘₐₓ) when *p*ₛ < 1, otherwise max(*f*₁, *f*ₘᵢₙ).
   >
   > *f* = *f*₂ when *f*₂ ≠ 0, and the single nearest to 0.01 (0.009999999776482582) when *f*₂ = 0.
   >
   > *T*₃ = trunc(RZ₅₃(*T*₂ ÷ *f*))

   The lower limit holds at every power level and the upper limit only under a deficit, so with
   *f*ₘᵢₙ above *f*ₘₐₓ a deficit gives *f*ₘₐₓ. With full power *f* is 1 (unless *f*ₘᵢₙ is above 1).
   Under any deficit, with the absent-key defaults (*q* = 1, *f*ₘᵢₙ = 0.5, *f*ₘₐₓ = the single nearest to 0.9), *f* is the power fraction held
   between 0.5 and about 0.9: the smallest deficit already makes a build about 11 % slower, and no
   deficit makes it more than twice as slow. Which buildings stop under low power, and power itself,
   are S-0405's.
   - Backing: `measurable` (scenario B4 for the formula and the order of the limits; B4 part c for
     the narrowing, which changes *R* only for rare power values, so part c uses values found by an
     offline search of F4)
   - Confidence: high (formula and order of the limits); medium (the single-precision narrowing of
     *p*, with S-0405 owning *p* itself; with `quirks.chop_rounding` off, that the narrowing rounds to
     nearest is S-0205's rule and is not measured here)

6. **Several factory buildings.** Let *n* be the number of factory buildings of the item's kind that
   the house owns on the map, with naval and ground vehicle factories counted apart: an infantry item
   counts the house's infantry factories, a ground-vehicle item its non-naval vehicle factories, a
   naval item its naval vehicle factories, an aircraft item its aircraft factories, and both ordinary
   buildings and defences count the house's building factories. Every such building counts from the
   moment it is placed (also while it plays its build-up) until it leaves the map, also when it is
   unpowered, being sold, or of a type whose owners cannot build this item (quirk below). With *g* the
   game-wide multiple-factory factor (a single), when *g* > 0 and *n* ≥ 2 the time is multiplied by *g*
   once for each factory beyond the first, truncating after each multiplication:

   > *T*₄ = *T*₃ when *g* ≤ 0 or *n* ≤ 1; otherwise *T*₄ = *u*ₙ₋₁ with *u*₀ = *T*₃ and
   > *u*ᵢ = trunc(RZ₅₃(*u*ᵢ₋₁ × *g*)) for *i* = 1 … *n* − 1.

   The count changes the time only when the step period is worked out (claim 8).
   - Backing: `measurable` (scenario B5)
   - Confidence: high (the formula and the per-kind counting); medium (that a factory still in
     build-up and one of another side count)

7. **Walls.** For a building type with the wall flag, the time is multiplied by the game-wide wall
   build-speed coefficient *w* (a double) after all other factors:

   > *T* = trunc(RZ₅₃(*T*₄ × *w*)) for a wall, *T* = *T*₄ otherwise.

   - Backing: `measurable` (scenario B6)
   - Confidence: high

8. **When the step period is worked out.** *R* is worked out from the item's current build time:
   - when the item starts, and when the line resumes after a hold (S-0404, claims 3 and 5); the
     next step then comes a full *R* ticks later, so the part of a step that had passed before the
     hold is lost, and
   - for every line of a house, at each power re-evaluation of that house. The new *R* applies from
     the next step on: the step already under way keeps the period it started with. In the original
     a power re-evaluation is due when S-0405 (claim 7) says so; this spec adds no event of its own.
     A change of the power fraction or of the factory count of claim 6 therefore reaches the step
     period in the first house update that S-0405 (claim 7) gives it, with that claim's timing for
     health changes and for changes made by orders. The house update comes after the production
     lines of its tick (S-0203, claim 8), so the change first shortens or lengthens a step that
     starts in a later tick than the house update that sees it.

   Nothing else changes *R* during a build.
   - Backing: `measurable` (scenario B7)
   - Confidence: high (start, resume and the refresh at re-evaluation; the step under way keeps its
     period); medium (the timing of the re-evaluation, which S-0405 owns and this claim only cites)

9. **Factors with no effect on build time.** The build-time factor of the difficulty rows, the
   country's general build-time factor and the game-speed bias factor are read from the rules data
   but no build time uses them. Game speed itself does not change *T* or *R* in ticks (glossary,
   claim G3). Veterancy, crates and superweapons have no effect on build time. So in our engine the
   optional per-player handicap setting (*handicap*, S-0412, claim 5) does not change build
   times, and no AI house builds faster or slower than a human house.
   - Backing: `measurable` (scenario B8) for the difficulty-row and country factors and game speed, and
     `unmeasured` for "no other factor" as a whole (a statement about every rule; each other topic
     that could change build time would state it)
   - Confidence: medium (each house takes these factors when the houses are set up, and
     nothing in the original was found to use them)

10. **Identical rules for every house.** Claims 1 to 9 apply to every house alike: human-controlled
    houses, AI houses and the two non-player houses. No build-time rule of the original depends on
    whether a house is human-controlled.
    - Backing: `unmeasured` (the original's computer-controlled houses start items without orders,
      at moments a recorder cannot see, so their build times cannot be timed from the start; the
      claim rests on the formulas, none of which depends on who controls the house; for our engine
      it is a rule)
    - Confidence: high

## Formulas

All quantities in ticks are integers (S-0205, claim 1). The fields *C* (integer), *S*, *w* (doubles)
and *c*, *m*, *q*, *f*ₘᵢₙ, *f*ₘₐₓ, *g* (singles, widened exactly to doubles, S-0205 claim 4) are imported
(Import mapping). Every arithmetic result is RZ₅₃ with `quirks.chop_rounding` on, RN₅₃ with it off
(S-0205, Formulas); every trunc is S-0205's conversion of the common form. The original computes
F1 to F6 in floating point, at the places marked RZ; F7 is an integer quotient.

- **F1 (base).** *T*₀ = trunc(RZ₅₃(RZ₅₃(*C* × *S*) × *k*₀)), *k*₀ the double nearest 0.9.
- **F2 (country).** *T*₁ = trunc(RZ₅₃(*c* × *T*₀)).
- **F3 (type).** *T*₂ = trunc(RZ₅₃(*T*₁ × *m*)).
- **F4 (low power).** *p*ₛ, *f*₁, *f*₂, *f* and *T*₃ as in claim 5. *p*ₛ, the narrowed form of S-0405's *p* (F3),
  is the only value narrowed to single precision here; *f*₁ and *f*₂ stay doubles.
- **F5 (several factories).** *T*₄ as in claim 6.
- **F6 (walls).** *T* as in claim 7.
- **F7 (step period).** *R* = min(255, max(1, *T* ÷ 54)), integer quotient truncated toward zero; a
  negative *T* (possible only when some data is negative, for example a negative *c* or *m*, or
  data that makes *f* negative, Edge cases) gives *R* = 1.
- **Duration.** Without holds, money waits or refreshes, the item is done 54*R* ticks after the
  tick in which its line started.

Example (illustrative values, not stock data): *C* = 1000, *S* = 1.0, all factors 1 and full power
give *T*₀ = trunc(RZ₅₃(1000 × *k*₀)) = 900 (the product is just above or at 900), *R* = 16, a build
of 864 ticks; at half power with the absent-key defaults, *T*₃ = 1800, *R* = 33, 1782 ticks.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Progress steps per item | 54 | steps | claim 1 |
| Step period limits | 1 to 255 | ticks | claim 1 |
| Longest uninterrupted build | 13770 | ticks | 54 × 255 |
| *k*₀ | double nearest 0.9 | ticks per thousandth of a game minute | 900 ticks per game minute ÷ 1000; *C* × *S* is in thousandths of a game minute |
| Zero-speed replacement | single nearest 0.01 | dimensionless | claim 5 |
| Absent-key default, build speed *S* | 1.0 | game minutes per 1000 credits | |
| Absent-key default, *q* | 1.0 | dimensionless | |
| Absent-key default, *f*ₘᵢₙ | 0.5 | dimensionless | |
| Absent-key default, *f*ₘₐₓ | single nearest 0.9 | dimensionless | |
| Absent-key default, *g* | 1.0 | dimensionless | |
| Absent-key default, *w* | 0.5 | dimensionless | |
| Absent-key default, *c* (each kind) and *m* | 1.0 | dimensionless | |

## State and ordering

| Moment | What happens to *R* |
|---|---|
| Item starts (S-0404, claim 3) | *R* worked out; first step *R* ticks later |
| Line resumes after a hold (S-0404, claim 5) | *R* worked out; next step a full *R* ticks later |
| Power re-evaluation of the owner (house update) | *R* worked out again for each of the house's lines; the step under way keeps its period |
| Step made | the next step is due *R* ticks later, with the *R* in force at that moment |
| Step refused for lack of money (S-0404, claim 4) | the next attempt is due *R* ticks later |

Observable consequence of the order: a factory building placed in tick *t* speeds up a running line
from the step that starts after tick *t* at the earliest, never the step that falls in tick *t*.

## Edge cases and quirks

- **Factories that cannot build the item still count** (`quirks.multiple_factory_counts_all`,
  default on = original). Claim 6 counts every factory building of the kind: one still in build-up,
  an unpowered one, one being sold, and one whose type the item's owners cannot use (for example a
  captured factory of another side). With the switch off, only factory buildings that could make the
  item with power and dock occupancy ignored count (S-0404, claim 3's conditions without the power
  switch and the free dock: not being sold, of a type whose owner countries share at least one
  country with the item type's, of the matching naval or ground kind, and with the tech-tree rules
  allowing the type).
- **Step period cap** (`quirks.build_step_period_cap`, default on = original). *R* never exceeds 255,
  so no uninterrupted build lasts more than 13770 ticks, however high *T*. With the switch off, *R*
  = max(1, *T* ÷ 54) without the upper limit.
- **Resume loses the partial step** (`quirks.resume_restarts_step`, default on = original). Claim 8: a
  resumed line waits a full *R* before its next step. With the switch off, the line keeps the ticks
  that remained of the interrupted step.
- **Rounding to whole steps.** *T* below 108 gives *R* = 1 (54 ticks), so a *T* below 54 is
  lengthened to 54 ticks; within the limits of claim 1, *T* is rounded down to a multiple of 54. This
  is the original's design and has no switch.
- **Data that makes the speed factor zero or negative.** A zero *f*₂ uses the 0.01 replacement
  (claim 5). *f* is negative only under a deficit, and then only when *f*ₘₐₓ is negative, or when
  *f*ₘᵢₙ is negative and the deficit is large enough to make *f*₁ negative, which needs a *q* above
  1; *T*₃ is then negative and *R* = 1. A negative *q* on its own gives *f*₁ ≥ 1 and never makes *f*
  negative. The importer warns for a negative *q*, *f*ₘᵢₙ, *f*ₘₐₓ, *g* or *w*, and for
  *f*ₘᵢₙ > *f*ₘₐₓ.
- **Large products.** *C* × *S* beyond the integer range converts by S-0205 claim 7 (wrap, or 0 at
  2⁶³ and above); no stock data comes near it.

## Import mapping

| Key (file, section) | Concept | Form | Absent |
|---|---|---|---|
| `Cost` (rules, type) | cost field *C* | integer, credits | 0 |
| `BuildSpeed` (rules, `General`) | build speed *S* | decimal (S-0003, claims 6 and 7), held as a double, game minutes per 1000 credits | 1.0 |
| `BuildTimeInfantryMult`, `BuildTimeUnitsMult`, `BuildTimeAircraftMult`, `BuildTimeBuildingsMult`, `BuildTimeDefensesMult` (rules, country) | country factor *c* for infantry, vehicles, aircraft, ordinary buildings, defences | decimal, held as a single | 1.0 |
| `BuildTimeMultiplier` (rules, type) | type multiplier *m* | decimal, held as a single | 1.0 |
| `BuildCat` (rules, building type) | the combat category makes the type a defence | category name | not combat |
| `Naval` (rules, vehicle type; rules, building type) | naval item; naval factory building | boolean | no |
| `Factory` (rules, building type) | factory building of a kind | named value: producing vocabulary, 31-byte text limit, whole-text ASCII-insensitive comparison and fallback as S-0003 claim 21 and Import mapping row C19; not a registered-type reference | initially no factory; missing or empty retains the value in force; unmatched non-empty text gives no factory, with an importer warning |
| `LowPowerPenaltyModifier` (rules, `General`) | *q* | decimal, held as a single | 1.0 |
| `MinLowPowerProductionSpeed`, `MaxLowPowerProductionSpeed` (rules, `General`) | *f*ₘᵢₙ, *f*ₘₐₓ | decimal, held as a single | 0.5; single nearest 0.9 |
| `MultipleFactory` (rules, `General`) | *g* | decimal, held as a single | 1.0 |
| `WallBuildSpeedCoefficient` (rules, `General`) | *w* | decimal, held as a double | 0.5 |
| `Wall` (rules, building type) | wall flag | boolean | no |
| `BuildTime` (rules, difficulty-row sections `Easy`, `Normal`, `Difficult`; rules, country) | no effect (claim 9) | decimal | — |
| `GameSpeedBias` (rules, `General`) | no effect on build time (claim 9) | decimal | — |

## Measurement scenarios

All scenarios use the measurement setup (glossary), a small purpose-made skirmish map with one human
player and one bystander computer player (glossary), game speed free (claim 9 says it does not
matter, scenario B8 checks it), and named test rules files in the map's override sections. The
observed area is the human player's base: its factory buildings, power plants and the buildings it
places. Each case is run once, since no formula of this spec draws a random number; a run that the
bystander's void rule voids is repeated. "Done tick" is the tick in which
the item becomes done: for a building, the tick its build icon first shows the ready state; for a
unit, the tick before the one at the end of which it leaves its factory building, because the
interface's automatic place order for a finished unit is issued in the next tick and applied at the
end of it (S-0411, claim 9, whose scenario S7 measures that one-tick delay). A unit's done tick is
thus inferred from its leaving tick, with a hold or a blocked exit excluded from every scenario
that infers a done tick this way. Every item whose done tick is observed is ordered by scripted
input at a known tick *n*, as in B1, and an expected time *T* stands for the done tick *n* + 54*R*
with *R* from F7. Credits are kept far above every price so that money never stops a line. Ticks are read from
the tick counter of the measurement setup's recorder or from drawn frames (one per tick in a
single-player game).

**B1. Step count and period.**
- Setup: a test building type and a test infantry type with *C* chosen so that *T* is 5400, 5401,
  5453, 100, 54 and 20000 ticks (*S* = 1.0, all factors 1, full power, one factory each), ordered by
  scripted input at a known tick *n*.
- Observable: done tick; the image frame of the build icon's clock at each drawn frame (S-0411,
  claim 8).
- Expected: done at *n* + 54*R* with *R* = 100, 100, 100, 1, 1 and 255; the clock's image frame changes every
  *R* ticks, 54 times; the unit leaves its factory at the end of the tick after its done tick (S-0411, claim 9), so
  at the end of tick *n* + 54*R* + 1.
- Falsified if: any done tick differs, the clock changes at another period, or the 20000 case is not
  capped at 13770 ticks.

**B2. Base time and price independence.**
- Setup: test types with *C* = 999, 1000, 1001 and *S* = 0.7, 1.0, 1.3 (nine runs), all factors 1,
  then the *C* = 1000 type with a country cost factor of 0.5 and with a test building (a kind of
  price-lowering building) whose cost bonus for the kind is 0.5.
- Observable: done ticks.
- Expected: 54 × (F1 ÷ 54) per case; the price changes leave the done tick unchanged.
- Falsified if: any case differs from F1 and F7, or a price change moves the done tick.

**B3. Country and type factors.**
- Setup: country factors 1.5 for infantry, 1.25 for vehicles, 0.625 for aircraft, 1.125 for
  ordinary buildings and 0.75 for defences, a type multiplier of 1.3 on one test vehicle, by test
  rules file.
- Observable: done ticks of an infantry item, an aircraft, a defence, an ordinary building, a ground
  and a naval vehicle.
- Expected: F2 and F3 per kind, the naval and ground vehicles alike.
- Falsified if: a kind uses another kind's factor, or the defence uses the ordinary-building factor.

**B4. Low power.**
- Setup: test power plants and test consumers giving power fractions 1, 0.99, 0.75, 0.5, 0.3 and 0
  (output 0); runs with the absent-key defaults and with *q* = 0.5, *f*ₘᵢₙ = 0.2, *f*ₘₐₓ = 1.0 by test
  rules file. Power is set before the order and not changed during the build.
- Observable: done ticks of an item with *T*₂ = 5400.
- Expected: F4 then F7 for each case; with the defaults the 0.99 case is as slow as about 0.9 speed,
  and the 0.3 and 0 cases as slow as 0.5 speed.
- Falsified if: any done tick differs from F4 and F7, or the 0.99 case runs at full speed.
- Part c (the narrowing of *p*): the absent-key defaults, `quirks.chop_rounding` on, a power
  output of 2814 and a drain of 4217 (so *p* = RZ₅₃(2814 ÷ 4217)), and an item with *C* = 8208,
  *S* = 1.0 and all factors 1, so that *T*₂ = 7387. Observable: the step period (the tick of every
  step). Expected: *T*₃ = 11070 and *R* = 205, whereas without the narrowing *T*₃ = 11069 and
  *R* = 204. Falsified if: *R* = 204 (no narrowing, or one that rounds to nearest). The values come
  from an offline search of F4 for a case in which the two give different step periods; if the test
  rules change them, the search is repeated.

**B5. Several factories.**
- Setup: *g* = 0.8 by test rules file; a ground-vehicle item with *T*₃ = 5400 built with 1, 2, 3 and
  4 non-naval vehicle factories; a naval item with 1 naval and 3 non-naval factories; a run where the
  second factory is a captured factory of another side; a run where the second factory is still in
  build-up when the item starts; a run with *g* = 0.
- Observable: done ticks.
- Expected: F5 (5400, 4320, 3456, 2764 before F7); the naval item at the one-factory time; the
  captured and the in-build-up factory counted; *g* = 0 at the one-factory time.
- Falsified if: any time differs, naval and ground factories mix, or the captured or the in-build-up
  factory is not counted.

**B6. Walls.**
- Setup: a test wall type and an identical non-wall building type, *w* = 0.5 and *w* = 0.3.
- Observable: done ticks.
- Expected: F6 for the wall, the plain time for the other.
- Falsified if: the coefficient is missing for the wall or applied to the other type.

**B7. When the period is worked out.**
- Setup: an item with *R* = 200 at full power; (a) a power plant leaves the map (sold or destroyed), giving a fraction of 0.5,
  in a tick known from the recorder while the line is between steps; (b) a second factory is placed
  in the middle of a step; (c) the line is held for 50 ticks in the middle of a step and resumed.
- Observable: tick of every step (changes of the clock's image frame).
- Expected: (a) and (b): the step under way ends at its original time, the following steps use the
  new period; (c) the next step comes a full 200 ticks after the resume.
- Falsified if: the step under way changes length, the new period starts later than the next step,
  or the resumed line keeps the partial step.

**B8. Factors with no effect.**
- Setup: test rules file setting the normal difficulty row's build-time factor to 2.0, the country's
  general build-time factor to 2.0 and the game-speed bias to 2.0; runs at game speeds 1 and 4.
- Observable: done tick of an item with known *T*.
- Expected: unchanged from the run without the test file, at both speeds.
- Falsified if: any of the three factors or the game speed changes the done tick in ticks.

## Open questions

1. Whether any other rule outside this topic adds to the build time (claim 9) is left to the specs of
   those topics.
