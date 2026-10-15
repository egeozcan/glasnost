---
id: Q-0678
title: Which span the far-side named cell of a direct pier collapse checks
status: answered
related: [S-0511]
answered-by: [S-0511]
---

## Need

S-0511 claim 6 says that after a direct pier collapse "Both adjacent spans have claim 7's
isolated-run result: the start-side span is checked to completion before the far-side span".
Claim 7's *Finding the span* names the reference cell's start-side neighbour and then its
far-side neighbour as the two loss cells. From each one it takes the first marked neighbour,
resolves its section, and searches "toward the start side up to the legal-map boundary ... for
the nearest start ramp at sub-tile 8 along x or 12 along y, or non-collapsed pier at sub-tile 5
along x or 7 along y".

Read literally, the far-side search cannot reach the far-side span. The far-side neighbour is
the collapsed pier's own sub-tile 5 (along x) or 7 (along y) cell. Its marked neighbours resolve
to sections at that cell or to its start side. The search goes toward the start side, so the
first start piece it can meet that belongs to the far-side span is the pier that has just
collapsed, and the pier is now in state 4. The search passes over it and finds the start-side
span's start piece. The result is that the start-side span is checked twice and the far-side
span is not checked at all. This contradicts claim 6 and scenario B15, which expect actions on
both spans.

`bridges::high::span_check` needs to know which span the far-side named cell checks.

## Questions

1. After a direct pier collapse, does the far-side named cell's search accept the pier that has
   just collapsed as the far-side span's start piece? Or does the original run the search before
   the pier's tile changes to state 4? Or does it do something else?
2. If the literal reading holds, so that the far-side search finds the start-side span, is that
   span checked a second time? Does it then receive a second set of band raises?
3. Does a pier that collapsed earlier, on its own or through a notice, count as a start piece for
   a later section loss on its far side? Claim 7's "non-collapsed" wording says no.

## Possible observation

Use S-0511 scenario B15's two tagged adjacent spans. Give each span one missing section, so that
every check of a span raises its band once. Then collapse the pier between them directly and
count each tag's raises. A count of one raise per span answers question 1. A count of two
start-side raises and no far-side raise answers question 2.

## Current workaround

The far-side named cell's start search also accepts the pier collapsed by this attempt, at its
start sub-tile, as its span's start. The start-side search, and every search after a section
loss, require a non-collapsed pier. The choice is marked in the code with this request's ID.
