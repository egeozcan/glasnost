---
id: Q-1622
title: The fixed cursor palette of the bomb icon and the wrench is not found by the search order
status: answered
related: [S-0111, S-0519, S-0005]
answered-by: [S-0111]
---

## Need
S-0111's Import mapping says the wrench sprite (claim 19) and the bomb icon sprite (S-0519
claim 24) use the palette file `MOUSE.PAL`, resolved by S-0005. S-0519's Import mapping gives the
bomb icon a "fixed cursor palette", owned by S-0111. On the local lane's install, no loose file and
no member of any archive in S-0005 Table 1 resolves under that name, whether before a game, while
it loads or in play, for any side group. A scan of every archive of the install, nested archives
included, finds no member with that name's id either. The bomb icon image itself
(`BOMBCURS.SHP`) does resolve. So the engine draws no bomb icon at all on a stock install, though
S-0519 T25 expects the icon to show. S-0111 says a missing sprite draws nothing and the importer
warns; it gives no rule for a missing palette. The code is `mapview::combat` (`MOUSE_PALETTE`,
`Cache::bomb_item`) and the load-time note in `mapview::live`.

## Questions
1. Which palette does the original draw the bomb icon (and the repair wrench) through, given that
   a file of the name `MOUSE.PAL` is not in the stock archives' search order? Is it a different
   file name, a palette held in another resource, or a palette built in memory?
2. If the palette is named by a file that can be missing, what does the original draw when it is
   missing, and what should our engine do: draw nothing and warn (as for a missing sprite), or fall
   back to another named palette?

## Possible observation
S-0519 T25 on a stock install: capture the bomb icon and compare its pixels with each frame of
the bomb icon image under the stock archives' candidate palettes. S-0111's wrench scenario gives a
second sample of the same palette.

## Current workaround
Interim rule: the bomb icon is drawn only when both the bomb icon image and a palette resolved
under the name `MOUSE.PAL` are present. Otherwise no icon is drawn and a note says so once, at
load, the same handling as S-0111's missing-sprite rule. On the stock install this means no bomb
icon shows. Placement, frame (S-0519 F3) and drawing order (claim 24) are implemented and tested
with synthetic assets.
