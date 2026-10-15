---
id: S-0512
title: "Superweapons: charge and readiness"
topic: superweapons
revision: 7
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: [Q-0742, Q-0730, Q-0731, Q-0737]
supersedes: []
---

# S-0512: Superweapons: charge and readiness

## Summary

How a house comes to have a superweapon and loses it, how its charge runs, stops and starts again,
when it is ready, what firing it does to the charge, and what the sidebar and the on-screen
countdown show of that state. Skirmish only; campaigns and network games are out of scope.
What a launched superweapon does (missile, storm, curtain,
chronosphere jump, paradrop and the rest) belongs to S-0513; this spec gives only the contract
between a fire order and the effect.

## Terms

- **Superweapon record.** The charge and availability state of one superweapon type for one house.
  A house can have only one charge of a given type, however many buildings give it. A house "has"
  a superweapon when that state is *granted*. This is a behavioural term, not an implementation
  requirement.
- **Granted, one-time.** A granted record shows an icon to its owner and can charge and fire. A
  *one-time* record is a granted record that was given by a trigger action as a single use (claim
  12); ordinary types are revoked when they fire. Drain-mode types are an exception (claim 19).
- **Permanent trigger grant.** A record granted by the trigger action that gives a superweapon for
  good (claim 13). Buildings and power never suspend it or take it away.
- **Charge time *T*.** The superweapon type's charge time in ticks (Formulas F1).
- **Charge clock, remaining time.** Each record has a charge clock that is either *running* or
  *stopped*, and a *remaining time* in ticks. A running clock counts down by one per tick; a
  stopped clock keeps its remaining time unchanged (Formulas F2).
- **Ready.** A granted record whose charge has completed (claim 5). This refines S-0411's *ready
  superweapon* (Terms): a ready superweapon in S-0411's sense is a ready record that is not
  suspended and is inside its flash window (claim 17).
- **Suspended, resumed.** A granted record is *suspended* when power has disabled its sidebar
  targeting and normally stopped its clock (claims 4 and 6), and *resumed* when that condition ends.
  Claims 10 and 14 describe exceptions in which the clock runs despite suspension. These are the suspension and
  resumption of S-0407 claim 12, event (k), and the "holds its charge" of S-0405 claim 15. Specs
  write "suspended", not "held": *hold* is the glossary's mobile-unit countdown.
- **Lost.** A granted record that stops being granted because nothing gives it any more (claim 7).
- **Power-dependent.** A superweapon type whose power-dependence flag is set (Import mapping).
- **Giving building.** For a house and a superweapon type, a building of that house that is on the
  map, is not destroyed, and gives the type in one of two ways:
  - its type's first or second superweapon field names the type, and the type has no auxiliary
    building, or the house owns at least one building of the auxiliary building type; or
  - one of the upgrades it carries (S-0406) has the type in its first or second superweapon field.
    The auxiliary building condition does not apply to upgrades.

  Health, being in operation (S-0405), an electromagnetic pulse and being erased play no part. The
  power switch matters only for suspension (claim 6).
- **Clock stage.** A whole number from 0 to 54 that tells how far a record has charged (Formulas
  F3). The sidebar shows it on the superweapon's icon; how it looks is the implementer's design.
- **Readiness label.** The word state that the sidebar shows on a superweapon icon: none,
  *on hold*, *ready*, and for drain-mode superweapons *charging* and *active* (claim 16).
- **Two-stage superweapon.** A pair of types: a *first stage*, whose firing only picks a source
  (its first-stage flag is set), and a *second stage*, whose type has the second-stage flag and
  names the first stage through its link field (Import mapping). The stock chronosphere is one.
  The second stage need not be granted itself; its fire order is offered after the first stage fires
  (claim 10).
- **Drain-mode superweapon, manual-charge superweapon.** Types with the drain-mode flag or the
  manual-charge flag (claims 19 and 20). No stock type uses either.
- **Countdown row.** One line of the on-screen superweapon countdown (claim 18).

## Behaviour

1. **Records.** Each house has independent availability and charge for each superweapon type.
   Before any building or trigger grants a type, it is not granted, ready or suspended. A record is granted by a building (claim 3) or by a
   trigger action (claims 12 and 13), and stops being granted when it is lost (claim 7), when a
   one-time record fires (claim 12) or when its house is defeated (claim 22). Being granted, ready
   and suspended are independent flags with the combinations given in State and ordering.
   - Backing: `measurable` (scenarios X1 and X4)
   - Confidence: high

2. **When buildings give superweapons.** A house is granted a superweapon type, and the house is
   checked for losing it, only at the moments below. Between them, buildings that come and go
   change nothing about the house's superweapons.
   - (a) every recheck of the house (S-0407 claim 12), in the house update where it happens:
     suspension, resumption and loss (claims 6 and 7) and new grants (claim 3) take effect in that
     update. A record lost there can be granted again there if its giving conditions still hold.
   - (b) a change of the house's low-power state, worked out in its house update (S-0405 claims 4
     and 7): the check for suspension, resumption and loss only.
   - (c) a building of the house gaining an upgrade: the grant check, at once, in addition to the
     recheck that S-0407 claim 12 (d) makes due.
   - (d) a building of the house losing an upgrade, or a building that has a superweapon in its
     first or second superweapon field, with its auxiliary condition satisfied, being removed:
     the check for suspension, resumption and loss, at once. Upgrade removal makes this immediate
     check only when the removed upgrade names a first superweapon; naming only a second one
     does not suffice (`quirks.superweapon_upgrade_second_removal_waits`, default on = original,
     off: an upgrade naming either superweapon field makes the immediate availability check).

   An event of S-0407 claim 12 that makes a recheck due therefore takes effect on superweapons in
   the house update that S-0407 claim 12 names: for a building placed by order, the next tick's
   house update; for a capture or a deploy, the same tick's.
   - Backing: `measurable` (scenarios X1 and X4, including X4's upgrade variants)
   - Confidence: high for parts a and b; medium for c and d

3. **Grant by buildings.** At a grant check, a record that is not granted is granted when the house
   is not defeated and has at least one giving building for its type. Two exceptions:
   - a disableable superweapon is not granted while the superweapon setting is off (S-0412 claim
     9, part b), and
   - a one-time record is already granted, so a grant check leaves it as it is (claim 12).

   Whether a building type that gives a disableable superweapon can be built at all while the
   setting is off is S-0407 claim 7's rule. A captured building that gives a superweapon gives it
   to its new owner through this claim; the other consequences of capturing tech buildings are
   S-0515's.

   The new grant sets the remaining time to *T* and starts the clock at the grant (for a
   suspended start, see claim 4; for manual charge, claim 20). It makes no announcement.
   Subject to the tab-capacity rule of S-0004 claim 11, the local player's sidebar gains the icon (S-0004 claim 11 places it; S-0007 claim 31 gives its art). The grant takes
   no random draw.
   - Backing: `measurable` (scenarios X1 and X12; X1 includes a random-state control)
   - Confidence: high

4. **Suspended from the start under low power.** A record that buildings grant while its house has
   low power (S-0405, Terms) starts suspended, with its remaining time *T*, whether or not the type
   is power-dependent. It is resumed by the rules of claim 6: when the house's power becomes full
   while it has a giving building that is switched on, or at a later recheck that finds the same.
   A type that is not power-dependent is never suspended in any other way
   (`quirks.superweapon_grant_suspends_by_house_power`, default on = original; off: a grant
   suspends the record exactly when claim 6 would, so a type that is not power-dependent always
   starts charging, and a power-dependent one also starts suspended when no giving building is
   switched on). With the original behaviour, a power-dependent type granted while the house has
   full power but every giving building is switched off starts charging and is suspended only at
   the next check of claim 2.
   - Backing: `measurable` (scenario X3)
   - Confidence: high

5. **Charging and becoming ready.** In a house update, a granted record whose clock is running and whose remaining time has reached 0 becomes ready if it is neither suspended nor already ready. If that update also includes a check under claim 2 (a), the record requests its ready announcement even if the check then removes it; a low-power change that suspends it in that update prevents completion. A record whose clock started with remaining time *R* in
   the house update of tick *s* therefore becomes ready in the house update of tick *s* + *R*; an
   ordinary record granted in tick *g*, charging without interruption, becomes ready in tick
   *g* + *T*. Becoming ready requests the announcement of S-0701 claim 28 only for the local
   player's own record; the kind chooses the announcement (Import mapping). Playback still
   follows that spec's suppression rules. Other houses get no ready request for this record. The
   tick in which it becomes ready is the record's *ready tick* (claim 17). Drain-mode completion
   has no ready announcement and does not set a ready tick (claim 19).
   - Backing: `measurable` (scenario X1)
   - Confidence: high

6. **Suspension and resumption.** At a check of claim 2 (a), (b) or (d), a granted record that is
   neither one-time nor a permanent trigger grant, of a house that is not defeated and that has a
   giving building for it, is judged as follows. The record is *powered* when at least one giving
   building is switched on (S-0405 claim 10) and the house does not have low power.
   - Not powered, power-dependent, not yet suspended: it is suspended. The clock stops and keeps
     its remaining time.
   - Powered, suspended: it is resumed. An ordinary clock runs again from the remaining time it
     kept; a manual-charge clock stays stopped (claim 20).
   - Otherwise nothing changes. In particular a record that is not power-dependent and was
     suspended at its grant (claim 4) stays suspended until it is powered.

   Except for the running-through-suspension cases of claims 10 and 14, a suspension of *d* ticks
   delays readiness by exactly *d* ticks, and the countdown row shows
   the same value while it lasts. A ready record can be suspended too: it stays ready, but it cannot
   be selected for firing through the sidebar and normally cannot fire an order (claim 8). It shows the *on hold* label until it is resumed. Damage, an
   electromagnetic pulse, being erased and being drained (S-0405) do not directly suspend a
   record, nor does leaving operation for any reason other than the switch. A resulting change
   to the house's power is still subject to the powered test above. A suspension or
   resumption makes a recheck due (S-0407 claim 12 (k)). For the local player it also cancels the
   targeting mode of that superweapon if it was active. This refines S-0405 claim 15.
   - Backing: `measurable` (scenario X2). Electromagnetic-pulse independence is `unmeasured`:
     the giving-building and switch tests do not inspect that state, but no stock skirmish pulse
     delivery is traced here to provide an independent observable test. Independent targeting-mode
     cancellation on suspension/resumption is also `unmeasured`: X2's power-switch input already
     leaves aiming, so it does not isolate cancellation by the availability check
   - Confidence: high

7. **Loss.** At the same checks, a granted record that is neither one-time nor a permanent trigger
   grant is lost when its house has no giving building for it, or the type is a disableable
   superweapon and the superweapon setting is off. A lost record is no longer granted or ready and
   leaves the countdown (claim 18). A suspended record that is lost stays marked suspended, which
   matters only for a later one-time grant (claim 12). Its charge is forgotten: when buildings grant it again, it
   starts from the full *T* (claim 3), however far it had charged. The loss makes a recheck due
   (S-0407 claim 12 (k)), and the local player's icon goes at that recheck.
   - Backing: `measurable` (scenario X4)
   - Confidence: high

8. **Firing and recharging.** A fire order for a superweapon (S-0213, orders table) is applied with
   the other orders of its tick (S-0203 claim 7). Except for drain-mode superweapons (claim 19) and
   second stages (claim 10), it is carried out only when the house's record for that type is
   granted and ready and its charge clock is running; otherwise it does nothing. Ordinary
   suspension stops that clock, so normally prevents firing. Suspension by itself is not an
   order-level refusal: a ready suspended record whose clock was restarted by claim 14 or a
   one-time grant can accept an already-issued order, although its sidebar does not offer one
   (`quirks.superweapon_order_tests_clock`, default on = original; off: require granted, ready
   and not suspended instead). An order refused because a storm is already pending or raging
   (S-0513 claim 14), or because a domination is in force (S-0612 claim 3), keeps the charge and
   the ready state; the local player gets a refusal message if the order was its own.
   When the order is carried out, it requests the effect (S-0513 claim 3), the record stops being ready, and its
   clock restarts with remaining time *T*, so it becomes ready again in tick *L* + *T* for an order
   applied in tick *L*. The exceptions are one-time records (claim 12), first stages (claim 10) and
   manual-charge types (claim 20). If an ordinary non-first-stage record fires through the
   suspended-but-running bypass, it consumes readiness but does not restart its clock while
   suspended; the existing remaining time continues to run. A non-one-time missile-kind
   superweapon whose house has no building able to
   launch the missile consumes its charge in the same way and launches nothing
   (`quirks.missile_without_launcher_consumes_charge`, default on = original; off: the order is
   refused and the record stays ready). S-0513 claim 7 owns the launcher choice: only the first
   matching silo type in building-type-list order is considered by default. The chosen silo
   must accept the launch: selling prevents it, and an already-running launch sequence need
   not start another missile. A chosen selling or busy silo still records the target and plays
   the launch sounds; with no matching building there are no launch sounds. These cases all
   consume readiness under the original behaviour. The quirk's off state refuses an order
   that would produce no new launch sequence, including the selling and busy cases.
   - Backing: `measurable` (scenario X5) for normal firing, refusal and the missing launcher.
     The exceptional already-issued order on a suspended but running record is `unmeasured`:
     no scenario here reliably produces that order through the original's sidebar
   - Confidence: high; medium for the missile without a launcher

9. **Firing never touches other records.** Apart from the first stage of a two-stage superweapon
   (claim 10), firing one record changes, by the firing itself, no other record of the same house
   or of any other house. An effect that destroys or captures giving buildings acts on the records
   only through the checks of claims 6 and 7, and spy infiltration through claim 15.
   - Backing: `measurable` (scenario X5)
   - Confidence: medium

10. **Two-stage superweapons.** Firing the first stage at a cell under claim 8's order test
    picks that cell as the source and starts the first effect stage (S-0513 claim 33). Sidebar
    targeting additionally requires that it is not suspended (claim 16). It does not
    consume the charge: for a normally completed, unsuspended record, remaining time stays 0 and
    it is ready again in the next house update. The player may pick another source or never go
    on. If claim 14 has changed a ready record to positive remaining time, picking a source
    preserves that time instead. In either case, readiness returns only at an eligible charge
    check under claim 5: the time must have expired and suspension must have ended. One-time first stages follow claim 12. The fire order of the second
    stage is carried out without any check of its own record or of the first stage: it takes the
    first stage's last picked source, starts the second effect stage (S-0513 claim 34), and then sets the
    first stage's record not ready, with its clock restarted with remaining time *T* from that tick.
    Before any source has ever been picked, the source is cell (0, 0), in integer cell coordinates,
    not an absent selection that cancels the effect. The ordinary warp effect uses that coordinate
    and its effect block even when it lies outside the playable area; it does not substitute the
    destination or the giving building. A granted linked first stage restarts from *T* anyway,
    whether it was ready or still charging. With the unchecked-order quirk off, no source selection
    means refusal, with neither effect nor restart. That restart requires the linked first-stage record to be granted. It happens even when that
    first stage is suspended, and its clock then runs through the suspension, so the suspension
    time is not added (`quirks.superweapon_restart_ignores_suspension`, default on = original,
    claim 14 shares it; off: a suspended first stage restarts with its clock stopped at *T*).
    While suspended it does not become ready through clock completion. An
    unchecked second-stage order is accepted in the original
    (`quirks.second_stage_order_unchecked`, default on = original; off: the second-stage order is
    refused unless the first stage is granted, not suspended and has an unconsumed source selection). The second-stage record itself is not made ready or recharged by this. A trigger-granted
    one-time second stage is nevertheless consumed when fired, as in claim 12.

    Picking a source, in our engine, does not repeat the first stage's ready announcement and does
    not restart its flash window (claim 17). The original repeats both in the next house update,
    because it treats the still-expired clock as a new completion.
    - Backing: `measurable` (scenarios X6 and X17) for source retention, initial source,
      completion, restart and repeated announcement. First-stage firing through claim 8's suspension bypass and unchecked
      second-stage orders on a suspended linked first stage are `unmeasured`: suspension cancels
      sidebar targeting, so the proposed normal-input test cannot issue them
    - Confidence: high for the repeated completion announcement; medium for the two-stage contract

11. **Finding the first stage.** The second stage's link field names a superweapon kind (Import
    mapping). The original uses the number of that kind as a position in the list of superweapon
    types (S-0004), and takes the house's record at that position as the first stage, whatever
    type stands there. With stock data the type at that position is of that kind, so the two
    readings agree (`quirks.predependent_kind_as_list_position`, default on = original; off: the
    first stage is the first type in the list whose kind is the named kind). A link outside the
    type list is invalid data: the original does not safely validate it. Our engine refuses the
    second-stage order and warns at import, rather than inventing a source (a departure).
    - Backing: `measurable` (scenario X11 part c, using named test data) for valid links.
      Invalid links are `unmeasured`: their uncontrolled failure gives no reproducible source or
      recharge result to compare
    - Confidence: high

12. **One-time grants.** The trigger action that gives a one-time superweapon (S-0011) grants the
    record as one-time and makes it ready at once, in the tick of the action, with no announcement.
    Its ready tick is that tick. The local player's sidebar gains the icon. A one-time record is
    not suspended, resumed or lost by building and power checks (claims 6 and 7); defeat can
    revoke it (claim 22). When
    it fires (claim 8), it stops being granted and ready and leaves the countdown; its icon goes at
    once (the original leaves the icon on the sidebar, inert, until the house's next recheck).
    One exception: a record that was lost while suspended (claim 7) keeps its suspension, and a
    one-time grant does not end it. That record is ready but suspended, shows the *on hold* label
    and cannot be selected for firing through the sidebar (claim 8). Since neither building
    grants nor power checks change a one-time record, it stays suspended until its house's defeat
    (`quirks.one_time_grant_keeps_stale_suspension`, default on = original; off: a one-time grant
    ends any suspension, as a building grant does). If
    the house has a giving building for the type, the next grant check grants it again in the
    usual way, from the full *T*.

    A one-time grant on a record that is already granted does nothing: a charging record stays
    charging. A building does not change a one-time record either (claim 3).
    - Backing: `measurable` (scenario X7; the stale suspension by X7 part f)
    - Confidence: high; medium for the stale suspension

13. **Permanent trigger grants.** The trigger action that gives a superweapon for good (S-0011)
    grants the record as a building would (claim 3), with no announcement and never suspended at
    the grant, and from then on the record is never suspended, resumed or lost by the checks of
    claims 6 and 7. Only its house's defeat takes it away. If the record was already granted, the
    action only removes it from those checks, so a record that is suspended at that moment stays
    suspended for good (`quirks.permanent_grant_freezes_suspension`, default on = original; off:
    the action also resumes the record).
    - Backing: `measurable` (scenario X7, including part g for frozen suspension)
    - Confidence: high; medium for the frozen suspension

14. **Setting the charge.** The trigger action that sets a superweapon's charge to a percentage
    *p* (0 to 100) acts only on a granted record. It sets the remaining time by Formulas F4 and
    starts the clock running from that tick, even when the record is suspended; the clock then
    runs through the suspension (`quirks.superweapon_restart_ignores_suspension`, default on =
    original; off: keep a suspended clock stopped at the new remaining time). It does not clear
    an existing ready state: a ready record stays ready and gets no new completion announcement,
    despite its changed countdown (`quirks.set_charge_keeps_ready`, default on = original,
    off: clear readiness and let the new clock complete normally).
    For a positive charge time in the bound of F4, the remaining time is at least 1 for every
    *p* in range. A previously charging record therefore becomes ready through claim 5, with its
    announcement, at the earliest in the next tick's house update, and only after suspension ends.
    At *p* = 100 the remaining time is exactly 1 within that bound. Values outside 0 to 100 do nothing.
    - Backing: `measurable` (scenario X7)
    - Confidence: high

15. **Resetting the charge.** Spy infiltration resets only the building owner's record for the
    building type's first superweapon, never its second (S-0510 claim 3, row E4). The trigger
    action that resets a superweapon's charge (S-0011) resets the owner's record for the
    specified type. In either case, the record stops being ready and its remaining time becomes *T*. A running clock restarts from that tick; a stopped clock stays stopped.
    Suspension itself is unchanged: an ordinary suspended record stays stopped at the full *T*,
    but a clock already running through suspension (claim 14) remains running. The reset does not grant a record
    that is not granted.
    - Backing: `measurable` (scenario X8)
    - Confidence: high

16. **Sidebar readiness.** For each of the local player's granted records, the sidebar shows:
    - the clock stage of Formulas F3, from 0 (just started) to 53 while charging, and 54 when
      ready. Drain-mode types follow claim 19. While the record is not ready (and always for
      drain-mode types), the icon shows the charging progress. When it is ready, it does not.
    - the readiness label: *on hold* while suspended (whether ready or not); otherwise *ready* when
      ready; otherwise none. Drain-mode types show *charging*, *ready* or *active* by their state
      (claim 19), except that suspension always shows *on hold*.

    A suspended record whose clock is stopped shows the clock stage of the time it kept. The
    icon offers targeting when granted and not suspended and, for an ordinary type, ready; for
    a drain-mode type, in its *ready* or *active* state. These sidebar conditions differ from
    order application in claim 8's exceptional states. Selecting an icon arms that superweapon
    type, identified by its registration index in the superweapon type list, not by its kind.
    Aim offers follow S-0513 claim 1. A battlefield or radar click instead orders the first type
    in list order whose aim action matches the cursor action, under
    `quirks.superweapon_order_first_type_of_action` (S-0513 claim 2, default on = original,
    off: the order names the selected type). Thus selecting a ready later type can result in
    refusal for an earlier type's unready record, without spending the selected record's charge.
    **Charge overlay and label placement (Q-0742 question 5).** The original uses the same
    imported clock overlay for production and superweapon icons. Its zero-based image frame is
    clock stage + 1, exact integer addition: frames 1 through 54 for a superweapon whose progress
    is visible. It is aligned to the icon's top-left corner, not drawn beside the icon or in the
    battlefield countdown. An ordinary ready record has no overlay, even when suspended; a
    drain-mode record retains it in all three drain states. The readiness label is separate text
    on the icon, using the same centred top position as a production icon's ready label and its
    on-hold label when there is no queue count. For the original's 60 × 48 pixel icon, the label's
    horizontal anchor is 30 pixels from its left edge and its text origin is 1 pixel below its top
    edge. All four label states use that same position and styling; suspension replaces any drain
    state label with *on hold*. The clock overlay is composited after this label where both are
    present. The label is not the superweapon display name, which is drawn near the icon's bottom.

    These original-image and position details are reference presentation facts, not requirements
    to reproduce the original shell: clock artwork, label fonts and positions remain the
    implementer's design under S-0411 claim 8. The visible state and progress rules above still
    apply. The giving building's own charge animation follows the record's remaining time by
    S-0108 claim 18.
    - Backing: `measurable` (scenarios X2, X11 and X14, by labels, icon progress and countdown rows, and
      S-0513 scenario SW1 part d for shared aim actions)
    - Confidence: high

17. **Flash window.** A record is a ready superweapon in S-0411's sense (S-0411 claim 11) while it
    is ready, not suspended (for a drain-mode type: in its *ready* or *active* state), and inside
    its flash window. With the flash setting −1 (its engine default), the window is unlimited.
    With 0, there is no
    window and the record never makes the tab flash; with *F* > 0, the window is the *F* ticks
    starting at the ready tick (claim 5 or 12), so the record counts in ticks *r* to *r* + *F* − 1
    for ready tick *r*. A drain-mode completion does not refresh the ready tick: its finite
    window instead uses its last one-time grant tick, or −100 if there has been none. Presentation only.
    - Backing: `measurable` (scenario X9; drain-mode reference ticks by X11 part a)
    - Confidence: high

18. **Countdown rows.** Each granted record whose type has the countdown flag and whose house is not
    a passive house (S-0412, Terms) has a countdown row, and every player sees every row, whatever
    the house and whatever the alliances. Rows are kept in the order in which their records were granted; a row
    leaves when its record stops being granted, and a new grant adds it at the end. A row shows the
    superweapon's display name in its house's colour and the time *s* = ⌊remaining / 15⌋ in game
    seconds (Formulas F6), as hours, minutes and seconds when *s* is an hour or more and as
    minutes and seconds otherwise. A suspended record's row stays, showing the time it kept. At
    *s* = 0 the time text blinks, changing state once per second of real time; its name stays
    in the house colour. Position, font and blink
    look are the implementer's design. Presentation only.
    - Backing: `measurable` (scenario X2)
    - Confidence: high

19. **Drain-mode superweapons.** A drain-mode record has three states: *charging*, *ready* and
    *active*. It is granted in *charging* with the full *T* (or in *ready*, if one-time). With a
    running clock and no suspension, expiry in *charging* makes it *ready*. This completion
    requests no ready announcement and does not change the ready tick used for flashing.
    A fire order in *ready* starts the kind's launch effect and moves to *active*, with the clock
    running for the active time of F5. An order in *active* moves to *ready*, with the clock
    running for the recharge time of F5, but launches no effect and issues no effect-ending
    action. It may be fired again immediately, for a shorter active time. Expiry in *active*,
    while not suspended, moves to *charging* with the full *T*, without an effect-ending action.
    An order in *charging* does nothing. Neither early return to *ready* nor natural expiry
    ends any running effect. Launch effects retain their own lifetimes (S-0513 claims 19, 26,
    39, 40, 42 and 43); the label *active* does not itself define an effect's lifetime.

    These drain-state order transitions do not test grant, readiness, clock-running or suspension
    again when the order is applied, and restart the clock even if suspended; the sidebar still
    uses claim 16's conditions. A one-time drain record follows these transitions and is not
    consumed by them (`quirks.drain_orders_bypass_normal_checks`, default on = original; off:
    require granted and not suspended, and consume a one-time record on launch).
    The original's computer-player path may launch without changing drain state; our AI player
    houses follow the human rules above. Neutral and special houses retain the original's
    separate rule: they request a launch without a drain-state change when their firestorm state
    is off (S-0211 claim 11), with the kind's own readiness test still applying. No stock skirmish
    event turns that firestorm state on.
    - Backing: `measurable` (scenario X11 part a) for the human state cycle, timings and absence
      of effect-ending actions. Exceptional queued orders and non-player houses are `unmeasured`:
      no normal-input scenario here produces those states and orders
    - Confidence: medium

20. **Manual-charge superweapons.** A manual-charge record is granted with its clock stopped at the
    full *T* and is never resumed by power (claim 6); it charges only when a trigger sets its
    charge (claim 14). Power resumption clears suspension but leaves its clock stopped; it does
    not start automatic charging. After it fires, its clock stops again at the full *T*.
    - Backing: `measurable` (scenario X11 part b, using named test data)
    - Confidence: medium

21. **The charge time.** *T* is the type's imported charge time in game minutes converted to ticks
    by Formulas F1. Absent, and also when the imported value is exactly 0, *T* keeps the value it
    had (the engine default of Constants, or an earlier rules file's value): 0 cannot be set.
    - Backing: `measurable` (scenarios X1 and X12 part e)
    - Confidence: high

22. **Defeat.** When a house is defeated (S-0414), every granted record it has, including one-time
    records and permanent trigger grants, is lost at the next availability check of claim 2
    (a), (b) or (d), and buildings no longer grant it anything. Its countdown rows go with them.
    Defeat alone does not directly revoke these records: until an availability check occurs they
    remain granted. S-0414 owns which houses still get house updates.
    - Backing: `measurable` (scenario X10)
    - Confidence: medium

23. **Aim action and crate selection.** A superweapon type's aim action is imported separately
    from its effect kind; neither implies the other. The nuclear crate's type selection uses the
    nuke aim action, not the missile kind, subject to the separate kind-presence test of S-0520
    claim 21. A newly registered type has no action unless one is imported. An absent action
    keeps the previous layer's action; an unrecognised action word instead sets no action.
    An empty written value also keeps the previous action, initially no action: it supplies no
    entry under S-0001 claim 11. The complete action-word vocabulary is in Import mapping.
    Its entries are distinct actions for matching, including words not normally used by a
    superweapon; recognition does not promise that a battlefield click issues a superweapon
    order for every action. Selecting an otherwise fireable icon with no aim action issues
    that selected type's fire order at cell (0, 0), without entering aiming. Any recognised
    nonzero action instead enters aiming; this alone does not spend the charge. S-0513
    claims 1 and 2 own subsequent aim offers and aimed-click order selection.
    - Backing: `measurable` (scenarios X13 and X16)
    - Confidence: high

24. **Kind and link vocabulary.** The kind and second-stage link use the same twelve-word
    vocabulary in Import mapping, with case-insensitive matching of the complete word. No
    alternate spellings are recognised by inserting or removing hyphens, underscores or internal
    spaces. Ordinary text trimming follows S-0003 claim 1; it is not punctuation removal.
    An unknown word, an omitted field or an empty written value keeps the previous layer's
    kind or link respectively. A new type initially has neither kind nor link. In particular,
    the no-kind description is not a special word that clears an earlier kind. Invalid link
    use remains subject to claim 11.
    - Backing: `measurable` (scenario X15)
    - Confidence: high

## Formulas

Floating-point results follow S-0205 claims 4, 5 and 7: double precision, each arithmetic result
rounded toward zero by default, integer conversions truncated toward zero. Write *Z(x)* for
that double rounding and *trunc(x)* for integer conversion. The shared `quirks.chop_rounding`
controls arithmetic rounding, not integer conversion. Formulas other than F1 assume *T* > 0,
drain formulas also assume *ρ* > 0. Times are integer ticks unless stated otherwise.

- **F1, charge time.** For an imported value *v* in game minutes that is not 0,
  *T* = trunc(*Z(v × 900)*). The input follows S-0003 claims 6 and 7: ordinary decimal text becomes
  a single-precision value widened exactly to a double; a percent form uses that spec's percentage
  conversion. The 900-tick factor is exact. Negative values and conversion overflow are covered
  by Edge cases and S-0205 claim 7.
- **F2, remaining time.** A clock started in tick *s* with remaining time *R* has, in tick
  *n* ≥ *s*, remaining time max(0, *R* − (*n* − *s*)). Stopping it stores that value; a stopped
  clock keeps it. Exact integer arithmetic in range, with no rounding. A clock stays running
  after reaching zero unless an event explicitly stops it.
- **F3, clock stage.** Not granted: 0. Ready (not drain-mode): 54. Otherwise, with remaining time
  *r*: stage = min(53, ⌊54 × (*T* − *r*) / *T*⌋). For a drain-mode record in *active*:
  stage = min(53, ⌊54 × *r* / (*T* × *ρ*)⌋). Drain-mode *charging* and *ready* use the first
  formula, so a fully charged *ready* state shows 53; an early end to *active* instead shows
  partial recharge progress. Our engine evaluates these non-negative ratios exactly over the
  integer times and imported ratio, flooring once. The original uses double arithmetic: its
  charging fraction is *Z((T − r) / T)*, and its active fraction is
  *Z(1 − Z(Z(Z(T × ρ) − r) / Z(T × ρ)))*. It then forms *Z(fraction × 54)*, truncates toward zero
  and caps at 53. At exact boundaries it can show one stage less; this presentation bug has no switch.
- **F4, set charge.** For an integer percentage *p* from 0 to 100,
  *r* = *T* − trunc(*Z(Z(p × c) × T)*), where *c* is the single-precision value nearest 0.01
  (0.0099999997764825820922852…). Integer subtraction is exact in range. For
  1 ≤ *T* < 2²² the floating-point products are exact, *r* ≥ 1, and *p* = 100 gives *r* = 1.
  For larger positive *T*, use the formula rather than assuming a one-tick remainder.
- **F5, drain mode.** With *ρ* the imported game-wide charge-to-drain ratio, a double, and *r*
  the integer remaining time when the order is applied: *ready* → *active* gives active time
  trunc(*Z((T − r) × ρ)*). *Active* → *ready* gives recharge time
  trunc(*Z(T − Z(r / ρ))*). The inner integer difference in the first formula is exact in range,
  each floating-point operation rounds as above, followed by truncation toward zero to whole ticks.
- **F6, countdown seconds.** *s* = ⌊*r* / 15⌋, integers. The divisor is 15 ticks per game second,
  the same unit as F1's 900 = 60 × 15, and does not change with the game speed.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| default charge time | 4500 | ticks | when no rules file sets the charge time (claim 21) |
| ticks per game minute (charge import) | 900 | ticks per minute | F1 |
| top clock stage | 54 | stages | ready; charging stages run 0 to 53 (F3) |
| countdown divisor | 15 | ticks per second | F6 |
| set-charge factor *c* | single-precision 0.01 | — | F4 |
| countdown blink | 1 | second of real time | claim 18 |
| initial flash reference tick | −100 | ticks relative to game start | drain-mode finite window, claim 17 |
| default charge-to-drain ratio | 3 | drain ticks per charge tick | absent from every rules layer |

## State and ordering

States of one record (drain-mode types: claim 19; manual-charge: claim 20).

| State | Event | Result |
|---|---|---|
| not granted | grant check finds a giving building, house has full power | charging, remaining *T* (claim 3) |
| not granted | grant check finds a giving building, house has low power | suspended, remaining *T* (claim 4) |
| not granted | one-time trigger grant | ready, one-time (claim 12); stays suspended if it was lost while suspended |
| not granted | permanent trigger grant | as a building grant, never suspended (claim 13) |
| charging | remaining reaches 0 at a house update | ready, announcement (claim 5) |
| charging or ready | check: not powered, power-dependent | suspended, clock stopped (claim 6) |
| suspended | check: powered | resumed: charging, or ready if it was (claim 6) |
| ordinary building grant | check: no giving building, or barred by the setting | not granted, charge forgotten (claim 7) |
| any granted | defeated house's next availability check | not granted (claim 22) |
| ready | fire order carried out | charging, remaining *T* from that tick (claim 8) |
| ready, one-time | fire order carried out | not granted (claim 12) |
| ready, first stage | fire order | source picked; ready again at the next house update (claim 10) |
| any granted first stage | second-stage fire order | first stage charging, remaining *T* (claims 10, 11) |
| granted | set charge *p* | readiness unchanged, remaining by F4, clock running (claim 14) |
| any | reset | not ready, remaining *T*, clock stopped if it was (claim 15) |

Observable ordering:
- A record that completes in the house update of a recheck that loses it requests its ready
  announcement despite disappearing in that update. But if a change to low power suspends it
  in that update, it does not become ready. A record resumed by a power change in tick *n* keeps
  its saved remaining time for that tick; no elapsed charging tick is credited for the suspension.
- A fire order applied in tick *L* restarts the clock in tick *L*, after every house update of that
  tick, so readiness returns in the house update of tick *L* + *T*.
- A building placed by order in tick *n* grants its superweapon in the house update of tick *n* + 1
  (S-0407 claim 12), so the record is ready in tick *n* + 1 + *T*.

## Edge cases and quirks

- `quirks.superweapon_grant_suspends_by_house_power` (claim 4): at grant, suspension follows house
  low power only.
- `quirks.superweapon_restart_ignores_suspension` (claims 10 and 14, default on = original): a
  second-stage launch and a set-charge action start the clock running even on a suspended record.
  With it off they preserve the stop. Expiry makes a charging record ready at the next eligible
  charge check after resumption, not necessarily in the resumption tick.
- `quirks.superweapon_order_tests_clock` (claim 8).
- `quirks.set_charge_keeps_ready` (claim 14).
- `quirks.drain_orders_bypass_normal_checks` (claim 19).
- `quirks.superweapon_upgrade_second_removal_waits` (claim 2).
- `quirks.predependent_kind_as_list_position` (claim 11).
- `quirks.second_stage_order_unchecked` (claim 10).
- `quirks.missile_without_launcher_consumes_charge` (claim 8).
- `quirks.superweapon_order_first_type_of_action` (claim 16; S-0513 claim 2 owns it).
- `quirks.permanent_grant_freezes_suspension` (claim 13).
- `quirks.one_time_grant_keeps_stale_suspension` (claim 12).
- A one-time record's icon is removed when it fires; the original leaves it until the next
  recheck (claim 12). Presentation only, no switch.
- Picking a first-stage source does not repeat the ready announcement or the flash window; the
  original repeats them (claim 10). Presentation only, no switch.
- The clock stage is computed exactly (F3); the original may show one stage less at exact
  boundaries. Presentation only, no switch.
- A record that is ready and then suspended stays ready but its sidebar cannot offer targeting
  until resumed (claims 6 and 8).
- A negative imported charge time makes a record ready in the first eligible house update after
  its clock starts; its clock stage is not meaningful. Our importer refuses non-positive converted
  charge times, and non-positive drain ratios, with a warning (departure for invalid timing data).
- A building whose type names the same superweapon in both its fields, or two buildings that give
  the same superweapon, give one record: a house never has two of one superweapon.
- Selling or losing a giving building at any charge and rebuilding it starts again from *T*
  (claim 7). Switching it off instead keeps the charge (claim 6).

## Import mapping

| Key (file, section) | Concept | Parse form | Absent |
|---|---|---|---|
| `[SuperWeaponTypes]` (rules) | the list of superweapon types and their positions (S-0004) | list | — |
| `RechargeTime` (rules, superweapon) | charge time *T*, F1 | floating point, game minutes | keeps the previous *T*; engine default 4500 ticks; 0 also keeps it |
| `IsPowered` (rules, superweapon) | power-dependent | boolean | yes |
| `DisableableFromShell` (rules, superweapon) | disableable superweapon (S-0412 owns) | boolean | no |
| `ShowTimer` (rules, superweapon) | countdown flag, claim 18 | boolean | no |
| `FlashSidebarTabFrames` (rules, superweapon) | flash window *F* in ticks, claim 17; −1 means unlimited | integer | −1 |
| `PreClick` (rules, superweapon) | first-stage flag | boolean | no |
| `PostClick` (rules, superweapon) | second-stage flag | boolean | no |
| `PreDependent` (rules, superweapon) | second stage's link, claim 24: the same twelve kind words as `Type`; kind numbers start at 0 in the order below | complete word, case-insensitive; unknown or empty keeps previous link | keeps previous link; initially none |
| `Type` (rules, superweapon) | superweapon kind, claim 24 | complete word, case-insensitive, mapping below; unknown or empty keeps previous kind | keeps previous kind; initially no kind |
| `Action` (rules, superweapon) | aim action, claim 23; complete vocabulary below | complete word, case-insensitive; unknown nonempty word sets no action; empty `Action=` keeps previous action | keeps previous action; initially no action |
| `UseChargeDrain` (rules, superweapon) | drain-mode flag, claim 19 | boolean | no |
| `ManualControl` (rules, superweapon) | manual-charge flag, claim 20 | boolean | no |
| `AuxBuilding` (rules, superweapon) | auxiliary building type for the giving-building test | building type name | none |
| `UIName` (rules, superweapon) | display name of the countdown row | string-table label | — |
| `SidebarImage` (rules, superweapon) | icon art, S-0007 claim 31 | — | — |
| `GCLOCK2.SHP` (image file, no INI key) | shared production and superweapon clock overlay, claim 16 | imported sprite frames, numbered from 0; file resolution follows S-0005 | no alternate overlay established here; artwork is presentation design |
| `TXT_READY` (string table) | ordinary and drain-mode ready label, claim 16 | fixed entry identifier, not literal game text | no alternate entry |
| `TXT_HOLD` (string table) | suspended record's label, overriding ready or drain state, claim 16 | fixed entry identifier | no alternate entry |
| `TXT_CHARGING` (string table) | drain-mode charging label, claim 19 | fixed entry identifier | no alternate entry |
| `TXT_FIRESTORM_ON` (string table) | drain-mode active label, claim 19; the identifier does not restrict it to a particular effect kind | fixed entry identifier | no alternate entry |
| `SuperWeapon`, `SuperWeapon2` (rules, building) | first and second superweapon fields | type name | none |
| `ChargeToDrainRatio` (rules, `[General]`) | *ρ* of F5 | floating point | keeps previous value; initially 3 |
| `MultiplayPassive` (rules, country) | passive house (S-0412 owns), claim 18 | boolean | no |
| `SpecialSound` (rules, superweapon) | effect-specific fading sound, S-0513 claim 32 | sound-entry reference | keeps previous value when absent or unresolved; initially none |
| `StartSound` (rules, superweapon) | effect-specific start sound, S-0513 claim 32 | sound-entry reference | keeps previous value when absent or unresolved; initially none |

Kind words for `Type` and `PreDependent`, in kind-number order (0 to 11):

| Word | Kind |
|---|---|
| `multimissile` | missile |
| `ironcurtain` | iron curtain |
| `lightningstorm` | storm |
| `chronosphere` | chronosphere |
| `chronowarp` | chronosphere second stage |
| `paradrop` | paradrop |
| `amerparadrop` | American paradrop |
| `psychicdominator` | dominator |
| `spyplane` | spy plane |
| `geneticconverter` | genetic converter |
| `forceshield` | force shield |
| `psychicreveal` | psychic reveal |

Both vocabulary tables give literal accepted words in lower case; any case variant is accepted.
The complete accepted `Action` vocabulary follows. Every word denotes a distinct matching action,
except that case variants of the same word are identical. `None` is no action; `Nuke` is the
nuke action used by S-0520 claim 21. Recognition of a word is separate from cursor presentation
and click dispatch (S-0513 claims 1 and 2). Hyphens, underscores and internal spaces are not
ignored. Confidence: high; backing: `measurable` (X16).

| Action word | Action word | Action word |
|---|---|---|
| `none` | `move` | `nomove` |
| `enter` | `self` | `attack` |
| `harvest` | `select` | `toggleselect` |
| `capture` | `eaten` | `repair` |
| `sell` | `sellunit` | `nosell` |
| `norepair` | `sabotage` | `tote` |
| `dontuse2` | `dontuse3` | `nuke` |
| `dontuse4` | `dontuse5` | `dontuse6` |
| `dontuse7` | `dontuse8` | `guardarea` |
| `heal` | `damage` | `grepair` |
| `nodeploy` | `noenter` | `nogrepair` |
| `togglepower` | `notogglepower` | `entertunnel` |
| `noentertunnel` | `ironcurtain` | `lightningstorm` |
| `chronosphere` | `chronowarp` | `paradrop` |
| `placewaypoint` | `tibsunbug` | `enterwaypointmode` |
| `followwaypoint` | `selectwaypoint` | `loopwaypointpath` |
| `dragwaypoint` | `attackwaypoint` | `enterwaypoint` |
| `patrolwaypoint` | `areaattack` | `ivanbomb` |
| `noivanbomb` | `detonate` | `detonateall` |
| `disarmbomb` | `selectnode` | `attacksupport` |
| `placebeacon` | `selectbeacon` | `attackmovenav` |
| `attackmovetar` | `demolish` | `amerparadrop` |
| `psychicdominator` | `spyplane` | `geneticconverter` |
| `forceshield` | `noforceshield` | `airstrike` |
| `psychicreveal` | — | — |

A ready announcement is selected by superweapon kind, under S-0701 claim 28: missile, iron-curtain, storm, chronosphere, paradrop (including American paradrop), dominator, spy-plane, genetic-converter, force-shield and psychic-reveal kinds each request their corresponding kind-specific ready announcement. A chronosphere second-stage type and a type with no kind request no ready announcement.

Readiness labels are the behavioural states *on hold*, *ready*, *charging* and *active*.
The entries above identify the original's imported wording, not text to hard-code. Their wording
is presentation design; the state shown in each circumstance is specified in claim 16 and claim 19.

The two sound-entry references associated with a superweapon each keep their previous value when
absent or unresolved (initially none). Their use does not establish a generic charging or
recharge-completion sound: charge completion uses the kind-based announcement table above.
Effect-specific use of these sound references belongs to S-0513 claim 32. Sound-reference default
and unresolved-name retention are `unmeasured`: no scenario here independently isolates them.
Confidence: high.

## Measurement scenarios

All scenarios use standard skirmish at 800 × 600, game-speed index 2, director and mods off,
superweapon setting on unless stated. Use stock data except for the named test-map overrides
explicitly listed below. A purpose-made small map gives the human player the needed buildings,
plants and a spare consumer for power changes. One bystander computer player starts farthest
from the observed area (the test buildings and target cells); apply its glossary void rule.
Scenarios needing another house's superweapon explicitly permit only those named effects.
Keep enough surviving houses that the game does not end before observation, including X10.
Use a read-only recorder for tick-labelled orders, announcement requests, sidebar labels, icons,
countdown rows and tab flashing. Scripted mouse and keyboard input may time the procedures.
Record the actual grant, suspension and resumption ticks, rather than equating an order tick with
its later check. Three runs per variant unless stated. "*T*" is F1's result for the chosen type.
A run with an ambiguous endpoint is inconclusive and repeated, not passed with a one-tick allowance.
The proposed recorder records observables only; this spec neither changes nor requires changes
to the user's bench. Where automation is unavailable, tick-labelled video plus manual input fits.

**X1: grant to ready (claims 1, 2, 3, 5, 21).**
- *Setup:* a power-dependent type with the countdown flag; the house has full power. Run (a): the
  giving building is on the map from the start. Run (b): it is built and placed by order in tick
  *n*.
- *Observable:* tick of the icon's appearance, tick of the ready announcement request, countdown
  row values.
- *Expected:* (a) ready announcement in tick *T* after the first house update; (b) icon in tick *n*
  + 1, ready in tick *n* + 1 + *T*; no announcement at the grant; the countdown row starts at
  ⌊*T* / 15⌋.
- *Falsified if:* readiness is not exactly *T* ticks after the observed grant, or an announcement
  comes at grant. Repeat at game-speed index 6: the tick duration must be unchanged.
- *Random draws:* use the allowed random-state recorder around the grant. Isolate it from other
  scenario-stream users with X12's otherwise identical no-superweapon control building. A draw
  attributable to the grant falsifies the no-draw claim; interleaving that cannot be separated
  makes this sub-test inconclusive.

**X2: suspension and resumption (claims 6, 16, 18).**
- *Setup:* as X1 (a), with a power-dependent type that has the countdown flag.
- *Procedure:* (a) at tick *T*/3, sell plants so the house has low power for 600 ticks, then
  restore power; (b) switch the giving building off for 600 ticks, full power throughout; (c)
  damage the giving building to half health without changing house power; (d) let it get ready,
  then switch it off, try to fire, switch it on and fire. Repeat (c) while a stock erasing unit
  erases it without completing removal. In another run a stock power-draining unit drains it,
  with spare plants sufficient to prevent low power. Record its switch and house power as controls.
- *Observable:* ready tick against a control run, the countdown row value during the suspension,
  and the label.
- *Expected:* (a) and (b) ready exactly *d* ticks later than the control, where *d* is the
  recorded resumption tick minus suspension tick (aim for 600 ticks), row frozen, label *on hold*.
  (c) ready on the control tick, also during incomplete erasure or draining with full house
  power; (d) no fire while switched off, fire after. A charge delay in either full-power
  special-state run falsifies the corresponding independence rule.
- *Falsified if:* any differs. Also sample the icon at successive stages and at readiness:
  stages must follow F3 except for the documented original boundary error, with 54 when ready.
  In a two-giving-building variant, switching off only one must not stop charge.
- *Countdown extension (claim 18):* grant two countdown types to different non-allied houses at
  separated ticks, then lose and regrant the earlier one. Both rows must be visible locally,
  ordered by their successive grants; the regranted row moves to the end. Observe the final
  30 ticks and ten seconds of real time at zero: displayed seconds follow F6 and blink toggles
  are one real second apart. A private enemy row, wrong order, moving suspended countdown,
  different seconds conversion or tick-paced blinking falsifies that subclaim. Repeat with
  named test map `S0512-countdown.map` overriding one type's charge time to exceed an hour and
  granting it to a passive test house: the long normal row uses hours, while the passive row is absent.

**X3: suspension at grant (claim 4).**
- *Setup:* a stock type that is not power-dependent. Named test map `S0512-grant-power.map`
  enables its countdown row without changing its charge time or power dependence.
- *Procedure:* (a) place the giving building while the house has low power; restore full power
  900 ticks later. (b) Control: place it under full power, then low power for 900 ticks.
- *Expected:* (a) ready exactly *T* ticks after the recorded resumption, row frozen from grant
  until resumption; (b) ready *T* ticks after its own grant. Relative to each grant, (a)'s extra
  delay is exactly its recorded resumption tick minus grant tick. With the switch off our engine
  gives the control time in (a).
- *Falsified if:* (a) shows no delay, or (b) is delayed. Also grant a power-dependent type under
  full power while its giving building is already switched off, using named test map
  `S0512-offline-grant.map` to arrange the initial grant and next unrelated recheck at distinct ticks.
  Arrange the switched-off building on the map before the first grant check, not a permanent
  trigger grant (which bypasses building and power suspension). Its row starts counting at
  grant and stops at the later recheck, not at grant; any other boundary falsifies claim 4's
  switched-off exception.

**X4: loss and regrant (claims 2, 7).**
- *Procedure:* at half charge, sell the only giving building; when it is gone, place a new one.
- *Expected:* icon and row leave at the recheck after the removal; the new building starts the row
  again at ⌊*T* / 15⌋ and readiness comes *T* after the new grant.
- *Falsified if:* charge resumes from half, or loss/regrant occurs at a different check boundary.
- *Upgrade variants:* named test map `S0512-upgrades.map` defines otherwise identical upgrades,
  one giving a type in its first field and one only in its second. Install each on a stock host,
  record the icon's grant tick, then remove it at a recorded tick. Installation grants immediately.
  Removal of the first-field upgrade loses the record immediately; second-only removal waits
  for a later recheck. Keep unrelated rechecks away; an identical loss boundary falsifies claim 2.
  Record row disappearance separately from the original's delayed icon removal.
- *Completion versus loss:* time capture of the sole giving building to a tick whose house update
  both completes the old owner's charge and rechecks its loss. The old owner must request the
  ready announcement despite losing the row. A controlled low-power transition on that tick
  instead prevents completion. A different result falsifies the ordering claim.

**X5: fire and recharge (claims 8, 9).**
- *Setup:* at least two granted types for the human player and one countdown type for the other
  house; all test icons fit below the tab-capacity limit. Record every row, not only the fired type.
- *Procedure:* (a) when ready, fire at a cell away from every giving building at tick *L*; (b)
  a map trigger or the computer house starts a storm, and during it the local player orders its
  own ready storm-kind superweapon; (c) a missile-kind superweapon granted by
  a permanent trigger grant to a house with no launching building, fired.
- *Expected:* (a) ready again at *L* + *T*, other houses' rows unchanged; (b) the local player's
  order refused with a message, its record still ready; (c) nothing launched, row restarts at ⌊*T* / 15⌋.
- *Launcher variants:* repeat (c) with the chosen silo selling and with it still in its launch
  sequence (S-0513 scenario SW2 parts b, c and e). Record sounds, target, missiles and readiness:
  charge is spent and sounds play, but no additional missile need appear. With two matching silo
  types, owning only the later type still gives no launcher under the default first-type rule.
- *Falsified if:* any differs. Repeat (b) for a pending storm and for domination already under
  way; either consuming readiness falsifies refusal. After a source pick, also repeat with a
  ready first stage whose charge was set to 0 %: the pick must preserve the positive countdown
  rather than restore immediate readiness (claims 10 and 14).

**X6: two-stage superweapon (claim 10).**
- *Procedure:* when the first stage is ready, pick a source at tick *a*, cancel targeting, wait
  300 ticks, pick another source and complete the jump at tick *b*. In a separate run complete
  the destination without cancelling. Keep power full and the first stage granted.
- *Observable:* source and destination cells, ready announcement requests, icon readiness,
  countdown, tab flashing and the ready tick after completing the destination. In a named
  test-map variant `S0512-source-flash.map`, give the first stage a finite positive flash window
  and wait until it has ended before picking the source.
- *Expected:* the original repeats the ready announcement at the first house update after *a*.
  The source pick alone leaves zero remaining time and permits another source pick. In the
  finite-window variant, the original restarts flashing at that next house update; our engine
  neither repeats the announcement nor restarts flashing. A different original flash boundary
  falsifies that presentation-bug account. Completing
  the destination restarts the first stage at *T*, with readiness at *b* + *T*.
- *Falsified if:* a source pick consumes charge, another source cannot be picked, completion
  recharges the second stage instead, or the expected announcement or ready tick differs.
  The suspended-second-stage order quirk is not measured by this scenario: suspension cancels
  targeting before that order can normally be given.

**X7: trigger grants and set charge (claims 12, 13, 14).**
- *Setup:* a test map whose triggers, at given ticks, give the local player a one-time superweapon
  of a type it has no building for, a one-time grant of a type that is charging, a permanent grant,
  and set the charge of a charging record to 100 % and, separately, 50 %. Part f: a giving
  building of a power-dependent type is sold while the house has low power, and a one-time grant
  of that type follows. Part g gives a permanent grant to a suspended record, then restores power.
  Part h sets a ready record to 0 %, waits briefly and fires it. Part i sets charge to 50 % while
  suspended, waits more than *T*, and restores power. Part j grants a permanent record while
  it is already charging but not suspended. Part k grants an ordinary one-time record with
  no stale suspension, then cycles house power and removes any giving buildings before firing.
  Use distinct records or fresh games.
- *Expected:* one-time: ready in the tick of the action, no announcement, icon gone after firing,
  not granted again; one-time on a charging record: still charging, row unchanged; permanent grant:
  not suspended by low power; set 100 %: ready, with announcement, in the next tick; set 50 %:
  row at ⌊(*T* − trunc(50 × *c* × *T*)) / 15⌋; part f: ready but *on hold* and not selectable
  for targeting, even after power returns; part g: still suspended after power returns; part h:
  countdown resets but label remains *ready*, firing succeeds and no second completion
  announcement is requested; part i: countdown runs to zero while suspended, but readiness and
  its announcement wait for the first eligible house charge check after resumption. Observe the
  original's icon-removal delay separately from our presentation fix. A consumed one-time type
  with a surviving giving building must instead be regranted at its next grant check at full *T*.
  Part j preserves its prior countdown and ready tick, but subsequent low power or building loss
  does not suspend or revoke it. Part k stays ready, selectable and granted until firing; power
  and building checks must not change it.
- *Falsified if:* any differs. Also apply −1 % and 101 % in named test map
  `S0512-charge-limits.map`: either changing the countdown falsifies the ignored-input rule.

**X8: reset (claim 15).**
- *Procedure:* (a) an enemy spy enters a ready superweapon building; (b) the same while the
  record is suspended by low power, then power restored.
- *Expected:* (a) not ready, row at ⌊*T* / 15⌋ and counting; (b) row at ⌊*T* / 15⌋ and frozen,
  counting after power returns. Repeat with the reset-charge trigger instead of a spy. Also
  reset after X7 part i has restarted a suspended clock: it must continue counting at full *T*,
  while the label stays *on hold*. A reset on an ungranted type must not add an icon or row.
- *First/second variant:* named test map `S0512-spy-reset.map` gives a spyable building two
  distinct superweapon types, with neither radar, positive base power output nor tech-lab-list
  membership. Let both records become ready, then infiltrate it. Only the first type's record
  resets; the second stays ready with its countdown unchanged. In a separate run remove the
  first field, keeping only the second: infiltration must not reset that second record.
- *Falsified if:* any expected countdown, readiness or grant result differs, including any
  infiltration reset of the second type's distinct record.

**X9: flash window (claim 17).**
- *Procedure:* a type with a positive flash setting *F*, and one with the setting absent; let each
  get ready.
- *Expected:* the tab flashes for *F* ticks from the ready tick and then stops while still ready.
  The other flashes until fired. Measured with S-0411 scenario S8's method. Repeat with named
  test map `S0512-flash.map` setting the window to zero: readiness must never cause tab flashing.
- *Falsified if:* a finite window ends on a different tick, an unlimited one ends before firing,
  or the zero-window type flashes because of readiness. Keep all production icons unready.

**X10: defeat (claim 22).**
- *Procedure:* a map trigger gives a computer house a countdown superweapon by permanent grant,
  which only defeat removes; the house is then defeated.
- *Expected:* its row leaves at the first availability check after defeat, including an immediate
  check caused by removal of a superweapon-giving building; before defeat, loss of buildings
  does not remove a permanent grant. Repeat for a one-time grant.
- *Falsified if:* either grant survives an observed availability check after defeat, or disappears
  before defeat merely because its building is lost. A run ending before the check, or in which
  the computer house fires the test record before defeat, is void.

**X11: test-data types (claims 11, 19, 20).**
- *Setup:* named test map `S0512-charge-types.map` overrides (a) an ordinary iron-curtain-kind
  type to use drain mode and a finite flash window, (b) a separate ordinary type to use manual
  charge, and (c) the linked type at the chronosphere kind's list position to a different kind,
  appending a new first-stage chronosphere type. Give both possible first stages by buildings.
  Choose positive test durations, with the curtain effect longer than the short drain interval.
- *Procedure:* (a) wait for readiness, fire, fire again part-way through the drain interval, then
  fire immediately. In a second run let the active interval expire without a second order.
  Repeat using a one-time grant. (b) wait 2 × *T*, cycle low/full power, then set charge to 0 %
  by trigger; after readiness fire and wait another 2 × *T*. (c) charge both candidate first
  stages, pick a source with the appended one, and complete the destination normally.
- *Observable:* labels, clock stages, countdown rows, tab flash, announcement requests and
  protected targets' health under identical shots before and after the early drain-state change.
- *Expected:* (a) readiness has no announcement or newly started finite flash window. *Active*
  and early-return *ready* times follow F5; immediate reactivation uses the partial recharge.
  For a building grant with no earlier one-time grant, its finite window is eligible only in
  ticks earlier than −100 + *F*, regardless of when charging completes. Choose one test window
  that ends before completion and another that extends beyond it; completing, activating and
  returning early must never extend that boundary. For a one-time grant in tick *q*, the boundary
  is instead *q* + *F*. Any refreshed boundary falsifies claim 17's drain-mode rule.
  Natural expiry starts *charging* at full *T*. Neither early return nor expiry ends the curtain
  effect: protection follows its own duration. A one-time drain record remains granted after
  firing. (b) no automatic readiness before setting charge, readiness *T* ticks after setting it,
  then no automatic recharge after firing; power cycling never starts its stopped clock.
  (c) the record at the kind's list position restarts, not the appended chronosphere record.
- *Falsified if:* any differs. Record target health as the effect observable, not just its tint.

**X12: grant dependencies and import (claims 3 and 21).**
- *Setup:* named test map `S0512-grants.map` supplies test buildings and superweapon types with
  otherwise identical data: (a) two giving buildings for the same type, (b) an auxiliary-building
  condition, (c) a giving upgrade with that same condition, (d) disableable and non-disableable
  types, (e) a newly registered type omitting charge time and an existing type overridden with
  zero charge time. Add a no-superweapon control building for X1's random-state comparison.
- *Procedure:* place/remove/capture the buildings and auxiliary building, attach/remove upgrades,
  and force an unrelated recheck after each change. Run (d) in separate games with the setup
  superweapon setting on and off. Grant (e) by trigger and record time to readiness.
- *Observable:* icon count, countdown rows, grant/loss tick and ready tick relative to grant.
- *Expected:* (a) one icon and charge, preserved when only one giving building is lost; capture
  grants the new owner its own full charge at its grant check. (b) no grant until the auxiliary
  building is owned, loss at the applicable check when it is lost. (c) the upgrade grants even
  without that auxiliary building. (d) only disableable building grants are barred by the setting,
  while trigger grants bypass it. (e) omission uses the 4500-tick engine default for the new type,
  and zero retains the prior charge time for the existing type. In a further test use a positive
  fractional imported duration whose product with 900 is not integral: readiness follows F1's
  converted value, not rounding to nearest.
- *Falsified if:* any grant, count, loss or duration differs. Use the recorded checks, not assumed
  immediate changes, for the auxiliary condition.

**X13: crate action mapping and default (claim 23).**
- *Setup:* named test map `S0512-crate-action.map`, following S-0520 scenario CR15, with fixed
  nuclear crates and the crate setting off. Keep a missile-kind type present and give a separate,
  earlier test type a different effect kind. Only that earlier type receives the nuke action.
  Use fresh games and records for each pickup, with no prior grant of the expected type.
- *Procedure:* collect a crate and record which icon becomes ready. Repeat with mixed-case
  spelling of the action word. In separate variants, omit the action on the newly registered
  earlier type, explicitly choose no action, or supply an unrecognised word. Keep a later
  nuke-action type for these controls, avoiding S-0520 claim 21's unsafe missing-action case.
  Finally import the earlier type's nuke action in a rules layer and either omit its action in
  the map layer or replace it with an unrecognised word there.
- *Expected:* the earlier type is granted for either spelling of the nuke action, despite its
  different kind. Omission on a new type, explicit no action and an unrecognised word leave
  that type unmatched, so the later type is granted. Layer omission preserves the earlier
  type's nuke action; an unrecognised map-layer word clears it and selects the later type.
- *Falsified if:* the kind alone determines the granted type, matching is case-sensitive, a
  new omitted action matches nuke, omission clears an earlier action, or an unrecognised
  word retains it. Record sidebar readiness and consumption under S-0520 scenario CR15.

**X14: shared clock and label placement (claim 16; Q-0742 question 5).**
- *Setup:* use X1's stock superweapon and an unfinished production item. For drain states use
  X11's explicitly named test-map override; no stock type exercises drain mode. Use the common
  skirmish settings above, and repeat three times for each state.
- *Procedure:* record tick-labelled close-ups of production and superweapon icons at several
  matching progress values, then of the superweapon when ready, suspended while charging,
  suspended while ready, and in each drain state. In separate fresh program sessions, use a
  named local presentation-only probe string table replacing each of the four entries in Import
  mapping with distinct short test text; retain other entries. No rules changes beyond X11's
  named override are needed.
- *Observable:* overlay frame outlines relative to icon bounds, the label's text origin and
  alignment, which test label appears, and whether an overlay remains. Compare to the user's
  imported clock image, not to a newly drawn approximation.
- *Expected:* the same overlay image and stage-plus-one frame selection for production and
  superweapon progress; superweapon frames follow F3. Ordinary readiness removes the overlay,
  including while suspended. Drain states retain it. Labels use the top-centred position of
  claim 16, with the corresponding imported test text; suspension always selects the suspended
  label. The production queue-count exception does not move a superweapon label.
- *Falsified if:* a different clock image or frame mapping is required, ordinary readiness keeps
  its overlay, a drain state loses it, or any label's selected entry or original position differs.
  Record the original F3 boundary error separately from our exact-stage display rule. These
  captures check the original reference look; they do not impose its shell layout on our engine.

**X15: kind and link words (claim 24; Q-0730).**
- *Setup:* named test map `S0512-kind-words.map` and named local rules layers
  `S0512-kind-base.ini` and `S0512-kind-override.ini`. Use the common skirmish settings,
  positive charge times and independently identifiable countdown records. The base supplies a
  known kind and valid link; each override changes just one word from Import mapping. Keep
  links within the type list. Include newly registered types with no base kind or link.
- *Procedure:* test each of the twelve literal words, its lower-case and mixed-case versions,
  and variants with inserted hyphens, underscores or internal spaces. Also test omission, an
  empty written value, an unrelated word and the explicit no-action word as a kind or link.
  Grant and charge the type, then fire. For links, grant the possible linked first stages and
  fire a one-time second stage, recording which first-stage countdown restarts.
- *Observable:* ready-announcement kind, launched effect, and which countdown restarts.
- *Expected:* literal words and case variants choose the table's kind or link. Every nonmatching
  spelling, omission or empty value preserves the base choice; without a base choice it leaves
  no kind or link. Do not execute an invalid-link order in the latter case: claim 11 specifies
  the safe departure, not a reproducible original outcome. For a newly registered no-kind type,
  a grant can still charge but produces no kind-specific ready announcement or launch effect.
- *Falsified if:* any accepted spelling selects a different kind, punctuation variants are
  accepted as aliases, or unknown/empty input clears or replaces the base choice. Keep records
  independent, so the first-action-match rule cannot obscure which type was fired.

**X16: complete action vocabulary and empty values (claim 23; Q-0731).**
- *Setup:* named test map `S0512-action-words.map`, using the common settings. In fresh games
  override a single ordinary, non-first-stage curtain-kind type's aim action with each word in
  the complete Import mapping table. Give it a one-time grant and a visible icon. Do not change
  the effect kind when changing the action. No other selected units may intercept aiming.
- *Procedure:* select the ready icon, record whether it enters aiming without firing, then
  cancel without a battlefield click. Repeat with case variants and an unrelated word. For
  layering, first give the type a nuke aim action in a named base layer, then omit its action,
  write an empty value, write a whitespace-only value, or use the explicit no-action word in
  the map layer. Repeat the empty cases on a newly registered type with no earlier action.
- *Observable:* targeting mode, whether selecting the icon immediately consumes the one-time
  record, and X13's crate selection for the layered nuke-action control. Record aim offers
  with the read-only recorder; do not infer word recognition solely from cursor artwork.
- *Expected:* every listed word other than no action enters aiming and leaves the record
  unspent until an order is issued. Explicit no action and unknown nonempty words instead
  issue the no-aim fire order of claim 23, at cell (0, 0). Case variants are identical.
  Empty and whitespace-only values behave exactly like omission: they preserve the base
  nuke action and X13's earlier crate match, or remain no action on a new type. They do not
  clear an existing action. Different listed words are separate matches under S-0513 claim 2,
  and for actions dispatched as superweapon orders, repeat its SW1 part d with two shared-action
  types and a different-action control.
- *Falsified if:* a listed nonzero action behaves as no action, an unknown word is retained as
  a new action, case changes recognition, or an empty value clears the inherited action.

**X17: second stage before any source pick (claim 10; Q-0737).**
- *Setup:* named test map `S0512-no-source.map`, common settings, no earlier first-stage order.
  Permanently grant the linked first stage and give it a visible countdown. Give the second
  stage a one-time grant and a visible icon using an existing imported icon reference, with
  its ordinary warp aim action and valid link. Put identifiable mobile objects around the
  first stage's giving building and around a chosen destination, away from cell (0, 0).
- *Procedure:* fire only the second stage at recorded tick *L*. Run once while the granted
  first stage is still charging and once after it is ready. As controls, fire after an ordinary
  source pick, and run with the first stage never granted. Keep power full.
- *Observable:* first-stage countdown and readiness, destination effects, radar-event cell
  coordinates, and any objects moved or killed. Use the read-only recorder for the source
  radar event when its cell is outside the visible map; video alone cannot identify that cell.
- *Expected:* without a pick the source event uses cell (0, 0); the warp is attempted under
  S-0513 claim 34, not suppressed for lack of a pick. Units merely near the giving building
  are not substituted as recipients. In both granted variants the first stage becomes not
  ready and restarts at *T* in tick *L*, becoming ready at *L* + *T*. The never-granted
  control does not acquire a charge or become granted. With the unchecked-order quirk off,
  our engine refuses the no-pick order and changes neither effects nor charge.
- *Falsified if:* no-pick alone suppresses the destination effect or restart, a different source
  coordinate is used, or the granted first stage preserves its old remaining time. An empty
  default source block producing no moved units is not evidence that the order was refused.

## Open questions

- The completion-versus-loss announcement is traced, but X4 still needs tight timing to confirm
  it independently. Record announcement requests rather than assuming every request is audible,
  since S-0701 claim 28 can suppress playback.
- The order-only suspension bypasses in claims 8, 10 and 19 have no normal-input measurement
  here. X6 tests the ordinary two-stage flow, not those bypasses.
- Which houses still get house updates after defeat, and so how soon claim 22 applies, is S-0414's.
  The claim states "the next check".
- Battle-lab force-shield availability follows claim 3's imported-building grant rule.
  S-0513 claim 31 describes the effect and scenario SW16 proposes its test. There is no separate
  unresolved effect-scope gate.
