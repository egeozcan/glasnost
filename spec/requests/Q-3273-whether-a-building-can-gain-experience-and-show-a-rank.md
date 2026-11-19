---
id: Q-3273
title: Whether a building can gain experience and show a rank
status: answered
related: [S-0311, S-0520, S-0111, S-0415]
answered-by: [S-0311]
---

## Need

An issue report (#452) says that in a native game on an imported install, an armed defensive building gained a rank after kills and showed the veteran insignia above its health bar. The lead says that in the original a building, including that one, never ranks up.

The specs as written require what was seen, and the engine follows them:

- S-0311 claim 1: every infantry unit, vehicle, aircraft **and building** has an experience value and a rank.
- S-0311 claim 8: "a trainable credited object gains the experience itself". Claims 8 and 15 and the garrison text also name "trainable-building self-credit" and "the earlier trainable-building self-recipient rule".
- S-0311 Import mapping: `Trainable` defaults to yes when absent, with no exception for building types. S-0520's Import mapping repeats this for building types (claim 22).
- S-0111 claim 18: every unit **and building** whose veterancy value is outside the rookie band shows an insignia.

A local check of the install's data, kept out of the repository, found no rules entry that makes the reported building's type untrainable. So an armed building that kills earns experience under claims 8 and 10 and shows the insignia under S-0111 claim 18.

What the code does now:

- `combat::veterancy::destruction_credited` (crates/deterministic/sim/src/combat/veterancy.rs) gives the credited object the experience of F3 when its type is trainable, whatever its kind, buildings included (claim 8, case "a trainable credited object gains it itself"). Only an untrainable credited object goes on to the missile-launcher and garrison-occupant rules.
- `combat::data` (crates/deterministic/sim/src/combat/data.rs) reads the trainable flag as `c.trainable.unwrap_or(true)` for every kind (S-0311 Import mapping).
- `objects::veteran_at_creation` (crates/deterministic/sim/src/objects.rs) makes a building a veteran at creation when its house has the infiltrated-vehicle-factory flag and its type is trainable, not naval and undeploys into a vehicle (S-0311 claim 11).
- `econ::deploy` (crates/deterministic/sim/src/econ/deploy.rs) copies the vehicle's experience to the building on deploying (S-0415 claim 6) and the building's experience to the vehicle on packing up (S-0415 claim 10; S-0311 claim 13).
- `crates::veteran` (crates/deterministic/sim/src/crates.rs) raises every trainable object in the crate area, buildings included (S-0520 claim 22).
- The client's `mapview::combat::insignia` (crates/app/src/mapview/combat.rs), called from `mapview/live.rs`, draws the insignia for any object whose experience is outside the rookie band, buildings included (S-0111 claim 18).

The engine changes nothing until this is answered. The places to change afterwards are the recipient rule in `combat::veterancy` and, if the insignia rule changes, the insignia drawing in the client, each citing the revised claims.

## Questions

1. Can a building gain experience from a kill credited to it? If it cannot, which rule says so: a building exclusion in S-0311 claim 8 (the self-credit case), a different absent-key default of the trainable flag for building types, or something else? Does the answer differ for a garrisoned building whose occupants fire, and for a building that undeploys into a vehicle?
2. If a building cannot gain kill experience, can it still hold experience from the other sources the specs name, and does it then show a rank and an insignia?
   - S-0311 claim 11: a building created as a veteran through the infiltrated-vehicle-factory flag.
   - S-0311 claim 13 and S-0415 claim 6: a vehicle that deploys into a building passes its experience to the building.
   - S-0520 claim 22: the veteran crate raises every trainable object in its area, buildings included.
   - S-0311 F5: an object placed by the map takes its experience from the map's veterancy value.
3. Does S-0111 claim 18's "and building" still hold: is an insignia ever drawn on a building, and if so in which of the cases above?
4. Does a building that holds experience (from any source the answer keeps) apply the veterancy abilities of its type's lists (S-0311 claim 3) at its rank?
5. Does a building that undeploys into a vehicle still pass its experience to the vehicle (S-0311 claim 13; S-0415 claim 10), whatever the answers above, and does a vehicle that deploys pass its experience to the building so that it comes back on packing up?

## Possible observation

A skirmish with an armed defensive building whose type gives no trainable key, placed by a place order. Let it destroy a series of enemy units of known cost, enough that F3 would make a trainable killer of the building's cost a veteran and then an elite. Record its experience or rank through a read-only recorder, and whether an insignia is drawn above it. Repeat with the building's type made trainable explicitly in the test map, and once with it made untrainable. Then, separately: a vehicle with experience 1 that deploys into a building and packs up again; a veteran crate taken next to a trainable building; a map that places a building with veterancy value 100; an infiltrated vehicle factory followed by a construction-yard-type building made by deploying. In each, record the building's experience, rank and insignia, and the experience of the vehicle that packs up.

## Current workaround

None in the code: the engine follows S-0311 claims 1, 8, 11 and 13, S-0520 claim 22 and S-0111 claim 18 as written, so a trainable building, which is every building whose type gives no trainable key, gains kill experience, ranks and shows the insignia. We change nothing until this is answered.
