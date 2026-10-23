---
id: Q-1620
title: Does a depot's processing after a send-away repeat the contact-loss slot changes
status: answered
related: [S-0108, S-0314]
answered-by: [S-0108]
---

## Need
S-0108 claim 20 has the row "Repair depot in the working phase: a step answers *done*, *not
needed* or *cannot pay*, or the depot is no longer in contact → *production* and *special-2*
stop; *special-3* and *active-1* start". It also says that when the depot sends the vehicle away
the slot changes happen on the tick of that step, before the contact is released, and that "when
contact ends without a step ... the depot reacts at its next processing".

After a *done* step that sends the vehicle away, the depot is still in its repair behaviour and
(S-0314 claim 22) its next repair processing finds no contact. Read literally, the working row's
"no longer in contact" event could fire again at that processing, so *special-3* would be replaced
(a new animation continuing from the running one's image frame, with its passes counted afresh)
and *active-1* started again. The engine needs to know whether that second change happens,
because *special-3*'s lifetime decides when a settling depot with no contact returns to *guard*
(S-0314 claim 22). The code is `glasnost_sim::depot::process_repair` with
`glasnost_sim::depot_anim::service_ended`.

## Questions
1. After a working step answers *done* and the vehicle is sent away, does the depot's next repair
   processing without contact start *special-3* and *active-1* again (replacing the running
   *special-3*), or does it make no slot change?
2. Is the answer the same after a working *not needed* step that sends the vehicle away?
3. When the departure fails (no destination, contact kept) and a later settling step answers
   *not needed* and sends the vehicle away, does the following processing without contact make
   any slot change (with or without *production* or *special-2* running)?

## Possible observation
B18 part a or c with a *special-3* type of several passes (a positive loop count) and slot starts
and ends logged each tick: if *special-3* is restarted at the processing after the send-away, its
pass count starts again from that tick and it ends later than one lifetime after the *done* step.

## Current workaround
Interim rule: the slot changes come once, at the step that sends the vehicle away; the next
processing without contact makes no second change (`DepotState::sent_away`, cleared at any
processing with a contact). A contact that ends without a step still gets the row at the next
processing, as the claim says.
