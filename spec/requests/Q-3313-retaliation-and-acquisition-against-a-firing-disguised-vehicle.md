---
id: Q-3313
title: Retaliation and acquisition against a disguised vehicle that fires
status: answered
related: [S-0516, S-0309, S-0214]
answered-by: [S-0516]
---

## Need

Issue #456: the lead reports that defences fire back at a mirage-type vehicle (a vehicle whose
type disguises when still) that attacks them, and says they should. Our specs say they do not
while it stays disguised:

- S-0516 claim 39: firing sets the fake blink and "does not remove the disguise". While it runs,
  claim 37 (a) shows the real unit to other houses, S-0214 claim 6 lets the attack cursor through
  and S-0309 claim 25 (g) applies.
- S-0516 claim 36: "an object does not retaliate against an attacker that appears disguised to its
  house"; target acquisition uses claim 35 through S-0309 claim 12, table C, disguise row.
- S-0309 claim 25 (g) lets a blinking disguise through, with a draw, only for a searcher of the
  computer-owner houses (the neutral and special houses, and the test-only marked player house).
  A searcher of a player house gets no such chance.

So a defence of a player house whose type cannot detect disguises neither acquires nor retaliates
against a still, firing, disguised vehicle. The engine follows this.

Guard tests on synthetic rules (`claims_s0516_issue_456.rs`) show, for a human-owned defence that
cannot detect disguises and a disguise-when-still vehicle of another house in its range:

- still and disguised: the defence's range search does not pick it, also while its fake blink
  runs;
- on every tick it moves it has no disguise (claims 33 a, 34 b), and the search picks it;
- after a damaging hit, through its re-disguise lockout (claims 34 a, 33 c), the search picks it
  until it disguises again;
- after a stop, it takes its disguise again in its own turn of its first still tick (claim 33 c,
  no lockout after moving), so in our tick pipeline no state between ticks shows it still and
  undisguised. The tests do not check searches made inside that tick.

## Questions

1. In the original, does a hit from a disguised vehicle of this kind let the victim retaliate
   against it, or let the victim's house acquire it, despite S-0516 claims 36 and 39 and S-0309
   table C's disguise row? If so, for which houses (player, computer-owned, both), and is it the
   fake blink that allows it, as in S-0309 claim 25 (g), with or without a draw?
2. If a see-through exists for player houses, does it need the fake blink to be running at the
   moment of the search or the retaliation, or does the hit itself (S-0302 claim 12's damage on the
   victim, not on the firer) make the firer acquirable for a while?
3. Does the retaliation rule of S-0309 claim 18 read claim 35's predicate at the moment of the hit,
   when the projectile was created by a vehicle whose fake blink may have ended by then?
4. In the tick a disguise-when-still vehicle stops, does any other object's search or retaliation
   see it still and undisguised before it takes its disguise again in its own turn (claim 33 c)?
   That is, is the re-disguise of the first still tick taken before or after other objects' turns
   in that tick?

## Possible observation

A player-owned defence that cannot detect disguises and a computer-owned one, each attacked from
inside its range by a still disguise-when-still vehicle of a weapon with a known fake-blink time;
record each tick whether each defence has the vehicle as its target, and when it fires. Repeat with
the vehicle stopping inside the defence's range after a move, recording its disguise and the
defence's target in the stop tick and the next.

## Current workaround

Today's behaviour is kept: the claims as they read, with the guard tests above. Nothing is built.
