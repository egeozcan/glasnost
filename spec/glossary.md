---
id: glossary
title: Glossary of shared terms
topic: rules-import        # cross-cutting; every topic uses these terms
revision: 35
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# glossary: Glossary of shared terms

## Summary

**Document identity.** This file, `glossary.md`, is the draft under review, not a supplementary
copy of an approved glossary. Its spec ID is `glossary`; it deliberately has no S-number. Review
its numbered claims G1 to G3 and its shared-term definitions under the same checklist as a
numbered spec. Its front matter records the revision and draft status.

The shared vocabulary of the specs. A spec may use any term below without defining it again. A term
used by one spec only stays in that spec's **Terms** section. Where a spec gives a term a narrower
meaning, the spec says so and wins for that spec. Behavioural claims here cover skirmish only.
Network games and campaigns are out of scope. References to those modes preserve vocabulary,
not rules for implementing them.

## Conventions

- Entries are grouped by game topic, using the topic names in `spec/README.md`. Within each
  topic they are in alphabetical order, ignoring a leading "the". A cross-topic term appears
  once, under its main subject; its pointers still identify the specs that own each rule.
  Spec-writing and measurement vocabulary is grouped under rules-import as shared conventions.
- Units are named in each entry that has one. Numbers below are engine constants of the original
  or definitions of our own units (marked "Definition"), never values from stock data.
- "See S-xxxx" points to the spec that owns the full definition. Any statement in that entry about
  what the original does is a claim of the named spec, and that spec carries its backing,
  confidence and measurement scenario.
- A planned spec is a spec that is neither approved nor a draft of this glossary's batch. It may
  not be written yet, or it may exist as a draft of a later batch; either way it is not signed off
  with this batch. A spec cites a planned spec as
  "planned S-xxxx" or as "S-xxxx (its subject, planned)". A planned spec owns no claim, so a
  pointer to one is a pointer only: in an entry with such a pointer, every statement about the original is a
  claim G1 to G3 or a claim of a written spec that the entry names, and the rest is vocabulary. At
  sign-off every S-number a spec cites must be an approved spec listed in `spec/README.md`, a draft
  in the same batch or a planned spec that the citing text labels as planned in one of the two forms
  above. A draft in the batch is cited by its number and
  claim like an approved spec, never as planned, because calling it planned would take away the
  claims it owns.
- A pointer to a planned spec that has since been approved reads as a plain pointer. The same holds
  for a pointer that calls a draft of the same batch planned. The word "planned" is then ignored,
  the spec pointed to owns its claims as usual, and the next revision of the citing text drops the
  word. This covers every spec of the first two batches, all of them now approved, among them
  S-0010, S-0106, S-0109, S-0201, S-0202, S-0204, S-0205 and S-0206, which earlier revisions of
  this glossary listed as planned; S-0301, which revision 7 listed as planned; and every draft of
  the third batch (combat), S-0301 to S-0315, all now approved, S-0309 among them, which revision
  11 cited as a draft of its batch. A draft of a later batch is still planned.
- This revision cites approved specs at their indexed revisions and the latest revisions of
  them in its current correction batch. S-0401 to S-0416 are now approved; their revised drafts
  in this batch are plain spec pointers, not planned specs. S-0417 is also approved; its revised
  draft in this batch is likewise a plain pointer. S-0501, S-0504, S-0506 and S-0614 are approved. S-0503,
  S-0507, S-0511, S-0512, S-0513, S-0516, S-0522 and S-0524 are approved.
  S-0508, S-0509, S-0510, S-0515, S-0502, S-0514, S-0517 and S-0525 are approved.
  S-0518, S-0519 and S-0520 are drafts in this batch and own the terms attributed to them below. Sign-off checks every cross-spec
  change against the versions handed off together.
- Open questions are questions about the original's behaviour that a spec leaves open. They need
  no Q-ID: Q-IDs number the implementers' spec requests only, and a spec's `requests` field lists
  just the Q-IDs it answers. Specs therefore don't say that no request tracks a question.
  Decisions that belong to the project lead (scope, quirk defaults, the measurement setup, the form
  of a spec) are not open questions: they go in a final list, "Questions for the project lead",
  after the open questions. Once the lead has decided one, the spec applies the decision and drops
  the question.
- The glossary keeps the id `glossary` and lives at `spec/glossary.md`, beside the specs rather than
  under a topic. It takes no S-number.
- An entry otherwise only fixes what a word means in the specs. That is a choice of vocabulary, not
  a claim about the original, so it needs no backing. Where an entry names a rule that has no
  approved spec and no draft in this batch, its
  description is a pointer only: specs must not cite it as a rule.
- The few statements about the original that the glossary makes itself are the numbered claims G1
  to G3 in **Behaviour**, each with its backing, confidence and measurement scenario. They were
  written before any spec covered them. Where a written spec now states the same behaviour in more
  detail (S-0204 for G1, S-0201 for G2, and for G3 the specs its exceptions name and S-0403, claim
  9, for production times), that spec cites the glossary claim, the two agree, and the spec's claim adds the detail. The glossary claim stays
  because other specs cite it; a later revision that finds the two in disagreement corrects both.

## Terms

### rules-import

**absent-key default.** The value a field takes when no data gives its key: the key is missing, or
present with an empty value. It is fixed by the original's program, so it is an engine constant and
specs may state it. When rules are layered, a missing key normally keeps the value already in force
instead (see *rules layer*). For a type section that is the value from an earlier layer of the same
game, or the absent-key default. For a game-wide key, and for a tiberium's key, it can be a value
that an earlier game of the session left behind (see *rules database*). See S-0003 and S-0006.

**art database.** The parsed contents of the expansion's art file: the settings of each art section,
which say how a type is drawn (sprite or voxel model, image-frame ranges, animation settings). A type
finds its art section mostly through its *image name*, and S-0007 gives the cases that use the type
name instead. *Image name* and *art section* are S-0007's terms: an image name names an art section,
not a sprite. See S-0007.

**backing.** How a claim is supported. One of: `measured` (an M-ID records an observation that
confirms it), `measurable` (a measurement scenario in the spec would confirm or falsify it) or
`unmeasured` (it cannot practically be observed, even with the allowances of the *measurement
setup*, such as whether a wait exists inside the original when no observable shows it; each use
must say why). Draw counts and draw order of the random streams are measurable through the
random-state recorder of the measurement setup.

**base rules.** The expansion's main rules file: at every game start, the lowest of the rules layers
(see *rules layer*). It restores every key it gives. A game-wide key it does not give is not reset by it (see *rules
database*). See S-0004 and S-0006.

**bystander computer player.** Measurement vocabulary. A computer player of the original (see
*computer-controlled*) that a measurement scenario includes as the skirmish's opponent, and keeps
away from what it observes. Scenarios write "one bystander computer player" (or "one allied and
one enemy bystander computer player"). The term is used in place of "passive AI house", which is
wrong twice over: the oracle runs the original, which has computer players and no *AI houses*, and
a *passive house* (S-0009, Terms) is a different thing.
- *Not inactive.* A scenario does not assume that the computer player stays idle. It relies on
  the conditions below and on its void rule instead. It may add named test data meant to hold the
  computer player back, but the void rule still applies.
- *Kept away.* Its start position is the one farthest from the test objects, on a test map where
  its objects cannot reach the observed area within the run, unless the scenario places or uses
  some of its objects (for example the allied buildings of a build-off-ally test). Its starting
  credits and starting units follow from the game-wide setup values that the scenario sets for
  every house (S-0412, claims 1 and 2), plus the original's computer-only extras (S-0412, claim
  3). A scenario that needs ample credits for the observed house therefore gives them to the
  bystander too. Whether it is allied with the observed house follows from the lobby teams
  (S-0412, claim 8).
- *Not a passive house.* Its country is not marked passive. So it takes a start position and gets
  starting units like any player (S-0009; S-0412), and it is tested for defeat (S-0414, claim 1).
- *Void rule.* A run is void, and is repeated, when any object of the bystander that the scenario
  does not name enters the observed area or acts on a test object, or when the bystander is
  defeated before the observation ends. Its defeat would run the end test and could end the game
  (S-0414, claim 6). Each scenario names its observed area. Scenarios that fix a void rule of their
  own (S-0203, S-0110, S-0213) keep it.
- *On our engine.* When a scenario is replayed on our engine, the bystander's counterpart is an AI
  house that gives no orders. The void rule makes sure nothing the original's computer player did
  reached the observables.

**claim.** One numbered, observable and testable statement in a spec's Behaviour section. Every claim
carries a backing and a confidence.

**confidence.** How sure the spec author is of a claim: high, medium or low. Low-confidence claims
should have an open question or a measurement scenario.

**departure.** A documented place where our engine deliberately does not reproduce the original,
and offers no quirk switch for it. The spec states the original's behaviour as behaviour and then
our engine's result. Departures cover two cases.
- *Bad data.* Where bad data makes the original crash, stop with a program error, give an
  undefined result, never finish a computation (it hangs) or leave the game frozen for good, our
  engine uses a safe, deterministic result, such as treating the value as absent and using the
  default, and the importer warns (see *importer warning*). Examples: an unparseable number, a
  list shorter than its fixed count, an integer division by zero (S-0205, claim 3), a ranged draw
  over a span of 2³¹ or more (S-0202, claim 5) and a "pause the game" trigger action with
  parameter 0 (S-0201, Edge cases and quirks).
- *Host or real time.* Where the original depends on the host machine or on real time, our engine
  uses a fixed, deterministic behaviour. Examples: the order in which the operating system lists
  files (our engine sorts by the bytes of the names), the host clock that seeds a skirmish
  (S-0202, claim 8), the real-time pacing of cycles, including index 0 (S-0201, claims 4 and 5),
  and the real-time length of a timed freeze (S-0201, claim 10).

A spec may also make a design choice of scope a departure, as S-0005 does for loose files (see
*loose file*) and S-0205 (claim 8) does for the exponent range of intermediate floating-point
results. The project lead accepted the latter with no importer warning, since the importer cannot
tell in general which data would reach it.

**engine add-on.** Anything that changes the original's program behaviour beyond what its data files
control: a patched executable, an injected library, a launcher that alters the game while it runs, or
a third-party extension of the rules language. Data-only changes are *mods*, not engine add-ons. A
presentation-only display compatibility layer is not an engine add-on either, although it may be an
injected library: it changes only how finished drawn frames reach the screen, never the game's logic
or the pixels it draws (see *measurement setup*).

**engine constant.** A number fixed inside the original's program rather than read from data, such as
the number of leptons per cell, a table size, a hard-coded multiplier or an absent-key default. Specs
may and should state engine constants, always with a unit.

**entry, key, value.** An entry is one key/value pair inside a section of a data file. Its key is the trimmed text before the first equals sign, and its value is the trimmed text after that sign. See S-0001.

**field.** One setting of a type, or of a game-wide section, that the original reads from a value.
Each field has a value form (integer, percent, boolean, list and so on, see S-0003) and an absent-key
default. Specs refer to stock numbers only through the field: "the weapon's rate-of-fire field".

**game-mode file.** The rules file that belongs to the selected multiplayer or skirmish game mode. It
is a rules layer that stands, in the list of *rules layer*, above the base rules and the language
rules file and below the map layer. A mode may have none, and a campaign mission has no game-mode layer. In a game whose language rules
file fails its check, the game-mode layer is skipped (see *language rules file*). See S-0006.

**game-wide section.** A rules section that is neither a type's own section nor a type list: the
general settings, the combat-damage settings, the AI settings, the difficulty sections, the land-type
sections and the like. See S-0004 and S-0006.

**import mapping.** The section of a spec that says which keys of the user's data files (rules, art,
maps and other data) feed which concept, with each key's value form and its absent-key default. It is
the only place in a spec where data-file key names may appear, because those names are the user's
data format. The *importer* is the part of our engine that reads those files into our own format.

**importer warning.** A report that the importer gives the user about one input, naming the file and
the place in it. A warning never changes the imported result: the result is the one the owning spec
gives, either the original's behaviour or, for a *departure*, our engine's safe deterministic result.
The owning spec says which inputs warn. Examples: a duplicate section or key, a byte-order mark, a
file whose lines end in a carriage return only, a key written in a different letter case from the
one the original reads, input that the original trims wrongly (S-0001, S-0002, S-0003), and bad data
that a departure replaces with a default. The data-file quirks of the original apply to every input,
stock data, user maps and mods alike, and stay on by default; the warning is how a mod author learns
of them.

**language rules file.** An optional rules file of the original. When a file of that name can be
resolved and is valid, it is a rules layer that stands, in the list of *rules layer*, above the base
rules and below the game-mode file. A file that holds at least one section is valid only if it carries a matching
integrity digest. The file is checked at program start and again at every game start, and what
happens to one that fails depends on when it fails (S-0006, claim 12). Failing at program start, it
makes the program's start-up fail. Failing at a later game start, whether it first appeared after
program start or was valid then and has since been edited or replaced, it is ignored for that game,
and so is that game's game-mode layer. Stock data has none. See S-0005 and S-0006.

**layer.** Ambiguous on its own. Write *rules layer* for a rules file applied on top of earlier ones,
*render layer* for one of the ordered drawing lists, or *cell layer* for ground or bridge-deck
occupancy.

**loose file.** A file stored directly in the game directory, outside any archive. A loose file is
found before an archive member of the same name, so a loose file can override shipped data. Our
engine looks for loose files in the game directory only. The original also looks in a few legacy
places outside it; S-0005 claim 4 makes not doing so a *departure*, with no switch. See S-0005.

**map layer.** The map file as a rules layer: the highest of the rules layers of a game (see *rules
layer*), so its rules sections normally override the lower layers, subject to S-0006's
section-specific exceptions. In particular, a colour name already defined keeps its first
definition, comparing names without regard to letter case (S-0103, claim 5; S-0006, claim 2).
S-0103 owns `quirks.colour_redefinition_ignored` (default original); with it off, a later
definition replaces the value while keeping its position. What it sets in type sections lasts for that game only, except in
tiberiums' sections: what it sets there, and any tiberium it adds, lasts for the rest of the session
(S-0006, claim 10). What it sets in game-wide sections can outlast the game too (see *rules
database*). See S-0006.

**measurement.** An observation of the original, recorded by the project lead as an M-xxxx record in
the project. It holds only observables: timings in ticks, credits, counts, cells, health values, win
rates and the like. When a measurement disagrees with a spec, the measurement wins and the spec gets a
new revision.

**measurement scenario.** A spec's proposal for confirming a measurable claim. It gives the setup
(map, houses, objects, game speed, settings), the procedure, the observable, the number of runs, the
expected result and the result that would falsify the claim.

**measurement setup.** The conditions every measurement runs under, unless a scenario says
otherwise. The original is installed unmodified: its files match the release's published file list.
Only stock data is used. No loose files override shipped data, and no engine add-on is active.
Purpose-made test maps are allowed. Game speed and screen resolution are free unless the scenario
fixes them. Five further allowances apply:
- *Display compatibility layer.* A presentation-only layer may sit in the display path. It changes
  only how finished drawn frames reach the screen (for example whether showing a drawn frame waits for
  the monitor's refresh), never the game's logic or the pixels the game draws, so it is not an engine
  add-on. The measurement record names it generically, by what it does, not by product.
- *Named test files.* Where a scenario needs extra test data, it names it, and the test data is
  removed after the run. Named test data may be a test map's override section, a test file, or a
  loose file that overrides shipped data on purpose: for example a loose test archive that wins over
  the stock archives, a loose copy of a stock archive with one member left out to make that file
  absent (S-0007, S-0105), a loose palette, a palette removed from its archive, or a loose copy of a
  rules file. Such files are the one exception to the rule above, and only for the scenario that
  names them.
- *Statistical runs.* Nothing is placed inside the original to fix its random state. A sub-test
  whose result depends on the random seed is measured statistically instead, over many runs, and
  its scenario states the number of runs and the statistic that confirms or falsifies the claim,
  unless it reads the random state with the state recorder below.
- *Random-state recorder.* A program may read, while the original runs, the seed and the state of
  its random streams, and log them with the tick number, so that a scenario can count draws and
  check their order and values. It only reads: it never sets the seed or the stream state, so it is
  a *recorder* and not an engine add-on.
- *Scripted input.* Mouse and keyboard input may be produced by a script instead of a person, for
  exact timing or for many runs. It acts only through the game's normal mouse and keyboard input,
  as a player would, and never writes into the game, so it is not an engine add-on.

A claim that was unmeasured only because it needed named test files, the random-state recorder or
scripted input is therefore measurable.

A scenario's opponent in the original is a *bystander computer player* (see that entry). A
scenario that needs a *passive house* names it as a test object, as S-0414 (scenario V3) does.

**mod.** A change to the data files only: rules, art, maps, images or sounds. Our engine supports
mods through the importer. A mod is never part of stock data or of a measurement setup, except the
named test files a scenario uses (see *measurement setup*).

**oracle.** The original, used as the reference that decides what correct behaviour is. Measurements
are taken from the oracle.

**the original.** The reference game: *Command & Conquer: Yuri's Revenge* version 1.001 as shipped,
meaning the reference executable together with its stock data. "Original behaviour" is what this
release does, defects included. "YR behaviour" means the same.

**override section.** A rules section inside a game-mode file or a map that changes the rules from
that game on. It has the same name as the section it overrides. An override of a type section lasts
for that game only, except an override of a tiberium's section, which lasts for the rest of the
session (S-0006, claim 10). An override of a game-wide section can outlast the game too (see *rules
database*). See S-0006.

**quirk switch.** A named setting of our engine, written `quirks.<name>`, that controls one unusual or
defective behaviour of the original. Its default is always the original behaviour. The spec that
names a switch describes both the original behaviour and the alternative ("fixed") behaviour that the
switch selects when turned off. Where the original crashes, stops with a program error, gives an
undefined result, hangs or leaves the game frozen for good on bad data, or depends on the host
machine or on real time, there is no switch: that is a *departure*. A switch shared by
several specs is named once in each of them; the glossary points to the shared ones (Edge cases and
quirks).

**recorder.** Whatever a measurement uses to observe the original without changing it: a stopwatch, a
screen or video capture, or a program that only reads the running game's state, such as its count
of ticks run or the seed and state of its random streams (the *random-state recorder* of
*measurement setup*). A recorder never writes into the game, injects code, changes its timing,
fixes its random state or gives orders. It is therefore not an engine add-on, and the measurement
setup allows it. Scripted input, which does give orders, is not a recorder but a separate allowance
of the measurement setup.

**registry, registration index.** The registry of a type kind is the ordered collection of all types
of that kind in the current game. A type's registration index is its position there, a non-negative
integer counted from 0. Indices are dense: the *n*-th type created for a kind has index *n* − 1.
See S-0004.

**reset on override.** A property of a few rules keys: when a rules layer contains the key's
section but omits the key, the value is set to a fixed reset value instead of keeping the value in
force. The reset value is a fixed engine default, or the current value of another field. For some
keys the reset applies in the base rules too, not only in later layers. See S-0006 (claim 5) and
S-0105 (claim 27).

**rules database.** The parsed rules data in force during a game. At every game start the rules
layers (see *rules layer*: the base rules, the language rules file if a valid one exists, the
game-mode file in a skirmish or network game unless the language rules file fails its check at a
later game start (see *language rules file*), and the map layer, from lowest to highest) take effect on the rules state already
there. In a campaign mission the mission-named file changes that state too, below every layer of
the mission, but it is not a rules layer and most of its changes are lost (S-0006, claim 14).
Every type registry except the tiberiums' starts each skirmish empty, so its other type sections
hold only what this game's layers gave. The deferred campaign exception is the sides newly
created by the mission-named file: they remain ahead of the base rules' sides for that mission,
but no side of the previous game survives (S-0006, claim 14). This exception preserves context
only; campaigns remain out of scope.
Game-wide sections are not cleared at a game start, apart from the resets S-0006 lists (for
example, the colour-adjustment table depends only on this game's layers, as the table in S-0006
claim 2 gives, never on earlier games). So a game-wide key that the base
rules do not give keeps, in later games of the same session, the value an earlier game's game-mode
file or map gave it. Tiberiums carry over between games in a similar way (S-0006, claim 10). A fresh
program start clears both. See S-0005 and S-0006.

**rules layer.** One rules file applied to the current rules state. The layers of a game form one
ordered list, from lowest to highest: the base rules, the language rules file (if a valid one
exists), the game-mode file (skirmish and network games only, and see *language rules file*) and
the map layer (S-0006, claim 1). A layer changes only what it mentions, with two kinds of
exception: the reset cases that S-0006 (claim 5) and S-0105 (claim 27) list (see *reset on
override*), and the light values that S-0104 (claim 12) converts when a higher layer reads their
section again without giving them. That conversion truncates toward zero to whole units, with
+1 thousandth added for building-light intensity and tint when the truncated whole-unit value
is negative; extra light gets no such addition. Specs write "higher" and "lower" for places in
this list; "later" and "earlier" mean the same. The list decides four observable things:
- *Overrides.* Where two layers give the same key, the value of the higher one is normally in
  force, subject to the section-specific exceptions (S-0006, claims 1 to 5). Colour definitions
  are one exception: the first definition of a name wins, comparing names without regard to
  letter case (S-0103, claim 5; S-0006, claim 2). S-0103 owns the alternative under
  `quirks.colour_redefinition_ignored`, as described under *map layer*.
- *Registration order.* A type first named by a lower layer has a lower registration index than a
  type of the same kind first named by a higher one; a higher layer can only add types at the end
  of a registry (S-0004, claim 4; S-0006, claim 6).
- *Section reads.* A type never reads its own section from a layer below the one that created
  it, so a type that a higher layer adds takes nothing from a lower layer's section of the same
  name (S-0006, claim 7).
- *Empty layers.* An applied layer that gives nothing (a game-mode file that cannot be resolved or
  holds no section, or a language rules file with no section) changes no rules value but still
  counts: in that layer, every animation that already exists when the animations are read takes its
  settings from the art file once more. The number of these reads in a game is observable,
  because it can change which art section's theater and new-theater flags are in force for the
  animation (S-0006, claims 1 and 8; S-0007, claim 2).
See S-0006.

**section.** A named group of entries in a data file, introduced by a header line `[name]`. See
S-0001.

**settings file.** The user's options file in the game directory that holds the player's own
settings, such as the game speed (see Import mapping) and the difficulty. It is a user file, not
stock data, so a scenario may set its values.

**side.** One of the faction groups, listed in the rules data, that countries belong to (S-0004,
claim 9). The local player's side selects the side group of archives, whose art gives the sidebar
its look (S-0005, claim 13). Which types a side's houses may build is S-0407's. See S-0004 and
S-0005.

**stock data.** The unmodified data files of the release of the original: the shipped archives and
the rules, art, AI, sound, game-mode and map files inside or beside them. Specs never copy values from
stock data. They refer to the field instead, and a value in an example is invented.

**type.** One kind of thing defined by the rules data, such as one infantry, one weapon or one
warhead, with its own section and fields. Every object is an instance of a type.

**type kind.** One family of types with its own registry: countries, sides, superweapons, buildings,
vehicles, aircraft, infantry, weapons, projectiles, warheads, animations, voxel animations, particles,
particle systems, terrain objects, smudges, overlays and tiberiums (the resource types, such as ore
and gems). See S-0004.

**type list.** A rules section whose entries name the types of one kind, in order. The values are
type names, and the keys normally only give positions. The side list and the tiberium list are
exceptions: their keys carry meaning (S-0004, claims 9 and 10). See S-0002 and S-0004.

**type name.** The identifier that names a type in the rules data: the value in a type list, the name
of the type's own section, and the name other fields use to refer to it.

**type section.** The rules section named after a type, holding that type's fields. See S-0006.

**unspecified.** The original produces a value that depends on unrelated internal state and cannot be
reproduced. A spec that calls a result unspecified also says what our engine does instead: a safe,
deterministic result, with an importer warning where the cause is in the data (see *departure*).

**YR.** Short for *Yuri's Revenge*, the expansion of *Command & Conquer: Red Alert 2*. In the specs,
"YR" and "YR behaviour" refer to the original.

### formats

**archive.** A container file holding named members, in the public "MIX" format. An archive can be a
member of another archive (a *nested archive*). When a file is requested, a loose file of that name
takes precedence over any archive member, and among the mounted archives a ranking decides which
member is used. The ranking follows the order in which archives are mounted, so it changes during a
session (S-0005). See S-0005.

**cell.** One square of the map grid, identified by integer cell coordinates (x, y). Increasing x
runs towards the lower right of the screen, increasing y towards the lower left. A cell is 256
leptons along each map axis, and its centre is at world position (256x + 128, 256y + 128) leptons.
Its corner topmost on the screen is its *top corner*, the point (256x, 256y) leptons, which S-0009
(Terms) calls its *cell corner*. See S-0009 and S-0101.

**cell animation.** An animation that the rules data attaches to an overlay type and that starts
when an overlay of that type is placed normally on a cell, map loading included, at a position
near that cell. Low-bridge end markers and their direct end-row and piece writes do not start
cell animations, even when the written types name them (S-0202, claims 10 and 14). Not the same
thing as a *terrain animation*. S-0009 owns the ordinary placement and position (Terms, claim 12).
S-0202 gives its random draws and the low-bridge exclusions.

**height level.** A cell's terrain elevation, an integer. A map file may give any level from −128 to
127 (S-0009, claim 7). One height level is 104 leptons of world height (S-0204, claim 1).

**legal cell.** A cell that exists on a given map. Which cells are legal follows from the map size
(S-0009, claim 1). See S-0009.

**local area.** The screen-aligned sub-rectangle of a map in which play takes place and to which the
view's scrolling is limited. It is given by four integers (left, top, width, height) in the same units
as the *map size*: horizontal numbers count cell widths (60 pixels on screen), vertical numbers count
cell heights (30 pixels on screen). See S-0009 and S-0101.

**map size.** The screen-aligned rectangle that bounds a map, given by its map width and map height:
the width counts cell widths (60 pixels on screen) along a screen row, the height counts cell heights
(30 pixels on screen). See S-0009 and S-0101.

**overlay.** A cell-level map object kind that is not terrain and not a building: walls, fences,
bridge parts, crates, ore and gem fields and similar. Each overlay cell has an overlay type and an
overlay data value. See S-0009.

**overlay data value.** An integer from 0 to 255 that every legal cell has, alongside its overlay
type (S-0009, claim 13). Its meaning depends on the overlay type: for
example the stage of a resource cell (S-0401, claim 1) or the damage stage of a wall. Cells without an overlay
still have one. See S-0009.

**resolve.** To find the single copy of a requested data file that the game uses: a loose file of
that name if one exists, otherwise the archive member that the archive precedence of
S-0005 selects. A name that
resolves to nothing is "not found". See S-0005.

**slope.** A cell whose ground is inclined rather than flat. Its slope shape comes from its terrain
tile (S-0010, claim 6). S-0106 owns the slope shapes and the surface height within the cell (claims
1 and 2). Slopes affect height within the cell, drawing and movement. See S-0009, S-0010 and S-0106.

**sub-tile.** A terrain tile can cover several cells. A cell's sub-tile is the index of the part of
its tile that lies on that cell. See S-0009.

**terrain animation.** An animation that the game creates as part of a cell's terrain, when the
map's terrain is set up or when a cell is recalculated in play (S-0010, claim 19), such as the
animation of a tile that the theater animates, rather than one created by an object or an event. Not the same thing as an *animated terrain object*, which is a
terrain object. See S-0103 and S-0104.

**theater.** The terrain set of a map, one of six. It selects the terrain tiles and palettes, and it
changes the names of some art files. See S-0007, S-0103 and S-0010.

**tileset.** A named group of consecutive terrain tiles declared in a theater's data, such as one
cliff set or one road set. A theater's tiles are numbered consecutively across its tilesets in
declaration order. See S-0009.

**top corner.** Of a cell (*x*, *y*): its corner at world position (256*x*, 256*y*) leptons, the
corner with the smallest x and y. It is the cell's map north-west corner and its topmost point on
the screen. Of a foundation: the top corner of its top-left cell, which is the topmost point of the
footprint on the screen. Specs write "top corner", not "north-west corner", for this point.

**usable area, usable cell, playable area.** The part of the local area in which play takes place. A legal cell is
*usable* when the test of S-0009 (claim 4) puts it inside the local area. S-0009 (claim 5) says
which uses take the test's height-aware form; cursor orders, object placement and the nearby-cell
search do (claim 5a). *Playable area* (S-0009, S-0010, S-0213) means the same. See S-0009. S-0211
(claim 11) uses it for the off-map passability category.

### simulation

**AI house.** A house of our engine that our own AI plays. It takes the place of the original's
computer player, whose script and team system is out of scope (see *team member*). By decision of
the project lead, an AI house has the same unit rules and the same order legality as a
human-controlled house. This is a rule of our engine, not a claim about the original.
- *Orders.* An AI house acts only through orders, and its orders are applied, although the
  original drops orders issued in the name of a computer-controlled house (S-0213, claim 3). They pass the same
  legality rules as a person's (S-0214) and the same checks when they are applied (S-0213), with
  no exemption from the pending-order limit. The original checks the actor's owner only for
  sell orders; AI-house orders follow that same rule (S-0213, claims 12 and 15).
- *Unit rules.* Its units follow the rules that the specs give for units of a human-controlled
  owner. A behaviour rule that the original applies only to units of a computer-controlled owner
  (see *computer-controlled*) does not apply to them, and the recruitable property of S-0215
  (claim 20) has no effect.
- *The original's computer behaviours.* What the original's computer player gets from such rules is
  recorded in the specs as behaviour of the original. Our AI may choose to reproduce it by giving
  ordinary orders.
- *Player houses only.* The ruling covers the houses of the players. The two non-player houses that
  every skirmish creates, the *neutral house* and the *special house* (see *house*), are not AI
  houses: they keep the original's rules for owners that are not human-controlled (decision of the
  project lead, 2026-10-10; for example S-0208, claims 11 and 15, and S-0107, claims 14 and 15).

The decision was taken on claims of S-0213 and S-0215, and their approved revisions apply it there. It is
a general rule of our engine, so it holds in every spec. Where a spec gives a rule for
computer-controlled owners, that rule describes the original, and an AI house follows the rule the
spec gives for human-controlled owners. Specs that give such a rule mark it as the original's
only, except for units of the neutral and special houses, which keep it in our engine
(see *Player houses only* above). In particular an AI house has the house skill level of a human-controlled house:
0 in a skirmish or network game (see *house skill level*). By default it also uses the *normal
row* of the difficulty skill block, as a human-controlled house in a skirmish does (see *difficulty
row* and *difficulty skill block*).

**alliance, allied, mutually allied.** An *alliance* is a one-way relation between two houses:
house A *allies* house B when A counts B as a friend. A and B are *mutually allied* when each allies
the other, and a house counts as allied with itself. S-0412 owns the terms and the alliances a
skirmish starts with (Terms, claim 8), and S-0414 how alliances change during play (claims 12 and
13). S-0209, S-0210, S-0214 and S-0314 define *allied* in their own Terms for their own tests, each
from the side it names (S-0210 always from the crusher's side, S-0214 for objects as well as
houses); S-0409 (Terms) takes S-0314's sense.

**behaviour.** One of the standing orders that an actor (an infantry, vehicle, aircraft or building
object) carries out, such as *move*, *attack*, *guard* or *area guard*. S-0215 names all of them
and gives each actor a current, a queued and a suspended behaviour (Terms, claim 1). In the specs a
behaviour name in italics means the behaviour, never the sequence kind of the same name in S-0107.
See S-0215; orders that set a behaviour are S-0213's.

**capacity (transport).** A transport type's imported passenger count, an integer
(S-0507 claim 1). Not a harvester's resource capacity.

**computer-controlled.** A house of the original that its computer player plays rather than a
person. The specs use the word for the original: a rule given for a "computer-controlled owner"
states what the original does for units of such a house. In our engine the computer plays *AI
houses*, which follow the unit rules and order legality of human-controlled houses (see *AI
house*). Units of the neutral and special houses keep such a rule in our engine too (*AI house*,
*Player houses only*). Keeping these rules does not give the neutral and special houses the maximum
*house skill level*: they keep level 0 (S-0412 claim 3). The opposite of *human-controlled*.

**country.** The nationality a house plays, chosen from the countries listed in the rules data. A
country belongs to one side (S-0004, claim 9). Which types the houses of a country may build is
S-0407's (claims 5 and 11), and what each field of a country does is S-0413's (claim 1).

**country factor, per-kind factor, general factor.** A *country factor* is a factor that a
country's rules section gives. A *per-kind factor* holds one factor for one kind of item
(infantry, vehicles, aircraft, ordinary buildings or defences), a *general factor* one factor for
every kind. S-0413 owns the terms and which factors have an effect (Terms, claims 1 to 3); S-0403,
S-0404, S-0408 and S-0412 use them.

**cycle.** The unit of real time in which the original runs a game. Cycles follow one another
without overlap. A cycle that is not frozen takes the player's input and runs one tick. A *frozen*
cycle, during a freeze, runs no tick and takes no input. In a single-player game every cycle makes
one drawn frame. While the game stands still (the game window has lost the input focus, or the
options menu is open) there are no cycles. See S-0201 (Terms, claims 2, 6, 8, 9 and 10). In rules
about ticks, drawn frames and real time, "cycle" means this. A spec that defines another kind of
cycle, such as the recoil cycle of S-0105, the firing cycle (S-0301) or a cycle of image
frames, names it in full where it defines it and may then shorten it.

**defeated, undefeated, contender.** A house is *defeated* once the defeat of S-0414 (claim 5) has
struck it, and *undefeated* otherwise. A *contender* is an undefeated house still in the game
that is not passive. A house removed by unused-house cleanup is absent, not an empty contender
(S-0009, claim 28; S-0414, Terms and claim 1).
S-0414 owns the terms (Terms, claims 1 to 10); S-0409 and S-0410 use *defeated*.

**difficulty row, normal row.** One entry of the *difficulty skill block*: the factors of one
difficulty, easy, normal or difficult. The multipliers a house takes from its row are fixed when the
houses are set up (S-0302, claim 3). The *normal row* is the row
of the normal difficulty, which the original gives every human-controlled house in a skirmish; by
the decision recorded under *difficulty skill block* it is the row every player house uses in our engine by
default. Rows are applied to player houses only; the neutral and special houses have none (factors
1.0; S-0412 claim 5). S-0302 owns the terms (Terms, claim 3); S-0301 (claim 3) uses them.

**difficulty skill block, skill block.** In the original, the factors that scale a house's rate of
fire, firepower, armour, cost and build time, given once for each difficulty. There is one block,
and its entries, one per difficulty (easy, normal, difficult), are the *difficulty rows*. S-0301 (claim 3) owns the rows' rate-of-fire
factor and S-0302 (claim 3, Edge cases) their firepower and armour factors; S-0404 (claim 2) gives
what the rows' cost factor does and S-0403 (claim 9) their build-time factor. It is not the *house
skill level*. Which row
a house takes is set-up behaviour of the original, recorded by the specs that use the factors
(S-0301 and S-0302, claim 3). By decision of the project lead, in our engine every player house, human or
AI, uses by default the row the original gives a human-controlled house in a skirmish, the *normal
row*. AI difficulty comes from the AI's strategy modules, never from these factors. Choosing
another difficulty row is available only as an optional per-player handicap setting, off by
default, and is never tied to AI difficulty automatically (decided 2026-10-10). This is a rule of
our engine, not a claim about the original. In a skirmish, a difficulty row that a map sets for a
house is ignored; only the optional handicap applies (decided by the project lead, 2026-10-10).
Rows in a campaign mission are deferred to the future campaign spec, since campaigns are out of
scope for now (decided by the project lead, 2026-10-10).

**direction.** The direction of a point *Q* from a point *P* is the facing that the original
computes for the way from *P* to *Q*. It is close to the exact angle, but not always equal to it.
See S-0204 (claim 8, Formulas F3). One of the eight ways from a cell to a neighbouring cell is a
*step direction*, and a facing rounded to one of *n* equal steps is an *n-step facing*.

**disableable superweapon.** A superweapon type whose imported disableable flag lets the setup
screen's superweapon setting (see *setup values*) take it out of the game. S-0412 owns the term
and what the setting does to such superweapons and to the buildings that name them (Terms, claim
9). S-0407 (claim 7) and S-0413 use it. Specs write "disableable superweapon", not "switchable
superweapon", because *switched on* and *switched off* name a building's power switch.

**facing.** A direction, given as an integer *f* from 0 to 65535 that stands for *f*/65536 of a full
turn in the map plane (not an angle measured on the screen, where the isometric projection
distorts angles): one facing unit is 1/65536 of a turn. 0 is *map north* and values grow clockwise as seen on the
screen, so 16384 is *map east*, 32768 *map south* and 49152 *map west*. Map north is the direction of
decreasing cell y (towards the upper right of the screen), map east the direction of increasing cell
x (lower right), map south increasing y (lower left) and map west decreasing x (upper left).
Compound names follow: map north-west is straight up the screen. Arithmetic on facings wraps modulo
65536. The 65536 units per turn are our own unit, a definition. That the original uses this same
zero and sense is claim G1. S-0204 owns the rest (claims 10 and 11, Formulas F4 and F5): that the
original's facings retain the full resolution of one facing unit, how a coarser *n-step
facing* (*n* a power of two from 2 to 256) is derived from a facing, and how two facings are
compared (the *signed facing difference*). A spec that derives a coarse facing in another way says
so.

**game.** One match, from the moment the map is loaded until it ends. Several games can run in one
session.

**game minute.** The unit in which some data fields give durations. The original converts a game
minute to ticks at a fixed 900 ticks per game minute, an engine constant; the specs that convert a
minutes field state it (S-0107, S-0108). Other specs may give values in game minutes without
converting them, as S-0006 does for some reset values. A game minute is a conversion unit, not a minute of
real time: at game speed *s* from 1 to 6 a host that keeps up runs 62.5 / *s* ticks per second, so a
game minute lasts 14.4 × *s* seconds of real time, exactly one minute at no index from 1 to 6
(S-0201, Formulas F1); at index 0 the original's tick rate depends on the host (S-0201, claim 5). Specs write "900 ticks per game minute", not a "nominal" tick rate.

**game speed.** The player's speed setting, as an integer index *s*. 0 is the fastest and larger
values are slower (claim G2). The speed controls of the skirmish lobby and of the options menu offer
0 to 6 (S-0201, claims 7 and 8), and the claims here cover 0 to 6. The settings file can hold other
values (Import mapping), and no claim here covers them. The game-speed normalisation table of S-0107
(Constants, T3) has a column for index 7, which lies outside the covered range.
- *Where it comes from* is owned by S-0201. In a skirmish it is the position of the lobby's speed
  control, which starts at the index saved in the settings file's skirmish section; the settings
  file's general game-speed value does not decide it (claim 7). A campaign mission always runs at
  index 2 (claim 12). Network games are a pointer only (claim 13).
- *Ticks per second.* In a single-player game at index *s* from 1 to 6 a cycle does not end before
  a pacing clock of 16 ms units has advanced by *s* units since the cycle began, so a host that
  keeps up runs 62.5 / *s* ticks per second. Index 0 adds no wait of its
  own (S-0201, claims 2 and 5, Formulas F1; claim G2). Our engine paces every cycle instead: it
  lasts exactly 16 × *s* ms at index *s* from 1 to 6 and 8 ms at index 0, or as long as its work
  takes if that is longer. This is a *departure* of S-0201 (claims 4 and 5) with no switch, not a
  claim about the original.
- *Timings in ticks.* Game speed changes no timing of game state counted in ticks, with the two
  exceptions of claim G3: the speed-normalised delays and whatever waits on them, and the number of
  ticks a chrono flash runs before its own freeze when a freeze falls within it. Our engine
  reproduces both by default. In our engine game speed reaches game state only through the one
  switch `quirks.speed_normalised_delays`, which covers both exceptions (claim G3, Edge cases and
  quirks).

**game start.** The moment a new game (a skirmish, a network game or a campaign mission) begins
loading. The rules layers are applied at every game start. In a skirmish every type registry
except the tiberiums' starts empty (S-0004, claim 1; S-0006, claims 1 and 10). S-0006 claim 14
records a deferred campaign exception: sides newly created by the mission-named file remain ahead
of the base rules' sides for that mission, but no side of the previous game survives. This is
context only, not a campaign implementation rule. Specs write "game start", not "scenario start"
("scenario" would clash with *measurement scenario*).

**guard spot.** The cell or object that an actor in *area guard* stays near. See S-0215 (Terms,
claim 26).

**handicap.** Our engine's optional per-player setup value that picks a *difficulty row* for that
player's house; it is off by default and never tied to AI difficulty (see *difficulty skill
block*). S-0412 owns it (Terms, claim 5); S-0403, S-0404 and S-0413 say what it does not change.
This is a rule of our engine, not a claim about the original.

**house.** One owner of objects in a game: it has credits, power, a colour, a country and an
alliance state with every other house. Each player, human or computer, is one house. A skirmish game
also always creates two non-player houses, after the players: the *neutral house* and the *special
house*, which own neutral map objects (S-0009, claims 19 and 27). The program creates them itself
and takes their countries from the rules data. S-0009 gives how those countries are found, the
order in which all houses are created and what that order decides (claim 19), and which of the two
an unused-house cleanup can remove after they are created (claim 28).

**house skill level.** An integer per house that gates some automatic behaviours of its units,
by comparison with game-wide thresholds from the rules data. In a skirmish or network game the
house of every computer player (a lobby computer slot) gets the rules' maximum skill level when the
houses are set up; human-controlled houses and the neutral and special houses keep the level 0 that
every house starts with, in our engine too (S-0209, claim 11; S-0215, claim 15; S-0412, claim 3). The maximum's absent-key default is 5 (S-0209, Import mapping). In a campaign mission a
house that the map defines takes its level from the map (S-0215, claim 15). S-0215's *AI level* is
the same value; specs write "house skill level". In our engine an *AI house* in a skirmish or
network game has the level 0 of a human-controlled house, not the rules' maximum, because AI houses
follow the unit rules of human houses (see *AI house*). So every rule gated by the skill level
treats its units as it treats a human player's: for example the area-guard row of S-0215 (claim
15), the cell-wide scatter rule of S-0209 (claim 11) and automatic crushing in S-0210 (claim 11).

**human participant.** A person who takes part in a game through the lobby, observers included.
S-0412 owns the term (Terms); S-0414 uses it.

**human-controlled.** A house played by a person rather than by the computer. A rule that depends on
this says "human-controlled owner" or "computer-controlled owner" (see *computer-controlled*). In our
engine an *AI house* counts as human-controlled for every unit rule and for the legality and
checking of orders; whether other rules that name human-controlled houses, such as a trigger action
that acts on every human-controlled house, also cover AI houses is said by the spec that owns them.

**in play, enter play, leave play; on the map, off the map; destroyed, removed.** Three different
questions about an object, which specs keep apart.
- *In play*: the object is in the update order and takes turns. It *enters play* when it joins
  that order and *leaves play* when it leaves it. S-0203 owns the terms and when they happen
  (Terms, claims 2, 3 and 6). S-0107 (claim 17) and S-0312 (claims 10, 15, 16 and 21) add when
  killed objects leave play: a sinking or crashing object, an infantry unit playing its death
  sequence and some destroyed buildings stay in play for a while after the killing hit.
- *On the map*: the object is placed in the world at a position. It is *off the map* while it
  exists without being placed there, for example inside any transport, open-topped or not, or
  inside a building.
- *Destroyed* and *removed*: a destroyed object has been killed. A *removed* object has left play
  for good after it was destroyed or deleted: it no longer exists. When an object is removed is
  S-0203's claim 6 and the claims named above.

The three differ. A passenger of an open-topped transport is off the map and in play (S-0203,
claim 3); a passenger of any other transport, or an object inside a building, is off the map and
out of play, but neither destroyed nor removed. A killed object that stays in play is destroyed
but not yet removed. A spec writes "off the map" or "inside" for an object held without being
placed, "not removed" (or "still exists") for an object that has not gone for good, and "in play"
only in S-0203's sense.

**kept passenger.** The passenger a vehicle transport with a type turret count above 0 keeps
inside during an unload, subject to the single-passenger exception (S-0507 claim 11).

**lepton.** The unit of world distance: 1/256 of a cell edge. Positions, ranges and speeds are given
in leptons (speeds in leptons per tick). World height is also measured in leptons.

**load (vehicle transport).** The whole-number sum used to count its passengers' sizes,
truncated toward zero after each addition. For non-negative sizes and representable sums,
this equals the sum of each passenger's size rounded down (S-0507 claim 1).
Not a harvester's resource load.

**local player.** The house of the person in front of this machine. In our engine only what is drawn
or heard may depend on it, never game state; each spec that uses it says what depends on it. See
S-0107.

**multiplayer game, network game.** A game played by several machines together, over a local
network or the internet, as opposed to a *single-player game*. The two names mean the same, so
"skirmish or multiplayer game" means every game that is not a campaign mission. A mode or setting
named for multiplayer (a multiplayer game mode, a country marked passive for multiplayer) can apply
in a skirmish too, and its spec says so. How the original runs network games (the agreed tick
rate, the synchronisation draws) is left by S-0201 (claim 13) and S-0202 (claim 12) to a future
network-game spec.

**n-step facing, 8-direction.** A facing rounded to the nearest of *n* equally spaced directions,
*n* a power of two from 2 to 256, numbered 0 to *n* − 1 clockwise from map north. The 8-step facing
is the *8-direction*; it is a *step direction*. See S-0204 (claim 10, Formulas F4), which owns the
rounding.

**object.** Anything that the game tracks individually, on or off the map: infantry, vehicles,
aircraft, buildings, overlays, terrain objects, smudges, animations, projectiles and so on. Every
object is an instance of one type.

**open-topped transport.** A vehicle or aircraft transport whose type has the imported
open-topped flag (S-0507 claim 14). Its passengers fire from inside it. S-0203 claim 3 owns
their place in the update order; S-0507 claim 15 owns position and target hand-down, and
S-0307, S-0308, S-0311 and S-0313 own its combat effects in their topics.

**passenger order, position 1.** The order in which a transport holds its passengers.
Boarding places a passenger at position 1, and passengers leave from that position, so the
order is last-in first-out. A failed exit puts the passenger back at position 1
(S-0507 claims 3 and 10).

**per-mille (‰).** An integer ratio where 1000 means a factor of 1.0.

**player house.** A house of a player, human or AI (see *AI house*). The *neutral house* and the
*special house* are not player houses. S-0409 defines it (Terms); most specs of the batch use it.

**rear direction.** The step direction nearest to a transport's body facing plus 32767
facing units, one facing unit short of a half turn. S-0507 claim 10 owns its use for exits
and specifies its rounding.

**reference point.** The point of an object that distance and direction measurements use: for a
building the centre of its foundation, for every other object its position. See S-0204 (claim 4).

**round-toward-zero mode.** The floating-point rounding mode under which the original rounds
floating-point results toward zero. S-0205 (claim 5) owns its scope and its single switch
`quirks.chop_rounding` (default: the original behaviour), and its Edge cases list the claims of
other specs whose results depend on the mode, among them claims of S-0003, S-0004, S-0101, S-0104,
S-0105, S-0107, S-0108 and S-0202. A spec whose results depend on the mode points to S-0205 and
defines no switch of its own.

**scenario random stream.** The specs' name for the original's pseudo-random stream whose draws
decide game state, as opposed to the *unsynchronised random stream*. A *draw* takes one raw value
from it. A *ranged draw* gives a number in an inclusive range and may take no raw value, one or
several (S-0202, claim 4; S-0009). Each topic spec states how many draws happen and in which
order so that our engine can reproduce the random choices of the behaviour it specifies. This is
not a promise of identical match outcomes where a departure or a non-default quirk changes a rule
or its draws. Network games are
left to a future spec (see *multiplayer game, network game*). S-0202
owns its algorithm, seeding and derived forms (claims 1 to 9) and the kinds of systems that draw
from it (claim 10); each topic spec states the draws it takes.

**session.** One run of the program, from start to exit.

**setup values.** The values that the lobby fixes for one game: game-wide ones, named "the …
setting" (among them the *credits setting*, *unit-count setting*, *bases setting*, *short-game
setting*, *superweapon setting*, *crate setting* and *build-off-ally setting*), and per-player ones.
S-0412 owns them (Terms, claim 1); S-0406, S-0407 and S-0414 use the settings.

**signed facing difference.** The value *δ*(*a*, *b*) = *b* − *a* of two facings, reduced to the
range −32768 … 32767. A positive value is a clockwise turn, and a half turn counts as −32768. See
S-0204 (claim 11, Formulas F5).

**single-player game.** A skirmish or a campaign mission, as opposed to a *network game*. See S-0201.

**size, size limit (transport).** An infantry or vehicle type's floating-point size, and a
transport type's floating-point largest accepted size of one passenger. The size limit's
absent-key default is 0. Vehicle transports test it; ordinary air loading does not
(S-0507 claims 1 and 2).

**skirmish.** A non-campaign game set up in the skirmish lobby with chosen houses, start positions
and a game mode, rather than played over a network. In the original the opponents are computer
players. Our engine also permits a skirmish with no person-played house; its end test is S-0414's
(claim 6), by the project lead's decision.

**step direction.** One of the eight directions from a cell to a neighbouring cell, numbered 0 to 7
clockwise from map north: 0 is the cell (*x*, *y* − 1), 1 is (*x* + 1, *y* − 1), 2 is (*x* + 1, *y*),
3 is (*x* + 1, *y* + 1), 4 is (*x*, *y* + 1), 5 is (*x* − 1, *y* + 1), 6 is (*x* − 1, *y*) and 7 is
(*x* − 1, *y* − 1). Even numbers are straight steps, odd numbers diagonal ones. A definition: the
numbering that S-0208 and S-0212 use.

**target, destination.** An actor's *target* is the object it currently attacks, captures or
enters, and its *destination* the cell or object it currently moves to. Either may be absent. See
S-0215 (Terms); orders set them (S-0213), and the movement specs keep further points of their own.

**team member.** An actor that belongs to a team of the original's computer-player script system.
That system is out of scope for glasnost, so specs note only where membership changes a rule of the
game. An *AI house* of our engine has no such teams, so a rule's exception for team members never
applies to its units. See S-0215 (Terms).

**tick.** One simulation step: the unit of game time. Timings of game state such as delays,
cooldowns, animation stages and build times are ordinarily counted in ticks, against the *tick
number*. The chrono flash also takes steps in frozen cycles, as claim G3's exception 2 describes.
A tick runs in every
*cycle* that is not frozen; how many run per second of real time at each game speed is S-0201's
(claims 2 and 5, Formulas F1; claim G2). Game speed leaves the length in ticks of every timing of
game state unchanged, with the two exceptions of claim G3: the speed-normalised delays and whatever
waits on them, and the number of ticks a chrono flash runs before its own freeze when a freeze falls
within it. In our engine game speed reaches game state only through the switch
`quirks.speed_normalised_delays`, which covers both (claim G3). What happens inside one tick, and in
which observable order, is S-0203's.

**tick number.** The number of ticks a game has completed: 0 while its first tick runs, and one
more at the end of every tick that does not end the game. Timings of game state counted in ticks are
measured against it. See S-0201 (Terms, claim 1).

**transport, vehicle transport, air transport.** A vehicle or aircraft whose type has a
capacity above 0. A vehicle transport is a vehicle that is a transport, an air transport an
aircraft that is one. Vehicle and air capacity rules differ (S-0507 claims 1 and 2).

**turn.** Two senses, told apart by their wording.
- An amount of rotation: a full turn is 65536 facing units (see *facing*), and a facing turns at a
  turn rate (S-0206). This is the sense of "turn" in every spec that does not say otherwise.
- An object's share of one tick, in which it moves, fires, counts down and starts or ends its own
  actions (S-0203, Terms), as in "takes its turn" or "its turn in tick *n*". The *turn pass* is the
  part of a tick in which objects take their turns (S-0203, Terms and claim 1). A spec that uses this sense
  says so in its Terms, as S-0203 and S-0215 do, and then writes a rotation with facing words
  ("turns by", "turn rate", "a quarter turn").

**unloading phases.** The four phases of a vehicle transport's *unload* processing:
*settling*, *turning*, *letting out* and *finished* (S-0507 claims 8 to 12).

**unsynchronised random stream.** The specs' name for the original's second pseudo-random stream,
distinct from the *scenario random stream*. It has the same algorithm and is seeded with the same
seed when a game is set up (S-0202, claims 1 to 3 and 8). During play it serves only choices that
never change game state, such as the idle comment chance of a unit owned by the local player (S-0107,
claim 14). S-0010 (claim 14) takes the draws of its 8 × 8 ground-variant pattern from it at map
load. S-0202 (claim 11) classifies its uses during play and on the setup screens, and settles how
our engine runs it: each machine runs it from the game's seed, it is not synchronised between the
machines of a network game, and none of its draws during play reaches game state. A spec that names a draw from this stream also says what the
draw affects.

**would-be passenger.** A unit in *enter* whose contact is a transport (S-0507 claim 5).

### rendering

**animated terrain object.** A terrain object (a type kind, such as a tree) of the group that S-0102
draws in the object pass instead of painting it in the cell pass. S-0102 says which terrain objects
are animated. Not the same thing as a *terrain animation*. See S-0101 and S-0102.

**battlefield view.** The part of the game screen that shows the map, without the sidebar and the
strip above it. See S-0101.

**chrono flash.** The full-screen chrono warp effect that a trigger action starts, one of the
*global effects*. It is not the chronosphere superweapon's jump (S-0104). S-0104 owns its lighting
and counts it as in force from the tick its action runs until it finishes. S-0201 (claim 11) owns
the rest of the effect, and a spec may use the term for the whole of it: its steps (one in every
cycle, frozen cycles included), the order block from its action on, and the freeze that starts in
the cycle after its step *D* + 44 and lasts to its last step, where *D* is the action's parameter. The two specs agree: its last step
falls in a frozen cycle, so it is in force in the ticks up to its freeze and in no tick after it.

**colour-adjustment table.** A table of 16 positions, each holding a tint colour with a red, a green
and a blue component. The rules data fills it, and the game combines a position's colour with an
object's pixels to tint it, for example a building under the invulnerability effect. A position that
holds zero (red, green and blue all 0) adds no tint. Rules layers change the table by entry position.
How it is reset and filled at each game start is owned by S-0006 (claims 2 and 12). See S-0006.

**construction state.** The state of a building while it plays its build-up sprite (see *sprite*):
forwards after it is placed (*build-up*), and backwards while it is sold (*selling*). Specs write
"construction state" for the state, and "build-up" or "selling" only for one direction of it, so a
rule that holds "outside the construction state" excludes both build-up and build-down. S-0408 and S-0415 call
the backwards direction the *build-down* (S-0408, Terms). S-0108 owns its timing. See S-0108.

**detail level.** The player's visual-detail option, kept in the settings file: 0 = low, 1 =
medium, 2 = high. Specs write "detail level", not "detail setting". See S-0104.

**disguise.** A unit's disguised state, disguise type and disguise house (which may be none).
The type may be an infantry, vehicle, overlay or terrain-object type. S-0516 claims 30 to 34
own taking and losing a disguise. *Appears disguised to H* is the game-state test, distinct
from the presentation test *drawn as its disguise for V* (S-0516 claims 35 and 37).
S-0107 claim 13 owns drawing infantry through a disguise.

**display path.** Everything between a drawn frame that the game has finished and the picture on the
monitor: the graphics driver and its settings, any compatibility layer, and whether showing a drawn
frame waits for the monitor's refresh. In a single-player game every cycle makes exactly one drawn
frame (S-0201, claim 6), so a display path that waits for the refresh can hold the tick rate back
(S-0201, scenario T1). The measurement setup allows a presentation-only *display compatibility
layer* (see *measurement setup*).

**drawn as its disguise for V.** The presentation test of whether viewing house *V* is shown
a unit's disguise image. It does not feed game state (S-0516 claim 37).

**drawn frame.** One redraw of the screen. Not the same thing as a tick, although in a single-player
game every cycle makes exactly one drawn frame, so a rule "per drawn frame" and a rule "per tick"
coincide there except during a freeze, which makes drawn frames without ticks (S-0201, claim 6).
Rendering specs say which of the two each rule follows. Not the same thing as an *image frame*
either. Specs should not write "frame" or "game frame" on its own.

**fake blink.** A countdown in ticks, set when a disguise-when-still vehicle fires and creates
a projectile, during which non-owner viewers are shown the real unit. It does not remove
the disguise (S-0516 claim 39).

**global effect.** One of the map-wide lighting effects of S-0104: the *nuke flash*, the *chrono
flash*, an *ion storm* or the *psychic dominator*. Each name, and the phases of each, means what
S-0104 defines, except that S-0201 owns the chrono flash's steps, order block and freeze (see
*chrono flash*). In particular, the chrono flash is not the chronosphere superweapon's jump (S-0104).
See S-0103 and S-0104.

**height above the surface, height above the terrain.** Two heights of an object, both in leptons,
that differ only for an object on a bridge deck. Its *height above the surface* is its world height
minus the height of the surface it is on or over: the bridge deck for an object on a bridge deck,
otherwise the terrain of its cell, so an object standing on a bridge deck has height 0 above the
surface (S-0104, S-0107). Its *height above the terrain* is its world height minus the world height
of the terrain of the cell it is over. Over a high bridge's cell that terrain is the ground under the
deck, never the deck (S-0102). Specs write one of these two, never "height above (the) ground" on
its own.

**height lift.** The number of pixels by which a point at world height *z* is drawn above the
projection of its ground point, written *L(z)*. See S-0101.

**house colour.** The colour that marks a house's objects. It replaces the palette's remap range
(indices 16 to 31) with a ramp built from the house's colour definition. See S-0103.

**image frame.** One picture in a sprite's shape file, numbered from 0. Not a drawn frame.

**object pass, cell pass.** The two parts of making a drawn frame in S-0102. The *cell pass* paints
the persistent terrain picture (terrain tiles, overlays, terrain objects that are not animated,
building bodies and the like), and the *object pass* paints the five render layers over it. Both are
rendering terms; the *turn pass* of S-0203 is a part of a tick. See S-0102.

**palette.** 256 colour entries. Once loaded, each entry is a colour with a red, a green and a blue
component, each an integer from 0 to 255; S-0103 says which of those values loading can produce.
Sprites and voxel models store palette indices, not colours. Each theater has its own unit, tile
and theater palettes; the anim palette is the same in every theater. See S-0103.

**render layer.** One of the five ordered lists in which objects are drawn: underground, surface,
ground, air and top. See S-0102.

**speed-normalised delay.** A delay that the original converts with the game-speed normalisation
N(*d*, *s*) of S-0107 (Formulas F3), so that it lasts about the same real time at every game speed
and its length in ticks depends on the game speed *s*. Exception 1 of claim G3 lists the delays it
covers and the specs that own them: S-0107, S-0108, S-0109 and S-0111. Their shared switch is
`quirks.speed_normalised_delays` (Edge cases and quirks). Game speed can change during a skirmish
through the options menu (S-0201, claim 8). In S-0107 (claim 5, at the start of a sequence),
S-0108 (claim 6, at the events of its table) and S-0109 (claim 4, when an animation's delay is
chosen) a delay is converted with the game speed in force when it is set, a later change of speed
leaves it as it is, and the delays set from then on use the new speed (S-0201, claim 8). S-0111
(claim 19) gives the repair wrench's period as a function of the game speed and owns how it follows
a change. The frozen cycles of a timed freeze also
depend on the game speed, but they are not converted this way and are not speed-normalised delays
(exception 2 of claim G3; S-0201). The switch keeps its name but covers that path too, the chrono
flash's steps during a timed freeze: it is the one switch through which game speed reaches game
state in our engine. What turning it off does on each path it covers is given in claim G3, in the
paragraph *Our engine*.
S-0701 claim 28 also uses this normalisation for the low-funds announcement's speech-delay
countdown. That countdown controls only what is heard (S-0701 claim 29): it is presentation
timing, not another game-state timing exception under G3.

**sprite.** A two-dimensional image stored in a shape file in the public "SHP" format: a numbered
sequence of image frames, each a rectangle of palette indices. Infantry, buildings, animations and
some vehicles are drawn as sprites. Aircraft bodies are drawn only as voxel models; an aircraft
type without voxel art draws no body (S-0102 claim 21; S-0105). A sprite that a type's art names
for one role is called by that role: for a building the *body sprite*, *build-up sprite*, *bib sprite*, *deploying
sprite*, *roof-deploying sprite*, *under-door sprite* and *under-roof-door sprite*; for infantry the
*body sprite*. Specs don't call them "sheets" or "images" (an *image name*, see *art database*, names
an art section, not a sprite), and a numbered picture inside one is an
*image frame*, never a bare "frame". See S-0007, S-0102, S-0105, S-0107 and S-0108.

**voxel model.** A volumetric model in the public "VXL" format, with a companion motion file in the
public "HVA" format of the same base name. See S-0105.

**world height.** The height of a point in leptons above height level 0, not above the local ground.
See S-0101.

**world pixel.** An integer pixel position (*q<sub>x</sub>*, *q<sub>y</sub>*) in an unbounded plane
that contains the whole map, *q<sub>x</sub>* growing to the right and *q<sub>y</sub>* downwards. The
battlefield view shows a window of it. See S-0101.

### movement

**aircraft.** A type kind and its objects: units that fly. Aircraft bodies are drawn only as voxel
models; an aircraft type without voxel art draws no body (S-0102 claim 21; S-0105). Infantry of a
jumpjet type and hovering vehicles are not aircraft: an object's kind is the kind of its type.
S-0501 claims 1 to 10 own ordinary flying-locomotor flight, take-off and landing.
Rocket missiles instead use S-0506 claims 14 to 21.

**arrival edge.** A house's map edge, numbered 0 north, 1 east, 2 south, 3 west, from
which its paradrop planes enter and by which retreating aircraft leave. S-0522 claims 4 and 5
own arrival and edge selection; S-0501 claim 34 owns retreat.

**bridge, high bridge, bridge deck.** A *bridge* is a crossing of water or a gap built from bridge
overlays. A *high bridge* is a bridge with a raised *bridge deck* that units can drive on and pass
under; a bridge without one is a *low bridge*. The deck lies 4 height levels above the terrain of
its cell (S-0102, claim 15; S-0104). Specs write "high bridge" and "bridge deck", not "raised bridge" or "raised bridge
deck". An object "on a bridge deck" stands or drives on the deck, as opposed to passing under it.

**cell layer.** A cell has a ground layer and, when it carries a high bridge, a bridge-deck layer:
these are its cell layers. An object on a bridge deck belongs to the deck layer, every other object
in the cell to the ground layer. It is neither a rules layer nor a render layer. See S-0209 (Terms).

**contact.** The one pairing that a mobile actor can hold with another object while the two carry
out an exchange: for example a harvester with the refinery it unloads at, a unit with the
transport or building it is entering, or a vehicle leaving the factory that built it. It lasts
until the exchange ends or one side breaks it off. A building can hold several contacts at once.
An actor "in contact with nothing" holds none. See S-0215 (Terms); the specs of the exchanges own
when contact starts and ends. A *tether link* is a firmer state that some exchanges add to a
contact.

**cruise height.** An aircraft type's flight-level field, in leptons, or the game-wide flight
level when the type's field is absent. S-0501 claim 2 owns climbing toward it; its Terms
distinguish cruise height from the height held during landing or destination guidance.
Not the *jumpjet cruise height*.

**deploy-to-land vehicle.** A vehicle whose type has both the simple-deployer and deploy-to-land
flags. It hovers when idle and lands to deploy (S-0502 claim 13); S-0517 owns its deploy cycle.

**drop attempt.** One attempt to release a carrying aircraft's first passenger beside it.
S-0522 claims 9 and 11 own the attempt and its use by aircraft shots.

**drop point.** The point beside a carrying aircraft at which a drop attempt is judged,
using a 128-lepton horizontal step to the selected side. S-0522 claim 9 owns the test and
specifies the point's direction and rounding in its formula.

**entry outcome.** The answer a moving unit gets for a neighbouring cell: one of eight values, in
order of increasing severity, *clear*, *cloaked blocker*, *moving blocker*, *closed friendly gate*,
*friendly obstacle*, *enemy obstacle*, *stationary friendly* and *blocked*. See S-0209 (Terms,
claim 4); S-0212 (claim 27) gives the rows it relies on for a vehicle mover.

**follower car, leader, car chain.** A *follower car* is a vehicle that the map links to another
vehicle, its *leader*, so that it follows it, as a carriage follows its engine. A leader that is
not itself a follower car forms a *car chain* together with the follower cars behind it. S-0212
(claim 26) gives how the cars are sent after their leader, and S-0207 (claim 10) how they copy its
speed fraction. Specs write "follower car", not "following car", and "car chain", not "train", for
the linked group (see *train*).

**hover height.** A hovering vehicle's world height above the terrain surface at its position,
less 416 leptons when it stands on a deck (S-0503 claim 8).

**hover slot.** A per-cell mark held by at most one hovering jumpjet. S-0502 claims 7 and 13
own claiming and releasing the slot.

**hover speed.** A hovering vehicle's double-precision fraction from 0 to 1 of its per-tick
speed that it travels, changed by acceleration and braking. It is distinct from the speed
fraction already included in that per-tick speed (S-0503 claims 3 and 4).

**hovering vehicle.** A vehicle with the hovering locomotor. S-0503 claims 1 to 12 own its
movement, lift and switch-off behaviour.

**in-the-air answer.** An aircraft's test for being in the air: the height-above-the-surface
test used by S-0307 (at least 208 leptons above the surface), except for two game-wide
rocket types that use their locomotor's answer. S-0501 claim 18 uses this test for pad search; its Terms distinguish it from that
spec's on-the-ground and in-flight tests. See also *airborne*.

**jumpjet cruise height.** A jumpjet type's imported height field: the height above the terrain
at which its jumpjets fly, in leptons. It is also the hover height that S-0101 uses. S-0102 gives its minimum, its
absent-key default and its role as the boundary between the air and top render layers. Specs write
"jumpjet cruise height", not "hover-height field". See S-0102.

**jumpjet state.** One of seven flight states: *grounded*, *ascending*, *hovering*, *cruising*,
*descending*, *crashing* or *crashed* (S-0502 Terms and claim 3).

**jumpjet type, jumpjet.** A jumpjet type is an infantry or vehicle type with the imported jumpjet
flag, which flies with a jump-pack style locomotor. A *jumpjet* is a unit of a jumpjet type that is
currently moving with that jumpjet locomotor. Specs say which of the two a rule needs. See S-0107.

**land multiplier.** The factor *M*(*ℓ*, *c*) for land type *ℓ* and locomotion class *c*, stored in
single precision. S-0207 uses it as a speed factor; a multiplier of exactly zero means that the class
cannot enter that land. See S-0211 (Terms, claims 2 and 10).

**land type.** One of twelve categories of ground that each cell has at any time: *clear*, *road*,
*water*, *rock*, *wall*, *tiberium* (resource ground), *beach*, *rough*, *ice*, *railroad*, *tunnel*
and *weeds*. See S-0211 (Terms, claims 5 to 8).

**landing reservation.** A descending jumpjet's occupancy mark in its landing cell, made before
touchdown (S-0502 claim 15).

**local point.** An integer point (*a*, *b*) in the local area's own frame: *a* counts cells
across the screen from its left side, *b* half-rows down from its top. S-0522 claim 4 owns
its conversion to a cell and specifies the conversion in its formula.

**locomotion class.** The column of the land-type table that a mobile type uses: *foot*, *track*,
*wheel*, *hover*, *winged*, *float*, *amphibious* or *float-on-beach*, or none. It is not the
*locomotor* (how an object moves) or the *movement zone* (where the pathfinder lets it go); a type
has all three, chosen by three separate fields. See S-0211 (Terms, claim 3).

**locomotor.** The movement method of a mobile type, chosen by an imported field: for example driving,
walking, hovering, flying or jumping. It decides how the object moves between cells and what
movement states it reports. The movement specs own its details: S-0207 the driving locomotor's
movement, S-0208 infantry walking, S-0504 claim 1 the ship locomotor, S-0501 claims 1 to 10
ordinary flying locomotion, S-0503 claims 1 to 12 hovering movement,
S-0506 claims 14 to 21 rocket flight, S-0206 turning under each
locomotor, and S-0106 (claim 5) which locomotors tilt a body on slopes. The other locomotors
are left to later movement specs.

**mission target.** The object or cell that a unit's current behaviour aims to enter, capture,
sabotage or otherwise reach, while the unit is in a behaviour that has one. S-0209 (Terms) defines
it and uses it in its entry outcomes (claim 4). Which behaviours have one belongs to the behaviour
specs and is open (S-0209, Open questions). The master of an enslaved unit is
not a mission target: S-0209 gives it a rule of its own. See S-0209.

**movement heading.** The facing a hovering vehicle keeps besides its body facing. It turns
towards the heading point, and the vehicle travels along it (S-0503 claim 6).

**movement zone.** A mobile type's imported category of where the pathfinder lets it go, such as on
foot, by wheel or track, by water or amphibiously. There are thirteen. The specs call them by the
lower-case names of S-0211 (claim 12), for example *normal*, *crusher*, *amphibious* and
*amphibious-destroyer*; these are our own names, which S-0107 uses too, and S-0211's Import mapping
gives how each is written in the data. S-0211 (claims 10 and 12) gives which cells each zone may cross, S-0212 builds
its connectivity from them, and S-0107 (claim 7) lets infantry of the zone *amphibious-destroyer*
use their water sequences. It is not the *locomotion class* or the *locomotor*.

**moving now.** Movement-specific, not interchangeable between locomotors. A jumpjet is moving
now in every jumpjet state except *grounded* and *hovering* (S-0502 claim 20). A hovering
vehicle is moving now when it has a destination or heading point and its throttle is not 0
(S-0503 Terms).

**neighbour step.** A jumpjet's move to a neighbouring cell chosen by one ranged draw over
0 to 7 from the scenario random stream when its desired hover slot is taken (S-0502 claim 21).

**occupancy record.** The record that each cell layer keeps of which standing positions are taken,
whether a vehicle, a building or another object is present, and the house of the last infantry unit
to mark a position there. See S-0209 (Terms, claim 1).

**pad, pad reservation.** A *pad* is a building whose type is one of an aircraft type's dock
types. A *pad reservation* is the aircraft's contact with that building. S-0501 claims 21
to 24 own choosing, accepting, reserving and docking at pads. See *refinery, dock types* for
the other senses of dock types.

**pad-bound.** An aircraft type that may rest only on a pad. S-0501 claim 18 owns its rest
rules, including crashing when no pad can be obtained.

**pad total.** The sum of the docking-place counts of a house's placed helipad buildings.
S-0501 claim 30 owns its accounting and the aircraft-capacity test that uses it.

**paradrop plane.** An aircraft of the paradrop plane type sent by a paradrop launch,
a visiting aircraft carrying its paratroopers as passengers (S-0522 claim 3).

**paratrooper.** A passenger of a paradrop plane, and the same unit after it is dropped until
it lands (S-0522 claim 9).

**pass counter.** A paradrop plane's count of passes it may still start without a successful
drop. It starts at 5 and a successful drop resets it to 5 (S-0522 claims 6 to 8).

**passability category.** A number from 0 to 7 that each cell carries, computed from its land type,
overlay and contents: *open*, *crushable*, *wall-like*, *beach*, *water*, *partial obstacle*,
*blocked* and *off-map*. See S-0211 (Terms, claim 11).

**payload list.** A pair of rules lists, infantry types and counts, whose entry *i* means
one plane carrying *n*ᵢ units of type *t*ᵢ (S-0522 claim 2).

**powered (hovering vehicle).** Not switched off, or still at a hover height above 0.
A switched-off hovering vehicle therefore remains powered until it settles. This is the
hover-specific test, not a building's power condition (S-0503 claim 10).

**service step.** One round added or one repair step made by a pad for one aircraft settled
on it. S-0501 claims 26 and 27 own servicing and repair.

**sinking descent.** The lowering of a sinking vessel by 5 leptons per tick until its height
above the terrain is strictly below −400 leptons and it leaves play. S-0504 claim 4 owns it.

**speed fraction.** A movement speed multiplier. For aircraft, a floating-point value from
0 to 1, with a current and a target value (S-0501 claim 5). Ground vehicles also have a
current and a target speed fraction; S-0207 claims 9 and 10 own their rules, and S-0504
claim 1 applies those rules to ships. S-0503 claim 4 owns a hovering vehicle's speed fraction,
which is separate from its *hover speed*. A spec uses the sense for its movement kind.

**spin (hovering vehicle).** The brief uncontrolled rotation initiated by switching off a
hovering vehicle. Re-enabling does not itself end it (S-0503 claim 10).

**submerged.** An underwater-type vehicle in the cloaking, cloaked or uncloaking state.
S-0504 claim 6 owns the naval vocabulary; S-0516 claim 41 owns submerging and surfacing
through the cloak transitions. An underwater type without the ability to cloak is never submerged.

**tether link, tethered.** A link that two objects in *contact* both hold during some exchanges.
An object that holds one is *tethered*. Contact alone does not imply a tether. The spec that owns
an exchange says whether and when the link starts and ends; the term itself does not promise
that every loading, unloading or docking manoeuvre creates one. See S-0208 (Terms).
Concrete factory examples are a ground vehicle leaving its factory, whose clear
report ends contact and tether (S-0417, claims 2 and 7), and an infantry factory or cloning
building giving a released infantry unit clearance to unload, tethering it when the contact
exchange succeeds until its next cell-arrival clear report (S-0417, claim 10).
A newly released aircraft that appears docked at its factory is tethered from its release tick
unless its type must return to a dock; that type refuses the release tether. Both cases establish
contact (S-0417, claim 11). This release example is not provisional. Landing tethering and the
ending of an aircraft tether when it leaves the ground remain provisional hypotheses, not
established rules of this glossary (S-0213, Open questions; S-0417, claim 11 and Open questions).
S-0417 owns the release-aircraft take-off hypothesis and its test (claim 11, scenario U12),
not a rule inherited from this glossary. Which other situations of play set the link
is not fully known (S-0208 and S-0213, Open questions). What a tether changes is owned by the
specs that use it, for example S-0213 (table T1) for orders.

**throttle.** A hovering vehicle's double-precision per-tick drive setting: 0, ½ or 1
(S-0503 claim 2). It is not its actual speed.

**train, train type.** A *train type* is a vehicle type with the imported train flag, and a
*train* is a vehicle of such a type (S-0210, S-0212, S-0214). The flag is a field of the type; the
link that makes a vehicle a *follower car* comes from the map, and a group of linked vehicles is a
*car chain*, not a train (see *follower car*).

**underground type.** A type whose movement zone is *subterranean* (see *movement zone*; S-0211,
claim 12). The property is the zone itself, so it is a property of the type in the surface state
as well as below the surface. S-0214 owns the term (Terms) and uses it in its click rules (claims 6
and 11).

**vehicle.** A type kind and its objects: wheeled, tracked, hovering and naval units, drawn as voxel
models or sprites (see S-0007, claim 4).

**visiting aircraft.** An aircraft marked as not meant to stay on the map. Placement sets
the mark unless its type is both selectable and landable and its primary weapon is not
a camera weapon; placement never clears an existing mark, and dispatch can set it earlier.
S-0501 claims 1 and 10 own placement and leaving the map; its Terms define the mark.

**wake.** An instance of the game-wide wake animation created over water.
S-0504 claims 2 and 4 own moving-ship and sinking-vessel wakes; S-0503 claim 9 owns
hovering-vehicle wakes.

### infantry

**deployed-firing rule.** A deployed infantry unit keeps an in-range target; otherwise it runs
a range search centred on its own position. If it then has no target it falls back to rest,
unless its current behaviour is *guard* or its type is a jumpjet type (S-0517 claim 16).

**infantry.** A type kind and its objects: soldiers and other units drawn as sprites and moving on
foot. See S-0107.

**infiltrated-vehicle-factory flag, infiltrated-infantry-factory flag.** Two yes/no states of a
house, initially clear. Each is set when that house's spy infiltrates another house's factory
of the corresponding kind and the factory effect is selected; once set, it stays set for the
rest of the game. They are read from the house owning the object being created or shown,
not the victim house (S-0510 claim 5).

**infiltration.** A spy's building entry on arrival, using up the spy even when no effect results,
including entry into its own house's building. Entry into another house's building applies at
most one effect chosen by the building type under the default single-effect rule
(S-0510 claims 1 to 3).

**prone state.** The lying-down state of an infantry unit. S-0107 (Terms) owns its sequences and
the fear level that leads to it; S-0315 owns its effects in combat (claim 3), and S-0302, S-0310
and S-0312 use it.

**scatter.** A unit's short move away from its spot, made to dodge a threat or to make room, set off
by a *scatter request*. For infantry, S-0208 gives which requests a unit accepts and where it moves
(claims 11 to 13), through the nearby-cell search of its claim 13 (with S-0212, claim 25). By
decision of the project lead it also owns infantry scatter from damage: which scatter requests a
hit sends (claim 15). S-0107 (claims 14 and 15) gives the idle and flee scatters, and for scatter from damage it points to
S-0208 (claims 11 and 15). S-0209
(claims 11 and 12) gives the cell-wide scatter requests that make room, and S-0215 (claim 19) the
scatter property of behaviours. S-0207 (claim 16) owns acceptance and destinations for every
vehicle, whatever its locomotor; S-0402 (claim 10) uses that rule for a refused refinery-arrival
report. Scatter for other unit kinds remains with their movement specs.

**scatter request.** A request to a unit to scatter, with an optional *threat point* (the point to
move away from) and two yes/no options: *forced* and *move anywhere*. Each option bypasses
only the gates that the receiver's owning spec names, not every refusal. S-0208 (claim 11)
owns infantry acceptance, including when movement makes a forced request count as not forced.
For vehicles, forced bypasses the scatter-property gate and the target-dependent acceptance
chance for a request with a threat point; move-anywhere bypasses the ordinary destination gate.
The other refusals remain (S-0207, claim 16). In particular, forced alone does
not bypass an existing destination, as S-0402 (claim 10) illustrates. S-0208 (Terms) lists the
specs that send each kind of request.

**steal fraction.** The game-wide share of the victim's available credits that a spy's money
theft takes (S-0510 claim 6).

**used-up engineer.** An engineer removed from play, not killed, at the end of a repair,
damaging hit, capture or futile entry into a non-arrival-friendly building of a type that
cannot be captured (S-0509 claim 10).

### combat

**aim point.** The fixed world point a rocket missile flies to, set when it first carries
out its move behaviour. S-0506 claim 14 owns its selection and fixing; later move orders
do not change it.

**aiming facing.** The facing of a firer that the firing check compares with the direction of its
target: its turret facing or its body facing. S-0310 owns which facing aims and the aiming
tolerance (claims 1 to 5); S-0306 and S-0313 use it.

**airborne.** Ambiguous on its own in a rule, because the combat specs use two different tests:
S-0307 gives a test by height above the surface, which S-0306 uses. S-0307 claim 26 qualifies
that height-test sense for the game-wide V3-rocket and dreadnought-missile aircraft types:
their locomotor's own in-the-air answer replaces the height test, and *floor-bound* is the
opposite of that answer.
S-0308 means the state in which the original counts an object as in the air, which S-0309 uses.
For aircraft, S-0501 claim 18 supplies that second test as the *in-the-air answer*. A spec that
uses "airborne" in a rule says in its Terms which test it means. In a measurement setup the word keeps its everyday sense:
the object is in the air.

**ammunition.** A firer's count of shots left. A type whose maximum ammunition says it uses none
never runs out. S-0301 owns how shots spend it and how vehicles reload (claims 10 and 11). S-0302
calls the same count *readiness* where damage lowers it (Terms, claim 10); S-0312 and S-0315 use
it.

**armed.** Ambiguous on its own in a rule, because the original asks different questions in
different rules: S-0209 (Terms) has its own test for walkers and for vehicles, and S-0214 (Terms)
calls an object armed when it has a weapon in the weapon slot it currently uses.
S-0309 uses S-0214's sense, and S-0312 (Terms) a third sense for one formula. A spec
that uses "armed" in a rule says in its Terms which test it means. In a measurement setup the word
keeps its everyday sense: the object has a weapon.

**armour class.** The category of protection a type belongs to, taken from the type's imported
armour field. A warhead scales its damage by its *versus value* for the target's armour class.
S-0305 owns the list of classes, their order and how a type's class is read (claims 1 to 5).
S-0302 (claim 6) applies the class to damage, and S-0304 gives what small versus values mean.

**attack run, firing pass.** An aircraft's *attack run* is one approach to a target ending
in one or more *firing passes*, each a processing in which the run fires. A run that fires
spends one round of ammunition. S-0501 claims 12 to 15 own the runs, passes and spending.

**automatic search, automatic target scan.** A search in which an object picks a target by itself,
without an order naming that target. S-0309 owns the searches, their kinds (*range search*, *area
search* and *map-wide search*), their candidates and their scoring (Terms, claims 1 to 17). Specs
write "automatic search". S-0304's *automatic target scan* means the same, and S-0215's
opportunity, area and normal searches are S-0309's range, area and map-wide searches (S-0309,
Terms).

**awaiting removal.** The state of a destroyed building kept in play after its death effects,
until its removal turn. S-0312 owns the term and the removal conditions (Terms, claim 21).

**being erased.** The state of an object with at least one temporal beam attached, from the
first lock until the last beam lets go or the object is removed (S-0514 claims 4 and 10).
An object in the leaving stage of a chrono move also counts as being erased, without a beam
(S-0513 claim 39; S-0514 Terms). S-0215's *warped out* means the same state. See also
S-0109 (Terms) and S-0214 (Terms).

**berserk.** The state of a unit that a berserk-causing warhead has put out of its owner's
control. S-0301 (claim 8) and S-0302 (claim 10) give what it changes in firing and in damage, and
S-0309 uses it. S-0610 claims 4 and 5 own entry into the state, its countdown and expiry.

**bomb detector.** An object on the map whose type has a positive bomb-sight range in whole
cells. It lets its owner see planted bombs within that range (S-0519 claim 10).

**bomb-planting warhead, disarm warhead.** Warheads whose projectile detonations respectively
plant a bomb on the target instead of doing area damage, or remove its planted bomb without
damage. S-0519 claims 2, 3 and 7 own planting, firing legality and disarming.

**bomb visible.** The presentation and input state of a planted bomb that the local player
planted, or can see through an own-house bomb detector in range. S-0519 claim 10 owns the
visibility updates and range test; visibility does not change bomb timing or damage.

**burst counter.** A per-firer count of the shots made in the current burst, shared by all weapons
of the firer. S-0301 owns it (Terms, claim 4); S-0313 (claim 2) uses it.

**cell radiation.** The radiation held by a cell: a double-precision floating-point running sum
of field contributions, less the amounts taken away by decay and topping up, never below 0.
It can include contributions from fields that have ended (S-0525 claims 3 and 6).

**charge countdown.** A building's delayed-kill countdown when an explosive charge started it,
with the charge planter recorded. S-0519 claim 14 owns its start; S-0302 claim 16 owns expiry.

**charger list, charger.** A building's list of infantry whose electric-charge shots have
reached it, each listed at most once. Its length is S-0405's charger count
*n*<sub>c</sub>. S-0518 claims 11 and 12 own registration and removal.

**charging mark.** A per-unit flag set when a unit starts charging a building by itself. It
clears whenever the unit is given a target again, including the same target
(S-0518 claims 14 and 15).

**close-range property.** An imported boolean type flag, not a description of a weapon's range.
S-0307 owns the term and its effects on approach-position searches (Terms, claim 22).
S-0215 uses the same flag for infantry in its close-range delay shortening (Formulas F3),
whose separate weapon-range condition does not require the flag.

**current weapon.** The weapon in the slot of an object's weapon number when its type is a
multi-weapon type, and the weapon in its slot 0 otherwise. See S-0309 (Terms), with S-0308 for
the weapon number; S-0312 uses it.

**curtain protection, temporarily invulnerable.** An object is *temporarily invulnerable* while an
iron curtain or force shield effect protects it (S-0302, Terms). *Curtain protection* is that
effect of the iron-curtain kind, as opposed to the force-shield kind (S-0303, Terms); S-0312 uses
it. S-0302 and S-0303 give what the state changes in damage; how the effects start and end belongs
to the superweapon specs.

**damage that ignores defences.** A hit delivered with a marker that tells the target to
disregard its defences. S-0302 owns which hits carry it (claim 1) and what it skips (claim 9).
S-0107 and S-0312 use it, S-0312 as the *ignore-defences marker*.

**death weapon.** The weapon that an exploding object's death detonates. S-0312 owns it (Terms,
claim 6); S-0303 gives the detonation.

**debris.** Animations thrown by a death or a detonation. S-0312 owns them (Terms, claim 4,
Formulas F1); S-0303 (claim 22) throws them from a warhead.

**deploy-fire vehicle.** A vehicle whose type has the deploy-fire flag but not the in-place
deploy flag. While *unload* is current it attempts to fire its slot-0 weapon at its own cell,
subject to the firing check (S-0517 claim 2).

**deploy-to-fire cell test.** A cell passes when its slope shape is flat, it holds no building
and its land type has the buildable flag (S-0517 claim 14).

**detonation, detonation point.** One application of area damage at a point, the *detonation
point*, with a damage amount, a warhead, and optionally the object and the house that caused it.
S-0303 owns what a detonation does (Terms, claims 1 to 23). Projectiles (S-0306), damaging
animations (S-0109), deaths (S-0312) and other specs produce detonations, and each says what it
passes. S-0302 turns each resulting *hit* into a health change.

**drone parasite, grappling parasite.** A parasite whose type is both naval and organic is
a *grappling parasite*; every other parasite is a *drone parasite*. S-0614 claims 6 and 10
own their deliveries: drones use their weapon's rate of fire, grapplers a fixed strike
cycle. S-0614 claim 11 owns a grappler's cull.

**elevation level.** Of a cell: its height level, plus 4 height levels when it has the
*elevation-deck mark*, whether an object stands on the deck or under it. S-0307 owns it
(Terms, claim 6, Formulas F2); S-0306 (claim 22) uses it.

**elevation-deck mark.** The cell mark used for the elevation bonus of S-0307 (Terms, claim 6).
It is distinct from S-0212's *deck-entry mark*, which controls movement onto a deck, and from
S-0211's high-bridge-cell test (claim 15). S-0307 owns its verified marking scope (Terms).

**elite weapon, elite slot.** A type's optional second weapon for one weapon slot, which an object
of elite rank uses instead; the slot that holds it is that slot's *elite slot*. S-0311 (claim 6)
owns when it is used and S-0308 (Terms, claims 1 and 2) the slots; S-0313 gives its firing positions.

**erasure amount.** The integer remaining amount for a target with temporal beams attached.
It starts at 10 times the target type's strength and falls each tick by the sum of the attackers'
raw weapon damage, counting at most the leading attacker and the 51 most recent joiners.
Erasure completes at 0 or less (S-0514 claims 6 and 7).

**expected damage.** The damage that the original estimates, when an object fires at another, for
target choice. S-0302 owns how it is computed (claim 18); S-0309 uses it for its *expected
remaining health*.

**experience.** A number per object, held in single precision, from which its *veterancy rank*
follows. S-0311 owns how it is gained, kept and lost (Terms, claims 1, 2 and 7 to 13); S-0302 and
S-0312 use it.

**field level.** A radiation field's integer current level, ⌊*L*·*T*/*N*⌋, where *L* is its
level set at creation or topping up, *T* its time left and *N* its duration in ticks. The current
level is 0 when *T* < 1. S-0525 claims 4 and 7 own the scaling and rounding: integer evaluation
for the deployed-infantry test, floating-point evaluation truncated toward zero for topping up.
Not the cell radiation.

**fire point.** The world position from which a shot is fired, where its projectile normally
starts. S-0313 owns it and the muzzle positions it is built from (Terms, claim 1). S-0306 places
the projectile there (claim 8), and launches a projectile whose turn rate is 0 and that does not
drop from it towards the impact point (claim 5). A steering projectile also starts at the fire
point; only a dropping projectile starts from the firer's own position instead (claim 5). S-0307 (claim 8) does not use it: the range test measures from the source
point that claim gives.

**firer, shot, void shot.** A *firer* is an object that fires a weapon: a vehicle, infantry unit,
aircraft or building. A *shot* is one firing of one weapon by a firer: one projectile, beam or
effect launched. An attempt that the firing check refuses makes no shot, and neither does a *void
shot*, an attempt that passed the check but whose projectile could not be put on the map. S-0301
owns the terms (Terms, claim 15); most combat specs use them.

**firing check, answer.** The test that every attempt to fire a weapon at a target makes. Its
*answer* is *can fire* or a reason why not: *rearming*, *out of ammunition*, *not facing*, *still
turning*, *moving*, *cloaked*, *out of range*, *illegal*, *impossible*, *busy* or *must deploy*.
S-0301 owns the check, the answer names and their precedence (claim 9). S-0306, S-0307, S-0308,
S-0309 and S-0310 give the answers of their own topics, and S-0206 and S-0215 use the term.

**garrisoned building.** A building whose type lets infantry occupy it and fire from it, while at
least one occupant is inside. S-0301 (Terms) defines it, and S-0302 (Terms) gives the same
definition; S-0302, S-0307, S-0308, S-0311 and S-0313 give what it changes in their topics.
S-0508 claims 1 to 17 own occupation, eviction and loss of occupants.

**healer, healer for acquisition.** An object whose *mean weapon damage* is below 0. S-0308 owns
the mean weapon damage and the test (claim 15, where the object is called a *healer for
acquisition*); the two names mean the same test. S-0309 uses it for automatic searches (claim 6),
and S-0314 owns healing in combat. S-0214 does not use this test: its healer rows of the cursor and
order rules use its own test, the *healing actor* (S-0214, Terms), which looks at the weapon the
object would use by default. S-0308 (claim 15) notes that the two tests can differ. S-0314 (Terms)
uses the word *healer* in S-0214's sense and says so.

**health, strength, health ratio.** An object's *health* is its current hit points, an integer from
0 to its *strength*, the maximum its type gives (S-0302, Terms). Its *health ratio* is health
divided by strength (S-0214, Terms). S-0302 owns how hits change health; S-0303, S-0309, S-0312
and S-0314 use the terms.

**hit, healing hit.** A *hit* is one delivery of an amount of damage, with a warhead, to one
object. S-0302 owns hits in general and what one does (claims 1 to 19). S-0303 uses the word in a
narrower sense, the hit of one detonation on one object at one distance (Terms, claims 8 and 11).
A *healing hit* is a hit whose delivered amount is negative (S-0302, claim 6); S-0309, S-0314 and
S-0315 use it.

**hold, held.** A mobile unit's hold is a countdown in ticks; it is *held* while that timer
runs. A parasite starts it on its host and is itself held after leaving a living host.
S-0614 claims 7, 8 and 13 own renewal, effects and post-exit hold, including exceptions.
Being held stops most locomotors and makes the unit not movable for orders; it is distinct
from the electromagnetic-pulse disable called *paralysed* by S-0214.

**in-place deploying vehicle.** A vehicle whose type deploys in place, changing between mobile
and deployed without becoming another object (S-0517 Terms and claim 3). Not a vehicle that
deploys into a building under S-0415.

**known attacker.** Damage has a known attacker when it comes with the object that caused it.
See S-0107 (Terms); S-0208 (claim 15) uses it for the scatter requests that damage sends. S-0302
(Terms, claim 1) calls that object the *attacker*, adds the *attacking house* that a hit credits,
and says which sources give which.

**landing-first type.** An in-place deploying vehicle type that must touch down before it can
deploy (S-0517 claim 5).

**latching weapon.** A weapon whose warhead has the parasite flag and whose launch puts
the firer inside its own projectile. S-0614 claim 1 owns this launch and the attempt to
latch when the projectile detonates.

**launch pass.** One run of a spawn controller: the first comes 20 ticks after its creation,
then one every 10 ticks. S-0506 claim 2 owns this cadence and its exceptions.

**launching count.** The number of a spawner's slots on the rail plus its preparing slots
whose object is of a missile type (with the missile-spawn flag) and on the map.
The spawner is *launching* while this
count is above 0. S-0506 claim 10 owns the resulting lockouts; its Terms define the count.

**leading attacker.** Of the attackers whose temporal beams are attached to one target, the one
that carries the erasure amount. When it lets go, the most recent remaining joiner takes over
without losing progress (S-0514 claim 9).

**overpowered.** The state of an overpowerable building with at least three registered
chargers, or at least one while switched on with its owner at full power. It is decided in
the building's turn, not whenever the charger count changes (S-0518 claim 13).

**parasite, victim, host.** A *parasite* is a unit that attaches itself to another unit,
its *victim*, and damages it while attached. The victim is its *host* while it is attached.
S-0614 claims 1 to 5 own latching, claims 6 to 11 deliveries and hold, claims 12 to 14
leaving a living host, and claim 15 losing the host. S-0315 claims 6 and 7 own suppression
by damage and healing. S-0209 and S-0214 use these terms for entry and order targeting.

**planted bomb, planter's house.** A bomb attached to a carrier object by a bomb-planting
infantry unit. The planter's house is that infantry unit's owner when it planted the bomb.
The bomb keeps that house even after the planter changes owner or is removed
(S-0519 claims 2 and 8). S-0213 claim 11 owns the detonate order.

**prism master, supporter.** The prism-type building making an attack is the master; a
prism-type building of the same house that it recruits to beam to it is a supporter
(S-0518 claims 1 to 3).

**projectile.** The object that carries a shot from where it is launched (normally its fire point,
S-0306 claim 5) to where it detonates. S-0203
(Terms) names it, S-0306 owns its launch, flight and detonation, and most combat specs use it.

**radiation field.** A patch of irradiated ground with a centre cell, a spread in whole cells,
an integer level, a duration and time left in ticks, and a light source. At most one field has
a given centre cell; fields with different centres can overlap (S-0525 Terms and claim 2).
See *field level* and *cell radiation* for the distinct level concepts.

**range test, weapon range.** The *range test* is the yes/no question whether a target is within
reach of a weapon of a firer; the firing check's *out of range* is its "no". A weapon's *weapon
range* is its range field in leptons. S-0307 owns both (Terms, claims 1 to 12); S-0306 and S-0309
use them.

**rearm countdown.** A per-firer countdown in ticks, set by every shot the firer makes. While it
runs, the firer may fire none of its weapons; one countdown serves all of them. S-0301 owns it
(Terms, claims 1 and 2); S-0215's *rearm time remaining* is the number of ticks it still has to
run. S-0308 and S-0310 use it.

**release, flight start, detachment (spawned objects).** Three different events, not
synonyms. *Release* sends preparing objects against the controller target; a released
missile is enrolled for repeated attack orders (S-0506 claims 8 and 12). A rocket's
*flight start* is its first carrying-out of move behaviour after placement, fixing its
aim point independently of release (S-0506 claims 8 and 14). *Detachment* ends its
membership in a spawn slot; for an ordinary released rocket this happens when rail time
ends, and the slot starts regeneration (S-0506 claim 9). A released rocket can therefore
still occupy an on-the-rail slot. Release in production has its own meaning (S-0404
claim 13), not this one.

**retaliation.** An actor's automatic attack on the object that has just damaged it, as opposed
to an ordered attack or a target picked in an automatic search. S-0309 owns when an object
retaliates (claim 18), S-0304 (claim 3) its versus-value condition and S-0215 (claims 18 and 19)
what retaliation does; S-0308 uses it.

**rocket missile.** An aircraft-kind object of one of the three rocket types flying under
rocket-flight movement to a fixed aim point, not an ordinary projectile. S-0506 claims
14 to 21 own its flight, detonation and interception.

**run lock.** A mark held during the later passes of an aircraft's strafing run, a fighter's
firing pass and a spy plane's overfly. It makes the aircraft not ready to change, subject
to the spy-plane approach exception of S-0501 claim 31.

**silent removal.** For a parasite, immediate removal without death effects, wreck,
survivor, experience or death sound. S-0614 claims 4, 13 and 15 use this outcome; its
Terms take the definition from S-0315 claim 7. It is removal, not a killing hit.

**spawn controller, spawner.** A *spawn controller* belongs to an object, the *spawner*,
whose type names a spawned type and a positive spawn count. It holds the slots and
launches, regenerates and reloads their objects. S-0506 claims 1 to 9 own this shared
controller for missiles and carrier aircraft; S-0501 claim 35 owns carrier-aircraft flight.

**spawn slot, slot object, slot state.** One place in a spawn controller, holding at most
one *slot object* of the spawned type, in one of these *slot states*: docked, on the rail,
preparing, attacking, returning, reloading or empty. S-0506 claim 4 owns the states and
transitions; S-0501 claim 35 supplies the non-missile flight rules.

**spawned type, spawn count.** The aircraft type a spawner's type names as the type it
spawns, and its number of spawn slots. S-0506 claim 1 owns creation and initial fill.

**spread, spread radius.** A warhead's *spread* is its spread field, a decimal number of cells; the
*spread radius* is the distance in leptons, derived from it, within which a detonation hits
objects. S-0303 owns both (Terms, claim 3, Formulas F1); S-0302 and S-0312 use them.

**strafing weapon.** A weapon whose projectile has a signed turn rate at most 1 and is not
invisible. S-0501 claim 11 uses this test to identify a strafer; claims 13 and 15 own its
run and ammunition spending.

**suicide weapon.** A weapon whose shot creates no projectile but instead hits its own firer
for the firer type's full strength, not its current health (S-0519 claim 11).

**support beam.** A drawing-only beam from a supporter to its master's fire point as recorded
at recruitment. It has no projectile and does no damage; the supporter then rearms
(S-0518 claim 6).

**support count.** A per-building whole number counting recruitments for the master's next
shot, not necessarily distinct supporters. It scales that shot's area-damage input
(S-0518 claims 4, 8, 9 and 10).

**versus value.** The factor, a double-precision number, that a warhead gives one armour class,
read from the warhead's versus list (S-0003, claim 13, which with Import mapping row C5 gives the
order of the items). S-0305 names the class at each list position (Terms, claim 1) and says where
the class is used (claim 6); the two specs agree. S-0302 (claim 6) multiplies damage by it, and S-0304 owns
what zero and small values mean. "The versus value of a weapon against an object" is that of the
weapon's warhead for the object's armour class (S-0304, Terms).

**veterancy ability.** One of the capabilities that a type may grant at the veteran rank, at the
elite rank or at both. S-0311 owns the list and which rank grants each (Terms, claims 3
to 5); S-0107, S-0214, S-0215 and several combat specs use it.

**veterancy rank.** *Rookie*, *veteran* or *elite*, derived from an object's *experience*.
*Veteran* and *elite* alone name the two raised ranks. S-0311 owns it (Terms, claim 1). S-0214
(Terms) uses the same three words, and S-0301, S-0308, S-0309 and S-0312 use it.

**warhead.** The type that decides what a hit does: its damage scaling per armour class, its area
spread and its side effects. A weapon's warhead is one of the weapon's fields. S-0302 owns what a
warhead does in damage resolution, S-0303 its area damage and detonation effects, and S-0304 and
S-0305 its versus values.

**weapon.** The type that describes one way of attacking: damage, range, delay between shots,
projectile and warhead are all fields of it. S-0301 owns its firing cycle, S-0307 its range test,
S-0308 which weapon an object uses, S-0313 where its shots start and S-0306 the flight of its
projectile.

**weapon slot, weapon it would use.** A type holds numbered *weapon slots*, each with a weapon or
nothing; slot 0 is the *primary* and slot 1 the *secondary* slot (S-0308, Terms). The *weapon it
would use* against a target is the weapon that the weapon choice of S-0308 picks, and the combat
rules test that weapon (S-0308, claim 14; S-0304, Terms). S-0309, S-0310, S-0311, S-0313 and
S-0314 use the terms.

### structures

**absorbing building.** A building whose type takes in infantry, vehicles or both as passengers
that do not come out on their own (S-0312, Terms, *absorbed passengers*). S-0405 names it
(Terms), and S-0408 (claim 8) uses it.

**adjacency.** A building type's adjacency field, in cells, and the placement condition that
S-0406 builds on it. S-0406 owns both (Terms, claim 4); S-0415 uses the term.

**arrival-friendly building.** For an entering engineer, a building of its own house, of a house
its house counts as allied, or a garrisonable building of a passive neutral house. This arrival
test is distinct from the cursor's engineer-friendly test (S-0509 claim 3).

**blow-up of a cell.** The effects on occupants and animations when a high-bridge section or
pier is lost: ground-layer demolition hits, deck drops and falling, and explosion animations.
Protection from the hits is still tested (S-0511 claim 9).

**bridge repair hut.** A building type that rebuilds the bridge linked to it when an engineer
enters it (S-0511 claims 17 to 20). S-0511 claim 16 owns hut demolition.

**building.** A type kind and its objects: a stationary object that occupies a foundation of cells.
Also called a structure.

**building entry.** An infantry unit in *capture*, *area guard* or *patrol* finishing a walking
step in a cell holding the building that is its destination or target. Vehicle-like destinations
and garrison attempts have the separate outcomes specified by S-0509 claim 1.

**building ownership change.** A change of a building's owning house, whatever its cause.
Giving it to the same house has no ownership-change effect (S-0509 claims 12 to 16).

**bunkered, tank bunker.** A *tank bunker* is a building that a vehicle can drive into for
protection, and a vehicle inside one is *bunkered* (S-0206, Terms). S-0301, S-0302, S-0307 and
S-0309 give what being bunkered changes in their topics; entering and leaving are left to the
structures specs.

**cell test.** The test that a single cell passes or fails for a building type that is being
placed. S-0406 owns it (Terms, claim 3); S-0415 (claim 3) uses it for deploying.

**civilian garrison building.** A garrisonable building whose type's tech level is exactly −1.
S-0508 claims 7 to 9 own its occupant-dependent ownership and forced evacuation.

**civilian house.** The first house, in house order, whose country belongs to the civilian side
(S-0508 claim 8).

**construction yard.** Ambiguous on its own. S-0213, S-0214 and S-0415 (Terms) mean a building
whose type has the construction-yard flag; S-0215 (Terms) means a building whose type is in a list
of the original's computer player, which can be a different set of types. S-0522 claim 5 keeps
two senses apart: a *building factory*, whose type produces buildings, and a *yard-flag
building*, whose type has the construction-yard flag. A spec that uses the word in a rule
says in its Terms which sense it means.

**destroy attempt.** One application of the bridge destruction rules at a cell. It succeeds
when it removes a high-bridge section, collapses a pier or destroys a low strip. A failed
attempt can still change the bridge (S-0511 claims 1 and 2).

**eviction.** Putting all occupants of a building out at once, with hunting and inside-fallback
options. Exit failure can instead remove them or leave them inside under the chosen quirk
settings (S-0508 claims 10 to 13).

**exit apron cell.** The cell where a ground vehicle factory's exit track ends. It need not
belong to the factory's foundation. S-0417 owns the term and its position (Terms, claim 3).
It is distinct from S-0404's *exit cell*, used for production release.

**exit ring.** The cells bordering a building's foundation rectangle, outside it, including
corners. Which cells an eviction tries and their priority belong to S-0508 claim 10.

**fall rate.** A falling object's vertical speed in leptons per update, decreased each update
down to an imported floor (S-0511 claim 10).

**foundation.** The cells a building occupies, within a bounding rectangle of W × H cells, W along
the map x axis and H along y (S-0108, Import mapping, gives the sizes). The cells are normally the
full rectangle, but not always: in its building-fire draws S-0302 (claim 13) gives the zero-size
foundation word no cells, and the 8-cell 3 × 3 foundation word only 8 of the 9 cells of its
rectangle. S-0108's vocabulary has two 3 × 3 words (Import mapping, foundation vocabulary); in
those draws the other one, the plain 3 × 3 word, has all 9 cells. Which
cells a building occupies for placement, its *foundation cells* within its *foundation rectangle*,
is S-0406's (Terms, claim 2); S-0408 and S-0415 use those terms. Its top-left cell is the foundation cell with the smallest x and y. "Top-left" refers to the map
axes: on the screen that cell is the topmost cell of the footprint, not the leftmost. The
foundation's *top corner* is the top corner of its top-left cell, the topmost point of the footprint
on the screen.

**garrisonable type.** A building type whose occupiable flag is set (S-0508 claim 1).

**high-bridge section.** One high-bridge overlay and the four cells it marks. Its overlay data
value distinguishes intact, chipped on one side, damaged and open on one side states. A lost
section has no overlay (S-0511 claims 3 to 5).

**hut demolition.** Destruction of a whole bridge started at its repair hut by a detonating
timed bomb or an expiring delayed-kill countdown (S-0511 claim 16).

**inside fallback cell.** The foundation rectangle's cell with the largest x and y, used when
an eviction allows inside fallback and no exit-ring cell qualifies (S-0508 claim 12).

**lost-to-capture mark.** A house mark set when an engineer captures one of its buildings,
never cleared during the game, and read by trigger event code 3 (S-0509 claim 11).

**low strip.** The three cells across a low bridge at one step along its line. Changes to the
strip's class affect all three cells together (S-0511 claims 13 to 15).

**manned.** A building's permanent state from its first building ownership change onward.
A newly placed building is unmanned. This is the state read by the engineer row of S-0405
claim 8, not an infantry unit remaining inside (S-0515 claim 2).

**multi-engineer rule.** A setting under which an engineer damages a healthy enemy building
instead of capturing it; always off in a skirmish (S-0509 claim 6).

**occupant.** An infantry unit inside a garrisonable building, off the map and out of play,
keeping its own owner (S-0508 claim 5).

**occupant capacity.** A garrisonable type's maximum number of occupants (S-0508 claim 1).

**occupier type, assaulter type.** Infantry types marked as able to garrison and as able to
clear an enemy garrison, respectively. A type with both marks is an occupier only
(S-0508 claims 1 and 14).

**pier.** A bridge tile standing between two spans, in state 0 (intact) to 4 (collapsed), with
a reference cell and a three-cell column across the bridge (S-0511 claim 6).

**smudge footprint.** The rectangle of cells a smudge type covers, starting at its placement
cell (the *anchor cell*) and extending towards increasing map *x* and *y*. Its width counts cells
along *x* and its height counts cells along *y*. S-0312 owns the term and the destruction-smudge
fit test (Terms, claim 26).

**span check.** After a successful direct high-bridge destroy attempt, the check that removes
runs of sections touching neither end of the affected span, subject to the span bounds in
S-0511 claim 7.

**tech building.** A building whose type has the needs-engineer flag. The stock examples are
neutral service buildings: oil derrick, hospitals, machine shop, airport, outpost, secret lab
and civilian power plant. Their services are owned by the specs indexed in S-0515 claim 1.

**upgrade.** A building type that is added to an existing building instead of standing on its own
cells, and a building of such a type. S-0405 names it (Terms), and S-0406 owns how it is placed
(Terms, *upgrade type*, claim 12); S-0407 and S-0408 (claim 5) use it.

### economy

**available credits.** What a house can spend at a given moment: its *credit balance* plus the
credit value of the resources held in the house's own storage. The house's own storage is not a
building's storage (S-0408, claim 10) and not the separate store that a weeder fills (S-0402
leaves weeders to a later spec). S-0404 owns the term, the sum and the stored part (Terms, claim
4); the balance part is as in *credit balance*. No other spec owns a house's storage. In a skirmish nothing puts resources into it: harvests are paid straight into the
balance (S-0402, claim 12), and so are a sold building's stored resources (S-0408, claim 10).
So the stored part is 0, and available credits equal the credit balance (S-0404, claim 4).
S-0416 uses the term.

**bale.** The unit in which resource is held in a *resource cell*, harvested and carried. S-0401
owns how many bales a resource cell holds and what each is worth (Terms, claim 2); S-0402 counts a
harvester's *load* in bales.

**being drained.** The state of a building to which an enemy unit with a draining weapon is
attached. S-0405 owns the term (Terms) and what it changes in power (claim 5); S-0408 (claim 3)
uses it. How the attaching starts and ends belongs to the special-abilities specs.

**collector.** The object that picks up a crate by committing to a ground step into its cell,
by landing in it as a jumpjet, or after landing in it through its own teleporting locomotor
(S-0520 claim 6). Superweapon chronoshift arrival is not a pickup.

**counted building.** Ambiguous on its own. S-0405 (Terms, claim 3) counts the buildings whose power
a house sums, and S-0414 (Terms, claim 4) the buildings that keep a house alive in its defeat
tests; the two sets differ. A spec that uses the term says whose sense it means.

**crate.** A cell holding an overlay with the crate flag. Its overlay data value is its
contents: 0 … 18 name a fixed power-up; any other value means random contents
(S-0520 claim 8).

**crate area.** The objects whose table-root distance from the crate point (the cell centre at
terrain surface height, without a bridge-deck lift) is strictly less than the crate radius. A building is measured to its reference point. S-0520 claims 15 and 19
own the affected sets and distance rule; the relevant effect decides whether off-map objects
qualify.

**crate ownership counts.** Counts of objects in existence, including objects off the map and
in production, with the exclusions and kind assignments of S-0520 claims 9, 10 and 13.
They are not the on-map vehicle, infantry and aircraft counts used for defeat by
S-0414 claims 3 and 4; in particular, buildings that count as vehicles enter the crate vehicle
count, not defeat's on-map vehicle count.

**crate places.** The scenario's 256 crate places, each holding at most one crate cell and its
lifetime timer (S-0520 claim 1).

**credit balance.** A house's own stock of credits, not counting resources held in storage (see
*available credits*). S-0412 (claim 2) gives its starting value in a skirmish, S-0402 (claim 12)
what harvesting adds, S-0404 (claims 4 and 6) what production takes and gives back, and S-0408
(claims 1 and 10) what a sale adds.

**credits.** The unit of money: costs, refunds, harvested resources and a house's balance are given
in credits. Each economy spec states the number domain and rounding of the amounts it computes:
S-0402 (claim 12) for income, S-0404 (claims 2, 4 and 6) for prices, payments and cancel refunds,
and S-0408 (claim 1) for sale refunds. See also *credit balance* and *available credits*.

**harvester, load, capacity.** A *harvester* is a vehicle whose type has the harvester flag. Its
*load* is the *bales* it carries, kept per tiberium, and its *capacity* the number of bales it can
carry, its type's storage field. S-0402 owns the terms and the harvest cycle (Terms, claims 1 to
17), and names the *slave miner* and the *chrono miner* there; S-0401, S-0404 and S-0415 use
*harvester*.

**in operation.** Whether a building currently works as a powered building. It is worked out each
time it is asked, and it is not the state of the power switch, for which see *switched on,
switched off*. S-0405 owns it
(Terms, claims 8 and 9); S-0408 uses it.

**low power, power fraction.** Two derived power states of a house: whether it has *low power*,
and its *power fraction*, a number from 0 to 1, which compare its *power output* with its *power
drain*. S-0405 owns both (Terms, claim 4, Formulas F3); S-0403 (claim 5) uses the fraction, and
S-0409 and S-0410 use low power.

**power-up.** One of the nineteen crate effects, numbered 0 … 18, each with an imported
weight, animation, water flag and argument (S-0520 claim 8).

**power output, power drain.** What a building makes and what it needs, in power units (integers),
and a house's sums of them. S-0405 owns both (Terms, claims 1 to 7); S-0403, S-0404, S-0408 and
S-0410 use them.

**refinery, dock types.** Ambiguous on its own. In the harvest cycle a *refinery* is any building
of one of a harvester's *dock types*, the building types that its type lists as places to unload
(S-0402, Terms). S-0407's refinery *generic group* (Terms) is another set of building types,
S-0414's *refinery-list building* (Terms) is one building type taken from a list of its own, and an
aircraft type's dock types are the buildings it
docks at (S-0404, claim 2; S-0408, Terms). A spec that uses "refinery" or "dock types" in a rule
says in its Terms which it means.

**refund.** Ambiguous on its own. S-0408 (Terms, claim 1) gives the refund for an object that a
house sells, S-0404 (claim 6) the refund for a cancelled item, and the two are computed
differently. S-0415 (Terms, *sale refund*) and S-0416 use them. Where the context does not tell,
specs write "sale refund" or "cancel refund".

**resource cell, stage of a resource cell.** A *resource cell* is a cell whose overlay type has
the tiberium flag; its *stage* is its overlay data value, which sets how many *bales* it holds.
S-0401 owns both (Terms, claims 1 and 2) and how resource grows, spreads and is removed; S-0402 and
S-0410 use them. Specs write "stage of a resource cell" wherever "stage" alone could be read as a
stage of a sprite or animation (S-0108, S-0109).

**sale.** The course of selling one building, from the first processing of its *selling* behaviour
(S-0215) to its removal or its return to another behaviour. S-0408 owns it (Terms, claims 4 to
10), and S-0405, S-0406 and S-0415 use it.

**switched on, switched off.** The two states of a building's power switch, which power orders
and some trigger actions change. S-0213 (claim 13) calls them *online* and *offline*; specs write
"switched on" and "switched off". S-0405 owns the power switch and its rules (Terms, claim 10); S-0404, S-0407, S-0408,
S-0410 and S-0411 use it. It is not *in operation*.

### production

**barracks door kind.** One of three building-type flags that give an infantry factory a door
cell tried first as its *exit cell*. S-0404 owns the term and the door-cell rules (Terms, claim 13).

**build limit.** A type's imported limit on how many objects of it a house may have, or may ever
have built. S-0407 owns it (Terms, claims 9, 10 and 16); S-0404, S-0406, S-0411 and S-0415 use it.

**build time, progress, step period.** An *item*'s *build time* is the number of ticks that S-0403
computes for it. A *production line* builds its item in *progress* steps, one every *step period*
ticks, which follows from the build time. S-0403 owns the three terms (Terms, claims 1 to 8),
and S-0404, S-0405 and S-0411 use them.

**cloning building.** A building whose type has the cloning flag. S-0404 owns the term and the
copies of released infantry (Terms, claim 14); S-0213 and S-0214 use it for rally-capable buildings.

**could make, could release.** Whether a factory building of an item's kind can serve an *item*
of a given type, and whether it can release the finished object. S-0404 owns the tests (Terms,
claims 3, 7 and 8); S-0403, S-0406, S-0407 and S-0411 use them.

**defence, ordinary building.** A building type is a *defence* when its build category field is
the combat category, and an *ordinary building* otherwise. The two are separate kinds for the
production lines and for the per-kind country factors. S-0403 owns the split (Terms); S-0404,
S-0406, S-0408, S-0411 and S-0413 use it.

**exit cell, exit displacement.** In production, an *exit cell* is the cell a released unit
leaves its factory building towards. An *exit displacement* is a building type's displacement of
the point where a released unit appears, in leptons. S-0404 owns these terms and the release rules
(Terms, claim 13). An exit cell in another context, such as a tunnel, keeps that context's meaning.

**factory building.** A building whose type's factory field names the kind of object it makes:
infantry, vehicles, aircraft or buildings. S-0403 defines it (Terms, *factory building of a
kind*). S-0404 owns which factory building *could make* an item and which one releases it (Terms, claim
7), and S-0213 (Terms) names the *primary factory*, the one that receives a house's new objects of
its kind. S-0406, S-0407 and S-0411 use the term.

**item.** The one object that a *production line* is building at a time; its type is the type
ordered. S-0403 names it (Terms), and S-0404 owns how an item starts, is paid for, held, cancelled
and completed (claims 3 to 9). S-0406, S-0407, S-0411, S-0413 and S-0414 use it.

**line kind.** Which of a house's production lines builds a type: infantry, ground vehicles, naval
vehicles, aircraft, buildings or defences. S-0403 names the kinds (Terms) and S-0404 (claim 1) the
lines a house has; S-0411 uses the term. S-0413 (Terms, *Factor kind*) groups items into five
kinds for the per-kind country factors, with ground and naval vehicles in one kind.

**offered.** A type is *offered* to a house while it is in the house's taken-up set (S-0407,
Terms and claims 1 and 12). The set changes only at a recheck. A type can therefore remain offered
after current conditions change, until a recheck removes it. *Buildable* describes current
conditions: the type's current tech-tree answer is allowed or at limit, and some factory building
of the house could make it without the tech-tree rule; a factory building being sold or queued to
be sold does not count (S-0411, Terms; S-0407, Terms and claim 1; S-0404 owns *could make*).
Being sold and queued to be sold have the meanings of S-0407 Terms. A type can remain offered while no longer
buildable, or become buildable before the next recheck while remaining unoffered. Buildability
alone does not remove an icon or give cancellation orders (S-0411, claim 2; S-0404, claim 9).
S-0411 uses the offer for order legality and icon membership, and current conditions for darkening
(claims 2 to 4). Darkening is not a full buildability test: stale prerequisite refusal alone does
not darken an icon (S-0411, claim 3). S-0415 also uses *offered*.
Capacity-aware offers and delayed refill are owned by S-0411 (claim 15): a type suppressed by a
full tab is not offered, and a vacancy left by removals at a recheck is not filled during that
same recheck or an ordinary sidebar update. Still-eligible types compete for admission at the
next recheck; a vacancy alone schedules none. In our engine this offer rule applies to every
player house, not just the local sidebar.

**place order, build order, hold order, cancel order, cancel-all order.** The production orders a
house gives. Each names a type, and so a *production line*; a place order of a building also names
a cell. They are orders in the sense of S-0213. S-0404 owns them (Terms, claims 3 to 7), S-0406
what a place order needs (claims 1 to 12) and S-0411 which clicks give them; S-0407 and S-0415 use
them.

**prerequisite.** A type that a house must own (usually a building) before another type can be built.
Which types are prerequisites of which comes from imported fields. S-0407 owns when a prerequisite
is met (claim 8), including the *generic groups* that a prerequisite list can name instead of a
type (Terms).

**price.** What a house pays for one *item* of a type, in credits. S-0404 owns it (Terms, claim
2), and S-0403, S-0408, S-0412 and S-0413 use it.

**production line, queue.** One of a house's production processes: its *item*, the item's
progress, its hold state and its *queue*, the ordered list of types waiting to be started after
the item. S-0203 (Terms) names it and places its update in the tick (claim 8). S-0404 owns how many
lines a house has and what a line does (Terms, claims 1 to 12), with the line states *player hold*,
*automatic hold* and *waiting for money*. S-0402, S-0403, S-0405, S-0411 and S-0414 use it, and
S-0406 and S-0407 shorten it to "line".

**tech level.** Ambiguous on its own. A *type's tech level* is its imported tech-level field
(S-0108, Terms); the *game's tech level* and a *house's tech level* are S-0009's (Terms), and
S-0407 (Terms, *house tech level*, claim 5) owns how the house's tech level limits what it may
build. S-0406 and S-0412 use them. A spec says which of them it means.

**tech-lab list.** A game-wide list of building types in the rules data. A spy that infiltrates a
building on it can win its house stolen tech (S-0407, claim 6), and the superweapon setting never bars
a building type on it from being built (S-0412, claim 9, part a). S-0407 owns the term (Terms). S-0412 uses it.
Specs write "tech-lab list", not "tech-building list", which is easy to confuse with the *tech*
generic group of S-0407 (Terms) and with the neutral buildings that players capture.

### visibility

**appears disguised to H.** The game-state test of whether a unit's disguise fools house *H*.
It is separate from how the unit is drawn for a viewer (S-0516 claim 35).

**cloak delay.** The per-object countdown in ticks set when an uncloak completes, during
which the object cannot start cloaking again (S-0516 claim 7). Buildings do not start this
mobile-object delay on completing an uncloak (S-0516 claim 18).

**cloak field.** The cells a house's cloak-generating buildings hide for that house. Coverage
is a yes-or-no flag per house and cell, not a count (S-0516 claims 19 and 20).

**discovered by a house.** A per-house mark of objects, set by reveals for that house. S-0409
owns when it is set (Terms, *discovered by H*, claim 16); S-0410 (claim 7) uses it.

**disguise-detector coverage.** A per-house, per-cell signed count fed only by detector
buildings. A positive count defeats disguises on that cell in both the game-state and
presentation tests, but does not reveal cloaked objects (S-0516 claim 40).

**explored, seen.** Two per-house states of every legal cell. An *explored* cell is free of
*shroud* for the house; with *fog of war* on, a *seen* cell is also not under fog. The opposites are
*unexplored* and *unseen*. S-0409 owns when cells become explored and seen and when they are
covered again, for every *player house* (Terms, claims 1 to 19). S-0110 (Terms) uses the words for
the local player when it draws the cover, and S-0406 (claim 5) and S-0410 use them.

**fog of war.** An optional game setting. With it on, map areas a house has explored but cannot see at
the moment keep showing what the house last saw there; S-0102 says which objects appear there as
*remembered images* and how they are drawn. With it off, explored areas show what is there now.
When an area counts as explored or seen is S-0409's (see *explored, seen*); what is drawn under fog is S-0102's and S-0110's. *Shroud* is a separate
cover.

**gap generator, gap field.** A *gap generator* is an object whose type has the gap-generator
flag. While it generates, it holds one *gap field*, a set of cells around it. S-0409 owns both
(Terms, claim 11); S-0110 (Terms) names the stock building, and S-0405 (claim 16) and S-0410 use
the terms.

**hidden from H.** Fully cloaked on a cell that house *H* does not sense. A building also
counts at fade levels 11 to 15. This game-state test applies to the owner too
(S-0516 claim 24).

**re-disguise lockout.** A countdown in ticks during which a disguise-when-still vehicle
cannot take a new disguise. S-0302 claim 12 calls it the disguise-disruption countdown
(S-0516 claims 33 and 34).

**sensor coverage, sensed cell.** A signed count per house and cell, fed by that house's
sensors only. A cell is sensed by a house while its count is above 0. Allies do not share
coverage (S-0516 claims 21 to 23).

**shroud.** The cover that hides map areas a house has not explored yet. When a cell becomes
explored, and when it is covered again, is S-0409's (see *explored, seen*); S-0110 draws the
cover.

**sight, sight radius.** An object's *sight* is the coverage it holds for a house: the cells it
explores around its *sight centre*. Its *sight radius* is the radius in cells that its sight uses
at a given moment. S-0409 owns both (Terms, claims 2 to 10); S-0406 points to it.

**spy satellite.** A building type flag, and a building of a type with it, whose active reveal
explores the whole map for its house. S-0409 owns it (Terms, claim 15); S-0405 (claim 14) and
S-0410 use it.

### superweapons

**arrival hold.** The period after a warp arrival during which the object is still phasing in,
lasting its held delay in ticks (S-0513 claim 40).

**camera sight.** The spy plane's own sight with its radius replaced by its primary weapon's
damage, subject to the radius cap and zero-damage fallback of S-0513 claim 44.

**charge clock, remaining time.** A superweapon record's running or stopped countdown toward
readiness, with remaining time in ticks. A running clock counts down by one per tick, never
below zero; a stopped clock keeps its remaining time. S-0512 claim 5 owns completion.
Its claims 10 and 14 describe clocks that run despite suspension.

**clock stage.** The whole-number charge progress from 0 to 54 shown on a superweapon icon.
S-0512 claim 16 owns the display, including drain-mode exceptions.

**cloud, strike.** A *cloud* is a storm cloud animation over a cell; its *strike* is the bolt
with damage it makes when it matures. S-0513 claims 16 to 18 own them.

**countdown row.** One line of the on-screen superweapon countdown (S-0512 claim 18).

**drain-mode superweapon.** A type with the imported drain-mode flag, whose record has the
states charging, ready and active. Its orders can restart the clock despite suspension, and
one-time drain records are not consumed by those transitions under the original behaviour
(S-0512 claim 19).

**effect block.** The 3 × 3 cells centred on a cell, in the fixed order that S-0513 claim 4
specifies for the iron curtain and chronosphere.

**giving building.** For one house and superweapon type, an on-map, non-destroyed building
that gives the type through its superweapon fields or a carried upgrade. S-0512 claim 3 owns
granting and its giving-building definition includes the auxiliary-building condition.

**launching house.** The house that owns the superweapon record being launched
(S-0513 claim 5).

**leaving stage.** The fixed 60-tick period before a warped object leaves its old position,
during which it counts as being erased (S-0513 claim 39).

**manual-charge superweapon.** A type whose record is granted with its clock stopped at the
full charge time and charges only when a trigger sets its charge (S-0512 claim 20).

**one-time superweapon.** A trigger-granted single use, ordinarily revoked on firing.
S-0512 claim 12 owns the grant and stale-suspension exception; claim 19 owns the drain-mode
exception, whose orders do not consume a one-time record under the original behaviour.

**permanent trigger grant.** A superweapon record granted for good by a trigger action,
exempt from building and power suspension or loss checks (S-0512 claim 13).

**phasing in.** A warped object's state from leaving its old position until its arrival hold
ends (S-0513 claims 39 and 40).

**protection timer.** The countdown of an object's temporary invulnerability, with its kind:
curtain or force shield (S-0513 claims 25 and 26).

**ready (superweapon).** A granted superweapon record whose charge has completed
(S-0512 claim 5). S-0411's sidebar sense additionally requires no suspension and being inside
the flash window (S-0512 claim 17).

**storm, weather storm.** The single game-wide storm of the weather superweapon, with states
idle, counting down, raging and clearing (S-0513 claims 14 to 19). S-0524 claim 1 uses
*weather storm* for this same effect; S-0104 calls it the *ion storm*.

**storm owner.** The house credited for storm strikes, possibly no house (S-0513 claim 19).

**superweapon record.** The availability and charge state of one superweapon type for one
house, including granted, ready and suspended state and its charge clock. More giving
buildings do not give the house more charges of the same type (S-0512 claim 1).

**suspended (superweapon).** A granted record whose power condition disables sidebar targeting
and normally stops its charge clock. It may still be ready. S-0512 claims 4 and 6 own
suspension; claims 10 and 14 give running-through-suspension exceptions. Specs write
*suspended*, not *held*: the latter is the mobile-unit countdown.

**swept layer.** The cell layer whose listed objects an area superweapon reaches in one cell
(S-0513 claim 4).

**warning marker.** The animation at a nuclear strike's target shown only to qualifying
non-allied viewers with psychic detection (S-0513 claim 11).

**warp group.** The mobile objects one chronosphere warp moves or kills (S-0513 claim 34).

### triggers

**cell tag.** A trigger tag attached to one cell by the map. How a map attaches cell tags is
specified in S-0009 (claim 31). What an object entering the cell sets off is owned by S-0011
(claim 18, the entered-by event) for the codes of its subset; other codes are left to later
revisions of the trigger specs.

**demolition warhead.** The warhead the rules name as the demolition-charge warhead
(S-0524 claim 5).

**entry raise.** An entered-by raise on the entered object's tag, with the entering unit as
causing object. S-0524 claims 10 to 12 own the additional object-entry raises; S-0011 claim 18
owns event matching.

**starting-building record.** The snapshot of each house's owned building types and cells
taken at the end of loading, before the first tick. It records neither health nor tags
(S-0524 claim 9).

**trigger, trigger action.** A *trigger* is a rule that a map defines: when its conditions are met,
it runs its *trigger actions*, such as playing an animation or changing the lighting. S-0011 owns
how triggers are set up, evaluated and fired, and the events and actions of its multiplayer subset.
Codes outside that subset are pointers only, left to later revisions of the trigger specs; some of
them are owned elsewhere, such as the freeze and chrono-flash actions of S-0201 (claims 10 and 11).
Specs write "trigger action", not "script action".

**waypoint point.** The centre of a waypoint's cell at its terrain surface height, raised
416 leptons on a high bridge. S-0011 claim 28 owns the point; S-0524 claim 5 uses it for
demolition blasts, unlike the cell-only actions of S-0524 claims 1, 2 and 4.

### ui

**announcement.** A spoken line of the battlefield announcer, from the announcement data.
Announcements play one at a time separately from sound channels. See S-0701 (Terms, claims 26
to 28).

**audio clock.** Real time in milliseconds, distinct from ticks. S-0701 owns which sound timings
use it rather than game time (Terms, claims 9 to 14, 27 and 29).

**build icon, cameo.** One entry of a *sidebar tab* (S-0004, Terms): a type *offered* to the local
player, shown by its picture. S-0004 (claim 11) owns the tab and position of each icon, and
S-0411 what an icon shows and which orders its clicks give (Terms, claims 1 to 12), and
capacity-aware offers and delayed refill (claim 15); S-0404 and
S-0407 use the term. Specs write "build icon"; "cameo" means the same.

**channel.** In audio, one of the 16 mixing voices on which sound instances play; announcements
play separately. This is not a colour component in rendering. See S-0701 (Terms, claims 9 and 27).

**free mover.** An actor that the click rules of S-0214 treat apart when it is sent to a cell or an
object: one that is in the air, is an aircraft, is of an *underground type* or is of a jumpjet infantry type,
with the conditions S-0214 gives. S-0214 owns the term (Terms) and its rules (claim 11).

**lobby.** The pre-game screen of a skirmish or network game, where the players, their countries,
colours, teams and start positions and the game mode are chosen.

**positional sound.** A sound request at a map position, whose level and left-right placement
follow from where that position appears in the battlefield view. See S-0701 (Terms, claim 6,
Formulas F2).

**radar, radar map, radar building.** A house *has radar* while the conditions of S-0410 (claim
1) hold. The *radar map* is the picture in the sidebar's radar panel, and a *radar building* is a
building whose type has the radar flag. S-0410 owns the three terms (Terms, claims 1 to 11), and
S-0405 (claim 12) and S-0409 use them.

**response.** A spoken acknowledgement by one of the local player's objects on selection, an
order, damage or death. Its kind uses a list or a single sound entry on the object's type, with
the fallbacks owned by S-0701 (Terms, claims 16 to 20, Formulas F6). S-0214's *selection
responses* are the selection kind.

**screen sound.** A sound request not tied to a map position, with centre left-right placement.
Its request level is full unless the request specifies a volume fraction; the entry's own level
still applies. See S-0701 (Terms, claims 5 and 35).

**sidebar.** The panel beside the battlefield view during play, with the radar, the local
player's credits and the build icons. S-0004 (claim 11) owns the order of the build icons, S-0411
what the icons show and what clicking them does (see *build icon*), S-0416 the credits counter and
S-0410 the radar map (see *radar*). The rest of its behaviour is left to later interface specs.

**sound entry.** A named entry of the sound data, holding a list of samples and sound settings.
In-game sounds other than music and announcements play sound entries. See S-0701 (Terms,
claims 1 and 2).

**sound instance.** One playing of a sound entry, from its request until it ends or is cut.
See S-0701 (Terms, claims 8 to 14).

**sound level.** Loudness on a scale from 0 (silent) to 16384 (full). A spec that instead uses a
level fraction in 0 … 1 says so. This is not a height level. See S-0701 (Terms, claim 15,
Formulas F1 to F4).

**vehicle thief.** An infantry unit whose type has the imported vehicle-thief flag: it takes over a
vehicle by entering it. Its rules belong to the special-abilities specs. See S-0209 and S-0214.

## Behaviour

### Simulation: direction and game time

These are the only claims the glossary makes itself. Every other statement about the original in an
entry above belongs to the spec its "See S-xxxx" pointer names (see Conventions).

G1. **Direction convention.** Directions in the original have the zero and the sense of *facing*:
    0 points to map north (decreasing cell y, upper right of the screen), and directions grow
    clockwise as seen on the screen, so a quarter turn points to map east (increasing x), a half
    turn to map south (increasing y) and three quarters of a turn to map west (decreasing x). A
    vehicle pre-placed by a map starts facing *v*/256 of a turn, where *v* is the map's facing value
    (an integer from 0 to 255), that is, *v* × 256 facing units. The 65536 facing units of a full
    turn are our own unit (see *facing*). How finely the original distinguishes directions in general is
    S-0204's (claim 10).
    - Backing: `measurable` (Scenario G1) for the zero, the sense and the map value's scale to
      within the 1/32 turn that a drawn vehicle body shows (S-0105, claim 3). The step of the map
      value finer than that, down to 1/256 turn, is `measurable` through the time a first turn
      takes (S-0206, scenario R2, with the turn rule of S-0206; S-0204, claim 10)
    - Confidence: high for the zero, the sense, the scale and the 1/256-turn step, as for S-0204
      claim 10

G2. **Game-speed direction (skirmish).** In a skirmish game, with a game-speed index *s* from 0 to
    6:
    - (a) for every *s* from 2 to 6, index *s* runs strictly fewer ticks per second of real time
      than index *s* − 1.
    - (b) index 0 runs at least as many ticks per second as index 1. (b′) Index 0 adds no wait of
      its own between ticks, so its tick rate is limited only by the time the machine takes to
      compute and show each tick, and it runs strictly more ticks per second than index 1
      whenever the machine and the display path can compute and show ticks faster than index 1
      allows.
    - (c) the skirmish lobby's speed control sets the same index: its seven positions, from the
      slowest to the fastest, set indices 6, 5, …, 0.

    Part (a) holds whenever the machine can compute and show a tick in less real time than index
    *s* − 1 allows. A run in which index 0 is faster than index 1 shows that this is so for every
    index from 1 to 6.

    S-0201 states the same behaviour in more detail and cites this claim (see Conventions): its
    claim 2 gives each index from 1 to 6 a tick rate of 62.5 / *s* ticks per second on a host that
    keeps up, which implies part (a); its claim 5 restates part (b′); its claim 7 restates part (c)
    and says where the control's starting position comes from. S-0201 (claims 4 and 5) also gives
    our engine fixed paced periods, 8 ms at index 0, as a departure; that is a rule of our engine
    and leaves this claim about the original unchanged.
    - Backing: `measurable` (Scenario G2) for parts (a), (b) and (c). Part (b′) is `unmeasured`:
      whether index 0 adds a wait is internal to the original, and its one observable consequence,
      *N*₀ > *N*₁ in Scenario G2, depends on how much faster than index 1 the machine and display
      path could run, which no observable of the scenario establishes independently. A run set
      with *N*₀ > *N*₁ is consistent with (b′), but one without it cannot falsify (b′).
    - Confidence: high for parts (a), (b) and (b′), as for S-0201 claims 2 and 5; medium for part
      (c), as for S-0201 claim 7

G3. **Game speed and timings counted in ticks (skirmish).** In a skirmish game the game speed
    changes how much real time passes between ticks (claim G2). Every timing of game state counted
    in ticks takes the same number of ticks at every game speed, with two exceptions: the
    speed-normalised delays and whatever waits on them, and a chrono flash during a freeze. For
    example, a production time and a movement over a fixed route take the same number of ticks at
    every game speed.

    **Exception 1: the speed-normalised delays.** The original converts some animation stage
    delays with the game-speed normalisation N(*d*, *s*) of S-0107 (Formulas F3), so that they last
    about the same real time at every game speed. Their length in ticks therefore depends on *s*, on purpose. Which delays
    are converted, and when, is owned by the specs named here; G3 only points to them:
    - the infantry sequence kinds that S-0107 claim 5 lists (*idle-a*, *idle-b*, *wet-idle-a*,
      *wet-idle-b*, *hover* and *cheer*),
    - the building body stages that S-0108 claim 6 converts, including the wrap of a body range,
    - animations whose type is marked as normalised (S-0109, claim 4),
    - the period of the repair wrench (S-0111, claim 19), which is drawn only.

    G3 makes no claim about any other delay that may be converted the same way.
    When game speed changes during a game, which speed a converted delay uses is also owned by
    these specs (see *speed-normalised delay*).

    A rule that waits for one of these stages inherits the dependence, and that can reach game
    state, not only what is drawn. The specs that own these rules say how. For example, by the
    claims of S-0107 an idle attempt goes ahead only while the current sequence kind is stand,
    guard or tread, so the length of an idle sequence changes which idle-timer expiries lead to an idle attempt that
    goes ahead, and with them the number of draws from the scenario random stream (S-0107, claim
    14). A *cheer* cannot be interrupted by a request that is not forced (S-0107, claim 6), and a
    shot leaves when the current sequence's stage reaches the fire frame (defined in S-0107, claim
    11), so the shot timing of a cheering unit can depend on the game speed (S-0107, claims 6 and
    11). A building's construction timing follows its body stage (S-0108, claims 5 and 6). The
    damage, end effects and random draws of a normalised animation follow its stages, so they run a
    different number of ticks at different game speeds (S-0109, claim 18). G3 makes no claim about
    any of these. It makes no claim either that exception 1 is only visual.

    **Exception 2: a chrono flash during a freeze.** A timed freeze lasts a fixed real time, so its
    number of frozen cycles depends on the game speed (S-0201, claim 10). A chrono flash in force
    takes one step in every cycle, frozen cycles included, and its own freeze starts at a fixed step
    (S-0201, claim 11). So when a timed freeze falls within a chrono flash, the number of ticks that
    run before the chrono flash's own freeze, and whether the two freezes merge into one hold,
    depend on the game speed (S-0201, Formulas F4; in its scenario T6, part G, the second hold
    begins at tick number 365 at index 6 and at 315 at index 1). Every rule of game state that
    waits on that freeze, or on the end of the chrono flash's order block, inherits the dependence.
    S-0201 owns this path (claim 11). G3 only points to it. (S-0201 also has a chrono flash step
    during the freeze of a radar-panel movie, but that happens only in a campaign mission, which
    always runs at index 2 (S-0201, claims 11 and 12), so it is outside G3's skirmish scope.)
    - Backing: `measurable` for production and fixed-route movement (Scenario G3). The assertion
      that these are the only two kinds of exception is `unmeasured`: no finite set of scenarios
      checks every game-state timing and every interaction; Scenario G3 tests examples, not that
      exhaustive assertion. Each named exception inherits the backing and scenarios of its owning
      spec; no completed measurement is claimed here
    - Confidence: medium (no other use of the game-speed index by a game rule is known. The one
      timer of the original known to run in real time during a game and reach game state is the
      length of a timed freeze, which reaches it only through exception 2 (S-0201, claims 10 and
      11); our engine counts that length in cycles, a departure of S-0201 that removes the host's
      influence but keeps the game-speed dependence. Relighting does not depend on real time
      (S-0104, claim 12). Not every timer of the original is known)

    **Our engine.** It reproduces both exceptions by default. Game speed reaches game state in our
    engine only through the single switch `quirks.speed_normalised_delays` (default: the original
    behaviour), which covers both exceptions: exception 1, shared by S-0107, S-0108, S-0109 and
    S-0111, and exception 2, the chrono flash's steps during a timed freeze, owned by S-0201 (claim
    11, Edge cases and quirks). There is no separate switch for exception 2. With the switch off,
    every path it covers loses its game-speed dependence, as follows:
    - *Infantry sequences (S-0107, claim 5).* Every sequence kind, the six normalised kinds
      included, uses its base delay *d* = *b* at every game speed, so every sequence lasts the same
      number of ticks at every speed. Which idle-timer expiries lead to an idle attempt (S-0107,
      claim 14), and so the draws it takes, and the shot timing of a cheering unit (S-0107, claims
      6 and 11) no longer depend on the game speed.
    - *Building body stages (S-0108, claim 6).* Every delay that S-0108's table gives as
      N(*d*, *s*) is the raw *d* instead, so every body stage lasts the same number of ticks at
      every speed, and S-0108's switch `quirks.building_wrap_normalisation` has no effect. The build-up
      delay of S-0108 claim 5 is raw in both settings.
    - *Animations (S-0109, claim 4).* An animation whose type is marked normalised keeps the delay
      *d* chosen for it, at every speed. So its damage, the effects of its middle event, the events
      at its pass ends and at its end, and the change to its follow-up type (S-0109, claim 18) take the same
      number of ticks at every speed.
    - *Repair wrench (S-0111, claim 19).* Its period is 3 ticks at every speed. It is drawn only.
    - *Chrono flash during a freeze (S-0201, claim 11, Formulas F4).* A chrono flash takes a step
      in a frozen cycle only while its own freeze is in force. A timed freeze with no chrono flash
      in force is unchanged. When a timed freeze starts before the chrono flash's step *D* + 44,
      the chrono flash waits through it and then still runs *D* + 44 − *k* ticks before its own
      freeze, where *k* is the steps taken by the end of the tick in which the timed freeze
      starts, so there are always two holds, with the same number of ticks between them at every
      speed. When the timed freeze starts in the tick of step *D* + 44, the two freezes make one
      hold, as with the switch on. In the example of exception 2 (S-0201, scenario T6, part G) the
      second hold then begins at tick number 374 at every speed.

    So with the switch off no timing of game state counted in ticks depends on the game speed. The
    game speed still sets the real time between ticks (claim G2) and the number of frozen cycles of
    a timed freeze (S-0201, claim 10), but no tick runs in a frozen cycle, so that number changes
    no timing counted in ticks. A path by which game speed reaches game state that a later spec
    finds joins this switch instead of getting one of its own, and that spec says what it becomes
    with the switch off. This paragraph and its list are a rule of our engine, decided by the
    project lead, and not a claim about the original; the confidence above is that of the claim
    about the original. The specs named in the list give the same rule in their own terms.

## Formulas

> **Facing represented as a turn:** *f* / 65536 of a full turn, clockwise from map north. *f* is an integer from 0 to 65535; every sum or difference of facings is reduced modulo 65536 to that range. The ratio is exact integer arithmetic; no rounding applies. Unit: turns. (Definition.)

> facing of a pre-placed vehicle = 256 × *v*, where *v* is the map's facing value, an integer from
> 0 to 255. Exact integer arithmetic, so no rounding applies. Unit: facing units. (Claim G1.)

> centre of cell (*x*, *y*) = (256*x* + 128, 256*y* + 128). *x* and *y* are integer cell
> coordinates; exact integer arithmetic, so no rounding applies. Unit: leptons. (Owned by S-0009
> and S-0101.)

Every other formula belongs to the spec that uses it, together with its rounding rule.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Cell edge | 256 | leptons | See S-0009 and S-0101 |
| Height level | 104 | leptons of world height | See S-0101 |
| Full turn | 65536 | facing units | Definition (our unit, see *facing*) |
| Map facing value step | 256 | facing units per step | 256 steps per turn; claim G1 (the step finer than 1/32 turn is tested by S-0206 scenario R2) |
| Fastest game speed | 0 | game-speed index | Claim G2 |
| Game-speed indices covered | 0 to 6 | game-speed index | Claims G2 and G3; S-0107's table T3 also has a column for 7, outside this range (see *game speed*) |
| Ticks per game minute | 900 | ticks per game minute | See *game minute*; owned by S-0107 and S-0108 |
| Colour-adjustment table | 16 | tint colours | See S-0006 |
| Remap range | 16 to 31 | palette index | See S-0103 |
| Palette size | 256 | entries | See S-0103 |
| Per-mille unit | 1000 | ‰ per factor 1.0 | Definition |
| Render layers | 5 | layers | See S-0102 |
| Theaters | 6 | theaters | See S-0007 and S-0103 |

## State and ordering

None of its own. The orders the entries mention (loose files before archive members, the list of
rules layers and what it decides, the order in which houses are created) are stated as observable
consequences in the specs they point to: S-0004, S-0005, S-0006 and S-0009.

## Edge cases and quirks

The glossary owns no quirk; *quirk switch* defines the notation the specs use. It points to five
switches that several specs share. Each spec that uses one names it, and no spec defines a second
switch for the same behaviour:
- `quirks.speed_normalised_delays`: both exceptions of claim G3, and the one switch through which
  game speed reaches game state in our engine. Exception 1 is used by S-0107, S-0108, S-0109
  (claims 4 and 18) and S-0111 (claim 19); exception 2, the chrono flash's steps during a timed
  freeze, by S-0201 (claim 11, Edge cases and quirks), which has no switch of its own for it.
  What the switch does when turned off, on each of these paths, is given in claim G3 (paragraph
  *Our engine*).
- `quirks.chop_rounding`: the *round-toward-zero mode*, owned by S-0205 (claim 5), whose Edge cases
  list the claims of other specs that depend on it.
- `quirks.table_root_distance`: the table square root of S-0204 (claim 5, Formulas F1), owned by
  S-0204. Every distance taken with that table follows it. The specs that name it are S-0104,
  S-0009 (the farthest-sum rule for start positions), S-0207, S-0209 (claim 8, Formulas
  F1), S-0212, S-0214, S-0215 and S-0402 (claim 9), and S-0208, which notes that it changes nothing there. An earlier revision of S-0009
  proposed a switch of a different name for the same table; by decision of the project lead the two
  are one switch under this name, and S-0009 uses it. It is the only name
  of this switch.
- `quirks.last_bale_lost`: the last bale of a resource cell, owned by S-0401 (claim 3); S-0402
  names it for what a harvester gets.
- `quirks.radar_first_building_decides`: the first qualifying radar building deciding a house's
  radar, owned by S-0410 (claim 1, Edge cases and quirks). S-0405 (claim 12) and S-0409 point
  to it. The original tests only that first building; with the switch off any qualifying building
  that is neither electromagnetically locked nor being erased can provide radar.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| *Settings file*, section `[Skirmish]`, key `GameSpeed` | game-speed index *s* of a skirmish, as the starting position of the lobby's speed control (claims G2 and G3; S-0201, claim 7) | integer; the claims cover 0 to 6, and no other value is covered here (7 has a column in S-0107's table T3) | S-0201: the rules' lobby default (rules, section `[MultiplayerDialogSettings]`, key `GameSpeed`), and 6 when that is absent too; every scenario here sets the key |
| *Settings file*, section `[Options]`, key `GameSpeed` | not the speed of a skirmish started from the lobby: S-0201 (claim 7) replaces it at skirmish start with the lobby's index | integer | 3 (S-0201) |
| Map, pre-placed vehicle section `[Units]`, the sixth comma-separated value of an entry | the vehicle's starting facing, as *v* (claim G1) | integer; the claim covers 0 to 255, other values are not covered | each scenario writes it |

Every other key belongs to the import mapping of the spec that owns its concept.

## Measurement scenarios

All three run under the *measurement setup*: an unmodified install with no engine add-ons.
Use the standard skirmish mode at 800 × 600 pixels, with the director and mods off, no superweapon
use, and no trigger effects unless a referenced sub-test explicitly needs them.
Their local tests need no extra data beyond their test maps. The finer-facing sub-test delegated
by G1 to S-0206 R2 uses the named test rules file of that scenario. Observations are made with a
*recorder*. Each scenario records the display path it used, naming any display compatibility layer
generically (see *measurement setup*).

**Scenario G1: direction convention (claim G1).**
- *Setup:* a purpose-made flat map of one theater, no hills, no overlays near the test area. A
  skirmish with one human player and one bystander computer player whose start position is on the
  far side of the map. The map pre-places seven vehicles of one stock vehicle type that is drawn
  as a voxel model and whose body has a visible front. They are owned by the human player (the map names their owner
  as S-0009, claim 27, requires), each stands on its own cell at least three cells from the others,
  and their facing values are 0, 8, 16, 32, 64, 128 and 192. The observed area is the test area
  where the seven vehicles stand. Any game speed.
- *Procedure:* start the game, give no orders and pause on the first tick the map is visible.
  Take a screenshot.
- *Observable:* for each vehicle, the direction its front points on the screen, read against the
  cell grid: the neighbouring cell it points to, or the two neighbours it points between.
- *Runs:* 1 (no random draw is involved).
- *Expected:* value 0 points to cell (*x*, *y* − 1), upper right on the screen; 32 to
  (*x* + 1, *y* − 1), straight to the right; 64 to (*x* + 1, *y*), lower right; 128 to
  (*x*, *y* + 1), lower left; 192 to (*x* − 1, *y*), upper left. Value 16 points between the
  directions of values 0 and 32 and is drawn differently from both (in the map plane it lies
  exactly halfway, but on the screen the projection distorts angles, so halfway is not tested).
  Value 8 points between the directions of values 0 and 16 and is drawn differently from both. All
  seven are drawn in visibly different directions.
- *Falsified if:* value 0, 32, 64, 128 or 192 points to a different neighbour, or value 8 or 16 is
  drawn the same as a neighbouring value of the list or points outside the angle between its
  neighbours. If all are rotated by the same quarter or half turn, the zero is wrong; if 64 and 192
  are swapped, the sense is wrong; if 8 looks the same as 0 or 16, the map value moves the drawn
  body in steps coarser than 1/32 turn, against the scale of G1.
- *What it cannot show:* a vehicle body is drawn in 32 directions (S-0105, claim 3), so values that
  differ by less than 8 can be drawn alike whatever the step. The 1/256-turn step of claim G1 is
  tested instead through turn times, by S-0206 scenario R2.

**Scenario G2: game-speed direction (claim G2).**
- *Setup:* a small purpose-made or stock skirmish map, one human player and one bystander computer
  player. The scenario has no test objects and observes no area of the map, only the number of
  ticks run, so of the bystander's void rule only its defeat applies: a run in which the bystander
  is defeated before the 300 seconds end is void. Before each run the settings file's skirmish game-speed value is set to one of 0, 1, 2, 3, 4, 5
  and 6, and in the lobby the speed control is left at the position where it starts, which by
  S-0201 (claim 7) gives that index; the run records the starting position. A recorder that reads
  the number of ticks run. A display path that does not wait for the monitor's
  refresh, because one that waits can hold index 0 down to the rate of index 1 (see *display
  path*). If the game's own
  display path waits for it, a display compatibility layer that does not wait is used, and the
  record names it generically.
- *Procedure:* start the game and let it run with no orders for 300 seconds of real time after
  the map is visible.
- *Observable:* *N*ₛ, the number of ticks run in those 300 seconds at index *s*, the mean of the
  runs at that index; the lobby control's starting position in each run.
- *Runs:* 2 per index. A second set of runs, 2 per position, sets the speed by moving the skirmish
  lobby's speed control to each of its seven positions instead, and records the counts the same
  way. The second run set has the setup of S-0201 scenario T1, part B, which also sets the speed
  with the lobby's control, and may serve both.
- *Expected:* *N*₀ ≥ 0.99 × *N*₁ (part b; 1 % allows for timing noise, as in S-0201 scenario T1).
  In a run set with *N*₀ > 1.01 × *N*₁, which shows that the machine can compute and show ticks
  faster than index 1 allows and so establishes the condition of part (a), *N*ₛ < 0.99 × *N*ₛ₋₁ for
  every *s* from 2 to 6. The lobby positions, from the slowest to the fastest, give counts within
  1 % of *N*₆, *N*₅, …, *N*₀ in that order. Where two neighbouring indices have counts within
  2 % of each other (in practice only indices 0 and 1, when the machine or display path limits index 0), the
  counts cannot tell their two lobby positions apart. Those two positions are then re-run as in
  *Resolution*, 5 runs per position of 600 seconds each, with the two indices re-run the same way.
  If the indices' counts are again within 2 % of each other, which of the two positions sets which
  index is recorded as inconclusive at this resolution, and part (c) is judged on the other
  positions.
- *Falsified if:* *N*₀ < 0.99 × *N*₁ (part b); in a run set with *N*₀ > 1.01 × *N*₁,
  *N*ₛ > 1.01 × *N*ₛ₋₁ for some *s* from 2 to 6 (part a); or a lobby position's
  count is not within 1 % of the index it is expected to match (part c). A run of the first set
  whose control did not start at the position of its index tests S-0201 claim 7, not this claim,
  and is void here. No result falsifies part (b′), which is `unmeasured` (claim G2).
- *Resolution:* the scenario resolves differences of 1 % or more. A run set with
  *N*₀ ≥ 0.99 × *N*₁ supports part (b) within this tolerance; it cannot establish exact equality.
  A run set in which *N*₀ is not above 1.01 × *N*₁ cannot
  test part (a), because the machine may have limited the rate; part (a) is judged only on a run
  set, on this or a faster machine or display path, in which it is. For part (a), each pair of
  neighbouring indices falls in one of three regions: *N*ₛ < 0.99 × *N*ₛ₋₁ confirms the claim for
  that pair, *N*ₛ > 1.01 × *N*ₛ₋₁ falsifies it, and a count with
  0.99 × *N*ₛ₋₁ ≤ *N*ₛ ≤ 1.01 × *N*ₛ₋₁ neither confirms nor falsifies it; that pair of indices is
  run again with 5 runs per index and longer runs of 600 seconds. If the re-run lands in the band
  again, that pair is recorded as inconclusive at this resolution.

**Scenario G3: timings counted in ticks across game speeds (claim G3).** It tests the part of
G3 outside the exceptions: a production time and a movement. Exception 1 is tested by the
scenarios of S-0107 (M3), S-0108 (B11), S-0109 (AN4) and S-0111 (OV14), exception 2 by S-0201
scenario T6, part G.
- *Setup:* a purpose-made map with open flat ground. A skirmish with one human player who has a
  construction yard whose type has the construction-yard flag and produces buildings, enough
  credits for one building, and one vehicle standing at the start of a
  straight, free route 10 cells long; one bystander computer player whose start position is on the
  far side of the map. Use the same stock vehicle type, initial body facing and route at every
  speed. The house has enough unchanged power for production, one factory building of the chosen
  building's kind, and no harvesting, damage, construction or sale during the observation. The
  observed area includes that factory building, its power buildings and the vehicle's route.
  Test all game-speed indices 0 to 6.
- *Procedure:* in each run, order one building of one stock type (the same type in every run), and
  order the vehicle to the cell at the end of the route. The orders may be given at any time; no
  exact tick is needed.
- *Observable:* *D*ᵇ, the number of ticks from the first tick on which the building's production
  shows progress to the tick on which it becomes ready to place; *D*ᵐ, the number of ticks from the
  first tick on which the vehicle moves to the tick on which it reaches its destination cell.
- *Runs:* 3 per speed. Record each endpoint's tick number with a read-only tick recorder and
  tick-labelled captures. A run is void if another unit interferes, power or factory count
  changes, credits run out, or an endpoint cannot be identified to an exact tick. Repeated
  variation at one speed with these controls is inconclusive and needs investigation, not a pass
  based merely on overlapping ranges.
- *Expected:* the same integer *D*ᵇ and the same integer *D*ᵐ in every valid run at all speeds.
- *Falsified if:* either duration is repeatably different at any two speeds under these controls.
  Equality supports only these two examples; it cannot confirm the exhaustive part of G3.

## Open questions

- How game speed works in network games is left by S-0201 (claim 13) to a future network-game spec.
  Claims G2 and G3 cover skirmish games only, and S-0201 claim 6 (one drawn frame per cycle) covers
  single-player games only.

## Questions for the project lead

No question for the project lead remains open in this revision.
