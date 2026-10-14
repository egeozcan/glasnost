---
id: Q-0336
title: Do the guard, stop and scatter command sounds play when the command orders nothing
status: answered
related: [S-0701, S-0213]
answered-by: [S-0701]
---

## Need

S-0701 claim 19 says that the guard, stop and scatter key commands play their game-wide interface
sounds as screen sounds, instead of a response. It does not say whether a sound plays when the
command gives no orders. That happens when nothing is selected, or when no selected object takes
the command (S-0213 table T2). The client hands commands to `Audio::key` in
`glasnost_audio::events` and has to choose which ones to pass.

## Questions

1. With nothing selected, does pressing the guard, stop or scatter key play its sound?
2. With only objects selected that the command does not apply to, such as a building for
   scatter, does the sound play?
3. With several objects selected, does the sound play once per key press, or once per object
   ordered?

## Possible observation

Follow S-0701 scenario D4 three times: with an empty selection, with only a building selected,
and with three units selected. Count the interface sounds heard each time.

## Current workaround

The client passes a key command to the audio only when the command issued at least one order, and
the sound plays once per key press. Documented in docs/development.md, "Audio (M5b)".
