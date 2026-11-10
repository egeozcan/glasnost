---
id: Q-1700
title: How the game picks, plays and changes music tracks
status: answered
related: [S-0005, S-0701]
answered-by: [S-0703]
---

## Need

M8c adds music (PLAN.md, D10). The client can already find the music database and the theme archive
(S-0005 claims 10 and 12 and its file table) and can decode the archive's sound files (the ADPCM decoder
of `glasnost-fmt` that the sound effects use). No approved spec says which track plays when, so
`glasnost-audio` plays no music.

## Questions

1. **Start of a skirmish.** When does the first track start (at the loading screen, at the first game
   tick, after a delay)? Which track is it: does the player's side or country choose it, is it drawn at
   random, or is it the first eligible entry in the database's order?
2. **Next track.** When a track ends, which plays next: the database order, a random pick (can the same
   track play twice in a row?), or the same track again? Is there a gap between tracks?
3. **Eligibility.** Which database entries can play in a skirmish? Which entry fields decide it, and
   what happens to an entry whose field is missing or malformed?
4. **Player controls.** Which music choices does the in-game options screen offer (a track list, repeat,
   shuffle, a volume level)? What does each choice do to the order of questions 1 and 2, and do the
   choices last past the end of the game?
5. **Volume.** How does the music volume setting scale the output (steps, the top and bottom of the
   range)? Is the music quieter while an announcement or a sound effect plays?
6. **Game events.** Does the music change on any event in play (victory, defeat, the score screen, a
   superweapon, the game paused or minimised)?
7. **Menus.** Which track plays on the main menu and on the skirmish set-up screen, and does the menu
   track carry on into the game or stop?
8. **Missing data.** A database entry whose sound file is not in the theme archive, or does not decode:
   is it skipped, or is there silence for its length? With no playable entry at all, is there silence?

## Possible observation

Start several skirmishes with each side and note the first track and the following three. Try the
options screen's controls in a game. Then repeat with one entry's sound file removed from a copy of the
archive, or an entry's fields changed in a copy of the database.

## Current workaround

None: no music plays. The client builds only what the specs give: finding the database and archive,
and decoding the sound files.
