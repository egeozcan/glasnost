---
id: Q-1703
title: Measurement round for the arena-reached and frequent claims, in ranked order
status: answered
related: [S-0202, S-0301, S-0302, S-0306, S-0309, S-0311, S-0207, S-0206, S-0307]
answered-by: [M-0008, M-0004]
---

## Need

M8 track A turns `measurable` claims into measured ones. Our unit arena fails 55 of 155 pairings
against M-0001 to M-0003, mostly because our games vary much less from seed to seed than the stock
games. The claims below are the ones whose measurement would most change play: those the arena
reaches (draws, delays and spreads first), then the frequent rules of every game. Each row names the
spec's own measurement scenario; the scenario text and setup are in that spec.

## Questions

1. For each row below, in rank order: run the named spec scenario and record the result as an
   M-record, stating relations and tolerances (no stock rules values in committed files; use
   `value-visibility: local` where a value is rules data).
2. Where a record contradicts a claim, revise the spec (the measurement wins), and say so in the
   record's verdict.
3. Rows may be delivered one or a few at a time; each record names its row's spec and claims.

## Ranked rows

Claim numbers are those of the named spec; scenario letters are the spec's own. Where two rows touch
one claim, the cheaper scenario first. Cost is our rough guess from the scenario text.

| Rank | Spec and scenario | Claims it settles | Why it ranks here | Cost |
|---|---|---|---|---|
| 1 | S-0301 F1, F3, F14 (with the random-state recorder) | 1, 2, 4, 18: rearm delay with its draw, in-burst delay draw, the draws one shot takes and their order | Sets the spread of every shot interval in every fight; its draws are the arena's seed-to-seed variation | medium |
| 2 | S-0202 R2, R5 | 1, 2, 3, 4, 6, 8, 10: seeding function, stream recurrence, range rule, the creation draw | Decides how a seed becomes a game; wrong here and every game varies too little or too alike | medium |
| 3 | S-0309 A1 (incl. part f) | 1, 2, 3, 8: search radius, when searches run, targeting timer draw, conditional draw ordering | The target-search timer draw staggers every unit's first shot; a main source of spread | medium |
| 4 | S-0309 A6 | 4, 19, 21, 23, 24: keeping and replacing targets | Decides when a unit switches targets mid-fight; moves time to kill | medium |
| 5 | S-0306 P3 | 4, 9, 21, 24: scatter of shots | A random spread of where shots land; hit rate depends on it | medium |
| 6 | S-0306 P9 | 5, 6, 12 to 15, 19: steering projectiles | Backs the open arena discrepancy (anti-air missiles detonating at the launcher); confidence below high | medium |
| 7 | S-0309 A11 | 1, 7, 11, 17, 18, 22: map-wide search | The hunting behaviour the arena uses; backs the open hunter-versus-hoverer discrepancy | medium |
| 8 | S-0302 D1, D2 | 2, 3, 4, 5, 6, 11: versus, truncation, firepower, house armour, minimum damage | Every damage number; cheap and settles several claims at once | cheap |
| 9 | S-0311 X1, X3, X7 | 1, 7, 8, 9, 10, 11, 14: kill gain, threshold, cap, recipients, starting experience | The veterancy path of the arena; threshold and cap change who wins long fights | medium |
| 10 | S-0301 F2 | 2, 3, 6: house and veteran rate-of-fire factors | Cheap; scales every delay once ranks are reached | cheap |
| 11 | S-0207 D1, D3, D8 | 1, 2, 3, 4, 9, 10, 12: steady progress, acceleration, positions between points, leftover budget | Speed of every ground vehicle; decides who closes first | cheap |
| 12 | S-0207 D7, D9, D17 | 3, 5 to 8, 10, 15, 16: curve against turn in place, no chaining, a refused step | Movement feel and blocking in the arena; several claims are low confidence | medium |
| 13 | S-0206 R2, R7, R4 | 1, 3, 4 to 10, 12: turn time, a new target during a turn, turret aiming | Turn time before a first shot; every fight starts with a turn | cheap |
| 14 | S-0307 RG1, RG2 | 1 to 5, 8, 9: range field, comparisons, bonuses, airborne firer | Who gets the first shot at the edge of range | cheap |
| 15 | S-0303 A1, A11, A3 | 2, 5, 7, 8, 9, 11, 12: falloff, order of hits, one hit per foundation cell | Splash on groups and buildings; arena groups are clumped | medium |
| 16 | S-0312 D6, D13, D10, D9, D1 | 1, 3, 6 to 8, 12 to 14, 18, 25: death weapon, survivors, crew, passengers, order inside one death | Death effects feed back into fights and the post-fight state the arena reads | medium |
| 17 | S-0310 T1, T2, T3 | 1, 2, 3, 9: facing for fire, tolerances, still turning | Delays the first shot of every turreted or turning firer | cheap |
| 18 | S-0309 A7, A8, A4, A9 | 8, 9, 10, 13 to 17, 20: threat preference, weak-target preference, ring order, enemy house bonus | Chooses whom a group shoots; changes losses in mixed groups | medium |
| 19 | S-0306 P1, P4, P22 | 1, 2, 5 to 7, 9, 10, 19, 21, 27: shot speed, gravity, direction, arm time | Time of flight and so time to kill; the straight-shot path | medium |
| 20 | S-0208 W3, W9, W7 | 3, 11 to 13, 15: infantry position-order draw, draw counts and order, scatter acceptance | Draws that stagger infantry; arena infantry pairings fail often | medium |
| 21 | S-0208 W4, W5, W10 | 6 to 8, 10, 16 to 18: walking step and speed, crawling, panic, end of a walk | Infantry speed, as S-0207 D1 is for vehicles | cheap |
| 22 | S-0215 B3, B6, B7, B12 | 3, 7 to 9, 11, 12, 14, 15, 17 to 19, 22, 25, 28 to 31: guard, scatter, sticky, return fire | Behaviours of idle and attacked units; scatter off a building is a draw | medium |
| 23 | S-0209 C1 to C5 | 1 to 11: cell capacity, vehicles and infantry, enemy occupants, blockage delay | Crowding feel (plan step A4); cheap and several claims per run | cheap |
| 24 | S-0212 P1, P7, P4 | 1, 3, 4, 6 to 8, 10, 12, 13, 15, 17, 18, 21, 23 to 25, 27: path shape, destination substitution, blocked routes | Every move order in every game; many claims, middling confidence | expensive |
| 25 | S-0402 H1, H4, H2, H3 | 1 to 11, 16: a full harvest cycle, refinery choice, field choice, partial load | The economy loop of every game; one cycle settles seven claims | cheap |
| 26 | S-0401 E5, E4, E6, E10 | 5 to 13, 17, 18: spread, growth period and count, slopes, no regrowth | Ore regrowth feeds the economy of long games; draws are involved | medium |
| 27 | S-0404 P1 to P4, P12 | 1 to 4, 10, 13, 14: parallel lines, price, payment while waiting, queue limit, release | Production pacing for every game; cheap | cheap |
| 28 | S-0403 B1 to B5 | 1 to 6: step count and period, base time, country and type factors, low power, several factories | Build times shape every build order | cheap |
| 29 | S-0405 W1, W4, W7 | 1, 2, 5 to 8, 10, 18: output and drain, blackouts, switching power | Frequent rule; low power slows production and defences | cheap |
| 30 | S-0308 W1, W4, W8, W7 | 1 to 5, 8 to 10, 14: slots and elite weapons, switching, deployed weapons, passengers | Which weapon a unit fires at which target; arena types use several | medium |
| 31 | S-0407 T1, T2, T8, T10, T14 | 1, 2, 8 to 15: offer timing, prerequisites, build limits, rechecks | Decides what a player can build and when; frequent in every game | medium |
| 32 | S-0314 H3, H13, H14, H18 | 3 to 5, 8, 12 to 15, 20 to 24: repair step and cost, automatic repair and sale draw, depot contact | Repair and the automatic-sale draw alter late-game fights | medium |
| 33 | S-0408 S1, S3, S5 | 1, 3, 4, 6, 9, 15 to 17: refund, sellable test, course of a sale | Selling is frequent and moves money; cheap | cheap |
| 34 | S-0406 PL1, PL8, PL11 | 1, 7 to 11, 14, 15, 17, 18: place order outcomes, walls and gates, construction cadence | Base building in every game | medium |
| 35 | S-0315 P1 to P5 | 1 to 7: fear and panic, fraidycat, prone | Infantry in the arena; several claims at low-to-medium confidence | cheap |
| 36 | S-0313 P1, P2, P3 | 1 to 4: vehicle fire point, burst alternation, building turret displacement | Shot origin moves hit timing and line of fire | cheap |
| 37 | S-0210 K1, K2, K5 | 1, 2, 4, 9: who is crushed, the moment and radius, the crush slowdown | Vehicles versus infantry in the arena | medium |
| 38 | S-0304 V1, V2 | 1 to 6: the bands of special versus values, weapon choice | Few claims, cheap, and it sets how special values enter every damage number | cheap |
| 39 | S-0502 J2, J1, J3, J5 | 1 to 5, 7, 8, 9, 22, 23: jumpjet take-off, field copy, occupation, ground hold | Jumpjets are in the arena and back the hovering discrepancy | medium |
| 40 | S-0213 O1, O2, O5 | 1, 2, 8, 9, 14, 18: tick boundary of orders, pending list limit, stop and scatter | Every human order passes through these rules | cheap |

## Possible observation

The scenarios named in each row, as their specs give them.

## Current workaround

The engine follows the specs' claims as written; the arena's spread gap stays open (M8 step A1 audits
it from our side).
