---
id: S-0301
title: Firing cycle (delay between shots, bursts, ammunition and reloading, the firing check)
topic: combat
revision: 1
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: []
supersedes: []
---

# S-0301: Firing cycle (delay between shots, bursts, ammunition and reloading, the firing check)

## Summary

When an object that has fired may fire again: the delay between shots and how rate-of-fire
bonuses combine into it, bursts and the short delays inside a burst, ammunition and reloading,
delayed and animation-synchronised shots, and the firing check, the per-attempt test whose answer
says whether a shot can be fired now and, if not, why. Aiming tolerances and turret behaviour are
S-0310; where a shot starts is S-0313; which weapon is used is S-0308; range is S-0307.

## Terms

Glossary terms are used as defined there (*weapon*, *tick*, *tick number*, *turn* in both senses,
*target*, *destination*, *house*, *country*, *vehicle*, *infantry*, *building*, *aircraft*,
*scenario random stream*, *ranged draw*, *round-toward-zero mode*, *importer warning*, *quirk
switch*, *departure*, *AI house*, *speed-normalised delay*). From S-0202: R(*a*, *b*), the ranged
draw. From S-0205: RZ₅₃ and trunc (Formulas there). From S-0203: *turn pass*. In addition:

- **Firer.** An object that fires a weapon: a vehicle, infantry, aircraft or building.
- **Shot.** One firing of one weapon by a firer: one projectile, beam or effect launched. A shot is
  *made* when the firer's weapon is launched; an attempt that the firing check refuses, or whose
  launch fails (claim 15), makes no shot.
- **Void shot.** An attempt that passed the firing check but whose projectile could not be put on
  the map (claim 15). It makes no shot.
- **Rearm countdown.** A per-firer countdown in ticks. It is set by every shot the firer makes, to
  that shot's *delay* (claim 2 or 5), and it is *running* until that many ticks have passed. While it
  runs, the firer may not fire any of its weapons. One countdown serves all weapons of a firer.
  S-0215's *rearm time remaining* is the number of ticks still to run.
- **Rate-of-fire field** *R*. The imported weapon field that sets the base delay between shots, in
  ticks (Import mapping).
- **Burst count** *B*. The imported weapon field that sets how many shots one burst has.
- **Burst counter** *k*. A per-firer integer, 0 when the firer is created. It counts the shots
  made in the current burst, and one counter serves all weapons of a firer.
- **House rate-of-fire factor** *h*. A per-house double, set when the houses are set up (claim 3).
- **Difficulty row.** As S-0302 (Terms): the entry of the glossary's *difficulty skill block* for
  one difficulty, one of the three rules sections *easy*, *normal* and *difficult* (Import
  mapping). Each row has a rate-of-fire factor *ℓ*. The *normal row* is the one the original gives
  a human-controlled house in a skirmish. Which row a house uses is set-up behaviour (claim 3).
- **Veteran, elite.** The two raised veterancy states of a unit, as S-0311 (veterancy)
  defines them; S-0311 claim 3 owns the test of whether a unit has an ability. A type's *veteran rate-of-fire ability* and *elite rate-of-fire ability* are two
  imported flags that say whether the raised state speeds up its firing.
- **Veteran rate-of-fire factor** *v*. A game-wide double from the rules data, holding a decimal
  read as S-0003 claims 6 and 7 state (a single-precision value unless written with a percent
  sign). The difficulty-row and country factors are held the same way.
- **Garrisoned building.** A building whose type can be occupied by infantry and may fire while
  occupied (two imported flags), and which has at least one occupant. Its occupant count is *n*.
- **Occupy factor** *m*, **bunker factor** *b*. Two game-wide single-precision decimals from the
  rules data (Import mapping).
- **Bunkered.** A vehicle that is inside a tank bunker (S-0206, Terms).
- **Berserk.** The state of a unit that a berserk-causing warhead has put out of its owner's
  control (the warhead specs own how it starts and ends).
- **Effect weapon.** A weapon whose shot keeps a lasting effect attached to the firer: a sonic
  wave, a spark-particle or fire-particle emitter, or a railgun particle trail (each an imported
  weapon flag). The effect is *alive* from the shot until the effect ends (the combat and rendering
  specs own its duration).
- **Disc laser weapon.** A weapon with the imported disc-laser flag (S-0203, Terms).
- **Magnetic-beam weapon.** A weapon with the imported magnetic-beam flag. Its shot makes a
  *magnetic wave* attached to the firer, and the target it has taken hold of is the firer's *held
  target* for as long as the wave lasts; what the wave does to that target belongs to the combat
  specs.
- **Temporal weapon.** A weapon with the imported temporal flag: an erasing weapon (glossary,
  *being erased*). Its *temporal beam* is attached to its target from the moment it locks on until
  it lets go. S-0308 (claim 6) and S-0310 (claim 7) also use the term.
- **Holding a target.** A firer *holds* a target with a temporal weapon while its temporal beam is
  attached to that target, and with a magnetic-beam weapon while that target is its held target.
- **Spawner weapon, spawned aircraft.** A *spawner weapon* is a weapon with the imported spawner
  flag. Its attempts do not launch a projectile; they send the firer's *spawned aircraft* (the
  aircraft that the firer's type keeps and launches, whose life cycle the aircraft specs own)
  against the target. A spawned aircraft is *alive* unless it is dead. A firer is *launching* spawned
  aircraft while at least one of them is in the aircraft specs' launch phase.
- **Seeker type.** A vehicle type with the imported hunter-seeker flag (Import mapping). Only its
  effect on the firing check (claim 9, rank 9) is specified here.
- **Materialising.** The state of an object that has just teleported and is phasing in at its
  destination, until the teleport specs end it.
- **Gattling type.** As S-0308 (Terms). **Spray-attack type.** As S-0313 (Terms).
- **Ammunition** *a*. A per-firer integer. A type's *maximum ammunition* *A* is an imported field, and
  *A* = −1 means the type does not use ammunition, and then *a* stays −1 (unlimited).
- **Reload countdown.** A per-firer countdown in ticks, used only by vehicles (claim 11).
- **Firing check.** The test that every attempt to fire a weapon at a target makes. Its *answer*
  is one of the categories of claim 9: *can fire*, or a reason why not. An attempt either asks for
  the range test ("with range") or does not; a check that does not ask never answers *out of
  range* from the ordinary range test (S-0307). The attacking behaviours say which they ask (S-0310
  claim 9 for vehicles, claim 14 here for a delayed building shot).
- **Firing-frame counter.** A per-vehicle counter that is either *idle* or *running* with a whole
  number value, used by vehicles whose sprite has firing frames (claim 13).
- **Delayed-fire countdown.** A per-building countdown used by buildings whose shots wait for an
  animation (claim 14).
- **Charge turret.** A vehicle type with the imported charge-turret flag; it has several turret
  images (its turret count *N*), and which one is current follows the rearm countdown (claim 16).

## Behaviour

### The delay between shots

1. **Rearm countdown.** When a firer makes a shot in the tick with tick number *t*, its rearm
   countdown is set to that shot's delay *D* (claims 2 to 8). The firing check then answers
   *rearming* for every weapon of the firer, in every tick with tick number below *t* + *D*, and
   no longer from tick *t* + *D* on. So *D* ticks pass from one shot to the next at the least; with
   *D* ≤ 0 the countdown is over at once. How often a firer attempts to fire (for a vehicle, once in
   each of its turns) belongs to S-0215 and to claim 9 here. A firer that has never fired, including
   one created during play, has no running countdown: it can fire as soon as it has a target. The
   countdown is not changed by switching weapons or targets. Counting is in ticks: game speed does
   not change *D*, and `quirks.speed_normalised_delays` does not apply to any delay of this spec.
   - Backing: `measurable` (scenario F1)
   - Confidence: high

2. **The full delay.** After the last shot of a burst (claim 4), or after any shot of a weapon
   whose burst count is 1, the delay is *D* of Formulas F1. It starts from the weapon's rate-of-fire
   field *R* times the firing house's rate-of-fire factor *h* (claim 3) plus one ranged draw
   R(0, 2) from the scenario random stream, truncated; then, in this order, the veteran factor
   (claim 6), the garrison division (claim 7), the bunker division (claim 7) and the berserk halving
   (claim 8) apply where their condition holds. Each step truncates toward zero; F1 gives the number
   domain of each.
   - Backing: `measurable` (scenarios F1 and F2)
   - Confidence: high

3. **The house rate-of-fire factor.** In a skirmish, *h* is the rate-of-fire factor of the house's
   difficulty row times the rate-of-fire factor of the house's country, a product of two doubles
   (F2). In a campaign mission *h* is the row's factor alone; the country's factor is not used
   (campaigns are a pointer only).
   - *Our engine: every house, human or AI, uses the normal row* (a rule of our engine by decision
     of the project lead; no switch; the glossary's *difficulty skill block*). AI difficulty comes
     from the AI's strategy modules and never from this factor. A house's row can differ only
     through an optional per-player handicap setting, which is off by default and is never tied to
     an AI house's difficulty automatically.
   - *The original's set-up (a departure, no switch).* The original gives every human-controlled
     house in a skirmish the normal row, and each computer-controlled house the row of its lobby
     difficulty (in the order easy, normal, difficult), moved one row towards easy when more than
     one human player is in the game, the rules' computer-easy-bonus flag is on and the row is not
     already the easy one. In a campaign mission the original takes the row of the player's chosen
     difficulty for the player's house and the row of the scenario's computer difficulty for the
     others. Our engine does not reproduce these assignments; the handicap setting can.
   - Backing: `measurable` (scenario F2)
   - Confidence: high (the product, and that the campaign uses the row's factor alone); medium (the
     original's row assignments, recorded as set-up behaviour only)

4. **Bursts.** A weapon with burst count *B* fires bursts of *B* shots. A shot made while the
   firer's burst counter is *k* is the *j*-th shot of its burst, with *j* = *k* + 1. If *j* < *B*,
   the delay after it is the *in-burst delay* below; otherwise it is the full delay of claim 2.
   After the shot the counter is *j* mod *B*, so after the last shot of a burst it is 0 again. The
   in-burst delay after the *j*-th shot is:
   - for a vehicle with 1 ≤ *j* ≤ 4 whose type gives a burst delay *j* that is not −1: that value,
     in ticks, with no draw,
   - otherwise one ranged draw R(3, 5) from the scenario random stream: 3, 4 or 5 ticks.

   No rate-of-fire bonus applies to an in-burst delay, but the berserk halving does (claim 8). The
   burst counter is shared by the firer's weapons, so a firer that changes weapons in mid-burst
   carries the count over and the new weapon's *B* decides. When the firer's target is cleared (it
   has none), its burst counter is set to 0; a change from one target to another keeps it.
   - Backing: `measurable` (scenario F3)
   - Confidence: high (the counter and both in-burst delays); medium (the reset when the target is
     cleared, and the carry-over between weapons)

5. **Effect weapons fire at their bare rate.** The delay after a shot of a sonic, spark-particle,
   fire-particle or railgun weapon is the rate-of-fire field *R* itself, with no draw, no house
   factor and none of the bonuses of claims 6 and 7 (the berserk halving of claim 8 still applies).

   A firer has at most one live effect of each kind (one sonic wave, one spark emitter, one fire
   emitter, one railgun trail), whichever of its weapons started it. Each weapon of a firer has an
   *other weapon*: for the weapon in slot 0 it is the weapon in slot 1, and for a weapon in any
   other slot it is the weapon in slot 0 (S-0308 numbers the slots). The firing check (claim 9)
   answers as follows for the asked weapon:

   | The firer has a live effect of | Answer for the asked weapon |
   |---|---|
   | the asked weapon's own kind | *rearming* (rank 6) |
   | the kind of the asked weapon's other weapon, whatever the asked weapon's own kind | *impossible* (rank 4) |

   So an effect weapon fires again after max(*R*′, the effect's lifetime) ticks, where *R*′ is *R*
   (trunc(*R* / 2) for a berserk firer, claim 8), and no draw is taken for its delay.

   A magnetic wave shares the sonic wave's single slot, so a live magnetic wave also counts as a
   live sonic effect: it blocks a sonic weapon of the same firer (*rearming* for the sonic weapon's
   own kind, *impossible* as the other weapon's effect), and the reverse (`quirks.shared_wave_slot`,
   default on; the fixed behaviour keeps two slots).
   - Backing: `measurable` (scenario F4)
   - Confidence: high (the bare rate, the missing draw, *rearming* for the weapon's own kind),
     medium (*impossible* for the other weapon, and that the effect's lifetime can lengthen the
     interval, which depends on the effect specs); low for the shared slot of magnetic and sonic
     waves (a firer needs both weapons, so test data must supply the case)

6. **Veteran factor.** If the firer is veteran and its type has the veteran rate-of-fire ability,
   or it is elite and its type has the veteran or the elite rate-of-fire ability, the delay is
   multiplied once by *v* and truncated (F1). Elite does not apply the factor twice.
   - Backing: `measurable` (scenario F2)
   - Confidence: high

7. **Garrisons and bunkers.** A garrisoned building divides the delay by its occupant count *n*
   (integer division, truncated toward zero), then, if the occupy factor *m* is above 0, divides
   by *m* and truncates. A bunkered vehicle divides the delay by the bunker factor *b* and truncates,
   if *b* is a number other than 0 (a *b* that is not a number skips the division, Edge cases). A building is never treated as bunkered.
   - Backing: `measurable` (scenario F5)
   - Confidence: high (formula); medium (the occupant-count division as observed in play, since
     which occupant's weapon fires is S-0308's)

8. **Berserk halving.** A firer that is berserk when it makes a shot gets half the delay chosen by
   claims 2 to 7, truncated toward zero, whatever kind of delay it is (full, in-burst, effect or
   building). Exception: the shot of a disc laser weapon is never halved.
   - Backing: `measurable` (scenario F6)
   - Confidence: high

9. **The firing check and its answers.** Every attempt to fire asks the firing check, with the
   target and the weapon. Its answer is one of these categories:

   | Answer | Meaning |
   |---|---|
   | can fire | the shot can be made now |
   | rearming | the rearm countdown is running, the weapon's own effect is still alive (claim 5), a building's delayed shot is pending (claim 14), a firing-frame gate is closed (claim 13), and a few early cases named in the rank table |
   | out of ammunition | the firer's ammunition *a* is 0 |
   | not facing | the weapon is not pointed at the target (S-0310) |
   | still turning | the turret is still turning (S-0310) |
   | moving | the firer may not fire while it moves (S-0310) |
   | cloaked | the weapon needs the firer uncloaked and it is not |
   | out of range | the target is beyond the weapon's range (S-0307) |
   | illegal | the target is not a legal target for this firer and weapon. The combat specs give the cases: a versus value of exactly zero (S-0304, claim 1), an airborne or non-ground target the projectile cannot hit, or a non-unit target of a projectile that is not anti-ground (S-0306, claim 23), the healing, capture and parasite tests (the feature specs). S-0214, claim 6(e), defers to these for what the check refuses |
   | impossible | the firer cannot fire this weapon now for a reason of its own: no weapon in the slot, the effect of its other weapon still alive (claim 5), the weapon needs power or the firer is disabled, and similar |
   | busy | the firer is launching spawned aircraft (Terms) |
   | must deploy | a vehicle whose type must deploy before it fires, when its own cell fails the deploy test (a slope, a building in the cell, or a land type that does not allow it) and none of ranks 1 to 10 below gives an answer, that is, the part of the check that does not look at aiming would let the shot through. It then outranks *not facing* (S-0310, claim 9), and in the rank table also *moving* and *still turning*, an order that no game state shows (see below). On a cell that passes the test none of this applies: the vehicle gets the aiming answers as usual and S-0310 deploys it on *can fire* or *not facing* |

   Where more than one reason holds, the firer gets the answer of the highest rank below (rank 1
   is the highest):

   | Rank | Answer | Reasons at this rank |
   |---|---|---|
   | 1 | impossible | a vehicle-specific reason (vehicles only) |
   | 2 | busy | a vehicle that is launching spawned aircraft |
   | 3 | illegal | a rare vehicle-specific reason (a vehicle tied to another unit that blocks its attack); no spec here owns it |
   | 4 | illegal, impossible, rearming | most *illegal* and *impossible* reasons (those about the firer's state, the other weapon's alive effect and most target tests), and the early *rearming* cases: the firer is materialising, it already holds this very target with a temporal or magnetic-beam weapon, or the weapon is a spawner weapon none of whose spawned aircraft is alive. Their precedence among each other is not stated |
   | 5 | rearming | the rearm countdown runs, or the firing-frame gate of claim 13 is closed |
   | 6 | rearming | the weapon's own effect is alive (claim 5) |
   | 7 | out of ammunition | |
   | 8 | cloaked | |
   | 9 | out of range, illegal | the late target tests: a seeker type (Terms) answers *out of range*, whatever the distance; a warhead that cannot affect the target's armour class, and the capture and parasite tests, give *illegal* |
   | 10 | out of range | the attempt asked for the range test and the target is beyond the weapon's range |
   | 11 | must deploy | a vehicle only (S-0310, claim 9) |
   | 12 | illegal, impossible | the remaining vehicle tests: a healing weapon's *illegal* (S-0314), and one rare *impossible* that no spec here owns |
   | 13 | moving | |
   | 14 | still turning | |
   | 15 | not facing | |
   | 16 | the locomotor's own refusal | only when every test above passes (S-0310, claim 7) |

   So a firer whose countdown runs gets *rearming* even when its target is out of range or it is
   out of ammunition. The rank of *rearming* for a building's pending delayed shot (claim 14), and
   of *busy* for a firer that is not a vehicle, is not stated. Among ranks 11 to 15 only the
   difference between *must deploy* and *not facing* shows in game state: a vehicle that gets *not
   facing* re-aims its turret (S-0310, claim 9), and one that gets *must deploy* does not.
   *Moving* and *still turning* leave a vehicle as idle as *must deploy* does, so no game state
   shows the order of those three. The rank table gives the original's order. An implementation
   that does not expose the answer cannot tell the three apart, so where the rank table's answer
   is one of the three, it may give instead another of the three whose reason also holds.
   - Backing: `measurable` (scenarios F7 and F8)
   - Confidence: high (rearming before ammunition and range, and facing and moving after them, and
     *must deploy* before *not facing*); medium (the full membership of *illegal* and *impossible*,
     and the early *rearming* cases, which other specs own; only the ones relevant to this spec are
     listed); low (ranks 3 and 12, whose reasons are named only roughly)

### Ammunition and reloading

10. **Ammunition.** A firer starts with its type's initial ammunition (Import mapping). Each shot
    that a vehicle, infantry or building makes uses one round when *a* > 0; *a* = −1 is never
    changed. With *a* = 0 the firing check answers *out of ammunition*. A disc laser shot uses no
    round (and it neither advances a garrison's muzzle index, S-0313, nor restarts a vehicle's
    reload, claim 11). Aircraft ammunition belongs to the aircraft specs. The 1-tick rule depends on
    a building's ammunition before the shot: a building that holds more than 1 round before a shot
    has a delay of exactly 1 tick after that shot, with no draw
    (`quirks.building_ammo_fast_rearm`). The shot made with its last round gets the normal delay of
    claim 2, and a building with unlimited ammunition (*a* = −1) is not affected by the rule. A
    building with 3 rounds therefore fires in ticks *t*, *t* + 1 and *t* + 2, then answers
    *rearming* until its countdown ends, and only then *out of ammunition*.
    - Backing: `measurable` (scenario F9)
    - Confidence: high (rounds used, out of ammunition); medium (the building rule: it needs a building with
      ammunition above 1, so test data must supply it)

11. **Reloading (vehicles only).** A vehicle whose type has *A* ≥ 0 regains ammunition over time:
    - Whenever the vehicle makes a shot that launches a projectile and then has *a* < *A*, its
      reload countdown restarts at the reload delay of F3, unless its type has the manual-reload
      flag or *A* = 0. So continuous firing keeps postponing the reload.
    - A vehicle that has not yet had its reload countdown restarted (it has made no shot that
      restarts it and nothing else has) regains a round in its first turn if *a* < *A*, as if its
      countdown were over.
    - When the reload countdown is over and *a* < *A*, *a* grows by 1 and the countdown restarts at
      the reload delay of F3 for the new *a*, so one round comes back per reload delay.
    - The reload delay is the type's empty-reload field when *a* = 0 and that field is not −1,
      otherwise the reload field plus the reload-increment field times *q*², where *q* = 1 when the
      type's pip-wrap field is 0, else *q* is *a* divided by the pip-wrap field in integer
      division toward zero (F3).
    - While a vehicle whose type lacks the mobile-fire flag is being moved by its locomotor, its
      reload countdown does not advance.

    Infantry and buildings never regain ammunition over time. A vehicle regains at most one round
    in each of its turns, so a reload delay of 0 or less still gives one round per tick.
    - Backing: `measurable` (scenario F10)
    - Confidence: high (formula, restart on each shot, one round per completion); medium (the
      frozen countdown while moving, and the restart by damage of claim 17)

12. **Cloaked firers.** A weapon with the fire-only-visible flag (set unless the data clears it)
    gets the answer *cloaked* while its firer is in any cloaking state other than fully visible
    (cloaking, cloaked or uncloaking); an aircraft gets it only while fully cloaked. The firer's own behaviour then uncloaks it (the cloaking specs; S-0310, claim 9).
    - Backing: `measurable` (scenario F7, part c)
    - Confidence: medium

### Shots that wait

13. **Firing frames (sprite vehicles).** A vehicle whose type has *F* > 0 firing frames starts its
    firing-frame counter running at 2*F* − 1 when it makes a shot outside the gate below. In each
    of the vehicle's turns a running counter falls by 1, and in the turn after it has reached 0 it
    is idle again; it does not change while the vehicle travels through a tunnel. If the type gives
    a firing sync frame *s* for burst position *k* mod *B* = 0 or 1 (the data value −1 gives none,
    Import mapping), then while the counter runs, an attempt with the primary weapon gets
    *rearming* until the counter equals *s*; in the tick it equals *s* the rearm countdown is
    ignored and the shot can be made, and that shot does not start the counter again. With no sync
    frame given, firing frames change only the picture.
    - Backing: `measurable` (scenario F11, test data only)
    - Confidence: medium

14. **Animation-delayed building shots.** A building whose type has the animation-delayed-fire
    flag does not fire when its attack behaviour gets *can fire* (S-0215, claim 32). Instead it
    starts its firing animation and its delayed-fire countdown, set by the type's delayed-fire field
    *d* (ticks). The delayed shot is pending until the building's turn max(*d* − 1, 0) ticks after
    the turn that started it. (A building takes a turn in every tick. The countdown does not depend on how often its attack behaviour is processed,
    which is every other tick while it turns or rearms, S-0215 claim 32.) In that turn the building asks the
    firing check again, with range. On *can fire* it fires its weapon at its current target; on any
    other answer the shot is dropped. Either way the shot is no longer pending. While the shot is
    pending, the firing check answers *rearming*. The rearm countdown starts only with the shot
    itself.
    - Backing: `measurable` (scenario F12)
    - Confidence: medium (the countdown arithmetic); low (the support charging of a prism-type building,
      which uses the same countdown and is left to a future structures spec)

15. **Failed launches.** If the projectile of a shot cannot be put on the map, the shot is void:
    the rearm countdown, the burst counter, the ammunition and the garrison muzzle position (S-0313)
    stay as they were.
    - Backing: `measurable` (S-0306, scenario P5 run d: an arcing shot for which the launch pitch has
      no solution is removed before it enters play, S-0306 claim 6 a)
    - Confidence: medium

### Related state

16. **Charge turrets.** While its rearm countdown runs, a charge-turret vehicle shows turret
    number *c* = trunc(*N* × *r* / *D*) clamped to 0 … *N* − 1, where *r* is the rearm time remaining and
    *D* the delay last chosen (after any berserk halving); with *D* ≤ 0, or once the countdown is
    over, *c* = 0. This is recomputed in each of its turns. It applies only to a charge-turret type
    with a turret count *N* above 0 that is not a gattling type. The turret number is for drawing
    only: it does not select the weapon. A charge-turret type's weapon is the slot of its weapon
    number, which stays at 0 because the type ignores every request to change it (S-0308, claim 5),
    so such a vehicle fires its slot-0 weapon throughout the countdown.
    - Backing: `measurable` (scenario F13)
    - Confidence: medium

17. **Damage can empty a magazine.** A type with the imported damage-reduces-readiness flag loses
    ammunition when damaged and restarts its reload countdown (claim 11) at once. The amount
    belongs to S-0302.
    - Backing: `measurable` (scenario F10, part d)
    - Confidence: low

18. **Draws taken by one shot.** A shot takes, from the scenario random stream and in this order:
    the spray pick of S-0313 (only for the first shot of a burst of a spray-attack type), the
    projectile's own draws (S-0306), the draws of creating an effect weapon's particle system
    (S-0202, claim 10, when this shot starts the effect), then the one delay draw of claim 2 or 4 if
    the delay takes one, and after it the draws of the effects the shot creates last: its
    muzzle-flash animation, a sonic wave or a laser (S-0202, claim 10, and the specs of those
    effects). A refused attempt takes no draw at all. A delay without a draw (claims 4, 5, 10) takes
    no delay draw. A void shot as in claim 15 takes the projectile's draws but no delay draw.
    - Backing: `measurable` (scenario F14, with the random-state recorder; the void-shot rule through
      S-0306 scenario P5 run d)
    - Confidence: high (one delay draw per shot, or none); medium (its place between the projectile's and the animation's
      draws)

## Formulas

**F1. Full delay** (claims 2, 6, 7, 8). *R* is the rate-of-fire field (integer, ticks), *h* the
house factor (double), *r* = R(0, 2), *v* the veteran factor (double), *n* the occupant count
(integer), *m* and *b* the occupy and bunker factors (singles, widened exactly to doubles). RZ₅₃ is
S-0205's rounding of a double result, which is round toward zero under `quirks.chop_rounding` (the
default) and round to nearest with it off; trunc is the conversion of S-0205 claim 7.

> *D*₁ = trunc(RZ₅₃(RZ₅₃(*R* × *h*) + *r*))
> *D*₂ = trunc(RZ₅₃(*D*₁ × *v*)) if the veteran condition of claim 6 holds, else *D*₁
> *D*₃ = trunc(*D*₂ / *n*) (integer division toward zero) for a garrisoned building, else *D*₂
> *D*₄ = trunc(RZ₅₃(*D*₃ / *m*)) for a garrisoned building when *m* is a number above 0, else *D*₃
> *D*₅ = trunc(RZ₅₃(*D*₄ / *b*)) for a bunkered vehicle when *b* is a number other than 0, else *D*₄
> *D* = trunc(*D*₅ / 2) (toward zero) for a berserk firer (claim 8), else *D*₅

Every value is a whole number of ticks. Examples (with *h* = 1, *v* = 0.5 as a double, *r* = 1):
*R* = 30 gives *D*₁ = 31 and a veteran's *D*₂ = 15; a garrison of *n* = 4 occupants with *m* = 2 then
gives *D*₃ = 3 and *D*₄ = 1.

**F2. House factor** (claim 3). With *ℓ* the difficulty row's rate-of-fire factor and *c* the country's
(doubles): *h* = RZ₅₃(*ℓ* × *c*) in a skirmish; *h* = *ℓ* in a campaign mission.

**F3. Reload delay** (claim 11). Integers, ticks. With *A* the maximum ammunition, *a* the current
ammunition, *E* the empty-reload field, *L* the reload field, *I* the reload-increment field and *W*
the pip-wrap field:

> delay = *E* when *a* = 0 and *E* ≠ −1,
> otherwise delay = *L* + *I* × *q*², with *q* = 1 if *W* = 0, else *q* = trunc(*a* / *W*).

The product and sum are 32-bit integer arithmetic (S-0205 claim 1).

**F4. In-burst delay** (claim 4). After the *j*-th shot of a burst (1 ≤ *j* < *B*): the type's burst
delay *j* (an integer, ticks) for a vehicle when *j* ≤ 4 and that field ≠ −1; else R(3, 5). For a
berserk firer that value is then halved and truncated toward zero, as in the last step of F1
(claim 8, which exempts a disc laser shot). Integer arithmetic throughout.

**F5. Charge-turret number** (claim 16; multi-image, non-gattling types only). *c* = min(*N* − 1, max(0, trunc(*N* × *r* / *D*))) when
*D* > 0 and the countdown runs; else 0. Integer arithmetic.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Full-delay draw | R(0, 2) | ticks | Claim 2; one draw |
| In-burst delay draw | R(3, 5) | ticks | Claim 4; when no burst-delay field applies |
| Burst-delay fields read | 4 | fields | Claim 4; positions 1 to 4 of a burst, vehicles only |
| Firing sync positions | 2 | burst positions | Claim 13; burst positions 0 and 1 |
| Building multi-round delay | 1 | ticks | Claim 10 |
| Veteran factor, absent-key default | 1.0 | factor | Import mapping |
| Occupy and bunker factors, absent-key default | 1.0 | factor | Import mapping |
| Difficulty-row and country factors, absent-key default | 1.0 | factor | Import mapping |
| Burst count, absent-key default | 1 | shots | Import mapping |
| Rate-of-fire field, absent-key default | 0 | ticks | Import mapping |
| Maximum and initial ammunition, absent-key default | −1 | rounds | Import mapping; −1 = unlimited |
| Empty-reload field, absent-key default | −1 | ticks | Import mapping; −1 = not used |
| Reload, reload increment, pip wrap, absent-key default | 0 | ticks, ticks, rounds | Import mapping |

## State and ordering

**A firer's firing state.**

| State | Event | Result |
|---|---|---|
| ready (no countdown running) | an attempt passes the firing check | shot made; its delay is chosen from its burst position *j* = *k* + 1 (claims 2 to 8); the burst counter becomes *j* mod *B*; the countdown runs |
| countdown running | an attempt | answer *rearming*, unless a reason of rank 1 to 4 gives its own answer (claim 9) or it is claim 13's sync tick; no shot, no draw |
| countdown running | *D* ticks have passed since the shot | ready |
| any | the firer's target is cleared | burst counter 0; countdown unchanged |
| building, delayed shot pending | its turn max(*d* − 1, 0) ticks after the turn that started the shot | firing check with range: a shot at the current target on *can fire*, otherwise dropped; no longer pending (claim 14) |

**Ordering consequences.**
- The delay draw of a shot is taken in the firer's turn, after the draws of the projectile it
  launches and before those of its muzzle-flash animation (claim 18); two firers that fire in the same tick take their draws in update order
  (S-0203).
- A shot made in tick *t* with delay *D* ≥ 1 allows the next shot in tick *t* + *D* at the
  earliest (claim 1), so measured shot intervals are *D* ticks when the firer attempts every tick.
- A building with a pending delayed shot reports *rearming*; under S-0215 claim 32 its attack
  behaviour is therefore processed every other tick (it keeps turning towards its target) until the
  shot. The delayed shot itself is timed by the building's own turns, one per tick, so its tick is
  the same whether or not a processing falls in it (claim 14).

## Edge cases and quirks

- **Building rounds** (`quirks.building_ammo_fast_rearm`, default on). A building whose ammunition
  is above 1 rearms in 1 tick after every shot (claim 10). With the switch off, the delay is the
  normal one of claim 2 at every ammunition level.
- **Shared wave slot** (`quirks.shared_wave_slot`, default on). Sonic waves and magnetic-beam waves
  share one effect slot per firer (claim 5). With the switch off, each has its own slot and a live
  magnetic wave does not affect a sonic weapon.
- **Bare-rate effect weapons and berserk.** A sonic weapon's delay is the bare field and is still
  halved when berserk (claim 8 applies after the choice). This is the original behaviour; no switch.
- **The campaign ignores the country factor** (claim 3). Original behaviour; no switch.
- **Burst count 0 or less.** The original fails on a burst count of 0 (a division by zero). Our
  engine treats any burst count below 1 as 1, and the importer warns, naming the weapon. Policy:
  bad data gives a safe deterministic result plus an importer warning.
- **Negative or zero delays.** A rate-of-fire field of 0 or less, a negative house, veteran or
  bunker factor, or a burst-delay field below −1, can give a delay of 0 or below; the countdown is
  then over at once (claim 1). The importer warns for a negative rate-of-fire field, a negative
  factor, or a burst-delay field below −1. A bunker factor or an occupy factor above 0 and below 1
  lengthens the delay; no warning.
- **Occupy factor not above 0, bunker factor 0.** Then that division is skipped (claim 7). An
  occupy or bunker factor that is not a number also skips its division.
- **Ammunition in reload arithmetic.** F3 is integer arithmetic. A pip-wrap field below 0 gives a *q* of
  the opposite sign to *a*, or 0 when *a* is smaller than the field's magnitude, because the
  division is toward zero; the square of *q* is never negative. No warning.

## Import mapping

| Key (file, section) | Concept | Form | Absent |
|---|---|---|---|
| `ROF` (rules, weapon) | rate-of-fire field *R* | integer, ticks | 0 |
| `Burst` (rules, weapon) | burst count *B* | integer | 1 |
| `IsSonic`, `UseSparkParticles`, `UseFireParticles`, `IsRailgun` (rules, weapon) | effect weapon kinds (claim 5) | boolean | no |
| `IsMagBeam` (rules, weapon) | magnetic-beam weapon (claim 5, Terms) | boolean | no |
| `Temporal` (rules, weapon) | temporal weapon (Terms) | boolean | no |
| `Spawner` (rules, weapon) | spawner weapon (Terms, claim 9) | boolean | no |
| `HunterSeeker` (rules, vehicle type) | seeker type (Terms, claim 9) | boolean | no |
| `DiskLaser` (rules, weapon) | disc laser weapon (claim 8) | boolean | no |
| `DecloakToFire` (rules, weapon) | needs the firer uncloaked (claim 12) | boolean | yes |
| `BurstDelay` followed by 0, 1, 2 or 3 (rules, vehicle type) | burst delay *j* = 1 … 4 (claim 4) | integer, ticks | −1 |
| `FiringFrames` (art, vehicle image) | firing frames *F* (claim 13) | integer | 0 |
| `FiringSyncFrame` followed by 0 or 1 (art, vehicle image) | firing sync frames for burst positions 0 and 1 (claim 13) | integer | −1 |
| `Ammo` (rules, type) | maximum ammunition *A* | integer | −1 |
| `InitialAmmo` (rules, type) | initial ammunition (how a new object's *a* is set from it and from *A* belongs to the production specs) | integer | −1 |
| `Reload`, `EmptyReload`, `ReloadIncrement` (rules, type) | *L*, *E*, *I* of F3 | integer, ticks | 0, −1, 0 |
| `PipWrap` (rules, type) | *W* of F3 | integer | 0 |
| `ManualReload` (rules, type) | no reload restart on firing (claim 11) | boolean | no |
| `MobileFire` (rules, type) | reload advances while moving (claim 11); see S-0310 | boolean | yes |
| `DamageReducesReadiness` (rules, type) | claim 17 | boolean | no |
| `VeteranAbilities`, `EliteAbilities` (rules, type) | the entry `ROF` in either list sets the veteran or elite rate-of-fire ability (claim 6) | list | — |
| `VeteranROF` (rules, `General`) | veteran factor *v* | decimal (S-0003, claims 6 and 7), held as a double | 1.0 |
| `OccupyROFMultiplier`, `BunkerROFMultiplier` (rules, the combat-damage section) | *m*, *b* | decimal (S-0003, claims 6 and 7), narrowed to single precision | 1.0 |
| `ROF` (rules, the three difficulty-row sections `Easy`, `Normal`, `Difficult`) | difficulty-row factor *ℓ* (easy, normal, difficult row) | decimal (S-0003, claims 6 and 7), held as a double | 1.0 (see S-0006 for a layer that has the section but not the key) |
| `ROF` (rules, country section) | country factor *c* | decimal (S-0003, claims 6 and 7), held as a double | 1.0 |
| `CompEasyBonus` (rules, `General`) | the computer-easy-bonus flag of claim 3 (the original's set-up only; read by the optional handicap setting) | boolean | no |
| `CanBeOccupied`, `CanOccupyFire` (rules, building type) | garrisoned building (Terms) | boolean | no |
| `IsAnimDelayedFire`, `DelayedFireDelay` (art, building image) | claim 14 | boolean; integer, ticks | no; 0 |
| `IsChargeTurret`, `TurretCount` (rules, vehicle type) | charge turret and *N* (claim 16) | boolean; integer | no; 0 |

The importer warns for a `Burst` below 1, a negative `ROF`, a negative factor, and a burst-delay
value below −1 (Edge cases).

## Measurement scenarios

All scenarios use stock data unless a named test rules file is listed, under the measurement setup
(glossary), a small purpose-made skirmish map, one game speed (any; claim 1 says it does not matter),
and the random-state recorder where draws matter. "Shot ticks" means the tick numbers at which the
firer's shots are made, read from projectile creation or the target's health drops.

**F1. Rearm countdown and full delay.**
- Setup: one human-house vehicle with a single-shot weapon (burst count 1, not an effect weapon) and
  a stationary enemy target building with high strength in range; the house uses the normal row; the country factor is 1.0 (test rules file sets it if needed). Run a second time at another
  game speed.
- Observable: shot ticks over 20 shots; the recorder's draw results for R(0, 2).
- Expected: each interval equals *D*₁ of F1 for that shot's draw *r*; the first shot
  comes in the first tick in which the vehicle is facing the target; the intervals in ticks are the
  same at both speeds.
- Falsified if: any interval differs from that value, or the intervals change with game speed.

**F2. House and veteran factors.**
- Setup: as F1, with a test rules file that sets the house's normal-row factor to 1.5 and its
  country factor to 0.8 (expected *h* = 1.2), and a run in a campaign-style single-player map,
  played at the difficulty whose row the test file sets, where the country factor must not count,
  and optionally computer houses at each lobby difficulty (the original's set-up of claim 3, recorded
  and not reproduced by our engine); a veteran vehicle of a type with the veteran
  rate-of-fire ability, and an elite one of a type with only the veteran ability; *v* = 0.5.
- Observable: shot intervals and draws.
  A further run sets *v* by test data, once written as 0.6 and once as 60%, for a weapon whose
  *D*₁ is 10.
- Expected: F1 and F2 exactly; elite gets the factor once. The 0.6 run gives *D*₂ = 6 (the single
  just above 0.6), the 60% run *D*₂ = 5 (a double just below 0.6 under the round-toward-zero mode).
- Falsified if: the country factor acts in the campaign run, does not act in the skirmish run, the
  skirmish intervals do not follow *h* = 1.2, the elite interval shows the factor twice, the 0.6 run
  gives a *D*₂ other than 6, or the 60% run gives one other than 5.

**F3. Bursts.**
- Setup: a vehicle whose weapon has burst count 3 (test rules file), then the same with burst delay
  fields 0 and 1 set to 7 and 9; an infantry unit with a burst-3 weapon.
- Observable: shot ticks and draws.
- Expected: without burst-delay fields, intervals inside a burst are 3, 4 or 5 matching the draws,
  and after the third shot the full delay; with the fields, the vehicle's intervals are 7 and 9 with
  no draw; the infantry unit always draws R(3, 5). Clearing the target in mid-burst (stop order) and
  giving a new one starts a fresh burst. Changing the target (not clearing it) in mid-burst keeps
  the burst going.
- Carry-over: a vehicle with a burst-3 weapon in one slot and a burst-2 weapon in the other (test
  rules file), made to fire the first once and then the second. Expected: the second weapon's shot is
  position 2 of 2 of its burst, so its delay is the full delay.
- Falsified if: in-burst intervals use the full delay, the counter is not reset by the stop, a
  change of target resets it, or the carried count does not decide the second weapon's delay.

**F4. Effect weapons.**
- Setup: a stock vehicle with a sonic weapon; a stock unit with a fire-particle or spark-particle
  weapon; a stock railgun weapon if available, else a test rules file.
- Observable: shot ticks; draws.
- Expected: no delay draw is taken; intervals are max(*R*, effect lifetime).
- Falsified if: a draw is taken for the delay, or the interval is below *R*.
- Shared slot: with test data, a vehicle that carries a sonic weapon and a magnetic-beam weapon:
  fire the magnetic one, then try the sonic one while the magnetic wave is alive. Expected: no sonic
  shot until the wave ends. Falsified if the sonic weapon fires at once.
- Other weapon: with test data, a firer with a sonic weapon in slot 0 and an ordinary weapon in slot
  1, and a firer with the two the other way round. Fire the sonic weapon and ask for the ordinary one
  while the wave is alive. Expected: the ordinary weapon does not fire until the wave ends (the check
  answers *impossible*). Falsified if it fires.

**F5. Garrisons and bunkers.**
- Setup: a garrisonable building with 1, 2 and 4 occupants of one infantry type attacking a target,
  a vehicle inside a tank bunker; the stock occupy and bunker factors.
- Observable: shot intervals of the building and the bunkered vehicle.
- Expected: F1 with *n* and *m*, and with *b*.
- Falsified if: the intervals do not scale as F1 says.

**F6. Berserk.**
- Setup: a vehicle made berserk by a stock berserk warhead, firing at a nearby unit.
- Observable: shot intervals and draws.
- Expected: half the claim-2 delay, truncated. A further run, with test data giving the berserk
  vehicle a disc laser weapon: the interval is the full delay.
- Falsified if: the interval is the full delay or rounds up, or the disc laser interval is halved.

**F7. Precedence of answers.**
- Setup: (a) a building defence whose countdown runs when its target moves out of range,
  (b) a vehicle with *A* = 1 whose countdown runs when its round is spent; (c) a cloaked unit whose
  weapon needs decloaking; (d) a deploy-to-fire vehicle on a sloped cell (S-0310, scenario T7), with
  a target outside its aiming tolerance.
- Observable: whether the building keeps its target and turns (S-0215, claim 32, second row) until
  its countdown ends; when the vehicle reports out of ammunition; when the cloaked unit uncloaks; and,
  in (d), the vehicle's turret or body facing.
- Expected: (a) the building keeps the target until the countdown ends and drops it only at the next
  processing after; (b) out of ammunition shows only after the countdown; (c) the unit uncloaks before
  its first shot; (d) the vehicle does not re-aim, as the vehicle gets *must deploy* and not *not
  facing*.
- Falsified if: (a) the building drops its target while its countdown runs; (b) *out of ammunition*
  shows while the countdown still runs; (c) the unit fires while in any cloaking state other than
  fully visible; (d) the vehicle re-aims.

**F8. Facing only when ready.**
- Setup: S-0206's turret scenario with a target that moves round the vehicle during its rearm period.
- Observable: the turret facing every tick.
- Expected: the facing-error re-aim of S-0206 claim 9 acts only in ticks when the weapon is ready.
- Falsified if: the re-aim acts during the rearm period.

**F9. Ammunition.**
- Setup: test rules file: a building defence with maximum ammunition 3 and initial ammunition 3,
  and an infantry unit with maximum and initial ammunition 2 (both keys set, so the result does not
  depend on how a new object's ammunition is derived).
- Observable: shot ticks, the number of shots, and when the building drops its target.
- Expected: the building fires in ticks *t*, *t* + 1 and *t* + 2 and never again. It keeps its
  target and turns (S-0215, claim 32, second row) until the full delay after the third shot has
  run, and drops it at its first processing after that. The infantry unit fires twice and never
  again.
- Falsified if: the building fires a fourth time, its intervals differ, it drops its target while
  its countdown runs, or either regains rounds.

**F10. Reloading.**
- Setup: test rules file: a vehicle with maximum ammunition 4, reload field 50, reload increment
  10, pip wrap 2 and empty-reload field 120. Parts: (a) fire all rounds, then wait. (b) Fire one round
  every 30 ticks. (c) As (a), with the mobile-fire flag off, while driving. (d) A type with the
  damage-reduces-readiness flag that is shot.
- Observable: the ammunition pips over time.
- Expected: (a) the first round comes back 120 ticks after the last shot (*a* = 0 uses the
  empty-reload field), the second 50 ticks later (*a* = 1: *q* = 0), the third and fourth 60 ticks
  apart (*a* = 2 and 3: *q* = 1). (b) No round comes back while the vehicle fires every 30 ticks.
  (c) No progress while it drives. (d) Rounds are lost and the countdown restarts.
- Falsified if: any delay differs from F3, a round comes back while the vehicle fires every 30
  ticks, the countdown advances while it drives (part c), or the damaged vehicle in part (d) keeps
  all its rounds or does not restart its countdown.

**F11. Firing frames and sync.**
- Setup: test art file: a sprite vehicle with 4 firing frames and firing sync frame 3 for burst
  position 0, a weapon with burst count 1 and a long rate-of-fire field.
- Observable: shot ticks.
- Expected: the first shot starts the counter at 7; the next shot comes in the vehicle's turn in
  which the counter equals 3 (4 turns later), not at the rearm delay; the shot after that follows
  the rearm delay again.
- Falsified if: the shot ticks follow the rearm delay only.

**F12. Animation-delayed building.**
- Setup: a stock building with the animation-delayed-fire flag (or a test art file giving it a
  delayed-fire field of 10) attacking a target.
- Observable: tick of the attack decision (turret stops on target) and tick of the shot.
- Expected: the shot comes *d* − 1 ticks after the turn that started the animation.
- Falsified if: it comes at another tick.

**F13. Charge turret.**
- Setup: a stock charge-turret vehicle firing repeatedly.
- Observable: the turret image index each tick, and which weapon (the projectile type) each shot uses.
- Expected: F5 for the image; every shot uses the slot-0 weapon.
- Falsified if: the index does not follow the remaining countdown, or a shot uses another slot's
  weapon while the image is not 0.

**F14. Draw order.**
- Setup: the recorder; a vehicle whose projectile takes a known number of draws (S-0306) firing
  alone on the map.
- Observable: the recorded raw values taken in the firing tick.
- Expected: the projectile's draws, then one R(0, 2) (or R(3, 5) inside a burst), then the
  muzzle-flash animation's draws (S-0202, claim 10).
- Falsified if: a second delay draw appears, or the delay draw comes before the projectile's.
- Void-shot part: in S-0306's scenario P5 run d the recorder shows the projectile's draws, if the
  projectile takes any, and no delay draw; the countdown, burst counter and ammunition are
  unchanged. Falsified if a delay draw is taken or the countdown starts.

## Open questions

- Claim 9 lists only the members of *illegal* and *impossible* that matter here; the combat specs
  named there and the feature specs own the rest. The precedence among those is not stated.
- The amount of ammunition lost to damage (claim 17) is S-0302's.
- Aircraft ammunition and rearming at landing pads belong to the aircraft specs.
- The support charging of a prism-type building (claim 14) needs a structures spec.
- The temporal weapon is believed to restart the rearm countdown when its beam lets go. That is not
  a claim of this spec; whether it holds, with its backing and confidence, belongs to the combat
  spec of that weapon.
- Claim 13: once a running firing-frame counter has passed the sync frame *s* (after the sync shot,
  or when no shot was made in that tick), does an attempt with the primary weapon still get
  *rearming* from the gate until the counter is idle, or does the rearm countdown alone decide?
  Scenario F11 cannot tell the two apart while the rate-of-fire field is long.
- Claim 5: does a magnetic-beam weapon get *rearming* while the firer's own magnetic wave (or, with
  the shared slot, a sonic wave) is alive, as an effect weapon does for its own kind? And with
  `quirks.shared_wave_slot` off, does the magnetic wave's own slot block the magnetic-beam weapon?
- Claim 9, rank 16: which of the answers of claim 9 the locomotor's own refusal gives (S-0310,
  claim 7).
