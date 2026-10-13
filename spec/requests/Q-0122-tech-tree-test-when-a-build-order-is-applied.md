---
id: Q-0122
title: Does a build order test the tech-tree rules when it is applied?
status: answered
related: [S-0404, S-0407, S-0411]
answered-by: [S-0404]
---

## Need

`econ::production::build` applies a build order (S-0404 claim 3). The specs disagree about
what that test includes.

- S-0404 r2 claim 3: a build order is applied only if the house has a factory building that
  *could make* the type and is switched on. Its Terms define *could make* with the tech-tree
  rules (S-0407) as the last condition.
- S-0407 r4 claim 13: "the build order itself checks only the factory conditions and the build
  limit when it is applied". It also says a type still in the taken-up set after claims 5 to 8
  have come to refuse it (through a non-event of claim 12) "can therefore be ordered and
  produced" (`quirks.stale_offer_orderable`).
- S-0411 r5 claim 4 (*In the span of claim 2*): the build order for such a stale-offered type
  "may take effect under S-0404 claim 3".

If S-0404 claim 3's *could make* includes the tech-tree rule, the stale offer of S-0407
claim 13 can be ordered but never produced, and `quirks.stale_offer_orderable` changes only
legality.

## Questions

1. When a build order is applied, does the factory test of S-0404 claim 3 use *could make* or
   *could make without the tech-tree rule*?
2. If it leaves out the tech-tree rule, does it still apply the build limit with the production
   allowance (S-0407 claims 9 and 16), and is this the only tech-tree-related test at
   application time?
3. Does the same answer hold for a resume order of a held item (S-0404 claim 4) and for the
   check of claim 8, which S-0404 says judges *could make* with the tech-tree rule?

## Possible observation

S-0407's T14 run 8 state: a type stays in the taken-up set after a non-event makes its
prerequisites fail. Issue a build order for it before the next recheck and see whether
production starts.

## Current workaround

`production::build` applies S-0407 claim 13 as written. The test is
`tech::could_make_limit_only`: some switched-on factory building could make the type without
the tech-tree rule, and the build-limit answer with the production allowance is Allowed. The
check of claim 8 and the start of the next queued item still use *could make* with the tech-tree
rule, as S-0404 claim 8 says. Listed in the provisional table of `docs/development.md`.
