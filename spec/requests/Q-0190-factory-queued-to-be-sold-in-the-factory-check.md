---
id: Q-0190
title: Does a factory building queued to be sold count for the factory checks of S-0404 claim 8?
status: open
related: [S-0404, S-0411, S-0407]
answered-by: []
---

## Need
S-0411 claim 14 (revision 6) excludes a factory building that is queued to be sold (its sale
ordered but not yet begun) from new starts, queue requests, resumes, releases and buildability.
S-0404's Terms define "could make" with "is not being sold" only, and S-0404 claim 8 (the check
when a factory building appears, leaves or loses power, and the cancellation when a factory
building is removed for good) reads that definition. The two texts do not say whether the
claim 8 check and the permanent-removal rule treat a building queued to be sold as one that
could make the item. In `econ::production::check_line` and `factory_removed` the engine must
decide.

## Questions
1. A house has two vehicle factory buildings, X and Y, both switched on; Y's sale is queued but
   has not begun. X is switched off (a claim 8 check). Is the running item put on an automatic
   hold (Y does not count), or does it keep running (Y counts)?
2. Same house, but X is destroyed instead. Is the running item cancelled by the claim 8 check
   or by the permanent-removal rule because only Y, queued to be sold, could still make it?
3. In either case, does a queued type that only Y could make stay in the queue?

## Possible observation
S-0404 scenario P8 with the second factory's sale queued (for example, a sale ordered while the
building is still being built up or busy) at the moment of the switch-off or destruction.

## Current workaround
The claim 8 check and the permanent-removal rule count a factory building queued to be sold as
one that could make the type (S-0404 Terms as written: only a building being sold is excluded).
New starts, queue requests, resumes and releases exclude it (S-0411 claim 14).
