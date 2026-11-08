---
id: S-0411
title: Sidebar behaviour (build icons, clicks and the orders they give, icon states, queue counts, ready and placement)
topic: ui
revision: 57
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-08
measurements: []
requests: [Q-0121, Q-0123, Q-0196, Q-0275, Q-0276, Q-0282, Q-0283, Q-0339, Q-2238, Q-2266, Q-2791]
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

**Out of scope — lead decision of 2026-10-14, Interface looks.** Sidebar and shell
positions, sizes, side-dependent layout, clock artwork, label and count fonts and positions,
and the lit tab's artwork are the implementer's design (Q-0276 and Q-0283).
A client that draws the install's tab images instead uses their lit frames (claim 19).
Shared battlefield status messages instead follow S-0213 claim 25 for wrapping,
explicit line breaks and imported-font metrics (Q-0282 and Q-0393); that contract
is separate from sidebar labels. This spec fixes sidebar values, states and timing,
not a reproduction of the original shell. Power values and meaningful
meter scaling belong to S-0405, radar state to S-0410 and credit values to S-0416.
The icon-file content rules of claim 16 answer Q-0275 separately from shell layout.

Drawing the install's command-button and tab images is optional. When the client
draws them, claim 19 fixes the file, frame and palette for each state (Q-2238), and its
*Pixels* part how those images show their pixels on every side (Q-2266).

**Tips (Q-2791).** Claims 20 to 22 give the sidebar's tips: the tip of a build icon (the type's
display name and price, or a superweapon's name, shown at once while the pointer moves over the
strip), its place beside the icon and its look (S-0111's name-tip box), and the tips of the tab,
scroll, repair, sell, options and diplomacy buttons and of the power meter. Where the controls
themselves lie stays the implementer's design (above): a tip's place is given relative to them.

**Paired revisions.** S-0404 revision 13 supplies the application and cancellation-timing
rules, excludes factories being sold or queued to be sold, and cancels a busy or failed unit
release with a refund at the place order's application boundary, without retry (its claims 7
and 13). No eligible releasing factory instead leaves the item done. Claim 14 here covers
production continuing during a sale or queued sale; claim 15 owns full-tab suppression and
later admission. S-0404 relies on those behaviours in its claims 3, 7–9 and 11.
S-0407 revision 13 distinguishes cancellation orders issued
at the recheck from their application at the tick's end, exempts held-item resume from the
issue-time limit test, and retains the application-time limit test on queued starts (claims
13 and 16). Its claim 17 owns simultaneous admission order, which can affect the existing
icon order used by claim 13 below. Its T14 run 7 distinguishes the permanent-removal refund
from the later sidebar cancellation boundary. The former per-tick-removal correction requests are resolved.
Claim 15 answers Q-0196: full-tab suppression excludes a type from the offer, gives no
removal orders for it, and retries admission only at a later recheck. S-0004 claim 11 retains
capacity and sorting ownership. S-0004 claim 11 and Open question 7,
S-0407 Terms, claims 1 and 17 and Open question 4, and S-0404 claim 9 now delegate
full-tab consequences to claim 15. At joint hand-off, confirm these paired passages
preserve delayed refill rather than removing old icons before judging admission capacity.

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
  (S-0004, claim 11), outside this spec except for its tip (claim 20).
- **Recorder.** The glossary term, narrowed: one that also logs every order given, with the tick in
  which it was given and the tick in which it was applied, and the sounds played and announcement
  requests (used where a scenario says so). It still never gives orders.
- **Unit item.** An item of an infantry, vehicle or aircraft type; the other items are building and
  defence items. "Unit" alone, as the kind of an item, icon, type or line ("a done unit", "a unit
  line"), refers to unit items; "the unit" that a done unit item releases is the object it becomes.
- **Ready superweapon.** For the tab flash of claim 11 only, a superweapon eligible under
  S-0512 claim 17: not suspended, inside its flash window, and ready (for a drain-mode
  type, in its ready or active state). Charge completion alone does not establish
  flash eligibility. S-0512 claim 17 owns the window and its reference tick.
  S-0004 claim 11 places the icon.
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
- **Sidebar tip.** The text box that the sidebar shows for the build icon or the sidebar control
  under the pointer (claims 20 to 22). It is drawn like S-0111's name tip and, like it, is
  presentation only: it changes no game state.
- **Slot.** One place of the shown tab's strip at the current scroll position, filled by an icon or
  empty. Only the places that are shown count.
- **Tip area.** The rectangle of the sidebar that sidebar tips are placed in: the sidebar's whole
  width, from the lower edge of its radar panel to the sidebar's bottom edge. In our layout, the
  matching part of our sidebar, which holds the tabs, the strip and their buttons.
- **Insertion point.** A place in a string-table entry's text where a value is written in: a *text
  point* takes a text, a *number point* a whole number (Import mapping gives the entries that have
  them). In the entry's text a text point is marked by a percent sign followed by the letter s, and
  a number point by a percent sign followed by the letter d; the points are filled in the order
  they stand. A number is written in decimal, with a leading minus sign when it is negative and no
  digit grouping.

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
   claim 11). Claim 15 gives suppressed types' offer membership, later admission and lack of
   removal orders. S-0407 claim 12 owns the recheck timing; S-0004 claim 11 owns capacity and placement.
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
   | building or defence whose line has an item (any type, not covered by the rows above) | none | only the unable-to-comply announcement is requested (claim 12) |
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
   - Backing: `measurable` (scenario S5 for reachable clicks; the right click on a done unit with
     no factory building that could release it, by S7's fourth run; on a done building, by S8's third run); `unmeasured`
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
   - The clock's production value is progress + 1, an integer from 1 to 55 (S-0403,
     claim 1). For an unfinished item at settled display stages, this is the shown stage.
     At done the clock overlay is absent and the *ready* label replaces it: stage 55
     is the completed value, not an image that must still cover the icon. The original
     can cosmetically anticipate progress around a new-build click; that animation and
     how a clock value is drawn are outside the look requirements under the lead's
     Interface looks decision. They change neither production progress nor label state.
   - A *ready* label is shown while the item is done (for buildings until placed; for units before a place order releases or cancels the item, or while no factory building
     could release it, S-0404 claim 7).
   - An *on hold* label is shown while the line is on a player or automatic hold and the item is not
     done.
   - A line waiting for money shows neither label; its clock simply stops.
   - Backing: `measurable` (scenario S6)
   - Confidence: high (no clock overlay at done); medium (labels and settled clock value)

9. **Automatic release of finished units.** When a unit item of the local player becomes done in
   tick *n*, the sidebar gives a place order without a cell for it (S-0404, claim 7) in tick *n* + 1,
   without any click. The order is applied at the end of tick *n* + 1; when a factory building
   could release it, the release result follows S-0404 claim 13. A released unit leaves the
   production line at that boundary; a busy or failed release cancels the done item with
   S-0404 claim 6's refund at the same boundary, with no retry. Obstructions need not prevent
   release: a ground vehicle can appear inside its factory before unloading takes it outside.
   No unconditional clear-exit test is added here. The sidebar does not repeat the automatic
   order. When no factory building of the house could release the unit (S-0404, Terms,
   *could release*, which includes the switched-on test and, for an aircraft, its fallback),
   the order does nothing and the item stays done even after such a building appears. A later
   left click gives claim 4's order; a right click can cancel it (claim 5). For a building item the sidebar instead plays the construction-complete
   announcement and makes the tab flash (claim 11). In our engine every player house, AI houses
   included, gets this automatic place order, issued in tick *n* + 1 for an item done in tick *n*,
   as the local player's is (AI-house ruling).
   - Backing: `measurable` (scenario S7 for units, S4 j for a clicked failed release,
     S-0404 P12 for per-kind release outcomes; scenario S8 for the announcement and the tab
     flash of a building item); `unmeasured` for issuing the automatic place order for every
     player house, a project rule not observable in the original
   - Confidence: high (the automatic place order for units); medium (the one-tick delay,
     retention with no eligible factory, cancellation and refund on a busy or failed release,
     the announcement and the flash; S-0404 claims 7 and 13)

10. **Placement.** In placement mode a left click on a battlefield cell gives a place order with that
    cell (S-0404, claim 7), subject to the order-legality checks of S-0406 claim 6. Placement mode
    keeps S-0406's cursor shroud result, refreshed when the cell under the cursor changes; that
    spec owns its use and the unknown value at mode start. A placement refused by its placement
    tests requests only the cannot-deploy-here announcement and placement mode resumes
    with the same building. A place order
    ignored because its factory condition fails instead follows S-0406 claim 1: the item stays
    done and no sound plays. Refused placement requests the cannot-deploy-here announcement,
    not the game-wide refusal sound (claim 12; S-0701 claim 28). When it succeeds, placement mode ends.
    A battlefield right click without a scroll drag instead ends placement mode and removes
    the carried outline, while retaining the completed item ready for placement. It gives no
    place or cancel order and no refund. Clicking the ready icon again re-enters placement
    mode. This differs from a right click on the sidebar icon while it is ready: that gives
    a cancel order (claim 5), which also ends placement mode.

    S-0214 claim 28 owns targeting-mode cancellation: a battlefield right click exits
    repair, sell, power-switch, superweapon targeting, route-editing or beacon-placement
    mode without the corresponding action or an additional ordinary deselection. A
    classified right drag followed by release preserves the active mode, including building
    placement; it neither cancels the completed item nor refunds it. S-0214 claim 27 owns
    the distinction between a scroll drag and a right click.
    - Backing: `measurable` (scenario S8, with its third run for the icon cancellation
      and fourth run for battlefield exit and drag preservation; S-0214 scenario T20 for
      the other targeting modes, and
      S-0406 PL1 for ignored placement with no factory, and its claim 6's scenarios for legality)
    - Confidence: medium

11. **Tab flashing (Q-0283).** While the local player has a done building or defence
    waiting for placement, or a ready superweapon eligible under S-0512 claim 17,
    its tab flashes. A suspended superweapon or one outside its flash window does
    not sustain flashing, even if its charge is complete. Let *t* be the tick
    whose sidebar update starts a new flash, and *r* = *t* mod 10. The initial state is
    lit exactly when ⌊*t*/10⌋ + 1 is even. It lasts for 20 − *r* sidebar updates,
    including the starting update; the first toggle is visible in tick *t* + 19 − *r*.
    Later toggles are separated by 10 ticks. After the first toggle, a tick *u* shows
    lit exactly when ⌊(*u* + 1)/10⌋ is even. Thus the first state spans an extra
    period; it is not simply the parity of the period containing the completion tick.
    This corrects the earlier claim of a first toggle at the next multiple of 10.

    Repeatedly finding a ready item does not restart an existing flash. With no ready
    subject on the tab, flashing stops and its lit flash state clears. These updates
    do not run during frozen cycles or while the game stands still, so pausing does
    not consume the flash period. Lit means the flash state, independent of selection
    or mouse-over styling; its artwork is the implementer's design or the install's
    lit tab frames of claim 19. Presentation only.
    - Backing: `measurable` (scenario S8, including restart and pause controls,
      S-0512 scenario X9 for superweapon flash eligibility)
    - Confidence: high (initial parity and update counts); medium (visible tick boundary
      and ready-superweapon trigger)

12. **Sounds.** The production clicks identified below that start, queue, resume, hold or cancel
    use the game-wide build-click sound (S-0701 claim 24). Announcements follow S-0701 claims
    27 and 28, including nearby-event suppression where applicable: requesting one does not
    guarantee immediate audible playback.
    Starting or resuming infantry requests the training announcement; starting or resuming
    any other producible kind requests the building announcement, including vehicles,
    naval vehicles, aircraft and defences. Adding a queued copy to an already active line
    requests no new training or building announcement, but retains the build-click sound.
    These are announcement requests, not promises of audible playback. S-0701 claim 28
    already states the same split and suppression.
    The busy-building-line refusal requests the unable-to-comply announcement, without a
    build-click or game-wide refusal sound. Refused building placement instead requests the
    cannot-deploy-here announcement, without the game-wide refusal sound. These two refusal
    cues are not the same entry (S-0701 claim 24 and F5).
    This spec fixes a cue only for:
    - a left click whose order starts or queues an item, resumes one or cancels one (claim 4, rows 1,
      3 and 5 and the last row, each only when its order takes effect: not a build order that is
      ignored when it is applied, as in the sale span of claim 2, nor one that is not legal)
    - the refusal row for a busy building line (claim 4)
    - a right click that holds or cancels (claim 5)
    - construction complete for a done building (claim 9), and unit ready when a produced unit
      leaves its factory (S-0701 claim 28), not merely when its item becomes done
    - a refused placement (claim 10).

    It claims no sound for: the place order on a done unit (row 2); the start of placement mode (row
    4); the build-limit row; and a click whose build order is ignored when it is applied (the
    darkened icon, the lit aircraft icon with every dock taken). Scenarios S3 and S4 record these
    without a prediction.
    - Backing: `measurable` (scenarios S3, S4, S5, S7, S8 and S14; S-0701 F5 and F11)
    - Confidence: medium

13. **Several offers removed at one recheck (Q-0123).** For types with shown icons, the
    cancellation orders are given in tab order, first through fourth, irrespective of which tab
    is displayed. A type still suppressed by a full tab gives no removal orders and has no
    position in this ordering (claim 15). Within a tab they
    follow its existing icon order immediately before removal (S-0004 claim 11), including icons
    outside the visible scrolled portion. S-0407 claim 17 owns the admission order when several
    new offers appear together; it can affect that existing icon order without changing the
    cancellation rule here. For each removed type its cancel order, when claim 2
    requires one, precedes its cancel-all order, when required. All orders for that type precede
    those for the next removed type. This is not registration order and not a group of all
    cancels followed by all cancel-all orders. In our engine the same order applies for every
    player house, using the icon order that S-0004 would give that house as the local player.

    Each replacement start receives its scenario-stream creation value at the cancellation
    that causes it, in the application order above (S-0404 claim 11; S-0213 claim 3).
    This is not a separate batch in line creation or payment-priority order. Intervening
    actions keep their draws between starts; no value is reserved ahead of the causing
    action. A replacement started and cancelled at the same boundary has already taken
    its draw; removing it from the queue before it starts takes none. Turn-pass starts
    precede order-caused starts at that tick's end. Across houses, order-boundary starts
    in our engine follow house creation order and each house's production-order issue
    order (S-0213 claim 3). That between-house extension is deterministic design, not
    a finding about the original's computer production.

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
    - Backing: `measurable` (scenario S9; S-0404 P11 B/C for value assignment and turn-pass
      precedence) for local-player orders and intermediate starts; `unmeasured` for the
      every-player and between-house order-boundary extensions, project rules not observable
      in the original
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

15. **Full tabs, offers and later admission (Q-0196).** A suppressed production type is
    absent from the taken-up set: eligibility without an icon is not an offer. In our engine a
    starting, queuing or resuming build order for that type is therefore illegal, for every
    player house, whoever issues it (claim 4; S-0404 claim 10). This is the capacity-aware
    application of the existing project rule that the offer is the house's icon membership,
    not a claim that the original's computer production tests sidebar membership.

    Capacity is judged before the removals of that recheck. An icon still present at the
    start of the recheck occupies a place for admission even if the same recheck will remove
    it. With a full tab, losing a shown offer leaves a vacancy at that recheck but does not
    give a suppressed type that vacancy then, nor in an ordinary later sidebar update.
    At the next recheck, still-eligible suppressed types can gain places, provided a
    qualifying factory of their kind exists (S-0407 Terms). There is no waiting-list priority:
    retrying types compete with other newly eligible types in S-0407 claim 17's admission
    order. Within one kind this is increasing registration index; across kinds sharing a
    tab, the earliest acquired qualifying factory of each kind decides precedence.
    Sorting position under S-0004 claim 11 is not admission priority. A retrying type
    without a qualifying factory stays absent even when the tab has room.

    More generally, the candidates for admission are the types with an allowed or at-limit
    tech-tree answer presented by a qualifying factory. A newly added candidate also
    occupies a place against later candidates in that recheck, even if S-0407 claim 1 (b)'s
    factory-matching condition will remove it before the recheck finishes. Removal does
    not let earlier or later suppressed candidates use that place in the same recheck.
    After admission, shown types that fail claim 1 (a) or (b) are removed under claim 2.
    Thus a tab can finish a recheck below capacity despite eligible suppressed types.
    Rechecks are still caused only by S-0407 claim 12's events: a vacancy alone schedules
    no extra recheck. Existing valid icons are not displaced by a better-sorting candidate.

    A type that remains suppressed and loses availability gives neither a cancel nor a
    cancel-all order. It has no place in claim 13's order because no icon is removed.
    Independent cancellation by a production-line factory check remains S-0404 claim 8's
    rule, not a suppression-removal order. Once a type gains an icon, its later removal
    follows claims 2 and 13 normally.

    In our engine the 76-entry limit is separate for each tab of each player house. Entries
    outside the scrolled view count; a superweapon icon counts in its tab too (S-0004
    claim 11). No place is shared between houses, and observing another house cannot
    change its offers. Every-player admission and cancellation use the ordering that the
    house would have as local player, as in claim 13.
    - Backing: `measurable` (scenario S10) for suppression, retry timing, competing
      admissions and absence of suppressed-type removal orders in the original's local
      sidebar. `unmeasured` for the every-player capacity and order-legality extension:
      it is a project rule, not an observable rule of the original's computer production.
    - Confidence: high (capacity check and no suppressed-type removal orders); medium
      (retry timing and its interaction with admission order); high (project-rule extension)

16. **Icon content, frame and darkening (Q-0275).** S-0007 claims 22 and 25 already
    choose the main or alternate icon file. Whichever file is chosen, the sidebar draws
    image frame 0. Building, unavailable, held and ready states do not select another
    frame of that icon file; clocks and labels are separate displays. The palette source
    is the dedicated icon palette named in Import mapping, not a theater unit palette
    or a house-colour ramp. Selection of the same icon file is therefore independent
    of theater and house colour; side archive precedence can supply different file
    contents (S-0005). Palette loading, session lifetime, colour conversion and missing-file
    behaviour belong to S-0103 claim 13, including its safe missing-palette departure.

    Darkening draws frame 0 of the separate darkening-mask file over the already-drawn
    icon. At each nonzero mask index, each destination hi-colour channel becomes
    ⌊*v*/2⌋, with *v* the integer channel value before the mask; zero mask indices
    leave it unchanged. It does not select a darker icon frame, recolour the icon's
    palette, or paint the mask's palette colours. This operation precedes the label
    and clock displays, so those are not halved with the icon.

    In the original, the icon's whole canvas origin is the slot's top-left, with no
    centring or scaling. The image frame's own displacement within that canvas is
    preserved. The reference content region is 60 × 48 pixels. A smaller canvas does
    not expand to fill it. Pixels at or beyond the region's right or bottom edge are
    clipped, as are pixels outside the sidebar surface. The clipping region begins
    at the surface origin, not at each slot's top-left: a frame with negative
    displacement can spill toward an earlier slot. This describes file-content
    placement, not a required screen position or shell layout.
    - Backing: `measurable` (scenario S11)
    - Confidence: high (frame, mask operation and palette selection); medium (canvas
      anchoring and non-slot-local clipping)

17. **Sidebar labels are not transient status messages (Q-0282).** Starting, queuing,
    resuming, holding and cancelling production do not add a text message to the
    battlefield message list. Neither the busy-building-line refusal nor a
    build-limit click adds one. Their established feedback is claim 12's audio and
    the icon/line state. Waiting for money stops the clock without a text notice
    from the sidebar. Construction completion changes the icon to ready and requests
    its announcement; it adds no battlefield text notice. The ready and on-hold
    labels use the two string-table entries in Import mapping and stay only as long
    as claim 8's states hold: no separate expiry duration or message-list place is
    consumed. Queue counts are decimal integers, not string-table messages.
    S-0213 claim 25 owns the shared battlefield status display's wrapping, explicit
    line breaks and imported-font metrics, as well as its capacity and lifetime.
    Those rules do not apply to sidebar labels or queue counts; their font and
    layout remain design choices, with their values and state-bound presence fixed here.

    Placement failure, sell/repair refusal and route-mode notices are respectively
    S-0406's, S-0408's and S-0213's ownership. This claim does not remove a message
    independently generated by those rules, a trigger or a superweapon. In particular
    no generic timed “refused build” text is to be inferred from an announcement.
    - Backing: `measurable` (scenario S12)
    - Confidence: high (listed sidebar events and label entries)

18. **Scrolling and end feedback (Q-0339).** A scroll-button click requests one page
    in its direction on the displayed tab. A page is the number of complete visible
    icon rows. The original has two icons per row; shell geometry remains a design
    choice. At settled positions let *N* be the tab's icon count, *R* its complete
    visible row count and *T* the zero-based row at the top. The top end is *T* = 0.
    the bottom end is reached when 2 × (*T* + *R*) ≥ *N*. A button request already
    at its requested end requests the game-wide refusal sound. With any room left
    it is accepted and requests the tab-scroll sound instead, even when less than
    a full page remains. Movement stops at the end; reaching that end during the
    accepted movement generates no additional refusal sound. S-0701 claim 24
    already establishes this feedback distinction, not the input dispatch below.

    Both buttons are enabled whenever the tab has more icons than fit, even at
    either end. When all icons fit, both buttons are disabled. Clicking a disabled
    button gives no scroll request and no scroll/refusal sound. Thus the conditional
    refusal for a dispatched request on an all-fitting strip is not a prediction
    of a sound from an ordinary click on its disabled button.

    The button acts on release over the button after a press there, not repeatedly
    while held. Left and right button clicks use the same page response. Holding
    for many cycles adds no requests; releasing outside the button does not activate
    it. There is no held-button repeat delay or interval to import.

    The mouse wheel instead requests one row per wheel event, upward for a positive
    wheel delta and downward for a negative one, irrespective of the delta's magnitude
    or the pointer's location over the battlefield or sidebar. It affects the
    displayed tab and uses the same ends, but requests neither the tab-scroll nor
    the refusal sound, including at an end or when all icons fit. It does not
    activate a disabled page button. These controls change presentation only,
    not production orders or offer membership. The predictions concern normal
    active skirmish play, not wheel events delivered to a modal dialog.
    - Backing: `measurable` (scenario S13; S-0701 F10)
    - Confidence: high (end predicate, partial-page feedback, disabled buttons and
      wheel's one-row silent path); medium (release activation and pointer-independent
      wheel observation)

19. **Command-button and tab images (Q-2238).** The original draws the sell button,
    the repair button, each of the four tab buttons and the two scroll buttons from
    one image file each, named in Import mapping. The tab files are numbered in claim
    1's tab order. Each file is resolved by the ordinary precedence of S-0005 claims 1
    and 7, with the local player's side group mounted as S-0005 claims 13 and 13a
    give. Whether these pictures are stored for the session is left open by S-0005,
    whose Open questions name interface art other than the four palettes of its claim
    22, and is not claimed here. Every one of these images is drawn with the
    dedicated sidebar palette named in Import mapping, the same file name for every
    side: the copy found by the ordinary precedence. S-0005 claim 22 owns that
    palette's discarding and resolution at each side-group mount.

    *Shell images.* The original also draws the sidebar's shell, the art the buttons are
    drawn over, from four image files of fixed name, one each, named in Import mapping and
    resolved by the same precedence as the button files. It draws them with the same sidebar
    palette as the buttons, so that palette's entries give the shell's colours too, and draws
    the buttons over them. Which part of the sidebar each file covers, and where, is not
    claimed: S15 reads where the shell lies beneath a button from run c0d's capture. That
    some shell file lies beneath a button and shows its pixels in the sidebar palette is
    rated high once run c0d shows one checkerboard position in black and white. That there
    are four such files, one each, their names, their resolution and each file's own palette
    stay medium, since S15 gives all four test files the same checkerboard and a position
    does not tell which file lies beneath it. Our engine's shell art is its layout's design
    (*Layout*). The four files enter this spec as
    S15's test inputs, and our engine need not draw them.

    *Pixels (Q-2266).* These images show their pixels as follows, on every side, with
    the sidebar palette above (not the icon palette of S-0103 claim 13). Each rating of high
    in this paragraph holds in each frame that S15 part c's captures show: frames 0, 1 and 2
    of each tab image, frames 0 and 1 of the sell and repair images, and frames 0, 1 and 2
    of each scroll image. Two ratings below are scoped further, per frame and position, and
    hold high only where their scope says. In the lit tab frames 3 and 4, which no capture
    shows, each of these facts is medium, since such a frame shares its image's drawing. A pixel of
    palette index 0 is transparent: it is not
    drawn, so what lies beneath the button shows through (medium), and palette entry
    0's colour plays no part: what an index-0 pixel shows does not depend on entry 0
    (high in a captured frame with an index-0 pixel at a checkerboard position of S15 run
    c0d, medium in any other frame), and it is not one fixed colour (medium; high once S15 run c1d
    shows it).
    Every other index, 1 to 255, shows its converted sidebar-palette colour, packed by
    S-0103 claim 9's 16-bit step: the captured pixel equals that packed colour (high, with
    which 16-bit layout a display path presents medium, as S-0103 claim 9 rates it).
    That such a pixel is drawn opaque, taking nothing from what lies beneath, is the
    statement that no index from 1 to 255 is drawn as a shadow or translucent, and is
    rated as that statement is: a mixing nearer opaque than S15 run c1d's bound can pack
    to the same colour and cannot be told from opaque. No index from 1 to 255 is
    drawn as a shadow or translucent (high only for an index in a captured frame in which S15
    run c1d shows it at two neighbouring checkerboard positions of run c0d, over black and
    white, and only for any
    darkening at an index other than run c1d's black checkerboard entry, since no other
    entry of that run's test palette packs to black, and for any mixing
    that keeps no more of the image's own colour than that run's bound *a*ₘ; medium for that
    index in any other frame, for any other index and for a mixing nearer opaque than the
    bound); index 0 is not
    drawn at all, as above (medium). Neither the house colour nor the lighting the map
    file sets (its ambient level and its red, green and blue tints) changes a button's
    colours (high, for a tint channel only where S15 run c2 records a captured button pixel
    whose packed colour has that component above 0, and for the house colour at an index of
    the remap range only where it records a captured button pixel at that index; medium
    otherwise). The ion storm (S-0104 claim 3) does not change them either
    (medium, high once S15 run c3 shows it, at the indices other than 240 to 254), nor does a
    trigger action that sets a channel of the map tint (S-0104 claim 17) or the base ambient
    (S-0104 claim 14) (medium, high once S15 run c4 shows it, at the indices other than 240 to
    254). At indices 240 to 254 both stay medium: a house-colour shade table drawn at its row 6
    or above keeps those tint-exempt indices at full colour whatever its tint (S-0103 claim 7),
    so neither run could show a tint there. No other global effect (the nuke
    flash, the chrono flash and the psychic dominator, S-0104) changes them (medium).
    A button's state changes only which frame it shows, never the colour of an index
    (high).

    The first row that matches the button's state, read top to bottom, gives the frame.

    | Button | State | Frame |
    |---|---|---|
    | Tab | its tab has no icons (unavailable) | 2 |
    | Tab | lit (claim 11) and shown | 4 |
    | Tab | lit | 3 |
    | Tab | shown | 1 |
    | Tab | any other state | 0 |
    | Sell, repair | its mode active | 1 |
    | Sell, repair | any other state | 0 |
    | Scroll | disabled (claim 18) | 2 |
    | Scroll | its press in progress, with the pointer over it | 1 |
    | Scroll | any other state | 0 |

    The shown tab is the one whose icons the sidebar displays. A lit tab therefore
    alternates between frames 3 and 0, or between 4 and 1 while shown, at claim 11's
    toggles. A tab with no icons shows frame 2 whatever its flash state. A held press
    has no frame of its own on a tab, sell or repair button. Only the scroll buttons
    show it, and a disabled scroll button shows frame 2 even while pressed. A pointer
    resting over a button without a press changes no frame. In the original, a scroll
    button's frame while its press is in progress with the pointer off it is not
    claimed. Our engine shows frame 0 then (design).

    *Mode buttons.* A click on the sell or repair button while its mode is active ends
    that mode, as a click while it is inactive enters it (S-0214 claim 13). Whether a
    click changes either mode while the local player owns no building, or while a
    building placement is in progress, is not claimed here.

    *Mode lighting.* Our engine uses the sell and repair rows above: frame 1 in every
    frame drawn while the mode is active, however it was entered, and frame 0 in every
    other frame. The original differs, a display bug without gameplay effect and so
    without a quirk switch: it shows frame 0 for a mode entered other than by its
    button, and its frame after a mode ends follows the sidebar updates, as below.
    For the original, the first row that matches the mode gives the frame.

    | Sell or repair mode | Original's frame |
    |---|---|
    | active, last entered by a click on its button | 1 in every frame drawn |
    | active, entered another way (for example by its key) | 0 in every frame drawn |
    | inactive | 0 in every frame drawn after the first sidebar update that finishes with the mode inactive |

    A click on the button that ends its mode is handled by such an update, so no frame
    drawn after the click shows frame 1. For a mode ended another way, the frames drawn
    between the ending and that update are not claimed for the original.

    *Power-mode button.* Our engine's power-mode button (S-0214 claim 13) and its
    picture are the implementer's design, and no install image is imported for it.
    Whether the original offers a power-mode button or image is not claimed here.

    *Layout (Q-2238 question 4).* Positions, spacing and button cells stay the
    implementer's design (Summary, Interface looks). The images can be drawn into our
    own cells and carry no placement our engine must reproduce. The image sizes are
    not specified here, so how an image fits its cell is a design choice too. Where
    a file or a needed frame cannot be resolved, our engine draws its own symbol for
    that button (design).
    - Backing: `measurable` (scenario S15 part a for the files, frames and palette by
      opening the install's files; part c run c0d for *Shell images*: a single-pixel
      checkerboard of the test shell images shown through a button's index-0 copy shows some
      shell file drawn beneath that button, and its colours being the black and white of run
      c1d's two changed palette entries shows that file drawn with the sidebar palette; the
      four test files share one checkerboard, so which file it is, and the other files'
      drawing and palette, are not shown; part c for the
      pixels of all eight images on each
      side: index 0 independent of entry 0 (runs c1d and c1b, read at the checkerboard
      positions of run c0d) and not one fixed colour (run c1d, which changes the shell
      images; decisive only where a button's index-0 pixels at those positions show both
      checkerboard colours), every index 1 to 255 shown in its packed colour, the same
      under a second house colour and the map file's lighting (run c2), during an ion
      storm (run c3) and after trigger actions that set the map tint and the base
      ambient (run c4), in frames 0 to 2 of each tab and scroll image and frames 0 and 1
      of the sell and repair images, each pixel of both scroll images' pressed frame
      read in at least one capture; part b for the
      frame per state in play, during a
      held press on every button kind, with the pointer over and off each scroll
      button, for the mode buttons' clicks, each side's images and the original's mode
      lighting, with S8's recorder dating each sidebar update), and `unmeasured` for a
      tab with no icons while lit, which no scenario sets up, and for our engine's mode
      lighting for modes entered other than by their buttons, the scroll frame while the
      pointer is off a pressed button, the power-mode picture, layout and the
      missing-art symbol, which are design decisions, not behaviour of the original.
      The other global effects (the nuke flash, the chrono flash and the psychic
      dominator) are `unmeasured`: no run starts them. A trigger action that sets the map
      tint retints every tintable table, the named palettes included (S-0104 claim 17),
      and an ion storm rebuilds the same tables (S-0104 claim 3). Whether the sidebar
      palette is drawn through such a table is not settled by S-0104, whose named-palette
      backing is `unmeasured` in both claims, so both are medium here: the storm until
      run c3 shows the buttons unchanged during it, the trigger actions until run c4
      shows them unchanged after a map-tint action and after a base-ambient action
      (S-0104 claim 14). What a global effect changes is
      the tint of S-0104 claim 3's tintable tables, every cell's lighting (S-0104 claims
      5 and 9) and the ambient through its fade (S-0104 claim 4). This claim draws every
      button index in its converted sidebar-palette colour, packed and nothing else, so
      a global effect would reach the buttons only if the sidebar palette were one of
      those tables, which runs c3 and c4 test. That a button takes no cell's light is
      this claim's own statement, backed by run c2. Its map-file lighting puts every cell
      at light level 0 with a map tint of 0 in every channel. A drawing shaded through
      S-0104 claim 15's rows of a table of two or more shades then takes row 0, which is
      entirely black, tint-exempt indices (S-0103 claim 7) included (S-0104 claim 15). A
      table of one shade is its palette times its tint, and brightness does not darken it
      (S-0104 claims 6 and 15): built with the map tint it is black. The single-shade tables
      that are never tinted, the single-shade house-colour palettes (S-0104 claim 3), take
      neither brightness nor tint, so a drawing through one does not depend on the lighting
      at all, and the house-colour arm below covers it. A drawing through a table built with
      the map tint loses each channel. No entry of c1's test palette packs to black,
      so a lighting-dependent drawing would change every nonzero-index pixel, and a
      single channel's tint every pixel with that packed component above 0. Its house
      colour's remap ramp packs apart from c1's at every remap-range index other than 31
      (S-0103), so a house-colour drawing would change every button pixel at such an index.
      Index 31 is pure black in every house colour's ramp (S-0103 claim 6), and no entry of
      c1's test palette packs to black, so c1's exact-colour reading already rules out a
      house-colour drawing at index 31. That no other effect
      reaches the buttons by another route than those runs c2 to c4 take is not run,
      hence medium.
      That no index from 1 to 255 is drawn as a shadow or translucent rests first on
      c1's exact-colour reading: every pixel at such an index equals its entry's own
      converted colour, packed. A shadow darkens and a translucent drawing mixes in what
      lies beneath, so either misses that exact colour, except where what lies beneath
      already has the entry's colour closely enough that the result packs the same.
      c1 cannot tell that coincidence apart, since what lies beneath a button is not known
      there. Run c0d supplies it: its button copies hold index 0 only, so each pixel of a
      button's rectangle shows what an index-0 pixel shows there. Where that is the
      single-pixel checkerboard of the test shell images, unbroken around the pixel, it is
      that checkerboard, which lies beneath. Run c1d settles the coincidence at those
      positions: there every index stands at two neighbouring pixels, one over black and
      one over white. Two colours
      that merely differ would not do, since a blend over each of two nearby colours can
      pack to the entry's colour twice. Over black and white, a mixing that keeps a weight
      *a* of the entry's colour gives two results that differ, in every component, by
      (1 − *a*) times the difference beneath. They pack apart once that is large enough,
      so at least one pixel misses the entry's colour for every *a* up to the bound
      *a*ₘ (Formulas). A mixing nearer opaque than *a*ₘ moves a pixel by less than
      (1 − *a*ₘ) × 255 = 16 × 255 / (*D* − 7) levels over any colour beneath. That is
      about two packing steps for the black and white checkerboard. Such a mixing can pack
      to the entry's colour over both checkerboard colours and pass c1 and c1d alike: it
      keeps medium. A
      darkening shadow shows black over black, since a darkening never makes a colour
      lighter. In run c1d only the black checkerboard entry packs to black, so at any other
      index the pixel over black misses its entry's colour. That entry is not judged for
      darkening. A checkerboard that is not black and white raises nothing to high, neither
      for a darkening nor for any mixing. Both readings are made frame by frame: an index
      keeps medium in a frame in
      which it stands at no such pair, and entry-0 independence keeps medium in a frame
      with no index-0 pixel at a checkerboard position. The
      darkening premise is the ordinary meaning of a shadow and is not measured on its own.
      That an index-0 pixel shows exactly what lies beneath stays medium. Run c0d is a
      control, not a view of the region with no button drawn: its copies rest on the same
      transparency. So part c shows that an index-0 pixel shows what an all-index-0 copy
      shows there, does not depend on entry 0 and is not one fixed colour.
    - Confidence: high (files, tab numbering, palette name and file form, resolution by
      the ordinary precedence; the tab frames 0 to 2 in the states S15 part b holds; the
      scroll frames except during a press with the pointer off the button; no frame
      change from a resting pointer; a click on the sell or repair button ending its
      active mode in the states S15 part b holds; the original's first two mode rows for
      a mode entered by its button or by its key, and its last row for a mode ended by
      its button; no frame of its own for a held press on a tab, sell or repair button
      in the states S15 part b holds it); medium (the lit-tab frames 3 and 4 and their
      alternation, at claim 11's toggles, whose timing claim 11 rates medium in
      part); medium (a lit tab with no icons showing frame 2); medium (*Shell images*: some shell
      file drawn beneath a button with the sidebar palette, high once run c0d shows a
      checkerboard position in black and white; the four files, one each, their names and
      resolution, and each file's own palette, medium); medium (the original's last mode
      row for a mode ended other than by its button, and its second row for an entry
      other than by the key); medium (design parts). Pixels, for each of the eight images
      on each of the three sides, in the frames part c captures (frames 0 to 2 of each
      tab and scroll image, frames 0 and 1 of the sell and repair images): high (every
      index 1 to 255 shown in its packed colour, the 16-bit packing, and no change by
      house colour, by the ambient level and tints the map file sets or by button state
      other than the frame, for a tint channel only where run c2 records a captured pixel
      with that packed component above 0 and for the house colour at a remap-range index
      only where it records a captured pixel at that index, medium otherwise); medium for
      no change by an ion storm, high once run c3 shows
      it at the indices other than 240 to 254; medium for no change by a trigger action that
      sets the map tint or the base ambient, high once run c4 shows it at the indices other
      than 240 to 254; medium for both at indices 240 to 254 (tint-exempt in a house-colour
      shade table, S-0103 claim 7); high for an index-0 pixel independent of entry 0
      in a captured frame with an index-0 pixel at a checkerboard position of run c0d,
      medium in any other frame; medium for an index-0 pixel not being one fixed colour, high
      once run c1d shows a button's index-0 pixels at those positions in both
      checkerboard colours; high for no shadow or translucency at an index in a captured
      frame in which run c1d shows it at two neighbouring pixels at those positions, over
      black and white, for any darkening at an index other than run c1d's black checkerboard
      entry, since no other entry of that run's test palette packs to black, and for any mixing
      up to the bound *a*ₘ; medium for that index in any other
      frame and at any other index, where c1's exact colour alone leaves the coincidence
      case open, and medium for a mixing nearer opaque than *a*ₘ; medium for no change by any other
      global effect (the nuke flash, the chrono flash and the psychic dominator), which no
      run starts (S-0104); medium in the lit tab frames 3 and 4, which no capture of part
      c shows and which share their image's drawing; medium for which 16-bit layout a
      display path presents, as S-0103 claim 9 rates it; medium that an index-0 pixel
      shows exactly what lies beneath the button (the transparent drawing the icons
      use, S-0103 claim 13; run c0d's all-index-0 copies rest on that same drawing)

20. **Build-icon tips: when they show and what they say (Q-2791 questions 1, 2, 4 and 6).** While
    the tooltip option of S-0111 claim 20 is on, the sidebar shows a sidebar tip for the build icon
    under the pointer. With the option off, no sidebar tip ever shows.
    - **Over the slots, no wait.** A movement that brings the pointer onto the strip's slots from
      anywhere else hides any tip that shows and starts S-0111 claim 20's ordinary wait of 1000 ms
      of real time. Once one drawn frame has passed with the pointer on a slot, every later movement
      that ends on a slot, of any distance, shows the tip of that slot's icon at once, with no wait.
      A movement within one icon keeps its tip showing, with its text chosen again, and a movement
      onto another icon replaces the tip with that icon's tip at once. A movement that ends on an
      empty slot hides the tip. If the pointer rests from the movement that brought it onto the
      slots, the tip shows when that wait ends. A movement that ends off the slots is S-0111 claim
      20's ordinary movement: it hides the tip and starts the wait, and what shows after the rest
      is the tip of the place where the pointer rests (claim 22 here, or S-0111's name tip on the
      battlefield view).
    - **Lifetime.** A tip shown over the slots hides when its lifetime of 10000 ms of real time
      ends, and the next movement there shows it again at once. The lifetime starts when the tip
      shows: at the end of the wait for a tip that shows after a rest, and at the movement for one
      that a movement shows at once. Unlike S-0111 claim 20's lifetime, every movement over the
      slots that shows a tip, the same tip included, starts the lifetime again. A text chosen again
      in a drawn frame (below), another icon's included, does not start it again.
    - **Buttons and blocks.** Pressing or releasing the left, right or middle mouse button hides the
      tip, as S-0111 claim 20 gives. Over the slots the next movement shows it again at once, so a
      click on an icon hides its tip only until the pointer moves. A held button does not stop a
      movement over the slots from showing the tip at once. Of S-0111 claim 20's blocks only the
      hidden mouse cursor applies to sidebar tips, so an order block, which hides the cursor,
      blocks them too. A moving battlefield view blocks no sidebar tip. The options menu hides a
      sidebar tip and stops its wait as S-0111 claim 20 gives for the name tip.
    - **Text.** For an icon of an infantry, vehicle, aircraft or building type, the text is the
      money-format entry's text (Import mapping) with its text point filled by the type's display
      name (S-0004 claim 17) and its number point by the type's price for the local player's
      house: S-0404 claim 2's *P*, in its form in parts for a building whose cost pays for units
      that come with it. Then every space of the whole text, from the display name and from the
      entry alike, becomes a line feed, so that S-0111 claim 24's layout shows each word on a line
      of its own. For a superweapon icon the text is the superweapon type's display name alone,
      with no price and with its spaces kept. An empty slot has no text, and no tip shows there.
    - **Every state the same.** An icon's state changes neither its text nor whether its tip shows.
      A darkened icon (claim 3), the icon of an item that is building, waiting for money, on hold
      or done, a done building waiting for placement or in placement mode (claim 10), and a
      superweapon icon charging, ready or suspended all show the text above. The price is always
      the type's whole price, not what is left to pay or what has been paid.
    - **Chosen again in every drawn frame.** While a sidebar tip shows, its text is chosen again in
      every drawn frame for the slot or control (claim 22) on which the pointer's last movement
      ended, as S-0111 claim 26 does for the name tip. So the price follows the local player's
      cost factor and bonuses as they stand: gaining or losing a building that grants a cost bonus
      changes the price shown. When a wheel scroll (claim 18) or a change of the tab's icons brings
      another icon into that slot, the tip shows that icon's text, and when the slot becomes empty
      the tip hides, and nothing shows until the pointer moves again (Edge cases for the
      original). A change of an icon's state alone changes nothing.
    - Backing: `measurable` (scenario S16 parts a to f, part c for the held button, part e for the
      price changing under a showing tip, part f for a text chosen again not starting the lifetime
      again; part i for the original's own handling of the movement
      that leaves the slots, Edge cases). `unmeasured` for the price in parts: no S16 building
      type pays for a unit that comes with it, and the tip writes in the same *P* that S-0404's
      scenario P2 measures. `unmeasured` for the order block: the block
      of a game's first ticks lasts 4 ticks, into which no part times a movement over the slots,
      and no part sets up a trigger's block. `unmeasured` for the one drawn frame after the movement
      onto the slots: no part times a second movement within one drawn frame of the first, so S16
      tells "at once after one drawn frame" from "at once" only through the first movement's wait.
      `unmeasured` for the moving battlefield view blocking no sidebar tip: no part scrolls the view
      while a sidebar tip shows. `unmeasured` for the option off: S16 runs with the option on, and
      no part turns it off. `unmeasured` for the options menu: no part opens it while a sidebar tip
      shows or waits. `unmeasured` for the middle button: part c presses only the left and right
      buttons. `unmeasured` for a done item other than a building: no S16 fixture keeps a done unit
      item on the strip, and part d reads a done building only. `unmeasured` for an item waiting
      for money: no S16 fixture empties the observed house's credits. `unmeasured` for the choice in
      every drawn frame, as against a slower cadence: parts e and f read that the price or the text
      changes under a showing tip, but no part names the frame in which the change must appear, as
      S-0111 claim 26 gives for the name tip.
    - Confidence: high for the tip of a unit or building icon, its text with the display name and
      the local player's price for a unit icon and for the icon of a building whose cost field pays
      for no unit that comes with it, the line feeds, the superweapon icon's text, the empty slot
      and the same text in every state that parts c and d read. Medium, `measurable`, for the timing
      over the slots (the ordinary wait for the movement that brings the pointer onto them, no wait
      for a later movement that ends on a slot, the lifetime started again at every movement), the
      left and right buttons, the held button, and the price and the text following a change while
      the tip shows. Medium, `unmeasured` (Backing), for the price in parts of a building whose cost
      field pays for units that come with it, whose pad term S-0404 claim 2 rates medium, the order
      block, the one drawn frame that must pass first, a done item other than a building, an item
      waiting for money, the choice in every drawn frame, the moving battlefield view, the option
      off, the options menu and the middle button.

21. **Build-icon tips: place and look (Q-2791 question 3).** A sidebar tip is laid out, sized and
    drawn as S-0111 claims 24 and 25 give for the name tip: in the game font, with claim 24's line
    breaks and wrapping, in a box *w* = *w*<sub>t</sub> + 8 pixels wide and *h* = *h*<sub>t</sub>
    + 4 pixels high, filled with opaque black and framed one pixel wide, its frame and text in the
    tip colour, the text's origin 4 pixels right of and 2 pixels below the box's top-left corner.
    Its wrap width is *A*<sub>w</sub> − 8, where the tip area (Terms) spans *A*<sub>x</sub> to
    *A*<sub>x</sub> + *A*<sub>w</sub> horizontally and *A*<sub>y</sub> to *A*<sub>y</sub> +
    *A*<sub>h</sub> vertically, in screen pixels, its right and bottom edges exclusive.

    An icon's tip is placed by its slot, not by the pointer. For a slot whose top-left pixel is
    (*x*<sub>s</sub>, *y*<sub>s</sub>), *w*<sub>s</sub> pixels wide and *h*<sub>s</sub> pixels high,
    the box's top-left corner is (*l*, *m*):
    - *l*₀ = *x*<sub>s</sub> + *w*<sub>s</sub> − 5 when *x*<sub>s</sub> < *A*<sub>x</sub> +
      ⌊*A*<sub>w</sub> / 2⌋, and *l*₀ = *x*<sub>s</sub> − *w* + 5 otherwise. Then *l*₁ =
      max(*l*₀, *A*<sub>x</sub>), and *l* = *A*<sub>x</sub> + *A*<sub>w</sub> − *w* when *l*₁ + *w*
      > *A*<sub>x</sub> + *A*<sub>w</sub>, *l* = *l*₁ otherwise.
    - *m*₀ = *y*<sub>s</sub> + *h*<sub>s</sub> − 5 when *y*<sub>s</sub> < *A*<sub>y</sub> +
      ⌊*A*<sub>h</sub> / 2⌋, and *m*₀ = *y*<sub>s</sub> − *h* + 5 otherwise. Then *m* = max(*m*₀,
      *A*<sub>y</sub>).

    All values are integers, and ⌊ ⌋ rounds the quotient down. So the box overlaps the 5 by 5
    pixels of the icon's corner that faces the middle of the tip area and reaches from there toward
    that middle: right of and below an icon in the area's upper left part, left of and above one
    in its lower right part. It keeps its place while the pointer moves within the icon, and is
    placed again for each text chosen again (claim 20). There is no correction at the area's
    bottom edge. A sidebar tip is drawn over the sidebar, on top of everything drawn there, and is
    cut at the sidebar's edges: it never covers the battlefield view.
    - Backing: `measurable` (scenario S16 part g for the place, part h for the look of a control's
      tip). `unmeasured` for the look of an icon's tip, which is drawn by the same rule as a
      control's tip: part h reads the look on control tips only. `unmeasured` for the cut at the
      sidebar's edges: part g reads icons in the top and bottom shown rows, whose boxes reach
      toward the middle of the tip area, and no part brings a box against an edge.
    - Confidence: high for the game font, the black fill, the frame and the one colour of frame
      and text of a control's tip, as S-0111 claim 25 rates them; medium for the same look of an
      icon's tip; medium for the box's margins and the text's origin,
      as S-0111 claims 24 and 25 rate them, the tip colour's side mapping, as S-0111 claim 25
      rates it, the wrap width, the placing by the slot and its corrections, the tip area and the
      cut at the sidebar's edges

22. **Tips of the sidebar's other controls (Q-2791 question 5).** The controls of the table show a
    sidebar tip with its text. Its timing is S-0111 claim 20's (the 1000 ms wait after the last
    movement, the 10000 ms lifetime, a press or release hiding it) with claim 20's blocks here (the
    hidden cursor and the options menu). For the first movement off the build-icon slots this is our
    engine's timing; the original's is in Edge cases. It is placed as S-0111 claim 24 places the
    name tip, by the pointer, with the tip area in place of the battlefield view (the original's
    differences: Edge cases), and is laid out and drawn as claim 21 gives. Its text is chosen again
    in every drawn frame (claim 20). The spaces of these texts are kept: none becomes a line feed.

    | Control | Text |
    |---|---|
    | each of the four tab buttons | its tab's label (Import mapping, numbered in claim 1's tab order); while its tab has no icons (claim 19, frame 2), followed by a line feed, an opening parenthesis, the disabled label and a closing parenthesis |
    | scroll-up button, scroll-down button | the scroll-up label and the scroll-down label; while both buttons are disabled because all of the tab's icons fit (claim 18), followed by the same disabled line |
    | repair button | the repair-mode label |
    | sell button | the sell-mode label |
    | power meter | the power entry's text with its first number point filled by the local player's house output *O* and its second by its house drain *D* (S-0405 Terms) |
    | options button | the options label |
    | diplomacy button | the diplomacy label |

    No other part of the sidebar shows a tip: a rest on the radar panel, the credits counter, the
    shell art, a gap between controls or any control the table does not name shows nothing, and
    S-0111 claim 20 gives no name tip there. The planning-mode (waypoint) button lies on the
    command bar, outside the sidebar (Open question 10). Our engine's power-mode button (claim 19)
    has a tip only if its design gives one. Whether our sidebar has an options or a diplomacy button,
    and where, is the implementer's design (Summary); where it has one, that button shows the
    table's tip.
    - Backing: `measurable` (scenario S16 part h) for the table's controls and for the radar panel,
      the credits counter and a gap between controls, and for the wait. `unmeasured` for any other
      place, since S16 rests on no other part of the sidebar. `unmeasured` for the lifetime, a press
      or release and the blocks over a control: part h reads only the wait, and S-0111's NT
      scenarios read that timing on the name tip.
    - Confidence: high for the controls that have a tip and their texts, the disabled line's form
      and the power meter's two numbers; medium for the disabled line's conditions, the timing (our
      engine's for the first movement off the slots, Edge cases) and the wait, the placing by the
      pointer in the tip area and the absence of tips elsewhere on the sidebar; medium, `unmeasured`
      (Backing), for the lifetime, a press or release and the blocks over a control

## Formulas

- **Clock stage** = progress + 1 (claim 8). Integers: progress is S-0403's integer from 0 to 54, so
  the stage runs from 1 to 55. Exact, no rounding.
- **Shown count** = *q* + *i* (claim 7), where *q* is the number of queued copies of the icon's type
  in its line and *i* = 1 when the line's item is of the icon's type, otherwise 0. Integers, exact,
  no rounding. The count is shown exactly when *q* ≥ 1.
- **Initial flash duration** = 20 − (*t* mod 10) sidebar updates, including the starting
  update (claim 11). First visible toggle tick = *t* + 19 − (*t* mod 10). Integers,
  exact, no rounding; the remainder lies from 0 to 9. Subsequent toggle spacing is 10 ticks.
  Initial lit state: ⌊*t*/10⌋ + 1 even. After the first toggle: ⌊(*u* + 1)/10⌋ even
  at tick *u*. Each floor division rounds down in the non-negative integer domain.

- **Settled scroll end** (claim 18): *E* = max(0, ⌈*N*/2⌉ − *R*) rows.
  *N*, *R* and the top row *T* are non-negative integers; *R* is positive.
  The division rounds upward, counting an incomplete final row. A single accepted
  page from a settled strip finishes at min(*E*, *T* + *R*) downward or
  max(0, *T* − *R*) upward. A wheel event uses 1 row instead of *R*.
  All additions, comparisons and clamping are exact integer operations.
  The end-feedback test uses the starting *T*, not the clamped destination.

- **Mixing bound of S15 run c1d** *a*ₘ = 1 − 16 / (*D* − 7) for *D* > 23 (claim 19), where *D* is
  the smallest difference, over the three 8-bit components, between the two checkerboard colours as
  captured. Real arithmetic, no rounding. The 7 is the most a captured 8-bit component can sit above
  the true value packed into its level, and the 16 is two of S-0103 claim 9's coarsest packing steps
  of 8 levels: one for the two results to pack apart, one for the mixing's own rounding. For black
  and white, *D* is 248 with zero low-bit fill and 255 with copied top bits, so *a*ₘ is 1 − 16/241
  or 1 − 16/248. A mixing keeping a weight *a* ≤ *a*ₘ of the image's colour shows at one pixel of
  each neighbouring pair. The bound is defined only for *D* > 23, where it lies between 0 and 1.
  Claim 19's high uses it for the black and white checkerboard only.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Tab flash period | 10 | ticks | claim 11: spacing after the first toggle |
| Clock stages | 55 | stages | progress 0 … 54 plus 1 |
| Icon-tip overlap | 5 | pixels | claim 21: the box's overlap with the icon's corner, each way |
| Sidebar-tip wrap margin | 8 | pixels | claim 21: wrap width *A*<sub>w</sub> − 8 |

## State and ordering

Icon display states (for the icon with the line's item):

| Line state | Clock | Label |
|---|---|---|
| running | stage = progress + 1, advancing | none |
| waiting for money | stage frozen | none |
| player or automatic hold | stage frozen | on hold |
| done (building, or unit not yet released) | completed value 55, no overlay | ready |

Ordering consequences:
- A click is an input event of S-0213 (claim 1); the order it gives is applied at the end of the tick
  of the click. Immediate starts, holds, cancellations and refunds can already appear in that
  tick's drawn frame; subsequent progress first becomes possible in the next tick.
- The automatic place order of a unit done in tick *n* is applied at the end of tick *n* + 1 (claim 9).

## Edge cases and quirks

- **Extended initial tab-flash state** (`quirks.tab_flash_initial_extra_period`, default
  on = original). Claim 11 uses an extra 10 updates before the first toggle and the
  parity of the next period for its initial lit state. With the switch off, initial
  lit means ⌊*t*/10⌋ even and the first countdown is 10 − (*t* mod 10) updates,
  including the start update, with its toggle in tick *t* + 9 − (*t* mod 10).
  Thereafter both settings use claim 11's 10-tick spacing and subsequent parity.
  This is presentation only, and S8 tests the original setting.

- **Surface-origin icon clipping** (`quirks.sidebar_icon_surface_clip`, default on =
  original). Claim 16 clips right and bottom against the content region but permits
  negative displaced frame content up to the sidebar surface's top and left edges.
  With the switch off, all four edges clip to the icon's own content region, so it
  cannot spill into an earlier slot. Neither setting centres or scales the canvas.
  S11's negative-displacement variant tests the original setting.

- **No retry of the automatic release** (`quirks.release_order_once`, default on = original). The
  sidebar sends its automatic place order once, in tick *n* + 1 for an item done in tick *n*
  (claim 9). If no factory building could release the unit at that moment (S-0404, Terms), nothing
  sends the order again: the item stays done until a left click on the icon, even after a building
  that could release it appears, unless another cancellation ends it. A busy or failed release
  instead cancels the item with a refund at that boundary (S-0404 claims 7 and 13), leaving
  nothing to retry. With the switch off, the sidebar sends the place order in every tick in
  which a unit item of a player house is done. At the first boundary whose order finds an
  eligible factory, the item is released or cancelled with a refund according to S-0404
  claims 7 and 13. This switch changes order repetition, not the factory's release result.
  Scenario S7 (second run) measures the default.
- **Left click on a done item with no factory building** (claim 4, rows 1 and 3): the click refunds
  the item, as the removal of claim 2 does. It has no switch: for a non-aircraft type it is reached
  only in the span of claim 2, until a recheck removes the icon, and for an aircraft it is the quirk below.
- **Left click on a done aircraft with every dock taken cancels it**
  (`quirks.done_aircraft_click_cancels`, default on = original). The test of claim 4's first row
  counts, for an aircraft, only aircraft factory buildings with a free dock, so a click on a done
  aircraft while all docks are taken refunds it, although the place order itself would have used a
  switched-on aircraft factory building without a free dock (S-0404, claims 7 and 13): a type
  that need not return to a dock enters from the local area's edge, whereas a dock-bound type
  is cancelled with a refund. With the switch off, the test ignores docks and the click gives
  the place order, subject to those same release results.
- **The removal of unbuildable icons cancels production** (claim 2): in the original only for the
  local player; in our engine for every player house (S-0404, claim 9). This is a rule of our
  engine (glossary, *AI house* and *local player*: game state may not depend on the local player),
  not a quirk, so it has no switch.
- **Icon tips of a full tab** (display-only, no switch; claim 20). In the original, a tab that
  holds 75 icons, or 76, S-0004 claim 11's cap, shows no icon tip at all. Our engine shows them.
  Backing: `measurable` (scenario S16 part i) for a tab of 76 icons. `unmeasured` for a tab of
  exactly 75: part i uses S-0004 scenario F4's map, whose tab holds 76, and no S16 part fills a
  tab with exactly 75. Confidence: medium.
- **The movement that leaves the slots** (display-only, no switch; claim 20). In the original the
  first movement off the slots is still handled without a wait: the tip of the place where it
  ends, a control's tip or the name tip of an object on the battlefield view, shows at once. Our
  engine handles it as an ordinary movement. Backing: `measurable` (scenario S16 part i) for a
  fast movement from an icon onto a vehicle; `unmeasured` for a movement that ends elsewhere, a
  control or an empty place included, since part i reads no other. Confidence: medium.
- **A slot filled again under a resting pointer** (display-only, no switch; claim 20). In the
  original, when a tip over the slots has hidden because its slot became empty or because the
  mouse cursor was hidden, and the pointer still rests there when the hidden tip's lifetime would
  have ended, the slot's tip is chosen again at that moment and shows if the slot then holds an
  icon and the cursor is shown. A press or release of a mouse button ends that lifetime, so a tip
  it hid does not come back this way. The name tip does the same (S-0111 Edge cases). Our engine
  shows nothing until the pointer moves. Backing: `measurable` (scenario S16 part i) for the
  emptied slot; `unmeasured` for the hidden cursor, since no part hides it while a sidebar tip
  shows. Confidence: medium.
- **Icon-tip text of 66 text units or more** (departure, no switch; claim 20). What the original
  shows for an icon tip whose text, with the display name and the price written in, is 66 text
  units or longer is undefined. Our engine shows it as S-0111's name tip shows a long text
  (S-0111 Edge cases). Backing: `unmeasured`, since the original's result is undefined.
  Confidence: medium.
- **Sidebar-tip boxes placed with a smaller size** (display-only, no switch; claims 21 and 22). The
  original places a sidebar tip with a box 4 pixels narrower and 1 pixel lower than the box it
  draws, and wraps the drawn text at the tip area's width. So an icon tip in the right half of
  the area stands 4 pixels further right, and one in its lower half 1 pixel lower, than claim 21
  gives. These shifts are of icon tips only. A control's tip stands where claim 22 places it,
  except in the cases of S-0111's name-tip Edge cases (the box at the bottom edge, the box wider
  than the view, the two wraps and the box after a smaller text), which the original shows here
  with the tip area in place of the battlefield view. A box that would start left of the sidebar
  is not drawn at all. Our engine places each box with the size it draws and wraps at
  *A*<sub>w</sub> − 8. Backing: `measurable` (scenario S16 part g) for the 4-pixel and 1-pixel
  shifts, and (scenario S16 part h) for a control's tip standing where claim 22 places it.
  `unmeasured` for the rest: no S16 text has a line wider than *A*<sub>w</sub> − 8, so none wraps,
  no part changes a control's tip to a smaller text while it shows, and no part aims a rest at the
  one pointer row of the bottom-edge case. Confidence: medium.

## Import mapping

The sidebar reads no rules keys of its own. Its fixed asset and text inputs are:

| Input | Concept | Form and absent behaviour |
|---|---|---|
| `Cameo`, `AltCameo` (art, type's art section) | icon file choice before frame 0 is drawn | text cut to 24 bytes; main empty/missing uses `XXICON.SHP`, alternate empty/missing has no alternate; first-read retention and use conditions are S-0007 claims 22 and 25 |
| `CAMEO.PAL` | dedicated icon palette | fixed file name, no rules selector; file resolution follows S-0005; startup loading, conversion, session lifetime and missing-file behaviour, including the black-palette warning departure, are S-0103 claim 13's ownership |
| `DARKEN.SHP` | unavailable-icon mask | fixed file name, frame 0, zero index transparent, nonzero indices halve destination channels; no rules selector; a missing mask adds no darkening |
| `TXT_READY` (string table) | ready label | fixed entry identifier, not literal game text; missing entry displays a diagnostic containing the requested identifier; string-table file resolution follows S-0005 |
| `TXT_HOLD` (string table) | on-hold label | fixed entry identifier, not literal game text; missing entry displays a diagnostic containing the requested identifier; string-table file resolution follows S-0005 |
| `TXT_MONEY_FORMAT_2` (string table) | money-format entry: the text of a unit's or building's icon tip (claim 20) | fixed entry identifier, not literal game text; its text holds a text point and then a number point, marked as *Insertion point* (Terms) gives; a missing entry shows S-0004 claim 17's placeholder alone, with no name and no price |
| `TXT_POWER_DRAIN` (string table) | power entry: the power meter's tip (claim 22) | fixed entry identifier, not literal game text; two number points, output then drain, marked as above; missing entry: the placeholder alone |
| `TXT_REPAIR_MODE`, `TXT_SELL_MODE` (string table) | repair-mode and sell-mode labels (claim 22) | fixed entry identifiers, not literal game text; missing entry: the placeholder |
| `Tip:Tab1`, `Tip:Tab2`, `Tip:Tab3`, `Tip:Tab4` (string table) | the first to fourth tab labels, in claim 1's tab order (claim 22) | fixed entry identifiers, not literal game text; missing entry: the placeholder |
| `Tip:ScrollUp`, `Tip:ScrollDown` (string table) | scroll-up and scroll-down labels (claim 22) | as above |
| `Tip:Disabled` (string table) | disabled label (claim 22) | as above |
| `Tip:OptionsButton`, `Tip:DiplomacyButton` (string table) | options and diplomacy labels (claim 22) | as above |
| settings file `[Options]`, `ToolTips` | tooltip option (claims 20 and 22) | S-0111 Import mapping |
| No data key | queue count format and label lifetime | decimal integer count; labels are state-bound, not timed messages; claims 7, 8 and 17 |
| No data key | tab flash cadence | initial parity/countdown and later 10-tick periods are engine constants, claim 11 |
| `FlashSidebarTabFrames` (rules, superweapon) | superweapon flash-window eligibility | integer ticks; initially −1, unlimited; 0 disables the window; positive values limit it from the reference tick; S-0512 claim 17 owns the full rule |
| No data key | scroll dispatch and end tests | claim 18: page button on release, no held repeat, wheel one row per event; disabled buttons emit no request; no imported repeat timer |
| `GUITabSound`, `ScoldSound` (rules, audio-visual section) | accepted page-scroll and refused page-scroll sound respectively | single sound-entry names; initially no sound, later omission or unknown name retains earlier value, including session retention; S-0701 Import mapping |
| `SELL.SHP`, `REPAIR.SHP` | sell and repair button images | fixed file names, no rules selector; frames 0 and 1 by claim 19; resolved by the ordinary precedence of S-0005 claims 1 and 7, with the local player's side group mounted per S-0005 claims 13 and 13a; unresolved file: our engine's own symbol |
| `TAB00.SHP`, `TAB01.SHP`, `TAB02.SHP`, `TAB03.SHP` | first to fourth tab images | fixed file names; the two digits are the tab's zero-based position in claim 1's order; frames 0 to 4 by claim 19's first table; resolution as above |
| `R-UP.SHP`, `R-DN.SHP` | scroll-up and scroll-down button images | fixed file names; frame 0 ready, 1 press in progress over it, 2 disabled (claim 19); resolution as above |
| `SIDEBAR.PAL` | palette of the sell, repair, tab and scroll images above and of the shell images below | fixed file name for every side, no rules selector; discarding and resolution at each side-group mount are S-0005 claim 22's; 768-byte file of 256 entries, each a red, a green and a blue byte; a file byte *c* gives the palette component (4·*c*) mod 256; each high below in each frame S15 part c's captures show (frames 0 to 2 of each tab and scroll image, frames 0 and 1 of the sell and repair images), except where a rating below is scoped per frame and position, medium in the lit tab frames 3 and 4, which no capture shows; index 0 transparent (independent of entry 0: high in a captured frame with an index-0 pixel at a checkerboard position of S15 run c0d, medium in any other frame; not one fixed colour: medium, high once S15 run c1d shows it; showing exactly what lies beneath: medium), every other index shown in its packed colour (high), drawn opaque with no shadow or translucency (high only for an index in a captured frame in which S15 run c1d shows it at two neighbouring checkerboard positions of S15 run c0d, over black and white, for any darkening at an index other than run c1d's black checkerboard entry *p*, which is not judged for darkening, since no other entry of that run's test palette packs to black, and for any mixing up to the bound *a*ₘ; medium for that index in any other frame, for any other index and for a mixing nearer opaque), colours packed by S-0103 claim 9 (high; the 16-bit layout a display path presents: medium, as rated there), unchanged by house colour and by the lighting the map file sets (high, for a tint channel only where S15 run c2 records a captured button pixel whose packed colour has that component above 0 and for the house colour at a remap-range index only where it records a captured button pixel at that index; medium otherwise), by the ion storm (medium, high once S15 run c3 shows it, at indices other than 240 to 254), by a trigger action that sets the map tint or the base ambient (medium, high once S15 run c4 shows it, at indices other than 240 to 254) and by any other global effect (medium) (claim 19 *Pixels*); unresolved file: our engine's own symbols (claim 19) |
| `SIDE1.SHP`, `SIDE2.SHP`, `SIDE3.SHP`, `ADDON.SHP` | the sidebar's shell images (claim 19 *Shell images*) | fixed file names, resolved as the button files; drawn beneath the buttons with the sidebar palette above (some shell file beneath a button in that palette: medium, high once S15 run c0d shows a checkerboard position in black and white; the four files, one each, and each file's own palette: medium, since the four test files share one checkerboard); what each draws and where is not claimed; S15 runs c1d, c0d and c1b replace them as test inputs; our engine need not draw them |
| No data key | power-mode button image | no install image is imported; our picture is design (claim 19) |
| `EVA_Training`, `EVA_Building` (announcement data, entry names) | infantry start/resume versus other-kind start/resume | fixed entry identifiers, resolved by S-0701's announcement rules; missing entry supplies no announcement; an active-line queue click requests neither |

The money-format and power entries named above, and what a missing money-format or power entry
shows, are `unmeasured`: no S16 part replaces or removes an entry, so a run cannot tell the named
entry from another entry with the same text, or show a missing entry's result. What a missing
entry of the repair-mode, sell-mode, tab, scroll, disabled, options or diplomacy rows shows (the
placeholder), and the identity of those rows' entries, are `unmeasured` for the same reasons: no
S16 part replaces or removes one, and part h reads only the text shown. Confidence: medium.

These delegated inputs matter to claims 13 and 14:

| Key (file, section) | Concept | Form and absent default |
|---|---|---|
| `Prerequisite` (rules, type) | which offers a prerequisite loss removes | list of building names and generic-group words, empty when absent; S-0407 |
| `Factory` (rules, building type) | factory kind | type kind, none when absent; S-0404 |
| `Owner` (rules, type and factory type) | country overlap in factory eligibility | country-name list, none when absent; S-0404 |
| `Naval` (rules, vehicle and factory type) | matching naval kind | boolean, no when absent; S-0404 |
| `BuildLimit` (rules, type) | whether an intermediate queued start passes its limit test | signed 32-bit integer, 2³¹ − 1 when absent; S-0407 |
| Sidebar ordering keys (S-0004 Import mapping) | cancellation order within a tab uses its existing icon order, not registration order | forms and absent defaults owned by S-0004; no new ordering key |
| No data key | sale-start and queued-sale cancellation timing | behaviour of claims 2 and 14, not an imported delay |
| No data key | per-house tab capacity and delayed refill | 76 entries per tab, an engine constant, not an imported limit or delay; claim 15 |
| `BuildingTypes`, `InfantryTypes`, `VehicleTypes`, `AircraftTypes` (rules or map, section names) | same-kind priority for admission or retry into a vacancy | ordered type-name entries, keys do not determine the order; missing section adds no types; S-0004 claims 2 to 4 |
| `TechLevel` (rules, type) | whether a candidate passes the tech-tree answer at a retry | signed integer, initially −1; S-0407 Import mapping; capacity does not override this answer |

Build times and step payments retain S-0403's and S-0404's import mappings.
The game-wide build-click sound uses `GUIBuildSound` (rules, audio-visual section), a sound
name whose absent-value retention and resolution follow S-0701 Import mapping. Announcement
entries and voice sets retain S-0701's import mapping.

## Measurement scenarios

Measurement setup (glossary); a small purpose-made skirmish map; one human house (the observed
house) and one bystander computer player, whose void rule applies with the observed house's base as
the observed area; scripted input for exact click ticks; screen and audio capture of the sidebar at
every drawn frame; a recorder (Terms) where a scenario names it. Game speed 6, resolution
800 × 600, no engine add-on and no mods (the named test files of a scenario excepted), except the named test map overrides. The test map is
named “sidebar probe map” and contains the test types, limits, prerequisites and timing overrides each
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
  and leave them off through the click so the item remains done; (g) in a fresh run for each modifier key, left-click an idle infantry icon once with that key held; (j) a done infantry icon with no queued copies. Keep its item done by switching its only
  factory off at the end of the done tick, as in S7's second run. Block every candidate exit
  cell of S-0404 claim 13, then switch the factory on and left-click the icon in a later
  cycle. The recorder must confirm that the done item still exists before the click and
  that the factory remains eligible; void a run where either condition fails.
- Observable: line state, queue count, credits, sounds played; a recorder logs orders in every
  part, including no-order refusals and the place order of (j). In (e) to (e3) it also dates
  sale start, the click, building removal and icon removal.
- Expected: (a) one item and two queued; (b) resumed, no new copy; (b2) the held building and the
  held limited unit without the extra on-map object are resumed, with no refusal and no new copy
  (claim 4, Legality). With the extra object the at-limit held infantry icon still gives a build
  order requesting resume, but the application-time limit allowance does not permit it, so the
  item stays held and nothing queues (S-0404 claim 3; S-0407 claim 9); (c) no item
  starts or queues in any of the three states, and each click requests only the
  unable-to-comply announcement; (d) placement
  mode; (e) and (e2) the item ends refunded, and no click after the start of the sale gives a place
  order or starts placement mode (rows 1 and 3, or the removal of claim 2); which path gave the
  refund is recorded. In a no-click control of parts e and e2, the icon remains until the removal's
  recheck. In (e3), it remains until that recheck too: the recorder logs one build order per
  click on the shown idle icon, or one build order requesting resume per click on the held item's
  icon; nothing starts, queues or resumes in either run. (f) refund (the done aircraft is cancelled); (g) at most one copy started or
  queued. In (j) the click gives a place order without a cell, not a cancel order. Its failed release
  cancels the item with a refund when that place order is applied (S-0404 claims 7 and 13).
  Each click whose order starts, queues, resumes or cancels plays the game-wide build-click
  sound (claim 12). Each busy-building-line refusal in (c) requests the unable-to-comply
  announcement, without that click sound or the game-wide refusal sound. Record the requested
  entry as well as audible playback to distinguish announcement suppression from no request.
  The sounds of (d), (j) and the ignored
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
  money-starved line shows no label and a frozen clock; settled clock stages equal
  recorded progress + 1; the done item shows the *ready* label with no clock overlay. The automatic
  hold also shows the on-hold label and a frozen clock, and those end after switch-on.
- Falsified if: any number or label differs, no *ready* label shows at done, a settled
  stage differs from recorded progress + 1, a clock overlay remains at done, or the
  automatic hold lacks its label or keeps advancing the clock. Record early cosmetic
  anticipation separately; it is not a prediction of required clock artwork (claim 8).

**S7. Automatic release.**
- Setup: a vehicle type whose build time gives a step period *R* of 1 (S-0403, claim 1; for example a
  test type with a very low cost field); the recorder's tick counter; second run with the only vehicle
  factory building switched off by its power switch, S-0405, by an order applied at the end
  of the done tick (after completion but before the automatic place order), so that no building
  could release the unit (S-0404, Terms), then switched on again; third run with an infantry
  item, no queued copies and an eligible switched-on infantry factory whose every candidate
  exit cell is blocked at completion (S-0404 claim 13 and P12 d), then cleared after the
  automatic order is applied. Also repeat S-0404 P12 b's busy ground-factory setup, with no
  other same-type factory to receive the release and no queued copies; fourth run as the second, then a
  right click on the done icon before switching on, with a recorder.
- Observable: recorded progress and done tick, automatic place-order issue and application
  ticks, release tick and first drawn frame showing the unit outside the factory building; sounds.
  The recorder distinguishes release at a boundary from later movement out of the exit.
- Expected: in the clear-exit control the automatic order is issued in the tick after the done
  tick and releases the unit at that tick's end; in the second run it does not
  appear after switching on until a left click on the icon. In the third run a failed
  infantry release cancels the item with a refund at the automatic order's application
  boundary; the busy ground-factory variant does the same. With unchanged price since the
  start, the refund equals the credits paid. Clearing the exit later releases nothing and
  causes no retry: the item no longer exists. In the fourth
  run the recorder shows a cancel order and the item is refunded (claim 5).
  With no recent nearby unit-produced event to suppress it, the successful unit release
  requests the unit-ready announcement (S-0701 claim 28 and F7).
  Merely becoming done does not request it, including the retained-item second run.
  Record announcement requests separately from audible playback.
- Falsified if: in the first run the automatic order or release boundary differs from the
  prediction, the sidebar re-sends the
  order by itself in the second run, or the fourth-run right click gives no cancel order or no refund, or the
  second-run unit appears after switching on without a click, or becoming done without release
  requests unit ready, or successful release with no suppressing nearby event fails to request
  it, or in the third run or its busy variant the done item survives the failed
  or busy release, the refund is missing or dated to a later boundary, or clearing the exit
  produces the cancelled unit or a retry order. A ground vehicle that has already appeared
  inside a factory and is waiting to move outside is not a retained production item.

**S8. Placement and flashing.**
- Setup: an ordinary building of low build time that completes during the recording, with the second
  tab shown; the first tab holds its icon. The first run is made twice, with build times chosen so
  that the done ticks differ in their remainder modulo 10. Second run: a house that owns a ready superweapon, the
  tab of its icon not shown. Third run: as the first.
- Procedure: first run: watch the first tab's button from the done tick; open the tab, click the
  icon, click an explored, adjacent cell blocked by an immovable object (legal order, failed
  placement), then a valid cell. Third run: click the icon (placement mode), then
  right-click the icon. Fourth run: enter placement mode and right-click explored battlefield
  ground without dragging; click the ready icon again, then make a classified right scroll
  drag under S-0214 claim 27 and release. Finally place the retained building on a valid cell.
  Repeat this fourth run three times, with scripted input and a recorder.
- Observable: the tab button's look per drawn frame, with the tick number of each change; the sounds played from the done tick on,
  whether placement mode continues after the invalid click; in the third run whether placement mode
  ends and the credits. In the fourth run record the outline, ready item, production orders,
  credits delta, tab flashing and successful re-entry and later placement.
- Expected: the construction-complete announcement plays and the button starts flashing when the item is done (claims 9 and 11; the exact tick is recorded for a later
  revision). With *t* the recorded flash-start tick, not an assumed done tick, the
  initial lit state is ⌊*t*/10⌋ + 1 even. Its first toggle is in tick
  *t* + 19 − (*t* mod 10); subsequent toggles are 10 ticks apart, and the lit state
  follows claim 11's subsequent parity. The
  invalid click requests the cannot-deploy-here announcement, without the game-wide refusal
  sound, and placement mode continues; record the requested entry separately from playback.
  The valid click places the
  building and ends the mode. Second run: while the superweapon is eligible under
  S-0512 claim 17, its tab button flashes the same way. Third
  run: placement mode ends, the item is cancelled and refunded. Fourth run: the battlefield
  right click removes the outline but leaves the item ready and the tab flashing, with no
  production order or credits change. Re-entry restores the outline; the scroll drag and
  release retain placement mode and the ready item, with no cancellation or refund. The
  final valid left click places the building normally.
- Superweapon eligibility controls: use S-0512 scenario X9's named flash-window
  overrides, with no done defence or other eligible superweapon on the tab. In
  separate runs, switch the sole giving building of a ready power-dependent type
  off and on, once before a positive window expires and once after its expiry.
  Record suspension, resumption and flash-state ticks. Expected: an unlimited window sustains flashing,
  a zero window never starts it, and a positive window ceases to sustain it at
  the expiry tick fixed by S-0512 claim 17. Suspension stops the flash; resumption
  restores eligibility only if the window still permits it. A flash sustained by
  an ineligible record, or failure to flash when it alone qualifies, falsifies
  this control. Drain-mode tests use S-0512 scenario X11's named test data and
  the reference-tick expectations of S-0512 claim 17.
- Pause/restart controls: open the options menu during the initial state, then resume.
  Expected: neither countdown nor lit state advances while the game stands still.
  Keep a ready subject on the tab for at least 50 ticks: repeated readiness must not
  restart the countdown. Place the last ready building, then complete another with
  a different start remainder. Expected: the old flash clears and the new flash
  uses its new start tick. Repeat once for each start remainder from 0 through 9,
  and with adjacent even/odd period numbers. The recorder dates the sidebar update
  and its drawn frame; record any differing one-tick visible phase separately.
- Falsified if: initial parity, the first toggle tick or subsequent 10-tick spacing
  differs, the countdown advances during pause or restarts while readiness persists,
  the flash stays after its last subject is removed, the announcement is missing, placement mode ends after the
  invalid click, that click requests a different announcement or the game-wide refusal sound,
  valid placement fails to place or end the mode, the superweapon tab does not
  flash while eligible, or the third-run icon right click leaves placement mode on or fails to cancel and
  refund the item. The fourth run is falsified by battlefield exit cancelling/refunding the
  item, failure to re-enter and place it, or a classified right drag ending placement mode.

**S9. Cancellation order for several types (claim 13).**
- Setup: “sidebar probe map” overrides define infantry A and B with the same prerequisite and eligible
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
  0 for transient B because both its start and its cancel use the same current price.
  Across the other lines, replacements receive their creation values in the recorded
  cancellation-application order, not line creation or payment-priority order (S-0404
  P11 B). Any intervening action's draws remain between those starts. A transient item
  keeps its consumed draw even when a later order cancels it. Other release-specific
  draws remain S-0404's and are recorded separately. Void a run with an unrelated draw or earlier cancellation.
- Falsified if: orders follow registration rather than icon order, are separated into all
  cancels then all cancel-all orders, or any ordering, intermediate start, refund or draw
  differs despite unchanged factory and limit eligibility. If no transient recording is
  available, order recordings answer the order part only, not the intermediate-start part.

**S10. Full-tab retries and suppressed losses (claim 15).**
- Setup: the sidebar probe map supplies exactly 76 initially admitted ordinary-building
  types, including shown type A with a dedicated prerequisite, and two further building
  types B and C with another prerequisite. Give the house a qualifying construction yard,
  ample credits and generous limits. Register B before C, but give C sorting keys that
  would put it ahead of B on screen. Prevent other new first-tab candidates. Use the
  common skirmish settings and an order recorder, plus tick-labelled screen capture.
- Procedure: place the prerequisite of B and C while the tab is full. Confirm suppression.
  Destroy A's prerequisite by a test trigger, leaving the yard and B/C eligibility intact.
  Observe the resulting recheck and ten quiet ticks with no other recheck event. Then
  place an unrelated defence, causing another recheck without adding a first-tab icon.
  Repeat in a fresh run with C registered before B. Run each variant three times.
- Observable: all first-tab entries including scrolled entries, their first-appearance
  ticks, recheck ticks and removal-order issue ticks.
- Expected: the full tab suppresses both B and C. A's removal recheck leaves 75 icons,
  with neither retry appearing during that recheck or the quiet interval. The next
  recheck admits only B in the first variant, only C in the reversed-registration
  variant, reaching 76 icons. A better sorting key does not win the vacancy.
- Suppressed-loss run: from the initial full state with B and C suppressed, destroy
  their prerequisite, not A's. Expected: no cancellation order for B or C, no first-tab
  vacancy and no change to its 76 icons. Later restore their prerequisite and free A's
  place as above: the ordinary retry rules apply, with no priority from earlier attempts.
- Candidate-retention control: start with 75 first-tab icons and a switched-on factory
  of the building kind. Two further building types pass their tech-tree answers; the
  lower-index one has owner-country eligibility that no owned building factory shares,
  while the higher-index one has a matching factory. Give both no prerequisite beyond
  the established ones. Cause a recheck by placing an unrelated defence. Expected:
  the lower-index candidate leaves no final icon, the higher-index candidate is
  suppressed, and the tab ends at 75. At a later recheck it repeats unless eligibility
  changes. This isolates a newly admitted then removed candidate occupying capacity.
- Falsified if: a suppressed type appears without a later recheck, a vacancy freed by
  removal is refilled in that same recheck, priority follows sorting rather than the
  stated admission order, a still-suppressed loss issues either removal order, or the
  candidate-retention control admits its higher-index type. Void any run with an
  intervening recheck event or unintended candidate. Losing an owned object alone
  does not free its type's icon: the setup must actually remove its offer.
- Extension checks on our engine only: give two player houses these factories and
  prerequisites, with separate tab states. A build order naming a suppressed type must
  be illegal and change no line or credits. A vacancy in one house must not admit a
  type for the other. These check the project rule, not oracle behavior. No trigger or
  recorder is assumed able to inject a production order into the original.

**S11. Icon frame, mask and canvas (claim 16).**
- Setup: the common sidebar probe skirmish, with named loose icon test files and a
  test art copy selecting them under S-0007's first-read rules. Give each file two
  visibly distinct frames, a known nonzero-index grid and transparent holes. Use
  canvases smaller than, equal to and larger than 60 × 48 pixels, including an
  asymmetric frame displacement and one negative displacement. Also name a test
  darkening-mask copy with alternating zero and nonzero indices. Remove test files
  after each run. Start a fresh program session for each changed file configuration
  so retained art cannot hide it. Engine add-ons and mods other than these named files remain off.
- Procedure: show each icon idle, unavailable, running, held and done; repeat for
  each theater and side and two house colours. In a separate run make the mask
  unavailable through a named test archive omitting its member, with no other copy
  resolvable. Capture the sidebar, without mouse-over styling, at settled stages.
- Observable: identified image-frame marks, canvas placement relative to the slot,
  clipped pixels, mask coverage and destination hi-colour channel values.
- Expected: always frame 0 of the chosen main/alternate file; its origin stays at
  the slot top-left without scaling. Right/bottom excess clips; negative displaced
  content can extend toward earlier slots. Nonzero mask coverage halves each
  pre-mask integer channel, rounding down; holes leave it unchanged. Missing mask
  leaves unavailable icons without that darkening. Clocks/labels are independent.
  The same resolved palette and icon bytes do not change with house colour/theater.
- Falsified if: a state uses frame 1, the canvas is centred or stretched, clipping
  differs, or the mask paints colours rather than halving destination channels.
  Palette-file loading, conversion, session lifetime and missing-file tests are
  delegated to S-0103 claim 13 and PL, including the original startup failure and
  our engine's safe departure. One run per variant, screen capture plus
  read-only image/palette inspection by the measurer.

**S12. Sidebar text versus audio (claim 17).**
- Setup: the common skirmish map, enough credits to start slow building and infantry
  items, a limited type already at its limit and a blocked placement cell. Use a
  screen/audio recorder and a read-only recorder of message additions and label
  string-entry requests. Exclude triggers that print text and unrelated message events.
- Procedure: start, queue, hold, resume, cancel, click another icon on a busy building
  line, click the limited type, then exhaust credits with a running item and let
  a building complete after restoring them. Keep a second building on hold for
  300 ticks and a done building unplaced for 300 ticks, pausing midway. One run.
- Observable: battlefield-message additions, the ready/hold string identifiers,
  label presence per tick and announcement requests separately from playback.
- Expected: none of these sidebar actions adds a battlefield text message. The
  two labels resolve the entries of Import mapping and remain for the entire
  corresponding state, including across pause, ending at state change rather
  than after any set number of ticks. Decimal counts follow claim 7. Busy-line refusal is
  announcement-only. A placement-failure control uses S-0406's separately owned
  feedback, not a generic client-invented “refused build” notice.
- String-entry controls: in fresh program sessions use a named loose “sidebar-label
  probe string table”, retaining all other entries but replacing the two label
  entries with distinct test markers. Repeat with either entry removed in turn.
  Expected: ready and held states show their respective markers; a missing entry
  shows a missing-text diagnostic containing that identifier, not the other label
  or an empty default. Remove the test file after the controls.
- Falsified if: a listed sidebar event adds a text notice, a label expires while
  its state remains, a different string-table entry supplies either label, or the
  missing-entry result lacks its identifier.

**S13. Scroll dispatch and end feedback (claim 18, Q-0339).**
- Setup: common sidebar probe skirmish, speed 6, 800 × 600, no engine add-on and no mods (the named test files of a scenario excepted).
  Named map overrides offer enough types for two extra rows on one tab and fewer
  than one page on another. Count complete visible rows rather than assuming a
  shell size. Screen/audio capture and a read-only recorder log scroll positions,
  input activations and requested sound entries. Use S-0701 F10's named distinct
  tab-scroll/refusal test sounds if the stock entries cannot be distinguished.
- Procedure: at the top, click up; click down once with only two rows left, wait
  for movement to settle, then click down at the bottom. Repeat upward with less
  than a page left. On the all-fitting tab, click both disabled buttons. Separately
  press and hold each enabled button for 100 cycles before releasing over it,
  then repeat releasing outside it. Repeat with right-button clicks. Wheel in
  both directions at the top, interior and bottom and on the all-fitting tab,
  first with the pointer over icons and then over the battlefield. Compare single
  delivered wheel events with different positive and negative delta magnitudes.
  Run each variant once; date received input rather than script-send time.
- Observable: request counts, settled top rows, enabled states, sound-entry
  requests and audible playback. Allow the existing movement to finish before
  testing an end, so animation does not obscure the starting position.
- Expected: an enabled page button already at its end requests refusal; a
  partial page requests the tab-scroll cue and stops at the end without a later
  refusal. Both buttons stay enabled on the overflowing tab. Both are disabled
  and silent on the all-fitting tab. Holding dispatches nothing until release
  over the button, then exactly one page; release outside gives none. Left and
  right clicks agree. Wheel events move at most one row each, independent of
  magnitude and pointer location, and request neither cue in all tested states.
- Falsified by different end feedback, refusal after accepted partial movement,
  a disabled-button request, held repeat, activation on press, outside-release
  activation, wheel page movement or either wheel cue. The recorder distinguishes
  a missing sound request from playback suppression.

**S14. Start/resume announcement kind and queue suppression (claim 12).**
- Setup: common skirmish, slow infantry, ground/naval vehicle, aircraft, ordinary
  building and defence items, eligible factories, free docks, ample credits and
  unreached limits. Screen/audio and read-only announcement-request recorder.
- Procedure: start each kind, hold it, then resume from its own icon. For each
  unit line add another copy while its first item runs. One run per kind, with
  distinct input cycles; S-0701 F11 may serve as the same run.
- Observable: requested announcement identifiers, build-click requests and actual
  line/queue changes, separately from audible playback.
- Expected: infantry start/resume requests training, every other kind building.
  Queue additions retain build-click feedback but request neither announcement.
  Falsified by any opposite kind, missing predicted request or a new start/resume
  announcement merely for adding an active-line queued copy.

**S15. Command-button and tab images (claim 19, Q-2238).**
- Setup: (a) read-only: the stock install's side group that S-0005 claims 13 and 13a
  mount for each of the Allied, Soviet and Yuri sides. (b) the common sidebar probe
  skirmish with stock files, run once each with an Allied, a Soviet and a Yuri human
  house, each run the first game of a freshly started session, with S8's recorder,
  which dates each sidebar update and its drawn frame and here also logs per tick
  whether sell mode and repair mode are active. Bind keys to the sell-mode and
  repair-mode commands in the keyboard options. The map carries S13's named map
  overrides: one tab offers more icons than fit, so both scroll buttons are enabled
  while it is shown (claim 18), and another offers fewer than one page, so both are
  disabled while it is shown.
- Procedure: (a) for each of the three sides, resolve the sell, repair, four tab and
  two scroll images and the sidebar palette named in Import mapping by the ordinary
  precedence with that side's group mounted; render every frame of each with the
  palette resolved the same way. (b) In play capture the
  sidebar: at the start, before any icon exists (every tab unavailable); with each
  tab shown in turn; while a done building flashes its tab, once with that tab shown
  and once with another tab shown, across several toggles; with sell and with repair
  mode entered by its button and left by its button; entered by its button and ended
  by a battlefield right click without a scroll drag (S-0214 claim 28), which ends
  that mode; entered by its key and ended the same way; entered by its key and ended
  by its button; with a scroll button held, released, disabled (all icons fit), and
  pressed while disabled. With each scroll button once enabled and once disabled, and
  with each tab, sell and repair button in its state at the time, all with no press,
  capture it with the pointer away from the sidebar and with the pointer resting over
  the button. Hold a press, the
  pointer over the button and before the release, on a tab button while its tab is
  unavailable, while it is not lit and shown, while it is not lit and not shown, while
  it is lit and shown and while it is lit and not shown; and on the sell and on the
  repair button, once with its mode off and once with it active, entered by its button.
- Observable: (a) frame counts and the look of each frame. (b) per capture, the
  frame each button shows, matched against the part a renders, and the sidebar
  update it falls in; for each mode ended by the right click, every drawn frame of
  its image from the last tick the mode is active to the first frame 0; for each
  click on a sell or repair button, whether its mode is active in the recorder's log
  before and after the click.
- Expected: (a) every file and the palette resolve for each side, and each file has
  the frames claim 19's first table uses. (b) Claim 19's first table, with sell and
  repair modes entered and ended by their buttons: the first click on the button
  makes its mode active and the second makes it inactive. Each side shows the images of its mounted
  side group, in the first game of the session, as part a renders them for that side,
  colours included. A mode entered by key shows frame 0 while active, and in every
  frame drawn after the click on its button that ends it. A scroll button shows
  frame 0 when enabled and frame 2 when disabled, with the pointer away and with it
  resting over the button, and every other button shows the same frame in both
  pointer positions. A mode entered by its button and ended by the right click shows
  frame 1 in each frame drawn while it is active, and frame 0 in each frame drawn
  after the first sidebar update that finishes with the mode inactive. During each
  held press on a tab, sell or repair button, the button shows the frame claim 19's
  first table gives for its state at that moment. A disabled scroll button shows
  frame 2 while pressed.
- Falsified if: any button shows a frame other than claim 19's first table predicts for a
  state part b captures, apart from the original's mode lighting for a mode entered
  or ended other than by its button, which the items below test. Also falsified if a
  tab, sell or repair button shows any other frame during a held press, a click on a
  mode's button leaves an active mode active, a pointer position without a press
  changes a button's frame, or a key-entered mode shows frame 1 while active or in a
  frame drawn after the click on its button that ends it. Also if a button-entered
  mode ended by the right click shows frame 0 while active or frame 1 in a frame drawn
  after that first sidebar update, a side shows images other than those of its
  mounted side group in the first game of the session, colours included, or a file
  lacks a frame claim 19 uses. One run per side, screen capture plus read-only image
  and palette inspection.
- Part (c), pixels (claim 19 *Pixels*, Q-2266). Setup: part b's skirmish with a
  human house on a 5-6-5 display path, as S-0103's *Reading a pixel* requires. Every
  run below is made once for each side: an Allied, a Soviet and a Yuri human house,
  each with its side group mounted as S-0005 claims 13 and 13a give.
  Every run adds loose files, resolved ahead of the stock ones and of every side
  group's files by S-0005's precedence:
  - a test copy of each of the eight button images of claim 19 (sell, repair, the four
    tabs, the two scrolls), each with its stock file's frame count and frame size. Every
    frame holds every index from 1 to 255 at least once at two horizontally neighbouring
    pixels, laid out in a pattern of its own, so that a capture tells which frame is
    shown, and index 0 at the rest of its pixels. Before the runs, check that every
    stock frame has more than 510 pixels. If one has not, its indices are split over
    two copies, each index in one copy only, and the runs below are made once with
    each.
  - a test copy of the sidebar palette in which, for every entry from 1 to 255, the red
    and the blue file bytes are odd and their converted value (claim 19's Import
    mapping) is not from 128 to 159, no two entries pack to the same 16-bit value, and
    no entry packs to black (every component 0 after S-0103 claim 9's step) or to white
    (every component at the top of its packed range).
    Under these rules a red or blue component's full 8-bit value differs from its
    packed value with either low-bit fill (S-0103 claim 9), so a full-precision drawing
    shows. Entry 0 holds a test colour.
  - Run c1: as above.
    Run c2: c1's files, the human house given in the skirmish setup a house colour whose
    remap ramp (S-0103) packs to a value other than c1's house colour's ramp at every
    index of the remap range other than 31 (pure black in every ramp, S-0103 claim 6),
    both ramps' packed values recorded, on a flat test copy of
    the map with no light sources whose normal lighting has ambient level 0, ground bias 0,
    level factor 0 and red, green and blue tints 0. Every cell's light level is then 0
    (S-0104 claim 9). The run records, for each of the three components, whether a
    captured nonzero-index button pixel has that packed component above 0, and which
    remap-range indices the captured button pixels hold. Run c1d: c1's files, plus loose
    test copies of the four
    shell images named in Import mapping, each with its stock file's frame count and frame
    size and every frame filled with a checkerboard of single pixels in two nonzero
    indices *p* and *q*, and with test palette entries *p* and *q* changed so that they pack
    to black and to white. These two entries are exempt from c1's palette rules. In c1d,
    c1b and c0d a button pixel at index *p* or *q* is judged only against that run's own
    entry, and is left out of the full-precision falsifier and of every comparison with c1.
    The two checkerboard colours are black and white, as claim 19's *Shell images* gives.
    If c0d's capture shows the single-pixel checkerboard in two other colours, that
    falsifies *Shell images*' palette. Those two colours are then recorded and used, and
    c1d's readings over them raise no rating to high (claim 19). Run c0d: c1d's files, plus
    loose test copies of the eight button images, each with its stock file's frame
    count and frame size and index 0 at every pixel. Run c1b: c1d's files, with only
    entry 0 of the test palette changed to another colour, whose packed value differs
    from c1's entry 0 in every component.
    Where the shell images lie beneath the sidebar's pixels is read from c0d's capture,
    not claimed. A checkerboard position of a button is a pixel of its rectangle at
    which c0d's capture shows one of the two checkerboard colours and each of its four
    edge neighbours shows the other. The colour beneath it is c0d's pixel there. Run
    c3: c1's files on a test copy of the map whose own lighting section sets the ion
    storm's lighting set to test values, as S-0103's scenario PD does: ambient level 0,
    ground bias 0, level factor 0 and red, green and blue tints 0. The map starts an ion
    storm on a fixed tick through its own trigger, and the storm lasts long enough for the
    current ambient to reach the storm's ambient (S-0104 claim 4) and past the run's last
    capture. Its captures are all taken during the storm, after the current ambient has
    reached the storm's. Every tintable table (S-0104 claim 3) is then rebuilt with a tint
    of 0 in every channel, which is black at every index that is not tint-exempt (S-0104
    claim 6), and every cell's light falls to level 0. Run c4: c1's files on a test copy of
    the map with a trigger that, on a fixed tick before the captures, sets each of the
    three map-tint channels (S-0104 claim 17) and the base ambient (S-0104 claim 14) to 0,
    so that a table retinted with the map tint is black at every index that is not
    tint-exempt. Its captures are taken once the ground's light has stopped changing after
    that tick.
- Procedure: in each run, captures of the sidebar with the pointer away from it and no
  tab lit, taken in the same states in every run. One at the start, before any icon
  exists, where every tab is unavailable (frame 2). Then, once every tab has icons (part
  b's map gives each tab some), one at rest with each tab shown in turn, the overflowing
  tab at its top row: the shown tab shows frame 1 and the others frame 0, and the scroll
  buttons show frame 0 with the overflowing tab shown and frame 2 with the tab of fewer
  than one page shown. Then, with the overflowing tab shown, one with sell mode entered
  by its button and one with repair mode entered by its button (sell mode ended first).
  Every capture shows every button. The scroll frames of the start capture are not
  read. Then, with part b's overflowing tab shown at its top row, where both scroll
  buttons are enabled (claim 18), press each of the two scroll buttons in turn, the up
  button first. For each, two or more captures hold a press on it with the pointer
  resting on it (frame 1, claim 18), each with the pointer at another place on the
  button, near its opposite edges first. Each capture is read while the press is held.
  Each press then ends by moving the pointer off the button and releasing there, which
  requests no scroll (claim 18), so the page stays at the top row. A capture taken after
  a release that changed the shown page is not used, and the press is repeated. More
  places are added until every pixel of each scroll button lies outside the pointer's
  image in at least one of its captures, so every index of both pressed frames is read.
  Run c0d presses at the same pointer places as c1d. A
  run counts only if
  each capture shows
  the same frame of each button as the same side's c1 capture in the same state, the
  sidebar's icons and their states are the same as in that capture, and every sidebar
  file is the same as in the run's base except the ones the run changes. c1 is the base
  of c2, c3, c4 and c1d (which changes only the four shell images and
  test palette entries *p* and *q*). c1d is the base of c0d
  (only the eight button images) and of c1b (only entry 0 of the test palette). In
  c0d, whose button frames hold index 0 only, the frame is not read: its captures count
  when they are taken in the same state and at the same pointer place as c1d's.
- Observable: per run and per capture, each button's captured pixels, matched pixel by
  pixel against the frame of its test image that the capture shows, leaving out the
  pixels under the pointer's image in the pressed-scroll captures (in c0d's or c1d's
  capture at the same place, when the two are compared). A pixel at an index
  from 1 to 255 is read by S-0103's *Reading a pixel*, and its full 8-bit value is kept
  too. *Reading a pixel* leaves out index 0, so an index-0 pixel is read by its own rule:
  its positions are those the test copy's layout gives index 0 in the frame shown, and at
  each of those positions the captured screen pixel's three 8-bit components are kept as
  they are. These
  values are never compared against a palette entry. In c0d, every pixel of each
  button's rectangle is kept in the same way, and the checkerboard positions are found
  from it (Setup). Where a frame's indices are split
  over two copies (Setup), each copy's runs are read on their own, every index is judged
  in the copy that holds it, and the index-0 positions of every copy are read.
- Expected: in every run and capture, every pixel at an index from 1 to 255 equals that
  entry's converted colour packed by S-0103 claim 9's step, its low bits filled with
  zeros or with copies of the top bits. On each side, c1d's, c1b's, c2's, c3's and c4's
  pixels at nonzero indices equal c1's. At each index-0 position that is a checkerboard
  position, c1d's pixel equals c0d's, and c1b's equals c1d's. Index-0 positions
  elsewhere are recorded and judge nothing. Where a button's index-0 positions at
  checkerboard positions include both checkerboard colours in c0d, c1d's index-0
  pixels there show both colours. A darkened nonzero-index pixel, or one blended
  keeping a weight of its entry's colour no greater than *a*ₘ, misses its entry's packed
  colour in c1d at one pixel at least of each of its neighbouring pairs at checkerboard
  positions, so it fails the first sentence of this Expected, as follows. A blend
  nearer opaque than *a*ₘ may pack to its entry's colour over both
  checkerboard colours and pass. Two horizontally neighbouring checkerboard positions
  stand over both checkerboard colours. Let *D* be the smallest difference over the three components
  between the two captured checkerboard colours (at least the top of the packed range for black and
  white). A mixing that keeps a weight *a* of the entry's colour moves each result by (1 − *a*)
  times what lies beneath, so its two results differ by at least (1 − *a*)(*D* − 7) levels in every
  component before packing, less up to 8 for the mixing's own rounding. They then pack to different
  values, so at least one pixel misses the entry's colour, for every *a* up to
  *a*ₘ = 1 − 16 / (*D* − 7) when *D* > 23 (Formulas). A darkening shadow shows black over
  black. No entry of c1d's test palette other than *p* packs to black, so at any other
  index it misses the entry's colour at the pixel over black. Index *p* is not judged for
  darkening.
  With split copies, the statements about every index from 1 to 255 hold over the union
  of the copies' indices, and the statements about index-0 positions hold in every copy.
- Falsified if: a nonzero-index pixel shows the converted colour at full 8-bit
  precision, or another colour, in any capture. Also falsified, as a shadow or
  translucency, if a nonzero-index pixel of c1d differs from the same side's c1. If no
  index's pair lies at checkerboard positions, that reading is inconclusive and the
  rating of "no shadow or translucency" stays medium at every index. If the
  checkerboard is not black and white, that falsifies claim 19's *Shell images* palette, and
  "no shadow or translucency" stays medium at every index, since claim 19 rates it high
  only over black and white. If run c2 records no captured nonzero-index pixel with a
  component above 0, its reading of that component's tint is inconclusive, and if it
  records no captured pixel at a remap-range index, its house-colour reading there is
  inconclusive. Also
  falsified, as a drawing of index 0 that depends on entry 0, if at an index-0 position
  that is a checkerboard position c1b's pixel differs from c1d's. Also falsified, as a
  drawing of index 0 other than what an all-index-0 copy shows, if at such a position
  c1d's pixel differs from c0d's. Also falsified if any nonzero-index pixel of c1b, c2,
  c3 or c4 differs from the same side's c1. A c3 capture set whose battlefield ground
  pixels equal c1's in the same state decides nothing, since the storm's set was then not
  applied, as in S-0103's scenario PD. The same holds for c4 and its trigger action. In c3
  and c4 a pixel at an index from 240 to 254 that equals c1's raises nothing to high. If an
  image has no index-0 position at a
  checkerboard position, the entry-0 reading is inconclusive for that image, and its
  rating stays medium. If no button's index-0 positions at checkerboard positions
  include both checkerboard colours, the reading of "not one fixed colour" is
  inconclusive, and its rating stays medium. One capture set per run, screen
  capture plus read-only
  image and palette inspection.

**S16. Sidebar tips (claims 20 to 22, Q-2791).**
- Setup: the common setup with the tooltip option on, at 800 × 600, and again at 1024 × 768 for
  parts g and h. Screen capture at every drawn frame, each frame stamped with its real time, and
  scripted pointer movements. The skirmish's bases setting is on, and the observed house deploys
  its base unit (S-0009 Terms) into its construction yard at the start of the run; it has
  credits. A structure that the map's building list gives the observed house is dropped
  (S-0009 claim 27), so every building that S16 hands to it is pre-placed for a donor house, as
  in S-0405's measurement setup (a computer house with nothing else on the map), and carries a
  test-map tag whose triggers' elapsed-time events (S-0011 claim 13) change its owner to the
  house at the observed house's start position (S-0011 claims 23 and 24) or destroy the objects
  that carry the tag (S-0011 claim 27). The sidebar probe map has: a vehicle test type, a copy of
  a simple stock vehicle that the observed house's country may build, with an invented cost
  field, a prerequisite field that names the vehicle factory type below alone, and a display name
  of three short words,
  taken from a probe entry in a loose probe string table built as S-0004 scenario L builds
  it; the invented country cost factor for vehicles of S-0404 scenario P2 for the observed house's
  country; a building test type that grants an invented cost bonus for vehicles, has no power
  output and needs only the construction yard, with one building of it pre-placed for the donor
  house, whose tag hands it to the observed house at a game time *t*<sub>1</sub> and destroys it
  at a later game time *t*<sub>2</sub>, both chosen for part e; power plants pre-placed for the
  donor house, enough for the observed house to have full power until they go, whose tag hands
  them to the observed house within the first ticks and destroys
  them at a game time *t*<sub>3</sub> chosen for part d; a stock vehicle factory building type of
  the observed house's side, with one building of it pre-placed for the donor house, whose tag
  hands it to the observed house within the first ticks, so that the vehicle test type is
  buildable (Terms) and its icon is shown from then on; and a building test type of the same
  line (claim 1) that grants a superweapon test type, needs only the construction yard and has
  a base drain above the output of every building the observed house owns other than the power
  plants (S-0405 Terms), so that while either of the two builds the other's icon is darkened
  (claim 3, a building line with an item). The superweapon test type is power-dependent (S-0512
  Terms) and has a display name of three short words, taken from a second probe entry of the
  same table. The bonus building's test type and the superweapon building's test type each have a
  display name of two short words, taken from a third and a fourth probe entry of that table. The field values and the triggers' times chosen are recorded with the measurement.
  A run is void for a part when a handed-over building is not the observed house's after its
  hand-over time, when a trigger's event falls outside the rest that the part names for it, or
  when, in part d, the house does not have low power after *t*<sub>3</sub>.
- Procedure and expected results:
  - (a) Move the pointer in one movement from the battlefield view onto the vehicle icon and rest
    12 s, then move it by one pixel within the icon and rest 12 s. Expected: the tip shows 1000 ms
    after the first movement and hides 10000 ms after it showed. The one-pixel movement shows it
    again in the first frame after the movement, with the same text and place, and it hides 10000
    ms after that movement.
  - (b) With the tip showing, move onto the neighbouring icon in one movement, then keep moving
    between icons every 2 s for 30 s, then rest. Expected: each icon's tip shows in the first
    frame after the movement that reaches it, the tip never hides between movements, and it hides
    10000 ms after the last one. A movement onto an empty slot hides it.
  - (c) With the vehicle tip showing, left-click the icon and move the pointer by one pixel 1 s
    later. Then right-click the icon, which puts its item on hold (claim 5), with the tip
    showing, and again move the pointer by one pixel 1 s later.
    Expected: the tip hides at the press and shows again at the movement, at
    once, with the same text and price each time. Then press the left button on the icon, move the
    pointer by one pixel within the icon with the button held, and release it 1 s later. Expected:
    the tip hides at the press, shows again at once at the movement with the button held, and hides
    at the release.
  - (d) While the superweapon building builds, rest on the bonus building's darkened icon. Rest on
    the superweapon building's icon while it builds, when it is done and waiting for placement and
    in placement mode, and on the superweapon icon while it charges, when it is ready, and after
    the power plants' tag has destroyed them at *t*<sub>3</sub>, chosen after the superweapon is
    ready, so that the house has low power (S-0405 Terms) and the superweapon is suspended (S-0512
    claim 6; its icon shows the on-hold label). Expected: each building icon shows its type's probe
    name, one word on each line, and the price S-0404 claim 2 gives for its recorded cost field,
    written as in (e). The superweapon icon shows its type's display name alone, the three words on one line
    with the spaces between them, with no price, in each of the three states. Falsified by a price,
    a line break between the words, or a text that differs between the states.
  - (e) Before *t*<sub>1</sub>, while the observed house owns no bonus building, rest on the
    vehicle icon and read its tip. Expected: the
    money-format entry's text with the probe name and the price written in, every space a line
    break, so the three words on three lines and the price on the line where the entry places it.
    The price is S-0404 claim 2's *P* for the recorded cost field and factor, with no bonus. Keep
    the tip showing so that *t*<sub>1</sub> falls within its lifetime. Expected: with no movement,
    the price shown changes to *P* with the handed-over building's bonus once the building is the
    observed house's, and the tip stays. Then move the pointer by one pixel within the icon, so
    that the tip's lifetime starts again (claim 20), with *t*<sub>2</sub> within that lifetime.
    Expected: with no movement after that, the price shown changes back to *P* without the bonus
    once the building is destroyed, and the tip stays. Then build and place a bonus building and
    move the pointer back onto the vehicle icon. Expected: the price is *P* with the bonus again.
    Falsified by the cost field shown as it is, by any space kept, by a price without the factor,
    or by a showing tip's price that does not follow the gain or the loss.
  - (f) With enough icons in the tab that one wheel step downward brings an icon into a filled slot
    of the last shown row, rest on that slot until its tip shows, then, within 5 s, turn the mouse
    wheel by one step downward without moving the pointer. Rest until the tip hides. Then move the
    pointer by one pixel within that slot and, while its tip shows,
    turn the wheel step by step until that slot is empty. Expected: after the first step the tip
    shows the icon now in that slot, its box placed by claim 21, with the original's shifts of Edge
    cases as in (g), for that slot and the new text's size (the same place when the two texts give
    boxes of the same size), and it hides 10000 ms
    after it first showed: the step does not start the lifetime again. When the slot becomes empty
    the tip hides, and nothing shows until the pointer moves.
  - (g) At each resolution rest on an icon of each column in the top and the bottom shown row,
    and read the box's place against the slot's rectangle and the tip area. Expected: claim 21's
    corner, with the original's 4-pixel and 1-pixel shifts of Edge cases for an icon in the right
    or the lower half of the area. Falsified by a box placed by the pointer, or by any other
    overlap with the icon.
  - (h) Rest on each tab button, one of them while its tab has no icons; on each scroll button with
    all icons fitting and with more than fit; on the repair and sell buttons, the power meter, the
    options button and the diplomacy button; and on the radar panel, the credits counter and a gap
    between two controls. Expected: claim 22's text, 1000 ms after the last movement, placed as
    claim 22 gives (16 pixels below the pointer, or above it near the area's bottom edge, kept
    inside the area at its right edge), or, in the original, as Edge cases give in their cases, in
    the name tip's look (S-0111 scenario NT7). The power numbers equal the house output and drain
    that S-0405's recorder gives. No tip on the radar panel, the credits counter or the gap.
  - (i) For the original only, the Edge cases: a tab holding 76 icons (S-0004 scenario
    F4's map); a fast movement from an icon onto a vehicle on the battlefield view; and, as in
    (f), a wheel step that empties the slot under the resting pointer 2 s after its tip shows,
    then a step back 2 s later. Expected: no icon tip in that tab; the vehicle's name tip showing
    in the first frame after the movement; and the slot's tip showing again 10000 ms after it
    first showed, with no movement.
- Observable: each tip's text lines, its box's place and size, and the real times of its showing
  and hiding, from the stamped frames.
- Falsified by: a wait of 1000 ms between icons, a tip hiding between movements over the slots, a
  text other than the one predicted, a box placed otherwise, or a tip where none is expected.

## Open questions

1. Battlefield right-click exit without item cancellation is resolved by claim 10 and
   S-0214 claim 28. Whether a key also leaves placement mode without cancelling remains open.
2. Cosmetic early clock anticipation is outside the look requirements under the lead's
   Interface looks decision of 2026-10-14; claim 8 fixes the production value.
3. Resolved by claim 8: a done item's clock overlay is absent.
4. Resolved by claim 12: busy-building refusal and refused placement request different
   announcements, not a shared game-wide refusal sound.
5. Whether a right click in the same cycle as a queueing or resuming left click gives an extra hold
   order (claim 5). S-0213 claim 1 makes these cases unreachable through permitted skirmish input.
6. Which sound, if any, the build-limit click, the darkened-icon click and the click on a lit
   aircraft icon with every dock taken play (claims 4 and 12, S3).
7. Q-0283 is resolved by claim 11: initial and subsequent lit periods are specified.
   The lit artwork is the implementer's design or claim 19's install frames.
8. Q-0120 is partly resolved by S-0404 claims 7, 13 and 15: busy or failed release
   cancels with a refund without retry, and claim 15 gives ordinary aircraft pad positions.
   S-0417 claim 14 supplies contact-place count, infantry exit-candidate acceptance and
   nearby destinations for edge-entering aircraft. Remaining no-rally infantry walk-out,
   take-off, specialised post-contact movement and initial-placement draw questions
   belong to S-0417's open questions. Specialised docking combinations remain outside the
   verified pad-position rule. This spec adds no rules for those outcomes and does not
   claim to answer the whole request. Q-0196's four questions are answered by claim 15.
9. Q-2238 question 1 and Q-2791 question 5, power-mode part: whether the original offers a
   power-mode button or an image for one is not claimed here. Our engine's power-mode button and its
   picture are design (claim 19).
10. Q-2791 question 5, waypoint part: the original's planning-mode (waypoint) button and the other
    command buttons lie on the command bar, outside the sidebar, and show tips of their own.
    Those tips are not claimed here.
11. An entry of claims 20 and 22 whose insertion points differ from Import mapping's (another
    count, kind or order): what the original shows then is not claimed here.

## Changes for other specs

- S-0214: claim 13 gives the sell and repair buttons as mode entry. Claim 19 here
  relies on a click on either button ending its mode while active. Add that sentence
  to claim 13 at hand-off. No behavioural conflict.
- S-0404: refresh its paired Summary reference to S-0411 claims 9, 14 and 15
  at joint hand-off. Its application-boundary release/refund, sale and capacity contracts
  already agree; no behavioural replacement is required.
- S-0701: Q-0339's dispatch is now claim 18 and S13 here. Replace its remaining
  dispatch-open wording with that pointer: all-fitting tabs disable both buttons,
  holding does not repeat, and wheel requests are one-row and silent.
- S-0406: Q-0282 placement text/no-text is already answered by claim 20 and PL15.
  retain that ownership. No behavioural change required.
- S-0408: Q-0282 sell/repair text/no-text is already answered by claim 18 and S12.
  retain that ownership. No behavioural change required.
- S-0213: claim 25 owns route notices and the shared status display's lifetime,
  capacity, wrapping, explicit line breaks and font metrics; retain that ownership
  separately from sidebar labels. No behavioural change required.
- S-0007: Q-0275 main/alternate icon selection is already claims 22 and 25.
  retain the pointer to claim 16 here. No behavioural change required.
- S-0103: Q-0275 palette conversion, lifetime and missing-file behaviour are already
  claim 13; retain that ownership. No behavioural change required.
- S-0005: its scenarios that read a unit's cost on the sidebar tooltip read the local player's
  price (claim 20, S-0404 claim 2), which equals the cost field only while the house's cost
  factor and bonuses for that kind are 1. Their set-ups should keep both at 1. No rule changes.
- S-0002 scenario D2, S-0004 scenarios H and L and S-0015 scenario C read display names on the
  sidebar tooltip: there each space of a name shows as a line break (claim 20), and S-0004 L2's box
  widths are those of S-0111's name tip, not of the sidebar tip. S-0015 C1 (g) compares the boxes
  of S-0111's name tip too, where spaces stay spaces: in the sidebar tooltip the two-space copy
  shows an empty line between its words rather than a wider box. S-0002 D2's cost read there is
  the price of claim 20. No rule changes.
- Glossary, *sidebar*: may add that S-0411 gives the sidebar's tips (claims 20 to 22).
Earlier correction requests are resolved by the current companion drafts: S-0406 claims 1
and 20 give placement's appropriate announcement and text/no-text feedback; S-0007 claims
22 and 25 point to claim 16 for icon content; S-0103 claim 13 owns dedicated icon-palette
loading/conversion and missing-file behaviour. S-0408 claim 18 and S-0213 claim 25 own
sell/repair text and route notices respectively, separate from sidebar label lifetimes.

## Questions for the project lead

1. Should the proposed switch `quirks.release_order_once` (the sidebar sends its automatic place
   order once; default on = original) be kept? Without it our engine reproduces the original with no
   way to change it.
