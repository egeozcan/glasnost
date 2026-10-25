---
id: Q-1772
title: Do the slot animations of a tech building run before its first ownership change, and what does the change do to them?
status: answered
related: [S-0108, S-0109, S-0405, S-0515, S-0509]
answered-by: [S-0108]
---

## Need

The lead's play note 11 of M8 step A4, round 2 (docs/m8-claims/a4-play-notes.md): "captured oil
derricks do not animate, nor they get the player color". The colour, and the animation of a
captured building, were a drawing bug (a building placed by the map stayed in the scene built at
load) and are fixed. What the animation should do before and at the capture is not settled by
the specs, which read two ways for a building that needs an engineer:

- S-0108 claim 11 starts the *idle*, *active-1* to *active-4* slots at the grand opening, which
  for a building placed by the map happens during placement at game set-up. Nothing in S-0108 or
  S-0109 stops them for an unmanned building.
- S-0405 claim 9 says that a building not in operation stops "its lights and working animations
  (rendering specs)". By S-0405 claim 8 (manned-when-required row) and S-0515 claim 2, a building
  whose type has the needs-engineer flag is not in operation until its first ownership change.
- The only rendering rule that stops or freezes slots for a building that is not in operation is
  S-0108 claim 13, and it covers power only, and only a building that needs and draws power. The
  data's one cash-producing needs-engineer type has no needs-power flag (local lane, read-only),
  so claim 13 never applies to it; the rendering specs give no rule that S-0405 claim 9 could
  point to for it.

The live map view (`crates/app/src/mapview/slot_play.rs`, `live.rs`) has to decide, for such a
building, whether its slots run while it is unmanned, and what its first ownership change does to
them. This is drawing state only (the slot player is not simulation state), so no gate or golden
depends on it.

## Questions

1. A tech building (its type has the needs-engineer flag and does not need power; for example a
   cash-producing one) is placed by the map for the neutral house and has never changed owner.
   Do its slot animations play (advance through their frames), stand still at one frame (which
   one), or not show at all? Does the answer differ between the *idle* slot and the *active-1* to
   *active-4* slots?
2. The same building is captured by an engineer (its first ownership change). On which tick, if
   at all, do its slots start, resume or restart? If they were standing still, does each resume
   from the frame it held or start from its first frame? Which variant (normal, damaged) does a
   slot started at this point use?
3. If the slots already play before the capture (question 1), does the capture change them at
   all (restart, a different slot, nothing)?
4. Does a later ownership change of a building that is already manned (S-0515 claim 2: it stays
   manned) start, stop or restart any slot?
5. Is the answer to questions 1 and 2 the same for a tech building that a map gives to a player
   house from the start (S-0515 claim 2, first bullet), and for a needs-engineer type that does
   need power (where S-0108 claim 13 also applies)?
6. Which slots does "working animations" in S-0405 claim 9 mean, and does it apply to the other
   failed rows of S-0405 claim 8 (switched off, under a pulse, blackout, construction or selling)
   in any way that S-0108 claim 13 does not already give?

## Possible observation

A purpose-made map with a neutral cash-producing tech building and a second one given to a player
house at start, both in view. Record per tick from the first frame of play, from a screen capture
that keeps every drawn frame: whether each slot animation's image frame changes. Then capture the
neutral one with an engineer and record the ticks round the ownership change; then capture it
again with the other house's engineer.

## Current workaround

The code is left as it was for the open case. A map building whose type needs an engineer and
that is not manned keeps the drawing of the scene built at load: its slots stand at their first
image frame (the live slot player runs them from the grand opening, S-0108 claim 11, but they are
not drawn). Once its owner changes (it is then manned) or its health changes, it is drawn live and
its slots play from the frame the slot player has reached; the ownership change neither starts,
stops nor restarts a slot. Every other building's slots play from the grand opening (S-0108
claim 11, S-0109). S-0405 claim 9's working animations are not otherwise modelled, as power
(S-0108 claims 13 and 14) is not (the existing seam `notes::SLOT_ANIMS_LIVE` in
`crates/app/src/mapview/build.rs`). The code site is `Live::redraw` in
`crates/app/src/mapview/live.rs` (`unmanned_tech`).
