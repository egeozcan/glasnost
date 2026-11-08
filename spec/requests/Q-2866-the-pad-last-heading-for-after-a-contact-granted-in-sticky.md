---
id: Q-2866
title: The pad last heading for, after a contact granted in sticky
status: open
related: [S-0501]
answered-by: []
---

## Need

S-0501 r167 claim 29 (*A contact held at cruise height*) has a pad-bound aircraft in *sticky*
that holds a pad's contact with *enter* queued run claim 18's in-flight pad row at each processing
at exactly its cruise height. Claim 21's search there asks first the pad the aircraft was last
heading for. The spec leaves open whether the pad that granted the contact in *sticky* is recorded
as that pad (claim 29, Open question 32: "Not traced: whether the held pad is recorded as that pad
when the contact was granted in *sticky*"). Claim 24 states one way the record is made (the
pad-bound aircraft on the ground with a destination other than the building in its own cell), and
no claim states whether a granted contact makes it.

Our engine records the pad as the pad last heading for at every contact a pad grants
(`air::request_contact`, `crates/deterministic/sim/src/air.rs`). That covers both contacts claim 29
gives in *sticky*: the one claim 24's contact rule makes when a pad is found below cruise height,
and the one the in-flight pad row's own request makes at cruise height.

## Questions

1. A pad-bound aircraft in *sticky* is granted a pad's contact by the in-flight pad row's own
   request (claim 18) at a processing at its cruise height. At its next such processing a second
   pad of its dock types, nearer to it and free, also accepts it. Does the search return the pad it
   holds or the nearer one?
2. The same when the contact was made by claim 24's contact rule, after a pad found below cruise
   height (claim 29, *A pad found below cruise height*).
3. Does a contact granted outside *sticky* (the in-flight pad row in *guard*, or claim 24's contact
   rule in *enter*) record the pad as the pad last heading for?

## Possible observation

AC33 part c with a second pad of *A*'s dock type, with a free place, built nearer to *A*'s path
than *P* after *A* has been granted *P*'s contact; record which pad *A* holds at each later
processing at its cruise height. For question 2, the same with *A* below its cruise height and one
round spent (if a means is found to put such an aircraft in *sticky*).

## Current workaround

`air::request_contact` records the pad as the pad last heading for at every granted contact, as
before r167, with a `// design: Q-2866` cite. With one pad of the aircraft's dock types, as in every
AC33 part, both readings give the same pad.
