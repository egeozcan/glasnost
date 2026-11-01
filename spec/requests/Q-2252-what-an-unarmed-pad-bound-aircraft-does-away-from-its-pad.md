---
id: Q-2252
title: What an unarmed pad-bound aircraft with dock types does when a flight ends away from its pad
status: open
related: [S-0501, S-0213]
answered-by: []
---

## Need
S-0213 Scenario O8 part x (6), the pad-bound game, ends with "Record what A
does after P₂" and gives no expectation for it. A there is a pad-bound type
with a dock type, and in our synthetic copy of that game it is unarmed. It
left its pad (and its contact with it) to fly the route.

Applied as written, the rules give this after P₂, with no point left to
issue (S-0213 claim 16d: "S-0501 claim 6's pad-bound test and landing follow
unchanged"):
- S-0501 claim 6: in its destination cell, a pad-bound aircraft whose cell
  holds no building it is in contact with does not land; it falls back to
  rest (claim 18).
- S-0501 table T2: in flight, with no passengers, unarmed and not in a team,
  the only row that applies is "in flight, otherwise": destination the F8
  landing spot, *move*. The rows that search for a pad ("a pad is found and
  grants contact", "no pad, pad-bound: crash", and the after-the-row new pad
  search) are all for an armed aircraft.
- F8: the house's buildings of the dock type are compared, the pad is chosen,
  and the free spot near the pad is taken. Once the aircraft is within 256
  leptons (weighted) of the pad, F8 takes a free cell near its own cell.

So A flies to a cell next to its pad, slows there, and from then on its
landing trigger fires at every tick: it falls back to rest, gets the same
F8 cell and *move* again, and hovers at cruise height above that cell for as
long as the game lasts. It never asks the pad for contact, never docks and
never lands. In our engine this goes on for well over 1,300 ticks (test
below). Nothing in S-0501 says this is what the game does. Its armed cases
are covered (claims 45 and 46, AC10), but the unarmed pad-bound case with
dock types is not.

## Questions
1. An unarmed, pad-bound, landable aircraft type that has a dock type flies a
   move order to a far cell, away from its own free pad of that dock type,
   and arrives there. What does it do then: fly to the pad and dock, crash,
   land somewhere, or hover?
2. If it flies towards a cell near its pad (an F8 landing spot): once there,
   does it stay in the air above that cell for good, with *move* current? Or
   does something else happen later (it asks the pad for contact, it lands
   on that cell, it crashes)?
3. Does the answer change when the pad has no free docking place, or when
   the house has no building of its dock types at all?
4. Is the answer the same after an ordinary move order as after a route's
   last point (S-0213 claim 16d)?

## Possible observation
S-0213 Scenario O8 part x (6)'s pad-bound game, with A's type a pad-bound
test copy without a weapon (test values). Record A's behaviours, destination,
contact, height and cell at every tick from its arrival over P₂ for 3,000
ticks. Repeat with a plain move order to P₂ instead of the route. Repeat
with A's pad occupied by a second aircraft of the same type, and with the
pad sold.

## Current workaround
The engine applies the rules as written (S-0501 claim 6, table T2, F8), so
A hovers above its F8 cell next to the pad under *move* and falls back to
rest at every tick. No code change. Test:
`claim_16d_after_p2_an_unarmed_pad_bound_aircraft_falls_back_above_its_f8_spot`
(`claims_s0213_r89.rs`) pins this until the request is answered.
