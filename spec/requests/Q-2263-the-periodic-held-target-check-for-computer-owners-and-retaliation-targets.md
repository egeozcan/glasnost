---
id: Q-2263
title: The periodic held-target check for holders of computer-controlled owners, and for targets from retaliation or a revert
status: answered
related: [S-0309, S-0304, S-0215]
answered-by: [S-0309]
---

## Need
S-0309 claim 29 (revision 70) covers holders of a human-controlled owner whose target came from
an order or a search; in our engine every player house, AI houses included. Its Open questions
(*Other holders and targets*) leave open whether the rule applies unchanged to holders of
computer-controlled owners (in our engine the neutral and special houses) and to targets that
retaliation (S-0215 claim 18) or a revert (S-0215 claim 10) gave, with a leaning that it does
(low). The Changes for S-0304 claim 2 move the drop of an illegal force-fire target to claim 29's
check only for the actors that claim 29 covers. Our engine does not record what gave a target.

## Questions
1. Does a unit or building of the neutral or special house that holds a target the firing check
   answers *illegal* or *impossible* for drop it in a tick whose tick number is a multiple of 16,
   or at its attack processing, or both?
2. Does a holder of a human-controlled owner whose target came from retaliation, or was given back
   by a revert, drop it at the same ticks as one whose target came from an order?

## Possible observation
S-0309 scenario A6 part p run p1 with *D* owned by a computer player, and a variant in which *D*
takes *X* by retaliation (a hit from *X* with a weapon that does damage, then *X*'s warhead set so
that *D*'s weapon answers *illegal*), recording the tick of the drop.

## Current workaround
Holders of player houses make the check whatever gave the target. Holders of the neutral and
special houses make no periodic check and keep S-0304 claim 2's drop of an *illegal* target at
the first check of their attack processing; a holder of a player house no longer drops it there.
