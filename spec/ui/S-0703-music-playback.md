---
id: S-0703
title: Music playback (track eligibility, next-track order, shuffle and repeat)
topic: ui
revision: 58
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-25
measurements: []
requests: [Q-1700, Q-1830, Q-1831, Q-1832, Q-1833]
supersedes: []
---

# S-0703: Music playback (track eligibility, next-track order, shuffle and repeat)

## Summary

How music starts in a skirmish, which entries of the music database can play there, and which track
follows when one ends: list order, shuffle, repeat, the fallback when nothing can play, missing
sound files and sound files that do not play, and the shuffle and repeat settings when the settings
file does not give them. It also gives what the music does at the end of the game, through the end
delay and once the game is over, on the score screen, while the options menu is open and when the
game is left from it. This answers Q-1831, Q-1832 and Q-1833, and is a partial answer to Q-1830 (the
length of the delay before the first track, and whether a game mode's own data can give a map theme,
are open) and to Q-1700. The options screen's music controls, volume scaling, other game events,
music in the other menus and the gap between tracks are open (Open questions). Music is
presentation: no game state depends on it.

## Terms

- **music database.** As in S-0005: the music file the expansion reads at startup (S-0005 claims
  17 and 18).
- **track list.** The ordered list of track entries the music database names. Its order is the
  order of the list's values (Import mapping).
- **track entry.** One item of the track list, named by its identifier. Its own section gives
  the two flags that decide when it plays (*normal* and *repeat*), the side restriction that may
  limit it and the sound file it plays. It also gives a campaign scenario number, which plays no
  part in a skirmish (claim 2), and a display name; what either does outside a skirmish is not
  established here (Open questions, Import mapping). Entry *k* is the item at zero-based position *k*; *n* is the number
  of entries.
- **file present.** A track entry whose sound file is found among the mounted archives and loose
  files (claim 8). When presence is decided (at startup or later) is claim 8's.
- **available track.** In a skirmish, a track entry that is file present, has its *normal* flag
  set and whose side restriction passes claim 2's side-restriction rule.
- **ended track.** The track entry most recently started, by a pick of claims 4 to 7, by the map
  theme's direct start (claim 11) or by a direct start of claim 12; the next pick is made
  relative to it. A skirmish starts with no
  ended track, and the first start after its loading is made with none (claim 11).
- **failed start.** A start of a track entry whose sound file cannot be opened, or is opened but
  does not parse as a playable sound (claim 8a). Nothing is heard; what it leaves is claim 4's.
- **loading track.** The track entry, if the music database has one, whose identifier is the
  fixed name given in Import mapping. It is started while a game loads (claim 11).
- **map theme.** The track a map names for the start of its game (Import mapping), if that name
  matches a track entry's identifier (claim 11).
- **in-game music setting.** A player choice, read from the settings file (Import mapping), that
  decides whether music plays during a game at all (claim 11).
- **score track.** The track entry, if the music database has one, whose identifier is the fixed
  name given in Import mapping. It is started when the score screen opens (claim 12).
- **menu track.** The track entry, if the music database has one, whose identifier is the other
  fixed name given in Import mapping. It is started when the main menu opens (claim 12).
- **end of the game, end delay, game over, score screen.** These follow S-0414. The *end of the
  game* is the moment its end test first succeeds (S-0414 claim 6). Play then goes on through the
  *end delay*, whose ticks are ordinary play, and the *game is over* at the end of its final tick
  (S-0414 claim 8). The *score screen* is S-0414's result screen as the original shows it after a
  skirmish is over, before the main menu; our engine's counterpart is its end screen.
- **music update.** The periodic check at which, once the playing track has finished, the next
  track is picked and started. When it runs and when a track has finished are claim 10's.
- **music started.** The state in which music updates make picks. It begins with the first start
  after loading (claims 10 and 11), not with the loading track's start.
- **shuffle setting, repeat setting.** The two player choices that change the next-track rule
  (claims 4 to 6). Both are read from the settings file, and are off when it does not give them
  (Import mapping); no scenario here separates a read at startup from a later one. Which in-game controls change
  them, and whether a change is saved, are open (Q-1700 question 4).

## Behaviour

1. **Track list.** The track list is read once, at startup, from the music database (S-0005
   claim 17). An identifier listed twice gives one track entry, at the position of its first
   listing. List order is the order claim 5 follows.
   - Backing: `measurable` (MU1 part c) for the repeated identifier and the order. That the list
     is read once, at startup, is S-0005 claim 17's rule for the databases, with its backing.
   - Confidence: high for repeats of the same text and the order. Low for repeats that differ
     only in case (`unmeasured`, Import mapping).

1a. **Sound file name.** The file looked for is the entry's sound file name with any leading `$`
   and `#` characters removed and the extension `.wav` added, among the mounted archives and loose
   files (claim 8). The extension is added whatever the name ends with, so a sound file name that
   already ends in `.wav` gets it a second time: the file looked for is that name followed by
   `.wav` again, matched ignoring case like every file name (S-0005 claims 2 and 6). An entry whose sound
   file name is empty or absent never has its file present, and neither has one made only of `$`
   and `#` characters, since removing them leaves the name empty, provided that no file named by
   the extension alone is mounted. With such a file mounted, the outcome is not established.
   - Backing: `measurable` (MU1 part e for the removed characters, with prefixes of one, two and
     more characters in mixed orders, and its last run for the same characters elsewhere in the
     name being kept; for the extension, the test tracks of every scenario, each
     a file whose name ends in the extension while its entry's name does not; MU1 part g for an
     empty or absent name and, in its third run, a name made
     only of those characters, while no file named by the extension alone is supplied; MU1 part m
     for a name that already ends in the extension).
     `unmeasured` for such a name when a file named by the extension alone exists, which no
     scenario supplies.
   - Confidence: high, except medium for the extension added a second time, until MU1 part m
     is made, and medium for an empty or absent name and a name made only of those
     characters, which in the original rest on a file named by the extension alone never being
     found.

2. **Availability in a skirmish.** Claims 5 and 6 pick only available tracks. A track entry is an
   available track while its sound file is present (claim 8), its *normal* flag is set, and its
   side restriction is "any side" or matches the side of the local player's country (how a
   restriction is matched is given below). The campaign
   scenario number plays no part in a skirmish. Some rules can still start a track that is not
   available: repeat (claim 4), the fallback (claim 7), the loading track and the map theme
   (claim 11), and the score track and the menu track (claim 12). S-0701 claim 30 schedules skirmish
   audio passes only in a loaded game, and music in the other menus is outside this spec (Open questions), so
   this rule covers skirmish playback only; once the game is over claim 12 applies. Our engine matches a side
   restriction to a side by name, ignoring case. In the original, a restriction written as the
   name of the local player's side, when that side is not the first (index 0, S-0004 claim 9),
   does not by that make its entry available (MU1 part b): a restriction meant for a side can
   fail to play for that side, which changes music only. For the first side the original's
   result is not established (MU1 part b only records it).
   Which entries the original makes available instead is not established (Open questions), and
   our engine does not reproduce it, so it gets no quirk switch; see Edge cases. A
   restriction that names no side of the game, as S-0004 claim 1 builds the sides for that game
   (sides that a mode or map layer adds included), is bad data: in our engine its entry is never
   available in that game (Edge cases). A game whose
   local player's country has no side (S-0004 claim 9) never reaches play (S-0005 claim 13a), so
   no matching rule is needed for such a country.
   - Backing: `measurable` (MU1 part a) for the file and *normal* parts. `measurable` (MU1
     part d) for the scenario part, with a scenario number higher than every campaign
     mission's, a mission's number and a negative number.
     `measurable` for a restriction written as the name of a side other than the first not
     matching in the original (MU1 part b); the first side's result is only recorded and is not
     claimed. `measurable` for our engine's name matching (MU1 part b's engine fixture).
   - Confidence: high for the *normal* part and for the scenario part; medium for our engine's
     name matching, a design rule that only our engine's fixture checks, with no oracle run behind
     it; medium for the file part (only tracks whose files are present are picked), as claims 3
     and 8 rate the pass-over, until MU1 part l runs; medium for a restriction written as the name
     of a side other than the first not matching in the original.

3. **When the next track is picked.** Once music has started (claim 11), each time the playing track finishes the next track is picked at the next
   music update and started at once. In a pick under claim 5 or claim 6, an entry that is not an available
   track is passed over at the pick itself: no silence of its length is inserted, and the pick goes on to the next candidate.
   Under list order this is the rule our engine follows; whether the original passes over at the
   pick or starts, fails and picks again is open (Open questions), and MU1 part l tells them apart
   once MU4 part b's third run has shown, by its draw count and its recovery, that a failed start
   makes its entry the ended track.
   In the original, no pick is made while the music volume is zero, and no track is started at
   zero volume; which in-game setting gives a music volume of zero is open (Q-1700 question 5),
   and this part can be observed only once that is known. Our engine departs from this (no
   switch, Edge cases): its music update picks and starts tracks at every music volume, and a
   track started at zero volume plays silently, its logical duration counting as usual (claim
   10). So the music volume, a setting of each player, never adds or removes a ranged draw on the
   unsynchronised stream, as S-0701 claim 30 rules for sound instances under mute settings.
   - Backing: `measurable` (MU1 part a) for the passing over without a silent gap of a track's
     length; `measurable` (MU6 part a) for the pass-over at the pick under shuffle, told apart
     by the draws the pick makes. `measurable` (MU1 part l, read after MU4 part b's third run) for
     the pass-over at the pick under list order: MU1 part a cannot tell it from a start attempt that fails followed by a new pick at
     the next music update, except by a gap whose length depends on how often the original's
     music update runs (Q-1700 question 2, open), but an entry with a missing file and its
     *repeat* flag set can, provided a failed start makes its entry the ended track (claim 4).
     `unmeasured`
     for zero volume in the original until it is known which in-game setting gives a music
     volume of zero (Q-1700 question 5); MU6 part c then tests it by the ranged draws a shuffle
     pick makes. `measurable` for our engine's rule (MU6 part c's engine fixture). `measurable`
     (MU6 part a, third and fourth runs) for the pass-over at the pick under shuffle of an entry
     with no section of its own, which has no sound file name and so no file present.
     `unmeasured` for the same pass-over of an entry whose named sound file is missing at startup
     or whose side restriction does not match: no draw-level scenario includes such entries.
   - Confidence: high for no silent gap of a track's length, and for the pass-over at the pick under
     shuffle of an entry whose *normal* flag is clear, the case MU6 part a tests; medium for the
     pass-over at the pick under shuffle of an entry with no section of its own, until MU6 part a's
     third and fourth runs are made, and of an entry whose named sound file is missing at startup or
     whose side restriction does not match, which no draw-level scenario includes; medium for the
     pass-over at the pick under list order, until MU1 part l is made; low for zero volume in the
     original (`unmeasured`: which in-game setting gives a music volume of zero, and so whether the
     original makes no pick there, is open, Q-1700 question 5); medium for our engine's zero-volume
     rule, a design rule that only our engine's fixture checks, with no oracle run behind it.

4. **Repeat.** When there is an ended track and its entry has its *repeat* flag set, or there is
   an ended track and the repeat setting is on, the next track is the ended track again, whatever
   the shuffle setting and whether or not it is still available, and no ranged draw is made.
   Otherwise, including when there is no ended track, claim 5 or claim 6 picks the next track. A failed start (Terms) still leaves its entry
   the ended track (State and ordering), so repeat starts it again. What a start leaves does not
   depend on the entry's *repeat* flag or on the repeat setting: whether the start fails or not, the
   started entry becomes the ended track and music stays started (claim 10), with either flag and
   either setting; repeat acts only at the next pick. The first start after loading is made with
   no ended track, so repeat does not choose the loading track then (claim 11). The first track of a skirmish is started in the same way
   (claim 11), so it is the ended track for the second pick and repeat applies to it.
   - Backing: `measurable` (MU3 parts a and b for an available track, MU3 part c for a track that
     is not available; MU6 part d for no ranged draw). `measurable` (MU4 part b, second run) for
     a failed start of a track that was not the ended track making it the ended track: there the
     fallback starts entry 0, whose file was absent at startup (the random-state recorder's
     advance of 1000 ranged draws shows that fallback pick), and the one available track can
     play again only if the failed start leaves music started (claim 10) and leaves that track no
     longer the ended track. The run does not tell entry 0 becoming the ended track from a failed
     start that leaves no ended track. `measurable` (MU4 part b, third run) for that distinction
     and for repeat choosing a failed track again: entry 0 there has its *repeat* flag set and
     its file absent from startup. Three outcomes are told apart. If the failed start made entry
     0 the ended track, repeat chooses it again at each pick with no ranged draw, so the silence
     after the fallback's failed start shows no further advance of 1000 ranged draws, and entry
     0 is heard once its test track is supplied during the run. If it left no ended track, the
     next pick accepts T1, which is heard again. If it left T1 the ended track, every pick in the
     silence finds no other available track and makes the fallback's 1000 ranged draws again, so
     the random-state recorder sees a further such advance in the silence, although the tones
     match the first outcome. The recovery also tells repeated failed starts from music that
     stopped. That the *repeat* flag and the
     repeat setting play no part in what a failed start leaves is `unmeasured`: the runs that
     tell what a failed start leaves (MU4 part b's third run, MU3 parts d and e) all use an entry
     with its *repeat* flag set, and none uses the repeat setting. That each such repeat choice
     makes a new failed start is `unmeasured`: silence looks the same whether every choice makes a
     failed start or the choices in between make no attempt, and the recovery in MU4 part b's third
     run shows only that a choice made after the file is supplied starts it. `measurable` (MU3
     part b and MU8 part a's third run) for repeat of the first track of a skirmish.
     `measurable` (MU10 part b) for a failed start of a file that is present but does not parse
     making its entry the ended track. `measurable` (MU8 part g) for the first start after
     loading being made with no ended track, even with the loading track's *repeat* flag set. `measurable` (MU3 parts d and e) for repeat of a track whose file
     is deleted during play, with named test files changed during the run (Measurement
     scenarios).
   - Confidence: high; medium for a failed start making its entry the ended track, until MU4 part b's second and
     third runs are made, for repeat starting a failed track again, until MU3 parts d and e
     are run, and for the *repeat* flag and the repeat setting playing no part in what a start
     leaves (`unmeasured`, as the backing says); medium for a new failed start at each repeat
     choice (`unmeasured`, as the backing says: it is what claim 4's start rule gives at each
     choice, and no observation separates it from choices that make no attempt); medium for the
     first start after loading being made with no ended track, until MU8 part g is made

5. **List order (shuffle setting off).** The next track is the first available track found by
   stepping forward through the list from the entry after the ended track, wrapping from the last
   entry to entry 0. The ended track itself is the last candidate of the cycle, so with no other
   available track it plays again if it is available. If there is no ended track (claim 11),
   stepping begins at entry 0. If no entry is available, claim 7 applies. A list-order pick makes
   no ranged draw.
   - Backing: `measurable` (MU1 part a) for the order and the wrap; `measurable` (MU4 part a)
     for a list with no available entry; `measurable` (MU6 part e) for no ranged draw.
     `measurable` (MU4 part c) for the ended track as the last candidate, with a single available
     track other than entry 0.
   - Confidence: high for the order, the wrap and no ranged draw for entries without a side
     restriction (for entries with one, as claim 2 rates the original's side rule); medium for
     the ended track as the last candidate, until MU4 part c is made
   - The start at entry 0 when there is no ended track: `measurable` (MU8 parts a and c),
     medium until they are made.

6. **Shuffle (shuffle setting on).** Unless claim 7 applies, the next track is a uniformly random
   available track other than the ended track. It is drawn from the unsynchronised random stream
   (S-0202 claim 11) as repeated ranged draws *R*(0, *n* − 1) (S-0202 claim 4) over all entries,
   available or not; a ranged draw is accepted when it differs from the ended track and names an
   available track. A pick that has made 1000 ranged draws without accepting one stops, and claim
   7 applies: such a pick has consumed exactly 1000 ranged draws. In our engine (design) the
   thousandth draw is made but never accepted, so a pick accepts only among its first 999 draws.
   Whether the original can accept the thousandth is an open question (Open questions). The
   question matters only for a pick that has an available track other than the ended one and has
   missed with all of its first 999 draws. Such a miss has probability at most (1 − 1/*n*)<sup>999</sup>. The raw values each
   ranged draw takes follow S-0202 claim 4: with one entry, *R*(0, 0) takes none. So shuffle never
   picks the ended track unless claim 4 applies or the pick finds no other available track. In the
   latter case claim 7 applies. With the switch on, its fallback starts entry 0, which is the
   ended track only when the ended track is entry 0. With the switch off, its rule starts the
   ended track again when that is the only available track. With exactly two
   available tracks its picks alternate, except after claim 7's fallback; a start that fails
   (claim 8) is silent, so the tracks heard need not alternate. These guarantees hold for
   every pick that accepts a ranged draw.
   - Backing: `measurable` (MU2 part a) for no immediate repeat and the alternation; `measurable`
     (MU2 part b) for the uniform choice among the other available tracks; `measurable` (MU6 parts
     a and b, by the random-state recorder) for the range of the draws, the raw values they take
     and the total of 1000 draws on a pick that accepts none. `measurable` (MU6 part a, third and
     fourth runs) for the range also covering an entry with no section of its own, so with no
     sound file. `unmeasured` for the range also covering an entry whose named sound file is
     missing at startup or whose side restriction does not match: no draw-level scenario
     includes such entries. The thousandth draw never being accepted is `unmeasured` (our
     engine's design): it differs from accepting it only on a pick that misses with its first
     999 draws, which no scenario can arrange.
   - Confidence: high for no immediate repeat and the alternation, for entries without a side
     restriction (for entries with one, as claim 2 rates the original's side rule); medium for the uniform
     choice, pending MU2 part b, which can detect a fixed order or a strong bias but not a small
     one; high for the range of the draws and the total of 1000, which MU6 part b counts
     exactly, where the unavailable entries are ones whose *normal* flag is clear; medium for
     the range also covering an entry with no section of its own, until MU6 part a's third and
     fourth runs are made, and an entry whose named sound file is missing at startup or whose
     side restriction does not match, which no MU6 part includes (as claim 3 rates their
     pass-over); medium for the thousandth draw never being accepted (`unmeasured`, our engine's
     design)

7. **Fallback when nothing is picked (quirk).** With `quirks.music_fallback_entry_zero` on (the
   default, the original's behaviour), when claim 5 finds no available entry, or claim 6 stops
   without accepting a ranged draw, entry 0 is started whether or not it is available. If entry
   0's start does not fail (Terms) it plays, even when its *normal* flag is clear or its side
   restriction does not match. If it cannot, there is silence and the pick is made again at the
   next music update. Consequences, which apply only when claim 4 does not select the ended
   track:
   - With list order and no available entry at all, entry 0 is started again and again; each
     start plays only if it does not fail (claim 8a).
   - With shuffle and only one available track *k*: if *k* is entry 0, it plays again and again.
     Otherwise *k* and entry 0 alternate, with entry 0 silent if its file is missing, so *k* is
     heard again after that brief failed start, which makes entry 0 the ended track (claim 4).
     This holds unless entry 0 has its *repeat* flag set, which keeps entry 0 playing (claim 4),
     and except where a pick after entry 0 again
     accepts no draw (claim 6), which starts entry 0 once more.
   - Shuffle can also stop without accepting a draw while other tracks are available; claim 6
     gives the bound. Entry 0 is then started in the same way.

   With the switch off (our engine's design for that setting):
   - List order with no available entry: no track starts and no ranged draw is made. This holds
     from the first pick of the game (claim 11): that pick starts nothing, yet it begins music
     started (claim 10), so the pick is made again at each music update, and the music stays silent: file presence is fixed at
     startup (claim 8) and the other availability inputs do not change during a game, so no entry
     becomes available later in it.
     With only the ended track available, it plays again, as with the switch on (claim 5).
   - Shuffle with at least one available track other than the ended track: the ranged draws of
     claim 6 go on past the thousandth until one is accepted. So the result and the draws are those
     of the switch on whenever the switch on accepts a draw.
   - Shuffle with the ended track the only available track: it plays again, and no ranged draw
     is made.
   - Shuffle with no available track: as list order with no available entry, and no ranged draw
     is made.
   - Backing: `measurable` for the switch on: MU4 part b and MU6 part b for shuffle, and MU4 part a
     for list order, which can also falsify it by silence, since music starts with no available
     entry (claim 11); `measurable` for the switch off, by
     MU4's engine fixture, for list order with no available entry (part a) and for shuffle with
     the ended track the only available track (part b); `unmeasured`, our engine's design, for
     shuffle with no available track, which no fixture sets up with a non-empty list, and for the
     continuation past the thousandth ranged draw, which no fixture reaches; `unmeasured` for an
     entry 0 that is not available only because its side restriction does not match, which no
     scenario sets up (MU4 and MU6 give entry 0 no side restriction)
   - Confidence: medium for list order, until MU4 part a is made; medium for shuffle (the
     alternation rests on the pick rule of claim 6 applied twice and, with entry 0's file missing,
     on claim 4's failed start making entry 0 the ended track, which MU4 part b's second and third
     runs test, claim 4); medium for the switch-off rules that MU4's engine fixture checks, design
     rules with no oracle run behind them (list order with no available entry, and shuffle with the
     ended track the only available track), and medium for the other switch-off rules (shuffle with
     no available track, and the continuation past the thousandth ranged draw), which are
     `unmeasured` (our engine's design); medium for an entry 0 whose side restriction does not match
     (`unmeasured`)

8. **Missing sound files.** A track entry whose sound file is not present is never an available
   track, so claims 5 and 6 never pick it to be heard. In our engine they pass over it at the
   pick; whether the original does so, or starts it, fails and picks again, is claim 3's open
   alternative, which claim 3 rates for each setting. Presence counts the archives and loose files
   mounted when the program starts, the theme archive included (S-0005 claim 12). A file supplied
   after the program has started does not make its entry available, and a file removed after it
   has started does not make its entry unavailable. When the track of such a removed file is due
   to start and its sound file cannot be opened (removing a loose file can uncover an
   archived copy, S-0005 claim 1), it cannot play: there is silence, and the next track is picked at the next music
   update as if the track had ended.
   - Backing: `measurable` (MU1 part a) for a missing file's entry never being heard and leaving
     no silence of a track's length. Whether under list order it is passed over at the pick or
     started, failed and followed by a new pick is claim 3's, with claim 3's backing.
     `measurable` (MU1 part h) for files supplied or removed after the program has started, with
     a file removed and another supplied during play (first run) and between startup and the
     start of the skirmish (second run), through the file system only (Measurement scenarios).
     MU1 part h cannot tell a
     failed start of a file that has gone missing from a pass-over of it, except by the length of
     the gap, which depends on how often the music update runs (open, Q-1700 question 2). MU3 parts
     d and e tell them apart only for a track with the *repeat* flag
     set (claim 4); for a track
     without it, no proposed scenario separates the failed start from a pass-over until that
     question is answered. `measurable` (MU1 part k) for the theme archive's files counting
     as present.
   - Confidence: high for a missing file's entry never being heard and leaving no silence of a
     track's length; under list order, how it is passed over as claim 3 rates it (medium); medium
     for a file supplied after the program has started not making its entry available, until
     MU1 part h runs; medium for a file removed after startup not making its entry unavailable,
     and for its failed start, when the entry has the *repeat* flag, as in claim 4, until MU3
     parts d and e run; low for a file removed after startup from an entry without the *repeat*
     flag, both for its entry staying available and for its failed start, since MU1 part h cannot
     tell that from a pass-over (`unmeasured` until Q-1700 question 2 is answered); medium for the theme archive's files counting, since part k shows it only for
     tracks whose files are found in that archive alone

8a. **Sound files that are present but do not play.** Presence (claim 8) depends only on the file
   being found: its content is not judged, for each kind of file that MU10 part a runs (a found
   file whose format chunk declares a very large size is not covered: Open questions). So an entry whose sound file is found but is not a
   playable sound is file present, and is an available track when claim 2's other conditions
   hold; claims 5 and 6 can pick it. A start fails (Terms) when its sound file cannot be opened,
   or when the file opened does not parse as a playable sound. A playable sound file is a RIFF file
   of form WAVE whose format chunk comes before its data chunk, whose format code is 1
   (uncompressed PCM) or 17 (IMA ADPCM), and which holds sample data after its header; a file that
   lacks any of these does not parse (of the other format codes, MU10 part a tries only code 2), a file with no data chunk included. The chunk order is part
   of this format: a format chunk placed after the data chunk does not count, so such a file does
   not parse (medium, like the other cases, until MU10 part a's run with the data chunk first is
   made).

   A failed start of a present file has the same result as one of a file that has gone missing
   (claims 4 and 8): silence, its entry becomes the ended track, music stays started (claim 10),
   and the next pick is made at the next music update. For each kind of file that does not parse that MU10 part a runs, the program goes on
   running; whether it survives a found file whose format chunk declares a very large size is open
   (Open questions). A file that
   parses but whose sample data is cut short ends where its data ends: a track plays the lesser of
   the data size its header gives and the sample data the file holds, and data beyond the size the
   header gives is not played. Our engine (design) gives the importer warning for a found sound
   file that does not parse, naming the entry, and still counts the file as present.
   - Backing: `measurable` (MU10 part a, by the random-state recorder) for a present file that does
     not parse staying an available track and its start failing, one run for each way a file can fail
     the format (no RIFF header, no data chunk, the format chunk after the data chunk, another
     format code, no sample data), but none for a RIFF header whose form is not WAVE, which no run isolates; `measurable` (MU10 part b) for that failed start making its entry the ended
     track; `measurable` (MU10 part c) for the end of a track whose data is cut short or longer than its
     header gives. `measurable` for our engine's warning (MU10 part a's engine fixture).
   - Confidence: medium for presence depending only on the file being found and for each of the
     cases that do not parse, until MU10 part a's runs are made, and because MU10 part a's run with
     a text file does not tell a missing RIFF header from a form that is not WAVE, and for the end
     of a track cut short, until MU10 part c is made; medium for format codes other than 1, 17 and
     2, which no run tries; medium for our engine's warning, a design rule that only our engine's
     fixture checks, with no oracle run behind it. A format chunk with a very large declared size,
     and a cut through the middle of the last block of a compressed file, are not covered (Open
     questions).

9. **Empty track list.** In our engine (our engine's choice, with either switch setting and
   either repeat setting), a music database that holds sections but no track entry (*n* = 0) gives
   silence throughout: no music update makes a pick and no ranged draw is taken. In the original no
   pick is made before music has started (Terms); what the original does once a skirmish with an
   empty list has loaded is open (Open questions). A music database that cannot be
   resolved, or that holds no section at all, stops the program from starting (S-0005 claim 16).
   - Backing: `measurable` for our engine (MU5's engine fixture). The original's behaviour with
     an empty list is open; MU5 records it. The rest is S-0005's.
   - Confidence: medium for our engine's rule, a design rule that only our engine's fixture checks,
     with no oracle run behind it; the original's behaviour with an empty list is not rated (open).

10. **Music update and music started.** How often the original's music update runs, and where
    its draws fall among the sound choices of S-0701 claim 30, are open (Open questions). Our
    engine's rule (design):
    - One music update runs at the end of every audio pass of S-0701 claim 30's schedule, after
      that pass's sound choices, and one at each of that claim's music-only passes while the
      options menu is open (claim 13). The ranged draws of a shuffle pick (claim 6) therefore follow
      that pass's draws on the same unsynchronised stream. A pick that makes no ranged draw (list
      order, repeat) leaves the stream unchanged.
    - A track finishes at the first pass, an audio pass or a music-only pass, at which it would
      complete under S-0701 claim 30's completion rule for a sample with pitch percentage *p* = 100,
      its logical duration counted on music time (S-0701 claim 30) from the pass at which it
      started. Outside the options menu, music time advances with logical audio time. The music
      update at the end of that same pass is the "next music update" of claim 3: it makes the pick
      and starts the track picked, whose logical duration then counts from that pass. A track's
      logical duration is that of the sample data it actually plays (claim 8a): for a file cut
      short, the frames present, not the data size its header gives. A failed start (Terms) plays
      nothing, and the pick is made again at the update of the following pass (claims 8 and 8a).
    - Music started begins with claim 11's first start after loading: the map theme's start, or
      the first pick, even when that pick starts nothing (claim 7 with its switch off and no
      available entry). The loading track's start does not begin it.
      Before it, no music update makes a pick. It does not require that a track is heard: a
      failed start still leaves music started, in the original and in our engine.
    - Backing: `measurable` for our engine's rules (MU7, an engine-only scenario). `measurable`
      (MU4 part b, second run) for the original's failed start leaving music started: there the
      one available track plays again after the fallback's failed start of entry 0, shown by the
      recorder's advance of 1000 ranged draws, only if that start leaves music started and stops
      that track being the ended track (claim 4). MU4 part b's third run and MU3 part d also
      show it, by the recovery once the missing test track is supplied.
    - Confidence: medium for our engine's schedule, a design rule that only MU7 checks on our
      engine, with no oracle run behind it; medium for the original's failed start leaving music
      started, until MU4 part b's second run is made.

11. **Start of music in a skirmish.** This claim holds while the music volume is above zero during
    loading; what the original does when it is zero then is open (Open questions, Q-1700 question
    5). Every skirmish starts with no ended track, whatever was played before it in the session, in
    the menus or in an earlier game (claim 12 gives what plays after a game). When a game is started
    from the menus, whatever is playing as it starts to load, the menu track included, is stopped at
    once, with no fade, whether or not the music database has a loading track: no music carries over
    from the menus into a game. The original's options menu offers no restart in a skirmish (claim
    13). While the game loads, the loading track (Terms), if the music database has one, is started
    directly, not by a pick: like every start it plays whether or not it is available, unless it
    fails (claim 8a), and it makes no ranged draw. It does not begin music started (claim 10), and
    no pick is made while the game loads. Whether the loading track's start failed or not, and
    whatever its *repeat* flag and the repeat setting, the first start after loading is made with no
    ended track (claim 4). Then, if the in-game music setting is on:
    - if the map theme (Terms) names a track entry, that entry is started first, directly, whether
      or not it is available: it plays unless the start fails, and it becomes the ended track.
    - otherwise the first track is chosen by an ordinary pick with no ended track (claims 4 to 7):
      under list order stepping begins at entry 0 (claim 5), under shuffle no ranged draw is
      rejected for naming the ended track (claim 6), and with no available entry in a list that
      holds at least one entry the fallback of claim 7 applies, so a track is started even then
      (claim 9 covers an empty list). With claim 7's switch off and no available entry in such a
      list, our engine's first pick starts nothing; music is started all the same (claim 10), and
      the pick is made again at each music update (claim 7).

    This first start is an ordinary start: it leaves music started (claim 10), its entry becomes
    the ended track for the second pick, and repeat applies to it (claim 4). The start does not
    depend on the local player's side, except through availability (claim 2). Apart from the map
    theme, no game setting chooses the first track; whether a game mode's own data can give a map
    theme is not established (Open questions). With the in-game
    music setting off, as read at startup, no track is picked or started during the game, and no
    ranged draw is made for music. What changing that setting during a session does is open
    (Q-1700 question 4).
    In the original, if the loading track is still playing when loading finishes, it fades out
    first, and the first start is made at the first music update after the fade has ended; how
    long the fade lasts is not established (Open questions). With no loading track playing, the
    first start is made at the first music update of the game. In the original, a map theme that
    names the loading track's own entry gives no fade: the loading track plays on to its end and
    is then started again from its start, as the map theme. Our engine (design): the loading
    track, if any, plays while the game loads and is stopped when the game starts (a fade, if one
    is drawn, is presentation and delays nothing), and the first start is made at the music update
    of the game's first audio pass (claim 10), so under shuffle the first pick's ranged draws
    follow that pass's sound choices on the unsynchronised stream.
    - Backing: `measurable` (MU8 part a) for the first pick under list order and for repeat of the
      first track; `measurable` (MU8 part b, by the random-state recorder) for the first shuffle
      pick having no ended track; `measurable` (MU8 part c) for a new skirmish forgetting the ended
      track of an earlier one; `measurable` (MU8 part d) for the map theme, and (its fourth run, by
      the random-state recorder) for its start making no ranged draw; `measurable` (MU4 part a) for
      music starting with no available entry; `measurable` (MU8 part e) for the in-game music
      setting off, as read at startup, and (its second run) for the loading track playing with that
      setting off; `measurable` (MU8 part h) for the first track not depending on the local player's
      side or on the other skirmish settings that part h varies; `measurable` for our engine's
      timing (MU8's engine fixture). The original's delay from the end of loading is only recorded
      (MU8 part f), so the original's first start coming at the first music update, and not at a
      later one, is `unmeasured`: how often the music update runs is open (claim 10). `unmeasured`
      for no ranged draw with the in-game music setting off: MU8 part e records no random state.
      `measurable` (MU8 part f, its run with a long loading track) for the loading track's fade
      ending before the first start. `measurable` (MU11 part g) for the track playing in the menus
      being stopped at once when a game started from the menus starts to load and the music database
      has no loading track, and `unmeasured` when it has one, as no run of MU11 part g gives it one.
      `measurable` (MU8 part g) for the first pick having no ended track when the music database has
      a loading track. `unmeasured` for a map theme that names the loading track's entry: no
      scenario sets it up. `unmeasured` for the loading track playing whether or not it is available
      and making no ranged draw: MU8 part g gives it the normal flag, and neither MU8 part f nor
      part g reads the random state at its start.
    - Confidence: medium for the first pick being an ordinary pick with no ended track, until MU8
      parts a and b are made (and, when the music database has a loading track, part g), as claim 5
      rates where list order starts; medium for the first track not depending on the side or on
      another game setting, until MU8 part h is made and because part h varies only the settings it
      lists; medium for the loading track playing with the in-game music setting off and for the
      setting being the one read at startup, until MU8 part e's runs are made; medium for a map
      theme naming the loading track's entry (`unmeasured`, as the backing says); medium for the
      loading track playing whether or not it is available and making no ranged draw
      (`unmeasured`); medium for the original's first start at the first music update and for no
      ranged draw with the in-game music setting off (`unmeasured`); high for the first track
      becoming the ended track, for the in-game music setting off giving no music and for music in
      the menus being stopped at once when a game starts to load with no loading track in the music
      database; medium for that stop when the database has a loading track (`unmeasured`); medium
      for a new skirmish forgetting the ended track, until MU8 part c is made; medium for the map
      theme, until MU8 part d is made; medium for the loading track's fade coming first in the
      original, until MU8 part f's run with a long loading track is made (its length is not
      established and is only recorded); medium for our engine's timing, a design rule that only
      our engine's fixture checks, with no oracle run behind it.

12. **The end of the game and the score screen.** This claim holds while the music volume is
    above zero; what the original does at zero is open (Q-1700 question 5). In the original:
    - At the end of the game, won or lost, the music does not change. The end delay's ticks are
      ordinary play (S-0414 claim 8): the playing track plays on, music updates go on in them as
      in play, and a track that finishes there is followed by an ordinary pick (claims 3 to 7).
    - When the game is over, the playing track is still neither stopped nor faded. It plays on
      through the original's real-time wait for the result announcement (S-0414 claim 8), until
      the score screen opens. Music updates go on
      in that wait too, so a track that finishes there is also followed by an ordinary pick. How
      long the wait lasts belongs to the result screens, not to this spec.
    - When the score screen opens, the playing track is stopped at once, with no fade, and the
      score track (Terms) is started directly, from its start, not by a pick. Like every direct
      start, it plays whether or not it is available, unless the start fails (claim 8a), and it
      makes no ranged draw. With no score track in the music database, the score screen is
      silent.
    - Music updates go on at the score screen. Only the picks of claims 5 and 6 are left out:
      when the score track finishes, no such pick follows, and the score screen is silent from then
      on. Under the repeat setting, or with the score track's own *repeat* flag set, the score
      track starts again each time it finishes, with no ranged draw (claim 4).
    - The score track plays even with the in-game music setting off: that setting covers only
      music during a game (claim 11).
    - Leaving the score screen for the main menu stops the score track at once, and the menu
      track (Terms), if any, starts. Music in the menus is otherwise outside this spec, except
      that a game starting to load stops whatever is playing (claim 11).

    Our engine (design, a departure with no switch, Edge cases): through the end delay the music
    is as in the original. Its ticks are ordinary play in ordinary cycles, so audio passes, and
    with them music updates and picks, go on in them on S-0701 claim 30's schedule. The end
    screen, in place of the score screen, opens when the game is over (S-0414 claim 8): at the
    end of the cycle that runs the end delay's final tick, after that cycle's audio pass if one
    falls there (S-0701 claim 30). The music update of that pass is the last of the game: no
    music update, and so no pick, runs after that cycle. Only the original's real-time wait
    for the result announcement is dropped, as S-0414 claim 8 drops it, and with it the music
    updates the original runs in that wait. The playing track is stopped at once when the end
    screen opens, and the end screen then follows the score-screen rules above: the score track
    is started directly, with the same silence after it, repeats and silent screen without a
    score track, and leaving it for the main menu starts the menu track. The score track's start,
    the silence after it and its repeats make no ranged draw, so they need no audio pass, and the
    end screen has none (Changes for other specs, S-0701 claim 30). On the end screen the score
    track is timed by our engine's audio output, not by audio passes: it has finished when its
    whole length has been played out, and under repeat it starts again from its start as soon as
    it has finished. Nothing synchronised depends on either moment. A failed start of the score
    track (claim 8a) leaves the end screen silent until it is left: our engine makes no further
    attempt, even under repeat, and starts no other track. The original, under repeat, attempts
    the start again at each music update, which is equally silent while the start fails.
    - Backing: `measurable` (MU11 part a) for the track playing on through the end delay and the
      wait to the score screen, the picks in either, its stop there and the score track's start,
      after a win, and (MU11 part i) for the same after a loss, except the picks; (MU11 part b)
      for the silence after the score track and its repeats; (MU11 part c) for the silent score
      screen with no score track; (MU11 part d) for the score track with the in-game music setting
      off; (MU11 part g) for the menu track stopping the score track. That the original, under
      repeat, attempts a failed score-track start again is `unmeasured`: silence looks the same
      whether or not each attempt is made, as claim 4 says of repeat choices. That the original's
      score-track start and its repeats make no ranged draw is `unmeasured`: MU11 does not record
      the random state at the score screen, as claim 11 says of the loading track. `measurable`
      for our engine (MU11's engine fixture).
    - Confidence: medium for the stop at the score screen and the direct start of the score track,
      until MU11 part a is made, and after a loss until part i is made; medium for the silence after
      the score track and its repeats, until MU11 part b is made; medium for the silent score screen
      with no score track, until MU11 part c is made; medium for the original's score track making
      no ranged draw (`unmeasured`, as the backing says); medium for the track playing on, unfaded,
      through the end delay and the wait and for a pick in either, until MU11 part a is made, and
      after a loss until part i is made (a pick after a loss rests on part a's runs, since part i
      places no tone's end in the end sequence); medium for the score track with the in-game music
      setting off, until MU11 part d is made; medium for the menu track's start; medium for the
      original's repeated attempts at a failed score-track start (`unmeasured`, as the backing
      says); medium for our engine's rules, a design rule that only our engine's fixture checks,
      with no oracle run behind it (music updates through the end delay, none after the cycle that
      runs its final tick, the stop when the end screen opens, the score-screen rules on the end
      screen, the score track's timing there and its failed start, with the single start attempt
      that the fixture's runs with the score track's file absent record).

13. **Options menu, leaving and restarting.** This claim holds while the music volume is above
    zero (Q-1700 question 5). In the original, in a skirmish:
    - While the options menu is open during a skirmish (a stand-still), the music is not
      paused. The playing track goes on, music updates continue, and a
      track that finishes is followed by an ordinary pick (claims 3 to 7), whose track is heard
      while the menu is still open.
    - The options menu offers one way to leave the game, and it ends the game at once, as S-0201
      claim 1 describes for a choice that ends it at once. The menu calls it aborting the mission,
      and it is how a skirmish player gives up the game. It is not S-0414 claim 11's surrender
      order: no house is marked to die or defeated, no end delay runs, no result announcement is
      given, and no score screen opens; the program goes back to the main menu. No choice of the
      menu sends the surrender order of S-0414 claim 11 in a skirmish.
    - When the game is left this way, the playing track starts to fade out, and no pick follows
      it. The program then waits, in real time, while the fade and any sound still playing go
      on, up to a fixed bound whose length is not established (Open questions); whatever still
      sounds at the end of the wait is stopped at once. The main menu then opens and starts the
      menu track, if the music database has one, as when the score screen is left (claim 12).
    - The options menu offers no restart in a skirmish.

    Our engine (design; the project lead has accepted it): the music plays on while the options
    menu is open, as in the original. The menu stays a stand-still for sound (S-0701 claim 30),
    but music time advances there with real time and S-0701 claim 30's music-only passes run, each
    with one music update (claim 10). The playing track is heard on, a track that finishes in the
    menu is followed by an ordinary pick (claims 3 to 7) at the music-only pass at which it
    completes on music time, and the track picked is heard while the menu is still open. On the
    return to the game the track goes on from the point it had reached, and its logical duration
    keeps counting on music time. Lost focus stops music time, also while the menu is open, and
    the unfocused stretch adds nothing to it (S-0701 claim 30). How long the menu stays open, in
    real time, decides when tracks end and how many picks are made there; when the track playing
    as the menu closes ends afterwards, and so when every later track starts and ends; and,
    through the shuffle picks' ranged draws, the unsynchronised stream's position for later sound
    choices and later picks. S-0701 claim 30 lists what depends on it. This follows the original,
    and no game state depends on it (S-0202 claim 11). Our engine's own settings screen, opened
    from a game, has no counterpart in the original; the music plays on there in the same way
    (design). Leaving through the options menu ends the game at once, as in the original (S-0201
    claim 1): the playing track is faded out or stopped at once, which is presentation and delays
    nothing, and no music update or pick follows, since no skirmish audio pass occurs after
    leaving the match (S-0701 claim 30); the main menu then starts the menu track. A surrender
    order, where our engine's interface offers one (S-0414 claim 11), leaves the music as in play
    for as long as play goes on: in our engine play goes on while a hostile pair remains, with the
    person who surrendered watching, and claim 12 applies from the end of the game, with our end
    screen once the game is over. A restart, where our engine offers one, starts the music as
    claim 11 says; whether the old track fades is presentation and delays nothing.
    - Backing: `measurable` (MU11 part f) for the music going on in the options menu and the pick
      made there; (MU11 part e) for leaving through the options menu ending the game at once, with
      no destruction of the local player's objects, no score screen and the main menu next, and for
      the fade and the menu track then, the wait's bound being only recorded (MU11 part e). That the
      wait has a fixed bound, and that a sound still playing at its end is stopped at once, is
      `unmeasured`: no run here has a sound outlast the wait. `measurable` (MU11 part h) for the
      menu offering no restart and no choice that sends the surrender order; `measurable` for our
      engine's rules (MU11's engine fixture).
    - Confidence: medium for the track going on while the options menu is open and for the pick
      made in the menu, until MU11 part f is made. Medium for leaving ending the game at once
      with no surrender order, for the fade and the menu track after it, and for the menu
      offering no restart and no choice that sends the surrender order, until MU11 parts e and h
      are made; medium for the wait's fixed bound and the stop at its end (`unmeasured`); medium
      for our engine's rules, a design rule that only our engine's fixture checks, with no oracle
      run behind it.

## Formulas

Let *e* be the ended track (or "none"), *n* the number of entries and *A* the set of available
tracks. All values are integers.

Repeat (claim 4). If *e* is not "none" and either *e*'s *repeat* flag is set or the repeat
setting is on, the result is *e* and no ranged draw is made. Otherwise the shuffle or list-order
pick below gives the result, with *e* as given ("none" included).

Shuffle pick (claim 6), *n* ≥ 1. Make the ranged draws *x<sub>i</sub>* = *R*(0, *n* − 1) on the
unsynchronised stream for *i* = 1, 2, …, stopping at the first *i* with *x<sub>i</sub>* ≠ *e* and
*x<sub>i</sub>* ∈ *A*, which is the result.
- Switch on: if no *x<sub>i</sub>* with *i* ≤ 999 is accepted, the ranged draw *x*<sub>1000</sub>
  is made as well, and if the pick has still accepted none, the result is entry 0. Our engine
  never accepts *x*<sub>1000</sub>; whether the original can is open (claim 6).
- Switch off: if *A* contains an entry other than *e*, the draws continue until one is accepted.
  Otherwise no ranged draw is made: the result is *e* when *e* ∈ *A*, and no track when *A* is
  empty.

List-order pick (claim 5), *n* ≥ 1. Let *s* = *e* + 1, or 0 when *e* is "none". The result is the
first *j* in the sequence *s*, *s* + 1, … taken modulo *n*, *n* steps long and so ending at *e* (at
*n* − 1 when *e* is "none"), with *j* ∈ *A*. If there is none, the result is entry 0 with the
switch on and no track with the switch off.

Empty list, *n* = 0 (claim 9), our engine: no pick and no ranged draw, with either switch
setting and either repeat setting.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| shuffle draws on failure | 1000 | ranged draws | claim 6, switch on |
| fallback entry | 0 | entry position | claim 7, switch on |

## State and ordering

| State | Event | Result |
|---|---|---|
| A track plays | It finishes | At the next music update (in our engine, that of the pass in which it finishes, claim 10): claim 4, else claim 5 or 6, picks; the pick starts at once |
| A track plays | Music update | Nothing |
| A skirmish loads | Loading starts | The ended track is forgotten; the loading track, if any, is started directly; music is not yet started, and no pick is made (claims 10 and 11) |
| A skirmish loads | Loading finishes | The first start after loading is made with no ended track (claim 11) |
| Loading finished, in-game music setting on | First music update (in the original, after the loading track's fade, except as claim 11 gives for a map theme that names the loading track's own entry) | The map theme's entry is started, else a pick with no ended track; music started begins, even if that pick starts nothing (claims 10 and 11) |
| In-game music setting off (read at startup) | Music update | No pick, nothing starts (claim 11) |
| Picked entry's start does not fail | Start | It plays; it becomes the ended track for the next pick |
| Picked entry's start fails (Terms; claim 8a) | Start | Silence; it becomes the ended track (claim 4); the next music update picks again |
| No track picked (switch off, claim 7) | Music update | The pick is made again |
| Music volume zero | Music update | Original: no pick, nothing starts (claim 3, low). Our engine: as at any other volume, the track playing silently |
| Music not yet started (before claim 11's start) | Music update | No pick, whatever the track list holds |
| Music started, no track playing (a failed start or a track that finished) | Music update | A pick (claims 4 to 7), unless, in the original, the music volume is zero; with an empty list, none in our engine (claim 9) |
| A track plays | End of the game | Original and our engine: no change; the end delay's ticks are ordinary play, with music updates and picks as in play (claim 12) |
| A track plays | The game is over (end of the end delay's final tick) | Original: it plays on through the real-time wait for the result announcement, with music updates and picks, until the score screen opens. Our engine: the end screen opens at the end of the cycle that runs the final tick, and no music update runs after that cycle (claim 12) |
| Any track plays, or none | Score screen opens | The playing track stops at once; the score track, if any, is started directly (claim 12) |
| Score track plays | It finishes | Repeat setting or its *repeat* flag: it starts again; otherwise silence, no pick (claim 12) |
| Score screen | Main menu opens | The score track stops at once; the menu track, if any, starts (claim 12) |
| A track plays in the menus | A game started from the menus starts to load | It stops at once (claim 11) |
| A track plays | Options menu opens | Original and our engine: it plays on, and a track that finishes there is followed by a pick, heard in the menu; our engine makes it at a music-only pass (claim 13) |
| Options menu open | The game is left through it | The game ends at once (S-0201 claim 1), with no surrender order and no score screen. Original: the track fades out, with no pick, through a real-time wait, and whatever still sounds is then stopped; the main menu starts the menu track. Our engine: the track is faded or stopped at once (presentation), no music update follows, and the main menu starts the menu track (claim 13) |
| A track plays | A surrender order (our engine, where its interface offers one) | No change; the music goes on as in play, and claim 12 applies from the end of the game (claim 13) |

## Edge cases and quirks

- `quirks.music_fallback_entry_zero` (claim 7). Default on. The fallback is not a bug: data can
  rely on it (an entry 0 that is never available but
  plays whenever nothing else can). The switch-off behaviour is one of several reasonable
  answers, so a switch keeps the original's result selectable. Backing and confidence: as claim
  7 rates the fallback.
- **Picks at zero music volume (departure, no switch).** Our engine picks and starts tracks at every
  music volume, silently at zero (claim 3), where, by claim 3's reading of the original
  (`unmeasured`, low), the original makes no pick at zero volume. The music volume is a setting of
  each player, and a pick's ranged draws are on the unsynchronised stream that sound choices share,
  so a rule that skipped picks would make later sound choices depend on that setting. Nothing is
  heard at zero volume in either. When the volume is raised again, our engine is heard part-way
  through whatever track it is then playing; what the original then plays is open (Q-1700 question
  5). Basis: the precedent of S-0701 claim 30, which already keeps sound choices independent of each
  player's mute settings; this departure extends that rule to music, a design choice recorded here
  with that reason; the project lead has accepted it. Backing and confidence: as claim 3 rates them
  (the original's rule low and `unmeasured`, our engine's rule medium, a design rule that only MU6
  part c's engine fixture checks).
- **End screen when the game is over (departure, no switch).** In the original the music goes
  on unchanged through the end delay's ticks and then through the real-time wait for the result
  announcement that follows the game being over (S-0414 claim 8); a track that finishes in either is followed by a pick (claim 12). Our engine keeps the
  end delay with its music: its ticks are ordinary play, so music updates and picks run in them
  as in the original. Only the real-time wait is dropped, as S-0414 claim 8 drops it: the end
  screen opens when the game is over, at the end of the cycle that runs the final tick, and no
  music update runs after that cycle, so there is no pick in the wait. The wait is measured in
  real time (it lasts until the result announcement has been given), not in ticks, so a pick in it would put ranged draws
  on the unsynchronised stream at a point set by real time; what is heard there is presentation,
  and no game state depends on it. A switch is not proposed: the behaviour changes music only,
  and reproducing the wait would bring back that dependence on real time. The project lead has
  accepted it. Backing and confidence: as claim 12 rates them: `measurable` (MU11 parts a
  and i) for the original's music playing on through the end delay and the wait, with its picks,
  medium until those parts are made; `measurable` (MU11's engine fixture) for our engine's end
  screen opening when the game is over with no music update after that cycle, medium, a design
  rule that only our engine's fixture checks, with no oracle run behind it.
- **Side restriction matched by name (design choice of scope, no switch).** This is a design choice,
  not a departure from a known rule of the original, whose own rule is not established. Our engine
  makes an entry whose side restriction names the local player's side available to that player
  (claim 2). In the original such an entry does not by that become available when the side is not
  the first (MU1 part b), and for the first side the result is not established. Which entries it
  makes available instead is not established and is not reproduced. Only that failure to match by
  name is claimed about the original. Whether it is a bug is not: if the original's rule is meant to
  be by name, it is a bug that changes music only; if the rule is not by name at all (Open
  questions), the stock data may suit it, and this is a difference of reading the data. Giving it no
  switch is a design choice recorded here: the behaviour changes music only, and the original's rule
  is not established, so there is no known behaviour for a switch to select. The project lead has
  accepted it. Backing and confidence: as claim 2 rates them (our engine's name matching medium, a
  design rule that only its fixture checks, the original's failure to match medium).
- A side name in a track entry's side restriction that names no side of the rules data (bad
  data): our engine gives the importer warning, naming the entry and the value, and treats the
  entry as never available. This spec covers skirmish playback only (claim 2), so the
  rule matters only in a game. Our engine makes this check at
  each skirmish start against the sides as S-0004 claim 1 builds them for that game, so a side
  that a mode or map layer adds counts, and outside a game against the sides of the rules data
  (design). What the original does with
  such a name belongs to its side rule, which is not established (claim 2).
  - Backing: `measurable` (MU1 part b's engine fixture, second run) for the warning and the
    entry never being available for a name that matches no side of the rules data. When the
    check is made (at each skirmish start, against a mode's or map layer's added sides, and
    outside a game) is our engine's design and `unmeasured` here: the fixture does not vary it.
  - Confidence: medium for the warning and the never-available entry, a design rule that only our
    engine's fixture checks, with no oracle run behind it; medium for when the check is made

## Import mapping

Music database (S-0005 claims 17 and 18):

| Key (section) | Concept | Parse form | Default when absent |
|---|---|---|---|
| `[Themes]`, every value | track entry identifiers, in list order; the key names play no other part, except that each position's value is looked up by its key, so a key written more than once gives its winning occurrence's value at each of its positions (S-0002 claim 4, S-0004 claim 2); a key whose value is empty adds no entry, and the values `none` and `<none>` are ordinary identifiers here, each an entry like any other, so the special meaning that S-0004 claim 2 gives them in rules type lists does not apply. S-0003 claim 14's reserved names apply to references to rules types; a track entry is not a rules type, and the track list has no reserved name, so every nonempty value names an entry | string with a text limit of 31 bytes (S-0003 claim 1); in our engine compared ignoring case, a repeat being the same entry (claim 1; `unmeasured`, low for the original) | none (empty list, claim 9) |
| `Sound` (the entry's own section) | sound file name (claim 1a) | string with a text limit of 255 bytes (MU1 part i; S-0003 claim 1 gives how a value is cut); then any leading `$` and `#` characters are dropped; the extension `.wav` is added, also to a name that already ends in it (MU1 part m); an empty value is treated as absent | empty: the file is never present, so the entry is never available, within claim 1a's limits |
| `Name` | display name, whose use is not established here (Q-1700 question 4, open) | not used here | not used here |
| `Normal` | *normal* flag (claim 2) | boolean (S-0003) | true |
| `Repeat` | *repeat* flag (claim 4) | boolean (S-0003) | false |
| `Side` | side restriction (claim 2) | a side name with a text limit of 127 bytes (S-0003 claim 1), matched ignoring case; empty counts as absent (MU1 part b, third run) | any side |
| `Scenario` | campaign scenario number; not used in a skirmish (claim 2); its role in a campaign is not established here | not used here | not used here |
| `LOADING`, as a `[Themes]` value | the loading track (claim 11): the entry whose identifier is this fixed name | compared with the entry identifiers ignoring case, as claim 1 compares them | no such entry: no loading track |
| `SCORE`, as a `[Themes]` value | the score track (claim 12): the entry whose identifier is this fixed name | compared with the entry identifiers ignoring case, as claim 1 compares them | no such entry: a silent score screen |
| `INTRO`, as a `[Themes]` value | the menu track (claim 12): the entry whose identifier is this fixed name | compared with the entry identifiers ignoring case, as claim 1 compares them | no such entry: no menu track |

A listed identifier for which the section lookup below (S-0002) finds no section keeps every
default, so its file is never present
and it never plays, within claim 1a's limits for an empty name. A malformed value of `Normal`, `Repeat` or `Side` is converted by
S-0003's rules for the key's parse form in the table, with S-0003's backing; no scenario here
supplies one.

Finding an entry's own section, and a key within it, follows S-0002, including its claim 3: two
different names with the same name hash are the same name there (`quirks.ini_hash_identity`).
Claim 1's test for a repeated identifier compares the identifiers' text, ignoring case, not
their hashes.

- Backing: `measurable`. MU1 part a's entries set only the sound file name unless a part says
  otherwise, so its cycle tests the defaults of `Normal`, `Repeat` and `Side`; MU1
  part c tests a repeat and a repeated key, MU6 part a an empty value and the values
  `none` and `<none>` in the track list, MU1 part f the 31-byte limit, MU1 part i the limit of
  255 bytes and MU1 part j that of 127 bytes, parts i and j each with the removal of trailing whitespace (part j
  for our engine only: `unmeasured` for the original, whose side rule claim 2 leaves open), and
  MU1 part g an identifier with no section of its own and an empty sound file name, MU11 parts a
  and g the fixed names of the score track and the menu track. `unmeasured`
  for the ignored case: MU1 part c's second run gives the same cycle whether a repeat in another
  case is the same entry or a separate entry with no section of its own (S-0001 claim 13).
- Confidence: high for the defaults of `Normal`, `Repeat` and `Side` and the repeat; medium for an
  empty `Side` value counting as absent, until MU1 part b's third run is made; low for the
  ignored case (`unmeasured`); medium for the empty value and the values `none` and `<none>` until
  MU6 part a's second to fourth runs are made; medium for the repeated key, which rests on S-0002
  claim 4; medium for the three text limits, for an identifier with no section and for an empty
  sound file name (they rest on a file named by the extension alone never being found), except
  low for the side restriction's limit in the original (`unmeasured`, part j).

The settings file:

| Key (section) | Concept | Parse form | Default when absent |
|---|---|---|---|
| `[Audio]` `IsScoreRepeat` | repeat setting (claim 4) | boolean (S-0003 claim 2): a value whose first byte is none of the six it recognises, such as `on` or `off`, keeps the default | off |
| `[Audio]` `IsScoreShuffle` | shuffle setting (claims 5 and 6) | boolean (S-0003 claim 2), as above | off |
| `[Audio]` `InGameMusic` | in-game music setting (claim 11) | boolean (S-0003 claim 2), as above | on |

The settings file is read only as a loose file, and a missing settings file means every default
above (S-0005 claim 5).

- Backing: `measurable`. Every scenario sets both keys before the program starts, and the
  expected sequences change with them: MU3 part b sets the repeat key (the same track over and
  over only with it on), MU2 parts a and b and MU6 set the shuffle key (no fixed cycle only with
  it on), and MU1 part a sets both off (the list-order cycle). MU9 parts a to c test the
  defaults with the keys absent, with values that are not recognised and with no settings
  file; MU8 part e sets the in-game music key off, and MU9 part c and its last run show its default.
- Confidence: high for the keys, their section and their boolean form; high for the defaults
  (MU9 parts a to c); medium for an
  unrecognised value keeping the default, which rests on S-0003 claim 2, until MU9 part b is made.

The map file (S-0009):

| Key (section) | Concept | Parse form | Default when absent |
|---|---|---|---|
| `[Basic]` `Theme` | map theme (claim 11) | a track identifier with a text limit of 127 bytes (S-0003 claim 1), compared with the entry identifiers ignoring case; a value that matches no entry gives no map theme; an empty value counts as absent | no map theme |

- Backing: `measurable` (MU8 part d: an entry named in another case, and a name that matches no
  entry). The text limit is `unmeasured`: no scenario supplies a name that long.
- Confidence: medium for the key, the case and the unmatched name, until MU8 part d is made; low
  for the text limit (`unmeasured`).

## Measurement scenarios

All scenarios: stock data with no add-ons or mods, plus loose test files put in
the game directory before the program starts and removed afterwards. These are the measurement
setup's named test files. MU1 part h and MU3 parts d and e also delete or supply one of them while
the program runs (MU4 part b's third run and MU10 part b's second run supply one), through the
file system only and never through the game,
and the files are still removed after the run. The project lead ruled on 2026-10-22 that named
test files may be deleted or supplied during a run in this way. Every other part supplies or
withholds each test file for the whole run, and MU1 part k supplies no test track and uses the
theme archive's own files. A
*test music database* is a
loose copy of the music database whose track list is replaced by the entries given below. Each
*test track* is a loose sound file with the extension `.wav`, in the same format as the stock
tracks, about 4 seconds long, holding one steady tone whose pitch differs from every other test
track's. Each entry's sound file name is the test track's name without the extension. A small
two-player skirmish map, one computer opponent, any game speed. If the in-game options have a
music volume control, the music volume is set to its maximum there; otherwise its default is kept
(Q-1700 question 5). *Recording:* the game's audio
output is recorded throughout, with the sound-effect and voice volumes set to zero in the options
so that only music is captured. Each track heard is identified by its tone. The shuffle and repeat
settings are set in the settings file before the program starts (Import mapping), and are
off unless a scenario says otherwise. That edit is one of the named test files: the settings file
is saved before the run and put back afterwards. The gap between the end of one tone and the
start of the next is noted for the Open questions. The in-game music setting is on. No test
music database has an entry whose identifier is the loading track's name (Import mapping), so no
loading track plays, and the map's theme key is absent or names no entry of the test music
database, unless a part says otherwise. The first track heard is the start of claim 11, which
MU8 tests; so that the other scenarios do not also rest on claim 11, their expected sequences
are judged by which track follows which from the second track heard on, in whatever phase the
cycle starts, unless a part says otherwise.
Each scenario's *engine fixture* runs the same set-up on our engine, with the switch off unless the
scenario says otherwise (MU5's fixture runs both settings, and MU7 is engine only), and checks
the switch-off results of claim 7 and the name matching of claim 2.

- **MU1. List order, skipping, side, scenario number and file name.**
  - (a) Entries T0 to T4 in that order, each section setting only the entry's sound file name,
    except: T2's test track is not supplied, and T3 has *normal* clear. Shuffle and repeat settings
    off. *Observable:* the sequence of tones over at least 12 tracks. *Expected:* the cycle T0, T1,
    T4, T0, T1, T4, … in that order, with no silence longer than the ordinary gap where T2 or T3
    would fall. *Falsified if:* T2's slot shows a silence of about a track's length, T3 is heard, or
    the order differs.
  - (b) As (a), but T1's side restriction names a side *X* whose index (S-0004 claim 9) is not 0,
    and no other entry has a side restriction. Run once with the local player's country on *X* and
    once on the side whose index is 0. *Expected (original):* T1 is not heard in the first run,
    where the cycle is T0, T4 (claim 2: no match by name). Whether T1 is heard in the second run
    is only recorded. *Falsified if:* T1 is heard in the first run. The engine fixture expects
    the reverse, with T1's side restriction written in another case than the side's name in the
    rules data (lower case for upper). It is run a second time with T1's side restriction naming
    no side of the rules data, and expects the importer warning naming T1 and the value, and T1
    never heard (Edge cases). Third run, original and engine: T1's side restriction key present
    with an empty value. *Expected:* the cycle of (a), T0, T1, T4 (an empty value counts as
    absent). *Falsified if:* T1 is not heard.
  - (c) As (a), with T1's identifier listed again after T4. Repeat with the second listing in
    another case (lower case for upper). *Expected:* the same cycle as (a), T0, T1, T4, in both
    runs. An entry kept at its last listing would give the cycle T0, T4, T1, which runs the other
    way round. *Falsified if:* T1 is heard twice per cycle, or T4 is followed by T1 and T1 by
    T0. Third run: as (a), with T4's line written under the same key as T1's line. *Expected:*
    the cycle T0, T1 if T1's line is the winning occurrence (S-0002 claim 4), or T0, T4 if T4's
    is. *Falsified if:* T1 and T4 are both heard.
  - (d) As (a), with T1's campaign scenario number set higher than any campaign mission's number.
    Run again with the number 1 (the run needs only a small positive number, one that could be a
    campaign mission's), and again with −1. *Expected:* the
    same cycle as (a) in every run. *Falsified if:* T1 is not heard in some run.
  - (e) As (a), with T4's sound file name written with a leading `$` and `#` while its test
    track's name has neither. Run again with each of the prefixes `#`, `$$`, `##$` and `#$#$` in
    its place. *Expected:* the same cycle as (a) in every run. *Falsified if:* T4 is not heard in
    some run, which would mean that only some runs of the two characters are removed. Last run:
    T4's test track and its sound file name both have `$#` in the middle of the name and no
    leading one. *Expected:* the cycle of (a). *Falsified if:* T4 is not heard, which would mean
    that the characters are removed elsewhere in the name too.
  - (f) As (a), with a sixth entry T5 after T4 whose identifier is 32 bytes long, its test track
    supplied. Run once with T5's own section named by all 32 bytes and once named by its first
    31 bytes. *Expected:* T5 is heard (after T4) only in the second run. *Falsified if:* T5 is
    heard in the first run, or not in the second.
  - (g) As (a), with a sixth entry T5 after T4 that has no section of its own, while a test track
    whose name is T5's identifier is supplied. Run again with T5 given a section of its own whose
    sound file name key is present with an empty value. Third run: T5's own section's sound file
    name is `$#$`, made only of the two removed characters, while a test track named `$#$` with
    the extension is supplied (and none named by the extension alone). *Expected:* the same cycle
    as (a) in all three runs. *Falsified if:* T5 is heard.
  - (h) As (a). After the first full cycle, while T0 plays, delete T1's test track and supply
    T2's (named test files changed during the run). *Expected:* from then on the cycle T0, T4,
    repeated, with T0 followed by a short silence before T4: T1 is still picked, since its file
    was present at startup, and fails to open (claim 8); T2 is never heard, since its file was
    absent at startup. The gap from T0's end to T4's start is recorded. *Falsified if:* T1 or T2
    is heard after the change, or T4 does not follow T0. Run a second time with the same change
    made through the file system while the program sits in the menus, after it has started and
    before the skirmish starts, with the same
    expected result. *Falsified if* (second run): T2 is heard, which points to a test made when the
    game loads rather than at startup. In both runs the gap is only recorded. If a test track
    cannot be deleted or supplied while the program runs, the part is reported unreached.
  - (i) As (a), with T4's sound file name written as its test track's name, then spaces, then the
    letter X, 256 bytes in all. Run again with 255 bytes in all. *Observable:* the sequence of
    tones over at least two cycles. *Expected:* T0, T1, T4 in the first run (the cut drops the X
    and the trailing spaces are removed); T0, T1 in the second (the file named with the X is never
    found). *Falsified if* T4 is missing from the first run or heard in the second.
  - (j) As (a), with T1's side restriction *Y* and T4's restriction *Y*, then spaces, then the
    letter X, 128 bytes in all, where *Y* names the rules data's side of index 0; the local
    player's country is on that side. Run again with 127 bytes in all. *Observable:* as part i.
    *Expected (our engine's fixture; for the original the cycles are only recorded, since its
    side rule is not established, claim 2):* T0, T1, T4 in the first run (the cut makes both
    restrictions the same name); T0, T1 in the second (T4's restriction is then another name,
    which does not match the local player's side, claim 2). *Falsified, for our engine, if* T4 is
    missing from the first run or heard in the second.
  - (k) Stock data and stock archives, no test tracks and no loose sound files; a skirmish with
    the music volume at its highest, shuffle and repeat settings off, on a map whose theme key
    names no entry of the stock music database (claim 11). The track heard while the game loads
    is the loading track (claim 11) and is not counted. *Observable:* the tracks
    heard over at least three track ends. *Expected:* the track after the first is a different
    one, as list order moves to the next available entry (claim 5). The stock tracks' files are
    found only in the theme archive, so a presence decision made before that archive is mounted
    would leave no track available, and then entry 0 would play again and again (claim 7).
    *Falsified if* the same track is heard four times in a row. *Inconclusive if* any
    track heard has its *repeat* flag set in the stock music database (claim 4), or fewer than
    two of its entries have their *normal* flag set and no side restriction (entries with a
    restriction are not counted, since the original's side rule is open, claim 2).
  - (l) Pass-over under list order (claim 3). As (a), with T1's *repeat* flag set and no test
    track supplied for T1 from the program's start, so that its file is missing throughout (T2's
    test track is still not supplied and T3 still has *normal* clear). Record over at least
    two full cycles. *Expected:* the cycle T0, T4, T0, T4, … with no silence longer than the gap
    between other tracks: the pick passes over T1 as it passes over T2 and T3. *Falsified if:* after T0
    the music falls silent for at least three track lengths, which a start of T1 that fails
    would give, since the failed entry then becomes the ended track and its *repeat* flag starts
    it again at each pick (claims 4 and 8). Part l is read only after MU4 part b's third run has
    met its expectation, including the single fallback advance of 1000 ranged draws during the
    silence and the recovery once entry 0's test track is supplied, which together show that a
    failed start makes its entry the ended track. Until then the expected cycle
    does not tell a pass-over from a start of T1 that fails followed by a new pick, and part l is
    inconclusive for claim 3.
  - (m) Extension already in the name (claim 1a). As (a), with T4's sound file name written as
    its test track's name followed by `.wav`, while only a test track with that name and the
    extension once (the ordinary test track) is supplied. *Expected:* the cycle T0, T1 (T4 is never
    found). Second run: a test track whose name is T4's sound file name followed by `.wav`, so
    with the extension twice, is supplied as well. *Expected:* the cycle of (a), T0, T1, T4. Third
    run: as the second, with T4's sound file name ending in `.WAV` instead. *Expected:* the cycle
    of (a). *Falsified if:* T4 is heard in the first run, or not in the second or third.
- **MU2. Shuffle.**
  - (a) Entries T0 to T2 available, shuffle setting on, repeat off; 60 tracks. Then quit, remove
    T2's test track, start the program again (so that T2's file is absent at startup, claim 8)
    and record 30 tracks. *Observable:* the tone sequence. *Expected:* no tone twice in a row in
    either run; all three heard in the first run; strict alternation in the second.
    *Falsified if:* any immediate repeat, or any break in the alternation. Claim 7's fallback can
    be ignored at this size: with *n* = 3, each pick misses with all of its first 999 draws with
    probability at most (2/3)<sup>999</sup>, below 10<sup>−175</sup>.
  - (b) Uniform choice. Entries T0 to T3 available, shuffle setting on, repeat off. Record until
    each of the four tracks has ended at least 60 times, counted per track (one long run or
    several, since only counts matter: the draws come from the unsynchronised stream, so no seed
    repeats them). *Observable:* for each ended track, the count of each track that follows it.
    *Expected:* each of the three other tracks follows it in about a third of its picks.
    *Falsified if:* for some ended track, one of the others follows it in fewer than one eighth
    or more than five eighths of its picks (under the claim, with 60 picks per ended track,
    this happens by chance in all twelve counts together with probability below 0.2 %), for
    instance a fixed cycle, in which each ended track always has the same successor.
- **MU3. Repeat.** (a) As MU1 (a), with T1's *repeat* flag set. *Expected:* once T1 plays after another track, only T1
  follows. (b) As MU1 (a), repeat setting on, then shuffle also on. *Expected:* in both runs, every
  track heard is the same as the first, which is the ended track for the second pick (claims 4
  and 11). *Falsified if:* a track differs from the one before it. (c) Shuffle on: T0 with
  *normal* clear and its *repeat* flag set (test track supplied), T1 available, T2 with *normal*
  clear. *Expected:* T1 is followed by T0 (the fallback of claim 7), and once T0 plays only T0
  follows, although T0 is not available. *Falsified if:* T1 is heard again after T0. The first
  track heard is T1, the only available track, chosen by the first pick (claim 11); a run whose
  first track is T0 falsifies claim 11 and is not counted for claim 4. The engine fixture
  expects, with the switch off, T1 over and over, under the same condition. (d) A failed start
  repeated: as (a). After T1 has played at least twice, delete T1's test track while T1 plays (a
  named test file changed during the run). *Expected:* after that T1 ends, silence for at least
  five track lengths: T1 stays the ended track, so it is started again at each music update and
  fails each time (claims 4 and 8). Then supply T1's test track again. *Expected:* T1 is heard
  again, and then over and over. *Falsified if:* any
  other track is heard after that end, which a pass-over of T1 would give (T4 next), or the
  silence continues for five track lengths after T1's test track is supplied again, which means
  the music stopped after the failed start rather than keeping T1 as the ended track. If the test track cannot be
  deleted or supplied while the program runs, part d is reported unreached. (e) A failed start
  becomes the ended track: shuffle on, T0 to T2 available, T1 with its *repeat* flag set. While
  the first track heard plays, if it is not T1, delete T1's test track (a named test file
  changed during the run); if it is T1, the run is inconclusive and is repeated. *Expected:*
  tracks other than T1 are heard until a pick names T1; that start fails, T1 becomes the ended
  track and its *repeat* flag starts it again at each music update, so silence follows for at
  least five track lengths. Then supply T1's test track again: T1 is heard over and over.
  *Falsified if:* after a silence a track other than T1 is heard, which means the failed start
  left the earlier track the ended track and shuffle picked again, or T1 is not heard after its
  test track is supplied again. A run in which T1 is never picked within 30 tracks is
  inconclusive.
- **MU4. Fallback.** (a) List order: entries T0 to T2, with T0's *normal* flag clear (test track
  supplied) and the test tracks of T1 and T2 not supplied. *Expected:* T0 is heard over and over.
  This is judged from the first track heard: music starts with no available entry, and the
  first pick's fallback starts T0 (claim 11). *Falsified if:* T0, once heard, is followed by
  silence at its end, or silence throughout while the control is heard. Repeat with T0's test track also not
  supplied. *Expected:* silence throughout; this run only records the original's behaviour.
  *Control:* MU1 part a under the same options, just before, in which music is heard. (b) Shuffle
  on: T0 with *normal* clear (test track supplied), T1 available, T2 with *normal* clear.
  *Expected:* T1 and T0 alternate (a pick that misses T1 in all of its first 999 draws, with *n* =
  3 at most (2/3)<sup>999</sup>, is ignored). Repeat with T0's test track not supplied. *Expected:*
  T1 over and over, each separated by at most a short gap: the fallback's start of T0 fails and
  makes T0 the ended track (claim 4), so the next pick accepts T1. *Falsified if:* T2 is heard, T0
  never plays in the first run, or, in the second run, T1 is not heard again within five track
  lengths after it ends: then the failed start did not both leave music started (claim 10) and
  stop T1 being the ended track (claim 4), and the run does not say which of the two failed.
  The tones alone do not show that the fallback started entry 0, since T1 played again without
  that start sounds the same. So the second run is made with the random-state recorder and MU6's
  accounting of unrelated draws: the run searches the ticks from T1's end to the start of the
  next track heard for one tick in which the unsynchronised stream advances by the 1000 ranged
  draws *R*(0, 2) predicted from the recorded state (S-0202 claim 4), the pick that accepts none
  and so starts entry 0 (claim 7). That tick is the first music update after T1's end, which may
  come later than the tick at which T1 ends, since how often the update runs is open (Q-1700
  question 2). Without that advance in the searched ticks the second run is inconclusive for
  claims 4 and 10. Third run: as the second, with T0's *repeat* flag set (test data), and with
  the random-state recorder and MU6's accounting of unrelated draws over the whole run.
  *Expected:* T1 is heard until the fallback first starts T0, then silence for the rest of the
  record (at least five track lengths): the failed start makes T0 the ended track, and its
  *repeat* flag starts it again at each pick with no ranged draw (claim 4). The recorder sees
  exactly one advance of the unsynchronised stream by the 1000 ranged draws *R*(0, 2), found as
  in the second run, and no further advance by such draws from that tick until T0's test track
  is supplied. *Falsified if:* T1 is heard again after that failed start: then either the
  failed start left no ended track or repeat does not start a failed track again, and the run
  does not say which. *Falsified if:* the recorder finds a further advance by 1000 ranged draws
  *R*(0, 2) in the silence: then the failed start left T1 the ended track, every pick after it
  finds no other available track, and the fallback chooses T0 and fails again, which sounds the
  same. Under that outcome such an advance comes at every music update in the silence, and five
  track lengths hold at least one music update, as they hold the picks after the tracks heard
  before it. After at least five track lengths of that silence, supply T0's test track (a named
  test file changed during the run). *Expected:* T0 is heard, and then over and over (claim 4).
  *Falsified if:* the silence continues for five track lengths after T0's test track is
  supplied, which means the music stopped after the failed start. The draw count tells T0 from
  T1 as the ended track, and the recovery tells repeated failed starts from music that stopped,
  so the run does not depend on the second run or on how the *repeat* flag affects what a
  failed start leaves. If the test track cannot be supplied while the program runs, the
  recovery is reported unreached: silence with no further advance then fits both repeated
  failed starts of T0 and music that stopped, and the run is then inconclusive on whether the music
  stopped. Without the
  first advance in the searched ticks, the third run is inconclusive, as the second is. (c) List
  order: T0 and T2 with *normal* clear (test tracks supplied), T1 available. *Expected:* from
  the second track heard on, T1 over and over (the ended track is the last candidate, claim 5),
  and T0 never heard after the first track heard. *Falsified if:* T0 is heard after T1, which the
  fallback of claim 7 would give. The engine fixture expects, with the switch off, silence in both
  runs of (a), T1 over and over in every run of (b) and in (c), and T0 never after the first track
  heard.
- **MU5. Empty list.** A test music database whose track list has no entry but which keeps its
  other sections. Shuffle setting off, then on, each with the repeat setting off and then on.
  *Observable:* whether the program starts, whether any music plays, and any failure of the
  program during the skirmish, which is recorded with the tick at which it happens. The result
  records the original's behaviour for the open question; no claim predicts it. The engine
  fixture runs every combination with the switch off and again with it on, and expects silence
  and no ranged draw in every run (claim 9).
  *Control:* MU1 part a under the same options, just before, in which music is heard; without
  it, silence does not count for the claim.
- **MU6. Draws and zero volume (claims 3 to 7), with the random-state recorder.** The
  set-up above, with the random-state recorder reading the unsynchronised stream (S-0202 claim
  11) and the audio recording aligned to its tick log. Shuffle setting on and repeat setting
  off, unless a part says otherwise. The computer opponent is the distant bystander computer
  player of S-0202 scenario R5 part a, placed so that nothing it does is heard in the observed
  area; any draw it makes on the unsynchronised stream is accounted for as an unrelated draw. As in
  S-0202 scenario R6 part b, nothing else on the map plays a sound while the observed picks
  happen; any other draw on the unsynchronised stream is accounted for, and a pick whose tick
  has an unrelated draw that cannot be accounted for is void. When the audio cannot place a
  pick's tick to within one tick, for example through buffering of the recording (S-0701 E2),
  the predicted advance is looked for in each tick of that window, and a match in one of them
  counts as the pick's tick. Since nothing else draws on the stream unaccounted for during the
  observed picks, an advance in that window, beyond the draws accounted for, that differs from
  the prediction falsifies; for a pick predicted to advance the stream, a window that shows no
  such advance at all is inconclusive.
  - (a) Accepted draws, *n* = 5. T0 to T2 available; T3 and T4 with *normal* clear, test
    tracks supplied. For each of 20 picks, predict from the state recorded before the tick in
    which the next tone starts the ranged draws *R*(0, 4) (S-0202 claim 4) up to the first that
    differs from the ended track and names T0, T1 or T2. *Expected:* in that tick the stream
    advances by exactly the predicted raw values, rejected values included, and the tone heard
    is the predicted entry. *Falsified if:* in a pick that is not void, the advance or the entry
    differs from the prediction, for instance because the draws range over the available tracks
    only. Second run: as the first, with one more key in the track list after T4 whose value is
    empty. Third run: as the first, with that key's value `none` instead, and no section named
    `none`. Fourth run: as the third, with the value `<none>`. *Expected:* in the second run the
    predictions are made with *n* = 5 and hold (the empty value adds no entry); in the third and
    fourth they are made with *n* = 6, ranged draws *R*(0, 5), and hold, and the sixth entry is
    never heard (it has no section, so no sound file). *Falsified if:* the second run's advances
    match *n* = 6 rather than 5, or the third or fourth run's match *n* = 5 rather than 6.
  - (b) Fallback count, *n* = 4. T0 with *normal* clear (test track supplied), T1 available, T2
    and T3 with *normal* clear. Each *R*(0, 3) takes exactly one raw value (S-0202 claim 4).
    *Expected:* in the tick in which T0 starts after T1 ends, the stream advances by exactly
    1000 raw values (1000 ranged draws, none accepted), and T0 is heard (claim
    7). After T0 ends, the advance is one raw value per draw up to the first that gives T1,
    predicted from the recorded state, and T1 is heard. *Falsified if:* the fallback advance is
    any other count, such as 999 or 1001, or a tone other than T0 follows T1. The engine fixture
    expects, with the switch off, no advance at T1's end and T1 again.
  - (c) Zero volume. As (b). While T1 plays, set the music volume to its lowest setting in the
    in-game options by scripted input, leave the options and keep the volume there for at least
    three track lengths of play after T1 would have ended, counted from entering the options,
    then set it back to its maximum.
    *Expected:* while the volume is at its lowest, no advance of 1000 raw values and no tone; after it is raised, at the next music
    update, an advance of 1000 raw values and T0 heard. *Falsified if:* an advance of 1000 raw
    values happens while the volume is at its lowest and that setting is known to be zero volume.
    Until that is known (Q-1700 question 5), such an advance is reported, not counted against
    claim 3. *Inconclusive if:* the in-game options have no music volume control, music is still
    heard at the lowest setting (which is then not zero volume), or no such advance follows the
    raise (what a volume change does to the music is open, Q-1700 question 5). The engine
    fixture sets the music volume to zero as T1 starts and expects, with the switch on, at T1's
    end the advance of 1000 raw values and T0 started (the fallback of claim 7), and with the
    switch off no advance and T1 started again; nothing is heard in either run (claim 3).
  - (d) Repeat makes no draw. As (a), with T1's *repeat* flag set. *Expected:* once T1 plays after another
    track, it starts again at each end, and at each of 10 such restarts the tick in which T1 starts
    again shows no advance attributable to the pick. Run a second time with no *repeat* flag
    and the repeat setting on, judging the restarts of the second track heard in the same way.
    Part (a), run just before, is the control for both runs.
    *Falsified if:* a restart tick that is not void shows an advance that unrelated draws do
    not explain.
  - (e) List order makes no draw. MU1 part a's entries and options (shuffle setting off), with
    the recorder. *Expected:* at each of 10 picks, the tick in which the next tone starts shows
    no advance attributable to the pick. *Falsified if:* such a tick that is not void shows an
    advance that unrelated draws do not explain. Part (a), run just before, is the control.

- **MU7. Music update schedule (engine only, claim 10).** Our engine, with the random-state
  recorder and a log of each audio pass's logical time *C*. Shuffle setting on, repeat off; T0 to
  T2 available, with test tracks of known frame count and sample rate. The fixture plays a test
  sound with several samples whose choice is made per instance (S-0701 claim 30) so that the
  choice falls in the pass at which the first track heard completes; a run in which no
  per-instance sound choice falls in that pass is void.
  *Expected:* that track completes at the first pass with *C* ≥ *C*<sub>0</sub> + *T*, where
  *T* is its logical duration at *p* = 100 (S-0701 claim 30); the shuffle pick's ranged draws
  appear in that same pass, after that pass's per-instance sound-choice draws (S-0701 claim 30),
  and the next track's logical start is that pass. Second run: the fixture makes T1's file
  unreadable after startup. *Expected:* when
  a pick names T1, nothing plays, and the next pick's draws appear at the following pass.
  *Falsified if:* the pick comes at a later pass than the completion, its draws come before the
  pass's sound-choice draws, or after a failed start the pick is made in the same pass or later
  than the following one.

- **MU8. Start of music (claim 11).** The set-up above; the recording runs from the moment the
  skirmish is started in the menus, and the end of loading (the game view appearing) is marked
  in it. Here the first track heard is judged.
  - (a) MU1 part a's entries and options. *Observable:* the first three tones after loading.
    *Expected:* T0, T1, T4. *Falsified if:* the first tone is not T0, or T0 is not followed by T1.
    Second run: as the first, with T0's *normal* flag clear (test track supplied). *Expected:* the
    first tone is T1, then T4, then T1 (T0 is never heard). *Falsified if:* T0 is heard. Third
    run: as the first, with the repeat setting on. *Expected:* T0 over and over from the first
    tone. *Falsified if:* the second tone differs from the first.
  - (b) First shuffle pick, with the random-state recorder and MU6's accounting of unrelated
    draws. MU6 part a's entries and options (T0 to T2 available, T3 and T4 with *normal* clear).
    For the first tone, predict from the state recorded before the tick in which it starts the
    ranged draws *R*(0, 4) up to the first that names T0, T1 or T2, with no draw rejected for
    naming an ended track. *Expected:* the stream advances by exactly the predicted raw values
    and the predicted entry is heard. Make at least 10 runs. *Falsified if:* in a run that is not
    void the advance or the entry differs from the prediction, for instance because a draw naming
    an available entry is rejected as if that entry were the ended track. A run in which that
    alternative gives the same prediction does not tell them apart and is only counted for the advance.
  - (c) A new skirmish. MU1 part a's entries and options. In the first skirmish, after at least
    one full cycle, leave the game while T1 plays, and start a second skirmish in the same session
    without leaving the program. What is heard between the two games is only recorded. Let *L* be
    the last track the recording shows starting before the second game's loading (T1 unless a
    track starts between the games). *Expected:* the first tone after the second game's loading
    is T0. *Falsified if:* it is *L*'s list-order successor and not T0, which an ended track kept
    from before would give. *Inconclusive if:* *L*'s successor is T0 itself (*L* is T4).
  - (d) Map theme. MU1 part a's entries and options, on a test copy of the map whose theme key
    names T3 (whose *normal* flag is clear). *Expected:* the first tone is T3, then T4, then T0,
    T1, T4, … *Falsified if:* the first tone is not T3, or T3 is followed by any track but T4.
    Second run: the theme key names T3 in another case (lower case for upper). *Expected:* as
    the first run. Third run: the theme key names a text that matches no entry. *Expected:* the
    first tone is T0, as in part a. *Falsified if* (third run): T3 or any track other than T0 is
    heard first. Fourth run: as the first, with the shuffle setting on and the random-state
    recorder reading the unsynchronised stream, with MU6's accounting of unrelated draws.
    *Expected:* the tick in which T3 starts adds no music draw to the stream (no ranged draw
    *R*(0, 4)), and the first ranged draws for music come at the pick after T3 ends. *Falsified
    if:* a ranged draw is made for music in the tick in which T3 starts.
  - (e) In-game music off. MU1 part a's entries and options, with the in-game music key set off
    in the settings file, by a value that S-0003 claim 2 recognises (not `off`, which it does not), before the program
    starts; the setting is not changed in the game. *Expected:* no tone during the game, for at
    least five track lengths after loading. *Control:* part a, just before, in which music is
    heard. *Falsified if:* any tone is heard. Second run: as the first, with part f's added
    loading-track entry and its test track. *Expected:* the loading track's tone during loading,
    then no tone during the game. *Falsified if:* no loading tone is heard while part f's run with
    the same entry, on the same machine, hears it; or a tone is heard after loading.
    *Inconclusive if:* part f's run hears no loading tone either (loading too short).
  - (f) Delay. In parts a and d, and in a further run as part a with an added entry whose
    identifier is the loading track's name (Import mapping) and whose test track has its own
    tone, record the time from the end of loading to the first tone, and whether the loading
    track's tone is heard during loading and how it ends. This part records the original's delay
    and fade (Open questions); no claim predicts their length. Further run, for their order: as
    the run with the added entry, with that entry's test track longer than loading lasts, so that
    it still plays at the end of loading. *Expected* (claim 11): its tone fades and has ended before
    the first tone after loading starts. *Falsified if:* the first tone after loading starts while
    the loading track's tone is still heard, or the loading tone is cut off at once with no fade.
    *Inconclusive if:* the loading tone has already ended when loading finishes.
  - (g) Loading track and the first pick. A test music database with entries T0, T1, *D* and T2
    in that order, where *D*'s identifier is the loading track's name (Import mapping) and *D* has
    its own test track and tone with its *normal* flag set, so that it is also an available track.
    Shuffle and repeat settings off. *Observable:* the tones during loading and the first two
    tones that start after the end of loading. *Expected:* *D* during loading, if loading lasts
    long enough to hear it; then T0, then T1. *Falsified if:* the first tone that starts after
    loading is T2, the list-order successor of *D*, which an ended track kept from the loading
    track's start would give, or is *D*. Second run: as the first, with *D*'s *repeat* flag set.
    *Expected:* the first tone that starts after loading is T0. *Falsified if:* it is *D*, which
    repeat of a kept ended track would give. A tone of *D* still fading at the end of loading does
    not count as a start.
  - (h) Side and settings. Part a's first run repeated with the local player on a country of
    each other side, and once more with the starting credits, the short-game option, the
    superweapons option and the crates option each changed from part a's values. *Expected:* the
    first tone is T0 in every run. *Falsified if:* any run's first tone is not T0.
  - Engine fixture: parts a to e, g and h on our engine, and a check that the first start is made at
    the music update of the game's first audio pass, with the loading track of part f stopped when
    the game starts. A further run with the switch off and no available entry (MU4 part a's
    entries with every entry unavailable) checks that the first pick starts nothing, that music
    started begins all the same, and that a pick is made again at each later music update with no
    ranged draw.
- **MU9. Settings when the settings file does not give them (Q-1831).** MU1 part a's entries.
  - (a) The settings file with its shuffle and repeat keys removed. *Expected:* the cycle T0,
    T1, T4 of MU1 part a over at least 12 tracks: list order (shuffle off) and no track twice in a
    row (repeat off). *Falsified if:* the order is not that cycle, or a track follows itself.
  - (b) Both keys present with the value `on`. Second run: both with the value `2`.
    *Expected:* as (a) in both runs (a value that is not recognised keeps the default).
    *Falsified if:* a track follows itself, or the order breaks the cycle.
  - (c) No settings file: it is moved out of the game directory before the program starts and
    put back afterwards; any settings file the program has written is removed after the run,
    before the original is put back. No option is changed in the game during this run, an
    exception to the set-up: the music, effect and voice volumes stay at their defaults, and each
    test track's steady tone is told from the game's own sounds by its pitch (set-up), so the run
    gives the program no cause to write a settings file, or to read one, before the run ends. The test
    music database and test tracks are as in MU1 part a. *Expected:* music is heard during the game
    (in-game music on) and the cycle is that of (a). *Falsified if:* no music during the game, or
    the cycle of (a) breaks, unless the run is inconclusive. *Inconclusive if:* no music at all is
    heard in the session, menus included: the default music volume, which Q-1700 question 5 leaves
    open, may then be inaudible. A repeat whose test music database also holds a menu entry (as
    MU11's) gives that control: the menu track heard in the main menu shows the default volume is
    audible. Last run: the settings file present, with the in-game music key set
    to `off`, which is not recognised. *Expected:* music is heard, as in (a).
- **MU10. Sound files that are present but do not play (claim 8a).**
  - (a) MU6 part a's set-up, with the random-state recorder and MU6's accounting, except that
    T2's test track is replaced by a file of the same name that does not parse. One run for each
    such file: a text file of a few hundred bytes; a RIFF WAVE file with format code 2; one whose
    data chunk comes before its format chunk; one with no data chunk; one with a valid header and
    no sample data after it. *Expected,* in every run: each pick is predicted with T2 as an
    available track, and holds; a pick that accepts the draw naming T2 is followed by silence and
    by a new pick at a later music update, whose draws are predicted with T2 as the ended
    track; T2's tone is never heard and the program keeps running. *Falsified if:* a pick's advance
    matches a prediction in which T2 is not available (a draw naming T2 rejected) and not the
    prediction with T2 available, or the program stops. A run in which no pick names T2 within
    20 picks is inconclusive. The engine fixture also expects the importer warning naming T2.
  - (b) A failed start of a present file becomes the ended track. MU1 part a's entries, with T1's
    *repeat* flag set and T1's test track replaced, before the program starts, by the text file
    of part a. *Expected:* after T0, silence for at least five track lengths: T1 is available, is
    picked, fails, becomes the ended track and its *repeat* flag starts it again at each pick
    (claims 4 and 8a). *Falsified if:* T4 is heard after T0, which a pass-over of T1 would give.
    The silence would also follow if the failed start stopped music updates altogether, so a
    second run supplies a playable test track for T1 during the silence, as MU4 part b's third
    run supplies entry 0's. *Expected:* T1's tone is heard at a later music update, within five
    track lengths of the moment the playable track is supplied. *Falsified if:* the silence lasts
    more than five track lengths after the playable track is supplied.
    As MU1 part l, this part is read only after MU4 part b's third run has met its
    expectation; until then it is inconclusive. Read with MU1 part l, where a missing file is passed over, it
    shows that presence depends on the file being found, not on its content.
  - (c) Data cut short. MU1 part a's entries, with T0's test track written as uncompressed PCM
    and its sample data cut to the first half while its header still gives the full data size.
    *Expected:* T0's tone lasts about half as long as the other test tracks', and T1 follows after
    the ordinary gap. *Falsified if:* a silence of about half a track's length follows T0's
    shortened tone before T1, which would mean the track ends at the size the header gives.
    Second run: T0's test track with sample data of a second, different pitch appended after the
    data the header gives (the file's overall RIFF size updated, the data chunk's size not).
    *Expected:* the second pitch is never heard and T0's tone lasts as long as the other test
    tracks'. *Falsified if:* the second pitch is heard.
- **MU11. The end of the game, the score screen, the options menu and leaving (Q-1833; claims 11 to 13).**
  MU1 part a's entries and options, plus two entries after T4: *S*, whose identifier is the score
  track's name, and *M*, whose identifier is the menu track's name (Import mapping), each with its
  own test track and tone and with its *normal* flag clear, so that neither is picked in play
  (claim 2) and each part also tests the direct start. The recording runs through the score
  screen and back to the main menu. The tick of the end of the game and the end delay's final
  tick (the last tick before the tick count stops: the game is over) are marked in it by the
  measurement harness; the opening of the score screen, each opening and closing of the options
  menu and the opening of the main menu are marked by the harness or from a screen capture
  aligned with the audio.
  - (a) Win. A test copy of the map on which the computer opponent is quickly beaten (for
    example with a single unit of its own beside the local player's force). Win while a tone
    plays. *Observable:* the tones from the end of the game to the opening of the score screen, the
    tone's envelope at that opening, and the first tone on the score screen. *Expected:* the
    tone playing at the end of the game goes on at full level, with no fade, through the end
    delay and the wait after the game is over, until the score screen opens; it stops there at once, within the recording's resolution, and *S* starts from its
    beginning. If a tone ends in the end delay or in the wait, the next one is its list-order
    successor (claim 12). The end delay's length in ticks, the wait's length in real time and the
    moment each tone ends are recorded. If no tone ends within the end delay, the run is repeated
    with the win given, by the tick count, a little less than the end delay's length (S-0414
    claim 8, from the rules in force) before a tone ends. If no tone ends within the wait, a
    further run gives the win so that a tone is due to end just after the end delay's final tick,
    and if none ends there either, the wait's part is reported unreached. *Falsified if:* the tone
    stops or fades at the end of the game or when the game is over, before the score screen
    opens; a tone that ends in the end delay or the wait is not followed by its successor; it is
    still heard after the score screen opens; or the first tone on the score screen is not *S*.
  - (b) Score track's end. As (a), staying on the score screen for at least three of *S*'s
    lengths after it starts. *Expected:* *S* once, then silence. Second run: repeat setting on.
    Third run: repeat setting off, *S*'s *repeat* flag set. *Expected* (second and third runs):
    *S* over and over. *Falsified if:* any tone other than *S* is heard on the score screen, *S*
    is heard twice in the first run, or silence follows *S* in the second or third run.
  - (c) No score track. As (a), without *S*. *Expected:* the tone stops at once when the score
    screen opens, and the score screen stays silent for at least three track lengths.
    *Falsified if:* any tone is heard on the score screen.
  - (d) In-game music off. As (a), with the in-game music key set off in the settings file, by a
    value that S-0003 claim 2 recognises (not `off`, which it does not).
    *Expected:* no tone during the game (MU8 part e) and *S* heard when the score screen opens.
    *Falsified if:* *S* is not heard on the score screen.
  - (e) Leaving through the options menu. As (a) on the unchanged map: while T1 plays, open the options menu and leave the
    game through it. The moment leaving is confirmed, the screens shown after it and the moment
    the main menu opens are marked in the recording, and the read-only recorder gives the last
    tick that runs. *Expected:* no tick runs after leaving is confirmed, so no object of the local
    player's is destroyed and no result announcement is given; no score screen opens, and *S* is
    never heard; the program goes to the main menu. T1 starts to fade when leaving is confirmed
    (its envelope falls over a stretch of time rather than at once), no other test tone starts
    before the main menu opens, and *M* starts when it opens. *Falsified if:* the local player's
    objects are destroyed or play resumes after leaving, the score screen opens or *S* is heard,
    T1 plays on at full level after leaving, or another test tone starts before the main menu.
    The length of T1's fade and of the stretch from leaving to the main menu are recorded.
  - (f) Options menu. MU1 part a's entries and options: open the options menu about one second
    before a tone ends and keep it open for at least two track lengths. *Expected:* that tone
    sounds to its end without a break or a fade, and the next tone, its list-order successor,
    starts after the ordinary gap while the menu is still open, and then the one after it.
    Second run: as the first, with the shuffle setting on, the random-state recorder reading the
    unsynchronised stream across the menu's opening and closing. *Expected:* a tone different from
    the one that ended starts in the menu; any advance of the stream in that stretch is recorded
    for the Open questions. *Falsified if:* silence follows the tone's end until the menu
    closes, or the tone breaks off or pauses when the menu opens.
  - (g) Score screen to the main menu, and on into a game. *M*'s test track lasts at least three
    minutes. After (a), leave the score screen for the main menu, then start a skirmish from the
    menus as soon as they allow, while *M* still sounds. *Expected:* *S* stops at once when the
    main menu opens and *M* starts; the tone playing when the second game starts to load, *M* or
    any other menu tone, stops at once then, with no fade (its envelope drops within the
    recording's resolution), and since the test music database has no loading track, loading is
    silent. *Falsified if:* *S* is heard after the main menu opens; the tone playing when loading
    starts fades out rather than stopping at once; or *M* or any menu tone is heard during the
    second game's loading. If no tone sounds when loading starts, the run is repeated
    with a longer *M*; if none sounds then either, the stop at loading is reported unreached.
    What plays in the menus between the main menu and the start of loading is otherwise only
    recorded.
  - (h) The menu's choices. In (e)'s run, before leaving, the options menu's choices are captured
    on screen, down to the confirmation that leaving brings up. *Expected:* no choice restarts
    the game and none surrenders while play goes on; the one way to leave is that of (e).
    *Falsified if:* the skirmish's options menu offers a restart or a surrender.
  - (i) Loss. As (a), on a test copy of the map on which the local player is quickly beaten (for
    example with the local player's only unit beside the computer opponent's force), the loss
    given while a tone plays. *Observable* and *Expected:* as (a), with the loss in place of the
    win: the tone playing at the end of the game goes on at full level, with no fade, through the
    end delay and the wait after the game is over, until the score screen opens; it stops there
    at once and *S* starts from its beginning. A tone that ends in the end delay or the wait is
    followed by its list-order successor; unlike (a), no further runs are made to place a tone's
    end there, and the moment each tone ends is only recorded. *Falsified if:* as (a).
  - Engine fixture: parts a to e, g and i on our engine with its end screen in place of the score
    screen, expecting the original's results except where claims 12 and 13 give our engine's design:
    in part a music updates and picks go on through the end delay as in the original, the end screen
    opens at the end of the cycle that runs the end delay's final tick, the track stops there at
    once, and no music update runs after that cycle (a track due to finish after it is followed by
    no pick); in part b's second and third runs the score track starts again as soon as its whole
    length has played out, with no audio pass on the end screen, and a further run of part b with
    the score track's file absent, once without and once with the repeat setting, expects a silent
    end screen and records our engine's attempts to start the score track, expecting exactly one
    attempt in each run; in part e the game ends at once, with no later music update and no pick,
    and the main menu starts *M*. A further run, on a three-player test copy of the map whose two
    computer opponents are hostile to each other, gives a surrender order, where our engine's
    interface offers one, expecting play to go on after the local player's defeat, since a hostile
    pair remains, with the music as in play; and one restarts, where our engine offers it, expecting
    the restarted game's first tone to be T0 (claim 11). Part f on our engine, with the test driver
    supplying the menu's clock readings as logical input (as S-0701 H2 does) and a log of each
    music-only pass's music time, expecting the original's result of part f (claim 13): the tone
    sounds on to its end while the menu is open, and its successor starts in the menu at the first
    music-only pass at which the tone completes on music time; in the second run the ranged draws of
    each shuffle pick made while the menu is open come at the music-only pass at which the track
    before it completes, however many picks the menu's stretch holds, and no draw other than those
    picks' is made while the menu is open. The same run with our engine's own settings screen opened
    from the game in place of the options menu, expecting the same results.

## Changes for other specs

In a skirmish, the original's options menu gives the game up only by aborting the mission, which
ends the game at once and does not send the surrender order of S-0414 claim 11 (claim 13). The
hand-offs below therefore move every run of the original that gives the human's surrender, in
S-0414 and in other specs, to a network game of the original, and correct approved text that
assumes a skirmish surrender. They rest on claim 13, which is rated medium until MU11 parts e and
h are made; part h's capture shows the labels of the menu's choices, so a run of part h that finds
a choice that surrenders in a skirmish withdraws them. A network-game run backs a skirmish rule
only on the assumption that the original carries out the surrender order, and ends the game after
it, in the same way in a network game and in a skirmish. A network game's end handling is left to
the future multiplayer spec (S-0201 claims 13 and 15), no run of the original can test that
assumption, and it is rated low (the S-0414 entry below). So each rating of those specs that
comes to rest on a network-game run alone drops to low, unless a skirmish run of the original
also backs the same rule, in which case that run's rating stands. The reason for the move: in a
skirmish the menu's leave choice ends the game with no tick after it, and no choice of that menu
marks the house to die (claim 13, MU11 parts e and h), so a run that left through the menu would
end the game at once; no other input of the original's skirmish is known to give a surrender, and
scripted input acts only through the game's normal input (glossary, *measurement setup*).

- **S-0701 claim 30.** Two changes. First, its schedule of audio passes gains the music update of
  S-0703 claim 10, run at the end of each pass after that pass's sound choices. Second, its sentence "No skirmish audio passes occur in setup menus before loading
  or after leaving the match." becomes "No skirmish audio passes occur in setup menus before
  loading, after the cycle that runs the end delay's final tick (the game is over, S-0414 claim
  8), or after leaving the match." The end delay's
  ticks need no change: they run in ordinary cycles of the loaded skirmish, so the claim's
  schedule already gives their audio passes, with their music updates (the first change). The
  pass at the end of the cycle that runs the end delay's final tick, if one falls there, is the
  last; our end screen opens after it (S-0703 claim 12) and has none. The score track and the end screen need no audio
  pass, since they make no ranged draw. S-0701 carries both changes. It also adds music-only
  passes while the options menu is open (S-0701 claim 30): the menu stays a stand-still for
  sound, but music time advances there with real time and each music-only pass runs one music update, so the music plays on in the menu (S-0703 claim 13).
  Lost focus stops music time there too. It also lists the glossary amendment for this
  exception (S-0701, Changes for other specs).
- **S-0009.** A map's basic section also carries the map theme key, which S-0703 claim 11 and
  its Import mapping interpret.
- **S-0202 claim 11 and scenario R6 part b.** No change: the music track choice they name is
  S-0703 claim 6's shuffle pick, drawn from the unsynchronised stream.
- **S-0414 claim 11, its Summary, its Measurement scenarios and the scenarios that give a
  surrender.** In a skirmish the original's options menu sends no surrender order: its one way to
  leave, aborting the mission, is how a skirmish player gives up, and it ends the game at once
  (S-0201 claim 1, with its entry below, and S-0703 claim 13); no other skirmish input that gives
  the order is established. The original's surrender order is a network game's (S-0201 claim 15,
  with its entry below), so every run of the original that needs the human's surrender becomes a
  network game of the original, in a form that this entry adds to S-0414's Measurement
  scenarios. Such a run measures a network game of the original, whose end handling (the end
  test, the end delay, the final tick and what is committed in it) is among the network-game
  states that S-0201 claims 13 and 15 leave to the future multiplayer spec. So what it backs holds
  for a skirmish only on the assumption that the original carries out the surrender order in the
  same way in a network game and in a skirmish, and that assumption is rated low: no run of the
  original can test it. Each S-0414 rating that comes to rest on such a run alone therefore drops
  to low, as listed below, unless a skirmish run of the original also backs the same rule (as V2's
  defeat from another cause may), in which case that run's rating stands; ratings that skirmish
  runs back keep theirs. The changes are as follows.
  - Summary. The sentence "Campaign missions (whose results come from map triggers) and network
    games are outside this spec." gains: "; a network game of the original is used only to
    measure the surrender (Measurement scenarios, *network-game form*)". A paragraph is added
    after the first: "In a skirmish the original's options menu sends no surrender order: its one
    way to leave ends the game at once (S-0703 claim 13). So every run of the original that gives
    the human's surrender (V4, V8 part a, V11, V12 parts a and b, V14, V15 part a, V17) is given
    in a network game of the original, in the *network-game form* of the Measurement scenarios.
    A rating that rests on such a run alone is low, as the assumption that the order's effect does
    not depend on the kind of game is (claim 11), unless a skirmish run also backs the same rule
    (claims 5, 11, 15 to 18 and 21). The form rests on S-0201 claim 15."
  - Claim 11. The sentences "In the original only the local player can surrender, through the
    options menu, and not when the local player is an observer, nor from the end of the game on:
    the original refuses the options menu from then on (S-0201 claim 15), and in a skirmish no
    tick runs while it is open, so a skirmish player cannot surrender in the end delay." become:
    "In the original's network game only the local player can surrender, through the options
    menu, and not when the local player is an observer, nor from the end of the game on: the
    original refuses the options menu from then on (S-0201 claim 15). In a skirmish the
    original's options menu sends no surrender order: its one way to leave, aborting the mission,
    is how a skirmish player gives up, and it ends the game at once (S-0703 claim 13 and S-0201
    claim 1). No other skirmish input that gives the order is established here; the original's
    network game gives it (S-0201 claim 15). This spec assumes that the original carries out the
    surrender order in the same way in a network game and in a skirmish, nothing in carrying it
    out depending on the kind of game. On that assumption what the order does is measured in a
    network game of the original (Measurement scenarios, *network-game form*), and our engine
    gives the order that effect in a skirmish." The sentence "in the original the human
    participant's surrender ends the game at that defeat (claim 6, departure)" becomes "in the
    original no skirmish input that gives the surrender order is established, and claim 6's
    departure ends a skirmish at the human participant's defeat from any other cause". In the
    backing, "since no skirmish input can give it" becomes "since no skirmish input that gives it
    is established", and the backing gains: "That the order's effect does not depend on the kind
    of game is `unmeasured`: no run of the original's skirmish is known to give the order, and the
    network-game runs measure the effect itself." The confidence's "high for the destruction in
    the next house update" becomes "low for the destruction in the next house update, which V8
    measures only in the network-game form, unless a skirmish run also backs it", and the
    confidence gains "low that the effect does not depend on the kind of game, an assumption that
    no run of the original can test (unmeasured)". The pending-result case, its backing and its
    rating need no change.
  - Measurement scenarios. After the first paragraph (the common setup) a paragraph is added:
    "*Network-game form.* A scenario that gives the human's surrender in the original is run as
    a network game of the original, since no input of the original's skirmish sends the
    surrender order (claim 11). Two machines on one local network, as in S-0201 scenario T8,
    each run the stock game and the recorder; the scenario's human is machine 1's player, and
    the scenario's houses, map, game speed (set in the lobby) and named test files are otherwise
    as stated. Machine 2's player is an added human participant, far from the observed area, in
    one lobby team with every computer player of the scenario, all of which are the human's
    enemies wherever this form is used; if the lobby offers no such team, the run is not made,
    and what it would back is `unmeasured`. Machine 2's house is a contender created for a human
    participant and remains, so claim 6's departure does not end the game; the human's defeat
    leaves no hostile pair, and the game ends at it by claim 6's ordinary test (no hostile pair
    remains), assuming the original's network game ends as a skirmish does. How the original
    ends a network game is left to the future multiplayer spec (S-0201 claims 13 and 15), so a
    rating that rests on a run in this form alone is low, as claim 11 rates that assumption,
    unless a skirmish run also backs the same rule. Machine 2's player gives no orders except the
    one below; a run in which machine 2's house is defeated, attacks or is attacked is void.
    Machine 1's player surrenders through the options menu's choice that surrenders and keeps
    the player watching (S-0201 claim 15), by scripted input where a tick is aimed at. The order
    is carried out in tick *c* + *W*, where *c* is the tick in which the choice is made and *W*
    is the order delay (S-0201 Terms and claim 17); wherever a scenario names the tick of the
    surrender, or the tick in which it is applied, it means that tick. That tick is recorded,
    not computed: it is the one tick at whose end machine 1's read-only recorder records the
    human's house as marked to die (claim 11 and State and ordering), so no reading is needed
    after the surrender. *W* serves only to aim the choice: each run reads it once, before the
    surrender, from a move order that machine 2's scripted input gives one of its own units (the
    tick in which the unit's destination changes, less the tick in which the order is given), and
    a run whose surrender is carried out in a tick other than the one a scenario aims at is void,
    so no rule here rests on the order delay being the same for every order. The surrender comes
    at least 600 s of real time after the map is visible, as in S-0201 scenario T8, so that
    timing orders have been carried out. The draws that a network game adds to the scenario
    random stream in every tick (S-0202 claim 12) are not part of any count of draws below."
  - Scenario V4. "The human surrenders" (the setup) and "when the human surrenders" (part b)
    are given in the network-game form; part c reads the kill counts on machine 1's result
    screen, machine 2's house among the houses whose count must not rise. Claim 5's confidence
    "high for the destruction, the ignored defences, the first two table rows and the
    permanent-control rows" becomes "high for the destruction (V2) and the permanent-control rows
    (S-0601 scenario MC10); low for the ignored defences and the first two table rows where they
    rest only on V4, which runs in the network-game form, unless a skirmish run also backs them".
  - Scenario V8. The setup "the human surrenders from the options menu at tick *t*" becomes "the
    human surrenders in the network-game form, the order carried out in tick *t*". Part b needs
    no change: it is a run of our engine only, with S-0201 scenario T8, part H as the
    original's counterpart. Claim 11's rating changes as given above.
  - Scenario V11. "Procedure: surrender" (the main run, also used by part b's "during
    surrender"), part c's "Surrender while P is still held" and part d's "Surrender and record"
    give the surrender in the network-game form. The main run gains: "Before the surrender, at
    least 100 ticks with no death are recorded, so that the network game's own draws of each
    tick (S-0202 claim 12) are known; a draw in the tick of the destruction that is neither one
    of those nor a draw of the sweep makes the run inconclusive." Claim 15's confidence "high for
    creation order, no off-map exclusion and no repeated hit solely due to delayed removal"
    becomes low for the same, since V11 measures them only in the network-game form, unless a
    skirmish run also backs them.
  - Scenario V12, parts a and b. "surrender in tick n − 1" becomes "surrender in the
    network-game form, the order carried out in tick n − 1"; the control with an ordinary
    vehicle likewise. Claim 16's confidence "high for the relative places of destruction, power,
    radar and tech-tree work" becomes low for the same, since V12 parts a and b measure them,
    and the final tick that commits them, only in the network-game form, unless a skirmish run
    also backs them. Claim 8 keeps its rating: its backing (V12 part c and V6) gives no
    surrender.
  - Scenario V14. "Procedure: surrender and sample during the destruction interval" becomes
    "Procedure: surrender in the network-game form and sample during the destruction interval".
    Claim 17's confidence "high for the health-independent count and deletion rule" becomes low
    for the same, unless a skirmish run also backs it.
  - Scenario V15, part a. "Apply surrender in tick 7" becomes "Surrender in the network-game
    form, the order carried out in a tick 8*m* − 1 for a whole number *m* (machine 1's choice
    made in tick 8*m* − 1 − *W*, with *W* from the run's reading); a run whose order is carried
    out in another tick is void". In the rest of part a, "tick 8" becomes "tick 8*m*" and
    "tick 16" becomes "tick 8*m* + 8", in the end-delay sentence and in the counter run, which
    also surrenders in this form. Part c's comparison "unlike surrender in part a" needs no
    change. Claim 18's confidence "high for the relative places and continued raises after
    defeat" becomes "high for what V15 parts b and c measure in a skirmish; low for the
    surrender's place relative to the assigned-tag raise and the raises after that defeat, which
    part a measures only in the network-game form, unless a skirmish run also backs them".
  - Scenario V17. "No-attacker surrender gives losses only" gains "(the surrender given in the
    network-game form; machine 2's house gains no kill)". Claim 21's confidence gains "low for
    the counts after a surrender, which V17 measures only in the network-game form, unless a
    skirmish run also backs them".
  - Claim 5's backing ("V4 does not arrange an object under an erasing weapon at surrender"),
    the rule text of claims 15 to 18 and State and ordering describe the order's effect rather
    than how it is given, and need no change.

  Reason as in this section's opening paragraph.
- **S-0313 (the defeat variant of its firing-index scenario), S-0308 (scenario part g, own-house
  defeat) and S-0508 (scenario G15).** "surrender the human house through ordinary input",
  "surrender the occupants' house" and "surrender A through normal input" become a surrender of
  that house, the human's, in the network-game form of S-0414's Measurement scenarios (the entry
  above); the house that receives the building and the third contender are computer players of
  machine 2's lobby team. A rating of these specs that comes to rest on such a run alone drops to
  low, unless a skirmish run also backs the same rule. Reason as in this section's opening
  paragraph.
- **S-0311 claim 15 and S-0601 claim 14.** S-0311 claim 15's confidence "high for whole-building
  actions, defeat's off-map inclusion and the engine safety rule" keeps high where X15 or S-0508
  G14 back the rule and for the engine safety rule; a part that rests only on S-0508 G15 (the
  defeat hits with an unchanged numeric index in a surviving foreign-owned building) drops to low,
  unless a skirmish run also backs it, since G15 gives its surrender in the network-game form
  (the entry above). S-0601 claim 14's confidence "high" keeps high where S-0414 V2 variant j or
  MC10 backs the rule in a skirmish; a part that rests only on S-0414 V4 parts a and b drops to
  low, unless a skirmish run also backs it, since V4 gives its surrender in the network-game form.
  Reason as in this section's opening paragraph.
- **S-0109 (the death fixture's after-pass surrender destruction).** The surrender is given in the
  network-game form of S-0414's Measurement scenarios. A rating that comes to rest on that run
  alone drops to low, unless a skirmish run also backs it. Reason as in this section's opening
  paragraph.
- **S-0413 (scenario C3, part a).** "at the end of a game that the human ends by resigning at tick
  300" becomes "at the end of a game that ends with the human's defeat at a fixed tick: a timed
  action of the test map removes the human's last counted object (as in S-0414 scenario V12,
  part c), the skirmish ends at that defeat (S-0414 claim 6) and the score screen opens". Reason:
  in a skirmish the options menu's only way to give up, aborting the mission, is no resignation: it
  ends the game at once with no score screen (S-0703 claim 13), so the score screen this scenario reads would not
  open. A network game is not used here, since its score screen would also list machine 2's
  house.
- **S-0203 (the measurement note "Assigned-tag raises within a house update (claim 8)").** Its
  first sentence, "S-0414 V15 owns the shared measurement: a named test skirmish map, scripted
  input and a read-only recorder compare surrender in tick 7 with the assigned-tag raise in tick
  8, then use a turn-pass gate to make a tag remove prerequisites or the last qualifying object
  in tick 16.", becomes "S-0414 V15 owns the shared measurement: a named test map, scripted
  input and a read-only recorder compare a surrender carried out in a tick 8*m* − 1 for a whole
  number *m* with the assigned-tag raise in tick 8*m* (V15 part a, given in the network-game
  form of S-0414's Measurement scenarios), then use a turn-pass gate in a skirmish to make a tag
  remove prerequisites or the last qualifying object in tick 16 (V15 parts b and c)." The rest
  of the note needs no change: its ticks 16 and 17 belong to parts b and c, which give no
  surrender. Claim 8's backing (S-0414 scenarios V7, V8, V11, V12 and V15) needs no change, but
  its confidence "S-0414's high for destruction creation order and within-update ordering"
  becomes "high for what S-0414 scenarios V7 and V15 parts b and c measure in a skirmish; low for
  destruction creation order and within-update ordering where they rest only on V8, V11, V12 and
  V15 part a, which run in the network-game form, unless a skirmish run also backs them", since
  how the original ends a network game is left to the future multiplayer spec (S-0201 claims 13
  and 15). Reason as in this section's opening paragraph.
- **S-0011 claim 35.** Its backing "S-0414 V15 for within-update timing and continued raises
  after defeat" needs no change, but its confidence "high for within-update timing and continued
  raises after defeat" becomes "high for the within-update timing that S-0414 V15 parts b and c
  measure in a skirmish; low for the raise's place relative to the surrender destruction and for
  continued raises after that defeat, which V15 part a measures only in the network-game form,
  unless a skirmish run also backs them", since how the original ends a network game is left to
  the future multiplayer spec (S-0201 claims 13 and 15). Reason as in this section's opening paragraph.
- **S-0201 claim 1.** The sentence "A resignation counts here: it makes the player's house lose in
  a later tick, followed by the end delay, rather than ending play immediately." becomes "A
  resignation counts here: it does not end play immediately. What it does to the player's house is
  S-0414 claim 11's (it may change nothing, or bring the house's defeat in a later tick); a defeat
  it brings ends the game only through the end test and the end delay, as above." The bullets
  "**Left from the options menu.** When the player leaves the game through an options-menu choice
  that ends it at once, the game ends between two cycles and no further tick runs. The tick
  number stays at the count of ticks run, one more than the number of the last tick." and "Which
  of the menu's choices ends the game at once and which one resigns is not established (Open
  questions). Our engine reproduces both rules." become one bullet: "**Left from the options menu,
  in a skirmish.** In a skirmish the options menu offers one way to leave, and no choice of that
  menu resigns (S-0703 claim 13). Leaving through it ends the game at once, between two cycles,
  and no further tick runs. The tick number stays at the count of ticks run, one more than the
  number of the last tick. Our engine reproduces the ending by a rule of game state and this
  skirmish leave." In the backing, "`unmeasured` for the exact menu-choice mapping, since no
  button-to-result mapping has been established" becomes "`measurable` (S-0703 scenario MU11,
  parts e and h) for the menu's one leave choice in a skirmish, ending the game at once; claim 15
  gives a network game's choices", and the confidence's "low for which menu choice is which"
  becomes "medium for the skirmish menu's one leave choice, until S-0703 MU11 parts e and h are
  made". The Open questions entry on which menu choice is which is settled for a skirmish by
  claim 13. Reason as in this section's opening paragraph.
- **S-0201 claim 15.** Its sentence "After the player has resigned through the menu, it does not
  open again until the player's house is defeated;" becomes "In a network game, after the player
  has resigned through the menu, it does not open again until the player's house is defeated (in
  a skirmish the menu's abort is no resignation, S-0703 claim 13);". In *A surrender during the
  end delay*, "a skirmish player cannot surrender during the end delay" becomes "a skirmish player
  cannot surrender through the options menu during the end delay, nor at any other time: a
  skirmish's options menu sends no surrender order; its abort ends the game at once (S-0703 claim
  13)", and "choosing surrender in it then still sends the surrender order" becomes "choosing the
  surrender that keeps the player watching in it then still sends the surrender order". The
  backing's "`unmeasured` for the menu after resigning in a skirmish" with its reason, and the
  confidence's "medium for the menu after resigning in a skirmish", are dropped, since the case
  cannot arise in a skirmish. The claim gains a bullet after *A surrender during the end delay*:
  "**What leaving offers in a network game.** While at least two houses of human participants
  are undefeated, the player's own among them, and the player is not an observer, leaving
  through the options menu offers two choices that send the surrender order (S-0414 claim 11):
  one then leaves the game, the other keeps the player watching. Otherwise the menu offers only
  a leave, which sends no surrender order, and an observer is offered no surrender (S-0414 claim
  11, `unmeasured`, see the backing). Backing: `measurable` for the choice that keeps the player
  watching (part F of scenario T8, and the network-game form of S-0414's Measurement scenarios,
  in each of its runs), and for the condition on the undefeated human houses and the leave-only
  menu otherwise through three readings of scenario T8: part F's first reading (both human houses
  undefeated: two surrender choices), part F's further run (the opener's house undefeated, the
  other human house defeated: a leave only) and part D's capture (the opener's house defeated, the
  other undefeated: a leave only), with part F's three-machine run for a game of three human
  participants; `unmeasured` that the choice that surrenders and leaves sends the surrender order
  and then leaves the game, since no part takes it (T8 tells it from the other choice by its
  label only), and for what the two leave choices do on the other machines, since no part records
  them there; `unmeasured` that the leave-only choice sends no surrender order, since no part
  takes it; `unmeasured` that an observer is offered no surrender choice, since no part has an
  observer open the menu. Confidence: medium for the choice that keeps the player watching, and
  for the condition and the leave-only menu until those readings are made; low for what the
  choice that surrenders and leaves does, for what the two choices do on the other machines, for
  the leave-only choice sending no surrender order and for the observer's menu (unmeasured)."
  S-0414's network-game form rests on this bullet. Reason as in this section's opening
  paragraph.
- **S-0201 scenario T8, parts D and F.** Part F's first sentence, "machine 2's player resigns
  through the options menu, and the scripted input tries to open the menu once every 100 ms until
  it opens.", becomes "machine 2's player opens the options menu and leaves through the choice
  that surrenders and keeps the player watching, and the scripted input then tries to open the
  menu once every 100 ms until it opens." Its observable gains, first, "the leave choices the
  menu offers before the resignation (screen capture)"; its expected gains, first, "two choices
  that send the surrender order are offered, one leaving the game and one keeping the player
  watching"; and it is falsified also if "the menu does not offer exactly two choices that send
  the surrender order, one that leaves the game and one that keeps the player watching (a missing
  choice of either kind, or two of one kind)". It gains: "The kind of the choice that is not
  taken is read from its label only: this part does not take it, so what it does is not tested
  (claim 15, `unmeasured`). *Further run (the leave-only menu):* in the same game, after part D's
  reading, machine 1's player opens the options menu, machine 2's house now being defeated.
  *Observable:* the leave choices offered (screen capture). *Expected:* only a leave is offered.
  *Falsified if:* a choice that surrenders is offered." and, after its void-run sentence:
  "*Three-machine run:* if a third machine is available, a game of its own with a third human
  participant on machine 3, set up as machine 2. Machine 3's player leaves through the choice
  that surrenders and keeps the player watching; once machine 3's house is defeated, machine 1's
  player opens the options menu. *Observable:* the leave choices offered (screen capture).
  *Expected:* the two choices that send the surrender order, as in part F's first reading (two
  human houses, machine 1's among them, are undefeated). *Falsified if:* only a leave is offered.
  Without a third machine the run is not made and the condition's case of three or more human
  participants stays unconfirmed." Part D's observable gains "and the leave choices it offers
  (screen capture)", its expected gains "and offers only a leave (claim 15: the opener's own house
  is defeated)", and it is falsified also if "a choice that surrenders is offered". Reason as in
  this section's opening paragraph.
- **S-0201 scenario T1, part D.** "and leave the game, trying each menu choice that leaves the game
  in its own runs" becomes "and leave the game through the menu's one leave choice of a skirmish
  (claim 1, S-0703 claim 13)"; "*Runs:* 2 per choice" becomes "*Runs:* 2"; the expected "at least
  one choice ends the game with neither value changing after the menu closes" becomes "the choice
  ends the game with neither value changing after the menu closes"; the sentence "A choice that
  resigns lets ticks run on through the defeat and its end delay, and its final tick then behaves as
  in part C." becomes "A choice that resigns, which only a network game offers (claim 15), is
  outside this part."; and "*Falsified if:* every choice that leaves the game lets at least one more
  tick run after the menu closes and ends as in part C" becomes "*Falsified if:* the leave choice
  lets at least one more tick run after the menu closes". Scenario T9 needs no change. Reason as in
  this section's opening paragraph.

## Open questions

- (Claim 11.) How long the original's fade of the loading track lasts, and so how long after the
  end of loading the first track starts; MU8 part f records it. The delay also depends on how
  often the music update runs (Q-1700 question 2).
- (Claim 11; Q-1700 question 4.) What changing the in-game music setting during a session does.
- (Claim 11; Q-1700 question 5.) What the original starts first when the music volume is zero
  while a game loads with a loading track in the music database: the start of claim 11 may then
  give way to the loading track once the volume is raised. No scenario here sets that up.
- (Claim 1, Import mapping.) Whether a track identifier listed again in another case is the same
  entry in the original, or a separate entry with no section of its own: MU1 part c's second
  run gives the same cycle either way.
- (Claim 2, Import mapping.) The original's text limit for a side restriction; this spec gives
  127 bytes, tested for our engine only (MU1 part j), since the original's side rule is open.
- (Claim 3.) Under list order, whether an entry that is not available is passed over at the pick
  or started, failed and followed by a new pick at the next music update; MU1 part l proposes a test that
  tells them apart once MU4 part b's third run is made (claim 3).
- (Q-1700 question 8, part; claim 9.) With an empty track list: what the original does once a
  skirmish has loaded (MU5 records it).
- (Q-1700 question 2, part.) The gap between tracks: how often the music update runs. MU1 records
  the gap.
- (Q-1700 question 2, part.) Where the original's music update falls among the sound choices made
  in the same stretch of real time on the unsynchronised stream (S-0701 claim 30), before them or
  after them. A variant of MU6 in which a sound choice is made just as a track ends, with the
  random-state recorder, would tell the two orders apart.
- (Q-1700 question 4.) The options screen's music controls; which in-game controls change the
  shuffle and repeat settings, and whether a change is saved to the settings file.
- (Q-1700 question 4, part.) When the shuffle and repeat settings are read from the settings
  file: once at startup or again later. No scenario here separates the two (Terms).
- (Claim 6.) Whether the original can accept the thousandth ranged draw of a shuffle pick, as
  against our engine, which never accepts it. The two answers differ only when it would be
  accepted, with probability at most (1 − 1/*n*)<sup>999</sup>, so no scenario here tells them
  apart.
- (Q-1700 question 3, part.) Malformed values of the normal flag, the repeat flag and the side
  restriction. This spec applies S-0003's rules for their parse forms (Import mapping); no
  scenario here supplies one.
- (Q-1700 question 5.) How the volume setting scales the output; whether music is quieter under
  announcements or effects; what a volume change to zero does to a track already playing; which
  in-game setting gives a music volume of zero, and whether the original then makes no pick and
  starts no track (claim 3, MU6 part c).
- (Claim 1a.) With a file named by the extension alone mounted, whether an entry whose sound file
  name is empty, absent or made only of `$` and `#` finds that file and so has its file present.
- (Q-1700 question 6, part.) Victory, defeat, the score screen and the options menu are claims
  12 and 13. Still open: changes on a superweapon, on a pause of the game other than the options
  menu, and on minimising the game window, and music
  started or changed by a trigger action that names a music theme (S-0011): how such a start
  interacts with the next-track rules here.
- (Claim 12.) How long the original's real-time wait for the result announcement lasts after
  the game is over, before the score screen opens; MU11 part a records it.
- (Claim 12; Q-1700 question 5.) What the score screen plays when the music volume is zero as it
  opens, and once the volume is raised there.
- (Claim 11; Q-1830 question 2, part.) Whether a game mode's own data can give a map theme, and
  so choose the first track, in place of or on top of the map's; no scenario here sets one.
- (Claim 13.) How long the original's fade of the playing track lasts when the game is left
  through the options menu, and the length of the real-time bound on the wait that follows. MU11
  part e records both.
- (Claim 13.) Where a pick made in the original's options menu falls on the unsynchronised stream,
  among other draws; MU11 part f's second run records the advance.
- (Q-1700 question 7, part.) Music in the menus does not carry into a game: it stops when a game
  starts to load (claim 11), and the main menu starts the menu track (claim 12). Still open: what
  the other menus play, and how the menu track continues there.
- (Claim 8a.) A found sound file whose format chunk declares a very large size, which the original may not survive, even at startup; and a file cut through the middle
  of the last block of a compressed file. No scenario supplies either.
- (Claim 2.) Whether a track entry's side restriction that names a side the rules data does not
  have changes anything a player can see outside a game, for example the sides offered in a
  menu or their order, is not established. No claim here says that it does.
- (Q-1700 question 8, part.) Whether music plays at all without the theme archive mounted.
- (Claim 2.) The rule by which the original matches a side restriction to a side, which may not
  be by name (MU1 part b tests the name), including whether a restriction written as the first
  side's name makes its entry available.
- (Terms, Import mapping.) What a track entry's display name and campaign scenario number do
  outside a skirmish: where the name is shown, and whether the number limits the entry in a
  campaign.
- (Edge cases, claim 13; settled.) The project lead has accepted the end screen opening when
  the game is over (Edge cases). The project lead has also accepted that the music plays on
  while the options menu is open, as in the original (claim 13). Neither is still open.
