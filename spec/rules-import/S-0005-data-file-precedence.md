---
id: S-0005
title: Data file precedence (loose files, archive mount order, name lookup)
topic: rules-import
revision: 3
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-09
measurements: []
requests: [Q-0002]
supersedes: []
---

# S-0005: Data file precedence (loose files, archive mount order, name lookup)

## Summary

When the original game asks for a data file by name (rules, art, sprites, models, palettes, sounds),
exactly one copy is used: a loose file in the game directory if there is one, otherwise the copy in
the highest-ranked mounted archive that holds the name. This spec gives that rank order, how a name
is matched inside an archive, and which copy of each stock rules, art and game-mode file the game
therefore reads; how those files are then layered is S-0006.

## Terms

*Archive* (and *nested archive*), *loose file*, *art database*, *base rules*, *game-mode file*,
*language rules file*, *override section*, *resolve*, *session*, *game*, *skirmish*, *lobby*,
*side*, *house*, *local player*, *settings file*, *sidebar*, *sprite*, *image frame*, *palette*,
*voxel model*, *theater*, *type name*, *infantry*, *vehicle*, *aircraft*, *building*, *overlay*,
*stock data*, *unspecified*, *measurement setup* and *measurement scenario* are used as in the
glossary. The kinds of art in claim 22 and Scenario M11 are those of S-0007, in its wording: a
building's main sprite and its other sprites (build-up, deploying, roof deploying, door, under-door,
under-roof-door, rubble, special overlay and bib sprites, S-0007 claim 11), sidebar icons,
secondary translucent images (S-0007 claim 23), motion files, theater-dependent sprites, a type's
image name, a sprite's first-choice name and the generic fallback name. *Placeholder icon* and an
art section's *theater flag* (Scenarios M3 and M11) are also used as in S-0007. Where S-0007 writes
*type id*, this spec writes the glossary's *type name*. S-0108 says when a building shows each of
its sprites.
Terms used only by this spec:

- **Expansion, base game.** *The expansion* is YR (glossary). *The base game* is *Command &
  Conquer: Red Alert 2*, of which YR is the expansion.
- **Member id.** The 32-bit number an archive stores for each member instead of its name (claim 6).
- **Mounted archive.** An archive the game has opened for lookups. Only members of mounted archives
  can be found. A nested archive's members become visible only when the nested archive is mounted
  itself.
- **Rank.** The position of a mounted archive in the search order. Rank 1 is searched first; a
  "higher" rank means "searched earlier".
- **Main archives.** The expansion's main archive `ra2md.mix` (the *main expansion archive*) and the
  base game's main archive `ra2.mix` (claim 10). Both are top-level files of the install.
- **Language archives.** The expansion's language archive `langmd.mix` and the base game's language
  archive `language.mix` (claim 9). Both are top-level files of the install.
- **Expansion archive.** A loose archive named `expandmdnn.mix`, where nn is two decimal digits.
- **Extra-cache and extra-local archives.** Loose archives whose names match `ecache*.mix` and
  `elocal*.mix` (any characters in place of the star).
- **Theater group, side group.** The archives mounted for the current map's theater (claim 14) and
  for the local player's side (claim 13).
- **String table.** The file of player-visible texts (object names, messages, menu texts) that the
  game loads once at startup.
- **AI, sound, speech and music databases; mode list.** The parsed contents of the expansion's AI,
  sound-effect, voice-announcement and music files, and of the file that lists the multiplayer and
  skirmish game modes. Their file names are in the Import mapping section.
- **Kept art.** Art for which the first successful request of a session settles what later
  requests in that session get, for the same name and for any name with the same member id, apart
  from the discarding that claim 22 describes (claim 22). It comes in two sorts, which claim 22 tells apart by the requested name and by the
  kind of art that makes the request.
  - A **kept file** is resolved and read once, and that copy is used for the rest of the session
    (except the four interface palettes of claim 22, which are discarded at every side-group mount).
  - A **deferred sprite** has its name fixed for the rest of the session by the first request,
    but its picture is read from the copy that resolves under that name when the sprite is first
    drawn in each game.
- **Read afresh.** Said of art that is resolved and read again at every request, whatever earlier
  requests of the session found (claim 22).
- **Short name.** The form of a file name that the original's host file system gives every file:
  at most eight characters before a single dot and at most three after it. A name already in this
  form serves as its own short name. For a longer name the host makes up a different short form,
  which this spec does not specify (claims 14a and 20).
- **Integrity digest section.** A section of a data file that holds a checksum of the file's
  contents, written as text. Claim 21 says when it decides whether the game starts, claim 21b how
  the checksum is formed and how it is written, and claim 21a that the AI database's section has
  no effect. Its section name is in the Import mapping section.
- **Interface layout file.** The expansion's data file that describes parts of the in-game
  interface, among them the buttons of the command bar. Its file name is in the Import mapping
  section. Claim 16a says when it is read.
- **Command bar.** The row of command buttons along the bottom edge of the screen during play. Which
  buttons it shows, and in which order, comes from the interface layout file (claim 16a).
- **Keyboard file.** The expansion's data file that binds keys to commands. Its file name is in the
  Import mapping section; claim 18 says which copy is used.
- **Map package.** An archive, kept as a loose file with the extension `yro`, that holds skirmish
  and multiplayer maps together with a description member listing them. Claim 14a says how the game
  finds and mounts map packages.
- **Side archive number.** The number nn in the names of the side group's archives (claim 13a).
- **Observing local player.** A local player who takes part in a multiplayer game only to watch,
  from the lobby's observer slot, and so has no country of its own.
- **Player entry.** The entry of a map's basic section that names the house the local player
  plays in a campaign mission. Its key is in the Import mapping section.
- **jumpjet or balloon-hover unit** (Scenario M11). A unit of a jumpjet type (glossary), or of a
  type whose rules section sets the flag that makes it hover under a balloon. S-0108 names both
  flags in its Import mapping and says that a war factory shows its roof deploying and
  under-roof-door sprites while it releases such a unit.
- **War factory** (Scenario M11). A building that produces vehicles and releases each newly built
  one from inside it.
- **Gate building** (Scenario M11). A building in a wall line that opens to let its owner's units
  pass and closes again. S-0108, claim 4, covers its gate stages.
- **Never reaches play.** Said of a game the player has set up and started: its map never appears
  and no tick of it runs. What the original shows instead (a message, or a return to the menus) is
  not specified.

## Behaviour

### Loose files

1. **A loose file wins over every archive.** Whenever a file is opened for reading and a loose file
   of that name exists in the game directory, that loose file is the copy used, whatever the mounted
   archives hold. The mounted archives decide only when no such loose file exists. This
   holds for every kind of data file, including nested archives (claim 3) and the rules, art and
   game-mode files. The test is made afresh at every open: a loose file added or removed while the
   game runs affects every later open. Some files are opened only once per session (claim 17), and
   claim 22 says when each kind of art is opened, so a change reaches them only at those moments.
   - Backing: `measurable` (Scenario M2 for the precedence, Scenario M15 for the test at every open)
   - Confidence: high (precedence), medium (the test at every open)

2. **Loose names match case-insensitively.** The loose-file test uses the host file system of the
   original, which ignores letter case. A loose file spelled `RulesMd.Ini` therefore matches a
   request for `rulesmd.ini`. An implementation on a case-sensitive file system has to match loose
   names without regard to ASCII letter case.
   - Backing: `measurable` (Scenario M8)
   - Confidence: high

3. **A loose or higher-ranked copy of a nested archive replaces the whole nested archive.** A nested
   archive is resolved by name like any other file (claims 1 and 7) at the moment it is mounted. If
   a loose file with the nested archive's name exists, or a higher-ranked archive holds a member of
   that name, that copy is mounted *instead of* the base copy, at the rank the nested archive has in
   Table 1. Nothing of the base copy stays visible: the two are not merged member by member.
   - Backing: `measurable` (Scenario M3)
   - Confidence: high

4. **Legacy extra search directories.** If a requested loose file is not in the game directory, the
   original also tries the same name in a short list of extra directories: the root of each disc
   drive, or a directory given on the command line. A loose file found there counts as loose. This
   exists for disc installs. Our engine looks for loose files in the game directory only. This is a
   design choice, not a quirk: disc installs are outside the project's scope, and a normal install
   never has files in those places, so no switch is proposed.
   - Backing: `unmeasured` (it needs a disc drive or a command-line launch, and it does not affect a
     normal install)
   - Confidence: medium

5. **Some archives are only ever taken as loose files.** Expansion archives (claim 9) and
   extra-cache and extra-local archives (claim 11) are found only as loose files in the game
   directory. A copy of one inside another archive is never mounted. The expansion's language
   archive `langmd.mix` is also found only as a loose file, because no archive is mounted yet when
   it is mounted (claim 9). The base language archive `language.mix` is not loose-only: when no
   loose `language.mix` exists, a member of that name inside `langmd.mix` is mounted in its place
   (claim 3). The settings file is likewise read only as a loose file. A missing settings file
   means defaults, never an archived copy.
   - Backing: `measurable` (Scenario M5 (b) for nested expansion, extra-cache and extra-local
     archives; Scenario M5 (c) for the settings file; Scenario M23 (a) for `langmd.mix`, Scenario
     M23 (b) for `language.mix` inside `langmd.mix`)
   - Confidence: high

### Lookup inside archives

6. **Member ids.** A requested name's member id is the public archive name-hash (Formulas) of the
   name with ASCII letters a–z upper-cased and all other bytes unchanged. The id is a 32-bit value,
   computed from the whole requested name.
   Two different names with the same id are the same file as far as the archives and the kept art
   of claim 22 are concerned. Requested names are at most 259 bytes long (Constants). For a longer
   name the original's result is unspecified; our engine treats such a name as not found in any
   archive.
   - Backing: `measurable` (Scenario M8 for the id and the upper-casing, Scenario M21 for two names
     with the same id); `unmeasured` for names longer than 259 bytes (no data path in an ordinary
     install requests one)
   - Confidence: high (the id, two names with the same id), medium (the length limit)

7. **First-ranked archive wins.** Mounted archives are searched in rank order, and the first one
   whose index search (claim 8) finds the member id supplies the file. Lower-ranked copies are
   invisible. This includes any lower-ranked member that merely shares the id (claim 6). A member
   that an archive holds but whose index search misses counts as absent from that archive, and the
   search goes on to the next rank.
   - Backing: `measurable` (Scenarios M1, M3, M4, M5; Scenario M21 (a) for a member that merely
     shares the id)
   - Confidence: high

8. **Index search is by signed id.** Within one archive the member id is looked up by the bisection
   given in Formulas (Index search), which compares ids as *signed* 32-bit integers. If the index is
   sorted ascending by signed id, which is what the public format prescribes, every member is found.
   With an index in any other order, some members that are present may not be found: a member is
   found exactly when the bisection of Formulas reaches it. In particular, an index sorted by
   *unsigned* id that holds ids both below and at or above 2³¹ always misses at least one of two
   members: the one with the largest id below 2³¹, or the one with the smallest id at or above 2³¹.
   If one archive holds two entries with the same id, the one the bisection reaches is used.
   (`quirks.mix_signed_binary_search`)
   - Backing: `measurable` (Scenario M9)
   - Confidence: high

### Rank order of the archives

The rank order is the order in which the archives are mounted: each newly mounted archive goes to
the end of the search order. An archive that is released and mounted again therefore moves to the
end. Claims 9 to 14a list the order. Table 1 in State and ordering gives it in one piece.

9. **Language archives first, then the expansion archives.** The first two archives mounted, and so
   the two highest ranks, are the expansion's language archive `langmd.mix` and then the base
   language archive `language.mix`. If neither a loose file nor either language archive holds the
   string table, the game refuses to start (its main menu never appears), even when an expansion
   archive or any other archive mounted later holds it: the string table is read while these two
   are the only archives mounted, so no later archive can supply it. Next come the expansion archives
   that are present as loose files: of two of them, the one with the higher number ranks above the
   other (Formulas, Expansion archive order). All of these rank above the main expansion archive. A member at the top
   level of either language archive, such as the expansion's string table, the keyboard file and the
   nested sound and icon archives of `langmd.mix` and `language.mix`, therefore cannot be overridden
   by an expansion archive. Only a loose file can override it, and a member of `language.mix` can
   also be overridden by a member of the same name in `langmd.mix` (claim 7).
   - Backing: `measurable` (Scenarios M3, M4 (a), M16 (a) and (b) for the order and the keyboard
     file, Scenario M14 for the rank of `language.mix` below `langmd.mix` and above the expansion
     archives, Scenario M4 (b) for the expansion archives ranking above the main expansion archive,
     Scenario M16 (c) and (d) for the string table's rank, Scenario M16 (e) for the refusal to start
     when only a later archive holds the string table)
   - Confidence: high

10. **Then the main, cache, local, neutral and sound archives.** After the expansion archives come, in this order:
    `ra2md.mix`, `ra2.mix`, `cachemd.mix`, `cache.mix`, `localmd.mix`, `local.mix`, `ntrlmd.mix`,
    `neutral.mix`, and then one sound archive, `audiomd.mix`, or `audio.mix` only if `audiomd.mix`
    cannot be found. The sound archive is mounted only when the sound system starts. In stock data
    the cache, local and neutral archives are nested inside the main archives (the expansion's
    inside `ra2md.mix`, the base game's inside `ra2.mix`), and the sound archives inside the
    language archives (`audiomd.mix` inside `langmd.mix`, `audio.mix` inside `language.mix`); all
    of them are resolved through the archives already mounted (claim 3).
    - Backing: `measurable` (Scenarios M1, M5, M14 for the order; Scenario M22 (a) for the sound
      archive fallback); `unmeasured` for no sound archive without a working sound system (a game
      whose sound system started but plays nothing audible looks the same from outside as one whose
      sound system did not start, so a test cannot tell which case it ran)
    - Confidence: high (order, fallback), medium (no sound archive when the sound system does not
      start)

11. **Then the extra-cache and extra-local archives.** Every loose file matching `ecache*.mix` is
    mounted, then every loose file matching `elocal*.mix`. A matching directory entry is skipped if
    it is a directory, or if the host marks it as hidden, as a system file or as a temporary file.
    Within each pattern the original mounts the matches in the order the host file system lists
    them, which the original itself does not define, so two hosts can mount the same files in
    different orders. Our engine instead mounts the matches of each pattern in sorted name order
    (Formulas), on every host. This is a deliberate departure, not a quirk, and no switch is
    proposed: the result must not depend on the host. Every extra-cache archive ranks above every
    extra-local archive in both cases. These archives rank *below* all the archives of claim 10. A
    member that already exists in, for example, the local archives cannot be overridden by them.
    They rank above everything in claim 12 onwards.
    - Backing: `measurable` (Scenario M5 (a) and (b) for the rank of the extra-cache archives,
      Scenario M5 (f) for the rank of the extra-local archives, Scenario M5 (d) for the skipping
      of hidden, system and temporary entries, Scenario M5 (e) for the original's order among
      several matches); `unmeasured` for directories (a directory can never be read as an archive, and an
      archive that cannot be read is skipped silently anyway, claim 16, so skipping it has no
      observable effect)
    - Confidence: high (rank), medium (the skipped entries, the order among several matches)

12. **Then the remaining startup archives.** In this order: `conqmd.mix`, `genermd.mix`,
    `generic.mix`, `isogenmd.mix`, `isogen.mix`, `conquer.mix`, `cameomd.mix`, `cameo.mix`, the
    expansion maps archive `mapsmd03.mix` and a base maps archive `maps03.mix` (absent in stock),
    `multimd.mix`, one theme archive (`thememd.mix`, or `theme.mix` only if `thememd.mix` cannot be
    found), and one movie archive (`movmd03.mix`, or `movies03.mix` only if the first cannot be
    found). The number 03 in the maps and movie archive names is an engine constant (Constants,
    see also the open question on a change of disc).
    - Backing: `measurable` (Scenario M5 for the rank of `conqmd.mix`, Scenario M14 for the order
      within this claim, Scenario M22 (b) for the theme fallback, Scenario M22 (c) for the movie
      fallback, Scenario M22 (d) for `maps03.mix` being mounted when present)
    - Confidence: high

13. **Per-game archives go to the end.** During a session, these are mounted later and therefore
    rank below all the startup archives:
    - the loading-screen archives `loadmd.mix` and then `load.mix`. They are mounted when a game's
      loading screen starts and released again when that game has finished loading, so they are in
      the search order only while a game loads, never during play. Everything a game resolves while
      it loads can see them: its map file, the language rules file read for that game (claim 21),
      the archives of its theater group and side group, the interface layout file (claim 16a) and
      the art requested for the game's objects. (The base rules, the databases, the mode list and
      the game-mode files of claim 17 were read at startup and are not resolved again.) A deferred
      sprite (claim 22) is requested while the game loads but its picture is read only when it is
      first drawn, after the loading archives are released. So a deferred sprite that only the
      loading archives hold is never drawn in play, and one whose name another mounted archive
      also holds is drawn with that other archive's picture
      (`quirks.loading_archive_sprites_vanish`). Because they are mounted
      afresh for every game, they rank below a theater group left mounted from the previous game
      (claim 14) but above a theater group mounted for this game,
    - the theater group (claim 14), released and mounted again whenever a game starts on a theater
      different from the previous game's,
    - the side group: for the local player's side, `sidecnnmd.mix`, `sidecnn.mix` and `sidencnn.mix`,
      where nn is the side archive number of claim 13a. The group is released and mounted again at
      the start of every game, after the theater group. It therefore always ranks last, unless the neutral pair
      is mounted again after it (next paragraph).
    The neutral pair (`ntrlmd.mix`, `neutral.mix`) holds interface art. It can also be released and
    mounted again later in a session (for example around the menus), which moves it to the end.
    - Backing: `measurable` (Scenario M6 for the theater and side groups, Scenario M13 (a) for the loading archives being
      searched while a game loads, Scenario M13 (b) for their rank against a theater group mounted
      for this game and one left mounted from the previous game, Scenario M13 (c) for their release
      before play and the deferred sprites, Scenario M13 (d) for the order of the two loading
      archives, Scenario M17 (c) for the order inside the side group); `unmeasured` for when the
      neutral pair is mounted again (the moment is not known, Open questions, so no scenario can
      be timed to it)
    - Confidence: high (theater and side order, the loading archives searched while a game loads),
      medium (the release before play as seen through deferred sprites: it relies on no object
      being drawn before the release), low (when the neutral pair is mounted again)

13a. **Side archive number.** The side archive number depends only on the side index s of the
    local player's country: its registration index in the side registry (S-0004, claim 9). It is
    s + 1, except that s = 2 gives 2, the number of s = 1 (Formulas, Side archive number). It is
    written in decimal with at least two digits, with a leading zero below 10. The index is the one the country
    has once all rules layers of the game, the map layer included, have been applied (S-0006), so a
    game-mode or map override of that country's side field changes which side group is mounted.
    Which country counts:
    - In a skirmish or multiplayer game, the country the local player plays, as chosen in the lobby
      (a random choice has been turned into a real country by then). An observing local player
      counts as playing the country of registration index 0.
    - In a campaign mission, the country of the house that the map's player entry names. If no house
      of that name exists, s = 0 is used.
    - A country with no side at all (S-0004, claim 9 allows one) has no side archive number: no side
      group is mounted, and the game never reaches play.

    With stock data this gives 01 for the Allied side (index 0) and 02 for the Soviet side (index 1)
    and the Yuri side (index 2). These two numbers are consequences of the stock side order, not
    engine constants. A side of index 3 or higher asks for the archives numbered 04, 05 and so on.
    Stock data has none, so such a game never reaches play unless a loose file or an archive
    supplies `sidecnn.mix` (claim 16). In a skirmish or multiplayer game the side indices follow
    S-0004 claim 9's multiplayer order, the order in which the base rules' countries first name each
    side.
    - Backing: `measurable` (Scenario M7 (i) for the stock outcome, the Yuri side included; Scenario
      M24 (a) for a side of index 3, M24 (b) for a map override of the side field, M24 (c) for a
      country with no side, M24 (d) for a campaign mission); `unmeasured` for an observing local
      player (it needs a networked game with this machine as the observer, which the single-machine
      measurement setup cannot provide) and for a player entry that names no house (a mission
      without its player's house is not expected to be playable for other reasons)
    - Confidence: high (the formula, the index taken after the map layer, the country of a skirmish
      and of a campaign mission), medium (the observing local player, the country with no side, the
      player entry that names no house)

14. **Theater group.** For a theater with control name C, extension E, palette name P and tile-art
    name A (Constants, Table 2), the group is, in rank order:
    - `snowmd.mix` for the snow theater only (C ‖ "md.mix", and no other theater gets this archive),
    - C ‖ ".mix",
    - E ‖ ".mix",
    - P ‖ "md.mix",
    - A ‖ ".mix".
    If the new game uses the same theater as the previous one, the group is not mounted again and
    stays mounted at its position.
    - Backing: `measurable` (Scenario M6; Scenario M13 (b) for a group left mounted from the
      previous game)
    - Confidence: high

14a. **Map packages.** When the game builds its list of maps, every loose file in the game directory
    whose name matches `*.yro`, except one named `missions.yro` (letter case ignored), is mounted as
    an archive. Directory entries are skipped by the same rule as in claim 11 (directories and
    entries marked hidden, system or temporary). Each package names its maps in a description member
    whose name is the package's file name with its last three characters (the extension `yro`)
    replaced by `pkt`: `test01.yro` holds `test01.pkt`. A package without that member is still
    mounted, but contributes no maps to the list. For a package whose name is not a valid short
    name (more than eight characters before the dot, for example), the original uses the host's
    short form of the name instead, which this spec does not specify. The original mounts several
    packages in the host file system's listing order, as in claim 11; our engine mounts them in
    sorted name order (Formulas), the same deliberate departure as in claim 11. This happens once per
    session, during startup, after all the archives of claim 12 are mounted. The packages therefore
    rank below all startup archives and above every per-game archive of claim 13, except a neutral
    pair mounted again later (claim 13), which then ranks below them. A package added or
    removed while the game runs has no effect until the game restarts. Stock data includes several
    map packages, all loose in the game directory and all with names of at most eight characters
    before the dot. Map contents belong to S-0009.
    - Backing: `measurable` (Scenario M12 (a) for the startup-only mounting and for stock packages
      being present, Scenario M12 (b) for
      the rank, Scenario M12 (c) for the description member's name and the skipped hidden, system and
      temporary entries,
      Scenario M12 (d) for the `missions.yro` exclusion, Scenario M12 (e) for the original's order
      among several packages); `unmeasured` for long package names, whose
      short form comes from the host and is outside this spec, and for directories (reason as in
      claim 11)
    - Confidence: medium (mounted once at startup, rank), high (the description member's name, the
      `missions.yro` exclusion), medium (the skipped entries, the order among several packages)

15. **Base archives the expansion mounts.** Of the base game's archives, the expansion mounts the
    two top-level files `language.mix` and `ra2.mix`; from inside `ra2.mix`, `cache.mix`,
    `local.mix`, `neutral.mix`, `generic.mix`, `isogen.mix`, `conquer.mix`, the base theater
    archives, the base side archives and `load.mix`; and from inside `language.mix`, `cameo.mix`
    (these are the places stock data holds them). It mounts `audio.mix` (inside `language.mix`) and
    the top-level `theme.mix` only as fallbacks (claims 10 and 12). It never mounts the base
    multiplayer-maps archive `multi.mix` or the base maps archives `maps01.mix` and `maps02.mix`
    (but see the open question on a change of disc, which is not expected to happen). The movie
    fallback of claim 12 is `movies03.mix`, which stock data does not have, so the base movie archives `movies01.mix` and
    `movies02.mix` are never mounted. Base files that the expansion never asks for by name (the base
    game's own rules, art, AI, sound, speech, music and game-mode files, which have different names
    from the expansion's) are
    never read, even though their archives are mounted.
    - Backing: `measurable` (Scenario M7 (a) and (d) for the archives never mounted, Scenario M7 (i)
      and (j) for `conquer.mix`, `cameo.mix` and the side archives being found where stated,
      Scenario M6 (b) for the base snow-theater archives, Scenario M13 (a) (ii) for `load.mix`,
      Scenario M22 (a) and (b) for the two fallbacks, Scenario M14 for the rank of the mounted ones,
      Scenario M20 (c) for the base game's own files never being read)
    - Confidence: high

16. **Missing archives and files.** An archive that cannot be resolved is skipped without any
    message to the player, and the archives after it keep their relative order. Of the archives,
    only these are required: the game refuses to start if one of `conqmd.mix`, `conquer.mix`,
    `cameomd.mix`, `cameo.mix`, `mapsmd03.mix` or `multimd.mix` cannot be resolved, or if neither
    of the movie archives `movmd03.mix` and `movies03.mix` can be resolved (either one is enough).
    It also refuses to start if the string table cannot be resolved while only the language
    archives are mounted (claim 9), and if the art database, the sound database, the speech
    database or the music database cannot be resolved, or is resolved but holds no section at all (an empty file,
    for example). Claim 21 gives one more reason to refuse, the language rules file. A game whose
    local player's side archive `sidecnn.mix` (claim 13) cannot be resolved never reaches play,
    while games whose local player has a different side archive number (claim 13a) are unaffected. No game reaches play
    while the interface layout file cannot be resolved or holds no section at all (claim 16a).
    Three other files are not required, and their absence is not reported:
    - A missing base rules file is treated exactly like a base rules file that is present but empty:
      startup goes on with empty base rules. What empty base rules then lead to is outside this
      spec.
    - A missing AI database does not stop the game from starting: its main menu appears. Startup
      goes on with an empty AI database. What computer players then do is not specified (Open
      questions).
    - A missing mode list does not stop the game from starting: its main menu appears. The game
      then has no game modes. What the skirmish and multiplayer screens then offer is not specified
      (Open questions).
    - Backing: `measurable` (Scenario M7 (b), (c), (e), (f) and (j) for the seven archives and the
      art, sound, speech and music databases missing, Scenario M16 (e) (i) for the string table, Scenario M22 (c)
      for either movie archive being enough,
      Scenario M7 (n) for one of those four databases holding no section; Scenario M7 (i) for the side
      archive; Scenario M7 (g) for the interface layout file; Scenario M7 (k) for the base rules,
      Scenario M7 (l) for the AI database, Scenario M7 (m) for the mode list)
    - Confidence: high (that missing archives are skipped, that the seven archives, the string
      table and the art, sound, speech and music databases are required, one of those four databases without any section, the side archive),
      medium (the interface layout file without any section, the base rules treated as empty, the
      missing AI database), low (that the main menu appears without a mode list)

16a. **The interface layout file is read for every game.** The interface layout file, which among
    other things sets the buttons of the command bar, is resolved and read afresh at the start of
    every game, after the side group of that game is mounted (claim 13). It is not read at startup.
    Two consequences follow. A loose interface layout file added or changed while the game runs
    takes effect from the next game on. And a side archive can supply the file, but only when no
    loose file and no higher-ranked archive holds it; in stock data `localmd.mix` holds it, so with
    stock data it always comes from there (claim 18).
    - Backing: `measurable` (Scenario M17 (a) for the read per game, Scenario M17 (b) for the side
      archive supplying it)
    - Confidence: high (read per game, after the side group), medium (that a skirmish shows the
      command bar's buttons as the file lists them)

### Which copy of the stock data files is used

17. **Rules, art and the other databases are resolved once, at startup.** The base rules, the art
    database, the AI database, the sound, speech and music databases and the mode list are resolved
    and read once, while the game starts. So is every game-mode file the mode list names: each is
    read once, at startup, when the mode list is read, and is not read again, even by the first game
    played in that mode. Later changes to files on disk have no effect until the game restarts.
    Every one of these files is resolved by claims 1 to 16. None of them is looked for under a base
    game name.
    - Backing: `measurable` (Scenario M2 (a) for the base rules, M2 (b) and its variant (ii) for a game-mode file, M2 (c)
      for the art, sound, speech and music databases and the mode list, M2 (d) for the AI database,
      Scenario M20 (c) for the base game names)
    - Confidence: high (the base rules, the game-mode files, the art, sound, speech and music
      databases, the mode list), medium (the AI database: its observable depends on computer play,
      which involves random draws)

18. **Stock outcome.** With stock data and no loose data files, the base rules and the sound
    database come from the expansion archive `expandmd01.mix` (the 1.001 patch copies). Each of
    these files is also present in the local archive, but that copy is never read. The art database,
    the AI database, the speech and music databases, the mode list, every game-mode file and the
    interface layout file come from `localmd.mix`: no archive that ranks above it holds them under
    the names the expansion asks for. In stock data it is the only mounted archive that holds them. The
    keyboard file comes from the expansion's language archive `langmd.mix`.
    - Backing: `measurable` (Scenario M1 for the rules and sound databases, Scenario M20 (a) for the
      art, speech and music databases, the mode list, a game-mode file and the interface layout
      file, Scenario M20 (b) for the AI database, Scenario M16 for the keyboard file)
    - Confidence: high (all but the AI database), medium (the AI database: its observable depends
      on computer play, which involves random draws)

19. **Missing game-mode file.** The mode list names one game-mode file per mode. If that file cannot
    be resolved, the mode behaves as if the file were empty: the mode is still offered and the
    game-mode layer changes nothing. The other rules layers (S-0006) apply as usual.
    - Backing: `measurable` (Scenario M7 (h))
    - Confidence: medium

20. **Alternative rules picker.** At startup the game also looks for loose files whose names match
    `rulemd*.ini`. That pattern lacks the letter s after "rule", so the standard rules file never
    matches it. If one or more exist, the game shows a modal list before the main menu with the
    standard rules file (resolved as usual) first and the matching files after it. The original
    lists several matching files in the host file system's listing order, as in claim 11, which it
    does not define. Our engine lists them in sorted name order (Formulas) of their names in the
    game directory, the same deliberate departure as in claim 11, with no switch. The chosen file
    becomes the base rules. With no matching files the standard rules file is used without a prompt.
    This is a developer feature, and stock installs contain no matching files. For a matching name
    that is not a valid short name, for example one with more than eight characters before the dot,
    the original opens the file under the host's short form of the name, which this spec does not
    specify (as in claim 14a). (`quirks.rules_picker`: YR shows the list. The fixed behaviour ignores
    such files.)
    - Backing: `measurable` (Scenario M10 (a) for no list without matching files, M10 (b) for the
      list and its entries, M10 (c) for the chosen file becoming the base rules, M10 (d) for the
      original's order of several matching files); `unmeasured` for long names (the short form comes from the
      host and is outside this spec)
    - Confidence: high (the list, its first entry, the chosen file), medium (the order of several
      matching files)

21. **Optional language rules file.** If a file named `langrule.ini` can be resolved, it is read
    at startup, and it is resolved and read again each time the rules are applied for a game; each
    read is layered on top of the base rules (S-0006). If the file holds at least one section, it
    must carry a valid integrity digest section (claim 21b). At startup, if that section is missing
    or its checksum does not match, the game refuses to start (its main menu never appears). A file
    that holds no section at all, an empty file for example, is accepted without any check and
    changes nothing. Stock data has no such file. A file that fails the check only when a later
    game's rules are applied (one added or changed after startup) does not stop the game: S-0006
    claim 12 says what that game does (the file is ignored for that game, and so is its game-mode
    layer).
    - Backing: `measurable` (Scenario M18 (a) to (c) for the refusal and for the file without
      sections, M18 (d) for the layering of a valid file, M18 (h) for the read per game)
    - Confidence: high (the refusal), medium (the file without sections, the layering, the read per
      game)

21b. **How the integrity digest is formed.** The checksum covers the file as parsed, not the bytes
    as stored. It is the standard SHA-1 hash, 20 bytes long, of a fixed text form of the document
    that S-0001 and S-0002 make of this file on its own (a fresh load), with the integrity digest
    section left out (Formulas, Integrity digest). The stored checksum is the standard base64
    decoding of a text that the integrity digest section's entries give, one piece per entry, joined
    in entry order. An entry's piece is the value that a lookup of its key finds (S-0002), cut to its
    first 127 bytes, with the trailing whitespace of the cut text removed: bytes after the 127th byte
    of a value are ignored (Formulas). As long as no two of the keys are the same name (S-0002,
    claims 1 and 3), only their order matters, so the text may be split over any number of entries.
    The section is valid when the first 20 bytes that the text decodes to equal the computed hash.
    Consequences:
    - Whatever S-0001's parse leaves out of the document makes no difference: comments, blank
      lines, lines that create no entry, the whitespace its trimming removes, CR bytes, sections
      without entries and entries with an empty value. Whitespace that the trimming keeps (S-0001,
      claim 10) is part of the key or value and counts.
    - The order of the sections, the order of the entries within each section, and every byte of
      each section name, key and value (letter case included) do make a difference.
    - **Repeated names.** A key written twice in a section, and a section name that two non-empty
      sections share, are written out in full: every occurrence, in document order, like any other
      entry or section. Only one integrity digest section is taken out: the one that a lookup of its
      name finds (S-0002, claim 2). Any other section of that name is written out as an ordinary
      section, so its text is part of the checksummed text. If the integrity digest section itself
      names one key twice (or two keys that are the same name, S-0002, claim 3), both positions
      give the piece of the occurrence that a lookup finds (S-0002, claim 1), and the other
      occurrence's value is never read.
    - Applying this rule to the stock AI database reproduces the checksum stored in its own
      integrity digest section, although that section has no effect (claim 21a).
    - Backing: `measurable` (Scenario M18 (d) for a valid digest, M18 (e) for what makes no
      difference, M18 (f) for entry order, M18 (g) for a digest split over two entries, M18 (i)
      for the 127-byte cut and the removal of trailing whitespace after it, M18 (j) for repeated
      names), and `unmeasured` for the stock AI database's checksum (a property of the stock file
      itself, checked directly by reading it, which a game run cannot add to)
    - Confidence: high (the text form, SHA-1, base64, the comparison of 20 bytes, the 127-byte cut
      and the removal of trailing whitespace after it), medium (repeated names: a key written twice,
      two non-empty sections of the same name, two integrity digest sections, a key written twice in
      the integrity digest section)

21a. **The AI database's digest is not enforced.** Whether the AI database's integrity digest
    section is valid (in the sense of claim 21), wrong or missing makes no difference: the game
    starts and uses the file's contents in the same way. The stock AI database carries such a section, so an edited copy is used even though its
    checksum no longer matches.
    - Backing: `measurable` (Scenario M19)
    - Confidence: high (the game starts), medium (that the contents are used in the same way)

### Kept art

22. **Sprites and palettes: what is kept, what is read again.** Each kind of art is of one of three
    sorts (Terms): a kept file, a deferred sprite, or art read afresh. The table says which sort each
    kind is.
    - **Kept art.** The table says which kinds of art are kept art; the others are read afresh.
      For kept art, a later request of the same name, or of any name with the same member id
      (claim 6), gets what the first successful request of the session got. A request whose name
      was not found settles nothing, and the name is looked up again at the next request. Whether
      kept art is a deferred sprite or a kept file is decided by the requested name and kind: a
      request made for an overlay or an animation, or one whose file name contains ".shp" (letters
      in any case), gives a deferred sprite; every other request for kept art gives a kept file.
    - **Kept file.** The copy found at the first request is used for the rest of the session, even
      if the archives or the loose files change.
    - **Deferred sprite.** The first request fixes, for the rest of the session, the name under
      which the sprite's picture is read. That name must resolve at the request, or the request
      settles nothing. The picture itself is read from the copy that resolves under that name when the sprite
      is first drawn in a game, and that picture is used for the rest of that game. At the start of
      every skirmish or multiplayer game the pictures of all deferred sprites are discarded, so each
      such game reads them again, from whatever copy resolves then. Whether a campaign mission does
      the same is not established (Open questions). If the name does not resolve when the sprite is to be drawn,
      nothing is drawn for it, and the next drawing tries again.
    - **Read afresh.** The kind is resolved and read at every request (for most kinds, once per
      game, when the game's objects get their art: S-0007).

    In the Confidence column, "(c)" rates the sort as seen through Scenario M11 (c) (whether a file
    added between two games shows), and "(d)" the split between deferred sprites and art read afresh
    as seen through Scenario M11 (d), which is medium throughout because it relies on no object being
    drawn before the loading archives are released.

    | Kind of art | Sort | Confidence | Scenario |
    |---|---|---|---|
    | sprites of overlays and animations, whatever the file's extension; sprites, named with ".shp", of infantry, vehicles and aircraft drawn as sprites, and of terrain objects and smudges that are not theater-dependent | deferred sprite | high (c), medium (d); medium (c) for aircraft and terrain objects (stock data has no example, so the test sets one up) | M11 (c) and (d), rows 1–3, 15–18; M11 (a), M11 (e) |
    | a building's main sprite and build-up sprite, named with ".shp" | deferred sprite | high (c), medium (d) | M11 (c) and (d), rows 4, 5 |
    | sidebar icons; secondary translucent images | deferred sprite | high (c), medium (d) | M11 (c) and (d), rows 6, 7 |
    | a building's special overlay sprite | deferred sprite | high (c), medium (d) | M11 (c) and (d), row 19 |
    | a building's bib, under-door and under-roof-door sprites under the generic fallback name of S-0007, used when the first-choice name is not found | deferred sprite | high (c), medium (d) | M11 (c) and (d), rows 13, 22, 24 |
    | the palette "unit" ‖ E ‖ ".pal" loaded with each theater group (E as in Table 2) | kept file | high | M11 (c), row 8 |
    | the other palette loaded with each theater group, C ‖ ".pal" (C as in Table 2) | kept file | medium | unmeasured: its use in drawing is open in S-0103, so no picture is known to show it |
    | a sprite named with a theater extension instead of ".shp" (S-0007), requested for a kind other than overlays and animations | kept file | medium (c), medium (d) | M11 (c) and (d), row 25 |
    | the four interface palettes named below | kept file, discarded at every side-group mount | high | M11 (b) |
    | voxel models; motion files | read afresh | high | M15 (a), (b) |
    | the theater-dependent sprites of terrain objects and smudges loaded when a game starts (S-0007) | read afresh | high (c), medium (d) | M11 (c) and (d), rows 9, 10 |
    | a building's deploying and roof deploying sprites | read afresh | high (c), medium (d) | M11 (c) and (d), rows 11, 20 |
    | a building's bib, under-door and under-roof-door sprites under their first-choice names | read afresh | high (c), medium (d) | M11 (c) and (d), rows 12, 21, 23 |
    | a building's rubble sprite under its first-choice name | read afresh | medium | unmeasured: as for the door sprite, below |
    | a building's door sprite (under either name); its rubble sprite under the generic fallback name | deferred sprite | medium | unmeasured: stock data gives no building a door or rubble sprite, and no spec says when either is drawn, so a test would have no picture to look at (the rule is that of the special overlay and bib sprites, so the confidence is medium) |

    Four interface palettes, `sidebar.pal`, `uibkgd.pal`, `observer.pal` and `yrii.pal`, are
    discarded whenever the side group is mounted and are resolved afresh after it, so each game's
    sidebar uses the palette of its own side group. With stock data alone and no file changed during
    a session, the sorts are not known to make a visible difference: stock data gives
    theater-dependent art distinct names, and no stock object is known to use a deferred sprite that
    only the loading archives hold. The sorts become visible when a file changes during a session (Scenarios M11 (c) and (e)), when a name is
    shared by two theater groups (Scenario M11 (a)), when two names share a member id (Scenario
    M21 (b)) or when a deferred sprite is held only by the loading archives (claim 13, Scenario
    M11 (d)). The discarding of the four palettes is visible with stock data (Scenario M11 (b)).
    (`quirks.art_cache_never_invalidated`: YR keeps a kept file's first copy, and a deferred
    sprite's first name, for the whole session. The fixed behaviour discards both at the start of
    every game, so each game resolves all its art afresh.)
    - Backing: `measurable` (per kind as in the table; Scenario M11 (c) row 14 for names that were
      not found, Scenario M21 (b) for two names with the same member id getting the same art,
      Scenario M11 (f) for a deferred sprite whose name does not resolve when it is to be drawn,
      and for the next drawing trying again), `unmeasured` where the table says so
    - Confidence: per kind as in the table, medium for the within-game part (Scenario M11 (e)),
      and high for a deferred sprite whose name does not resolve being left undrawn and tried again at
      the next drawing.
      Whether interface art not named here is stored is an open question.

## Formulas

**Member id.** Let N be the requested name as bytes, and let U be N with bytes a–z replaced by A–Z.
Let n = |U| and r = n mod 4. The hashed byte string is

> H = U, if r = 0
> H = U ‖ (r) ‖ (U[4·⌊n/4⌋] repeated 3 − r times), if r > 0

where (r) is the single byte with value r and U[k] is the byte at zero-based index k (the first byte
of the incomplete last group of four). Then

> id = CRC-32(H)

with the standard reflected CRC-32 used by zip archives: generator polynomial of degree 32, all-ones
start value, final inversion. The result is an unsigned 32-bit integer. For the index search it is
reinterpreted as a two's-complement signed 32-bit integer s(id) = id − 2³² if id ≥ 2³¹, else id. An
index is fully searchable when its entries are in ascending order of s(id). Example: a 5-byte name
"ABCDE" hashes the 8 bytes "ABCDE", (1), "E", "E". All arithmetic here is exact integer arithmetic
on bytes and 32-bit values; there is no floating point.

**Index search.** Let the archive's index entries, in stored order, have signed ids e[0], e[1], …,
e[m − 1], and let t = s(id) of the requested name. The search state is a pair (a, c) of integers:
the first position and the length of a run of consecutive entries still in play. It starts at
(0, m). For a state with c > 0, let h = ⌊c / 2⌋ (integer division, rounding down). The entry e[a + h]
decides the result:

| State | Entry e[a + h] compared with t | Result |
|---|---|---|
| (a, c), c > 0 | e[a + h] = t | the member at position a + h is found |
| (a, c), c > 0 | e[a + h] > t | next state (a, h) |
| (a, c), c > 0 | e[a + h] < t | next state (a + h + 1, c − h − 1) |
| (a, 0) | — | not in this archive |

Comparisons are between signed 32-bit integers. An
implementation may obtain the same found / not-found result for every name in any way, for example
with a hash map when the index is sorted, or by working out at mount time which entries this
search can reach.

**Expansion archive order.** For two expansion archives j and k (0 ≤ j, k ≤ 99) that are both
present, k is searched before j exactly when k > j. A number with no loose file has no place in the
order. This is an exact comparison of integers; there is no arithmetic and no rounding.

**Sorted name order** (claims 11, 14a and 20). Our engine orders loose files that match a
pattern by their names in the game directory, as byte strings, after bytes a–z are replaced by A–Z:
name x comes before name y when, at the first position where they differ, x has the smaller byte
value, or when x is a proper prefix of y. Two names that differ only in letter case cannot both be
present (Edge cases). This is an exact byte comparison; there is no arithmetic and no rounding. The
original has no such order: it takes the host's listing order.

**Side archive number** (claim 13a). For the side index s of the local player's country (an integer,
s ≥ 0):

> nn(s) = 2, if s = 2
> nn(s) = s + 1, otherwise

written in decimal with at least two digits (a leading zero below 10). A country with no side has
no nn, and no side group is mounted for it. Exact integer arithmetic; no rounding.

**Integrity digest** (claim 21b). The integrity digest section here is the one that a lookup of
its name finds (S-0002, claim 2). Let the document left after that section is taken out have the
non-empty sections S₁, …, Sₘ in document order, every occurrence of a repeated section name
included (another section with the integrity digest section's name among them). Let section Sᵢ have the name nᵢ
and the entries (kᵢ₁, vᵢ₁), …, (kᵢq, vᵢq) in entry order, every occurrence included. Names, keys
and values are the byte strings of S-0001 after its trimming. Let L be the two bytes 13, 10 (CR, LF).
Then

> Hᵢ = "[" ‖ nᵢ ‖ "]" ‖ L ‖ (kᵢ₁ ‖ "=" ‖ vᵢ₁ ‖ L) ‖ … ‖ (kᵢq ‖ "=" ‖ vᵢq ‖ L)
> T = H₁ ‖ (L ‖ H₂) ‖ … ‖ (L ‖ Hₘ)
> checksum = SHA-1(T)

with no other bytes added: no spaces around "=", nothing before the first section and nothing
after the L that ends the last entry. For m = 0, T is empty. SHA-1 is the standard secure hash of FIPS 180-4, and the
checksum is its 20-byte result in the standard byte order. The stored text is D = d₁ ‖ d₂ ‖ … ‖ dₚ,
where p is the number of entries of the integrity digest section, every occurrence included. For
the j-th entry in entry order, let wⱼ be the value that a lookup of that entry's key in the section
finds (S-0002; the entry's own value unless its key occurs twice). Then dⱼ is the first
min(|wⱼ|, 127) bytes of wⱼ with every trailing whitespace byte (S-0001) removed. A stored value
never starts with whitespace (S-0001), so S-0001's trimming defect cannot apply to this removal: it
always removes all trailing whitespace. D is decoded as standard base64 (alphabet A–Z, a–z, 0–9, "+",
"/", with "=" filling the last group of four). The section is valid exactly when the first 20
decoded bytes equal the checksum. A section whose text decodes to no bytes counts as missing. All of this is byte-string
work; there is no arithmetic and no floating point.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| expansion archive numbers | 0 to 99 | archive number (integer) | two-digit decimal in the file name, highest number searched first |
| maps and movie archive number | 3 | archive number (integer) | claim 12; written as the two decimal digits 03 in the names of the maps and movie archives |
| longest name | 259 | bytes | longer requested names: unspecified in the original, "not found" in our engine (claim 6) |
| member id width | 32 | bits | compared as signed for the index search |
| required startup archives | 7 | count | claim 16; the movie archive counts once (either name) |
| side archive number | nn = s + 1 for side index s, except nn = 2 for s = 2 | archive number (integer) | claim 13a; at least two decimal digits; the stock values follow from the stock side order |
| palettes resolved afresh per side group | 4 | count | claim 22 |
| integrity digest checksum length | 20 | bytes | claims 21, 21a and 21b; SHA-1, stored as base64 text |
| integrity digest value length used | 127 | bytes | claim 21b; per entry of the integrity digest section; bytes after the 127th are ignored, and trailing whitespace is removed after the cut |

**Table 2 — theater archive names** (engine constants; the same table drives S-0007). Every value
is a name, a text, so no unit applies.

| Theater | Control name C | Extension E | Palette name P | Tile-art name A | Extra archive |
|---|---|---|---|---|---|
| temperate | TEMPERAT | TEM | ISOTEM | ISOTEMP | — |
| snow | SNOW | SNO | ISOSNO | ISOSNOW | snowmd.mix |
| urban | URBAN | URB | ISOURB | ISOURB | — |
| desert | DESERT | DES | ISODES | ISODES | — |
| new urban | URBANN | UBN | ISOUBN | ISOUBN | — |
| lunar | LUNAR | LUN | ISOLUN | ISOLUN | — |

## State and ordering

**Table 1 — search order** (rows are searched from row 1 down, and the archives within a row in the
order listed; row 0 is the loose-file test of claim 1, which comes before every archive; "or" means
only one of the two is mounted).

| Row | Archive(s) | When mounted | Only as loose file? |
|---|---|---|---|
| 0 | any loose file of the requested name | — | yes |
| 1 | `langmd.mix` | startup, first | yes (no archive is mounted yet, claim 5) |
| 2 | `language.mix` | startup | no (a loose file, or else a member of `langmd.mix`, claim 5) |
| 3 | `expandmd99.mix` … `expandmd00.mix` (those present) | startup | yes |
| 4 | `ra2md.mix` | startup | no |
| 5 | `ra2.mix` | startup | no |
| 6 | `cachemd.mix`, `cache.mix` | startup | no |
| 7 | `localmd.mix`, `local.mix` | startup | no |
| 8 | `ntrlmd.mix`, `neutral.mix` | startup (may move to the end later) | no |
| 9 | `audiomd.mix` or `audio.mix` | startup, if sound starts | no |
| 10 | `ecache*.mix`, then `elocal*.mix` (each pattern in host listing order; in our engine in sorted name order, claim 11) | startup | yes |
| 11 | `conqmd.mix`, `genermd.mix`, `generic.mix`, `isogenmd.mix`, `isogen.mix`, `conquer.mix`, `cameomd.mix`, `cameo.mix` | startup | no |
| 12 | `mapsmd03.mix`, `maps03.mix` | startup | no |
| 13 | `multimd.mix` | startup | no |
| 14 | `thememd.mix` or `theme.mix` | startup | no |
| 15 | `movmd03.mix` or `movies03.mix` | startup | no |
| 15a | map packages `*.yro` (claim 14a; host listing order, in our engine sorted name order) | startup, once, when the map list is built | yes |
| 16 | `loadmd.mix`, `load.mix` | while a game loads only; mounted at the start of every game's loading and released when loading ends, before play (claim 13) | no |
| 17 | theater group (claim 14) | game start, on theater change (a group left mounted from the previous game ranks above the loading archives instead) | no |
| 18 | side group (claim 13) | every game start | no |

| State | Event | Result |
|---|---|---|
| theater group of theater T mounted | game starts on theater T | nothing changes |
| theater group of theater T mounted | game starts on theater T′ ≠ T | T's group is released; T′'s group is appended at the end |
| side group mounted | game starts | the side group is released and the local player's side group is appended at the end, after any theater group mounted for this game; the four palettes of claim 22 are then resolved afresh |
| no loading archives mounted | a game's loading starts | `loadmd.mix` and `load.mix` are appended at the end |
| loading archives mounted | the game has finished loading | both are released; they are absent during play |
| deferred sprites hold pictures from the previous game | a skirmish or multiplayer game starts | every deferred sprite's picture is discarded; each is read again at its first drawing in the new game (claim 22) |
| name requested | loose file exists | loose file used |
| name requested | no loose file | first archive in Table 1 order whose index search finds the member id is used; none means "not found" |

Observable consequence: since the side group is always mounted last, a side-archive member never
overrides a same-named member of any other archive, except of a neutral pair mounted again after it
(claim 13).

## Edge cases and quirks

- **Signed index order** (`quirks.mix_signed_binary_search`, claim 8). Default: members are looked
  up by the signed bisection of Formulas, so an archive whose index is not sorted by signed id can
  hide some of its members, exactly those the bisection does not reach. Fixed: find any member
  regardless of index order (for example by a hash map).
- **Host listing order replaced by sorted order** (claims 11, 14a and 20). The original takes
  several extra-cache archives, several extra-local archives, several map packages and several
  alternative rules candidates in the order the host file system lists them, which it does not
  define. Our engine always uses the sorted name order of Formulas. This is a deliberate departure,
  so that the result never depends on the host. No switch is proposed.
- **Rules picker** (`quirks.rules_picker`, claim 20). Default: prompt. Fixed: ignore.
- **Session-long kept art** (`quirks.art_cache_never_invalidated`, claim 22). Default: a kept
  file keeps its first copy, and a deferred sprite the name of its first request, for the whole
  session. Fixed: both are discarded at the start of every game, so each game resolves all its art
  afresh. The visible difference: a kept file that changed on disk, or a pair of names with the same
  member id, is seen as changed from the next game on.
- **Sprites held only by the loading archives** (`quirks.loading_archive_sprites_vanish`, claim 13).
  Default: a deferred sprite's picture is read when it is first drawn, after the loading archives
  are released, so a sprite that only they hold is not drawn in play, and a same-named copy in
  another mounted archive is drawn instead. Fixed: a deferred sprite's picture is read from the copy
  found at its request, so the loading archives' copy is drawn.
- **Id collisions.** Two different names that hash to the same id shadow each other across archives
  (claim 7). No switch is proposed: an archive stores only member ids, not names (claim 6), so no
  alternative lookup can tell two colliding archive members apart; stock data has no colliding
  names that matter, and an importer should report collisions.
- **Base-name files are dead data.** The base game's rules, art, AI, sound, speech, music and
  game-mode files sit inside mounted archives but are never requested (claim 15). An importer must
  not fall back to them.
- **No extra search directories** (claim 4). The original also looks for loose files on disc drives
  or in a directory given on the command line; our engine looks in the game directory only. This is
  a deliberate departure for disc installs, which are out of scope. No switch is proposed.
- **Over-long names** (claim 6). The original's result is unspecified; our engine treats a requested
  name longer than 259 bytes as not found. No switch is proposed, because the original's result
  cannot be reproduced.
- **Loose names that differ only in letter case** (claim 2). On a case-sensitive host the game
  directory can hold two loose files whose names differ only in letter case, such as `rulesmd.ini`
  and `RULESMD.INI`. The original's host file system cannot hold both, so the original has no
  behaviour here. Our engine reports such a pair as an error at import and does not start until one
  of them is removed, so that the result never depends on the host. No switch is proposed.

## Import mapping

File names the importer resolves, with the precedence of this spec (names are matched without
regard to letter case):

| File | Role | Stock source (claim 18) |
|---|---|---|
| `rulesmd.ini` | base rules | `expandmd01.mix` |
| `artmd.ini` | art database | `localmd.mix` |
| `aimd.ini` | AI database (claim 21a) | `localmd.mix` |
| `soundmd.ini` | sound database | `expandmd01.mix` |
| `evamd.ini` | speech database | `localmd.mix` |
| `thememd.ini` | music database | `localmd.mix` |
| `mpmodesmd.ini` | mode list: one entry per mode giving its name, tooltip, game-mode file, map filter and whether AI players are allowed | `localmd.mix` |
| game-mode files named in the mode list | rules layer of the selected mode (S-0006) | `localmd.mix` |
| `uimd.ini` | interface layout file (claim 16a) | `localmd.mix` |
| `keyboardmd.ini` | keyboard file | `langmd.mix` |
| `langrule.ini` | language rules file (claim 21) | absent |
| `rulemd*.ini` (loose only) | alternative rules candidates (claim 20) | absent |
| `ra2md.ini` (loose only) | settings file | loose |

Archive file names are those of Table 1 and Table 2. The four palette names of claim 22 are fixed
names, not imported keys.

**Integrity digest section** (claims 21, 21a and 21b). Its section name is `Digest`. The values of
its entries, joined in entry order, hold the 20-byte checksum as base64 text (Formulas, Integrity
digest). The key names play no part beyond their order, as long as no key occurs twice (claim 21b).

**Player entry** (claim 13a). The key `Player` in the map's `[Basic]` section, a house name (text).
Absent: the engine default `Americans`. In a skirmish or multiplayer game it plays no part in the
side archive number.

## Measurement scenarios

Every scenario runs under the glossary's *measurement setup*, with its three allowances: a
presentation-only display compatibility layer, named generically in the measurement record; the
named test files a scenario lists, which are the only loose override files allowed; and
statistical runs for sub-tests whose result depends on the random seed. Loose files left over from
earlier work would win every test below (claim 1), so check that the game directory holds only the
installed files. Scenarios that add test data say so; remove it afterwards and restore any
installed file it replaced. Where a scenario
relies on what claim 22 stores for the session, each variant runs in a fresh session. Game speed is free.

**Runs.** Resolving and reading data files involves no random draw, and every observable below is a
presence, absence or identity check (which copy shows, whether the game starts). A repeat therefore
gives the same result, and one run per variant or step is enough. Each scenario's *Runs* line says
what one run is. The exceptions are the sub-tests whose observable is the computer opponent's play
(Scenario M2 (d), Scenario M19 and Scenario M20 (b)): that play involves random draws, so they are
measured statistically, over the number of runs and with the statistic their *Runs* lines state.

**Scenario M1: expansion copy beats the local copy (claims 7, 10, 18).**
- *Setup:* measurement setup. Pick one rule that differs between the base rules copy in
  `expandmd01.mix` and the copy in `localmd.mix` and that a skirmish shows, for example a unit that
  gains an immunity or a new sound in the expansion-archive copy. Likewise pick one sound entry that
  only the sound database copy in `expandmd01.mix` defines and that a skirmish triggers.
- *Procedure:* start a skirmish where both are visible or audible.
- *Observable:* whether the behaviour and the sound of the expansion-archive copies are seen and
  heard.
- *Runs:* one skirmish.
- *Expected:* both are.
- *Falsified if:* the local copy's behaviour is seen, or the sound is missing.

**Scenario M2: a loose file beats every archive; files are read once (claims 1, 17).**
- (a) The base rules.
  - *Setup:* measurement setup plus a loose copy of the expansion rules file with one visible
    change, for example one unit's cost raised by 1 credit.
  - *Procedure:* start the game, open a skirmish and read the cost on the sidebar tooltip. Then edit
    the loose file again while the game runs (for example the cost raised by 2 credits instead) and
    start a second skirmish without restarting. Then restart and start a third skirmish.
  - *Observable:* the unit's cost on the sidebar tooltip in each of the three skirmishes.
  - *Runs:* one session of two skirmishes, then one skirmish after the restart.
  - *Expected:* +1 credit in the first and second skirmish, +2 credits in the third.
  - *Falsified if:* the first skirmish shows the stock cost, the second shows the edit made while
    the game was running, or the third does not show it.
- (b) A game-mode file.
  - *Setup:* measurement setup plus a loose copy of one stock game-mode file that is not the default
    mode's, with one added override section that raises one unit's cost by 1 credit. Play on a map
    without override sections.
  - *Procedure:* start the game and play a skirmish in that mode; read the cost on the sidebar
    tooltip. While the game runs, edit the loose file so that the cost is raised by 2 credits
    instead, and play a second skirmish in the same mode without restarting. Then restart and play a
    third skirmish in that mode.
  - *Observable:* the unit's cost on the sidebar tooltip in each of the three skirmishes.
  - *Runs:* one session of two skirmishes, then one skirmish after the restart.
  - *Expected:* +1 credit in the first and second skirmish, +2 credits in the third.
  - *Falsified if:* the first skirmish shows the stock cost, the second shows +2 credits, or the
    third does not show +2 credits.
  - Variant (ii), read at startup rather than at the first game: *Setup:* measurement setup; the
    same loose game-mode file (+1 credit), stored outside the game directory at first.
    *Procedure:* start the game and wait for the main menu; then, before any game, put the file
    loose in the game directory and play a skirmish in that mode. *Observable:* the unit's cost on
    the sidebar tooltip. *Runs:* one session of one skirmish. *Expected:* the stock cost (the file
    was read before it was added). *Falsified if:* +1 credit shows.

- (c) The art, sound, speech and music databases and the mode list.
  - *Setup:* measurement setup. Separately, stored outside the game directory at first, loose
    copies of five stock files, each with one visible or audible edit chosen by role: in the art
    database, one buildable unit's sidebar icon entry names the stock icon of a different unit; in
    the sound database, the entry a unit plays when it is selected is given the sounds of a clearly
    different entry; in the speech database, the announcement made when a structure finishes
    building is given the sound of a different announcement; in the music database, one track's
    entry is deleted; in the mode list, one mode's entry is deleted.
  - *Procedure:* start the game, open the skirmish lobby and play a skirmish in which all five edits
    could show (build the structure and the unit, select the unit, look at the in-game music track
    list). Without restarting, put the five copies loose in the game directory, open the lobby again
    and play a second such skirmish. Then restart and do the same a third time.
  - *Observable:* in each skirmish, the unit's sidebar icon, the sound heard when the unit is
    selected, the announcement heard when the structure is finished, whether the deleted track is
    listed, and whether the deleted mode is offered in the lobby.
  - *Runs:* one session of two skirmishes, then one skirmish after the restart.
  - *Expected:* the first and second skirmish show none of the five edits; the third shows all five.
  - *Falsified if:* any edit shows in the first or second skirmish, or any edit is missing from the
    third.
- (d) The AI database. Low priority.
  - *Setup:* measurement setup. Separately, stored outside the game directory at first, the edited
    copy of the stock AI database of Scenario M19 (b).
  - *Procedure:* play a skirmish as in Scenario M19. Without restarting, put the copy loose in the
    game directory and play a second such skirmish. Then restart and play a third.
  - *Observable:* whether the computer opponent sends any units against the local player's base in
    each skirmish.
  - *Runs:* 10 repeats of the whole sequence (a session of two skirmishes, then one skirmish after
    the restart), each on the same map and settings. The opponent's play involves random draws, so
    the statistic is the number of repeats in which the opponent attacks, counted separately for the
    first, second and third skirmish.
  - *Expected:* the opponent attacks in at least 8 of the 10 first skirmishes and at least 8 of the
    10 second skirmishes, and in none of the 10 third skirmishes.
  - *Falsified if:* given at least 8 attacks in the 10 first skirmishes, the opponent attacks in any
    third skirmish, or attacks in fewer than 8 of the 10 second skirmishes. If the first skirmishes show an attack in fewer than 8 repeats, the run says nothing: repeat it
    with longer skirmishes.

**Scenario M3: language archive outranks the expansion archives; a nested archive is replaced whole
(claims 3, 9).**
- *Setup:* build a test copy of the expansion's icon archive (`cameomd.mix`, normally inside
  `langmd.mix`) in which one unit's sidebar icon is replaced by an obviously different picture, and
  the icon of a unit that exists only in the expansion (so the base icon archive has no icon of that
  name) is deleted. Variant (a): put it inside a test archive `expandmd98.mix`. Variant (b):
  instead, put it loose in the game directory.
- *Procedure:* in a fresh session per variant, start a skirmish in which both units can be built.
- *Observable:* the sidebar icons of the two units.
- *Runs:* one fresh session per variant.
- *Expected:* (a) both icons are stock. (b) The replaced icon shows and the deleted icon falls back
  to the placeholder icon (S-0007), even though the stock archive still holds it.
- *Falsified if:* variant (a) shows the replaced icon or the placeholder icon, or variant (b) shows
  the stock picture for the replaced icon or still shows the deleted icon.

**Scenario M4: expansion archive numbering and rank (claims 3, 7, 9).**
- (a) Numbering.
  - *Setup:* two test archives, `expandmd98.mix` and `expandmd02.mix`, each containing a copy of the
    expansion rules file with a different cost for the same unit (for example +1 and +2 credits).
  - *Procedure:* start a skirmish and read the unit's cost on the sidebar tooltip.
  - *Observable:* the sidebar cost.
  - *Runs:* one skirmish.
  - *Expected:* the cost from number 98.
  - *Falsified if:* the cost from number 02 or the stock cost shows.
- (b) The expansion archives rank above the main expansion archive.
  - *Setup:* a test `expandmd98.mix` holding one member: a copy of the stock `conqmd.mix` (which
    stock data holds inside `ra2md.mix`) in which the infantry sprite of Scenario M5 (a) is
    visibly altered.
  - *Procedure:* in a fresh session, start a skirmish with that infantry type available.
  - *Observable:* the infantry's look.
  - *Runs:* one fresh session.
  - *Expected:* the altered sprite: `conqmd.mix` is resolved through the archives already mounted,
    and the copy in `expandmd98.mix` outranks the one in `ra2md.mix` (claim 3).
  - *Falsified if:* the stock sprite shows.

**Scenario M5: extra-cache archives rank between the local archives and `conqmd.mix`; some archives
are taken only loose; hidden and system entries are skipped (claims 5, 7, 11, 12).**
- *Setup:* (a) a loose test archive `ecache00.mix` holding (i) a rules file with a visible cost
  change, and (ii) a visibly altered copy of an infantry sprite that stock data holds in
  `conqmd.mix` and in no higher-ranked archive. (b) Instead of the loose `ecache00.mix`, a test
  `expandmd98.mix` that holds three nested archives and nothing else: the `ecache00.mix` of (a), an
  `elocal00.mix` with the same two changes, and an `expandmd97.mix` holding a rules file with a
  different visible cost change for the same unit. (c) The loose settings file moved out of the game
  directory, and a test `expandmd98.mix` holding a settings file with one visible non-default
  setting, for example the player name shown in the skirmish lobby.
- *Procedure:* in a fresh session per variant, start a skirmish with that infantry type available
  ((a), (b)), or open the skirmish lobby ((c)).
- *Observable:* the unit's cost and the infantry's look ((a), (b)); the lobby's player name ((c)).
- *Runs:* one fresh session per variant.
- *Expected:* (a) stock cost (the expansion and local copies rank higher) and the altered sprite.
  (b) Neither change shows: stock cost and stock sprite. (c) The default player name; the archived
  settings file is not used.
- *Falsified if:* in (a) the cost changes or the sprite stays stock; in (b) any change shows; in (c)
  the archived copy's player name shows.
- Afterwards restore the original loose settings file and remove any settings file the game wrote.
- (d) Hidden, system and temporary entries are skipped (claim 11).
  - *Setup:* the loose `ecache00.mix` of (a). Variant (i): the file marked hidden on the host.
    Variant (ii): instead, the file marked as a system file. Variant (iii): instead, the file's
    temporary attribute set, with any tool that can set it. Variant (iv), the control: the file with
    none of these marks, which is (a) itself.
  - *Procedure:* in a fresh session per variant, start a skirmish with that infantry type available.
  - *Observable:* the infantry's look.
  - *Runs:* one fresh session per variant, four in all.
  - *Expected:* (i), (ii) and (iii) stock sprite; (iv) the altered sprite.
  - *Falsified if:* (i), (ii) or (iii) shows the altered sprite, or (iv) shows the stock sprite.
- (e) The original's order among several extra-cache archives (claim 11). This sub-test records
  the original's behaviour, from which our engine departs on purpose (it uses the sorted order).
  - *Setup:* two loose test archives `ecache00.mix` and `ecache01.mix`, each holding a copy of the
    infantry sprite of (a) with a clearly different picture. Before the run, list the game directory
    in the host's own, unsorted directory order (the order the host returns entries in, which is the
    order the original sees) and note which of the two names comes first.
  - *Procedure:* in a fresh session, start a skirmish with that infantry type available.
  - *Observable:* which picture the infantry shows.
  - *Runs:* one fresh session.
  - *Expected:* the picture of the archive listed first.
  - *Falsified if:* the picture of the archive listed second shows.
- (f) Rank of the extra-local archives (claim 11).
  - *Setup:* variant (i): instead of the loose `ecache00.mix` of (a), a loose `elocal00.mix` with
    the same contents (the rules file with the cost change and the altered infantry sprite).
    Variant (ii): a loose `ecache00.mix` and a loose `elocal00.mix`, each holding a copy of the
    infantry sprite of (a) with a clearly different picture, and nothing else.
  - *Procedure:* in a fresh session per variant, start a skirmish with that infantry type available.
  - *Observable:* the unit's cost ((i)) and the infantry's look ((i), (ii)).
  - *Runs:* one fresh session per variant.
  - *Expected:* (i) stock cost (the archives of claim 10 rank higher) and the altered sprite (the
    extra-local archive outranks `conqmd.mix`). (ii) The picture of `ecache00.mix`, whichever of the
    two names the host lists first.
  - *Falsified if:* (i) the cost changes or the sprite stays stock; (ii) the picture of
    `elocal00.mix` shows.

**Scenario M6: order of the per-game groups (claims 13, 14).**
- (a) Expansion archive above the theater group.
  - *Setup:* a test `expandmd98.mix` holding a visibly altered copy of a temperate-theater-only
    terrain object sprite.
  - *Procedure:* play a temperate skirmish, then a snow skirmish, then a temperate one in the same
    session.
  - *Observable:* the terrain object's look in the two temperate games.
  - *Runs:* one session of three skirmishes.
  - *Expected:* altered in both temperate games (the expansion archive ranks above the theater
    group).
  - *Falsified if:* the terrain object shows its stock look in either temperate game.
- (b) Order inside the theater group.
  - *Setup:* loose copies of the five stock snow-theater archives (`snowmd.mix`, `snow.mix`,
    `sno.mix`, `isosnomd.mix`, `isosnow.mix`), each given an extra sprite under one shared new name,
    with a clearly different picture in each, and a tiny snow map whose override section points one
    infantry type's image at that name.
  - *Procedure:* play the map in a fresh session. Then delete the extra sprite from the copy whose
    picture showed and play again in a fresh session, until one copy is left.
  - *Observable:* which picture the infantry shows each time.
  - *Runs:* one fresh session per remaining copy, five in all.
  - *Expected:* the pictures appear in the order `snowmd.mix`, `snow.mix`, `sno.mix`,
    `isosnomd.mix`, `isosnow.mix`.
  - *Falsified if:* any run shows a picture other than that of the first remaining copy in this
    order.
- (c) Side group.
  - *Setup:* variant (i): a test `expandmd98.mix` holding a visibly altered copy of the sidebar's
    radar frame image, which stock data holds only in the side archives; variant (ii): instead, a
    loose copy of the stock `temperat.mix` holding the same altered image.
  - *Procedure:* a temperate skirmish with an Allied local player, once with each variant.
  - *Observable:* the radar frame in the sidebar.
  - *Runs:* one skirmish per variant.
  - *Expected:* altered in both variants (the expansion archive and the theater group both rank
    above the side group).
  - *Falsified if:* the stock radar frame shows in either variant.

**Scenario M7: missing archives and files (claims 13, 15, 16, 16a, 19).** Each step is separate; restore
the install after each. For every step, the *Procedure* is to start the game and, where the step
says so, open the skirmish map list or start a skirmish. *Runs:* one start per step, plus the
repeats, variants and skirmishes that steps (e), (g), (h), (i), (j), (k) and (n) name.
- (a) *Setup:* `multi.mix`, `maps01.mix` and `maps02.mix` moved out of the game directory.
  *Observable:* whether the game starts, and the skirmish map list. *Expected:* the game starts and
  the map list is unchanged. *Falsified if:* the game refuses to start or the map list changes.
- (b) *Setup:* `multimd.mix` moved out. *Observable:* whether the game starts. *Expected:* it
  refuses to start. *Falsified if:* it starts.
- (c) *Setup:* `movmd03.mix` moved out (no `movies03.mix` exists in stock). *Observable:* whether
  the game starts. *Expected:* it refuses to start. *Falsified if:* it starts.
- (d) *Setup:* `movies01.mix` and `movies02.mix` moved out. *Observable:* whether the game starts,
  and the skirmish map list. *Expected:* the game starts and the map list is unchanged. *Falsified
  if:* it refuses to start or the map list changes.
- (e) *Setup:* a loose copy of the stock `localmd.mix` (extracted from the main expansion archive)
  without the speech database. (In the steps below, "the loose `localmd.mix` copy of (e) without X"
  means such a copy that lacks only X, every other member unchanged.) Repeated with the art database removed
  instead, and with the music database removed instead. *Observable:* whether the game starts. *Expected:* it refuses to start
  each time. *Falsified if:* it starts in any of these runs.
- (f) *Setup:* the loose `localmd.mix` copy of (e) without the sound database, and in place of the
  stock `expandmd01.mix` a copy of it without the sound database. *Observable:* whether the game
  starts. *Expected:* it refuses to start. *Falsified if:* it starts.
- (g) *Setup:* variant (i): the loose `localmd.mix` copy of (e) without the interface layout file.
  Variant (ii): instead, measurement setup plus a loose interface layout file that is empty (no
  section at all). *Observable:* whether the game starts, and whether a skirmish reaches play.
  *Expected:* in both variants the game starts, but the skirmish never reaches play. *Falsified
  if:* the game refuses to start, or the skirmish plays, in either variant.
- (h) *Setup:* a loose copy of the stock mode list in which one mode's game-mode file name is
  changed to a name that no archive and no loose file holds. Pick a mode whose stock game-mode file
  makes a change that a skirmish shows (for example a different starting condition or an object that
  is only present in that mode). Play it on a map without override sections, so that only the base
  rules and the game-mode layer can apply. *Observable:* whether that mode is offered, and whether
  its change shows in a skirmish played in it. *Expected:* the mode is offered, the skirmish starts,
  and the mode's change is absent: the skirmish plays as with the base rules alone. *Falsified if:*
  the mode is missing from the list, the game refuses to start, the skirmish never reaches play, or
  the mode's change still shows.
- (i) *Setup:* variant (i): in place of the installed `ra2.mix`, a copy of it without the Allied
  side archive `sidec01.mix`. Variant (ii): instead, a copy of the installed `ra2.mix` without the
  Soviet side archive `sidec02.mix`. In stock data each of these two archives is a top-level member
  of `ra2.mix` only, so no other archive and no loose file can supply it. *Observable:* whether the
  game starts; whether a skirmish reaches play with an Allied, with a Soviet and with a Yuri local
  player. *Runs:* per variant, one start, then one Allied, one Soviet and one Yuri skirmish, each in
  a fresh session. *Expected:* the game starts in both variants. Variant (i): the Allied skirmish
  never reaches play, the Soviet and the Yuri skirmish play. Variant (ii): the Allied skirmish
  plays, the Soviet and the Yuri skirmish never reach play (the Yuri side uses the Soviet side's
  archives). *Falsified if:* the game refuses to start in either variant, or any of the six
  skirmishes reaches play or fails to reach play otherwise than stated.
- (j) *Setup:* each of the five remaining required archives made unresolvable in turn:
  `mapsmd03.mix` moved out of the game directory; `conqmd.mix` removed from a copy of `ra2md.mix`,
  `conquer.mix` removed from a copy of `ra2.mix`, `cameomd.mix` removed from a copy of `langmd.mix`
  and `cameo.mix` removed from a copy of `language.mix`, each copy put in place of the installed
  file. In stock data each of these archives is held only where it is removed here. *Observable:*
  whether the game starts. *Runs:* one start per archive, five in all. *Expected:* it refuses to
  start each time. *Falsified if:* it starts in any of these runs.
- (k) *Setup:* variant (i): the loose `localmd.mix` copy of (e) without the base rules file, and in
  place of the stock `expandmd01.mix` a copy of it without the base rules file, so that no copy of
  the base rules can be resolved. Variant (ii): instead, measurement setup plus a loose base rules
  file of zero bytes. *Observable:* everything the game shows from its start until the main menu
  appears or the game stops (record the screen): every message, and whether the main menu appears.
  *Runs:* one start per variant. *Expected:* both variants behave the same: the same messages (or
  none) and the same outcome. *Falsified if:* the two variants differ in any recorded message or
  in whether the main menu appears.
- (l) *Setup:* the loose `localmd.mix` copy of (e) without the AI database. In stock data no other
  archive holds it (claim 18). *Observable:* every message shown before the main menu (record the
  screen), and whether the main menu appears. *Expected:* no message about the missing file, and
  the main menu appears. *Falsified if:* a message about the missing file appears, or the game
  refuses to start.
- (m) *Setup:* the loose `localmd.mix` copy of (e) without the mode list. In stock data no other
  archive holds it (claim 18). *Observable:* every message shown before the main menu (record the
  screen), and whether the main menu appears. *Expected:* no message about the missing file, and
  the main menu appears. *Falsified if:* a message about the missing file appears, or the game
  refuses to start.
- (n) *Setup:* measurement setup plus one loose file of zero bytes named as the art database.
  Repeated with the sound database instead, then with the speech database, then with the music
  database (one zero-byte file per run; the loose file wins over every archived copy, claim 1).
  *Observable:* whether the game starts. *Runs:* one start per database, four in all. *Expected:*
  it refuses to start each time. (Compare step (k) variant (ii): a zero-byte base rules file is
  not, of itself, a reason to refuse, claim 16.) *Falsified if:* it starts in any of these runs.

**Scenario M8: case-insensitive names (claims 2, 6).**
- *Setup:* variant (a): the Scenario M2 loose rules file saved as `rUlEsMd.InI`. Variant (b):
  instead, a test `expandmd98.mix` holding the same changed rules file, with its member id hashed
  from the lower-case name `rulesmd.ini` (hand-built index).
- *Procedure:* start a skirmish and read the unit's cost on the sidebar tooltip.
- *Observable:* whether the changed cost shows.
- *Runs:* one skirmish per variant.
- *Expected:* (a) the change applies. (b) The member is not found (ids are computed from upper-case
  names), so the stock cost shows.
- *Falsified if:* (a) shows the stock cost, or (b) shows the changed cost.

**Scenario M9: signed index order (claim 8).** Low priority: it only matters to archive-building
tools.
- *Setup:* a test `expandmd98.mix` holding several visibly altered files whose ids include both
  values below 2³¹ and values at or above 2³¹, among them the member with the largest id below 2³¹
  and the one with the smallest id at or above 2³¹, with the index sorted by *unsigned* id. Before
  the run, work out with the Index search of Formulas which members are reached. A second variant
  re-sorts the index by signed id.
- *Procedure:* in a fresh session per variant, start a skirmish in which every altered file is used.
- *Observable:* which altered files show.
- *Runs:* one fresh session per variant.
- *Expected:* unsigned order: exactly the members the bisection reaches show altered, the rest
  stock. At least one of the two named members is stock. Signed order: all show altered.
- *Falsified if:* the set of altered files differs from the worked-out set, or any is still stock
  after re-sorting by signed id.

**Scenario M10: rules picker (claim 20).**
- *Setup:* (a) measurement setup only. (b) and (c): measurement setup plus a copy of the expansion
  rules file, loose under the name `rulemdt.ini` (a valid short name), with one visible change: one
  unit's cost raised by 1 credit.
- *Procedure:* (a) start the game. (b) Start the game and read the list. (c) Two fresh starts: in
  one choose the second entry of the list, in the other the first; each time play a skirmish and
  read the unit's cost on the sidebar tooltip.
- *Observable:* (a) whether a selection list appears before the main menu, (b) the list's entries,
  (c) the unit's cost after each choice.
- *Runs:* (a) one start, (b) one start, (c) two starts.
- *Expected:* (a) no list; the main menu appears directly. (b) A list with two entries: the
  standard rules file, then `rulemdt.ini`. (c) +1 credit after choosing the second entry, the stock
  cost after choosing the first.
- *Falsified if:* (a) a list appears; (b) no list appears, or it does not have exactly these two
  entries in this order; (c) the second entry gives the stock cost, or the first gives +1 credit.
- (d) The original's order of several matching files. As in Scenario M5 (e), this records the
  original's behaviour, from which our engine departs on purpose (claim 20).
  - *Setup:* measurement setup plus two loose copies of the expansion rules file, `rulemdt.ini` and
    `rulemdu.ini`. Before the run, list the game directory in the host's own, unsorted directory
    order (as in Scenario M5 (e)) and note which of the two names comes first.
  - *Procedure:* start the game and read the list.
  - *Observable:* the order of the list's entries.
  - *Runs:* one start.
  - *Expected:* the standard rules file, then the two files in the host's listing order.
  - *Falsified if:* the standard rules file is not first, or the two files appear in the opposite
    of the host's listing order.

**Scenario M11: kept art, deferred sprites and art read afresh (claims 13, 22).** Every altered
or test copy below keeps the number of image frames and the image size of the copy it replaces (see Open
questions).
- (a) A deferred sprite's picture is read again in every skirmish. Low priority. It needs a sprite name
  that exists only in two theater groups, with different content in each. Stock data has none, so
  test data is needed.
  - *Setup:* loose copies of the stock temperate and snow control archives (`temperat.mix`,
    `snow.mix`), each given an extra sprite under the same new name with clearly different pictures,
    and a map override that points one infantry type's image at that new name, on one temperate and
    one snow map.
  - *Procedure:* a temperate skirmish, then a snow skirmish, in one session; then restart and play
    the snow skirmish first.
  - *Observable:* which picture the infantry shows in each snow game.
  - *Runs:* two sessions: one of two skirmishes, one of a single skirmish.
  - *Expected:* both snow games show the snow picture: the name stays the same, and its picture is
    read again from the snow group in each snow game.
  - *Falsified if:* either snow game shows the temperate picture.
- (b) The four palettes.
  - *Setup:* measurement setup.
  - *Procedure:* in one session, a skirmish with an Allied local player, then one with a Soviet
    local player. For comparison, a fresh session with a Soviet local player first.
  - *Observable:* the sidebar's colours.
  - *Runs:* one session of two skirmishes, plus one comparison session.
  - *Expected:* each game shows its own side's sidebar colours; the second game of the first session
    looks the same as the comparison session.
  - *Falsified if:* the Soviet game of the first session shows the Allied sidebar colours, or looks
    different from the comparison session.

- (c) Each kind of art, by a file added between two games. This separates kept files from the
  other two sorts.
  - *Setup:* measurement setup. For each row of the table below, a subject chosen by its role, and a
    visibly altered loose copy of the subject's file, stored outside the game directory at first.
    Choose a stock map of the theater named in the row (temperate where none is named) on which the
    subject is seen; where no stock map shows it, use a tiny test map that places it. Rows 16, 17
    and 25 also need the test data their row names, present from the start in both sessions; for
    them, "stock" below means the picture of that test data.
  - *Procedure:* per row, two sessions. Main session: play skirmish 1 on that map with the subject
    seen; without restarting, put the loose copy in the game directory; play skirmish 2 on the same
    map with the subject seen again. (For the theater palette row: skirmish 1 on a temperate map,
    then add the copy, then a snow skirmish, then skirmish 2 on the temperate map, because that
    palette is requested only when the theater group is mounted.) Control session: a fresh session
    with the loose copy present from the start, one skirmish on the same map.
  - *Observable:* whether the subject shows its stock or its altered picture (or colours) in
    skirmish 2 and in the control.
  - *Runs:* per row, one main session and one control session.
  - *Expected:* the control shows the altered picture in every row (the copy itself is valid and is
    found). Skirmish 2 shows the stock picture for a "kept file" row and the altered picture for
    every other row.
  - *Falsified if:* skirmish 2 shows the altered picture for a "kept file" row, or the stock picture
    for any other row, while the control shows the altered picture. (A control that shows the stock
    picture makes that row's run void: fix the test copy and run it again.)
- (d) Each kind of art, by a file that only the loading archives hold. This separates deferred
  sprites from art read afresh and kept files.
  - *Setup:* measurement setup. Per row of the table except rows 8 and 14, the subject's file is
    made resolvable only through the loading archives: a loose copy of the stock archive that holds
    it, without that file (claim 3), and a loose copy of the stock `loadmd.mix` (extracted from the
    main expansion archive) with that file added. Choose the subject so that no other archive of
    stock data holds a file of that name; for rows 13, 22 and 24, also none of the first-choice
    name. For rows 16, 17 and 25, put the file under the new name in the `loadmd.mix` copy instead of
    the test archive the row names.
  - *Procedure:* per row, one fresh session; play one skirmish on the row's map with the subject
    seen.
  - *Observable:* whether the subject is drawn with its picture (for a sidebar icon: whether its
    sidebar slot shows a picture).
  - *Runs:* one fresh session per row.
  - *Expected:* as the last column of the table: "not drawn" for a deferred sprite (its picture is
    read only after the loading archives are released, claim 13), "drawn" for art read afresh and
    for kept files (both are read while the game loads).
  - *Falsified if:* a "not drawn" row shows the subject's picture, or a "drawn" row does not.

  | Row | Subject (chosen by role) | File | Sort (claim 22) | Expected in (c), skirmish 2 | Expected in (d) |
  |---|---|---|---|---|---|
  | 1 | an infantry type the local player can train | its main sprite | deferred sprite | altered | not drawn |
  | 2 | an overlay the map shows, such as a wall or fence | its sprite | deferred sprite | altered | not drawn |
  | 3 | an animation that plays on an object present from the start of the game | its sprite | deferred sprite | altered | not drawn |
  | 4 | a building the local player can build | its main sprite | deferred sprite | altered | not drawn |
  | 5 | the same building | its build-up sprite (seen when it is placed) | deferred sprite | altered | not drawn |
  | 6 | a unit the local player can build | its sidebar icon | deferred sprite | altered | not drawn |
  | 7 | an object type that has a secondary translucent image | that image | deferred sprite | altered | not drawn |
  | 8 | the temperate theater | its palette "unit" ‖ E ‖ ".pal" (E from Table 2), seen on the colours of objects drawn with it | kept file | stock | — |
  | 9 | a terrain object of the temperate theater whose sprite depends on the theater, whose image name equals its type name (S-0007) | that sprite | read afresh | altered | drawn |
  | 10 | a smudge whose sprite depends on the theater, whose image name equals its type name, placed by the map | that sprite | read afresh | altered | drawn |
  | 11 | a war factory that has a deploying sprite (shown while it releases a newly built vehicle, S-0108) | that sprite | read afresh | altered | drawn |
  | 12 | a building whose bib sprite stock data holds under its first-choice name for this theater | that file | read afresh | altered | drawn |
  | 13 | a building whose bib sprite stock data does not hold under its first-choice name for this theater, so that the generic fallback name is used (S-0007) | the file of the fallback name | deferred sprite | altered | not drawn |
  | 14 | an infantry type whose image name a tiny test map's override section sets to a new name that no archive holds (in skirmish 1 the infantry has no sprite) | a sprite under that new name | — | altered (a name that was not found is not stored) | — |
  | 15 | a vehicle the local player can build that stock data draws as a sprite | its main sprite | deferred sprite | altered | not drawn |
  | 16 | an aircraft type the local player can build, whose image name a tiny test map's override section sets to a new name that the art database has no section for, so that it is drawn as a sprite (S-0007); a test `expandmd98.mix` holds a sprite under that name | that sprite | deferred sprite | altered | not drawn |
  | 17 | a terrain object type whose image name a tiny test map's override section sets to a new name that the art database has no section for, so that its sprite does not depend on the theater, placed by that map; a test `expandmd98.mix` holds a sprite under that name | that sprite | deferred sprite | altered | not drawn |
  | 18 | a smudge whose sprite does not depend on the theater, placed by the map | its sprite | deferred sprite | altered | not drawn |
  | 19 | a gate building that has a special overlay sprite | the file of that sprite this theater uses | deferred sprite | altered | not drawn |
  | 20 | a war factory that has a roof deploying sprite (shown while it releases a jumpjet or balloon-hover unit, S-0108) | that sprite | read afresh | altered | drawn |
  | 21 | a war factory whose under-door sprite stock data holds under its first-choice name for this theater (pick the theater accordingly) | that file | read afresh | altered | drawn |
  | 22 | a war factory whose under-door sprite stock data does not hold under its first-choice name for this theater, so that the generic fallback name is used | the file of the fallback name | deferred sprite | altered | not drawn |
  | 23 | as row 21, for the under-roof-door sprite (shown while a jumpjet or balloon-hover unit is released) | that file | read afresh | altered | drawn |
  | 24 | as row 22, for the under-roof-door sprite | the file of the fallback name | deferred sprite | altered | not drawn |
  | 25 | an infantry type the local player can train, whose art section a loose copy of the stock art database gives the theater flag, so that its sprite is named image name ‖ "." ‖ E (S-0007); a test `expandmd98.mix` holds that file for the temperate theater | that file | kept file | stock | drawn |

- (e) Within one game, a deferred sprite's picture is read at its first drawing.
  - *Setup:* measurement setup. Separately, stored outside the game directory at first, visibly
    altered loose copies of the main sprites of two infantry types I1 and I2 that the local player
    can train. Play against an opponent of the other side, so that only the local player can have
    I1 or I2, on a map that places none of them.
  - *Procedure:* in a fresh session, start skirmish 1, train an I1 and look at it. Without
    restarting, put both copies loose in the game directory. Look at the I1 again, then train the
    first I2 and look at it. Then play skirmish 2 in the same session and train an I1.
  - *Observable:* the picture of I1 after the copies were added, of the first I2, and of I1 in
    skirmish 2.
  - *Runs:* one session of two skirmishes.
  - *Expected:* in skirmish 1, I1 keeps its stock picture and the first I2 shows the altered one.
    In skirmish 2, I1 shows the altered picture.
  - *Falsified if:* I1 turns altered during skirmish 1, the first I2 shows its stock picture, or I1
    is stock in skirmish 2.
- (f) A deferred sprite whose name does not resolve when it is to be drawn.
  - *Setup:* a new sprite name N that no archive of stock data holds, nor its generic fallback name
    (S-0007); a loose sprite file N ‖ ".shp", a visibly altered copy of the stock main sprite of
    the infantry type I below (same number of image frames and image size), present in the game directory at
    first; a tiny test map whose override section points the image of an
    infantry type I that the local player can train at N, placing none. Play against an opponent of
    the other side, so that only the local player can have I.
  - *Procedure:* in a fresh session, play skirmish 1 on the map, train an I and look at it. Back in
    the menus, move the loose file out of the game directory. Play skirmish 2 on the same map,
    train an I and look at it (an I that is not drawn can still be found by dragging a selection
    box over the spot where it appears). Without leaving
    skirmish 2, put the loose file back in the game directory and look at the same I again.
  - *Observable:* whether I is drawn, and with which picture, in skirmish 1, in skirmish 2 before
    the file is put back, and in skirmish 2 after it is put back.
  - *Runs:* one session of two skirmishes.
  - *Expected:* skirmish 1: the altered picture. Skirmish 2 before the file is put back: nothing is
    drawn for I (the name is stored from skirmish 1, but its picture was discarded at the start of
    skirmish 2 and the name does not resolve). After the file is put back: the altered picture, within
    the same game, without any new request (the loose-file test is made at every open, claim 1).
  - *Falsified if:* skirmish 1 does not show the altered picture, I is drawn with any picture in
    skirmish 2 before the file is put back, or I stays undrawn for the rest of skirmish 2 after the
    file is put back.

**Scenario M12: map packages (claim 14a).**
- (a) Mounted once at startup.
  - *Setup:* measurement setup, which includes several stock map packages loose in the game
    directory (claim 14a). Pick one of them.
  - *Procedure:* open the skirmish map list. Then quit, rename one stock package's extension,
    restart and open the list. Then, with the game running, rename it back and open the list again
    without restarting. Finally restart and open the list.
  - *Observable:* whether the maps inside that package appear in the skirmish map list.
  - *Runs:* one per step, four in all.
  - *Expected:* present; missing; still missing; present.
  - *Falsified if:* the game directory of the measurement setup holds no map package, or the
    package's maps are missing at first, still appear after the rename, reappear before a restart,
    or are missing after the final restart.
- (b) Rank of the packages.
  - *Setup:* a test copy of one stock package, in place of the installed file, keeping its
    description member and maps, with two added members: (i) a visibly altered copy of a
    temperate-theater-only terrain object sprite, which stock data holds only in the temperate
    theater group; (ii) a visibly altered copy of an infantry sprite that stock data holds in
    `conqmd.mix` and in no higher-ranked archive.
  - *Procedure:* in a fresh session, a temperate skirmish with that infantry type available.
  - *Observable:* the terrain object's look and the infantry's look.
  - *Runs:* one fresh session.
  - *Expected:* the terrain object is altered (the package ranks above the theater group); the
    infantry is stock (the package ranks below `conqmd.mix`).
  - *Falsified if:* the terrain object is stock, or the infantry is altered.

- (c) The description member's name, and hidden and system packages.
  - *Setup:* a test package `test01.yro` built from one tiny skirmish map that no stock package or
    loose file holds and that the lobby's default mode accepts, with a description member `test01.pkt` that lists that map. Variant (i): the
    package as built. Variant (ii): the same package renamed `test02.yro`, still holding
    `test01.pkt` and no `test02.pkt`, and also holding the altered terrain object sprite of (b) (i).
    Variant (iii): variant (i) marked hidden on the host.
    Variant (iv): variant (i) marked as a system file instead. Variant (v): variant (i) with its
    temporary attribute set instead, with any tool that can set it.
  - *Procedure:* in a fresh session per variant, open the skirmish map list; in variant (ii), also
    play a temperate skirmish on a stock map where that terrain object stands.
  - *Observable:* whether the test map is listed; in (ii), the terrain object's look.
  - *Runs:* one fresh session per variant, five in all.
  - *Expected:* listed in (i); not listed in (ii), (iii), (iv) and (v); in (ii) the terrain object
    is altered (the package is mounted although it lists no maps).
  - *Falsified if:* (i) does not list it, any of (ii) to (v) lists it, or the terrain object is
    stock in (ii).
- (d) The `missions.yro` exclusion.
  - *Setup:* variant (i): the package of (c) (i), but named `missions.yro` and with its description
    member named `missions.pkt`. Variant (ii), the control: the same contents named `test03.yro`
    with the description member `test03.pkt`. If the install already holds a loose `missions.yro`,
    move it out of the game directory first and restore it afterwards.
  - *Procedure:* in a fresh session per variant, open the skirmish map list.
  - *Observable:* whether the test map is listed.
  - *Runs:* one fresh session per variant.
  - *Expected:* (i) not listed; (ii) listed.
  - *Falsified if:* (i) lists it, or (ii) does not.
- (e) The original's order among several packages. As in Scenario M5 (e), this records the
  original's behaviour, from which our engine departs on purpose (claim 14a).
  - *Setup:* two test packages `test04.yro` and `test05.yro`, each with its own description member
    (`test04.pkt`, `test05.pkt`) listing no maps, and each holding a copy of the temperate-theater-only
    terrain object sprite of (b) (i) with a clearly different picture. Before the run, list the game
    directory in the host's own, unsorted directory order (as in Scenario M5 (e)) and note which of
    the two names comes first.
  - *Procedure:* in a fresh session, play a temperate skirmish on a stock map where that terrain
    object stands.
  - *Observable:* which picture the terrain object shows.
  - *Runs:* one fresh session.
  - *Expected:* the picture of the package listed first.
  - *Falsified if:* the picture of the package listed second shows.

**Scenario M13: loading archives (claim 13).** Parts (a), (b) and (d) use the interface layout file,
which is read whole while each game loads (claim 16a), so its copy is decided while the loading
archives are mounted.
- (a) They are searched while a game loads.
  - *Setup:* variant (i): the loose `localmd.mix` copy without the interface layout file of
    Scenario M7 (g) (i), plus a loose copy of the stock `loadmd.mix` (extracted from the main
    expansion archive) to which the swapped interface layout file of Scenario M17 (a) is added.
    Variant (ii): instead of the `loadmd.mix` copy, a loose copy of the stock `load.mix` (extracted
    from `ra2.mix`) with the same file added.
  - *Procedure:* in a fresh session per variant, play a skirmish.
  - *Observable:* whether the skirmish reaches play, and the order of the command-bar buttons.
  - *Runs:* one fresh session per variant. (Scenario M7 (g) (i) is the control: without the
    loading-archive copy the skirmish never reaches play.)
  - *Expected:* in both variants the skirmish plays, with the swapped order.
  - *Falsified if:* either skirmish never reaches play, or shows the stock order.
- (b) Their rank against the theater group.
  - *Setup:* the loose `localmd.mix` copy of (a), plus a loose copy of the stock `loadmd.mix` with
    interface layout file A added, and a loose copy of the stock `temperat.mix` with interface
    layout file B added. A is the swapped file of Scenario M17 (a) (first two buttons swapped in
    both lists); B is the stock file with the last two buttons of both lists swapped instead.
  - *Procedure:* in one fresh session, play a snow skirmish, then a temperate skirmish, then a
    second temperate skirmish. The snow game makes the first temperate game a theater change, so
    its temperate group is mounted for that game, after the loading archives; the second temperate
    game keeps the group left mounted from the first.
  - *Observable:* the order of the command-bar buttons in each game.
  - *Runs:* one session of three skirmishes.
  - *Expected:* A in the snow game, A in the first temperate game (the loading archives rank above
    a theater group mounted for this game), B in the second temperate game (a theater group left
    mounted from the previous game ranks above the loading archives).
  - *Falsified if:* any game shows another order.
- (c) They are released before play, seen through a deferred sprite.
  - *Setup:* variant (i): a loose copy of the stock `loadmd.mix` with one extra sprite, picture L,
    under a new name S that, like its generic fallback name (S-0007), no archive of stock data
    holds; and a tiny temperate map whose override section points one infantry type's image at S.
    Variant (ii): also, a loose copy of the stock `temperat.mix` with a sprite under S that has a
    clearly different picture T.
  - *Procedure:* in a fresh session per variant, play a skirmish on the map with that infantry
    seen. In a fresh session the temperate group is mounted for the game, so at the request the
    `loadmd.mix` copy ranks above it ((b)).
  - *Observable:* the infantry's picture.
  - *Runs:* one fresh session per variant.
  - *Expected:* (i) the infantry is not drawn: S was found when it was requested, but the loading
    archives are gone when its picture is read. (ii) The infantry shows T.
  - *Falsified if:* either variant shows L, or (ii) does not draw the infantry.
- (d) Their order: `loadmd.mix` ranks above `load.mix`.
  - *Setup:* the loose `localmd.mix` copy of (a), plus a loose copy of the stock `loadmd.mix` with
    interface layout file A of (b) added, and a loose copy of the stock `load.mix` (extracted from
    `ra2.mix`) with interface layout file B of (b) added.
  - *Procedure:* in a fresh session, play a skirmish.
  - *Observable:* whether the skirmish reaches play, and the order of the command-bar buttons.
  - *Runs:* one fresh session.
  - *Expected:* the skirmish plays, with order A.
  - *Falsified if:* the skirmish never reaches play, or shows order B or the stock order.

**Scenario M14: order of the startup archives (claims 9, 10, 12).** Low priority.
- *Setup:* for two archives X and Y that are next to each other in Table 1 (rows 1 to 15, X listed
  first), test copies of both, each given an extra sprite under one shared new name, with clearly
  different pictures; and a tiny map whose override section points one infantry type's image at that
  name. A test copy of an archive that is nested in stock is placed loose (claim 3); a test copy of
  a top-level installed archive takes the place of the installed file. For row 3, which has no
  installed archive, X or Y is a loose test expansion archive `expandmd98.mix` holding only the
  sprite.
- *Procedure:* play the map in a fresh session.
- *Observable:* the infantry's picture.
- *Runs:* one fresh session per pair, for at least the pairs `langmd.mix`/`language.mix`,
  `language.mix`/`expandmd98.mix`, `cachemd.mix`/`cache.mix`,
  `localmd.mix`/`local.mix`, `generic.mix`/`isogenmd.mix`, `cameo.mix`/`mapsmd03.mix` and
  `multimd.mix`/`thememd.mix`.
- *Expected:* X's picture.
- *Falsified if:* Y's picture shows.

**Scenario M15: the loose-file test is made at every open; voxel models and motion files are read
afresh (claims 1, 22).**
- *Setup:* variant (a): a visibly altered loose copy of one vehicle's voxel model, stored outside
  the game directory at first. Variant (b): instead, a loose copy of the same vehicle's motion file
  in which every transform is changed so that the vehicle is drawn clearly displaced or turned (the
  voxel model itself unchanged), also stored outside at first.
- *Procedure:* per variant, in one session: start the game and play a skirmish with that vehicle.
  Without restarting, add the loose file and play a second skirmish. Then, still without
  restarting, remove it and play a third.
- *Observable:* the vehicle's look in each game.
- *Runs:* one session of three skirmishes per variant.
- *Expected:* in both variants stock, altered, stock (voxel models and motion files are read afresh,
  claim 22, and each game loads its objects' art again, S-0007).
- *Falsified if:* in either variant the second game shows the stock look or the third the altered
  one.

**Scenario M16: the keyboard file and the string table come from the language archive (claims 9,
18).**
- *Setup:* a copy of the stock keyboard file in which one command is bound to a different key.
  Variant (a): the copy inside a test `expandmd98.mix`. Variant (b): the copy loose in the game
  directory. Variants (c) and (d): instead, a copy of the expansion's string table in which the
  label of one main-menu button is changed to a clearly different text; (c) the copy inside a test
  `expandmd98.mix`, (d) the copy loose in the game directory.
- *Procedure:* in a fresh session per variant: (a), (b) start a skirmish and press the new key and
  the stock key for that command; (c), (d) start the game and read the main menu.
- *Observable:* (a), (b) which key performs the command; (c), (d) the button's label.
- *Runs:* one fresh session per variant.
- *Expected:* (a) the stock key (the language archive outranks every expansion archive). (b) The new
  key (a loose file wins). (c) The stock label. (d) The changed label.
- *Falsified if:* (a) the new key performs the command, (b) the stock key still does and the new
  one does not, (c) the changed label shows, or (d) the stock label shows.
- Afterwards remove any keyboard file the game wrote.
- (e) The string table is read while only the language archives are mounted (claims 9, 16).
  - *Setup:* variant (i): in place of the installed `langmd.mix`, a copy of it without the string
    table (in stock data no other archive holds it). Variant (ii): the same, plus a test
    `expandmd98.mix` holding the changed string table of (c). Variant (iii): the same as (i), plus
    the changed string table loose in the game directory.
  - *Procedure:* start the game, one fresh start per variant.
  - *Observable:* whether the main menu appears, and the label of the changed button.
  - *Runs:* one start per variant, three in all.
  - *Expected:* (i) and (ii): the game refuses to start, so the main menu never appears. (iii) The
    main menu appears with the changed label.
  - *Falsified if:* the main menu appears in (i) or (ii), or in (iii) it does not appear or shows
    the stock label.

**Scenario M17: the interface layout file is read for every game (claims 13, 16a).**
- (a) Read per game.
  - *Setup:* measurement setup. Separately, a copy of the stock interface layout file in which the
    two command-bar button lists (the one for skirmish and solo games and the one for multiplayer
    games) both have their first two buttons swapped, stored outside the game directory at first.
  - *Procedure:* start the game and play a skirmish. Without restarting, put the copy loose in the
    game directory and play a second skirmish. Still without restarting, remove it and play a
    third.
  - *Observable:* the order of the buttons on the command bar in each game.
  - *Runs:* one session of three skirmishes.
  - *Expected:* stock order, swapped order, stock order.
  - *Falsified if:* the second game shows the stock order, or the third shows the swapped order.
- (b) A side archive can supply it.
  - *Setup:* the loose `localmd.mix` copy without the interface layout file of Scenario M7 (g), plus
    a loose copy of the stock Allied side archive `sidec01.mix` (claim 3) to which the swapped
    interface layout file of (a) is added.
  - *Procedure:* in one fresh session each, a skirmish with an Allied local player and a skirmish
    with a Soviet local player.
  - *Observable:* whether the game starts; whether each skirmish reaches play; the order of the
    command-bar buttons in the Allied game.
  - *Runs:* two fresh sessions.
  - *Expected:* the game starts both times; the Allied skirmish plays with the swapped order; the
    Soviet skirmish never reaches play, since no archive it mounts holds the file.
  - *Falsified if:* the game refuses to start, the Allied skirmish does not play or shows the stock
    order, or the Soviet skirmish plays.
- (c) Order inside the side group (claim 13).
  - *Setup:* the loose `localmd.mix` copy without the interface layout file of Scenario M7 (g) (i).
    Interface layout file A is the swapped file of (a), B the file of Scenario M13 (b) (last two
    buttons of both lists swapped), C the stock file. Variant (i): a loose `sidec01md.mix` with A
    added and a loose `sidec01.mix` with B added. Variant (ii): a loose `sidec01.mix` with B added
    and a loose `sidenc01.mix` with C added. Each of these loose archives is a copy of the stock
    archive of that name (claim 3) with the file added, or, if stock data holds no archive of that
    name, a test archive holding only that file.
  - *Procedure:* in a fresh session per variant, a skirmish with an Allied local player.
  - *Observable:* whether the skirmish reaches play, and the order of the command-bar buttons.
  - *Runs:* one fresh session per variant.
  - *Expected:* both skirmishes play; (i) shows order A, (ii) order B.
  - *Falsified if:* either skirmish never reaches play, (i) shows any order but A, or (ii) any order
    but B.

**Scenario M18: the language rules file's integrity check (claims 21, 21b).**
- (a) to (c): the refusal and the file without sections.
  - *Setup:* measurement setup plus a loose `langrule.ini`. Variant (a): one harmless section with
    one entry that changes nothing visible, and no integrity digest section. Variant (b): the same,
    plus an integrity digest section whose checksum is not this file's (for example the stock AI
    database's integrity digest section, copied unchanged). Variant (c): an empty file (no section
    at all).
  - *Procedure:* start the game, one fresh start per variant.
  - *Observable:* whether the main menu appears.
  - *Runs:* one start per variant, three in all.
  - *Expected:* (a) and (b): the game refuses to start, so the main menu never appears. (c): the
    main menu appears.
  - *Falsified if:* the main menu appears in (a) or (b), or does not appear in (c).
- (d) to (g): a valid digest, and what it covers. In these steps the *test file* is a loose
  `langrule.ini` with one section that changes something a skirmish shows (for example one
  buildable type's cost field set to a different number), and the *digest* is an integrity digest
  section computed for the file by Formulas (Integrity digest), written as one entry.
  - *Setup:* variant (d): measurement setup plus the test file and its digest. Variant (e): the
    file of (d), then, without computing the digest again, a comment line, blank lines, one space
    on both sides of each `=` (no indentation, so that S-0001's trimming defect cannot apply), a
    comment after the entry's value, separated from it by one space, and every line ending changed
    from CR LF to LF alone (or the reverse). Variant (f): a test file with two entries in its section and the
    digest computed for that order, then the two entry lines swapped without computing the digest
    again. Variant (g): the file of (d) with its digest text split into two entries of the
    integrity digest section, under two different arbitrary keys, the first part in the first
    entry.
  - *Procedure:* start the game, one fresh session per variant; where the main menu appears, play
    one skirmish.
  - *Observable:* whether the main menu appears; whether the skirmish shows the test file's change.
  - *Runs:* one session per variant, four in all.
  - *Expected:* (d), (e) and (g): the main menu appears and the skirmish shows the change. (f): the
    game refuses to start.
  - *Falsified if:* the game refuses to start in (d), (e) or (g), the change is missing from their
    skirmish, or the main menu appears in (f).
- (h) Read per game.
  - *Setup:* two test files as in (d), each with its own valid digest: file 1 changes one visible
    value, file 2 a different one and not the first.
  - *Procedure:* start the game with file 1 in place and play a skirmish. Back in the menus,
    replace it with file 2 without leaving the game, and play a second skirmish.
  - *Observable:* which of the two changes each skirmish shows.
  - *Runs:* one session of two skirmishes.
  - *Expected:* the first skirmish shows file 1's change only, the second file 2's change only (the
    rules are rebuilt for each game, S-0006).
  - *Falsified if:* the second skirmish lacks file 2's change or still shows file 1's.
- (i) The 127-byte cut and the removal of trailing whitespace after it. With base64 characters
  alone the cut cannot be seen: the first 20 decoded bytes come from the first 28 characters of the
  joined text, so a first entry of 128 or more such characters gives the same checksum whether it is
  cut or not. Spaces are used as filler instead.
  - *Setup:* the test file of (d), with its digest text D (28 characters) split over two entries of
    the integrity digest section under two different keys. Variant (i): the first entry's value is
    the first 12 characters of D, then spaces up to a value length of exactly 127 bytes, then the
    four letters "AAAA" (bytes 128 to 131 of the value); the second entry's value is the remaining
    16 characters of D. Variant (ii), the control: the same, but with four spaces fewer, so that
    "AAAA" falls on bytes 124 to 127 and the value ends there.
  - *Procedure:* start the game, one fresh session per variant; where the main menu appears, play
    one skirmish.
  - *Observable:* whether the main menu appears; whether the skirmish shows the test file's change.
  - *Runs:* one session per variant, two in all.
  - *Expected:* (i) the main menu appears and the skirmish shows the change: the cut drops "AAAA",
    and the spaces left at the end of the cut text are removed, so the joined text is D. (ii) The
    game refuses to start: "AAAA" lies within the first 127 bytes and stays in the text.
  - *Falsified if:* (i) refuses to start or lacks the change, or the main menu appears in (ii).
- (j) Repeated names.
  - *Setup:* every variant is a test file built as in (d), with its digest computed by Formulas
    (Integrity digest) under the stated assumption; "lookup" means S-0002's rule. Variant (i): the
    test file's section names the cost key of (d) twice, with two different values, and the digest
    covers both entries in file order. Variant (ii): the same file, with a digest computed over the
    text in which only the occurrence that a lookup finds is written. Variant (iii): the test file
    holds two non-empty sections of the same name, each with one different visible entry, and the
    digest covers both sections in file order. Variant (iv): the same file, with a digest computed
    over the text in which only the section that a lookup finds is written. Variant (v): the file of
    (d) with two integrity digest sections, the first holding the digest computed with the first
    taken out and the second written as an ordinary section, the second holding the stock AI
    database's digest text. Variant (vi): as (v) with the roles swapped: the second section holds
    the digest computed with the second taken out and the first written as an ordinary section, the
    first holding the stock AI database's digest text. Variant (vii): the file of (d) with two
    integrity digest sections that both hold the digest computed with both taken out. Variants
    (viii) and (ix): the file of (d), with its digest text changed so that its final "=" is
    replaced by "A" (that character affects only the 21st decoded byte, which is ignored) and split
    into two entries of 14 characters each; in (viii), the control, under two different keys; in
    (ix), both under the same key.
  - *Procedure:* start the game, one fresh session per variant; where the main menu appears, play
    one skirmish.
  - *Observable:* whether the main menu appears; whether the skirmish shows the test file's change.
  - *Runs:* one session per variant, nine in all.
  - *Expected:* (i) and (iii): the main menu appears and the skirmish shows the change of the
    occurrence a lookup finds. (ii) and (iv): the game refuses to start. Of (v) and (vi), exactly
    one starts: the one whose valid digest is in the integrity digest section that a lookup of its
    name finds. (vii): the game refuses to start (the section that is not taken out is part of the
    text). (viii): the main menu appears and the skirmish shows the change. (ix): the game refuses
    to start (both positions give the same piece of 14 characters, so the text is that piece twice,
    and its first 20 decoded bytes are not the checksum).
  - *Falsified if:* (i), (iii) or (viii) refuses to start, (ii), (iv), (vii) or (ix) starts, or (v)
    and (vi) both start, both refuse, or the one that starts is not the one predicted by the lookup.
    Variant (viii) is the control of (ix): if it refuses, the run of (ix) says nothing about repeated keys.

**Scenario M19: the AI database's digest is not enforced (claim 21a).**
- *Setup:* measurement setup plus a loose copy of the stock AI database. Variant (a): the copy with
  its integrity digest section deleted and nothing else changed. Variant (b): the copy with its
  integrity digest section left as it is, and with every entry of its list of the conditions under
  which computer players form teams deleted, so that its checksum no longer matches. Variant (c),
  the control: measurement setup only.
- *Procedure:* in one fresh session per variant, start the game and play a skirmish for 30 minutes
  of real time at the default game speed against one computer opponent at the hardest difficulty, with superweapons turned off
  in the lobby, on the same map and settings each time, without attacking it.
- *Observable:* whether the game starts and the skirmish plays; whether the computer opponent sends
  any units against the local player's base.
- *Runs:* 10 fresh sessions per variant. The computer opponent's play involves random draws, so
  the statistic is, per variant, the number of sessions in which the opponent attacks.
- *Expected:* the game starts and the skirmish plays in every session. The opponent attacks in at
  least 8 of the 10 sessions of (a) and of (c); in (b) it never forms an attack team, so it attacks
  in none of the 10.
- *Falsified if:* the game refuses to start in any session; the opponent of (b) attacks in any
  session; or the count of (a) is more than 2 below that of (c). If (c) shows an attack in fewer
  than 8 sessions, the run says nothing: play all three variants again for longer.

**Scenario M20: which copy of the stock data files is read (claims 15, 17, 18).**
- (a) The local archive supplies the databases, the mode list, the game-mode files and the interface
  layout file.
  - *Setup:* a loose copy of the stock `localmd.mix` (extracted from the main expansion archive) in
    which these files are edited: the art database, speech database, music database and mode list
    as in Scenario M2 (c); the game-mode file of a mode that the edited mode list still offers, with
    one added override section that raises one unit's cost by 1 credit; the interface layout file
    with the button swap of Scenario M17 (a); and, as controls, the copies of the base rules and of
    the sound database in this archive, edited as in Scenario M2 (a) but for a second unit (its
    cost raised by 2 credits) and as in Scenario M2 (c) (the selection sound).
  - *Procedure:* in a fresh session, open the skirmish lobby, then play a skirmish in the mode whose
    game-mode file was edited, on a map without override sections, in which every edit could show.
  - *Observable:* the five observables of Scenario M2 (c), the two edited units' costs on the
    sidebar tooltip, and the order of the command-bar buttons.
  - *Runs:* one fresh session.
  - *Expected:* the art, speech, music and mode-list edits show; the first unit's cost is raised by
    1 credit (the game-mode file's edit); the second unit's cost is stock (the base rules copy in
    `expandmd01.mix` ranks higher); the buttons are swapped; the selection sound is the stock one
    (the sound database copy in `expandmd01.mix` ranks higher).
  - *Falsified if:* any of the art, speech, music, mode-list, game-mode or button edits does not
    show, the second unit's cost is raised, or the edited selection sound is heard.
- (b) The AI database. Low priority.
  - *Setup:* a loose copy of the stock `localmd.mix` in which the AI database carries the edit of
    Scenario M19 (b).
  - *Procedure, Observable:* as in Scenario M19.
  - *Runs:* 10 fresh sessions, compared with the 10 sessions of the control (c) of Scenario M19.
    The statistic is the number of sessions in which the opponent attacks, as in Scenario M19.
  - *Expected:* the opponent attacks in none of the 10 sessions, while the control's opponent
    attacks in at least 8 of its 10.
  - *Falsified if:* the opponent attacks in any session while the control's count is at least 8.
    If the control's count is below 8, the run says nothing, as in Scenario M19.
- (c) Files under the base game's names are never read.
  - *Setup:* measurement setup plus loose copies of the base game's own rules file and art database
    (the base game's local archive holds them, under names that differ from the expansion's), each
    with one visible edit chosen as in Scenarios M2 (a) and M2 (c): one unit's cost raised by
    1 credit, and one unit's sidebar icon entry naming a different unit's stock icon.
  - *Procedure:* start the game and play a skirmish in which both units can be built.
  - *Observable:* the first unit's cost on the sidebar tooltip and the second unit's sidebar icon.
  - *Runs:* one skirmish.
  - *Expected:* both are stock (a loose file would win if the name were ever requested, claim 1).
  - *Falsified if:* either edit shows.

**Scenario M21: two names with the same member id (claims 6, 7, 22).**
- *Setup:* two new sprite names N1 and N2, each of at most eight letters and digits, such that
  N1 ‖ ".shp" and N2 ‖ ".shp" have the same member id by Formulas, and such that no archive of stock
  data holds either name or its generic fallback name (S-0007). (Computing the id of about 2¹⁷
  random names and looking for a repeat finds such a pair with high probability.) Two sprites with
  clearly different pictures P1 and P2. Variant (a): a test `expandmd98.mix` holding one member,
  stored under the id of N1 (which is also the id of N2), with picture P1, and a test `expandmd97.mix`
  holding one member under the same id with picture P2; a tiny map whose override section points
  one infantry type I2's image at N2, and no type's image at N1. Variant (b): instead, two loose
  files, N1 ‖ ".shp" with picture P1 and N2 ‖ ".shp" with picture P2, no test archive, and a tiny
  map whose override section points infantry type I1's image at N1 and a different infantry type
  I2's image at N2.
- *Procedure:* in a fresh session per variant, play a skirmish on the map in which the infantry
  types it names are seen.
- *Observable:* (a) the picture of I2; (b) the pictures of I1 and I2.
- *Runs:* one fresh session per variant.
- *Expected:* (a) I2 shows P1: the request for N2 is answered by the member stored for N1, because
  only the id is compared, and the higher-ranked archive hides the lower-ranked member with the
  same id. (b) I1 and I2 show the same picture, P1 or P2: the name requested first is used for
  every name with that member id (claim 22).
- *Falsified if:* (a) I2 is invisible or shows P2; (b) I1 and I2 show different pictures.

**Scenario M22: the fallback archives (claims 10, 12, 15, 16).** Each step uses a *probe sprite*:
an extra sprite, added to the test archive the step names, under a new name that, like its generic
fallback name (S-0007), no archive of stock data holds; and a tiny map whose override
section points one infantry type's image at that name. If the test archive is mounted, the infantry
shows the probe sprite; if not, the infantry is invisible. Each variant runs in a fresh session,
plays one skirmish on that map, and records whether the game starts and the infantry's look.
- (a) The sound archive.
  - *Setup:* sound turned on in the game's settings, with a working audio device. A loose test copy
    of the stock `audio.mix` (extracted from `language.mix`) with the probe sprite added. Variant
    (i), the control: nothing else changed. Variant (ii): also, in place of the installed
    `langmd.mix`, a copy of it without `audiomd.mix` (in stock data `audiomd.mix` is held only
    there).
  - *Runs:* one fresh session per variant.
  - *Expected:* (i) the infantry is invisible (`audio.mix` is not mounted while `audiomd.mix` can be
    found). (ii) The probe sprite shows.
  - *Falsified if:* (i) shows the probe sprite, or (ii) shows an invisible infantry. If no sound
    at all is heard in (i), the sound system did not start and the step is void: fix the audio
    setup and run it again.
- (b) The theme archive.
  - *Setup:* a test copy of the installed `theme.mix`, in its place, with the probe sprite added.
    Variant (i), the control: nothing else changed. Variant (ii): also, `thememd.mix` moved out of
    the game directory.
  - *Runs:* one fresh session per variant.
  - *Expected:* (i) the infantry is invisible. (ii) The probe sprite shows.
  - *Falsified if:* (i) shows the probe sprite, or (ii) shows an invisible infantry.
- (c) The movie archive.
  - *Setup:* a loose test archive `movies03.mix`: a copy of the installed `movmd03.mix` under that
    name, with the probe sprite added. Variant (i), the control: nothing else changed. Variant (ii):
    also, `movmd03.mix` moved out of the game directory.
  - *Runs:* one fresh session per variant.
  - *Expected:* the game starts in both variants (one of the two movie archives is enough,
    claim 16). (i) The infantry is invisible. (ii) The probe sprite shows.
  - *Falsified if:* the game refuses to start in either variant, (i) shows the probe sprite, or (ii)
    shows an invisible infantry.
- (d) The base maps archive.
  - *Setup:* a loose test archive `maps03.mix` holding only the probe sprite.
  - *Runs:* one fresh session.
  - *Expected:* the probe sprite shows (`maps03.mix` is mounted when it is present).
  - *Falsified if:* the infantry is invisible.

**Scenario M23: the language archives (claim 5, Table 1 rows 1 and 2).**
- (a) `langmd.mix` is taken only as a loose file.
  - *Setup:* in both variants, the stock string table (extracted from `langmd.mix`) put loose in the
    game directory, so that it is found whatever happens to `langmd.mix`. Variant (i): the installed
    `langmd.mix` moved out of the game directory and put, unchanged, inside a test
    `expandmd98.mix`. Variant (ii), the control: the same test `expandmd98.mix`, with the installed
    `langmd.mix` also left in place.
  - *Procedure:* start the game, one fresh start per variant.
  - *Observable:* whether the main menu appears.
  - *Runs:* one start per variant.
  - *Expected:* (i) the game refuses to start: `cameomd.mix`, which the game requires (claim 16), is
    held only inside `langmd.mix`, and `langmd.mix` is not mounted from the test archive. (ii) The
    main menu appears.
  - *Falsified if:* the main menu appears in (i), or does not appear in (ii). (If `langmd.mix` could
    be mounted from inside the test archive, (i) would start.)
- (b) `language.mix` can come from inside `langmd.mix`.
  - *Setup:* variant (i): the installed `language.mix` moved out of the game directory. In stock
    data no archive holds a member of that name, and `cameo.mix`, which the game requires
    (claim 16), is held only by `language.mix`. Variant (ii): also, in place of the installed
    `langmd.mix`, a copy of it to which the stock `language.mix` is added as a member.
  - *Procedure:* start the game, one fresh start per variant; in (ii), also play a skirmish.
  - *Observable:* whether the main menu appears; in (ii), whether the skirmish reaches play.
  - *Runs:* one start per variant, plus one skirmish in (ii).
  - *Expected:* (i) the game refuses to start. (ii) The main menu appears and the skirmish plays.
  - *Falsified if:* the main menu appears in (i), or in (ii) it does not appear or the skirmish
    never reaches play.

**Scenario M24: the side archive number (claim 13a).** Each step is separate; restore the install
after each. In this scenario an *override map* is a tiny skirmish map whose override section for one
country sets only that country's side field; *the Allied side's name* and *the Soviet side's name*
mean those sides' names as the stock side list writes them.
- (a) A side of index 3.
  - *Setup:* variant (i): measurement setup plus an override map that sets an Allied country's side
    field to a new side name that no rules data uses. Variant (ii): also a loose `sidec04.mix` that
    is a copy of the stock `sidec01.mix` (extracted from `ra2.mix`).
  - *Procedure:* one skirmish on that map per variant, the local player playing that country, each
    in a fresh session.
  - *Observable:* whether the skirmish reaches play; in (ii), the look of the sidebar's radar frame.
  - *Runs:* two skirmishes.
  - *Expected:* (i) the skirmish never reaches play (the new side has index 3, so the game asks for
    `sidec04.mix`). (ii) It plays, with the Allied side's radar frame.
  - *Falsified if:* (i) plays, or (ii) never reaches play or shows another side's radar frame.
- (b) A map override of the side field.
  - *Setup:* the copy of `ra2.mix` without `sidec02.mix` of Scenario M7 (i) variant (ii), plus two
    override maps: map 1 sets an Allied country's side field to the Soviet side's name, map 2 sets
    a Soviet country's side field to the Allied side's name.
  - *Procedure:* one skirmish on each map, the local player playing the country that map changes,
    each in a fresh session.
  - *Observable:* whether each skirmish reaches play.
  - *Runs:* two skirmishes.
  - *Expected:* the skirmish on map 1 never reaches play; the one on map 2 plays.
  - *Falsified if:* map 1's skirmish plays, or map 2's never reaches play.
- (c) A country with no side.
  - *Setup:* variant (i): measurement setup plus a loose base rules file: a copy of the stock one
    with one new country added at the end of the country list, whose section is a copy of an Allied
    country's section without its side field, and which no side list names. Variant (ii), the
    control: the same, but the new country's section keeps the side field.
  - *Procedure:* one skirmish per variant, the local player choosing the new country in the lobby,
    each in a fresh session.
  - *Observable:* whether the skirmish reaches play.
  - *Runs:* two skirmishes.
  - *Expected:* (i) never reaches play; (ii) plays.
  - *Falsified if:* (i) plays, or (ii) never reaches play (then the test country itself is at fault
    and the step must be rebuilt).
- (d) A campaign mission.
  - *Setup:* the two variants of Scenario M7 (i).
  - *Procedure:* per variant, start the first mission of the Allied campaign and, in a fresh
    session, the first mission of the Soviet campaign.
  - *Observable:* whether each mission reaches play.
  - *Runs:* four missions.
  - *Expected:* variant (i), without `sidec01.mix`: the Allied mission never reaches play, the
    Soviet one plays. Variant (ii), without `sidec02.mix`: the Allied mission plays, the Soviet one
    never reaches play.
  - *Falsified if:* any mission reaches play or fails to reach play otherwise than stated.

## Open questions

- When exactly the neutral pair is mounted again during a session (claim 13). It only matters for a
  name collision between the neutral archives and the per-game archives.
- Whether the sound archive fallback (claim 10) depends only on the sound system starting. This is
  irrelevant to data that matters for the simulation.
- Whether interface art other than the four palettes of claim 22 is stored for the session (claim 22)
  or read afresh.
- Layering of the language rules file and of the game-mode files: S-0006.
- When the original handles a change of disc, it can release and mount again some archives, among
  them `main.mix` (which the install does not have), the multiplayer-map archives `multimd.mix` and
  `multi.mix`, and the theme archives `thememd.mix` and `theme.mix`. The number 03 of claim 12 is
  fixed in the original, so a change of disc is not expected to happen, and claim 15's "never mounts `multi.mix`" assumes it
  does not. Not specified here.
- What the original does with an integrity digest section whose text holds bytes outside the base64
  alphabet, or decodes to fewer than 20 bytes (claim 21b). Our engine treats both as a checksum that
  does not match.
- What computer players do when the AI database is missing or empty, and what the skirmish and
  multiplayer screens offer when the mode list is missing (claim 16). Both start the game; their
  later behaviour is not specified.
- The short form of a long name that the original uses to find a map package's description member
  (claim 14a) or to open an alternative rules candidate (claim 20) comes from the host file system
  and is not specified.
- Whether the start of a campaign mission discards the pictures of deferred sprites as a skirmish
  or multiplayer game does (claim 22). This spec's scenarios are all skirmishes.
- For a deferred sprite (claim 22), the number of image frames and the image size of the copy found
  at the first request may be used together with the picture of a copy read later. What is drawn when
  the two copies differ in number of image frames or size is not specified; the scenarios avoid it by keeping both
  the same.
