---
id: Q-1610
title: When a working repair depot reacts to losing its vehicle's contact
status: answered
related: [S-0314, S-0108, S-0504]
answered-by: [S-0314, S-0108, M-0007]
---

## Need
S-0314 claim 22 (revision 31) says a depot left without any contact "reacts at its next repair
processing", and in the working phase it then enters settling and rechecks after 1 tick. S-0314
claim 6 spaces the working phase's processings by the repair steps: the second step falls due
*D* + *K*<sub>u</sub> − 1 ticks after the settling step, each later one *K*<sub>u</sub> ticks after
the one before. Nothing in S-0314 brings a processing forward when the contacted vehicle is
destroyed or leaves.

S-0108 claim 20 (revision 17) says that when contact ends without a step, "the depot reacts at its
next processing: on the same or the next tick in the working and settling phases, at the waiting
phase's own spacing otherwise".

The two read differently for a working depot. With S-0314's spacing alone, a vehicle destroyed
just after a working step leaves the depot in working, with its repair behaviour and contact
place unchanged, for up to *K*<sub>u</sub> ticks; S-0108 says the reaction (and its slot changes:
*production* and *special-2* stop, *special-3* and *active-1* start) comes on the same or the next
tick. Our depot (`glasnost_sim::depot::process_repair`) follows S-0314: a synthetic test with an
invented repair rate shows the working depot reacting 27 ticks after the vehicle was destroyed.

The settling phase has the same question after a *cannot pay* answer, whose retries are *D* ticks
apart (claim 6).

## Questions
1. When the vehicle a working depot is servicing is destroyed (or leaves under S-0314 claims 20
   to 22) between two working steps, on which tick does the depot enter settling: at the next
   working step's due tick, or on the same or the next tick after the contact ended?
2. The same for a settling depot that is waiting *D* ticks between *cannot pay* retries.
3. If the reaction comes on the same or the next tick, does the step spacing of claim 6 then
   resume unchanged for a vehicle that takes contact afterwards, or does that processing restart
   the depot's schedule?

## Possible observation
S-0314 scenario H4 part d with S-0108 B18 part d: a depot with an invented long repair rate
(large *K*<sub>u</sub>), a damaged vehicle in its working phase, destroyed by scripted damage one
tick after a working step. Record per tick the depot's behaviour, the slot animations running and
their start ticks, and when the depot returns to *guard*.

## Current workaround
S-0314's spacing: the depot notices the lost contact at its next scheduled repair processing
(the next working step's due tick, or the next settling retry), then follows claim 22's rows
(working: settling and a 1-tick recheck; settling: *guard* once *special-3* no longer runs, at
once while the simulation has no slot model). No processing is brought forward.
