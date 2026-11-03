---
id: Q-2356
title: The attack cursor and order acceptance for an actor that is materialising
status: answered
related: [S-0214, S-0301, S-0304, S-0207, S-0513]
answered-by: [S-0214]
---

## Need

S-0301 r69 claim 9 (*A materialising firer within rank 4*) gives a materialising firer
*rearming* for every target and weapon, ahead of every other rank-4 answer: the projectile
refusals of S-0306 claim 23, the hidden-target *impossible*, the parasite warhead's *illegal*
against a protected object and the rest. S-0214 claim 6 (e) asks whether "the firing rules of the
combat specs do not rule the target illegal for the weapon it would use", and says that "other
reasons for not firing at this moment (facing, reloading, rotating, range and so on) do not
count". Our attack rule asks the firing check without ranks 5 to 8 for (e) (S-0214 claim 6 (e),
S-0304 claim 2).

Materialising is a state of the moment, like reloading, but its answer sits in rank 4 above the
target tests. Read one way, (e) passes over it, so the target tests decide and the attack cursor
does not appear over a target the weapon cannot hit. Read the other way, the firing check's answer
is *rearming*, not *illegal*, so (e) holds for every target while the actor materialises, and the
cursor appears, and an attack order is accepted, over a target its weapon can never hit (for
instance an aircraft in the air, for a weapon that is not anti-air). The same question decides
the AI's lasting-legality tests, which use the same check.

S-0214 settles the falling firer's rank-4 refusal explicitly (a firing rule, so (e) fails), but not
the materialising *rearming*. Where: `combat::check::answer`, the `moment` flag, used by
`check::check_lasting` (`targeting::attack_rule` part e and the AI).

## Questions

1. Without force-fire, while an own unit is phasing in after a jump of the teleporting locomotor
   (S-0207 claim 20 (f)), or while a superweapon warp member phases in (S-0513 claim 39), does
   the attack cursor appear over an enemy object that its weapon cannot hit (an aircraft in the
   air for a weapon that is not anti-air; a target hidden from its house; a protected object for a
   parasite warhead)?
2. Over an enemy object its weapon can hit, does the cursor appear while it phases in?
3. If the cursor appears in question 1, is the attack order accepted, and does the unit drop the
   target at S-0309 claim 29's first check tick after the phase-in ends (S-0309 A6 part p, run
   p10a)?

## Possible observation

Select a unit that moves with the teleporting locomotor and whose weapon is not anti-air, with an
enemy aircraft hovering or flying within view, and a second enemy that it can hit. Order a jump;
during the phase-in after it, hover the mouse over each enemy without force-fire, record the
cursor, and click; record whether an order is accepted. Repeat after the phase-in as the control.

## Current workaround

Interim: the attack rule's check (`check_lasting`, `moment` off) passes over the materialising
*rearming* as a reason of the moment, as it passes over ranks 5 to 8, so the target tests decide
(S-0214 claim 6 (e), "other reasons for not firing at this moment ... do not count"). Every other
firing check (the held-target check, the firing step, the searches and retaliation) gives the
materialising *rearming* first in rank 4, as S-0301 r69 claim 9 gives. No quirk switch.
