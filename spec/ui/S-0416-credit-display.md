---
id: S-0416
title: Credit display (the ticking credits counter on the sidebar)
topic: ui
revision: 3
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-17
measurements: []
requests: []
supersedes: []
---

# S-0416: Credit display (the ticking credits counter on the sidebar)

## Summary

The credits counter on the local player's sidebar: what value it aims at, how it moves toward that
value a step at a time instead of jumping, how large each step is, when it moves, and the tick sounds
that go with it. It is presentation only: no game rule reads the shown value. What the house can
spend is S-0404's (available credits) and the economy specs'. Skirmish only; the observer's clock that
replaces the counter in network games is out of scope.

## Terms

Glossary terms are used as defined there (*sidebar*, *local player*, *credits*, *cycle*, *drawn
frame*, *tick*, *house*). From S-0404: *available credits* *A*. From S-0403: *R* (used in
scenario D2 only). In addition:

- **Target** *G*. The value the counter aims at: the local player's available credits, or 0 when
  they are 0 or less. Not the glossary's *target* (what an actor attacks, captures or enters).
- **Shown value** *D*. The number the counter displays, an integer in credits.
- **Credit tick sounds.** The game-wide list of sounds for the counter (Import mapping); its first
  entry plays when the counter rises and its second when it falls.

## Behaviour

1. **What it aims at.** The target is the local player's available credits (S-0404, claim 4) at the
   moment of the counter's update, with any value of 0 or less shown as 0. The counter is drawn as the
   shown value in decimal.
   - Backing: `measurable` (scenario D1)
   - Confidence: high

2. **It moves at most one step per cycle.** Once in every cycle that is not frozen, when the shown value
   differs from the target, the shown value moves toward the target by

   > *s* = min(143, max(1, ⌊|*G* − *D*| / 8⌋))

   credits, never past it (a step of at least 1 and at most 143, an eighth of the remaining gap
   rounded down). So a gap of 8 or less closes by 1 credit per cycle, a gap of 1144 or more closes by
   143 per cycle, and in between the counter slows as it nears the target. A change of the target
   while the counter moves simply changes the remaining gap. Since a single-player game makes one
   drawn frame per cycle, the counter changes at most once per drawn frame. Whether a frozen cycle
   (glossary, *cycle*) also moves the counter was not established.
   - Backing: `measurable` (scenario D2)
   - Confidence: high (the step rule); low (frozen cycles)

3. **It starts at 0.** At game start the shown value is 0, so the counter counts up to the starting
   credits over the first cycles by claim 2.
   - Backing: `measurable` (scenario D3)
   - Confidence: medium

4. **Tick sounds.** In every cycle in which the counter moves, one credit tick sound is requested:
   the list's first entry when rising, its second when falling. When the list has fewer than
   two entries, no tick sound is requested at all. This is a centred screen request at integer
   request level 8192 on S-0701's scale, whose full level is 16384 (claim 35). The original uses
   the exactly representable floating-point fraction 0.5: trunc(0.5 × 16384) = 8192, with
   truncation toward zero. This halves the request level, not the audible amplitude: under
   otherwise equal conditions it is about 10 dB below the same entry requested at full level.
   S-0701 F3 and F4 own the combined level, integer rounding and output law; its playback limits
   can prevent a request from being heard.
   - Backing: `measurable` (scenario D4)
   - Confidence: medium

5. **Presentation only.** No game rule reads the shown value: spending, refunds and every check of
   money use the house's credits (S-0404). The shown value can therefore lag behind what the house
   has; a build that the counter seems unable to afford may still be paid. In our engine the counter
   is drawn for the local player only and never reaches game state.
   - Backing: `measurable` (scenario D5, for the consequence that an item is paid for while the
     counter lags; that no other money rule reads the shown value would show as a dependence in
     any measurement of money in the economy specs' scenarios)
   - Confidence: high

## Formulas

> *G* = max(0, *A*), *A* the local player's available credits (S-0404, F3).
>
> When *D* ≠ *G*: *s* = min(143, max(1, ⌊|*G* − *D*| / 8⌋)), and *D* becomes *D* + *s* when *G* > *D*,
> *D* − *s* when *G* < *D*. Integer arithmetic; ⌊ ⌋ is the floor of a non-negative number, the same as
> rounding toward zero there.

The step never overshoots: for a gap of *d* ≥ 1, *s* ≤ *d*.

Example: a refund of 1000 credits moves the counter by 125, 109, 95, 83, … per cycle, and the last
8 credits by 1 per cycle.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Gap divisor | 8 | dimensionless | claim 2 |
| Largest step | 143 | credits per cycle | claim 2 |
| Smallest step | 1 | credits per cycle | claim 2 |
| Tick sound request level | 8192 | level units (16384 = full request) | claim 4; S-0701 claim 35 |
| Shown value at game start | 0 | credits | claim 3 |

## State and ordering

| State | Event (once in every cycle that is not frozen; frozen cycles: open question 1) | Result |
|---|---|---|
| *D* = *G* | update | nothing |
| *D* < *G* | update | *D* increases by *s*; when the credit tick sound list has at least two entries, the first entry is requested |
| *D* > *G* | update | *D* decreases by *s*; when the credit tick sound list has at least two entries, the second entry is requested |

## Edge cases and quirks

- **Negative available credits** show as 0 (claim 1). Whether available credits can be negative at
  all is the economy specs' question.
- **Slow convergence near the target.** The last 8 credits of any change always take 8 cycles; a
  change of exactly 1 credit takes one cycle. This is the original's design; no switch.

## Import mapping

| Key (file, section) | Concept | Form | Absent |
|---|---|---|---|
| `CreditTicks` (rules, audio-visual section) | credit tick sounds: first entry rising, second falling | list of sound names | empty (no tick sounds) |

## Measurement scenarios

Measurement setup (glossary); a small purpose-made skirmish map; one human house; screen capture of
the counter at every drawn frame (one per cycle in a single-player game); audio capture for D4.
Runs: 1 per scenario and sub-test, since no claim here involves a random draw.

**D1. Target and the zero floor.**
- Setup: starting credits set by the lobby; a test trigger that sets the house's credits to known
  values (for example by a cash-giving trigger action).
- Observable: the settled shown value.
- Expected: it equals the target *G*: the house's available credits, or 0 when they are 0 or less.
  The counter never shows a negative number.
- Falsified if: the settled value differs from *G*.
- What it cannot show: this setup does not make the available credits negative, so the floor below
  0 is not exercised (see Edge cases and quirks).

**D2. Step rule.**
- Setup: settled counter; then a single change of known size *d* (a cancel refund of a known amount,
  S-0404 claim 6, or a trigger gift): *d* = 5, 100, 1000 and 5000. For a fall: an item with *R* = 1
  (S-0403) and a known price, whose payments (S-0404, F2) give a known target in every tick.
- Observable: the shown value in every drawn frame until settled.
- Expected: the sequence of claim 2 against the target of each frame (for a rise of 1000: 125, 109,
  95, 83, …; for 5000: 143 per frame until the gap is below 1144).
- Falsified if: any step differs from the formula, or the counter overshoots.

**D3. Start of game.**
- Setup: a skirmish with starting credits 10000.
- Observable: the shown value in the first drawn frames.
- Expected: it counts up from 0 by 143 per frame at first, as claim 2 gives.
- Falsified if: the counter shows the full amount in the first frame.

**D4. Tick sounds.**
- Setup: stock audio data, then a named test rules file whose credit tick list has one entry.
  For the level comparison, use named test sound data assigning the same single sample to a
  credit tick entry and a selection response entry, each with volume percentage 100 and no
  pitch or volume shift. Set the sound volume option to full, game speed to index 6, and keep
  the observed area quiet; use one bystander computer player kept outside it. No engine add-on and
  no mods (the named test files of a scenario and the purpose-made test maps excepted).
- Procedure: make the counter rise and fall in each list test. In the comparison test, capture
  isolated ticks and a local unit's selection response separately, without overlapping sounds.
- Observable: audio capture of the rising and falling sounds and the same sample's output level
  in the tick and response, alongside frame capture of the counter.
- Expected: stock data uses the first entry on a rise and the second on a fall, subject to
  S-0701's playback limits; the one-entry list is silent. The comparison gives ticks about
  10 dB below the full-level response, with the exact difference given by S-0701 F3/F4
  (including its full-volume quirk).
- Falsified if: the one-entry list plays a sound, the wrong directional entry is heard, or the
  isolated comparison differs from the output-law prediction by more than one F4 step.

**D5. Presentation only.**
- Setup: two runs of the same skirmish, with no change of the house's credits other than those
  named here. In both, the counter is settled, then a trigger gift of a known large amount (for
  example 5000) raises the available credits. Run A orders an item in the cycle after the gift,
  while the shown value is still below the item's price and the price is at most the available
  credits. Run B orders the same item at the same available credits after the counter has settled.
- Observable: the item's production and payments, and the available credits, in every tick from
  the order on.
- Expected: the same in both runs, counted from the order.
- Falsified if: the runs differ, for example run A's production waits until the shown value reaches
  the price.

## Open questions

1. Whether frozen cycles (timed freezes, S-0201) move the counter (claim 2).
2. Whether loading a saved game starts the counter at 0 is outside skirmish play and not studied.
3. In which part of a cycle the target is taken: from the available credits before or after that
   cycle's tick has changed them (claim 1). The answer shifts the shown sequence by one cycle when
   the credits change every tick, as in the fall of scenario D2.
