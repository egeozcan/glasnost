---
id: Q-1329
title: Restoration of a set-down unit that stays in guard
status: answered
related: [S-0607, S-0215]
answered-by: [S-0607]
---

## Need
S-0607 claim 20 restores a set-down unit "on its first fall-back to rest after touchdown". A seized unit is in *guard* (claim 5), and S-0215 claim 24 gives *guard* no fall-back to rest. So an unarmed or idle set-down unit stays seized until something else makes it fall back. That can be a move order (scenario L11 part a), or a target found by automatic search and then refused under claim 10. Until then it cannot fire (claim 10) and refuses destinations (claim 9).

## Questions
1. Does a set-down unit left alone in *guard* stay seized indefinitely, with its weapons refused, until an order or a refused target makes it fall back to rest?
2. Is the first move order after touchdown lost (its destination refused while seized), so that only a second order moves the unit (L11 part a, "the tick the unit first accepts a move order")?
3. A survivor kept in play with `quirks.lift_impact_removes_survivor` off: is it placed on the surface under its impact point? Its fall may end a few leptons below the surface.

## Current workaround
As in questions 1 and 2: restoration waits for the first fall-back to rest. The first move order's destination is refused, and *move* with no destination falls back to rest and restores the unit. A survivor set down with the quirk off is put on the surface. A fall that ends by crossing a bridge deck puts the unit on the deck.
