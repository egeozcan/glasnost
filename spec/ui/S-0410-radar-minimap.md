---
id: S-0410
title: Radar and minimap (when radar is available, what the radar map shows and how it is coloured)
topic: ui
revision: 5
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-13
measurements: []
requests: []
supersedes: []
---

# S-0410: Radar and minimap (when radar is available, what the radar map shows and how it is coloured)

## Summary

When a house has radar, and what the radar map in the sidebar shows while it does: which cells and
objects appear, in which colours, how cells map to the panel, and which object wins when several
share a point. Radar reaches no game state: it only reads what other specs own (explored state,
positions, colours). Skirmish only.

## Terms

Shared terms used here are defined in the glossary: *house*, *local player*, *cell*,
*legal cell*, *usable cell*, *local area*, *building*, *type*, *field*, *tick*, *drawn frame*,
*sidebar*, *house colour*, *disguise*, *veterancy ability*, *quirk switch*, *measurement setup*,
*recorder*, *scripted input*, *override section*, *being erased*, *alliance*, *turn* (second sense), *low power*, *importer warning*, *departure*,
*bystander computer player*, *type kind*, *infantry*, *vehicle*, *aircraft*, *sprite*, *image frame*, *selling* (the behaviour, glossary *behaviour* and *sale*). *Explored*, *gap field*, *spy
satellite*, *discovered by H* and *player house* are S-0409's (Terms). The point test for shroud
is S-0110's (claim 8), with its *candidate cells* (the one or two cells that decide whether a point is
shrouded) and its number of whole height levels of the point, written *z* here (S-0110 calls it *u*,
and in this spec *u* is a radar point column, claim 4, and *h* is an image height). A *gap field* is *hostile* to a
house as S-0409 defines (Terms); a gap field that is not hostile to *H* is a *friendly* gap field of *H*
(it is *H*'s own or its ally's, or any field once *H* has been defeated, S-0409 claim 11). In addition:

- **Radar object.** An object of one of the four type kinds infantry, vehicle, aircraft and building.
  Only radar objects are ever listed on the radar map (claim 7). Terrain objects, overlays, smudges,
  animations and projectiles never are, whatever their flags or owner. Wherever claims 6 to 10 say
  "object", they mean a radar object.
- **Frame radar colour.** The 8-bit triple (channel values 0 to 255) that a sprite stores with each of its
  image frames as that image frame's colour on the radar map. It is stored data, not computed from the
  image frame's picture and not looked up in a palette. An image frame that the sprite does not have, or a type
  with no loaded sprite, has the frame radar colour (0, 0, 0). No spec of the project owns this stored field yet; it belongs to the sprite file's format.
- **Radar building.** A building whose type has the radar flag (Import mapping).
- **Radar outage.** A per-house countdown in ticks during which the house has no radar, started by
  the weather storm superweapon against every house not allied with its owner, for the storm's
  duration, and by a radar-outage trigger action (superweapon and trigger specs).
- **Free radar.** A map flag that gives every house radar (Import mapping).
- **Electromagnetic lock.** An object under the electromagnetic pulse effect of S-0203 (Terms) has
  a running lock countdown. The combat and superweapon specs own how it starts and ends. It is a
  different state from *being erased* (glossary), which claim 1 also tests.
- **Switched on.** The state of a building's power switch, as S-0405 defines it (Terms). It is not the in-operation
  test of S-0405 (claim 8): a building in build-up that is switched on counts.
- **Stored position.** For an object that is not a building, its position (S-0204); for a building,
  the centre of the top-left cell of its foundation (S-0409, Terms), not the centre of the whole
  foundation. Claim 8 and F4 place objects by it.
- **Listed, arrival.** An object is *listed* at a radar pixel (F4) when it passes the tests of its
  case in claim 7 (the common tests and the owned or not-owned tests; not the draw-time tests) at the
  start of its turn (S-0203, glossary *turn*, second sense). Its *arrival* at that pixel is the moment
  of the turn at the start of which it became listed there: it was not listed at the end of its
  previous turn, or was listed at another pixel then. Arrivals are totally ordered: by tick, and
  within one tick by the order in which the objects' turns run (S-0203). An object that stays listed
  at one pixel keeps its arrival; one that stops being listed or moves to another pixel loses it, and
  gets a new arrival when it is listed there. An object that is listed but fails a draw-time test of
  claim 7 keeps its arrival. A building is listed at every pixel of its set (claim 8), with its own
  arrival at each of them. Claim 8 orders the objects at one pixel by arrival.
- **Shown.** An object is *shown* at a radar pixel when it is listed there and passes the draw-time
  tests of claim 7 at that pixel.
- **Tile radar colour.** The 8-bit triple (channel values 0 to 255) that S-0010 (claim 20, Terms
  there) gives a tile cell as its colour on the radar map. It is not a *radar colour* in the sense
  below, which is the colour of a map pixel at display precision.
- **Radar colour.** The colour of a radar map pixel, held at the channel precision of the display
  surface after S-0103's 16-bit step (claim 9 there): 5 bits red, 6 bits green and 5 bits blue when
  the display surface has 5, 6 and 5 bits per channel, and 5 bits each when it has 5 bits in every
  channel (the bits per channel are the display surface's, not a setting of this spec). A *channel value* of a radar colour is one such channel, 0 to 2^*k* − 1 for *k* bits.
  Every colour constant and every house or object colour in this spec is given as an *8-bit
  triple*: channel values 0 to 255. An 8-bit triple becomes a radar colour by dropping the low bits
  of each channel: *c*₈ becomes ⌊*c*₈ / 2^(8 − *k*)⌋. (So (170, 170, 130) is the radar colour with
  red ⌊170 / 8⌋ = 21 at 5 bits.)
- **Radar map.** The picture in the sidebar's radar panel, 140 × 108 pixels. Its pixels outside the
  radar image (claim 5) are not drawn by the radar: they keep what the panel shows there (the
  sidebar spec owns it).
- **Radar point.** One position of the radar map's own grid, before scaling (claim 4). A cell covers two
  radar points side by side.
- **Radar image, radar pixel.** The radar point grid scaled into the radar map (claim 5); a *radar
  pixel* (*p*, *q*) is a pixel of that image, counted from its top-left pixel.
- **Terrain picture, point colour.** The *point colour* of a radar point is the 8-bit triple that
  claim 9 gives it; a radar point that no usable cell covers has the point colour (0, 0, 0)
  (low confidence: nothing paints it); the *terrain picture* is the radar image made from the point colours by the
  area-weighted mean of Formulas F6, converted to radar colours as *radar colour* says.
- **Theater brightness.** A per-theater number *f* that scales the tile colours of claim 9. An
  engine constant (Constants).
- **Cell shown at a pixel.** The one cell that a radar pixel stands for in the shroud, gap and
  object tests (claim 6, Formulas F3).
- **Radar-invisible, insignificant, invisible, radar-visible.** Type flags (Import mapping), read
  from the types of the four radar-object kinds.
- **Radar recheck.** The re-evaluation of a house's radar against claim 1 (claim 2).
- **Flash frame time, combat flash time.** Two game-wide rules values in ticks (Import mapping).
- **Radar flash countdown.** A per-object countdown in ticks, set to the combat flash time when the
  object is hurt (combat specs). Claim 10 says what it changes on the radar map.

## Behaviour

1. **Radar availability.** House *H* has radar when no radar outage of *H* is running and either
   the map's free-radar flag is set or all of these hold:
   - *H* is not on low power: its power output is at least its drain, or its drain is 0 (S-0405).
     Low power is tested once for the whole house, not per radar building,
   - *H* has at least one qualifying building, and the first of them is not under an
     electromagnetic lock and is not being erased. If the first qualifying building is locked or
     being erased, *H* has no radar even when a later radar building is fine (Edge cases,
     `quirks.radar_first_building_decides`). A building qualifies when it is a radar building
     that is switched on, on the map, alive, and not being sold (neither its current nor its queued
     behaviour is *selling*); "first" is the earliest in the order in which *H* came to own its
     buildings. A qualifying building
     need not be in operation (S-0405, claim 8): the switch is all that is tested, so a radar
     building still in build-up counts (S-0405, claim 12).

   The free-radar flag is fixed for the whole game: nothing in skirmish is known to change it (trigger
   actions were not checked, Open questions).

   The conditions above decide *H*'s radar only until *H* is defeated (S-0414, claim 5). In the tick
   of the defeat the map is revealed for *H* (S-0409, claim 15) and *H*'s radar comes on, without
   waiting for a recheck (claim 2). From then on it stays on for the rest of the game, whatever *H*'s
   power, outage, free-radar flag or buildings are. In the original the human house's defeat ends
   the game even while a computer house remains (S-0414, claim 6), so this radar state is
   observable only during the end delay (S-0414, claim 8). In our engine it lasts while the
   defeated person watches the remaining sides fight, until the game is over.
   **Departure (scope, no switch):** in the original
   only the local player's house follows these rules; in our engine every player house whose radar
   map is shown (the local player's, or another's shown to an observer) follows them, each until its
   own defeat. Radar reaches no game state, so nothing else changes (decision of the project
   lead, 2026-10-09, item 4, and 2026-10-11). Claims 6 to 10 speak of the house *H* whose radar map is shown.
   - Backing: `measurable` (scenarios R1, R2 and R8)
   - Confidence: high for the radar flag, low power, outage, free radar, the first-building rule
     and radar after defeat; medium for the other conditions (the switch, being sold, the order of
     ownership)

2. **When availability changes.** A house's radar is re-evaluated against claim 1 in the house's
   next update in the houses' part of the tick (S-0203) after an event that triggers
   re-evaluation. The known such events are: its power balance changes, it gains, loses or starts
   selling a building, an electromagnetic lock of one of its radar buildings ends, and a radar
   outage of the house starts or ends, and an upgrade is successfully placed on one of its
   buildings (S-0406, claim 12), whether or not the upgrade changes its power balance (the list may
   be incomplete, Open questions). The upgrade's own radar flag does not grant radar: no building
   of the upgrade type enters play, and claim 1 still tests the host building's type. The free-radar
   flag is not an event: it is fixed for the game (claim 1). After its defeat (claim 1) a house's
   radar is no longer re-evaluated. The spy satellite rule that S-0409 owns (its claim 15) has
   its own timing there, and nothing about it is a radar matter here.
   Whether switching a radar building off or on, or the start of an electromagnetic lock, is itself
   an event is not known (Open questions; scenarios R1 and R2 read the panel only after a known
   event, and record what happens without one). Until that is settled, our engine treats these
   three as events too, so the recheck runs in the next house update after each (a choice of this
   spec, not a statement about the original, low confidence: if the original waits for a later
   event, claim 2 is revised).
   - Backing: `measurable` (scenarios R1 and R2, the recorder logs the tick on which the panel
     switches; the three uncertain causes are recorded only, not falsifiers). The upgrade-specific
     request is `unmeasured` as in S-0406 claim 12: an upgrade can change the answer through power,
     already a listed cause, while with unchanged inputs and radar already matching claim 1 a
     recheck leaves no observable difference
   - Confidence: medium for the listed events and the timing, including upgrade placement; low
     for the completeness of the list and for the three uncertain causes

3. **Radar on and off.** When a house's radar becomes available or unavailable, its radar panel
   plays the radar-on or radar-off sound (Import mapping) and its opening or closing animation.
   Without radar the panel shows no map. The sidebar spec owns the panel's animation.
   - Backing: `measurable` (scenario R1)
   - Confidence: medium

4. **Radar points.** The radar map shows the usable cells only (the height-aware test of S-0009,
   claim 5a). Legal cell (*x*, *y*) covers the two radar points (*u*, *v*) and (*u* + 1, *v*) with
   *u* = *x* − *y* + *a* and *v* = *x* + *y* − *b*, where *b* is the smallest *x* + *y* of a usable
   cell and *a* is fixed by the local area (Formulas F1). A cell whose left point would be column −1
   covers only column 0, and a cell whose left point would be column 2·*L*w − 1 (*L*w the local
   area's width) covers only that column. The numbers *u* and *v* as computed are the columns and
   rows of the point grid; there is no further shift. In a local area of the usual rectangular
   shape the mapping puts the leftmost covered column at 0 (the area's left edge has *u* equal to
   −1 or 0 in every row), and in every local area the choice of *b* puts the top row at *v* = 0. For a local
   area for which the leftmost covered column is not 0, and for any cell with *u* ≤ −2 or
   *u* ≥ 2·*L*w, nothing more is known: the original's mapping is not known. Our engine applies the formulas of this spec as written
   there (a choice of this spec, not a statement about the original: if scenario R3 part C shows
   that the original maps such an area differently, claim 4 is revised), and *W* of F1 may not
   describe the original's grid (low confidence, Open questions, scenario R3 part C). So the
   radar map shows the map in the same orientation as the battlefield view, one point row per screen
   half-row of cells. Each radar point carries a point colour (claim 9); the two points of one cell have the same one. A
   radar point that no usable cell covers is black (Terms, *terrain picture, point colour*); in the
   usual local area every point of the grid up to each row's end is covered.
   - Backing: `measurable` (scenario R3 part A; part C for the unusual local areas)
   - Confidence: medium for the mapping in the usual case; low for the exact values of *a* and *b*
     and for local areas where the leftmost column is not 0

5. **Scaling into the panel.** The grid of radar points, *W* columns by *N* rows, is scaled
   uniformly into a radar image centred in the 140 × 108 radar map. The scale *s* is 140 / *W*, or
   108 / *N* when *N* × (140 / *W*) ≥ 108, held in single precision; the image is then 140 pixels
   wide and trunc(*s* × *N*) high, or trunc(*s* × *W*) wide and 108 high (Formulas F2). The terrain
   colour of a radar pixel is the area-weighted mean of the point colours it covers: the pixel is a
   rectangle of *W* / *w* points by *N* / *h* points (*w*, *h* the image's size), and each point
   counts with the area of its overlap with that rectangle (Formulas F6). The radar map's pixels
   outside the image show the panel's own background (Terms, *radar map*; low confidence).
   Shroud, gap fields and objects are not scaled this way: each radar pixel tests one cell (claim
   6), and each object lies at the radar pixels of claim 8.
   - Backing: `measurable` (scenario R3, parts A, B, D and E)
   - Confidence: high for the factor, the image size and the centring; medium for the mean (its
     weights are exact, its rounding at exact halves is not established, F6); low for the margin

6. **What a radar pixel shows.** For the house *H* whose radar map is shown, each radar pixel shows, in this
   order of precedence:
   - the colour of the first object that is shown at the pixel (Terms; claims 7 and 8),
   - otherwise, when the pixel's cell lies in a friendly gap field of *H* (Terms: *H*'s own or an ally's,
     or any gap field once *H* has been defeated), the pixel's own terrain colour (the terrain
     picture's pixel, not the point colour of the cell shown at it) with each channel
     value halved (truncated; *c* becomes ⌊*c* / 2⌋ on the radar colour's channel value at its own
     bit width, Terms *radar colour*),
   - otherwise, when the pixel's cell is shrouded for *H*, black,
   - otherwise the pixel's terrain colour (claims 5 and 9).

   "The pixel's cell" is the cell shown at the pixel (Formulas F3), found from the pixel by the
   inverse of the radar point mapping, in floating point, and rounded to the nearest cell.
   "Shrouded" and "lies in a friendly gap field" are tested with the projection of S-0110's point
   test (claim 8) on the point at the cell's centre (cell centre in *X* and *Y*) and at the ground
   height of that cell (S-0101): when *z*, the number of whole height levels of that point (Terms), is odd,
   the gap test needs both candidate cells to lie in a friendly gap field, while the shroud test is
   the point test (shrouded when neither is explored). Because the gap
   test comes first, a friendly gap field shows darkened terrain even over cells that
   are shrouded for *H*. The cell shown at a pixel need not be the cell whose point colour
   contributes to the pixel: at scale 1 the right-hand point of cell (*x*, *y*) shows cell
   (*x* + 1, *y*) in the shroud, gap and object tests (F3 with F1). That is the
   original's behaviour, a quirk (Edge cases, `quirks.radar_pixel_cell_shift`, default on). F3 can
   give a cell that is not a usable cell (at the right-hand edge and the bottom rows of the image):
   the original does not appear to guard against it, and its result is not established. Our engine
   treats such a cell as not shrouded and in no gap field, so the pixel shows its terrain colour
   (a choice of this spec, low confidence, scenario R3 part F).
   - Backing: `measurable` (scenario R4; R3 part A for the pixel-to-cell shift, R3 part F for an
     unusable cell)
   - Confidence: high for the order of precedence, black shroud and object colours; medium for the
     gap darkening and the pixel-to-cell rounding and shift, and low
     for the cell tests at an unusable cell

7. **Which objects are shown.** Only radar objects (Terms) can be listed: a tree, an overlay, a
   smudge, an animation or a projectile is never drawn as an object, so its cell's terrain colour
   (claim 9) shows. A radar object is *listed* at its radar pixel when it passes the tests of
   its case below, and *shown* there when it is listed and also passes the draw-time tests below
   (Terms). Common to both cases: it is on the map, it is not inside a
   transport or building, and its type is not invisible. Then:
   - **Owned by *H*:** it is discovered by *H* (S-0409, Terms). Nothing else is tested, so an own
     object that is discovered stays listed on unshrouded, shrouded or out-of-area ground alike.
     (An object that *H* does not own but that counts as its own for sight, S-0409, claim 1, is not
     covered: the original appears to treat only *H*'s own objects this way; low confidence.)
   - **Not owned by *H*:** it is inside the usable area (the height-aware test of S-0009, claim 5a,
     at its position), its point is not shrouded for *H* (S-0110's point test), it has not gained
     the radar-invisible veterancy ability (S-0311), and it is not hidden from *H* by cloaking or by
     being underground (the special-abilities specs own those states).
   Draw-time tests, for every listed object, applied at the object's radar pixel:
   - when it is not owned by *H*, the cell shown at its radar pixel is not shrouded for *H*
     (claim 6),
   - its type is not radar-invisible, unless *H* allies its owner (the relation is one-way, glossary
     *alliance*: *H*'s side; low confidence for the side),
   - its type is not insignificant, unless its type is radar-visible or it has an owner whose
     country is not passive for multiplayer (so insignificant objects of the neutral and special
     houses, and ownerless ones, are hidden).
   - Backing: `measurable` (scenarios R5 parts A to C, E and G, R6 part B, R9); the ownerless
     insignificant object is `unmeasured` (R5 part D, no skirmish setup creates it)
   - Confidence: medium for the restriction to radar objects (the evidence so far shows only the
     four radar-object kinds ever listed, and no other kind); high for the type flags, the
     passive-country rule (both exceptions to "insignificant", R5 part G), the shroud test at the
     pixel and the own-object case; low for the cloaking, underground and veterancy-ability tests (a
     further condition also decides whether such an object is listed, and for some objects the
     original lists one although the rule above hides it; which ones is not established, Open
     questions)

8. **Object position, colour and priority.** An object that is not a building lies at the one
   radar pixel of its stored position (F4); a building lies at a set of radar pixels around the
   pixel of its stored position that covers its foundation at the current scale (low confidence for
   the set). An object of *H* outside the radar image, other than a building, is drawn at the
   nearest pixel on the image's edge; other houses' objects outside the usable area are not shown
   (claim 7). A shown object is drawn in its owner's house colour (the 8-bit triple of the house's colour
   definition, S-0103, converted as Terms *radar colour* says), or in the colour of the house it is
   disguised as when it is disguised against *H*; a radar object without an owner is drawn in a fixed
   neutral colour. When several listed objects lie at one radar pixel, the colour is that of the
   first of them in this order that is shown (passes the draw-time tests of claim 7): *H*'s own
   objects first, the one with the latest arrival (Terms) first among them, then the other objects,
   the one with the earliest arrival first. Because arrivals are ordered within a tick by the order of
   the objects' turns, of two own objects that arrive at one pixel in the same tick the one whose turn
   runs later wins, and of two other objects the one whose turn runs earlier (low confidence for these
   same-tick results). A building is listed at every pixel of its set, with the building's own arrival
   at each of them. A pixel of a building's set that lies outside the radar image is not drawn (a
   building is never moved to the image's edge). An object of another house whose pixel lies outside
   the image is not drawn either (low confidence; it can only occur at the edge of the usable area).
   An object is drawn at the pixel where it is listed (Terms, *Listed, arrival*): the pixel F4 gives
   for its stored position as evaluated at the start of its latest turn, not for its position at
   the drawn frame. An object that crosses into another pixel during the turn of tick *n* is
   therefore still drawn at the old pixel in the drawn frames of tick *n*, and appears at the new
   pixel from the drawn frames of tick *n* + 1 on, the tick in which the start-of-turn evaluation
   sees the new pixel, so the drawn pixel lags the object's position by up to one tick (low
   confidence for the lag; scenario R5 part F). The draw-time tests of claim 7, and the shroud, gap
   and terrain colours of claim 6, use the cells' states at the drawn frame itself. Because F4
   puts an object at the centre of cell (*x*, *y*) on point row *v* + 1, at scale 1 the cell
   shown at its pixel (F3) is (*x* + 1, *y* + 1), so the draw-time shroud test of an object of
   another house is made on that cell, not on the cell it stands on (scenario R5 part F's setup
   makes it observable; R3 part A measures the pixel).
   - Backing: `measurable` (scenarios R5 parts A to C and F, R10, R11); two rules are
     `unmeasured`. The neutral colour of an ownerless radar object: no skirmish setup is known that
     creates a radar object without a house. The disguise colour: disguising is an order of the
     object's owner, the other houses are computer players that no scenario can make
     disguise a unit, and a disguise of the human's own object is not a case against *H*, so no
     skirmish setup under the scenarios' control creates it
   - Confidence: high for the arrival order and the single pixel of an object that is not a building; medium for the edge
     clamp; low for a building's set of pixels and the pixels outside the image, for the same-tick
     results, for the one-tick lag of the drawn pixel, for the neutral colour and for the disguise colour
     (including whether the disguise depends on the viewer, Open questions)

9. **Terrain colour.** Every radar point of a cell gets one point colour, the same for both points
   of the cell (an 8-bit triple, Terms). The first case below that applies decides:
   - a cell holding a terrain object (such as a tree) takes the fixed pale yellow of the Constants.
   - otherwise a cell with a bridge takes the frame radar colour (Terms) of the bridge's sprite,
     image frame 0 (low confidence). Which cells count as having a bridge here is not
     established: either every cell with a bridge-part overlay, or only a cell that carries a high
     bridge deck (Open questions, scenario R6 part F). A bridge-part overlay in a cell that does not
     count takes the overlay case below.
   - otherwise a cell with an overlay takes a frame radar colour, as follows. An overlay at list
     position 239 counts as no overlay, whatever its type (a resource included), so the next
     case applies. A resource overlay takes that of the resource's sprite at the cell's own image
     frame, with the colour as stored, or the fixed grey-yellow of the Constants when the resource
     type has no sprite. Any other overlay takes that of its own sprite, at the image frame and
     with the channel order that table T1 of Edge cases gives for its list position (low
     confidence, Open questions, `quirks.overlay_list_position_rules`); an overlay type without a
     loaded sprite gives (0, 0, 0).
   - otherwise the cell takes its tile colour: the first of the two tile radar colours of the tile cell
     that the cell shows (S-0010, claim 20; Terms there; the second has no effect on the radar), taken from the
     image variant the cell shows, with each channel *c* turned into
     ⌊trunc(min(*c* · *f*, 255)) / 2⌋ (Formulas F7; *f* is the theater brightness of the
     Constants); the cell takes the fixed dark grey of the Constants, not scaled and not
     halved, when its tile image has no tile radar colour for the cell's sub-tile.
   The halving belongs to the tile colour only: the other colours are used as given. The point
   colours are then scaled into the terrain picture (claim 5).
   - Backing: `measurable` (scenario R6, parts A to F)
   - Confidence: medium for the tile colour, its brightness and halving, and for the order of the
     cases; low for the bridge and overlay cases, the image-frame rule, the channel exchange and the
     fallback cases (the colour cases are not yet confirmed by observation)

10. **Flash under attack.** An object of the house *H* whose radar map is shown, while its radar flash countdown runs
    (it is set to the combat flash time when the object is hurt, combat specs) is drawn with its
    colour inverted (each channel value *c* of *k* bits replaced by 2^*k* − 1 − *c*, Terms *radar colour*) in every
    drawn frame whose tick has
    trunc((*t* − 1) / *F*) odd, where *t* is the countdown's remaining ticks and *F* the flash
    frame time; in other ticks it has its normal colour. *F* is an integer; F5 says what happens for *F* of 0 or less ("odd" means not divisible by 2, for a negative number too). The phase follows the tick, so drawn frames
    between ticks (a freeze) keep the colour of the tick shown. Objects of other houses do not
    flash.
    - Backing: `measurable` (scenario R7; part D for a negative *F*; *F* = 0 is a departure with no scenario)
    - Confidence: medium for the inversion and the phase; low for when the countdown is set

11. **Radar events and the view outline.** Radar events (S-0011, claim 32), beacons (markers that
    players place on the map) and the outline that marks the battlefield view's part of the map are
    drawn over the radar map; a click on the radar
    map moves the battlefield view (S-0102, *view jump*). Their details belong to the interface
    specs.
    - Backing: `unmeasured` here (pointer only; the owning specs carry the scenarios)
    - Confidence: high that they exist

## Formulas

**F1. Radar point of a cell** (claim 4). Integers. Over all usable cells (*x*, *y*): *b* =
min(*x* + *y*).

> *u* = *x* − *y* + *a*, *v* = *x* + *y* − *b*; points (*u*, *v*) and (*u* + 1, *v*), except that a
> cell with *u* = −1 covers only (0, *v*) and a cell with *u* = 2·*L*w − 1 covers only (*u*, *v*).

*a* is fixed by the local area's x origin *L*x (S-0009, claim 4) and the map width *W*ₘ: *a* =
*W*ₘ − 2·*L*x − 1 (low confidence for this closed form). In a local area of the usual rectangular
shape it gives the leftmost usable cells *u* = −1, so that the leftmost covered column is 0
(claim 4). It is not known to do so for every local area; where it does not, the closed form still
holds. For
a usable cell let *c* be its leftmost covered column (*u*, or 0 when *u* = −1) and *e* its number of
covered columns (2, or 1 when clipped). Then *W* = the largest *c* + *e* over the usable cells (the
last covered column + 1, counted from column 0) and *N* = max(*v*) + 1 over the usable cells. This
*W* is the width of the original's point grid when each row of the area starts at column 0 (claim
4). For other local areas the original's width may differ from it, and nothing more is known
(claim 4, low confidence, Open questions, scenario R3 part C).

**F2. Scale** (claim 5). *s* is a single-precision value, the quotient rounded toward zero to
single precision.

> *s*₀ = 140 / *W*.
>
> *s* = *s*₀ and (width, height) = (140, trunc(*s* × *N*)) when *N* × *s*₀ < 108, and
> *s* = 108 / *N* and (width, height) = (trunc(*s* × *W*), 108) when *N* × *s*₀ ≥ 108.
>
> The products *N* × *s*₀ and *s* × *W* are formed in double precision, as S-0205 (claim 4) has
> it. *s* has
> 24 significant bits and *N*, *W* are small integers, so both products are exact and the test and
> the truncation see the exact product, under either rounding mode.
>
> The image's top-left pixel lies at (trunc((140 − width) / 2), trunc((108 − height) / 2)) in the
> radar map.

**F3. Cell shown at radar pixel (*p*, *q*)** (claim 6). Floating point: *s* as in F2, the other
operations in double precision under the round-toward-zero mode of S-0205.

> *u*′ = *p* / *s* − *a*, *v*′ = *q* / *s* + *b*; cell = ( trunc((*u*′ + *v*′) × 0.5 + 0.5),
> trunc((*v*′ − *u*′) × 0.5 + 0.5) ).

**F4. Radar pixel of an object** at stored position (*X*, *Y*) in leptons (claim 8). *s* single
precision, the product exact in double precision.

> *p* = trunc( (256·*a* + *X* − *Y*) × *s* / 256 ), *q* = trunc( (*X* + *Y* − 256·*b*) × *s* / 256 ).
> For an object of *H* that is not a building, the pixel used is (min(max(*p*, 0), width − 1),
> min(max(*q*, 0), height − 1)), with width and height those of the image (F2). For every other
> object, (*p*, *q*) is used unclamped (for a building, the pixel its set lies around, claim 8).

**F5. Flash phase** (claim 10). Integers, *t* ≥ 1: inverted when trunc((*t* − 1) / *F*) is odd, the
division truncated toward zero. *F* is read from rules as an integer; for *F* ≥ 1 this is the
original. For a negative *F* the same formula applies. *F* = 0 makes the original divide
by zero (a program error): our engine treats it as a *departure* (glossary): an importer warning,
and no inversion at all.

**F6. Terrain picture** (claims 5 and 9). Single precision for *s*, the spans and the positions, and
double precision under the round-toward-zero mode of S-0205 for the sums; integers for channel
values. Let *w*, *h* be the image's width and height (F2) and *s* as in F2.

> *g*ₓ = *W* / *w*, *g*ᵧ = *N* / *h* (quotients rounded toward zero to single precision),
> *u*₀ = *p* / *s*, *v*₀ = *q* / *s*, *u*₁ = *u*₀ + *g*ₓ, *v*₁ = *v*₀ + *g*ᵧ (each result rounded
> toward zero to single precision, as S-0205 claim 5 gives for every conversion to single precision).
>
> The overlap of point column *j* with the pixel is *o*ₓ(*j*) = length of [*j*, *j* + 1) ∩ [*u*₀, *u*₁),
> and *o*ᵧ(*i*) likewise for rows with *v*₀, *v*₁; each length is formed in double precision from
> the single-precision bounds, rounded toward zero (S-0205 claim 5). For a channel, the mean is
> Σ *o*ₓ(*j*) · *o*ᵧ(*i*) · *c*(*j*, *i*) / (*g*ₓ · *g*ᵧ) over the points (*j*, *i*) of the grid, and the
> pixel's 8-bit channel value is min(255, trunc(mean + 0.5)) (rounded half up; trunc toward zero,
> the mean is not negative).

A pixel that lies inside one point has the weight of its whole span. Points outside the grid
(*j* ≥ *W* or *i* ≥ *N*) are left out. A point inside the grid that no usable cell covers has the
point colour (0, 0, 0) and enters the mean as such. The original sums in floating point, so a mean that falls
within about 10⁻⁶ of an exact half may round the other way in another implementation; this
spec does not fix those cases (Open questions). The 8-bit channel values then become radar colours
by Terms, *radar colour*.

**F7. Tile colour of a point** (claim 9). *c* is one channel of the tile cell's tile radar colour
(Terms), an integer from 0 to 255; *f* is the theater brightness, a single-precision number (Constants).

> *c*′ = ⌊ trunc( min(*c* · *f*, 255) ) / 2 ⌋.
>
> The product *c* · *f* is formed in double precision under the round-toward-zero mode of S-0205,
> trunc drops its fraction, and the division by 2 is integer, with the fraction dropped.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Radar map width | 140 | pixels | claim 5; fixed by the original, not taken from the sidebar art |
| Radar map height | 108 | pixels | claim 5; fixed by the original, not taken from the sidebar art |
| Radar points per cell | 2 | points, side by side | claim 4 |
| Gap darkening | ½ | factor per colour channel, truncated | claim 6 |
| Shroud colour | (0, 0, 0) | 8-bit triple: red, green, blue, each 0 to 255 | claim 6 |
| Resource colour without sprite | (170, 170, 130) | 8-bit triple (0 to 255 per channel) | claim 9, low confidence |
| Terrain-object cell colour | (200, 200, 160) | 8-bit triple (0 to 255 per channel) | claim 9, low confidence |
| Fallback terrain colour | (60, 60, 60) | 8-bit triple (0 to 255 per channel) | claim 9: a cell whose tile image has no tile radar colour for its sub-tile; low confidence |
| Theater brightness *f*, snow theater | 0.8 | single-precision factor (stored value 0.800000011920929) | claim 9 |
| Theater brightness *f*, every other theater | 1 | single-precision factor | claim 9 |
| Overlay positions with image frame 1 | 74 to 99 and 205 to 230 | position in the overlay type list, counted from 0 | claim 9, table T1; low confidence |
| Overlay position treated as no overlay | 239 | position in the overlay type list, counted from 0 | claim 9, table T1; low confidence |
| Overlay positions with green and blue exchanged | 127 to 138 and 147 to 158 | position in the overlay type list, counted from 0 | claim 9, table T1; low confidence |
| Half-step added to the terrain mean | ½ | 8-bit channel value (added before truncation) | F6 |
| Flash frame time, absent | 7 | ticks | Import mapping |
| Combat flash time, absent | 21 | ticks | Import mapping |

## State and ordering

Each recheck (claim 2) sets the house's radar to what claim 1 says at that moment. The table lists
the transitions that the events of claim 2 cause.

| State of house *H* | Event | Next state and effect |
|---|---|---|
| radar | at the recheck after an event, claim 1 is false: an outage is running, or the free-radar flag is not set and one of these holds: power is low, no building qualifies, the first qualifying building is locked or being erased | no radar; radar-off sound; panel closes |
| radar | at the recheck, claim 1 is true | radar stays |
| no radar | at the recheck after an event, claim 1 is true (outage over, power restored, a radar building switched on, a lock over, a qualifying building becoming the first) | radar; radar-on sound; panel opens |
| no radar | at the recheck, claim 1 is false | no radar stays |
| any | *H* is defeated (S-0414, claim 5) | radar on in the tick of the defeat (panel opens and the radar-on sound plays if it was closed, claim 3); no recheck afterwards |

A switch-off, a switch-on and an electromagnetic lock starting each cause a recheck in our engine
(claim 2, a choice until R1 and R2 settle the original's behaviour); a lock ending is a known cause. Rechecks run in the houses' part of the tick (claim 2). The
terrain, shroud and gap colours follow the cells' states at the time of each drawn frame. Objects
are drawn at their listed pixels (claim 8): a pixel change that happens in a turn shows from the
next tick's drawn frames on. The order of objects at one pixel is claim 8's.

## Edge cases and quirks

- **`quirks.radar_first_building_decides`** (default on; this spec owns the switch, S-0405 claim 12
  and S-0409 point to it). Only the first qualifying radar building
  is looked at: when it is under an electromagnetic lock or being erased, the house has no radar even
  if another radar building is working. Off: the house has radar when any qualifying radar building
  is neither locked nor being erased.
- **`quirks.overlay_list_position_rules`** (default on; low confidence). Claim 9's overlay colours
  depend on the overlay type's fixed position in the overlay type list, counted from 0 in the order
  the types are registered (S-0004), as table T1 gives. Off: every overlay uses its cell's own
  image frame, its colour as stored, and an overlay at position 239 colours its cell like any
  other overlay. A mod that reorders or extends the overlay list would want it off.

  **T1. Overlay colour by list position** (switch on). Rows 1 to 3 apply to overlay types
  that are not resources; row 4 applies to every overlay type, a resource included. A resource
  overlay type at any other position takes the cell's own image frame, colour as stored.

  | Position of the overlay type | Image frame used | Colour |
  |---|---|---|
  | 74 to 99, 205 to 230 | 1 | as stored |
  | 127 to 138, 147 to 158 | the cell's own | green and blue channels exchanged |
  | any other | the cell's own | as stored |
  | 239 (every overlay type) | none: the cell counts as having no overlay | the tile colour shows |
- **`quirks.radar_pixel_cell_shift`** (default on; whether the project keeps a switch is the
  lead's call, Questions for the project lead). The cell that a radar pixel stands for in the
  shroud, gap and object tests is the one F3 gives, which at scale 1 is not the cell whose point
  colour the pixel shows: the pixel at the right-hand point of cell (*x*, *y*) stands for cell
  (*x* + 1, *y*), and an object at the centre of cell (*x*, *y*) is tested on (*x* + 1, *y* + 1)
  (claims 6 and 8). Off: the cell shown at pixel (*p*, *q*) is the cell that covers the radar point
  (⌊*p* / *s*⌋, ⌊*q* / *s*⌋) of F1, with *p* / *s* and *q* / *s* formed as in F3 (the single-precision *s*, the
  quotient in double precision under round toward zero, then truncated; *p* and *q* are not
  negative, so truncation and ⌊ ⌋ agree; F6's rounding of the bounds to single precision is not applied here, and
  the two forms can differ only for a quotient within single-precision rounding of an integer).
  By F1 that is the cell (*x*, *y*) with *x* + *y* = ⌊*q* / *s*⌋ + *b* and *x* − *y* equal to
  whichever of ⌊*p* / *s*⌋ − *a* and ⌊*p* / *s*⌋ − *a* − 1 has the same parity as
  ⌊*q* / *s*⌋ + *b*. This alternative is ours, not the original's, so it has no scenario. It
  changes only which cells' shroud, gap and object tests apply to a pixel. It does not make an
  object be tested on the cell it stands on: F4 puts an object at the centre of cell (*x*, *y*) on
  point row *v* + 1, so with the switch off that object is tested on cell (*x*, *y* + 1) instead
  of (*x* + 1, *y* + 1).
- **Insignificant neutral objects.** Insignificant objects of passive countries are hidden unless
  radar-visible (claim 7); this is the original's design and needs no switch.
- **Campaign-only behaviour** (out of scope): in a campaign mission the original also requires some
  buildings to have been discovered before they give radar, and discovers objects by the shroud
  test.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, building type section, `Radar` | radar flag (claim 1) | boolean | no |
| Map, `[Basic]`, `FreeRadar` | free radar (claim 1) | boolean | no |
| Rules, infantry, vehicle, aircraft or building type section, `RadarInvisible` | radar-invisible (claim 7) | boolean | no |
| Rules, infantry, vehicle, aircraft or building type section, `Insignificant` | insignificant (claim 7) | boolean | no |
| Rules, infantry, vehicle, aircraft or building type section, `Invisible` | invisible (claim 7) | boolean | no |
| Rules, infantry, vehicle, aircraft or building type section, `RadarVisible` | radar-visible (claim 7) | boolean | no |
| Rules, `[General]`, `FlashFrameTime` | flash frame time *F* (claim 10) | integer, ticks | 7 (engine default) |
| Rules, `[General]`, `RadarCombatFlashTime` | combat flash time (claim 10) | integer, ticks | 21 (engine default) |
| Rules, `[AudioVisual]`, `RadarOn`, `RadarOff` | radar-on and radar-off sounds (claim 3) | sound name | none |
| Rules, country section, `MultiplayPassive` | passive country (claim 7) | boolean | no |

## Measurement scenarios

**Common setup.** The glossary's *measurement setup*; a skirmish with one human player and one
enemy bystander computer player (scenarios that need an allied or a second one say so) on a
purpose-made map, fog of war off, any game speed; screenshots of the radar
panel at a fixed screen resolution; a *recorder* for tick numbers. One valid run per case.

- **R1 (claims 1 to 3).** The human player owns a power plant, a stock radar building whose power
  drain is not 0 (map data), and two spare buildings with no power output, no power drain and no
  radar flag (map data). A cause that claim 2 does not know to be an event is always followed by a
  known event, and the panel is read after that event's recheck: the sale of a spare building (it
  starts selling, a known event). Steps, by scripted input: (a) note the panel at the start; (b)
  switch the radar building off with the power switch; (b′) start selling the first spare building,
  (c) switch the radar building on; (c′) start selling the second spare building; (d) build
  buildings until power is low; (e) sell a power-draining building to restore power; (f) sell the
  radar building; (g) place and build another radar building (the house gains a building). A
  second run does (b) and (c) alone, without (b′) and (c′), and records the tick on which the panel
  changes (recorder): it settles the open question of claim 2 and falsifies nothing. A second map
  starts a radar outage of the human house with a radar-outage trigger action at a known tick and
  ends it at another known tick (a trigger action, because the computer player's firing of a
  weather storm is not under the scenario's control; a repeat with a weather storm fired at the
  human player is qualitative only). A third map has the map's free-radar flag set and no radar
  building. Observable: whether the radar map is shown after each step, the tick on which it
  changes (recorder), the sound played and the panel's animation at each change. Expected: shown
  at (a), after (c′), at (e), at (g) and with free radar; not shown after (b′), at (d), at (f) and
  during the outage; at (g) shown from the first house update after the placement, before the
  build-up ends (a building in build-up counts, claim 1). Each change to shown comes with the
  radar-on sound and the opening animation, and each change to not shown with the radar-off sound
  and the closing animation. Falsified if any of these differs (at (g) in particular: radar only
  once the build-up ends, which revises claim 1's build-up rule, or not at all), or if a change
  has no sound or the other sound. Timing (claim 2): the panel changes on the tick of *H*'s first
  house update (S-0203) after each known cause (the sale at (b′) and (c′), a power change at (d)
  and (e), the sale at (f), the placement at (g), the outage's start and end). Falsified if, for
  one of these causes, the panel changes on another tick.
- **R2 (claim 1, first building).** Named test data: the human player owns two radar buildings
  (map data, the first placed first), a spare building with no power output, no power drain and no
  radar flag (map data), and a human test vehicle whose weapon applies the electromagnetic lock.
  The vehicle is given an attack order with force-fire held (scripted input, S-0214 claim 6) on
  the first radar building, and the next tick the human starts selling the spare building (a known
  event of claim 2, so the house is rechecked while the lock runs; the lock must still run at that
  recheck, otherwise the run is void and repeated). No computer player is involved, and the
  bystander computer player only keeps the match running. Observable: whether the radar map is
  shown, and the tick it changes. Expected: no radar from the recheck of the sale until the lock
  ends, and radar again from the recheck after the lock's end (a known event). A repeat with the
  order on the second radar building (and the same sale) keeps radar throughout. A variant builds
  the second radar building during play, so that it is certainly the later one owned. Falsified
  if: radar stays at the recheck of the sale while the first is locked (that is the switch turned
  off), or is lost when only the second is locked, or returns before the lock's end. Timing: the
  panel changes on the tick of *H*'s first house update after the sale starts and after the lock
  ends; falsified if it changes on another tick. The tick at which the panel changes at the lock's
  start is recorded only, since it settles the open question of claim 2 and cannot falsify
  anything.
- **R3 (claims 4, 5 and 6).** Three maps with usable areas of different aspect ratios (wide, tall,
  square). Part A (mapping, size and pixels): observable: the size and position of the radar
  image in the panel, and the outline of the map's cells. Expected: the image fills 140 pixels
  across or 108 down, as F2 gives, centred, the map is shown in battlefield orientation. In each
  map a human vehicle stands on a known cell, and its pixel in the image matches F4, and a lone
  explored cell in shroud lights the pixels whose F3 cell it is; the vehicle at a cell's centre
  lies on point row *v* + 1. Falsified if: another size, another orientation, or pixels off by one
  or more from F3 and F4.
  Part B: the radar map's pixels outside the image, on both sides of the image, show the panel's
  own background (the same as with radar off); falsified if they are black, terrain or anything
  else.
  Part C (claim 4, low confidence): a map whose local area is a rectangle at a corner of a large
  map and another that is placed so that its leftmost covered column is not 0 (the unusual
  case of claim 4); observable: the outline of the shown cells and the image size. Expected: the
  outline is the cell mapping of F1 and F3, with *W* of F1 where each row starts at column 0.
  Falsified if the shown cells are shifted so that the leftmost covered column is 0, or if the
  image size is not that of F2 with F1's *W*. Where the second map's result disagrees with F1's
  *W*, record it and revise claim 4.
  Part D (claim 5, the mean of F6): a map whose point grid gives a scale below 1 that is not a
  power of two (for example *W* = 210, so *s* = 2/3 and each pixel spans 1.5 points), with terrain
  cells of two test tiles whose tile radar colours differ strongly (named test data, S-0010) laid
  in alternating columns and rows, chosen so that no expected mean lies within 0.01 of an exact
  half. Observable: the pixel colours, as 8-bit-precision channel values read through the radar
  colour conversion. Expected: each pixel equals F6's mean for the point colours of claim 9, then
  the conversion of Terms, *radar colour*; in particular the pixels over a colour boundary are
  in-between values weighted by overlap. Falsified if a boundary pixel shows one of the two
  colours alone, or an unweighted mean of the covered points, or a channel differs from F6 by one
  or more steps after the conversion. Pixels whose mean lies within 10⁻⁶ of an exact half are not
  measured (Open questions).
  Part E (the margin of claim 5, low confidence): in the wide map, the image height is the smaller
  of the two limits, and both sides of the *N* × *s*₀ = 108 test are checked with two maps whose
  point grids differ by one row. Falsified if the other limit decides.
  Part F (claim 6, an edge pixel whose cell is not a usable cell; low confidence): at scale 1,
  with the cells of the rightmost usable column and of the bottom rows explored, the colour of the
  radar pixels at the right-hand points of the rightmost cells and at the lowest rows. Expected:
  the terrain colour of claim 9 (not black), as claim 6 states. Falsified if they are black.
- **R4 (claim 6).** The human player owns a powered gap generator in explored ground and has an
  unexplored area beside it. Expected: the gap generator's field is drawn at half brightness on the
  radar map (it covers nothing for its owner, S-0409), unexplored cells are black, explored cells
  show terrain; where the field reaches over unexplored cells, the radar shows darkened terrain
  there, not black. Observable: pixel colours of the radar map. Falsified if: the field is not darkened, the shroud is
  not black, or the field is black over unexplored cells. Part B: an allied computer player's gap
  generator over explored ground darkens the same way for the human player, and an enemy's does
  not. Falsified if the allied field is not darkened, or the enemy's is. Part C: after the human house is defeated (as R8), a gap field of any house darkens.
  Falsified if not. Part D: pixel channel values, one explored terrain pixel inside a gap field and its twin
  outside it over the same flat ground, at a scale of 2 or more, with 5, 6 and 5 bits for red,
  green and blue (Terms, *radar colour*); falsified if the darkened channel values are not ⌊*c* / 2⌋
  of the plain ones. Part E (claim 6, pixel or cell colour): a gap field over a boundary between two
  terrain tiles of strongly different tile radar colours, at a scale below 1 that is not a power of two
  (as R3 part D), so that a pixel's footprint straddles the boundary. Expected: the darkened pixel
  is half of the pixel's mean colour (F6), channel by channel. Falsified if it is half of the point
  colour of the cell shown at the pixel (claim 6 would then change).
- **R5 (claims 7 and 8).** Test map with two computer players besides the human: *E*₁ (the
  bystander, an enemy) and *E*₂ (a second computer player, an enemy at the start and allied to the
  human later where a part says so); part G adds a third, *E*₃. The houses have house colours that
  differ clearly (map data). Every ordering part below puts the objects in explored ground, on one radar pixel (the
  stored positions of the vehicles stand in neighbouring cells that F4 maps to one pixel, which
  needs a map large enough that the scale is below 1), and the vehicles arrive one after the
  other.
  Part A (the order of arrival). (i) Two enemy vehicles of different houses, one of *E*₁ and one
  of *E*₂, arrive one after the other, and a repeat in the opposite order. Expected: the pixel
  shows the colour of the one that arrived first. (ii) Two human vehicles, *P*₁ and *P*₂, arrive
  one after the other; both are in the human house colour, so the own order is made visible by the
  flash of claim 10: the override section gives the combat flash time a long value (named test
  data) and, before the later arrival, another human vehicle hurts the vehicle named for the run
  with force-fire (scripted input, as in R2), so that its radar flash countdown runs while the
  later one arrives and afterwards. Runs: (1) *P*₁ first, *P*₂ second, *P*₁ hurt; (2) the same
  order, *P*₂ hurt; (3) *P*₂ first, *P*₁ second, *P*₂ hurt. Expected: (1) the pixel does not
  alternate (it shows *P*₂, the later arrival); (2) it alternates between the human colour and
  its inverse (it shows *P*₂, which flashes); (3) it does not alternate (it shows *P*₁, the later
  arrival, which is not hurt). Falsified if: the order of any (i) run differs, or the pixel
  alternates in (1) or (3), or does not alternate in (2).
  Further objects, each on its own pixel in explored ground at scale 1 or more: an enemy vehicle
  of a type made radar-invisible by the override section; an insignificant neutral object (a
  building of a neutral house, of a type made insignificant by the override section); an enemy
  vehicle in shroud. Observable: the colour of each pixel. Expected: none of the three is shown,
  and the same radar-invisible vehicle is shown after the human player has an alliance with its
  owner (the vehicle is *E*₂'s, and the alliance is made by scripted input, S-0414, claims 12 and
  13). Falsified if any differs.
  Part B (arrival is not changed by the draw-time tests): a radar-invisible vehicle of *E*₂ stands
  at a pixel, a normal vehicle of *E*₁ arrives at that pixel later, and then the human player
  allies *E*₂ (by scripted input, S-0414, claims 12 and 13). Expected: before the alliance the
  pixel shows *E*₁'s colour (the invisible one is not shown) and after it *E*₂'s colour (it kept
  the earlier arrival while hidden). Falsified if it shows *E*₁'s colour after the alliance.
  Part C (same tick): two enemy vehicles, one of *E*₁ and one of *E*₂, arrive at one pixel in the
  same tick, placed so that the order of their turns (S-0203) is known, and a repeat with the
  order swapped; likewise two human vehicles, with the flash marker of part A (ii) on the one that
  is hurt, in both orders of the turns. Expected: of the enemy vehicles the one whose turn runs
  earlier is shown, and of the human vehicles the one whose turn runs later (the pixel alternates
  when the later one is the hurt one). Falsified if either is the other way round (that settles
  the low-confidence same-tick results of claim 8).
  Part D: an insignificant radar object without an owner is not shown: `unmeasured`, because no
  skirmish setup is known that creates an ownerless radar object (as in claim 8).
  Part G (the exceptions to "insignificant", claim 7). Types are made insignificant, and where
  stated radar-visible, by the override section (named test data). Each object is on its own pixel
  in explored ground at scale 1 or more. Objects: (i) a building of the neutral house (map data)
  of a type that is insignificant and radar-visible, (ii) a building of the neutral house of a
  type that is insignificant and not radar-visible, (iii) a vehicle of the bystander computer
  player *E*₁ of a type that is insignificant and not radar-visible, (iv) the human player's own
  vehicle of such a type, (v) a vehicle of such a type of a third computer player *E*₃ that is a
  *passive house*, named as a test object as S-0414 (scenario V3) does: the override section marks
  *E*₃'s country passive for multiplayer, and *E*₃ owns nothing else. Observable: the colour of
  each object's pixel. Expected: (i) shown in the neutral house's colour, (ii) not shown, (iii)
  shown in *E*₁'s house colour, (iv) shown in the human house colour, (v) not shown. Falsified
  when (i), (iii) or (iv) is not shown (an exception does not exist as stated) or when (ii) or (v)
  is shown (the passive-country test or the radar-visible flag differs).
  Part E (only radar objects are listed, claim 7): on explored ground at scale 1 or more, a lone
  tree, a smudge, an animation playing over a cell, a projectile in flight and an overlay each, one
  at a time and each without any radar object in its cell. Observable: the pixel colours of those
  cells. Expected: each shows its cell's terrain colour of claim 9 (for the overlay, its overlay
  colour), never the neutral object colour nor a house colour. Falsified if any of them shows an
  object colour.
  Part F (claim 8, the listed pixel and the cell tested; low confidence): on a map large enough
  that the scale is 1 or more, a human vehicle drives in a straight line across a pixel boundary
  at a known speed, with every drawn frame captured and tagged with its tick (recorder), and the
  tick in which its stored position (recorder) first maps to the new pixel by F4 is noted.
  Expected: the new pixel shows the vehicle's colour in the drawn frames of the next tick on, not
  in the drawn frames of the tick of the crossing, and the old pixel stops showing it at the same
  drawn frame. Falsified if the new pixel shows it in the drawn frames of the crossing tick (the
  object is then drawn at its current position, and claim 8's listing rule changes), or two ticks
  later. A second setup, at scale 1: an enemy vehicle standing at the centre of a cell *C* in
  explored ground, with the diagonal neighbour (*x* + 1, *y* + 1) shrouded and *C* not, then the
  reverse. Expected: it is hidden in the first setup and shown in the second. Falsified if the
  opposite.
- **R6 (claim 9).** Screenshots of the radar map, read as pixel colours at a scale of 2 or more (a
  small usable area), at pixels whose footprint lies inside one cell, so that no mean mixes
  colours; the expected point colours are computed from claim 9. Part A (tile colour, brightness
  and halving): flat ground of a test tile whose tile cells have named tile radar colours, in the snow
  theater and in each other theater. Expected: each channel is ⌊trunc(*c* · *f*) / 2⌋ with *f* the
  theater brightness of the Constants (0.8 for snow, 1 otherwise), then the conversion of Terms,
  *radar colour*; so with the same tile colours, the snow theater is never lighter than the others
  in any channel. Falsified if the colours differ by one or more steps from this,
  if the second tile radar colour is the one shown, or if the factor differs per theater from the
  Constants. Part B (other cases, one cell each, in a theater whose brightness is 1): ore and
  gems, a bridge, a wall, a lone tree, a resource whose type has no sprite (override section), and
  a tile cell whose sub-tile has no tile radar colour (a test tile with a missing cell). Expected: each
  shows the case of claim 9: the tree the pale yellow, the imageless resource the grey-yellow, the
  colourless tile cell the dark grey, none of them halved; the others as the overlay or bridge
  sprite gives. Falsified if one differs, or if a halved value shows for a non-tile case. Part C
  (order of cases): a tree standing on a resource cell, and a tree on a bridge. Expected: the pale
  yellow in both. Falsified if the resource or bridge colour shows. Part D (variant): a tile with
  two images of different tile radar colours, shown in each variant by the cell's alternate-image state
  (S-0010, claim 13). Expected: the colour of the image the cell shows (low confidence), and
  falsified if the plain image's colour shows for the variant. Part E (positions in the overlay
  type list, low confidence): overlay types at the list positions that table T1 of Edge cases names (also listed in Constants). An override section can only add types at the end of the
  list (glossary, *rules layer*), so the types are put at those positions by a loose copy of the
  rules file whose overlay type list places test types there (named test data, glossary
  *measurement setup*). Expected: as table T1 of Edge cases states, including the exchanged channels; a resource
  overlay type placed at a position of the first three ranges of T1 takes the cell's own image
  frame, its colour as stored, and an overlay at position 239 (a resource overlay included) shows
  the tile colour; falsified if not. The averaging is measured in R3 part D.
  Part F (the bridge case, claim 9; low confidence): one cell of a low bridge and one cell of a
  high bridge (S-0104), each on flat ground, in a theater whose brightness is 1. Observable: the
  pixel colour of each cell. Expected: the frame radar colour of the bridge's sprite, image frame
  0, for both; falsified if one shows another colour (the original's bridge case then covers only
  one of the two kinds: record which, and revise claim 9).
- **R7 (claim 10).** Part A: a human vehicle is attacked by another human vehicle with force-fire
  (scripted input, as in R2, so no computer player's firing is involved); capture every drawn
  frame of the radar map. Observable: the colour of the attacked vehicle's radar pixel in each
  drawn frame, with the tick number (recorder). Expected: the vehicle's pixel alternates between
  its colour and the inverted colour in runs of *F* ticks for the combat flash time. Part B: a
  human vehicle attacks an enemy vehicle that the map pre-places within weapon range; Expected:
  the enemy vehicle's pixel does not flash. Falsified if: no inversion, another phase length, or
  flashing of enemy units. Part C (a second part freezes the game while the vehicle flashes and
  captures drawn frames): expected, the colour of the frozen tick in every drawn frame; falsified
  if the colour alternates during the freeze. Part D (F5, negative frame time): the override
  section gives the flash frame time the value −7 in one run and 7 in another, with the same
  attack as part A; expected: the same phase in both runs (for a negative *F* the formula of F5
  applies, so the pixel alternates in runs of |*F*| ticks); falsified if they differ or the
  negative run never inverts. *F* = 0 has no scenario: the original fails there (F5).
- **R8 (claims 1 and 2, defeat).** The human player owns no radar building, the map has no free
  radar, and the human house is defeated (S-0414 scenario V2) while an enemy bystander computer
  house remains. Named test data: a test rules file lengthens the end-delay field of S-0414
  (claim 8) enough to capture the panel opening and several later drawn frames. The original ends
  the game at the human's defeat even with the computer house still alive (S-0414, claim 6), so
  all observations are taken within that delay, before the result screen. Observable: the radar
  map in the drawn frames from the defeat tick until the game is over, with the recorder, and
  the sound and the panel's animation at the defeat. Expected: radar comes on in the defeat tick
  with the radar-on sound and opening animation, the whole map is revealed, and the map stays
  shown once the panel opens throughout the remaining end delay. A repeat with a radar building
  and a radar outage that the radar-outage trigger action starts a few ticks before the defeat
  (so that it still runs at the defeat; a weather storm is not used, as in R1) shows the same.
  Falsified if: no radar at the defeat, or radar lost again within the end delay. Our engine's
  longer observation period, while the remaining hostile sides fight on (S-0414, claim 6), is a
  documented departure, not something this run can measure.
- **R9 (claim 7, own and not-owned cases; low-confidence tests).** Observable (all parts): whether
  the object's radar pixel shows its house colour, and on which tick it first does. Part A: an own vehicle not yet
  discovered by the human player (placed by a trigger at game start in a place the player has not
  yet seen, S-0409) is not shown; once discovered it is shown, also while it stands in shroud
  (after the area is covered again) and outside the usable area. Part B: an own vehicle inside a
  transport, and a vehicle of a type with the invisible flag, are not shown. Part C: an enemy vehicle
  that is cloaked, one that is underground (the special-abilities specs' states), and
  one that has the radar-invisible veterancy ability, each in explored ground. Expected for C
  (hypothesis, low confidence): none is shown. Falsified if: A or B differ; for C, any is shown
  (that records the original's behaviour and revises claim 7's confidence, not the other parts).
- **R10 (claim 8, buildings).** An own building and an enemy building of each of two foundation
  sizes in explored ground, on a large map (scale below 1) and on a small one (scale above 1); an
  own building at the edge of the usable area, so that part of its pixel set lies outside the image.
  Observable: the set of radar pixels with the building's colour. Expected (low confidence): a set
  around the pixel of its top-left cell centre that covers its foundation at the scale. Falsified
  if: the set lies around the foundation's centre pixel, or the pixel of the top-left cell centre
  is not in the set. For the edge building: expected, the pixels of the set outside the image are
  not drawn and none is drawn on the image's edge instead; falsified if the building is drawn on the
  edge.
- **R11 (claim 8, edge clamp).** An own aircraft flies out past the usable area's edge; an enemy
  aircraft likewise. Observable: the radar pixel of the aircraft. Expected: the own aircraft is shown
  on the nearest pixel of the image's edge; the enemy one is not shown outside the usable area.
  Falsified if: the own one vanishes or sits unclamped, or the enemy one is shown.


## Open questions

- Which pixels the mean of F6 rounds either way when it lies within about 10⁻⁶ of an exact half
  (the original sums in floating point and the order of its summation was not fixed here): not
  measurable by design; an implementation may differ there.
- Claim 9, overlay positions (low confidence, scenario R6 part E): table T1 of Edge cases gives the
  original's position rules as the evidence shows them: they cover overlay types that are not
  resources (a resource overlay always uses the cell's own image frame and its colour as stored),
  and the exclusion of position 239 covers every overlay type. They are fixed positions in the
  list order, so which stock types they are is the importer's concern. Whether the bridge case
  also exchanges the green and blue channels is not established. Whether the bridge case of
  claim 9 covers the cells of a low bridge as well as those of a high one is not established either
  (scenario R6 part F).
- Claim 7, cloaking, underground and veterancy-ability tests: for some such objects the original
  lists the object although claim 7's rule hides it, so a further condition, not yet identified,
  also decides whether they are listed. Which objects (which cloak or detection state, which height of an
  underground object, which veterancy level) is not established; scenario R9 part C records it.
- The disguise colour (claim 8): whether it depends on the viewing house, as claim 8 states, or
  applies to every viewer, was not established, and no scenario can create the case (claim 8).
- Whether a bridge cell's colour (claim 9) is taken from image frame 0 of the bridge's sprite, as
  the evidence so far suggests, or from the cell's own image frame (scenario R6 part F).
- The colour of an object without a house (claim 8): the original draws it in the colour of a
  default colour scheme; which scheme is not established, and no skirmish setup creates the case.
- When the original redraws the terrain picture after a cell's terrain colour changes (a tree
  destroyed, ore harvested, a bridge destroyed): it appears to redraw only the changed area; whether
  every such event does so was not established. Claim 5 assumes the picture always follows the cells'
  current state.
- Whether the tile radar colour of a tile cell comes from the image variant the cell shows (claim 9, as
  the evidence so far suggests) rather than the plain image: scenario R6 part D decides; S-0010
  asks the same question.
- The events that request a radar recheck (claim 2) were not established in full; in particular whether
  switching a radar building off or on and the start of an electromagnetic lock request one (our
  engine assumes they do, claim 2). Scenarios R1 and R2 decide.
- The radar gap test for "any gap field once *H* has been defeated" (claim 6) and the channel bit
  widths that the radar colours use (Terms, *radar colour*): scenario R4 parts C and D decide.
- Whether the unusual local areas of claim 4 (leftmost column not 0) are mapped as written: scenario
  R3 part C decides. Whether any trigger action changes the free-radar flag during play (claim 1)
  was not checked.
- Whether a terrain object really colours its cell (claim 9) or the test picks something else:
  scenario R6 part B with a lone tree on clear ground decides.

## Questions for the project lead

1. **Switch for the pixel-to-cell shift (claim 6, Edge cases).** The spec proposes
   `quirks.radar_pixel_cell_shift` (default on, the original's shift). Keep the switch, or state
   that the shift is fixed with no switch, as for the neutral insignificant objects? Note that
   with the switch off an object at a cell's centre is still tested on a neighbouring cell
   ((*x*, *y* + 1), not its own), so the switch does not make object tests exact; widening the
   alternative to cover that is also the lead's call.

The radar departure of claim 1 is decided (lead decision of 2026-10-09, item 4, and 2026-10-11).
