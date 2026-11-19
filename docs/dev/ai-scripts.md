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

| level | harvesters kept | attack wave | base defence | what a player sees |
|---|---|---|---|---|
| easy | 1 | 3 units | reacts on the house's think ticks only | one harvester on the field, small attacks, a slower answer to a raid on its base |
| normal | 2 | 6 units | every tick | today's AI |
| hard | 3 | 6 units | every tick | a third harvester on the field, so more money and a larger army sooner |

`brain::plan_for` gives the plan of each level; the defence gate is in `Brain::think_from`.

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
