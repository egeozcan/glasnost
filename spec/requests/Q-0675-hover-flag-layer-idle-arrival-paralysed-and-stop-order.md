---
id: Q-0675
title: Hover actual-position flag layer, arrival with no destination, failed arrival search, paralysed state and the stop order's two destinations
status: open
related: [S-0503, S-0209, S-0212, S-0213, S-0417]
answered-by: []
---

## Need
S-0503 (revision answering Q-0690 to Q-0696) settles the hover's flags, between-steps state,
current speed, cell-arrival updates, initial heading, first choice and stop order. Five points
in it cannot be built without guessing. The code is `hover::step` (the F5 arrival),
`hover::choose_next`, `hover::take` and `hover::stop_order`.

## Questions
1. **Layer of the actual-position flag (claim 13).** The deck-or-ground rule is stated for "a
   marked point" (deck only with a high bridge in the cell and the point at least 416 leptons
   above the surface there). When the flag "at the vehicle's actual position" is released at the
   first advance, or restored after an arrival, does the same rule apply to the actual position,
   or is it the layer the vehicle is listed in?
2. **Arrival with no destination (claims 7, 13 and 15).** When the vehicle reaches its heading
   point and has no destination at all (for example after a stop order cleared it while it was
   travelling), is a choice "attempted"? Is the flag at the actual position restored, and is there
   a cell-arrival update?
3. **A failed arrival-time search (claims 7, 13 and 15).** On an F5 arrival with no stored step
   left, the fresh search fails (or is not made because the retry delay has not run out) and no
   heading point results. Does this count as a choice that "succeeds without supplying a heading
   point" (flag restored, cell-arrival update), or as an unsuccessful one (no flag restored, no
   update)? If the latter, which cell, if any, carries the vehicle's flag while it waits?
4. **Paralysed (claim 5).** "A paralysed vehicle merely remembers the destination without this
   immediate choice." Which states make a hovering vehicle paralysed: disabled by an EMP effect,
   switched off by a lost control centre (it still counts as powered until it settles), held by a
   parasite, lifted by a magnetic beam, or another? Is "paralysed" ever true of a vehicle that is
   powered?
5. **The moving-now path update (claim 5).** "If moving now but without a heading point, it sets
   throttle and ρ to 1 and attempts the path update immediately." Is this path update a fresh
   search made at once, regardless of the retry delay *R*, and does it start *R*? Or is it only the
   destination test that decides whether to choose there?
6. **A person's stop order on one destination field (claim 16).** The order-level destination is
   cleared while the locomotor's destination *D* and stored path are kept when *D* equals the
   heading point. Other rules (behaviours, S-0417 claim 7's arrival test, the S-0212 searches)
   read the destination. While the order-level destination is cleared but *D* is kept, which of
   them see a destination? In particular, does the claim 7 completion at *P* (destination cleared,
   cell-arrival update when the contacted object is in the present cell) still happen?

## Possible observation
On a tiny flat map with an invented hovering vehicle type:
- give a stop order while it travels to its heading point and log the cell flags and any factory
  clear report at its arrival;
- block the only route so the arrival-time search fails, and log which cell carries a flag;
- EMP-disable it, give a move, and log whether a heading point appears at once;
- order it onto and off a high bridge and log the flag's layer at rest on the deck and below.

## Current workaround
- Q1: `hover::mark_layer` applies the marked-point rule to the actual position as well.
- Q2: `hover::choose_next` returns `Idle` with no destination: no choice, no flag restored, no
  cell-arrival update. Travel continues as for any choice.
- Q3: a failed or deferred arrival-time search that leaves no heading point counts as a successful
  choice without one: the flag at the actual position is restored and a cell-arrival update follows.
- Q4: no state counts as paralysed; an unpowered vehicle makes no immediate choice.
- Q5: no separate path update; `choose_next` runs (with its own *R*-gated search) only when the
  destination is gone or in the present cell.
- Q6: the single destination field is cleared by the order, so the arrival at *P* is Q2's case;
  the heading point, throttle, hover speed and ρ are kept.
