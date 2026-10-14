---
id: S-0412
title: Skirmish setup rules (setup values, starting credits, random lobby choices, handicap, alliances at game start, superweapon and crate settings)
topic: simulation
revision: 6
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0045, Q-0192]
supersedes: []
---

# S-0412: Skirmish setup rules (setup values, starting credits, random lobby choices, handicap, alliances at game start, superweapon and crate settings)

## Summary

What the skirmish lobby fixes for a game and what each value does at game start: setup values,
starting credits, random country and colour choices, our engine's optional handicap, the alliances
every game starts with, the superweapon and crate settings, and the observable order of game start.
Start positions, starting units and the unused-house cleanup belong to S-0009, victory and defeat to
S-0414, country fields to S-0413; network games and campaigns are outside this spec.

## Terms

Glossary terms are used as defined there, in particular *house*, *country*, *side*, *skirmish*,
*lobby*, *game start*, *tick*, *tick number*, *credits*, *scenario random stream* (with *ranged
draw*), *AI house*, *human-controlled*, *computer-controlled*, *difficulty row*, *normal row*,
*difficulty skill block*, *house skill level*, *registry* and *registration index*, *rules layer*,
*base rules*, *absent-key default*, *quirk switch*, *departure*, *importer warning*, *available
credits*, *bystander computer player* and *measurement setup*. RZ₅₃ is S-0205's notation (Terms: a
real number rounded toward zero to a double). S-0009's terms *house order*, *house number*,
*neutral house*, *special house*, *passive house*, *observer*, *starting units*, *base unit*,
*base cell*, *budget* (with *budget pool* and *start pools*), *placement search*, *start waypoint
markers*, *early phase* and *late phase*, *game's tech level* and *R(a, b)* are used as S-0009
defines them, S-0107's *fraidycat* as S-0107 defines it (Terms), and S-0414's *contender* and
*person-played house* (Terms), *hostile pair* (claim 6) and the side test of claim 12, with its
*civilian side*, as S-0414 defines them. *Secret lab* and *wall-owner flag* appear only as pointers (S-0407 claim 14 and
S-0413 claim 6). In addition:

- **Budget unit.** A vehicle or infantry unit that a house buys with its budget as part of its
  starting units (S-0009 claim 45), as opposed to its base unit.
- **Counted building.** Always in S-0414's sense (Terms, claim 4): a building that keeps a house
  alive in the defeat tests, not S-0405's sense of buildings whose power is summed.
- **Computer player house.** A house that the original creates from a lobby computer slot (a
  *computer-controlled* house made for a player). The neutral and special houses are not computer
  player houses, although the original's rules for owners that are not human-controlled apply to
  their units. Claim 3 gives the maximum house skill level to computer player houses only, as
  S-0209 (claim 11 and its Import mapping) does. In our engine the AI houses take these slots
  (claim 12).
- **Setup values.** The values the lobby fixes for one game, listed in claim 1: game-wide values
  (credits setting, unit-count setting, bases setting, short-game setting, superweapon setting,
  crate setting, build-off-ally setting, redeploy setting, game speed, game mode, map) and
  per-player values (country, colour, start position, team, and for a computer player a
  difficulty). Our engine adds one per-player value, the handicap (claim 5).
- **Credits setting.** The setup value that gives every player house its starting balance
  (claim 2), an integer number of credits.
- **Unit-count setting.** The setup value *U* of S-0009 claim 45, a non-negative integer.
- **Bases setting.** The yes/no setup value of S-0009 claim 45 that decides whether every house
  receives a base unit.
- **Base-unit list.** A game-wide list of vehicle types in the rules data (Import mapping). S-0009
  uses it for the base unit, and S-0414 claim 2 for its first three types.
- **Short-game setting.** The yes/no setup value that selects the defeat rule of S-0414 (claim 2).
- **Lobby team.** A per-player setup value: either "no team" or one of a small set of team
  numbers. Players with the same team number start allied (claim 8).
- **Alliance, allied, mutually allied.** An alliance is a one-way relation between two houses: house
  A *allies* house B when A counts B as a friend. A and B are *mutually allied* when each allies the
  other. A house always counts as allied with itself. How alliances change during play is S-0414's
  (claims 12 and 13).
- **Random choice.** A lobby value left on "random": a country, a colour or a start position.
  Start positions are resolved by S-0009 (claims 17 to 22); countries and colours here.
- **The term "disableable superweapon"** is owned here, as the glossary says. It names a superweapon type whose type
  section carries the flag that lets the setup screen's superweapon setting take it out of the game
  (Import mapping); claim 9 states what the setting does to it. S-0407 and S-0413 use the term.
- **Mode list.** The data file that lists the lobby's game modes in families, one section per
  family (Import mapping). Which family a mode is listed under decides some of its set-up rules
  (claim 13).
- **Superweapon of a building type.** A building type names at most two superweapons, a first and
  a second (superweapon specs, planned S-0512; a pointer only).
- **Tech-lab list.** S-0407's term (Terms): a game-wide list of building types in the rules data
  (Import mapping). Here it matters only through S-0407 claim 7, which exempts its types from the
  build bar of the superweapon setting.
- **Difficulty index.** A number 0, 1 or 2 that every house has in the original, in the order easy,
  normal, difficult (the same order as the three difficulty rows). It selects the entry of every
  per-difficulty list that the original keeps in its rules (claim 3). It is a different thing
  from the *difficulty row* a house takes: a row is *applied* to a house, which copies its factors into
  the house (claims 4 and 5). A player house gets an index and an applied row together when it is
  created; the neutral and special houses get an index but never an applied row (claim 5).
- **Crate.** A random-bonus object placed on a cell, as S-0009 (Terms, *crate type*) describes it.
  What a crate gives and how crates expire or come back belong to the crate rules, which have no
  spec yet (a pointer only).
- **Secret lists.** S-0407's game-wide lists of secret infantry, vehicle and building types (claim
  14 there); K9 part a only sets how many entries they hold in total.
- **Human participant.** A person taking part in the game through the lobby, observers included. In
  the original a skirmish has exactly one. Our engine's setup may also start a skirmish with none,
  every player house then being an AI house; such a game runs and ends by S-0414 claim 6 (lead
  decision of 2026-10-11, "play to the last side").
- **Handicap.** Our engine's optional per-player setup value that picks a difficulty row for that
  player's house (claim 5). It is a rule of our engine, not of the original.

## Behaviour

### Setup values

1. **Where each setup value comes from in a skirmish.** The table lists every setup value, its
   source in the original and what reads it. In our engine the setup supplies every value; which
   defaults the setup screen offers, and the ranges of its sliders, are interface matters (Import
   mapping lists the rules keys the original uses for them).

   | Setup value | Source in the original's skirmish | Read by |
   |---|---|---|
   | credits setting | the lobby's credits slider, as an integer number of credits | claim 2 |
   | unit-count setting *U* | the lobby's unit-count slider | S-0009 claim 45 |
   | bases setting | no lobby control: the value of the base rules' bases default, copied once when the program starts (as the game's tech level is, S-0009 Terms); no later rules layer reaches the copy | S-0009 claim 45; S-0414 claim 2 |
   | short-game setting | lobby check box | S-0414 claim 2 |
   | superweapon setting | lobby check box | S-0407 claim 7; claim 9 |
   | crate setting | lobby check box | claim 10 |
   | build-off-ally setting | lobby check box | S-0406 claim 4 |
   | redeploy setting (whether a deployed base unit may pack up again) | lobby check box | S-0415 (claim 9, the pack-up) |
   | game speed | S-0201 claim 7 | S-0201 |
   | game's tech level | S-0009 Terms (no lobby control) | S-0009 claim 45 |
   | fog of war | always off in a skirmish | S-0409 claim 19; S-0110 |
   | bridge destruction | always on in a skirmish: bridges can be destroyed | bridge rules (planned S-0511) |
   | harvester truce | always off in a skirmish | economy rules |
   | multi-engineer rule | always off in a skirmish | capture rules (planned S-0509) |
   | per player: country, colour | lobby; "random" resolved by claims 6 and 7 | S-0009, S-0103 claim 10, S-0413 |
   | per player: start position | lobby; "random" resolved by S-0009 claims 19 and 22 | S-0009 |
   | per player: lobby team | lobby | claim 8 |
   | per computer player: difficulty | lobby | claim 4 (the original only) |

   In our engine the four values the original fixes in a skirmish (fog of war off, bridge
   destruction on, harvester truce off, multi-engineer off) are fixed the same way, and the bases
   setting is a setup value whose default is the base rules' bases default, or yes when the base
   rules omit it.
   - Backing: `measurable` (scenario K1 for the bases setting and its source; K2 for the four
     fixed values, through their effects). The slider and check-box sources are the lobby's
     interface and need no measurement.
   - Confidence: high for the sources of the lobby values and the four fixed values; medium for
     the bases setting being the copy taken at program start (no skirmish control was found, and
     a network lobby earlier in the same program run can change the copy, which is outside this
     spec)

### Starting credits and the original's computer-only advantages

2. **Starting credits.** When the houses are created, every player house, human or computer,
   observer included, gets a credit balance equal to the credits setting. The neutral and special
   houses start with 0 credits. A positive budget left over from buying starting units is added
   later (S-0009 claim 45). In our engine every player house, human or AI, starts with exactly
   the credits setting plus that left-over budget; nothing else is added at game start.
   - Backing: `measurable` (scenario K3)
   - Confidence: high

3. **Computer-only start rules of the original.** At game start the original also does three
   things that a human-controlled house never gets:
   - for every house that is neither human-controlled nor passive, an extra amount of credits,
     added to its balance, which includes its own left-over budget (claim 14):
     trunc(RZ₅₃(RZ₅₃(*p* × *c*) × *M*)), with *c* the double nearest to 1/100 (a constant of the
     program), *p* the integer percentage that the house's difficulty index (Terms) selects from the
     original's extra-credit percentage list (Import mapping; index 0, 1, 2 select its first, second
     and third entry) and *M* the house's available credits (glossary) at that moment. At game start nothing is stored, so
     that is its balance: for a computer player house the credits setting plus its left-over budget, for a
     neutral or special house only its left-over budget, since it starts with 0 credits (claim 2).
     The arithmetic is floating point (S-0205 claim 5: each multiplication rounds toward zero as
     a double) followed by a conversion that truncates (S-0205 claim 7); the result is an integer
     number of credits, and the house's balance rises by it. A neutral or special house with no
     left-over budget has *M* = 0 and so receives 0. Its left-over budget is above 0 only when it has had the
     limit of failed placements of starting units that S-0009 claim 45 names.
   - for every such house, a timer whose duration is taken by the difficulty index from another
     per-difficulty list (Import mapping), which only the original's computer player uses,
   - for every computer player house (Terms) and for no other house, the rules' maximum house skill
     level (glossary, *house skill level*); the neutral and special houses keep level 0, even when
     their countries are not passive.
   None of this applies to AI houses in our engine: they get the starting credits of claim 2, the
   house skill level of a human-controlled house, and no timer (glossary, *AI house*, decision of
   the project lead). This is recorded as the original's behaviour for AI designers, who may give
   their AI any advantage only through ordinary orders. The neutral and special houses keep the
   original's rules (decision of the project lead): when a rules file makes one of their
   countries non-passive, that house receives the extra credits by the same formula, with *p*
   taken by the difficulty index it was created with (claim 5 gives that index). That index selects
   a list entry only: no difficulty row is applied to these houses (claim 5). While their countries
   are passive, they receive nothing. A percentage list with fewer entries than the index needs
   leaves the original's result unspecified; our engine then gives 0 extra credits and the
   importer warns (Edge cases).
   - Backing: `measurable` for the extra credits (scenario K9 part b for computer player houses, the
     list entry each difficulty index selects and the rounding; part c for a non-passive special
     house, its index and its *M*) and for the skill level, which unit rules read: scenario K10
     for the neutral and special houses keeping level 0 even when their countries are not
     passive (our engine depends on this, since those houses keep the original's rules), and
     S-0209 scenario C7 (parts a, c and c2) for a computer player house at the maximum level and
     for the two houses at 0 with the stock passive countries. The timer is `unmeasured`: it acts
     only through the original's computer-player behaviour, which no scenario observes directly
     and which our engine does not reproduce (it has no effect on any unit rule).
   - Confidence: high that the three exist, for whom they apply (the skill level included: in a
     skirmish only the creation of a computer player house sets it), for the form of the formula
     and for the list the percentage comes from; medium that *M* is exactly the balance at that
     moment (the original uses the house's available credits, which at game start are its balance)

4. **Difficulty indexes and rows of player houses in the original.** When the houses are created,
   a player house gets its difficulty index (Terms) and, with it, that row applied (S-0302 claim
   3; S-0301 claim 3). In a skirmish the original gives every human-controlled house index 1, the
   normal row. A computer player gets the index that its lobby entry gives, and the row of that
   index. The lobby's three computer entries give these indexes (engine constants of the
   original, not rules data):

   | Lobby entry | Index given | Row applied |
   |---|---|---|
   | easy computer | 2 | difficult row |
   | normal computer | 1 | normal row |
   | hard computer | 0 | easy row |

   Each lobby entry carries its index with it, so the pairing does not depend on the order in which
   the setup screen lists the entries. This claim owns the pairing; S-0301 claim 3 and S-0302
   claim 3, which own what the rows' factors do, take it from here. Nothing in our engine depends
   on it (claim 5). So the original's "easy" computer entry gets the row named difficult and its
   "hard" entry the row named easy; which row's factors favour which house is rules data.

   When the game has more than one human participant and the rules' computer easy-bonus flag is
   on, a computer player whose index is above 0 gets the index one lower instead (so the easy
   computer gets index 1, the normal row, and the normal computer index 0, the easy row; the hard
   computer stays on 0). A skirmish has at most one human participant, so this step never happens in a
   skirmish and is recorded for completeness only.
   In our engine this claim describes the original only: claim 5 replaces it.
   - Backing: `measurable` for the pairing of entries with indexes and rows (S-0302 scenario D19
     part c, which sees the computer houses' firepower factors and tells this pairing
     from its inverse) and for the index each entry gives (K9 part b, which reads the extra credits that
     the index selects); the human's normal row is backed by the scenarios of S-0301 and S-0302,
     which run with it. The one-step rule is `unmeasured`: it needs a game with more than one human
     participant, which a skirmish never has and network games are out of scope.
   - Confidence: high for the human's normal row, for the order of the three rows by index and for
     the index each lobby entry gives, and so for the pairing with rows (K9 part b measures the
     index and D19 part c the row); medium for the one-step rule (never observed: it needs a game
     with more than one human participant)

5. **Rows and the handicap in our engine.** Every player house, human or AI, uses the normal row
   (glossary, *difficulty skill block*, decision of the project lead). The handicap is an optional
   per-player setup value: when a player sets it, that player's house uses the chosen row (easy,
   normal or difficult) instead. It is off by default, it is never set by AI difficulty, and a
   difficulty row that the map gives a house is ignored in a skirmish (decision of the project
   lead, 2026-10-10). The chosen row is fixed when the houses are created and acts only through
   the factors that the specs owning them use: the rate-of-fire factor (S-0301 claim 3) and the
   firepower factor (S-0302 claim 3). The row's armour, cost and build-time factors change
   nothing (S-0302 claim 3; build time and prices: S-0403 claim 9 and S-0404 claim 2). The neutral and special houses have a difficulty index but never have a row applied (the
   index and the applied row of the Terms differ here). When they are created in a skirmish,
   the original gives each of them difficulty index 1, which selects the normal (second) entry of
   every per-difficulty list (claim 3 uses it). No row is applied to them: their
   rate-of-fire and firepower factors are exactly 1.0, with neither a row nor their country's
   general factors applied (the original's rule, which they keep in our engine by decision of the
   project lead). Their countries' per-kind factors still apply to them (S-0413 claim 8). This
   claim states of the creation of these two houses only the index and the factors; which houses a
   skirmish creates is S-0009's. A row also holds fields
   that only the original's computer player uses (for example a delay before its buildings are
   repaired); they have no effect in our engine, handicap or not.
   - Backing: the rule of our engine (every house on the normal row, the handicap, the ignored map
     row) is `unmeasured`: it is our engine's own rule, and the factors themselves are measured
     through the scenarios of S-0301 and S-0302. The original's index of the neutral and special
     houses is `measurable` through the extra credits of claim 3 (scenario K9 part c, with a
     percentage list whose three entries differ). That no row and no country general factor is
     applied to them is `measurable` (scenario K5 part d: firepower factor exactly 1.0, and the
     interval between shots with a rate-of-fire factor of exactly 1.0).
   - Confidence: high for the normal row for every player house, the handicap and the missing row
     of the two non-player houses; medium for their index being 1 (in the original the index is
     the game-wide computer difficulty, which the main menu resets to 1 each time it is shown, so
     it is 1 in every skirmish started from the menu, also after a campaign was played earlier in
     the same program run; this is inferred, not observed, and K9 part c would
     show a different index through a different extra amount)

### Random lobby choices

6. **Random country.** A player whose country is left on "random" gets the country with
   registration index *k* (S-0004), where *k* is one ranged draw R(0, 9). The range 0 … 9 is fixed:
   it does not depend on how many countries the rules list, nor on whether a country is passive or
   carries the multiplayer listing flag (S-0413 claim 4), so a passive country can be chosen, and
   so can one without the flag, and a country registered eleventh
   or later never is (quirk `quirks.random_country_fixed_range`; registration indices count from 0, so
   the countries registered first to tenth can be drawn). In the original, players are resolved
   in the order of claim 7's paragraph "Order of the draws". With the switch off, our engine
   chooses uniformly among the countries that the setup screen offers instead.
   - Backing: `measurable` (scenario K4)
   - Confidence: high for the range and the order

7. **Random colour.** A player whose colour is left on "random" gets the colour slot (S-0103 claim
   10) given by the first of successive ranged draws R(0, 7) that is free. A slot is free for a
   player when no human player already has it (a human still on "random" has none), no computer
   slot has it fixed (a computer slot still on "random" has none until its turn) and no player
   resolved before this one has been given it. An observer gets no
   colour slot and takes no draw (S-0009 claim 22 owns the observer's other rules).
   With nine or more players needing a slot the original would draw for ever; the lobby allows at
   most eight players (S-0009 claim 23), so this cannot arise from the lobby. Our engine rejects a
   setup that leaves no free colour slot for a player on "random" (a departure for a case the
   original's lobby never produces).
   - Backing: `measurable` (scenario K4)
   - Confidence: high

   **Order of the draws** (claims 6 and 7, the original). On the scenario random stream, the ranged
   draws of claims 6 and 7 come in one group per player. The groups of the human players come
   first, in lobby order, and then those of the computer slots, in slot order (a slot with no
   player has no group). Within one player's group, the country draw (one ranged draw, present
   only when the country is on "random") comes before the colour draws (present only when the
   colour is on "random": as many ranged draws as it takes to reach a free slot). A colour slot
   given to a player of an earlier group counts as taken for the colour draws of every later
   group.
   Our engine need not follow this order: with the draws coming from the setup, no observable
   shows in which order the players are resolved. It must only give the same results as far as
   the ranges and the uniqueness of colours go (claims 6 and 7).
   - Backing: `measurable` for the original (scenario K4 part c, which replays the draws from the
     recorded stream state)
   - Confidence: high

   **When the random choices are drawn.** In the original the draws of claims 6 and 7 are taken
   when the player starts the game from the lobby, before the streams are seeded for the game
   (S-0202 claim 8). They therefore come from whatever state the scenario random stream had before,
   and they take nothing from the game's own draws: the seeded stream starts afresh afterwards. In
   our engine the setup resolves random countries and colours before the game, within the
   ranges and the uniqueness rule above; how the setup draws them, and in which order it resolves
   the players, is an interface matter, like the seed (S-0202 claim 8). The game's scenario random stream takes no draw for them. Random start
   positions are different: S-0009 (claims 18 to 20) draws them from the game's own stream while the
   map loads.
   - Backing: `measurable` (scenario K4, part b)
   - Confidence: high

### Alliances at game start

8. **Alliances every skirmish starts with.** Houses start with no alliances. Game start makes the
   following alliances, and they are in place before tick 0 (claim 14):
   - every house other than the special house is made to ally the special house, and the special
     house is made to ally it,
   - for every two houses that are not passive and have the same lobby team number (not "no
     team"), each is made to ally the other.
   Each of these one-way alliances is made only when the house to be allied does not play a
   country of the civilian side (the side test of S-0414 claim 12, which applies at game start too, although the
   other conditions of that claim do not apply while the game is being set up). So when the
   special house's country belongs to the civilian side, the special house allies every house whose
   country is not of the civilian side, and no house allies the special house; and the special house allies the neutral house only when the
   neutral house's country is not of the civilian side. Which side the stock countries belong to is
   stock data. No alliance message or announcement is shown or played for these alliances. Nothing else is allied at the start: players on "no team",
   and players on different teams, start as enemies of each other. An AI house's lobby team works
   exactly like a human's. What each direction of an alliance decides (for example automatic
   targeting) belongs to the specs of those rules.
   - Backing: `measurable` (scenario K5: part a the teams and the absence of a message, part b
     the special house and the human, part c the special house and the neutral house)
   - Confidence: high for the team rule, the special house's alliances and the side test; medium
     that no message is shown at game start

### The superweapon setting

9. **Superweapon setting off.** While the superweapon setting is off:
   - (a) **Building.** Whether a building type may be built is S-0407 claim 7's rule, which owns
     the build bar and its exemption for tech-lab list types; this spec states only the setting
     it reads. A building type that the bar refuses is never offered, so no house of the game owns
     one by building it.
   - (b) **Granting.** No house is granted a disableable superweapon, whichever building would give
     it (first or second superweapon, built, captured or placed by the map, tech-lab list or
     not). A superweapon that is not disableable is granted as usual. Part (b) tests the flag of
     the superweapon, not the building's position in the list, so the exemption of S-0407 claim 7
     does not reach it.
   With the setting on, neither rule applies. A one-time superweapon that a map trigger gives is
   not covered by rule (b); trigger grants belong to the trigger specs. How a superweapon is
   granted, charged and fired belongs to the superweapon specs (planned S-0512).
   - Backing: `measurable` (scenario K6 for part b; part a is S-0407 scenario T7)
   - Confidence: medium for part b (the granting rule is known for buildings the house owns,
     and captured buildings follow from it); part a is S-0407's

### The crate setting

10. **Crate setting.** With the crate setting on, the game places *n* crates at game start, after
    the starting units are placed (claim 14):

    > *n* = min(max(*C*<sub>min</sub>, *H*), *C*<sub>max</sub>)

    with *C*<sub>min</sub> and *C*<sub>max</sub> the rules' crate minimum and crate maximum
    (integers, Import mapping; absent-key defaults 1 and 255) and *H* the number of human
    participants, an observer included: computer players and AI houses do not count (quirk
    `quirks.crate_count_humans_only`; with the switch off, *H* is the number of player houses).
    *H* is 1 in a skirmish of the original. In our engine it is 1, or 0 in a skirmish with no human
    participant (Terms), which then starts with min(max(*C*<sub>min</sub>, 0), *C*<sub>max</sub>)
    crates under the switch, so none when the crate minimum is 0. Integers, exact. Where each crate goes, the
    draws it takes, and how crates are replaced after being collected or after their time runs out,
    belong to the crate rules (no spec yet; S-0203 row 16 gives the place of crate replacement in
    the tick). With the crate setting off, no crate is placed at game start and none is ever
    replaced, so a skirmish then has no crates at all: a map's own crate overlays never appear in a
    skirmish either way (S-0009 claim 12).
    - Backing: `measurable` (scenario K7)
    - Confidence: medium (the formula is clear; that *H* is 1 in a skirmish is known for a
      skirmish started the usual way, not for every way a skirmish can start). The case of no
      human participant is a rule of our engine and needs no backing.

### Further setup rules

11. **The bases and short-game settings together.** With the bases setting off, no house receives
    a base unit (S-0009 claim 45). With the short-game setting on as well, a house passes the
    short-game test (S-0414 claim 2) only through a counted building (S-0414's sense, Terms) or a vehicle of one of the
    first three base-unit types, which it then owns by other means: a building or vehicle
    pre-placed for it by the map (S-0009 claim 27: a human-controlled house never keeps a
    pre-placed building, but keeps a pre-placed vehicle), or a vehicle of such a type among its
    budget units, when the budget pool holds one. A house with none of these at tick 1, the first
    tick with a defeat test, fails the test then. On a map that pre-places none of them, and
    with a unit-count setting that gives no such vehicle, every non-passive house fails in tick 1
    (quirk `quirks.short_game_without_bases`). The houses are tested in house order, in which the
    human participant's house comes first (S-0009 claim 19). In our engine the game ends in tick 1,
    at the first of these defeats after which no hostile pair remains, and the houses still
    undefeated then fail the test in the same tick as well. With `quirks.defeat_during_end_delay`
    on, that failure defeats them, so every player house loses, all in tick 1; with it off, they
    keep their win (S-0414 claims 6, 9 and 10). The original ends the game already at the human's defeat, the first of the tick
    (S-0414 claim 6, departure); the human's loss in tick 1 is the same. Where the map does
    pre-place such objects, only the houses without any fail, and the game goes on or ends as
    S-0414 claim 6 says for those defeats. The original's network lobby keeps the two settings consistent (switching short game
    on switches bases on, and switching bases off switches short game off); the skirmish lobby
    has no bases control. With the switch off, our engine refuses a setup with the short-game
    setting on and the bases setting off.
    - Backing: `measurable` for the original (scenario K1, parts b and c, which observe the
      original's end at the human's defeat); our engine's end of such a game is S-0414's rule
    - Confidence: medium (the outcome follows from S-0414 claim 2 and S-0009 claims 27 and 45, and
      the observed combinations are only those of K1)

12. **AI houses take the computer slots.** Every rule of the original that counts computer
    players of the lobby counts AI houses in our engine, because an AI house takes the place of a
    computer slot. In particular the unused-house cleanup of S-0009 (claim 28), which runs only
    in a game with no computer players, never runs in a game with an AI house (this answers
    question 5 of request Q-0045). This is a rule of our engine; it does not change any unit rule (glossary, *AI house*).
    AI houses are also contenders in S-0414's end test (claim 6), so they are sides for the end of
    the game like any player house; no rule of this spec counts the person-played house.
    - Backing: `unmeasured` (a rule of our engine; the original's condition is measured by S-0009
      scenario M16)
    - Confidence: high

13. **A mode that gives every base unit.** The mode list files some game modes under a family
    named in the Import mapping, the *all-base-units family*. Each mode of that family gives every
    eligible house the whole base-unit list, and in it the bases setting is forced on. A house is
    eligible under the same non-passive, non-observer conditions as S-0009 claim 45. No other
    field of the mode or of its game-mode file selects this rule: it belongs to the family.
    Instead of S-0009's single base unit, every eligible house receives one vehicle of each
    type in the base-unit list, whatever that type's owner list says. The vehicles are placed in
    reverse list order, the last entry's first and the first entry's last; the order is observable
    through which vehicle takes which cell. Their cells:
    - the first entry's vehicle stands at the centre of the cell three cells further along both the
      x and the y axis from the base cell, when that cell can take it,
    - every other vehicle, and the first entry's vehicle when that cell cannot take it, gets the
      cell that S-0009's placement search (claim 46) finds with first radius 3.

    What the houses end up with (a vehicle that cannot be created or placed does not remain in the
    game; an empty list position counts as such a failure):

    | The list's vehicles, for this house | This house | Every later house (house order) |
    |---|---|---|
    | every vehicle created and placed | one vehicle of each listed type; its budget units and left-over credits as in S-0009 claim 45 | starting units as usual |
    | at least one failure | every vehicle of the list that could be created and placed, and nothing else: no budget units, no left-over credits | no starting units (S-0009 claim 45's early stop) |

    S-0009 claims 45 and 46 describe standard skirmish starting units. This claim replaces their
    single-base-unit placement and failure rules for the all-base-units family. The budget,
    budget-unit purchases and shared placement search still follow those claims, except where
    this claim states an alternative.

    **Starting-unit draws in this family.** Houses retain S-0009's house order. For each house,
    the base-list vehicles' draws follow reverse list order: each vehicle's creation draw,
    followed by its placement-search draws when a search is needed, then its placement draw
    if its type has a visceroid flag (S-0009 claim 46). Selecting a base-list type takes no
    random draw. A successful direct placement of the first list entry takes no search draw.
    Every search uses S-0009 claim 46's candidate priority and draw boundary, with first
    radius 3 rather than the standard base unit's radius 1. Thus this family's starting-unit
    draw sequence need not equal the standard mode's. When all base-list vehicles succeed,
    the house's budget-unit draws follow as in S-0009 claim 46. After all houses succeed,
    the final draw R(0, 65535) of S-0009 claim 20 is taken. A base-list failure does not cancel
    the draws or placements of the other base-list vehicles of that house, but it prevents
    that house's budget-unit draws, every later house's starting-unit draws and the final draw.

    Other game modes may change further setup rules; they are outside this spec (Open questions).
    - Backing: `measurable` (scenario K8: the list rule and the cells in part a, the reverse order
      in part b, the failure rule and the early stop in part c; part d for the family being what
      selects the rule; part e for the starting-unit draw sequence)
    - Confidence: medium for the list rule, the order and the early stop; low for the placements
      and their draws; medium that the family is the selector (K8 part d)

14. **The order of game start (Q-0192).** Game start takes place once the map is loaded and the houses
    exist, and is over before tick 0. This claim is the one place that states the observable order
    of what it draws and changes (S-0407 claim 14 and S-0413 claim 6 point here; the table "Game
    start, in observable order" in State and ordering sums it up). It covers the standard
    skirmish game mode; what other modes add at game start, such as a whole-map reveal, is outside
    this spec (claim 13 is the exception), and this claim fixes no order between those additions
    and the items below, except that all of it is over before tick 0.
    - **Draws.** These groups of scenario-stream draws occur in this order: the starting-unit
      draws (S-0009 claims 45 and 46 for standard skirmish mode); the initial crate draws,
      when the crate setting is on, each crate taking its own draws (claim 10; crate rules,
      no spec yet); the secret-lab draws, one ranged draw per secret lab when the conditions
      of S-0407 claim 14 hold; the end-of-loading tag draws (S-0011 claim 34); and the
      game-start tile-animation creation draws described by S-0202 claim 14. The last group
      draws only when that pass starts an animation. Thus secret-lab choices are not the last
      loading draws: their choices affect the stream values received by the later tags and
      tile animations. The game-start recalculation follows crates, secret labs and tags even
      when no starting units were placed or their placement stopped early. This claim fixes
      these relative positions, not an exhaustive last-draw boundary for loading.
      The all-base-units family retains this group order, but its starting-unit group's draws
      follow claim 13, not the standard mode's base-unit draw sequence. Scenario K9 part a
      tests the group order in both families; S-0202 R10 tests the later animation draws.
    - **Removed houses.** A house that the unused-house cleanup (S-0009 claim 28) removes takes no
      draw for starting units and receives no starting units (K9 part d). It also gets no
      left-over budget, none of the extra credits of claim 3, none of the alliances of claim 8 and
      no new wall-owner flag value.
    - **Balances.** The left-over budget credited to a house is what its own starting units left
      of its budget (S-0009 claims 45 and 46). The extra credits of claim 3 are computed on a
      balance that already includes that left-over budget: the house's starting balance of claim 2
      (the credits setting for a player house, 0 for the neutral and special houses) plus its own
      left-over budget. Nothing else from game start is in it (crates take no credits at game
      start). The extra credits take no draw and nothing from another house, so whether they are
      added before or after another house's starting units is not observable, and this spec fixes
      no such order.
    - **Wall-owner flags.** The country flags of S-0413 claim 6 take their new values before tick
      0, for the houses that remain after the cleanup. No order within game start is observable for
      them, and this spec fixes none.
    - **Alliances.** The alliances of claim 8 exist between the houses that remain after the
      cleanup and are in place before tick 0. Nothing observable depends on when within that span
      they are made, and this spec does not fix it; nor does it fix the order, for one house, of
      its extra credits, its country's flag and its alliances with the special house, which no
      observable separates.
    Random countries and colours (claims 6, 7) are resolved before all of this, before the streams
    are seeded; the consequence is that they take no draw of the game (claim 7, K4 part b). When
    the houses are created (claims 2 and 5) relative to the map's loading is S-0009's (claim 19 and
    its loading order); this claim states no order for it, and the only fact of this spec that
    depends on it is that the balances and indexes of claims 2 and 5 exist before game start.
    - Backing: `measurable` (scenario K9 part a, with the random-state recorder of the measurement
      setup, for the starting-unit, crate and lab draw groups and their order; S-0202 R10,
      including variants A and B, for the later tag and tile-animation ordering and the
      no-unit and early-stop cases; K9 part d for the cleanup before the starting
      units; the balances through K9 parts b and c). The unobservable orders named above are not
      measurable and are not claimed.
    - Confidence: high for the draw groups and their order and for the balances; medium for the
      cleanup before the starting units (S-0009 claim 28 backs that a removed house gets none; K9
      part d is the first scenario that separates the order through the draw count)

## Formulas

**Initial crate count** (claim 10). With *C*<sub>min</sub>, *C*<sub>max</sub> and *H* signed 32-bit
integers:

> *n* = min(max(*C*<sub>min</sub>, *H*), *C*<sub>max</sub>)

No crate is placed when *n* ≤ 0. Integers, exact.

**Random country** (claim 6). *k* = R(0, 9) on the original's scenario random stream, as it stands
before the game's seeding; the house's country is the country with registration index *k*.

**Random colour** (claim 7). The player's colour slot is the first value *s* of the successive draws
R(0, 7) that is free (claim 7).

**Extra credits of a computer-only start** (claim 3, original only). With *p* an integer percentage,
*c* the double nearest to 1/100 and *M* an integer number of credits: extra = trunc(RZ₅₃(RZ₅₃(*p* × *c*) ×
*M*)), floating point (each product a double rounded toward zero, S-0205 claim 5), then a conversion
that truncates toward zero (S-0205 claim 7); the result is an integer number of credits.

## Constants


| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Random country range | 0 … 9 | registration index | claim 6; fixed in the original |
| Random colour range | 0 … 7 | colour slot | claim 7 |
| Neutral and special houses' starting balance | 0 | credits | claim 2 |
| Crate minimum and crate maximum, absent-key defaults | 1, 255 | crates | claim 10 |
| Percentage factor of the extra credits | the double nearest to 1/100 | none (a factor) | claim 3 |
| Credits setting, absent-key default of the default offered | 10000 | credits | Import mapping |
| Credits slider, absent-key defaults of minimum, maximum and step | 2500, 10000, 100 | credits | Import mapping; interface |
| Unit-count setting, absent-key defaults of minimum, default and maximum | 1, 10, 20 | count | Import mapping; interface |
| Bases setting, absent-key default | yes | boolean | claim 1 |
| Crate setting, short-game setting, superweapon setting, build-off-ally setting, redeploy setting: absent-key defaults of the defaults offered | yes, no, yes, yes, yes | boolean | Import mapping; interface |
| Computer easy-bonus flag, absent-key default | yes | boolean | claim 4 (the original only) |
| Difficulty index of a human-controlled house in a skirmish | 1 | difficulty index | claim 4 (the original only) |
| Difficulty index of the neutral and special houses | 1 | difficulty index | claim 5; selects list entries only |
| Difficulty index given by the lobby entries easy, normal and hard computer | 2, 1, 0 | difficulty index | claim 4; the original only (K9 part b; S-0302 D19 part c) |
| Distance of the first vehicle's cell from the base cell in claim 13 | 3 | cells, along x and along y each | claim 13 |
| First radius of the placement search in claim 13 | 3 | cells | claim 13 |

## State and ordering

**Game start, in observable order** (skirmish). Claim 14 states the observable order of game start
and is its one owner; the table sums it up and adds what comes before it. The rows give only what
this spec adds to S-0009's loading order (S-0009 claim 20 and its table "Start position
resolution").

| When | What happens | Owner |
|---|---|---|
| Before seeding | random countries and colours resolved (claims 6, 7) | this spec |
| House creation | player houses get the credits setting (claim 2), the index and row of claims 4 and 5; neutral and special houses get 0 credits and, by claim 5, an index and no row; the place of this step relative to map loading is S-0009's | values: this spec; order: S-0009 claim 19 |
| Map loading | start positions, map objects | S-0009 |
| Game start | the unused-house cleanup: a removed house takes no starting-unit draw and receives nothing from the rows below (claim 14) | S-0009 claim 28; claim 14 |
| Game start, first group of draws | starting units of the remaining houses; each house's balance gains what its own starting units left of its budget | S-0009 claims 45, 46; claim 13 for the all-base-units family |
| Game start, after the starting units | crates placed when the crate setting is on; the second group of draws | claim 10 |
| Game start, not ordered against other houses | the extra credits of claim 3 for each house that gets them, computed on a balance that includes the house's own left-over budget (claim 14) | claim 3; claim 14 |
| Game start, before tick 0, after the cleanup | the alliances of claim 8 and the wall-owner flag rewrites, in no observable order | claim 8; S-0413 claim 6; claim 14 |
| Game start, after initial crates | secret labs get their types, one ranged scenario-stream draw per lab when the draw conditions hold (the alliances and flag rewrites above are not ordered against them) | S-0407 claim 14; claim 14 |
| End of loading, after secret labs | remaining tags take their creation draws | S-0011 claim 34; S-0202 claim 14 |
| Game-start recalculation, after end-of-loading tags | tile animations started by the pass take their creation draws in visiting order, before tick 0; this is not a claim that no later loading draw exists | S-0010 claims 12 and 19; S-0202 claim 14 |
| Tick 0 | first tick; no defeat test in tick 0 (S-0414 claim 1) | S-0414 |

## Edge cases and quirks

- **`quirks.random_country_fixed_range`** (claim 6). Original: the random country is one of the
  countries registered 0 … 9, whatever the country list holds, passive ones included. With a list
  of fewer than ten countries the original can pick an index with no country; that case is
  unspecified in the original, and our engine then draws again until the index names a country (a
  departure; the importer warns when the rules list fewer than ten countries). Off: uniform among
  the countries the setup offers.
- **The build bar's switch** (claim 9 a) is not this spec's: S-0407 claim 7 owns the rule, and
  whether it gets a quirk switch is S-0407's to decide (proposed name
  `quirks.superweapon_setting_tests_first_only`; original: only a building type's first superweapon
  is tested).
- **`quirks.crate_count_humans_only`** (claim 10). Original: only human participants count, so a
  skirmish starts with max(*C*<sub>min</sub>, 1) crates, capped by *C*<sub>max</sub> (in our engine
  max(*C*<sub>min</sub>, 0) when the skirmish has no human participant, claim 10). Off: every
  player house counts.
- **`quirks.short_game_without_bases`** (claim 11; default on, original). Short game with bases
  off fails every non-passive house that owns no counted building and no vehicle of the first
  three base-unit types, in tick 1. When neither the map nor the budget units supply any of those
  objects, the original ends at the human's defeat with a loss in tick 1 (S-0414 claim 6,
  departure). Our engine instead ends at the first defeat that leaves no hostile pair, and the
  remaining houses are defeated later in house order in the same tick. Every player house loses
  when `quirks.defeat_during_end_delay` is on; with it off, those undefeated at the end keep
  their wins (S-0414 claims 6, 9 and 10; claim 11). Off: such a setup is refused.
- **Extra-credit percentage list too short** (claim 3). When the list holds fewer entries than the
  house's difficulty index needs (or none), the original's result is unspecified (claim 3); our
  engine gives 0 extra credits and the importer warns.
- **Random colour with no free slot** (claim 7): never produced by the original's lobby; our engine
  refuses the setup (a departure).
- **Bases setting from an earlier network lobby.** In the original a network lobby can change the
  bases copy, which a later skirmish of the same program run then uses. Network games are out of
  scope; our engine takes the bases setting from the setup only.
- **The country's colour field** is overwritten when houses are created (S-0413 claim 5).

## Import mapping

Most keys of the base rules' multiplayer dialog section only set what the setup screen offers
(claim 1); the game then reads the setup values themselves. The exceptions are `Bases` (the
bases setting), `TechLevel` (the game's tech level) and `AlliesAllowed` and `AllyChangeAllowed`
(rules flags), as the rows say. Parse forms follow S-0003.

| Key | Concept | Parse form | Default when absent |
|---|---|---|---|
| `[MultiplayerDialogSettings]` `Money` | default credits setting offered | integer | 10000 |
| `[MultiplayerDialogSettings]` `MinMoney`, `MaxMoney`, `MoneyIncrement` | credits slider range and step (interface) | integers | 2500, 10000, 100 |
| `[MultiplayerDialogSettings]` `UnitCount`, `MinUnitCount`, `MaxUnitCount` | default unit-count setting offered and its slider range (interface) | integers | 10, 1, 20 |
| `[MultiplayerDialogSettings]` `Bases` | the bases setting (claim 1): the base rules' value, copied once when the program starts | boolean | yes |
| `[MultiplayerDialogSettings]` `ShortGame`, `Crates`, `MCVRedeploys` | defaults offered for the short-game, crate and redeploy settings (interface) | boolean | no, yes, yes |
| `[MultiplayerDialogSettings]` `SuperWeaponsAllowed`, `BuildOffAlly` | defaults offered for the superweapon and build-off-ally settings (interface) | boolean | yes, yes |
| `[MultiplayerDialogSettings]` `FogOfWar`, `BridgeDestruction`, `HarvesterTruce`, `MultiEngineer` | read, but a skirmish fixes these four values (claim 1) | boolean | — |
| `[MultiplayerDialogSettings]` `AlliesAllowed`, `AllyChangeAllowed` | not setup values: rules flags, read with the game's rules layers, that decide whether the alliance order can be given (S-0414 claim 14) | boolean | no, yes |
| `[MultiplayerDialogSettings]` `TechLevel` | the game's tech level (S-0009) | integer | 10 |
| `[CrateRules]` `CrateMinimum`, `CrateMaximum` | *C*<sub>min</sub>, *C*<sub>max</sub> (claim 10) | integers | keeps the value in force; 1 and 255 when no layer gives them |
| superweapon type section, `DisableableFromShell` | disableable superweapon (claim 9 b; S-0407 claim 7) | boolean | no |
| building type section, `SuperWeapon`, `SuperWeapon2` | first and second superweapon of the building type (claim 9 b; S-0407 claim 7) | superweapon type name | none |
| `[AI]` `BuildTech` | tech-lab list (S-0407 claim 7, which owns it; claim 9 only points) | list of building type names | empty |
| `[General]` `MultiplayerAICM` | the original's extra-credit percentage list (claim 3, original only): entry *i* is for difficulty index *i* | list of integers | no entries |
| `[General]` `AIHateDelays` | the original's computer-player timer list (claim 3, original only; no effect in our engine): entry *i* is for difficulty index *i* | list of integers | no entries |
| `[AI]` `CompEasyBonus` | computer easy-bonus flag (claim 4, the original only) | boolean | yes |
| mode list file, section `Unholy` | the all-base-units family (claim 13): every mode listed under this section has the rule; the section name is fixed by the program | mode entries | no family, no mode |
| `[General]` `BaseUnit` | base-unit list (claim 13; S-0009) | list of vehicle type names | empty |
| difficulty sections `Easy`, `Normal`, `Difficult` | the three difficulty rows (claims 4, 5; S-0301, S-0302) | — | — |
| theater section named by the tileset's `SetName`, `TileNNAnim`, `TileNNAttachesTo` | tile-animation eligibility for the later draw group of claim 14; full placement mapping belongs to S-0010 claim 19 | animation-type name and integer sub-tile index; NN is the tile number within its set, formatted as S-0010 specifies | no animation; attaches-to −1, which never starts |

No setup value or data key selects the relative draw-group order of claim 14: it is fixed.
End-of-loading tag eligibility, timer fields and their parse forms and defaults remain owned
by S-0011 claims 13, 14 and 34 and its Import mapping; applicable animation-creation draws
remain owned by S-0202 claim 10.

The settings file's skirmish section remembers the lobby's last choices (credits, unit count, the
check boxes and the computer slots), it is interface data and feeds no rule here.

## Measurement scenarios

All scenarios use the measurement setup (glossary): stock data unless a test rules file is named,
at 800 × 600, with exact seeds where a seed matters.

**K1: the bases setting.**
- *Part a.* Setup: a skirmish on a small two-player map, one computer player. Run 1 with the stock
  rules; run 2 with a loose test copy of the base rules whose multiplayer dialog section sets the
  bases key to no (start the program after placing it); run 3 with that key set to no only in a
  test game-mode file. Observable: whether each house has its base unit at tick 1. Expected: run 1
  yes, run 2 no, run 3 yes (only the base rules' value at program start counts). Falsified if run 2
  shows a base unit, or run 3 shows none.
- *Part b.* As run 2, with the short-game box ticked, unit-count setting 0 (the copy sets the
  minimum unit count to 0, as K3 does) and a test map that pre-places nothing for the human or
  the computer player. Observable: the game's end and its tick. Expected (the original): the game
  ends in tick 1 with a loss for the human (S-0414 claims 6 and 10). Falsified if play continues.
- *Part c.* As part b, on test maps that pre-place objects: (1) a vehicle of the first base-unit
  type for the human and nothing for the computer player; (2) a vehicle of the first base-unit
  type for the human and a counted building for the computer player; (3) a counted building owned
  by the human (it is dropped, S-0009 claim 27) and nothing else. Observable: the game's end and
  its tick. Expected: (1) the game ends in tick 1 with a win for the human (the computer player
  fails alone); (2) play continues past tick 1; (3) as part b, a loss for the human in tick 1.
  Falsified by any other outcome, in particular when (2) ends in tick 1 (then no house would
  survive without bases) or when (3) continues.

**K2: the four fixed values.** Setup: a skirmish with a test base rules file setting fog of war on,
bridge destruction off, harvester truce on and multi-engineer on in the dialog section. Procedure:
destroy a bridge with artillery; attack the enemy harvester; let one engineer enter a large enemy
building. Observable: whether fog appears, the bridge falls, the harvester takes damage, and the
building is captured by one engineer. Expected: no fog, the bridge falls, the harvester takes
damage, the engineer behaves as with multi-engineer off. Falsified if any of the four follows the
test file.

**K3: starting credits.** Setup: a skirmish with a loose test copy of the base rules, placed before
the program is started, as K1 run 2 uses one (the slider's lower end for the unit-count setting
is read from the base rules only, so a test rules layer cannot lower it); the copy sets the minimum
unit count to 0. Unit-count setting 0 (no budget), credits slider at its minimum, bases setting
left on, crate setting off (a crate could add credits), one computer player, no production, selling
or income. Observable: the human's credits as the sidebar shows them once the counter has settled
(S-0416 claim 2: the counter rises by at most 143 credits per cycle, so the sidebar shows the
balance only after at least the slider value divided by 143 cycles; read when the value has not
changed for five drawn frames; the climb itself is S-0416 scenario D3, and at tick 1 the sidebar
shows a much smaller number); the neutral house's credits through a test map trigger that fires when
that house's credits exceed 0. Expected: the human exactly the slider value; the neutral house 0.
Repeat at the maximum. Falsified if the human's settled balance differs from the slider value, or
if the neutral house's trigger fires (its credits are above 0).

**K4: random country and colour.**
- *Part a.* Setup: a skirmish with the human and seven computer slots, all countries and colours on
  random, and a test rules file that lists at least eleven countries, marks the country
  registered fifth (index 4) passive and leaves the multiplayer listing flag off on the country
  registered seventh (index 6) and on for the other nine of indices 0 … 9. 200 games started from the lobby with scripted input (the
  scenario random stream continues from game to game, which is also where the draws come from, so
  several games per program run are fine). Observable: every house's country and colour at the
  first tick, 1600 country samples and 1600 colour samples. Expected: (i) every country has
  registration index 0 … 9; (ii) the country of index 4 (passive) and the country of index 6 (no listing flag) each appear at
  least once; (iii)
  the counts of the ten indices have a chi-square statistic against 160 each of at most 27.88
  (nine degrees of freedom, a 0.1 per cent false alarm); (iv) in every game all eight colours differ.
  Falsified if any country of index 10 or more appears (the range is wider), if index 4 never appears
  (passive countries are excluded) or index 6 never appears (countries without the flag are
  excluded), if the statistic exceeds 27.88 (the draw is not uniform on
  0 … 9, and a statistic far below 27.88 does not confirm the stream itself, which is S-0202's),
  or if two players of one game share a colour.
- *Part b.* Setup: as part a (the human and seven computer slots, the test rules file, scripted
  input), with the random-state recorder of the measurement setup; one run with every country and
  colour on random and one with every country and colour fixed, each on the same map with every
  start position on random (those are drawn from the game's own stream, S-0009 claims 18 to 20). Observable: the scenario
  stream's state when the game is seeded, and the number of raw values taken between the seeding and the
  first start-position draw, in each run. Expected: equal counts
  in both runs (the random choices take no draw of the game). Falsified if the count differs.
- *Part c (order of the draws).* Setup: the human and three computer players in lobby slots 1, 3
  and 5 (the other slots empty), all countries and colours on random, except that the colour of
  slot 3 is fixed to colour slot 6. 20 games, with the recorder reading the
  scenario stream's state just before the start button is pressed. Observable: that state, and
  every house's country and colour at the first tick. The test replays ranged draws from the
  recorded state, in the order of claim 7's paragraph "Order of the draws" (human country,
  human colour draws, slot 1 country, slot 1 colour draws, slot 3 country, slot 5 country,
  slot 5 colour draws; slot 3's colour is fixed and takes no draw, and each colour draw is
  rejected when the slot is taken), with
  S-0202's range rule. Expected: the replay predicts every country and colour of every game.
  Falsified if the replay fails in some game, whether or not an alternative order predicts all
  games; an alternative order that predicts all games (for example: all
  countries drawn before all colours, the computer slots before the human) also shows which order the original has; a colour
  draw not rejected for a slot that a computer slot shows is tested by the fixed colour 6 of
  slot 3: no player on random ever gets colour slot 6, including the human and slot 1, who are
  resolved before slot 3.

**K5: alliances at game start.**
- *Part a (teams).* Setup: a skirmish with four players: the human and computer A on team 1,
  computer B on team 2, computer C on no team. Observable: the in-game player list, and whether
  units fire at each other. Also observed: the message list during the first ticks. Expected: the human and A mutually
  allied, B and C enemies of everyone, no alliance message. Falsified if B or C start allied with
  anyone, the human and A fight, or an alliance message appears.
- *Part b (the special house).* Setup: a test rules file that gives the special country a test side
  that is not the civilian side (run 1), then the civilian side (run 2); a test map with an armed
  test defence building of the special house next to the human's start and a second, unarmed
  special-house building. Observable: whether the defence fires at the human's units, and whether
  the human's units on guard attack the unarmed building by themselves. Expected: the defence never
  fires in either run (the special house allies the human); in run 1 the human's units leave the
  building alone, in run 2 they treat it as the building of a house they do not ally, as the
  targeting rules give (S-0309). Falsified if the defence fires in either run, if in run 1 the human's units on guard attack the
  unarmed building, or if in run 2 they leave it alone.
- *Part c (the special house and the neutral house).* Setup: a test map with an armed defence
  building and an unarmed vehicle for each of the neutral house and the special house, placed so
  that each defence reaches the other house's vehicle and nothing reaches the human. Three runs
  with a test rules file that gives the two countries test sides: (1) neither on the civilian
  side; (2) the neutral country on the civilian side, the special country not; (3) the special
  country on the civilian side, the neutral country not. Observable: which defence fires at the
  other house's vehicle. Expected, by claim 8 and the targeting rules (S-0309): (1) neither fires
  (mutually allied); (2) the special defence fires at the neutral vehicle (the special house does
  not ally the neutral house) and the neutral defence does not fire at the special vehicle; (3) the
  neutral defence fires at the special vehicle and the special defence does not fire at the
  neutral vehicle. Falsified by any other pattern of fire.
- *Part d (no row and no general factors for the non-player houses, claim 5).* Setup: part c run
  (3), with the test rules file also giving the three difficulty rows the firepower factors 2.0,
  1.5 and 0.5 and the rate-of-fire factors 2.0, 1.5 and 0.5 (easy, normal, difficult row), the
  neutral country the general firepower factor 1.5 and the general rate-of-fire factor 1.5 (all
  exact in binary), and the special country every armour factor 1.0. The neutral defence's weapon
  is a test weapon of damage 8, versus 100 % against the special vehicle (strength 1000), with the
  rate-of-fire field 40; the defence is not veteran. Observable: the special vehicle's loss per
  hit, and the ticks between successive shots of the defence over 20 shots. Expected: a loss of 8
  per hit, and every interval 40, 41 or 42 ticks (*D*<sub>1</sub> of S-0301 Formula F1 with a house
  factor of exactly 1.0 and its draw *u* = R(0, 2)). Falsified if the loss is 12 (the country
  factor or the normal row alone), 18 (the normal row times the country factor), 16, 4, 24 or 6
  (other rows, with or without the country factor), or if any interval lies outside 40 to 42, in
  particular 60 to 62, 90 to 92, 80 to 82, 20 to 22, 120 to 122 or 30 to 32 (one of those factors
  applied to the delay).

**K6: the superweapon setting (granting).** Setup: a test rules file with a test building type T2
whose second superweapon is disableable and whose first is none, T3 with a disableable first
superweapon, listed in the tech-lab list (so S-0407 claim 7 lets it be built), and T4 like T3 but
not listed (the build bar is not in play for it: the test map places T4 as a neutral building
next to the human's start, since a human-owned map structure would be dropped, S-0009 claim 27); an
engineer of the human captures T4. Superweapon setting off. Observable:
which superweapon icons appear after building T2 and T3 and after capturing T4. Expected: no icon for
any disableable superweapon. With the setting on: every icon shown. Falsified if a disableable
superweapon's icon appears with the setting off, or if with the setting on an icon of T2, T3 or T4
is missing. Whether T1-like types (first superweapon
disableable, not listed) are offered is S-0407 scenario T7.

**K7: crates at game start.** Setup: test rules file with crate minimum 0 and crate maximum 8,
skirmish with the human and three computer players, crate setting on. Observable: the number of
crates at tick 1 (screenshot after revealing the map with a test trigger). Expected: 1. Then crate
minimum 3, maximum 8: expected 3; crate minimum 3, maximum 2: expected 2. With the crate setting
off: 0 and none ever after 10 game minutes. Falsified if the first run shows a count other than 1
(4 would mean all players are counted), if the second run shows a count other than 3 or the third one other than 2, or if any
crate is present with the crate setting off.

**K8: the mode that gives every base unit.** Setup: that game mode on a small map, a test rules file
whose base-unit list has three vehicle types V1, V2 and V3 in that order, each owned by one side
only; the human on a country of the side of V1; the other houses are computer players.
- *Part a (the list rule and the cells).* Observable: the vehicles the human owns at tick 1 and
  their cells. Expected: one of each of the three types; V1 on the cell three cells further along
  x and y from the base cell; V2 and V3 around the base cell within radius 3 or more. Falsified if
  the human receives only the type of its own side, or V1 is elsewhere while that cell is free.
- *Part b (the order).* Setup: as part a, on open flat ground with no pre-placed objects near
  the base cell. Observable: the three vehicles' cells at tick 0. Expected: V3 occupies the
  base cell (the search's initial candidate), V2 occupies the first accepting radius-3
  candidate, and V1 occupies the cell three further along x and y. Falsified if V2 occupies
  the base cell instead of V3, or the cells disagree with those predictions. A test object
  at V1's direct-placement cell does not by itself establish that the direct placement fails:
  S-0009 claim 46's loading-time placement bypass applies.
- *Part c (failure and early stop).* Setup: as part a on the map of S-0009 scenario M26 variant E
  (only the base cell can take a unit), with the list's second type made a type that cannot be
  created in the test (an empty list position, a name that matches no vehicle type), and unit-count
  setting above 0; the human is house 0, and a second computer player follows in house order.
  Observable: the vehicles and credits of both houses at tick 1. Expected: the human owns the vehicles
  of the list that could be created and placed (never the failed one) and no budget units, with no left-over credits added; the computer player owns no starting
  units at all. Falsified if the human owns budget units or the credits setting plus a left-over
  budget, or the computer player owns starting units.
- *Part d (the family selects the rule).* Setup: a loose copy of the mode list (placed before the
  program starts, as K1 run 2 places its rules copy) that files one mode under the standard family
  and one under the all-base-units family, both with an empty test game-mode file, on the map and
  rules file of part a. Observable: the vehicles the human owns at tick 1 in each mode. Expected:
  one of each of V1, V2 and V3 in the all-base-units mode, and the single base unit of S-0009
  claim 45 in the standard mode. Falsified if the two modes give the human the same vehicles.

- *Part e (draw sequence).* Setup: part b's open map, the random-state recorder, crate setting
  off, no secret labs, unit-count setting 0 and V1, V2 and V3 without visceroid flags. At game
  speed index 2, record 20 games from seeding through tick 0. Procedure: replay the starting-unit
  draws from the recorded state and the observed base cells, using claim 13 and S-0009's
  creation-draw and search rules. Observable: every vehicle's cell and the stream state after
  starting-unit generation. Expected: per house, V3's creation draw with no search draw on the
  free base cell, V2's creation and radius-3 search draws, V1's creation draw with no search
  draw at its direct-placement cell; after the last house, R(0, 65535). Repeat with part c's
  failed list position: the other entries of that house retain their draws, but no budget-unit,
  later-house or final draw occurs. Falsified if the predicted cells or stream state differ in
  any run. Record counts of raw values using S-0202's ranged-draw rule, not an assumption that
  every ranged draw uses exactly one raw value.

**K9: game start.** All parts use a test map and the random-state recorder of the measurement
setup.
- *Part a (the draw groups).* Setup: a test map with the crate setting on (crate minimum 3, so
  three crates), two secret labs on the map and a test rules file whose secret lists hold three
  entries in total; the human and one bystander computer player; a unit-count setting above 0 so that the
  starting units take draws; the special country left passive. Observable: the recorded draws from
  the game's seeding to tick 0, with stream-state checkpoints around starting-unit placement,
  initial crates and lab assignment. Controls turn crates off, omit labs, or set unit count 0.
  Identify each group's count from its own checkpoints, not a whole-load count difference:
  later conditional animation draws can also change when earlier groups change. Expected: the
  starting-unit draws precede initial crate draws, which precede lab draws. Falsified if these
  three groups occur in another order; a later tag or animation draw is not a falsifier.
  Repeat this part in the all-base-units family with the test base-unit list of K8: its
  starting-unit group follows claim 13 and can take different draws, but these three groups
  must retain the same order. A different group order falsifies claim 14 for that family.
  Use game speed 2, director and mods off, and record five runs of each variant without
  giving orders before tick 0. For the later groups, also run S-0202 R10 and its variants A
  and B, using that scenario's named theater fixtures and recorder checkpoints. Record the
  lab choices, tag-creation draws, and tile animations' cells and initial stage delays.
  Expected: the game-start tile-animation draws follow labs and end-of-loading tags, in
  visiting order, before tick 0; this ordering also holds with no starting units and with
  early-stop placement. Falsified by a tile-animation creation in an earlier group or
  deferred to tick 0, a reversed visiting order, or a creation draw count different from
  S-0202's animation-creation rule. No last-draw assertion is tested.
- *Part b (extra credits of computer player houses, claim 3).* Setup: a test rules file (a loose
  copy of the base rules, placed before the program starts, as K1 run 2 and K3 use one) whose
  extra-credit percentage list is 7, 30, 55, whose minimum unit count is 0 and whose credits slider
  has the minimum 2500, the maximum 10000 and the step 100 (so those are named test values); the human and
  three bystander computer players, one of each lobby entry (easy, normal, hard computer; so
  difficulty indexes 2, 1 and 0); credits slider at its minimum (2500, the copy's), unit-count setting 0, bases
  setting on, crate setting off, the special and neutral countries passive. Triggers of the test
  map, each owned by a map country and evaluated in the first tick that evaluates triggers,
  test the credits of a house against a threshold: for the human, thresholds 2499, 2500 and 2501
  (this calibrates whether the event means "more than" or "at least", the human's balance being
  2500, claim 2); for each computer house *h*, thresholds *E*<sub>h</sub> − 1, *E*<sub>h</sub> and
  *E*<sub>h</sub> + 1. The run is void for a house that spent, sold or lost credits before the
  first reading (the base unit of each house needs many ticks before any purchase). Expected
  *E*<sub>h</sub> = 2500 + extra with extra by S-0205's rounding: 174 for the hard computer
  (list entry 0, *p* = 7), 749 for the normal computer (entry 1, *p* = 30), 1374 for the easy computer
  (entry 2, *p* = 55): *E* = 2674, 3249 and 3874. The three triggers of each house must fire exactly
  as the human's calibration triggers do for a balance of 2500 (shifted by *E*<sub>h</sub> − 2500). Falsified if
  a computer's triggers show a different balance: 175, 750 or 1375 extra
  (round-to-nearest arithmetic instead of S-0205 claim 5), the extra of another list entry (the
  difficulty index of claim 4 is not the one given), no extra, or an extra on the human. Repeat
  with the slider at its maximum (10000, the copy's): extra 699, 2999 and 5499, *E* = 10699, 12999 and 15499.
- *Part c (a non-passive special house, claims 3 and 5).* Setup: the map of S-0009 scenario M26
  variant E (a local area that holds only base cell 0, so every placement away from base cell 0
  fails) with a third start waypoint marker outside the local area, and the same loose base-rules
  copy as part b with the bases setting off; the human at position 0, a bystander computer
  player with one pre-placed building of its own, and the special country made non-passive by a
  map override with one pre-placed building of the special house, one vehicle type and one
  infantry type with tech level 1 and the special country in their owner lists (as S-0009 scenario
  M16 arranges), so that its start pools are not empty. Unit-count setting above 0. The early
  phase sees the special country as passive (S-0009 claim 33), so the special house takes the one free position, outside
  the local area, in the late phase, as the neutral house does in S-0009 scenario M26 variant I
  (claims 28 and 29); the computer player means that the cleanup of claim 28 does not run.
  Compute *B* from S-0009 claim 45 with the test data. Every purchase of the special house
  then fails (S-0009 claim 45, variant E), so its left-over budget is *B*, its *M* is *B*, and
  its balance before the extra is *B*. Expected: the special house's credits equal
  *B* + trunc(RZ₅₃(RZ₅₃(30 × *c*) × *B*)), entry 1 of the list (index 1, claim 5); for *B* = 2500 that is
  2500 + 749. Observable: the credits of the special house, through triggers with thresholds
  one below, at and one above that value, calibrated against a trigger on the human's known
  balance as in part b. Falsified if the credits equal *B* alone (no extra credits for a
  non-passive special house), *B* plus the extra of entry 0 or 2 (the index is not 1), or *B* plus
  a round-to-nearest extra. Control runs: (1) unit-count setting 0, so *B* = 0: expected 0
  credits for the special house (*M* = 0); (2) the override removed so the special country is
  passive: expected 0 credits. Falsified if either shows credits above 0.

- *Part d (the cleanup before the starting units, claim 14).* Setup: S-0009 scenario M16's map,
  type overrides and neutral-country override, played as a skirmish with the human only (no
  computer player, fewer players than the maximum, so the cleanup runs), the bases setting off in
  a loose base-rules copy as in part b, and a unit-count setting above 0. Run 1 with the neutral
  override present (the cleanup removes the neutral house); run 2 without it (the neutral country
  stays passive and the house never receives units). Observable: the number of raw values taken
  between the game's seeding and tick 0, and the units around the neutral house's start position.
  Expected: the same count in both runs and no neutral unit in either (a removed house takes no
  starting-unit draws). Falsified if run 1 takes more draws than run 2, or shows any neutral unit.

**K10: the skill level of the non-player houses (claim 3).** Setup: S-0209 scenario C7 part (c)
(a one-cell-wide corridor; an infantry unit of the special house, a stock type that is not a
fraidycat (S-0107 Terms), the human's crusher; a computer player present so that the cleanup does not run), with
the special country made non-passive by a map override as in K9 part c, and with S-0209 C7's two
provisions (the stock scatter threshold is above 0 and at most the maximum skill level; the
infantry type has no scatter veterancy ability at the rank the map gives). Variant 2: the same
with the infantry a unit of the neutral house and the neutral country made non-passive. Control: C7
part (a) in the same rules file, the infantry owned by the computer player. Observable: whether
the infantry survives or is crushed. Runs: 3 each, counted as in C7. Expected: the infantry of
the special house and of the neutral house is crushed (level 0), and the computer player's
survives (the maximum level). Falsified if either non-player house's infantry survives (the house
then has the maximum level when non-passive), or if the control is crushed (then the provisions do
not hold and the run says nothing).

## Open questions

- Game modes other than the one of claim 13 may change set-up rules (one of them reveals the map at
  game start); they are not covered here.
- Whether *H* in claim 10 is the human count on every way a skirmish can start.

## Cross-spec hand-off

**Q-0192 is already resolved behaviourally in approved revision 4**, claim 14, its ordering
table and K9 part a. This revision records both questions as answered: tile-animation draws
of the game-start pass follow secret-lab choices and end-of-loading tag draws, and the table
lists those later groups. Approved S-0202 revision 10 claim 14 and S-0011 revision 7 claim 34
already give that order. Their current drafts and S-0009's current loading table agree.
Joint sign-off confirms S-0407 claim 14 and S-0009's joint hand-off checks against this
order; their current texts already agree, so no last-lab correction remains outstanding.
There is no assertion that the recalculation is the final loading-draw group:
S-0202 claim 14 also owns subsequent resource-glint draws. Neither question of Q-0192 is
left open. This ordering does not supply downstream particle or smudge draw counts and
must never be used to treat them as zero. Approved S-0202 revision 11 leaves those counts
unresolved; its current revision 12 draft claim 15 supplies creation-draw rules for joint
review, with scenario R14 proposed rather than measured. Sign-off must confirm that
extension and update any coordination note that still calls those creation counts unresolved.

S-0414 claims 6, 9 and 10 own the end-test and end-delay rules used by claim 11. Claims 3, 4
and 5 and scenario K10 remain unchanged from approved revision 1: S-0215's current revision
cites those approved rules, so that dependency needs no simultaneous hand-off. S-0009's current
claims 45 and 46 already scope their starting-unit rules to standard skirmish mode and defer
all-base-units outcomes to claim 13. Joint sign-off checks those scopes together with claim 13's
reuse of the shared search and claim 14's group ordering: the two families need not take the
same starting-unit draws. No further S-0009 scope edit is required. The earlier requests to reconcile the lobby
difficulty pairing and the non-player houses' skill levels have been resolved in the approved
specs and glossary.

Decided and applied: every house uses the normal row in a skirmish, a map's difficulty row is
ignored and only the optional handicap changes a row (claim 5; 2026-10-10); the neutral and
special houses keep the original's rules (claims 3 and 5; 2026-10-10); game end by "play to the
last side" (claims 10 and 11, Terms *human participant*; 2026-10-11).
