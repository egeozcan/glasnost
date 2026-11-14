---
id: Q-3087
title: Which key and which layers give a building type's water-bound default
status: open
related: [S-0406, S-0211, S-0006]
answered-by: [S-0406]
---

## Need

GitHub issue #200. S-0406's Import mapping row for `WaterBound` (rules, building type) gives the absent-key default:

> no, unless the type's locomotion-class key already names *float*

To build this, the importer has to read a locomotion-class key in a building section. No spec says which key that is in a building section, how its value is read there, or what "already" means across rules layers.

S-0211's Import mapping maps the locomotion-class key, `SpeedType`, only for "vehicle, infantry or aircraft section", with the names `Foot`, `Track`, `Wheel`, `Hover`, `Winged`, `Float`, `Amphibious` and `FloatBeach` (case ignored) and defaults for those three kinds only. No row maps it for a building section. Today the converter (`crates/convert`) reads `SpeedType` for infantry, vehicle and aircraft types only, and `place_type` in `crates/deterministic/sim/src/econ/placement.rs` applies "absent: no" alone.

## Questions

1. In a building type's section, is the locomotion-class key of S-0406's `WaterBound` row the key `SpeedType`, read with S-0211's names (case ignored)? If not, which key is it?
2. If a building section's `SpeedType` holds a word that is not one of S-0211's names, or is empty, is the type water-bound by default?
3. "Already": rules layers are applied in order (S-0006 claim 1). The building type's `WaterBound` key is absent in every layer. Is the default decided by the class in force at the end of all layers, or by the class in force at some earlier point? For example: the base rules name `Float` and a later layer names `Wheel`; or the base rules name no class and a later layer names `Float`.
4. A layer that names `WaterBound` explicitly, followed by a later layer that changes only `SpeedType`. Does the explicit value stand, or does the later class change it?
5. Does a building section's locomotion-class key change anything else about the building or its placement besides this default?

## Possible observation

In a test rules file, give a 1 × 1 buildable test structure `SpeedType=Float` and no `WaterBound` key. Try to place it on a water cell and on clear land, and compare with the same type given `WaterBound=yes` and with neither key. Repeat with `SpeedType` set in a map's rules layer instead of the rules file, and with the two keys split across layers as in questions 3 and 4.

## Current workaround

No change. A building type with no `WaterBound` key is not water-bound, whatever its section's locomotion-class key says (`place_type`, S-0406 Import mapping, "no").
