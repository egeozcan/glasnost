---
id: Q-1760
title: Which turret model a multi-turret vehicle shows
status: answered
related: [S-0105, S-0007, S-0308, S-0606, S-0209]
answered-by: [S-0105]
---

## Need

S-0105 claim 12 says a vehicle of a multi-turret type draws the turret and barrel models of its
*selected turret index*. Its Terms say that the index is set "for most such types by the weapon
currently chosen, for some by the progress of reloading", and leave the rule to the vehicle and
combat specs. No approved spec defines the index. S-0606 claim 1 ("the weapon of its current
turret index") and S-0209 ("the weapon of its currently selected turret") use the index too, but
neither says what sets it. S-0308 defines the weapon number (claim 5) but does not tie it to the
turret model.

In the code, the vehicle turret drawing in `crates/app/src/mapview/build.rs` (the turret block
after the body, about lines 4553 to 4600) needs this index to pick the importer's part
`turret{k}` / `barrel{k}` (`crates/convert/src/art.rs`, `voxel_parts`) and to fill the index
component of the turret image key (S-0105 claim 26).

## Questions

1. For a vehicle of a multi-turret type, which turret model is shown: the one numbered like the
   vehicle's weapon number (S-0308 claim 5), or something else? Does a gunner vehicle's model
   change at the moment its weapon number changes when a passenger enters or leaves?
2. Which multi-turret types choose the model by the progress of reloading instead? What property
   of the type makes them do so? How does the shown model step through the turret models as the
   reload runs, and which model is shown when the vehicle is fully loaded or empty?
3. What is shown when the value that selects the model is not below the type's turret count
   (for example a weapon number equal to or above the multiple-turret count)? Is it the first
   turret model, no turret, or something else?
4. Which turret model does a newly created vehicle show (built, placed by the map, or unloaded
   from a transport), and is it kept through veterancy changes?
5. Does the frame of a numbered turret model follow S-0105 claim 16's vehicle turret frame?
   That claim leaves this open.

## Possible observation

Make a test vehicle type with the turret flag, a multiple-turret count of 3 and three clearly
different turret models (TUR, TUR1, TUR2). Give it the gunner flag and three passenger types
whose mode indices are 0, 1 and 2. Load and unload each passenger in turn and screenshot the
turret. Then try a weapon number at or above the turret count. Repeat with a type whose model is
expected to follow its reloading, firing until it is empty and then letting it reload.

## Current workaround

Today no numbered turret is drawn: the renderer reads only the single `turret` / `barrel`
parts, so multi-turret vehicles show no turret (a separate code fix). Until this request is
answered, the index rule stays unspecified. Any interim choice, such as the weapon number with
an out-of-range value drawing model 0, needs the lead's approval and a named quirk switch marked
unspecified.
