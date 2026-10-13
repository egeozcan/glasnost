---
id: Q-0180
title: A pre-placed building with the switched-on flag 0 whose type a power-off order would not switch off
status: open
related: [S-0009, S-0405]
answered-by: []
---

## Need
S-0009 revision 13, claim 40 ("Switched-on flag"), says that a building whose switched-on field
is 0 "starts switched off, exactly as if its owner had switched it off in play (S-0405 claim 10)".
S-0405 claim 10 says an applicable power-off order switches an on building off only when its type
has a base drain above 0 or the needs-power flag, and that the order does not apply at all to a type
with the firestorm-wall flag. The loader (`glasnost-sim`, `objects.rs`, pre-placed buildings) marks
every such building switched off, whatever its type. It cannot tell from the two specs whether the
conditions of S-0405 claim 10 also decide the start state.

## Questions
1. A pre-placed building with switched-on field 0, whose type has a base drain of 0 and no
   needs-power flag (for example a power plant type): does it start switched off, or switched on?
2. The same for a type with the firestorm-wall flag.
3. If it starts switched off, does a later power-on order switch it on in the usual way (S-0405
   claim 10)?

## Possible observation
A test map with two pre-placed buildings owned by the human player, both with switched-on field 0:
one of a type with a base drain above 0, one of a power-producing type with base drain 0 and no
needs-power flag. Observe the power bar and each building's power-switch indicator at the first
drawn frame, then give each a power-on order.

## Current workaround
Every pre-placed building with switched-on field 0 starts switched off, whatever its type (the
reading that existed before revision 13). No quirk switch.
