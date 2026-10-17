---
id: Q-1202
title: The absent-key values of AICaptureLowMoneyMark and AICaptureWoundedMark
status: open
related: [S-0601]
answered-by: []
---

## Need
S-0601's Import mapping gives "not determined" as the absent-key value of
`AICaptureLowMoneyMark` and `AICaptureWoundedMark`. Claim 8's fate decision
(run for the neutral and special houses in our engine) compares the
controlling house's money with the first and the victim's health ratio with
the second. Rules without these keys need a defined behaviour.

## Questions
1. With `AICaptureLowMoneyMark` absent from every rules layer, does a
   computer house with no money use the low-money table after a takeover?
2. With `AICaptureWoundedMark` absent, does a badly damaged victim taken by
   a computer house use the wounded table?
3. What mark values reproduce the behaviour of the absent keys?

## Possible observation
On a test copy of the rules with each key removed in turn, let a neutral or
computer house's controller take a damaged unit while that house has no
money, with fate tables that make each table's choice visible (for example
only the hunt weight set in one table), and record what the unit does.

## Current workaround
An absent mark never selects its table: the decision falls through to the
next table in the order of claim 8 (`mind::fate_table`).
