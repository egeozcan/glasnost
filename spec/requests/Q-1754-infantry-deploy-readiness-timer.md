---
id: Q-1754
title: The timer after an infantry deploy that S-0214's deploy readiness names
status: answered
related: [S-0214, S-0213, S-0215]
answered-by: [S-0214]
---

## Need
S-0214 Terms, *deploy readiness*: "An infantry unit has it when its type deploys in place, and,
if its type has an undeploy delay, when it is neither deployed nor undeploying nor within the
timer that follows a deploy." No approved spec says what that timer is, when it starts, how long
it runs, or whether it is the same thing as the delay S-0215 claim 30 gives after an undeployed
unit starts its deploy (the undeploy delay, in ticks). The engine needs it for the deploy key of
S-0213 table T2 and its deploy rule (`crates/deterministic/sim/src/targeting.rs`,
`deploy_readiness`), fixed in M8 step A4.

## Questions
1. For an infantry type that deploys in place and has an undeploy delay of 0 or more, which timer
   is meant? Does it start when the deploy press is applied, when the deploy sequence starts, or
   when it ends?
2. How long does it run: the undeploy delay, the deploy sequence's frame count, or something else?
3. While it runs, does a deploy press give that unit no order at all (so it neither deploys again
   nor undeploys), even after the deploy sequence has finished?
4. Does anything else end it early (a move order, a stop order, the unit being hit)?

## Possible observation
An infantry test type that deploys in place with an undeploy delay of 60 ticks and a deploy
sequence of clearly fewer ticks. Select it, press deploy, and press deploy again every tick for
120 ticks, recording each tick whether an order is given and the unit's sequence. Repeat with an
undeploy delay of 0.

## Current workaround
The timer is not modelled: readiness for such a unit is decided by its deployed and undeploying
states alone (deployed means the deployed, deployed-firing and deployed-idle sequences;
undeploying the undeploy sequence). No timer is invented.
