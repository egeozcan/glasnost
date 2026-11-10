---
id: Q-1830
title: How music starts in a skirmish, and what the first pick is relative to
status: answered
related: [S-0703, S-0701, Q-1700]
answered-by: [S-0703]
---

## Need
M8 step C1 plays music in a skirmish by S-0703: the music update of claim 10 picks the next track
by claims 4 to 7 each time the playing track finishes, but only once *music started* holds (Terms).
What makes music start is open (S-0703 Open questions, Q-1700 question 1), and without a start no
track ever plays, so the client must choose something.

Code: `crates/audio/src/music.rs` (`Music::start_game`, `Music::update`) and
`crates/audio/src/events.rs` (`Audio::start`, `Audio::serve`).

## Questions
1. In a skirmish, at what moment does the first track start (while loading, at the first tick of
   play, after some delay)? Does music start when no entry is available (S-0703 MU4 part a)?
2. Which track is it? Is it picked by the same next-track rule as later tracks (list order from
   entry 0, or a shuffle draw), or chosen another way (by the local player's side, a fixed entry,
   a map or mode setting)?
3. Does that start leave an *ended track* (S-0703 claim 4) that the next pick is made relative to,
   and does a new game forget the ended track of the game before it?

## Possible observation
S-0703 MU1 part a's set-up with the shuffle and repeat settings off: record from the end of the
loading screen and note how long after play begins the first tone is heard and which entry it is;
start a second skirmish in the same session and note its first tone. Repeat with entry 0's normal
flag clear.

## Current workaround
Music is started when a skirmish's audio starts (`Audio::start`); the first music update, at the
first audio pass of the game, makes an ordinary pick by claims 4 to 7 with no ended track (list
order from entry 0, or a shuffle draw on the unsynchronised stream; claim 7's fallback when nothing
is picked). Every game start forgets the ended track. Marked `// Q-1830 (interim: ...)`.
