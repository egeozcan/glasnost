---
id: Q-1590
title: Whether a railgun system without a held particle type makes its laser beam
status: open
related: [S-0616, S-0518]
---

## Need
S-0616 claim 31 says a railgun system whose type has the laser flag makes one laser beam in its
line turn (claim 17). Claim 17 says that when the line count *n* ≤ 0 nothing is made and nothing is
drawn. Claim 20 says that with no held type a railgun system makes no particle and takes no draw,
and is still marked. Neither claim says whether such a system, with *n* > 0, still makes the
laser beam of claim 31. The game client draws the beam from the line turn
(`Effect::RailLine`, `crates/deterministic/sim/src/particles.rs`, `railgun_turn`;
`crates/app/src/mapview/beams.rs`, `rail_beam`) and must decide this case.

## Questions
1. A railgun system whose type has the laser flag but no held particle type (no first particle
   type, S-0616 Terms *held type*), with *n* > 0: does its line turn make the laser beam of
   claim 31?
2. Does the answer depend on *n* in that case (is the beam made also when *n* ≤ 0 without a held
   type)?

## Possible observation
Named test rules: a railgun system type with the laser flag, a pure laser colour, a positive
particles-per-lepton field and no particle type, fired at a point a few cells away. Frame capture
of the 10 ticks after the shot, checked for the laser line.

## Current workaround
Interim rule (docs/development.md, *Visual review 2026-10-19*): the beam is made only when the
system has a held type and *n* > 0; a system without a held type draws no laser.
