---
id: S-0301
title: Firing cycle (delay between shots, bursts, ammunition and reloading, the firing check)
topic: combat
revision: 73
sources-class: [decomp, measurement, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-05
measurements: [M-0004, M-0021]
requests: [Q-0060, Q-0798, Q-1523, Q-1580, Q-1770, Q-2181, Q-2246, Q-2416, Q-1714, Q-2567]
supersedes: []
---

# S-0301: Firing cycle (delay between shots, bursts, ammunition and reloading, the firing check)

## Summary

When an object that has fired may fire again: the delay between shots and how rate-of-fire
bonuses combine into it, bursts and the short delays inside a burst, ammunition and reloading,
delayed and animation-synchronised shots, and the firing check, the per-attempt test whose answer
says whether a shot can be fired now and, if not, why. Aiming tolerances and turret behaviour are
S-0310; where a shot starts is S-0313; which weapon is used is S-0308; range is S-0307.
Drain attempts replace ordinary shots (S-0608 claim 1; claim 9 here). Disc-laser firing and its
lasting effect are S-0615 claims 12 to 17. Aircraft carrying a first passenger replace ordinary
shots with drop attempts (S-0522 claim 11), outside the ordinary-shot rules below. Which shot animation a shot makes,
where it is and what it follows are claims 20 to 23; the target line that a vehicle with the
target-laser flag shows its owner is claim 24.

## Terms

Glossary terms are used as defined there (*weapon*, *tick*, *tick number*, *turn* in both senses,
*target*, *destination*, *house*, *country*, *vehicle*, *infantry*, *building*, *aircraft*,
*scenario random stream*, *ranged draw*, *round-toward-zero mode*, *importer warning*, *quirk
switch*, *departure*, *AI house*, *speed-normalised delay*). From S-0202: R(*a*, *b*), the ranged
draw. From S-0205: RZ₅₃ and trunc (Formulas there). From S-0203: *turn pass*. In addition:

- **Firer.** An object that fires a weapon: a vehicle, infantry, aircraft or building.
- **Shot.** One firing of one weapon by a firer: one projectile, beam or effect launched, or, for a spawner weapon, one target handed to the spawn controller as a pending target. A shot is *made* when the weapon's firing action succeeds; an attempt that the firing check refuses, or whose projectile launch fails (claim 15), makes no shot.
- **Draining weapon, drain attempt, drainable type, draining, being drained.** As S-0608
  claims 1 to 3 and Terms. A drain attempt replaces an ordinary shot and does not count as a
  shot for this spec's countdown, burst, per-shot ammunition or ordinary-shot draw rules
  (claim 9). Aircraft pending run rounds remain separate (S-0608 claim 16; S-0501 claim 15).
- **Drop attempt.** A carrying aircraft's attempt to release its first passenger instead of an
  ordinary shot (S-0522 claim 11). Neither success nor refusal counts as an ordinary shot here.
- **Void shot.** An attempt that passed the firing check but whose projectile could not be put on
  the map (claim 15). It makes no shot.
- **Rearm countdown.** A per-firer countdown in ticks. It is set by every shot the firer makes, to
  that shot's *delay* (claims 2 to 8 and 10), and it is *running* until that many ticks have passed. While it
  runs, the firer may not fire any of its weapons, except for the sync shot of claim 13 and
  the unchecked shots of claim 9. One countdown serves all weapons of a firer.
  Charge arrival also replaces it without a shot (claim 19).
  S-0215's *rearm time remaining* is the number of ticks still to run.
- **Rate-of-fire field** *R*. The imported weapon field that sets the base delay between shots, in
  ticks (Import mapping).
- **Burst count** *B*. The imported weapon field that sets how many shots one burst has.
- **Burst counter** *k*. A per-firer integer, 0 when the firer is created. It counts the shots
  made in the current burst, and one counter serves all weapons of a firer.
- **House rate-of-fire factor** *h*. A per-house double, set when the houses are set up (claim 3).
  The neutral and special houses (glossary, *house*) have *h* = 1.0 exactly.
- **Difficulty row.** As S-0302 (Terms): the entry of the glossary's *difficulty skill block* for
  one difficulty, one of the three rules sections *easy*, *normal* and *difficult* (Import
  mapping). Each row has a rate-of-fire factor *ℓ*. The *normal row* is the one the original gives
  a human-controlled house in a skirmish. Which row a house uses is set-up behaviour (claim 3).
- **Veteran, elite.** The two raised veterancy ranks of a unit, as S-0311 (veterancy)
  defines them; S-0311 claim 3 owns the test of whether a unit has an ability. A type's *veteran rate-of-fire ability* and *elite rate-of-fire ability* are two
  imported flags that say whether the raised rank speeds up its firing.
- **Veteran rate-of-fire factor** *v*. A game-wide double from the rules data, holding a decimal
  read as S-0003 claims 6 and 7 state (a single-precision value unless written with a percent
  sign). The difficulty-row and country factors are held the same way.
- **Garrisoned building.** A building whose type can be occupied by infantry and may fire while
  occupied (two imported flags), and which has at least one occupant. Its occupant count is *n*,
  with the original/engine distinction of claim 7.
- **Occupy factor** *m*, **bunker factor** *b*. Two game-wide single-precision decimals from the
  rules data (Import mapping).
- **Bunkered.** A vehicle that is inside a tank bunker (S-0206, Terms).
- **Berserk.** The state of a unit that a berserk-causing warhead has put out of its owner's
  control (S-0610 claims 4 to 6). *Berserk-causing warhead*, *psionics immunity* and
  *berserk-friendly* have the meanings of S-0610 Terms.
- **Effect weapon.** A weapon whose shot keeps a lasting effect attached to the firer: a sonic wave,
  a spark-particle or fire-particle emitter, or a railgun particle trail (each an imported weapon
  flag). A sonic wave is *alive* from the shot until it ends (the combat and rendering specs own its
  duration). A particle emitter's weapon flag sets its *emitter place* (fire, spark or railgun,
  S-0616 Terms), and the behaviour kind of its system's type sets its *kind*. An emitter whose kind
  matches its place is called *matched*. Every emitter, matched or not, is *alive* while its place
  holds it. Claim 5's place-and-kind rule gives what marks its system and when its place is freed.
- **Disc laser weapon.** A weapon with the imported disc-laser flag (S-0203, Terms).
- **Magnetic-beam weapon.** A weapon with the imported magnetic-beam flag. Its shot can make a
  *magnetic wave* attached to the firer (S-0607 claim 21). The firer's *held target* is its
  gripped unit (S-0607 claim 5); that grip lasts until an event of S-0607 claim 12, independently
  of the wave's lifetime. S-0607 owns the lift and release.
- **Temporal weapon.** A weapon whose warhead has the imported temporal flag: an erasing weapon
  (glossary, *being erased*). A firer's persistent erasing capability is initially supplied by
  its primary weapon's warhead. Its *temporal beam* is attached to its target from
  the moment it locks on until it lets go. S-0308 (claim 6) and S-0310 (claim 7) also use the term.
- **Holding a target.** A firer *holds* a target with a temporal weapon while its temporal beam is
  attached to that target, and with a magnetic-beam weapon while that target is its held target.
- **Spawner weapon, spawned aircraft.** A *spawner weapon* is a weapon with the imported spawner
  flag. Its shots launch no projectile; they hand the shot's target to the spawn controller as
  a pending target (S-0506 claim 3), rather than launching aircraft immediately. The firer's
  *spawned aircraft* are the slot objects whose life cycle S-0506 owns. A spawned aircraft is
  *alive* while its slot is not empty (S-0506 Terms). A firer is *launching* spawned aircraft
  while S-0506's launching count is above 0 (S-0506 Terms and claim 10).
- **Seeker type.** A vehicle type with the imported hunter-seeker flag (Import mapping). Only its
  effect on the firing check (claim 9, rank 9) is specified here.
- **Playing death frames.** A vehicle whose type has death frames is *playing* them from the tick its
  death counter starts (S-0102, claim 22), until S-0312 removes it.
- **Materialising.** The state of an object that has just teleported and is phasing in at its
  destination. For the teleporting locomotor's own move, this is exactly *phasing in* of
  S-0207 claim 20(f): its delay is Formula F9 there, with that claim's end tick and harvester
  exception. Superweapon and other effect teleports remain owned by their feature specs.
- **Gattling type.** As S-0308 (Terms). **Spray-attack type.** As S-0313 (Terms).
- **Ammunition** *a*. A per-firer integer, in rounds. A type's *maximum ammunition* *A* and
  *initial-ammunition field* *J* are imported integers. An initial field of −1 selects *A*
  (claim 10); ammunition of −1 is unlimited. A maximum of −1 disables vehicle reloading,
  but does not override an explicit initial field.
- **Reload countdown.** A per-firer countdown in ticks, used only by vehicles (claim 11).
- **Firing check.** The test made by attempts to fire a weapon at a target, apart from the
  unchecked shots of claim 9. Its *answer*
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
- **Shot animation.** The animation a shot makes at its start (claims 20 to 23), as S-0109
  claim 29 names it. A weapon has an ordered *animation list* of zero or more animation types,
  numbered from entry 0, and may name one *occupant animation* and one *open-topped animation*
  (Import mapping). It also may name an assault animation, which ordinary shots never use.
- **Target line, line timer.** The line a vehicle with the imported target-laser flag shows its
  owner after it fires, and the per-vehicle timer that decides how long it is shown (claim 24).
- **Shot facing.** The facing of a firer that chooses an entry from an animation list of
  exactly eight entries (claim 21).
- **Launch point.** The point from which a shot's projectile is launched, height included: the
  fire point of S-0313 claims 1 and 5 to 8, or the firer's own position for a dropping
  projectile (S-0306 claim 5).
- **Passenger firing out.** An infantry passenger firing from inside an open-topped transport
  (S-0313 claim 5; glossary, *open-topped transport*).

## Behaviour

### The delay between shots

1. **Rearm countdown.** When a firer makes a shot in the tick with tick number *t*, its rearm
   countdown is set to that shot's delay *D* (claims 2 to 8 and 10). The firing check then answers
   *rearming* for every weapon of the firer, in every tick with tick number below *t* + *D*, and
   no longer from tick *t* + *D* on. This holds unless a reason of ranks 1 to 4 gives its own answer (claim 9), and
   except in the sync tick of claim 13, between shots of an aircraft's same-tick full burst
   (claim 4), and for claim 9's unchecked building secondary attempts, in which the countdown
   does not hold the shot back. Apart from those exceptions,
   *D* ticks pass from one shot to the next at the least. With *D* ≤ 0 the countdown is over at once. How often a firer attempts to fire (for a vehicle, once in
   each of its turns) belongs to S-0215 and to claim 9 here. A firer that has never fired, including
   one created during play, has no running countdown: it can fire as soon as it has a target. The
   countdown is not changed by switching weapons or targets. Charge arrival replaces it without
   a shot under claim 19, including when a countdown already runs. Counting is in ticks: game speed does
   not change *D*, and `quirks.speed_normalised_delays` does not apply to any delay of this spec.
   A carrying aircraft's drop attempt instead leaves the countdown unchanged on refusal and
   expires it immediately on success; it sets no ordinary shot delay (S-0522 claim 11).
   - Backing: `measured` (M-0004: the countdown, its independence of the game speed, and that a
     retarget or a change of weapon leaves it running); `measurable` (scenario F1, fifth run, for
     a turret turn shorter than the countdown; S-0522 scenario P7 for drop attempts, scenario F18
     for charge arrival)
   - Confidence: high

2. **Which delay a shot gets.** Four kinds of delay exist: the 1-tick delay of a building with
   spare rounds (claim 10), the bare rate of an effect weapon (claim 5), the in-burst delay
   (claim 4) and the full delay below. When more than one kind could apply to a shot, the first
   in this order wins and the others are not used, so no draw of a lower kind is taken:

   | Order | Kind | Applies to a shot when |
   |---|---|---|
   | 1 | 1-tick delay (claim 10) | the firer is a building that holds more than 1 round before the shot |
   | 2 | bare rate (claim 5) | the weapon is a sonic weapon, or a spark-particle, fire-particle or railgun weapon: every shot of such a weapon, whether or not an effect of its kind was already alive (the shot starts its emitter or trail when none is) |
   | 3 | in-burst delay (claim 4) | the burst position *j* is below the burst count *B* |
   | 4 | full delay (this claim) | none of the above |

   So a building with spare rounds has no rearm-countdown gap beyond 1 tick, whatever its
   weapon's burst count or kind; other gates, including a live effect, can still prevent a shot.
   An effect weapon with a burst count of 2 or more takes its bare rate after every shot of
   the burst and never draws R(3, 5). Only the full delay takes the factors of claims 3, 6 and 7,
   and the berserk halving of claim 8 applies to whichever kind was chosen. The burst counter
   advances in every case (claim 4). These are shot delays; the non-shot delay at charge
   arrival instead uses claim 19's conditions and the unchanged burst counter.

   Creating the emitter or trail is taken to always succeed. A creation that fails because the host
   machine runs out of memory, which can happen in the original, is not modelled. That
   original failure is `unmeasured`: no scenario safely isolates host exhaustion from other
   failures; confidence medium. Successful creation is the engine design assumption.

   The full delay is *D* of Formula F1. Its base is the weapon's rate-of-fire
   field *R* times the firing house's rate-of-fire factor *h* (claim 3) plus one ranged draw
   R(0, 2) from the scenario random stream. Each floating-point operation is rounded according to
   `quirks.chop_rounding` and S-0205; each conversion to a whole number of ticks is truncated toward
   zero. The veteran factor (claim 6), the garrison division (claim 7), the bunker division (claim 7)
   and the berserk halving (claim 8) scale it where their condition holds, nested as Formula F1 nests
   them. The nesting shows in the delays: the same factors nested otherwise can truncate to a
   different whole number of ticks. Formula F1 gives the number domain of each result.
   - Backing: `measured` (M-0004: the full delay as a base plus its one R(0, 2) draw, the in-burst
     delay, the 1-tick delay of a building with spare rounds before both, and a fire-particle
     weapon's bare rate, each as a countdown); `measured` (M-0021, for the full delay's base with
     a computer house's factor: for each of its three test weapons, in every run, the shortest gap
     between one vehicle's shots is *R* × *h* with the draw at 0, converted to whole ticks as above,
     the factor's value kept local); `measurable` (scenario F2 for the factors, F4 for the
     other effect weapons and its building part for the 1-tick delay before the bare rate, F14's
     no-draw part for the absence of a delay draw with a burst-delay field, the bare rate and the
     1-tick delay, which M-0004 does not separate from the background draws)
   - Confidence: high (the full delay as a base plus one R(0, 2) draw, its factors carrying the
     ratings of claims 3, 6 and 7, and the 1-tick delay before the full and in-burst delays); medium
     (the 1-tick delay before the bare rate, until scenario F4's building part runs; that a
     burst-delay field, the bare rate and the 1-tick delay take no draw; effect weapon before
     in-burst; no delay draw was identified for M-0004's fire-particle weapon, which does not show
     that order, and no other effect kind was run; scenario F4's burst part isolates the order)

3. **The house rate-of-fire factor.** In a skirmish, *h* of a player house is the rate-of-fire factor
   of the house's difficulty row times the rate-of-fire factor of the house's country, a product of two
   doubles (Formula F2). In a campaign mission *h* is the row's factor alone; the country's factor is
   not used (this records the original; our engine's campaign rule is deferred, see below).
   - **Exception: the neutral and special houses.** The two non-player houses every skirmish creates
     have *h* = 1.0 exactly: no difficulty row and no country factor applies to them, whatever their
     countries' rate-of-fire factors or the rows hold. This is the original's rule, and our engine
     keeps it by decision of the project lead (S-0412 claim 5; S-0413 claim 8). Their delay is
     therefore the rate-of-fire field *R* plus the draw, before the other factors of Formula F1.
   - *Our engine: every player house, human or AI, uses the normal row* (a rule of our engine by
     decision of the project lead; no switch; the glossary's *difficulty skill block*). AI difficulty
     comes from the AI's strategy modules and never from this factor. A house's row can differ only
     through an optional per-player handicap setting, which is off by default and is never tied to
     an AI house's difficulty automatically. A difficulty row that a map sets for a house is
     ignored in a skirmish; only the handicap setting can change a house's row. Campaign
     difficulty is deferred to the future campaign spec (campaigns are out of scope for now).
   - *The original's set-up (a departure, no switch).* The original gives the house of every
     human player in a skirmish the normal row. The house of each computer player gets the row that
     its lobby entry selects (S-0412 claim 4 owns the pairing): the easy computer the difficult row,
     the normal computer the normal row and the hard computer the easy row. The computer-easy-bonus shift is conditional on more than one human participant and the rules' computer-easy-bonus flag being on. A skirmish has at most one human participant, including observers (S-0302, claim 3), so this shift does not occur in a skirmish. Its effect outside this spec's skirmish scope is not specified here. In a campaign mission the original takes the row of the player's chosen difficulty for the player's house and the row of the scenario's computer difficulty for the others. Our engine does not reproduce these assignments; the handicap setting can.
   - Backing: `measured` (M-0021, for the product: the gaps of its three test weapons match *h* as
     the row's factor times the country's for a hard computer house of each of its two countries,
     with the factors kept local, so the record does not show which row a lobby entry selects),
     `measurable` (scenario F2); `unmeasured` for the one-step move of the
     computer-easy-bonus flag (it needs a game with more than one human player: a skirmish does not
     have one in our engine, and network games are out of scope). The historical campaign
     rules in this claim are `unmeasured` here: campaigns are outside this spec's
     skirmish scope and have no proposed run
   - Confidence: high (the product, that the campaign uses the row's factor alone, and the factor of
     exactly 1.0 of the two non-player houses, as S-0412 scenario K5 part d measures it); high for
     the row each lobby entry selects (S-0302 scenario D19 part c measures it); medium for the
     one-step move (never observed: it needs a game with more than one human player, which a
     skirmish does not have in our engine and network games are out of scope)

4. **Bursts.** A weapon with burst count *B* fires bursts of *B* shots. A shot made while the
   firer's burst counter is *k* is the *j*-th shot of its burst, *j* = *k* + 1, and leaves the
   counter at *j* mod *B*, so the counter is 0 again after the last shot of a burst.
   Charge arrival is not a shot: it queries a delay using *k* itself, without advancing or
   wrapping the counter (claim 19). The delay
   after the shot depends on *j*:

   | Burst position | Delay after the shot |
   |---|---|
   | *j* < *B* | the *in-burst delay* below, unless claim 2's order gives a 1-tick delay or a bare rate |
   | *j* ≥ *B* | the full delay of claim 2, unless claim 2's order gives a 1-tick delay or a bare rate |

   The in-burst delay after the *j*-th shot is:
   - for a vehicle with 1 ≤ *j* ≤ 4 whose type gives a burst delay *j* that is not −1: that value,
     in ticks, with no draw,
   - otherwise one ranged draw R(3, 5) from the scenario random stream: 3, 4 or 5 ticks.

   The counter changes in the same way whichever kind of delay the shot gets. No rate-of-fire bonus applies to an in-burst delay, but the berserk halving does (claim 8). The
   burst counter is shared by the firer's weapons, so a firer that changes weapons in mid-burst
   carries the count over and the new weapon's *B* decides. When the firer's target is cleared (it
   has none), its burst counter is set to 0; a change from one target to another keeps it.
   An aircraft's full burst makes all *B* shots in one tick without a firing check between shots
   (S-0501 claim 12). Each shot still advances the burst counter and sets the countdown.
   The in-burst delays do not separate those shots. Each attempt made while a first passenger
   exists is instead a drop attempt: it never advances the burst counter or chooses an ordinary
   full or in-burst delay, including their draws (S-0522 claim 11). The exception ends when no
   first passenger remains; any further attempts in that same burst are not cargo drops.
   - Backing: `measured` (M-0004: the counter; a vehicle's fixed in-burst delays, its type's
     burst-delay fields set to 7 and 9, as countdowns; the drawn in-burst delay as a countdown for
     a building defence, and an infantry unit's in-burst R(3, 5) draws; the reset by a stop order,
     the count kept across a retarget and carried over to the other weapon, one run each), and
     `measurable` (scenario F3 for a vehicle's drawn in-burst delay, its type's burst-delay fields
     set to −1, and for an infantry unit's countdown after an in-burst shot; scenario F14's no-draw
     part for a burst-delay field taking no draw; S-0501 scenarios AC7 to AC9 for aircraft, S-0522
     scenario P7 for drop attempts).
     The reset for a target cleared otherwise than by a stop order is `unmeasured`
     here: no record clears the target in another way.
   - Confidence: high (the counter; a vehicle's fixed in-burst delays; a building's drawn in-burst
     delay; an infantry unit's in-burst draw from R(3, 5)); medium (a vehicle's drawn in-burst
     delay, and an infantry unit's in-burst delay being that draw in ticks, until scenario F3
     runs; that a burst-delay field takes no draw; the reset by a stop order, and the retarget and
     weapon-change rules, each seen
     in one run; the reset for a target cleared otherwise than by a stop order, `unmeasured`)

5. **Effect weapons fire at their bare rate.** The delay after a shot of a sonic, spark-particle,
   fire-particle or railgun weapon is the rate-of-fire field *R* itself, with no draw, no house
   factor and none of the bonuses of claims 6 and 7 (the berserk halving of claim 8 still applies).
   It outranks the in-burst delay and is outranked by a building's 1-tick delay (claim 2's order).

   A firer has at most one sonic wave and one emitter in each of its fire, spark and railgun places
   (Terms), whichever of its weapons started it. Each weapon of a firer has an
   *other weapon*: for the weapon in slot 0 it is the weapon in slot 1, and for a weapon in any
   other slot it is the weapon in slot 0 (S-0308 numbers the slots). The firing check (claim 9)
   answers as follows for the asked weapon:

   | The firer has a live effect of | Answer for the asked weapon |
   |---|---|
   | any kind whose effect flag the asked weapon has | *rearming* (rank 5) |
   | any kind whose effect flag the asked weapon's other weapon has, whatever the asked weapon's own flags | *impossible* (rank 4) |

   These flags are independent, not mutually exclusive: the gate tests fire, railgun, spark
   and sonic effects, and any one held effect of an enabled kind suffices to refuse the
   weapon. If both rows apply, rank 4 wins. A permitted particle-effect shot makes an emitter
   for every enabled particle-effect flag whose place is empty, in the order fire, spark,
   railgun. Thus one shot can make three emitters; their creation order determines their
   subsequent turn order (S-0616 claim 1). The next checked shot must wait for every effect
   selected by its flags to end (its life in Terms), as well as for the other firing restrictions
   to pass.

   With just one effect flag, an effect weapon fires again at the first attempt that is both at
   least *R*′ ticks after the shot and after the effect has ended, where *R*′ is *R* (trunc(*R* / 2)
   for a berserk firer, claim 8), and no draw is taken for its delay. For a sonic weapon the effect
   ends when the wave does, so the interval is max(*R*′, the wave's lifetime). For a particle
   emitter it ends when its place is freed (the place-and-kind rule below). For a matched emitter: a
   spark-particle or railgun weapon fires again once its emitter's system has left play. So does a
   fire-particle weapon, when its firer keeps its target, stays put and lives: it fires at the first
   attempt after both its emitter's system has left play and *R*′ ticks after the shot. When the
   system leaves play is S-0613 claim 17's and S-0616 claim 22's. For a building that holds more
   than 1 round, *R*′ is 1 instead (0 if berserk, claim 8), so the live-effect gate can still hold
   it back (claim 10).

   **Place-and-kind rule** (every particle emitter, matched or not, Q-2181). The gate follows the
   place, and the kind of the system that the place holds decides what marks that system:
   - **Marks.** Each kind keeps its own marks (S-0616 claim 22, the column for a system with an
     owner object): a spark system when its counter *K* runs out (S-0616 claim 15), a railgun
     system after its line (S-0616 claim 17), and a fire system by its own turn when its owner is
     no longer alive (S-0616 claim 10). Every kind is also marked by a positive lifetime (S-0613
     claim 17), the trigger action that marks systems (S-0616 claim 1) and its owner's removal
     (S-0616 claim 4). In addition, a let-go of the fire place marks the system that place holds,
     whatever its kind. The fire place's let-go events are a retarget to a target not within
     reach, a move order and the firer's death (S-0616 claim 1: what counts as within reach is its
     own, and the measurement in F4 reads it as the weapon's range). The spark and railgun places
     have no let-go events. So a fire-kind system held in the spark or railgun place is not marked
     by its own turn while its owner is alive and on the map.
   - **Freeing.** The fire place is freed at the first of its let-go events and its system's
     leaving play. The spark and railgun places are freed only when their system leaves play. A
     marked system leaves play in its first turn in which it holds no particle (S-0613 claim 17).
   - **Moment of a retarget's let-go (Q-2246).** The test of a retarget is made at the change of the
     firer's target, in whichever part of the tick makes that change, and not in the turn of the
     system that the fire place holds. It reads nothing of that system's kind, so it is the same for
     every kind. The retarget's mark affects only system turns after the target change. An attack
     order takes effect after the turn pass (S-0203 claim 7). So after a retarget by an order, the
     system's turn in the next tick is the first one that the retarget's mark affects. A marked turn
     is the kind's own turn with the mark set. Smoke and fire systems emit no new particle (S-0616
     claims 9 and 10), though a smoke system's dying particles still split into successors as
     S-0616 claim 8 gives. A railgun system that has not yet made its line makes none (S-0616
     claim 17).
     The mark does not stop a spark system's bursts: while its counter *K* is above 0 it bursts as
     S-0616 claim 15 gives. A gas system's turn is unchanged. Neither turn depends on the mark
     (S-0616 claim 15, S-0613 claim 6). A system of a behaviour kind keeps updating and ending its
     particles, and every system leaves play as S-0613 claim 17 gives (a system with no behaviour
     kind never updates its particles, S-0613 claim 17).
   - **Spark-turns field of 0 or less.** A spark system of such a type makes no burst. In the
     original its counter never marks it, so only the other marks above end it: held in the fire
     place, a let-go marks it, and since it holds no particle it leaves play at its next turn. Under
     S-0616 claim 20's fix, our engine marks it in its first turn, whoever made it and whatever
     place holds it, so it frees its place after that turn, and a later let-go finds the place free.

   For an emitter that is not matched, with the system's type of lifetime 0 or less, no trigger
   action and the firer alive on the map, the rule gives:
   - **Fire flag, spark-kind or railgun-kind system.** The weapon fires again at its first attempt
     that is both at least *R*′ ticks after its previous shot and made after the earlier of these
     two: a let-go event of the fire place (a retarget to a target not within reach, a move order),
     or its system leaving play (a spark system once its counter has run out and it holds no
     particle, a railgun system after its line once it holds no particle, S-0616 claims 15, 17 and
     22). A let-go marks the spark or railgun system, so it then leaves play in its first turn in
     which it holds no particle, even with its counter not run out (S-0613 claim 17).
   - **Spark or railgun flag, spark-kind or railgun-kind system.** No let-go event frees the place.
     The weapon fires again at its first attempt that is both at least *R*′ ticks after its
     previous shot and made after its system has left play by its own counter or line.
   - **Spark or railgun flag, fire-kind system.** No let-go event frees the place, and the system is
     not marked by its own turn while its owner is alive and on the map. So the weapon does not fire
     again while the firer lives on the map, whatever orders it gets, unless a positive lifetime
     (S-0613 claim 17) or the trigger action that marks systems (S-0616 claim 1) ends the system.
     The fire system goes on making its flames as S-0616 claims 10 and 11 give.
   For a system of another kind (smoke, gas or none) in any place, the same rule applies with that
   kind's end (S-0616 claim 22, S-0613 claim 17). No switch: the original's result follows from
   the data, and our engine keeps it. A firer that leaves the map without being removed (into a
   transport) is not covered here (S-0616 claim 4 leaves that case open).

   A magnetic wave shares the sonic wave's single slot, so a live magnetic wave also counts as a
   live sonic effect: it blocks a sonic weapon of the same firer (*rearming* for the sonic weapon's
   own kind, *impossible* as the other weapon's effect) (`quirks.shared_wave_slot`,
   default on; the fixed behaviour keeps two slots).

   This does not make a magnetic-beam weapon an effect weapon of this claim: it does not get
   the bare rate or the max(*R*′, lifetime) interval. Its own live wave adds *rearming* only
   when the firer grips a unit and the asked target is neither that unit nor a building
   (S-0607 claim 11; claim 9, rank 4 here). Holding that very target remains a separate early
   refusal. There is no general own-wave firing block for a magnetic-beam weapon, including
   when the shared wave is sonic. A magnetic shot made while a wave is alive creates no new
   wave (S-0607 claim 21). Grip and wave lifetimes are independent.
   - Backing: `measured` (M-0004: a fire-particle weapon's countdown is its bare rate, and its
     refire waits beyond it); `measurable` (scenario F14's no-draw part for the missing draw,
     which M-0004 does not separate from the background draws; scenario F4, with its
     multi-flag, burst and building parts; S-0607 scenarios L9 and L10 for magnetic firing,
     grip independence and wave creation; scenario F4's unmatched part for the refire of a weapon
     whose emitter is not matched, Q-2181, its second run, u1 against u8, for a let-go's mark of a
     spark-kind system in the fire place, and u3 and u4 for a move order in the spark and railgun
     places); `unmeasured` for the smoke, gas and no-kind cases of that refire (no scenario here
     sets such a type up), and for four parts of the place-and-kind rule whose reasons the
     Confidence line gives: a retarget's effect on the spark and railgun places, a let-go's mark of
     a railgun-kind system, the
     fire place's freeing at the let-go itself rather than when its marked system leaves play, and
     the original's let-go end of a spark system with a spark-turns field of 0 or less. For the
     moment of a retarget's let-go: `measurable` (scenario F4's unmatched part, third run: u10 and
     the fire firer for the first turn that the mark affects, u9 for a marked spark system's
     bursts); `unmeasured` that the test is made at the change itself rather than at the start of
     the system's next turn (nothing between the two reads the mark, so no run tells them apart),
     and for a railgun-kind system (the moment shows only for a retarget between the shot and the
     system's first turn, which no scenario here sets up) and for gas-kind and no-kind systems
     (their turns do not depend on the mark, so the moment changes nothing observable)
   - Confidence: high (the bare rate of a fire-particle weapon, *rearming* for an effect weapon's
     own kind, the any-flag gate and particle-emitter creation order, and the magnetic conditional
     gate and absence of a general own-wave block), medium (the bare rate of the sonic,
     spark-particle and railgun kinds, which no record has run, until scenario F4 runs; the missing
     draw, until scenario F14's no-draw part runs; *impossible* for the other weapon, that the
     effect's lifetime can lengthen the interval, which depends on the effect specs, and the let-go
     events of an emitter in the fire place, including what counts as within reach, which this spec
     leaves to S-0616 claim 1: scenario F4, particle part; that a matched fire weapon's refire waits
     for its system to leave play: M-0004 measured a fire weapon of stock data refiring after much
     more than *R* under one attack order, which fits the system lifetime but recorded neither the
     system nor whether the firer kept its target and stood still); low for the shared slot of
     magnetic and sonic waves (a firer needs both weapons, so test data must supply the case).
     For the place-and-kind rule, matched or not: high that the gate follows the place, that a move
     order frees neither the spark nor the railgun place (scenario F4's unmatched part, second run,
     u3 and u4), and that a fire-kind system held there is not marked by its own turn while its
     owner is alive and on the map (its first run, u3 and u4); medium, `unmeasured`, that a
     retarget to a target not within reach frees neither of those places either (no scenario here
     gives a firer in those places such a retarget; it rests on S-0616 claim 1's let-go events,
     rated medium there); high for each kind's own marks
     and for the releases by the system's own end in the three pairings listed, which rest on
     S-0616 claims 10, 15, 17 and 22, claim 4 for removal and S-0613 claim 17 (all rated high
     there); medium for the release by a let-go event in the first pairing, and medium that a
     let-go of the fire place marks the system it holds whatever its kind, both resting on S-0616
     claim 1's let-go events (rated medium there; scenario F4's unmatched part, second run, u1
     against u8, for a spark-kind system); medium, `unmeasured`, for that mark of a railgun-kind
     system (its own line marks it in its first turn, so a let-go's mark would show only before
     that turn, which no scenario here sets up; it rests on the same let-go events); medium,
     `unmeasured`, that the fire place is freed at the let-go itself rather than when its marked
     system leaves play (no scenario here records whether that system still holds a particle at
     the firer's first attempt after the let-go, so both readings give the same shot ticks; it
     rests on the same let-go events); medium, `unmeasured`, that a let-go ends a spark system held
     in the fire place whose type's spark-turns field is 0 or less in the original (no scenario
     here sets such a type in the fire place; it rests on the same let-go events); our engine's
     first-turn mark of such a system as S-0616 claim 20 rates it; medium for the smoke, gas and
     no-kind cases, which rest on those kinds' end rules. For the moment of a retarget's let-go:
     medium, `unmeasured`, that the test is made at the change of target and not at the start of
     the system's next turn, medium that the system's first turn after the change is the first one
     the mark affects, whatever the held system's kind, and medium for each kind's marked turn, all
     resting on S-0616 claim 1's let-go events (rated medium there), the marked turns also on S-0616
     claims 9, 10, 15 and 17 and S-0613 claim 6 (rated high there)

6. **Veteran factor.** If the firer is veteran and its type has the veteran rate-of-fire ability,
   or it is elite and its type has the veteran or the elite rate-of-fire ability, the delay is
   multiplied once by *v* and truncated (Formula F1). Elite does not apply the factor twice.
   - Backing: `measurable` (scenario F2)
   - Confidence: high

7. **Garrisons and bunkers.** A garrisoned building's delay is the quotient of the delay by its
   occupant count *n* (integer division, truncated toward zero), divided further by the occupy
   factor *m* and truncated when *m* is above 0 (Formula F1, *D*₃ and *D*₄). A bunkered vehicle's delay is
   divided by the bunker factor *b* and truncated when *b* is not 0 (Formula F1, *D*₅). An imported value
   that is not a number never reaches *b*: S-0003 claim 8 makes the importer treat it as absent
   (Edge cases). A building is never treated as
   bunkered.

   **Count after a death inside.** In the original, *n* includes retained dead occupant places
   (S-0508 claim 20), not just living occupants. This describes the divisor, not a promise
   of a successful shot through a dead place: subsequent use of that place has no dependable
   original result. Our engine instead uses *n* after same-update death removal under
   S-0508 claim 23, a safety departure with no quirk switch. Removed places do not contribute
   to the delay quotient. When removal leaves *n* = 0, the building is no longer garrisoned
   for this claim and Formula F1 applies neither the occupant division nor the occupy factor.
   This does not itself grant a shot or an occupant weapon.
   - Backing: `measurable` (scenario F5; S-0508 scenario G18 for engine death removal).
     Unsafe original continuation through a dead place is `unmeasured` under S-0508 claim 20:
     no safe confirming shot outcome is established.
   - Confidence: high (formula, original retained-count divisor and engine post-removal rule); medium (the occupant-count division as observed in play, since
     which occupant's weapon fires is S-0308's)

8. **Berserk halving.** A firer that is berserk when it makes a shot gets half the delay chosen by
   claim 2's order (any of its four kinds, with the factors of claims 3, 6 and 7 where they
   apply), truncated toward zero. Exception: the shot of a disc laser weapon is never halved
   (S-0615 claim 12). The
   rule is stated for the firer's state, not for its kind: whether a building can be berserk
   belongs to the warhead specs, and if it can, its 1-tick delay halves to 0.
   - Backing: `measurable` (scenario F6)
   - Confidence: high

9. **The firing check and its answers.** Except for the unchecked shots named below, every
   attempt to fire asks the firing check, with the target and the weapon. Its answer is one of
   these categories:

   | Answer | Meaning |
   |---|---|
   | can fire | the shot can be made now |
   | rearming | the rearm countdown is running, the weapon's own effect is still alive (claim 5), a building's delayed shot is pending (claim 14), a firing-frame gate is closed (claim 13), and a few early cases named in the rank table |
   | out of ammunition | the firer's ammunition *a* is 0, or an aircraft marked as a passenger carrier now carries no passengers (S-0310 claim 5; S-0501 claims 1 and 19). These causes have different ranks below |
   | not facing | the weapon is not pointed at the target (S-0310) |
   | still turning | the turret is still turning (S-0310) |
   | moving | the firer may not fire while it moves (S-0310) |
   | cloaked | the weapon needs the firer uncloaked and it is not |
   | out of range | the target is beyond the weapon's range (S-0307) |
   | illegal | the target is not a legal target for this firer and weapon. The combat specs give the cases: a versus value of exactly zero (S-0304, claim 1), an airborne or non-ground target the projectile cannot hit, or a non-unit target of a projectile that is not anti-ground (S-0306, claim 23), the healing tests (their feature specs), the takeover test (S-0601 claim 3), the berserk refusals (S-0610 claims 3 and 6), the bomb-planting and disarm tests (S-0519 claim 3), the parasite tests (S-0614 claim 5), a held organic firer (S-0614 claim 8), a sinking firer or target (S-0504 claim 5), a falling firer (S-0511 claim 10), a deactivated firer (S-0405 claims 13 and 20), the drain refusals of S-0608 claim 3 (rank 4 below), a target that has a superweapon warp pending, from the launch until its departure, with every weapon, a temporal weapon (Terms) included (S-0513 claim 39), or, for an infantry firer whose type has the keep-acting flag, a vehicle target that is melee-linked to an object other than the firer (S-0214, Terms, *melee-linked*). S-0214, claim 6(e), defers to these for what the check refuses |
   | impossible | the firer cannot fire this weapon now for a reason of its own: no weapon in the slot, the effect of its other weapon still alive (claim 5), the weapon needs power, a hidden target fails the test below (S-0516 claim 24), and similar |
   | busy | the firer is launching spawned aircraft (Terms) |
   | must deploy | a vehicle whose type must deploy before it fires, when its own cell fails the deploy-to-fire cell test of S-0517 claim 14 (a slope, a building in the cell, or a land type that does not allow it) and no reason of ranks 1 to 10 below holds. It then outranks *not facing* (S-0310, claim 9), and in the rank table also *moving* and *still turning*, an order that no game state shows (see below). On a cell that passes the test none of this applies: the vehicle gets the aiming answers as usual and S-0310 deploys it on *can fire* or *not facing* |

   **Hidden-target refusal (S-0516 claim 24).** A target hidden from the firer's house gives
   *impossible* when the firer's mean weapon damage is above 0 or the target is not allied with
   the firer. Hidden means fully cloaked without that house's sensor coverage; a building also
   counts at fade levels 11 to 15 without coverage (S-0516 claim 18). Mean weapon damage is the
   target-independent integer of S-0308 claim 15, including its ambient component and
   truncation toward zero. It is not the selected weapon's damage or the shot damage after
   firepower or armour scaling. The refusal applies even to a forced attack on an own object.
   Allied hidden targets with mean weapon damage at or below 0 pass this test only; all other
   firing restrictions still apply.

   Where more than one reason holds, the firer gets the answer of the highest rank below (rank 1
   is the highest):

   | Rank | Answer | Reasons at this rank |
   |---|---|---|
   | 1 | impossible | a vehicle that is playing its death frames (S-0102, claim 22: its death counter has started; vehicles only) |
   | 2 | busy | a vehicle that is launching spawned aircraft (S-0506 claim 10) |
   | 3 | illegal | a vehicle that is melee-linked (S-0214, Terms), whatever its target (vehicles only) |
   | 4 | illegal, impossible, rearming | most *illegal* and *impossible* reasons (those about the firer's state, the other weapon's alive effect and most target tests), the *illegal* answer of a deactivated firer (S-0405 claims 13 and 20), for every target and weapon, the *illegal* answer of an infantry healer (S-0310, claim 11), the *illegal* answer of an infantry firer whose type has the keep-acting flag against a vehicle target that is melee-linked to an object other than the firer (S-0214, Terms, *melee-linked*), the *illegal* answer when the firer is sinking (S-0504 claim 5) or falling (S-0511 claim 10), the *illegal* answer when the target is the building the firer drains (with any weapon), or when a draining weapon targets an object already being drained or of a type that is not drainable (S-0608 claim 3; the latter two tests do not apply to cell targets), the *illegal* answer for a berserk-causing warhead against a psionics-immune target, a bunkered vehicle or a tank bunker holding a vehicle (S-0610 claim 3), the *illegal* answer for a berserk firer against a berserk-friendly target (S-0610 claim 6), the *illegal* answer, with any weapon, for a target that has a superweapon warp pending, until its departure (S-0513 claim 39), the *illegal* answer for a firing vehicle that is deploying or undeploying, or for a weapon whose warhead carries its target with a locomotor against a target vehicle that is deploying or undeploying, or is an in-place deploying type with *unload* current (S-0517 claim 11), the *impossible* hidden-target refusal defined above (S-0516 claim 24), the *impossible* answer for a spawner weapon whose mobile firer is under the parasite hold regardless of organic type (S-0506 claim 10; this precedes the organic-only *illegal* test at rank 9), and the early *rearming* cases: the firer is materialising, the asked target is its own gripped unit (with any weapon, S-0607 claim 11), it already holds this very target with a temporal weapon, a magnetic-beam weapon's firer grips a unit and has a live wave while the asked target is neither the gripped unit nor a building (S-0607 claim 11), or the weapon is a spawner weapon none of whose spawned aircraft is alive (S-0506 claim 10); and for a building, the *rearming* of its pending delayed shot (claim 14). The specified precedence within this rank is given below |
   | 5 | rearming | the rearm countdown runs, the firing-frame gate of claim 13 is closed, or the weapon's own effect is alive (claim 5) |
   | 6 | (not used) | an earlier numbering kept the weapon's own live effect here, with the same answer as rank 5; the two are one rank, and the number stays unused so that other specs' pointers to the later ranks stay valid |
   | 7 | out of ammunition | the firer's ammunition *a* is 0 |
   | 8 | cloaked | |
   | 9 | out of range, illegal | the late target tests: a seeker type (Terms) answers *out of range*, whatever the distance; a warhead that cannot affect the target's armour class (the zero-versus test of S-0304 claim 1), a failed takeover test (S-0601 claim 3), a bomb-planting warhead against a target that already carries a planted bomb or a disarm warhead against one that carries none (S-0519 claim 3), the parasite tests (S-0614 claim 5), the target is sinking (S-0504 claim 5), or the firer is held and organic (S-0614 claim 8; a spawner weapon is already refused at rank 4), give *illegal* |
   | 10 | out of range | the attempt asked for the range test and the target is beyond the weapon's range |
   | 11 | must deploy, out of ammunition | a vehicle whose own cell fails S-0517 claim 14's deploy-to-fire cell test gets *must deploy* (S-0310 claim 9); an aircraft marked as a passenger carrier that now carries no passengers gets *out of ammunition*, regardless of its ammunition count (S-0310 claim 5; S-0501 claim 19). The two causes apply to different kinds of firer |
   | 12 | illegal, impossible | the remaining vehicle tests: a vehicle that is tethered (glossary) with a building as its contact gets *impossible*, and a vehicle whose default weapon is a healing weapon gets *illegal* (S-0310, claim 11; S-0314). Which of the two wins when both hold is not stated; no scenario here distinguishes that overlap |
   | 13 | moving | |
   | 14 | still turning | |
   | 15 | not facing | |
   | 16 | a refusal of the vehicle's locomotor | a vehicle whose locomotor adds a refusal of its own gets it only when no reason above holds; which answer it gives is open (Open questions) |

   **Grip refusals within rank 4 (S-0607 claim 11).** Against the firer's own gripped unit,
   *rearming* applies with any weapon and outranks every other rank-4 answer except the
   materialising firer's *rearming*, the *illegal* refusal of a firer bound to a slave master
   and, for an infantry firer, the answers asked of it first: the infantry healer's *illegal*
   (S-0310 claim 11, which ranks above the general part), the keep-acting *illegal* below and
   any answer that S-0310's open question on dying and falling sequences settles as asked of an
   infantry firer before them. In particular, target-layer and projectile refusals cannot
   displace it: the lifter keeps its gripped unit as its target even when its weapon cannot hit
   an airborne unit.
   By contrast, the magnetic-beam refusal against another non-building target while a grip
   and live wave coexist follows the target-layer and projectile tests. A failed one of
   those tests therefore gives its refusal rather than this conditional *rearming*.

   **Deactivated firers within rank 4.** A deactivated firer (S-0405 claims 13
   and 20) gets *illegal* for every target and weapon. This refusal
   yields to the slave-bound firer's *illegal*, the materialising firer's *rearming* and
   the own-gripped-target *rearming*. It outranks the remaining rank-4 answers, including
   the held-temporal-target *rearming* and absent-weapon or power-sensitive-weapon
   *impossible*. For vehicles, the earlier tube-transit *impossible*, launching-spawn
   *busy* and melee-link *illegal* answers still take precedence.

   **A materialising firer within rank 4.** A materialising firer gets *rearming* for every
   target and weapon. It yields only to the *illegal* of a firer bound to a slave master, for
   vehicles to their answers of ranks 1 to 3 (the *impossible* of a vehicle playing its death
   frames, the launching-spawn *busy* and the melee-link *illegal*) and the earlier tube-transit
   *impossible*, and for infantry to the answers that an infantry firer is asked first: the
   infantry healer's *illegal* (S-0310 claim 11), the *illegal* of an infantry firer whose type has
   the keep-acting flag (S-0214, Terms, *melee-linked*) against a vehicle target that is
   melee-linked to an object other than the firer, and any answer that S-0310's open question on
   dying and falling sequences settles as asked of an infantry firer before them. It outranks
   every other rank-4 answer, among them a falling vehicle's *illegal*, the hidden-target
   *impossible* (S-0516 claim 24), the projectile refusals of S-0306 claim 23 and the *illegal*
   for a parasite warhead against a protected object (S-0513 claim 27), and the own-gripped-unit
   *rearming*, which *Grip refusals within rank 4* above already puts below the materialising
   firer's *rearming*. The healer's *illegal* outranks every answer of the general part, the
   materialising *rearming* among them (S-0310 claim 11). So a firer that meets all three
   answers gets the healer's *illegal* first, then the materialising *rearming*, then the
   gripped-unit *rearming*. The three meet only in an infantry firer that heals and holds a unit
   it seized, since the healer and keep-acting answers are asked only of infantry firers and the
   gripped-unit *rearming* only of a firer holding a unit it seized (S-0607). Such a firer whose
   target is the unit it seized gets the healer's *illegal* at S-0309 claim 29's check whenever
   S-0310 claim 11 gives it, and drops the target there by that claim's rule for *illegal*. Its
   firing step replies to the same answer by S-0310 claim 11's reply table. A materialising one
   drops the target at the first of its reply to *illegal* in its firing step and its first
   checking turn in a check tick (S-0309 claim 29). The keep-acting *illegal* is an answer of its
   own, asked of an infantry firer before the other rank-4 reasons: an infantry firer whose type
   has the keep-acting flag gets *illegal* against a vehicle target that is melee-linked to an
   object other than the firer. This paragraph gives no answer for an infantry unit in a dying
   or falling sequence: that stays S-0310's open question, so whether a materialising infantry
   firer that is falling gets *rearming* or its falling answer rests on that question.

   **A building's pending delayed shot within rank 4 (claim 14; Q-1770).** While a building's
   delayed shot is pending, its *rearming* yields only to the building's own answers of S-0405
   table T1: an occupiable building without occupants who can fire, a drained building, a
   building in its *construction* or *selling* behaviour, a pulse cannon, and a building that is
   not in operation. It outranks every other reason of this table that applies to a building and
   gives a different answer (an own-gripped target's *rearming*, claim 9, gives the same one),
   among them the hidden-target *impossible*, the absent-weapon and power-sensitive-weapon
   *impossible*, the drain refusals (below: a draining weapon's refusals and the refusal against
   the firer's own current drained building), every row of S-0306 claim 23 (the airborne, render-layer, anti-ground, water and
   land rows) and the later ranks. So a building whose target becomes one that a row of S-0306
   claim 23 refuses while its shot is pending gets *rearming*, not *illegal*, and its attack
   behaviour keeps that target (S-0215 claim 32). The check made when the delay ends is not
   refused by the pending shot itself (claim 14), so there those rows decide.

   **Firers being erased (S-0514 claim 17).** Ordinary firing stops during erasure.
   That inactivity does not establish a firing-check answer: ordinary processing may skip
   the attempt entirely. This spec assigns neither an answer label nor a rank to erasure,
   and claims no precedence between erasure and the ranked refusals above. This does not
   permit ordinary firing during erasure.

   **Targets with a superweapon warp pending within rank 4 (S-0513 claim 39).** A target that
   has a superweapon warp pending, from the launch until its departure, answers *illegal* for
   every weapon, a temporal weapon included. Within rank 4 this answer comes after every other
   reason listed there, except the following, which it outranks: the spawner-weapon answers
   (S-0506 claim 10: the parasite-hold *impossible* and the *rearming* when no spawned aircraft is
   alive), the other weapon's live-effect *impossible*, every row of S-0306 claim 23, and the
   magnetic-beam *rearming* against a target other than the gripped unit (S-0607 claim 11). So it
   comes after, among others, the *illegal* of a firer bound to a slave master, deactivated or
   sinking, the materialising firer's *rearming*, the own-gripped-unit *rearming*, the
   held-temporal-target *rearming* (S-0514 claim 19), the drain and berserk refusals, and the
   hidden-target, absent-weapon and power-sensitive-weapon *impossible*. An infantry healer's
   *illegal* (S-0310 claim 11) and a building's answers of S-0405 table T1, its pending delayed
   shot's *rearming* included, come before it too. It also outranks every reason of ranks 5 and 7
   to 16. So a firer whose rearm countdown is running, or whose weapon's own effect is alive,
   gets *illegal* for such a target. This test is not the test of a target being erased: what
   that one answers for a target that a temporal beam holds outside a warp is not settled here.

   **A held temporal target within rank 4 (S-0514 claim 19).** The held-temporal-target
   *rearming* outranks the hidden-target *impossible* (S-0516 claim 24) and the *illegal* for a
   protected target (S-0513 claim 28). So a firer whose temporal beam is attached to a target
   that is hidden from its house or protected keeps that target at S-0309 claim 29's check,
   unless an answer that this claim ranks above that *rearming* holds, on the map or, for a
   superweapon warp member, off it (S-0513 claim 39a). Both rules are `unmeasured`, as S-0513
   claim 39a gives, and rated medium, as that claim rates the keep.

   **A building's own answers above a held temporal target (Q-2567).** Every answer of S-0405 table
   T1 ranks above the held-temporal-target *rearming*, as it does above a pending delayed shot's
   *rearming* (above), and no T1 answer ranks below it. So a building whose temporal beam is
   attached to its target gets *illegal* or *impossible* for that target, not *rearming*, while a
   row of T1 holds: while it is not in operation (S-0405 claim 8: among other cases, switched off or
   short of power, and not charged, or under a pulse), drained, occupiable without occupants who can
   fire, or in its *construction* or *selling* behaviour. It does not keep the target. It drops it
   at the first of two updates, whichever comes first: its next *attack* processing while *attack*
   is its current behaviour, where S-0215 claim 32's row for those answers drops the target and
   starts *guard* in the same processing, and its checking turn in a check tick of S-0309 claim 29,
   where that claim drops it. This holds for a target on the map and for a superweapon warp member
   that has departed (S-0513 claim 39a, *The firer*). Dropping the target does not let the beam go
   (S-0514 claim 14): a building does not fall back to rest in S-0215 claim 13's sense, so the beam
   stays attached after the drop. In its *construction* or *selling* behaviour no *attack*
   processing runs, and its behaviour stays as it is. There its turn in a check tick drops the
   target, if it still holds it then, and an opportunity target (S-0215 Terms) goes earlier: S-0215
   claim 33 drops one in every tick of either behaviour. A pulse-cannon type gets one of S-0405
   table T1's answers in every state, so no ordinary shot of its own attaches a beam, and its row
   does not arise here. A building whose type has the SAM flag takes S-0215 claim 32's SAM rows
   instead, as S-0513 claim 39a reads their power row. This follows the original, with no quirk
   switch: a building that cannot fire also keeps no target that its beam holds.

   **Drain refusals within rank 4 (S-0608 claim 3).** The refusal against the firer's own
   current drained building and the refusals specific to a draining weapon have different
   precedence, even though each answers *illegal*:
   - Against its own current drained building, the firer gets *illegal* rather than the
     held-temporal-target *rearming*, hidden-target *impossible*, absent-weapon or
     power-sensitive-weapon *impossible*, other-weapon live-effect *impossible*, or
     conditional magnetic-wave *rearming* answer. Materialisation and the own-gripped-unit
     *rearming* answers still outrank this refusal.
   - A draining weapon against an already drained object or a non-drainable type instead
     yields to the held-temporal-target, hidden-target, absent-weapon and power-sensitive-weapon
     answers. Its *illegal* outranks the other-weapon live-effect *impossible*, conditional
     magnetic-wave *rearming* and empty-spawner *rearming* answers. Rearm, ammunition,
     cloak and the rank-9 takeover test cannot displace either drain refusal. The one
     exception is a building's pending delayed shot (above): while it is pending, the
     building's *rearming* comes ahead of both drain refusals.

   The own-current-building refusal does not cover an earlier building stranded by relinking
   (S-0608 claim 3). A draining weapon is still refused there because it is already drained.
   Another weapon may fire if all remaining checks pass. Relative ordering of reasons that
   all give *illegal*, including the berserk and projectile refusals, is not required by
   these rules: their answer and consequences are identical. Apart from these rules and a
   building's pending delayed shot (above), no other precedence within rank 4 is specified here.

   **Falling and the landing turn (S-0511 claim 10).** A falling firer gets *illegal*
   at rank 4. For ordinary infantry and vehicles, the fall and any landing hit precede
   that turn's firing check. A unit still falling after its fall step therefore cannot fire,
   including an infantry unit whose firing sequence began before the drop. A unit that
   lands and survives loses this falling-specific refusal in the same turn: it may fire
   then if it attempts a shot and all other firing restrictions pass. Landing does not
   itself grant a shot or restore a target lost during the fall.

   **Drain attempts (S-0608 claim 1).** Passing the firing check with a draining weapon
   leads to a drain attempt rather than an ordinary shot: no projectile, shot damage or
   warhead effect, no rearm-countdown start, no burst advance, no ordinary per-shot
   ammunition use and no ordinary-shot draws. An admitted initial aircraft firing pass
   still makes its run's round pending, even when its attempt is a drain attempt
   (S-0608 claim 16; S-0501 claim 15). S-0608 claims 1, 2 and 5 own the link attempt, target clearing and
   link-formation effects. Clearing the target still resets the burst counter under claim 4.
   Link effects, including animation creation and release of controlled units, retain their
   own possible random draws; their randomness is not an ordinary-shot draw.

   A takeover weapon that passes S-0601 claim 3's test still faces the remaining target tests
   at rank 9, including the zero-versus test, and all later ranks (S-0601 claim 4).
   Passing takeover legality alone therefore does not give *can fire*.

   The aircraft cargo refusal at rank 11 follows every general refusal (ranks 4 to 10)
   and precedes aircraft aiming at rank 15. An empty marked carrier therefore answers
   *rearming* while its countdown runs, *out of range* when the ordinary range test fails,
   and otherwise *out of ammunition* even with rounds remaining and its aim wrong.
   A fresh aircraft placed empty has no passenger-carrier mark and no cargo refusal
   (S-0501 claim 1). This is the cargo-availability rule, not a defect or a quirk:
   passenger shots release cargo (S-0501 claim 20); unloading does not turn a marked
   carrier into an ordinary armed aircraft. No switch applies.

   So a firer whose countdown runs gets *rearming* even when its target is out of range or it is
   out of ammunition. The rank of *busy* for a firer that is not a vehicle is not stated.

   Where reasons of two of ranks 1 to 4 hold at once, the answer may be that of either if
   the behaviours that respond to the answer (S-0215, S-0310) respond to the two answers
   alike; otherwise the rank table's order holds.

   Among ranks 11 to 15, S-0310 (claim 9) gives *must deploy*, *moving* and *still turning* no
   reply, so a vehicle with any of them stays as it is, while *not facing* makes it re-aim. So the
   order of those three among themselves shows in no game state, and an implementation may answer
   whichever of the three holds. The order of each of the three against *not facing* does show: a
   vehicle for which one of the three holds (a vehicle with a destination that is also facing the
   wrong way, for example) does not re-aim. *Illegal* and *impossible* of rank 12 draw replies of
   their own (S-0310, claim 11), so their place among the others shows too.

   **Exception (S-0215, claim 32).** The secondary shot of a building whose type has the SAM
   flag is made in the same processing as its primary shot without a firing check of its own, so
   the table above does not apply to it. The same exception applies to the secondary firing
   attempt of a non-SAM building with an installed upgrade when S-0215 claim 32's upgrade
   weapon gate passes: a primary attempt is followed by a secondary attempt in that processing,
   without another firing check. A rearm countdown begun by the primary does not prevent the
   secondary attempt. These are attempts, not a guarantee of two successful shots. If the
   secondary slot resolves to no weapon, that attempt does nothing: no shot, no burst-counter
   change, no rearm-countdown change, no ammunition use and no random draw. It produces no
   firing-check answer, because it asks no firing check. In particular, it leaves the primary
   shot's burst position and rearm delay intact. This requires the resolved slot to be empty,
   not merely the gate-tested upgrade's own secondary: S-0308 claim 12 owns resolution from
   other upgrades, occupants and the host. S-0215 claim 32 owns the gate and paired attempts.
   Claim 14 here specifies their priority over animation-delayed firing. An aircraft's full burst likewise has no firing check between
   its shots (S-0501 claim 12).
   - Backing: `measurable` (scenarios F7, F8 and F15; S-0405 scenario W8 part j for
     deactivated firing refusal; scenario F15a and S-0514 scenario E11 part a for inactivity during erasure; S-0506 scenarios R2 and R6 for spawner
     handoff and lockouts; S-0607 scenario L9 parts c, d and f for the gripping lifter's conditional
     rearming and any-weapon refusal against its gripped unit; scenario F7 part f for
     their precedence against target-layer and projectile refusals; S-0614 scenarios P2 and P9 for parasite restrictions; S-0504 scenario
     N6 for sinking restrictions; S-0511 scenario B3, turn-order variant, for falling
     refusal and the same-turn surviving-landing boundary; S-0517 scenario P4 for transition and magnetic-beam refusals
     and P5 for the deploy-to-fire cell test; S-0501 scenarios AC7 to AC9 for aircraft bursts, and
     scenario F15 part f and S-0516 scenario CS9 part c for hidden targets; S-0519 scenario T3 for bomb-planting and disarm legality,
     S-0601 scenarios MC2, MC3 and MC11 for takeover legality; S-0610 scenario P4 for the
     berserk-friendly refusal; scenario F7 part e and S-0501 scenario AC11 for the
     empty passenger-carrier refusal and its precedence; S-0608 scenarios D1 and D2 for
     drain attempts instead of shots, D3 for the rank-4 drain refusals, D9 part a for
     stranded buildings, D11 for drain refusal precedence and D12 for aircraft pending
     rounds despite a drain attempt; also
     S-0215 scenario B36 and its extensions for the upgrade gate and ordered paired attempts
     despite the primary's rearm countdown; scenario F17 for an empty resolved secondary
     slot and animation-delayed upgrade buildings; scenario F21 for a building's pending
     delayed shot against the water and land rows, parts a and b, and against the render-layer row,
     part e).
     The berserk-weapon refusals of S-0610 claim 3 are `unmeasured` here for that claim's
     reason: no proposed setup isolates the single-target firing check from the area effect.
     Exact precedence of the sinking target refusal is `unmeasured` for the reason stated by
     S-0504 claim 5. `unmeasured` for the order among ranks 1 to 4 when two of them hold at once:
     no scenario here establishes both a reachable overlap and a differing observable reply
     to its competing answers. This includes a spawner vehicle also playing death frames or
     melee-linked. The order remains a prediction, not an observed distinction.
     Deactivation's specified precedence against other rank-4 reasons is `unmeasured`:
     W8 part j observes inactivity, not overlapping refusal labels, and no normal-input
     setup isolating those overlaps is established here. The deactivation answer label itself
     is also `unmeasured` here: W8 part j does not record it. The slave-bound exception to
     own-grip precedence is `unmeasured`: F7 part f isolates the projectile comparison,
     not a slave-bound lifter. Its confidence is medium. Erasure's firing-check label and
     rank remain open, not rules asserted by this claim; E11 part a establishes only
     ordinary firing inactivity.
     The materialising firer's precedence within rank 4, the infantry keep-acting *illegal*
     and its place, and the order and drop for an infantry firer that heals and holds a
     seized unit are `unmeasured`: no scenario here sets up those overlaps.
     The warp-pending *illegal* and its rank are `unmeasured`, since no scenario records
     firing-check answers; the drops that the answer causes in infantry and vehicle holders are
     `measurable` (S-0513 scenario SW12 parts i and j). The shots not made are `unmeasured`: SW12
     part j does not establish an otherwise eligible firing opportunity before the first check
     tick, SW12 part k (a temporal weapon's making no shot at such a target) is a recorded run
     whose outcome fits more than one rule, and no part sets up building or aircraft holders.
     Rank 16 is `unmeasured`: no isolated locomotor refusal and its answer have yet been
     identified, so no scenario here can distinguish it from another refusal.
     A building's pending delayed shot against every reason other than the water, land,
     render-layer and airborne rows of S-0306 claim 23 is `unmeasured`: F21 refuses its target
     only by those rows, and no setup here isolates the shot's place against another reason of
     rank 4 or a later rank (the drain refusals included), or against S-0405 table T1's answers.
     F12 part b drops a pending shot whose target has left range or died, but compares no
     ranks. A drain refusal during the delay would need the building's target to become drained
     by another drainer within the delay, which no input ensures. In part e both the
     render-layer row and the airborne row start to refuse the lifting unit at the same height,
     208 leptons above the surface. S-0102 claim 3 keeps a unit with jumpjet movement in the
     ground layer below that height, and S-0307 Terms makes it airborne from it. Both rows
     answer *illegal* at rank 4, so part e cannot tell which of the two refuses the unit, and it
     needs to tell neither.
     That a building whose temporal beam is attached to its target drops that target, with the beam
     still attached, while it is switched off or short of power and not charged is `measurable` for
     a target on the map (scenario F22, parts a and b). For a superweapon warp member that has
     departed it is `unmeasured`: with a clean landing the member appears in the tick after its
     departure (S-0513 claim 39), so no switch order or change of power can fall between the two,
     and no scenario here sets up a landing that is not clean. The other rows of S-0405 table T1
     against the held-temporal-target *rearming* are `unmeasured`: no scenario here drains, pulses,
     empties or sells a building that holds a beam, and S-0215 claim 33's per-tick drop of an
     opportunity target is that spec's.
   - Confidence: medium (a building's pending delayed shot ahead of the water, land,
     render-layer and airborne rows of S-0306 claim 23, until F21 runs; F21 tests it through the
     target kept and dropped, and S-0306 claim 23 leaves this rank unstated), medium (its place against every other reason,
     ahead of the rest of rank 4 and the later ranks and behind S-0405 table T1's answers),
     medium (S-0405 table T1's answers ahead of the held-temporal-target *rearming*, and the drop
     with the beam kept that they cause, until F22 runs; medium for the rows that F22 does not set
     up and for a departed warp member),
     medium (the deactivated refusal, its unmeasured within-rank precedence,
     and ordinary firing inactivity during erasure), high
     (the two grip-refusal comparisons against projectile and target-layer refusals within rank 4,
     the empty-secondary no-op and the upgrade-building paired attempts without a second firing check,
     falling refusal and the same-turn surviving-landing boundary,
     the three drain refusals, their specified precedence (except against a building's pending
     delayed shot, rated with that shot's place above) and no ordinary shot from a drain attempt),
     medium (its missing countdown start and per-shot ammunition use, as S-0608 claim 1,
     and aircraft pending-round composition, as S-0608 claim 16), high (rearming before ammunition and range, facing and moving after them, and
     *must deploy* before *not facing*, and the hidden-target refusal using mean weapon damage),
     medium (the full membership of *illegal* and *impossible*,
     and the early *rearming* cases, which other specs own; only the ones relevant to this spec are
     listed; the tether and death-frame reasons); low (ranks 1 to 4 among themselves; rank 3, whose
     cause (S-0214) is itself low; rank 16). The passenger-carrier refusal and its
     precedence have medium confidence. The materialising firer within rank 4: medium, and low
     for its place after a vehicle's answers of ranks 1 to 3 (it rests on the order of ranks 1 to
     4 among themselves), low for the infantry keep-acting *illegal* and its place (reached only
     through S-0214's link, whose creation that spec rates low), low for the order given for an
     infantry firer that heals and holds a seized unit and its drop at the check, and low for a
     materialising one's drop through its firing step's reply, as S-0310 claim 11 rates that the
     infantry answer shows through its reply. The warp-pending *illegal*, its rank and the shots
     it prevents: medium.

### Ammunition and reloading

10. **Ammunition.** A new vehicle, infantry unit, building or aircraft starts with ammunition
    *a*₀ = *A* when its type's initial-ammunition field *J* is −1, and *a*₀ = *J* otherwise
    (Formula F6). Thus an absent initial field fills a finite magazine, including 0 rounds
    for a maximum of 0; it does not grant unlimited ammunition. An explicit initial value is
    used without clamping to the maximum, even above it (`quirks.initial_ammo_above_capacity`,
    default original). An explicit 0 starts empty, including when the maximum is −1.
    The same rule applies to objects placed by the map and objects created for production.
    There is no separate map ammunition value. This is the starting count, before any turns
    that can reload a vehicle (claim 11) or service an aircraft. Each shot
    that a vehicle, infantry or building makes uses one round when *a* > 0; *a* = −1 is never
    changed. With *a* = 0 the firing check answers *out of ammunition*. An attempt that asks no
    firing check (claim 9's secondary attempt of a SAM building or of a passing upgrade pair) is
    not refused at *a* = 0: it is made as at any other count, uses no round, and a shot it makes
    gets claim 2's normal delay, since the building holds no spare round. A disc laser shot uses no
    round (S-0615 claim 12; and it neither advances a garrison's muzzle index, S-0313,
    nor restarts a vehicle's reload, claim 11). A drain attempt likewise uses no ordinary
    per-shot round (claim 9; S-0608 claim 1). Aircraft instead make one round pending per
    attack run whose initial firing pass is admitted, not per shot, including a pass that
    makes a drain attempt rather than a shot (S-0501 claim 15; S-0608 claim 16). A successful drop attempt additionally spends one round immediately,
    whereas a refused drop has no net immediate ammunition change (S-0522 claims 9 and 11).
    Run-level pending spending still applies even when every drop attempt in the firing pass
    is refused; it is additional to successful drops' immediate spending.
    Servicing on pads belongs to S-0501 claims 26 to 28. Only their starting count is owned here. The 1-tick rule depends on
    a building's ammunition before the shot: a building that holds more than 1 round before a shot
    has a delay of exactly 1 tick after that shot, with no draw, whatever its weapon's burst count
    or kind (claim 2's order; `quirks.building_ammo_fast_rearm`). The shot made with its last round gets the normal delay of
    claim 2, and a building with unlimited ammunition (*a* = −1) is not affected by the rule.

    **A building's refill.** In every tick in which a building is in play, after its own
    firing in that tick (after both attempts of a paired firing, claim 9), its ammunition is set
    to its type's maximum *A* if it is 0, unless the type has the hospital flag or the armory
    flag. The type's reload fields play no part, and no countdown of any kind is involved. So a
    building eligible for this refill, with a nonzero maximum, is seen at 0 rounds after its first
    tick only between a shot and the end of that shot's tick, and never stays *out of ammunition*:
    - a building with 3 rounds and no other firing gate fires in ticks *t*, *t* + 1 and
      *t* + 2, holds 3 rounds again at the end of tick *t* + 2, and fires its next group of three
      when the delay that claim 2 gives the third shot has run, keeping its target throughout
    - a building that starts with 0 rounds and *A* > 0 is filled at the end of its first tick, so
      from its first attempt after that it fires as one that started full
    - an initial count above *A* (above) is spent first, and every group
      after it holds *A* rounds
    - with *A* = −1 and an initial count of 0, the refill gives −1, so the building then has
      unlimited ammunition
    - with *A* = 0 the refill gives 0
    - Backing: `measured` (M-0004: a building defence with 3 and with 4 rounds and a burst weapon,
      its 1-tick delays as countdowns, its group size, its refill within the tick of the last
      round's shot, and its never stopping for lack of ammunition, with a long reload field
      set). Also `measurable` (scenario F14's no-draw part for the 1-tick delay taking no
      draw, scenario F9's paired-refill part, variants a and b, for the refill's place after
      both attempts and variant b for the unchecked secondary attempt at 0 rounds, the
      building part of scenario F4, S-0522 scenario P7 for drop ammunition, S-0608
      scenario D12 for aircraft drain-attempt pending rounds, and the refill-exception part of
      scenario F9 for the hospital and armory flags)
    - Confidence: high (starting count, no clamp, the same rule for map and production, rounds used,
      out of ammunition, the 1-tick rule before the full and in-burst delays and the refill of a
      building defence with a burst weapon that spent its last round, the kind M-0004 ran); medium
      (the refill of other kinds of building, which no record has run; the 1-tick rule before the
      bare rate, until the
      building part of scenario F4 runs; the refill's exceptions for the hospital and armory flags,
      a building that starts with 0 rounds, the refill with *A* = −1, which no record has run, and
      the refill's place after the second attempt of a paired firing, until scenario F9's
      paired-refill part runs; the unchecked secondary attempt of an upgrade pair at 0 rounds,
      until variant b of that part runs; and that the 1-tick delay takes no draw, which M-0004
      does not separate from the background draws). The same attempt by a SAM building is
      `unmeasured` (no scenario here gives a SAM building a finite magazine), medium, since it
      asks no firing check as the upgrade pair's does (claim 9)

11. **Reloading (vehicles only).** A vehicle whose type has *A* ≥ 0 regains ammunition over time.
    The ordinary reload opportunities below are subject to the warp exclusion described after the table.

    | Vehicle state or event | Reload behaviour |
    |---|---|
    | A projectile-launching shot leaves *a* < *A*; the type lacks the manual-reload flag and *A* ≠ 0 | The reload countdown restarts at the reload delay of Formula F3 for the current *a*. Continuous firing therefore keeps postponing the reload. |
    | The reload countdown has never been started and *a* < *A* | The countdown counts as over. The vehicle regains one round in the first of its turns in which *a* < *A*, whenever that turn comes; the countdown then starts as described in the next row. This includes a manual-reload vehicle until its first regained round. |
    | The reload countdown is over and *a* < *A* | *a* grows by 1 and the countdown restarts at the reload delay of Formula F3 for the new *a*. One round comes back per reload delay. |
    | The vehicle lacks the mobile-fire flag and its locomotor is moving it | Its reload countdown does not advance. |

    The reload delay is the type's empty-reload field when *a* = 0 and that field is not −1; otherwise
    it is the reload field plus the reload-increment field times *q*², where *q* = 1
    when the type's pip-wrap field is 0, else *q* is *a* divided by the pip-wrap field in integer
    division toward zero (Formula F3). Infantry never regain ammunition. A building regains it
    only by claim 10's refill, all at once and with no delay. The reload fields have no effect on
    either.
    A vehicle regains at most one round in each of its turns, so a reload delay of 0 or less still
    gives one round per tick.

    **Warp exclusion (engine design).** Under S-0513 claim 51, from the first pending warp
    update through arrival effects inclusive, the vehicle performs no ordinary reload work:
    no round is granted and no countdown is restarted by that work. Elapsed time on an
    existing reload countdown still passes; this is not the moving-vehicle freeze in the
    table. During arrival hold, ordinary reload eligibility returns after the hold-end test,
    whether the hold continues or expires. There are no catch-up grants: an expired countdown
    permits at most one round in that eligible turn, followed by the ordinary restart.
    Immediate damage and explicit event-driven effects remain eligible under their own rules,
    including claim 17. This is our engine's continuation rule, not an established original
    per-stage rule; S-0513 claim 52 leaves the original's continuation unresolved.
    - Backing: `measurable` (scenario F10; its warp part tests our engine only).
      Original per-stage warp continuation is `unmeasured` under S-0513 claim 52's
      incomplete continuation evidence (scenario SW23 part g there).
    - Confidence: high (warp design contract, formula, restart on each shot, one round per completion); medium (the
      never-started countdown that counts as over, the frozen countdown while moving, and the restart
      by damage of claim 17)

12. **Cloaked firers.** A weapon with the fire-only-visible flag (set unless the data clears it)
    gets the answer *cloaked* while its firer is in any cloaking state other than fully visible
    (cloaking, cloaked or uncloaking); an aircraft gets it only while fully cloaked. The firer's own behaviour then uncloaks it (the cloaking specs; S-0310, claim 9).
    - Backing: `measurable` (scenario F7, part c)
    - Confidence: medium

### Shots that wait

13. **Firing frames (sprite vehicles).** A vehicle whose type has *F* > 0 firing frames has a
    firing-frame counter that changes as follows at an ordinary eligible opportunity
    (subject to the warp exclusion below):

    | Counter | Event | Result |
    |---|---|---|
    | idle or running | the vehicle makes a shot that is not a sync shot (below) | running at 2*F* − 1 |
    | running at a value above 0 | a turn of the vehicle, not travelling through a tunnel | running at one less |
    | running at 0 | a turn of the vehicle, not travelling through a tunnel | idle |
    | running | a turn in which the vehicle travels through a tunnel | unchanged |

    The type may give a firing sync frame for each of the values 0 and 1 of *k* mod *B*, where *k*
    is the burst counter (the data value −1 gives none, Import mapping). The sync frame *s* that
    applies follows the current value of *k* mod *B*:

    | *k* mod *B* | Sync frame *s* |
    |---|---|
    | 0 | the type's sync frame for value 0, if it gives one |
    | 1 | the type's sync frame for value 1, if it gives one |
    | 2 or more | none |

    *k* is below *B* except after a change of weapon has carried a count over (claim 4), so for
    a one-shot weapon the value is 0 and for a burst of 2 it is the burst position already made. Where a sync frame applies, an attempt
    with the primary weapon while the counter runs and has not yet reached *s* gets *rearming*.
    In the tick in which it equals *s* the rearm countdown is ignored and the shot can be made.
    That shot is a *sync shot*. Once the counter has passed *s*, whether this gate still
    refuses a shot is open (Open questions). With no sync frame given, firing frames change
    only the picture.

    **Warp exclusion (engine design).** S-0513 claim 51 suppresses firing-frame advancement
    from the first pending warp update through arrival effects inclusive. The counter retains
    its value; elapsed time on the separate rearm countdown still passes. During arrival hold,
    ordinary advancement eligibility returns after the hold-end test, whether the hold
    continues or expires, with no replay of the skipped advancements. This does not grant a
    shot while materialising or bypass any other firing refusal (claim 9). The rule is our
    engine's design, not established original per-stage continuation (S-0513 claim 52).
    - Backing: `measurable` (scenario F11, test data only; its tunnel part for the tunnel rows
      and its warp part for our engine's contract). Original per-stage warp continuation is
      `unmeasured` under S-0513 claim 52's incomplete continuation evidence
      (scenario SW23 part g there). Whether the first decrement occurs in the shot's turn
      or the next remains open, rather than a measured rule of this table.
    - Confidence: high (warp design contract); medium (other firing-frame rules)

14. **Animation-delayed building shots.** In the ordinary non-SAM attack path, outside
    S-0215 claim 32's active upgrade weapon gate, a building whose type has the
    animation-delayed-fire flag does not fire when its attack behaviour gets *can fire*.
    Instead it
    starts its firing animation and its delayed-fire countdown, set by the type's delayed-fire field
    *d* (ticks). The delayed shot is pending until the building's turn max(*d* − 1, 0) ticks after
    the turn that started it. (A building takes a turn in every tick. The countdown does not depend on how often its attack behaviour is processed,
    which is every other tick while it turns towards its target or rearms, S-0215 claim 32.) In that turn the building makes its scheduled firing check again, with range; that check is not refused merely because this same delayed shot is pending. On *can fire* it fires its weapon at its current target; on any other answer the shot is dropped. Either way the shot is no longer pending. Other firing checks made while the shot is pending answer *rearming*, unless one of the building's own answers of S-0405 table T1 holds (claim 9). The rearm countdown starts only with the shot itself.

    **A target that becomes refused while the shot is pending (Q-1770).** The check and the
    shot at the end of the delay use the building's target at that moment and the weapon slot
    chosen when the delay started; the slot is not chosen again, even where S-0308 would now
    choose the other one (land-targeting mode 2, for example). This is the one exception to
    S-0308 claim 14's fresh choice of the weapon for every firing check (Changes for other
    specs). A shot that this check refuses is
    not made: no projectile, rearm countdown, ammunition use or draw, and the check itself
    leaves the target as it is. An attack order that gives the building another target while the
    shot is pending changes its target but starts no new delay, so the shot at the end of the
    delay is made at the new target if the check admits it. While the shot is pending, the attack behaviour gets *rearming*
    (claim 9, rank 4), so the building keeps its target and keeps turning towards it, also when
    the target has meanwhile become one that a row of S-0306 claim 23 refuses. Examples are an
    enemy vehicle that drives from water or beach onto clear land or a bridge for a type with
    land-targeting mode 1, or from clear land onto water or beach for a type with
    naval-targeting mode 6, or a jumping infantry unit that lifts off and climbs to 208 leptons
    (two height levels) above the surface or higher. From that height it is both outside the
    ground layer (S-0102 claim 3) and airborne (S-0307 Terms), so the render-layer row and the
    airborne row both refuse it for a projectile that is not anti-air. Below that height neither
    row refuses it, so for a lifting unit still below 208 leptons when the delay ends those rows
    do not stop the shot, and it is made unless another answer of the check refuses it. In the tick that ends the delay, an attack processing that falls in
    it while the shot is already pending comes before the end-of-delay check and still gets *rearming*. The end-of-delay check sees
    the target as it stands at the building's turn (update order, S-0203). If a row refuses the
    target then, the shot is dropped, and the attack behaviour's first processing after that
    tick gets *illegal* and drops the target, queuing *guard* (S-0215 claim 32), unless the
    target has become legal again by then. A shot that the check admits is an ordinary shot of
    that weapon at that target, under the same rules as an undelayed shot (S-0306; claim 1 for
    its rearm countdown, claim 9 for a draining weapon, claim 15 for a void shot).

    **Upgrade gate priority (Q-0798).** For a non-SAM building to which S-0215 claim 32's
    upgrade weapon gate applies, the animation-delayed-fire flag does not defer a passing
    gate's pair: the primary and secondary attempts occur in that processing, with no
    delayed shot started by it. A failed gate with the original quirk setting starts
    neither an attempt nor a delayed shot. With that gate's quirk off, a failed gate uses
    the ordinary selected-weapon path, including this claim's animation delay when enabled.
    A passing gate still makes the immediate pair. An empty resolved secondary slot has
    claim 9's no-op outcome, including when the host has the animation-delayed-fire flag.
    The gate's applicability, including the upgrade-position test, belongs to S-0215
    claim 32; weapon-provider order belongs to S-0308 claim 12 and upgrade placement to
    S-0406 claim 12. A gate bypass leaves the ordinary animation-delay rule in force.

    **Prism-type buildings.** S-0518 claims 1–10 own their support recruitment and supported
    shots. In the ordinary attack path covered there (no SAM flag or installed upgrades), a
    prism-type building recruits supporters before starting its own delayed shot. That shot
    always uses this countdown, regardless of the animation-delayed-fire flag, and both its
    final firing check and its shot use weapon slot 0 (S-0518 claims 4 and 7).
    - Backing, by part:
      - The countdown arithmetic: `measurable` (scenarios F12 and F17).
      - Upgrade gate priority: `measurable` (scenario F17). The prism exceptions: `measurable`
        (S-0518 scenarios PS1 and PS5).
      - The target kept while the shot is pending, and the shot dropped at the end of the delay
        when a row refuses the target then: `measurable` (scenario F21 parts a, b and e, with
        part c as control).
      - The slot chosen when the delay started: `measurable` (scenario F21 part d).
      - An attack processing in the tick that ends the delay coming before the end-of-delay
        check: `measurable` (scenario F21). With F21's delay field of 30 ticks, the processing
        after the one that starts the shot comes 1 tick later and each later one 2 ticks after
        the last (S-0215 claim 32), so one falls in the tick that ends the delay; the reverse
        order would drop the target in that tick.
      - The check and the shot using the building's target at the end of the delay rather than
        the one it had at the start, and a new attack order during the delay starting no new
        delay: `measurable` (scenario F21 part f).
    - Confidence, by part:
      - The countdown arithmetic: medium.
      - Upgrade gate priority and the prism exceptions: high.
      - The target kept, and the shot dropped at the end of the delay: medium until F21 runs.
      - The slot chosen at the start: medium until F21 part d runs, since it departs from
        S-0308 claim 14's high-rated fresh choice.
      - The order within the tick that ends the delay: medium until F21 runs.
      - The target at the end of the delay rather than at the start, and no new delay at an
        order: medium until F21 part f runs.

15. **Failed launches.** If the projectile of a shot cannot be put on the map, the shot is void:
    the rearm countdown, the burst counter, the ammunition and the garrison muzzle position (S-0313)
    stay as they were. A carrying aircraft's drop attempt is not a void shot merely because
    it launches no projectile: successful release and its ammunition spending remain, and its
    rearm countdown expires; refusal retains the countdown (S-0522 claim 11). Neither outcome
    produces ordinary-shot effects: no spray draw, weapon report or shot animation (claim 23).
    - Backing: `measured` (M-0004 records attempts of a spray-attack vehicle whose countdown stayed
      0, with no attributed delay draw, and gives no burst position for them; their classification
      as void shots is an inference); `measurable` (S-0306, scenario P5 run d: an arcing shot for
      which the launch
      pitch has no solution is removed before it enters play, S-0306 claim 6 a, observed here
      by scenario F14, void-shot part, also for the absence of any delay draw, which M-0004
      does not separate from the background draws; S-0522 scenario P7 for the drop
      exception); `unmeasured` for the burst counter, the ammunition and the muzzle position that
      a void shot leaves: F14's void-shot fixture sets no finite ammunition and no burst position
      that a shot would change; `unmeasured` for a void shot made while a countdown runs (claim 13's
      sync tick, an
      aircraft's same-tick burst or an unchecked building secondary attempt): no scenario of
      this spec makes a launch
      fail in one of those attempts, and a countdown of 0 cannot show whether a running one is
      kept
    - Confidence: medium (a void shot at a countdown of 0 leaves it at 0, until scenario F14's
      void-shot part runs; that it takes
      no delay draw, until scenario F14's void-shot part runs; a running countdown kept by a
      void shot, the burst counter, the ammunition and the muzzle position of a void shot, and
      which launches fail); high (the drop exception)

### Related state

16. **Charge turrets.** While its rearm countdown runs, a charge-turret vehicle shows turret
    number *c* = trunc(*N* × *r* / *D*) clamped to 0 … *N* − 1, where *r* is the rearm time remaining and
    *D* the delay last chosen (after any berserk halving); with *D* ≤ 0, or once the countdown is
    over, *c* = 0. This is recomputed in each of its turns. It applies only to a charge-turret type
    with a turret count *N* above 0 that is not a gattling type. The turret number is for drawing
    only: it does not select the weapon. A charge-turret type's weapon is the slot of its weapon
    number, which stays at 0 because the type ignores every request to change it (S-0308, claim 5),
    so every shot of such a vehicle uses its slot-0 weapon, whichever turret image is shown (the one
    shot that can fall inside the countdown, a sync shot of claim 13, is no exception).
    - Backing: `measurable` (scenario F13)
    - Confidence: medium

17. **Damage can empty a magazine.** A type with the imported damage-reduces-readiness flag loses
    ammunition when damaged and restarts its reload countdown (claim 11) at once. The amount
    belongs to S-0302.
    - Backing: `measurable` (scenario F10, part d)
    - Confidence: low

18. **Draws taken by one shot.** These are ordinary-shot draws, not the possible
    link-effect draws of a drain attempt (claim 9; S-0608 claims 1 and 5), nor the
    standing-position and parachute draws of a drop attempt (S-0522 claims 9, 11 and 13).
    A drop attempt takes no ordinary-shot draws, whether successful or refused. A shot takes, from the scenario random stream and in this order:
    the spray pick of S-0313 (only for the first shot of a burst of a spray-attack type), the
    projectile's own draws (S-0306), then the one delay draw (R(0, 2) of claim 2 or R(3, 5) of claim 4) if
    the delay kind chosen by claim 2's order takes one, and after it the draws of the effects the shot creates last: its
    shot animation (claim 20), a sonic wave or a laser (S-0202, claim 10, and the specs of those
    effects). Creating an effect weapon's particle system is not a draw: its draws come in the
    system's own turns (S-0616 claims 10 to 18). A refused attempt takes no draw at all. A
    delay without a draw (a burst-delay field of claim 4, the bare rate of claim 5, the 1-tick
    delay of claim 10) takes no delay draw. A void shot as in claim 15 takes the projectile's
    draws but no delay draw.
    - Backing: `measured` (M-0004: one full-delay draw after an inaccurate projectile's two
      draws at each shot that ends its burst, one in-burst draw per in-burst shot of a building
      defence, no further draw in the tick of the shot whose draws
      were separated by its control run, and the spray pick first in an attempt read as a void shot,
      followed by
      the projectile's draws and no attributed delay draw)
      - `measurable` (scenario F3 for one in-burst draw per in-burst shot of a vehicle and of an
        infantry unit; scenario F14, with the random-state recorder, its no-draw part for a
        burst-delay field, the bare rate and the 1-tick delay taking no delay draw, and its
        void-shot part for a void shot taking none, which M-0004 does not separate from the
        background draws; S-0608 scenario D1 part a for drain attempts and link effects, S-0522
        scenario P7 for drop attempts)
      - `unmeasured` for a refused attempt taking no draw: in the ticks of a refusal the firer's
        behaviour processing can take its own draw over the same range (S-0215 claim 23), and
        neither M-0004's comparison nor F14's refusal part tells a firing-check draw apart from it
    - Confidence: high (one delay draw per shot of the full delay, and per in-burst shot of a
      building defence); medium (one in-burst draw per in-burst shot of a vehicle or an infantry
      unit, until scenario F3 runs; none
      for a refused attempt, `unmeasured`; no delay draw for a burst-delay field, the bare rate and
      the 1-tick delay, until F14's no-draw part runs, and none for a void shot, until its void-shot
      part runs; the delay draw's place between the projectile's and the animation's draws)

19. **Charge arrival queries a delay without firing.** At the sabotage arrival of S-0519
    claim 13, whether the charge is planted or refused, the infantry walker's rearm countdown
    is replaced by the arrival delay for its second weapon. This is not a shot: it uses the
    current burst counter *k*, not *k* + 1, and neither advances nor wraps it. It uses no
    ammunition and creates no weapon effect. The first applicable row determines the delay:

    | Condition | Arrival delay, integer ticks | Scenario-stream delay draws |
    |---|---|---|
    | no second weapon | 1 | none |
    | second weapon is sonic, or is a spark-particle, fire-particle or railgun weapon with an effect of its kind already alive on the walker | its rate-of-fire field *R*, unchanged | none |
    | *k* < the second weapon's burst count *B* | R(3, 5) | one ranged draw |
    | otherwise | *D*₅ of Formula F1, with the second weapon's rate-of-fire field and the applicable factors of claims 3, 6 and 7 | one R(0, 2) |

    Thus a fresh walker with *k* = 0 takes the short draw even for a single-shot second
    weapon, unless an earlier row applies. Unlike a successful particle shot (claims 2 and 5),
    arrival does not create an effect that would make that weapon eligible for the bare rate.
    No vehicle burst-delay field applies to this infantry walker. Berserk halving (claim 8)
    applies to none of the rows. The full-delay row retains Formula F1's floating-point
    domains, rounding and truncations, but stops at *D*₅. Any delay draw precedes the subsequent
    arrival scatter's draws. Separate target clearing still resets *k* under claim 4.
    S-0519 claim 13 owns the arrival conditions and remaining effects.
    - Backing: `measurable` (scenario F18; S-0519 scenario T13)
    - Confidence: high

### Shot animation

20. **Which shot animation a shot makes.** Every shot made in ordinary firing makes at most one
    shot animation, chosen as follows. No random draw is taken for the choice.
    - (a) **From the animation list**, of length *n*:
      - *n* = 0: no type is chosen.
      - *n* = 8: entry *e* = (*d* + 1) mod 8, where *d* is the 8-direction of the firer's shot
        facing (claim 21; Formula F7).
      - any other *n* ≥ 1, including 1 and 16: entry 0, whatever the facing.
    - (b) **Garrisoned building.** If the firer is a garrisoned building (Terms), the weapon's
      occupant animation replaces the choice of (a). A weapon without an occupant animation
      then makes none, even if its animation list is not empty.
    - (c) **Open-topped fallback.** If (a) and (b) have chosen no type, the firer is a
      passenger firing out (Terms) and the weapon names an open-topped animation, that type is
      chosen. A passenger whose weapon has a non-empty list therefore plays the list's entry,
      chosen by the passenger's own shot facing, and never its open-topped animation.

    So with an eight-entry list, entry 0 is drawn for a firer facing straight up the screen
    (map north-west, *d* = 7), and the entries then follow the screen clockwise:

    | Entry *e* | 8-direction *d* | Way the firer faces |
    |---|---|---|
    | 0 | 7 | map north-west (straight up the screen) |
    | 1 | 0 | map north (upper right) |
    | 2 | 1 | map north-east (right) |
    | 3 | 2 | map east (lower right) |
    | 4 | 3 | map south-east (straight down) |
    | 5 | 4 | map south (lower left) |
    | 6 | 5 | map south-west (left) |
    | 7 | 6 | map west (upper left) |

    Every shot of a burst, a sync shot (claim 13) and an animation-delayed building's shot (claim
    14) makes its own choice at the moment it is fired. A gattling type's shot uses the weapon of its
    current stage (S-0609), so its list and therefore its shot animation change with the stage.
    - Backing: `measurable` (scenario F19 parts a, b, c, e, h and j)
    - Confidence: high (the list rule, the one-step shift of entries and the absence of a draw); medium (the
      garrison replacement and the open-topped fallback)

21. **Shot facing.** The shot facing is read at the moment of the shot:

    | Firer | Shot facing |
    |---|---|
    | vehicle whose own type has the turret flag | its current turret facing |
    | any other vehicle | its current body facing |
    | infantry, including a passenger firing out | its current body facing |
    | aircraft | its current turret facing, the facing that S-0310 claim 5 compares |
    | building without a target | its turret facing (S-0310 claim 4) rounded to the 256-step facing |
    | building with a target, turreted with a sprite turret (S-0310 Terms and claim 4) | its turret facing rounded to the 32-step facing |
    | any other building with a target | the direction of the target from the building's aiming point, as S-0310 claim 4 measures it |

    A rounded building facing is rounded again to 8 steps (Formula F7). The two roundings can
    give another entry than one rounding would: with the 32-step rounding, turret facings 3072
    to 4095 give *d* = 1, and with the 256-step rounding, facings 3968 to 4095 do, where a
    single rounding gives *d* = 0. For a garrisoned building the facing does not matter, because
    claim 20 (b) replaces the list's choice.
    - Backing: `measurable` (scenario F19 parts a, b, f and i) for every row except a building
      without a target, which is `unmeasured`: no normal-input situation is known in which a
      building fires while it holds no target.
    - Confidence: high (vehicles and infantry); medium (aircraft and the three building rows)

22. **Where it is and what it follows.** The shot animation is created in the firer's turn, at
    the shot (S-0109 claim 29), at the shot's launch point (Terms). This includes the muzzle of
    the shot for a garrisoned building (S-0313 claim 6) and the transport's muzzle for a
    passenger firing out (S-0313 claim 5). It is created with no initial loop delay, repeat
    factor 1 and no reverse request; the type's own reverse flag still applies (S-0109 claim 8).
    - **Firer that is not a building.** The animation is attached to the firer (S-0109 claim 22).
      It follows the firer as it moves, including a passenger moving with its transport. It is in
      the ground layer (S-0102 claim 3), even for an aircraft in flight. It disappears when the
      firer is removed from the game.
    - **Building.** The animation is not attached. It stays where it was created, in its type's
      layer (S-0102 claim 3). Its creator replaces its depth adjustment (S-0102 claim 13) with
      *z*ₐ of Formula F8.
    - **Palette.** Its creator gives it no palette of its object. It is not an object's own
      animation drawn with its object, so it takes row 4 or 5 of S-0103 claim 8 by its own art
      flags, also when attached.
    - Backing: `measurable` (scenario F19 parts d, e, g and h). The depth adjustment is
      `unmeasured`: its only effect is which of two overlapping pictures is drawn in front, for a
      short-lived animation, and no reliable observation of that is proposed.
    - Confidence: high (place, attachment and its absence for buildings); medium (a passenger's
      animation following its transport, the depth adjustment and the palette rows)

23. **What makes no shot animation, and what is drawn instead.** None of these makes a shot
    animation: an attempt that the firing check refuses; any shot made void by claim 15; a drain attempt
    (claim 9); a drop attempt (S-0522 claim 11); a spawner weapon's shot (Terms); the firing of a
    disc laser weapon (S-0615 claim 12); the firing of a suicide weapon (glossary; S-0519
    claim 11); and a shot for which claim 20 chooses no type. A shot without a chosen type gets
    no default or placeholder picture at its launch point. The other things drawn when a weapon
    fires belong to their own specs: its sonic wave, laser, electric bolt or particle emitter
    (claim 5; S-0616 and the combat specs of those effects), and a building's turret and barrel
    recoil (S-0105 claim 27). One more is local: the target line of claim 24.
    - Backing: `measurable` (scenario F19 parts e, h and j; claim 15's backing for shots made void; S-0522
      scenario P7 for drop attempts)
    - Confidence: high

### Target line

24. **The target line (Q-1580).** A vehicle whose type has the imported target-laser flag shows
    a line from itself to its target for a short time after it fires. It is presentation only: it
    is started only for the local player who owns the vehicle at the triggering shot, and it
    changes no game state and takes no random draw. Later ownership changes follow part b.
    - (a) **What starts it.** The line is started by every shot that reaches the choice of a shot
      animation (claim 20), whether or not that choice finds a type: every shot of a burst, of
      either weapon, a sync shot (claim 13) included. It is not started by any of the other
      exclusions of claim 23: an attempt that the firing check refuses, a shot made void by
      claim 15, a drain attempt, a drop attempt, a spawner weapon's shot, the firing of a disc
      laser weapon or of a suicide weapon. The owner test is made at the shot: the firer's house
      must be the local player's house then. The type flag is read for any firer, but only a
      vehicle ever draws the line, so the flag on an infantry, aircraft or building type shows
      nothing.
    - (b) **How long.** Each vehicle has one line timer. A starting shot in the tick with tick
      number *t*₀ sets it. Subject to part c's visibility and target conditions, the line is
      drawn in every drawn frame whose current tick number *t* satisfies 0 ≤ *t* − *t*₀ < 15,
      that is, for 15 ticks. A new starting shot within those
      ticks restarts the 15 ticks from its own tick; there is never more than one line per
      vehicle. A new vehicle shows no line before its first starting shot. A change of owner
      after the shot does not end the line.
    - (c) **When it is drawn.** It is drawn in the vehicle's own drawing step, after everything
      else that step draws. When that step does not run (the vehicle is outside the drawn part
      of the view, hidden from the viewer, or not on the map), there is no line, even when its
      target is on screen. It is drawn only while the vehicle has a target at the moment of
      drawing; without one, nothing is drawn, though the timer keeps running.
    - (d) **Start point.** The vehicle's turret pivot (S-0313, Terms), recomputed in every drawn
      frame from the vehicle's current position, body transform and turret facing, height
      included. The start therefore follows the vehicle and its turret during the 15 ticks.
    - (e) **End point.** The aim point of S-0306 claim 3 for the vehicle's *current* target,
      recomputed in every drawn frame: the target's reference point, led ahead along its body
      facing when it is a moving vehicle, by the lead of that claim with the current distance
      and speed. It is not the point the shot was aimed at. So the end follows a moving target.
      If the vehicle changes target within the 15 ticks, without firing again, the line goes to
      the new target until the timer runs out. If the target is cleared (for example because it
      was destroyed or removed from the map), the line disappears while there is no target.
      Restoring a target before expiry shows the line for only the timer's remaining ticks.
      Whether the target can be seen is not tested.
    - (f) **Projection and clipping.** Both points are projected to screen pixels as every
      object's position is (S-0101 claims 1 to 5, including height lift and integer rounding).
      This includes that spec's fixed drawing-height rule: the original's shared projection
      can visibly misalign objects with terrain. The line and the two end marks are clipped
      to the tactical view.
    - (g) **Look.** The colour is palette index 8 of the general palette (S-0306, Terms; the same
      file as S-0111's overlay palette), in its neutral shade row 26 of 53, so it is never
      lit, tinted or house-coloured, and no rules key sets it. The line is one pixel wide
      and dotted: walking the line one pixel per step along its longer screen axis, every
      second pixel is drawn, and the drawn and undrawn pixels swap from each tick to the next.
      Each end point gets a solid 3 × 3 pixel square in the same colour, whose bottom-right
      pixel is the end point (it covers the end point and the two pixels to its left and above).
      There is no fading: the line looks the same
      in each of its 15 ticks apart from the dot swap.
    - (h) **Draw-order place.** The line is drawn right after its vehicle's picture, in that
      vehicle's place in the object drawing order of S-0102, with no depth test and no depth
      write. It covers its own vehicle and what was drawn before it; objects drawn later and the
      status extras (S-0102 claim 1) cover it.
    - Backing: `measurable` (scenario F20, with its height-lift departure tested on our engine).
      The absence of a depth write is `unmeasured`: F20 part i tests visible covering order and depth-test
      independence, but does not isolate a later depth reader that distinguishes a depth write.
      Locomotor transforms outside the tested positions are `unmeasured` for the limitation of
      S-0313 claim 1: their complete reachable conditions are not established.
    - Confidence: high (the trigger, the 15-tick timer and its restart, the vehicle-only and
      owner-only rules, the current-target end point, the colour index, the one-pixel dotted line,
      the 3 × 3 squares); medium (the end's lead for a moving target, the projection of both
      points, the draw-order place, the frames in which the vehicle is not drawn); low (which
      pixels are drawn in a given tick, that is, the phase of the dots)

## Formulas

**F1. Full delay** (claims 2, 6, 7, 8). *R* is the rate-of-fire field (integer, ticks), *h* the
house factor (double), *u* = R(0, 2), *v* the veteran factor (double), *n* the occupant count
(integer, using claim 7's original retained count or engine post-removal count), *m* and *b* the
occupy and bunker factors (singles, widened exactly to doubles). RZ₅₃ is
S-0205's rounding of a double result, which is round toward zero under `quirks.chop_rounding` (the
default) and round to nearest with it off; trunc is the conversion of S-0205 claim 7.

> *D*₁ = trunc(RZ₅₃(RZ₅₃(*R* × *h*) + *u*))
> *D*₂ = trunc(RZ₅₃(*D*₁ × *v*)) if the veteran condition of claim 6 holds, else *D*₁
> *D*₃ = trunc(*D*₂ / *n*) (integer division toward zero) for a garrisoned building, else *D*₂
> *D*₄ = trunc(RZ₅₃(*D*₃ / *m*)) for a garrisoned building when *m* is above 0, else *D*₃
> *D*₅ = trunc(RZ₅₃(*D*₄ / *b*)) for a bunkered vehicle when *b* ≠ 0, else *D*₄
> *D* = trunc(*D*₅ / 2) (toward zero) for a berserk firer, unless the shot is a disc laser shot (claim 8), else *D*₅

Every value is a whole number of ticks. Invented examples (with *h* = 1, *v* = 0.5 as a double, *u* = 1):
*R* = 30 gives *D*₁ = 31 and a veteran's *D*₂ = 15; a garrison of *n* = 4 occupants with *m* = 2 then
gives *D*₃ = 3 and *D*₄ = 1.

**F2. House factor** (claim 3). With *ℓ* the difficulty row's rate-of-fire factor and *κ* the country's
(doubles): *h* = RZ₅₃(*ℓ* × *κ*) for a player house in a skirmish, *h* = *ℓ* in a campaign mission, and
*h* = 1.0 for the neutral and special houses, whatever *ℓ* and *κ* hold.

**F3. Reload delay** (claim 11). Integers, ticks. With *A* the maximum ammunition, *a* the current
ammunition, *E* the empty-reload field, *L* the reload field, *I* the reload-increment field and *W*
the pip-wrap field:

> delay = *E* when *a* = 0 and *E* ≠ −1,
> otherwise delay = *L* + *I* × *q*², with *q* = 1 if *W* = 0, else *q* = trunc(*a* / *W*).

The product and sum are 32-bit integer arithmetic (S-0205 claim 1).

**F4. In-burst delay** (claim 4; used only when claim 2's order reaches it). After the *j*-th shot of a burst (1 ≤ *j* < *B*): the type's burst
delay *j* (an integer, ticks) for a vehicle when *j* ≤ 4 and that field ≠ −1; else R(3, 5). For a
berserk firer that value is then halved and truncated toward zero, as in the last step of Formula F1
(claim 8, which exempts a disc laser shot). Integer arithmetic throughout.

**F5. Charge-turret number** (claim 16; multi-image, non-gattling types only). *c* = min(*N* − 1, max(0, trunc(*N* × *r* / *D*))) when
*D* > 0 and the countdown runs; else 0. Integer arithmetic.

**F6. Starting ammunition** (claim 10). Signed integers, rounds; no rounding.
*a*₀ = *A* when *J* = −1, otherwise *J*. With `quirks.initial_ammo_above_capacity`
on (the default), an explicit *J* above a nonnegative *A* remains *J*. With the switch off,
that result is capped at *A*.

**F7. Eight-entry choice** (claims 20 and 21). Integers. With *f* the shot facing (facing units),
*d* = ⌊*f* / 8192 + ½⌋ mod 8 (S-0204 Formula F4 with *n* = 8) and *e* = (*d* + 1) mod 8. For a
building whose row of claim 21 rounds first, *f* is replaced by *f*′ = (65536 / *n*) ×
(⌊*f* × *n* / 65536 + ½⌋ mod *n*) with *n* = 256 or 32, and *d* is taken from *f*′.

**F8. Depth adjustment of a building's shot animation** (claim 22). Integers, leptons and depth
units of S-0102. With *y*ₗ the y-coordinate of the launch point and *y*₀ = 256 × the smallest cell
y of the building's foundation (the y-coordinate of its map-north edge):
*z*ₐ = −200 when the building has at least one occupant, otherwise
*z*ₐ = min(0, −trunc((*y*ₗ − *y*₀) / 4)), trunc toward zero.
The coordinates are signed 32-bit integers in leptons and *z*ₐ is a signed 32-bit integer in
depth units (S-0102). The formula uses integer division truncated toward zero; the selection
itself requires no further rounding.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Full-delay draw | R(0, 2) | ticks | Claim 2; one draw |
| In-burst delay draw | R(3, 5) | ticks | Claim 4; when no burst-delay field applies |
| Burst-delay fields read | 4 | fields | Claim 4; positions 1 to 4 of a burst, vehicles only |
| Firing sync values | 2 | burst-counter values | Claim 13; burst-counter values 0 and 1 |
| Building multi-round delay | 1 | ticks | Claim 10 |
| Veteran factor, absent-key default | 1.0 | factor | Import mapping |
| Occupy and bunker factors, absent-key default | 1.0 | factor | Import mapping |
| Difficulty-row and country factors, absent-key default | 1.0 | factor | Import mapping |
| Burst count, absent-key default | 1 | shots | Import mapping |
| Rate-of-fire field, absent-key default | 0 | ticks | Import mapping |
| Maximum and initial ammunition, absent-key default | −1 | rounds | Maximum −1 disables vehicle reloading; initial −1 selects the maximum (claim 10) |
| Empty-reload field, absent-key default | −1 | ticks | Import mapping; −1 = not used |
| Reload, reload increment, pip wrap, absent-key default | 0 | ticks, ticks, rounds | Import mapping |
| Facing-chosen animation list length | 8 | entries | Claim 20; every other length plays entry 0 |
| Entry shift of an eight-entry list | 1 | entries | Claim 20, Formula F7 |
| Occupied building's shot-animation depth adjustment | −200 | depth units (S-0102) | Claim 22, Formula F8 |
| Divisor of a building's shot-animation depth adjustment | 4 | divisor (leptons) | Claim 22, Formula F8 |
| Target line duration | 15 | ticks | Claim 24 (b) |
| Target line colour | 8 | palette index (general palette, row 26) | Claim 24 (g) |
| Target line width | 1 | pixels | Claim 24 (g) |
| Target line dot period | 2 | pixels (one drawn, one not) | Claim 24 (g); the phase swaps every tick |
| Target line end mark | 3 × 3 | pixels, bottom-right pixel on the end point | Claim 24 (g) |

## State and ordering

**A firer's firing state.**

| State | Event | Result |
|---|---|---|
| ready (no countdown running), or countdown running in claim 13's sync tick | an attempt passes the firing check and makes a shot | the shot is the *j*-th of its burst, *j* = *k* + 1; the countdown runs for the shot's delay *D*, of the kind that claim 2's order gives for *j*, with the factors of claims 3, 6 and 7 where that kind takes them and the halving of claim 8 where it applies; the burst counter is *j* mod *B* |
| ready | an attempt passes the firing check with a draining weapon | a drain attempt replaces the ordinary shot (claim 9; S-0608 claim 1); no countdown start, burst advance, ordinary per-shot ammunition use or ordinary-shot draws; target clearing still resets the burst counter; an admitted initial aircraft pass still makes its run round pending (S-0608 claim 16; S-0501 claim 15) |
| infantry walker, any rearm state | charge arrival under S-0519 claim 13 | countdown replaced by claim 19's non-shot delay; burst counter and ammunition unchanged by the query |
| carrying aircraft with a first passenger | firing reaches a drop attempt | no ordinary shot, burst advance, delay draw, spray draw, report or shot animation; refusal retains countdown and ammunition, success expires countdown and spends one round immediately; run spending remains separate (S-0522 claim 11) |
| ready | an attempt passes the firing check, but its projectile cannot be put on the map (a void shot, claim 15) | countdown, burst counter, ammunition and garrison muzzle position unchanged; projectile draws already taken remain consumed (claim 18) |
| ready, animation-delayed non-SAM building on the ordinary path | its attack behaviour gets *can fire*, outside an active upgrade gate | the delayed shot becomes pending (claim 14); no shot is made yet |
| ready, animation-delayed non-SAM building with active upgrade gate | *can fire*, gate passes | immediate primary and secondary attempts, no delayed shot started; an empty secondary does nothing (claims 9 and 14) |
| ready, animation-delayed non-SAM building with active upgrade gate | *can fire*, gate fails with the gate quirk on | no attempt and no delayed shot started (claim 14) |
| countdown running, outside claim 13's sync tick | an attempt | answer *rearming*, unless a reason of ranks 1 to 4 gives its own answer (claim 9); either way no shot and no draw |
| countdown running | *D* ticks have passed since the shot | ready |
| any | the firer's target is cleared | burst counter 0; countdown unchanged |
| building, delayed shot pending | any other firing check, including its attack behaviour's | *rearming* unless one of S-0405 table T1's answers holds, even against a target that a row of S-0306 claim 23 now refuses; the attack keeps its target (claims 9 and 14) |
| building, delayed shot pending | its turn max(*d* − 1, 0) ticks after the turn that started the shot | firing check with range, with the weapon slot chosen at the start: a shot at the current target on *can fire*, otherwise dropped with no projectile, countdown or draw; no longer pending (claim 14) |

**Ordering consequences.**
- The delay draw of a shot is taken in the firer's turn, after the draws of the projectile it
  launches and before those of its shot animation (claims 18 and 20); two firers that fire in the same tick take their draws in update order
  (S-0203).
- A shot's animation is created after its projectile is placed and after its delay draw, and
  before its sonic wave or laser (claims 18 and 22; S-0109 claim 29). Choosing its type takes no
  draw (claim 20).
- A shot made in tick *t* with delay *D* ≥ 1 allows the next shot in tick *t* + *D* at the
  earliest (claim 1), unless that next shot is a sync shot (claim 13) or another shot of the
  aircraft's same-tick full burst (claim 4), or an unchecked building secondary attempt
  (claim 9). So measured shot intervals are *D* ticks when the firer attempts every tick
  and uses none of those exceptions.
- A building with a pending delayed shot reports *rearming*, unless one of S-0405 table T1's
  answers holds (claim 9); under S-0215 claim 32 its attack
  behaviour is therefore processed every other tick (it keeps turning towards its target) until the
  shot. The delayed shot itself is timed by the building's own turns, one per tick, so its tick is
  the same whether or not a processing falls in it (claim 14).

## Edge cases and quirks

- **Building rounds** (`quirks.building_ammo_fast_rearm`, default on). A building whose ammunition
  is above 1 rearms in 1 tick after every shot (claim 10), over every other kind of delay. With the
  switch off, the delay is the one that claim 2's order gives without its first row, at every
  ammunition level. Backing: `measured` (M-0004, a building with a burst weapon, before the full and
  in-burst delays); `measurable` (scenarios F9 and F4, building parts, for other weapons and before
  the bare rate); confidence high before the full and in-burst delays, medium before the bare rate.
- **Shared wave slot** (`quirks.shared_wave_slot`, default on). Sonic waves and magnetic-beam waves
  share one effect slot per firer (claim 5). With the switch off, each has its own slot and a live
  magnetic wave does not affect a sonic weapon. Backing: `measurable` (scenario F4, shared slot
  part); confidence low.
- **Bare-rate effect weapons and berserk.** A sonic weapon's delay is the bare field and is still
  halved when berserk (claim 8 applies after the choice). This is the original behaviour; no switch. Backing: `measurable` (scenario F6, bare-rate
  part), confidence high.
- **The campaign ignores the country factor** (claim 3). Historical original behaviour only.
  no campaign rule is specified for our engine. Backing: `unmeasured` here because campaigns
  are out of scope and no campaign run is proposed; confidence high.
- **Burst count 0 or less** (departure, no switch). The original fails on a burst count of 0 (a
  division by zero). For a negative count *B* it keeps going: after the shot at position *j* the burst
  counter is the remainder of *j* divided by |*B*|, a value from 0 to |*B*| − 1, but no position is
  below *B*, so every
  shot gets the full delay and never an in-burst delay. Our engine treats any burst count below 1
  as 1, and the importer warns, naming the weapon. Policy: bad data gives a safe deterministic
  result plus an importer warning. Backing: `unmeasured` for the original's zero- and
  negative-count behaviour (confidence medium): no scenario here attempts the failing input
  or isolates the negative-count remainder. Our engine's safe replacement and warning are
  checked by the invalid-burst part of scenario F16 (`measurable`, confidence high).
- **Negative or zero delays.** A rate-of-fire field of 0 or less, a negative house, veteran or
  bunker factor, or a burst-delay field below −1, can give a delay of 0 or below; the countdown is
  then over at once (claim 1). The importer warns for a negative rate-of-fire field, a negative
  factor, or a burst-delay field below −1. A bunker factor or an occupy factor above 0 and below 1
  lengthens the delay; no warning. Backing: `measurable` (scenario F16, including its
  negative-factor, burst-delay and fractional-occupy parts); confidence high.
- **Occupy factor not above 0, bunker factor 0.** Then that division is skipped (claim 7). An
  occupy or bunker value written as text that is not a number is read as absent (S-0003 claim 8,
  a departure of that spec), so the factor keeps its default of 1.0 and divides by 1 (backing `measurable`, scenario F5, skips
  part, confidence high). The original
  also skips the division for a factor that holds a not-a-number value; no imported text produces
  one, so there is no scenario for it (`unmeasured`, confidence low).
- **Starting ammunition above the maximum** (`quirks.initial_ammo_above_capacity`, default
  original). An explicit initial field above a nonnegative maximum supplies that many rounds,
  not the maximum. With the switch off, the starting count is capped at the maximum.
  Backing: `measurable` (scenario F9, starting-count part); confidence high.
- **A building with an unlimited maximum that starts empty.** With a maximum of −1 and an
  initial count of 0, claim 10's refill gives −1 at the end of the building's first tick, so it
  fires as an unlimited building from then on. A vehicle or an infantry unit with the same
  fields stays empty (claims 10 and 11). This is the original behaviour; no switch. Backing:
  `measurable` (scenario F9, starting-count part); confidence medium.
- **Ammunition below −1.** A firer whose *a* is below 0 spends no rounds when firing
  (a round is used only when *a* > 0) and is never *out of ammunition*
  (that answer needs *a* = 0). Without reloading it therefore behaves as unlimited.
  vehicle reloading is still governed by claim 11. Backing: `measurable` (scenario F9, below-minus-one part), with
  confidence medium.
- **Firing sync frame at or above the counter's start.** The counter starts at 2*F* − 1 and only
  falls, so a sync frame *s* ≥ 2*F* is never reached: read literally, claim 13's gate gives the
  primary weapon *rearming* for the whole run of the counter, so a following shot comes only after
  the counter is idle again. Backing: `measurable` (scenario F11, beyond-the-start part, test art
  data); confidence low. The part also settles the first claim-13 open question.
- **Ammunition in reload arithmetic.** Formula F3 is integer arithmetic. A pip-wrap field below 0 gives a *q* of
  the opposite sign to *a*, or 0 when *a* is smaller than the field's magnitude, because the
  division is toward zero; the square of *q* is never negative. No warning. Backing: `measurable` (scenario F10, part f), confidence
  medium.

## Import mapping

| Key (file, section) | Concept | Form | Absent |
|---|---|---|---|
| `ROF` (rules, weapon) | rate-of-fire field *R* | integer, ticks | 0 |
| `Burst` (rules, weapon) | burst count *B* | integer | 1 |
| `IsSonic`, `UseSparkParticles`, `UseFireParticles`, `IsRailgun` (rules, weapon) | effect weapon kinds (claim 5) | boolean | no |
| `IsMagBeam` (rules, weapon) | magnetic-beam weapon (claim 5, Terms) | boolean | no |
| `Temporal` (rules, warhead) | temporal weapon; the primary weapon's warhead initially supplies the firer's temporal beam (Terms) | boolean | no |
| `Spawner` (rules, weapon) | spawner weapon (Terms, claim 9) | boolean | no |
| `HunterSeeker` (rules, vehicle type) | seeker type (Terms, claim 9) | boolean | no |
| `DiskLaser` (rules, weapon) | disc laser weapon (claim 8) | boolean | no |
| `DecloakToFire` (rules, weapon) | needs the firer uncloaked (claim 12) | boolean | yes |
| `BurstDelay` followed by 0, 1, 2 or 3 (rules, vehicle type) | burst delay *j* = 1 … 4 (claim 4) | integer, ticks | −1 |
| `FiringFrames` (art, vehicle image) | firing frames *F* (claim 13) | integer | 0 |
| `FiringSyncFrame` followed by 0 or 1 (art, vehicle image) | firing sync frames for burst-counter values 0 and 1 (claim 13) | integer | −1 |
| `Ammo` (rules, type) | maximum ammunition *A*, also the starting count when *J* = −1 (claim 10) | signed integer, rounds | −1 |
| `InitialAmmo` (rules, type) | initial-ammunition field *J*: −1 selects *A*, every other value is literal, with no cap by default (claim 10, Formula F6) | signed integer, rounds | −1 (selects *A*, not necessarily unlimited) |
| `Reload`, `EmptyReload`, `ReloadIncrement` (rules, type) | *L*, *E*, *I* of Formula F3 | integer, ticks | 0, −1, 0 |
| `Hospital`, `Armory` (rules, building type) | the hospital flag and the armory flag: a building type with either is not refilled (claim 10); S-0314 claim 7 and S-0311 claim 12 own their other effects | boolean | no |
| `PipWrap` (rules, type) | *W* of Formula F3 | integer | 0 |
| `ManualReload` (rules, type) | no reload restart on firing (claim 11) | boolean | no |
| `MobileFire` (rules, type) | reload advances while moving (claim 11); see S-0310 | boolean | yes |
| `DamageReducesReadiness` (rules, type) | claim 17 | boolean | no |
| `VeteranAbilities`, `EliteAbilities` (rules, type) | the entry `ROF` in either list sets the veteran or elite rate-of-fire ability (claim 6) | list | — |
| `VeteranROF` (rules, `General`) | veteran factor *v* | decimal (S-0003, claims 6 and 7), held as a double | 1.0 |
| `OccupyROFMultiplier`, `BunkerROFMultiplier` (rules, the combat-damage section) | *m*, *b* | decimal (S-0003, claims 6 and 7), narrowed to single precision | 1.0 |
| `ROF` (rules, the three difficulty-row sections `Easy`, `Normal`, `Difficult`) | difficulty-row factor *ℓ* (easy, normal, difficult row) | decimal (S-0003, claims 6 and 7), held as a double | 1.0 (see S-0006 for a layer that has the section but not the key) |
| `ROF` (rules, country section) | country factor *κ* | decimal (S-0003, claims 6 and 7), held as a double | 1.0 |
| `CompEasyBonus` (rules, `General`) | the computer-easy-bonus flag of claim 3 (the original's set-up only; read by the optional handicap setting) | boolean | no |
| `CanBeOccupied`, `CanOccupyFire` (rules, building type) | garrisoned building (Terms) | boolean | no |
| `IsAnimDelayedFire`, `DelayedFireDelay` (art, building image) | claim 14's ordinary delayed shot; a passing upgrade gate bypasses both | boolean; integer, ticks | no; 0 |
| `LandTargeting`, `NavalTargeting` (rules, type) | land-targeting and naval-targeting modes of the firer's type (claims 9 and 14, scenario F21); S-0308 owns the keys, S-0306 claim 23 the rows | integer, used as given | 0 |
| `Primary`, `Secondary`, `ElitePrimary`, `EliteSecondary` (rules, type) | weapon references for slots 0 and 1 and their elite replacements; S-0308 claims 1, 2 and 12 own resolution; claim 9 here covers an empty resolved secondary | weapon type name | held value; engine default no weapon |
| `IsChargeTurret`, `TurretCount` (rules, vehicle type) | charge turret and *N* (claim 16) | boolean; integer | no; 0 |
| `Anim` (rules, weapon) | animation list (Terms, claim 20); its length *n* and entry order as written | list of animation type names | empty list |
| `OccupantAnim` (rules, weapon) | occupant animation (claim 20 b) | animation type name | none |
| `OpenToppedAnim` (rules, weapon) | open-topped animation (claim 20 c) | animation type name | none |
| `AssaultAnim` (rules, weapon) | assault animation; not used by ordinary shots (Terms) | animation type name | none |
| `Turret` (rules, vehicle type) | the vehicle row of claim 21; S-0310 owns the flag | boolean | no |
| `TargetLaser` (rules, type; only a vehicle shows the line) | the target line of claim 24 | boolean | no |
| none | general palette, whose index 8 colours the target line (claim 24 g) | palette selection inherited from S-0306 claim 30 | no independent selector |

The importer warns for a `Burst` below 1, a negative `ROF`, a negative factor, and a burst-delay
value below −1 (Edge cases).

## Measurement scenarios

Scenario labels F1 to F21 are separate from the formulas F1 to F8; the text says "scenario" or
"Formula" each time. All scenarios use stock data unless a named test rules file is listed, under the measurement setup
(glossary), a small purpose-made skirmish map, one game speed (any; claim 1 says it does not matter),
and the random-state recorder where draws matter. Unless a scenario states otherwise, each distinct
setup is run once; any explicitly stated repetitions or comparisons between runs take precedence.
"Shot ticks" means the tick numbers at which the firer's shots are made, read from projectile creation or the target's health drops.

**F1. Rearm countdown and full delay.**
- Setup: one human-house vehicle with a single-shot weapon (burst count 1, not an effect weapon) and
  a stationary enemy target building with high strength in range; the house uses the normal row; the country factor is 1.0 (test rules file sets it if needed). Run a second time at another
  game speed. In a third run, while a countdown runs, change the target to another building in
  range, and in a fourth give the vehicle an order to use its other weapon (test data: a second
  single-shot weapon in slot 1). In a fifth, with a turreted vehicle, while a countdown runs,
  change the target to a second building in range at a bearing that its turret reaches before
  the countdown's due tick, so that the turn ends while the countdown still runs. A run in which
  the turret turn does not end while the countdown runs decides nothing.
- Observable: shot ticks over 20 shots; the recorder's draw results for R(0, 2).
- Expected: each interval equals *D*₁ of Formula F1 for that shot's draw *u*; the first shot
  comes in the first tick in which the vehicle is facing the target; the intervals in ticks are
  governed by the same formula at both speeds, using each run's recorded draws rather than
  requiring independently seeded runs to have identical intervals; the target change, the
  weapon change and the turret turn of the fifth run do not move the tick of the next shot.
- Falsified if: any interval differs from that value, the formula's tick delay depends on game speed, or a
  target or weapon change or a turret turn shortens or lengthens the running countdown.

**F2. House and veteran factors.**
- Setup: as F1, with a test rules file that sets the house's normal-row factor to 1.5 and its
  country factor to 0.8 (expected *h* as Formula F2 gives it, about 1.2),
  and optionally computer houses at each lobby entry (easy, normal and hard computer: the original's
  set-up of claim 3 with the pairing of S-0412 claim 4, recorded and not reproduced by our engine; in
  these runs the rows' factors differ, and the expected *h* is the factor of the row that claim 3
  gives the entry times the country factor); a veteran vehicle of a type with the veteran
  rate-of-fire ability, and an elite one of a type with only the veteran ability; the test rules
  file also sets *v* = 0.5. Include a rookie control, a veteran with only the elite
  rate-of-fire ability, an elite with only that elite ability, and an elite with both abilities,
  each in a separate run with otherwise identical fields.
- Observable: shot intervals and draws.
  A further run sets *v* by test data, once written as 0.6 and once as 60%, for a weapon whose
  *D*₁ is 10.
- Expected: Formulas F1 and F2 exactly; elite gets the factor once with either or both
  abilities, while the rookie and the veteran with only the elite ability get no factor.
  The 0.6 run gives *D*₂ = 6 (the single
  just above 0.6), the 60% run *D*₂ = 5 (a double just below 0.6 under the round-toward-zero mode).
- A further run (the non-player houses): the test rules file gives the three rows the factors 2.0,
  1.5 and 0.5 (easy, normal, difficult) and the neutral country the factor 1.5 (all exact in binary).
  A defence building of the neutral house with a rate-of-fire field of 40, not veteran, fires at a
  stationary target of the special house (S-0412 scenario K5 part d sets this up) over 20 shots.
  Expected: every interval 40, 41 or 42 ticks (*D*₁ with *h* = 1.0 and the draw *u*). Falsified if an
  interval lies outside 40 to 42, in particular at 60 to 62, 80 to 82 or 90 to 92 (a country factor
  or a row applied).
- Falsified if: the country factor does not act in the skirmish run, the
  skirmish intervals do not follow that *h*, the elite interval shows the factor twice, the 0.6 run
  gives a *D*₂ other than 6, the 60% run gives one other than 5, or an ability control
  applies or omits the factor contrary to the expected rank test.

**F3. Bursts.**
- Setup: a vehicle whose weapon has burst count 3 and whose type sets burst delays 1 and 2
  (claim 4) to −1 (test rules file), then the same with them set to 7 and 9; an infantry unit with
  a burst-3 weapon.
- Observable: shot ticks, the countdown after each shot, and draws.
- Expected: with the fields at −1, intervals inside a burst are 3, 4 or 5 matching the draws,
  and after the third shot the full delay; with 7 and 9, the vehicle's intervals are 7 and 9 with
  no draw; inside a burst the infantry unit always draws R(3, 5), and its countdown after such a
  shot is that draw. Clearing the target in mid-burst (stop order) and
  giving a new one starts a fresh burst. Changing the target (not clearing it) in mid-burst keeps
  the burst going.
- Carry-over: a vehicle with a burst-3 weapon in one slot and a burst-2 weapon in the other (test
  rules file), made to fire the first once and then the second. Expected: the second weapon's shot is
  position 2 of 2 of its burst, so its delay is the full delay.
- Building part: a building defence whose weapon has burst count 2 and is not an effect weapon, with
  3 rounds (test rules file). Expected: shots in ticks *t*, *t* + 1 and *t* + 2, and no R(3, 5)
  draw after the first two (claim 2's order puts the 1-tick delay first); the third shot is made
  with the last round and is the first of a new burst (*j* = 1 < *B*), so the delay after it is
  one R(3, 5) draw, not a full delay (the recorder shows it). With 4 rounds instead, the fourth
  shot is *j* = 2 = *B* and
  its delay is the full delay, with an R(0, 2) draw. Falsified if the second shot comes 3 to 5
  ticks after the first, if no R(3, 5) draw follows the third shot of the 3-round run, or if an
  R(0, 2) draw follows it.
- Falsified if: any interval or delay draw differs from the expected case, an infantry unit's
  in-burst countdown differs from its draw, in-burst intervals use the full delay, the counter is
  not reset by the stop, a
  change of target resets it, or the carried count does not decide the second weapon's delay.

**F4. Effect weapons.**
- Setup: a vehicle with a sonic weapon and a vehicle with a railgun weapon; named test rules
  supply either weapon assignment when needed.
- Observable: shot ticks; draws; a read-only recorder's firing-check answers before the
  attack behaviour responds. Where this scenario names an answer, a different recorded
  answer falsifies that label independently of whether a shot follows.
- Expected (sonic weapon): no delay draw is taken; intervals are max(*R*, wave lifetime).
- Falsified if: a draw is taken for the delay, or an interval differs from
  max(*R*, wave lifetime) when facing, range and all other firing gates remain satisfied.
- Particle part: a test rules file gives four firers one weapon each, with *R* shorter than every
  emitter's life: a spark-particle weapon, a railgun weapon and two fire-particle weapons, each
  weapon's system type of the kind that matches its flag (each firer with a stationary durable
  target building in range, all other gates satisfied). The first
  fire weapon's system type has a lifetime of 0 or less, and the second's a positive lifetime *L*
  longer than *R*. Record shot ticks, and the tick at which each emitter and each system leaves
  play (S-0616 scenario P1). Expected: the spark and railgun firers shoot again at the first
  attempt after their emitter has left play, and not earlier. The first fire firer shoots once and
  not again while it keeps the target and stays put. The second fire firer shoots again at the
  first attempt after its system has left play, and not earlier (claim 5; S-0613 claim 17 gives
  when the system leaves play). Falsified if a spark or railgun firer shoots before its emitter
  has left play or never shoots a second time, if the first fire firer shoots a second time with
  its target kept, or if the second fire firer shoots again before its system has left play or
  does not shoot again after it.
- Unmatched part (Q-2181): the particle part's setup, with six firers whose weapon's system type is
  of a kind that does not match its flag, each type with a lifetime of 0 or less and a rate-of-fire
  field shorter than the system's life: (u1) fire flag, spark-kind type with a spark-turns field of
  1 or more; (u2) fire flag, railgun-kind type; (u3) spark flag, fire-kind type; (u4) railgun flag,
  fire-kind type; (u5) spark flag, railgun-kind type; (u6) railgun flag, spark-kind type with a
  spark-turns field of 1 or more. A seventh firer (u7) is u3 with a positive lifetime. Each keeps
  its target and stays put for a run of a test length. Record shot ticks and the tick at which each
  system leaves play (S-0616 scenario P1). In a second run, give u1's type a spark-turns field large
  enough that a system of that type made at u1's first shot is still in play when u1's new attack
  order below is given (a test value), and a spark chance field of 0 (S-0616 claim 15, F10). The
  second run's Expected needs u1's and u8's systems to hold no particle before their last counter
  turn: the Void line makes a run in which either bursts before that turn void, so in a counted
  run neither does. Add u8, a copy of that u1 that is given no order and that makes its first shot
  in the same tick as u1 or earlier. After the first shot of u1, of u3 and of u4, give each a move
  order to a neighbouring cell from which the target stays in range, then a new attack order on
  the same target, both while u8's system is still in play, and record the same.
  - Expected: u1, u2, u5 and u6 shoot again at the first attempt after their system has left
    play, and not earlier. u3 and u4 shoot once only. u7 shoots again at the first attempt after
    its system has left play. In the second run, u1 shoots again at its first attempt after the new
    attack order that is at least *R*′ ticks after its first shot, before u8's system has left play
    (the move order let u1's system go), and u3 and u4 shoot no second time. u1's first spark
    system leaves play in its first turn after the move order (the let-go marked it, and in a
    counted run it held no particle, see Void), while u8's system stays in play until its counter
    has run out.
  - Falsified if: in the first run, u1, u2, u5 or u6 shoots before its system has left play or
    never shoots a second time, u3 or u4 shoots a second time, or u7 does not shoot again after its
    system has left play; in the second run, u1 does not shoot again before u8's system has left
    play, u3 or u4 shoots again, u1's first system stays in play after its first turn following the
    move order, or u8's system leaves play before its counter has run out.
  - Void: a run in which a firer is removed, leaves the map or is destroyed, or a trigger action
    marks a system. A second run in which u1's or u8's system bursts before its last counter turn
    is void.
  - Third run (Q-2246, the moment of a retarget's let-go): three firers, each with a second enemy
    placed outside its weapon's range. u9 is u1's pairing with a spark chance field of 1, a
    particle cap of 2 or more and a spark-turns field large enough that its counter has not run
    out by the end of the run. Its held spark type has x and y velocity fields of 1, a vertical
    velocity range of 1, a minimum vertical velocity above twice the rules' gravity and a particle
    lifetime of 2 or more ticks, and its system type keeps the default spawn direction (0, 0, 0)
    (F10, F11 of S-0616), so that every spark rises with no sideways
    motion in its first update and the system holds the sparks of each burst at the end of the
    turn that makes them. u10 has a fire flag and a smoke-kind type with a spawn interval of
    1, a slow-down field of 0 and a spawn cut-off of 1 or more. The third is the particle part's first fire firer with its type's
    spawn interval set to 1. Each of the three types has a held particle type, and u10's names
    no next particle type. All of these are test values, so that each system, while unmarked,
    makes a particle in every turn. After the
    first shot of each, give it an attack order on its second enemy. For each firer, let *t* be
    the tick at whose end its new target is first recorded. Record every tick whether its first
    system takes its turn and whether it makes a particle in that turn, and the tick at which
    that system leaves play. A run in which a system takes no turn in tick *t* or in a later tick
    of the record while it is in play is void.
    - Expected: u9's system makes sparks in every turn, in tick *t* + 1 and later as before (with
      a chance field of 1 its decision always passes).
      u10's and the fire firer's systems make a particle in tick *t*'s turn and none in tick
      *t* + 1's turn or any later one.
    - Falsified if: u9's system makes no sparks in a turn after tick *t* while its counter is above
      0, or u10's or the fire firer's system makes a particle in tick *t* + 1 or later, or
      none in tick *t*.
- Multi-flag part: use S-0616 scenario P22 part a's named test rules with all three
  particle-effect flags on one weapon, one named system type and a durable enemy vehicle
  in range. Keep the firer stationary and all other gates satisfied, at speed index 2,
  with no engine add-on, the measurement harness's in-game helper off and no mods except the named test data. In three runs, order sustained
  attack and record shot ticks, emitter creation order, held places and firing-check answers
  with tick-labelled capture and a read-only recorder. Expected: the first shot makes three
  emitters of that same system type in fire, spark, railgun order; no further checked shot
  occurs while any of those places is held, even after the rearm countdown expires. Repeat
  with each two-flag combination and a single-flag control. A missing emitter, different
  creation order or checked shot with any selected place held falsifies the corresponding
  rule. If ordinary play never frees all selected places, report the later-shot boundary
  inconclusive rather than treating permanent refusal as evidence for that boundary.
- Fire let-go part: as the first fire firer above (lifetime 0 or less), then, in separate runs,
  (a) retarget it to a target outside its weapon's range, (b) give it a move order, (c) kill it,
  (d) retarget it to a second target that is within range. Expected: in (a) and (b), a shot at the
  first attempt after the firer again has a target in range; in (c) none; in (d) no second shot.
  Falsified if a shot comes in (c) or (d), or none comes in (a) or (b) while all gates hold. This
  part reads "within reach" as the weapon's range. Where (a) and (d) disagree with that, the
  measurement replaces the reading and S-0616 claim 1 is revised.
- Burst part (claim 2's order): with a test rules file, the sonic vehicle's weapon gets burst count
  2, and its *R* is longer than the wave's lifetime. Expected: after every shot, including the
  first of a burst, the interval is *R* and the recorder shows no R(3, 5) draw. Falsified if the
  interval after the first shot is 3 to 5 ticks (a wave still alive would then answer *rearming*
  for the weapon's own kind, so the shot ticks show it) or a draw appears.
- Building part: with a test rules file, a building defence that cannot be garrisoned, with a
  sonic weapon (or a spark-particle weapon), *R* far longer than the effect's lifetime, and 3
  rounds. Record shot ticks, the building's ammunition at the end of every tick and whether it
  keeps its target. Expected: the second shot comes when the first wave ends (for a
  spark-particle weapon, when its emitter has left play), not *R* ticks after the first (a 1-tick
  delay with the effect gate holding it), and the third at the first attempt after the second
  wave ends. The building
  is refilled to 3 rounds at the end of the tick of its third shot (claim 10). The delay after
  that shot is *R*, so its next group starts at the first attempt *R* ticks after the third
  shot. As in scenario F9, it keeps its target throughout and is never at 0 rounds at the end of
  a tick. Falsified if the second shot comes *R* ticks after the first, the building holds 0
  rounds at the end of a tick, drops its target, or makes no shot of a next group once *R* has
  run since its third shot.
- Shared slot: with test data, a vehicle that carries a sonic weapon and a magnetic-beam weapon,
  the magnetic weapon's full delay (its rate-of-fire field plus up to 2) set shorter than the
  wave's lifetime (the effect specs give it), so that the rearm countdown is over while the wave
  is alive and only the wave can hold the sonic weapon back. Fire the magnetic one, then try the
  sonic one while the magnetic wave is alive. Expected: no sonic shot until the wave ends.
  Falsified if the sonic weapon fires while the wave is alive. Control: with
  `quirks.shared_wave_slot` off it fires at once.
- Magnetic control: named test map data supplies a magnetic-beam weapon with an ordinary,
  non-seizing warhead and a full delay shorter than the wave's remaining lifetime. Use a
  stationary firer and durable target building, speed index 2, no engine add-on, the measurement harness's in-game helper off and no mods (the named test files of a scenario and the purpose-made test maps excepted).
  Keep the target unchanged and all other firing gates satisfied. Record shot ticks, grip
  presence and live wave count with tick-labelled capture and a read-only recorder, for three
  runs. Expected: no grip; repeated shots follow their claim-2 delays while the first wave
  remains alive, with no second wave. Falsified by an own-wave-only firing refusal or a new
  wave on those later shots. S-0607 scenario L9 parts c and d separately tests the conditional
  refusal while gripping and the continued grip after retargeting to a building.
- Other weapon: with test data, a firer with a sonic weapon in slot 0 and an ordinary weapon in slot
  1, and a firer with the two the other way round, the sonic weapon's rate-of-fire field set below
  the wave's lifetime so that the rearm countdown is over while the wave is alive. Fire the sonic
  weapon and ask for the ordinary one while the wave is alive. Expected: the ordinary weapon does
  not fire until the wave ends (the check answers *impossible*), and fires in the first attempt
  after it. Falsified if it fires while the wave is alive.

**F5. Garrisons and bunkers.**
- Setup: a garrisonable building with 1, 2 and 4 occupants of one infantry type attacking a target,
  a vehicle inside a tank bunker; the stock occupy and bunker factors.
- Observable: shot intervals of the building and the bunkered vehicle.
- Expected: Formula F1 with *n* and *m*, and with *b*.
- Skips (test rules file): an occupy factor of 0 and a negative one, and a bunker factor of 0.
  Expected: each division is skipped, so the delay is the one without it. Falsified if a division by
  zero or a changed delay appears.
- Invalid-text controls: named test rules separately give each factor a nonnumeric value,
  with no earlier layer supplying it. Expected on our engine: the absent default 1.0 and
  S-0003 claim 8's warning, not a division by an invalid number. A different imported value,
  missing warning or different delay falsifies that departure; this is not an oracle prediction.
- Falsified if: the intervals do not scale as Formula F1 says.
- Death-removal comparison (our engine only): use S-0508 scenario G18's foreign-owned
  garrison and defeat its occupants' house. Record the occupant count in the death update.
  Expected: the count falls by the number killed, reaching 0 after the last death; no removed
  place supplies a weapon. In G18's controlled partial-removal fixture, retain a living
  occupant and a legal target, then record each subsequent shot's delay and delay draw.
  Expected: Formula F1 uses the reduced count, not the pre-death count. A retained count,
  removed-place shot or delay computed with the old divisor falsifies the engine rule.
  This is a design check, not an oracle prediction; do not seek confirming original shots
  through dead places. S-0508 scenario G15 limits the original comparison to count retention
  before unsafe continuation.

**F6. Berserk.**
- Setup: a vehicle made berserk by a stock berserk warhead, firing at a nearby unit.
- Observable: shot intervals and draws.
- Expected: half the claim-2 delay, truncated. A further run, with test data giving the berserk
  vehicle a disc laser weapon: the interval is the full delay.
- In-burst and bare-rate parts (test rules file): a berserk vehicle with a burst-3 weapon and no
  burst-delay fields, and a berserk vehicle with a sonic weapon. Expected: the in-burst intervals
  are 1 or 2 ticks (the R(3, 5) draw halved and truncated, matching the draws); the sonic interval
  is max(trunc(*R* / 2), the wave's lifetime). A bare rate of 1 gives a halved value of 0, so only
  the wave's lifetime holds it.
- Falsified if: the interval is the full delay or rounds up, the disc laser interval is halved,
  or an in-burst or bare-rate delay is not halved.

**F7. Precedence of answers.**
- Setup: (a) a building defence whose countdown runs when its target moves out of range,
  (b) a vehicle with *A* = 1 whose countdown runs when its round is spent; (c) a cloaked unit whose
  weapon needs decloaking, and, with test data, an aircraft with the same weapon and cloak; (d) a deploy-to-fire vehicle on a sloped cell (S-0310, scenario T8), with
  a target outside its aiming tolerance.
- Observable: whether the building keeps its target and turns (S-0215, claim 32, second row) until
  its countdown ends; in (b), the tick at which the combat specs' consequence of *out of
  ammunition* (S-0310 claim 9 and S-0215, for example the vehicle giving up the attack or going
  to reload) starts, which is read from the vehicle's order state and position; when the cloaked
  unit uncloaks; and,
  in (d), the vehicle's turret or body facing.
- Expected: (a) the building keeps the target until the countdown ends and drops it only at the next
  processing after; (b) the consequence of *out of ammunition* starts only after the countdown; (c) the unit
  uncloaks before its first shot, and the aircraft does so only while fully cloaked, not while
  cloaking or uncloaking; (d) the vehicle does not re-aim, as the vehicle gets *must deploy* and not *not
  facing*.
- Falsified if: (a) the building drops its target while its countdown runs; (b) the consequence of
  *out of ammunition* starts while the countdown still runs; (c) the non-aircraft unit fires
  before becoming fully visible, or the aircraft's cloak refusal persists while cloaking or
  uncloaking with all other gates satisfied; (d) the vehicle re-aims.

- (e) Empty passenger carrier: use the named facing-test override of S-0501 scenario
  AC11 on a small skirmish map, speed index 2, no engine add-on, the measurement harness's in-game helper off and no mods (the named test files of a scenario and the purpose-made test maps excepted), with a
  bystander computer player outside the test area under the glossary's void rule.
  Place a controllable non-fighter aircraft loaded, then unload every passenger. Give
  it ample ammunition and a durable enemy ground target. Compare a fresh aircraft of
  the same type placed empty. Repeat three times, recording ammunition, passenger count,
  aiming facing, shot ticks and the firing-check answer before the attack behaviour
  responds, using tick-labelled capture and a read-only recorder as in AC11.
  Test the emptied marked carrier with: (i) countdown running and aim wrong, (ii) countdown
  over, target out of range and the attempt asking for range, (iii) countdown over and
  target in range but aim wrong, and (iv) countdown over, in range and correctly aimed.
  Its last successful passenger drop expires its countdown (S-0522 claim 11), so does not
  supply case (i). Test that case only if normal inputs establish an emptied marked carrier
  with a running countdown; otherwise report it inconclusive. Keep all other refusal causes absent. Expected answers are respectively *rearming*,
  *out of range*, *out of ammunition* and *out of ammunition*, with no ordinary projectile
  shot in any case. The fresh empty control answers *not facing* in case iii and may fire
  in case iv. A different recorded answer, or ordinary projectile fire from the emptied
  marked carrier, falsifies the corresponding part. A state or range-request combination
  not reached through normal orders is inconclusive, not a passing trial.

- (f) Grip precedence: use S-0607 scenario L9's small skirmish map and named test data,
  speed index 2, no engine add-on, the measurement harness's in-game helper off and no mods
  except the named test data. Keep a bystander computer
  outside the test area. Give the lifter a non-magnetic weapon whose projectile cannot hit
  airborne units. While its gripped unit is airborne, retain that unit as its target as in
  L9 part f. Record target retention and the firing-check answer with tick-labelled capture
  and a read-only recorder over three runs. Expect *rearming*, not a projectile or layer
  refusal, and no shot. Then, while grip and wave still coexist, attempt a magnetic shot at
  a different airborne non-building target the projectile cannot hit. Expect that target's
  projectile or layer refusal, not the conditional grip-and-wave *rearming*. A contrary
  answer falsifies the corresponding precedence rule. Void a trial if the grip, wave or
  target changes before the recorded check, or another earlier refusal applies. A state
  not reached through normal orders is inconclusive.

**F8. Facing only when ready.**
- Setup: S-0206's turret scenario with a target that moves round the vehicle during its rearm period.
- Observable: the turret facing every tick.
- Expected: the facing-error re-aim (the facing-error row of S-0206 claim 9, fed by the replies of S-0310 claim 9) acts only in ticks when the weapon is ready.
- Falsified if: the re-aim acts during the rearm period.

**F9. Ammunition.**
- Setup: test rules file: a building defence whose weapon has burst count 1 and is not an effect
  weapon, with maximum ammunition 3 and initial ammunition 3 and a reload field far longer than
  the observation, and an infantry unit with maximum and initial ammunition 2 (both keys set, so
  the result does not depend on how a new object's ammunition is derived).
- Observable: shot ticks, the number of shots, each firer's ammunition at the end of every tick,
  and whether the building keeps its target.
- Expected: the building fires in ticks *t*, *t* + 1 and *t* + 2, holds 3 rounds again at the
  end of tick *t* + 2, and repeats the group of three each time the full delay after a group's
  third shot has run. It keeps its target throughout and is never at 0 rounds at the end of a
  tick. The infantry unit fires twice and never again.
- Falsified if: the building stops firing after a group, holds 0 rounds at the end of a tick,
  drops its target for lack of ammunition, or its intervals differ, or the infantry unit regains
  rounds.
- Paired-refill part (claim 10): scenario F17's passing control (a populated single-shot
  secondary, the upgrade gate passing), with the host's maximum ammunition set to 2 in place of
  unlimited ammunition. Give the primary and the secondary weapon a rate-of-fire field of 30 (a
  test value), so that every full delay of Formula F1 is longer than 1 tick. Record the
  ammunition and the rearm countdown at the end of every tick, and both attempts' shots. If the
  host's ammunition cannot be read at the end of a tick, the part is inconclusive.
  - Variant a, initial ammunition 2. Expected: in the first paired processing the primary shot,
    made with 2 rounds, takes the 1-tick delay and leaves 1 round; the secondary shot, made with
    1 round, takes the delay claim 2 gives a building at its last round and leaves 0; the
    building holds 2 rounds at the end of that tick. Falsified if it holds 0 or 1 round at the
    end of that tick (a refill tested only between the two attempts or not in that tick), or if
    the secondary shot takes the 1-tick delay (a refill before it).
  - Variant b, initial ammunition 1, so that the primary shot empties the magazine. Expected: in
    the first paired processing the primary shot, made with the last round,
    leaves 0; the secondary attempt is still made, at 0 rounds (it asks no firing check,
    claim 9), takes the full delay and uses no round; the building holds 2 rounds at the end of
    that tick. Falsified if the secondary shot takes the 1-tick delay or the building holds 1
    round at the end of that tick (a refill right after the shot that emptied the magazine, so
    that the secondary is made with 2 rounds), if it holds 0 rounds then (no refill in that
    tick), or if the secondary attempt makes no shot (an ammunition gate on the unchecked
    attempt).
- Starting-count part (Q-0060 question 1): a named test rules file sets a buildable vehicle,
  infantry and ungarrisoned defence building to maximum ammunition 4. Remove the initial field
  from every rules layer for those types. Use a small isolated skirmish test area, speed index 2,
  no engine add-on, the measurement harness's in-game helper off and no mods (the named test files of a scenario and the purpose-made test maps excepted), with one bystander computer player kept outside the area
  under the glossary's void rule. Repeat with an explicit initial field of −1, 0 and 6.
  also test maximum 0 with the initial field absent, and maximum −1 with initial field 0.
  Supply a single-shot ordinary weapon with a rate-of-fire field of 30 (a test value), so that
  every full delay of Formula F1 is longer than 1 tick, and a durable stationary enemy target in
  range.
  Set the vehicle's ordinary and empty reload delays longer than the observation window,
  leave manual reload off, and exclude damage, servicing and ammo-changing effects.
- Procedure: in each case compare one map-placed object with one produced and released by the
  human house. Map-placed vehicles and infantry belong to the human house; map-placed
  buildings belong to the neutral house, since the original skips human-house buildings
  placed by a skirmish map (S-0009). Give each a target hostile to its own house.
  Order sustained fire where orders apply, and let the neutral building acquire its target.
  Record projectile creation ticks by
  tick-labelled video, counting shots until no shot occurs for several full delays, before
  the vehicle's reload can finish. A building refills (claim 10), so for a building count the
  shots of its first group instead: its shots up to the first one that is followed by an
  interval longer than 1 tick. Run each case once for each creation method and kind.
  For the aircraft starting count, compare map-placed and produced aircraft of a test type
  with the same field pairs, using its ammunition pips before takeoff or servicing; if pips
  cannot distinguish an overfull count, report that observation as inconclusive rather than
  interpreting a full display as a cap.
- Expected: absent or explicit −1 with maximum 4 gives four shots, and a first group of four for
  the building. Initial 0 starts empty: infantry make no shot; the vehicle regains one round in
  its first turn (claim 11's never-started countdown), so makes one shot before the long reload
  wait; the building is refilled at the end of its first tick, so its first group has four
  shots. To tell that refill from a building that starts full, also record the building's
  ammunition when it is created, before its first tick's logic (expected 0), and at the end of
  its first tick (expected 4), with a target in range from tick 0. If the ammunition cannot be
  read before the first tick's logic, report the empty-start case as inconclusive: the shots
  alone cannot tell it from a full start. Initial 6 gives six, not four, and a first group of six
  for the building. Maximum 0
  with the initial field absent gives none. Maximum −1 with initial 0 gives none for the vehicle
  and the infantry unit; the building is refilled to unlimited ammunition and fires at every
  full delay, with no 1-tick interval. Map placement and production agree.
  Aircraft start with the same counts under Formula F6. Falsified by another count,
  an unlimited result for absent initial ammunition with a finite maximum (for the building:
  no 1-tick interval), a cap at four
  in the six-round case, or a difference between creation methods. An aircraft count not
  distinguishable through pips remains an unconfirmed part of the proposed measurement.
- Refill-exception part (claim 10): a test rules file gives two building types that cannot be
  garrisoned the weapon of the main part, burst count 1, and maximum and initial ammunition 2. One
  type has the hospital flag and the other the armory flag. A control type is the same with
  neither flag. Each building has a durable stationary enemy target in range. Use speed index 2,
  no engine add-on, the measurement harness's in-game helper off and no mods (the named test files
  excepted), and record shot ticks and each building's ammunition at the end of every tick, for 3
  runs of each type. Expected: each flagged building fires twice, 1 tick apart, holds 0 rounds at
  the end of the tick of its second shot and at the end of every later tick, and fires no third
  shot. The control fires groups of two without end. Falsified if a flagged building fires a third
  shot or holds a nonzero count after its second shot, or if the control stops. A flagged building
  that never takes the target leaves its run inconclusive.
- Below-minus-one part: test rules file, an infantry unit with maximum and initial ammunition −2
  (both keys set), firing at a durable target for 20 shots. Expected: it never answers *out of
  ammunition*, fires every full delay as an unlimited unit does, and shows no ammunition change.
  Falsified if it stops firing, or its ammunition changes.

**F10. Reloading.**
- Setup: test rules file: a vehicle with maximum and initial ammunition 4 (both keys set), reload
  field 50, reload increment 10, pip wrap 2 and empty-reload field 120. Parts: (a) fire all rounds, then wait. (b) Fire one round
  every 30 ticks. (c) As (a), with the mobile-fire flag off, while driving. (d) A type with the
  damage-reduces-readiness flag that is shot. (e) A type with the manual-reload flag, maximum
  and initial ammunition 4 (both keys set): it fires once. (f) As (a), with pip wrap −2.
- Observable: the ammunition pips over time.
- Expected: (a) the first round comes back 120 ticks after the last shot (*a* = 0 uses the
  empty-reload field), the second 50 ticks later (*a* = 1: *q* = 0), the third and fourth 60 ticks
  apart (*a* = 2 and 3: *q* = 1). (b) No round comes back while the vehicle fires every 30 ticks, for as long as it has rounds.
  (c) No progress while it drives. (d) Rounds are lost and the countdown restarts. (e) The round comes back in the first turn after the shot, not a reload delay later, since its countdown was never started (it held all 4 rounds, so nothing started it before the shot). (f) The same delays as (a): 120, 50, 60 and 60 ticks.
- Falsified if: any delay differs from Formula F3, a round comes back while the vehicle fires every 30
  ticks, the countdown advances while it drives (part c), the damaged vehicle in part (d) keeps
  all its rounds or does not restart its countdown, or the manual-reload vehicle of part (e) does
  not regain its round in the first turn after its shot (never regains it, or only after the
  reload delay), or part (f) differs from part (a).

- Warp part (engine-only contract test, not an oracle prediction): use the stationary vehicle
  of part (a), short of at least two rounds, with a reload countdown that expires during the
  warp. Exercise every boundary in S-0513 claim 51, including off-map retries and arrival
  effects, then a multi-tick arrival hold. Record ammunition, countdown deadline and stage
  each tick. Expect no ordinary reload grant or restart through arrival effects, an unchanged
  deadline despite elapsed time, and just one round on the first eligible hold turn after
  the hold-end test, with its normal reload restart. A grant in an excluded turn, a postponed
  deadline solely due to warp, or multiple catch-up rounds falsifies the contract. Repeat with
  a never-started countdown: its first grant likewise waits for an eligible hold turn.

**F11. Firing frames and sync.**
- Setup: test art file: a sprite vehicle with 4 firing frames and firing sync frame 3 for
  burst-counter value 0, a weapon with burst count 1 and a long rate-of-fire field.
- Observable: shot ticks and the firing-frame counter before each firing attempt, using
  tick-labelled capture and a read-only recorder.
- Expected: the first shot starts the counter at 7; the next shot comes in the vehicle's turn in
  which the counter equals 3 after four decrements, not at the rearm delay. Record that equality
  directly rather than presuming whether the first decrement occurs in the shot's turn. The long rearm delay for that
  sync shot outlasts the counter, so the next shot follows that delay under either open
  interpretation of the post-sync gate.
- Falsified if: the second shot follows the rearm delay rather than the sync opportunity,
  or its tick differs from the stated counter-equality tick.
- Sync-selection controls: named test art and rules give a burst-2 vehicle different sync
  values for counter values 0 and 1; record the counter, chosen value and shot ticks with
  a read-only recorder. Repeat with both sync fields absent, then with burst count 3.
  Expect only counter residues 0 and 1 to select their corresponding sync values; residue
  2 and absent fields add no firing gate. The first shot starts at 2*F* − 1, and a sync
  shot does not restart that counter. A different selection, restart on a sync shot or
  gate from an absent field falsifies the corresponding rule. Record the first-decrement
  boundary separately to resolve the open question, not as a presumed pass condition.
- Beyond-the-start part: test art file, the same vehicle with firing sync frame 8 (2*F*) for
  burst-counter value 0, and a rate-of-fire field of 2. Expected, under the literal gate of
  claim 13: after a shot the counter runs from 7 to idle, the primary weapon answers *rearming*
  throughout, and the next shot comes only after the counter is idle, so intervals are at least 8
  ticks although *D* is about 2. Falsified if intervals of about *D* ticks appear while the counter
  runs (the gate does not hold the shot back at all).
- Tunnel part: test map with a tunnel pair; the vehicle of the main part makes a shot (counter
  7), and in the next tick is ordered through the tunnel (transit of *T* ticks), then ordered to
  attack again. Expected: the counter does not fall during the transit, so the next shot is made
  *T* ticks later than it would be without the tunnel. Falsified if the transit turns count
  (the shot comes as without the tunnel).

- Warp part (engine-only contract test, not an oracle prediction): arrange a running
  firing-frame counter on the main-part vehicle when warp begins. Exercise every boundary
  in S-0513 claim 51, including off-map retries, arrival effects and a multi-tick arrival hold.
  Record stage, counter and the rearm deadline each tick. Expect no counter advancement through
  arrival effects, but no extension of the rearm deadline. On hold turns expect only the normal
  single advancement after the hold-end test, without catch-up decrements; materialising still
  prevents firing. Any contrary counter change, warp-only deadline extension or shot while
  materialising falsifies the contract. Original continuation remains unresolved; S-0513
  scenario SW23 part g proposes the separate oracle comparison.

**F12. Animation-delayed building.**
- Setup: a stock building with the animation-delayed-fire flag (or a test art file giving it a
  delayed-fire field of 10) attacking a target.
- Observable: tick of the attack decision (turret stops on target) and tick of the shot.
- Expected: the shot comes max(*d* − 1, 0) ticks after the turn that started the animation.
- Falsified if: it comes at another tick.
- Pending and dropped parts: (a) while the shot is pending, the building keeps its target and turns
  to it, and no second animation starts. (b) The target is moved out of range, or killed, between
  the start and the shot (a test unit ordered away, or a second attacker). Expected: no shot is
  made, the delayed shot is no longer pending, and no rearm countdown has started, so the
  building's next *can fire* starts a new animation at once. Falsified if a shot is made at the
  old target, a countdown runs after the dropped shot, or the building stays pending.

**F13. Charge turret.**
- Setup: a stock charge-turret vehicle firing repeatedly.
- Observable: the turret image index each tick, and which weapon (the projectile type) each shot uses.
- Expected: Formula F5 for the image; every shot uses the slot-0 weapon.
- Falsified if: the index does not follow the remaining countdown, or any shot uses a weapon other
  than the slot-0 weapon.

**F14. Draw order.**
- Setup: the recorder; a vehicle whose projectile takes a known number of draws (S-0306) firing
  alone on the map.
- Observable: the recorded raw values taken in the firing tick.
- Expected: the projectile's draws, then one R(0, 2) (or R(3, 5) inside a burst), then the
  shot animation's draws (S-0202, claim 10; S-0109).
- Falsified if: a second delay draw appears, the delay draw comes before the projectile's,
  or shot-animation draws precede the delay draw.
- Spray and particle part: a spray-attack type's first shot of a burst (test data), and a
  fire-particle weapon's first shot. Expected: the spray pick comes before the projectile's draws
  and its delay draw follows them. For the fire-particle weapon, creating the emitter takes no draw
  (its draws come in the system's own turns, S-0616 claims 10 to 18), and there is no delay
  draw at all (claim 5). Read the fire-particle shot under the no-draw part's emitter
  conditions (below), so that no turn of the new system draws in the firing tick. Falsified
  if either ordering differs, if a delay draw appears for the fire-particle shot, or if the
  spray pick appears for any shot other than the first of a burst.
  In M-0004 every attempt of a stock tank given the spray flag and a burst-3 cannon was reported
  with no projectile and no damage. Reading them as void shots is an inference.
  Each took the pick and no attributed delay draw. M-0004 records
  no burst position for these shots; by claim 15 a void shot leaves the burst counter
  unchanged, so each would count as the first of a burst. The place of the delay draw after the
  pick needs a spray type whose shots are made: use a spray-attack type with a direct-fire
  weapon, and read the shots that make a projectile.
- Refusal part: an attempt refused by the firing check while the countdown runs, in each of the
  ticks of the countdown. Expected: in those ticks the firer's draws are only those of its
  behaviour processing (S-0215 claim 23). Falsified if the firer takes more draws in those ticks
  than its behaviour processings account for, as a draw per refused attempt would give. This part
  does not tell a firing-check draw apart from a behaviour-processing draw in the same tick, so it
  checks only for draws in excess and does not back claim 18's absence for a refused attempt.
- Void-shot part: in S-0306's scenario P5 run d the recorder shows the projectile's draws, if the
  projectile takes any, and no delay draw; the countdown, burst counter and ammunition are
  unchanged. Falsified if a delay draw is taken or the countdown starts.
- No-draw part: three pairs of runs. In each pair the test run and its control are the same
  game, with the same seed, placement and orders, and differ only in test data that changes the
  delay kind claim 2 gives the shot read. Nothing before that shot depends on that data, so
  both runs log the same draws up to the shot, including every behaviour-processing draw
  (S-0215). The shot read is each run's first, made before any impact.
  - Run 1: a vehicle whose weapon has burst count 2 or more and a burst-delay field for its
    first shot (test data). Its control lacks that field, so its first shot takes the in-burst
    delay and its R(3, 5) draw.
  - Run 2: a building defence whose single-shot ordinary weapon is not an effect weapon,
    holding 2 rounds before the shot. Its control holds 1 round, so the shot takes the full
    delay and its R(0, 2) draw.
  - Run 3: a fire-particle weapon (claim 5). Its control is the same weapon without the
    fire-particle flag, so the shot takes the full delay and its R(0, 2) draw. The new system
    must take no draw in the firing tick: give its system type a spawn interval longer than
    the run (test data) and place the target straight ahead of a firer that does not turn in
    that tick, so that no emission and no flame sweep falls in it (S-0616 claims 10 and 11).
    The system has no particle yet, so no update draw is taken. If the firer turns in the firing
    tick, the pair is inconclusive.
  - Observable: the draws of the firing tick in both runs, with their ranges, and the countdown
    after the shot.
  - Compared: the draws of the firing tick up to and including the shot's own draws (the
    projectile's draws, the delay draw and the shot animation's), by range and order, not by raw
    result. Once a draw is missing, every later draw in the tick takes the stream one step
    earlier, so later raw results, and the number of later draws whose count depends on an
    earlier result, may differ: draws after the shot's own are not compared.
  - Expected: within the compared draws, the test run's are the control's with exactly one draw
    removed, the control's delay draw of range 3 to 5 or 0 to 2, at its place after the
    projectile's draws. The countdown is the burst-delay field, 1 tick or the bare rate.
  - Falsified if the test run's compared draws include the control's delay draw, or differ from
    the control's in any other range or place. A difference before the shot means the runs are
    not the same game, and the pair is rerun.

**F15. The named reasons of the rank table.** Each part checks that a shot is not made while
one reason holds, with every other reason absent (the rearm countdown over, the target in range
and in the aiming tolerance, ammunition above 0), and that the same firer shoots when the reason
is absent (the control).
- Observable: in each part, whether and in which tick the firer makes a shot (projectile creation
  or the target's health drops); in (d)(ii), whether a spawned aircraft is launched; in (e), also
  the vehicle's behaviour.
- (a) Death frames (rank 1): test art file giving a sprite vehicle a death frame count of 20 or
  more and a start frame. The vehicle is ready with a target in range; a strong enemy hit destroys
  it. Expected: no shot is made from the tick its death counter starts. Falsified if a shot is
  made while the death frames play. If the vehicle takes no further turns once destroyed, the
  record says so and the part stays open.
- (b) Melee link (rank 3): S-0214's scenario part C, second case (an enemy shooter with a
  link-flag weapon links itself to an own vehicle), with a second enemy in range of the vehicle's
  ready weapon. Expected: the vehicle makes no shot while the link holds; the same vehicle fires
  at the second enemy before the link forms. Falsified if it fires while linked.
- (c) Tether (rank 12): an armed vehicle that docks at a refinery (a stock armed harvester if the
  data has one, else test data giving a harvester a weapon), with an enemy in range while it
  unloads. Expected: no shot from the tick the tether begins until it ends. Falsified if a shot is
  made while it is tethered.
- (d) Early *rearming* (rank 4): (i) use the armed test vehicle and distance/delay controls of
  S-0207 scenario D22 runs 1 and 5, teleporting by its own locomotor, with a stationary enemy
  target in range at the landing point. Record the jump tick, the end of phasing in and shot
  ticks. Expected: no shot while materialising, which ends exactly as S-0207 claim 20(f) and
  Formula F9 specify, including the next-tick end for a nonpositive delay. Once that state ends,
  it adds no firing restriction of its own; other firing checks and the timing of attempts still
  apply. Falsified by a shot during phasing in or by materialising ending at a different tick.
  (ii) a spawner weapon none of whose spawned aircraft is
  alive: no launch until one is alive; (iii) a temporal weapon whose beam holds the target is
  covered by S-0310's scenario T6. Falsified if a shot or launch is made while the condition
  holds.
- (e) Seeker type (rank 9): a seeker type (stock if the data has one, else test data) with its
  target well inside the weapon's range and everything else passing. Expected: the attempt is
  refused although the target is in range, so the vehicle does not make an ordinary shot and
  shows the behaviour S-0215 gives to *out of range*. Falsified if it makes an ordinary shot.

- (f) Hidden targets (rank 4): a named test map, speed index 2, no engine add-on, the measurement harness's in-game helper off and no mods (the named test files of a scenario and the purpose-made test maps excepted),
  one allied and one enemy bystander computer player outside the observed area under the
  glossary's void rule. Its override sections supply a self-cloaking target and an armed vehicle
  with two ordinary weapons whose signed damage-plus-ambient sums give positive, zero and
  negative mean weapon damage in separate runs (S-0308 claim 15). Keep the weapon being
  attempted unchanged while changing only the other weapon's contribution. Attempt forced
  attacks on an own hidden target, and attacks on allied and enemy targets, with and without
  the firer's own sensor coverage; repeat each case three times. Where target selection is
  unavailable, assign the target before cloaking and record target loss separately rather than
  treating it as a firing refusal.
  Expected: without sensing, no shot at the hidden own or allied target when the mean is
  positive, and no shot at a hidden enemy whatever its sign. A nonpositive mean permits the
  own or allied attempt if no other restriction holds. Sensing removes this refusal. Record
  shot ticks, target retention and health changes by tick-labelled capture and a read-only
  recorder. Falsified by a shot in a refused case or by a refusal that follows the unchanged
  attempted weapon rather than the changed mean. A lost target makes that trial inconclusive.
  Repeat with a building cloaking inside its own field: the refusal starts at fade level 11,
  not only at level 15 (S-0516 claim 18).

**F15a. Erasure inactivity boundary (claim 9; S-0514 claim 17).**
- Setup: use S-0514 scenario E11 part a's test map with moving/firing infantry,
  vehicles, landed aircraft and armed buildings, at speed index 2, no
  engine add-on, the measurement harness's in-game helper off and no mods except that
  scenario's named test data. Keep a bystander computer
  player outside the observed area under the glossary's void rule. Give each victim a
  durable enemy target in range and establish ordinary firing before the lock.
- Procedure: attach an erasing beam through normal attack orders and maintain it for
  at least two of the victim's ordinary full firing delays. Repeat three times per kind.
- Observable: tick-labelled beam attachment and ordinary projectile or weapon-effect
  creation by the victim; exclude effects launched before attachment.
- Expected: ordinary firing stops during the attached lock. Any new ordinary shot by
  the victim during that interval falsifies the inactivity rule. A lock that releases
  early or a victim that was not firing in the control is inconclusive. This observation
  establishes no answer label or precedence for a check that might not be attempted.

**F16. Delays at the edges.**
- Setup: test rules file; a single-shot vehicle weapon, a target of high strength in range.
- Observable: shot ticks and the recorder's draw results for R(0, 2).
- (a) Rate-of-fire field 0, and −5. Expected: with 0, the interval is the draw *u* of R(0, 2),
  but never below 1 tick since the vehicle attempts once per tick (intervals 1, 1 and 2 for draws
  0, 1 and 2); with −5, a shot in every tick the vehicle can attempt. Falsified if a gap of 2 ticks or
  more appears with −5, or the 0 run's intervals do not follow the draws.
- (b) A bunkered vehicle (S-0206) with a bunker factor of 0.5 and of 0: Formula F1's *D*₅ is double
  *D*₄ for 0.5 (the delay lengthens) and equals *D*₄ for 0 (the division is skipped). Falsified
  if either run differs from Formula F1.
- (c) Isolate negative factors in separate runs: house factor −1, veteran factor −1
  with the veteran ability active, and bunker factor −1 on a bunkered vehicle. Set all
  other applicable factors to 1 and the rate-of-fire field to 20. Record the delay and
  shot ticks over 20 shots. Expected: Formula F1 produces a negative delay, with no
  countdown-imposed gap; the vehicle can shoot in each eligible turn. A positive
  countdown or a missing otherwise eligible next-turn shot falsifies the rule.
- (d) Set burst count 2 and the vehicle's first burst-delay field to −2, leaving other
  gates satisfied. Expected: no delay draw after each first shot, and the second shot
  in the next eligible turn; a running positive countdown or a delay draw there
  falsifies the rule. Record importer warnings separately on our engine: each negative
  input in parts a, c and d produces the warning named in Edge cases.
- (e) Give a garrisoned building two living occupants, an ordinary weapon, unlimited
  ammunition and occupy factor 0.5 in named test rules. Record 20 shot delays and draws.
  Expected: Formula F1 doubles the post-occupant-division delay at its occupy division,
  without an importer warning on our engine. A different computed delay falsifies the
  arithmetic; the original is not expected to produce our importer warnings.
- (f) Invalid burst (engine only): named test rules set burst count first to 0, then −2.
  For each run record the importer warning and 20 otherwise eligible shot ticks and delay
  draws. Expect one warning naming the weapon and single-shot behaviour with the full
  delay after every shot, as for burst count 1. A missing warning, failure to run or any
  in-burst delay falsifies the safe replacement. Do not run the failing original input.
- For every engine-only warning check above, a missing required warning or a warning where
  the stated rule requires none falsifies the corresponding import rule.

**F17. Upgrade pair with an empty secondary and animation delay (Q-0798).**
- Setup: tiny skirmish test map, speed index 2, 800 × 600, no engine add-on, the measurement harness's in-game helper off and no mods (the named test files of a scenario and the purpose-made test maps excepted),
  one bystander computer player outside the test area under the glossary's void rule.
  Named map overrides supply a non-SAM, ungarrisoned host and an upgrade in position 1
  with a primary ordinary projectile weapon of burst count 2 and no secondary. Remove
  every host and upgrade secondary, including elite replacements, and keep ammunition
  unlimited. Use a durable stationary enemy target in range and correct aim. Set the
  host's animation-delay field to 20 ticks; compare its animation-delay flag off and on.
  Add a passing control with a distinct, single-shot secondary and a failing-gate control
  whose upgrade primary burst count is 1, with no secondary.
- Procedure: install the upgrade through normal placement and order sustained attack.
  Run each fixture three times. With tick-labelled capture and a read-only recorder,
  record projectile creation, burst counter, rearm delay, ammunition, delayed-shot pending
  state and random-stream draws at each attack processing. Stop before other objects
  affect the target. No injection or state writes are permitted.
- Expected: with the passing gate and empty secondary, each processing that gets *can fire*
  makes only the primary shot. Its first shot leaves burst counter 1 and one in-burst
  delay draw; its next shot finishes that burst and gets the full delay. There is no
  extra burst step, delay draw, countdown replacement or ammunition change from the
  missing secondary. Enabling animation delay neither postpones these shots nor starts
  a pending shot. With the populated secondary both attempts occur in the same processing,
  not 19 ticks later. With the failing gate and original quirk setting neither shots nor
  a pending delay appear; on our engine with the gate quirk off, the failing control
  instead starts the ordinary delayed single shot. A no-upgrade ordinary control uses
  scenario F12's delay. The original cannot test our engine's quirk-off alternative.
- Falsified by any extra shot or state/draw change attributable to the empty secondary,
  an immediate-pair fixture becoming delayed, or a failed original gate starting a delay.
  Record unreachable fixtures as inconclusive rather than inferred confirmations.

**F18. Non-shot charge-arrival delay (claim 19).**
- Setup and procedure: use S-0519 scenario T13's named test map and override variants,
  speed index 2, no engine add-on, the measurement harness's in-game helper off and no mods (the named test files of a scenario and the purpose-made test maps excepted), a bystander computer player kept outside the
  observed area under the glossary's void rule. Run each variant three times. Compare
  successful, protected and duplicate-countdown arrivals with an ordinary second weapon,
  first at *k* = 0 with burst count 1, then with a retained *k* at least its burst count.
  Include its missing-weapon, sonic and particle-effect absent/alive controls. Void a
  retained-counter trial if target clearing resets *k* before arrival.
- Observable: arrival tick, rearm time remaining immediately after arrival, burst counter,
  ammunition and random-stream draws, followed by the scatter draws. Do not infer the
  delay solely from the next shot, since other firing gates can postpone it.
- Expected: claim 19's table, unchanged burst counter and ammunition, at most one delay
  draw before scatter. In particular, ordinary *k* = 0 with burst count 1 gives R(3, 5),
  not R(0, 2). A missing particle effect does not grant the bare rate.
- Berserk comparison: repeat a reachable arrival while the walker is berserk, with named
  test data keeping it berserk through arrival and a delay distinguishable from its halved
  value. Expect the unchanged table result, not half. If normal orders and stock
  berserk application cannot establish such an arrival, record that variant inconclusive.
  Do not force the state by writing to the game.
- Falsified by a counter advance or wrap caused by the query, a different delay or delay
  draw, a draw after rather than before scatter, or berserk halving at a reached arrival.

**F19. Shot animations (claims 20 to 23; Q-1523).**
- Setup: a purpose-made flat test map at speed index 2, no engine add-on, the measurement harness's in-game helper off and no mods (the named
  test files of this scenario and the purpose-made test map excepted), a bystander computer player
  kept outside the observed area. Test art gives sixteen named test animations, T0 to T15, each a
  distinct digit picture with a long fixed rate, one pass, no random rate and no start effects.
  Test weapons: W8 with the list T0 … T7, W16 with T0 … T15, W1 with T0 alone, W0 with an empty
  list, WG with the list T0 … T7 and occupant animation T9, WG0 with the list T0 … T7 and no
  occupant animation, WP with an empty list and open-topped animation T10, and WP8 with the list
  T0 … T7 and open-topped animation T10. No test weapon has a report sound or an effect flag
  (claim 5), and each fires a straight, draw-free test projectile. The random-state recorder
  runs throughout.
- Procedure and observable, per part: the shot tick, which test picture appears, its screen
  position relative to the firer and the target, whether it moves with the firer, and the raw
  stream values taken in the shot tick.
  - (a) A turreted test vehicle armed with W8 holds still and fires at targets placed so that its
    turret faces each of the eight 8-directions in turn, one shot each. Repeat with a vehicle
    without the turret flag, which turns its body. Expected: claim 20's table, entry
    (*d* + 1) mod 8. Falsified if entry *d* appears, or any entry other than the table's.
  - (b) As (a) with an infantry firer armed with W8, and with an aircraft. Expected: the same
    table by body facing (infantry) and turret facing (aircraft).
  - (c) As (a) with W16 and W1, at every 8-direction. Expected: T0 every time. Falsified by any
    other picture.
  - (d) The W8 vehicle fires once while driving, with a test animation lasting at least 60 ticks.
    Expected: the picture moves with the vehicle and vanishes at once if the vehicle is destroyed
    before it ends. Repeat with an aircraft in flight: the picture follows it and is drawn
    behind objects of the air and top layers (S-0102 claim 3).
  - (e) A garrisoned test building whose occupants' garrison weapon is WG, then one whose
    occupants' garrison weapon is WG0. Expected: T9 at each shot's muzzle, then no picture.
    Falsified by any T0 … T7.
  - (f) A turreted test building with a sprite turret armed with W8 fires at a stationary target
    whose direction from the building's aiming point (S-0310 claim 4; S-0204's direction) is
    about 3584 facing units, the middle of the window of claim 21. Count only shots made after
    the turret has stopped turning towards that target (S-0206), and record the turret image shown
    as a cross-check of its facing. Expected: entry 2 (32-step rounding gives *d* = 1). Falsified
    if entry 1 appears (a single rounding). Repeat at the eight 8-directions for the table.
  - (g) The building of (f) fires while the screen shows the picture: it stays in place and is
    not attached. Its palette part: a test animation with the alternative-palette art flag shows
    the single-shade house-colour palette (S-0103 row 4), one without it the anim palette.
  - (h) An infantry passenger of an open-topped transport armed with WP, then WP8. Expected:
    T10 at the transport's muzzle, moving with the transport; then entries T0 … T7 by the
    passenger's own facing, never T10.
  - (i) As (f) with a test building that is not turreted and one with a voxel turret, both
    armed with W8, at targets in each 8-direction from their aiming points and at about 3584.
    Expected: the entry of the target direction's own 8-direction (entry 1 at about 3584).
  - (j) Repeat the vehicle part with a burst, F11's reachable sync shot and F12's delayed
    building shot; use distinct lists for two gattling stages. Expect a new choice at each
    actual shot from the weapon and facing current then. Give W0 an assault animation only:
    expect no shot animation or placeholder. Repeat the excluded firing cases using the
    setups of F14's refused/void parts, S-0608 scenario D1, S-0522 scenario P7, S-0506
    scenario R2, S-0615 scenario DL1 and S-0519 scenario T12, with a named list added to the
    attempted weapon. Expect no shot animation for those excluded attempts or firings.
- For every part, a missing expected picture, an extra or different picture, a different
  attachment/removal result, layer, palette or position falsifies the corresponding rule.
  The stated expected image must be identifiable in a control; occluded or unreachable
  cases are inconclusive. Also record creation settings with a read-only recorder: no
  initial loop delay, repeat factor 1 and no reverse request. A differing setting falsifies
  claim 22; repeat with the type's reverse flag enabled to check its independent effect.
- Draws in every part: no raw value is taken between the shot's delay draw and the test
  animation's creation. Falsified by an extra draw that depends on the list length.

**F20. Target line (claim 24; Q-1580).**
- Setup: a purpose-made test map at speed index 2, no engine add-on, the measurement harness's
  in-game helper off and no mods except this scenario's named map overrides and test art. Use flat ground except for part l,
  neutral test lighting and a hi-colour display path. Test vehicle TV has the target-laser flag,
  a turret and an ordinary draw-free direct-fire weapon with a rate-of-fire field of 40 ticks
  and burst count 1. TV2 differs only in its rate-of-fire field of 5 ticks. These are invented
  test values. Targets are a durable stationary building, a vehicle driving straight and a weak
  vehicle one shot destroys. A bystander computer player stays outside the observed area under
  the glossary's void rule, except for the explicitly named firer of part f.
- Record every drawn frame with its tick number, using tick-labelled screen capture and the
  random-state recorder. A read-only recorder supplies current targets, ownership, positions,
  facings and shot ticks. Run each part three times. Compare a flag-off clone: no line and no
  line-related random draw or change to shot timing, damage or ammunition are expected. Any
  such draw or gameplay difference, or a line with the flag off, falsifies that rule. Void a
  run if an unnamed object affects the test or a required state is not reached normally.
- Procedure and observable, per part:
  - (a) TV, owned by the local player, fires once at the stationary building. Record the first
    and last line ticks, its pixels and both end marks. Expect no line before that first shot,
    then a line in exactly ticks *t*₀ … *t*₀ + 14. Expect a one-pixel-wide dotted line, alternating
    drawn and undrawn pixels along its longer screen axis, swapping that set each tick. Each end
    gets a solid 3 × 3 square whose bottom-right pixel is the projected end point. Apart from the
    dot swap, there is no fade or other change. An early/missing line, a line at *t*₀ + 15,
    another width or pattern, a missing swap, different square geometry or a fade falsifies
    the corresponding rule. Do not presume the unresolved absolute dot phase.
  - (b) As (a), but drive TV or turn its turret during the 15 ticks without another shot,
    retaining a target. Compute the projected turret pivot from each frame's recorded pose
    using S-0313 claim 1 with zero muzzle displacement and S-0101 claims 1 to 5.
    Expect the start mark there in every frame. A start left at the firing position or at any
    different pixel falsifies the rule. Include a named nonzero turret displacement to separate
    the pivot from the vehicle's reference point.
  - (c) Fire at the moving vehicle. Compute its current aim point using S-0306 claim 3 and
    project it with S-0101 claims 1 to 5. Expect the end there in every frame, including changes
    of distance or speed. An end fixed at the shot's aim point, attached to the unled reference
    point when the predicted lead is nonzero, or differing from the predicted pixel falsifies it.
  - (d) Fire at the weak vehicle and record when its destruction clears TV's target. Expect
    the line to disappear in the first drawn frame with no target. In a separate run, retarget
    TV before it can fire again: expect the end to jump to the new current aim point without
    extending expiry beyond *t*₀ + 14. Clear and restore a target within that interval as a third
    run: expect no line while targetless and a line only for the remaining ticks after restoration.
    Persistence without a target, failure to follow the new target, or restarted expiry without
    a shot falsifies the corresponding rule.
  - (e) Let TV2 fire repeatedly, then stop further shots while retaining its target (the named
    test clone may have a finite magazine). Expect one line, never multiple lines, present
    through *t*last + 14 and absent at *t*last + 15. Duplicate lines, expiry from the first
    rather than the last shot, or any other expiry tick falsifies the rule.
  - (f) A named computer-owned TV fires at a local-player building in view. Expect no line.
    Separately let local-player infantry, aircraft and building test types with the flag fire
    ordinary shots: expect no line. Any line from these firers falsifies the relevant owner
    or vehicle-only restriction. Confirm the local-player TV control does show one.
  - (g) After a shot, scroll until TV is outside the drawn part of the view while its target
    stays visible. Expect neither line nor target-end square. For the hidden case, first
    transfer TV to the enemy as in part k, then cloak it without viewer sensor coverage.
    An own cloaked vehicle is not a hidden-from-owner control. Also test it off-map inside a transport, using
    named test data to finish those transitions within the timer. Re-show it before expiry:
    expect only the timer's remaining ticks, not a restart. Any mark while TV is not drawn,
    or a visibility-driven timer restart, falsifies the corresponding rule.
  - (h) Read line pixels with S-0103's pixel-reading method. Repeat with different house colours,
    lighting and coloured light. Expect the general palette's index 8 in neutral row 26 of 53,
    unchanged across those runs. Any different pixel colour falsifies the prediction. If the
    installed palette makes candidate rows indistinguishable, use a named loose test palette
    with a distinguishable index-8 colour and shade results; otherwise report row selection
    inconclusive rather than inferring it from identical colours.
  - (i) Drive a test vehicle across the line, once earlier and once later than TV in the
    ground-layer drawing order. Arrange visible status extras across the line too. Expect
    line pixels over the earlier vehicle and over TV's own opaque pixels, but under the later
    vehicle and the status extras. Choose differing scene depths at the earlier-vehicle overlap
    so a depth-tested line would be hidden. Any contrary covering order or depth-dependent
    disappearance falsifies the relevant prediction. This tests visible order and absence of
    a depth test, not the separate unmeasured absence of a depth write.
  - (j) Trigger controls: give TV a burst, both weapon slots, no chosen shot animation, and
    F11's reachable sync shot in separate named fixtures. Expect each admitted ordinary shot
    to start or restart the same 15-tick line. Use F14's refused and void attempts, S-0608
    scenario D1's drain attempt, S-0506 scenario R2's spawner shot, S-0615 scenario DL1's disc
    laser and S-0519 scenario T12's surviving-suicide fixture as exclusion controls, each with
    the flag enabled. Expect none to start or restart the timer; record its deadline directly
    if no line could otherwise be seen. S-0522 scenario P7 supplies the aircraft drop control.
    Any contrary timer start, missing restart or changed deadline falsifies the corresponding
    trigger rule. An excluded attempt with no prior running timer must leave it expired.
  - (k) Change TV's owner through normal takeover after a local-player shot, retaining a
    current target and preventing further shots. Expect the original local view's line to
    continue until its unchanged deadline. Separately take over a computer-owned TV after
    its shot: expect no line merely because it now belongs to the local player. Early loss
    or a new timer due only to takeover falsifies the shot-time owner test. Record loss of
    the target separately and void that comparison if a target cannot be retained or restored
    before expiry; target loss alone cannot test ownership.
  - (l) Projection, clipping and hidden targets: repeat a shot with firer and target at
    differing terrain heights, and on a slope. The exact height-lift result of S-0101 claim 3
    is an engine-only departure check, not an oracle prediction. On the original, compare
    the endpoints with that run's shared object-position projection; do not interpret its
    known terrain misalignment as a target-line-specific error. On our engine compute pixels
    as in parts b and c. Keep TV drawn while scrolling each end mark and the line partly beyond every
    tactical-view edge. Expect only their intersection with the tactical view, with unchanged
    projected endpoints, not clamped endpoints. In a separate run cloak the target after the
    shot, retaining it as TV's current target without another shot. Expect its line to remain
    despite the target's invisibility. A height lift differing from the applicable engine's
    shared projection, drawing outside the view,
    endpoint clamping, or disappearance caused solely by target invisibility falsifies the
    corresponding rule. Target clearing makes the invisibility comparison inconclusive.

**F21. Delayed building shot against a target that crosses a shore (claims 9 and 14; Q-1770).**
- Setup: tiny skirmish test map, speed index 2, 800 × 600, no engine add-on, the measurement
  harness's in-game helper off and no mods (the named test files of a scenario and the
  purpose-made test maps excepted), one bystander computer player outside the test area under
  the glossary's void rule. A straight shore line
  of clear land and water cells, no bridge in range. A test copy of a non-SAM, ungarrisoned,
  powered building type with no upgrades, defined only in the scenario's named test rules file,
  with its image defined only in a named test art file (both supplied through the file system,
  no stock file changed), and owned by the human-controlled
  house, which builds it beside the shore during the run (the starting units of S-0009 include
  no building). Give its image, in that art file, the animation-delayed-fire flag and a delay
  field of 30 ticks (Import mapping), and give the type one ordinary
  projectile weapon in slot 0. That weapon has burst count 1, a non-anti-air projectile, a range
  covering the whole path below and a rate-of-fire field longer than a run. An enemy amphibious
  vehicle of a second computer house, durable enough to survive one shot, is moved across the
  shore by the test map's own scripted move order. Parts:
  - a: land-targeting mode 1, naval-targeting mode 0; the vehicle starts on water and crosses
    onto clear land.
  - b: land-targeting mode 0, naval-targeting mode 6; the vehicle starts on clear land and
    crosses onto water.
  - c: control for a and b: the same fixtures, with the vehicle staying on its starting side.
  - d: land-targeting mode 2 and naval-targeting mode 5, with two ordinary weapons (no row 1
    to 12 of S-0308 table P applies) that make distinguishable projectiles, have equal range and
    have non-zero versus values against the vehicle's armour. S-0308 table P then gives slot 0
    against the vehicle on water (row 17, table N mode 5) and slot 1 against it on land
    (row 18), and S-0306 claim 23's water row refuses neither, since table N mode 5 gives a
    weapon. The vehicle crosses from water onto land.
  - e: the part a building, but with land-targeting mode 0, against an enemy jumping infantry
    unit standing on clear land. The test map's own scripted move order sends that unit to a
    clear land cell several cells away during the delay. A grounded unit with jumpjet movement
    that has a movement commitment takes off (S-0502 claim 5), so it lifts off with no state
    write. Its type, in the named test rules file, has jumpjet movement with a cruise height of
    more than 208 leptons, so that it climbs past 208 leptons above the surface instead of levelling
    off exactly there; any such height serves. In this part the crossing is
    the first tick at which the unit's height above the surface is 208 leptons or more (S-0102
    claim 3, S-0307 Terms).
  - f: the part c fixtures, with a delay field of 150 ticks in this part, and a second enemy
    vehicle of the same type and house standing still on the vehicle's side of the shore,
    in range, in the cell just beyond the first vehicle on the line from the building, so that
    the building faces both alike. Neither vehicle moves.
- Procedure: order the building to attack the vehicle (or the infantry unit), and time the
  scripted move so that the crossing (in part e, the tick it reaches 208 leptons) falls inside the delay. Vary the
  move's start over 10 runs per part so that the crossing falls at different points of the
  delay. With tick-labelled capture and a read-only recorder, record per tick the building's
  target, its facing, its current behaviour, its delayed-shot pending state and remaining delay, its rearm
  countdown, every projectile it creates with its weapon slot, and the target's cell, land type
  and height above the surface in leptons. No injection or state writes are permitted. In part
  f, order the building to attack the first vehicle; once its delayed shot is pending, order it
  to attack the second vehicle, at a different point of the delay in each of 5 runs.
- Expected: in parts a, b and e, the building keeps its target and its *attack* behaviour in
  every tick of the delay, including the tick of the crossing. In the tick that ends the delay
  it creates no projectile when the target then stands on the refused side (or, in part e, is
  208 leptons or more above the surface). In part e, a run in which the unit is still below
  208 leptons when the delay ends is a run whose crossing comes after the end of the delay:
  neither row refuses the unit then, and the shot is made.
  Where the check refuses the shot, no rearm countdown starts, an attack processing falls in the
  tick that ends the delay (claim 14's confidence note) and keeps the target and *attack*, and at
  its first attack processing after that tick, 2 ticks later, the building has no target and is in *guard*. A run whose crossing comes after the
  end of the delay makes the shot. In part c every run makes the shot, and its projectile flies
  at the vehicle as an undelayed shot's would (S-0306). In part d the shot that the end of the
  delay makes is of slot 0, the slot chosen when the delay started (claim 14), not slot 1, which a
  fresh choice by S-0308 claim 14 would give against the vehicle now on land. In part f the
  building's target is the second vehicle from the order on, its delayed shot stays pending
  with its remaining delay counting on unchanged (the order starts no new delay), and the
  one projectile made at the end of the delay is aimed at the second vehicle.
- Falsified by a dropped target or a change of behaviour before the end of the delay, the
  target dropped or *guard* queued in the tick that ends the delay, a
  projectile created at the end of the delay while the target stands on the refused side (in
  part e, at 208 leptons or more above the surface), no projectile in part e when the unit is
  still below 208 leptons at the end of the delay, a shot in part c that differs from an undelayed shot, a slot-1 projectile in part d (the
  fresh choice), or, in part f, a projectile aimed at the first vehicle, no projectile at the end
  of the delay, or a delay that starts again at the order.
  Record as inconclusive a run of parts a, b and d whose crossing tick cannot be placed inside
  the delay, and a run of part e whose crossing tick (the tick the unit reaches 208 leptons)
  comes before the delay starts. A run of part e whose crossing comes after the end of the delay
  is the control above. Record as inconclusive also a run in which the building misses a turn
  (S-0203 claim 10) between the start of the delay and its first attack processing after the
  tick that ends the delay. Record as inconclusive a run of part f in which the building does not
  take the second vehicle as its target at the order, and a run in which a shot is expected at
  the end of the delay but the building's facing then does not point at the target (a *not
  facing* or *still turning* answer, S-0310).

**F22. A building whose temporal beam holds its target goes out of operation (claim 9; Q-2567).**
- Setup: tiny skirmish test map, speed index 2, 800 × 600, no engine add-on, the measurement
  harness's in-game helper off and no mods (the named test files of a scenario and the purpose-made
  test maps excepted), one bystander computer player outside the test area under the glossary's void
  rule. A test building type *T* of the scenario's named test rules file (no stock file changed),
  owned by the human-controlled house, which builds it during the run: not occupiable, not a pulse
  cannon, without the SAM flag, turret-snap, animation delay or upgrades, with the needs-power flag
  and a base drain above 0, and armed with one test temporal weapon (Terms) as its only weapon. That
  weapon's projectile detonates within a few ticks of the shot, and its damage field is small enough
  against the target's strength field that the target's erasure (S-0514 formula F3) lasts at least
  100 ticks after the lock. The human's house has power plants whose output covers *T*'s drain with
  a margin. In range of *T* stands an enemy vehicle *V* of a second computer house, of a weaponless
  test type, which the test map gives no order. Parts:
  - a: switched off. In a tick *w* after the lock (S-0514), at least 20 ticks before the erasure
    would complete, the human switches *T* off (S-0405 claim 10). *T* has no chargers.
  - b: short of power. A second test building *P* of the human's house, with the needs-power flag
    and a base drain such that *P*'s and *T*'s drains together exceed the house's output while *T*'s
    alone does not, is switched off by the human once built. After the lock the human switches *P*
    on. Here *w* is the first tick at whose end the house has low power (S-0405 claims 4 and 7),
    which must fall at least 20 ticks before the erasure would complete.
  - c: control. Parts a and b's fixtures with no switch order after the lock and *P* left switched
    off.
- Procedure: the human orders *T* to attack *V* without force-fire. Vary *w* over 10 runs per part
  a and b, so that it falls at both parities of *T*'s *attack* processing ticks and at different
  distances from the next check tick of S-0309 claim 29 (a tick whose number is a multiple of 16).
  With tick-labelled capture and a read-only recorder, record per tick, from the order to the end of
  the erasure or the run: whether *T*'s beam is attached to the target, *T*'s target and current
  behaviour just after each of its updates, the ticks of *T*'s *attack* processings and of its
  shots, whether *T* took its turn (S-0203 claim 10), *T*'s switch state, the house's low-power
  state at the end of the tick, and the target's presence. No injection or state writes are
  permitted.
- Expected (claim 9, *A building's own answers above a held temporal target*): call *u* the first
  of *T*'s updates in a tick after *w* that is an *attack* processing or falls in a check tick. In
  parts a and b, *T* holds the target and is in *attack* just after each of its updates from the
  lock to the last one before tick *w*. It drops the target at one of its updates from tick *w* to
  *u*, both included, that is an *attack* processing or falls in a check tick: just after that
  update it holds no target, its beam is still attached, and, when the update is an *attack*
  processing, it is in *guard*. In part c, *T* holds *V* and stays in *attack* just after every
  update from the lock until the erasure ends, its *attack* processings fall every other tick, it
  makes no second shot, and its beam stays attached.
- Falsified by, in parts a and b, *T* still holding the target just after *u*, a drop before tick
  *w* or at an update that is neither an *attack* processing nor in a check tick, *T* in *attack*
  just after a dropping update that is an *attack* processing, or a beam let go from tick *w* to
  *u*; in part c, by a drop, *guard*, a second shot, processings one tick apart or a beam let go
  before the erasure ends. A run in which the beam is not attached at the end of the tick before *w*
  (in part c, from the lock on), *T* misses a turn from tick *w* to *u*, *V* moves or is damaged by
  anything but *T*'s beam, or the erasure ends before *u*, judges nothing. What *T* does after the
  dropping update (its *guard* may take a target again) judges nothing.

## Changes for other specs

- **S-0306 (approved, revision 33), claim 23, "Rank of the water and land rows".** Replace "Where
  the *rearming* of a building's pending delayed shot (S-0301 claim 14) falls against the two
  rows is not stated, as S-0301 claim 9 leaves its rank unstated." with: "A building's pending
  delayed shot (S-0301 claim 14) gives *rearming* ahead of the two rows and of every other row
  of this claim; S-0301 claim 9 gives its rank." In "Consequences", after "drops it (S-0215
  claim 32) rather than waiting out its countdown", add: "unless its delayed shot is pending.
  It then keeps the target until the end of the delay; the end-of-delay check and the target
  handling after it follow S-0301 claim 14".
- **S-0308 (approved, revision 15), claim 14, "The weapon it would use is the one tested
  everywhere".** After "asked afresh each time" add: ", except the check and shot that end a
  building's animation-delayed shot, which use the slot chosen when the delay started (S-0301
  claim 14)". In its Confidence line, add: "medium, not high, for the check and shot that end
  a building's animation-delayed shot, until S-0301 scenario F21 part d runs". This spec's
  claim 14 states that rule, rated medium until scenario F21 part d, which tells the two
  readings apart, runs.
- **S-0215 (approved), claim 32.** Its table already keeps the target on *rearming* and drops it
  on *illegal*. Its sentence "one whose target leaves its weapon's range returns to *guard* at its
  next processing" can add: "unless its animation-delayed shot is pending, when the firing check
  answers *rearming* until the end of the delay (S-0301 claim 14)".
- **S-0215 (approved, revision 88), scenario B45, case (e) "Non-SAM ammunition exhaustion".** A
  building with one finite round is refilled in the tick of its shot (S-0301 claim 10), so the
  firing check never answers *out of ammunition* there. Replace "giving the building one finite
  ammunition round and a reload delay longer than the observation" with "giving the building a
  maximum ammunition of 0 and no initial-ammunition value, so that every attempt answers *out of
  ammunition* (S-0301 claim 10)", and "around each processing after the shot, through the first
  *out of ammunition* answer" with "around each processing from the first attack order, through
  the first *out of ammunition* answer". This changes what case (e) tests: an attack that is
  refused for lack of ammunition from its first attempt, not a building that runs out after a
  shot.
- **S-0616 (approved, revision 20), claims 1, 20 and 22 and scenario P1: an effect weapon's
  emitter by place and kind.** Where these sentences say "fire emitter", "spark emitter" or
  "railgun emitter" for a let-go or an end, they mean the place that holds the system, not the
  system's kind. S-0301 claim 5's place-and-kind rule is the complete rule, and Terms points to it:
  - (a) Marks. Each kind keeps its own marks of claim 22's first column: a spark system when *K*
    runs out (claim 15), or in its first turn when the spark-turns field is 0 or less (claim 20's
    fix), a railgun system after its line (claim 17), and a fire system by its own turn when its
    owner is no longer alive (claim 10). Every kind is also marked by a positive lifetime, claim
    1's trigger action and the owner's removal (claim 4). In addition, a let-go of the fire place
    marks whatever system the fire place holds, whatever its kind.
  - (b) Freeing. The fire place is freed by its let-go events or by its system leaving play. The
    spark and railgun places are freed only by their system leaving play.
  - (c) Creation. The creation sentences name the place too: the system made for the fire or spark
    place takes the shot's target object, and its target point is that object's position at the
    shot (the world origin when the shot has none). The system made for the railgun place takes no
    target object at its creation, and its target point is F14's end point. This holds whatever
    the system's kind. So a railgun-kind system held in the fire or spark place makes its line
    (claim 17) to that fire-or-spark target point, not to F14's end point. The sentences:
    claim 1's table, effect-weapon row, fourth column (its "fire and spark" part and its "railgun:
    the beam's end point (F14), fixed at the shot" part), claim 1's
    Effect weapons row text ("For fire and spark emitters the creator
    passes the shot's target object" and "For a railgun emitter it passes no target object") and
    F14's first sentence ("at the shot that makes a railgun emitter"). No wording changes: read
    "fire", "spark" and "railgun" there as the place.
  - Claim 1, Effect weapons row text: replace "A fire emitter is marked and let go when" with "The
    system held in the fire place, whatever its kind, is marked and let go when", and replace "A
    spark or railgun emitter is not let go by any of these; it is marked only by claim 4's owner
    removal, and otherwise ends by its own counter or line (claim 22)." with "A system held in the
    spark or railgun place is not let go by any of these, whatever its kind. It ends by the marks
    of its own kind, owner removal (claim 4), a positive lifetime or the trigger action (claim
    22)."
  - Claim 1, Confidence line: replace "the three events that let a fire emitter go" with "the three
    events that let the system held in the fire place go".
  - Claim 20, spark-turns bullet (the original's behaviour): replace "It ends only by its owner's
    removal (claim 4), a positive lifetime or the trigger action of claim 1. Neither a move order,
    nor a new target, nor the firer's death lets a spark emitter go (claim 1), so in the original
    the firer's spark place stays held" with "It ends only by its owner's removal (claim 4), a
    positive lifetime, the trigger action of claim 1 or, while it is held in the fire place, a
    let-go of that place (claim 1): the let-go marks it, and since it makes no burst it holds no
    particle and leaves play in its next turn (claim 22). Neither a move order, nor a new target,
    nor the firer's death lets a system held in the spark place go (claim 1), so in the original a
    spark place that holds such a system stays held". The engine fix in that bullet is unchanged:
    it marks every spark system of such a type in its first turn, whoever made it and whatever
    place holds it, so after that turn a let-go of the fire place finds the place free.
  - Claim 22's table, fire row, first column: replace "or lets the emitter go (claim 1)" with "or,
    while it is held in the fire place, when that place is let go (claim 1). Held in the spark or
    railgun place, it is not marked by its own turn while its owner is alive and on the map".
  - Claim 22's table, spark row, first column: replace "Marked when *K* runs out (claim 15) or the
    owner is removed" with "Marked when *K* runs out (claim 15), when the owner is removed or,
    while it is held in the fire place, when that place is let go (claim 1)".
  - Claim 22's table, railgun row, first column: replace "Marked after its line (claim 17) or when
    the owner is removed." with "Marked after its line (claim 17), when the owner is removed or,
    while it is held in the fire place, when that place is let go (claim 1)."
  - Claim 22, the sentence after the table: replace "but owner removal, a positive lifetime or a
    trigger mark could still end it" with "but owner removal, a positive lifetime, a trigger mark
    or, while it is held in the fire place, a let-go of that place could still end it".
  - Scenario P1, Falsified: replace "by a fire emitter let go at a tick other than claim 1 gives"
    with "by the system held in the fire place let go at a tick other than claim 1 gives".
  - (d) A marked spark system's bursts (Q-2246). Claim 15's burst does not depend on the mark, so
    the mark does not stop a spark system's bursts: while its *K* is above 0 it bursts as claim 15
    gives (S-0301 claim 5, *Moment of a retarget's let-go*). Terms, *Marked*: replace "A marked
    system makes no new particles and leaves play" with "A marked system makes no emission (claims
    9, 11 and 17), though the mark does not stop a spark system's bursts while its *K* is above 0
    (claim 15), and it leaves play". The system-states table's row for a marked system with
    particles remaining: replace "keeps taking turns, no emission" with "keeps taking turns and
    makes no emission, except that the mark does not stop a spark system's bursts while *K* is above
    0 (claim 15)". Scenario P1 part b, Expected: replace "each marked system makes no new particle"
    with "each marked system makes no emission, except that the mark does not stop the spark
    system's bursts while its *K* is above 0 (claim 15)". Scenario P1 part b, Falsified: replace "by
    a marked system that emits" with "by a marked smoke, fire or railgun system that emits". Level:
    medium, `measurable` (S-0301 scenario F4's unmatched part, third run, u9).
  - Levels. Each kind's own marks and the place's freeing when its system leaves play: as S-0616
    rates them (high: claims 10, 15, 17 and 22, claim 4 for removal, and claim 20's fix and the
    original's spark-turns behaviour); a fire-kind system in the spark or railgun place not marked
    by its own turn while its owner is alive and on the map: high. A move order frees neither the
    spark nor the railgun place: high. A retarget to a target not within reach frees neither of
    them: medium, resting on claim 1's let-go events (rated medium there). A let-go's mark of the
    system in the fire place, whatever its kind: medium, resting on claim 1's let-go events (rated
    medium there). The fire place's freeing at the let-go itself rather than when its marked system
    leaves play: medium. The original's let-go end of a spark system with a spark-turns field of 0
    or less held in the fire place: medium. The creation reading (c): high for a matched emitter, as
    S-0616 rates claim 1 and F14, and medium for a system whose kind does not match its place.
  - Backing. `measurable` (S-0301 scenario F4, unmatched part: u3 and u4 for the fire-kind system in
    the spark or railgun place, u5 and u6 for the spark and railgun places' freeing, the second
    run's u1 against u8 for a let-go's mark of a spark-kind system and its u3 and u4 for a move
    order in the spark and railgun places). `unmeasured` for a retarget in the spark and railgun
    places (no scenario here gives a firer in those places a retarget to a target not within reach),
    for a let-go's mark of a railgun-kind system (its own line marks it in its first turn, so a
    let-go's mark would show only before that turn, which no scenario here sets up), for the fire
    place's freeing at the let-go itself (no scenario here records whether the marked system still
    holds a particle at the firer's first attempt after the let-go, so both readings give the same
    shot ticks), for the spark-turns case in the fire place (no scenario here sets such a type in
    the fire place) and for the creation reading (c) when the kind does not match the place (no
    scenario here records the line or target point of a railgun-kind system held in the fire or
    spark place). Each kind's own marks, the freeing when the system leaves play and the creation
    reading for a matched emitter: as S-0616 backs them (its scenarios P1, P4, P8, P11, P12, P14 and
    P22). Claim 20's fix: a design decision (S-0616 claim 20).

- **S-0607 (approved, revision 37), claim 11, "Against its gripped unit".** Replace "so a
  gripped unit that is in the air never gets *illegal* or *impossible* from its lifter. Of S-0301
  claim 9's rank-4 answers it is preceded only by the materialising firer's *rearming* and by the
  refusal of a firer that is bound to a slave master. The lifter therefore keeps the gripped unit
  as its target through the pull: neither that check nor any other rule of this spec drops it."
  with: "so a gripped unit that is in the air never gets *illegal* or *impossible* from its lifter
  through those tests. Of S-0301 claim 9's rank-4 answers it is preceded only by the materialising
  firer's *rearming*, by the refusal of a firer that is bound to a slave master and, for an
  infantry lifter, by the answers that an infantry firer is asked first (S-0301 claim 9, *Grip
  refusals within rank 4*): the infantry healer's *illegal* (S-0310 claim 11), the keep-acting
  *illegal* against a vehicle that is melee-linked to an object other than the lifter, and any
  answer that S-0310's open question on dying and falling sequences settles as asked first. A
  lifter that gets none of those keeps the gripped unit as its target through the pull: neither
  that check nor any other rule of this spec drops it. An infantry lifter that gets one of those
  *illegal* answers against its gripped unit drops it as its target as S-0301 claim 9 gives for
  that firer, at S-0309 claim 29's check or through its firing step's reply. The grip stays, as
  claim 12 has no row for a target change." In its Confidence line, add: "low, and its backing
  `unmeasured`, for an infantry lifter's answers ahead of the gripped-unit *rearming* and the
  drop they cause, as S-0301 claim 9 rates them".

## Open questions

- Claim 9 leaves the firing-check answer label and rank during erasure open. Ordinary
  firing inactivity is established independently by S-0514 claim 17; it cannot distinguish
  a refused check from an attempt skipped entirely.
- Claim 9 lists only the members of *illegal* and *impossible* that matter here; the combat specs
  named there and the feature specs own the rest. Apart from the within-rank rules stated
  in claim 9, precedence within rank 4 is not stated.
- The amount of ammunition lost to damage (claim 17) is S-0302's.
- Aircraft ammunition spending and servicing at landing pads are specified by S-0501 claims
  15 and 26 to 28; starting ammunition is answered by claim 10 (Q-0060 question 1).
- Whether a temporal weapon's firing side restarts the rearm countdown when its beam lets go
  remains open. Beam release itself does not establish such a restart, and
  no release-time rearm rule is claimed here.
- Claim 13: once a running firing-frame counter has passed the sync frame *s* (after the sync shot,
  or when no shot was made in that tick), does an attempt with the primary weapon still get
  *rearming* from the gate until the counter is idle, or does the rearm countdown alone decide?
  Scenario F11's main part cannot tell the two apart while the rate-of-fire field is long; its
  beyond-the-start part, with a short field, can for a sync frame that is never reached.
- Claim 13: does the counter first fall in the turn of the shot that starts it or in the next
  one? The counter reaches the sync value after the same number of decrements either way,
  but their tick numbers and the tick at which the counter goes idle can differ by one.
  F11 records the counter at each firing attempt rather than assuming either decrement boundary.
- Claim 9, rank 16: which of the answers of claim 9 the locomotor's own refusal gives (S-0310,
  claim 7).
- Claim 24 (g): which of the two dot phases a given tick shows, and from which end the dots are
  counted, is not stated (low confidence); only the period and the swap every tick are.
- Claim 15: why every attempt of a stock tank given the spray flag and a burst-3 cannon was
  reported with no projectile in M-0004 is not known (reading those attempts as void shots is an
  inference). The same tank's shots with the same cannon and no spray flag were made. Until it is
  known, claim 15 states only what a void shot leaves unchanged, and the spray part of S-0313 has no
  observed shot that was made.
