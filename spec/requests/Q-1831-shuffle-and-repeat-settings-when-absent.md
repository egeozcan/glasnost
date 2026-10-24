---
id: Q-1831
title: The shuffle and repeat settings when the settings file does not give them
status: answered
related: [S-0703, S-0005, Q-1700]
answered-by: [S-0703]
---

## Need
S-0703 reads the shuffle and repeat settings from the settings file's `[Audio]` section
(`IsScoreShuffle`, `IsScoreRepeat`, Import mapping) but leaves their values open when the keys are
absent (Open questions, Q-1700 question 4). S-0005 claim 5 says a missing settings file means
defaults. The client must play with some value of each.

Code: `crates/audio/src/music_files.rs` (`InstallMusic::read`, with `Vfs::loose_only` in
`crates/vfs/src/game.rs`) and `crates/audio/src/music.rs` (`MusicSettings`).

## Questions
1. With no settings file, or a settings file without `[Audio]` `IsScoreShuffle`, is the next track
   chosen in list order or by shuffle?
2. Likewise for `IsScoreRepeat`: with the key absent, does the ended track play again?
3. Does a malformed value (one that S-0003's boolean form does not read) keep the absent default?

## Possible observation
S-0703 MU1 part a's set-up, once with the settings file removed from the game directory and once
with the two keys deleted from it: a fixed cycle in list order means both off, a cycle with no fixed
order means shuffle on, the same track over and over means repeat on.

## Current workaround
Both settings are off when the settings file or the key is missing, or when the value is not a
boolean that S-0003 reads. The client reads `ra2md.ini` only as a loose file (S-0005 claim 5),
when the install is read. Our engine offers no in-game control for the two settings (Q-1700
question 4 is open). Marked `// Q-1831 (interim: ...)`.
