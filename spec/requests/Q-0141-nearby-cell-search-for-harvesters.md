---
id: Q-0141
title: The nearby-cell search around a queueing cell and off a refinery
status: answered
related: [S-0402, S-0212]
answered-by: [S-0402]
---

## Need

Two claims of S-0402 send a harvester to a free cell:

- Claim 9 sends a harvester that has not reserved a refinery to the refinery's queueing cell "or
  the nearest free cell around it". With no such cell, the harvester drives towards the refinery
  itself.
- Claim 13 moves an idle harvester off a refinery-flag building "to a free cell nearby".

Neither claim says which search finds that cell, how far it looks, what counts as free, or how
ties are broken (S-0402 Open questions, last item).

The engine needs this in `econ::harvest::queueing_cell` and `econ::harvest::step_off_refinery`.

## Questions

1. Which search finds the free cell around the queueing cell: S-0212's nearby-cell search (and
   with which use), or another? What is its extent, and how does it order cells that are equally
   near?
2. Is the queueing cell itself used when it is free? What makes it free: the entry outcome
   *clear* for the harvester, no object at all, or something else?
3. Questions 1 and 2 again, for the idle harvester's step off a refinery-flag building
   (claim 13).
4. When the search finds nothing, where exactly does "towards the refinery" send the harvester?

## Possible observation

- Run scenario H4 part b with parked vehicles blocking the queueing cell in growing rings.
- Run scenario H5 with an idle harvester placed on a refinery cell.

In both, record the harvester's destination cell.

## Current workaround

- **Queueing cell:** used when the harvester's entry outcome there is *clear* (S-0209 claim 5).
- **Otherwise:** S-0212's nearby-cell search, starting at the queueing cell and using the
  substitution use for the harvester, gives the cell.
- **No cell found:** the harvester's destination is the refinery object.
- **Idle step-off:** the same nearby-cell search, from the harvester's own cell.

This is listed in the provisional table of `docs/development.md`.
