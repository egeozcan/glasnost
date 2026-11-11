---
id: Q-2981
title: Does an object's added level-factor term switch at once when a dominator ends?
status: open
related: [S-0104]
answered-by: []
---

## Need

S-0104 claim 13 adds a level-factor term λ to some objects' brightness: 4λ for a vehicle on a bridge deck, (*h* ÷ 208) × λ for infantry on a deck or high over a shadowed cell, and (*h* ÷ 208) × λ<sub>air</sub> for aircraft. λ is the level factor of the set chosen by claim 2's per-cell ranking. The claim says it is open "whether an object's added λ term switches immediately after a dominator ends, while its cell still retains dominator height terms" (also listed under Open questions).

Render issue #74 (GitHub, render audit #154) made live vehicles, infantry and aircraft take these terms (merged 38e51cd). The client has to pick λ in that interval and needs the answer.

## Questions

1. A dominator effect ends, and a cell still holds its dominator height terms. For a vehicle on a bridge deck in that cell, which level factor does the vehicle's 4λ term use until the cell is recomputed: the dominator set's factor, or the factor of the set chosen after the effect ends?
2. Does the same hold for infantry on a deck, and for infantry more than 208 leptons up over a shadowed cell, in that interval?
3. Aircraft use λ<sub>air</sub>, which claim 13 says is never the dominator factor. Is anything about an aircraft's brightness different in that interval?

## Possible observation

Start and end a dominator effect over a high bridge with a vehicle and an infantry unit standing on the deck. Sample the units' drawn shade rows on each frame from the effect's end until the deck cells' light is recomputed, against the same units under a control with no effect.

## Current workaround

The client stops the λ term from the dominator set when the global choice moves off it, without waiting for the cell. This is marked `design: render (#74)` in `crates/render` and recorded in `docs/m8-claims/render-issues.md` (batch R2, #74) as an interim until this request is answered.
