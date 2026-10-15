---
id: Q-0752
title: A paradrop with no parachute animation type
status: answered
related: [S-0522, S-0511, S-0109]
answered-by: [S-0522]
---

## Need

S-0522 Open question 2 leaves open what happens when the rules name no parachute animation type, or name one that does not exist. The engine must decide two things:

- whether the drop still succeeds;
- which fall-rate recurrence of S-0511 F3 then applies.

The code is `release` in `crates/deterministic/sim/src/paradrop.rs`, together with `SwRules::parachute`.

## Questions

1. With `Parachute` absent from `[General]`, does a drop attempt still release the paratrooper?
2. If it does, does the paratrooper fall at the parachute rate or at the no-parachute rate?
3. Is the answer the same when `Parachute` names an animation type that is not listed?

## Possible observation

Run S-0522 scenario P8 with `Parachute` removed. Run it again with `Parachute` set to an unlisted name. Record the paratrooper's height on each update.

## Current workaround

- The drop succeeds.
- No animation is attached, so the paratrooper falls with the no-parachute recurrence.
- No animation creation draws are taken.

These choices are marked in the code with this request's ID.
