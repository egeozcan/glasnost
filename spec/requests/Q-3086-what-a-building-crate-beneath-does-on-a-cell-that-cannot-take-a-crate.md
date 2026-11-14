---
id: Q-3086
title: What a building's crate beneath does on a top-left cell that holds an overlay or lies outside the usable area
status: answered
related: [S-0312, S-0520]
answered-by: [S-0312]
---

## Need

GitHub issue #193. S-0312 claim 20 places a destroyed building's crate beneath with no condition:

> When the type has a crate beneath, a crate appears on the building's top-left foundation cell: a money crate (power-up 0) when the type says so, otherwise a crate with random contents (S-0520 claim 5).

S-0520 claim 5 says death crates "occupy crate places, expire and are collected by this spec's rules". It does not say which of claim 3's placement conditions apply to them.

S-0520 claim 3's random placement refuses a cell "outside the height-aware usable area" or one that "has an overlay". It puts a water-crate overlay on *water* and a wooden-crate overlay otherwise.

The same claim 20 puts rubble on every foundation cell, the top-left one included, when the type leaves rubble. Rubble is an overlay (registration index 239).

The engine now places the crate on the top-left cell itself, after the building has left its cells (`crates::place_beneath`). It needs to know what to do when that cell cannot take a crate under claim 3's conditions.

## Questions

1. A building's top-left foundation cell already holds an overlay when the building is destroyed (for example a resource overlay, or a wall overlay under a building placed over it by a map). Does the crate beneath still appear there, replacing that overlay? Or does no crate appear? Or does it appear elsewhere?
2. A building type has both a crate beneath and rubble (claim 20's rubble condition holds). Which overlay does the top-left cell hold afterwards, rubble or crate? If both rules run, which runs first?
3. The top-left foundation cell lies outside the height-aware usable area (a building at the map's edge). Does the crate beneath still appear?
4. Every crate place is occupied (S-0520 claim 1). Is the crate beneath dropped with no draw, as for a random placement?
5. On a *water* top-left cell, is the crate beneath a water crate, as claim 3 gives for a random placement?

## Possible observation

S-0312 scenario D14 variant b with a test building type that has a money crate beneath. (a) Lay a resource overlay under its top-left cell before it is destroyed. (b) Give the type rubble as well. (c) Place it so that its top-left cell is outside the usable area. (d) Fill every crate place first. Record the top-left cell's overlay after destruction, and the stream position, in each run.

## Current workaround

The crate is placed on the top-left cell with only claim 1's crate-place limit and the rule that crate overlays are named (S-0520 claim 1, S-0524 Edge cases). It replaces whatever overlay the cell holds, with no usable-area test, as claim 20 states no condition. Its overlay is the water crate on *water* and the wooden crate otherwise, as in claim 3. Rubble is not modelled, so question 2 does not arise yet.
