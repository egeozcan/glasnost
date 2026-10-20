---
id: Q-1527
title: A protection colour selector that names no position of the colour-adjustment table
status: open
related: [S-0102, S-0513, S-0006]
answered-by: []
---

## Need
S-0102 claim 34 and F9 and S-0513 Import mapping take the curtain and force-shield
colour selectors (`[AudioVisual]` `IronCurtainColor`, `ForceShieldColor`) as zero-based
positions in the 16-entry colour-adjustment table (S-0006 claim 2). Neither spec says what
a protected drawing does when its selector is negative or 16 or more, which rules files can
give. The game client (`crates/app/src/mapview/protection.rs`, `Masks::read`) has to pick the
mask the curtain-protected voxel vehicles and force-shielded buildings take.

## Questions
1. With a selector below 0 or above 15, does a curtain-protected voxel vehicle or a
   force-shielded building take any colour mask, and if so which triple (another table
   position, a fixed value, a value from outside the table)?
2. Does such a selector change anything else: the brightness envelope (S-0513 claim 29),
   the other selector's drawings, or any other drawing?
3. Is the selector reduced in some way first (modulo the table size, clamped, or read as an
   unsigned byte), so that, for example, 16 or −1 selects a valid position?

## Possible observation
In a named test map, set each selector in turn to −1, 16 and a large value such as 300 while the
table holds distinct invented triples at every position, then capture a curtain-protected voxel
vehicle and a force-shielded building on neutral ground as in S-0513 SW10 part g; compare with
the unprotected pixels and with each table position's expected F9 union.

## Current workaround
Interim rule (docs/development.md, *Visual review 2026-10-19*): a selector outside 0 … 15 gives
the mask K = 0, so the drawing keeps only the brightness envelope; the scene notes it.
