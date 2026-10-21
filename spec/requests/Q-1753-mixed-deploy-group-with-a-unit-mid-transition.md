---
id: Q-1753
title: How a deploy press treats an in-place deployer that is still deploying or undeploying
status: open
related: [S-0213, S-0214, S-0517]
answered-by: []
---

## Need
S-0213's deploy rule says that when a selection holds both in-place deployers that are deployed and in-place deployers that are not, the deployed ones get no order. It does not say which side a unit falls on while it is still deploying or undeploying. The engine needs that to fix the group-deploy press (crates/deterministic/sim/src/targeting.rs `key_command`), which the project lead found mixing deployed and undeployed members in play (M8 step A4).

## Questions
1. A selection holds one deployed in-place deployer and one that is part-way through deploying (its deploy animation has started but not finished). A person presses deploy. Does the deployed one undeploy, or does it get no order? What happens to the one still deploying?
2. The same with one unit part-way through undeploying and one fully undeployed. Does the fully undeployed one deploy? Does the undeploying one stop, carry on, or deploy again?
3. If every in-place deployer in the selection is mid-transition, does the press reverse any of them?
4. Is the answer the same for infantry and vehicle in-place deployers?

## Possible observation
Two in-place deployers of one type side by side. Deploy one, select both while it is still animating, then press deploy. Record each unit's deployed state every tick for 100 ticks. Repeat for the undeploying case, with an infantry and a vehicle type.

## Current workaround
Until this is answered, the fix applies the mixed-group exception only to units that are fully deployed or fully undeployed. A mid-transition member is handled by the deploy check and deploy readiness of S-0214 alone. No other rule is invented.
