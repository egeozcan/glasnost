---
id: S-0000
title: Short behavioural title
topic: combat              # see spec/README.md for topics
revision: 1
sources-class: []          # any of: decomp, measurement, modenc, openra, xcc, format-doc, design
status: draft              # draft | scrubbed | approved | retired
approved-by:               # filled in at sign-off
approved-on:               # YYYY-MM-DD
measurements: []           # M-IDs backing this spec
requests: []               # Q-IDs this spec answers
supersedes: []             # S-IDs this spec replaces, if any
---

# S-0000: Short behavioural title

## Summary

One or two sentences: what behaviour this spec covers, in our own vocabulary.

## Terms

Terms used below that are not yet in `spec/glossary.md`.

## Behaviour

Numbered claims, each observable and testable. Every claim states its backing and confidence.

1. **Claim.** What happens, when, to what.
   - Backing: `measured` (M-xxxx) | `measurable` (see Measurement scenarios) | `unmeasured`
   - Confidence: high | medium | low

## Formulas

Mathematical notation with units and the rounding rule for each operation.

> value = ⌊a × b / 100⌋, truncated toward zero after the multiplication. Units: leptons.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| | | ticks / leptons / per-mille / credits | Engine default only. Stock rules values are never written here; refer to the imported value by role. |

## State and ordering

State tables (state, event, result) and ordering stated as observable consequences.

| State | Event | Result |
|---|---|---|
| | | |

## Edge cases and quirks

Unusual or buggy behaviour of the original, described as behaviour. Name a quirk switch for each one that a mod might want to turn off; it defaults to the original behaviour. Exception: an obvious bug with no gameplay impact (a display glitch, a drawing artefact, a view that jumps) gets no switch; state the fixed behaviour as the claim and record the original's bug in one sentence.

## Import mapping

Only if relevant: which keys in the user's rules or art files feed which fields of our format, and how absent keys default. Rules-file key names are allowed here and only here.

## Measurement scenarios

For each `measurable` claim: map or setup, game settings, what to record, how many runs, and the expected result.

## Open questions

Anything still uncertain, with the request IDs that track it.
