---
id: Q-1757
title: Whether an infantry unit released with no rally point ever leaves the factory, and what then happens to the next release
status: answered
related: [S-0417, S-0404, S-0215, S-0208, S-0209]
answered-by: [S-0417]
---

## Need
S-0417 claim 10 and S-0404 claim 13 have no destination for an infantry unit that an
infantry factory releases with no rally point, or with a rally point in the unit's own
cell. The unit appears on a foundation cell of the factory, rests in *guard* and keeps
factory contact and tether. With one contact place, every later infantry release from that
factory is *busy* (S-0417 claim 14). S-0404 claim 7 then cancels the item with a refund.
S-0417 Open question 3 leaves open whether any later event moves the unit off the
foundation. Claim 10 says this low-confidence case is "not a prediction of a permanent
wait".

Our engine implements the claims as written: `econ::release::infantry`
(crates/deterministic/sim/src/econ/release.rs:849-850) gives no destination and keeps contact.
In play (M8 step A4) the project lead reports that units from a barracks with no rally point
stay at the entrance, and that later units cannot get out until a rally point is set. The
engine needs to know whether this matches the game or whether the game has a walk-out
rule that the specs lack.

## Questions
1. In a normal skirmish game, a human house builds one infantry unit at an infantry factory
   with no rally point set. Does the unit, without any order from the player, ever leave the
   factory's foundation cells? If it does, on which tick after release does it first move,
   which cell does it go to, and what starts the move (for example the release itself, a
   periodic event of resting infantry, or the next unit's release)?
2. If it leaves, when does the factory stop being in contact with it (on which arrival, or on
   leaving the foundation)?
3. A second infantry unit of the same house finishes while the first one is still resting on
   the foundation (if question 1 leaves that possible). Does the second unit appear? If not,
   what happens to the finished item: is it kept done until it can be released, retried,
   or cancelled with a refund? Does the answer differ between factories that have one contact
   place and those with several?
4. Does setting a rally point after the first unit is out move that resting unit, or does it
   affect only units released afterwards?
5. Does any of the above differ for a cloning building's copy, a jumpjet infantry type or a
   teleporting infantry type released without a rally point?

## Possible observation
S-0417 scenarios U10 part b and U13 part a already describe this. Record the released unit's
destination, position, current behaviour, contact and tether for 300 ticks or more with no
rally point and no orders, then finish a second unit and record its release result, sidebar
state and credits. Repeat with a rally point set after the first release (question 4).

## Current workaround
The claims as written: no destination and contact kept, so later releases are busy and their
items are cancelled with a refund (S-0404 claim 7). No invented walk-out, scatter or contact
timeout.
