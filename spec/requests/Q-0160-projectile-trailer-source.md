---
id: Q-0160
title: Where a projectile's trailer animation and trailer delay are read (S-0306 against S-0007 claim 33)
status: open
related: [S-0306, S-0007, S-0004]
answered-by: []
---

## Need
S-0007 revision 7 claim 33 and S-0004 revision 8 claim 14 (answering Q-0083) say that a
projectile's trailer animation and trailer delay come from the art database, from the section
named by the non-empty image value of that read's rules section, and that a read without an image
value keeps both settings. S-0306 revision 3, claim 16 and its Import mapping, still list
`Trailer` and `SpawnDelay` under "rules, projectile", with defaults none and 3. The importer's
simulation data (`convert::sim_combat`) reads both from the rules section today, and the
simulation's trailer timing (S-0306 claim 16) uses those values. The two specs cannot both hold.

## Questions
1. Does S-0306 claim 16 use the trailer animation and trailer delay of S-0007 claim 33 (art
   section named by the projectile's current rules image), so that `Trailer` and `SpawnDelay` in a
   projectile's rules section have no effect?
2. If so, does S-0306's warning about a trailer with a delay of 0 apply to the art values?

## Possible observation
S-0007 scenario A24 already places distinct trailers in the rules section and the art sections;
recording which trailer appears in flight settles question 1.

## Current workaround
The type registration of trailer animations follows S-0004 claim 14 (art section). The
simulation keeps reading the trailer animation and delay from the projectile's rules section, as
S-0306 says, until the conflict is settled.
