---
id: S-0202
title: Random streams (generator, seeding, ranged draws, which systems draw from which stream)
topic: simulation
revision: 62
sources-class: [decomp, measurement, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-03
measurements: [M-0008]
requests: [Q-0018, Q-0046, Q-0101, Q-0182, Q-0217, Q-0327, Q-0711, Q-1562, Q-2336]
supersedes: []
---

# S-0202: Random streams (generator, seeding, ranged draws, which systems draw from which stream)

## Summary

The original runs several independent copies, or *streams*, of one pseudo-random generator; this
spec gives the generator exactly, the rule that turns raw values into ranged draws and their derived
forms, when and from what the streams are seeded, and which game systems draw from the scenario
random stream and which from the unsynchronised random stream. It also states three kinds of draw that
no feature spec lists yet: the creation draw of an object, the draws of overlay placement (with the
low-bridge and vein placement rules they depend on) and the creation draws of an animation
(claim 10).

**Joint hand-off checks.** Claim 14 agrees with S-0009's loading-order table and
S-0412 claim 14: game-start tile-animation draws follow starting units, crates,
secret-lab choices and end-of-loading tags. The cleanup and low-bridge refinements
remain in force. S-0109 claims 14 and 25 own the middle-event gates and sprite-derived μ.
Claim 15 here supplies the downstream particle and smudge creation draws at map load
and during play, including immediate middle events (Q-0217).
S-0404 claim 11 and P11 B/C, with S-0203 claim 8, supply production-start draw assignment
at the causing action, distinct from payment priority: turn-pass starts precede order-boundary
starts, and completion alone creates no replacement-start draw batch. The between-house
order-boundary extension remains an unmeasured design rule. S-0701 claims 13, 14, 20 and 25 supply sound-instance and
feedback draws on the unsynchronised stream and the lightning-sound exception on the
scenario stream. S-0401 claim 20 and S-0011 claim 36 agree with S-0109 claim 13:
the creation start event can react, while later trigger-played start events are suppressed.
Check these revisions together, including claim 15's 6-bale removal under S-0401 claim 3:
a stage-6 resource cell becomes stage 0, not empty. S-0011 claim 36 covers the specified
middle-event composition during loading and play, while subsequent particle updates remain
outside it and must not be treated as draw-free. Event timing, sprite-derived μ and the height gate remain S-0109's.
S-0414 claims 19 and 23 and V18 agree with claim 16 here: post-game score requests use
the scenario stream in listed-house order after simulation stops, and equal bounds take
no raw value. Check this addition with the same hand-off.

## Terms

Shared terms used here are defined in the glossary: *scenario random stream* (with *draw* and
*ranged draw*), *unsynchronised random stream*, *game*, *game start*, *session*, *skirmish*,
*object*, *tick*, *departure*, *importer warning*, *quirk switch*, *recorder*, *measurement setup*
(with its *random-state recorder*), *multiplayer game, network game*, *round-toward-zero mode*. In
addition:

- **Stream.** One copy of the generator with its own state. The original has three that matter
  here: the scenario random stream, the unsynchronised random stream, and the *map-generator stream*
  (claim 13).
- **Raw value.** One output of a stream: an unsigned 32-bit integer, 0 to 4294967295. Taking a raw
  value advances the stream by one. Where a spec reads a raw value as a signed number, it is the
  two's-complement reading, −2147483648 to 2147483647.
- **Stream sequence.** The raw values a stream produces after seeding, numbered x₀, x₁, x₂, … in the
  order they are taken. The 250 *seed values* x₋₂₅₀ … x₋₁ are the values the seeding function
  gives (claim 3); they are never returned as raw values themselves.
- **Draw allocation order.** The assignment of successive stream values to effects, not an
  implementation's internal work schedule. An earlier group uses the incoming stream state.
  A later group uses the state left by the earlier groups, including their rejections and
  conditional draws. Thus changing an earlier group can change a later effect's random outcome.
  Ordered tables below specify this observable assignment; they require neither a particular
  data structure nor a particular schedule for computing values whose assignment is unchanged.
- **Seed.** The 32-bit unsigned integer from which a stream's 250 seed values are computed.
- **Ranged draw R(a, b).** A draw with two signed 32-bit bounds *a* and *b*. Claim 4 gives
  its inclusive-range result from zero or more raw values; claim 5 gives the exceptional spans,
  including a result outside that range. This refines the glossary's *ranged draw*: one ranged
  draw may take no raw value, one, or several.
- **Unit-interval value.** A double-precision number from 0 to 1 derived from one ranged draw
  (claim 6, Formulas).
- **Percent draw.** The ranged draw R(0, 99).
- **List pick.** A raw value, read as unsigned, reduced modulo a list length *n* to an index 0 … *n*−1.
- **Suspended.** A state of a stream in which every raw value it is asked for is 0 and its state
  does not change (claim 7).
- **Overlay type stream, overlay data stream.** The map's stream of overlay types and its stream of
  overlay data values, as S-0009 uses them (claims 12 and 13). Pointers only.
- **Visiting order.** The order in which S-0009 visits a map's cells (S-0009, claim 3, and its
  scenario M2, sub-test "visiting order"). A pointer only.
- **Tile animation.** The animation that a cell's tile names and that the cell starts when it is
  recalculated (S-0010, claim 19). It is a *terrain animation* in the glossary's sense.
- **Resource glint.** An animation selected for a resource cell during game-start setup by
  the game-wide glint animation and frequency fields (claim 14). Not a tile animation.
- **Cell animation.** The animation attached to an overlay type and started by ordinary
  placement, as the glossary and S-0009 define it. Its position is displaced from the overlay
  cell, and low-bridge writes do not start it (claims 10 and 14).
- **Edge neighbour.** One of the four cells one straight step away from a cell: the cells in step
  directions 0, 2, 4 and 6 (glossary *step direction*), that is map north, map east, map south and
  map west.
- **End row, bridge piece.** For the low bridges of claim 10: the end row is the row of three cells
  across the bridge's line, centred on the end marker's own cell, that the marker writes where it
  is placed; a bridge piece is one of the three cells across of each cell step between two end
  rows. Claim 10 and the Formulas table *Low-bridge end markers* give which three cells, when they
  can be written and how a marker finds its partner. What a finished bridge does in play belongs
  to the bridge rules, which have no spec yet; a pointer only.
- **Line, search direction.** Each end marker builds along a fixed *line*, the map's x axis or its
  y axis, and looks for its partner in a fixed *search direction* along that line, one of the
  step directions map north, map east, map south and map west (glossary *step direction*).
- **Vein, vein cell.** A cell holding the vein overlay, the overlay type at position 126 of the
  overlay type list (claim 10). "Loses its vein" means the cell no longer holds any overlay. The
  vein rules, which have no spec yet, own what veins do; a pointer only.
- **Pattern centre.** A vein cell whose overlay data is 48 or more. A placement (claim 10) gives this
  to the placed cell, flat (48 … 50) or sloped, and also to every sloped neighbour it sets (their data
  is 53 or more, formula *Vein pattern values*). The flat cells that a placement sets or
  re-patterns, and those that a recompute keeps, have data below 48 and are not pattern centres. Neither is a flat placed cell that gets 46 under
  `quirks.random_magnitude_overflow` (Edge cases).
- **Veins flag.** A flag of an overlay type in the rules data (Import mapping) that marks it as part
  of the vein family. The vein overlay itself is identified by its position 126, whatever its flag
  says. The flag matters for the other types that carry it and, in exactly two places of claim 10,
  for the vein overlay too: (i) in the vein placement test, a cell or edge neighbour that already
  holds the vein overlay passes only when the vein overlay type has the flag; (ii) in the vein
  connection pattern, a sloped neighbour holding a pattern centre counts only when the vein
  overlay type has the flag (the table *Vein connections* in claim 10).
- **Vein candidate.** At map load, a cell that holds the vein overlay with overlay data 48 or more
  once the overlay data stream has been read (S-0009 calls these cells vein seeds; this spec keeps
  the word *seed* for a stream's seed). Claim 10, *Map loading*, gives what a candidate holds
  when its placement is tried.
- **Vein placement test.** The condition a cell must meet for a vein placement to take effect there
  (claim 10, which states it).
- **Vein connection pattern.** The integer 0 … 15, written *c*, that claim 10 derives for a cell
  from which of its four edge neighbours count as vein connections; it enters the formula *Vein
  pattern values*.
- **The veinhole monster.** The map object that the rules data's veinhole-monster overlay type creates
  where it is placed: the stationary object that grows veins around it during play, which S-0009
  (Terms) calls the veinhole. What it does belongs to the vein rules; a pointer only. Creating it
  takes no draw of its own (claim 10).
- **Fence post, fence line.** A fence post is a building whose type the rules data marks as a post.
  Between two matching fence posts within range the original builds a line of fence-section
  buildings, the fence line. This rule applies when the imported rules data defines matching
  fence-post building types. The fence rules have no spec yet, a pointer only.
- **Visceroid flags.** Two flags of a vehicle type in the rules data (Import mapping). A type has
  "either visceroid flag" when at least one of the two is on.
- **Animation terms of S-0109.** *Bouncing type* (S-0109, Terms), *random rate* and *random rate
  pair* (S-0109, claim 3), *loop delay* (S-0109, claim 11), *start event* (S-0109, claim 13) and
  *middle event*, and the quantity μ of S-0109 claim 14, mean what S-0109 defines. Pointers only.
- **Stand-in cell.** The cell that the original writes to, and reads, in place of a cell outside
  the map, in the sense of S-0009 (claim 44). A pointer only.

## Behaviour

### The generator

The engine requirements below cover skirmish only. Network games, campaigns and the
original's recording modes are retained as deferred context, not implementation requirements.

1. **State and output.** A stream's state is its 250 most recent values x<sub>N−250</sub> …
   x<sub>N−1</sub>, seed values included, where *N* is the number of raw values taken since the
   seeding; by claim 2 they determine every later raw value. Each raw value is the
   recurrence's result of claim 2 itself, all 32 bits of it, with no bit dropped or changed. A
   stream returns the same sequence for the same seed on every machine.
   - Backing: `measured` on one host (M-0008: every logged draw of the eight runs it describes,
     replayed from its seed on this generator, matches in result and in index steps); that the
     sequence is the same on every machine is `measurable` (scenario R2 (b), the local-network
     games) and not measured: M-0008 compares no two machines
   - Confidence: high

2. **Recurrence.** For every n ≥ 0, x<sub>n</sub> = x<sub>n−250</sub> ⊕ x<sub>n−147</sub>, where ⊕
   is bitwise exclusive-or and x₋₂₅₀ … x₋₁ are the seed values. So the first 147 raw values, x₀ …
   x₁₄₆, combine two seed values each; x₁₄₇ … x₂₄₉ combine an earlier raw value and a seed value; and from x₂₅₀ on every raw value combines two earlier raw values. Consequence: any bit position of the raw values,
   including the lowest two, obeys the same relation on its own, so the low bits that a small
   power-of-two range keeps (claim 4) satisfy x<sub>n</sub> = x<sub>n−250</sub> ⊕
   x<sub>n−147</sub> too.
   - Backing: `measured` (M-0008, the same replay); `measurable` (scenarios R1 and R2)
   - Confidence: high

3. **Seeding function.** Seeding a stream with seed *s* sets seed value x<sub>k−250</sub> = F(*s*,
   *k*) for *k* = 0 … 249, where F is the four-round mixing function of the Formulas. Seeding
   discards all previous state, and the next raw value taken is x₀. Test values are in the
   Formulas.
   - Backing: `measured` (M-0008: the replay from seed 1000 and from three clock seeds), and
     `measurable` (scenario R2)
   - Confidence: high

### Ranged draws and derived forms

4. **Range rule.** Let *a* and *b* be signed 32-bit integers, *lo* = min(*a*, *b*),
   *hi* = max(*a*, *b*), and *d* = *hi* − *lo*, calculated exactly as an integer in
   0 … 4294967295. The result and stream consumption are:

   | Condition | Result | Raw values consumed |
   |---|---|---|
   | *d* = 0 | *a* | none |
   | 1 ≤ *d* ≤ 2147483646 | Let *m* be the least positive integer such that *d* < 2<sup>*m*</sup>. Let *x* be the first raw value, beginning at the stream's next value, with *x* mod 2<sup>*m*</sup> ≤ *d*. The result is *lo* + (*x* mod 2<sup>*m*</sup>). | Every value from the next raw value through *x*, including rejected values |
   | *d* ≥ 2147483647 | As specified in claim 5. | As specified in claim 5. |

   For 1 ≤ *d* ≤ 2147483646, each integer in *lo* … *hi* is equally likely.
   If *d* + 1 is a power of two, exactly one raw value is consumed. Otherwise the expected
   number consumed is 2<sup>*m*</sup> / (*d* + 1), an exact rational less than 2.
   For example, R(0, 10) retains the lowest 4 bits, rejects values 11 … 15, and consumes
   16/11 raw values on average. The exceptional spans are defined separately in claim 5.
   - Backing: `measured` (M-0008: the replay matches the index steps that each rejected value of
     every logged ranged draw takes, and in M-0008's control interval the logged draws with equal
     bounds take no raw value), and
     `measurable` (R1 and R5 for ordinary ranges, R13 for equal and reversed bounds)
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
     draw consumes every raw value from the stream's next one up to that one. This finishes
     after 1 … 251 raw values for every stream state obeying claim 2. If the first 250 values
     all fail, each has its highest bit set. The next value combines the first and the 104th
     of those values by exclusive-or, so its highest bit is clear and it is at most 2147483647,
     which passes. This bound is a mathematical consequence of the generator, not a new
     fallback, draw cap or change to its output. The suspended stream's zero also passes.
   - A spec whose imported fields can make a span of 2147483647 or more says so, and the importer
     warns for such data, naming the field and the span; the game still runs, with the result
     above for each span.
   - Backing: `unmeasured` for the original's exceptional results and hang (no scenario part
     exercises either span). The departure and its finite-consumption bound are design and
     a mathematical consequence of claim 2, respectively, not results measurable in the original.
     Claim 14's glint frequency can reach these spans, but R13 deliberately does not run them
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
     specs measure in their own scenarios, except the negative "magnitude first" remainder for the
     raw value whose signed reading is −2147483648 (Edge cases), which is `unmeasured`: it needs
     that one raw value in 2³², and the measurement setup cannot choose the seed (claim 8), so no
     scenario can practically make it occur. That this list covers every form in use is
     `unmeasured` too: no finite number of runs can confirm a list of every form in use
   - Confidence: high (the forms); medium (that this list covers every form in use)

7. **No draws while a game is cleared.** Every game start, from the menus or as a campaign
   continuation, clears what the previous game left before the map is read. Clearing a game takes
   no draws from the scenario random stream: the stream has the same state when the clearing ends
   as when it began. In the original the clearing comes after the seeding of claim 8 on a start
   from the menus; because the clearing takes no draws, nothing observable in the scenario random
   stream depends on that order. Our engine takes no draws from the scenario random stream while it
   clears a game. (In the original this holds because the stream is suspended, Terms, while the
   clearing runs: anything that asks it for a raw value then receives 0, so a ranged draw with span 0 … 2147483646 under
   claim 4 returns its lower bound, and the stream does not advance. Our engine need not have this
   state.)
   - Backing: `unmeasured` (no scenario samples both ends of clearing separately from map
     loading; the first-tick state alone cannot assign earlier consumption to clearing
     rather than loading)
   - Confidence: medium

### Seeding and saved games

8. **Seeding at game start.** When a game is set up from the game's setup or menu screens (every
   skirmish, every network game, and a campaign mission started from the menus), both the scenario
   random stream and the unsynchronised random stream are seeded with the same seed before the map
   is read. The two streams therefore start in identical states: their raw-value sequences are the
   same, and only the number of values each has taken differs once their users diverge. Where the
   seed comes from:
   - *Skirmish and campaign mission:* the original uses the host's millisecond tick count, modulo
     2³², read when the game is set up. That count runs from about the start of the host computer
     (on the measured host it trailed the uptime by about 0.5 s). So the seed differs from game
     to game and from launch to launch, and the seeds of two games started in one session differ
     by the time between their starts, in milliseconds.
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
     reseeded: they continue from the state the previous mission left. Campaigns are out of scope
     for now: this is recorded as the original's behaviour, and the future campaign spec decides
     what our engine does.

   Backing and confidence by part:

   | Part | Backing | Confidence |
   |---|---|---|
   | Skirmish: same seed for both streams (the unsynchronised stream, seeded before at program start, is reseeded at game start) | `measured` for a game started without the setup menus (M-0008: three launches, each stream's state at the first tick is the seed's state advanced by a count); `measurable` from the menus (R2 a); the earlier seeding at program start rests on claim 11 (R9 d) | medium (the launches did not use the setup menus) |
   | Skirmish: seeding precedes map reading | `unmeasured` (R2 a reads only at the first tick and does not isolate the seeding checkpoint before map reading) | medium (`unmeasured`) |
   | Skirmish: the seed is a millisecond counter | `measured` for a game started without the setup menus (M-0008: in three launches the seed equals the host's millisecond tick count read just before the seeding, and seed differences match wall-clock differences within 2 ms); `measurable` from the menus (R2 a); the reduction modulo 2³² is `unmeasured` and rests on the formula: the measured seeds are far below the wrap | medium (the launches did not use the setup menus) |
   | Skirmish: seeds of games started in one session differ by the time between their starts | `measurable` (R2 a's games of one session; R4 for the change between launches) | medium (it follows from the seed being the millisecond counter, rated medium above because the launches did not use the setup menus) |
   | Skirmish: the counter counts from about the start of the host computer | `measured` as far as the host layer passes the counter on (M-0008: the seed equals the host's millisecond tick count, which was the host's uptime minus about 0.5 s) | medium (the measurement setup runs the original on a host layer that supplies the counter) |
   | Local network: one shared seed, in 0 … 32767 | `measurable` (R2 b) | medium |
   | Internet game and other setup programs | `unmeasured` (the measurement setup has no online service or other setup program) | medium |
   | Playback uses the recording's seed | `unmeasured` (the recording and playback modes are development features outside the measurement setup, as in claim 12) | medium |
   | Recording and playback modes skip the clock | `unmeasured` (same reason) | low |
   | Campaign continuation: no reseeding | `measurable` (R2 c) | medium |
   - Backing: `measured` (M-0008) and `measurable` (scenarios R2, R4) for the parts so marked
     above; the rest `unmeasured`, with the reasons given above
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

10. **Scenario random stream: game-state decisions and more.** Except for low-bridge repair's
    piece choices on the map-generator stream (claim 13), random decisions that can affect game
    state draw from the scenario random stream: for example combat
    (projectile spread, scatter, damage-related picks), unit behaviour (idle actions, S-0107),
    computer-player decisions, map loading and start positions (S-0009), resource growth and
    spread (including the vein patterns below), random map events, crates and triggers. Several
    effects that look like presentation also draw from it: animations, particles and particle
    systems, debris, smudges, and the overlay placement effects below. Post-game winner-score
    bonuses also use it after simulation stops (claim 16). Consequence for our engine:
    these presentation-like effects contribute to the deterministic simulation's draw allocation.
    Omitting one shifts the values assigned to later choices and can change gameplay outcomes.
    Three kinds of draw are stated here because no feature spec lists them yet:
    - **The creation draw.** Every creation of an infantry, vehicle, aircraft or building object
      takes one raw value, whatever else that creation draws, at the moment the object is created.
      Creating an object is not the same as its entering play (S-0203): the draw is taken even for
      an object that never enters play.
      **Creation-derived report value (Q-0327).** Our engine retains the low 16 bits of
      this existing draw as an unsigned integer *r* = *x* mod 65536, in 0 … 65535,
      where *x* is the creation's raw value. This exact integer reduction involves no
      rounding and no additional draw. The value belongs to the object's persistent
      simulation state: saved games preserve it, loading restores it, and the
      deterministic state hash includes it. Presentation can obtain it for pre-placed,
      starting, newly created and restored objects, even if presentation begins after
      creation. Gameplay rules do not consult it, and sound choices do not feed back
      into simulation. S-0701 claims 21 and 32 own the report selection from this value,
      including exceptions. Retaining it must not replace the variant with zero or
      reconstruct it using another draw. Persistence and hashing are a design contract
      of our engine, not an additional claim about the original's save or hash format.
      Where the creation falls:
      - *Map loading.* A pre-placed object's creation draw is attributable to its map entry.
        Its tag choices (S-0011 claim 34) use the state left by that creation draw; its placement
        choices use the state left by its tag. Thus the object's creation shifts both sets of
        outcomes, even when the creation itself has no visible random effect. An entry naming
        no house of the game (S-0009 claim 27) or an unknown type creates nothing and takes no
        draw; an entry that is created but cannot be placed has
        still taken its draw. The players' starting units take their creation draws as S-0009
        (claim 46) orders them. The table *Draws during map loading* (State and ordering) places all of these
        relative to S-0009's phases and S-0011's tag draws.
      - *Production.* A produced object is created when its production begins, not when it leaves
        the building that produced it. Its creation draw therefore falls in whatever part of the
        tick starts that production (in S-0203's tick table, a house update, the production
        lines or the processing of orders), not at its entering play. The production specs fix that timing,
        including when the next item of a queue begins. S-0404 claim 11 assigns
        each creation draw at the action that causes the start: production orders receive
        values in application order, including automatic cancellations in S-0411 claim 13's
        tab/icon order, not line creation or payment-priority order. An order-boundary start
        receives a value from the state left by any turn-pass starts and intervening actions
        at that tick. Thus intervening random effects can change the new object's assigned
        value; production does not reserve values ahead of the causing action. Across houses,
        the values assigned to turn-pass starts follow turn order.
        Order-boundary starts in our engine follow S-0213 claim 3's house creation order
        and each house's production-order issue order; this between-house extension is
        an unmeasured design rule, not an original computer-production finding.
        Simultaneous completion does not create a separate batch of replacement starts:
        the later release or place action causes any queued successor's start.
      - *Packing up a slave miner.* S-0402 claim 19a owns the temporary hidden-slave
        creations: on success and refusal alike, vehicle-placement choices and the old
        slaves' release or destruction choices use the state left by the return vehicle's
        and new hidden slaves' creation draws. Success destroys the temporary slaves
        before transferring the old working group, without undoing their draws. Refusal
        retains the new group under S-0415 claim 11's default quirk. With
        `quirks.failed_packup_keeps_vehicle` off, refusal removes the already-created
        vehicle and new group; their creation draws still occur. This alternative is
        our engine's rule, not an additional oracle outcome.
      - *Every other creation* (reinforcements, units awarded by crates, deploying,
        paratroopers and similar) happens at a time the spec of that feature fixes,
        and the creation draw comes with it. S-0520 claims 3, 13 and 27 own crate
        draw composition: placement and replacement draws, awarded-unit creation
        and placement draws, and their order within a pickup. The crate overlay
        itself takes no object-creation draw (see *Overlay placement* below).
        In particular, paratroopers are created at launch after their plane is successfully
        placed, not when dropped (S-0522 claim 3). The launch's draw allocation follows payload
        entry order. Within an entry the groups are plane creation, arrival-cell choices, plane
        placement and each paratrooper's creation. Thus arrival and placement outcomes reflect
        the plane's creation draw, and the next entry's outcomes reflect the preceding entry's
        paratrooper creations. Failed plane placement creates no
        paratroopers. S-0522 claim 16 owns this launch order and the later drop-attempt
        standing-position and successful-drop parachute-animation draws.
    - **Parasite ranged choices.** S-0614 claims 9 and 10 own these choices on the
      scenario random stream. A drone parasite's delivery to a non-infantry host
      takes R(0, 1) from the state left by its effect creations; its hit's choices use
      the state left by that rocking choice. An infantry delivery takes no rocking choice.
      For a grappling parasite:
      - Each push start assigns R(0, 1) to rocking direction. The three wakes' groups
        follow that direction choice in draw allocation order. Within each wake, the
        groups are R(−180, 180), R(−64, 64) and wake creation. The two ranged results
        are integer displacement components in leptons; the second component depends
        on the first choice's consumption, and the next wake depends on any preceding
        wake-creation draws. Thus every push start has six wake-displacement ranged
        choices, not six fixed raw values.
      - Each strike's cull test uses the state left by three splash-effect groups.
        Each splash's groups are R(0, 1), R(0, *n* − 1) for selection from the
        game-wide splash list of length *n*, and splash creation. Consequently both
        choices and any creation draws affect the state available to later splashes
        and the cull test.
      - The first push starts at the end of the grab. After a strike, a new push's
        direction and wake choices occur only after a hit that leaves the host
        carrying the parasite; a cull or lost attachment takes no next-push choices.
      These are explicit ranged choices, with raw-value consumption governed by
      claim 4 (including rejection and equal bounds). They are not complete raw
      counts for the delivery, strike or push: effect creation and playback may
      contribute additional draws, which S-0614 leaves unestablished.
    - **Lightning sound choice.** Each lightning strike takes one raw value, read as
      unsigned and reduced modulo the game-wide lightning sound list's length. Its incoming
      state includes the bolt-animation choice and all that animation's creation draws.
      Explosion-animation and damage choices use the state left by the sound choice.
      Consequently even an inaudible sound choice shifts those later effects. It is taken
      regardless of audibility, sound settings, view or shroud. An empty sound list takes no value. Repeating a
      previous strike's point does not suppress this choice or any later strike effects
      and draws (S-0513 claim 18). S-0701 claim 25 owns the sound.
    - **Overlay placement.** Creating an overlay object takes no draw by itself. Some effects of
      placing it on a cell do:
      - *Bridge-building end markers.* The overlay types at positions 122 … 125 and 233 … 236 of
        the overlay type list are end markers for two families of low bridges. They pair up as 122
        with 123, 124 with 125, 233 with 234 and 235 with 236. The Formulas table *Low-bridge end
        markers* gives each marker's line, end-row cells, end-row overlay type, search direction
        and the type it looks for. Placing a marker on cell (*x*, *y*) has these effects:
        - *End row.* Its end row is the three cells across its line centred on its own cell:
          (*x*, *y* − 1), (*x*, *y*), (*x*, *y* + 1) for a marker whose line is the x axis, and
          (*x* − 1, *y*), (*x*, *y*), (*x* + 1, *y*) for one whose line is the y axis. What the
          marker does depends only on whether any of the three holds an overlay:

          | End-row cells | Result | Draw |
          |---|---|---|
          | none holds an overlay | each of the three cells gets the marker's end-row overlay type, with overlay data 0, 1 and 2 in order of increasing *y* or *x*. Each cell has its recalculation effects at this write, with their draw allocation in the same coordinate order. The written row is eligible for the partner and piece results below | none of its own (the recalculation can start a tile animation, below) |
          | any holds an overlay | nothing is written, and nothing else follows: no partner, no pieces | none |
        - *The marker itself.* The marker's own overlay type is never left on a cell. After a
          written end row its own cell is the end row's middle cell (the end-row type with overlay
          data 1), until pieces replace it in the case *m* = 1 below, and after an unwritten one it
          holds whatever it held before.
        - *Partner.* A marker that has written its end row has a partner when one exists: the
          nearest legal cell, on the marker's own row (line along x) or column (line along y) and
          in its search direction, that holds the partner's end-row type with overlay data 1 (the middle
          of the partner's end row) and is separated from the marker's cell only by legal cells
          (S-0009, claim 1). Overlays of any other kind between them make no difference, and
          there is no distance limit. When there is no such cell, the marker has no partner and
          nothing more happens. A partner whose end
          row was written later than the marker, by a marker later in the overlay type stream, is
          found by that later marker, so a bridge is completed by whichever of its two markers
          comes second.
        - *Pieces.* When the middle of the partner's end row lies *m* cells from the marker's own
          cell, *d* = max(*m* − 1, 1) cell steps become bridge pieces. For *m* ≥ 2 these are
          exactly the *d* = *m* − 1 cells strictly between the two end rows' middles. For *m* = 1
          (the two end rows side by side) the one step is the marker's own end row, whose three
          cells become pieces. At each step the three cells across the line become pieces, whatever
          they held, so 3*d* pieces in all. Each piece takes one raw value: its overlay type is the
          base type *t*₀ of its marker pair plus the lowest two bits of that raw value (the *low
          bits* form; formula *Bridge piece type*), so completing a bridge takes 3*d* raw values.
          The assignment of values to piece types follows steps from the partner's middle cell
          toward the marker, beginning with the adjacent step on the marker's side. Within a
          step the assignment follows increasing *y* or *x* (overlay data 0, 1, 2); this is the
          numbering *j* of the formula. Reversing either assignment can change the visible
          bridge. Neither the end-row writes nor the piece writes start a cell animation,
          even when the type written names one. The end marker's own cell animation is also not started.
          These writes do not count as placed overlays: they set off none of the other effects
          in this list (claim 14). Each piece's cell is recalculated as the piece appears,
          though, which can start that cell's tile animation. That animation uses the state
          immediately following the piece's raw value; the next piece uses the state left by
          the animation. A tile animation can therefore change every later piece type.
        - *Cells outside the map.* An end-row cell or a piece cell across the line can lie outside
          the map at its edge. The original then writes to its stand-in cell (S-0009, claim 44).
          Our engine writes nothing outside the map and counts such a cell as holding no overlay
          for the end-row test (S-0009, claim 44, a departure); in the original that is so as long
          as nothing has written to the stand-in. In both, a piece there still takes its raw
          value.
        This happens during map loading too, in the order of the overlay type stream. An overlay is
        placed only where S-0009 claim 12 allows one, which includes the slope condition stated
        there.
      - *Vein patterns.* The vein overlay is the overlay type at position 126 of the list (the
        type whose cells S-0009, claim 14, rebuilds). "Land type" below is the cell's land type as it
        stands (S-0211), and a cell's slope shape is its index of S-0106 (0 is flat).
        *The vein placement test.* A placement on cell *C* takes effect only when all of these
        hold; otherwise it changes nothing and takes no draw:
        - *C*'s slope shape is at most 4, its land type is none of *water*, *rock*, *beach* and
          *ice*, and it holds no overlay or one whose type has the veins flag (Terms),
        - each of *C*'s four edge neighbours has a land type other than *water*, *rock*, *beach*
          and *ice*, and holds no overlay or one whose type has the veins flag,
        - when *C* is flat, no edge neighbour has a slope shape above 4 while holding no overlay.

        An edge neighbour that lies outside the map is read as S-0009 claim 44 says (in our
        engine a blank cell, which passes).
        *The vein connection pattern* of a cell is *c* = Σ 2<sup>*i*</sup> over the edge
        neighbours *i* = 0 (map north), 1 (map east), 2 (map south) and 3 (map west) that count as
        a *connection*. Whether a neighbour counts depends on what it holds and on whether it is
        flat (slope shape 0) or sloped (any other slope shape):

        | Edge neighbour holds | Flat neighbour counts | Sloped neighbour counts |
        |---|---|---|
        | no overlay | no | no |
        | the vein overlay with overlay data 48 or more (a pattern centre, Terms) | yes | only when the vein overlay type has the veins flag |
        | the vein overlay with overlay data below 48 | no | no |
        | an overlay of another type that has the veins flag | yes | yes |
        | an overlay of another type without the veins flag | no | no |

        This is the table *Vein connections*. So *c* records only the pattern centres and the other
        vein-family overlays around a cell.
        *The placement.* A cell that passes the test gets the vein overlay and takes one raw value
        (formula *Vein pattern values*). A sloped placed cell does nothing more. For a flat one,
        each edge neighbour has the result of the table below. Values are assigned to the
        placed cell and eligible neighbours in the order placed cell, map north, map east,
        map south, map west. Consequently a neighbour that takes no value shifts the variants
        of later eligible neighbours. Each neighbour's *c* includes the placed cell's new data.
        Because the placed cell passed the test, each neighbour holds no overlay, the vein
        overlay or an overlay of
        another type that has the veins flag, so exactly one row applies to it:

        | Edge neighbour | Result | Draw |
        |---|---|---|
        | a pattern centre, or holding an overlay of another type that has the veins flag | unchanged | none |
        | sloped, holding no overlay or a vein that is not a pattern centre | gets the vein overlay, data by the sloped row of *Vein pattern values* | one raw value |
        | flat, holding no overlay | gets the vein overlay, data 3*c* + remainder, with *c* its connection pattern at that moment | one raw value |
        | flat vein cell that is not a pattern centre (data below 48), whose ⌊data / 3⌋ differs from its connection pattern *c* at that moment | re-patterned: data 3*c* + remainder | one raw value |
        | flat vein cell that is not a pattern centre (data below 48), whose ⌊data / 3⌋ equals *c* | unchanged | none |

        The placed cell is a pattern centre by then, so every flat neighbour's *c* has at least
        the bit pointing back at it (except for the one raw value of
        `quirks.random_magnitude_overflow`, Edge cases). Two edge neighbours of one cell are
        never edge neighbours of each other, so the order of the four does not change any *c*:
        it fixes only the order of the raw values. A neighbour that is already a pattern centre
        is never re-patterned. During play
        this placement happens when the vein rules grow veins (around a veinhole monster); what
        sets it off belongs to those rules.
        *Recompute.* The veins around a cell are recomputed during play when a unit that harvests
        veins harvests that cell, when a veinhole monster is removed (its own cell, which must be a
        pattern centre, is recomputed) and when a fence line (Terms) is built across a vein cell,
        which can also happen near the end of map loading (the map-loading table, row *Fence
        lines*).
        A recompute concerns the recomputed cell and those of its edge neighbours that are flat
        vein cells and not pattern centres. Each of them takes at most one raw value; the overlay
        data of one that keeps its vein follows the formula *Vein pattern values* (row "a flat
        cell set, re-patterned or kept"):

        | Cell | Condition | Draw | Result |
        |---|---|---|---|
        | The recomputed cell | sloped, or no vein connection (connection pattern 0) | none | loses its vein |
        | The recomputed cell | otherwise | one raw value ("magnitude after" remainder, modulo 3) | stays a vein |
        | A flat vein edge neighbour that is not a pattern centre | no vein connection (connection pattern 0) | none | loses its vein |
        | A flat vein edge neighbour that is not a pattern centre | a vein connection | one raw value (same form) | keeps its vein |

        Draw allocation order is the recomputed cell, map north, map east, map south, map west,
        omitting cells that take no value. Thus keeping or losing a vein at an earlier position
        can shift the variants assigned later. Each cell's connection pattern includes the
        results at earlier positions in that order.
        *Map loading.* Placing a vein from the overlay type stream takes no draw. The vein
        overlays are instead rebuilt by the pass of S-0009 claim 14, which runs after the overlay
        data stream has been read and the first recalculation pass has run. The vein candidates
        (Terms) are the cells that hold the vein overlay with overlay data 48 or more at that
        point. The pass's veins and draws are those of trying a placement as above, with its test
        and its draws, on each candidate in the reverse visiting order, on the map with every vein
        removed: every cell that held the vein overlay, candidate or not, holds no overlay and
        overlay data 0 unless a placement of the pass gives it a vein. So when its
        placement is tried, a candidate holds no overlay and overlay data 0, unless the placement
        of an earlier candidate in that order has already put a vein on it as an edge neighbour.
        Such a vein makes the candidate pass the first condition of the test only when the vein
        overlay type has the veins flag. The data values these draws set are therefore the ones in
        force after loading, for those cells. A vein cell with data below 48 is not a candidate:
        it keeps no vein unless a passing candidate next to it gives it one.
      - *Cell animations.* An overlay type with a cell animation starts that animation when it is
        placed, during map loading too, with no loop delay; the animation takes the creation draws
        of the next bullet. For an overlay on cell (*x*, *y*), the animation's position is the
        overlay cell's centre moved by 384 leptons along +x and +y: the top corner of cell
        (*x* + 2, *y* + 2), where its resource chain reaction acts. Its height is the terrain
        surface height at the overlay cell's centre (S-0009 claim 12). Its creation draws,
        including immediate start-event draws, precede the tile-animation draws of the
        recalculation caused by the same overlay placement. Thus later draws depend on the
        cell animation before that recalculation's tile animation.
      - Walls, ore and gem fields, crates, the high bridge overlays and the veinhole-monster overlay
        take no draw of their own when placed; the veinhole monster that the last one creates takes
        none either. (Placing any overlay recalculates its cell, which can start that cell's tile
        animation; see the map-loading table.)
    - **Animation creation draws.** In this table and the paragraph after it, *animation* means an
      animation of S-0109's sprite kind (S-0109, Terms), as distinct from the voxel-model debris
      pieces of S-0317, whose creation draws are S-0317's (claims 3 and 4). The table gives draw
      allocation order from top to bottom, with conditional rows omitted when inapplicable. Thus a random stage delay can change
      the starting motion, and either group can change immediate start-event outcomes.
      The resulting state determines later scenario-stream choices, including other creations.

      | Animation condition | Draws at creation |
      |---|---|
      | The type's random rate gives a range (S-0109, claim 3) | One ranged draw for the stage delay. |
      | The animation is a sprite animation of S-0109 of a bouncing type (S-0109, Terms); not a voxel-model debris piece | Starting motion uses a group of three raw values and a subsequent group of three ranged draws R(−65535, 65535). Each ranged request uses the state left by the preceding request and may consume more than one raw value (claim 4). What the motion does belongs to the debris and projectile specs. |
      | The animation is created with no loop delay (S-0109 claim 11) | Its start event (S-0109 claim 13) has immediate effects. When μ = 0, the resource chain reaction sees the cell and stream state left by the middle event (S-0109 claim 14), so a crater can empty the cell and suppress the reaction. The particle, smudge and chain-reaction specs own those effects' draws. |

      A chain reaction takes no draw and has no effect when the cell containing the animation's position holds no resource. This does not suppress any preceding middle-event draws. S-0109 claims 14 and 25 state the middle-event gates, sprite-derived μ and particle-before-choice ordering at map load and during play. Claim 15 below completes the requested particle and smudge creation draws. S-0401 claim 20 gives the resource-chain-reaction draw sequence, including conditional child-animation and detonation effects. No other draw comes with an animation's creation. A tile animation (S-0010, claim 19) and a cell animation are both created with no loop delay, so each takes the applicable draws at the moment it starts, including during map loading; nothing in loading holds animations back.
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | The creation draw is taken when an object is created | `measurable` (R5 b for its timing; R8 G for its exact count), high; M-0008 is consistent with it for an infantry unit (placed during play, it takes at its creation tick a raw value, then *R*(0, 3), then *R*(0, 2), which fit, in order and by range, the creation draw, its standing-position draw and its first behaviour-processing draw: a fit by range and order; that the third is the first behaviour-processing draw rests on the *guard* processing family fit of the S-0215 hand-off below, medium) |
      | Creation-derived report value: low-bit reduction and fixed normal report selection | `measurable` (S-0701 F2), high confidence |
      | Our engine retains, saves, restores and hashes the report value and exposes it to presentation without another draw | `unmeasured` design contract (Q-0327), high confidence; this is a requirement of our engine, not an oracle claim |
      | A produced object is created, with its draw, when its production begins | `measurable` (R5 c), high; M-0008 is consistent with it for an infantry unit (exactly one raw value logged at tick 300, after the order applied before that tick's logic, the unit leaving the building later: a fit by range and order; M-0008 does not exclude a different draw at the start of production that takes one raw value); a produced vehicle `measurable` (R5 c, vehicle repeat), medium; other produced kinds `unmeasured` (R5 c's repeat produces only a vehicle), medium |
      | Paradrop plane, arrival, placement and paratrooper creation order at launch; later drop draws | `measurable` (S-0522 P2/P3), high confidence |
      | Pack-up vehicle and temporary hidden-slave creation draws precede placement and old-group removal, including successful placement | `measurable` (S-0402 H24), high confidence; retaining these draws with the failed-pack-up quirk off is an `unmeasured` design rule |
      | Production starts receive values at their causing actions, not in payment-priority order | `measurable` (S-0404 P11 B/C); the between-house order-boundary extension is `unmeasured` design, not an oracle result |
      | Parasite explicit ranged choices, including six wake-displacement choices per push start and their ordering around effects and hits | `measurable` (S-0614 P8); high confidence for the explicit choices, medium for their composition with additional effect-creation choices, which remains unestablished |
      | Lightning sound choice: stream, count, empty-list gate, repeated-point non-suppression and position within a strike | `measurable` (Companion assignment and sound tests; S-0701 F6), high confidence |
      | Pre-placed objects (this row covers objects placed at map load only; an infantry unit placed during play takes its standing-position draw, S-0216 claim 3): an entry that is dropped takes no draw, a visceroid-flagged vehicle takes one R(0, 29), an infantry unit takes no placement draw, a building's upgrade takes its own creation draw | `measurable` (R8 G) |
      | Pre-placed objects: one that is created but cannot be placed has still taken its creation draw | `unmeasured` (no case is established in which the original creates a pre-placed object and then fails to place it: an entry on a cell that is not legal is unspecified there, and a vehicle is placed whatever occupies its cell, S-0009 claim 38) |
      | Bridges: number, form, order and base types of the piece draws | `measurable` (R8 A) |
      | Bridges: the end-row test, the partner, the case *m* = 1 | `measurable` (R8 D) |
      | Bridges: a tile animation started by a piece draws right after that piece's raw value | `measurable` (R8 H) |
      | Bridges: a piece outside the map still takes its raw value | `measurable` (R8 D, case vi) |
      | Veins: the placement draws, their order within a placement and across candidates | `measurable` (R8 B; the load-time pass uses the same placement rule as a placement during play) |
      | Veins: rejection by a water neighbour or a wall neighbour, flat vein connections and what a candidate holds when it is tried | `measurable` (R8 E) |
      | Veins: other land-type exclusions, slope thresholds, sloped placement and connections, and other vein-family overlay types | `unmeasured` (R8 B/E use flat sites with the vein flag on; no scenario part isolates these other conditions or formulas) |
      | Walls, ore and gems take no draw | `measurable` (R8 C) |
      | Crates, the high bridge overlays and the veinhole-monster overlay (and the monster it creates) take no draw | `unmeasured` (map crates are never created in a skirmish, S-0009 claim 12, and the high bridges and the veinhole monster need set-up conditions of rules that have no spec yet, so no scenario can yet place them in a known state) |
      | Fence lines: linking posts, section creation and their position in loading | `unmeasured` (no scenario part supplies a fence-post fixture; the fence rules have no spec yet), low confidence |
      | What sets off a vein placement during play; the recompute draws | `unmeasured` (the recompute's triggers are bound up with other effects, such as a harvesting unit's or a veinhole monster's own draws, that no spec lists yet, and one recompute gives at most five values, each telling only its remainder modulo 3, too few to find its place in the stream the way R8 does) |
      | Animation creation: the stage-delay draw, and nothing else for a type without start-event effects | `measurable` (R8 F) |
      | Animation creation: the bouncing draws | `measurable` (R8 F, variant ii) |
      | Animation creation: the bouncing row does not cover a voxel-model debris piece | `measurable` (S-0317 V1), medium until V1 runs and while the lead's question in S-0317 on this scope is open |
      | Cell animation position and chain-reaction cell; its creation draws precede the tile-animation draws of the same placement | `measurable` (R8 I, using S-0009 M6e) |
      | Animation creation: a chain reaction on a cell without resource takes no draw | `measurable` (R8 F, variant iii) |
      | Game-start pass: new tile animations take their creation draws in visiting order, after end-of-loading tags and before the first tick; cells that already started one in the second pass take no further creation draws | `measurable` (R10) |
      | Animation creation: the start-event draws come at once, before any later draw | `measurable` for cell-before-tile ordering (R8 I), middle-event gates and choice (S-0109 claims 14 and 25, AN23), and the resource-chain-reaction sequence (S-0401 claim 20, E17); particle and smudge draw counts are completed by claim 15 (R14) |
      | The list of systems that draw from the scenario random stream | `unmeasured` (no finite number of runs can confirm a list of every system that draws; each system's draws are backed by the topic spec that states them) |
    - Confidence: high (the creation draw is taken; a produced infantry unit is created when its
      production begins; the overlay draw counts and forms); medium (the same timing for other
      produced kinds: a vehicle on R5 c's vehicle repeat, the others `unmeasured`,
      production-start assignment at the causing action, S-0404 claim 11; the list of kinds; which
      vein neighbours are set or re-patterned and in which order, the placement test and the connection
      pattern; the end-row cells, the partner search and the *m* = 1 case; the order of the
      bridge pieces and the base types; where each map-loading group sits in the stream; the
      recompute: its draws, order and list of triggers; the cell animation's position and
      creation-before-recalculation order; the game-start pass's draws); high (no chain-reaction
      draw on a cell without resource); medium (the animation creation draws, whose
      bouncing and start-event parts are well established, while how an animation is placed on
      the map is only partly established; medium for the bouncing row's not covering a voxel-model
      debris piece, until S-0317 scenario V1 runs and while the lead's question in S-0317 on it is
      open)

11. **Unsynchronised random stream: presentation during play and at map load, setup values
    before play.** No draw from this stream that changes game state has been found. Its uses:
    - *During play:* which variant of a sound or voice response plays (including the idle comment
      of S-0107, claim 14, and the local player's unit responses and the spin-up sound of
      rapid-fire weapons, whose draw timing is S-0609 claim 13), except the lightning sound
      choice of claim 10. Sound instances
      also use this stream for pitch, volume shift, conditional pre-delay, sample choices
      and play-order choices (S-0701 claims 13 and 14). Equal-ended ranged draws take
      no raw value under claim 4; these are not a fixed raw count per sound.
      S-0701 claim 14 also owns the established partial delayed-mode choices:
      fresh completion-delay requests and decay-candidate choices before attack/body
      selection. Its complete audible sequence, preparation count, terminal wait
      duration and full draw ordering remain explicitly deferred; the partial rules
      imply neither a complete draw total nor a draw-free replacement.
      Normal weapon reports instead reuse the creation-derived value without another
      draw; spin sounds and occupant fire retain S-0701 claim 21's unsynchronised
      choices. Claim 10 here and S-0701 claim 32 retain the report value's
      persistence, save/restore and hash contract.
      The damage-feedback chance is R(0, 99) from this stream when a hit turns an object
      yellow and its type has a feedback list, even when its owner is not the local
      player (S-0701 claim 20). This chance is distinct from any subsequent response
      choice and sound-instance draws. Other uses are the choice of music track, the screen-shake displacement of an
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
      setup screen and its random map seed (claim 13), the default network port range, and one ranged
      draw over 1 … 32767 from which a setup screen derives a game seed (through the third
      generator of claim 13; for a local-network game this gives the seed of claim 8). These draws
      use the stream in whatever state the previous game left it or, before the first game of a
      session, the state of a seeding by claim 8's clock rule that the original makes when the
      program starts. Such a value then becomes part of the game setup, like the game seed, and reaches the
      game only through the setup; in our engine these values are interface choices (claim 8's
      departure).

    Our engine runs this stream from the game's seed (claim 8) with the same algorithm and lets
    none of its draws reach game state. Game-side sound choices and per-instance playback
    choices share this one unsynchronised stream, not separate generators (S-0701 claim 30).
    In the original, host-dependent playback service interleaving can change presentation
    choices without changing scenario-stream state or game rules. Our engine uses S-0701
    claim 30's deterministic cycle-end audio passes and logical sample-completion and wait
    opportunities. Game-side sound choices are assigned at their causing events before any
    per-instance choices at that cycle's end pass; claim 30 fixes the pass choices' assignment
    order on this same shared stream. This specified schedule does not complete the
    delayed-mode sequence still deferred under S-0701 claim 14.
    How this stream runs on the machines of a network game is for the future multiplayer
    spec (network games are out of scope for now).
    - Backing: `measurable` (scenario R6 a) that no draw of this stream reaches the scenario random
      stream; `measurable` (R6 b) for voice responses and the choice of music track. Sound-instance
      pitch, volume shift, pre-delay, sample and play-order draws inherit S-0701 claims 13 and 14's
      backing (E1 and E3) and confidence for the explicitly established choices only; the complete
      delayed-mode sequence remains outside coverage, not an additional claim with backing here.
      Shared-stream retention and the deterministic schedule are S-0701 claim 30's design
      requirements, tested by its engine-only H2 rather than oracle measurements. Report choices and
      the creation-derived value retain S-0701 claims 21 and 32's backing and confidence; the
      damage-feedback chance inherits claim 20's high confidence and `measurable` backing (F1). The
      map-load use through S-0010's scenario V2: part A for the pattern's properties, part B for the
      draws themselves, with the random-state recorder's reading of the unsynchronised stream
      (Measurement scenarios); the random map seed through R9 b; the seeding at program start
      through R9 d; the local-network seed's range through R2 b; `measured` (M-0008), medium, that
      each launch has an earlier seeding, of a stream the record does not identify; the time between
      the two seedings is not shown. That this earlier seeding is a clock seeding at program start
      rests on R9 d. `unmeasured`: the screen shake, the electric-bolt and radiation-beam jitter,
      the flicker colour, the default network port range and how a setup screen derives the game
      seed (each combat effect comes with a sound whose variant is drawn from the same stream in the
      same tick, so the effect's own draws cannot be told apart from the sound's; the objects that
      flicker and the moment the port range is chosen are not identified, so no scenario can set
      them off on purpose; and the seed derivation passes through the third generator of claim 13,
      which shows only through the resulting seed)
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

13. **Map-generator stream (Q-0711).** The random-map generator has a stream of its own with the same
    algorithm (claims 1 to 3). An **ordinary map** here is a map loaded from already-defined map
    data, not generated from random-map settings. The stream is initially seeded with the unsigned
    integer 0 at session start, independently of the game seed. Starting a skirmish on an ordinary
    map neither selects a map-generator seed nor reseeds this stream. In particular there is no
    unsynchronised R(0, 65535) for it, before or after map-load presentation draws. A fresh
    session that has not generated a random map or otherwise used this stream therefore begins
    ordinary-map play with the untouched seed-0 state: its next raw value is x₀ of claim 3's
    seed-0 sequence. The game's seed cannot change that state.

    Ordinary-map games in the same session retain the state left by earlier uses, including
    earlier games' low-bridge repairs and random-map generation on the setup screens. Returning
    to the menus and starting another ordinary-map game does not restore seed 0. During play,
    low-bridge repairs advance the existing stream, including rejected values, without reseeding.
    Thus this is not a once-per-game seeding rule. To reproduce a continuing session our engine
    retains this stream's state across its games; the game seed alone does not identify that
    continuing state. A new session begins at seed 0, not at a value derived from a fresh copy
    of the unsynchronised stream.

    In a skirmish the random-map generator can be reached from the map selection
    of the setup screens, through a random-map setup screen, in a game mode that offers it.
    Only when generating a random map, including its generation at game start, is the stream
    reseeded from the random map's 16-bit seed setting,
    from 0 to 65535, or, when that setting is 0, from a ranged draw R(0, 65535) of the
    unsynchronised random stream, taken after that stream has been seeded for the game (claim 8).
    The random-map setup screen shows no seed and offers no way to type one. It chooses a new
    setting itself, with a ranged draw R(0, 65535) of the unsynchronised random stream (claim 11),
    whenever the player asks it to choose all its settings at random. Until it has made such a
    choice in a session the setting is 0, so a random map generated without one is played from a
    seed drawn at game start. Besides generating the random map, whose rules belong to a future
    spec, this stream supplies low-bridge repair's piece choices during play (S-0511 claim 20).
    For each strip requiring a random intact piece, consider the successive unsigned raw
    values *g* from the map-generator stream. For each value, *r* = trunc(*g* × 4 × *c*),
    where *c* is the double-precision constant nearest 1 / (2³² − 1). Both multiplications
    use double-precision arithmetic in S-0205 claim 5's round-toward-zero mode; the final
    result is truncated toward zero to an integer. The strip's piece corresponds to the
    first value for which *r* lies in 0 … 3. Every preceding value with *r* above 3 is also
    consumed. Strips receive these groups in increasing bridge-axis coordinate order, so
    each strip's choice uses the state left by the preceding strip's accepted and rejected
    values. This selection is not claim 4's ranged-draw rule. Strips requiring no random
    piece choice take no map-generator value; occupant deaths and scatter retain their own
    scenario-stream draws (S-0511 claims 17 and 20).
    A third generator of a different kind serves only the original's setup screens
    (to choose a network game's seed, claim 8), visual effects and the names of saved-game files.
    Our engine need not reproduce it.

    | Part | Backing | Confidence |
    |---|---|---|
    | The skirmish setup screens can reach the generator, in a game mode that offers it | `measurable` (R9) | medium |
    | Same algorithm; seeded from the seed setting, which lies in 0 … 65535 | `measurable` (R9 a) | medium |
    | The random-map setup screen chooses the setting with a ranged draw R(0, 65535) of the unsynchronised stream | `measurable` (R9 b) | medium |
    | Session initialization uses seed 0; an ordinary-map start has no fallback draw or reseeding, irrespective of the game seed or map-load presentation draws | `measurable` (R9 g) | high |
    | Ordinary-map games retain prior stream state across the menu boundary; repairs advance it without reseeding | `measurable` (R9 g/h) | high |
    | Without a random-settings choice in a fresh session, a generated map's game-start seed comes from R(0, 65535) of the newly seeded unsynchronised stream, not necessarily its first value | `measurable` (R9 e) | low |
    | The hidden setting is initially 0 and 0 selects the fallback | `unmeasured` (R9 e reads the resulting seed, not the hidden setting; no scenario part distinguishes the sentinel itself) | low |
    | No map-generator-stream draw during R9's sampled play without low-bridge repair | `measurable` (R9 c) | medium |
    | Low-bridge repair piece choices: stream, formula, consumption including rejection retries, and strip order | `measurable` (R9 f; S-0511 claim 20) for ordinary choices and order; `unmeasured` for the rejection endpoint, which needs a particular raw value in 2³² that the recorder cannot force | medium |
    | The third generator and its uses | `unmeasured` (a list of the only uses a generator has cannot be confirmed by any finite number of runs; its one use that reaches a game, the local-network seed, shows through that seed's range in R2 b) | medium |
    - Backing: `measurable` (scenario R9) for the parts so marked above; the rest `unmeasured`,
      with the reasons given above
    - Confidence: medium

14. **Loading-order refinements (Q-0101).** In a new skirmish, the following relations
    fix observable animation and resource outcomes and the stream state available to later
    choices. They describe draw allocation, not a required internal loading schedule:
    - **House creation (Q-2336).** Each house that the game start creates takes one ranged
      draw R(450, 1800) when it is created. The bounds are a constant of the executable, the same
      for every house, with no source in the rules data, and the draw takes raw values as claim
      4's ranged-draw rule gives for that range. No rule of this
      project uses the draw's result. A skirmish game's start
      creates S-0009 claim 19's house list twice, both times in that claim's order, so these
      draws come in two rounds, one draw per house in each. The first round comes before S-0009
      claim 20's early phase, whose position
      draws are taken for the first round's houses. Those houses are then discarded and take
      no further observable part in the game: each player keeps the position the early phase
      gave it (S-0009 claim 20). The second round comes after the early phase and before the
      initial fill, so before every later group of the map-loading table, the fill's draws of a
      water-filled map included. A network game's
      start is the same. A campaign mission creates each house its map lists once, in the
      map's order, each with one such draw.
      - Backing: `measurable` (R17) for the draw, its bounds for every house, the two rounds
        and their places on either side of the early phase and before the fill's draws (variant D)
        in a skirmish game. M-0019's counts of raw values before
        tick 0 include both rounds (S-0216 claim 8), which a reading of those records' draws
        shows, not an observation. `unmeasured` for the first round's houses taking no further
        part, since no scenario part here observes houses in play, and R17 records the stream
        only. `unmeasured` for the network game and the campaign mission, which no record or
        scenario part reaches.
        Confidence: medium for the draw, its bounds, the two rounds and their places on either
        side of the early phase and before the fill; medium for the first round's houses taking no
        further part
        (unmeasured); low for the network game and the campaign mission.
    - The unused-house cleanup of S-0009 claim 28 follows the second whole-map
      recalculation of S-0010 claim 12 and precedes the late start-position phase. Taking a
      removed house's grounded pre-placed objects out of their occupied cells recalculates
      those cells, including a building's foundation cells. A recalculation can start a missing
      tile animation under S-0010 claim 19, and its creation draws occur during the cleanup,
      not during the second pass or the later game-start pass. Thus late start-position
      choices include any cleanup consumption, whereas second-pass choices do not.
      A cell whose tile-animation start is already recorded since the second pass takes no
      new creation draws.
      Removal of an object not occupying a ground cell does not imply a ground-cell
      recalculation.
    - A tile animation created by game-start recalculation receives values from the state
      left by starting units, initial crates (S-0412 claim 10), secret-lab choices (S-0407
      claim 14) and end-of-loading tags. This includes the starting-unit group's final
      R(0, 65535) only when that request occurs. Missing starting units or an early stop
      without that final request omit only those draws; crate, lab and tag choices still
      affect the animations' assigned values. The animations exist before the first tick.
      Their creation groups use claim 10's draws in visiting order, so an earlier cell's
      animation can change a later cell's delay or immediate effects.
    - After the game-start recalculation, resource-glint setup can take further scenario-stream
      draws before tick 0. When a glint animation type is configured, each cell with a nonzero
      contained resource value takes R(0, *f* − 1), where *f* is the imported glint frequency
      (a signed 32-bit integer) and *f* − 1 wraps in signed 32-bit arithmetic (S-0205).
      Eligible cells receive their choice groups in visiting order. Contained value is the resource's
      imported value per bale times the cell's bale count, with signed 32-bit multiplication
      wrapping as S-0205 specifies; empty cells and zero-value resource cells take no draw.
      A zero result creates the glint animation with no loop delay, with all applicable
      animation-creation and start-event draws of claim 10. The next eligible cell's choice
      uses the state left by this entire group, so a glint's creation can change which later
      cells glint. A nonzero result creates nothing. With no configured animation, the whole group
      takes no draw. For *f* = 1 the choice consumes no raw value and every eligible cell
      creates an animation, whose own draws still apply. Other bounds obey claims 4 and 5,
      including reversed bounds and extreme spans. Recalculation is therefore not an
      exhaustive final-draw boundary for loading.
    - Low-bridge end markers write their end-row and piece types directly: neither those
      types' cell animations nor the marker type's cell animation start. Thus an end-row
      cell's write can take tile-animation creation draws from its recalculation, but no
      cell-animation creation draws. A piece's tile animation uses the state left by that
      piece's type choice, and the next piece's type uses the state left by that animation.
      Consequently a tile animation can change the later bridge pattern, while assigning a
      cell animation to the written type cannot: no cell-animation draw is inserted.

    S-0010 claims 12 and 19 own the recalculation passes and tile-animation eligibility.
    S-0009 claim 28 owns unused-house cleanup. Claim 10 supplies the low-bridge exclusions,
    and claim 15 completes downstream particle and smudge creation draws. S-0412 claim 14
    agrees with these relative positions without asserting that secret labs or recalculation
    take the final loading draws.
    - Backing: `measurable` (R10 variants A and B for game-start placement,
      R12 for low-bridge cell-animation exclusion, R13 for resource-glint setup at
      frequencies 0, 1 and 2). Glint choices at other frequencies and extreme-span
      behaviour are `unmeasured`: R13 contains no part exercising those bounds; their
      predictions follow claims 4/5 and the imported field's arithmetic. The contained-value
      formula's signed-overflow cases are also `unmeasured`: R13's resource controls do not
      isolate wrapping products.
      Cleanup's position, recalculations and conditional animation draws are `unmeasured`:
      the former R11 reused a deferred network fixture, and no discriminating skirmish
      fixture with no computer player has been established.
    - Confidence: high for the relative loading positions, cell-animation exclusion and
      resource-glint draw rule; medium for cleanup's recalculations and their conditional
      tile-animation draws

15. **Middle-event draw composition at map load and during play (Q-0182, Q-0217).**
    S-0109 claims 13, 14 and 25 already answer the immediate event with no loop delay and
    μ = 0, event gates, sprite-derived μ, particle-before-choice ordering and the
    crater request's taking 6 bales under S-0401 claim 3. A resource cell at stage
    *d* ≥ 6 remains at stage *d* − 6; one at stage 0 … 5 is emptied. In particular,
    stage 6 becomes stage 0, not empty. These are integer stage changes with no rounding,
    and S-0401 claim 3 owns the last-bale quirk. S-0312 claim 26 already answers the smudge
    footprint fit test, registration-order candidates, ranged selection and draw-free placement.
    The following completes their composition, equally during loading and during play.

    An animation created during play with no loop delay and μ = 0 takes the same
    middle-event draws, in the same order, as one created during loading. Its creator
    (a weapon, teleport, death, debris, trigger or terrain effect) does not change that rule.
    All draws below use the scenario random stream, not the unsynchronised random stream.
    The event's incoming state includes any stage-delay and bouncing creation draws of
    claim 10. A start-event resource chain reaction uses the cell and stream state left
    by the complete middle event; thus particle or smudge choices can shift its later
    choices, and a crater can suppress it by emptying the resource. There is no extra
    phase-dependent draw or omitted effect. Equal incoming stream states, animation data
    and relevant cell state give equal outcomes and consumption; different fitting smudges
    or resource state can of course
    change the conditional groups and selection bounds. The same composition applies when
    a middle event occurs later in playback under S-0109 claim 14; subsequent particle
    updates are not particle creation and remain outside this spec. That exclusion does not
    make them draw-free: S-0613 claims 6 to 8 own gas-update draws and successor creation
    draw ordering, and S-0616 claims 6, 8, 9, 11, 12 and 15 to 18 own the non-gas update,
    successor, emission and burst draws, separately from this event's group.

    The smudge height gate is also the same in both phases: for integer world position
    (X, Y, Z), test Z − H(X, Y) < 30 leptons, with H the terrain surface height at that
    actual position under S-0106 claims 2 and 3, including its in-cell displacement and
    their rounding. The subtraction and comparison are integer operations, with no new
    rounding. The test is not against the creator's cell centre or a bridge deck. Negative
    height above terrain passes, exactly 30 leptons fails. This gate suppresses the
    scorch-or-crater choice and smudge request, not the preceding particle groups; its
    complete flag cases and choice rule belong to S-0109 claim 14.

    **Particles.** The table gives each particle's draw allocation order. Its lifetime
    variation uses the state left by any smoke-speed draw, and its colour uses the state
    left by lifetime variation. The next particle uses the state left by that whole group,
    so a colour draw for one particle can change the next particle's speed or lifetime.
    No particle-system creation draw is added: the request creates a particle in the
    existing animation-particle system. Particles do not take the
    infantry/vehicle/aircraft/building creation draw of claim 10.

    The table below belongs to the particle itself, so it applies to every creator of a
    particle, not only to an animation's middle event: a particle made by a particle system's
    emission, spark burst or railgun line (S-0616 claims 9, 11, 15 and 17), by a smoke
    successor (S-0616 claim 8), or by a gas-behaviour particle's successor creation (S-0613
    claims 5 and 7) takes the same draws in the same order. The owning spec places the group
    among its own draws.

    | Condition | Draw and form |
    |---|---|
    | Smoke particle behaviour | One raw value, signed remainder modulo 2, for a starting-speed variation. |
    | Every particle | One raw value for its lifetime variation: magnitude after signed remainder, modulo 10 for railgun behaviour, otherwise modulo the imported lifetime field. With a nonzero modulus the variation lies in 0 … \|modulus\| − 1; with a zero lifetime field the zero-divisor case below applies instead. The variation is added to the lifetime field to give the particle's initial lifetime, in ticks (S-0613 F9). |
    | The colour list is nonempty and at least one component of either starting colour is nonzero | One ranged draw R(0, 2147483646) for the starting-colour interpolation. |

    There are no other scenario-stream creation draws for these requests. A nonempty colour
    list with both starting colours black uses its first colour without a colour draw. An empty
    colour list takes no colour draw, even with nonblack starting colours. Normalised particle
    timing adds no scenario-stream draw. Particle count
    at most zero, or no particle type, means no particle group. Cell position, terrain height,
    theater, overlays, buildings and other map state do not alter these groups or their bounds.
    They can affect the particle's position and later life, not its creation draw count.
    The lifetime field is a signed integer; its zero-divisor case follows S-0205 claim 3:
    the original fails after taking the raw value, while our engine uses a remainder equal to
    that raw value's signed reading, with an importer warning, retaining that draw. The
    subsequent magnitude uses signed 32-bit wrapping, so only −2147483648 stays negative.
    Negative lifetime divisors also follow S-0205's signed division rules, including its
    overflowing-quotient departure for −2147483648 divided by −1. Railgun behaviour has
    neither exceptional divisor case.

    **Smudges.** The conditional scorch-or-crater choice of S-0109 claim 14 uses the state
    left by all particle groups, so changing the requested particles can change the smudge
    kind. The chosen kind uses S-0312 claim 26's fit and selection rule, with these forms:

    | Request | Fit and candidate preference |
    |---|---|
    | Scorch mark, or crater without the large-crater flag | Test buildings as in the plain form. Prefer fitting 1 × 1 types, unless the middle frame's visible width is greater than 60 pixels and its visible height greater than 50 pixels; with both dimensions above those thresholds, use all fitting types. If the preferred list is empty, use all fitting types. |
    | Crater with the large-crater flag | The large form of S-0312 claim 26: do not test buildings, prefer fitting types wider than 1 cell and taller than 1 cell, falling back to all fitting types. |

    Visible dimensions are the middle image frame's bounds, not the full sprite canvas, and
    are those of the sprite resolved by S-0109 claim 25. If no sprite is available, both
    dimensions are 30 pixels. These are integer comparisons, with no rounding. The anchor
    is the cell containing the animation's actual position, including cell-animation displacement.
    With *n* candidates in registration order, selection is R(0, *n* − 1): no draw for *n* = 0
    or 1, and no additional placement draw. Already-present smudges, overlays, slopes,
    buildings in the non-large form, and the theater's morphable tiles affect the candidates
    and hence the range and rejection count. A failed request is not a reason to omit the
    preceding particle groups or the scorch-or-crater choice.

    **Resource removal relative to map data.** An immediate crater request during overlay-type
    placement, including a tile animation started by that placement's recalculation, acts
    before the overlay data stream is read. It takes 6 bales under S-0401 claim 3 from the resource state then
    present. The later data stream overwrites every legal cell's overlay data, even a cell
    whose resource was removed. It does not restore an overlay type removed by that request.
    By contrast the first whole-map recalculation follows the data stream, so a tile animation
    first created in that pass takes those bales from the imported resource state. The second and game-start
    passes act later still. Thus there is no single before/after answer for all map-load
    animations: the animation's creation group in the loading table determines it.

    Resource removal that empties a cell can itself recalculate that cell. A missing tile
    animation started there has claim 10's creation and start-event effects immediately.
    A partial take, including stage 6 becoming stage 0, has no such emptying effects.
    Emptying also makes S-0401 claims 5 and 8's conditional neighbour spread offers.
    The offers use the state left by any recalculation-created animation; smudge-type
    selection uses the state left by both. Thus resource exhaustion can change the chosen
    smudge type through those intervening draws, even with the same fitting candidates.
    They are conditional effects, not an extra fixed draw per crater.

    In draw allocation order the groups are particles, optional kind choice, a crater's
    conditional resource-removal effects (recalculation effects and neighbour offers), and
    smudge-type selection. S-0401 claim 20's start-event chain reaction sees the cell and
    stream state left by that entire immediate middle event.
    - Backing: `measurable` (R14 A for particles, B for smudge forms and dimensions, C for
      data-stream ordering and conditional recalculation, D for the same composition during
      play and later playback, E for the shared height test). The zero-divisor failure and
      retaining its raw value are `unmeasured` here: R14 excludes that crash case; the
      deterministic replacement is S-0205's design departure. Negative lifetime divisors
      are also `unmeasured`: R14 A uses positive divisors only. The exhaustive assertion that
      no map state can change a particle creation group is `unmeasured`: R14 tests named
      terrain, overlay, occupation and theater variants, not every possible map state.
      Independence from every creator is also `unmeasured` beyond the tile, overlay,
      weapon and trigger fixtures in R14: it gives no teleport-, death- or debris-created
      fixture. The corresponding event composition is still required for those creators.
      The table's application to the other creators named in the paragraph above (emission,
      spark burst, railgun line, smoke successor and gas-behaviour successor) is `measurable`
      by the closing part of R14 A, and the gas lifetime draw is also checked by
      S-0613's scenario G5. Until that scenario has been run, the application to those
      creators rests on the owning specs' draw descriptions.
    - Confidence: high for particle draw forms, gates and order, smudge candidate preferences
      and data-stream ordering, the phase-independent event composition and terrain-height
      test; medium for absence of further placement draws and conditional
      resource-removal recalculation composition; medium for the table's application to the
      other creators named in the paragraph above

16. **Post-game winner-score draws.** Preparing the skirmish result statistics after simulation
    has stopped takes one scenario-stream ranged request per undefeated listed house, in
    house order, under S-0414 claims 19 and 23. For that house's signed 32-bit integer running
    score *p*, let *b* = max(*p*, 0) score points. Its request is R(⌊*b* / 2⌋, *b*), with
    exact integer division rounded down. Defeated houses and houses omitted from the result
    statistics request none. Listed houses are those still present, non-passive and not the
    observer's house, including defeated houses. The order is house order, not an order
    derived from their displayed scores.

    The stream continues from the state left by play; there is no score-specific reseeding.
    Equal bounds take no raw value under claim 4, so a zero-score undefeated house advances
    nothing. Other bounds consume claim 4's raw values, including rejections, before the next
    eligible house's request. Eligibility uses defeat state at statistics preparation, not
    a committed victory result: a house defeated in the final tick requests none even if its
    committed result remains a win. These draws contribute only to the displayed score.
    They cause no further simulation tick and change no simulation outcome. S-0414 claim 23
    owns the displayed-score formula.
    - Backing: `measurable` (R15 and S-0414 V18)
    - Confidence: high

17. **Reading a particle type's colour list (Q-1562).** The colour list whose entries gate claim
    15's colour draw, and which S-0616 claim 29 draws from, is read from its text value as follows.
    The value is taken as S-0003 claim 1 gives a text value with a 511-byte limit: comment
    removed, surrounding whitespace trimmed, cut to its first 511 bytes, and any trailing
    whitespace of the cut text then removed. So a cut that ends in `…,(4,5,6 ` leaves the third
    item `6`, whose blue component reads 0 once its last byte is removed.
    - **Items.** The text is split at every comma. Empty items, from two adjacent commas or from a
      leading or trailing comma, are dropped. No item is trimmed. Parentheses are neither
      separators nor group marks: they are ordinary bytes of the items they sit in.
    - **Entries.** The items are taken in consecutive threes from the start. Each complete three
      gives one entry, in list order, so a list of *k* items has ⌊*k* / 3⌋ entries, and one or two
      items left over at the end give nothing. No three is skipped, and no component is kept from
      an earlier entry, from the value in force or from an earlier layer.
    - **Components.** Of each three, the red component is read from the first item without its
      first byte, the green component from the second item whole, and the blue component from the
      third item without its last byte. In a clean list the removed bytes are the opening and the
      closing parenthesis. Each is read by the decimal form of S-0003 claim 3 (optional
      whitespace, an optional sign, decimal digits up to the first other byte, 0 when there is no
      digit), with S-0003 claim 4's wrapping, and the result is reduced modulo 256 to the least
      non-negative residue: −1 gives 255, 300 gives 44. The `$` and `h` hexadecimal forms of
      S-0003 claim 3 do not apply: `1Fh` reads as 1.
    - **Malformed groups.** These follow from the rules above (invented test texts):

      | Text | Entries |
      |---|---|
      | `(10,20)` | none: two items |
      | `(10,20),(30,40,50)` | one, (10, 20, 0): the third item is `(30`, which reads 0 without its last byte; `40` and `50)` are left over |
      | `(1,2,3,4)` | one, (1, 2, 0): the third item `3` without its last byte has no digit; `4)` is left over |
      | `(1,2,3,4),(5,6,7)` | two, (1, 2, 0) and (0, 0, 0) |
      | `(1,x,3)` | one, (1, 0, 3) |
      | `( ,2,3)` | one, (0, 2, 3) |
      | `(1,,3)` | none: the empty item is dropped, leaving two |
      | `(1,2,3),(4,5` | one, (1, 2, 3): the unclosed group has two items |
      | `(1,2,3),(4,5,67` | two, (1, 2, 3) and (4, 5, 6): an unclosed group with three items is an entry, and its last item loses its last digit |
      | `( 7,+8,9z)` | one, (7, 8, 9) |
      | `(1,2,3), (4,5,6)` | two, (1, 2, 3) and (0, 5, 6): the space after the comma is the removed first byte, and `(4` reads 0 |
      | `junk(1,2,3)` | one, (0, 2, 3) |
      | `(1,2,3)junk` | one, (1, 2, 3) |
      | `1,2,3` | one, (0, 2, 0) |

      So a group with a missing, extra or empty component shifts every later group, and a group
      with too few items at the end of the list is not an entry. A list whose reading has no
      entry, such as `(10,20)` alone, is an empty list for claim 15: it takes no colour draw.
    - **Importer warning.** With either setting of `quirks.colour_list_item_triples`, the importer
      warns when the value is not a clean list, that is when (a) this claim's reading and the
      parenthesis-aware reading of the quirk's off setting (its groups as that setting defines
      them) differ, (b) a component read by either reading has no digit, or bytes other than
      whitespace follow its digits, (c) bytes other than whitespace and the separating commas lie
      outside groups, an unclosed group included, or (d) that setting finds a malformed group,
      or a well-formed group whose split at commas gives other than exactly three parts. So
      `(1,2,00,4)`, which both readings read as (1, 2, 0), warns under (d).
      It also warns when the 511-byte cut falls inside the list. The warning does not change the
      result.
    - Backing: `measurable` (R16) for the original's reading: the items, the entries, the
      components with their decimal form, wrapping and modulo-256 reduction, the table and the
      empty-list case. `unmeasured` for the 511-byte cut and the trailing-whitespace removal after
      it, as for S-0003 claim 20's 511-byte limit, because S-0001's physical-line limit cuts a
      value from a file sooner. The importer warning is a rule of our own (`design`), not an
      observation of the original
    - Confidence: high

## Formulas

All arithmetic in this section is on integers, exact, unless it says otherwise. "mod 2³²" reduces
to 0 … 4294967295. For a value *t* in 0 … 2³² − 1, *σ*(*t*) is its signed reading: *t* if *t* <
2³¹, else *t* − 2³². ⊕ is bitwise exclusive-or of two 32-bit values.

**Raw sequence** (claims 1, 2): x<sub>n</sub> = x<sub>n−250</sub> ⊕ x<sub>n−147</sub> for n ≥ 0,
with x<sub>k−250</sub> = F(*s*, *k*) for *k* = 0 … 249.

**Mixing function F(s, k)** (claim 3). The four rounds act on a pair of 32-bit unsigned values, starting from *L*₀ = *s* and *R*₀ = *k*:

| Round *i* | *L*<sub>*i*+1</sub> | *R*<sub>*i*+1</sub> |
|---|---|---|
| 0 | *R*<sub>0</sub> | *L*<sub>0</sub> ⊕ *G*<sub>0</sub>(*R*<sub>0</sub>) |
| 1 | *R*<sub>1</sub> | *L*<sub>1</sub> ⊕ *G*<sub>1</sub>(*R*<sub>1</sub>) |
| 2 | *R*<sub>2</sub> | *L*<sub>2</sub> ⊕ *G*<sub>2</sub>(*R*<sub>2</sub>) |
| 3 | *R*<sub>3</sub> | *L*<sub>3</sub> ⊕ *G*<sub>3</sub>(*R*<sub>3</sub>) |

F(*s*, *k*) = *R*₄. All values and operations have the unsigned 32-bit domains specified above.

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
| 0 | 3 131 664 519 | 1 259 289 432 |
| 1 | 504 877 868 | 3 899 977 923 |
| 2 | 62 708 796 | 1 767 228 838 |
| 3 | 255 054 258 | 1 437 059 654 |

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
> *d* ≥ 2147483648: departure: *w* is the smallest integer in 0 … 250 with
> x<sub>*q*+*w*</sub> ≤ *d*. Such a *w* always exists by claim 5's recurrence argument.
> Result *σ*((*lo* + x<sub>*q*+*w*</sub>) mod 2³²). The draw consumes exactly the *w* + 1
> raw values x<sub>*q*</sub> … x<sub>*q*+*w*</sub>, hence at most 251 raw values.

Expected number of raw values consumed, *w* + 1, for 1 ≤ *d* ≤ 2147483646: 2<sup>*m*</sup> / (*d* + 1),
an exact rational (a statistical statement about the draw, not a value the engine computes).

**Unit-interval value** (claim 6). With *r* = R(0, 2147483646), an integer, and *c* the double
(2³⁰ + 1) / 2⁶¹ (the double nearest to 1/2147483646):

> *u* = RZ(*r* × *c*), the exact product rounded to a double

where RZ is the original's rounding toward zero (S-0205, claim 5). *u* ranges from 0 to 1 − 2⁻⁵³. The exact
product for *r* = 2147483646 is 1 − 2⁻⁶⁰; rounding toward zero gives 1 − 2⁻⁵³, while rounding to
nearest would give exactly 1. So with the switch `quirks.chop_rounding` of S-0205 off, *u* can equal 1.

**List pick:** index = *x* mod *n* for one raw value *x* and list length *n* ≥ 1.

**Signed remainder** (positive modulus *n* ≥ 1): *σ*(*x*) rem *n* (sign of *σ*(*x*)); "magnitude first": |*σ*(*x*)| rem *n*,
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

**Low-bridge end markers** (claim 10). All values are positions in the overlay type list, apart
from the line and the search direction. The end-row cells are listed in the order of their overlay
data 0, 1, 2, for a marker on cell (*x*, *y*).

| Marker | Line | End-row cells | End-row type written | Search direction | Partner | Type looked for, with data 1 |
|---|---|---|---|---|---|---|
| 122 | x axis | (*x*, *y* − 1), (*x*, *y*), (*x*, *y* + 1) | 92 | map west (*x* − 1) | 123 | 94 |
| 123 | x axis | as for 122 | 94 | map east (*x* + 1) | 122 | 92 |
| 124 | y axis | (*x* − 1, *y*), (*x*, *y*), (*x* + 1, *y*) | 96 | map south (*y* + 1) | 125 | 98 |
| 125 | y axis | as for 124 | 98 | map north (*y* − 1) | 124 | 96 |
| 233 | x axis | as for 122 | 223 | map west | 234 | 225 |
| 234 | x axis | as for 122 | 225 | map east | 233 | 223 |
| 235 | y axis | as for 124 | 227 | map south | 236 | 229 |
| 236 | y axis | as for 124 | 229 | map north | 235 | 227 |

The pieces of one step lie on the same three cells across the line as the end row, displaced
from the step's cell on the line by (0, −1),
(0, 0), (0, +1) for the x-axis markers, and (−1, 0), (0, 0),
(+1, 0) for the y-axis markers. With the middle of the partner's end row *m* ≥ 1 cells away, the
number of steps is *d* = max(*m* − 1, 1), an integer.

**Vein pattern values** (claim 10). For one raw value *x* per cell, with *c* the cell's vein
connection pattern (an integer 0 … 15, claim 10) and *s* its slope shape (S-0106; 0
is flat):

| Cell | Overlay data |
|---|---|
| The placed cell, flat | 48 + (\|*σ*(*x*)\| rem 3) ("magnitude first"): 48 … 50, or 46 for *x* = 2147483648 |
| A flat cell set, re-patterned or kept (a flat edge neighbour that a placement sets or re-patterns, or a flat cell that keeps its vein in a recompute) | 3*c* + \|*σ*(*x*) rem 3\| ("magnitude after"): 3*c* … 3*c* + 2 |
| A cell set by a placement, sloped (*s* ≥ 1) | 51 + 2*s* + (*x* mod 2) |

One placement on a flat cell assigns consecutive raw values to the placed cell and each
eligible edge neighbour (each neighbour set or re-patterned under claim 10). The assignment
order is placed cell, map north, map east, map south, map west, omitting ineligible neighbours.
Thus the visible variants of later neighbours depend on which earlier neighbours take a value.

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
| Map-generator seed range | 0 … 65535 | seed value | Random-map generation, claim 13 |
| Initial map-generator seed | 0 | unsigned seed value | Once per session, not once per ordinary-map game (claim 13) |
| Network setup seed range | 0 … 32767 | seed value | The original's own setup screen (claim 8) |
| Mixing constants *P*₀ … *P*₃, *Q*₀ … *Q*₃ | Formulas table | unsigned 32-bit values | Claim 3 |
| Bridge end-marker positions | 122 … 125 and 233 … 236 | positions in the overlay type list | Claim 10; pairs 122–123, 124–125, 233–234, 235–236 |
| Bridge piece base types *t*₀ | 74, 83, 205, 214 | positions in the overlay type list | Formulas table *Bridge piece type* |
| Bridge pieces per cell step | 3 | pieces (one raw value each) | Claim 10 |
| End-row types | 92, 94, 96, 98 and 223, 225, 227, 229 | positions in the overlay type list | Formulas table *Low-bridge end markers* |
| End-row and piece width | 3 | cells across the line | Claim 10 |
| Partner search limit | none | cells | Ends at the first cell that is not legal (claim 10) |
| Vein placement slope limit | 4 | slope shape index (S-0106) | Claim 10: at most 4 for the placed cell; above 4 blocks a flat placed cell's empty neighbour |
| Animation bounce draws | 3 raw values, then 3 ranged draws over −65535 … 65535 | draws | Claim 10, bouncing types of sprite animations only |
| Visceroid placement draw | 0 … 29 | ranged-draw result (integer) | One ranged draw R(0, 29) when a vehicle of a type with either visceroid flag is placed (claim 10, map-loading table) |
| Vein overlay position | 126 | position in the overlay type list | Claim 10; S-0009, claim 14 |
| Vein data base, flat placed cell | 48 | overlay data value | 46 for the one raw value of `quirks.random_magnitude_overflow` |
| Vein data base, sloped cell | 51 | overlay data value | Plus 2*s* and the lowest bit |
| Vein data step per connection pattern | 3 | overlay data values | Flat cell set, re-patterned or kept: 3*c* plus the remainder |

## State and ordering

| Stream state | Event | Result |
|---|---|---|
| Any scenario or unsynchronised stream state | Game start from the setup or menu screens | These two streams seeded with the game's seed; next raw value is x₀ (claim 8) |
| Map-generator stream not yet initialized | Session starts | Seed 0; next raw value is x₀ of the seed-0 sequence (claim 13) |
| Any map-generator state | Ordinary-map skirmish starts, including after returning to the menus | State unchanged; no seed selection or fallback draw (claim 13) |
| Any map-generator state | Random-map generation begins | Replaced by the state seeded from the selected random-map seed (claim 13) |
| Running map-generator stream | Low-bridge repair needs a random piece | Claim 13's accepted and rejected values advance the existing state; no reseeding |
| Any | Campaign continuation (next mission or restart) | No seeding; both streams continue (claim 8) |
| Running | A raw value is requested | Returns x<sub>n</sub>, advances to x<sub>n+1</sub> |
| Running | Clearing of the previous game begins (at a game start, before the map is read) | No draws until the clearing ends, and the same state then as now (claim 7). In the original the scenario random stream is suspended for that time; our engine need not have this state |
| Suspended | A raw value is requested | Returns 0; no advance (the original) |
| Suspended | Clearing ends | Running again, with the state it had when the clearing began (the original) |
| Running | Saved game loaded | Scenario random stream state replaced by the saved state (claim 9) |
| Running | Post-game statistics prepared after simulation stops | Eligible listed houses take claim 16's ranged requests in house order, continuing the scenario stream without reseeding; equal bounds do not advance it |

Observable ordering: because both streams start from the same seed, the first draws of a game from
the two streams are the same raw values when no other draw has been taken from either. The order
of draws within a tick is set by the specs of the systems that draw; every draw shifts all later
draws of the same stream, which is why those specs state counts and order.

**Draws during map loading.** Each row is a group of effects and scenario-stream draws
attributable to loading or game-start setup before the first tick. Rows give draw allocation
order: a group's random outcomes use the state left by all earlier groups, including conditional
draws. For example, changing a bridge's length can shift pre-placed-object, starting-unit and
animation choices; adding an animation at a later group cannot change an earlier group's choices.
Within a row the stated allocation similarly fixes which effect receives which value, not how an
implementation schedules its internal work. The table extends S-0009 claim 20 and S-0011 claim 34
with claim 10's draws. Rows marked "pointer" belong to a spec that does not yet state them.

| Group | Effects and draw allocation within the group | Owner |
|---|---|---|
| House creation, first round | one house-creation draw R(450, 1800) (claim 14) per house created, over S-0009 claim 19's house list in its order (skirmish and network games) | claim 14 |
| S-0009 early phase | list 1, list 2, random positions | S-0009, claim 20 |
| House creation, second round | the same list created again, one house-creation draw R(450, 1800) per house, in the same order (skirmish and network games; a campaign mission creates its houses once, here, with one draw each) | claim 14 |
| Initial fill | one per visited cell, water fill only | S-0009, claim 6 |
| Tags of cell tags | the draws of each tag created by a cell tag | S-0011, claim 34 |
| Overlay type stream | in the stream's order: the bridge pieces of each bridge completed by an end marker (3 per cell step) and the creation draws of cell animations (claim 10, *Animation creation draws*). Tile animations started by placement recalculations take their draws in this group too; the placement's cell-animation creation and immediate start-event draws precede that recalculation's tile-animation draws | claim 10 |
| Overlay data stream | no draw of its own; overwrites overlay data after all overlay-type placement effects and before the first whole-map recalculation (claim 15) | S-0009 claim 13; claim 15 |
| First recalculation pass | the creation draws of tile animations (S-0010, claims 12 and 19). In this pass and in everything before the second pass, a tile animation starts at the first recalculation of its cell; a cell recalculated earlier in loading (for example by an overlay placement or a bridge piece) starts it there, among that group's draws (low confidence). Each takes the animation creation draws of claim 10 | S-0010, claims 12 and 19; claim 10 |
| Vein candidates | for each candidate cell that passes the test, in reverse visiting order: the vein pattern draws of claim 10 | S-0009, claim 14; claim 10 |
| Terrain objects | none found (medium confidence) | — |
| Pre-placed objects | Values are assigned by kind as in S-0009 claim 43 and S-0203 claim 3 (vehicles, aircraft, infantry, buildings), each kind in entry order. An object's tag outcomes use the state left by its creation draw; its placement outcomes use the state left by its tag. The next object's creation uses the state left by this whole group. Of the placement draws, a vehicle of a type with either visceroid flag (Import mapping) takes one ranged draw R(0, 29) when it is placed (the rule S-0009 claim 46 states for starting units), and an infantry unit placed while the map loads takes none (its standing position comes from the map, S-0208 claim 5); the other placement draws belong to the placement specs (medium confidence). A building's upgrade creations use the state left by that building's tag and contribute their own creation draws to later choices | claim 10; S-0011, claim 34; placement specs (pointer) |
| Smudges | none found (medium confidence) | — |
| Second recalculation pass | the creation draws of the tile animations that S-0010's second pass (claim 12) starts; these replace the ones started earlier in loading (S-0010, claim 19). Each takes the animation creation draws of claim 10 | S-0010, claims 12 and 19; claim 10 |
| Unused-house cleanup | after the second pass: tile-animation creation draws from occupied-ground-cell recalculations caused by removal, only when an animation has not already started since that pass (claim 14) | S-0009 claim 28; S-0010 claim 19; claims 10 and 14 |
| S-0009 late phase | list 3, late re-assignments | S-0009, claim 20 |
| Starting units | house by house, as S-0009 claim 46 orders them: each house's base unit and other starting units, each with its type choice, creation draw and placement draws; after the last house, one ranged draw over 0 … 65535 whose value has no effect | S-0009, claims 45 and 46; claim 10 |
| Initial crates | crate placement draws when the crate setting is on; S-0520 claim 3 owns each placement's draw composition, and S-0412 claim 10 owns the count | S-0412 claim 10; S-0520 claim 3 |
| Secret labs | the conditional secret-lab choice draws, after initial crates and before end-of-loading tags and the game-start recalculation | S-0407 claim 14; claim 14 |
| Fence lines | When fence posts in the imported rules data can link on the map: the creation draws of the fence sections built between them, and the recompute draws of claim 10 for vein cells the line crosses. If no such posts are present, this row has no draws (low confidence) | fence rules (pointer); claim 10 |
| End of loading | the tags created at the end of loading, in registration order | S-0011, claim 34 |
| Game-start recalculation pass | in visiting order, the creation and immediate start-event draws of each tile animation this pass starts, after starting units, initial crates, secret labs and end-of-loading tags and before the first tick, even when no starting units were placed. A cell that started one in the second pass starts no second one and takes no further animation-creation draws in this pass | S-0010, claims 12 and 19; S-0011, claim 34; claim 10 |
| Resource-glint setup | Each eligible cell's ranged choice uses the state left by game-start recalculation and preceding eligible cells in visiting order. A zero result includes animation-creation and immediate start-event draws in that cell's group, changing the state available to later glint choices. No choices when no glint type is configured; all effects belong to setup before tick 0 | claim 14 |

S-0009 points to this table for the rows between the initial fill and the late phase, and states
the starting units' draws itself (claim 46). S-0011 claim 34 cites this spec for the
overlay and vein-candidate groups, claim 14's conditional cleanup tile-animation draws,
and the secret-lab group before remaining tags. Creating terrain objects, smudges and tunnels
from the map was not found to take draws (medium confidence, `unmeasured`: no scenario
isolates these creations between known stream checkpoints). Neither S-0009
nor S-0011 lists the two *House creation* rows, which claim 14 states (R17). Neither lists the
*Fence lines* row either; it matters only for data with fence posts. Where each group
sits is `unmeasured`, apart from what scenario R8 shows (the bridge and vein groups' own draws,
that each bridge or vein site follows the one before it in the stream, that the overlays of R8 C
take no draw and, through R8 G, the total of the pre-placed objects' draws before the starting
units, and R8 I, R10 and R13 for the cell-animation, game-start recalculation and resource-glint ordering): a group's position
shows only as a shift of later draws, and the groups between the
overlay type stream and the starting units are not yet predicted draw by draw on one map.

## Edge cases and quirks

- **`quirks.ranged_draw_half_span`** (default: on, the original). A ranged draw whose bounds differ
  by exactly 2147483647 takes no raw value and returns the lower bound plus 2147483648, wrapped,
  which is outside the requested range. Off: the general rule with 31 kept bits, one raw value,
  result inside the range. With either setting the importer warns where data can produce such a
  span (claim 5): the switch keeps the original's result for faithful replays, while the warning
  tells the author that the data reaches a case the original handles wrongly.
- **Spans of 2147483648 or more** (departure, no switch). The original hangs (claim 5). Our engine
  draws with all 32 bits kept and rejection, consuming 1 … 251 raw values. Claim 5 proves
  acceptance within that bound for every stream state; no fallback or cap is added. The importer
  warns where data can produce such a span.
- **`quirks.random_magnitude_overflow`** (default: on, the original). In the "magnitude first"
  signed remainder (claim 6), the raw value whose signed reading is −2147483648 keeps its negative
  sign, so the remainder lies in −(*n* − 1) … 0 instead of 0 … *n* − 1, for that one raw value in
  2³². Off: the magnitude is taken as 2147483648 and the remainder is never negative. The topic
  specs that use this form refer to this switch. One user is the flat placed vein cell of claim 10,
  whose overlay data becomes 46 instead of 48 … 50 for that raw value. That cell is then not a
  pattern centre (Terms), so a later placement or recompute treats it as an ordinary flat vein
  cell.
- **Seed from the host's clock** (departure, claim 8): the seed is a game-setup value in our engine.
- **Terrain-image pattern kept for the session** (departure, no switch; claim 11; a design choice
  of scope in the glossary's sense, which the project lead accepts case by case). The original
  builds S-0010's 8 × 8 pattern once per session, so the ground images of a game depend on earlier
  games in the same program run and can differ between the machines of a network game. Our engine
  builds the pattern at every map load, at the point S-0010 names, from the unsynchronised stream
  seeded for that game, so the images depend only on the game's seed and setup. In the first game
  of a session the two agree as long as the stream has given the same draws before the pattern is
  built; later games can show other ground images. Nothing else changes, since the pattern never
  reaches game state.
- **`quirks.colour_list_item_triples`** (default: on, the original). Claim 17's reading of a
  particle type's colour list ignores parentheses and takes comma-separated items in threes, so one
  group with a missing, extra or empty component shifts every later group. Off: a parenthesis-aware
  reading. The text is read from left to right. A group starts at an opening parenthesis and ends at
  the first closing parenthesis after it; reading then continues after that closing parenthesis. A
  group with no other opening parenthesis inside it is one entry. A group with another opening
  parenthesis inside it is malformed and gives nothing, so `((1,2,3))` and `(1,(2,3)` give no entry,
  and in the first the last closing parenthesis lies outside every group. Text outside groups, a
  closing parenthesis with no opening one and an unclosed final group included, gives nothing.
  Inside a well-formed group the text is split at every comma, empty parts kept; the first three
  parts are the red, green and blue components, each read by claim 17's decimal form and reduced
  modulo 256; a missing part or one without a digit gives 0, and parts after the third are ignored.
  On a clean list the two readings agree. With either setting the importer warns under claim 17. The
  off setting is a rule of our own (`design`): the original has no parenthesis-aware reading, so it
  is not measurable on the original, and our engine's tests check it on its own.
- **Identical starting states.** Both streams start each game from the same seed. A mod cannot
  change this, and our engine keeps it: it costs nothing and keeps the unsynchronised stream's
  first values the same as the original's for a given seed.

## Import mapping

The streams themselves read no rules or art keys. Claim 10 names overlay types by their position
in the rules' overlay type list (the end markers 122 … 125 and 233 … 236, their end-row types, the
vein overlay 126 and the bridge piece types of the Formulas tables), as S-0009 does, so the list
order is part of the data format; the positions are fixed in the original, whatever types the data
puts there. The placement rules of claim 10 read these keys:

| Key | Section | Form | Absent | Feeds |
|---|---|---|---|---|
| `IsVeins` | an overlay type's section | boolean | no | the veins flag (Terms) of the vein placement test and the connection pattern |
| `SmallVisceroid`, `LargeVisceroid` | a vehicle type's section | boolean each | no | either one on: placing a vehicle of the type takes one ranged draw R(0, 29) (map-loading table, pre-placed objects; S-0009 claim 46 for starting units) |
| `CellAnim` | an overlay type's section | animation-type name, case-insensitive; none and <none> clear it | no animation at type creation; an omitted or empty layered value keeps the previous value | cell animation of ordinary placement, S-0009 claim 12; not started by low-bridge markers or their end-row and piece writes (claim 14) |
| `RandomRate` | an animation type's art section | two integers | no random rate; −1 preserves that end | creation stage-delay draw, S-0109 claim 3; applies equally to tile animations started by cleanup or the game-start pass |
| `TileNNAnim`, `TileNNAttachesTo` | theater section named by the tileset's `SetName` | animation-type name and integer sub-tile index; NN is the tile number within its set | no animation, attaches-to −1 (never starts) | tile-animation eligibility for cleanup and game-start recalculations; full mapping and placement fields belong to S-0010 claim 19 and its Import mapping |
| `TiberiumChainReaction` | an animation type's art section | boolean | no | start-event chain-reaction draws, S-0401 claim 20; no resource means no chain-reaction draws |
| `SpawnsParticle`, `NumParticles` | an animation type's art section | particle-type name, integer count | no particle type, 0 | requested particle groups in claim 15 at map load and during play; event gates belong to S-0109 claim 14 |
| `Scorch`, `Crater`, `ForceBigCraters` | an animation type's art section | boolean each | no each | smudge kind request and large-crater selection form in claim 15 at map load and during play; shared terrain-height gate belongs to S-0109 claim 14 |
| `BehavesLike` | a particle type's rules section | case-insensitive name: Gas, Smoke, Fire, Spark or Railgun; at most 31 text bytes | no recognised behaviour (also for an unrecognised name or omitted key) | Smoke adds the speed raw value; Railgun uses lifetime modulus 10, every other value uses the lifetime field |
| `MaxEC` | a particle type's rules section | signed integer | 1 at type creation; omitted or empty layered values keep the previous value | lifetime variation modulus for non-railgun particles; zero warns (claim 15) |
| `ColorList` | a particle type's rules section | written as comma-separated parenthesised RGB triples, at most 511 text bytes; read as comma-separated items taken in threes, decimal components reduced modulo 256 (claim 17, `quirks.colour_list_item_triples`); malformed text warns | empty, also when omitted or empty in a later layer | a list with at least one entry enables the conditional starting-colour draw |
| `StartColor1`, `StartColor2` | a particle type's rules section | generic byte triple each (S-0003 claim 11), components reduced modulo 256 | black each at type creation; omitted layered values keep the previous value | nonblack component in either colour enables interpolation when the list is nonempty |
| `Normalized` | a particle type's rules section | boolean | no at type creation; omitted or empty layered values keep the previous value | normalised particle timing, no extra scenario-stream creation draw |
| `OreTwinkle` | rules [General] | animation-type name, case-insensitive | no animation at database creation; omitted or empty layered values keep the value in force | resource-glint type for claim 14; its art settings determine applicable creation draws |
| `OreTwinkleChance` | rules [AudioVisual] | signed integer | 50 at database creation; omitted or empty layered values keep the value in force | resource-glint frequency *f*, not a percentage; choice range R(0, *f* − 1), with subtraction wrapping in signed 32-bit arithmetic |

Smudge type kind flags and footprint dimensions are imported by S-0312 claim 26 and its
Import mapping; morphable tile eligibility is imported by S-0010. The visible image-frame
bounds and frame count determining μ are sprite data (S-0109 claim 25), not rules values.
The no-loop-delay creation condition is supplied by the creating feature, not an additional
INI field. Terrain height for claim 15 is imported terrain geometry under S-0010 and S-0106,
with no separate loading or play-time height setting.

The contained resource value used for glint eligibility comes from the resource cell and the
resource value field imported by S-0401, not from the glint frequency.

The land types and slope shapes that the vein placement test reads come from S-0211 and S-0106. The
game seed is a game-setup value (claim 8). Map-generator seeding has no rules or art key:

| Input | Form | Absent or ordinary-map case | Feeds |
|---|---|---|---|
| Random-map setup seed setting | unsigned 16-bit setup value, 0 … 65535; not an ordinary-map INI key | During random-map generation only, 0 selects the unsynchronised R(0, 65535) fallback | Replacement map-generator state, claim 13 |
| Ordinary-map selection | already-defined map data, not random-map settings | No seed is imported or selected; retain the session's map-generator state, initially seeded with 0 | Low-bridge repair choices, claim 13; the game seed is not a substitute |

## Measurement scenarios

All scenarios use the measurement setup: skirmish at speed 4 unless a part explicitly
names a deferred network or campaign test, with no engine add-on and no mods (the named test files of a scenario excepted). Scenarios R2, R3, R5, R6, R7, R8 and R9 use the glossary's *random-state
recorder*, which here means a read-only program that reads, without changing anything, the running original's seed of the current
game and the current state of the scenario random stream: the 250 values x<sub>N−250</sub> …
x<sub>N−1</sub>, seed values included, where *N* is the number of raw values taken since the
seeding; the recorder gives *N* only modulo 250 (its *position*). It reads the unsynchronised
random stream's state in the same way (S-0010's scenario V2 part B uses this), and, where a
scenario says so, the map-generator stream's seed and state (scenario R9). It is a
recorder in the glossary's sense: it fixes no random state, and every scenario takes the seed the game chose.
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
  session. With the random-state recorder, read the seed and the stream state at the first tick of each
  game, and note the host's wall-clock time and its uptime in milliseconds at that moment. Compute
  the 250 seed values F(seed, *k*) and advance the recurrence step by step. The recorder gives the
  number of raw values taken only modulo 250 (its position): find the smallest count *N*, congruent
  to the position modulo 250, for which the computed state equals the recorded one.
  *Expected:* such an *N* exists (well below 100 000) in every game; the unsynchronised stream's
  state is the same seed's state advanced by some count; for the games of one session, the
  difference of two seeds, modulo 2³², equals the wall-clock time between their first ticks in
  milliseconds, within 10 seconds (the difference in loading time). A seed not taken from a
  millisecond counter lands in that window with chance about 5 in a million.
  *Falsified if* no *N* up to 100 000 matches in some game; the unsynchronised stream's state is not
  the same seed's state advanced by any count up to 100 000; or a seed difference falls outside the
  window.
  *Secondary expectation* (for the counter's starting point only): each seed equals the host's
  uptime in milliseconds at the first tick, modulo 2³², minus at most 60 seconds of loading time. If
  it does not, only the medium-confidence part of claim 8 about the counter's start is wrong.
  (b) *Local network* (kept for the future multiplayer spec, not required of the skirmish engine).
  3 local-network games of two machines, with the random-state recorder on both.
  *Expected:* in each game both machines give the same seed, in 0 … 32767, and the same stream
  state at the first tick. If the seed were any 32-bit value instead, all three seeds would fall
  in 0 … 32767 with chance 2⁻⁵¹ under a uniform full-range alternative, so three games
  discriminate strongly against that alternative, without proving that no other seed is possible.
  *Falsified if* the seeds or states of the two machines differ in some game, or a seed exceeds
  32767.
  (c) *Campaign continuation* (kept for the future campaign spec, not required of the skirmish
  engine). Win a short campaign mission and let the next one start; read the
  seed and state at the last tick of the first mission and at the next mission's first tick. Then
  lose a short campaign mission and restart it without returning to the menus; read the seed and
  state at the lost mission's last tick and at the restarted mission's first tick. 1 run of each.
  *Expected:* in both, the seed is unchanged and the state is the previous mission's final state
  advanced by some count. *Falsified if,* in either, the seed changes or the state matches a fresh
  seeding.
- **R3, saved games (claim 9).** A small test map with one human house and ten infantry of a type
  with idle animations in the observed area, no crates and no other moving objects in that area,
  plus one distant bystander computer player under the glossary's void rule. Use the random-state
  recorder and scripted input; repeat any run voided by the bystander.
  *Procedure:* after tick 300, open the options menu to stop simulation, read the recorder,
  save the game without resuming play, and read the recorder again. Let that
  run continue without loading for two game minutes, video-recorded (run 0). Then load
  the save twice (runs 1 and 2), read the recorder at load completion before any resumed tick,
  give no orders and video-record two game minutes. Use scripted input to resume each run
  identically.
  If the recorder cannot isolate the pre-tick load checkpoint, the state comparison is
  inconclusive, not evidence of failed restoration.
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
  with one human house and one distant bystander computer player, kept out of the observed area
  under the glossary's bystander void rule. The bystander owns a construction yard and has the
  game-wide starting credits, 0 for every house (the glossary's *bystander computer player*, with
  the human house's exceptions below). The human house owns one power plant, outside the observed
  area. A skirmish drops pre-placed structures owned by a human player's house (S-0009 claim 27),
  so the measurement harness creates it on a stated cell after load and before tick 0. The observed
  area contains
  nothing that moves or animates except the named test infantry.
  With the random-state recorder, read the stream
  position at every tick and predict/account for the bystander's and any other unrelated draws
  under their owning specs; a tick whose unrelated draws cannot be accounted for is void for
  the stream comparison. Control interval: for the first 300 ticks the observed area holds no
  infantry. *Expected:* after accounting for the bystander's and other identified unrelated
  draws, there is no unexplained advance attributable to the observed area in any of those
  ticks. *Falsified if* an unexplained advance remains, in which case the scenario must first
  remove or account for its cause. Then a trigger places one infantry of the human house that
  has idle animations; leave it standing idle for 3 game minutes. From the recorded state,
  compute the raw values that each idle attempt takes (S-0107, claim 14: a timer draw
  R(0, 2147483646), then *r* = R(0, 10), then δ = R(0, 7) when the outcome is a turn) under
  claim 4's rule, including rejected values. Also predict the guard behaviour-processing delay
  draws and their timing under S-0215 claims 6 and 24, and the targeting-timer restart draws of
  S-0215 F6 that the per-tick search of S-0309 claim 1 takes. The first of those restart draws
  is identified, not predicted, since no spec gives its timing for every infantry type: it is
  the placed infantry's first *R*(0, 2) draw after its creation tick that is left once the
  bystander's and other unrelated draws and the predicted guard processings are accounted
  for. The placed infantry's type has a weapon, so that it takes those restart draws (as in
  S-0107 scenario M9). Exactly one such candidate must remain: with none or more than one, the
  part is inconclusive. Each later restart draw is then predicted from the one before by F6. If a
  bystander voids the window early, the part counts only when at least four idle attempts fall
  before the first void tick.
  *Expected:* at every non-void tick where
  the position advances, it advances by exactly the predicted count after accounting for the
  bystander's and other unrelated draws; the infantry's visible outcome matches the predicted
  *r* and δ; the idle timer expires after the length S-0107's timer formula gives from the
  predicted unit-interval value; and, if expiry falls between guard processings or eligibility
  conditions are not met, the next idle attempt occurs at the first subsequent guard processing
  that meets all of S-0107 claim 14's conditions. *Falsified if* any unexplained advance remains
  (counting the identified first restart draw as predicted), the later restart draws do not
  follow from the identified one by F6, any advance differs from the
  prediction (for example by counting one raw value per ranged draw, or none for rejections),
  an outcome differs from the prediction, or an attempt occurs before expiry or at a processing
  other than the first eligible one. (b) At the tick the
  infantry is placed, *expected:* after accounting for other predicted draws, the position
  advances by at least one (its creation draw; R8 G checks the exact count of a creation,
  where nothing else draws). *Falsified if* the position shows no advance attributable to the
  creation draw in that tick. (c) *Production.* The same map, with the human house also owning
  a power plant and an infantry-producing building before the control interval. A skirmish
  drops pre-placed structures owned by a human player's house (S-0009 claim 27), so the
  measurement harness creates them on stated cells after load and before tick 0. Scripted input
  before tick 0 also sets the human house's credits to a value, kept in the record, that pays
  for every production of this part. The harness creations of parts a and c and this credits
  change are exceptions to the glossary's measurement setup, and the record states each. The
  control interval then runs for 300 ticks from tick 0 and must still show no unexplained advance
  with them in place (otherwise the part is inconclusive). After the control interval, scripted
  input starts the production of one
  infantry unit. *Expected:* after accounting for unrelated draws, the position advances by
  at least one in the tick the production begins (the unit's creation draw), before the unit
  leaves the building. *Falsified if* there is no advance attributable to that creation in
  that tick (for example because the draw comes only when the finished unit leaves the building).
  Repeat with a vehicle-producing building and one vehicle in place of the infantry, with the
  same expectation, for a produced vehicle.
- **R6, unsynchronised draws stay out of game state (claim 11).** (a) The map and recorder of R5 (a),
  with two infantry instead of one and the same distant bystander computer player. 4 runs of
  2 game minutes: in 2 runs keep the infantry selected and click them repeatedly so that they
  give voice responses (each response takes draws from the unsynchronised stream); in 2 runs
  never select them. Predict and account for the bystander's and other unrelated draws, and
  include the infantry behaviour-processing delay draws as in R5 (a). Each infantry's first
  targeting-timer restart draw is identified as in R5 (a). If both fall on one tick, both ways
  of giving that tick's two values to the two infantry are tried, and the prediction that holds
  for every later draw is used. *Expected:* in every
  non-void tick the per-tick advance of the scenario random stream equals R5's prediction from
  the idle attempts and behaviour-processing delays, after accounting for the bystander's and
  other unrelated draws; ticks in which a voice response starts show no extra advance of the
  scenario random stream. *Falsified if* any unexplained advance remains, or a voice response
  coincides with an advance that the predicted scenario-stream draws do not explain.
  (b) *Play uses.* With the random-state recorder reading the unsynchronised stream, on R5 (a)'s
  map, where nothing else in the observed area plays a sound, set off each of these once, at
  least 30 ticks apart: a voice response (selecting a unit) and the start of a new music track
  when the previous one ends (with the music set to random play). Account separately for any
  bystander or other unrelated presentation draws; void a tick whose unrelated draws cannot
  be accounted for. 3 runs. *Expected:* the unsynchronised stream's position has an advance
  attributable to each effect in the tick it starts. *Falsified if* an effect starts with no
  advance attributable to it after accounting for unrelated draws.
- **R7, network checksum draws (claim 12; kept for the future multiplayer spec, not required of the skirmish engine).** Two machines in a network game on R5 (a)'s map,
  with the random-state recorder on one machine (it reads that machine's stream; both machines hold the
  same state). During the control interval, *expected:* after accounting for the bystander's
  and other unrelated draws as in R5 (a), the remaining advance is exactly 2 in every tick,
  apart from occasional ticks with one more ranged draw R(0, 2) (whose raw values the recorded
  state predicts). Repeat on the same map as a skirmish: *expected:* no remaining advance
  after that accounting. *Falsified if* the network game's remaining advance per tick is not
  2 (allowing the extra draws), or the skirmish shows an unexplained advance in the control
  interval. Void a tick whose unrelated draws cannot be accounted for.

- **R8, overlay placement draws (claim 10, map-loading table)** *(statistical runs)*. No run
  needs a chosen seed: each run uses the seed the game chose, which the random-state recorder reads.
  *Setup common to all parts:* a skirmish on a small purpose-made map of a theater whose clear and
  water tiles name no tile animation (check in the theater data), with the random-state recorder; no
  overlay other than the ones named. Before running, check in the rules data that the end-marker,
  vein and other overlay types used have no cell animation; otherwise the part that uses them is
  inconclusive. *Procedure for each run:* read the seed and the state at the first tick, find *N* as
  in R2 (a), compute x₀ … x<sub>*N*−1</sub> from the seed (claims 2 and 3), and read every piece's
  or vein cell's overlay type and data from a saved game made at the first tick.
  - (A) *Bridges.* Two bridges over water, each a pair of matching end markers of the family at
    positions 122 and 123 with *d* = 4 water cells between their end rows, the marker of position
    123 on the west side (lower x) of the one of position 122, as the search directions of the
    table *Low-bridge end markers* require (for the other pairs: 124 north of 125, 234 west of 233,
    235 north of 236); both markers of the
    first bridge come before both markers of the second in the overlay type stream (S-0009, claim
    12). 5 runs for each of the four end-marker pairs, rotating the bridge geometry
    to the y axis for pairs 124/125 and 235/236. *Expected:* there is a position *p* such
    that the first bridge's 12 pieces, in the order of claim 10, have overlay types *t*₀ +
    (x<sub>*p*+*j*</sub> mod 4), *j* = 0 … 11, with *t*₀ from the Formulas table for
    the tested pair; and the second bridge's 12 pieces match x<sub>*p*+12</sub> …
    x<sub>*p*+23</sub> in the same way. A wrong position matches one bridge with chance 4⁻¹², so
    a unique match is expected. Multiple matching positions are inconclusive; repeat the run.
    The second bridge following straight on the first shows that the first
    took exactly 3*d* = 12 raw values. *Falsified if* no *p* matches in some run, the base type
    differs, the pieces match in another order, or the second bridge starts at another position
    than *p* + 12. The pieces' overlay data is not checked, because the map's overlay data stream
    sets it afterwards (S-0009, claim 13). *Inconclusive if* no bridge appears: the marker types
    are then not created on this map (S-0009, claim 12).
  - (B) *Veins.* Four vein overlays (position 126), each with overlay data 48 in the map's
    overlay data stream, which makes it a vein candidate (Terms); the load-time pass empties these
    cells before it tries their placements (claim 10, *Map loading*). Each lies on a flat clear cell
    whose four edge neighbours are flat, clear and empty, every two sites at least four cells apart
    in x or in y. 5 runs. *Expected:* there is a position *p* such that, taking the sites in
    the reverse of S-0009's visiting order and each site's raw values in the order of the Formulas
    (placed cell, then map north, map east, map south, map west), the 20 raw values
    x<sub>*p*</sub> … x<sub>*p*+19</sub> give every cell's data by the *Vein pattern values*: the
    placed cell's data is 48 + (|*σ*(*x*)| rem 3) (46 for the one raw value of
    `quirks.random_magnitude_overflow`), and each neighbour's data, taken modulo 3, is
    |*σ*(*x*) rem 3| (the neighbours' 3*c* is checked in part E). A wrong
    position matches with chance 3⁻²⁰ under independent uniform remainders, so a unique
    *p* ≤ *N* − 20 is expected. Multiple matches are inconclusive; repeat the run. Each site's 5
    values following straight on the previous site's shows that a site with four empty flat
    neighbours takes exactly 5 raw values, so this also tests that all four such neighbours are
    set (claim 10, medium confidence). *Falsified if* no *p* matches in some run, or the
    match needs a gap or another neighbour order.
  - (C) *Overlays that take no draw.* The map of part A, with these overlays added on land cells
    that lie off both bridges' lines and end rows, and in the overlay type stream after the
    second marker of the first bridge and before the first marker of the second bridge: a run of 10 wall overlays, 10 ore cells and 10 gem
    cells. 5 runs. *Expected:* as in part A, the second bridge's pieces match
    x<sub>*p*+12</sub> … x<sub>*p*+23</sub>, so the overlays placed between the two bridges took
    no raw value (their cells hold no tile animation, by the setup). *Falsified if* the second
    bridge starts at a later position than *p* + 12.
  - (D) *End rows and the partner search.* The map of part A with six changes, all in the
    overlay type stream after the first bridge's second marker and before the second bridge's
    first marker unless stated: (i) one wall overlay on the second bridge's line between its two
    end rows, placed before the second bridge's later marker (the search passes it; it is then
    overwritten by a piece); (ii) a lone marker of position 124 whose column holds no marker of
    position 125; (iii) a marker of position 122 one of whose end-row cells already holds an ore
    overlay placed earlier in the stream; (iv) a third bridge of the pair 122 and 123 whose two
    end rows are side by side (*m* = 1), after the second bridge (the marker of position 123 first
    in the overlay type stream); (v) a reversed pair: a marker of
    position 122 west of one of position 123, 4 water cells apart, in a row of its own; (vi) a
    fourth bridge of the pair 122 and 123 with *m* = 1, after the third, the marker of position
    123 first in the overlay type stream and lying west of the other, placed where the map's
    edge runs diagonally to the axes (S-0009 claim 1) so that the earlier marker's end row lies
    wholly inside the map and exactly one cell of the later marker's end row lies outside it. 5
    runs, each the first game of a fresh launch (so the stand-in cell of S-0009 claim 44 holds no
    overlay when read). Read the overlay types
    of every end-row and piece cell from the first-tick save. *Expected:* the second bridge's
    pieces still match x<sub>*p*+12</sub> … x<sub>*p*+23</sub> (so (ii) and (iii) took no raw
    value) and the wall's cell holds a piece; marker (ii)'s end row holds type 96 in its three
    cells, marker (iii)'s three cells hold what they held before (the ore cell keeps its ore, the
    others no overlay), and no cell holds a marker type; the third bridge takes 3 raw values right
    after the second bridge's 12, and its pieces replace the later marker's own end row (types
    74 … 77 on the three cells of that row, the earlier marker's row keeping type 94); the
    reversed pair (v) writes its two end rows and no pieces, and takes no raw value, because each
    marker searches away from the other; the fourth bridge (vi) takes 3 raw values right after
    the third bridge's 3, and the two cells of the later marker's end row that lie inside the map
    hold piece types 74 … 77.
    *Falsified if* the second bridge's pieces start later than *p* + 12, an end row or marker
    cell holds another type than expected, the reversed pair holds pieces, the third bridge
    takes another number of raw values or fills other cells, or the fourth bridge takes 2 raw
    values (none for its piece outside the map) or none.
  - (E) *Vein placement test and connection pattern.* The map of part B with three additions:
    (i) a fifth site, between two of part B's sites in the visiting order, whose map-north
    neighbour is a water cell (it fails the test); (ii) a sixth site whose map-east neighbour holds
    a wall overlay (it fails the test); (iii) a pair of sites two cells apart along x, at (*x*, *y*)
    and (*x* + 2, *y*), both flat on flat clear land with flat clear empty neighbours, so that they
    share the edge neighbour (*x* + 1, *y*). 5 runs. *Expected:* sites (i) and (ii) hold no vein
    after loading and take no raw value (part B's 20-value match still holds, with the pair's
    values inserted at its place in the reverse visiting order); the pair takes 5 raw values for
    the site at (*x* + 2, *y*), which comes first, and then 5 for the site at (*x*, *y*), because
    the shared neighbour, set with *c* = 2 by the first (its other neighbour, the second site, has
    been emptied and holds no overlay yet), has *c* = 10 when the second is placed and is
    re-patterned. After loading, ⌊data / 3⌋ is 10 for the shared neighbour, 4 for each cell map
    north of a site, 1 for each cell map south of a site, 8 for the cell map east of the site at
    (*x* + 2, *y*) and 2 for the cell map west of the site at (*x*, *y*). Had the candidates kept
    their vein with data 48 until tried, the shared neighbour would get *c* = 10 at once and the
    second site would take only 4 raw values. *Falsified if* a failing site holds a vein or takes
    a raw value, the pair takes another number of raw values (4 for the second site in
    particular), or any data value's ⌊data / 3⌋ differs from the *c* of claim 10. Before running,
    check in the rules data that the vein overlay type has the veins flag: the second site of the
    pair passes the placement test only then, because its east neighbour already holds a vein.
    Otherwise the pair's part is inconclusive.
  - (F) *Animation creation draws.* A map rules override (S-0006) gives a test overlay type a
    cell animation whose animation type has a random rate pair that gives a range (S-0109 claim
    3) and is not a bouncing type, with no particles, scorch marks, craters or chain reaction in
    its middle event. Place three such overlays between the two bridges of part A. 5 runs.
    *Expected:* the second bridge's pieces start at *p* + 12 + *k*, where *k* is the number of raw
    values that the three stage-delay draws consume by the range rule (claim 4), computed from the
    recorded sequence. *Falsified if* the shift differs from *k*, for example by 3 (one raw value
    per animation) or by 0. This is variant (i). Variant (ii), *bouncing types:* the same, but the
    cell animation's type is a bouncing type (S-0109 Terms) with no random rate pair, so no
    stage-delay draw, and with no particles, scorch marks, craters or chain reaction in its
    middle event. *Expected:* the shift *k* is, for each of the three animations in turn, 3 raw
    values plus the raw values of three ranged draws R(−65535, 65535) by the range rule, computed
    from the recorded sequence. *Falsified if* the shift differs from that *k*.
    Variant (iii), *chain reaction without resource:* repeat variant (i) with the animation's
    chain-reaction flag on, particles and smudge effects absent, and no resource in any cell
    containing an animation's position (for the overlay animations these are the cells two steps
    along +x and +y, not the overlay cells). Use a sprite with more than one image frame so μ
    is positive. 5 runs at speed 4, no engine add-on and no mods (the named test files of a scenario excepted).
    *Expected:* the shift is still exactly the stage-delay draws' *k*, with no chain-reaction
    draws. *Falsified if* the shift needs any extra raw value. Repeat using tile animations on
    resource-free displaced cells in R10; each creation must have the same no-extra-draw result.
  - (G) *Pre-placed objects.* S-0009's scenario M26, variant G: its map, its five dropped entries
    and its neutral vehicle, and its prediction of the load's draws from the recorded seed, with
    these entries added, all owned by the neutral house and at least 12 cells from both base
    cells: a vehicle of a type with either visceroid flag; an infantry unit; and a building whose
    entry names one upgrade (S-0009 claim 41). 3 runs. *Expected:* the stream position at the
    first tick equals the prediction in which each created object takes one creation draw, the
    visceroid vehicle also one ranged draw R(0, 29) (its raw values computed by the range rule),
    the infantry unit no placement draw, the upgrade a creation draw of its own, and each dropped
    entry none. *Falsified if* the position fits instead a prediction missing an object's creation draw,
    without the R(0, 29) draw,
    with a draw for the infantry unit's placement or without the upgrade's creation draw. *Inconclusive if* it fits no prediction (an
    unexplained draw, for example of the building's own placement, which the placement specs
    own).
  - (H) *Tile animation under a piece.* Part A's map, with the first bridge built over land: one
    cell on its line strictly between the two end rows holds a tile that no other cell of the map
    uses, and a named test file (S-0010's theater data) gives that tile a tile animation whose
    type has a random rate pair that gives a range and is not a bouncing type, with no start-event
    effects. 5 runs. *Expected:* the first bridge's pieces match the raw values as in part A
    except for a gap of *k* raw values right after the piece on that cell, where *k* is the number
    of raw values that the animation's stage-delay draw consumes (range rule), and the second
    bridge starts at *p* + 12 + *k*. *Falsified if* the gap is missing or falls elsewhere among
    the first bridge's pieces (*k* is at least 1, since the range is not a single value).
    *Inconclusive if* the animation does not appear on the cell.
  - (I) *Cell animation before tile animation.* Reuse S-0009 M6e parts A to C for the displaced
    position and chain-reaction cell, and part D for draw order: its named cell and tile animation
    types have distinguishable non-singleton random-rate ranges and no other creation effects.
    Skirmish at speed 4, no engine add-on and no mods (the named test files of a scenario excepted), 5 runs with the
    random-state recorder. Record the stream state and each animation's initial stage delay
    at the overlay placement, before later passes replace the tile animation. *Expected:* the
    cell animation is at the top corner of cell (*x* + 2, *y* + 2); the recorded incoming stream
    predicts its rate draw before the tile animation's rate draw. *Falsified if* the position
    differs or the delays and outgoing state fit the reversed draw order instead. A run where
    both possible orders happen to give the same observations is inconclusive; repeat it.

- **R9, map-generator stream (claim 13).** A skirmish set up from the setup screens, in a game
  mode whose map selection offers a random map (check before running), with one computer player
  and the random-state recorder, which here also reads the map-generator stream. Open the
  random-map setup screen, read the unsynchronised stream's state, ask the screen once to choose
  all its settings at random, and read the unsynchronised stream's state again. Generate the map,
  use it and start the game. At the first tick read the map-generator stream's seed and state,
  then play 3 game minutes, with the starting units moving and fighting, and read the
  map-generator stream's state again. No low bridge is repaired during this interval. 3 runs.
  *Expected:* (a) the map-generator seed lies in 0 … 65535, and there is a count *N*, found as in
  R2 (a), for which the seed's computed state (claims 2 and 3) equals the recorded one; (b) the
  seed equals *x* mod 65536 for one of the raw values *x* that the unsynchronised stream gave
  between its two readings (a wrong hypothesis passes this with chance about *n* / 65536 per run,
  for *n* such raw values); (c) the map-generator stream's state at the end of play equals its
  state at the first tick; (d) in the first run, made as the first game of a fresh launch, the
  unsynchronised stream's state at its first reading equals the state computed from the seed
  that the recorder reads at that moment (the seeding at program start, claim 11), advanced by
  some count up to 100 000.
  *Falsified if* a seed exceeds 65535 or no *N* up to 100 000 matches; the seed matches none of
  those raw values; the map-generator stream's state changes during play; in the first run, no
  count up to 100 000 matches the unsynchronised stream's first reading; or no game mode's map
  selection offers a random map. *Inconclusive if* the generator fails to produce a playable map.
  (e) *Setting 0.* 3 further runs, each the first game of a fresh launch, in which the screen is
  never asked to choose its settings at random: generate the map, use it and start the game, and
  at the first tick read the game seed and the map-generator seed. Compute the unsynchronised
  stream's raw values x₀, x₁, … from the game seed (claims 2, 3 and 8). *Expected:* the
  map-generator seed equals x<sub>*k*</sub> mod 65536 for some *k* below 1000 (a wrong hypothesis
  passes with chance about 1000 / 65536 per run, so all three runs pass with chance below 4 in a
  million). *Falsified if* in some run the map-generator seed matches no such
  x<sub>*k*</sub>. A seed of 0 is allowed when the predicted draw gives 0.
  (f) *In-play low repair.* Use S-0511 claim 20's low-bridge fixture and repair procedure:
  a small stock-data test map with a damaged or destroyed low bridge, its hut, a human
  engineer and a distant bystander computer player, at a recorded fixed speed, no engine add-on and no mods (the named test files of a scenario excepted). Include several strips needing random intact pieces and an intact end
  piece needing no random choice. Sample the map-generator state immediately before and
  after engineer entry, and record each resulting overlay position. Repeat three times,
  including bridges along both axes. Compute successive raw values from the pre-entry
  state and apply claim 13's formula, counting rejected values too. *Expected:* exactly
  those choices and the resulting state advancement, assigned in increasing axis-coordinate
  order, with no value for the intact end piece. *Falsified by:* a mismatching position,
  strip order or stream advancement. Void a run if unrelated actors affect the bridge.
  This does not promise to encounter the rare rejection endpoint.
  (g) *Ordinary-map initialization (Q-0711).* Use the same low-bridge stock-data
  fixture, speed 4, no engine add-on and no mods (the named test files of a scenario excepted), with the human engineer and distant computer
  player. Make three fresh launches, never opening random-map generation. Record each
  game's naturally selected game seed and the map-generator state at the first tick,
  before repair, and after repair. Also record repaired piece positions. Expected:
  every first-tick and pre-repair map-generator state is exactly claim 3's seed-0 state,
  regardless of game seed. The repairs fit claim 13's formula beginning at seed-0 x₀,
  not at a stream seeded with the first unsynchronised ranged result. Falsified by any
  different initial state or unexplained advancement. After repair, return to the menus
  and start the same ordinary map again without exiting. Expected: its first-tick state
  equals the preceding game's post-repair state, with no reset and no fallback draw.
  A changed state falsifies persistence. Void a run with an intervening use of this stream.
  (h) *No in-play reseeding (Q-0711).* In one ordinary-map game, perform two successive
  low-bridge repairs, using two separate damaged bridges if needed. Record the state
  before and after each repair and the piece positions, including all accepted and
  rejected values. Expected: the second repair continues from the first repair's outgoing
  state, not a fresh seed-0 or game-seeded state. Repeat three fresh-launch runs; a reset,
  extra advancement between repairs or choice inconsistent with continuation falsifies it.
  The read-only random-state recorder and scripted input are proposed tools for g/h,
  not completed measurements.

- **R10, game-start animation draws (claim 10, loading-draw table).** Reuse S-0010 G4 part C's
  named theater fixtures: run 1 starts an animation only in the game-start pass, and run 2 starts
  one in the second pass and must not start another in the game-start pass. Give the test types
  distinguishable non-singleton random-rate ranges, no bouncing motion and no start-event
  effects. Skirmish at speed 4, no engine add-on and no mods (the named test files of a scenario excepted), 5 runs per
  fixture. Use the read-only random-state recorder to record each creation's incoming and
  outgoing scenario-stream states, initial stage delay and cell, plus the states at starting-unit
  completion (including its final R(0, 65535) when present), initial crate placement,
  secret-lab choices, end-of-loading tag setup and the first tick. Use two isolated instances of the
  fixture, in known visiting order, to test their draw order. *Expected:* run 1's creations
  occur after starting units, crates, secret-lab choices and end-of-loading tags and before the
  first tick, in visiting
  order, and each advances the stream by precisely its stage-delay ranged draw; run 2 takes no
  further animation-creation draw in the game-start pass for a cell already served in the second
  pass. *Falsified if* a creation is deferred to the first tick, precedes end-of-loading tags,
  occurs in reversed visiting order, consumes another count, or run 2 creates another animation.
  For R8 F variant (iii), turn on only the chain-reaction effect and keep every displaced cell
  free of resource: the predicted count must remain unchanged. Resource removal or any extra
  raw value falsifies that variant. Record creations removed later separately; the first-tick
  animation count alone cannot establish the load's draw count.
  Variant A adds one neutral secret lab meeting S-0407 claim 14's draw conditions and turns
  crates on. Record the lab's choice and the crate placements as well as every animation
  creation. Expected: both groups finish before the game-start fixture's creation, whose
  incoming state predicts its rate draw. A fixture creation before either group falsifies
  claim 14. Variant B repeats with bases off and unit count 0, so no starting units, then
  with S-0009's early-stop fixture. Expected: the pass still follows crates, secret labs and
  tags, with no synthetic final starting-unit draw in the early-stop case. A pass before
  those groups or an unexplained extra final draw falsifies the variant. Five runs each.

- **R11, cleanup (claim 14; Q-0101 Q2/Q3), deferred.** The former proposal used
  S-0009 M16 variant A, which is a network game, not a skirmish fixture. It supplies no
  skirmish backing. Cleanup remains unmeasured pending a supported skirmish setup with
  no computer player that reaches cleanup and permits the conditional-animation control.

- **R12, low-bridge writes do not start cell animations (claim 14; Q-0101 Q4).** Use R8 A's
  bridge geometry and recorder, speed 4, no engine add-on and no mods (the named test files of a scenario excepted).
  Give the end-marker types, their end-row types and all four piece types a cell animation
  with a non-singleton random-rate pair, no bouncing and no start-event effects. The terrain
  names no tile animation. Add one ordinary overlay of a separate type naming the same
  animation as a positive control, outside the bridge and after its later marker in stream
  order. Five runs. Record all animation creations during loading, not just survivors at
  the first tick, and bridge piece types. Expected: the bridge still consumes exactly 3*d*
  consecutive raw values in R8 A's order, creates no cell animation at any marker, end-row
  or piece site, and the control creates one animation with its predicted rate draw.
  Falsified by any bridge cell-animation creation or a rate-draw gap in the bridge sequence.
  Repeat with R8 H's tile-animation fixture: the piece's raw value must precede that tile's
  creation draws, without a cell-animation rate draw between them. This repeat separates
  the exclusion from suppression of all animations.

- **R13, resource-glint setup (claim 14).** Small purpose-made skirmish map, one human
  and one bystander computer player isolated from the observed resource area; speed 4,
  no engine add-on and no mods (the named test files of a scenario excepted). Put at least eight isolated resource cells
  with nonzero contained value in known visiting order, plus empty-cell and zero-value
  resource controls. Configure a glint type with a distinguishable non-singleton random-rate
  range, no bouncing motion and no start-event effects. Use frequency 2, then 1, then 0, then repeat
  with no configured glint type; five runs each. Keep R10's game-start-only tile animation
  as an ordering marker. A proposed read-only recorder logs the end of recalculation,
  each glint choice's incoming/outgoing stream states, cell and result, each creation's
  initial stage delay and outgoing state, and tick 0. Use each game's recorded seed, never
  force one. Expected: the glint choices occur after the marker's creation and before
  tick 0, in visiting order. At frequency 2 each eligible cell consumes one raw value for
  R(0, 1); a zero result inserts exactly the predicted rate-draw consumption before the
  next cell. Frequency 1 consumes no choice value but creates a glint on every eligible
  cell, each with its predicted rate draw. Frequency 0 tests reversed bounds:
  R(0, −1) consumes one raw value and returns −1 plus its lowest bit, so only a lowest
  bit of 1 creates a glint. Empty and zero-value controls take no choice
  or creation draw. No configured type means no glint draws. Falsified by a reversed
  order, draws before recalculation finishes, a creation on a nonzero choice, any control
  draw, a consumed choice value at frequency 1, a frequency-0 choice or consumption differing
  from that prediction, or animation draws not fitting claim 10.
  Void a run if the bystander acts in the observed area or is defeated before tick 0 is
  recorded. Checkpoints, not just the first-tick animation count, distinguish animations
  removed later and unrelated setup draws.

- **R14, middle-event composition at map load and during play (claim 15; Q-0182, Q-0217).** Small purpose-made skirmish
  map, speed 4, one human and one bystander computer player on an isolated remote start island.
  Keep the observed area away from starting units. No engine add-on and no mods (the named test files of a scenario excepted); these are map overrides, a test art file, one-frame test sprites (μ = 0), and a test smudge list.
  Use the random-state recorder to compare incoming and outgoing load-time states with the
  predicted sequence from each recorded seed, and a read-only cell/particle recorder for the
  resulting types, starting velocities, lifetime variations, colours and resource levels.
  Five fresh launches per variant; the recorder
  does not set the seed. Account separately for stage delay and bouncing draws, or disable
  them in the named animation test data. No chain reaction in these test animations.
  - **A, particles.** Repeat both a tile animation and an overlay cell animation with counts
    −1, 0, 1 and 2, plus a positive-count variant with no particle type. Use each of the five
    particle behaviours and an unrecognised behaviour.
    Set the lifetime field to test values 1 and 7. For each, try an empty colour list, a
    nonempty list with two black starting colours, and a nonempty list with one nonblack
    component. Include a normalised-timing variant. Repeat selected variants on raised
    ground, on an occupied cell, on an overlay cell, and in a second theater using the same
    named sprite. Expected: absent particle type or nonpositive count takes no particle draws.
    Each smoke particle takes two raw values before any colour draw, all other
    behaviours one; the optional colour draw is exactly R(0, 2147483646), with rejections
    counted. Count 2 gives two complete groups, not grouped speed draws followed by grouped
    lifetime draws. The smoke speed variation equals the signed first-value remainder modulo 2.
    Lifetime variation equals the magnitude of the next signed-value remainder, modulo 10 for
    railgun and modulo the test lifetime field otherwise. A different recorded variation
    falsifies its form even if the consumption matches. Terrain, overlays, theater and timing
    variants add no scenario draws. Enable both
    smudge flags in one variant and predict the kind choice from the state after those groups.
    Any extra/missing raw value, different ordering, or choice from the pre-particle state
    falsifies the rule. The field-1 case still consumes its raw lifetime value despite a
    constant remainder. Do not run the original's zero-divisor crash case.
    Creators other than an animation's middle event (claim 15, the paragraph on every creator
    of a particle): in the same map, with the same recorder, one run each for a smoke emitter
    with a held type, a spark burst, a railgun line, a smoke successor and a gas-behaviour
    successor. Give each creator's particle type lifetime field 7 and one nonempty colour list
    with one nonblack component. Observable: the raw values each creation takes, by the taking
    object, and each particle's lifetime and colour. Expected: each particle's draws are those
    of the table in table order (smoke speed, then lifetime, then colour), from the state left
    by the creator's own earlier draws. Falsified by any extra, missing or reordered raw value,
    or by a lifetime or colour result that the table does not give.
  - **B, smudges.** The test list gives each kind two 1 × 1 types and two 2 × 2 types, in
    known registration order. Place on flat morphable ground with no overlay or smudge at
    the animation position. Use test middle-frame visible bounds 60 × 51, 61 × 50 and
    61 × 51 pixels; change the full canvas without changing those bounds in one variant.
    Try scorch only, crater only, both, and large-crater enabled. Add a missing-sprite variant:
    expected visible bounds are 30 × 30 pixels and non-large selection prefers fitting 1 × 1
    types. Add a list with no fitting 1 × 1 type to test fallback to all fitting types for a
    small non-large request, and a list with no fitting wide-and-tall type to test the large
    form's fallback. A different candidate list or selected type falsifies either fallback.
    Add variants with a building,
    a slope, an existing smudge and a nonmorphable tile in a candidate footprint, plus
    candidate lists with only one fitting type and none. Expected: non-large requests prefer
    the two 1 × 1 types in the first two size cases and use all four in the third; large
    craters prefer the two 2 × 2 types regardless of sprite dimensions and ignore buildings.
    Each selection matches R(0, n − 1) from the recorded state, with no placement draws.
    Zero/one candidate consumes none. Wrong candidate eligibility, dependence on canvas
    dimensions, or a different draw count falsifies the claim.
  - **C, resource timing.** Use crater-only animations and resource cells at their actual
    positions. One fixture starts the animation during overlay placement, another first in
    the whole-map recalculation. Give resource data a test value 9 in the map data stream,
    and record resource type and levels at each load milestone, not merely after tick 0.
    Expected: a removal during placement acts on the earlier level, then data becomes 9
    if the data stream supplies it, without restoring a removed resource type. First-pass
    removal instead acts on stage 9, leaving stage 3. Add isolated single-event fixtures
    with incoming resource stages 0, 5, 6 and 7: stages 0 and 5 empty, while stage 6 keeps
    the resource overlay at stage 0 and stage 7 at stage 1. A stage-6 fixture that empties,
    or takes emptying-only recalculation or neighbour-offer draws, falsifies the rule.
    Separate second-pass repeats from that first event. A named test-data variant makes resource exhaustion expose a missing
    tile animation with a known random-rate range: its applicable creation draws must fall
    before smudge selection, and a cell with an already-recorded tile animation adds none.
    Add a qualifying resource neighbour under S-0401 claim 8 with its spread entry not yet
    marked, and record the spread-list state at exhaustion. Expected: its offer takes that
    claim's raw value after any recalculation-created animation draws and before the smudge
    draw. Repeat with the neighbour already marked: no offer draw. Omission of a qualifying
    offer, a draw for the marked control, or a different stream order falsifies this variant.
    An overlay type restored by data alone, reduction on the wrong side of the data-stream
    milestone, or a smudge choice made before the conditional tile-animation draws falsifies
    the claim. Void a fixture in which its intended creation checkpoint is not reached.
  - **D, during-play composition.** Use the same animation and particle data as A and B,
    now created by a named test weapon's detonation animation at a recorded tick. Its
    animation has no loop delay and μ = 0. Repeat the no-particle, smoke and railgun cases,
    both colour-draw gates, counts 1 and 2, both smudge flags, size thresholds and candidate
    counts 0, 1 and several. In a separate game create the same named type through the
    play-animation trigger action; its creation event must not omit the middle-event draws.
    Record the incoming and outgoing stream state at the animation creation, particle creation
    outcomes and cell state. Predict/account for weapon, projectile, trigger, bystander and
    other unrelated draws under their owning specs; void a comparison with unaccounted draws.
    Expected: the event consumes A/B's groups in their stated order from its own incoming
    state, including rejected ranged values, not merely the stage-delay and bouncing draws.
    Particle outcomes and the smudge choice match the prediction. Repeat C's exhausted and
    nonexhausted resource fixtures during play, including missing/already-started tile animation
    and marked/unmarked neighbour controls. Expected: any conditional recalculation-animation
    and neighbour-offer draws still precede smudge selection; no data stream subsequently
    overwrites the during-play levels. Add a chain-reaction-enabled creation with resource
    remaining, accounting for S-0401 claim 20's draws after the complete middle event, and
    one whose crater exhausts the cell: the latter has no subsequent reaction draws.
    Finally use a multi-frame sprite with μ > 0 and record the middle event at its first
    eligible playback advance instead of creation; predict the same event composition there.
    Five fresh launches per variant, same setup and recorders as A/C. Different group order,
    an omitted group during play, a phase-dependent extra draw, a wrong resource result, or
    a chain-reaction draw before middle-event completion falsifies the claim.
  - **E, shared terrain-height gate.** At creation during loading and during play, use an
    animation requesting particles and both smudge kinds, with μ = 0 and no loop delay.
    Use flat raised ground and a slope, two in-cell positions whose S-0106 surface heights
    differ, and a position above a bridge deck. Record actual (X, Y, Z) and H(X, Y), not just
    the source object's cell. Named fixtures give Z − H values −1, 29 and 30 leptons.
    Five fresh launches per fixture, recorders and unrelated-draw accounting as D.
    Expected: particle groups occur at every height; −1 and 29 permit the kind-choice draw
    and any eligible smudge selection, while 30 permits neither. At deck height the test
    still uses terrain H, not deck height. Resource removal happens only on a permitted
    crater request. A result based on cell-centre, creator-cell or deck height, or a height
    gate that differs between loading and play, falsifies the claim. Smudge placement may
    fail its fit test on slopes or overlays; that does not erase the kind-choice draw.

- **Companion assignment and sound tests (claims 10 and 11).** S-0404 P11 B/C are
  the proposed tests for production-start value assignment, not merely isolated draw counts:
  compare reversed line-creation order at one cancellation boundary, then a turn-pass
  replacement with an order-boundary start. S-0701 E1/F1 test sound-instance draw groups
  and the feedback chance from recorded unsynchronised states; F6 tests lightning's
  scenario-stream choice with audible/silent and empty-list controls. For repeated points,
  use S-0513 SW7's storm fixture with its named test data, no engine add-on and no mods (the named test files of a scenario excepted), speed 4,
  800 × 600 pixels and the read-only random-state recorder. Make two trigger clouds over
  the same flat cell, with no other clouds and no terrain-height change between strikes.
  Repeat for 20 strike pairs at exactly the same point, accounting separately for bolt,
  explosion, damage and
  debris draws under S-0513 claim 18. Expected: each strike with a nonempty sound list
  takes its own raw sound-choice value after bolt-animation creation; the second strike
  retains its later effects and draws. Suppression on the second strike falsifies the rule.
  S-0522 P2/P3 test paratrooper creation at launch, after plane placement, separately from
  the later drop-attempt and parachute-animation draws of S-0522 claim 16.
  The companion scenarios own their other setups, observations and falsifiers. No results
  from them are claimed here, and none measures the between-house design extension.

- **R15, post-game winner-score requests (claim 16).** Use S-0414 V18's purpose-made
  skirmish fixture with stock data except its named map overrides, no engine add-on and no mods (the named test files of a scenario excepted),
  speed 4 and 800 × 600 pixels. Run ten naturally recorded seeds with scripted input,
  screen capture and read-only running-score, defeat-state and random-state recorders.
  Include multiple undefeated listed houses with distinct positive scores, a zero-score
  undefeated control, and defeated and omitted-house controls. Record the scenario stream
  at the end of play and during statistics preparation, with the final tick number.
  Expected: each eligible house receives R(⌊max(p, 0) / 2⌋, max(p, 0)) from the continuing
  stream in house order, with all rejections accounted for. The zero-score control and
  excluded houses consume no raw values. Displayed scores match S-0414 claim 23, and no
  new simulation tick occurs during preparation. A reseed, other stream, wrong assignment
  order, extra or missing value, or simulation tick caused by the requests falsifies the rule.
  S-0414 V12 c supplies the additional final-tick-defeat control: a committed win does not
  override the defeated house's exclusion from the bonus. Void runs with unaccounted
  scenario draws between the sampled end-of-play state and the score requests.

- **R16, colour-list reading (claim 17; Q-1562).** A small purpose-made skirmish map with stock
  data except named test rules, no engine add-on and no mods, speed 4 and 800 × 600 pixels, and
  scripted input. Each run takes the seed the game chose (the recorder fixes none), so no part
  relies on a test run and its control sharing a seed: part (a) uses an observable that does
  not depend on the draws, and part (b) compares draw counts.
  - **Part (a), entries.** The test rules give a spark-behaviour particle type both start colours
    black, so a particle's first colour is entry 0 of its list without a colour draw (S-0616
    claim 29 and F16), a colour speed of 2.0, and a lifetime long enough that it reaches the last
    entry of the longest reading and is still shown there for several frames; a particle system
    at a fixed point emits it. By F16 of S-0616 the random part of each update's colour progress
    is smaller than 0.05 in size, so with that speed every update passes 1.0: each update moves
    the particle on by exactly one entry and draws that entry unmixed, and at the last pair it
    holds the last entry. The particle therefore shows its entries in list order whatever the
    draws, and only their frame timing depends on the seed. Every test text and every control
    begins with the same two clean, distinct triples, `(250,0,0),(0,250,0),`. A complete three
    shifts nothing after it, so each reading has these two entries first. Every test text and
    every control also ends with the same kind of suffix, so that every entry under test is a
    middle entry, shown for exactly one update: first *p* filler items `,9)`, where *p* is the
    number of items (0, 1 or 2) that claim 17's item rule needs to bring the prefix and the test
    text to a whole number of threes, and then the clean sentinel triple `,(0,0,250)`. Choose
    the scaled test values so that no predicted entry is (0, 0, 250); where one would be, use
    another clean sentinel triple that differs from every predicted entry. The filler items join the leftover items of the test
    text in one predicted entry by claim 17's rules: for `(200,100),(50,150,250)`, *p* = 1, and
    the leftover `150` and `250)` with the filler give (50, 250, 9). Run the type once with each
    test text of claim 17's table placed between the prefix and the suffix, with component
    values scaled so that the nonzero components are clearly visible (for example 200, 100 and
    50 in place of 10, 20 and 30), and once with its control: the prefix, the predicted entries
    written cleanly, and the sentinel (for `(200,100),(50,150,250)` the control is
    `(250,0,0),(0,250,0),(200,100,0),(50,250,9),(0,0,250)`). A shape with no entry of its own,
    such as `(200,100)`, still gives the entry that its leftover items form with the filler,
    (200, 100, 9) here; the empty-list case itself is part (b)'s.
    Capture every frame of the particle's life. Each middle entry is shown for one update only,
    so the recorder also logs the tick of every drawn and captured frame: a run in which any tick
    of the particle's life has no drawn and captured frame is inconclusive and is repeated (as in
    S-0616's frame-rate check). Observable: the colour the particle shows after each update, one
    value per update, from its creation up to the first update that shows the sentinel. Equal
    neighbouring entries, such as two (0, 0, 0) entries, are not merged: since each middle entry
    is shown for exactly one update, the number of consecutive updates on one colour is the
    number of entries of that colour. Expected: each test run shows the same per-update
    sequence as its control, and reaches the sentinel after the same number of updates. A test
    run whose sequence or count differs from its control's falsifies the rule, for example
    `(200,100),(50,150,250)` showing two entries between the prefix and the filled-out entry, or a
    reading that adds or drops a (0, 0, 0) entry, which reaches the sentinel one update late or
    early.
    Two more test texts, each placed after the prefix, separate the number reading from
    readings a reimplementation might choose instead: `(1Fh,$20,-1)` gives the entry (1, 0, 255),
    against (31, 32, 255) for a reader that applies S-0003's hexadecimal forms; and
    `(300,-56,4294967340)` gives (44, 200, 44), against (44, 200, 255) for a reader that limits a
    number to the 32-bit range instead of wrapping it, and (255, 0, 44) for one that limits each
    component to 0 … 255 instead of reducing it modulo 256. Both are complete threes, so *p* = 0,
    and their controls are the prefix, `(1,0,255)` or `(44,200,44)`, and the sentinel.
  - **Part (b), the colour draw.** Claim 15's colour-draw gate does not depend on the behaviour
    kind, so this part uses a smoke-behaviour particle type, which is drawn from its image and not
    as pixels, with one start colour nonblack and no prefix. It uses R14 part A's setup and
    recorder: a one-frame test animation whose middle event at map load creates one such
    particle, with the scenario stream's state compared before and after the particle's creation
    group. Run it with `(200,100)` alone, with an empty list, and with `(200,100,50)`. Expected:
    the creation group is two draws (the smoke speed and the lifetime variation) in the first two
    runs and three in the third. A third draw in either of the first two runs falsifies the
    rule; no third draw in the third run means the setup does not reach the colour gate, and the part
    must be repeated another way. The smoke update draws of S-0616 claim 6 come later and are not
    part of the compared group.

- **R17, house-creation draws (claim 14; Q-2336).** A small purpose-made skirmish map with
  stock data, speed 4, no engine add-on and no mods (the named test files of a scenario
  excepted): no water, no resource, no overlay, no trigger, no tag and no pre-placed object,
  and four start waypoint markers, so no start list has a random entry. The skirmish runs under
  the oracle harness, whose settings file gives the skirmish setup (as for M-0001's games: no
  menus), with the harness's arena set-up and its gameplay changes off. The bases setting has no
  lobby control and no settings-file route: it is the base rules' bases default, copied when the
  program starts (S-0412 claim 1). So a loose test copy of the base rules, placed before the
  program starts as in S-0412 scenarios K1 run 2 and K3, sets the bases default off and the
  minimum unit count 0. The settings file sets the lobby's values: the unit-count setting 0,
  crates off, and fixed colours and fixed start positions for every player. Runs with two, three
  and four players (one human, the rest computer), five naturally recorded seeds each, with a
  read-only random-state recorder that logs every scenario-stream request, its bounds and its
  result, from the seeding to tick 0. Variant B adds one pre-placed vehicle of the neutral house,
  of a type with neither visceroid flag (Import mapping), no initial slot objects (S-0506
  claim 1) and no initial slave group (S-0402 claim 19), so that its creation makes no other
  object (S-0216 claim 3). Variant C leaves every player's start
  position on random, so the human player, first in S-0009 claim
  19's house order with no position yet taken, takes that claim's one ranged draw over
  0 … 3, and every later player gets the farthest-sum position with no draw. Variant D sets the
  map's fill setting to water (S-0009 claim 6), so the initial fill takes one draw over 0 … 3
  per visited cell.
  - *Expected:* with *p* players, 2 × (*p* + 2) ranged draws R(450, 1800) first (the *p*
    players and the neutral and special houses, twice, each round in S-0009 claim 19's order),
    then the starting-unit step's final R(0, 65535) and no other draw before tick 0.
    In variant B the vehicle's creation draw comes after all the house draws, followed by any
    placement draws that the placement specs give the vehicle (map-loading table, *Pre-placed
    objects*), and those are the only other draws. In variant C the one draw over 0 … 3 comes
    after the first *p* + 2 house draws and before the other *p* + 2,
    and it is the only other draw. In variant D the fill's draws over 0 … 3 come directly after
    the second round's last house draw (the later draws of a water map are not compared).
  - *Falsified if:* another count, house draws with other bounds, house draws after the
    vehicle's creation draw, in variant C the draw over 0 … 3 in another place, in variant D a
    fill draw before a second-round house draw, or, in the runs without water, any other draw
    before tick 0 (variant B's placement draws for its vehicle excepted).

## Changes for other specs

- S-0009 claims 19 and 20 (Q-2336): each house created takes claim 14's house-creation draw,
  and a skirmish game creates claim 19's house list twice. So claim 20's draw order starts with
  the first round of these draws, before list 1's random entries, and has the second round
  after the random-position draws and before the initial fill's draws. Claim 20's *Draw order*
  list can name both rounds and point here.
- S-0009 scenario M9c (Q-2336): with *h* houses created per round (the *p* players and the
  neutral and special houses, *h* = *p* + 2; *h* = 4 in M9c's two-player game), the raw values
  before the fill's include both rounds' 2 × *h* house-creation draws (claim 14), whose raw
  values follow from the recorded seed by this spec's range rule (claim 4). Variant A's *Expected*
  becomes: *k* is
  1 plus the raw values that the 2 × *h* house draws take, all computed from the recorded seed,
  and the human starts at waypoint 0 exactly when the raw value directly after the first round's
  raw values gives 0 by the range rule. Its parenthesis on a *k* larger than 1 becomes: a *k* that
  exceeds this value by the same amount in every run falsifies only the absence of other draws,
  and variant B then adds that amount. Variant B's *Expected* becomes: *k* is the raw values of
  the first round, plus those of the 4 random entries, plus 1, plus those of the second round,
  all computed from the recorded seed, and the human starts at the position that the raw value
  directly before the second round's raw values gives. "S-0202's table "Draws during map loading"
  lists nothing before the early phase" becomes: that table lists only the first house-creation
  round before the early phase. The rest of M9c, list 3 and the fill's draws included, is
  unchanged.

- S-0215 F6 (targeting timer): in M-0008 one infantry unit of one type, placed during play at tick
  300, and one armed tank type created before tick 0, both left in *guard*, show a second periodic
  *U*(0, 2) draw. The tank's run is not stock: a test rules file makes the Americans' buildings
  durable and gives the tank test copies of its weapons, the tank fires no shot, and its
  surroundings are the Americans' two barracks, power plant and air pad and one Russian far power
  plant. M-0008 does not show that the weapon copies leave the targeting timer unchanged. M-0008
  tells the second draw apart from their *guard* processings' draws by ticks and values only: the
  infantry unit's first draw comes a fixed first run-out after the tick of its creation and the
  tank's the same run-out after tick 0 (its length local to M-0008), and each later one comes the
  normal targeting delay plus the previous draw after the one before. The draws and that spacing are
  found together, so the spacing is a fit, not a separate check. It fits F6's timer restarted by the
  per-tick search of S-0309 claim 1, first row. For those two cases this hand-off asks S-0215 to
  state that an object's targeting timer first runs out a fixed delay after its creation, or after
  tick 0 for an object created before it, which S-0215 should give as a constant of its own, low
  confidence; M-0008 keeps the observed length local. Backing: `measured` (M-0008) for the draws of
  those two units. That they are the timer's restarts, and their spacing, are a fit, since M-0008's
  recorder does not record the timer itself. Confidence low for those two cases. For other units and
  other behaviours this hand-off establishes no first-restart timing. S-0215 B10 part (a), an armed
  vehicle in *guard*, has in M-0008 only the tank's first draw at the first run-out and its later
  draws at family T's spacing (M-0008's name for the infantry unit's chain of these draws), in a
  control run with no attack order and with the test rules file above (durable buildings, test
  copies of the tank's weapons), picked out of that run's draws by the same fit. The same fit also
  picks out a second chain of the tank's draws, from tick 0, at family G's spacing (M-0008's chain
  of the infantry unit's *guard* processing draws, next bullet). That this chain is the tank's
  *guard* processing is a fit, low confidence, since the family fit of the next bullet is rated
  medium for one infantry type only.
- S-0215 claims 6 and 24, F1 and F2 (*guard* processing): in M-0008 one family of an infantry
  unit's *U*(0, 2) draws is taken as its *guard* processings' delay draws. M-0008 tells that
  family apart by ticks and values only: its first draw is the third of the placement tick's
  draws, and each idle attempt falls on one of its ticks. In every interval of its clean window
  the family fits the spacing of the guard rate plus the previous
  processing's delay draw. The family and its spacing are found together, so the spacing is a
  fit, not a separate check. The first *guard* processing of an infantry unit placed during
  play falls on its placement tick in the same fit. Backing: `measured` (M-0008) for the
  family's draws, ticks and values; the family's identification, its spacing and the first
  processing's tick are a fit. Confidence medium for one infantry type in *guard*.
- S-0216 claim 3: the order at an infantry unit's placement during play, the creation draw, then
  the standing-position draw *R*(0, 3), then the first *guard* processing's delay draw, has
  consistent evidence in M-0008 (part b of scenario R5: three draws that fit these in order and
  by range). A fit does not raise the claim: S-0216 claim 3 stays at medium until its scenario
  PT3 runs, with M-0008 as consistent evidence.
  That the third draw is the first *guard* processing's delay draw rests on the family fit of
  the hand-off above, at medium.

## Open questions

- A discriminating skirmish fixture for cleanup's phase order and conditional tile-animation
  draws is not established (claim 14; deferred R11).
- Q-0046 is answered across specs: claim 10 covers immediate creation timing,
  cell-animation ordering and the no-resource chain-reaction case. Q1 belongs to S-0009,
  Q2a's resource-bearing sequence to S-0401 claim 20/E17, and Q2c's sprite-derived μ to
  S-0109 claim 25. For Q2b, S-0109 claims 14 and 25 cover map-load event gates and the
  particle-before-choice ordering. Claim 15 completes Q-0182's particle creation draws,
  smudge forms and map-data timing. These effects must not be treated as taking zero draws
  during map loading. Claim 15 and R14 D/E also answer all of Q-0217: creation-time
  middle events during play use the same scenario-stream composition and terrain-height
  test. Subsequent particle updates remain outside this spec, not draw-free. S-0616 claims 6, 8, 9, 11, 12 and 15 to 18 own the non-gas update, successor, emission and burst draws.
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
- Whether, in the original, the machines of a network game build the same terrain-image pattern in
  the first game of a session (that needs the same unsynchronised draws before the build on every
  machine). Low confidence either way; it does not affect our engine (claim 11).
