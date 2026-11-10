---
id: Q-2099
title: How "round" settles an exact half in the arena records' group counts
status: answered
related: [M-0001, M-0002, M-0003, M-0017]
answered-by: [S-0216]
---

## Need
M-0001 to M-0003 Setup give each side "a budget of 9000 credits: round(9000 / the type's cost)
units, at least 1 (an infantry group at most 60)", and leave the counts to be computed from the
imported rules. "Round" does not say what an exact half gives. Our oracle arena
(`crates/tools/src/oracle.rs`, `round_half_even`) takes the even one of the two counts. Until the
records' revision of 2026-10-29 that choice rested on their CSVs: the army shares left were whole
multiples of 1/n only for the even count. The revised CSVs have no shares (M-0001 Procedure step
3), so the restated records no longer support it, and nothing else in `measurements/` or `spec/`
says how the half goes.

Whether any group's budget is an exact half of two counts follows from stock costs and is not
stated here.

## Questions
1. When 9000 / cost is exactly a half (n + 1/2), does the stock harness give the group n or
   n + 1 units, or the even one of the two, as the earlier CSVs showed?
2. Is the rule the same for infantry groups below the cap of 60 and for vehicle groups?

## Possible observation
For each pairing whose type's budget is an exact half, count the units of each side at tick 0 in
one game, or state the harness's rounding rule.

## Current workaround
The arena keeps the even count (`oracle::round_half_even`), the choice the records' earlier CSVs
supported. If the answer differs, the group counts change for those types only, and the arena's
games and verdicts are re-run.
