---
id: S-0202
title: Random streams (generator, seeding, ranged draws, which systems draw from which stream)
topic: simulation
revision: 5
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: []
supersedes: []
---

# S-0202: Random streams (generator, seeding, ranged draws, which systems draw from which stream)

## Summary

The original runs several independent copies, or *streams*, of one pseudo-random generator; this
spec gives the generator exactly, the rule that turns raw values into ranged draws and their derived
forms, when and from what the streams are seeded, and which game systems draw from the scenario
random stream and which from the unsynchronised random stream. It also states two kinds of draw that
no feature spec lists yet: the creation draw of an object and the draws of overlay placement
(claim 10).

## Terms

Shared terms used here are defined in the glossary: *scenario random stream* (with *draw* and
*ranged draw*), *unsynchronised random stream*, *game*, *game start*, *session*, *skirmish*,
*object*, *tick*, *departure*, *importer warning*, *quirk switch*, *recorder*, *measurement setup*,
*round-toward-zero mode*. In addition:

- **Stream.** One copy of the generator with its own state. The original has three that matter
  here: the scenario random stream, the unsynchronised random stream, and the *map-generator stream*
  (claim 13).
- **Raw value.** One output of a stream: an unsigned 32-bit integer, 0 to 4294967295. Taking a raw
  value advances the stream by one. Where a spec reads a raw value as a signed number, it is the
  two's-complement reading, −2147483648 to 2147483647.
- **Stream sequence.** The raw values a stream produces after seeding, numbered x₀, x₁, x₂, … in the
  order they are taken. The 250 *seed values* x₋₂₅₀ … x₋₁ are the values the seeding function
  gives (claim 3); they are never returned as raw values themselves.
- **Seed.** The 32-bit unsigned integer from which a stream's 250 seed values are computed.
- **Ranged draw R(a, b).** The number in the inclusive range between two signed 32-bit integers *a*
  and *b* that claim 4 derives from zero or more raw values. This refines the glossary's *ranged
  draw*: one ranged draw may take no raw value, one, or several.
- **Unit-interval value.** A double-precision number from 0 to 1 derived from one ranged draw
  (claim 6, Formulas).
- **Percent draw.** The ranged draw R(0, 99).
- **List pick.** A raw value, read as unsigned, reduced modulo a list length *n* to an index 0 … *n*−1.
- **Suspended.** A state of a stream in which every raw value it is asked for is 0 and its state
  does not change (claim 7).
- **Network game.** A multiplayer game played over a local network or the internet, as opposed to a
  skirmish or a campaign mission.
- **Overlay type stream, overlay data stream.** The map's stream of overlay types and its stream of
  overlay data values, as S-0009 uses them (claims 12 and 13). Pointers only.
- **Visiting order.** The order in which S-0009 visits a map's cells (S-0009, claim 6, and its
  scenario M2, sub-test "visiting order"). A pointer only.
- **Tile animation.** The animation that a cell's tile names and that the cell starts when it is
  recalculated (S-0010, claim 19). It is a *terrain animation* in the glossary's sense.
- **Cell animation.** The animation that the rules data attaches to an overlay type and that plays
  on each cell holding that overlay, as S-0009 defines it (Terms).
- **End row, bridge piece.** For the low bridges of claim 10: the end row is the row of three cells
  across the bridge's line that an end marker writes where it is placed; a bridge piece is one of
  the three cells across of each cell step between two end rows. The shape of an end row (which
  three cells, and when it can be written) belongs to the bridge rules, which have no spec yet; a
  pointer only.
- **Vein, vein cell.** A cell holding the vein overlay, the overlay type at position 126 of the
  overlay type list (claim 10). "Loses its vein" means the cell no longer holds any overlay. The
  vein rules, which have no spec yet, own what veins do; a pointer only.
- **Pattern centre.** A vein cell whose overlay data is 48 or more. A placement (claim 10) gives this
  to the placed cell, flat (48 … 50) or sloped, and also to every sloped neighbour it sets (their data
  is 53 or more, formula *Vein pattern values*). The re-patterned and kept flat cells have data below
  48 and are not pattern centres. Neither is a flat placed cell that gets 46 under
  `quirks.random_magnitude_overflow` (Edge cases).
- **Vein placement test.** The condition a cell must meet for a vein placement to take effect there
  (claim 10). It belongs to the vein rules; a pointer only.
- **Vein connection pattern.** The integer 0 … 15 that the vein rules derive for a flat vein cell from
  its vein edge neighbours, written *c* in the formula *Vein pattern values*. It belongs to the vein
  rules; a pointer only.
- **The veinhole monster.** The map object that the rules data's veinhole-monster overlay type creates
  where it is placed: the stationary object that grows veins around it during play, which S-0009
  (Terms) calls the veinhole. What it does belongs to the vein rules; a pointer only. Creating it
  takes no draw of its own (claim 10).
- **Fence post, fence line.** A fence post is a building whose type the rules data marks as a post.
  Between two matching fence posts within range the original builds a line of fence-section
  buildings, the fence line. Stock data has no such building type. The fence rules have no spec yet,
  a pointer only.

## Behaviour

### The generator

1. **State and output.** A stream's state is its 250 most recent values x<sub>N−250</sub> …
   x<sub>N−1</sub>, seed values included, where *N* is the number of raw values taken since the
   seeding; by claim 2 they determine every later raw value. Each raw value is the
   recurrence's result of claim 2 itself, all 32 bits of it, with no bit dropped or changed. A
   stream returns the same sequence for the same seed on every machine.
   - Backing: `measurable` (scenario R2)
   - Confidence: high

2. **Recurrence.** For every n ≥ 0, x<sub>n</sub> = x<sub>n−250</sub> ⊕ x<sub>n−147</sub>, where ⊕
   is bitwise exclusive-or and x₋₂₅₀ … x₋₁ are the seed values. So the first 147 raw values, x₀ …
   x₁₄₆, combine two seed values each; x₁₄₇ … x₂₄₉ combine an earlier raw value and a seed value; and from x₂₅₀ on every raw value combines two earlier raw values. Consequence: any bit position of the raw values,
   including the lowest two, obeys the same relation on its own, so the low bits that a small
   power-of-two range keeps (claim 4) satisfy x<sub>n</sub> = x<sub>n−250</sub> ⊕
   x<sub>n−147</sub> too.
   - Backing: `measurable` (scenarios R1 and R2)
   - Confidence: high

3. **Seeding function.** Seeding a stream with seed *s* sets seed value x<sub>k−250</sub> = F(*s*,
   *k*) for *k* = 0 … 249, where F is the four-round mixing function of the Formulas. Seeding
   discards all previous state, and the next raw value taken is x₀. Test values are in the
   Formulas.
   - Backing: `measurable` (scenario R2)
   - Confidence: high

### Ranged draws and derived forms

4. **Range rule.** A ranged draw R(*a*, *b*) between signed 32-bit integers:
   - if *a* = *b*, the result is *a* and no raw value is taken,
   - otherwise the bounds may come in either order: with *lo* the smaller and *hi* the larger, and
     *d* = *hi* − *lo* (an integer from 1 to 4294967295), the result is *lo* + *v*, where *v* =
     *x* mod 2<sup>*m*</sup> for the first raw value *x*, counting from the stream's next one, with
     *x* mod 2<sup>*m*</sup> ≤ *d*, and *m* is the number of binary digits of *d* (the smallest *m*
     with *d* < 2<sup>*m*</sup>). So *v* is the lowest *m* bits of that raw value. The draw consumes
     every raw value from the stream's next one up to that one, the rejected ones included.
   - The spans of claim 5 behave differently.

   The result is uniform over *lo* … *hi*. A range whose size *d* + 1 is a power of two takes
   exactly one raw value. Otherwise the expected number of raw values is 2<sup>*m*</sup> / (*d* + 1),
   which is below 2. For example R(0, 10) keeps 4 bits and rejects 11 … 15, so it takes 16/11 raw
   values on average.
   - Backing: `measurable` (scenarios R1, R5)
   - Confidence: high

5. **Very wide spans.** Two spans fall outside claim 4, both reachable only when a range bound comes
   from extreme data:
   - *d* = 2147483647 exactly (for example R(0, 2147483647)): no raw value is taken and the result is
     *lo* + 2147483648, wrapped into the signed 32-bit range, which lies outside the requested range
     (R(0, 2147483647) gives −2147483648). Quirk `quirks.ranged_draw_half_span` (Edge cases).
     Because the result lies outside the range the data asked for, the importer warns for data that
     can make a span exactly this wide, as it does for the spans below.
   - *d* ≥ 2147483648: the original never finishes the draw. It takes raw values forever and the
     program stops responding. Departure: our engine applies claim 4's rule with *m* = 32: the result is
     *lo* + *x*, wrapped into the signed range, for the first raw value *x* with *x* ≤ *d*, and the
     draw consumes every raw value from the stream's next one up to that one.
   - A spec whose imported fields can make a span of 2147483647 or more says so, and the importer
     warns for such data, naming the field and the span; the game still runs, with the result
     above for each span.
   - Backing: `unmeasured` (no stock data reaches either span; a test would need a data path to a
     range bound that the topic specs have not yet named)
   - Confidence: high

6. **Derived forms.** Other specs use these forms of the scenario random stream; each spec names
   the form it uses and, for comparisons, whether a test is "less than" or "at most".
   - *Unit-interval value u* (Formulas): one ranged draw R(0, 2147483646) times a fixed double
     close to 1/2147483646. Its largest value is 1 − 2⁻⁵³, never 1, in the original; that depends on
     the round-toward-zero mode (S-0205, claim 5).
   - *Chance test against a decimal p*: a unit-interval value compared with *p*, a fraction (for
     example 0.25 for 25 %). With "less than", the chance is the fraction of the 2147483647 equally
     likely values of R(0, 2147483646) whose *u* lies below *p*.
   - *Percent draw*: R(0, 99), compared with an integer percentage.
   - *List pick*: one raw value read as unsigned, modulo the list length; one raw value, no
     rejection, so a list length that is not a power of two is very slightly biased toward low
     indices.
   - *Signed remainders*: one raw value read as signed, then its remainder on division by *n*
     (sign of the dividend; S-0205, claim 2), in one of three variants: the plain remainder (from
     −(*n* − 1) to *n* − 1); "magnitude first", the remainder of the magnitude; and "magnitude
     after", the magnitude of the remainder, which always lies in 0 … *n* − 1. The magnitude of
     −2147483648 is −2147483648 itself, so the "magnitude first" variant can still give a negative
     remainder, for one raw value in 2³² (Edge cases); "magnitude after" has no such case.
   - *Low bits*: one raw value, keeping some of its lowest bits.
   - Backing: `measurable` for the unit-interval value through scenario R5's recorder (the draw
     is a ranged draw whose raw values R5 predicts); the other forms are definitions that the topic
     specs measure in their own scenarios
   - Confidence: high (the forms); medium (that this list covers every form in use)

7. **Suspended while a game is cleared.** Every game start, from the menus or as a campaign
   continuation, clears what the previous game left before the map is read. While that clearing
   runs, the scenario random stream is suspended: anything that asks it for a raw value then
   receives 0 (so a ranged draw under claim 4 returns its lower bound) and the stream does not advance.
   Consequence: clearing a game takes no draws, and the stream has the same state when the clearing
   ends as when it began. In the original the clearing comes after the seeding of claim 8 on a
   start from the menus; because the clearing takes no draws, nothing observable in the scenario
   random stream depends on that order. Our engine takes no draws from the scenario random stream
   while it clears a game.
   - Backing: `unmeasured` (the state recorder of the Measurement scenarios gives only the number
     of raw values taken since the seeding, modulo 250, so draws during the clearing, had there
     been any, could not be told apart from the map loading's own draws that follow)
   - Confidence: medium

### Seeding and saved games

8. **Seeding at game start.** When a game is set up from the game's setup or menu screens (every
   skirmish, every network game, and a campaign mission started from the menus), both the scenario
   random stream and the unsynchronised random stream are seeded with the same seed before the map
   is read. The two streams therefore start in identical states: their raw-value sequences are the
   same, and only the number of values each has taken differs once their users diverge. Where the
   seed comes from:
   - *Skirmish and campaign mission:* the original uses a millisecond counter of the host, the
     number of milliseconds that the host computer has been running, modulo 2³², read when the game
     is set up. So the seed differs from game to game and from launch to launch, and the seeds of
     two games started in one session differ by the time between their starts, in milliseconds.
     **Departure** (host- and clock-dependent): our engine takes the seed as a 32-bit unsigned value
     given with the game setup, and is otherwise fully deterministic in it. How the setup chooses
     that value is a matter for the interface, not for the simulation.
   - *Local-network game:* the hosting machine's setup screen chooses a seed from 0 to 32767 and
     sends it to the other machines, so every machine of the game uses the same seed.
   - *Internet game, or a game set up by another setup program:* the seed arrives with the game
     options and can be any 32-bit value; every machine uses the same seed.
   - *Playback of a recorded game:* the seed stored with the recording. In addition, when the
     original is started in one of its own recording or playback modes, it does not read the clock
     for a skirmish or campaign mission either, and keeps the seed it already has.
   - *Campaign continuation:* when a campaign mission is followed directly by the next mission
     after a win, or is restarted after a loss, without returning to the menus, the streams are not
     reseeded: they continue from the state the previous mission left. Our engine does the same.

   Backing and confidence by part:

   | Part | Backing | Confidence |
   |---|---|---|
   | Skirmish: same seed for both streams, seeded before the map is read | `measurable` (R2 a) | high |
   | Skirmish: the seed is a millisecond counter (seeds of one session differ by the time between game starts) | `measurable` (R2 a; R4 for the change between launches) | high |
   | Skirmish: the counter counts from the start of the host computer | `measurable` (R2 a, secondary expectation) | medium (the measurement setup runs the original on a host layer that supplies the counter) |
   | Local network: one shared seed, in 0 … 32767 | `measurable` (R2 b) | medium |
   | Internet game and other setup programs | `unmeasured` (the measurement setup has no online service or other setup program) | medium |
   | Playback uses the recording's seed | `unmeasured` (the recording and playback modes are development features outside the measurement setup, as in claim 12) | medium |
   | Recording and playback modes skip the clock | `unmeasured` (same reason) | low |
   | Campaign continuation: no reseeding | `measurable` (R2 c) | medium |
   - Backing: `measurable` (scenarios R2, R4) for the parts so marked above; the rest `unmeasured`,
     with the reasons given above
   - Confidence: high to low by part, as given above

9. **Saved games keep the scenario random stream.** A saved game stores the scenario random
   stream's full state, and loading it restores that state, so the game continues with the raw
   values it would have taken had it not been saved. Loading the same saved game twice, with the
   same orders given at the same ticks, therefore reproduces the same random outcomes. What state
   the unsynchronised random stream has after a load is not established (Open questions); it does
   not affect game state (claim 11).
   - Backing: `measurable` (scenario R3)
   - Confidence: medium

### Which systems draw from which stream

10. **Scenario random stream: everything that can change game state, and more.** Every random
    decision that can affect game state draws from the scenario random stream: for example combat
    (projectile spread, scatter, damage-related picks), unit behaviour (idle actions, S-0107),
    computer-player decisions, map loading and start positions (S-0009), resource growth and
    spread (including the vein patterns below), random map events, crates and triggers. Several
    effects that look like presentation also draw from it: animations, particles and particle
    systems, debris, smudges, and the overlay placement effects below. Consequence for our engine:
    these presentation-like effects must run inside the deterministic simulation, in the
    original's order, or every later draw of the stream shifts. Two kinds of draw are stated here
    because no feature spec lists them yet:
    - **The creation draw.** Every creation of an infantry, vehicle, aircraft or building object
      takes one raw value, whatever else that creation draws, at the moment the object is created.
      Creating an object is not the same as its entering play (S-0203): the draw is taken even for
      an object that never enters play. Where the creation falls:
      - *Map loading.* A pre-placed object is created when its map entry is read, so its creation
        draw comes before the draws of its own tag (S-0011, claim 34) and before any draw its
        placement takes. An entry that names no house of the game (S-0009, claim 27) or an unknown
        type creates nothing and takes no draw; an entry that is created but cannot be placed has
        still taken its draw. The players' starting units take their creation draws when they are
        placed. The table *Draws during map loading* (State and ordering) places all of these
        relative to S-0009's phases and S-0011's tag draws.
      - *Production.* A produced object is created when its production begins, not when it leaves
        the building that produced it. Its creation draw therefore falls in whatever part of the
        tick starts that production (in S-0203's tick table, the production lines or the
        processing of orders), not at its entering play. The production specs fix that timing,
        including when the next item of a queue begins.
      - *Every other creation* (reinforcements, crates, deploying, paratroopers and similar)
        happens at a time the spec of that feature fixes, and the creation draw comes with it.
    - **Overlay placement.** Creating an overlay object takes no draw by itself. Some effects of
      placing it on a cell do:
      - *Bridge-building end markers.* The overlay types at positions 122 … 125 and 233 … 236 of
        the overlay type list are end markers for two families of low bridges. They pair up as 122
        with 123, 124 with 125, 233 with 234 and 235 with 236. A placed marker writes its end row
        (Terms), which takes no draw of its own (its cells are recalculated, as the pieces' cells
        are below), and looks along its line for the end row of its partner. When
        it finds one, the *d* cells between the two end rows (*d* ≥ 1, counted along the line) are
        filled with bridge pieces, three across for every cell step, so 3*d* pieces. Each piece takes
        one raw value: its overlay type is the base type *t*₀ of its marker pair plus the lowest two
        bits of that raw value (the *low bits* form; formula *Bridge piece type*), so completing a
        bridge takes 3*d* raw values. The pieces take their raw values one cell step after another,
        starting next to the partner's end row and moving toward the marker just placed, and within
        a step in the order of their place across the bridge (their overlay data 0, 1, 2). The
        pieces do not count as placed overlays: they set off none of the other effects in this
        list. Each piece's cell is recalculated as the piece appears, though, which can start that
        cell's tile animation; that animation's creation draws then come right after the piece's
        own raw value. This also happens during map loading, where the
        marker that comes later in the overlay type stream completes the bridge.
      - *Vein patterns.* The vein overlay is the overlay type at position 126 of the list (the
        type S-0009, claim 14, reads as a marker). Placing it during play on a cell that passes the
        vein placement test takes one raw value for that cell. On a flat cell it then takes one
        more for each of the cell's four edge neighbours that the placement re-patterns, visiting
        them in the order map north, map east, map south, map west (which neighbours are
        re-patterned, and their connection pattern, belong to the vein rules, which have no spec
        yet; a flat neighbour that holds no overlay is always re-patterned); a sloped placed cell
        re-patterns no neighbour. A cell that fails the test takes no draw. The draws set the
        cells' overlay data by the formula *Vein pattern values*: a flat placed cell uses the
        "magnitude first" remainder modulo 3, a flat neighbour the "magnitude after" remainder
        modulo 3, and a sloped cell the lowest bit.
        A neighbour that is already a pattern centre (Terms) is never re-patterned. During play
        this placement happens when the vein rules grow veins (around a veinhole monster); what
        sets it off belongs to those rules.
        *Recompute.* The veins around a cell are recomputed during play when a unit that harvests
        veins harvests that cell, when a veinhole monster is removed (its own cell, which must be a
        pattern centre, is recomputed) and when a fence line (Terms) is built across a vein cell,
        which can also happen near the end of map loading (the map-loading table, row *Fence
        lines*).
        A recompute concerns the recomputed cell and those of its edge neighbours that are flat
        vein cells and not pattern centres. Each of them takes at most one raw value; the overlay
        data of one that keeps its vein follows the formula *Vein pattern values* (row "a
        re-patterned or kept cell, flat"):

        | Cell | Condition | Draw | Result |
        |---|---|---|---|
        | The recomputed cell | sloped, or no vein connection (connection pattern 0) | none | loses its vein |
        | The recomputed cell | otherwise | one raw value ("magnitude after" remainder, modulo 3) | stays a vein |
        | A flat vein edge neighbour that is not a pattern centre | no vein connection (connection pattern 0) | none | loses its vein |
        | A flat vein edge neighbour that is not a pattern centre | a vein connection | one raw value (same form) | keeps its vein |

        In stream order, the recomputed cell's raw value, if it takes one, comes first, then the
        neighbours' in the order map north, map east, map south, map west. How the connection
        pattern is derived belongs to the vein rules.
        *Map loading.* Placing a vein from the overlay type stream takes no draw; instead the pass
        of S-0009 claim 14, which runs after the overlay data stream has been read, places the
        vein again on every candidate cell that passes its test, with the placement draws above,
        one candidate after another in the reverse visiting order. The data values these draws set
        are therefore the ones in force after loading, for those cells.
      - *Cell animations.* An overlay type with a cell animation starts that animation when it is
        placed, during map loading too; the animation takes whatever draws its creation takes
        (the animation specs).
      - Walls, ore and gem fields, crates, the high bridge overlays and the veinhole-monster overlay
        take no draw of their own when placed; the veinhole monster that the last one creates takes
        none either. (Placing any overlay recalculates its cell, which can start that cell's tile
        animation; see the map-loading table.)
    - Backing: `measurable` for the creation draw (scenario R5 variant b); for the bridge draws
      (scenario R8 part A: their number, form, order and base types); for the vein placement draws
      (R8 part B, which tests the placement rule through the load-time pass, the same rule as a
      placement during play); for walls, ore and gems taking no draw (R8 part C). `unmeasured`:
      that crates, the high bridge overlays and the veinhole-monster overlay (and the monster it
      creates) take no draw (each needs set-up conditions of rules that have no spec yet; a later
      revision can add them to R8 C); what
      sets off a vein placement during play, and the recompute draws (the recompute's triggers are
      bound up with other effects, such as a harvesting unit's or a veinhole monster's own draws,
      that no spec lists yet, and one recompute gives at most five values, each telling only its
      remainder modulo 3, too few to find its place in the stream the way R8 does). The
      classification is the union of the topic specs' claims.
    - Confidence: high (the creation draw is taken; a produced object is created when its
      production begins; the overlay draw counts and forms); medium (the list of kinds; which
      vein neighbours are re-patterned and in which order; the order of the bridge pieces and the
      base types; where each map-loading group sits in the stream; the recompute: its draws, order
      and list of triggers)

11. **Unsynchronised random stream: presentation during play and at map load, setup values
    before play.** No draw from this stream that changes game state has been found. Its uses:
    - *During play:* which variant of a sound or voice response plays (including the idle comment
      of S-0107, claim 14, and the local player's unit responses and the spin-up sound of
      rapid-fire weapons), the choice of music track, the screen-shake displacement of an
      explosion, the jitter of electric-bolt and radiation-beam effects and a flicker colour on
      some objects.
    - *At map load:* the 8 × 8 terrain-image pattern of S-0010 (claim 14), built from 64 to 4096
      consecutive draws the first time in a session that a map needs it. It chooses which image of
      a multi-image tile each cell shows, so it changes only what is drawn. In the original it is
      built once and then kept for the rest of the session, so it depends on the stream's history
      in that session: two machines of a network game draw the same pattern only if the pattern
      was built on both in the same state, and a machine that has played an earlier game in the
      session keeps that game's pattern. The machines of a network game can therefore show
      different ground images for the whole game. Our engine builds the pattern afresh at every
      map load (Edge cases).
    - *Before play, on the setup screens:* setup values: the defaults offered by the random-map
      setup screen, a random map seed (claim 13), the default network port range, and one ranged
      draw over 1 … 32767 from which a setup screen derives a game seed (through the third
      generator of claim 13; for a local-network game this gives the seed of claim 8). These draws
      use the stream in whatever state the previous game left it or, before the first game of a
      session, the state of a seeding by claim 8's clock rule that the original makes when the
      program starts. Such a value then becomes part of the game setup, like the game seed, and reaches the
      game only through the setup; in our engine these values are interface choices (claim 8's
      departure).

    Our engine runs this stream on each machine from the game's seed (claim 8) with the same
    algorithm, does not synchronise it between the machines of a network game, and lets none of
    its draws reach game state.
    - Backing: `measurable` (scenario R6); the map-load use through S-0010's scenario V2: part A for
      the pattern's properties, part B for the draws themselves, with the state recorder's reading
      of the unsynchronised stream (Measurement scenarios)
    - Confidence: medium (every use was classified, but some only briefly; Open questions); high
      for the map-load use

12. **Extra draws in network games.** In a network game the original also takes two raw values from
    the scenario random stream in each tick for its own synchronisation check (the values are mixed
    into a checksum that the machines compare), and in some conditions one more ranged draw R(0, 2)
    whose result is not used. None of these happens in a skirmish or a campaign mission. During
    playback of a recorded game, however, the original takes the two checksum draws in every tick
    whatever kind of game was recorded (low confidence on what this does to the playback of a
    skirmish). Network games are out of scope for the skirmish specs: the draws are recorded here
    as the original's behaviour, and whether and how our engine reproduces them is decided by the
    future multiplayer spec. In a skirmish, which is all that this spec's engine requirement
    covers, our engine takes no such draws.
    - Backing: `measurable` (scenario R7); the playback sentence is `unmeasured` (the recording
      and playback modes are development features outside the measurement setup)
    - Confidence: medium (two per tick, network games only); low (the condition of the extra draw)

13. **Map-generator stream.** The random-map generator has a stream of its own with the same
    algorithm. It is seeded from the random map's 16-bit seed setting, or, when that setting is 0,
    from a ranged draw R(0, 65535) of the unsynchronised random stream. It is used for nothing but
    generating the random map, whose rules belong to a future spec. A third generator of a different
    kind serves only the original's setup screens (to choose a network game's seed, claim 8),
    visual effects and the names of saved-game files; our engine need not reproduce it.
    - Backing: `unmeasured` (random maps are outside the current scope)
    - Confidence: medium

## Formulas

All arithmetic in this section is on integers, exact, unless it says otherwise. "mod 2³²" reduces
to 0 … 4294967295. For a value *t* in 0 … 2³² − 1, *σ*(*t*) is its signed reading: *t* if *t* <
2³¹, else *t* − 2³². ⊕ is bitwise exclusive-or of two 32-bit values.

**Raw sequence** (claims 1, 2): x<sub>n</sub> = x<sub>n−250</sub> ⊕ x<sub>n−147</sub> for n ≥ 0,
with x<sub>k−250</sub> = F(*s*, *k*) for *k* = 0 … 249.

**Mixing function F(s, k)** (claim 3). F acts on a pair of 32-bit unsigned values over four
rounds, starting from *L*₀ = *s* and *R*₀ = *k*. For *i* = 0, 1, 2, 3:

> *L*<sub>*i*+1</sub> = *R*<sub>*i*</sub> and *R*<sub>*i*+1</sub> = *L*<sub>*i*</sub> ⊕
> *G*<sub>*i*</sub>(*R*<sub>*i*</sub>), and F(*s*, *k*) = *R*₄.

The round function *G*<sub>*i*</sub> of a 32-bit unsigned value *y*, with the constants
*P*<sub>*i*</sub> and *Q*<sub>*i*</sub> of the table below and *A* = *y* ⊕ *P*<sub>*i*</sub>, is

> *G*<sub>*i*</sub>(*y*) = ((*T*(*B*) ⊕ *Q*<sub>*i*</sub>) + *h* × *ℓ*) mod 2³²

where

> *h* = ⌊*σ*(*A*) / 65536⌋ (floor, so *h* is a signed integer from −32768 to 32767) and
> *ℓ* = *A* mod 65536,
>
> *B* = (4294967295 − *h*² + *ℓ*²) mod 2³²,
>
> *T*(*B*) = (*B* mod 65536) × 65536 + ⌊*B* / 65536⌋ when *B* < 2³¹,
> and *T*(*B*) = 4294901760 + ⌊*B* / 65536⌋ when *B* ≥ 2³¹.

*h* × *ℓ* is a signed product (negative when *h* < 0), reduced mod 2³² as part of the sum. When
*B* ≥ 2³¹, *T*(*B*)'s upper 16 bits are all ones whatever *B*'s lower half was: *T* is a swap of the
two 16-bit halves only for *B* < 2³¹.

| *i* | *P*<sub>*i*</sub> | *Q*<sub>*i*</sub> |
|---|---|---|
| 0 | 3131664519 | 1259289432 |
| 1 | 504877868 | 3899977923 |
| 2 | 62708796 | 1767228838 |
| 3 | 255054258 | 1437059654 |

Test values (decimal):

| Seed | x₋₂₅₀, x₋₂₄₉, x₋₂₄₈ | x₋₁ | x₀, x₁, x₂, x₃ |
|---|---|---|---|
| 0 | 2905516568, 1718636987, 3199744093 | 3610121526 | 1229352179, 1781224790, 2424954917, 2632964923 |
| 1 | 4099924131, 2128042508, 2801846478 | 1868701209 | 2025287381, 660436142, 3663968561, 2901129938 |
| 123456789 | 1236345482, 1460399687, 2538562447 | 461485239 | 3323095506, 1961208251, 3793762349, 793961667 |

For seed 0, x₉₉₉ = 2255550017.

**Ranged draw R(a, b)** (claims 4, 5). With *lo* = min(*a*, *b*), *hi* = max(*a*, *b*), *d* =
*hi* − *lo* (exact, 0 … 4294967295), and x<sub>*q*</sub> the stream's next raw value:

> *d* = 0: result *a*, no raw value.
>
> 1 ≤ *d* ≤ 2147483646: *m* = ⌊log₂ *d*⌋ + 1, and *w* is the smallest integer *w* ≥ 0 with
> (x<sub>*q*+*w*</sub> mod 2<sup>*m*</sup>) ≤ *d*. Result *lo* + (x<sub>*q*+*w*</sub> mod
> 2<sup>*m*</sup>), which lies in *lo* … *hi*. The draw consumes the *w* + 1 raw values
> x<sub>*q*</sub> … x<sub>*q*+*w*</sub>.
>
> *d* = 2147483647: no raw value; result *σ*((*lo* + 2147483648) mod 2³²) (quirk
> `quirks.ranged_draw_half_span`; with the switch off, the 1 ≤ *d* ≤ 2147483646 rule applies with
> *m* = 31).
>
> *d* ≥ 2147483648: departure: *w* is the smallest integer *w* ≥ 0 with x<sub>*q*+*w*</sub> ≤ *d*.
> Result *σ*((*lo* + x<sub>*q*+*w*</sub>) mod 2³²). The draw consumes x<sub>*q*</sub> …
> x<sub>*q*+*w*</sub>.

Expected number of raw values consumed, *w* + 1, for 1 ≤ *d* ≤ 2147483646: 2<sup>*m*</sup> / (*d* + 1),
an exact rational (a statistical statement about the draw, not a value the engine computes).

**Unit-interval value** (claim 6). With *r* = R(0, 2147483646), an integer, and *c* the double
(2³⁰ + 1) / 2⁶¹ (the double nearest to 1/2147483646):

> *u* = RZ(*r* × *c*), the exact product rounded to a double

where RZ is the original's rounding toward zero (S-0205, claim 5). *u* ranges from 0 to 1 − 2⁻⁵³. The exact
product for *r* = 2147483646 is 1 − 2⁻⁶⁰; rounding toward zero gives 1 − 2⁻⁵³, while rounding to
nearest would give exactly 1. So with the switch `quirks.chop_rounding` of S-0205 off, *u* can equal 1.

**List pick:** index = *x* mod *n* for one raw value *x* and list length *n* ≥ 1.

**Signed remainder:** *σ*(*x*) rem *n* (sign of *σ*(*x*)); "magnitude first": |*σ*(*x*)| rem *n*,
where |−2147483648| is taken as −2147483648 (Edge cases); "magnitude after": |*σ*(*x*) rem *n*|,
always 0 … *n* − 1. For *n* = 3, "magnitude first" gives −2 for *x* = 2147483648 (signed reading
−2147483648 = −715827882 × 3 − 2).

**Bridge piece type** (claim 10): the piece's overlay type number (its position in the overlay type
list) is *t*₀ + (*x* mod 4) for one raw value *x*, where *t*₀ depends only on the pair of end markers
that built the bridge; the piece's overlay data is its place across the bridge, 0, 1 or 2 (during
map loading the overlay data stream, read afterwards, then sets every cell's data; S-0009, claim
13). With the pieces numbered *j* = 0 … 3*d* − 1 in the order of claim 10 (*j* = 3(step − 1) +
place across, the step counted from 1 next to the partner's end row), and *p* the stream position
of the first piece's raw value, piece *j* uses x<sub>*p*+*j*</sub> when no tile animation starts
among the pieces.

| End-marker pair (overlay type positions) | *t*₀ (overlay type position) | Piece types |
|---|---|---|
| 122 and 123 | 74 | 74 … 77 |
| 124 and 125 | 83 | 83 … 86 |
| 233 and 234 | 205 | 205 … 208 |
| 235 and 236 | 214 | 214 … 217 |

**Vein pattern values** (claim 10). For one raw value *x* per cell, with *c* the cell's vein
connection pattern (an integer 0 … 15, from the vein rules) and *s* its slope shape (S-0106; 0
is flat):

| Cell | Overlay data |
|---|---|
| The placed cell, flat | 48 + (\|*σ*(*x*)\| rem 3) ("magnitude first"): 48 … 50, or 46 for *x* = 2147483648 |
| A re-patterned or kept cell, flat | 3*c* + \|*σ*(*x*) rem 3\| ("magnitude after"): 3*c* … 3*c* + 2 |
| A cell set by a placement, sloped (*s* ≥ 1) | 51 + 2*s* + (*x* mod 2) |

One placement on a flat cell uses consecutive raw values: the placed cell's first, then one for each
re-patterned edge neighbour in the order map north, map east, map south, map west.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| State | 250 | values of the stream sequence (unsigned 32-bit) | Per stream: the 250 most recent values (claim 1) |
| Long lag | 250 | raw values | Claim 2 |
| Short lag | 147 | raw values | Claim 2 |
| Mixing rounds | 4 | rounds | Claim 3; constants in the Formulas table |
| Unit-interval range bound | 2147483646 | ranged-draw result (integer) | Upper bound of the ranged draw behind *u* |
| Unit-interval factor *c* | (2³⁰ + 1) / 2⁶¹ | dimensionless (double) | Double precision |
| Percent draw range | 0 … 99 | percent | Inclusive |
| Network checksum draws | 2 | raw values per tick | The original's network games only; none in a skirmish (claim 12); the multiplayer spec decides |
| Map-generator seed range | 0 … 65535 | seed value | Claim 13 |
| Network setup seed range | 0 … 32767 | seed value | The original's own setup screen (claim 8) |
| Mixing constants *P*₀ … *P*₃, *Q*₀ … *Q*₃ | Formulas table | unsigned 32-bit values | Claim 3 |
| Bridge end-marker positions | 122 … 125 and 233 … 236 | positions in the overlay type list | Claim 10; pairs 122–123, 124–125, 233–234, 235–236 |
| Bridge piece base types *t*₀ | 74, 83, 205, 214 | positions in the overlay type list | Formulas table *Bridge piece type* |
| Bridge pieces per cell step | 3 | pieces (one raw value each) | Claim 10 |
| Vein overlay position | 126 | position in the overlay type list | Claim 10; S-0009, claim 14 |
| Vein data base, flat placed cell | 48 | overlay data value | 46 for the one raw value of `quirks.random_magnitude_overflow` |
| Vein data base, sloped cell | 51 | overlay data value | Plus 2*s* and the lowest bit |
| Vein data step per connection pattern | 3 | overlay data values | Flat re-patterned cell: 3*c* plus the remainder |

## State and ordering

| Stream state | Event | Result |
|---|---|---|
| Any | Game start from the setup or menu screens | Both streams seeded with the game's seed; next raw value is x₀ (claim 8) |
| Any | Campaign continuation (next mission or restart) | No seeding; both streams continue (claim 8) |
| Running | A raw value is requested | Returns x<sub>n</sub>, advances to x<sub>n+1</sub> |
| Running | Clearing of the previous game begins (at a game start, before the map is read) | Scenario random stream suspended (claim 7; the original's mechanism: our engine takes no draws while clearing and need not have this state) |
| Suspended | A raw value is requested | Returns 0; no advance (the original) |
| Suspended | Clearing ends | Running again, with the state it had when the clearing began (the original) |
| Running | Saved game loaded | Scenario random stream state replaced by the saved state (claim 9) |

Observable ordering: because both streams start from the same seed, the first draws of a game from
the two streams are the same raw values when no other draw has been taken from either. The order
of draws within a tick is set by the specs of the systems that draw; every draw shifts all later
draws of the same stream, which is why those specs state counts and order.

**Draws during map loading.** The rows are groups of scenario-stream draws taken while a map loads,
in stream order: every draw of a row comes after the draws of the rows above it. It extends S-0009's
table *Start position resolution* and S-0011's claim 34 with the draws of claim 10; within a row the
order is the one given. Rows marked "pointer" belong to a spec that does not yet state them.

| Group | Draws, in stream order | Owner |
|---|---|---|
| S-0009 early phase | list 1, list 2, random positions | S-0009, claim 20 |
| Initial fill | one per visited cell, water fill only | S-0009, claim 6 |
| Tags of cell tags | the draws of each tag created by a cell tag | S-0011, claim 34 |
| Overlay type stream | in the stream's order: the bridge pieces of each bridge completed by an end marker (3 per cell step) and the creation draws of cell animations | claim 10; animation specs (pointer) |
| First recalculation pass | the creation draws of tile animations (S-0010, claims 12 and 19). In this pass and in everything before the second pass, a tile animation starts at the first recalculation of its cell; a cell recalculated earlier in loading (for example by an overlay placement or a bridge piece) starts it there, among that group's draws (low confidence) | animation specs (pointer) |
| Vein candidates | for each candidate cell that passes the test, in reverse visiting order: the vein pattern draws of claim 10 | S-0009, claim 14; claim 10 |
| Terrain objects | none found (medium confidence) | — |
| Pre-placed objects | kind by kind as in S-0203, claim 3 (vehicles, aircraft, infantry, buildings), each kind in entry order; for each object: its creation draw, then its tag's draws, then its placement's draws. A building's upgrades are created after that building's tag and take their own creation draws | claim 10; S-0011, claim 34; placement specs (pointer) |
| Smudges | none found (medium confidence) | — |
| Second recalculation pass | the creation draws of the tile animations that S-0010's second pass (claim 12) starts; these replace the ones started earlier in loading (S-0010, claim 19) | S-0010, claims 12 and 19; animation specs (pointer) |
| S-0009 late phase | list 3, late re-assignments | S-0009, claim 20 |
| Starting units | house by house: each starting unit's creation draw and its placement's draws, then any further draws S-0009 lists for this step | claim 10; S-0009; starting-unit rules (pointer) |
| Later loading steps | crates and similar (pointer) | S-0009 |
| Fence lines | for each fence post on the map that can link to a matching post: the creation draws of the fence sections built between them, and the recompute draws of claim 10 for vein cells the line crosses. Stock data has no fence posts, so this row is empty with stock data (low confidence) | fence rules (pointer); claim 10 |
| End of loading | the tags created at the end of loading, in registration order | S-0011, claim 34 |

S-0009 (its draft revision 5) points to this table for the rows between the initial fill and the
late phase. S-0011's table *Load-time draw order* lists no draws for the overlays and leaves the
vein candidates open; it needs a matching addition (Open questions). Creating terrain objects,
smudges and tunnels from the map was not found to take draws (medium confidence). Neither S-0009
nor S-0011 lists the *Fence lines* row; it matters only for data with fence posts. Where each group
sits is `unmeasured`, apart from what scenario R8 shows (the bridge and vein groups' own draws, and
that each bridge or vein site follows the one before it in the stream, and that the overlays of
R8 C take no draw): a group's position shows
only as a shift of later draws.

## Edge cases and quirks

- **`quirks.ranged_draw_half_span`** (default: on, the original). A ranged draw whose bounds differ
  by exactly 2147483647 takes no raw value and returns the lower bound plus 2147483648, wrapped,
  which is outside the requested range. Off: the general rule with 31 kept bits, one raw value,
  result inside the range. With either setting the importer warns where data can produce such a
  span (claim 5): the switch keeps the original's result for faithful replays, while the warning
  tells the author that the data reaches a case the original handles wrongly.
- **Spans of 2147483648 or more** (departure, no switch). The original hangs (claim 5). Our engine
  draws with all 32 bits kept and rejection; the importer warns where data can produce such a
  span.
- **`quirks.random_magnitude_overflow`** (default: on, the original). In the "magnitude first"
  signed remainder (claim 6), the raw value whose signed reading is −2147483648 keeps its negative
  sign, so the remainder lies in −(*n* − 1) … 0 instead of 0 … *n* − 1, for that one raw value in
  2³². Off: the magnitude is taken as 2147483648 and the remainder is never negative. The topic
  specs that use this form refer to this switch. One user is the flat placed vein cell of claim 10,
  whose overlay data becomes 46 instead of 48 … 50 for that raw value. That cell is then not a
  pattern centre (Terms), so a later placement or recompute treats it as an ordinary flat vein
  cell.
- **Seed from the host's clock** (departure, claim 8): the seed is a game-setup value in our engine.
- **Terrain-image pattern kept for the session** (departure, no switch; claim 11). The original
  builds S-0010's 8 × 8 pattern once per session, so the ground images of a game depend on earlier
  games in the same program run and can differ between the machines of a network game. Our engine
  builds the pattern at every map load, at the point S-0010 names, from the unsynchronised stream
  seeded for that game, so the images depend only on the game's seed and setup. In the first game
  of a session the two agree as long as the stream has given the same draws before the pattern is
  built; later games can show other ground images. Nothing else changes, since the pattern never
  reaches game state.
- **Identical starting states.** Both streams start each game from the same seed. A mod cannot
  change this, and our engine keeps it: it costs nothing and keeps the unsynchronised stream's
  first values the same as the original's for a given seed.

## Import mapping

The streams read no rules or art keys. Claim 10 names overlay types by their position in the
rules' overlay type list (the end markers 122 … 125 and 233 … 236, the vein overlay 126 and the
bridge piece types of the Formulas table), as S-0009 does, so the list order is part of the data
format; the positions are fixed in the original, whatever types the data puts there. The
map-generator seed (claim 13) is a setting of the random-map setup screen, not a rules key. The
game seed is a game-setup value (claim 8).

## Measurement scenarios

All scenarios use the measurement setup. Scenarios R2, R3, R5, R6, R7 and R8 use a *state recorder*: a
read-only program that reads, without changing anything, the running original's seed of the current
game and the current state of the scenario random stream: the 250 values x<sub>N−250</sub> …
x<sub>N−1</sub>, seed values included, where *N* is the number of raw values taken since the
seeding; the recorder gives *N* only modulo 250 (its *position*). It reads the unsynchronised
random stream's state in the same way (S-0010's scenario V2 part B uses this). This is a recorder
in the glossary's sense: it fixes no random state, and every scenario takes the seed the game chose.
The recorder is part of the measurement setup (the project lead has approved it); it is a read-only observer of the original.

- **R1, recurrence on the water fill (claims 2, 4).** Use S-0009's scenario M2, sub-test "visiting
  order": a 15 × 15 map (435 legal cells) with the fill set to water and no terrain stream, played as
  the first game of a fresh program run, so each cell shows tile 0 … 3 equal to the two lowest bits
  of one raw value (a power-of-two range, one raw value per cell, claim 4). Number the cells 0 … 434
  in S-0009's visiting order. 1 run. *Expected:* for every *k* from 250 to 434, the value of cell *k*
  equals the exclusive-or of the values of cells *k* − 250 and *k* − 147 (185 equations). *Falsified
  if* any equation fails. (A wrong lag pair fails about three in four of them.)
- **R2, seeding function and recurrence (claims 1, 2, 3, 8).**
  (a) *Skirmish.* Any skirmish map, 3 fresh launches plus 2 further skirmish games in the same
  session. With the state recorder, read the seed and the stream state at the first tick of each
  game, and note the host's wall-clock time and its uptime in milliseconds at that moment. Compute
  the 250 seed values F(seed, *k*) and advance the recurrence step by step. The recorder gives the
  number of raw values taken only modulo 250 (its position): find the smallest count *N*, congruent
  to the position modulo 250, for which the computed state equals the recorded one.
  *Expected:* such an *N* exists (well below 100000) in every game; the unsynchronised stream's
  state is the same seed's state advanced by some count; for the games of one session, the
  difference of two seeds, modulo 2³², equals the wall-clock time between their first ticks in
  milliseconds, within 10 seconds (the difference in loading time). A seed not taken from a
  millisecond counter lands in that window with chance about 5 in a million.
  *Falsified if* no *N* up to 100000 matches in some game; the unsynchronised stream's state is not
  the same seed's state advanced by any count up to 100000; or a seed difference falls outside the
  window.
  *Secondary expectation* (for the counter's starting point only): each seed equals the host's
  uptime in milliseconds at the first tick, modulo 2³², minus at most 60 seconds of loading time. If
  it does not, only the medium-confidence part of claim 8 about the counter's start is wrong.
  (b) *Local network.* 3 local-network games of two machines, with the state recorder on both.
  *Expected:* in each game both machines give the same seed, in 0 … 32767, and the same stream
  state at the first tick. If the seed were any 32-bit value instead, all three seeds would fall
  in 0 … 32767 with chance 2⁻⁵¹, so three games confirm the range.
  *Falsified if* the seeds or states of the two machines differ in some game, or a seed exceeds
  32767.
  (c) *Campaign continuation.* Win a short campaign mission and let the next one start; read the
  seed and state at the last tick of the first mission and at the next mission's first tick. Then
  lose a short campaign mission and restart it without returning to the menus; read the seed and
  state at the lost mission's last tick and at the restarted mission's first tick. 1 run of each.
  *Expected:* in both, the seed is unchanged and the state is the previous mission's final state
  advanced by some count. *Falsified if,* in either, the seed changes or the state matches a fresh
  seeding.
- **R3, saved games (claim 9).** A small test map with one human house and ten infantry of a type
  with idle animations, no computer players, no crates and no other moving objects, with the state
  recorder.
  *Procedure:* at tick 300, read the recorder, save the game, and read the recorder again. Let that
  run continue without loading for 1800 ticks (2 game minutes), video-recorded (run 0). Then load
  the save twice (runs 1 and 2), read the recorder right after each load, give no orders and
  video-record 1800 ticks.
  *Expected (required):* the stream state and position read just before the save, just after the
  save and right after each load are all equal: saving takes no draw, and loading restores the
  saved state rather than seeding anew. *Expected (secondary):* the same infantry perform the same
  idle actions (animation or turn) at the same ticks in runs 0, 1 and 2.
  *Falsified if* any of the recorded states differs, or any idle action differs in kind, unit or
  tick between the runs.
- **R4, seed differs per launch (claim 8)** *(statistical)*. Use the 10 × 6 water map (114 legal
  cells) of S-0009's scenario M2, sub-test "distribution"; R1 uses the other water map of that
  scenario, the 15 × 15 map of its sub-test "visiting order". 40 fresh launches (the distribution
  sub-test's launches can serve). *Expected:* no two launches show the same 114-cell pattern (a repeat
  has probability far below 1 % if the seed changes per launch). *Falsified if* any two launches
  repeat the pattern, or the pattern is the same every launch.
- **R5, range rule, unit-interval value and the creation draw (claims 4, 6, 10).** (a) A test map
  with no computer players and nothing that moves or animates. With the state recorder, read the
  stream position at every tick. Control interval: for the first 300 ticks the map holds no
  infantry. *Expected:* the position does not advance in any of those ticks (this shows that no
  other system draws every tick on this map). *Falsified if* it advances, in which case the
  scenario must first remove the cause. Then a trigger places one infantry of the human house that
  has idle animations; leave it standing idle for 3 game minutes. From the recorded state, compute
  the raw values that each idle attempt takes (S-0107, claim 14: a timer draw R(0, 2147483646), then
  *r* = R(0, 10), then δ = R(0, 7) when the outcome is a turn) under claim 4's rule, including
  rejected values. *Expected:* at every tick where the position advances, it advances by exactly
  the predicted count; the infantry's visible outcome matches the predicted *r* and δ; and the next
  idle attempt comes at the tick that S-0107's timer formula gives from the predicted unit-interval
  value. *Falsified if* any advance differs from the prediction (for example by counting one raw
  value per ranged draw, or none for rejections), or an attempt comes at another tick. (b) At the
  tick the infantry is placed, *expected:* the position advances by the creation draw (one) plus
  whatever that infantry's creation draws according to its specs. *Falsified if* the advance is one
  less than that.
- **R6, unsynchronised draws stay out of game state (claim 11).** The map and recorder of R5 (a),
  with two infantry instead of one. 4 runs of 2 game minutes: in 2 runs keep the infantry selected
  and click them repeatedly so that they give voice responses (each response takes draws from the
  unsynchronised stream); in 2 runs never select them. *Expected:* in every run the per-tick
  advance of the scenario random stream equals R5's prediction from the idle attempts alone, and
  ticks in which a voice response starts show no extra advance. *Falsified if* any voice response
  coincides with an advance that the idle attempts do not explain.
- **R7, network checksum draws (claim 12; kept for the future multiplayer spec, not required of the skirmish engine).** Two machines in a network game on R5 (a)'s map,
  with the state recorder on one machine (it reads that machine's stream; both machines hold the
  same state). During the control interval, *expected:* the position advances by exactly 2 in
  every tick, apart from occasional ticks with one more ranged draw R(0, 2) (whose raw values the
  recorded state predicts). Repeat on the same map as a skirmish: *expected:* no advance.
  *Falsified if* the network game's advance per tick is not 2 (allowing the extra draws), or the
  skirmish shows any advance in the control interval.

- **R8, overlay placement draws (claim 10, map-loading table)** *(statistical runs)*. No run
  needs a chosen seed: each run uses the seed the game chose, which the state recorder reads.
  *Setup common to all parts:* a skirmish on a small purpose-made map of a theater whose clear and
  water tiles name no tile animation (check in the theater data), with the state recorder; no
  overlay other than the ones named. Before running, check in the rules data that the end-marker,
  vein and other overlay types used have no cell animation; otherwise the part that uses them is
  inconclusive. *Procedure for each run:* read the seed and the state at the first tick, find *N* as
  in R2 (a), compute x₀ … x<sub>*N*−1</sub> from the seed (claims 2 and 3), and read every piece's
  or vein cell's overlay type and data from a saved game made at the first tick.
  - (A) *Bridges.* Two bridges over water, each a pair of matching end markers of the family at
    positions 122 and 123 with *d* = 4 water cells between their end rows; both markers of the
    first bridge come before both markers of the second in the overlay type stream (S-0009, claim
    12). 5 runs, then 5 more with the pair 233 and 234. *Expected:* there is a position *p* such
    that the first bridge's 12 pieces, in the order of claim 10, have overlay types *t*₀ +
    (x<sub>*p*+*j*</sub> mod 4), *j* = 0 … 11, with *t*₀ from the Formulas table (74, or 205 for the
    second family); and the second bridge's 12 pieces match x<sub>*p*+12</sub> …
    x<sub>*p*+23</sub> in the same way. A wrong position matches one bridge with chance 4⁻¹², so
    the match is unique. The second bridge following straight on the first shows that the first
    took exactly 3*d* = 12 raw values. *Falsified if* no *p* matches in some run, the base type
    differs, the pieces match in another order, or the second bridge starts at another position
    than *p* + 12. The pieces' overlay data is not checked, because the map's overlay data stream
    sets it afterwards (S-0009, claim 13). *Inconclusive if* no bridge appears: the marker types
    are then not created on this map (S-0009, claim 12).
  - (B) *Veins.* Four vein overlays (position 126) with overlay data 48, each on a flat clear cell
    whose four edge neighbours are flat, clear and empty, every two sites at least four cells apart
    in x or in y. 5 runs. *Expected:* there is a position *p* such that, taking the sites in
    the reverse of S-0009's visiting order and each site's raw values in the order of the Formulas
    (placed cell, then map north, map east, map south, map west), the 20 raw values
    x<sub>*p*</sub> … x<sub>*p*+19</sub> give every cell's data by the *Vein pattern values*: the
    placed cell's data is 48 + (|*σ*(*x*)| rem 3) (46 for the one raw value of
    `quirks.random_magnitude_overflow`), and each neighbour's data, taken modulo 3, is
    |*σ*(*x*) rem 3| (the neighbour's 3*c* depends on the vein rules and drops out). A wrong
    position matches with chance 3⁻²⁰, so exactly one *p* ≤ *N* − 20 should match. Each site's 5
    values following straight on the previous site's shows that a site with four empty flat
    neighbours takes exactly 5 raw values, so this also tests that all four such neighbours are
    re-patterned (claim 10, medium confidence). *Falsified if* no *p* matches in some run, or the
    match needs a gap or another neighbour order.
  - (C) *Overlays that take no draw.* The map of part A, with these overlays added on land cells
    that lie off both bridges' lines and end rows, and in the overlay type stream after the
    second marker of the first bridge and before the first marker of the second bridge: a run of 10 wall overlays, 10 ore cells and 10 gem
    cells. 5 runs. *Expected:* as in part A, the second bridge's pieces match
    x<sub>*p*+12</sub> … x<sub>*p*+23</sub>, so the overlays placed between the two bridges took
    no raw value (their cells hold no tile animation, by the setup). *Falsified if* the second
    bridge starts at a later position than *p* + 12.

## Open questions

- Whether any of the unsynchronised random stream's users that were classified only briefly
  changes game state. In internet games, two draws are taken under a condition on the local
  player's house (the same kind of condition as claim 12's extra draw) and could not be tied to a
  feature; they look like a network or statistics feature, not game state, but this is low
  confidence.
- Whether some of the building animation draws from the scenario random stream are taken when a
  building is drawn rather than in the tick. If so, the stream would depend on what is on screen.
  This needs the building animation spec to establish.
- What state the unsynchronised random stream has after a saved game is loaded (claim 9).
- The condition under which a network game takes the extra unused draw of claim 12 (for the future multiplayer spec).
- **Specs that must add the draws of claim 10.** Until they do, this spec and its map-loading table
  are the only statement of these draws:
  - S-0009: its draft revision 5 in this batch already points to this spec's table for the groups
    between the fill and the late phase, and gives the starting units' creation draws. Its claim 12
    ("placement effects") should still say that bridge completion and cell animations draw, and its
    claim 14 should give the candidates' draws. Claim 14 also says a passing candidate "gets a
    veinhole"; what was found is that the cell gets the vein overlay again as a pattern centre
    (overlay data 48 … 50) and its edge neighbours become vein cells (claim 10), while the veinhole
    monster comes from another overlay type. S-0009 should be checked on this point.
  - S-0011: claim 34 already puts a pre-placed object's creation draw before its tag's draws. Its
    table *Load-time draw order* lists "none known" for the overlays and "not established" for the
    vein set-up; it should take the bridge and cell-animation draws and the vein candidates' draws
    of claim 10 from this spec.
  - The production specs (none yet): a produced object is created, with its creation draw, when its
    production begins.
  - The animation specs: the creation draws of cell animations and tile animations at map load,
    and exactly when a tile animation starts before the second pass (the first recalculation of its
    cell, which may come at an overlay placement, at a bridge piece or at the first pass).
  - S-0010 (claim 14, its Edge cases and its Open questions) still says that claim 11 here does not
    list the map-load use; claim 11 lists it now. S-0010 should point to claim 11 and to the
    departure that rebuilds the pattern at every map load (Edge cases here).
- Whether, in the original, the machines of a network game build the same terrain-image pattern in
  the first game of a session (that needs the same unsynchronised draws before the build on every
  machine). Low confidence either way; it does not affect our engine (claim 11).
