---
id: Q-1735
title: The text that goes with a spoken announcement
status: open
related: [S-0701]
answered-by: []
---

## Need
M8 step C3 adds captions, an accessibility setting of ours: while it is on, every announcement
that starts playing (S-0701 claims 26 to 28) also shows a line of text over the battlefield for a
few seconds, so a person who cannot hear it still learns what was said. The client needs, for each
announcement entry, the text to show.

S-0701 claim 26 gives each announcement entry a name, a kind, a priority and one sample per voice
set, but no text. No approved spec says whether the game's data carries a written form of an
announcement (for example an entry of the game's string data tied to the announcement by name or
by a key of its section), or whether the game itself ever shows announcement text.

Code: `crates/app/src/game/captions.rs` (`Captions::push`, the caption list) and
`crates/app/src/game/client.rs` (`Client::update`, which pushes a caption for each announcement
`glasnost_audio::events::Audio::take_started` reports).

## Questions
1. Does the game's data give a written text for an announcement entry? If it does, which file and
   which key of the announcement's section (or which naming rule) leads from the entry to the text,
   and what happens when that text is missing?
2. Does the game ever show the text of an announcement on screen (in a skirmish, with any option),
   and if so where, for how long, and how several announcements in a row are shown?
3. Does the text depend on the voice set (S-0701 claim 26) of the local player's side?

## Possible observation
A skirmish with production of a building: when the "construction complete" announcement of S-0701
claim 28 plays, watch the screen for any text; then look in the announcement data section of that
entry and in the game's string data for an entry that carries the same words.

## Current workaround
Our own setting (default on, stored with the other settings, never in the simulation): the caption
of an announcement is the name of its entry in the person's announcement data, as read (S-0701
claim 26), shown for 4 seconds, at most 3 at once, newest last. The code marks this with
`// Q-1735 (interim: ...)`.
