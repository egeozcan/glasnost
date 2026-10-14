---
id: Q-0413
title: Slaves of a master that ends other than by destruction or sale, and slaves while the master builds up or sells
status: answered
related: [S-0402, S-0312, S-0408, S-0415]
answered-by: [S-0402]
---

## Need

S-0312 claim 30 says what becomes of a destroyed master's slaves, and S-0408
claim 10a a sold one's. A master can also end in other ways, and its group
can be due while it is in construction or selling. The code is
`econ::slaves` (`removing`, `master_sold`, `end_group`, `group_turn`) and
`econ::deploy` (a failed pack-up).

## Questions

1. A deployed slave miner starts to pack up, and the pack-up fails at its
   end, so the building is gone and no vehicle comes out (S-0415 claims 10 to
   12). What becomes of its slaves, inside and outside?
2. A master removed from the game without being destroyed or sold (for
   example by a map trigger that removes it): what becomes of its slaves?
3. While a deployed master is still building up, or is being sold or packed
   up, do ready slaves leave it? Do slaves that return go inside and pay?
   Does it check for a field or pack up?
4. A slave left without a master, with no house to receive it: S-0312 claim
   30 hits it with its current health. Is it also told to guard first, as a
   slave given to another house is?

## Possible observation

A purpose-made map with a deployed slave miner of a human player on a field
with its slaves out. (a) Block the pack-up's return cell, then pack it up.
(b) Remove it with a trigger. (c) Sell it while slaves are walking back.
(d) Place it with a trigger and watch its build-up. Record each slave's
owner, behaviour, health and position.

## Current workaround

1 and 2: the slaves are treated as after a sale (S-0408 claim 10a). 3: no
slave leaves and the master makes no field check or pack-up while it is in
construction or selling; returning slaves still go inside and pay. 4: they
are told to guard too. Listed in docs/development.md (slave miners).
