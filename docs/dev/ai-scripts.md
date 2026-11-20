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

| level | harvesters (plan, at most) | attack wave | acts on | base defence | superweapon and special buildings | what a player sees |
|---|---|---|---|---|---|---|
| easy | 1, at most 2 | 3 units | one think tick in three (`EASY_EVERY`) | on its acting think ticks only | none | one or two harvesters, small attacks, a base that grows and answers a raid slowly, no superweapon strikes, no sensor, grinder, gattling defence, bunker, lifter or drainer |
| normal | 2, at most 4 | 6 units | every think tick | every tick | built | today's AI |
| hard | 3, at most 6 | 6 units | every think tick | every tick | built | a third harvester (up to six on a large field), so more money and a larger army sooner |

The plan's count is the harvesters kept; on a large explored field the house buys one more for
every 20 resource cells within reach, up to twice the plan's count (`Script::harvesters_wanted`;
a Yuri house's slave miners are not capped), so the plan counts 1, 2 and 3 are on stock maps
often 2, 4 and 6 harvesters. `brain::plan_for` gives the plan of each level; the think rate, the
defence gate and the skipped building parts are in `Brain::think_from`, read from the tick and the
level alone (no state is added, so normal hashes and plays as before).

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
