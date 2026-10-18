---
id: Q-1164
title: S-0606 claim 12 (revision 5) against S-0415 claim 1 (revision 18), whether the deploy-fire rule applies to a bunkered vehicle's unload
status: open
related: [S-0606, S-0415, S-0517]
answered-by: []
---

## Need

Both specs changed in the same round, and they now give opposite answers for a bunkered vehicle
of a type with the deploy-fire flag.

S-0606 revision 5, claim 12 (*Delay*), lists four rules that a bunkered vehicle's *unload* goes on
with after the release, the first one that applies taking effect: "the deploy-fire rule (S-0517
claim 2, with its own delay), the harvester or weeder rule (S-0402), the deploy-into rule (S-0415)
or the simple-deployer rule (S-0517)". A type with "the deploy-fire flag without the
simple-deployer flag" therefore follows the deploy-fire rule, and S-0606 says that "S-0415 claim 1
gives the order of the first three rules".

S-0415 revision 18, claim 1, says the opposite: "Passenger capacity and the deploy-fire flag keep a
vehicle that is not bunkered from deploying. For a bunkered vehicle they do not: one that deploys
into a building and has no harvester or weeder flag makes its deploy attempt whatever its passenger
capacity or deploy-fire flag." It lists three properties (harvester or weeder, deploy-into,
simple deployer), and ends: "A bunkered vehicle with none of the three properties, a deploy-fire
vehicle included, does nothing further, with no draw, and chooses a delay of exactly 450 ticks.
The deploy-fire rule above does not apply to a bunkered vehicle."

The two differ for (a) a bunkered deploy-fire type without the deploy-into, harvester, weeder or
simple-deployer properties (S-0606: the deploy-fire rule fires and chooses its own delay; S-0415:
nothing more, 450 ticks), and (b) a bunkered deploy-fire type that also deploys into a building
(S-0606: the deploy-fire rule; S-0415: the deploy attempt).

## Questions

1. After the release of claim 12, does a bunkered vehicle of a deploy-fire type (without the
   simple-deployer flag) follow S-0517 claim 2's deploy-fire rule in the same processing?
2. If so, does the deploy-fire rule also come before the deploy attempt for a bunkered type that
   both has the deploy-fire flag and deploys into a building?

## Possible observation

S-0606 scenario K6 with a deploy-fire vehicle type, if K6 reaches *unload*: whether the vehicle
fires its deploy weapon in the release tick, and the delay until its next processing. A variant
whose type also deploys into a building: whether it fires or makes a deploy attempt.

## Current workaround

Interim rule, cited as "Q-1164 interim" in `docs/m7-claims/step1k-m7a.md` and
`docs/development.md` (Simulation, M7a: Vehicle bunker). S-0415 claim 1 decides, as built: the
deploy-fire rule does not apply to a bunkered vehicle. After the release the vehicle goes on with
the harvester or weeder rule, then the deploy attempt, then the simple-deployer rule
(`structures::bunker::continue_unload`); a deploy-fire type with none of these does nothing more
and chooses 450 ticks. Test: `claim_12_bunkered_deploy_fire_interim`.
