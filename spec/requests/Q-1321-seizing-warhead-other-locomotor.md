---
id: Q-1321
title: A seizing warhead that names another locomotor, or none
status: answered
related: [S-0607]
answered-by: [S-0607]
---

## Need
S-0607 claim 5 links "its jumpjet locomotor". The import mapping keys the seizing warhead's locomotor name. Data can give the seizing flag with another locomotor name, or with none.

## Questions
1. A warhead that seizes but names a locomotor other than jumpjet: does a hit seize at all, deal ordinary damage, or do nothing?
2. Same question for a seizing warhead that names no locomotor.

## Possible observation
Test data: a magnetic-beam weapon whose warhead names the hover locomotor, then none; fire at a light vehicle.

## Current workaround
Only a warhead whose locomotor is jumpjet seizes. Any other seizing hit does nothing, with no damage: the seize takes the place of damage under claim 1.
