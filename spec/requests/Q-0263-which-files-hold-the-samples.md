---
id: Q-0263
title: Which sample banks and loose sound files does the game play samples from, and which copy wins when a sample name is in several?
status: open
related: [S-0005]
answered-by: []
---

## Need

S-0005 claim 10 says which sound archive is mounted and when. It does not say which files inside
the sound archive (and the other mounted archives) hold the samples that sound and speech entries
name, nor which copy is used when two banks hold a sample of the same name. `glasnost-audio`
builds its `SoundBank` from the player's files and must not hard-code file names it has not been
given.

## Questions

1. Which indexed sample banks (an index file and its data file) does the game read samples from,
   and are they found by name, by archive, or otherwise? Are announcements in the same banks?
2. Can a sample also come from a separate sound file (an AUD file or a WAV file) in an archive or
   loose, and how is it named?
3. When two banks hold a sample with the same name, which one plays?

## Possible observation

A loose test bank (our own synthetic index and data file) holding a sample named like a stock one,
placed in each candidate location, recording which version is heard.

## Current workaround

`glasnost_audio::bank::discover` finds every index file in the mounted archives by its content
and pairs it with the entry of the same archive whose size equals the end of its furthest sample
(as the asset viewer does). Banks are taken in archive mount order, and the first bank holding a
sample name wins. Loose AUD and WAV files are not read unless the caller adds them by name.
Documented in docs/development.md, "Audio (M5b)".
