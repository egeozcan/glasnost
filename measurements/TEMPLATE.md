---
id: M-0000
title: Short description of what was measured
topic: simulation               # same topics as spec/
specs: []                       # S-IDs this confirms or contradicts
requests: []                    # Q-IDs this answers
verdict: confirms               # confirms | contradicts | new-finding
value-visibility: committed     # committed | local (value kept in measurements/local/M-0000.json)
measured-on: YYYY-MM-DD
method: manual                  # manual | harness | screen-capture
---

# M-0000: Short description

## Setup

- Game build: stock, executable hash verified.
- Data: stock, no mods.
- Settings: game speed, map (our own micro-map or a stock map by name/content hash), players, factions, starting credits, any other relevant option.
- Harness: manual, or the name of the scripted scenario. AI changes and gameplay mods off.

## Procedure

What was done and what was recorded, step by step, so it can be repeated.

## Result

| Quantity | Unit | n | Mean / value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| | ticks / leptons / credits / % | | | | |

If `value-visibility: local`, leave the value columns as `local` and state the **relation** instead:

> Observed reload interval equals the imported weapon's reload value, ±1 tick (n = 20).

## Evidence

Recording or log hash (the files themselves stay local), number of runs, anything that went wrong.

## Notes

Surprises, follow-up measurements needed.
