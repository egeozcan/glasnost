---
id: Q-1341
title: The importer warning for an empty warhead locomotor value
status: answered
related: [S-0607, S-0001]
answered-by: [S-0607]
---

## Need
S-0607 claim 22 (r4) says that an empty value of a seizing warhead's locomotor key is ignored as if the key were absent and that our importer warns about it. S-0001 claim 8 says that a line whose value is empty makes no entry at all, so the rules reader, ours included, never sees an empty value: it cannot be told apart from an absent key, which does not warn. The behaviour (the locomotor in force, else teleporting) is the same either way; only the warning is in question. The gap is in the warhead import in `crates/convert/src/sim_combat.rs`.

## Questions
1. Is the warning for an empty value meant to come from a separate scan of the raw rules text, outside S-0001's reading, or should the empty case be dropped from the warning because the value never reaches the reader?

## Possible observation
None needed: this is a question about our importer's messages, not about the game.

## Current workaround
An empty value is no entry (S-0001 claim 8): it reads as absent and gives no warning. A malformed non-empty value (a bare name, or digits without braces) on a seizing warhead warns as claim 22 says.
