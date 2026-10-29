---
id: Q-2071
title: Pip scale with an unknown pip-scale word or a negative pip wrap
status: answered
related: [S-0111, S-0003]
answered-by: [S-0111]
---

## Need

S-0111's Import mapping gives the `PipScale` vocabulary (none, ammo, tiberium, passengers,
power, mindcontrol) but, unlike `Pip` and `OccupyPip`, no result for a word that matches none
of it. S-0003 claim 16 says each named-value field has its own fixed fallback, stated by its
topic spec. Claim 15 covers a pip wrap of 0 and above 0, but not a negative one. The pip-row
drawing in `crates/app/src/mapview/pip_scale.rs` (`pip_kind`, `ammo_row`) and the importer
warnings in `crates/convert/src/sim_combat.rs` (`pip_scale_checks`) need both answers.

## Questions

1. A type whose `PipScale` is a word outside the vocabulary (for example `sideways`): which
   pip scale does it show? Does it behave as one of the six kinds (which), or as something
   else? Does it also change the row length a passenger capacity uses (claim 12, T2)?
2. A type with the ammunition kind and a negative `PipWrap`: which ammunition pips does it
   show, and in which image frames? Does the result depend on the ammunition capacity and the
   current ammunition, as for a wrap of 0?

## Possible observation

OV10's setup with a test type given `PipScale=sideways` and a passenger capacity, loaded with
one infantry passenger; and an ammunition type with `PipWrap=-2` at a few ammunition counts.

## Current workaround

An unknown word reads as none (no row, row length 0), with an importer warning. A negative wrap
draws as a wrap of 0. Both are marked `design:` with this request's ID.
