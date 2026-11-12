---
id: Q-3012
title: S-0301 claim 9 rank 12, a tethered vehicle healer: impossible or illegal?
status: open
related: [S-0301, S-0310]
---

## Need

S-0301 claim 9's rank 12 has two reasons: "a vehicle that is tethered (glossary) with a building
as its contact gets *impossible*, and a vehicle whose default weapon is a healing weapon gets
*illegal* (S-0310, claim 11; S-0314). Which of the two wins when both hold is not stated; no
scenario here distinguishes that overlap." GitHub issue #176 found the tether reason unbuilt; we
now build it. For a vehicle healer that is tethered to a building (released from a factory,
docked at a depot) and whose target fails S-0310 claim 11's test, one of the two answers has to
come first.

## Questions

1. When both reasons of rank 12 hold, does the firing check answer *impossible* (the tether) or
   *illegal* (the healer)?
2. Does the answer change for a caller that asks for illegal reasons only?

## Possible observation

A vehicle healer released from a vehicle factory while a full-health allied vehicle stands in
range, ordered to heal it during the release. *Illegal* and *impossible* lead to different
behaviour for a held target (S-0309 claim 29's periodic check drops an illegal one), so the
target recorder would separate them.

## Current workaround

The healer's *illegal* keeps its place before the tether, as it stood before the tether was
built; the code cites this request.
