---
id: Q-2232
title: Does a departed warp member still count as standing on a deck after its placement on that deck is refused?
status: open
related: [S-0306, S-0513]
answered-by: []
---

## Need
S-0306 revision 57, claim 17: a superweapon warp member between departure and appearance counts
"as standing on a deck only once its landing check has passed for a landing cell on a deck", and
otherwise by its height above the surface beneath the position it kept. So, with
`quirks.warp_deck_departure_airborne` on, a member that departed from a high-bridge deck is
airborne for a projectile until a landing check passes for a deck cell.

S-0513 claim 50 lets a placement be refused after a passed check (a passed check is not a
reservation). The member then goes back to the landing retry, and its next update repeats the full
landing check, which can pass again, fail, or redirect it to another cell. The engine must decide
whether "once its landing check has passed" still holds in that retry
(`superweapons::warp::passed_for_deck`).

## Questions
1. A member that departed from a high-bridge deck has a landing check pass for a landing cell on a
   deck, and then its placement there is refused (S-0513 claim 50). Until its next landing check,
   does a projectile aimed at it treat it as standing on a deck (not airborne), or as airborne
   again?
2. If that next check fails and the member is redirected to a landing cell that is not on a deck,
   is it airborne again (until a check passes for a deck cell), or does it still count as standing
   on a deck?
3. If a later check passes for a landing cell that is not on a deck, does the member still count
   as standing on a deck?

## Possible observation
S-0306 scenario P11 run f with V on a high-bridge deck and an anti-air test projectile, the
landing cell on another deck. Block the placement after the passed check as S-0513's claim 50
scenario does (a unit moved onto the landing cell between check and placement). Record in each
tick whether the anti-air row detonates the projectile near V's old position, and which target
the projectile has after a landing-check kill in the retry.

## Current workaround
The member counts as standing on a deck only while a passed check for a deck landing cell waits for
its placement (`Stage::Away` with a landing cell on a deck). A refused placement takes it back to
the retry, where it counts by its height again until a check passes for a deck cell. Questions 2
and 3 follow from that: only the latest passed check counts, and only until the next placement or
refusal.
