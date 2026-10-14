---
id: Q-0325
title: Which announcement data entry each skirmish announcement of S-0701 claim 28 plays, and how its sample file is named
status: answered
related: [S-0701, S-0415, S-0408, S-0411, S-0314, S-0405, S-0414]
answered-by: [S-0701]
---

## Need

S-0701 claim 28 lists the announcements of a skirmish by role and says when each is raised. The
roles are: base under attack, ally under attack, harvester under attack, low funds, low power,
unit ready, construction complete, new construction options, the production-feedback lines,
structure sold and unit sold, unit lost, unit repaired, unit promoted, primary building selected,
rally point set, building captured, tech building lost, structure garrisoned and abandoned,
building online and offline, the superweapon lines, victory, defeat and player defeated.

It does not say which entry of the announcement data each role plays. The only entry name a spec
gives is the cannot-deploy announcement's (S-0415 Import mapping).
`glasnost_audio::names::SoundNames::announcement` needs the entry, or the key the importer reads,
for every role.

Claim 26 also says an entry's sample is its voice set's file name "cut to 8 characters with the
extension added", but it does not name the extension.

## Questions

1. For each role of claim 28, which announcement data entry is announced: a fixed entry name, or a
   key of the rules data that names it? Does any role pick between several entries?
2. Which file extension is added to the 8-character sample name? Is the sample then looked up in
   the same sample banks as the samples of sound entries?
3. Do announcements follow the player's sound volume option, a separate voice option, or neither
   (S-0701 Open questions)?

## Possible observation

Scenarios G1 and G3 of S-0701, with a loose test copy of the announcement data that gives each
entry a distinct test sample.

## Current workaround

- Every role except cannot-deploy resolves to no entry and is silent.
- The raise conditions, the throttles (F7) and the queue of claim 27 all run, so a role starts
  playing as soon as its entry is known.
- The 8-character sample name is looked up as it is, without an extension, in the sample bank.
  The bank's index names carry no extension.
- Announcements are mixed at full level and ignore the volume option.

Documented in docs/development.md, "Audio".
