---
id: Q-2649
title: The entry report and the service after a touchdown in a cell next to the docking position's
status: answered
related: [S-0501]
answered-by: [S-0501]
---

## Need

S-0501 r132 claim 24 (*Not pad-bound*): "After a touchdown in a cell next to it, untested as every
touchdown is (claim 7), its own forget test does not apply, so it keeps the pad as destination. Its
next processing then finds it not landing and reports entry, and its movement follows claim 7's
after an untested touchdown that kept the destination. Where it is then serviced is not traced."

The engine has to make that report and decide when the pad services an aircraft that is still
sliding. Claim 26 skips an aircraft that is still moving or flying in. Our engine applies claim
25's landing facing at the docking stage's report after a touchdown (Open question 4's interim),
and turning a sliding aircraft would change its forward step.

## Questions

1. Does the report after such a touchdown turn the aircraft to claim 25's landing facing, as a
   report after a touchdown in the docking position's cell may, or does it leave its facing as it
   is, as the report in flight does?
2. Does the pad service it while it slides (claim 26 skips an aircraft that is still moving), and
   once it has come to rest, wherever that is, is it serviced there?
3. Does its *guard* after the report (claims 28 and 29, an aircraft in contact) change its
   destination or its slide?

## Possible observation

AC31 part h (1) with a touchdown in a cell next to the docking position's. Record each tick from
that touchdown: position, height, body facing, speed fraction, behaviours, destination, the pad's
behaviour and the aircraft's ammunition, until 300 ticks after the report.

## Current workaround

The report is made as the report in flight is (`air::report_entered`): the pad starts servicing
and has it queue *sleep*, it queues *guard*, and its facing is left as it is. Claim 26's service
steps skip it while it moves or is not at rest, so it is serviced once it comes to rest, wherever
that is (claim 24: neither the report nor the pad tests where it touched down). Cited
`design: S-0501 claim 24 and Q-2649`.
