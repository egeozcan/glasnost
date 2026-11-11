# Oracle wishlist: ranked measurement scenarios (M8 step A2)

This is the list of scenarios the project lead can run against the original game, in the order in which
a record would most change play in our engine. It feeds Track A of [../m8-plan.md](../m8-plan.md).
It is a reading aid, not a spec: every claim and scenario below is defined by the spec it names, and
the spec's *Measurement scenarios* section holds the setup. Keep it current as records land (a row
whose claims a record settles moves to the done list at the end).

**How it was built.** Every approved spec marks each claim `Backing: measured | measurable |
unmeasured`. All 109 specs together carry 2113 claims whose backing includes `measurable`, which is
nearly all of them, so "measurable" alone does not rank anything. Rows are therefore ranked by
scenario, by how much the result would constrain play, highest first:

1. Claims the unit arena reaches (`crates/tools/src/oracle.rs`, records M-0001 to M-0003): firing,
   targeting, projectiles, movement, veterancy, death. Anything that is a random draw, a delay or a
   spread ranks first, because the arena's main failure is that our games vary too little from seed to
   seed. The seeding itself (S-0202) is in this group.
2. Frequent rules of every game: economy, production, power, harvest, ground movement, crowding.
3. Claims whose spec confidence is below high, and claims behind an open arena discrepancy
   (steering anti-air projectiles, hunting units against hovering jumpjets).
4. Cheapness: a scenario that settles several claims at once beats one that settles a single claim.

Cost is the rough effort of one oracle round, my judgement from the scenario text: cheap (one small
test map, a few runs, a plain recorder), medium (several parts or a statistical run over many seeds),
expensive (many parts, special setups, or a recorder of random-stream state).

**Measured values are never committed.** A record states relations and tolerances (for example "the
interval is the rate plus a draw from a range of three, over N runs, within a tolerance"), the way
M-0001 to M-0003 do, and no stock rules value goes into a tracked file. The scenarios use invented
test files, as the specs require.

## Top 40 scenarios

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

Notes on reading the table. Claim numbers are those of the named spec. A scenario letter is as the
spec gives it, with the spec's own part letters inside it. Where two rows touch one claim (for example
S-0301 claim 2 under F1 and F2) run the cheaper scenario first.

## Added by the arena triage (M8 step A6b)

Scenarios behind arena rows that still fail after [a6b-arena-triage.md](a6b-arena-triage.md),
ranked among themselves by the rows they would decide. All four rank with group 3 above (claims
behind an open arena discrepancy); entry 1 decides the most rows and would sit beside rank 7.

| Rank | Spec and scenario | Claims it settles | Why it ranks here | Cost |
|---|---|---|---|---|
| A6b-1 | S-0215 B11 (b) with the building full, and S-0508 G1's refused arrival, for a computer-controlled occupier infantry unit in *hunt*: record its behaviour, target, destination and queued behaviour on every tick from the refusal on, and whether it takes the building as target from *area guard* before its first *hunt* processing (Q-2926). Since S-0508 r19 (intake 123) this is scenario **G21**, which allows M-0001's set-up, side B, as its map; **Q-2951** asks it for the rows on the right, which r19 does not reproduce | S-0215 claims 9, 27, 28 (hunt rule); S-0309 claim 25 (e); S-0508 claims 1 and 4 (r19) | Decides M-0001 TELE–E2 and M-0002 INIT–E1 and YURI–E1 (side B), and bears on TELE–E1, YTNK–E1 and YURI–E2 | cheap |
| A6b-2 | The re-read that S-0309's open question *Q-2112 question 1* names: one DISK–FV game of M-0001, every change of each hunting vehicle's behaviour, target and destination with its tick, each route failure and give-up, and each jumpjet's layer and height on those ticks | S-0309 claims 7, 22; S-0215 claim 18 | Decides DISK–FV's win test (M-0001), failing since A6 | cheap (an existing game) |
| A6b-3 | A6b-1's recorder on M-0001 CAOS–E1, side A, seeds 0 to 5: which E1 enter a building, when, and what the rest do | S-0309 claim 25 (e); S-0215 claim 27; S-0508 claims 1, 4 | Decides CAOS–E1 side A, systematic before and after A6b's fix | medium |
| A6b-4 | M-0015's hold recording for M-0001 TELE against V3, side A: each grip, set-down, restoration and new grip with its tick | S-0607 claims 8, 12, 16 to 18; S-0215 claim 37 | Decides TELE–V3 side A (our games end without a winner), open since intake 45 | medium |

## Measurable claims per spec

A claim counts when its backing line contains `measurable` (a claim whose backing is split between
`measurable` and `unmeasured` counts once). Counts come from a script over the claim lists and may be
off by a claim where a spec formats a claim unusually.

| Spec | Measurable claims | Claims in spec |
|---|---|---|
| S-0513 (superweapon-effects) | 50 | 52 |
| S-0009 (map-file-semantics) | 46 | 48 |
| S-0501 (aircraft) | 43 | 44 |
| S-0516 (cloak-stealth) | 43 | 43 |
| S-0105 (voxel-rendering) | 42 | 42 |
| S-0701 (sound-selection) | 40 | 40 |
| S-0215 (behaviours) | 38 | 38 |
| S-0011 (map-triggers-mp) | 35 | 36 |
| S-0102 (draw-order) | 33 | 34 |
| S-0306 (projectiles) | 33 | 33 |
| S-0007 (art-lookup) | 31 | 34 |
| S-0214 (order-targeting-rules) | 31 | 31 |
| S-0312 (death-and-destruction) | 31 | 32 |
| S-0616 (particle-systems) | 31 | 31 |
| S-0502 (jumpjet-movement) | 30 | 31 |
| S-0109 (animation-playback) | 28 | 29 |
| S-0402 (harvest-cycle) | 28 | 28 |
| S-0520 (crates) | 28 | 28 |
| S-0207 (ground-vehicle-movement) | 27 | 27 |
| S-0212 (pathfinding) | 26 | 27 |
| S-0303 (area-damage) | 26 | 26 |
| S-0506 (missiles) | 26 | 26 |
| S-0514 (chrono-weapons) | 26 | 27 |
| S-0213 (human-orders) | 25 | 25 |
| S-0309 (target-acquisition) | 25 | 25 |
| S-0314 (healing-repair) | 25 | 25 |
| S-0519 (terror-attack-animals) | 25 | 26 |
| S-0607 (magnetic-lift) | 25 | 26 |
| S-0108 (building-sprites) | 24 | 24 |
| S-0208 (infantry-movement) | 24 | 24 |
| S-0301 (firing-cycle) | 24 | 24 |
| S-0508 (garrisons) | 24 | 25 |
| S-0512 (superweapon-charge) | 24 | 24 |
| S-0614 (parasites) | 24 | 24 |
| S-0307 (range) | 23 | 26 |
| S-0414 (victory-and-defeat) | 23 | 23 |
| S-0518 (linked-weapons) | 23 | 23 |
| S-0107 (infantry-sprite-sequences) | 22 | 22 |
| S-0511 (bridges) | 22 | 22 |
| S-0003 (ini-value-forms) | 21 | 22 |
| S-0005 (data-file-precedence) | 21 | 22 |
| S-0302 (damage-resolution) | 20 | 20 |
| S-0401 (ore-and-gems) | 20 | 20 |
| S-0409 (shroud-sight) | 20 | 21 |
| S-0111 (in-game-overlays) | 19 | 19 |
| S-0406 (building-placement) | 19 | 20 |
| S-0608 (drain) | 19 | 19 |
| S-0405 (power) | 18 | 20 |
| S-0411 (sidebar-behaviour) | 18 | 18 |
| S-0503 (hover-amphibious) | 18 | 18 |
| S-0522 (paradrops) | 18 | 18 |
| S-0601 (mind-control) | 18 | 18 |
| S-0613 (gas-clouds) | 18 | 18 |
| S-0615 (special-hit-effects) | 18 | 18 |
| S-0104 (lighting) | 17 | 18 |
| S-0407 (tech-tree) | 17 | 17 |
| S-0605 (infantry-absorber) | 17 | 17 |
| S-0606 (vehicle-bunker) | 17 | 21 |
| S-0612 (psychic-dominator) | 17 | 18 |
| S-0308 (weapon-choice) | 16 | 16 |
| S-0408 (sell-repair) | 16 | 18 |
| S-0417 (factory-unloading) | 16 | 16 |
| S-0507 (transports) | 16 | 16 |
| S-0517 (deployable-units) | 16 | 16 |
| S-0525 (radiation-fields) | 16 | 16 |
| S-0004 (type-registration-order) | 15 | 16 |
| S-0101 (isometric-projection) | 15 | 17 |
| S-0103 (palettes-house-colour) | 15 | 15 |
| S-0206 (turn-rates) | 15 | 17 |
| S-0209 (occupancy-crowding) | 15 | 15 |
| S-0509 (engineers-capture) | 15 | 17 |
| S-0524 (mp-trigger-completion) | 15 | 15 |
| S-0610 (psychic-weapons) | 15 | 16 |
| S-0001 (ini-lexical-rules) | 14 | 14 |
| S-0202 (random-streams) | 14 | 15 |
| S-0210 (crushing) | 14 | 14 |
| S-0404 (production-queue-payment) | 14 | 15 |
| S-0410 (radar-minimap) | 14 | 15 |
| S-0412 (skirmish-setup-rules) | 14 | 15 |
| S-0504 (naval-units) | 14 | 14 |
| S-0603 (grinder) | 14 | 15 |
| S-0609 (gattling-spin-up) | 14 | 14 |
| S-0611 (mutation) | 14 | 14 |
| S-0106 (voxel-tilt-slopes) | 13 | 13 |
| S-0311 (veterancy) | 13 | 15 |
| S-0112 (overlay-terrain-object-drawing) | 12 | 12 |
| S-0006 (rules-layering) | 11 | 14 |
| S-0201 (game-speed-tick-rate) | 11 | 14 |
| S-0203 (tick-order) | 11 | 12 |
| S-0310 (turrets-facing-for-fire) | 11 | 11 |
| S-0415 (mcv-deploy-undeploy) | 11 | 12 |
| S-0510 (spies-infiltration) | 11 | 11 |
| S-0002 (ini-duplicates) | 10 | 13 |
| S-0012 (map-block-encodings) | 10 | 10 |
| S-0313 (firing-positions) | 10 | 10 |
| S-0110 (shroud-rendering) | 9 | 10 |
| S-0211 (land-types-passability) | 9 | 15 |
| S-0403 (build-time) | 9 | 10 |
| S-0010 (theater-tileset-semantics) | 8 | 25 |
| S-0204 (coordinates-distance-facing) | 8 | 13 |
| S-0304 (special-versus-values) | 8 | 8 |
| S-0515 (tech-buildings) | 8 | 8 |
| S-0315 (suppression-panic) | 7 | 7 |
| S-0013 (voxel-lighting-table-layout) | 6 | 6 |
| S-0014 (game-font-layout) | 6 | 6 |
| S-0205 (number-domains) | 6 | 9 |
| S-0305 (armour-classes) | 5 | 6 |
| S-0413 (country-bonuses) | 5 | 8 |
| S-0416 (credit-display) | 5 | 5 |

Total: 2113 measurable claims over 109 specs. The specs with the largest share not marked measurable are S-0010 (8 of 25), S-0211 (9 of 15) and S-0204 (8 of 13).
