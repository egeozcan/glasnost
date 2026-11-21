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
- **Output:** `report.txt` (no timing lines, so the same options give the same text whatever
  `threads=` is; `tests/ai_tournament.rs` checks it on the fixture) and one replay per game,
  `replays/game-NNN.replay`, to `out=` (default `local-lane/ai-issues/b9/tournament/`).
  The worker threads are in the tool only; each game is its own world.
