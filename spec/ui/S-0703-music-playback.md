---
id: S-0703
title: Music playback (track eligibility, next-track order, shuffle and repeat)
topic: ui
revision: 34
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-23
measurements: []
requests: [Q-1700]
supersedes: []
---

# S-0703: Music playback (track eligibility, next-track order, shuffle and repeat)

## Summary

Which entries of the music database can play in a skirmish, and which track follows when one
ends: list order, shuffle, repeat, the fallback when nothing can play, and missing sound files.
This is a partial answer to Q-1700. How music starts, the options screen, volume scaling, game
events, menus and the gap between tracks are open (Open questions). Music is presentation: no
game state depends on it.

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
  files (claim 8). The scenario parts that back claims 2, 5 and 6 (MU1 parts a to g, i, j and l,
  MU2, MU4 parts a and c, MU4 part b's first and second runs, and MU6) supply or withhold each
  test file for the whole run; MU1 part k supplies no test track and uses the theme archive's own
  files. MU1 part h, MU3 parts d and e and MU4 part b's third run change files during the run and
  back claims 4 and 8. When presence is decided (at startup or later) is claim 8's, rated there:
  medium for a file supplied after the program has started, until MU1 part h runs, and for a
  file removed after startup from an entry with the *repeat* flag, until MU3 parts d and e run,
  and low for a file removed after startup from an entry without that flag.
- **available track.** In a skirmish, a track entry that is file present, has its *normal* flag
  set and whose side restriction passes claim 2's side-restriction rule.
- **ended track.** The track entry most recently started by a pick of claims 4 to 7; the next pick is made relative to it. Whether the start of the music sets it is open (claim 4).
- **music update.** The periodic check at which, once the playing track has finished, the next
  track is picked and started. When it runs and when a track has finished are claim 10's.
- **music started.** The state in which music updates make picks (claim 10).
- **shuffle setting, repeat setting.** The two player choices that change the next-track rule
  (claims 4 to 6). Both are read from the settings file (Import mapping); no
  scenario here separates a read at startup from a later one. Which in-game controls change
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
   files (claim 8). The sound file name is given without that extension (a name that already
   ends in it is open, Open questions). An entry whose sound
   file name is empty or absent never has its file present, and neither has one made only of `$`
   and `#` characters, since removing them leaves the name empty, provided that no file named by
   the extension alone is mounted. With such a file mounted, the outcome is not established.
   - Backing: `measurable` (MU1 part e for the removed characters, with prefixes of one, two and
     more characters in mixed orders, and its last run for the same characters elsewhere in the
     name being kept; for the extension, the test tracks of every scenario, each
     a file whose name ends in the extension while its entry's name does not; MU1 part g for an
     empty or absent name and, in its third run, a name made
     only of those characters, while no file named by the extension alone is supplied).
     `unmeasured` for such a name when a file named by the extension alone exists, which no
     scenario supplies.
   - Confidence: high, except medium for an empty or absent name and a name made only of those
     characters, which in the original rest on a file named by the extension alone never being
     found.

2. **Availability in a skirmish.** Claims 5 and 6 pick only available tracks. A track entry is an
   available track while its sound file is present (claim 8), its *normal* flag is set, and its
   side restriction is "any side" or matches the side of the local player's country (how a
   restriction is matched is given below). The campaign
   scenario number plays no part in a skirmish. Two rules can still start a track that is not
   available: repeat (claim 4) and the fallback (claim 7). S-0701 claim 30 schedules skirmish
   audio passes only in a loaded game, and menu music is outside this spec (Open questions), so
   this rule covers skirmish playback only. Our engine matches a side
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
   - Confidence: high for the *normal* part, for the scenario part and for our engine's name
     matching; medium for the file part (only tracks whose files are present are picked), as
     claims 3 and 8 rate the pass-over, until MU1 part l runs; medium for a restriction written
     as the name of a side other than the first not matching in the original.

3. **When the next track is picked.** Once music has started (Terms; how it starts is open,
   Q-1700 question 1), each time the playing track finishes the next track is picked at the next
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
   - Confidence: high for no silent gap of a track's length, and for the pass-over at the pick
     under shuffle of an entry whose *normal* flag is clear, the case MU6 part a tests; medium
     for the pass-over at the pick under shuffle of an entry with no section of its own, until
     MU6 part a's third and fourth runs are made, and of an entry whose named sound file is
     missing at startup or whose side restriction does not match, which no draw-level scenario
     includes; medium for the
     pass-over at the pick under list order, until MU1 part l is made; low for zero volume in the
     original (`unmeasured`: which in-game setting gives a music volume of zero, and so whether
     the original makes no pick there, is open, Q-1700 question 5); high for our engine's
     zero-volume rule, which our engine chooses and its fixture checks.

4. **Repeat.** When there is an ended track and its entry has its *repeat* flag set, or there is
   an ended track and the repeat setting is on, the next track is the ended track again, whatever
   the shuffle setting and whether or not it is still available, and no ranged draw is made.
   Otherwise, including when there is no ended track, claim 5 or claim 6 picks the next track. A start whose file could not be opened still leaves its entry
   the ended track (State and ordering), so repeat starts it again. What a start leaves does not
   depend on the entry's *repeat* flag or on the repeat setting: whether its file opens or not, the
   started entry becomes the ended track and music stays started (claim 10), with either flag and
   either setting; repeat acts only at the next pick. The first track heard is the ended
   track only if whatever starts the music leaves it so (Q-1700 question 1), so the scenarios
   judge repeat from the second track heard on.
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
     run shows only that a choice made after the file is supplied starts it. `measurable` (MU3 parts d and e) for repeat of a track whose file
     is deleted during play, with named test files changed during the run (Measurement
     scenarios)
   - Confidence: high; medium for a failed start making its entry the ended track, until MU4 part b's second and
     third runs are made, for repeat starting a failed track again, until MU3 parts d and e
     are run, and for the *repeat* flag and the repeat setting playing no part in what a start
     leaves (`unmeasured`, as the backing says); medium for a new failed start at each repeat
     choice (`unmeasured`, as the backing says: it is what claim 4's start rule gives at each
     choice, and no observation separates it from choices that make no attempt)

5. **List order (shuffle setting off).** The next track is the first available track found by
   stepping forward through the list from the entry after the ended track, wrapping from the last
   entry to entry 0. The ended track itself is the last candidate of the cycle, so with no other
   available track it plays again if it is available. If there is no ended track (no pick of
   claims 4 to 7 has started a track, and whatever starts the music left none, claim 4; whether
   a new game restores this is open), stepping begins at entry 0. If no entry is available, claim 7 applies. A
   list-order pick makes no ranged draw.
   - Backing: `measurable` (MU1 part a) for the order and the wrap; `measurable` (MU4 part a)
     for a list with no available entry; `measurable` (MU6 part e) for no ranged draw.
     `measurable` (MU4 part c) for the ended track as the last candidate, with a single available
     track other than entry 0
   - Confidence: high for the order, the wrap and no ranged draw for entries without a side
     restriction (for entries with one, as claim 2 rates the original's side rule); medium for
     the ended track as the last candidate, until MU4 part c is made
   - The start at entry 0 when there is no ended track: `unmeasured`, low, because every
     scenario judges from the second track heard and what starts the music is open (Q-1700
     question 1), and whether a game start resets this is not traced

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
   0's sound file can be opened it plays, even when its *normal* flag is clear or its side
   restriction does not match. If it cannot, there is silence and the pick is made again at the
   next music update. Consequences, which apply only when claim 4 does not select the ended
   track:
   - With list order and no available entry at all, entry 0 is started again and again; each
     start plays only if its file can be opened (claim 8).
   - With shuffle and only one available track *k*: if *k* is entry 0, it plays again and again.
     Otherwise *k* and entry 0 alternate, with entry 0 silent if its file is missing, so *k* is
     heard again after that brief failed start, which makes entry 0 the ended track (claim 4).
     This holds unless entry 0 has its *repeat* flag set, which keeps entry 0 playing (claim 4),
     and except where a pick after entry 0 again
     accepts no draw (claim 6), which starts entry 0 once more.
   - Shuffle can also stop without accepting a draw while other tracks are available; claim 6
     gives the bound. Entry 0 is then started in the same way.

   With the switch off (our engine's design for that setting):
   - List order with no available entry: no track starts and no ranged draw is made. The pick is
     made again at each music update, and the music stays silent: file presence is fixed at
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
     for list order, which can confirm it but cannot falsify it by silence, since whether music
     starts with no available entry is open (Q-1700 question 1); `measurable` for the switch off, by
     MU4's engine fixture, for list order with no available entry (part a) and for shuffle with
     the ended track the only available track (part b); `unmeasured`, our engine's design, for
     shuffle with no available track, which no fixture sets up with a non-empty list, and for the
     continuation past the thousandth ranged draw, which no fixture reaches; `unmeasured` for an
     entry 0 that is not available only because its side restriction does not match, which no
     scenario sets up (MU4 and MU6 give entry 0 no side restriction)
   - Confidence: medium for list order (MU4 part a is one-sided); medium for shuffle (the
     alternation rests on the pick rule of claim 6 applied twice and, with entry 0's file
     missing, on claim 4's failed start making entry 0 the ended track, which MU4 part b's second
     and third runs test, claim 4); high for the switch-off rules that MU4's engine fixture checks (list order
     with no available entry, and shuffle with the ended track the only available track), and
     medium for the other switch-off rules (shuffle with no available track, and the continuation
     past the thousandth ranged draw), which are `unmeasured` (our engine's
     design); medium for an entry 0 whose side restriction does not match (`unmeasured`)

8. **Missing sound files.** A track entry whose sound file is not present is never an available
   track, so claims 5 and 6 never pick it to be heard. In our engine they pass over it at the
   pick; whether the original does so, or starts it, fails and picks again, is claim 3's open
   alternative, which claim 3 rates for each setting. Presence counts the archives and loose files
   mounted when the program starts, the theme archive included (S-0005 claim 12). A file supplied
   after the program has started does not make its entry available, and a file removed after it
   has started does not make its entry unavailable. When the track of such a removed file is due
   to start and its sound file can no longer be opened (removing a loose file can uncover an
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

9. **Empty track list.** In our engine (our engine's choice, with either switch setting and
   either repeat setting), a music database that holds sections but no track entry (*n* = 0) gives
   silence throughout: no music update makes a pick and no ranged draw is taken. In the original no
   pick is made before music has started (Terms). Whether music can start with an empty list, and
   what the original does if it does, are unknown (Open questions). A music database that cannot be
   resolved, or that holds no section at all, stops the program from starting (S-0005 claim 16).
   - Backing: `measurable` for our engine (MU5's engine fixture). The original's behaviour after
     a start is open; MU5 records it. The rest is S-0005's.
   - Confidence: high for our engine's rule, which our engine chooses and its fixture checks
     directly; the original's behaviour after a start is not rated (open)

10. **Music update and music started.** How often the original's music update runs, and where
    its draws fall among the sound choices of S-0701 claim 30, are open (Open questions). Our
    engine's rule (design):
    - One music update runs at the end of every audio pass of S-0701 claim 30's schedule, after
      that pass's sound choices. The ranged draws of a shuffle pick (claim 6) therefore follow
      that pass's draws on the same unsynchronised stream. A pick that makes no ranged draw (list
      order, repeat) leaves the stream unchanged.
    - A track finishes at the first pass at which it would complete under S-0701 claim 30's
      completion rule for a sample with pitch percentage *p* = 100, its logical duration counted
      from the pass at which it started. The music update at the end of that same pass is the
      "next music update" of claim 3: it makes the pick and starts the track picked, whose
      logical duration then counts from that pass. A start whose file cannot be opened plays
      nothing, and the pick is made again at the update of the following pass (claim 8).
    - Music started begins when something starts the music, which is open (Q-1700 question 1).
      Before it, no music update makes a pick. It does not require that a track is heard: a start
      whose file cannot be opened still leaves music started, in the original and in our engine.
    - Backing: `measurable` for our engine's rules (MU7, an engine-only scenario). `measurable`
      (MU4 part b, second run) for the original's failed start leaving music started: there the
      one available track plays again after the fallback's failed start of entry 0, shown by the
      recorder's advance of 1000 ranged draws, only if that start leaves music started and leaves
      that track no longer the ended track (claim 4). MU4 part b's third run and MU3 part d also
      show it, by the recovery once the missing test track is supplied.
    - Confidence: high for our engine's schedule, which our engine chooses and MU7 checks
      directly; medium for the original's failed start leaving music started, until MU4 part b's
      second run is made.

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
| Picked entry's file opens | Start | It plays; it becomes the ended track for the next pick |
| Picked entry's file does not open | Start | Silence; it becomes the ended track (claim 4); the next music update picks again |
| No track picked (switch off, claim 7) | Music update | The pick is made again |
| Music volume zero | Music update | Original: no pick, nothing starts (claim 3, low). Our engine: as at any other volume, the track playing silently |
| Music not yet started (Terms: no start has happened yet, Q-1700 question 1) | Music update | No pick, whatever the track list holds |
| Music started, no track playing (a failed start or a track that finished) | Music update | A pick (claims 4 to 7), unless, in the original, the music volume is zero; with an empty list, none in our engine (claim 9) |

## Edge cases and quirks

- `quirks.music_fallback_entry_zero` (claim 7). Default on. The fallback is not a bug: data can
  rely on it (an entry 0 that is never available but
  plays whenever nothing else can). The switch-off behaviour is one of several reasonable
  answers, so a switch keeps the original's result selectable. Backing and confidence: as claim
  7 rates the fallback.
- **Picks at zero music volume (departure, no switch).** Our engine picks and starts tracks at
  every music volume, silently at zero (claim 3), where, by claim 3's reading of the original
  (`unmeasured`, low), the original makes no pick at zero volume. The music volume is a setting of each player, and a pick's ranged draws are on the
  unsynchronised stream that sound choices share, so a rule that skipped picks would make later
  sound choices depend on that setting. Nothing is heard at zero volume in either. When the
  volume is raised again, our engine is heard part-way through whatever track it is then
  playing; what the original then plays is open (Q-1700 question 5). Basis: the precedent of S-0701
  claim 30, which already keeps sound choices independent of each player's mute settings; this
  departure extends that rule to music, a design choice recorded here with that reason; the
  project lead's acceptance of it is pending (Questions for the project lead). Backing and confidence: as claim 3 rates them (the original's rule low and
  `unmeasured`, our engine's rule high through MU6 part c's engine fixture).
- **Side restriction matched by name (design choice of scope, no switch).** This is a design
  choice, not a departure from a known rule of the original, whose own rule is not established.
  Our engine makes an entry whose side
  restriction names the local player's side available to that player (claim 2). In the original
  such an entry does not by that become available when the side is not the first (MU1 part b),
  and for the first side the result is not established. Which entries it makes available instead
  is not established and is not reproduced. Only that failure to match by name is
  claimed about the original. Whether it is a bug is not: if the original's rule is meant to be by
  name, it is a bug that changes music only; if the rule is not by name at all (Open questions),
  the stock data may suit it, and this is a difference of reading the data. Giving it no switch
  is a design choice recorded here: the behaviour changes music only, and the original's rule is
  not established, so there is no known behaviour for a switch to select. The project lead's
  acceptance of it is pending (Questions for the project lead).
  Backing and confidence: as claim 2 rates them (our engine's name matching high, the original's
  failure to match medium).
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
  - Confidence: high for the warning and the never-available entry, which the fixture checks
    directly; medium for when the check is made

## Import mapping

Music database (S-0005 claims 17 and 18):

| Key (section) | Concept | Parse form | Default when absent |
|---|---|---|---|
| `[Themes]`, every value | track entry identifiers, in list order; the key names play no other part, except that each position's value is looked up by its key, so a key written more than once gives its winning occurrence's value at each of its positions (S-0002 claim 4, S-0004 claim 2); a key whose value is empty adds no entry, and the values `none` and `<none>` are ordinary identifiers here, each an entry like any other, so the special meaning that S-0004 claim 2 gives them in rules type lists does not apply. S-0003 claim 14's reserved names apply to references to rules types; a track entry is not a rules type, and the track list has no reserved name, so every nonempty value names an entry | string with a text limit of 31 bytes (S-0003 claim 1); in our engine compared ignoring case, a repeat being the same entry (claim 1; `unmeasured`, low for the original) | none (empty list, claim 9) |
| `Sound` (the entry's own section) | sound file name (claim 1a) | string with a text limit of 255 bytes (MU1 part i; S-0003 claim 1 gives how a value is cut); then any leading `$` and `#` characters are dropped; the extension `.wav` is added; an empty value is treated as absent | empty: the file is never present, so the entry is never available, within claim 1a's limits |
| `Name` | display name, whose use is not established here (Q-1700 question 4, open) | not used here | not used here |
| `Normal` | *normal* flag (claim 2) | boolean (S-0003) | true |
| `Repeat` | *repeat* flag (claim 4) | boolean (S-0003) | false |
| `Side` | side restriction (claim 2) | a side name with a text limit of 127 bytes (S-0003 claim 1), matched ignoring case; empty counts as absent (MU1 part b, third run) | any side |
| `Scenario` | campaign scenario number; not used in a skirmish (claim 2); its role in a campaign is not established here | not used here | not used here |

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
  MU1 part g an identifier with no section of its own and an empty sound file name. `unmeasured`
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
| `[Audio]` `IsScoreRepeat` | repeat setting (claim 4) | boolean (S-0003) | not established (Open questions) |
| `[Audio]` `IsScoreShuffle` | shuffle setting (claims 5 and 6) | boolean (S-0003) | not established (Open questions) |

- Backing: `measurable`. Every scenario sets both keys before the program starts, and the
  expected sequences change with them: MU3 part b sets the repeat key (the same track over and
  over only with it on), MU2 parts a and b and MU6 set the shuffle key (no fixed cycle only with
  it on), and MU1 part a sets both off (the list-order cycle). The defaults when the keys are
  absent are not established (Open questions).
- Confidence: high for the two keys, their section and their boolean form.

## Measurement scenarios

All scenarios: stock data with no add-ons or mods, plus loose test files put in
the game directory before the program starts and removed afterwards. These are the measurement
setup's named test files. MU1 part h and MU3 parts d and e also delete or supply one of them while
the program runs (MU4 part b's third run supplies one), through the file system only and never through the game,
and the files are still removed after the run. The project lead ruled on 2026-10-22 that named
test files may be deleted or supplied during a run in this way. A
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
start of the next is noted for the Open questions. The first track heard may come from whatever
starts the music (Q-1700 question 1), so neither it nor the pick of the track after it is
judged (whether that start leaves an ended track is open, claim 4): every expected sequence below
is judged by which track follows which from the second track heard on, in whatever phase the
cycle starts.
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
    the music volume at its highest, shuffle and repeat settings off. *Observable:* the tracks
    heard over at least three track ends. *Expected:* the track after the first is a different
    one, as list order moves to the next available entry (claim 5). The stock tracks' files are
    found only in the theme archive, so a presence decision made before that archive is mounted
    would leave no track available, and then entry 0 would play again and again (claim 7).
    *Falsified if* the same track is heard four times in a row. *Inconclusive if* any
    track heard has its *repeat* flag set in the stock music database (claim 4), or fewer than
    two of its entries have their *normal* flag set and no side restriction (entries with a
    restriction are not counted, since the original's side rule is open, claim 2). If music does
    not start in a skirmish by itself (Q-1700 question 1), the part is reported unreached.
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
  track from the third heard on is the same as the second. Whether the second is the same as the
  first depends on whether what started the music left the first as the ended track (Q-1700
  question 1, claim 4), and is only recorded. *Falsified if:* a track after the second differs
  from the one before it. (c) Shuffle on: T0 with
  *normal* clear and its *repeat* flag set (test track supplied), T1 available, T2 with *normal*
  clear. *Expected:* T1 is followed by T0 (the fallback of claim 7), and once T0 plays only T0
  follows, although T0 is not available. *Falsified if:* T1 is heard again after T0. If the
  first track heard is T0, whatever started the music started it (Q-1700 question 1), and whether its
  *repeat* flag then keeps it playing depends on claim 4's open point: that run is inconclusive and is
  repeated; if every run starts with T0, part c is reported unreached. The engine fixture
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
  *Falsified if:* T0, once heard, is followed by silence at its end. Silence throughout is
  inconclusive, not a falsification: whether music starts at all with no available entry is
  open (Q-1700 question 1), so it is only recorded. Repeat with T0's test track also not
  supplied. *Expected:* silence throughout; this run only records the original's behaviour.
  *Control:* MU1 part a under the same options, just before, in which music is heard. (b) Shuffle
  on: T0 with *normal* clear (test track supplied), T1 available, T2 with *normal* clear.
  *Expected:* T1 and T0 alternate (a pick that misses T1 in all of its first 999 draws, with *n* =
  3 at most (2/3)<sup>999</sup>, is ignored). Repeat with T0's test track not supplied. *Expected:*
  T1 over and over, each separated by at most a short gap: the fallback's start of T0 fails and
  makes T0 the ended track (claim 4), so the next pick accepts T1. *Falsified if:* T2 is heard, T0
  never plays in the first run, or, in the second run, T1 is not heard again within five track
  lengths after it ends: then the failed start did not both leave music started (claim 10) and
  leave T1 no longer the ended track (claim 4), and the run does not say which of the two failed.
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
    three track lengths of play after T1 would have ended, counted from leaving the options (on
    the assumption that playback does not advance while the options menu is open; if the record
    shows that it does, count from entering the options), then set it back to its maximum.
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

## Changes for other specs

- **S-0701 claim 30.** Its schedule of audio passes gains the music update of S-0703 claim 10,
  run at the end of each pass after that pass's sound choices.
- **S-0202 claim 11 and scenario R6 part b.** No change: the music track choice they name is
  S-0703 claim 6's shuffle pick, drawn from the unsynchronised stream.

## Open questions

- (Q-1700 question 1.) What starts the music in a skirmish, when, and with which track, including
  whether music starts at all when no entry is available (MU4); whether that start leaves an
  ended track (Terms, claim 4), which decides the first repeat and the first list-order pick
  after it, and which every scenario avoids by judging from the second track heard on; whether
  having no ended track (claim 5) is restored by a new game.
- (Claim 1, Import mapping.) Whether a track identifier listed again in another case is the same
  entry in the original, or a separate entry with no section of its own: MU1 part c's second
  run gives the same cycle either way.
- (Claim 2, Import mapping.) The original's text limit for a side restriction; this spec gives
  127 bytes, tested for our engine only (MU1 part j), since the original's side rule is open.
- (Claim 3.) Under list order, whether an entry that is not available is passed over at the pick
  or started, failed and followed by a new pick at the next music update; MU1 part l proposes a test that
  tells them apart once MU4 part b's third run is made (claim 3).
- (Q-1700 question 8, part; claim 9.) With an empty track list: whether music can start, and
  what the original does if it does (MU5 records it).
- (Q-1700 question 2, part.) The gap between tracks: how often the music update runs. MU1 records
  the gap.
- (Q-1700 question 2, part.) Where the original's music update falls among the sound choices made
  in the same stretch of real time on the unsynchronised stream (S-0701 claim 30), before them or
  after them. A variant of MU6 in which a sound choice is made just as a track ends, with the
  random-state recorder, would tell the two orders apart.
- (Q-1700 question 4.) The options screen's music controls; which in-game controls change the
  shuffle and repeat settings, whether a change is saved to the settings file, and the settings'
  values when the settings file lacks their keys.
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
- (Q-1700 question 6.) Changes on game events (victory, defeat, score screen, a superweapon,
  pause, minimise),
  and music started or changed by a trigger action that names a music theme (S-0011): how such
  a start interacts with the next-track rules here.
- (Q-1700 question 7.) Menu music, and whether it carries into the game.
- (Q-1700 question 8, part.) A sound file that is present but does not decode.
- (Q-1700 question 3, part.) A sound file name that already ends in the extension: whether the
  file looked for gets the extension twice.
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

## Questions for the project lead

- (Edge cases.) Accept or reject the two design choices in Edge cases. Until the lead decides,
  both stand as this spec's design choices with their stated reasons.
  - Picks at zero music volume, a departure with no switch: our engine picks and starts tracks
    at every music volume, where the original, by claim 3's low-rated reading, makes no pick at
    zero volume.
  - The side restriction matched by name, with no switch: a design choice of scope, not a
    departure from a known rule. The tension to weigh: the original's
    failure to match a side's name (MU1 part b) is claimed, but its actual side rule is not
    established. So it cannot be said whether the original has a bug with no gameplay impact
    (which the project's rule lets go without a switch) or a different reading of the data that
    a switch could reproduce. Since the original's rule is unknown, a switch would have no known
    behaviour to select.
