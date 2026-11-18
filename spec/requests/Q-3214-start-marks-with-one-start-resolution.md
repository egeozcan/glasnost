---
id: Q-3214
title: How header-pair start marks apply with quirks.double_start_resolution off
status: answered
related: [S-0009]
answered-by: [S-0009]
---

## Need
S-0009 claim 29 (r45) applies the header pairs 9 … min(N_h, 16) as start position marks twice:
in the early phase, where a chosen position's mark is its chooser's house number whatever a pair
says, and again in the late phase, where the pairs win over the early-phase marks. S-0009 Edge
cases, `quirks.double_start_resolution`, fixes the double resolution when off: "build one list,
after the terrain is loaded, and assign every position against it." Neither text says how the
marks apply when there is only that one resolution. The engine builds claim 29 now (sim batch
S27, `load::new_game`), and its switched-off path needs a rule.

## Questions
1. With `quirks.double_start_resolution` off, are the header pairs applied once? Or twice, as in
   the two phases, with the second application overwriting a chosen position's mark?
2. If they are applied once, does a chosen position keep its chooser's house number whatever a
   pair says, as in claim 29's early phase? Or does the pair win, as in its late phase?
3. Is an unread component completed from −1, the mark at the start of resolution, in that one
   application?

## Possible observation
None in the original: the switched-off state is our engine's own fixed behaviour. This is a
design question for the spec side and the lead.

## Current workaround
Our engine's interim rule, noted in `docs/dev/simulation.md` (*sim issues S27*): one
resolution, after the terrain, by claim 29's early-phase rule against the one list. Marks start
at −1, the applied pairs set theirs (an unread component keeps −1), then each chooser's house
number wins on its chosen position, and the houses that hold none are assigned in house order.
