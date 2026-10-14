---
id: Q-0281
title: Which countries, how many player slots and which team numbers does the setup screen offer?
status: open
related: [S-0412, S-0103]
answered-by: []
---

## Need

S-0412 gives the setup values of a skirmish (players, countries, colours, teams, start positions)
and how random choices resolve. The setup screen must also know which countries it lists, how
many player rows it shows and which team numbers it offers. S-0412's Terms speaks of "a small set
of team numbers" without a count.

## Questions

1. Which countries does the setup screen list, and in which order? Are passive countries, the
   neutral and special houses, or countries of a civilian side left out?
2. How many player rows does the screen show at most? Does it depend on the map's start
   positions?
3. Which team numbers can a player pick, and is "no team" one of them?

## Possible observation

Open the skirmish setup with rules that add passive and civilian countries, and with maps of 2,
4 and 8 start positions; record the lists offered.

## Current workaround

Interim rule (`glasnost_ui::setup`): the countries offered are the non-passive countries other
than the neutral and special houses, in registration order. At most eight rows (one per colour
slot of S-0412 claim 7), never more than the map's start positions. Teams 1 ... 4 and "no team".
Recorded in docs/development.md, "Game client (M5b)".
