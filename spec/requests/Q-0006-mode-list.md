---
id: Q-0006
title: How the mode list names each game mode and its game-mode file
status: answered
related: [S-0005, S-0006]
answered-by: [S-0006]
---

## Need
The game-mode layer of S-0006 claim 1 is the file that the mode list
(`mpmodesmd.ini`, S-0005 Import mapping) names for the selected mode: "the
third field of the mode's entry" (S-0006 Import mapping). To list the modes
and find each one's file, the importer must know where the entries are and
how a field is delimited. Code gap: game-mode selection in
`glasnost_convert` and the viewer's import panel.

## Questions
1. Which section or sections of the mode list hold the mode entries, and does
   each entry's key play a part (for example as the mode's identifier or its
   position in the lobby list)?
2. How is an entry's value split into fields (separator, trimming, text
   limits), and which field is the game-mode file's name? Is the name used
   as written, or completed (for example with an extension)?
3. Which value of that field means "no file" (S-0006 claim 1: "empty when the
   mode names no file")?
4. Are several entries with one key, or entries naming the same file,
   handled specially?
5. Which mode does a skirmish use when the player makes no choice (the
   default mode)?

## Possible observation
A loose copy of the mode list with one entry's file field pointing at a
loose test file that raises one unit's cost (the pattern of S-0005
scenario M2 (b)).

## Current workaround
The importer applies a game-mode layer only when the user names the
game-mode file directly; it never reads the mode list. The reference ruleset
is converted with an empty game-mode layer (which S-0006 claim 1 allows:
"empty when the mode names no file").
