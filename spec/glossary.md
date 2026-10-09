---
id: glossary
title: Glossary of shared terms
topic: rules-import        # cross-cutting; every topic uses these terms
revision: 4
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-09
measurements: []
requests: []
supersedes: []
---

# Glossary of shared terms

## Summary

The shared vocabulary of the specs. A spec may use any term below without defining it again. A term
used by one spec only stays in that spec's **Terms** section. Where a spec gives a term a narrower
meaning, the spec says so and wins for that spec.

## Conventions

- Entries are in alphabetical order, ignoring a leading "the".
- Units are named in each entry that has one. Numbers below are engine constants of the original
  or definitions of our own units (marked "Definition"), never values from stock data.
- "See S-xxxx" points to the spec that owns the full definition. Any statement in that entry about
  what the original does is a claim of the named spec, and that spec carries its backing,
  confidence and measurement scenario.
- Planned specs are specs that are not written yet. They are: S-0010 (theater and tileset
  semantics), S-0106 (voxel tilt on slopes), S-0109 (animation playback), S-0201 (game speed and
  tick rate), S-0202 (the random streams), S-0204 (coordinates and facing), S-0205 (number
  domains), S-0206 (turn rates) and S-0301 (the firing cycle). A spec cites one as "planned
  S-xxxx" or as "S-xxxx (its subject, planned)", for example "S-0205 (number domains, planned)".
  A planned spec owns no claim, so a pointer to one is a pointer only: in an entry with such a
  pointer, every statement about the original is a claim G1 to G3 or a claim of a written spec
  that the entry names, and the rest is vocabulary. At sign-off every S-number a spec cites must
  be either a draft in the same batch or a planned spec of this list.
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
  a claim about the original, so it needs no backing. Where an entry names a topic whose specs are
  not written yet (for example combat, economy or movement), its description is a pointer only: specs must not
  cite it as a rule.
- The few statements about the original that no spec owns yet are the numbered claims G1 to G3 in
  **Behaviour**, each with its backing, confidence and measurement scenario.

## Terms

**absent-key default.** The value a field takes when no data gives its key: the key is missing, or
present with an empty value. It is fixed by the original's program, so it is an engine constant and
specs may state it. When rules are layered, a missing key normally keeps the value already in force
instead (see *rules layer*). For a type section that is the value from an earlier layer of the same
game, or the absent-key default. For a game-wide key, and for a tiberium's key, it can be a value
that an earlier game of the session left behind (see *rules database*). See S-0003 and S-0006.

**aircraft.** A type kind and its objects: units that fly, drawn as voxel models or sprites (see
S-0007, claim 4, and S-0105). Infantry of a jumpjet type and hovering vehicles are not aircraft: an object's kind is the kind of
its type. Which kinds can take off and land, and where, belongs to the
aircraft specs.

**animated terrain object.** A terrain object (a type kind, such as a tree) of the group that S-0102
draws in the object pass instead of painting it in the cell pass. S-0102 says which terrain objects
are animated. Not the same thing as a *terrain animation*. See S-0101 and S-0102.

**archive.** A container file holding named members, in the public "MIX" format. An archive can be a
member of another archive (a *nested archive*). When a file is requested, a loose file of that name
takes precedence over any archive member, and among the mounted archives a ranking decides which
member is used. The ranking follows the order in which archives are mounted, so it changes during a
session (S-0005). See S-0005.

**armour class.** The category of protection a type belongs to, taken from the type's imported
armour field. A warhead scales its damage by a percentage chosen by the target's armour class. The
combat specs own the rules.

**art database.** The parsed contents of the expansion's art file: the settings of each art section,
which say how a type is drawn (sprite or voxel model, image-frame ranges, animation settings). A type
finds its art section mostly through its *image name*, and S-0007 gives the cases that use the type
name instead. *Image name* and *art section* are S-0007's terms: an image name names an art section,
not a sprite. See S-0007.

**backing.** How a claim is supported. One of: `measured` (an M-ID records an observation that
confirms it), `measurable` (a measurement scenario in the spec would confirm or falsify it) or
`unmeasured` (it cannot practically be observed, such as random-generator internals; each use must
say why).

**base rules.** The expansion's main rules file, read as the first rules layer at every game start.
It restores every key it gives. A game-wide key it does not give is not reset by it (see *rules
database*). See S-0004 and S-0006.

**battlefield view.** The part of the game screen that shows the map, without the sidebar and the
strip above it. See S-0101.

**bridge, high bridge, bridge deck.** A *bridge* is a crossing of water or a gap built from bridge
overlays. A *high bridge* is a bridge with a raised *bridge deck* that units can drive on and pass
under; a bridge without one is a *low bridge*. The deck lies 4 height levels above the terrain of
its cell (S-0102, claim 15; S-0104). Specs write "high bridge" and "bridge deck", not "raised bridge" or "raised bridge
deck". An object "on a bridge deck" stands or drives on the deck, as opposed to passing under it.

**building.** A type kind and its objects: a stationary object that occupies a foundation of cells.
Also called a structure.

**cell.** One square of the map grid, identified by integer cell coordinates (x, y). Increasing x
runs towards the lower right of the screen, increasing y towards the lower left. A cell is 256
leptons along each map axis, and its centre is at world position (256x + 128, 256y + 128) leptons.
Its corner topmost on the screen is its *top corner*. See S-0009 and S-0101.

**cell tag.** A trigger tag attached to one cell by the map. How a map attaches cell tags is
specified in S-0009 (claim 31). What an object entering the cell sets off is a pointer only: it
belongs to the trigger specs, which are not written yet.

**claim.** One numbered, observable and testable statement in a spec's Behaviour section. Every claim
carries a backing and a confidence.

**colour-adjustment table.** A table of 16 positions, each holding a tint colour with a red, a green
and a blue component. The rules data fills it, and the game combines a position's colour with an
object's pixels to tint it, for example a building under the invulnerability effect. A position that
holds zero (red, green and blue all 0) adds no tint. Rules layers change the table by entry position.
How it is reset and filled at each game start is owned by S-0006 (claims 2 and 12). See S-0006.

**confidence.** How sure the spec author is of a claim: high, medium or low. Low-confidence claims
should have an open question or a measurement scenario.

**construction state.** The state of a building while it plays its build-up sprite (see *sprite*):
forwards after it is placed (*build-up*), and backwards while it is sold (*selling*). Specs write
"construction state" for the state, and "build-up" or "selling" only for one direction of it, so a
rule that holds "outside the construction state" also holds while selling. S-0108 owns its timing.
See S-0108.

**country.** The nationality a house plays, chosen from the countries listed in the rules data. A
country belongs to one side (S-0004, claim 9). Which types a country may build is left to the
production specs.

**credits.** The unit of money: costs, refunds, harvested resources and a house's balance are given
in credits. Each economy spec states the number domain and rounding of the amounts it computes.

**departure.** A documented place where our engine deliberately does not reproduce the original,
and offers no quirk switch for it. The spec states the original's behaviour as behaviour and then
our engine's result. Departures cover two cases. Where the original crashes or gives an undefined
result on bad data (for example an unparseable number, or a list shorter than its fixed count),
our engine uses a safe, deterministic result, such as treating the value as absent and using the
default, and the importer warns (see *importer warning*). Where the original depends on the host
machine or on real time (for example the order in which the operating system lists files, or work
spread over ticks by a real-time budget), our engine uses a fixed, deterministic behaviour, such as
an order sorted by the bytes of the names. A spec may also make a design choice of scope a
departure, as S-0005 does for loose files (see *loose file*).

**detail level.** The player's visual-detail option, kept in the settings file: 0 = low, 1 =
medium, 2 = high. Specs write "detail level", not "detail setting". See S-0104.

**disguise.** A unit's ability to look like a unit of another type to some players. While a disguise
is in effect against a player, that player sees the unit drawn as the disguise type. Which units can
disguise, and against whom it is in effect, belongs to the special-abilities specs. See S-0107.

**display path.** Everything between a drawn frame that the game has finished and the picture on the
monitor: the graphics driver and its settings, any compatibility layer, and whether showing a drawn
frame waits for the monitor's refresh. Whether a display path that waits for the refresh also holds back
the tick rate depends on how drawn frames relate to ticks (see Open questions). The measurement
setup allows a presentation-only *display compatibility layer* (see *measurement setup*).

**drawn frame.** One redraw of the screen. Not the same thing as a tick: rendering specs say which of
the two each rule follows. Not the same thing as an *image frame* either. Specs should not write
"frame" or "game frame" on its own (see Open questions).

**engine add-on.** Anything that changes the original's program behaviour beyond what its data files
control: a patched executable, an injected library, a launcher that alters the game while it runs, or
a third-party extension of the rules language. Data-only changes are *mods*, not engine add-ons. A
presentation-only display compatibility layer is not an engine add-on either, although it may be an
injected library: it changes only how finished drawn frames reach the screen, never the game's logic
or the pixels it draws (see *measurement setup*).

**engine constant.** A number fixed inside the original's program rather than read from data, such as
the number of leptons per cell, a table size, a hard-coded multiplier or an absent-key default. Specs
may and should state engine constants, always with a unit.

**entry, key, value.** An entry is one key/value pair inside a section of a data file, written as a
line `key=value`. The key is the trimmed text before the first `=`, the value the trimmed text after
it. See S-0001.

**facing.** A direction, given as an integer *f* from 0 to 65535 that stands for *f*/65536 of a full
turn: one facing unit is 1/65536 of a turn. 0 is *map north* and values grow clockwise as seen on the
screen, so 16384 is *map east*, 32768 *map south* and 49152 *map west*. Map north is the direction of
decreasing cell y (towards the upper right of the screen), map east the direction of increasing cell
x (lower right), map south increasing y (lower left) and map west decreasing x (upper left).
Compound names follow: map north-west is straight up the screen. Arithmetic on facings wraps modulo
65536. Coarser facings (8, 16 or 32 steps of a turn) are derived from a facing, and each spec that
uses one gives the derivation and its rounding. The 65536 units per turn are our own
unit, a definition. That the original uses this same zero and sense is claim G1. The facing spec (planned S-0204) will own the full
definition.

**field.** One setting of a type, or of a game-wide section, that the original reads from a value.
Each field has a value form (integer, percent, boolean, list and so on, see S-0003) and an absent-key
default. Specs refer to stock numbers only through the field: "the weapon's rate-of-fire field".

**fog of war.** An optional game setting. With it on, map areas a house has explored but cannot see at
the moment keep showing what the house last saw there; S-0102 says which objects appear there as
*remembered images* and how they are drawn. With it off, explored areas show what is there now.
Which objects stay hidden under fog, and when an area counts as explored or seen, belongs to the
visibility specs. *Shroud* is a separate cover.

**foundation.** The rectangle of W × H cells a building occupies, W along the map x axis and H along
y. Its top-left cell is the foundation cell with the smallest x and y. "Top-left" refers to the map
axes: on the screen that cell is the topmost cell of the footprint, not the leftmost. The
foundation's *top corner* is the top corner of its top-left cell, the topmost point of the footprint
on the screen.

**game.** One match, from the moment the map is loaded until it ends. Several games can run in one
session.

**game minute.** The unit in which some data fields give durations. The original converts a game
minute to ticks at a fixed 900 ticks per game minute, an engine constant; the specs that convert a
minutes field state it (S-0107, S-0108). Other specs may give values in game minutes without
converting them, as S-0006 does for some reset values. A game minute is a conversion unit, not a minute of
real time: how many ticks run per second of real time at each game speed belongs to planned S-0201.
Specs write "900 ticks per game minute", not a "nominal" tick rate.

**game-mode file.** The rules file that belongs to the selected multiplayer or skirmish game mode. It
is read as a rules layer after the base rules and the language rules file, and before the map layer.
A mode may have none, and a campaign mission has no game-mode layer. In a game whose language rules
file fails its check, the game-mode layer is skipped (see *language rules file*). See S-0006.

**game speed.** The player's speed setting, as an integer index *s*. 0 is the fastest and larger
values are slower (claim G2). The skirmish lobby offers 0 to 6, and the claims here cover 0 to 6.
The settings file can hold other values (Import mapping), and no claim here covers them. The
game-speed normalisation table of S-0107 (Constants, T3) has a column for index 7, which lies
outside the covered range. In a skirmish game, game speed changes how many ticks run per second
of real time (claim G2). It also changes the length in ticks of the speed-normalised delays, and of
whatever waits on them, but no other timing of game state counted in ticks (claim G3). Our engine
reproduces that dependence by default (`quirks.speed_normalised_delays`, see claim G3). The mapping
from *s* to ticks per second, and how game speed works in campaign missions and network games,
belong to the tick-rate spec (planned S-0201).

**game start.** The moment a new game (a skirmish, a multiplayer game or a campaign mission) begins
loading. The rules layers are applied at every game start, and every type registry except the
tiberiums' is rebuilt then (S-0004, claim 1; S-0006, claims 1 and 10). Specs write "game start", not "scenario start" ("scenario" would
clash with *measurement scenario*).

**game-wide section.** A rules section that is neither a type's own section nor a type list: the
general settings, the combat-damage settings, the AI settings, the difficulty sections, the land-type
sections and the like. See S-0004 and S-0006.

**global effect.** One of the map-wide lighting effects of S-0104: the *nuke flash*, the *chrono
flash*, an *ion storm* or the *psychic dominator*. Each name, and the phases of each, means what
S-0104 defines. In particular, the chrono flash is not the chronosphere superweapon's jump (S-0104).
See S-0103 and S-0104.

**height above the surface, height above the terrain.** Two heights of an object, both in leptons,
that differ only for an object on a bridge deck. Its *height above the surface* is its world height
minus the height of the surface it is on or over: the bridge deck for an object on a bridge deck,
otherwise the terrain of its cell, so an object standing on a bridge deck has height 0 above the
surface (S-0104, S-0107). Its *height above the terrain* is its world height minus the world height
of the terrain of the cell it is over. Over a high bridge's cell that terrain is the ground under the
deck, never the deck (S-0102). Specs write one of these two, never "height above (the) ground" on
its own.

**height level.** A cell's terrain elevation, an integer. A map file may give any level from −128 to
127 (S-0009, claim 7); stock maps use 0 and above (S-0009, Terms). One height level is 104 leptons of world height. See
S-0101.

**height lift.** The number of pixels by which a point at world height *z* is drawn above the
projection of its ground point, written *L(z)*. See S-0101.

**house.** One owner of objects in a game: it has credits, power, a colour, a country and an
alliance state with every other house. Each player, human or computer, is one house. A skirmish game
also always creates two non-player houses: the *neutral house* and the *special house*, which own
neutral map objects. The program creates them itself and takes their countries from the rules data.
S-0009 gives how those countries are found, the order in which all houses are created and what that
order decides.

**house colour.** The colour that marks a house's objects. It replaces the palette's remap range
(indices 16 to 31) with a ramp built from the house's colour definition. See S-0103.

**human-controlled.** A house played by a person rather than by the computer. A rule that depends on
this says "human-controlled owner" or "computer-controlled owner".

**image frame.** One picture in a sprite's shape file, numbered from 0. Not a drawn frame.

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

**infantry.** A type kind and its objects: soldiers and other units drawn as sprites and moving on
foot. See S-0107.

**jumpjet cruise height.** A jumpjet type's imported height field: the height above the terrain
at which its jumpjets fly, in leptons. It is also the hover height that S-0101 uses. S-0102 gives its minimum, its
absent-key default and its role as the boundary between the air and top render layers. Specs write
"jumpjet cruise height", not "hover-height field". See S-0102.

**jumpjet type, jumpjet.** A jumpjet type is an infantry or vehicle type with the imported jumpjet
flag, which flies with a jump-pack style locomotor. A *jumpjet* is a unit of a jumpjet type that is
currently moving with that jumpjet locomotor. Specs say which of the two a rule needs. See S-0107.

**language rules file.** An optional rules file of the original. When a file of that name can be
resolved and is valid, it is applied as a rules layer after the base rules and before the
game-mode file. A file that holds at least one section is valid only if it carries a matching
integrity digest. The file is checked at program start and again at every game start, and what
happens to one that fails depends on when it fails (S-0006, claim 12). Failing at program start, it
makes the program's start-up fail. Failing at a later game start, whether it first appeared after
program start or was valid then and has since been edited or replaced, it is ignored for that game,
and so is that game's game-mode layer. Stock data has none. See S-0005 and S-0006.

**layer.** Ambiguous on its own. Write *rules layer* for a rules file applied on top of earlier ones,
or *render layer* for one of the ordered drawing lists.

**legal cell.** A cell that exists on a given map. Which cells are legal follows from the map size
(S-0009, claim 1). See S-0009.

**lepton.** The unit of world distance: 1/256 of a cell edge. Positions, ranges and speeds are given
in leptons (speeds in leptons per tick). World height is also measured in leptons.

**lobby.** The pre-game screen of a skirmish or multiplayer game, where the players, their countries,
colours, teams and start positions and the game mode are chosen.

**local area.** The screen-aligned sub-rectangle of a map in which play takes place and to which the
view's scrolling is limited. It is given by four integers (left, top, width, height) in the same units
as the *map size*: horizontal numbers count cell widths (60 pixels on screen), vertical numbers count
cell heights (30 pixels on screen). See S-0009 and S-0101.

**local player.** The house of the person in front of this machine. In our engine only what is drawn
or heard may depend on it, never game state; each spec that uses it says what depends on it. See
S-0107.

**locomotor.** The movement method of a mobile type, chosen by an imported field: for example driving,
walking, hovering, flying or jumping. It decides how the object moves between cells and what
movement states it reports. The movement specs own its details.

**loose file.** A file stored directly in the game directory, outside any archive. A loose file is
found before an archive member of the same name, so a loose file can override shipped data. Our
engine looks for loose files in the game directory only. The original also looks in a few legacy
places outside it; S-0005 claim 4 makes not doing so a *departure*, with no switch. See S-0005.

**map layer.** The map file read as the last rules layer of a game: any rules section it contains
overrides the earlier layers. What it sets in type sections lasts for that game only, except in
tiberiums' sections: what it sets there, and any tiberium it adds, lasts for the rest of the session
(S-0006, claim 10). What it sets in game-wide sections can outlast the game too (see *rules
database*). See S-0006.

**map size.** The screen-aligned rectangle that bounds a map, given by its map width and map height:
the width counts cell widths (60 pixels on screen) along a screen row, the height counts cell heights
(30 pixels on screen). See S-0009 and S-0101.

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
fixes them. Three further allowances apply:
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
  names them. A claim that was unmeasured only because it needed such a file is therefore
  measurable.
- *Statistical runs.* Nothing is placed inside the original to fix its random state. A sub-test
  whose result depends on the random seed is measured statistically instead, over many runs, and
  its scenario states the number of runs and the statistic that confirms or falsifies the claim.

**mod.** A change to the data files only: rules, art, maps, images or sounds. Our engine supports
mods through the importer. A mod is never part of stock data or of a measurement setup, except the
named test files a scenario uses (see *measurement setup*).

**movement zone.** A mobile type's imported category of where it may move, such as on foot, by
wheel or track, by water or amphibiously. One value, "amphibious-destroyer", also makes infantry
use their water sequences (S-0107). The movement specs own the full meaning.

**object.** Anything placed on the map that the game tracks individually: infantry, vehicles,
aircraft, buildings, overlays, terrain objects, smudges, animations, projectiles and so on. Every
object is an instance of one type.

**oracle.** The original, used as the reference that decides what correct behaviour is. Measurements
are taken from the oracle.

**the original.** The reference game: *Command & Conquer: Yuri's Revenge* version 1.001 as shipped,
meaning the reference executable together with its stock data. "Original behaviour" is what this
release does, defects included. "YR behaviour" means the same.

**overlay.** A cell-level map object kind that is not terrain and not a building: walls, fences,
bridge parts, crates, ore and gem fields and similar. Each overlay cell has an overlay type and an
overlay data value. See S-0009.

**overlay data value.** An integer from 0 to 255 that every legal cell has, alongside its overlay
type (S-0009, claim 13). Its meaning depends on the overlay type: for
example the amount of ore in an ore cell or the damage stage of a wall. Cells without an overlay
still have one. See S-0009.

**override section.** A rules section inside a game-mode file or a map that changes the rules from
that game on. It has the same name as the section it overrides. An override of a type section lasts
for that game only, except an override of a tiberium's section, which lasts for the rest of the
session (S-0006, claim 10). An override of a game-wide section can outlast the game too (see *rules
database*). See S-0006.

**palette.** 256 colour entries. Once loaded, each entry is a colour with a red, a green and a blue
component, each an integer from 0 to 255; S-0103 says which of those values loading can produce.
Sprites and voxel models store palette indices, not colours. Each theater has its own unit, tile
and theater palettes; the anim palette is the same in every theater. See S-0103.

**per-mille (‰).** An integer ratio where 1000 means a factor of 1.0.

**prerequisite.** A type that a house must own (usually a building) before another type can be built.
Which types are prerequisites of which comes from imported fields.

**quirk switch.** A named setting of our engine, written `quirks.<name>`, that controls one unusual or
defective behaviour of the original. Its default is always the original behaviour. The spec that
names a switch describes both the original behaviour and the alternative ("fixed") behaviour that the
switch selects when turned off. Where the original crashes, gives an undefined result or depends on
the host machine or on real time, there is no switch: that is a *departure*. A switch shared by
several specs is named once in each of them; the glossary points to the shared ones (Edge cases and
quirks).

**recorder.** Whatever a measurement uses to observe the original without changing it: a stopwatch, a
screen or video capture, or a program that only reads the running game's state, such as its count
of ticks run. A recorder never writes into the game, injects code, changes its timing, fixes its
random state or gives orders. It is therefore not an engine add-on, and the measurement setup
allows it.

**registry, registration index.** The registry of a type kind is the ordered collection of all types
of that kind in the current game. A type's registration index is its position there, a non-negative
integer counted from 0. Indices are dense: the *n*-th type created for a kind has index *n* − 1.
See S-0004.

**render layer.** One of the five ordered lists in which objects are drawn: underground, surface,
ground, air and top. See S-0102.

**reset on override.** A property of a few rules keys: when a rules layer contains the key's
section but omits the key, the value is set to a fixed reset value instead of keeping the value in
force. The reset value is a fixed engine default, or the current value of another field. For some
keys the reset applies in the base rules too, not only in later layers. See S-0006 (claim 5) and
S-0105 (claim 27).

**resolve.** To find the single copy of a requested data file that the game uses: a loose file of
that name if one exists, otherwise the archive member that the archive precedence of
S-0005 selects. A name that
resolves to nothing is "not found". See S-0005.

**round-toward-zero mode.** The floating-point rounding mode under which the original rounds
floating-point results toward zero. S-0003, S-0105, S-0107 and S-0108 state it for the results they
cover. The mode's full scope will be owned by S-0205 (number domains, planned), together with its
single switch `quirks.chop_rounding` (default: the original behaviour). A spec whose results depend on
the mode points to S-0205 (number domains, planned) and defines no switch of its own.

**rules database.** The parsed rules data in force during a game. At every game start the base
rules, the language rules file (if a valid one exists), the game-mode file and the map layer are
applied, in that order, on top of the rules state already there. Every type registry except the
tiberiums' starts each game empty, so type sections hold only what this game's layers gave.
Game-wide sections are not cleared at a game start, apart from the resets S-0006 lists (for
example, the colour-adjustment table depends only on this game's layers, as the table in S-0006
claim 2 gives, never on earlier games). So a game-wide key that the base
rules do not give keeps, in later games of the same session, the value an earlier game's game-mode
file or map gave it. Tiberiums carry over between games in a similar way (S-0006, claim 10). A fresh
program start clears both. See S-0005 and S-0006.

**rules layer.** One rules file applied to the current rules state. The layers, from lowest to
highest precedence, are the base rules, the language rules file (if a valid one exists), the
game-mode file (multiplayer and skirmish games only, and see *language rules file*) and the map
layer. A later layer changes only what it mentions, apart from the reset cases its spec lists. See
S-0006.

**scatter.** A unit's short move away from its spot, made to dodge a threat or to make room. Where
it moves belongs to the movement specs. See S-0107, which for now also holds the infantry behaviour
it needs; that part may later move to the infantry specs.

**scenario random stream.** The specs' name for the original's pseudo-random number generator whose
draws decide game state, as opposed to the *unsynchronised random stream*. A *draw* takes one number
from it, a *ranged draw* one number reduced to an inclusive range. Our engine must give every machine
of a multiplayer game the same draws in the same order, which is why specs state how many draws
happen and in which order. Its algorithm and seeding will be owned by planned S-0202; S-0009 and
S-0107 state the draws they take from it.

**section.** A named group of entries in a data file, introduced by a header line `[name]`. See
S-0001.

**session.** One run of the program, from start to exit.

**settings file.** The user's options file in the game directory that holds the player's own
settings, such as the game speed (see Import mapping) and the difficulty. It is a user file, not
stock data, so a scenario may set its values.

**shroud.** The cover that hides map areas a house has not explored yet. When it is removed belongs
to the visibility specs.

**side.** One of the faction groups, listed in the rules data, that countries belong to (S-0004,
claim 9). The local player's side selects the side group of archives, whose art gives the sidebar
its look (S-0005, claim 13). Which types a side's houses may build belongs to the production specs.
See S-0004 and S-0005.

**sidebar.** The panel beside the battlefield view during play, with the radar, the local
player's credits and the build icons. The order of the build icons is owned for now by S-0004
(claim 11) and may later move to the interface specs, which are not written yet and will own the
rest of its behaviour.

**skirmish.** A game against computer opponents, set up in the game's lobby screen with chosen houses,
start positions and a game mode.

**slope.** A cell whose ground is inclined rather than flat. Its slope shape comes from its terrain
tile. Slopes affect height within the cell, drawing and movement. See S-0009 and planned S-0010.

**sprite.** A two-dimensional image stored in a shape file in the public "SHP" format: a numbered
sequence of image frames, each a rectangle of palette indices. Infantry, buildings, animations and
some vehicles and aircraft are drawn as sprites. A sprite that a type's art names for one role is
called by that role: for a building the *body sprite*, *build-up sprite*, *bib sprite*, *deploying
sprite*, *roof-deploying sprite*, *under-door sprite* and *under-roof-door sprite*; for infantry the
*body sprite*. Specs don't call them "sheets" or "images" (an *image name*, see *art database*, names
an art section, not a sprite), and a numbered picture inside one is an
*image frame*, never a bare "frame". See S-0007, S-0107 and S-0108.

**stock data.** The unmodified data files of the release of the original: the shipped archives and
the rules, art, AI, sound, game-mode and map files inside or beside them. Specs never copy values from
stock data. They refer to the field instead, and a value in an example is invented.

**sub-tile.** A terrain tile can cover several cells. A cell's sub-tile is the index of the part of
its tile that lies on that cell. See S-0009.

**terrain animation.** An animation that the game creates as part of a cell's terrain when the
map's terrain is set up, such as the animation of a tile that the theater animates, rather than one
created by an object or an event. Not the same thing as an *animated terrain object*, which is a
terrain object. See S-0103 and S-0104.

**theater.** The terrain set of a map, one of six. It selects the terrain tiles and palettes, and it
changes the names of some art files. See S-0007, S-0103 and planned S-0010.

**tick.** One simulation step: the unit of game time. All timings of game state (delays, cooldowns,
animation stages, build times) are counted in ticks. In a skirmish game, game speed changes how many
ticks run per second of real time (claim G2). It leaves the length in ticks of every timing of game
state unchanged, except for the speed-normalised delays and whatever waits on them (claim G3). The
tick-rate spec (planned S-0201) will own the full definition.

**tileset.** A named group of consecutive terrain tiles declared in a theater's data, such as one
cliff set or one road set. A theater's tiles are numbered consecutively across its tilesets in
declaration order. See S-0009.

**top corner.** Of a cell (*x*, *y*): its corner at world position (256*x*, 256*y*) leptons, the
corner with the smallest x and y. It is the cell's map north-west corner and its topmost point on
the screen. Of a foundation: the top corner of its top-left cell, which is the topmost point of the
footprint on the screen. Specs write "top corner", not "north-west corner", for this point.

**trigger, trigger action.** A *trigger* is a rule that a map defines: when its conditions are met,
it runs its *trigger actions*, such as playing an animation or changing the lighting. Triggers belong
to the trigger specs, which are not written yet, so specs mention them only as pointers. Specs write
"trigger action", not "script action".

**type.** One kind of thing defined by the rules data, such as one infantry, one weapon or one
warhead, with its own section and fields. Objects in play are instances of types.

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

**unsynchronised random stream.** The specs' name for the original's second pseudo-random number
generator, distinct from the *scenario random stream*. S-0107 (claim 14) names one draw from it: the
idle comment chance of a unit owned by the local player. A spec that names a draw from this stream
also says what the draw affects. Which other uses draw from it, whether any of its draws reach game
state, and its algorithm will be owned by planned S-0202 (Open questions). How our engine handles
the stream across the machines of a multiplayer game will be settled with that spec.

**vehicle.** A type kind and its objects: wheeled, tracked, hovering and naval units, drawn as voxel
models or sprites (see S-0007, claim 4).

**voxel model.** A volumetric model in the public "VXL" format, with a companion motion file in the
public "HVA" format of the same base name. See S-0105.

**warhead.** The type that decides what a hit does: its damage scaling per armour class, its area
spread and its side effects. A weapon's warhead is one of the weapon's fields. The combat specs own
the rules.

**weapon.** The type that describes one way of attacking: damage, range, delay between shots,
projectile and warhead are all fields of it.

**world height.** The height of a point in leptons above height level 0, not above the local ground.
See S-0101.

**world pixel.** An integer pixel position (*q<sub>x</sub>*, *q<sub>y</sub>*) in an unbounded plane
that contains the whole map, *q<sub>x</sub>* growing to the right and *q<sub>y</sub>* downwards. The
battlefield view shows a window of it. See S-0101.

**YR.** Short for *Yuri's Revenge*, the expansion of *Command & Conquer: Red Alert 2*. In the specs,
"YR" and "YR behaviour" refer to the original.

## Behaviour

These are the only claims the glossary makes itself. Every other statement about the original in an
entry above belongs to the spec its "See S-xxxx" pointer names (see Conventions).

G1. **Direction convention.** Directions in the original have the zero and the sense of *facing*:
    0 points to map north (decreasing cell y, upper right of the screen), and directions grow
    clockwise as seen on the screen, so a quarter turn points to map east (increasing x), a half
    turn to map south (increasing y) and three quarters of a turn to map west (decreasing x). A
    vehicle pre-placed by a map starts facing *v*/256 of a turn, where *v* is the map's facing value
    (an integer from 0 to 255), that is, *v* × 256 facing units. The 65536 facing units of a full
    turn are our own unit (see *facing*); G1 says nothing about how finely the original stores a
    direction in general.
    - Backing: `measurable` (Scenario G1) for the zero, the sense and the map value's scale to
      within the 1/32 turn that a drawn vehicle body shows (S-0105, claim 3). The step of the map
      value finer than that, down to 1/256 turn, is `unmeasured`: the body is drawn in 32 directions
      only, so a screenshot cannot show it. The only finer observable known, the number of ticks a
      first turn takes, depends on a turn rule that no spec owns yet (planned S-0206), so it cannot
      be measured against a stated expectation until that spec exists (Open questions).
    - Confidence: high for the zero and the sense; medium for the scale; medium for the 1/256-turn
      step, which Scenario G1 does not test

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
    - Backing: `measurable` (Scenario G2) for parts (a), (b) and (c). Part (b′) is `unmeasured`:
      whether index 0 adds a wait is internal to the original, and its one observable consequence,
      *N*₀ > *N*₁ in Scenario G2, depends on how much faster than index 1 the machine and display
      path could run, which no observable of the scenario establishes independently. A run set
      with *N*₀ > *N*₁ is consistent with (b′), but one without it cannot falsify (b′).
    - Confidence: high for parts (a) and (b); medium for parts (b′) and (c)

G3. **Game speed and timings counted in ticks (skirmish).** In a skirmish game the game speed
    changes how much real time passes between ticks (claim G2). Every timing of game state counted
    in ticks takes the same number of ticks at every game speed, with one exception: the
    speed-normalised delays and whatever waits on them. For example, a production time and a
    movement over a fixed route take the same number of ticks at every game speed.

    **The exception.** The original converts some animation stage delays with the game-speed
    normalisation N(*d*, *s*) of S-0107 (Formulas F3), so that they last about the same real time
    at every game speed. Their length in ticks therefore depends on *s*, on purpose. Which delays
    are converted, and when, is owned by the specs named here; G3 only points to them:
    - the infantry sequence kinds that S-0107 claim 5 lists (*idle-a*, *idle-b*, *wet-idle-a*,
      *wet-idle-b*, *hover* and *cheer*),
    - the building body stages that S-0108 claim 6 converts, including the wrap of a body range,
    - animations whose type is marked as normalised (animation playback, planned S-0109).

    G3 makes no claim about any other delay that may be converted the same way (Open questions).

    A rule that waits for one of these stages inherits the dependence, and that can reach game
    state, not only what is drawn. The specs that own these rules say how. For example, by the
    claims of S-0107 an idle action is attempted only while *stand*, *guard* or *tread* is current, so the length of an idle
    sequence changes which idle-timer expiries lead to an attempt, and with them the number of
    draws from the scenario random stream (S-0107, claim 14). A *cheer* cannot be interrupted, and a
    shot leaves when the current sequence's stage reaches the fire frame (defined in S-0107, claim
    11), so the shot timing of a
    cheering unit can depend on the game speed (S-0107, claims 6 and 11). A building's construction
    timing follows its body stage (S-0108, claims 5 and 6). An animation's end can set off effects
    (planned S-0109). G3 makes no claim about any of these. It makes no claim either that the
    exception is only visual.
    - Backing: `measurable` (Scenario G3)
    - Confidence: medium (no other use of the game-speed index by a game rule is known; no timer of
      the original that counts real time during a game is known to reach game state, but not every
      one has been traced. One that is known spreads a relighting of map cells over several ticks by a
      real-time budget (S-0104, claim 12); our engine replaces it with a relighting that completes
      within the tick it is requested, a departure of S-0104)

    **Our engine.** It reproduces the exception by default. The switch
    `quirks.speed_normalised_delays`, shared by S-0107 and S-0108, selects it (default: the original
    behaviour); those two specs say what each converted delay becomes with the switch off.

## Formulas

> facing angle = *f* / 65536 of a full turn, clockwise on the screen from map north. *f* is an
> integer from 0 to 65535; every sum or difference of facings is reduced modulo 65536 to that range.
> Exact integer arithmetic, so no rounding applies. Unit: facing units. (Definition.)

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
| Map facing value step | 256 | facing units per step | 256 steps per turn; claim G1 |
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

None of its own. The orders the entries mention (loose files before archive members, rules-layer
precedence, the order in which houses are created) are stated as observable consequences in the
specs they point to: S-0005, S-0006 and S-0009.

## Edge cases and quirks

The glossary owns no quirk; *quirk switch* defines the notation the specs use. It points to two
switches that several specs share:
- `quirks.speed_normalised_delays`: the speed-normalised delays of claim G3, owned by S-0107 and
  S-0108.
- `quirks.chop_rounding`: the *round-toward-zero mode*, which will be owned by S-0205 (number
  domains, planned).

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| *Settings file*, section `[Options]`, key `GameSpeed` | game-speed index *s* (claims G2 and G3) | integer; the claims cover 0 to 6, and no other value is covered here (7 has a column in S-0107's table T3) | no claim here: the absent case belongs to planned S-0201; every scenario sets the key |
| Map, pre-placed vehicle section `[Units]`, the sixth comma-separated value of an entry | the vehicle's starting facing, as *v* (claim G1) | integer; the claim covers 0 to 255, other values are not covered | each scenario writes it |

Every other key belongs to the import mapping of the spec that owns its concept.

## Measurement scenarios

All three run under the *measurement setup*: an unmodified install with no loose override files and
no engine add-ons. None of them needs test data beyond its test map. Observations are made with a
*recorder*. Each scenario records the display path it used, naming any display compatibility layer
generically (see *measurement setup*).

**Scenario G1: direction convention (claim G1).**
- *Setup:* a purpose-made flat map of one theater, no hills, no overlays near the test area. A
  skirmish with one human player and one computer player whose start position is on the far side of
  the map. The map pre-places seven vehicles of one stock vehicle type that is drawn as a voxel model
  and whose body has a visible front. They are owned by the human player (the map names their owner
  as S-0009, claim 27, requires), each stands on its own cell at least three cells from the others,
  and their facing values are 0, 8, 16, 32, 64, 128 and 192. Any game speed.
- *Procedure:* start the game, give no orders and pause on the first tick the map is visible.
  Take a screenshot.
- *Observable:* for each vehicle, the direction its front points on the screen, read against the
  cell grid: the neighbouring cell it points to, or the two neighbours it points between.
- *Runs:* 1 (no random draw is involved).
- *Expected:* value 0 points to cell (*x*, *y* − 1), upper right on the screen; 32 to
  (*x* + 1, *y* − 1), straight to the right; 64 to (*x* + 1, *y*), lower right; 128 to
  (*x*, *y* + 1), lower left; 192 to (*x* − 1, *y*), upper left. Value 16 points halfway between
  the directions of values 0 and 32. Value 8 points between the directions of values 0 and 16. All
  seven are drawn in visibly different directions.
- *Falsified if:* value 0, 32, 64, 128 or 192 points to a different neighbour, or value 8 or 16 is
  drawn the same as a neighbouring value of the list or points outside the angle between its
  neighbours. If all are rotated by the same quarter or half turn, the zero is wrong; if 64 and 192
  are swapped, the sense is wrong; if 8 looks the same as 0 or 16, the map value moves the drawn
  body in steps coarser than 1/32 turn, against the scale of G1.
- *What it cannot show:* a vehicle body is drawn in 32 directions (S-0105, claim 3), so values that
  differ by less than 8 can be drawn alike whatever the step. The scenario therefore does not test
  the 1/256-turn step of claim G1.

**Scenario G2: game-speed direction (claim G2).**
- *Setup:* a small purpose-made or stock skirmish map, one human player and one computer player.
  Before each run the settings file's game-speed index is set to one of 0, 1, 2, 3, 4, 5 and 6. A
  recorder that reads the number of ticks run. A display path that does not wait for the monitor's
  refresh, because one that waits can hold index 0 down to the rate of index 1. If the game's own
  display path waits for it, a display compatibility layer that does not wait is used, and the
  record names it generically.
- *Procedure:* start the game and let it run with no orders for 60 seconds of real time after the
  map is visible.
- *Observable:* *N*ₛ, the number of ticks run in those 60 seconds at index *s*, the mean of the runs
  at that index.
- *Runs:* 2 per index. A second set of runs, 2 per position, sets the speed through the skirmish
  lobby's speed control at each of its seven positions instead of through the settings file, and
  records the counts the same way.
- *Expected:* *N*₀ ≥ 0.98 × *N*₁ (part b; 2 % allows for timing noise). In a run set with
  *N*₀ > 1.02 × *N*₁, which shows that the machine can compute and show ticks faster than index 1
  allows and so establishes the condition of part (a), *N*ₛ < 0.98 × *N*ₛ₋₁ for every *s* from 2
  to 6. The lobby positions, from the slowest to the fastest, give counts within 2 % of *N*₆, *N*₅,
  …, *N*₀ in that order.
- *Falsified if:* *N*₀ < 0.98 × *N*₁ (part b); in a run set with *N*₀ > 1.02 × *N*₁,
  *N*ₛ ≥ *N*ₛ₋₁ for some *s* from 2 to 6, equal counts included (part a); or a lobby position's
  count is not within 2 % of the index it is expected to match (part c). No result falsifies part
  (b′), which is `unmeasured` (claim G2).
- *Resolution:* the scenario resolves differences of 2 % or more. A run set with
  *N*₀ ≥ 0.98 × *N*₁ confirms part (b). A run set in which *N*₀ is not above 1.02 × *N*₁ cannot
  test part (a), because the machine may have limited the rate; part (a) is judged only on a run
  set, on this or a faster machine or display path, in which it is. For part (a), a count with
  0.98 × *N*ₛ₋₁ ≤ *N*ₛ < *N*ₛ₋₁ neither confirms nor falsifies the claim; that pair of indices is
  run again with 5 runs per index and longer runs of 300 seconds. If the re-run lands in the band
  again, that pair is recorded as inconclusive at this resolution.

**Scenario G3: timings counted in ticks across game speeds (claim G3).** It tests the part of
G3 outside the exception: a production time and a movement. The exception is tested by the
scenarios of S-0107 (M3) and S-0108 (B11).
- *Setup:* a purpose-made map with open flat ground. A skirmish with one human player who has a
  construction yard (a building that produces buildings), enough credits for one building and one vehicle standing at the start of a
  straight, free route 10 cells long; one computer player whose start position is on the far side of
  the map. Game speeds 1 and 5.
- *Procedure:* in each run, order one building of one stock type (the same type in every run), and
  order the vehicle to the cell at the end of the route. The orders may be given at any time; no
  exact tick is needed.
- *Observable:* *D*ᵇ, the number of ticks from the first tick on which the building's production
  shows progress to the tick on which it becomes ready to place; *D*ᵐ, the number of ticks from the
  first tick on which the vehicle moves to the tick on which it reaches its destination cell.
- *Runs:* 3 per speed.
- *Expected:* the same *D*ᵇ and the same *D*ᵐ in every run at both speeds. If the runs at one speed
  differ among themselves, the range of values at speed 1 and the range at speed 5 overlap.
- *Falsified if:* for either duration, every value at speed 5 is below every value at speed 1, or
  every value at speed 5 is above every value at speed 1.

## Open questions

- Whether any draw from the *unsynchronised random stream* reaches game state in the original,
  and which uses draw from it besides the one S-0107 names, is not settled. Planned S-0202 should
  answer it. Until then no spec may assume that a draw from that stream leaves game state alone.
- Whether exactly one drawn frame is produced per tick, at every game speed, is not settled yet. The
  tick-rate spec should answer it. Until then, rendering specs say explicitly whether a rule follows
  ticks or drawn frames.
- *Facing*, *game speed*, *tick* and *game minute* are owned in full by specs not yet written
  (planned S-0201 and S-0204), and *slope* and *theater* partly by planned S-0010. The definitions
  here must be checked against those specs when they arrive. How game speed works in campaign
  missions and in network games is left to planned S-0201; claims G2 and G3 cover skirmish games
  only.
- How far the speed-normalised delays of claim G3 reach game state is not settled in full. S-0107
  and S-0108 own their own cases. Whether an animation marked as normalised can change game state
  through its end, or through effects tied to its stages, belongs to planned S-0109.
- Two further delays are believed to be converted with the same game-speed normalisation, but no
  spec owns them and claim G3 does not cover them: the shortest time between two repeats of a
  spoken warning to a house (for example the warning that a house cannot pay for what it is
  building), and the period of one pulsing marker drawn over objects. Both are believed to affect
  only what is heard or drawn, not game state. If either reached game state, it would fall under
  the exception of claim G3, and the spec that comes to own it would say how. Their exact rule, and which warnings and markers they cover, should be
  settled by the interface and rendering specs.
- Whether the original keeps the full 1/256-turn step of a map's facing value (claim G1) is not
  tested: a drawn vehicle body shows only 32 directions. An observable that depends on the starting
  facing more finely, such as the number of ticks a first turn takes, could settle it once the
  turn-rate spec (planned S-0206) is written.
