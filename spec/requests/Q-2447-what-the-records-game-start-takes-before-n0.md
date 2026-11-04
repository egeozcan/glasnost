---
id: Q-2447
title: What else do the records' game starts take from the scenario stream before M-0019's N₀?
status: answered
related: [S-0202, S-0216, S-0009, M-0019]
answered-by: [S-0216]
---

## Need

S-0216 revision 67 claim 8 (*Game start* and *Removal before the groups*) and S-0202 revision 62
claim 14 (*House creation*) answered Q-2336. We built both (intake 87). Our arena's game start
now has five houses in S-0009 claim 19's order: the human player house at start 3, side A, side
B, the neutral house and the special house. Every player has a fixed start. The bases setting is
on, the unit-count setting is 0 and crates are off. The start takes two rounds of five
R(450, 1800) draws, then the rest of map loading, three base units with one creation draw each,
and the final R(0, 65535).

S-0216 claim 8's *Acceptance* requires the first set-up object's creation draw to take the raw
value at place *N*₀ of M-0019 for its seed. That still fails in every seed, and the miss is not
the same sign in every seed. The local-lane test `arena::m0019_the_first_creation_draw_takes_the_raw_value_at_n0`
gives, by seed (raw values, counted from place 0 as M-0019 counts them):

| Seed | 0 | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|---|
| Ours before claim 14 and *Game start* (r54 and r59) | 693 | 685 | 685 | 695 | 693 | 689 |
| Ours now (r62 and r67) | 711 | 698 | 702 | 709 | 711 | 708 |
| M-0019's *N*₀ | 706 | 709 | 711 | 718 | 706 | 701 |
| Ours now − *N*₀ | +5 | −11 | −9 | −9 | +5 | +7 |
| Round 1's raw values (places from 0) | 6 (0 to 5) | 6 (0 to 5) | 6 (0 to 5) | 9 (0 to 8) | 10 (0 to 9) | 8 (0 to 7) |
| Round 2's raw values | 6 (6 to 11) | 5 (6 to 10) | 8 (6 to 13) | 5 (9 to 13) | 7 (10 to 16) | 8 (8 to 15) |
| Ours after round 2, to the first creation draw | 699 | 687 | 688 | 695 | 694 | 692 |
| M-0019's *N*₀ minus our two rounds | 694 | 698 | 697 | 704 | 689 | 685 |

Notes on the numbers:

- Every start is fixed, so our early phase takes no draw, and the two rounds are adjacent from
  place 0. On the records' map this was checked as follows:
  - the map defines start waypoints 0 to 3, so the three players need no random start cell;
  - the houses get starts 3, 0 and 1, so no random-position draw is taken (asserted in
    `arena::pt6_the_records_game_start_in_our_setup`);
  - the other steps between the rounds (the shroud start, the trigger set-up and the radar flags)
    take no draw (read in the code). The synthetic R17 test `claims_s0202_houses::claim_14_two_house_rounds_then_the_final_draw`
  checks that order exactly (2 to 4 players, seeds 1 to 5).
- Each R(450, 1800) draw has d = 1350, so it keeps the lowest 11 bits of a raw value and rejects
  values above 1350 (S-0202 claim 4). `quirks.ranged_draw_half_span` is on, but it changes only
  spans of 2147483647, so it has no effect here.
- The place of the first creation draw is the same in every pairing of a seed (the test checks
  that).
- In all three rows, seeds 0 and 4 give the same total (693, 711, 706), although their two rounds
  take 12 and 17 raw values.
- The removal takes no draw, so it cannot move *N*₀. It removes the two computer houses' base
  units and the 17 vehicles the map gives the neutral house.
- Even before claim 14, our count differed by seed (685 to 695). So our map loading takes
  draws that can take more than one raw value apart from the house rounds. We have not
  instrumented which they are. Because such draws exist, a draw that we lack or take too many
  of anywhere before them shifts what they take. So the miss can depend on the seed even if
  its cause is a fixed count of draws.

We do not know which draws of the records' game start we lack or take too many of, so we have
not changed anything to fit the numbers.

## Questions

1. Between the seeding and the first set-up object's creation draw, which random draws did the
   records' games (M-0001 to M-0003, M-0016, M-0017, M-0019) take that S-0202 claim 14, S-0009
   and S-0216 claim 8 do not give, and which of the draws those specs give did the games not
   take? For each, give its range, how many there are, and where it falls relative to the two
   house rounds, the base units' creation draws and the final R(0, 65535).
2. In those games, how many houses did each round of S-0202 claim 14 create, and in which order?
   Were there any houses beyond the five in S-0216 claim 8 (for example, empty player or
   observer slots that the settings file fills)?
3. Do the records' games take any draw between the two rounds when every player's start is
   fixed?

## Possible observation

M-0019 counts the raw values taken before tick 0 in each game. The same recorder, run on a
records' game start at a few chosen points (after the house rounds, after map loading, after
the starting units), would show where our count first departs. R17 (S-0202) run on the records'
map with its five houses would settle question 2.

## Current workaround

Our arena takes exactly the draws that S-0202 r62 and S-0216 r67 give. The local-lane
acceptance test still fails and prints the per-seed numbers above. We have changed nothing to
fit them.
