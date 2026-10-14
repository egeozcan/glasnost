---
id: Q-0329
title: Delayed-mode sample sequence of a sound instance (attack, decay, waits and draws)
status: answered
related: [S-0701]
answered-by: [S-0701]
---

## Need

S-0701 claim 14 specifies immediate mode fully. Delayed mode applies when the delay's first number
is 33 ms or more. For it, the claim gives only the body rules. It says the attack and decay
samples, the waits between samples and the draw counts are not established, and must not be
inferred from immediate mode. `glasnost_audio::mixer` still has to play something for such
entries.

## Questions

1. In delayed mode, which waits separate the samples? Is it a fresh R(*a*, *b*) per sample, the
   delay's numbers, or no wait? Is there a wait before the first sample and after the last?
2. Are attack and decay samples played in delayed mode, and with which draws?
3. In which order are the draws of a delayed instance taken?

## Current workaround

A delayed instance:

- plays the same attack, body and decay partition as immediate mode;
- follows the delayed body rules of claim 14: a random body sample is chosen once and kept across
  loops, and with *all* the body samples play without repetition within a pass;
- takes no startup ordering draw;
- has no wait between samples.

Documented in docs/development.md, "Audio".
