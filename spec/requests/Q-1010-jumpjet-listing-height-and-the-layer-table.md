---
id: Q-1010
title: Which height lists a flying jumpjet, 208 leptons or the layer table of the render-layer spec
status: answered
related: [S-0502, S-0102, S-0209]
answered-by: [S-0502]
---

## Need

S-0502 revision 5 claim 4 says that in the grounded, ascending, descending, crashing and crashed
states each physics step lists the new position only "when its evaluated draw layer is ground
(S-0102 claim 3)", and that on flat land without bridges this "permits listing at positive heights
below 208 leptons, but not at 208 leptons or above". S-0102 claim 3's table puts a unit with jumpjet
movement in the ground layer only "on the ground, or in the air at height 0"; in the air below its
jumpjet cruise height (500 by default, never below 208) the layer is *air*, and at or above it *top*.
Read literally the two statements disagree for a type whose cruise height is above 208: at 100
leptons the table says *air*, the claim says it is listed. Scenario J3 expects listing at heights 1
and 50 and none at 300, which is the 208-lepton reading and also true of a table reading for a type
with cruise height 208.

`jump::placement_qualifies` needs one rule, also for the bridge cases (the claim says to use the
"full jumpjet layer rule", S-0102's effective height *h*′, not a blanket comparison).

## Questions

1. For a jumpjet in a placing state, is a position listed when its height *h*′ (S-0102's effective
   height) is below 208 leptons, whatever the type's cruise height, or below the type's own cruise
   height, or only at height 0?
2. In the bridge case, is "falling" in S-0102's *h*′ rule the fall of a unit dropped from a lost
   bridge deck only, or does a crashing jumpjet count as falling too?

## Possible observation

S-0502 scenario J3 with two types, cruise heights 208 and 500, listing recorded at heights 100 and
250 over flat land and over a high bridge.

## Current workaround

Interim rule: a position is listed (and marked) when *h*′ is below 208 leptons, with *h*′ =
*h* − 416 over a high-bridge cell for a unit that is not on the deck, not in a bridge-collapse
fall and has *h* ≥ 416 (S-0102), whatever the type. A crashing jumpjet is not counted as falling.
Noted in `docs/development.md` (section *Simulation (M6c: jumpjets)*).
