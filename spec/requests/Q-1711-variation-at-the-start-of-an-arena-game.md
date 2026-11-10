---
id: Q-1711
title: What varies from seed to seed before two placed groups first meet
status: answered
related: [S-0202, S-0009, S-0215, S-0301, S-0208, S-0207]
answered-by: [S-0216, M-0006]
---

## Need
Our oracle-arena games vary much less from seed to seed than the stock games of M-0001 to M-0003:
of the 310 sides (155 pairings, two sides each), the stock outcome (winner, end frame and both
shares) is the same for all six seeds on 135 sides, ours on 178. The audit of M8 step A1
(`docs/m8-claims/a1-arena-spread.md`) found that the seed reaches our games: the scenario stream
is seeded with the game's seed, and the number of raw values taken differs from seed to seed.
But in our vehicle pairings the game state is the same for every seed until the groups first
fire (about tick 30 to 90): nothing drawn before then changes what a vehicle does, so the groups
meet in the same way in every game. Walking infantry differ from tick 2 (the standing-position
draw of S-0208 claim 3), and in our infantry pairings the shares left vary with the seed but the winner seldom does.

The records' Setup says where each unit stands, which way it faces and that it starts in the
hunt behaviour. It does not say whether anything about a unit varies before the first contact.
Our setup creates each unit with its creation draw (S-0202 claim 10), places it on its stated cell
with its stated facing, and starts its hunt behaviour with the behaviour timer expired
(S-0215 claim 3), the same for every seed. Infantry stand at the centre standing point of their
cell.

Code: `crates/tools/src/oracle.rs` (`setup`), `crates/deterministic/sim/src/load.rs`.

## Questions
1. In a game where two groups are placed hunting and given no orders, does every unit start
   moving, search for a target and fire its first shot at the same tick in every game, or do these
   moments vary from seed to seed? If they vary, by how much and from what?
2. Does a placed vehicle keep exactly the facing it was placed with until it first turns, or does
   its body or turret facing vary between games?
3. Where in its cell does a placed infantry unit stand: always at one fixed standing position, or
   at one that varies between games?
4. Do the units of a group take exactly the same path to the enemy in every game, or do the paths
   (or the order in which the units move off) vary?
5. For the records: were the arena's units created as map entries of the scenario (with whatever
   the map load does for such entries) or by another means after the game had started?

## Possible observation
In the arena map, one group of eight vehicles and one of eight infantry units per side, all hunting,
played with two different seeds. Record per unit, for the first 100 ticks, its position, body and
turret facing, behaviour and target, and the tick of its first shot. Compare the two runs.

## Current workaround
Each unit is placed as the records' Setup states, with nothing varied by the seed apart from the
draws the specs name (the creation draw and the draws of play). No variation is invented.
