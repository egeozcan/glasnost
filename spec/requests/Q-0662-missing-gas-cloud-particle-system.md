---
id: Q-0662
title: A rules set without the fixed gas-cloud particle system (S-0613 against S-0004)
status: answered
related: [S-0613, S-0004]
answered-by: [S-0613]
---

## Need
S-0004 claim 3 and its Import mapping say that when no particle system of the engine-fixed
gas-cloud name exists once all rules layers are processed, one is created at game start and reads
no rules section. S-0613 claim 1 and its Edge cases say a rules set without that type cannot be
played: the importer rejects it. The importer and the type registration order need one rule.

## Questions
1. When the gas-cloud particle system type is missing from every rules layer, does the game
   start with a created type that read no rules section (S-0004), or is the rules set unplayable
   (S-0613)?
2. If a type is created, what are its observable values (lifetime, behaviour kind, particle
   type), given that it read no section?
3. If S-0613's rejection is our own design departure rather than the original's behaviour, should
   S-0004 say so?

## Current workaround
The importer follows S-0004 claim 3: when no layer names the gas-cloud particle system, it
creates the type at game start, reading no section (`convert::layers`, `finish`). Gas clouds
themselves are not built, so nothing uses that type yet.
