---
id: Q-1832
title: A music track whose sound file is present but does not decode, or whose name ends in .wav
status: open
related: [S-0703, Q-1700]
answered-by: []
---

## Need
S-0703 claim 8 says what happens to a track whose sound file is missing (never available) or can
no longer be opened when it is due to start (silence, a new pick at the next music update). It
leaves open a file that is found but does not decode (Open questions, Q-1700 question 8), and a
sound file name that already ends in the extension (claim 1a, Q-1700 question 3). The client must
decide what a pick of such an entry does.

Code: `crates/audio/src/music.rs` (`Music::update`, `Music::loaded`) and
`crates/audio/src/music_files.rs` (`VfsTracks::find`).

## Questions
1. A track entry whose sound file is found but is not a sound file the game can play (a damaged
   header, an unknown compression, a truncated body): is it available, so that the next-track rule
   can pick it? When picked, is there silence for some length, silence until the next music update,
   or does the game fail?
2. A file that decodes part of the way and then stops (a truncated body): does the track end where
   the data ends, or at the length its header states?
3. An entry whose sound file name already ends in `.wav`: is the file looked for with the extension
   twice, or once?

## Possible observation
S-0703 MU1 part a's set-up with one test track replaced by a file of the same name holding text,
then by a copy of a test track cut off half way; then with another entry's sound file name written
with the extension, its test track named once with the extension and once with it twice.

## Current workaround
A found file counts as present (claim 8). A picked entry whose file does not parse as a sound file
starts like a file that cannot be opened: silence, the entry becomes the ended track (claim 4) and
the next pick is made at the next music update. A file whose data stops early ends where its data
ends: its logical duration is counted from the frames its data holds. A sound file name that already
ends in `.wav` gets the extension added again, as claim 1a states the rule. Marked
`// Q-1832 (interim: ...)`.
