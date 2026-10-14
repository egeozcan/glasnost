---
id: Q-0480
title: The radar's shroud and friendly-gap tests at cell-centre heights below 0, after S-0110 revision 9
status: answered
related: [S-0410, S-0110]
answered-by: [S-0110]
---

## Need

S-0110 revision 9, claim 8, now owns the shroud point test for heights of
both signs. It sets *u* = trunc(*Z* / 104), and for odd *u* < 0 it uses the
pair *k* = (*u* + 3)/2, under the switch `quirks.shroud_negative_odd_pair`
(default on; off: *k* = (*u* + 1)/2 for every odd *u*). It says the switch
applies wherever the point test is used. S-0111 revision 8 already points
its wrench test to claim 8.

S-0410 (revision 14), claim 6, still says in its backing: "Centre heights
below 0 are `unmeasured` and their radar point-test result is not specified
here: S-0110 claim 8 covers non-negative heights." That reason no longer
holds. Claim 6 builds two tests on claim 8's projection: the shroud test,
and the friendly-gap test, which needs both candidate cells for odd *z*.

The code now gives both radar tests the candidate cells of S-0110
revision 9, including the odd-negative pair and its switch. These are
`econ::minimap::cell_shrouded` and `in_friendly_gap` in
`crates/deterministic/sim`, which share `econ::shroud::point_candidates`.
Before this change, the gap test clamped negative heights to 0.

## Questions

1. At a cell-centre height below 0, is the radar's shroud test exactly
   S-0110 claim 8's point test, including the odd-negative pair and
   `quirks.shroud_negative_odd_pair`?
2. Does the friendly-gap test use the same candidate cells for *Z* < 0? For
   odd *u* < 0, are those the pair *k* = (*u* + 3)/2 and *k* − 1, and must
   both cells lie in a friendly gap field, as for odd *z* ≥ 0?
3. S-0410's Terms already take *z* from S-0110 (its *u*). Can claim 6's
   backing replace the "not specified here" sentence with a pointer to
   S-0110 claim 8, as S-0111 revision 8 did for the wrench?

## Possible observation

Use an R4-style radar capture on S-0110 SH10's flat level −2 map (centre
height −207). Leave *c* − 1 and *c* unexplored, and explore *c* + 1. Run it
once with no friendly gap field, and once with a friendly gap field
covering exactly *c* − 1 and *c*. Record the radar pixel shown for *c*:
black in the first run, darkened terrain in the second.

## Current workaround

Both radar tests take S-0110 revision 9's candidate cells, with the
switch. The S-0410 claim ledger (`docs/m5-claims/S-0410.md`, claim 6)
records this as pending this request.
