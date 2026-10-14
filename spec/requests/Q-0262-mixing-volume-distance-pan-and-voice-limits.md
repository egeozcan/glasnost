---
id: Q-0262
title: How loud does a sound play at a distance from the view, how is it panned, and how many sounds can play at once?
status: answered
related: [S-0416, S-0202]
answered-by: [S-0701]
---

## Need

`glasnost-audio`'s mixer has to decide the gain of every playing sound, its stereo position, and
what happens when too many sounds want to play. S-0416 claim 4 says a credit tick plays "at half
volume", which needs a volume scale to mean anything. No spec covers the rest.

## Questions

1. How does a sound played at a map position get quieter with its distance from the view (the
   centre of the tactical view, or the visible area)? Is there a range beyond which it is not
   played at all, and is that range per sound or game-wide? Is distance measured on the map
   (leptons, cells) or on the screen?
2. Is a positional sound panned left or right by its position on the screen? By how much at the
   screen edge?
3. Is "half volume" (S-0416 claim 4) half the amplitude, or half on a logarithmic (decibel) scale?
   How do the game's sound and voice volume settings combine with an entry's own volume?
4. How many sounds can play at once? When a new sound would exceed that, is it refused, or does it
   replace a playing one, and which (lowest priority, oldest, quietest)? Are voices
   (acknowledgements, announcements) counted separately from effects?
5. Is a sound that starts outside the audible range started silently (and heard if the view moves
   to it), or never started? Does a playing sound follow the view as it scrolls?

## Possible observation

Audio captures of one looping test sound at measured distances from the view centre and at the
left and right screen edges; then a burst of many simultaneous short test sounds, counting how
many are heard.

## Current workaround

Our own interim parameters, marked as such in `mixer::MixParams` and docs/development.md
("Audio (M5b)"): 16 voices; full volume within 10 cells of the view centre and silence beyond
30 cells, linear in between, measured on the map; pan by the screen-x offset (x − y in leptons),
fully to one side at 20 cells; gain and pan follow the view while a sound plays; a sound that
starts out of range is not started; a new sound that does not fit replaces the playing voice of
lowest priority (oldest first among equals) when its own priority is at least as high, and is
dropped otherwise; volumes are linear amplitude, so "half volume" is amplitude × 1/2.
