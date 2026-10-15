---
id: S-0510
title: Spies and infiltration
topic: infantry
revision: 1
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0510: Spies and infiltration

## Summary

How a spy enters another house's building, how it is used up, which single effect the entry has,
chosen by the building's type, and what the entry raises on trigger tags. The consumers of the
effects are approved elsewhere and are only pointed to: the whole-map re-cover (S-0409 claim 15),
the timed blackout (S-0405 claims 5 to 7), stolen tech (S-0407 claim 6), the superweapon charge
reset (S-0512 claim 15) and veterans from infiltrated factories (S-0311 claim 11). This spec owns
the money theft and the infiltrated-factory flags. Disguise (taking, showing, detecting and losing
it) is S-0516's. Who may be ordered to infiltrate what is S-0214 (table I, row I24); the approach
to the building is S-0215 claim 28. Skirmish only; network games and campaigns are out of scope.

Coordinated corrections required at hand-off: S-0311 claim 11 and Terms must identify the flags
as the infiltrating house's, not the victim's (claim 5 here); S-0111 claim 9 and scenario OV6
must not expect infiltration to reveal pips (claim 8 here). S-0405 claim 6 and S-0409 claim 15
must describe spies of another house, including conditional allied entries, with the single-effect
precedence of claim 3 here. S-0407 claim 6 already agrees with E3's default conditions.

## Terms

Shared terms used here are defined in the glossary: *house*, *ally*, *local player*, *tick*,
*infantry*, *building*, *building type*, *target, destination*, *mission target*, *in play, on the
map, removed*, *credit balance*, *available credits*, *factory building*, *radar building*, *spy
satellite*, *tech-lab list*, *superweapon record*, *trigger, tag*, *entry raise*, *announcement*,
*experience*, *veterancy rank*, *quirk switch*. In addition:

- **Spy.** An infantry unit whose type has the spy flag (Import mapping). S-0214 uses the same
  flag for its order rules.
- **Infiltration.** A spy's entry into a building by claim 1, including the no-effect own-house case. The *infiltrating
  house* is the spy's owner at the moment of entry; the *victim house* is the building's owner at
  that moment.
- **Steal fraction *f*.** A game-wide decimal: the share of the victim's available credits that a
  theft takes (claim 6, Import mapping).
- **Infiltrated-vehicle-factory flag, infiltrated-infantry-factory flag.** Two yes/no states of a
  house, both clear at the start of a game. A house's flag is set when one of its spies infiltrates
  another house's factory building of that kind (claim 5). S-0311 claim 11 and S-0007 read them
  from the house that owns the object being created or shown.
- **Vehicle-like building.** As S-0214 (Terms).

## Behaviour

### Entry and consumption

1. **When a spy enters.** A spy enters a building in the update in which it arrives at a standing
   position (S-0208 claims 6 and 22 give the arrival and its tick) when all of these hold:
   - its current behaviour is *capture*, *area guard* or *patrol*,
   - the cell of that standing position holds a building (any cell of the building's foundation),
   - that building is the spy's destination or its target.

   Nothing else is tested at entry: not the building type's spyable flag, not alliance, not the
   building's health, construction or sale. Those gates belong to giving the order (S-0214 row
   I24) and to the approach (S-0215 claim 28). How a walker comes to stand in a cell of its
   destination building is S-0209 (claim 4, mission-target override). A spy whose type also has
   the engineer flag is handled as an engineer and never infiltrates (S-0509). A destination that
   is a vehicle-like building, reached at the building's own cell, is handled by a different entry
   rule that does not look at the spy flag (S-0509); no infiltration happens then.
   - Backing: `measurable` (scenarios I1 and I6) for ordinary *capture* arrival and engineer
     precedence; `unmeasured` for *area guard*, *patrol*, target-only arrival, the absence of
     extra entry gates and the vehicle-like exception: no scenario here establishes those
     arrival states independently of the order and approach gates
   - Confidence: high (*capture*, the destination-or-target test, the engineer precedence),
     medium (*area guard* and *patrol*, that nothing else is tested, the vehicle-like case)

2. **The spy is used up.** Every entry removes the spy in the same update, whatever the
   building and whether or not an effect results (claim 3, including the own-house case). The
   removal is not a death: no killing hit, no experience for anyone, no death sequence, death
   sound or survivor, and the spy's disguise (S-0516) plays no part in it. The spy's house loses
   the unit as it would lose any removed unit.
   - Backing: `measurable` (scenario I1) for consumption and death presentation.
     `unmeasured` for no experience, no killing hit and disguise independence: I1 has no
     potential experience recipient, killing-hit observable or controlled disguise comparison
   - Confidence: high

### Effect selection

3. **One effect, chosen by the building type.** If the infiltrating house is the victim house
   itself, the entry has no effect (the spy is still used up, claim 2). Otherwise the first row of
   this table that the building's *type* matches gives the effect, and no other row applies
   (`quirks.infiltration_single_effect`, on by default). E3's exclusions below express
   precedence in that default state, not additional requirements when the switch is off:

   | Row | Building type | Effect, at once, in the spy's update |
   |---|---|---|
   | E1 | has the radar flag | the victim's whole map is re-covered, unless the victim has an active spy satellite (S-0409 claim 15) |
   | E2 | base power output above 0 (S-0405) | a timed blackout of the victim, of the game-wide spy blackout length (S-0405 claim 6) |
   | E3 | on the tech-lab list, without the radar flag and with base power output 0 or below | the infiltrating house gains the stolen-tech flag chosen by the type's base-planning-side field (S-0407 claim 6) |
   | E4 | has a first superweapon | the victim's superweapon record for that superweapon type is reset (S-0512 claim 15); the type's second superweapon is never reset |
   | E5 | storage above 0 | money theft (claim 6) |
   | E6 | factory building of vehicles (naval yards included) | the infiltrating house's infiltrated-vehicle-factory flag is set (claim 5) |
   | E7 | factory building of infantry | the infiltrating house's infiltrated-infantry-factory flag is set (claim 5) |
   | — | none of the above (aircraft and building factories included) | nothing |

   E3's explicit exclusions are the same conditions as S-0407 claim 6: E1 and E2
   take precedence over membership of the tech-lab list. They do not grant stolen tech.

   Only the infiltrating house's identity is compared with the victim's: a spy that reaches an
   allied building has the full effect against its ally. Upgrades do not count for row E2: only
   the type's own base output is read. The rows read type fields, so a captured building is
   infiltrated by its current owner's enemies like any other.
   - Backing: `measurable` (scenarios I2 and I5) for row precedence, ordinary effects and
     conditional own/allied entry; `unmeasured` for upgrade exclusion and captured-building
     equivalence: these scenarios contain no controlled upgrade or completed-capture comparison
   - Confidence: high (the order and the single effect); medium (the allied case, which an order
     cannot reach directly: S-0214 row I24 asks for an enemy building)

4. **Timing of the pointed-to effects.** The effect happens in the spy's own update of the arrival
   tick. For row E2 that tick is the blackout's start tick *t*₀ of S-0405 claim 6. For row E3 the
   house's buildable set follows in the house update of the same tick (S-0407 claim 12). For row
   E4 the reset keeps a stopped clock stopped and a running clock running, as S-0512 claim 15
   states. Row E1 re-covers at once.
   - Backing: `measurable` (scenario I2; the pointed-to scenarios S-0405 W4, S-0407 T6, S-0409 V9
     part B and S-0512 X8 measure the effects themselves)
   - Confidence: high

### Effects owned here

5. **Infiltrated-factory flags.** Row E6 sets the infiltrating house's infiltrated-vehicle-factory
   flag and row E7 its infiltrated-infantry-factory flag. The flags belong to the infiltrator's
   house, not the victim's: after its spy enters an enemy barracks, the *infiltrating* house's own
   later infantry of trainable types are created as veterans (S-0311 claim 11), and its sidebar
   shows their alternate icons (S-0007). A flag, once set, stays set for the rest of the game: it
   is not cleared when the infiltrated building is destroyed, sold or captured, nor when the
   victim is defeated. Setting a flag that is already set changes nothing.
   - Backing: `measurable` (scenario I4)
   - Confidence: high

6. **Money theft.** Row E5 moves money from the victim to the infiltrating house. With *B* the
   victim's available credits at the moment of entry and *f* the steal fraction, the infiltrating
   house's credit balance rises by *s* (Formulas F1) and the victim pays *s* from its credit balance; a payment above the balance empties it, as
   S-0404 claim 4 describes for spending, and never makes it negative.
   Normally 0 ≤ *f* ≤ 1, so the victim loses exactly *s*. With *f* above 1 the victim loses its
   whole balance while the thief still gains the full *s*; with *f* below 0 the money flows the
   other way and the infiltrating house's balance can go below 0
   (`quirks.spy_theft_unbacked_gain`).
   - Backing: `measurable` (scenario I3)
   - Confidence: high (the formula and the truncation); medium (the out-of-range fractions)

7. **Repeat entries.** An infiltration leaves no mark on the building. Each later entry, by the
   same or another spy, applies its row again: another theft from the reduced balance, a blackout
   that replaces the running one (S-0405 claim 6), another reset, another re-cover. For two spies entering in the same tick, the final balances are those produced by applying the
   thefts serially in S-0203's object-update order: each theft uses the victim balance left by the
   preceding theft.
   - Backing: `measurable` (scenario I3, parts c and f) for repeated and same-tick theft.
     `unmeasured` for repeat blackout, reset and re-cover: no repeat scenario for those
     effects is supplied here
   - Confidence: high

8. **Nothing is revealed about the building.** An infiltration does not mark the building, or any
   object, as spied on by the infiltrating house. In particular it shows the infiltrating house
   neither the building's pip scale nor what the building is producing. In a skirmish nothing
   else marks an object as spied on either.
   - Backing: `measurable` (scenario I7) for the absence of infiltration displays.
     `unmeasured` for the universal absence of any other marking rule: a finite display test
     cannot establish that no other skirmish event ever marks an object
   - Confidence: medium (from the absence of any setting rule)

### Triggers

9. **Trigger raises.** An entry by claim 1 raises the building's tag with the entered-by code and
   the spy as causing object (an entry raise, S-0011 claim 18) before the effect, so the actions of
   that raise see the state before the effect: for example the victim's balance before a theft and
   the building's owner at the time. After the effect and before its removal, the spy's own tag
   is raised with the destroyed-by-anything code, with the spy as causing object (S-0011 claim 19).
   The entry raise also happens for an entry that has no effect (own house, or no matching row)
   and for the other entrants of claim 10.
   - Backing: `measurable` (scenario I8) for both raises and their order; `unmeasured` for what the
     either raise's actions see relative to the effect, the spy's continued presence during
     its own raise, and the conditional own-house raise: I8's visible actions distinguish
     raise order, not balance changes, presence or own-house reachability
   - Confidence: high

### Other entrants

10. **Infantry that reach a building the same way without being spies or engineers.** An infantry
    unit that is neither a spy nor an engineer and that meets the three conditions of claim 1 has
    no effect on the building. Its destination is cleared and it is sent a scatter request away
    from the building's centre, forced and ignoring its behaviour and destination (S-0208 claims 11
    and 13 decide whether and where it goes). This covers a type with only the infiltrate flag:
    S-0214 lets it be ordered into an enemy capturable building, but the entry does nothing.
    - Backing: `measurable` (scenario I6) for no effect and scattering.
      `unmeasured` for destination clearing and the forced/ignore-behaviour qualifications:
      I6 does not distinguish those from other scatter requests
    - Confidence: medium

### Presentation

11. **Announcements.** These are presentation and take no draw. When the local player's house is
    the infiltrating house or the victim house, and those houses differ, an *infiltrated* radar event (S-0410) is created at
    the building's cell; that kind never suppresses, so it is always created. The announcements
    requested are, by row:

    | Row | Local player is the victim | Local player is the infiltrator |
    |---|---|---|
    | E1 | radar sabotaged, unless the victim has an active spy satellite | building infiltrated, radar sabotaged (one line), unless the victim has an active spy satellite |
    | E2 | power sabotaged | building infiltrated, then enemy base powered down |
    | E3, E6, E7 | technology stolen | building infiltrated, then new technology acquired |
    | E4 | building infiltrated | building infiltrated |
    | E5 | cash stolen | building infiltrated, cash stolen (one line) |
    | none, or own house | nothing | nothing |

    The lines are announcement entries named in Import mapping. Playback follows S-0701.
    - Backing: `measurable` (scenario I9) for announcements and radar events.
      `unmeasured` for no random draw: no isolated presentation-only random-state comparison
      is supplied
    - Confidence: high

## Formulas

**F1. Theft amount.**

> *s* = trunc(RZ₅₃(*B* × *f*)) credits

*B* is an integer (the victim's available credits, credits). *f* is held in single precision
(Import mapping). The product is formed in floating point, which our engine reproduces as a double
under S-0205 claim 5 (rounded toward zero with `quirks.chop_rounding` on, to nearest with it off),
and the single-precision *f* widens exactly. trunc is the conversion of S-0205 claim 7, toward zero.
The infiltrating house's balance rises by *s*; the victim's balance falls by min(*s*, its balance)
for *s* ≥ 0, and rises by −*s* for *s* < 0.

Examples, with the original rounding: *B* = 10 001 and *f* = 0.5 give 5000. A named test file
giving `57%` stores *f* as the single just below 0.57, so *B* = 100 gives 56, not 57; `33%` with
*B* = 1000 gives 329 (330 with `quirks.chop_rounding` off).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Steal fraction, absent-key default | 0.5 | fraction of available credits | Import mapping |

## State and ordering

| State of the infiltrating house *H* | Event | Result |
|---|---|---|
| infiltrated-vehicle-factory flag clear | a spy of *H* infiltrates another house's vehicle factory building whose type matches no earlier row | flag set for the rest of the game (claim 5) |
| infiltrated-infantry-factory flag clear | the same with an infantry factory building | flag set for the rest of the game |
| either flag set | any building loss, capture, sale or defeat | flag stays set |

In the arrival tick, actions caused by the building's entry raise observe the building and house
before the infiltration effect. Actions caused by the spy's destroyed-by-anything raise observe
the state after that effect, while the spy is still present; by the end of the update, the spy has
been removed (claim 9, including its unmeasured qualification). The house updates of that tick
observe the infiltration's result (S-0203 claim 8).

## Edge cases and quirks

- `quirks.infiltration_single_effect` (YR default: on). A building type that matches several rows
  of claim 3 gives only the first. A radar building with a positive output re-covers and causes no
  blackout; a power plant on the tech-lab list blacks out and gives no stolen tech; a factory with
  storage is robbed and its flag is not set. Off: every matching row applies, in the table's
  order, in the same update, with the announcements of each row. In that off state E3
  requires only tech-lab-list membership; its radar and power exclusions are removed.
  Both switch-off alternatives here, and the importer warning below, are design rules,
  `unmeasured` because they are not behaviours of the original.
- `quirks.spy_theft_unbacked_gain` (YR default: on). The thief gains the computed *s* even when the
  victim pays less (*f* above 1 with *B* above 0) and even when *s* is negative. Off: the amount is
  clamped to 0 ≤ *s* ≤ the victim's credit balance before either balance changes, so money is
  neither created nor taken from the thief. Our importer warns when *f* is outside 0 … 1 (S-0003
  claim 18 already warns for a likely forgotten percent sign).
- **Own building.** A spy that reaches a building of its own house is used up for nothing (claims 2
  and 3). Whether a building's change of owner while a spy walks towards it clears the spy's
  target and destination was not traced (Open questions); if it does not, a spy whose target is
  captured by its own side's engineer is lost on arrival.
- **The spyable flag is not rechecked.** Claim 1 does not test the spyable flag. A spy that holds a
  non-spyable building as target or destination in one of the three behaviours (possible through
  computer behaviour of the original, S-0215 claim 28 row "original only") infiltrates it.
- **Engineer and spy flags together.** The engineer rules win (claim 1).
- **Spy satellite and row E1.** The row is still the one chosen: a victim with an active spy
  satellite loses nothing, the spy is used up, and neither side hears the radar lines.

## Import mapping

| Key | Meaning here | Form | Default when absent |
|---|---|---|---|
| Rules, infantry type section, `Agent` | spy flag (Terms; claim 1); also S-0214 | boolean (S-0003) | no |
| Rules, infantry type section, `Engineer` | engineer precedence (claim 1); S-0509 owns | boolean | no |
| Rules, infantry type section, `Infiltrate` | infiltrate flag: order rules of S-0214 only; not read at entry (claim 10) | boolean | no |
| Rules, building type section, `Spyable` | order rule of S-0214 row I24 only; not read at entry | boolean | no |
| Rules, building type section, `Radar` | row E1 | boolean | no |
| Rules, building type section, `Power` | row E2: base output, the positive part (S-0405 owns the key) | integer | 0 |
| Rules, `[AI]`, `BuildTech` | row E3, tech-lab list (S-0407 owns) | list of building type names | empty |
| Rules, building type section, `AIBasePlanningSide` | row E3 flag choice (S-0407 owns) | integer | −1 |
| Rules, building type section, `SuperWeapon` | row E4, first superweapon (S-0407, S-0512) | superweapon type name | none |
| Rules, building type section, `Storage` | row E5 test: storage above 0 | integer | 0 |
| Rules, building type section, `Factory` | rows E6 and E7: factory kind (S-0404 owns the vocabulary) | named value | none |
| Rules, `[General]`, `SpyMoneyStealPercent` | steal fraction *f* (claim 6, F1) | percentage field (S-0003 claim 7), kept in single precision; a later layer that omits it keeps the earlier value | 0.5 |
| Rules, `[General]`, `SpyPowerBlackout` | blackout length of row E2 (S-0405 owns) | integer, ticks | 0 |
| Announcement data, `EVA_RadarSabotaged` | "radar sabotaged" (claim 11) | announcement entry | no line |
| Announcement data, `EVA_BuildingInfRadarSabotaged` | "building infiltrated, radar sabotaged" | announcement entry | no line |
| Announcement data, `EVA_PowerSabotaged` | "power sabotaged" | announcement entry | no line |
| Announcement data, `EVA_BuildingInfiltrated` | "building infiltrated" | announcement entry | no line |
| Announcement data, `EVA_EnemyBasePoweredDown` | "enemy base powered down" | announcement entry | no line |
| Announcement data, `EVA_TechnologyStolen` | "technology stolen" | announcement entry | no line |
| Announcement data, `EVA_NewTechnologyAcquired` | "new technology acquired" | announcement entry | no line |
| Announcement data, `EVA_CashStolen` | "cash stolen" | announcement entry | no line |
| Announcement data, `EVA_BuildingInfCashStolen` | "building infiltrated, cash stolen" | announcement entry | no line |

The trainable and naval flags that S-0311 claim 11 reads with the factory flags are mapped there.

## Measurement scenarios

Common setup unless stated: the glossary's measurement setup, with all unrelated mods off; a tiny
test map; the person plays an Allied house that can train spies, and one enemy bystander computer
player. Named test data below are override sections of the
purpose-made map, removed after each run. Run each part once unless it calls for a repeat.
The observed area is the test buildings and their approaches; apply the glossary's bystander void rule. The enemy owns only pre-placed buildings, has no construction yard,
no harvester and no refinery, and a known starting balance (map setting), so its balance stays
known; where it owns factories, its starting balance is 0 so it cannot build. Spies are ordered
into the building with the ordinary infiltrate order. Game speed index 3, 800 × 600 pixels; record ticks and object positions with a read-only
recorder and capture video and audio. Requested recorder observations below are measurement
proposals, not assertions that an existing harness already exposes them.

- **I1 (claims 1, 2).** A test building type with none of the claim 3 properties and the spyable
  flag, owned by the enemy. Order a spy in. Record the tick at which the spy arrives in the
  building's cell (bench position trace) and the tick it disappears. Expected: it disappears in its
  arrival tick, with no death sequence, death sound or survivor, and the enemy and the building
  are unchanged. Second part: a building of the same type whose foundation is 3 × 3; the spy is
  ordered in from a side so that the first cell it reaches is an edge cell. Expected: it is used
  up on that edge cell. Falsified if the spy survives, dies visibly, or only enters at a centre
  cell.
- **I2 (claims 3, 4).** A named test file gives five enemy test building types, each spyable,
  with property pairs: (radar, positive power), (positive power, on the tech-lab list),
  (tech-lab list, first superweapon), (first superweapon, storage), (storage, vehicle factory).
  One spy per building, in separate runs. Expected effects: re-cover only (the enemy's map is not
  ours to see: use the enemy's view in a second run where the person plays the victim and a
  computer spy enters, or observe our announcements of claim 11), blackout only, stolen tech
  only (our sidebar offers, S-0407 T6), superweapon reset only (the enemy's countdown row, S-0512),
  theft only. With
  `quirks.infiltration_single_effect` off in our engine, both effects of each pair. Falsified if a
  second effect occurs in the original, or the first effect is missing. Add a (radar,
  tech-lab list) pair: re-cover only, no stolen tech, agreeing with S-0407 claim 6.
  Record arrival, re-covered cells, blackout start, buildable offers and charge-clock state
  with tick-labelled read-only observations. Expected: re-cover, blackout start and reset
  occur in the arrival update, and stolen-tech offers are available after that tick's house
  update. Repeat the reset with a running clock and a suspended stopped clock: the former
  continues from full duration, the latter stays stopped at full duration. A one-tick delay,
  unexpected resumption or unexpected stop falsifies claim 4.
- **I3 (claims 6, 7; F1).** A named test file gives a spyable enemy test building type storage
  above 0 and none of the earlier properties of claim 3. Part a: enemy balance 10 001, and the
  named test file gives the steal fraction `0.5`. Expected: our balance rises by
  5000 in the arrival tick. Part b: the named file gives `57%`, enemy balance 100. Expected: +56,
  not +57. Part b′: `33%` and enemy balance 1000. Expected: +329 (330 would mean nearest rounding).
  Part c: two spies enter in consecutive ticks with `0.5` and enemy balance 1000. Expected: +500
  then +250. Part d: the named file gives `150%`, enemy balance 1000. Expected: +1500 for us.
  Record both houses' balances as well as ours in every part: expected victim losses in a–d
  are 5000, 56, 329, then 500 and 250, then 1000 respectively. Part e: `−50%`, enemy
  balance 1000 and our balance 100; expected our balance −400 and enemy balance 1500.
  Part f: two spies arrive in the same tick with `0.5` and enemy balance 1000. Arrange
  their starting distances to coincide, and void the run unless the position trace confirms
  both arrivals in that tick. Expected: the final enemy balance is 250 and our gain is 750,
  not a gain of 1000 from two independently computed thefts.
  Falsified by any other balance or arrival-tick delta.
- **I4 (claim 5).** Enemy barracks and enemy naval yard; a named test file makes both types
  spyable. The person plays a country whose veteran lists contain none of the types trained
  below. Before any infiltration, train one trainable infantry and one trainable vehicle of ours: expected rookies.
  Spy into the barracks; train again: the infantry is a veteran (rank insignia), the vehicle a
  rookie. Spy into the naval yard; train a trainable ground vehicle: veteran; build an aircraft:
  rookie. Then destroy both enemy buildings and train again: still veterans. Falsified if the
  enemy's units rather than ours are promoted, or if a flag clears. Also record our sidebar
  icons before and after each entry: eligible types change to their alternate icons, not the
  enemy's. Repeat an entry before destruction: ranks and icons stay unchanged. In separate
  runs replace destruction with sale, capture by a third house, and victim defeat; retain an
  additional enemy contender so defeat does not end the game. Train again after each event:
  still veterans with alternate icons. Any reversion or additional promotion falsifies claim 5.
- **I5 (claim 3, own and allied; Open questions).** A second computer house is allied with us.
  A spy of ours is ordered into an enemy power plant; while it walks, a map trigger changes the
  plant's owner to the allied house (run 1) or to our house (run 2). Three outcomes are possible
  in each run: the spy enters and the effect happens, the spy enters and nothing happens, or the
  spy stops short of the plant. Expected: run 1, the ally's power is blacked out (a bench read of
  its power state); run 2, the spy is used up and nothing happens. Falsified if the spy enters and
  the outcome differs. A spy that stops short is inconclusive for claim 3 and answers the open
  question only if a read-only target/destination trace confirms clearing; otherwise it is
  inconclusive about why the approach stopped.
- **I6 (claims 1, 10).** A named test file gives an infantry type the infiltrate flag only (no spy,
  no engineer flag), and another type both the spy and the engineer flags. Order each into an enemy
  capturable building with no garrison. Expected: the first stands in the building's cell, then
  walks off it, the building unchanged; the second behaves as an engineer (S-0509). Falsified if
  either infiltrates.
- **I7 (claim 8).** The enemy owns a transport-like building (a named test file gives a stock
  spyable building type a passenger capacity and the passengers pip-scale kind, S-0111 OV6) and a
  barracks that is training. For this run only give the enemy enough credits to train,
  and void it if any unrelated enemy action reaches the observed area. A named override
  makes the barracks spyable. A spy enters the first; a second spy enters the barracks. Expected:
  no pip scale appears on the first building and no production display on the barracks, before or
  after. Falsified if either appears.
- **I8 (claim 9).** The infiltrated test building carries a map tag whose trigger has the
  entered-by event and an action that gives the person a known text or sound; the spy carries a
  tag with destroyed-by-anything and a different visible action. Expected: both actions in the
  spy's arrival tick, the entered-by one first, also for a building of no row and for I6's
  non-spy entrant (only the entered-by action for the non-consuming scatter case). Falsified if either
  required action is missing, they come in another order or in different ticks, or the
  non-consuming entrant causes a destroyed-by-anything action.
- **I9 (claim 11).** Run I2 and I3 with announcements recorded (audio capture or the bench's
  announcement log), once as infiltrator and once as victim (a computer spy entering the person's
  buildings). Add single-property test buildings for each row not independently exercised by I2,
  including both infantry and vehicle factories. Repeat E1 with an active spy satellite: no radar
  lines. Record radar event kind and cell with the recorder: every different-house entry involving
  the local player creates the infiltrated event at the building, including the no-row and
  satellite cases. I5's own-house entry creates no event and no line, conditional on entry being
  reached. Expected: the lines of the table, in that order, and none for a building of no row.
  Falsified if an event is missing, has another kind or cell, an own-house event appears,
  a line is missing, an extra line plays, or a pair plays in the other order.

## Open questions

- Whether a building's change of owner clears the target and destination of units heading to it
  was not traced. If it does not, the own-house case of claim 3 is reachable in normal play and a
  quirk switch for it (fixed behaviour: the spy does not enter a building of its own house) may be
  wanted. Scenario I5's two runs test this, with a stop-short outcome remaining inconclusive
  about the entry effect.
- The entry rule for a vehicle-like destination building (claim 1) is not specified here; S-0509
  should own it.
