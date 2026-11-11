---
id: Q-2986
title: Which hits on a building set the last base hit tick of S-0217
status: answered
related: [S-0217, S-0302]
answered-by: [S-0217]
---

## Need

S-0217 claim 3 sets a house's last base hit tick *L* to the current tick "whenever a building of
the house is hit by damage that has an attacker". Its Open questions leave out whether a hit that
the building's armour reduces to nothing, or a healing hit, counts. S-0302 claim 8 stops some hits
before any damage is worked out, in stages: stage 1 (a temporarily invulnerable target, a target
being erased, a building's own damage without the self-damage mark), stage 2 (immunities, allies,
the bunker rows) and stage 4 (an amount of 0, immune, no health on a non-building). S-0217 does
not say whether a hit that S-0302 claim 8 stops at one of these stages is a hit "by damage" for
*L*. The building's own damage without the self-damage mark is settled by S-0217 itself (it does
not hit the building); the other rows are not.

Our test-only computer-controlled houses (S-0215 Terms) need one answer, since *L* decides whether
the house sells its buildings and gives the house-wide hunt order at a check.

## Questions

1. Does a hit with an attacker that the building's armour (or the warhead's modifier against that
   armour) reduces to a final amount of 0 set *L*?
2. Does a healing hit (a negative final amount) with an attacker set *L*, whether or not the
   building is at full health?
3. Does a hit with an attacker that S-0302 claim 8 stops at stage 1 (temporary invulnerability, a
   building being erased) or at stage 2 (an ally's warhead that does not affect allies, an
   immunity) set *L*?
4. Does a hit with an attacker whose final amount is positive but which finds the building already
   at 0 health (S-0302 F4: *m*′ = 0) set *L*?

## Possible observation

Scenario H2 of S-0217, part (b), with *D*'s unit armed with (1) a warhead whose modifier against
the building's armour is 0, (2) a healing weapon, (3) a warhead that does not affect allies with *D*
allied to *C*. In each, record the tick at which *C*'s building starts its sale; a sale from
*S*(901) to *S*(1012) means the hit did not set *L*, a sale only from *S*(*h* + 901) means it did.

## Current workaround

`computer::building_hit` is called from `combat::damage::apply_hit` only for a hit that lowers the
building's health (the lost amount *m*′ of S-0302 F4 is above 0) and has an attacker that is an
object in the world. Hits that S-0302 claim 8 stops, hits with a final amount of 0, healing hits
and hits on a building at 0 health do not set *L*. Cited as `// spec: S-0217 claim 3 ... and
Q-2986 (interim ...)`. Tests: `crates/deterministic/sim/tests/claims_s0217.rs`
(`claim_3_a_hit_with_an_attacker_sets_the_last_base_hit_tick`).
