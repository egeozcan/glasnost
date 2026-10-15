---
id: Q-0702
title: Passengers of a transport that leaves play without a death rule for them
status: open
related: [S-0507, S-0312, S-0408]
answered-by: []
---

## Need

S-0312 claims 5, 12 and 16 give the fate of passengers when their transport is destroyed. No spec
gives it when the transport leaves play another way: sold at a service depot or removed by a
script, a sinking vessel that leaves play, or an aircraft destroyed on the ground (claim 16: it
"leaves play at once"; claim 12 covers only vehicles). Claim 12's second row says the passengers
of a crashable vehicle on the ground "die when it leaves play" but, unlike rows 1 and 3, does not
say whether those deaths are credited to the attacker.

The code in `crates/deterministic/sim/src/transport.rs` (`removing`, `remove_passengers`) and
`crates/deterministic/sim/src/combat/death.rs` (`leave_play`) needs the rule.

## Questions

1. What happens to the passengers of a transport removed without a death (a sale, a script
   removal)?
2. What happens to the passengers of an aircraft destroyed on the ground, and of a sinking vessel
   when it leaves play?
3. Are the deaths of claim 12's second row credited to the attacker of the killing hit?

## Possible observation

S-0312 scenario D9 variants: destroy a loaded aircraft on the ground, sink a loaded vessel, and
sell a loaded vehicle; watch the passengers and the kill counts.

## Current workaround

The passengers of a transport that leaves play by a path without a rule for them are taken out of
play with it, without a death and without credit. Claim 12's second-row deaths are credited to no
one. The choice is marked in the code with this request's ID.
