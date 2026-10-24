---
id: Q-1833
title: What music does when a skirmish ends or is left
status: answered
related: [S-0703, S-0701, Q-1700]
answered-by: [S-0703]
---

## Need
S-0701 claim 30 runs skirmish audio passes only in a loaded game, and S-0703 claim 10 runs the
music update at the end of each pass. What the music does at the end of play (victory, defeat, the
score screen) and when the person leaves the game is open (S-0703 Open questions, Q-1700 questions
6 and 7). The client switches to its end screen as soon as play ends.

Code: `crates/app/src/game/client.rs` (`Client::update`, which services the audio only on the game
screen and stops the music when play ends or the game is left) and `crates/audio/src/events.rs`
(`Audio::start`, `Audio::stop_music`).

## Questions
1. When a skirmish is won or lost, does the playing track go on, stop, or give way to another
   track? Does the next-track rule of S-0703 go on choosing tracks on the score screen?
2. When the person leaves a skirmish from the options menu, does the track stop at once or play on
   into the menus?
3. While the options menu is open during a skirmish, does the playing track go on, and does a track
   that finishes then lead to a new pick?

## Possible observation
S-0703 MU1 part a's set-up: win a short skirmish while a test track plays and record the audio
through the score screen; then leave a second skirmish from the options menu while a track plays;
then open the options menu just before a track ends and wait there for one track length.

## Current workaround
Music is heard only while the game is played: when play ends or the person leaves, the client
stops the music and no pick is made; the next game starts afresh (Q-1830). With the options menu
open the audio is serviced as for sound, so the track plays on and picks go on. On our own
settings screen opened from the game the audio is not serviced, as for sound: the playing track
sounds to its end and the next pick waits for the return to the game. Marked `// Q-1833 (interim: ...)`.
