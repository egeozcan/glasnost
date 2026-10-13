---
id: Q-0140
title: Where a harvester docks at a refinery and how it gets there
status: open
related: [S-0402, S-0207, S-0212, S-0215]
answered-by: []
---

## Need

S-0402 claim 10 says that in *enter* the harvester "drives to the refinery's dock and backs into
the docking cell". The Terms define the docking cell only from the unloading side: it is the
harvester's own cell, and the refinery occupies the neighbouring cell in step direction 6 (map
west). The spec leaves open (S-0402 Open questions, first item):

- which cell of a given refinery the docking cell is, relative to its top-left cell and its
  foundation *W* × *H*;
- how the harvester reaches it: the path, backing in, and its facing on arrival;
- when *enter* hands over to *unload*;
- how often *enter* is processed.

Claim 8's connectivity test also needs a cell of the refinery to test against.

The engine needs this in `econ::harvest::docking_cell` and `econ::harvest::process_enter`.

## Questions

1. For a refinery with top-left cell (*x*₀, *y*₀) and foundation *W* × *H*, which cell is its
   docking cell? Does the art move it, for example through a dock offset or the dock-unload flag?
2. How does a harvester in *enter* reach that cell? Possible answers include an ordinary path to
   the cell, or a path to a cell in front of it and then backwards. Which facing does it have on
   arrival?
3. At which processing does *enter* give way to *unload*: on arrival, at rest, or after a turn?
   Is *unload* started at once or queued?
4. What is the processing delay of *enter* for a harvester, and does it take a draw?
5. Which cell does claim 8's movement-zone connectivity test use for a refinery: the docking cell,
   the top-left cell, or another?

## Possible observation

Run scenario H1 and record, for each tick from the *requesting* phase to the first payment, the
harvester's cell, facing and behaviour. Use refineries of two foundation sizes.

## Current workaround

- **Docking cell:** (*x*₀ + *W*, *y*₀ + ⌊*H*/2⌋), the cell just east of the foundation's middle
  row, so the refinery occupies the cell's step-direction-6 neighbour.
- **Approach:** in *enter* the harvester drives to the docking cell by an ordinary path, with the
  docking cell as its destination and the standard delay of *enter* between processings.
- **Hand-over:** once it stands on the docking cell with no movement in progress, *unload* is
  queued and started at once. Claim 11's turn to map east follows.
- **Connectivity:** claim 8's test uses the docking cell.
- **Enter order:** an *enter* order on a refinery that accepts the harvester takes contact at the
  first processing.

This is listed in the provisional table of `docs/development.md`.
