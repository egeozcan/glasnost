---
id: Q-3302
title: Damage aimed at a bunkered vehicle, and whether the bunker takes it
status: answered
related: [S-0302, S-0309, S-0606, S-0303]
answered-by: [S-0302]
---

## Need

Issue #457: the lead reports that a vehicle inside a tank bunker takes damage, and says it should
take none, with the bunker taking the damage instead.

S-0302 claim 8, stage 2, has two bunker rows: a bunkered unit hit by a warhead that does not
penetrate bunkers takes nothing, and a tank bunker holding a unit hit by a warhead that does
penetrate bunkers takes nothing. *Hits without a warhead* says a hit with no warhead counts as
neither, so neither row stops it. S-0309 claim 24 redirects an attack on a bunkered vehicle to the
bunker only for a temporal weapon, a locomotor warhead and the airstrike callers. S-0606 claim 9
cites both. Nothing passes a stopped hit on to the bunker.

The engine follows these claims. Every hit reaches one damage function whose stage 2 rows are
the two above, for a direct hit and for each object an area detonation (S-0303) reaches. Tests on
synthetic rules show:

- a direct hit on the bunkered vehicle with a non-penetrating warhead: the vehicle takes nothing,
  the bunker takes nothing;
- the same with a penetrating warhead: the vehicle is damaged, the bunker takes nothing;
- splash centred on the vehicle or on the bunker's centre: with a non-penetrating warhead the
  bunker is damaged and the vehicle is not; with a penetrating warhead the vehicle is damaged and
  the bunker is not;
- a hit with no warhead: neither bunker row holds, but one that does not ignore defences has
  *m*₀ = 0 (claim 6) and the zero-amount row stops it, so the vehicle takes nothing; one that
  ignores defences damages the vehicle;
- a hit that ignores defences passes both rows.

So a vehicle in a bunker takes damage today from a penetrating warhead (direct or splash), from a
hit that ignores defences, and from a hit with no warhead that ignores defences, as the specs say.
No hit stopped at the vehicle is passed on to the bunker.

## Questions

1. In the original, does a bunkered vehicle take damage from a penetrating warhead, direct or
   splash, while the bunker takes none? Or is that damage, or some of it, given to the bunker?
2. When a non-penetrating warhead hits a bunkered vehicle directly and is stopped, does the bunker
   take that hit instead, or does nobody take it?
3. Does a hit with no warhead that ignores defences (S-0302 claim 8, *Hits without a warhead*)
   damage a bunkered vehicle?
4. Does any attacker other than the three of S-0309 claim 24 have its attack on a bunkered vehicle
   turned into an attack on the bunker?

## Possible observation

A tank bunker holding one vehicle of a known strength, attacked in turn by a unit with a
penetrating warhead and by one with a non-penetrating warhead, each from outside its splash
reach of other objects; record both objects' health each tick and which object each shot names.
Repeat with an area weapon whose detonation falls on the vehicle's position and on the bunker's
centre.

## Current workaround

Today's behaviour is kept: the claims as they read, with guard tests that cite them
(`claims_issue_457.rs`). Nothing is passed on to the bunker.
