---
id: Q-0142
title: A harvester's unload without contact, and orders given while it unloads
status: open
related: [S-0402, S-0215, S-0213]
answered-by: []
---

## Need

The first row of S-0402 claim 11 covers a harvester in *unload* that is in contact with nothing.
It "stops, clears its destination and falls back to rest", and its next processing is "as its
resting behaviour". But S-0215 claim 15 (the fall-back) queues nothing while *unload* is current,
so *unload* would stay current and the row would repeat for good.

The spec also leaves open how an order given while the harvester unloads ends unloading. Its Open
questions note only that the original "appears to treat it like an empty load".

The engine needs this in `econ::harvest::process_unload`.

## Questions

1. After the first row of claim 11, which behaviour does the harvester have at its next
   processing, and after how many ticks?
2. An order (a destination and a queued behaviour) reaches a harvester that is unloading. What
   happens at its next processing of *unload*: does it keep its load, become *finishing*, or
   leave at once?
3. When an order breaks off contact (S-0213 table T3 (b)), is the refinery's dock slot freed at
   once?

## Possible observation

Run scenario H1 with a move order given in the tick after the first payment. Run a variant in
which the refinery is sold during unloading. Record the harvester's behaviour, load and contact
for each tick.

## Current workaround

- **First row:** runs as written: the harvester stops, its destination is cleared, and S-0215
  claim 15's fall-back runs. When that leaves nothing queued, the harvester is forced into
  *guard* in the same processing, with delay 1.
- **Order while unloading:** the order breaks off contact (S-0213 table T3 (b)), so the next
  processing takes the first row, with the order's behaviour queued.
- **Dock slot:** a slot counts as held only while its harvester still holds contact.

This is listed in the provisional table of `docs/development.md`.
