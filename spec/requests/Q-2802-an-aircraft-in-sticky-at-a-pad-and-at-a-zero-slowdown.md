---
id: Q-2802
title: An aircraft in sticky that reaches a pad, and one with no slowdown distance
status: open
related: [S-0501, S-0215]
answered-by: []
---

## Need

S-0501 r150 claim 29 gives the processing of *sticky* for an aircraft: the outcomes of *guard*,
with every queued behaviour waiting, since an aircraft in *sticky* is never ready to change (claim
31). Our engine builds it in `air::guard` (`crates/deterministic/sim/src/air.rs`). The spec's Open
questions 32 and 33 leave three cases open, and each has an interim in the code with a
`// design: Q-2802` cite.

## Questions

1. (Open question 32.) An armed aircraft in *sticky* below its cruise height finds a pad and flies
   to it, with *enter* queued and no contact. What does it do on reaching the pad without *enter*
   current: does it land on the pad, and is it serviced (claim 26)?
2. (Open question 32, AC33 part c.) An armed pad-bound aircraft in *sticky* at cruise height is
   granted a pad's contact at its processing, and turns toward the pad with *enter* queued. Its next
   processings come 1 tick apart while it keeps that height. Do they keep the pad as its destination
   and keep the contact, or end the contact and give it another destination or queued behaviour?
3. (Open question 33.) An unarmed landable aircraft in *sticky* outside a team, given the cell it
   is over at each processing at cruise height, has a slowdown distance of 0, or a converted speed
   below 20. Where does it stop, and does it land? The same for an unarmed type that cannot land
   and that claim 5 holds at full speed: where does it fly?

## Possible observation

AC33 part c, recorded on past the processing it names, answers question 2. AC33 part a with a test
type whose slowdown distance is 0, and again with a converted speed below 20, answers question 3.
For question 1: AC33 part b's type with one round spent before it is placed in *sticky* in flight
below its cruise height, with a pad on the map, recording its behaviours, contact, height and
ammunition.

## Current workaround

The rules as they stand in our engine, with nothing added for *sticky*:
1. It reaches the pad with *enter* queued and no contact. Its flight and *guard*'s rules (claims 6
   and 18) decide what follows; nothing in our engine makes *enter* current for it.
2. Each processing runs claim 29's armed outcome again: the fall-back of claim 18 and T2, which
   keeps the contact it holds and the pad as its destination, with *enter* queued.
3. Its flight takes each new cell as any destination (claim 50's rules for a flight to a cell),
   and it lands where claim 6 lets it.
