---
id: Q-1012
title: Which clears of the order-level destination invoke the jumpjet stop reroute
status: open
related: [S-0502, S-0213, S-0215, S-0517]
answered-by: []
---

## Need

S-0502 revision 5 claim 16 says "A direct no-point instruction is distinct from an order-level
destination clear: the latter invokes claim 17", and claim 24 says that a stop order of S-0213
claim 8 clears the order-level destination and invokes claim 17 when the jumpjet is moving. Only the
stop order is named. Other orders and behaviours also set the order-level destination to nothing for
a unit that may be a moving jumpjet: the deploy order (S-0213 claim 10, "destination nothing"),
behaviour changes (S-0215), the end of a move behaviour. The code treats every change of the
order-level destination of a moving jumpjet to nothing as claim 24 does (`jump::take_destination`
with no destination invokes `jump::stop_with`), because a deploy order to a hovering landing-first
vehicle (S-0517 claim 5) must still be able to land.

## Questions

1. Does every clearing of the order-level destination of a jumpjet with a movement commitment
   reroute it as a stop does, or only the stop order (and the failed landing)?
2. If only the stop order, what happens to the movement point when a deploy order clears the
   order-level destination of a hovering landing-first vehicle: is the point kept, so that its
   landing (S-0502 claim 15, "its position becomes the destination point") has a point?

## Possible observation

Order a hovering landing-first jumpjet vehicle to deploy (S-0517 P1 b) and record its movement point
before and after, and where it lands.

## Current workaround

Interim rule: every clear of the order-level destination of a moving jumpjet (a hovering one that
holds a move order included) is claim 17's reroute with the claim 23 search settings; the order-level
destination stays empty and the locomotor holds the replacement point. A jumpjet that is not moving
does nothing. Noted in `docs/development.md` (section *Simulation (M6c: jumpjets)*).
