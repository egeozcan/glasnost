---
id: Q-0330
title: Defaults of the game-wide sound keys and the speak delay when the rules leave them out
status: answered
related: [S-0701]
answered-by: [S-0701]
---

## Need

S-0701's Open questions say that the defaults for absent keys are not traced, and must not be
guessed. This covers the game-wide sound references of the rules and the speak-delay field. The
low-funds announcement of claim 28 repeats every N(⌊*k* × 900⌋) ticks, where *k* is the speak
delay. When the key is absent, our engine has no *k*.

## Questions

1. When `[AudioVisual]` leaves out one of the sound keys that S-0701 lists, which sound plays:
   none, or a built-in entry name?
2. When `SpeakDelay` is absent, what is *k*?

## Current workaround

An absent game-wide sound key plays nothing. When `SpeakDelay` is absent, the low-funds
announcement is raised once and never repeated. Documented in docs/development.md, "Audio".
