---
id: Q-1163
title: S-0606 claim 20, the order of the S1 row and the depot's repair processing within one processing of a type that is both
status: open
related: [S-0606, S-0314]
answered-by: []
---

## Need

S-0606 revision 4, claim 20, says that for a building type with both the bunker flag and the
unit-repair flag, "each processing carries out at most one row of table S1 and also runs the
depot's repair processing (S-0314 claim 6). The delay is the one the depot's processing chooses."
It does not say which of the two runs first. The two read and write shared state: the S1 row can
queue *guard* on the building, take the vehicle in (it then starts *guard* at once, claim 7) or
send it on the entry track. The depot's processing reads the serviced vehicle's contact and
behaviour, and it can queue a behaviour on the building or send the vehicle away. The outcome of
a processing can therefore depend on the order. Open question 8 leaves the overall outcome open,
but the order within one processing is a separate fact.

## Questions

1. In one *repair* processing of such a building, does the S1 row come before the depot's repair
   processing, or after it?
2. When the S1 row queues *guard* on the building (its last row, or claim 16's return to
   clearing) and the depot's processing then queues something else, which queued behaviour
   remains?

## Possible observation

Scenario K9 of S-0606 (the combined test fixture), with the recorder reading the building's queued
behaviour, the bunker stage and the vehicle's health after every processing.

## Current workaround

Interim rule, cited as "Q-1163 interim" in `docs/m7-claims/step1k-m7a.md` and
`docs/development.md` (Simulation, M7a: Vehicle bunker). The S1 row runs first
(`structures::bunker::process_repair`), then the depot's repair processing
(`depot::process_repair`), whose delay is returned. A later queue replaces an earlier one, as
usual. Test: `claim_20_combined_roles`.
