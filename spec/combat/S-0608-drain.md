---
id: S-0608
title: Drain (attaching to a building, draining its power and credits, and letting go)
topic: combat
revision: 6
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-19
measurements: []
requests: [Q-1440, Q-1441, Q-1442, Q-1443, Q-1444, Q-1507]
supersedes: []
---

# S-0608: Drain (attaching to a building, draining its power and credits, and letting go)

## Summary

A unit with a draining weapon that stands over an enemy building can attach itself to it. While
attached, the building is *being drained*, and a building of a resource-destination type pays
credits to the drainer's house at a fixed cadence. This spec owns:

- the drain attempt and what it does instead of an ordinary shot,
- where the drainer must be for the link to form, and that a link is one drainer to one building,
- what happens at the moment the link forms,
- the credit payouts: cadence, amount, insufficient funds, payer and recipient,
- every way the link ends, and what is restored when it does.

Skirmish only. Network games and campaigns are out of scope. This is not the charge-drain mode of
some superweapons (S-0512 claim 19), which shares only the word.

Rules owned elsewhere and not restated here:

- **Choosing the draining weapon.** S-0308 claim 3, table P row 8.
- **Getting over the building.** A balloon-hover type whose chosen weapon has a vertical
  projectile takes the target object as its destination: S-0215 claim 37 h, with S-0307 claim 18.
  The Disc's flight is S-0502 claims 5 to 17. Whether it may fire while moving is S-0310.
- **What being drained does to the building and its house.**
  - The house's drain blackout and the recheck of its power: S-0405 claims 5 and 7.
  - The building's own firing answer (*illegal* while drained), so drained defences cannot fire:
    S-0405 claim 9, table T1. Every consequence of the blackout for the house's other buildings
    is S-0405 claims 8 and 9.
  - A drained building cannot be sold: S-0408 claim 3. Repair eligibility does not test the
    drain: S-0408 claim 14.
  - The special-outage animation slots: S-0108 claim 14.
  - A drained building cannot support a linked weapon: S-0518 claim 2.
- **Other rules that test the draining state of the drainer.**
  - No ordinary approach while draining: S-0215 claim 37 d.
  - Retaliation of the neutral and special houses while draining: S-0309 claim 25 h.
  - Fire-once weapons drop their target: S-0309 claim 21 (the drain attempt drops it anyway,
    claim 1 below).
- **Release of controlled units.** S-0601 claims 9 and 10 own the release effects and order.
  Claim 5 below owns when a drain triggers it.
- **The Disc's laser.** S-0301 claims 8 and 10 own its firing exceptions. Its visible effect is
  not part of this spec.

## Terms

From the glossary and other specs: *being drained* (glossary; S-0405 Terms), *draining weapon*
(S-0308 table P row 8), *tick number* (S-0201), *turn* in the sense of an object's share of a tick
(glossary; S-0203 Terms), *available credits* and *credit balance* (glossary; S-0404 claim 4),
*alliance* (one-way; glossary), *killing hit* (S-0302 claim 14), *foundation* (S-0406),
*controller* and *victim* (S-0601 Terms), *being erased* (glossary; S-0514).

New terms:

- **Drainable type.** A type whose drainable flag is set (Import mapping).
- **Resource-destination type.** A type whose resource-destination flag is set (Import mapping).
  The same flag as in S-0308 table P row 6.
- **Drain attempt.** What a shot with a draining weapon does instead of an ordinary shot (claim 1).
- **Drainer, drain link.** A *drain link* joins one object, the *drainer*, to one building, which
  is then *being drained*. An object is *draining* while it is the drainer of a link.
- **Payout.** One transfer of credits from the drained building's house to the drainer's house
  (claim 7).
- **Payout period *P*, payout amount *A*.** Two game-wide integers (Import mapping): *P* in ticks,
  *A* in credits.
- **Position check.** The test of claim 12 by which a drainer that is a vehicle lets go of a
  building it no longer stands over.
- **Drain animation.** The game-wide animation shown on a drainer while it drains (claim 5).

## Behaviour

### The drain attempt and the link

1. **A drain attempt replaces the shot.** When a firer makes a shot with a draining weapon (the
   firing cycle and its check are S-0301's), no ordinary shot happens: no projectile, no shot damage,
   no warhead effect and no ordinary-shot random draws. The attempt neither starts the rearm
   countdown, nor advances the burst, nor uses ordinary per-shot ammunition. Clearing its target
   still resets the burst counter to 0 (S-0301 claim 4). Aircraft run bookkeeping is separate
   (claim 16). Link-formation effects (claim 5),
   including animation creation and release of controlled units, retain their own rules and may
   draw randomness. Instead:
   - against a target that is not an object (a cell), nothing at all happens and the firer keeps
     its target,
   - against an object whose type is not drainable, nothing happens and the firer keeps its target
     (the firing check normally refuses this case first, claim 3),
   - against an object of a drainable type, the firer's target is cleared whether or not a link
     forms (claim 2 decides the link) and whatever the weapon's fire-once flag says. The destination is not touched.

   Because the countdown does not start, a later drain attempt is limited only by the firing
   check, not by the weapon's rate of fire.
   - Backing: `measurable` (scenario D1, parts a, c and d; D2 part b). The absence of
     ordinary-shot random draws is checked with the random-stream recorder (D1 part a).
     The redundant non-drainable guard inside the attempt is `unmeasured`: the earlier firing
     refusal prevents an ordinary order from isolating it (D1 part d).
   - Confidence: high (no shot, target cleared on every attempt against a drainable object),
     medium (no countdown and no ammunition use, the cell target)

2. **Where the drainer must be.** A drain attempt forms a link only if the cell that contains the
   drainer's position (its horizontal position; height does not matter) is a cell of the target
   building's foundation. Any cell of the foundation will do. Otherwise the attempt forms no link
   and changes nothing but the firer's target (claim 1). It follows that only buildings can be
   drained: an infantry unit, vehicle or aircraft of a drainable type never gets a link, even
   though the firing check allows the attempt. A drainer that fires from a neighbouring cell (its
   weapon's range allows that) loses its target without draining.
   - Backing: `measurable` (scenario D2)
   - Confidence: high (the foundation rule), medium (that an attempt from a neighbouring cell can
     happen at all with stock movement: it depends on S-0215 claim 37 h and the firing timing of
     S-0310)

3. **What the firing check refuses.** In addition to S-0301 claim 9's other answers, the firing
   check answers *illegal* (rank 4) when:
   - (a) the target is the building that the firer itself drains, whatever the weapon,
   - (b) the weapon is a draining weapon and the target is an object that is already being drained,
     by any drainer,
   - (c) the weapon is a draining weapon and the target is an object whose type is not drainable.

   Protection of the target adds no refusal here for player houses (claim 19).
   Parts b and c do not apply to a target that is not an object. Neither the firing check nor the
   drain attempt tests alliance or ownership: the only alliance test of the whole mechanism before
   a link forms is the weapon choice of S-0308 table P row 8, which picks the draining weapon only
   when the firer is draining nothing, the target's type is drainable and the firer's house does
   not ally the target's house. That row does not test whether the target is already being
   drained, so a second drainer chooses its draining weapon against a building that another
   drainer holds, and then gets *illegal* here.

   Only the drainer's current building is refused by part a. A stranded earlier building
   (Edge cases) is not refused by part a with either weapon. A draining weapon is nevertheless
   refused against that earlier building by part b. Its other weapon may fire there if the
   remaining checks pass.

   **Precedence relevant to drain.** Both drain refusals give the same *illegal* answer, but
   they occupy different places among S-0301 claim 9's rank-4 answers:
   - The own-current-building refusal precedes the held-temporal-target *rearming* answer,
     hidden-target *impossible*, absence-of-weapon and power-sensitive-weapon *impossible*,
     the other weapon's live-effect *impossible*, and the conditional magnetic-wave
     *rearming* answer. Materialisation and the own-gripped-unit *rearming* answers precede it.
   - The draining-weapon refusals follow the held-temporal-target, hidden-target,
     absence-of-weapon and power-sensitive-weapon answers. They precede the berserk-warhead
     target refusals, projectile eligibility, conditional magnetic-wave refusal, other-weapon
     live-effect refusal, and empty-spawner *rearming* answer.
   - The berserk firer's berserk-friendly target refusal precedes the draining-weapon refusals,
     but not the own-current-building refusal. The takeover test is later still, at rank 9,
     after rearm, ammunition and cloak. No rank numbered below 4 is interleaved between
     the drain refusals. Higher-priority kind-specific answers retain S-0301 claim 9's ranks.

   This fixes the answer when a drain refusal competes with a differently named answer.
   it does not require an internal order between reasons that all give *illegal*.
   - Backing: `measurable` (D3, D9 part a, D11). Relative ordering of reasons that all answer
     *illegal* is `unmeasured`: the answer and its consequences cannot distinguish them.
   - Confidence: high

4. **One to one.** By claim 3 and S-0308 table P row 8, a building is drained by at most one
   drainer and, with stock data, a drainer drains at most one building. A drainer that is draining
   uses its other weapon against every target (the row does not apply), so ordering it to attack
   another building does not move the link. The one exception needs data that lets the weapon choice
   pick the draining weapon for a firer that is still draining (quirk
   `quirks.drain_relink_strands_previous`, Edge cases).
   - Backing: `measurable` (scenario D3 part c; D9 part a for the relinking quirk)
   - Confidence: high

5. **When the link forms.** At the drain attempt that forms a link:
   - the building is being drained from that moment, and its house is due to work out its power
     (S-0405 claim 7),
   - if the building is a controller (S-0601 Terms), it releases all its victims at once, with
     S-0601 claims 9 and 10's effects and order. This happens only when a link actually forms,
     not at an attempt that fails claim 2,
   - the drain animation starts attached to the drainer and follows it while the animation
     remains alive. S-0109 claims 3, 7, 13 and 14 own its playback, random rate and animation
     effects. Ending the animation's playback does not end the drain link. With no drain
     animation in the data, nothing is shown,
   - in the original a drainer that belongs to a scripted team leaves it (the original's
     computer-player teams; our engine has none, glossary *team member*).
   - Backing: `measurable` (scenario D4 for power, release and the animation, including playback
     ending before the link); `unmeasured` for
     the team departure (out of scope for our engine)
   - Confidence: high

6. **What the drainer does once linked.** The drainer has no target after the attempt (claim 1),
   so its *attack* behaviour falls back (S-0215 claim 23). A stock Disc that has completed its
   approach normally stays over the building. This is not a restraint imposed by the link:
   the attempt does not itself clear a pre-existing destination (claim 1). While draining it
   may fire its other weapon at objects in range (claim 4), it takes no ordinary approach step (S-0215 claim 37 d), and an
   attack order on the building it drains is refused by the firing check (claim 3 a). Movement
   that takes a vehicle drainer off the foundation is noticed at claim 12's next position check,
   not necessarily at the movement itself.
   - Backing: `measurable` (scenario D1, part b; D5)
   - Confidence: medium

### Credit payouts

7. **Who pays, when, how much.** While a building of a resource-destination type is being drained,
   it makes a payout in its own turn of every tick whose tick number is a multiple of |*P*|
   (Formulas F1). The payer is the building's house at that moment and the recipient is the
   drainer's house at that moment. The amount *t* is F2: the payout amount *A*, but never more than
   the payer's available credits. The payer's credit balance falls by *t* and the recipient's rises
   by *t*, in the same payout. Nothing else happens: no message, no sound, no score (S-0414
   claim 22 has no row for it), no harvest. A building that is not of a resource-destination type
   pays nothing, even when it is drainable. Power and payouts are independent: a type that has both
   output and the resource-destination flag gives both effects.
   - Backing: `measurable` (scenario D6 parts a, b, e and f; D7 for live owners).
     Same-house payouts and overflow are `unmeasured`: no setup here isolates a payout
     between owner change and detach or reaches the credit limits without unrelated overflow.
   - Confidence: high

8. **Cadence is tied to the game clock, not to the link.** The payout ticks are fixed by the tick
   number alone. With no skipped building turns or claim 10 suppression, the first payout of
   a continuing link comes 0 to |*P*| ticks after formation. The upper endpoint occurs when
   formation falls on a payout tick after that building's payout opportunity. A link that lasts
   less than |*P*| ticks may pay nothing or once. A payout can fall on the formation tick only
   if the building's payout opportunity follows formation (S-0203 claim 4). A payout on a tick
   in which the link ends occurs only if the payout opportunity precedes that end.
   - Backing: `measurable` (scenario D6, part c)
   - Confidence: high (alignment to the tick number), low (the payout on the forming or ending
     tick)

9. **Insufficient funds.** When the payer's available credits are below *A* and above 0, the payout
   takes all of them. When they are 0, the payout moves nothing, and the link continues as before.
   No payout is ever refused or postponed, and none is made up later.
   This describes non-negative *A* and available credits. F2 also covers negative data.
   - Backing: `measurable` (scenario D6 parts d and f)
   - Confidence: high

10. **Payouts skip a building in the middle of erasure or materialisation.** A drained building
    that is being erased (S-0514), or that is materialising after a chronoshift, makes no payout in
    that tick. It keeps its link. Being switched off, low power, not being in operation and
    damage do not stop payouts.
    - Backing: `measurable` (scenario D6, part e); `unmeasured` for materialisation (a building
      cannot be chronoshifted in a skirmish)
    - Confidence: medium

### How the link ends

11. **The building's house allies the drainer's house.** In every ordinary turn of the drainer
    that reaches the alliance test (claim 15), the link ends if the building's house allies the drainer's house, including when both are the same house.
    The alliance is read in that direction only (glossary, *alliance* is one-way). Ownership
    changes that lead to it, all read at the drainer's next eligible alliance test:
    - the building is captured by the drainer's house, or by any other house that allies the
      drainer's house (S-0509),
    - the drainer is taken over by the building's house or by a house the building's house allies
      (S-0601), or dominated likewise (S-0612 claim 14).

    An ownership change of either party to a house that does not ally the other party leaves the
    link in place: the building's new house then pays (claim 7), the drainer's new house receives,
    and the building's new house gets the drain blackout (S-0405 claims 5 and 7). The weapon-choice
    test of S-0308 runs in the opposite direction (the firer's house allying the target's), so a
    link can form against a building whose house allies the drainer's house without the drainer's
    house allying it back. It then ends at the drainer's next eligible alliance test, and
    the release of claim 5 has already happened.
    - Backing: `measurable` (scenario D7, parts a to c); `unmeasured` for the one-way case and its
      release (a skirmish lobby makes only mutual team alliances, so no setup reaches it)
    - Confidence: high (the test and its direction), medium (the one-way case)

12. **The drainer leaves the building (position check).** A drainer that is a vehicle performs the
    position check in its turn of every tick whose tick number *n* satisfies *n* mod 16 = 2
    (Formulas F3): if the building whose foundation covers the cell containing its position is not
    the building it drains (or there is none), the link ends. The link therefore survives for up to
    16 ticks after the drainer has moved off the building, and the blackout and the payouts go on
    meanwhile. Moving within the building's foundation keeps the link. Any cause of movement counts
    the same way: a move order, a magnetic lift (S-0607), or anything else that displaces the
    drainer. Being disabled by an electromagnetic pulse does not end the link and does not stop the
    check. A drainer that is not a vehicle never performs the check (quirk
    `quirks.drain_position_check_vehicles_only`, Edge cases).
    Skipped turns can delay a check further (S-0203 claim 10). The 16-tick bound assumes the
    drainer continues to receive a turn every tick.
    - Backing: `measurable` (scenario D5; D9 part b for aircraft drainers). The infantry-only
      part is `unmeasured`: no legal infantry-over-building link setup has been established.
      Pulse-disabled drainers are `unmeasured`: no independent stock skirmish pulse delivery
      has been established.
    - Confidence: high (the period and phase, the vehicle rule), medium (pulse-disabled drainers)

13. **Either party is destroyed or removed.** The link that involves it ends at once, except that a
    first link stranded by the second-link quirk (Edge cases) does not end at its drainer's killing
    hit: it ends when the drainer is removed. When the stranded first building receives its killing
    hit or is removed, that link ends at once, and the drainer's newer link ends with it:
    - when the drainer or the building receives its killing hit (S-0302 claim 14), before the
      death effects that follow it,
    - when the drainer or the building is removed from the game in any other way (for example a
      building removed at the end of a sale it had started before the drain, or an object erased by
      a temporal weapon, S-0514).
    - Backing: `measurable` (scenario D8; D9 part a for the stranded first link)
    - Confidence: high (killing hit and removal, and the stranded state of Edge cases). Medium for
      the stranded first building's destruction interactions, as Edge cases gives them.
      Temporary departures without destruction are covered by claim 17.

14. **What ending the link restores.** When a link ends, for any cause:
    - the building is no longer being drained, and its house is due to work out its power.
      At the next power update this link no longer causes a blackout; other drained buildings
      or a timed blackout may still keep the house blacked out (S-0405 claims 5 and 7).
      The building's own firing answer, sale and special-outage state return to their ordinary rules,
    - the drain animation is removed,
    - the drainer is no longer draining: S-0308 table P row 8 can choose its draining weapon again
      and claim 3 a no longer refuses the building.

    Nothing else is restored: credits already paid stay paid, released victims stay free, and the
    drainer does not get the building back as a target.
    - Backing: `measurable` (scenarios D5, D7 and D8)
    - Confidence: high

15. **Turn boundaries (Q-1440).** These are local observable constraints, not shared drain phases:
    - A vehicle's position check samples its position before that turn's behaviour start,
      due behaviour processing, ordinary movement and separate firing. Movement later in that
      turn cannot make the earlier sample fail. A link formed by that turn's firing therefore
      is not position-checked again in the same turn.
    - The alliance test precedes due behaviour processing, its shots, ordinary movement,
      separate firing and the controller's overload update. It does not re-test a link newly
      formed by a later shot in that turn. An alliance-ending link is already gone when those
      later opportunities ask whether the unit is draining.
    - In the original, a vehicle still performs its position check before its erasure or
      warp-only early exit. That exit skips the alliance test. An erased building skips
      its payout; it does not pay and then end its turn. No general rule that an erased
      drainer runs the alliance test is implied. Our engine's superweapon warp-only stages
      instead follow S-0513 claim 51's exclusion of ordinary work, including these checks.
      This does not settle the original's unresolved continuation in other warp stages
      (S-0513 claim 52).
    - A building's payout follows its early operation-state refresh and its due periodic
      income, but precedes due behaviour processing, including behaviour-issued firing or
      sale progression, and the later health-change power recheck. A pending behaviour may
      already have started before the payout. Thus periodic income available in that turn
      can be taken by the payout, and credits paid before later sale removal stay paid.
      Capture or erasure by an earlier object's action is already seen by the payout, not
      deferred until after it. A building erased at its payout opportunity pays nothing.
    - House power recalculation is not part of the building's turn: S-0405 claim 7 and
      S-0203 claim 8 put it after the turn pass. A link ended by a position check in that
      pass has already ended for that same tick's house power update. It is the drained
      building owner's update that matters, not necessarily the drainer owner's update.
    - Backing: `measurable` (D10); the original's other warp-stage continuation remains
      `unmeasured` for S-0513 claim 52's reason. The warp-only exclusion is the cited
      engine contract, not a new claim about those original stages.
    - Confidence: high for the traced local boundaries and house-update consequence

16. **Attempt side effects and precedence (Q-1442).** A drain attempt does not record a new most
    recent shot, perform the ordinary firing reveal to the target's house, or itself uncloak
    the firer. This holds whether or not it forms a link. Ordinary firing-check behaviour
    still applies: a weapon requiring a visible firer can receive *cloaked* and cause the
    behaviour to start uncloaking before any attempt is admitted (S-0301 claim 12 and
    S-0516 claim 11). A draining weapon allowed to fire cloaked does not itself break that cloak.

    If the same weapon is also a suicide weapon, the suicide effect replaces the drain attempt:
    it applies the suicide hit to its firer, without attempting attachment, drain-specific
    target clearing or an ordinary shot. The resulting death effects belong to S-0312 claims
    1 and 4 to 7. Protection from the suicide hit does not make it proceed to draining. Death and hit effects keep their own rules.

    An aircraft's admitted initial firing pass can still make its run's round pending
    (S-0501 claim 15), even though the drain attempt makes no ordinary shot and spends
    no per-shot round. The attempt is not a reason to roll back a pass or its pending-round
    mark. When an attempt clears the object target of an aircraft that remains in play,
    S-0501 claim 12's departure cancels the remaining burst attempts and that pass's scatter
    request, and enters *done* with a 1-tick processing delay. It preserves any formed link
    and pending round, which is spent at S-0501 claim 15's ordinary boundary. It neither
    restores the target nor itself clears the destination or starts rearm. Cell-target
    attempts keep their target and use the normal pass transitions. Other attack-run
    transitions and later round spending remain S-0501 claims 12 to 17's rules.
    - Backing: `measurable` (D12) for aircraft bookkeeping through a cell-target attempt.
      The original object-target continuation fault is `unmeasured`: no destructive crash
      test is proposed. Safe object-target continuation is S-0501 claim 12's design departure,
      not an oracle outcome; S-0501 claim 12's engine-only scenario checks it.
    - Confidence: high (no recent-shot record, reveal, direct uncloak and suicide precedence),
      medium (aircraft composition with target clearing)

17. **Temporary storage ends the link (Q-1443).** Successfully taking either party off the map
    through ordinary temporary storage ends the link immediately, with claim 14's effects,
    not at its next turn or on return. This includes boarding a transport: no continuing
    blackout cause, payout, sale refusal or firing refusal from that link survives while the
    drainer is stored. Already-kept house power totals still wait for claim 14's power update. Returning does not restore it; a new drain attempt is needed. This also clears
    a stranded earlier building's drain when its drainer leaves the map. Failed boarding
    or an unsuccessful removal does not count. Being lifted while still on the map is not
    storage and retains claim 12's position-check rule.

    The same departure rule applies to a drained building if a rule successfully takes it
    off the map without destroying it; no reachable skirmish storage route for such a
    building is established here. Conversion that destroys or removes the original object
    already falls under claim 13, not a preserved link to its replacement.
    - Backing: `measurable` (D13, transport fixture). Temporary building storage is
      `unmeasured`: no legal skirmish building carrier or storage route is established.
    - Confidence: high for ordinary successful off-map departure; medium for composing it
      with each feature's boarding eligibility

18. **Occupants and entrants (Q-1444).** Being drained adds no occupant-specific gate to a tank
    bunker, infantry absorber or grinder:
    - A bunkered vehicle keeps its own firing check; the drained bunker's *illegal* answer
      is not forwarded to it. Its bunker rate-of-fire, damage and range changes stay in
      force (S-0606 claim 9). This is not immunity to independent loss of power or disability.
    - Bunker entry and release retain S-0606 claims 2, 3, 5, 6 and 12's ordinary conditions.
    - Absorber entry, holding and voluntary release retain S-0605 claims 1, 7, 9 and 11 to 13's rules.
    - Grinder entry, consumption and recycling credit retain S-0603 claims 5 and 8's rules.
      Recycling credit goes to the grinder's owner, not directly to the drainer. A later
      ordinary payout can take those credits only under claim 7.

    Starting a link neither ejects nor consumes these occupants, ends their bunker link,
    nor clears an approaching entrant's contact solely because of draining. Other effects
    still apply independently: in particular claim 5 releases controlled victims, and
    S-0405 claims 5, 8 and 9 own any house blackout and operation changes. There is no
    further direct drain modifier to occupant capacity, owner, bonuses or recycling value.
    - Backing: `measurable` (D14)
    - Confidence: high for absence of a direct drain gate in the named interactions,
      medium for combined blackout effects, whose eligibility remains with their owners

19. **Protection does not stop a drain (Q-1507).** Neither protection kind of S-0513 (the
    invulnerability effect or a force shield) plays any part in draining, for the drained building
    or for the drainer. A curtain that reaches an infantry drainer or an organic-vehicle drainer
    hits it instead of protecting it (S-0513 claims 23 and 24); a killing hit from it ends the link
    under claim 13.
    - **Forming.** A drain attempt against a protected building forms the link exactly as against
      an unprotected one (claims 1, 2 and 5): the building is being drained, its house gets the
      drain blackout when the building has output (S-0405 claim 5), controlled victims are released
      and payouts follow claim 7. The weapon choice of S-0308 table P row 8 and the drain refusals of
      claim 3 do not test protection, and the attempt never fails because of it. There is no case
      in which the drainer fires and then loses the link to protection.
    - **Protection starting.** Protection that starts on a drained building, or on a drainer, while
      the link exists leaves the link, the blackout, the payouts, the sale refusal and the drained
      building's *illegal* firing answer unchanged.
    - **Protection ending.** The end of protection is not an event for the link either: the link
      continues until one of claims 11 to 13 or 17 ends it.
    - **Who may fire.** For a firer of a player house the firing check gives no answer for a
      protected target beyond those it would give an unprotected one (S-0513 claim 28, the human
      rule our engine gives every player house), so the drain proceeds. The one exception is
      S-0513 claim 27's parasite-warhead refusal, which applies to a draining weapon given such a
      warhead as to any weapon. A firer of one of the two
      non-player houses gets the *illegal* answer of S-0513 claim 28 against a protected target
      with its draining weapon, as with any weapon: it never makes the attempt, and what it does
      next follows the ordinary *illegal* consequences (S-0301 claim 9; S-0215 claim 23). In the
      original the same refusal applies to every house that is not human-controlled.

    Whether an event that ends a link (a killing hit, a capture) can reach a protected party is
    that event's owner's rule (S-0302 claim 8, S-0509). When it does, claim 11 or 13 applies.
    - Backing: `measurable` (scenario D15)
    - Confidence: high (no protection test in the attempt, the link, the payouts or any ending),
      medium (the non-player-house case, which rests on S-0513 claim 28's general rule)

## Formulas

**F1, payout ticks.** A payout happens in the drained building's turn of tick number *n* when

> *n* mod |*P*| = 0

with *n* a non-negative integer tick number and *P* a signed 32-bit integer period in ticks.
The remainder is exact integer arithmetic, with no rounding or floating point. The absolute
value is mathematical, including when *P* is the minimum signed integer. Tick number 0 is a
payout tick. A period of exactly 0 is a bad-data case (Questions for the project lead).

**F2, payout amount.** With *A* the payout amount and *C* the payer's available credits at the
payout (S-0404 claim 4), both integers:

> *t* = min(*A*, *C*)

For different houses the payer's credit balance becomes *Y*<sub>p</sub> − *t* and the
recipient's *Y*<sub>r</sub> + *t*. When both owners are the same house, its net balance change
is 0. Amounts and balances use signed 32-bit integer arithmetic, with wrapping as in S-0205
claim 1 and no rounding or floating point. The statements about non-negative balances assume
no overflow. In a skirmish available credits equal the credit balance (S-0404 claim 4), so a
non-negative payer balance never becomes negative through the payment. When *A* < 0 and
*C* ≥ *A*, *t* = *A*: the payer gains |*A*| and the recipient loses |*A*|. If *C* is still lower,
*t* = *C* instead. Thus a negative payer balance can reverse the flow even for *A* ≥ 0.
The recipient's balance can fall below 0 (Edge cases).

**F3, position check ticks.** A vehicle drainer checks its position in its turn of tick number *n*
when

> *n* mod 16 = 2

with *n* a non-negative integer tick number. The remainder is exact integer arithmetic,
with no rounding or floating point.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| position check period | 16 | ticks | claim 12, F3 |
| position check phase | 2 | ticks | tick numbers 2, 18, 34, … |
| payout period when absent | 100 | ticks | engine default of *P* (Import mapping) |
| payout amount when absent | 0 | credits | engine default of *A*; F2 still applies to negative payer credits |

## State and ordering

| State | Event | Result |
|---|---|---|
| not draining | drain attempt, drainer's cell is in the target building's foundation, target not drained | linked (claim 5 effects); target cleared |
| not draining | drain attempt, drainer's cell not in the foundation | not draining; target cleared |
| linked | tick number a multiple of \|*P*\|, building of a resource-destination type takes its turn, neither being erased nor materialising | payout F2 |
| linked | drainer's turn, the building's house allies the drainer's house | not draining (claim 14) |
| linked, vehicle drainer | turn in a tick with *n* mod 16 = 2, its cell is not in the building's foundation | not draining (claim 14) |
| linked | killing hit, removal, or successful temporary off-map storage of either party | not draining (claims 14 and 17) |
| linked | ownership change to a house that does not ally the other party | linked; payer, recipient and blackout follow the new owners |
| linked | protection of either party starts or ends | linked; nothing changes (claim 19) |

Observable ordering consequences:

- With uninterrupted turns, the link can outlast the drainer's departure by up to 16 ticks
  (claim 12). Payouts still require a due payout tick and blackout follows S-0405 claim 5.
- A link ended by the drainer's death leaves no payout after the killing hit, even in the same
  tick (claim 13).
- Power is restored at the house's next power update after the link ends, not at the end itself
  (S-0405 claim 7).

## Edge cases and quirks

- **A second link strands the first building**
  (`quirks.drain_relink_strands_previous`, default on).
  - A drain attempt that forms a link while the drainer is still linked to another building does
    not end the first link. The first building stays being drained, with its blackout, its refused
    firing and its sale refusal, and a first building of a resource-destination type goes on paying
    the drainer's house (claim 7). Neither the alliance test (claim 11) nor the position check
    (claim 12) looks at the first building any more, so it stays drained until the drainer is
    taken off the map or removed from the game: a killing hit on the drainer ends only the newer
    link at once, and the first ends when the dead drainer is removed. Temporary storage also
    ends both links under claim 17. Destroying or removing the stranded first
    building also tears down the drainer's newer link.
  - Backing: `measurable` (D9 part a). Confidence: high for the stranded state, medium for its
    destruction interactions.
  - It needs a drainer that is still linked (the position check has not yet run since it left the
    first building) and data for which S-0308 table P picks the draining weapon for a firer that is
    draining, which row 8 never does (for example a primary weapon with versus value 0 against the
    second building, row 16). Stock data cannot reach it.
  - Fixed: forming a new link first ends the old one with all of claim 14's effects.
  - The old drain animation also stays on the drainer in the original. That is a display defect:
    our engine removes it whatever the switch says.
- **Only vehicles check their position**
  (`quirks.drain_position_check_vehicles_only`, default on).
  - A drainer that is an infantry unit or an aircraft (possible only with data that gives such a
    type a draining weapon) never performs the position check, so it keeps its link after it moves
    away, until claim 11 or 13 ends it.
  - Backing: `measurable` (D9 part b for aircraft), `unmeasured` for infantry for claim 12's
    setup limitation. Confidence: high.
  - Fixed: every drainer performs the check of claim 12.
- **Negative credits or payout amount** can reverse the flow (F2). With *A* < 0 and payer
  credits *C* ≥ *A*, the drained house gains and the drainer's house loses |*A*| per payout.
  There is no zero clamp on the recipient. If *C* < *A*, the transferred amount is *C* instead,
  including when *A* is non-negative. Overflow follows F2.
  Backing: `measurable` (D6 part f). Confidence: high.
  Whether to warn about negative amounts is a question for the project lead.
- **A negative payout period** gives the same payout ticks as its absolute value (F1).
  Backing: `measurable` (D6 part f). Confidence: high.
- **Drainable mobile types** can be targeted (the firing check allows it) but never linked
  (claim 2); every attempt costs the drainer its target.
- **Drain attempts against cells** do nothing and keep the target (claim 1), so a draining weapon
  ordered against a cell repeats its harmless attempt at every chance to fire.

## Import mapping

| Source | Key | Feeds | Form | Absent |
|---|---|---|---|---|
| rules, weapon | `DrainWeapon` | draining weapon (claims 1, 3 and 16; S-0308 owns the term) | boolean | no |
| rules, weapon | `Suicide` | suicide precedence over draining (claim 16) | boolean | no |
| rules, weapon | `DecloakToFire` | fire-only-visible gate before an attempt (claim 16; S-0301 claim 12) | boolean | yes |
| rules, weapon | `RevealOnFire` | ordinary firing reveal, skipped by a drain attempt (claim 16) | boolean | yes |
| rules, type | `Drainable` | drainable type (claims 1 to 3), including bunker, absorber and grinder types in claim 18 | boolean | no |
| rules, type | `ResourceDestination` | resource-destination type (claim 7) | boolean | no |
| rules, `[CombatDamage]` | `DrainMoneyFrameDelay` | payout period *P* (F1) | integer, ticks | 100 |
| rules, `[CombatDamage]` | `DrainMoneyAmount` | payout amount *A* (F2) | integer, credits | 0 |
| rules, `[CombatDamage]` | `DrainAnimationType` | drain animation (claim 5) | animation name; a name not yet known registers a new animation type (S-0004 registration) | none: no animation |

Claim 15's local turn boundaries and claim 17's successful-departure cleanup have no data
key. Claim 18 adds no occupant import: bunker fields remain S-0606's, absorber fields S-0605's,
and recycling fields S-0603's. None overrides the drainable flag above. Claim 19 adds no key:
protection kinds and durations remain S-0513's imports.

## Measurement scenarios

Use a purpose-made small skirmish map with clear land, with no engine add-on and no mods (the named test files of a scenario and the purpose-made test map excepted), speed
index 2 and 800×600. One human house owns the drainers (Floating Discs). One computer house owns
the target buildings and is allied with nobody unless a part says so; it is given buildings only
and 0 starting credits (raised only in D6). Void a run if unrequested building, repair, sale,
resource unloading or any other activity changes the observed credits, power or test objects.
One bystander computer player stays outside the observed area:
apply the glossary's bystander void rule. Named test overrides are removed afterwards and change
data only, never the executable. Run each part three times and record the initial random state.

Use scripted input and a read-only per-tick recorder for: tick number, each house's credit balance
and power totals, each object's position, cell, target, destination, current behaviour, owner,
health and its drain-link partner (whether it is draining, and which building is being drained by
whom), the rearm countdown, and the random-stream positions. Use a standalone recorder and
normal game input; do not use a patched launcher. Request any missing recorder capability rather than
patching the executable or the harness. Record ammunition and burst position as well.
Every expected result below is falsified by a different recorded outcome; where a part merely
asks whether a stock opportunity is reachable (D2 part c), record it as exploration, not
confirmation of that reachability. The observed area is the test buildings and the cells within
weapon range of them.

- **D1, the attempt (claims 1, 6).**
  - Part a: order a Disc to attack an enemy refinery from a distance. Record the tick of the
    attempt. Repeat with named test-map overrides disabling the drain animation and using a
    building with no controlled victims to isolate the attempt's random draws. Expected: no
    projectile or shot damage, no ordinary-shot random draw, unchanged ammunition, burst
    position 0 after target clearing, no started rearm countdown, cleared target, and a formed link. Compare the
    random-stream changes with and without animation separately under S-0109 claim 3.
    Falsified by a draw in the isolated attempt, a spent round, advanced burst, started countdown
    or kept target. Use a named finite-ammunition Disc variant initially holding 5 rounds for
    the ammunition check.
  - Part b: after part a, record the Disc's behaviour and position for 300 ticks with no new
    order. Expected: it falls back from *attack*, keeps no destination and stays over the
    refinery. Then place an enemy tank within its laser range: it may fire its laser at the tank but
    takes no step towards a tank out of range.
  - Part c: a named test-map Disc variant with a draining primary weapon and a projectile that
    can target cells; order it to force-fire at an empty cell beneath it. Expected: nothing
    happens and the target stays. Repeat over several turns to check repeated harmless attempts.
  - Part d: use that draining-primary variant against a non-drainable building. Expected: the
    firing check refuses it without spending ammunition or starting rearm, and no link forms.
    The attempt's additional non-drainable guard is `unmeasured`: no normal order bypasses
    this earlier firing refusal, so this part tests claim 3 c rather than that redundant guard.
- **D2, geometry (claim 2).**
  - Part a: an enemy drainable building with a multi-cell foundation, its foundation cells recorded during the run. Attack
    it from each side. Record which cell the Disc stops in. Expected: a link whenever the stop cell
    is any foundation cell. Repeat with named test-map flight-height variants at two different
    heights, both within weapon range. Expected: the same foundation-cell requirement,
    independent of height. A different cell rule at either height falsifies claim 2.
  - Part b: named test data giving the draining weapon a range of several cells and the Disc a
    non-vertical projectile for it, so S-0215 claim 37 h does not take it over the building. Order
    the attack from three cells away. Expected: the attempt happens outside the foundation, the
    target is cleared and no link forms. Falsified by a link.
  - Part c: a stationary Disc in a cell next to an enemy refinery, stock data, attack order.
    Record whether it fires before moving over the building. Expected: either it moves first and
    links, or it fires at once, loses its target and does not link; record which (claim 2's medium
    part).
  - Part d: named test data making an enemy tank type drainable; order a drain attack on it.
    Expected: the Disc ends over the tank, the attempt clears its target and no link forms.
- **D3, refusals and one to one (claims 3, 4).**
  - Part a: Disc A drains an enemy refinery; order Disc B to attack the same refinery. Expected:
    B chooses its draining weapon (row 8) and gets *illegal*; no second link.
  - Part b: order Disc A, while draining, to attack the refinery it drains. Expected: refused
    (*illegal*); the link stays.
  - Part c: order Disc A, while draining, to attack a second enemy building next to the first.
    Expected: it uses its laser against the second building and no link to it forms.
  - Part d: order a Disc to attack an enemy building of a non-drainable type with force-fire, and
    record the weapon. Expected: never a drain attempt against it.
- **D4, forming the link (claim 5).**
  - Part a: drain an enemy power plant whose house has drain greater than 0. Expected: the house's
    output reads 0 at its next power update and its powered buildings stop (S-0405).
  - Part b: give the enemy a psychic tower holding three of the human's infantry, then drain the
    tower. Expected: all three are released, most recently taken first, at the attempt tick. A
    failed attempt from beside the tower (D2 part b data) releases none.
  - Part c: record the drain animation's presence on the Disc and its removal when the link ends.
    In a named test-map variant select a short finite animation, with no follow-up or animation
    effects: playback ends while the link and payouts continue. In another variant omit the
    animation setting in fresh base test data: no animation appears and the link still forms.
- **D5, position check (claims 6, 12, 14).** With a Disc draining an enemy refinery and an enemy
  power plant in two separate runs, order it to move one cell off the building at varied ticks
  (start the move at ticks with *n* mod 16 = 0 to 15). Record the tick at which its cell first lies
  outside the foundation and the tick at which the link ends. Expected: the link ends at the first
  position check whose sampled position is outside the foundation, with *n* mod 16 = 2.
  An end-of-tick position alone cannot decide whether movement preceded that tick's check.
  A departure later in that turn may remain linked until the next due check. Payouts on due
  ticks and the blackout continue until release, subject to their ordinary gates. Also
  move it to another cell of the foundation of a multi-cell building (foundation cells recorded during the run): the link stays. Falsified by a release
  on a different phase, or survival after a due check sampled outside the foundation.
  The pulse case needs an established skirmish delivery setup before it can be measured.
- **D6, payouts (claims 7 to 10).**
  - Part a: Disc drains an enemy refinery; the enemy has a large balance. Record both balances on
    every tick. Expected: on exactly the ticks whose number is a multiple of *P* (stock data value),
    the enemy loses *A* and the human gains *A*. No score change for either house.
  - Part b: drain an enemy drainable building that is not a resource destination (a power plant or
    a defence). Expected: no balance changes.
  - Part c: form the link at varied ticks. Expected: the first payout at the first multiple of *P*
    at or after the link tick whose payout opportunity follows formation. In particular,
    formation after the payout opportunity on a multiple of |*P*| waits a full |*P*| ticks.
    Include forming and ending links on due ticks with the building before and after the
    drainer in update order; any payout after a prior link end falsifies claim 8.
  - Part d: set the enemy's balance (named test map starting credits) to below *A* and to 0.
    Expected: the first payout takes the whole remainder, later ones take 0; the link stays.
  - Part e: while draining, lock the refinery with a temporal weapon of a third house allied with
    the human. Expected: no payouts while it is being erased, payouts resume after an early release.
    In named test-map variants give a drainable resource-destination building the power-switch
    capability and power output, then switch it off, damage it and make its house low power:
    payouts continue whenever it is not being erased, and positive output while drained gives
    the S-0405 claim 5 blackout too.
  - Part f: named test-map periods 7 and −7 ticks, and payout amounts 11, 0 and −11 credits.
    Record at least three due ticks with abundant, zero and negative payer credits. Establish
    negative credits without memory writes: first let the future payer's Disc drain another
    house with the negative amount until the Disc owner's balance falls below −11, then have
    a third house drain that owner's refinery. Expected: every transfer is min(*A*, *C*),
    including *C* < *A* < 0; a negative period has the positive period's due ticks. Record
    resulting balances rather than only sidebar animation. Any clamp or differing amount
    falsifies F2. The combination *C* < 0 ≤ *A* is `unmeasured`: the negative-amount setup
    does not reach it while the game-wide amount remains non-negative.
- **D7, ownership (claim 11).**
  - Part a: capture the drained refinery with an engineer of the human house. Expected: the link
    ends at the Disc's next turn; no payout after it.
  - Part b: mind-control the draining Disc with a third, enemy house's controller. Expected: the
    link stays; payouts now go to the third house.
  - Part c: mind-control the draining Disc with the enemy house that owns the refinery. Expected:
    the link ends.
- **D8, loss (claims 13, 14).** Kill the draining Disc; in another run destroy the drained power
  plant; in another start selling an enemy power plant (enemy scripted input) and drain it before
  its sale ends. Expected: the link
  ends at the killing hit or removal; the drained house's power returns at its next power update,
  and no payout falls after the killing hit.

- **D9, data-only quirks (claims 4, 12; Edge cases).**
  - Part a: named test-map overrides give a Disc a draining primary weapon. Use two adjacent
    drainable buildings, with one a resource destination. Form the first link, then move over
    the second and attack it between position-check ticks. Void and repeat if a position check
    released the first link before the second formed. Expected: both buildings stay drained,
    but the drainer's current partner is only the second. The first building keeps paying
    without a return visit. In separate runs kill the drainer (only the newer link ends at
    the hit; the stranded link ends on removal), and destroy the first building (the newer
    link also ends). Automatic first-link cleanup on relinking falsifies the quirk.
    Observe sale refusal and firing refusal of the stranded building too. Order the drainer to
    fire its non-draining weapon at that first building: claim 3 a does not refuse it. Select
    a draining weapon against it in a separate fixture: claim 3 b does refuse it.
  - Part b: named test-map aircraft variant with a draining weapon and hovering attack capability,
    and a drainable resource-destination building. Form a link while over the foundation,
    then move away for at least 64 ticks. Expected: the aircraft remains linked and due payouts
    continue until death or the ownership condition of claim 11; release on a vehicle's
    position-check phase falsifies the non-vehicle rule. If no legal aircraft attack can form
    a link with these data, report this scenario as blocked, not as confirmation.

- **D10, local turn boundaries (claim 15).** Extend D5 with a move that crosses the foundation
  edge during the drainer's own turn on a due check tick. Record position before and after its
  turn. Expected: release waits for the next due sample. Repeat with displacement by an earlier
  object: a due sample already outside releases it that tick. Record the drained owner's house
  power update: it already sees the ended link, subject to other blackout causes.
  In D6 erase the refinery on a payout tick, varying the temporal attacker's place before or
  after it: only an opportunity before lock-on can pay. Named test data adds periodic income
  due on a payout tick with initially empty credits: expected income is available to the payout.
  Start a sale before draining: a payout opportunity before later sale removal still pays.
  Void any run where behaviour start already removed the building. Any opposite ordering
  falsifies the relevant boundary. For vehicle erasure, lock a linked test drainer temporally
  and displace it by an independent lift where legal: the due position sample still runs.
  If no legal displacement exists, that subtest is blocked, not confirmation. Warp-only engine
  exclusion is tested under S-0513 claim 51, not as an original all-stage assertion.
- **D11, competing firing answers (claim 3).** Named draining-primary test variants and a
  read-only check-answer recorder. Against a drained building hidden without sensor coverage,
  expected *impossible*. Make it the firer's current drain partner instead: expected *illegal*.
  Keep the vehicle drainer over the foundation so position checks cannot release its link.
  Further variants combine draining with power-sensitive, spawner or magnetic-beam properties,
  supplying respectively missing power, no living spawns, or another gripped target plus a live
  wave. Expected: power-sensitive *impossible* beats the draining-weapon refusal, but that
  refusal beats empty-spawner and conditional magnetic-wave *rearming*. Record answers,
  admission and target retention. Different answers falsify the corresponding precedence.
  If ordinary input and data cannot reach an exact combined state, mark that subtest blocked.
  Two identical *illegal* answers cannot establish an internal order.
- **D12, non-shot effects (claim 16).** Named variant gives a drainer a cloak ability and a
  draining weapon permitted to fire cloaked, with ordinary firing reveal enabled. Remove other
  reveal causes. Record cloak, visibility to the target's owner, recent-shot tick and link.
  Expected: attachment without a new shot tick, firing reveal or direct uncloak. Repeat D2
  part b: failed attachment likewise has no shot effects. Control: require a visible firer,
  expecting the earlier *cloaked* answer to initiate uncloaking before an admitted attempt.
  Add the suicide property in a separate run: expected suicide hit and no attachment, even
  over the foundation. For aircraft use a draining-primary variant that can target the ground,
  force-firing at a cell rather than an object. Record admitted initial pass, pending-round mark
  and ammunition through run completion. Expected: no immediate per-shot spending, but ordinary
  pending run-round spending at S-0501 claim 15's boundary. Any new shot record, direct reveal
  or uncloak, attachment from the suicide weapon, or rollback of an admitted pass's pending
  round falsifies the corresponding rule. The cell test does not test the object-target
  continuation departure of S-0501 claim 12; that claim's engine-only scenario checks it.
  Use a healthy, non-critical-health cloaker to avoid random uncloak.
- **D13, temporary departure (claim 17).** Named test data gives a non-balloon airborne vehicle
  drainer eligibility to board an ordinary transport. Board between due position checks and
  void a run if the link ended before boarding. Record actual successful boarding, both partners,
  animation, power and balances. Expected: immediate detach, no later payouts, power recheck
  under S-0405 claim 7, no automatic attachment after unloading. Repeat with an open-topped
  carrier: turns inside do not preserve the link. Repeat the two-link D9 fixture if reachable:
  both links end on departure. A surviving or automatically restored link falsifies claim 17.
  If legal boarding while linked cannot be reached, report the fixture blocked, not a kept link.
- **D14, drained occupants (claim 18).** Named map overrides make bunker, absorber and grinder
  types drainable. In separate runs drain each while occupied and while an entrant approaches.
  Pair with undrained controls. Keep bunker and grinder without output or powered-unit
  dependencies to isolate direct drain effects. Record contacts, entry, bunker stages and link,
  occupant shots, damage and intervals, absorber count and release, grinder consumption and
  recipient credits. Expected: unchanged eligibility, bonuses, occupancy and recycling result.
  Use an established bunker release input route under S-0606 claim 12; otherwise mark that
  subtest blocked, not refused by draining. Absorber blackout does not itself prevent entry
  or voluntary release. Recycling credit goes to the building owner; a drain credit transfer
  needs a separately due resource-destination payout. A drain-specific refusal, lost bunker
  bonus, ejection or changed recycling amount falsifies claim 18. Repeat output-enabled
  variants separately to distinguish S-0405's ordinary blackout consequences.

- **D15, protection (claim 19).** Two player houses: the human house owns a Disc, the computer
  house a power plant and a refinery, with enough credits for payouts, under scripted input for
  its superweapon launches. Keep infantry out of the protected 3 × 3 area. Record the link
  partners, both balances, the plant house's power totals and blackout, and each object's
  protection state; if the hovering Disc is itself protected, record that rather than voiding.
  - Part a: protect the plant with the invulnerability effect, then order the drain attack.
    Expected: the link forms at the attempt and the house's output reads 0 at its next power
    update. Repeat against the refinery: due payouts happen.
  - Part b: as part a with a force shield over the refinery. The launch's own timed blackout of
    the launching house (S-0513 claim 30) hides the power effect, so observe the link partner and
    the payouts. Expected: link and due payouts as unprotected. If named test data lengthens the
    protection past that blackout, the drained plant's blackout is also seen afterwards.
  - Part c: link first, then protect the drained building, in separate runs with each kind.
    Expected: no detach, payouts continue on due ticks, the drained building still cannot fire or
    be sold.
  - Part d: let the protection of part c expire while the link holds. Expected: no change at
    expiry.
  - Part e: a computer-house Disc under scripted input ordered against a protected human
    building, with a check-answer recorder. Expected in the original: *illegal* and no attempt
    (S-0513 claim 28's original rule). If such an order cannot be given, mark it blocked.

  A detach, a refused or failed attempt in parts a to d, or a change at the start or end of
  protection falsifies claim 19.

## Open questions

1. **Payout on the tick of forming or ending.** Claim 8 leaves it to turn order (S-0203 claim 4).
   D6 part c and D7 record it.
2. **A stock attempt from a neighbouring cell.** Whether stock movement and firing timing ever let
   a Disc attempt a drain before it is over the building (claim 2): D2 part c.

## Questions for the project lead

- **A payout period of exactly 0.** The original fails with a division fault when a building of
  a resource-destination type is drained with *P* = 0. Proposed departure: the importer warns
  and payouts never happen. This safe replacement is pending the lead's choice, not an
  original-behaviour claim. Backing: `unmeasured` (no destructive crash test proposed).
  Confidence: high for the failure.
- **A negative payout amount.** F2 retains the original signed behaviour; should the importer
  also warn about this data?
