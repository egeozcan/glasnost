---
id: glossary
title: Glossary of shared terms
topic: rules-import        # cross-cutting; every topic uses these terms
revision: 7
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
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
- A planned spec is a spec that is neither approved nor a draft of this glossary's batch. It may
  not be written yet, or it may exist as a draft of a later batch; either way it is not signed off
  with this batch. The only one this glossary cites is S-0301 (the firing cycle). A spec cites a
  planned spec as "planned S-xxxx" or as "S-xxxx (its subject, planned)", for example "S-0301 (the firing cycle, planned)". A planned spec owns no claim, so a pointer to
  one is a pointer only: in an entry with such a pointer, every statement about the original is a
  claim G1 to G3 or a claim of a written spec that the entry names, and the rest is vocabulary. At
  sign-off every S-number a spec cites must be an approved spec listed in `spec/README.md`, a draft
  in the same batch or a planned spec of this list. A draft in the batch is cited by its number and
  claim like an approved spec, never as planned, because calling it planned would take away the
  claims it owns.
- A pointer to a planned spec that has since been approved reads as a plain pointer. The same holds
  for a pointer that calls a draft of the same batch planned. The word "planned" is then ignored,
  the spec pointed to owns its claims as usual, and the next revision of the citing text drops the
  word. This covers every spec of the first two batches: S-0010, S-0106, S-0109, S-0201, S-0202,
  S-0204, S-0205 and S-0206, which earlier revisions of this glossary listed as planned, and every
  other spec that is approved or a draft of this batch. A draft of a later batch is still planned.
- This revision of the glossary cites, besides planned S-0301, the specs of its batch as they
  stand in it: the drafts S-0010, S-0011, S-0106, S-0109 to S-0111 and S-0201 to S-0215; the
  revisions of approved specs made in the batch, which are S-0002 revision 6, S-0003 revision 8,
  S-0004 revision 7, S-0005 revision 4, S-0006 revision 6, S-0007 revision 6, S-0009 revision 7,
  S-0104 revision 8, S-0105 revision 4 and S-0107 revision 4 (see *scatter*); and every other approved
  spec of the index in `spec/README.md` at its current revision. It is signed off in the same
  batch as those drafts and revisions.
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
  a claim about the original, so it needs no backing. Where an entry names a topic that has no
  approved spec and no draft in this batch (for example combat, economy or production), its
  description is a pointer only: specs must not cite it as a rule.
- The few statements about the original that the glossary makes itself are the numbered claims G1
  to G3 in **Behaviour**, each with its backing, confidence and measurement scenario. They were
  written before any spec covered them. Where a written spec now states the same behaviour in more
  detail (S-0204 for G1, S-0201 for G2, and for G3 the specs its exceptions name), that spec cites
  the glossary claim, the two agree, and the spec's claim adds the detail. The glossary claim stays
  because other specs cite it; a later revision that finds the two in disagreement corrects both.

## Terms

**absent-key default.** The value a field takes when no data gives its key: the key is missing, or
present with an empty value. It is fixed by the original's program, so it is an engine constant and
specs may state it. When rules are layered, a missing key normally keeps the value already in force
instead (see *rules layer*). For a type section that is the value from an earlier layer of the same
game, or the absent-key default. For a game-wide key, and for a tiberium's key, it can be a value
that an earlier game of the session left behind (see *rules database*). See S-0003 and S-0006.

**AI house.** A house of our engine that our own AI plays. It takes the place of the original's
computer player, whose script and team system is out of scope (see *team member*). By decision of
the project lead, an AI house has the same unit rules and the same order legality as a
human-controlled house. This is a rule of our engine, not a claim about the original.
- *Orders.* An AI house acts only through orders, and its orders are applied, although the
  original drops orders issued in a computer house's name (S-0213, claim 3). They pass the same
  legality rules as a person's (S-0214) and the same checks when they are applied (S-0213), with
  no exemption from the owner check or from the limit on pending orders.
- *Unit rules.* Its units follow the rules that the specs give for units of a human-controlled
  owner. A behaviour rule that the original applies only to units of a computer-controlled owner
  (see *computer-controlled*) does not apply to them, and the recruitable property of S-0215
  (claim 20) has no effect.
- *The original's computer behaviours.* What the original's computer player gets from such rules is
  recorded in the specs as behaviour of the original. Our AI may choose to reproduce it by giving
  ordinary orders.

The decision was taken on claims of S-0213 and S-0215, and their next revisions apply it there. It is
a general rule of our engine, so it holds in every spec. Where a spec gives a rule for
computer-controlled owners, that rule describes the original, and an AI house follows the rule the
spec gives for human-controlled owners. The spec's next revision marks such a rule as the
original's only. In particular an AI house has the house skill level of a human-controlled house:
0 in a skirmish or network game (see *house skill level*).

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

**armed.** Ambiguous on its own in a rule, because the original asks different questions in
different rules: S-0209 calls a walker armed when the combined damage of its weapons is positive,
and S-0214 calls an object armed when it has a weapon in the weapon slot it currently uses. A spec
that uses "armed" in a rule says in its Terms which test it means. In a measurement setup the word
keeps its everyday sense: the object has a weapon.

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
`unmeasured` (it cannot practically be observed, even with the allowances of the *measurement
setup*, such as whether a wait exists inside the original when no observable shows it; each use
must say why). Draw counts and draw order of the random streams are measurable through the
random-state recorder of the measurement setup.

**base rules.** The expansion's main rules file: at every game start, the lowest of the rules layers
(see *rules layer*). It restores every key it gives. A game-wide key it does not give is not reset by it (see *rules
database*). See S-0004 and S-0006.

**battlefield view.** The part of the game screen that shows the map, without the sidebar and the
strip above it. See S-0101.

**behaviour.** One of the standing orders that an actor (an infantry, vehicle, aircraft or building
object) carries out, such as *move*, *attack*, *guard* or *area guard*. S-0215 names all of them
and gives each actor a current, a queued and a suspended behaviour (Terms, claim 1). In the specs a
behaviour name in italics means the behaviour, never the sequence kind of the same name in S-0107.
See S-0215; orders that set a behaviour are S-0213's.

**being erased.** The state of an object that an erasing weapon (a temporal weapon) has locked
onto, from the moment the first such weapon locks on until the last one lets go or the object is
removed. S-0215's *warped out* means the same state. The combat specs own erasing weapons. See
S-0109 (Terms) and S-0214 (Terms).

**bridge, high bridge, bridge deck.** A *bridge* is a crossing of water or a gap built from bridge
overlays. A *high bridge* is a bridge with a raised *bridge deck* that units can drive on and pass
under; a bridge without one is a *low bridge*. The deck lies 4 height levels above the terrain of
its cell (S-0102, claim 15; S-0104). Specs write "high bridge" and "bridge deck", not "raised bridge" or "raised bridge
deck". An object "on a bridge deck" stands or drives on the deck, as opposed to passing under it.

**bridge repair hut.** A building type that rebuilds the bridge linked to it when an engineer
enters it. The structures specs own it. See S-0214 (Terms).

**building.** A type kind and its objects: a stationary object that occupies a foundation of cells.
Also called a structure.

**cell.** One square of the map grid, identified by integer cell coordinates (x, y). Increasing x
runs towards the lower right of the screen, increasing y towards the lower left. A cell is 256
leptons along each map axis, and its centre is at world position (256x + 128, 256y + 128) leptons.
Its corner topmost on the screen is its *top corner*. See S-0009 and S-0101.

**cell layer.** A cell has a ground layer and, when it carries a high bridge, a bridge-deck layer:
these are its cell layers. An object on a bridge deck belongs to the deck layer, every other object
in the cell to the ground layer. It is neither a rules layer nor a render layer. See S-0209 (Terms).

**cell tag.** A trigger tag attached to one cell by the map. How a map attaches cell tags is
specified in S-0009 (claim 31). What an object entering the cell sets off is owned by S-0011
(claim 18, the entered-by event) for the codes of its subset; other codes are left to later
revisions of the trigger specs.

**chrono flash.** The full-screen chrono warp effect that a trigger action starts, one of the
*global effects*. It is not the chronosphere superweapon's jump (S-0104). S-0104 owns its lighting
and counts it as in force from the tick its action runs until it finishes. S-0201 (claim 11) owns
the rest of the effect, and a spec may use the term for the whole of it: its steps (one in every
cycle, frozen cycles included), the order block from its action on, and the freeze that starts in
the cycle after its step *D* + 44 and lasts to its last step, where *D* is the action's parameter. The two specs agree: its last step
falls in a frozen cycle, so it is in force in the ticks up to its freeze and in no tick after it.

**claim.** One numbered, observable and testable statement in a spec's Behaviour section. Every claim
carries a backing and a confidence.

**colour-adjustment table.** A table of 16 positions, each holding a tint colour with a red, a green
and a blue component. The rules data fills it, and the game combines a position's colour with an
object's pixels to tint it, for example a building under the invulnerability effect. A position that
holds zero (red, green and blue all 0) adds no tint. Rules layers change the table by entry position.
How it is reset and filled at each game start is owned by S-0006 (claims 2 and 12). See S-0006.

**computer-controlled.** A house of the original that its computer player plays rather than a
person. The specs use the word for the original: a rule given for a "computer-controlled owner"
states what the original does for units of such a house. In our engine the computer plays *AI
houses*, which follow the unit rules and order legality of human-controlled houses (see *AI
house*). The opposite of *human-controlled*.

**confidence.** How sure the spec author is of a claim: high, medium or low. Low-confidence claims
should have an open question or a measurement scenario.

**construction state.** The state of a building while it plays its build-up sprite (see *sprite*):
forwards after it is placed (*build-up*), and backwards while it is sold (*selling*). Specs write
"construction state" for the state, and "build-up" or "selling" only for one direction of it, so a
rule that holds "outside the construction state" also holds while selling. S-0108 owns its timing.
See S-0108.

**contact.** The one pairing that a mobile actor can hold with another object while the two carry
out an exchange: for example a harvester with the refinery it unloads at, a unit with the
transport or building it is entering, or a vehicle leaving the factory that built it. It lasts
until the exchange ends or one side breaks it off. A building can hold several contacts at once.
An actor "in contact with nothing" holds none. See S-0215 (Terms); the specs of the exchanges own
when contact starts and ends. A *tether link* is a firmer state that some exchanges add to a
contact.

**country.** The nationality a house plays, chosen from the countries listed in the rules data. A
country belongs to one side (S-0004, claim 9). Which types a country may build is left to the
production specs.

**credits.** The unit of money: costs, refunds, harvested resources and a house's balance are given
in credits. Each economy spec states the number domain and rounding of the amounts it computes.

**cycle.** The unit of real time in which the original runs a game. Cycles follow one another
without overlap. A cycle that is not frozen takes the player's input and runs one tick. A *frozen*
cycle, during a freeze, runs no tick and takes no input. In a single-player game every cycle makes
one drawn frame. While the game stands still (the game window has lost the input focus, or the
options menu is open) there are no cycles. See S-0201 (Terms, claims 2, 6, 8, 9 and 10). In rules
about ticks, drawn frames and real time, "cycle" means this. A spec that defines another kind of
cycle, such as the recoil cycle of S-0105, the firing cycle (planned S-0301) or a cycle of image
frames, names it in full where it defines it and may then shorten it.

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

**detail level.** The player's visual-detail option, kept in the settings file: 0 = low, 1 =
medium, 2 = high. Specs write "detail level", not "detail setting". See S-0104.

**difficulty skill block.** In the original, the group of factors that scale a house's rate of fire,
firepower, armour, cost and build time, chosen by the house's difficulty. Which block a house takes
is set-up behaviour of the original, recorded by the specs that use the factors. By decision of the
project lead, in our engine every house, human or AI, uses by default the block the original gives a
human-controlled house. AI difficulty comes from the AI's strategy modules, never from these
factors. The original's per-difficulty choice is available only as an optional per-player handicap
setting, off by default, and is never tied to AI difficulty automatically. This is a rule of our
engine, not a claim about the original.

**direction.** The direction of a point *Q* from a point *P* is the facing that the original
computes for the way from *P* to *Q*. It is close to the exact angle, but not always equal to it.
See S-0204 (claim 8, Formulas F3). One of the eight ways from a cell to a neighbouring cell is a
*step direction*, and a facing rounded to one of *n* equal steps is an *n-step facing*.

**disguise.** A unit's ability to look like a unit of another type to some players. While a disguise
is in effect against a player, that player sees the unit drawn as the disguise type. Which units can
disguise, and against whom it is in effect, belongs to the special-abilities specs. See S-0107.

**display path.** Everything between a drawn frame that the game has finished and the picture on the
monitor: the graphics driver and its settings, any compatibility layer, and whether showing a drawn
frame waits for the monitor's refresh. In a single-player game every cycle makes exactly one drawn
frame (S-0201, claim 6), so a display path that waits for the refresh can hold the tick rate back
(S-0201, scenario T1). The measurement setup allows a presentation-only *display compatibility
layer* (see *measurement setup*).

**drawn frame.** One redraw of the screen. Not the same thing as a tick, although in a single-player
game every cycle makes exactly one drawn frame, so a rule "per drawn frame" and a rule "per tick"
coincide there except during a freeze, which makes drawn frames without ticks (S-0201, claim 6).
Rendering specs say which of the two each rule follows. Not the same thing as an *image frame*
either. Specs should not write "frame" or "game frame" on its own.

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

**entry outcome.** The answer a moving unit gets for a neighbouring cell: one of eight values, in
order of increasing severity, *clear*, *cloaked blocker*, *moving blocker*, *closed friendly gate*,
*friendly obstacle*, *enemy obstacle*, *stationary friendly* and *blocked*. See S-0209 (Terms,
claim 4); S-0212 (claim 27) gives the rows it relies on for a vehicle mover.

**facing.** A direction, given as an integer *f* from 0 to 65535 that stands for *f*/65536 of a full
turn in the map plane (not an angle measured on the screen, where the isometric projection
distorts angles): one facing unit is 1/65536 of a turn. 0 is *map north* and values grow clockwise as seen on the
screen, so 16384 is *map east*, 32768 *map south* and 49152 *map west*. Map north is the direction of
decreasing cell y (towards the upper right of the screen), map east the direction of increasing cell
x (lower right), map south increasing y (lower left) and map west decreasing x (upper left).
Compound names follow: map north-west is straight up the screen. Arithmetic on facings wraps modulo
65536. The 65536 units per turn are our own unit, a definition. That the original uses this same
zero and sense is claim G1. S-0204 owns the rest (claims 10 and 11, Formulas F4 and F5): that the
original stores every facing at the full resolution of one facing unit, how a coarser *n-step
facing* (*n* a power of two from 2 to 256) is derived from a facing, and how two facings are
compared (the *signed facing difference*). A spec that derives a coarse facing in another way says
so.

**field.** One setting of a type, or of a game-wide section, that the original reads from a value.
Each field has a value form (integer, percent, boolean, list and so on, see S-0003) and an absent-key
default. Specs refer to stock numbers only through the field: "the weapon's rate-of-fire field".

**fog of war.** An optional game setting. With it on, map areas a house has explored but cannot see at
the moment keep showing what the house last saw there; S-0102 says which objects appear there as
*remembered images* and how they are drawn. With it off, explored areas show what is there now.
Which objects stay hidden under fog, and when an area counts as explored or seen, belongs to the
visibility specs. *Shroud* is a separate cover.

**follower car, leader, car chain.** A *follower car* is a vehicle that the map links to another
vehicle, its *leader*, so that it follows it, as a carriage follows its engine. A leader that is
not itself a follower car forms a *car chain* together with the follower cars behind it. S-0212
(claim 26) gives how the cars are sent after their leader, and S-0207 (claim 10) how they copy its
speed fraction. Specs write "follower car", not "following car", and "car chain", not "train", for
the linked group (see *train*).

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
real time: at game speed *s* from 1 to 6 a host that keeps up runs 62.5 / *s* ticks per second, so a
game minute lasts 14.4 × *s* seconds of real time, exactly one minute at no index from 1 to 6
(S-0201, Formulas F1); at index 0 the original's tick rate depends on the host (S-0201, claim 5). Specs write "900 ticks per game minute", not a "nominal" tick rate.

**game-mode file.** The rules file that belongs to the selected multiplayer or skirmish game mode. It
is a rules layer that stands, in the list of *rules layer*, above the base rules and the language
rules file and below the map layer. A mode may have none, and a campaign mission has no game-mode layer. In a game whose language rules
file fails its check, the game-mode layer is skipped (see *language rules file*). See S-0006.

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
loading. The rules layers are applied at every game start, and every type registry except the
tiberiums' is rebuilt then (S-0004, claim 1; S-0006, claims 1 and 10). Specs write "game start", not "scenario start" ("scenario" would
clash with *measurement scenario*).

**game-wide section.** A rules section that is neither a type's own section nor a type list: the
general settings, the combat-damage settings, the AI settings, the difficulty sections, the land-type
sections and the like. See S-0004 and S-0006.

**global effect.** One of the map-wide lighting effects of S-0104: the *nuke flash*, the *chrono
flash*, an *ion storm* or the *psychic dominator*. Each name, and the phases of each, means what
S-0104 defines, except that S-0201 owns the chrono flash's steps, order block and freeze (see
*chrono flash*). In particular, the chrono flash is not the chronosphere superweapon's jump (S-0104).
See S-0103 and S-0104.

**guard spot.** The cell or object that an actor in *area guard* stays near. See S-0215 (Terms,
claim 26).

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
also always creates two non-player houses, after the players: the *neutral house* and the *special
house*, which own neutral map objects (S-0009, claims 19 and 27). The program creates them itself
and takes their countries from the rules data. S-0009 gives how those countries are found, the
order in which all houses are created and what that order decides (claim 19), and which of the two
an unused-house cleanup can remove after they are created (claim 28).

**house colour.** The colour that marks a house's objects. It replaces the palette's remap range
(indices 16 to 31) with a ramp built from the house's colour definition. See S-0103.

**house skill level.** An integer per house that gates some automatic behaviours of its units,
by comparison with game-wide thresholds from the rules data. In a skirmish or network game every
computer-controlled house gets the rules' maximum skill level when the houses are set up, and every
human-controlled house keeps the level 0 that every house starts with (S-0209, claim 11; S-0215,
claim 15). The maximum's absent-key default is 5 (S-0209, Import mapping). In a campaign mission a
house that the map defines takes its level from the map (S-0215, claim 15). S-0215's *AI level* is
the same value; specs write "house skill level". In our engine an *AI house* in a skirmish or
network game has the level 0 of a human-controlled house, not the rules' maximum, because AI houses
follow the unit rules of human houses (see *AI house*). So every rule gated by the skill level
treats its units as it treats a human player's: for example the area-guard row of S-0215 (claim
15), the cell-wide scatter rule of S-0209 (claim 11) and automatic crushing in S-0210 (claim 11).

**human-controlled.** A house played by a person rather than by the computer. A rule that depends on
this says "human-controlled owner" or "computer-controlled owner" (see *computer-controlled*). In our
engine an *AI house* counts as human-controlled for every unit rule and for the legality and
checking of orders; whether other rules that name human-controlled houses, such as a trigger action
that acts on every human-controlled house, also cover AI houses is said by the spec that owns them.

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

**known attacker.** Damage has a known attacker when it comes with the object that caused it.
See S-0107 (Terms); S-0208 (claim 15) uses it for the scatter requests that damage sends.

**land multiplier.** The factor *M*(*ℓ*, *c*) for land type *ℓ* and locomotion class *c*, stored in
single precision. S-0207 uses it as a speed factor; a multiplier of exactly zero means that the class
cannot enter that land. See S-0211 (Terms, claims 2 and 10).

**land type.** One of twelve categories of ground that each cell has at any time: *clear*, *road*,
*water*, *rock*, *wall*, *tiberium* (resource ground), *beach*, *rough*, *ice*, *railroad*, *tunnel*
and *weeds*. See S-0211 (Terms, claims 5 to 8).

**language rules file.** An optional rules file of the original. When a file of that name can be
resolved and is valid, it is a rules layer that stands, in the list of *rules layer*, above the base
rules and below the game-mode file. A file that holds at least one section is valid only if it carries a matching
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

**lobby.** The pre-game screen of a skirmish or network game, where the players, their countries,
colours, teams and start positions and the game mode are chosen.

**local area.** The screen-aligned sub-rectangle of a map in which play takes place and to which the
view's scrolling is limited. It is given by four integers (left, top, width, height) in the same units
as the *map size*: horizontal numbers count cell widths (60 pixels on screen), vertical numbers count
cell heights (30 pixels on screen). See S-0009 and S-0101.

**local player.** The house of the person in front of this machine. In our engine only what is drawn
or heard may depend on it, never game state; each spec that uses it says what depends on it. See
S-0107.

**locomotion class.** The column of the land-type table that a mobile type uses: *foot*, *track*,
*wheel*, *hover*, *winged*, *float*, *amphibious* or *float-on-beach*, or none. It is not the
*locomotor* (how an object moves) or the *movement zone* (where the pathfinder lets it go); a type
has all three, chosen by three separate fields. See S-0211 (Terms, claim 3).

**locomotor.** The movement method of a mobile type, chosen by an imported field: for example driving,
walking, hovering, flying or jumping. It decides how the object moves between cells and what
movement states it reports. The movement specs own its details: S-0207 the driving locomotor's
movement, S-0208 infantry walking, S-0206 turning under each locomotor, and S-0106 (claim 5) which
locomotors tilt a body on slopes. The other locomotors are left to later movement specs.

**loose file.** A file stored directly in the game directory, outside any archive. A loose file is
found before an archive member of the same name, so a loose file can override shipped data. Our
engine looks for loose files in the game directory only. The original also looks in a few legacy
places outside it; S-0005 claim 4 makes not doing so a *departure*, with no switch. See S-0005.

**map layer.** The map file as a rules layer: the highest of the rules layers of a game (see *rules
layer*), so any rules section it contains overrides the lower layers. What it sets in type sections lasts for that game only, except in
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

**mission target.** The object or cell that a unit's current behaviour aims to enter, capture,
sabotage or otherwise reach, while the unit is in a behaviour that has one. S-0209 (Terms) defines
it and uses it in its entry outcomes (claim 4). Which behaviours have one belongs to the behaviour
specs and is open (S-0209, Open questions). The master of an enslaved unit is
not a mission target: S-0209 gives it a rule of its own. See S-0209.

**mod.** A change to the data files only: rules, art, maps, images or sounds. Our engine supports
mods through the importer. A mod is never part of stock data or of a measurement setup, except the
named test files a scenario uses (see *measurement setup*).

**movement zone.** A mobile type's imported category of where the pathfinder lets it go, such as on
foot, by wheel or track, by water or amphibiously. There are thirteen. The specs call them by the
lower-case names of S-0211 (claim 12), for example *normal*, *crusher*, *amphibious* and
*amphibious-destroyer*; these are our own names, which S-0107 uses too, and S-0211's Import mapping
gives how each is written in the data. S-0211 (claims 10 and 12) gives which cells each zone may cross, S-0212 builds
its connectivity from them, and S-0107 (claim 7) lets infantry of the zone *amphibious-destroyer*
use their water sequences. It is not the *locomotion class* or the *locomotor*.

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

**object.** Anything placed on the map that the game tracks individually: infantry, vehicles,
aircraft, buildings, overlays, terrain objects, smudges, animations, projectiles and so on. Every
object is an instance of one type.

**object pass, cell pass.** The two parts of making a drawn frame in S-0102. The *cell pass* paints
the persistent terrain picture (terrain tiles, overlays, terrain objects that are not animated,
building bodies and the like), and the *object pass* paints the five render layers over it. Both are
rendering terms; the *turn pass* of S-0203 is a part of a tick. See S-0102.

**occupancy record.** The record that each cell layer keeps of which standing positions are taken,
whether a vehicle, a building or another object is present, and the house of the last infantry unit
to mark a position there. See S-0209 (Terms, claim 1).

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

**parasite, victim.** A *parasite* is a unit that attaches itself to another unit, its *victim*,
and harms it from there, for example by climbing inside a vehicle or grabbing a ship. The combat and
special-abilities specs own parasites. See S-0209 (Terms) and S-0214 (Terms).

**passability category.** A number from 0 to 7 that each cell carries, computed from its land type,
overlay and contents: *open*, *crushable*, *wall-like*, *beach*, *water*, *partial obstacle*,
*blocked* and *off-map*. See S-0211 (Terms, claim 11).

**per-mille (‰).** An integer ratio where 1000 means a factor of 1.0.

**prerequisite.** A type that a house must own (usually a building) before another type can be built.
Which types are prerequisites of which comes from imported fields.

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

**reference point.** The point of an object that distance and direction measurements use: for a
building the centre of its foundation, for every other object its position. See S-0204 (claim 4).

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
floating-point results toward zero. S-0205 (claim 5) owns its scope and its single switch
`quirks.chop_rounding` (default: the original behaviour), and its Edge cases list the claims of
other specs whose results depend on the mode, among them claims of S-0003, S-0004, S-0101, S-0104,
S-0105, S-0107, S-0108 and S-0202. A spec whose results depend on the mode points to S-0205 and
defines no switch of its own.

**rules database.** The parsed rules data in force during a game. At every game start the rules
layers (see *rules layer*: the base rules, the language rules file if a valid one exists, the
game-mode file in a skirmish or network game unless the language rules file fails its check at a
later game start (see *language rules file*), and the map layer, from lowest to highest) take effect on the rules state already
there. In a campaign mission the mission-named file changes that state too, below every layer of
the mission, but it is not a rules layer and most of its changes are lost (S-0006, claim 14). Every type registry except the
tiberiums' starts each game empty, so type sections hold only what this game's layers gave.
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
override*), and the light values that S-0104 (claim 12) truncates to whole units when a higher
layer reads their section again without giving them. Specs write "higher" and "lower" for places in this list; "later" and "earlier"
mean the same. The list decides four observable things:
- *Overrides.* Where two layers give the same key, the value of the higher one is in force
  (S-0006, claims 1 and 3 to 5).
- *Registration order.* A type first named by a lower layer has a lower registration index than a
  type of the same kind first named by a higher one; a higher layer can only add types at the end
  of a registry (S-0004, claim 4; S-0006, claim 6).
- *Section reads.* A type never reads its own section from a layer below the one that created
  it, so a type that a higher layer adds takes nothing from a lower layer's section of the same
  name (S-0006, claim 7).
- *Empty layers.* An applied layer that gives nothing (a game-mode file that cannot be resolved or
  holds no section, or a language rules file with no section) changes no rules value but still
  counts: each animation that exists when that layer's animations are read gets one type read in
  it, from the art file (S-0006, claims 1 and 8).
See S-0006.

**scatter.** A unit's short move away from its spot, made to dodge a threat or to make room, set off
by a *scatter request*. For infantry, S-0208 gives which requests a unit accepts and where it moves
(claims 11 to 13), through the nearby-cell search of its claim 13 (with S-0212, claim 25). By
decision of the project lead it also owns infantry scatter from damage: which scatter requests a
hit sends (claim 15). S-0107 (claims 14 and 15) gives the idle and flee scatters. Its revision 3
also gives a scatter from damage with a known attacker (claim 16); where that differs from S-0208,
S-0208 holds, and S-0107's revision 4, in the same batch as this glossary, drops the condition
and points to S-0208. S-0209
(claims 11 and 12) gives the cell-wide scatter requests that make room, and S-0215 (claim 19) the
scatter property of behaviours. How vehicles and other kinds of unit scatter is left to later
revisions of the movement specs.

**scatter request.** A request to a unit to scatter, with an optional *threat point* (the point to
move away from) and two yes/no options: *forced* (it ignores the unit's current behaviour) and *move
anywhere* (it ignores the unit's own destination). See S-0208 (Terms), which lists the specs that
send each kind of request.

**scenario random stream.** The specs' name for the original's pseudo-random stream whose draws
decide game state, as opposed to the *unsynchronised random stream*. A *draw* takes one raw value
from it. A *ranged draw* gives a number in an inclusive range and may take no raw value, one or
several (S-0202, claim 4; S-0009). Our engine reproduces the original's draws in number and in
order, so that a game played from the same seed with the same orders has the same outcome as in
the original; that is why specs state how many draws happen and in which order. Network games are
left to a future spec (see *multiplayer game, network game*). S-0202
owns its algorithm, seeding and derived forms (claims 1 to 9) and the kinds of systems that draw
from it (claim 10); each topic spec states the draws it takes.

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

**signed facing difference.** The value *δ*(*a*, *b*) = *b* − *a* of two facings, reduced to the
range −32768 … 32767. A positive value is a clockwise turn, and a half turn counts as −32768. See
S-0204 (claim 11, Formulas F5).

**single-player game.** A skirmish or a campaign mission, as opposed to a *network game*. See S-0201.

**skirmish.** A game against computer opponents, set up in the game's lobby screen with chosen houses,
start positions and a game mode.

**slope.** A cell whose ground is inclined rather than flat. Its slope shape comes from its terrain
tile (S-0010, claim 6). S-0106 owns the slope shapes and the surface height within the cell (claims
1 and 2). Slopes affect height within the cell, drawing and movement. See S-0009, S-0010 and S-0106.

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

**sprite.** A two-dimensional image stored in a shape file in the public "SHP" format: a numbered
sequence of image frames, each a rectangle of palette indices. Infantry, buildings, animations and
some vehicles and aircraft are drawn as sprites. A sprite that a type's art names for one role is
called by that role: for a building the *body sprite*, *build-up sprite*, *bib sprite*, *deploying
sprite*, *roof-deploying sprite*, *under-door sprite* and *under-roof-door sprite*; for infantry the
*body sprite*. Specs don't call them "sheets" or "images" (an *image name*, see *art database*, names
an art section, not a sprite), and a numbered picture inside one is an
*image frame*, never a bare "frame". See S-0007, S-0107 and S-0108.

**step direction.** One of the eight directions from a cell to a neighbouring cell, numbered 0 to 7
clockwise from map north: 0 is the cell (*x*, *y* − 1), 1 is (*x* + 1, *y* − 1), 2 is (*x* + 1, *y*),
3 is (*x* + 1, *y* + 1), 4 is (*x*, *y* + 1), 5 is (*x* − 1, *y* + 1), 6 is (*x* − 1, *y*) and 7 is
(*x* − 1, *y* − 1). Even numbers are straight steps, odd numbers diagonal ones. A definition: the
numbering that S-0208 and S-0212 use.

**stock data.** The unmodified data files of the release of the original: the shipped archives and
the rules, art, AI, sound, game-mode and map files inside or beside them. Specs never copy values from
stock data. They refer to the field instead, and a value in an example is invented.

**sub-tile.** A terrain tile can cover several cells. A cell's sub-tile is the index of the part of
its tile that lies on that cell. See S-0009.

**target, destination.** An actor's *target* is the object it currently attacks, captures or
enters, and its *destination* the cell or object it currently moves to. Either may be absent. See
S-0215 (Terms); orders set them (S-0213), and the movement specs keep further points of their own.

**team member.** An actor that belongs to a team of the original's computer-player script system.
That system is out of scope for glasnost, so specs note only where membership changes a rule of the
game. An *AI house* of our engine has no such teams, so a rule's exception for team members never
applies to its units. See S-0215 (Terms).

**terrain animation.** An animation that the game creates as part of a cell's terrain when the
map's terrain is set up, such as the animation of a tile that the theater animates, rather than one
created by an object or an event. Not the same thing as an *animated terrain object*, which is a
terrain object. See S-0103 and S-0104.

**tether link, tethered.** A link that two objects in *contact* both hold during some exchanges.
An object that holds one is *tethered*. Both take it when one of them receives from the other notice
that the other begins a loading or unloading manoeuvre, clearance to unload, or a request to dock at
a refinery, and both lose it when that manoeuvre ends or the link is broken off. See S-0208
(Terms). Which situations of play send these notices is not fully traced (S-0208 and S-0213, Open
questions); S-0213 notes a probable further case, an aircraft that has landed on its pad. What a
tether changes is owned by the specs that use it, for example S-0213 (table T1) for orders.

**theater.** The terrain set of a map, one of six. It selects the terrain tiles and palettes, and it
changes the names of some art files. See S-0007, S-0103 and S-0010.

**tick.** One simulation step: the unit of game time. All timings of game state (delays, cooldowns,
animation stages, build times) are counted in ticks, against the *tick number*. A tick runs in every
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

**tileset.** A named group of consecutive terrain tiles declared in a theater's data, such as one
cliff set or one road set. A theater's tiles are numbered consecutively across its tilesets in
declaration order. See S-0009.

**top corner.** Of a cell (*x*, *y*): its corner at world position (256*x*, 256*y*) leptons, the
corner with the smallest x and y. It is the cell's map north-west corner and its topmost point on
the screen. Of a foundation: the top corner of its top-left cell, which is the topmost point of the
footprint on the screen. Specs write "top corner", not "north-west corner", for this point.

**train, train type.** A *train type* is a vehicle type with the imported train flag, and a
*train* is a vehicle of such a type (S-0210, S-0212, S-0214). The flag is a field of the type; the
link that makes a vehicle a *follower car* comes from the map, and a group of linked vehicles is a
*car chain*, not a train (see *follower car*).

**trigger, trigger action.** A *trigger* is a rule that a map defines: when its conditions are met,
it runs its *trigger actions*, such as playing an animation or changing the lighting. S-0011 owns
how triggers are set up, evaluated and fired, and the events and actions of its multiplayer subset.
Codes outside that subset are pointers only, left to later revisions of the trigger specs; some of
them are owned elsewhere, such as the freeze and chrono-flash actions of S-0201 (claims 10 and 11).
Specs write "trigger action", not "script action".

**turn.** Two senses, told apart by their wording.
- An amount of rotation: a full turn is 65536 facing units (see *facing*), and a facing turns at a
  turn rate (S-0206). This is the sense of "turn" in every spec that does not say otherwise.
- An object's share of one tick, in which it moves, fires, counts down and starts or ends its own
  actions (S-0203, Terms), as in "takes its turn" or "its turn in tick *n*". The *turn pass* is the
  part of a tick in which objects take their turns (S-0203, Terms and claim 1). A spec that uses this sense
  says so in its Terms, as S-0203 and S-0215 do, and then writes a rotation with facing words
  ("turns by", "turn rate", "a quarter turn").

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

**unsynchronised random stream.** The specs' name for the original's second pseudo-random stream,
distinct from the *scenario random stream*. It has the same algorithm and is seeded with the same
seed when a game is set up (S-0202, claims 1 to 3 and 8). During play it serves only choices that
never change game state, such as the idle comment chance of a unit owned by the local player (S-0107,
claim 14). S-0010 (claim 14) takes the draws of its 8 × 8 ground-variant pattern from it at map
load. S-0202 (claim 11) classifies its uses during play and on the setup screens, and settles how
our engine runs it: each machine runs it from the game's seed, it is not synchronised between the
machines of a network game, and none of its draws during play reaches game state. A spec that names a draw from this stream also says what the
draw affects.

**usable area, usable cell, playable area.** The part of the local area in which play takes place. A legal cell is
*usable* when the test of S-0009 (claim 4) puts it inside the local area. S-0009 (claim 5) says
which uses take the test's height-aware form; cursor orders, object placement and the nearby-cell
search do (claim 5a). *Playable area* (S-0009, S-0010, S-0213) means the same. See S-0009. S-0211
(claim 11) uses it for the off-map passability category.

**vehicle.** A type kind and its objects: wheeled, tracked, hovering and naval units, drawn as voxel
models or sprites (see S-0007, claim 4).

**vehicle thief.** An infantry unit whose type has the imported vehicle-thief flag: it takes over a
vehicle by entering it. Its rules belong to the special-abilities specs. See S-0209 and S-0214.

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
    turn are our own unit (see *facing*). How finely the original stores a direction in general is
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

    G3 makes no claim about any other delay that may be converted the same way (Open questions).
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
    - Backing: `measurable` (Scenario G3 for the timings outside the exceptions; the exceptions are
      measured by the scenarios of the specs that own them)
    - Confidence: medium (no other use of the game-speed index by a game rule is known. The one
      timer of the original known to run in real time during a game and reach game state is the
      length of a timed freeze, which reaches it only through exception 2 (S-0201, claims 10 and
      11); our engine counts that length in cycles, a departure of S-0201 that removes the host's
      influence but keeps the game-speed dependence. Relighting does not depend on real time
      (S-0104, claim 12). Not every timer of the original has been traced)

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

The glossary owns no quirk; *quirk switch* defines the notation the specs use. It points to three
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
  S-0009 (revision 7, the farthest-sum rule for start positions), S-0207, S-0209 (claim 8, Formulas
  F1), S-0212, S-0214 and S-0215, and S-0208, which notes that it changes nothing there. An earlier revision of S-0009
  proposed a switch of a different name for the same table; by decision of the project lead the two
  are one switch under this name, and S-0009 revision 7, in this batch, uses it. It is the only name
  of this switch.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| *Settings file*, section `[Skirmish]`, key `GameSpeed` | game-speed index *s* of a skirmish, as the starting position of the lobby's speed control (claims G2 and G3; S-0201, claim 7) | integer; the claims cover 0 to 6, and no other value is covered here (7 has a column in S-0107's table T3) | S-0201: the rules' lobby default (rules, section `[MultiplayerDialogSettings]`, key `GameSpeed`), and 6 when that is absent too; every scenario here sets the key |
| *Settings file*, section `[Options]`, key `GameSpeed` | not the speed of a skirmish started from the lobby: S-0201 (claim 7) replaces it at skirmish start with the lobby's index | integer | 3 (S-0201) |
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
- *Setup:* a small purpose-made or stock skirmish map, one human player and one computer player.
  Before each run the settings file's skirmish game-speed value is set to one of 0, 1, 2, 3, 4, 5
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
  *N*₀ ≥ 0.99 × *N*₁ confirms part (b). A run set in which *N*₀ is not above 1.01 × *N*₁ cannot
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

- How game speed works in network games is left by S-0201 (claim 13) to a future network-game spec.
  Claims G2 and G3 cover skirmish games only, and S-0201 claim 6 (one drawn frame per cycle) covers
  single-player games only.
- One further delay is believed to be converted with the same game-speed normalisation, but no spec
  owns it and claim G3 does not cover it: the shortest time between two repeats of a spoken warning
  to a house (for example the warning that a house cannot pay for what it is building). It is
  believed to affect only what is heard, not game state. If it reached game state, it would fall
  under exception 1 of claim G3, and the spec that comes to own it would say how. Its exact rule,
  and which warnings it covers, should be settled by the interface specs.

## Questions for the project lead

- S-0301 (the firing cycle) and S-0310 (turrets and facing for fire) exist as drafts of a later
  batch. By *Conventions* this glossary treats S-0301 as planned, because it is not signed off with
  this batch, and does not cite S-0310. If the lead puts either into this batch's sign-off, the
  next revision cites it as a draft of the batch (by number and claim), not as planned.
