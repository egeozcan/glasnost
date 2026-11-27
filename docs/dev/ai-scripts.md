# AI scripts, difficulty and the tournament (#449)

Our own design under the lead's no-cheating rule: an AI house knows what a person in its seat
knows and gives orders a person could give. Nothing here is game behaviour; every number is
tuning of our own opponent or tooling (`design: own AI (#449)` or `nocite`).

## The contract

`glasnost_ai::scripts::AiScript` is what a script is:

- **In:** a `View` (the world as the house may read it: its own objects and what its house has
  explored, `View::sees`, the same test as the shroud filter of the default script) and a
  `Schedule` (the tick and whether this is one of the house's think ticks,
  `scripted::thinks_at`, every `THINK_PERIOD` ticks with the house's offset). It also gets the
  difficulty and the house's own living objects listed once a tick for all houses.
- **Out:** a list of `glasnost_sim::orders::Order`, issued through `orders::issue` like a
  person's. A refused order is not an error.

A script decides only on what it sees. `tests/scripts.rs`
(`every_script_decides_only_on_what_it_sees`) checks it for every script and every difficulty:
two worlds that differ only by an enemy unit on a cell the house has not explored give the same
orders over a think period, and a think tick falls within it.

## The registry

`scripts::REGISTRY` is fixed, ordered and compiled in. Each entry has a stable id (saved in setups,
replays and game saves, so an id is never reused), a key for the tools (`default`), a display
name for the setup screen and a line about it.

| id | key | name | about |
|---|---|---|---|
| 1 | `default` | Default | the M5b–M8 brain (`brain::Brain`), unchanged at normal |
| 2 | `siege` | Siege | the default's base without superweapons and one army sent as a strike (`siege::Siege`; see [Siege](#siege-script-as1)) |

An unknown id (a setup from a later build) plays the default script (`scripts::known`).

To add a script: give it the next id, add a variant to `scripts::ScriptState` that holds its state
(serializable, since it is saved and hashed), start it in `ScriptState::start`, implement
`AiScript` for its state, and add its entry to `REGISTRY`. The shroud test then covers it.

## State, saves and the hash

Each AI house's `scripts::Seat` holds its choice (`AiSeat`: script id and difficulty) and its
script's state. `Game::save` and `Game::load` keep the seats (AI game format 139), and
`Game::state_hash` hashes the world with the AI state. A game in which every seat is the default
script at normal hashes its brains exactly as before (the bytes of the brain list), so the
goldens did not move; any other choice hashes the seats, choice included.

The choice lives in the setup: `GameSetup.ai`, one `AiSeat` per house in house order, empty by
default (`serde(default)`; a missing entry is the default script at normal). The simulation
carries it and never reads it. A replay holds the setup (save format 161), so a replay plays the
same scripts at the same difficulty again. The skirmish setup screen has a Script and a Difficulty
cell on each computer row, and writes the choices only when one is not the default; the lobby's
choices edit carries a computer row's choice over the wire (wire version 13); the skirmish lane's
AI modes take `GLASNOST_AI_SEATS=default:easy,default:hard`.

Postcard is not self-describing, so `serde(default)` does not make an older replay's bytes load;
an older replay is refused by its version, as before. A setup with no choices, or with fewer
entries than houses, plays the default at normal (`a_setup_without_choices_plays_the_default_at_normal`).

## Difficulty

Three levels: easy, normal and hard. Normal is today's play. A level changes only how well the
script plays, never a house's rules, credits or factors.

What the default script does with it, and why a player sees it:

| level | harvesters (plan, at most) | attack wave | acts on | base defence | superweapon and special buildings | units sent alone | what a player sees |
|---|---|---|---|---|---|---|---|
| easy | 1, at most 2, on the home field only | 3 units | one think tick in three (`EASY_EVERY`) | on its acting think ticks only | none built, none fired (its country's own included) | about a fifth of the army's spend (`SPENT_SHARE`) | one or two harvesters and no refinery by a new field, small attacks, a base that grows and answers a raid slowly, no superweapon strikes or paradrops, no sensor, grinder, gattling defence, bunker, lifter or drainer |
| normal | 2, at most 4 | 6 units | every think tick | every tick | built | about a fifth of the army's spend (`SPENT_SHARE`) | today's AI |
| hard | 3, at most 6 | 6 units | every think tick | every tick | built | about a ninth of the army's spend (`HARD_SPENT_SHARE`) | a third harvester (up to six on a large field), so more money and a larger army sooner, and fewer units thrown away alone |

The plan's count is the harvesters kept; on a large explored field the house buys one more for
every 20 resource cells within reach, up to twice the plan's count (`Script::harvesters_wanted`;
a Yuri house's slave miners are not capped), so the plan counts 1, 2 and 3 are on stock maps
often 2, 4 and 6 harvesters. `brain::plan_for` gives the plan of each level; the think rate, the
defence gate, the skipped building parts and the skipped superweapon launches are in
`Brain::think_from`, read from the tick and the level alone (no state is added, so normal hashes
and plays as before). The home-field rule is a flag of the think's memo
(`Script::stay_on_home_field`), set by `think_from` at easy; `Script::field_refinery` answers no
under it. A refinery the house lost is still rebuilt (`Script::refinery_lost`). Without the rule
the plan's count did not hold at easy: each refinery by a new field brings a harvester.

**Units sent alone (batch 12).** Some army types never join a wave: an engineer, a type with a
bomb, a suicide weapon or explosive charges, and an air-strike designator
(`scripted::spent_alone`). Their special attacks and the engineers' step give each such unit its
own order, and it is lost on use. The army round offers them only while the house's spend on
them (ever built, times the type's cost), times the share, is under its spend on the rest of its
army, and only where a kind has another type to build. Share 4 (`SPENT_SHARE`) holds them to about a fifth of
the army's credits. At hard, `Brain::think_from` sets the memo flag `Script::spend_less_alone`,
and the share is 8 (`HARD_SPENT_SHARE`), about a ninth. Without the limit, such a type was picked about as often as
any fighting type. The round ranks by fewest living, and a spent-alone type has none on the map
once used. Spent-alone units also cost more each. A side whose list holds several such types
spent much of its army there (below).

### Telling the levels apart (batch 10)

Rounds on the install, `ai-tournament game=install maps=0,2,8 seeds=7-10 ticks=27000 threads=2`:
144 games, 48 per pair, 16 per pair and ground, 22 to 25 minutes each. B9's 24 games (8 per
pair) were too few. Head-to-head score of the stronger level of each pair (win 1, draw ½):

| round | easy v normal: normal | easy v hard: hard | normal v hard: hard | Elo easy / normal / hard |
|---|---|---|---|---|
| base (f62d868, before the batch) | 0.53 (16-19-13) | 0.59 (17-23-8) | 0.66 (23-17-8) | 1512.7 / 1439.4 / 1547.9 |
| round 1 (easy acts on one think tick in three) | 0.56 (20-14-14) | 0.55 (17-19-12) | 0.58 (21-14-13) | 1535.9 / 1458.2 / 1505.9 |
| round 2 (and no superweapon or special buildings at easy) | 0.56 (20-14-14) | 0.61 (24-11-13) | 0.58 (21-14-13) | 1534.3 / 1445.5 / 1520.2 |

- **The side decides.** In every pair and round the entrant on country index 0 scores
  0.56 to 0.75 and on country index 1 0.04 to 0.31. Since each pair plays both countries
  equally, the stronger level's lead on the same country is 2S − 1 for its overall score S: the
  split by side adds nothing to S.
- **The spread.** Normal v hard is not touched by the levels of this batch, yet moved from 0.66 to
  0.58 between the base and round 1 (the batch's spies and bombers change play at every level), so
  about 0.08 is within the spread of one round. Rounds 1 and 2 give normal v hard game for game
  the same (the levels do not reach normal or hard).
- **Not separated.** Hard leads normal (0.58 to 0.66) and easy (0.55 to 0.61), but easy and normal
  are not told apart (0.53 to 0.56) though easy now thinks at a third of the rate and builds no
  superweapon: every easy game changed, its results hardly. Easy's wins come late (at the cap on
  worth, or by victory after tick 15000), not from an early rush of its small waves. What decides
  a game between two levels of this script is still unknown; the Elo order depends on the order
  of the games and is not the measure.

### Telling the levels apart (batch 11)

The same round as batch 10 (144 games, about 24 minutes, the size of the base round so that the
tables compare). The criterion was fixed before the first round: each pair's stronger level
scores at least 0.60, and the Elo order is hard, normal, easy. The final build then plays the
same round on seeds 11 to 14, which it had not been tuned on, and must pass the same bar.
Stronger level's score (won-drawn-lost):

| round | easy v normal: normal | easy v hard: hard | normal v hard: hard | Elo easy / normal / hard |
|---|---|---|---|---|
| base (f393424) | 0.552 (19-15-14) | 0.615 (24-11-13) | 0.667 (23-18-7) | 1543.0 / 1418.8 / 1538.2 |
| E1 (easy fires no superweapon) | 0.635 (21-19-8) | 0.719 (30-9-9) | 0.667 (23-18-7) | 1491.3 / 1445.0 / 1563.7 |
| E2 (and stays on its home field) | 0.812 (34-10-4) | 0.823 (35-9-4) | 0.667 (23-18-7) | 1347.1 / 1521.6 / 1631.3 |
| E2 on seeds 11-14 | 0.750 (32-8-8) | 0.750 (34-4-10) | 0.646 (25-12-11) | 1393.3 / 1499.9 / 1606.8 |

- **What held easy up.** The base round split by country: normal lost 11 games to easy by
  victory, 10 of them with easy on country index 0, whose country paradrop easy still fired (B10
  skipped only the superweapon buildings). E1 takes that away. E1 then passed the three pair scores
  but not the Elo clause (easy 1491.3 above normal 1445.0, though normal scored 0.484 over its
  games and easy 0.323): the sequential Elo is noisy at this size, but the clause stayed.
  A dump of one easy-v-normal game showed why easy kept up on worth: it ended with five
  refineries and five harvesters, against a plan of one, since each refinery by a new field brings
  a harvester. E2 keeps easy on its home field.
- **Normal and hard are untouched.** Normal v hard is game for game the same in all three rounds
  on seeds 7-10. The quick lane, the sweeps, the arena and the bench play only normal and do
  not move.
- **The country still shows.** In E2 the stronger level on country index 0 scores 0.875 to 0.958,
  and on country index 1 0.375 (hard over normal) to 0.771. Hard on country index 1 still loses
  to normal on country index 0 (see below).

### The two countries (batch 11)

The default script on country index 1 loses heavily to itself on country index 0. Normal mirror,
`ai-tournament game=install entrants=default:normal,default:normal maps=0,2,8 seeds=7-14
ticks=27000 threads=2`, 96 games, each start and each country equally:

- **Country index 0 scores 0.844** (63 won, 32 drawn, 1 lost).
- **Its country paradrop is half the gap.** In a probe build where the country paradrop is never
  fired, country index 0 scores 0.667. The paradrop is a rules asymmetry that a person in the same
  seat has too, so #449 allows it, and normal keeps firing it.
- **The rest is play, not the judge.** In the probe, country index 0 wins 11 games on maps 0
  and 2 by victory against 3, and 25 against 3 at the cap. Map 8 is even (0.531). Scoring the capped games on
  army and aircraft worth alone, with buildings left out, gives 0.714, not nearer 0.5. On map 2
  country index 1 holds half the army worth at the cap (11.9k against 23.2k). In the base lane it
  never sends a first wave while country index 0 does. In one dump it lost about 2.2 times the
  army units (82 against 37), 9 of them to aircraft. Its side fields no army aircraft, while
  country index 0 holds 3k to 4.5k of aircraft at the cap.
- **One fix tried, no effect.** While the house saw more enemy army aircraft than it had army
  units that hit them, each factory's pick went to an anti-air type. The probe mirror went from
  0.667 to 0.656, within one standard error of about 0.05. It was not shipped. Why country index 1
  fields half the army was left open (answered in batch 12, below); no simulation claim was found.

### The two countries (batch 12)

The same mirror and probe on base 2ce4555, results as country index 0's won-drawn-lost. The
normal mirror scores 0.781 (55-40-1) and the paradrop-off probe 0.620 (37-45-14); B11 had 0.844
and 0.667 on its base.

- **Where the money went.** In the probe, both countries bought about the same army (20995
  against 20646 credits a game), fired a first wave alike, and lost the fighting ground types at
  the same rate. They differ in what they bought. Country index 1 spent 45.7 % of its army's
  credits on units sent alone (`spent_alone`), and country index 0 spent 21.9 %. Country index 1's list holds four
  such types: a bomb planter, a suicide vehicle, an air-strike designator and an engineer. Country
  index 0's holds an engineer and two infantry types with charges. Its third line, army aircraft,
  survives. So country index 1 stood with 5529 credits of army a game against 8867.
- **The fix.** The spent-alone share (above) takes the probe from 0.620 to 0.516 (26-47-23), under
  the 0.60 bar fixed before the round. Map 0 goes 12-11-9, map 2 13-13-6, map 8 1-23-8.
  - Country index 1 now sends 2.99 waves a game (1.77 before) and stands with 6799 credits of
    army.
  - Its spent share is 0.247 and country index 0's 0.191. The share can pass a fifth because the
    limit is checked before a pick, not after the pick's cost, and a kind with no other type is
    not limited.
  - The normal mirror goes from 0.781 to 0.771 (56-36-4). There the country paradrop, a rules
    asymmetry that #449 allows, still decides.

### Telling the levels apart (batch 12)

B11's round, rerun because normal changed: 144 games on seeds 7 to 10, then the kept build on seeds
11 to 14. Same criterion: each pair's stronger level scores at least 0.60, and the Elo order is
hard, normal, easy. Stronger level's score (won-drawn-lost):

| round | easy v normal: normal | easy v hard: hard | normal v hard: hard | Elo easy / normal / hard |
|---|---|---|---|---|
| base (2ce4555) | 0.719 (28-13-7) | 0.781 (34-7-7) | 0.594 (21-15-12) | 1390.9 / 1545.6 / 1563.5 |
| the spent-alone share | 0.812 (33-12-3) | 0.812 (34-10-4) | 0.531 (14-23-11) | 1317.7 / 1571.8 / 1610.5 |
| H1 (and share 8 at hard), kept | 0.812 (33-12-3) | 0.812 (36-6-6) | 0.552 (18-17-13) | 1305.0 / 1562.0 / 1633.0 |
| H2 (and a plan of 4 harvesters at hard), dropped | 0.812 (33-12-3) | 0.771 (36-2-10) | 0.594 (19-19-10) | 1338.8 / 1539.6 / 1621.6 |
| H1 on seeds 11-14 | 0.781 (32-11-5) | 0.812 (36-6-6) | 0.531 (15-21-12) | 1391.8 / 1498.5 / 1609.7 |

- **Not met for hard over normal.** The other two pairs and the Elo order pass in every round.
  Hard over normal was already under the bar on this base (0.594; B11's head had 0.667).
- **The country again.** With H1, hard on country index 0 beats normal 16-7-1, and on country index 1
  goes 2-10-12. On seeds 11 to 14 the results are 14-9-1 and 1-12-11. In the seeds 7 to 10 round,
  hard on country index 1 lost 8 games at the cap on worth and 4 by kill. What is left is hard on
  country index 1 against normal on country index 0, which is the country gap again.
- **Two more tries, not kept.**
  - **H2: a plan of four harvesters at hard.** It cost hard 9 of its 24 games against easy on
    country index 0 (7 by kill), against 3 with H1. The army is held back while the house has
    fewer harvesters than the plan, so it waited for the fourth.
  - **Try 3: four harvesters wanted at hard, beside the army.** It does not bind where the field
    is large, since normal already orders four there. It was dropped before any commit.
  - The levels were handed back to the lead with H1 kept.
- **Normal moved this time.** The quick lane, the M6c sweep, the bench and the M6c duel golden
  moved, since they all seat normal. 19 quick-lane games changed winner, and each was traced to
  its first differing tick: an infantry pick of the army round. The arena, the M7a and M7b sweeps
  and the other eight goldens did not move.

**The house difficulty row is not touched.** Every player house in a skirmish, person or AI, uses
the normal row: S-0302 Terms (*Difficulty row*) and claim 3 ("AI difficulty comes from the AI's
strategy modules and never from stat factors"; the handicap is never tied to an AI house's
difficulty), S-0301 claim 3 (the house rate-of-fire factor), S-0412 claim 5 (the handicap is
off by default and never set by AI difficulty) and the glossary's *difficulty skill block*. The
specs give a computer-controlled house no other row, so no spec request was needed.
`the_difficulty_row_is_a_persons_at_every_ai_difficulty` checks it: an AI house and a person's
house of the same country have the same difficulty index, handicap, rate-of-fire and firepower
factors and prices at every AI difficulty.

## Siege (script AS1)

`glasnost_ai::siege`, registry id 2, key `siege`. Our own second script, beside the default and
sharing nothing with it but what it calls: it runs the default's brain (`Brain`) for the base on
its own plan, and keeps the army for itself. Every number is tuning (`design: own AI (#449)`).

**What it does.**

- **The base** is the brain's: build plan, harvesters, base defence, special abilities. The plan
  holds the brain's waves (`HELD`), so the brain never sends one.
- **No superweapons** at normal and hard: the brain's orders for a building that gives a
  superweapon (S-0512 Terms) are dropped, and since the brain orders such a building in place of
  its tech-climb building, the climb building is ordered instead (`Siege::no_superweapons`). A
  brain that owes money for a superweapon building holds its army lines while its base is safe;
  a house that keeps its army at home is safe more often, so it saved and starved its army.
- **One army.** Idle army units gather at home. At the level's count (`strike`) they go out as
  one strike, minus a guard nearest home, with attack-move orders: first, at hard, to the seen
  enemy harvester or refinery nearest home (`economy`), then from building to building of the
  enemy house nearest home, then to an enemy start not yet seen empty. New units join the strike
  in groups (`join`).
- **The edge (hard).** The house remembers the enemy army units it has seen (`Army::remember`: by
  their cost, S-0403 claim 2; one is forgotten when the house sees its last cell without it, or
  after `MEMORY` ticks). A strike goes only when the units that would go are worth 120% of that;
  after 3000 ticks without a strike, 80%. A strike worth less than half of it is called home with
  plain moves, and the army gathers again.
- **The hunt (hard).** Once every enemy start has been seen and no enemy building is in sight,
  each unit goes to the nearest seen enemy, else the nearest cell the house does not see, so the
  game ends by victory (S-0414 claim 3) and not at the tournament's cap.

**Fair play.** It reads the house's view only: its own objects, the enemies it sees
(`Script::seen_enemies`, the default's shroud filter) and what it remembers of them, the cells it
has explored or sees, and the map's start cells (lobby knowledge, as the default). It gives move,
attack-move and build orders, on its house's think ticks, and its house has a person's rules and
money. `every_script_decides_only_on_what_it_sees` covers it with every registry entry.

**Levels.**

| level | harvesters (plan) | superweapons | strike / join / guard | acts on | edge, recall, economy first, hunt |
|---|---|---|---|---|---|
| easy | 1 | the brain builds none at easy | 6 / 6 / 1 | one think tick in three | no |
| normal | 3 | none | 12 / 6 / 3 | every think tick | no |
| hard | 3 | none | 10 / 3 / 2 | every think tick | yes (120%, 50%; 80% after 3000 ticks) |

**Why hard wins against the default.** It spends nothing on superweapons, so its army lines run;
it keeps its army together and sends it only with the edge, so the default's waves meet the whole
army at home and are lost; its strike goes for the harvesters first; and it hunts, so most of its
wins are victories. Development rounds against `default:hard` (maps 0, 2 and 8, seeds 7 to 10,
48 games; score of siege:hard, win 1, draw ½):

| round | change | W-L-D | score | country 0 / 1 | victories / worth |
|---|---|---|---|---|---|
| dev-1 | gathered strike, more harvesters | 13-13-22 | 0.500 | 0.771 / 0.229 | 5 / 8 |
| dev-2 | no superweapons | 29-12-7 | 0.677 | 0.812 / 0.542 | 26 / 3 |
| dev-3 | the climb building in their place | 28-10-10 | 0.688 | 0.833 / 0.542 | 25 / 3 |
| dev-4 | remembered enemy army: edge 150%, recall 50% | 30-9-9 | 0.719 | 0.917 / 0.521 | 23 / 7 |
| dev-5 | economy first, edge 120% | 33-4-11 | 0.802 | 0.938 / 0.667 | 25 / 8 |
| dev-6 | the edge judged on the goers only | 36-3-9 | 0.844 | 0.958 / 0.729 | 29 / 7 |
| dev-7 | enemy aircraft not counted (dropped) | 36-4-8 | 0.833 | 0.958 / 0.708 | 27 / 9 |
| dev-8 | the costliest vehicle type (dropped) | 23-11-14 | 0.625 | 0.792 / 0.458 | 14 / 9 |
| dev-9 | patience, the levels reworked (kept) | 37-4-7 | 0.844 | 0.917 / 0.771 | 29 / 8 |

The levels among themselves (the same grounds, 144 games): with normal building superweapons,
easy beat normal (normal 0.406) while hard led normal 0.833 and easy 0.760; with normal's
superweapons dropped and easy on one harvester and strikes of six, the order holds: normal leads
easy 0.635, hard leads normal 0.542 and easy 0.625.

### Hard, script AS2

Script AS2 took the siege's hard level further toward 2000 Elo, with the same rules of fair play.
Its baseline is AS1's certifying round on the base: `siege:hard` at 1801.2 (163-37-40). Its
losses on map 0 were mostly early: twelve of 23 ended between ticks 6 000 and 9 300, ten of them
against `siege:easy`. A watch of one (`siege-watch`) showed why. The brain builds no army while
its plan's harvesters do not stand (`short` in the army round), and hard's plan keeps three. So
hard stood with no army unit until after tick 4 500, and easy's first strike, five infantry at
about tick 3 500, took its base apart.

- **The early guard, tried and dropped.** Hard only: while the house had fewer than six army
  units on the ground and the brain ordered no infantry, the siege ordered the cheapest infantry
  army type not sent alone, with the money in hand beyond what the running items owed. In the
  development round it took the easy rush away (42-0-6 against `siege:easy`) and added eight
  victories, but its pair with `default:hard` fell from 0.740 to 0.635 (map 2: 3-4-9). A watch
  of one of those games (map 2, seed 10) showed hard at two refineries all game with the guard
  and at six or seven on the base: the money the guard spent early was the money of the
  refineries by the new fields. It was reverted.
- **More factories when rich** (`Tuning::rich`, hard only). The same watch on the base showed
  hard from tick 19 500 on with 3 000 to 9 566 credits in hand and an army of 6 to 12. Only one
  infantry and one vehicle line ran, and with no superweapon to save for, nothing else drew on
  the money. Each more factory of a kind shortens the build times of that kind (S-0403
  claim 6), as a person with money to spare builds a second war factory. With at least `rich`
  credits in hand, the siege orders the cheapest factory it can build, place and pay for, of
  the kind (infantry or vehicles) it owns fewer of, vehicles on a tie, up to three of each
  kind. The first factory of each kind stays the brain's own plan. It does so only when no
  building is ordered this think and the building line is free. When the new factory would
  leave the house short of power, it orders the power plant the plan builds next instead, as a
  person builds the plant first (on the synthetic skirmish every factory was held so, the
  brain building plants only once the house is short).
- **A strike at a harmless building, checked.** An AI house counts as human-controlled, so its
  units' own target searches pass an enemy building with no weapon (S-0309 table C, row
  "harmless building"). The strike might then stand beside an enemy base without firing. A test
  shows it does not: once the strike's units arrive they stand idle, and the brain's army part
  orders each to attack the seen enemy it can hit. No change was needed.
- **Where it stands.** The certifying round (seeds 15 to 18, the six entrants) put `siege:hard`
  at 1734.0 (1715.2 order-free), top of the pool at a score of 0.790 (168-29-43, 115 victories),
  but short of 2000. Its weakest pair is `siege:normal` (0.688) and its weakest country is
  country 0 (0.679, against 0.900 on country 1).

### Hard, script AS3

Script AS3 took the siege's hard level further toward 2000 Elo, with the same rules of fair play.
Its baseline is the certifying round on its base: `siege:hard` at 1739.5 (1702.6 order-free),
166-34-40. Of its 34 losses, 29 came with country 0, and on map 0 with country 0 it lost to
`siege:easy` 7 of 8, to `siege:normal` 8 of 8. Two watches (`siege-watch`, map 0, seed 8) showed
the house with no army unit until after tick 5 000: the brain's army round waits for the plan's
three harvesters, and against `siege:easy` the second harvester was lost, so hard stood with
5 700 credits in hand and nothing to fight `siege:easy`'s first strike of five infantry at
tick 3 500. Against `siege:normal` it stood at tick 6 000 with an army worth 1 400 to the
enemy's 2 500, whose strike of 14 then took its base.

Development rounds play `siege:hard` against each of the five other entrants (maps 0, 2 and 8,
seeds 7 to 14, 96 games a pair). The baseline round, dev-0, scored 0.803 (346-55-79, 215
victories).

- **No navy while the enemy is reached by land, tried and dropped.** Hard only: while every
  enemy start lay in the reach region (S-0212 claim 4) of the house's ground vehicles, the
  brain's orders for a naval building (S-0406 claim 3 d) were dropped and the climb went on
  without them. The watch showed a ship later in the game, gone with the change, but the two
  pairs played (`siege:easy`, `siege:normal`) came out game for game as dev-0 (0.797, 0.646).
  It was reverted.
- **More factories only after the first strike, tried and dropped.** The fourth factory at tick
  3 500 in the watch was a second war factory: a house starts with more than `rich` credits, so
  AS2's more factories when rich ordered one while the brain still held its army. With the
  factories held until the first strike had gone, the pair with `siege:normal` fell from 0.646
  to 0.557: the early second war factory pays later. It was reverted.
- **The guard when threatened, tried and dropped.** Hard only. AS2's early guard trained
  infantry whenever the house had few army units, and spent the money of the refineries by new
  fields on map 2. This one trains only while the house is threatened and the brain holds its
  army: while the enemy army the house remembers (`Army::seen`) is worth more than its own army
  on the ground, and it has fewer harvesters than the plan, it orders the cheapest infantry army
  type it can build that it does not send alone, on a free line, with money in hand beyond what
  its running items owe, when the brain orders no infantry. A person who sees the enemy's
  soldiers at the gate trains soldiers; one who sees none builds the economy. The round came out
  at 0.807 (348-53-79, 222 victories): `siege:easy` rose from 0.797 to 0.823 and `default:easy`
  gave six more victories, but the other three pairs played game for game as dev-0, since by the
  time those enemies strike the harvesters stand. That is short of a clear gain, so it was
  reverted.

Three development rounds in a row brought no clear gain, so script AS3 stopped there with the
hard level as AS2 left it: no code of AS3's is kept. What the rounds showed is where the next
gain lies. Hard's losses are early losses with country 0, mostly on map 0, before its first
strike, while it waits for its third harvester. A lever that answers them has to field an army
sooner without the early second war factory going, and the guard shows that soldiers alone,
trained only while threatened, come too few and too late.

### Hard, script AS4

Script AS4 took the siege's hard level further toward 2000 Elo, with the same rules of fair play.
It kept three hard-only levers and reached 1762.2 (1748.4 order-free) in its last certifying
round, short of the goal.
Its baseline is AS3's: the simulation and the AI have not changed since, so AS3's certifying
round (`siege:hard` 1739.5, 1702.6 order-free) and its development round dev-0 (0.803) stand.
Development rounds are AS3's: `siege:hard` against each of the five other entrants on maps 0, 2
and 8, seeds 7 to 14 (480 games).

- **Soldiers beside the harvesters** (`Tuning::beside`, hard only). While the house keeps fewer
  harvesters than its plan, the brain's army round waits for them, whatever the money in hand
  (AS3's watch: 5 700 credits and no soldier at tick 3 000). A person with that money trains
  soldiers at the barracks while the war factory builds the harvesters. So while the plan's
  harvesters do not stand, the siege orders one infantry army type a think on the free infantry
  line when the money in hand, beyond what the running items owe, pays for the next harvester
  and the soldier both (with no harvester type to build yet, the cheapest war factory stands in
  for it). The type is the one the house has fewest of on the map, then has built fewest of,
  then the cheapest, and never one it sends alone (`scripted::spent_alone`). It is the brain's
  own wealth test for a harvester beyond the plan (`rich` in the army round), applied to the
  plan's count; the vehicle line is left to the harvesters. AS2's early guard and AS3's guard
  when threatened both trained soldiers whatever money the harvesters needed; this one spends
  only what is left over.
- **A floor under the first strike** (`Tuning::floor`, hard only). The edge is a share of the
  enemy army the house remembers, and early in a game it remembers none, so the edge holds for
  any strike. With the soldiers beside the harvesters the strike's count of ten stood by tick
  6 000 (a dev-1 watch, map 2 seed 8: eight dogs and soldiers out, the strike lost and called
  home by tick 9 000), where the base's first strike went near tick 10 500 with tanks. So at
  hard a new strike also needs the units that would go to be worth 5 000 credits at least; the
  soldiers wait at home as a guard until the war factory's units join them. Joining a strike
  under way is not held.
- **No navy while the enemy is reached by land** (`Tuning::navy`, false at hard only). AS3
  tried dropping the naval buildings and found that it did not bind. With the two levers above,
  a dev-2 watch (map 0 seed 7, country 0, against `siege:normal`) shows hard ahead at tick 12 000
  (17 army units to 2). From tick 10 500 on its building and naval lines go to a shipyard,
  destroyers and a naval transport, while it has one refinery. It lost its land army and then
  its base by tick 24 600, with three destroyers left. So at hard, while every enemy start lies
  in the reach region (S-0212 claim 4) of the house's first ground vehicle, the brain's orders
  for a naval building (S-0406 claim 3 d) and for a unit whose movement zone is water are
  dropped. A dropped building gives way to the climb building as with the superweapons. With no
  ground vehicle, or an enemy start across the water, the navy stays.

The rounds, with `siege:hard`'s score against each pair (win 1, draw ½):

| round | `siege:normal` | `siege:easy` | `default:easy` | `default:normal` | `default:hard` | all | victories |
|---|---|---|---|---|---|---|---|
| dev-0 (base) | 0.646 | 0.797 | 0.917 | 0.828 | 0.828 | 0.803 | 215 |
| dev-1 (soldiers) | 0.630 | 0.901 | 0.922 | 0.844 | 0.745 | 0.808 | 241 |
| dev-2 (and the floor) | 0.667 | 0.927 | 0.901 | 0.849 | 0.792 | 0.827 | 246 |
| dev-3 (and no navy) | 0.729 | 0.932 | 0.911 | 0.854 | 0.802 | 0.846 | 257 |

- dev-1 was no clear gain: 26 more victories, but `default:hard` fell 0.083. Its watch is the
  floor's reason above. The soldiers were kept, since the floor builds on them.
- dev-2 was a clear gain on dev-0: +0.024 and 31 more victories, with no pair down more than
  0.036.
- dev-3 rose on every pair against dev-2, +0.019 and 11 more victories. That is just short of
  both bars, so it was no clear gain by the rule. The lever was kept anyway: it only drops
  spending, and no pair fell.

Certifying rounds (seeds 15 to 18, the six entrants, 720 games):

| round | code | Elo | order-free | W-L-D | score |
|---|---|---|---|---|---|
| cert-0 (AS3, base) | df22a12 | 1739.5 | 1702.6 | 166-34-40 | 0.775 |
| cert-1 | soldiers and the floor | 1761.9 | 1729.5 | 172-30-38 | 0.796 |
| cert-2 | and no navy | 1762.2 | 1748.4 | 177-25-38 | 0.817 |

The goal of 2000 was not met. At the cap, with the pool's ratings, 2000 needs about 0.93
against every entrant. cert-2 has `siege:easy` at 0.948 but `siege:normal` at 0.667 and
`default:hard` at 0.792. What remains:

- Country 0 against `siege:normal` (79-18-23 with country 0 in cert-2, 98-7-15 with country 1).
- Map 2, where most games end in a draw at the cap (44-10-26). A base watch there shows hard on
  two refineries all game against four.

The batch handed back one round into the stop count, to leave room for the final checks.

### Hard, script AS5

Script AS5 took the siege's hard level further toward 2000 Elo, with the same rules of fair play.
Sim batch S50 changed how vehicles path after AS4 measured, so AS5 took its own baseline on its
base: the certifying round cert-0 put `siege:hard` at 1808.7 (1794.9 order-free), 190-32-18,
score 0.858; the development round dev-0 (AS4's: the five other entrants, maps 0, 2 and 8, seeds
7 to 14, 480 games) scored 0.849 (369-34-77, 270 victories). Map 2 is the weak ground (98-11-51,
0.772): there hard draws at the cap even against `default:easy`, and `default:hard`'s ten wins
against it all came at the cap on worth, six of them on map 2.

- **More refineries** (`Tuning::refineries`, hard only). A watch of a game `default:hard` won on
  worth (map 2, seed 7) shows hard on one refinery until tick 10 500 and two after, with four
  harvesters and its credits near 0 all game, while `default:hard` stood on three refineries by
  tick 7 500 and seven by 21 000. Each refinery comes with a harvester (S-0404 claim 2, S-0402
  claim 18), and hard's building line stood idle most of the game while its army lines took every
  credit. The brain builds a refinery by a new field only once the field by its refineries runs
  low (`Script::field_refinery`). A person who sees his credits run out with fields unworked
  builds the next refinery early. So at hard, once the plan's harvesters stand and the house owns
  a refinery, while it owns fewer than four and the brain orders no base building this think
  (construction yard, power plant, refinery or factory), the siege orders the cheapest refinery
  type it can place (`Script::place_refinery`: beside the field of `Script::ore_goal`) on the
  free building line, the plan's next power plant first when the refinery would leave the house
  short of power, as with the factories when rich. The refinery takes the place of the brain's
  other building order of that think (its tech climb, a defence or a sensor): a rich brain keeps
  its building line busy, and when the line frees it orders the next building at once.
  Development round 1: 0.912 against dev-0's 0.849, every pair up (`default:hard` 0.786 to
  0.927); cert-1 1912.1 (order-free 1885.9) against cert-0's 1808.7.
- **Refineries before factories** (`Tuning::rich_after`, hard 2). Of dev-1's 19 losses to
  `siege:normal`, 18 came on country 0. A watch of one (map 0, seed 11) shows hard spending its
  starting money on more factories (`Tuning::rich`, 3000 credits in hand), its credits near 0 from
  tick 5 000, one refinery until tick 9 000, and its base lost at 11 135. The starting money is
  not income. A person puts it into refineries first and builds a second war factory once the
  income stands. So at hard, money in hand counts as rich only once the plan's harvesters stand and
  the house owns two refineries. More refineries are ordered before more factories. A factory added
  when rich takes the place of the brain's other building order, as the refinery does (shared
  `base_ordered`). A rich brain orders a building every think its line frees, so without that
  rule more factories had fired only before the economy stood, which is just what this lever
  stops. A test trial showed it: on four refineries and 30 000 credits the house added no factory.
  Development round 2: 0.919 against dev-1's 0.912, with 293 victories against 265 (`siege:normal`
  0.724 to 0.771); cert-2 1914.9 (order-free 1894.8).
- **Tanks first** (`Tuning::tanks_first`, hard only). dev-2's draws are stalemates at the cap at
  high worth, hard ahead but under 3/2 (about 40 000 against 30 000). A watch of one (map 2,
  seed 11, against `default:normal`) shows hard on seven refineries with its credits near 0 and
  an army of 31 units worth 11 300, mostly cheap infantry (dogs, rifles, gunners) with one or
  two tanks. It made three strikes all game, and each was called home. The brain picks the army
  type the house has fewest of on the map, so its fast infantry line builds as often as the
  war factory and takes the money. A person facing tanks and base defences puts most of his
  money into tanks. So at hard, while the house owns a vehicle factory and has spent more on
  infantry army units than on vehicle army units (S-0407 claim 10's ever-built counts at the
  types' costs), the siege drops the brain's orders for infantry army units, and the money
  goes to the war factory. The soldiers trained beside the harvesters (`beside`) stay.
  Development round 3: 0.914 against dev-2's 0.919, with 294 victories against 293
  (`default:normal` 0.906 to 0.953, `siege:easy` 0.990 to 0.964). No clear gain, so the lever
  and its test are reverted.
- **More refineries only with the money** (hard). dev-2's losses to `siege:normal` come on
  country 0. A watch of one (map 0, seed 8) shows both houses even to tick 4 500, hard with 11
  army units and normal none. At 5 250, with about 800 credits in hand, hard orders its second
  refinery (more refineries). It owes more than the money in hand, so the brain holds both army
  lines to pay for it (its rule for a refinery by a new field that waits for money). Hard's army
  stays at 11 units until 6 750 while normal's grows to 12 units worth nearly twice as much and
  strikes. Hard's army is gone by 8 250 and its base by 9 955, with 4 748 credits unspent. A
  person builds the next refinery when he can pay for it and keeps training meanwhile. So the
  more refinery is ordered only while the money in hand pays its price.
  Development round 4, stopped after two pairs: `siege:normal` level at 0.771, `default:hard`
  0.943 to 0.839 (map 2 14-3-15). A drop past 0.06 on a pair is no clear gain, so the lever and
  its test are reverted: the refineries ordered broke pay back on the long games.
- **Broke refineries only with an army** (hard, `Tuning::floor`). The narrower form of the last
  lever: the more refinery may still be ordered with less than its price in hand, but only
  while the house's army is worth the level's floor for a strike (5 000 at hard). The army lines
  the brain then holds stand on an army that can guard the base. In the watch above hard's army
  was worth 2 800 when it ordered the refinery. Development round 5, stopped after two pairs:
  `default:hard` 0.911 (dev-2 0.943), `siege:normal` level at 0.771 with country 0's losses
  unchanged (14). Even a perfect last three pairs would not make it a clear gain, so the lever
  and its test are reverted. That made three rounds in a row with no clear gain, so AS5 stopped
  on the code of round 2, which cert-2 certified.

### Hard, script AS6

Script AS6 took the siege's hard level further toward 2000 Elo, with the same rules of fair play.
Sim batch S51 changed depot arrivals after AS5 measured, so AS6 took its own baseline on its base:
cert-0 put `siege:hard` at 1914.9 (1894.8 order-free), 211-8-21, and dev-0 (AS5's: the five other
entrants, maps 0, 2 and 8, seeds 7 to 14, 480 games) scored 0.919 (418-16-46, 293 victories), both
exactly as AS5's last round: S51 did not move them. All 15 of dev-0's losses to `siege:normal` came
with country 0; dev-0's 46 draws are mostly stalemates at the cap on map 2.

- **The army trains while a new refinery waits for money** (`Tuning::train_while_paying`, hard
  only). A watch of a dev-0 loss to `siege:normal` (map 0, seed 12, country 0) shows AS5's watch 4
  again: hard stands with 11 army units from tick 3 750 to 6 750 and both army lines held, its
  credits at 22 to 792, while its second refinery is paid. The brain pays for a refinery by a new
  field first (`field_refinery_short`: such a refinery owes more than the money in hand and the
  house is safe) and holds every other item for it. Normal's army grows to 13 units worth 5 200,
  strikes, and hard's base falls at 10 592 with 6 220 credits unspent. A person who builds a
  refinery on credit keeps training soldiers beside it, and lets the refinery wait. So at hard,
  while the only reason the brain pays for its economy first is a refinery by a new field (the
  house owns a refinery, has lost none and keeps its economy target), the siege drops the brain's
  hold orders for army items and resumes the army items it held (S-0404 claims 3 and 5); the
  refinery is paid beside them. The rule is the siege's own; the brain's, which the default script
  plays, is unchanged.
  Development round 1, stopped after two pairs: `siege:normal` 0.771 to 0.750 (country 0 still
  lost 14), `default:hard` 0.943 to 0.885. Down 7.6 points and 13 victories on dev-0, more than
  the other three pairs have room to make up, so the lever and its test are reverted: the
  refinery that waits for the army comes late, and its income with it.
- **The recall judged on the whole army** (`Tuning::whole`, hard only). Most of dev-0's draws
  are stalemates at the cap. A watch of one (map 2, seed 10, country 1, against
  `default:normal`, a draw at 45 500 against 34 600) shows hard spending 60 200 on its army
  against 33 900. It made five strikes, and each was called home once it was worth less than half
  the enemy army it remembered (four aircraft among it), while more of its units stood at home.
  The enemy's harvesters fell to none or one, but its 26 buildings stood, and hard's 30 units
  waited at home from tick 22 500 to 25 500. A person whose army at home is on its way does not
  call the strike back. So at hard a strike is called home only while the strike and the free
  army units at home together are worth less than the recall share of the remembered enemy
  army; the units at home join it as before (`Tuning::join`).
  Development round 2, stopped after two pairs: `siege:normal` 0.745 (dev-0 0.771),
  `default:hard` 0.943 to 0.880 (map 2 from 26-0-6 to 16-0-16), past the limit of 0.06. The
  lever and its test are reverted: a strike kept out loses the army whose worth wins at the cap.
- **No more economy while outmatched** (`Tuning::wary`, hard only). All seven of dev-0's losses
  to `siege:normal` on map 8 come late (ticks 21 210 to 26 731, the enemy worth 39 400 to
  51 900). A watch of one (seed 7, country 0) shows both houses taking in about 70 000 by tick
  21 000. Hard put 36 000 of it into its army, 10 000 into factories, 9 800 into harvesters and
  8 000 into refineries; normal put 50 100 into its army, on two refineries and four harvesters.
  So hard's more refineries bought no more income there. From tick 13 500, when it remembered an
  enemy army worth 10 500 against its own 4 700, it still added two factories, and at 18 000 a
  refinery. A person who sees a larger army than his own puts the money into his army first. So
  at hard, while the enemy army the house remembers is worth at least the level's floor and more
  than its own army units, the siege's more refineries and more factories wait. The brain's own
  plan is unchanged.
  Development round 3: 0.922 (421-16-43, 292 victories) against dev-0's 0.919 (293). `siege:normal`
  rose to 0.781 and `default:normal` to 0.917, and `default:hard` played as dev-0. That is no
  clear gain, so the lever and its test are reverted.

Three development rounds in a row brought no clear gain, so AS6 stopped with the hard level as AS5
left it: no code of AS6's is kept, and cert-0 (1914.9) stands. What the rounds showed:

- `default:hard` on map 2 is where a change shows first. Both levers that sent more of the army
  into fights (training through the refinery's wait, keeping a strike out) turned worth wins at
  the cap there into draws (26-0-6 to 19-2-11 and 16-0-16). Hard wins that pair by keeping its
  army alive to the cap.
- The losses to `siege:normal` with country 0 did not move under any lever (14 to 16 of them in
  every round). On map 8 they come late, against an enemy army that outgrows hard's on the same
  income; on map 0 they come early, while the base is paid for.

### Hard, script AS7

Script AS7 took the siege's hard level further toward 2000 Elo, with the same rules of fair play
and AS6's measurement. On its base (b289f95) cert-0 put `siege:hard` at 1914.9 (1894.8
order-free), 211-8-21, and dev-0 scored 0.919 (418-16-46, 293 victories), both exactly as AS6's.
AS6's levers had changed how hard spends; AS7 looks at what hard knows when it decides to strike.

- **A look at the enemy base** (`Tuning::scout`, hard only). Two watches of dev-0 losses to
  `siege:normal` with country 0 show hard striking blind. On map 0 (seed 7) hard stands with 11
  soldiers worth 2 800 from tick 3 750, while normal's base holds 0 to 3 army units and no
  defence until 5 250. Hard remembers no enemy army, so the floor (5 000) keeps the soldiers home,
  and they die there at 8 250 when normal's 12 units arrive. On map 8 (seed 7) hard remembers an
  enemy army worth 1 000 at tick 9 000 when normal has 20 units worth 8 000 at home. Its one
  strike, 24 units worth 9 200, goes into them and is lost. A person sends a fast unit to look at
  the enemy base while his army builds up. So at hard, while no strike is out and the house has
  seen no enemy building for the level's `scout` ticks, the fastest idle army unit at home beyond
  the guard goes to look (a plain move to the strike's goal), and is called home once the house
  sees an enemy building, or when it stands idle. The enemy army it sees is remembered as any
  other. While the house's last look at an enemy base, within the memory's time, showed no armed
  enemy building, the floor does not hold a new strike: the edge alone decides, against the enemy
  army the house remembers. The brain's own scout, which explores the house's side of the map, is
  unchanged.
  Development round 1, stopped after two pairs: `siege:normal` 0.771 as dev-0 (map 0 worse, 8 to 13
  losses, maps 2 and 8 better), `default:hard` 0.943 to 0.870 (draws 11 to 21 on maps 2 and 8),
  past the limit of 0.06. The lever and its test are reverted: strikes of cheap soldiers on a
  base seen open lose the army that wins `default:hard` at the cap.
- **The recalled come home** (`Army::homing`, hard only). A watch of a dev-0 draw against
  `default:hard` (map 2, seed 10, country 0, 52 200 against 39 200 at the cap) shows hard with 32 to
  45 army units (10 800 to 16 200) from tick 21 000 to the cap against an enemy army remembered at
  800 to 7 000, and no strike after its second recall. A local count of the army's think (not
  kept in the code) shows why: of those units only 5 to 18 wait at home; 20 to 40 stand more than
  30 cells from home, most of them fighting or moving, and the brain gives them no order. The
  recall sends the strike home with plain moves and drops it from the strike; units fired on on
  the way stop to fight, and from then on neither the strike nor the gathering at home counts
  them. A person who calls his army home clicks the stragglers home again. So at hard the units a
  recall calls home are kept as homing; at each think, one that is neither idle nor within the
  recall's reach of home (the brain's 30 cells) and not under a move home is ordered home again
  (unless the brain ordered it this think). It leaves the homing ones once idle or near home, and
  then waits for the next strike as any other.
  Development round 2, stopped after four pairs: `siege:normal` 0.771 to 0.797, `default:normal`
  0.906 to 0.932, `default:easy` 0.984 as before, but `default:hard` 0.943 to 0.917 (10 of its 16
  draws on map 2). Combined +0.005 and 7 more victories; with `siege:easy` holding only 2
  non-wins, 12 more could no longer be reached. The lever and its test are reverted.
- **The strike is fed, not called home** (`Tuning::feed`, hard only). A watch of a dev-2 draw
  against `default:hard` that dev-0 won (map 2, seed 8, hard as house 0: 46 600 against 37 600 at
  the cap) shows hard's third strike, 21 to 24 units sent at tick 18 000, killing the enemy's
  harvesters (7 to 0) and taking its total from 44 800 to 35 200. At 24 000 it is called home
  with 15 left, below half of the enemy army it remembers, and hard does not strike again until
  27 000. The recall is judged before the units waiting at home are sent to join the strike, so
  a strike that would be fed is called home instead. A person whose winning attack thins sends
  the new units after it. So at hard, while a strike is out, the units waiting at home beyond the
  guard, once there are as many as the level's join, count with it against the recall: while
  together they are worth the recall's share of the remembered enemy army, the strike is not
  called home, and they go out to join it as before.
  Development round 3, stopped after three pairs: `siege:normal` 0.771 to 0.776 (40 victories
  against 38), `default:hard` 0.943 and `default:normal` 0.906 as dev-0, one victory fewer and as
  many. One more victory in all; with 5 non-wins left in the two easy pairs, a clear gain could no
  longer be reached. The lever and its test are reverted.

Three development rounds in a row brought no clear gain, so AS7 stopped with the hard level as AS5
left it: no code of AS7's is kept, and cert-0 (1914.9) stands. What the rounds showed:

- `default:hard` on map 2 still guards the score. The look at the enemy base sent cheap strikes
  out and turned its wins into draws (0.870); calling the stragglers home cost it 0.026; feeding
  an outmatched strike left it as it was.
- The losses to `siege:normal` with country 0 (15 to 18 in every round) are not about what hard
  knows of the enemy or about stragglers: hard's army trails normal's while its economy leads,
  and every lever that changed how the army fights left them in place.
- What is left between the strike's rules and 2000 is how much army hard's lead in income buys:
  from tick 15 000 in the watches its money goes into buildings while its army stays level with
  the enemy's.

### Hard, script AS8

Script AS8 took the siege's hard level further toward 2000 Elo, with the same rules of fair play
and AS7's measurement. On its base (dceb638, with sim batch S52 merged) cert-0 put `siege:hard` at
1914.9 (1894.8 order-free), 211-8-21, and dev-0 scored 0.919 (418-16-46, 293 victories), both
exactly as AS7's. Of dev-0's 16 losses 15 are to `siege:normal` with hard on country 0, and that
pair goes 22-15-11 on country 0 against 45-0-3 on country 1: the same script and the same money,
so what differs is the army the two countries' types make.

- **The army matched to the enemy's** (`Tuning::matchup`, hard only). A watch of a dev-0 loss
  (map 8, seed 7, country 0, against `siege:normal`) shows hard's army mostly cheap infantry
  (dogs, rifles, guardians and engineers, with a few tanks) against an enemy army whose worth is
  mostly vehicles. Its strike of 24 units worth 9 200 is lost to 19 worth 7 900. The brain's
  army round picks, on each free line, the type the house has fewest of on the map, whatever it
  faces. A person builds what beats the army he has seen. So at hard, once the house remembers an
  enemy army, the type the brain's army round ordered on each infantry and vehicle line is
  replaced by the type of that line with the best value against the remembered enemy army. The
  value is our own estimate, after the square law of attrition: a force's strength grows with its
  units' damage, their strength and the square of their number, so per credit it is
  - the damage per tick the type deals the remembered units (for each, the better of its first
    two weapon slots: damage times burst times the versus value of the unit's armour, over the
    rate of fire), each unit's share taken per point of its strength and weighted by its cost,
  - times the type's strength over the damage per tick the remembered units deal its armour,
  - over the type's cost squared.
  Unarmed types (engineers) are worth nothing by it and are not picked. Types the house sends
  alone and loses on use, lifters and naval types keep the brain's own rules. The brain's pick
  stays while no enemy army is remembered, or when no type of the line is worth anything against
  it. The brain's own army round, which the default script plays, is unchanged.
  Development round 1, stopped after two pairs: `default:hard` 0.943 to 0.901 (country 0 better,
  42-0-6 to 45-1-2; country 1 worse, 43-0-5 to 36-3-9), `siege:normal` 0.771 to 0.708 (country 0
  0.573 to 0.615; country 1 45-0-3 to 35-6-7), past the limit of 0.06. A watch of a new country-1
  loss (map 2, seed 8) showed a fault: both army lines stood on a player hold from tick 7 500 to
  the end. The brain resumes a held item by ordering its type again (S-0404 claims 3 and 5), and
  the swap changed that order to another type, so the held item waited for ever and the house
  built no army while its credits rose to 23 000. The swap now takes only an order on a free
  line, which is what the army round orders. With that fixed the same game is a draw (40 700
  against 32 700) where dev-0 won at the cap (46 500 against 15 400): the infantry line builds
  conscripts only, over 40 of them, the cheapest type. The square law's cost squared favours the
  cheapest type beyond what it is worth in a fight of crowded units.
- **The matchup by what a credit deals before it dies** (`matchup_value`, hard only). The value
  of the matchup above with the cost taken once, not squared: the damage per tick a credit spent
  on the type deals the remembered army (as weighted above), times how long the type lasts
  against it (its strength over the damage per tick the remembered units deal its armour). A type
  twice as strong and twice as dear is worth as much; under the square law it was worth half.
  On the watched game it picks as before (the same conscripts and flak tracks, the same draw).
  Development round 2, stopped after four pairs: `default:hard` 0.943 to 0.917 (map 2 26-0-6 to
  21-1-10), `siege:normal` 0.771 to 0.792 (country 0 0.573 to 0.667), `default:normal` 0.906 to
  0.964 (10 more victories), `default:easy` 0.984 to 0.974. Up 4 points and 8 victories; with
  `siege:easy` holding 2 non-wins, 12 more could no longer be reached. The matchup, its fix and
  its value are reverted. A watch of one of the new draws against `default:hard` (map 2, seed 8,
  country 1, 34 700 against 31 800) shows the infantry line all riflemen (20 to 33 of them) beside
  2 to 7 flak tracks, against a small remembered army that is partly aircraft.
- **The matchup on the vehicle line only** (hard only). The matchup above with the infantry line
  left to the brain's round robin: its gains came where the enemy fields tanks, its stalls where
  the cheapest soldier replaced the barracks' mix. Development round 3, all five pairs:
  `default:hard` 0.932 (map 2 22-0-10), `siege:normal` 0.792 (country 0 26-12-10),
  `default:normal` 0.943, `default:easy` 0.990, `siege:easy` 0.990. In all 428-16-36, 0.929
  against dev-0's 0.919, with 303 victories against 293: up 0.010 and 10 victories, no pair down
  more than 0.011, short of a clear gain. The lever and its test are reverted.

Three development rounds in a row brought no clear gain, so AS8 stopped with the hard level as AS5
left it: no code of AS8's is kept, and cert-0 (1914.9) stands. What the rounds showed:

- Picking the army by the enemy's types moves the pairs the earlier levers did not:
  `default:normal` gained 9 to 10 victories in rounds 2 and 3 and `siege:normal` with country 0
  rose from 0.573 to 0.646 to 0.667. But each form cost `default:hard` on map 2 (0.011 to 0.042),
  and the gains stayed under 12 victories or 0.02.
- A rule that swaps the brain's orders has to leave a resume order alone: the brain resumes a held
  item by ordering its type again (S-0404 claims 3 and 5).
- The early country-0 losses to `siege:normal` on map 0 are still AS5's: the army lines held from
  tick 5 250 to 6 750 while the second refinery is paid (a watch of map 0, seed 8), against an
  enemy that strikes at 7 000 with tanks.

### Hard, script AS9

Script AS9 took the siege's hard level further toward 2000 Elo, with the same rules of fair play
and AS8's measurement. On its base (ed0dfd3) cert-0 put `siege:hard` at 1914.9 (1894.8
order-free), 211-8-21, and dev-0 scored 0.919 (418-16-46, 293 victories), both exactly as AS8's.

- **The vehicle matchup once the enemy army is known** (`Tuning::matchup`, hard only). AS8's
  matchup on the vehicle line alone (its round 3) came nearest a clear gain of any lever since
  AS5: +0.010 and 10 more victories, `siege:normal` with hard on country 0 from 0.573 to 0.667,
  `default:normal` 9 more victories. What it cost was `default:hard` on map 2 (26-0-6 to
  22-0-10), and AS8's watch of a new draw there showed the swap answering a small remembered
  army, partly aircraft. A person picks his tanks by the enemy army once he has seen enough of
  it to judge; a glimpse of a few units says little. So AS8's vehicle-line matchup comes back
  (the type on a free vehicle line of best value against the remembered enemy army, the cost
  taken once, a held item's resume order left alone), and it swaps only while the ground army
  the house remembers is worth at least the level's floor for a strike (`Tuning::floor`, 5 000
  at hard). Below that the brain's round robin picks, as on the base.
  Development round 1, stopped after four pairs: `default:hard` 0.943 as dev-0 (map 2 26-0-6 again),
  `default:normal` 0.906 to 0.932, `default:easy` 0.984 as before, but `siege:normal` 0.771 to 0.740
  (country 0 22-15-11 to 22-17-9). Four pairs 0.900 against 0.901 and 4 more victories; with
  `siege:easy` holding 2 non-wins, neither bar could be reached. The gate kept `default:hard`, but
  took away the matchup's gain against `siege:normal` with country 0 (AS8's 0.667 to 0.552): there
  the matchup pays early, before much of the enemy army is known. The gate and its test are
  reverted.
- **The matchup and no economy while outmatched** (`Tuning::matchup` and `Tuning::wary`, hard
  only). A watch of a dev-0 loss (map 8, seed 7, country 0, against `siege:normal`; the game
  AS6 to AS8 watched) shows it again. By tick 15 000 hard has put 26 500 into its army against
  normal's 28 600, but 10 000 into factories and 8 000 into refineries against 5 500 and 4 000.
  From 12 750, remembering an enemy army worth 9 000 against its own 3 600, it adds two more
  factories (7 500 to 10 000 by 14 250), and its army is mostly dogs, riflemen and guardians
  against normal's tanks. The two nearest gains of the last batches answer those two things in
  different pairs: AS8's vehicle-line matchup (its round 3, +0.010 and 10 victories, mostly
  `default:normal` and `siege:normal` with country 0) and AS6's no more economy while outmatched
  (its round 3, +0.003, `siege:normal` on map 8 and `default:normal`). So both come back
  together: AS8's vehicle-line matchup without the gate above, and AS6's rule that, while the
  enemy army the house remembers is worth at least the level's floor and more than its own army
  units, the siege's more refineries and more factories wait. The brain's own plan is unchanged.

## The tournament

`glasnost-headless ai-tournament [key=value …]` (`glasnost_tools::ai_tournament`):

- **Entrants:** `entrants=key:level,…` (default the default script at easy, normal and hard).
- **Games:** every pair of entrants on every ground (`maps=`: fixture grounds 0 and 1, or the
  install's map indices with `game=install`), seed (`seeds=1-4` or a list), start assignment and
  side assignment: four games per pair, ground and seed, in that fixed order. On the install the
  two sides are the first open country of each of `sides=a,b` (default the first two sides).
- **The cap:** `ticks=` (default 27 000). A game the victory rules end (S-0414) is won by its
  winner. At the cap, each house's worth is the cost field (S-0403 claim 2) summed over its live
  buildings and units on the map; a house worth at least 3/2 of the other wins, any other game is
  a draw.
- **Elo:** every entrant starts at 1500; after all games are played, the games are taken in their
  fixed order and both entrants move by 32 · (score − expected), score 1, ½ or 0, expected
  1 / (1 + 10^((R_other − R_own) / 400)).
- **Order-free Elo** (script AS2): the Elo above depends on the order of the games (an entrant's
  last games weigh most), so the report adds a line with each entrant's mean final Elo over 100
  seeded shuffles of the games (`ai_tournament::order_free_ratings`). The games are first put in
  the order of their job (ground, seed, pair, start, side) and result, so the line does not
  depend on the order they came in; shuffle *n* is a Fisher-Yates shuffle drawn from a small
  generator of our own (splitmix64) seeded with *n*. The line is a second view only: the goal
  measure stays the Elo in the table.
- **Output:** `report.txt` (no timing lines, so the same options give the same text whatever
  `threads=` is; `tests/ai_tournament.rs` checks it on the fixture) and one replay per game,
  `replays/game-NNN.replay`, to `out=` (default `local-lane/ai-issues/b9/tournament/`).
  The worker threads are in the tool only; each game is its own world.
