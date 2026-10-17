---
id: Q-1267
title: Whether an entrant's contact with an absorbing building ends when its enter ends otherwise
status: answered
related: [S-0605, S-0215, S-0213]
answered-by: [S-0605]
---

## Need
S-0605 names the ways an entrant gives up its contact with an absorbing building: a refused
loading request (claims 3 and 5), a refusal after waiting in line (claim 4), going in (claim 7),
and leaving the map (Edge cases). Claim 5 leaves "other interruptions" to other specs (a new
order, S-0213 table T3; destruction, sale and owner change), and S-0215 Terms says contact lasts
"until the exchange ends or one side breaks it off", leaving when to the exchange's spec.

In an AI-versus-AI game of ours an armed infantry entrant with contact kept a target from
before; the approach step of S-0215 claim 37 replaced its destination (the building) by an
approach cell, it walked there in *enter*, and on arrival it fell back to rest. No rule ended
its contact, so it held the building's only link place for the rest of the game, and every
later entrant waited in line for good (claim 3: "If the building already holds any contact, the
actor starts waiting in line"). A new behaviour order is not the problem: S-0213 table T3 (b)
breaks off any contact. Code:
`crates/deterministic/sim/src/structures/absorb.rs`, `contact_outlived`.

## Questions
1. Does an entrant's fall-back to rest (S-0215 claim 13), from any cause, end its contact with an
   absorbing building?
2. If not: can an actor that is no longer in *enter* keep a link place of an absorbing
   building indefinitely, so that every later entrant waits in line for good?
3. Is the approach step of S-0215 claim 37 run for an entrant in *enter* with a target, and may it
   replace the building as destination?

## Possible observation
Scenario A2 with one link place: while the first entrant walks with contact and a second waits
in line, let the first fall back to rest without an order (for example an armed entrant that
keeps an earlier target out of range). Record each tick the first one's
contact, behaviour and destination, and whether the second gets contact and goes in.

## Current workaround
In its turn after its movement (the place of claim 4's retry), an actor in contact with an
absorbing building that neither has *enter* as its current behaviour nor has *enter* queued
gives the contact up. The window in which a waiting actor has been granted contact and has
*enter* queued but not started keeps the contact.
