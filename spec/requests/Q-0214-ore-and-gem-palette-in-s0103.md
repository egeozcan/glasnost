---
id: Q-0214
title: S-0103 still leaves the ore and gem overlay palette open after S-0104's hand-off note
status: open
related: [S-0103, S-0104, S-0112]
answered-by: []
---

## Need
S-0104's *Cross-spec changes required at hand-off* says that S-0103 claim 8 and its open
questions must delegate the ore and gem overlay palette to S-0112 claim 1. S-0103 revision 4 still
says in claim 8 that the palette of ore and gem overlays "is not established (Open questions)",
and its Open questions still list it. The map viewer draws ore and gems with S-0112 claim 1's
start-up resource table (`app::mapview::build`, `startup_row`). That code cites S-0112, and S-0103
contradicts it.

## Questions
1. Is S-0112 claim 1 (row 3: the start-up resource table at brightness 1000) the rule for ore and
   gem overlays, so that S-0103 claim 8 should point to it?
2. Should S-0103's open question on "Ore, gems and other objects" be narrowed to the overlays,
   smudges and terrain objects S-0112 does not cover, or closed?

## Possible observation
None needed if the S-0104 hand-off note stands; S-0112's own scenarios already back claim 1.

## Current workaround
The viewer follows S-0112 claim 1 and S-0104 claim 13. No code change is waiting on this.
