---
id: Q-0261
title: Which speech database entry is announced for each event the specs mention, and how does an entry become a sample?
status: open
related: [S-0005, S-0311, S-0314, S-0408, S-0411, S-0414, S-0415, S-0011]
answered-by: []
---

## Need

S-0415 imports one announcement by its speech database entry name (`EVA_CannotDeployHere`) and
leaves "which sound it plays" to the sound specs. Other specs mention announcements without an
Import mapping: the construction-complete announcement of a done building (S-0411 claim 9), the
unit-sold announcement (S-0408 claim 11), the repair-complete and low-funds announcements
(S-0314 claim 6), the promotion announcement (S-0311), the defeat, alliance and alliance-break
announcements (S-0414 claims 5, 12 to 14) and the play-speech trigger action (S-0011 claim 32).
`glasnost-audio` needs, for each, the entry name and how that entry becomes the sample that plays.

## Questions

1. For each event above, which speech database entry is announced (the key or fixed name the
   importer reads)? Does the local player's side or country choose between variants of one
   announcement, and how?
2. How does a speech database entry name its sample, and what other settings does it carry
   (volume, priority, delay or repeat suppression)?
3. Do announcements queue behind one another, interrupt one another, or play together? Is there a
   repeat suppression shared by all announcements (S-0314 claim 6 has one for repair complete)?
4. What does an unknown entry name do?

## Possible observation

A loose test copy of the speech database giving each entry a distinct test sample, then setting
off each event once in a skirmish and recording what is heard.

## Current workaround

An announcement's entry name resolves to the sample of the same name, through the same interim
catalog as sounds (Q-0260), and announcements play together like any other sound. Only
`EVA_CannotDeployHere` has an entry name; every other announcement is silent
(`names::Announcement` resolves to nothing). Documented in docs/development.md, "Audio (M5b)".
