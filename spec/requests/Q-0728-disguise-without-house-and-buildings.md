---
id: Q-0728
title: Disguise tests for buildings, aircraft and a disguise without a house
status: open
related: [S-0516, S-0410, S-0209]
answered-by: []
---

## Need

Three disguise questions are open across three specs:

- S-0516 Open questions ask whether claim 35 should apply unchanged to buildings and aircraft.
- S-0516 claim 38 and S-0410 claim 8 leave open which radar colour a disguised unit shows to other
  houses when its disguise house is none.
- S-0209 row WP25 asks whether enemy infantry "appears friendly" through a disguise. That is not
  defined for a disguise without a house.

The code needs these rules in four places:

- `crates/deterministic/sim/src/cloak/disguise.rs` (`appears_disguised`)
- `cloak/view.rs` (`radar_house`)
- `econ/minimap.rs`
- `occupancy.rs` (row WP25)

## Questions

1. Does a disguised building or aircraft appear disguised to every house, its owner included, for
   targeting and retaliation?
2. A unit has a disguise with no disguise house, such as a terrain disguise. Which colour does it
   show on an enemy radar: its owner's, none (not drawn), or the neutral colour?
3. Does an enemy infantry unit whose disguise has no house count as appearing friendly to a walker
   (S-0209 row WP25)?

## Possible observation

- Place a mirage-type vehicle disguised as a tree next to an enemy radar.
- Place an enemy spy, disguised by default, in the corridor of an armed walker's route.

## Current workaround

- Claim 35 is applied unchanged to buildings and aircraft.
- A disguise without a house shows its owner's colour on the radar.
- Row WP25 applies only to a disguise that has a house.

These choices are marked in the code with this request's ID.
