---
id: Q-2229
title: The exact moment an aircraft issues its next route point at a flight's end
status: answered
related: [S-0213, S-0501]
answered-by: [S-0213]
---

## Need
S-0213 r84 claim 16d: an aircraft issues its next route point when a flight
ends, at the moment it would begin to land, unless its behaviour is then
*enter*. S-0501 claim 6 has several steps at a flight's end: a further
queued waypoint outside *enter* (wait is queued and the aircraft goes on),
the pad-bound test (a pad-bound aircraft whose cell holds no building it is
in contact with falls back to rest), and the start of the landing. Claim 16d
does not say where among these the route issue comes, nor whether the
aircraft still begins to land in the tick of the issue.

S-0213 claim 16c also says that each issue is preceded by a queue of *wait*
with an immediate start. Claim 16d's moments read "each such issue" for
aircraft too, but the text is written for the fall-back to rest.

## Questions
1. At a flight's end, which comes first: a queued further waypoint, the
   route's next point, or the pad-bound fall-back? Does a pad-bound aircraft
   away from its pad issue its next point or fall back to rest?
2. In the tick it issues the point, does the aircraft also begin to land
   (and then take off again), or does it stay in flight?
3. Is the issue preceded by *wait* with an immediate start, as at a
   fall-back to rest? Is *wait* then current until the issued order's
   behaviour becomes current?

## Possible observation
S-0213 Scenario O8 part x (6), recording every tick from the aircraft's
arrival over P1: its height, current and queued behaviour, and destination.
A second run with a pad-bound aircraft type.

## Current workaround
`air::landing_trigger` issues the point after the queued-waypoint step and
before the pad-bound test, through `routes::issue_later` (the queue of
*wait*, then the issue), and returns without starting the landing (marked
`design (Q-2229)` in `crates/deterministic/sim/src/air.rs`).
