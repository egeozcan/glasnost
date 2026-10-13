---
id: Q-0046
title: Animation creation draws at map load (start events, the overlay cell-animation key)
status: answered
related: [S-0009, S-0010, S-0109, S-0202]
answered-by: [S-0009, S-0010, S-0109, S-0202, S-0401]
---

## Need
S-0202 claim 10 (*Animation creation draws*) and its table "Draws during
map loading" put the creation draws of tile animations (S-0010 claim 19)
and of overlay cell animations (S-0009 claim 12) into the map's load. Two
inputs of those draws are not stated, so `glasnost_sim::load` cannot take
them in full.

## Questions
1. **The overlay type's cell-animation key.** S-0009's Import mapping names
   only "rules overlay type: its cell-animation key", and no approved spec
   names the key. Which rules key of an overlay type gives its cell
   animation, with which value form and which absent-key default? Does the
   importer's "has artwork or a cell animation" test of claim 12 read the
   same key?
2. **Draws of the start event at creation.** An animation created with no
   loop delay has its start event at once (S-0109 claim 13): the type's
   resource chain reaction, and the middle event when μ = 0 (S-0109
   claim 14: particles, scorch marks and craters). S-0202 claim 10 says the
   specs of those effects own their draws, and none is approved. For an
   animation started at map load:
   a. Which draws does the resource chain reaction of the start event take
      (for example when the cell holds no resource)?
   b. Which draws do the middle event's particles, scorch marks and craters
      take, and under which conditions (the height condition of claim 14
      included)?
   c. Is μ taken from the sprite as claim 25 names it at map load, when the
      animation's sprite is that of the map's theater?

## Possible observation
S-0202's scenario R8 (part F) with tile animations whose types have a
chain reaction, particles, a scorch mark or a crater, and an overlay type
with a cell animation, read with the random-state recorder.

## Current workaround
No overlay type has a cell animation in our data, so cell animations take
no draw. A tile animation takes the stage-delay draw of S-0109 claim 3 and
the draws of a bouncing type (S-0202 claim 10); a type whose start event
may take draws (it has a chain reaction, or μ = 0 and a type of middle-event
effect) takes none for its start event, and the world's notes name this
request.
