---
id: Q-0640
title: Does a naval type with no weight setting sink when killed on water
status: answered
related: [S-0312, S-0504]
answered-by: [S-0504]
---

## Need
S-0312 claim 10 makes a killed naval vehicle sink when its type's weight is at least the
game-wide sinking weight. No spec gives the weight a type has when its rules section does
not set it (S-0312's Import mapping and S-0504's do not list an absent default for the
type's weight). The engine has to decide, at a killing hit on water, whether such a type
sinks or explodes. `naval::sinks_at_kill` in `crates/deterministic/sim/src/naval.rs` is
where this is decided.

## Questions
1. A surface naval vehicle type (not underwater, not organic) whose rules section has no
   weight setting is killed on open water with the game-wide sinking weight left as it is.
   Does it sink (stay in play and go down) or explode and leave play at once?
2. Does the answer change when the game-wide sinking weight is set to 0, or to a very
   small positive number?
3. If such a type behaves as if it had some weight, which weight is it (for example, is it
   the same as a type that sets the weight to 1)?

## Possible observation
Copy a stock surface vessel type, remove its weight line, and kill it on open water with
the game-wide sinking weight unchanged, then with it set to 0 and to 0.5. Record whether
it sinks or explodes in each run.

## Current workaround
Interim rule: a type without a weight setting never sinks at a killing hit (it explodes and
leaves play like any other destroyed vehicle). The test `claim_10_sinking_condition` in
`tests/claims_s0504.rs` pins this. A cull (S-0614 claim 11) still starts sinking whatever
the weight.
