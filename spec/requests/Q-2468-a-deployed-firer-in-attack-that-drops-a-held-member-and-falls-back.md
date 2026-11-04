---
id: Q-2468
title: Does a deployed infantry firer in attack keep its beam when its deployed-firing rule drops a held member and it falls back to rest
status: open
related: [S-0513, S-0517, S-0514, S-0215, S-0309]
answered-by: []
---

## Need

S-0513 r108 claim 39a (*The firer*, Q-2418) says of a deployed infantry unit of a
human-controlled owner whose temporal beam is attached to a warp member: "At each processing at
which S-0215 claim 23 or 31 runs S-0517 claim 16's deployed-firing rule, that rule applies to the
member at its old position, as to a target on the map there: beyond the range of the weapon that
rule selects, it drops the member, or takes the target that rule's search finds instead. Its beam
stays attached (*Release*, Q-2418)." S-0309 r113 claim 27 repeats "while its beam stays attached".

S-0517 claim 16 ends: "A unit left targetless falls back to rest under S-0215 unless its current
behaviour is *guard* or its type is a jumpjet type." S-0514 claim 14 lists "reaching the common
fall-back-to-rest effects" among the release causes, and S-0513 claim 39a's *Release* item says
the beam lets go only at S-0514's causes, claim 14 among them.

In *guard* (S-0215 claim 31) the rule drops the member and there is no fall-back, so the beam stays
attached: the specs agree. In *attack* (S-0215 claim 23, the case Q-2418 asked about), the rule
drops the member, finds nothing, and the unit falls back to rest. The fall-back's common effects
then let the beam go by S-0514 claim 14, which disagrees with "its beam stays attached".

SW12 part l9 orders *T* to deploy after its attack order, so which behaviour *T* is in when its
rule runs depends on what the deploy order leaves current.

## Questions

1. A deployed infantry unit of a human-controlled owner in *attack*, whose temporal beam is
   attached to a warp member beyond its weapon's range of the old position, drops the member at
   its deployed-firing rule with no other target found. Does it fall back to rest there (S-0517
   claim 16), and does its beam then let go (S-0514 claim 14)? Or does the beam stay attached, and
   if so, is it because no fall-back happens, or because this fall-back is not a release cause?
2. Is the answer the same for such a unit holding an ordinary target on the map that it erases
   (not a warp member), when its target moves out of range?
3. In SW12 part l9, which behaviour is *T* in at the first processing after its deploy completes:
   *attack*, *guard*, or another?

## Possible observation

SW12 part l9 as written, also recording *T*'s behaviour just before and just after each processing
from the deploy's completion to the departure update.

## Current workaround

The engine follows S-0517 claim 16 and S-0514 claim 14: in *attack* the drop is followed by the
fall-back to rest, which lets the beam go; in *guard* the beam stays attached. The test
`claims_s0513_r89::r108_claim39a_a_deployed_infantry_firer_beyond_range_drops_the_member_and_stays_put`
asserts both, with the attack case marked as the interim answer.
