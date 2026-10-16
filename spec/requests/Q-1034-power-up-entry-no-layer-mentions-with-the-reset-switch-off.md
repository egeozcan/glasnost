---
id: Q-1034
title: The held value of a power-up entry that no layer mentions when the reset-on-override switch is off
status: open
related: [S-0006, S-0520]
answered-by: []
---

## Need

S-0006 claim 5 says that with `quirks.override_resets_keys` off the game-wide power-up rows keep the
held value, and that for a key no earlier layer gave "this spec fixes each of them at the reset value of
the table (a design choice)": weight 0 and no animation. S-0520's Import mapping says that at a fresh
program start the table holds the built-in weights 50, 20, 1, 3, 5, 5, 20, 1, 1, 10, 10, 10, 1, 3, 1, 1,
1, 1, 1 and no animations, and that a layer without the section changes none of the entries. The two
agree when no layer holds a `[Powerups]` section. They differ when a layer holds the section, the switch
is off, and a given entry is mentioned by no layer: S-0006 gives weight 0, S-0520's start table gives the
built-in weight.

## Questions

1. With the switch off and a section present in some layer, does an entry that no layer mentions have
   weight 0 and no animation (S-0006's fixed default) or its built-in start value (S-0520)?

## Current workaround

Interim rule: S-0006's fixed default (weight 0, no animation) for an entry that no layer mentions; an
entry that an earlier layer gave keeps its values when a later layer omits it (the switch-off meaning of
claim 28). The default switch setting (on) is unaffected. Noted in `docs/development.md` (section
*Simulation (M6c: crates)*).
