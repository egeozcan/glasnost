---
id: Q-1031
title: When a jumpjet vehicle's destination-refresh state is active and when a balloon-hover vehicle substitutes its retained destination
status: answered
related: [S-0502, S-0213, S-0517]
answered-by: [S-0502]
---

## Need

S-0502 revision 6 introduces the destination-refresh state (Terms) and says its "exceptional
activation conditions remain unmeasured here", and claim 24 says a balloon-hover vehicle "can first
substitute its retained destination while it still has a target". Both are exceptions to the
suppression of an unchanged vehicle destination assignment. The specs give no rule for when either
applies, so the code cannot implement them: `jump::stop_ordered` and `jump::take_destination` have no
condition to test.

## Questions

1. Under what conditions does a jumpjet vehicle's destination assignment have to be reapplied even
   though the order-level destination is unchanged (the destination-refresh state)?
2. Under what conditions, and with what result, does a balloon-hover vehicle that has a target
   substitute its retained destination for the destination being assigned?
3. Can either happen through an ordinary stop, deploy, move or behaviour sequence, or only through
   states ordinary play does not reach?

## Possible observation

S-0502 J22's diagnostic extensions (record the refresh state and the substitution with a balloon-hover
vehicle that holds a target).

## Current workaround

Interim rule: neither exception exists in our engine. An unchanged assignment to a vehicle is
always ignored; a balloon-hover vehicle never substitutes. Noted in `docs/development.md` (section
*Simulation (M6c: jumpjets)*).
