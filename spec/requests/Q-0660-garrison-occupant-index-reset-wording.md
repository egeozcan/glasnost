---
id: Q-0660
title: When the garrison occupant index returns to 0 (S-0311 Terms against S-0308 and S-0313)
status: open
related: [S-0311, S-0308, S-0313]
answered-by: []
---

## Need
S-0311's Terms (*Garrisoned building, firing occupant*) still say the shared occupant/muzzle index
"returns to 0 when occupants leave or are killed". S-0308 revision 7 (claim 13 and its table) and
S-0313 claim 6 now say it returns to 0 only when the building is emptied by occupants leaving or
being killed, and that an individual departure or death while others remain is not a reset rule.
S-0308's open questions say S-0311's Terms must be narrowed to the same condition. The firing
occupant named by S-0311 (kill credit and experience for garrison shots) depends on which reading
holds.

## Questions
1. Does the index return to 0 only when the building is emptied, as S-0308 claim 13 and S-0313
   claim 6 state, so that S-0311's Terms are to be read with that condition?
2. When one occupant leaves or dies while others remain, what happens to the index (kept as is,
   kept but reduced modulo the new count, or something else)? S-0308 says this is not established.

## Current workaround
Garrisons are not built; nothing reads or resets the index yet. No code depends on the answer.
