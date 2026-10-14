---
id: Q-0510
title: S-0011 claim 19 still limits destroyed-by-anything to a killing hit, while S-0312 r16 and S-0408 r15 raise it for hidden slaves without one
status: answered
related: [S-0011, S-0312, S-0408, S-0415]
answered-by: [S-0011]
---

## Need

S-0312 revision 16 (claims 30 and 31) and S-0408 revision 15 (claim 10a) give a tagged
hidden slave the common destruction notifications, destroyed-by-anything included, when
its group ends without a killing hit on the slave: a master's killing hit, a removal
without a killing hit, a true sale, and the temporary hidden slaves a successful pack-up
discards (S-0415 claim 10). S-0312 claim 30 itself says that S-0011 claim 19 "must also
admit" these raises and that the passages are to be handed off only with the matching
S-0011 claim 19 correction.

S-0011 is still at revision 15. Its claim 19 opens with "A killing hit on an object
carrying a tag raises that tag with this code", and its deferred-death paragraph says
that removal produces no new destroyed-by-anything raise. Read alone, it gives no raise
for a hidden slave that receives no hit.

The engine (`econ::slaves::end_group`, `combat::death::destruction_raises`) follows
S-0312 and S-0408: the hidden slave raises the common group, with no attacker for a
sale or a removal.

## Questions

1. Does S-0011 claim 19 now admit destroyed-by-anything (and the following
   outside-subset raise) for a tagged hidden slave destroyed when its group ends without
   a killing hit on the slave, in the four cases above?
2. Do the latch and repeat rules of S-0011 claims 8, 9 and 12 apply to these raises as
   they do to a killing hit's? That is, is a repeating tag on such a slave counted the
   same way as for an infantry death without an attacker?
3. Are there other removals that S-0011 claim 19 should list beside these, or are these
   the only exceptions to "no raise at removal"?

## Possible observation

S-0408 scenario S13 already proposes a repeating destroyed-by-anything tag on a hidden
slave with a calibrated counter action. S-0312 notes that no data-only setup is known
that attaches a tag to a generated slave. Any such setup would settle questions 1 and 2.

## Current workaround

The code follows S-0312 r16 and S-0408 r15. A tagged hidden slave whose group ends
raises the common group: with the master's known attacker on a killing hit, and without
an attacker on a removal, a sale or a pack-up's temporary-group cleanup. The tests
`claim_10a_sold_master_hidden_slaves_destroyed` and
`s0312_claim_30_tagged_hidden_slave_raises` in `claims_s0402_slaves.rs` check only that
the tag fires, and that a sale fires it as often as a removal does. They assert no
firing count.
