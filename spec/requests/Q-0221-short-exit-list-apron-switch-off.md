---
id: Q-0221
title: The exit apron of a short exit list with the switch off, an outlying apron, and the full-ring exit lists
status: answered
related: [S-0417, S-0404]
answered-by: [S-0417]
---

## Need

S-0417 claim 3 takes the exit apron cell E from the eleventh pair of a foundation's exit list
(S-0404 Constants). For shorter lists, the proposed `quirks.factory_apron_short_list` gives the
original's special pairs: (32767, 32767) for the 2 × 1 and 1 × 2 words, wrapping to an outlying
cell, and (0, 0) for the 1 × 1, 3 × 3 refinery, 6 × 4 and 0 × 0 words. Open question 5 leaves
the switch's fixed alternative and the handling of an apron that is not a legal map cell to the
owner. Open question 6 asks whether, with S-0404's `quirks.foundation_exit_lists` off, the
apron keeps its original E or uses the eleventh entry of the full ring.

The switch must have an off state for the schema, and the engine must do something when E is
not a legal map cell (the exit track would carry the vehicle off the map).

## Questions

1. With `quirks.factory_apron_short_list` off, which cell is E for each short-list foundation
   word (2 × 1, 1 × 2, 1 × 1, 3 × 3 refinery, 6 × 4 and 0 × 0)?
2. When E is not a legal map cell (the outlying (32767, 32767) result, or a factory at the map
   edge), what happens to the apron test, the exit instruction and the factory's unloading?
3. With `quirks.foundation_exit_lists` off, does the apron keep the original list's E (S-0417's
   proposal) or take the eleventh entry of the full ring, and what happens when the ring has
   fewer than eleven entries?

## Current workaround

- Switch on: the original pairs. Switch off: the last pair of the short list, and (0, 0) for the
  empty 0 × 0 list.
- E not a legal map cell: no object blocks it, and at letting out the exit instruction is
  refused; the unit gives its clear report at once so the factory can close (no permanent wait,
  S-0417 Open question 4).
- `quirks.foundation_exit_lists` off: the original list's E.

Interim row in `docs/development.md` (Q-0221).
