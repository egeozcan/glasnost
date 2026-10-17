---
id: Q-1154
title: S-0215 says no rule of our engine gives an enter-nearest instruction; S-0605 claim 16 gives one
status: open
related: [S-0215, S-0605, S-0601]
answered-by: []
---

## Need

S-0215's Terms (*Enter-nearest instruction*) and claim 24 say that only the original's computer
player gives an enter-nearest instruction, that "no rule of our engine gives them", and that the
rules carrying them out (claim 24 and claim 26, row 3) "never apply" in our engine. S-0605
revision 2, claim 16, makes the bio-reactor fate of S-0601 claim 8 give the instruction *enter the
nearest absorbing building* to a taken or released unit of the neutral or special house, and relies
on S-0215 claim 24 to repeat the search at each *guard* and *area guard* processing (with
`quirks.enter_nearest_survives_absorption`). The two also differ on "already bound": S-0215 claim 24
names contact with the building only, S-0605 claim 16 names contact or waiting in line for it. The
code is `absorb::instruction_turn` and its calls in `process::process_guard` and
`process::process_area_guard`.

## Questions

1. Should S-0215's Terms and claim 24 be revised to say that S-0605 claim 16 gives the instruction
   in our engine, so that claims 24 and 26 row 3 apply?
2. For the repeat in *guard* and *area guard*, is an actor already waiting in line for the found
   building "already bound" (S-0605 claim 16), or only one in contact with it (S-0215 claim 24)?

## Possible observation

S-0605 scenario A13 variant (f) and S-0215 scenario B21 (c) together would show whether a neutral
unit waiting in line for its building is sent again by its *guard* processing.

## Current workaround

S-0605's reading is the interim, cited as "Q-1154 interim" in `process.rs` and noted in
`docs/development.md`: the fate gives the instruction, *guard* and *area guard* carry it out by
S-0215 claim 24 and claim 26 row 3 with *D*<sub>r</sub> of the behaviour being processed and no
draw, and an actor in contact with the found building or waiting in line for it is already bound.
