---
id: Q-2262
title: Whether holders in behaviours other than attack and guard make the periodic held-target check
status: open
related: [S-0309, S-0215]
answered-by: []
---

## Need
S-0309 claim 29 (revision 70) gives the periodic held-target check to holders whose current
behaviour is *attack* or *guard*, and none to holders in *capture* or *sabotage*. Its Open
questions (*Other behaviours*) leave every other behaviour open. Our engine must decide, for each
behaviour, whether `combat::held::periodic_check` runs.

## Questions
1. Does a holder whose current behaviour is *hunt*, *area guard*, *sticky*, *move*, *patrol* or
   *attack move* drop, in a tick whose tick number is a multiple of 16, a held target that the
   firing check answers *illegal* or *impossible* for?
2. The same for *enter*, *repair*, *harvest*, *return*, *unload*, *rescue* and *missile*, in which
   no weapon is aimed at the held target: is a held target dropped at such a tick, and what does
   the firing check answer for it there?
3. Do aircraft follow the same rule in their own behaviours (S-0501)?

## Possible observation
S-0309 scenario A6 part p run p1, with the holder given each of these behaviours while it holds
the refused target (for *hunt*, a hunt order with the target already held), recording the tick of
any drop and the behaviour current then.

## Current workaround
Only *attack* and *guard* make the check. A holder in any other behaviour keeps such a target
until another rule drops it.
