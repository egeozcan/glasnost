---
id: S-0411
title: Sidebar behaviour (build icons, clicks and the orders they give, icon states, queue counts, ready and placement)
topic: ui
revision: 6
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-13
measurements: []
requests: [Q-0121, Q-0123]
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
Network games and campaigns are out of scope.

**Cross-spec changes required at hand-off.** S-0404 revision 3 already supplies the application
and cancellation-timing corrections formerly requested here. Its factory eligibility definition
must also exclude a building queued to be sold, not only one already being sold (claim 14).
S-0407 claim 13 and its state table must distinguish issuing cancellation orders at the recheck
from applying them at the tick's end, and except held-item resume from the issue-time limit test.
Its claim 16 must retain S-0404 claim 10's application-time limit test on queued starts.
S-0407's claim 1/Open question 4 and the glossary's hand-off note must drop their obsolete
request for per-tick-removal corrections: approved S-0411 revision 5 already made them.
S-0407 T14 run 7 must distinguish the refund on permanent factory removal from the later
sidebar cancellation boundary, as claim 14 and S2 do here.
S-0407's taken-up-set definition and claim 1's unconditional admission row must also account
for S-0004 claim 11's full-tab suppression, rather than equating every otherwise eligible type
with a shown icon. Whether a suppressed type is offered for order legality and what happens
when space opens remain unresolved here; no cancellation or refill rule is inferred for it.

## Terms

Glossary terms are used as defined there (*sidebar*, *local player*, *house*, *type*, *building*,
*infantry*, *vehicle*, *aircraft*, *cycle*, *drawn frame*, *tick*, *AI house*, *quirk switch*).
From S-0004 (Terms, claim 11): *sidebar tab*. From S-0213: *order*. From S-0403: *item*, *line kind*,
*defence*, *ordinary building*, *progress*, *done*, *step period* *R*. From S-0404: *could make* (and
*could make without the tech-tree rule*), *free dock*, *production line*, *queue*, *player
hold*, *automatic hold*, *waiting for money*, *build order*, *hold order*, *cancel order*, *cancel-all
order*, *place order*. From S-0405: *switched on*, *switched off*. From S-0407: *tech-tree rules*,
*at limit* (a tech-tree answer) and *offered* (S-0407 claim 1: in the house's taken-up set as of the
last recheck). In addition:

- **Buildable.** A type is *buildable* for a house when its current tech-tree answer is allowed or
  at limit and some factory building of the house could make it without the tech-tree rule
  (S-0404, Terms; a factory building being sold or queued to be sold does not count).
  Being sold and queued to be sold have the meanings of S-0407 Terms. Unlike *offered*, this
  describes current conditions, not membership in the taken-up set. A type can remain offered
  and keep its icon after it stops being buildable, until the next recheck of S-0407 (claim 12).
  A type can also be buildable before it is offered. Buildability alone does not remove an icon
  or give cancellation orders. Legality (claim 4) and icon membership (claim 2) follow the offer.
  Darkening uses the current conditions of claim 3, not the full buildability test.
- **House update, turn pass.** As S-0203 (claim 8) places them: the *turn pass* is the part of a tick
  in which objects take their turns; each house's own *house update* follows it in the same tick and
  comes before the orders of the tick.
- **Tick boundary.** As S-0213 (claim 1): an order given in tick *n* is applied at the end of tick
  *n*. An object first acts on it in tick *n* + 1; the drawn frame after the application boundary
  can already show immediate changes (S-0203 claim 7).

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
  A left click while the line has an item follows claim 4: it may resume, queue, cancel, request
  release, start placement mode or give no order.
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
   player, except that an item becoming available while its tab is full is not shown (S-0004,
   claim 11). This spec does not establish whether or when such an item is shown after a slot
   opens. S-0407 claim 12 owns the recheck timing; S-0004 claim 11 owns capacity and placement.
   A shown icon is removed only
   when a recheck removes the type from the taken-up set. The removal and its cancellation orders
   occur in the house update containing that recheck: the first house update after a listed event
   of S-0407 claim 12, not a separate check in every tick. The recheck considers every offered
   type, including changes left pending by earlier non-events. A type that stops being buildable
   and becomes buildable again before the next recheck keeps its icon.

   When destruction of a prerequisite or factory building in the turn pass of tick *n* makes
   the offered type fail the recheck, the recheck,
   icon removal and orders occur in tick *n*. The start of a sale alone removes no icon and gives
   no cancellation orders. With no intervening recheck event, the icon remains until the first
   house update after the sold building's removal. An intervening event can cause an earlier
   recheck and removal (S-0407 claim 12). In our engine the cancellation orders are given at that
   same recheck for every player house, the local player's included; for another player house no
   icon is involved. An order given by the sidebar is applied at the end of the tick it is given in.

   On the removal the sidebar gives a cancel order when the removed type is the item of its line, and
   a cancel-all order for the type when the house has a production line of the type's kind (S-0404,
   claim 9). When it gives both, a recorder logs the cancel order before the cancel-all order. The
   conditions for giving these orders are judged when the recheck removes the icon, not again
   after an earlier cancellation order has taken effect. Claim 13 fixes the order between types
   and the intermediate starts it can cause. When the
   tab being shown has no icons left, the sidebar switches to the first tab that has icons.

   Separately, S-0404 (claim 8) cancels an item itself when its last factory building leaves the map
   for good. So a done unit whose last factory building is lost is refunded by one path or the other,
   or earlier by a click (claim 4, rows 1 and 3); no click after the loss gives it a place order.
   The left-click rows that test for a missing factory building (claim 4, rows 1 and 3) are
   therefore reached, for a non-aircraft type, only in the span between the last factory building
   ceasing to be able to make the type and the icon's removal at a recheck. For a sale with no
   intervening recheck event, this span begins when selling becomes current or queued, whichever
   happens first, and ends at the recheck after the building's removal (claim 14). In that span the type is no longer buildable, the icon is still shown and a click gives
   the order its row says (claim 4). Scenarios S4 (e), (e2) and (e3) probe reachable click ticks in separate runs.
   - Backing: `measurable` (scenarios S2 and S4 e to e3; S-0407 scenario T14 run 7,
     S-0004 F4 for the full-tab exception) for the local player's behaviour; `unmeasured` for extending automatic orders to every player
     house, a project rule rather than behaviour observable in the original
   - Confidence: high (recheck timing); medium (the removal, its two orders and the click span)

3. **Unavailable icons.** An icon is drawn darkened while any of these holds: it is a building or
   defence and its line has an item; no factory building of the house that is switched on could make
   the type with the full tech-tree condition left out (but with the build-limit test retained,
   including its allowance for an item in production, S-0407 claim 9); its build limit is reached (S-0407, tech-tree answer *at limit*, the
   answer that keeps a type buildable while its limit alone darkens the icon). The icon that holds its own line's item
   is never darkened, whichever of these holds: it shows its state display instead (claim 8). So
   in a building line the item's icon stays lit while every other icon of that line is darkened, and a held item of a type at its build limit keeps its lit icon. (The cycle of a new-build click
   that is still pending, before the item exists, is not claimed; scenario S3 records it.) A stale refusal of a prerequisite or another full tech-tree condition
   does not by itself darken an icon: the offer stays until the next recheck, and its build order
   can still take effect under S-0407 claim 13. S3's third run probes that case. Dock occupancy plays no part in the darkening: an
   aircraft icon stays lit while every dock is taken, although a build order for it is then ignored
   (S-0404, claim 3). Darkening is only a display: it does not change which order a click gives. What a click on a
   darkened icon does is given by claim 4 alone. A darkened icon never has the line's
   item, so none of claim 4's rows for an icon's own item (rows 1 to 5) applies to it: the causes
   "building line has an item" and "build limit reached" are covered by the two rows that give no
   order (rows 6 and 7). A darkened icon whose only cause is the
   missing switched-on factory building gives a build order like any other, and S-0404 (claim 3)
   then ignores it when it is applied, so nothing starts or queues.
   - Backing: `measurable` (scenario S3)
   - Confidence: medium (the three causes, the stale-refusal case and the lit icon of the line's
     item). No prediction is made for a pending new-build click.

4. **Left click.** A left click gives at most one order. The icon's item state determines the response: a qualifying done item is cancelled or placed (or enters placement mode) as described in the done-item rows. Otherwise, a held item is resumed before the busy-line and build-limit cases are considered; a busy building/defence line and a type at its build limit give no order; all remaining cases give the build order shown in the table. The table states the observable conditions and result for each case.

   | Icon state | Order given | Notes |
   |---|---|---|
   | icon's item done, unit, and no factory building of the house could make the type without the tech-tree rule, switched on or not (for an aircraft, counting only buildings with a free dock; quirk below) | cancel order | refund, S-0404 claim 6 |
   | icon's item done, unit, in any other case | place order without a cell | |
   | icon's item done, building, and no factory building of the house could make the type without the tech-tree rule, switched on or not | cancel order | |
   | icon's item done, building, in any other case | none: placement mode starts | |
   | icon's item on a player hold or automatic hold | build order | requests resume, subject to S-0404 claim 3's application tests |
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
   condition, play no part in choosing the click's order. Application checks are S-0404 claim 3's,
   and offer legality is S-0407 claim 13's.

   *Legality* (S-0404, claim 10 takes its state rule from here; the offer rule is S-0407's, claim
   13). A build order that starts or queues an item is legal when its type is offered, is not at its
   build limit and, for a building or defence, its line has no item. A build order that resumes the
   held item from its own icon (row 5) is exempt from the last two tests: it is legal whatever the
   type's build-limit state, and also for a building or defence whose line has that item; its type
   must still be offered. Both kinds stay legal when no switched-on factory building could make the
   type because the factory is missing, selling, queued to be sold or switched off.
   S-0404 (claim 3) then ignores
   the order when it is applied. Exemption from a limit at issue time does not remove the
   build-limit allowance tested with the factory when the order is applied.

   *In the span of claim 2.* The click table looks only at the state of the icon and its line, so a
   click on a shown icon of a type that is no longer buildable gives the order its row says: a build
   order from an idle icon, or a resume from the icon of a held item (row 5). What Legality then
   says depends on the cause of the loss. When the last factory building is being sold, the type is
   still offered until the next recheck (with no intervening event, the sale's removal recheck,
   S-0407 claim 12), so
   the order is legal; no factory building could make the type, so S-0404 (claim 3) ignores it
   when it is applied, and nothing starts, queues or resumes. When a non-event of S-0407 claim 12
   makes the tech-tree rules refuse the type, its offer likewise remains unchanged until a recheck.
   Under S-0407 claim 13, the build order is still legal while the offer remains, provided it passes
   the other legality tests above, and it may take effect under S-0404 claim 3. Once a recheck removes the offer, the icon is gone and no further click
   on it is possible. A recorder logs the orders given during the span. So the sidebar's
   rules and Legality agree: Legality is the rule for what takes effect, and the sidebar gives
   orders from the icon's state alone.
   - Backing: `measurable` (scenarios S3 and S4; the sale span by S4 e3; stale tech-tree refusal
     by S-0407 scenario T11)
   - Confidence: medium (the table and the orders given in the span); high (one build order per
     click); medium (no factory building test in the click; the exemption of the resume click from the build-limit and busy-line tests)

5. **Right click.** A right click on an icon gives the order of the row that applies, if any:

   | Icon state | Order given |
   |---|---|
   | icon's item running or waiting for money | hold order |
   | icon's item on hold or done | cancel order; with Shift, cancel-all order |
   | no icon's item, but a copy of the type in its line's queue | cancel order (removes one queued copy, S-0404 claim 6); with Shift, cancel-all order |
   | otherwise | none |

   *Extra hold order, unreachable through skirmish input.* If a right click could reach an icon
   in the same cycle as its still-pending new-build click, it would give a hold order, leaving the
   new item on player hold after the build order and then the hold order were applied. S-0213
   claim 1 takes at most one input event per cycle, so scripted input cannot arrange those two
   clicks in a skirmish. This conditional rule is `unmeasured`: the permitted input cannot reach
   it, and a recorder cannot make it reachable.

   *No claim for other cases.* This spec makes no claim whether a right click in the same cycle as
   (i) a left click that queues a copy (the line has an item of another type, or, for a unit line,
   an item of the icon's own type), or (ii) a left click that resumes a held item, also gives a hold
   order. They are not established and have the same input restriction as the extra hold order.
   A later revision may state them if a reachable input is found; until then this spec constrains
   neither case.

   So a running item is first held and then cancelled by a second right click; queued copies of a
   type are removed before its item (S-0404, claim 6).
   - Backing: `measurable` (scenario S5 for reachable clicks; the right click on a done unit that
     waits for release, by S7's fourth run; on a done building, by S8's third run); `unmeasured`
     for the conditional extra hold order, for the input restriction explained above
   - Confidence: medium (the Shift variant); high (hold first, then cancel); medium (the extra hold
     order after a pending new-build click)

6. **Building line exclusivity.** While a building or defence line has an item, the sidebar gives no
   build order that would start or queue an item on it, whatever the item's state (claim 4,
   busy-line row). The one build order it gives while such a line has an item is the resume of a
   held item from that item's own icon (claim 4, row 5); the icon of a done building starts
   placement mode or gives a cancel order instead. In our engine this is also a legality rule: no house, AI houses
   included, has a starting or queuing build order take effect for a building or defence while that line
   has an item, and the resume of the held item stays legal (claim 4, Legality; S-0404, claim 10).
   - Backing: `measurable` (scenario S4, parts b2 and c) for sidebar clicks; `unmeasured` for
     extending legality to AI houses, a project rule not tested on the original
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
     display around the new-build click may differ from progress + 1, but no early-stage or
     smoothing rule is asserted here. Scenario S6 records those drawn frames for a later revision.
     They are presentation only and do not change progress. After the item exists, the claimed
     stage is progress + 1, which S6 tests against recorded progress.
   - A *ready* label is shown while the item is done (for buildings until placed; for units only
     while the release waits, S-0404 claim 7).
   - An *on hold* label is shown while the line is on a player or automatic hold and the item is not
     done.
   - A line waiting for money shows neither label; its clock simply stops.
   - Backing: `measurable` (scenario S6)
   - Confidence: medium (labels, clock stage); low (the stage at done). Early-stage timing and
     smoothing remain open, not predicted claims.

9. **Automatic release of finished units.** When a unit item of the local player becomes done in
   tick *n*, the sidebar gives a place order without a cell for it (S-0404, claim 7) in tick *n* + 1,
   without any click. The order is applied at the end of tick *n* + 1; when a factory building
   could release it and its exit is clear, the unit is released at that boundary. If the unit is not released in tick *n* + 1, the sidebar does not repeat the
   order. When no factory building of the house could release the unit (S-0404, Terms, *could release*,
   which includes the switched-on test and, for an aircraft, its fallback), the item stays done
   until a left click on the icon (claim 4), even after such a building appears. A blocked exit
   does not guarantee that the house's item stays done: the refund, retention, hand-off and retry
   conditions remain open under S-0404 claim 7 and Q-0120. For a building item the sidebar instead plays the construction-complete
   announcement and makes the tab flash (claim 11). In our engine every player house, AI houses
   included, gets this automatic place order, issued in tick *n* + 1 for an item done in tick *n*,
   as the local player's is (AI-house ruling).
   - Backing: `measurable` (scenario S7 for units; scenario S8 for the announcement and the tab
     flash of a building item); `unmeasured` for issuing the automatic place order for every
     player house, a project rule not observable in the original
   - Confidence: high (automatic release of units); medium (the one-tick delay, the announcement
     and the flash)

10. **Placement.** In placement mode a left click on a battlefield cell gives a place order with that
    cell (S-0404, claim 7), subject to the order-legality checks of S-0406 claim 6. Placement mode
    keeps S-0406's cursor shroud result, refreshed when the cell under the cursor changes; that
    spec owns its use and the unknown value at mode start. A placement refused by its placement
    tests plays a refusal sound and placement mode resumes with the same building. A place order
    ignored because its factory condition fails instead follows S-0406 claim 1: the item stays
    done and no sound plays. Which sound it is, and whether it is the
    one of the refusal row of claim 4, belongs to the sound spec (claim 12; planned S-0701). When it succeeds, placement mode ends.
    A right click on the icon while it is ready gives a cancel order (claim 5), which also ends
    placement mode.
    - Backing: `measurable` (scenario S8, with its third run for the right-click cancel,
      S-0406 PL1 for ignored placement with no factory, and its claim 6's scenarios for legality)
    - Confidence: medium

11. **Tab flashing.** While the local player has a done building or defence waiting for placement,
    or a ready superweapon, the tab button that holds it flashes. The flash phase is a function of
    the tick number *t*: the look changes at the ticks whose number is a multiple of 10, so every 10
    ticks (every 10 drawn frames in a single-player game outside frozen cycles), also when the flash
    started in the middle of a period. The first period of a flash that starts at tick *t* lasts
    10 − (*t* mod 10) ticks, which is 10 when *t* is a multiple of 10. Which look (lit or dark)
    belongs to which periods is not established; scenario S8 records it without a prediction.
    Presentation only.
    - Backing: `measurable` (scenario S8, with its second run for the superweapon)
    - Confidence: medium (done building, the multiples of 10 and the first period); low (ready
      superweapon). The look of each period remains open.

12. **Sounds.** Clicks that start, queue, resume, hold, cancel or refuse play interface sounds; done
    items play announcements. Which sound plays, and whether the refusals of claims 4 and 10 are one
    sound or two, belongs to the sound spec (planned S-0701). This spec fixes only that a cue plays
    for:
    - a left click whose order starts or queues an item, resumes one or cancels one (claim 4, rows 1,
      3 and 5 and the last row, each only when its order takes effect: not a build order that is
      ignored when it is applied, as in the sale span of claim 2, nor one that is not legal)
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

13. **Several offers removed at one recheck (Q-0123).** For types with shown icons, the
    cancellation orders are given in tab order, first through fourth, irrespective of which tab
    is displayed. No rule is asserted here for a type whose icon was suppressed by a full tab
    (claim 2), including whether its loss of availability gives cancellation orders. Within a tab they
    follow its existing icon order immediately before removal (S-0004 claim 11), including icons
    outside the visible scrolled portion. For each removed type its cancel order, when claim 2
    requires one, precedes its cancel-all order, when required. All orders for that type precede
    those for the next removed type. This is not registration order and not a group of all
    cancels followed by all cancel-all orders. In our engine the same order applies for every
    player house, using the icon order that S-0004 would give that house as the local player.

    An automatic cancellation is not a build order. When applied it can start the next queued
    type under S-0404 claims 6 and 10, even if the same recheck has removed that type's offer.
    Offer membership is not tested again for such a queued start. Factory eligibility and the
    application-time build-limit test still apply. With an item of A, no queued A, and one
    queued B, both losing their offers but keeping eligible factories and limits: if A's icon
    precedes B's, the orders are cancel A, cancel-all A, cancel-all B. B starts at the first
    cancellation and is cancelled by the last. If B's icon precedes A's, cancel-all B removes
    the queued B before cancel A, and no B item starts. The first case takes one creation draw
    for B (S-0404 claim 11); the second takes none. B makes no progress step between orders at
    the same boundary. With no price change between those orders, B starts at the current
    price and is refunded 0 credits: it has paid nothing. A bonus gained before this boundary
    does not by itself change that result. General cancellation refunds remain S-0404 claim 6's.
    - Backing: `measurable` (scenario S9) for local-player orders and intermediate starts,
      `unmeasured` for the every-player extension, a project rule not observable in the original
    - Confidence: high (tab and icon order, per-type grouping); medium (intermediate starts)

14. **Production during a sale (Q-0121).** The start of selling, and merely queueing a sale,
    give no sidebar removal or cancellation orders by themselves. With no other recheck event,
    the existing icon remains until the recheck after the building's removal (claim 2).
    A running item continues to make due progress steps and payments while its last factory
    is being sold or queued to be sold, provided money is available and no independent hold,
    cancellation, completion or factory check intervenes. The sale alone does not put the
    line on hold. The ordinary step period and payment rules remain S-0403's and S-0404's.

    This does not make the selling factory eligible for a new start, a queue request, resume or
    release. A factory building being sold or queued to be sold is excluded from those factory
    tests and from buildability. An intervening recheck can therefore remove the existing icon
    even before the sale begins or ends, and its cancellation orders apply at that tick's end.
    Without such an event, an unfinished item whose last factory building is removed is
    refunded by S-0404 claim 8's permanent-removal rule in the removal tick itself, independently
    of the sidebar orders. Icon removal is still in the first house update after that removal.
    Queue clearing and any remaining cancellation follow the applicable factory check and
    claim 2's orders; an already-refunded item is not refunded a second time.
    - Backing: `measurable` (scenario S2 sale extensions; S4 e3)
    - Confidence: high (no sidebar orders at sale start, recheck timing); medium (continued
      progress, queued-sale factory exclusion and permanent-removal refund)

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
  of the click. Immediate starts, holds, cancellations and refunds can already appear in that
  tick's drawn frame; subsequent progress first becomes possible in the next tick.
- The automatic place order of a unit done in tick *n* is applied at the end of tick *n* + 1 (claim 9).

## Edge cases and quirks

- **No retry of the automatic release** (`quirks.release_order_once`, default on = original). The
  sidebar sends its automatic place order once, in tick *n* + 1 for an item done in tick *n*
  (claim 9). If no factory building could release the unit at that moment (S-0404, Terms), nothing
  sends the order again: the item stays done until a left click on the icon, even after a building
  that could release it appears. Blocked-exit retention and retry remain open under S-0404
  claim 7 and Q-0120. With the switch off,
  the sidebar sends the place order in every tick in which a unit item of a player house is done and
  not yet released, so the unit leaves at the end of the first tick whose order finds a building
  that could release it with an unblocked exit.
  Scenario S7 (second run) measures the default.
- **Left click on a done item with no factory building** (claim 4, rows 1 and 3): the click refunds
  the item, as the removal of claim 2 does. It has no switch: for a non-aircraft type it is reached
  only in the span of claim 2, until a recheck removes the icon, and for an aircraft it is the quirk below.
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

The sidebar reads no keys of its own. These delegated inputs matter to claims 13 and 14:

| Key (file, section) | Concept | Form and absent default |
|---|---|---|
| `Prerequisite` (rules, type) | which offers a prerequisite loss removes | list of building names and generic-group words, empty when absent; S-0407 |
| `Factory` (rules, building type) | factory kind | type kind, none when absent; S-0404 |
| `Owner` (rules, type and factory type) | country overlap in factory eligibility | country-name list, none when absent; S-0404 |
| `Naval` (rules, vehicle and factory type) | matching naval kind | boolean, no when absent; S-0404 |
| `BuildLimit` (rules, type) | whether an intermediate queued start passes its limit test | signed 32-bit integer, 2³¹ − 1 when absent; S-0407 |
| Sidebar ordering keys (S-0004 Import mapping) | cancellation order within a tab uses its existing icon order, not registration order | forms and absent defaults owned by S-0004; no new ordering key |
| No data key | sale-start and queued-sale cancellation timing | behaviour of claims 2 and 14, not an imported delay |

Build times and step payments retain S-0403's and S-0404's import mappings.

## Measurement scenarios

Measurement setup (glossary); a small purpose-made skirmish map; one human house (the observed
house) and one bystander computer player, whose void rule applies with the observed house's base as
the observed area; scripted input for exact click ticks; screen and audio capture of the sidebar at
every drawn frame; a recorder (Terms) where a scenario names it. Game speed 6, resolution
800 × 600, director off and mods off, except the named test map overrides. The test map is
named SidebarProbe and contains the test types, limits, prerequisites and timing overrides each
part needs; its chosen field values are recorded with the measurement. No predicted outcome
depends on random draws, although production starts do take creation draws (S-0404 claim 11).
Each part is run once; repeat a part only to confirm a surprising result. Scripted clicks are
scheduled in distinct cycles, respecting S-0213 claim 1; the recorder's order-issue tick, not
the time the script sent a button event, dates a click.

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
  Third run: use the named map overrides to make the removed type the only offered type in
  the displayed tab, with icons in at least one other tab.
- Procedure: before the first run's destruction, create the prerequisite and record the first
  recheck that offers the test type, with spare room in its tab. Then record the destruction tick.
  In the sale run record both the sale
  start and the building's removal, with no other event of S-0407 claim 12 between them. Void and
  repeat the sale run if another recheck event occurs during that interval. In the third run
  destroy the prerequisite while showing its tab. Use a recorder in all three runs. Fourth run:
  reproduce the stale refinery-group loss of S-0407 T14 run 8, then unload the required vehicle
  before any recheck event and place an unrelated building afterward. Void this run if another
  event intervenes or the vehicle does not restore the prerequisite.
- Observable: the icon, the credits, the tab shown; with a recorder, the orders given and their
  ticks relative to the destruction, sale start and building removal.
- Expected: first run, the icon first appears at the recheck that offers the type (with spare
  room in the tab), then disappears and cancellation orders are given in the destruction
  tick's house update. Sale run, the icon remains throughout the sale until the first house update
  after the building's removal, with no removal orders at sale start. The item is refunded and the
  queue cleared; when it was the tab's
  last icon, the sidebar shows the first tab with icons. In the first run the recorder logs a
  cancel order and a cancel-all order, the cancel order first. In the fourth run the icon
  remains throughout the loss and restoration, and the later recheck gives no cancellation
  orders for that type.
- Falsified if: the newly offered type lacks its icon despite spare tab capacity, the icon
  appears before the offer's recheck, or after the predicted removal boundary the icon stays,
  production continues,
  the item is not refunded, queued copies
  remain, or, when it was the tab's last icon, the sidebar does not show the first tab with icons,
  or, with a recorder (first run), the cancel order and the cancel-all order are not both logged
  with the cancel order first, or the icon or removal orders change at a tick other than the
  predicted recheck tick, or the fourth run removes the icon or cancels production despite
  restored buildability before the recheck (claim 2).

- Sale extensions (claim 14): use a long sale and a slow, unfinished infantry item, with
  enough credits, no cost or power changes and no manual hold. Choose the sale's duration and
  the item's phase so at least two steps fall due before removal and the item cannot complete.
  Record progress, balance owed and actual credits per tick, separate from the animated credit
  display, and the permanent-removal refund tick. Expected: due steps and payments continue
  during the sale, no cancellation at its start, the item refunded in the removal tick by the
  permanent-removal rule, and the icon removed at the following recheck. Falsified if the sale
  alone stops steps, refunds before removal or changes the icon before a recheck.
- Queued-sale run: sell the only factory during an uninterruptible construction state, with
  an item already running. The recorder must confirm that selling is queued but not current
  for at least one due step; otherwise the run is inconclusive. Expected: the icon and item
  remain, that step is paid, and no sidebar cancellation order is issued merely for queueing
  selling. In a separate run place an unrelated building while selling is still queued.
  Expected: its recheck removes the icon and issues cancellation orders, applied at that
  tick's end. Falsified if queueing alone cancels, holds or removes the icon, or the control
  recheck retains the type despite its only factory being queued to be sold. Void any run
  with another recheck event or factory check before the intended observation.

**S3. Unavailable icons.**
- Setup: a building in production; an aircraft factory building with all docks full; a build-limit-1 type
  already built.
- Observable: whether each icon is darkened (including the icon of the building in production and
  the limited type's icon in the held, over-limit infantry state of S4 b2), whether a
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

- Third run: reproduce S-0407 T11 run 2's stale refinery-group refusal, with the test type's line
  idle and a switched-on factory, no reached build limit and no other darkening cause. Record
  its icon and click it before another recheck. Expected: the icon stays lit and the build order
  starts an item. Falsified if: stale prerequisite refusal alone darkens it or the click cannot
  start production (claims 3 and 4).

**S4. Left clicks.**
- Setup and procedure: left-click (a) an idle infantry icon three times; (b) a held item's icon,
  (b2) the icon of a held building, and in a second house state the icon of a held unit item of a
  build-limit-1 infantry type, with an extra on-map infantry of that type supplied by test trigger
  after the item is held so its current answer really is at limit. Also run the held limited
  infantry case without the extra on-map object (the production allowance then applies).
  (c) a second building icon
  while a building is in progress, held and ready,
  (d) a ready building icon; (e) a done unit icon after its only factory building has been sold:
  the item is left done by switching the factory building off at completion (as S7, second run), the
  building is then sold. In separate fresh runs, click the shown icon once at each reachable
  input cycle after sale start until icon removal, dating each by the recorder. Do not click
  in the sell-order cycle. No other recheck event is allowed between sale start and
  removal (void and repeat if one occurs); (e2) the same for a done
  building waiting for placement and the last factory building that could make it; (e3) with a
  recorder, in the same sale as (e) but with the type's line idle (the done item cancelled before
  the sale, a second run with a held item of the type instead), left-click the idle icon, and in
  the second run the held item's icon, once at each reachable cycle after sale start in separate
  fresh runs, until the icon is gone; (f) a done
  aircraft icon while every dock of the house's aircraft factory buildings has been taken during its
  build. Switch those buildings off at the end of the done tick, before the automatic place order,
  and leave them off through the click so the item remains done; (g) in a fresh run for each modifier key, left-click an idle infantry icon once with that key held; (j) a done unit icon
  while its exit is blocked, only if the recorder confirms that the house's item still exists
  and is done (Q-0120 leaves that retention open).
- Observable: line state, queue count, credits, sounds played; a recorder logs orders in every
  part, including no-order refusals and the place order of (j). In (e) to (e3) it also dates
  sale start, the click, building removal and icon removal.
- Expected: (a) one item and two queued; (b) resumed, no new copy; (b2) the held building and the
  held limited unit without the extra on-map object are resumed, with no refusal and no new copy
  (claim 4, Legality). With the extra object the at-limit held infantry icon still gives a build
  order requesting resume, but the application-time limit allowance does not permit it, so the
  item stays held and nothing queues (S-0404 claim 3; S-0407 claim 9); (c) no item
  starts or queues in any of the three states, and each click plays a refusal sound; (d) placement
  mode; (e) and (e2) the item ends refunded, and no click after the start of the sale gives a place
  order or starts placement mode (rows 1 and 3, or the removal of claim 2); which path gave the
  refund is recorded. In a no-click control of parts e and e2, the icon remains until the removal's
  recheck. In (e3), it remains until that recheck too: the recorder logs one build order per
  click on the shown idle icon, or one build order requesting resume per click on the held item's
  icon; nothing starts, queues or resumes in either run. (f) refund (the done aircraft is cancelled); (g) at most one copy started or
  queued. In (j) the click gives a place order without a cell and does not cancel the item.
  Each click whose order starts, queues, resumes or cancels, and each busy-building-line
  refusal in (c), plays an interface sound (claim 12). The sounds of (d), (j) and the ignored
  over-limit resume in (b2) are recorded without a prediction.
- Falsified if: any outcome differs, a left click with a modifier key gives more than one build order, a resume in
  (b2)'s eligible control is refused or queues a copy, the over-limit held infantry icon gives no
  build order or its order resumes or queues an item, or in (e) or (e2) the item is still done after the building has
  left the map or a click in the span gives a place order or starts placement mode, or an (e3) click starts,
  queues or resumes an item, or the no-click controls or (e3) remove the icon before the removal's
  recheck, or an (e3) click on the shown icon gives no build or resume order.

**S5. Right clicks.**
- Setup and procedure: right-click a running item's icon twice; repeat with two queued copies of the
  same type; Shift-right-click a held item with queued copies; right-click an icon whose type is only
  queued. Second run: left-click an idle icon, then right-click it in a later cycle after the
  recorder confirms that the item exists. Also right-click an idle icon with no queued copy,
  and an item waiting for money. The same-cycle two-click cases are not oracle runs: S-0213
  claim 1 makes them unreachable through permitted scripted input (claim 5).
- Observable: line state, queue count, credits, sounds played; in the second run also the orders
  given (recorder) and the line state after each order's application.
- Expected: hold, then cancel with refund; queued copies removed first; Shift removes all copies and
  the item; the only-queued icon loses one copy; each hold and cancel click plays an interface sound
  (claim 12). Second run: the later right click holds the running item; the idle icon gives no
  order, and the waiting-for-money item is put on player hold.
- Falsified if: the first right click cancels a running item, a cancel removes the item before
  the queued copies, a Shift-right click leaves a queued copy or the item, the right click on the
  only-queued icon removes other than one copy, a hold or cancel click plays no interface sound,
  or, in the second run, the item runs on after the hold is applied, the idle icon gives an order,
  or the waiting-for-money item is not held.

**S6. Queue count and labels.**
- Setup: queue two infantry types A, A, B while A is in production; use slow steps so settled
  clock stages can be distinguished. A recorder logs actual progress per tick as well as drawn
  stages. Second run: an unfinished item on automatic hold after its only factory is switched off.
- Procedure: hold A, resume it, exhaust the money and then restore enough to complete the items.
  Record through the last queued A becoming the item, with B still queued. In the second run
  record the label and clock during automatic hold, then switch the factory on. Include a done
  building waiting for placement, so the done stage can be observed for several ticks.
- Observable: the numbers on every icon, including the icon of a type with no queued copy while its item runs (claim 7), and the labels; the clock stage in every drawn frame, from the
  click tick on; the icon of the done item compared with an idle icon.
- Expected: A shows 3, B shows 1 (only queued); once A's last copy is the item and B is queued, A shows no number; the held line shows the on-hold label; the
  money-starved line shows no label and a frozen clock; once an item exists the clock stage equals
  recorded progress + 1; the done item shows the *ready* label and the last stage. The automatic
  hold also shows the on-hold label and a frozen clock, and those end after switch-on.
- Falsified if: any number or label differs, no *ready* label shows at done, the stage shown from
  the first drawn frame in which the item exists differs from recorded progress + 1, or the
  stage shown at done is not 55 (low confidence, claim 8), or the automatic hold lacks its label
  or keeps advancing the clock. The frames before the item exists are recorded
  for a later revision (claim 8).

**S7. Automatic release.**
- Setup: a vehicle type whose build time gives a step period *R* of 1 (S-0403, claim 1; for example a
  test type with a very low cost field); the recorder's tick counter; second run with the only vehicle
  factory building switched off by its power switch, S-0405, by an order applied at the end
  of the done tick (after completion but before the automatic place order), so that no building
  could release the unit (S-0404, Terms), then switched on again; third run with the only vehicle factory building's exit blocked
  by units at completion, then cleared (as S-0404, scenario P7); fourth run as the second, then a
  right click on the done icon before switching on, with a recorder.
- Observable: recorded progress and done tick, automatic place-order issue and application
  ticks, release tick and first drawn frame showing the unit outside the factory building; sounds.
  The recorder distinguishes release at a boundary from later movement out of the exit.
- Expected: in the clear-exit control the automatic order is issued in the tick after the done
  tick and releases the unit at that tick's end; in the second run it does not
  appear after switching on until a left click on the icon. The third run records refund,
  retention, hand-off and the need for another click without a prediction (Q-0120). In the fourth
  run the recorder shows a cancel order and the item is refunded (claim 5).
  The done item plays an announcement (claim 12).
- Falsified if: in the first run the automatic order or release boundary differs from the
  prediction, the sidebar re-sends the
  order by itself in the second run, or the fourth-run right click gives no cancel order or no refund, or the
  second-run unit appears after switching on without a click, or no announcement plays when the
  item is done.

**S8. Placement and flashing.**
- Setup: an ordinary building of low build time that completes during the recording, with the second
  tab shown; the first tab holds its icon. The first run is made twice, with build times chosen so
  that the done ticks differ in their remainder modulo 10. Second run: a house that owns a ready superweapon, the
  tab of its icon not shown. Third run: as the first.
- Procedure: first run: watch the first tab's button from the done tick; open the tab, click the
  icon, click an explored, adjacent cell blocked by an immovable object (legal order, failed
  placement), then a valid cell. Third run: click the icon (placement mode), then
  right-click the icon.
- Observable: the tab button's look per drawn frame, with the tick number of each change; the sounds played from the done tick on,
  whether placement mode continues after the invalid click; in the third run whether placement mode
  ends and the credits.
- Expected: the construction-complete announcement plays and the button starts flashing when the item is done (claims 9 and 11; the exact tick is recorded for a later
  revision), with its look changing only at ticks whose number is a multiple of 10 (so
  every 10 ticks, 10 drawn frames outside frozen cycles) and a first period of
  10 − (*t* mod 10) ticks, where *t* is the recorded flash-start tick, not an assumed done tick
  (claim 11); which look belongs to which period is recorded; the
  invalid click plays a refusal sound and placement mode continues; the valid click places the
  building and ends the mode. Second run: the superweapon's tab button flashes the same way. Third
  run: placement mode ends, the item is cancelled and refunded.
- Falsified if: the look changes at a tick that is not a multiple of 10, the first period differs
  from 10 − (*t* mod 10) ticks, the announcement is missing, placement mode ends after the
  invalid click, valid placement fails to place or end the mode, the superweapon tab does not
  flash, or the right click leaves placement mode on or fails to cancel and refund the item.

**S9. Cancellation order for several types (claim 13).**
- Setup: SidebarProbe overrides define infantry A and B with the same prerequisite and eligible
  factory, generous build limits, unequal tech levels so their icon order is unambiguous and
  opposite their registration order. A is the unfinished item, with no queued A and one queued
  B. Neither item can complete during observation. Other lines have unfinished items and
  queued copies whose types share that prerequisite, spanning the first, second and fourth
  tabs. Use the common speed and settings above, an order/line recorder and a random-state
  recorder. Establish a quiet control interval as S-0404 P11 does.
- Procedure: destroy only the shared prerequisite by a test trigger; do not remove a factory.
  Record from before the destruction through the application of every automatic order. Repeat
  with B preceding A in icon order, keeping A as the item and B queued. Repeat with different
  displayed tabs and scroll positions. In fresh controls gain a cost-bonus building after A
  starts and B is queued, before the prerequisite loss, so B's current price differs from its
  unchanged-data initial price. Record all chosen test fields. Each variant runs three times.
- Observable: issue/application order with type names, each transient item creation/deletion,
  progress, balance owed, credit changes and scenario-stream position. Per-boundary readings
  are required: final line state alone cannot distinguish the two intermediate outcomes.
- Expected: tab order first through fourth and within-tab icon order, complete per-type groups
  with cancel before cancel-all, unaffected by scrolling or the displayed tab. For the infantry
  line, A-before-B creates B and cancels it at the same boundary, with one creation draw and
  no B progress step and a refund of 0 credits. B-before-A creates no B and takes no creation
  draw for B. Its queued removal changes no credits. The cost-bonus control still refunds
  0 for transient B because both its start and its cancel use the same current price. Other line starts contribute the draws that S-0404 specifies
  and are recorded separately. Void a run with an unrelated draw or earlier cancellation.
- Falsified if: orders follow registration rather than icon order, are separated into all
  cancels then all cancel-all orders, or any ordering, intermediate start, refund or draw
  differs despite unchanged factory and limit eligibility. If no transient recording is
  available, order recordings answer the order part only, not the intermediate-start part.

## Open questions

1. Whether a right click on the battlefield, or a key, also leaves placement mode without cancelling
   is not established.
2. Early clock stages and smoothing around a new-build click are not established (claim 8).
3. Whether the last clock stage (55) of a done item still covers part of the icon (claim 8, S6).
4. Whether the sound of a refused build click (claim 4) and of a refused placement (claim 10) are
   the same (claim 12; planned S-0701).
5. Whether a right click in the same cycle as a queueing or resuming left click gives an extra hold
   order (claim 5). S-0213 claim 1 makes these cases unreachable through permitted skirmish input.
6. Which sound, if any, the build-limit click, the darkened-icon click and the click on a lit
   aircraft icon with every dock taken play (claims 4 and 12, S3).
7. Which look (lit or dark) the tab flash shows in each period (claim 11, S8).
8. Q-0120: blocked-exit retention, hand-off, refund and retry rules belong to S-0404 and remain
   open there; S4 j is conditional on a recorded done item and S7's third run makes no exit
   prediction.
9. Whether a type suppressed by a full tab is offered for order legality, whether or when a type suppressed by a full tab gains an icon after a slot opens, and whether
   losing availability while suppressed gives cancellation orders, are not established (claims
   2 and 13). S-0004 F4 tests suppression, not these later outcomes.

## Questions for the project lead

1. Should the proposed switch `quirks.release_order_once` (the sidebar sends its automatic place
   order once; default on = original) be kept? Without it our engine reproduces the original with no
   way to change it.
