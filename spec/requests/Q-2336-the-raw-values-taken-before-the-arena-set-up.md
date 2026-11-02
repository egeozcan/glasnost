---
id: Q-2336
title: Which draws make up the records' game start before the arena set-up, given that ours takes 12 to 26 fewer raw values?
status: open
related: [S-0216, S-0202, S-0009, S-0412, M-0019, M-0001, M-0002]
answered-by: []
---

## Need

S-0216 revision 59 claim 8 (*Acceptance*) asks that the first object's creation draw in our
arena take the raw value at place *N*₀ of M-0019 for the seed: 706, 709, 711, 718, 706 and 701
for seeds 0 to 5. The draws before the set-up are to be our engine's map loading and game start,
as S-0202 claim 10 (its map-loading table) and claim 14 order them, with no padding.

Intake 77 built claim 8 as revision 59 gives it and added the check
(`arena::m0019_the_first_creation_draw_takes_the_raw_value_at_n0`, local lane). It fails. On the
records' map and setup, our engine has taken this many raw values when the first object's
creation draw is taken:

| Seed | 0 | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|---|
| M-0019 *N*₀ | 706 | 709 | 711 | 718 | 706 | 701 |
| Ours | 693 | 685 | 685 | 695 | 693 | 689 |
| Short by | 13 | 24 | 26 | 23 | 13 | 12 |

Ours depends on the seed only, the same in every pairing, as M-0019 finds for *N*₀. Every
infantry unit's standing position in our arena is therefore read from another raw value than in
the records' games, so claim 2's positions at tick 0 cannot agree with M-0019's one by one.

Our arena's houses are set up as M-0001's Setup reads: no starting units and bases off (our
`setup::skirmish` with a unit count of 0 and `bases` off), so our engine's starting-units step
(S-0009 claims 45 and 46; S-0202, *Draws during map loading*, row *Starting units*) creates no
object and takes only its final R(0, 65535). M-0019 (Setup and *Placement raw values*) finds
*starting objects* in every game: vehicles of both houses, created before the set-up's first
object, off the map at tick 0 and gone at the next tick. It reads them as the computer players'
starting construction vehicles that the harness removes, and says their raw values, if any, fall
within *N*₀. Our arena creates no such objects.

## Questions

1. Did the records' game start create a base unit (or other starting units) for each of the two
   houses, with the type choice, creation draw and placement draws of S-0009 claims 45 and 46,
   before the harness removed them? If so, with which settings (bases on, a unit count), so that
   our arena can set its houses up the same way?
2. Was anything else in the records' game start drawn that our engine's map loading and game
   start (S-0202 claim 14's order) leave out, for example a draw of the computer players' own
   start (their base planning or first choices) taken before tick 0?
3. Should claim 8's set-up, then, create and remove the two houses' starting objects before the
   first group as the records' harness did, and by which removal (with no draw)?

## Possible observation

On M-0019's map at seed 0, with the records' settings file: count the scenario stream's raw
values after the map is loaded, after the starting units step, and after each later game-start
step of S-0202 claim 14, up to the set-up's first object. The same with bases off and no
starting units. Our engine's counts per step can be given for comparison.

## Evidence (intake 77, uncommitted)

On main d645ca3 with intake 77 rebased on it (intake 74's held-target check included), YURI
against GGI and YURI against SHK pass, 12 of 12 each, with no padding; CAOS against E1 still fails
its side outcome (0/6/6). So the shortfall is not what decided the two YURI pairings.

Before intake 74, on main 23869cf: a scratch build of our arena that took unused raw values up to
*N*₀ before the set-up (a padding claim 8 forbids; reverted, not committed) played the three
records (`arena --record all`). Scored pairings failing, of 155: 15 as built at intake 77, 11
padded (M-0001 11 → 7, M-0002 3 → 3, M-0003 1 → 1). Row wins in the pairings Q-2277 and Q-2286
ask about, side A and side B of 6 games each:

| Pairing | Stock | Intake 77 as built | Padded to *N*₀ |
|---|---|---|---|
| YURI against GGI | 6 / 6 | 1 / 2 | 2 / 5 |
| YURI against SHK | 6 / 6 | 1 / 6 | 1 / 5 |
| CAOS against E1 (column wins) | 6 / 6 | 3 / 3 | 4 / 5 |

Padding moves every game's draws, so it moves vehicle pairings as much as infantry ones: under
it LTNK against E1, TELE against TTNK, TELE against ROBO, CAOS against APOC and CAOS against E2
pass and CAOS against V3 fails the win test again. On that base it did not bring YURI's side-A games near
the records': in every build we ran there (intake 71, intake 77, padded), YURI on side A won 1 or
2 of 6 against GGI and against SHK. Intake 74's change brings them to 6 of 6. The shortfall
still decides which standing position each unit reads, so claim 2's positions at tick 0 cannot be
compared with M-0019's one by one until it is settled.

## Current workaround

As claim 8 states: no padding; the draws before the set-up are our engine's game start with no
starting units. The local-lane check of *N*₀ fails and is reported as a known acceptance failure.
