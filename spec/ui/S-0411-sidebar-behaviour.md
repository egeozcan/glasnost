---
id: S-0411
title: Sidebar behaviour (build icons, clicks and the orders they give, icon states, queue counts, ready and placement)
topic: ui
revision: 4
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-12
measurements: []
requests: []
supersedes: []
---

# S-0411: Sidebar behaviour (build icons, clicks and the orders they give, icon states, queue counts, ready and placement)

## Summary

How the local player's build sidebar behaves during a skirmish: when build icons appear and
disappear, which production order each left or right click gives, how an icon shows its state
(unavailable, building, on hold, ready, queue count), the orders the sidebar gives by itself, and
building placement. The state rules for clicks (the busy-line, build-limit and resume rules of claim
4) are also legality rules for every house's production orders (S-0404, claim 10); the offer rule
is S-0407's; tab assignment and icon order are S-0004's (claim 11),
production rules S-0404's, build times S-0403's and the credits display S-0416's.

## Terms

Glossary terms are used as defined there (*sidebar*, *local player*, *house*, *type*, *building*,
*infantry*, *vehicle*, *aircraft*, *cycle*, *drawn frame*, *tick*, *AI house*, *quirk switch*).
From S-0004 (Terms, claim 11): *sidebar tab*. From S-0213: *order*. From S-0403: *item*, *line kind*,
*defence*, *ordinary building*, *progress*, *done*, *step period* *R*. From S-0404: *could make* (and
*could make without the tech-tree rule*), *free dock*, *production line*, *queue*, *player
hold*, *automatic hold*, *waiting for money*, *build order*, *hold order*, *cancel order*, *cancel-all
order*, *place order*. From S-0405: *switched on*, *switched off*. From S-0407: *tech-tree rules*,
*at limit* (a tech-tree answer) and *offered* (S-0407 claim 1: in the house's taken-up set as of the
last recheck, which can lag the sidebar's own test below). In addition:

- **Buildable.** This spec's own test, made by the sidebar (claim 2): a type is *buildable* for a
  house when its tech-tree answer is allowed or at limit and some factory building of the house
  could make it without the tech-tree rule (S-0404, Terms; a factory building that is being sold
  does not count). It differs from *offered* in two ways: it is judged afresh in every tick, while
  the offer changes only at the rechecks of S-0407 (claim 12); and it does not wait for a taken-up
  set. So a type can be offered and no longer buildable, between the start of a sale and the
  recheck at its removal. Legality (claim 4) uses *offered*; the icons (claims 2 and 3) use
  *buildable*.
- **House update, turn pass.** As S-0203 (claim 8) places them: the *turn pass* is the part of a tick
  in which objects take their turns; each house's own *house update* follows it in the same tick and
  comes before the orders of the tick.
- **Tick boundary.** As S-0213 (claim 1): an order given in tick *n* is applied at the end of tick
  *n*, so an order's effect is first seen in tick *n* + 1.

- **Build icon (cameo).** The glossary term, narrowed: a superweapon icon is also an entry of a tab
  (S-0004, claim 11), outside this spec.
- **Recorder.** The glossary term, narrowed: one that also logs every order given, with the tick in
  which it was given and the tick in which it was applied, and the sounds played (used where a
  scenario says so). It still never gives orders.
- **Unit item.** An item of an infantry, vehicle or aircraft type; the other items are building and
  defence items. "Unit" alone, as the kind of an item, icon, type or line ("a done unit", "a unit
  line"), refers to unit items; "the unit" that a done unit item releases is the object it becomes.
- **Ready superweapon.** A superweapon whose charge is complete and that the player may fire. The
  superweapons are outside this spec (S-0004, claim 11 only places their icons); the term is used
  only for the tab flash of claim 11.
- **New-build click.** A left click on an icon whose line has no item, that gives a build order which
  starts an item (claim 4, last row). *Pending* means: its build order has not yet been applied.
  A left click on an icon whose line has an item gives a build order that queues a copy, or none.
- **Icon's line.** The production line of the icon type's kind (S-0404, claim 1).
- **Icon's item.** The line's item when it is of the icon's type.
- **Shift.** The keyboard modifier held during a click.
- **Resume click.** A left click on the icon of a held item, which gives a build order that resumes
  the item (claim 4, row 5).
- **Placement mode.** The input mode in which the mouse carries the outline of a done building and a
  left click on the battlefield gives a place order with that cell.

## Behaviour

1. **Tabs and lines.** S-0004 (claim 11) gives each icon its tab and its position. The tabs feed the
   production lines as follows: the first tab the ordinary-building line; the second tab the defence
   line (and the superweapons); the third tab the infantry line; the fourth tab three lines, the
   ground-vehicle, naval-vehicle and aircraft lines. Icons of one tab that feed different lines can
   be busy at the same time.
   - Backing: `measurable` (scenario S1)
   - Confidence: high

2. **Icons appear and disappear.** An icon is added when its type becomes offered to the local
   player (S-0407, at the recheck of its claim 12; S-0004 claim 11 places it). It is removed when the type stops being buildable: no
   factory building of the house could make it without the tech-tree rule (S-0404, Terms: a factory
   building that is being sold no longer counts; whether a building is switched on, and whether an
   aircraft factory building has a free dock, play no part), or the tech-tree rules refuse it. The
   removal follows buildability (Terms), not the offer. In which tick the removal and its orders
   come, relative to the tick of the loss, is not established (low confidence; scenario S2 records
   it). Until it is measured, our engine takes buildability as it stands at the house update of each
   tick (S-0203, claim 8: after the turn pass of the tick and before the orders), for every player
   house alike, the local player's included: the icon is removed, and its orders are given, in the
   first tick at whose house update the type is no longer buildable, and a type that stops being
   buildable and is buildable again before that point is not removed. For a player house other than
   the local player the orders are the same and no icon is involved. So for a loss in the turn
   pass of tick *n* (a factory building or prerequisite destroyed) the icon goes and the orders are
   given in tick *n*; for a loss that an order brings about (the start of a sale, an order applied
   at the end of tick *n*, tick boundary in Terms) they come in tick *n* + 1. An order the sidebar
   gives is applied at the end of the tick it is given in.

   On the removal the sidebar gives a cancel order when the removed type is the item of its line, and
   a cancel-all order for the type when the house has a production line of the type's kind (S-0404,
   claim 9). When it gives both, a recorder logs the cancel order before the cancel-all order. The
   end state is the same either way: item refunded, every queued copy of the type removed. When the
   tab being shown has no icons left, the sidebar switches to the first tab that has icons.

   Separately, S-0404 (claim 8) cancels an item itself when its last factory building leaves the map
   for good. So a done unit whose last factory building is lost is refunded by one path or the other,
   or earlier by a click (claim 4, rows 1 and 3); no click after the loss gives it a place order.
   The left-click rows that test for a missing factory building (claim 4, rows 1 and 3) are
   therefore reached, for a non-aircraft type, only in the span between the loss of the last factory
   building (the start of the sale, for a sale) and the removal (low confidence; scenario S4 (e),
   (e2) and (e3) probe that span with a click in every tick). In that span the type is no longer
   buildable, the icon is still shown and a click gives the order its row says (claim 4).
   - Backing: `measurable` (scenario S2)
   - Confidence: medium (the removal and its two orders); low (the tick of the recheck, and the span)

3. **Unavailable icons.** An icon is drawn darkened while any of these holds: it is a building or
   defence and its line has an item; no factory building of the house that is switched on could make
   the type (S-0404, Terms); its build limit is reached (S-0407, tech-tree answer *at limit*, the
   only answer that leaves an icon buildable but darkened). The icon that holds its own line's item
   is never darkened, whichever of these holds: it shows its state display instead (claim 8). So
   in a building line the item's icon stays lit while every other icon of that line is darkened, and a held item of a type at its build limit keeps its lit icon. (The cycle of a new-build click
   that is still pending, before the item exists, is not claimed; scenario S3 records it.) The factory building test includes the tech-tree
   condition, but a type the tech-tree rules refuse is removed from the sidebar (claim 2), so such
   an icon is darkened at most until that removal. Dock occupancy plays no part in the darkening: an
   aircraft icon stays lit while every dock is taken, although a build order for it is then ignored
   (S-0404, claim 3). Darkening is only a display: it does not change which order a click gives. What a click on a
   darkened icon does is given by claim 4 alone. A darkened icon never has the line's
   item, so none of claim 4's rows for an icon's own item (rows 1 to 5) applies to it: the causes
   "building line has an item" and "build limit reached" are covered by the two rows that give no
   order (rows 6 and 7). A darkened icon whose cause is the
   missing switched-on factory building gives a build order like any other, and S-0404 (claim 3)
   then ignores it when it is applied, so nothing starts or queues.
   - Backing: `measurable` (scenario S3)
   - Confidence: medium (the three causes; the lit icon of the line's item); low (the cycle of a
     pending new-build click)

4. **Left click.** A left click on an icon gives at most one order, by the first row that applies:

   | Icon state | Order given | Notes |
   |---|---|---|
   | icon's item done, unit, and no factory building of the house could make the type without the tech-tree rule, switched on or not (for an aircraft, counting only buildings with a free dock; quirk below) | cancel order | refund, S-0404 claim 6 |
   | icon's item done, unit, in any other case | place order without a cell | |
   | icon's item done, building, and no factory building of the house could make the type without the tech-tree rule, switched on or not | cancel order | |
   | icon's item done, building, in any other case | none: placement mode starts | |
   | icon's item on a player hold or automatic hold | build order | resumes, S-0404 claim 5 |
   | building or defence whose line has an item (any type, not covered by the rows above) | none | a refusal sound plays (claim 12) |
   | type at its build limit (S-0407), not covered by the rows above | none | no sound is claimed (claim 12) |
   | any other state, including a type no switched-on factory building could make (darkened, claim 3) | build order | starts or queues (S-0404, claim 3) |

   For the darkened case of the last row the order is given but ignored when it is applied (no
   factory building condition met). No sound is claimed for that click (claim 12); scenario S3
   records it. The same holds for the click on a lit aircraft icon while every dock is taken: the
   order is given and ignored (S-0404, claim 3).

   Rows 1 and 3 give the same result as the removal of claim 2 (the item refunded). For a
   non-aircraft type they are reached only in the span described in claim 2. For an aircraft, row 1
   is reached whenever every dock is taken (quirk below).

   No click with a modifier key gives several build orders at once. For a new build (the last three
   rows), which order a left click gives depends only on the state of the icon's line and on whether
   the type is at its build limit. Whether a factory building could make the type, and the tech-tree
   condition, play no part in it; they are judged only when the order is applied.

   *Legality* (S-0404, claim 10 takes its state rule from here; the offer rule is S-0407's, claim
   13). A build order that starts or queues an item is legal when its type is offered, is not at its
   build limit and, for a building or defence, its line has no item. A build order that resumes the
   held item from its own icon (row 5) is exempt from the last two tests: it is legal whatever the
   type's build-limit state, and also for a building or defence whose line has that item; its type
   must still be offered. Both kinds stay legal when no switched-on factory building could make the
   type; S-0404 (claim 3) then ignores the order when it is applied.

   *In the span of claim 2.* The click table looks only at the state of the icon and its line, so a
   click on a shown icon of a type that is no longer buildable gives the order its row says: a build
   order from an idle icon, or a resume from the icon of a held item (row 5). What Legality then
   says depends on the cause of the loss. When the last factory building is being sold, the type is
   still offered (the offer changes only at the recheck of the sale's removal, S-0407 claim 12), so
   the order is legal; no factory building could make the type, so S-0404 (claim 3) ignores it
   when it is applied, and nothing starts, queues or resumes. When the loss is that the tech-tree
   rules now refuse the type, the order is not legal (S-0407, claim 13) and has no effect. Either
   way the line ends as it would without the click, and a recorder logs the order. So the sidebar's
   rules and Legality agree: Legality is the rule for what takes effect, and the sidebar gives
   orders from the icon's state alone.
   - Backing: `measurable` (scenarios S3 and S4; the span by S4 (e3))
   - Confidence: medium (the table); high (one build order per click); low (the orders given in the
     span); medium (no factory building test in the click; the exemption of the resume click from the build-limit and busy-line tests)

5. **Right click.** A right click on an icon gives the order of the row that applies, if any:

   | Icon state | Order given |
   |---|---|
   | icon's item running or waiting for money | hold order |
   | icon's item on hold or done | cancel order; with Shift, cancel-all order |
   | no icon's item, but a copy of the type in its line's queue | cancel order (removes one queued copy, S-0404 claim 6); with Shift, cancel-all order |
   | otherwise | none |

   *Extra hold order.* When the icon received a new-build click in the same cycle and that click is
   still pending, a right click gives a hold order (the rows above give none at that moment, because
   the item does not exist yet). After the tick the item exists and is on a player hold, and a
   recorder logs the build order before the hold order.

   *No claim for other cases.* This spec makes no claim whether a right click in the same cycle as
   (i) a left click that queues a copy (the line has an item of another type, or, for a unit line,
   an item of the icon's own type), or (ii) a left click that resumes a held item, also gives a hold
   order. They are not established. Scenario S5 (third run) records both so that a later revision can
   state them; until then an engine may choose either.

   So a running item is first held and then cancelled by a second right click; queued copies of a
   type are removed before its item (S-0404, claim 6).
   - Backing: `measurable` (scenario S5; the right click on a done unit that waits for release, by
     S7's fourth run; on a done building, by S8's third run)
   - Confidence: medium (the Shift variant); high (hold first, then cancel); medium (the extra hold
     order after a pending new-build click)

6. **Building line exclusivity.** While a building or defence line has an item, the sidebar gives no
   build order that would start or queue an item on it, whatever the item's state (claim 4,
   busy-line row). The one build order it gives while such a line has an item is the resume of a
   held item from that item's own icon (claim 4, row 5); the icon of a done item gives a place order
   or starts placement mode instead. In our engine this is also a legality rule: no house, AI houses
   included, has a starting or queuing build order take effect for a building or defence while that line
   has an item, and the resume of the held item stays legal (claim 4, Legality; S-0404, claim 10).
   - Backing: `measurable` (scenario S4, parts b2 and c)
   - Confidence: high (the refusal while the line has an item); medium (the resume of a held
     building through its own icon)

7. **Queue count.** An icon shows a number while at least one copy of its type is queued in its
   line. The number is the count of its type in its line: the queued copies, plus 1 when the line's
   item is of this type. Otherwise no number is shown; in particular an item of the icon's type with
   no queued copy shows none.
   - Backing: `measurable` (scenario S6)
   - Confidence: medium

8. **State display of the icon with the line's item.**
   - A clock overlay is drawn on the icon while its item is not done; its stage is the item's progress
     (0 to 54) plus 1, so it advances by one stage at every progress step (S-0403, claim 1). When the
     item is done the stage is the last one (55) and the *ready* label shows, in place of the running display.
     Whether stage 55 still covers part of the icon is not settled (low confidence; scenario S6). The
     first stage can already show in the tick of the click, before the build order is applied; the
     shown stage then moves gradually toward progress + 1. This early stage and the movement are
     presentation only and not yet established or predicted: they change nothing in game state and their
     exact timing is not established; scenario S6 records the stage in every drawn frame so that a later revision can state
     them.
   - A *ready* label is shown while the item is done (for buildings until placed; for units only
     while the release waits, S-0404 claim 7).
   - An *on hold* label is shown while the line is on a player or automatic hold and the item is not
     done.
   - A line waiting for money shows neither label; its clock simply stops.
   - Backing: `measurable` (scenario S6)
   - Confidence: medium (labels, clock stage); low (the stage at done, the early first stage and the
     movement)

9. **Automatic release of finished units.** When a unit item of the local player becomes done in
   tick *n*, the sidebar gives a place order without a cell for it (S-0404, claim 7) in tick *n* + 1,
   without any click. The order is applied at the end of tick *n* + 1, so the unit leaves its
   factory building at the end of tick *n* + 1. If the unit is not released in tick *n* + 1, the sidebar does not repeat the
   order. When no factory building of the house could release the unit (S-0404, Terms, *could release*,
   which includes the switched-on test and, for an aircraft, its fallback), the item stays done
   until a left click on the icon (claim 4), even after such a building appears. When a building holds the unit but cannot
   release it yet (for example its exit is blocked), that building's own retry releases it
   (S-0404, claim 7), with no click. For a building item the sidebar instead plays the construction-complete
   announcement and makes the tab flash (claim 11). In our engine every player house, AI houses
   included, gets this automatic place order, issued in tick *n* + 1 for an item done in tick *n*,
   as the local player's is (AI-house ruling).
   - Backing: `measurable` (scenario S7 for units; scenario S8 for the announcement and the tab
     flash of a building item)
   - Confidence: high (automatic release of units); medium (the one-tick delay, the announcement
     and the flash)

10. **Placement.** In placement mode a left click on a battlefield cell gives a place order with that
    cell (S-0404, claim 7). Whether the cell is valid is S-0406. When the order fails, a refusal sound
    plays and placement mode resumes with the same building. Which sound it is, and whether it is the
    one of the refusal row of claim 4, belongs to the sound spec (claim 12; planned S-0701). When it succeeds, placement mode ends.
    A right click on the icon while it is ready gives a cancel order (claim 5), which also ends
    placement mode.
    - Backing: `measurable` (scenario S8, with its third run for the right-click cancel)
    - Confidence: medium

11. **Tab flashing.** While the local player has a done building or defence waiting for placement,
    or a ready superweapon, the tab button that holds it flashes. The flash phase is a function of
    the tick number *t*: the look changes at the ticks whose number is a multiple of 10, so every 10
    ticks (every 10 drawn frames in a single-player game outside frozen cycles), also when the flash
    started in the middle of a period. The first period of a flash that starts at tick *t* lasts
    10 − (*t* mod 10) ticks, which is 10 when *t* is a multiple of 10. Which look (lit or dark)
    belongs to which periods is not established; it is probably fixed by the parity of ⌊*t* / 10⌋
    (low confidence; scenario S8 records it). Presentation only.
    - Backing: `measurable` (scenario S8, with its second run for the superweapon)
    - Confidence: medium (done building, the multiples of 10 and the first period); low (ready
      superweapon; the look of each period)

12. **Sounds.** Clicks that start, queue, resume, hold, cancel or refuse play interface sounds; done
    items play announcements. Which sound plays, and whether the refusals of claims 4 and 10 are one
    sound or two, belongs to the sound spec (planned S-0701). This spec fixes only that a cue plays
    for:
    - a left click whose order starts or queues an item, resumes one or cancels one (claim 4, rows 1,
      3 and 5 and the last row, each only when its order takes effect: not a build order that is
      ignored when it is applied, nor one that is not legal, as in the span of claim 2)
    - the refusal row for a busy building line (claim 4)
    - a right click that holds or cancels (claim 5)
    - the done announcement (claim 9)
    - a refused placement (claim 10).

    It claims no sound for: the place order on a done unit (row 2); the start of placement mode (row
    4); the build-limit row; and a click whose build order is ignored when it is applied (the
    darkened icon, the lit aircraft icon with every dock taken). Scenarios S3 and S4 record these
    without a prediction.
    - Backing: `measurable` (scenarios S3, S4, S5, S7 and S8)
    - Confidence: medium

## Formulas

- **Clock stage** = progress + 1 (claim 8). Integers: progress is S-0403's integer from 0 to 54, so
  the stage runs from 1 to 55. Exact, no rounding.
- **Shown count** = *q* + *i* (claim 7), where *q* is the number of queued copies of the icon's type
  in its line and *i* = 1 when the line's item is of the icon's type, otherwise 0. Integers, exact,
  no rounding. The count is shown exactly when *q* ≥ 1.
- **First flash period** = 10 − (*t* mod 10) ticks (claim 11), where *t* is the tick number in which
  the flash starts. Integers, exact, no rounding: *t* mod 10 is the remainder of *t* divided by 10,
  from 0 to 9, so the first period lasts 1 to 10 ticks.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Tab flash period | 10 | ticks | claim 11: the length of each period in which the look stays the same; the look changes at the ticks whose number is a multiple of 10 |
| Clock stages | 55 | stages | progress 0 … 54 plus 1 |

## State and ordering

Icon display states (for the icon with the line's item):

| Line state | Clock | Label |
|---|---|---|
| running | stage = progress + 1, advancing | none |
| waiting for money | stage frozen | none |
| player or automatic hold | stage frozen | on hold |
| done (building, or unit not yet released) | last stage (55) | ready |

Ordering consequences:
- A click is an input event of S-0213 (claim 1); the order it gives is applied at the end of the tick
  of the click, so the line first reacts in the next tick.
- The automatic place order of a unit done in tick *n* is applied at the end of tick *n* + 1 (claim 9).

## Edge cases and quirks

- **No retry of the automatic release** (`quirks.release_order_once`, default on = original). The
  sidebar sends its automatic place order once, in tick *n* + 1 for an item done in tick *n*
  (claim 9). If no factory building could release the unit at that moment (S-0404, Terms), nothing
  sends the order again: the item stays done until a left click on the icon, even after a building
  that could release it appears. A building that
  holds the unit but cannot release it yet retries by itself (S-0404, claim 7). With the switch off,
  the sidebar sends the place order in every tick in which a unit item of a player house is done and
  not yet released, so the unit leaves at the end of the first tick whose order finds a building
  that could release it.
  Scenario S7 (second run) measures the default.
- **Left click on a done item with no factory building** (claim 4, rows 1 and 3): the click refunds
  the item, as the removal of claim 2 does. It has no switch: for a non-aircraft type it is reached
  only in the span of claim 2 (an engine that removes the icon in the same tick as the loss never
  shows it, and the refund is the same), and for an aircraft it is the quirk below.
- **Left click on a done aircraft with every dock taken cancels it**
  (`quirks.done_aircraft_click_cancels`, default on = original). The test of claim 4's first row
  counts, for an aircraft, only aircraft factory buildings with a free dock, so a click on a done
  aircraft while all docks are taken refunds it, although the place order itself would have used a
  switched-on aircraft factory building without a free dock (S-0404, claim 7). With the switch off, the test
  ignores docks and the click gives the place order.
- **The removal of unbuildable icons cancels production** (claim 2): in the original only for the
  local player; in our engine for every player house (S-0404, claim 9). This is a rule of our
  engine (glossary, *AI house* and *local player*: game state may not depend on the local player),
  not a quirk, so it has no switch.

## Import mapping

None: the sidebar reads no keys of its own. The keys that decide tabs and order are S-0004's, those
that decide buildability S-0407's, and those of the production rules S-0403's and S-0404's.

## Measurement scenarios

Measurement setup (glossary); a small purpose-made skirmish map; one human house (the observed
house) and one bystander computer player, whose void rule applies with the observed house's base as
the observed area; scripted input for exact click ticks; screen and audio capture of the sidebar at
every drawn frame; a recorder (Terms) where a scenario names it. No scenario involves a random draw,
so each part is run once; repeat a part only to confirm a surprising result.

**S1. Tabs and parallel lines.**
- Setup: the house owns every kind of factory building, including a naval and an aircraft factory building, and
  enough credits for every item below.
- Procedure: in quick succession (within a few ticks) start an ordinary building (first tab), a
  defence (second tab), an infantry (third tab), and, in the fourth tab, a ground vehicle, a naval
  vehicle and an aircraft.
- Observable: the six icons' clocks, read by showing each tab in turn at a fixed number of ticks
  after the last start.
- Expected: all six clocks have advanced and keep advancing, none having waited for another (claim 1).
- Falsified if: any two of the six wait for one another (a tab's lines are not independent, or two
  tabs share a line).

**S2. Icons disappearing.**
- Setup: a test unit type that needs a test prerequisite building; an item of it in production and one
  queued; the prerequisite destroyed by test trigger; second run: the last factory building of its kind sold.
- Observable: the icon, the credits, the tab shown; with a recorder, the orders given and the
  tick of the removal counted from the tick of the loss (recorded for a later revision, claim 2).
- Expected: the icon disappears, the item is refunded and the queue cleared; when it was the tab's
  last icon, the sidebar shows the first tab with icons. In the first run the recorder logs a
  cancel order and a cancel-all order, the cancel order first.
- Falsified if: the icon stays, production continues, the item is not refunded, queued copies
  remain, or, when it was the tab's last icon, the sidebar does not show the first tab with icons,
  or, with a recorder (first run), the cancel order and the cancel-all order are not both logged
  with the cancel order first (claim 2).

**S3. Unavailable icons.**
- Setup: a building in production; an aircraft factory building with all docks full; a build-limit-1 type
  already built.
- Observable: whether each icon is darkened (including the icon of the building in production and
  the limited type's icon in a second house state where a held item of that type exists), whether a
  click starts anything, and the sounds played by each click.
- Procedure: click each icon once and record the sounds; for the icon of the building in production
  also in the cycle of its new-build click, drawn frame by drawn frame.
- Second run: the only factory building of a kind switched off with its power switch, S-0405; its icon
  observed and clicked, with a recorder.
- Expected: the icon of the building in production is lit and shows its state display, and the held limited item's icon is lit (claim 3). The other icons of that building's line and the limited icon are darkened and do not start an item; the
  aircraft icon is not darkened, but a click on it neither starts nor queues an aircraft. In the
  second run the icon is darkened and a click starts and queues nothing, with the line unchanged
  (the order is given and ignored, S-0404 claim 3).
- Falsified if: an icon expected darkened is drawn lit, the icon of a line's own item is darkened, any of them starts or queues an item, or the aircraft icon is darkened only because
  the docks are full, or the second-run click gives no build order (seen in the recorder). The
  sounds of the build-limit icon's click, the second run's darkened icon's click and the lit aircraft icon's
  click are recorded without a prediction (claim 12).

**S4. Left clicks.**
- Setup and procedure: left-click (a) an idle infantry icon three times; (b) a held item's icon,
  (b2) the icon of a held building, and in a second house state the icon of a held unit item of a
  build-limit-1 type; (c) a second building icon while a building is in progress, held and ready,
  (d) a ready building icon; (e) a done unit icon after its only factory building has been sold:
  the item is left done by switching the factory building off at completion (as S7, second run), the
  building is then sold, and the icon is clicked by scripted input in every tick from the tick of
  the sell order until the icon is gone or the building has left the map; (e2) the same for a done
  building waiting for placement and the last factory building that could make it; (e3) with a
  recorder, in the same sale as (e) but with the type's line idle (the done item cancelled before
  the sale, a second run with a held item of the type instead), left-click the idle icon, and in
  the second run the held item's icon, in every tick from the tick of the sell order until the icon
  is gone; (f) a done
  aircraft icon while every dock of the house's aircraft factory buildings has been taken during its
  build; (g) in a fresh run for each modifier key, left-click an idle infantry icon once with that key held; (j) a done unit icon
  while a factory building holds the unit but its exit is blocked.
- Observable: line state, queue count, credits, sounds played; in (e) and (e2) also the tick of the
  icon's removal and, with a recorder, the orders given.
- Expected: (a) one item and two queued; (b) resumed, no new copy; (b2) the held building and the
  held limited unit are resumed, with no refusal and no new copy (claim 4, Legality); (c) no item
  starts or queues in any of the three states, and each click plays a refusal sound; (d) placement
  mode; (e) and (e2) the item ends refunded, and no click after the start of the sale gives a place
  order or starts placement mode (rows 1 and 3, or the removal of claim 2); which path gave the
  refund is recorded; (e3) the recorder logs the orders the clicks give in the span (a build order
  from the idle icon, a resume from the held item's icon, or none), recorded without a prediction
  beyond claim 4's table; nothing starts, queues or resumes in either run; (f) refund (the done aircraft is cancelled); (g) at most one copy started or
  queued. Each click that starts, queues, resumes, cancels or is refused plays an interface sound
  (claim 12); the sounds of (d) and (j) are recorded without a prediction.
- Falsified if: any outcome differs, a left click with a modifier key gives more than one build order, a resume in
  (b2) is refused or queues a copy, or in (e) or (e2) the item is still done after the building has
  left the map or a click in the span gives a place order or starts placement mode, or an (e3) click starts,
  queues or resumes an item.

**S5. Right clicks.**
- Setup and procedure: right-click a running item's icon twice; repeat with two queued copies of the
  same type; Shift-right-click a held item with queued copies; right-click an icon whose type is only
  queued. Second run: left-click the icon of a type whose line is free and right-click the same icon
  in the same tick (scripted input). Third run, with a recorder: (i) left-click an
  infantry icon while another infantry type is in production and right-click it in the same tick
  (a queued copy); (ii) left-click the icon of a running unit item of the same type and right-click
  it in the same tick (a queued copy of the item's own type); (iii) left-click a held item's icon (a
  resume) and right-click it in the same tick.
- Observable: line state, queue count, credits, sounds played; in the second and third runs also the
  orders given (recorder) and the line state after the tick.
- Expected: hold, then cancel with refund; queued copies removed first; Shift removes all copies and
  the item; the only-queued icon loses one copy; each hold and cancel click plays an interface sound
  (claim 12). Second run: after the tick the item exists and is on a player hold (clock frozen, *on
  hold* label); the recorder shows a build order and then a hold order. Third run: recorded
  only, no prediction (claim 5 makes no claim for the three cases).
- Falsified if: the first right click cancels a running item, a cancel removes the item before
  the queued copies, a Shift-right click leaves a queued copy or the item, the right click on the
  only-queued icon removes other than one copy, a hold or cancel click plays no interface sound,
  or, in the second run, the item runs on after the tick.

**S6. Queue count and labels.**
- Setup: queue two infantry types A, A, B while A is in production; hold; run out of money.
- Procedure: after the money test, let the item run to completion with enough money.
- Observable: the numbers on every icon, including the icon of a type with no queued copy while its item runs (claim 7), and the labels; the clock stage in every drawn frame, from the
  click tick on; the icon of the done item compared with an idle icon.
- Expected: A shows 3, B shows 1 (only queued); once A's last copy is the item and B is queued, A shows no number; the held line shows the on-hold label; the
  money-starved line shows no label and a frozen clock; the clock advances one stage per step; the
  done item shows the *ready* label and the last stage.
- Falsified if: any number or label differs, no *ready* label shows at done, the stage shown from
  the tick after the one in which the build order was applied differs from progress + 1, or the
  stage shown at done is not 55 (low confidence, claim 8). The frames before that tick are recorded
  for a later revision (claim 8).

**S7. Automatic release.**
- Setup: a vehicle type whose build time gives a step period *R* of 1 (S-0403, claim 1; for example a
  test type with a very low cost field); the recorder's tick counter; second run with the only vehicle
  factory building switched off by its power switch, S-0405, at completion, so that no building
  could release the unit (S-0404, Terms), then switched on again; third run with the only vehicle factory building's exit blocked
  by units at completion, then cleared (as S-0404, scenario P7); fourth run as the second, then a
  right click on the done icon before switching on, with a recorder.
- Observable: the done tick (clock full) and the tick the unit appears outside the factory building; the
  sounds played.
- Expected: the unit appears at the end of the tick after the done tick; in the second run it does not
  appear after switching on until a left click on the icon; in the third run the unit stays done and
  is released by the building's own retry once the exit clears (S-0404, claim 7), without a click. In the fourth run the recorder shows a cancel order and the item is refunded (claim 5).
  The done item plays an announcement (claim 12).
- Falsified if: in the first run the unit appears at the end of any tick other than the tick after the done tick, the sidebar re-sends the
  order by itself in the second run, or the third run needs a click, or the fourth-run right click gives no cancel order or no refund, or the
  second-run unit appears after switching on without a click, or no announcement plays when the
  item is done.

**S8. Placement and flashing.**
- Setup: an ordinary building of low build time that completes during the recording, with the second
  tab shown; the first tab holds its icon. The first run is made twice, with build times chosen so
  that the done ticks differ in their remainder modulo 10. Second run: a house that owns a ready superweapon, the
  tab of its icon not shown. Third run: as the first.
- Procedure: first run: watch the first tab's button from the done tick; open the tab, click the
  icon, click an invalid cell, then a valid one. Third run: click the icon (placement mode), then
  right-click the icon.
- Observable: the tab button's look per drawn frame, with the tick number of each change; the sounds played from the done tick on,
  whether placement mode continues after the invalid click; in the third run whether placement mode
  ends and the credits.
- Expected: the construction-complete announcement plays and the button starts flashing when the item is done (claims 9 and 11; the exact tick is recorded for a later
  revision), with its look changing only at ticks whose number is a multiple of 10 (so
  every 10 ticks, 10 drawn frames) and a first period of 10 − (*t* mod 10) ticks for a done tick
  *t* (claim 11); which look belongs to which period is recorded; the
  invalid click plays a refusal sound and placement mode continues; the valid click places the
  building and ends the mode. Second run: the superweapon's tab button flashes the same way. Third
  run: placement mode ends, the item is cancelled and refunded.
- Falsified if: the look changes at a tick that is not a multiple of 10, the first period differs
  from 10 − (*t* mod 10) ticks, the announcement is missing, placement mode ends after the
  invalid click, the superweapon tab does not flash, or the right click leaves placement mode on.

## Open questions

1. Whether a right click on the battlefield, or a key, also leaves placement mode without cancelling
   is not established.
2. The smoothing of the clock while a build order is pending (claim 8) is only roughly known.
3. Whether the last clock stage (55) of a done item still covers part of the icon (claim 8, S6).
4. Whether the sound of a refused build click (claim 4) and of a refused placement (claim 10) are
   the same (claim 12; planned S-0701).
5. Whether a right click in the same cycle as a queueing or resuming left click gives an extra hold
   order (claim 5, S5 third run).
6. Which sound, if any, the build-limit click, the darkened-icon click and the click on a lit
   aircraft icon with every dock taken play (claims 4 and 12, S3).
7. The tick of the sidebar's recheck of buildability relative to the loss of a factory building or a
   prerequisite, and so the span in which claim 4's rows 1 and 3 can be reached (claim 2, S2, S4).
8. Which look (lit or dark) the tab flash shows in each period (claim 11, S8).

## Questions for the project lead

1. Should the proposed switch `quirks.release_order_once` (the sidebar sends its automatic place
   order once; default on = original) be kept? Without it our engine reproduces the original with no
   way to change it.
