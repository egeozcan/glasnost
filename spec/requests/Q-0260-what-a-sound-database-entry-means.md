---
id: Q-0260
title: How does a sound name from the rules data become the samples that play, and what do the sound database's entries control?
status: answered
related: [S-0005, S-0001, S-0107, S-0214, S-0408, S-0415, S-0416, S-0410, S-0210, S-0109, S-0207]
answered-by: [S-0701]
---

## Need

Many specs name sounds by role and import them as sound names (`SellSound`, `DeploySound`,
`PackupSound`, `CrushSound`, `CreditTicks`, `RadarOn`, `RadarOff`, `VoiceSelect`, `VoiceComment`,
`EnterWaterSound`, `LeaveWaterSound`, `ChronoInSound`, `ChronoOutSound`, an animation's `Report`,
the cue pairs of S-0107). S-0005 says which copy of the sound database is used, and S-0001 says
how it is read, but no spec says what its entries mean. S-0411 claim 12 points to a planned
S-0701 for this. `glasnost-audio` needs it to turn a sound name into the samples it plays
(`catalog::SoundCatalog`) and to tell whether a sound name is known (S-0107 Import mapping:
an unknown sound name leaves the field as it was).

## Questions

1. Which entry does a sound name select (case rules, length limits, what an unknown name does)?
2. How does an entry name the sample files it may play, and how is one chosen when it lists several
   (in order, at random from the unsynchronised stream of S-0202 claim 11, with or without
   repeats)? How many draws does one play take?
3. What does an entry control besides its samples: volume, a random volume or pitch range, a
   priority, a limit on how many copies play at once, a minimum gap between plays, looping,
   whether it plays at a position or everywhere, whether only the owning player hears it, and the
   range over which it can be heard? What are the defaults for each when the entry does not give
   it, and what do the game-wide settings of the database (if any) do?
4. What happens when the samples an entry lists are missing from the sound archives?

## Possible observation

A loose test copy of the sound database (in the style of S-0005 Scenario M2 c) giving one test
entry each setting in turn, triggered by a selection response, recorded by ear and with an audio
capture.

## Current workaround

`glasnost-audio` resolves a sound name to the single sample of the same name (ASCII case
folded), at full volume, with the default priority, no per-sound limit and no randomisation
(`catalog::SoundCatalog::interim`). Interim rule documented in docs/development.md, "Audio (M5b)".
