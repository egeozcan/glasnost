---
id: Q-1200
title: May a mind-controlled construction vehicle deploy into a construction yard
status: open
related: [S-0601, S-0214, S-0415]
answered-by: []
---

## Need
S-0214 Terms says a vehicle is not deploy-ready to deploy into a
construction yard while it is mind-controlled. S-0601 claim 15 names "a
construction vehicle into a construction yard" as an example of a
controlled vehicle that deploys, and its scenario MC7 deploys a controlled
construction vehicle. The engine has to pick one rule for
`econ::deploy::deploy_ready`.

## Questions
1. Can a player who has taken an enemy construction vehicle by mind control
   order it to deploy, and does a construction yard appear?
2. If it cannot, does the deploy order show a refusal (cursor or sound), or
   is it silently ignored?
3. Does the answer differ for a slave miner (deploys into its refinery) or
   another vehicle that deploys into a non-yard building?

## Possible observation
Take an enemy construction vehicle with a mind-control unit, select it and
give the deploy order on open ground; repeat with a taken slave miner.

## Current workaround
S-0214's refusal is kept for a construction yard: a controlled vehicle that
deploys into a construction yard is not deploy-ready. Claim 15 is
implemented and tested with a vehicle that deploys into an ordinary
building (`claims_s0601_deploy.rs`).
