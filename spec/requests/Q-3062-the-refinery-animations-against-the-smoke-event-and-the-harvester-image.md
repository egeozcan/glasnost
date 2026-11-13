---
id: Q-3062
title: In claim 11's payment and empty-load rows, do the refinery's animations start before or after the smoke event, and does an unloading harvester change its own image?
status: answered
related: [S-0402, S-0108, S-0109, S-0616]
answered-by: [S-0402]
---

## Need

S-0402 claim 11 gives the rows of *unload*. A payment row plays the building's payment animation
"unless that is already playing" and pays the harvester. The empty-load row stops the payment
animation and, for a refinery-flag building, plays the unloading animation. Under **Refinery
smoke**, each of these rows also "produces one smoke event on the building in the neighbouring
cell, as defined by S-0616 claim 1", and "by the time payment is applied at that processing, the
smoke event has occurred".

So the smoke event comes before the payment, but no text orders the smoke event against the
animation's start in the same row. Both create objects in that processing: the slot animation is
an animation object of the update order with its creation draws (S-0108 Terms, S-0109 claims 3,
28 and 29), and a smoke event creates up to four particle systems (S-0616 claim 1). Their order
decides the order in which the new objects enter play and, for an animation with a random rate,
where its creation draws fall.

GitHub issue #430 asks for the three refinery animations. It also notes that the report says the
harvester itself shows no motion while it unloads. S-0402 claim 5a's harvesting sprite is for
loading only ("returning, docking and unloading do not themselves show this sprite"), and no spec
gives an unloading harvester an image of its own.

## Questions

1. In a payment row, does the payment animation start (when it is not already playing) before
   or after the smoke event's particle systems are created?
2. In the empty-load row, in what order do the three steps come: the stop of the payment
   animation, the start of the unloading animation, and the smoke event?
3. Does a harvester show any image, sprite sequence or facing change of its own while it unloads
   (from the start row of *unload* to the end of *finishing*), other than the turn to map east
   that claim 11 already gives? If so, what starts and ends it?

## Possible observation

A refinery type whose payment and unloading animations have a random rate and whose refinery smoke
system is set, with the recorder logging per tick the objects created (kind, type, number in the
update order) and the random-number draws taken at the processing of each payment row and of the
empty-load row, answers questions 1 and 2. A recording of the harvester's drawn image frame per
tick from docking to departure, beside one of the same harvester standing still elsewhere,
answers question 3.

## Current workaround

The engine starts the animation first, in the order the row's text gives, then makes the smoke
event, then pays: in the empty-load row, the payment animation stops, the unloading animation
starts, then the smoke event. A harvester shows no image of its own while it unloads.
