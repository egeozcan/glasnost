---
id: Q-3206
title: S-0613 claim 19 (gas made on a high bridge's deck sinks through it) is reported wrong by the project lead
status: answered
related: [S-0613, S-0616, S-0102]
answered-by: [M-0026, S-0613]
---

## Need
GitHub issue #424 was reopened on 2026-11-16. The project lead writes: "The claim is wrong, gas
doesn't sink under the bridge in the original game." The claim is S-0613 claim 19 (r22, the answer
to Q-3054). It gives `quirks.gas_through_bridge_deck`, default on: a gas particle has no on-deck
state, every height rule measures from the terrain under the bridge, and a cloud made at deck
height sinks under the deck, is drawn under the bridge and hits only ground-layer objects (claims
9, 11, 13 and 19).

The engine has the other branch built, behind the switch off (render batch R9, merged 4231289).
In that branch, a gas particle at or above its high-bridge cell's deck height takes the deck as
its ground for claims 9 and 11, and it hits the bridge-deck layer (claim 19's Off text, S-0616
claim 14's deck height). The engine needs to know which branch is the original's, and whether the
Off text is right in its details, before the default changes. Code changes nothing until this is
answered.

## Questions
1. A gas cloud is made on a high bridge's deck: an infantry unit standing on the deck dies a virus
   death, or a chaos drone on the deck releases its gas. Does the cloud stay at the deck's height
   for its whole life, as the lead reports, rather than sinking to the ground or water under the
   bridge?
2. If it stays at the deck's height: while it is over the bridge's cells, does it sink toward
   5 leptons above the deck, as claim 19's Off text gives (the deck height in place of the
   terrain for claims 9 and 11)? Or does it keep the height it was made at?
3. When such a particle drifts off the bridge's cells (by wind or deflection), what happens? Does
   it fall to the terrain at the usual sinking rate, drop at once, or keep the deck's height?
4. Does a cloud on the deck hit the objects standing on the deck (the bridge-deck layer), and no
   longer the objects on the ground or water under the bridge?
5. Is a cloud on the deck drawn over the bridge, with the deck's objects?
6. A cloud made under the bridge (S-0613 scenario G4 (a)): is it unchanged, staying under the
   bridge, drawn under it and hitting only ground-layer objects?

## Possible observation
The mirror of S-0613 scenario G4 (a), as Q-3054 proposed. Kill an infantry unit with a virus
weapon in the middle of a high bridge's deck, with a second infantry unit next to it on the deck
and a third on the ground under the bridge. Record the cloud's on-screen height against the deck
and the ground for a few seconds, which of the other two units take damage, and where the cloud
goes once the wind carries it past the bridge's edge. Repeat with a chaos drone on the deck
releasing its gas.

## Current workaround
The code follows S-0613 r22 as written: `quirks.gas_through_bridge_deck` is on by default, so gas
made on a deck sinks under it. A player or the lead can turn the switch off in a ruleset to get
the deck-ground branch now. No default changes until this is answered.
